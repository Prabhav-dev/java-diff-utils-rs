#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathNode {
    pub i: usize,
    pub j: isize,
    pub is_snake: bool,
    pub is_bootstrap: bool,
    pub prev: Option<usize>,
}