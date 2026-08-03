use my_diff_crate::diff_utils::DiffUtils;
use my_diff_crate::patch::conflict_produces_merge_conflict;

#[test]
fn test_patch_change_with_exception_processor() {
    let mut change_test_from = vec![
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

let patch = DiffUtils::diff(&change_test_from, &change_test_to, None)
    .with_conflict_output(conflict_produces_merge_conflict);

    change_test_from[2] = "CDC".to_string();

    let data = DiffUtils::patch(&change_test_from, &patch)
        .expect("Patch application failed");

    assert_eq!(data.len(), 9);

    let expected = vec![
        "aaa".to_string(),
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

#[test]
fn test_patch_three_way_issue_138() {
    let base: Vec<String> = "Imagine there's no heaven"
        .split_whitespace()
        .map(String::from)
        .collect();
    let left: Vec<String> = "Imagine there's no HEAVEN"
        .split_whitespace()
        .map(String::from)
        .collect();
    let right: Vec<String> = "IMAGINE there's no heaven"
        .split_whitespace()
        .map(String::from)
        .collect();

    let right_patch = DiffUtils::diff(&base, &right, None)
    .with_conflict_output(conflict_produces_merge_conflict);

    let applied = DiffUtils::patch(&left, &right_patch)
        .expect("Three-way patch application failed");

    assert_eq!(applied.join(" "), "IMAGINE there's no HEAVEN");
}