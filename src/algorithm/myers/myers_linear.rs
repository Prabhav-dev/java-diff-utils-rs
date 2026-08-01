use crate::algorithm::change::Change;
use crate::algorithm::diff_algorithm_listener::DiffAlgorithmListener;
use crate::algorithm::DiffAlgorithm;

#[derive(Default)]
pub struct MyersDiffLinear<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> MyersDiffLinear<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: PartialEq> DiffAlgorithm<T> for MyersDiffLinear<T> {
    fn compute_diff(&self, _source: &[T], _target: &[T]) -> Vec<Change> {
        // TODO: Implement O(N) space linear Myers algorithm (Hirschberg refinement)
        Vec::new()
    }

    fn compute_diff_with_listener(
        &self,
        source: &[T],
        target: &[T],
        _listener: Option<&mut dyn DiffAlgorithmListener>,
    ) -> Vec<Change> {
        self.compute_diff(source, target)
    }
}