use my_diff_crate::patch::chunk::Chunk;
use my_diff_crate::patch::verify_chunk::VerifyChunk;

fn to_char_list(s: &str) -> Vec<char> {
    s.chars().collect()
}

#[test]
fn test_verify_chunk() {
    let chunk = Chunk::new(7, to_char_list("test"), None);

    // Normal check
    assert_eq!(
        chunk.verify_chunk(&to_char_list("prefix test suffix")),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk(&to_char_list("prefix  es  suffix")),
        Ok(VerifyChunk::ContentDoesNotMatchTarget)
    );

    // Position checks
    assert_eq!(
        chunk.verify_chunk(&to_char_list("short test suffix")),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk(&to_char_list("loonger test suffix")),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk(&to_char_list("prefix test suffix")),
        Ok(VerifyChunk::ContentDoesNotMatchTarget)
    );
    assert_eq!(
        chunk.verify_chunk(&to_char_list("prefix test suffix")),
        Ok(VerifyChunk::ContentDoesNotMatchTarget)
    );

    // Fuzz checks
    assert_eq!(
        chunk.verify_chunk(&to_char_list("prefix test suffix")),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk(&to_char_list("prefix  es  suffix")),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk(&to_char_list("prefix      suffix")),
        Ok(VerifyChunk::ContentDoesNotMatchTarget)
    );
}