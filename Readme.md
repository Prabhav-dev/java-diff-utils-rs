# java-diff-utils-rs

Experimental preview release of a Rust port of the core diffing and patching behavior behind `java-diff-utils`.

This project aims to provide Java-compatible diff semantics in a safe, idiomatic Rust implementation while keeping the public API approachable for users who want patch generation, text diffing, and unified diff support without unsafe Rust code.

## Release status

This is an experimental alpha release.

The crate is intended for early adopters and maintainers who want to evaluate the Rust port against the upstream Java project. The public API is intentionally focused on correctness and parity rather than making unsupported speed claims. Performance benchmarking is ongoing and the long-running algorithm benchmark is kept ignored by default so normal test runs stay fast.

## Overview

This crate includes:

- `MyersDiff`: classic quadratic-space Myers algorithm
- `MyersDiffWithLinearSpace`: linear-space Myers variant for larger inputs
- `HistogramDiff`: anchor-based diff path with fallback behavior for tricky sequences
- patch generation and application helpers
- unified diff parsing and writing support
- inline and side-by-side diff row generation

The project is structured around a small set of clearly separated layers:

- `src/algorithm/`: diff algorithm implementations and factories
- `src/patch/`: deltas, chunk verification, and patch application
- `src/text/`: diff row generation and string utilities
- `src/unifieddiff/`: unified diff readers and writers
- `src/diff_utils.rs`: public convenience helpers and default selection points

## Safety and idiomatic constraints

The crate intentionally enforces a no-unsafe policy:

- `#![forbid(unsafe_code)]` at the crate root and binary entry points
- no raw pointer arithmetic or unsafe memory aliasing
- explicit ownership and borrowing patterns
- bounded, predictable workspace allocation strategies

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

## Upstream Java parity check

The port was validated against the official `java-diff-utils` reference library in Docker using the same representative edit scenario.

Java reference output:

```text
delta_count=1
CHANGE src=1:1 tgt=1:1
```

The Rust implementation matches the same semantic result for the equivalent diff case.

## Performance notes

The implementation is designed to balance correctness and efficiency, but this project does not yet make a blanket claim that it is faster than Java. Performance measurement is ongoing, and the long-running algorithm benchmark is intentionally ignored by default so the standard validation loop stays practical for day-to-day development.

## Benchmarking

Use Criterion to compare the available algorithms when you want to run the longer benchmark intentionally:

```bash
cargo bench --bench myers_diff
```

The benchmark exercises:

- public API diffing
- Myers quadratic vs linear-space behavior
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
