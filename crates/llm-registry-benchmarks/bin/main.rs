//! Benchmark Runner Binary
//!
//! Command-line tool for running the LLM Registry benchmark suite.

use anyhow::Result;
use clap::{Parser, Subcommand};
use llm_registry_benchmarks::{
    run_all_benchmarks, save_results, save_raw_results, load_results,
    compare_results, generate_report, generate_comparison_report,
    OutputFormat, DEFAULT_OUTPUT_DIR,
};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// LLM Registry Benchmark Runner
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run all benchmarks
    Run {
        /// Output format (json, json-pretty, csv)
        #[arg(short, long, default_value = "json-pretty")]
        format: String,

        /// Save raw results (one file per benchmark)
        #[arg(short, long)]
        raw: bool,

        /// Generate markdown report
        #[arg(short, long)]
        markdown: bool,
    },

    /// Compare two benchmark result files
    Compare {
        /// Baseline results file
        baseline: PathBuf,

        /// Current results file
        current: PathBuf,

        /// Generate markdown comparison report
        #[arg(short, long)]
        markdown: bool,
    },

    /// Generate a markdown report from results file
    Report {
        /// Results file to generate report from
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let log_level = if args.verbose { "debug" } else { "info" };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    match args.command {
        Commands::Run {
            format,
            raw,
            markdown,
        } => {
            tracing::info!("Running benchmark suite...");

            // Run benchmarks
            let results = run_all_benchmarks().await;

            // Determine output format
            let output_format = match format.as_str() {
                "json" => OutputFormat::Json,
                "json-pretty" => OutputFormat::JsonPretty,
                "csv" => OutputFormat::Csv,
                _ => {
                    tracing::warn!("Unknown format '{}', using json-pretty", format);
                    OutputFormat::JsonPretty
                }
            };

            // Save main results
            let output_path = save_results(&results, output_format)?;
            println!("Results saved to: {}", output_path.display());

            // Save raw results if requested
            if raw {
                let raw_paths = save_raw_results(&results)?;
                println!("Raw results saved: {} files", raw_paths.len());
            }

            // Generate markdown report if requested
            if markdown {
                let report = generate_report(&results);
                let report_path = PathBuf::from(DEFAULT_OUTPUT_DIR)
                    .join("benchmark_report.md");
                std::fs::write(&report_path, report)?;
                println!("Markdown report saved to: {}", report_path.display());
            }

            // Print summary
            println!("\n{}", generate_summary(&results));
        }

        Commands::Compare {
            baseline,
            current,
            markdown,
        } => {
            tracing::info!("Comparing benchmark results...");

            // Load results
            let baseline_results = load_results(&baseline)?;
            let current_results = load_results(&current)?;

            // Compare
            let summary = compare_results(&baseline_results, &current_results);

            // Print comparison
            println!("\n=== Benchmark Comparison ===\n");
            println!("Baseline: {}", baseline.display());
            println!("Current:  {}\n", current.display());

            println!("Total benchmarks: {}", summary.comparisons.len());
            println!("Improvements: {}", summary.improvements().len());
            println!("Regressions:  {}", summary.regressions().len());
            println!("Unchanged:    {}\n", summary.unchanged().len());

            // Show top regressions
            let mut regressions = summary.regressions();
            regressions.sort_by(|a, b| {
                b.duration_change_pct
                    .partial_cmp(&a.duration_change_pct)
                    .unwrap()
            });

            if !regressions.is_empty() {
                println!("Top Regressions:");
                for (i, comp) in regressions.iter().take(5).enumerate() {
                    println!(
                        "  {}. {} ({:+.1}%): {:.2}ms → {:.2}ms",
                        i + 1,
                        comp.target_id,
                        comp.duration_change_pct,
                        comp.baseline_duration_ms,
                        comp.current_duration_ms
                    );
                }
                println!();
            }

            // Show top improvements
            let mut improvements = summary.improvements();
            improvements.sort_by(|a, b| {
                a.duration_change_pct
                    .partial_cmp(&b.duration_change_pct)
                    .unwrap()
            });

            if !improvements.is_empty() {
                println!("Top Improvements:");
                for (i, comp) in improvements.iter().take(5).enumerate() {
                    println!(
                        "  {}. {} ({:+.1}%): {:.2}ms → {:.2}ms",
                        i + 1,
                        comp.target_id,
                        comp.duration_change_pct,
                        comp.baseline_duration_ms,
                        comp.current_duration_ms
                    );
                }
                println!();
            }

            // Generate markdown report if requested
            if markdown {
                let report = generate_comparison_report(
                    &summary,
                    &baseline.display().to_string(),
                    &current.display().to_string(),
                );
                let report_path = PathBuf::from(DEFAULT_OUTPUT_DIR)
                    .join("comparison_report.md");
                std::fs::write(&report_path, report)?;
                println!("Markdown comparison report saved to: {}", report_path.display());
            }
        }

        Commands::Report { file } => {
            tracing::info!("Generating report from: {}", file.display());

            // Load results
            let results = load_results(&file)?;

            // Generate report
            let report = generate_report(&results);

            // Print to stdout
            println!("{}", report);
        }
    }

    Ok(())
}

/// Generate a summary of benchmark results
fn generate_summary(results: &[llm_registry_benchmarks::BenchmarkResult]) -> String {
    use llm_registry_benchmarks::BenchmarkStatus;

    let total = results.len();
    let successful = results.iter().filter(|r| r.status == BenchmarkStatus::Success).count();
    let failed = results.iter().filter(|r| r.status == BenchmarkStatus::Failed).count();

    let mut summary = format!("=== Benchmark Summary ===\n\n");
    summary.push_str(&format!("Total:      {}\n", total));
    summary.push_str(&format!("Successful: {} ✓\n", successful));

    if failed > 0 {
        summary.push_str(&format!("Failed:     {} ✗\n", failed));
    }

    if successful > 0 {
        let avg_duration: f64 = results
            .iter()
            .filter(|r| r.status == BenchmarkStatus::Success)
            .map(|r| r.metrics.duration_ms)
            .sum::<f64>()
            / successful as f64;

        summary.push_str(&format!("\nAverage Duration: {:.2}ms\n", avg_duration));
    }

    summary
}
