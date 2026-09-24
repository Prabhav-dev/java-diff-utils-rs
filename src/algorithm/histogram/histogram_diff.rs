//! Histogram diff algorithm (Patience / low-occurrence anchor based diff).
//!
//! Ported to match the behavior of `JGit` / `java-diff-utils` `HistogramDiff`.
//! This algorithm selects elements with low occurrence counts as anchors to split sequences
//! recursively, falling back to Myers' algorithm when no low-occurrence anchors remain.

use crate::algorithm::{
    change::{Change, DeltaType},
    diff_algorithm_factory::DiffAlgorithmFactory,
    diff_algorithm_listener::DiffAlgorithmListener,
    DiffAlgorithm,
};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Default maximum occurrence count for an element to be considered as a pivot anchor.
pub const DEFAULT_MAX_CHAIN_LENGTH: usize = 64;

/// Histogram diff algorithm implementation.
pub struct HistogramDiff<T> {
    max_chain_length: usize,
    equalizer: Option<crate::algorithm::diff_algorithm::EqualizerFn<T>>,
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_max_chain_length(mut self, max_chain_length: usize) -> Self {
        self.max_chain_length = max_chain_length;
        self
    }

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
        if let Some(ref eq) = self.equalizer {
            compute_diff_with_listener_and_max_chain(
                source,
                target,
                eq,
                self.max_chain_length,
                listener,
            )
        } else {
            compute_diff_with_listener_and_max_chain(
                source,
                target,
                |a, b| a == b,
                self.max_chain_length,
                listener,
            )
        }
    }
}

pub fn compute_diff<T: Eq + Hash>(source: &[T], target: &[T]) -> Vec<Change> {
    compute_diff_with(source, target, |a, b| a == b)
}

pub fn compute_diff_with<T, F>(source: &[T], target: &[T], equalizer: F) -> Vec<Change>
where
    T: Eq + Hash,
    F: Fn(&T, &T) -> bool,
{
    let _noop = ();
    compute_diff_full(
        source,
        target,
        equalizer,
        DEFAULT_MAX_CHAIN_LENGTH,
        None::<&mut noop_listener::NoopListener>,
    )
}

mod noop_listener {
    use crate::algorithm::diff_algorithm_listener::DiffAlgorithmListener;
    pub struct NoopListener;
    impl DiffAlgorithmListener for NoopListener {}
}

pub fn compute_diff_with_listener_and_max_chain<T, F>(
    source: &[T],
    target: &[T],
    equalizer: F,
    max_chain_length: usize,
    listener: &mut dyn DiffAlgorithmListener,
) -> Vec<Change>
where
    T: Eq + Hash,
    F: Fn(&T, &T) -> bool,
{
    compute_diff_full(
        source,
        target,
        equalizer,
        max_chain_length,
        Some(listener),
    )
}

pub fn compute_diff_full<T, F, L>(
    source: &[T],
    target: &[T],
    equalizer: F,
    max_chain_length: usize,
    mut listener: Option<&mut L>,
) -> Vec<Change>
where
    T: Eq + Hash,
    F: Fn(&T, &T) -> bool,
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

    let mut ctx = HistogramCtx {
        source,
        target,
        equalizer: &equalizer,
        max_chain_length,
        script: &mut script,
        listener: listener.as_deref_mut(),
        total_steps,
    };

    ctx.histogram_rec(HistoRegion {
        src_start: 0,
        src_end: source.len(),
        tgt_start: 0,
        tgt_end: target.len(),
    });

    if let Some(l) = listener {
        l.diff_end();
    }

    normalize_replacements(&mut script);
    script
}

#[derive(Clone, Copy)]
struct MatchAnchor {
    src_idx: usize,
    tgt_idx: usize,
    len: usize,
}

#[derive(Clone, Copy)]
struct HistoRegion {
    src_start: usize,
    src_end: usize,
    tgt_start: usize,
    tgt_end: usize,
}

#[derive(Clone, Copy)]
struct DeltaSpan {
    src_start: usize,
    src_end: usize,
    tgt_start: usize,
    tgt_end: usize,
}

struct HistogramCtx<'a, T, F, L: ?Sized> {
    source: &'a [T],
    target: &'a [T],
    equalizer: &'a F,
    max_chain_length: usize,
    script: &'a mut Vec<Change>,
    listener: Option<&'a mut L>,
    total_steps: usize,
}

