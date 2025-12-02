//! I/O utilities for benchmark results
//!
//! Provides functions for saving and loading benchmark results in various formats.

use crate::result::BenchmarkResult;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

/// Default output directory for benchmark results
pub const DEFAULT_OUTPUT_DIR: &str = "benchmarks/output";

/// Default directory for raw JSON results
pub const DEFAULT_RAW_DIR: &str = "benchmarks/output/raw";

/// Output format for benchmark results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON format (default)
    Json,
    /// Pretty-printed JSON
    JsonPretty,
    /// CSV format
    Csv,
}

/// Save benchmark results to a file
///
/// The filename will be automatically generated with a timestamp.
/// Format: `benchmark_results_YYYYMMDD_HHMMSS.{ext}`
pub fn save_results(
    results: &[BenchmarkResult],
    format: OutputFormat,
) -> Result<PathBuf> {
    let output_dir = Path::new(DEFAULT_OUTPUT_DIR);
    fs::create_dir_all(output_dir)
        .context("Failed to create output directory")?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let extension = match format {
        OutputFormat::Json | OutputFormat::JsonPretty => "json",
        OutputFormat::Csv => "csv",
    };

    let filename = format!("benchmark_results_{}.{}", timestamp, extension);
    let filepath = output_dir.join(filename);

    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string(&results)
                .context("Failed to serialize results to JSON")?;
            fs::write(&filepath, json)
                .context("Failed to write JSON file")?;
        }
        OutputFormat::JsonPretty => {
            let json = serde_json::to_string_pretty(&results)
                .context("Failed to serialize results to pretty JSON")?;
            fs::write(&filepath, json)
                .context("Failed to write JSON file")?;
        }
        OutputFormat::Csv => {
            save_results_csv(results, &filepath)?;
        }
    }

    Ok(filepath)
}

/// Save raw JSON results (one per benchmark)
///
/// Each benchmark result is saved as a separate JSON file for easier analysis.
pub fn save_raw_results(results: &[BenchmarkResult]) -> Result<Vec<PathBuf>> {
    let raw_dir = Path::new(DEFAULT_RAW_DIR);
    fs::create_dir_all(raw_dir)
        .context("Failed to create raw output directory")?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let mut paths = Vec::new();

    for result in results {
        // Sanitize target_id for filename
        let sanitized_id = result.target_id.replace(['/', '\\', ':'], "_");
        let filename = format!("{}_{}.json", sanitized_id, timestamp);
        let filepath = raw_dir.join(filename);

        let json = serde_json::to_string_pretty(&result)
            .context("Failed to serialize result to JSON")?;

        fs::write(&filepath, json)
            .context("Failed to write raw JSON file")?;

        paths.push(filepath);
    }

    Ok(paths)
}

/// Load benchmark results from a JSON file
pub fn load_results(filepath: impl AsRef<Path>) -> Result<Vec<BenchmarkResult>> {
    let content = fs::read_to_string(filepath.as_ref())
        .context("Failed to read results file")?;

    let results: Vec<BenchmarkResult> = serde_json::from_str(&content)
        .context("Failed to deserialize results from JSON")?;

    Ok(results)
}

/// Save results in CSV format
fn save_results_csv(results: &[BenchmarkResult], filepath: &Path) -> Result<()> {
    use std::io::Write;

    let mut file = fs::File::create(filepath)
        .context("Failed to create CSV file")?;

    // Write header
    writeln!(
        file,
        "target_id,status,duration_ms,throughput_ops_per_sec,memory_bytes,success_count,error_count,timestamp,error"
    )?;

    // Write data rows
    for result in results {
        writeln!(
            file,
            "{},{:?},{},{},{},{},{},{},{}",
            result.target_id,
            result.status,
            result.metrics.duration_ms,
            result.metrics.throughput_ops_per_sec.map(|v| v.to_string()).unwrap_or_default(),
            result.metrics.memory_bytes.map(|v| v.to_string()).unwrap_or_default(),
            result.metrics.success_count.map(|v| v.to_string()).unwrap_or_default(),
            result.metrics.error_count.map(|v| v.to_string()).unwrap_or_default(),
            result.timestamp.to_rfc3339(),
            result.error.as_deref().unwrap_or("")
        )?;
    }

    Ok(())
}

