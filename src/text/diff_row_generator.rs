//! Generates `DiffRow` instances for side-by-side or inline text views.
use crate::algorithm::myers::myers_linear::compute_diff;
use crate::patch::chunk::Chunk;
use crate::patch::delta::Delta;
use crate::patch::delta_type::DeltaType;
use crate::patch::patch::Patch;
use crate::text::delta_merge::delta_merge_utils::DeltaMergeUtils;
use crate::text::delta_merge::inline_delta_merge_info::InlineDeltaMergeInfo;
use crate::text::diff_row::{DiffRow, Tag};
use crate::text::string_utils;
use lazy_static::lazy_static;
use regex::Regex;
use std::sync::Arc;

lazy_static! {
    pub static ref SPLIT_BY_WORD_PATTERN: Regex =
        Regex::new(r"\s+|[,.\[\](){}/\\*+\-#<>;:&']+").unwrap();
    static ref WHITESPACE_RE: Regex = Regex::new(r"\s+").unwrap();
    pub static ref WHITESPACE_EQUALITIES_MERGER: InlineDeltaMergerFn =
        whitespace_equalities_merger();
}

// Type alias definitions for flexible closures
pub type EqualizerFn = Arc<dyn Fn(&str, &str) -> bool + Send + Sync>;
pub type TagGeneratorFn = Arc<dyn Fn(Tag, bool) -> String + Send + Sync>;
pub type StringTransformFn = Arc<dyn Fn(&str) -> String + Send + Sync>;
pub type SplitterFn = Arc<dyn Fn(&str) -> Vec<String> + Send + Sync>;
pub type InlineDeltaMergerFn =
    Arc<dyn Fn(&InlineDeltaMergeInfo<String>) -> Vec<Delta<String>> + Send + Sync>;

/// Adjusts whitespace in a string by collapsing consecutive whitespaces into a single space.
pub fn adjust_whitespace(raw: &str) -> String {
    WHITESPACE_RE.replace_all(raw.trim(), " ").to_string()
}

/// Default equalizer checking strict string equality.
pub fn default_equalizer() -> EqualizerFn {
    Arc::new(|orig, rev| orig == rev)
}

/// Equalizer ignoring whitespace differences.
pub fn ignore_whitespace_equalizer() -> EqualizerFn {
    Arc::new(|orig, rev| adjust_whitespace(orig) == adjust_whitespace(rev))
}

/// Default line normalizer for HTML escaping.
pub fn html_line_normalizer() -> StringTransformFn {
    Arc::new(|line| string_utils::normalize(line))
}

/// Character-by-character splitter.
pub fn splitter_by_character() -> SplitterFn {
    Arc::new(|line| line.chars().map(|c| c.to_string()).collect())
}

/// Word-by-word splitter.
pub fn splitter_by_word() -> SplitterFn {
    Arc::new(|line| split_string_preserve_delimiter(line, &SPLIT_BY_WORD_PATTERN))
}

/// Default inline delta merger returning unmodified deltas.
pub fn default_inline_delta_merger() -> InlineDeltaMergerFn {
    Arc::new(|info: &InlineDeltaMergeInfo<String>| info.deltas().to_vec())
}

/// Whitespace equalities inline delta merger.
pub fn whitespace_equalities_merger() -> InlineDeltaMergerFn {
    Arc::new(move |info| {
        DeltaMergeUtils::merge_inline_deltas(info, move |equalities: &[String]| {
            equalities
                .iter()
                .all(|s| WHITESPACE_RE.replace_all(s, "").is_empty())
        })
    })
}

/// Helper function to split a string while preserving matched delimiters.
pub fn split_string_preserve_delimiter(str_input: &str, pattern: &Regex) -> Vec<String> {
    let mut list = Vec::new();
    let mut pos = 0;

    for mat in pattern.find_iter(str_input) {
        if pos < mat.start() {
            list.push(str_input[pos..mat.start()].to_string());
        }
        list.push(mat.as_str().to_string());
        pos = mat.end();
    }

    if pos < str_input.len() {
        list.push(str_input[pos..].to_string());
    }

    list
}

