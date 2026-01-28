use sdiff_rs::diff::{Change, ChangeType, Diff, DiffStats};
use sdiff_rs::output::{format_diff, OutputFormat, OutputOptions};
use sdiff_rs::Node;

#[test]
fn test_format_plain_no_changes() {
    let diff = Diff {
        changes: vec![],
        stats: DiffStats::new(),
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &OutputOptions::default()).unwrap();
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
    let output = format_diff(&diff, &OutputFormat::Plain, &OutputOptions::default()).unwrap();
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
    let output = format_diff(&diff, &OutputFormat::Json, &OutputOptions::default()).unwrap();
    assert!(output.contains("\"age\""));
    assert!(output.contains("30"));
    assert!(output.contains("31"));
    assert!(output.contains("\"modified\""));
    assert!(output.contains("\"stats\""));
}

#[test]
fn test_format_terminal_no_changes() {
    let diff = Diff {
        changes: vec![],
        stats: DiffStats::new(),
    };
    let output = format_diff(&diff, &OutputFormat::Terminal, &OutputOptions::default()).unwrap();
    assert!(output.contains("No changes"));
}

#[test]
fn test_format_change_types() {
    let options = OutputOptions::default();

    let added = Change {
        path: vec!["new_field".to_string()],
        change_type: ChangeType::Added,
        old_value: None,
        new_value: Some(Node::String("value".to_string())),
    };
    let diff = Diff {
        changes: vec![added],
        stats: DiffStats {
            added: 1,
            removed: 0,
            modified: 0,
            unchanged: 0,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &options).unwrap();
    assert!(output.starts_with('+'));
    assert!(output.contains("new_field"));

    let removed = Change {
        path: vec!["old_field".to_string()],
        change_type: ChangeType::Removed,
        old_value: Some(Node::String("value".to_string())),
        new_value: None,
    };
    let diff = Diff {
        changes: vec![removed],
        stats: DiffStats {
            added: 0,
            removed: 1,
            modified: 0,
            unchanged: 0,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &options).unwrap();
    assert!(output.starts_with('-'));
    assert!(output.contains("old_field"));

    let modified = Change {
        path: vec!["field".to_string()],
        change_type: ChangeType::Modified,
        old_value: Some(Node::Number(1.0)),
        new_value: Some(Node::Number(2.0)),
    };
    let diff = Diff {
        changes: vec![modified],
        stats: DiffStats {
            added: 0,
            removed: 0,
            modified: 1,
            unchanged: 0,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &options).unwrap();
    assert!(output.contains("→"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

#[test]
fn test_format_summary_empty() {
    let diff = Diff {
        changes: vec![],
        stats: DiffStats::new(),
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &OutputOptions::default()).unwrap();
    assert!(output.contains("No changes"));
}

#[test]
fn test_format_summary_with_changes() {
    let diff = Diff {
        changes: vec![
            Change {
                path: vec!["a".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::Null),
            },
            Change {
                path: vec!["a".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::Null),
            },
            Change {
                path: vec!["b".to_string()],
                change_type: ChangeType::Removed,
                old_value: Some(Node::Null),
                new_value: None,
            },
            Change {
                path: vec!["c".to_string()],
                change_type: ChangeType::Modified,
                old_value: Some(Node::Number(1.0)),
                new_value: Some(Node::Number(2.0)),
            },
            Change {
                path: vec!["c".to_string()],
                change_type: ChangeType::Modified,
                old_value: Some(Node::Number(1.0)),
                new_value: Some(Node::Number(2.0)),
            },
            Change {
                path: vec!["c".to_string()],
                change_type: ChangeType::Modified,
                old_value: Some(Node::Number(1.0)),
                new_value: Some(Node::Number(2.0)),
            },
        ],
        stats: DiffStats {
            added: 2,
            removed: 1,
            modified: 3,
            unchanged: 5,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &OutputOptions::default()).unwrap();
    assert!(output.contains("2 added"));
    assert!(output.contains("1 removed"));
    assert!(output.contains("3 modified"));
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
    let diff = Diff {
        changes: vec![added],
        stats: DiffStats {
            added: 1,
            removed: 0,
            modified: 0,
            unchanged: 0,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &options).unwrap();
    assert!(output.contains("test"));

    let unchanged = Change {
        path: vec!["unchanged".to_string()],
        change_type: ChangeType::Unchanged,
        old_value: Some(Node::Null),
        new_value: Some(Node::Null),
    };
    let diff = Diff {
        changes: vec![unchanged],
        stats: DiffStats {
            added: 0,
            removed: 0,
            modified: 0,
            unchanged: 1,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &options).unwrap();
    assert!(!output.contains("unchanged"));
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
    let diff = Diff {
        changes: vec![unchanged],
        stats: DiffStats {
            added: 0,
            removed: 0,
            modified: 0,
            unchanged: 1,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &options).unwrap();
    assert!(output.contains("test"));
}

#[test]
fn test_node_to_json_value() {
    let diff = Diff {
        changes: vec![
            Change {
                path: vec!["null".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::Null),
            },
            Change {
                path: vec!["bool".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::Bool(true)),
            },
            Change {
                path: vec!["number".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::Number(42.0)),
            },
            Change {
                path: vec!["string".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::String("test".to_string())),
            },
            Change {
                path: vec!["array".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::Array(vec![Node::Number(1.0), Node::Number(2.0)])),
            },
        ],
        stats: DiffStats {
            added: 5,
            removed: 0,
            modified: 0,
            unchanged: 0,
        },
    };

    let output = format_diff(&diff, &OutputFormat::Json, &OutputOptions::default()).unwrap();
    assert!(output.contains("null"));
    assert!(output.contains("true"));
    assert!(output.contains("42"));
    assert!(output.contains("\"test\""));
    assert!(output.contains("["));
}

#[test]
fn test_format_path_simple() {
    let change = Change {
        path: vec!["user".to_string(), "name".to_string()],
        change_type: ChangeType::Modified,
        old_value: Some(Node::String("old".to_string())),
        new_value: Some(Node::String("new".to_string())),
    };
    let diff = Diff {
        changes: vec![change],
        stats: DiffStats {
            added: 0,
            removed: 0,
            modified: 1,
            unchanged: 0,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &OutputOptions::default()).unwrap();
    assert!(output.contains("user.name"));
}

#[test]
fn test_format_path_array() {
    let change = Change {
        path: vec!["items".to_string(), "[0]".to_string(), "id".to_string()],
        change_type: ChangeType::Modified,
        old_value: Some(Node::Number(1.0)),
        new_value: Some(Node::Number(2.0)),
    };
    let diff = Diff {
        changes: vec![change],
        stats: DiffStats {
            added: 0,
            removed: 0,
            modified: 1,
            unchanged: 0,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &OutputOptions::default()).unwrap();
    assert!(output.contains("items[0].id"));
}

#[test]
fn test_format_path_root() {
    let change = Change {
        path: vec![],
        change_type: ChangeType::Modified,
        old_value: Some(Node::Number(1.0)),
        new_value: Some(Node::Number(2.0)),
    };
    let diff = Diff {
        changes: vec![change],
        stats: DiffStats {
            added: 0,
            removed: 0,
            modified: 1,
            unchanged: 0,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &OutputOptions::default()).unwrap();
    assert!(output.contains("(root)"));
}

#[test]
fn test_format_value_primitives() {
    let diff = Diff {
        changes: vec![
            Change {
                path: vec!["null".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::Null),
            },
            Change {
                path: vec!["bool".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::Bool(true)),
            },
            Change {
                path: vec!["num".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::Number(42.0)),
            },
            Change {
                path: vec!["str".to_string()],
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(Node::String("hello".to_string())),
            },
        ],
        stats: DiffStats {
            added: 4,
            removed: 0,
            modified: 0,
            unchanged: 0,
        },
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &OutputOptions::default()).unwrap();
    assert!(output.contains("null"));
    assert!(output.contains("true"));
    assert!(output.contains("42"));
    assert!(output.contains("\"hello\""));
}

#[test]
fn test_format_value_truncation() {
    let long_string = "a".repeat(100);
    let diff = Diff {
        changes: vec![Change {
            path: vec!["long".to_string()],
            change_type: ChangeType::Added,
            old_value: None,
            new_value: Some(Node::String(long_string)),
        }],
        stats: DiffStats {
            added: 1,
            removed: 0,
            modified: 0,
            unchanged: 0,
        },
    };
    let options = OutputOptions {
        max_value_length: 20,
        ..Default::default()
    };
    let output = format_diff(&diff, &OutputFormat::Plain, &options).unwrap();
    assert!(output.contains("..."));
}
