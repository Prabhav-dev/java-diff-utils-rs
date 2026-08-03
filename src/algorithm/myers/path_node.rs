use std::fmt::{self, Display};

// Point in the Myers edit graph.
#[derive(Debug, Clone, Copy)]
pub struct PathNode {
    pub i: usize,
    pub j: isize, // -1 reserved for bootstrap node
    pub is_snake: bool,
    pub is_bootstrap: bool,
    pub prev: Option<usize>,
}

impl PathNode {
    pub fn new(i: usize, j: isize, is_snake: bool, is_bootstrap: bool, prev: Option<usize>) -> Self {
        Self { i, j, is_snake, is_bootstrap, prev }
    }

    pub fn fmt_path(arena: &[PathNode], start_idx: usize) -> String {
        format!("{}", PathFormatter { arena, start_idx })
    }

    /// Exact port of Java's `PathNode.previousSnake()`:
    ///   if (isBootstrap()) return null;
    ///   if (!isSnake() && prev != null) return prev.previousSnake();
    ///   return this;
    ///
    /// Called on a node (by index) the same way Java calls `somePrev.previousSnake()`.
    pub fn previous_snake(arena: &[PathNode], idx: usize) -> Option<usize> {
        let node = arena[idx];
        if node.is_bootstrap {
            return None;
        }
        if !node.is_snake {
            if let Some(p) = node.prev {
                return PathNode::previous_snake(arena, p);
            }
        }
        Some(idx)
    }
}

pub struct PathFormatter<'a> {
    arena: &'a [PathNode],
    start_idx: usize,
}

impl Display for PathFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut curr = Some(self.start_idx);
        let mut first = true;

        while let Some(idx) = curr {
            let Some(node) = self.arena.get(idx) else { break };
            
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "({},{})", node.i, node.j)?;
            first = false;

            if node.is_bootstrap {
                break;
            }
            curr = node.prev;
        }

        write!(f, "]")
    }
}