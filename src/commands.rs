use std::io::Read as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use chrono::{DateTime, Utc};

use crate::config::Config;
use crate::git;
use crate::types::{FrontMatter, Tag, Ticket, TicketId, TicketStatus, TicketType, Title};

pub struct WorkingDir(PathBuf);

impl WorkingDir {
    pub fn new(base: PathBuf) -> Result<Self> {
        let wd = WorkingDir(base);
        if !wd.all().exists() || !wd.archived().exists() {
            bail!("tickets directory not initialised — run `tickets init` first");
        }
        Ok(wd)
    }

    pub fn all(&self) -> PathBuf { self.0.join("all") }
    pub fn archived(&self) -> PathBuf { self.0.join("archived") }
    pub fn config_path(&self) -> PathBuf { self.0.join(".tickets.toml") }
}

impl AsRef<Path> for WorkingDir {
    fn as_ref(&self) -> &Path { &self.0 }
}

pub fn resolve_dir(flag: Option<PathBuf>) -> PathBuf {
    flag.or_else(|| std::env::var("TICKETS_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("tickets"))
}

pub fn cmd_init(base: PathBuf) -> Result<()> {
    let config_path = base.join(".tickets.toml");
    if config_path.exists() {
        bail!("already initialised — .tickets.toml already exists");
    }
    init_directories(&base)?;
    let config_content = "\
# plain-tickets configuration
# Uncomment and set values to override defaults.

# [git]
# auto_commit = false
";
    std::fs::write(&config_path, config_content)
        .with_context(|| format!("could not create {}", config_path.display()))?;
    println!("  created {}", config_path.display());

    if git_detect(&base).is_ok() {
        println!(
            "hint: git repository detected — set auto_commit = true in .tickets.toml to commit on every change"
        );
    }
    Ok(())
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
    dir: WorkingDir,
    cfg: &Config,
    title: Title,
    ticket_type: TicketType,
    status: TicketStatus,
    tags: Vec<Tag>,
    parent: Option<TicketId>,
    blocked_by: Vec<TicketId>,
    body: Option<String>,
) -> Result<()> {
    let all_dir = dir.all();

    const ALPHA: [char; 36] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
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
        Some("-") => read_body_from_stdin()?,
        Some(text) => text.to_string(),
    };
    let ticket = Ticket { front_matter, body };

    std::fs::write(&path, ticket.to_string())
        .with_context(|| format!("could not write {}", path.display()))?;

    if cfg.git.auto_commit {
        let message = format!("tickets: new {} \"{}\"", id, title);
        git::git_commit(Path::new("."), &path, &message)?;
    }

    println!("{} {}", id, filename);
    Ok(())
}

pub fn cmd_show(dir: WorkingDir, id: TicketId) -> Result<()> {
    let path = find_ticket(&dir.all(), &id)?;
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("could not read {}", path.display()))?;
    let ticket: Ticket = raw
        .parse()
        .map_err(|e| anyhow::anyhow!("could not parse {}: {e}", path.display()))?;
    print_ticket(&ticket)?;
    Ok(())
}

fn relative_time(dt: DateTime<Utc>) -> String {
    let secs = (Utc::now() - dt).num_seconds().max(0);
    if secs < 60 {
        return "just now".to_string();
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{mins} minute{} ago", if mins == 1 { "" } else { "s" });
    }
    let hours = mins / 60;
    if hours < 24 {
        return format!("{hours} hour{} ago", if hours == 1 { "" } else { "s" });
    }
    let days = hours / 24;
    if days < 30 {
        return format!("{days} day{} ago", if days == 1 { "" } else { "s" });
    }
    let months = days / 30;
    if months < 12 {
        return format!("{months} month{} ago", if months == 1 { "" } else { "s" });
    }
    let years = months / 12;
    format!("{years} year{} ago", if years == 1 { "" } else { "s" })
}

fn fmt_timestamp(dt: DateTime<Utc>) -> String {
    format!("{} · {}", dt.format("%Y-%m-%d"), relative_time(dt))
}

fn print_ticket(ticket: &Ticket) -> Result<()> {
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
        print_body(&ticket.body)?;
    }
    Ok(())
}

fn print_body(body: &str) -> Result<()> {
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
            .context("could not spawn bat")?;
        use std::io::Write as _;
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(body.as_bytes());
        }
        let _ = child.wait();
    } else {
        print!("{}", body);
    }
    Ok(())
}

