use sdiff_rs::diff::{Change, ChangeType, Diff, DiffStats};
use sdiff_rs::filter::{filter_diff, FilterConfig, PathPattern, PatternSegment};
use sdiff_rs::Node;

#[test]
fn test_pattern_parse_literal() {
    let pattern = PathPattern::parse("foo.bar.baz");
    assert_eq!(pattern.segments.len(), 3);
    assert_eq!(
        pattern.segments[0],
        PatternSegment::Literal("foo".to_string())
    );
    assert_eq!(
        pattern.segments[1],
        PatternSegment::Literal("bar".to_string())
    );
    assert_eq!(
        pattern.segments[2],
        PatternSegment::Literal("baz".to_string())
    );
}

#[test]
fn test_pattern_parse_wildcards() {
    let pattern = PathPattern::parse("**.foo.*");
    assert_eq!(pattern.segments.len(), 3);
    assert_eq!(pattern.segments[0], PatternSegment::DoubleWildcard);
    assert_eq!(
        pattern.segments[1],
        PatternSegment::Literal("foo".to_string())
    );
    assert_eq!(pattern.segments[2], PatternSegment::SingleWildcard);
}

#[test]
fn test_pattern_matches_literal() {
    let pattern = PathPattern::parse("foo.bar");
    assert!(pattern.matches(&["foo".to_string(), "bar".to_string()]));
    assert!(!pattern.matches(&["foo".to_string(), "baz".to_string()]));
    assert!(!pattern.matches(&["foo".to_string()]));
    assert!(!pattern.matches(&["foo".to_string(), "bar".to_string(), "baz".to_string()]));
}

#[test]
fn test_pattern_matches_single_wildcard() {
    let pattern = PathPattern::parse("foo.*.baz");
    assert!(pattern.matches(&["foo".to_string(), "bar".to_string(), "baz".to_string()]));
    assert!(pattern.matches(&["foo".to_string(), "anything".to_string(), "baz".to_string()]));
    assert!(!pattern.matches(&["foo".to_string(), "baz".to_string()]));
}

#[test]
fn test_pattern_matches_double_wildcard() {
    let pattern = PathPattern::parse("**.version");
    assert!(pattern.matches(&["version".to_string()]));
    assert!(pattern.matches(&["package".to_string(), "version".to_string()]));
    assert!(pattern.matches(&[
        "deep".to_string(),
        "nested".to_string(),
        "version".to_string()
    ]));
    assert!(!pattern.matches(&["package".to_string(), "name".to_string()]));
}

#[test]
fn test_pattern_matches_double_wildcard_prefix() {
    let pattern = PathPattern::parse("metadata.**");
    assert!(pattern.matches(&["metadata".to_string()]));
    assert!(pattern.matches(&["metadata".to_string(), "foo".to_string()]));
    assert!(pattern.matches(&["metadata".to_string(), "foo".to_string(), "bar".to_string()]));
    assert!(!pattern.matches(&["other".to_string(), "metadata".to_string()]));
}

#[test]
fn test_filter_config_ignore() {
    let config = FilterConfig::new()
        .ignore("metadata.timestamp")
        .ignore("**.internal");

    assert!(!config.should_include(&["metadata".to_string(), "timestamp".to_string()]));
    assert!(!config.should_include(&["foo".to_string(), "internal".to_string()]));
    assert!(config.should_include(&["metadata".to_string(), "author".to_string()]));
    assert!(config.should_include(&["data".to_string(), "value".to_string()]));
}

#[test]
fn test_filter_config_only() {
    let config = FilterConfig::new().only("spec.**").only("metadata.name");

    assert!(config.should_include(&["spec".to_string(), "replicas".to_string()]));
    assert!(config.should_include(&["metadata".to_string(), "name".to_string()]));
    assert!(!config.should_include(&["metadata".to_string(), "timestamp".to_string()]));
    assert!(!config.should_include(&["status".to_string()]));
}

#[test]
fn test_filter_config_combined() {
    let config = FilterConfig::new().only("spec.**").ignore("spec.internal");

    assert!(config.should_include(&["spec".to_string(), "replicas".to_string()]));
    assert!(!config.should_include(&["spec".to_string(), "internal".to_string()]));
    assert!(!config.should_include(&["metadata".to_string()]));
}

#[test]
fn test_filter_diff() {
    let changes = vec![
        Change {
            path: vec!["metadata".to_string(), "timestamp".to_string()],
            change_type: ChangeType::Modified,
            old_value: Some(Node::String("old".to_string())),
            new_value: Some(Node::String("new".to_string())),
        },
        Change {
            path: vec!["spec".to_string(), "replicas".to_string()],
            change_type: ChangeType::Modified,
            old_value: Some(Node::Number(1.0)),
            new_value: Some(Node::Number(2.0)),
        },
        Change {
            path: vec!["data".to_string(), "value".to_string()],
            change_type: ChangeType::Added,
            old_value: None,
            new_value: Some(Node::String("added".to_string())),
        },
    ];

    let diff = Diff {
        changes,
        stats: DiffStats {
            added: 1,
            removed: 0,
            modified: 2,
            unchanged: 0,
        },
    };

    let config = FilterConfig::new().ignore("metadata.**");
    let filtered = filter_diff(&diff, &config);

    assert_eq!(filtered.changes.len(), 2);
    assert_eq!(filtered.stats.added, 1);
    assert_eq!(filtered.stats.modified, 1);
}
