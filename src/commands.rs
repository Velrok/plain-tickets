use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process;

use chrono::{DateTime, Utc};

use crate::config::Config;
use crate::git;
use crate::types::{FrontMatter, Tag, Ticket, TicketId, TicketStatus, TicketType, Title};

pub fn resolve_dir(flag: Option<PathBuf>) -> PathBuf {
    flag.or_else(|| std::env::var("TICKETS_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("tickets"))
}

pub fn cmd_init(dir: PathBuf) {
    let config_path = dir.join(".tickets.toml");
    if config_path.exists() {
        eprintln!("error: already initialised — .tickets.toml already exists");
        process::exit(1);
    }
    init_directories(&dir);
    let config_content = "\
# plain-tickets configuration
# Uncomment and set values to override defaults.

# [git]
# auto_commit = false
";
    std::fs::write(&config_path, config_content).unwrap_or_else(|e| {
        eprintln!("error: could not create .tickets.toml: {e}");
        process::exit(1);
    });
    println!("  created {}", config_path.display());

    if git_detect(&dir).is_ok() {
        println!("hint: git repository detected — set auto_commit = true in .tickets.toml to commit on every change");
    }
}

/// Returns `Ok(())` if a `.git` directory is found at or above `dir`.
fn git_detect(dir: &Path) -> Result<(), ()> {
    let mut current = dir;
    loop {
        if current.join(".git").exists() {
            return Ok(());
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return Err(()),
        }
    }
}

pub fn cmd_new(
    dir: PathBuf,
    cfg: &Config,
    title: Title,
    ticket_type: TicketType,
    status: TicketStatus,
    tags: Vec<Tag>,
    parent: Option<TicketId>,
    blocked_by: Vec<TicketId>,
    body: Option<String>,
) {
    let all_dir = dir.join("all");
    if !all_dir.exists() {
        eprintln!("error: tickets directory not initialised — run `tickets init` first");
        process::exit(1);
    }

    const ALPHA: [char; 36] = [
        '0','1','2','3','4','5','6','7','8','9',
        'a','b','c','d','e','f','g','h','i','j','k','l','m',
        'n','o','p','q','r','s','t','u','v','w','x','y','z',
    ];
    let id = TicketId::from(nanoid::nanoid!(6, &ALPHA));
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

    let body = match body.as_deref() {
        None => String::new(),
        Some("-") => read_body_from_stdin(),
        Some(text) => text.to_string(),
    };
    let ticket = Ticket { front_matter, body };

    std::fs::write(&path, ticket.to_string()).unwrap_or_else(|e| {
        eprintln!("error: could not write {}: {}", path.display(), e);
        process::exit(1);
    });

    if cfg.git.auto_commit {
        let message = format!("tickets: new {} \"{}\"", id, title);
        if let Err(e) = git::git_commit(&dir, &path, &message) {
            eprintln!("{e}");
            process::exit(1);
        }
    }

    println!("{} {}", id, filename);
}

pub fn cmd_show(dir: PathBuf, id: TicketId) {
    let all_dir = dir.join("all");
    if !all_dir.exists() {
        eprintln!("error: tickets directory not initialised — run `tickets init` first");
        process::exit(1);
    }
    let path = find_ticket(&all_dir, &id);
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error: could not read {}: {}", path.display(), e);
        process::exit(1);
    });
    let ticket: Ticket = raw.parse().unwrap_or_else(|e| {
        eprintln!("error: could not parse {}: {}", path.display(), e);
        process::exit(1);
    });
    print_ticket(&ticket);
}

fn relative_time(dt: DateTime<Utc>) -> String {
    let secs = (Utc::now() - dt).num_seconds().max(0);
    if secs < 60 { return "just now".to_string(); }
    let mins = secs / 60;
    if mins < 60 { return format!("{mins} minute{} ago", if mins == 1 { "" } else { "s" }); }
    let hours = mins / 60;
    if hours < 24 { return format!("{hours} hour{} ago", if hours == 1 { "" } else { "s" }); }
    let days = hours / 24;
    if days < 30 { return format!("{days} day{} ago", if days == 1 { "" } else { "s" }); }
    let months = days / 30;
    if months < 12 { return format!("{months} month{} ago", if months == 1 { "" } else { "s" }); }
    let years = months / 12;
    format!("{years} year{} ago", if years == 1 { "" } else { "s" })
}

fn fmt_timestamp(dt: DateTime<Utc>) -> String {
    format!("{} · {}", dt.format("%Y-%m-%d"), relative_time(dt))
}

