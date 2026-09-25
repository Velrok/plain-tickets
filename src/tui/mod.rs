mod app;
mod render;

pub use app::{App, Screen};

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify::{RecursiveMode, Watcher};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::application_types::WorkingDir;
use crate::config::Config;
use crate::domain_types::{FrontMatter, Ticket, TicketId, TicketStatus, TicketType, Title};

use app::{Cmd, Message, update};

// ── public entry point ────────────────────────────────────────────────────────

pub fn run(working_dir: WorkingDir, cfg: &Config) -> Result<()> {
    let columns = cfg.tui.kanban_columns.clone();
    let mut app = build_app(&working_dir, columns)?;

    // Watch tickets/all/ for external file changes.
    let (fs_tx, fs_rx) = mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(fs_tx)?;
    watcher.watch(&working_dir.all(), RecursiveMode::NonRecursive)?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, &mut app, &working_dir, cfg, &fs_rx);

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    result
}

// ── event loop ────────────────────────────────────────────────────────────────

fn event_loop<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    working_dir: &WorkingDir,
    cfg: &Config,
    fs_rx: &mpsc::Receiver<notify::Result<notify::Event>>,
) -> Result<()> {
    loop {
        terminal.draw(|f| render::view(f, app))?;

        // Drain any pending file-system events and reload if anything changed.
        let mut fs_changed = false;
        while fs_rx.try_recv().is_ok() {
            fs_changed = true;
        }
        if fs_changed {
            reload_tickets(app, working_dir)?;
        }

        // Poll for a key event with a short timeout so the loop stays responsive
        // to file changes even when the user is idle.
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            let Some(msg) = key_to_message(key.code, &app.screen) else {
                continue;
            };
            match update(app, msg) {
                Cmd::None => {}
                Cmd::Quit => return Ok(()),
                Cmd::SaveFocused => save_focused(app, working_dir, cfg)?,
                Cmd::OpenEditor => open_in_editor(terminal, app, working_dir)?,
                Cmd::CreateAndEdit => create_and_edit(terminal, app, working_dir)?,
                Cmd::CopyId(id) => copy_id_to_clipboard(app, &id),
            }
        }
    }
}

// ── input → message ───────────────────────────────────────────────────────────

fn key_to_message(code: KeyCode, screen: &Screen) -> Option<Message> {
    match screen {
        // `/` does nothing here — filtering is board-only scope.
        Screen::Help => match code {
            KeyCode::Char('/') => None,
            _ => Some(Message::CloseOverlay),
        },
        Screen::Detail => match code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Message::CloseOverlay),
            KeyCode::Char('e') => Some(Message::OpenEditor),
            _ => None,
        },
        Screen::Board => match code {
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('h') | KeyCode::Left => Some(Message::MoveLeft),
            KeyCode::Char('l') | KeyCode::Right => Some(Message::MoveRight),
            KeyCode::Char('j') | KeyCode::Down => Some(Message::MoveDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::MoveUp),
            KeyCode::Char('H') => Some(Message::MoveTicketLeft),
            KeyCode::Char('L') => Some(Message::MoveTicketRight),
            KeyCode::Enter | KeyCode::Char(' ') => Some(Message::OpenDetail),
            KeyCode::Char('e') => Some(Message::OpenEditor),
            KeyCode::Char('n') => Some(Message::NewTicket),
            KeyCode::Char('y') => Some(Message::CopyId),
            KeyCode::Char('?') | KeyCode::F(1) => Some(Message::ToggleHelp),
            KeyCode::Char('/') => Some(Message::OpenFilter),
            KeyCode::Esc => Some(Message::ClearFilter),
            _ => None,
        },
        // Total key capture: every printable character is literal query text,
        // never a command. Only Enter/Esc/Backspace are control keys here.
        Screen::Filter => match code {
            KeyCode::Enter => Some(Message::FilterCommit),
            KeyCode::Esc => Some(Message::FilterCancel),
            KeyCode::Backspace => Some(Message::FilterBackspace),
            KeyCode::Char(c) => Some(Message::FilterInput(c)),
            _ => None,
        },
    }
}

// ── Cmd handlers (side effects) ───────────────────────────────────────────────

