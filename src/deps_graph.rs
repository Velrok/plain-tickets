//! Petgraph-backed dependency graph for `tickets deps-graph`.
//!
//! This module is the eventual replacement for the hand-rolled `graph`
//! module (removed in ticket fe17ed once `deps-graph` covers the same
//! ground). Unlike `graph`, which nests a ticket under its blockers,
//! `deps-graph` nests each ticket under everything IT blocks — so a root is
//! a ticket with no unresolved blocker, and its children are the tickets
//! it unblocks.
use std::collections::{HashMap, HashSet};

use anyhow::Result;
use petgraph::Direction;
use petgraph::algo::{tarjan_scc, toposort};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::application_types::WorkingDir;
use crate::domain_types::{Ticket, TicketId, TicketStatus};

/// How a single `blocked_by` id resolves, against the active ticket set
/// first and then, on a miss, one fallback lookup in `archived/`.
///
/// This is the single seam widened by 7se1mu, xptucc and t9p76e.
/// `resolve_blocker` is total — every id lands in exactly one variant, so
/// there is no "not yet resolved" case left to model as `None`. `list
/// --unblocked` (3p7tpt/rm49xa) already calls [`resolve_blocker`] via
/// [`is_unblocked`], so the archived and missing fallbacks are shared with
/// it for free — extend `resolve_blocker` and `blocker_satisfied` rather
/// than reimplementing the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockerResolution {
    /// Blocker is an active ticket — render an edge into it regardless of
    /// its status. The graph shows active structure; only out-of-active-set
    /// ids get resolved away.
    Active(TicketId),
    /// Blocker isn't active, but was found archived with status `done` —
    /// satisfied. No edge, no node.
    ArchivedDone(TicketId),
    /// Blocker isn't active, and was found archived with any other status
    /// (e.g. `rejected`) — NOT satisfied. Renders as a distinct stub root
    /// naming both the id and its status (xptucc).
    ArchivedStub(TicketId, TicketStatus),
    /// Blocker isn't active and isn't in the archive either — a data
    /// problem (bad id/typo), not an intentional filter. NOT satisfied.
    /// Renders as a distinct stub root naming the id (t9p76e).
    Missing(TicketId),
}

/// Resolve a single `blocked_by` id: active tickets always resolve
/// (`Active`, any status); otherwise fall back to one lookup in the
/// archived set. An archived hit resolves to `ArchivedDone` only when that
/// ticket's status is `done` — never a negation of the other statuses, so
/// this keeps resolving correctly as `TicketStatus` grows new variants —
/// and to `ArchivedStub` for any other status. No hit anywhere resolves to
/// `Missing`. Total: every id lands in exactly one variant.
fn resolve_blocker(
    active: &HashMap<TicketId, Ticket>,
    archived: &HashMap<TicketId, Ticket>,
    id: &TicketId,
) -> BlockerResolution {
    if active.contains_key(id) {
        return BlockerResolution::Active(id.clone());
    }
    match archived.get(id) {
        Some(ticket) if ticket.front_matter.status == TicketStatus::Done => {
            BlockerResolution::ArchivedDone(id.clone())
        }
        Some(ticket) => {
            BlockerResolution::ArchivedStub(id.clone(), ticket.front_matter.status.clone())
        }
        None => BlockerResolution::Missing(id.clone()),
    }
}

/// Whether a resolved blocker counts as satisfied — i.e. no longer blocks
/// whatever depends on it. Only `done` ever satisfies a dependency; the
/// `Active` arm is deliberately not an exhaustive match on `TicketStatus` —
/// it's a positive equality check, so a newly added status (e.g. `review`)
/// keeps blocking by default rather than needing this to be touched.
/// `ArchivedStub` and `Missing` are both always unsatisfied — the dependency
/// stays and is rendered as a stub instead.
fn blocker_satisfied(active: &HashMap<TicketId, Ticket>, resolution: &BlockerResolution) -> bool {
    match resolution {
        BlockerResolution::Active(id) => active[id].front_matter.status == TicketStatus::Done,
        // Already established at resolution time: `resolve_blocker` only
        // ever returns this variant when the archived ticket's status is
        // `done`.
        BlockerResolution::ArchivedDone(_) => true,
        BlockerResolution::ArchivedStub(_, _) => false,
        BlockerResolution::Missing(_) => false,
    }
}

