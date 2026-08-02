//! Utility functions for merging inline deltas.

use crate::patch::chunk::Chunk;
use crate::patch::delta::Delta;
use crate::patch::delta_type::DeltaType;
use crate::text::delta_merge::inline_delta_merge_info::InlineDeltaMergeInfo;

/// Utility features for merging inline deltas.
pub struct DeltaMergeUtils;

impl DeltaMergeUtils {
    /// Merges adjacent or close inline deltas if the intervening unchanged lines
    /// satisfy the provided `replace_equality` predicate.
    pub fn merge_inline_deltas<F>(
        delta_merge_info: &InlineDeltaMergeInfo<String>,
        replace_equality: F,
    ) -> Vec<Delta<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        let original_deltas = delta_merge_info.deltas();
        if original_deltas.len() < 2 {
            return original_deltas.to_vec();
        }

        let mut new_deltas: Vec<Delta<String>> = Vec::new();
        new_deltas.push(original_deltas[0].clone());

        for current_delta in original_deltas.iter().skip(1) {
            let previous_delta = new_deltas.last().unwrap();

            let prev_source_pos = previous_delta.source().position();
            let prev_source_len = previous_delta.source().len();
            let start_idx = prev_source_pos + prev_source_len;
            let end_idx = current_delta.source().position();

            let equalities = if start_idx <= end_idx && end_idx <= delta_merge_info.orig_list().len() {
                &delta_merge_info.orig_list()[start_idx..end_idx]
            } else {
                &[]
            };

            if replace_equality(equalities) {
                // Merge previous delta, equalities, and current delta into a single Change delta
                let mut all_source_lines = Vec::new();
                all_source_lines.extend(previous_delta.source().lines().iter().cloned());
                all_source_lines.extend(equalities.iter().cloned());
                all_source_lines.extend(current_delta.source().lines().iter().cloned());

                let mut all_target_lines = Vec::new();
                all_target_lines.extend(previous_delta.target().lines().iter().cloned());
                all_target_lines.extend(equalities.iter().cloned());
                all_target_lines.extend(current_delta.target().lines().iter().cloned());

                let replacement_source = Chunk::new(
                    previous_delta.source().position(),
                    all_source_lines,
                    None,
                );
                let replacement_target = Chunk::new(
                    previous_delta.target().position(),
                    all_target_lines,
                    None,
                );

                let replacement = Delta::new(
                    DeltaType::Change,
                    replacement_source,
                    replacement_target,
                );

                new_deltas.pop();
                new_deltas.push(replacement);
            } else {
                new_deltas.push(current_delta.clone());
            }
        }

        new_deltas
    }
}