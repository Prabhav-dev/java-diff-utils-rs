use super::{Change, DiffAlgorithmListener};

pub trait DiffAlgorithm<T> {
    fn compute_diff(&self, source: &[T], target: &[T]) -> Vec<Change>;

    fn compute_diff_with_listener(
        &self,
        source: &[T],
        target: &[T],
        _listener: Option<&mut dyn DiffAlgorithmListener>,
    ) -> Vec<Change> {
        self.compute_diff(source, target)
    }
}

impl<T, F> DiffAlgorithm<T> for F
where
    F: Fn(&[T], &[T]) -> Vec<Change>,
{
    fn compute_diff(&self, source: &[T], target: &[T]) -> Vec<Change> {
        self(source, target)
    }
}