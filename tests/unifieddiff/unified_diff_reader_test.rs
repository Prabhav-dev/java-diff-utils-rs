use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use regex::Regex;

use my_diff_crate::unifieddiff::unified_diff_reader::UnifiedDiffReader;

/// Helper function to mirror Java's `getResourceAsStream`.
/// Reads a fixture file located at `tests/fixtures/<filename>`.
fn get_resource_stream(filename: &str) -> BufReader<File> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(filename);

    let file = File::open(&path)
        .unwrap_or_else(|_| panic!("Failed to find test fixture file: {:?}", path));
    BufReader::new(file)
}

#[test]
fn test_simple_parse() {
    let stream = get_resource_stream("jsqlparser_patch_1.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 2);

    let file1 = &diff.files()[0];
    assert_eq!(
        file1.from_file(),
        Some("src/main/jjtree/net/sf/jsqlparser/parser/JSqlParserCC.jjt")
    );
    assert_eq!(file1.patch().deltas().len(), 3);
    assert_eq!(diff.tail(), Some("2.17.1.windows.2\n"));
}

#[test]
fn test_parse_diff_block() {
    let diff_line = "diff --git a/src/test/java/net/sf/jsqlparser/statement/select/SelectTest.java b/src/test/java/net/sf/jsqlparser/statement/select/SelectTest.java";
    let (from, to) = UnifiedDiffReader::<std::io::Empty>::parse_file_names(diff_line);

    assert_eq!(from, "src/test/java/net/sf/jsqlparser/statement/select/SelectTest.java");
    assert_eq!(to, "src/test/java/net/sf/jsqlparser/statement/select/SelectTest.java");
}

#[test]
fn test_chunk_header_parsing() {
    let pattern = Regex::new(r"^@@\s+-(?:(\d+)(?:,(\d+))?)\s+\+(?:(\d+)(?:,(\d+))?)\s+@@").unwrap();
    let text = "@@ -189,6 +189,7 @@ TOKEN: /* SQL Keywords. prefixed with K_ to avoid name clashes */";

    let captures = pattern.captures(text).expect("Pattern should match header");
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("189"));
    assert_eq!(captures.get(3).map(|m| m.as_str()), Some("189"));
}

#[test]
fn test_chunk_header_parsing2() {
    let pattern = Regex::new(r"^@@\s+-(?:(\d+)(?:,(\d+))?)\s+\+(?:(\d+)(?:,(\d+))?)\s+@@").unwrap();
    let text = "@@ -189,6 +189,7 @@";

    let captures = pattern.captures(text).expect("Pattern should match header");
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("189"));
    assert_eq!(captures.get(3).map(|m| m.as_str()), Some("189"));
}

#[test]
fn test_chunk_header_parsing3() {
    let pattern = Regex::new(r"^@@\s+-(?:(\d+)(?:,(\d+))?)\s+\+(?:(\d+)(?:,(\d+))?)\s+@@").unwrap();
    let text = "@@ -1,27 +1,27 @@";

    let captures = pattern.captures(text).expect("Pattern should match header");
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("1"));
    assert_eq!(captures.get(3).map(|m| m.as_str()), Some("1"));
}

