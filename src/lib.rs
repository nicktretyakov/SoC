//! # esp32-ml-activity
//!
//! Activity recognition for ESP32 + MPU6050.
//!
//! Pipeline (3 threads on FreeRTOS via `std::thread::scope`):
//!
//! ```text
//!   [sensor thread]  Mpu6050 --I2C--> Vec<AccelSample> window
//!        |  mpsc::sync_channel
//!        v
//!   [ml thread]      FeatureExtractor --> Classifier (k-NN | decision tree)
//!        |  mpsc::sync_channel
//!        v
//!   [net thread]     JSON --> HTTP POST over Wi-Fi
//! ```
//!
//! Design notes:
//! - **OOP** via traits (`Sensor`, `Classifier`, `DataSink`) and structs that implement them.
//! - **SOLID**: SRP (one responsibility per module), OCP (new sensors/classifiers/sinks plug in
//!   via traits without touching the pipeline), LSP (any `Classifier` substitutes), ISP
//!   (small focused traits), DIP (the pipeline depends on traits, never on concretes).
//! - **Multithreading**: three scoped threads + bounded `mpsc` channels (back-pressure).
//! - **Crates**: `esp-idf-hal`, `esp-idf-svc`, `esp-idf-sys`, `smartcore`, `serde`, `serde_json`, `log`.
//! - **Macros**: custom `activity_label!`, `features!`, `sample!`, `tag_log!`, plus `serde_json::json!`.
//! - **ML**: hand-rolled k-NN and a smartcore decision-tree, both behind one `Classifier` trait.

#![allow(dead_code)]

pub mod config;
pub mod error;
pub mod macros;
pub mod util;

pub mod sensor;
pub mod features;
pub mod ml;
pub mod transport;
pub mod pipeline;

// Re-export the most-used types so `main.rs` and tests can write `crate::*`.
pub use error::{AppError, AppResult};
pub use sensor::{AccelSample, Activity, Mpu6050, Sensor, WindowedSampler};
pub use features::FeatureExtractor;
pub use ml::{Classification, Classifier, DecisionTreeClassifierModel, KnnClassifier, Prediction};
pub use transport::{DataSink, HttpSink, LoggingSink};
