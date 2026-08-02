//! Writer utility for exporting a `UnifiedDiff` back into standard unified diff format strings or streams.

use std::io::{self, Write};

use super::unified_diff::UnifiedDiff;
use crate::patch::delta::Delta;

/// Writer for outputting unified diff formatting.
pub struct UnifiedDiffWriter;

impl UnifiedDiffWriter {
    /// Writes a `UnifiedDiff` out to a std::io::Write output stream.
    pub fn write<W, F>(
        diff: &UnifiedDiff,
        original_lines_provider: F,
        writer: &mut W,
        context_size: usize,
    ) -> io::Result<()>
    where
        W: Write,
        F: Fn(Option<&str>) -> Vec<String>,
    {
        Self::write_consumer(
            diff,
            original_lines_provider,
            |line| {
                let _ = writeln!(writer, "{}", line);
            },
            context_size,
        )
    }

    /// Writes a `UnifiedDiff` using a line consumer closure.
    pub fn write_consumer<F, C>(
        diff: &UnifiedDiff,
        original_lines_provider: F,
        mut writer: C,
        context_size: usize,
    ) -> io::Result<()>
    where
        F: Fn(Option<&str>) -> Vec<String>,
        C: FnMut(&str),
    {
        if let Some(header) = diff.header() {
            writer(header);
        }

        for file in diff.files() {
            let patch_deltas = file.patch().deltas();
            if !patch_deltas.is_empty() {
                if let Some(cmd) = file.diff_command() {
                    writer(cmd);
                }
                if let Some(index) = file.index() {
                    writer(&format!("index {}", index));
                }

                let from_file_str = match file.from_file() {
                    None => "/dev/null",
                    Some(f) if f.is_empty() => "/dev/null",
                    Some(f) => f,
                };
                writer(&format!("--- {}", from_file_str));

                if let Some(to_file) = file.to_file() {
                    writer(&format!("+++ {}", to_file));
                }

                let original_lines = original_lines_provider(file.from_file());
                let mut deltas: Vec<&Delta<String>> = Vec::new();

                let mut delta = &patch_deltas[0];
                deltas.push(delta);

                if patch_deltas.len() > 1 {
                    for next_delta in patch_deltas.iter().skip(1) {
                        let position = delta.source().position();

                        if (position + delta.source().size() + context_size)
                            >= next_delta.source().position().saturating_sub(context_size)
                        {
                            deltas.push(next_delta);
                        } else {
                            Self::process_deltas(
                                &mut writer,
                                &original_lines,
                                &deltas,
                                context_size,
                                false,
                            );
                            deltas.clear();
                            deltas.push(next_delta);
                        }
                        delta = next_delta;
                    }
                }

                Self::process_deltas(
                    &mut writer,
                    &original_lines,
                    &deltas,
                    context_size,
                    patch_deltas.len() == 1 && file.from_file().is_none(),
                );
            }
        }

        if let Some(tail) = diff.tail() {
            writer("--");
            writer(tail);
        }

        Ok(())
    }

    fn process_deltas<C>(
        writer: &mut C,
        orig_lines: &[String],
        deltas: &[&Delta<String>],
        context_size: usize,
        new_file: bool,
    ) where
        C: FnMut(&str),
    {
        if deltas.is_empty() {
            return;
        }

        let mut buffer: Vec<String> = Vec::new();
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

        Self::get_delta_text(&mut |txt| buffer.push(txt.to_string()), cur_delta);
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

            Self::get_delta_text(&mut |txt| buffer.push(txt.to_string()), next_delta);
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

        writer(&format!(
            "@@ -{},{} +{},{} @@",
            orig_start, orig_total, rev_start, rev_total
        ));

        for txt in buffer {
            writer(&txt);
        }
    }

    fn get_delta_text<C>(writer: &mut C, delta: &Delta<String>)
    where
        C: FnMut(&str),
    {
        for line in delta.source().lines() {
            writer(&format!("-{}", line));
        }
        for line in delta.target().lines() {
            writer(&format!("+{}", line));
        }
    }
}