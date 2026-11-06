//! Graph-authority acceptance tests (Phase 2): cycle refusal, optimistic
//! revision, rewire-in-flight refusal, single-owner transitions and toposort.

use hiverouter_contract::{AgentState, Refusal, TaskId};
use hiverouter_core::graph::GraphAuthority;

fn chain() -> (GraphAuthority, TaskId, TaskId, TaskId) {
    let graph = GraphAuthority::new();
    let root = TaskId::new();
    let middle = TaskId::new();
    let leaf = TaskId::new();
    graph.add_task(root, None).expect("root");
    graph.add_task(middle, Some(root)).expect("middle");
    graph.add_task(leaf, Some(middle)).expect("leaf");
    (graph, root, middle, leaf)
}

#[test]
fn cycle_rejected() {
    let (graph, root, middle, leaf) = chain();
    let nodes_before = graph.node_count().expect("nodes");
    let edges_before = graph.edge_count().expect("edges");
    let revision_before = graph.revision().expect("revision");

    let refused = graph.rewire(middle, leaf, 0).expect_err("must refuse cycle");
    assert_eq!(refused, Refusal::Cycle);
    assert_eq!(refused.code(), "E_CYCLE");
    assert_eq!(graph.node_count().expect("nodes"), nodes_before);
    assert_eq!(graph.edge_count().expect("edges"), edges_before);
    assert_eq!(graph.parent_of(middle).expect("parent"), Some(root));
    assert_eq!(graph.revision().expect("revision"), revision_before);

    // Rewiring an ancestor under a sibling subtree is still allowed.
    graph.rewire(leaf, root, 0).expect("acyclic rewire");
    assert_eq!(graph.parent_of(leaf).expect("parent"), Some(root));
}

#[test]
fn rewire_to_self_is_a_cycle() {
    let graph = GraphAuthority::new();
    let task = TaskId::new();
    graph.add_task(task, None).expect("task");
    assert_eq!(graph.rewire(task, task, 0), Err(Refusal::Cycle));
    assert_eq!(graph.edge_count().expect("edges"), 0);
}

#[test]
fn stale_expected_revision_requires_resync() {
    let (graph, root, middle, _leaf) = chain();
    assert_eq!(
        graph.rewire(middle, root, 99),
        Err(Refusal::ResyncRequired)
    );
    assert_eq!(graph.parent_of(middle).expect("parent"), Some(root));
    // Correct revision succeeds and bumps the revision.
    graph.rewire(middle, root, 0).expect("rewire");
    assert_eq!(
        graph.rewire(middle, root, 0),
        Err(Refusal::ResyncRequired),
        "revision must advance after a successful rewire"
    );
}

#[test]
fn rewire_in_flight_is_refused_then_cleared() {
    let (graph, root, middle, _leaf) = chain();
    graph.begin_rewire(middle, 0).expect("first begin");
    assert_eq!(graph.begin_rewire(middle, 0), Err(Refusal::RewireInFlight));
    graph.finish_rewire(middle);
    graph.begin_rewire(middle, 0).expect("begin after finish");
    graph.finish_rewire(middle);
    assert_eq!(
        graph.rewire(middle, root, 0).map(|()| ()),
        Ok(()),
        "rewire is possible once the in-flight marker is cleared"
    );
}

#[test]
fn unknown_task_is_refused_by_name() {
    let graph = GraphAuthority::new();
    assert_eq!(
        graph.rewire(TaskId::new(), TaskId::new(), 0),
        Err(Refusal::TaskNotFound)
    );
    assert_eq!(
        graph.set_state(TaskId::new(), AgentState::Running),
        Err(Refusal::TaskNotFound)
    );
}

#[test]
fn snapshot_exposes_toposort_and_states() {
    let (graph, root, middle, leaf) = chain();
    graph.set_state(middle, AgentState::Running).expect("state");
    let snapshot = graph.snapshot().expect("snapshot");
    assert_eq!(snapshot.nodes.len(), 3);
    assert_eq!(snapshot.edges.len(), 2);
    assert_eq!(snapshot.toposort, vec![root, middle, leaf]);
    let middle_node = snapshot
        .nodes
        .iter()
        .find(|node| node.task_id == middle)
        .expect("middle node");
    assert_eq!(middle_node.state, AgentState::Running);
    assert_eq!(middle_node.parent_task_id, Some(root));
    assert_eq!(
        snapshot.nodes.iter().find(|node| node.task_id == leaf).expect("leaf").parent_task_id,
        Some(middle)
    );
}

#[test]
fn single_owner_rejects_duplicate_task_ids() {
    let graph = GraphAuthority::new();
    let task = TaskId::new();
    graph.add_task(task, None).expect("first");
    assert_eq!(graph.add_task(task, None), Err(Refusal::SchemaInvalid));
    assert_eq!(graph.node_count().expect("nodes"), 1);
}
