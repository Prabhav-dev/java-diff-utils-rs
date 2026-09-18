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
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

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

impl<T: Eq + Hash> DiffAlgorithm<T> for HistogramDiff<T> {
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
pub fn compute_diff<T: Eq + Hash>(source: &[T], target: &[T]) -> Vec<Change> {
    compute_diff_with(source, target, |a, b| a == b)
}

/// Computes the diff between two slices using HistogramDiff and a custom equalizer.
pub fn compute_diff_with<T, F>(source: &[T], target: &[T], equalizer: F) -> Vec<Change>
where
    T: Eq + Hash,
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
    T: Eq + Hash,
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

    normalize_replacements(&mut script);

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
    T: Eq + Hash,
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

    if src_len > 256
        && tgt_len > 256
        && ((looks_like_high_entropy(source, src_start, src_end)
            && looks_like_high_entropy(target, tgt_start, tgt_end))
            || (looks_like_low_entropy(source, src_start, src_end)
                && looks_like_low_entropy(target, tgt_start, tgt_end)))
    {
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
        return;
    }

    // Build both vector buckets and hash indexes in one pass per sequence.
    let (src_buckets, _) = build_buckets(source, src_start, src_end, equalizer);
    let (tgt_buckets, _) = build_buckets(target, tgt_start, tgt_end, equalizer);
    let src_distinct = src_buckets.len();
    let tgt_distinct = tgt_buckets.len();

    // Check whether any value exceeds max_chain_length (high-frequency check).
    // Re-use the already-built bucket lists rather than scanning again.
    let src_has_high_freq = src_buckets
        .iter()
        .any(|b| b.positions.len() > max_chain_length);
    let tgt_has_high_freq = tgt_buckets
        .iter()
        .any(|b| b.positions.len() > max_chain_length);

    // Histogram's recursive bucket rebuilds are wasteful when almost every
    // element is unique. Dispatch high-distinct, low-repetition regions to
    // Myers, which is substantially cheaper for this shape.
    let src_max_frequency = src_buckets
        .iter()
        .map(|bucket| bucket.positions.len())
        .max()
        .unwrap_or(0);
    let tgt_max_frequency = tgt_buckets
        .iter()
        .map(|bucket| bucket.positions.len())
        .max()
        .unwrap_or(0);
    let low_repetition = (src_distinct > 32 || tgt_distinct > 32)
        && src_max_frequency <= 2
        && tgt_max_frequency <= 2;

    if src_has_high_freq || tgt_has_high_freq || low_repetition {
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
        return;
    }
    // Drop the bucket lists — find_best_anchor will rebuild them internally.
    // (The alternative of passing them in would require threading through the
    // recursive signature; the build cost is O(n) per slice, which is acceptable.)
    drop(src_buckets);
    drop(tgt_buckets);

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

struct OccurrenceBucket<'a, T> {
    value: &'a T,
    positions: Vec<usize>,
}

/// Build occurrence buckets for `sequence[start..end]` in a single pass.
///
/// Returns the occurrence buckets and a hash index into them.
fn build_buckets<'a, T, F>(
    sequence: &'a [T],
    start: usize,
    end: usize,
    equalizer: &F,
) -> (Vec<OccurrenceBucket<'a, T>>, HashMap<&'a T, usize>)
where
    T: Eq + Hash,
    F: Fn(&T, &T) -> bool + ?Sized,
{
    let mut buckets: Vec<OccurrenceBucket<'a, T>> = Vec::new();
    let mut bucket_by_value: HashMap<&'a T, usize> = HashMap::new();

    for i in start..end {
        let value = &sequence[i];
        if let Some(&bucket_index) = bucket_by_value.get(value) {
            buckets[bucket_index].positions.push(i);
        } else {
            let bucket_index = buckets.len();
            buckets.push(OccurrenceBucket {
                value,
                positions: vec![i],
            });
            bucket_by_value.insert(value, bucket_index);
        }
    }

    let _ = equalizer;
    (buckets, bucket_by_value)
}

fn looks_like_high_entropy<T: Eq + Hash>(sequence: &[T], start: usize, end: usize) -> bool {
    let sample_end = (start + 64).min(end);
    let mut sample = HashSet::with_capacity(sample_end - start);
    for value in &sequence[start..sample_end] {
        if !sample.insert(value) {
            return false;
        }
    }
    true
}

