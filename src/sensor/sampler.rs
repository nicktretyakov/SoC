//! `WindowedSampler` — turns a raw [`Sensor`] into a stream of fixed-size
//! windows. Keeps the timing/windowing concern out of the driver and out of the
//! pipeline (Single Responsibility), and lets the pipeline depend on the
//! [`SampleSource`] abstraction rather than on `Mpu6050` directly.

use std::time::Duration;

use crate::error::AppResult;
use crate::sensor::traits::{SampleSource, Sensor};
use crate::sensor::types::AccelSample;
use crate::tag_log;

pub struct WindowedSampler<S: Sensor> {
    sensor: S,
    window: Vec<AccelSample>,
    window_size: usize,
    period: Duration,
}

impl<S: Sensor> WindowedSampler<S> {
    /// Build a sampler that emits a window every `window_size` samples taken at
    /// `hz` Hz. `window` is pre-allocated once and reused.
    pub fn new(sensor: S, window_size: usize, hz: u32) -> Self {
        let period = if hz == 0 {
            Duration::from_millis(20)
        } else {
            Duration::from_millis(1000 / hz as u64)
        };
        Self {
            sensor,
            window: Vec::with_capacity(window_size),
            window_size,
            period,
        }
    }
}

impl<S: Sensor> SampleSource for WindowedSampler<S> {
    fn next_window(&mut self) -> AppResult<Vec<AccelSample>> {
        self.window.clear();
        while self.window.len() < self.window_size {
            match self.sensor.read_accel() {
                Ok(sample) => self.window.push(sample),
                Err(e) => {
                    // Transient I2C glitches happen: log and back off, but keep
                    // the partial window rather than dropping it silently.
                    tag_log!("sensor", "read error: {e}; retrying");
                    std::thread::sleep(self.period / 2);
                }
            }
            std::thread::sleep(self.period);
        }
        Ok(std::mem::take(&mut self.window))
    }
}
