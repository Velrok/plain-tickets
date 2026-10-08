use crate::domain::status::Status;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Parsed from `.tickets/config.toml`.
///
/// `statuses` holds only the custom statuses, in display order. The first is
/// the default for new tickets. `Done` and `Rejected` are built in and always
/// sort last.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    statuses: Vec<Status>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    statuses: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            statuses: vec![
                Status::Custom("todo".into()),
                Status::Custom("in progress".into()),
            ],
        }
    }
}

impl Config {
    pub fn default_status(&self) -> &Status {
        &self.statuses[0]
    }

    /// Custom statuses in display order, without the built-ins.
    pub fn custom_statuses(&self) -> &[Status] {
        &self.statuses
    }

    /// True for the built-ins and every configured status.
    pub fn allows(&self, status: &Status) -> bool {
        status.is_terminal() || self.statuses.contains(status)
    }

    /// Display position: configured statuses in order, then `Done`, then `Rejected`.
    pub fn sort_key(&self, status: &Status) -> usize {
        match status {
            Status::Done => self.statuses.len(),
            Status::Rejected => self.statuses.len() + 1,
            custom => self
                .statuses
                .iter()
                .position(|s| s == custom)
                .unwrap_or(self.statuses.len() + 2),
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw = RawConfig {
            statuses: self.statuses.iter().map(|s| s.to_string()).collect(),
        };
        let toml = toml::to_string(&raw).expect("Config serialisation is infallible");
        f.write_str(&toml)
    }
}

const CONFIG_PATH: &str = ".tickets/config.toml";

/// The nearest `.tickets/config.toml` in `start` or any of its ancestors.
fn find(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|dir| dir.join(CONFIG_PATH))
        .find(|path| path.is_file())
}

/// The directory that holds the nearest `.tickets/`; tickets live under its `tickets/`.
pub fn root(start: &Path) -> Option<PathBuf> {
    let config = find(start)?;
    Some(config.parent()?.parent()?.to_path_buf())
}

/// Reads the nearest config. Errors if none exists; `init` creates one.
pub fn load(start: &Path) -> Result<Config, String> {
    let Some(path) = find(start) else {
        return Err("no .tickets/config.toml found; run `tickets init`".to_string());
    };
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    text.parse().map_err(|e| format!("{}: {e}", path.display()))
}

/// Writes the default config to `dir/.tickets/config.toml` and returns its
/// path. Fails if that file already exists.
pub fn init(dir: &Path) -> Result<PathBuf, String> {
    use std::io::Write;

    let path = dir.join(CONFIG_PATH);
    let parent = path.parent().expect("config path has a parent");
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => format!("{} already exists", path.display()),
            _ => format!("cannot write {}: {e}", path.display()),
        })?;
    file.write_all(Config::default().to_string().as_bytes())
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

impl std::str::FromStr for Config {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw: RawConfig = toml::from_str(s).map_err(|e| format!("invalid config: {e}"))?;
        let mut statuses: Vec<Status> = Vec::new();
        for name in &raw.statuses {
            let status: Status = name.parse()?;
            if status.is_terminal() {
                return Err(format!(
                    "'{status}' is built in and cannot be listed in statuses"
                ));
            }
            if statuses.contains(&status) {
                return Err(format!("duplicate status '{status}'"));
            }
            statuses.push(status);
        }
        if statuses.is_empty() {
            return Err("statuses must list at least one status".to_string());
        }
        Ok(Config { statuses })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom(s: &str) -> Status {
        Status::Custom(s.into())
    }

    #[test]
    fn parses_and_normalises_statuses() {
        let c: Config = r#"statuses = ["Todo", " In Review "]"#.parse().unwrap();
        assert_eq!(c.custom_statuses(), [custom("todo"), custom("in review")]);
    }

    #[test]
    fn default_status_is_the_first_entry() {
        let c: Config = r#"statuses = ["backlog", "todo"]"#.parse().unwrap();
        assert_eq!(c.default_status(), &custom("backlog"));
    }

