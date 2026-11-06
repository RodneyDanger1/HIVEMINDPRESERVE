//! Live provider proof. Ignored by default so the normal suite never spends
//! money or touches the network. Run explicitly with `--ignored` after setting
//! `DEEPSEEK_API_KEY`; the key is never printed.
//!
//! This test exercises the same `ProviderExecutorAdapter` the server uses:
//! planner call -> validated plan -> one leaf call -> truthful usage.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use hiverouter_contract::{AgentState, RunId};
use hiverouter_core::config::ProviderSettings;
use hiverouter_core::entities::TaskRecord;
use hiverouter_core::execution::{DeltaSink, ProviderExecutor};
use hiverouter_core::provider_executor::ProviderExecutorAdapter;
use tokio_util::sync::CancellationToken;

#[tokio::test]
#[ignore = "live DeepSeek provider call; set DEEPSEEK_API_KEY and run with --ignored"]
async fn live_plan_and_leaf_are_real() {
    assert!(
        hiverouter_provider::ApiKey::present_in_env(),
        "DEEPSEEK_API_KEY is required for the live smoke"
    );
    let settings = ProviderSettings::default();
    let adapter = Arc::new(
        ProviderExecutorAdapter::from_env(settings).expect("provider adapter from environment"),
    );
    let run_id = RunId::new();
    let cancel = CancellationToken::new();

    let plan = tokio::time::timeout(
        Duration::from_secs(180),
        adapter.plan(
            run_id,
            "Create the smallest useful DAG for this task: reply with exactly HIVE-PROVIDER-OK.",
            &cancel,
        ),
    )
    .await
    .expect("planner timeout")
    .expect("planner call");
    plan.validate().expect("valid plan");
    assert!(!plan.nodes.is_empty());
    println!("plan_nodes={:?}", plan.nodes.iter().map(|n| n.role.clone()).collect::<Vec<_>>());

    let task = TaskRecord {
        id: plan.nodes[0].node_id,
        run_id,
        parent_task_id: None,
        role: plan.nodes[0].role.clone(),
        depth: plan.nodes[0].depth,
        spawn_seq: 1,
        state: AgentState::Running,
        rev: 0,
    };
    let streamed: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink: DeltaSink = {
        let streamed = Arc::clone(&streamed);
        Arc::new(move |delta: &str| {
            if let Ok(mut fragments) = streamed.lock() {
                fragments.push(delta.to_owned());
            }
        })
    };
    let reply = tokio::time::timeout(
        Duration::from_secs(180),
        adapter.execute(
            run_id,
            &task,
            "Reply with exactly HIVE-PROVIDER-OK",
            &cancel,
            &sink,
        ),
    )
    .await
    .expect("leaf timeout")
    .expect("leaf call");

    let text = reply.text();
    assert!(!text.trim().is_empty(), "provider returned empty content");
    assert!(reply.usage.prompt_tokens.is_some(), "usage must be measured");
    let fragments = streamed.lock().expect("streamed fragments");
    assert!(!fragments.is_empty(), "the live reply must stream at least one fragment");
    assert_eq!(
        fragments.concat(),
        text,
        "streamed fragments concatenate to the persisted reply text"
    );
    drop(fragments);
    println!("reply={text}");
    println!("usage={:?}", reply.usage);
}
