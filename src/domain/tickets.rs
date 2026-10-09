use super::id::ID;
use super::status::Status;
use super::timestamp::Timestamp;
use crate::config::Config;
use serde::{Deserialize, Serialize};

type Tag = String;
type Title = String;
type Type = String;

/// The body is stored after the YAML front matter, so serde skips it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ticket {
    pub id: ID,

    pub title: Title,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    pub status: Status,

    pub parent: Option<ID>,
    pub blocked_by: Vec<ID>,

    pub tags: Vec<Tag>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,

    #[serde(skip)]
    pub body: String,
}

/// Fields for `set`; `None` leaves a field as it is.
#[derive(Default)]
pub struct SetFields {
    pub title: Option<Title>,
    pub status: Option<Status>,
    /// `Some(None)` clears the field.
    pub r#type: Option<Option<Type>>,
    pub parent: Option<Option<ID>>,
}

impl SetFields {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.status.is_none()
            && self.r#type.is_none()
            && self.parent.is_none()
    }
}

/// A single edit to an existing ticket, applied by `Ticket::apply`.
pub enum Change {
    AppendNote(String),
    AddTags(Vec<Tag>),
    RemoveTags(Vec<Tag>),
    AddBlockers(Vec<ID>),
    RemoveBlockers(Vec<ID>),
    Set(SetFields),
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

/// Trims each tag and drops duplicates, keeping first-seen order. Tags must be
/// single non-empty lines.
fn clean_tags(tags: Vec<Tag>) -> Result<Vec<Tag>, String> {
    let mut cleaned: Vec<Tag> = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() || tag.contains('\n') {
            return Err("a tag must be a single non-empty line".to_string());
        }
        if !cleaned.iter().any(|t| t == tag) {
            cleaned.push(tag.to_string());
        }
    }
    Ok(cleaned)
}

/// Trims the title; it must be a single non-empty line.
fn clean_title(title: &str) -> Result<Title, String> {
    let title = title.trim();
    if title.is_empty() || title.contains('\n') {
        return Err("title must be a single non-empty line".to_string());
    }
    Ok(title.to_string())
}

impl Ticket {
    /// A copy of this ticket with `change` applied. Not validated; pass the
    /// result to `revise`.
    pub fn apply(&self, change: Change) -> Ticket {
        let mut t = self.clone();
        match change {
            Change::AddTags(tags) => t.tags.extend(tags),
            Change::AddBlockers(ids) => t.blocked_by.extend(ids),
            Change::RemoveBlockers(ids) => t.blocked_by.retain(|b| !ids.contains(b)),
            Change::Set(f) => {
                t.title = f.title.unwrap_or(t.title);
                t.status = f.status.unwrap_or(t.status);
                t.r#type = f.r#type.unwrap_or(t.r#type);
                t.parent = f.parent.unwrap_or(t.parent);
            }
            Change::RemoveTags(tags) => t.tags.retain(|x| !tags.iter().any(|r| r.trim() == x)),
            Change::AppendNote(text) => {
                t.body.truncate(t.body.trim_end().len());
                if !t.body.is_empty() {
                    t.body.push_str("\n\n");
                }
                t.body.push_str(&text);
                t.body.push('\n');
            }
        }
        t
    }

    /// The edited ticket as it should be saved, or `None` if nothing but
    /// `updated_at` changed. `updated_at` is always set here.
    pub fn revise(
        &self,
        mut edited: Ticket,
        config: &Config,
        now: Timestamp,
    ) -> Result<Option<Ticket>, String> {
        if edited.id != self.id {
            return Err("id cannot be changed".to_string());
        }
        if edited.created_at != self.created_at {
            return Err("created_at cannot be changed".to_string());
        }
        edited.title = clean_title(&edited.title)?;
        edited.tags = clean_tags(std::mem::take(&mut edited.tags))?;
        if edited.blocked_by.contains(&self.id) {
            return Err("a ticket cannot block itself".to_string());
        }
        if edited.parent == Some(self.id) {
            return Err("a ticket cannot be its own parent".to_string());
        }
        let mut seen = Vec::new();
        edited.blocked_by.retain(|id| {
            let fresh = !seen.contains(id);
            seen.push(*id);
            fresh
        });
        if !config.allows(&edited.status) {
            return Err(format!("status '{}' is not in the config", edited.status));
        }
        edited.updated_at = self.updated_at;
        if edited.to_string() == self.to_string() {
            return Ok(None);
        }
        edited.updated_at = now;
        Ok(Some(edited))
    }

