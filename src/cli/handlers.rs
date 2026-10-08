use super::args::Command;
use super::presenter::TicketCliLinePresenter;
use crate::config::Config;
use crate::domain::id::ID;
use crate::domain::query::Filter;
use crate::domain::store;
use crate::domain::tickets::{NewTicket, Ticket};
use std::path::Path;

/// Runs every command except `init`, which needs no existing config.
pub fn run(command: Command, root: &Path, config: &Config) -> Result<(), String> {
    match command {
        Command::New {
            title,
            r#type,
            status,
            tags,
            parent,
            blocked_by,
            message,
        } => {
            let draft = NewTicket {
                title,
                r#type,
                status,
                tags,
                parent,
                blocked_by,
                body: message.unwrap_or_default(),
            };
            let id = new(root, config, draft)?;
            println!("{id}");
            Ok(())
        }
        Command::List {
            status,
            r#type,
            tag,
            parent,
            blocked,
            ready,
        } => {
            let filter = Filter {
                status,
                r#type,
                tags: tag,
                parent,
                blocked,
                ready,
            };
            for t in filter.select(store::list(root)?, config) {
                println!("{}", TicketCliLinePresenter(&t));
            }
            Ok(())
        }
        _ => Err("not implemented yet".to_string()),
    }
}

/// Creates a ticket under `root` and returns its ID.
fn new(root: &Path, config: &Config, draft: NewTicket) -> Result<ID, String> {
    for id in draft.parent.iter().chain(&draft.blocked_by) {
        if store::read(root, *id)?.is_none() {
            return Err(format!("no ticket with ID {id}"));
        }
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let ticket = Ticket::new(draft, config, ID::rand(), now)?;
    store::create(root, &ticket)?;
    Ok(ticket.id)
}
