# Round 9 — `deep-modernize` seat: architecture / modernization audit

**Seat:** audit-only (no product, test or config edits; no HiveMind start/stop; no browser; no provider/key).
**Skill:** `deep-modernize` (available and loaded). Its transform procedure is inapplicable here because the seat forbids edits, so the skill's *review* discipline is applied instead: pin a baseline, inventory the transformations/abstractions that would be touched, and record a ledger whose delta is identity.
**Workspace HEAD:** `8b9ccc8` `feat: subagent cards and live-snapshot preservation` (+ uncommitted `http.rs` / `snapshot_history.rs` / launcher changes in the working tree).
**Rust:** edition 2021, `rust-version = 1.89.0`, toolchain `stable-x86_64-pc-windows-gnu` (`rust-toolchain.toml`).
**Web:** React 19.3.1, TypeScript 7.0.2, Vite 8.3.6, zustand 5.0.16 (`web/package.json:1-40`).

---

## 0. Baseline pinned before review (evidence, not assertion)

| Gate | Command (run in this seat) | Observed result |
|---|---|---|
| Web unit suite | `node node_modules/vitest/vitest.mjs run --reporter=dot` (`web/`) | `Test Files 23 passed (23)` / `Tests 187 passed (187)`, 8.52 s |
| Web typecheck | `node node_modules/typescript/bin/tsc --noEmit` (`web/`) | exit `0`, no output |
| Rust (uncommitted change) | `cargo test --locked -p hiverouter-core --test snapshot_history` | `4 passed; 0 failed` (`migration_roundtrip`, `history_api_returns_real_revisions`, `corrupt_snapshot`, `history_http_api`) — so the uncommitted `agent_roster` compiles and its new assertions pass |
| Rust formatting gate | `cargo fmt --all -- --check` (`backend/`) | **FAIL — 538 `Diff in …` hunks across all four crates** (e.g. `hiverouter-contract/src/lib.rs:30`, `hiverouter-core/src/http.rs:30`, `hiverouter-tools/src/cli.rs:93`) |

**Transform ledger (skill output contract).** No transform was applied, so every row is identity. This table exists to prove the seat changed nothing.

| transform | files | baseline | after | delta |
|---|---|---|---|---|
| *none applied — audit-only seat* | 0 | Rust `4 passed/0 failed`; web `187 passed/23 files`; tsc `exit 0`; fmt `538 hunks` | unchanged | 0 |

