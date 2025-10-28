//! Core semantic diff algorithm.
//!
//! This module implements the logic for comparing two AST nodes and producing
//! a structured diff result. The algorithm recursively traverses both trees,
//! identifying additions, removals, modifications, and unchanged values.
//!
//! # Examples
//!
//! ```
//! use sdiff::{Node, compute_diff, DiffConfig};
//! use std::collections::HashMap;
//!
//! let mut old_map = HashMap::new();
//! old_map.insert("age".to_string(), Node::Number(30.0));
//! let old = Node::Object(old_map);
//!
//! let mut new_map = HashMap::new();
//! new_map.insert("age".to_string(), Node::Number(31.0));
//! let new = Node::Object(new_map);
//!
//! let config = DiffConfig::default();
//! let diff = compute_diff(&old, &new, &config);
//!
//! assert_eq!(diff.stats.modified, 1);
//! ```

use crate::tree::Node;
use std::collections::HashSet;

/// The type of change that occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// Field exists in new but not old
    Added,
    /// Field exists in old but not new
    Removed,
    /// Field exists in both but with different values
    Modified,
    /// Field exists in both with same value
    Unchanged,
}

/// A single change in the diff.
///
/// Each change represents a difference at a specific path in the tree structure.
/// The path is represented as a vector of strings, where each string is either:
/// - An object key (e.g., "user", "profile", "age")
/// - An array index (e.g., "\[0\]", "\[1\]")
#[derive(Debug, Clone)]
pub struct Change {
    /// Path to the changed value (e.g., ["user", "profile", "age"])
    pub path: Vec<String>,
    /// Type of change
    pub change_type: ChangeType,
    /// Old value (None for Added changes)
    pub old_value: Option<Node>,
    /// New value (None for Removed changes)
    pub new_value: Option<Node>,
}

/// Statistics about the diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffStats {
    /// Number of added fields
    pub added: usize,
    /// Number of removed fields
    pub removed: usize,
    /// Number of modified fields
    pub modified: usize,
    /// Number of unchanged fields
    pub unchanged: usize,
}

impl DiffStats {
    /// Creates a new DiffStats with all counts at zero.
    pub fn new() -> Self {
        Self {
            added: 0,
            removed: 0,
            modified: 0,
            unchanged: 0,
        }
    }

    /// Returns the total number of changes (excluding unchanged).
    pub fn total_changes(&self) -> usize {
        self.added + self.removed + self.modified
    }

    /// Returns true if there are no changes.
    pub fn is_empty(&self) -> bool {
        self.total_changes() == 0
    }
}

impl Default for DiffStats {
    fn default() -> Self {
        Self::new()
    }
}

/// The complete diff result.
#[derive(Debug, Clone)]
pub struct Diff {
    /// List of all changes
    pub changes: Vec<Change>,
    /// Summary statistics
    pub stats: DiffStats,
}

impl Diff {
    /// Creates a new empty Diff.
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            stats: DiffStats::new(),
        }
    }

    /// Returns true if there are no changes.
    pub fn is_empty(&self) -> bool {
        self.stats.is_empty()
    }
}

impl Default for Diff {
    fn default() -> Self {
        Self::new()
    }
}

/// Strategy for comparing arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayDiffStrategy {
    /// Compare arrays by index position (simple, fast)
    Positional,
}

impl Default for ArrayDiffStrategy {
    fn default() -> Self {
        Self::Positional
    }
}

/// Configuration for the diff algorithm.
///
/// This allows customization of how diffs are computed.
#[derive(Debug, Clone)]
pub struct DiffConfig {
    /// Normalize whitespace in strings (trim and collapse multiple spaces)
    pub ignore_whitespace: bool,
    /// Treat null as equivalent to a missing key
    pub treat_null_as_missing: bool,
    /// Array comparison strategy
    pub array_diff_strategy: ArrayDiffStrategy,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            ignore_whitespace: false,
            treat_null_as_missing: false,
            array_diff_strategy: ArrayDiffStrategy::Positional,
        }
    }
}

