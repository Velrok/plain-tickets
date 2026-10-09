use super::args::{Command, Format};
use super::output;
use super::presenter::{
    PlainDetailPresenter, PlainLinePresenter, PrettyDetailPresenter, PrettyListPresenter, Relation,
};
use crate::config::Config;
use crate::domain::id::ID;
use crate::domain::query::Filter;
use crate::domain::store;
use crate::domain::tickets::{Change, NewTicket, SetFields, Ticket};
use crate::domain::timestamp::Timestamp;
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
            let id = new(root, config, draft, ID::generate)?;
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
            archived,
            format,
        } => {
            let mode = output::current(format);
            let filter = Filter {
                status,
                r#type,
                tags: tag,
                parent,
                blocked,
                ready,
            };
            let found = if archived {
                store::list_archived(root)?
            } else {
                store::list(root)?
            };
            let selected = filter.select(found, config);
            match mode.format {
                Format::Plain => {
                    for t in &selected {
                        println!("{}", PlainLinePresenter(t));
                    }
                }
                Format::Pretty => {
                    let table = PrettyListPresenter {
                        tickets: &selected,
                        colour: mode.colour,
                        width: terminal_size::terminal_size().map(|(w, _)| w.0 as usize),
                    }
                    .to_string();
                    if !table.is_empty() {
                        println!("{table}");
                    }
                }
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
        Command::Show { id, format } => {
            let mode = output::current(format);
            let ticket = store::find(root, id)?;
            match mode.format {
                Format::Plain => println!("{}", PlainDetailPresenter(&ticket)),
                Format::Pretty => {
                    // A relation whose file cannot be read is shown as not found.
                    let relation = |id: ID| Relation {
                        id,
                        ticket: store::find(root, id).ok(),
                    };
                    println!(
                        "{}",
                        PrettyDetailPresenter {
                            parent: ticket.parent.map(relation),
                            blockers: ticket.blocked_by.iter().copied().map(relation).collect(),
                            ticket: &ticket,
                            colour: mode.colour,
                        }
                    );
                }
            }
            Ok(())
        }
        Command::Archive { id } => archive(root, id),
        Command::Unarchive { id } => store::unarchive(root, id),
        Command::Note { id, text } => modify(root, config, id, Change::AppendNote(text)),
        _ => Err("not implemented yet".to_string()),
    }
}

fn ensure_exist<'a>(root: &Path, ids: impl Iterator<Item = &'a ID>) -> Result<(), String> {
    for id in ids {
        store::require(root, *id)?;
    }
    Ok(())
}

/// Applies `change` to the ticket and saves it, if it changed anything.
fn modify(root: &Path, config: &Config, id: ID, change: Change) -> Result<(), String> {
    let ticket = store::require(root, id)?;
    save_revision(root, config, &ticket, ticket.apply(change))
}

/// Validates `edited` against `ticket` and writes it if it differs.
fn save_revision(
    root: &Path,
    config: &Config,
    ticket: &Ticket,
    edited: Ticket,
) -> Result<(), String> {
    match ticket.revise(edited, config, Timestamp::now())? {
        Some(revised) => {
            ensure_exist(root, revised.parent.iter().chain(&revised.blocked_by))?;
            store::replace(root, &revised)
        }
        None => Ok(()),
    }
}

/// Archives a done or rejected ticket.
fn archive(root: &Path, id: ID) -> Result<(), String> {
    let ticket = store::require(root, id)?;
    if !ticket.status.is_terminal() {
        return Err(format!(
            "cannot archive {id}: status '{}' is not done or rejected",
            ticket.status
        ));
    }
    store::archive(root, id)
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
    let ticket = store::require(root, id)?;
    let edited: Ticket = edit_text(id, &ticket.to_string())?.parse()?;
    save_revision(root, config, &ticket, edited)
}

/// How many IDs to draw before giving up on finding a free one.
const ID_ATTEMPTS: usize = 5;

/// Creates a ticket under `root` with the first free ID from `next_id`.
fn new(
    root: &Path,
    config: &Config,
    draft: NewTicket,
    mut next_id: impl FnMut() -> ID,
) -> Result<ID, String> {
    ensure_exist(root, draft.parent.iter().chain(&draft.blocked_by))?;
    let id = (0..ID_ATTEMPTS)
        .map(|_| next_id())
        .find(|id| !store::exists(root, *id))
        .ok_or("could not find a unique ticket ID")?;
    let ticket = Ticket::new(draft, config, id, Timestamp::now())?;
    store::create(root, &ticket)?;
    Ok(ticket.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> NewTicket {
        NewTicket {
            title: "Fix it".into(),
            r#type: None,
            status: None,
            tags: vec![],
            parent: None,
            blocked_by: vec![],
            body: String::new(),
        }
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tickets-new-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn new_draws_another_id_when_one_is_taken_by_an_active_or_archived_ticket() {
        let root = scratch("collision");
        let config = Config::default();
        new(&root, &config, draft(), || ID(1)).unwrap();
        new(&root, &config, draft(), || ID(2)).unwrap();
        store::archive(&root, ID(2)).unwrap();
        let mut ids = [1, 2, 3].map(ID).into_iter();
        let id = new(&root, &config, draft(), || ids.next().unwrap()).unwrap();
        assert_eq!(id, ID(3));
    }

    #[test]
    fn new_gives_up_after_a_few_taken_ids() {
        let root = scratch("giveup");
        let config = Config::default();
        new(&root, &config, draft(), || ID(7)).unwrap();
        let err = new(&root, &config, draft(), || ID(7)).unwrap_err();
        assert!(err.contains("unique"), "{err}");
    }
}
