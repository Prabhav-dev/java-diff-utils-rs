// tests/integration_tests.rs

//! Comprehensive Integration & System Test Suite for `my_diff_crate`

use my_diff_crate::algorithm::{
    change::{Change, DeltaType},
    diff_algorithm::DiffAlgorithm,
    diff_algorithm_factory::{DiffAlgorithmFactory, MyersDiffFactory},
    diff_algorithm_listener::DiffAlgorithmListener,
    myers::{
        compute_diff as compute_diff_linear, compute_diff_full, compute_diff_with,
        myers::MyersDiff, path_node::PathNode, LinearWorkspace,
    },
};
use my_diff_crate::patch::{
    chunk::Chunk,
    delta::Delta,
    delta_type::DeltaType as PatchDeltaType,
    patch_failed_exception::PatchFailedException,
    verify_chunk::VerifyChunk,
    Patch,
};

use my_diff_crate::text::{
    delta_merge::{
        delta_merge_utils::DeltaMergeUtils, inline_delta_merge_info::InlineDeltaMergeInfo,
    },
    diff_row_generator::DiffRowGenerator,
    string_utils,
};
use my_diff_crate::UnifiedDiffUtils;

// =========================================================================
// 1. Change Struct & PathNode Unit Integrations
// =========================================================================

#[test]
fn test_change_creation_and_mutators() {
    let change = Change::new(DeltaType::Insert, 0, 1, 0, 1);
    assert_eq!(change.end_original, 1);
    assert_eq!(change.end_revised, 1);

    let mut updated_orig = change;
    updated_orig.end_original = 5;
    assert_eq!(updated_orig.end_original, 5);
    assert_eq!(updated_orig.end_revised, 1);

    let mut updated_rev = change;
    updated_rev.end_revised = 10;
    assert_eq!(updated_rev.end_original, 1);
    assert_eq!(updated_rev.end_revised, 10);
}

#[test]
fn test_path_node_bootstrap() {
    let root = PathNode::new(0, -1, true, true, None);

    assert!(root.is_bootstrap);
    assert!(root.is_snake);
    assert_eq!(root.prev, None);
}

#[test]
fn test_path_node_previous_snake_traversal() {
    let mut arena = Vec::new();

    // Index 0: Root bootstrap node
    let root_idx = arena.len();
    arena.push(PathNode::new(0, -1, true, true, None));

    // Index 1: Non-snake edit step
    let edit1_idx = arena.len();
    arena.push(PathNode::new(0, 0, false, false, Some(root_idx)));

    // Index 2: Snake match step
    let snake1_idx = arena.len();
    arena.push(PathNode::new(1, 1, true, false, Some(edit1_idx)));

    // Index 3: Non-snake edit step
    let edit2_idx = arena.len();
    arena.push(PathNode::new(1, 2, false, false, Some(snake1_idx)));

    // Traversal test: previous snake from edit2 should skip back to snake1 (idx 2)
    let found = PathNode::previous_snake(&arena, edit2_idx);
    assert_eq!(found, Some(snake1_idx));

    // Traversal test: previous snake from root should return None
    let root_found = PathNode::previous_snake(&arena, root_idx);
    assert_eq!(root_found, None);
}

#[test]
fn test_path_node_formatting() {
    let mut arena = Vec::new();

    let root_idx = arena.len();
    arena.push(PathNode::new(0, -1, true, true, None));

    let step1_idx = arena.len();
    arena.push(PathNode::new(1, 0, false, false, Some(root_idx)));

    let formatted = PathNode::fmt_path(&arena, step1_idx);
    assert_eq!(formatted, "[(1,0), (0,-1)]");
}

// =========================================================================
// 2. Listener Lifecycle & Factory Integrations
// =========================================================================

struct MockListener {
    started: bool,
    steps: Vec<(usize, usize)>,
    ended: bool,
}

impl MockListener {
    fn new() -> Self {
        Self {
            started: false,
            steps: Vec::new(),
            ended: false,
        }
    }
}

