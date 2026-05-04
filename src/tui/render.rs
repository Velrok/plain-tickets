use std::time::Duration;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::domain_types::TicketType;

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
        let chunk = chunks[col_idx];
        let is_focused = col_idx == app.col;

        let cards_area = Rect { y: chunk.y + 1, height: chunk.height.saturating_sub(1), ..chunk };
        let indices = app.col_indices(col_idx);

        // Compute scroll offset: keep focused card visible in focused column.
        let rough_cap = ((cards_area.height / 3) as usize).max(1);
        let scroll = if is_focused {
            app.row.saturating_sub(rough_cap.saturating_sub(1))
        } else {
            0
        };
        let (rendered, total) = draw_cards(f, app, col_idx, &indices, cards_area, scroll);

        let has_above = scroll > 0;
        let has_below = scroll + rendered < total;
        let suffix = match (has_above, has_below) {
            (true, true) => " ↕",
            (true, false) => " ↑",
            (false, true) => " ↓",
            (false, false) => "",
        };

        let label_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let label_area = Rect { height: 1, ..chunk };
        let label_text = format!("{}{}", col_name, suffix);
        f.render_widget(Paragraph::new(label_text).style(label_style), label_area);
    }

    // Footer: show flash message for 2 s, then fall back to hint.
    const FLASH_DURATION: Duration = Duration::from_secs(2);
    let footer_text = app
        .flash
        .as_ref()
        .filter(|(_, t)| t.elapsed() < FLASH_DURATION)
        .map(|(msg, _)| msg.as_str())
        .unwrap_or("h/l columns  j/k tickets  H/L move  y copy id  Enter detail  e edit  n new  ? help  q quit");
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
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
        Line::from("  y          copy ticket id"),
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

// ── ticket cards ──────────────────────────────────────────────────────────────

