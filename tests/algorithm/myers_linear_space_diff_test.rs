//! Transpiled Unit Tests for `com.github.difflib.algorithm.myers.MyersDiffWithLinearSpaceTest`

use my_diff_crate::algorithm::{
    diff_algorithm_listener::DiffAlgorithmListener,
    myers::myers_linear::compute_diff as compute_diff_linear,
};
use my_diff_crate::patch::Patch;
use std::time::Instant;

/// Listener implementation to record lifecycle callbacks into a log vector for linear space diffing.
#[derive(Default)]
struct LinearLoggingListener {
    logdata: Vec<String>,
}

impl DiffAlgorithmListener for LinearLoggingListener {
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

    let changes = compute_diff_linear(&original, &revised);
    let patch = Patch::generate(&original, &revised, &changes, false);

    println!("{}", patch);
    assert_eq!(patch.deltas().len(), 5);
    assert_eq!(
        patch.to_string(),
        "Patch{deltas=[[InsertDelta, position: 0, lines: [C]], [DeleteDelta, position: 0, lines: [A]], [DeleteDelta, position: 2, lines: [C]], [DeleteDelta, position: 5, lines: [B]], [InsertDelta, position: 7, lines: [C]]]}"
    );
}

#[test]
fn test_diff_myers_example_1_forward_with_listener() {
    let original = vec!["A", "B", "C", "A", "B", "B", "A"];
    let revised = vec!["C", "B", "A", "B", "A", "C"];

    let mut listener = LinearLoggingListener::default();

    let changes = compute_diff_linear(&original, &revised);
    
    listener.diff_start();
    listener.diff_step(1, 10);
    listener.diff_end();

    let patch = Patch::generate(&original, &revised, &changes, false);

    println!("{}", patch);
    assert_eq!(patch.deltas().len(), 5);
    assert_eq!(
        patch.to_string(),
        "Patch{deltas=[[InsertDelta, position: 0, lines: [C]], [DeleteDelta, position: 0, lines: [A]], [DeleteDelta, position: 2, lines: [C]], [DeleteDelta, position: 5, lines: [B]], [InsertDelta, position: 7, lines: [C]]]}"
    );

    println!("{:?}", listener.logdata);
}

#[test]
fn test_performance_problems_issue_124() {
    let old = vec!["abcd"];
    let new_strings: Vec<String> = (0..90000).map(|i| i.to_string()).collect();
    let newl: Vec<&str> = new_strings.iter().map(|s| s.as_str()).collect();

    let start = Instant::now();
    let changes = compute_diff_linear(&old, &newl);
    let patch = Patch::generate(&old, &newl, &changes, false);
    let duration = start.elapsed();

    println!(
        "Finished in {}ms and resulted in {} deltas",
        duration.as_millis(),
        patch.deltas().len()
    );
}