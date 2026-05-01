use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub enum GitError {
    NotInstalled,
    NotARepo(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::NotInstalled => write!(f, "error: git not found on PATH"),
            GitError::NotARepo(msg) => write!(f, "error: not a git repository: {msg}"),
        }
    }
}

/// Returns `Ok(())` if `dir` (or any ancestor) is inside a git repo.
pub fn git_detect(dir: &Path) -> Result<(), GitError> {
    let output = Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "rev-parse", "--git-dir"])
        .output()
        .map_err(|_| GitError::NotInstalled)?;

    if output.status.success() {
        Ok(())
    } else {
        let msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(GitError::NotARepo(msg))
    }
}

/// Stages `file` then creates a commit with `message`.
/// Git's stderr is piped directly to the calling process's stderr.
pub fn git_commit(dir: &Path, file: &Path, message: &str) -> Result<(), String> {
    let dir_s = dir.to_string_lossy();
    let file_s = file.to_string_lossy();

    let add = Command::new("git")
        .args(["-C", &dir_s, "add", "--", &file_s])
        .status()
        .map_err(|e| format!("error: failed to run git add: {e}"))?;

    if !add.success() {
        return Err(format!("error: git add failed (exit {})", add.code().unwrap_or(1)));
    }

    let commit = Command::new("git")
        .args(["-C", &dir_s, "commit", "-m", message, "--", &file_s])
        .status()
        .map_err(|e| format!("error: failed to run git commit: {e}"))?;

    if commit.success() {
        Ok(())
    } else {
        Err(format!("error: git commit failed (exit {})", commit.code().unwrap_or(1)))
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
    fn detect_err_not_a_repo() {
        // Must be outside the project tree so no ancestor .git is found
        let dir = PathBuf::from("/tmp/plain-tickets-test-not-a-repo");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let err = git_detect(&dir).unwrap_err();
        assert!(matches!(err, GitError::NotARepo(_)), "expected NotARepo, got {err:?}");
    }
}
