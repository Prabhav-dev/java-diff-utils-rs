use super::{
    change::Change,
    diff_algorithm_listener::{DiffAlgorithmListener, NoOpListener},
};

/// Type alias for an element equality predicate closure.
pub type EqualizerFn<T> = Box<dyn Fn(&T, &T) -> bool>;

pub trait DiffAlgorithm<T> {
    fn diff(&self, source: &[T], target: &[T]) -> Vec<Change> {
        let mut noop = NoOpListener;
        self.diff_with_listener(source, target, &mut noop)
    }

    fn diff_with_listener(
        &self,
        source: &[T],
        target: &[T],
        listener: &mut dyn DiffAlgorithmListener,
    ) -> Vec<Change>;
}

impl<T, F> DiffAlgorithm<T> for F
where
    F: Fn(&[T], &[T]) -> Vec<Change>,
{
    fn diff(&self, source: &[T], target: &[T]) -> Vec<Change> {
        self(source, target)
    }

    fn diff_with_listener(
        &self,
        source: &[T],
        target: &[T],
        _listener: &mut dyn DiffAlgorithmListener,
    ) -> Vec<Change> {
        self(source, target)
    }
}
