//! Transport abstraction.
//!
//! The pipeline calls `send(&str)` with a pre-serialised JSON payload. The
//! implementation decides where it goes: HTTP POST, MQTT, a serial log, ...

use crate::error::AppResult;

/// Anything that can ship a payload off-device. `Send` so it can live in the
/// network thread.
pub trait DataSink: Send {
    fn send(&mut self, payload: &str) -> AppResult<()>;
}

/// Blanket impl so `Box<dyn DataSink + Send>` can be used wherever a `DataSink`
/// is expected — lets `main` pick HTTP vs. logging at runtime.
impl<T: DataSink + ?Sized> DataSink for Box<T> {
    fn send(&mut self, payload: &str) -> AppResult<()> {
        (**self).send(payload)
    }
}
