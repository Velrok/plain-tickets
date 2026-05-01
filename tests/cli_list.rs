mod common;

#[test]
fn list_errors_when_not_initialised() {
    let dir = common::test_dir("list_errors_when_not_initialised");
    let out = common::tickets(&dir, &["list"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not initialised"), "unexpected stderr: {}", stderr);
}

#[test]
fn list_empty_is_silent() {
    let dir = common::test_dir("list_empty_is_silent");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);
    assert_eq!(out.stdout, b"", "expected no output, got: {:?}", out.stdout);
}

#[test]
fn list_shows_one_ticket() {
    let dir = common::test_dir("list_shows_one_ticket");
    common::tickets(&dir, &["init"]);
    let (id, _) = common::create_ticket(&dir, "Fix login bug");

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got: {:?}", lines);
    assert!(lines[0].starts_with(&id), "line should start with id: {}", lines[0]);
    assert!(lines[0].contains("draft"), "line should contain status: {}", lines[0]);
    assert!(lines[0].contains("task"), "line should contain type: {}", lines[0]);
    assert!(lines[0].contains("Fix login bug"), "line should contain title: {}", lines[0]);
}

#[test]
fn list_shows_multiple_tickets() {
    let dir = common::test_dir("list_shows_multiple_tickets");
    common::tickets(&dir, &["init"]);
    common::create_ticket(&dir, "Fix login bug");
    common::create_ticket(&dir, "Add OAuth support");
    common::create_ticket(&dir, "Write docs");

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success(), "list failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "expected 3 lines, got: {:?}", lines);

    let titles: Vec<&str> = lines.iter().map(|l| l.trim()).collect();
    assert!(titles.iter().any(|l| l.contains("Fix login bug")));
    assert!(titles.iter().any(|l| l.contains("Add OAuth support")));
    assert!(titles.iter().any(|l| l.contains("Write docs")));
}

#[test]
fn list_sorted_by_status_then_created_at() {
    let dir = common::test_dir("list_sorted_by_status_then_created_at");
    common::tickets(&dir, &["init"]);

    // Create in reverse desired order: draft first, then in-progress
    let (id_draft, _) = common::create_ticket(&dir, "Draft ticket");
    common::tickets(&dir, &["edit", &id_draft, "--status", "draft"]);

    let (id_todo, _) = common::create_ticket(&dir, "Todo ticket");
    common::tickets(&dir, &["edit", &id_todo, "--status", "todo"]);

    let (id_active, _) = common::create_ticket(&dir, "Active ticket");
    common::tickets(&dir, &["edit", &id_active, "--status", "in-progress"]);

    let out = common::tickets(&dir, &["list"]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);

    // in-progress first, then todo, then draft
    assert!(lines[0].contains("Active ticket"), "first should be in-progress: {}", lines[0]);
    assert!(lines[1].contains("Todo ticket"), "second should be todo: {}", lines[1]);
    assert!(lines[2].contains("Draft ticket"), "third should be draft: {}", lines[2]);
}
