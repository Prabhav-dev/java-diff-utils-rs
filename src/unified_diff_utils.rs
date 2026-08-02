//! Utilities for generating and parsing single-file Unified Diffs.

use std::collections::HashMap;
use regex::Regex;

use crate::diff_utils::DiffUtils;
use crate::patch::change_delta::ChangeDelta;
use crate::patch::chunk::Chunk;
use crate::patch::delta::Delta;
use crate::patch::Patch;

const NULL_FILE_INDICATOR: &str = "/dev/null";

/// Utility methods for unified diff parsing and formatting.
pub struct UnifiedDiffUtils;

impl UnifiedDiffUtils {
    /// Parses a sequence of unified diff lines and returns a `Patch<String>`.
    pub fn parse_unified_diff(diff: &[String]) -> Patch<String> {
        let chunk_regex =
            Regex::new(r"^@@\s+-(\d+)(?:,(\d+))?\s+\+(\d+)(?:,(\d+))?\s+@@.*$").unwrap();

        let mut in_prelude = true;
        let mut raw_chunk: Vec<(String, String)> = Vec::new();
        let mut patch = Patch::new();

        let mut old_ln: usize = 0;
        let mut new_ln: usize = 0;

        for line in diff {
            if in_prelude {
                if line.starts_with("+++") {
                    in_prelude = false;
                }
                continue;
            }

            if let Some(caps) = chunk_regex.captures(line) {
                Self::process_lines_in_prev_chunk(&mut raw_chunk, &mut patch, old_ln, new_ln);

                old_ln = caps
                    .get(1)
                    .map_or(1, |m| m.as_str().parse::<usize>().unwrap_or(1));
                new_ln = caps
                    .get(3)
                    .map_or(1, |m| m.as_str().parse::<usize>().unwrap_or(1));

                if old_ln == 0 {
                    old_ln = 1;
                }
                if new_ln == 0 {
                    new_ln = 1;
                }
            } else if !line.is_empty() {
                let tag = &line[0..1];
                let rest = &line[1..];
                if tag == " " || tag == "+" || tag == "-" {
                    raw_chunk.push((tag.to_string(), rest.to_string()));
                }
            } else {
                raw_chunk.push((" ".to_string(), String::new()));
            }
        }

        Self::process_lines_in_prev_chunk(&mut raw_chunk, &mut patch, old_ln, new_ln);

        patch
    }

    fn process_lines_in_prev_chunk(
        raw_chunk: &mut Vec<(String, String)>,
        patch: &mut Patch<String>,
        old_ln: usize,
        new_ln: usize,
    ) {
        if raw_chunk.is_empty() {
            return;
        }

        let mut old_chunk_lines = Vec::new();
        let mut new_chunk_lines = Vec::new();
        let mut remove_position = Vec::new();
        let mut add_position = Vec::new();

        let mut remove_num = 0usize;
        let mut add_num = 0usize;

        for (tag, rest) in raw_chunk.iter() {
            if tag == " " || tag == "-" {
                remove_num += 1;
                old_chunk_lines.push(rest.clone());
                if tag == "-" {
                    remove_position.push((old_ln - 1) + remove_num);
                }
            }
            if tag == " " || tag == "+" {
                add_num += 1;
                new_chunk_lines.push(rest.clone());
                if tag == "+" {
                    add_position.push((new_ln - 1) + add_num);
                }
            }
        }

        let source_chunk = Chunk::new(
            old_ln.saturating_sub(1),
            old_chunk_lines,
            Some(remove_position),
        );
        let target_chunk = Chunk::new(
            new_ln.saturating_sub(1),
            new_chunk_lines,
            Some(add_position),
        );

        let delta = ChangeDelta::new(source_chunk, target_chunk);
        patch.add_delta(delta);
        raw_chunk.clear();
    }

