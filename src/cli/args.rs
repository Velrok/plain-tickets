use crate::domain::id::ID;
use crate::domain::status::Status;
use clap::{Parser, Subcommand};

/// Markdown tickets with YAML front matter.
#[derive(Parser)]
#[command(name = "tickets", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create .tickets/config.toml with the default statuses
    Init,
    /// Create a ticket and print its ID
    New {
        title: String,
        #[arg(short = 't', long)]
        r#type: Option<String>,
        /// Defaults to the first configured status
        #[arg(short, long)]
        status: Option<Status>,
        #[arg(short = 'g', long = "tag")]
        tags: Vec<String>,
        #[arg(short, long)]
        parent: Option<ID>,
        #[arg(short, long = "blocked-by")]
        blocked_by: Vec<ID>,
        /// Body text; opens $EDITOR if omitted on a terminal
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Print a ticket's front matter and body
    Show { id: String },
    /// List tickets
    List {
        #[arg(short, long)]
        /// Repeatable; matches any
        status: Vec<Status>,
        #[arg(short = 't', long)]
        r#type: Option<String>,
        #[arg(short = 'g', long)]
        /// Repeatable; matches any
        tag: Vec<String>,
        #[arg(short, long)]
        parent: Option<ID>,
        /// Only tickets with unfinished blockers
        #[arg(long, conflicts_with = "ready")]
        blocked: bool,
        /// Only open tickets whose blockers are all done
        #[arg(long)]
        ready: bool,
    },
    /// Open a ticket in $EDITOR
    Edit { id: ID },
    /// Set scalar fields
    Set {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        parent: Option<String>,
    },
    /// Add tags
    Tag {
        id: String,
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// Remove tags
    Untag {
        id: String,
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// Add blockers
    Block {
        id: String,
        #[arg(required = true)]
        blockers: Vec<String>,
    },
    /// Remove blockers
    Unblock {
        id: String,
        #[arg(required = true)]
        blockers: Vec<String>,
    },
    /// Append text to the body
    Note { id: String, text: String },
    /// Archive a ticket (reversible)
    Archive { id: String },
    /// Restore an archived ticket
    Unarchive { id: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn definition_is_valid() {
        Cli::command().debug_assert();
    }
}
