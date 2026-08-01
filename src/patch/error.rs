use std::error::Error;
use std::fmt;

/// Base error type for all diff and patch operations in this library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffError {
    /// A general patch processing exception with a descriptive error message.
    General(String),
    /// Indicates that a patch application or verification failed.
    PatchFailed(String),
    /// Indicates that an unsupported or invalid operation was attempted.
    UnsupportedOperation(String),
}

impl DiffError {
    /// Constructs a general error with the given message.
    pub fn new(msg: impl Into<String>) -> Self {
        Self::General(msg.into())
    }

    /// Helper constructor for patch failure errors.
    pub fn patch_failed(msg: impl Into<String>) -> Self {
        Self::PatchFailed(msg.into())
    }

    /// Helper constructor for unsupported operation errors.
    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::UnsupportedOperation(msg.into())
    }
}

impl fmt::Display for DiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::General(msg) => write!(f, "{msg}"),
            Self::PatchFailed(msg) => write!(f, "Patch failed: {msg}"),
            Self::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {msg}"),
        }
    }
}

impl Error for DiffError {}

impl From<String> for DiffError {
    fn from(msg: String) -> Self {
        Self::General(msg)
    }
}

impl From<&str> for DiffError {
    fn from(msg: &str) -> Self {
        Self::General(msg.to_string())
    }
}

/// Type aliases for module parity.
pub type PatchError = DiffError;
pub type PatchFailedException = DiffError;