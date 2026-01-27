//! Git integration for sdiff.

use std::env;
use std::process::Command;

pub type GitResult<T> = Result<T, GitError>;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Failed to execute git command: {0}")]
    CommandFailed(String),

    #[error("Git is not installed or not in PATH")]
    GitNotFound,

    #[error("Failed to determine sdiff executable path")]
    ExecutableNotFound,

    #[error("Git command returned error: {0}")]
    GitError(String),
}

/// Installs sdiff as a git difftool and diff driver.
pub fn install() -> GitResult<()> {
    let sdiff_path = get_executable_path()?;

    run_git_config(
        "difftool.sdiff.cmd",
        &format!("{} \"$LOCAL\" \"$REMOTE\"", sdiff_path),
    )?;
    run_git_config("diff.sdiff.command", &sdiff_path)?;
    run_git_config("difftool.sdiff.prompt", "false")?;

    println!("Successfully installed sdiff as git difftool.");
    println!();
    println!("Usage:");
    println!("  git difftool -t sdiff HEAD~1 -- file.json");
    println!("  git difftool -t sdiff branch1 branch2 -- config.yaml");
    println!();
    println!("To use automatically for specific files, add to .gitattributes:");
    println!("  *.json diff=sdiff");
    println!("  *.yaml diff=sdiff");
    println!("  *.toml diff=sdiff");

    Ok(())
}

/// Uninstalls sdiff from git configuration.
pub fn uninstall() -> GitResult<()> {
    run_git_config_unset("difftool.sdiff.cmd")?;
    run_git_config_unset("difftool.sdiff.prompt")?;
    run_git_config_unset("diff.sdiff.command")?;

    println!("Successfully uninstalled sdiff from git configuration.");

    Ok(())
}

/// Shows the current git configuration status for sdiff.
pub fn status() -> GitResult<()> {
    println!("Git sdiff configuration status:");
    println!();

    match get_git_config("difftool.sdiff.cmd") {
        Ok(value) => {
            println!("  difftool.sdiff.cmd: {}", value);
        }
        Err(_) => {
            println!("  difftool.sdiff.cmd: (not configured)");
        }
    }

    match get_git_config("difftool.sdiff.prompt") {
        Ok(value) => {
            println!("  difftool.sdiff.prompt: {}", value);
        }
        Err(_) => {
            println!("  difftool.sdiff.prompt: (not configured)");
        }
    }

    match get_git_config("diff.sdiff.command") {
        Ok(value) => {
            println!("  diff.sdiff.command: {}", value);
        }
        Err(_) => {
            println!("  diff.sdiff.command: (not configured)");
        }
    }

    println!();

    let has_config = get_git_config("difftool.sdiff.cmd").is_ok()
        || get_git_config("diff.sdiff.command").is_ok();

    if has_config {
        println!("sdiff is configured as a git difftool.");
        println!();
        println!("Usage:");
        println!("  git difftool -t sdiff HEAD~1 -- file.json");
    } else {
        println!("sdiff is not configured. Run 'sdiff --git-install' to set up.");
    }

    Ok(())
}

/// Detects if invoked with git's 7-argument diff driver protocol.
/// Returns `Some((old_file, new_file))` if 7-arg mode is detected.
pub fn detect_git_diff_driver_args(args: &[String]) -> Option<(String, String)> {
    if args.len() != 7 {
        return None;
    }

    let old_hex = &args[2];
    let new_hex = &args[5];

    if !is_git_hash(old_hex) || !is_git_hash(new_hex) {
        return None;
    }

    let old_file = args[1].clone();
    let new_file = args[4].clone();

    Some((old_file, new_file))
}

/// Checks if a file path represents a deleted or new file (/dev/null).
pub fn is_null_file(path: &str) -> bool {
    path == "/dev/null" || path == "nul" || path == "NUL"
}

fn is_git_hash(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn get_executable_path() -> GitResult<String> {
    env::current_exe()
        .map_err(|_| GitError::ExecutableNotFound)
        .map(|p| p.to_string_lossy().into_owned())
}

fn run_git_config(key: &str, value: &str) -> GitResult<()> {
    let output = Command::new("git")
        .args(["config", "--global", key, value])
        .output()
        .map_err(|_| GitError::GitNotFound)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::GitError(stderr.into_owned()));
    }

    Ok(())
}

fn run_git_config_unset(key: &str) -> GitResult<()> {
    let output = Command::new("git")
        .args(["config", "--global", "--unset", key])
        .output()
        .map_err(|_| GitError::GitNotFound)?;

    if !output.status.success() && output.status.code() != Some(5) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::GitError(stderr.into_owned()));
    }

    Ok(())
}

fn get_git_config(key: &str) -> GitResult<String> {
    let output = Command::new("git")
        .args(["config", "--global", "--get", key])
        .output()
        .map_err(|_| GitError::GitNotFound)?;

    if !output.status.success() {
        return Err(GitError::GitError(format!("Key {} not found", key)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_hash_valid() {
        // Valid SHA-1 hash
        assert!(is_git_hash("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"));
        // Null hash (40 zeros)
        assert!(is_git_hash("0000000000000000000000000000000000000000"));
        // All hex characters
        assert!(is_git_hash("abcdef0123456789abcdef0123456789abcdef01"));
    }

    #[test]
    fn test_is_git_hash_invalid() {
        // Too short
        assert!(!is_git_hash("a1b2c3"));
        // Too long
        assert!(!is_git_hash("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3"));
        // Invalid characters
        assert!(!is_git_hash("g1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"));
        // Empty
        assert!(!is_git_hash(""));
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
        // Too few arguments
        let args = vec!["file1.json".to_string(), "file2.json".to_string()];
        assert!(detect_git_diff_driver_args(&args).is_none());

        // Too many arguments
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
}