impl<'a, T, F, L> HistogramCtx<'a, T, F, L>
where
    T: Eq + Hash,
    F: Fn(&T, &T) -> bool,
    L: DiffAlgorithmListener + ?Sized,
{
    fn push_change(&mut self, delta_type: DeltaType, span: DeltaSpan) {
        if let Some(last) = self.script.last_mut() {
            if last.delta_type == delta_type {
                match delta_type {
                    DeltaType::Delete if last.end_original == span.src_start => {
                        last.end_original = span.src_end;
                        return;
                    }
                    DeltaType::Insert if last.end_revised == span.tgt_start => {
                        last.end_revised = span.tgt_end;
                        return;
                    }
                    _ => {}
                }
            }
        }

        self.script.push(Change {
            delta_type,
            start_original: span.src_start,
            end_original: span.src_end,
            start_revised: span.tgt_start,
            end_revised: span.tgt_end,
        });
    }

    fn histogram_rec(&mut self, region: HistoRegion) {
        let mut src_start = region.src_start;
        let mut src_end = region.src_end;
        let mut tgt_start = region.tgt_start;
        let mut tgt_end = region.tgt_end;

        // Fast path: trim matching prefix
        while src_start < src_end
            && tgt_start < tgt_end
            && (self.equalizer)(&self.source[src_start], &self.target[tgt_start])
        {
            src_start += 1;
            tgt_start += 1;
        }

        // Fast path: trim matching suffix
        while src_end > src_start
            && tgt_end > tgt_start
            && (self.equalizer)(&self.source[src_end - 1], &self.target[tgt_end - 1])
        {
            src_end -= 1;
            tgt_end -= 1;
        }

        let src_len = src_end - src_start;
        let tgt_len = tgt_end - tgt_start;

        if src_len == 0 && tgt_len == 0 {
            return;
        }

        if let Some(l) = self.listener.as_deref_mut() {
            l.diff_step(src_start + tgt_start, self.total_steps);
        }

        // Base cases: purely insertion or deletion
        if src_len == 0 {
            self.push_change(
                DeltaType::Insert,
                DeltaSpan {
                    src_start,
                    src_end: src_start,
                    tgt_start,
                    tgt_end,
                },
            );
            return;
        }

        if tgt_len == 0 {
            self.push_change(
                DeltaType::Delete,
                DeltaSpan {
                    src_start,
                    src_end,
                    tgt_start,
                    tgt_end: tgt_start,
                },
            );
            return;
        }

        let curr_region = HistoRegion {
            src_start,
            src_end,
            tgt_start,
            tgt_end,
        };

        let anchor = find_best_anchor(
            self.source,
            self.target,
            curr_region,
            self.equalizer,
            self.max_chain_length,
        );

        if let Some(a) = anchor {
            // Recurse left
            self.histogram_rec(HistoRegion {
                src_start,
                src_end: a.src_idx,
                tgt_start,
                tgt_end: a.tgt_idx,
            });

            // Recurse right
            self.histogram_rec(HistoRegion {
                src_start: a.src_idx + a.len,
                src_end,
                tgt_start: a.tgt_idx + a.len,
                tgt_end,
            });
        } else {
            // Fallback to linear Myers on non-splittable region
            let myers_changes = crate::algorithm::myers::myers_linear::compute_diff_with(
                &self.source[src_start..src_end],
                &self.target[tgt_start..tgt_end],
                self.equalizer,
            );

            for c in myers_changes {
                self.push_change(
                    c.delta_type,
                    DeltaSpan {
                        src_start: src_start + c.start_original,
                        src_end: src_start + c.end_original,
                        tgt_start: tgt_start + c.start_revised,
                        tgt_end: tgt_start + c.end_revised,
                    },
                );
            }
        }
    }
}

struct OccurrenceBucket {
    positions: Vec<usize>,
}

type BucketsResult<'a, T> = (Vec<OccurrenceBucket>, HashMap<&'a T, usize>);

