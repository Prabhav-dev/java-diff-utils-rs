# Benchmark Comparison

Cross-language performance comparison for `java-diff-utils-rs` against its reference implementation (`java-diff-utils` / JGit), libgit2, and Python's standard library.

**Methodology:** 10 warmup runs, 9 measured runs per case, median timing reported. Same input data across all implementations per case. Full harness: [`Tester/test_app/src/main.rs`](Tester/test_app/src/main.rs), [`Tester/test_app/benchmarks/JavaBenchmark.java`](Tester/test_app/benchmarks/JavaBenchmark.java), [`Tester/test_app/run-benchmark.ps1`](Tester/test_app/run-benchmark.ps1).

---

## 1. Myers Diff: Rust vs Java (100k Workloads)

Direct comparison of the Myers diff algorithm implementation in Rust (`java-diff-utils-rs`) against upstream Java (`java-diff-utils`).

| Case | Rust (ms) | Java (ms) | Winner | Ratio | Deltas |
|---|---:|---:|---|---:|---:|
| 100k alternating edits | 0.676 | 1.288 | **Rust** | 1.91x | 9 |
| 100k append | 0.220 | 0.388 | **Rust** | 1.76x | 1 |
| 100k clustered | 0.083 | 1.664 | **Rust** | 20.15x | 3 |
| 100k clustered edits | 0.213 | 0.478 | **Rust** | 2.24x | 3 |
| 100k high churn | 0.374 | 0.866 | **Rust** | 2.31x | 9 |
| 100k identical | 0.191 | 1.061 | **Rust** | 5.56x | 0 |
| 100k prepend | 0.318 | 0.845 | **Rust** | 2.66x | 1 |
| 100k repeated | 0.108 | 0.216 | **Rust** | 2.00x | 2 |

**Analysis: Rust wins 8 of 8 cases (100% sweep)**, ranging from 1.76x up to 20.15x faster than Java.

---

## 2. Large-Size Myers Benchmark (250k Workloads)

Testing scaling characteristics on large 250k element sequences with Myers diff.

| Case | Rust Myers (ms) | Java Myers (ms) | Winner | Ratio | Rust Deltas | Java Deltas |
|---|---:|---:|---|---:|---:|---:|
| 250k append | 0.551 | 1.580 | **Rust** | 2.86x | 1 | 1 |
| 250k identical | 0.490 | 1.073 | **Rust** | 2.19x | 0 | 0 |
| 250k prepend | 0.564 | 0.856 | **Rust** | 1.52x | 1 | 1 |
| 250k repeated | 0.694 | 1.876 | **Rust** | 2.70x | 3 | 3 |

**Analysis: Rust wins 4 of 4 cases**, maintaining sub-millisecond execution times on 250k element workloads.

---

## 3. Histogram Diff: Rust vs Java (JGit)

Comparing Rust's `HistogramDiff` with upstream Java / Eclipse JGit's `HistogramDiff` implementation.

| Case | Rust (ms) | Java/JGit (ms) | Winner | Ratio | Rust Deltas | Java Deltas |
|---|---:|---:|---|---:|---:|---:|
| 100k append | 0.249 | 0.186 | **Java** | 1.34x | 1 | 1 |
| 100k clustered | 1.367 | 5.525 | **Rust** | 4.04x | 3 | 3 |
| 100k clustered edits | 1.177 | 1.650 | **Rust** | 1.40x | 3 | 3 |
| 100k identical | 0.135 | 0.779 | **Rust** | 5.76x | 0 | 0 |
| 100k prepend | 0.246 | 0.193 | **Java** | 1.27x | 1 | 1 |
| 100k repeated | 0.686 | 0.900 | **Rust** | 1.31x | 2 | 2 |
| 250k identical | 0.557 | 0.695 | **Rust** | 1.25x | 0 | 0 |

**Analysis: Rust wins 5 of 7 cases**, outperforming JGit by up to 5.76x on identical inputs and 4.04x on clustered diffs.

---

## 4. Native Default Benchmark (Rust Histogram vs Java Myers)

Comparison of each library's default diff algorithm configuration (`DiffUtils::diff` in Rust defaults to Histogram; `DiffUtils.diff` in Java defaults to Myers).

