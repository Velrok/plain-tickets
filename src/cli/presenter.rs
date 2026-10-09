use super::style::{BLOCKED_ICON, Cell, DATES_ICON, PARENT_ICON, Style};
use crate::domain::id::{ID, IdAbbrev};
use crate::domain::tickets::Ticket;
use std::fmt;

/// One tab-separated line per ticket: `<id>\t<status>\t<type>\t<title>`.
pub struct PlainLinePresenter<'a>(pub &'a Ticket);

impl fmt::Display for PlainLinePresenter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = self.0;
        write!(
            f,
            "{}\t{}\t{}\t{}",
            t.id,
            t.status,
            t.r#type.as_deref().unwrap_or(""),
            t.title
        )
    }
}

/// An aligned table with a header row, for people. Empty for no tickets.
pub struct PrettyListPresenter<'a> {
    pub tickets: &'a [Ticket],
    pub ids: &'a IdAbbrev,
    pub colour: bool,
    /// Terminal width; titles are cut to fit. `None` leaves them whole.
    pub width: Option<usize>,
}

/// Cuts `text` to at most `max` characters, ending in `…` when it was cut.
fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

impl fmt::Display for PrettyListPresenter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tickets.is_empty() {
            return Ok(());
        }
        let style = Style::new(self.colour);
        let rows: Vec<[Cell; 3]> = self
            .tickets
            .iter()
            .map(|t| {
                [
                    style.id_cell(&t.id.to_string(), self.ids.unique_len(t.id)),
                    style.status_cell(&t.status.to_string()),
                    style.type_cell(t.r#type.as_deref().unwrap_or("")),
                ]
            })
            .collect();
        let header = [
            style.heading("ID"),
            style.heading("STATUS"),
            style.heading("TYPE"),
        ];
        let widths: [usize; 3] = std::array::from_fn(|i| {
            rows.iter()
                .map(|r| r[i].width)
                .chain([header[i].width])
                .max()
                .unwrap_or(0)
        });
        let title_width = self
            .width
            .map(|w| w.saturating_sub(widths.iter().sum::<usize>() + 6).max(1));
        let line = |cells: &[Cell; 3], title: &str| {
            format!(
                "{}  {}  {}  {title}",
                cells[0].padded(widths[0]),
                cells[1].padded(widths[1]),
                cells[2].padded(widths[2])
            )
        };
        write!(f, "{}", line(&header, &style.heading("TITLE").shown))?;
        for (t, cells) in self.tickets.iter().zip(&rows) {
            let title = t.title.to_string();
            let title = match title_width {
                Some(max) => truncate(&title, max),
                None => title,
            };
            write!(f, "\n{}", line(cells, &title))?;
        }
        Ok(())
    }
}

/// A ticket that another ticket points at, looked up by the caller. `ticket`
/// is `None` when the file no longer exists.
pub struct Relation {
    pub id: ID,
    pub ticket: Option<Ticket>,
}

/// A readable header block (title, ids, relations, dates), then the raw body.
pub struct PrettyDetailPresenter<'a> {
    pub ticket: &'a Ticket,
    pub ids: &'a IdAbbrev,
    pub parent: Option<Relation>,
    pub blockers: Vec<Relation>,
    pub colour: bool,
}

/// `<id> <title> [<status>]`, or just the id when the ticket is gone.
fn describe(relation: &Relation, style: &Style, ids: &IdAbbrev) -> String {
    match &relation.ticket {
        Some(t) => format!(
            "{} {} [{}]",
            show_id(relation.id, style, ids),
            t.title,
            style.status_with_icon(&t.status.to_string())
        ),
        None => format!("{} (not found)", show_id(relation.id, style, ids)),
    }
}

/// The full ID, with the characters beyond its shortest unique prefix greyed out.
fn show_id(id: ID, style: &Style, ids: &IdAbbrev) -> String {
    style.id_cell(&id.to_string(), ids.unique_len(id)).shown
}

