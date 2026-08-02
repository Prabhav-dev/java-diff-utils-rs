# java-diff-utils-rs

A Rust port of [java-diff-utils](https://github.com/java-diff-utils/java-diff-utils) — implementing the Myers diff algorithm, patch generation and application, unified diff parsing, and side-by-side diff rendering.

## Status: Work in Progress — Hackathon Submission

This project is submitted as-is. While the core Myers diff algorithm is solid and well-tested, several peripheral subsystems (such as text formatting, unified diff parsing facades, and fuzzy matching) are incomplete or diverge from the Java reference implementation. 

Below is an honest, exact breakdown of the current test runner output across all 8 test binaries.

### Test Results Summary

| Suite | Passed | Failed | Ignored | Status / Key Failure Notes |
| :--- | :---: | :---: | :---: | :--- |
| `algorithm` | 7 | 2 | 0 | Fuzzy-patch matching and one conflict-processor test failing[cite: 3] |
| `patch` | 5 | 3 | 0 | `Chunk::verify_chunk` and exception-processor mismatches[cite: 5, 9] |
| `text` | 16 | 36 | 0 | `DiffRowGenerator` and string utils largely misaligned with Java semantics[cite: 10] |
| `unifieddiff` | 40 | 1 | 1 | Solid reader/writer behavior; single failure on new-file header syntax (`@@ -1,0 @@` vs `@@ -0,0 @@`)[cite: 7, 10] |
| `integration_tests` | 34 | 2 | 0 | Failing on fuzzy patch unsupported error variants and unicode text-wrap boundary limits[cite: 8] |
| `diff_utils_test` | 2 | 4 | 0 | Facade delta count assertions failing due to default-algorithm selection wiring[cite: 4] |
| `generate_unified_diff_test` | 11 | 0 | 0 | **All passing**[cite: 7] |
| `example` | 0 | 4 | 0 | Missing input fixtures (`original.txt`, etc.) on disk |
| **Total** | **115** | **52** | **1** | |

---

### What's Confirmed Working

* **Myers Diff Algorithm (`myers.rs`)**: Arena and graph-based path construction matches the canonical Java `MyersDiff.buildPath` / `buildRevision` logic, producing correct, minimally-grouped deltas.
* **Basic Patch Application**: Insert, delete, and change deltas apply and restore properly across standard integration suites.
* **Conflict-Marker Output**: `conflict_produces_merge_conflict` correctly generates expected git-style merge conflict blocks.
* **Unified Diff Generation**: Full generation logic runs cleanly and matches expected output structures.

---

### Known Issues & Current Failures

* **`myers_linear.rs` (O(N)-space Myers variant)**: Contains a tie-break bug in the greedy fallback walk (`partition_and_build`), comparing total region size instead of remaining unprocessed length. This triggers panics in fuzzy-patch conformance suites (e.g., `test_fuzzy_apply` pair #16)[cite: 3].
* **`text::string_utils` & `DiffRowGenerator`**: Normalization and wrapping functions diverge significantly from Java reference behaviors (e.g., handling of HTML entities, `<br/>` substitutions, and tab/space normalization directions), cascading into 36 failures in the `text` test suite[cite: 10].
* **Chunk Verification (`Chunk::verify_chunk`)**: Returns `ContentDoesNotMatchTarget` unexpectedly against strict validation checks in `patch` tests[cite: 5, 9].
* **Missing Example Fixtures**: The `example` binary test runner fails immediately due to missing path dependencies (`original.txt`, `issue_170_original.txt`) on disk[cite: 5].

---

## Building

```bash
cargo build