/// Whether every one of `ticket`'s blockers is satisfied (or there are
/// none). This is the seam `list --unblocked` and `deps-graph` both defer
/// to for "is this dependency resolved" — extend `resolve_blocker` and
/// `blocker_satisfied` above rather than reimplementing this check.
pub(crate) fn is_unblocked(
    active: &HashMap<TicketId, Ticket>,
    archived: &HashMap<TicketId, Ticket>,
    ticket: &Ticket,
) -> bool {
    ticket
        .front_matter
        .blocked_by
        .iter()
        .all(|blocker_id| blocker_satisfied(active, &resolve_blocker(active, archived, blocker_id)))
}

/// How a stub node (an out-of-active-set blocker that does not satisfy the
/// dependency) should render. Carries just enough to label itself — stub
/// nodes are never looked up in `tickets`, since they aren't active tickets.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StubKind {
    /// Found archived, but not `done` (xptucc). Carries the status so the
    /// stub can name it.
    Archived(TicketStatus),
    /// Not found anywhere (t9p76e).
    Missing,
}

impl StubKind {
    /// Render the stub root line for `id`. The two forms are deliberately
    /// visually distinct from each other and from a normal ticket line.
    fn label(&self, id: &TicketId) -> String {
        match self {
            StubKind::Archived(status) => format!("[archived: {id} {status}]"),
            StubKind::Missing => format!("[missing: {id}]"),
        }
    }
}

/// The dependency graph: one node per active ticket plus one node per
/// out-of-active-set blocker that does not satisfy its dependency (a
/// "stub" — see [`StubKind`]), and one edge per unsatisfied `blocked_by`
/// relationship, directed blocker -> dependent. Stub nodes let an
/// unsatisfied archived/missing blocker render as a root naming itself,
/// with the dependent nested underneath — the same "nest under every
/// blocker" shape used for real diamonds.
pub struct DepsGraph {
    graph: DiGraph<TicketId, ()>,
    index_of: HashMap<TicketId, NodeIndex>,
    tickets: HashMap<TicketId, Ticket>,
    stubs: HashMap<TicketId, StubKind>,
    /// Sort key for every node, real or stub. Real tickets use their own
    /// `created_at`; a stub has none of its own, so it takes the earliest
    /// `created_at` among the dependents it blocks — it sorts as of the
    /// point the dependency was first noticed.
    sort_keys: HashMap<TicketId, chrono::DateTime<chrono::Utc>>,
}

impl DepsGraph {
    pub fn build(dir: &WorkingDir) -> Result<Self> {
        let active = load_active(dir)?;
        let archived = load_archived(dir)?;
        Ok(Self::from_maps(active, &archived))
    }

    /// Construct from a pre-loaded active map, resolving `blocked_by`
    /// against `active` and then, on a miss, `archived`. Archived tickets
    /// are never added as full nodes — only active tickets are — but an
    /// unsatisfied out-of-active-set blocker (archived non-done, or found
    /// nowhere) gets a stub node so it renders as a root naming itself.
    fn from_maps(active: HashMap<TicketId, Ticket>, archived: &HashMap<TicketId, Ticket>) -> Self {
        let mut graph = DiGraph::new();
        let mut index_of = HashMap::new();
        let mut stubs: HashMap<TicketId, StubKind> = HashMap::new();
        for id in active.keys() {
            index_of.insert(id.clone(), graph.add_node(id.clone()));
        }
        for (id, ticket) in &active {
            for blocker in &ticket.front_matter.blocked_by {
                match resolve_blocker(&active, archived, blocker) {
                    BlockerResolution::Active(blocker_id) => {
                        let from = index_of[&blocker_id];
                        let to = index_of[id];
                        graph.add_edge(from, to, ());
                    }
                    BlockerResolution::ArchivedDone(_) => {}
                    BlockerResolution::ArchivedStub(blocker_id, status) => {
                        let from = *index_of
                            .entry(blocker_id.clone())
                            .or_insert_with(|| graph.add_node(blocker_id.clone()));
                        stubs.insert(blocker_id, StubKind::Archived(status));
                        let to = index_of[id];
                        graph.add_edge(from, to, ());
                    }
                    BlockerResolution::Missing(blocker_id) => {
                        let from = *index_of
                            .entry(blocker_id.clone())
                            .or_insert_with(|| graph.add_node(blocker_id.clone()));
                        stubs.insert(blocker_id, StubKind::Missing);
                        let to = index_of[id];
                        graph.add_edge(from, to, ());
                    }
                }
            }
        }

        let mut sort_keys: HashMap<TicketId, chrono::DateTime<chrono::Utc>> = active
            .iter()
            .map(|(id, t)| (id.clone(), t.front_matter.created_at))
            .collect();
        for stub_id in stubs.keys() {
            let ix = index_of[stub_id];
            let earliest_dependent = graph
                .neighbors_directed(ix, Direction::Outgoing)
                .map(|n| sort_keys[&graph[n]])
                .min()
                .expect("a stub node always has at least one dependent");
            sort_keys.insert(stub_id.clone(), earliest_dependent);
        }

        DepsGraph {
            graph,
            index_of,
            tickets: active,
            stubs,
            sort_keys,
        }
    }

