//! Histogram diff algorithm (Patience / low-occurrence anchor based diff).
//!
//! Ported to match the behavior of JGit / `java-diff-utils` `HistogramDiff`.
//! This algorithm selects elements with low occurrence counts as anchors to split sequences
//! recursively, falling back to Myers' algorithm when no low-occurrence anchors remain.

use crate::algorithm::{
    change::{Change, DeltaType},
    diff_algorithm_factory::DiffAlgorithmFactory,
    diff_algorithm_listener::DiffAlgorithmListener,
    myers::myers_linear::MyersDiffWithLinearSpace,
    DiffAlgorithm,
};

/// Default maximum occurrence count for an element to be considered as a pivot anchor.
pub const DEFAULT_MAX_CHAIN_LENGTH: usize = 64;

/// Histogram diff algorithm implementation.
pub struct HistogramDiff<T> {
    max_chain_length: usize,
    equalizer: Option<Box<dyn Fn(&T, &T) -> bool>>,
}

impl<T> Default for HistogramDiff<T> {
    fn default() -> Self {
        Self {
            max_chain_length: DEFAULT_MAX_CHAIN_LENGTH,
            equalizer: None,
        }
    }
}

impl<T> HistogramDiff<T> {
    /// Creates a new `HistogramDiff` with default max chain length (64).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom maximum chain length threshold.
    #[must_use]
    pub fn with_max_chain_length(mut self, max_chain_length: usize) -> Self {
        self.max_chain_length = max_chain_length;
        self
    }

    /// Sets a custom element equality predicate.
    #[must_use]
    pub fn with_equalizer<F>(mut self, equalizer: F) -> Self
    where
        F: Fn(&T, &T) -> bool + 'static,
    {
        self.equalizer = Some(Box::new(equalizer));
        self
    }
}

impl<T: PartialEq> DiffAlgorithm<T> for HistogramDiff<T> {
    fn diff_with_listener(
        &self,
        source: &[T],
        target: &[T],
        listener: &mut dyn DiffAlgorithmListener,
    ) -> Vec<Change> {
        let eq: &dyn Fn(&T, &T) -> bool = match &self.equalizer {
            Some(f) => f.as_ref(),
            None => &|a, b| a == b,
        };
        compute_diff_full(source, target, eq, self.max_chain_length, Some(listener))
    }
}

/// Computes the diff between two slices using HistogramDiff and default equality.
pub fn compute_diff<T: PartialEq>(source: &[T], target: &[T]) -> Vec<Change> {
    compute_diff_with(source, target, |a, b| a == b)
}

/// Computes the diff between two slices using HistogramDiff and a custom equalizer.
pub fn compute_diff_with<T, F>(source: &[T], target: &[T], equalizer: F) -> Vec<Change>
where
    T: PartialEq,
    F: Fn(&T, &T) -> bool,
{
    compute_diff_full(
        source,
        target,
        &equalizer,
        DEFAULT_MAX_CHAIN_LENGTH,
        Option::<&mut dyn DiffAlgorithmListener>::None,
    )
}

/// Full histogram diff entry point with workspace and listener support.
pub fn compute_diff_full<T, F, L>(
    source: &[T],
    target: &[T],
    equalizer: &F,
    max_chain_length: usize,
    mut listener: Option<&mut L>,
) -> Vec<Change>
where
    T: PartialEq,
    F: Fn(&T, &T) -> bool + ?Sized,
    L: DiffAlgorithmListener + ?Sized,
{
    if source.is_empty() && target.is_empty() {
        return Vec::new();
    }

    if let Some(l) = listener.as_deref_mut() {
        l.diff_start();
    }

    let mut script = Vec::new();
    let total_steps = source.len() + target.len();

    histogram_rec(
        source,
        target,
        0,
        source.len(),
        0,
        target.len(),
        equalizer,
        max_chain_length,
        &mut script,
        listener.as_deref_mut(),
        total_steps,
    );

    if let Some(l) = listener {
        l.diff_end();
    }

    script
}

#[derive(Clone, Copy)]
struct MatchAnchor {
    src_idx: usize,
    tgt_idx: usize,
    len: usize,
}

