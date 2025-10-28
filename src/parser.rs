//! File parsing for JSON and YAML formats.
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
use std::path::Path;

/// Parses a file into a Node AST.
///
/// The format is detected by file extension (.json, .yaml, .yml). If the extension
/// is unknown or missing, this function will attempt to parse as JSON first, then
/// YAML if JSON fails.
///
/// # Arguments
///
/// * `path` - Path to the file to parse
///
/// # Returns
///
/// Returns the parsed Node on success, or a ParseError on failure.
///
/// # Errors
///
/// This function will return an error if:
/// - The file does not exist (`ParseError::FileNotFound`)
/// - The file cannot be read (`ParseError::ReadError`)
/// - The file contains invalid JSON (`ParseError::JsonError`)
/// - The file contains invalid YAML (`ParseError::YamlError`)
/// - The file format cannot be determined (`ParseError::UnknownFormat`)
///
/// # Examples
///
/// ```no_run
/// use sdiff::parser::parse_file;
/// use std::path::Path;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let node = parse_file(Path::new("data.json"))?;
/// println!("Parsed successfully!");
/// # Ok(())
/// # }
/// ```
pub fn parse_file(path: &Path) -> Result<Node, ParseError> {
    // Check if file exists
    if !path.exists() {
        return Err(ParseError::file_not_found(
            path.to_string_lossy().to_string(),
        ));
    }

    // Read file contents
    let content = fs::read_to_string(path)
        .map_err(|e| ParseError::read_error(path.to_string_lossy().to_string(), e))?;

    // Detect format by extension
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("json") => parse_json(&content)
            .map_err(|e| ParseError::json_error(path.to_string_lossy().to_string(), e)),
        Some("yaml") | Some("yml") => parse_yaml(&content)
            .map_err(|e| ParseError::yaml_error(path.to_string_lossy().to_string(), e)),
        _ => {
            // Try JSON first, then YAML
            parse_json(&content)
                .map_err(|_| ())
                .or_else(|_| parse_yaml(&content).map_err(|_| ()))
                .map_err(|_| ParseError::unknown_format(path.to_string_lossy().to_string()))
        }
    }
}

/// Parses a JSON string into a Node.
///
/// # Arguments
///
/// * `content` - The JSON string to parse
///
/// # Returns
///
/// Returns the parsed Node on success, or a serde_json::Error on failure.
///
/// # Examples
///
/// ```
/// use sdiff::parser::parse_json;
///
/// let json = r#"{"name": "Alice", "age": 30}"#;
/// let node = parse_json(json).unwrap();
/// ```
pub fn parse_json(content: &str) -> Result<Node, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(content)?;
    Ok(json_to_node(value))
}

/// Parses a YAML string into a Node.
///
/// # Arguments
///
/// * `content` - The YAML string to parse
///
/// # Returns
///
/// Returns the parsed Node on success, or a serde_yaml::Error on failure.
///
/// # Examples
///
/// ```
/// use sdiff::parser::parse_yaml;
///
/// let yaml = "name: Alice\nage: 30";
/// let node = parse_yaml(yaml).unwrap();
/// ```
pub fn parse_yaml(content: &str) -> Result<Node, serde_yaml::Error> {
    let value: serde_yaml::Value = serde_yaml::from_str(content)?;
    Ok(yaml_to_node(value))
}

