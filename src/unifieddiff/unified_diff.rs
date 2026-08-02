//! Data structure representing a complete Unified Diff document containing zero or more file diffs.

use super::unified_diff_file::UnifiedDiffFile;
use crate::patch::patch_failed_exception::PatchFailedException;

/// Container for unified diff header, tail, and multi-file patches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnifiedDiff {
    header: Option<String>,
    tail: Option<String>,
    files: Vec<UnifiedDiffFile>,
}

impl UnifiedDiff {
    /// Creates a new, empty `UnifiedDiff`.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn header(&self) -> Option<&str> {
        self.header.as_deref()
    }

    pub fn set_header(&mut self, header: impl Into<String>) {
        self.header = Some(header.into());
    }

    pub fn add_file(&mut self, file: UnifiedDiffFile) {
        self.files.push(file);
    }

    pub fn files(&self) -> &[UnifiedDiffFile] {
        &self.files
    }

    pub fn files_mut(&mut self) -> &mut Vec<UnifiedDiffFile> {
        &mut self.files
    }

    pub fn set_tail_txt(&mut self, tail_txt: impl Into<String>) {
        self.tail = Some(tail_txt.into());
    }

    pub fn tail(&self) -> Option<&str> {
        self.tail.as_deref()
    }

    /// Finds the target file matching `find_file` predicate and applies its patch to `original_lines`.
    ///
    /// If no file matches, returns `original_lines` unchanged.
    pub fn apply_patch_to<F>(
        &mut self,
        find_file: F,
        original_lines: &[String],
    ) -> Result<Vec<String>, PatchFailedException>
    where
        F: Fn(&str) -> bool,
    {
        let target_file = self.files.iter_mut().find(|diff| {
            diff.from_file()
                .map(|file_path| find_file(file_path))
                .unwrap_or(false)
        });

        if let Some(file) = target_file {
             Ok(file.patch_mut().apply_to(original_lines)?)
        } else {
            Ok(original_lines.to_vec())
        }
    }

    /// Constructs a `UnifiedDiff` from optional header, tail, and a sequence of files.
    pub fn from(
        header: Option<impl Into<String>>,
        tail: Option<impl Into<String>>,
        files: Vec<UnifiedDiffFile>,
    ) -> Self {
        let mut diff = Self::new();
        if let Some(h) = header {
            diff.set_header(h);
        }
        if let Some(t) = tail {
            diff.set_tail_txt(t);
        }
        for file in files {
            diff.add_file(file);
        }
        diff
    }
}