fn save_focused(app: &App, working_dir: &WorkingDir, cfg: &Config) -> Result<()> {
    let Some(ticket) = app.focused_ticket() else {
        return Ok(());
    };
    let Some(path) = find_ticket_path(working_dir, ticket) else {
        return Ok(());
    };
    std::fs::write(&path, ticket.to_string())
        .with_context(|| format!("could not write {}", path.display()))?;
    if cfg.git.auto_commit {
        let msg = format!(
            "tickets: edit {} \"{}\"",
            ticket.front_matter.id, ticket.front_matter.title
        );
        crate::git::git_commit_silent(Path::new("."), &path, &msg)?;
    }
    Ok(())
}

fn open_in_editor<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    working_dir: &WorkingDir,
) -> Result<()> {
    let Some(ticket) = app.focused_ticket() else {
        return Ok(());
    };
    let Some(path) = find_ticket_path(working_dir, ticket) else {
        return Ok(());
    };
    suspend(terminal)?;
    launch_editor(&path);
    resume(terminal)?;
    reload_tickets(app, working_dir)?;
    Ok(())
}

fn create_and_edit<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    working_dir: &WorkingDir,
) -> Result<()> {
    // Determine status from the current column.
    let status = app.columns[app.col].clone();

    let path = create_draft_ticket(working_dir, status)?;

    suspend(terminal)?;
    launch_editor(&path);
    resume(terminal)?;
    reload_tickets(app, working_dir)?;
    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn copy_id_to_clipboard(app: &mut App, id: &str) {
    let msg = match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(id)) {
        Ok(()) => format!("Copied {}", id),
        Err(_) => "Clipboard unavailable".to_string(),
    };
    app.flash = Some((msg, Instant::now()));
}

/// Build the initial `App` from disk, surfacing any tickets that failed to
/// read or parse as a flash message rather than dropping them silently.
///
/// This runs before the terminal is touched (see `run`), so a flash seeded
/// here is the *only* signal available at startup — there is no `App` yet
/// for a later call site to write to. See fd38vu: the motivating failure
/// (a stale binary that couldn't deserialise `status: review`) happened
/// exactly at this point, before any per-key command had a chance to run.
fn build_app(working_dir: &WorkingDir, columns: Vec<TicketStatus>) -> Result<App> {
    let (tickets, failures) = load_tickets(working_dir)?;
    let mut app = App::new(tickets, columns);
    seed_load_flash(&mut app, &failures);
    Ok(app)
}

/// Reload tickets into an already-running `App` (e.g. after an external
/// edit or a filesystem watch event), surfacing any read/parse failures the
/// same way `build_app` does at startup.
fn reload_tickets(app: &mut App, working_dir: &WorkingDir) -> Result<()> {
    let (tickets, failures) = load_tickets(working_dir)?;
    app.set_tickets(tickets);
    seed_load_flash(app, &failures);
    Ok(())
}

fn seed_load_flash(app: &mut App, failures: &[LoadFailure]) {
    if let Some(msg) = format_load_failures(failures) {
        app.flash = Some((msg, Instant::now()));
    }
}

/// A ticket file that exists in `tickets/all/` but could not be turned into
/// a `Ticket` — either the file could not be read, or its contents could
/// not be parsed. Recorded rather than discarded so the caller can tell the
/// user something was dropped (fd38vu).
struct LoadFailure {
    path: PathBuf,
    reason: String,
}

/// Reads every `.md` file in `working_dir`'s `all/` directory, returning the
/// tickets that loaded successfully alongside every failure encountered.
/// Deliberately never drops a failure on the floor — the caller decides how
/// to surface `failures`, but the type makes it impossible to forget them.
fn load_tickets(working_dir: &WorkingDir) -> Result<(Vec<Ticket>, Vec<LoadFailure>)> {
    let all_dir = working_dir.all();
    let entries = std::fs::read_dir(&all_dir)
        .with_context(|| format!("could not read {}", all_dir.display()))?
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

/// Footer flash text for tickets `load_tickets` could not read or parse, or
/// `None` if nothing was dropped. Names the affected files (not just a
/// count) so the user has somewhere to start looking.
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
        "{} ticket{} could not be loaded: {}",
        failures.len(),
        if failures.len() == 1 { "" } else { "s" },
        details.join(", ")
    ))
}