    pub fn new(
        draft: NewTicket,
        config: &Config,
        id: ID,
        now: Timestamp,
    ) -> Result<Ticket, String> {
        let title = clean_title(&draft.title)?;
        let status = draft
            .status
            .unwrap_or_else(|| config.default_status().clone());
        if !config.allows(&status) {
            return Err(format!("status '{status}' is not in the config"));
        }
        let tags = clean_tags(draft.tags)?;
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
        let (yaml, body) = match s.find("\n---\n") {
            Some(end) => (&s[..end], s[end + 5..].trim_start_matches('\n')),
            None => (
                s.strip_suffix("\n---")
                    .ok_or_else(|| "missing front matter closing delimiter".to_string())?,
                "",
            ),
        };
        let body = body.to_string();
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
            created_at: ts("2026-04-30T19:00:00.123Z"),
            updated_at: ts("2026-05-01T08:30:00.000Z"),
        }
    }

    fn ts(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    #[test]
    fn stores_timestamps_as_iso_8601_strings() {
        let text = ticket("").to_string();
        assert!(
            text.contains("created_at: 2026-04-30T19:00:00.123Z"),
            "{text}"
        );
        assert!(
            text.contains("updated_at: 2026-05-01T08:30:00.000Z"),
            "{text}"
        );
        let parsed: Ticket = text.parse().unwrap();
        assert_eq!(parsed.created_at, ts("2026-04-30T19:00:00.123Z"));
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
    fn accepts_a_closing_delimiter_at_the_end_of_the_file() {
        let text = ticket("").to_string();
        let trimmed = text.strip_suffix('\n').unwrap();
        let parsed: Ticket = trimmed.parse().unwrap();
        assert_eq!(parsed.to_string(), text);
        assert_eq!(parsed.body, "");
    }

    #[test]
    fn stores_ids_as_thirteen_character_base36() {
        let mut t = ticket("");
        t.id = ID(0xab);
        let text = t.to_string();
        assert!(text.contains("id: 000000000004r"), "{text}");
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
        let t = Ticket::new(
            draft("Fix it"),
            &config,
            ID(5),
            ts("2026-04-30T19:00:00.123Z"),
        )
        .unwrap();
        assert_eq!(t.status, Status::Custom("backlog".into()));
        let now = ts("2026-04-30T19:00:00.123Z");
        assert_eq!((t.id, t.created_at, t.updated_at), (ID(5), now, now));
    }

    #[test]
    fn new_ticket_uses_a_requested_status_if_the_config_allows_it() {
        let config = Config::default();
        for status in [Status::Done, Status::Custom("in progress".into())] {
            let mut d = draft("x");
            d.status = Some(status.clone());
            let t = Ticket::new(d, &config, ID(1), ts("2026-04-30T19:00:00.000Z")).unwrap();
            assert_eq!(t.status, status);
        }
    }

    #[test]
    fn new_ticket_rejects_a_status_the_config_does_not_list() {
        let mut d = draft("x");
        d.status = Some(Status::Custom("blocked".into()));
        let err =
            Ticket::new(d, &Config::default(), ID(1), ts("2026-04-30T19:00:00.000Z")).unwrap_err();
        assert!(err.contains("blocked"), "{err}");
    }

    #[test]
    fn new_ticket_drops_duplicate_tags_keeping_first_order() {
        let mut d = draft("x");
        d.tags = vec!["b".into(), "a".into(), "b".into()];
        let t = Ticket::new(d, &Config::default(), ID(1), ts("2026-04-30T19:00:00.000Z")).unwrap();
        assert_eq!(t.tags, ["b", "a"]);
    }

    #[test]
    fn new_ticket_rejects_blank_or_multiline_titles() {
        for title in ["", "   ", "a\nb"] {
            let r = Ticket::new(
                draft(title),
                &Config::default(),
                ID(1),
                ts("2026-04-30T19:00:00.000Z"),
            );
            assert!(r.is_err(), "{title:?}");
        }
    }

    #[test]
    fn rejects_missing_closing_delimiter() {
        assert!("---\nid: 1\n".parse::<Ticket>().is_err());
    }
}
