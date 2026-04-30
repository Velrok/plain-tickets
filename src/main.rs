use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(name = "tickets", about = "Plain markdown ticket system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise tickets directory
    Init,
    /// Create a new ticket
    New {
        /// Ticket title (required, max 120 chars)
        #[arg(long)]
        title: Title,
        /// Ticket type
        #[arg(long, default_value = "task")]
        r#type: TicketType,
        /// Tags (repeatable; letters, numbers, _ and - only)
        #[arg(long)]
        tag: Vec<Tag>,
        /// Parent ticket id
        #[arg(long)]
        parent: Option<TicketId>,
        /// Blocked-by ticket ids (repeatable)
        #[arg(long)]
        blocked_by: Vec<TicketId>,
        /// Initial status (default: draft)
        #[arg(long, default_value = "draft")]
        status: TicketStatus,
    },
}

#[derive(clap::ValueEnum, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
enum TicketType {
    Epic,
    Story,
    #[default]
    Task,
    Bug,
}

#[derive(clap::ValueEnum, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
enum TicketStatus {
    #[default]
    Draft,
    Todo,
    InProgress,
    Done,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct TicketId(String);

impl TicketId {
    fn as_str(&self) -> &str {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct Title(String);

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct Tag(String);

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
struct FrontMatter {
    id: TicketId,
    title: Title,
    r#type: TicketType,
    status: TicketStatus,
    tags: Vec<Tag>,
    parent: Option<TicketId>,
    blocked_by: Vec<TicketId>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn resolve_dir() -> PathBuf {
    std::env::var("TICKETS_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tickets"))
}

impl Title {
    fn slugify(&self) -> String {
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

fn init_directories(dir: &Path) {
    let all = dir.join("all");
    let archived = dir.join("archived");

    for path in [&all, &archived] {
        if path.exists() {
            println!("  exists  {}", path.display());
        } else {
            std::fs::create_dir_all(path).unwrap_or_else(|e| {
                eprintln!("error: could not create {}: {}", path.display(), e);
                process::exit(1);
            });
            println!("  created {}", path.display());
        }
    }
}

fn cmd_init(dir: PathBuf) {
    init_directories(&dir);
}

fn cmd_new(
    dir: PathBuf,
    title: Title,
    ticket_type: TicketType,
    status: TicketStatus,
    tags: Vec<Tag>,
    parent: Option<TicketId>,
    blocked_by: Vec<TicketId>,
) {
    let all_dir = dir.join("all");
    if !all_dir.exists() {
        eprintln!("error: tickets directory not initialised — run `tickets init` first");
        process::exit(1);
    }

    let id = TicketId(nanoid::nanoid!(6, &nanoid::alphabet::SAFE));
    let now = Utc::now();

    let front_matter = FrontMatter {
        id: id.clone(),
        title: title.clone(),
        r#type: ticket_type,
        status,
        tags,
        parent,
        blocked_by,
        created_at: now,
        updated_at: now,
    };

    let slug = title.slugify();
    let filename = format!("{}_{}.md", id, slug);
    let path = all_dir.join(&filename);

    let yaml = serde_yaml::to_string(&front_matter).unwrap_or_else(|e| {
        eprintln!("error: could not serialise front matter: {}", e);
        process::exit(1);
    });

    let content = format!("---\n{}---\n", yaml);

    std::fs::write(&path, &content).unwrap_or_else(|e| {
        eprintln!("error: could not write {}: {}", path.display(), e);
        process::exit(1);
    });

    println!("{} {}", id, filename);
}

fn main() {
    let cli = Cli::parse();
    let dir = resolve_dir();

    match cli.command {
        Commands::Init => cmd_init(dir),
        Commands::New {
            title,
            r#type,
            status,
            tag,
            parent,
            blocked_by,
        } => cmd_new(dir, title, r#type, status, tag, parent, blocked_by),
    }
}
