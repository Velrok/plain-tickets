use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::domain_types::{FrontMatter, TicketType};

use super::app::{App, Screen};

/// Entry point — renders the correct screen (view function in ELM terms).
pub fn view(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Board => draw_board(f, app),
        Screen::Detail => {
            draw_board(f, app);
            draw_detail(f, app);
        }
        Screen::Help => {
            draw_board(f, app);
            draw_help(f);
        }
    }
}

// ── board ─────────────────────────────────────────────────────────────────────

fn draw_board(f: &mut Frame, app: &App) {
    if app.columns.is_empty() {
        return;
    }
    let area = f.area();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let board_area = vertical[0];
    let footer_area = vertical[1];

    // Columns
    let col_count = app.columns.len() as u32;
    let constraints: Vec<Constraint> =
        app.columns.iter().map(|_| Constraint::Ratio(1, col_count)).collect();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(board_area);

    for (col_idx, col_name) in app.columns.iter().enumerate() {
        let is_focused = col_idx == app.col;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(col_name.as_str())
            .borders(Borders::ALL)
            .border_style(border_style);

        let indices = app.col_indices(col_idx);
        let items: Vec<ListItem> = indices
            .iter()
            .map(|&ti| ticket_list_item(&app.tickets[ti].front_matter))
            .collect();

        let mut list_state = ListState::default();
        if is_focused && !indices.is_empty() {
            list_state.select(Some(app.row.min(indices.len().saturating_sub(1))));
        }

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(list, chunks[col_idx], &mut list_state);
    }

    // Footer
    let footer = Paragraph::new(
        "h/l columns  j/k tickets  H/L move  Enter detail  e edit  n new  ? help  q quit",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, footer_area);
}

// ── detail view ───────────────────────────────────────────────────────────────

fn draw_detail(f: &mut Frame, app: &App) {
    let Some(ticket) = app.focused_ticket() else {
        return;
    };
    let area = centered_rect(80, 80, f.area());
    let fm = &ticket.front_matter;

    let mut lines = vec![
        Line::from(format!("ID:      {}", fm.id)),
        Line::from(format!("Title:   {}", fm.title)),
        Line::from(format!("Status:  {}", fm.status)),
        Line::from(format!("Type:    {}", fm.r#type)),
    ];

    if !fm.tags.is_empty() {
        let tags: Vec<String> = fm.tags.iter().map(|t| t.to_string()).collect();
        lines.push(Line::from(format!("Tags:    {}", tags.join(", "))));
    }
    if let Some(ref p) = fm.parent {
        lines.push(Line::from(format!("Parent:  {}", p)));
    }
    if !fm.blocked_by.is_empty() {
        let ids: Vec<String> = fm.blocked_by.iter().map(|t| t.to_string()).collect();
        lines.push(Line::from(format!("Blocked: {}", ids.join(", "))));
    }
    lines.push(Line::from(format!("Created: {}", fm.created_at.format("%Y-%m-%d"))));
    lines.push(Line::from(format!("Updated: {}", fm.updated_at.format("%Y-%m-%d"))));

    if !ticket.body.is_empty() {
        lines.push(Line::from(""));
        for line in ticket.body.lines() {
            lines.push(Line::from(line.to_string()));
        }
    }

    let block = Block::default()
        .title("  Detail    e edit    q/Esc back  ")
        .borders(Borders::ALL);
    let para = Paragraph::new(Text::from(lines)).block(block).wrap(Wrap { trim: false });

    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

// ── help overlay ──────────────────────────────────────────────────────────────

fn draw_help(f: &mut Frame) {
    let area = centered_rect(46, 60, f.area());

    let lines = vec![
        Line::from("  Keybindings"),
        Line::from(""),
        Line::from("  h / ←      move focus left"),
        Line::from("  l / →      move focus right"),
        Line::from("  j / ↓      move focus down"),
        Line::from("  k / ↑      move focus up"),
        Line::from("  H          move ticket left"),
        Line::from("  L          move ticket right"),
        Line::from("  Enter/Spc  open detail view"),
        Line::from("  e          open in editor"),
        Line::from("  n          new ticket"),
        Line::from("  ? / F1     show this help"),
        Line::from("  q          quit"),
        Line::from(""),
        Line::from("  [any key]  dismiss"),
    ];

    let block = Block::default().title("  Help  ").borders(Borders::ALL);
    let para = Paragraph::new(Text::from(lines)).block(block);

    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

// ── ticket card ───────────────────────────────────────────────────────────────

fn ticket_list_item(fm: &FrontMatter) -> ListItem<'static> {
    let mut spans = vec![type_span(&fm.r#type), Span::raw(fm.title.to_string())];
    for tag in &fm.tags {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("#{}", tag),
            Style::default().fg(tag_color(tag.to_string().as_str())),
        ));
    }
    ListItem::new(Line::from(spans))
}

fn type_span(t: &TicketType) -> Span<'static> {
    // Emoji are 2-wide; ratatui measures via unicode-width so layout is correct.
    match t {
        TicketType::Epic  => Span::raw("🌟 "),
        TicketType::Story => Span::raw("📖 "),
        TicketType::Task  => Span::raw("📋 "),
        TicketType::Bug   => Span::raw("🐛 "),
    }
}

/// Deterministic colour from the tag name via a simple hash.
fn tag_color(tag: &str) -> Color {
    // FNV-1a hash for stable, fast hashing without std HashMap
    let mut hash: u64 = 14695981039346656037;
    for b in tag.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    // Pick from a curated palette that reads well on dark terminals
    const PALETTE: [Color; 8] = [
        Color::Cyan,
        Color::Green,
        Color::Yellow,
        Color::Magenta,
        Color::LightBlue,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightMagenta,
    ];
    PALETTE[(hash % PALETTE.len() as u64) as usize]
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// ── snapshot tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ratatui::{Terminal, backend::TestBackend};

    use crate::domain_types::{FrontMatter, Ticket, TicketId, TicketStatus, TicketType, Title};
    use crate::tui::app::App;

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

    fn render_to_string(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| view(f, app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn board_renders_three_columns() {
        let columns = vec!["todo".to_string(), "in-progress".to_string(), "done".to_string()];
        let tickets = vec![
            make_ticket("aaa111", "Fix login bug", TicketStatus::Todo),
            make_ticket("bbb222", "Add search", TicketStatus::InProgress),
        ];
        let app = App::new(tickets, columns);
        let output = render_to_string(&app, 80, 20);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn detail_view_renders_ticket_fields() {
        let columns = vec!["todo".to_string(), "in-progress".to_string(), "done".to_string()];
        let tickets = vec![make_ticket("abc123", "Fix login bug", TicketStatus::Todo)];
        let mut app = App::new(tickets, columns);
        app.screen = Screen::Detail;
        let output = render_to_string(&app, 80, 24);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn help_overlay_renders_keybindings() {
        let columns = vec!["todo".to_string(), "in-progress".to_string(), "done".to_string()];
        let mut app = App::new(vec![], columns);
        app.screen = Screen::Help;
        let output = render_to_string(&app, 80, 24);
        insta::assert_snapshot!(output);
    }
}
