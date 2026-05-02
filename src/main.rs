use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod git;
mod types;

use commands::{cmd_archive, cmd_edit, cmd_init, cmd_list, cmd_new, cmd_show, resolve_dir};
use types::{Tag, TicketId, TicketStatus, TicketType, Title};

#[derive(Parser)]
#[command(name = "tickets", about = "Plain markdown ticket system")]
struct Cli {
    /// Override the tickets directory (takes precedence over TICKETS_DIR)
    #[arg(long, global = true)]
    dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Archive tickets by ID or status
    Archive {
        /// Ticket IDs to archive
        ids: Vec<TicketId>,
        /// Archive all tickets with status=rejected
        #[arg(long)]
        all_rejected: bool,
    },
    /// Initialise tickets directory
    Init,
    /// List all active tickets
    List,
    /// Show a single ticket
    Show {
        /// Ticket ID
        id: TicketId,
    },
    /// Edit an existing ticket
    Edit {
        /// Ticket id
        id: TicketId,
        /// New title
        #[arg(long)]
        title: Option<Title>,
        /// New type
        #[arg(long)]
        r#type: Option<TicketType>,
        /// New status
        #[arg(long)]
        status: Option<TicketStatus>,
        /// Replace tags (repeatable)
        #[arg(long)]
        tag: Vec<Tag>,
        /// Set parent ticket id
        #[arg(long)]
        parent: Option<TicketId>,
        /// Clear parent
        #[arg(long)]
        clear_parent: bool,
        /// Set blocked-by ticket ids (repeatable)
        #[arg(long)]
        blocked_by: Vec<TicketId>,
        /// Clear blocked-by list
        #[arg(long)]
        clear_blocked_by: bool,
        /// Body text; use `-` to read from STDIN
        #[arg(long)]
        body: Option<String>,
    },
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
        /// Body text; use `-` to read from STDIN
        #[arg(long)]
        body: Option<String>,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let base_dir = resolve_dir(cli.dir);

    if let Commands::Init = cli.command {
        return cmd_init(base_dir);
    }

    let working_dir = commands::WorkingDir::new(base_dir)?;
    let cfg = config::load(working_dir.as_ref())?;

    match cli.command {
        Commands::Init => unreachable!(),
        Commands::Archive { ids, all_rejected } => {
            cmd_archive(working_dir, &cfg, ids, all_rejected)
        }
        Commands::List => cmd_list(working_dir, &cfg),
        Commands::Edit {
            id,
            title,
            r#type,
            status,
            tag,
            parent,
            clear_parent,
            blocked_by,
            clear_blocked_by,
            body,
        } => cmd_edit(
            working_dir,
            &cfg,
            id,
            title,
            r#type,
            status,
            tag,
            parent,
            blocked_by,
            body,
            clear_parent,
            clear_blocked_by,
        ),
        Commands::Show { id } => cmd_show(working_dir, id),
        Commands::New {
            title,
            r#type,
            status,
            tag,
            parent,
            blocked_by,
            body,
        } => cmd_new(
            working_dir,
            &cfg,
            title,
            r#type,
            status,
            tag,
            parent,
            blocked_by,
            body,
        ),
    }
}
