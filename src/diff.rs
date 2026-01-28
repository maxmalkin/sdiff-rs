//! Core semantic diff algorithm.
//!
//! This module implements the logic for comparing two AST nodes and producing
//! a structured diff result. The algorithm recursively traverses both trees,
//! identifying additions, removals, modifications, and unchanged values.
//!
//! # Examples
//!
//! ```
//! use sdiff_rs::{Node, compute_diff, DiffConfig};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrayDiffStrategy {
    /// Compare arrays by index position (simple, fast)
    #[default]
    Positional,
    /// Use Longest Common Subsequence algorithm to detect insertions and deletions
    Lcs,
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
/// use sdiff_rs::{Node, compute_diff, DiffConfig};
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

fn diff_nodes(
    old: &Node,
    new: &Node,
    path: Vec<String>,
    changes: &mut Vec<Change>,
    config: &DiffConfig,
) {
    if nodes_equal(old, new, config) {
        if let (Node::Object(old_map), Node::Object(new_map)) = (old, new) {
            diff_objects(old_map, new_map, path, changes, config);
        } else if let (Node::Array(old_arr), Node::Array(new_arr)) = (old, new) {
            diff_arrays(old_arr, new_arr, path, changes, config);
        }
        return;
    }

    match (old, new) {
        (Node::Object(old_map), Node::Object(new_map)) => {
            diff_objects(old_map, new_map, path, changes, config);
        }
        (Node::Array(old_arr), Node::Array(new_arr)) => {
            diff_arrays(old_arr, new_arr, path, changes, config);
        }
        _ => {
            changes.push(Change {
                path,
                change_type: ChangeType::Modified,
                old_value: Some(old.clone()),
                new_value: Some(new.clone()),
            });
        }
    }
}

fn diff_objects(
    old_map: &std::collections::HashMap<String, Node>,
    new_map: &std::collections::HashMap<String, Node>,
    path: Vec<String>,
    changes: &mut Vec<Change>,
    config: &DiffConfig,
) {
    let old_keys: HashSet<&String> = old_map.keys().collect();
    let new_keys: HashSet<&String> = new_map.keys().collect();

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

    for key in old_keys.intersection(&new_keys) {
        let mut new_path = path.clone();
        new_path.push((*key).clone());

        let old_value = old_map.get(*key).unwrap();
        let new_value = new_map.get(*key).unwrap();

        diff_nodes(old_value, new_value, new_path, changes, config);
    }
}

fn diff_arrays(
    old_arr: &[Node],
    new_arr: &[Node],
    path: Vec<String>,
    changes: &mut Vec<Change>,
    config: &DiffConfig,
) {
    match config.array_diff_strategy {
        ArrayDiffStrategy::Positional => {
            diff_arrays_positional(old_arr, new_arr, path, changes, config);
        }
        ArrayDiffStrategy::Lcs => {
            diff_arrays_lcs(old_arr, new_arr, path, changes, config);
        }
    }
}

fn diff_arrays_positional(
    old_arr: &[Node],
    new_arr: &[Node],
    path: Vec<String>,
    changes: &mut Vec<Change>,
    config: &DiffConfig,
) {
    let min_len = old_arr.len().min(new_arr.len());

    for i in 0..min_len {
        let mut new_path = path.clone();
        new_path.push(format!("[{}]", i));
        diff_nodes(&old_arr[i], &new_arr[i], new_path, changes, config);
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
enum EditOp {
    Keep(usize, usize),
    Delete(usize),
    Insert(usize),
}

fn compute_lcs_edits(old: &[Node], new: &[Node], config: &DiffConfig) -> Vec<EditOp> {
    let n = old.len();
    let m = new.len();

    let mut dp = vec![vec![0usize; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            if nodes_equal(&old[i - 1], &new[j - 1], config) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut edits = Vec::new();
    let mut i = n;
    let mut j = m;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && nodes_equal(&old[i - 1], &new[j - 1], config) {
            edits.push(EditOp::Keep(i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            edits.push(EditOp::Insert(j - 1));
            j -= 1;
        } else {
            edits.push(EditOp::Delete(i - 1));
            i -= 1;
        }
    }

    edits.reverse();
    edits
}

fn diff_arrays_lcs(
    old_arr: &[Node],
    new_arr: &[Node],
    path: Vec<String>,
    changes: &mut Vec<Change>,
    config: &DiffConfig,
) {
    let edits = compute_lcs_edits(old_arr, new_arr, config);

    let mut new_idx = 0;

    for edit in edits {
        match edit {
            EditOp::Keep(old_idx, new_i) => {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", new_i));
                diff_nodes(
                    &old_arr[old_idx],
                    &new_arr[new_i],
                    new_path,
                    changes,
                    config,
                );
                new_idx = new_i + 1;
            }
            EditOp::Delete(old_idx) => {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", new_idx));
                changes.push(Change {
                    path: new_path,
                    change_type: ChangeType::Removed,
                    old_value: Some(old_arr[old_idx].clone()),
                    new_value: None,
                });
            }
            EditOp::Insert(new_i) => {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", new_i));
                changes.push(Change {
                    path: new_path,
                    change_type: ChangeType::Added,
                    old_value: None,
                    new_value: Some(new_arr[new_i].clone()),
                });
                new_idx = new_i + 1;
            }
        }
    }
}

fn nodes_equal(old: &Node, new: &Node, config: &DiffConfig) -> bool {
    if config.ignore_whitespace {
        if let (Node::String(s1), Node::String(s2)) = (old, new) {
            return normalize_whitespace(s1) == normalize_whitespace(s2);
        }
    }

    old.semantic_equals(new)
}

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