fn print_ticket(ticket: &Ticket) {
    let fm = &ticket.front_matter;
    println!("🎫  {}", fm.title);
    println!("📌  {}", fm.status);
    println!("🏷   {}", fm.r#type);
    if !fm.tags.is_empty() {
        let tags: Vec<String> = fm.tags.iter().map(|t| t.to_string()).collect();
        println!("🔖  {}", tags.join(", "));
    }
    if let Some(ref p) = fm.parent {
        println!("⬆️   {}", p);
    }
    if !fm.blocked_by.is_empty() {
        let ids: Vec<String> = fm.blocked_by.iter().map(|t| t.to_string()).collect();
        println!("🚫  {}", ids.join(", "));
    }
    println!("📅  created   {}", fmt_timestamp(fm.created_at));
    println!("✏️   updated   {}", fmt_timestamp(fm.updated_at));
    if !ticket.body.is_empty() {
        println!();
        print_body(&ticket.body);
    }
}

fn print_body(body: &str) {
    let bat_ok = std::process::Command::new("bat")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if bat_ok {
        let mut child = std::process::Command::new("bat")
            .args(["--language=md", "--style=plain", "--color=always", "-"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| {
                eprintln!("error: could not spawn bat: {e}");
                process::exit(1);
            });
        use std::io::Write as _;
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(body.as_bytes());
        }
        let _ = child.wait();
    } else {
        print!("{}", body);
    }
}

pub fn cmd_list(dir: PathBuf, _cfg: &Config) {
    let all_dir = dir.join("all");
    if !all_dir.exists() {
        eprintln!("error: tickets directory not initialised — run `tickets init` first");
        process::exit(1);
    }

    let mut tickets: Vec<Ticket> = std::fs::read_dir(&all_dir)
        .unwrap_or_else(|e| {
            eprintln!("error: could not read directory {}: {}", all_dir.display(), e);
            process::exit(1);
        })
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|raw| raw.parse::<Ticket>().ok())
        .collect();

    tickets.sort_by(|a, b| {
        let status_order = |s: &TicketStatus| match s {
            TicketStatus::InProgress => 0,
            TicketStatus::Todo => 1,
            TicketStatus::Draft => 2,
            TicketStatus::Done => 3,
            TicketStatus::Rejected => 4,
        };
        status_order(&a.front_matter.status)
            .cmp(&status_order(&b.front_matter.status))
            .then(a.front_matter.created_at.cmp(&b.front_matter.created_at))
    });

    let id_w = tickets.iter().map(|t| t.front_matter.id.to_string().len()).max().unwrap_or(6).max(6);
    let status_w = tickets.iter().map(|t| t.front_matter.status.to_string().len()).max().unwrap_or(6).max(6);
    let type_w = tickets.iter().map(|t| t.front_matter.r#type.to_string().len()).max().unwrap_or(4).max(4);

    for ticket in &tickets {
        let fm = &ticket.front_matter;
        println!(
            "{:<id_w$}  {:<status_w$}  {:<type_w$}  {}",
            fm.id, fm.status, fm.r#type, fm.title,
            id_w = id_w, status_w = status_w, type_w = type_w,
        );
    }
}

pub fn cmd_edit(
    dir: PathBuf,
    cfg: &Config,
    id: TicketId,
    title: Option<Title>,
    ticket_type: Option<TicketType>,
    status: Option<TicketStatus>,
    tags: Vec<Tag>,
    parent: Option<TicketId>,
    blocked_by: Vec<TicketId>,
    body: Option<String>,
    clear_parent: bool,
    clear_blocked_by: bool,
) {
    let all_dir = dir.join("all");
    let path = find_ticket(&all_dir, &id);

    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error: could not read {}: {}", path.display(), e);
        process::exit(1);
    });

    let mut ticket: Ticket = raw.parse().unwrap_or_else(|e| {
        eprintln!("error: could not parse {}: {}", path.display(), e);
        process::exit(1);
    });

    let fm = &mut ticket.front_matter;
    if let Some(t) = title { fm.title = t; }
    if let Some(t) = ticket_type { fm.r#type = t; }
    if let Some(s) = status { fm.status = s; }
    if !tags.is_empty() { fm.tags = tags; }
    if clear_parent { fm.parent = None; } else if parent.is_some() { fm.parent = parent; }
    if clear_blocked_by { fm.blocked_by = vec![]; } else if !blocked_by.is_empty() { fm.blocked_by = blocked_by; }
    fm.updated_at = Utc::now();

    if let Some(b) = body {
        ticket.body = match b.as_str() {
            "-" => read_body_from_stdin(),
            _ => b,
        };
    }

    std::fs::write(&path, ticket.to_string()).unwrap_or_else(|e| {
        eprintln!("error: could not write {}: {}", path.display(), e);
        process::exit(1);
    });

    if cfg.git.auto_commit {
        let message = format!("tickets: edit {} \"{}\"", ticket.front_matter.id, ticket.front_matter.title);
        if let Err(e) = git::git_commit(&dir, &path, &message) {
            eprintln!("{e}");
            process::exit(1);
        }
    }

    println!("updated {}", path.file_name().unwrap().to_string_lossy());
}

