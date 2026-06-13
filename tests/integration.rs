use std::path::PathBuf;
use std::process::Command;

fn cx_binary() -> PathBuf {
    let debug = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("cx");
    if debug.exists() {
        return debug;
    }
    let release = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("release")
        .join("cx");
    if release.exists() {
        return release;
    }
    debug
}

fn setup_temp_repo() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let repo_path = dir.path().to_path_buf();

    let output = Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("failed to run git init");
    assert!(output.status.success(), "git init failed");

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&repo_path)
        .output()
        .expect("failed to set git user.email");
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("failed to set git user.name");

    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("failed to create initial commit");

    (dir, repo_path)
}

fn run_cx(repo_path: &PathBuf, args: &[&str]) -> (bool, String, String) {
    let output = Command::new(cx_binary())
        .args(args)
        .current_dir(repo_path)
        .output()
        .expect("failed to run cx");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

#[test]
fn test_cx_init() {
    let (_dir, repo_path) = setup_temp_repo();

    let (success, stdout, stderr) = run_cx(&repo_path, &["init"]);
    assert!(success, "cx init failed: stderr={stderr} stdout={stdout}");
    assert!(stdout.contains("Initialized .cx/"), "stdout: {stdout}");

    let cx_dir = repo_path.join(".cx");
    assert!(cx_dir.exists(), ".cx/ directory should exist");
    assert!(cx_dir.join("sessions").exists(), ".cx/sessions/ should exist");

    let (success, _, stderr) = run_cx(&repo_path, &["init"]);
    assert!(!success, "second cx init should fail: stderr={stderr}");
}

#[test]
fn test_cx_non_git_repo() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().to_path_buf();

    let (success, _, _) = run_cx(&path, &["init"]);
    assert!(!success, "cx init should fail outside git repo");
}

#[test]
fn test_full_workflow_abandon() {
    let (_dir, repo_path) = setup_temp_repo();

    let (success, _, stderr) = run_cx(&repo_path, &["init"]);
    assert!(success, "init failed: {stderr}");

    let (success, _, stderr) = run_cx(&repo_path, &["start", "Add payment module"]);
    assert!(success, "start failed: {stderr}");

    std::fs::write(repo_path.join("test.txt"), "hello world").unwrap();
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("git add failed");
    assert!(output.status.success());

    let (success, stdout, stderr) = run_cx(
        &repo_path,
        &["apply", "-m", "Add test file", "--intent", "feature,scope=test"],
    );
    assert!(success, "apply failed: stderr={stderr} stdout={stdout}");
    assert!(stdout.contains("Applied snapshot"), "stdout: {stdout}");

    let (success, stdout, stderr) = run_cx(&repo_path, &["status"]);
    assert!(success, "status failed: {stderr}");
    assert!(stdout.contains("feature"), "status should show feature intent");

    let (success, _, _) = run_cx(&repo_path, &["log"]);
    assert!(success, "log should succeed");

    let (success, stdout, stderr) = run_cx(&repo_path, &["end", "--abandon"]);
    assert!(success, "end --abandon failed: stderr={stderr} stdout={stdout}");
    assert!(stdout.contains("abandoned"), "stdout: {stdout}");
}

#[test]
fn test_full_workflow_merge() {
    let (_dir, repo_path) = setup_temp_repo();

    let (success, _, stderr) = run_cx(&repo_path, &["init"]);
    assert!(success, "init failed: {stderr}");

    let (success, _, stderr) = run_cx(&repo_path, &["start", "Fix login bug"]);
    assert!(success, "start failed: {stderr}");

    std::fs::write(repo_path.join("fix.txt"), "fix content").unwrap();
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("git add failed");
    assert!(output.status.success());

    let (success, _, stderr) = run_cx(
        &repo_path,
        &["apply", "-m", "Fix login", "--intent", "fix,scope=login"],
    );
    assert!(success, "apply failed: {stderr}");

    let (success, stdout, stderr) = run_cx(&repo_path, &["end", "--merge"]);
    assert!(success, "end --merge failed: stderr={stderr} stdout={stdout}");
    assert!(stdout.contains("merged"), "stdout: {stdout}");
}
