use super::id::ID;
use super::tickets::Ticket;
use std::path::{Path, PathBuf};

fn path(root: &Path, id: ID) -> PathBuf {
    root.join("tickets/all").join(format!("{id}.md"))
}

/// Writes the ticket to `root/tickets/all/<id>.md`. Never overwrites.
pub fn create(root: &Path, ticket: &Ticket) -> Result<PathBuf, String> {
    use std::io::Write;

    let path = path(root, ticket.id);
    let dir = path.parent().expect("ticket path has a parent");
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => format!("{} already exists", path.display()),
            _ => format!("cannot write {}: {e}", path.display()),
        })?;
    file.write_all(ticket.to_string().as_bytes())
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// The active ticket with this ID, or `None` if there is no such file.
pub fn read(root: &Path, id: ID) -> Result<Option<Ticket>, String> {
    let path = path(root, id);
    match std::fs::read_to_string(&path) {
        Ok(text) => text
            .parse()
            .map(Some)
            .map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("cannot read {}: {e}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::domain::id::ID;
    use crate::domain::tickets::NewTicket;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tickets-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn ticket(id: u64) -> Ticket {
        let draft = NewTicket {
            title: "Fix it".into(),
            r#type: None,
            status: None,
            tags: vec![],
            parent: None,
            blocked_by: vec![],
            body: "Body\n".into(),
        };
        Ticket::new(draft, &Config::default(), ID(id), 1).unwrap()
    }

    #[test]
    fn created_ticket_is_stored_under_tickets_all_and_reads_back() {
        let root = scratch("create");
        let path = create(&root, &ticket(0xab)).unwrap();
        assert_eq!(path, root.join("tickets/all/00000000000000ab.md"));
        assert_eq!(
            read(&root, ID(0xab)).unwrap().unwrap().to_string(),
            ticket(0xab).to_string()
        );
    }

    #[test]
    fn create_refuses_to_overwrite_an_existing_ticket() {
        let root = scratch("overwrite");
        create(&root, &ticket(1)).unwrap();
        let err = create(&root, &ticket(1)).unwrap_err();
        assert!(err.contains("already exists"), "{err}");
    }

    #[test]
    fn read_returns_none_for_unknown_ids() {
        let root = scratch("unknown");
        assert!(read(&root, ID(9)).unwrap().is_none());
    }
}
