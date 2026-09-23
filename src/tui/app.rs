use std::time::Instant;

use chrono::Utc;

use crate::domain_types::{Ticket, TicketStatus};

// ── Model ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Board,
    Detail,
    Help,
    /// The `/` filter prompt is open; every printable key is literal text.
    Filter,
}

pub struct App {
    pub tickets: Vec<Ticket>,
    /// Status for each kanban column, e.g. [Todo, InProgress, Done]
    pub columns: Vec<TicketStatus>,
    /// Focused column index
    pub col: usize,
    /// Focused row within the focused column
    pub row: usize,
    pub screen: Screen,
    /// Transient status bar message with the time it was set.
    pub flash: Option<(String, Instant)>,
    /// Live filter query, matched case-insensitively against id and title.
    /// Session-only: never persisted, cleared on restart.
    pub filter: String,
}

impl App {
    pub fn new(tickets: Vec<Ticket>, columns: Vec<TicketStatus>) -> Self {
        App {
            tickets,
            columns,
            col: 0,
            row: 0,
            screen: Screen::Board,
            flash: None,
            filter: String::new(),
        }
    }

    /// Indices into `self.tickets` for tickets belonging to column `col`,
    /// narrowed by the current filter query (if any).
    pub fn col_indices(&self, col: usize) -> Vec<usize> {
        let status = &self.columns[col];
        let query = self.filter.to_lowercase();
        self.tickets
            .iter()
            .enumerate()
            .filter(|(_, t)| t.front_matter.status == *status)
            .filter(|(_, t)| ticket_matches_query(t, &query))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn focused_ticket_index(&self) -> Option<usize> {
        self.col_indices(self.col).get(self.row).copied()
    }

    pub fn focused_ticket(&self) -> Option<&Ticket> {
        self.focused_ticket_index().map(|i| &self.tickets[i])
    }

    /// Replace tickets (e.g. after an external edit) and clamp cursor to valid position.
    pub fn set_tickets(&mut self, tickets: Vec<Ticket>) {
        self.tickets = tickets;
        self.clamp_row();
    }

    // ── private state mutators (called by update) ─────────────────────────────

    pub(super) fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
            self.clamp_row();
        }
    }

    pub(super) fn move_right(&mut self) {
        if self.col + 1 < self.columns.len() {
            self.col += 1;
            self.clamp_row();
        }
    }

    pub(super) fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
        }
    }

    pub(super) fn move_down(&mut self) {
        let count = self.col_indices(self.col).len();
        if count > 0 && self.row < count - 1 {
            self.row += 1;
        }
    }

    /// Move the focused ticket left one column. Returns true if moved.
    pub(super) fn move_ticket_left(&mut self) -> bool {
        if self.col == 0 {
            return false;
        }
        self.move_ticket_to(self.col - 1)
    }

    /// Move the focused ticket right one column. Returns true if moved.
    pub(super) fn move_ticket_right(&mut self) -> bool {
        if self.col + 1 >= self.columns.len() {
            return false;
        }
        self.move_ticket_to(self.col + 1)
    }

    fn move_ticket_to(&mut self, target_col: usize) -> bool {
        let Some(idx) = self.focused_ticket_index() else {
            return false;
        };
        let ticket_id = self.tickets[idx].front_matter.id.to_string();
        let new_status = self.columns[target_col].clone();

        self.tickets[idx].front_matter.status = new_status;
        self.tickets[idx].front_matter.updated_at = Utc::now();
        self.col = target_col;

        // Keep focus on the moved ticket in its new column.
        self.row = self
            .col_indices(target_col)
            .iter()
            .position(|&i| self.tickets[i].front_matter.id.to_string() == ticket_id)
            .unwrap_or(0);
        true
    }

    fn clamp_row(&mut self) {
        let len = self.col_indices(self.col).len();
        if len == 0 {
            self.row = 0;
        } else {
            self.row = self.row.min(len - 1);
        }
    }

    // ── filter mutators (called by update) ─────────────────────────────────

    pub(super) fn open_filter(&mut self) {
        self.screen = Screen::Filter;
    }

    /// Append a character to the live filter query and re-clamp focus.
    pub(super) fn push_filter_char(&mut self, c: char) {
        self.filter.push(c);
        self.clamp_row();
    }

    /// Remove the last character from the live filter query and re-clamp focus.
    pub(super) fn pop_filter_char(&mut self) {
        self.filter.pop();
        self.clamp_row();
    }

    /// `Enter` from the prompt: keep the query, return to normal navigation.
    pub(super) fn commit_filter(&mut self) {
        self.screen = Screen::Board;
    }

    /// `Esc` from the prompt: discard the query entirely, even one that was
    /// already active before the prompt was opened.
    pub(super) fn cancel_filter(&mut self) {
        self.filter.clear();
        self.screen = Screen::Board;
        self.clamp_row();
    }

    /// `Esc` from the board with an active filter: clear it. No-op otherwise.
    pub(super) fn clear_filter(&mut self) {
        if !self.filter.is_empty() {
            self.filter.clear();
            self.clamp_row();
        }
    }
}

