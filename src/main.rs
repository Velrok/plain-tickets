use clap::{Parser, Subcommand};

mod commands;
mod types;

use commands::{cmd_init, cmd_new, resolve_dir};
use types::{Tag, TicketId, TicketStatus, TicketType, Title};

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
        /// Body text; use `-` to read from STDIN
        #[arg(long)]
        body: Option<String>,
    },
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
            body,
        } => cmd_new(dir, title, r#type, status, tag, parent, blocked_by, body),
    }
}
