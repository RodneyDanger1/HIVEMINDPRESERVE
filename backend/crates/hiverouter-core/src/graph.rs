//! Authoritative task graph — the single owner of graph state.
//!
//! `petgraph::DiGraph` is the authority (blueprint WP 2.7): edges point from
//! parent to child, insertion is preceded by a cycle check, a successful rewire
//! stores a fresh `toposort` order, and `DIFF`-like mutation from anywhere else
//! is impossible because all mutations go through this type.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, MutexGuard};

use hiverouter_contract::{AgentState, Refusal, TaskId};
use petgraph::algo::{has_path_connecting, is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};

/// One node of the authoritative graph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskNode {
    /// Task id.
    pub task_id: TaskId,
    /// Current parent (`None` for the root task).
    pub parent_task_id: Option<TaskId>,
    /// Current lifecycle state.
    pub state: AgentState,
    /// Optimistic-concurrency revision, bumped on every mutation.
    pub rev: u64,
}

/// A read-only view of the whole graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphSnapshot {
    /// Graph revision, bumped on every successful mutation.
    pub revision: u64,
    /// All nodes, in insertion order.
    pub nodes: Vec<TaskNode>,
    /// Parent -> child edges.
    pub edges: Vec<(TaskId, TaskId)>,
    /// Current topological order (parents before children).
    pub toposort: Vec<TaskId>,
}

#[derive(Debug, Default)]
struct GraphState {
    graph: DiGraph<TaskId, ()>,
    index: HashMap<TaskId, NodeIndex>,
    nodes: HashMap<TaskId, TaskNode>,
    revision: u64,
    rewires_in_flight: HashSet<TaskId>,
}

/// The single graph-authority owner.
#[derive(Debug, Default)]
pub struct GraphAuthority {
    state: Mutex<GraphState>,
}