/// A ticket matches an already-lower-cased query if it contains it in either
/// the id or the title (case-insensitive). An empty query matches everything.
fn ticket_matches_query(ticket: &Ticket, query_lower: &str) -> bool {
    if query_lower.is_empty() {
        return true;
    }
    let id = ticket.front_matter.id.to_string().to_lowercase();
    if id.contains(query_lower) {
        return true;
    }
    let title = ticket.front_matter.title.to_string().to_lowercase();
    title.contains(query_lower)
}

// ── Message ───────────────────────────────────────────────────────────────────

/// All user-driven events, expressed as domain actions (not raw key codes).
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveTicketLeft,
    MoveTicketRight,
    OpenDetail,
    CloseOverlay,
    OpenEditor,
    NewTicket,
    ToggleHelp,
    CopyId,
    Quit,
    /// `/` from the board: open the filter prompt.
    OpenFilter,
    /// A literal character typed while the filter prompt is open.
    FilterInput(char),
    /// `Backspace` while the filter prompt is open.
    FilterBackspace,
    /// `Enter` from the filter prompt: keep the query, return to the board.
    FilterCommit,
    /// `Esc` from the filter prompt: discard the query, return to the board.
    FilterCancel,
    /// `Esc` from the board: clear an active filter (no-op if none).
    ClearFilter,
}

// ── Cmd ───────────────────────────────────────────────────────────────────────

/// Side-effect instructions returned by `update`. The runtime executes these.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    None,
    Quit,
    /// Persist the focused ticket to disk (and auto-commit if configured).
    SaveFocused,
    /// Suspend TUI, open the focused ticket in $EDITOR, then resume.
    OpenEditor,
    /// Create a draft ticket with the current column's status, then open in $EDITOR.
    CreateAndEdit,
    /// Copy the given ticket ID string to the system clipboard.
    CopyId(String),
}

// ── update ────────────────────────────────────────────────────────────────────

