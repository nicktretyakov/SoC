//! Feature extraction from a window of accelerometer samples.
//!
//! Each window is collapsed into a 5-dimensional feature vector:
//!
//! | idx | feature         | why it separates rest / walk / run              |
//! |-----|-----------------|------------------------------------------------|
//! | 0   | mean magnitude  | rest ~1.0 g; walk/run lift the average         |
//! | 1   | std magnitude   | rest ~0; walk moderate; run high               |
//! | 2   | max magnitude   | peak impact grows with intensity               |
//! | 3   | min magnitude   | deceleration dips below gravity when moving    |
//! | 4   | range (max-min) | overall swing amplitude                        |
//!
//! All math uses `f32` and the std `sqrt`; no floating-point surprises on
//! Xtensa because we are on the `std` target.

use crate::sensor::types::AccelSample;

/// Names of the features, in vector order. Used in the JSON payload and logs.
pub const FEATURE_NAMES: [&str; 5] = [
    "mean_mag",
    "std_mag",
    "max_mag",
    "min_mag",
    "range_mag",
];

/// Number of features produced per window. Kept as a `const` so callers can
/// size buffers without depending on `FEATURE_NAMES.len()`.
pub const N_FEATURES: usize = FEATURE_NAMES.len();

/// Stateless feature extractor. Being stateless it is trivially `Send + Sync`
/// and can be shared across threads by reference if ever needed.
#[derive(Clone, Copy, Debug, Default)]
pub struct FeatureExtractor;

impl FeatureExtractor {
    /// Collapse a window of samples into a feature vector.
    ///
    /// Returns a zero vector for an empty window rather than panicking, so a
    /// transient sensor hiccup cannot crash the ML thread.
    pub fn extract(window: &[AccelSample]) -> Vec<f32> {
        if window.is_empty() {
            return vec![0.0; N_FEATURES];
        }

        // Pre-compute the per-sample magnitude once.
        let mags: Vec<f32> = window.iter().map(|s| s.magnitude()).collect();

        let n = mags.len() as f32;
        let mean = mags.iter().sum::<f32>() / n;
        let variance = mags.iter().map(|m| (m - mean) * (m - mean)).sum::<f32>() / n;
        let std = variance.sqrt();
        let mut max = f32::MIN;
        let mut min = f32::MAX;
        for &m in &mags {
            if m > max {
                max = m;
            }
            if m < min {
                min = m;
            }
        }
        let range = max - min;

        // Order MUST match FEATURE_NAMES.
        vec![mean, std, max, min, range]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(ax: f32, ay: f32, az: f32) -> AccelSample {
        AccelSample {
            ax_g: ax,
            ay_g: ay,
            az_g: az,
            t_ms: 0,
        }
    }

    #[test]
    fn empty_window_is_zero() {
        let f = FeatureExtractor::extract(&[]);
        assert_eq!(f, vec![0.0; N_FEATURES]);
    }

    #[test]
    fn still_window_has_low_variance() {
        // gravity only, ±tiny noise
        let win: Vec<_> = (0..25)
            .map(|i| sample(0.0, 0.0, 1.0 + (i as f32) * 1e-4))
            .collect();
        let f = FeatureExtractor::extract(&win);
        assert!((f[0] - 1.0).abs() < 1e-3, "mean {}", f[0]);
        assert!(f[1] < 1e-3, "std {}", f[1]);
    }

    #[test]
    fn moving_window_has_higher_range() {
        // simulate 2g swings
        let win: Vec<_> = (0..25)
            .map(|i| {
                let v = if i % 2 == 0 { 2.0 } else { 0.0 };
                sample(0.0, 0.0, v)
            })
            .collect();
        let f = FeatureExtractor::extract(&win);
        assert!((f[4] - 2.0).abs() < 1e-3, "range {}", f[4]);
    }
}