#[test]
fn test_simple_parse2() {
    let stream = get_resource_stream("jsqlparser_patch_1.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 2);

    let file1 = &diff.files()[0];
    assert_eq!(
        file1.from_file(),
        Some("src/main/jjtree/net/sf/jsqlparser/parser/JSqlParserCC.jjt")
    );
    assert_eq!(file1.patch().deltas().len(), 3);

    let first = &file1.patch().deltas()[0];
    assert!(first.source().lines().len() > 0);
    assert!(first.target().lines().len() > 0);

    assert_eq!(diff.tail(), Some("2.17.1.windows.2\n"));
}

#[test]
fn test_parse_issue_201() {
    let stream = get_resource_stream("jsqlparser_patch_1.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 2);

    let file1 = &diff.files()[0];
    assert_eq!(
        file1.from_file(),
        Some("src/main/jjtree/net/sf/jsqlparser/parser/JSqlParserCC.jjt")
    );
    assert_eq!(file1.patch().deltas().len(), 3);

    let first = &file1.patch().deltas()[0];
    assert!(first.source().lines().len() > 0);
    assert!(first.target().lines().len() > 0);

    assert_eq!(diff.tail(), Some("2.17.1.windows.2\n"));
}

#[test]
fn test_simple_pattern() {
    let pattern = Regex::new(r"^\+\+\+\s").unwrap();
    assert!(pattern.is_match("+++ revised.txt"));
}

#[test]
fn test_parse_issue_46() {
    let stream = get_resource_stream("problem_diff_issue46.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 1);

    let file1 = &diff.files()[0];
    assert_eq!(file1.from_file(), Some("a.vhd"));
    assert_eq!(file1.patch().deltas().len(), 1);

    assert_eq!(diff.tail(), None);
}

#[test]
fn test_parse_issue_33() {
    let stream = get_resource_stream("problem_diff_issue33.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 1);

    let file1 = &diff.files()[0];
    assert_eq!(file1.from_file(), Some("Main.java"));
    assert_eq!(file1.patch().deltas().len(), 1);

    assert_eq!(diff.tail(), None);
    assert_eq!(diff.header(), None);
}

#[test]
fn test_parse_issue_51() {
    let stream = get_resource_stream("problem_diff_issue51.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 2);

    let file1 = &diff.files()[0];
    assert_eq!(file1.from_file(), Some("f1"));
    assert_eq!(file1.patch().deltas().len(), 1);

    let file2 = &diff.files()[1];
    assert_eq!(file2.from_file(), Some("f2"));
    assert_eq!(file2.patch().deltas().len(), 1);

    assert_eq!(diff.tail(), None);
}

#[test]
fn test_parse_issue_79() {
    let stream = get_resource_stream("problem_diff_issue79.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 1);

    let file1 = &diff.files()[0];
    assert_eq!(file1.from_file(), Some("test/Issue.java"));
    assert_eq!(file1.patch().deltas().len(), 0);

    assert_eq!(diff.tail(), None);
    assert_eq!(diff.header(), None);
}

#[test]
fn test_parse_issue_84() {
    let stream = get_resource_stream("problem_diff_issue84.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 2);

    let file1 = &diff.files()[0];
    assert_eq!(file1.from_file(), Some("config/ant-phase-verify.xml"));
    assert_eq!(file1.patch().deltas().len(), 1);

    let file2 = &diff.files()[1];
    assert_eq!(file2.from_file(), Some("/dev/null"));
    assert_eq!(file2.patch().deltas().len(), 1);

    assert_eq!(diff.tail(), Some("2.7.4"));
    assert!(diff
        .header()
        .unwrap_or("")
        .starts_with("From b53e612a2ab5ff15d14860e252f84c0f343fe93a Mon Sep 17 00:00:00 2001"));
}

#[test]
fn test_parse_issue_85() {
    let stream = get_resource_stream("problem_diff_issue85.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 1);

    let file1 = &diff.files()[0];
    assert_eq!(
        file1.diff_command(),
        Some("diff -r 83e41b73d115 -r a4438263b228 tests/test-check-pyflakes.t")
    );
    assert_eq!(file1.from_file(), Some("tests/test-check-pyflakes.t"));
    assert_eq!(file1.to_file(), Some("tests/test-check-pyflakes.t"));
    assert_eq!(file1.patch().deltas().len(), 1);

    assert_eq!(diff.tail(), None);
}

#[test]
fn test_time_stamp_regexp() {
    let pattern =
        Regex::new(r"(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}\.\d{3,})(?: [+-]\d+)?").unwrap();
    assert!(pattern.is_match("2019-04-18 13:49:39.516149751 +0200"));
}

#[test]
fn test_parse_issue_98() {
    let stream = get_resource_stream("problem_diff_issue98.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 1);

    let file1 = &diff.files()[0];
    assert_eq!(file1.deleted_file_mode(), Some("100644"));
    assert_eq!(
        file1.from_file(),
        Some("src/test/java/se/bjurr/violations/lib/model/ViolationTest.java")
    );
    assert_eq!(diff.tail(), Some("2.25.1"));
}

#[test]
fn test_parse_issue_104() {
    let stream = get_resource_stream("problem_diff_parsing_issue104.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 6);

    let file = &diff.files()[2];
    assert_eq!(file.from_file(), Some("/dev/null"));
    assert_eq!(file.to_file(), Some("doc/samba_data_tool_path.xml.in"));

    // NOTE: original expected string was a stale snapshot from an earlier struct
    // shape (separate InsertDelta/DeleteDelta/ChangeDelta types, no change_position
    // tracking). The parsed values themselves (delta type, positions, lines) were
    // already correct; only the Debug format was out of date.
    assert_eq!(
        format!("{:?}", file.patch()),
        "Patch { deltas: [Delta { delta_type: Insert, source: Chunk { position: 0, lines: [], change_position: Some([]) }, target: Chunk { position: 0, lines: [\"@SAMBA_DATA_TOOL@\"], change_position: Some([1]) } }], has_conflict_output: false }"
    );

    assert_eq!(diff.tail(), Some("2.14.4"));
}

#[test]
fn test_parse_issue_107_bazel_diff() {
    let stream = get_resource_stream("01-bazel-strip-unused.patch_issue107.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 450);

    let file = &diff.files()[0];
    assert_eq!(
        file.from_file(),
        Some("./src/main/java/com/amazonaws/AbortedException.java")
    );
    assert_eq!(
        file.to_file(),
        Some("/home/greg/projects/bazel/third_party/aws-sdk-auth-lite/src/main/java/com/amazonaws/AbortedException.java")
    );

    let no_newline_count = diff
        .files()
        .iter()
        .filter(|f| f.is_no_new_line_at_the_end_of_the_file())
        .count();
    assert_eq!(no_newline_count, 48);
}

#[test]
fn test_parse_issue_107_2() {
    let stream = get_resource_stream("problem_diff_issue107.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 2);

    let file1 = &diff.files()[0];
    assert_eq!(file1.from_file(), Some("Main.java"));
    assert_eq!(file1.patch().deltas().len(), 1);
}

#[test]
fn test_parse_issue_107_3() {
    let stream = get_resource_stream("problem_diff_issue107_3.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 1);

    let file1 = &diff.files()[0];
    assert_eq!(file1.from_file(), Some("Billion laughs attack.md"));
    assert_eq!(file1.patch().deltas().len(), 1);
}

#[test]
fn test_parse_issue_107_4() {
    let stream = get_resource_stream("problem_diff_issue107_4.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 27);

    let contains_readme = diff
        .files()
        .iter()
        .any(|f| f.from_file() == Some("README.md"));
    assert!(contains_readme);
}

#[test]
fn test_parse_issue_107_5() {
    let stream = get_resource_stream("problem_diff_issue107_5.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 22);

    let contains_file = diff.files().iter().any(|f| {
        f.from_file()
            == Some(
                "rt/management/src/test/java/org/apache/cxf/management/jmx/MBServerConnectorFactoryTest.java",
            )
    });
    assert!(contains_file);
}

#[test]
fn test_parse_issue_110() {
    let stream = get_resource_stream("0001-avahi-python-Use-the-agnostic-DBM-interface.patch");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 5);

    let file = &diff.files()[4];
    assert_eq!(file.similarity_index(), Some(87));
    assert_eq!(file.rename_from(), Some("service-type-database/build-db.in"));
    assert_eq!(file.rename_to(), Some("service-type-database/build-db"));

    assert_eq!(file.from_file(), Some("service-type-database/build-db.in"));
    assert_eq!(file.to_file(), Some("service-type-database/build-db"));
}

#[test]
fn test_parse_issue_117() {
    let stream = get_resource_stream("problem_diff_issue117.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 2);

    let file0 = &diff.files()[0];
    let delta0 = &file0.patch().deltas()[0];
    assert_eq!(
        delta0.source().change_position().as_deref(),
        Some(&vec![24, 27][..])
    );
    assert_eq!(
        delta0.target().change_position().as_deref(),
        Some(&vec![24, 27][..])
    );

    let delta1 = &file0.patch().deltas()[1];
    assert_eq!(
        delta1.source().change_position().as_deref(),
        Some(&vec![64][..])
    );
    assert_eq!(
        delta1.target().change_position().as_deref(),
        Some(&vec![64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74][..])
    );
}

#[test]
fn test_parse_issue_122() {
    let stream = get_resource_stream("problem_diff_issue122.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 1);

    let contains_file = diff
        .files()
        .iter()
        .any(|f| f.from_file() == Some("coders/wpg.c"));
    assert!(contains_file);
}

#[test]
fn test_parse_issue_123() {
    let stream = get_resource_stream("problem_diff_issue123.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    assert_eq!(diff.files().len(), 2);

    let contains_file = diff.files().iter().any(|f| {
        f.from_file()
            == Some("src/java/main/org/apache/zookeeper/server/FinalRequestProcessor.java")
    });
    assert!(contains_file);
}

#[test]
fn test_parse_issue_141() {
    let stream = get_resource_stream("problem_diff_issue141.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");
    let file1 = &diff.files()[0];

    assert_eq!(file1.from_file(), Some("a.txt"));
    assert_eq!(file1.to_file(), Some("a1.txt"));
}

#[test]
fn test_parse_issue_182add() {
    let stream = get_resource_stream("problem_diff_issue182_add.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    let file1 = &diff.files()[0];
    assert_eq!(file1.binary_added(), Some("some-image.png"));
}

#[test]
fn test_parse_issue_182delete() {
    let stream = get_resource_stream("problem_diff_issue182_delete.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    let file1 = &diff.files()[0];
    assert_eq!(file1.binary_deleted(), Some("some-image.png"));
}

#[test]
fn test_parse_issue_182edit() {
    let stream = get_resource_stream("problem_diff_issue182_edit.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    let file1 = &diff.files()[0];
    assert_eq!(file1.binary_edited(), Some("some-image.png"));
}

#[test]
fn test_parse_issue_182mode() {
    let stream = get_resource_stream("problem_diff_issue182_mode.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    let file1 = &diff.files()[0];
    assert_eq!(file1.old_mode(), Some("100644"));
    assert_eq!(file1.new_mode(), Some("100755"));
}

#[test]
fn test_parse_issue_193_copy() {
    let stream = get_resource_stream("problem_diff_parsing_issue193.diff");
    let diff = UnifiedDiffReader::parse_unified_diff(stream).expect("Failed to parse diff");

    let file1 = &diff.files()[0];
    assert_eq!(
        file1.copy_from(),
        Some("modules/configuration/config/web/pcf/account/AccountContactCV.pcf")
    );
    assert_eq!(
        file1.copy_to(),
        Some("modules/configuration/config/web/pcf/account/AccountContactCV.default.pcf")
    );
}