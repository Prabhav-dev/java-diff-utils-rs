#![no_main]
use libfuzzer_sys::fuzz_target;

use java_diff_utils_rs::DiffUtils;

fuzz_target!(|data: &[u8]| {
    let split = data.len() / 2;
    let original = String::from_utf8_lossy(&data[..split]);
    let revised = String::from_utf8_lossy(&data[split..]);
    let original_lines: Vec<String> = original.lines().map(str::to_owned).collect();
    let revised_lines: Vec<String> = revised.lines().map(str::to_owned).collect();

    let patch = DiffUtils::diff(&original_lines, &revised_lines, None);
    let result = patch
        .apply_to(&original_lines)
        .expect("a generated diff must apply to its source");
    assert_eq!(result, revised_lines);
});