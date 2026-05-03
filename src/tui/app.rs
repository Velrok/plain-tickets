use chrono::Utc;
use clap::ValueEnum as _;

use crate::domain_types::{Ticket, TicketStatus};

// ── Model ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Board,
    Detail,
    Help,
}

pub struct App {
    pub tickets: Vec<Ticket>,
    /// Status strings for each kanban column, e.g. ["todo", "in-progress", "done"]
    pub columns: Vec<String>,
    /// Focused column index
    pub col: usize,
    /// Focused row within the focused column
    pub row: usize,
    pub screen: Screen,
}

impl App {
    pub fn new(tickets: Vec<Ticket>, columns: Vec<String>) -> Self {
        App { tickets, columns, col: 0, row: 0, screen: Screen::Board }
    }

    /// Indices into `self.tickets` for tickets belonging to column `col`.
    pub fn col_indices(&self, col: usize) -> Vec<usize> {
        let col_name = &self.columns[col];
        self.tickets
            .iter()
            .enumerate()
            .filter(|(_, t)| t.front_matter.status.to_string() == *col_name)
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
        let new_status_str = self.columns[target_col].clone();
        let Ok(new_status) = TicketStatus::from_str(&new_status_str, true) else {
            return false;
        };

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
    Quit,
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
                if app.move_ticket_left() { Cmd::SaveFocused } else { Cmd::None }
            }
            Message::MoveTicketRight => {
                if app.move_ticket_right() { Cmd::SaveFocused } else { Cmd::None }
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
        },
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::domain_types::{FrontMatter, TicketId, TicketStatus, TicketType, Title};

    fn make_ticket(id: &str, title: &str, status: TicketStatus) -> Ticket {
        let now = Utc::now();
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

    fn default_columns() -> Vec<String> {
        vec!["todo".to_string(), "in-progress".to_string(), "done".to_string()]
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
        assert_eq!(app.focused_ticket().unwrap().front_matter.id.to_string(), "a");
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
}
