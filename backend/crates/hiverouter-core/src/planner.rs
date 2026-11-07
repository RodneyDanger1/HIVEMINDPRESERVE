//! Plan DAG produced by one provider planning call, validated before dispatch
//! (blueprint WP 4.5/4.7).

use std::collections::{HashMap, HashSet};

use hiverouter_contract::{Refusal, TaskId};
use serde::{Deserialize, Serialize};

/// Maximum plan nodes / super-steps (blueprint §2a: 40-call ceiling).
pub const MAX_PLAN_NODES: usize = 40;

/// One planned node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanNode {
    /// Stable node id (becomes the task id).
    pub node_id: TaskId,
    /// Agent role/tier name.
    pub role: String,
    /// Nodes that must finish before this node.
    pub deps: Vec<TaskId>,
    /// Spawn depth (1-based, hard-capped at 3 by [`crate::caps`]).
    pub depth: u8,
    /// Planned parent node. `None` means the run root (legacy flat plans).
    ///
    /// When present, the parent must be another planned node at `depth - 1`.
    #[serde(default)]
    pub parent: Option<TaskId>,
}

/// One plan: an acyclic DAG of nodes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// Nodes in executor order (validation recomputes the order).
    pub nodes: Vec<PlanNode>,
}

impl Plan {
    /// Creates a plan.
    #[must_use]
    pub fn new(nodes: Vec<PlanNode>) -> Self {
        Self { nodes }
    }

    /// Validates the plan DAG: non-empty, <= 40 nodes, unique ids, deps refer to
    /// known nodes, and no cycle.
    ///
    /// # Errors
    /// `E_CALL_LIMIT` when over the node cap; `E_SCHEMA_INVALID` for duplicate
    /// ids, unknown deps or a cycle (cycle refused at plan validation).
    pub fn validate(&self) -> Result<(), Refusal> {
        if self.nodes.is_empty() {
            return Err(Refusal::SchemaInvalid);
        }
        if self.nodes.len() > MAX_PLAN_NODES {
            return Err(Refusal::CallLimit);
        }
        let mut known = HashSet::new();
        for node in &self.nodes {
            if !known.insert(node.node_id) {
                return Err(Refusal::SchemaInvalid);
            }
        }
        for node in &self.nodes {
            if node.role.trim().is_empty() || node.depth == 0 {
                return Err(Refusal::SchemaInvalid);
            }
            for dep in &node.deps {
                if !known.contains(dep) {
                    return Err(Refusal::SchemaInvalid);
                }
            }
        }
        let by_id: HashMap<TaskId, &PlanNode> =
            self.nodes.iter().map(|node| (node.node_id, node)).collect();
        for node in &self.nodes {
            if let Some(parent_id) = node.parent {
                if parent_id == node.node_id {
                    return Err(Refusal::SchemaInvalid);
                }
                let parent = by_id.get(&parent_id).ok_or(Refusal::SchemaInvalid)?;
                if parent.depth.saturating_add(1) != node.depth {
                    return Err(Refusal::SchemaInvalid);
                }
            }
            // A parent chain that loops would make the hierarchy non-tree-shaped
            // even though the dependency graph is acyclic.
            let mut seen = HashSet::new();
            let mut current = Some(node.node_id);
            while let Some(id) = current {
                if !seen.insert(id) {
                    return Err(Refusal::SchemaInvalid);
                }
                current = by_id.get(&id).and_then(|item| item.parent);
            }
        }
        if self.topo_order()?.len() != self.nodes.len() {
            return Err(Refusal::SchemaInvalid);
        }
        Ok(())
    }

