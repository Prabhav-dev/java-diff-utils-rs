#![forbid(unsafe_code)]

pub mod algorithm;
pub mod diff_utils;
pub mod patch;
pub mod text;
pub mod unified_diff_utils;
pub mod unifieddiff;

// Clean re-exports for root library usage
pub use algorithm::change::Change;
pub use algorithm::diff_algorithm::DiffAlgorithm;
pub use algorithm::diff_algorithm_factory::{
    DiffAlgorithmFactory, HistogramDiffFactory, MyersDiffFactory, MyersLinearDiffFactory,
};
pub use algorithm::diff_algorithm_listener::{DiffAlgorithmListener, NoOpListener};
pub use algorithm::histogram::HistogramDiff;
pub use algorithm::myers::myers::MyersDiff;
pub use algorithm::myers::myers_linear::MyersDiffWithLinearSpace;

pub use diff_utils::DiffUtils;

pub use patch::chunk::Chunk;
pub use patch::delta::Delta;
pub use patch::delta_type::DeltaType;
pub use patch::error::PatchError;
pub use patch::patch::Patch;
pub use patch::verify_chunk::VerifyChunk;

// Re-exports for text generation & row diffing
pub use text::diff_row::{DiffRow, Tag};
pub use text::diff_row_generator::DiffRowGenerator;
pub use text::string_utils::StringUtils;

// Re-export UnifiedDiffUtils at crate root
pub use unified_diff_utils::UnifiedDiffUtils;