/// Wraps elements in a string sequence with start/end tags.
pub fn wrap_in_tag(
    sequence: &mut Vec<String>,
    start_position: usize,
    end_position: usize,
    tag: Tag,
    tag_generator: &TagGeneratorFn,
    process_diffs: Option<&StringTransformFn>,
    replace_linefeed_with_space: bool,
) {
    if start_position >= sequence.len() && end_position >= sequence.len() {
        return;
    }

    let mut end_pos = end_position.min(sequence.len());

    while end_pos >= start_position {
        while end_pos > start_position {
            if sequence[end_pos - 1] != "\n" {
                break;
            } else if replace_linefeed_with_space {
                sequence[end_pos - 1] = " ".to_string();
                break;
            }
            end_pos -= 1;
        }

        if end_pos == start_position {
            break;
        }

        sequence.insert(end_pos, tag_generator(tag, false));
        if let Some(proc) = process_diffs {
            sequence[end_pos - 1] = proc(&sequence[end_pos - 1]);
        }
        end_pos -= 1;

        while end_pos > start_position {
            if sequence[end_pos - 1] == "\n" {
                if replace_linefeed_with_space {
                    sequence[end_pos - 1] = " ".to_string();
                } else {
                    break;
                }
            }
            if let Some(proc) = process_diffs {
                sequence[end_pos - 1] = proc(&sequence[end_pos - 1]);
            }
            end_pos -= 1;
        }

        sequence.insert(end_pos, tag_generator(tag, true));
        if end_pos == 0 {
            break;
        }
        end_pos -= 1;
    }
}

/// Primary generator for `DiffRow` side-by-side views.
#[derive(Clone)]
pub struct DiffRowGenerator {
    column_width: usize,
    equalizer: EqualizerFn,
    ignore_white_spaces: bool,
    inline_diff_splitter: SplitterFn,
    merge_original_revised: bool,
    old_tag: TagGeneratorFn,
    new_tag: TagGeneratorFn,
    report_lines_unchanged: bool,
    line_normalizer: StringTransformFn,
    process_diffs: Option<StringTransformFn>,
    inline_delta_merger: InlineDeltaMergerFn,
    equality_processor: Option<StringTransformFn>,
    show_inline_diffs: bool,
    replace_original_linefeed_in_changes_with_spaces: bool,
    decompress_deltas: bool,
}

impl DiffRowGenerator {
    /// Creates a builder to configure a `DiffRowGenerator`.
    pub fn create() -> Builder {
        Builder::new()
    }

    /// Helper to forward delimiter splitting.
    pub fn split_string_preserve_delimiter(str_input: &str, pattern: &Regex) -> Vec<String> {
        split_string_preserve_delimiter(str_input, pattern)
    }

    /// Generates `DiffRow` items comparing two string sequences.
    pub fn generate_diff_rows(
        &self,
        original: &[String],
        revised: &[String],
    ) -> Vec<DiffRow> {
        let changes = if self.ignore_white_spaces {
            let orig_norm: Vec<String> = original.iter().map(|s| adjust_whitespace(s)).collect();
            let rev_norm: Vec<String> = revised.iter().map(|s| adjust_whitespace(s)).collect();
            compute_diff(&orig_norm, &rev_norm)
        } else {
            compute_diff(original, revised)
        };

        let mut patch = Patch::generate(original, revised, &changes, false);
        self.generate_diff_rows_from_patch(original, &mut patch)
    }

    /// Generates `DiffRow` items comparing an original text sequence against an existing `Patch`.
    pub fn generate_diff_rows_from_patch(
        &self,
        original: &[String],
        patch: &mut Patch<String>,
    ) -> Vec<DiffRow> {
        let mut diff_rows = Vec::new();
        let mut end_pos = 0;
        let delta_list = patch.deltas().to_vec();

        if self.decompress_deltas {
            for original_delta in &delta_list {
                for delta in self.decompress_deltas_internal(original_delta) {
                    end_pos = self.transform_delta_into_diff_row(original, end_pos, &mut diff_rows, &delta);
                }
            }
        } else {
            for delta in &delta_list {
                end_pos = self.transform_delta_into_diff_row(original, end_pos, &mut diff_rows, delta);
            }
        }

        if end_pos < original.len() {
            for line in &original[end_pos..] {
                let processed = self.process_equalities(line);
                diff_rows.push(self.build_diff_row(Tag::Equal, &processed, &processed));
            }
        }

        diff_rows
    }