impl DiffAlgorithmListener for MockListener {
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
fn test_listener_lifecycle() {
    let mut listener = MockListener::new();

    listener.diff_start();
    listener.diff_step(5, 100);
    listener.diff_step(50, 100);
    listener.diff_end();

    assert!(listener.started);
    assert_eq!(listener.steps, vec![(5, 100), (50, 100)]);
    assert!(listener.ended);
}

struct DummyAlgorithm;

impl<T: PartialEq> DiffAlgorithm<T> for DummyAlgorithm {
    fn diff_with_listener(
        &self,
        source: &[T],
        target: &[T],
        _listener: &mut dyn DiffAlgorithmListener, // <--- Matching trait signature
    ) -> Vec<Change> {
        if source == target {
            vec![]
        } else {
            vec![Change::new(
                DeltaType::Change,
                0,
                source.len(),
                0,
                target.len(),
            )]
        }
    }
}

#[test]
fn test_diff_algorithm_trait_flexibility() {
    let algo = DummyAlgorithm;

    let source_vec = vec!["a", "b"];
    let target_vec = vec!["a", "c"];
    let changes = algo.diff(&source_vec, &target_vec);
    assert_eq!(changes.len(), 1);

    let source_arr = [1, 2, 3];
    let target_arr = [1, 2, 3];
    let changes_arr = algo.diff(&source_arr, &target_arr);
    assert!(changes_arr.is_empty());
}

#[test]
fn test_closure_blanket_implementation() {
    let simple_diff = |a: &[i32], b: &[i32]| -> Vec<Change> {
        if a == b {
            vec![]
        } else {
            vec![Change::new(DeltaType::Change, 0, a.len(), 0, b.len())]
        }
    };

    let res = simple_diff.diff(&[1, 2], &[3, 4]);
    assert_eq!(res.len(), 1);
}

#[test]
fn test_myers_factory_default_equality() {
    let factory = MyersDiffFactory;
    let algo = factory.create();

    let source = vec!["A", "B", "C"];
    let target = vec!["A", "C"];

    let changes = algo.diff(&source, &target);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].start_original, 1);
}

#[test]
fn test_myers_factory_custom_equalizer() {
    let factory = MyersDiffFactory;
    let algo = factory.create_with_equalizer(Box::new(|a: &&str, b: &&str| {
        a.eq_ignore_ascii_case(b)
    }));

    let source = vec!["apple", "BANANA"];
    let target = vec!["APPLE", "banana"];

    let changes = algo.diff(&source, &target);
    assert!(changes.is_empty());
}

// =========================================================================
// 3. Myers Diff Engine Edge Cases
// =========================================================================

#[test]
fn test_myers_empty_inputs() {
    let source: Vec<i32> = vec![];
    let target: Vec<i32> = vec![];
    let changes = compute_diff_linear(&source, &target);
    assert!(changes.is_empty());
}

#[test]
fn test_myers_identical_inputs() {
    let source = vec!["A", "B", "C"];
    let target = vec!["A", "B", "C"];
    let changes = compute_diff_linear(&source, &target);
    assert!(changes.is_empty());
}

#[test]
fn test_myers_pure_deletion() {
    let source = vec!["A", "B", "C"];
    let target = vec!["A", "C"];
    let changes = compute_diff_linear(&source, &target);

    assert_eq!(changes.len(), 1);
    assert_eq!(
        changes[0],
        Change {
            delta_type: DeltaType::Delete,
            start_original: 1,
            end_original: 2,
            start_revised: 1,
            end_revised: 1,
        }
    );
}

#[test]
fn test_myers_pure_insertion() {
    let source = vec!["A", "C"];
    let target = vec!["A", "B", "C"];
    let changes = compute_diff_linear(&source, &target);

    assert_eq!(changes.len(), 1);
    assert_eq!(
        changes[0],
        Change {
            delta_type: DeltaType::Insert,
            start_original: 1,
            end_original: 1,
            start_revised: 1,
            end_revised: 2,
        }
    );
}

#[test]
fn test_myers_classic_example() {
    let source: Vec<char> = "ABCABBA".chars().collect();
    let target: Vec<char> = "CBABAC".chars().collect();
    let changes = compute_diff_linear(&source, &target);

    assert!(!changes.is_empty());
    for change in &changes {
        assert!(change.start_original <= change.end_original);
        assert!(change.start_revised <= change.end_revised);
    }
}

#[test]
fn test_myers_with_workspace_reuse_and_listener() {
    let mut ws = LinearWorkspace::new();
    let mut listener = MockListener::new();

    let a = vec![1, 2, 3];
    let b = vec![1, 4, 3];

    let changes = compute_diff_full(&a, &b, |x, y| x == y, &mut ws, Some(&mut listener));

    assert_eq!(changes.len(), 2);
    assert!(listener.started);
    assert!(listener.ended);

    let changes_reuse =
        compute_diff_full(&a, &a, |x, y| x == y, &mut ws, None::<&mut MockListener>);
    assert!(changes_reuse.is_empty());
}

