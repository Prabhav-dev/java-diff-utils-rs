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
    let patch = Patch::generate(&original, &revised, &changes, false);

    assert_eq!(patch.deltas().len(), 4);
    assert_eq!(
        patch.to_string(),
        "Patch{deltas=[[DeleteDelta, position: 0, lines: [A, B]], [InsertDelta, position: 3, lines: [B]], [DeleteDelta, position: 5, lines: [B]], [InsertDelta, position: 7, lines: [C]]]}"
    );
}

#[test]
fn test_diff_myers_example_1_forward_with_listener() {
    let original = vec!["A", "B", "C", "A", "B", "B", "A"];
    let revised = vec!["C", "B", "A", "B", "A", "C"];

    let mut listener = LoggingListener::default();
    let myers = MyersDiff::<&str>::default();

    let changes = myers.diff_with_listener(&original, &revised, &mut listener);
    let patch = Patch::generate(&original, &revised, &changes, false);

    assert_eq!(patch.deltas().len(), 4);
    assert_eq!(
        patch.to_string(),
        "Patch{deltas=[[DeleteDelta, position: 0, lines: [A, B]], [InsertDelta, position: 3, lines: [B]], [DeleteDelta, position: 5, lines: [B]], [InsertDelta, position: 7, lines: [C]]]}"
    );

    println!("{:?}", listener.logdata);
    assert_eq!(listener.logdata.len(), 8);
}