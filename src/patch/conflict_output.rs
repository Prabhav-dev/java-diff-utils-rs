use super::delta::Delta;
use super::error::PatchError;
use super::verify_chunk::VerifyChunk;

/// Handler function trait for processing diff/patch conflicts.
pub trait ConflictOutput<T> {
    /// Processes a conflict that occurred while trying to apply a delta.
    fn process_conflict(
        &self,
        verify_chunk: VerifyChunk,
        delta: &Delta<T>,
        result: &mut Vec<T>,
    ) -> Result<(), PatchError>;
}

// Blanket implementation allowing closures to be used as `ConflictOutput`.
impl<T, F> ConflictOutput<T> for F
where
    F: Fn(VerifyChunk, &Delta<T>, &mut Vec<T>) -> Result<(), PatchError>,
{
    fn process_conflict(
        &self,
        verify_chunk: VerifyChunk,
        delta: &Delta<T>,
        result: &mut Vec<T>,
    ) -> Result<(), PatchError> {
        (self)(verify_chunk, delta, result)
    }
}