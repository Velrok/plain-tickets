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
        with_icon(
            match text {
                "todo" => Some("⚪"),
                "in progress" => Some("🟡"),
                "done" => Some("✅"),
                "rejected" => Some("❌"),
                _ => None,
            },
            self.status(text),
        )
    }

    /// The type with its icon in front, when it has one.
    pub fn type_with_icon(&self, text: &str) -> String {
        with_icon(
            match text {
                "bug" => Some("🐛"),
                "feature" => Some("✨"),
                "task" => Some("🧩"),
                _ => None,
            },
            self.r#type(text),
        )
    }

    fn paint(&self, code: Option<&str>, text: &str) -> String {
        match code {
            Some(code) if self.enabled => format!("{code}{text}{RESET}"),
            _ => text.to_string(),
        }
    }
}

fn with_icon(icon: Option<&str>, text: String) -> String {
    match icon {
        Some(icon) => format!("{icon} {text}"),
        None => text,
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
    fn icons_stay_when_colour_is_off() {
        let style = Style::new(false);
        assert_eq!(style.status_with_icon("in progress"), "🟡 in progress");
        assert_eq!(style.type_with_icon("feature"), "✨ feature");
    }

    #[test]
    fn unknown_statuses_and_types_get_no_icon() {
        let style = Style::new(false);
        assert_eq!(style.status_with_icon("blocked"), "blocked");
        assert_eq!(style.type_with_icon("chore"), "chore");
    }
}
