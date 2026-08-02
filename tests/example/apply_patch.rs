use std::fs;
use std::error::Error;

use my_diff_crate::diff_utils::DiffUtils;
use my_diff_crate::UnifiedDiffUtils;

const ORIGINAL_PATH: &str = "tests/fixtures/issue10_base.txt";
const PATCH_PATH: &str = "tests/fixtures/issue10_patch.txt";

#[test]
fn test_apply_patch_example() -> Result<(), Box<dyn Error>> {
    let original: Vec<String> = fs::read_to_string(ORIGINAL_PATH)?
        .lines()
        .map(String::from)
        .collect();

    let patch_lines: Vec<String> = fs::read_to_string(PATCH_PATH)?
        .lines()
        .map(String::from)
        .collect();

    // 1. parse_unified_diff returns Patch<String> directly (no ?)
    let patch = UnifiedDiffUtils::parse_unified_diff(&patch_lines);

    // 2. Apply patch
    let result = DiffUtils::patch(&original, &patch)?;

    assert!(!result.is_empty());

    Ok(())
}