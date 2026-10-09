//! Semantic colours and icons shared by every pretty presenter, so a status
//! or type looks the same in `list` and `show`.
//!
//! Colour is switched by `Style::new(enabled)`. Icons mark meaning, not
//! colour, so they stay on under `NO_COLOR`. Only single-codepoint emoji are
//! used, so terminals agree on their width.

const RESET: &str = "\x1b[0m";

/// Icons that lead the labelled lines of the detail view.
pub const PARENT_ICON: &str = "📁";
pub const BLOCKED_ICON: &str = "⛔";
pub const DATES_ICON: &str = "📅";

/// Text ready to print and the terminal columns it occupies, so a table can
/// pad it correctly even when it holds colour codes or double-width emoji.
pub struct Cell {
    pub shown: String,
    pub width: usize,
}

impl Cell {
    pub fn plain(text: &str) -> Cell {
        Cell {
            shown: text.to_string(),
            width: text.chars().count(),
        }
    }

    /// `shown` followed by spaces up to `width` columns.
    pub fn padded(&self, width: usize) -> String {
        format!(
            "{}{}",
            self.shown,
            " ".repeat(width.saturating_sub(self.width))
        )
    }
}

pub struct Style {
    enabled: bool,
}

impl Style {
    pub fn new(enabled: bool) -> Self {
        Style { enabled }
    }

    pub fn status(&self, text: &str) -> String {
        self.paint(
            match text {
                "todo" => Some("\x1b[90m"),
                "in progress" => Some("\x1b[33m"),
                "done" => Some("\x1b[32m"),
                "rejected" => Some("\x1b[31m"),
                _ => None,
            },
            text,
        )
    }

    pub fn r#type(&self, text: &str) -> String {
        self.paint(
            match text {
                "bug" => Some("\x1b[31m"),
                "feature" => Some("\x1b[36m"),
                _ => None,
            },
            text,
        )
    }

    pub fn bold(&self, text: &str) -> String {
        self.paint(Some("\x1b[1m"), text)
    }

    /// The status with its icon in front, when it has one.
    pub fn status_with_icon(&self, text: &str) -> String {
        self.status_cell(text).shown
    }

    /// A table heading: bold, but still only as wide as its text.
    pub fn heading(&self, text: &str) -> Cell {
        Cell {
            shown: self.bold(text),
            width: text.chars().count(),
        }
    }

    pub fn status_cell(&self, text: &str) -> Cell {
        with_icon(
            match text {
                "todo" | "open" | "backlog" => Some("⚪"),
                "in progress" | "doing" => Some("🟡"),
                "in review" => Some("👀"),
                "blocked" => Some("⛔"),
                "testing" | "qa" => Some("🧪"),
                "on hold" | "waiting" => Some("🔵"),
                "done" => Some("✅"),
                "rejected" | "won't do" => Some("❌"),
                "duplicate" => Some("🔁"),
                _ => None,
            },
            self.status(text),
            text,
        )
    }

    /// The type with its icon in front, when it has one.
    pub fn type_with_icon(&self, text: &str) -> String {
        self.type_cell(text).shown
    }

    pub fn type_cell(&self, text: &str) -> Cell {
        with_icon(
            match text {
                "bug" => Some("🐛"),
                "feature" => Some("✨"),
                "task" => Some("🧩"),
                "chore" => Some("🧹"),
                "epic" => Some("🎯"),
                "story" => Some("📖"),
                "spike" | "research" => Some("🔬"),
                "docs" => Some("📝"),
                "refactor" => Some("🔧"),
                "test" => Some("🧪"),
                "incident" => Some("🚨"),
                "idea" => Some("💡"),
                _ => None,
            },
            self.r#type(text),
            text,
        )
    }

    fn paint(&self, code: Option<&str>, text: &str) -> String {
        match code {
            Some(code) if self.enabled => format!("{code}{text}{RESET}"),
            _ => text.to_string(),
        }
    }
}

/// `painted` (the coloured `raw`) behind its icon and a space. An emoji takes
/// two columns.
fn with_icon(icon: Option<&str>, painted: String, raw: &str) -> Cell {
    let width = raw.chars().count();
    match icon {
        Some(icon) => Cell {
            shown: format!("{icon} {painted}"),
            width: width + 3,
        },
        None => Cell {
            shown: painted,
            width,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_precede_the_coloured_text() {
        let style = Style::new(true);
        assert_eq!(
            style.status_with_icon("done"),
            format!("✅ {}", style.status("done"))
        );
        assert_eq!(
            style.type_with_icon("bug"),
            format!("🐛 {}", style.r#type("bug"))
        );
    }

    #[test]
    fn cell_width_ignores_colour_codes_and_counts_the_icon_as_three_columns() {
        let coloured = Style::new(true).status_cell("done");
        let plain = Style::new(false).status_cell("done");
        assert_eq!(coloured.width, 7);
        assert_eq!(plain.width, 7);
        assert_eq!(Style::new(true).type_cell("misc").width, 4);
        assert_eq!(plain.padded(9), "✅ done  ");
    }

    #[test]
    fn common_types_have_icons() {
        let style = Style::new(false);
        for (name, icon) in [
            ("chore", "🧹"),
            ("epic", "🎯"),
            ("story", "📖"),
            ("spike", "🔬"),
            ("research", "🔬"),
            ("docs", "📝"),
            ("refactor", "🔧"),
            ("test", "🧪"),
            ("incident", "🚨"),
            ("idea", "💡"),
        ] {
            assert_eq!(style.type_with_icon(name), format!("{icon} {name}"));
        }
    }

    #[test]
    fn common_statuses_have_icons() {
        let style = Style::new(false);
        for (name, icon) in [
            ("open", "⚪"),
            ("backlog", "⚪"),
            ("doing", "🟡"),
            ("in review", "👀"),
            ("blocked", "⛔"),
            ("testing", "🧪"),
            ("qa", "🧪"),
            ("on hold", "🔵"),
            ("waiting", "🔵"),
            ("duplicate", "🔁"),
            ("won't do", "❌"),
        ] {
            assert_eq!(style.status_with_icon(name), format!("{icon} {name}"));
        }
    }

    #[test]
    fn icons_stay_when_colour_is_off() {
        let style = Style::new(false);
        assert_eq!(style.status_with_icon("in progress"), "🟡 in progress");
        assert_eq!(style.type_with_icon("feature"), "✨ feature");
    }

    #[test]
    fn unknown_statuses_and_types_get_no_icon() {
        let style = Style::new(false);
        assert_eq!(style.status_with_icon("triaged"), "triaged");
        assert_eq!(style.type_with_icon("misc"), "misc");
    }
}