    fn transform_delta_into_diff_row(
        &self,
        original: &[String],
        end_pos: usize,
        diff_rows: &mut Vec<DiffRow>,
        delta: &Delta<String>,
    ) -> usize {
        let orig = delta.source();
        let rev = delta.target();

        let target_pos = orig.position().min(original.len());
        if end_pos < target_pos {
            for line in &original[end_pos..target_pos] {
                let processed = self.process_equalities(line);
                diff_rows.push(self.build_diff_row(Tag::Equal, &processed, &processed));
            }
        }

        match delta.delta_type() {
            DeltaType::Insert => {
                for line in rev.lines() {
                    diff_rows.push(self.build_diff_row(Tag::Insert, "", line));
                }
            }
            DeltaType::Delete => {
                for line in orig.lines() {
                    diff_rows.push(self.build_diff_row(Tag::Delete, line, ""));
                }
            }
            _ => {
                if self.show_inline_diffs {
                    diff_rows.extend(self.generate_inline_diffs(delta));
                } else {
                    let max_size = orig.lines().len().max(rev.lines().len());
                    for j in 0..max_size {
                        let orig_line = orig.lines().get(j).map(|s| s.as_str()).unwrap_or("");
                        let rev_line = rev.lines().get(j).map(|s| s.as_str()).unwrap_or("");
                        diff_rows.push(self.build_diff_row(Tag::Change, orig_line, rev_line));
                    }
                }
            }
        }

        orig.last() + 1
    }

    fn decompress_deltas_internal(&self, delta: &Delta<String>) -> Vec<Delta<String>> {
        if delta.delta_type() == DeltaType::Change && delta.source().len() != delta.target().len() {
            let mut deltas = Vec::new();
            let min_size = delta.source().len().min(delta.target().len());
            let orig = delta.source();
            let rev = delta.target();

            let orig_sub = orig.lines()[..min_size].to_vec();
            let rev_sub = rev.lines()[..min_size].to_vec();

            deltas.push(Delta::new(
                DeltaType::Change,
                Chunk::new(orig.position(), orig_sub, None),
                Chunk::new(rev.position(), rev_sub, None),
            ));

            if orig.lines().len() < rev.lines().len() {
                deltas.push(Delta::new(
                    DeltaType::Insert,
                    Chunk::new(orig.position() + min_size, Vec::new(), None),
                    Chunk::new(
                        rev.position() + min_size,
                        rev.lines()[min_size..].to_vec(),
                        None,
                    ),
                ));
            } else {
                deltas.push(Delta::new(
                    DeltaType::Delete,
                    Chunk::new(
                        orig.position() + min_size,
                        orig.lines()[min_size..].to_vec(),
                        None,
                    ),
                    Chunk::new(rev.position() + min_size, Vec::new(), None),
                ));
            }

            deltas
        } else {
            vec![delta.clone()]
        }
    }

    fn build_diff_row(&self, tag_type: Tag, orgline: &str, newline: &str) -> DiffRow {
        if self.report_lines_unchanged {
            DiffRow::new(tag_type, orgline, newline)
        } else {
            let mut wrap_org = self.preprocess_line(orgline);
            if Tag::Delete == tag_type {
                if self.merge_original_revised || self.show_inline_diffs {
                    wrap_org = format!(
                        "{}{}{}",
                        (self.old_tag)(tag_type, true),
                        wrap_org,
                        (self.old_tag)(tag_type, false)
                    );
                }
            }

            let mut wrap_new = self.preprocess_line(newline);
            if Tag::Insert == tag_type {
                if self.merge_original_revised {
                    wrap_org = format!(
                        "{}{}{}",
                        (self.new_tag)(tag_type, true),
                        wrap_new,
                        (self.new_tag)(tag_type, false)
                    );
                } else if self.show_inline_diffs {
                    wrap_new = format!(
                        "{}{}{}",
                        (self.new_tag)(tag_type, true),
                        wrap_new,
                        (self.new_tag)(tag_type, false)
                    );
                }
            }

            DiffRow::new(tag_type, wrap_org, wrap_new)
        }
    }

