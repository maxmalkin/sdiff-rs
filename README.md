# SDIFF - Semantic Diff Tool for Structured Data

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**SDIFF** is a high-performance semantic diff tool that compares structured data files (JSON, YAML) and shows only meaningful changes, ignoring formatting, whitespace, and key ordering differences.

## The Problem

Traditional diff tools like `diff` and `git diff` show line-by-line changes, which produces massive, noisy output when:
- Files are reformatted (indentation, whitespace changes)
- Object keys are reordered in JSON/YAML
- Arrays elements are modified
- Configuration files are restructured

## The Solution

SDIFF parses files into Abstract Syntax Trees and compares the semantic structure, not the text representation.

### Example

**Input files:**

`file1.json`:
```json
{"name": "John", "age": 30, "city": "NYC"}
```

`file2.json`:
```json
{
  "name": "John",
  "age": 31,
  "city": "NYC"
}
```

**Traditional diff output:**
```diff
< {"name": "John", "age": 30, "city": "NYC"}
> {
>   "name": "John",
>   "age": 31,
>   "city": "NYC"
> }
```
(Shows everything changed due to formatting)

**SDIFF output:**
```
• age: 30 → 31

Summary: 1 modified
```
(Shows only the actual semantic change)

## Features

- **Semantic comparison**: Ignores formatting, whitespace, and key ordering
- **Multiple formats**: Supports JSON and YAML (mixed comparisons supported)
- **Colored output**: Visual distinction between added, removed, and modified fields
- **Three output modes**: Terminal, JSON, Plain
- **Configurable**: Control verbosity, value display, whitespace handling
- **Fast**: Written in Rust with quicker algorithms
- **Library + CLI**: Use as a CLI tool or integrate into your projects

## Installation

### From Source

```bash
git clone https://github.com/maxmalkin/sdiff
cd sdiff
cargo build --release
cargo install --path .
```

The binary will be installed to `~/.cargo/bin/sdiff`.

### Using Cargo

```bash
cargo install sdiff
```

## Usage

### Basic Usage

```bash
# Compare two JSON files
sdiff old.json new.json

# Compare JSON and YAML
sdiff config.json config.yaml

# Use different output formats
sdiff old.json new.json --format=json
sdiff old.json new.json --format=plain
```

### CLI Options

```
USAGE:
    sdiff [OPTIONS] <FILE1> <FILE2>

ARGS:
    <FILE1>    First file to compare
    <FILE2>    Second file to compare

OPTIONS:
    -f, --format <FORMAT>
            Output format [default: terminal] [possible values: terminal, json, plain]

    -c, --compact
            Show only changes (hide unchanged fields) [default: true]

        --show-values
            Show full values instead of previews

        --max-value-length <LENGTH>
            Maximum length for displayed values [default: 80]

        --null-as-missing
            Treat null values as missing keys

        --ignore-whitespace
            Ignore whitespace differences in strings

    -v, --verbose
            Verbose output (show parsing progress)

    -q, --quiet
            Quiet mode (only show changes, suppress summary)

    -h, --help
            Print help information

    -V, --version
            Print version information
```

### Examples

#### Show only changes (default)
```bash
sdiff old.json new.json --compact
```

#### Show all fields including unchanged
```bash
sdiff old.json new.json --compact=false
```

#### JSON output for programmatic use
```bash
sdiff old.json new.json --format=json | jq '.stats'
```

#### Ignore whitespace differences
```bash
sdiff old.yaml new.yaml --ignore-whitespace
```

#### Verbose mode
```bash
sdiff large1.json large2.json --verbose
```
Output:
```
Parsing large1.json...
Parsing large2.json...
Computing diff...
Formatting output...
• user.age: 30 → 31

Summary: 1 modified
```

### Exit Codes

- **0**: No changes detected (files are semantically identical)
- **1**: Changes found
- **2**: Error (file not found, parse error, invalid arguments)

Use in scripts:
```bash
if sdiff config.json config.yaml --quiet; then
    echo "Files are identical"
else
    echo "Files differ"
fi
```

## Output Formats

### Terminal (Default)

Colored output with visual indicators:
- 🟢 **Green (+)**: Added fields
- 🔴 **Red (-)**: Removed fields
- 🟡 **Yellow (•)**: Modified fields

```
+ email: "john@example.com"
- deprecated: "old field"
• age: 30 → 31

Summary: 1 added, 1 removed, 1 modified
```

### JSON

