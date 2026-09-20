# Changes

## 0.1.0-beta.2

### Changed

- Continue the release toward beta.2 with focused Clippy cleanup across the library, binaries, tests, and benchmarks.
- Add `#[must_use]` annotations and behavior-preserving lint fixes without changing the diff algorithms or their performance-sensitive hot paths.
- Keep the fuzz target exercising arbitrary byte input through the public diff and patch APIs.
- Keep the standalone Tester and release benchmark catalogue available for local correctness and stress validation without publishing its generated artifacts, downloaded JARs, or nested Git metadata.

### Validation

- All three validation checks pass: host 64-bit `cargo test`, `cargo +nightly miri test --target i686-unknown-linux-gnu`, and `cargo +nightly miri test --target s390x-unknown-linux-gnu`.
- `cargo clippy --all-targets -- -D warnings` passes with zero warnings and errors.
- The fuzz target completed 1,000,000 runs successfully; the full output is stored locally in `fuzz_run_1m.log` and is excluded from Git.
- The Tester correctness checks and all 11 release benchmark workloads pass, including the 500,000-element stress case.

### Release Checklist Snapshot

| Category | Task | Status |
| --- | --- | --- |
| Correctness | 1M+ fuzzing runs and property-style patch round-trip checks | Done |
| Linting | Zero Clippy warnings across all Cargo targets | Done |
| API | Generic `&[T]` support | Done |
| API | `thiserror`-based custom error types | Pending |
| Performance | Criterion and release benchmark coverage | Partial |
| Performance | Allocation profiling | Pending |
| Automation | GitHub Actions CI matrix for Linux, Windows, and macOS | Pending |
| Documentation | Complete `///` coverage and verified non-ignored doc-tests | Pending |

**Checklist result:** 3 of 8 sub-items are complete, or 2 of the original 6 categories are fully complete. Performance is partially covered by existing benchmarks; it is not counted as complete until allocation profiling is added.

## 0.1.0-beta.1

### Changed

- Update the crate to the public beta release.
- Keep the default algorithm selection tuned for correctness and speed across mixed edit patterns.
- Keep the public-facing documentation aligned with the beta milestone and remove the old alpha preview framing.
- Verify the implementation against repeated, clustered, and standard workloads in release mode.

### Validation

- Full `cargo test` suite passes.
- Release-mode benchmark checks confirm the implementation is faster in many cases without requiring blanket performance claims for every workload.

## 0.1.0-alpha.5

### Changed

- Add hybrid Histogram indexing with hash lookup and occurrence-position vectors.
- Require `Eq + Hash` for the default Histogram-backed `DiffUtils` path.
- Dispatch high-entropy and low-entropy large regions to Myers when Histogram indexing would add unnecessary overhead.
- Apply the configured maximum occurrence-chain limit consistently before anchor selection.
- Extend the standalone tester with clustered and repeated workload verification.

### Tests and Benchmarks

- Full `cargo test` suite passes with zero failures.
- Standalone tester correctness and stress catalogue passes all 11 workloads.
- Clustered and repeated workloads were verified against Myers in release mode.

## 0.1.0-alpha.2

### Changed

- Use `HistogramDiff` as the default algorithm in `DiffUtils`.
- Normalize adjacent histogram insert/delete records that represent one replacement into a single `Change` record.
- Keep low-entropy repeated-value regions on the histogram path instead of forcing them through the generic fallback.
- Update the crate and lockfile version to `0.1.0-alpha.2`.

### Tests

- Added regression coverage for the 100,000-element repeated-value case.
- Verified the affected `algorithm`, `diff_utils_test`, and `patch` targets with 35 passing tests and 1 ignored test.
