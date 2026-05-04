use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use clap::Args;

use crate::domain_types::{Tag, TicketId, TicketStatus, TicketType, Title};

#[derive(Args)]
pub struct ListArgs {
    /// Filter by status (repeatable, OR semantics)
    #[arg(long)]
    pub status: Vec<TicketStatus>,
    /// Filter by type (repeatable, OR semantics)
    #[arg(long)]
    pub r#type: Vec<TicketType>,
    /// Filter by tag (repeatable, AND semantics)
    #[arg(long)]
    pub tag: Vec<Tag>,
}

pub struct WorkingDir(PathBuf);

impl WorkingDir {
    pub fn new(base: PathBuf) -> Result<Self> {
        let wd = WorkingDir(base);
        if !wd.all().exists() || !wd.archived().exists() {
            bail!("tickets directory not initialised — run `tickets init` first");
        }
        Ok(wd)
    }

    pub fn all(&self) -> PathBuf { self.0.join("all") }
    pub fn archived(&self) -> PathBuf { self.0.join("archived") }
}

impl AsRef<Path> for WorkingDir {
    fn as_ref(&self) -> &Path { &self.0 }
}

#[derive(Args)]
pub struct ArchiveArgs {
    /// Ticket IDs to archive
    pub ids: Vec<TicketId>,
    /// Archive all tickets with status=rejected
    #[arg(long)]
    pub all_rejected: bool,
}

#[derive(Args)]
pub struct EditArgs {
    /// Ticket id
    pub id: TicketId,
    /// New title
    #[arg(long)]
    pub title: Option<Title>,
    /// New type
    #[arg(long)]
    pub r#type: Option<TicketType>,
    /// New status
    #[arg(long)]
    pub status: Option<TicketStatus>,
    /// Replace tags (repeatable)
    #[arg(long)]
    pub tag: Vec<Tag>,
    /// Set parent ticket id
    #[arg(long)]
    pub parent: Option<TicketId>,
    /// Clear parent
    #[arg(long)]
    pub clear_parent: bool,
    /// Set blocked-by ticket ids (repeatable)
    #[arg(long)]
    pub blocked_by: Vec<TicketId>,
    /// Clear blocked-by list
    #[arg(long)]
    pub clear_blocked_by: bool,
    /// Body text; use `-` to read from STDIN
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Args)]
pub struct NewArgs {
    /// Ticket title (required, max 120 chars)
    #[arg(long)]
    pub title: Title,
    /// Ticket type
    #[arg(long, default_value = "task")]
    pub r#type: TicketType,
    /// Tags (repeatable; letters, numbers, _ and - only)
    #[arg(long)]
    pub tag: Vec<Tag>,
    /// Parent ticket id
    #[arg(long)]
    pub parent: Option<TicketId>,
    /// Blocked-by ticket ids (repeatable)
    #[arg(long)]
    pub blocked_by: Vec<TicketId>,
    /// Initial status (default: draft)
    #[arg(long, default_value = "draft")]
    pub status: TicketStatus,
    /// Body text; use `-` to read from STDIN
    #[arg(long)]
    pub body: Option<String>,
}
