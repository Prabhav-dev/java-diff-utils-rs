//! Parser for reading and building a `UnifiedDiff` from text streams or readers.

use std::io::{BufRead, BufReader, Read};
use regex::Regex;

use super::unified_diff::UnifiedDiff;
use super::unified_diff_file::UnifiedDiffFile;
use crate::patch::change_delta::ChangeDelta;
use crate::patch::chunk::Chunk;
use crate::patch::delete_delta::DeleteDelta;
use crate::patch::delta::Delta;
use crate::patch::equal_delta::EqualDelta;
use crate::patch::insert_delta::InsertDelta;
use crate::unifieddiff::unified_diff_parser_exception::UnifiedDiffParserException;

lazy_static::lazy_static! {
    static ref UNIFIED_DIFF_CHUNK_REGEXP: Regex =
        Regex::new(r"^@@\s+-(?:(\d+)(?:,(\d+))?)\s+\+(?:(\d+)(?:,(\d+))?)\s+@@").unwrap();
    static ref TIMESTAMP_REGEXP: Regex =
        Regex::new(r"(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}\.\d{3,})(?: [+-]\d+)?").unwrap();

    static ref DIFF_COMMAND_RE: Regex = Regex::new(r"^diff\s").unwrap();
    static ref SIMILARITY_INDEX_RE: Regex = Regex::new(r"^similarity index (\d+)%$").unwrap();
    static ref INDEX_RE: Regex = Regex::new(r"^index\s[\da-zA-Z]+\.\.[\da-zA-Z]+(\s(\d+))?$").unwrap();
    static ref FROM_FILE_RE: Regex = Regex::new(r"^---\s").unwrap();
    static ref TO_FILE_RE: Regex = Regex::new(r"^\+\+\+\s").unwrap();
    static ref RENAME_FROM_RE: Regex = Regex::new(r"^rename\sfrom\s(.+)$").unwrap();
    static ref RENAME_TO_RE: Regex = Regex::new(r"^rename\sto\s(.+)$").unwrap();
    static ref COPY_FROM_RE: Regex = Regex::new(r"^copy\sfrom\s(.+)$").unwrap();
    static ref COPY_TO_RE: Regex = Regex::new(r"^copy\sto\s(.+)$").unwrap();
    static ref NEW_FILE_MODE_RE: Regex = Regex::new(r"^new\sfile\smode\s(\d+)").unwrap();
    static ref DELETED_FILE_MODE_RE: Regex = Regex::new(r"^deleted\sfile\smode\s(\d+)").unwrap();
    static ref OLD_MODE_RE: Regex = Regex::new(r"^old\smode\s(\d+)").unwrap();
    static ref NEW_MODE_RE: Regex = Regex::new(r"^new\smode\s(\d+)").unwrap();
    static ref BINARY_ADDED_RE: Regex = Regex::new(r"^Binary\sfiles\s/dev/null\sand\sb/(.+)\sdiffer").unwrap();
    static ref BINARY_DELETED_RE: Regex = Regex::new(r"^Binary\sfiles\sa/(.+)\sand\s/dev/null\sdiffer").unwrap();
    static ref BINARY_EDITED_RE: Regex = Regex::new(r"^Binary\sfiles\sa/(.+)\sand\sb/(.+)\sdiffer").unwrap();

    static ref LINE_NORMAL_RE: Regex = Regex::new(r"^\s").unwrap();
    static ref LINE_DEL_RE: Regex = Regex::new(r"^-").unwrap();
    static ref LINE_ADD_RE: Regex = Regex::new(r"^\+").unwrap();
}

struct InternalUnifiedDiffReader<R: Read> {
    reader: BufReader<R>,
    last_line: Option<String>,
}

impl<R: Read> InternalUnifiedDiffReader<R> {
    fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            last_line: None,
        }
    }

    fn read_line(&mut self) -> std::io::Result<Option<String>> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;
        if bytes_read == 0 {
            self.last_line = None;
            return Ok(None);
        }
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        self.last_line = Some(line.clone());
        Ok(Some(line))
    }

    fn last_line(&self) -> Option<&str> {
        self.last_line.as_deref()
    }
}

/// Main parser for Unified Diff format.
pub struct UnifiedDiffReader<R: Read> {
    reader: InternalUnifiedDiffReader<R>,
    data: UnifiedDiff,
    actual_file: Option<UnifiedDiffFile>,

