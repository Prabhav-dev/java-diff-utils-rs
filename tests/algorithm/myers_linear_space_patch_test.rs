// tests/algorithm/myers_linear_space_patch_test.rs

//! Transpiled Unit Tests for `com.github.difflib.algorithm.myers.WithMyersDiffWithLinearSpacePatchTest`

use my_diff_crate::algorithm::myers::compute_diff as compute_diff_linear;
use my_diff_crate::patch::conflict_formatter::CONFLICT_PRODUCES_MERGE_CONFLICT;
use my_diff_crate::patch::{
    error::PatchFailedException,
    ChangeDelta, Chunk, Patch,
};

#[test]
fn test_patch_insert() -> Result<(), PatchFailedException> {
    let insert_test_from = vec!["hhh"];
    let insert_test_to = vec!["hhh", "jjj", "kkk", "lll"];

    let changes = compute_diff_linear(&insert_test_from, &insert_test_to);
    let patch = Patch::generate(&insert_test_from, &insert_test_to, &changes, false);

    let patched = patch.apply_to(&insert_test_from)?;
    assert_eq!(patched, insert_test_to);
    Ok(())
}

#[test]
fn test_patch_delete() -> Result<(), PatchFailedException> {
    let delete_test_from = vec!["ddd", "fff", "ggg", "hhh"];
    let delete_test_to = vec!["ggg"];

    let changes = compute_diff_linear(&delete_test_from, &delete_test_to);
    let patch = Patch::generate(&delete_test_from, &delete_test_to, &changes, false);

    let patched = patch.apply_to(&delete_test_from)?;
    assert_eq!(patched, delete_test_to);
    Ok(())
}

#[test]
fn test_patch_change() -> Result<(), PatchFailedException> {
    let change_test_from = vec!["aaa", "bbb", "ccc", "ddd"];
    let change_test_to = vec!["aaa", "bxb", "cxc", "ddd"];

    let changes = compute_diff_linear(&change_test_from, &change_test_to);
    let patch = Patch::generate(&change_test_from, &change_test_to, &changes, false);

    let patched = patch.apply_to(&change_test_from)?;
    assert_eq!(patched, change_test_to);
    Ok(())
}

// region fuzzy apply utils

fn int_range(count: usize) -> Vec<String> {
    (0..count).map(|i| i.to_string()).collect()
}

fn join(lists: &[&[String]]) -> Vec<String> {
    lists.iter().flat_map(|l| l.iter().cloned()).collect()
}

struct FuzzyApplyTestPair {
    from: Vec<String>,
    to: Vec<String>,
    required_fuzz: usize,
}

// endregion

#[test]
fn test_fuzzy_apply() {
    let mut patch = Patch::default();
    let delta_from: Vec<String> = vec!["aaa", "bbb", "ccc", "ddd", "eee", "fff"]
        .into_iter()
        .map(String::from)
        .collect();
    let delta_to: Vec<String> = vec!["aaa", "bbb", "cxc", "dxd", "eee", "fff"]
        .into_iter()
        .map(String::from)
        .collect();

    patch.add_delta(ChangeDelta::new(
        Chunk::new(6, delta_from.clone(), None),
        Chunk::new(6, delta_to.clone(), None),
    ));

    let moves: Vec<Vec<String>> = vec![
        int_range(6), // no patch move
        int_range(3), // forward patch move
        int_range(9), // backward patch move
        int_range(0), // apply to the first
    ];

    for pair in FUZZY_APPLY_TEST_PAIRS.iter() {
        for move_prefix in &moves {
            let from = join(&[move_prefix, &pair.from]);
            let to = join(&[move_prefix, &pair.to]);

            for i in 0..pair.required_fuzz {
                let max_fuzz = i;
                let result = patch.apply_fuzzy(&from, max_fuzz);
                assert!(
                    result.is_err(),
                    "Expected failure for {:?} -> {:?} with max_fuzz {} (required: {})",
                    from,
                    to,
                    max_fuzz,
                    pair.required_fuzz
                );
            }

            for max_fuzz in pair.required_fuzz..4 {
                let result = patch.apply_fuzzy(&from, max_fuzz);
                assert_eq!(
                    result.unwrap(),
                    to,
                    "Failed fuzzy apply with max_fuzz {}",
                    max_fuzz
                );
            }
        }
    }
}

