use super::style::Style;
use crate::domain::id::ID;
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

/// Pads to `width` columns by the length of the raw `text`, appending the
/// padding after `shown` (the same text, possibly coloured), so columns line
/// up with or without colour.
fn cell(shown: &str, text: &str, width: usize) -> String {
    let padding = " ".repeat(width.saturating_sub(text.chars().count()));
    format!("{shown}{padding}")
}

impl fmt::Display for PrettyListPresenter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tickets.is_empty() {
            return Ok(());
        }
        let types: Vec<&str> = self
            .tickets
            .iter()
            .map(|t| t.r#type.as_deref().unwrap_or(""))
            .collect();
        let status_width = self
            .tickets
            .iter()
            .map(|t| t.status.to_string().chars().count())
            .chain([6])
            .max()
            .unwrap_or(6);
        let type_width = types
            .iter()
            .map(|t| t.chars().count())
            .chain([4])
            .max()
            .unwrap_or(4);
        let id_width = 16;
        let title_width = self.width.map(|w| {
            w.saturating_sub(id_width + status_width + type_width + 6)
                .max(1)
        });
        write!(
            f,
            "{}  {}  {}  TITLE",
            cell("ID", "ID", id_width),
            cell("STATUS", "STATUS", status_width),
            cell("TYPE", "TYPE", type_width)
        )?;
        for (t, r#type) in self.tickets.iter().zip(types) {
            let status = t.status.to_string();
            let id = t.id.to_string();
            let style = Style::new(self.colour);
            write!(
                f,
                "\n{}  {}  {}  {}",
                cell(&id, &id, id_width),
                cell(&style.status(&status), &status, status_width),
                cell(&style.r#type(r#type), r#type, type_width),
                match title_width {
                    Some(max) => truncate(&t.title.to_string(), max),
                    None => t.title.to_string(),
                }
            )?;
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
    pub parent: Option<Relation>,
    pub blockers: Vec<Relation>,
    pub colour: bool,
}

/// `YYYY-MM-DD` (UTC) for seconds since the Unix epoch.
fn date(secs: u64) -> String {
    // Days to civil date, after Howard Hinnant's algorithm.
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// `<id> <title> [<status>]`, or just the id when the ticket is gone.
fn describe(relation: &Relation, style: &Style) -> String {
    match &relation.ticket {
        Some(t) => format!(
            "{} {} [{}]",
            relation.id,
            t.title,
            style.status(&t.status.to_string())
        ),
        None => format!("{} (not found)", relation.id),
    }
}

impl fmt::Display for PrettyDetailPresenter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = self.ticket;
        let style = Style::new(self.colour);
        writeln!(f, "{}", style.bold(&t.title.to_string()))?;
        let mut meta = vec![t.id.to_string()];
        meta.extend(t.r#type.as_deref().map(|ty| style.r#type(ty)));
        meta.push(style.status(&t.status.to_string()));
        if !t.tags.is_empty() {
            let tags: Vec<String> = t.tags.iter().map(|tag| format!("#{tag}")).collect();
            meta.push(tags.join(" "));
        }
        writeln!(f, "{}", meta.join(" · "))?;
        if let Some(parent) = &self.parent {
            writeln!(f, "Parent: {}", describe(parent, &style))?;
        }
        for (i, blocker) in self.blockers.iter().enumerate() {
            let label = if i == 0 { "Blocked by:" } else { "           " };
            writeln!(f, "{label} {}", describe(blocker, &style))?;
        }
        write!(
            f,
            "Created {} · Updated {}",
            date(t.created_at),
            date(t.updated_at)
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
    use crate::domain::id::ID;
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
        Ticket::new(draft, &config, ID(id), 0).unwrap()
    }

    #[test]
    fn presents_id_status_type_and_title_separated_by_tabs() {
        let ticket = ticket(0xab, "in progress", None, "Fix it");
        assert_eq!(
            PlainLinePresenter(&ticket).to_string(),
            "00000000000000ab\tin progress\t\tFix it"
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
            "Fix it\n00000000000000ab · todo\nCreated 1970-01-01 · Updated 1970-01-01"
        );
    }

    #[test]
    fn shows_type_and_tags_in_the_meta_line_and_real_dates() {
        let mut ticket = ticket(0xab, "in progress", Some("bug"), "Fix it");
        ticket.tags = vec!["cli".into(), "x".into()];
        ticket.created_at = 1_791_504_758;
        ticket.updated_at = 1_791_504_758 + 86_400;
        assert_eq!(
            detail(&ticket),
            "Fix it\n00000000000000ab · bug · in progress · #cli #x\nCreated 2026-10-09 · Updated 2026-10-10"
        );
    }

    #[test]
    fn resolves_parent_and_blockers_with_title_and_status() {
        let ticket = ticket(0xab, "todo", None, "Fix it");
        let epic = self::ticket(1, "in progress", None, "The epic");
        let first = self::ticket(2, "done", None, "First");
        let shown = PrettyDetailPresenter {
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
             00000000000000ab · todo\n\
             Parent: 0000000000000001 The epic [in progress]\n\
             Blocked by: 0000000000000002 First [done]\n\
             \x20           0000000000000003 (not found)\n\
             Created 1970-01-01 · Updated 1970-01-01"
        );
    }

    #[test]
    fn prints_a_rule_then_the_raw_body() {
        let mut ticket = ticket(0xab, "todo", None, "Fix it");
        ticket.body = "# Heading\n\nSome *body*\n".into();
        assert_eq!(
            detail(&ticket),
            format!(
                "Fix it\n00000000000000ab · todo\nCreated 1970-01-01 · Updated 1970-01-01\n{}\n# Heading\n\nSome *body*",
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
        assert!(coloured.contains(&format!("[{}]", style.status("done"))));
    }

    #[test]
    fn dates_handle_leap_days() {
        assert_eq!(date(1_709_164_800), "2024-02-29");
        assert_eq!(date(1_709_251_200), "2024-03-01");
    }

    #[test]
    fn truncates_long_titles_with_an_ellipsis_to_fit_the_width() {
        let tickets = [
            ticket(1, "todo", Some("bug"), "A very long title that cannot fit"),
            ticket(2, "todo", None, "Short"),
        ];
        let table = PrettyListPresenter {
            tickets: &tickets,
            colour: false,
            width: Some(40),
        }
        .to_string();
        let lines: Vec<&str> = table.lines().collect();
        assert!(lines.iter().all(|l| l.chars().count() <= 40), "{lines:?}");
        assert_eq!(lines[1], "0000000000000001  todo    bug   A very …");
        assert_eq!(lines[2], "0000000000000002  todo          Short");
    }

    #[test]
    fn does_not_truncate_without_a_known_width() {
        let tickets = [ticket(1, "todo", None, "A very long title that cannot fit")];
        let table = PrettyListPresenter {
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
            tickets: &tickets,
            colour: false,
            width: None,
        }
        .to_string();
        let coloured = PrettyListPresenter {
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
