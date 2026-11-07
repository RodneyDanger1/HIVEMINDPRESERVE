# 16 — GitNexus Current Audit (Calibrated)

## Calibration: what is genuinely wired

The following are **not** defects and should not be “fixed” as if broken:

| Capability | Evidence | Verdict |
|---|---|---|
| Provider executor | `core/src/http.rs:126-147` builds the real executor; `UnconfiguredExecutor` is an honest fallback | WIRED |
| 17 frozen WS commands dispatch with typed ACK/ERROR | `core/src/ws.rs` command handler | WIRED |
| Approval → grant CAS → `commit_side_effect` | `core/src/tools_bridge.rs:197-232` | WIRED |
| Web resync `/api/history/*` routes | `core/src/http.rs:387-397`; store hydration path | WIRED |
| Approval ledger | separate JSONL ledger at `resolve_approval_ledgered`; not the same as the general audit log | WIRED |
| Read-only tool loop and side-effect execution | execution/tool_loop/side_effects tests | WIRED |

The failure pattern is **silence, not fake success**: features that look shipped terminate in a queued row, an empty map, or a missing init call.

## Tier 1 — Outright broken

| # | Finding | Evidence | Severity |
|---|---|---|---|
| 1 | **TEMPLATE_INSERT creates tasks that never execute.** `insert_template_dag` writes a task row + graph node and returns; it never calls `enqueue_spawned_task`. The only production caller of `enqueue_spawned_task` is `spawn_sub_hive` (`execution.rs:825`), and `dispatch_spawned_tasks` drains only that queue (`execution.rs:1137`). Template nodes stay queued until terminal reconciliation sweeps them. | `core/src/ws.rs:260-295`; `core/src/execution.rs:602,825,1137` | **Critical** |
| 2 | **All four Library buttons are dead by construction.** `LibraryPanel` ships `content-writer`, `data-analyst`, `code-reviewer`, `chatbot`; the embedded registry has only `planner`; the backend test asserts those ids must fail. | `web/src/panels/LibraryPanel.tsx:7-11,60-64`; `core/src/prompts.rs:41`; `ws.rs:1130-1160` | **Critical** |
| 3 | **No runtime audit trail.** `init_json_audit`, `record_tool_call`, `record_refusal` are implemented/exported; only tests call them. `main.rs` never initializes the sink, so every tool call/refusal goes unrecorded. | `tools/src/audit.rs:56-124`; `tools/src/lib.rs:57-58`; `core/src/main.rs:102-128` | **High** |
| 4 | **NODE `kind` vanishes after rewire.** Engine computes kind from task tier (`execution.rs:527-540`), but `publish_node_event` hardcodes `kind: None` while claiming the tier is not threaded. `task_agent_tier` is `pub(crate)` and does exist. | `core/src/ws.rs:945-958`; `core/src/execution.rs:98-115` | **Medium** |
| 5 | **Persona tool authorization is unreachable.** `authorize_tool_call` checks scoped `allowed_tool_ids`, but nothing loads `personas.json`; `load_personas` has only a test caller, so the allowlist is always empty and the check always passes. | `core/src/execution.rs:704-722`; `core/src/prompts.rs:394`; `core/src/main.rs` | **High** |

## Tier 2 — Not wired at all

| Capability | State | Evidence |
|---|---|---|
| Scoped memory/facts | `switch_on_rewire` injects an empty fact map; no producer writes facts | `core/src/ws.rs:341-352` |
| Persona registry | Loader is test-only; rewire keeps a stale capability set | `core/src/ws.rs:330-337`; `prompts.rs:394` |
| Workspace file watcher | `WorkspaceWatcher::start` only in unit tests; no `FS_CHANGED` reaches the bus | `tools/src/watcher.rs:187-228` |
| `RUN_METRICS` | Defined/decoded in contract; no publisher in `core/src` | `contract/src/event.rs:268-270`; grep in core = 0 |
| `PING`/PONG keepalive | `publish_pong` has zero callers | `core/src/ws.rs:1010` |
| Usage/cost persistence | No usage/cost table; RunBudget is per-process only | `core/src/db.rs:36-120`; `provider/src/budget.rs:114` |
| Multi-key safety | `enforce_single_key` has no production caller | `provider/src/budget.rs:502`; only test callers |

Consequence: HUD metrics fall back to graph-derived values (`source: 'derived'`); no cost history; sessions cannot enforce one-key policy.

