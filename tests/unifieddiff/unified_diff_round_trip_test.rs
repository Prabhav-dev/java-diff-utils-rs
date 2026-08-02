use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::path::PathBuf;

use my_diff_crate::diff_utils::DiffUtils;
use my_diff_crate::unifieddiff::unified_diff::UnifiedDiff;
use my_diff_crate::unifieddiff::unified_diff_file::UnifiedDiffFile;
use my_diff_crate::unifieddiff::unified_diff_reader::UnifiedDiffReader;
use my_diff_crate::unifieddiff::unified_diff_writer::UnifiedDiffWriter;

/// Helper function to read file lines from the fixture directory (`tests/fixtures/`).
fn file_to_lines(filename: &str) -> Vec<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(filename);

    let file = File::open(&path)
        .unwrap_or_else(|_| panic!("Failed to find test fixture file: {:?}", path));
    
    BufReader::new(file)
        .lines()
        .map(|l| l.expect("Failed to read line"))
        .collect()
}

/// Helper function to perform round-trip diffing, formatting, parsing, and patch applying.
fn verify(orig_lines: &[String], rev_lines: &[String], original_file: &str, revised_file: &str) {
    let patch = DiffUtils::diff(orig_lines, rev_lines, None);

    let file_entry = UnifiedDiffFile::from(original_file, revised_file, patch);
    let unified_diff_data = UnifiedDiff::from(Some("header"), Some("tail"), vec![file_entry]);

    let mut writer_buffer = Vec::new();
    UnifiedDiffWriter::write(
        &unified_diff_data,
        |_file_name| orig_lines.to_vec(),
        &mut writer_buffer,
        10,
    )
    .expect("Failed to write unified diff");

    let diff_output_str = String::from_utf8(writer_buffer)
        .expect("Generated diff should be valid UTF-8");
    println!("{}", diff_output_str);

    let mut parsed_diff = UnifiedDiffReader::parse_unified_diff(Cursor::new(diff_output_str.as_bytes()))
        .expect("Failed to parse unified diff");

    let patched_lines = parsed_diff
        .apply_patch_to(|file| file == original_file, orig_lines)
        .expect("Failed to apply patch");

    assert_eq!(
        rev_lines.len(),
        patched_lines.len(),
        "Patched file line count does not match revised line count"
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

    verify(&orig_lines, &rev_lines, "original.txt", "revised.txt");
}

#[test]
fn test_generate_unified_with_one_delta() {
    let orig_lines = file_to_lines("one_delta_test_original.txt");
    let rev_lines = file_to_lines("one_delta_test_revised.txt");

    verify(&orig_lines, &rev_lines, "one_delta_test_original.txt", "one_delta_test_revised.txt");
}

#[test]
fn test_generate_unified_diff_without_any_deltas() {
    let test = vec!["abc".to_string()];
    let patch = DiffUtils::diff(&test, &test, None);

    let file_entry = UnifiedDiffFile::from("abc", "abc", patch);
    let diff_data = UnifiedDiff::from(Some("header"), Some("tail"), vec![file_entry]);

    let mut writer_buffer = Vec::new();
    UnifiedDiffWriter::write(
        &diff_data,
        |_file_name| test.clone(),
        &mut writer_buffer,
        0,
    )
    .expect("Failed to write unified diff");

    let output = String::from_utf8(writer_buffer).unwrap();
    println!("{}", output);
}

#[test]
fn test_diff_issue_10() {
    let base_lines = file_to_lines("issue10_base.txt");
    let patch_lines = file_to_lines("issue10_patch.txt");

    let patch_content = patch_lines.join("\n");
    let unified_diff = UnifiedDiffReader::parse_unified_diff(Cursor::new(patch_content.as_bytes()))
        .expect("Failed to parse patch");

    let patch = &unified_diff.files()[0].patch();
    let result = DiffUtils::patch(&base_lines, patch);

    assert!(result.is_ok(), "Patching failed: {:?}", result.err());
}

#[test]
#[ignore = "Disabled in original Java test"]
fn test_patch_with_no_deltas() {
    let lines1 = file_to_lines("issue11_1.txt");
    let lines2 = file_to_lines("issue11_2.txt");
    verify(&lines1, &lines2, "issue11_1.txt", "issue11_2.txt");
}

#[test]
fn test_diff_5() {
    let lines1 = file_to_lines("5A.txt");
    let lines2 = file_to_lines("5B.txt");
    verify(&lines1, &lines2, "5A.txt", "5B.txt");
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
    let file_entry = UnifiedDiffFile::from("original", "revised", patch);
    let diff_data = UnifiedDiff::from(Some("header"), Some("tail"), vec![file_entry]);

    let mut writer_buffer = Vec::new();
    UnifiedDiffWriter::write(
        &diff_data,
        |_file_name| original.clone(),
        &mut writer_buffer,
        10,
    )
    .expect("Failed to write diff");

    let diff_str = String::from_utf8(writer_buffer).unwrap();
    println!("{}", diff_str);

    let parsed_diff = UnifiedDiffReader::parse_unified_diff(Cursor::new(diff_str.as_bytes()));
    assert!(parsed_diff.is_ok(), "Failed to parse round-tripped diff string");
}