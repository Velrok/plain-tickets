use super::args::Command;
use super::presenter::TicketCliLinePresenter;
use crate::config::Config;
use crate::domain::id::ID;
use crate::domain::query::Filter;
use crate::domain::store;
use crate::domain::tickets::{Change, NewTicket, SetFields, Ticket};
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
        Command::Edit { id } => edit(root, config, id),
        Command::Tag { id, tags } => modify(root, config, id, Change::AddTags(tags)),
        Command::Untag { id, tags } => modify(root, config, id, Change::RemoveTags(tags)),
        Command::Block { id, blockers } => modify(root, config, id, Change::AddBlockers(blockers)),
        Command::Unblock { id, blockers } => {
            modify(root, config, id, Change::RemoveBlockers(blockers))
        }
        Command::Set {
            id,
            title,
            status,
            r#type,
            clear_type,
            parent,
            clear_parent,
        } => {
            let fields = SetFields {
                title,
                status,
                r#type: if clear_type {
                    Some(None)
                } else {
                    r#type.map(Some)
                },
                parent: if clear_parent {
                    Some(None)
                } else {
                    parent.map(Some)
                },
            };
            if fields.is_empty() {
                return Err("nothing to set; pass at least one field".to_string());
            }
            modify(root, config, id, Change::Set(fields))
        }
        Command::Note { id, text } => modify(root, config, id, Change::AppendNote(text)),
        _ => Err("not implemented yet".to_string()),
    }
}

fn ensure_exist<'a>(root: &Path, ids: impl Iterator<Item = &'a ID>) -> Result<(), String> {
    for id in ids {
        if store::read(root, *id)?.is_none() {
            return Err(format!("no ticket with ID {id}"));
        }
    }
    Ok(())
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Applies `change` to the ticket and saves it, if it changed anything.
fn modify(root: &Path, config: &Config, id: ID, change: Change) -> Result<(), String> {
    let ticket = store::read(root, id)?.ok_or_else(|| format!("no ticket with ID {id}"))?;
    save_revision(root, config, &ticket, ticket.apply(change))
}

/// Validates `edited` against `ticket` and writes it if it differs.
fn save_revision(
    root: &Path,
    config: &Config,
    ticket: &Ticket,
    edited: Ticket,
) -> Result<(), String> {
    match ticket.revise(edited, config, now())? {
        Some(revised) => {
            ensure_exist(root, revised.parent.iter().chain(&revised.blocked_by))?;
            store::replace(root, &revised)
        }
        None => Ok(()),
    }
}

/// Runs `$VISUAL` (or `$EDITOR`) on a temporary file holding `text` and returns
/// what the editor left in it.
fn edit_text(id: ID, text: &str) -> Result<String, String> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .map_err(|_| "set $VISUAL or $EDITOR to choose an editor".to_string())?;
    let tmp = std::env::temp_dir().join(format!("tickets-{id}-{}.md", std::process::id()));
    std::fs::write(&tmp, text).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$0\""))
        .arg(&tmp)
        .status()
        .map_err(|e| format!("cannot run editor: {e}"))?;
    let edited = std::fs::read_to_string(&tmp);
    let _ = std::fs::remove_file(&tmp);
    if !status.success() {
        return Err(format!("editor exited with {status}"));
    }
    edited.map_err(|e| format!("cannot read edited file: {e}"))
}

/// Opens the ticket in `$EDITOR` and saves the result.
fn edit(root: &Path, config: &Config, id: ID) -> Result<(), String> {
    let ticket = store::read(root, id)?.ok_or_else(|| format!("no ticket with ID {id}"))?;
    let edited: Ticket = edit_text(id, &ticket.to_string())?.parse()?;
    save_revision(root, config, &ticket, edited)
}

/// Creates a ticket under `root` and returns its ID.
fn new(root: &Path, config: &Config, draft: NewTicket) -> Result<ID, String> {
    ensure_exist(root, draft.parent.iter().chain(&draft.blocked_by))?;
    let ticket = Ticket::new(draft, config, ID::rand(), now())?;
    store::create(root, &ticket)?;
    Ok(ticket.id)
}
