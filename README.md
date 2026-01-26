# SDIFF

[![CI](https://github.com/maxmalkin/sdiff/actions/workflows/ci.yml/badge.svg)](https://github.com/maxmalkin/sdiff/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.93.0-blue.svg)](https://blog.rust-lang.org/)

Semantic diff tool for JSON and YAML. Compares structured data and shows only meaningful changes, ignoring formatting, whitespace, and key ordering.

## Example

```bash
$ sdiff old.json new.json
• age: 30 → 31

Summary: 1 modified
```

Where traditional `diff` would show every line changed due to reformatting, SDIFF shows only the actual semantic change.

## Installation

```bash
cargo install sdiff
```

Or build from source:

```bash
git clone https://github.com/maxmalkin/sdiff
cd sdiff
cargo install --path .
```

## Usage

```bash
sdiff old.json new.json              # Compare files
sdiff config.json config.yaml        # Mixed formats supported
sdiff old.json new.json --format=json  # JSON output for scripting
sdiff old.json new.json --quiet      # Suppress summary
```

Run `sdiff --help` for all options.

### Exit Codes

- **0**: No changes (files are semantically identical)
- **1**: Changes found
- **2**: Error

## Library

```rust
use sdiff::{parse_file, compute_diff, format_diff, DiffConfig, OutputFormat, OutputOptions};
use std::path::Path;

let old = parse_file(Path::new("old.json"))?;
let new = parse_file(Path::new("new.json"))?;
let diff = compute_diff(&old, &new, &DiffConfig::default());
let output = format_diff(&diff, &OutputFormat::Terminal, &OutputOptions::default())?;
println!("{}", output);
```

## License

MIT
