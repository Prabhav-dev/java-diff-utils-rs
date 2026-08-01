//! Progress reporting types for tracking long-running diff computations.

pub trait DiffAlgorithmListener {
    fn diff_start(&mut self) {}
    fn diff_step(&mut self, value: usize, max: usize) {
        let _ = (value, max);
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