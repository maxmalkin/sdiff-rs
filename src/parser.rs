//! File parsing for JSON, YAML, and TOML formats.
//!
//! This module handles parsing structured data files into our AST representation.
//! It supports automatic format detection based on file extension, and falls back
//! to attempting JSON then YAML parsing if the extension is unknown.
//!
//! # Examples
//!
//! ```no_run
//! use sdiff::parser::parse_file;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse a JSON file
//! let node = parse_file(Path::new("data.json"))?;
//!
//! // Parse a YAML file
//! let node = parse_file(Path::new("config.yaml"))?;
//! # Ok(())
//! # }
//! ```

use crate::error::ParseError;
use crate::tree::Node;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

/// Hint for the input format when auto-detection is not possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormatHint {
    /// Automatically detect format (try JSON, then YAML, then TOML)
    #[default]
    Auto,
    /// Parse as JSON
    Json,
    /// Parse as YAML
    Yaml,
    /// Parse as TOML
    Toml,
}

/// Parses content from stdin into a Node.
pub fn parse_stdin(hint: FormatHint) -> Result<Node, ParseError> {
    let mut content = String::new();
    io::stdin()
        .read_to_string(&mut content)
        .map_err(|e| ParseError::read_error("<stdin>", e))?;

    parse_content(&content, hint, "<stdin>")
}

/// Parses content string with the given format hint.
pub fn parse_content(content: &str, hint: FormatHint, source: &str) -> Result<Node, ParseError> {
    match hint {
        FormatHint::Json => {
            parse_json(content).map_err(|e| ParseError::json_error(source.to_string(), e))
        }
        FormatHint::Yaml => {
            parse_yaml(content).map_err(|e| ParseError::yaml_error(source.to_string(), e))
        }
        FormatHint::Toml => {
            parse_toml(content).map_err(|e| ParseError::toml_error(source.to_string(), e))
        }
        FormatHint::Auto => parse_json(content)
            .map_err(|_| ())
            .or_else(|_| parse_yaml(content).map_err(|_| ()))
            .or_else(|_| parse_toml(content).map_err(|_| ()))
            .map_err(|_| ParseError::unknown_format(source.to_string())),
    }
}

/// Parses a file into a Node AST. Format is detected by file extension.
pub fn parse_file(path: &Path) -> Result<Node, ParseError> {
    if !path.exists() {
        return Err(ParseError::file_not_found(
            path.to_string_lossy().to_string(),
        ));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| ParseError::read_error(path.to_string_lossy().to_string(), e))?;

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("json") => parse_json(&content)
            .map_err(|e| ParseError::json_error(path.to_string_lossy().to_string(), e)),
        Some("yaml") | Some("yml") => parse_yaml(&content)
            .map_err(|e| ParseError::yaml_error(path.to_string_lossy().to_string(), e)),
        Some("toml") => parse_toml(&content)
            .map_err(|e| ParseError::toml_error(path.to_string_lossy().to_string(), e)),
        _ => parse_json(&content)
            .map_err(|_| ())
            .or_else(|_| parse_yaml(&content).map_err(|_| ()))
            .or_else(|_| parse_toml(&content).map_err(|_| ()))
            .map_err(|_| ParseError::unknown_format(path.to_string_lossy().to_string())),
    }
}

/// Parses a JSON string into a Node.
pub fn parse_json(content: &str) -> Result<Node, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(content)?;
    Ok(json_to_node(value))
}

/// Parses a YAML string into a Node.
pub fn parse_yaml(content: &str) -> Result<Node, serde_yaml::Error> {
    let value: serde_yaml::Value = serde_yaml::from_str(content)?;
    Ok(yaml_to_node(value))
}

/// Parses a TOML string into a Node.
pub fn parse_toml(content: &str) -> Result<Node, toml::de::Error> {
    let value: toml::Value = toml::from_str(content)?;
    Ok(toml_to_node(value))
}

fn json_to_node(value: serde_json::Value) -> Node {
    match value {
        serde_json::Value::Null => Node::Null,
        serde_json::Value::Bool(b) => Node::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Node::Number(f)
            } else {
                Node::Number(0.0)
            }
        }
        serde_json::Value::String(s) => Node::String(s),
        serde_json::Value::Array(arr) => Node::Array(arr.into_iter().map(json_to_node).collect()),
        serde_json::Value::Object(obj) => {
            let map: HashMap<String, Node> =
                obj.into_iter().map(|(k, v)| (k, json_to_node(v))).collect();
            Node::Object(map)
        }
    }
}

fn yaml_to_node(value: serde_yaml::Value) -> Node {
    match value {
        serde_yaml::Value::Null => Node::Null,
        serde_yaml::Value::Bool(b) => Node::Bool(b),
        serde_yaml::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Node::Number(f)
            } else if let Some(i) = n.as_i64() {
                Node::Number(i as f64)
            } else if let Some(u) = n.as_u64() {
                Node::Number(u as f64)
            } else {
                Node::Number(0.0)
            }
        }
        serde_yaml::Value::String(s) => Node::String(s),
        serde_yaml::Value::Sequence(seq) => {
            Node::Array(seq.into_iter().map(yaml_to_node).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let hash_map: HashMap<String, Node> = map
                .into_iter()
                .map(|(k, v)| {
                    let key_str = match k {
                        serde_yaml::Value::String(s) => s,
                        serde_yaml::Value::Number(n) => n.to_string(),
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        serde_yaml::Value::Null => "null".to_string(),
                        other => format!("{:?}", other),
                    };
                    (key_str, yaml_to_node(v))
                })
                .collect();
            Node::Object(hash_map)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_node(tagged.value),
    }
}

fn toml_to_node(value: toml::Value) -> Node {
    match value {
        toml::Value::String(s) => Node::String(s),
        toml::Value::Integer(i) => Node::Number(i as f64),
        toml::Value::Float(f) => Node::Number(f),
        toml::Value::Boolean(b) => Node::Bool(b),
        toml::Value::Datetime(dt) => Node::String(dt.to_string()),
        toml::Value::Array(arr) => Node::Array(arr.into_iter().map(toml_to_node).collect()),
        toml::Value::Table(t) => {
            let map: HashMap<String, Node> =
                t.into_iter().map(|(k, v)| (k, toml_to_node(v))).collect();
            Node::Object(map)
        }
    }
}
