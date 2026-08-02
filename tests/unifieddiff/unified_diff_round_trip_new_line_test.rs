use std::io::Cursor;

use my_diff_crate::unifieddiff::unified_diff_reader::UnifiedDiffReader;

#[test]
fn test_issue_135_missing_no_new_line_in_patched() {
    let before_content = "rootProject.name = \"sample-repo\"";
    let after_content = "rootProject.name = \"sample-repo\"\n";

    let patch = "diff --git a/settings.gradle b/settings.gradle\n\
                 index ef3b8e2..ab30124 100644\n\
                 --- a/settings.gradle\n\
                 +++ b/settings.gradle\n\
                 @@ -1 +1 @@\n\
                 -rootProject.name = \"sample-repo\"\n\
                 \\ No newline at end of file\n\
                 +rootProject.name = \"sample-repo\"\n";

    let stream = Cursor::new(patch.as_bytes());

    let unified_diff = UnifiedDiffReader::parse_unified_diff(stream)
        .expect("Failed to parse unified diff");

    let file = &unified_diff.files()[0];
    // Convert to Vec<String> instead of Vec<&str>
    let before_lines: Vec<String> = before_content.lines().map(String::from).collect();

    let patched_lines = file
        .patch()
        .apply_to(&before_lines)
        .expect("Failed to apply patch");

    let mut unified_after_content = patched_lines.join("\n");
    if !unified_after_content.is_empty() {
        unified_after_content.push('\n');
    }

    assert_eq!(unified_after_content, after_content);
}