    fn build_diff_row_without_normalizing(&self, tag_type: Tag, orgline: &str, newline: &str) -> DiffRow {
        DiffRow::new(
            tag_type,
            string_utils::wrap_text(orgline, self.column_width),
            string_utils::wrap_text(newline, self.column_width),
        )
    }

    pub fn normalize_lines(&self, list: &[String]) -> Vec<String> {
        if self.report_lines_unchanged {
            list.to_vec()
        } else {
            list.iter().map(|line| (self.line_normalizer)(line)).collect()
        }
    }

    fn generate_inline_diffs(&self, delta: &Delta<String>) -> Vec<DiffRow> {
        let orig = self.normalize_lines(delta.source().lines());
        let rev = self.normalize_lines(delta.target().lines());

        let joined_orig = orig.join("\n");
        let joined_rev = rev.join("\n");

        let mut orig_list = (self.inline_diff_splitter)(&joined_orig);
        let mut rev_list = (self.inline_diff_splitter)(&joined_rev);

        let changes = if self.ignore_white_spaces {
            let orig_norm: Vec<String> = orig_list.iter().map(|s| adjust_whitespace(s)).collect();
            let rev_norm: Vec<String> = rev_list.iter().map(|s| adjust_whitespace(s)).collect();
            compute_diff(&orig_norm, &rev_norm)
        } else {
            compute_diff_with_equalizer(&orig_list, &rev_list, &self.equalizer)
        };

        let patch = Patch::generate(&orig_list, &rev_list, &changes, false);
        let original_inline_deltas = patch.deltas().to_vec();

        let inline_merge_info = InlineDeltaMergeInfo::new(
            original_inline_deltas,
            orig_list.clone(),
            rev_list.clone(),
        );
        let mut inline_deltas: Vec<Delta<String>> = (self.inline_delta_merger)(&inline_merge_info);
        inline_deltas.reverse();

        for inline_delta in &inline_deltas {
            let inline_orig: &Chunk<String> = inline_delta.source();
            let inline_rev: &Chunk<String> = inline_delta.target();

            match inline_delta.delta_type() {
                DeltaType::Delete => {
                    wrap_in_tag(
                        &mut orig_list,
                        inline_orig.position(),
                        inline_orig.position() + inline_orig.len(),
                        Tag::Delete,
                        &self.old_tag,
                        self.process_diffs.as_ref(),
                        self.replace_original_linefeed_in_changes_with_spaces
                            && self.merge_original_revised,
                    );
                }
                DeltaType::Insert => {
                    if self.merge_original_revised {
                        let insert_slice = &rev_list
                            [inline_rev.position()..inline_rev.position() + inline_rev.len()];
                        let pos = inline_orig.position().min(orig_list.len());
                        for (idx, item) in insert_slice.iter().enumerate() {
                            let item: &String = item;
                            orig_list.insert(pos + idx, item.clone());
                        }
                        wrap_in_tag(
                            &mut orig_list,
                            inline_orig.position(),
                            inline_orig.position() + inline_rev.len(),
                            Tag::Insert,
                            &self.new_tag,
                            self.process_diffs.as_ref(),
                            false,
                        );
                    } else {
                        wrap_in_tag(
                            &mut rev_list,
                            inline_rev.position(),
                            inline_rev.position() + inline_rev.len(),
                            Tag::Insert,
                            &self.new_tag,
                            self.process_diffs.as_ref(),
                            false,
                        );
                    }
                }
                DeltaType::Change => {
                    if self.merge_original_revised {
                        let insert_slice = &rev_list
                            [inline_rev.position()..inline_rev.position() + inline_rev.len()];
                        let pos = (inline_orig.position() + inline_orig.len()).min(orig_list.len());
                        for (idx, item) in insert_slice.iter().enumerate() {
                            let item: &String = item;
                            orig_list.insert(pos + idx, item.clone());
                        }
                        wrap_in_tag(
                            &mut orig_list,
                            inline_orig.position() + inline_orig.len(),
                            inline_orig.position() + inline_orig.len() + inline_rev.len(),
                            Tag::Change,
                            &self.new_tag,
                            self.process_diffs.as_ref(),
                            false,
                        );
                    } else {
                        wrap_in_tag(
                            &mut rev_list,
                            inline_rev.position(),
                            inline_rev.position() + inline_rev.len(),
                            Tag::Change,
                            &self.new_tag,
                            self.process_diffs.as_ref(),
                            false,
                        );
                    }
                    wrap_in_tag(
                        &mut orig_list,
                        inline_orig.position(),
                        inline_orig.position() + inline_orig.len(),
                        Tag::Change,
                        &self.old_tag,
                        self.process_diffs.as_ref(),
                        self.replace_original_linefeed_in_changes_with_spaces
                            && self.merge_original_revised,
                    );
                }
                _ => {}
            }
        }

        let orig_result: String = orig_list.concat();
        let rev_result: String = rev_list.concat();

        let original_split: Vec<String> = orig_result.split('\n').map(|s| s.to_string()).collect();
        let revised_split: Vec<String> = rev_result.split('\n').map(|s| s.to_string()).collect();

        let max_lines = original_split.len().max(revised_split.len());
        let mut diff_rows = Vec::new();

        for j in 0..max_lines {
            let orig_line = original_split.get(j).map(|s| s.as_str()).unwrap_or("");
            let rev_line = revised_split.get(j).map(|s| s.as_str()).unwrap_or("");
            diff_rows.push(self.build_diff_row_without_normalizing(Tag::Change, orig_line, rev_line));
        }

        diff_rows
    }

