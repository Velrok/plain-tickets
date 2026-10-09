use super::id::{ID, IdPrefix};
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

/// Whether an active or archived ticket already has this ID.
pub fn exists(root: &Path, id: ID) -> bool {
    path(root, id).exists() || archived_path(root, id).exists()
}

/// The active ticket with this ID, or `None` if there is no such file.
pub fn read(root: &Path, id: ID) -> Result<Option<Ticket>, String> {
    read_file(&path(root, id))
}

/// The ticket with this ID, active or archived. For read-only commands.
pub fn find(root: &Path, id: ID) -> Result<Ticket, String> {
    if let Some(ticket) = read(root, id)? {
        return Ok(ticket);
    }
    read_file(&archived_path(root, id))?.ok_or_else(|| not_found(root, id))
}

/// The IDs of all active and archived tickets, sorted, taken from the file names alone.
pub fn all_ids(root: &Path) -> Vec<ID> {
    let mut ids: Vec<ID> = ["tickets/all", "tickets/archived"]
        .iter()
        .filter_map(|dir| std::fs::read_dir(root.join(dir)).ok())
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? != "md" {
                return None;
            }
            path.file_stem()?.to_str()?.parse().ok()
        })
        .collect();
    ids.sort();
    ids
}

/// The ID of the active or archived ticket that `prefix` names. A full ID is returned as is, so
/// the usual "no ticket" / "is archived" errors come from the command using it.
pub fn resolve(root: &Path, prefix: &IdPrefix) -> Result<ID, String> {
    if let Some(id) = prefix.full() {
        return Ok(id);
    }
    let matches: Vec<ID> = all_ids(root)
        .into_iter()
        .filter(|id| prefix.matches(*id))
        .collect();
    match matches[..] {
        [] => Err(format!("no ticket with ID prefix '{prefix}'")),
        [id] => Ok(id),
        _ => {
            let lines: Vec<String> = matches
                .iter()
                .map(|id| match find(root, *id) {
                    Ok(t) => format!("  {id}  {}", t.title),
                    Err(_) => format!("  {id}"),
                })
                .collect();
            Err(format!(
                "ID prefix '{prefix}' matches {} tickets:\n{}",
                matches.len(),
                lines.join("\n")
            ))
        }
    }
}

fn read_file(path: &Path) -> Result<Option<Ticket>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => text
            .parse()
            .map(Some)
            .map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("cannot read {}: {e}", path.display())),
    }
}

fn archived_path(root: &Path, id: ID) -> PathBuf {
    root.join("tickets/archived").join(format!("{id}.md"))
}

/// The error for an ID that is not active: says so when the ticket is archived.
pub fn not_found(root: &Path, id: ID) -> String {
    if archived_path(root, id).is_file() {
        format!("ticket {id} is archived")
    } else {
        format!("no ticket with ID {id}")
    }
}

/// Like `read`, but a missing ticket is an error.
pub fn require(root: &Path, id: ID) -> Result<Ticket, String> {
    read(root, id)?.ok_or_else(|| not_found(root, id))
}

/// Moves an active ticket to `tickets/archived`. Never overwrites.
pub fn archive(root: &Path, id: ID) -> Result<(), String> {
    let from = path(root, id);
    let to = archived_path(root, id);
    let dir = to.parent().expect("archive path has a parent");
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    if to.exists() {
        return Err(format!("{} already exists", to.display()));
    }
    std::fs::rename(&from, &to).map_err(|e| format!("cannot move {}: {e}", from.display()))
}

/// Moves an archived ticket back to `tickets/all`. Never overwrites.
pub fn unarchive(root: &Path, id: ID) -> Result<(), String> {
    let from = archived_path(root, id);
    let to = path(root, id);
    if !from.is_file() {
        return Err(format!("no archived ticket with ID {id}"));
    }
    if to.exists() {
        return Err(format!("{} already exists", to.display()));
    }
    std::fs::rename(&from, &to).map_err(|e| format!("cannot move {}: {e}", from.display()))
}

/// Atomically overwrites an existing ticket file. Errors if the ticket does not exist.
pub fn replace(root: &Path, ticket: &Ticket) -> Result<(), String> {
    let path = path(root, ticket.id);
    if !path.is_file() {
        return Err(not_found(root, ticket.id));
    }
    // Write next to the target and rename over it, so a crash never leaves a half-written ticket.
    // The temp name does not end in `.md`, so listing ignores a leftover one.
    let tmp = path.with_extension("tmp");
    let written = std::fs::write(&tmp, ticket.to_string())
        .and_then(|()| std::fs::File::open(&tmp)?.sync_all())
        .and_then(|()| std::fs::rename(&tmp, &path));
    written.map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("cannot write {}: {e}", path.display())
    })
}

/// Every active ticket. A missing `tickets/all` directory means no tickets.
pub fn list(root: &Path) -> Result<Vec<Ticket>, String> {
    read_dir(&root.join("tickets/all"))
}

/// Every archived ticket.
pub fn list_archived(root: &Path) -> Result<Vec<Ticket>, String> {
    read_dir(&root.join("tickets/archived"))
}

fn read_dir(dir: &Path) -> Result<Vec<Ticket>, String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("cannot read {}: {e}", dir.display())),
    };
    let mut tickets = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
            .path();
        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let parsed = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))
            .and_then(|text| {
                text.parse::<Ticket>()
                    .map_err(|e| format!("{}: {e}", path.display()))
            });
        match parsed {
            Ok(ticket) => tickets.push(ticket),
            Err(e) => eprintln!("warning: skipping {e}"),
        }
    }
    Ok(tickets)
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
        Ticket::new(
            draft,
            &Config::default(),
            ID(id),
            "1970-01-01T00:00:01Z".parse().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn created_ticket_is_stored_under_tickets_all_and_reads_back() {
        let root = scratch("create");
        let path = create(&root, &ticket(0xab)).unwrap();
        assert_eq!(path, root.join("tickets/all/000000000004r.md"));
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

    #[test]
    fn replace_leaves_only_the_ticket_file_behind() {
        let root = scratch("replace-clean");
        create(&root, &ticket(1)).unwrap();
        let mut changed = ticket(1);
        changed.title = "Changed".into();
        replace(&root, &changed).unwrap();
        let files: Vec<_> = std::fs::read_dir(root.join("tickets/all"))
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(files, [format!("{}.md", ID(1))]);
        assert_eq!(
            read(&root, ID(1)).unwrap().unwrap().to_string(),
            changed.to_string()
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_replace_leaves_the_old_file_intact() {
        use std::os::unix::fs::PermissionsExt;
        let root = scratch("replace-fails");
        create(&root, &ticket(1)).unwrap();
        let before = ticket(1).to_string();
        let dir = root.join("tickets/all");
        // A read-only directory stops a temp file being created next to the ticket.
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();
        let mut changed = ticket(1);
        changed.title = "Changed".into();
        let result = replace(&root, &changed);
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.is_err());
        assert_eq!(read(&root, ID(1)).unwrap().unwrap().to_string(), before);
    }
}
