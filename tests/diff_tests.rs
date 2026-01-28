use sdiff_rs::{compute_diff, ArrayDiffStrategy, ChangeType, DiffConfig, Node};
use std::collections::HashMap;

#[test]
fn test_diff_stats_new() {
    let config = DiffConfig::default();
    let diff = compute_diff(&Node::Null, &Node::Null, &config);
    assert!(diff.is_empty());
}

#[test]
fn test_diff_stats_total_changes() {
    let config = DiffConfig::default();
    let diff = compute_diff(&Node::Bool(true), &Node::Bool(false), &config);
    assert_eq!(diff.stats.total_changes(), 1);
    assert!(!diff.stats.is_empty());
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
    assert_eq!(diff.stats.modified, 1);
    assert_eq!(diff.stats.added, 1);

    let age_change = diff
        .changes
        .iter()
        .find(|c| c.path == vec!["users", "[0]", "age"])
        .unwrap();
    assert_eq!(age_change.change_type, ChangeType::Modified);

    let active_change = diff
        .changes
        .iter()
        .find(|c| c.path == vec!["active"])
        .unwrap();
    assert_eq!(active_change.change_type, ChangeType::Added);
}

#[test]
fn test_lcs_basic() {
    let config = DiffConfig {
        array_diff_strategy: ArrayDiffStrategy::Lcs,
        ..Default::default()
    };

    let old = Node::Array(vec![
        Node::Number(1.0),
        Node::Number(2.0),
        Node::Number(3.0),
    ]);
    let new = Node::Array(vec![
        Node::Number(1.0),
        Node::Number(4.0),
        Node::Number(2.0),
        Node::Number(3.0),
    ]);

    let diff = compute_diff(&old, &new, &config);
    assert_eq!(diff.stats.added, 1);
    assert_eq!(diff.stats.removed, 0);
    assert_eq!(diff.stats.modified, 0);

    let added = diff
        .changes
        .iter()
        .find(|c| c.change_type == ChangeType::Added)
        .unwrap();
    assert_eq!(added.new_value, Some(Node::Number(4.0)));
}

#[test]
fn test_lcs_deletion() {
    let config = DiffConfig {
        array_diff_strategy: ArrayDiffStrategy::Lcs,
        ..Default::default()
    };

    let old = Node::Array(vec![
        Node::Number(1.0),
        Node::Number(2.0),
        Node::Number(3.0),
        Node::Number(4.0),
    ]);
    let new = Node::Array(vec![
        Node::Number(1.0),
        Node::Number(3.0),
        Node::Number(4.0),
    ]);

    let diff = compute_diff(&old, &new, &config);
    assert_eq!(diff.stats.removed, 1);
    assert_eq!(diff.stats.added, 0);
    assert_eq!(diff.stats.modified, 0);

    let removed = diff
        .changes
        .iter()
        .find(|c| c.change_type == ChangeType::Removed)
        .unwrap();
    assert_eq!(removed.old_value, Some(Node::Number(2.0)));
}

#[test]
fn test_lcs_reorder() {
    let config = DiffConfig {
        array_diff_strategy: ArrayDiffStrategy::Lcs,
        ..Default::default()
    };

    let old = Node::Array(vec![
        Node::Number(1.0),
        Node::Number(2.0),
        Node::Number(3.0),
    ]);
    let new = Node::Array(vec![
        Node::Number(3.0),
        Node::Number(1.0),
        Node::Number(2.0),
    ]);

    let diff = compute_diff(&old, &new, &config);
    assert_eq!(diff.stats.added, 1);
    assert_eq!(diff.stats.removed, 1);
}

#[test]
fn test_lcs_nested_objects() {
    let config = DiffConfig {
        array_diff_strategy: ArrayDiffStrategy::Lcs,
        ..Default::default()
    };

    let mut obj1_old = HashMap::new();
    obj1_old.insert("id".to_string(), Node::Number(1.0));
    obj1_old.insert("name".to_string(), Node::String("Alice".to_string()));

    let mut obj2 = HashMap::new();
    obj2.insert("id".to_string(), Node::Number(2.0));
    obj2.insert("name".to_string(), Node::String("Bob".to_string()));

    let mut obj1_new = HashMap::new();
    obj1_new.insert("id".to_string(), Node::Number(1.0));
    obj1_new.insert("name".to_string(), Node::String("Alicia".to_string()));

    let old = Node::Array(vec![Node::Object(obj1_old), Node::Object(obj2.clone())]);
    let new = Node::Array(vec![Node::Object(obj1_new), Node::Object(obj2)]);

    let diff = compute_diff(&old, &new, &config);
    assert_eq!(diff.stats.added, 1);
    assert_eq!(diff.stats.removed, 1);
}

#[test]
fn test_lcs_with_identical_objects() {
    let config = DiffConfig {
        array_diff_strategy: ArrayDiffStrategy::Lcs,
        ..Default::default()
    };

    let mut obj1 = HashMap::new();
    obj1.insert("id".to_string(), Node::Number(1.0));

    let mut obj2 = HashMap::new();
    obj2.insert("id".to_string(), Node::Number(2.0));

    let mut obj3 = HashMap::new();
    obj3.insert("id".to_string(), Node::Number(3.0));

    let old = Node::Array(vec![Node::Object(obj1.clone()), Node::Object(obj2.clone())]);
    let new = Node::Array(vec![
        Node::Object(obj1),
        Node::Object(obj3),
        Node::Object(obj2),
    ]);

    let diff = compute_diff(&old, &new, &config);
    assert_eq!(diff.stats.added, 1);
    assert_eq!(diff.stats.removed, 0);
    assert_eq!(diff.stats.modified, 0);
}

#[test]
fn test_lcs_empty_arrays() {
    let config = DiffConfig {
        array_diff_strategy: ArrayDiffStrategy::Lcs,
        ..Default::default()
    };

    let empty = Node::Array(vec![]);
    let non_empty = Node::Array(vec![Node::Number(1.0), Node::Number(2.0)]);

    let diff = compute_diff(&empty, &non_empty, &config);
    assert_eq!(diff.stats.added, 2);
    assert_eq!(diff.stats.removed, 0);

    let diff = compute_diff(&non_empty, &empty, &config);
    assert_eq!(diff.stats.removed, 2);
    assert_eq!(diff.stats.added, 0);
}

#[test]
fn test_lcs_vs_positional_comparison() {
    let old = Node::Array(vec![
        Node::Number(1.0),
        Node::Number(2.0),
        Node::Number(3.0),
    ]);
    let new = Node::Array(vec![
        Node::Number(1.0),
        Node::Number(4.0),
        Node::Number(2.0),
        Node::Number(3.0),
    ]);

    let positional_config = DiffConfig {
        array_diff_strategy: ArrayDiffStrategy::Positional,
        ..Default::default()
    };
    let lcs_config = DiffConfig {
        array_diff_strategy: ArrayDiffStrategy::Lcs,
        ..Default::default()
    };

    let positional_diff = compute_diff(&old, &new, &positional_config);
    let lcs_diff = compute_diff(&old, &new, &lcs_config);

    assert_eq!(positional_diff.stats.modified, 2);
    assert_eq!(positional_diff.stats.added, 1);
    assert_eq!(positional_diff.stats.removed, 0);

    assert_eq!(lcs_diff.stats.modified, 0);
    assert_eq!(lcs_diff.stats.added, 1);
    assert_eq!(lcs_diff.stats.removed, 0);
}
