use my_diff_crate::patch::chunk::Chunk;
use my_diff_crate::patch::verify_chunk::VerifyChunk;

fn to_char_list(s: &str) -> Vec<char> {
    s.chars().collect()
}

#[test]
fn test_verify_chunk() {
    let chunk = Chunk::new(7, to_char_list("test"), None);

    // normal check
    assert_eq!(
        chunk.verify_chunk(&to_char_list("prefix test suffix")),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk_at(&to_char_list("prefix  es  suffix"), 0, 7),
        Ok(VerifyChunk::ContentDoesNotMatchTarget)
    );

    // position
    assert_eq!(
        chunk.verify_chunk_at(&to_char_list("short test suffix"), 0, 6),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk_at(&to_char_list("loonger test suffix"), 0, 8),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk_at(&to_char_list("prefix test suffix"), 0, 6),
        Ok(VerifyChunk::ContentDoesNotMatchTarget)
    );
    assert_eq!(
        chunk.verify_chunk_at(&to_char_list("prefix test suffix"), 0, 8),
        Ok(VerifyChunk::ContentDoesNotMatchTarget)
    );

    // fuzz
    assert_eq!(
        chunk.verify_chunk_at(&to_char_list("prefix test suffix"), 1, 7),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk_at(&to_char_list("prefix  es  suffix"), 1, 7),
        Ok(VerifyChunk::Ok)
    );
    assert_eq!(
        chunk.verify_chunk_at(&to_char_list("prefix      suffix"), 1, 7),
        Ok(VerifyChunk::ContentDoesNotMatchTarget)
    );
}