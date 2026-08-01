pub trait DiffAlgorithmListener {
    fn on_diff_start(&mut self) {}
    fn on_diff_step(&mut self, _progress: usize, _total: usize) {}
    fn on_diff_end(&mut self) {}
}