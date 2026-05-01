mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

/// Like `common::tickets` but sets `current_dir` to `dir` so the binary's
/// `Path::new(".")` resolves to the test git repo, not the project root.
fn tickets(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(common::bin())
        .args(args)
        .env("TICKETS_DIR", dir)
        .current_dir(dir)
        .output()
        .expect("failed to run tickets binary")
}

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).status().unwrap();
    Command::new("git").args(["config", "user.email", "test@example.com"]).current_dir(dir).status().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).status().unwrap();
}

fn git_log_count(dir: &std::path::Path) -> usize {
    let out = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .count()
}

fn git_log_last(dir: &std::path::Path) -> String {
    let out = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn enable_auto_commit(dir: &std::path::Path) {
    fs::write(dir.join(".tickets.toml"), "[git]\nauto_commit = true\n").unwrap();
}

#[test]
fn new_auto_commits_when_enabled() {
    let dir = common::test_dir("autocommit_new_enabled");
    init_git_repo(&dir);
    tickets(&dir, &["init"]);
    enable_auto_commit(&dir);

    let out = tickets(&dir, &["new", "--title", "Test ticket"]);
    assert!(out.status.success(), "new failed: {:?}", out);
    assert_eq!(git_log_count(&dir), 1, "expected 1 commit after new");

    let log = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let msg = String::from_utf8_lossy(&log.stdout);
    assert!(msg.contains("tickets: new"), "unexpected commit message: {msg}");
    assert!(msg.contains("Test ticket"), "commit message missing title: {msg}");
}

#[test]
fn new_no_commit_when_disabled() {
    let dir = common::test_dir("autocommit_new_disabled");
    init_git_repo(&dir);
    tickets(&dir, &["init"]);
    // .tickets.toml left with default (auto_commit = false)

    let out = tickets(&dir, &["new", "--title", "Silent ticket"]);
    assert!(out.status.success(), "new failed: {:?}", out);
    assert_eq!(git_log_count(&dir), 0, "expected no commits when auto_commit = false");
}

#[test]
fn edit_auto_commits_when_enabled() {
    let dir = common::test_dir("autocommit_edit_enabled");
    init_git_repo(&dir);
    tickets(&dir, &["init"]);
    enable_auto_commit(&dir);

    // new auto-commits the file; now edit it
    let new_out = tickets(&dir, &["new", "--title", "Edit me"]);
    assert!(new_out.status.success());
    let stdout = String::from_utf8_lossy(&new_out.stdout);
    let last_line = stdout.trim().lines().last().unwrap();
    let id = last_line.split_whitespace().next().unwrap().to_string();

    let edit_out = tickets(&dir, &["edit", &id, "--status", "todo"]);
    assert!(edit_out.status.success(), "edit failed: {:?}", edit_out);
    assert_eq!(git_log_count(&dir), 2, "expected 2 commits (new + edit)");

    let log = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let msg = String::from_utf8_lossy(&log.stdout);
    assert!(msg.contains("tickets: edit"), "unexpected commit message: {msg}");
}

#[test]
fn archive_auto_commits_when_enabled() {
    let dir = common::test_dir("autocommit_archive_enabled");
    init_git_repo(&dir);
    tickets(&dir, &["init"]);
    enable_auto_commit(&dir);

    // new auto-commits; then archive
    let new_out = tickets(&dir, &["new", "--title", "Archive me"]);
    assert!(new_out.status.success());
    let stdout = String::from_utf8_lossy(&new_out.stdout);
    let id = stdout.trim().lines().last().unwrap().split_whitespace().next().unwrap().to_string();

    let arc_out = tickets(&dir, &["archive", &id]);
    assert!(arc_out.status.success(), "archive failed: {:?}", arc_out);
    assert_eq!(git_log_count(&dir), 2, "expected 2 commits (new + archive)");

    let msg = git_log_last(&dir);
    assert!(msg.contains("tickets: archive"), "unexpected commit message: {msg}");
}

fn git_is_clean(dir: &std::path::Path) -> bool {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().is_empty()
}

#[test]
fn archive_by_id_auto_commit_leaves_clean_working_tree() {
    // Regression: archive committed only the dst (archived/) file but not the
    // deletion of the src (all/) file, leaving it as an unstaged deletion.
    let dir = common::test_dir("autocommit_archive_clean_tree");
    init_git_repo(&dir);
    tickets(&dir, &["init"]);
    enable_auto_commit(&dir);

    let new_out = tickets(&dir, &["new", "--title", "Clean tree"]);
    assert!(new_out.status.success(), "new failed: {}", String::from_utf8_lossy(&new_out.stderr));
    let stdout = String::from_utf8_lossy(&new_out.stdout);
    let id = stdout.trim().lines().last().unwrap().split_whitespace().next().unwrap().to_string();

    let arc_out = tickets(&dir, &["archive", &id]);
    assert!(arc_out.status.success(), "archive failed: {:?}", arc_out);

    assert!(git_is_clean(&dir), "working tree is dirty after archive — deletion of all/ file was not committed");
}

#[test]
fn archive_all_rejected_auto_commit_leaves_clean_working_tree() {
    // Same regression for the --all-rejected path.
    let dir = common::test_dir("autocommit_archive_rejected_clean_tree");
    init_git_repo(&dir);
    tickets(&dir, &["init"]);
    enable_auto_commit(&dir);

    let new_out = tickets(&dir, &["new", "--title", "Reject me", "--status", "rejected"]);
    assert!(new_out.status.success());

    let arc_out = tickets(&dir, &["archive", "--all-rejected"]);
    assert!(arc_out.status.success(), "archive --all-rejected failed: {:?}", arc_out);

    assert!(git_is_clean(&dir), "working tree is dirty after --all-rejected archive — deletion of all/ file was not committed");
}