    #[test]
    fn rejects_built_in_names() {
        for name in ["done", "Rejected"] {
            let err = format!(r#"statuses = ["todo", "{name}"]"#)
                .parse::<Config>()
                .unwrap_err();
            assert!(err.contains("built in"), "{err}");
        }
    }

    #[test]
    fn rejects_duplicates_after_normalisation() {
        let err = r#"statuses = ["todo", "TODO"]"#.parse::<Config>().unwrap_err();
        assert!(err.contains("duplicate"), "{err}");
    }

    #[test]
    fn rejects_empty_list() {
        assert!("statuses = []".parse::<Config>().is_err());
    }

    #[test]
    fn rejects_blank_status() {
        assert!(r#"statuses = ["todo", " "]"#.parse::<Config>().is_err());
    }

    #[test]
    fn rejects_unknown_keys() {
        assert!(r#"statues = ["todo"]"#.parse::<Config>().is_err());
        assert!(
            r#"statuses = ["todo"]
extra = 1"#
                .parse::<Config>()
                .is_err()
        );
    }

    #[test]
    fn rejects_missing_statuses_key() {
        assert!("".parse::<Config>().is_err());
    }

    #[test]
    fn default_config_round_trips_through_toml() {
        let text = Config::default().to_string();
        assert_eq!(text.parse::<Config>().unwrap(), Config::default());
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tickets-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn load_errors_without_writing_when_missing() {
        let dir = scratch("missing");
        let err = load(&dir).unwrap_err();
        assert!(err.contains("tickets init"), "{err}");
        assert!(!dir.join(".tickets").exists());
    }

    #[test]
    fn init_writes_the_default_config() {
        let dir = scratch("init");
        let path = init(&dir).unwrap();
        assert_eq!(path, dir.join(CONFIG_PATH));
        assert_eq!(load(&dir).unwrap(), Config::default());
    }

    #[test]
    fn init_refuses_to_overwrite() {
        let dir = scratch("init-twice");
        std::fs::create_dir_all(dir.join(".tickets")).unwrap();
        std::fs::write(dir.join(CONFIG_PATH), r#"statuses = ["backlog"]"#).unwrap();
        let err = init(&dir).unwrap_err();
        assert!(err.contains("already exists"), "{err}");
        assert_eq!(load(&dir).unwrap().custom_statuses(), [custom("backlog")]);
    }

    #[test]
    fn load_reads_existing_config_from_an_ancestor() {
        let dir = scratch("ancestor");
        std::fs::create_dir_all(dir.join(".tickets")).unwrap();
        std::fs::write(dir.join(CONFIG_PATH), r#"statuses = ["backlog"]"#).unwrap();
        let nested = dir.join("a/b");
        std::fs::create_dir_all(&nested).unwrap();
        let config = load(&nested).unwrap();
        assert_eq!(config.custom_statuses(), [custom("backlog")]);
        assert!(!nested.join(".tickets").exists());
    }

    #[test]
    fn load_reports_invalid_config_with_its_path() {
        let dir = scratch("invalid");
        std::fs::create_dir_all(dir.join(".tickets")).unwrap();
        std::fs::write(dir.join(CONFIG_PATH), "statuses = []").unwrap();
        let err = load(&dir).unwrap_err();
        assert!(err.contains("config.toml"), "{err}");
    }

    #[test]
    fn default_config_has_todo_and_in_progress() {
        let c = Config::default();
        assert_eq!(c.custom_statuses(), [custom("todo"), custom("in progress")]);
    }

    #[test]
    fn allows_built_ins_and_configured_only() {
        let c = Config::default();
        assert!(c.allows(&Status::Done));
        assert!(c.allows(&Status::Rejected));
        assert!(c.allows(&custom("todo")));
        assert!(!c.allows(&custom("blocked")));
    }

    #[test]
    fn sorts_configured_then_done_then_rejected_then_unknown() {
        let c = Config::default();
        let mut all = vec![
            custom("blocked"),
            Status::Rejected,
            Status::Done,
            custom("in progress"),
            custom("todo"),
        ];
        all.sort_by_key(|s| c.sort_key(s));
        assert_eq!(
            all,
            [
                custom("todo"),
                custom("in progress"),
                Status::Done,
                Status::Rejected,
                custom("blocked"),
            ]
        );
    }
}