fn looks_like_low_entropy<T: Eq + Hash>(sequence: &[T], start: usize, end: usize) -> bool {
    let sample_end = (start + 64).min(end);
    let mut sample = HashSet::with_capacity(sample_end - start);
    for value in &sequence[start..sample_end] {
        sample.insert(value);
    }
    sample.len() <= 32
}

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
    T: Eq + Hash,
    F: Fn(&T, &T) -> bool + ?Sized,
{
    // One pass each — bucket list doubles as the distinct-value index.
    let (mut source_buckets, _) = build_buckets(source, src_start, src_end, equalizer);
    let (target_buckets, target_by_value) = build_buckets(target, tgt_start, tgt_end, equalizer);

    // Sort source buckets by ascending count so unique elements (count == 1)
    // are visited first. This lets us hit the early-exit path as early as possible.
    source_buckets.sort_unstable_by_key(|b| b.positions.len());

    let mut best_anchor: Option<MatchAnchor> = None;
    let mut lowest_occurrence = max_chain_length + 1;
    let mut best_len = 0usize;

    for source_bucket in &source_buckets {
        let target_bucket_index = target_by_value
            .get(source_bucket.value)
            .copied()
            .or_else(|| {
                target_buckets
                    .iter()
                    .position(|bucket| equalizer(source_bucket.value, bucket.value))
            });
        let Some(target_bucket_index) = target_bucket_index else {
            continue;
        };
        let target_bucket = &target_buckets[target_bucket_index];

        let occurrence_count = source_bucket.positions.len().min(target_bucket.positions.len());
        // occurrence_count == 0 is impossible because build_buckets initialises
        // every entry with count = 1. Skip values that exceed the chain-length
        // threshold — they are too common to make reliable anchors.
        if occurrence_count > max_chain_length {
            continue;
        }

        for &src_idx in &source_bucket.positions {
            for &tgt_idx in &target_bucket.positions {
                if !equalizer(&source[src_idx], &target[tgt_idx]) {
                    continue;
                }
                let mut len = 1usize;
                while (src_idx + len) < src_end
                    && (tgt_idx + len) < tgt_end
                    && equalizer(&source[src_idx + len], &target[tgt_idx + len])
                {
                    len += 1;
                }

                if occurrence_count < lowest_occurrence
                    || (occurrence_count == lowest_occurrence && len > best_len)
                {
                    lowest_occurrence = occurrence_count;
                    best_len = len;
                    best_anchor = Some(MatchAnchor { src_idx, tgt_idx, len });

                    // Unique element found — this is the best possible anchor; stop early.
                    if occurrence_count == 1 {
                        return best_anchor;
                    }
                }
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

fn normalize_replacements(script: &mut Vec<Change>) {
    let mut normalized: Vec<Change> = Vec::with_capacity(script.len());

    for change in script.drain(..) {
        if let Some(previous) = normalized.last_mut() {
            let replacement = match (previous.delta_type, change.delta_type) {
                (DeltaType::Insert, DeltaType::Delete)
                    if previous.start_original == change.start_original
                        && previous.end_revised == change.start_revised => Some(Change {
                        delta_type: DeltaType::Change,
                        start_original: change.start_original,
                        end_original: change.end_original,
                        start_revised: previous.start_revised,
                        end_revised: change.end_revised,
                    }),
                (DeltaType::Delete, DeltaType::Insert)
                    if previous.end_original == change.start_original
                        && previous.end_revised == change.start_revised => Some(Change {
                        delta_type: DeltaType::Change,
                        start_original: previous.start_original,
                        end_original: previous.end_original,
                        start_revised: previous.start_revised,
                        end_revised: change.end_revised,
                    }),
                _ => None,
            };

            if let Some(replacement) = replacement {
                *previous = replacement;
                continue;
            }
        }

        normalized.push(change);
    }

    *script = normalized;
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

impl<T: Eq + Hash + 'static> DiffAlgorithmFactory<T> for HistogramDiffFactory {
    fn create(&self) -> Box<dyn DiffAlgorithm<T>>
    where
        T: Eq + Hash + 'static,
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
