use std::fmt;

use super::chunk::Chunk;
use super::delta::Delta;
use super::delta_type::DeltaType;
use super::error::PatchError;

/// Describes a delete-delta representing removed content from an original sequence.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeleteDelta<T> {
    inner: Delta<T>,
}

impl<T> DeleteDelta<T> {
    /// Creates a new `DeleteDelta` with the given original (source) and revised (target) chunks.
    pub fn new(original: Chunk<T>, revised: Chunk<T>) -> Self {
        Self {
            inner: Delta::new(DeltaType::Delete, original, revised),
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

    /// Applies this delete delta to the target vector.
    ///
    /// Drains/removes the elements specified by the source chunk's range.
    pub fn apply_to(&self, target: &mut Vec<T>) -> Result<(), PatchError> {
        let position = self.source().position();
        let size = self.source().len();

        if position > target.len() || position + size > target.len() {
            return Err(PatchError::PatchFailed(format!(
                "DeleteDelta range [{}..{}] out of bounds for target length {}",
                position,
                position + size,
                target.len()
            )));
        }

        target.drain(position..position + size);
        Ok(())
    }

    /// Restores (un-applies) this delete delta on the target vector.
    ///
    /// Re-inserts the removed original lines back into the target vector at the recorded position.
    pub fn restore(&self, target: &mut Vec<T>) -> Result<(), PatchError>
    where
        T: Clone,
    {
        let position = self.target().position();

        if position > target.len() {
            return Err(PatchError::PatchFailed(format!(
                "DeleteDelta restore position {} out of bounds for target length {}",
                position,
                target.len()
            )));
        }

        target.splice(
            position..position,
            self.source().lines().iter().cloned(),
        );

        Ok(())
    }

    /// Creates a new `DeleteDelta` with custom source and target chunks.
    #[must_use]
    pub fn with_chunks(&self, original: Chunk<T>, revised: Chunk<T>) -> Self {
        Self::new(original, revised)
    }
}

impl<T: fmt::Debug> fmt::Display for DeleteDelta<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[DeleteDelta, position: {}, lines: {:?}]",
            self.source().position(),
            self.source().lines()
        )
    }
}

impl<T> From<DeleteDelta<T>> for Delta<T> {
    fn from(delete: DeleteDelta<T>) -> Self {
        delete.inner
    }
}