//! Crate-wide error type.
//!
//! A single enum keeps the API simple and lets the pipeline use `?` across I2C,
//! Wi-Fi, HTTP and ML call sites without leaking third-party error types.

use std::fmt;

/// All recoverable failures in the application fall into one of these buckets.
#[derive(Debug)]
pub enum AppError {
    I2c(String),
    Wifi(String),
    Http(String),
    Ml(String),
    Sensor(String),
    Config(String),
    Generic(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::I2c(m) => write!(f, "I2C error: {m}"),
            AppError::Wifi(m) => write!(f, "Wi-Fi error: {m}"),
            AppError::Http(m) => write!(f, "HTTP error: {m}"),
            AppError::Ml(m) => write!(f, "ML error: {m}"),
            AppError::Sensor(m) => write!(f, "sensor error: {m}"),
            AppError::Config(m) => write!(f, "config error: {m}"),
            AppError::Generic(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Generic(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Generic(format!("JSON: {e}"))
    }
}

impl AppError {
    /// Convenience constructor used by sensor/transport code to wrap a debug
    /// string from an `EspError`-carrying result.
    pub fn i2c<E: fmt::Debug>(e: E) -> Self {
        AppError::I2c(format!("{e:?}"))
    }
    pub fn wifi<E: fmt::Debug>(e: E) -> Self {
        AppError::Wifi(format!("{e:?}"))
    }
    pub fn http<E: fmt::Debug>(e: E) -> Self {
        AppError::Http(format!("{e:?}"))
    }
    pub fn ml<E: fmt::Debug>(e: E) -> Self {
        AppError::Ml(format!("{e:?}"))
    }
}