## Tier 3 — Wired poorly / designed poorly

| Finding | Evidence | Why it matters |
|---|---|---|
| **Two schema authorities** | `build_tools` runs `ToolsBridge::migrate` outside `Database::migrate` and only logs failure (`http.rs:107-112`); `Database::migrate` is documented authority (`db.rs:287`) | A broken approval table can leave `/health` ok while every approval refuses |
| **Config authority bypassed** | `HIVEROUTER_SIDE_EFFECTS` read via `std::env::var` inside the bridge (`tools_bridge.rs:590-597`), not `Config`; absent from settings/health/diagnose | Invisible kill switch; granted effects silently become `E_TOOL_NOT_ALLOWED` |
| **Dispatch queue volatile + racy** | `spawn_queue: Mutex<Vec<TaskId>>` (`execution.rs:373`), drained once at end of `dispatch_inner` (`:1126-1128`) | A spawn after drain is enqueued to nobody; restart loses queue; M-009 only true in a narrow window |
| **Approval replay capped at 20** | `MAX_APPROVAL_REPLAY_EVENTS = 20` (`ws.rs:566-580`) | 21+ pending approvals silently under-report; no pagination/“more” signal |
| **Contradictory doc comments** | `ws.rs:332` says `ContextStore::set` has no production caller while `seed_rewire_context` calls it 10 lines above; `ws.rs:548` says snapshot API “lands in Phase 9” although it shipped (`http.rs:387-397`) | Comments are load-bearing and misdescribe code |
| **Single-run hardcoded** | `submit_prompt` refuses `E_RUN_ACTIVE` whenever any run is active (`execution.rs:755-758`) | No queue/multi-run; one run per process |

## Tier 4 — Missing entirely

| Missing capability | Evidence |
|---|---|
| `projects` table | `project_id` is bare TEXT with no table/validation/lifecycle (`db.rs:53-60`); Library project name/id are client-only (`LibraryPanel.tsx:22-27`) |
| Usage/cost tables | Only per-process `RunBudget`; no persistence across restart (`budget.rs:114`) |
| Facts/memory store | `ScopedContext` is in-memory only (`context.rs:136`); nothing survives restart |
| Persona assets loaded | `personas.json` exists but is never loaded in production |
| Multi-node DAG manifests | Catalog fabricates one single-node DAG per template; real multi-node DAG data does not exist (`ws.rs:191`); sub-hive fan-out from templates impossible |

## Severity order

| # | Finding | Severity | User-visible? |
|---|---|---|---|
| 1 | Template-inserted tasks never dispatch | Critical | Yes — silent hang |
| 2 | Library templates all refuse | Critical | Yes — four dead buttons |
| 5 | Persona authorization dead | High | No — security check no-op |
| 3 | Audit log never initialized | High | No — forensic gap |
| — | Spawn queue volatile/racy | High | Intermittent |
| — | Side-effects kill switch invisible | Medium | Confusing refusals |
| 4 | `kind` lost on rewire | Medium | Yes — graph degrades |
| — | No `RUN_METRICS` publisher | Medium | Yes — HUD always derived |
| — | No usage persistence | Medium | Yes — no cost history |
| — | Two schema authorities | Medium | Latent boot corruption |

## Fix order

1. Enqueue `insert_template_dag` tasks into the engine queue (or route them through a real dispatcher).
2. Add the four template assets/manifests, or remove the Library buttons; do not leave dead UI.
3. Initialize the JSONL audit sink in `main.rs` and call the record functions at tool/refusal boundaries.
4. Thread the real tier into `publish_node_event` (use `task_agent_tier`) so `kind` is stable.
5. Load `personas.json` in production and populate `allowed_tool_ids`; otherwise remove the M-015 “check”.
6. Replace the raw env gate with Config/health/diagnose visibility.
7. Replace the one-shot queue with a durable/awakenable dispatch queue; close the spawn-vs-terminal race.
8. Add projects/usage/cost/facts tables; add `RUN_METRICS` and PONG producers; start the watcher.
9. Fix the migration authority and the lying comments.

## What this document does **not** claim

- It does not claim the provider/tool/approval execution path is fake — that path is genuinely wired.
- It does not claim the app refuses unsafely — the failure mode is silence.
- It does not claim these findings were fixed; they are current at HEAD `aa16e13`/`5db2f21`.
- Line numbers come from the GitNexus audit text and were spot-checked by grep; re-resolve before editing.
