//! Data structure representing one patched file in a unified diff document.

use crate::patch::patch::Patch;

/// Holds metadata and the patch for a single file in a unified diff.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnifiedDiffFile {
    diff_command: Option<String>,
    from_file: Option<String>,
    from_timestamp: Option<String>,
    to_file: Option<String>,
    rename_from: Option<String>,
    rename_to: Option<String>,
    copy_from: Option<String>,
    copy_to: Option<String>,
    to_timestamp: Option<String>,
    index: Option<String>,
    new_file_mode: Option<String>,
    old_mode: Option<String>,
    new_mode: Option<String>,
    deleted_file_mode: Option<String>,
    binary_added: Option<String>,
    binary_deleted: Option<String>,
    binary_edited: Option<String>,
    patch: Patch<String>,
    no_new_line_at_the_end_of_the_file: bool,
    similarity_index: Option<i32>,
}

impl UnifiedDiffFile {
    /// Constructs a new, empty `UnifiedDiffFile`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a `UnifiedDiffFile` with initial `from_file`, `to_file`, and `patch`.
    pub fn from(
        from_file: impl Into<String>,
        to_file: impl Into<String>,
        patch: Patch<String>,
    ) -> Self {
        Self {
            from_file: Some(from_file.into()),
            to_file: Some(to_file.into()),
            patch,
            ..Default::default()
        }
    }

    pub fn diff_command(&self) -> Option<&str> {
        self.diff_command.as_deref()
    }

    pub fn set_diff_command(&mut self, diff_command: impl Into<String>) {
        self.diff_command = Some(diff_command.into());
    }

    pub fn from_file(&self) -> Option<&str> {
        self.from_file.as_deref()
    }

    pub fn set_from_file(&mut self, from_file: impl Into<String>) {
        self.from_file = Some(from_file.into());
    }

    pub fn to_file(&self) -> Option<&str> {
        self.to_file.as_deref()
    }

    pub fn set_to_file(&mut self, to_file: impl Into<String>) {
        self.to_file = Some(to_file.into());
    }

    pub fn index(&self) -> Option<&str> {
        self.index.as_deref()
    }

    pub fn set_index(&mut self, index: impl Into<String>) {
        self.index = Some(index.into());
    }

    pub fn patch(&self) -> &Patch<String> {
        &self.patch
    }

    pub fn patch_mut(&mut self) -> &mut Patch<String> {
        &mut self.patch
    }

    pub fn set_patch(&mut self, patch: Patch<String>) {
        self.patch = patch;
    }

    pub fn from_timestamp(&self) -> Option<&str> {
        self.from_timestamp.as_deref()
    }

    pub fn set_from_timestamp(&mut self, from_timestamp: impl Into<String>) {
        self.from_timestamp = Some(from_timestamp.into());
    }

    pub fn to_timestamp(&self) -> Option<&str> {
        self.to_timestamp.as_deref()
    }

    pub fn set_to_timestamp(&mut self, to_timestamp: impl Into<String>) {
        self.to_timestamp = Some(to_timestamp.into());
    }

    pub fn similarity_index(&self) -> Option<i32> {
        self.similarity_index
    }

    pub fn set_similarity_index(&mut self, similarity_index: Option<i32>) {
        self.similarity_index = similarity_index;
    }

    pub fn rename_from(&self) -> Option<&str> {
        self.rename_from.as_deref()
    }

    pub fn set_rename_from(&mut self, rename_from: impl Into<String>) {
        self.rename_from = Some(rename_from.into());
    }

    pub fn rename_to(&self) -> Option<&str> {
        self.rename_to.as_deref()
    }

    pub fn set_rename_to(&mut self, rename_to: impl Into<String>) {
        self.rename_to = Some(rename_to.into());
    }

    pub fn copy_from(&self) -> Option<&str> {
        self.copy_from.as_deref()
    }

    pub fn set_copy_from(&mut self, copy_from: impl Into<String>) {
        self.copy_from = Some(copy_from.into());
    }

    pub fn copy_to(&self) -> Option<&str> {
        self.copy_to.as_deref()
    }

    pub fn set_copy_to(&mut self, copy_to: impl Into<String>) {
        self.copy_to = Some(copy_to.into());
    }

    pub fn new_file_mode(&self) -> Option<&str> {
        self.new_file_mode.as_deref()
    }

    pub fn set_new_file_mode(&mut self, new_file_mode: impl Into<String>) {
        self.new_file_mode = Some(new_file_mode.into());
    }

    pub fn deleted_file_mode(&self) -> Option<&str> {
        self.deleted_file_mode.as_deref()
    }

    pub fn set_deleted_file_mode(&mut self, deleted_file_mode: impl Into<String>) {
        self.deleted_file_mode = Some(deleted_file_mode.into());
    }

    pub fn old_mode(&self) -> Option<&str> {
        self.old_mode.as_deref()
    }

    pub fn set_old_mode(&mut self, old_mode: impl Into<String>) {
        self.old_mode = Some(old_mode.into());
    }

    pub fn new_mode(&self) -> Option<&str> {
        self.new_mode.as_deref()
    }

    pub fn set_new_mode(&mut self, new_mode: impl Into<String>) {
        self.new_mode = Some(new_mode.into());
    }

    pub fn binary_added(&self) -> Option<&str> {
        self.binary_added.as_deref()
    }

    pub fn set_binary_added(&mut self, binary_added: impl Into<String>) {
        self.binary_added = Some(binary_added.into());
    }

    pub fn binary_deleted(&self) -> Option<&str> {
        self.binary_deleted.as_deref()
    }

    pub fn set_binary_deleted(&mut self, binary_deleted: impl Into<String>) {
        self.binary_deleted = Some(binary_deleted.into());
    }

    pub fn binary_edited(&self) -> Option<&str> {
        self.binary_edited.as_deref()
    }

    pub fn set_binary_edited(&mut self, binary_edited: impl Into<String>) {
        self.binary_edited = Some(binary_edited.into());
    }

    pub fn is_no_new_line_at_the_end_of_the_file(&self) -> bool {
        self.no_new_line_at_the_end_of_the_file
    }

    pub fn set_no_new_line_at_the_end_of_the_file(&mut self, val: bool) {
        self.no_new_line_at_the_end_of_the_file = val;
    }
}