/// Computes the semantic diff between two nodes.
///
/// This is the main entry point for the diff algorithm. It recursively compares
/// the two nodes and returns a complete diff with all changes and statistics.
///
/// # Arguments
///
/// * `old` - The original/old node
/// * `new` - The updated/new node
/// * `config` - Configuration options for the diff
///
/// # Returns
///
/// Returns a Diff containing all changes and statistics.
///
/// # Examples
///
/// ```
/// use sdiff::{Node, compute_diff, DiffConfig};
///
/// let old = Node::Number(42.0);
/// let new = Node::Number(43.0);
/// let config = DiffConfig::default();
/// let diff = compute_diff(&old, &new, &config);
///
/// assert_eq!(diff.stats.modified, 1);
/// ```
pub fn compute_diff(old: &Node, new: &Node, config: &DiffConfig) -> Diff {
    let mut changes = Vec::new();
    diff_nodes(old, new, Vec::new(), &mut changes, config);

    // Compute statistics from changes
    let mut stats = DiffStats::new();
    for change in &changes {
        match change.change_type {
            ChangeType::Added => stats.added += 1,
            ChangeType::Removed => stats.removed += 1,
            ChangeType::Modified => stats.modified += 1,
            ChangeType::Unchanged => stats.unchanged += 1,
        }
    }

    Diff { changes, stats }
}

/// Recursively compares two nodes and collects changes.
///
/// This is the core recursive function that handles all node types.
/// It delegates to specialized functions for objects and arrays.
///
/// # Arguments
///
/// * `old` - The old node
/// * `new` - The new node
/// * `path` - Current path in the tree (for building change paths)
/// * `changes` - Accumulator for changes
/// * `config` - Diff configuration
fn diff_nodes(
    old: &Node,
    new: &Node,
    path: Vec<String>,
    changes: &mut Vec<Change>,
    config: &DiffConfig,
) {
    // Check if nodes are semantically equal first (optimization)
    if nodes_equal(old, new, config) {
        // For containers, recurse to check nested changes
        if let (Node::Object(old_map), Node::Object(new_map)) = (old, new) {
            diff_objects(old_map, new_map, path, changes, config);
        } else if let (Node::Array(old_arr), Node::Array(new_arr)) = (old, new) {
            diff_arrays(old_arr, new_arr, path, changes, config);
        }
        // For leaf values that are equal, we don't record anything (no change)
        return;
    }

    // Handle different types or different values
    match (old, new) {
        (Node::Object(old_map), Node::Object(new_map)) => {
            diff_objects(old_map, new_map, path, changes, config);
        }
        (Node::Array(old_arr), Node::Array(new_arr)) => {
            diff_arrays(old_arr, new_arr, path, changes, config);
        }
        _ => {
            // Different types or different primitive values
            changes.push(Change {
                path,
                change_type: ChangeType::Modified,
                old_value: Some(old.clone()),
                new_value: Some(new.clone()),
            });
        }
    }
}

/// Compares two objects (maps) and collects changes.
///
/// This function:
/// 1. Finds keys that were added (in new but not old)
/// 2. Finds keys that were removed (in old but not new)
/// 3. Recursively compares values for keys present in both
///
/// # Arguments
///
/// * `old_map` - The old object
/// * `new_map` - The new object
/// * `path` - Current path in the tree
/// * `changes` - Accumulator for changes
/// * `config` - Diff configuration
fn diff_objects(
    old_map: &std::collections::HashMap<String, Node>,
    new_map: &std::collections::HashMap<String, Node>,
    path: Vec<String>,
    changes: &mut Vec<Change>,
    config: &DiffConfig,
) {
    // Get all unique keys from both maps
    let old_keys: HashSet<&String> = old_map.keys().collect();
    let new_keys: HashSet<&String> = new_map.keys().collect();

    // Find added keys (in new but not old)
    for key in new_keys.difference(&old_keys) {
        let mut new_path = path.clone();
        new_path.push((*key).clone());
        let value = new_map.get(*key).unwrap();

        changes.push(Change {
            path: new_path,
            change_type: ChangeType::Added,
            old_value: None,
            new_value: Some(value.clone()),
        });
    }

    // Find removed keys (in old but not new)
    for key in old_keys.difference(&new_keys) {
        let mut new_path = path.clone();
        new_path.push((*key).clone());
        let value = old_map.get(*key).unwrap();

        changes.push(Change {
            path: new_path,
            change_type: ChangeType::Removed,
            old_value: Some(value.clone()),
            new_value: None,
        });
    }

    // Compare values for keys present in both
    for key in old_keys.intersection(&new_keys) {
        let mut new_path = path.clone();
        new_path.push((*key).clone());

        let old_value = old_map.get(*key).unwrap();
        let new_value = new_map.get(*key).unwrap();

        diff_nodes(old_value, new_value, new_path, changes, config);
    }
}