/// Build occurrence buckets for `sequence[start..end]` in a single pass.
fn build_buckets<'a, T, F>(
    sequence: &'a [T],
    start: usize,
    end: usize,
    _equalizer: &F,
) -> BucketsResult<'a, T>
where
    T: Eq + Hash,
    F: Fn(&T, &T) -> bool + ?Sized,
{
    let mut buckets: Vec<OccurrenceBucket> = Vec::new();
    let mut bucket_by_value: HashMap<&'a T, usize> = HashMap::new();

    for (offset, value) in sequence[start..end].iter().enumerate() {
        let i = start + offset;
        if let Some(&bucket_index) = bucket_by_value.get(value) {
            buckets[bucket_index].positions.push(i);
        } else {
            let bucket_index = buckets.len();
            buckets.push(OccurrenceBucket {
                positions: vec![i],
            });
            bucket_by_value.insert(value, bucket_index);
        }
    }

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
    region: HistoRegion,
    equalizer: &F,
    max_chain_length: usize,
) -> Option<MatchAnchor>
where
    T: Eq + Hash,
    F: Fn(&T, &T) -> bool + ?Sized,
{
    let src_start = region.src_start;
    let src_end = region.src_end;
    let tgt_start = region.tgt_start;
    let tgt_end = region.tgt_end;

    let src_len = src_end - src_start;
    let tgt_len = tgt_end - tgt_start;

    if src_len > 1024
        && tgt_len > 1024
        && (looks_like_high_entropy(source, src_start, src_end)
            || looks_like_high_entropy(target, tgt_start, tgt_end))
    {
        return None;
    }

    if src_len > 256
        && tgt_len > 256
        && (looks_like_low_entropy(source, src_start, src_end)
            || looks_like_low_entropy(target, tgt_start, tgt_end))
    {
        return None;
    }

    let (src_buckets, src_index) = build_buckets(source, src_start, src_end, equalizer);

    let mut best_anchor: Option<MatchAnchor> = None;
    let mut min_occurrence_count = usize::MAX;

    for (tgt_offset, tgt_val) in target[tgt_start..tgt_end].iter().enumerate() {
        let tgt_idx = tgt_start + tgt_offset;
        if let Some(&bucket_idx) = src_index.get(tgt_val) {
            let bucket = &src_buckets[bucket_idx];
            let count = bucket.positions.len();

            if count > max_chain_length {
                continue;
            }

            for &src_idx in &bucket.positions {
                if count < min_occurrence_count {
                    let mut match_len = 1;
                    while src_idx + match_len < src_end
                        && tgt_idx + match_len < tgt_end
                        && equalizer(&source[src_idx + match_len], &target[tgt_idx + match_len])
                    {
                        match_len += 1;
                    }

                    min_occurrence_count = count;
                    best_anchor = Some(MatchAnchor {
                        src_idx,
                        tgt_idx,
                        len: match_len,
                    });
                }
            }
        }
    }

    best_anchor
}

fn normalize_replacements(script: &mut Vec<Change>) {
    if script.len() < 2 {
        return;
    }

    let mut normalized = Vec::with_capacity(script.len());
    let mut i = 0;
    while i < script.len() {
        if i + 1 < script.len() {
            let first = &script[i];
            let second = &script[i + 1];

            if first.delta_type == DeltaType::Delete
                && second.delta_type == DeltaType::Insert
                && first.end_original == second.start_original
                && first.start_revised == second.start_revised
            {
                normalized.push(Change {
                    delta_type: DeltaType::Change,
                    start_original: first.start_original,
                    end_original: first.end_original,
                    start_revised: second.start_revised,
                    end_revised: second.end_revised,
                });
                i += 2;
                continue;
            } else if first.delta_type == DeltaType::Insert
                && second.delta_type == DeltaType::Delete
                && first.start_original == second.start_original
                && first.end_revised == second.start_revised
            {
                normalized.push(Change {
                    delta_type: DeltaType::Change,
                    start_original: second.start_original,
                    end_original: second.end_original,
                    start_revised: first.start_revised,
                    end_revised: first.end_revised,
                });
                i += 2;
                continue;
            }
        }

        normalized.push(script[i]);
        i += 1;
    }

    *script = normalized;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HistogramDiffFactory;

impl HistogramDiffFactory {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn with_max_chain_length(max_chain_length: usize) -> Self {
        let _ = max_chain_length;
        Self
    }
}

impl<T: Eq + Hash + 'static> DiffAlgorithmFactory<T> for HistogramDiffFactory {
    fn create(&self) -> Box<dyn DiffAlgorithm<T>>
    where
        T: PartialEq + 'static,
    {
        Box::new(HistogramDiff::new())
    }

    fn create_with_equalizer(
        &self,
        equalizer: crate::algorithm::diff_algorithm_factory::BoxedEqualizer<T>,
    ) -> Box<dyn DiffAlgorithm<T>> {
        Box::new(HistogramDiff::new().with_equalizer(move |a: &T, b: &T| equalizer(a, b)))
    }
}
