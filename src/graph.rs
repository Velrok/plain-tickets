use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Result;

use crate::application_types::WorkingDir;
use crate::domain_types::{Ticket, TicketId};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Active,
    Archived,
    Missing,
}

pub struct DepGraph {
    active: HashMap<TicketId, Ticket>,
    archived: HashMap<TicketId, Ticket>,
    /// blocked_by edges: id → list of IDs that block it
    edges: HashMap<TicketId, Vec<TicketId>>,
}

impl DepGraph {
    pub fn build(dir: &WorkingDir) -> Result<Self> {
        let active = load_dir(&dir.all())?;
        let archived = load_dir(&dir.archived())?;
        Ok(Self::from_maps(active, archived))
    }

    /// Construct from pre-loaded maps — used in tests and by `build`.
    pub fn from_maps(
        active: HashMap<TicketId, Ticket>,
        archived: HashMap<TicketId, Ticket>,
    ) -> Self {
        let mut edges: HashMap<TicketId, Vec<TicketId>> = HashMap::new();
        for (id, ticket) in active.iter().chain(archived.iter()) {
            edges.insert(id.clone(), ticket.front_matter.blocked_by.clone());
        }
        DepGraph { active, archived, edges }
    }

    pub fn node_kind(&self, id: &TicketId) -> NodeKind {
        if self.active.contains_key(id) {
            NodeKind::Active
        } else if self.archived.contains_key(id) {
            NodeKind::Archived
        } else {
            NodeKind::Missing
        }
    }

    pub fn get_ticket(&self, id: &TicketId) -> Option<&Ticket> {
        self.active.get(id).or_else(|| self.archived.get(id))
    }

