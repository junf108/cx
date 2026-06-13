use std::path::PathBuf;
use std::process::Command;

/// Git operation error
#[derive(Debug)]
pub struct GitError {
    pub message: String,
    pub stderr: String,
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.stderr.is_empty() {
            write!(f, "git error: {}", self.message)
        } else {
            write!(f, "git error: {}\n{}", self.message, self.stderr)
        }
    }
}

impl std::error::Error for GitError {}

pub type Result<T> = std::result::Result<T, GitError>;

pub fn run_git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| GitError {
            message: format!("failed to run git: {e}"),
            stderr: String::new(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(GitError {
            message: format!("git {} failed", args.join(" ")),
            stderr,
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get the current git repository root
pub fn rev_parse_toplevel() -> Result<PathBuf> {
    run_git(&["rev-parse", "--show-toplevel"]).map(PathBuf::from)
}

/// Get the current HEAD commit hash
pub fn rev_parse_head() -> Result<String> {
    run_git(&["rev-parse", "HEAD"])
}

/// Get the current branch name
pub fn current_branch() -> Result<String> {
    run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Check for uncommitted changes
pub fn has_uncommitted_changes() -> Result<bool> {
    let output = run_git(&["status", "--porcelain"])?;
    Ok(!output.is_empty())
}

/// Stash uncommitted changes
pub fn stash() -> Result<()> {
    run_git(&["stash", "--include-untracked"])?;
    Ok(())
}

/// Pop the stash
pub fn stash_pop() -> Result<()> {
    run_git(&["stash", "pop"])?;
    Ok(())
}

/// Create and switch to a new branch
pub fn checkout_new_branch(branch: &str) -> Result<()> {
    run_git(&["checkout", "-b", branch])?;
    Ok(())
}

/// Switch to an existing branch
pub fn checkout(branch: &str) -> Result<()> {
    run_git(&["checkout", branch])?;
    Ok(())
}

/// Stage all changes
pub fn add_all() -> Result<()> {
    run_git(&["add", "-A"])?;
    Ok(())
}

/// Create a commit
pub fn commit(message: &str) -> Result<String> {
    run_git(&["commit", "-m", message])?;
    rev_parse_head()
}

/// Merge a branch
pub fn merge(branch: &str) -> Result<()> {
    run_git(&["merge", branch])?;
    Ok(())
}

/// Force-delete a branch
pub fn delete_branch(branch: &str) -> Result<()> {
    run_git(&["branch", "-D", branch])?;
    Ok(())
}

/// Get list of staged files
pub fn diff_cached_names() -> Result<Vec<String>> {
    let output = run_git(&["diff", "--cached", "--name-only"])?;
    if output.is_empty() {
        return Ok(Vec::new());
    }
    Ok(output.lines().map(|s| s.to_string()).collect())
}

/// Parse active session ID from the current branch name
/// Branch format: cx/s_xxxxxxxx-<slug>
pub fn current_session_id() -> Result<Option<String>> {
    let branch = current_branch()?;
    if branch == "HEAD" || !branch.starts_with("cx/") {
        return Ok(None);
    }
    let after_prefix = &branch[3..]; // skip "cx/"
    if let Some(dash_pos) = after_prefix.find('-') {
        let sid = &after_prefix[..dash_pos];
        if sid.starts_with("s_") {
            return Ok(Some(sid.to_string()));
        }
    }
    Ok(None)
}

/// Diff between two commits
#[allow(dead_code)]
pub fn diff(from: &str, to: &str) -> Result<String> {
    run_git(&["diff", &format!("{from}..{to}")])
}

/// Get diff stats (file list + line counts)
pub fn diff_stat(from: &str, to: &str) -> Result<String> {
    run_git(&["diff", "--stat", &format!("{from}..{to}")])
}

/// Get list of changed files
#[allow(dead_code)]
pub fn diff_files(from: &str, to: &str) -> Result<Vec<String>> {
    let output = run_git(&["diff", "--name-only", &format!("{from}..{to}")])?;
    if output.is_empty() {
        return Ok(Vec::new());
    }
    Ok(output.lines().map(|s| s.to_string()).collect())
}

/// Get git log (compact format)
#[allow(dead_code)]
pub fn log(pretty_format: &str, max_count: u32) -> Result<String> {
    run_git(&[
        "log",
        &format!("--pretty=format:{pretty_format}"),
        &format!("-{max_count}"),
    ])
}

/// Check if inside a git repository
pub fn is_git_repo() -> bool {
    rev_parse_toplevel().is_ok()
}
