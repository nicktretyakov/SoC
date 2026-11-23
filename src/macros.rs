//! Declarative macros used across the crate.
//!
//! These exist to show idiomatic `macro_rules!` usage (the "макросы" requirement):
//! declaring labelled constants, building feature vectors, declaring training
//! samples, and a thin logging helper that tags every line with a module name.

/// Declares a `pub const &str` for an activity label.
///
/// ```ignore
/// activity_label!(LABEL_REST, "rest");
/// ```
#[macro_export]
macro_rules! activity_label {
    ($name:ident, $value:expr) => {
        pub const $name: &str = $value;
    };
}

/// Builds an owned `Vec<f32>` feature vector from arbitrary numeric expressions.
/// Every element is coerced to `f32`, so callers may mix `i32`, `f32`, etc.
///
/// ```ignore
/// let v = features![1.0, 0.2, 1.8, 0.4, 1.4];
/// ```
#[macro_export]
macro_rules! features {
    ($($x:expr),+ $(,)?) => {
        vec![$($x as f32),+]
    };
    () => { vec![] };
}

/// Declares one labelled training sample for the ML dataset.
///
/// ```ignore
/// sample!(Activity::Rest; 1.00, 0.02, 1.05, 0.95, 0.10)
/// ```
#[macro_export]
macro_rules! sample {
    ($label:expr; $($x:expr),+ $(,)?) => {
        $crate::ml::dataset::Sample::new($label, vec![$($x as f32),+])
    };
}

/// Tagged logging shortcut. Expands to `log::info!(target: $tag, ..)`.
///
/// ```ignore
/// tag_log!("sensor", "window full: {} samples", n);
/// ```
#[macro_export]
macro_rules! tag_log {
    ($tag:expr, $($arg:tt)*) => {
        ::log::info!(target: $tag, $($arg)*)
    };
}

#[cfg(test)]
mod macro_tests {
    #[test]
    fn features_macro_builds_vec() {
        let v: Vec<f32> = crate::features![1, 2.0, 3];
        assert_eq!(v, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn features_macro_empty() {
        let v: Vec<f32> = crate::features![];
        assert!(v.is_empty());
    }
}