impl fmt::Display for PrettyDetailPresenter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = self.ticket;
        let style = Style::new(self.colour);
        writeln!(f, "{}", style.bold(&t.title.to_string()))?;
        let mut meta = vec![show_id(t.id, &style, self.ids)];
        meta.extend(t.r#type.as_deref().map(|ty| style.type_with_icon(ty)));
        let status = t.status.to_string();
        meta.push(style.status_with_icon(&status));
        if !t.tags.is_empty() {
            let tags: Vec<String> = t.tags.iter().map(|tag| format!("#{tag}")).collect();
            meta.push(tags.join(" "));
        }
        writeln!(f, "{}", meta.join(" · "))?;
        if let Some(parent) = &self.parent {
            writeln!(
                f,
                "{PARENT_ICON} Parent: {}",
                describe(parent, &style, self.ids)
            )?;
        }
        for (i, blocker) in self.blockers.iter().enumerate() {
            let label = if i == 0 {
                format!("{BLOCKED_ICON} Blocked by:")
            } else {
                " ".repeat(14)
            };
            writeln!(f, "{label} {}", describe(blocker, &style, self.ids))?;
        }
        write!(
            f,
            "{DATES_ICON} Created {} · Updated {}",
            t.created_at.date(),
            t.updated_at.date()
        )?;
        let body = t.body.trim_end();
        if !body.is_empty() {
            write!(f, "\n{}\n{body}", "─".repeat(40))?;
        }
        Ok(())
    }
}

/// The ticket as stored: front matter, then body. Has no trailing newline, so
/// printing it with `println!` ends the output with exactly one.
pub struct PlainDetailPresenter<'a>(pub &'a Ticket);

