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
        // No overlay: the filter prompt replaces the footer hint in-place.
        Screen::Filter => draw_board(f, app),
    }
}

// ── board ─────────────────────────────────────────────────────────────────────

/// Default footer hint. Single-space separated (the previous double-space
/// grouping no longer fit at 80 columns once `/ filter` was added).
const DEFAULT_FOOTER_HINT: &str =
    "h/l col j/k row H/L mv y copy Enter view e edit n new / filter ? help q quit";

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
    let constraints: Vec<Constraint> = app
        .columns
        .iter()
        .map(|_| Constraint::Ratio(1, col_count))
        .collect();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(board_area);

    for (col_idx, col_name) in app.columns.iter().enumerate() {
        let chunk = chunks[col_idx];
        let is_focused = col_idx == app.col;

        let cards_area = Rect {
            y: chunk.y + 1,
            height: chunk.height.saturating_sub(1),
            ..chunk
        };
        let indices = app.col_indices(col_idx);

        // Compute scroll offset: keep focused card visible in focused column.
        let scroll = if is_focused {
            scroll_for_row(&indices, app, app.row, cards_area)
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

    // Footer: flash message (2 s) > filter prompt/indicator > default hint.
    const FLASH_DURATION: Duration = Duration::from_secs(2);
    let footer_text: String = if let Some(msg) = app
        .flash
        .as_ref()
        .filter(|(_, t)| t.elapsed() < FLASH_DURATION)
        .map(|(msg, _)| msg.clone())
    {
        msg
    } else if app.screen == Screen::Filter {
        format!("/{}", app.filter)
    } else if !app.filter.is_empty() {
        format!("filter: \"{}\"  (Esc clears)", app.filter)
    } else {
        DEFAULT_FOOTER_HINT.to_string()
    };
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
    lines.push(Line::from(format!(
        "Created: {}",
        fm.created_at.format("%Y-%m-%d")
    )));
    lines.push(Line::from(format!(
        "Updated: {}",
        fm.updated_at.format("%Y-%m-%d")
    )));

    let header_len = lines.len();

    let mut body_lines: Vec<Line> = Vec::new();
    if !ticket.body.is_empty() {
        body_lines.push(Line::from(""));
        for line in ticket.body.lines() {
            body_lines.push(Line::from(line.to_string()));
        }
    }

    // The body is user-authored and unbounded, so it can outgrow the box.
    // Never drop the overflow silently - truncate with an explicit marker
    // naming how much is hidden and how to see the rest. Real scrolling
    // was considered (see `4x7e81` implementation notes) and declined: it
    // needs new `App` state and keybindings, which is out of scope for a
    // render-only fix, and `e` already opens the ticket in an editor.
    let inner_height = area.height.saturating_sub(2) as usize; // top/bottom border
    if header_len + body_lines.len() > inner_height {
        let available_for_body = inner_height.saturating_sub(header_len).saturating_sub(1); // reserve one row for the marker itself
        let shown = available_for_body.min(body_lines.len());
        let hidden = body_lines.len() - shown;
        lines.extend(body_lines.into_iter().take(shown));
        lines.push(Line::styled(
            format!(
                "… {hidden} more line{} — press e to open",
                if hidden == 1 { "" } else { "s" }
            ),
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        lines.extend(body_lines);
    }

    let block = Block::default()
        .title("  Detail    e edit    q/Esc back  ")
        .borders(Borders::ALL);
    let para = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

// ── help overlay ──────────────────────────────────────────────────────────────

/// `(key, description)` pairs shown in the help overlay, in display order.
const HELP_KEYBINDINGS: &[(&str, &str)] = &[
    ("h / ←", "move focus left"),
    ("l / →", "move focus right"),
    ("j / ↓", "move focus down"),
    ("k / ↑", "move focus up"),
    ("H", "move ticket left"),
    ("L", "move ticket right"),
    ("Enter/Spc", "open detail view"),
    ("e", "open in editor"),
    ("n", "new ticket"),
    ("y", "copy ticket id"),
    ("/", "filter by id/title"),
    ("? / F1", "show this help"),
    ("q", "quit"),
];

/// Width of the key column before the description, in the single-column
/// layout (matches the longest key, "Enter/Spc").
const KEY_COL_WIDTH: usize = 11;

/// Full single-column layout: one line per keybinding.
fn help_lines_single() -> Vec<Line<'static>> {
    let mut lines = vec![Line::from("  Keybindings"), Line::from("")];
    for (key, desc) in HELP_KEYBINDINGS {
        lines.push(Line::from(format!("  {key:<KEY_COL_WIDTH$}{desc}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("  [any key]  dismiss"));
    lines
}

/// Compact two-column layout: same keybindings, roughly half the height.
/// Used only when the terminal is too short for the single-column layout, so
/// the whole list still fits with no clipping.
fn help_lines_compact() -> Vec<Line<'static>> {
    let desc_col_width = HELP_KEYBINDINGS
        .iter()
        .map(|(_, desc)| desc.len())
        .max()
        .unwrap_or(0);
    let split = HELP_KEYBINDINGS.len().div_ceil(2);
    let (left, right) = HELP_KEYBINDINGS.split_at(split);

    let mut lines = vec![Line::from("  Keybindings"), Line::from("")];
    for (i, &(lk, ld)) in left.iter().enumerate() {
        let row = match right.get(i) {
            Some((rk, rd)) => {
                format!("  {lk:<KEY_COL_WIDTH$}{ld:<desc_col_width$}  {rk:<KEY_COL_WIDTH$}{rd}")
            }
            None => format!("  {lk:<KEY_COL_WIDTH$}{ld}"),
        };
        lines.push(Line::from(row));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("  [any key]  dismiss"));
    lines
}

fn draw_help(f: &mut Frame) {
    let full_area = f.area();

    // Prefer the single-column layout; only fall back to the compact
    // two-column one when the terminal is too short for it, so every
    // keybinding stays visible either way - never silently clipped.
    let single = help_lines_single();
    let lines = if single.len() as u16 + 2 <= full_area.height {
        single
    } else {
        help_lines_compact()
    };

    let height = lines.len() as u16 + 2; // + top/bottom border
    let content_width = lines.iter().map(|l| l.width() as u16).max().unwrap_or(0);
    let width = content_width + 4; // + left/right border and one column of padding each side
    let area = centered_rect_fixed(width, height, full_area);

    let block = Block::default().title("  Help  ").borders(Borders::ALL);
    let para = Paragraph::new(Text::from(lines)).block(block);

    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

// ── ticket cards ──────────────────────────────────────────────────────────────

/// Returns the scroll offset that places `target_row` as the bottom-most visible
/// card, filling any remaining space with cards above it.
fn scroll_for_row(indices: &[usize], app: &App, target_row: usize, area: Rect) -> usize {
    if indices.is_empty() || target_row >= indices.len() {
        return 0;
    }
    let text_width = area.width.saturating_sub(2) as usize;
    let card_h = |ti: usize| -> u16 {
        let title = app.tickets[ti].front_matter.title.to_string();
        2 + wrap_text(&title, text_width).len() as u16
    };
    let focused_h = card_h(indices[target_row]);
    if focused_h >= area.height {
        return target_row;
    }
    let mut remaining = area.height - focused_h;
    let mut scroll = target_row;
    while scroll > 0 {
        let h = card_h(indices[scroll - 1]);
        if h > remaining {
            break;
        }
        remaining -= h;
        scroll -= 1;
    }
    scroll
}

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

        let card_area = Rect {
            x: inner.x,
            y,
            width: card_width,
            height: card_height,
        };

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

        let content: Vec<Line> = title_lines.into_iter().map(Line::from).collect();
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
        TicketType::Epic => Span::raw("🌟"),
        TicketType::Story => Span::raw("📖"),
        TicketType::Task => Span::raw("📋"),
        TicketType::Bug => Span::raw("🐛"),
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

/// Centres a box of an exact `width`/`height` within `r`, clamping both to
/// `r`'s bounds so the box never renders outside the terminal.
fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let width = width.min(r.width);
    let height = height.min(r.height);
    let x = r.x + (r.width - width) / 2;
    let y = r.y + (r.height - height) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

// ── snapshot tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use ratatui::{Terminal, backend::TestBackend};

    use crate::domain_types::{FrontMatter, Ticket, TicketId, TicketStatus, TicketType, Title};
    use crate::tui::app::App;

    /// Fixed, obviously-fictional timestamp for test fixtures, so rendered
    /// snapshots never depend on the wall clock. Constructed deterministically
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
        let now = fixed_timestamp();
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
        let columns = vec![TicketStatus::Todo];
        let tickets = vec![
            make_ticket("aaa111", "First ticket", TicketStatus::Todo),
            make_ticket("bbb222", "Second ticket", TicketStatus::Todo),
            make_ticket("ccc333", "Third ticket", TicketStatus::Todo),
        ];
        let mut app = App::new(tickets, columns);
        app.row = 2; // focus on 3rd card
        let output = render_to_string(&app, 30, 10);
        assert!(
            output.contains("ccc333"),
            "focused card not visible: {}",
            output
        );
        assert!(
            !output.contains("aaa111"),
            "first card should be scrolled off: {}",
            output
        );
    }

    #[test]
    fn board_shows_down_indicator_when_cards_clipped_below() {
        let columns = vec![TicketStatus::Todo];
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
        let columns = vec![TicketStatus::Todo];
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

    #[test]
    fn focused_card_always_visible_while_navigating_down() {
        // Cards with 2-line titles have height=4.
        // Terminal height=13 → board=12, cards_area=11.
        // rough_cap = 11/3 = 3, but actual capacity = 11/4 = 2 (3 cards = 12 > 11).
        // Bug: at row=2, scroll=0 so card 2 is not rendered (only 0 and 1 fit).
        let ids = ["aa1111", "bb2222", "cc3333", "dd4444", "ee5555"];
        let long_title = "A title that wraps to a second line here";
        let columns = vec![TicketStatus::Todo];
        let tickets = ids
            .iter()
            .map(|id| make_ticket(id, long_title, TicketStatus::Todo))
            .collect();
        let mut app = App::new(tickets, columns);
        for (row, id) in ids.iter().enumerate() {
            app.row = row;
            let output = render_to_string(&app, 30, 13);
            assert!(
                output.contains(id),
                "row={row}: focused card '{id}' not visible\n{output}"
            );
        }
    }

    // ── existing snapshot tests ────────────────────────────────────────────

    #[test]
    fn board_card_renders_as_bordered_box_with_id_in_title() {
        let columns = vec![TicketStatus::Todo];
        let tickets = vec![make_ticket("abc123", "Fix login bug", TicketStatus::Todo)];
        let app = App::new(tickets, columns);
        let output = render_to_string(&app, 30, 10);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn board_card_footer_shows_tags() {
        let columns = vec![TicketStatus::Todo];
        let tickets = vec![make_ticket_with_tags(
            "abc123",
            "Fix login bug",
            TicketStatus::Todo,
            &["tui", "config"],
        )];
        let app = App::new(tickets, columns);
        let output = render_to_string(&app, 30, 10);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn board_card_wraps_long_title() {
        let columns = vec![TicketStatus::Todo];
        let tickets = vec![make_ticket(
            "abc123",
            "Fix the login bug on the home page",
            TicketStatus::Todo,
        )];
        let app = App::new(tickets, columns);
        let output = render_to_string(&app, 30, 12);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn board_cards_clip_when_column_overflows() {
        let columns = vec![TicketStatus::Todo];
        let tickets = vec![
            make_ticket("aaa111", "First ticket", TicketStatus::Todo),
            make_ticket("bbb222", "Second ticket", TicketStatus::Todo),
            make_ticket(
                "ccc333",
                "Third ticket should not appear",
                TicketStatus::Todo,
            ),
        ];
        let app = App::new(tickets, columns);
        // height=10: outer column borders (2) + 3 cards × 3 lines = 11 → third card clips
        let output = render_to_string(&app, 30, 10);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn board_renders_three_columns() {
        let columns = vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ];
        let tickets = vec![
            make_ticket("aaa111", "Fix login bug", TicketStatus::Todo),
            make_ticket("bbb222", "Add search", TicketStatus::InProgress),
        ];
        let app = App::new(tickets, columns);
        let output = render_to_string(&app, 80, 20);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn board_renders_five_columns() {
        let columns = vec![
            TicketStatus::Draft,
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Review,
            TicketStatus::Done,
        ];
        let tickets = vec![
            make_ticket("aaa111", "Draft idea", TicketStatus::Draft),
            make_ticket("bbb222", "Fix login bug", TicketStatus::Todo),
            make_ticket("ccc333", "Add search", TicketStatus::InProgress),
            make_ticket("ddd444", "Check payments", TicketStatus::Review),
            make_ticket("eee555", "Ship release", TicketStatus::Done),
        ];
        let app = App::new(tickets, columns);
        let output = render_to_string(&app, 100, 20);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn detail_view_renders_ticket_fields() {
        let columns = vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ];
        let tickets = vec![make_ticket("abc123", "Fix login bug", TicketStatus::Todo)];
        let mut app = App::new(tickets, columns);
        app.screen = Screen::Detail;
        let output = render_to_string(&app, 80, 24);
        insta::assert_snapshot!(output);
    }

    fn make_ticket_with_body(id: &str, title: &str, status: TicketStatus, body: &str) -> Ticket {
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
            body: body.to_string(),
        }
    }

    /// A long body (40 lines) at 80x24 must never be silently cut - the
    /// overflow must be named with a truncation marker rather than dropped.
    #[test]
    fn detail_view_long_body_is_not_silently_truncated() {
        let columns = vec![TicketStatus::Todo];
        let body: String = (1..=40)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let tickets = vec![make_ticket_with_body(
            "abc123",
            "Fix login bug",
            TicketStatus::Todo,
            &body,
        )];
        let mut app = App::new(tickets, columns);
        app.screen = Screen::Detail;
        let output = render_to_string(&app, 80, 24);
        assert!(
            !output.contains("line 40"),
            "expected the tail of the body to be cut off in this fixture, got: {output}"
        );
        assert!(
            output.contains("more line") && output.contains("press e to open"),
            "expected a truncation marker naming the hidden content, got: {output}"
        );
    }

    /// The truncation marker must state a concrete, accurate count of hidden
    /// lines, not just a vague "more content" signal.
    #[test]
    fn detail_view_truncation_marker_states_accurate_hidden_count() {
        let columns = vec![TicketStatus::Todo];
        let body: String = (1..=40)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let tickets = vec![make_ticket_with_body(
            "abc123",
            "Fix login bug",
            TicketStatus::Todo,
            &body,
        )];
        let mut app = App::new(tickets, columns);
        app.screen = Screen::Detail;
        let output = render_to_string(&app, 80, 24);

        // Count how many "line N" body rows actually made it into the output.
        let shown = (1..=40)
            .filter(|n| output.contains(&format!("line {n}")))
            .count();
        let hidden = 40 - shown;
        assert!(hidden > 0, "fixture should force truncation: {output}");
        let expected_marker = format!("{hidden} more line");
        assert!(
            output.contains(&expected_marker),
            "expected marker to report {hidden} hidden lines, got: {output}"
        );
    }

    /// Boundary: a body that exactly fills the available height must render
    /// in full, with no marker and nothing cut.
    ///
    /// At 80x24, `centered_rect(80, 80, ...)` yields an area of height 20
    /// (18 inner rows once borders are subtracted). For a ticket fixture
    /// with no tags/parent/blocked-by, the header is 6 metadata lines plus
    /// 1 blank separator = 7, leaving exactly 11 rows for the body.
    #[test]
    fn detail_view_body_exactly_filling_height_shows_no_marker() {
        let columns = vec![TicketStatus::Todo];
        let body: String = (1..=11)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let tickets = vec![make_ticket_with_body(
            "abc123",
            "Fix login bug",
            TicketStatus::Todo,
            &body,
        )];
        let mut app = App::new(tickets, columns);
        app.screen = Screen::Detail;
        let output = render_to_string(&app, 80, 24);
        for n in 1..=11 {
            assert!(
                output.contains(&format!("line {n}")),
                "line {n} should be fully visible when the body exactly fits: {output}"
            );
        }
        assert!(
            !output.contains("more line"),
            "a body that exactly fits should show no truncation marker: {output}"
        );
    }

    /// One line over the boundary: the 12th line must not be silently
    /// dropped - it must be represented by the truncation marker.
    #[test]
    fn detail_view_body_one_line_over_height_shows_marker() {
        let columns = vec![TicketStatus::Todo];
        let body: String = (1..=12)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let tickets = vec![make_ticket_with_body(
            "abc123",
            "Fix login bug",
            TicketStatus::Todo,
            &body,
        )];
        let mut app = App::new(tickets, columns);
        app.screen = Screen::Detail;
        let output = render_to_string(&app, 80, 24);
        assert!(
            output.contains("more line") && output.contains("press e to open"),
            "one line over capacity should trigger the truncation marker: {output}"
        );
    }

    // ── filter ─────────────────────────────────────────────────────────────

    #[test]
    fn board_renders_filter_prompt_while_typing() {
        let columns = vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ];
        let tickets = vec![
            make_ticket("aaa111", "Fix login bug", TicketStatus::Todo),
            make_ticket("bbb222", "Add search", TicketStatus::InProgress),
        ];
        let mut app = App::new(tickets, columns);
        app.screen = Screen::Filter;
        app.filter = "log".to_string();
        let output = render_to_string(&app, 80, 20);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn board_renders_active_filter_indicator_after_enter() {
        let columns = vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ];
        let tickets = vec![
            make_ticket("aaa111", "Fix login bug", TicketStatus::Todo),
            make_ticket("bbb222", "Add search", TicketStatus::InProgress),
        ];
        let mut app = App::new(tickets, columns);
        app.screen = Screen::Board;
        app.filter = "log".to_string();
        let output = render_to_string(&app, 80, 20);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn help_overlay_renders_keybindings() {
        let columns = vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ];
        let mut app = App::new(vec![], columns);
        app.screen = Screen::Help;
        let output = render_to_string(&app, 80, 24);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn help_overlay_shows_quit_keybinding_at_80x24() {
        let columns = vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ];
        let mut app = App::new(vec![], columns);
        app.screen = Screen::Help;
        let output = render_to_string(&app, 80, 24);
        assert!(
            output.contains("q          quit"),
            "quit keybinding not visible: {}",
            output
        );
    }

    #[test]
    fn help_overlay_shows_dismiss_line_at_80x24() {
        let columns = vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ];
        let mut app = App::new(vec![], columns);
        app.screen = Screen::Help;
        let output = render_to_string(&app, 80, 24);
        assert!(
            output.contains("[any key]  dismiss"),
            "dismiss line not visible: {}",
            output
        );
    }

    /// Terminal too short for the single-column layout (needs 19 rows):
    /// the overlay must switch to the compact two-column layout rather than
    /// clip, so every keybinding stays visible.
    #[test]
    fn help_overlay_switches_to_compact_layout_on_short_terminal() {
        let columns = vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ];
        let mut app = App::new(vec![], columns);
        app.screen = Screen::Help;
        let output = render_to_string(&app, 80, 15);
        for (key, _) in HELP_KEYBINDINGS {
            assert!(
                output.contains(key),
                "keybinding '{key}' not visible at 80x15: {output}"
            );
        }
        assert!(
            output.contains("dismiss"),
            "dismiss line not visible at 80x15: {output}"
        );
    }

    #[test]
    fn help_overlay_compact_layout_renders_correctly() {
        let columns = vec![
            TicketStatus::Todo,
            TicketStatus::InProgress,
            TicketStatus::Done,
        ];
        let mut app = App::new(vec![], columns);
        app.screen = Screen::Help;
        let output = render_to_string(&app, 80, 15);
        insta::assert_snapshot!(output);
    }
}
