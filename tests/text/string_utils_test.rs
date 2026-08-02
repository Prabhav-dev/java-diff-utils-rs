use my_diff_crate::text::string_utils::StringUtils;

#[test]
fn test_html_entities() {
    assert_eq!("&lt;test&gt;", StringUtils::html_entities("<test>"));
}

#[test]
fn test_normalize_string() {
    assert_eq!("    test", StringUtils::normalize("\ttest"));
}

#[test]
fn test_wrap_text_string_int() {
    assert_eq!("te<br/>st", StringUtils::wrap_text("test", 2));
    assert_eq!("tes<br/>t", StringUtils::wrap_text("test", 3));
    assert_eq!("test", StringUtils::wrap_text("test", 10));

    // Testing Unicode surrogate pairs / UTF-8 grapheme boundaries safely:
    assert_eq!(".𐀁<br/>.", StringUtils::wrap_text(".𐀁.", 2));
    assert_eq!("..<br/>𐀁", StringUtils::wrap_text("..\u{10001}", 3));
}

#[test]
#[should_panic(expected = "column width must be positive")]
fn test_wrap_text_string_int_zero() {
    StringUtils::wrap_text("test", 0);
}