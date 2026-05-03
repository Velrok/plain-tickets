mod app;
mod render;

pub use app::{App, Screen};

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Context as _, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify::{RecursiveMode, Watcher};
use ratatui::{Terminal, backend::CrosstermBackend};

use clap::ValueEnum as _;

use crate::application_types::WorkingDir;
use crate::config::Config;
use crate::domain_types::{FrontMatter, Ticket, TicketId, TicketStatus, TicketType, Title};

use app::{Cmd, Message, update};

// ── public entry point ────────────────────────────────────────────────────────

pub fn run(working_dir: WorkingDir, cfg: &Config) -> Result<()> {
    let columns = vec!["todo".to_string(), "in-progress".to_string(), "done".to_string()];
    let tickets = load_tickets(&working_dir)?;
    let mut app = App::new(tickets, columns);

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
            app.set_tickets(load_tickets(working_dir)?);
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
            }
        }
    }
}

// ── input → message ───────────────────────────────────────────────────────────

fn key_to_message(code: KeyCode, screen: &Screen) -> Option<Message> {
    match screen {
        Screen::Help => Some(Message::CloseOverlay),
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
            KeyCode::Char('?') | KeyCode::F(1) => Some(Message::ToggleHelp),
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
    app.set_tickets(load_tickets(working_dir)?);
    Ok(())
}

fn create_and_edit<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    working_dir: &WorkingDir,
) -> Result<()> {
    // Determine status from the current column.
    let col_name = app.columns[app.col].clone();
    let status = TicketStatus::from_str(&col_name, true).unwrap_or(TicketStatus::Todo);

    let path = create_draft_ticket(working_dir, status)?;

    suspend(terminal)?;
    launch_editor(&path);
    resume(terminal)?;
    app.set_tickets(load_tickets(working_dir)?);
    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn load_tickets(working_dir: &WorkingDir) -> Result<Vec<Ticket>> {
    let all_dir = working_dir.all();
    let tickets = std::fs::read_dir(&all_dir)
        .with_context(|| format!("could not read {}", all_dir.display()))?
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|raw| raw.parse::<Ticket>().ok())
        .collect();
    Ok(tickets)
}

fn find_ticket_path(working_dir: &WorkingDir, ticket: &Ticket) -> Option<PathBuf> {
    let prefix = format!("{}_", ticket.front_matter.id);
    std::fs::read_dir(working_dir.all()).ok()?.flatten().find_map(|e| {
        if e.file_name().to_string_lossy().starts_with(&prefix) {
            Some(e.path())
        } else {
            None
        }
    })
}

fn create_draft_ticket(working_dir: &WorkingDir, status: TicketStatus) -> Result<PathBuf> {
    const ALPHA: [char; 36] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
        'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
        'y', 'z',
    ];
    let id = TicketId::from(nanoid::nanoid!(6, &ALPHA));
    let now = chrono::Utc::now();
    let title: Title = "new ticket".parse().map_err(|e: String| anyhow::anyhow!(e))?;

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

    let path = working_dir.all().join(format!("{}_{}.md", id, title.slugify()));
    std::fs::write(&path, ticket.to_string())
        .with_context(|| format!("could not write {}", path.display()))?;
    Ok(path)
}

fn launch_editor(path: &Path) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let _ = std::process::Command::new(&editor).arg(path).status();
}

fn suspend<B: ratatui::backend::Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<()> {
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

    #[test]
    fn board_q_maps_to_quit() {
        assert_eq!(key_to_message(KeyCode::Char('q'), &Screen::Board), Some(Message::Quit));
    }

    #[test]
    fn board_hjkl_map_to_navigation() {
        assert_eq!(key_to_message(KeyCode::Char('h'), &Screen::Board), Some(Message::MoveLeft));
        assert_eq!(key_to_message(KeyCode::Char('l'), &Screen::Board), Some(Message::MoveRight));
        assert_eq!(key_to_message(KeyCode::Char('j'), &Screen::Board), Some(Message::MoveDown));
        assert_eq!(key_to_message(KeyCode::Char('k'), &Screen::Board), Some(Message::MoveUp));
    }

    #[test]
    fn board_arrows_map_to_navigation() {
        assert_eq!(key_to_message(KeyCode::Left, &Screen::Board), Some(Message::MoveLeft));
        assert_eq!(key_to_message(KeyCode::Right, &Screen::Board), Some(Message::MoveRight));
        assert_eq!(key_to_message(KeyCode::Down, &Screen::Board), Some(Message::MoveDown));
        assert_eq!(key_to_message(KeyCode::Up, &Screen::Board), Some(Message::MoveUp));
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
        assert_eq!(key_to_message(KeyCode::Enter, &Screen::Board), Some(Message::OpenDetail));
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
        assert_eq!(key_to_message(KeyCode::F(1), &Screen::Board), Some(Message::ToggleHelp));
    }

    #[test]
    fn detail_q_and_esc_close_overlay() {
        assert_eq!(
            key_to_message(KeyCode::Char('q'), &Screen::Detail),
            Some(Message::CloseOverlay)
        );
        assert_eq!(key_to_message(KeyCode::Esc, &Screen::Detail), Some(Message::CloseOverlay));
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
        assert_eq!(key_to_message(KeyCode::Esc, &Screen::Help), Some(Message::CloseOverlay));
    }
}
