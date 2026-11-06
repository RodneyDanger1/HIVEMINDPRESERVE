//! Scoped agent context and the rewire-time context switch (blueprint WP 3.2/3.3).
//!
//! A task's context is a set of scoped facts tagged by the parent that supplied
//! them (`parent:<task id>:<key>`), the allowed tool ids and the output schema
//! id. When a task is rewired, the previous parent's scoped facts are cleared
//! and the new parent's facts are injected — the previous parent's context never
//! leaks into the new branch.

use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use hiverouter_contract::ids::TaskId;
use hiverouter_contract::Refusal;

/// Prefix for facts scoped to a parent task.
pub const FACT_PREFIX: &str = "parent:";

/// Scoped context handed to one agent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScopedContext {
    /// Parent that supplied the current facts.
    pub parent_task_id: Option<TaskId>,
    /// Scoped facts keyed `parent:<parent id>:<key>` (or plain keys for root).
    pub facts: BTreeMap<String, String>,
    /// Tool ids this agent may use.
    pub allowed_tool_ids: Vec<String>,
    /// Output schema id the child result must satisfy.
    pub output_schema_id: Option<String>,
}

impl ScopedContext {
    /// Context with no facts.
    #[must_use]
    pub fn new(parent_task_id: Option<TaskId>) -> Self {
        Self {
            parent_task_id,
            ..Self::default()
        }
    }

    /// Adds a fact scoped to `parent`.
    #[must_use]
    pub fn with_fact(mut self, parent: TaskId, key: &str, value: &str) -> Self {
        self.facts.insert(fact_key(&parent, key), value.to_owned());
        self
    }

    /// Facts visible to this agent (ordered).
    #[must_use]
    pub fn token_set(&self) -> Vec<String> {
        let mut tokens = Vec::with_capacity(self.facts.len() * 2);
        for (key, value) in &self.facts {
            tokens.push(key.clone());
            tokens.push(value.clone());
        }
        tokens
    }

    /// Whether any fact from `parent` is still present.
    #[must_use]
    pub fn contains_parent_facts(&self, parent: TaskId) -> bool {
        let prefix = parent_prefix(&parent);
        self.facts.keys().any(|key| key.starts_with(&prefix))
    }
}

fn parent_prefix(parent: &TaskId) -> String {
    format!("{FACT_PREFIX}{parent}:")
}

fn fact_key(parent: &TaskId, key: &str) -> String {
    format!("{}{key}", parent_prefix(parent))
}

/// Result of one context switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextSwitchOutcome {
    /// New context after the switch.
    pub context: ScopedContext,
    /// Parent whose facts were cleared.
    pub cleared_parent: Option<TaskId>,
    /// Fact keys removed.
    pub cleared_facts: Vec<String>,
    /// Fact keys injected from the new parent.
    pub injected_facts: Vec<String>,
}

/// Switches a task's context from its old parent to a new parent.
///
/// Rules: clear every fact scoped to the old parent, keep root facts, inject the
/// new parent's facts, and replace `allowed_tool_ids`/`output_schema_id`.
#[must_use]
pub fn switch_context(
    current: &ScopedContext,
    new_parent_task_id: Option<TaskId>,
    new_parent_facts: &BTreeMap<String, String>,
    new_allowed_tool_ids: Vec<String>,
    new_output_schema_id: Option<String>,
) -> ContextSwitchOutcome {
    let mut facts = current.facts.clone();
    let cleared_parent = current.parent_task_id;
    let mut cleared_facts = Vec::new();
    if let Some(old_parent) = cleared_parent {
        let prefix = parent_prefix(&old_parent);
        let keys: Vec<String> = facts
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .cloned()
            .collect();
        for key in &keys {
            facts.remove(key);
        }
        cleared_facts = keys;
    }
    let mut injected_facts = Vec::new();
    if let Some(new_parent) = new_parent_task_id {
        for (key, value) in new_parent_facts {
            let scoped = fact_key(&new_parent, key);
            facts.insert(scoped.clone(), value.clone());
            injected_facts.push(scoped);
        }
    }
    ContextSwitchOutcome {
        context: ScopedContext {
            parent_task_id: new_parent_task_id,
            facts,
            allowed_tool_ids: new_allowed_tool_ids,
            output_schema_id: new_output_schema_id,
        },
        cleared_parent,
        cleared_facts,
        injected_facts,
    }
}

/// In-memory context owner (one context per task).
#[derive(Debug, Default)]
pub struct ContextStore {
    contexts: Mutex<HashMap<TaskId, ScopedContext>>,
}

