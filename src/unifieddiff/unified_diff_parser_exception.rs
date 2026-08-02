use std::fmt;
use std::error::Error;

/// Equivalent to Java's `UnifiedDiffParserException`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifiedDiffParserException {
    pub message: String,
}

impl UnifiedDiffParserException {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UnifiedDiffParserException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UnifiedDiffParserException: {}", self.message)
    }
}

impl Error for UnifiedDiffParserException {}

// Allows automatically converting standard IO errors into this parser exception
impl From<std::io::Error> for UnifiedDiffParserException {
    fn from(err: std::io::Error) -> Self {
        UnifiedDiffParserException::new(err.to_string())
    }
}