impl GraphAuthority {
    /// Creates an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> Result<MutexGuard<'_, GraphState>, Refusal> {
        self.state.lock().map_err(|_| Refusal::InternalError)
    }

    /// Adds a task node under an optional parent.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND` when the parent is unknown, `E_SCHEMA_INVALID` when
    /// the task id already exists.
    pub fn add_task(
        &self,
        task_id: TaskId,
        parent_task_id: Option<TaskId>,
    ) -> Result<GraphSnapshot, Refusal> {
        let mut state = self.lock()?;
        if state.nodes.contains_key(&task_id) {
            return Err(Refusal::SchemaInvalid);
        }
        if let Some(parent) = parent_task_id {
            if !state.nodes.contains_key(&parent) {
                return Err(Refusal::TaskNotFound);
            }
        }
        let node_index = state.graph.add_node(task_id);
        state.index.insert(task_id, node_index);
        if let Some(parent) = parent_task_id {
            let parent_index = *state
                .index
                .get(&parent)
                .ok_or(Refusal::TaskNotFound)?;
            state.graph.add_edge(parent_index, node_index, ());
        }
        state.nodes.insert(
            task_id,
            TaskNode {
                task_id,
                parent_task_id,
                state: AgentState::Queued,
                rev: 0,
            },
        );
        state.revision += 1;
        snapshot_of(&state)
    }

    /// Marks a rewire as in flight for a task (used by async rewire flows).
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND` for an unknown task, `E_REWIRE_IN_FLIGHT` when a rewire
    /// for this task is already in flight.
    pub fn begin_rewire(&self, task_id: TaskId, expected_rev: u64) -> Result<(), Refusal> {
        let mut state = self.lock()?;
        let node = state.nodes.get(&task_id).ok_or(Refusal::TaskNotFound)?;
        if node.rev != expected_rev {
            return Err(Refusal::ResyncRequired);
        }
        if !state.rewires_in_flight.insert(task_id) {
            return Err(Refusal::RewireInFlight);
        }
        Ok(())
    }

    /// Clears the in-flight rewire marker for a task.
    pub fn finish_rewire(&self, task_id: TaskId) {
        if let Ok(mut state) = self.state.lock() {
            state.rewires_in_flight.remove(&task_id);
        }
    }

    /// Rewires `task_id` under `new_parent_id`, refusing cycles.
    ///
    /// The mutation is atomic: on `E_CYCLE` (or any other refusal) the node/edge
    /// counts and the topological order are unchanged.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND`, `E_RESYNC_REQUIRED` (stale `expected_rev`),
    /// `E_REWIRE_IN_FLIGHT`, or `E_CYCLE`.
    pub fn rewire(
        &self,
        task_id: TaskId,
        new_parent_id: TaskId,
        expected_rev: u64,
    ) -> Result<(), Refusal> {
        let mut state = self.lock()?;
        let task_index = *state.index.get(&task_id).ok_or(Refusal::TaskNotFound)?;
        let parent_index = *state.index.get(&new_parent_id).ok_or(Refusal::TaskNotFound)?;
        {
            let node = state.nodes.get(&task_id).ok_or(Refusal::TaskNotFound)?;
            if node.rev != expected_rev {
                return Err(Refusal::ResyncRequired);
            }
        }
        if !state.rewires_in_flight.insert(task_id) {
            return Err(Refusal::RewireInFlight);
        }

        // A rewire to self is always a cycle.
        if task_id == new_parent_id {
            state.rewires_in_flight.remove(&task_id);
            return Err(Refusal::Cycle);
        }
        // Adding parent -> task closes a cycle exactly when `task` already has a
        // path to `new_parent` (task is an ancestor of the proposed parent).
        if has_path_connecting(&state.graph, task_index, parent_index, None) {
            state.rewires_in_flight.remove(&task_id);
            return Err(Refusal::Cycle);
        }

        let previous_parent = state
            .nodes
            .get(&task_id)
            .and_then(|node| node.parent_task_id);
        let previous_parent_index = previous_parent.and_then(|parent| state.index.get(&parent).copied());
        if let Some(previous) = previous_parent_index {
            if let Some(edge) = state.graph.find_edge(previous, task_index) {
                state.graph.remove_edge(edge);
            }
        }
        state.graph.add_edge(parent_index, task_index, ());
        if is_cyclic_directed(&state.graph) {
            // Defensive: revert if the invariant was violated.
            if let Some(edge) = state.graph.find_edge(parent_index, task_index) {
                state.graph.remove_edge(edge);
            }
            if let Some(previous) = previous_parent_index {
                state.graph.add_edge(previous, task_index, ());
            }
            state.rewires_in_flight.remove(&task_id);
            return Err(Refusal::Cycle);
        }
        if let Some(node) = state.nodes.get_mut(&task_id) {
            node.parent_task_id = Some(new_parent_id);
            node.rev += 1;
        }
        state.revision += 1;
        state.rewires_in_flight.remove(&task_id);
        Ok(())
    }

    /// Sets a task's lifecycle state.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND` for an unknown task.
    pub fn set_state(&self, task_id: TaskId, state_value: AgentState) -> Result<(), Refusal> {
        let mut state = self.lock()?;
        let node = state.nodes.get_mut(&task_id).ok_or(Refusal::TaskNotFound)?;
        node.state = state_value;
        node.rev += 1;
        state.revision += 1;
        Ok(())
    }

    /// Snapshot of the whole graph.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` if the graph lock is poisoned.
    pub fn snapshot(&self) -> Result<GraphSnapshot, Refusal> {
        let state = self.lock()?;
        snapshot_of(&state)
    }

    /// Number of nodes.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` if the graph lock is poisoned.
    pub fn node_count(&self) -> Result<usize, Refusal> {
        Ok(self.lock()?.nodes.len())
    }

    /// Number of edges.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` if the graph lock is poisoned.
    pub fn edge_count(&self) -> Result<usize, Refusal> {
        Ok(self.lock()?.graph.edge_count())
    }

    /// Current revision counter.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` if the graph lock is poisoned.
    pub fn revision(&self) -> Result<u64, Refusal> {
        Ok(self.lock()?.revision)
    }

    /// Whether a task exists.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` if the graph lock is poisoned.
    pub fn contains(&self, task_id: TaskId) -> Result<bool, Refusal> {
        Ok(self.lock()?.nodes.contains_key(&task_id))
    }

    /// Current parent of a task.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND` for an unknown task.
    pub fn parent_of(&self, task_id: TaskId) -> Result<Option<TaskId>, Refusal> {
        let state = self.lock()?;
        state
            .nodes
            .get(&task_id)
            .map(|node| node.parent_task_id)
            .ok_or(Refusal::TaskNotFound)
    }

    /// Rebuilds the whole graph from a persisted snapshot (kill-restore path).
    ///
    /// # Errors
    /// `E_SNAPSHOT_CORRUPT` when the snapshot's edges refer to unknown nodes or
    /// the restored graph would contain a cycle; `E_INTERNAL_ERROR` if the graph
    /// lock is poisoned.
    pub fn restore_from_snapshot(&self, snapshot: &GraphSnapshot) -> Result<(), Refusal> {
        let mut state = self.lock()?;
        let mut graph = DiGraph::<TaskId, ()>::new();
        let mut index = HashMap::new();
        let mut nodes = HashMap::new();
        for node in &snapshot.nodes {
            let node_index = graph.add_node(node.task_id);
            index.insert(node.task_id, node_index);
            nodes.insert(node.task_id, node.clone());
        }
        for (parent, child) in &snapshot.edges {
            let parent_index = index.get(parent).copied().ok_or(Refusal::SnapshotCorrupt)?;
            let child_index = index.get(child).copied().ok_or(Refusal::SnapshotCorrupt)?;
            graph.add_edge(parent_index, child_index, ());
        }
        if is_cyclic_directed(&graph) {
            return Err(Refusal::SnapshotCorrupt);
        }
        state.graph = graph;
        state.index = index;
        state.nodes = nodes;
        state.revision = snapshot.revision;
        Ok(())
    }
}

