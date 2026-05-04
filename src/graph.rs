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
