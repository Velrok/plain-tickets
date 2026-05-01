mod common;

use std::fs;

#[test]
fn edit_updates_status() {
    let dir = common::test_dir("edit_updates_status");
    common::tickets(&dir, &["init"]);
    let (id, filename) = common::create_ticket(&dir, "Some ticket");

    let out = common::tickets(&dir, &["edit", &id, "--status", "in-progress"]);
    assert!(out.status.success(), "edit failed: {:?}", out);

    let content = fs::read_to_string(dir.join("all").join(&filename)).unwrap();
    assert!(content.contains("status: in-progress"));
}

#[test]
fn edit_updates_title() {
    let dir = common::test_dir("edit_updates_title");
    common::tickets(&dir, &["init"]);
    let (id, _) = common::create_ticket(&dir, "Old title");

    let out = common::tickets(&dir, &["edit", &id, "--title", "New title"]);
    assert!(out.status.success(), "edit failed: {:?}", out);

    let entry = fs::read_dir(dir.join("all"))
        .unwrap()
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with(&id))
        .expect("ticket file not found after edit");

    let content = fs::read_to_string(entry.path()).unwrap();
    assert!(content.contains("New title"));
}

#[test]
fn edit_sets_and_clears_parent() {
    let dir = common::test_dir("edit_sets_and_clears_parent");
    common::tickets(&dir, &["init"]);
    let (parent_id, _) = common::create_ticket(&dir, "Parent ticket");
    let (child_id, child_file) = common::create_ticket(&dir, "Child ticket");

    let out = common::tickets(&dir, &["edit", &child_id, "--parent", &parent_id]);
    assert!(out.status.success());
    let content = fs::read_to_string(dir.join("all").join(&child_file)).unwrap();
    assert!(content.contains(&format!("parent: {}", parent_id)));

    let out = common::tickets(&dir, &["edit", &child_id, "--clear-parent"]);
    assert!(out.status.success());
    let content = fs::read_to_string(dir.join("all").join(&child_file)).unwrap();
    assert!(!content.contains(&format!("parent: {}", parent_id)));
}

#[test]
fn edit_updates_type() {
    let dir = common::test_dir("edit_updates_type");
    common::tickets(&dir, &["init"]);
    let (id, filename) = common::create_ticket(&dir, "Some ticket");

    let out = common::tickets(&dir, &["edit", &id, "--type", "bug"]);
    assert!(out.status.success(), "edit failed: {:?}", out);

    let content = fs::read_to_string(dir.join("all").join(&filename)).unwrap();
    assert!(content.contains("type: bug"));
}

#[test]
fn edit_updates_body() {
    let dir = common::test_dir("edit_updates_body");
    common::tickets(&dir, &["init"]);
    let (id, filename) = common::create_ticket(&dir, "Some ticket");

    let out = common::tickets(&dir, &["edit", &id, "--body", "Updated body text."]);
    assert!(out.status.success(), "edit failed: {:?}", out);

    let content = fs::read_to_string(dir.join("all").join(&filename)).unwrap();
    assert!(content.contains("Updated body text."));
}

#[test]
fn edit_replaces_tags() {
    let dir = common::test_dir("edit_replaces_tags");
    common::tickets(&dir, &["init"]);
    // create with initial tags
    let out = common::tickets(&dir, &["new", "--title", "Tagged ticket", "--tag", "old-tag"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut parts = stdout.trim().splitn(2, ' ');
    let id = parts.next().unwrap().to_string();
    let filename = parts.next().unwrap().to_string();

    let out = common::tickets(&dir, &["edit", &id, "--tag", "new-tag"]);
    assert!(out.status.success());

    let content = fs::read_to_string(dir.join("all").join(&filename)).unwrap();
    assert!(content.contains("new-tag"), "new tag missing");
    assert!(!content.contains("old-tag"), "old tag still present");
}

#[test]
fn edit_sets_and_clears_blocked_by() {
    let dir = common::test_dir("edit_sets_and_clears_blocked_by");
    common::tickets(&dir, &["init"]);
    let (blocker_id, _) = common::create_ticket(&dir, "Blocker ticket");
    let (child_id, child_file) = common::create_ticket(&dir, "Blocked ticket");

    let out = common::tickets(&dir, &["edit", &child_id, "--blocked-by", &blocker_id]);
    assert!(out.status.success());
    let content = fs::read_to_string(dir.join("all").join(&child_file)).unwrap();
    assert!(content.contains(&blocker_id.to_string()));

    let out = common::tickets(&dir, &["edit", &child_id, "--clear-blocked-by"]);
    assert!(out.status.success());
    let content = fs::read_to_string(dir.join("all").join(&child_file)).unwrap();
    assert!(!content.contains(&blocker_id.to_string()));
}

#[test]
fn edit_nonexistent_ticket_fails() {
    let dir = common::test_dir("edit_nonexistent_ticket_fails");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["edit", "zzzzzz", "--status", "open"]);
    assert!(!out.status.success());
}
