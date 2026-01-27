//! Output formatting for diff results.

use crate::diff::{Change, ChangeType, Diff};
use crate::error::OutputError;
use crate::tree::Node;
use colored::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Terminal,
    Json,
    Plain,
}

#[derive(Debug, Clone)]
pub struct OutputOptions {
    pub compact: bool,
    pub show_values: bool,
    pub max_value_length: usize,
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

fn format_value(node: &Node, max_length: usize) -> String {
    node.preview(max_length)
}

fn should_show_change(change: &Change, options: &OutputOptions) -> bool {
    if options.compact {
        !matches!(change.change_type, ChangeType::Unchanged)
    } else {
        true
    }
}

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
