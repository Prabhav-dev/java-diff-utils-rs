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

    // Walk back up to find the parent snake node
    pub fn previous_snake(arena: &[PathNode], curr_idx: usize) -> Option<usize> {
        let node = arena.get(curr_idx)?;
        if node.is_bootstrap {
            return None;
        }

        let mut curr = node.prev;
        while let Some(idx) = curr {
            let n = arena.get(idx)?;
            if n.is_snake {
                return Some(idx);
            }
            if n.is_bootstrap {
                break;
            }
            curr = n.prev;
        }

        None
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