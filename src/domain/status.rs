use serde::{Deserialize, Serialize};

/// `Done` and `Rejected` are built in; every other status comes from config.
/// Always stored trimmed and lowercase, so `Custom` never holds a built-in name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Done,
    Rejected,
    Custom(String),
}

impl Status {
    /// Terminal statuses end a ticket's life: they unblock dependents and can be archived.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Status::Done | Status::Rejected)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Status::Done => "done",
            Status::Rejected => "rejected",
            Status::Custom(s) => s,
        }
    }
}

impl std::str::FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "" => Err("status must not be empty".to_string()),
            "done" => Ok(Status::Done),
            "rejected" => Ok(Status::Rejected),
            _ => Ok(Status::Custom(s)),
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Status {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_built_ins_case_insensitively() {
        assert_eq!("Done".parse(), Ok(Status::Done));
        assert_eq!(" REJECTED ".parse(), Ok(Status::Rejected));
    }

    #[test]
    fn normalises_custom_statuses() {
        assert_eq!(
            "  In Review ".parse(),
            Ok(Status::Custom("in review".into()))
        );
    }

    #[test]
    fn rejects_empty() {
        assert!("".parse::<Status>().is_err());
        assert!("   ".parse::<Status>().is_err());
    }

    #[test]
    fn only_done_and_rejected_are_terminal() {
        assert!(Status::Done.is_terminal());
        assert!(Status::Rejected.is_terminal());
        assert!(!Status::Custom("todo".into()).is_terminal());
    }

    #[test]
    fn serialises_as_plain_lowercase_string() {
        assert_eq!(serde_yaml::to_string(&Status::Done).unwrap(), "done\n");
        let custom = Status::Custom("in progress".into());
        assert_eq!(serde_yaml::to_string(&custom).unwrap(), "in progress\n");
    }

    #[test]
    fn deserialises_and_normalises() {
        let s: Status = serde_yaml::from_str("Done").unwrap();
        assert_eq!(s, Status::Done);
        let s: Status = serde_yaml::from_str("In Progress").unwrap();
        assert_eq!(s, Status::Custom("in progress".into()));
    }

    #[test]
    fn deserialising_empty_fails() {
        assert!(serde_yaml::from_str::<Status>("''").is_err());
    }
}