impl fmt::Display for PlainDetailPresenter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.0.to_string();
        f.write_str(text.strip_suffix('\n').unwrap_or(&text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::domain::id::{ID, IdAbbrev};
    use crate::domain::tickets::NewTicket;

    fn ticket(id: u64, status: &str, r#type: Option<&str>, title: &str) -> Ticket {
        let draft = NewTicket {
            title: title.into(),
            r#type: r#type.map(|t| t.parse().unwrap()),
            status: Some(status.parse().unwrap()),
            tags: vec![],
            parent: None,
            blocked_by: vec![],
            body: String::new(),
        };
        let config: Config = r#"statuses = ["todo", "in progress"]"#.parse().unwrap();
        Ticket::new(
            draft,
            &config,
            ID(id),
            "1970-01-01T00:00:00Z".parse().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn presents_id_status_type_and_title_separated_by_tabs() {
        let ticket = ticket(0xab, "in progress", None, "Fix it");
        assert_eq!(
            PlainLinePresenter(&ticket).to_string(),
            "000000000004r\tin progress\t\tFix it"
        );
    }

    fn without_colour(s: &str) -> String {
        let mut out = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                for c in chars.by_ref() {
                    if c == 'm' {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    fn detail(ticket: &Ticket) -> String {
        PrettyDetailPresenter {
            ids: &IdAbbrev::full(),
            ticket,
            parent: None,
            blockers: vec![],
            colour: false,
        }
        .to_string()
    }

    #[test]
    fn shows_title_id_status_and_dates_for_a_bare_ticket() {
        let ticket = ticket(0xab, "todo", None, "Fix it");
        assert_eq!(
            detail(&ticket),
            "Fix it\n000000000004r · ⚪ todo\n📅 Created 1970-01-01 · Updated 1970-01-01"
        );
    }

    #[test]
    fn shows_type_and_tags_in_the_meta_line_and_real_dates() {
        let mut ticket = ticket(0xab, "in progress", Some("bug"), "Fix it");
        ticket.tags = vec!["cli".into(), "x".into()];
        ticket.created_at = "2026-10-09T10:12:38Z".parse().unwrap();
        ticket.updated_at = "2026-10-10T10:12:38Z".parse().unwrap();
        assert_eq!(
            detail(&ticket),
            "Fix it\n000000000004r · 🐛 bug · 🟡 in progress · #cli #x\n📅 Created 2026-10-09 · Updated 2026-10-10"
        );
    }

    #[test]
    fn resolves_parent_and_blockers_with_title_and_status() {
        let ticket = ticket(0xab, "todo", None, "Fix it");
        let epic = self::ticket(1, "in progress", None, "The epic");
        let first = self::ticket(2, "done", None, "First");
        let shown = PrettyDetailPresenter {
            ids: &IdAbbrev::full(),
            ticket: &ticket,
            parent: Some(Relation {
                id: ID(1),
                ticket: Some(epic),
            }),
            blockers: vec![
                Relation {
                    id: ID(2),
                    ticket: Some(first),
                },
                Relation {
                    id: ID(3),
                    ticket: None,
                },
            ],
            colour: false,
        }
        .to_string();
        assert_eq!(
            shown,
            "Fix it\n\
             000000000004r · ⚪ todo\n\
             📁 Parent: 0000000000001 The epic [🟡 in progress]\n\
             ⛔ Blocked by: 0000000000002 First [✅ done]\n\
             \x20              0000000000003 (not found)\n\
             📅 Created 1970-01-01 · Updated 1970-01-01"
        );
    }

    #[test]
    fn prints_a_rule_then_the_raw_body() {
        let mut ticket = ticket(0xab, "todo", None, "Fix it");
        ticket.body = "# Heading\n\nSome *body*\n".into();
        assert_eq!(
            detail(&ticket),
            format!(
                "Fix it\n000000000004r · ⚪ todo\n📅 Created 1970-01-01 · Updated 1970-01-01\n{}\n# Heading\n\nSome *body*",
                "─".repeat(40)
            )
        );
    }

    #[test]
    fn colours_with_the_shared_style_and_keeps_the_layout() {
        let style = Style::new(true);
        let ticket = ticket(0xab, "in progress", Some("bug"), "Fix it");
        let done = self::ticket(1, "done", None, "Epic");
        let render = |colour| {
            PrettyDetailPresenter {
                ids: &IdAbbrev::full(),
                ticket: &ticket,
                parent: Some(Relation {
                    id: ID(1),
                    ticket: Some(done.clone()),
                }),
                blockers: vec![],
                colour,
            }
            .to_string()
        };
        let (plain, coloured) = (render(false), render(true));
        assert_eq!(without_colour(&coloured), plain);
        assert!(!plain.contains('\x1b'));
        assert!(coloured.starts_with(&style.bold("Fix it")));
        assert!(coloured.contains(&style.r#type("bug")));
        assert_eq!(coloured.matches(&style.status("in progress")).count(), 1);
        assert!(coloured.contains(&format!("[{}]", style.status_with_icon("done"))));
    }

    #[test]
    fn list_icons_sit_inside_the_columns_and_widths_count_emoji_as_two() {
        let tickets = [
            ticket(1, "todo", Some("bug"), "A"),
            ticket(2, "in progress", Some("task"), "B"),
            ticket(3, "done", None, "C"),
        ];
        let table = PrettyListPresenter {
            ids: &IdAbbrev::full(),
            tickets: &tickets,
            colour: false,
            width: None,
        }
        .to_string();
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(
            lines[0],
            format!(
                "ID{}STATUS{}TYPE{}TITLE",
                " ".repeat(13),
                " ".repeat(10),
                " ".repeat(5)
            )
        );
        assert_eq!(
            lines[1],
            format!(
                "0000000000001  ⚪ todo{}🐛 bug{}A",
                " ".repeat(9),
                " ".repeat(3)
            )
        );
        assert_eq!(lines[2], "0000000000002  🟡 in progress  🧩 task  B");
        assert_eq!(
            lines[3],
            format!("0000000000003  ✅ done{}{}C", " ".repeat(9), " ".repeat(9))
        );
    }

    #[test]
    fn a_type_without_an_icon_still_aligns_with_icon_rows() {
        let tickets = [
            ticket(1, "todo", Some("bug"), "A"),
            ticket(2, "todo", Some("misc"), "B"),
        ];
        let table = PrettyListPresenter {
            ids: &IdAbbrev::full(),
            tickets: &tickets,
            colour: false,
            width: None,
        }
        .to_string();
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines[1], "0000000000001  ⚪ todo  🐛 bug  A");
        assert_eq!(lines[2], "0000000000002  ⚪ todo  misc    B");
    }

    #[test]
    fn the_header_row_is_bold_only_with_colour() {
        let tickets = [ticket(1, "todo", Some("bug"), "A")];
        let render = |colour| {
            PrettyListPresenter {
                ids: &IdAbbrev::full(),
                tickets: &tickets,
                colour,
                width: None,
            }
            .to_string()
        };
        let (plain, coloured) = (render(false), render(true));
        let style = Style::new(true);
        let header = coloured.lines().next().unwrap();
        for title in ["ID", "STATUS", "TYPE", "TITLE"] {
            assert!(header.contains(&style.bold(title)), "{header:?}");
        }
        assert!(!plain.contains('\x1b'));
        assert_eq!(without_colour(&coloured), plain);
    }

    #[test]
    fn truncates_long_titles_with_an_ellipsis_to_fit_the_width() {
        let tickets = [
            ticket(1, "todo", Some("bug"), "A very long title that cannot fit"),
            ticket(2, "todo", None, "Short"),
        ];
        let table = PrettyListPresenter {
            ids: &IdAbbrev::full(),
            tickets: &tickets,
            colour: false,
            width: Some(40),
        }
        .to_string();
        let lines: Vec<&str> = table.lines().collect();
        // Row 1 holds two emoji, each two columns wide but one char.
        assert_eq!(lines[1], "0000000000001  ⚪ todo  🐛 bug  A very …");
        assert_eq!(lines[1].chars().count() + 2, 40);
        assert_eq!(
            lines[2],
            format!("0000000000002  ⚪ todo{}Short", " ".repeat(10))
        );
    }

    #[test]
    fn does_not_truncate_without_a_known_width() {
        let tickets = [ticket(1, "todo", None, "A very long title that cannot fit")];
        let table = PrettyListPresenter {
            ids: &IdAbbrev::full(),
            tickets: &tickets,
            colour: false,
            width: None,
        }
        .to_string();
        assert!(table.ends_with("A very long title that cannot fit"));
    }

    #[test]
    fn colours_status_and_type_without_disturbing_alignment() {
        let tickets = [
            ticket(1, "todo", Some("bug"), "A"),
            ticket(2, "done", Some("feature"), "B"),
            ticket(3, "in progress", Some("task"), "C"),
        ];
        let plain = PrettyListPresenter {
            ids: &IdAbbrev::full(),
            tickets: &tickets,
            colour: false,
            width: None,
        }
        .to_string();
        let coloured = PrettyListPresenter {
            ids: &IdAbbrev::full(),
            tickets: &tickets,
            colour: true,
            width: None,
        }
        .to_string();
        assert!(!plain.contains('\x1b'));
        assert_eq!(without_colour(&coloured), plain);
        assert!(coloured.contains("\x1b[90mtodo\x1b[0m"));
        assert!(coloured.contains("\x1b[32mdone\x1b[0m"));
        assert!(coloured.contains("\x1b[33min progress\x1b[0m"));
        assert!(coloured.contains("\x1b[31mbug\x1b[0m"));
        assert!(coloured.contains("\x1b[36mfeature\x1b[0m"));
        assert!(coloured.contains("task"));
        assert!(!coloured.contains("\x1b[0mtask"));
    }
}
