mod common;

use common::{bin, test_dir};
use std::process::Command;

/// Run tickets with --dir flag but WITHOUT setting TICKETS_DIR env var.
fn tickets_dir(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    let mut full_args = vec!["--dir", dir.to_str().unwrap()];
    full_args.extend_from_slice(args);
    Command::new(bin())
        .args(&full_args)
        .output()
        .expect("failed to run tickets binary")
}

#[test]
fn dir_flag_new_creates_ticket_in_specified_dir() {
    let dir = test_dir("dir_flag_new");
    tickets_dir(&dir, &["init"]);

    let out = tickets_dir(&dir, &["new", "--title", "hello world"]);
    assert!(out.status.success(), "new failed: {:?}", out);

    let all = dir.join("all");
    let entries: Vec<_> = std::fs::read_dir(&all).unwrap().flatten().collect();
    assert_eq!(entries.len(), 1, "expected one ticket file in {:?}", all);
}

#[test]
fn dir_flag_init_creates_dirs_in_specified_dir() {
    let dir = test_dir("dir_flag_init");
    let out = tickets_dir(&dir, &["init"]);
    assert!(out.status.success(), "init failed: {:?}", out);
    assert!(dir.join("all").is_dir());
    assert!(dir.join("archived").is_dir());
}

#[test]
fn dir_flag_edit_updates_ticket_in_specified_dir() {
    let dir = test_dir("dir_flag_edit");
    tickets_dir(&dir, &["init"]);

    let out = tickets_dir(&dir, &["new", "--title", "original title"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let id = stdout.trim().splitn(2, ' ').next().unwrap().to_string();

    let out = tickets_dir(&dir, &["edit", &id, "--title", "updated title"]);
    assert!(out.status.success(), "edit failed: {:?}", out);

    let all = dir.join("all");
    let content = std::fs::read_dir(&all)
        .unwrap()
        .flatten()
        .map(|e| std::fs::read_to_string(e.path()).unwrap())
        .next()
        .unwrap();
    assert!(content.contains("updated title"), "title not updated in file");
}

#[test]
fn dir_flag_takes_precedence_over_tickets_dir_env() {
    let env_dir = test_dir("dir_flag_precedence_env");
    let flag_dir = test_dir("dir_flag_precedence_flag");

    // Init both dirs so commands don't fail on missing structure
    tickets_dir(&env_dir, &["init"]);
    tickets_dir(&flag_dir, &["init"]);

    // Run with TICKETS_DIR pointing at env_dir but --dir pointing at flag_dir
    let out = Command::new(bin())
        .args(["--dir", flag_dir.to_str().unwrap(), "new", "--title", "precedence test"])
        .env("TICKETS_DIR", &env_dir)
        .output()
        .expect("failed to run tickets binary");
    assert!(out.status.success(), "new failed: {:?}", out);

    let flag_tickets: Vec<_> = std::fs::read_dir(flag_dir.join("all")).unwrap().flatten().collect();
    let env_tickets: Vec<_> = std::fs::read_dir(env_dir.join("all")).unwrap().flatten().collect();
    assert_eq!(flag_tickets.len(), 1, "ticket should be in --dir path");
    assert_eq!(env_tickets.len(), 0, "TICKETS_DIR path should be empty");
}
