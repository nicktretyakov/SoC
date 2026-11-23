//! Sensor layer.
//!
//! Responsibilities:
//! - define the [`Sensor`] and [`crate::sensor::SampleSource`] abstractions (OOP interface),
//! - implement the MPU6050 driver over I2C,
//! - provide a `WindowedSampler` that turns a raw `Sensor` into a stream of windows.
//!
//! The I2C implementation detail is confined to [`mpu6050`]; everything above it
//! depends only on the [`Sensor`] trait (Dependency Inversion).

pub mod mpu6050;
pub mod sampler;
pub mod traits;
pub mod types;

pub use mpu6050::Mpu6050;
pub use sampler::WindowedSampler;
pub use traits::{SampleSource, Sensor};
pub use types::{AccelSample, Activity};
