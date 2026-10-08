#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tickets-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn tickets(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tickets"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

pub fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

pub fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

pub fn initialised(name: &str) -> PathBuf {
    let dir = scratch(name);
    assert!(tickets(&dir, &["init"]).status.success());
    dir
}

pub fn new_id(dir: &Path, args: &[&str]) -> String {
    let mut full = vec!["new"];
    full.extend(args);
    let out = tickets(dir, &full);
    assert!(out.status.success(), "{}", stderr(&out));
    stdout(&out)
}
