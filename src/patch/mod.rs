// src/patch/mod.rs

pub mod change_delta;
pub mod chunk;
pub mod conflict_output;
pub mod delta;
pub mod delta_type;
pub mod delete_delta;
pub mod equal_delta;
pub mod error;
pub mod insert_delta;
pub mod patch_failed_exception;
pub mod patch;
pub mod verify_chunk;
pub mod conflict_formatter;

// Re-export Change from the algorithm module
pub use crate::algorithm::change::Change;

pub use change_delta::ChangeDelta;
pub use chunk::Chunk;
pub use conflict_output::ConflictOutput;
pub use delta::Delta;
pub use delta_type::DeltaType;
pub use delete_delta::DeleteDelta;
pub use equal_delta::EqualDelta;
pub use error::{DiffError, PatchError};
pub use insert_delta::InsertDelta;
pub use patch::Patch;
pub use verify_chunk::VerifyChunk;
