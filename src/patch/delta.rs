//! Delta representation of sequence modifications between target lists.

use std::fmt;
use serde::{Deserialize, Serialize};

use super::chunk::Chunk;
use super::delta_type::DeltaType;
use super::error::PatchError;
use super::verify_chunk::VerifyChunk;

/// Represents a single modification delta between a source chunk and a target chunk.
#[derive(Serialize, Deserialize,Debug, Clone, PartialEq, Eq, Hash)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct Delta<T> {
    delta_type: DeltaType,
    source: Chunk<T>,
    target: Chunk<T>,
}

impl<T> Delta<T> {
    /// Creates a new `Delta` with the specified type, source, and target chunks.
    pub fn new(delta_type: DeltaType, source: Chunk<T>, target: Chunk<T>) -> Self {
        Self {
            delta_type,
            source,
            target,
        }
    }

    /// Returns a reference to the source chunk.
    #[inline]
    pub fn source(&self) -> &Chunk<T> {
        &self.source
    }

    /// Returns a mutable reference to the source chunk.
    #[inline]
    pub fn source_mut(&mut self) -> &mut Chunk<T> {
        &mut self.source
    }

    /// Returns a reference to the target chunk.
    #[inline]
    pub fn target(&self) -> &Chunk<T> {
        &self.target
    }

    /// Returns a mutable reference to the target chunk.
    #[inline]
    pub fn target_mut(&mut self) -> &mut Chunk<T> {
        &mut self.target
    }

    /// Returns the type of this delta.
    #[inline]
    pub fn delta_type(&self) -> DeltaType {
        self.delta_type
    }

    /// Verifies whether the source chunk of this delta fits the provided target sequence.
    pub fn verify_chunk_to_fit_target(&self, target: &[T]) -> Result<VerifyChunk, PatchError>
    where
        T: PartialEq,
    {
        self.source.verify_chunk(target)
    }

    /// Verifies that the source chunk matches the target sequence and applies the delta in-place if valid.
    pub fn verify_and_apply_to(&self, target: &mut Vec<T>) -> Result<VerifyChunk, PatchError>
    where
        T: Clone + PartialEq,
    {
        let verify = self.verify_chunk_to_fit_target(target)?;
        if verify == VerifyChunk::Ok {
            self.apply_to(target)?;
        }
        Ok(verify)
    }

    /// Applies this delta to the target vector using its source chunk position.
    pub fn apply_to(&self, target: &mut Vec<T>) -> Result<(), PatchError>
    where
        T: Clone + PartialEq,
    {
        self.apply_at(target, self.source.position())
    }

    /// Applies this delta to the target vector starting at an explicit position.
    pub fn apply_at(&self, target: &mut Vec<T>, pos: usize) -> Result<(), PatchError>
    where
        T: Clone + PartialEq,
    {
        if pos > target.len() {
            return Err(PatchError::PatchFailed(format!(
                "Patch position {} out of bounds for target length {}",
                pos,
                target.len()
            )));
        }

        match self.delta_type {
            DeltaType::Delete => {
                let len = self.source.len();
                if pos + len > target.len() {
                    return Err(PatchError::PatchFailed(format!(
                        "Delete delta range [{}..{}] exceeds target length {}",
                        pos,
                        pos + len,
                        target.len()
                    )));
                }
                target.drain(pos..pos + len);
            }
            DeltaType::Insert => {
                let lines = self.target.lines();
                target.splice(pos..pos, lines.iter().cloned());
            }
            DeltaType::Change => {
                let len = self.source.len();
                if pos + len > target.len() {
                    return Err(PatchError::PatchFailed(format!(
                        "Change delta source range [{}..{}] exceeds target length {}",
                        pos,
                        pos + len,
                        target.len()
                    )));
                }
                target.splice(pos..pos + len, self.target.lines().iter().cloned());
            }
            DeltaType::Equal => {}
        }

        Ok(())
    }

    /// Restores (un-applies) this delta, reverting the target sequence back to its original state.
    pub fn restore(&self, target: &mut Vec<T>) -> Result<(), PatchError>
    where
        T: Clone + PartialEq,
    {
        let pos = self.target.position();

        match self.delta_type {
            DeltaType::Delete => {
                let lines = self.source.lines();
                target.splice(pos..pos, lines.iter().cloned());
            }
            DeltaType::Insert => {
                let len = self.target.len();
                if pos + len > target.len() {
                    return Err(PatchError::PatchFailed(format!(
                        "Restore insert delta range [{}..{}] exceeds target length {}",
                        pos,
                        pos + len,
                        target.len()
                    )));
                }
                target.drain(pos..pos + len);
            }
            DeltaType::Change => {
                let len = self.target.len();
                if pos + len > target.len() {
                    return Err(PatchError::PatchFailed(format!(
                        "Restore change delta target range [{}..{}] exceeds target length {}",
                        pos,
                        pos + len,
                        target.len()
                    )));
                }
                target.splice(pos..pos + len, self.source.lines().iter().cloned());
            }
            DeltaType::Equal => {}
        }

        Ok(())
    }

    /// Applies fuzzy patch matching at a given position with context tolerances.
    /// Applies fuzzy patch matching at a given position with context tolerances.
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

        match self.delta_type {
            DeltaType::Delete => {
                let src_len = self.source.len();
                let end = (position + src_len).min(target.len());
                if position < end {
                    target.drain(position..end);
                }
                Ok(())
            }
            DeltaType::Insert => {
                let insert_pos = position.min(target.len());
                let lines = self.target.lines();
                target.splice(insert_pos..insert_pos, lines.iter().cloned());
                Ok(())
            }
            DeltaType::Change => {
                let src_len = self.source.len();
                let target_lines = self.target.lines();
                let end = (position + src_len).min(target.len());

                target.splice(position..end, target_lines.iter().cloned());
                Ok(())
            }
            DeltaType::Equal => Ok(()),
        }
    }

    /// Returns a new instance of `Delta` with customized source and target chunk values.
    #[must_use]
    pub fn with_chunks(&self, source: Chunk<T>, target: Chunk<T>) -> Self {
        Self {
            delta_type: self.delta_type,
            source,
            target,
        }
    }
}

impl<T> AsRef<Delta<T>> for Delta<T> {
    fn as_ref(&self) -> &Delta<T> {
        self
    }
}

impl<T> From<Box<Delta<T>>> for Delta<T> {
    fn from(boxed: Box<Delta<T>>) -> Self {
        *boxed
    }
}

fn format_lines<T: fmt::Display>(lines: &[T]) -> String {
    let formatted_items = lines
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", formatted_items)
}

impl<T: fmt::Display> fmt::Display for Delta<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.delta_type {
            DeltaType::Insert => write!(
                f,
                "[InsertDelta, position: {}, lines: {}]",
                self.source().position(),
                format_lines(self.target().lines())
            ),
            DeltaType::Delete => write!(
                f,
                "[DeleteDelta, position: {}, lines: {}]",
                self.source().position(),
                format_lines(self.source().lines())
            ),
            DeltaType::Change => write!(
                f,
                "[ChangeDelta, position: {}, lines: {} to {}]",
                self.source().position(),
                format_lines(self.source().lines()),
                format_lines(self.target().lines())
            ),
            DeltaType::Equal => write!(
                f,
                "[EqualDelta, position: {}, lines: {}]",
                self.source().position(),
                format_lines(self.source().lines())
            ),
        }
    }
}