//! Petgraph-backed dependency graph for `tickets deps-graph`.
//!
//! This module is the eventual replacement for the hand-rolled `graph`
//! module (removed in ticket fe17ed once `deps-graph` covers the same
//! ground). Unlike `graph`, which nests a ticket under its blockers,
//! `deps-graph` nests each ticket under everything IT blocks — so a root is
//! a ticket with no unresolved blocker, and its children are the tickets
//! it unblocks.
use std::collections::HashMap;

use anyhow::Result;
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};

use crate::application_types::WorkingDir;
use crate::domain_types::{Ticket, TicketId, TicketStatus};

/// How a single `blocked_by` id resolves, against the active ticket set
/// first and then, on a miss, one fallback lookup in `archived/`.
///
/// This is the single seam later tickets widen: archived non-done (xptucc)
/// gets an `Archived` stub variant, and a genuinely unknown id (t9p76e)
/// gets a `Missing` stub variant. `list --unblocked` (rm49xa) is expected
/// to call the same resolution function rather than reimplementing the
/// rule.
#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockerResolution {
    /// Blocker is an active ticket — render an edge into it regardless of
    /// its status. The graph shows active structure; only out-of-active-set
    /// ids get resolved away.
    Active(TicketId),
    /// Blocker isn't active, but was found archived with status `done` —
    /// satisfied. No edge, no node.
    ArchivedDone(TicketId),
}

/// Resolve a single `blocked_by` id: active tickets always resolve
/// (`Active`, any status); otherwise fall back to one lookup in the
/// archived set, which only resolves (`ArchivedDone`) when that archived
/// ticket's status is `done` — never a negation of the other statuses, so
/// this keeps resolving correctly as `TicketStatus` grows new variants.
fn resolve_blocker(
    active: &HashMap<TicketId, Ticket>,
    archived: &HashMap<TicketId, Ticket>,
    id: &TicketId,
) -> Option<BlockerResolution> {
    if active.contains_key(id) {
        return Some(BlockerResolution::Active(id.clone()));
    }
    archived.get(id).and_then(|ticket| {
        (ticket.front_matter.status == TicketStatus::Done)
            .then(|| BlockerResolution::ArchivedDone(id.clone()))
    })
}

/// The dependency graph: one node per active ticket, one edge per
/// satisfied `blocked_by` relationship, directed blocker -> dependent.
pub struct DepsGraph {
    graph: DiGraph<TicketId, ()>,
    index_of: HashMap<TicketId, NodeIndex>,
    tickets: HashMap<TicketId, Ticket>,
}

impl DepsGraph {
    pub fn build(dir: &WorkingDir) -> Result<Self> {
        let active = load_active(dir)?;
        let archived = load_dir(&dir.archived())?;
        Ok(Self::from_maps(active, &archived))
    }

    /// Construct from a pre-loaded active map, resolving `blocked_by`
    /// against `active` and then, on a miss, `archived`. Archived tickets
    /// are never added as nodes — only active tickets are.
    fn from_maps(active: HashMap<TicketId, Ticket>, archived: &HashMap<TicketId, Ticket>) -> Self {
        let mut graph = DiGraph::new();
        let mut index_of = HashMap::new();
        for id in active.keys() {
            index_of.insert(id.clone(), graph.add_node(id.clone()));
        }
        for (id, ticket) in &active {
            for blocker in &ticket.front_matter.blocked_by {
                if let Some(BlockerResolution::Active(blocker_id)) =
                    resolve_blocker(&active, archived, blocker)
                {
                    let from = index_of[&blocker_id];
                    let to = index_of[id];
                    graph.add_edge(from, to, ());
                }
            }
        }
        DepsGraph {
            graph,
            index_of,
            tickets: active,
        }
    }

    /// Root tickets: no in-graph blocker, ordered by `created_at` ascending.
    fn roots(&self) -> Vec<TicketId> {
        let mut roots: Vec<TicketId> = self
            .index_of
            .iter()
            .filter(|&(_, &ix)| {
                self.graph
                    .neighbors_directed(ix, Direction::Incoming)
                    .next()
                    .is_none()
            })
            .map(|(id, _)| id.clone())
            .collect();
        self.sort_by_created_at(&mut roots);
        roots
    }

