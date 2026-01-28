use sdiff_rs::{parse_file, parse_json, parse_yaml, Node};
use std::fs;
use std::io::Write;
use std::path::Path;
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
    assert!(parse_yaml("key: value: invalid").is_err());
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

    fs::remove_file(&path).unwrap();
}

#[test]
fn test_parse_file_not_found() {
    let result = parse_file(Path::new("/nonexistent/file.json"));
    assert!(result.is_err());
}

#[test]
fn test_parse_file_unknown_extension() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"{{"key": "value"}}"#).unwrap();
    let path = file.path().with_extension("txt");
    fs::copy(file.path(), &path).unwrap();

    let node = parse_file(&path).unwrap();
    match node {
        Node::Object(map) => {
            assert_eq!(map.get("key").unwrap(), &Node::String("value".to_string()));
        }
        _ => panic!("Expected object"),
    }

    fs::remove_file(&path).unwrap();
}

#[test]
fn test_yaml_non_string_keys() {
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