#[test]
fn test_myers_custom_predicate() {
    let source = vec![10, 20, 30];
    let target = vec![20, 30, 40];

    let changes = compute_diff_with(&source, &target, |a, b| a % 10 == b % 10);
    assert!(changes.is_empty());
}

// =========================================================================
// 4. Core Patch & Delta Operations
// =========================================================================

#[test]
fn test_basic_diff_and_patch() -> Result<(), PatchFailedException> {
    let source = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
    let target = vec![
        "apple".to_string(),
        "blueberry".to_string(),
        "cherry".to_string(),
        "date".to_string(),
    ];

    let changes = compute_diff_linear(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    let applied = patch.apply_to(&source)?;
    assert_eq!(applied, target);

    let restored = patch.restore(&target)?;
    assert_eq!(restored, source);

    Ok(())
}

#[test]
fn test_myers_struct_patch_integration() -> Result<(), PatchFailedException> {
    let source = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];
    let target = vec!["A".to_string(), "C".to_string(), "D".to_string(), "E".to_string()];

    let myers = MyersDiff::<String>::default();
    let changes = myers.diff(&source, &target);

    let patch = Patch::generate(&source, &target, &changes, false);
    let patched = patch.apply_to(&source)?;

    assert_eq!(patched, target);
    Ok(())
}

