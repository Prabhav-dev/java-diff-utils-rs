//! Helper functions for escaping HTML entities, expanding tabs, and wrapping text.

/// Replaces all opening and closing tags with `&lt;` or `&gt;`.
pub fn html_entities(str_input: &str) -> String {
    str_input.replace('<', "&lt;").replace('>', "&gt;")
}

/// Normalizes line content by replacing HTML meta characters and converting tabs to 4 spaces.
pub fn normalize(str_input: &str) -> String {
    html_entities(str_input).replace('\t', "    ")
}

/// Wraps a list of text lines according to the specified column width.
pub fn wrap_text_list(list: &[String], column_width: usize) -> Vec<String> {
    list.iter().map(|line| wrap_text(line, column_width)).collect()
}

/// Wraps a single line of text with `<br/>` tags at fixed character intervals.
/// 
/// Handles UTF-8 byte boundaries safely so Unicode characters are never split.
pub fn wrap_text(line: &str, column_width: usize) -> String {
    if column_width == 0 {
        return line.to_string();
    }

    let char_count = line.chars().count();
    if char_count <= column_width {
        return line.to_string();
    }

    let delimiter = "<br/>";
    let delimiter_len = delimiter.len();

    let mut b = line.to_string();
    let mut width_index = column_width;
    let mut count = 0;

    while char_count > width_index {
        let mut target_char_idx = width_index;

        // Convert character index to exact UTF-8 byte index
        let mut byte_idx = char_to_byte_idx(&b, target_char_idx + delimiter_len * count);

        // Adjust if we hit the very end of a string slice boundary
        if byte_idx == b.len() && target_char_idx > 1 {
            target_char_idx -= 1;
            byte_idx = char_to_byte_idx(&b, target_char_idx + delimiter_len * count);
        }

        b.insert_str(byte_idx, delimiter);

        count += 1;
        width_index += column_width;
    }

    b
}

/// Converts a character offset to a safe UTF-8 byte offset in a string slice.
fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_entities() {
        assert_eq!(html_entities("<div>test</div>"), "&lt;div&gt;test&lt;/div&gt;");
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("a\tb<c>"), "a    b&lt;c&gt;");
    }

    #[test]
    fn test_wrap_text() {
        assert_eq!(wrap_text("1234567890", 3), "123<br/>456<br/>789<br/>0");
        assert_eq!(wrap_text("hello", 0), "hello");
        assert_eq!(wrap_text("hello", 10), "hello");
    }
}