/// Pure state transition. Mutates `app` and returns any side effect to execute.
pub fn update(app: &mut App, msg: Message) -> Cmd {
    match app.screen {
        Screen::Help => {
            // Any message dismisses the overlay.
            app.screen = Screen::Board;
            Cmd::None
        }
        Screen::Detail => match msg {
            Message::CloseOverlay | Message::Quit => {
                app.screen = Screen::Board;
                Cmd::None
            }
            Message::OpenEditor => Cmd::OpenEditor,
            _ => Cmd::None,
        },
        Screen::Filter => match msg {
            Message::FilterInput(c) => {
                app.push_filter_char(c);
                Cmd::None
            }
            Message::FilterBackspace => {
                app.pop_filter_char();
                Cmd::None
            }
            Message::FilterCommit => {
                app.commit_filter();
                Cmd::None
            }
            Message::FilterCancel => {
                app.cancel_filter();
                Cmd::None
            }
            _ => Cmd::None,
        },
        Screen::Board => match msg {
            Message::Quit => Cmd::Quit,
            Message::MoveLeft => {
                app.move_left();
                Cmd::None
            }
            Message::MoveRight => {
                app.move_right();
                Cmd::None
            }
            Message::MoveUp => {
                app.move_up();
                Cmd::None
            }
            Message::MoveDown => {
                app.move_down();
                Cmd::None
            }
            Message::MoveTicketLeft => {
                if app.move_ticket_left() {
                    Cmd::SaveFocused
                } else {
                    Cmd::None
                }
            }
            Message::MoveTicketRight => {
                if app.move_ticket_right() {
                    Cmd::SaveFocused
                } else {
                    Cmd::None
                }
            }
            Message::OpenDetail => {
                if app.focused_ticket().is_some() {
                    app.screen = Screen::Detail;
                }
                Cmd::None
            }
            Message::OpenEditor => Cmd::OpenEditor,
            Message::NewTicket => Cmd::CreateAndEdit,
            Message::ToggleHelp => {
                app.screen = Screen::Help;
                Cmd::None
            }
            Message::CloseOverlay => Cmd::None,
            Message::CopyId => {
                if let Some(t) = app.focused_ticket() {
                    Cmd::CopyId(t.front_matter.id.to_string())
                } else {
                    Cmd::None
                }
            }
            Message::OpenFilter => {
                app.open_filter();
                Cmd::None
            }
            Message::ClearFilter => {
                app.clear_filter();
                Cmd::None
            }
            // Filter-editing messages only ever arrive on Screen::Filter.
            _ => Cmd::None,
        },
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    use crate::domain_types::{FrontMatter, TicketId, TicketStatus, TicketType, Title};

    /// Fixed, obviously-fictional timestamp for test fixtures, so rendered
    /// output never depends on the wall clock. Constructed deterministically
    /// rather than parsed, so it can never fail at runtime.
    fn fixed_timestamp() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap()
    }

    fn make_ticket(id: &str, title: &str, status: TicketStatus) -> Ticket {
        let now = fixed_timestamp();
        Ticket {
            front_matter: FrontMatter {
                id: TicketId::from(id.to_string()),
                title: title.parse::<Title>().unwrap(),
                r#type: TicketType::Task,
                status,
                tags: vec![],
                parent: None,
                blocked_by: vec![],
                created_at: now,
                updated_at: now,
            },
            body: String::new(),
        }
    }

    fn default_columns() -> Vec<TicketStatus> {
        vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ]
    }

    // ── col_indices / grouping ─────────────────────────────────────────────

    #[test]
    fn tickets_grouped_into_correct_columns() {
        let tickets = vec![
            make_ticket("a", "Fix bug", TicketStatus::Todo),
            make_ticket("b", "Add feature", TicketStatus::InProgress),
            make_ticket("c", "Old task", TicketStatus::Done),
        ];
        let app = App::new(tickets, default_columns());
        assert_eq!(app.col_indices(0).len(), 1);
        assert_eq!(app.col_indices(1).len(), 1);
        assert_eq!(app.col_indices(2).len(), 1);
    }

    #[test]
    fn tickets_not_in_any_column_are_excluded() {
        let tickets = vec![
            make_ticket("a", "Draft ticket", TicketStatus::Draft),
            make_ticket("b", "Todo ticket", TicketStatus::Todo),
        ];
        let app = App::new(tickets, default_columns());
        assert_eq!(app.col_indices(0).len(), 1); // only todo
        assert_eq!(app.col_indices(1).len(), 0);
        assert_eq!(app.col_indices(2).len(), 0);
    }

    #[test]
    fn multiple_tickets_in_same_column() {
        let tickets = vec![
            make_ticket("a", "Task one", TicketStatus::Todo),
            make_ticket("b", "Task two", TicketStatus::Todo),
            make_ticket("c", "Task three", TicketStatus::Done),
        ];
        let app = App::new(tickets, default_columns());
        assert_eq!(app.col_indices(0).len(), 2);
        assert_eq!(app.col_indices(2).len(), 1);
    }

    // ── focused_ticket ─────────────────────────────────────────────────────

    #[test]
    fn focused_ticket_returns_first_ticket_in_first_column() {
        let tickets = vec![
            make_ticket("a", "First", TicketStatus::Todo),
            make_ticket("b", "Second", TicketStatus::Done),
        ];
        let app = App::new(tickets, default_columns());
        let focused = app.focused_ticket().unwrap();
        assert_eq!(focused.front_matter.id.to_string(), "a");
    }

    #[test]
    fn focused_ticket_returns_none_when_column_is_empty() {
        let tickets = vec![make_ticket("a", "Done task", TicketStatus::Done)];
        let app = App::new(tickets, default_columns());
        // col 0 = "todo" is empty
        assert!(app.focused_ticket().is_none());
    }

    // ── navigation via update ──────────────────────────────────────────────

    #[test]
    fn update_move_right_advances_column() {
        let tickets = vec![
            make_ticket("a", "Todo task", TicketStatus::Todo),
            make_ticket("b", "Done task", TicketStatus::Done),
        ];
        let mut app = App::new(tickets, default_columns());
        let cmd = update(&mut app, Message::MoveRight);
        assert_eq!(cmd, Cmd::None);
        assert_eq!(app.col, 1);
    }

    #[test]
    fn update_move_right_no_op_at_last_column() {
        let mut app = App::new(vec![], default_columns());
        app.col = 2;
        update(&mut app, Message::MoveRight);
        assert_eq!(app.col, 2);
    }

    #[test]
    fn update_move_left_no_op_at_first_column() {
        let mut app = App::new(vec![], default_columns());
        update(&mut app, Message::MoveLeft);
        assert_eq!(app.col, 0);
    }

    #[test]
    fn update_move_down_advances_row() {
        let tickets = vec![
            make_ticket("a", "First", TicketStatus::Todo),
            make_ticket("b", "Second", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, default_columns());
        update(&mut app, Message::MoveDown);
        assert_eq!(app.row, 1);
    }

    #[test]
    fn update_move_down_no_op_at_last_row() {
        let tickets = vec![make_ticket("a", "Only", TicketStatus::Todo)];
        let mut app = App::new(tickets, default_columns());
        update(&mut app, Message::MoveDown);
        assert_eq!(app.row, 0);
    }

    #[test]
    fn update_move_right_clamps_row_in_new_column() {
        let tickets = vec![
            make_ticket("a", "First", TicketStatus::Todo),
            make_ticket("b", "Second", TicketStatus::Todo),
            make_ticket("c", "Only", TicketStatus::InProgress),
        ];
        let mut app = App::new(tickets, default_columns());
        app.row = 1;
        update(&mut app, Message::MoveRight);
        assert_eq!(app.col, 1);
        assert_eq!(app.row, 0);
    }

    // ── move ticket ────────────────────────────────────────────────────────

    #[test]
    fn update_move_ticket_right_changes_status_and_returns_save() {
        let tickets = vec![make_ticket("a", "Fix bug", TicketStatus::Todo)];
        let mut app = App::new(tickets, default_columns());
        let cmd = update(&mut app, Message::MoveTicketRight);
        assert_eq!(cmd, Cmd::SaveFocused);
        assert_eq!(app.tickets[0].front_matter.status, TicketStatus::InProgress);
    }

    #[test]
    fn update_move_ticket_right_keeps_focus_on_moved_ticket() {
        let tickets = vec![make_ticket("a", "Fix bug", TicketStatus::Todo)];
        let mut app = App::new(tickets, default_columns());
        update(&mut app, Message::MoveTicketRight);
        assert_eq!(app.col, 1);
        assert_eq!(
            app.focused_ticket().unwrap().front_matter.id.to_string(),
            "a"
        );
    }

    #[test]
    fn update_move_ticket_right_at_last_col_returns_none() {
        let tickets = vec![make_ticket("a", "Done", TicketStatus::Done)];
        let mut app = App::new(tickets, default_columns());
        app.col = 2;
        let cmd = update(&mut app, Message::MoveTicketRight);
        assert_eq!(cmd, Cmd::None);
        assert_eq!(app.tickets[0].front_matter.status, TicketStatus::Done);
    }

    #[test]
    fn update_move_ticket_left_changes_status_and_returns_save() {
        let tickets = vec![make_ticket("a", "In flight", TicketStatus::InProgress)];
        let mut app = App::new(tickets, default_columns());
        app.col = 1;
        let cmd = update(&mut app, Message::MoveTicketLeft);
        assert_eq!(cmd, Cmd::SaveFocused);
        assert_eq!(app.tickets[0].front_matter.status, TicketStatus::Todo);
    }

    #[test]
    fn update_move_ticket_left_at_leftmost_col_returns_none() {
        let tickets = vec![make_ticket("a", "Fix bug", TicketStatus::Todo)];
        let mut app = App::new(tickets, default_columns());
        let cmd = update(&mut app, Message::MoveTicketLeft);
        assert_eq!(cmd, Cmd::None);
    }

    #[test]
    fn update_move_ticket_right_no_focused_ticket_returns_none() {
        // col 0 = todo is empty
        let tickets = vec![make_ticket("a", "Done task", TicketStatus::Done)];
        let mut app = App::new(tickets, default_columns());
        let cmd = update(&mut app, Message::MoveTicketRight);
        assert_eq!(cmd, Cmd::None);
    }

    // ── screen transitions ─────────────────────────────────────────────────

    #[test]
    fn update_quit_returns_quit() {
        let mut app = App::new(vec![], default_columns());
        assert_eq!(update(&mut app, Message::Quit), Cmd::Quit);
    }

    #[test]
    fn update_open_detail_transitions_to_detail_screen() {
        let tickets = vec![make_ticket("a", "Fix bug", TicketStatus::Todo)];
        let mut app = App::new(tickets, default_columns());
        update(&mut app, Message::OpenDetail);
        assert_eq!(app.screen, Screen::Detail);
    }

    #[test]
    fn update_open_detail_no_op_when_column_empty() {
        let mut app = App::new(vec![], default_columns());
        update(&mut app, Message::OpenDetail);
        assert_eq!(app.screen, Screen::Board);
    }

    #[test]
    fn update_close_overlay_from_detail_returns_to_board() {
        let tickets = vec![make_ticket("a", "Fix bug", TicketStatus::Todo)];
        let mut app = App::new(tickets, default_columns());
        app.screen = Screen::Detail;
        update(&mut app, Message::CloseOverlay);
        assert_eq!(app.screen, Screen::Board);
    }

    #[test]
    fn update_toggle_help_shows_help_screen() {
        let mut app = App::new(vec![], default_columns());
        update(&mut app, Message::ToggleHelp);
        assert_eq!(app.screen, Screen::Help);
    }

    #[test]
    fn update_any_message_from_help_returns_to_board() {
        let mut app = App::new(vec![], default_columns());
        app.screen = Screen::Help;
        update(&mut app, Message::MoveLeft);
        assert_eq!(app.screen, Screen::Board);
    }

    #[test]
    fn update_open_editor_from_board_returns_open_editor_cmd() {
        let mut app = App::new(vec![], default_columns());
        assert_eq!(update(&mut app, Message::OpenEditor), Cmd::OpenEditor);
    }

    #[test]
    fn update_new_ticket_returns_create_and_edit_cmd() {
        let mut app = App::new(vec![], default_columns());
        assert_eq!(update(&mut app, Message::NewTicket), Cmd::CreateAndEdit);
    }

    #[test]
    fn update_copy_id_returns_cmd_with_focused_id() {
        let tickets = vec![make_ticket("abc123", "Fix bug", TicketStatus::Todo)];
        let mut app = App::new(tickets, default_columns());
        let cmd = update(&mut app, Message::CopyId);
        assert_eq!(cmd, Cmd::CopyId("abc123".to_string()));
    }

    #[test]
    fn update_copy_id_no_op_when_no_focused_ticket() {
        let mut app = App::new(vec![], default_columns());
        let cmd = update(&mut app, Message::CopyId);
        assert_eq!(cmd, Cmd::None);
    }

    // ── set_tickets ───────────────────────────────────────────────────────

    #[test]
    fn set_tickets_clamps_row_when_column_shrinks() {
        let tickets = vec![
            make_ticket("a", "First", TicketStatus::Todo),
            make_ticket("b", "Second", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, default_columns());
        app.row = 1;
        app.set_tickets(vec![make_ticket("a", "First", TicketStatus::Todo)]);
        assert_eq!(app.row, 0);
    }

    // ── filter ────────────────────────────────────────────────────────────

    #[test]
    fn col_indices_returns_only_tickets_matching_the_query() {
        let tickets = vec![
            make_ticket("a", "Fix login bug", TicketStatus::Todo),
            make_ticket("b", "Add search", TicketStatus::Todo),
            make_ticket("c", "Fix payments", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, default_columns());
        app.filter = "fix".to_string();
        let indices = app.col_indices(0);
        assert_eq!(indices.len(), 2);
        for i in indices {
            assert!(
                app.tickets[i]
                    .front_matter
                    .title
                    .to_string()
                    .contains("Fix")
            );
        }
    }

    #[test]
    fn col_indices_matches_id_or_title_independently() {
        let tickets = vec![
            make_ticket("abc999", "Nothing special", TicketStatus::Todo),
            make_ticket("xyz111", "abc widget", TicketStatus::Todo),
            make_ticket("qqq222", "Unrelated", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, default_columns());
        app.filter = "abc".to_string();
        let indices = app.col_indices(0);
        let matched_ids: Vec<String> = indices
            .iter()
            .map(|&i| app.tickets[i].front_matter.id.to_string())
            .collect();
        // Matches via id (abc999) and via title (xyz111 has "abc widget"),
        // but not the unrelated third ticket.
        assert!(matched_ids.contains(&"abc999".to_string()));
        assert!(matched_ids.contains(&"xyz111".to_string()));
        assert_eq!(matched_ids.len(), 2);
    }

    #[test]
    fn col_indices_query_is_case_insensitive() {
        let tickets = vec![
            make_ticket("ABC123", "Fix Login Bug", TicketStatus::Todo),
            make_ticket("def456", "Unrelated", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, default_columns());

        // Lowercase query matches mixed-case title.
        app.filter = "login".to_string();
        assert_eq!(app.col_indices(0).len(), 1);

        // Uppercase query matches lowercase id.
        app.filter = "ABC123".to_string();
        assert_eq!(app.col_indices(0).len(), 1);
    }

    #[test]
    fn row_is_clamped_when_the_filter_shrinks_the_focused_column() {
        let tickets = vec![
            make_ticket("a", "Fix login", TicketStatus::Todo),
            make_ticket("b", "Fix search", TicketStatus::Todo),
            make_ticket("c", "Unrelated", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, default_columns());
        app.row = 2; // focused on "Unrelated"
        app.push_filter_char('f');
        app.push_filter_char('i');
        app.push_filter_char('x');
        // Only 2 tickets match "fix" now; row must be clamped into range.
        assert_eq!(app.col_indices(app.col).len(), 2);
        assert_eq!(app.row, 1);
    }

    #[test]
    fn focused_ticket_none_and_no_key_panics_when_nothing_matches() {
        let tickets = vec![
            make_ticket("a", "Fix login", TicketStatus::Todo),
            make_ticket("b", "Fix search", TicketStatus::InProgress),
        ];
        let mut app = App::new(tickets, default_columns());
        app.row = 1;
        app.filter = "nonexistent-query".to_string();
        app.clamp_row();

        assert!(app.focused_ticket().is_none());
        assert_eq!(app.col_indices(0).len(), 0);

        // Enter/y/e all route through focused_ticket() / focused_ticket_index()
        // and must be safe no-ops; H/L move must also be safe no-ops.
        assert_eq!(update(&mut app, Message::OpenDetail), Cmd::None);
        assert_eq!(app.screen, Screen::Board);
        assert_eq!(update(&mut app, Message::CopyId), Cmd::None);
        assert_eq!(update(&mut app, Message::OpenEditor), Cmd::OpenEditor);
        assert_eq!(update(&mut app, Message::MoveTicketLeft), Cmd::None);
        assert_eq!(update(&mut app, Message::MoveTicketRight), Cmd::None);
    }

    // ── filter mode transitions via update ──────────────────────────────

    #[test]
    fn update_open_filter_switches_to_filter_screen() {
        let mut app = App::new(vec![], default_columns());
        update(&mut app, Message::OpenFilter);
        assert_eq!(app.screen, Screen::Filter);
    }

    #[test]
    fn update_filter_input_appends_live_and_narrows_board() {
        let tickets = vec![
            make_ticket("a", "Fix login", TicketStatus::Todo),
            make_ticket("b", "Unrelated", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, default_columns());
        app.screen = Screen::Filter;
        update(&mut app, Message::FilterInput('f'));
        update(&mut app, Message::FilterInput('i'));
        update(&mut app, Message::FilterInput('x'));
        assert_eq!(app.filter, "fix");
        assert_eq!(app.col_indices(0).len(), 1);
    }

    #[test]
    fn update_filter_backspace_removes_last_character() {
        let mut app = App::new(vec![], default_columns());
        app.screen = Screen::Filter;
        app.filter = "fix".to_string();
        update(&mut app, Message::FilterBackspace);
        assert_eq!(app.filter, "fi");
    }

    #[test]
    fn update_filter_commit_keeps_query_and_returns_to_board() {
        let mut app = App::new(vec![], default_columns());
        app.screen = Screen::Filter;
        app.filter = "fix".to_string();
        update(&mut app, Message::FilterCommit);
        assert_eq!(app.screen, Screen::Board);
        assert_eq!(app.filter, "fix");
    }

    #[test]
    fn update_filter_cancel_clears_query_even_if_active_before_prompt_opened() {
        let mut app = App::new(vec![], default_columns());
        app.filter = "already active".to_string();
        app.screen = Screen::Filter;
        update(&mut app, Message::FilterCancel);
        assert_eq!(app.screen, Screen::Board);
        assert_eq!(app.filter, "");
    }

    #[test]
    fn update_clear_filter_from_board_clears_active_filter() {
        let mut app = App::new(vec![], default_columns());
        app.filter = "fix".to_string();
        update(&mut app, Message::ClearFilter);
        assert_eq!(app.filter, "");
    }

    #[test]
    fn update_clear_filter_from_board_is_no_op_when_no_filter_active() {
        let mut app = App::new(vec![], default_columns());
        let cmd = update(&mut app, Message::ClearFilter);
        assert_eq!(cmd, Cmd::None);
        assert_eq!(app.filter, "");
    }
}
