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
use crate::domain_types::{Ticket, TicketId};

/// How a single `blocked_by` id resolves against the active ticket set.
///
/// This slice only ever produces `Active` — a `blocked_by` id that isn't an
/// active ticket (archived, or missing entirely) is dropped without a node
/// or edge for now. This is the single seam later tickets widen: archived
/// done (7se1mu) and archived non-done (xptucc) fall back to a lookup in
/// `archived/`, and a genuinely unknown id (t9p76e) becomes a `Missing`
/// stub. `list --unblocked` (rm49xa) is expected to call the same
/// resolution function rather than reimplementing the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockerResolution {
    /// Blocker is an active ticket — render an edge into it.
    Active(TicketId),
}

fn resolve_blocker(active: &HashMap<TicketId, Ticket>, id: &TicketId) -> Option<BlockerResolution> {
    active
        .contains_key(id)
        .then(|| BlockerResolution::Active(id.clone()))
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
        Ok(Self::from_active(active))
    }

    fn from_active(active: HashMap<TicketId, Ticket>) -> Self {
        let mut graph = DiGraph::new();
        let mut index_of = HashMap::new();
        for id in active.keys() {
            index_of.insert(id.clone(), graph.add_node(id.clone()));
        }
        for (id, ticket) in &active {
            for blocker in &ticket.front_matter.blocked_by {
                if let Some(BlockerResolution::Active(blocker_id)) =
                    resolve_blocker(&active, blocker)
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

fn load_active(dir: &WorkingDir) -> Result<HashMap<TicketId, Ticket>> {
    let mut map = HashMap::new();
    let all_dir = dir.all();
    if !all_dir.exists() {
        return Ok(map);
    }
    for entry in std::fs::read_dir(&all_dir)?.flatten() {
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
