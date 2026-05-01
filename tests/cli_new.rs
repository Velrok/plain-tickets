mod common;

use std::fs;

#[test]
fn new_creates_ticket_file() {
    let dir = common::test_dir("new_creates_ticket_file");
    common::tickets(&dir, &["init"]);

    let out = common::tickets(&dir, &["new", "--title", "Fix login bug"]);
    assert!(out.status.success(), "new failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let parts: Vec<&str> = stdout.trim().splitn(2, ' ').collect();
    assert_eq!(parts.len(), 2, "unexpected stdout: {}", stdout);

    let path = dir.join("all").join(parts[1]);
    assert!(path.exists(), "ticket file not found at {}", path.display());

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("Fix login bug"));
    assert!(content.contains("type: task"));
    assert!(content.contains("status: draft"));
}

#[test]
fn new_with_type_tags_status() {
    let dir = common::test_dir("new_with_type_tags_status");
    common::tickets(&dir, &["init"]);

    let out = common::tickets(&dir, &[
        "new",
        "--title", "Auth epic",
        "--type", "epic",
        "--status", "todo",
        "--tag", "auth",
        "--tag", "backend",
    ]);
    assert!(out.status.success(), "new failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let filename = stdout.trim().splitn(2, ' ').nth(1).unwrap();
    let content = fs::read_to_string(dir.join("all").join(filename)).unwrap();

    assert!(content.contains("type: epic"));
    assert!(content.contains("status: todo"));
    assert!(content.contains("auth"));
    assert!(content.contains("backend"));
}

#[test]
fn new_filename_slug_matches_title() {
    let dir = common::test_dir("new_filename_slug_matches_title");
    common::tickets(&dir, &["init"]);

    let out = common::tickets(&dir, &["new", "--title", "Fix Login Bug"]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let filename = stdout.trim().splitn(2, ' ').nth(1).unwrap();
    assert!(filename.ends_with("fix-login-bug.md"), "unexpected filename: {}", filename);
}

#[test]
fn new_stdout_id_matches_filename_prefix() {
    let dir = common::test_dir("new_stdout_id_matches_filename_prefix");
    common::tickets(&dir, &["init"]);

    let out = common::tickets(&dir, &["new", "--title", "Some ticket"]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut parts = stdout.trim().splitn(2, ' ');
    let id = parts.next().unwrap();
    let filename = parts.next().unwrap();
    assert!(filename.starts_with(id), "filename {} does not start with id {}", filename, id);
}

#[test]
fn new_with_body() {
    let dir = common::test_dir("new_with_body");
    common::tickets(&dir, &["init"]);

    let out = common::tickets(&dir, &["new", "--title", "Bodied ticket", "--body", "This is the body."]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let filename = stdout.trim().splitn(2, ' ').nth(1).unwrap();
    let content = fs::read_to_string(dir.join("all").join(filename)).unwrap();
    assert!(content.contains("This is the body."));
}

#[test]
fn new_with_parent() {
    let dir = common::test_dir("new_with_parent");
    common::tickets(&dir, &["init"]);
    let (parent_id, _) = common::create_ticket(&dir, "Parent ticket");

    let out = common::tickets(&dir, &["new", "--title", "Child ticket", "--parent", &parent_id]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let filename = stdout.trim().splitn(2, ' ').nth(1).unwrap();
    let content = fs::read_to_string(dir.join("all").join(filename)).unwrap();
    assert!(content.contains(&format!("parent: {}", parent_id)));
}

#[test]
fn new_with_blocked_by() {
    let dir = common::test_dir("new_with_blocked_by");
    common::tickets(&dir, &["init"]);
    let (blocker_id, _) = common::create_ticket(&dir, "Blocker ticket");

    let out = common::tickets(&dir, &["new", "--title", "Blocked ticket", "--blocked-by", &blocker_id]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let filename = stdout.trim().splitn(2, ' ').nth(1).unwrap();
    let content = fs::read_to_string(dir.join("all").join(filename)).unwrap();
    assert!(content.contains(&blocker_id.to_string()));
}

#[test]
fn new_without_init_fails() {
    let dir = common::test_dir("new_without_init_fails");
    // deliberately skip init
    let out = common::tickets(&dir, &["new", "--title", "Orphan ticket"]);
    assert!(!out.status.success(), "expected failure without init");
}
