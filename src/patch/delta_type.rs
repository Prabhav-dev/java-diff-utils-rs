use std::fmt;
use serde::{Deserialize, Serialize};

/// Specifies the classification type of a sequence modification.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeltaType {
    /// A block of data in the original sequence is replaced by another block of data.
    Change,
    /// A block of data in the original sequence is removed.
    Delete,
    /// A block of data is inserted into the sequence at a given position.
    Insert,
    /// Represents an unchanged block of data where original and revised content match.
    Equal,
}

impl fmt::Display for DeltaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Change => write!(f, "CHANGE"),
            Self::Delete => write!(f, "DELETE"),
            Self::Insert => write!(f, "INSERT"),
            Self::Equal => write!(f, "EQUAL"),
        }
    }
}