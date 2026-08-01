use std::error::Error;
use std::fmt;

use super::error::DiffError;

/// Error type thrown whenever a delta or patch cannot be applied to a given sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchFailedException {
    message: String,
}

impl PatchFailedException {
    /// Creates a new `PatchFailedException` with an empty error message.
    pub fn new() -> Self {
        Self {
            message: String::new(),
        }
    }

    /// Creates a new `PatchFailedException` with a descriptive message.
    pub fn with_message(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }

    /// Returns the descriptive message associated with this exception.
    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Default for PatchFailedException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PatchFailedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            write!(f, "Patch failed to apply")
        } else {
            write!(f, "Patch failed: {}", self.message)
        }
    }
}

impl Error for PatchFailedException {}

// Seamless coercion between PatchFailedException and base DiffError
impl From<PatchFailedException> for DiffError {
    fn from(err: PatchFailedException) -> Self {
        DiffError::PatchFailed(err.message)
    }
}

impl From<DiffError> for PatchFailedException {
    fn from(err: DiffError) -> Self {
        Self::with_message(err.to_string())
    }
}

impl From<&str> for PatchFailedException {
    fn from(msg: &str) -> Self {
        Self::with_message(msg)
    }
}

impl From<String> for PatchFailedException {
    fn from(msg: String) -> Self {
        Self::with_message(msg)
    }
}