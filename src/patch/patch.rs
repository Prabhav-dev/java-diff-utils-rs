use std::fmt;
use serde::{Deserialize, Serialize};

use crate::algorithm::change::Change;

use super::chunk::Chunk;
use super::conflict_output::ConflictOutput;
use super::delta::Delta;
use super::delta_type::DeltaType;
use super::error::PatchError;
use super::verify_chunk::VerifyChunk;

struct PatchApplyingContext<'a, T> {
    result: &'a mut Vec<T>,
    max_fuzz: usize,
    last_patch_end: isize,
    current_fuzz: usize,
    default_position: usize,
    before_out_range: bool,
    after_out_range: bool,
}

impl<'a, T> PatchApplyingContext<'a, T> {
    fn new(result: &'a mut Vec<T>, max_fuzz: usize) -> Self {
        Self {
            result,
            max_fuzz,
            last_patch_end: -1,
            current_fuzz: 0,
            default_position: 0,
            before_out_range: false,
            after_out_range: false,
        }
    }
}

/// Represents a collection of deltas to transform a source sequence into a target sequence.
#[derive(Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct Patch<T> {
    deltas: Vec<Delta<T>>,
    #[serde(skip, default)]
    conflict_output: Option<Box<dyn ConflictOutput<T>>>,
}

impl<T: fmt::Debug> fmt::Debug for Patch<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Patch")
            .field("deltas", &self.deltas)
            .field("has_conflict_output", &self.conflict_output.is_some())
            .finish()
    }
}

impl<T> Clone for Patch<T>
where
    Delta<T>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            deltas: self.deltas.clone(),
            conflict_output: None,
        }
    }
}

impl<T: PartialEq> PartialEq for Patch<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deltas == other.deltas
    }
}

impl<T: Eq> Eq for Patch<T> {}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Patch<T> {
    /// Creates a new empty `Patch`.
    pub fn new() -> Self {
        Self::with_capacity(10)
    }

