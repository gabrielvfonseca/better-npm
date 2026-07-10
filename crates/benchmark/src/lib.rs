//! Benchmarking harness using Criterion for Dash subsystems.
//!
//! Provides a simple API for running benchmarks and collecting
//! performance statistics on sync and async operations.

use std::time::{Duration, Instant};

/// Error type for benchmark operations.
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    #[error("benchmark error: {0}")]
    Benchmark(String),
    #[error("no iterations completed")]
    NoIterations,
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Results of a single benchmark run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkResult {
    /// Name of the benchmark.
    pub name: String,
    /// Number of iterations.
    pub iterations: u32,
    /// Total wall-clock duration.
    pub total_duration: Duration,
    /// Average duration per iteration.
    pub avg_duration: Duration,
    /// Minimum observed duration.
    pub min_duration: Duration,
    /// Maximum observed duration.
    pub max_duration: Duration,
    /// Operations per second.
    pub ops_per_sec: f64,
    /// 50th percentile (median).
    pub percentile_50: Duration,
    /// 95th percentile.
    pub percentile_95: Duration,
    /// 99th percentile.
    pub percentile_99: Duration,
    /// All individual durations (for further analysis).
    #[serde(skip)]
    pub all_durations: Vec<Duration>,
}

impl BenchmarkResult {
    /// Summarize the result as a string.
    pub fn summary(&self) -> String {
        format!(
            "{:<40} {:>8} iters  {:>8.2} ops/s  avg={:>8?}  min={:>6?}  max={:>6?}  p50={:>6?}  p95={:>6?}  p99={:>6?}",
            self.name,
            self.iterations,
            self.ops_per_sec,
            self.avg_duration,
            self.min_duration,
            self.max_duration,
            self.percentile_50,
            self.percentile_95,
            self.percentile_99,
        )
    }

    /// Format as CSV.
    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{:.2},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            self.name,
            self.iterations,
            self.ops_per_sec,
            self.avg_duration.as_secs_f64(),
            self.min_duration.as_secs_f64(),
            self.max_duration.as_secs_f64(),
            self.percentile_50.as_secs_f64(),
            self.percentile_95.as_secs_f64(),
            self.percentile_99.as_secs_f64(),
        )
    }
}

/// Run a synchronous benchmark.
///
/// Executes `f` `iterations` times and collects timing statistics.
pub fn benchmark<F>(name: &str, f: F, iterations: u32) -> BenchmarkResult
where
    F: Fn(),
{
    // Warmup
    for _ in 0..iterations.min(10) {
        f();
    }

    let mut durations = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let start = Instant::now();
        f();
        durations.push(start.elapsed());
    }

    compute_result(name, durations)
}

/// Run an asynchronous benchmark.
pub async fn benchmark_async<F, Fut>(name: &str, f: F, iterations: u32) -> BenchmarkResult
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    // Warmup
    for _ in 0..iterations.min(10) {
        f().await;
    }

    let mut durations = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let start = Instant::now();
        f().await;
        durations.push(start.elapsed());
    }

    compute_result(name, durations)
}

/// Run a benchmark with parameterized input.
pub fn benchmark_with_input<T, F>(
    name: &str,
    inputs: &[T],
    f: F,
) -> Vec<BenchmarkResult>
where
    T: Clone + std::fmt::Debug,
    F: Fn(T),
{
    inputs
        .iter()
        .map(|input| {
            let input_name = format!("{}/{:?}", name, input);
            benchmark(&input_name, || f(input.clone()), 100)
        })
        .collect()
}

/// Run a group of benchmarks and return all results.
pub fn benchmark_group(name: &str, benchmarks: Vec<(&str, Box<dyn Fn()>)>) -> Vec<BenchmarkResult> {
    benchmarks
        .into_iter()
        .map(|(bench_name, f)| {
            let full_name = format!("{name}/{bench_name}");
            benchmark(&full_name, f, 100)
        })
        .collect()
}

fn compute_result(name: &str, mut durations: Vec<Duration>) -> BenchmarkResult {
    let count = durations.len() as u32;

    if count == 0 {
        return BenchmarkResult {
            name: name.to_string(),
            iterations: 0,
            total_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            min_duration: Duration::ZERO,
            max_duration: Duration::ZERO,
            ops_per_sec: 0.0,
            percentile_50: Duration::ZERO,
            percentile_95: Duration::ZERO,
            percentile_99: Duration::ZERO,
            all_durations: Vec::new(),
        };
    }

    durations.sort();

    let total: Duration = durations.iter().sum();
    let avg = total / count;
    let min = durations[0];
    let max = durations[durations.len() - 1];

    let ops_per_sec = if avg.as_secs_f64() > 0.0 {
        1.0 / avg.as_secs_f64()
    } else {
        0.0
    };

    let percentile = |p: f64| -> Duration {
        if durations.is_empty() {
            return Duration::ZERO;
        }
        let idx = ((durations.len() as f64) * p / 100.0).ceil() as usize;
        let idx = idx.max(1).min(durations.len()) - 1;
        durations[idx]
    };

    BenchmarkResult {
        name: name.to_string(),
        iterations: count,
        total_duration: total,
        avg_duration: avg,
        min_duration: min,
        max_duration: max,
        ops_per_sec,
        percentile_50: percentile(50.0),
        percentile_95: percentile(95.0),
        percentile_99: percentile(99.0),
        all_durations: durations,
    }
}