fn draw_cards(
    f: &mut Frame,
    app: &App,
    col_idx: usize,
    indices: &[usize],
    inner: Rect,
    scroll: usize,
) -> (usize, usize) {
    let total = indices.len();
    let scroll = scroll.min(total);
    let visible = &indices[scroll..];

    let is_focused_col = col_idx == app.col;
    let card_width = inner.width;
    let text_width = card_width.saturating_sub(2) as usize; // minus left/right border

    let mut y = inner.y;
    let max_y = inner.y + inner.height;
    let mut rendered = 0;

    for (idx_in_visible, &ti) in visible.iter().enumerate() {
        let row_idx = idx_in_visible + scroll; // actual row index in column
        let fm = &app.tickets[ti].front_matter;
        let title_lines = wrap_text(&fm.title.to_string(), text_width);
        let card_height = 2 + title_lines.len() as u16; // top border + content + bottom border

        if y + card_height > max_y {
            break;
        }

        let card_area = Rect { x: inner.x, y, width: card_width, height: card_height };

        let is_focused = is_focused_col && row_idx == app.row;
        let border_style = if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let header = Line::from(vec![type_span(&fm.r#type), Span::raw(fm.id.to_string())]);
        let mut block = Block::default()
            .title_top(header)
            .borders(Borders::ALL)
            .border_style(border_style);

        if !fm.tags.is_empty() {
            let mut tag_spans: Vec<Span> = Vec::new();
            for tag in &fm.tags {
                tag_spans.push(Span::raw(" "));
                tag_spans.push(Span::styled(
                    format!("#{}", tag),
                    Style::default().fg(tag_color(tag.to_string().as_str())),
                ));
            }
            tag_spans.push(Span::raw(" "));
            block = block.title_bottom(Line::from(tag_spans).right_aligned());
        }

        let content: Vec<Line> =
            title_lines.into_iter().map(|l| Line::from(l)).collect();
        let para = Paragraph::new(Text::from(content)).block(block);
        f.render_widget(para, card_area);

        y += card_height;
        rendered += 1;
    }

    (rendered, total)
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn type_span(t: &TicketType) -> Span<'static> {
    // Emoji are 2-wide; ratatui measures via unicode-width so layout is correct.
    match t {
        TicketType::Epic  => Span::raw("🌟"),
        TicketType::Story => Span::raw("📖"),
        TicketType::Task  => Span::raw("📋"),
        TicketType::Bug   => Span::raw("🐛"),
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

    fn make_ticket_with_tags(id: &str, title: &str, status: TicketStatus, tags: &[&str]) -> Ticket {
        let now = Utc::now();
        Ticket {
            front_matter: FrontMatter {
                id: TicketId::from(id.to_string()),
                title: title.parse::<Title>().unwrap(),
                r#type: TicketType::Task,
                status,
                tags: tags.iter().map(|t| t.parse().unwrap()).collect(),
                parent: None,
                blocked_by: vec![],
                created_at: now,
                updated_at: now,
            },
            body: String::new(),
        }
    }

    // ── scroll ─────────────────────────────────────────────────────────────

    #[test]
    fn board_focused_card_visible_when_column_overflows() {
        // 3 cards, small height → only 2 fit; focus on 3rd → 3rd must be visible
        let columns = vec!["todo".to_string()];
        let tickets = vec![
            make_ticket("aaa111", "First ticket", TicketStatus::Todo),
            make_ticket("bbb222", "Second ticket", TicketStatus::Todo),
            make_ticket("ccc333", "Third ticket", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, columns);
        app.row = 2; // focus on 3rd card
        let output = render_to_string(&app, 30, 10);
        assert!(output.contains("ccc333"), "focused card not visible: {}", output);
        assert!(!output.contains("aaa111"), "first card should be scrolled off: {}", output);
    }

    #[test]
    fn board_shows_down_indicator_when_cards_clipped_below() {
        let columns = vec!["todo".to_string()];
        let tickets = vec![
            make_ticket("aaa111", "First ticket", TicketStatus::Todo),
            make_ticket("bbb222", "Second ticket", TicketStatus::Todo),
            make_ticket("ccc333", "Third ticket", TicketStatus::Todo),
        ];
        let app = App::new(tickets, columns); // row=0
        let output = render_to_string(&app, 30, 10);
        assert!(output.contains('↓'), "down indicator missing: {}", output);
    }

    #[test]
    fn board_shows_up_indicator_when_scrolled() {
        let columns = vec!["todo".to_string()];
        let tickets = vec![
            make_ticket("aaa111", "First ticket", TicketStatus::Todo),
            make_ticket("bbb222", "Second ticket", TicketStatus::Todo),
            make_ticket("ccc333", "Third ticket", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, columns);
        app.row = 2; // scroll past visible area
        let output = render_to_string(&app, 30, 10);
        assert!(output.contains('↑'), "up indicator missing: {}", output);
    }

    // ── existing snapshot tests ────────────────────────────────────────────

    #[test]
    fn board_card_renders_as_bordered_box_with_id_in_title() {
        let columns = vec!["todo".to_string()];
        let tickets = vec![make_ticket("abc123", "Fix login bug", TicketStatus::Todo)];
        let app = App::new(tickets, columns);
        let output = render_to_string(&app, 30, 10);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn board_card_footer_shows_tags() {
        let columns = vec!["todo".to_string()];
        let tickets = vec![make_ticket_with_tags("abc123", "Fix login bug", TicketStatus::Todo, &["tui", "config"])];
        let app = App::new(tickets, columns);
        let output = render_to_string(&app, 30, 10);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn board_card_wraps_long_title() {
        let columns = vec!["todo".to_string()];
        let tickets = vec![make_ticket("abc123", "Fix the login bug on the home page", TicketStatus::Todo)];
        let app = App::new(tickets, columns);
        let output = render_to_string(&app, 30, 12);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn board_cards_clip_when_column_overflows() {
        let columns = vec!["todo".to_string()];
        let tickets = vec![
            make_ticket("aaa111", "First ticket", TicketStatus::Todo),
            make_ticket("bbb222", "Second ticket", TicketStatus::Todo),
            make_ticket("ccc333", "Third ticket should not appear", TicketStatus::Todo),
        ];
        let app = App::new(tickets, columns);
        // height=10: outer column borders (2) + 3 cards × 3 lines = 11 → third card clips
        let output = render_to_string(&app, 30, 10);
        insta::assert_snapshot!(output);
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
