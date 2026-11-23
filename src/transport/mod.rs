//! Transport layer.
//!
//! - [`traits::DataSink`] is the abstraction (Dependency Inversion): the pipeline
//!   sends JSON to a `DataSink`, never to `HttpSink` directly.
//! - [`http::HttpSink`] POSTs over HTTP using `esp-idf-svc`.
//! - [`http::LoggingSink`] is a no-network fallback that just logs the payload;
//!   useful for bench testing without a server.
//! - [`wifi`] connects the ESP32 to an AP via `esp-idf-svc::wifi`.

pub mod http;
pub mod traits;
pub mod wifi;

pub use http::{HttpSink, LoggingSink};
pub use traits::DataSink;