/// Converts a serde_json::Value to our Node representation.
///
/// This function recursively converts JSON values to our AST, preserving all
/// data while normalizing the representation.
///
/// # Arguments
///
/// * `value` - The serde_json::Value to convert
///
/// # Returns
///
/// Returns a Node representing the same data.
fn json_to_node(value: serde_json::Value) -> Node {
    match value {
        serde_json::Value::Null => Node::Null,
        serde_json::Value::Bool(b) => Node::Bool(b),
        serde_json::Value::Number(n) => {
            // Convert to f64, preserving as much precision as possible
            if let Some(f) = n.as_f64() {
                Node::Number(f)
            } else {
                // Shouldn't happen, but handle gracefully
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

/// Converts a serde_yaml::Value to our Node representation.
///
/// This function recursively converts YAML values to our AST. YAML has additional
/// features beyond JSON (like anchors and tags) which are evaluated during parsing,
/// so the resulting Node represents the fully-evaluated YAML document.
///
/// Non-string keys in YAML maps are converted to strings for compatibility.
///
/// # Arguments
///
/// * `value` - The serde_yaml::Value to convert
///
/// # Returns
///
/// Returns a Node representing the same data.
fn yaml_to_node(value: serde_yaml::Value) -> Node {
    match value {
        serde_yaml::Value::Null => Node::Null,
        serde_yaml::Value::Bool(b) => Node::Bool(b),
        serde_yaml::Value::Number(n) => {
            // Convert to f64
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
                    // Convert key to string
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
        serde_yaml::Value::Tagged(tagged) => {
            // Evaluate the tagged value (ignore the tag, use the value)
            yaml_to_node(tagged.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_json_primitives() {
        assert_eq!(parse_json("null").unwrap(), Node::Null);
        assert_eq!(parse_json("true").unwrap(), Node::Bool(true));
        assert_eq!(parse_json("false").unwrap(), Node::Bool(false));
        assert_eq!(parse_json("42").unwrap(), Node::Number(42.0));
        assert_eq!(parse_json("3.15").unwrap(), Node::Number(3.15));
        assert_eq!(
            parse_json(r#""hello""#).unwrap(),
            Node::String("hello".to_string())
        );
    }

    #[test]
    fn test_parse_json_array() {
        let json = r#"[1, 2, 3]"#;
        let node = parse_json(json).unwrap();
        match node {
            Node::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Node::Number(1.0));
                assert_eq!(arr[1], Node::Number(2.0));
                assert_eq!(arr[2], Node::Number(3.0));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_json_object() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let node = parse_json(json).unwrap();
        match node {
            Node::Object(map) => {
                assert_eq!(map.len(), 2);
                assert_eq!(map.get("name").unwrap(), &Node::String("Alice".to_string()));
                assert_eq!(map.get("age").unwrap(), &Node::Number(30.0));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_json_nested() {
        let json = r#"{"user": {"name": "Bob", "scores": [10, 20, 30]}}"#;
        let node = parse_json(json).unwrap();
        match node {
            Node::Object(map) => match map.get("user").unwrap() {
                Node::Object(user) => {
                    assert_eq!(user.get("name").unwrap(), &Node::String("Bob".to_string()));
                    match user.get("scores").unwrap() {
                        Node::Array(scores) => {
                            assert_eq!(scores.len(), 3);
                        }
                        _ => panic!("Expected scores to be array"),
                    }
                }
                _ => panic!("Expected user to be object"),
            },
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_json_invalid() {
        assert!(parse_json("{invalid json}").is_err());
        assert!(parse_json("[1, 2,]").is_err());
    }

    #[test]
    fn test_parse_yaml_primitives() {
        assert_eq!(parse_yaml("null").unwrap(), Node::Null);
        assert_eq!(parse_yaml("~").unwrap(), Node::Null);
        assert_eq!(parse_yaml("true").unwrap(), Node::Bool(true));
        assert_eq!(parse_yaml("false").unwrap(), Node::Bool(false));
        // Note: serde_yaml 0.9 follows YAML 1.2 where only "true"/"false" are booleans
        // "yes"/"no" are treated as strings in YAML 1.2
        assert_eq!(parse_yaml("42").unwrap(), Node::Number(42.0));
        assert_eq!(parse_yaml("3.15").unwrap(), Node::Number(3.15));
        assert_eq!(
            parse_yaml("hello").unwrap(),
            Node::String("hello".to_string())
        );
    }

    #[test]
    fn test_parse_yaml_array() {
        let yaml = "- 1\n- 2\n- 3";
        let node = parse_yaml(yaml).unwrap();
        match node {
            Node::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Node::Number(1.0));
                assert_eq!(arr[1], Node::Number(2.0));
                assert_eq!(arr[2], Node::Number(3.0));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_yaml_object() {
        let yaml = "name: Alice\nage: 30";
        let node = parse_yaml(yaml).unwrap();
        match node {
            Node::Object(map) => {
                assert_eq!(map.len(), 2);
                assert_eq!(map.get("name").unwrap(), &Node::String("Alice".to_string()));
                assert_eq!(map.get("age").unwrap(), &Node::Number(30.0));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_yaml_nested() {
        let yaml = "user:\n  name: Bob\n  scores:\n    - 10\n    - 20\n    - 30";
        let node = parse_yaml(yaml).unwrap();
        match node {
            Node::Object(map) => match map.get("user").unwrap() {
                Node::Object(user) => {
                    assert_eq!(user.get("name").unwrap(), &Node::String("Bob".to_string()));
                    match user.get("scores").unwrap() {
                        Node::Array(scores) => {
                            assert_eq!(scores.len(), 3);
                        }
                        _ => panic!("Expected scores to be array"),
                    }
                }
                _ => panic!("Expected user to be object"),
            },
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_yaml_invalid() {
        // Invalid YAML: mapping value not allowed here
        assert!(parse_yaml("key: value: invalid").is_err());
        // Invalid YAML: unclosed bracket
        assert!(parse_yaml("[1, 2,").is_err());
    }

    #[test]
    fn test_parse_file_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"key": "value"}}"#).unwrap();
        let path = file.path().with_extension("json");
        fs::copy(file.path(), &path).unwrap();

        let node = parse_file(&path).unwrap();
        match node {
            Node::Object(map) => {
                assert_eq!(map.get("key").unwrap(), &Node::String("value".to_string()));
            }
            _ => panic!("Expected object"),
        }

        // Cleanup
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_parse_file_yaml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "key: value").unwrap();
        let path = file.path().with_extension("yaml");
        fs::copy(file.path(), &path).unwrap();

        let node = parse_file(&path).unwrap();
        match node {
            Node::Object(map) => {
                assert_eq!(map.get("key").unwrap(), &Node::String("value".to_string()));
            }
            _ => panic!("Expected object"),
        }

        // Cleanup
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_parse_file_not_found() {
        let result = parse_file(Path::new("/nonexistent/file.json"));
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::FileNotFound { .. } => {}
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_parse_file_unknown_extension() {
        // Create a file with JSON content but unknown extension
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"key": "value"}}"#).unwrap();
        let path = file.path().with_extension("txt");
        fs::copy(file.path(), &path).unwrap();

        // Should still parse as JSON (tries JSON first)
        let node = parse_file(&path).unwrap();
        match node {
            Node::Object(map) => {
                assert_eq!(map.get("key").unwrap(), &Node::String("value".to_string()));
            }
            _ => panic!("Expected object"),
        }

        // Cleanup
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_yaml_non_string_keys() {
        // YAML allows non-string keys, which we convert to strings
        let yaml = "1: first\n2: second\ntrue: yes";
        let node = parse_yaml(yaml).unwrap();
        match node {
            Node::Object(map) => {
                assert_eq!(map.len(), 3);
                assert_eq!(map.get("1").unwrap(), &Node::String("first".to_string()));
                assert_eq!(map.get("2").unwrap(), &Node::String("second".to_string()));
                assert_eq!(map.get("true").unwrap(), &Node::String("yes".to_string()));
            }
            _ => panic!("Expected object"),
        }
    }
}
