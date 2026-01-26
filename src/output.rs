//! Output formatting for diff results.
//!
//! This module handles formatting diff results in various output formats
//! (terminal with colors, JSON, plain text). It provides control over
//! what is displayed and how values are formatted.
//!
//! # Examples
//!
//! ```
//! use sdiff::{Node, compute_diff, DiffConfig, format_diff, OutputFormat, OutputOptions};
//!
//! let old = Node::Number(42.0);
//! let new = Node::Number(43.0);
//! let diff = compute_diff(&old, &new, &DiffConfig::default());
//!
//! let output = format_diff(&diff, &OutputFormat::Terminal, &OutputOptions::default()).unwrap();
//! println!("{}", output);
//! ```

use crate::diff::{Change, ChangeType, Diff};
use crate::error::OutputError;
use crate::tree::Node;
use colored::*;

/// Output format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Colored terminal output with ANSI escape codes
    Terminal,
    /// JSON representation of the diff
    Json,
    /// Plain text, no colors (suitable for piping)
    Plain,
}

/// Options for controlling output formatting.
///
/// These options control what information is displayed and how values
/// are formatted in the output.
#[derive(Debug, Clone)]
pub struct OutputOptions {
    /// Hide unchanged fields (only show changes)
    pub compact: bool,
    /// Show full values instead of previews for large values
    pub show_values: bool,
    /// Maximum length for displayed values (truncate if longer)
    pub max_value_length: usize,
    /// Show N unchanged lines around changes (context)
    pub context_lines: usize,
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            compact: true,
            show_values: false,
            max_value_length: 80,
            context_lines: 0,
        }
    }
}

/// Formats a diff according to the specified format and options.
///
/// # Arguments
///
/// * `diff` - The diff to format
/// * `format` - The output format (Terminal, JSON, or Plain)
/// * `options` - Formatting options
///
/// # Returns
///
/// Returns the formatted string on success, or an OutputError on failure.
///
/// # Examples
///
/// ```
/// use sdiff::{Node, compute_diff, DiffConfig, format_diff, OutputFormat, OutputOptions};
///
/// let old = Node::Number(42.0);
/// let new = Node::Number(43.0);
/// let diff = compute_diff(&old, &new, &DiffConfig::default());
///
/// let output = format_diff(&diff, &OutputFormat::Terminal, &OutputOptions::default()).unwrap();
/// assert!(output.contains("42"));
/// assert!(output.contains("43"));
/// ```
pub fn format_diff(
    diff: &Diff,
    format: &OutputFormat,
    options: &OutputOptions,
) -> Result<String, OutputError> {
    match format {
        OutputFormat::Terminal => Ok(format_terminal(diff, options)),
        OutputFormat::Json => format_json(diff),
        OutputFormat::Plain => Ok(format_plain(diff, options)),
    }
}

/// Formats a diff for terminal output with colors.
///
/// Color scheme:
/// - Added: green (bright_green for symbols)
/// - Removed: red (bright_red for symbols)
/// - Modified: yellow (bright_yellow for symbols)
/// - Unchanged: dim white (if shown)
///
/// # Arguments
///
/// * `diff` - The diff to format
/// * `options` - Formatting options
///
/// # Returns
///
/// Returns the formatted colored string.
fn format_terminal(diff: &Diff, options: &OutputOptions) -> String {
    let mut output = String::new();

    let changes: Vec<&Change> = diff
        .changes
        .iter()
        .filter(|c| should_show_change(c, options))
        .collect();

    if changes.is_empty() {
        return "No changes detected.".dimmed().to_string();
    }

    for change in changes {
        let line = format_change_terminal(change, options);
        output.push_str(&line);
        output.push('\n');
    }

    output.push('\n');
    output.push_str(&format_summary(&diff.stats));

    output
}

/// Formats a single change for terminal output.
fn format_change_terminal(change: &Change, options: &OutputOptions) -> String {
    let path = format_path(&change.path);

    match change.change_type {
        ChangeType::Added => {
            let value = format_value(change.new_value.as_ref().unwrap(), options.max_value_length);
            format!("{} {}: {}", "+".bright_green(), path.green(), value.green())
        }
        ChangeType::Removed => {
            let value = format_value(change.old_value.as_ref().unwrap(), options.max_value_length);
            format!("{} {}: {}", "-".bright_red(), path.red(), value.red())
        }
        ChangeType::Modified => {
            let old_value =
                format_value(change.old_value.as_ref().unwrap(), options.max_value_length);
            let new_value =
                format_value(change.new_value.as_ref().unwrap(), options.max_value_length);
            format!(
                "{} {}: {} {} {}",
                "•".bright_yellow(),
                path.yellow(),
                old_value.yellow(),
                "→".bright_yellow(),
                new_value.yellow()
            )
        }
        ChangeType::Unchanged => {
            let value = format_value(change.old_value.as_ref().unwrap(), options.max_value_length);
            format!("  {}: {}", path.dimmed(), value.dimmed())
        }
    }
}

