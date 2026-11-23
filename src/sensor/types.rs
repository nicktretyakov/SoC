//! Value types shared by the sensor, ML and transport layers.

use crate::config::{LABEL_REST, LABEL_RUN, LABEL_WALK};
use serde::{Deserialize, Serialize};

/// One tri-axial accelerometer reading in units of *g* (9.81 m/s^2) plus the
/// boot-relative timestamp in milliseconds.
///
/// `Copy` so the pipeline can hand it around without allocation pressure.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AccelSample {
    pub ax_g: f32,
    pub ay_g: f32,
    pub az_g: f32,
    pub t_ms: u32,
}

impl AccelSample {
    /// Magnitude of the acceleration vector in g. For a device lying still this
    /// is ~1.0 (gravity); motion adds oscillation around that baseline.
    #[inline]
    pub fn magnitude(&self) -> f32 {
        (self.ax_g * self.ax_g + self.ay_g * self.ay_g + self.az_g * self.az_g).sqrt()
    }
}

/// The three activity classes this project recognises.
///
/// `code()` / `from_code()` map to the integer label columns smartcore expects
/// (decision trees operate on numeric labels, not strings).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    #[default]
    Rest,
    Walk,
    Run,
}

impl Activity {
    #[inline]
    pub fn label(&self) -> &'static str {
        match self {
            Activity::Rest => LABEL_REST,
            Activity::Walk => LABEL_WALK,
            Activity::Run => LABEL_RUN,
        }
    }

    #[inline]
    pub fn code(&self) -> i32 {
        match self {
            Activity::Rest => 0,
            Activity::Walk => 1,
            Activity::Run => 2,
        }
    }

    #[inline]
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Activity::Rest),
            1 => Some(Activity::Walk),
            2 => Some(Activity::Run),
            _ => None,
        }
    }
}

impl std::fmt::Display for Activity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magnitude_of_gravity() {
        let s = AccelSample {
            ax_g: 0.0,
            ay_g: 0.0,
            az_g: 1.0,
            t_ms: 0,
        };
        assert!((s.magnitude() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn activity_round_trip() {
        for a in [Activity::Rest, Activity::Walk, Activity::Run] {
            assert_eq!(Activity::from_code(a.code()), Some(a));
        }
        assert_eq!(Activity::from_code(42), None);
    }
}