    /// IDs that block `id` (its `blocked_by` list).
    pub fn blockers(&self, id: &TicketId) -> &[TicketId] {
        self.edges.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn active_ids(&self) -> impl Iterator<Item = &TicketId> {
        self.active.keys()
    }

    /// Returns the set of ticket IDs that participate in a dependency cycle.
    pub fn cyclic_ids(&self) -> HashSet<TicketId> {
        let mut visited: HashSet<TicketId> = HashSet::new();
        let mut cyclic: HashSet<TicketId> = HashSet::new();

        for id in self.active.keys().chain(self.archived.keys()) {
            if !visited.contains(id) {
                let mut path: Vec<TicketId> = Vec::new();
                self.detect_cycles(id, &mut path, &mut visited, &mut cyclic);
            }
        }
        cyclic
    }

    fn detect_cycles(
        &self,
        id: &TicketId,
        path: &mut Vec<TicketId>,
        visited: &mut HashSet<TicketId>,
        cyclic: &mut HashSet<TicketId>,
    ) {
        if let Some(pos) = path.iter().position(|x| x == id) {
            for item in &path[pos..] {
                cyclic.insert(item.clone());
            }
            return;
        }
        if visited.contains(id) {
            return;
        }
        path.push(id.clone());
        for blocker in self.blockers(id) {
            self.detect_cycles(blocker, path, visited, cyclic);
        }
        path.pop();
        visited.insert(id.clone());
    }
}

/// Render the subtree rooted at `id` as an ASCII tree.
/// Returns a `String` with one line per node, terminated by `\n`.
pub fn render_tree(graph: &DepGraph, root: &TicketId) -> String {
    let mut output = String::new();
    let mut path: Vec<TicketId> = Vec::new();
    render_subtree(graph, root, "", "", &mut path, &mut output);
    output
}

/// Render all active tickets that have no active blockers as a forest.
pub fn render_forest(graph: &DepGraph) -> String {
    let mut roots: Vec<&TicketId> = graph
        .active_ids()
        .filter(|id| {
            graph
                .blockers(id)
                .iter()
                .all(|b| graph.node_kind(b) != NodeKind::Active)
        })
        .collect();
    roots.sort_by_key(|id| id.to_string());

    let mut output = String::new();
    for root in roots {
        let mut path: Vec<TicketId> = Vec::new();
        render_subtree(graph, root, "", "", &mut path, &mut output);
    }
    output
}

fn render_subtree(
    graph: &DepGraph,
    id: &TicketId,
    line_prefix: &str,
    child_base: &str,
    path: &mut Vec<TicketId>,
    output: &mut String,
) {
    if path.contains(id) {
        output.push_str(&format!("{}[cycle: {}]\n", line_prefix, id));
        return;
    }

    let label = format_label(graph, id);
    output.push_str(&format!("{}{}\n", line_prefix, label));

    path.push(id.clone());
    let blockers = graph.blockers(id);
    for (i, blocker) in blockers.iter().enumerate() {
        let is_last = i == blockers.len() - 1;
        let (connector, extension) = if is_last {
            ("└── ", "    ")
        } else {
            ("├── ", "│   ")
        };
        render_subtree(
            graph,
            blocker,
            &format!("{}{}", child_base, connector),
            &format!("{}{}", child_base, extension),
            path,
            output,
        );
    }
    path.pop();
}

fn format_label(graph: &DepGraph, id: &TicketId) -> String {
    match graph.node_kind(id) {
        NodeKind::Missing => format!("[missing: {}]", id),
        NodeKind::Active => {
            let t = graph.get_ticket(id).unwrap();
            format!("{}  {}  {}", id, t.front_matter.status, t.front_matter.title)
        }
        NodeKind::Archived => {
            let t = graph.get_ticket(id).unwrap();
            format!("{}  {}  {}  [archived]", id, t.front_matter.status, t.front_matter.title)
        }
    }
}

fn load_dir(dir: &Path) -> Result<HashMap<TicketId, Ticket>> {
    let mut map = HashMap::new();
    if !dir.exists() {
        return Ok(map);
    }
    for entry in std::fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(ticket) = raw.parse::<Ticket>() {
                map.insert(ticket.front_matter.id.clone(), ticket);
            }
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;

    use super::*;
    use crate::domain_types::{FrontMatter, Ticket, TicketStatus, TicketType};

    fn make_ticket(id: &str, blocked_by: Vec<&str>) -> Ticket {
        Ticket {
            front_matter: FrontMatter {
                id: id.parse().unwrap(),
                title: "Test ticket".parse().unwrap(),
                r#type: TicketType::Task,
                status: TicketStatus::Todo,
                tags: vec![],
                parent: None,
                blocked_by: blocked_by.iter().map(|s| s.parse().unwrap()).collect(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            body: String::new(),
        }
    }

    fn id(s: &str) -> TicketId {
        s.parse().unwrap()
    }

    // ── render_tree ───────────────────────────────────────────────────────────

    #[test]
    fn render_tree_single_node() {
        let graph = DepGraph::from_maps(
            [(id("abc123"), make_ticket("abc123", vec![]))].into_iter().collect(),
            HashMap::new(),
        );
        let out = render_tree(&graph, &id("abc123"));
        assert_eq!(out, "abc123  todo  Test ticket\n");
    }

    #[test]
    fn render_tree_with_one_child() {
        let graph = DepGraph::from_maps(
            [
                (id("a"), make_ticket("a", vec!["b"])),
                (id("b"), make_ticket("b", vec![])),
            ]
            .into_iter()
            .collect(),
            HashMap::new(),
        );
        let out = render_tree(&graph, &id("a"));
        assert_eq!(
            out,
            "a  todo  Test ticket\n└── b  todo  Test ticket\n"
        );
    }

    #[test]
    fn render_tree_with_two_children_uses_branch_chars() {
        let graph = DepGraph::from_maps(
            [
                (id("a"), make_ticket("a", vec!["b", "c"])),
                (id("b"), make_ticket("b", vec![])),
                (id("c"), make_ticket("c", vec![])),
            ]
            .into_iter()
            .collect(),
            HashMap::new(),
        );
        let out = render_tree(&graph, &id("a"));
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "a  todo  Test ticket");
        assert!(lines[1].starts_with("├── "), "first child: {}", lines[1]);
        assert!(lines[2].starts_with("└── "), "last child: {}", lines[2]);
    }

    #[test]
    fn render_tree_cycle_node() {
        // a blocked_by b, b blocked_by a
        let graph = DepGraph::from_maps(
            [
                (id("a"), make_ticket("a", vec!["b"])),
                (id("b"), make_ticket("b", vec!["a"])),
            ]
            .into_iter()
            .collect(),
            HashMap::new(),
        );
        let out = render_tree(&graph, &id("a"));
        assert!(out.contains("[cycle: a]"), "expected cycle label: {}", out);
    }

    #[test]
    fn render_tree_missing_node() {
        let graph = DepGraph::from_maps(
            [(id("a"), make_ticket("a", vec!["ghost"]))].into_iter().collect(),
            HashMap::new(),
        );
        let out = render_tree(&graph, &id("a"));
        assert!(out.contains("[missing: ghost]"), "expected missing label: {}", out);
    }

    #[test]
    fn render_tree_archived_node_has_label() {
        let graph = DepGraph::from_maps(
            [(id("a"), make_ticket("a", vec!["b"]))].into_iter().collect(),
            [(id("b"), make_ticket("b", vec![]))].into_iter().collect(),
        );
        let out = render_tree(&graph, &id("a"));
        assert!(out.contains("[archived]"), "expected archived label: {}", out);
    }

    #[test]
    fn render_forest_includes_unblocked_roots() {
        // c is blocked by active ticket b → not a root
        // a has no active blockers → root
        // b has no active blockers → root
        let graph = DepGraph::from_maps(
            [
                (id("a"), make_ticket("a", vec![])),
                (id("b"), make_ticket("b", vec![])),
                (id("c"), make_ticket("c", vec!["b"])),
            ]
            .into_iter()
            .collect(),
            HashMap::new(),
        );
        let out = render_forest(&graph);
        // a and b are roots; c is not
        assert!(out.contains("a  todo  Test ticket"));
        assert!(out.contains("b  todo  Test ticket"));
        // c appears as a child of b, not as a root line without prefix
        let lines: Vec<&str> = out.lines().collect();
        assert!(!lines.iter().any(|l| *l == "c  todo  Test ticket"),
            "c should not be a root: {}", out);
    }

    // ── node_kind ─────────────────────────────────────────────────────────────

    #[test]
    fn node_kind_active_ticket() {
        let graph = DepGraph::from_maps(
            [(id("abc123"), make_ticket("abc123", vec![]))].into_iter().collect(),
            HashMap::new(),
        );
        assert_eq!(graph.node_kind(&id("abc123")), NodeKind::Active);
    }

    #[test]
    fn node_kind_archived_ticket() {
        let graph = DepGraph::from_maps(
            HashMap::new(),
            [(id("abc123"), make_ticket("abc123", vec![]))].into_iter().collect(),
        );
        assert_eq!(graph.node_kind(&id("abc123")), NodeKind::Archived);
    }

    #[test]
    fn node_kind_missing_for_unknown_id() {
        let graph = DepGraph::from_maps(HashMap::new(), HashMap::new());
        assert_eq!(graph.node_kind(&id("unknown")), NodeKind::Missing);
    }

    #[test]
    fn blockers_returns_blocked_by_list() {
        let graph = DepGraph::from_maps(
            [
                (id("a"), make_ticket("a", vec!["b", "c"])),
                (id("b"), make_ticket("b", vec![])),
                (id("c"), make_ticket("c", vec![])),
            ]
            .into_iter()
            .collect(),
            HashMap::new(),
        );
        let blockers = graph.blockers(&id("a"));
        assert_eq!(blockers.len(), 2);
        assert!(blockers.contains(&id("b")));
        assert!(blockers.contains(&id("c")));
    }

    #[test]
    fn blockers_empty_for_unknown_id() {
        let graph = DepGraph::from_maps(HashMap::new(), HashMap::new());
        assert_eq!(graph.blockers(&id("unknown")), &[]);
    }

    #[test]
    fn cyclic_ids_detects_simple_cycle() {
        // a blocked_by b, b blocked_by a
        let graph = DepGraph::from_maps(
            [
                (id("a"), make_ticket("a", vec!["b"])),
                (id("b"), make_ticket("b", vec!["a"])),
            ]
            .into_iter()
            .collect(),
            HashMap::new(),
        );
        let cycles = graph.cyclic_ids();
        assert!(cycles.contains(&id("a")), "a should be in cycle");
        assert!(cycles.contains(&id("b")), "b should be in cycle");
    }

    #[test]
    fn cyclic_ids_empty_for_acyclic_graph() {
        let graph = DepGraph::from_maps(
            [
                (id("a"), make_ticket("a", vec!["b"])),
                (id("b"), make_ticket("b", vec!["c"])),
                (id("c"), make_ticket("c", vec![])),
            ]
            .into_iter()
            .collect(),
            HashMap::new(),
        );
        assert!(graph.cyclic_ids().is_empty());
    }

    #[test]
    fn missing_blocker_is_flagged_as_missing() {
        let graph = DepGraph::from_maps(
            [(id("a"), make_ticket("a", vec!["ghost"]))].into_iter().collect(),
            HashMap::new(),
        );
        assert_eq!(graph.node_kind(&id("ghost")), NodeKind::Missing);
    }

    #[test]
    fn archived_blocker_is_flagged_as_archived() {
        let graph = DepGraph::from_maps(
            [(id("a"), make_ticket("a", vec!["b"]))].into_iter().collect(),
            [(id("b"), make_ticket("b", vec![]))].into_iter().collect(),
        );
        assert_eq!(graph.node_kind(&id("b")), NodeKind::Archived);
    }
}
