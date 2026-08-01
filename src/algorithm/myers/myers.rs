use crate::algorithm::change::Change;
use crate::algorithm::diff_algorithm_listener::DiffAlgorithmListener;
use crate::algorithm::DiffAlgorithm;
use crate::patch::delta_type::DeltaType;
use super::path_node::PathNode;

#[derive(Default)]
pub struct DiffWorkspace {
    arena: Vec<PathNode>,
    diagonal: Vec<Option<usize>>,
}

impl DiffWorkspace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.arena.clear();
        self.diagonal.fill(None);
    }
}

pub struct MyersDiff<T> {
    equalizer: Option<Box<dyn Fn(&T, &T) -> bool>>,
}

impl<T> Default for MyersDiff<T> {
    fn default() -> Self {
        Self { equalizer: None }
    }
}

impl<T> MyersDiff<T> {
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

impl<T: PartialEq> DiffAlgorithm<T> for MyersDiff<T> {
    fn diff_with_listener(
        &self,
        source: &[T],
        target: &[T],
        _listener: &mut dyn DiffAlgorithmListener,
    ) -> Vec<Change> {
        if let Some(ref eq) = self.equalizer {
            compute_diff_with(source, target, eq)
        } else {
            compute_diff(source, target)
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
    let mut ws = DiffWorkspace::new();
    compute_diff_with_workspace(source, target, equalizer, &mut ws)
}

pub fn compute_diff_with_workspace<T, F>(
    source: &[T],
    target: &[T],
    equalizer: F,
    ws: &mut DiffWorkspace,
) -> Vec<Change>
where
    F: Fn(&T, &T) -> bool,
{
    if source.is_empty() && target.is_empty() {
        return Vec::new();
    }

    ws.clear();

    if let Some(head_idx) = build_path(source, target, &equalizer, ws) {
        build_revision(&ws.arena, head_idx)
    } else {
        Vec::new()
    }
}

fn build_path<T, F>(
    orig: &[T],
    rev: &[T],
    equalizer: &F,
    ws: &mut DiffWorkspace,
) -> Option<usize>
where
    F: Fn(&T, &T) -> bool,
{
    let n = orig.len();
    let m = rev.len();
    let max = n + m + 1;
    let size = 1 + 2 * max;
    let middle = max;

    ws.arena.reserve(max * 2);
    if ws.diagonal.len() < size {
        ws.diagonal.resize(size, None);
    } else {
        ws.diagonal.fill(None);
    }

    ws.arena.push(PathNode {
        i: 0,
        j: -1,
        is_snake: true,
        is_bootstrap: true,
        prev: None,
    });
    ws.diagonal[middle + 1] = Some(0);

    for d in 0..max {
        let d_isize = d as isize;

        for k in (-d_isize..=d_isize).step_by(2) {
            let kmiddle = (middle as isize + k) as usize;
            let kplus = kmiddle + 1;
            let kminus = kmiddle - 1;

            let (i_start, prev_idx) = if k == -d_isize {
                let p = ws.diagonal[kplus].unwrap_or(0);
                (ws.arena[p].i, p)
            } else if k != d_isize {
                let pm = ws.diagonal[kminus];
                let pp = ws.diagonal[kplus];

                match (pm, pp) {
                    (Some(pm), Some(pp)) => {
                        if ws.arena[pm].i < ws.arena[pp].i {
                            (ws.arena[pp].i, pp)
                        } else {
                            (ws.arena[pm].i + 1, pm)
                        }
                    }
                    (None, Some(pp)) => (ws.arena[pp].i, pp),
                    (Some(pm), None) => (ws.arena[pm].i + 1, pm),
                    (None, None) => (0, 0),
                }
            } else {
                let p = ws.diagonal[kminus].unwrap_or(0);
                (ws.arena[p].i + 1, p)
            };

            let mut i = i_start;
            let mut j = i as isize - k;

            let node_idx = ws.arena.len();
            ws.arena.push(PathNode {
                i,
                j,
                is_snake: false,
                is_bootstrap: false,
                prev: Some(prev_idx),
            });

            while i < n && j >= 0 && (j as usize) < m && equalizer(&orig[i], &rev[j as usize]) {
                i += 1;
                j += 1;
            }

            let final_node_idx = if i != ws.arena[node_idx].i {
                let snake_idx = ws.arena.len();
                ws.arena.push(PathNode {
                    i,
                    j,
                    is_snake: true,
                    is_bootstrap: false,
                    prev: Some(node_idx),
                });
                snake_idx
            } else {
                node_idx
            };

            ws.diagonal[kmiddle] = Some(final_node_idx);

            if i >= n && j >= 0 && (j as usize) >= m {
                return Some(final_node_idx);
            }
        }
    }

    None
}

fn build_revision(arena: &[PathNode], head_idx: usize) -> Vec<Change> {
    let mut changes = Vec::new();
    let mut curr_idx = Some(head_idx);

    // Mirror: if (path.isSnake()) path = path.prev;
    if let Some(idx) = curr_idx {
        if arena[idx].is_snake {
            curr_idx = arena[idx].prev;
        }
    }

    loop {
        let idx = match curr_idx {
            Some(i) => i,
            None => break,
        };
        let node = &arena[idx];

        let prev_idx = match node.prev {
            Some(p) => p,
            None => break,
        };

        // Mirror: while (path != null && path.prev != null && path.prev.j >= 0)
        if arena[prev_idx].j < 0 {
            break;
        }

        let i = node.i;
        let j = node.j.max(0) as usize;

        // Mirror: path = path.prev;  (move exactly once, unconditionally)
        let path_idx = prev_idx;
        let path_node = &arena[path_idx];
        let ianchor = path_node.i;
        let janchor = path_node.j.max(0) as usize;

        let delta_type = match (ianchor == i, janchor == j) {
            (true, false) => DeltaType::Insert,
            (false, true) => DeltaType::Delete,
            _ => DeltaType::Change,
        };

        changes.push(Change {
            delta_type,
            start_original: ianchor,
            end_original: i,
            start_revised: janchor,
            end_revised: j,
        });

        // Mirror: if (path.isSnake()) path = path.prev;
        curr_idx = if arena[path_idx].is_snake {
            arena[path_idx].prev
        } else {
            Some(path_idx)
        };
    }

    changes.reverse();
    changes
}