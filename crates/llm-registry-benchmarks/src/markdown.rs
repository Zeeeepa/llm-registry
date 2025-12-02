//! Markdown report generation for benchmark results
//!
//! Provides functionality to generate human-readable markdown reports
//! from benchmark results for documentation and CI/CD integration.

use crate::result::{BenchmarkResult, BenchmarkStatus};
use crate::io::{ComparisonSummary, BenchmarkComparison};
use chrono::Utc;
use std::fmt::Write;

/// Generate a markdown report from benchmark results
pub fn generate_report(results: &[BenchmarkResult]) -> String {
    let mut report = String::new();

    // Header
    writeln!(&mut report, "# Benchmark Results\n").unwrap();
    writeln!(
        &mut report,
        "**Generated:** {}\n",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )
    .unwrap();

    // Summary statistics
    let total = results.len();
    let successful = results.iter().filter(|r| r.is_success()).count();
    let failed = results.iter().filter(|r| r.is_failed()).count();
    let skipped = total - successful - failed;

    writeln!(&mut report, "## Summary\n").unwrap();
    writeln!(&mut report, "- **Total Benchmarks:** {}", total).unwrap();
    writeln!(&mut report, "- **Successful:** {} ✓", successful).unwrap();

    if failed > 0 {
        writeln!(&mut report, "- **Failed:** {} ✗", failed).unwrap();
    }
    if skipped > 0 {
        writeln!(&mut report, "- **Skipped:** {} ⊘", skipped).unwrap();
    }

    writeln!(&mut report).unwrap();

    // Results table
    if !results.is_empty() {
        writeln!(&mut report, "## Results\n").unwrap();
        writeln!(
            &mut report,
            "| Benchmark | Status | Duration (ms) | Throughput (ops/sec) | Memory (MB) |"
        )
        .unwrap();
        writeln!(
            &mut report,
            "|-----------|--------|---------------|----------------------|-------------|"
        )
        .unwrap();

        for result in results {
            let status_icon = match result.status {
                BenchmarkStatus::Success => "✓",
                BenchmarkStatus::Failed => "✗",
                BenchmarkStatus::Skipped => "⊘",
            };

            let throughput = result
                .metrics
                .throughput_ops_per_sec
                .map(|t| format!("{:.2}", t))
                .unwrap_or_else(|| "-".to_string());

            let memory = result
                .metrics
                .memory_bytes
                .map(|m| format!("{:.2}", m as f64 / 1024.0 / 1024.0))
                .unwrap_or_else(|| "-".to_string());

            writeln!(
                &mut report,
                "| {} | {} | {:.2} | {} | {} |",
                result.target_id,
                status_icon,
                result.metrics.duration_ms,
                throughput,
                memory
            )
            .unwrap();
        }

        writeln!(&mut report).unwrap();
    }

    // Performance insights
    if successful > 0 {
        writeln!(&mut report, "## Performance Insights\n").unwrap();

        // Fastest and slowest benchmarks
        let mut successful_results: Vec<_> = results.iter().filter(|r| r.is_success()).collect();
        successful_results.sort_by(|a, b| {
            a.metrics
                .duration_ms
                .partial_cmp(&b.metrics.duration_ms)
                .unwrap()
        });

        if let Some(fastest) = successful_results.first() {
            writeln!(
                &mut report,
                "**Fastest:** {} ({:.2}ms)",
                fastest.target_id, fastest.metrics.duration_ms
            )
            .unwrap();
        }

        if let Some(slowest) = successful_results.last() {
            writeln!(
                &mut report,
                "**Slowest:** {} ({:.2}ms)",
                slowest.target_id, slowest.metrics.duration_ms
            )
            .unwrap();
        }

        // Average duration
        let avg_duration: f64 = successful_results
            .iter()
            .map(|r| r.metrics.duration_ms)
            .sum::<f64>()
            / successful_results.len() as f64;

        writeln!(&mut report, "**Average Duration:** {:.2}ms", avg_duration).unwrap();

        writeln!(&mut report).unwrap();
    }

    // Failed benchmarks details
    if failed > 0 {
        writeln!(&mut report, "## Failed Benchmarks\n").unwrap();

        for result in results.iter().filter(|r| r.is_failed()) {
            writeln!(&mut report, "### {}\n", result.target_id).unwrap();

            if let Some(error) = &result.error {
                writeln!(&mut report, "```").unwrap();
                writeln!(&mut report, "{}", error).unwrap();
                writeln!(&mut report, "```\n").unwrap();
            }
        }
    }

    // System metadata (if available)
    if let Some(result) = results.first() {
        if let Some(metadata) = &result.metadata {
            writeln!(&mut report, "## System Information\n").unwrap();

            if let Some(rust_ver) = &metadata.rust_version {
                writeln!(&mut report, "- **Rust Version:** {}", rust_ver).unwrap();
            }
            if let Some(os) = &metadata.os_info {
                writeln!(&mut report, "- **Operating System:** {}", os).unwrap();
            }
            if let Some(cpu) = &metadata.cpu_model {
                writeln!(&mut report, "- **CPU:** {}", cpu).unwrap();
            }
            if let Some(cores) = metadata.cpu_cores {
                writeln!(&mut report, "- **CPU Cores:** {}", cores).unwrap();
            }
            if let Some(mem) = metadata.total_memory {
                writeln!(
                    &mut report,
                    "- **Total Memory:** {:.2} GB",
                    mem as f64 / 1024.0 / 1024.0 / 1024.0
                )
                .unwrap();
            }

            writeln!(&mut report).unwrap();
        }
    }

    report
}

