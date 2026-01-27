//! Abstract Syntax Tree representation for structured data.

use std::collections::HashMap;

/// A node representing a value in structured data (JSON, YAML, TOML).
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Object(HashMap<String, Node>),
    Array(Vec<Node>),
}

impl Node {
    pub fn type_name(&self) -> &str {
        match self {
            Node::Null => "null",
            Node::Bool(_) => "boolean",
            Node::Number(_) => "number",
            Node::String(_) => "string",
            Node::Object(_) => "object",
            Node::Array(_) => "array",
        }
    }

    /// Checks if two nodes are semantically equal (ignores key ordering, uses epsilon for floats).
    pub fn semantic_equals(&self, other: &Node) -> bool {
        match (self, other) {
            (Node::Null, Node::Null) => true,
            (Node::Bool(a), Node::Bool(b)) => a == b,
            (Node::String(a), Node::String(b)) => a == b,
            (Node::Number(a), Node::Number(b)) => {
                const EPSILON: f64 = 1e-10;
                (a - b).abs() < EPSILON
            }
            (Node::Object(a), Node::Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter()
                    .all(|(key, value)| b.get(key).is_some_and(|v| value.semantic_equals(v)))
            }
            (Node::Array(a), Node::Array(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|(item_a, item_b)| item_a.semantic_equals(item_b))
            }
            _ => false,
        }
    }

    /// Returns a short preview of the node's value, truncated to max_len.
    pub fn preview(&self, max_len: usize) -> String {
        let preview = match self {
            Node::Null => "null".to_string(),
            Node::Bool(b) => b.to_string(),
            Node::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            Node::String(s) => format!("\"{}\"", s),
            Node::Object(map) => {
                let count = map.len();
                if count == 0 {
                    "{}".to_string()
                } else if count == 1 {
                    format!("{{ {} key }}", count)
                } else {
                    format!("{{ {} keys }}", count)
                }
            }
            Node::Array(arr) => {
                let count = arr.len();
                if count == 0 {
                    "[]".to_string()
                } else if count == 1 {
                    format!("[ {} item ]", count)
                } else {
                    format!("[ {} items ]", count)
                }
            }
        };

        if preview.len() > max_len {
            format!("{}...", &preview[..max_len.saturating_sub(3)])
        } else {
            preview
        }
    }

    /// Returns an approximate size in bytes for this node.
    pub fn size(&self) -> usize {
        match self {
            Node::Null => std::mem::size_of::<Node>(),
            Node::Bool(_) => std::mem::size_of::<Node>(),
            Node::Number(_) => std::mem::size_of::<Node>(),
            Node::String(s) => std::mem::size_of::<Node>() + s.len(),
            Node::Object(map) => {
                let base =
                    std::mem::size_of::<Node>() + std::mem::size_of::<HashMap<String, Node>>();
                let entries: usize = map.iter().map(|(k, v)| k.len() + v.size()).sum();
                base + entries
            }
            Node::Array(arr) => {
                let base = std::mem::size_of::<Node>() + std::mem::size_of::<Vec<Node>>();
                let elements: usize = arr.iter().map(|n| n.size()).sum();
                base + elements
            }
        }
    }
}
