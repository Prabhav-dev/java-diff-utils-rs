use std::fs;
use std::io;

use my_diff_crate::UnifiedDiffUtils;

fn file_to_lines(filename: &str) -> io::Result<Vec<String>> {
    let content = fs::read_to_string(filename)?;
    Ok(content.lines().map(String::from).collect())
}

#[test]
fn test_generate_original_and_diff() {
    let orig_lines = file_to_lines("mocks/original.txt")
        .expect("Failed to read original.txt");
    let rev_lines = file_to_lines("mocks/revised.txt")
        .expect("Failed to read revised.txt");

    // Corrected parameter order: (lines1, lines2, name1, name2)
    let original_and_diff = UnifiedDiffUtils::generate_original_and_diff(
        &orig_lines,
        &rev_lines,
        Some("original.txt"),
        Some("revised.txt"),
    );

    let output = original_and_diff.join("\n");
    println!("{output}");
}

#[test]
fn test_generate_original_and_diff_first_line_change() {
    let orig_lines = file_to_lines("mocks/issue_170_original.txt")
        .expect("Failed to read issue_170_original.txt");
    let rev_lines = file_to_lines("mocks/issue_170_revised.txt")
        .expect("Failed to read issue_170_revised.txt");

    // Corrected parameter order: (lines1, lines2, name1, name2)
    let original_and_diff = UnifiedDiffUtils::generate_original_and_diff(
        &orig_lines,
        &rev_lines,
        Some("issue_170_original.txt"),
        Some("issue_170_revised.txt"),
    );

    let output = original_and_diff.join("\n");
    println!("{output}");
}