# java-diff-utils-rs

> **Individual Rust project**
> **Target Repository:** [`java-diff-utils`](https://github.com/java-diff-utils/java-diff-utils)

A pure Rust port of `java-diff-utils` — implementing the Myers diff algorithm, patch generation and application, unified diff parsing, and side-by-side diff rendering.

---

## Status: Work in Progress

This repository is an individual Rust port of `java-diff-utils`. The core Myers diff algorithm, patch application, and unified diff generation/parsing are solid and verified against the test suite. The remaining failures are concentrated in `DiffRowGenerator`/text formatting, one unified-diff header edge case, and two known edge-case divergences.

Numbers below are taken directly from `cargo test --test <suite> -- --nocapture` runs, one binary at a time, so each row is a real, verifiable result.

### Test Results Summary

| Suite / Test File | Passed | Failed | Ignored | Total | Status / Key Failure Notes |
| --- | --- | --- | --- | --- | --- |
| `algorithm.rs` | 10 | 0 | 0 | 10 | **PASSED** — Myers core + linear-space variant, including the 3+ min `test_performance_problems_issue_124` perf test. |
| `diff_utils_test.rs` | 6 | 0 | 0 | 6 | **PASSED** — All core diff utility operations pass. |
| `patch.rs` | 8 | 0 | 0 | 8 | **PASSED** — Patch application, chunk verification, and delta processing pass. |
| `example.rs` | 4 | 0 | 0 | 4 | **PASSED** — End-to-end example fixtures pass cleanly. |
| `generate_unified_diff_test.rs` | 11 | 0 | 0 | 11 | **PASSED** — Standard unified diff generation tests pass. |
| `unifieddiff.rs` | 40 | 1 | 1 | 42 | **FAILED** — Reader/writer verified; single failure on new-file header syntax. |
| `integration_tests.rs` | 34 | 2 | 0 | 36 | **FAILED** — Fuzzy patch exception semantics and unicode wrapping boundaries. |
| `text.rs` | 37 | 15 | 0 | 52 | **FAILED** — `DiffRowGenerator` tag placement and inline diff merging diverge from Java outputs. |
| **Total** | **150** | **18** | **1** | **169** | **150 / 169 passing** |

---

### What's Confirmed Working

* **Myers Diff Algorithm (`myers.rs` / `algorithm.rs`)**: Path construction and revision generation match upstream Java behavior across the core algorithm suite (10/10) and the standalone Myers linear-space variant, including a real performance test on a larger input (`test_performance_problems_issue_124`, ~203s, passes and produces the expected 2 deltas).
* **Chunk Verification (`Chunk::verify_chunk_at`)**: Rewritten to align with Java bounds and line comparison semantics; all `patch.rs` tests (8/8) execute cleanly.
* **Basic Patch Application**: Insert, delete, and change deltas apply and restore correctly across `patch.rs`, `example.rs`, and `diff_utils_test.rs`.
* **Unified Diff Reader**: All 25 `unified_diff_reader_test` cases pass, including numerous upstream issue-reproduction tests (issue_10, issue_33, issue_46, issue_84, issue_107 variants, issue_110, issue_117, issue_122/123, issue_135, issue_141, issue_182 variants, issue_193, issue_201).
* **Unified Diff Generation**: All 11 tests in `generate_unified_diff_test.rs` pass, including edge cases like empty deltas, wrong context length, and header-line-in-text handling.
* **Example Fixtures**: All 4 example suites (compute_difference, original_and_diff variants, apply_patch) pass cleanly.

---

### Key Failure Root Causes

#### 1. Unified Diff Writer (`unifieddiff.rs`)

* **`unified_diff_writer_test::test_write_with_new_file`**: Header position offset assertion failure on new-file diffs — expected header differs from the rendered one for the zero-line-origin case.
* One test (`unified_diff_round_trip_test::test_patch_with_no_deltas`) is intentionally **ignored**, marked "Disabled in original Java test."

#### 2. Integration Edge Cases (`integration_tests.rs`)

* **`test_fuzzy_patch_unsupported`**: Panics because the thrown error variant does not yet match Java exception semantics (`Expected unsupported error variant matching Java exception semantics`).
* **`test_wrap_text_unicode_safety`**: Multi-byte Unicode grapheme cluster boundaries aren't handled correctly during column wrapping — emoji pairs are being split one-per-line instead of kept together, producing extra `<br/>` breaks.

#### 3. Row Generator & Text Formatting (`text.rs`)

15 of 52 tests fail in `diff_row_generator_test`, all variations on the same root cause: inline diff tag placement (`<span class="editOldInline">` / `<span class="editNewInline">`) and delta merging diverge from the Java reference. Affected tests:

* `test_generator_issue14`, `test_generator_issue22`, `test_generator_issue22_2`, `test_generator_issue22_3`
* `test_generator_example1`, `test_generator_example2`
* `test_generator_with_merge3`, `test_generator_with_merge_by_word4`, `test_generator_with_merge_by_word5`
* `test_generator_with_whitespace_delta_merge`, `test_issue129_with_delta_decompression`
* `test_ignore_whitespace_issue64`, `test_ignore_whitespace_issue66`, `test_ignore_whitespace_issue66_2`
* `test_replace_diffs_issue63`

This is the largest remaining chunk of work — the core diff/patch engine is solid, but the presentation-layer row generator needs a deeper pass to match Java's merge/tagging order exactly.

---

## Running Tests

To run a specific test binary individually:

```bash
cargo test --test algorithm
cargo test --test diff_utils_test
cargo test --test patch
cargo test --test example
cargo test --test generate_unified_diff_test
cargo test --test unifieddiff
cargo test --test integration_tests
cargo test --test text
```

Or run everything at once:

```bash
cargo test
```