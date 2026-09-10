//! Utility functions for text manipulation and wrapping.

// 1. Standalone public functions
pub fn html_entities(str_input: &str) -> String {
    str_input.replace('<', "&lt;").replace('>', "&gt;")
}

/// Expands tab characters into 4 spaces and HTML-escapes the result,
/// matching java-diff-utils' `StringUtils.normalize`.
pub fn normalize(str_input: &str) -> String {
    html_entities(&str_input.replace('\t', "    "))
}

use unicode_segmentation::UnicodeSegmentation;

/// Wraps text to column_width, joining wrapped segments with `<br/>` tags.
/// A column_width of 0 leaves the line untouched (no wrapping is possible).
pub fn wrap_text(line: &str, column_width: usize) -> String {
    if column_width == 0 {
        return line.to_string();
    }

    let graphemes: Vec<&str> = line.graphemes(true).collect();
    if graphemes.len() <= column_width {
        return line.to_string();
    }

    let mut result = String::new();
    for (i, &g) in graphemes.iter().enumerate() {
        if i > 0 && i % column_width == 0 {
            result.push_str("<br/>");
        }
        result.push_str(g);
    }

    result
}

pub fn wrap_text_list(list: &[String], column_width: usize) -> Vec<String> {
    list.iter()
        .map(|s| wrap_text(s, column_width))
        .collect()
}

// 2. Struct wrapper mapping to the standalone functions
pub struct StringUtils;

impl StringUtils {
    pub fn html_entities(str_input: &str) -> String {
        html_entities(str_input)
    }

    pub fn normalize(str_input: &str) -> String {
        normalize(str_input)
    }

    /// Unlike the free `wrap_text` function, this associated method matches
    /// java-diff-utils' `StringUtils.wrapText(String, int)`, which requires a
    /// positive column width and panics otherwise.
    pub fn wrap_text(line: &str, column_width: usize) -> String {
        if column_width == 0 {
            panic!("column width must be positive");
        }
        wrap_text(line, column_width)
    }

    pub fn wrap_text_list(list: &[String], column_width: usize) -> Vec<String> {
        wrap_text_list(list, column_width)
    }
}