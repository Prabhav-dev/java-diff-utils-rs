use my_diff_crate::diff_utils::DiffUtils;
use my_diff_crate::UnifiedDiffUtils;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Helper function mirroring Java's `fileToLines`.
/// Reads a file from the `tests/fixtures/` directory line-by-line into a Vec<String>.
fn file_to_lines(filename: &str) -> Vec<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(filename);

    let file = File::open(&path)
        .unwrap_or_else(|_| panic!("Failed to open test fixture file at: {:?}", path));

    BufReader::new(file)
        .lines()
        .map(|l| l.expect("Failed to read line from fixture"))
        .collect()
}

fn verify(orig_lines: Vec<String>, rev_lines: Vec<String>, original_file: &str, revised_file: &str) {
    let patch = DiffUtils::diff(&orig_lines, &rev_lines, None);
    let unified_diff = UnifiedDiffUtils::generate_unified_diff(
        Some(original_file),
        Some(revised_file),
        &orig_lines,
        &patch,
        10,
    );

    println!("{}", unified_diff.join("\n"));

    let from_unified_patch = UnifiedDiffUtils::parse_unified_diff(&unified_diff);

    let patched_lines = from_unified_patch
        .apply_to(&orig_lines)
        .expect("Patch failed to apply");

    assert_eq!(
        rev_lines.len(),
        patched_lines.len(),
        "Line counts do not match"
    );

    for (i, (expected, actual)) in rev_lines.iter().zip(patched_lines.iter()).enumerate() {
        assert_eq!(
            expected,
            actual,
            "Line {} of the patched file did not match the revised original",
            i + 1
        );
    }
}

#[test]
fn test_generate_unified() {
    let orig_lines = file_to_lines("original.txt");
    let rev_lines = file_to_lines("revised.txt");

    verify(orig_lines, rev_lines, "original.txt", "revised.txt");
}

#[test]
fn test_generate_unified_with_one_delta() {
    let orig_lines = file_to_lines("one_delta_test_original.txt");
    let rev_lines = file_to_lines("one_delta_test_revised.txt");

    verify(
        orig_lines,
        rev_lines,
        "one_delta_test_original.txt",
        "one_delta_test_revised.txt",
    );
}

#[test]
fn test_generate_unified_diff_without_any_deltas() {
    let test = vec!["abc".to_string()];
    let test_revised = vec!["abc2".to_string()];

    let patch = DiffUtils::diff(&test, &test_revised, None);
    let unified_diff = UnifiedDiffUtils::generate_unified_diff(
        Some("abc1"),
        Some("abc2"),
        &test,
        &patch,
        0,
    );
    let unified_diff_txt = unified_diff.join("\n");
    println!("{}", unified_diff_txt);

    assert!(
        unified_diff_txt.contains("--- abc1"),
        "original filename should be abc1"
    );
    assert!(
        unified_diff_txt.contains("+++ abc2"),
        "revised filename should be abc2"
    );
}

#[test]
fn test_diff_issue_10() {
    let base_lines = file_to_lines("issue10_base.txt");
    let patch_lines = file_to_lines("issue10_patch.txt");

    let p = UnifiedDiffUtils::parse_unified_diff(&patch_lines);

    DiffUtils::patch(&base_lines, &p).expect("Failed to apply patch");
}

#[test]
fn test_patch_with_no_deltas() {
    let lines1 = file_to_lines("issue11_1.txt");
    let lines2 = file_to_lines("issue11_2.txt");

    verify(lines1, lines2, "issue11_1.txt", "issue11_2.txt");
}

#[test]
fn test_diff5() {
    let lines1 = file_to_lines("5A.txt");
    let lines2 = file_to_lines("5B.txt");

    verify(lines1, lines2, "5A.txt", "5B.txt");
}

