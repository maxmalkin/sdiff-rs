//! SDIFF command-line interface.
//!
//! This is the main entry point for the sdiff CLI tool. It uses clap for
//! argument parsing and wires together all the library modules to perform
//! semantic diffs on structured data files.

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use sdiff::{
    compute_diff,
    filter::filter_diff,
    filter::FilterConfig,
    format_diff,
    git::{self, detect_git_diff_driver_args, is_null_file},
    parse_content, parse_file, ArrayDiffStrategy, DiffConfig, FormatHint, OutputFormat,
    OutputOptions,
};
use std::env;
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
    #[arg(value_name = "FILE1", required_unless_present_any = ["git_install", "git_uninstall", "git_status"])]
    file1: Option<String>,

    /// Second file to compare (use "-" for stdin)
    #[arg(value_name = "FILE2", required_unless_present_any = ["git_install", "git_uninstall", "git_status"])]
    file2: Option<String>,

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

    /// Install sdiff as a git difftool
    #[arg(long)]
    git_install: bool,

    /// Uninstall sdiff from git configuration
    #[arg(long)]
    git_uninstall: bool,

    /// Show git configuration status
    #[arg(long)]
    git_status: bool,

    /// Additional arguments (for git diff driver 7-arg mode)
    #[arg(hide = true, trailing_var_arg = true)]
    extra_args: Vec<String>,
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
    // Check for git 7-argument diff driver mode before parsing with clap
    // Git passes: path old-file old-hex old-mode new-file new-hex new-mode
    let args: Vec<String> = env::args().skip(1).collect();
    if let Some((old_file, new_file)) = detect_git_diff_driver_args(&args) {
        match run_git_diff_driver(&old_file, &new_file) {
            Ok(exit_code) => process::exit(exit_code),
            Err(err) => {
                eprintln!("Error: {}", err);
                process::exit(2);
            }
        }
    }

    let cli = Cli::parse();

    match run(cli) {
        Ok(exit_code) => process::exit(exit_code),
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(2);
        }
    }
}

/// Runs sdiff in git diff driver mode with the extracted file paths.
fn run_git_diff_driver(old_file: &str, new_file: &str) -> Result<i32> {
    // Handle /dev/null for added/deleted files
    if is_null_file(old_file) {
        println!("File added");
        let new = parse_file(&PathBuf::from(new_file))
            .with_context(|| format!("Failed to parse new file: {}", new_file))?;
        println!("New content: {}", new.preview(200));
        return Ok(1);
    }

    if is_null_file(new_file) {
        println!("File deleted");
        let old = parse_file(&PathBuf::from(old_file))
            .with_context(|| format!("Failed to parse old file: {}", old_file))?;
        println!("Deleted content: {}", old.preview(200));
        return Ok(1);
    }

    // Normal diff
    let old = parse_file(&PathBuf::from(old_file))
        .with_context(|| format!("Failed to parse old file: {}", old_file))?;
    let new = parse_file(&PathBuf::from(new_file))
        .with_context(|| format!("Failed to parse new file: {}", new_file))?;

    let config = DiffConfig::default();
    let diff = compute_diff(&old, &new, &config);

    let output_options = OutputOptions::default();
    let output = format_diff(&diff, &OutputFormat::Terminal, &output_options)
        .context("Failed to format diff output")?;

    println!("{}", output);

    if diff.is_empty() {
        Ok(0)
    } else {
        Ok(1)
    }
}

fn run(cli: Cli) -> Result<i32> {
    // Handle git commands first
    if cli.git_install {
        git::install().context("Failed to install git integration")?;
        return Ok(0);
    }

    if cli.git_uninstall {
        git::uninstall().context("Failed to uninstall git integration")?;
        return Ok(0);
    }

    if cli.git_status {
        git::status().context("Failed to get git status")?;
        return Ok(0);
    }

    // Normal diff mode - files are required
    let file1 = cli.file1.as_ref().expect("file1 is required for diff");
    let file2 = cli.file2.as_ref().expect("file2 is required for diff");

    let file1_is_stdin = file1 == "-";
    let file2_is_stdin = file2 == "-";

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
        eprintln!("Parsing {}...", file1);
    }

    let old = if file1_is_stdin {
        parse_content(stdin_content.as_ref().unwrap(), format_hint, "<stdin>")
            .context("Failed to parse stdin")?
    } else {
        parse_file(&PathBuf::from(file1))
            .with_context(|| format!("Failed to parse first file: {}", file1))?
    };

    if cli.verbose {
        eprintln!("Parsing {}...", file2);
    }

    let new = if file2_is_stdin {
        parse_content(stdin_content.as_ref().unwrap(), format_hint, "<stdin>")
            .context("Failed to parse stdin")?
    } else {
        parse_file(&PathBuf::from(file2))
            .with_context(|| format!("Failed to parse second file: {}", file2))?
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
