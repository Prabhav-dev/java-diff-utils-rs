// tests/algorithm/myers_linear_space_patch_test.rs

//! Transpiled Unit Tests for `com.github.difflib.algorithm.myers.WithMyersDiffWithLinearSpacePatchTest`

use my_diff_crate::algorithm::myers::compute_diff as compute_diff_linear;
use my_diff_crate::patch::conflict_formatter::conflict_produces_merge_conflict;
use my_diff_crate::patch::{
    error::PatchFailedException,
    Patch,
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
    let original: Vec<String> = vec!["aaa", "bbb", "ccc", "ddd", "eee", "fff"]
        .into_iter()
        .map(String::from)
        .collect();
    let revised: Vec<String> = vec!["aaa", "bbb", "cxc", "dxd", "eee", "fff"]
        .into_iter()
        .map(String::from)
        .collect();

    let patch = Patch::generate(&original, &revised, &compute_diff_linear(&original, &revised), false);

    for (pair_idx, pair) in FUZZY_APPLY_TEST_PAIRS.iter().enumerate() {
        let prefix = int_range(6);
        let target = join(&[&prefix, &pair.from]);
        let expected = join(&[&prefix, &pair.to]);

        for max_fuzz in 0..=3 {
            let result = patch.apply_fuzzy(&target, max_fuzz);

            if max_fuzz < pair.required_fuzz {
                assert!(
                    result.is_err(),
                    "Pair #{}: Expected error for fuzz {} < required {}",
                    pair_idx,
                    max_fuzz,
                    pair.required_fuzz
                );
            } else {
                match result {
                    Ok(current) => {
                        if current != expected {
                            println!("\n=== FAILURE DIAGNOSTIC ===");
                            println!("Pair Index   : {}", pair_idx);
                            println!("Max Fuzz     : {}", max_fuzz);
                            println!("Required Fuzz: {}", pair.required_fuzz);
                            println!("Target Len   : {}", target.len());
                            println!("Target       : {:?}", target);

                            println!("\nDeltas in Patch:");
                            for (d_idx, delta) in patch.get_deltas().iter().enumerate() {
                                println!(
                                    "  Delta #{}: {:?} | Source Pos: {} | Source: {:?} | Target: {:?}",
                                    d_idx,
                                    delta.delta_type(),
                                    delta.source().position(),
                                    delta.source().lines(),
                                    delta.target().lines()
                                );
                            }

                            println!("\nLine-by-line diff (Actual vs Expected):");
                            let max_len = current.len().max(expected.len());
                            for i in 0..max_len {
                                let act = current.get(i).map(|s| s.as_str()).unwrap_or("<NONE>");
                                let exp = expected.get(i).map(|s| s.as_str()).unwrap_or("<NONE>");
                                let mark = if act == exp { " " } else { "!" };
                                println!("{:>2} {} | Actual: {:<10} | Expected: {:<10}", i, mark, act, exp);
                            }

                            panic!(
                                "Failed fuzzy apply for pair #{} with max_fuzz {}",
                                pair_idx, max_fuzz
                            );
                        }
                    }
                    Err(e) => {
                        panic!(
                            "Pair #{}: Unexpected failure with max_fuzz {}: {:?}",
                            pair_idx, max_fuzz, e
                        );
                    }
                }
            }
        }
    }
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
        .with_conflict_output(conflict_produces_merge_conflict);

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