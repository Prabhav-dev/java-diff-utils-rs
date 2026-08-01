use std::fmt;

/// Represents the result status when verifying a chunk against a target sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VerifyChunk {
    /// The chunk matches the target sequence at the specified position.
    Ok,
    /// The specified position or range lies outside the boundaries of the target sequence.
    PositionOutOfTarget,
    /// The target sequence content does not match the chunk's expected lines.
    ContentDoesNotMatchTarget,
}

impl fmt::Display for VerifyChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::PositionOutOfTarget => write!(f, "POSITION_OUT_OF_TARGET"),
            Self::ContentDoesNotMatchTarget => write!(f, "CONTENT_DOES_NOT_MATCH_TARGET"),
        }
    }
}