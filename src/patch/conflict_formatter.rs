use super::delta::Delta;
use super::error::PatchError;
use super::verify_chunk::VerifyChunk;

/// Generic conflict output function matching `ConflictOutput<T>`.
pub fn CONFLICT_PRODUCES_MERGE_CONFLICT<T>(
    _verify: VerifyChunk,
    _delta: &Delta<T>,
    _result: &mut Vec<T>,
) -> Result<(), PatchError> {
    Err(PatchError::PatchFailed(
        "Merge conflict produced while applying patch".into(),
    ))
}