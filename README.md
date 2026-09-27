# java-diff-utils-rs

Rust port of the core diffing and patching behavior behind `java-diff-utils`.

This crate provides Java-compatible diff semantics in a safe, idiomatic Rust implementation with support for Myers diffing, histogram-based diffing, patch generation, and unified diff handling.

## Release status

This is the `0.1.0-beta.3` release.

The project is suitable for production validation and public use. All core algorithms, patch generation, and unified diff utilities match upstream behavior with zero unsafe code.

## Features

- `MyersDiff`: classic quadratic-space Myers algorithm
- `MyersDiffWithLinearSpace`: linear-space Myers variant for larger inputs
- `HistogramDiff`: anchor-based diff path with fallback behavior for tricky sequences
- Patch generation and application helpers
- Unified diff parsing and writing support
- Inline and side-by-side diff row generation

The project is split into clean, focused modules:

- `src/algorithm/`: diff algorithm implementations, factories, and listeners
- `src/patch/`: deltas, chunk verification, error types, and patch application
- `src/text/`: diff row generation and string utilities
- `src/unifieddiff/`: unified diff readers and writers
- `src/diff_utils.rs`: public convenience helpers and default selection points

## Safety and Code Quality

The crate strictly enforces zero unsafe code and zero Clippy suppressions:

- `#![forbid(unsafe_code)]` at the crate root and binary entry points
- Zero raw pointer arithmetic or unsafe memory aliasing
- **Zero Clippy warnings** enforced via `cargo clippy --all-targets -- -D warnings` without relying on `#![allow(clippy::...)]` pragmas
- Explicit ownership, borrowing, and clean type abstractions

## Quick start

```rust
use java_diff_utils_rs::{DiffUtils, HistogramDiff, MyersDiff};
use java_diff_utils_rs::algorithm::DiffAlgorithm;

let original = vec!["A", "B", "C", "D"];
let revised = vec!["A", "X", "C", "D"];

let changes = MyersDiff::default().diff(&original, &revised);
let patch = java_diff_utils_rs::Patch::generate(&original, &revised, &changes, false);
let applied = patch.apply_to(&original).unwrap();
assert_eq!(applied, revised);

let histogram_changes = HistogramDiff::new().diff(&original, &revised);
assert!(!histogram_changes.is_empty());
```

## Benchmarking

Use Criterion to benchmark and compare the available algorithms:

```bash
cargo bench --bench myers_diff
```

The benchmark exercises:

- Public API diffing
- Myers quadratic vs. linear-space behavior
- Histogram diff behavior on similar and pathological inputs

## Testing & Fuzzing

The detailed test matrix and validation commands live in [TESTS.md](TESTS.md).

Run the standard test suite:

```bash
cargo test --quiet
```

### Fuzzing

Fuzzing targets live in `fuzz/`. Local verification includes 1,000,000+ fuzz iterations on arbitrary inputs via:

```bash
cargo fuzz run --manifest-path fuzz/Cargo.toml fuzz_target_1
```

### Docker

Build and run the release container:

```bash
docker build -t java-diff-utils-rs .
docker run --rm java-diff-utils-rs
```

The Docker build executes the test suite before building the release demonstration binary.

### Miri Validation

Cross-target Miri validation checks for undefined behavior:

```bash
cargo +nightly miri test --target i686-unknown-linux-gnu
cargo +nightly miri test --target s390x-unknown-linux-gnu
```

## License

This project is licensed under the Apache License, Version 2.0.
See the [LICENSE](LICENSE) file for details.