/// Formats a diff as JSON.
///
/// The JSON structure includes both the changes and statistics.
///
/// # Arguments
///
/// * `diff` - The diff to format
///
/// # Returns
///
/// Returns the JSON string on success, or an OutputError on failure.
fn format_json(diff: &Diff) -> Result<String, OutputError> {
    use serde_json::json;

    let changes: Vec<serde_json::Value> = diff
        .changes
        .iter()
        .map(|c| {
            json!({
                "path": c.path,
                "type": format!("{:?}", c.change_type).to_lowercase(),
                "old_value": c.old_value.as_ref().map(node_to_json_value),
                "new_value": c.new_value.as_ref().map(node_to_json_value),
            })
        })
        .collect();

    let output = json!({
        "changes": changes,
        "stats": {
            "added": diff.stats.added,
            "removed": diff.stats.removed,
            "modified": diff.stats.modified,
            "unchanged": diff.stats.unchanged,
        }
    });

    serde_json::to_string_pretty(&output)
        .map_err(|e| OutputError::JsonSerializationError { source: e })
}

/// Formats a diff for plain text output (no colors).
///
/// Uses the same format as terminal output but without ANSI color codes.
///
/// # Arguments
///
/// * `diff` - The diff to format
/// * `options` - Formatting options
///
/// # Returns
///
/// Returns the formatted plain text string.
fn format_plain(diff: &Diff, options: &OutputOptions) -> String {
    let mut output = String::new();

    let changes: Vec<&Change> = diff
        .changes
        .iter()
        .filter(|c| should_show_change(c, options))
        .collect();

    if changes.is_empty() {
        return "No changes detected.".to_string();
    }

    for change in changes {
        let line = format_change_plain(change, options);
        output.push_str(&line);
        output.push('\n');
    }

    output.push('\n');
    output.push_str(&format_summary(&diff.stats));

    output
}

/// Formats a single change for plain text output.
fn format_change_plain(change: &Change, options: &OutputOptions) -> String {
    let path = format_path(&change.path);

    match change.change_type {
        ChangeType::Added => {
            let value = format_value(change.new_value.as_ref().unwrap(), options.max_value_length);
            format!("+ {}: {}", path, value)
        }
        ChangeType::Removed => {
            let value = format_value(change.old_value.as_ref().unwrap(), options.max_value_length);
            format!("- {}: {}", path, value)
        }
        ChangeType::Modified => {
            let old_value =
                format_value(change.old_value.as_ref().unwrap(), options.max_value_length);
            let new_value =
                format_value(change.new_value.as_ref().unwrap(), options.max_value_length);
            format!("• {}: {} → {}", path, old_value, new_value)
        }
        ChangeType::Unchanged => {
            let value = format_value(change.old_value.as_ref().unwrap(), options.max_value_length);
            format!("  {}: {}", path, value)
        }
    }
}

/// Converts a path vector to a readable string.
///
/// Joins path components with dots for object keys and preserves
/// array index notation.
///
/// # Arguments
///
/// * `path` - The path vector
///
/// # Returns
///
/// Returns the formatted path string.
///
/// # Examples
///
/// - `["user", "name"]` → `"user.name"`
/// - `["items", "[0]", "id"]` → `"items[0].id"`
/// - `["[0]"]` → `"[0]"`
fn format_path(path: &[String]) -> String {
    if path.is_empty() {
        return "(root)".to_string();
    }

    let mut result = String::new();
    for (i, component) in path.iter().enumerate() {
        if component.starts_with('[') {
            result.push_str(component);
        } else {
            if i > 0 {
                result.push('.');
            }
            result.push_str(component);
        }
    }
    result
}

/// Formats a node value for display.
///
/// Shows a preview of the value, truncating if it exceeds max_length.
///
/// # Arguments
///
/// * `node` - The node to format
/// * `max_length` - Maximum length before truncation
///
/// # Returns
///
/// Returns the formatted value string.
fn format_value(node: &Node, max_length: usize) -> String {
    node.preview(max_length)
}

