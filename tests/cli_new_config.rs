mod common;

use std::fs;

// ── default_type ──────────────────────────────────────────────────────────────

#[test]
fn new_uses_config_default_status() {
    let dir = common::test_dir("new_uses_config_default_status");
    common::tickets(&dir, &["init"]);
    fs::write(
        dir.join(".tickets.toml"),
        "[new]\ndefault_status = \"todo\"\n",
    )
    .unwrap();

    let out = common::tickets(&dir, &["new", "--title", "My ticket"]);
    assert!(out.status.success(), "new failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let filename = stdout.trim().splitn(2, ' ').nth(1).unwrap();
    let content = fs::read_to_string(dir.join("all").join(filename)).unwrap();
    assert!(content.contains("status: todo"), "expected todo status: {}", content);
}

#[test]
fn new_uses_config_default_type() {
    let dir = common::test_dir("new_uses_config_default_type");
    common::tickets(&dir, &["init"]);
    fs::write(dir.join(".tickets.toml"), "[new]\ndefault_type = \"bug\"\n").unwrap();

    let out = common::tickets(&dir, &["new", "--title", "My ticket"]);
    assert!(out.status.success(), "new failed: {:?}", out);

    let filename = String::from_utf8_lossy(&out.stdout).trim().splitn(2, ' ').nth(1).unwrap().to_string();
    let content = fs::read_to_string(dir.join("all").join(filename)).unwrap();
    assert!(content.contains("type: bug"), "expected bug type: {}", content);
}

#[test]
fn explicit_flag_overrides_config_default() {
    let dir = common::test_dir("explicit_flag_overrides_config_default");
    common::tickets(&dir, &["init"]);
    fs::write(
        dir.join(".tickets.toml"),
        "[new]\ndefault_status = \"todo\"\ndefault_type = \"bug\"\n",
    )
    .unwrap();

    let out = common::tickets(&dir, &["new", "--title", "My ticket", "--status", "draft", "--type", "epic"]);
    assert!(out.status.success(), "new failed: {:?}", out);

    let filename = String::from_utf8_lossy(&out.stdout).trim().splitn(2, ' ').nth(1).unwrap().to_string();
    let content = fs::read_to_string(dir.join("all").join(filename)).unwrap();
    assert!(content.contains("status: draft"), "explicit flag should override: {}", content);
    assert!(content.contains("type: epic"), "explicit flag should override: {}", content);
}

#[test]
fn no_config_falls_back_to_hardcoded_defaults() {
    let dir = common::test_dir("no_config_falls_back_to_hardcoded_defaults");
    common::tickets(&dir, &["init"]);

    let out = common::tickets(&dir, &["new", "--title", "My ticket"]);
    assert!(out.status.success(), "new failed: {:?}", out);

    let filename = String::from_utf8_lossy(&out.stdout).trim().splitn(2, ' ').nth(1).unwrap().to_string();
    let content = fs::read_to_string(dir.join("all").join(filename)).unwrap();
    assert!(content.contains("status: draft"), "expected draft default: {}", content);
    assert!(content.contains("type: task"), "expected task default: {}", content);
}

#[test]
fn init_scaffolds_new_section_in_config() {
    let dir = common::test_dir("init_scaffolds_new_section_in_config");
    common::tickets(&dir, &["init"]);

    let content = fs::read_to_string(dir.join(".tickets.toml")).unwrap();
    assert!(content.contains("new"), "expected [new] section mention: {}", content);
    assert!(content.contains("default_status") || content.contains("default_type"),
        "expected default_status or default_type mention: {}", content);
}