/// Generate a comparison report showing performance changes
pub fn generate_comparison_report(
    summary: &ComparisonSummary,
    baseline_name: &str,
    current_name: &str,
) -> String {
    let mut report = String::new();

    writeln!(&mut report, "# Benchmark Comparison Report\n").unwrap();
    writeln!(
        &mut report,
        "**Generated:** {}\n",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )
    .unwrap();

    writeln!(&mut report, "**Baseline:** {}", baseline_name).unwrap();
    writeln!(&mut report, "**Current:** {}\n", current_name).unwrap();

    // Summary
    let improvements = summary.improvements();
    let regressions = summary.regressions();
    let unchanged = summary.unchanged();

    writeln!(&mut report, "## Summary\n").unwrap();
    writeln!(
        &mut report,
        "- **Total Benchmarks:** {}",
        summary.comparisons.len()
    )
    .unwrap();
    writeln!(&mut report, "- **Improvements:** {} 🚀", improvements.len()).unwrap();
    writeln!(&mut report, "- **Regressions:** {} 🐌", regressions.len()).unwrap();
    writeln!(&mut report, "- **Unchanged:** {} ≈", unchanged.len()).unwrap();
    writeln!(&mut report).unwrap();

    // Detailed comparison table
    writeln!(&mut report, "## Detailed Comparison\n").unwrap();
    writeln!(
        &mut report,
        "| Benchmark | Baseline (ms) | Current (ms) | Change | % Change |"
    )
    .unwrap();
    writeln!(
        &mut report,
        "|-----------|---------------|--------------|--------|----------|"
    )
    .unwrap();

    let mut sorted_comparisons = summary.comparisons.clone();
    sorted_comparisons.sort_by(|a, b| {
        b.duration_change_pct
            .partial_cmp(&a.duration_change_pct)
            .unwrap()
    });

    for comp in sorted_comparisons {
        let change = comp.current_duration_ms - comp.baseline_duration_ms;
        let icon = if comp.duration_change_pct < -5.0 {
            "🚀"
        } else if comp.duration_change_pct > 5.0 {
            "🐌"
        } else {
            "≈"
        };

        writeln!(
            &mut report,
            "| {} | {:.2} | {:.2} | {:+.2} | {:+.1}% {} |",
            comp.target_id,
            comp.baseline_duration_ms,
            comp.current_duration_ms,
            change,
            comp.duration_change_pct,
            icon
        )
        .unwrap();
    }

    writeln!(&mut report).unwrap();

    // Highlight regressions
    if !regressions.is_empty() {
        writeln!(&mut report, "## 🐌 Performance Regressions\n").unwrap();

        for comp in regressions {
            writeln!(
                &mut report,
                "- **{}**: {:.2}ms → {:.2}ms ({:+.1}%)",
                comp.target_id,
                comp.baseline_duration_ms,
                comp.current_duration_ms,
                comp.duration_change_pct
            )
            .unwrap();
        }

        writeln!(&mut report).unwrap();
    }

    // Highlight improvements
    if !improvements.is_empty() {
        writeln!(&mut report, "## 🚀 Performance Improvements\n").unwrap();

        for comp in improvements {
            writeln!(
                &mut report,
                "- **{}**: {:.2}ms → {:.2}ms ({:+.1}%)",
                comp.target_id,
                comp.baseline_duration_ms,
                comp.current_duration_ms,
                comp.duration_change_pct
            )
            .unwrap();
        }

        writeln!(&mut report).unwrap();
    }

    report
}

