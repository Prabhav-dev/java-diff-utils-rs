use my_diff_crate::diff_utils::DiffUtils;
use my_diff_crate::patch::Chunk;

#[test]
fn test_diff_insert() {
    let source = vec!["hhh"];
    let target = vec!["hhh", "jjj", "kkk"];

    let patch = DiffUtils::diff(&source, &target, None);
    let deltas = patch.deltas();

    assert_eq!(deltas.len(), 1);
    let delta = &deltas[0];

    assert_eq!(delta.source(), &Chunk::new(1, vec![], None));
    assert_eq!(delta.target(), &Chunk::new(1, vec!["jjj", "kkk"], None));
}

#[test]
fn test_diff_delete() {
    let source = vec!["ddd", "fff"];
    let target = vec![];

    let patch = DiffUtils::diff(&source, &target, None);
    let deltas = patch.deltas();

    assert_eq!(deltas.len(), 1);
    let delta = &deltas[0];

    assert_eq!(delta.source(), &Chunk::new(0, vec!["ddd", "fff"], None));
    assert_eq!(delta.target(), &Chunk::new(0, vec![], None));
}

#[test]
fn test_diff_change() {
    let source = vec!["aaa", "bbb", "ccc"];
    let target = vec!["aaa", "zzz", "ccc"];

    let patch = DiffUtils::diff(&source, &target, None);
    let deltas = patch.deltas();

    assert_eq!(deltas.len(), 1);
    let delta = &deltas[0];

    assert_eq!(delta.source(), &Chunk::new(1, vec!["bbb"], None));
    assert_eq!(delta.target(), &Chunk::new(1, vec!["zzz"], None));
}

#[test]
fn test_diff_equal() {
    let source = vec!["hhh", "jjj", "kkk"];
    let target = vec!["hhh", "jjj", "kkk"];

    let patch = DiffUtils::diff(&source, &target, None);
    let deltas = patch.deltas();

    assert_eq!(deltas.len(), 1);
    let delta = &deltas[0];

    assert_eq!(delta.source(), &Chunk::new(0, vec!["hhh", "jjj", "kkk"], None));
    assert_eq!(delta.target(), &Chunk::new(0, vec!["hhh", "jjj", "kkk"], None));
}

#[test]
fn test_diff_multiple_deltas() {
    let source = vec!["hhh"];
    let target = vec!["hhh", "jjj", "kkk"];

    let patch = DiffUtils::diff(&source, &target, None);
    let deltas = patch.deltas();

    assert_eq!(deltas.len(), 2);

    let delta0 = &deltas[0];
    assert_eq!(delta0.source(), &Chunk::new(0, vec!["hhh"], None));
    assert_eq!(delta0.target(), &Chunk::new(0, vec!["hhh"], None));

    let delta1 = &deltas[1];
    assert_eq!(delta1.source(), &Chunk::new(1, vec![], None));
    assert_eq!(delta1.target(), &Chunk::new(1, vec!["jjj", "kkk"], None));
}

#[test]
fn test_delta_type_names() {
    let source = vec!["The", "dog", "is", "brown"];
    let target = vec!["The", "fox", "is", "down"];

    let patch = DiffUtils::diff(&source, &target, None);
    
    let types: Vec<_> = patch
        .deltas()
        .iter()
        .map(|d| d.delta_type())
        .collect();

    assert!(!types.is_empty());

    let delta0 = &patch.deltas()[0];
    assert_eq!(delta0.source(), &Chunk::new(0, vec!["The"], None));
    assert_eq!(delta0.target(), &Chunk::new(0, vec!["The"], None));

    let delta1 = &patch.deltas()[1];
    assert_eq!(delta1.source(), &Chunk::new(1, vec!["dog"], None));
    assert_eq!(delta1.target(), &Chunk::new(1, vec!["fox"], None));

    let delta2 = &patch.deltas()[2];
    assert_eq!(delta2.source(), &Chunk::new(2, vec!["is"], None));
    assert_eq!(delta2.target(), &Chunk::new(2, vec!["is"], None));

    let delta3 = &patch.deltas()[3];
    assert_eq!(delta3.source(), &Chunk::new(3, vec!["brown"], None));
    assert_eq!(delta3.target(), &Chunk::new(3, vec!["down"], None));
}