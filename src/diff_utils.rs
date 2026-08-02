//! High-level convenience functions for computing diffs, applying patches, and unpatching lists or text.

use std::sync::RwLock;

use crate::algorithm::myers::myers::MyersDiff;
use crate::algorithm::{DiffAlgorithm, DiffAlgorithmFactory, DiffAlgorithmListener};
use crate::patch::patch_failed_exception::PatchFailedException;
use crate::patch::Patch;

static DEFAULT_DIFF_FACTORY: RwLock<Option<Box<dyn DiffAlgorithmFactory<String> + Send + Sync>>> = RwLock::new(None);

pub struct DiffUtils;

impl DiffUtils {
    pub fn with_default_diff_algorithm_factory(factory: Box<dyn DiffAlgorithmFactory<String> + Send + Sync>) {
        if let Ok(mut guard) = DEFAULT_DIFF_FACTORY.write() {
            *guard = Some(factory);
        }
    }

    pub fn diff<T>(
        original: &[T],
        revised: &[T],
        progress: Option<&dyn DiffAlgorithmListener>,
    ) -> Patch<T>
    where
        T: PartialEq + Clone + 'static,
    {
        let algo = Self::get_default_algorithm::<T>();
        Self::diff_with_algorithm(original, revised, algo.as_ref(), progress, false)
    }

    pub fn diff_with_options<T>(
        original: &[T],
        revised: &[T],
        include_equal_parts: bool,
    ) -> Patch<T>
    where
        T: PartialEq + Clone + 'static,
    {
        let algo = Self::get_default_algorithm::<T>();
        Self::diff_with_algorithm(original, revised, algo.as_ref(), None, include_equal_parts)
    }

    pub fn diff_text(
        source_text: &str,
        target_text: &str,
        progress: Option<&dyn DiffAlgorithmListener>,
    ) -> Patch<String> {
        let original: Vec<String> = source_text.lines().map(|s| s.to_string()).collect();
        let revised: Vec<String> = target_text.lines().map(|s| s.to_string()).collect();
        Self::diff(&original, &revised, progress)
    }

    pub fn diff_with_equalizer<T, F>(
        source: &[T],
        target: &[T],
        equalizer: Option<F>,
    ) -> Patch<T>
    where
        T: PartialEq + Clone + 'static,
        F: Fn(&T, &T) -> bool + Send + Sync + 'static,
    {
        if let Some(eq) = equalizer {
            let algo = MyersDiff::with_equalizer(eq);
            Self::diff_with_algorithm(source, target, &algo, None, false)
        } else {
            let algo = MyersDiff::default();
            Self::diff_with_algorithm(source, target, &algo, None, false)
        }
    }

    pub fn diff_with_algorithm<T>(
        original: &[T],
        revised: &[T],
        algorithm: &dyn DiffAlgorithm<T>,
        _progress: Option<&dyn DiffAlgorithmListener>,
        include_equal_parts: bool,
    ) -> Patch<T>
    where
        T: Clone + 'static,
    {
        let deltas = algorithm.diff(original, revised);
        Patch::generate(original, revised, &deltas, include_equal_parts)
    }

    pub fn diff_inline(original: &str, revised: &str) -> Patch<String> {
        let orig_list: Vec<String> = original.chars().map(|c| c.to_string()).collect();
        let rev_list: Vec<String> = revised.chars().map(|c| c.to_string()).collect();

        let mut patch = Self::diff(&orig_list, &rev_list, None);

        for delta in patch.deltas_mut() {
            let source_lines = Self::compress_lines(delta.source_mut().lines(), "");
            delta.source_mut().set_lines(source_lines);

            let target_lines = Self::compress_lines(delta.target_mut().lines(), "");
            delta.target_mut().set_lines(target_lines);
        }

        patch
    }

    pub fn patch<T>(original: &[T], patch: &Patch<T>) -> Result<Vec<T>, PatchFailedException>
    where
        T: PartialEq + Clone,
    {
        patch.apply_to(original).map_err(PatchFailedException::from)
    }

    pub fn unpatch<T>(revised: &[T], patch: &Patch<T>) -> Result<Vec<T>, PatchFailedException>
    where
        T: PartialEq + Clone,
    {
        patch.restore(revised).map_err(PatchFailedException::from)
    }

    fn compress_lines(lines: &[String], delimiter: &str) -> Vec<String> {
        if lines.is_empty() {
            Vec::new()
        } else {
            vec![lines.join(delimiter)]
        }
    }

    fn get_default_algorithm<T: PartialEq + Clone + 'static>() -> Box<dyn DiffAlgorithm<T>> {
        Box::new(MyersDiff::default())
    }
}