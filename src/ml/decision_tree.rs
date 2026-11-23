//! Decision-tree classifier backed by `smartcore`.
//!
//! The tree is trained once at boot from the hard-coded dataset (see
//! [`crate::ml::dataset`]) and reused for every prediction. smartcore 0.3 is
//! pinned in `Cargo.toml`; if you bump the version, check that the
//! `DecisionTreeClassifier` type-parameter order and the `Parameters` struct
//! still match — they have changed between minor releases.

use smartcore::linalg::naive::dense_matrix::DenseMatrix;
use smartcore::tree::decision_tree_classifier::{
    Criterion, DecisionTreeClassifier, DecisionTreeClassifierParameters,
};

use crate::config;
use crate::error::{AppError, AppResult};
use crate::ml::dataset::Sample;
use crate::ml::traits::{Classifier, Prediction};
use crate::sensor::Activity;

/// Type alias for the concrete smartcore model so the struct field is readable.
/// `<f32>` features, `i32` labels, `DenseMatrix<f32>` storage.
type SmartcoreTree = DecisionTreeClassifier<f32, i32, DenseMatrix<f32>>;

pub struct DecisionTreeClassifierModel {
    model: SmartcoreTree,
}

impl DecisionTreeClassifierModel {
    /// Train the tree from a labelled dataset. Returns an error if the dataset
    /// is empty or if smartcore reports a fit failure.
    pub fn train(dataset: &[Sample]) -> AppResult<Self> {
        if dataset.is_empty() {
            return Err(AppError::Ml("decision tree: empty training set".into()));
        }
        let n_rows = dataset.len();
        let n_cols = dataset[0].features.len();

        // Flatten the dataset into a row-major Vec<f32>; smartcore copies it.
        let data: Vec<f32> = dataset.iter().flat_map(|s| s.features.iter().copied()).collect();
        let x = DenseMatrix::from_vec(n_rows, n_cols, &data);
        let y: Vec<i32> = dataset.iter().map(|s| s.label.code()).collect();

        let params = DecisionTreeClassifierParameters {
            min_samples_leaf: config::DT_MIN_SAMPLES_LEAF,
            min_samples_split: config::DT_MIN_SAMPLES_SPLIT,
            max_depth: Some(config::DT_MAX_DEPTH),
            criterion: Criterion::Gini,
            ..Default::default()
        };

        let model = DecisionTreeClassifier::fit(&x, &y, &params).map_err(AppError::ml)?;
        log::info!(
            "decision tree trained on {} samples x {} features",
            n_rows,
            n_cols
        );
        Ok(Self { model })
    }
}

impl Classifier for DecisionTreeClassifierModel {
    fn name(&self) -> &'static str {
        "decision-tree"
    }

    fn classify(&self, features: &[f32]) -> Prediction {
        // Wrap the single query as a 1-row matrix.
        let x = DenseMatrix::from_vec(1, features.len(), features);
        match self.model.predict(&x) {
            Ok(pred) => {
                let code = pred.first().copied().unwrap_or(0);
                Prediction {
                    activity: Activity::from_code(code).unwrap_or(Activity::Rest),
                    // smartcore 0.3 does not expose per-class probability for the
                    // tree, so we report a flat confident value.
                    confidence: 1.0,
                }
            }
            Err(e) => {
                log::warn!("decision tree predict failed: {e:?}");
                Prediction {
                    activity: Activity::Rest,
                    confidence: 0.0,
                }
            }
        }
    }
}
