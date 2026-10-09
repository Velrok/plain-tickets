use crate::domain::tickets::Ticket;
use std::fmt;

/// One tab-separated line per ticket: `<id>\t<status>\t<type>\t<title>`.
pub struct TicketCliLinePresenter<'a>(pub &'a Ticket);

impl fmt::Display for TicketCliLinePresenter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = self.0;
        write!(f, "{}\t{}\t{}\t{}", t.id, t.status, t.r#type.as_deref().unwrap_or(""), t.title)
    }
}

/// The ticket as stored: front matter, then body. Has no trailing newline, so
/// printing it with `println!` ends the output with exactly one.
pub struct TicketCliDetailPresenter<'a>(pub &'a Ticket);

impl fmt::Display for TicketCliDetailPresenter<'_> {
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

    #[test]
    fn presents_id_status_type_and_title_separated_by_tabs() {
        let draft = NewTicket {
            title: "Fix it".into(),
            r#type: None,
            status: Some("in progress".parse().unwrap()),
            tags: vec![],
            parent: None,
            blocked_by: vec![],
            body: String::new(),
        };
        let config: Config = r#"statuses = ["todo", "in progress"]"#.parse().unwrap();
        let ticket = Ticket::new(draft, &config, ID(0xab), 0).unwrap();
        assert_eq!(
            TicketCliLinePresenter(&ticket).to_string(),
            "00000000000000ab\tin progress\t\tFix it"
        );
    }
}
