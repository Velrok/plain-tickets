use crate::domain::status::Status;
use serde::Deserialize;

/// Parsed from `.tickets/config.toml`.
///
/// `statuses` holds only the custom statuses, in display order. The first is
/// the default for new tickets. `Done` and `Rejected` are built in and always
/// sort last.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    statuses: Vec<Status>,
}

#[derive(Deserialize)]
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
