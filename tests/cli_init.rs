mod common;

#[test]
fn init_creates_subdirs() {
    let dir = common::test_dir("init_creates_subdirs");
    let out = common::tickets(&dir, &["init"]);
    assert!(out.status.success(), "init failed: {:?}", out);
    assert!(dir.join("all").is_dir(), "all/ not created");
    assert!(dir.join("archived").is_dir(), "archived/ not created");
}

#[test]
fn init_errors_if_already_initialised() {
    let dir = common::test_dir("init_errors_if_already_initialised");
    let first = common::tickets(&dir, &["init"]);
    assert!(first.status.success(), "first init failed: {:?}", first);
    let second = common::tickets(&dir, &["init"]);
    assert!(!second.status.success(), "second init should fail");
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(stderr.contains("error:"), "expected error: {stderr}");
}

#[test]
fn init_force_preserves_existing_values_while_rewriting() {
    let dir = common::test_dir("init_force_preserves_existing_values");
    let first = common::tickets(&dir, &["init"]);
    assert!(first.status.success(), "first init failed: {:?}", first);

    std::fs::write(dir.join(".tickets.toml"), "[git]\nauto_commit = true\n").unwrap();

    let forced = common::tickets(&dir, &["init", "--force"]);
    assert!(forced.status.success(), "forced init failed: {:?}", forced);

    let content = std::fs::read_to_string(dir.join(".tickets.toml")).unwrap();
    assert!(
        content.contains("auto_commit = true"),
        "expected preserved auto_commit = true, got:\n{content}"
    );
    assert!(
        content.contains("[tui]"),
        "expected backfilled [tui] section, got:\n{content}"
    );
}

#[test]
fn init_force_fails_and_leaves_file_untouched_when_existing_config_is_invalid() {
    let dir = common::test_dir("init_force_invalid_existing_config");
    let first = common::tickets(&dir, &["init"]);
    assert!(first.status.success(), "first init failed: {:?}", first);

    let invalid = "[git]\nunknown_key = true\n";
    std::fs::write(dir.join(".tickets.toml"), invalid).unwrap();

    let forced = common::tickets(&dir, &["init", "--force"]);
    assert!(
        !forced.status.success(),
        "forced init over invalid config should fail"
    );
    let stderr = String::from_utf8_lossy(&forced.stderr);
    assert!(stderr.contains("error:"), "expected error: {stderr}");

    let content = std::fs::read_to_string(dir.join(".tickets.toml")).unwrap();
    assert_eq!(
        content, invalid,
        "invalid config should be left untouched on failure"
    );
}

#[test]
fn init_force_with_no_existing_config_behaves_like_plain_init() {
    let dir = common::test_dir("init_force_no_existing_config");
    let out = common::tickets(&dir, &["init", "--force"]);
    assert!(out.status.success(), "init --force failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("created"),
        "expected 'created' in output: {stdout}"
    );
    assert!(
        !stdout.contains("rewrote"),
        "did not expect 'rewrote' in output: {stdout}"
    );
}
