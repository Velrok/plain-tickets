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

#[cfg(unix)]
/// Runs `tickets` with `$EDITOR` set to a script that executes `body` with
/// the ticket file as `$1`. `$VISUAL` is cleared.
pub fn tickets_editing(dir: &Path, args: &[&str], body: &str) -> Output {
    use std::os::unix::fs::PermissionsExt;
    let script = dir.join("editor.sh");
    std::fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    Command::new(env!("CARGO_BIN_EXE_tickets"))
        .args(args)
        .current_dir(dir)
        .env_remove("VISUAL")
        .env("EDITOR", &script)
        .output()
        .unwrap()
}

pub fn ticket_text(dir: &Path, id: &str) -> String {
    std::fs::read_to_string(dir.join(format!("tickets/all/{id}.md"))).unwrap()
}

/// Shell snippet that replaces the title line of the file in `$1`.
pub fn set_title(title: &str) -> String {
    format!("sed 's/^title: .*/title: {title}/' \"$1\" > \"$1.new\" && mv \"$1.new\" \"$1\"")
}

/// Sets the ticket's `updated_at` to a fixed old time on disk and returns the new text, so a
/// later rewrite is distinguishable from an untouched file.
pub fn age_ticket(dir: &Path, id: &str) -> String {
    let text = ticket_text(dir, id);
    let old = text
        .lines()
        .find(|l| l.starts_with("updated_at: "))
        .unwrap()
        .to_string();
    let aged = text.replace(&old, "updated_at: 1970-01-01T00:00:05.000Z");
    std::fs::write(dir.join(format!("tickets/all/{id}.md")), &aged).unwrap();
    aged
}

/// Like `tickets`, with extra environment variables set.
pub fn tickets_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tickets"))
        .args(args)
        .current_dir(dir)
        .envs(env.iter().copied())
        .output()
        .unwrap()
}

/// Like `tickets`, with `input` piped to STDIN.
pub fn tickets_stdin(dir: &Path, args: &[&str], input: &str) -> Output {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_tickets"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
