# java-diff-utils-rs

A Rust port of [java-diff-utils](https://github.com/java-diff-utils/java-diff-utils) — Myers diff algorithm, patch generation/application, unified diff parsing, and side-by-side diff rendering.

## Status: Work in Progress — hackathon submission

This is submitted as-is. The core diff algorithm is solid and well-tested; several
peripheral subsystems (text rendering, unified diff parsing, a couple of facade
functions) are incomplete or diverge from the Java reference implementation.
Below is an honest breakdown of what works and what doesn't, based on the last
full test run.

### Test results (last full run, 8 test binaries)

| Suite | Passed | Failed | Notes |
|---|---|---|---|
| `algorithm` | 7 | 2 | fuzzy-patch matching and one conflict-processor test still failing |
| `patch` | 5 | 3 | `Chunk::verify_chunk`, conflict-processor tests |
| `text` | 12 | 40 | `DiffRowGenerator` / `string_utils` largely not matching Java semantics yet |
| `unifieddiff` | 9 | 32 (+1 ignored) | most failures are missing test fixture files, not verified logic bugs |
| `integration_tests` | 30 | 6 | HTML escaping, text wrapping, one fuzzy-patch test |
| `diff_utils_test` | 2 | 4 | `DiffUtils::diff` facade producing wrong delta counts — likely a default-algorithm wiring bug |
| `generate_unified_diff_test` | 3 | 8 | mostly missing fixture files under `tests/fixtures/` |
| `example` | 0 | 4 | all failures are missing fixture files (`original.txt`, etc. not present on disk) |
| **Total** | **68** | **99** (+1 ignored) | |

**A meaningful chunk of the failures above are not algorithm bugs** — `tests/fixtures/`
is missing many files the test suite expects (`.diff`/`.patch`/`.txt` fixtures for the
unified-diff reader, round-trip, and example tests). Those need to be restored from
the upstream Java project before those suites can be evaluated at all.

### What's confirmed working

- **Myers diff algorithm (arena/graph-based, `myers.rs`)** — path construction and
  revision-building verified against the canonical Java `MyersDiff.buildPath` /
  `buildRevision` logic; produces correct, minimally-grouped deltas.
- **Basic patch application** — insert, delete, and change deltas apply and restore
  correctly (`test_patch_insert`, `test_patch_delete`, `test_patch_change`, and the
  broader `patch_with_all_diff_algorithms_test` suite all pass).
- **Conflict-marker output** (`conflict_produces_merge_conflict`) — verified against
  its own test's expected git-style merge-conflict output.

### Known issues / not yet correct

- **`myers_linear.rs` (O(N)-space Myers variant)** — has at least one confirmed bug
  in the greedy fallback walk (`partition_and_build`'s tie-break was comparing
  total region size instead of remaining unprocessed length; partially fixed but
  not fully re-verified against the full fuzzy-patch suite).
- **Fuzzy patch matching** (`Patch::apply_fuzzy`) — offset/search-window bookkeeping
  does not yet match the java-diff-utils reference for all cases; the 32-pair
  parametrized conformance test (`test_fuzzy_apply`) is not passing.
- **`text::string_utils`** — `normalize()` and `wrap_text()` currently diverge from
  the Java reference (e.g. producing `\n`-based wrapping where `<br/>` tags are
  expected, and swapping tab/space normalization direction). This cascades into
  most of the 40 `text` suite failures.
- **HTML entity escaping** — currently escapes quotes and apostrophes that the
  reference implementation does not.
- **`DiffUtils::diff` facade** — delta counts don't match expected output in
  `diff_utils_test`; the default-algorithm selection path needs investigation.
- **Unified diff reader/writer** — largely unverified due to missing fixtures;
  the few tests that could run without fixtures (`test_parse_diff_block`,
  `test_chunk_header_parsing*`, `test_simple_pattern`, `test_time_stamp_regexp`)
  do pass.

## Building

\```bash
cargo build
\```

## Running tests

\```bash
cargo test --test algorithm
cargo test --test patch
cargo test --test integration_tests
# etc. — see tests/ for all binaries
\```

Note: `unifieddiff`, `generate_unified_diff_test`, and `example` test binaries
require fixture files under `tests/fixtures/` that are not currently present in
this repo checkout; those tests will fail with "file not found" until fixtures
are restored.

## License

[same as upstream java-diff-utils / Apache 2.0 License ]