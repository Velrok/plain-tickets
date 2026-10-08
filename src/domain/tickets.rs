use super::id::ID;
use serde::{Deserialize, Serialize};

type Tag = String;
type Title = String;
type Type = String;
type Status = String;

/// The body is stored after the YAML front matter, so serde skips it.
#[derive(Serialize, Deserialize)]
struct Ticket {
    id: ID,

    title: Title,
    r#type: Type,
    status: Status,

    parent: Option<ID>,
    blocked_by: Vec<ID>,

    tags: Vec<Tag>,
    created_at: u64,
    updated_at: u64,

    #[serde(skip)]
    body: String,
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
            r#type: "bug".into(),
            status: "open".into(),
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
    fn rejects_missing_closing_delimiter() {
        assert!("---\nid: 1\n".parse::<Ticket>().is_err());
    }
}
