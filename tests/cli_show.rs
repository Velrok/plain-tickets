mod common;

#[test]
fn show_errors_when_not_initialised() {
    let dir = common::test_dir("show_errors_when_not_initialised");
    let out = common::tickets(&dir, &["show", "abc123"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not initialised"), "unexpected stderr: {}", stderr);
}

#[test]
fn show_errors_when_id_not_found() {
    let dir = common::test_dir("show_errors_when_id_not_found");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["show", "xxxxxx"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("xxxxxx"), "expected id in error: {}", stderr);
}

#[test]
fn show_prints_fields_with_emojis() {
    let dir = common::test_dir("show_prints_fields_with_emojis");
    common::tickets(&dir, &["init"]);
    let (id, _) = common::create_ticket(&dir, "Fix login bug");

    let out = common::tickets(&dir, &["show", &id]);
    assert!(out.status.success(), "show failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("🎫"), "expected title emoji");
    assert!(stdout.contains("Fix login bug"), "expected title");
    assert!(stdout.contains("📌"), "expected status emoji");
    assert!(stdout.contains("draft"), "expected status");
    assert!(stdout.contains("🏷"), "expected type emoji");
    assert!(stdout.contains("task"), "expected type");
    assert!(stdout.contains("📅"), "expected timestamp emoji");
}

#[test]
fn show_omits_empty_optional_fields() {
    let dir = common::test_dir("show_omits_empty_optional_fields");
    common::tickets(&dir, &["init"]);
    let (id, _) = common::create_ticket(&dir, "Plain ticket");

    let out = common::tickets(&dir, &["show", &id]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("🔖"), "tags emoji should be absent");
    assert!(!stdout.contains("⬆"), "parent emoji should be absent");
    assert!(!stdout.contains("🚫"), "blocked-by emoji should be absent");
}

#[test]
fn show_displays_tags_and_parent_when_set() {
    let dir = common::test_dir("show_displays_tags_and_parent_when_set");
    common::tickets(&dir, &["init"]);
    let (parent_id, _) = common::create_ticket(&dir, "Parent ticket");
    let out = common::tickets(&dir, &["new", "--title", "Child ticket", "--tag", "auth", "--tag", "backend", "--parent", &parent_id]);
    assert!(out.status.success());
    let child_id = String::from_utf8_lossy(&out.stdout).split_whitespace().next().unwrap().to_string();

    let out = common::tickets(&dir, &["show", &child_id]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("🔖"), "expected tags emoji");
    assert!(stdout.contains("auth"), "expected tag auth");
    assert!(stdout.contains("backend"), "expected tag backend");
    assert!(stdout.contains("⬆"), "expected parent emoji");
    assert!(stdout.contains(&parent_id), "expected parent id");
}

#[test]
fn show_displays_body_when_present() {
    let dir = common::test_dir("show_displays_body_when_present");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["new", "--title", "Ticket with body", "--body", "Some description here."]);
    assert!(out.status.success());
    let id = String::from_utf8_lossy(&out.stdout).split_whitespace().next().unwrap().to_string();

    let out = common::tickets(&dir, &["show", &id]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Some description here."), "expected body text");
}

#[test]
fn show_timestamps_include_date_and_relative() {
    let dir = common::test_dir("show_timestamps_include_date_and_relative");
    common::tickets(&dir, &["init"]);
    let (id, _) = common::create_ticket(&dir, "Timestamp test");

    let out = common::tickets(&dir, &["show", &id]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Date format YYYY-MM-DD
    assert!(stdout.contains("2026-"), "expected date in output");
    // Relative time separator
    assert!(stdout.contains(" · "), "expected · separator");
    // Relative time (just created)
    assert!(stdout.contains("just now") || stdout.contains("ago"), "expected relative time");
}
