# java-diff-utils-rs

A Rust port of [java-diff-utils](https://github.com/java-diff-utils/java-diff-utils) — implementing the Myers diff algorithm, patch generation and application, unified diff parsing, and side-by-side diff rendering.

## Status: Work in Progress — Hackathon Submission

This project is submitted as-is. The core Myers diff algorithm and patch application are now solid and verified byte-for-byte against the real upstream Java implementation. Several peripheral subsystems (text formatting, unified diff parsing edge cases, and fuzzy matching) remain incomplete or diverge from the Java reference.

Below is an honest, exact breakdown of the current test runner output across all 8 test binaries.

### Test Results Summary

| Suite | Passed | Failed | Ignored | Status / Key Failure Notes |
| :--- | :---: | :---: | :---: | :--- |
| `algorithm` | 9 | 1 | 0 | Fuzzy-patch matching (`test_fuzzy_apply`) still failing; core Myers path/coalescing now correct |
| `patch` | 8 | 0 | 0 | **All passing** |
| `text` | 16 | 36 | 0 | `DiffRowGenerator` and string utils largely misaligned with Java semantics |
| `unifieddiff` | 40 | 1 | 1 | Solid reader/writer behavior; single failure on new-file header syntax (`@@ -1,0 @@` vs `@@ -0,0 @@`) |
| `integration_tests` | 34 | 2 | 0 | Failing on fuzzy patch unsupported error variants and unicode text-wrap boundary limits |
| `diff_utils_test` | 6 | 0 | 0 | **All passing** |
| `generate_unified_diff_test` | 11 | 0 | 0 | **All passing** |
| `example` | 4 | 0 | 0 | **All passing** |
| **Total** | **128** | **39** | **1** | |

---

### What's Confirmed Working

* **Myers Diff Algorithm (`myers.rs`)**: Arena and graph-based path construction now matches the canonical Java `MyersDiff.buildPath` / `buildRevision` logic exactly — verified by compiling and instrumenting the actual upstream `MyersDiff.java`/`PathNode.java` and comparing traces node-for-node. Fixed a bug in the snake-collapsing logic (`PathNode::previous_snake`) that was fragmenting and losing edits; deltas now correctly coalesce (or don't) exactly as Java's does.
* **Chunk Verification (`Chunk::verify_chunk_at`)**: Rewritten to match Java's real bounds/loop semantics; all `patch` suite tests pass.
* **Conflict-Marker Output**: `conflict_produces_merge_conflict` rewritten to match upstream behavior exactly — it never applies the delta's target replacement, only wraps the actual-vs-source content in git-style conflict markers in place.
* **Basic Patch Application**: Insert, delete, and change deltas apply and restore properly across standard integration suites.
* **Unified Diff Generation**: Full generation logic runs cleanly and matches expected output structures.
* **Example Fixtures**: Fixed path references (were pointing at a nonexistent `mocks/` directory instead of the existing `tests/fixtures/`).

---

### Known Issues & Current Failures

* **Fuzzy patch application (`test_fuzzy_apply`)**: Still failing — a separate code path from the core diff fix above, not yet investigated.
* **`myers_linear.rs` (O(N)-space Myers variant)**: Note — this is a legitimately *different* valid edit sequence from the regular Myers algorithm for some inputs (verified against Java's `MyersDiffWithLinearSpace`, which also diverges from `MyersDiff` on the same input). Some tests were previously importing the wrong algorithm module entirely; this has been fixed, but the linear-space algorithm itself hasn't been independently audited to the same depth as the regular one.
* **`text::string_utils` & `DiffRowGenerator`**: Normalization and wrapping functions diverge significantly from Java reference behaviors (e.g., handling of HTML entities, `<br/>` substitutions, and tab/space normalization directions), cascading into 36 failures in the `text` test suite. **This is the largest remaining chunk of work.**
* **`integration_tests`**: `test_fuzzy_patch_unsupported` (error variant naming mismatch) and `test_wrap_text_unicode_safety` (grapheme-boundary bug — appears to duplicate/double a unicode character per wrapped line instead of placing one).
* **`unifieddiff`**: One failure on new-file header syntax (`@@ -1,0 @@` vs `@@ -0,0 @@`).

---

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

To compare behavior directly against the real upstream Java implementation (useful for root-causing any further divergences):

```bash
git clone https://github.com/java-diff-utils/java-diff-utils.git
cd java-diff-utils/java-diff-utils/src/main/java
javac -d /tmp/javaout $(find com -iname "*.java")
# then compile a small driver against /tmp/javaout to compare output
```
