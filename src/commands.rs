use std::path::{Path, PathBuf};
use std::process;

use chrono::Utc;

use crate::types::{FrontMatter, Tag, TicketId, TicketStatus, TicketType, Title};

pub fn resolve_dir() -> PathBuf {
    std::env::var("TICKETS_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tickets"))
}

pub fn cmd_init(dir: PathBuf) {
    init_directories(&dir);
}

pub fn cmd_new(
    dir: PathBuf,
    title: Title,
    ticket_type: TicketType,
    status: TicketStatus,
    tags: Vec<Tag>,
    parent: Option<TicketId>,
    blocked_by: Vec<TicketId>,
) {
    let all_dir = dir.join("all");
    if !all_dir.exists() {
        eprintln!("error: tickets directory not initialised — run `tickets init` first");
        process::exit(1);
    }

    let id = TicketId::from(nanoid::nanoid!(6, &nanoid::alphabet::SAFE));
    let now = Utc::now();

    let front_matter = FrontMatter {
        id: id.clone(),
        title: title.clone(),
        r#type: ticket_type,
        status,
        tags,
        parent,
        blocked_by,
        created_at: now,
        updated_at: now,
    };

    let slug = title.slugify();
    let filename = format!("{}_{}.md", id, slug);
    let path = all_dir.join(&filename);

    let yaml = serde_yaml::to_string(&front_matter).unwrap_or_else(|e| {
        eprintln!("error: could not serialise front matter: {}", e);
        process::exit(1);
    });

    let content = format!("---\n{}---\n", yaml);

    std::fs::write(&path, &content).unwrap_or_else(|e| {
        eprintln!("error: could not write {}: {}", path.display(), e);
        process::exit(1);
    });

    println!("{} {}", id, filename);
}

fn init_directories(dir: &Path) {
    let all = dir.join("all");
    let archived = dir.join("archived");

    for path in [&all, &archived] {
        if path.exists() {
            println!("  exists  {}", path.display());
        } else {
            std::fs::create_dir_all(path).unwrap_or_else(|e| {
                eprintln!("error: could not create {}: {}", path.display(), e);
                process::exit(1);
            });
            println!("  created {}", path.display());
        }
    }
}
