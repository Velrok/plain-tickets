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
fn edit_nonexistent_ticket_fails() {
    let dir = common::test_dir("edit_nonexistent_ticket_fails");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["edit", "zzzzzz", "--status", "open"]);
    assert!(!out.status.success());
}
