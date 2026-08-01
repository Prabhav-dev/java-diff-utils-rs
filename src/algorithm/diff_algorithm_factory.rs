use super::DiffAlgorithm;

pub enum AlgorithmType {
    Myers,
    MyersLinear,
}

pub struct DiffAlgorithmFactory;

impl DiffAlgorithmFactory {
    pub fn create<T: PartialEq + 'static>(_algo_type: AlgorithmType) -> Box<dyn DiffAlgorithm<T>> {
        Box::new(super::myers::MyersDiff::new())
    }
}