//! This module provides a multi-file-aware implementation of Unified Diff tools.
//!
//! - To parse a unified diff, use [`UnifiedDiffReader::parse_unified_diff`].
//! - To format/export a diff, use [`UnifiedDiffWriter::write`].

pub mod unified_diff;
pub mod unified_diff_file;
pub mod unified_diff_parser_exception;
pub mod unified_diff_reader;
pub mod unified_diff_writer;