    // State for parsing chunks
    original_txt: Vec<String>,
    revised_txt: Vec<String>,
    add_line_idx_list: Vec<usize>,
    del_line_idx_list: Vec<usize>,
    old_ln: usize,
    old_size: usize,
    new_ln: usize,
    new_size: usize,
    del_line_idx: usize,
    add_line_idx: usize,
}

impl<R: Read> UnifiedDiffReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: InternalUnifiedDiffReader::new(reader),
            data: UnifiedDiff::default(),
            actual_file: None,
            original_txt: Vec::new(),
            revised_txt: Vec::new(),
            add_line_idx_list: Vec::new(),
            del_line_idx_list: Vec::new(),
            old_ln: 0,
            old_size: 0,
            new_ln: 0,
            new_size: 0,
            del_line_idx: 0,
            add_line_idx: 0,
        }
    }

    /// Helper static function to parse an input stream into a `UnifiedDiff`.
    pub fn parse_unified_diff(reader: R) -> Result<UnifiedDiff, UnifiedDiffParserException> {
        let mut parser = UnifiedDiffReader::new(reader);
        parser.parse()
    }

    pub fn parse(&mut self) -> Result<UnifiedDiff, UnifiedDiffParserException> {
        let mut current_line = self
            .reader
            .read_line()
            .map_err(|e| UnifiedDiffParserException::new(e.to_string()))?;

        while let Some(ref line_str) = current_line {
            let mut header_txt = String::new();

            // Header parsing loop
            let mut line_opt = Some(line_str.clone());
            while let Some(ref line) = line_opt {
                if self.valid_file_header_line(line) {
                    break;
                } else {
                    header_txt.push_str(line);
                    header_txt.push('\n');
                }
                line_opt = self
                    .reader
                    .read_line()
                    .map_err(|e| UnifiedDiffParserException::new(e.to_string()))?;
            }

            if !header_txt.is_empty() {
                self.data.set_header(header_txt);
            }

            current_line = line_opt;

            if let Some(ref line) = current_line {
                if !UNIFIED_DIFF_CHUNK_REGEXP.is_match(line) {
                    self.init_file_if_necessary()?;

                    while let Some(ref l) = current_line {
                        if UNIFIED_DIFF_CHUNK_REGEXP.is_match(l) {
                            break;
                        }

                        if !self.process_file_header_line(l) {
                            return Err(UnifiedDiffParserException::new(
                                "expected file start line not found",
                            ));
                        }

                        current_line = self
                            .reader
                            .read_line()
                            .map_err(|e| UnifiedDiffParserException::new(e.to_string()))?;
                    }
                }
            }

            if let Some(ref line) = current_line {
                self.process_chunk_line(line)?;

                while let Some(mut l) = self
                    .reader
                    .read_line()
                    .map_err(|e| UnifiedDiffParserException::new(e.to_string()))?
                {
                    l = self.check_for_no_new_line_at_the_end_of_the_file(l)?;

                    if !self.process_data_line(&l) {
                        return Err(UnifiedDiffParserException::new(
                            "expected data line not found",
                        ));
                    }

                    if (self.original_txt.len() == self.old_size
                        && self.revised_txt.len() == self.new_size)
                        || (self.old_size == 0
                            && self.new_size == 0
                            && self.original_txt.len() == self.old_ln
                            && self.revised_txt.len() == self.new_ln)
                    {
                        self.finalize_chunk();
                        break;
                    }
                }

                current_line = self
                    .reader
                    .read_line()
                    .map_err(|e| UnifiedDiffParserException::new(e.to_string()))?;

                if let Some(l) = current_line {
                    let checked = self.check_for_no_new_line_at_the_end_of_the_file(l)?;
                    current_line = Some(checked);
                }
            }

            if let Some(ref l) = current_line {
                if l.starts_with("--") && !l.starts_with("---") {
                    break;
                }
            } else {
                break;
            }
        }

        // Tail parsing
        let mut tail_txt = String::new();
        while let Some(line) = self
            .reader
            .read_line()
            .map_err(|e| UnifiedDiffParserException::new(e.to_string()))?
        {
            if !tail_txt.is_empty() {
                tail_txt.push('\n');
            }
            tail_txt.push_str(&line);
        }

        if !tail_txt.is_empty() {
            self.data.set_tail_txt(tail_txt);
        }

        // Flush last file into result dataset
        if let Some(file) = self.actual_file.take() {
            self.data.add_file(file);
        }

        Ok(self.data.clone())
    }

    fn check_for_no_new_line_at_the_end_of_the_file(
        &mut self,
        line: String,
    ) -> Result<String, UnifiedDiffParserException> {
        if line == r"\ No newline at end of file" {
            if let Some(ref mut file) = self.actual_file {
                file.set_no_new_line_at_the_end_of_the_file(true);
            }
            let next_line = self
                .reader
                .read_line()
                .map_err(|e| UnifiedDiffParserException::new(e.to_string()))?;
            Ok(next_line.unwrap_or_default())
        } else {
            Ok(line)
        }
    }

    pub fn parse_file_names(line: &str) -> (String, String) {
        let split: Vec<&str> = line.split(' ').collect();
        let from = Regex::new(r"^a/").unwrap().replace(split.get(2).copied().unwrap_or(""), "").to_string();
        let to = Regex::new(r"^b/").unwrap().replace(split.get(3).copied().unwrap_or(""), "").to_string();
        (from, to)
    }

    fn init_file_if_necessary(&mut self) -> Result<(), UnifiedDiffParserException> {
        if !self.original_txt.is_empty() || !self.revised_txt.is_empty() {
            return Err(UnifiedDiffParserException::new("Invalid state in reader"));
        }

        if let Some(file) = self.actual_file.take() {
            self.data.add_file(file);
        }
        self.actual_file = Some(UnifiedDiffFile::new());
        Ok(())
    }

    fn valid_file_header_line(&self, line: &str) -> bool {
        DIFF_COMMAND_RE.is_match(line)
            || SIMILARITY_INDEX_RE.is_match(line)
            || INDEX_RE.is_match(line)
            || FROM_FILE_RE.is_match(line)
            || TO_FILE_RE.is_match(line)
            || RENAME_FROM_RE.is_match(line)
            || RENAME_TO_RE.is_match(line)
            || COPY_FROM_RE.is_match(line)
            || COPY_TO_RE.is_match(line)
            || NEW_FILE_MODE_RE.is_match(line)
            || DELETED_FILE_MODE_RE.is_match(line)
            || OLD_MODE_RE.is_match(line)
            || NEW_MODE_RE.is_match(line)
            || BINARY_ADDED_RE.is_match(line)
            || BINARY_DELETED_RE.is_match(line)
            || BINARY_EDITED_RE.is_match(line)
            || UNIFIED_DIFF_CHUNK_REGEXP.is_match(line)
    }

    fn process_file_header_line(&mut self, line: &str) -> bool {
        if let Some(captures) = DIFF_COMMAND_RE.captures(line) {
            let _ = captures;
            if let Some(last_line) = self.reader.last_line() {
                let (from, to) = Self::parse_file_names(last_line);
                if let Some(ref mut file) = self.actual_file {
                    file.set_from_file(from);
                    file.set_to_file(to);
                    file.set_diff_command(line);
                }
            }
            return true;
        }

        if let Some(captures) = SIMILARITY_INDEX_RE.captures(line) {
            if let Some(val) = captures.get(1).and_then(|m| m.as_str().parse::<i32>().ok()) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_similarity_index(Some(val));
                }
            }
            return true;
        }

        if INDEX_RE.is_match(line) {
            if let Some(ref mut file) = self.actual_file {
                if line.len() >= 6 {
                    file.set_index(&line[6..]);
                }
            }
            return true;
        }

        if FROM_FILE_RE.is_match(line) {
            let name = self.extract_file_name(line);
            let ts = self.extract_timestamp(line);
            if let Some(ref mut file) = self.actual_file {
                file.set_from_file(name);
                if let Some(t) = ts {
                    file.set_from_timestamp(t);
                }
            }
            return true;
        }

        if TO_FILE_RE.is_match(line) {
            let name = self.extract_file_name(line);
            let ts = self.extract_timestamp(line);
            if let Some(ref mut file) = self.actual_file {
                file.set_to_file(name);
                if let Some(t) = ts {
                    file.set_to_timestamp(t);
                }
            }
            return true;
        }

        if let Some(captures) = RENAME_FROM_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_rename_from(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = RENAME_TO_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_rename_to(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = COPY_FROM_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_copy_from(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = COPY_TO_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_copy_to(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = NEW_FILE_MODE_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_new_file_mode(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = DELETED_FILE_MODE_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_deleted_file_mode(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = OLD_MODE_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_old_mode(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = NEW_MODE_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_new_mode(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = BINARY_ADDED_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_binary_added(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = BINARY_DELETED_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_binary_deleted(m.as_str());
                }
            }
            return true;
        }

        if let Some(captures) = BINARY_EDITED_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                if let Some(ref mut file) = self.actual_file {
                    file.set_binary_edited(m.as_str());
                }
            }
            return true;
        }

        false
    }

    fn process_chunk_line(&mut self, line: &str) -> Result<(), UnifiedDiffParserException> {
        if let Some(captures) = UNIFIED_DIFF_CHUNK_REGEXP.captures(line) {
            self.old_ln = captures
                .get(1)
                .and_then(|m| m.as_str().parse::<usize>().ok())
                .unwrap_or(1);
            self.old_size = captures
                .get(2)
                .and_then(|m| m.as_str().parse::<usize>().ok())
                .unwrap_or(1);
            self.new_ln = captures
                .get(3)
                .and_then(|m| m.as_str().parse::<usize>().ok())
                .unwrap_or(1);
            self.new_size = captures
                .get(4)
                .and_then(|m| m.as_str().parse::<usize>().ok())
                .unwrap_or(1);

            if self.old_ln == 0 {
                self.old_ln = 1;
            }
            if self.new_ln == 0 {
                self.new_ln = 1;
            }
            Ok(())
        } else {
            Err(UnifiedDiffParserException::new("Invalid chunk header"))
        }
    }

    fn process_data_line(&mut self, line: &str) -> bool {
        if LINE_NORMAL_RE.is_match(line) {
            let cline = &line[1..];
            self.original_txt.push(cline.to_string());
            self.revised_txt.push(cline.to_string());
            self.del_line_idx += 1;
            self.add_line_idx += 1;
            true
        } else if LINE_ADD_RE.is_match(line) {
            let cline = &line[1..];
            self.revised_txt.push(cline.to_string());
            self.add_line_idx += 1;
            self.add_line_idx_list
                .push(self.new_ln - 1 + self.add_line_idx);
            true
        } else if LINE_DEL_RE.is_match(line) {
            let cline = &line[1..];
            self.original_txt.push(cline.to_string());
            self.del_line_idx += 1;
            self.del_line_idx_list
                .push(self.old_ln - 1 + self.del_line_idx);
            true
        } else {
            false
        }
    }

    fn finalize_chunk(&mut self) {
        if !self.original_txt.is_empty() || !self.revised_txt.is_empty() {
            let has_deletes = !self.del_line_idx_list.is_empty();
            let has_inserts = !self.add_line_idx_list.is_empty();
            let has_context = self.original_txt.len() != self.del_line_idx_list.len()
                || self.revised_txt.len() != self.add_line_idx_list.len();

            let orig_chunk = Chunk::new(
                self.old_ln.saturating_sub(1),
                self.original_txt.clone(),
                Some(self.del_line_idx_list.clone()),
            );
            let rev_chunk = Chunk::new(
                self.new_ln.saturating_sub(1),
                self.revised_txt.clone(),
                Some(self.add_line_idx_list.clone()),
            );

            let delta: Delta<String> = if has_context || (has_deletes && has_inserts) {
                ChangeDelta::new(orig_chunk, rev_chunk).into()
            } else if has_deletes {
                DeleteDelta::new(orig_chunk, rev_chunk).into()
            } else if has_inserts {
                InsertDelta::new(orig_chunk, rev_chunk).into()
            } else {
                EqualDelta::new(orig_chunk, rev_chunk).into()
            };

            if let Some(ref mut file) = self.actual_file {
                file.patch_mut().add_delta(delta);
            }

            self.old_ln = 0;
            self.new_ln = 0;
            self.original_txt.clear();
            self.revised_txt.clear();
            self.add_line_idx_list.clear();
            self.del_line_idx_list.clear();
            self.del_line_idx = 0;
            self.add_line_idx = 0;
        }
    }

    fn extract_file_name(&self, line: &str) -> String {
        let mut clean_line = line.to_string();
        if let Some(m) = TIMESTAMP_REGEXP.find(line) {
            clean_line = clean_line[..m.start()].to_string();
        }
        let first_part = clean_line.split('\t').next().unwrap_or(&clean_line);
        let sliced = if first_part.len() >= 4 {
            &first_part[4..]
        } else {
            first_part
        };

        Regex::new(r"^(a|b|old|new)/")
            .unwrap()
            .replace(sliced, "")
            .trim()
            .to_string()
    }

    fn extract_timestamp(&self, line: &str) -> Option<String> {
        TIMESTAMP_REGEXP
            .find(line)
            .map(|m| m.as_str().to_string())
    }
}