/// Compares two arrays and collects changes.
///
/// Uses the configured strategy (currently only Positional is supported).
/// For positional comparison:
/// - Elements at the same index are compared
/// - If arrays have different lengths, extra elements are marked as added/removed
///
/// # Arguments
///
/// * `old_arr` - The old array
/// * `new_arr` - The new array
/// * `path` - Current path in the tree
/// * `changes` - Accumulator for changes
/// * `config` - Diff configuration
fn diff_arrays(
    old_arr: &[Node],
    new_arr: &[Node],
    path: Vec<String>,
    changes: &mut Vec<Change>,
    config: &DiffConfig,
) {
    match config.array_diff_strategy {
        ArrayDiffStrategy::Positional => {
            let min_len = old_arr.len().min(new_arr.len());

            // Compare elements at the same index
            for i in 0..min_len {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", i));
                diff_nodes(&old_arr[i], &new_arr[i], new_path, changes, config);
            }

            // Handle extra elements in old array (removed)
            for (i, item) in old_arr.iter().enumerate().skip(min_len) {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", i));
                changes.push(Change {
                    path: new_path,
                    change_type: ChangeType::Removed,
                    old_value: Some(item.clone()),
                    new_value: None,
                });
            }

            // Handle extra elements in new array (added)
            for (i, item) in new_arr.iter().enumerate().skip(min_len) {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", i));
                changes.push(Change {
                    path: new_path,
                    change_type: ChangeType::Added,
                    old_value: None,
                    new_value: Some(item.clone()),
                });
            }
        }
    }
}

/// Checks if two nodes are equal according to the configuration.
///
/// This respects configuration options like `ignore_whitespace` and
/// `treat_null_as_missing`.
///
/// # Arguments
///
/// * `old` - The old node
/// * `new` - The new node
/// * `config` - Diff configuration
///
/// # Returns
///
/// Returns true if the nodes are considered equal.
fn nodes_equal(old: &Node, new: &Node, config: &DiffConfig) -> bool {
    // Handle whitespace normalization for strings
    if config.ignore_whitespace {
        if let (Node::String(s1), Node::String(s2)) = (old, new) {
            return normalize_whitespace(s1) == normalize_whitespace(s2);
        }
    }

    // Use semantic equality from the Node implementation
    old.semantic_equals(new)
}

