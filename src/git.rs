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