Baseline caveats: the Rust baseline is one integration test binary, not the full workspace suite (running all ~60 binaries was out of the seat's time budget — see §5). The 538 fmt hunks predate this audit and indicate rustfmt is simply not enforced; they are **not** evidence that the tree is broken.

---

## 1. Findings table

Severity: **P0** = must fix before any release (none found, see §2). **P1** = wrong behaviour observable in a normal session, or an authority/contract violation that produces wrong state. **P2** = latent/conditional wrongness, structural duplication, or a truth defect in evidence. **P3** = hygiene/dead surface.
Safety: **safe** = local, deterministic, no wire change; **risky** = changes behaviour or wire/data semantics and needs its own test; **out of scope** = not this seat's file set or a product decision.

| ID | Sev | Area | Finding | Location | Blast radius | Safety |
|---|---|---|---|---|---|---|
| MOD-01 | P1 | history/error UI · Subagent | The HTTP replay/roster projection **infers** node `kind` from the free-form `role` string, which the contract explicitly forbids ("A producer populates `kind` only from the task's authoritative tier … an invented kind is worse than an absent one"). The live WS path uses the authoritative tier; the HTTP path uses a second, string-matching mapper. Divergence: a `role="subagent"`/`"sub_agent"` task is `worker` live and `subagent` after replay/roster; an empty role at depth 0 is `hyperagent` live and `worker` in replay (the tier mapper has a depth-0 rule, the role mapper does not). | `backend/crates/hiverouter-core/src/http.rs:743-751`, `:721`, `:958` vs `backend/crates/hiverouter-contract/src/event.rs:232-240`; live path `backend/crates/hiverouter-core/src/execution.rs:99-106`, `:151-163`, `backend/crates/hiverouter-core/src/ws.rs:1012` | Every history scrubber replay, every roster call, every node card/label/size/colour derived from `kind`; the `SubagentNode` card (added `a536417`) is reachable only through this inference | risky (either thread the tier into the persisted row, or drop the inferred field) |
| MOD-02 | P1 | hydration · Subagent | The web store **coerces** an absent/unknown `kind` to `'worker'`, contradicting the mirrored contract ("an unknown value is ignored … rather than coerced"). Recovery deliberately omits `kind` (`recovery.rs:443-446`: "kind stays absent because a task's tier is not threaded … the free-form role is never promoted to a kind"), so after **every boot recovery** the replayed nodes are silently re-typed as Worker in the UI (and counted as workers by `metrics.ts:42-44`). | `web/src/state/store.ts:1504-1507`, `:621`; `web/src/flow/graph.ts:302`; contract rule `web/src/api/contract.ts:229-230`; producer `backend/crates/hiverouter-core/src/recovery.rs:443-446`, `:467-473` | All recovered runs, any pre-1.3/unknown producer, any node whose task row read failed; HUD worker count, Library chips, card type | safe (keep `null` and render the generic card the contract describes) |
| MOD-03 | P1 | hydration | The resync merge keeps **live task records on top of** the authoritative snapshot with no per-task `rev` comparison (`{ ...projection.tasks, ...liveTasks }`). A node whose state was missed and is therefore stale locally cannot be repaired by the resync that exists to repair it — the stale record wins. | `web/src/state/store.ts:1175-1195` (compare `TaskRecord.rev`, `:67-88`, `:655-660`) | Every `RESYNC_REQUIRED` / reconnect / refusal-driven rehydrate; the primary correctness guarantee of the snapshot path | risky (merge per task by max `rev`, keep live-only tasks) |
| MOD-04 | P1 | hydration · Remote | `scheduleSessionRefresh` is armed on every close/error but is **never cancelled** when the transport reopens successfully (`onOpen` only sends hello). 1.5 s after a successful partysocket reconnect the client still closes the healthy socket, re-reads `/api/session`, re-hellos and restarts the approval replay window. The forced close fires `onClose`, which (because `manuallyClosed` was reset to `false`) schedules *another* refresh — one drop can cascade two or more full session re-opens. | `web/src/api/client.ts:325-330`, `:331-342`, `:377-385`, `:391-413` | Every transient disconnect; duplicate `SERVER_HELLO`, duplicate `APPROVAL_REQUEST` replays, approval staleness windows, auth token churn | risky (clear the timer on `open`, guard `recoveryInFlight`) |
| MOD-05 | P1 | hydration · Subagent/Remote | `SERVER_HELLO` carries the **server's current** seq, while the contract documents the field as "last server sequence number the client had seen". The store assigns it to its cursor before the replayed range is applied, so a socket that dies mid-replay reconnects reporting a cursor **ahead** of what it applied; `replay_after` then answers `Replay::Current` and the unapplied events are lost with **no gap and no refusal**. | `backend/crates/hiverouter-core/src/ws.rs:495-565` (esp. `:526-528`); contract doc `backend/crates/hiverouter-contract/src/event.rs:176-182`; client `web/src/state/store.ts:1362-1367`; server replay `backend/crates/hiverouter-core/src/bus.rs:170-206` | Any disconnect while a replay batch is in flight; silent divergence of graph/reply state | risky (send the client's reported seq, or keep the cursor at the last applied seq) |
| MOD-06 | P1 | memory · Subagent | Memory is **written** under `task.role` but **listed and cleared** under `persona.id`. For an imported profile whose id differs from the spawn role, the memory panel shows nothing and clearing it is a no-op. The same `role`-vs-`id` key confusion also silently disables the persona's `allowed_tool_ids` and spawn limits (lookups are `personas.get(&task.role)`). | write `backend/crates/hiverouter-core/src/execution.rs:880`, `:3027-3035`; read/clear `backend/crates/hiverouter-core/src/http.rs:985-1001`, `:1004-1019`; persona lookups `backend/crates/hiverouter-core/src/execution.rs:550-558`, `:871-876`; store keying `backend/crates/hiverouter-core/src/store.rs:612-671` | Memory UI honesty, recall into task context, persona capability enforcement for every imported profile | risky (pick one identity, migrate rows, keep a compatibility read) |
| MOD-07 | P1 | Subagent | `TEMPLATE_INSERT` turns a manifest id into a task `role` **without** `validate_spawn_role`, and `"planner"` maps to `HyperAgent`. The one shipped prompt asset is `planner`, so the operator can insert a child task that the tier machine treats as a second HyperAgent — a capability-rank promotion that `spawn_sub_hive` would have refused. That node is also filtered out of the canvas (`kind !== 'hyperagent'`), so the inserted task is **invisible** while it exists. | `backend/crates/hiverouter-core/src/ws.rs:124-132`, `:144-151`, `:240-291`; tier mapping `backend/crates/hiverouter-core/src/execution.rs:151-163`; canvas filter `web/src/flow/graph.ts:229-235`; spawn policy `backend/crates/hiverouter-core/src/execution.rs:108-142` | Template insert path; capability tiers; any node the operator cannot see or kill | risky (validate the spawn role for template nodes, or give templates a fixed non-tier role) |
| MOD-08 | P2 | CAS | The replay decoder **fabricates** `rev: 0` when the snapshot omits it, while the HTTP decoder fails closed. `rewireBlocker` treats a numeric `rev` as the confirmed server revision, so any future call that reaches `rewireSelected()` from replay state would send `expected_rev: 0` and lose `GraphAuthority::rewire` every time (`E_RESYNC_REQUIRED`), which is exactly the loop the store comment says it avoids. Currently latent: `Canvas.tsx:202-282` renders no nodes in replay, so drag cannot start. | `web/src/flow/graph.ts:318`; guard `web/src/state/store.ts:704-720`; CAS `backend/crates/hiverouter-core/src/graph.rs:141-155` | Replay read-only surface; any future store-level replay action | safe (keep `null` for absent `rev`) |
| MOD-09 | P2 | history/error UI | Two independent decoders for the **same** `HISTORY_SNAPSHOT` graph: the HTTP path throws a named schema refusal on a malformed node (`recordsFromSnapshotGraph`), the WS path silently skips it and invents defaults (`buildReplayTasks`). The Canvas consumes the lenient one, so identical server data yields different graphs depending on how it arrived. | strict `web/src/state/store.ts:582-645`; lenient `web/src/flow/graph.ts:280-329`; consumer `web/src/flow/Canvas.tsx:432`, `:652-653` | All replay rendering; schema-drift detection | safe (one decoder, strict, shared) |
| MOD-10 | P2 | A2A | `agent_roster` (new, uncommitted) returns `200 {count: 0}` for a run id that does not exist, and `503` for any store error, while every sibling run-scoped route refuses `E_RUN_NOT_FOUND` (`404`). A caller cannot distinguish "unknown run" from "run with no agents". | `backend/crates/hiverouter-core/src/http.rs:946-983`; sibling pattern `:772-781`, `:812-821`, `:1022-1031`; store returns `Ok(vec![])` `backend/crates/hiverouter-core/src/store.rs:356-375` | A2A discovery contract, any client mapping refusals to UI states | safe (check `get_run` first, mirror the sibling statuses) |
| MOD-11 | P2 | A2A | The roster endpoint has **no consumer**: no `roster`/`Roster` token exists in `web/src`, and every field it returns (`task_id`, `parent_task_id`, `role`, `kind`, `state`, `depth`, `rev`) is already produced by `history_graph_value`, with `kind` computed by a *different* rule. Two projections of the same authority, one unreachable. | route `backend/crates/hiverouter-core/src/http.rs:522`; handler `:942-983`; existing projection `:701-741` | Dead HTTP surface; future divergence between roster and graph | safe (wire the roster, or delete it before it ships) |
| MOD-12 | P2 | hydration · CAS | Revision authority is **split four ways**: the scrubber lists revisions from the SQLite index (`history.rs:76`), `state_at` authorises a read from the directory (`history.rs:110`), the engine's next revision is "index row count" (`execution.rs:1174-1180`), and recovery's next revision is `max(indexed, on_disk)+1` (`recovery.rs:420-439`). A crash between the file rename and the index insert (`snapshot.rs:76-104`) leaves a file that is readable-but-unlisted, and the next engine commit reuses the same `{revision:010}.json` name and **overwrites** it (`snapshot.rs:82` + `INSERT OR REPLACE`). | `backend/crates/hiverouter-core/src/history.rs:75-115`; `backend/crates/hiverouter-core/src/snapshot.rs:76-104`; `backend/crates/hiverouter-core/src/execution.rs:1174-1180`; `backend/crates/hiverouter-core/src/recovery.rs:420-439` | Time-travel correctness after any crash/partial write; "state 12 of 38" numbers; audit reproducibility | risky (one authority: directory or index, not both; adopt `max+1`) |
| MOD-13 | P2 | history/error UI | `revisions()` swallows a corrupt snapshot with `.ok()`, so a revision whose file cannot be read is listed with `run_status: None`; `state_at` on the same revision refuses `E_SNAPSHOT_CORRUPT`. A corrupt timestamp silently becomes epoch `0`. The scrubber therefore advertises readable history that is not readable, and prints a fabricated date. | `backend/crates/hiverouter-core/src/history.rs:79-80`; read refusal `:109-115`; same file `backend/crates/hiverouter-core/src/snapshot.rs:111-120` | Scrubber list vs read; operator trust in history | safe (surface the refusal on the revision row) |
| MOD-14 | P2 | memory · error UI | `store.messages()` repairs corrupt rows in place — nil UUID, `ChatRole::System`, epoch `0` — instead of refusing. A corrupt transcript is rendered as a system message with a fake id/date, which converts a visible storage failure into fabricated content. | `backend/crates/hiverouter-core/src/store.rs:445-459` | Rehydrated transcripts (the only durable user-visible history) | safe (refuse `E_SCHEMA_INVALID`/`E_SNAPSHOT_CORRUPT`) |
| MOD-15 | P2 | CAS | `begin_rewire`/`finish_rewire` are public API but have **no production caller** (only `tests/graph_authority.rs:67-71`), and the pair is unusable with the real mutation: `rewire` re-inserts the same in-flight marker and would refuse `E_REWIRE_IN_FLIGHT`. `restore_from_snapshot` never clears the marker set, so a crash between begin and finish is invisible to it. | `backend/crates/hiverouter-core/src/graph.rs:109-131`, `:156`, `:298-327` | Future rewire work; stale lock semantics after restore | safe (delete the pair, or make `rewire` consume the begun marker) |
| MOD-16 | P2 | history/error UI · UI truth | The web ships a **second state machine** (`STATE_TRANSITIONS`/`canTransition`) with no production consumer; only tests pin it. It contradicts the server's terminal rule: `failed: ['queued','running','completed']` while `run_state::RUN_TERMINAL_STATES` has no outgoing edge. | `web/src/state/agentState.ts:87-108`; tests only `web/tests/agentState.test.ts:42-45`, `web/tests/state.parity.test.ts:61`; server authority `backend/crates/hiverouter-core/src/run_state.rs:75-105` | Any future UI gating that starts using it would resurrect terminal rows in the UI | safe (delete, or generate from the contract) |
| MOD-17 | P2 | history/error UI | A component mutates the store snapshot directly (`store.setState`) twice — once to push a refusal, once to retract an optimistic message — bypassing the store's own refusal/rollback path and its invariants (`graphRev` bump, dedupe, persistence). | `web/src/chat/ChatCore.tsx:143-148`, `:180-182` | Keyless submit refusal; refusal toast ordering; optimistic rollback | safe (store actions `pushRefusal`/`retractMessage`) |
| MOD-18 | P2 | Watchdog · UI truth | The in-product FigCraft acceptance gate is **stale and false**. Two of eight accepted exports carry hashes that match neither the files on disk nor the watchdog ledger (`a-components`, `a-specimens`), and the gate claims `overall: 'PASS-WITH-NOTES'` at `04:54:33` while the ledger says `PASS` at `05:23:01`. Recomputed SHA-256 for the current files matches the **ledger**, not the product constant. | `web/src/assets/manifest.ts:28-32`, `:59-62`, `:66-69`; ledger `ledgers/ui-watchdog.json:46`, `:55`, `:82`, `:91` + `overall`, `checked_at` | The "these bytes were audited" claim that is the whole point of the seam | safe (regenerate the constants from the ledger, or fail closed on mismatch) |
| MOD-19 | P2 | provider API coverage | The documented `/models` metadata is fetched and **discarded**: `ModelsResponse` parses `context_window`/`max_output_tokens` but nothing consumes them; validation instead uses a compile-time `MAX_OUTPUT_TOKENS = 393_216`. The documented `effort.supported_levels` and `api_capabilities` are not modelled at all, so a configured `reasoning_effort` is never checked against what the model actually supports. | `backend/crates/hiverouter-provider/src/dto.rs:23`, `:547-551`, `:883-908`; fetch `backend/crates/hiverouter-provider/src/client.rs:266-276`; official shape [Lists Models](https://api-docs.deepseek.com/api/list-models) | Model/effort validation for every provider call; future models with different caps | risky (validate against the fetched catalogue; keep the constant as fallback) |
| MOD-20 | P2 | provider API coverage | The provider still serves the legacy ids `deepseek-v4-flash` and `deepseek-v4-flash-vision-exp` ("still accepted … billed at the Flash price"), but the client refuses them at config parse (`from_id` → `None`, pinned by a test named "legacy id refused"). A still-valid configuration fails at boot with `E_SCHEMA_INVALID`. | `backend/crates/hiverouter-provider/src/dto.rs:49-53`, `:774`; config use `backend/crates/hiverouter-core/src/config.rs:94-97`; official statement [Your First API Call](https://api-docs.deepseek.com/) | Operator configuration; upgrade path from older installs | out of scope (product decision: accept-and-map vs refuse-by-design) |
| MOD-21 | P2 | scheduler/scale | Scalability limits are structural: one global `Mutex<Connection>` serialises every query including HTTP snapshot reads; `history_graph_value` does a linear `find` per node over the run's tasks (O(n²)); the spawn queue is a `Vec` with `contains` + `remove(0)` (O(n) per pop). Nothing is wrong at current caps, all three grow with graph size. | `backend/crates/hiverouter-core/src/db.rs:295-350`; `backend/crates/hiverouter-core/src/http.rs:706-720`; `backend/crates/hiverouter-core/src/execution.rs:820-909` | History replay of larger runs; HTTP latency while a run writes; fan-out at the cap | risky (pooled connections / read-only connection; HashMap index; `VecDeque`) |
| MOD-22 | P2 | scheduler/scale | `stream_reply` holds the per-run budget mutex (`budgets.lock().await`) across the **entire** network stream, so no second stream can start while one is open. Correct for today's single-active-run rule, but it hard-serialises any future multi-run/parallel-subagent execution, and a hung stream holds the lock with it. | `backend/crates/hiverouter-core/src/provider_executor.rs:140-286` | Concurrency ceiling for any future parallel dispatch; failure isolation | risky (scope the lock to reserve/settle) |
| MOD-23 | P2 | Remote · security | `--bind 127.0.0.1:0` is an accepted, tested production configuration (`config.rs:425-434`), but it produces an **empty** origin allowlist (`:205-207`) and relaxed Host checking (`http.rs:571-584`). The server prints a live URL and serves the UI, yet every `/ws` upgrade is refused `E_ORIGIN_REJECTED` — an unreachable WebSocket in a mode the CLI advertises as valid. | `backend/crates/hiverouter-core/src/config.rs:205-207`, `:425-434`; `backend/crates/hiverouter-core/src/http.rs:571-584`; `backend/crates/hiverouter-core/src/ws.rs:72-84`, `:379-381` | Any port-0 deployment; the security posture of the Host/Origin pair | risky (derive origins after bind, or refuse port 0 outside tests) |
| MOD-24 | P3 | history/error UI | Three stale comments assert the opposite of the code: `execute_send_to_sibling`'s "Same-parent only … until a profile-level cross-branch approval gate lands" (the gate exists at `:1705-1728`), `GraphSnapshot`'s "nodes, in insertion order" (sorted by UUID), and the history projection's "snapshot … may predate run-scoped capture" (commit-time capture is already scoped). | `backend/crates/hiverouter-core/src/execution.rs:2499-2503`; `backend/crates/hiverouter-core/src/graph.rs:34` vs `:331-332`; `backend/crates/hiverouter-core/src/http.rs:699-700` vs `:1189-1216` | Reviewer trust; future edits that believe the comment | safe |
| MOD-25 | P3 | Remote | Every socket `Ping` publishes a **broadcast** `PONG` to all sessions (`publish_pong`), consuming a global sequence number and a replay-buffer slot per keepalive. Keepalive traffic is not session state. | `backend/crates/hiverouter-core/src/ws.rs:435-438`, `:1029-1031` | Replay-window pressure; sequence churn with multiple clients | safe (answer the socket, don't publish) |
| MOD-26 | P3 | Subagent · UI truth | The four Library template buttons send ids that the server can never resolve (`TemplateDagCatalog` refuses them by name because no prompt asset exists). Every click is a guaranteed `E_TEMPLATE_UNKNOWN` toast. | `web/src/panels/LibraryPanel.tsx:8-13`; refusal path `backend/crates/hiverouter-core/src/ws.rs:122-132`, `:219-228` | Operator-visible dead controls | safe (ship the four `.prompty` assets, or hide the buttons) |
| MOD-27 | P3 | history/error UI | `history_graph_value` clones the full run task list and does a linear `find` per node; it also re-implements the run filter that `capture_scoped` already applied at write time, so two authorities can disagree for old snapshots without any test. | `backend/crates/hiverouter-core/src/http.rs:701-741`; write-side filter `backend/crates/hiverouter-core/src/snapshot.rs:264-285` | Replay cost and consistency for pre-1.3 snapshot files | safe (build a `HashMap`, keep the filter only as legacy) |
| MOD-28 | P3 | all | Formatting is not gated: `cargo fmt --check` reports 538 hunks repo-wide, and the uncommitted roster hunk additionally collapses a function onto one line (`async fn agent_memory_list(...) -> Response {    let mut profiles…`). | `backend/crates/hiverouter-core/src/http.rs:985`; whole-workspace `cargo fmt --all -- --check` output | Diff noise; merge conflicts; reviewability | safe (add a fmt gate, format once) |

### Files examined (inventory)

Rust: `hiverouter-contract` (`event.rs`, `state.rs`, `refusal.rs`, `command.rs`), `hiverouter-core` (`http.rs`, `ws.rs`, `execution.rs`, `run_state.rs`, `store.rs`, `db.rs`, `graph.rs`, `history.rs`, `snapshot.rs`, `recovery.rs`, `provider_executor.rs`, `tools_bridge.rs`, `main.rs`), `hiverouter-tools` (`approval.rs`, `watcher.rs`, `health.rs`), `hiverouter-provider` (`dto.rs`, `client.rs`).
Web: `state/store.ts`, `state/agentState.ts`, `state/metrics.ts`, `api/client.ts`, `api/contract.ts`, `api/refusals.ts`, `flow/graph.ts`, `flow/Canvas.tsx`, `flow/nodes.tsx`, `chat/ChatCore.tsx`, `panels/LibraryPanel.tsx`, `assets/manifest.ts`, `tests/*`.
Evidence files: `ledgers/ui-watchdog.json`, `ledgers/ui-watchdog.md`, `docs/ui/figcraft/ASSETS/*`.

---

## 2. P0 / P1 / P2 ranking

### P0 — none found

No finding is P0. I checked the usual P0 classes explicitly and they hold:

- **Secret handling** — the key is read from env only, zeroized/redacted (`hiverouter-provider/src/secret.rs`), never in the session payload; `redact_args_preview` bounds and redacts approval previews (`hiverouter-contract/src/event.rs:286-340`).
- **Approval CAS** — `verify_grant` selects by `(run, tool, args_sha256)` then claims with `UPDATE … WHERE executed_at IS NULL` and treats `changed != 1` as `E_APPROVAL_REPLAY` (`hiverouter-tools/src/approval.rs:913-981`); resolution is `WHERE decision IS NULL` (`:564-580`). No double-execution path found.
- **Terminal-state resurrection** — enforced twice (SQL `WHERE state NOT IN (<terminal>)`, `store.rs:297-354`, plus the in-memory state machine `run_state.rs:457-476`).
- **Loopback-only transport** — bind validation (`config.rs:327-334`), `ConnectInfo` loopback check (`ws.rs:371-375`), Origin allowlist (`ws.rs:376-381`), Host guard (`http.rs:543-584`).

The two closest candidates are MOD-05 (silent event loss inside the documented replay window) and MOD-04 (reconnect that closes a healthy socket). Both are P1 until a test reproduces them against a live socket; see §5.

### P1 — fix before the next sealed release claim (7)

| ID | Why P1 | Evidence | Fix direction (not applied) |
|---|---|---|---|
| MOD-01 | Two authorities for the same wire field; the contract names this exact failure | `http.rs:743-751` vs `event.rs:232-240`; live `execution.rs:99-106` | Persist the tier on the task row (or omit `kind` in HTTP and let the client use the event) |
| MOD-02 | Every recovered run renders wrong node types; the producer deliberately omits `kind` for this reason | `store.ts:1504-1507`, `recovery.rs:443-446` | Keep `null`; generic card/label; count unknown as unknown |
| MOD-03 | The resync cannot do its job — stale local state wins over the server | `store.ts:1175-1195` | Merge per task by `rev`; keep live-only tasks |
| MOD-04 | Duplicate session opens and duplicate approval replays on every transient drop | `client.ts:325-330`, `:377-413` | Cancel the timer on successful `open`; single-flight the refresh |
| MOD-05 | Silent lost events are not observable and not refusable | `ws.rs:526-528`, `event.rs:176-182`, `store.ts:1365` | Echo the client's reported seq (or the last replayed seq) |
| MOD-06 | Persona identity is split; imported profiles lose memory, tools and spawn limits silently | `execution.rs:880`, `http.rs:987`, `execution.rs:550-558` | One key; migrate; keep a role fallback read |
| MOD-07 | A template insert can promote a child's capability tier and hide the node | `ws.rs:240-291`, `execution.rs:154`, `graph.ts:235` | Validate the spawn role; fix the canvas filter for non-root hyperagents |

MOD-04 and MOD-05 are also transport (Remote) findings; they are grouped under hydration because that is where their damage shows.

### P2 — structure, latent bugs and evidence truth (16)

MOD-08 (fabricated `rev`), MOD-09 (two replay decoders), MOD-10 (roster status codes), MOD-11 (dead roster surface), MOD-12 (revision authority split + overwrite hazard), MOD-13 (swallowed corrupt snapshot), MOD-14 (silently repaired chat rows), MOD-15 (unusable rewire CAS pair), MOD-16 (dead contradictory state machine), MOD-17 (component mutates store), MOD-18 (stale FigCraft gate — 2/8 hashes wrong, verdict wrong), MOD-19 (discarded `/models` metadata), MOD-20 (legacy model ids refused — product decision), MOD-21 (DB mutex + O(n²) hot paths), MOD-22 (budget mutex across the stream), MOD-23 (port-0 mode has no usable WebSocket). MOD-20 is out of scope for engineering; the rest are in scope.

### P3 — hygiene and dead surface (5)

MOD-24 (stale comments), MOD-25 (global PONG), MOD-26 (guaranteed-refusal template buttons), MOD-27 (duplicate run filter), MOD-28 (fmt gate absent).

### Safe modernization opportunities, ranked by (risk ↑) × (value ↓)

1. **MOD-02, MOD-03, MOD-08, MOD-09** — web decode/merge fixes; no wire change; covered by existing web tests (`graph.replay.test.ts`, `store.test.ts`) plus one new fixture per rule.
2. **MOD-15, MOD-16, MOD-24, MOD-28** — delete dead code / stale comments, add `cargo fmt` gate; mechanically verifiable (`cargo fmt --check`, `vitest run`).
3. **MOD-10, MOD-11, MOD-13, MOD-14** — honesty at the read boundary; each needs one refusal-path test.
4. **MOD-27, MOD-21** — replace `Vec::find`/`remove(0)`/per-node `find` with indexed structures; behaviour-preserving, benchmarked.
5. **MOD-19** — validate against `/models`; needs a version-pinned fetch + offline fixture.
6. **MOD-01, MOD-04, MOD-05, MOD-06, MOD-07, MOD-12, MOD-23** — each changes semantics; treat as features with their own failing test first.

---

## 3. Bugs to add to known areas

Concrete bugs only; each row is "what to add to the area's bug list", with the finding id.

| Known area | Add to its bug list | ID |
|---|---|---|
| **Hydration** | Resync merge overlays stale live task records over the authoritative snapshot without comparing per-task `rev`; a missed `NODE_STATE` cannot be repaired by the resync. | MOD-03 |
| | The web cursor is advanced to the server's current seq at `SERVER_HELLO`, before the replayed range is applied; a disconnect mid-replay loses events silently (`Replay::Current`, no gap). | MOD-05 |
| | Snapshots can be readable-but-unlisted (file written, index row missing); the next commit computes the same revision and overwrites the file. Two next-revision formulas exist (`count` vs `max+1`). | MOD-12 |
| | Approval/approval-window replay is restarted by a redundant session re-open 1.5 s after a successful reconnect. | MOD-04 |
| **CAS** | Replay decoder fabricates `rev: 0` for a snapshot node that has no rev; the rewire guard treats 0 as a confirmed server revision (currently latent because replay is read-only). | MOD-08 |
| | `begin_rewire`/`finish_rewire` cannot be used with `rewire` (same in-flight marker) and are never called in production; `restore_from_snapshot` does not clear stale markers. | MOD-15 |
| | Revision numbers are not monotonic under partial writes, so `expected_rev` history and the scrubber's revision identity are not reproducible. | MOD-12 |
| **History / error UI** | HTTP replay/roster infers node `kind` from the free-form `role`, contradicting `NODE_KIND_VOCABULARY`'s "never infer from role" rule; live and replay disagree (`worker` vs `subagent`). | MOD-01 |
| | Two `HISTORY_SNAPSHOT` decoders (strict vs lenient) with different failure semantics; Canvas uses the lenient one. | MOD-09 |
| | `revisions()` hides a corrupt snapshot as `run_status: None` and fabricates epoch-0 timestamps; `state_at` refuses the same revision. | MOD-13 |
| | `store.messages()` silently rewrites corrupt rows (nil UUID, `System` role, epoch 0) instead of refusing. | MOD-14 |
| | `ChatCore` mutates the store directly for refusals and optimistic rollback, bypassing the store's own path. | MOD-17 |
| | A second, unconsumed state machine (`STATE_TRANSITIONS`) contradicts the server's terminal stickiness. | MOD-16 |
| **Subagent / Remote / Watchdog** | Memory, persona tool allowlist and persona spawn limits are keyed by `role` while the API lists/clears by `persona.id`; imported profiles silently lose all three. | MOD-06 |
| | `TEMPLATE_INSERT` bypasses `validate_spawn_role`; the only shipped asset (`planner`) maps to HyperAgent, so an inserted child is a capability promotion and is filtered out of the canvas. | MOD-07 |
| | Web coerces absent `kind` to `worker`; boot-recovered nodes specifically omit `kind`, so recovery re-types every node in the UI. | MOD-02 |
| | `--bind 127.0.0.1:0` is accepted and advertises a URL, but the empty origin allowlist refuses every WebSocket upgrade. | MOD-23 |
| | The FigCraft/watchdog acceptance gate hardcodes 2/8 stale hashes and a stale overall verdict/timestamp. | MOD-18 |
| | Four Library template buttons always refuse `E_TEMPLATE_UNKNOWN`. | MOD-26 |
| **Memory** | Same key split as MOD-06: writes under role, reads/clears under persona id; failing recall (`if let Ok(memories) = …`) is indistinguishable from "no memories" (`execution.rs:880`). | MOD-06 |
| | Memory values are re-filtered only by a 4-marker substring check (`memory_text_is_safe`, `execution.rs:61-65`) before being stored as model-visible facts. | MOD-06 (adjacent) |
| **A2A** | New roster endpoint returns `200 count:0` for an unknown run and `503` for store errors, unlike every sibling run route (`404 E_RUN_NOT_FOUND`). | MOD-10 |
| | Roster is unwired and duplicates the history graph projection with a different `kind` rule. | MOD-11 |
| | The doc comment on the sibling-message path still says the cross-branch approval gate has not landed, while it has. | MOD-24 |
| **Scheduler / scale** | Single global SQLite mutex serialises all reads/writes, including HTTP snapshot reads; spawn queue is an O(n) `Vec`; history projection is O(n²). | MOD-21 |
| | Provider budget mutex is held across the whole network stream, hard-serialising any future concurrent execution. | MOD-22 |
| **Provider API coverage** | `/models` metadata (per-model `context_window`/`max_output_tokens`, `effort.supported_levels`, `api_capabilities`) is parsed and discarded; `max_tokens` validation uses a hardcoded constant instead of the server's reported limit. | MOD-19 |
| | Legacy ids the provider still serves (`deepseek-v4-flash`, `deepseek-v4-flash-vision-exp`) are refused at config parse. | MOD-20 |

---

## 4. Cross-checks I ran against the "known areas" list

| Known area | What I checked | Result |
|---|---|---|
| hydration | WS hello/replay ordering, HTTP snapshot/messages hydration, resync merge, cursor ownership | MOD-03, MOD-05, MOD-12 confirmed |
| CAS | `GraphAuthority::rewire` expected_rev, approval grant CAS, terminal-state SQL guard | grant CAS sound; MOD-08/MOD-15/MOD-12 in the revision layer |
| history / error UI | scrubber list vs read, replay decoders, refusal rendering, transcript repair | MOD-01/09/13/14/17 confirmed |
| Subagent / Remote / Watchdog | spawn policy/caps, subagent card path, persona identity, transport gating, watchdog ledger | MOD-02/06/07/18/23/26 confirmed |
| memory | write/read/clear keying, recall failure mode, redaction | MOD-06 confirmed |
| A2A | `ask_parent`/`send_to_sibling` bounds, cross-branch gate, roster | message budget + args binding sound; MOD-10/11 confirmed |
| scheduler / scale | dispatch order, spawn queue, caps, DB access, provider concurrency | MOD-21/22 confirmed |
| provider API coverage | documented endpoints/model ids/params vs DTO + validation (official docs fetched) | MOD-19/20 confirmed; model ids and `max_tokens` range themselves are **documented and correct** |

---

## 5. Unverified / limits of this audit

1. **P0 none** is based on reading the security/CAS/terminal paths, not on a red-team exercise; no browser, no live provider, no HiveMind start/stop (seat rules).
2. **MOD-04/MOD-05** are derived from code paths and the documented wire semantics; I did not reproduce a mid-replay disconnect or a partysocket fast reconnect. Reproduction needs a transport double plus a real socket — no such test exists today (`web/tests/apiClient.test.ts` uses a fake transport). That is the first thing I would have written.
3. **Rust baseline** covers only `snapshot_history` (4 tests). The full workspace suite was not run: it is large (≈60 test binaries) and outside this seat's budget. No claim in this report depends on the full suite.
4. **MOD-12** overwrite hazard assumes the engine's `snapshot_revision` can observe the index before a pending file is indexed. I verified the two formulas and the write order in code; I did not construct the crash.
5. **MOD-18** hashes were recomputed on this machine (`Get-FileHash -Algorithm SHA256`) and match the ledger and the files; only the product constants are stale. Whether a *future* asset replacement should fail closed is a product decision.
6. **MOD-20** depends on the provider's statement that the legacy ids are still accepted; the docs are live but the behaviour was not probed (no key use in this seat).
7. `cargo fmt --check` output is large; the 538-hunk count is the count of `Diff in …` lines printed by the toolchain on this host, not a stable repository property (a different rustfmt version can change it).
8. I did not run `clippy`; lint-level modernization findings (e.g. `&String` params, `map_or(true, …)`) are therefore **not** included.

---

## 6. Method note (skill compliance)

`deep-modernize` refuses to transform without a pinned baseline and a named target version. Both were treated as hard gates:

- **Baseline** — quoted in §0 (web 187/187; rust 4/4 for the touched binary; tsc 0; fmt red, pre-existing).
- **Target** — `edition 2021`, `rust-version 1.89.0`, TypeScript 7.0.2 / React 19.3.1 / Vite 8.3.6 from the manifests. No target was invented.
- **Ledger** — identity (no rows transformed), because the seat's own instruction ("no product code edits, no fixes") takes precedence over the skill's transform procedure. Every finding above is therefore *reported*, not applied; §2's "fix direction" column is a review note, not a change.