fn find_ticket(dir: &Path, id: &TicketId) -> PathBuf {
    let prefix = format!("{}_", id);
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| {
        eprintln!("error: could not read directory {}: {}", dir.display(), e);
        process::exit(1);
    });
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&prefix) {
            return entry.path();
        }
    }
    eprintln!("error: no ticket found with id {}", id);
    process::exit(1);
}

pub fn cmd_archive(dir: PathBuf, cfg: &Config, ids: Vec<TicketId>, all_rejected: bool) {
    let all_dir = dir.join("all");
    let archived_dir = dir.join("archived");

    if !all_dir.exists() || !archived_dir.exists() {
        eprintln!("error: tickets directory not initialised — run `tickets init` first");
        process::exit(1);
    }

    if all_rejected && !ids.is_empty() {
        eprintln!("error: --all-rejected and explicit IDs are mutually exclusive");
        process::exit(1);
    }

    if all_rejected {
        archive_all_rejected(&dir, &all_dir, &archived_dir, cfg);
    } else {
        archive_by_ids(&dir, &all_dir, &archived_dir, &ids, cfg);
    }
}

fn archive_by_ids(dir: &Path, all_dir: &Path, archived_dir: &Path, ids: &[TicketId], cfg: &Config) {
    // Validate all IDs upfront before moving anything
    let mut errors: Vec<String> = Vec::new();
    let mut paths: Vec<(PathBuf, PathBuf)> = Vec::new(); // (src, dst)

    for id in ids {
        let prefix = format!("{}_", id);
        let in_all = find_by_prefix(all_dir, &prefix);
        let in_archived = find_by_prefix(archived_dir, &prefix);

        match (in_all, in_archived) {
            (Some(src), _) => {
                let dst = archived_dir.join(src.file_name().unwrap());
                paths.push((src, dst));
            }
            (None, Some(_)) => errors.push(format!("{id}: already in archived/")),
            (None, None) => errors.push(format!("{id}: not found")),
        }
    }

    if !errors.is_empty() {
        for e in &errors {
            eprintln!("error: {e}");
        }
        eprintln!("no files moved");
        process::exit(1);
    }

    for (src, dst) in &paths {
        std::fs::rename(src, dst).unwrap_or_else(|e| {
            eprintln!("error: could not move {}: {}", src.display(), e);
            process::exit(1);
        });
        let id = dst.file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.split('_').next())
            .unwrap_or("?");
        println!("{}  archived → {}", id, dst.display());
        if cfg.git.auto_commit {
            let message = format!("tickets: archive {id}");
            if let Err(e) = git::git_commit(dir, dst, &message) {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    }
}

fn archive_all_rejected(dir: &Path, all_dir: &Path, archived_dir: &Path, cfg: &Config) {
    let tickets: Vec<(Ticket, PathBuf)> = std::fs::read_dir(all_dir)
        .unwrap_or_else(|e| {
            eprintln!("error: could not read {}: {}", all_dir.display(), e);
            process::exit(1);
        })
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .filter_map(|e| {
            let path = e.path();
            std::fs::read_to_string(&path).ok()
                .and_then(|raw| raw.parse::<Ticket>().ok())
                .map(|t| (t, path))
        })
        .filter(|(t, _)| t.front_matter.status == TicketStatus::Rejected)
        .collect();

    if tickets.is_empty() {
        eprintln!("nothing to archive");
        return;
    }

    for (_, src) in &tickets {
        let dst = archived_dir.join(src.file_name().unwrap());
        std::fs::rename(src, &dst).unwrap_or_else(|e| {
            eprintln!("error: could not move {}: {}", src.display(), e);
            process::exit(1);
        });
        let id = dst.file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.split('_').next())
            .unwrap_or("?");
        println!("{}  archived → {}", id, dst.display());
        if cfg.git.auto_commit {
            let message = format!("tickets: archive {id}");
            if let Err(e) = git::git_commit(dir, &dst, &message) {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    }
}

fn find_by_prefix(dir: &Path, prefix: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let name = e.file_name();
        if name.to_string_lossy().starts_with(prefix) {
            Some(e.path())
        } else {
            None
        }
    })
}

fn read_body_from_stdin() -> String {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
        eprintln!("error: could not read from STDIN: {}", e);
        process::exit(1);
    });
    buf
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
