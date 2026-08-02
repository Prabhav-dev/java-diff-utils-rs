//! Represents a single row in a side-by-side or line-by-line diff table.

use std::fmt;

/// Describes the operation tag associated with a diff row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tag {
    Insert,
    Delete,
    Change,
    Equal,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Insert => write!(f, "INSERT"),
            Tag::Delete => write!(f, "DELETE"),
            Tag::Change => write!(f, "CHANGE"),
            Tag::Equal => write!(f, "EQUAL"),
        }
    }
}

/// Describes a diff row in the form `[tag, old_line, new_line]` 
/// for showing differences between two texts side-by-side.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiffRow {
    tag: Tag,
    old_line: String,
    new_line: String,
}

impl DiffRow {
    /// Creates a new `DiffRow`.
    pub fn new(tag: Tag, old_line: impl Into<String>, new_line: impl Into<String>) -> Self {
        Self {
            tag,
            old_line: old_line.into(),
            new_line: new_line.into(),
        }
    }

    /// Returns the tag.
    #[inline]
    pub fn tag(&self) -> Tag {
        self.tag
    }

    /// Sets the tag.
    #[inline]
    pub fn set_tag(&mut self, tag: Tag) {
        self.tag = tag;
    }

    /// Returns a reference to the old line.
    #[inline]
    pub fn old_line(&self) -> &str {
        &self.old_line
    }

    /// Returns a reference to the new line.
    #[inline]
    pub fn new_line(&self) -> &str {
        &self.new_line
    }
}

impl fmt::Display for DiffRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{},{},{}]", self.tag, self.old_line, self.new_line)
    }
}