#[allow(clippy::too_many_arguments)]
fn histogram_rec<T, F, L>(
    source: &[T],
    target: &[T],
    mut src_start: usize,
    mut src_end: usize,
    mut tgt_start: usize,
    mut tgt_end: usize,
    equalizer: &F,
    max_chain_length: usize,
    script: &mut Vec<Change>,
    mut listener: Option<&mut L>,
    total_steps: usize,
) where
    T: PartialEq,
    F: Fn(&T, &T) -> bool + ?Sized,
    L: DiffAlgorithmListener + ?Sized,
{
    // Fast path: trim matching prefix
    while src_start < src_end
        && tgt_start < tgt_end
        && equalizer(&source[src_start], &target[tgt_start])
    {
        src_start += 1;
        tgt_start += 1;
    }

    // Fast path: trim matching suffix
    while src_end > src_start
        && tgt_end > tgt_start
        && equalizer(&source[src_end - 1], &target[tgt_end - 1])
    {
        src_end -= 1;
        tgt_end -= 1;
    }

    let src_len = src_end - src_start;
    let tgt_len = tgt_end - tgt_start;

    if src_len == 0 && tgt_len == 0 {
        return;
    }

    if let Some(l) = listener.as_deref_mut() {
        l.diff_step(src_start + tgt_start, total_steps);
    }

    // Base cases: purely insertion or deletion
    if src_len == 0 {
        push_change(
            script,
            DeltaType::Insert,
            src_start,
            src_start,
            tgt_start,
            tgt_end,
        );
        return;
    }
    if tgt_len == 0 {
        push_change(
            script,
            DeltaType::Delete,
            src_start,
            src_end,
            tgt_start,
            tgt_start,
        );
        return;
    }

    // Try finding the lowest-occurrence anchor in the target range
    if let Some(anchor) = find_best_anchor(
        source,
        target,
        src_start,
        src_end,
        tgt_start,
        tgt_end,
        equalizer,
        max_chain_length,
    ) {
        // Divide and conquer: Left subregion
        histogram_rec(
            source,
            target,
            src_start,
            anchor.src_idx,
            tgt_start,
            anchor.tgt_idx,
            equalizer,
            max_chain_length,
            script,
            listener.as_deref_mut(),
            total_steps,
        );

        // Right subregion (anchor region is skipped as it is equal)
        histogram_rec(
            source,
            target,
            anchor.src_idx + anchor.len,
            src_end,
            anchor.tgt_idx + anchor.len,
            tgt_end,
            equalizer,
            max_chain_length,
            script,
            listener,
            total_steps,
        );
    } else {
        // Fallback to linear Myers on this subregion
        let fallback_algo = MyersDiffWithLinearSpace::new();
        let sub_source = &source[src_start..src_end];
        let sub_target = &target[tgt_start..tgt_end];

        let sub_changes = fallback_algo.diff_with_listener(
            sub_source,
            sub_target,
            &mut crate::algorithm::diff_algorithm_listener::NoOpListener,
        );

        for c in sub_changes {
            push_change(
                script,
                c.delta_type,
                src_start + c.start_original,
                src_start + c.end_original,
                tgt_start + c.start_revised,
                tgt_start + c.end_revised,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn find_best_anchor<T, F>(
    source: &[T],
    target: &[T],
    src_start: usize,
    src_end: usize,
    tgt_start: usize,
    tgt_end: usize,
    equalizer: &F,
    max_chain_length: usize,
) -> Option<MatchAnchor>
where
    T: PartialEq,
    F: Fn(&T, &T) -> bool + ?Sized,
{
    let mut best_anchor: Option<MatchAnchor> = None;
    let mut lowest_occurrence = max_chain_length + 1;

    for i in src_start..src_end {
        let mut count = 0;
        let mut first_match_j = 0;

        for j in tgt_start..tgt_end {
            if equalizer(&source[i], &target[j]) {
                count += 1;
                if count == 1 {
                    first_match_j = j;
                }
                if count >= lowest_occurrence {
                    break;
                }
            }
        }

        if count > 0 && count < lowest_occurrence {
            lowest_occurrence = count;

            // Expand match length forward as far as possible
            let mut len = 1;
            while (i + len) < src_end
                && (first_match_j + len) < tgt_end
                && equalizer(&source[i + len], &target[first_match_j + len])
            {
                len += 1;
            }

            best_anchor = Some(MatchAnchor {
                src_idx: i,
                tgt_idx: first_match_j,
                len,
            });

            // If unique match found (occurrence == 1), this is the ideal anchor
            if count == 1 {
                break;
            }
        }
    }

    best_anchor
}

fn push_change(
    script: &mut Vec<Change>,
    delta_type: DeltaType,
    src_start: usize,
    src_end: usize,
    tgt_start: usize,
    tgt_end: usize,
) {
    if let Some(last) = script.last_mut() {
        if last.delta_type == delta_type {
            match delta_type {
                DeltaType::Delete if last.end_original == src_start => {
                    last.end_original = src_end;
                    return;
                }
                DeltaType::Insert if last.end_revised == tgt_start => {
                    last.end_revised = tgt_end;
                    return;
                }
                _ => {}
            }
        }
    }

    script.push(Change {
        delta_type,
        start_original: src_start,
        end_original: src_end,
        start_revised: tgt_start,
        end_revised: tgt_end,
    });
}

/// Factory for creating `HistogramDiff` algorithm instances.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HistogramDiffFactory {
    pub max_chain_length: usize,
}

impl HistogramDiffFactory {
    pub fn new() -> Self {
        Self {
            max_chain_length: DEFAULT_MAX_CHAIN_LENGTH,
        }
    }

    pub fn with_max_chain_length(max_chain_length: usize) -> Self {
        Self { max_chain_length }
    }
}

impl<T: PartialEq + 'static> DiffAlgorithmFactory<T> for HistogramDiffFactory {
    fn create(&self) -> Box<dyn DiffAlgorithm<T>>
    where
        T: PartialEq + 'static,
    {
        Box::new(HistogramDiff::new().with_max_chain_length(self.max_chain_length))
    }

    fn create_with_equalizer(
        &self,
        equalizer: Box<dyn Fn(&T, &T) -> bool + 'static>,
    ) -> Box<dyn DiffAlgorithm<T>> {
        Box::new(
            HistogramDiff::new()
                .with_max_chain_length(self.max_chain_length)
                .with_equalizer(move |a: &T, b: &T| equalizer(a, b)),
        )
    }
}
