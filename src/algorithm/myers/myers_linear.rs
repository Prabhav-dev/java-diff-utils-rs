//! Eugene Myers linear space diff algorithm with O(N) space complexity.

use crate::algorithm::{
    change::{Change, DeltaType},
    diff_algorithm_listener::DiffAlgorithmListener,
    DiffAlgorithm,
};

/// A Snake represents a diagonal run of identical elements between two sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Snake {
    start: usize,
    end: usize,
    diag: isize,
}

pub struct MyersDiffWithLinearSpace<T> {
    equalizer: Option<crate::algorithm::diff_algorithm::EqualizerFn<T>>,
}

impl<T> Default for MyersDiffWithLinearSpace<T> {
    fn default() -> Self {
        Self { equalizer: None }
    }
}

impl<T> MyersDiffWithLinearSpace<T> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_equalizer<F>(equalizer: F) -> Self
    where
        F: Fn(&T, &T) -> bool + 'static,
    {
        Self {
            equalizer: Some(Box::new(equalizer)),
        }
    }
}

impl<T: PartialEq> DiffAlgorithm<T> for MyersDiffWithLinearSpace<T> {
    fn diff_with_listener(
        &self,
        source: &[T],
        target: &[T],
        listener: &mut dyn DiffAlgorithmListener,
    ) -> Vec<Change> {
        if let Some(ref eq) = self.equalizer {
            compute_diff_with_listener(source, target, eq, listener)
        } else {
            compute_diff_with_listener(source, target, |a, b| a == b, listener)
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct LinearWorkspace {
    v_down: Vec<usize>,
    v_up: Vec<usize>,
}

impl LinearWorkspace {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn prepare_buffers(&mut self, required_len: usize) {
        if self.v_down.len() < required_len {
            self.v_down.resize(required_len, 0);
            self.v_up.resize(required_len, 0);
        } else {
            self.v_down.fill(0);
            self.v_up.fill(0);
        }
    }
}

pub fn compute_diff<T: PartialEq>(source: &[T], target: &[T]) -> Vec<Change> {
    compute_diff_with(source, target, |a, b| a == b)
}

pub fn compute_diff_with<T, F>(source: &[T], target: &[T], equalizer: F) -> Vec<Change>
where
    F: Fn(&T, &T) -> bool,
{
    let mut ws = LinearWorkspace::new();
    let _noop = ();
    compute_diff_full(source, target, equalizer, &mut ws, None::<&mut noop_listener::NoopListener>)
}

pub fn compute_diff_with_listener<T, F>(
    source: &[T],
    target: &[T],
    equalizer: F,
    listener: &mut dyn DiffAlgorithmListener,
) -> Vec<Change>
where
    F: Fn(&T, &T) -> bool,
{
    let mut ws = LinearWorkspace::new();
    compute_diff_full(source, target, equalizer, &mut ws, Some(listener))
}

mod noop_listener {
    use crate::algorithm::diff_algorithm_listener::DiffAlgorithmListener;
    pub struct NoopListener;
    impl DiffAlgorithmListener for NoopListener {}
}

pub fn compute_diff_full<T, F, L>(
    source: &[T],
    target: &[T],
    equalizer: F,
    workspace: &mut LinearWorkspace,
    mut listener: Option<&mut L>,
) -> Vec<Change>
where
    F: Fn(&T, &T) -> bool,
    L: DiffAlgorithmListener + ?Sized,
{
    if source.is_empty() && target.is_empty() {
        return Vec::new();
    }

    if let Some(l) = listener.as_deref_mut() {
        l.diff_start();
    }

    let buffer_size = source.len() + target.len() + 2;
    workspace.prepare_buffers(buffer_size);

    let mut script = Vec::new();
    let max_steps = source.len() + target.len();

    let mut ctx = LinearCtx {
        source,
        target,
        equalizer: &equalizer,
        ws: workspace,
        script: &mut script,
        listener: listener.as_deref_mut(),
        max_steps,
    };

    ctx.partition_and_build(SubRegion {
        src_start: 0,
        src_end: source.len(),
        tgt_start: 0,
        tgt_end: target.len(),
    });

    if let Some(l) = listener {
        l.diff_end();
    }

    script
}

/// Represents the active slicing window during recursion.
#[derive(Clone, Copy)]
struct SubRegion {
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

struct LinearCtx<'a, T, F, L: ?Sized> {
    source: &'a [T],
    target: &'a [T],
    equalizer: &'a F,
    ws: &'a mut LinearWorkspace,
    script: &'a mut Vec<Change>,
    listener: Option<&'a mut L>,
    max_steps: usize,
}

impl<'a, T, F, L> LinearCtx<'a, T, F, L>
where
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

    fn partition_and_build(&mut self, mut region: SubRegion) {
        let mut src_start = region.src_start;
        let mut src_end = region.src_end;
        let mut tgt_start = region.tgt_start;
        let mut tgt_end = region.tgt_end;

        while src_start < src_end
            && tgt_start < tgt_end
            && (self.equalizer)(&self.source[src_start], &self.target[tgt_start])
        {
            src_start += 1;
            tgt_start += 1;
        }

        while src_start < src_end
            && tgt_start < tgt_end
            && (self.equalizer)(&self.source[src_end - 1], &self.target[tgt_end - 1])
        {
            src_end -= 1;
            tgt_end -= 1;
        }

        region = SubRegion {
            src_start,
            src_end,
            tgt_start,
            tgt_end,
        };

        if src_start == src_end {
            if tgt_start < tgt_end {
                self.push_change(
                    DeltaType::Insert,
                    DeltaSpan {
                        src_start,
                        src_end,
                        tgt_start,
                        tgt_end,
                    },
                );
            }
            return;
        }

        if tgt_start == tgt_end {
            if src_start < src_end {
                self.push_change(
                    DeltaType::Delete,
                    DeltaSpan {
                        src_start,
                        src_end,
                        tgt_start,
                        tgt_end,
                    },
                );
            }
            return;
        }

        let snake = find_middle_snake(self.source, self.target, self.equalizer, region, self.ws);

        if let Some(ref mut l) = self.listener {
            let processed = (src_start + tgt_start) / 2;
            l.diff_step(processed, self.max_steps);
        }

        if let Some(sn) = snake {
            let sn_tgt_start = (sn.start as isize - sn.diag) as usize;
            let sn_tgt_end = (sn.end as isize - sn.diag) as usize;

            if sn.start > src_start || sn_tgt_start > tgt_start {
                self.partition_and_build(SubRegion {
                    src_start,
                    src_end: sn.start,
                    tgt_start,
                    tgt_end: sn_tgt_start,
                });
            }

            if src_end > sn.end || tgt_end > sn_tgt_end {
                self.partition_and_build(SubRegion {
                    src_start: sn.end,
                    src_end,
                    tgt_start: sn_tgt_end,
                    tgt_end,
                });
            }
        } else {
            self.push_change(
                DeltaType::Delete,
                DeltaSpan {
                    src_start,
                    src_end,
                    tgt_start,
                    tgt_end: tgt_start,
                },
            );
            self.push_change(
                DeltaType::Insert,
                DeltaSpan {
                    src_start: src_end,
                    src_end,
                    tgt_start,
                    tgt_end,
                },
            );
        }
    }
}

fn find_middle_snake<T, F>(
    source: &[T],
    target: &[T],
    equalizer: &F,
    region: SubRegion,
    ws: &mut LinearWorkspace,
) -> Option<Snake>
where
    F: Fn(&T, &T) -> bool,
{
    let src_len = region.src_end - region.src_start;
    let tgt_len = region.tgt_end - region.tgt_start;

    if src_len == 0 || tgt_len == 0 {
        return None;
    }

    let delta = src_len as isize - tgt_len as isize;
    let total_len = tgt_len + src_len;
    let offset = if total_len.is_multiple_of(2) {
        total_len
    } else {
        total_len + 1
    } / 2;

    ws.v_down[1 + offset] = region.src_start;
    ws.v_up[1 + offset] = region.src_end + 1;

    for d in 0..=offset {
        let d_step = d as isize;

        // --- Downward Search ---
        for k in (-d_step..=d_step).step_by(2) {
            let idx = (k + offset as isize) as usize;

            if k == -d_step || (k != d_step && ws.v_down[idx - 1] < ws.v_down[idx + 1]) {
                ws.v_down[idx] = ws.v_down[idx + 1];
            } else {
                ws.v_down[idx] = ws.v_down[idx - 1] + 1;
            }

            let mut x = ws.v_down[idx];
            let mut y =
                (x as isize - region.src_start as isize + region.tgt_start as isize - k) as usize;

            while x < region.src_end && y < region.tgt_end && equalizer(&source[x], &target[y]) {
                x += 1;
                ws.v_down[idx] = x;
                y += 1;
            }

            if delta % 2 != 0 && (delta - d_step) <= k && k <= (delta + d_step) {
                let up_idx = (idx as isize - delta) as usize;
                if ws.v_up.get(up_idx).is_some_and(|&v| v <= ws.v_down[idx]) {
                    return Some(expand_snake(
                        source,
                        target,
                        equalizer,
                        ws.v_up[up_idx],
                        k + region.src_start as isize - region.tgt_start as isize,
                        region.src_end,
                        region.tgt_end,
                    ));
                }
            }
        }

        // --- Upward Search ---
        let k_min = delta - d_step;
        let k_max = delta + d_step;
        for k in (k_min..=k_max).step_by(2) {
            let idx = (k + offset as isize - delta) as usize;

            if k == k_min || (k != k_max && ws.v_up[idx + 1] <= ws.v_up[idx - 1]) {
                ws.v_up[idx] = ws.v_up[idx + 1].saturating_sub(1);
            } else {
                ws.v_up[idx] = ws.v_up[idx - 1];
            }

            let mut x = ws.v_up[idx].saturating_sub(1);
            let mut y =
                (x as isize - region.src_start as isize + region.tgt_start as isize - k) as usize;

            while x >= region.src_start
                && y >= region.tgt_start
                && x < region.src_end
                && y < region.tgt_end
                && equalizer(&source[x], &target[y])
            {
                ws.v_up[idx] = x;
                if x == 0 || y == 0 {
                    break;
                }
                x -= 1;
                y -= 1;
            }

            if delta % 2 == 0 && -d_step <= k && k <= d_step {
                let down_idx = (idx as isize + delta) as usize;
                if ws.v_down.get(down_idx).is_some_and(|&v| ws.v_up[idx] <= v) {
                    return Some(expand_snake(
                        source,
                        target,
                        equalizer,
                        ws.v_up[idx],
                        k + region.src_start as isize - region.tgt_start as isize,
                        region.src_end,
                        region.tgt_end,
                    ));
                }
            }
        }
    }

    Some(Snake {
        start: region.src_start,
        end: region.src_end,
        diag: region.src_start as isize - region.tgt_start as isize,
    })
}

fn expand_snake<T, F>(
    source: &[T],
    target: &[T],
    equalizer: &F,
    start: usize,
    diag: isize,
    src_bound: usize,
    tgt_bound: usize,
) -> Snake
where
    F: Fn(&T, &T) -> bool,
{
    let mut end = start;
    while (end as isize - diag) < tgt_bound as isize
        && end < src_bound
        && equalizer(&source[end], &target[(end as isize - diag) as usize])
    {
        end += 1;
    }

    Snake { start, end, diag }
}
