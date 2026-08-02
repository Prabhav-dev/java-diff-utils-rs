use super::delta::Delta;
use super::error::PatchError;
use super::verify_chunk::VerifyChunk;

pub fn conflict_produces_merge_conflict<T>(
    _verify: VerifyChunk,
    delta: &Delta<T>,
    result: &mut Vec<T>,
) -> Result<(), PatchError>
where
    T: Clone + From<&'static str>,
{
    let pos = delta.source().position();
    let src_len = delta.source().len();

    if pos > result.len() || pos + src_len > result.len() {
        return Err(PatchError::PatchFailed(format!(
            "Conflict position {} (size {}) out of bounds for target length {}",
            pos, src_len, result.len()
        )));
    }

    // Capture what's actually in the target before we overwrite it.
    let actual: Vec<T> = result[pos..pos + src_len].to_vec();

    // Apply the delta's intended replacement as normal.
    result.splice(pos..pos + src_len, delta.target().lines().iter().cloned());

    // Splice conflict markers in right after the applied replacement.
    let insert_at = pos + delta.target().len();
    let mut marker_block: Vec<T> = Vec::with_capacity(actual.len() + src_len + 3);
    marker_block.push(T::from("<<<<<< HEAD"));
    marker_block.extend(actual);
    marker_block.push(T::from("======"));
    marker_block.extend(delta.source().lines().iter().cloned());
    marker_block.push(T::from(">>>>>>> PATCH"));

    result.splice(insert_at..insert_at, marker_block);

    Ok(())
}