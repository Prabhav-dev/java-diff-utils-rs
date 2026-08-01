//! Factory interface and implementations for constructing diff algorithms.

use super::{Change, DiffAlgorithm};

pub trait DiffAlgorithmFactory<T> {
    fn create(&self) -> Box<dyn DiffAlgorithm<T>>
    where
        T: PartialEq + 'static;

    fn create_with_equalizer(
        &self,
        equalizer: Box<dyn Fn(&T, &T) -> bool + 'static>,
    ) -> Box<dyn DiffAlgorithm<T>>
    where
        T: 'static;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MyersDiffFactory;

impl<T: 'static> DiffAlgorithmFactory<T> for MyersDiffFactory {
    fn create(&self) -> Box<dyn DiffAlgorithm<T>>
    where
        T: PartialEq + 'static,
    {
        Box::new(|source: &[T], target: &[T]| -> Vec<Change> {
            super::myers::compute_diff(source, target)
        })
    }

    fn create_with_equalizer(
        &self,
        equalizer: Box<dyn Fn(&T, &T) -> bool + 'static>,
    ) -> Box<dyn DiffAlgorithm<T>> {
        Box::new(move |source: &[T], target: &[T]| -> Vec<Change> {
            super::myers::compute_diff_with(source, target, &*equalizer)
        })
    }
}