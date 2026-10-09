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

const RESET: &str = "\x1b[0m";

fn status_colour(status: &str) -> Option<&'static str> {
    match status {
        "todo" => Some("\x1b[90m"),
        "in progress" => Some("\x1b[33m"),
        "done" => Some("\x1b[32m"),
        "rejected" => Some("\x1b[31m"),
        _ => None,
    }
}

fn type_colour(r#type: &str) -> Option<&'static str> {
    match r#type {
        "bug" => Some("\x1b[31m"),
        "feature" => Some("\x1b[36m"),
        _ => None,
    }
}

/// Pads to `width` columns, then colours only the text so the padding stays
/// uncoloured and the columns line up with or without colour.
/// Cuts `text` to at most `max` characters, ending in `…` when it was cut.
fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

fn cell(text: &str, width: usize, colour: Option<&str>) -> String {
    let padding = " ".repeat(width.saturating_sub(text.chars().count()));
    match colour {
        Some(code) => format!("{code}{text}{RESET}{padding}"),
        None => format!("{text}{padding}"),
    }
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
            cell("ID", id_width, None),
            cell("STATUS", status_width, None),
            cell("TYPE", type_width, None)
        )?;
        for (t, r#type) in self.tickets.iter().zip(types) {
            let status = t.status.to_string();
            let (status_code, type_code) = if self.colour {
                (status_colour(&status), type_colour(r#type))
            } else {
                (None, None)
            };
            write!(
                f,
                "\n{}  {}  {}  {}",
                cell(&t.id.to_string(), id_width, None),
                cell(&status, status_width, status_code),
                cell(r#type, type_width, type_code),
                match title_width {
                    Some(max) => truncate(&t.title.to_string(), max),
                    None => t.title.to_string(),
                }
            )?;
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
