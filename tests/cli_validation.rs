mod common;

// ── title validation ──────────────────────────────────────────────────────────

#[test]
fn new_empty_title_fails() {
    let dir = common::test_dir("validation_empty_title");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["new", "--title", ""]);
    assert!(!out.status.success());
}

#[test]
fn new_title_with_invalid_chars_fails() {
    let dir = common::test_dir("validation_title_invalid_chars");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["new", "--title", "foo!"]);
    assert!(!out.status.success());
}

#[test]
fn new_title_over_120_chars_fails() {
    let dir = common::test_dir("validation_title_too_long");
    common::tickets(&dir, &["init"]);
    let long = "a".repeat(121);
    let out = common::tickets(&dir, &["new", "--title", &long]);
    assert!(!out.status.success());
}

// ── tag validation ────────────────────────────────────────────────────────────

#[test]
fn new_tag_with_space_fails() {
    let dir = common::test_dir("validation_tag_with_space");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["new", "--title", "Valid title", "--tag", "foo bar"]);
    assert!(!out.status.success());
}

#[test]
fn new_tag_with_special_chars_fails() {
    let dir = common::test_dir("validation_tag_special_chars");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["new", "--title", "Valid title", "--tag", "foo!"]);
    assert!(!out.status.success());
}

// ── enum validation ───────────────────────────────────────────────────────────

#[test]
fn new_invalid_type_fails() {
    let dir = common::test_dir("validation_invalid_type");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["new", "--title", "Valid title", "--type", "banana"]);
    assert!(!out.status.success());
}

#[test]
fn new_invalid_status_fails() {
    let dir = common::test_dir("validation_invalid_status");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["new", "--title", "Valid title", "--status", "open"]);
    assert!(!out.status.success());
}
