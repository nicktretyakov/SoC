//! Hard-coded training dataset.
//!
//! Each row is one labelled feature vector in the same order as
//! [`crate::features::FEATURE_NAMES`]:
//! `[mean_mag, std_mag, max_mag, min_mag, range_mag]`.
//!
//! Values are typical of a wrist-mounted MPU6050 at the default +/-2 g range:
//! - **rest**: mean ~1.0 g (gravity), very low variance, tight range.
//! - **walk**: mean ~1.1 g, moderate variance (~0.3 g), range ~1.3 g.
//! - **run**:  mean ~1.3 g, high variance (~0.6 g), range ~2.4 g.
//!
//! Six samples per class (18 total) — small but enough for a demo. In a real
//! deployment you would collect these with a calibration mode and persist them
//! to NVS; the `Sample` struct and the `Classifier` API stay the same.

use crate::sensor::Activity;

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub features: Vec<f32>,
    pub label: Activity,
}

impl Sample {
    pub fn new(label: Activity, features: Vec<f32>) -> Self {
        Self { features, label }
    }
}

/// Build the training set. Declared via the `sample!` macro so the intent
/// ("label ; features") stays readable.
pub fn build_dataset() -> Vec<Sample> {
    vec![
        // ---- rest -----------------------------------------------------------
        sample!(Activity::Rest; 1.00, 0.02, 1.05, 0.95, 0.10),
        sample!(Activity::Rest; 1.01, 0.03, 1.06, 0.94, 0.12),
        sample!(Activity::Rest; 0.99, 0.01, 1.03, 0.97, 0.06),
        sample!(Activity::Rest; 1.00, 0.02, 1.04, 0.96, 0.08),
        sample!(Activity::Rest; 1.02, 0.04, 1.07, 0.93, 0.14),
        sample!(Activity::Rest; 0.98, 0.02, 1.02, 0.96, 0.06),
        // ---- walk -----------------------------------------------------------
        sample!(Activity::Walk; 1.10, 0.30, 1.80, 0.40, 1.40),
        sample!(Activity::Walk; 1.08, 0.25, 1.65, 0.45, 1.20),
        sample!(Activity::Walk; 1.15, 0.35, 1.95, 0.35, 1.60),
        sample!(Activity::Walk; 1.05, 0.28, 1.70, 0.42, 1.28),
        sample!(Activity::Walk; 1.12, 0.32, 1.85, 0.38, 1.47),
        sample!(Activity::Walk; 1.09, 0.27, 1.72, 0.44, 1.28),
        // ---- run ------------------------------------------------------------
        sample!(Activity::Run; 1.30, 0.60, 2.50, 0.10, 2.40),
        sample!(Activity::Run; 1.25, 0.55, 2.40, 0.12, 2.28),
        sample!(Activity::Run; 1.35, 0.70, 2.70, 0.08, 2.62),
        sample!(Activity::Run; 1.22, 0.52, 2.35, 0.15, 2.20),
        sample!(Activity::Run; 1.40, 0.72, 2.65, 0.05, 2.60),
        sample!(Activity::Run; 1.28, 0.58, 2.45, 0.11, 2.34),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_is_balanced_and_non_empty() {
        let ds = build_dataset();
        assert!(!ds.is_empty());
        let (mut rest, mut walk, mut run) = (0, 0, 0);
        for s in &ds {
            match s.label {
                Activity::Rest => rest += 1,
                Activity::Walk => walk += 1,
                Activity::Run => run += 1,
            }
            assert!(!s.features.is_empty());
        }
        assert_eq!((rest, walk, run), (6, 6, 6));
    }
}