/// Generate a GitHub-flavored markdown comment for PR integration
pub fn generate_pr_comment(summary: &ComparisonSummary) -> String {
    let mut comment = String::new();

    writeln!(&mut comment, "## 📊 Benchmark Results\n").unwrap();

    let improvements = summary.improvements();
    let regressions = summary.regressions();

    if regressions.is_empty() && improvements.is_empty() {
        writeln!(
            &mut comment,
            "✅ No significant performance changes detected."
        )
        .unwrap();
    } else {
        if !regressions.is_empty() {
            writeln!(&mut comment, "### 🐌 Performance Regressions\n").unwrap();
            for comp in regressions.iter().take(5) {
                // Limit to top 5
                writeln!(
                    &mut comment,
                    "- `{}`: {:+.1}% slower ({:.2}ms → {:.2}ms)",
                    comp.target_id,
                    comp.duration_change_pct,
                    comp.baseline_duration_ms,
                    comp.current_duration_ms
                )
                .unwrap();
            }
            writeln!(&mut comment).unwrap();
        }

        if !improvements.is_empty() {
            writeln!(&mut comment, "### 🚀 Performance Improvements\n").unwrap();
            for comp in improvements.iter().take(5) {
                // Limit to top 5
                writeln!(
                    &mut comment,
                    "- `{}`: {:.1}% faster ({:.2}ms → {:.2}ms)",
                    comp.target_id,
                    -comp.duration_change_pct,
                    comp.baseline_duration_ms,
                    comp.current_duration_ms
                )
                .unwrap();
            }
            writeln!(&mut comment).unwrap();
        }
    }

    writeln!(
        &mut comment,
        "<details><summary>View full benchmark results</summary>\n"
    )
    .unwrap();
    writeln!(&mut comment, "| Benchmark | Change |").unwrap();
    writeln!(&mut comment, "|-----------|--------|").unwrap();

    for comp in &summary.comparisons {
        writeln!(
            &mut comment,
            "| {} | {:+.1}% |",
            comp.target_id, comp.duration_change_pct
        )
        .unwrap();
    }

    writeln!(&mut comment, "\n</details>").unwrap();

    comment
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::{BenchmarkMetrics, BenchmarkMetadata};

    fn create_test_result(id: &str, duration: f64, status: BenchmarkStatus) -> BenchmarkResult {
        BenchmarkResult {
            target_id: id.to_string(),
            metrics: BenchmarkMetrics::new(duration),
            timestamp: Utc::now(),
            metadata: Some(BenchmarkMetadata::collect()),
            status,
            error: if status == BenchmarkStatus::Failed {
                Some("Test error".to_string())
            } else {
                None
            },
        }
    }

    #[test]
    fn test_generate_report() {
        let results = vec![
            create_test_result("test1", 100.0, BenchmarkStatus::Success),
            create_test_result("test2", 200.0, BenchmarkStatus::Success),
            create_test_result("test3", 150.0, BenchmarkStatus::Failed),
        ];

        let report = generate_report(&results);

        assert!(report.contains("# Benchmark Results"));
        assert!(report.contains("Total Benchmarks: 3"));
        assert!(report.contains("Successful: 2"));
        assert!(report.contains("Failed: 1"));
        assert!(report.contains("test1"));
        assert!(report.contains("test2"));
        assert!(report.contains("test3"));
    }

    #[test]
    fn test_generate_comparison_report() {
        use crate::io::BenchmarkComparison;

        let comparisons = vec![
            BenchmarkComparison {
                target_id: "test1".to_string(),
                baseline_duration_ms: 100.0,
                current_duration_ms: 80.0,
                duration_change_pct: -20.0,
            },
            BenchmarkComparison {
                target_id: "test2".to_string(),
                baseline_duration_ms: 100.0,
                current_duration_ms: 120.0,
                duration_change_pct: 20.0,
            },
        ];

        let summary = ComparisonSummary { comparisons };
        let report = generate_comparison_report(&summary, "baseline", "current");

        assert!(report.contains("# Benchmark Comparison Report"));
        assert!(report.contains("Improvements: 1"));
        assert!(report.contains("Regressions: 1"));
    }

    #[test]
    fn test_generate_pr_comment() {
        use crate::io::BenchmarkComparison;

        let comparisons = vec![BenchmarkComparison {
            target_id: "test".to_string(),
            baseline_duration_ms: 100.0,
            current_duration_ms: 90.0,
            duration_change_pct: -10.0,
        }];

        let summary = ComparisonSummary { comparisons };
        let comment = generate_pr_comment(&summary);

        assert!(comment.contains("📊 Benchmark Results"));
        assert!(comment.contains("🚀 Performance Improvements"));
    }
}
