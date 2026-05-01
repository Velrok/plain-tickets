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
fn init_is_idempotent() {
    let dir = common::test_dir("init_is_idempotent");
    common::tickets(&dir, &["init"]);
    let out = common::tickets(&dir, &["init"]);
    assert!(out.status.success(), "second init failed: {:?}", out);
    assert!(dir.join("all").is_dir());
    assert!(dir.join("archived").is_dir());
}
