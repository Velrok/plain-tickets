use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};

/// Returns `Ok(())` if `dir` (or any ancestor) is inside a git repo.
pub fn git_detect(dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "rev-parse", "--git-dir"])
        .output()
        .map_err(|_| anyhow::anyhow!("git not found on PATH"))?;

    if output.status.success() {
        Ok(())
    } else {
        let msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!("not a git repository: {msg}");
    }
}

/// Stages `file` (relative to `repo_root`) then creates a commit with `message`.
/// `repo_root` must be the root of the git repository.
pub fn git_commit(repo_root: &Path, file: &Path, message: &str) -> Result<()> {
    let file_s = file.to_string_lossy();

    let add = Command::new("git")
        .current_dir(repo_root)
        .args(["add", "--", &file_s])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git add: {e}"))?;

    if !add.success() {
        bail!("git add failed (exit {})", add.code().unwrap_or(1));
    }

    let commit = Command::new("git")
        .current_dir(repo_root)
        .args(["commit", "-m", message])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git commit: {e}"))?;

    if commit.success() {
        Ok(())
    } else {
        bail!("git commit failed (exit {})", commit.code().unwrap_or(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".testing")
            .join(format!("git_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn init_git_repo(dir: &Path) {
        Command::new("git").args(["init"]).current_dir(dir).status().unwrap();
        Command::new("git").args(["config", "user.email", "test@example.com"]).current_dir(dir).status().unwrap();
        Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).status().unwrap();
    }

    #[test]
    fn detect_ok_in_git_repo() {
        let dir = tmp_dir("detect_ok");
        init_git_repo(&dir);
        assert!(git_detect(&dir).is_ok());
    }

    #[test]
    fn commit_file_in_subdirectory() {
        // Regression: git_commit was called with the tickets subdir as repo_root,
        // causing git to look for `tickets/tickets/archived/foo.md` instead of
        // `tickets/archived/foo.md`.
        let repo = tmp_dir("commit_subdir");
        init_git_repo(&repo);

        // Create tickets/archived/ structure inside the repo
        let archived = repo.join("tickets").join("archived");
        fs::create_dir_all(&archived).unwrap();

        let file = archived.join("abc123_some-ticket.md");
        fs::write(&file, "# hello").unwrap();

        // file path is relative to repo root (as produced by commands.rs)
        let rel = PathBuf::from("tickets/archived/abc123_some-ticket.md");

        let result = git_commit(&repo, &rel, "tickets: archive abc123");
        assert!(result.is_ok(), "git_commit failed: {:?}", result.unwrap_err());
    }

    #[test]
    fn detect_err_not_a_repo() {
        // Must be outside the project tree so no ancestor .git is found
        let dir = PathBuf::from("/tmp/plain-tickets-test-not-a-repo");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let err = git_detect(&dir).unwrap_err();
        assert!(err.to_string().contains("not a git repository"), "unexpected error: {err:?}");
    }
}
