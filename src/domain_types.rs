use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(clap::ValueEnum, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TicketType {
    Epic,
    Story,
    #[default]
    Task,
    Bug,
}

impl std::fmt::Display for TicketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TicketType::Epic => "epic",
            TicketType::Story => "story",
            TicketType::Task => "task",
            TicketType::Bug => "bug",
        };
        f.write_str(s)
    }
}

#[derive(clap::ValueEnum, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TicketStatus {
    #[default]
    Draft,
    Todo,
    InProgress,
    Done,
    Rejected,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TicketStatus::Draft => "draft",
            TicketStatus::Todo => "todo",
            TicketStatus::InProgress => "in-progress",
            TicketStatus::Done => "done",
            TicketStatus::Rejected => "rejected",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TicketId(String);

impl TicketId {}

impl std::fmt::Display for TicketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for TicketId {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TicketId(s.to_string()))
    }
}

impl From<String> for TicketId {
    fn from(s: String) -> Self {
        TicketId(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Title(String);

const TITLE_MAX_LEN: usize = 120;

impl std::str::FromStr for Title {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err("title must not be empty".to_string());
        }
        if !trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || " _-.".contains(c))
        {
            return Err(format!(
                "invalid title {:?}: only letters, numbers, spaces, _, - and . are allowed",
                trimmed
            ));
        }
        if trimmed.len() > TITLE_MAX_LEN {
            return Err(format!(
                "title must be {TITLE_MAX_LEN} characters or fewer (got {})",
                trimmed.len()
            ));
        }
        Ok(Title(trimmed.to_string()))
    }
}

impl std::fmt::Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Title {
    pub fn slugify(&self) -> String {
        self.0
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tag(String);

impl std::str::FromStr for Tag {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("tag must not be empty".to_string());
        }
        if s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            Ok(Tag(s.to_string()))
        } else {
            Err(format!(
                "invalid tag {:?}: only letters, numbers, _ and - are allowed",
                s
            ))
        }
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Title::from_str ───────────────────────────────────────────────────────

    #[test]
    fn title_valid() {
        let t: Title = "Fix login bug".parse().unwrap();
        assert_eq!(t.to_string(), "Fix login bug");
    }

    #[test]
    fn title_trims_whitespace() {
        let t: Title = "  hello world  ".parse().unwrap();
        assert_eq!(t.to_string(), "hello world");
    }

    #[test]
    fn title_empty_is_err() {
        assert!("".parse::<Title>().is_err());
        assert!("   ".parse::<Title>().is_err());
    }

    #[test]
    fn title_invalid_chars_is_err() {
        assert!("foo!".parse::<Title>().is_err());
        assert!("foo@bar".parse::<Title>().is_err());
        assert!("foo/bar".parse::<Title>().is_err());
    }

    #[test]
    fn title_allowed_special_chars() {
        assert!("fix_login-bug.v2".parse::<Title>().is_ok());
    }

    #[test]
    fn title_over_120_chars_is_err() {
        let long = "a".repeat(121);
        assert!(long.parse::<Title>().is_err());
    }

    #[test]
    fn title_exactly_120_chars_is_ok() {
        let exactly = "a".repeat(120);
        assert!(exactly.parse::<Title>().is_ok());
    }

    // ── Title::slugify ────────────────────────────────────────────────────────

    #[test]
    fn slugify_lowercases() {
        let t: Title = "Fix Login Bug".parse().unwrap();
        assert_eq!(t.slugify(), "fix-login-bug");
    }

    #[test]
    fn slugify_replaces_spaces_with_dashes() {
        let t: Title = "hello world".parse().unwrap();
        assert_eq!(t.slugify(), "hello-world");
    }

    #[test]
    fn slugify_collapses_consecutive_separators() {
        let t: Title = "foo - bar".parse().unwrap();
        assert_eq!(t.slugify(), "foo-bar");
    }