    /// Root tickets: no in-graph blocker, ordered by `created_at` ascending
    /// (stubs use the earliest `created_at` among their dependents).
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
        ids.sort_by_key(|id| self.sort_keys[id]);
    }

    /// IDs participating in a `blocked_by` dependency cycle, sorted. Empty
    /// when the graph is acyclic.
    ///
    /// `toposort` detects whether a cycle exists at all (cheap, and matches
    /// the epic's stated detection mechanism); `tarjan_scc` then names every
    /// id involved, via strongly-connected components of size > 1 (or a
    /// single node with a self-loop, e.g. a ticket blocked by itself).
    pub fn cyclic_ids(&self) -> Vec<TicketId> {
        if toposort(&self.graph, None).is_ok() {
            return Vec::new();
        }
        let mut ids: Vec<TicketId> = tarjan_scc(&self.graph)
            .into_iter()
            .filter(|scc| scc.len() > 1 || self.graph.contains_edge(scc[0], scc[0]))
            .flat_map(|scc| scc.into_iter().map(|ix| self.graph[ix].clone()))
            .collect();
        ids.sort_by_key(ToString::to_string);
        ids
    }

    fn label(&self, id: &TicketId) -> String {
        if let Some(stub) = self.stubs.get(id) {
            return stub.label(id);
        }
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

/// Load every parseable ticket from `dir.archived()`, keyed by id.
pub(crate) fn load_archived(dir: &WorkingDir) -> Result<HashMap<TicketId, Ticket>> {
    load_dir(&dir.archived())
}

/// Load every parseable `.md` ticket directly under `path`, keyed by id.
/// Shared by the active-set and archived-set loaders above.
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
///
/// A ticket is nested under EVERY blocker it has: diamonds (and cycles)
/// repeat a node under each of its parents. The first occurrence (in
/// traversal order) expands its subtree in full; a later occurrence within
/// the same still-open branch is a genuine cycle, marked
/// `(cycle: see above)`; a later occurrence reached via a different branch
/// is a diamond repeat, marked `(see above)`. Either way recursion stops
/// there, so nothing loops and nothing is printed more than once in full.
///
/// Some ids are never reached by descending from a zero-incoming-edge root
/// at all — every member of a cycle that has no external edge in is exactly
/// this case. Those are given a synthetic root each, in `created_at` order,
/// after the real roots, so a cycle is never silently invisible.
pub fn render_forest(graph: &DepsGraph) -> String {
    let mut output = String::new();
    let mut path: HashSet<TicketId> = HashSet::new();
    let mut rendered: HashSet<TicketId> = HashSet::new();

    for root in graph.roots() {
        if !rendered.contains(&root) {
            render_node(graph, &root, "", "", &mut path, &mut rendered, &mut output);
        }
    }

    let mut leftover: Vec<TicketId> = graph
        .tickets
        .keys()
        .filter(|id| !rendered.contains(*id))
        .cloned()
        .collect();
    graph.sort_by_created_at(&mut leftover);
    for id in leftover {
        if !rendered.contains(&id) {
            render_node(graph, &id, "", "", &mut path, &mut rendered, &mut output);
        }
    }

    output
}

fn render_node(
    graph: &DepsGraph,
    id: &TicketId,
    line_prefix: &str,
    child_base: &str,
    path: &mut HashSet<TicketId>,
    rendered: &mut HashSet<TicketId>,
    output: &mut String,
) {
    if path.contains(id) {
        output.push_str(&format!(
            "{}{}  (cycle: see above)\n",
            line_prefix,
            graph.label(id)
        ));
        return;
    }
    if rendered.contains(id) {
        output.push_str(&format!(
            "{}{}  (see above)\n",
            line_prefix,
            graph.label(id)
        ));
        return;
    }

    output.push_str(&format!("{}{}\n", line_prefix, graph.label(id)));
    rendered.insert(id.clone());
    path.insert(id.clone());

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
            path,
            rendered,
            output,
        );
    }

    path.remove(id);
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

        assert_eq!(resolution, BlockerResolution::Active(id("a")));
    }

    #[test]
    fn archived_done_blocker_resolves_archived_done() {
        let active = HashMap::new();
        let archived: HashMap<TicketId, Ticket> = [(id("a"), make_ticket("a", TicketStatus::Done))]
            .into_iter()
            .collect();

        let resolution = resolve_blocker(&active, &archived, &id("a"));

        assert_eq!(resolution, BlockerResolution::ArchivedDone(id("a")));
    }

    #[test]
    fn archived_non_done_blocker_resolves_archived_stub() {
        let active = HashMap::new();
        let archived: HashMap<TicketId, Ticket> =
            [(id("a"), make_ticket("a", TicketStatus::Rejected))]
                .into_iter()
                .collect();

        let resolution = resolve_blocker(&active, &archived, &id("a"));

        assert_eq!(
            resolution,
            BlockerResolution::ArchivedStub(id("a"), TicketStatus::Rejected)
        );
    }

    #[test]
    fn blocker_missing_everywhere_resolves_missing() {
        let active = HashMap::new();
        let archived = HashMap::new();

        let resolution = resolve_blocker(&active, &archived, &id("a"));

        assert_eq!(resolution, BlockerResolution::Missing(id("a")));
    }

    #[test]
    fn blocker_satisfied_treats_archived_done_as_satisfied() {
        let active = HashMap::new();
        let resolution = BlockerResolution::ArchivedDone(id("a"));

        assert!(blocker_satisfied(&active, &resolution));
    }

    #[test]
    fn blocker_satisfied_treats_archived_stub_as_unsatisfied() {
        let active = HashMap::new();
        let resolution = BlockerResolution::ArchivedStub(id("a"), TicketStatus::Rejected);

        assert!(!blocker_satisfied(&active, &resolution));
    }

    #[test]
    fn blocker_satisfied_treats_missing_as_unsatisfied() {
        let active = HashMap::new();
        let resolution = BlockerResolution::Missing(id("a"));

        assert!(!blocker_satisfied(&active, &resolution));
    }

    #[test]
    fn is_unblocked_true_when_only_blocker_is_archived_done() {
        let active = HashMap::new();
        let archived: HashMap<TicketId, Ticket> =
            [(id("blocker"), make_ticket("blocker", TicketStatus::Done))]
                .into_iter()
                .collect();
        let mut dependent = make_ticket("dependent", TicketStatus::Todo);
        dependent.front_matter.blocked_by = vec![id("blocker")];

        assert!(is_unblocked(&active, &archived, &dependent));
    }

    #[test]
    fn is_unblocked_false_when_only_blocker_is_archived_non_done() {
        let active = HashMap::new();
        let archived: HashMap<TicketId, Ticket> = [(
            id("blocker"),
            make_ticket("blocker", TicketStatus::Rejected),
        )]
        .into_iter()
        .collect();
        let mut dependent = make_ticket("dependent", TicketStatus::Todo);
        dependent.front_matter.blocked_by = vec![id("blocker")];

        assert!(!is_unblocked(&active, &archived, &dependent));
    }

    #[test]
    fn is_unblocked_false_when_only_blocker_is_missing() {
        let active = HashMap::new();
        let archived = HashMap::new();
        let mut dependent = make_ticket("dependent", TicketStatus::Todo);
        dependent.front_matter.blocked_by = vec![id("blocker")];

        assert!(!is_unblocked(&active, &archived, &dependent));
    }
}
