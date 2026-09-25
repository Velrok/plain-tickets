use std::io::Read as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use chrono::{DateTime, Utc};

use crate::application_types::{ArchiveArgs, EditArgs, ListArgs, NewArgs, WorkingDir};
use crate::config;
use crate::config::Config;
use crate::deps_graph::{self, DepsGraph};
use crate::domain_types::{
    FrontMatter, Tag, Ticket, TicketId, TicketStatus, TicketType, Title, display_width,
};
use crate::git;
use crate::graph::{DepGraph, render_forest, render_tree};

pub fn resolve_dir(flag: Option<PathBuf>) -> PathBuf {
    flag.or_else(|| std::env::var("TICKETS_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("tickets"))
}

pub fn cmd_init(base: PathBuf, force: bool) -> Result<()> {
    let config_path = base.join(".tickets.toml");
    let already_exists = config_path.exists();
    if already_exists && !force {
        bail!("already initialised — .tickets.toml already exists");
    }
    init_directories(&base)?;
    let cfg = if already_exists {
        config::load(&base)?
    } else {
        Config::default()
    };
    let config_content = toml::to_string_pretty(&cfg).context("failed to serialise config")?;
    std::fs::write(&config_path, config_content)
        .with_context(|| format!("could not write {}", config_path.display()))?;
    println!(
        "  {} {}",
        if already_exists { "rewrote" } else { "created" },
        config_path.display()
    );

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

pub fn cmd_graph(dir: WorkingDir, id: Option<TicketId>) -> Result<()> {
    let graph = DepGraph::build(&dir)?;
    let output = match id {
        Some(ref root) => render_tree(&graph, root),
        None => render_forest(&graph),
    };
    print!("{}", output);

    let cyclic = graph.cyclic_ids();
    if !cyclic.is_empty() {
        let mut ids: Vec<String> = cyclic.iter().map(ToString::to_string).collect();
        ids.sort();
        eprintln!(
            "warning: dependency cycle detected among: {}",
            ids.join(", ")
        );
    }
    Ok(())
}

/// Renders the full dependency forest: no arguments, always the whole tree.
pub fn cmd_deps_graph(dir: WorkingDir) -> Result<()> {
    let graph = DepsGraph::build(&dir)?;
    print!("{}", deps_graph::render_forest(&graph));

    let cyclic = graph.cyclic_ids();
    if !cyclic.is_empty() {
        let ids: Vec<String> = cyclic.iter().map(ToString::to_string).collect();
        eprintln!(
            "warning: dependency cycle detected among: {}",
            ids.join(", ")
        );
    }
    Ok(())
}

pub fn cmd_new(dir: WorkingDir, cfg: &Config, args: NewArgs) -> Result<()> {
    let all_dir = dir.all();

    const ALPHA: [char; 36] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    let id = TicketId::from(nanoid::nanoid!(6, &ALPHA));
    let now = Utc::now();

    let ticket_type = args.r#type.unwrap_or(cfg.new.default_type.clone());
    let status = args.status.unwrap_or(cfg.new.default_status.clone());

    let front_matter = FrontMatter {
        id: id.clone(),
        title: args.title.clone(),
        r#type: ticket_type,
        status,
        tags: args.tag,
        parent: args.parent,
        blocked_by: args.blocked_by,
        created_at: now,
        updated_at: now,
    };

    let slug = args.title.slugify();
    let filename = format!("{}_{}.md", id, slug);
    let path = all_dir.join(&filename);

    let body = match args.body.as_deref() {
        None => String::new(),
        Some("-") => read_body_from_stdin()?,
        Some(text) => text.to_string(),
    };
    let ticket = Ticket { front_matter, body };

    std::fs::write(&path, ticket.to_string())
        .with_context(|| format!("could not write {}", path.display()))?;

    if cfg.git.auto_commit {
        let message = format!("tickets: new {} \"{}\"", id, args.title);
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
        print_markdown(&ticket.body)?;
    }
    Ok(())
}

fn print_markdown(body: &str) -> Result<()> {
    let bat_ok = std::process::Command::new("bat")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if bat_ok {
        let mut child = std::process::Command::new("bat")
            .args(["--language=md", "--style=plain", "--color=auto", "-"])
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

fn matches_filters(
    ticket: &Ticket,
    statuses: &[TicketStatus],
    types: &[TicketType],
    tags: &[Tag],
) -> bool {
    if !statuses.is_empty() && !statuses.contains(&ticket.front_matter.status) {
        return false;
    }
    if !types.is_empty() && !types.contains(&ticket.front_matter.r#type) {
        return false;
    }
    tags.iter()
        .all(|tag| ticket.front_matter.tags.contains(tag))
}

/// A ticket file that exists in `all/` but could not be turned into a
/// `Ticket` — either the file could not be read, or its contents could not
/// be parsed. Recorded rather than discarded so `cmd_list` can tell the
/// user something was dropped instead of silently listing fewer tickets
/// than exist on disk (4aawv9, mirroring fd38vu's fix in the TUI).
struct LoadFailure {
    path: PathBuf,
    reason: String,
}

/// Reads every `.md` file in `all_dir`, returning the tickets that loaded
/// successfully alongside every failure encountered. Deliberately never
/// drops a failure on the floor — the caller decides how to surface
/// `failures`, but the type makes it impossible to forget them.
fn load_tickets(all_dir: &Path) -> Result<(Vec<Ticket>, Vec<LoadFailure>)> {
    let entries = std::fs::read_dir(all_dir)
        .with_context(|| format!("could not read directory {}", all_dir.display()))?
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"));

    let mut tickets = Vec::new();
    let mut failures = Vec::new();
    for entry in entries {
        let path = entry.path();
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(e) => {
                failures.push(LoadFailure {
                    path,
                    reason: e.to_string(),
                });
                continue;
            }
        };
        match raw.parse::<Ticket>() {
            Ok(ticket) => tickets.push(ticket),
            Err(reason) => failures.push(LoadFailure { path, reason }),
        }
    }
    Ok((tickets, failures))
}

/// Stderr warning naming every file `load_tickets` could not read or parse,
/// or `None` if nothing was dropped. Names the affected files (not just a
/// count) so the user has somewhere to start looking. Kept off stdout so
/// `tickets list` stays pipeable (3mqhe3 already had to fix a broken-pipe
/// regression on this same loop).
fn format_load_failures(failures: &[LoadFailure]) -> Option<String> {
    if failures.is_empty() {
        return None;
    }
    let details: Vec<String> = failures
        .iter()
        .map(|f| {
            let name = f
                .path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| f.path.display().to_string());
            format!("{name} ({})", f.reason)
        })
        .collect();
    Some(format!(
        "warning: {} ticket{} could not be loaded: {}",
        failures.len(),
        if failures.len() == 1 { "" } else { "s" },
        details.join(", ")
    ))
}

pub fn cmd_list(dir: WorkingDir, _cfg: &Config, args: ListArgs) -> Result<()> {
    let all_dir = dir.all();
    let unblocked_ctx = args
        .unblocked
        .then(|| -> Result<_> {
            let active = deps_graph::load_active(&dir)?;
            let archived = deps_graph::load_archived(&dir)?;
            Ok((active, archived))
        })
        .transpose()?;

    let (loaded_tickets, failures) = load_tickets(&all_dir)?;
    if let Some(warning) = format_load_failures(&failures) {
        eprintln!("{warning}");
    }

    let mut tickets: Vec<Ticket> = loaded_tickets
        .into_iter()
        .filter(|t| matches_filters(t, &args.status, &args.r#type, &args.tag))
        .filter(|t| {
            unblocked_ctx
                .as_ref()
                .is_none_or(|(active, archived)| deps_graph::is_unblocked(active, archived, t))
        })
        .collect();

    tickets.sort_by(|a, b| {
        let status_order = |s: &TicketStatus| match s {
            TicketStatus::InProgress => 0,
            TicketStatus::Review => 1,
            TicketStatus::Todo => 2,
            TicketStatus::Draft => 3,
            TicketStatus::Done => 4,
            TicketStatus::Rejected => 5,
        };
        status_order(&a.front_matter.status)
            .cmp(&status_order(&b.front_matter.status))
            .then(a.front_matter.created_at.cmp(&b.front_matter.created_at))
    });

    // Columns are keyed off display width (terminal columns), not byte or
    // `char` length — the type column carries an emoji alongside the word,
    // and a double-width glyph like `🐛` is 4 bytes, 1 `char`, but 2
    // columns. Rust's built-in `{:<width$}` string padding pads by `char`
    // count, which is *also* wrong here, so columns are padded by hand via
    // `pad_to_display_width` below rather than relying on it.
    let rows: Vec<(String, String, String, &Title)> = tickets
        .iter()
        .map(|t| {
            let fm = &t.front_matter;
            (
                fm.id.to_string(),
                fm.status.to_string(),
                format!("{} {}", fm.r#type.emoji(), fm.r#type),
                &fm.title,
            )
        })
        .collect();

    let id_w = rows
        .iter()
        .map(|(id, ..)| display_width(id))
        .max()
        .unwrap_or(6)
        .max(6);
    let status_w = rows
        .iter()
        .map(|(_, status, ..)| display_width(status))
        .max()
        .unwrap_or(6)
        .max(6);
    let type_w = rows
        .iter()
        .map(|(_, _, type_col, _)| display_width(type_col))
        .max()
        .unwrap_or(4)
        .max(4);

    // Colour is applied *after* padding, never before: ANSI escapes are
    // zero-display-width bytes, so colouring first would make the padding
    // calculation above count escape bytes as columns and break alignment.
    // `anstream::stdout()` strips the escapes back out again when stdout
    // isn't a colour-capable terminal (respecting `NO_COLOR` and
    // `CLICOLOR_FORCE`), so writing through it here is enough to keep piped
    // output byte-identical to the uncoloured path.
    let mut out = anstream::stdout();
    for (ticket, (id, status, type_col, title)) in tickets.iter().zip(&rows) {
        let padded_status = pad_to_display_width(status, status_w);
        let styled_status = match ticket.front_matter.status.style() {
            Some(style) => format!("{style}{padded_status}{style:#}"),
            None => padded_status,
        };
        let result = writeln!(
            out,
            "{}  {}  {}  {}",
            pad_to_display_width(id, id_w),
            styled_status,
            pad_to_display_width(type_col, type_w),
            title,
        );
        // A downstream reader closing early (`tickets list | head`) shows up
        // here as `BrokenPipe` — that is the reader choosing to stop, not a
        // failure of `list` itself, so it exits cleanly and silently rather
        // than surfacing as `error: ...` with a non-zero exit. Any other
        // write error (disk full, I/O error) still propagates.
        match result {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => return Ok(()),
            Err(e) => return Err(e).context("failed to write to stdout"),
        }
    }
    Ok(())
}

/// Right-pad `s` with spaces so it occupies `width` terminal columns,
/// measured via [`display_width`] rather than byte or `char` length.
fn pad_to_display_width(s: &str, width: usize) -> String {
    let padding = width.saturating_sub(display_width(s));
    format!("{s}{}", " ".repeat(padding))
}

pub fn cmd_edit(dir: WorkingDir, cfg: &Config, args: EditArgs) -> Result<()> {
    let all_dir = dir.all();
    let path = find_ticket(&all_dir, &args.id)?;

    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("could not read {}", path.display()))?;

    let mut ticket: Ticket = raw
        .parse()
        .map_err(|e| anyhow::anyhow!("could not parse {}: {e}", path.display()))?;

    let title_changed = apply_edits(&mut ticket, args)?;

    std::fs::write(&path, ticket.to_string())
        .with_context(|| format!("could not write {}", path.display()))?;

    let new_path = title_changed
        .then(|| {
            renamed_path(
                &all_dir,
                &path,
                &ticket.front_matter.id,
                &ticket.front_matter.title,
            )
        })
        .flatten();

    let final_path = commit_edit(cfg, path, new_path, &ticket)?;

    println!(
        "updated {}",
        final_path.file_name().unwrap().to_string_lossy()
    );
    Ok(())
}

/// Applies `args` onto `ticket`'s front matter and body. Returns whether the
/// title was changed (the caller uses this to decide if a rename is needed).
fn apply_edits(ticket: &mut Ticket, args: EditArgs) -> Result<bool> {
    let title_changed = args.title.is_some();

    let fm = &mut ticket.front_matter;
    if let Some(t) = args.title {
        fm.title = t;
    }
    if let Some(t) = args.r#type {
        fm.r#type = t;
    }
    if let Some(s) = args.status {
        fm.status = s;
    }
    if !args.tag.is_empty() {
        fm.tags = args.tag;
    }
    if args.clear_parent {
        fm.parent = None;
    } else if args.parent.is_some() {
        fm.parent = args.parent;
    }
    if args.clear_blocked_by {
        fm.blocked_by = vec![];
    } else if !args.blocked_by.is_empty() {
        fm.blocked_by = args.blocked_by;
    }
    fm.updated_at = Utc::now();

    if let Some(b) = args.body {
        ticket.body = match b.as_str() {
            "-" => read_body_from_stdin()?,
            _ => b,
        };
    }

    Ok(title_changed)
}

/// Persists an edited ticket at `path`, renaming to `new_path` when set, and
/// commits the change under `cfg.git.auto_commit`. Returns the file's final path.
fn commit_edit(
    cfg: &Config,
    path: PathBuf,
    new_path: Option<PathBuf>,
    ticket: &Ticket,
) -> Result<PathBuf> {
    let message = format!(
        "tickets: edit {} \"{}\"",
        ticket.front_matter.id, ticket.front_matter.title
    );

    if let Some(new_path) = new_path {
        if cfg.git.auto_commit {
            git::git_mv(Path::new("."), &path, &new_path, &message)?;
        } else {
            std::fs::rename(&path, &new_path)
                .with_context(|| format!("could not rename {}", path.display()))?;
        }
        Ok(new_path)
    } else {
        if cfg.git.auto_commit {
            git::git_commit(Path::new("."), &path, &message)?;
        }
        Ok(path)
    }
}

/// Returns the path the ticket file should live at given its (possibly new) title,
/// or `None` if the slug embedded in `current_path`'s filename is already up to date.
fn renamed_path(
    all_dir: &Path,
    current_path: &Path,
    id: &TicketId,
    title: &crate::domain_types::Title,
) -> Option<PathBuf> {
    let current_filename = current_path.file_name()?.to_string_lossy();
    if title.slug_matches_filename(id, &current_filename) {
        return None;
    }
    Some(all_dir.join(format!("{id}_{}.md", title.slugify())))
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

pub fn cmd_archive(dir: WorkingDir, cfg: &Config, args: ArchiveArgs) -> Result<()> {
    if args.all_rejected && !args.ids.is_empty() {
        bail!("--all-rejected and explicit IDs are mutually exclusive");
    }

    if args.all_rejected {
        archive_all_rejected(&dir, cfg)
    } else {
        archive_by_ids(&dir, &args.ids, cfg)
    }
}

fn archive_by_ids(dir: &WorkingDir, ids: &[TicketId], cfg: &Config) -> Result<()> {
    // Validate all IDs upfront before moving anything
    let mut errors: Vec<String> = Vec::new();
    let mut paths: Vec<(PathBuf, PathBuf)> = Vec::new(); // (src, dst)

    for id in ids {
        let prefix = format!("{}_", id);
        let in_all = find_by_prefix(&dir.all(), &prefix);
        let in_archived = find_by_prefix(&dir.archived(), &prefix);

        match (in_all, in_archived) {
            (Some(src), _) => {
                let dst = dir.archived().join(src.file_name().unwrap());
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

fn archive_all_rejected(dir: &WorkingDir, cfg: &Config) -> Result<()> {
    let tickets: Vec<(Ticket, PathBuf)> = std::fs::read_dir(dir.all())
        .with_context(|| format!("could not read {}", dir.all().display()))?
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
        let dst = dir.archived().join(src.file_name().unwrap());
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".testing")
            .join(format!("commands_{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn init_scaffold_round_trips_to_default_config() {
        let dir = tmp_dir("init_round_trip");
        cmd_init(dir.clone(), false).unwrap();
        let loaded = crate::config::load(&dir).unwrap();
        assert_eq!(loaded, crate::config::Config::default());
    }

    fn new_args(title: &str) -> NewArgs {
        NewArgs {
            title: title.parse().unwrap(),
            r#type: None,
            tag: vec![],
            parent: None,
            blocked_by: vec![],
            status: None,
            body: None,
        }
    }

    fn edit_args(id: TicketId) -> EditArgs {
        EditArgs {
            id,
            title: None,
            r#type: None,
            status: None,
            tag: vec![],
            parent: None,
            clear_parent: false,
            blocked_by: vec![],
            clear_blocked_by: false,
            body: None,
        }
    }

    fn only_file_in(dir: &Path) -> PathBuf {
        fs::read_dir(dir).unwrap().flatten().next().unwrap().path()
    }

    #[test]
    fn edit_title_renames_file_to_match_new_slug() {
        let dir = tmp_dir("edit_rename");
        cmd_init(dir.clone(), false).unwrap();
        let cfg = Config::default();

        cmd_new(
            WorkingDir::new(dir.clone()).unwrap(),
            &cfg,
            new_args("Original Title"),
        )
        .unwrap();

        let all_dir = dir.join("all");
        let old_path = only_file_in(&all_dir);
        let old_filename = old_path.file_name().unwrap().to_string_lossy().to_string();
        let id: TicketId = old_filename.split('_').next().unwrap().parse().unwrap();

        let mut args = edit_args(id.clone());
        args.title = Some("Brand New Title".parse().unwrap());
        cmd_edit(WorkingDir::new(dir.clone()).unwrap(), &cfg, args).unwrap();

        let expected = all_dir.join(format!("{}_brand-new-title.md", id));
        assert!(
            expected.exists(),
            "expected renamed file {} to exist; dir contains: {:?}",
            expected.display(),
            fs::read_dir(&all_dir)
                .unwrap()
                .flatten()
                .map(|e| e.file_name())
                .collect::<Vec<_>>()
        );
        assert!(!old_path.exists(), "expected old filename to be gone");
    }

    #[test]
    fn edit_non_title_field_does_not_rename_file() {
        let dir = tmp_dir("edit_status_only");
        cmd_init(dir.clone(), false).unwrap();
        let cfg = Config::default();

        cmd_new(
            WorkingDir::new(dir.clone()).unwrap(),
            &cfg,
            new_args("Untouched Title"),
        )
        .unwrap();

        let all_dir = dir.join("all");
        let path = only_file_in(&all_dir);
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        let id: TicketId = filename.split('_').next().unwrap().parse().unwrap();

        let mut args = edit_args(id);
        args.status = Some(TicketStatus::Todo);
        cmd_edit(WorkingDir::new(dir.clone()).unwrap(), &cfg, args).unwrap();

        assert!(
            path.exists(),
            "filename should be unchanged after editing status"
        );
        assert_eq!(fs::read_dir(&all_dir).unwrap().flatten().count(), 1);
    }

    #[test]
    fn edit_title_to_same_value_does_not_rename_file() {
        let dir = tmp_dir("edit_title_unchanged");
        cmd_init(dir.clone(), false).unwrap();
        let cfg = Config::default();

        cmd_new(
            WorkingDir::new(dir.clone()).unwrap(),
            &cfg,
            new_args("Same Title"),
        )
        .unwrap();

        let all_dir = dir.join("all");
        let path = only_file_in(&all_dir);
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        let id: TicketId = filename.split('_').next().unwrap().parse().unwrap();

        let mut args = edit_args(id);
        args.title = Some("Same Title".parse().unwrap());
        cmd_edit(WorkingDir::new(dir.clone()).unwrap(), &cfg, args).unwrap();

        assert!(
            path.exists(),
            "filename should be unchanged when title is set to its current value"
        );
        assert_eq!(fs::read_dir(&all_dir).unwrap().flatten().count(), 1);
    }

    #[test]
    fn init_scaffold_lists_every_config_key() {
        let dir = tmp_dir("init_scaffold_keys");
        cmd_init(dir.clone(), false).unwrap();
        let content = fs::read_to_string(dir.join(".tickets.toml")).unwrap();
        for expected in [
            "[git]",
            "auto_commit",
            "[tui]",
            "kanban_columns",
            "[new]",
            "default_status",
            "default_type",
        ] {
            assert!(
                content.contains(expected),
                "expected scaffold to contain {expected:?}, got:\n{content}"
            );
        }
    }
}