/// List all available result files in the output directory
pub fn list_result_files() -> Result<Vec<PathBuf>> {
    let output_dir = Path::new(DEFAULT_OUTPUT_DIR);

    if !output_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    files.push(path);
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Compare two sets of benchmark results
///
/// Returns a summary of performance changes between baseline and current results.
pub fn compare_results(
    baseline: &[BenchmarkResult],
    current: &[BenchmarkResult],
) -> ComparisonSummary {
    use std::collections::HashMap;

    let baseline_map: HashMap<_, _> = baseline
        .iter()
        .map(|r| (r.target_id.clone(), r))
        .collect();

    let mut comparisons = Vec::new();

    for result in current {
        if let Some(base) = baseline_map.get(&result.target_id) {
            let duration_change_pct = if base.metrics.duration_ms > 0.0 {
                ((result.metrics.duration_ms - base.metrics.duration_ms) / base.metrics.duration_ms) * 100.0
            } else {
                0.0
            };

            comparisons.push(BenchmarkComparison {
                target_id: result.target_id.clone(),
                baseline_duration_ms: base.metrics.duration_ms,
                current_duration_ms: result.metrics.duration_ms,
                duration_change_pct,
            });
        }
    }

    ComparisonSummary { comparisons }
}

/// Summary of performance comparison
#[derive(Debug)]
pub struct ComparisonSummary {
    pub comparisons: Vec<BenchmarkComparison>,
}

/// Comparison of a single benchmark
#[derive(Debug)]
pub struct BenchmarkComparison {
    pub target_id: String,
    pub baseline_duration_ms: f64,
    pub current_duration_ms: f64,
    pub duration_change_pct: f64,
}

impl ComparisonSummary {
    /// Get benchmarks that improved (faster)
    pub fn improvements(&self) -> Vec<&BenchmarkComparison> {
        self.comparisons
            .iter()
            .filter(|c| c.duration_change_pct < -5.0) // More than 5% faster
            .collect()
    }

    /// Get benchmarks that regressed (slower)
    pub fn regressions(&self) -> Vec<&BenchmarkComparison> {
        self.comparisons
            .iter()
            .filter(|c| c.duration_change_pct > 5.0) // More than 5% slower
            .collect()
    }

    /// Get benchmarks that stayed roughly the same
    pub fn unchanged(&self) -> Vec<&BenchmarkComparison> {
        self.comparisons
            .iter()
            .filter(|c| c.duration_change_pct.abs() <= 5.0)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::{BenchmarkMetrics, BenchmarkStatus};

    fn create_test_result(id: &str, duration: f64) -> BenchmarkResult {
        BenchmarkResult {
            target_id: id.to_string(),
            metrics: BenchmarkMetrics::new(duration),
            timestamp: Utc::now(),
            metadata: None,
            status: BenchmarkStatus::Success,
            error: None,
        }
    }

    #[test]
    fn test_compare_results() {
        let baseline = vec![
            create_test_result("test1", 100.0),
            create_test_result("test2", 200.0),
        ];

        let current = vec![
            create_test_result("test1", 80.0),  // 20% improvement
            create_test_result("test2", 220.0), // 10% regression
        ];

        let summary = compare_results(&baseline, &current);

        assert_eq!(summary.comparisons.len(), 2);
        assert_eq!(summary.improvements().len(), 1);
        assert_eq!(summary.regressions().len(), 1);
    }

    #[test]
    fn test_output_format() {
        let results = vec![create_test_result("test", 100.0)];

        // Test JSON serialization
        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("100"));
    }
}