    #[test]
    fn slugify_handles_underscores_and_dots() {
        let t: Title = "foo_bar.baz".parse().unwrap();
        assert_eq!(t.slugify(), "foo-bar-baz");
    }

    // ── Ticket parse / display ────────────────────────────────────────────────

    fn minimal_ticket_str() -> &'static str {
        "---\nid: abc123\ntitle: My ticket\ntype: task\nstatus: draft\ntags: []\nparent: null\nblocked_by: []\ncreated_at: 2024-01-01T00:00:00Z\nupdated_at: 2024-01-01T00:00:00Z\n---\n"
    }

    #[test]
    fn ticket_parses_without_body() {
        let t: Ticket = minimal_ticket_str().parse().unwrap();
        assert_eq!(t.front_matter.title.to_string(), "My ticket");
        assert!(t.body.is_empty());
    }

    #[test]
    fn ticket_parses_with_body() {
        let s = format!("{}\nSome body text.", minimal_ticket_str());
        let t: Ticket = s.parse().unwrap();
        assert_eq!(t.body, "Some body text.");
    }

    #[test]
    fn ticket_display_round_trips() {
        let original = minimal_ticket_str();
        let t: Ticket = original.parse().unwrap();
        assert_eq!(t.to_string(), original);
    }

    #[test]
    fn ticket_display_round_trips_with_body() {
        let s = format!("{}\nSome body text.", minimal_ticket_str());
        let t: Ticket = s.parse().unwrap();
        assert_eq!(t.to_string(), s);
    }

    #[test]
    fn ticket_missing_closing_delimiter_is_err() {
        let bad = "---\nid: abc123\ntitle: My ticket\n";
        assert!(bad.parse::<Ticket>().is_err());
    }

    #[test]
    fn ticket_malformed_yaml_is_err() {
        let bad = "---\nnot: valid: yaml: here:\n---\n";
        assert!(bad.parse::<Ticket>().is_err());
    }

    // ── Tag::from_str ─────────────────────────────────────────────────────────

    #[test]
    fn tag_valid() {
        assert!("auth".parse::<Tag>().is_ok());
        assert!("my-tag".parse::<Tag>().is_ok());
        assert!("tag_1".parse::<Tag>().is_ok());
    }

    #[test]
    fn tag_empty_is_err() {
        assert!("".parse::<Tag>().is_err());
    }

    #[test]
    fn tag_space_is_err() {
        assert!("foo bar".parse::<Tag>().is_err());
    }

    #[test]
    fn tag_special_chars_are_err() {
        assert!("foo!".parse::<Tag>().is_err());
        assert!("foo.bar".parse::<Tag>().is_err());
    }
}

#[derive(Serialize, Deserialize)]
pub struct FrontMatter {
    pub id: TicketId,
    pub title: Title,
    pub r#type: TicketType,
    pub status: TicketStatus,
    pub tags: Vec<Tag>,
    pub parent: Option<TicketId>,
    pub blocked_by: Vec<TicketId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Ticket {
    pub front_matter: FrontMatter,
    pub body: String,
}

impl std::str::FromStr for Ticket {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("---\n").unwrap_or(s);
        let end = s
            .find("\n---\n")
            .ok_or_else(|| "missing front matter closing delimiter".to_string())?;
        let yaml = &s[..end];
        let body = s[end + 5..].trim_start_matches('\n').to_string();
        let front_matter: FrontMatter = serde_yaml::from_str(yaml)
            .map_err(|e| format!("invalid front matter: {e}"))?;
        Ok(Ticket { front_matter, body })
    }
}

impl std::fmt::Display for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let yaml = serde_yaml::to_string(&self.front_matter)
            .expect("FrontMatter serialisation is infallible");
        if self.body.is_empty() {
            write!(f, "---\n{}---\n", yaml)
        } else {
            write!(f, "---\n{}---\n\n{}", yaml, self.body)
        }
    }
}
