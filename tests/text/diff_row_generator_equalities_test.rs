use my_diff_crate::{DiffRowGenerator, Tag};

#[test]
fn test_default_equality_processing_leaves_text_unchanged() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(false)
        .build();

    let rows = generator.generate_diff_rows(
        &["hello world".to_string()],
        &["hello world".to_string()],
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].old_line(), "hello world");
    assert_eq!(rows[0].new_line(), "hello world");
    assert_eq!(rows[0].tag(), Tag::Equal);
}

#[test]
fn test_custom_equality_processing_is_applied() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(false)
        .process_equalities(|text| format!("[{}]", text))
        .build();

    let rows = generator.generate_diff_rows(
        &["A".to_string(), "B".to_string()],
        &["A".to_string(), "B".to_string()],
    );

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].old_line(), "[A]");
    assert_eq!(rows[1].old_line(), "[B]");
}

/// Verifies that process_equalities can be used to HTML-escape unchanged
/// lines while still working together with the default HTML-oriented line_normalizer.
#[test]
fn test_html_escaping_equalities_works_with_default_normalizer() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .process_equalities(|s| s.replace('<', "&lt;").replace('>', "&gt;"))
        .build();

    // both lines are equal -> Tag::Equal, process_equalities is invoked
    let rows = generator.generate_diff_rows(
        &["hello <world>".to_string()],
        &["hello <world>".to_string()],
    );

    let row = &rows[0];

    assert!(
        row.old_line().contains("&lt;world&gt;"),
        "Expected old_line to contain escaped HTML tags"
    );
    assert!(
        row.new_line().contains("&lt;world&gt;"),
        "Expected new_line to contain escaped HTML tags"
    );
}

/// Ensures equalities are processed while inline diff markup is still
/// present somewhere in the line.
#[test]
fn test_equalities_processed_but_inline_diff_still_present() {
    let generator = DiffRowGenerator::create()
        .show_inline_diffs(true)
        .inline_diff_by_word(true)
        .process_equalities(|s| format!("({})", s))
        .build();

    let rows = generator.generate_diff_rows(
        &["hello world".to_string()],
        &["hello there".to_string()],
    );

    let row = &rows[0];

    println!("OLD = {}", row.old_line());
    println!("NEW = {}", row.new_line());

    // Row must be Change
    assert_eq!(row.tag(), Tag::Change);

    // Inline diff markup must appear
    assert!(
        row.old_line().contains("span") || row.new_line().contains("span"),
        "Expected inline <span> diff markup in old or new line"
    );

    // Equalities inside Change row must NOT be wrapped by process_equalities
    assert!(
        row.old_line().starts_with("hello "),
        "Equal (unchanged) inline segment should remain unchanged"
    );
}