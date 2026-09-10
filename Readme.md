# java-diff-utils-rs

> **Individual Rust project**  
> **Target Repository:** [`java-diff-utils`](https://github.com/java-diff-utils/java-diff-utils)

A pure Rust port of `java-diff-utils` — implementing the Myers diff algorithm, patch generation and application, unified diff parsing and generation, and side-by-side / inline diff row rendering.

---

## Status: Fully Ported & 100% Passing

This repository is a complete, faithful Rust port of upstream `java-diff-utils`. All algorithms, patch operations, unified diff readers/writers, and presentation-layer diff row generators match the Java reference behavior and pass the complete test suite (168 passed, 1 ignored upstream matching Java).

### Test Results Summary

| Suite / Test File | Passed | Failed | Ignored | Total | Status / Notes |
| --- | --- | --- | --- | --- | --- |
| `algorithm.rs` | 10 | 0 | 0 | 10 | **PASSED** — Myers core + linear-space variant, including `test_performance_problems_issue_124` performance benchmark. |
| `diff_utils_test.rs` | 6 | 0 | 0 | 6 | **PASSED** — All core diff utility operations pass cleanly. |
| `patch.rs` | 8 | 0 | 0 | 8 | **PASSED** — Patch application, chunk verification, and delta processing pass. |
| `example.rs` | 4 | 0 | 0 | 4 | **PASSED** — End-to-end example fixtures pass cleanly. |
| `generate_unified_diff_test.rs` | 11 | 0 | 0 | 11 | **PASSED** — Unified diff generation across various delta configurations. |
| `unifieddiff.rs` | 41 | 0 | 1 | 42 | **PASSED** — Reader/writer verified across all issue fixtures; 1 test ignored upstream matching Java (`test_patch_with_no_deltas`). |
| `integration_tests.rs` | 36 | 0 | 0 | 36 | **PASSED** — Fuzzy patch exception semantics and Unicode grapheme cluster wrapping safety verified. |
| `text.rs` | 52 | 0 | 0 | 52 | **PASSED** — `DiffRowGenerator` tag placement, delimiter preservation, and inline diff merging match Java reference. |
| **Total** | **168** | **0** | **1** | **169** | **168 / 168 Active Tests Passing (100%)** |

---

## Key Features & Architecture

* **Myers Diff Algorithm (`myers.rs` / `algorithm.rs`)**: Path construction and revision generation matching upstream Java behavior across standard and linear-space Myers variants.
* **Chunk Verification & Patch Application (`patch/`)**: Robust chunk boundary checks and delta application (`InsertDelta`, `DeleteDelta`, `ChangeDelta`, `EqualDelta`) with fuzzy matching support.
* **Unified Diff Parsing & Generation (`unifieddiff/`)**: Full support for standard unified diff format (`@@ -l,s +l,s @@`), git-style diffs, timestamp parsing, and zero-line new-file headers (`@@ -0,0 +1,2 @@`).
* **Side-by-Side & Inline Diff Row Generation (`text/`)**: Complete `DiffRowGenerator` with customizable inline tag formatting (`<span class="...">`), customizable equality processing/normalizers, whitespace preservation/ignoring, and Unicode-safe line wrapping using grapheme clusters.

---

## Running Tests

To run the entire test suite:

```bash
cargo test --release
```

*(Note: `--release` is recommended when running the entire suite due to the large dataset in `test_performance_problems_issue_124`).*

To run specific test suites individually:

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