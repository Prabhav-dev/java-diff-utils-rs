use std::fs;
use std::error::Error;

use my_diff_crate::diff_utils::DiffUtils;

const ORIGINAL_PATH: &str = "mocks/original.txt";
const REVISED_PATH: &str = "mocks/revised.txt";

#[test]
fn test_compute_difference() -> Result<(), Box<dyn Error>> {
    let original: Vec<String> = fs::read_to_string(ORIGINAL_PATH)?
        .lines()
        .map(String::from)
        .collect();

    let revised: Vec<String> = fs::read_to_string(REVISED_PATH)?
        .lines()
        .map(String::from)
        .collect();

    // Pass None for the progress listener argument
    let patch = DiffUtils::diff(&original, &revised, None);

    for delta in patch.get_deltas() {
        println!("{delta:?}");
    }

    Ok(())
}