//! Progress reporting types for tracking long-running diff computations.

pub trait DiffAlgorithmListener {
    fn diff_start(&mut self) {}

    fn diff_step(&mut self, value: usize, max: usize) {
        let _ = (value, max);
    }

    /// Called when exploring path nodes in algorithm execution.
    /// Delegates to `diff_step` by default for parity with Java-Diff-Utils listeners.
    fn path_node(&mut self, i: usize, j: usize,_k: usize) {
        self.diff_step(i, j);
    }

    fn diff_end(&mut self) {}
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoOpListener;

impl DiffAlgorithmListener for NoOpListener {}
impl DiffAlgorithmListener for () {}

impl<F> DiffAlgorithmListener for F
where
    F: FnMut(usize, usize),
{
    fn diff_step(&mut self, value: usize, max: usize) {
        self(value, max);
    }
}