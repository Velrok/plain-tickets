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
#[allow(dead_code)]
pub fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".testing")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[allow(dead_code)]
pub fn tickets(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .env("TICKETS_DIR", dir)
        .output()
        .expect("failed to run tickets binary")
}

/// Like [`tickets`], but with extra environment variables set on top of
/// `TICKETS_DIR` — for exercising colour env vars (`NO_COLOR`,
/// `CLICOLOR_FORCE`) without disturbing the plain `tickets` helper used
/// everywhere else.
#[allow(dead_code)]
pub fn tickets_with_envs(dir: &Path, args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .env("TICKETS_DIR", dir)
        .envs(envs.iter().copied())
        .output()
        .expect("failed to run tickets binary")
}

/// Run `init` then `new --title <title>`. Returns `(id, filename)`.
#[allow(dead_code)]
pub fn create_ticket(dir: &Path, title: &str) -> (String, String) {
    let out = tickets(dir, &["new", "--title", title]);
    assert!(out.status.success(), "create_ticket failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut parts = stdout.trim().splitn(2, ' ');
    let id = parts.next().unwrap().to_string();
    let filename = parts.next().unwrap().to_string();
    (id, filename)
}

/// Overwrites the already-archived ticket `id`'s file under `dir/archived/`
/// with front matter that will not parse as a `Ticket` — used to reproduce
/// `o3c87i`: a corrupted file in `archived/` named as a blocker. Returns the
/// file's name so callers can assert stderr names it.
#[allow(dead_code)]
pub fn corrupt_archived_file(dir: &Path, id: &str) -> String {
    let archived = dir.join("archived");
    let prefix = format!("{id}_");
    let entry = fs::read_dir(&archived)
        .unwrap_or_else(|e| panic!("could not read {}: {e}", archived.display()))
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .unwrap_or_else(|| panic!("no archived file found with id {id}"));
    let path = entry.path();
    fs::write(&path, "---\nstatus: nonsense-status\n---\n").unwrap();
    entry.file_name().to_string_lossy().into_owned()
}
