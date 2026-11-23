//! Hand-rolled k-nearest-neighbour classifier.
//!
//! Why write this instead of using smartcore's `KNNClassifier`? Two reasons:
//! 1. It demonstrates the k-NN algorithm explicitly (distance, sort, majority
//!    vote) as the task asks ("Реализован алгоритм k-NN").
//! 2. It has zero extra dependencies, so it compiles on the constrained ESP32
//!    target without pulling in a BLAS backend.
//!
//! The implementation is deliberately simple and `O(n·d)` per prediction: with
//! 18 training samples and 5 features this is ~90 flops, negligible vs. the
//! 50 Hz sample budget.

use std::cmp::Ordering;

use crate::error::{AppError, AppResult};
use crate::ml::dataset::Sample;
use crate::ml::traits::{Classifier, Prediction};
use crate::sensor::Activity;

pub struct KnnClassifier {
    samples: Vec<Vec<f32>>,
    labels: Vec<Activity>,
    k: usize,
}

impl KnnClassifier {
    /// Train = memorise the dataset (lazy learning). `k` is clamped to the
    /// dataset size so a too-large `k` cannot panic.
    pub fn new(dataset: &[Sample], k: usize) -> AppResult<Self> {
        if dataset.is_empty() {
            return Err(AppError::Ml("k-NN: empty training set".into()));
        }
        let (samples, labels): (Vec<Vec<f32>>, Vec<Activity>) = dataset
            .iter()
            .map(|s| (s.features.clone(), s.label))
            .unzip();
        Ok(Self {
            samples,
            labels,
            k: k.max(1),
        })
    }

    /// Euclidean distance between two equal-length feature vectors. Lengths are
    /// assumed equal because every training sample and every query come through
    /// `FeatureExtractor` which always emits `N_FEATURES`.
    #[inline]
    fn distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let d = x - y;
                d * d
            })
            .sum::<f32>()
            .sqrt()
    }
}

impl Classifier for KnnClassifier {
    fn name(&self) -> &'static str {
        "k-NN"
    }

    fn classify(&self, features: &[f32]) -> Prediction {
        // Pair each sample with its distance to the query, then sort ascending.
        let mut dists: Vec<(f32, Activity)> = self
            .samples
            .iter()
            .zip(self.labels.iter())
            .map(|(s, &l)| (Self::distance(s, features), l))
            .collect();
        dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

        let k = self.k.min(dists.len()).max(1);

        // Tally votes. Three classes => fixed-size array indexed by code().
        let mut votes = [0u32; 3];
        for (_, label) in dists.iter().take(k) {
            votes[label.code() as usize] += 1;
        }
        let (winner_code, winner_votes) = votes
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| *c)
            .map(|(i, c)| (i as i32, *c))
            .unwrap_or((0, 0));

        Prediction {
            activity: Activity::from_code(winner_code).unwrap_or(Activity::Rest),
            confidence: winner_votes as f32 / k as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::FEATURE_NAMES;

    fn clf() -> KnnClassifier {
        KnnClassifier::new(&crate::ml::dataset::build_dataset(), 5).unwrap()
    }

    #[test]
    fn rejects_empty_dataset() {
        assert!(KnnClassifier::new(&[], 5).is_err());
    }

    #[test]
    fn classifies_each_class_correctly() {
        let clf = clf();
        // rest-like window
        let rest = vec![1.0_f32, 0.02, 1.05, 0.95, 0.10];
        assert_eq!(clf.classify(&rest).activity, Activity::Rest);
        // walk-like window
        let walk = vec![1.1, 0.30, 1.80, 0.40, 1.40];
        assert_eq!(clf.classify(&walk).activity, Activity::Walk);
        // run-like window
        let run = vec![1.3, 0.60, 2.50, 0.10, 2.40];
        assert_eq!(clf.classify(&run).activity, Activity::Run);
    }

    #[test]
    fn feature_count_matches_names() {
        // sanity: dataset rows have as many features as the extractor emits
        let ds = crate::ml::dataset::build_dataset();
        assert_eq!(ds[0].features.len(), FEATURE_NAMES.len());
    }
}
