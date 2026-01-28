//! Integration tests for the SDIFF CLI tool.
//!
//! These tests verify the complete end-to-end behavior of the CLI,
//! including argument parsing, file processing, and output formatting.

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to create a Command for the sdiff-rs binary
fn sdiff() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("sdiff-rs"))
}

#[test]
fn test_identical_files_exit_0() {
    sdiff()
        .arg("tests/fixtures/identical_1.json")
        .arg("tests/fixtures/identical_2.json")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("No changes"));
}

#[test]
fn test_different_files_exit_1() {
    sdiff()
        .arg("tests/fixtures/modified_old.json")
        .arg("tests/fixtures/modified_new.json")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("age"));
}

#[test]
fn test_file_not_found_exit_2() {
    sdiff()
        .arg("tests/fixtures/nonexistent.json")
        .arg("tests/fixtures/identical_1.json")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_formatting_differences_no_semantic_change() {
    sdiff()
        .arg("tests/fixtures/formatting_old.json")
        .arg("tests/fixtures/formatting_new.json")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("No changes"));
}

#[test]
fn test_modified_field() {
    sdiff()
        .arg("tests/fixtures/modified_old.json")
        .arg("tests/fixtures/modified_new.json")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("age"))
        .stdout(predicate::str::contains("30"))
        .stdout(predicate::str::contains("31"))
        .stdout(predicate::str::contains("Summary"));
}

#[test]
fn test_added_fields() {
    sdiff()
        .arg("tests/fixtures/added_old.json")
        .arg("tests/fixtures/added_new.json")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("age"))
        .stdout(predicate::str::contains("email"))
        .stdout(predicate::str::contains("2 added"));
}

#[test]
fn test_removed_field() {
    sdiff()
        .arg("tests/fixtures/removed_old.json")
        .arg("tests/fixtures/removed_new.json")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("deprecated"))
        .stdout(predicate::str::contains("1 removed"));
}

#[test]
fn test_nested_changes() {
    sdiff()
        .arg("tests/fixtures/nested_old.json")
        .arg("tests/fixtures/nested_new.json")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("user.profile"));
}

#[test]
fn test_array_changes() {
    sdiff()
        .arg("tests/fixtures/array_old.json")
        .arg("tests/fixtures/array_new.json")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("["));
}

#[test]
fn test_json_output_format() {
    sdiff()
        .arg("tests/fixtures/modified_old.json")
        .arg("tests/fixtures/modified_new.json")
        .arg("--format=json")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"changes\""))
        .stdout(predicate::str::contains("\"stats\""))
        .stdout(predicate::str::contains("\"modified\""));
}

#[test]
fn test_plain_output_format() {
    sdiff()
        .arg("tests/fixtures/modified_old.json")
        .arg("tests/fixtures/modified_new.json")
        .arg("--format=plain")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("age"));
}

#[test]
fn test_verbose_flag() {
    sdiff()
        .arg("tests/fixtures/identical_1.json")
        .arg("tests/fixtures/identical_2.json")
        .arg("--verbose")
        .assert()
        .code(0)
        .stderr(predicate::str::contains("Parsing"))
        .stderr(predicate::str::contains("Computing diff"));
}

#[test]
fn test_quiet_flag() {
    sdiff()
        .arg("tests/fixtures/modified_old.json")
        .arg("tests/fixtures/modified_new.json")
        .arg("--quiet")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("Summary").not());
}

#[test]
fn test_compact_flag() {
    sdiff()
        .arg("tests/fixtures/modified_old.json")
        .arg("tests/fixtures/modified_new.json")
        .arg("--compact")
        .assert()
        .code(1);
}

#[test]
fn test_help_flag() {
    sdiff()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Semantic diff tool"))
        .stdout(predicate::str::contains("FILE1"))
        .stdout(predicate::str::contains("FILE2"));
}

#[test]
fn test_version_flag() {
    sdiff()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("sdiff"));
}

#[test]
fn test_max_value_length() {
    sdiff()
        .arg("tests/fixtures/modified_old.json")
        .arg("tests/fixtures/modified_new.json")
        .arg("--max-value-length=10")
        .assert()
        .code(1);
}

#[test]
fn test_mixed_json_yaml() {
    sdiff()
        .arg("tests/fixtures/mixed.json")
        .arg("tests/fixtures/mixed.yaml")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("No changes"));
}

#[test]
fn test_invalid_file_format() {
    sdiff()
        .arg("tests/fixtures/invalid.txt")
        .arg("tests/fixtures/identical_1.json")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_large_file() {
    sdiff()
        .arg("tests/fixtures/large.json")
        .arg("tests/fixtures/large.json")
        .assert()
        .success()
        .code(0);
}