    fn preprocess_line(&self, line: &str) -> String {
        let normalized = (self.line_normalizer)(line);
        if self.column_width == 0 {
            normalized
        } else {
            string_utils::wrap_text(&normalized, self.column_width)
        }
    }

    fn process_equalities(&self, text: &str) -> String {
        if let Some(ref proc) = self.equality_processor {
            proc(text)
        } else {
            text.to_string()
        }
    }
}

/// Helper function to run Myers diff using a custom equalizer closure.
fn compute_diff_with_equalizer(
    orig: &[String],
    rev: &[String],
    equalizer: &EqualizerFn,
) -> Vec<crate::algorithm::change::Change> {
    if orig.iter().zip(rev.iter()).all(|(a, b)| equalizer(a, b)) && orig.len() == rev.len() {
        Vec::new()
    } else {
        compute_diff(orig, rev)
    }
}

/// Builder for constructing customized `DiffRowGenerator` instances.
pub struct Builder {
    show_inline_diffs: bool,
    ignore_white_spaces: bool,
    decompress_deltas: bool,
    old_tag: TagGeneratorFn,
    new_tag: TagGeneratorFn,
    column_width: usize,
    merge_original_revised: bool,
    report_lines_unchanged: bool,
    inline_diff_splitter: SplitterFn,
    line_normalizer: StringTransformFn,
    process_diffs: Option<StringTransformFn>,
    equalizer: Option<EqualizerFn>,
    replace_original_linefeed_in_changes_with_spaces: bool,
    inline_delta_merger: InlineDeltaMergerFn,
    equality_processor: Option<StringTransformFn>,
}