impl ContextStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a task's context.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` if the store lock is poisoned.
    pub fn set(&self, task_id: TaskId, context: ScopedContext) -> Result<(), Refusal> {
        let mut contexts = self.contexts.lock().map_err(|_| Refusal::InternalError)?;
        contexts.insert(task_id, context);
        Ok(())
    }

    /// Reads a task's context.
    ///
    /// # Errors
    /// `E_INTERNAL_ERROR` if the store lock is poisoned.
    pub fn get(&self, task_id: TaskId) -> Result<Option<ScopedContext>, Refusal> {
        let contexts = self.contexts.lock().map_err(|_| Refusal::InternalError)?;
        Ok(contexts.get(&task_id).cloned())
    }

    /// Applies a rewire context switch for `task_id`.
    ///
    /// # Errors
    /// `E_TASK_NOT_FOUND` when the task has no context, `E_INTERNAL_ERROR` if the
    /// store lock is poisoned.
    pub fn switch_on_rewire(
        &self,
        task_id: TaskId,
        new_parent_task_id: Option<TaskId>,
        new_parent_facts: &BTreeMap<String, String>,
        new_allowed_tool_ids: Vec<String>,
        new_output_schema_id: Option<String>,
    ) -> Result<ContextSwitchOutcome, Refusal> {
        let mut contexts = self.contexts.lock().map_err(|_| Refusal::InternalError)?;
        let current = contexts.get(&task_id).ok_or(Refusal::TaskNotFound)?.clone();
        let outcome = switch_context(
            &current,
            new_parent_task_id,
            new_parent_facts,
            new_allowed_tool_ids,
            new_output_schema_id,
        );
        contexts.insert(task_id, outcome.context.clone());
        Ok(outcome)
    }

    /// Number of tracked tasks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.contexts.lock().map(|contexts| contexts.len()).unwrap_or(0)
    }

    /// Whether the store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_clears_old_parent_facts_and_injects_new_ones() {
        let old_parent = TaskId::new();
        let new_parent = TaskId::new();
        let _task = TaskId::new();
        let current = ScopedContext::new(Some(old_parent))
            .with_fact(old_parent, "style", "honeycomb")
            .with_fact(old_parent, "system_prompt", "old branch");

        let mut new_facts = BTreeMap::new();
        new_facts.insert("style".to_owned(), "brutalist".to_owned());
        new_facts.insert("system_prompt".to_owned(), "new branch".to_owned());

        let outcome = switch_context(
            &current,
            Some(new_parent),
            &new_facts,
            vec!["file_read".to_owned()],
            Some("schema-v1".to_owned()),
        );
        assert_eq!(outcome.cleared_parent, Some(old_parent));
        assert_eq!(outcome.cleared_facts.len(), 2);
        assert!(!outcome.context.contains_parent_facts(old_parent));
        assert!(outcome.context.contains_parent_facts(new_parent));
        assert_eq!(
            outcome.context.facts.get(&fact_key(&new_parent, "style")),
            Some(&"brutalist".to_owned())
        );
        let tokens = outcome.context.token_set().join("|");
        assert!(!tokens.contains("old branch"), "old context leaked: {tokens}");
        assert!(tokens.contains("new branch"));
        assert_eq!(outcome.context.allowed_tool_ids, vec!["file_read".to_owned()]);
        assert_eq!(outcome.context.output_schema_id.as_deref(), Some("schema-v1"));
    }

    #[test]
    fn store_switches_on_rewire() {
        let store = ContextStore::new();
        let old_parent = TaskId::new();
        let new_parent = TaskId::new();
        let task = TaskId::new();
        store
            .set(task, ScopedContext::new(Some(old_parent)).with_fact(old_parent, "k", "old"))
            .expect("set");
        let mut facts = BTreeMap::new();
        facts.insert("k".to_owned(), "new".to_owned());
        let outcome = store
            .switch_on_rewire(task, Some(new_parent), &facts, Vec::new(), None)
            .expect("switch");
        assert!(!outcome.context.contains_parent_facts(old_parent));
        assert!(outcome.context.contains_parent_facts(new_parent));
        let reread = store.get(task).expect("get").expect("context");
        assert_eq!(reread.parent_task_id, Some(new_parent));
    }

    #[test]
    fn unknown_task_is_refused_by_name() {
        let store = ContextStore::new();
        assert_eq!(
            store.switch_on_rewire(TaskId::new(), None, &BTreeMap::new(), Vec::new(), None),
            Err(Refusal::TaskNotFound)
        );
    }
}
