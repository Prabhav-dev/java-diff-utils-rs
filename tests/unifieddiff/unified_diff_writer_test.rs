use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use my_diff_crate::diff_utils::DiffUtils;
use my_diff_crate::unifieddiff::unified_diff::UnifiedDiff;
use my_diff_crate::unifieddiff::unified_diff_file::UnifiedDiffFile;
use my_diff_crate::unifieddiff::unified_diff_reader::UnifiedDiffReader;
use my_diff_crate::unifieddiff::unified_diff_writer::UnifiedDiffWriter;

/// Helper function to read a fixture file's contents into a UTF-8 String.
fn read_fixture(filename: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(filename);

    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test fixture file at: {:?}", path))
}

#[test]
fn test_write() {
    let content = read_fixture("jsqlparser_patch_1.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(Cursor::new(content.as_bytes()))
        .expect("Failed to parse unified diff");

    let mut writer_buffer = Vec::new();
    UnifiedDiffWriter::write(&diff, |_f| Vec::<String>::new(), &mut writer_buffer, 5)
        .expect("Failed to write unified diff");

    let output = String::from_utf8(writer_buffer)
        .expect("Written output should be valid UTF-8");

    println!("{}", output);
}

/// Issue 47
#[test]
fn test_write_with_new_file() {
    let original: Vec<String> = Vec::new();
    let revised: Vec<String> = vec!["line1".to_string(), "line2".to_string()];

    let patch = DiffUtils::diff(&original, &revised, None);
    
    let mut diff = UnifiedDiff::new();
    diff.add_file(UnifiedDiffFile::from("", "revised", patch));

    let mut writer_buffer = Vec::new();
    UnifiedDiffWriter::write(&diff, |_f| original.clone(), &mut writer_buffer, 5)
        .expect("Failed to write unified diff");

    let output = String::from_utf8(writer_buffer).unwrap();

    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines[0], "--- /dev/null");
    assert_eq!(lines[1], "+++ revised");
    assert_eq!(lines[2], "@@ -0,0 +1,2 @@");
}