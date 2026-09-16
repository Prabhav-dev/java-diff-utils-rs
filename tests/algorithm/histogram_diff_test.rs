//! Unit tests for HistogramDiff algorithm.

use java_diff_utils_rs::algorithm::{
    diff_algorithm_factory::{DiffAlgorithmFactory, HistogramDiffFactory},
    diff_algorithm_listener::DiffAlgorithmListener,
    histogram::histogram_diff::{
        compute_diff as compute_diff_histogram, compute_diff_with, HistogramDiff,
    },
    DiffAlgorithm,
};
use java_diff_utils_rs::patch::Patch;

#[derive(Default)]
struct HistogramLoggingListener {
    steps: Vec<(usize, usize)>,
    started: bool,
    ended: bool,
}

impl DiffAlgorithmListener for HistogramLoggingListener {
    fn diff_start(&mut self) {
        self.started = true;
    }

    fn diff_step(&mut self, value: usize, max: usize) {
        self.steps.push((value, max));
    }

    fn diff_end(&mut self) {
        self.ended = true;
    }
}

#[test]
fn test_histogram_simple_forward() {
    let original = vec!["A", "B", "C", "A", "B", "B", "A"];
    let revised = vec!["C", "B", "A", "B", "A", "C"];

    let changes = compute_diff_histogram(&original, &revised);
    let patch = Patch::generate(&original, &revised, &changes, false);

    assert!(!patch.deltas().is_empty());
    let patched = patch.apply_to(&original).expect("Patch apply failed");
    assert_eq!(patched, revised);
}

#[test]
fn test_histogram_with_listener() {
    let original = vec!["alpha", "beta", "gamma", "delta"];
    let revised = vec!["alpha", "beta_new", "gamma", "delta", "epsilon"];

    let mut listener = HistogramLoggingListener::default();
    let algo = HistogramDiff::<&str>::new();
    let changes = algo.diff_with_listener(&original, &revised, &mut listener);

    assert!(listener.started);
    assert!(listener.ended);
    let patch = Patch::generate(&original, &revised, &changes, false);
    let patched = patch.apply_to(&original).expect("Patch apply failed");
    assert_eq!(patched, revised);
}

#[test]
fn test_histogram_patch_apply_and_restore() {
    let source = vec!["line 1", "line 2", "line 3", "line 4"];
    let target = vec!["line 1", "modified line 2", "line 3", "line 4", "line 5"];

    let algo = HistogramDiff::<&str>::new();
    let changes = algo.diff(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);

    let restored = patch.restore(&target).expect("Restore failed");
    assert_eq!(restored, source);
}

#[test]
fn test_histogram_identical_sequences() {
    let data = vec!["unchanged 1", "unchanged 2", "unchanged 3"];
    let changes = compute_diff_histogram(&data, &data);
    assert!(changes.is_empty());

    let patch = Patch::generate(&data, &data, &changes, false);
    assert!(patch.deltas().is_empty());
}

#[test]
fn test_histogram_pure_insertion() {
    let source: Vec<&str> = vec![];
    let target = vec!["new 1", "new 2", "new 3"];

    let changes = compute_diff_histogram(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    assert_eq!(patch.deltas().len(), 1);
    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);
}

#[test]
fn test_histogram_pure_deletion() {
    let source = vec!["old 1", "old 2", "old 3"];
    let target: Vec<&str> = vec![];

    let changes = compute_diff_histogram(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    assert_eq!(patch.deltas().len(), 1);
    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);
}

#[test]
fn test_histogram_with_custom_equalizer() {
    let source = vec!["Hello", "WORLD"];
    let target = vec!["hello", "world"];

    // Case-insensitive equalizer
    let changes = compute_diff_with(&source, &target, |a, b| a.eq_ignore_ascii_case(b));
    assert!(
        changes.is_empty(),
        "Should be considered equal under custom equalizer"
    );
}

#[test]
fn test_histogram_fallback_on_repeated_elements() {
    // When elements are repeated beyond max_chain_length (e.g. max_chain_length = 2),
    // histogram diff falls back to Myers linear space algorithm seamlessly.
    let source = vec!["x", "x", "x", "x", "x", "a", "x", "x"];
    let target = vec!["x", "x", "x", "x", "x", "b", "x", "x"];

    let algo = HistogramDiff::<&str>::new().with_max_chain_length(2);
    let changes = algo.diff(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);
}

#[test]
fn test_histogram_factory() {
    let factory = HistogramDiffFactory::new();
    let algo = factory.create();

    let source = vec!["foo".to_string(), "bar".to_string()];
    let target = vec!["foo".to_string(), "baz".to_string()];

    let changes = algo.diff(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);
    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);
}
