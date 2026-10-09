//! Semantic colours shared by every pretty presenter, so a status or type
//! looks the same in `list` and `show`.

const RESET: &str = "\x1b[0m";

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

    fn paint(&self, code: Option<&str>, text: &str) -> String {
        match code {
            Some(code) if self.enabled => format!("{code}{text}{RESET}"),
            _ => text.to_string(),
        }
    }
}
