# SDIFF

[![CI](https://github.com/maxmalkin/sdiff/actions/workflows/ci.yml/badge.svg)](https://github.com/maxmalkin/sdiff/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.93.0-blue.svg)](https://blog.rust-lang.org/)

Semantic diff tool for JSON, YAML, and TOML. Compares structured data and shows only meaningful changes, ignoring formatting, whitespace, and key ordering.

## Example

```bash
$ sdiff-rs old.json new.json
• age: 30 → 31

Summary: 1 modified
```

Where traditional `diff` would show every line changed due to reformatting, SDIFF shows only the actual semantic change.

## Installation

```bash
cargo install sdiff-rs
```

Or build from source:

```bash
git clone https://github.com/maxmalkin/sdiff
cd sdiff
cargo install --path .
```

## Usage

```bash
# Basic usage
sdiff-rs old.json new.json              # Compare files
sdiff-rs config.json config.yaml        # Mixed formats supported
sdiff-rs Cargo.toml Cargo.toml.bak      # TOML support

# Stdin support
cat file.json | sdiff-rs - other.json --input-format=json
curl -s api/config | sdiff-rs - local.json --input-format=json

# Output formats
sdiff-rs old.json new.json --format=json    # JSON output for scripting
sdiff-rs old.json new.json --format=plain   # Plain text (no colors)
sdiff-rs old.json new.json --quiet          # Suppress summary

# Path filtering
sdiff-rs old.json new.json --ignore "metadata.timestamp"   # Ignore specific paths
sdiff-rs old.json new.json --ignore "**.version"           # Ignore version at any depth
sdiff-rs old.json new.json --only "spec.**"                # Show only spec changes

# Array comparison strategies
sdiff-rs old.json new.json --array-strategy=positional  # Compare by index (default)
sdiff-rs old.json new.json --array-strategy=lcs         # Detect insertions/deletions
```

Run `sdiff-rs --help` for all options.

### Array Diff Strategies

**Positional** (default): Compares arrays element-by-element by index. Fast but shows misleading changes when elements are inserted.

**LCS** (Longest Common Subsequence): Detects true insertions and deletions. Better for arrays where elements may be added or removed in the middle.

```bash
# Example: [1, 2, 3] → [1, 4, 2, 3]
$ sdiff-rs old.json new.json --array-strategy=positional
• [1]: 2 → 4
• [2]: 3 → 2
+ [3]: 3
Summary: 1 added, 2 modified

$ sdiff-rs old.json new.json --array-strategy=lcs
+ [1]: 4
Summary: 1 added
```

### Path Filtering

Filter diff output using glob-style patterns:

- `foo.bar` - exact path match
- `*` - matches any single path segment
- `**` - matches any number of path segments

```bash
sdiff-rs old.json new.json --ignore "**.timestamp"     # Ignore all timestamp fields
sdiff-rs old.json new.json --only "spec.**"            # Only show spec changes
sdiff-rs old.json new.json --only "data.*" --ignore "data.internal"
```

### Git Integration

Use sdiff-rs as a git difftool for structured data files:

```bash
# Install sdiff-rs as a git difftool
sdiff-rs --git-install

# Use with git
git difftool -t sdiff-rs HEAD~1 -- config.json
git difftool -t sdiff-rs main feature -- settings.yaml

# Check configuration status
sdiff-rs --git-status

# Uninstall
sdiff-rs --git-uninstall
```

For automatic usage with specific file types, add to `.gitattributes`:

```
*.json diff=sdiff-rs
*.yaml diff=sdiff-rs
*.toml diff=sdiff-rs
```

### Exit Codes

- **0**: No changes (files are semantically identical)
- **1**: Changes found
- **2**: Error

## Library

```rust
use sdiff_rs::{parse_file, compute_diff, format_diff, DiffConfig, OutputFormat, OutputOptions};
use std::path::Path;

let old = parse_file(Path::new("old.json"))?;
let new = parse_file(Path::new("new.json"))?;
let diff = compute_diff(&old, &new, &DiffConfig::default());
let output = format_diff(&diff, &OutputFormat::Terminal, &OutputOptions::default())?;
println!("{}", output);
```

### Parsing from stdin or strings

```rust
use sdiff_rs::{parse_content, parse_stdin, FormatHint};

// Parse from string with format hint
let node = parse_content(r#"{"key": "value"}"#, FormatHint::Json, "input")?;

// Parse from stdin
let node = parse_stdin(FormatHint::Auto)?;
```

### Path filtering

```rust
use sdiff_rs::{compute_diff, DiffConfig};
use sdiff_rs::filter::{filter_diff, FilterConfig};

let diff = compute_diff(&old, &new, &DiffConfig::default());

let filter = FilterConfig::new()
    .ignore("metadata.**")
    .only("spec.**");
let filtered = filter_diff(&diff, &filter);
```

### LCS array diffing

```rust
use sdiff_rs::{compute_diff, DiffConfig, ArrayDiffStrategy};

let config = DiffConfig {
    array_diff_strategy: ArrayDiffStrategy::Lcs,
    ..Default::default()
};
let diff = compute_diff(&old, &new, &config);
```

## License

MIT
