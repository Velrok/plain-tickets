use std::path::Path;

use anyhow::{Context as _, Result};

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub tui: TuiConfig,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TuiConfig {
    #[serde(default = "TuiConfig::default_kanban_columns")]
    pub kanban_columns: Vec<String>,
}

impl TuiConfig {
    fn default_kanban_columns() -> Vec<String> {
        vec!["todo".to_string(), "in-progress".to_string(), "done".to_string()]
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self { kanban_columns: Self::default_kanban_columns() }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitConfig {
    #[serde(default)]
    pub auto_commit: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { git: GitConfig::default(), tui: TuiConfig::default() }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        Self { auto_commit: false }
    }
}

pub fn load(dir: &Path) -> Result<Config> {
    let path = dir.join(".tickets.toml");
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)
        .context("failed to read .tickets.toml")?;
    toml::from_str(&content)
        .context("invalid .tickets.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".testing")
            .join(format!("config_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_file_returns_defaults() {
        let dir = tmp_dir("missing");
        let cfg = load(&dir).unwrap();
        assert!(!cfg.git.auto_commit);
    }

    #[test]
    fn auto_commit_true_is_parsed() {
        let dir = tmp_dir("auto_commit_true");
        fs::write(dir.join(".tickets.toml"), "[git]\nauto_commit = true\n").unwrap();
        let cfg = load(&dir).unwrap();
        assert!(cfg.git.auto_commit);
    }

    #[test]
    fn invalid_toml_is_error() {
        let dir = tmp_dir("invalid_toml");
        fs::write(dir.join(".tickets.toml"), "not toml :::").unwrap();
        let err = load(&dir).unwrap_err();
        assert!(!err.to_string().is_empty(), "expected error, got empty: {err}");
    }

    #[test]
    fn tui_kanban_columns_defaults() {
        let dir = tmp_dir("tui_defaults");
        let cfg = load(&dir).unwrap();
        assert_eq!(cfg.tui.kanban_columns, vec!["todo", "in-progress", "done"]);
    }

    #[test]
    fn tui_kanban_columns_parsed_from_config() {
        let dir = tmp_dir("tui_columns_parsed");
        fs::write(
            dir.join(".tickets.toml"),
            "[tui]\nkanban_columns = [\"backlog\", \"active\", \"closed\"]\n",
        )
        .unwrap();
        let cfg = load(&dir).unwrap();
        assert_eq!(cfg.tui.kanban_columns, vec!["backlog", "active", "closed"]);
    }

    #[test]
    fn unknown_field_is_error() {
        let dir = tmp_dir("unknown_field");
        fs::write(dir.join(".tickets.toml"), "[git]\nunknown_key = true\n").unwrap();
        let err = load(&dir).unwrap_err();
        assert!(!err.to_string().is_empty(), "expected error, got empty: {err}");
    }
}
