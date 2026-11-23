//! Sensor abstractions (the OOP interface).
//!
//! [`Sensor`] is the smallest, most focused trait (Interface Segregation): it
//! only knows how to read a single sample. [`SampleSource`] builds on it to
//! produce windows; the pipeline talks to whichever it needs.
//!
//! Both traits are declared `Send` so a sensor can be moved into its own thread.

use crate::error::AppResult;
use crate::sensor::types::AccelSample;

/// Anything that can produce a single accelerometer sample on demand.
pub trait Sensor: Send {
    fn read_accel(&mut self) -> AppResult<AccelSample>;
}

/// Anything that yields fixed-size windows of samples. Implemented by
/// [`crate::sensor::WindowedSampler`] but kept as its own trait so the pipeline
/// can depend on the abstraction, not the concrete wrapper.
pub trait SampleSource: Send {
    fn next_window(&mut self) -> AppResult<Vec<AccelSample>>;
}

/// Blanket implementation: a `Sensor` is automatically usable as a
/// `SampleSource` once wrapped by [`WindowedSampler`]. The wrapper itself
/// implements `SampleSource` directly (see `mpu6050.rs`/module body).