fn find_ticket_path(working_dir: &WorkingDir, ticket: &Ticket) -> Option<PathBuf> {
    let prefix = format!("{}_", ticket.front_matter.id);
    std::fs::read_dir(working_dir.all())
        .ok()?
        .flatten()
        .find_map(|e| {
            if e.file_name().to_string_lossy().starts_with(&prefix) {
                Some(e.path())
            } else {
                None
            }
        })
}

fn create_draft_ticket(working_dir: &WorkingDir, status: TicketStatus) -> Result<PathBuf> {
    const ALPHA: [char; 36] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    let id = TicketId::from(nanoid::nanoid!(6, &ALPHA));
    let now = chrono::Utc::now();
    let title: Title = "new ticket"
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let ticket = Ticket {
        front_matter: FrontMatter {
            id: id.clone(),
            title: title.clone(),
            r#type: TicketType::Task,
            status,
            tags: vec![],
            parent: None,
            blocked_by: vec![],
            created_at: now,
            updated_at: now,
        },
        body: String::new(),
    };

    let path = working_dir
        .all()
        .join(format!("{}_{}.md", id, title.slugify()));
    std::fs::write(&path, ticket.to_string())
        .with_context(|| format!("could not write {}", path.display()))?;
    Ok(path)
}

fn launch_editor(path: &Path) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let _ = std::process::Command::new(&editor).arg(path).status();
}

fn suspend<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn resume<B: ratatui::backend::Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<()> {
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}

