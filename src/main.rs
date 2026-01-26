//! SDIFF command-line interface.
//!
//! This is the main entry point for the sdiff CLI tool. It uses clap for
//! argument parsing and wires together all the library modules to perform
//! semantic diffs on structured data files.

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use sdiff::{
    compute_diff, filter::filter_diff, filter::FilterConfig, format_diff, parse_content,
    parse_file, ArrayDiffStrategy, DiffConfig, FormatHint, OutputFormat, OutputOptions,
};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

/// SDIFF - Semantic diff tool for structured data
///
/// Intelligently compares JSON, YAML, and TOML files, showing only meaningful changes
/// while ignoring formatting, whitespace, and key ordering differences.
///
/// Use "-" to read from stdin: `cat file.json | sdiff - other.json`
#[derive(Parser)]
#[command(name = "sdiff")]
#[command(version)]
#[command(about = "Semantic diff tool for structured data", long_about = None)]
#[command(author = "SDIFF Contributors")]
struct Cli {
    /// First file to compare (use "-" for stdin)
    #[arg(value_name = "FILE1")]
    file1: String,

    /// Second file to compare (use "-" for stdin)
    #[arg(value_name = "FILE2")]
    file2: String,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "terminal")]
    format: OutputFormatArg,

    /// Input format for stdin (required when using "-")
    #[arg(long, value_enum)]
    input_format: Option<InputFormatArg>,

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

    /// Array comparison strategy
    #[arg(long, value_enum, default_value = "positional")]
    array_strategy: ArrayStrategyArg,

    /// Ignore paths matching these patterns (can be used multiple times)
    #[arg(long = "ignore", value_name = "PATTERN")]
    ignore_patterns: Vec<String>,

    /// Only show paths matching these patterns (can be used multiple times)
    #[arg(long = "only", value_name = "PATTERN")]
    only_patterns: Vec<String>,

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

/// Input format argument for clap
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum InputFormatArg {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
    /// Auto-detect format
    Auto,
}

impl From<InputFormatArg> for FormatHint {
    fn from(arg: InputFormatArg) -> Self {
        match arg {
            InputFormatArg::Json => FormatHint::Json,
            InputFormatArg::Yaml => FormatHint::Yaml,
            InputFormatArg::Toml => FormatHint::Toml,
            InputFormatArg::Auto => FormatHint::Auto,
        }
    }
}

/// Array comparison strategy argument for clap
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ArrayStrategyArg {
    /// Compare arrays by index position (simple, fast)
    Positional,
    /// Use LCS algorithm to detect insertions and deletions
    Lcs,
}

impl From<ArrayStrategyArg> for ArrayDiffStrategy {
    fn from(arg: ArrayStrategyArg) -> Self {
        match arg {
            ArrayStrategyArg::Positional => ArrayDiffStrategy::Positional,
            ArrayStrategyArg::Lcs => ArrayDiffStrategy::Lcs,
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
    let file1_is_stdin = cli.file1 == "-";
    let file2_is_stdin = cli.file2 == "-";

    // Validate stdin usage
    if file1_is_stdin && file2_is_stdin {
        bail!("Cannot read both inputs from stdin");
    }

    let format_hint: FormatHint = cli.input_format.map(Into::into).unwrap_or(FormatHint::Auto);

    // Read stdin content early if needed (can only read once)
    let stdin_content = if file1_is_stdin || file2_is_stdin {
        let mut content = String::new();
        io::stdin()
            .read_to_string(&mut content)
            .context("Failed to read from stdin")?;
        Some(content)
    } else {
        None
    };

    if cli.verbose {
        eprintln!("Parsing {}...", &cli.file1);
    }

    let old = if file1_is_stdin {
        parse_content(stdin_content.as_ref().unwrap(), format_hint, "<stdin>")
            .context("Failed to parse stdin")?
    } else {
        parse_file(&PathBuf::from(&cli.file1))
            .with_context(|| format!("Failed to parse first file: {}", &cli.file1))?
    };

    if cli.verbose {
        eprintln!("Parsing {}...", &cli.file2);
    }

    let new = if file2_is_stdin {
        parse_content(stdin_content.as_ref().unwrap(), format_hint, "<stdin>")
            .context("Failed to parse stdin")?
    } else {
        parse_file(&PathBuf::from(&cli.file2))
            .with_context(|| format!("Failed to parse second file: {}", &cli.file2))?
    };

    if cli.verbose {
        eprintln!("Computing diff...");
    }

    let diff_config = DiffConfig {
        ignore_whitespace: cli.ignore_whitespace,
        treat_null_as_missing: cli.null_as_missing,
        array_diff_strategy: cli.array_strategy.into(),
    };

    let mut diff = compute_diff(&old, &new, &diff_config);

    // Apply path filtering if configured
    if !cli.ignore_patterns.is_empty() || !cli.only_patterns.is_empty() {
        let mut filter_config = FilterConfig::new();
        for pattern in &cli.ignore_patterns {
            filter_config = filter_config.ignore(pattern);
        }
        for pattern in &cli.only_patterns {
            filter_config = filter_config.only(pattern);
        }
        diff = filter_diff(&diff, &filter_config);
    }

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
