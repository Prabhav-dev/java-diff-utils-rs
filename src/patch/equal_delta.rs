use std::fmt;

use super::chunk::Chunk;
use super::delta::Delta;
use super::delta_type::DeltaType;
use super::error::PatchError;

/// Represents an unchanged (equal) region of data between the source and target sequences.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EqualDelta<T> {
    inner: Delta<T>,
}

impl<T> EqualDelta<T> {
    /// Creates a new `EqualDelta` with the given source and target chunks.
    pub fn new(source: Chunk<T>, target: Chunk<T>) -> Self {
        Self {
            inner: Delta::new(DeltaType::Equal, source, target),
        }
    }

    /// Returns a reference to the underlying inner [`Delta`].
    #[inline]
    pub fn delta(&self) -> &Delta<T> {
        &self.inner
    }

    /// Consumes `self` and returns the inner [`Delta`].
    #[inline]
    pub fn into_delta(self) -> Delta<T> {
        self.inner
    }

    /// Returns a reference to the source chunk.
    #[inline]
    pub fn source(&self) -> &Chunk<T> {
        self.inner.source()
    }

    /// Returns a reference to the target chunk.
    #[inline]
    pub fn target(&self) -> &Chunk<T> {
        self.inner.target()
    }

    /// Applies this equal delta to the target vector.
    ///
    /// Since the lines are identical, this is a no-op that always succeeds.
    pub fn apply_to(&self, _target: &mut Vec<T>) -> Result<(), PatchError> {
        Ok(())
    }

    /// Restores (un-applies) this equal delta on the target vector.
    ///
    /// Since the lines are identical, this is a no-op that always succeeds.
    pub fn restore(&self, _target: &mut Vec<T>) -> Result<(), PatchError> {
        Ok(())
    }

    /// Applies fuzzy patching for equal lines.
    ///
    /// Since the content is equal, no modification occurs.
    pub fn apply_fuzzy_to_at(
        &self,
        _target: &mut Vec<T>,
        _fuzz: usize,
        _position: usize,
    ) -> Result<(), PatchError> {
        Ok(())
    }

    /// Creates a new `EqualDelta` with custom source and target chunks.
    #[must_use]
    pub fn with_chunks(&self, original: Chunk<T>, revised: Chunk<T>) -> Self {
        Self::new(original, revised)
    }
}

impl<T: fmt::Debug> fmt::Display for EqualDelta<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[EqualDelta, position: {}, lines: {:?}]",
            self.source().position(),
            self.source().lines()
        )
    }
}

impl<T> From<EqualDelta<T>> for Delta<T> {
    fn from(equal: EqualDelta<T>) -> Self {
        equal.inner
    }
}