use sdiff::Node;
use std::collections::HashMap;

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
    assert!(Node::Null.semantic_equals(&Node::Null));

    assert!(Node::Bool(true).semantic_equals(&Node::Bool(true)));
    assert!(Node::Bool(false).semantic_equals(&Node::Bool(false)));
    assert!(!Node::Bool(true).semantic_equals(&Node::Bool(false)));

    assert!(Node::String("hello".to_string()).semantic_equals(&Node::String("hello".to_string())));
    assert!(!Node::String("hello".to_string()).semantic_equals(&Node::String("world".to_string())));

    assert!(!Node::Null.semantic_equals(&Node::Bool(false)));
    assert!(!Node::Bool(true).semantic_equals(&Node::Number(1.0)));
}

#[test]
fn test_semantic_equals_numbers() {
    assert!(Node::Number(42.0).semantic_equals(&Node::Number(42.0)));

    assert!(Node::Number(1.0).semantic_equals(&Node::Number(1.0 + 1e-15)));
    assert!(Node::Number(1.0).semantic_equals(&Node::Number(1.0 - 1e-15)));

    assert!(!Node::Number(1.0).semantic_equals(&Node::Number(1.1)));
}

#[test]
fn test_semantic_equals_objects() {
    assert!(Node::Object(HashMap::new()).semantic_equals(&Node::Object(HashMap::new())));

    let mut obj1 = HashMap::new();
    obj1.insert("a".to_string(), Node::Number(1.0));
    obj1.insert("b".to_string(), Node::Number(2.0));

    let mut obj2 = HashMap::new();
    obj2.insert("a".to_string(), Node::Number(1.0));
    obj2.insert("b".to_string(), Node::Number(2.0));

    assert!(Node::Object(obj1.clone()).semantic_equals(&Node::Object(obj2)));

    let mut obj3 = HashMap::new();
    obj3.insert("b".to_string(), Node::Number(2.0));
    obj3.insert("a".to_string(), Node::Number(1.0));

    assert!(Node::Object(obj1.clone()).semantic_equals(&Node::Object(obj3)));

    let mut obj4 = HashMap::new();
    obj4.insert("a".to_string(), Node::Number(1.0));
    obj4.insert("b".to_string(), Node::Number(3.0));

    assert!(!Node::Object(obj1.clone()).semantic_equals(&Node::Object(obj4)));

    let mut obj5 = HashMap::new();
    obj5.insert("a".to_string(), Node::Number(1.0));
    obj5.insert("c".to_string(), Node::Number(2.0));

    assert!(!Node::Object(obj1).semantic_equals(&Node::Object(obj5)));
}

#[test]
fn test_semantic_equals_arrays() {
    assert!(Node::Array(vec![]).semantic_equals(&Node::Array(vec![])));

    let arr1 = vec![Node::Number(1.0), Node::Number(2.0), Node::Number(3.0)];
    let arr2 = vec![Node::Number(1.0), Node::Number(2.0), Node::Number(3.0)];
    assert!(Node::Array(arr1.clone()).semantic_equals(&Node::Array(arr2)));

    let arr3 = vec![Node::Number(3.0), Node::Number(2.0), Node::Number(1.0)];
    assert!(!Node::Array(arr1.clone()).semantic_equals(&Node::Array(arr3)));

    let arr4 = vec![Node::Number(1.0), Node::Number(2.0)];
    assert!(!Node::Array(arr1.clone()).semantic_equals(&Node::Array(arr4)));

    let arr5 = vec![Node::Number(1.0), Node::Number(5.0), Node::Number(3.0)];
    assert!(!Node::Array(arr1).semantic_equals(&Node::Array(arr5)));
}

#[test]
fn test_semantic_equals_nested() {
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
    assert!(Node::Object(HashMap::new()).semantic_equals(&Node::Object(HashMap::new())));
    assert!(Node::Array(vec![]).semantic_equals(&Node::Array(vec![])));

    let arr1 = vec![Node::Null, Node::Null];
    let arr2 = vec![Node::Null, Node::Null];
    assert!(Node::Array(arr1).semantic_equals(&Node::Array(arr2)));

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
    assert_eq!(Node::Object(HashMap::new()).preview(100), "{}");
    assert_eq!(Node::Array(vec![]).preview(100), "[]");

    let mut obj = HashMap::new();
    obj.insert("key".to_string(), Node::Number(1.0));
    assert_eq!(Node::Object(obj).preview(100), "{ 1 key }");

    let arr = vec![Node::Number(1.0)];
    assert_eq!(Node::Array(arr).preview(100), "[ 1 item ]");

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

    assert!(preview.len() <= 23);
    assert!(preview.ends_with("..."));
}

#[test]
fn test_size() {
    assert!(Node::Null.size() > 0);
    assert!(Node::Bool(true).size() > 0);
    assert!(Node::Number(42.0).size() > 0);

    let small = Node::String("hi".to_string());
    let large = Node::String("x".repeat(1000));
    assert!(large.size() > small.size());

    let mut obj = HashMap::new();
    obj.insert("key".to_string(), Node::String("value".to_string()));
    let obj_node = Node::Object(obj);
    assert!(obj_node.size() > Node::Null.size());

    let arr = vec![Node::Number(1.0), Node::Number(2.0), Node::Number(3.0)];
    let arr_node = Node::Array(arr);
    assert!(arr_node.size() > Node::Number(1.0).size());
}

#[test]
fn test_number_preview_formatting() {
    assert_eq!(Node::Number(42.0).preview(100), "42");
    assert_eq!(Node::Number(0.0).preview(100), "0");
    assert_eq!(Node::Number(-10.0).preview(100), "-10");

    assert_eq!(Node::Number(42.5).preview(100), "42.5");
    assert_eq!(Node::Number(3.25).preview(100), "3.25");
}