#[test]
fn test_diff_with_header_line_in_text() {
    let original = vec![
        "test line1".to_string(),
        "test line2".to_string(),
        "test line 4".to_string(),
        "test line 5".to_string(),
    ];

    let revised = vec![
        "test line1".to_string(),
        "test line2".to_string(),
        "@@ -2,6 +2,7 @@".to_string(),
        "test line 4".to_string(),
        "test line 5".to_string(),
    ];

    let patch = DiffUtils::diff(&original, &revised, None);
    let udiff = UnifiedDiffUtils::generate_unified_diff(
        Some("original"),
        Some("revised"),
        &original,
        &patch,
        10,
    );

    let _ = UnifiedDiffUtils::parse_unified_diff(&udiff);
}

#[test]
fn test_new_file_creation() {
    let original: Vec<String> = vec![];
    let revised = vec!["line1".to_string(), "line2".to_string()];

    let patch = DiffUtils::diff(&original, &revised, None);
    let udiff = UnifiedDiffUtils::generate_unified_diff(
        None,
        Some("revised"),
        &original,
        &patch,
        10,
    );

    assert_eq!(udiff[0], "--- /dev/null");
    assert_eq!(udiff[1], "+++ revised");
    assert_eq!(udiff[2], "@@ -0,0 +1,2 @@");

    let _ = UnifiedDiffUtils::parse_unified_diff(&udiff);
}

#[test]
fn test_change_position() {
    let patch_lines = file_to_lines("issue89_patch.txt");
    let patch = UnifiedDiffUtils::parse_unified_diff(&patch_lines);

    let real_remove_list_one = vec![3];
    let real_add_list_one = vec![3, 7, 8, 9, 10, 11, 12, 13, 14];
    validate_change_position(&patch, 0, &real_remove_list_one, &real_add_list_one);

    let real_remove_list_two = vec![];
    let real_add_list_two = vec![27, 28];
    validate_change_position(&patch, 1, &real_remove_list_two, &real_add_list_two);
}

fn validate_change_position(
    patch: &my_diff_crate::patch::Patch<String>,
    index: usize,
    real_remove_list: &[usize],
    real_add_list: &[usize],
) {
    let origin_chunk = patch.deltas()[index].source();
    let remove_binding = origin_chunk.change_position();
    let remove_list = remove_binding.as_deref().unwrap_or_default();

    assert_eq!(real_remove_list.len(), remove_list.len());
    for ele in real_remove_list {
        assert!(real_remove_list.contains(ele));
    }
    for ele in remove_list {
        assert!(real_remove_list.contains(ele));
    }

    let target_chunk = patch.deltas()[index].target();
    let add_binding = target_chunk.change_position();
    let add_list = add_binding.as_deref().unwrap_or_default();

    assert_eq!(real_add_list.len(), add_list.len());
    for ele in real_add_list {
        assert!(add_list.contains(ele));
    }
    for ele in add_list {
        assert!(real_add_list.contains(ele));
    }
}

#[test]
fn test_failing_patch_by_exception() {
    let mut base_lines = file_to_lines("issue10_base.txt");
    let patch_lines = file_to_lines("issue10_patch.txt");

    let p = UnifiedDiffUtils::parse_unified_diff(&patch_lines);

    // Corrupt the target original line to force a PatchFailedException
    base_lines[40] = format!("{} corrupted ", base_lines[40]);

    assert!(
        DiffUtils::patch(&base_lines, &p).is_err(),
        "Expected patch to fail on corrupted content"
    );
}

#[test]
fn test_wrong_context_length() {
    let original = file_to_lines("issue_119_original.txt");
    let revised = file_to_lines("issue_119_revised.txt");

    let patch = DiffUtils::diff(&original, &revised, None);
    let udiff = UnifiedDiffUtils::generate_unified_diff(
        Some("a/$filename"),
        Some("b/$filename"),
        &original,
        &patch,
        3,
    );

    assert!(
        udiff.iter().any(|line| line == "@@ -1,4 +1,4 @@"),
        "Expected chunk header '@@ -1,4 +1,4 @@' in output"
    );
}