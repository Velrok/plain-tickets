use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tickets-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn tickets(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tickets"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn initialised(name: &str) -> PathBuf {
    let dir = scratch(name);
    assert!(tickets(&dir, &["init"]).status.success());
    dir
}

fn new_id(dir: &Path, args: &[&str]) -> String {
    let mut full = vec!["new"];
    full.extend(args);
    let out = tickets(dir, &full);
    assert!(out.status.success(), "{}", stderr(&out));
    stdout(&out)
}

#[test]
fn new_prints_the_id_and_writes_the_ticket() {
    let dir = initialised("basic");
    let id = new_id(&dir, &["Fix it", "-m", "Some body", "-g", "a", "-g", "a"]);
    assert_eq!(id.len(), 16, "{id}");
    let text = std::fs::read_to_string(dir.join(format!("tickets/all/{id}.md"))).unwrap();
    assert!(text.contains("title: Fix it"), "{text}");
    assert!(text.contains("status: todo"), "{text}");
    assert!(text.ends_with("Some body"), "{text}");
}

#[test]
fn new_accepts_existing_parent_and_blockers() {
    let dir = initialised("links");
    let parent = new_id(&dir, &["Parent"]);
    let blocker = new_id(&dir, &["Blocker"]);
    let id = new_id(&dir, &["Child", "-p", &parent, "-b", &blocker]);
    let text = std::fs::read_to_string(dir.join(format!("tickets/all/{id}.md"))).unwrap();
    assert!(text.contains(&format!("parent: {parent}")), "{text}");
    assert!(text.contains(&blocker), "{text}");
}

#[test]
fn new_rejects_unknown_parent_and_blockers_without_writing() {
    let dir = initialised("unknown");
    let ghost = "00000000000000ff";
    for flag in ["-p", "-b"] {
        let out = tickets(&dir, &["new", "Child", flag, ghost]);
        assert!(!out.status.success());
        assert!(stderr(&out).contains(ghost), "{}", stderr(&out));
    }
    assert!(!dir.join("tickets").exists());
}

#[test]
fn new_rejects_a_blank_title() {
    let dir = initialised("blank");
    let out = tickets(&dir, &["new", "  "]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("title"), "{}", stderr(&out));
}

#[test]
fn new_works_from_a_nested_directory() {
    let dir = initialised("nested");
    let nested = dir.join("a/b");
    std::fs::create_dir_all(&nested).unwrap();
    let id = new_id(&nested, &["Deep"]);
    assert!(dir.join(format!("tickets/all/{id}.md")).is_file());
}

#[test]
fn new_sets_a_requested_status() {
    let dir = initialised("status");
    let id = new_id(&dir, &["Fix it", "-s", "In Progress"]);
    let text = std::fs::read_to_string(dir.join(format!("tickets/all/{id}.md"))).unwrap();
    assert!(text.contains("status: in progress"), "{text}");
}

#[test]
fn new_rejects_an_unconfigured_status_without_writing() {
    let dir = initialised("bad-status");
    let out = tickets(&dir, &["new", "Fix it", "--status", "blocked"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("blocked"), "{}", stderr(&out));
    assert!(!dir.join("tickets").exists());
}
