//! Holds the information required to merge deltas originating from an inline diff.

use crate::patch::delta::Delta;

/// Holds the information required to merge deltas originating from an inline diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineDeltaMergeInfo<T = String> {
    deltas: Vec<Delta<T>>,
    orig_list: Vec<T>,
    rev_list: Vec<T>,
}

impl<T> InlineDeltaMergeInfo<T> {
    /// Constructs a new `InlineDeltaMergeInfo` instance.
    pub fn new(deltas: Vec<Delta<T>>, orig_list: Vec<T>, rev_list: Vec<T>) -> Self {
        Self {
            deltas,
            orig_list,
            rev_list,
        }
    }

    /// Returns a slice of the deltas.
    pub fn deltas(&self) -> &[Delta<T>] {
        &self.deltas
    }

    /// Returns a slice of the original text elements.
    pub fn orig_list(&self) -> &[T] {
        &self.orig_list
    }

    /// Returns a slice of the revised text elements.
    pub fn rev_list(&self) -> &[T] {
        &self.rev_list
    }

    /// Consumes self and returns the inner tuple of `(deltas, orig_list, rev_list)`.
    pub fn into_parts(self) -> (Vec<Delta<T>>, Vec<T>, Vec<T>) {
        (self.deltas, self.orig_list, self.rev_list)
    }
}