    /// Tickets unblocked by `id`, ordered by `created_at` ascending.
    fn children(&self, id: &TicketId) -> Vec<TicketId> {
        let ix = self.index_of[id];
        let mut children: Vec<TicketId> = self
            .graph
            .neighbors_directed(ix, Direction::Outgoing)
            .map(|n| self.graph[n].clone())
            .collect();
        self.sort_by_created_at(&mut children);
        children
    }

    fn sort_by_created_at(&self, ids: &mut [TicketId]) {
        ids.sort_by_key(|id| self.tickets[id].front_matter.created_at);
    }

    fn label(&self, id: &TicketId) -> String {
        let t = &self.tickets[id];
        format!(
            "{}  {}  {}",
            id, t.front_matter.status, t.front_matter.title
        )
    }
}

/// Load every parseable ticket from `dir.all()`, keyed by id.
pub(crate) fn load_active(dir: &WorkingDir) -> Result<HashMap<TicketId, Ticket>> {
    load_dir(&dir.all())
}

/// Load every parseable `.md` ticket directly under `path`, keyed by id.
/// Shared by the active-set loader above and the archived-fallback lookup
/// in `DepsGraph::build`.
fn load_dir(path: &std::path::Path) -> Result<HashMap<TicketId, Ticket>> {
    let mut map = HashMap::new();
    if !path.exists() {
        return Ok(map);
    }
    for entry in std::fs::read_dir(path)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        if let Ok(raw) = std::fs::read_to_string(&path)
            && let Ok(ticket) = raw.parse::<Ticket>()
        {
            map.insert(ticket.front_matter.id.clone(), ticket);
        }
    }
    Ok(map)
}

/// Render the full forest as an indentation tree.
pub fn render_forest(graph: &DepsGraph) -> String {
    let mut output = String::new();
    for root in graph.roots() {
        render_node(graph, &root, "", "", &mut output);
    }
    output
}

fn render_node(
    graph: &DepsGraph,
    id: &TicketId,
    line_prefix: &str,
    child_base: &str,
    output: &mut String,
) {
    output.push_str(&format!("{}{}\n", line_prefix, graph.label(id)));

    let children = graph.children(id);
    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        let (connector, extension) = if is_last {
            ("└── ", "    ")
        } else {
            ("├── ", "│   ")
        };
        render_node(
            graph,
            child,
            &format!("{}{}", child_base, connector),
            &format!("{}{}", child_base, extension),
            output,
        );
    }
}

#[cfg(test)]
mod resolve_blocker_tests {
    use chrono::Utc;

    use super::*;
    use crate::domain_types::{FrontMatter, TicketStatus, TicketType};

    fn make_ticket(id: &str, status: TicketStatus) -> Ticket {
        Ticket {
            front_matter: FrontMatter {
                id: id.parse().unwrap(),
                title: "Test ticket".parse().unwrap(),
                r#type: TicketType::Task,
                status,
                tags: vec![],
                parent: None,
                blocked_by: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            body: String::new(),
        }
    }

    fn id(s: &str) -> TicketId {
        s.parse().unwrap()
    }

    #[test]
    fn active_blocker_resolves_active_regardless_of_status() {
        let active: HashMap<TicketId, Ticket> = [(id("a"), make_ticket("a", TicketStatus::Todo))]
            .into_iter()
            .collect();
        let archived = HashMap::new();

        let resolution = resolve_blocker(&active, &archived, &id("a"));

        assert_eq!(resolution, Some(BlockerResolution::Active(id("a"))));
    }

    #[test]
    fn archived_done_blocker_resolves_archived_done() {
        let active = HashMap::new();
        let archived: HashMap<TicketId, Ticket> = [(id("a"), make_ticket("a", TicketStatus::Done))]
            .into_iter()
            .collect();

        let resolution = resolve_blocker(&active, &archived, &id("a"));

        assert_eq!(resolution, Some(BlockerResolution::ArchivedDone(id("a"))));
    }

    #[test]
    fn archived_non_done_blocker_is_not_resolved() {
        let active = HashMap::new();
        let archived: HashMap<TicketId, Ticket> =
            [(id("a"), make_ticket("a", TicketStatus::Rejected))]
                .into_iter()
                .collect();

        let resolution = resolve_blocker(&active, &archived, &id("a"));

        assert_eq!(resolution, None);
    }

    #[test]
    fn blocker_missing_everywhere_is_not_resolved() {
        let active = HashMap::new();
        let archived = HashMap::new();

        let resolution = resolve_blocker(&active, &archived, &id("a"));

        assert_eq!(resolution, None);
    }
}
