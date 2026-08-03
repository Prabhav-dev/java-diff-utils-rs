use std::fmt;
use serde::{Deserialize, Serialize};

use super::error::PatchError;
use super::verify_chunk::VerifyChunk;

/// Represents a contiguous sub-sequence (chunk) of items participating in a diff/patch operation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct Chunk<T> {
    position: usize,
    lines: Vec<T>,
    change_position: Option<Vec<usize>>,
}

impl<T> Chunk<T> {
    /// Creates a new `Chunk` with a starting position, lines, and optional positions of modified lines.
    pub fn new(position: usize, lines: Vec<T>, change_position: Option<Vec<usize>>) -> Self {
        Self {
            position,
            lines,
            change_position,
        }
    }

    /// Convenience constructor creating a `Chunk` without granular line-change tracking indices.
    pub fn with_lines(position: usize, lines: Vec<T>) -> Self {
        Self::new(position, lines, None)
    }

    /// Verifies that this chunk's saved lines match the corresponding region in the target slice.
    pub fn verify_chunk(&self, target: &[T]) -> Result<VerifyChunk, PatchError>
    where
        T: PartialEq,
    {
        self.verify_chunk_at(target, 0, self.position)
    }

    /// Verifies that this chunk matches the target sequence starting at `position`, taking `fuzz` context into account.
    pub fn verify_chunk_at(
        &self,
        target: &[T],
        fuzz: usize,
        position: usize,
    ) -> Result<VerifyChunk, PatchError>
    where
        T: PartialEq,
    {
        let start_index = fuzz;
        let last_index = self.len().saturating_sub(fuzz);
        let last = position + self.len().saturating_sub(1);

        if position.saturating_add(fuzz) > target.len() || last.saturating_sub(fuzz) > target.len()
        {
            return Ok(VerifyChunk::PositionOutOfTarget);
        }

        for i in start_index..last_index {
            let target_idx = position + i;
            if target_idx >= target.len() || target[target_idx] != self.lines[i] {
                return Ok(VerifyChunk::ContentDoesNotMatchTarget);
            }
        }

        Ok(VerifyChunk::Ok)
    }
    /// Returns the zero-based start position of this chunk.
    #[inline]
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns a slice reference to the chunk's lines.
    #[inline]
    pub fn lines(&self) -> &[T] {
        &self.lines
    }

    /// Returns a mutable slice reference to the chunk's lines.
    #[inline]
    pub fn lines_mut(&mut self) -> &mut Vec<T> {
        &mut self.lines
    }

    /// Sets or replaces the lines stored inside this chunk.
    pub fn set_lines(&mut self, lines: Vec<T>) {
        self.lines = lines;
    }

    /// Returns an optional slice reference to the change position indices, if present.
    #[inline]
    pub fn change_position(&self) -> Option<&[usize]> {
        self.change_position.as_deref()
    }

    /// Returns the number of lines contained in this chunk.
    #[inline]
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Alias method for `len` matching common Java/C# diff library APIS.
    #[inline]
    pub fn size(&self) -> usize {
        self.lines.len()
    }

    /// Returns `true` if this chunk contains no lines.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Returns the zero-based index of the last line in the chunk (if non-empty).
    #[inline]
    pub fn last(&self) -> usize {
        if self.lines.is_empty() {
            self.position
        } else {
            self.position + self.lines.len() - 1
        }
    }
}

impl<T: fmt::Debug> fmt::Display for Chunk<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[position: {}, size: {}, lines: {:?}]",
            self.position,
            self.len(),
            self.lines
        )
    }
}