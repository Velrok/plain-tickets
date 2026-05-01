mod common;

use std::fs;

#[test]
fn init_creates_config_file() {
    let dir = common::test_dir("config_init_creates");
    let out = common::tickets(&dir, &["init"]);
    assert!(out.status.success(), "init failed: {:?}", out);
    assert!(dir.join(".tickets.toml").exists(), ".tickets.toml not created");
}

#[test]
fn init_config_is_valid_toml() {
    let dir = common::test_dir("config_init_valid_toml");
    common::tickets(&dir, &["init"]);
    let content = fs::read_to_string(dir.join(".tickets.toml")).unwrap();
    // Must be parseable as TOML (even if all lines are comments)
    let parsed: toml::Value = toml::from_str(&content).expect("created .tickets.toml is not valid TOML");
    drop(parsed);
}

#[test]
fn init_errors_if_already_initialised() {
    let dir = common::test_dir("config_init_already_initialised");
    let first = common::tickets(&dir, &["init"]);
    assert!(first.status.success(), "first init failed: {:?}", first);
    let second = common::tickets(&dir, &["init"]);
    assert!(!second.status.success(), "second init should fail");
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(stderr.contains("error:"), "expected error message, got: {stderr}");
}
