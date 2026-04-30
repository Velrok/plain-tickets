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

#[derive(clap::ValueEnum, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TicketStatus {
    #[default]
    Draft,
    Todo,
    InProgress,
    Done,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TicketId(String);

impl TicketId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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
