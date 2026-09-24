# Benchmark Comparison

Cross-language performance comparison for `java-diff-utils-rs` against its reference implementation (`java-diff-utils` / JGit) and, where relevant, libgit2 and Python's standard library.

**Methodology:** 10 warmup runs, 9 measured runs per case, median timing reported. Same input data across all implementations per case. Full harness: [`main.rs`](main.rs), [`JavaBenchmark.java`](JavaBenchmark.java), [`PythonBenchmark.py`](PythonBenchmark.py), [`run-benchmark.ps1`](run-benchmark.ps1).

## Headline result: Histogram diff, Rust vs JGit

The comparison that matters most, since `java-diff-utils`' own Histogram implementation is a direct wrapper around JGit's — this *is* the java-diff-utils comparison for Histogram, not a side one.

| Case | Rust (ms) | Java/JGit (ms) | Winner | Ratio |
|---|---:|---:|---|---:|
| 100k append | 0.287 | 0.368 | Rust | 1.28x |
| 100k clustered | 1.298 | 5.342 | Rust | 4.11x |
| 100k clustered edits | 1.526 | 4.460 | Rust | 2.92x |
| 100k identical | 0.294 | 1.160 | Rust | 3.94x |
| 100k prepend | 0.299 | 0.328 | Rust | 1.09x |
| 100k repeated | 0.848 | 2.726 | Rust | 3.22x |
| 250k identical | 0.539 | 0.494 | Java | 1.09x |

**Rust wins 6 of 7 cases**, several by 2.9x–4.1x. This result has been reproduced consistently across multiple benchmark runs at multiple scales.

This wasn't the starting point. An early Histogram implementation used a linear scan for anchor-bucket lookups, which caused an ~800x slowdown on clustered-edit inputs at smaller scale — the fix was switching to a hash-based lookup (matching how JGit and libgit2's `xhistogram.c` both do it), after which this result held.

## Myers diff: Java wins most cases

| Case | Rust (ms) | Java (ms) | Winner | Ratio |
|---|---:|---:|---|---:|
| 100k alternating edits | 3.076 | 1.685 | Java | 1.83x |
| 100k append | 1.809 | 0.456 | Java | 3.97x |
| 100k clustered | 2.007 | 2.340 | Rust | 1.17x |
| 100k clustered edits | 1.909 | 1.149 | Java | 1.66x |
| 100k high churn | 2.615 | 0.652 | Java | 4.01x |
| 100k identical | 1.681 | 0.581 | Java | 2.89x |
| 100k prepend | 1.983 | 1.143 | Java | 1.74x |
| 100k repeated | 1.935 | 0.318 | Java | 6.09x |

**Java wins 7 of 8 cases.** Java's `String.hashCode()`-style caching and JIT warmup benefit the Myers path more directly than they do Histogram's bucket-lookup-heavy approach. This is a known, open gap — not yet addressed.

## Native default path (Histogram vs Myers, as actually shipped)

| Case | Rust Default (ms) | Java Default (ms) | Winner | Ratio |
|---|---:|---:|---|---:|
| 100k alternating edits | 1.732 | 1.414 | Java | 1.23x |
| 100k append | 0.122 | 0.491 | Rust | 4.03x |
| 100k clustered | 1.383 | 0.989 | Java | 1.40x |
| 100k clustered edits | 1.434 | 1.153 | Java | 1.24x |
| 100k high churn | 0.792 | 0.590 | Java | 1.34x |
| 100k identical | 0.129 | 0.522 | Rust | 4.04x |
| 100k prepend | 0.312 | 1.248 | Rust | 4.01x |
| 100k repeated | 0.902 | 0.461 | Java | 1.96x |
| 250k identical | 0.584 | 1.306 | Rust | 2.24x |

Rust's default (Histogram) beats Java's default (Myers) in 4 of 9 cases here — a mixed but reasonably competitive result for the path most users actually hit.

## Python's `difflib.SequenceMatcher`

Run with `autojunk=False` for a fair, heuristic-free comparison against Rust/Java (neither has an equivalent junk-filtering heuristic).

| Case | Python (ms) |
|---|---:|
| 100k append | 71.962 |
| 100k clustered | 99.062 |
| 100k clustered edits | 101.027 |
| 100k identical | 72.187 |
| 100k prepend | 71.570 |
| 250k append | 199.742 |
| 250k identical | 226.440 |
| 250k prepend | 212.092 |
| 100k alternating edits | **TIMEOUT** |
| 100k high churn | **TIMEOUT** |
| 100k repeated | **TIMEOUT** |
| 250k repeated | **TIMEOUT** |

Python is 50-150x slower than Rust/Java on completed cases, consistent with CPython's interpreted execution model and `difflib`'s lack of a C-accelerated core.

**4 of 12 cases did not complete** within a 60-second budget. This is expected, documented CPython behavior: `autojunk` exists specifically to prevent pathological slowdowns on low-cardinality/high-repetition input, and disabling it (for a fair comparison) reintroduces exactly the case it was built to avoid. Not a bug in Python, `difflib`, or this benchmark harness.

## libgit2 — currently unreliable, not included

An in-process libgit2 comparison (via `git2-rs`, vendored) was attempted but is **not included above** — the current numbers (11-19ms on most cases) don't reflect the underlying diff algorithm alone. `Patch::from_blobs(...)` builds a full patch object with formatted hunks and context lines, doing meaningfully more work than the raw edit-script computation Rust/Java are timed on. This needs a lower-level API call (or a documented acknowledgment that no equivalent exists in `git2-rs`) before it's a fair comparison. Tracked as follow-up work.

## Known limitations

- Myers diff trails Java's JIT-optimized implementation on most workloads — open, not yet addressed
- The libgit2 comparison above is not yet apples-to-apples — excluded until fixed
- Benchmarks reflect synthetic, purpose-built input shapes (append/prepend/clustered/repeated/etc.), not real-world source-code diff corpora
- Single machine, single OS per run — no cross-hardware variance data yet