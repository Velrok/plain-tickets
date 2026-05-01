mod common;

use std::fs;
use std::process::Command;

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
    common::tickets(&dir, &["init"]);
    enable_auto_commit(&dir);

    let out = common::tickets(&dir, &["new", "--title", "Test ticket"]);
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
    common::tickets(&dir, &["init"]);
    // .tickets.toml left with default (auto_commit = false)

    let out = common::tickets(&dir, &["new", "--title", "Silent ticket"]);
    assert!(out.status.success(), "new failed: {:?}", out);
    assert_eq!(git_log_count(&dir), 0, "expected no commits when auto_commit = false");
}

#[test]
fn edit_auto_commits_when_enabled() {
    let dir = common::test_dir("autocommit_edit_enabled");
    init_git_repo(&dir);
    common::tickets(&dir, &["init"]);
    enable_auto_commit(&dir);

    // new auto-commits the file; now edit it
    let new_out = common::tickets(&dir, &["new", "--title", "Edit me"]);
    assert!(new_out.status.success());
    let stdout = String::from_utf8_lossy(&new_out.stdout);
    let last_line = stdout.trim().lines().last().unwrap();
    let id = last_line.split_whitespace().next().unwrap().to_string();

    let edit_out = common::tickets(&dir, &["edit", &id, "--status", "todo"]);
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
    common::tickets(&dir, &["init"]);
    enable_auto_commit(&dir);

    // new auto-commits; then archive
    let new_out = common::tickets(&dir, &["new", "--title", "Archive me"]);
    assert!(new_out.status.success());
    let stdout = String::from_utf8_lossy(&new_out.stdout);
    let id = stdout.trim().lines().last().unwrap().split_whitespace().next().unwrap().to_string();

    let arc_out = common::tickets(&dir, &["archive", &id]);
    assert!(arc_out.status.success(), "archive failed: {:?}", arc_out);
    assert_eq!(git_log_count(&dir), 2, "expected 2 commits (new + archive)");

    let msg = git_log_last(&dir);
    assert!(msg.contains("tickets: archive"), "unexpected commit message: {msg}");
}
