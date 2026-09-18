/// Criterion benchmark suite for java-diff-utils-rs Myers diff algorithms.
///
/// Benchmarks both the default public API path (DiffUtils::diff -> MyersDiffWithLinearSpace)
/// and the explicit linear-space and quadratic-space algorithms across representative input sizes.
///
/// Run with:
///   cargo bench
///   cargo bench -- --output-format bencher  (for compact output)
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use java_diff_utils_rs::algorithm::histogram::HistogramDiff;
use java_diff_utils_rs::algorithm::myers::myers::MyersDiff;
use java_diff_utils_rs::algorithm::myers::myers_linear::MyersDiffWithLinearSpace;
use java_diff_utils_rs::algorithm::DiffAlgorithm;
use java_diff_utils_rs::diff_utils::DiffUtils;
use java_diff_utils_rs::patch::Patch;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Build the pathological case: 1 source line vs N unique target lines.
/// This maximises edit distance and is the exact input from GitHub issue #124.
fn make_pathological(n: usize) -> (Vec<String>, Vec<String>) {
    let source = vec!["abcd".to_string()];
    let target: Vec<String> = (0..n).map(|i| i.to_string()).collect();
    (source, target)
}

/// Build a realistic "similar files" case: two sequences sharing ~80% of lines.
fn make_similar(n: usize) -> (Vec<String>, Vec<String>) {
    let original: Vec<String> = (0..n).map(|i| format!("line {i}")).collect();
    let revised: Vec<String> = (0..n)
        .map(|i| {
            if i % 5 == 0 {
                format!("changed line {i}")
            } else {
                format!("line {i}")
            }
        })
        .collect();
    (original, revised)
}

/// Build clustered input: long repeated blocks with a small changed region.
/// This is the workload where low-occurrence anchors should avoid Myers' broad search.
fn make_clustered(n: usize) -> (Vec<String>, Vec<String>) {
    let source: Vec<String> = (0..n)
        .map(|i| format!("block-{}", (i / 32) % 8))
        .collect();
    let mut target = source.clone();
    for value in target.iter_mut().skip(n / 2).take(32) {
        *value = "changed-cluster".to_string();
    }
    (source, target)
}

// ── benchmark groups ──────────────────────────────────────────────────────────

/// Benchmarks the public DiffUtils::diff() API path (uses MyersDiffWithLinearSpace by default).
fn bench_public_api_pathological(c: &mut Criterion) {
    let sizes = [100, 500, 1_000, 5_000, 10_000];
    let mut group = c.benchmark_group("public_api/pathological");
    group.sample_size(10);

    for &n in &sizes {
        let (source, target) = make_pathological(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let patch = DiffUtils::diff(black_box(&source), black_box(&target), None);
                black_box(patch.deltas().len())
            })
        });
    }
    group.finish();
}

/// Benchmarks the public DiffUtils::diff() API on realistic similar-file input.
fn bench_public_api_similar(c: &mut Criterion) {
    let sizes = [100, 1_000, 5_000, 10_000];
    let mut group = c.benchmark_group("public_api/similar");
    group.sample_size(10);

    for &n in &sizes {
        let (source, target) = make_similar(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let patch = DiffUtils::diff(black_box(&source), black_box(&target), None);
                black_box(patch.deltas().len())
            })
        });
    }
    group.finish();
}

/// Head-to-head: MyersDiffWithLinearSpace vs MyersDiff on pathological input.
fn bench_algo_comparison_pathological(c: &mut Criterion) {
    let sizes = [100, 500, 1_000, 5_000];
    let mut group = c.benchmark_group("algo_comparison/pathological");
    group.sample_size(10);

    for &n in &sizes {
        let (source, target) = make_pathological(n);

        group.bench_with_input(BenchmarkId::new("linear_space", n), &n, |b, _| {
            let algo = MyersDiffWithLinearSpace::<String>::default();
            b.iter(|| {
                let changes = algo.diff(black_box(&source), black_box(&target));
                let patch = Patch::generate(&source, &target, black_box(&changes), false);
                black_box(patch.deltas().len())
            })
        });

        group.bench_with_input(BenchmarkId::new("quadratic_space", n), &n, |b, _| {
            let algo = MyersDiff::<String>::default();
            b.iter(|| {
                let changes = algo.diff(black_box(&source), black_box(&target));
                let patch = Patch::generate(&source, &target, black_box(&changes), false);
                black_box(patch.deltas().len())
            })
        });
    }
    group.finish();
}

