use my_diff_crate::algorithm::myers::myers::MyersDiff;
use my_diff_crate::diff_utils::DiffUtils;
use my_diff_crate::patch::conflict_produces_merge_conflict;

#[test]
fn test_patch_change_with_exception_processor() {
    let change_test_from = vec![
        "aaa".to_string(),
        "bbb".to_string(),
        "ccc".to_string(),
        "ddd".to_string(),
    ];
    let change_test_to = vec![
        "aaa".to_string(),
        "bxb".to_string(),
        "cxc".to_string(),
        "ddd".to_string(),
    ];

    let algo = MyersDiff::new();
    let patch = DiffUtils::diff_with_algorithm(&change_test_from, &change_test_to, &algo, None, false)
    .with_conflict_output(conflict_produces_merge_conflict);

    let data = DiffUtils::patch(&change_test_from, &patch)
        .expect("Patch with conflict output failed");

    assert_eq!(data.len(), 11);

    let expected = vec![
        "aaa".to_string(),
        "bxb".to_string(),
        "cxc".to_string(),
        "<<<<<< HEAD".to_string(),
        "bbb".to_string(),
        "CDC".to_string(),
        "======".to_string(),
        "bbb".to_string(),
        "ccc".to_string(),
        ">>>>>>> PATCH".to_string(),
        "ddd".to_string(),
    ];

    assert_eq!(data, expected);
}