#[test]
fn test_fuzzy_apply_two_side_by_side_patches() -> Result<(), PatchFailedException> {
    let mut patch = Patch::default();
    let delta_from: Vec<String> = vec!["aaa", "bbb", "ccc", "ddd", "eee", "fff"]
        .into_iter()
        .map(String::from)
        .collect();
    let delta_to: Vec<String> = vec!["aaa", "bbb", "cxc", "dxd", "eee", "fff"]
        .into_iter()
        .map(String::from)
        .collect();

    patch.add_delta(ChangeDelta::new(
        Chunk::new(0, delta_from.clone(), None),
        Chunk::new(0, delta_to.clone(), None),
    ));
    patch.add_delta(ChangeDelta::new(
        Chunk::new(6, delta_from.clone(), None),
        Chunk::new(6, delta_to.clone(), None),
    ));

    let input = join(&[&delta_from, &delta_from]);
    let expected = join(&[&delta_to, &delta_to]);

    let result = patch.apply_fuzzy(&input, 0)?;
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_fuzzy_apply_to_nearest() -> Result<(), PatchFailedException> {
    let mut patch = Patch::default();
    let delta_from: Vec<String> = vec!["aaa", "bbb", "ccc", "ddd", "eee", "fff"]
        .into_iter()
        .map(String::from)
        .collect();
    let delta_to: Vec<String> = vec!["aaa", "bbb", "cxc", "dxd", "eee", "fff"]
        .into_iter()
        .map(String::from)
        .collect();

    patch.add_delta(ChangeDelta::new(
        Chunk::new(0, delta_from.clone(), None),
        Chunk::new(0, delta_to.clone(), None),
    ));
    patch.add_delta(ChangeDelta::new(
        Chunk::new(10, delta_from.clone(), None),
        Chunk::new(10, delta_to.clone(), None),
    ));

    let input1 = join(&[&delta_from, &delta_from, &delta_from]);
    let expected1 = join(&[&delta_to, &delta_from, &delta_to]);
    assert_eq!(patch.apply_fuzzy(&input1, 0)?, expected1);

    let prefix = int_range(1);
    let input2 = join(&[&prefix, &delta_from, &delta_from, &delta_from]);
    let expected2 = join(&[&prefix, &delta_to, &delta_from, &delta_to]);
    assert_eq!(patch.apply_fuzzy(&input2, 0)?, expected2);

    Ok(())
}

#[test]
fn test_patch_change_with_exception_processor() -> Result<(), PatchFailedException> {
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

    let changes = compute_diff_linear(&change_test_from, &change_test_to);

    // Chained the builder method to consume ownership properly
    let patch = Patch::generate(&change_test_from, &change_test_to, &changes, false)
        .with_conflict_output(CONFLICT_PRODUCES_MERGE_CONFLICT);

    // Simulate conflict: modifying source sequence before applying
    change_test_from[2] = "CDC".to_string();

    let data = patch.apply_to(&change_test_from)?;
    assert_eq!(data.len(), 11);

    let expected = vec![
        "aaa",
        "bxb",
        "cxc",
        "<<<<<< HEAD",
        "bbb",
        "CDC",
        "======",
        "bbb",
        "ccc",
        ">>>>>>> PATCH",
        "ddd",
    ];

    assert_eq!(data, expected);
    Ok(())
}

lazy_static::lazy_static! {
    static ref FUZZY_APPLY_TEST_PAIRS: Vec<FuzzyApplyTestPair> = vec![
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bbb".into(), "ccc".into(), "ddd".into(), "eee".into(), "fff".into()],
            to: vec!["aaa".into(), "bbb".into(), "cxc".into(), "dxd".into(), "eee".into(), "fff".into()],
            required_fuzz: 0,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bbb".into(), "ccc".into(), "ddd".into(), "eee".into(), "fff".into()],
            to: vec!["axa".into(), "bbb".into(), "cxc".into(), "dxd".into(), "eee".into(), "fff".into()],
            required_fuzz: 1,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bbb".into(), "ccc".into(), "ddd".into(), "eee".into(), "fxf".into()],
            to: vec!["aaa".into(), "bbb".into(), "cxc".into(), "dxd".into(), "eee".into(), "fxf".into()],
            required_fuzz: 1,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bbb".into(), "ccc".into(), "ddd".into(), "eee".into(), "fxf".into()],
            to: vec!["axa".into(), "bbb".into(), "cxc".into(), "dxd".into(), "eee".into(), "fxf".into()],
            required_fuzz: 1,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bxb".into(), "ccc".into(), "ddd".into(), "eee".into(), "fff".into()],
            to: vec!["aaa".into(), "bxb".into(), "cxc".into(), "dxd".into(), "eee".into(), "fff".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bxb".into(), "ccc".into(), "ddd".into(), "eee".into(), "fff".into()],
            to: vec!["axa".into(), "bxb".into(), "cxc".into(), "dxd".into(), "eee".into(), "fff".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bbb".into(), "ccc".into(), "ddd".into(), "exe".into(), "fff".into()],
            to: vec!["aaa".into(), "bbb".into(), "cxc".into(), "dxd".into(), "exe".into(), "fff".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bbb".into(), "ccc".into(), "ddd".into(), "exe".into(), "fff".into()],
            to: vec!["axa".into(), "bbb".into(), "cxc".into(), "dxd".into(), "exe".into(), "fff".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bxb".into(), "ccc".into(), "ddd".into(), "exe".into(), "fff".into()],
            to: vec!["aaa".into(), "bxb".into(), "cxc".into(), "dxd".into(), "exe".into(), "fff".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bxb".into(), "ccc".into(), "ddd".into(), "exe".into(), "fff".into()],
            to: vec!["axa".into(), "bxb".into(), "cxc".into(), "dxd".into(), "exe".into(), "fff".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bxb".into(), "ccc".into(), "ddd".into(), "eee".into(), "fxf".into()],
            to: vec!["aaa".into(), "bxb".into(), "cxc".into(), "dxd".into(), "eee".into(), "fxf".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bxb".into(), "ccc".into(), "ddd".into(), "eee".into(), "fxf".into()],
            to: vec!["axa".into(), "bxb".into(), "cxc".into(), "dxd".into(), "eee".into(), "fxf".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bbb".into(), "ccc".into(), "ddd".into(), "exe".into(), "fxf".into()],
            to: vec!["aaa".into(), "bbb".into(), "cxc".into(), "dxd".into(), "exe".into(), "fxf".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bbb".into(), "ccc".into(), "ddd".into(), "exe".into(), "fxf".into()],
            to: vec!["axa".into(), "bbb".into(), "cxc".into(), "dxd".into(), "exe".into(), "fxf".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bxb".into(), "ccc".into(), "ddd".into(), "exe".into(), "fxf".into()],
            to: vec!["aaa".into(), "bxb".into(), "cxc".into(), "dxd".into(), "exe".into(), "fxf".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bxb".into(), "ccc".into(), "ddd".into(), "exe".into(), "fxf".into()],
            to: vec!["axa".into(), "bxb".into(), "cxc".into(), "dxd".into(), "exe".into(), "fxf".into()],
            required_fuzz: 2,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bbb".into(), "czc".into(), "dzd".into(), "eee".into(), "fff".into()],
            to: vec!["aaa".into(), "bbb".into(), "czc".into(), "dzd".into(), "eee".into(), "fff".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bbb".into(), "czc".into(), "dzd".into(), "eee".into(), "fff".into()],
            to: vec!["axa".into(), "bbb".into(), "czc".into(), "dzd".into(), "eee".into(), "fff".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bxb".into(), "czc".into(), "dzd".into(), "eee".into(), "fff".into()],
            to: vec!["aaa".into(), "bxb".into(), "czc".into(), "dzd".into(), "eee".into(), "fff".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bxb".into(), "czc".into(), "dzd".into(), "eee".into(), "fff".into()],
            to: vec!["axa".into(), "bxb".into(), "czc".into(), "dzd".into(), "eee".into(), "fff".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bbb".into(), "czc".into(), "dzd".into(), "exe".into(), "fff".into()],
            to: vec!["aaa".into(), "bbb".into(), "czc".into(), "dzd".into(), "exe".into(), "fff".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bbb".into(), "czc".into(), "dzd".into(), "exe".into(), "fff".into()],
            to: vec!["axa".into(), "bbb".into(), "czc".into(), "dzd".into(), "exe".into(), "fff".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bxb".into(), "czc".into(), "dzd".into(), "exe".into(), "fff".into()],
            to: vec!["aaa".into(), "bxb".into(), "czc".into(), "dzd".into(), "exe".into(), "fff".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bxb".into(), "czc".into(), "dzd".into(), "exe".into(), "fff".into()],
            to: vec!["axa".into(), "bxb".into(), "czc".into(), "dzd".into(), "exe".into(), "fff".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bbb".into(), "czc".into(), "dzd".into(), "eee".into(), "fxf".into()],
            to: vec!["aaa".into(), "bbb".into(), "czc".into(), "dzd".into(), "eee".into(), "fxf".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bbb".into(), "czc".into(), "dzd".into(), "eee".into(), "fxf".into()],
            to: vec!["axa".into(), "bbb".into(), "czc".into(), "dzd".into(), "eee".into(), "fxf".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bxb".into(), "czc".into(), "dzd".into(), "eee".into(), "fxf".into()],
            to: vec!["aaa".into(), "bxb".into(), "czc".into(), "dzd".into(), "eee".into(), "fxf".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bxb".into(), "czc".into(), "dzd".into(), "eee".into(), "fxf".into()],
            to: vec!["axa".into(), "bxb".into(), "czc".into(), "dzd".into(), "eee".into(), "fxf".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bbb".into(), "czc".into(), "dzd".into(), "exe".into(), "fxf".into()],
            to: vec!["aaa".into(), "bbb".into(), "czc".into(), "dzd".into(), "exe".into(), "fxf".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bbb".into(), "czc".into(), "dzd".into(), "exe".into(), "fxf".into()],
            to: vec!["axa".into(), "bbb".into(), "czc".into(), "dzd".into(), "exe".into(), "fxf".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["aaa".into(), "bxb".into(), "czc".into(), "dzd".into(), "exe".into(), "fxf".into()],
            to: vec!["aaa".into(), "bxb".into(), "czc".into(), "dzd".into(), "exe".into(), "fxf".into()],
            required_fuzz: 3,
        },
        FuzzyApplyTestPair {
            from: vec!["axa".into(), "bxb".into(), "czc".into(), "dzd".into(), "exe".into(), "fxf".into()],
            to: vec!["axa".into(), "bxb".into(), "czc".into(), "dzd".into(), "exe".into(), "fxf".into()],
            required_fuzz: 3,
        },
    ];
}