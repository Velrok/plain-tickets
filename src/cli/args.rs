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
        /// List archived tickets instead of active ones
        #[arg(long)]
        archived: bool,
    },
    /// Open a ticket in $EDITOR
    Edit { id: ID },
    /// Set scalar fields
    Set {
        id: ID,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<Status>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long, conflicts_with = "type")]
        clear_type: bool,
        #[arg(long)]
        parent: Option<ID>,
        #[arg(long, conflicts_with = "parent")]
        clear_parent: bool,
    },
    /// Add tags
    Tag {
        id: ID,
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// Remove tags
    Untag {
        id: ID,
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// Add blockers
    Block {
        id: ID,
        #[arg(required = true)]
        blockers: Vec<ID>,
    },
    /// Remove blockers
    Unblock {
        id: ID,
        #[arg(required = true)]
        blockers: Vec<ID>,
    },
    /// Append text to the body
    Note { id: ID, text: String },
    /// Archive a ticket (reversible)
    Archive { id: ID },
    /// Restore an archived ticket
    Unarchive { id: ID },
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