    /// Generates unified diff output string lines from a given `Patch`.
    pub fn generate_unified_diff(
        original_file_name: Option<&str>,
        revised_file_name: Option<&str>,
        original_lines: &[String],
        patch: &Patch<String>,
        context_size: usize,
    ) -> Vec<String> {
        let patch_deltas = patch.deltas();
        if patch_deltas.is_empty() {
            return Vec::new();
        }

        let mut ret = Vec::new();
        ret.push(format!(
            "--- {}",
            original_file_name.unwrap_or(NULL_FILE_INDICATOR)
        ));
        ret.push(format!(
            "+++ {}",
            revised_file_name.unwrap_or(NULL_FILE_INDICATOR)
        ));

        let mut deltas: Vec<&Delta<String>> = Vec::new();
        let mut delta = patch_deltas[0].as_ref();
        deltas.push(delta);

        if patch_deltas.len() > 1 {
            for next_delta_box in patch_deltas.iter().skip(1) {
                let position = delta.source().position();
                let next_delta = next_delta_box.as_ref();

                if (position + delta.source().size() + context_size)
                    >= next_delta.source().position().saturating_sub(context_size)
                {
                    deltas.push(next_delta);
                } else {
                    let cur_block =
                        Self::process_deltas(original_lines, &deltas, context_size, false);
                    ret.extend(cur_block);
                    deltas.clear();
                    deltas.push(next_delta);
                }
                delta = next_delta;
            }
        }

        let is_new_file = patch_deltas.len() == 1 && original_file_name.is_none();
        let cur_block =
            Self::process_deltas(original_lines, &deltas, context_size, is_new_file);
        ret.extend(cur_block);

        ret
    }

    fn process_deltas(
        orig_lines: &[String],
        deltas: &[&Delta<String>],
        context_size: usize,
        new_file: bool,
    ) -> Vec<String> {
        let mut buffer = Vec::new();
        let mut orig_total = 0usize;
        let mut rev_total = 0usize;

        let cur_delta = deltas[0];

        let orig_start = if new_file {
            0
        } else {
            let pos_plus_one = cur_delta.source().position() + 1;
            if pos_plus_one > context_size {
                pos_plus_one - context_size
            } else {
                1
            }
        };

        let rev_pos_plus_one = cur_delta.target().position() + 1;
        let rev_start = if rev_pos_plus_one > context_size {
            rev_pos_plus_one - context_size
        } else {
            1
        };

        let context_start = cur_delta.source().position().saturating_sub(context_size);

        for line in context_start..cur_delta.source().position().min(orig_lines.len()) {
            buffer.push(format!(" {}", orig_lines[line]));
            orig_total += 1;
            rev_total += 1;
        }

        buffer.extend(Self::get_delta_text(cur_delta));
        orig_total += cur_delta.source().lines().len();
        rev_total += cur_delta.target().lines().len();

        let mut last_delta = cur_delta;
        for &next_delta in deltas.iter().skip(1) {
            let intermediate_start =
                last_delta.source().position() + last_delta.source().lines().len();

            for line in intermediate_start..next_delta.source().position().min(orig_lines.len()) {
                buffer.push(format!(" {}", orig_lines[line]));
                orig_total += 1;
                rev_total += 1;
            }

            buffer.extend(Self::get_delta_text(next_delta));
            orig_total += next_delta.source().lines().len();
            rev_total += next_delta.target().lines().len();
            last_delta = next_delta;
        }

        let post_context_start =
            last_delta.source().position() + last_delta.source().lines().len();
        let post_context_end = (post_context_start + context_size).min(orig_lines.len());

        for line in post_context_start..post_context_end {
            buffer.push(format!(" {}", orig_lines[line]));
            orig_total += 1;
            rev_total += 1;
        }

        let header = format!(
            "@@ -{},{} +{},{} @@",
            orig_start, orig_total, rev_start, rev_total
        );
        buffer.insert(0, header);

        buffer
    }

    fn get_delta_text(delta: &Delta<String>) -> Vec<String> {
        let mut buffer = Vec::new();
        for line in delta.source().lines() {
            buffer.push(format!("-{}", line));
        }
        for line in delta.target().lines() {
            buffer.push(format!("+{}", line));
        }
        buffer
    }