fn snapshot_of(state: &GraphState) -> Result<GraphSnapshot, Refusal> {
    let mut nodes: Vec<TaskNode> = state.nodes.values().cloned().collect();
    nodes.sort_by_key(|node| (node.task_id.as_uuid(), node.rev));
    let mut edges: Vec<(TaskId, TaskId)> = state
        .graph
        .edge_indices()
        .filter_map(|edge| {
            let (parent, child) = state.graph.edge_endpoints(edge)?;
            Some((*state.graph.node_weight(parent)?, *state.graph.node_weight(child)?))
        })
        .collect();
    edges.sort_by_key(|(parent, child)| (parent.as_uuid(), child.as_uuid()));
    let toposort_order: Vec<TaskId> = toposort(&state.graph, None)
        .map_err(|_| Refusal::Cycle)?
        .into_iter()
        .filter_map(|index| state.graph.node_weight(index).copied())
        .collect();
    Ok(GraphSnapshot {
        revision: state.revision,
        nodes,
        edges,
        toposort: toposort_order,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_task_links_parent_and_bumps_revision() {
        let graph = GraphAuthority::new();
        let root = TaskId::new();
        let child = TaskId::new();
        graph.add_task(root, None).expect("root");
        graph.add_task(child, Some(root)).expect("child");
        assert_eq!(graph.node_count().expect("count"), 2);
        assert_eq!(graph.edge_count().expect("edges"), 1);
        assert_eq!(graph.parent_of(child).expect("parent"), Some(root));
        let snapshot = graph.snapshot().expect("snapshot");
        assert_eq!(snapshot.toposort.first(), Some(&root));
        assert!(snapshot.revision >= 2);
    }

    #[test]
    fn missing_parent_is_refused_by_name() {
        let graph = GraphAuthority::new();
        assert_eq!(
            graph.add_task(TaskId::new(), Some(TaskId::new())),
            Err(Refusal::TaskNotFound)
        );
    }
}
