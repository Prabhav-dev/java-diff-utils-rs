//! Eugene Myers linear space diff algorithm with O(N) space complexity.
use crate::algorithm::{
    change::{Change, DeltaType},
    diff_algorithm_listener::DiffAlgorithmListener,
};

/// A Snake represents a diagonal run of identical elements between two sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Snake {
    start: usize,
    end: usize,
    diag: isize,
}

/// Pre-allocated workspace to avoid dynamic vector re-allocations during recursive divide-and-conquer steps.
#[derive(Default)]
pub struct LinearWorkspace {
    v_down: Vec<usize>,
    v_up: Vec<usize>,
}

impl LinearWorkspace {
    pub fn new() -> Self {
        Self::default()
    }

    fn prepare_buffers(&mut self, required_len: usize) {
        if self.v_down.len() < required_len {
            self.v_down.resize(required_len, 0);
            self.v_up.resize(required_len, 0);
        } else {
            self.v_down[..required_len].fill(0);
            self.v_up[..required_len].fill(0);
        }
    }
}

/// No-op listener used as a default when no progress updates are requested.
pub struct NoOpListener;
impl DiffAlgorithmListener for NoOpListener {}

pub fn compute_diff<T: PartialEq>(source: &[T], target: &[T]) -> Vec<Change> {
    compute_diff_with(source, target, |a, b| a == b)
}

pub fn compute_diff_with<T, F>(source: &[T], target: &[T], equalizer: F) -> Vec<Change>
where
    F: Fn(&T, &T) -> bool,
{
    let mut workspace = LinearWorkspace::new();
    compute_diff_full(
        source,
        target,
        equalizer,
        &mut workspace,
        Option::<&mut NoOpListener>::None,
    )
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
    L: DiffAlgorithmListener,
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

    partition_and_build(
        source,
        target,
        &equalizer,
        SubRegion {
            src_start: 0,
            src_end: source.len(),
            tgt_start: 0,
            tgt_end: target.len(),
        },
        workspace,
        &mut script,
        listener.as_deref_mut(),
        max_steps,
    );

    if let Some(l) = listener.as_deref_mut() {
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

fn push_change(
    script: &mut Vec<Change>,
    delta_type: DeltaType,
    src_start: usize,
    src_end: usize,
    tgt_start: usize,
    tgt_end: usize,
) {
    // Coalesce contiguous operations of the same delta type
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

fn partition_and_build<T, F, L>(
    source: &[T],
    target: &[T],
    equalizer: &F,
    region: SubRegion,
    ws: &mut LinearWorkspace,
    script: &mut Vec<Change>,
    mut listener: Option<&mut L>,
    max_steps: usize,
) where
    F: Fn(&T, &T) -> bool,
    L: DiffAlgorithmListener,
{
    if let Some(l) = listener.as_deref_mut() {
        let step = (region.src_end - region.src_start) / 2 + (region.tgt_end - region.tgt_start) / 2;
        l.diff_step(step, max_steps);
    }

    let middle_snake = find_middle_snake(source, target, equalizer, region, ws);

    let reached_terminal = match middle_snake {
        None => true,
        Some(s) => {
            let diag_offset = region.src_end as isize - region.tgt_end as isize;
            let start_offset = region.src_start as isize - region.tgt_start as isize;

            (s.start == region.src_end && s.diag == diag_offset)
                || (s.end == region.src_start && s.diag == start_offset)
        }
    };

    if reached_terminal {
        let mut i = region.src_start;
        let mut j = region.tgt_start;

        while i < region.src_end || j < region.tgt_end {
            if i < region.src_end && j < region.tgt_end && equalizer(&source[i], &target[j]) {
                i += 1;
                j += 1;
            } else if (region.src_end - i) > (region.tgt_end - j) {
                push_change(script, DeltaType::Delete, i, i + 1, j, j);
                i += 1;
            } else {
                push_change(script, DeltaType::Insert, i, i, j, j + 1);
                j += 1;
}
        }
    } else if let Some(snake) = middle_snake {
        let mid_tgt_1 = (snake.start as isize - snake.diag) as usize;
        let mid_tgt_2 = (snake.end as isize - snake.diag) as usize;

        // Left split branch
        partition_and_build(
            source,
            target,
            equalizer,
            SubRegion {
                src_start: region.src_start,
                src_end: snake.start,
                tgt_start: region.tgt_start,
                tgt_end: mid_tgt_1,
            },
            ws,
            script,
            listener.as_deref_mut(),
            max_steps,
        );

        // Right split branch
        partition_and_build(
            source,
            target,
            equalizer,
            SubRegion {
                src_start: snake.end,
                src_end: region.src_end,
                tgt_start: mid_tgt_2,
                tgt_end: region.tgt_end,
            },
            ws,
            script,
            listener.as_deref_mut(),
            max_steps,
        );
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
    let offset = if total_len % 2 == 0 { total_len } else { total_len + 1 } / 2;

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
            let mut y = (x as isize - region.src_start as isize + region.tgt_start as isize - k) as usize;

            while x < region.src_end && y < region.tgt_end && equalizer(&source[x], &target[y]) {
                x += 1;
                ws.v_down[idx] = x;
                y += 1;
            }

            if delta % 2 != 0 && (delta - d_step) <= k && k <= (delta + d_step) {
                let up_idx = (idx as isize - delta) as usize;
                if ws.v_up.get(up_idx).map_or(false, |&v| v <= ws.v_down[idx]) {
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
            let mut y = (x as isize - region.src_start as isize + region.tgt_start as isize - k) as usize;

            // FIX: Added upper bound checks `x < region.src_end` and `y < region.tgt_end`
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
                if ws.v_down.get(down_idx).map_or(false, |&v| ws.v_up[idx] <= v) {
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