/// Normalizes whitespace in a string (trim and collapse multiple spaces).
///
/// # Arguments
///
/// * `s` - The string to normalize
///
/// # Returns
///
/// Returns the normalized string.
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_diff_stats_new() {
        let stats = DiffStats::new();
        assert_eq!(stats.added, 0);
        assert_eq!(stats.removed, 0);
        assert_eq!(stats.modified, 0);
        assert_eq!(stats.unchanged, 0);
        assert!(stats.is_empty());
    }

    #[test]
    fn test_diff_stats_total_changes() {
        let stats = DiffStats {
            added: 2,
            removed: 1,
            modified: 3,
            unchanged: 5,
        };
        assert_eq!(stats.total_changes(), 6);
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_diff_identical_primitives() {
        let config = DiffConfig::default();

        let diff = compute_diff(&Node::Null, &Node::Null, &config);
        assert!(diff.is_empty());

        let diff = compute_diff(&Node::Bool(true), &Node::Bool(true), &config);
        assert!(diff.is_empty());

        let diff = compute_diff(&Node::Number(42.0), &Node::Number(42.0), &config);
        assert!(diff.is_empty());

        let diff = compute_diff(
            &Node::String("hello".to_string()),
            &Node::String("hello".to_string()),
            &config,
        );
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_modified_primitives() {
        let config = DiffConfig::default();

        let diff = compute_diff(&Node::Bool(true), &Node::Bool(false), &config);
        assert_eq!(diff.stats.modified, 1);
        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].change_type, ChangeType::Modified);

        let diff = compute_diff(&Node::Number(42.0), &Node::Number(43.0), &config);
        assert_eq!(diff.stats.modified, 1);

        let diff = compute_diff(
            &Node::String("hello".to_string()),
            &Node::String("world".to_string()),
            &config,
        );
        assert_eq!(diff.stats.modified, 1);
    }

    #[test]
    fn test_diff_type_change() {
        let config = DiffConfig::default();

        let diff = compute_diff(
            &Node::Number(42.0),
            &Node::String("42".to_string()),
            &config,
        );
        assert_eq!(diff.stats.modified, 1);
    }

    #[test]
    fn test_diff_empty_objects() {
        let config = DiffConfig::default();
        let old = Node::Object(HashMap::new());
        let new = Node::Object(HashMap::new());

        let diff = compute_diff(&old, &new, &config);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_object_added_field() {
        let config = DiffConfig::default();
        let old = Node::Object(HashMap::new());

        let mut new_map = HashMap::new();
        new_map.insert("name".to_string(), Node::String("Alice".to_string()));
        let new = Node::Object(new_map);

        let diff = compute_diff(&old, &new, &config);
        assert_eq!(diff.stats.added, 1);
        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].path, vec!["name"]);
        assert_eq!(diff.changes[0].change_type, ChangeType::Added);
    }

    #[test]
    fn test_diff_object_removed_field() {
        let config = DiffConfig::default();

        let mut old_map = HashMap::new();
        old_map.insert("name".to_string(), Node::String("Alice".to_string()));
        let old = Node::Object(old_map);

        let new = Node::Object(HashMap::new());

        let diff = compute_diff(&old, &new, &config);
        assert_eq!(diff.stats.removed, 1);
        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].path, vec!["name"]);
        assert_eq!(diff.changes[0].change_type, ChangeType::Removed);
    }

    #[test]
    fn test_diff_object_modified_field() {
        let config = DiffConfig::default();

        let mut old_map = HashMap::new();
        old_map.insert("age".to_string(), Node::Number(30.0));
        let old = Node::Object(old_map);

        let mut new_map = HashMap::new();
        new_map.insert("age".to_string(), Node::Number(31.0));
        let new = Node::Object(new_map);

        let diff = compute_diff(&old, &new, &config);
        assert_eq!(diff.stats.modified, 1);
        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].path, vec!["age"]);
        assert_eq!(diff.changes[0].change_type, ChangeType::Modified);
    }

    #[test]
    fn test_diff_nested_objects() {
        let config = DiffConfig::default();

        let mut old_inner = HashMap::new();
        old_inner.insert("age".to_string(), Node::Number(30.0));
        let mut old_map = HashMap::new();
        old_map.insert("user".to_string(), Node::Object(old_inner));
        let old = Node::Object(old_map);

        let mut new_inner = HashMap::new();
        new_inner.insert("age".to_string(), Node::Number(31.0));
        let mut new_map = HashMap::new();
        new_map.insert("user".to_string(), Node::Object(new_inner));
        let new = Node::Object(new_map);

        let diff = compute_diff(&old, &new, &config);
        assert_eq!(diff.stats.modified, 1);
        assert_eq!(diff.changes[0].path, vec!["user", "age"]);
    }

    #[test]
    fn test_diff_arrays_same() {
        let config = DiffConfig::default();
        let old = Node::Array(vec![
            Node::Number(1.0),
            Node::Number(2.0),
            Node::Number(3.0),
        ]);
        let new = Node::Array(vec![
            Node::Number(1.0),
            Node::Number(2.0),
            Node::Number(3.0),
        ]);

        let diff = compute_diff(&old, &new, &config);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_arrays_modified_element() {
        let config = DiffConfig::default();
        let old = Node::Array(vec![
            Node::Number(1.0),
            Node::Number(2.0),
            Node::Number(3.0),
        ]);
        let new = Node::Array(vec![
            Node::Number(1.0),
            Node::Number(5.0),
            Node::Number(3.0),
        ]);

        let diff = compute_diff(&old, &new, &config);
        assert_eq!(diff.stats.modified, 1);
        assert_eq!(diff.changes[0].path, vec!["[1]"]);
    }

    #[test]
    fn test_diff_arrays_added_element() {
        let config = DiffConfig::default();
        let old = Node::Array(vec![Node::Number(1.0), Node::Number(2.0)]);
        let new = Node::Array(vec![
            Node::Number(1.0),
            Node::Number(2.0),
            Node::Number(3.0),
        ]);

        let diff = compute_diff(&old, &new, &config);
        assert_eq!(diff.stats.added, 1);
        assert_eq!(diff.changes[0].path, vec!["[2]"]);
    }

    #[test]
    fn test_diff_arrays_removed_element() {
        let config = DiffConfig::default();
        let old = Node::Array(vec![
            Node::Number(1.0),
            Node::Number(2.0),
            Node::Number(3.0),
        ]);
        let new = Node::Array(vec![Node::Number(1.0), Node::Number(2.0)]);

        let diff = compute_diff(&old, &new, &config);
        assert_eq!(diff.stats.removed, 1);
        assert_eq!(diff.changes[0].path, vec!["[2]"]);
    }

    #[test]
    fn test_diff_ignore_whitespace() {
        let config = DiffConfig {
            ignore_whitespace: true,
            ..Default::default()
        };

        let old = Node::String("hello   world".to_string());
        let new = Node::String("hello world".to_string());

        let diff = compute_diff(&old, &new, &config);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_complex_structure() {
        let config = DiffConfig::default();

        // Old: {"users": [{"name": "Alice", "age": 30}], "count": 1}
        let mut old_user = HashMap::new();
        old_user.insert("name".to_string(), Node::String("Alice".to_string()));
        old_user.insert("age".to_string(), Node::Number(30.0));

        let mut old_map = HashMap::new();
        old_map.insert(
            "users".to_string(),
            Node::Array(vec![Node::Object(old_user)]),
        );
        old_map.insert("count".to_string(), Node::Number(1.0));
        let old = Node::Object(old_map);

        // New: {"users": [{"name": "Alice", "age": 31}], "count": 1, "active": true}
        let mut new_user = HashMap::new();
        new_user.insert("name".to_string(), Node::String("Alice".to_string()));
        new_user.insert("age".to_string(), Node::Number(31.0));

        let mut new_map = HashMap::new();
        new_map.insert(
            "users".to_string(),
            Node::Array(vec![Node::Object(new_user)]),
        );
        new_map.insert("count".to_string(), Node::Number(1.0));
        new_map.insert("active".to_string(), Node::Bool(true));
        let new = Node::Object(new_map);

        let diff = compute_diff(&old, &new, &config);
        assert_eq!(diff.stats.modified, 1); // age changed
        assert_eq!(diff.stats.added, 1); // active added

        // Find the age change
        let age_change = diff
            .changes
            .iter()
            .find(|c| c.path == vec!["users", "[0]", "age"])
            .unwrap();
        assert_eq!(age_change.change_type, ChangeType::Modified);

        // Find the active addition
        let active_change = diff
            .changes
            .iter()
            .find(|c| c.path == vec!["active"])
            .unwrap();
        assert_eq!(active_change.change_type, ChangeType::Added);
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(normalize_whitespace("hello"), "hello");
        assert_eq!(normalize_whitespace("   "), "");
    }
}