    /// Merges diff indicators into original file text (useful for visual diff applications).
    pub fn generate_original_and_diff(
        original: &[String],
        revised: &[String],
        original_file_name: Option<&str>,
        revised_file_name: Option<&str>,
    ) -> Vec<String> {
        let orig_name = original_file_name.unwrap_or("original");
        let rev_name = revised_file_name.unwrap_or("revised");

        let patch = DiffUtils::diff(original, revised, None);
        let mut unified_diff = Self::generate_unified_diff(
            Some(orig_name),
            Some(rev_name),
            original,
            &patch,
            0,
        );

        if unified_diff.is_empty() {
            unified_diff.push(format!("--- {}", orig_name));
            unified_diff.push(format!("+++ {}", rev_name));
            unified_diff.push("@@ -0,0 +0,0 @@".to_string());
        } else if unified_diff.len() >= 3 && !unified_diff[2].contains("@@ -1,") {
            unified_diff.insert(2, "@@ -0,0 +0,0 @@".to_string());
        }

        let original_with_prefix: Vec<String> =
            original.iter().map(|v| format!(" {}", v)).collect();
        Self::insert_orig(&original_with_prefix, &unified_diff)
    }

    fn insert_orig(original: &[String], unified_diff: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        let mut diff_list: Vec<Vec<String>> = Vec::new();
        let mut diff = Vec::new();

        for (i, u) in unified_diff.iter().enumerate() {
            if u.starts_with("@@") && u != "@@ -0,0 +0,0 @@" && !u.contains("@@ -1,") {
                diff_list.push(diff.clone());
                diff.clear();
                diff.push(u.clone());
                continue;
            }
            if i == unified_diff.len() - 1 {
                diff.push(u.clone());
                diff_list.push(diff.clone());
                diff.clear();
                break;
            }
            diff.push(u.clone());
        }

        Self::insert_orig_chunks(&diff_list, &mut result, original);
        result
    }

    fn insert_orig_chunks(
        diff_list: &[Vec<String>],
        result: &mut Vec<String>,
        original: &[String],
    ) {
        for (i, diff) in diff_list.iter().enumerate() {
            let nex_diff = diff_list.get(i + 1);
            let simb = if i == 0 { &diff[2] } else { &diff[0] };
            let nex_simb = nex_diff.map(|d| &d[0]);

            result.extend(diff.clone());
            let map = Self::get_row_map(simb);

            if let Some(n_simb) = nex_simb {
                let nex_map = Self::get_row_map(n_simb);
                let mut start = 0usize;
                if map.get("orgRow").cloned().unwrap_or(0) != 0 {
                    start = (map["orgRow"] + map["orgDel"]).wrapping_sub(1);
                }
                let end = nex_map["revRow"].saturating_sub(2);
                result.extend(Self::get_orig_list(original, start, end));
            }

            let mut start = (map["orgRow"] + map["orgDel"]).wrapping_sub(1);
            if start == usize::MAX {
                start = 0;
            }

            if simb.contains("@@ -1,")
                && nex_simb.is_none()
                && map["orgDel"] != original.len()
            {
                result.extend(Self::get_orig_list(original, start, original.len() - 1));
            } else if nex_simb.is_none()
                && (map["orgRow"] + map["orgDel"]).wrapping_sub(1) < original.len()
            {
                result.extend(Self::get_orig_list(original, start, original.len() - 1));
            }
        }
    }

    fn get_row_map(str_header: &str) -> HashMap<&'static str, usize> {
        let mut map = HashMap::new();
        if str_header.starts_with("@@") {
            let sp: Vec<&str> = str_header.split(' ').collect();
            if sp.len() > 1 {
                let org = sp[1];
                let org_sp: Vec<&str> = org.split(',').collect();
                if org_sp.len() >= 2 {
                    let org_row = org_sp[0][1..].parse::<usize>().unwrap_or(0);
                    let org_del = org_sp[1].parse::<usize>().unwrap_or(0);
                    map.insert("orgRow", org_row);
                    map.insert("orgDel", org_del);
                    map.insert("revRow", org_row);
                    map.insert("revAdd", org_del);
                }
            }
        }
        map
    }

    fn get_orig_list(original_with_prefix: &[String], start: usize, end: usize) -> Vec<String> {
        let mut list = Vec::new();
        if !original_with_prefix.is_empty()
            && start <= end
            && end < original_with_prefix.len()
        {
            for item in original_with_prefix.iter().take(end + 1).skip(start) {
                list.push(item.clone());
            }
        }
        list
    }
}