#[test]
fn test_empty_sequences_patch() -> Result<(), PatchFailedException> {
    let source: Vec<String> = vec![];
    let target: Vec<String> = vec![];

    let changes = compute_diff_linear(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    let result = patch.apply_to(&source)?;
    assert!(result.is_empty());

    Ok(())
}

// =========================================================================
// 5. Chunk Verification & Error Handling
// =========================================================================

#[test]
fn test_chunk_out_of_bounds_verification() {
    let chunk = Chunk::new(
        10,
        vec!["out".to_string(), "of".to_string(), "bounds".to_string()],
        None,
    );
    let target = vec!["in".to_string(), "bounds".to_string()];

    let verification = chunk.verify_chunk(&target).unwrap();
    assert_eq!(verification, VerifyChunk::PositionOutOfTarget);
}

#[test]
fn test_chunk_content_mismatch_verification() {
    let chunk = Chunk::new(0, vec!["foo".to_string(), "bar".to_string()], None);
    let target = vec!["foo".to_string(), "baz".to_string()];

    let verification = chunk.verify_chunk(&target).unwrap();
    assert_eq!(verification, VerifyChunk::ContentDoesNotMatchTarget);
}

#[test]
fn test_patch_failure_on_mismatched_target() {
    let source = vec!["line1".to_string(), "line2".to_string(), "line3".to_string()];
    let target = vec!["line1".to_string(), "modified".to_string(), "line3".to_string()];

    let changes = compute_diff_linear(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    let corrupted_source = vec!["wrong1".to_string(), "wrong2".to_string(), "wrong3".to_string()];
    let result = patch.apply_to(&corrupted_source);

    assert!(result.is_err(), "Expected patch application to fail on corrupted input");
}

#[test]
fn test_fuzzy_patch_unsupported() {
    let source = vec!["A".to_string(), "B".to_string()];
    let target_lines = vec!["A".to_string(), "X".to_string(), "B".to_string()];

    let source_chunk = Chunk::new(0, source, None);
    let target_chunk = Chunk::new(0, target_lines, None);
    let delta = Delta::new(PatchDeltaType::Insert, source_chunk, target_chunk);

    let mut target = vec!["A".to_string(), "B".to_string()];

    let result = delta.apply_fuzzy_to_at(&mut target, 1, 0);

    assert!(
        result.is_err(),
        "Expected unsupported error variant matching Java exception semantics"
    );
}

// =========================================================================
// 6. Text Utilities & HTML Formatting
// =========================================================================

#[test]
fn test_html_entities_escaping() {
    let input = "<html><body>'Hello' & \"World\"</body></html>";
    let expected = "&lt;html&gt;&lt;body&gt;'Hello' & \"World\"&lt;/body&gt;&lt;/html&gt;";
    assert_eq!(string_utils::html_entities(input), expected);
}

#[test]
fn test_html_entities_no_tags() {
    let input = "Plain text without tags";
    assert_eq!(string_utils::html_entities(input), input);
}

#[test]
fn test_normalize_tabs_and_entities() {
    let input = "fn main() {\n\tlet x = 1 < 2;\n}";
    let expected = "fn main() {\n    let x = 1 &lt; 2;\n}";
    assert_eq!(string_utils::normalize(input), expected);
}

#[test]
fn test_wrap_text_zero_width() {
    let input = "This string should remain untouched.";
    assert_eq!(string_utils::wrap_text(input, 0), input);
}

#[test]
fn test_wrap_text_short_line() {
    let input = "Short line";
    assert_eq!(string_utils::wrap_text(input, 20), input);
}

#[test]
fn test_wrap_text_exact_column_width() {
    let input = "1234567890";
    assert_eq!(string_utils::wrap_text(input, 10), input);
}

#[test]
fn test_wrap_text_multi_break() {
    let input = "ABCDEFGHIJKLM";
    let expected = "ABCD<br/>EFGH<br/>IJKL<br/>M";
    assert_eq!(string_utils::wrap_text(input, 4), expected);
}

#[test]
fn test_wrap_text_unicode_safety() {
    let input = "🦀🦀🦀🦀🦀🦀";
    let expected = "🦀🦀<br/>🦀🦀<br/>🦀🦀";
    assert_eq!(string_utils::wrap_text(input, 2), expected);
}

#[test]
fn test_wrap_text_list() {
    let input_list = vec![
        "Hello World".to_string(),
        "Short".to_string(),
        "ABCDEFGHIJKLMNOP".to_string(),
    ];

    let result = string_utils::wrap_text_list(&input_list, 5);

    assert_eq!(result[0], "Hello<br/> Worl<br/>d");
    assert_eq!(result[1], "Short");
    assert_eq!(result[2], "ABCDE<br/>FGHIJ<br/>KLMNO<br/>P");
}

// =========================================================================
// 7. Side-by-Side Diff Generator & Merging Engine
// =========================================================================

#[test]
fn test_diff_row_generator_side_by_side() {
    let source = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
    let target = vec![
        "Alpha".to_string(),
        "Beta Modified".to_string(),
        "Gamma".to_string(),
        "Delta".to_string(),
    ];

    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .build();

    let diff_rows = generator.generate_diff_rows(&source, &target);

    assert!(!diff_rows.is_empty());
    assert_eq!(diff_rows[0].old_line(), "Alpha");
    assert_eq!(diff_rows[0].new_line(), "Alpha");
}

#[test]
fn test_inline_delta_merge_utils() {
    let source = vec!["row1".to_string(), "row2".to_string()];
    let target = vec!["row1".to_string(), "row2_mod".to_string(), "row3".to_string()];

    let changes = compute_diff_linear(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    let merge_info =
        InlineDeltaMergeInfo::new(patch.deltas().to_vec(), source.clone(), target.clone());

    let merged = DeltaMergeUtils::merge_inline_deltas(&merge_info, |equalities| {
        equalities.iter().all(|s| s.trim().is_empty())
    });

    assert!(!merged.is_empty());
}

// =========================================================================
// 8. Unified Diff Generator & Parser Integrations
// =========================================================================

#[test]
fn test_unified_diff_generate_parse_and_apply() -> Result<(), PatchFailedException> {
    let source: Vec<String> = vec!["line 1", "line 2", "line 3", "line 4"]
        .into_iter()
        .map(String::from)
        .collect();

    let target: Vec<String> = vec!["line 1", "line 2 modified", "line 3", "line 4", "line 5"]
        .into_iter()
        .map(String::from)
        .collect();

    let changes = compute_diff_linear(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    let unified_lines = UnifiedDiffUtils::generate_unified_diff(
        Some("file1.txt"),
        Some("file2.txt"),
        &source,
        &patch,
        1,
    );

    assert!(!unified_lines.is_empty(), "Unified diff output should not be empty");
    assert!(unified_lines[0].starts_with("--- file1.txt"));
    assert!(unified_lines[1].starts_with("+++ file2.txt"));

    let parsed_patch = UnifiedDiffUtils::parse_unified_diff(&unified_lines);

    let applied = parsed_patch.apply_to(&source)?;
    assert_eq!(applied, target, "Applying parsed unified diff must yield target");

    Ok(())
}

#[test]
fn test_unified_diff_empty_inputs() -> Result<(), PatchFailedException> {
    let source: Vec<String> = vec![];
    let target: Vec<String> = vec![];

    let changes = compute_diff_linear(&source, &target);
    let patch = Patch::generate(&source, &target, &changes, false);

    let unified_lines = UnifiedDiffUtils::generate_unified_diff(
        Some("a.txt"),
        Some("b.txt"),
        &source,
        &patch,
        3,
    );

    let parsed_patch = UnifiedDiffUtils::parse_unified_diff(&unified_lines);
    let result = parsed_patch.apply_to(&source)?;

    assert!(result.is_empty());
    Ok(())
}