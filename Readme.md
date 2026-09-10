# java-diff-utils-rs

> **Individual Rust project**
> **Target Repository:** [`java-diff-utils`](https://github.com/java-diff-utils/java-diff-utils)

A pure Rust port of `java-diff-utils` — implementing the Myers diff algorithm, patch generation and application, unified diff parsing, and side-by-side diff rendering.

---

## Status: Work in Progress — Individual Project

This repository is an individual Rust port of `java-diff-utils`. The core Myers diff algorithm, patch application, and basic unified diff generation are verified against the test suite. Peripheral subsystems (specifically side-by-side text row generation, string utility wrapping, and specific edge cases in unified diffs and integration error variants) still diverge from the Java reference implementation.

Below is the updated test runner breakdown reflecting the latest test executions across the test suite binaries.

### Test Results Summary

| Suite / Test File | Passed | Failed | Ignored | Total | Status / Failure Notes |
| --- | --- | --- | --- | --- | --- |
| `diff_utils_test.rs` | 6 | 0 | 0 | 6 | **PASSED** — All core diff utility operations pass. |
| `example.rs` | 4 | 0 | 0 | 4 | **PASSED** — End-to-end example fixtures pass cleanly. |
| `generate_unified_diff_test.rs` | 11 | 0 | 0 | 11 | **PASSED** — Standard unified diff generation tests pass. |
| `patch.rs` | 8 | 0 | 0 | 8 | **PASSED** — Patch application, chunk verification, and delta processing pass. |
| `integration_tests.rs` | 34 | 2 | 0 | 36 | **FAILED** — Failing on fuzzy patch exception semantics and unicode wrapping boundaries. |
| `unifieddiff.rs` | 40 | 1 | 1 | 42 | **FAILED** — Reader/writer verified; single header formatting failure on new-file generation. |
| `text.rs` | 37 | 15 | 0 | 52 | **FAILED** — `DiffRowGenerator` tag placement and inline diff merging diverge from Java outputs. |
| **Total Logged Runs** | **149** | **18** | **1** | **168** | **149 / 168 Passing** |

---

### What's Confirmed Working

* **Myers Diff Algorithm (`myers.rs`)**: Path construction and revision generation match upstream Java behavior — verified across core diff tests and standard patch operations.
* **Chunk Verification (`Chunk::verify_chunk_at`)**: Rewritten to align with Java bounds and line comparison semantics; all tests in the `patch` suite execute cleanly without error.
* **Basic Patch Application**: Insert, delete, and change deltas apply and restore correctly across standard test environments.
* **Unified Diff Parsing & Generation**: Unified diff reader parses complex patches and multi-chunk diffs correctly. Standard diff generation works across standard input buffers.
* **Example Fixtures**: Example suites verifying patch applications and diff html generation execute successfully.

---

### Key Failure Root Causes

#### 1. Unified Diff Writer (`unifieddiff.rs`)

* **`unified_diff_writer_test::test_write_with_new_file`**: Header position offset assertion failure:
  * **Expected Header:** `@@ -0,0 +1,2 @@`
  * **Actual Rendered Header:** `@@ -1,0 +1,2 @@`

#### 2. Integration Edge Cases (`integration_tests.rs`)

* **`test_fuzzy_patch_unsupported`**: Panics because the thrown error variant does not yet match Java exception semantics (`Expected unsupported error variant matching Java exception semantics`).
* **`test_wrap_text_unicode_safety`**: String column wrapping logic fails to account for multi-byte Unicode grapheme cluster boundaries properly, leading to mismatched `<br/>` insertion positions.

#### 3. Row Generator & Text Formatting (`text.rs`)

15 test failures remain in `diff_row_generator_test`. These stem from subtle differences in inline diff tag formatting (`<span class="editOldInline">` / `<span class="editNewInline">`), Markdown decoration ordering (`**` / `~`), and word-level delta decompression or whitespace merging:

* Inverted tag/markdown order (e.g., expected `~J. G. Feldstein~**T. P. Pastor**, Chair` vs actual `**T. P. Pastor**~J. G. Feldstein~, Chair` in `test_generator_issue14`).
* Punctuation inclusion within inline diff ranges (e.g., `test_generator_issue22`, `test_generator_example1`).
* Delta decompression line counts on merged inline rows (`test_issue129_with_delta_decompression`).

---

## Running Tests

To run a specific failed test binary individually (e.g., `text.rs` or `integration_tests.rs`):

```bash
cargo test --test integration_tests
cargo test --test unifieddiff
cargo test --test text
```