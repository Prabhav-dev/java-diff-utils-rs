use std::fmt;

use super::chunk::Chunk;
use super::delta::Delta;
use super::delta_type::DeltaType;
use super::error::PatchError;

/// Describes a change-delta representing replaced content between original and revised sequences.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChangeDelta<T> {
    inner: Delta<T>,
}

impl<T> ChangeDelta<T> {
    /// Creates a new `ChangeDelta` with the given source and target chunks.
    pub fn new(source: Chunk<T>, target: Chunk<T>) -> Self {
        Self {
            inner: Delta::new(DeltaType::Change, source, target),
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

    /// Applies this change delta to the target vector.
    ///
    /// Replaces the element range at the source chunk's position with the target chunk's lines.
    pub fn apply_to(&self, target: &mut Vec<T>) -> Result<(), PatchError>
    where
        T: Clone + PartialEq,
    {
        let position = self.source().position();
        let source_size = self.source().len();

        if position > target.len() || position + source_size > target.len() {
            return Err(PatchError::PatchFailed(format!(
                "ChangeDelta position {} (size {}) out of bounds for target length {}",
                position,
                source_size,
                target.len()
            )));
        }

        // Efficient bulk splice replacement instead of item-by-item removal and insertion
        target.splice(
            position..position + source_size,
            self.target().lines().iter().cloned(),
        );

        Ok(())
    }

    /// Restores (un-applies) this change delta on the target sequence.
    ///
    /// Replaces the element range at the target chunk's position with the original source lines.
    pub fn restore(&self, target: &mut Vec<T>) -> Result<(), PatchError>
    where
        T: Clone + PartialEq,
    {
        let position = self.target().position();
        let target_size = self.target().len();

        if position > target.len() || position + target_size > target.len() {
            return Err(PatchError::PatchFailed(format!(
                "ChangeDelta restore position {} (size {}) out of bounds for target length {}",
                position,
                target_size,
                target.len()
            )));
        }

        target.splice(
            position..position + target_size,
            self.source().lines().iter().cloned(),
        );

        Ok(())
    }

    /// Applies the patch with a fuzzy tolerance context offset.
    pub fn apply_fuzzy_to_at(
        &self,
        target: &mut Vec<T>,
        _fuzz: usize,
        position: usize,
    ) -> Result<(), PatchError>
    where
        T: Clone + PartialEq,
    {
        if position > target.len() {
            return Err(PatchError::PatchFailed(format!(
                "Fuzzy patch position {} out of bounds for target length {}",
                position,
                target.len()
            )));
        }

        let end = (position + self.source().len()).min(target.len());

        target.splice(
            position..end,
            self.target().lines().iter().cloned(),
        );

        Ok(())
    }

    /// Creates a new `ChangeDelta` with custom source and target chunks.
    #[must_use]
    pub fn with_chunks(&self, original: Chunk<T>, revised: Chunk<T>) -> Self {
        Self::new(original, revised)
    }
}

impl<T: fmt::Debug> fmt::Display for ChangeDelta<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[ChangeDelta, position: {}, lines: {:?} to {:?}]",
            self.source().position(),
            self.source().lines(),
            self.target().lines()
        )
    }
}

impl<T> From<ChangeDelta<T>> for Delta<T> {
    fn from(change: ChangeDelta<T>) -> Self {
        change.inner
    }
}