/// Compare two benchmark results and compute the speedup.
pub fn compare_results(before: &BenchmarkResult, after: &BenchmarkResult) -> String {
    let speedup = before.avg_duration.as_secs_f64() / after.avg_duration.as_secs_f64();
    format!(
        "{}: {:.2}x speedup ({:?} → {:?})",
        before.name, speedup, before.avg_duration, after.avg_duration
    )
}

/// Format a duration for display.
pub fn format_duration(d: Duration) -> String {
    if d.as_secs() > 0 {
        format!("{:.3}s", d.as_secs_f64())
    } else if d.as_millis() > 0 {
        format!("{}ms", d.as_millis())
    } else if d.as_micros() > 0 {
        format!("{}µs", d.as_micros())
    } else {
        format!("{}ns", d.as_nanos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_sync() {
        let result = benchmark("test_sync", || {
            let mut x = 0u64;
            for i in 0..1000 {
                x += i;
            }
            std::hint::black_box(x);
        }, 50);

        assert_eq!(result.iterations, 50);
        assert!(result.avg_duration.as_nanos() > 0);
        assert!(result.ops_per_sec > 0.0);
        assert!(!result.summary().is_empty());
    }

    #[tokio::test]
    async fn test_benchmark_async() {
        let result = benchmark_async("test_async", || async {
            tokio::time::sleep(Duration::from_micros(10)).await;
        }, 10).await;

        assert_eq!(result.iterations, 10);
        assert!(result.ops_per_sec > 0.0);
    }

    #[test]
    fn test_benchmark_result_summary() {
        let result = BenchmarkResult {
            name: "test".to_string(),
            iterations: 100,
            total_duration: Duration::from_secs(1),
            avg_duration: Duration::from_millis(10),
            min_duration: Duration::from_millis(1),
            max_duration: Duration::from_millis(100),
            ops_per_sec: 100.0,
            percentile_50: Duration::from_millis(5),
            percentile_95: Duration::from_millis(50),
            percentile_99: Duration::from_millis(90),
            all_durations: Vec::new(),
        };

        let summary = result.summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("100 iters"));
    }

    #[test]
    fn test_benchmark_to_csv() {
        let result = BenchmarkResult {
            name: "csv_test".to_string(),
            iterations: 50,
            total_duration: Duration::from_secs(1),
            avg_duration: Duration::from_millis(20),
            min_duration: Duration::from_millis(10),
            max_duration: Duration::from_millis(30),
            ops_per_sec: 50.0,
            percentile_50: Duration::from_millis(20),
            percentile_95: Duration::from_millis(25),
            percentile_99: Duration::from_millis(28),
            all_durations: Vec::new(),
        };

        let csv = result.to_csv();
        assert!(csv.starts_with("csv_test"));
    }

    #[test]
    fn test_benchmark_group() {
        let results = benchmark_group("group", vec![
            ("fast", Box::new(|| { std::hint::black_box(1 + 1); })),
            ("slow", Box::new(|| {
                let _: Vec<u64> = (0..1000).collect();
            })),
        ]);

        assert_eq!(results.len(), 2);
        assert!(results[0].name.contains("fast"));
        assert!(results[1].name.contains("slow"));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_nanos(500)), "500ns");
        assert_eq!(format_duration(Duration::from_micros(1)), "1µs");
        assert_eq!(format_duration(Duration::from_millis(100)), "100ms");
        assert_eq!(format_duration(Duration::from_secs(2)), "2.000s");
    }

    #[test]
    fn test_compare_results() {
        let before = BenchmarkResult {
            name: "test".to_string(),
            iterations: 10,
            total_duration: Duration::from_millis(100),
            avg_duration: Duration::from_millis(10),
            min_duration: Duration::from_millis(5),
            max_duration: Duration::from_millis(15),
            ops_per_sec: 100.0,
            percentile_50: Duration::from_millis(10),
            percentile_95: Duration::from_millis(12),
            percentile_99: Duration::from_millis(14),
            all_durations: Vec::new(),
        };

        let after = BenchmarkResult {
            name: "test".to_string(),
            iterations: 10,
            total_duration: Duration::from_millis(50),
            avg_duration: Duration::from_millis(5),
            min_duration: Duration::from_millis(3),
            max_duration: Duration::from_millis(8),
            ops_per_sec: 200.0,
            percentile_50: Duration::from_millis(5),
            percentile_95: Duration::from_millis(7),
            percentile_99: Duration::from_millis(8),
            all_durations: Vec::new(),
        };

        let comparison = compare_results(&before, &after);
        assert!(comparison.contains("2.00x speedup"));
    }

    #[test]
    fn test_benchmark_empty() {
        let result = compute_result("empty", vec![]);
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn test_benchmark_with_input() {
        let results = benchmark_with_input("square", &[1, 2, 5, 10], |n| {
            std::hint::black_box(n * n);
        });
        assert_eq!(results.len(), 4);
    }
}
