//! Phase 3 acceptance: rewire context switch clears the previous parent's
//! scoped facts and injects the new parent's facts (blueprint card command
//! `rewire_context_switch`).

use std::collections::BTreeMap;

use hiverouter_contract::Refusal;
use hiverouter_core::context::{ContextStore, ScopedContext};

#[test]
fn rewire_context_switch() {
    let store = ContextStore::new();
    let run_task = hiverouter_contract::TaskId::new();
    let old_parent = hiverouter_contract::TaskId::new();
    let new_parent = hiverouter_contract::TaskId::new();

    store
        .set(
            run_task,
            ScopedContext::new(Some(old_parent))
                .with_fact(old_parent, "style", "old-honeycomb")
                .with_fact(old_parent, "secret_scope", "old-parent-only"),
        )
        .expect("store context");

    let mut new_facts = BTreeMap::new();
    new_facts.insert("style".to_owned(), "new-brutalist".to_owned());
    new_facts.insert("secrets".to_owned(), "new-parent-only".to_owned());

    let outcome = store
        .switch_on_rewire(
            run_task,
            Some(new_parent),
            &new_facts,
            vec!["file_read".to_owned()],
            Some("plan-v1".to_owned()),
        )
        .expect("switch");

    assert_eq!(outcome.cleared_parent, Some(old_parent));
    assert_eq!(outcome.cleared_facts.len(), 2);
    let tokens = outcome.context.token_set().join("|");
    assert!(
        !tokens.contains("old-honeycomb") && !tokens.contains("old-parent-only"),
        "previous parent's scoped facts leaked into the new context: {tokens}"
    );
    assert!(tokens.contains("new-brutalist"), "new parent facts missing: {tokens}");
    assert!(tokens.contains("new-parent-only"));
    let reread = store.get(run_task).expect("get").expect("context");
    assert_eq!(reread.parent_task_id, Some(new_parent));
    assert!(!reread.contains_parent_facts(old_parent));
    assert!(reread.contains_parent_facts(new_parent));
}

#[test]
fn switching_an_untracked_task_is_refused_by_name() {
    let store = ContextStore::new();
    assert_eq!(
        store.switch_on_rewire(
            hiverouter_contract::TaskId::new(),
            None,
            &BTreeMap::new(),
            Vec::new(),
            None,
        ),
        Err(Refusal::TaskNotFound)
    );
}