// ── input mapping tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── load_tickets / build_app: read & parse failures (fd38vu) ────────────

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".testing")
            .join(format!("tui_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("all")).unwrap();
        std::fs::create_dir_all(dir.join("archived")).unwrap();
        dir
    }

    fn write_ticket_file(dir: &Path, filename: &str, contents: &str) {
        std::fs::write(dir.join("all").join(filename), contents).unwrap();
    }

    fn valid_ticket_contents(id: &str) -> String {
        format!(
            "---\nid: {id}\ntitle: Valid ticket\ntype: task\nstatus: todo\ntags: []\nparent: null\nblocked_by: []\ncreated_at: 2000-01-01T00:00:00Z\nupdated_at: 2000-01-01T00:00:00Z\n---\n"
        )
    }

    #[test]
    fn load_tickets_reports_unparseable_file_without_dropping_valid_ones() {
        let dir = tmp_dir("unparseable");
        write_ticket_file(&dir, "a1_valid.md", &valid_ticket_contents("a1"));
        write_ticket_file(&dir, "b2_broken.md", "---\nstatus: nonsense-status\n---\n");

        let working_dir = WorkingDir::new(dir).unwrap();
        let (tickets, failures) = load_tickets(&working_dir).unwrap();

        assert_eq!(tickets.len(), 1, "valid ticket must still load");
        assert_eq!(tickets[0].front_matter.id.to_string(), "a1");
        assert_eq!(
            failures.len(),
            1,
            "broken ticket must be recorded, not dropped"
        );
        assert!(failures[0].path.to_string_lossy().contains("b2_broken.md"));
    }

    #[test]
    fn load_tickets_reports_unreadable_file_without_dropping_valid_ones() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tmp_dir("unreadable");
        write_ticket_file(&dir, "a1_valid.md", &valid_ticket_contents("a1"));
        let locked_path = dir.join("all").join("b2_locked.md");
        std::fs::write(&locked_path, valid_ticket_contents("b2")).unwrap();
        std::fs::set_permissions(&locked_path, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Sanity-check the fixture before trusting it: some environments
        // (notably running as root) ignore mode 000 entirely.
        let still_readable = std::fs::read_to_string(&locked_path).is_ok();

        let result = load_tickets(&WorkingDir::new(dir).unwrap());

        // Restore permissions regardless of outcome, so cleanup/inspection
        // of `.testing/` afterwards isn't left blocked.
        let _ = std::fs::set_permissions(&locked_path, std::fs::Permissions::from_mode(0o644));

        if still_readable {
            eprintln!(
                "skipping load_tickets_reports_unreadable_file_without_dropping_valid_ones: \
                 chmod 000 did not make the file unreadable (running as root?)"
            );
            return;
        }

        let (tickets, failures) = result.unwrap();
        assert_eq!(tickets.len(), 1, "valid ticket must still load");
        assert_eq!(tickets[0].front_matter.id.to_string(), "a1");
        assert_eq!(
            failures.len(),
            1,
            "unreadable ticket must be recorded, not dropped"
        );
        assert!(failures[0].path.to_string_lossy().contains("b2_locked.md"));
    }

    #[test]
    fn format_load_failures_none_when_nothing_failed() {
        assert!(format_load_failures(&[]).is_none());
    }

    #[test]
    fn format_load_failures_names_files_and_counts_them() {
        let failures = vec![
            LoadFailure {
                path: PathBuf::from("a1_broken.md"),
                reason: "invalid front matter: missing field `status`".to_string(),
            },
            LoadFailure {
                path: PathBuf::from("b2_locked.md"),
                reason: "permission denied".to_string(),
            },
        ];
        let msg = format_load_failures(&failures).unwrap();
        assert!(msg.contains("2 tickets"), "message was: {msg}");
        assert!(msg.contains("a1_broken.md"), "message was: {msg}");
        assert!(msg.contains("b2_locked.md"), "message was: {msg}");
    }

    #[test]
    fn format_load_failures_singular_for_one_failure() {
        let failures = vec![LoadFailure {
            path: PathBuf::from("a1_broken.md"),
            reason: "invalid front matter".to_string(),
        }];
        let msg = format_load_failures(&failures).unwrap();
        assert!(msg.starts_with("1 ticket "), "message was: {msg}");
    }

    #[test]
    fn build_app_seeds_flash_when_a_ticket_fails_to_load() {
        let dir = tmp_dir("build_app_flash");
        write_ticket_file(&dir, "a1_valid.md", &valid_ticket_contents("a1"));
        write_ticket_file(&dir, "b2_broken.md", "not front matter at all");

        let working_dir = WorkingDir::new(dir).unwrap();
        let app = build_app(&working_dir, vec![crate::domain_types::TicketStatus::Todo]).unwrap();

        assert_eq!(app.tickets.len(), 1);
        let (msg, _) = app.flash.expect("startup flash must be seeded when a ticket fails to load — this is the only signal available before any key is pressed");
        assert!(msg.contains("b2_broken.md"), "flash was: {msg}");
    }

    #[test]
    fn build_app_no_flash_when_everything_loads_cleanly() {
        let dir = tmp_dir("build_app_no_flash");
        write_ticket_file(&dir, "a1_valid.md", &valid_ticket_contents("a1"));

        let working_dir = WorkingDir::new(dir).unwrap();
        let app = build_app(&working_dir, vec![crate::domain_types::TicketStatus::Todo]).unwrap();

        assert_eq!(app.tickets.len(), 1);
        assert!(app.flash.is_none());
    }

    #[test]
    fn board_y_maps_to_copy_id() {
        assert_eq!(
            key_to_message(KeyCode::Char('y'), &Screen::Board),
            Some(Message::CopyId)
        );
    }

    #[test]
    fn board_q_maps_to_quit() {
        assert_eq!(
            key_to_message(KeyCode::Char('q'), &Screen::Board),
            Some(Message::Quit)
        );
    }

    #[test]
    fn board_hjkl_map_to_navigation() {
        assert_eq!(
            key_to_message(KeyCode::Char('h'), &Screen::Board),
            Some(Message::MoveLeft)
        );
        assert_eq!(
            key_to_message(KeyCode::Char('l'), &Screen::Board),
            Some(Message::MoveRight)
        );
        assert_eq!(
            key_to_message(KeyCode::Char('j'), &Screen::Board),
            Some(Message::MoveDown)
        );
        assert_eq!(
            key_to_message(KeyCode::Char('k'), &Screen::Board),
            Some(Message::MoveUp)
        );
    }

    #[test]
    fn board_arrows_map_to_navigation() {
        assert_eq!(
            key_to_message(KeyCode::Left, &Screen::Board),
            Some(Message::MoveLeft)
        );
        assert_eq!(
            key_to_message(KeyCode::Right, &Screen::Board),
            Some(Message::MoveRight)
        );
        assert_eq!(
            key_to_message(KeyCode::Down, &Screen::Board),
            Some(Message::MoveDown)
        );
        assert_eq!(
            key_to_message(KeyCode::Up, &Screen::Board),
            Some(Message::MoveUp)
        );
    }

    #[test]
    fn board_shift_h_l_map_to_move_ticket() {
        assert_eq!(
            key_to_message(KeyCode::Char('H'), &Screen::Board),
            Some(Message::MoveTicketLeft)
        );
        assert_eq!(
            key_to_message(KeyCode::Char('L'), &Screen::Board),
            Some(Message::MoveTicketRight)
        );
    }

    #[test]
    fn board_enter_and_space_open_detail() {
        assert_eq!(
            key_to_message(KeyCode::Enter, &Screen::Board),
            Some(Message::OpenDetail)
        );
        assert_eq!(
            key_to_message(KeyCode::Char(' '), &Screen::Board),
            Some(Message::OpenDetail)
        );
    }

    #[test]
    fn board_question_mark_and_f1_toggle_help() {
        assert_eq!(
            key_to_message(KeyCode::Char('?'), &Screen::Board),
            Some(Message::ToggleHelp)
        );
        assert_eq!(
            key_to_message(KeyCode::F(1), &Screen::Board),
            Some(Message::ToggleHelp)
        );
    }

    #[test]
    fn detail_q_and_esc_close_overlay() {
        assert_eq!(
            key_to_message(KeyCode::Char('q'), &Screen::Detail),
            Some(Message::CloseOverlay)
        );
        assert_eq!(
            key_to_message(KeyCode::Esc, &Screen::Detail),
            Some(Message::CloseOverlay)
        );
    }

    #[test]
    fn detail_unhandled_key_returns_none() {
        assert_eq!(key_to_message(KeyCode::Char('x'), &Screen::Detail), None);
    }

    #[test]
    fn help_any_key_returns_close_overlay() {
        assert_eq!(
            key_to_message(KeyCode::Char('a'), &Screen::Help),
            Some(Message::CloseOverlay)
        );
        assert_eq!(
            key_to_message(KeyCode::Esc, &Screen::Help),
            Some(Message::CloseOverlay)
        );
    }

    // ── filter mode ───────────────────────────────────────────────────────

    #[test]
    fn board_slash_enters_filter_mode() {
        assert_eq!(
            key_to_message(KeyCode::Char('/'), &Screen::Board),
            Some(Message::OpenFilter)
        );
    }

    #[test]
    fn filter_mode_q_is_text_input_not_quit() {
        assert_eq!(
            key_to_message(KeyCode::Char('q'), &Screen::Filter),
            Some(Message::FilterInput('q'))
        );
    }

    #[test]
    fn filter_mode_other_command_keys_are_also_text_input() {
        // j/k/n/e must be literal query text while the prompt is open, not
        // navigation/create/edit commands.
        assert_eq!(
            key_to_message(KeyCode::Char('j'), &Screen::Filter),
            Some(Message::FilterInput('j'))
        );
        assert_eq!(
            key_to_message(KeyCode::Char('k'), &Screen::Filter),
            Some(Message::FilterInput('k'))
        );
        assert_eq!(
            key_to_message(KeyCode::Char('n'), &Screen::Filter),
            Some(Message::FilterInput('n'))
        );
        assert_eq!(
            key_to_message(KeyCode::Char('e'), &Screen::Filter),
            Some(Message::FilterInput('e'))
        );
    }

    #[test]
    fn filter_mode_esc_cancels_and_enter_commits() {
        assert_eq!(
            key_to_message(KeyCode::Esc, &Screen::Filter),
            Some(Message::FilterCancel)
        );
        assert_eq!(
            key_to_message(KeyCode::Enter, &Screen::Filter),
            Some(Message::FilterCommit)
        );
    }

    #[test]
    fn filter_mode_backspace_deletes_a_character() {
        assert_eq!(
            key_to_message(KeyCode::Backspace, &Screen::Filter),
            Some(Message::FilterBackspace)
        );
    }

    #[test]
    fn slash_in_detail_and_help_screens_produces_no_message() {
        assert_eq!(key_to_message(KeyCode::Char('/'), &Screen::Detail), None);
        assert_eq!(key_to_message(KeyCode::Char('/'), &Screen::Help), None);
    }

    #[test]
    fn board_esc_maps_to_clear_filter() {
        assert_eq!(
            key_to_message(KeyCode::Esc, &Screen::Board),
            Some(Message::ClearFilter)
        );
    }
}
