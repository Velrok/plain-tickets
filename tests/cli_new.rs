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
fn new_without_init_fails() {
    let dir = common::test_dir("new_without_init_fails");
    // deliberately skip init
    let out = common::tickets(&dir, &["new", "--title", "Orphan ticket"]);
    assert!(!out.status.success(), "expected failure without init");
}