impl Builder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self {
            show_inline_diffs: false,
            ignore_white_spaces: false,
            decompress_deltas: true,
            old_tag: Arc::new(|_tag, f| {
                if f {
                    "<span class=\"editOldInline\">".to_string()
                } else {
                    "</span>".to_string()
                }
            }),
            new_tag: Arc::new(|_tag, f| {
                if f {
                    "<span class=\"editNewInline\">".to_string()
                } else {
                    "</span>".to_string()
                }
            }),
            column_width: 0,
            merge_original_revised: false,
            report_lines_unchanged: false,
            inline_diff_splitter: splitter_by_character(),
            line_normalizer: html_line_normalizer(),
            process_diffs: None,
            equalizer: None,
            replace_original_linefeed_in_changes_with_spaces: false,
            inline_delta_merger: default_inline_delta_merger(),
            equality_processor: None,
        }
    }

    pub fn show_inline_diffs(mut self, val: bool) -> Self {
        self.show_inline_diffs = val;
        self
    }

    pub fn ignore_white_spaces(mut self, val: bool) -> Self {
        self.ignore_white_spaces = val;
        self
    }

    pub fn report_lines_unchanged(mut self, val: bool) -> Self {
        self.report_lines_unchanged = val;
        self
    }

    pub fn old_tag<F>(mut self, generator: F) -> Self
    where
        F: Fn(Tag, bool) -> String + Send + Sync + 'static,
    {
        self.old_tag = Arc::new(generator);
        self
    }

    pub fn old_tag_simple<F>(mut self, generator: F) -> Self
    where
        F: Fn(bool) -> String + Send + Sync + 'static,
    {
        self.old_tag = Arc::new(move |_tag, f| generator(f));
        self
    }

    pub fn new_tag<F>(mut self, generator: F) -> Self
    where
        F: Fn(Tag, bool) -> String + Send + Sync + 'static,
    {
        self.new_tag = Arc::new(generator);
        self
    }

    pub fn new_tag_simple<F>(mut self, generator: F) -> Self
    where
        F: Fn(bool) -> String + Send + Sync + 'static,
    {
        self.new_tag = Arc::new(move |_tag, f| generator(f));
        self
    }

    pub fn process_diffs<F>(mut self, processor: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.process_diffs = Some(Arc::new(processor));
        self
    }

    pub fn process_equalities<F>(mut self, processor: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.equality_processor = Some(Arc::new(processor));
        self
    }

    pub fn column_width(mut self, width: usize) -> Self {
        self.column_width = width;
        self
    }

    pub fn merge_original_revised(mut self, merge: bool) -> Self {
        self.merge_original_revised = merge;
        self
    }

    pub fn decompress_deltas(mut self, decompress: bool) -> Self {
        self.decompress_deltas = decompress;
        self
    }

    pub fn inline_diff_by_word(mut self, inline_diff_by_word: bool) -> Self {
        self.inline_diff_splitter = if inline_diff_by_word {
            splitter_by_word()
        } else {
            splitter_by_character()
        };
        self
    }

    pub fn inline_diff_by_splitter<F>(mut self, splitter: F) -> Self
    where
        F: Fn(&str) -> Vec<String> + Send + Sync + 'static,
    {
        self.inline_diff_splitter = Arc::new(splitter);
        self
    }

    pub fn line_normalizer<F>(mut self, normalizer: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.line_normalizer = Arc::new(normalizer);
        self
    }

    pub fn equalizer<F>(mut self, equalizer: F) -> Self
    where
        F: Fn(&str, &str) -> bool + Send + Sync + 'static,
    {
        self.equalizer = Some(Arc::new(equalizer));
        self
    }

    pub fn replace_original_linefeed_in_changes_with_spaces(mut self, replace: bool) -> Self {
        self.replace_original_linefeed_in_changes_with_spaces = replace;
        self
    }

    pub fn inline_delta_merger<F>(mut self, merger: F) -> Self
    where
        F: Fn(&InlineDeltaMergeInfo<String>) -> Vec<Delta<String>> + Send + Sync + 'static,
    {
        self.inline_delta_merger = Arc::new(merger);
        self
    }

    pub fn inline_delta_merger_arc(mut self, merger: InlineDeltaMergerFn) -> Self {
        self.inline_delta_merger = merger;
        self
    }

    pub fn build(self) -> DiffRowGenerator {
        let equalizer = match self.equalizer {
            Some(eq) => eq,
            None => {
                if self.ignore_white_spaces {
                    ignore_whitespace_equalizer()
                } else {
                    default_equalizer()
                }
            }
        };

        DiffRowGenerator {
            column_width: self.column_width,
            equalizer,
            ignore_white_spaces: self.ignore_white_spaces,
            inline_diff_splitter: self.inline_diff_splitter,
            merge_original_revised: self.merge_original_revised,
            old_tag: self.old_tag,
            new_tag: self.new_tag,
            report_lines_unchanged: self.report_lines_unchanged,
            line_normalizer: self.line_normalizer,
            process_diffs: self.process_diffs,
            inline_delta_merger: self.inline_delta_merger,
            equality_processor: self.equality_processor,
            show_inline_diffs: self.show_inline_diffs,
            replace_original_linefeed_in_changes_with_spaces: self.replace_original_linefeed_in_changes_with_spaces,
            decompress_deltas: self.decompress_deltas,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}