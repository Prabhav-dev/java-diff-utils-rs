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

    if result.len() > pos {
        // Remove the actual content currently at the delta's source position
        // (this may differ from what the delta expected, which is why we're here).
        let end = (pos + src_len).min(result.len());
        let actual: Vec<T> = result.splice(pos..end, std::iter::empty()).collect();

        // Build the merge-conflict block: actual content vs. the patch's
        // original (source) content. The target/revised replacement is
        // intentionally never inserted here, matching upstream behavior.
        let mut org_data: Vec<T> = Vec::with_capacity(actual.len() + src_len + 3);
        org_data.push(T::from("<<<<<< HEAD"));
        org_data.extend(actual);
        org_data.push(T::from("======"));
        org_data.extend(delta.source().lines().iter().cloned());
        org_data.push(T::from(">>>>>>> PATCH"));

        result.splice(pos..pos, org_data);

        Ok(())
    } else {
        Err(PatchError::PatchFailed("Not supported yet.".into()))
    }
}