pub fn cmd_list(dir: WorkingDir, _cfg: &Config) -> Result<()> {
    let all_dir = dir.all();
    let mut tickets: Vec<Ticket> = std::fs::read_dir(&all_dir)
        .with_context(|| format!("could not read directory {}", all_dir.display()))?
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

    let id_w = tickets
        .iter()
        .map(|t| t.front_matter.id.to_string().len())
        .max()
        .unwrap_or(6)
        .max(6);
    let status_w = tickets
        .iter()
        .map(|t| t.front_matter.status.to_string().len())
        .max()
        .unwrap_or(6)
        .max(6);
    let type_w = tickets
        .iter()
        .map(|t| t.front_matter.r#type.to_string().len())
        .max()
        .unwrap_or(4)
        .max(4);

    for ticket in &tickets {
        let fm = &ticket.front_matter;
        println!(
            "{:<id_w$}  {:<status_w$}  {:<type_w$}  {}",
            fm.id,
            fm.status,
            fm.r#type,
            fm.title,
            id_w = id_w,
            status_w = status_w,
            type_w = type_w,
        );
    }
    Ok(())
}

pub fn cmd_edit(
    dir: WorkingDir,
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
) -> Result<()> {
    let all_dir = dir.all();
    let path = find_ticket(&all_dir, &id)?;

    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("could not read {}", path.display()))?;

    let mut ticket: Ticket = raw
        .parse()
        .map_err(|e| anyhow::anyhow!("could not parse {}: {e}", path.display()))?;

    let fm = &mut ticket.front_matter;
    if let Some(t) = title {
        fm.title = t;
    }
    if let Some(t) = ticket_type {
        fm.r#type = t;
    }
    if let Some(s) = status {
        fm.status = s;
    }
    if !tags.is_empty() {
        fm.tags = tags;
    }
    if clear_parent {
        fm.parent = None;
    } else if parent.is_some() {
        fm.parent = parent;
    }
    if clear_blocked_by {
        fm.blocked_by = vec![];
    } else if !blocked_by.is_empty() {
        fm.blocked_by = blocked_by;
    }
    fm.updated_at = Utc::now();

    if let Some(b) = body {
        ticket.body = match b.as_str() {
            "-" => read_body_from_stdin()?,
            _ => b,
        };
    }

    std::fs::write(&path, ticket.to_string())
        .with_context(|| format!("could not write {}", path.display()))?;

    if cfg.git.auto_commit {
        let message = format!(
            "tickets: edit {} \"{}\"",
            ticket.front_matter.id, ticket.front_matter.title
        );
        git::git_commit(Path::new("."), &path, &message)?;
    }

    println!("updated {}", path.file_name().unwrap().to_string_lossy());
    Ok(())
}

fn find_ticket(dir: &Path, id: &TicketId) -> Result<PathBuf> {
    let prefix = format!("{}_", id);
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("could not read directory {}", dir.display()))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&prefix) {
            return Ok(entry.path());
        }
    }
    bail!("no ticket found with id {}", id);
}

pub fn cmd_archive(
    dir: WorkingDir,
    cfg: &Config,
    ids: Vec<TicketId>,
    all_rejected: bool,
) -> Result<()> {
    let all_dir = dir.all();
    let archived_dir = dir.archived();

    if all_rejected && !ids.is_empty() {
        bail!("--all-rejected and explicit IDs are mutually exclusive");
    }

    if all_rejected {
        archive_all_rejected(&all_dir, &archived_dir, cfg)
    } else {
        archive_by_ids(&all_dir, &archived_dir, &ids, cfg)
    }
}

fn archive_by_ids(
    all_dir: &Path,
    archived_dir: &Path,
    ids: &[TicketId],
    cfg: &Config,
) -> Result<()> {
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
        bail!("no files moved");
    }

    for (src, dst) in &paths {
        let id = dst
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.split('_').next())
            .unwrap_or("?");
        if cfg.git.auto_commit {
            let message = format!("tickets: archive {id}");
            git::git_mv(Path::new("."), src, dst, &message)?;
        } else {
            std::fs::rename(src, dst)
                .with_context(|| format!("could not move {}", src.display()))?;
        }
        println!("{}  archived → {}", id, dst.display());
    }
    Ok(())
}

fn archive_all_rejected(
    all_dir: &Path,
    archived_dir: &Path,
    cfg: &Config,
) -> Result<()> {
    let tickets: Vec<(Ticket, PathBuf)> = std::fs::read_dir(all_dir)
        .with_context(|| format!("could not read {}", all_dir.display()))?
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .filter_map(|e| {
            let path = e.path();
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|raw| raw.parse::<Ticket>().ok())
                .map(|t| (t, path))
        })
        .filter(|(t, _)| t.front_matter.status == TicketStatus::Rejected)
        .collect();

    if tickets.is_empty() {
        eprintln!("nothing to archive");
        return Ok(());
    }

    for (_, src) in &tickets {
        let dst = archived_dir.join(src.file_name().unwrap());
        let id = dst
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.split('_').next())
            .unwrap_or("?");
        if cfg.git.auto_commit {
            let message = format!("tickets: archive {id}");
            git::git_mv(Path::new("."), src, &dst, &message)?;
        } else {
            std::fs::rename(src, &dst)
                .with_context(|| format!("could not move {}", src.display()))?;
        }
        println!("{}  archived → {}", id, dst.display());
    }
    Ok(())
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

fn read_body_from_stdin() -> Result<String> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("could not read from STDIN")?;
    Ok(buf)
}

fn init_directories(dir: &Path) -> Result<()> {
    let all = dir.join("all");
    let archived = dir.join("archived");

    for path in [&all, &archived] {
        if path.exists() {
            println!("  exists  {}", path.display());
        } else {
            std::fs::create_dir_all(path)
                .with_context(|| format!("could not create {}", path.display()))?;
            println!("  created {}", path.display());
        }
    }
    Ok(())
}
