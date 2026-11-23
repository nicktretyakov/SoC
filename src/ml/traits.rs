//! ML abstractions.
//!
//! [`Classifier`] is the polymorphism point of the whole crate: the pipeline
//! depends on it, never on `KnnClassifier` or `DecisionTreeClassifierModel`
//! directly (Dependency Inversion). The blanket impl for `Box<T>` lets a
//! boxed trait object (`Box<dyn Classifier + Send>`) be used wherever a
//! `Classifier` is expected.

use crate::sensor::Activity;

/// One model prediction: the chosen activity plus a confidence in `[0, 1]`.
///
/// Confidence is computed differently per model:
/// - k-NN: fraction of the `k` neighbours that voted for the winner.
/// - decision tree: `1.0` (smartcore 0.3 does not expose per-class probability
///   for the tree; we report a flat confident value rather than invent a number).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prediction {
    pub activity: Activity,
    pub confidence: f32,
}

/// The full result emitted by the ML thread to the network thread. Bundles the
/// prediction with the inputs that produced it, so the server can store a
/// reproducible record.
#[derive(Clone, Debug)]
pub struct Classification {
    pub activity: Activity,
    pub confidence: f32,
    pub features: Vec<f32>,
    pub samples: usize,
    pub timestamp_ms: u32,
}

impl Classification {
    pub fn new(prediction: Prediction, features: Vec<f32>, samples: usize, ts_ms: u32) -> Self {
        Self {
            activity: prediction.activity,
            confidence: prediction.confidence,
            features,
            samples,
            timestamp_ms: ts_ms,
        }
    }
}

/// The classifier interface. `Send` so a model can be moved into the ML thread.
pub trait Classifier: Send {
    /// Map a feature vector to a predicted activity + confidence.
    fn classify(&self, features: &[f32]) -> Prediction;

    /// Short human-readable id, used only for logs.
    fn name(&self) -> &'static str;
}

/// Blanket impl so `Box<dyn Classifier + Send>` (and `Box<T>` for any concrete
/// `T: Classifier`) can be passed wherever a `Classifier` is expected. This is
/// what lets `main.rs` choose the model at runtime via a trait object.
impl<T: Classifier + ?Sized> Classifier for Box<T> {
    fn classify(&self, features: &[f32]) -> Prediction {
        (**self).classify(features)
    }
    fn name(&self) -> &'static str {
        (**self).name()
    }
}