    /// Kahn topological order of node ids (parents before children).
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` when the graph contains a cycle.
    pub fn topo_order(&self) -> Result<Vec<TaskId>, Refusal> {
        let mut indegree: HashMap<TaskId, usize> =
            self.nodes.iter().map(|node| (node.node_id, 0)).collect();
        for node in &self.nodes {
            for dep in &node.deps {
                if let Some(entry) = indegree.get_mut(&node.node_id) {
                    *entry += 1;
                    let _ = dep;
                }
            }
        }
        let mut ready: Vec<TaskId> = self
            .nodes
            .iter()
            .filter(|node| indegree.get(&node.node_id) == Some(&0))
            .map(|node| node.node_id)
            .collect();
        let mut ordered = Vec::with_capacity(self.nodes.len());
        while let Some(id) = ready.pop() {
            ordered.push(id);
            for node in &self.nodes {
                if node.deps.contains(&id) {
                    if let Some(entry) = indegree.get_mut(&node.node_id) {
                        *entry = entry.saturating_sub(1);
                        if *entry == 0 {
                            ready.push(node.node_id);
                        }
                    }
                }
            }
        }
        if ordered.len() != self.nodes.len() {
            return Err(Refusal::SchemaInvalid);
        }
        Ok(ordered)
    }

    /// Leaf nodes (no other node depends on them) in topological order.
    ///
    /// # Errors
    /// Propagates the topological-order refusal.
    pub fn leaf_order(&self) -> Result<Vec<TaskId>, Refusal> {
        let mut depended_on = HashSet::new();
        for node in &self.nodes {
            for dep in &node.deps {
                depended_on.insert(*dep);
            }
            if let Some(parent) = node.parent {
                depended_on.insert(parent);
            }
        }
        Ok(self
            .topo_order()?
            .into_iter()
            .filter(|id| !depended_on.contains(id))
            .collect())
    }

    /// Aggregator nodes (they have a child/dependent) in execution order:
    /// deepest first, then reverse topological order. A parent must run after
    /// the children whose results it consumes.
    ///
    /// # Errors
    /// Propagates the topological-order refusal.
    pub fn aggregation_order(&self) -> Result<Vec<TaskId>, Refusal> {
        let topo = self.topo_order()?;
        let rank: HashMap<TaskId, usize> = topo
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index))
            .collect();
        let mut aggregators: Vec<&PlanNode> = self
            .nodes
            .iter()
            .filter(|node| {
                self.nodes.iter().any(|other| {
                    other.parent == Some(node.node_id) || other.deps.contains(&node.node_id)
                })
            })
            .collect();
        aggregators.sort_by(|left, right| {
            right
                .depth
                .cmp(&left.depth)
                .then_with(|| rank[&right.node_id].cmp(&rank[&left.node_id]))
        });
        Ok(aggregators.iter().map(|node| node.node_id).collect())
    }

    /// Looks a node up by id.
    #[must_use]
    pub fn node(&self, node_id: TaskId) -> Option<&PlanNode> {
        self.nodes.iter().find(|node| node.node_id == node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(role: &str, deps: Vec<TaskId>, depth: u8) -> PlanNode {
        PlanNode {
            node_id: TaskId::new(),
            role: role.to_owned(),
            deps,
            depth,
            parent: None,
        }
    }

    #[test]
    fn valid_dag_orders_parents_first() {
        let first = node("super", Vec::new(), 1);
        let second = node("worker", vec![first.node_id], 2);
        let plan = Plan::new(vec![first.clone(), second.clone()]);
        plan.validate().expect("valid plan");
        let order = plan.topo_order().expect("order");
        assert_eq!(order, vec![first.node_id, second.node_id]);
        assert_eq!(plan.leaf_order().expect("leaves"), vec![second.node_id]);
    }

    #[test]
    fn cycle_is_refused_at_plan_validation() {
        let first = node("super", Vec::new(), 1);
        let second = node("worker", vec![first.node_id], 2);
        let mut cyc1 = first.clone();
        cyc1.deps = vec![second.node_id];
        let plan = Plan::new(vec![cyc1, second]);
        assert_eq!(plan.validate(), Err(Refusal::SchemaInvalid));
    }

    #[test]
    fn node_cap_is_forty() {
        let nodes: Vec<PlanNode> = (0..41).map(|_| node("worker", Vec::new(), 1)).collect();
        let plan = Plan::new(nodes);
        assert_eq!(plan.validate(), Err(Refusal::CallLimit));
    }

    #[test]
    fn unknown_dependency_is_refused() {
        let plan = Plan::new(vec![node("worker", vec![TaskId::new()], 1)]);
        assert_eq!(plan.validate(), Err(Refusal::SchemaInvalid));
    }

    #[test]
    fn parented_hierarchy_is_valid() {
        let mut super_agent = node("super", Vec::new(), 1);
        let mut agent = node("worker", Vec::new(), 2);
        agent.parent = Some(super_agent.node_id);
        let subagent = {
            let mut sub = node("worker", Vec::new(), 3);
            sub.parent = Some(agent.node_id);
            sub
        };
        // Give the first planned node a real root parent only at insertion; the
        // planner validates the planned-node chain above it.
        super_agent.parent = None;
        let plan = Plan::new(vec![super_agent, agent, subagent]);
        plan.validate().expect("valid parent chain");
        assert_eq!(plan.nodes[1].parent, Some(plan.nodes[0].node_id));
        assert_eq!(plan.nodes[2].parent, Some(plan.nodes[1].node_id));
        assert_eq!(
            plan.aggregation_order().expect("aggregation order"),
            vec![plan.nodes[1].node_id, plan.nodes[0].node_id]
        );
    }

    #[test]
    fn parent_depth_mismatch_is_refused() {
        let mut parent = node("super", Vec::new(), 1);
        parent.parent = None;
        let mut child = node("worker", Vec::new(), 3);
        child.parent = Some(parent.node_id);
        let plan = Plan::new(vec![parent, child]);
        assert_eq!(plan.validate(), Err(Refusal::SchemaInvalid));
    }

    #[test]
    fn self_parent_is_refused() {
        let mut item = node("worker", Vec::new(), 1);
        let id = item.node_id;
        item.parent = Some(id);
        let plan = Plan::new(vec![item]);
        assert_eq!(plan.validate(), Err(Refusal::SchemaInvalid));
    }
}
