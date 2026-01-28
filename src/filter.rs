//! Path filtering for diff results.
//!
//! This module provides glob-style pattern matching to filter diff results,
//! allowing users to ignore specific paths or focus on particular areas.
//!
//! # Pattern Syntax
//!
//! - `foo` - matches literal segment "foo"
//! - `*` - matches any single path segment
//! - `**` - matches any number of path segments (including zero)
//! - `foo.bar` - matches nested path "foo.bar"
//! - `**.version` - matches "version" at any depth
//!
//! # Examples
//!
//! ```
//! use sdiff_rs::filter::{PathPattern, FilterConfig};
//!
//! let pattern = PathPattern::parse("metadata.timestamp");
//! assert!(pattern.matches(&["metadata".to_string(), "timestamp".to_string()]));
//!
//! let pattern = PathPattern::parse("**.version");
//! assert!(pattern.matches(&["package".to_string(), "version".to_string()]));
//! assert!(pattern.matches(&["dependencies".to_string(), "foo".to_string(), "version".to_string()]));
//! ```

use crate::diff::{Change, Diff, DiffStats};

/// A single segment in a path pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternSegment {
    /// Matches an exact string
    Literal(String),
    /// Matches any single path segment (*)
    SingleWildcard,
    /// Matches any number of path segments (**)
    DoubleWildcard,
}

/// A compiled path pattern for matching against diff paths.
#[derive(Debug, Clone)]
pub struct PathPattern {
    pub segments: Vec<PatternSegment>,
}

impl PathPattern {
    pub fn parse(pattern: &str) -> Self {
        let segments = pattern
            .split('.')
            .map(|s| match s {
                "**" => PatternSegment::DoubleWildcard,
                "*" => PatternSegment::SingleWildcard,
                _ => PatternSegment::Literal(s.to_string()),
            })
            .collect();
        Self { segments }
    }

    pub fn matches(&self, path: &[String]) -> bool {
        self.matches_recursive(&self.segments, path)
    }

    fn matches_recursive(&self, pattern: &[PatternSegment], path: &[String]) -> bool {
        match (pattern.first(), path.first()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_seg), None) => pattern
                .iter()
                .all(|s| matches!(s, PatternSegment::DoubleWildcard)),
            (Some(seg), Some(path_seg)) => match seg {
                PatternSegment::Literal(lit) => {
                    if lit == path_seg {
                        self.matches_recursive(&pattern[1..], &path[1..])
                    } else {
                        false
                    }
                }
                PatternSegment::SingleWildcard => self.matches_recursive(&pattern[1..], &path[1..]),
                PatternSegment::DoubleWildcard => {
                    self.matches_recursive(&pattern[1..], path)
                        || self.matches_recursive(pattern, &path[1..])
                }
            },
        }
    }
}

/// Configuration for filtering diff results.
#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    /// Patterns for paths to ignore (exclude from output)
    pub ignore_patterns: Vec<PathPattern>,
    /// Patterns for paths to include (if non-empty, only these are shown)
    pub only_patterns: Vec<PathPattern>,
}

impl FilterConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ignore(mut self, pattern: &str) -> Self {
        self.ignore_patterns.push(PathPattern::parse(pattern));
        self
    }

    pub fn only(mut self, pattern: &str) -> Self {
        self.only_patterns.push(PathPattern::parse(pattern));
        self
    }

    pub fn has_filters(&self) -> bool {
        !self.ignore_patterns.is_empty() || !self.only_patterns.is_empty()
    }

    pub fn should_include(&self, path: &[String]) -> bool {
        for pattern in &self.ignore_patterns {
            if pattern.matches(path) {
                return false;
            }
        }

        // If only patterns are specified, at least one must match
        if !self.only_patterns.is_empty() {
            for pattern in &self.only_patterns {
                if pattern.matches(path) {
                    return true;
                }
            }
            return false;
        }

        // No only patterns, and no ignore matched
        true
    }
}

/// Filters a diff based on the filter configuration.
pub fn filter_diff(diff: &Diff, config: &FilterConfig) -> Diff {
    if !config.has_filters() {
        return diff.clone();
    }

    let filtered_changes: Vec<Change> = diff
        .changes
        .iter()
        .filter(|change| config.should_include(&change.path))
        .cloned()
        .collect();

    let mut stats = DiffStats::new();
    for change in &filtered_changes {
        match change.change_type {
            crate::diff::ChangeType::Added => stats.added += 1,
            crate::diff::ChangeType::Removed => stats.removed += 1,
            crate::diff::ChangeType::Modified => stats.modified += 1,
            crate::diff::ChangeType::Unchanged => stats.unchanged += 1,
        }
    }

    Diff {
        changes: filtered_changes,
        stats,
    }
}
