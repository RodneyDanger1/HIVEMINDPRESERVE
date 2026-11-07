# 17 — Claim Register (post-wave 3)

Status: `VERIFIED` (command/test/proof), `READ` (source read), `UNVERIFIED`, `CONTRADICTED`.
This register is rebuilt from the current tree after commits up to `ce618cf`; it is not the frozen census.

| ID | Claim | Status | Evidence |
|---|---|---|---|
| C-01 | Plan nodes now carry an optional real parent | VERIFIED | `planner.rs` `PlanNode.parent`; `provider_executor.rs` `WireNode.parent`; `planned_parent_chain_is_persisted_not_flattened_to_root` |
| C-02 | Parent depth mismatch / self-parent is refused | VERIFIED | `planner::tests::parent_depth_mismatch_is_refused`, `self_parent_is_refused` |
| C-03 | Context is seeded at task creation, not only rewire | VERIFIED | `execution.rs::seed_task_context`; tree test asserts contexts for every task |
| C-04 | Scoped facts are injected into the provider prompt | VERIFIED (unit) / READ (end-to-end) | `prompt_context_tests`; `prompt_with_context`; no live provider call made |
| C-05 | Rewire now carries the new parent's scoped facts | VERIFIED | `ws.rs::apply_rewire_context`; context tests |
| C-06 | Non-leaf aggregators execute after children | VERIFIED | `planner::aggregation_order`; tree test 3 replies |
| C-07 | Every completed task persists a task-scoped result | VERIFIED | `task_results` migration + `complete_task`; tree test |
| C-08 | Four Library templates are embedded and resolve | VERIFIED | four `.prompty` assets; manifest; `include_str!`; `template_insert_resolves...` |
| C-09 | Library insert now enqueues tasks | READ | `ws.rs` TemplateInsert arm calls `enqueue_spawned_tasks`; no full WS insert/execute e2e yet |
| C-10 | Personas are embedded and loaded at boot | VERIFIED | `EMBEDDED_PERSONAS`; `load_personas_embedded`; `embedded_personas_load_without_a_data_dir` |
| C-11 | Persona allowlist is seeded into task context | VERIFIED | `boot_persona_allowlist_is_seeded_on_worker_contexts` |
| C-12 | Chat transcript hydrates after reload | VERIFIED (backend) / READ (web) | `/messages` route test; `store.ts` fetch/decode; no reload e2e |
| C-13 | Audit sink initializes at boot and records tool calls/refusals | READ | `main.rs` init; `finish_tool_call` / authorization refusal hooks; no binary audit-file assertion |
| C-14 | NODE kind is authoritative on spawn and rewire | VERIFIED | `ws_transport` kind assertion; `publish_node_event` |
| C-15 | File tree supports path scoping and measured size | VERIFIED | contract/command/tools tests; `DataPanel` expand |
| C-16 | `RUN_METRICS` has a producer | VERIFIED | lifecycle test counts `RUN_METRICS` |
| C-17 | `ask_parent` writes a durable message | VERIFIED | `ask_parent_writes_a_durable_message`; `agent_messages` migration v6 |
| C-18 | `send_to_sibling` is same-parent by default; cross-branch requires approval and emits `AGENT_MESSAGE` | VERIFIED | `cross_branch_message_requires_approval_then_writes_once`; `AGENT_MESSAGE` row+event test |
| C-19 | Side effects are Config-owned and visible in `/diagnose` | READ | `Config.side_effects_enabled`; `ToolsBridge` builder; diagnose field; no env-parsing test yet |
| C-20 | HMAS/AWN is complete | **CONTRADICTED** | Profiles can be imported/deleted and persist, but there is no full editor/controller/cross-branch approval or complete HMAS UI |
| C-21 | Worker/Agent→Subagent spawning exists | **PARTIAL** | Worker may spawn `subagent` and cannot escalate to Super; profiles may tighten `max_children`/`max_depth`; no per-run cap profile |
| C-22 | Memory lasts forever across runs | **PARTIAL** | `agent_memory` v7 + recall + list/clear API/UI; no semantic retrieval or project/global scopes |
| C-23 | Usage/cost persists across restart | **PARTIAL** | `run_usage` migration v8 persists measured tokens; cost is not persisted/settled |
| C-24 | Approval replay >20 is complete | **UNVERIFIED** | Cap remains 20; no paging |
| C-25 | Two schema authorities are fixed | **PARTIAL** | `ToolsBridge::migrate` still separate, but a failure now fails `/health` as a required check |
| C-26 | The contract command count is 17 | **CONTRADICTED** | `QA-HARNESS`/contract list says 14; derive from enum |
| C-27 | R1–R3 have no contract change | VERIFIED | Parent/context/aggregation are internal; golden schema unchanged by R1–R3 |

## Next proof required

- A real WS `TEMPLATE_INSERT` → terminal task test (C-09).
- A config/env test proving the side-effect gate (C-19).
- A negative `send_to_sibling` test (foreign parent/run) (C-18).
- A binary boot test that writes an audit file (C-13).
- A web reload/e2e proof of chat hydration (C-12).
- A negative `send_to_sibling` cross-parent test (C-18).
- Cost settlement/persistence, memory clear/consent, profile import/editor, cross-branch approval, HMAS UI.
- Agent profile/import, memory, cross-branch messaging, HMAS UI (C-20–C-23).