/// Determines if a change should be shown based on options.
///
/// # Arguments
///
/// * `change` - The change to check
/// * `options` - Formatting options
///
/// # Returns
///
/// Returns true if the change should be displayed.
fn should_show_change(change: &Change, options: &OutputOptions) -> bool {
    if options.compact {
        // In compact mode, hide unchanged values
        !matches!(change.change_type, ChangeType::Unchanged)
    } else {
        // In non-compact mode, show everything
        true
    }
}

/// Formats summary statistics.
///
/// # Arguments
///
/// * `stats` - The diff statistics
///
/// # Returns
///
/// Returns the formatted summary string.
fn format_summary(stats: &crate::diff::DiffStats) -> String {
    if stats.is_empty() {
        return "Summary: No changes".to_string();
    }

    let mut parts = Vec::new();
    if stats.added > 0 {
        parts.push(format!("{} added", stats.added));
    }
    if stats.removed > 0 {
        parts.push(format!("{} removed", stats.removed));
    }
    if stats.modified > 0 {
        parts.push(format!("{} modified", stats.modified));
    }
    if stats.unchanged > 0 {
        parts.push(format!("{} unchanged", stats.unchanged));
    }

    format!("Summary: {}", parts.join(", "))
}

/// Converts a Node to a serde_json::Value for JSON serialization.
///
/// # Arguments
///
/// * `node` - The node to convert
///
/// # Returns
///
/// Returns the JSON value.
fn node_to_json_value(node: &Node) -> serde_json::Value {
    use serde_json::json;

    match node {
        Node::Null => json!(null),
        Node::Bool(b) => json!(b),
        Node::Number(n) => json!(n),
        Node::String(s) => json!(s),
        Node::Array(arr) => {
            let values: Vec<serde_json::Value> = arr.iter().map(node_to_json_value).collect();
            json!(values)
        }
        Node::Object(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), node_to_json_value(v)))
                .collect();
            json!(obj)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::DiffStats;
    use std::collections::HashMap;

    #[test]
    fn test_format_path_simple() {
        assert_eq!(format_path(&["name".to_string()]), "name");
        assert_eq!(
            format_path(&["user".to_string(), "name".to_string()]),
            "user.name"
        );
    }

    #[test]
    fn test_format_path_array() {
        assert_eq!(format_path(&["[0]".to_string()]), "[0]");
        assert_eq!(
            format_path(&["items".to_string(), "[0]".to_string()]),
            "items[0]"
        );
        assert_eq!(
            format_path(&["items".to_string(), "[0]".to_string(), "id".to_string()]),
            "items[0].id"
        );
    }

    #[test]
    fn test_format_path_empty() {
        assert_eq!(format_path(&[]), "(root)");
    }

    #[test]
    fn test_format_value_primitives() {
        assert_eq!(format_value(&Node::Null, 100), "null");
        assert_eq!(format_value(&Node::Bool(true), 100), "true");
        assert_eq!(format_value(&Node::Number(42.0), 100), "42");
        assert_eq!(
            format_value(&Node::String("hello".to_string()), 100),
            "\"hello\""
        );
    }

    #[test]
    fn test_format_value_truncation() {
        let long_string = "a".repeat(100);
        let node = Node::String(long_string);
        let formatted = format_value(&node, 20);
        assert!(formatted.len() <= 23); // 20 + quotes + ellipsis
    }

    #[test]
    fn test_should_show_change_compact() {
        let options = OutputOptions {
            compact: true,
            ..Default::default()
        };

        let added = Change {
            path: vec!["test".to_string()],
            change_type: ChangeType::Added,
            old_value: None,
            new_value: Some(Node::Null),
        };
        assert!(should_show_change(&added, &options));

        let unchanged = Change {
            path: vec!["test".to_string()],
            change_type: ChangeType::Unchanged,
            old_value: Some(Node::Null),
            new_value: Some(Node::Null),
        };
        assert!(!should_show_change(&unchanged, &options));
    }

    #[test]
    fn test_should_show_change_non_compact() {
        let options = OutputOptions {
            compact: false,
            ..Default::default()
        };

        let unchanged = Change {
            path: vec!["test".to_string()],
            change_type: ChangeType::Unchanged,
            old_value: Some(Node::Null),
            new_value: Some(Node::Null),
        };
        assert!(should_show_change(&unchanged, &options));
    }

    #[test]
    fn test_format_summary_empty() {
        let stats = DiffStats::new();
        assert_eq!(format_summary(&stats), "Summary: No changes");
    }

    #[test]
    fn test_format_summary_with_changes() {
        let stats = DiffStats {
            added: 2,
            removed: 1,
            modified: 3,
            unchanged: 5,
        };
        let summary = format_summary(&stats);
        assert!(summary.contains("2 added"));
        assert!(summary.contains("1 removed"));
        assert!(summary.contains("3 modified"));
        assert!(summary.contains("5 unchanged"));
    }

    #[test]
    fn test_format_plain_no_changes() {
        let diff = Diff {
            changes: vec![],
            stats: DiffStats::new(),
        };
        let output = format_plain(&diff, &OutputOptions::default());
        assert_eq!(output, "No changes detected.");
    }

    #[test]
    fn test_format_plain_with_changes() {
        let diff = Diff {
            changes: vec![Change {
                path: vec!["age".to_string()],
                change_type: ChangeType::Modified,
                old_value: Some(Node::Number(30.0)),
                new_value: Some(Node::Number(31.0)),
            }],
            stats: DiffStats {
                added: 0,
                removed: 0,
                modified: 1,
                unchanged: 0,
            },
        };
        let output = format_plain(&diff, &OutputOptions::default());
        assert!(output.contains("age"));
        assert!(output.contains("30"));
        assert!(output.contains("31"));
        assert!(output.contains("Summary: 1 modified"));
    }

    #[test]
    fn test_format_json() {
        let diff = Diff {
            changes: vec![Change {
                path: vec!["age".to_string()],
                change_type: ChangeType::Modified,
                old_value: Some(Node::Number(30.0)),
                new_value: Some(Node::Number(31.0)),
            }],
            stats: DiffStats {
                added: 0,
                removed: 0,
                modified: 1,
                unchanged: 0,
            },
        };
        let output = format_json(&diff).unwrap();
        assert!(output.contains("\"age\""));
        assert!(output.contains("30"));
        assert!(output.contains("31"));
        assert!(output.contains("\"modified\""));
        assert!(output.contains("\"stats\""));
    }

    #[test]
    fn test_node_to_json_value() {
        assert_eq!(node_to_json_value(&Node::Null), serde_json::json!(null));
        assert_eq!(
            node_to_json_value(&Node::Bool(true)),
            serde_json::json!(true)
        );
        assert_eq!(
            node_to_json_value(&Node::Number(42.0)),
            serde_json::json!(42.0)
        );
        assert_eq!(
            node_to_json_value(&Node::String("test".to_string())),
            serde_json::json!("test")
        );

        let arr = Node::Array(vec![Node::Number(1.0), Node::Number(2.0)]);
        assert_eq!(node_to_json_value(&arr), serde_json::json!([1.0, 2.0]));

        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::String("value".to_string()));
        let obj = Node::Object(map);
        assert_eq!(
            node_to_json_value(&obj),
            serde_json::json!({"key": "value"})
        );
    }

    #[test]
    fn test_format_terminal_no_changes() {
        let diff = Diff {
            changes: vec![],
            stats: DiffStats::new(),
        };
        let output = format_terminal(&diff, &OutputOptions::default());
        assert!(output.contains("No changes"));
    }

    #[test]
    fn test_format_change_types() {
        let options = OutputOptions::default();

        // Added
        let added = Change {
            path: vec!["new_field".to_string()],
            change_type: ChangeType::Added,
            old_value: None,
            new_value: Some(Node::String("value".to_string())),
        };
        let output = format_change_plain(&added, &options);
        assert!(output.starts_with('+'));
        assert!(output.contains("new_field"));

        // Removed
        let removed = Change {
            path: vec!["old_field".to_string()],
            change_type: ChangeType::Removed,
            old_value: Some(Node::String("value".to_string())),
            new_value: None,
        };
        let output = format_change_plain(&removed, &options);
        assert!(output.starts_with('-'));
        assert!(output.contains("old_field"));

        // Modified
        let modified = Change {
            path: vec!["field".to_string()],
            change_type: ChangeType::Modified,
            old_value: Some(Node::Number(1.0)),
            new_value: Some(Node::Number(2.0)),
        };
        let output = format_change_plain(&modified, &options);
        assert!(output.contains("→"));
        assert!(output.contains("1"));
        assert!(output.contains("2"));
    }
}