    /// Creates a new empty `Patch` with a pre-allocated delta capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            deltas: Vec::with_capacity(capacity),
            conflict_output: None,
        }
    }

    /// Configures custom conflict resolution output behavior.
    #[must_use]
    pub fn with_conflict_output<C>(mut self, conflict_output: C) -> Self
    where
        C: ConflictOutput<T> + 'static,
    {
        self.conflict_output = Some(Box::new(conflict_output));
        self
    }

    /// Appends a new delta modification record to this patch.
    pub fn add_delta(&mut self, delta: impl Into<Delta<T>>) {
        self.deltas.push(delta.into());
    }

    /// Returns an immutable slice reference to the deltas.
    pub fn get_deltas(&self) -> &[Delta<T>] {
        &self.deltas
    }

    /// Returns a slice reference of deltas contained in this patch.
    pub fn deltas(&self) -> &[Delta<T>] {
        &self.deltas
    }

    /// Returns a mutable slice reference to the deltas.
    pub fn deltas_mut(&mut self) -> &mut [Delta<T>] {
        &mut self.deltas
    }

    /// Sorts internal deltas in-place by source chunk position.
    pub fn sort_deltas(&mut self) {
        self.deltas.sort_by_key(|d| d.source().position());
    }

    /// Applies this patch to a slice, returning a new patched vector.
    pub fn apply_to(&self, target: &[T]) -> Result<Vec<T>, PatchError>
    where
        T: Clone + PartialEq,
    {
        let mut result = target.to_vec();
        self.apply_to_existing(&mut result)?;
        Ok(result)
    }

    /// Applies this patch in-place to an existing vector using shared `&self`.
    pub fn apply_to_existing(&self, target: &mut Vec<T>) -> Result<(), PatchError>
    where
        T: Clone + PartialEq,
    {
        let mut sorted_deltas: Vec<&Delta<T>> = self.deltas.iter().collect();
        sorted_deltas.sort_by_key(|d| d.source().position());

        for delta in sorted_deltas.into_iter().rev() {
            let valid = delta.verify_and_apply_to(target)?;

            if valid != VerifyChunk::Ok {
                if let Some(ref handler) = self.conflict_output {
                    handler.process_conflict(valid, delta, target)?;
                } else {
                    return Err(PatchError::PatchFailed(format!(
                        "Could not apply patch due to {:?}",
                        valid
                    )));
                }
            }
        }

        Ok(())
    }

    /// Restores (un-applies) this patch on a target slice, returning a new restored vector.
    pub fn restore(&self, target: &[T]) -> Result<Vec<T>, PatchError>
    where
        T: Clone + PartialEq,
    {
        let mut result = target.to_vec();
        self.restore_to_existing(&mut result)?;
        Ok(result)
    }

    /// Restores changes in-place on an existing vector using shared `&self`.
    pub fn restore_to_existing(&self, target: &mut Vec<T>) -> Result<(), PatchError>
    where
        T: Clone + PartialEq,
    {
        let mut sorted_deltas: Vec<&Delta<T>> = self.deltas.iter().collect();
        sorted_deltas.sort_by_key(|d| d.source().position());

        for delta in sorted_deltas.into_iter().rev() {
            delta.restore(target)?;
        }

        Ok(())
    }

    /// Applies this patch using fuzzy context matching.
    pub fn apply_fuzzy(&self, target: &[T], max_fuzz: usize) -> Result<Vec<T>, PatchError>
    where
        T: Clone + PartialEq,
    {
        let mut result = target.to_vec();
        let mut ctx = PatchApplyingContext::new(&mut result, max_fuzz);

        let mut sorted_deltas: Vec<&Delta<T>> = self.deltas.iter().collect();
        sorted_deltas.sort_by_key(|d| d.source().position());

        let mut cumulative_offset: isize = 0;

        for delta in sorted_deltas {
            let src_pos = delta.source().position() as isize;
            let default_pos = src_pos + cumulative_offset;

            if default_pos < 0 {
                if let Some(ref handler) = self.conflict_output {
                    handler.process_conflict(
                        VerifyChunk::ContentDoesNotMatchTarget,
                        delta,
                        ctx.result,
                    )?;
                } else {
                    return Err(PatchError::PatchFailed(
                        "Negative fuzzy offset invalid for target sequence".into(),
                    ));
                }
                continue;
            }

            ctx.default_position = default_pos as usize;

            if let Some(patch_position) = find_position_fuzzy(&mut ctx, delta)? {
                let old_len = ctx.result.len();
                delta.apply_fuzzy_to_at(ctx.result, ctx.current_fuzz, patch_position)?;
                let new_len = ctx.result.len();

                let found_slop = patch_position as isize - default_pos;
                let length_delta = (new_len as isize) - (old_len as isize);
                cumulative_offset += found_slop + length_delta;

                // Detect a pure Deletion without needing a DeltaType enum:
                // If the applied change shrunk the array by exactly the size of the source chunk,
                // it is a Delete delta, meaning its footprint in the resulting array is 0.
                let src_len = delta.source().len();
                let is_delete = src_len > 0 && length_delta == -(src_len as isize);
                
                let effective_source_len = if is_delete { 0 } else { src_len };

                ctx.last_patch_end = patch_position as isize + effective_source_len as isize;
            } else if let Some(ref handler) = self.conflict_output {
                handler.process_conflict(
                    VerifyChunk::ContentDoesNotMatchTarget,
                    delta,
                    ctx.result,
                )?;
            } else {
                return Err(PatchError::PatchFailed(format!(
                    "Could not find fuzzy match position for delta at position {}",
                    delta.source().position()
                )));
            }
        }

        Ok(result)
    }

    /// Constructs a `Patch` from sequences and raw algorithm `Change` records.
    pub fn generate(
        original: &[T],
        revised: &[T],
        changes: &[Change],
        include_equals: bool,
    ) -> Self
    where
        T: Clone,
    {
        let mut patch = Self::with_capacity(changes.len());
        let mut start_original = 0;
        let mut start_revised = 0;

        let mut sorted_changes = changes.to_vec();
        sorted_changes.sort_by_key(|c| c.start_original);

        for change in &sorted_changes {
            if include_equals && start_original < change.start_original {
                patch.add_delta(Delta::new(
                    DeltaType::Equal,
                    build_chunk(start_original, change.start_original, original),
                    build_chunk(start_revised, change.start_revised, revised),
                ));
            }

            let org_chunk = build_chunk(change.start_original, change.end_original, original);
            let rev_chunk = build_chunk(change.start_revised, change.end_revised, revised);

            patch.add_delta(Delta::new(change.delta_type, org_chunk, rev_chunk));

            start_original = change.end_original;
            start_revised = change.end_revised;
        }

        if include_equals && start_original < original.len() {
            patch.add_delta(Delta::new(
                DeltaType::Equal,
                build_chunk(start_original, original.len(), original),
                build_chunk(start_revised, revised.len(), revised),
            ));
        }

        patch
    }
}

