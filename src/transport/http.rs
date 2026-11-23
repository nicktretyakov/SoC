//! HTTP and logging sinks.
//!
//! `HttpSink` POSTs JSON to `SERVER_URL` using `esp-idf-svc::http::client`.
//! `LoggingSink` is a no-network fallback that just writes the payload to the
//! log — handy when no server is available (e.g. on the bench). Both implement
//! `DataSink`, so the pipeline is agnostic to which one it holds.

use std::time::Duration;

use esp_idf_svc::http::client::{Configuration, EspHttpConnection, HttpClient};

use crate::error::{AppError, AppResult};
use crate::tag_log;
use crate::transport::traits::DataSink;

/// HTTP POST sink. Owns an `HttpClient` (which owns its connection + buffers),
/// so repeated POSTs reuse the same allocation.
pub struct HttpSink {
    client: HttpClient<'static>,
    url: String,
}

impl HttpSink {
    pub fn new(url: &str) -> AppResult<Self> {
        // `EspHttpConnection::new` borrows the `Configuration`; depending on
        // the esp-idf-svc version the returned connection carries that
        // lifetime. We leak the config once (a few dozen bytes, never freed)
        // so the connection — and therefore the `HttpClient` we store in this
        // struct — is `HttpClient<'static>` regardless of the exact signature.
        let config: &'static Configuration = Box::leak(Box::new(Configuration {
            buffer_size: 1024,
            buffer_size_large: 4096,
            timeout: Some(Duration::from_secs(15)),
            ..Default::default()
        }));
        let conn = EspHttpConnection::new(config).map_err(AppError::http)?;
        let client = HttpClient::wrap(conn);
        log::info!("HTTP sink ready, endpoint = {url}");
        Ok(Self {
            client,
            url: url.to_string(),
        })
    }
}

impl DataSink for HttpSink {
    fn send(&mut self, payload: &str) -> AppResult<()> {
        const HEADERS: [(&str, &str); 2] = [
            ("Content-Type", "application/json"),
            ("Connection", "keep-alive"),
        ];

        let mut request = self
            .client
            .post(&self.url, Some(&HEADERS))
            .map_err(AppError::http)?;

        request.write(payload.as_bytes()).map_err(AppError::http)?;
        request.finish().map_err(AppError::http)?;

        let response = request.submit().map_err(AppError::http)?;
        let status = response.status();
        if status >= 400 {
            return Err(AppError::Http(format!(
                "server returned HTTP {status} for POST {url}",
                url = self.url
            )));
        }
        tag_log!("net", "POST {} -> HTTP {}", self.url, status);
        Ok(())
    }
}

/// No-network sink: logs every payload at INFO. Useful for development and for
/// the case where Wi-Fi associates but no server is reachable yet.
#[derive(Default)]
pub struct LoggingSink;

impl LoggingSink {
    pub fn new() -> Self {
        Self
    }
}

impl DataSink for LoggingSink {
    fn send(&mut self, payload: &str) -> AppResult<()> {
        tag_log!("net", "payload: {payload}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_sink_always_succeeds() {
        let mut s = LoggingSink::new();
        assert!(s.send("{\"x\":1}").is_ok());
    }
}
