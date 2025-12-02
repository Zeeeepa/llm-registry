//! Benchmark Result Types
//!
//! Core data structures for representing benchmark results with metrics and metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the result of a single benchmark execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Unique identifier for the benchmark target
    pub target_id: String,

    /// Metrics collected during benchmark execution
    pub metrics: BenchmarkMetrics,

    /// Timestamp when the benchmark was executed
    pub timestamp: DateTime<Utc>,

    /// Optional metadata about the benchmark environment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BenchmarkMetadata>,

    /// Status of the benchmark execution
    pub status: BenchmarkStatus,

    /// Optional error message if benchmark failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Performance metrics collected during benchmark execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Duration of the operation in milliseconds
    pub duration_ms: f64,

    /// Throughput in operations per second (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput_ops_per_sec: Option<f64>,

    /// Memory usage in bytes (if measured)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,

    /// Number of successful operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_count: Option<u64>,

    /// Number of failed operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_count: Option<u64>,

    /// Additional custom metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, f64>>,
}

/// Metadata about the benchmark execution environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    /// Rust version used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,

    /// Target triple (e.g., x86_64-unknown-linux-gnu)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_triple: Option<String>,

    /// CPU model information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_model: Option<String>,

    /// Number of CPU cores
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_cores: Option<usize>,

    /// Total system memory in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_memory: Option<u64>,

    /// Operating system information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_info: Option<String>,

    /// Git commit hash (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,

    /// Additional custom metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, String>>,
}

/// Status of benchmark execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BenchmarkStatus {
    /// Benchmark completed successfully
    Success,
    /// Benchmark failed
    Failed,
    /// Benchmark was skipped
    Skipped,
}

impl BenchmarkResult {
    /// Create a new successful benchmark result
    pub fn success(target_id: impl Into<String>, metrics: BenchmarkMetrics) -> Self {
        Self {
            target_id: target_id.into(),
            metrics,
            timestamp: Utc::now(),
            metadata: None,
            status: BenchmarkStatus::Success,
            error: None,
        }
    }

    /// Create a new failed benchmark result
    pub fn failed(target_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            target_id: target_id.into(),
            metrics: BenchmarkMetrics {
                duration_ms: 0.0,
                throughput_ops_per_sec: None,
                memory_bytes: None,
                success_count: None,
                error_count: Some(1),
                custom: None,
            },
            timestamp: Utc::now(),
            metadata: None,
            status: BenchmarkStatus::Failed,
            error: Some(error.into()),
        }
    }

    /// Create a skipped benchmark result
    pub fn skipped(target_id: impl Into<String>) -> Self {
        Self {
            target_id: target_id.into(),
            metrics: BenchmarkMetrics {
                duration_ms: 0.0,
                throughput_ops_per_sec: None,
                memory_bytes: None,
                success_count: None,
                error_count: None,
                custom: None,
            },
            timestamp: Utc::now(),
            metadata: None,
            status: BenchmarkStatus::Skipped,
            error: None,
        }
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, metadata: BenchmarkMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Check if the benchmark was successful
    pub fn is_success(&self) -> bool {
        self.status == BenchmarkStatus::Success
    }

    /// Check if the benchmark failed
    pub fn is_failed(&self) -> bool {
        self.status == BenchmarkStatus::Failed
    }
}

impl BenchmarkMetrics {
    /// Create new metrics with just duration
    pub fn new(duration_ms: f64) -> Self {
        Self {
            duration_ms,
            throughput_ops_per_sec: None,
            memory_bytes: None,
            success_count: None,
            error_count: None,
            custom: None,
        }
    }

    /// Add throughput measurement
    pub fn with_throughput(mut self, ops_per_sec: f64) -> Self {
        self.throughput_ops_per_sec = Some(ops_per_sec);
        self
    }

    /// Add memory usage measurement
    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    /// Add operation counts
    pub fn with_counts(mut self, success: u64, errors: u64) -> Self {
        self.success_count = Some(success);
        self.error_count = Some(errors);
        self
    }

    /// Add custom metric
    pub fn with_custom_metric(mut self, key: impl Into<String>, value: f64) -> Self {
        self.custom
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value);
        self
    }
}

impl BenchmarkMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self {
            rust_version: None,
            target_triple: None,
            cpu_model: None,
            cpu_cores: None,
            total_memory: None,
            os_info: None,
            git_commit: None,
            custom: None,
        }
    }

    /// Collect system metadata automatically
    pub fn collect() -> Self {
        Self {
            rust_version: Some(env!("CARGO_PKG_RUST_VERSION").to_string()),
            target_triple: Some(env!("TARGET").to_string()),
            cpu_cores: Some(num_cpus::get()),
            os_info: Some(format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)),
            ..Default::default()
        }
    }
}

impl Default for BenchmarkMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_result() {
        let metrics = BenchmarkMetrics::new(123.45);
        let result = BenchmarkResult::success("test_benchmark", metrics);

        assert!(result.is_success());
        assert!(!result.is_failed());
        assert_eq!(result.target_id, "test_benchmark");
        assert_eq!(result.metrics.duration_ms, 123.45);
    }

    #[test]
    fn test_failed_result() {
        let result = BenchmarkResult::failed("test_benchmark", "Something went wrong");

        assert!(result.is_failed());
        assert!(!result.is_success());
        assert_eq!(result.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_metrics_builder() {
        let metrics = BenchmarkMetrics::new(100.0)
            .with_throughput(1000.0)
            .with_memory(1024 * 1024)
            .with_counts(50, 2)
            .with_custom_metric("latency_p99", 250.0);

        assert_eq!(metrics.duration_ms, 100.0);
        assert_eq!(metrics.throughput_ops_per_sec, Some(1000.0));
        assert_eq!(metrics.memory_bytes, Some(1024 * 1024));
        assert_eq!(metrics.success_count, Some(50));
        assert_eq!(metrics.error_count, Some(2));
        assert_eq!(metrics.custom.as_ref().unwrap().get("latency_p99"), Some(&250.0));
    }

    #[test]
    fn test_serialization() {
        let metrics = BenchmarkMetrics::new(100.0);
        let result = BenchmarkResult::success("test", metrics);

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: BenchmarkResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.target_id, deserialized.target_id);
        assert_eq!(result.status, deserialized.status);
    }
}
