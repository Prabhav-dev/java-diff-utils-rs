use std::fmt;

use super::chunk::Chunk;
use super::delta::Delta;
use super::delta_type::DeltaType;
use super::error::PatchError;

/// Describes an insert-delta representing new content added to a target sequence.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InsertDelta<T> {
    inner: Delta<T>,
}

impl<T> InsertDelta<T> {
    /// Creates a new `InsertDelta` with the given original (source) and revised (target) chunks.
    pub fn new(original: Chunk<T>, revised: Chunk<T>) -> Self {
        Self {
            inner: Delta::new(DeltaType::Insert, original, revised),
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

    /// Returns a reference to the source (original) chunk.
    #[inline]
    pub fn source(&self) -> &Chunk<T> {
        self.inner.source()
    }

    /// Returns a reference to the target (revised) chunk.
    #[inline]
    pub fn target(&self) -> &Chunk<T> {
        self.inner.target()
    }

    /// Applies this insert delta to the target vector.
    ///
    /// Inserts the revised chunk lines at the source chunk's target position.
    pub fn apply_to(&self, target: &mut Vec<T>) -> Result<(), PatchError>
    where
        T: Clone,
    {
        let position = self.source().position();

        if position > target.len() {
            return Err(PatchError::PatchFailed(format!(
                "InsertDelta position {} out of bounds for target length {}",
                position,
                target.len()
            )));
        }

        target.splice(
            position..position,
            self.target().lines().iter().cloned(),
        );

        Ok(())
    }

    /// Restores (un-applies) this insert delta on the target vector.
    ///
    /// Removes/drains the inserted target lines from the target vector.
    pub fn restore(&self, target: &mut Vec<T>) -> Result<(), PatchError> {
        let position = self.target().position();
        let size = self.target().len();

        if position > target.len() || position + size > target.len() {
            return Err(PatchError::PatchFailed(format!(
                "InsertDelta restore range [{}..{}] out of bounds for target length {}",
                position,
                position + size,
                target.len()
            )));
        }

        target.drain(position..position + size);
        Ok(())
    }

    /// Creates a new `InsertDelta` with custom source and target chunks.
    #[must_use]
    pub fn with_chunks(&self, original: Chunk<T>, revised: Chunk<T>) -> Self {
        Self::new(original, revised)
    }
}

impl<T: fmt::Debug> fmt::Display for InsertDelta<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[InsertDelta, position: {}, lines: {:?}]",
            self.source().position(),
            self.target().lines()
        )
    }
}

impl<T> From<InsertDelta<T>> for Delta<T> {
    fn from(insert: InsertDelta<T>) -> Self {
        insert.inner
    }
}