# Changes

## 0.1.0-beta.3

### Changed

- Accelerated `MyersDiff` (quadratic space) by 2.5x to 19x across all workload shapes while strictly maintaining `#![forbid(unsafe_code)]`.
- Eliminated massive upfront heap allocations in `MyersDiff` by dynamically growing diagonal buffers starting from ~1 KB rather than allocating up to 112 MB on each invocation.
- Integrated thread-local workspace reuse with re-entrancy fallback via `RefCell`, providing zero-allocation diffing for repeated invocations on the same thread.
- Flattened Myers inner loop branching and redundant bounds checks for faster path exploration.
- Fixed the dormant `DiffAlgorithmListener` bug where progress listeners passed to `DiffUtils::diff_with_algorithm`, `DiffUtils::diff`, and `DiffUtils::diff_text` were previously ignored instead of invoking `diff_with_listener`.
- Updated `DiffUtils::diff` and `DiffUtils::diff_text` signatures to accept `Option<&mut dyn DiffAlgorithmListener>`, enabling real progress callbacks during execution.
- Added comprehensive unit tests in `diff_utils_test` and `myers_linear_space_diff_test` verifying authentic listener lifecycle and progress tracking.

### Validation

- Full test suite passes: 170 unit tests passing (100% pass rate).
- All 11 standalone release-mode benchmark cases pass with major speedups across all edit shapes (up to 19.4x faster on clustered edits, 16.5x faster on identical sequences, 5.8x faster on appends).
- Zero warnings across `cargo check --all-targets` and strictly zero unsafe code.

## 0.1.0-beta.2

### Changed

- Eliminated all `#![allow(clippy::...)]` pragmas across the library, tests, and binaries by refactoring internal structures, type aliases, module namespaces, and iteration logic.
- Restructured `myers` and `patch` submodules (`myers_quadratic`, `patch_impl`) to prevent module inception without breaking module re-exports or public API paths.
- Added parameter structs (`LinearCtx`, `HistogramCtx`, `HistoRegion`, `DeltaSpan`) and type aliases (`BoxedEqualizer`, `EqualizerFn`, `BucketsResult`) to reduce function parameter counts and type complexity natively.
- Fixed slice iteration in unified diff writers and utilities to remove `needless_range_loop` warnings cleanly.
- Updated crate documentation (`README.md`, `changes.md`, `Decision.md`) for consistency and accuracy across all project milestones.

### Validation

- All three validation suites pass: host 64-bit `cargo test`, `cargo +nightly miri test --target i686-unknown-linux-gnu`, and `cargo +nightly miri test --target s390x-unknown-linux-gnu`.
- `cargo clippy --all-targets -- -D warnings` passes with **zero warnings and zero errors** without any suppression attributes.
- Local fuzzing validation: 1,000,000+ fuzzing runs verified on local development environment; logs are kept locally and excluded from git repository tracking.
- All 11 release benchmark workloads pass, including the 500,000-element stress test.

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
