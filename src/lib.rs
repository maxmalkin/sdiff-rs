//! SDIFF - Semantic diff tool for structured data.
//!
//! This library provides functionality for comparing structured data files (JSON, YAML)
//! semantically, ignoring formatting differences and focusing on actual content changes.
//!
//! # Example
//!
//! ```no_run
//! use sdiff_rs::{parse_file, compute_diff, DiffConfig, format_diff, OutputFormat, OutputOptions};
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse two files
//! let old = parse_file(Path::new("old.json"))?;
//! let new = parse_file(Path::new("new.json"))?;
//!
//! // Compute the semantic diff
//! let config = DiffConfig::default();
//! let diff = compute_diff(&old, &new, &config);
//!
//! // Format the output
//! let output = format_diff(&diff, &OutputFormat::Terminal, &OutputOptions::default())?;
//! println!("{}", output);
//! # Ok(())
//! # }
//! ```

pub mod diff;
pub mod error;
pub mod filter;
pub mod git;
pub mod output;
pub mod parser;
pub mod tree;

// Re-export commonly used types for convenience
pub use diff::{compute_diff, ArrayDiffStrategy, Change, ChangeType, Diff, DiffConfig};
pub use error::{OutputError, ParseError, SdiffError};
pub use output::{format_diff, OutputFormat, OutputOptions};
pub use parser::{
    parse_content, parse_file, parse_json, parse_stdin, parse_toml, parse_yaml, FormatHint,
};
pub use tree::Node;
