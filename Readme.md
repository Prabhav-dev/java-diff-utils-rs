# java-diff-utils-rs

Rust port of the core diffing and patching behavior behind `java-diff-utils`.

This crate provides Java-compatible diff semantics in a safe, idiomatic Rust implementation with support for Myers diffing, histogram-based diffing, patch generation, and unified diff handling.

## Release status

This is the 0.1.0-beta.1 release.

The project is intended for broader validation and public use. Performance varies by workload, and the implementation is faster in many cases while remaining practical across a wide range of edit patterns.

## Features

- `MyersDiff`: classic quadratic-space Myers algorithm
- `MyersDiffWithLinearSpace`: linear-space Myers variant for larger inputs
- `HistogramDiff`: anchor-based diff path with fallback behavior for tricky sequences
- patch generation and application helpers
- unified diff parsing and writing support
- inline and side-by-side diff row generation

The project is split into a few small, focused modules:

- `src/algorithm/`: diff algorithm implementations and factories
- `src/patch/`: deltas, chunk verification, and patch application
- `src/text/`: diff row generation and string utilities
- `src/unifieddiff/`: unified diff readers and writers
- `src/diff_utils.rs`: public convenience helpers and default selection points

## Safety and design

The crate intentionally enforces a no-unsafe policy:

- `#![forbid(unsafe_code)]` at the crate root and binary entry points
- no raw pointer arithmetic or unsafe memory aliasing
- explicit ownership and borrowing patterns
- bounded, predictable allocation behavior

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

Use Criterion to compare the available algorithms when you want to run the longer benchmark intentionally:

```bash
cargo bench --bench myers_diff
```

The benchmark exercises:

- public API diffing
- Myers quadratic vs. linear-space behavior
- histogram diff behavior on similar and pathological inputs

## Testing

The detailed test matrix and validation commands live in [TESTS.md](TESTS.md).

The standard project verification command is:

```bash
cargo test --quiet
```

## License

This project is licensed under the Apache License, Version 2.0.
See the [LICENSE](LICENSE) file for details.