| Case | Rust Default (ms) | Java Default (ms) | Winner | Ratio | Rust Deltas |
|---|---:|---:|---|---:|---:|
| 100k alternating edits | 1.216 | 1.524 | **Rust** | 1.25x | 8 |
| 100k append | 0.187 | 0.400 | **Rust** | 2.14x | 1 |
| 100k clustered | 2.114 | 0.778 | **Java** | 2.72x | 3 |
| 100k clustered edits | 1.056 | 0.647 | **Java** | 1.63x | 3 |
| 100k high churn | 0.477 | 0.840 | **Rust** | 1.76x | 9 |
| 100k identical | 0.280 | 0.650 | **Rust** | 2.32x | 0 |
| 100k prepend | 0.296 | 0.520 | **Rust** | 1.76x | 1 |
| 100k repeated | 0.556 | 0.427 | **Java** | 1.30x | 2 |
| 250k identical | 0.510 | 1.701 | **Rust** | 3.33x | 0 |

**Analysis: Rust default wins 6 of 9 cases.**

---

## 5. Rust Internal Algorithm Comparison (Myers vs Histogram)

Benchmarking both algorithms inside Rust on identical inputs to understand trade-offs.

| Case | Myers (ms) | Histogram (ms) | Winner | Ratio | Myers Deltas | Histogram Deltas |
|---|---:|---:|---|---:|---:|---:|
| 100k append | 0.220 | 0.249 | **Myers** | 1.13x | 1 | 1 |
| 100k clustered | 0.083 | 1.367 | **Myers** | 16.55x | 3 | 3 |
| 100k clustered edits | 0.213 | 1.177 | **Myers** | 5.52x | 3 | 3 |
| 100k identical | 0.191 | 0.135 | **Histogram** | 1.41x | 0 | 0 |
| 100k prepend | 0.318 | 0.246 | **Histogram** | 1.29x | 1 | 1 |
| 100k repeated | 0.108 | 0.686 | **Myers** | 6.36x | 2 | 2 |
| 250k identical | 0.490 | 0.557 | **Myers** | 1.14x | 0 | 0 |

**Analysis: Myers wins 5 cases; Histogram wins 2 cases.** Myers excels on clustered and repetitive inputs in Rust, while Histogram provides fast prefix/suffix reduction for identical and prepend patterns.

---

## 6. Python Standard Library (`difflib.SequenceMatcher`)

Benchmarking Python's built-in `difflib.SequenceMatcher` (`autojunk=False` for heuristic-free matching).

| Case | Python (ms) | Python Deltas |
|---|---:|---:|
| 100k append | 52.898 | 1 |
| 100k clustered | 74.494 | 3 |
| 100k clustered edits | 72.882 | 3 |
| 100k identical | 51.929 | 0 |
| 100k prepend | 54.139 | 1 |
| 250k append | 138.876 | 1 |
| 250k identical | 139.502 | 0 |
| 250k prepend | 141.781 | 1 |
| 100k alternating edits | **TIMEOUT** | TIMEOUT |
| 100k high churn | **TIMEOUT** | TIMEOUT |
| 100k repeated | **TIMEOUT** | TIMEOUT |
| 250k repeated | **TIMEOUT** | TIMEOUT |

**Analysis:**
- Python is ~50x–300x slower on completed cases compared to Rust.
- **4 cases timed out:** Disabling `autojunk` creates known pathological complexity on low-cardinality/repetitive sequence matching in CPython's standard library.

---

## 7. libgit2 (In-Process Vendored Normal Diff)

Measured using in-process vendored libgit2 normal (Myers-style) diff on 100k inputs.

| Case | libgit2 (ms) | libgit2 Hunks |
|---|---:|---:|
| 100k alternating edits | 11.557 | 8 |
| 100k append | 12.946 | 1 |
| 100k clustered | 14.131 | 3 |
| 100k clustered edits | 13.967 | 3 |
| 100k high churn | 8.458 | 9 |
| 100k identical | 0.196 | 0 |
| 100k prepend | 12.229 | 1 |
| 100k repeated | 12.180 | 2 |

**Analysis:**
- libgit2 completes 100k diffs in ~8–14 ms on non-identical inputs and 0.196 ms on identical inputs.
- Note: libgit2 creates full patch/hunk objects and formatted structures, doing additional work beyond raw edit-script computation.

---

## Summary & Key Takeaways

1. **Myers Diff Leadership:** Rust outperforms Java across all 8 tested 100k cases (up to 20.15x faster) and all 4 large 250k cases (up to 2.86x faster).
2. **Histogram Competitiveness:** Rust wins 5 of 7 cases against JGit's Histogram diff, running up to 5.76x faster.
3. **Out-of-the-box Defaults:** Rust's default diff path wins 6 of 9 cases against Java's default configuration.
4. **Sub-millisecond Scale:** On 100k–250k line inputs, Rust diffs complete in sub-millisecond to low-millisecond times across all tested patterns.