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
