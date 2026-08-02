// tests/myers_diff_test.rs

//! Transpiled Unit Tests for `com.github.difflib.algorithm.myers.MyersDiffTest`

use my_diff_crate::algorithm::{
    diff_algorithm::DiffAlgorithm, diff_algorithm_listener::DiffAlgorithmListener,
    myers::myers::MyersDiff,
};
use my_diff_crate::patch::Patch;

/// Listener implementation to record lifecycle callbacks into a log vector.
#[derive(Default)]
struct LoggingListener {
    logdata: Vec<String>,
}

impl DiffAlgorithmListener for LoggingListener {
    fn diff_start(&mut self) {
        self.logdata.push("start".to_string());
    }

    fn diff_step(&mut self, value: usize, max: usize) {
        self.logdata.push(format!("{} - {}", value, max));
    }

    fn diff_end(&mut self) {
        self.logdata.push("end".to_string());
    }
}

#[test]
fn test_diff_myers_example_1_forward() {
    let original = vec!["A", "B", "C", "A", "B", "B", "A"];
    let revised = vec!["C", "B", "A", "B", "A", "C"];

    let myers = MyersDiff::<&str>::default();
    let changes = myers.diff(&original, &revised);
    
    println!("--- RAW CHANGES ---");
    for c in &changes {
        println!("{:?}", c);
    }

    let patch = Patch::generate(&original, &revised, &changes, false);

    println!("--- GENERATED DELTAS ({}) ---", patch.deltas().len());
    for (idx, d) in patch.deltas().iter().enumerate() {
        println!("{}: {:?}", idx, d);
    }

    assert_eq!(patch.deltas().len(), 4);
}

#[test]
fn test_diff_myers_example_1_forward_with_listener() {
    let original = vec!["A", "B", "C", "A", "B", "B", "A"];
    let revised = vec!["C", "B", "A", "B", "A", "C"];

    println!("\n=================== START MYERS DEBUG TRACE ===================");
    println!("Original (N={}): {:?}", original.len(), original);
    println!("Revised  (M={}): {:?}", revised.len(), revised);
    println!("---------------------------------------------------------------");

    let mut listener = LoggingListener::default();
    let myers = MyersDiff::<&str>::default();

    let changes = myers.diff_with_listener(&original, &revised, &mut listener);

    println!("\n--- LISTENER TRACE LOG ({}) ---", listener.logdata.len());
    for (idx, log) in listener.logdata.iter().enumerate() {
        println!("[{:02}] {}", idx, log);
    }

    println!("\n--- GENERATED CHANGES ({}) ---", changes.len());
    for (idx, c) in changes.iter().enumerate() {
        println!(
            "[{:02}] {:<6} | Orig: {}..{} | Rev: {}..{}",
            idx,
            format!("{:?}", c.delta_type),
            c.start_original,
            c.end_original,
            c.start_revised,
            c.end_revised
        );
    }

    let patch = Patch::generate(&original, &revised, &changes, false);
    println!("\n--- GENERATED DELTAS ({}) ---", patch.deltas().len());
    for (idx, d) in patch.deltas().iter().enumerate() {
        println!("[{:02}] {:?}", idx, d);
    }
    println!("Patch String: {}", patch.to_string());
    println!("==================== END MYERS DEBUG TRACE ====================\n");

    // Detailed Assertions with Failure Messages
    assert_eq!(
        listener.logdata.len(),
        8,
        "Listener log count mismatch! Expected 8 events, got {}. Log contents:\n{:#?}",
        listener.logdata.len(),
        listener.logdata
    );

    assert_eq!(
        patch.deltas().len(),
        4,
        "Expected 4 Deltas in patch, got {}. Deltas dump:\n{:#?}",
        patch.deltas().len(),
        patch.deltas()
    );

    let expected_patch_str = "Patch{deltas=[[DeleteDelta, position: 0, lines: [A, B]], [InsertDelta, position: 3, lines: [B]], [DeleteDelta, position: 5, lines: [B]], [InsertDelta, position: 7, lines: [C]]]}";
    assert_eq!(
        patch.to_string(),
        expected_patch_str,
        "\nPatch string mismatch!\nGot:      {}\nExpected: {}",
        patch.to_string(),
        expected_patch_str
    );
}