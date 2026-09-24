pub mod myers_linear;
pub mod myers_quadratic;
pub mod path_node;

// Module alias for backward compatibility with `myers::myers` path
pub use myers_quadratic as myers;

// Re-export standard Myers
pub use myers_quadratic::*;

// Re-export linear Myers items
pub use myers_linear::MyersDiffWithLinearSpace;
pub use myers_linear::{
    compute_diff as compute_linear_diff, compute_diff_full,
    compute_diff_with as compute_linear_diff_with, LinearWorkspace,
};
pub use path_node::{PathFormatter, PathNode};
