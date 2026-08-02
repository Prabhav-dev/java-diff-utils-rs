//! Utility functions for text manipulation and wrapping.

// 1. Standalone public functions
pub fn html_entities(str_input: &str) -> String {
    str_input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn normalize(str_input: &str) -> String {
    str_input.replace("\r\n", "\n").replace('\r', "\n")
}

/// Wraps text to column_width and joins wrapped lines with newlines into a String.
pub fn wrap_text(line: &str, column_width: usize) -> String {
    if column_width == 0 || line.chars().count() <= column_width {
        return line.to_string();
    }

    let mut result = String::new();
    let mut current_len = 0;

    for ch in line.chars() {
        if current_len >= column_width {
            result.push('\n');
            current_len = 0;
        }
        result.push(ch);
        current_len += 1;
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

    pub fn wrap_text(line: &str, column_width: usize) -> String {
        wrap_text(line, column_width)
    }

    pub fn wrap_text_list(list: &[String], column_width: usize) -> Vec<String> {
        wrap_text_list(list, column_width)
    }
}