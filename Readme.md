# java-diff-utils-rs

> **Individual Rust project**
> **Target Repository:** [`java-diff-utils`](https://github.com/java-diff-utils/java-diff-utils)

A pure Rust port of `java-diff-utils` — implementing the Myers diff algorithm, patch generation and application, unified diff parsing, and side-by-side diff rendering.

---

## Status: Work in Progress — Individual Project

This is an individual Rust port of `java-diff-utils`. The core Myers diff algorithm, patch application, and fuzzy matching are verified against the test suite. Peripheral subsystems (text formatting and a few unified diff and integration edge cases) still diverge from the Java reference.

Below is the current test runner output across all 8 test binaries, excluding the long performance test `test_performance_problems_issue_124`.

### Test Results Summary

| Suite | Passed | Failed | Ignored | Status / Key Failure Notes |
| :--- | :---: | :---: | :---: | :--- |
| `algorithm` | 9 | 0 | 1 | Core Myers, linear-space, patch, listener, and fuzzy tests passing; performance test excluded |
| `patch` | 8 | 0 | 0 | **All passing** |
| `text` | 16 | 36 | 0 | `DiffRowGenerator` and string utils largely misaligned with Java semantics |
| `unifieddiff` | 40 | 1 | 1 | Solid reader/writer behavior; single failure on new-file header syntax (`@@ -1,0 @@` vs `@@ -0,0 @@`) |
| `integration_tests` | 34 | 2 | 0 | Failing on fuzzy patch unsupported error variants and unicode text-wrap boundary limits |
| `diff_utils_test` | 6 | 0 | 0 | **All passing** |
| `generate_unified_diff_test` | 11 | 0 | 0 | **All passing** |
| `example` | 4 | 0 | 0 | **All passing** |
| **Total** | **128** | **39** | **1** | Performance test excluded from these totals |

---

### What's Confirmed Working

* **Myers Diff Algorithm (`myers.rs`)**: Arena and graph-based path construction matches canonical Java `MyersDiff.buildPath` / `buildRevision` logic exactly — verified by compiling and instrumenting upstream `MyersDiff.java`/`PathNode.java` and comparing traces node-for-node.
* **Chunk Verification (`Chunk::verify_chunk_at`)**: Rewritten to match Java bounds/loop semantics; all `patch` suite tests pass cleanly.
* **Conflict-Marker Output**: `conflict_produces_merge_conflict` rewritten to match upstream behavior exactly — wraps actual-vs-source content in git-style conflict markers in place.
* **Basic Patch Application**: Insert, delete, and change deltas apply and restore properly across standard integration suites.
* **Unified Diff Generation**: Full generation logic runs cleanly and matches expected output structures.
* **Example Fixtures**: Fixed path references pointing to existing `tests/fixtures/`.

---

### Known Issues & Current Failures

* **`myers_linear.rs` (O(N)-space Myers variant)**: Diverges from standard `MyersDiff` on certain edit sequences, matching upstream Java's `MyersDiffWithLinearSpace` divergence behavior.
* **`text::string_utils` & `DiffRowGenerator`**: Normalization and wrapping functions diverge from Java reference behaviors (e.g., HTML entities, `<br/>` substitutions), cascading into 36 failures in the `text` suite.
* **`integration_tests`**: `test_fuzzy_patch_unsupported` (error variant naming mismatch) and `test_wrap_text_unicode_safety` (grapheme-boundary bug).
* **`unifieddiff`**: One header syntax mismatch on new-file diffs (`@@ -1,0 @@` vs `@@ -0,0 @@`).

---

## Building

```bash
cargo build