fn build_chunk<T: Clone>(start: usize, end: usize, data: &[T]) -> Chunk<T> {
    let lines = if start < end && start < data.len() {
        let actual_end = end.min(data.len());
        data[start..actual_end].to_vec()
    } else {
        Vec::new()
    };
    Chunk::with_lines(start, lines)
}

fn find_position_fuzzy<T: PartialEq>(
    ctx: &mut PatchApplyingContext<'_, T>,
    delta: &Delta<T>,
) -> Result<Option<usize>, PatchError> {
    for fuzz in 0..=ctx.max_fuzz {
        ctx.current_fuzz = fuzz;
        if let Some(pos) = find_position_with_fuzz(ctx, delta, fuzz)? {
            return Ok(Some(pos));
        }
    }
    Ok(None)
}

fn find_position_with_fuzz<T: PartialEq>(
    ctx: &mut PatchApplyingContext<'_, T>,
    delta: &Delta<T>,
    fuzz: usize,
) -> Result<Option<usize>, PatchError> {
    ctx.before_out_range = false;
    ctx.after_out_range = false;

    let mut more_delta = 0_usize;
    loop {
        if let Some(pos) = find_position_with_fuzz_and_more_delta(ctx, delta, fuzz, more_delta)? {
            return Ok(Some(pos));
        }

        if ctx.before_out_range && ctx.after_out_range {
            break;
        }

        match more_delta.checked_add(1) {
            Some(next) => more_delta = next,
            None => break,
        }
    }

    Ok(None)
}

fn find_position_with_fuzz_and_more_delta<T: PartialEq>(
    ctx: &mut PatchApplyingContext<'_, T>,
    delta: &Delta<T>,
    fuzz: usize,
    more_delta: usize,
) -> Result<Option<usize>, PatchError> {
    if !ctx.before_out_range {
        if ctx.default_position < more_delta {
            ctx.before_out_range = true;
        } else {
            let begin_at = ctx.default_position - more_delta;
            let begin_at_isize = begin_at as isize;

            if begin_at_isize < ctx.last_patch_end {
                ctx.before_out_range = true;
            }
        }
    }

    if !ctx.after_out_range {
        let src_len = delta.source().len();
        let effective_len = if src_len > 2 * fuzz { src_len - 2 * fuzz } else { 0 };
        let begin_at = ctx.default_position + more_delta + effective_len;
        if begin_at > ctx.result.len() {
            ctx.after_out_range = true;
        }
    }

    if !ctx.before_out_range {
        let test_pos = ctx.default_position - more_delta;
        let before = delta.source().verify_chunk_at(ctx.result, fuzz, test_pos)?;
        if before == VerifyChunk::Ok {
            return Ok(Some(test_pos));
        }
    }

    if !ctx.after_out_range && more_delta > 0 {
        let test_pos = ctx.default_position + more_delta;
        let after = delta.source().verify_chunk_at(ctx.result, fuzz, test_pos)?;
        if after == VerifyChunk::Ok {
            return Ok(Some(test_pos));
        }
    }

    Ok(None)
}

impl<T: fmt::Display> fmt::Display for Patch<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Patch{{deltas=[")?;
        for (i, d) in self.deltas.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", d)?;
        }
        write!(f, "]}}")
    }
}