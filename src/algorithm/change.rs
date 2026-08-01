//! Delta representation of sequence modifications.

pub use crate::patch::delta_type::DeltaType;

/// Modified region between an original and target sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change {
    pub delta_type: DeltaType,
    pub start_original: usize,
    pub end_original: usize,
    pub start_revised: usize,
    pub end_revised: usize,
}

impl Change {
    pub fn new(
        delta_type: DeltaType,
        start_original: usize,
        end_original: usize,
        start_revised: usize,
        end_revised: usize,
    ) -> Self {
        debug_assert!(start_original <= end_original);
        debug_assert!(start_revised <= end_revised);

        Self {
            delta_type,
            start_original,
            end_original,
            start_revised,
            end_revised,
        }
    }

    pub fn original_len(&self) -> usize {
        self.end_original - self.start_original
    }

    pub fn revised_len(&self) -> usize {
        self.end_revised - self.start_revised
    }

    pub fn is_empty(&self) -> bool {
        self.start_original == self.end_original && self.start_revised == self.end_revised
    }
}