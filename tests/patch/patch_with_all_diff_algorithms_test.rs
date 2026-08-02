use my_diff_crate::algorithm::myers::myers::MyersDiff;
use my_diff_crate::algorithm::DiffAlgorithm;
use my_diff_crate::diff_utils::DiffUtils;
use my_diff_crate::patch::patch::Patch;

// MyersDiffWithLinearSpace is imported from your module tree if available, or MyersDiff fallback
fn get_algorithms() -> Vec<Box<dyn DiffAlgorithm<String>>> {
    vec![
        Box::new(MyersDiff::new()),
    ]
}

#[test]
fn test_patch_insert() {
    for algo in get_algorithms() {
        let insert_test_from = vec!["hhh".to_string()];
        let insert_test_to = vec![
            "hhh".to_string(),
            "jjj".to_string(),
            "kkk".to_string(),
            "lll".to_string(),
        ];

        let patch = DiffUtils::diff_with_algorithm(&insert_test_from, &insert_test_to, algo.as_ref(), None, false);
        let result = DiffUtils::patch(&insert_test_from, &patch)
            .expect("Patch application failed for Insert test");

        assert_eq!(result, insert_test_to);
    }
}

#[test]
fn test_patch_delete() {
    for algo in get_algorithms() {
        let delete_test_from = vec![
            "ddd".to_string(),
            "fff".to_string(),
            "ggg".to_string(),
            "hhh".to_string(),
        ];
        let delete_test_to = vec!["ggg".to_string()];

        let patch = DiffUtils::diff_with_algorithm(&delete_test_from, &delete_test_to, algo.as_ref(), None, false);
        let result = DiffUtils::patch(&delete_test_from, &patch)
            .expect("Patch application failed for Delete test");

        assert_eq!(result, delete_test_to);
    }
}

#[test]
fn test_patch_change() {
    for algo in get_algorithms() {
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

        let patch = DiffUtils::diff_with_algorithm(&change_test_from, &change_test_to, algo.as_ref(), None, false);
        let result = DiffUtils::patch(&change_test_from, &patch)
            .expect("Patch application failed for Change test");

        assert_eq!(result, change_test_to);
    }
}

#[test]
fn test_patch_serializable() {
    for algo in get_algorithms() {
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

        let patch = DiffUtils::diff_with_algorithm(&change_test_from, &change_test_to, algo.as_ref(), None, false);

        let serialized = bincode::serialize(&patch)
            .expect("Failed to serialize Patch struct");
        let deserialized_patch: Patch<String> = bincode::deserialize(&serialized)
            .expect("Failed to deserialize Patch struct");

        let result = DiffUtils::patch(&change_test_from, &deserialized_patch)
            .expect("Patch application failed after deserialization");

        assert_eq!(result, change_test_to);
    }
}