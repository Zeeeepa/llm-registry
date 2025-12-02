//! LLM Registry Benchmarks
//!
//! A canonical benchmark suite for the LLM Registry project.
//!
//! This library provides:
//! - Standard benchmark interface via the `BenchTarget` trait
//! - Structured result types with metrics and metadata
//! - Adapters for common Registry operations
//! - I/O utilities for saving results in multiple formats
//! - Markdown report generation
//! - Main entrypoint via `run_all_benchmarks()`
//!
//! # Example
//!
//! ```no_run
//! use llm_registry_benchmarks::run_all_benchmarks;
//!
//! #[tokio::main]
//! async fn main() {
//!     let results = run_all_benchmarks().await;
//!     println!("Ran {} benchmarks", results.len());
//! }
//! ```

#![warn(missing_docs)]

mod adapters;
mod io;
mod markdown;
mod result;

use async_trait::async_trait;
use std::time::Instant;

// Re-export public API
pub use result::{BenchmarkMetadata, BenchmarkMetrics, BenchmarkResult, BenchmarkStatus};
pub use io::{
    save_results, save_raw_results, load_results, list_result_files,
    compare_results, ComparisonSummary, BenchmarkComparison, OutputFormat,
    DEFAULT_OUTPUT_DIR, DEFAULT_RAW_DIR,
};
pub use markdown::{generate_report, generate_comparison_report, generate_pr_comment};

/// Trait for implementing benchmark targets
///
/// Each benchmark should implement this trait to be included in the
/// benchmark suite. The trait provides methods for identification and execution.
#[async_trait]
pub trait BenchTarget: Send + Sync {
    /// Unique identifier for this benchmark
    ///
    /// Should be a stable identifier that can be used to track results over time.
    /// Format: category.operation (e.g., "db.asset_create", "cache.lookup")
    fn id(&self) -> String;

    /// Human-readable description of what this benchmark measures
    fn description(&self) -> String {
        format!("Benchmark: {}", self.id())
    }

    /// Run the benchmark and return results
    ///
    /// This method should:
    /// 1. Perform setup (if needed)
    /// 2. Execute the operation being benchmarked
    /// 3. Measure performance metrics
    /// 4. Return a BenchmarkResult with collected metrics
    async fn run(&self) -> BenchmarkResult;

    /// Optional setup before running the benchmark
    ///
    /// This is useful for initializing test data or resources.
    /// The setup time is not included in benchmark measurements.
    async fn setup(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Optional cleanup after running the benchmark
    ///
    /// This is useful for removing test data or freeing resources.
    async fn teardown(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Execute the benchmark with full lifecycle (setup, run, teardown)
    async fn execute(&self) -> BenchmarkResult {
        // Setup
        if let Err(e) = self.setup().await {
            return BenchmarkResult::failed(self.id(), format!("Setup failed: {}", e));
        }

        // Run benchmark
        let result = self.run().await;

        // Teardown (always attempt, even if benchmark failed)
        if let Err(e) = self.teardown().await {
            tracing::warn!("Teardown failed for {}: {}", self.id(), e);
        }

        result
    }
}

/// Helper function to measure the duration of an async operation
pub async fn measure_async<F, T>(f: F) -> (T, f64)
where
    F: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = f.await;
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    (result, duration_ms)
}

/// Helper function to measure the duration of a sync operation
pub fn measure_sync<F, T>(f: F) -> (T, f64)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    (result, duration_ms)
}

/// Run all registered benchmarks and return results
///
/// This is the main entrypoint for the benchmark suite. It executes all
/// benchmarks and collects their results into a vector.
pub async fn run_all_benchmarks() -> Vec<BenchmarkResult> {
    tracing::info!("Starting benchmark suite");

    let mut results = Vec::new();

    // Get all benchmark targets
    let targets = adapters::create_default_benchmarks();

    for target in targets {
        tracing::info!("Running benchmark: {}", target.id());

        let result = target.execute().await;

        match result.status {
            BenchmarkStatus::Success => {
                tracing::info!(
                    "✓ {} completed in {:.2}ms",
                    result.target_id,
                    result.metrics.duration_ms
                );
            }
            BenchmarkStatus::Failed => {
                tracing::error!(
                    "✗ {} failed: {}",
                    result.target_id,
                    result.error.as_deref().unwrap_or("Unknown error")
                );
            }
            BenchmarkStatus::Skipped => {
                tracing::warn!("⊘ {} skipped", result.target_id);
            }
        }

        results.push(result);
    }

    tracing::info!("Benchmark suite completed: {} total", results.len());

    results
}
