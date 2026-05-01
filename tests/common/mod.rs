use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps/
    path.pop(); // debug/
    path.push("tickets");
    path
}

/// Create an isolated test directory under `.testing/<name>/`.
/// Any previous run is wiped first.
pub fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".testing")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn tickets(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .env("TICKETS_DIR", dir)
        .output()
        .expect("failed to run tickets binary")
}

/// Run `init` then `new --title <title>`. Returns `(id, filename)`.
pub fn create_ticket(dir: &Path, title: &str) -> (String, String) {
    let out = tickets(dir, &["new", "--title", title]);
    assert!(out.status.success(), "create_ticket failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut parts = stdout.trim().splitn(2, ' ');
    let id = parts.next().unwrap().to_string();
    let filename = parts.next().unwrap().to_string();
    (id, filename)
}
