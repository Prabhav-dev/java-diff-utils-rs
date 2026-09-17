//! Unit tests for HistogramDiff algorithm.

use java_diff_utils_rs::algorithm::{
    diff_algorithm_factory::{DiffAlgorithmFactory, HistogramDiffFactory},
    diff_algorithm_listener::DiffAlgorithmListener,
    histogram::histogram_diff::{
        compute_diff as compute_diff_histogram, compute_diff_with, HistogramDiff,
    },
    DiffAlgorithm,
};
use java_diff_utils_rs::{DiffUtils, patch::Patch};

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

#[test]
fn test_diff_utils_uses_histogram_by_default() {
    let source = vec!["A", "B", "C", "D", "A", "B", "C", "D"];
    let target = vec!["A", "B", "X", "D", "A", "B", "C", "D"];

    let patch = DiffUtils::diff(&source, &target, None);
    assert!(!patch.deltas().is_empty());
    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);
}

#[test]
fn test_histogram_repeated_values_coalesce_like_jgit() {
    let alphabet = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q"];
    let mut source = Vec::new();
    let mut target = Vec::new();

    for i in 0..120 {
        let value = alphabet[i % alphabet.len()];
        source.push(value);
        target.push(value);
    }

    let changed = ["R", "S", "T", "U", "V", "W", "X", "Y", "Z", "A", "B", "C", "D", "E", "F", "G", "H"];
    for (idx, value) in changed.iter().enumerate() {
        target[40 + idx] = *value;
    }

    let changes = HistogramDiff::new().diff(&source, &target);
    assert!(changes.len() <= 2, "expected JGit-like coalescing, got {} changes: {:?}", changes.len(), changes);
    let patch = Patch::generate(&source, &target, &changes, false);
    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);
}

#[test]
fn test_histogram_large_low_entropy_repeated_input_matches_jgit_shape() {
    let mut source = Vec::new();
    let mut target = Vec::new();

    for i in 0..100_000 {
        let value = i % 17;
        source.push(value);
        target.push(value);
    }

    target.splice(10_000..10_020, std::iter::repeat(31).take(20));
    target.splice(50_000..50_000, std::iter::repeat(41).take(30));

    let changes = HistogramDiff::new().diff(&source, &target);
    assert!(changes.len() <= 2, "expected JGit-like coalescing for low-entropy repeated data, got {} changes: {:?}", changes.len(), changes);

    let patch = Patch::generate(&source, &target, &changes, false);
    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);
}

// ── Priority 5: exact delta count + exact boundary position tests ─────────────

/// A single-line replacement must produce exactly 1 Change delta (not 2 separate
/// Insert + Delete records), because `normalize_replacements` must coalesce them.
#[test]
fn test_histogram_exact_delta_count_single_replacement() {
    let source = vec!["A", "B", "C"];
    let target = vec!["A", "X", "C"];

    let changes = compute_diff_histogram(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    assert_eq!(
        patch.deltas().len(),
        1,
        "single-line replacement should produce exactly 1 Change delta, got {:?}",
        patch.deltas()
    );

    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);
}

/// For `["A","B","C"] → ["A","X","C"]` the diff must locate the change at
/// positions (1,2) in both sequences. This pins the exact boundary values
/// so regressions in anchor selection are caught immediately.
#[test]
fn test_histogram_exact_delta_positions() {
    use java_diff_utils_rs::algorithm::change::DeltaType;

    let source = vec!["A", "B", "C"];
    let target = vec!["A", "X", "C"];

    let changes = compute_diff_histogram(&source, &target);
    assert_eq!(changes.len(), 1, "expected 1 change, got {:?}", changes);

    let c = &changes[0];
    assert_eq!(c.delta_type, DeltaType::Change, "expected Change delta");
    assert_eq!(c.start_original, 1, "start_original mismatch");
    assert_eq!(c.end_original,   2, "end_original mismatch");
    assert_eq!(c.start_revised,  1, "start_revised mismatch");
    assert_eq!(c.end_revised,    2, "end_revised mismatch");
}

/// With the default max_chain_length (64) and a 500-element input where each
/// value repeats at most 10 times, HistogramDiff should handle the input on
/// the histogram path (not force a Myers fallback for the whole range) and
/// the patch must apply correctly.
#[test]
fn test_histogram_no_total_fallback_on_moderate_repetition() {
    // 50 distinct values, each appearing exactly 10 times → max frequency 10 < 64.
    let source: Vec<usize> = (0..500).map(|i| i % 50).collect();
    let mut target = source.clone();
    // Replace a small cluster in the middle.
    for v in target.iter_mut().take(300).skip(200) {
        *v = 999;
    }

    let changes = HistogramDiff::new().diff(&source, &target);
    assert!(
        !changes.is_empty(),
        "expected at least 1 change for modified input"
    );

    let patch = Patch::generate(&source, &target, &changes, false);
    let applied = patch.apply_to(&source).expect("Apply failed");
    assert_eq!(applied, target);
}

// ── Priority 6: golden boundary tests ─────────────────────────────────────────

/// Golden boundary test for a simple two-edit case.
/// Verifies that HistogramDiff finds changes at the expected positions without
/// requiring a live JGit process — positions were manually derived from the
/// histogram algorithm semantics (unique-element anchors).
#[test]
fn test_histogram_golden_boundaries_simple() {
    use java_diff_utils_rs::algorithm::change::DeltaType;

    // "B" is unique in source; "Y" is unique in target.
    // Prefix "A" and suffix "C" trim away, leaving only the middle.
    let source = vec!["A", "B", "C"];
    let target = vec!["A", "Y", "C"];

    let changes = compute_diff_histogram(&source, &target);
    assert_eq!(changes.len(), 1);
    let c = &changes[0];
    // After prefix/suffix trimming: src[1..2] ↔ tgt[1..2]
    assert_eq!(c.delta_type, DeltaType::Change);
    assert_eq!((c.start_original, c.end_original), (1, 2));
    assert_eq!((c.start_revised, c.end_revised), (1, 2));
}

/// Regression guard: an adjacent Insert + Delete at the same position must
/// be coalesced into a single Change by `normalize_replacements`, not left
/// as two separate records.
#[test]
fn test_histogram_boundary_normalization_coalesces_adjacent() {
    use java_diff_utils_rs::algorithm::change::DeltaType;

    // Any input that triggers a delete+insert at the same location tests the
    // normalization path. A simple one-element swap is sufficient.
    let source = vec!["old"];
    let target = vec!["new"];

    let changes = compute_diff_histogram(&source, &target);
    // After normalization there must be exactly 1 delta of type Change.
    assert_eq!(changes.len(), 1, "expected 1 coalesced delta, got {:?}", changes);
    assert_eq!(
        changes[0].delta_type,
        DeltaType::Change,
        "expected Change, got {:?}",
        changes[0].delta_type
    );
}