Structured output for tool integration:
```json
{
  "changes": [
    {
      "path": ["age"],
      "type": "modified",
      "old_value": 30,
      "new_value": 31
    }
  ],
  "stats": {
    "added": 0,
    "removed": 0,
    "modified": 1,
    "unchanged": 2
  }
}
```

### Plain

Plain text without colors (suitable for piping):
```
• age: 30 → 31

Summary: 1 modified
```

## Library

SDIFF can also be used as a Rust library:

```rust
use sdiff::{parse_file, compute_diff, format_diff, DiffConfig, OutputFormat, OutputOptions};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse files
    let old = parse_file(Path::new("old.json"))?;
    let new = parse_file(Path::new("new.json"))?;

    // Compute diff
    let config = DiffConfig::default();
    let diff = compute_diff(&old, &new, &config);

    // Format output
    let output = format_diff(&diff, &OutputFormat::Terminal, &OutputOptions::default())?;
    println!("{}", output);

    // Check if files are identical
    if diff.is_empty() {
        println!("No changes!");
    }

    Ok(())
}
```

Add to your `Cargo.toml`:
```toml
[dependencies]
sdiff = "0.1"
```

## Supported Formats

- **JSON**: Full JSON support
- **YAML**: Full YAML 1.2 support
  - Anchors and aliases are resolved
  - Tags are evaluated
  - Non-string keys are converted to strings

Format detection is automatic based on file extension (`.json`, `.yaml`, `.yml`), with fallback detection if extension is unknown.

<!--## Comparison to Other Tools

| Feature | SDIFF | jq | diff | dyff |
|---------|-------|-----|------|------|
| Semantic comparison | ✅ | ⚠️ | ❌ | ✅ |
| Ignores formatting | ✅ | ✅ | ❌ | ✅ |
| Ignores key ordering | ✅ | ❌ | ❌ | ✅ |
| Colored output | ✅ | ❌ | ⚠️ | ✅ |
| JSON support | ✅ | ✅ | ✅ | ✅ |
| YAML support | ✅ | ❌ | ✅ | ✅ |
| Mixed JSON/YAML | ✅ | ❌ | ❌ | ✅ |
| Standalone binary | ✅ | ✅ | ✅ | ✅ |
| Library API | ✅ | ❌ | ❌ | ❌ |
| Written in Rust | ✅ | ❌ | ❌ | ❌ |-->

## Performance

SDIFF is designed for high performance:

- **Target**: Parse and diff two 1MB JSON files in under 100ms
- **Memory**: Uses less than 50MB for typical files (< 10MB)
- **Binary size**: Under 5MB (release build)

- 10KB files: ~1ms
- 100KB files: ~10ms
- 1MB files: ~80ms

## Architecture

SDIFF uses a three-phase approach:

1. **Parse**: Convert JSON/YAML to a unified AST representation
2. **Diff**: Recursively compare ASTs to find semantic differences
3. **Format**: Present changes in the chosen output format

The AST representation normalizes:
- Formatting (whitespace, indentation)
- Key ordering in objects
- Number representation (all stored as f64)
- File format differences (JSON vs YAML)

<!--## Development

### Building

```bash
cargo build          # Debug build
cargo build --release  # Optimized release build
```-->

<!--### Testing

```bash
cargo test           # Run all tests
cargo test --lib     # Unit tests only
cargo test --test integration_tests  # Integration tests only
```-->

<!--### Linting

```bash
cargo fmt            # Format code
cargo clippy         # Run lints
```-->

<!--### Documentation

```bash
cargo doc --no-deps --open  # Generate and open documentation
```-->

## Structure

```
sdiff/
├── src/
│   ├── main.rs       # CLI entry point
│   ├── lib.rs        # Library exports
│   ├── tree.rs       # AST node definitions
│   ├── parser.rs     # JSON/YAML parsing
│   ├── diff.rs       # Core diff algorithm
│   ├── output.rs     # Output formatting
│   └── error.rs      # Error types
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/     # Test data files
└── Cargo.toml
```

<!--## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Ensure all tests pass (`cargo test`)
5. Ensure code is formatted (`cargo fmt`)
6. Ensure no clippy warnings (`cargo clippy`)
7. Commit your changes (`git commit -m 'feat: add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

Please follow conventional commit message format:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for test additions/changes
- `refactor:` for code refactoring
- `chore:` for maintenance tasks-->


## To-do

- [ ] LCS-based array diffing for better array change detection
- [ ] Context lines around changes
- [ ] Configurable color schemes
- [ ] Diff merging capabilities
- [ ] Support for more formats (TOML, XML)
- [ ] Streaming mode for very large files
- [ ] Interactive mode for conflict resolution

## License

This project is licensed under the MIT License.
