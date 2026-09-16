# Test Matrix and Verification

This document records the verification matrix for the experimental preview release of `java-diff-utils-rs`.

## Release status

This release is marked as experimental, and it is intended to be evaluated by early adopters and maintainers. The project is correctness-focused first: the public Rust test suite is green, and the Java reference behavior was checked against the upstream `java-diff-utils` implementation on a representative diff case.

## Core suite status

| Suite | Status | Notes |
| --- | --- | --- |
| `algorithm` | Passing | Myers core and linear-space variants, plus histogram coverage |
| `diff_utils_test` | Passing | default API and factory behavior |
| `patch` | Passing | patch generation and patch application |
| `example` | Passing | smoke tests and demo scenarios |
| `generate_unified_diff_test` | Passing | unified diff generation |
| `unifieddiff` | Passing | reader/writer parity and fixture coverage |
| `integration_tests` | Passing | system-level regression checks |
| `text` | Passing | inline and side-by-side row generation |

## Verification commands

The primary verification command is:

```bash
cargo test --quiet
```

For full release-mode validation, especially when large fixture sets are involved:

```bash
cargo test --release
```

To run the longer benchmark intentionally:

```bash
cargo bench --bench myers_diff
```

The slow algorithm performance regression test is intentionally ignored by default because it takes a long time to run. Remove the `#[ignore]` attribute in the corresponding test file when you want to execute that long benchmark deliberately.

## Notes on parity and behavior

- Myers diff remains the default baseline algorithm.
- Myers linear-space diff is retained for memory-efficient large-input workloads.
- Histogram diff is available as an additional anchor-based strategy with fallback to Myers linear-space when stable anchors are not possible.
- The crate enforces a zero-unsafe policy through `#![forbid(unsafe_code)]` at the library and binary roots.
- Performance benchmarking is ongoing; the project does not claim a blanket speed advantage over the Java reference without a controlled benchmark on both implementations.

## Upstream Java parity check

The project was validated against the official `java-diff-utils` library in Docker using the same minimal edit case.

Java reference output:

```text
delta_count=1
CHANGE src=1:1 tgt=1:1
```

This matches the same expected semantic behavior for the equivalent Rust diff case.

## Current verified run

The current project verification command is:

```bash
cargo test --quiet
```

This passes successfully on the repository state in this workspace with the following enabled-test totals:

- 19 passed, 0 failed in algorithm; 1 intentionally ignored (long benchmark)
- 6 passed, 0 failed in diff_utils_test
- 4 passed, 0 failed in example
- 11 passed, 0 failed in generate_unified_diff_test
- 36 passed, 0 failed in integration_tests
- 8 passed, 0 failed in patch
- 52 passed, 0 failed in text
- 42 passed, 0 failed in unifieddiff

Enabled test result: 178 passed, 0 failed, 1 ignored.

The current workspace is a passing build for the standard cargo test command above. The verification status is tracked here so that future changes can be checked against the same benchmark and suite matrix.
