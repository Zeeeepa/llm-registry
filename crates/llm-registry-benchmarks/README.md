# LLM Registry Benchmarks

A canonical benchmark suite for the LLM Registry project, providing standardized performance testing across all registry operations.

## Features

- **Unified Interface**: Standard `BenchTarget` trait for implementing custom benchmarks
- **Structured Results**: Rich result types with metrics, metadata, and timestamps
- **Multiple Output Formats**: JSON, CSV, and Markdown reports
- **Comparison Tools**: Compare benchmark runs to track performance changes
- **Registry Adapters**: Pre-built benchmarks for common Registry operations
- **CLI Tool**: Command-line interface for running and analyzing benchmarks

## Quick Start

### Running Benchmarks

```bash
# Run all benchmarks with pretty JSON output
cargo run --bin bench-runner -- run --format json-pretty

# Run with markdown report
cargo run --bin bench-runner -- run --markdown

# Run and save raw results (one file per benchmark)
cargo run --bin bench-runner -- run --raw
```

## License

Apache-2.0 OR MIT
