use sdiff_rs::git::{detect_git_diff_driver_args, is_null_file};

#[test]
fn test_is_git_hash_valid() {
    // Valid SHA-1 hashes are detected via the 7-arg mode
    let args = vec![
        "file.json".to_string(),
        "/tmp/old_file".to_string(),
        "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
        "100644".to_string(),
        "/tmp/new_file".to_string(),
        "b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3".to_string(),
        "100644".to_string(),
    ];
    assert!(detect_git_diff_driver_args(&args).is_some());
}

#[test]
fn test_is_git_hash_invalid() {
    // Invalid hashes should not be detected
    let args = vec![
        "file.json".to_string(),
        "/tmp/old_file".to_string(),
        "not_a_hash".to_string(),
        "100644".to_string(),
        "/tmp/new_file".to_string(),
        "also_not_a_hash".to_string(),
        "100644".to_string(),
    ];
    assert!(detect_git_diff_driver_args(&args).is_none());
}

#[test]
fn test_detect_git_diff_driver_args_valid() {
    let args = vec![
        "file.json".to_string(),
        "/tmp/old_file".to_string(),
        "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
        "100644".to_string(),
        "/tmp/new_file".to_string(),
        "b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3".to_string(),
        "100644".to_string(),
    ];

    let result = detect_git_diff_driver_args(&args);
    assert!(result.is_some());

    let (old, new) = result.unwrap();
    assert_eq!(old, "/tmp/old_file");
    assert_eq!(new, "/tmp/new_file");
}

#[test]
fn test_detect_git_diff_driver_args_wrong_count() {
    let args = vec!["file1.json".to_string(), "file2.json".to_string()];
    assert!(detect_git_diff_driver_args(&args).is_none());

    let args = vec![
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "4".to_string(),
        "5".to_string(),
        "6".to_string(),
        "7".to_string(),
        "8".to_string(),
    ];
    assert!(detect_git_diff_driver_args(&args).is_none());
}

#[test]
fn test_detect_git_diff_driver_args_invalid_hashes() {
    let args = vec![
        "file.json".to_string(),
        "/tmp/old_file".to_string(),
        "not_a_hash".to_string(),
        "100644".to_string(),
        "/tmp/new_file".to_string(),
        "also_not_a_hash".to_string(),
        "100644".to_string(),
    ];

    assert!(detect_git_diff_driver_args(&args).is_none());
}

#[test]
fn test_is_null_file() {
    assert!(is_null_file("/dev/null"));
    assert!(is_null_file("nul"));
    assert!(is_null_file("NUL"));
    assert!(!is_null_file("/tmp/file.json"));
    assert!(!is_null_file("file.json"));
}
