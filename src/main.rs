//! SDIFF command-line interface.
//!
//! This is the main entry point for the sdiff CLI tool. It uses clap for
//! argument parsing and wires together all the library modules to perform
//! semantic diffs on structured data files.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use sdiff::{
    compute_diff, format_diff, parse_file, ArrayDiffStrategy, DiffConfig, OutputFormat,
    OutputOptions,
};
use std::path::PathBuf;
use std::process;

/// SDIFF - Semantic diff tool for structured data
///
/// Intelligently compares JSON and YAML files, showing only meaningful changes
/// while ignoring formatting, whitespace, and key ordering differences.
#[derive(Parser)]
#[command(name = "sdiff")]
#[command(version)]
#[command(about = "Semantic diff tool for structured data", long_about = None)]
#[command(author = "SDIFF Contributors")]
struct Cli {
    /// First file to compare
    #[arg(value_name = "FILE1")]
    file1: PathBuf,

    /// Second file to compare
    #[arg(value_name = "FILE2")]
    file2: PathBuf,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "terminal")]
    format: OutputFormatArg,

    /// Show only changes (hide unchanged fields)
    #[arg(short, long, default_value = "true")]
    compact: bool,

    /// Show full values instead of previews
    #[arg(long)]
    show_values: bool,

    /// Maximum length for displayed values
    #[arg(long, default_value = "80")]
    max_value_length: usize,

    /// Treat null values as missing keys
    #[arg(long)]
    null_as_missing: bool,

    /// Ignore whitespace differences in strings
    #[arg(long)]
    ignore_whitespace: bool,

    /// Verbose output (show parsing progress)
    #[arg(short, long)]
    verbose: bool,

    /// Quiet mode (only show changes, suppress summary)
    #[arg(short, long)]
    quiet: bool,
}

/// Output format argument for clap
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormatArg {
    /// Colored terminal output
    Terminal,
    /// JSON representation
    Json,
    /// Plain text (no colors)
    Plain,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Terminal => OutputFormat::Terminal,
            OutputFormatArg::Json => OutputFormat::Json,
            OutputFormatArg::Plain => OutputFormat::Plain,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match run(cli) {
        Ok(exit_code) => process::exit(exit_code),
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(2);
        }
    }
}

fn run(cli: Cli) -> Result<i32> {
    if cli.verbose {
        eprintln!("Parsing {}...", cli.file1.display());
    }

    let old = parse_file(&cli.file1)
        .with_context(|| format!("Failed to parse first file: {}", cli.file1.display()))?;

    if cli.verbose {
        eprintln!("Parsing {}...", cli.file2.display());
    }

    let new = parse_file(&cli.file2)
        .with_context(|| format!("Failed to parse second file: {}", cli.file2.display()))?;

    if cli.verbose {
        eprintln!("Computing diff...");
    }

    let diff_config = DiffConfig {
        ignore_whitespace: cli.ignore_whitespace,
        treat_null_as_missing: cli.null_as_missing,
        array_diff_strategy: ArrayDiffStrategy::Positional,
    };

    let diff = compute_diff(&old, &new, &diff_config);

    if cli.verbose {
        eprintln!("Formatting output...");
    }

    let output_options = OutputOptions {
        compact: cli.compact,
        show_values: cli.show_values,
        max_value_length: cli.max_value_length,
        context_lines: 0,
    };

    let output_format: OutputFormat = cli.format.into();
    let output = format_diff(&diff, &output_format, &output_options)
        .context("Failed to format diff output")?;

    if !cli.quiet {
        println!("{}", output);
    } else {
        let lines: Vec<&str> = output.lines().collect();
        for line in lines {
            if !line.starts_with("Summary:") && !line.trim().is_empty() {
                println!("{}", line);
            }
        }
    }

    if diff.is_empty() {
        Ok(0)
    } else {
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_conversion() {
        assert_eq!(
            OutputFormat::from(OutputFormatArg::Terminal),
            OutputFormat::Terminal
        );
        assert_eq!(
            OutputFormat::from(OutputFormatArg::Json),
            OutputFormat::Json
        );
        assert_eq!(
            OutputFormat::from(OutputFormatArg::Plain),
            OutputFormat::Plain
        );
    }
}
