use super::id::ID;
use super::status::Status;
use crate::config::Config;
use serde::{Deserialize, Serialize};

type Tag = String;
type Title = String;
type Type = String;

/// The body is stored after the YAML front matter, so serde skips it.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    pub id: ID,

    pub title: Title,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    pub status: Status,

    pub parent: Option<ID>,
    pub blocked_by: Vec<ID>,

    pub tags: Vec<Tag>,
    pub created_at: u64,
    pub updated_at: u64,

    #[serde(skip)]
    pub body: String,
}

/// User-supplied fields for a new ticket; everything else is derived.
pub struct NewTicket {
    pub title: Title,
    pub r#type: Option<Type>,
    /// Defaults to the config's first status.
    pub status: Option<Status>,
    pub tags: Vec<Tag>,
    pub parent: Option<ID>,
    pub blocked_by: Vec<ID>,
    pub body: String,
}

impl Ticket {
    pub fn new(draft: NewTicket, config: &Config, id: ID, now: u64) -> Result<Ticket, String> {
        let title = draft.title.trim().to_string();
        if title.is_empty() || title.contains('\n') {
            return Err("title must be a single non-empty line".to_string());
        }
        let status = draft
            .status
            .unwrap_or_else(|| config.default_status().clone());
        if !config.allows(&status) {
            return Err(format!("status '{status}' is not in the config"));
        }
        let mut tags: Vec<Tag> = Vec::new();
        for tag in draft.tags {
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }
        Ok(Ticket {
            id,
            title,
            r#type: draft.r#type,
            status,
            parent: draft.parent,
            blocked_by: draft.blocked_by,
            tags,
            created_at: now,
            updated_at: now,
            body: draft.body,
        })
    }
}

impl std::str::FromStr for Ticket {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("---\n").unwrap_or(s);
        let end = s
            .find("\n---\n")
            .ok_or_else(|| "missing front matter closing delimiter".to_string())?;
        let yaml = &s[..end];
        let body = s[end + 5..].trim_start_matches('\n').to_string();
        let mut ticket: Ticket =
            serde_yaml::from_str(yaml).map_err(|e| format!("invalid front matter: {e}"))?;
        ticket.body = body;
        Ok(ticket)
    }
}

impl std::fmt::Display for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let yaml = serde_yaml::to_string(self).expect("Ticket serialisation is infallible");
        if self.body.is_empty() {
            write!(f, "---\n{}---\n", yaml)
        } else {
            write!(f, "---\n{}---\n\n{}", yaml, self.body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ticket(body: &str) -> Ticket {
        Ticket {
            id: ID(7),
            title: "Fix it".into(),
            r#type: Some("bug".into()),
            status: Status::Custom("open".into()),
            parent: None,
            blocked_by: vec![ID(1), ID(2)],
            body: body.into(),
            tags: vec!["a".into()],
            created_at: 1,
            updated_at: 2,
        }
    }

    #[test]
    fn round_trips_with_body() {
        let text = ticket("Some body\n").to_string();
        let parsed: Ticket = text.parse().unwrap();
        assert_eq!(parsed.to_string(), text);
        assert_eq!(parsed.body, "Some body\n");
    }

    #[test]
    fn round_trips_without_body() {
        let text = ticket("").to_string();
        assert!(text.ends_with("---\n"));
        let parsed: Ticket = text.parse().unwrap();
        assert_eq!(parsed.to_string(), text);
    }

    #[test]
    fn stores_ids_as_sixteen_digit_hex() {
        let mut t = ticket("");
        t.id = ID(0xab);
        let text = t.to_string();
        assert!(text.contains("id: 00000000000000ab"), "{text}");
    }

    #[test]
    fn round_trips_digit_only_hex_ids_in_references() {
        let mut t = ticket("");
        t.id = ID(0x1234);
        t.parent = Some(ID(0x99));
        t.blocked_by = vec![ID(0x10), ID(0xe)];
        let parsed: Ticket = t.to_string().parse().unwrap();
        assert_eq!(parsed.id, ID(0x1234));
        assert_eq!(parsed.parent, Some(ID(0x99)));
        assert_eq!(parsed.blocked_by, vec![ID(0x10), ID(0xe)]);
    }

    #[test]
    fn omits_type_from_front_matter_when_unset() {
        let mut t = ticket("");
        t.r#type = None;
        let text = t.to_string();
        assert!(!text.contains("type"), "{text}");
        let parsed: Ticket = text.parse().unwrap();
        assert_eq!(parsed.r#type, None);
    }

    fn draft(title: &str) -> NewTicket {
        NewTicket {
            title: title.into(),
            r#type: None,
            status: None,
            tags: vec![],
            parent: None,
            blocked_by: vec![],
            body: String::new(),
        }
    }

    #[test]
    fn new_ticket_gets_the_configs_default_status_and_timestamps() {
        let config: Config = r#"statuses = ["backlog", "todo"]"#.parse().unwrap();
        let t = Ticket::new(draft("Fix it"), &config, ID(5), 42).unwrap();
        assert_eq!(t.status, Status::Custom("backlog".into()));
        assert_eq!((t.id, t.created_at, t.updated_at), (ID(5), 42, 42));
    }

    #[test]
    fn new_ticket_uses_a_requested_status_if_the_config_allows_it() {
        let config = Config::default();
        for status in [Status::Done, Status::Custom("in progress".into())] {
            let mut d = draft("x");
            d.status = Some(status.clone());
            let t = Ticket::new(d, &config, ID(1), 0).unwrap();
            assert_eq!(t.status, status);
        }
    }

    #[test]
    fn new_ticket_rejects_a_status_the_config_does_not_list() {
        let mut d = draft("x");
        d.status = Some(Status::Custom("blocked".into()));
        let err = Ticket::new(d, &Config::default(), ID(1), 0).unwrap_err();
        assert!(err.contains("blocked"), "{err}");
    }

    #[test]
    fn new_ticket_drops_duplicate_tags_keeping_first_order() {
        let mut d = draft("x");
        d.tags = vec!["b".into(), "a".into(), "b".into()];
        let t = Ticket::new(d, &Config::default(), ID(1), 0).unwrap();
        assert_eq!(t.tags, ["b", "a"]);
    }

    #[test]
    fn new_ticket_rejects_blank_or_multiline_titles() {
        for title in ["", "   ", "a\nb"] {
            let r = Ticket::new(draft(title), &Config::default(), ID(1), 0);
            assert!(r.is_err(), "{title:?}");
        }
    }

    #[test]
    fn rejects_missing_closing_delimiter() {
        assert!("---\nid: 1\n".parse::<Ticket>().is_err());
    }
}
