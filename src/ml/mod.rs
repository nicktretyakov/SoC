//! Machine-learning layer.
//!
//! Two interchangeable implementations behind one [`Classifier`] trait:
//! - [`knn::KnnClassifier`] — a hand-rolled k-nearest-neighbour classifier
//!   (Euclidean distance + majority vote).
//! - [`decision_tree::DecisionTreeClassifierModel`] — wraps a decision tree
//!   trained with the `smartcore` crate.
//!
//! Open/Closed: adding a third model (SVM, logistic regression, ...) means a new
//! struct that implements `Classifier`; nothing else in the crate changes.

pub mod dataset;
pub mod decision_tree;
pub mod knn;
pub mod traits;

pub use dataset::Sample;
pub use decision_tree::DecisionTreeClassifierModel;
pub use knn::KnnClassifier;
pub use traits::{Classification, Classifier, Prediction};