/// Head-to-head: MyersDiffWithLinearSpace vs MyersDiff on similar-file input.
fn bench_algo_comparison_similar(c: &mut Criterion) {
    let sizes = [100, 1_000, 5_000, 10_000];
    let mut group = c.benchmark_group("algo_comparison/similar");
    group.sample_size(10);

    for &n in &sizes {
        let (source, target) = make_similar(n);

        group.bench_with_input(BenchmarkId::new("linear_space", n), &n, |b, _| {
            let algo = MyersDiffWithLinearSpace::<String>::default();
            b.iter(|| {
                let changes = algo.diff(black_box(&source), black_box(&target));
                let patch = Patch::generate(&source, &target, black_box(&changes), false);
                black_box(patch.deltas().len())
            })
        });

        group.bench_with_input(BenchmarkId::new("quadratic_space", n), &n, |b, _| {
            let algo = MyersDiff::<String>::default();
            b.iter(|| {
                let changes = algo.diff(black_box(&source), black_box(&target));
                let patch = Patch::generate(&source, &target, black_box(&changes), false);
                black_box(patch.deltas().len())
            })
        });
    }
    group.finish();
}

/// HistogramDiff benchmark on the same representative workloads, highlighting
/// the low-occurrence anchor strategy against the Myers families.
fn bench_histogram_comparison(c: &mut Criterion) {
    let sizes = [100, 500, 1_000, 5_000];
    let mut group = c.benchmark_group("algo_comparison/histogram");
    group.sample_size(10);

    for &n in &sizes {
        let (source, target) = make_similar(n);

        group.bench_with_input(BenchmarkId::new("histogram", n), &n, |b, _| {
            let algo = HistogramDiff::<String>::new();
            b.iter(|| {
                let changes = algo.diff(black_box(&source), black_box(&target));
                let patch = Patch::generate(&source, &target, black_box(&changes), false);
                black_box(patch.deltas().len())
            })
        });

        let (source_path, target_path) = make_pathological(n);
        group.bench_with_input(BenchmarkId::new("histogram_pathological", n), &n, |b, _| {
            let algo = HistogramDiff::<String>::new();
            b.iter(|| {
                let changes = algo.diff(black_box(&source_path), black_box(&target_path));
                let patch = Patch::generate(&source_path, &target_path, black_box(&changes), false);
                black_box(patch.deltas().len())
            })
        });
    }
    group.finish();
}

fn bench_histogram_clustered(c: &mut Criterion) {
    let sizes = [1_000, 5_000, 10_000];
    let mut group = c.benchmark_group("algo_comparison/clustered");
    group.sample_size(10);

    for &n in &sizes {
        let (source, target) = make_clustered(n);

        group.bench_with_input(BenchmarkId::new("histogram", n), &n, |b, _| {
            let algo = HistogramDiff::<String>::new();
            b.iter(|| black_box(algo.diff(black_box(&source), black_box(&target)).len()))
        });

        group.bench_with_input(BenchmarkId::new("myers_linear", n), &n, |b, _| {
            let algo = MyersDiffWithLinearSpace::<String>::default();
            b.iter(|| black_box(algo.diff(black_box(&source), black_box(&target)).len()))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_public_api_pathological,
    bench_public_api_similar,
    bench_algo_comparison_pathological,
    bench_algo_comparison_similar,
    bench_histogram_comparison,
    bench_histogram_clustered,
);
criterion_main!(benches);
