use super::args::Format;
use std::io::IsTerminal;

/// How list and show should render, once flags and environment are resolved.
pub struct Mode {
    pub format: Format,
    /// On for every pretty render (chosen by flag or by a terminal) unless `NO_COLOR` is set.
    pub colour: bool,
}

/// An explicit flag wins; otherwise pretty on a terminal, plain when piped.
pub fn resolve(flag: Option<Format>, is_tty: bool, no_color: bool) -> Mode {
    let format = flag.unwrap_or(if is_tty {
        Format::Pretty
    } else {
        Format::Plain
    });
    Mode {
        format,
        colour: format == Format::Pretty && !no_color,
    }
}

/// Resolves the mode for stdout from the flag, the terminal and `NO_COLOR`
/// (set and non-empty, per no-color.org).
pub fn current(flag: Option<Format>) -> Mode {
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    resolve(flag, std::io::stdout().is_terminal(), no_color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_pretty_on_a_terminal_and_plain_otherwise() {
        assert_eq!(resolve(None, true, false).format, Format::Pretty);
        assert_eq!(resolve(None, false, false).format, Format::Plain);
    }

    #[test]
    fn no_color_drops_colour_but_keeps_the_layout() {
        let coloured = resolve(None, true, false);
        assert!(coloured.colour);
        let plain_text = resolve(None, true, true);
        assert_eq!(plain_text.format, Format::Pretty);
        assert!(!plain_text.colour);
    }

    #[test]
    fn an_explicit_pretty_format_keeps_colour_when_piped_unless_no_color() {
        assert!(resolve(Some(Format::Pretty), false, false).colour);
        assert!(!resolve(Some(Format::Pretty), false, true).colour);
    }

    #[test]
    fn colour_is_off_for_plain_output() {
        assert!(!resolve(Some(Format::Plain), true, false).colour);
        assert!(!resolve(None, false, false).colour);
    }
}
