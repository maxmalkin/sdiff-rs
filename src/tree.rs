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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_name() {
        assert_eq!(Node::Null.type_name(), "null");
        assert_eq!(Node::Bool(true).type_name(), "boolean");
        assert_eq!(Node::Number(42.0).type_name(), "number");
        assert_eq!(Node::String("test".to_string()).type_name(), "string");
        assert_eq!(Node::Object(HashMap::new()).type_name(), "object");
        assert_eq!(Node::Array(vec![]).type_name(), "array");
    }

    #[test]
    fn test_semantic_equals_primitives() {
        // Null
        assert!(Node::Null.semantic_equals(&Node::Null));

        // Boolean
        assert!(Node::Bool(true).semantic_equals(&Node::Bool(true)));
        assert!(Node::Bool(false).semantic_equals(&Node::Bool(false)));
        assert!(!Node::Bool(true).semantic_equals(&Node::Bool(false)));

        // String
        assert!(
            Node::String("hello".to_string()).semantic_equals(&Node::String("hello".to_string()))
        );
        assert!(
            !Node::String("hello".to_string()).semantic_equals(&Node::String("world".to_string()))
        );

        // Different types are not equal
        assert!(!Node::Null.semantic_equals(&Node::Bool(false)));
        assert!(!Node::Bool(true).semantic_equals(&Node::Number(1.0)));
    }

    #[test]
    fn test_semantic_equals_numbers() {
        // Exact equality
        assert!(Node::Number(42.0).semantic_equals(&Node::Number(42.0)));

        // Epsilon tolerance
        assert!(Node::Number(1.0).semantic_equals(&Node::Number(1.0 + 1e-15)));
        assert!(Node::Number(1.0).semantic_equals(&Node::Number(1.0 - 1e-15)));

        // Beyond epsilon
        assert!(!Node::Number(1.0).semantic_equals(&Node::Number(1.1)));
    }

    #[test]
    fn test_semantic_equals_objects() {
        // Empty objects
        assert!(Node::Object(HashMap::new()).semantic_equals(&Node::Object(HashMap::new())));

        // Same content, same order
        let mut obj1 = HashMap::new();
        obj1.insert("a".to_string(), Node::Number(1.0));
        obj1.insert("b".to_string(), Node::Number(2.0));

        let mut obj2 = HashMap::new();
        obj2.insert("a".to_string(), Node::Number(1.0));
        obj2.insert("b".to_string(), Node::Number(2.0));

        assert!(Node::Object(obj1.clone()).semantic_equals(&Node::Object(obj2)));

        // Same content, different order (should still be equal)
        let mut obj3 = HashMap::new();
        obj3.insert("b".to_string(), Node::Number(2.0));
        obj3.insert("a".to_string(), Node::Number(1.0));

        assert!(Node::Object(obj1.clone()).semantic_equals(&Node::Object(obj3)));

        // Different values
        let mut obj4 = HashMap::new();
        obj4.insert("a".to_string(), Node::Number(1.0));
        obj4.insert("b".to_string(), Node::Number(3.0));

        assert!(!Node::Object(obj1.clone()).semantic_equals(&Node::Object(obj4)));

        // Different keys
        let mut obj5 = HashMap::new();
        obj5.insert("a".to_string(), Node::Number(1.0));
        obj5.insert("c".to_string(), Node::Number(2.0));

        assert!(!Node::Object(obj1).semantic_equals(&Node::Object(obj5)));
    }

    #[test]
    fn test_semantic_equals_arrays() {
        // Empty arrays
        assert!(Node::Array(vec![]).semantic_equals(&Node::Array(vec![])));

        // Same content, same order
        let arr1 = vec![Node::Number(1.0), Node::Number(2.0), Node::Number(3.0)];
        let arr2 = vec![Node::Number(1.0), Node::Number(2.0), Node::Number(3.0)];
        assert!(Node::Array(arr1.clone()).semantic_equals(&Node::Array(arr2)));

        // Same content, different order (should NOT be equal for arrays)
        let arr3 = vec![Node::Number(3.0), Node::Number(2.0), Node::Number(1.0)];
        assert!(!Node::Array(arr1.clone()).semantic_equals(&Node::Array(arr3)));

        // Different length
        let arr4 = vec![Node::Number(1.0), Node::Number(2.0)];
        assert!(!Node::Array(arr1.clone()).semantic_equals(&Node::Array(arr4)));

        // Different values
        let arr5 = vec![Node::Number(1.0), Node::Number(5.0), Node::Number(3.0)];
        assert!(!Node::Array(arr1).semantic_equals(&Node::Array(arr5)));
    }

    #[test]
    fn test_semantic_equals_nested() {
        // Nested objects in arrays
        let mut inner1 = HashMap::new();
        inner1.insert("x".to_string(), Node::Number(10.0));

        let mut inner2 = HashMap::new();
        inner2.insert("x".to_string(), Node::Number(10.0));

        let arr1 = vec![Node::Object(inner1)];
        let arr2 = vec![Node::Object(inner2)];
        assert!(Node::Array(arr1).semantic_equals(&Node::Array(arr2)));
    }

    #[test]
    fn test_semantic_equals_edge_cases() {
        // Empty containers
        assert!(Node::Object(HashMap::new()).semantic_equals(&Node::Object(HashMap::new())));
        assert!(Node::Array(vec![]).semantic_equals(&Node::Array(vec![])));

        // Null values in containers
        let arr1 = vec![Node::Null, Node::Null];
        let arr2 = vec![Node::Null, Node::Null];
        assert!(Node::Array(arr1).semantic_equals(&Node::Array(arr2)));

        // Object vs Array (different types)
        assert!(!Node::Object(HashMap::new()).semantic_equals(&Node::Array(vec![])));
    }

    #[test]
    fn test_preview_primitives() {
        assert_eq!(Node::Null.preview(100), "null");
        assert_eq!(Node::Bool(true).preview(100), "true");
        assert_eq!(Node::Bool(false).preview(100), "false");
        assert_eq!(Node::Number(42.0).preview(100), "42");
        assert_eq!(Node::Number(42.5).preview(100), "42.5");
        assert_eq!(Node::String("hello".to_string()).preview(100), "\"hello\"");
    }

    #[test]
    fn test_preview_containers() {
        // Empty containers
        assert_eq!(Node::Object(HashMap::new()).preview(100), "{}");
        assert_eq!(Node::Array(vec![]).preview(100), "[]");

        // Single item
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), Node::Number(1.0));
        assert_eq!(Node::Object(obj).preview(100), "{ 1 key }");

        let arr = vec![Node::Number(1.0)];
        assert_eq!(Node::Array(arr).preview(100), "[ 1 item ]");

        // Multiple items
        let mut obj = HashMap::new();
        obj.insert("a".to_string(), Node::Number(1.0));
        obj.insert("b".to_string(), Node::Number(2.0));
        assert_eq!(Node::Object(obj).preview(100), "{ 2 keys }");

        let arr = vec![Node::Number(1.0), Node::Number(2.0), Node::Number(3.0)];
        assert_eq!(Node::Array(arr).preview(100), "[ 3 items ]");
    }

    #[test]
    fn test_preview_truncation() {
        let long_string = "a".repeat(100);
        let node = Node::String(long_string);
        let preview = node.preview(20);

        // Should be truncated to around 20 chars (+ quotes and ellipsis)
        assert!(preview.len() <= 23);
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn test_size() {
        // Primitives should be relatively small
        assert!(Node::Null.size() > 0);
        assert!(Node::Bool(true).size() > 0);
        assert!(Node::Number(42.0).size() > 0);

        // Strings include their content
        let small = Node::String("hi".to_string());
        let large = Node::String("x".repeat(1000));
        assert!(large.size() > small.size());

        // Objects include keys and values
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), Node::String("value".to_string()));
        let obj_node = Node::Object(obj);
        assert!(obj_node.size() > Node::Null.size());

        // Arrays include all elements
        let arr = vec![Node::Number(1.0), Node::Number(2.0), Node::Number(3.0)];
        let arr_node = Node::Array(arr);
        assert!(arr_node.size() > Node::Number(1.0).size());
    }

    #[test]
    fn test_number_preview_formatting() {
        // Integers should not show decimal point
        assert_eq!(Node::Number(42.0).preview(100), "42");
        assert_eq!(Node::Number(0.0).preview(100), "0");
        assert_eq!(Node::Number(-10.0).preview(100), "-10");

        // Floats should show decimal
        assert_eq!(Node::Number(42.5).preview(100), "42.5");
        assert_eq!(Node::Number(3.25).preview(100), "3.25");
    }
}
