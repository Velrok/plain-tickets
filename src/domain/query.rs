use super::id::ID;
use super::status::Status;
use super::tickets::Ticket;
use crate::config::Config;
use std::collections::HashSet;

/// Which tickets `list` shows. Empty or `None` fields match everything.
/// `status` and `tags` match any of their values.
#[derive(Default)]
pub struct Filter {
    pub status: Vec<Status>,
    pub r#type: Option<String>,
    pub tags: Vec<String>,
    pub parent: Option<ID>,
    /// Has a blocker that exists and is not done or rejected.
    pub blocked: bool,
    /// Not done or rejected itself, and not blocked.
    pub ready: bool,
}

impl Filter {
    /// The matching tickets in display order: configured statuses, then
    /// `Done`, then `Rejected`; oldest first within a status.
    pub fn select(&self, mut tickets: Vec<Ticket>, config: &Config) -> Vec<Ticket> {
        let open: HashSet<ID> = tickets
            .iter()
            .filter(|t| !t.status.is_terminal())
            .map(|t| t.id)
            .collect();
        let is_blocked = |t: &Ticket| t.blocked_by.iter().any(|b| open.contains(b));

        tickets.retain(|t| {
            (!self.blocked || is_blocked(t))
                && (!self.ready || (!t.status.is_terminal() && !is_blocked(t)))
                && (self.status.is_empty() || self.status.contains(&t.status))
                && (self.parent.is_none() || t.parent == self.parent)
                && (self.r#type.is_none() || t.r#type == self.r#type)
                && (self.tags.is_empty() || t.tags.iter().any(|x| self.tags.contains(x)))
        });
        tickets.sort_by_key(|t| (config.sort_key(&t.status), t.created_at, t.id));
        tickets
    }
}
