# 17 — Scale, scheduler, and security implementation design (second wave, lane 8)

**Lane:** Scale/scheduler/security implementation design · **Read-only** · **Date:** 2026-11-07 (America/Chicago)
**Repository:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD read:** `bd575ba` `feat: real SuperAgent/Subagent personas and persona prompt execution`
**Tree at read:** `M backend/crates/hiverouter-core/src/provider_executor.rs` (+7/−2 dirty), `M docs/provider/AGENT-ARCHITECTURE-TRUTH.md`, `M` pycache/`HiveMind.exe`, `?? .claude/`, `?? AGENTS.md`, `?? CLAUDE.md`, `?? docs/research/`. **The dirty `provider_executor.rs` is a real, uncommitted persona/continuation fix — commit C0 below, do not duplicate it.** During this read-only lane other lanes also left `M execution.rs`, `M tool_loop.rs`, `M HiveMindLauncher.cs` uncommitted; those are concurrent work, not this lane's edits. Treat every C-commit as rebasing onto the full dirty set and verifying `git diff` file ownership before writing.
**Discipline used:** `/deep-inspection` is **not available** in this profile; this lane used **`/deep-investigate`** for the baseline → three-hypotheses → elimination method and **`/trace`** for the UI/event/terminal-state ladder (render → handler → client → route → provider → store → event → render-back). No test, build, server restart, paid call, or credential read was performed. Runtime statements are `UNVERIFIED`; code-path statements are `READ`/`VERIFIED`.
**Inputs read:** `docs/research/hmars/OPERATOR-DECISIONS.md`, `09-synthesis-and-goal.md`, `03-scale-concurrency.md`, `06-security-cost.md`, `07-code-backend.md`, plus `02-hmars-patterns.md`, `05-ui-ux-patterns.md`, `01-deepseek-api.md`, and the product source at the seams cited below.
**Label legend:** `VERIFIED` = confirmed by two independent source/command reads; `READ` = read in one cited source; `INFERRED` = derived from cited code; `UNVERIFIED` = runtime not executed; `READ-DOC` = research document (not product proof).

---

## 0. One-screen answer, and the ten gaps that matter

**The order is not the docs' order.** Five different first-wave artifacts nominate six overlapping but non-identical P0 sets (`09-synthesis-and-goal.md:247`; `06-security-cost.md:1076-1089`; `02-hmars-patterns.md:885-896`; the operator decisions; and this lane's brief). The implementation order must be: **tree hygiene → contract delta → Host/token → provider truth + preflight → spawn authority → parent-completion guarantee → deadline/stall → (then, gated) S0 harness → S1 admission → S2 storage/coalescing → S3 sessions → S4 AIMD/budget → S5 600-task UI.** Scale work before the P0s would amplify the existing security/correctness holes.

**The ten gaps (each is detailed in §2/§5):**

1. **P0 ownership is contradictory and incomplete.** `09:247` lists 8 P0s, `06:1076-1089` lists 6 (different set), the lane brief lists 6 again; **`ask_parent` answer/deadline and planner-blank retry are orphaned**, while `06`'s projected-cost, hive-budget and tool-result-fence P0s are absent from the lane brief. No single owner means two P0s can be "done" by nobody.
2. **The dirty tree already contains a P0 fix outside git.** `provider_executor.rs` has +7/−2 applying role normalization and persona-on-continuation (`git diff` at read); it is **not committed and has no dedicated test**, and `09:247` still lists "persona continuation + role normalization" as outstanding. C0 must land it first, with its own acceptance test, or a later checkout loses it.
3. **No Host allowlist + unauthenticated HTTP mutations/reads.** `http.rs:510-529` registers profile-import, profile-delete, memory-clear, history and agent-message endpoints; none reads a token, and `api_session` accepts a loopback peer with no Origin (`http.rs:1017-1043`). DNS rebinding yields a standing token (`06:528-566`; `[READ-DOC]`) — a P0 that Host validation closes in one middleware, but the token half of T12 remains.
4. **402/400/422 are misclassified.** `client.rs:507-524` maps 401 **and 402** to `E_PROVIDER_AUTH` and everything else including 400/422 to `E_PROVIDER_INVALID_RESPONSE`; the operator is told to rotate a good key when the balance is empty (`06:809-827,955-984`). The existing tests pin the wrong behaviour (`provider/tests/error_matrix.rs:161-180`: `payment_required_402_is_not_retried` asserts `E_PROVIDER_AUTH`).
5. **No context-window preflight.** `dto.rs:892-907` already parses `context_window`/`max_output_tokens`, but nothing in `provider_executor`/`budget` consumes it; the only estimate is `serde_json bytes/4` (`provider_executor.rs:436-442`), which the first wave itself calls under-counting (`06:748-770`; `09:261`). A 1M-context prompt can be built and sent before a 400.
6. **No parent-completion guard.** `complete_task` (`execution.rs:2973-3030`) checks task state but never descendants; the command-spawn queue is drained only **after** aggregation (`execution.rs:1486-1490`), and `spawn_sub_hive` checks only the **run** terminal state (`:1069-1073`). A queued child can be created under a parent that is already Completed. `02:477,707` records this as I6 PARTIAL.
7. **No run deadline and no stall detector.** `dispatch` (`execution.rs:1245-1307`) awaits `dispatch_inner` forever; the only timeout is the provider client's 600 s request timeout (`provider/src/client.rs:60-66`). A run can loop/spend 40 calls and 10+ h (`06:947-951`; `02:717`). `09:247` owns this but no code exists.
8. **The budget mutex serializes all provider streams, so AIMD/permits alone will not scale.** `provider_executor.rs:104-110,140-147` holds `tokio::Mutex<HashMap<RunId,RunBudget>>` across the whole `complete`/`stream_with_budget` await, and `stream_reply` holds it through the entire SSE loop (`:162-243`). Even 150 permits in `budget.rs:43` would run one stream at a time. `06:828-847` names this (T26) but the second-wave component list does not fix it.
9. **The first wave's scale plan disagrees on priority and prerequisites.** `03` makes admission the first change (S1 `:645-648`), `09` schedules the scheduler at P2 (`:249`), the operator puts hierarchy/memory first (`OPERATOR-DECISIONS.md:15`), and S1/S3 acceptance requires refusal codes and a viewport command that do not exist in the frozen contract (`E_QUEUE_FULL`, `E_DEADLINE`, `E_SESSION_SLOW`; `03:648,650,571`; refusal table `contract/src/refusal.rs:75-274`; no such command exists in the frozen command set).
10. **Operator policy conflicts are unresolved.** "No hard default budget" (`OPERATOR-DECISIONS.md:24`) vs the hard $0.50/400k/40-call caps (`budget.rs:24-43`); "no silent bypasses" (`:21-22`) vs the ungated `spawn_agent` (`execution.rs:1684-1691`) and same-parent sibling (`:1719-1720`); "per-hive fair admission queue default" (`:102`) vs single active run (`execution.rs:1032-1034`). A design that silently keeps all current defaults will contradict the operator; one that silently removes them contradicts the security model.

The remaining sections are the exact fixes, seams, commit order, tests, rollbacks and gates needed to close these without touching product code in this lane.

---

## 1. Method, evidence basis, and first-wave baseline

### 1.1 Baseline recorded

- HEAD `bd575ba`; at first read the only source change was the uncommitted `provider_executor.rs` diff (added `system_prompt_for` normalization at `:82-92` and persona prompt on continuation at `:375`; HEAD hard-codes `TASK_SYSTEM` at `:369-372`). `git diff --stat` at that moment = 1 file, +7/−2. **Later in the lane, `execution.rs`, `tool_loop.rs` and `HiveMindLauncher.cs` also became dirty from other lanes**; those edits are not analysed here and must be treated as concurrent. `READ` + `git diff`.
- `cargo` tests were **not** run: read-only assignment. The current test suite contains `provider/tests/error_matrix.rs`, `provider/tests/refusal_matrix.rs`, `core/tests/{provider_streaming,provider_adapter,tool_loop_engine,ws_transport,...}.rs`, and no `http_origin.rs`, `context_window.rs`, `parent_completion.rs`, `run_deadline.rs`, `scheduler.rs`, `db_actor.rs`, `delta_coalesce.rs`, `session_outbound.rs`, `aimd.rs`, or `cost_governor.rs` (`Get-ChildItem` at read). The acceptance tests below are therefore **new files/tests**, all offline.
- `CHANGE-LEDGER-IMPL.md` exists at the **repository root** (`C:\...\hiverouter-v1\CHANGE-LEDGER-IMPL.md:1-8`), despite some docs pointing to a `docs/contracts`-relative name. Contract codes may be added only with a ledger row (`refusal.rs:5-7`).
- The provider wire client is `hiverouter-provider`; core is async Tokio; SQLite `rusqlite 0.41.1` bundled; `tokio = 1.54.1`, `axum = 0.8.9`, React Flow `12.12.1` (`backend/Cargo.toml:12-43`; `web/package.json`).

### 1.2 `/deep-investigate` result for the six P0 symptoms

The skill's discipline requires a baseline, three ranked hypotheses, elimination evidence, and a reproduction. This is a **read-only design** task, so no live reproduction exists; runtime causes remain hypotheses, while **code-path causes are eliminated deterministically** by the source reads below.

| Symptom | H1 (provider) | H2 (local policy/route) | H3 (UI/observability) | Elimination result |
|---|---|---|---|---|
| `spawn_agent` has no approval | provider requires it | **loop dispatch bypasses approval branch** | approval UI absent | H1/H3 eliminated: `execution.rs:1684-1691` executes before `:1721-1734`; `06:372-401` agrees. H2 **confirmed by source**. |
| 402 looks like an auth error | provider sends 401 | **status classifier merges 401/402** | UI string says rotate key | H1/H3 eliminated; H2 confirmed at `client.rs:507-524` + `error_matrix.rs:161-180`. |
| Rebound page can mutate | network reachability | **no Host check + no token on routes** | CORS blocks it | H1/H3 eliminated: loopback peer + no Origin path in `api_session:1025-1033`; router has no Host middleware `:510-531`; `06:528-566` confirms. H2 confirmed. |
| Over-long prompt discovered after send | provider rejects | **no context-window preflight**; estimator is bytes/4 | Models panel displays 1M | H1 eliminated (documented 400/422 after send), H3 not causal; H2 confirmed by absence of `context_window` read (`grep` over provider/core) and `estimate_prompt_tokens:436-442`. |
| Parent completes before child | child already terminal | **completion checks state only; spawn queue drained last** | UI shows parent completed | H1 eliminated (spawn is engine-owned), H2 confirmed at `execution.rs:1486-1490,2973-3030,1069-1073`. |
| Run can hang indefinitely | provider timeout | **no run-level deadline/stall** | no progress indicator | H1 eliminated: provider has 600 s request and 600 s idle timeout (`client.rs:59-66`); H2 confirmed by no deadline in `Config` (`config.rs:190-195`) or `execution.rs`. |

**No `E_NOT_REPRODUCIBLE` reproduction is claimed:** all six are proven as code paths, not as live failures. A live probe requires a build and a run, both out of scope here.

### 1.3 `/trace` ladder for UI/event/terminal claims

Applied to the affected surfaces; verdicts:
- **Composer → `SUBMIT_PROMPT` → run**: WIRED-AND-WORKS by code (`ChatCore.tsx:77-86,140-152`; `client.ts:1835-1847`); runtime unverified.
- **`REPLY_DELTA` → canvas**: emitted per fragment (`execution.rs:1764-1795`) with no coalescing; UI store applies every event; at scale this is the failing hop (S2).
- **Approval card**: handler/route exist (`approval-flow.spec.ts`), but `spawn_agent` never reaches approval (P0-1).
- **Host/token security**: the `/api/session` token is fetched and placed in the WS **query string** (`web/src/api/client.ts:290-310`); no HTTP `Authorization` path exists (`grep` over `client.ts`, panels). Terminal truth for rebinding: MISSING.
- **Parent completion truth**: parent task can be `Completed` while a child row is `Queued` — the terminal record is **not truthful** in that race (P0-5).
- **LOD**: canvas passes all `nodes`/`edges` with no `onlyRenderVisibleElements` (`Canvas.tsx:351-405`); the render-back hop is WIRED-BUT-UNSCALED.

---

## 2. First-wave gap and contradiction audit

### 2.1 P0 ownership matrix

| P0 item | `09-synthesis` | `06-security-cost` | This lane's brief | Code state | Ruling |
|---|---|---|---|---|---|
| `ask_parent` answer/deadline | P0 (`09:247`) | not P0; T02 is P3 | **absent** | no inbox/wait (`execution.rs:2417-2487`) | **Orphaned.** Must be owned by memory/A2A lane or explicitly deferred with a gate; not silently dropped. |
| Parent-completion guard | P0 (`09:247`) | not listed | **required** | absent (`execution.rs:2973-3030`; `02:707`) | Lane owns (C5). |
| Spawn approval | P0 (`09:247`) | **P0** T07 (`06:372-401`, decision P0-3 `:1082`) | **required** | bypass (`execution.rs:1684-1691`) | Lane owns (C6), but needs an operator decision on approve-per-spawn vs declared `Delegate` (Q1). |
| Host allowlist | P0 (`09:247`) | **P0** T13/P0-1 (`06:528-560,1080`) | **required** | absent (`http.rs:510-531`) | Lane owns (C2). |
| 402/400/422 mapping | P0 (`09:247`) | **P1** T25 (`06:809`) + P0 matrix row? (`06:962-965`) | **required as P0** | wrong mapping (`client.rs:507-524`) | **Severity conflict.** This lane treats it P0 because balance misclassification causes repeated spend; record a severity decision in the ledger (Q5). |
| Context-window preflight | P0 (`09:247,261`) | not in `06` P0-1..6; gap H in `01:739` | **required** | absent (`provider_executor.rs:436-442`) | Lane owns (C4). |
| Run deadline/stall | P0 (`09:247`) | not in P0-1..6; `06:947-951` names only a proposed deadline | **required** | absent (`execution.rs:1245-1307`) | Lane owns (C7). |
| Persona continuation + role normalization | P0 (`09:247`) | **P0** T04/P0-6 (`06:313-331,1085`) | not listed (dirty fix exists) | dirty working tree (`provider_executor.rs:82-92,375`) | **C0 must land it + test it.** Do not re-implement. |
| Planner JSON blank retry | P0 (`09:247`) | not listed | **absent** | no retry (`provider_executor.rs:291-318,452-488`) | **Orphaned.** Add to C8 or assign to the provider lane. |
| Projected-cost preflight | not in `09` P0 | **P0** T22/P0-4 (`06:748-770,1083`) | not listed (but implied by cost governor) | preflight checks `spent` only (`budget.rs:235-284`) | **Lane's brief is incomplete vs 06.** Fold into C3/C15; do not ship AIMD without it. |
| Hive/global budget | not in `09` P0 | **P0** T21/P0-5 (`06:721-746,1084`) | not listed | no hive cap (`provider_executor.rs:104-107`) | **Lane's brief is incomplete vs 06.** Schedule C15/S4 before >8 concurrent; audit must say so. |
| Tool-result fence + memory trust | not in `09` P0 | **P0** T01/T04/P0-6 (`06:226-270,313-331,1085`) | not listed | raw tool text enters continuation (`provider_executor.rs:399-403`); `memory_text_is_safe` is 4 substrings (`execution.rs:61-65`) | **Not this lane's core**, but a hard gate for opening web-content ingestion; note as G-gate, not implemented. |

### 2.2 Contradiction table (first wave vs first wave, docs vs code)

| # | Contradiction | Evidence | Severity | Ruling / required action |
|---|---|---|---|---|
| A1 | P0 sets differ across docs; two P0s orphaned | `09:247` vs `06:1076-1089` vs lane brief; `02:885-896` | **Blocker** | Make `09:247` the union; add an owner column; C0's tree-hygiene commit includes this bookkeeping. |
| A2 | 402/400/422 severity: P0 vs P1 | `09:247`; `06:809` | High | Treat as P0 for implementation (cheap, prevents repeated spend); ledger records the override. |
| A3 | Contract additions are required by acceptance tests but absent from the plan | `03:648,650,571` demands `E_QUEUE_FULL`, `E_DEADLINE`, `E_SESSION_SLOW`; `refusal.rs:75-274` has none; `03:652` S3 viewport subscription is not a frozen command | **Blocker for S1/S3** | Add a contract-delta commit before S1/S3; do not fake codes by reusing unrelated ones (see Q5). |
| A4 | Scale priority is inconsistent | `03:645-648` S1 admission first; `09:249` scheduler P2; `OPERATOR-DECISIONS.md:15` hierarchy/memory first | High | This design keeps the P0 correctness wave first, then S0 harness, then S1; memory/A2A remain a parallel P1 lane and must not block the scheduler design. |
| A5 | "8 permits bind first" is misleading before parallel dispatch exists | `03:16,94`; actual execution is sequential (`execution.rs:1395-1550`), so in-flight is at most 1 today; `03`'s own S0 says throughput ~0.4 calls/s | Medium | S0 harness must measure the **sequential** baseline first; `IN_FLIGHT_LIMIT` is the first ceiling **after** the dispatcher exists, not today. |
| A6 | The budget mutex serializes streams; the scale architecture does not list it | `provider_executor.rs:104-110,140-147`; `06:828-847` T26 | **High** | AIMD/permits must ship with the budget-lock split (C14); otherwise S4 acceptance cannot pass. |
| A7 | "Process-wide 8 permits" | `03:94`; limiter is per `ClientConfig` (`client.rs:51-66`), created once per main client and again per provider proxy call (`http.rs:885,908`) | Low | Say "per client"; the main adapter is one client; metadata proxies add clients and can exceed 8 total, worth fixing under C1/C18. |
| A8 | Docs cite stale line numbers in places | `03:101` bus.rs:20-22 vs actual `:18-20`; `03:102` execution.rs:1769-1795 vs actual `:1764-1795`; `03:103` Canvas:351-435 is current | Low | Accept claims, pin line numbers at implementation; do not base acceptance on stale lines. |
| A9 | Host-only fix vs 06 P0-2 token requirement | `06:1080-1081` P0-1 Host and P0-2 token; lane brief says "Host allowlist" only | **High** | Host middleware is C2; the token/Origin gate on mutating/read endpoints is C2b (small, same test); leaving it out leaves the T12 hole. |
| A10 | Operator says "no hard default budget" | `OPERATOR-DECISIONS.md:24`; current hard caps `budget.rs:24-43`; 06 P0-5 adds a hard hive cap; 03 S4 requires global hard cap | **High (product policy)** | Q17: keep safety caps? Recommended: current per-run caps remain during the security wave; replace with defaults from settings (soft alert + emergency stop) only after the migration/load gate; no silent removal. |
| A11 | Operator says "no silent bypasses" | `OPERATOR-DECISIONS.md:21-22`; spawn bypass (`execution.rs:1684-1691`), same-parent sibling Mutate bypass (`:1719-1720`) | **High** | P0 spawn fix must be explicit in audit + setting; same-parent sibling is a P2 follow-up (T11), not silently accepted. |
| A12 | Operator wants per-hive fair admission; code has one active run and a global FIFO permit | `OPERATOR-DECISIONS.md:102`; `execution.rs:1032-1034`; `budget.rs:371-431` | Medium | S1 implements per-hive fairness **within one run**; multi-run is deferred (Q11). |
| A13 | `09` and `06` disagree on whether the current app "has UI claims" | `09:264`; `08`/UI are a different lane | Low | Out of scope: this lane makes no UI claims beyond LOD. |
| A14 | Dirty persona fix vs "outstanding P0" | `git diff provider_executor.rs`; `09:247` | **Blocker if lost** | C0 commits it with `tool_turn_persona` acceptance; then C0 is the baseline for every later commit. |
| A15 | First-wave path `CHANGE-LEDGER-IMPL.md` is root, not `docs/contracts` | `CHANGE-LEDGER-IMPL.md:1-8`; `06:974-978` says "a CHANGE-LEDGER row" | Low | Reference the root file explicitly in every contract commit. |
| A16 | Current tests pin the wrong 402 behavior | `error_matrix.rs:161-180` asserts `E_PROVIDER_AUTH` for 402 | **High** | P0 mapping commit must update the test **and** add the new assertion; a green suite after the fix is the proof the old behavior is gone. |

### 2.3 What the first wave got right (do not redo)

- The serial bottleneck, caps, DB single-connection, event ring arithmetic, and React-Flow no-culling facts are correct (`03:88-106`; code refs verified in §1; corroborated by `07`).
- The spawn/approval, Host, 402 and context-window gaps are real and correctly located (`06:372-401,528-566,809-827,748-770`; `09:247`; code refs in §1/§5).
- The parent-completion and deadline/stall invariants are the right invariants (`02:475-478,717`; `09:247`).
- The target bounds (admission queue ≤ 2×permits/age ≤ 9 min, DB writer batch 20–50 ms/200 rows, read pool 2–4, coalesce 16–33 ms, session queue 256, LOD ≤ 120 mounted) are the acceptance numbers this design uses (`03:542-550,648-652`).

---

## 3. Target architecture, invariants, and contract policy

### 3.1 Component graph (all in-process; no new service)

```text
RunEngine (single run today)
  submit_prompt -> planning call (admission ticket, deadline)
      -> plan.validate -> task rows
      -> Scheduler: bounded call-admission queue + fair dispatcher
           -> task futures (bounded) -> provider call -> ModelLease (AIMD)
                -> CostGovernor reserve/settle -> DB writer actor
                -> DeltaCoalescer -> EventBus control lane -> per-session outbound queues -> WS/UI
           -> DB reads via read pool (snapshots/history/API)
      -> parent-completion guard / queue drain -> run terminal (single owner)
```

### 3.2 New invariants (the design's contract)

| ID | Invariant | Enforced at |
|---|---|---|
| S-I1 | A call is never sent after its budget/context/deadline preflight refuses; refusal happens before `open`/`open_stream`. | `provider_executor.rs:94-151`; `client.rs:372-425,562-607`; new preflight |
| S-I2 | Every queued call has a deadline; an expired ticket fails by name before the provider call. | new `AdmissionQueue`; `execution.rs:1245-1307` |
| S-I3 | A task is `Completed` only when every descendant task is terminal. | new guard in `complete_task`; `02:707` |
| S-I4 | Exactly one component owns the run terminal frame; deadline/stall/cancel converge on it. | `execution.rs:1245-1307`; new watchdog |
| S-I5 | One writer thread commits all SQLite writes; reads use separate `query_only` connections. | new DB handle; `db.rs:295-350` |
| S-I6 | Delta events are coalesced before the bus; control events are never dropped by coalescing. | new coalescer; `execution.rs:1764-1795`; `bus.rs:92-155` |
| S-I7 | Every session has a bounded outbound queue; a slow client cannot block another session or the bus. | new session writer; `ws.rs:401-455` |
| S-I8 | Every provider call is admitted under a per-model AIMD lease; 429s reduce the lease, successes raise it. | new provider lease; `budget.rs:371-431` |
| S-I9 | Cost is reserved before send, settled from usage, persisted as Decimal, and never double-counted on retry. | `budget.rs:235-305`; new governor/ledger |
| S-I10 | A failed/partial/missing usage is never settled as zero; the run becomes INCOMPLETE. | existing honest behavior (`provider_executor.rs:186-188,232-243`); preserve |
| S-I11 | A refused call never creates a child row, an approval row, or a spend settlement. | spawn approval fix; approval bridge ordering |
| S-I12 | A slow session degrades its own delta stream only; control events remain ordered and resync is rate-limited. | new session writer |

### 3.3 Contract policy (frozen contract with a ledger)

The frozen contract is `hiverouter-contract` + `docs/contracts/UI-COMMANDS-v1.md`; a refusal code is stable and adding one requires a row in the root `CHANGE-LEDGER-IMPL.md` and a rerun of `hiverouter-contract/tests/ui_command_matrix.rs` (`refusal.rs:5-7`; `CHANGE-LEDGER-IMPL.md:1-8`). This design **requires** the following additive codes (one contract commit, C1):

| New code | Retryable | Use | Existing alternative |
|---|---|---|---|
| `E_PROVIDER_BALANCE` | no | HTTP 402, insufficient balance; freeze account spend | `E_PROVIDER_AUTH` (wrong) |
| `E_PROVIDER_REQUEST_REJECTED` | no | HTTP 400/422 documented request errors | `E_PROVIDER_INVALID_RESPONSE` (wrong) |
| `E_CONTEXT_WINDOW_EXCEEDED` | no | prompt+output exceeds model context before send | `E_TOKEN_BUDGET_EXCEEDED` (wrong scope) |
| `E_QUEUE_FULL` | yes | admission queue at capacity | `E_SEMAPHORE_FULL` (different meaning) |
| `E_DEADLINE` | no | run/ticket wall-clock deadline | `E_TOOL_TIMEOUT`/`E_PROVIDER_TIMEOUT` (wrong layer) |
| `E_STALLED` | no | no progress over N calls/window | no honest existing code |
| `E_HIVE_BUDGET_EXCEEDED` | no | optional per-hive cap exhausted (06 P0-5) | `E_BUDGET_EXCEEDED` collapses run/hive, obscures attribution |

**Contract-change checklist per code:** `refusal.rs` variant + message/how_to_fix; `Refusal::ALL`/serialization tests; `CHANGE-LEDGER-IMPL.md` row; `provider/tests/refusal_matrix.rs` or `error_matrix.rs`; `contract/tests/ui_command_matrix.rs` and `golden_schemas.rs`; `web/src/api/contract.ts` `KNOWN_REFUSAL_CODES` (`contract.ts:466-486`) + `web/src/api/refusals.ts` map + `web/src/state/strings.ts` code string; `docs/contracts/UI-COMMANDS-v1.md` §5. `E_HOST_REJECTED` is **not** added: reuse `E_ORIGIN_REJECTED` for Host rejection (06 P0-1 acceptance expects it), with a corrected generic message (messages may change; codes may not) and a distinct `detail`/log field.

---

## 4. Exact implementation plans

Each plan lists: goal → data model → algorithm → seams → config → ordered steps → deterministic acceptance → rollback. All config defaults must be a **decision input**, not an invented operator promise; the recommended value is marked.

### 4.1 Admission queue and fair dispatcher (S1)

**Goal.** Bound every provider call before it is sent; give each queued call a deadline; make a cancelled/expired call release its place; stop one hive/run from starving the rest. The queue unit is a **call ticket**, not a task, because a parent task may wait for an inline child while holding no provider lease — queuing tasks would deadlock parent/child at the permit limit. Tasks become concurrent only up to a bounded number of futures; the calls they make are admitted.

**Data model (new `backend/crates/hiverouter-core/src/scheduler.rs`)**

```rust
pub struct AdmissionQueue { capacity: usize, state: Mutex<QueueState>, permit_notify: Notify }
pub struct CallTicket { id: u64, run_id: RunId, task_id: TaskId, hive_id: HiveId,
                        enqueued_at: Instant, deadline: Instant }
pub enum AdmitError { Full /* E_QUEUE_FULL */, Expired /* E_DEADLINE */, Cancelled }
```

- `capacity = 2 × configured_permits` (03:542); default S1: 25 Flash / 10 Pro → 50/20.
- `deadline = min(now + task_deadline (default 9 min), run_deadline_remaining)`; expired tickets are removed by the dispatcher with `E_DEADLINE` **before** any provider call.
- Fairness: one FIFO per `hive_id`; dispatcher does deficit round-robin with quantum 1 and re-distributes unused permits every 100 ms (03:153). Single active run today: `hive_id = nearest SuperAgent ancestor task id, else run root`; multi-run is future (Q11).
- Ticket lifecycle: enqueue (bounded) → dequeue → provider call → release; cancellation removes a ticket and wakes the queue; shutdown drains with `ToolCancelled`.

**Dispatch algorithm (engine change).**
1. `dispatch_inner` plans, inserts tasks, then calls `run_bounded_dispatch(run, token)`.
2. Build readiness from parent/deps: leaves ready immediately; an aggregator becomes ready only when all children/deps are terminal; model-spawned children are scheduled as their own tickets (their parent tool call awaits the result; no permit held).
3. A bounded `JoinSet` (hard `MAX_TASK_FUTURES = MAX_AGENTS`, 64 today) runs ready tasks. Each task's every provider call (`execute`, `continue_with_tools`; planning call too) does `admission.acquire(ticket).await` before the executor and drops the ticket after the call. This is the only place that touches the queue.
4. Before each task completion, the parent-completion guard (§4.5) applies. Aggregation is a barrier: no aggregator starts until its children are terminal. After all tasks and the spawn queue are terminal, dispatch returns `Ok(())`; the single terminal owner (`dispatch`) finalizes exactly once.
5. A killed/expired task settles through the existing per-task token; the ticket is dropped by RAII; a late scheduler item is skipped by re-reading the task row (`execution.rs:1505-1520` pattern preserved).

**Seams.** New `scheduler.rs`; `lib.rs` module/export; `RunEngine` fields/constructors (`execution.rs:400-439,456-650`; `with_services:598`); `dispatch_inner` leaf/aggregation loops (`:1395-1490`) replaced by readiness/JoinSet; `execute_task_inner` call site (`:2793-2799`) and `execute_tool_turn` continuation call site (`:2928-2935`) wrap `admission.acquire`; planning call (`:1310-1319`) acquires a ticket; `spawn_sub_hive`/`execute_spawn_agent` enqueue tickets (`:1061-1111,2618-2729`); `execute_admission` naming.

**Config.** `HIVEROUTER_ADMISSION_CAP` (default 2×permits), `HIVEROUTER_CALL_DEADLINE_SECS` (default 540), `HIVEROUTER_MAX_TASK_FUTURES` (default 64), `HIVEROUTER_FAIR_QUANTUM_MS` (default 100). All documented in `--help` and `/diagnose` (counts only, no secrets).

**Acceptance (new `core/tests/scheduler.rs`, offline, wiremock repeated-usage server):**
- `queue_at_capacity_refuses_by_name`: fill 2×permits with delayed wiremock; next ticket returns `E_QUEUE_FULL`; no HTTP request from it; `queue_depth` metric equals capacity.
- `expired_ticket_never_calls_provider`: ticket whose deadline is in the past → `E_DEADLINE`; wiremock request count unchanged.
- `fair_round_robin_two_hives`: two hives enqueue 10 tickets each with 4 permits; per-hive dispatch share deviates from fair by ≤1 over a window; no starvation.
- `inline_child_does_not_deadlock_at_permit_limit`: one parent spawns an inline child at the permit limit; both complete; no `E_SEMAPHORE_FULL`.
- `cancel_releases_ticket_and_futures_join`: cancel with 25 pending; all tickets removed, no late provider call, terminal exactly once.
- Run with `cargo test -p hiverouter-core --test scheduler` + `--test run_lifecycle`.

**Rollback.** `HIVEROUTER_SCHEDULER=off` restores the existing sequential loops (kept behind a branch/feature during S1); commit revert is the fallback. S1 acceptance requires the flag on.

### 4.2 Per-model AIMD lease + budget-lock split (S4, but S1 start values)

**Goal.** Replace the fixed `IN_FLIGHT_LIMIT = 8` with a per-model lease that starts at the staged capacity and adapts to 429s without oscillation; make the lease actually usable by releasing the per-run budget lock across awaits.

**Data model (new `backend/crates/hiverouter-provider/src/lease.rs`, exported in `lib.rs`)**

```rust
pub struct ModelLease { state: Mutex<LeaseState>, waiters: Mutex<VecDeque<Waiter>>, notify: Notify }
struct LeaseState { model: Model, permits: usize, min: usize, max: usize,
                    in_flight: usize, success_streak: u32, cooldown_until: Option<Instant>,
                    last_decrease: Option<Instant> }
pub struct LeasePermit { model: Model, lease: Arc<ModelLease> } // Drop releases + wakes one waiter
```

- Defaults at S1: Flash start 25, Pro start 10; S4 AIMD max: Flash **150**, Pro **30**, min 4 (03:620 Q15); absolute account ceilings 2,500/500 are **not** local permits. `HIVEROUTER_FLASH_PERMITS`, `HIVEROUTER_PRO_PERMITS`, `HIVEROUTER_FLASH_MAX`, `HIVEROUTER_PRO_MAX`; values validate at boot.
- Additive increase: +1 after 20 consecutive successes without 429, capped at max; multiplicative decrease: on a 429, `permits = max(min, permits/2)`, set shared cooldown `now + max(2 s, Retry-After if present, 250 ms × 2^consecutive_429)` capped 30 s; reset success streak.
- Acquire is FIFO and cancellation-aware; expired ticket/deadline wakes with `E_DEADLINE`. Retries release the permit before backoff and re-acquire per attempt; a stream keeps its permit until the pump ends (current pump owns the permit at `client.rs:569-592`).
- 429 observation happens once per attempt in `client.rs:390-423` (complete) and `:594-604` (stream); success observation after `record_call` success (`:391-401,:580-592`). The client never silently retries after a 429 increase without updating the lease.

**Budget-lock split (required in the same commit).** Today: `provider_executor.rs:104-110` locks the map, gets `&mut RunBudget`, awaits `client.complete_with_budget` while held; `stream_reply` holds it across the whole stream (`:140-147,162-243`). Replace with:

```rust
budgets: Mutex<HashMap<RunId, Arc<tokio::sync::Mutex<RunBudget>>>>,
// 1. lock map briefly, clone Arc
// 2. lock run budget: reserve(estimate) -> Reservation; drop lock
// 3. await provider call with Reservation (no budget lock held)
// 4. lock run budget: settle(usage, reservation) / release on failure
```

`RunBudget` gains `reserve(&mut self, estimate) -> Result<Reservation>` and `settle(&mut self, reservation, usage) -> Result<CallCost>`; reservation is a `usize`/Decimal slot on a new `RunBudgetInner`; `preflight` becomes the reservation check. This is the only honest way to let concurrent streams share one run budget without serializing them. Keep `record_call` for retry/usage-only call sites or adapt tests.

**Seams.** `client.rs:40-66` (`ClientConfig.in_flight` → `Arc<ModelLease>`), `:380`/`:569` acquire, `:372-425`/`:562-607` retry/observe; `budget.rs:371-431` (`InFlightLimiter` kept for tests/legacy behind `#[cfg(test)]` or deleted with all call sites migrated); `provider_executor.rs:94-151` and `:162-243` budget lock; `budget.rs:235-305` reserve/settle; `client.rs:507-529` classify 429/402; `config.rs` env parse; `lib.rs` export.

**Acceptance (new `provider/tests/aimd.rs`, deterministic clock/wiremock):**
- `429_halves_permits_within_one_observation`: 8→4 on first 429; 4→2 on next; floor respected.
- `success_streak_raises_one_per_threshold`: 20 clean calls → 5→6 (or configured threshold); no raise during cooldown.
- `shared_cooldown_prevents_retry_storm`: two concurrent 429s produce one cooldown window; total attempts ≤ configured retries, not 2×.
- `permits_are_per_model`: Flash 429 does not lower Pro lease and vice versa.
- `budget_lock_is_not_held_across_stream` in `core/tests/provider_concurrency.rs`: two 300 ms delayed streams on one adapter overlap (<500 ms total, not ≥600 ms). This is the T26 test and **must pass before S4**.
- `no_call_after_cap`: with cap reached, wiremock count unchanged.

**Rollback.** `HIVEROUTER_MODEL_LEASE=off` falls back to `InFlightLimiter` at 8 (legacy); AIMD state is in-memory and resets on restart; commit revert.

### 4.3 DB writer actor and read pool (S2)

**Goal.** One writer, batched commits, off the async workers; read-only connections for snapshots/history so WAL readers don't serialize behind the mutex.

**Data model (`db.rs` refactor).**

```rust
pub struct Database { path: PathBuf, writer: WriterHandle, readers: ReadPool }
enum WriteCmd { InsertTask(TaskRecord, oneshot::Sender<Result<()>>), UpdateState(...),
                RecordResult(...), AppendMessage(...), IncrementCounters(...), RecordUsage(...),
                Custom(Box<dyn FnOnce(&mut Connection) -> Result<(), DbError> + Send>, ...) }
struct WriterHandle { tx: tokio::sync::mpsc::Sender<WriteCmd> }
struct ReadPool { connections: Vec<Mutex<Connection>>, next: AtomicUsize }
```

- Writer thread (`std::thread`) owns one `rusqlite::Connection`; receives commands on a bounded channel (`capacity = 4_096`); drains a batch, `BEGIN IMMEDIATE`, executes all, `COMMIT`, replies to every sender; flush policy 20–50 ms or 200 rows (03:242); busy retry counted, never surfaced as a panic; on shutdown drain + checkpoint; `WAL`/`NORMAL`/`foreign_keys` preserved (`db.rs:338-345`).
- Read pool: 2–4 `query_only` connections created after migration; round-robin; busy timeout **100–200 ms** (not 5 s), no writes; long read transactions avoided. If bundled SQLite `< 3.51.3`, read pool = **1** (U6 gate, `03:686-692`; verify with `SELECT sqlite_version()`).
- Migrations run synchronously in `Database::open`/`migrate` on a single connection **before** the writer thread starts and before any read connection exists; `schema_version()` reads through the pool only after migrations. All migrations v1–v10 unchanged; new v11 (`cost_ledger`) and optional v12 (`hive_budget`) are additive and tested under a fresh DB and a v10 copy.
- `with_connection` remains for tests/boot only; production store methods are split into `read`/`write` helpers. The `RunStore` public signatures stay identical so engine/tests compile unchanged; internals route to the actor. Do **not** move snapshots (filesystem) into the DB actor.

**Seams.** `db.rs:295-350` (Database/open/configure/lock), `db.rs:32-249` (migrations), `store.rs:76-871` (all write/read methods; hot list: `insert_task:241`, `update_task_state:297`, `append_message:399`, `record_task_result:478`, `record_agent_message:526`, `record_run_usage:678`, `record_agent_memory:612`, `record_receipt:838`); `execution.rs:1391-1527` snapshot/mid-run writes; `http.rs:842-875` usage route; `snapshot.rs` unaffected; tests in `core/tests/chat_store.rs` etc. must pass unchanged.

**Acceptance (new `core/tests/db_actor.rs`, tempfile, no wiremock):**
- `writer_batches_commits`: issue 1,000 inserts; assert commit count ≤ 10 and p99 reply latency < 10 ms at 200 writes/s (measure via test-only atomic counters, not wall-clock flake; assert batch size ≥ 20).
- `read_pool_reads_committed_write`: write task then read via pool; row visible after ack.
- `read_pool_is_query_only`: `INSERT` through a read connection fails with a query_only error, mapped to `DbError`/`E_DB_BUSY` as designed, never panics.
- `schema_migration_v10_to_v11_is_additive`: copy a v10 DB fixture; migrate; v10 rows intact; v11 ledger present; migrate twice idempotent.
- `sqlite_version_supports_read_pool_or_degrades`: `SELECT sqlite_version()` ≥ 3.51.3 → pool 2–4; else pool 1 and a named diagnostic.
- `engine_lifecycle_tests_still_green`: `cargo test -p hiverouter-core --test run_lifecycle --test chat_store --test snapshot_history --test kill_restore`.
- Soak thresholds are harness S2, not unit: write p99 <10 ms at 200 writes/s, snapshot build <50 ms, WAL <64 MB (`03:649`).

**Rollback.** `HIVEROUTER_DB_ACTOR=off` constructs the legacy single-mutex `Database`; migrations are forward-only, so rollback requires a pre-migration DB backup (tested restore) rather than dropping columns; no destructive schema changes allowed.

### 4.4 Delta coalescing before the event bus (S2)

**Goal.** Turn per-fragment `REPLY_DELTA` (6,000/s at 150 streams, 03:257-270) into ≤60 flushes/s per active task **before** the 1,024-slot ring, without changing the wire payload or losing text.

**Data model (new `core/src/coalesce.rs`; owned by `RunEngine`).**

```rust
struct DeltaCoalescer { pending: Mutex<HashMap<(RunId, TaskId), Pending>>, bus: Arc<EventBus>,
                        tick: Duration, max_batch_bytes: usize }
struct Pending { text: String, bytes: usize, first_at: Instant }
```

- `push(run, task, fragment)` appends; no bus event per fragment; flusher interval 16 ms (config 16–33) emits one `ReplyDelta { run_id, text_delta }` per task with concatenated fragments; if a pending batch exceeds 64 KiB, flush immediately; flush on every task terminal/fail/cancel and immediately before `REPLY_DONE`; `flush_all` at run terminal.
- Ordering: control events (`NODE_STATE`, `TOOL_CALL`, `REPLY_DONE`, `RUN_STATUS`) stay direct, so sequence numbers remain meaningful; delta flushes get sequence numbers in flush order; text order per task preserved because one coalescer owns each task's buffer.
- The sink signature changes from `reply_delta_sink(run_id)` to `reply_delta_sink(run_id, task_id)`; both first-turn and continuation sinks route through the coalescer (`execution.rs:2791,2922`); `publish_returned_deltas` also routes through it (`:1797-1820`); `reply_delta_sink` currently publishes every fragment at `:1764-1795`.
- No contract change: `ReplyDelta` payload is unchanged; a larger `text_delta` is valid concatenation.

**Acceptance (new `core/tests/delta_coalesce.rs`, paused Tokio clock + in-memory bus):**
- `one_thousand_fragments_emit_at_most_one_per_tick`: no flush before tick; exactly 1 event after 16 ms.
- `no_text_lost`: concatenation of all event payloads equals concatenation of inputs, including multibyte UTF-8.
- `control_events_are_not_coalesced`: `REPLY_DONE` keeps its own seq after the final delta flush; no delta after done.
- `terminal_flush_is_exactly_once`: complete/fail/cancel each flush pending text once; duplicate flush is empty.
- `bounded_batch`: 100 KiB input flushes ≤ 64 KiB chunks, never drops text.
- `coalesce_ratio_under_load_harness` (S2): 2,000 synthetic deltas/s for 10 min → ratio ≥50:1; `lagged=0` (`03:649`).

**Rollback.** `HIVEROUTER_DELTA_COALESCE=off` restores direct publish at the same seam; no data migration.

### 4.5 Parent-completion guard and deferral (P0, C5)

**Root cause.** `complete_task` (`execution.rs:2973-3030`) validates the task is `Running` then writes `Completed`, `REPLY_DONE`, task result, memory and node state. It does not check descendants. Command/model spawns can add a child while a parent turn is in flight; the post-plan queue is drained **after** aggregation (`:1486-1490`), and `spawn_sub_hive` only checks run terminal (`:1069-1073`). So a parent can be `Completed` with a live child, and aggregation can run before a queued child's result exists.

**Exact fix (no new state, no new refusal needed).**
1. Add `live_descendants(task_id)` in the engine: from `store.tasks_for_run`, build the parent map and test each non-terminal task's ancestor chain for `task_id`; call under the existing `task_state_lock` in `complete_task` (`:2979-2982`).
2. `complete_task` returns `CompletionOutcome` (`Completed(Option<String>)` | `Deferred`). If live descendants exist: store `reply` in `pending_completions: Mutex<HashMap<TaskId, StreamedReply>>`, do **not** write any completion state/event, return `Deferred`. Parent stays `Running` (legal: `run_state.rs` allows Running→Completed only; no new state).
3. `execute_task_inner` after the completion call:
   ```rust
   match self.complete_task(run, task, &final_reply)? {
       CompletionOutcome::Completed(text) => Ok(text),
       CompletionOutcome::Deferred => {
           let reply = self.pending_completion(task.id)?.ok_or(Refusal::InternalError)?;
           self.drain_children_of(task.id, run, token).await?;
           match self.complete_task(run, task, &reply)? {
               CompletionOutcome::Completed(text) => Ok(text),
               CompletionOutcome::Deferred => Err(Refusal::InternalError), // invariant still broken
           }
       }
   }
   ```
   `drain_children_of` loops the queued children whose ancestor chain reaches `task_id`, calls `execute_task` on each (which itself handles nested deferral), and returns only when no non-terminal descendants remain. It is bounded by the existing caps and token cancellation.
4. `spawn_sub_hive` rejects a **terminal** parent with `Refusal::TaskTerminal` before inserting a row (`:1069-1073`), and `execute_spawn_agent` relies on it; this closes post-completion spawn.
5. The run-level reconcile (`:1264-1268`) stays as the last-resort cleanup; it must not be the primary guard.

**Why not simply reorder the queue before aggregation?** Reordering helps planned children but not a spawn that lands while an aggregator's provider call is in flight; the completion-time guard is the only race-free check. The deferral keeps the parent's reply (no second model call) and preserves exactly-once `REPLY_DONE`.

**Acceptance (new `core/tests/parent_completion.rs`; stub executor with controllable completion):**
- `parent_not_completed_while_child_queued`: enqueue a command child under a parent, cause parent completion → task state remains `Running`, no `REPLY_DONE`; then child completes → parent completes once.
- `deferred_reply_is_not_lost`: final text persisted exactly once after flush.
- `aggregator_waits_for_queued_child`: an aggregator with a queued child does not consume a missing child result (`aggregation_order` test with a queued child; child result present before aggregator).
- `spawn_under_terminal_parent_is_refused`: terminal parent → `E_TASK_TERMINAL`, no child row.
- `run_terminal_reconcile_never_completes_a_parent_with_live_child` (existing recovery path included).
- `cargo test -p hiverouter-core --test parent_completion --test run_lifecycle`.

**Rollback.** Revert commit; the fix is inert when no child is live (existing tests unchanged). No schema/contract dependency.

### 4.6 Run deadline and deterministic stall detector (P0, C7)

**Root cause.** `dispatch` awaits `dispatch_inner` with only the run cancellation token (`execution.rs:1245-1307`); the provider has 600 s request/idle timeouts (`client.rs:59-66`) but a run can repeat calls/tools indefinitely within caps; there is no progress counter (`02:475-478,717`; `06:947-951`).

**Exact fix.**
- Config `RunGuard { deadline: Duration (default 3,600 s; 0 disables), stall_calls: u32 (default 8), stall_window: Duration (default 180 s) }` in `Config`/`RunEngine`; env `HIVEROUTER_RUN_DEADLINE_SECS`, `HIVEROUTER_STALL_CALLS`, `HIVEROUTER_STALL_WINDOW_SECS`; all surfaced in `/diagnose` as values/counters (no secrets).
- `RunGuard` field in `RunEngine` (constructed like `Caps::default`, `execution.rs:456-490`): `started_at`, `last_progress: AtomicU64` (monotonic millis), `calls_since_result: AtomicU32`, `last_task: Mutex<Option<TaskId>>`, `cause: Mutex<Option<Refusal>>`.
- Progress points: planning start; task start; provider call start; each delta flush; task result/state; agent message; approval resolution. `last_task` is set whenever a task starts/fails/stalls so the refusal names the branch (`02:838`).
- Watchdog (`tokio::spawn` per run): every 1 s:
  - `elapsed >= deadline` → set `cause = E_DEADLINE`, `task_token.cancel()`/`run_token.cancel()`, publish no second terminal (dispatch owns it), log branch/last task.
  - `now - last_progress >= stall_window` **or** `calls_since_result >= stall_calls` → set `cause = E_STALLED`, cancel run token.
  - `calls_since_result` increments on each provider attempt; resets on a task result/state/progress; a task that only streams counts progress (last_progress) but not result, so a completing stream is not stalled; the detector targets *no result/no state* loops per `02:475-478`.
- `dispatch` distinguishes cause: after `dispatch_inner` returns (likely `ToolCancelled` due to watchdog), read `guard.cause`; final status `Failed` with the cause code; publish **one** `RUN_STATUS`; cancel all task tokens; reconcile non-terminal tasks to Failed under the lifecycle lock; `E_DEADLINE`/`E_STALLED` are terminal. If the run completes normally, cause is `None`.
- Child inheritance: a ticket's deadline is `min(run_remaining, call_deadline)`; no child can outlive the run; inline child provider calls share the same run deadline.

**Contract.** Two new codes `E_DEADLINE`, `E_STALLED` (C1). `/diagnose` counters only.

**Acceptance (new `core/tests/run_deadline.rs`, paused time/stub executor):**
- `deadline_fails_run_with_named_code`: stub executor never returns; deadline 200 ms; run status Failed with `E_DEADLINE`; exactly one `RUN_STATUS`; no extra provider call after deadline.
- `stall_detector_stops_no_progress_loop`: stub returns a tool call/answer cycle without progress; after `stall_calls` calls run Failed `E_STALLED`, `error_code` and last task id in detail/audit (no secret); call count bounded.
- `progress_resets_stall`: a long stream with deltas and a final result is not stalled; run completes.
- `deadline_releases_tasks_and_tokens`: all non-terminal tasks become Failed/Cancelled, no task remains Running/Queued after terminal (`02:707` invariant), permits released ≤2 s.
- `deadline_zero_disables`: config 0 → no watchdog; existing tests unaffected.
- `cargo test -p hiverouter-core --test run_deadline --test kill_restore --test run_lifecycle`.

**Rollback.** Set env 0 (watchdog disabled) or revert commit; completion path unchanged when guard is off.

### 4.7 DB-backed cost governor and ledger (S4, with P0 projected preflight)

**Goal.** Reserve before send, settle actual, persist Decimal cost per call, enforce run + optional hive/global ceilings, and satisfy the operator's "soft guard/emergency stop" policy without silent hard defaults.

**Data model.**
- Extend `BudgetLimits` to `CostLimits { run_cap: Option<Decimal>, hive_cap: Option<Decimal>, global_cap: Option<Decimal>, soft_alert_usd: Option<Decimal>, hard_enabled: bool }`; defaults preserve current hard per-run caps during the security wave (do not silently remove; A10/Q17). Add `HIVEROUTER_BUDGET_MODE=observe|soft|hard` (default `soft` for new installs; `hard` required to enforce new global cap; existing caps still enforced by provider budget until the migration gate).
- New migration v11 `cost_ledger(id, run_id, hive_id, task_id, model, peak_class, prompt_hit_tokens, prompt_miss_tokens, output_tokens, cost_usd TEXT, price_source, created_at)` and v12 `hive_budget(hive_id, cap_usd, spent_usd, reserved_usd, period_start, period_end)`; Decimal stored as string; `hive_id` = `projects.id` (migration v10 already exists, `db.rs:233-247`).
- `CostGovernor` (provider crate or core service) owns `reserve(scope, estimate) -> Reservation` and `settle(reservation, usage) -> CallCost`; `RunBudget` already has `RunAccounting` and Decimal; add reserve/settle (also needed for §4.2 budget lock).
- Estimate: replace `estimate_prompt_tokens` with conservative `max(ceil(request_json_bytes/3), 64)` (documented as safety-overestimate; CJK `bytes/3` is ≥ characters while English is ~33% over; false-refusal trade-off is a setting) + `max_output_tokens` from request. Projected cost = `estimate × price_at(now)` (peak/off-peak from `pricing.rs`), refuse if `spent + reserved + projected > cap` **before** `open`; on missing usage, keep the reservation and mark INCOMPLETE (existing honesty).
- The P0 projected-cost fix (T22) is folded here: current `preflight` checks `spent` only (`budget.rs:252-261`); add the projected check in the same reservation API so `budget_preflight_refuses_before_any_send` plus new `projected_cost_refuses_before_send` pass.

**Seams.** `budget.rs:24-52,235-305`; `accounting.rs:1-16,260-292`; `provider_executor.rs:94-151` (reservation); `db.rs:32-249` migrations; `store.rs:678-735` usage; `execution.rs:698-719` persist usage (must persist cost too); `http.rs:842-875` `/api/usage/runs/{id}` and a new `/api/usage/hives/{id}`; UI meters (05 P0-9). Keep the existing `run_usage` honest and add a separate ledger, never rewrite historical rows.

**Acceptance (new `provider/tests/cost_governor.rs` + `core/tests/cost_ledger.rs`, wiremock, no paid call):**
- `projected_cost_refuses_before_send`: spent $0.49 of $0.50; projected Pro-peak call > remaining → `E_BUDGET_EXCEEDED`, attempts 0, wiremock requests 0.
- `reserve_blocks_concurrent_overspend`: two concurrent calls each project over remaining; exactly one succeeds; the other refuses; spent+reserved never exceeds cap.
- `settle_matches_fixture_exactly`: known 400 hit/600 miss/500 output → exact Decimal string pinned; peak class + price source stored.
- `hive_cap_refuses_second_run_before_http`: run 1 settles $0.30/hive cap $0.50; run 2 first preflight → `E_HIVE_BUDGET_EXCEEDED`, wiremock count unchanged.
- `incomplete_usage_keeps_reservation`: null usage → `E_USAGE_INCOMPLETE`, cost report `null`, reservation not silently zeroed.
- `observe_mode_never_refuses`: mode observe emits metrics only; no call blocked.
- `cargo test -p hiverouter-provider --lib budget accounting pricing` + new tests + `cargo test -p hiverouter-core --test cost_ledger`.

**Rollback.** `HIVEROUTER_BUDGET_MODE=observe` disables new hard refusals (existing caps still enforce); ledger migration is additive; a pre-migration DB backup is required for rollback (forward-only rule).

### 4.8 Bounded sessions (S3)

**Root cause.** `session_loop` (`ws.rs:401-455`) awaits `send_envelope` inline for every event; a slow socket blocks that session's receive loop and its ability to process control; the bus broadcast lag then forces `RESYNC_REQUIRED` (`:451-462`).

**Exact fix.**
- New `SessionOutbound` in `ws.rs`: one `mpsc::channel<SessionFrame>` (`capacity 256`; control frames prioritized) + a writer task owning the `WebSocket`; session loop only enqueues (never awaits socket send except hello). `SessionFrame::{Control(SequencedEvent), Delta(SequencedEvent), Ping/Pong, Close}`.
- On control queue full → terminate session with a named refusal (`E_WS_DISCONNECTED` today; optionally `E_SESSION_SLOW` later — not added to C1 unless the operator accepts another contract code; Q5). On delta queue full → drop oldest delta (never control), increment `dropped_delta` metric; client may resync.
- Send timeout 2 s; heartbeat every 20 s; resync budget 1 per 5 s; a lag is handled by enqueuing one `RESYNC_REQUIRED` control frame then closing (current semantic preserved). Slow-session isolation is proven by two sockets, one deliberately stalled.
- The queue count/batching is per session; aggregation across sessions is the coalescer's job (§4.4). Do **not** let sessions subscribe to raw bus deltas before coarse batching; the bus remains UI fan-out.
- Viewport subscription (03:650/05 P1-2) is **not** in the frozen command set; defer to a contract change (Q21). Client-side culling handles S5; server-side subscription is a later optimization.

**Seams.** `ws.rs:401-455` (session loop), `:1013-1024` (`send_envelope`), `:517-545` (hello/replay/approval paths), `bus.rs:84-172` (subscribe); `web/src/api/client.ts` transport (heartbeat/no change to payload); `e2e/specs/live-resync.spec.ts`, `live-security.spec.ts`.

**Acceptance (new `core/tests/session_outbound.rs`, raw WS TCP helper):**
- `slow_client_does_not_block_other_sessions`: session A stalls 500 ms/2 s for 10 s; session B receives all control events with 0 lag/resync; A resyncs ≤10 and disconnects ≤45 s.
- `outbound_queue_is_bounded`: enqueue >256 deltas while socket paused; queue depth stays ≤256, oldest deltas dropped, no control dropped; memory flat.
- `control_lane_lossless`: node/approval/terminal events never dropped while deltas are; order by seq.
- `send_timeout_closes_only_session`: stalled A times out at 2 s; B unaffected.
- `heartbeat_detects_dead_peer`: no pong within 20 s → close session (no busy loop).
- `cargo test -p hiverouter-core --test session_outbound --test ws_transport`.

**Rollback.** `HIVEROUTER_SESSION_QUEUE=off` restores inline send; no data changes.

### 4.9 UI LOD and culling (S5)

**Goal.** Keep ≥50 fps at 600 nodes, ≤120 mounted nodes, ≤50 animated edges (`03:446-455`); show a map, not a texture of cards (05:307-330).

**Exact fix.**
- `web/src/flow/lod.ts` (new): `zoomBand(zoom) -> B0..B4` with hysteresis ±0.04 and boundaries B0 0.10–0.34, B1 0.35–0.59, B2 0.60–1.19, B3 1.20–2.19, B4 2.20–3.00 (05:311-319); `clusterTasks(tasks, band)` collapses each SuperAgent subtree to a cluster node at B0/B1; deterministic ordering by `(depth, id)`; pure functions unit-tested.
- `graph.ts:143-165` `buildNodes` gains a band/viewport argument; `layoutTasks` memoized by `topologyKey = run + sorted(id:kind:parentId)` and never rerun on selection/drag (03:440-444); positions stored in store, not recomputed per event.
- `Canvas.tsx:351-405` adds `onlyRenderVisibleElements` (React Flow 12.12.1 supports it; first wave verified the type), passes `onMoveEnd` viewport into band calculation (already has `setViewport`, `:399-400`), renders cluster nodes for B0/B1 via a separate `nodeTypes` band, caps animated edges to visible/≤50, and avoids re-layout on zoom. Minimap honest: show tier/state rectangles, hide below 1280 with Overview (05:327).
- Subscription: deferred contract change (Q21); client-side culling is sufficient for S5; full-viewport-only server subscription is P1 after contract.
- Performance harness: deterministic unit tests for pure functions; Playwright e2e against a seeded 600-task snapshot (no provider) for mounted-count/fps; `e2e/specs/gui-gates.spec.ts` + new `lod.spec.ts`.

**Acceptance.**
- Unit (`web/src/flow/lod.test.ts`): band boundaries + hysteresis; cluster count equals number of SuperAgent roots; deterministic order; no node loss (cluster contains every descendant id).
- Playwright/Chrome: seed a 600-task snapshot via mock/history fixture; assert `[data-id]` ≤120; `document.querySelectorAll('.react-flow__edge')`/animated count ≤50; pan/zoom 10 s ≥50 fps on reference machine; initial paint <1 s; snapshot ≤300 KB / ≤4 revisions/s (03:454).
- No layout recompute on selection (spy on layout call count).

**Rollback.** `HIVEROUTER_UI_LOD=off` (localStorage/settings) returns to full rendering; no server data change.

---

## 5. Exact P0 fixes

Each fix: current root cause → change → seams → contract → deterministic acceptance → rollback/order. All tests are offline (`cargo test` or the repo's Playwright with installed browser; no paid call).

### P0-1 — `spawn_agent` approval bypass (C6)

**Root cause.** `spawn_agent` is `PermissionClass::Execute` (`tool.rs:332-339`) and `requires_approval` is true for non-Read (`tool.rs:55-57,626-627`), but `execute_tool_turn` handles it **before** the approval branch and comments that no approval is consumed (`execution.rs:1684-1691`); every other non-Read tool reaches `suspend_for_approval` at `:1721-1734`. An injected Worker can create children with the attacker's instruction appended to the run prompt (`:2671-2679`) and no operator visibility (`06:372-401`).

**Exact fix (recommended policy A — explicit approval, one decision per delegation turn).**
1. Add an engine-level branch **before** the loop-owned spawn: `plan.tool == "spawn_agent"` → `suspend_for_approval(run, task, call, &plan, turn, token)` exactly like the `:1721-1734` branch, after the tool-plan validation/authorization and before `execute_spawn_agent`. No child row exists until approval.
2. Approval payload binds role + `instruction` by `args_sha256` (existing approval binding already hashes args; `approval.rs` per 06:T08). The preview must include: role, bounded instruction excerpt, current `fanout/depth/agents`, and the child prompt hash; never the full prompt if it exceeds the args-preview cap (`MAX_ARGS_PREVIEW_CHARS`, `contract/src/event.rs:294`).
3. On approve → `execute_spawn_agent` exactly once; on deny → `ToolExecution::refused(E_TOOL_NOT_ALLOWED)` (or `E_APPROVAL_TIMEOUT` on timeout), parent loop continues, no child row. On timeout/cancel the existing wait terminal path finalizes the `TOOL_RESULT` (D13) and fails the task as today.
4. Count the attempt before waiting (`execution.rs:1727-1730` pattern); enforce `MAX_APPROVAL_WAITS_PER_RUN=8` (`tool_loop.rs:21`). For scale, later add a **delegation-budget grant** (one approval allows up to N children in the same parent turn, audited) — Q1 option B — only after S1/S4 load gates; do not silently bypass.
5. Declare an explicit policy in code/docs: operator `SPAWN_SUB_HIVE` remains authorized (operator-originated, already a command); model `spawn_agent` is approval-gated by default. Add `APPROVAL_EXEMPT` is **empty** for spawn; the static coverage test below enumerates it.
6. Audit: every approval request/resolve already goes through the approval ledger; add a `spawn_proposed` audit event with role/depth/fanout and args hash (no instruction body) so `spawn_agent` is never silent.

**Seams.** `execution.rs:1682-1736`; `tool.rs:332-339,55-57,626-627`; approval bridge/suspend (`execution.rs:1900-2417`; `tools_bridge.rs:169-247`); `docs/contracts/UI-COMMANDS-v1.md` approval row; UI approval card (05 P1-7).

**Acceptance (new `core/tests/approval_coverage.rs` + `core/tests/engine_spawn_approval.rs`):**
- `every_approval_tool_is_gated_or_declared_exempt`: enumerate `ToolId::all()`; for every `requires_approval()==true`, assert it reaches `suspend_for_approval` on a model-originated call or is in `APPROVAL_EXEMPT` (for spawn, the former). Static, no provider.
- `spawn_without_approval_creates_no_child`: call execute path with no resolve; wiremock executor returns one spawn_agent call; assert no child row, no `NODE_SPAWN`, tool result `E_APPROVAL_REQUIRED`/wait.
- `denied_spawn_creates_no_child_and_continues`: deny → one refused result, no child, parent continues to a normal reply.
- `approved_spawn_runs_once`: approve → exactly one child row/result; replay of the resolve cannot spawn twice (existing `E_APPROVAL_REPLAY` test extended).
- `cargo test -p hiverouter-core --test approval_coverage --test engine_spawn_approval --test approval_ledger`.

**Rollback.** Revert commit. A `HIVEROUTER_SPAWN_APPROVAL=off` rollback is **not** recommended (it reintroduces the P0); if operationally necessary it must log a loud warning and be impossible to enable in release builds.

### P0-2 — Host allowlist (C2)

**Root cause.** `build_router` has no Host validation (`http.rs:510-531`); `api_session` grants the token to any loopback peer without Origin (`:1017-1043`); mutating routes have no auth (`06:486-526,528-566`). A rebound origin is same-origin.

**Exact fix.**
1. New `HostAllowlist` (in `http.rs` or `config.rs`) built from the **actual bound address** after `TcpListener::bind` (main binds at `main.rs:136-152`; `serve` receives the listener at `http.rs:1050-1058`): allowed `Host` values are `127.0.0.1:<actual_port>`, `localhost:<actual_port>`, and `[::1]:<port>` when IPv6; `Host` missing/foreign/other port → `403 E_ORIGIN_REJECTED` (reuse code; update its message to "Host/Origin is not on the loopback allowlist" and keep the WS Origin path using the same message; detail field identifies `host_not_allowed`). Compare lowercased host, strip default port only when the bound address implies it; **never** trust `X-Forwarded-Host`/`Forwarded`.
2. Middleware ordering: outermost `from_fn_with_state(host_allowlist, require_allowed_host)` before all routes, including `/api/session`, `/health`, `/diagnose`, `/api/*`, `/ws` (the `/ws` handler already checks Origin/token; Host is defense in depth). Public exception decision: `/health` may stay public (no secrets) but must still pass Host (Q3); the test pins whichever choice.
3. C2b (same test family): token gate on mutating/read endpoints: `Authorization: Bearer <session token>` (or `X-HiveMind-Token`; choose one and document in `UI-COMMANDS-v1`); reject missing/wrong with `401 E_TOKEN_INVALID`; check Origin on these routes as defense-in-depth (`E_ORIGIN_REJECTED`). Required routes: profile import/delete, memory clear/list, history runs/messages, agent messages, usage, projects (read-only may be token-gated per 06 P0-2; keep `/health`/`/api/session` bootstrap Host-only). Update web `fetch` wrappers (`web/src/api/client.ts`, `AgentProfilesPanel.tsx`, `AgentMemoryPanel.tsx`) to send the header; the token is already in memory from `/api/session` (`client.ts:290-306`).
4. `provider_origin_allowed` (`http.rs:926-935`): change `None => true` to require an explicit allowed Origin for provider proxies (T14), or route them behind the token; keep `Cache-Control: no-store`.
5. Logging: one middleware warns with `code=E_ORIGIN_REJECTED`, path, host, origin (no token/body), satisfying L1 (`06:1007`).

**Seams.** `http.rs:510-531` (`build_router`), `:926-935` (`provider_origin_allowed`), `:1017-1043` (`api_session`), `:1046-1061` (`serve`); `config.rs:170-213,382-392`; `main.rs:136-166`; `ws.rs:362-383` (Origin/token check); `web/src/api/client.ts:183-306`; panels' fetch calls.

**Acceptance (new `core/tests/http_origin.rs`, raw TCP helper like `prompt_assets_wiring.rs:217`):**
- `rebound_host_header_is_rejected`: `GET /api/session` with `Host: rebind.evil.example:<port>` → 403 `E_ORIGIN_REJECTED`, no token in body.
- `loopback_host_still_works`: `Host: 127.0.0.1:<port>` and `Host: localhost:<port>` → 200 for `/api/session` from loopback.
- `mutating_endpoints_require_session_token_and_origin`: for each mutating route, no token / foreign Origin / foreign Host → 401/403; DB row count unchanged; with valid token+Host → success. Include memory/history/agent-message reads (T12).
- `provider_proxy_without_origin_is_refused` (T14) and `cache_control_no_store`.
- `health_policy_is_pinned` (public vs Host-gated).
- `cargo test -p hiverouter-core --test http_origin --test http_boot --test ws_transport`.

**Rollback.** `HIVEROUTER_HOST_ALLOWLIST=off` restores prior routing for development only, logs a loud warning; default on. C2b token can be independently reverted with `HIVEROUTER_HTTP_TOKEN=off`; default on.

### P0-3 — 401/402/400/422 provider mapping (C3, with projected cost)

**Root cause.** `classify_status` maps `401 | 402` → `E_PROVIDER_AUTH` and `_` → `E_PROVIDER_INVALID_RESPONSE` with "undocumented" text (`client.rs:500-524`); tests assert it (`error_matrix.rs:161-180`). 402 means insufficient balance; 400/422 are documented request errors (`06:955-972`).

**Exact fix.**
1. Add `Refusal::ProviderBalance` (`E_PROVIDER_BALANCE`, non-retryable) and `Refusal::ProviderRequestRejected` (`E_PROVIDER_REQUEST_REJECTED`, non-retryable) via C1 contract commit.
2. `classify_status`: 401 → `ProviderAuth` (unchanged; detail "check key source/rotate runbook", no body); 402 → `ProviderBalance` (detail "account balance exhausted (HTTP 402); check GET /user/balance; do not rotate the key"); 400 | 422 → `ProviderRequestRejected` (detail includes status and a request id if present, never the body/parameter value); 429 unchanged; 500/502/503/504 transient; other 4xx → `ProviderInvalidResponse` with status; 501/505 non-retryable invalid.
3. On 402, set `AccountState { balance_frozen_until: Some(now+cooldown), last_402_status }` in the provider client (or process-level `Arc`), so subsequent preflights fail fast with `E_PROVIDER_BALANCE` **before** HTTP until the operator clears/balance check passes; surface the state in `/health`/`/diagnose` (boolean/code only). Do not auto-retry a 402.
4. `http.rs:937-945` `provider_error_response`: map `E_PROVIDER_BALANCE` → HTTP 402 (or 401? use 402 for truth), `E_PROVIDER_REQUEST_REJECTED` → 400 (or 422 if status known), keep auth 401; expose code.
5. Update tests **and** any UI mapping: `web/src/api/contract.ts:466-486` known codes; `web/src/api/refusals.ts`; `web/src/state/strings.ts` provider strings; `docs/contracts/UI-COMMANDS-v1.md` §5; ledger row(s); `hiverouter-contract/tests/{ui_command_matrix,golden_schemas}.rs`; `provider/tests/{error_matrix,refusal_matrix}.rs`.

**Acceptance (update `provider/tests/error_matrix.rs`):**
- `http_402_is_balance_not_auth`: wiremock 402 → code `E_PROVIDER_BALANCE`, retryable false, requests ==1, `http_status == 402`; second call while frozen → refused before HTTP (count unchanged).
- `http_400_and_422_are_request_errors`: 400 and 422 → `E_PROVIDER_REQUEST_REJECTED`, no retry, detail contains status only, never body.
- `http_401_remains_auth`: unchanged code/retry behavior.
- `http_429_still_retries`: unchanged ≤3 attempts and final `E_LLM_RATE_LIMIT`.
- `provider_error_response_maps_balance_to_402` (core HTTP test).
- `cargo test -p hiverouter-provider --test error_matrix --test refusal_matrix` + `cargo test -p hiverouter-contract`.

**Rollback.** Revert both contract and mapping commit together; because the codes are additive, old code paths/tests are unaffected by leaving the variants (unused) if a partial revert is needed.

### P0-4 — Context-window preflight and conservative estimator (C4)

**Root cause.** Model metadata already has `context_window`/`max_output_tokens` (`dto.rs:892-907`; parsed in the mock fixture `:942-947`), but nothing checks them; the only pre-send estimate is JSON `bytes/4` (`provider_executor.rs:436-442`). `01:739` (gap H) and `09:247,261` require a pre-send refusal with zero HTTP calls.

**Exact fix.**
1. `ProviderSettings` adds `context_window: u64` (default 1,048,576 documented value; env `HIVEROUTER_CONTEXT_WINDOW` override for tests) and `context_safety_tokens: u64` (default 1,024; 2% margin min). Optional: a `ModelCatalog` cache loaded once at boot via `client.list_models()` (`client.rs:266-274`) in a background task; if the reported `context_window` is present use it, else default+log; never block startup on the network (provider may be disabled). The metadata proxy already exposes `context_window` to UI (`http.rs:881-900`; `ModelsPanel` per 01:739).
2. Replace `estimate_prompt_tokens` with a documented conservative estimator: `ceil(serde_json::to_vec(request)?.len() / 3) + 64` (English overestimates ~33%; CJK bytes/3 ≈ chars, closes the undercount; false refusals are visible and configurable via `context_safety_tokens`/estimator env for tests). Unit-test the CJK fixture.
3. Add `preflight_context(request, estimated_prompt, max_output, context_window)` in `ProviderExecutorAdapter` before the budget lock/HTTP for **plan, complete, and stream/continue**. Refuse `E_CONTEXT_WINDOW_EXCEEDED` when `estimated_prompt + max_output + safety > context_window`. Keep a separate token-budget check (`E_TOKEN_BUDGET_EXCEEDED`) for the run budget; do not conflate the two errors.
4. Plan calls use `planner_max_tokens`; leaves use `leaf_max_tokens`; continuation uses the same leaf cap; inline spawn inherits.
5. Apply the **projected cost** preflight (T22) at the same place: the reservation check uses `estimated_prompt × current price + max_output × output price` and refuses before send when the projection exceeds the remaining cap. This links P0-4 and 06 P0-4.

**Seams.** `provider_executor.rs:94-151` (complete/stream_reply preflight), `:291-318` (plan), `:320-353` (execute), `:362-429` (continue), `:436-442` (estimator); `config.rs:40-70`; `provider/dto.rs:887-907`; `provider/client.rs:266-313`; `http.rs:881-924` (models/balance display); `budget.rs:235-284` (projected cost).

**Acceptance (new `core/tests/context_window.rs` + `provider/tests/error_matrix.rs` extension):**
- `over_context_request_is_refused_before_send`: a request whose estimated prompt + max output > configured context → `E_CONTEXT_WINDOW_EXCEEDED`; wiremock request count 0.
- `exactly_within_context_is_sent`: estimate below boundary → one request; success.
- `cjk_estimate_is_conservative`: 1.2 MiB CJK JSON estimates ≥ the input's character count and triggers the expected refusal (fixture with low context window).
- `context_window_comes_from_env_or_metadata`: configured 8k vs 1M behavior differs; metadata-cache test with wiremock `/models`.
- `projected_cost_refuses_before_send` (shared with C3): covered in `cost_governor.rs`.
- `cargo test -p hiverouter-core --test context_window --test provider_adapter`.

**Rollback.** `HIVEROUTER_CONTEXT_WINDOW=0` disables the check (documented dev escape), default on; estimator remains conservative. No schema changes.

### P0-5 — Parent-completion guard

Fully specified in §4.5 (C5): `complete_task` deferral under `task_state_lock`, `pending_completions`, `drain_children_of`, `spawn_sub_hive` terminal-parent refusal, and the `parent_completion.rs` acceptance tests. No new contract code; no schema change; rollback is the commit revert.

### P0-6 — Run deadline and stall detector

Fully specified in §4.6 (C7): `RunGuard` watchdog, progress counter, `E_DEADLINE`/`E_STALLED`, single terminal frame, branch attribution, paused-time tests; rollback via env `0`/revert.

### P0-7 — Orphaned first-wave P0s (must be owned somewhere; not silently dropped)

1. **`ask_parent` answer/deadline.** Current: write-only message (`execution.rs:2417-2487`; only HTTP/UI reader `store.rs:574-605`, `http.rs:674-702`); no wait, no reply path, no status. `09:247` makes it P0 but the lane brief does not. **Recommendation:** assign to the memory/A2A lane as P0-AAA; this lane adds only the **deadline side**: every pending escalation has a deadline and the parent/operator answer path emits a status; no task blocking (02:489-503 explicitly keeps questions out of the lifecycle state machine). Gate: do not claim interactive A2A before it exists.
2. **Planner JSON blank retry.** Current: `plan()` performs exactly one `complete`; `message_text` returns `E_PROVIDER_INVALID_RESPONSE` when content is empty or finish is not `stop/tool_calls` (`provider_executor.rs:291-318,452-488`); a blank/refused planner response fails the run with no second attempt. `09:247` calls this P0. **Fix:** one bounded retry (max 1, same request, no tool calls) only when the completion success has empty content / finish reason `length`/`content_filter` and no tool calls; never retry malformed JSON (repo avoids retrying schema-invalid plans); test `planner_blank_response_retries_once_then_refuses`. Owned by provider lane; list in C8 as an external dependency.

---

## 6. Ordered commits, acceptance commands, rollbacks

All commits are independently revertible; no paid API; every test uses wiremock/in-memory/tempfile. Run from `backend/` unless stated.

| # | Commit (scope) | Files/seams | Deterministic acceptance (new unless noted) | Rollback |
|---|---|---|---|---|
| **C0** | Tree hygiene: commit the **existing dirty** persona normalization + continuation prompt with a dedicated test; rebase all later work on it. No behaviour beyond the diff. | `provider_executor.rs:82-92,375` (dirty) + `core/tests/tool_turn_persona.rs` (new) | `cargo test -p hiverouter-core --test tool_turn_persona --test provider_adapter`; assert continuation system equals persona prompt for `role=superagent`, fallback `TASK_SYSTEM` for unknown; role normalization (`Content Writer`→`content_writer` mapping when persona exists). | `git revert C0` (only if baseline intentionally reverted; do not lose it again) |
| **C1** | Contract delta: seven additive refusal variants + messages/ledger/UI maps (no behaviour). | `refusal.rs`; `CHANGE-LEDGER-IMPL.md`; `web/src/api/contract.ts:466-486`, `refusals.ts`, `strings.ts`; `docs/contracts/UI-COMMANDS-v1.md`; `contract/tests/ui_command_matrix.rs`, `golden_schemas.rs` | `cargo test -p hiverouter-contract`; `cargo test -p hiverouter-provider --test refusal_matrix` (samples only); `npm --prefix web run typecheck` (offline). Assert every new code has a sample, retryable flag, message, UI map. | `git revert C1` (revert contract commit; no code uses the codes until later commits) |
| **C2** | Host allowlist + HTTP token/Origin gate (+ provider proxy Origin). | `http.rs:510-531,926-935,1017-1061`; `config.rs:170-213`; `main.rs:136-166`; `web/src/api/client.ts:183-306`; panels; new `core/tests/http_origin.rs` | `cargo test -p hiverouter-core --test http_origin --test http_boot --test ws_transport`; raw TCP: rebound Host 403, loopback Host 200, mutating no-token 401/no-mutation, read leaks closed; `npm --prefix web run typecheck`. | `HIVEROUTER_HOST_ALLOWLIST=off` / `HIVEROUTER_HTTP_TOKEN=off` (dev only, warn); or revert |
| **C3** | Provider status mapping + projected-cost preflight; update wrong 402 test. | `client.rs:492-530,937-945`; `provider/tests/error_matrix.rs:141-180`; `budget.rs:235-284`; `http.rs:937-945`; new `cost_governor.rs` projection test | `cargo test -p hiverouter-provider --test error_matrix --test refusal_matrix`; 402 balance/no-retry/freeze, 400/422 request rejected, 401 auth, projected cost zero HTTP. | Revert C3 (contract codes may stay); `HIVEROUTER_PROJECTED_COST=off` dev only |
| **C4** | Context-window preflight + conservative estimator. | `provider_executor.rs:94-151,291-429,436-442`; `config.rs:40-70`; `provider/dto.rs` read; new `context_window.rs` | `cargo test -p hiverouter-core --test context_window --test provider_adapter`; zero HTTP on over-context; CJK conservative. | `HIVEROUTER_CONTEXT_WINDOW=0` / revert |
| **C5** | Spawn approval gate + audit + static coverage. | `execution.rs:1682-1736`; `tool.rs` unchanged; `core/tests/approval_coverage.rs`, `engine_spawn_approval.rs` | `cargo test -p hiverouter-core --test approval_coverage --test engine_spawn_approval --test approval_ledger`. | Revert C5; no env bypass in release |
| **C6** | Parent-completion guard/deferral + terminal-parent spawn refusal. | `execution.rs:2973-3030,1069-1073,2747-2762`; new `parent_completion.rs` | `cargo test -p hiverouter-core --test parent_completion --test run_lifecycle`. | Revert C6 |
| **C7** | Run deadline + stall detector. | `execution.rs:1245-1307`; `config.rs`; new `run_deadline.rs` | `cargo test -p hiverouter-core --test run_deadline --test kill_restore`. | `HIVEROUTER_RUN_DEADLINE_SECS=0`/revert |
| **C8** | Orphan P0s (external owner): ask_parent deadline/status; planner blank retry (max 1). | `execution.rs:2417-2487`; `provider_executor.rs:291-318,452-488`; new tests | `cargo test -p hiverouter-core --test ask_parent_deadline`; `cargo test -p hiverouter-core --test planner_blank_retry`; wiremock counts (1 retry max, no retry on malformed JSON). | Revert C8 (independent) |
| **C9** | S0 load harness (no product change) + baseline artifacts; verify SQLite version (U6) and sequential baseline. | `backend/tests/load/` new harness; no product code | Run harness against current build: captures in_flight≤1, throughput ~0.4 calls/s at 20 s latency, p50/p95/p99; records `sqlite_version()`; fails loudly if target claims cannot be reproduced. | n/a (non-product) |
| **C10** | S1 admission queue + bounded dispatcher (per-hive fair) + call deadlines. | `scheduler.rs`; `execution.rs:1310-1550,2793-2799,2928-2935`; `lib.rs`; config; new `scheduler.rs` tests | `cargo test -p hiverouter-core --test scheduler --test run_lifecycle`; S1 harness 60 pending/25 permits 10 min: in_flight p99 25, wait p99 <250 ms, 0 deadline expiries, named cap refusal, flat memory. | `HIVEROUTER_SCHEDULER=off` → sequential loops |
| **C11** | S2 DB writer actor + read pool + SQLite version gate (migrations unchanged; v11/v12 introduced only with C15). | `db.rs:295-350`; `store.rs` writes; new `db_actor.rs` | `cargo test -p hiverouter-core --test db_actor --test chat_store --test snapshot_history --test kill_restore`; batch ≥20, read-after-write, query_only, version gate. Soak per 03:649. | `HIVEROUTER_DB_ACTOR=off` + pre-migration DB backup |
| **C12** | S2 delta coalescer before bus. | `coalesce.rs`; `execution.rs:1764-1820,2791,2922`; new `delta_coalesce.rs` | `cargo test -p hiverouter-core --test delta_coalesce`; 1,000 fragments ≤1 event/tick, no text loss, control order; S2 2k events/s ratio ≥50:1. | `HIVEROUTER_DELTA_COALESCE=off` |
| **C13** | S3 bounded session outbound queues/heartbeat/resync budget. | `ws.rs:401-455,1013-1024`; new `session_outbound.rs` | `cargo test -p hiverouter-core --test session_outbound --test ws_transport`; slow-client test: other sessions 0 lag, A resyncs ≤10, queue ≤256, control lossless. | `HIVEROUTER_SESSION_QUEUE=off` |
| **C14** | S4 AIMD lease + budget-lock split (reservation/reservation-settle). | `provider/lease.rs`; `client.rs:40-66,370-425,562-607`; `provider_executor.rs:94-151,162-243`; `budget.rs:235-305,371-431`; new `aimd.rs`, `provider_concurrency.rs` | `cargo test -p hiverouter-provider --test aimd --test budget --test accounting`; `cargo test -p hiverouter-core --test provider_concurrency`; two 300 ms streams overlap <500 ms; 429 halves/recovery; no call after cap. | `HIVEROUTER_MODEL_LEASE=off` (8) / `HIVEROUTER_BUDGET_RESERVATION=off`; revert |
| **C15** | S4 two-level cost governor + v11/v12 migrations + peak-aware reserve/settle + usage routes/UI meters. | `provider/budget.rs`; `accounting.rs`; `db.rs` migrations; `store.rs:678-735`; `execution.rs:698-719`; `http.rs:842-875`; new `cost_governor.rs`,`cost_ledger.rs` | `cargo test -p hiverouter-provider --lib budget accounting pricing`; `cargo test -p hiverouter-core --test cost_ledger --test db_actor`; projection/reserve/settle/hive cap/observe mode; migration v10→v11 additive + backup restore; S4 150 soak with 429 injection passes S1–S3 criteria. | `HIVEROUTER_BUDGET_MODE=observe`; restore DB backup for migration rollback |
| **C16** | S5 LOD/culling/perf harness; deferred viewport-subscription contract item. | `web/src/flow/lod.ts`; `graph.ts:143-165`; `Canvas.tsx:351-405`; `nodes.tsx`; new unit + `e2e/specs/lod.spec.ts` | `npm --prefix web run typecheck`; `npm --prefix web test` (lod unit); Playwright seeded 600 nodes: ≤120 mounted, ≤50 animated, ≥50 fps pan/zoom, paint <1 s, snapshot budgets. | `HIVEROUTER_UI_LOD=off` |
| **C17** | S5 acceptance/load run + claim register/docs update; no product change. | `docs/research/hmars/17...` evidence append; claim register | Run S0–S5 harness matrix: every stage threshold green; record artifacts; explicitly mark 300–600 supported or not. | n/a |

**Dependency notes.** C1 before C3/C4/C7; C2 independent; C5/C6 before C10 (the scheduler must not amplify a race); C9 before C10; C10 before C14 (permit demand), C11 before C15 (v11/v12), C12 before C13? Coalescing (C12) makes sessions cheaper but can ship either order; C13 before S3; C16 only after C10/C12 pass. **Never** run C10 before C5/C6.

---

## 7. What must NOT change until migration/load gates pass

| Gate | Frozen thing | Why | Unlock evidence |
|---|---|---|---|
| **G0** | Product contract: frozen events/commands/refusal **meanings**; no unledgered code/event/command changes | UI/backend/QA rely on frozen names; `ui_command_matrix` and golden schemas enforce it | C1 ledger rows + full contract tests; any later change gets its own row + matrix rerun |
| **G1** | `MAX_AGENTS 64`, `MAX_FANOUT 8`, `MAX_SPAWN_DEPTH 3`, `CALL_CAP 40`, `TOKEN_BUDGET_COMBINED 400k`, `RUN_COST_CAP` | Raising any one alone yields a worse failure (more tasks, same permits/DB/event bounds); `03:88-106` and S5 gate | S1–S4 green + plan-size budgeting + hierarchical planning (`03:493-506`) + settings/cost governor |
| **G2** | `IN_FLIGHT_LIMIT=8` / `ModelLease` max >25 before S1/S4 | 150 permits without admission/DB/coalescing moves the failure to SQLite/event ring (`03:565`) | S1, S2, S3, S4 acceptance; AIMD tested under 429 injection |
| **G3** | One active run / single terminal owner / at-most-once effects | Multi-run without per-hive fair queue/DB actor can corrupt accounting and events (`03:153`; `02:I1`) | S1 fairness test with two simulated runs + DB actor; Q11 decision |
| **G4** | DB schema v1–v10; no destructive migration; no new connection >1 before S2 | v10 is live data; WAL read-pool architecture risks the SQLite <3.51.3 WAL-reset bug (`03:686-692`) | `SELECT sqlite_version()` gate + v10→v11 additive migration test + backup/restore test |
| **G5** | `spawn_agent` approval semantics and parent-completion guard | A scheduler that parallelizes before C5/C6 makes races worse; spawn authority cannot be loosened after being tightened silently | C5/C6 tests + one soak with queued-child races; operator decision Q1 |
| **G6** | Event bus capacity/replay and delta semantics; no per-agent subscriptions | 1,024/256 is a UI transport bound, not an agent bus; coalescing must precede any capacity change (`03:564`) | C12/C13 tests; S2/S3 soak; no new bus consumer without a ledger row |
| **G7** | UI data contracts/frozen events; LOD must not invent fields | React UI is the claim surface; LOD is a projection, not new state (`05:307-330`) | C16 tests + FigCraft/a11y review; no new event/command without G0 |
| **G8** | Budget policy defaults (hard $0.50, no hive/global hard cap) until operator decision | Operator wants no hard default; security wants denial-of-wallet control (`OPERATOR-DECISIONS.md:24`; `06:721-746`). A silent removal is unsafe; a silent hard cap contradicts the operator | Q17 decision + settings UI + emergency-stop test + S4 soak; preserve current caps during P0 wave |
| **G9** | Do not open web-content ingestion / Search/Fetch / model-tool expansion | T01/T04 injection unresolved; spawn/approval and context preflight are preconditions (`06:226-331,1076-1089`) | C4/C5/C7 + P1-1 redaction + tool-result fence (06 P0-6) + explicit operator setting |
| **G10** | No paid/live provider calls in the gate suite; no credentials in logs | Cost/secret policy; all acceptance above is offline wiremock (`06:1114-1130`) | QA harness allowlist; live smoke is a separate paid, explicit step |

**Migration gate detail:** v11/v12 must be additive, applied by the single writer after a verified backup, idempotent on restart, and readable by the read pool; rollback requires restoring the pre-migration file (no down-migrations). Snapshot files are untouched. No schema change ships in the same commit as a concurrency change.

---

## 8. Twenty new questions (with options and impact)

| # | Question | Options | Impact / what decides |
|---|---|---|---|
| Q1 | **Spawn approval shape:** per-spawn approval vs per-turn delegation grant vs declared `Delegate` exemption? | A per-spawn; B delegation budget grant (one approval, N children, audited); C `Delegate` class with explicit `APPROVAL_EXEMPT` + cap + audit | A is safest but 8-wait/run cap makes deep trees painful; B preserves scale and operator control (recommended); C is 06 P0-3 option (b) but needs "no silent bypass" audit. Decide before C5. |
| Q2 | **Host validation identity:** exact bind only, loopback aliases, or also IPv6/localhost variants? | A exact `127.0.0.1:<port>`; B + `localhost`/`[::1]`; C B + configurable aliases | A breaks legitimate `localhost` UI; C widens trust (`06:582-595` T15). Recommend B, no wildcard. |
| Q3 | **Is `/health`/`/diagnose` public behind Host?** | A Host-gated public; B token+Host; C fully public | Launcher/monitoring needs A; token+Host (B) is stricter but needs a bootstrap path. Pin in C2 test. |
| Q4 | **HTTP token transport:** `Authorization: Bearer`, `X-HiveMind-Token`, or cookie? | A Bearer (standard); B custom header; C same-site cookie | Bearer is standard and testable; query token for WS is existing and should move first-frame later (`06:597-611`). UI already has token in memory. |
| Q5 | **New refusal codes vs reuse:** 7 additive codes or reuse existing with distinct details? | A additive + ledger (recommended); B reuse (`E_PROVIDER_AUTH`, `E_TOKEN_BUDGET_EXCEEDED`, `E_SEMAPHORE_FULL`, `E_TOOL_TIMEOUT`); C hybrid | A is truthful and matches 03/06; B avoids contract churn but makes logs/UI lie; C ships P0 with B then A later (declared degraded). Needs Lead decision. |
| Q6 | **Context-window source:** static 1M default, `/models` metadata, env, or all? | A static; B metadata + static fallback; C env only | B is honest (metadata can change); startup must not block on provider (design uses background refresh). Tests can set env. |
| Q7 | **Estimator:** bytes/3 conservative vs tokenizer dependency vs model-reported usage only? | A bytes/3 (no new dep); B tokenizer crate (dependency/license); C ship `usage` after send (too late) | A meets safety with false-refusal risk; make the overestimate configurable and documented. No new dependency in P0. |
| Q8 | **Run deadline default:** 1 h, 30 min, or off by default? | A 1 h; B 30 min; C off | Long legitimate tasks vs runaway risk (`06:947-951`); choose A with operator setting; 0 disables. Pair with stall detector. |
| Q9 | **Stall definition:** N calls without result, wall-clock without progress, or repeated spawn signature? | A calls (8); B wall clock (180 s); C both + repeated `(role,instruction)` hash | C catches step-repetition (MAST FM-1.3) but risks false positives; A/B are deterministic. Recommended C with counters, tune after S0. |
| Q10 | **Parent-completion path:** defer completion (recommended) vs reject spawn after parent final turn vs block parent turn? | A defer + drain children; B reject spawn before completion; C park parent | A is race-free and preserves reply; B alone loses the race with a command spawn during the final provider call; C needs a new state. Choose A. |
| Q11 | **Admission fairness scope:** per-call tickets, per-task tickets, or per-run? | A call tickets (recommended); B task; C run | A avoids parent/child permit deadlock and matches 03's permit semantics; B can deadlock at permit limit; C is single-run today. Multi-run fairness is a later extension. |
| Q12 | **Queue capacity/deadline:** 2×permits/9 min vs 4×/5 min vs 1×/10 min? | A 2×/9 min (03:542); B 4×/5 min; C 1×/10 min | A bounds memory and matches provider 10-min close; B increases memory; C refuses too early. Acceptance S1 pins A. |
| Q13 | **AIMD parameters:** start 25/10 vs 120/30; +1 per 20 successes; halve on 429; floor 4? | A conservative start then climb; B start at 120/30; C fixed 25/10 for S1 only | Start must be below account limit and probe upward; 03 Q15 says start 120→150; S1 uses 25/10 for proof. Decide whether S4 start is 120 or 25 with a ramp schedule. |
| Q14 | **429 cooldown scope:** per-model, per-account, or per-client? | A per-model shared within process; B account-global; C per-client | Account limit is per-model (Flash/Pro), so A matches (`03:544,578`); account-global would couple models unnecessarily; per-client misses proxy clients. |
| Q15 | **DB batch/flush policy:** 20–50 ms/200 rows vs 10 ms/50 rows vs per-row? | A 20–50 ms/200; B 10/50; C per-row | A is the measured design (`03:242`); per-row is 24× worse; B lowers latency at higher commit count. Pick A, measure S2. |
| Q16 | **Read pool size + SQLite gate:** 2–4 readers vs 1 reader until SQLite ≥3.51.3? | A pool 2–4 after version check; B 1 reader always; C pool regardless | WAL-reset bug 3.7.0–3.51.2 is triggered by multiple connections (`03:686-692`); A is correct only after `sqlite_version()` proof; B is safe fallback. |
| Q17 | **Budget policy defaults:** keep current hard $0.50/400k/40 calls? Soft default + emergency stop? Hive/global caps on/off? | A keep until settings; B soft default + high emergency global hard cap; C remove all hard caps now | Operator says no hard default (`OPERATOR-DECISIONS.md:24`) but security requires denial-of-wallet control; A is safest for P0, B matches intent with a visible emergency stop, C is unsafe. Decide before C15; do not silently change. |
| Q18 | **Reservation basis:** reserve max output vs expected output; settle missing usage how? | A reserve max (safe but blocks concurrency); B reserve expected + safety; C no reservation | A can deadlock concurrent calls; B is practical with a reserve cap; missing usage must keep reservation and mark INCOMPLETE (`06:399-400`). |
| Q19 | **Delta coalescing tick:** 16 ms vs 33 ms vs adaptive (frame-rate-based)? | A 16; B 33; C adaptive | 16 ms is smoother but more messages; 33 ms halves volume; 03 says 16–33. Acceptance measures max latency and coalesce ratio; choose 16 initially and tune with S2/S3. |
| Q20 | **Viewport subscription contract:** client-only culling first vs add a `VIEWPORT_SET` command/event now? | A client-only S5, contract later; B add command now; C server sends only visible node deltas | A preserves G0 and still passes ≤120 mounted via culling; B changes frozen contract and needs a ledger row; C saves bandwidth but requires client/server protocol. Recommend A, then B if S5 bandwidth fails. |

Additional decisions needed but not polled (documented for Lead): whether `E_SESSION_SLOW` is added (Q5), whether same-parent `send_to_sibling` needs approval (T11 P2; operator "ordinary chatter no"), and whether `HIVEROUTER_ALLOWED_ORIGINS` is removed/validated (T15).

---

## 9. Evidence register, non-claims, and handoff

### 9.1 Material claims and status

| Claim | Evidence | Status |
|---|---|---|
| Current execution is sequential (max 1 task/provider call) | `execution.rs:1395-1550,2668-2681`; no `join_all`/`FuturesUnordered` in core | `VERIFIED` (source + absence) |
| Budget mutex is held across the whole stream | `provider_executor.rs:140-147,162-243` | `VERIFIED` |
| `spawn_agent` bypasses the declared Execute approval | `execution.rs:1684-1691`; `tool.rs:332-339`; `06:372-401` | `VERIFIED` |
| Host validation absent and `/api/session` trusts loopback without Origin | `http.rs:510-531,1017-1043`; `06:528-566` | `VERIFIED` |
| 401/402 merged; 400/422 called invalid response | `client.rs:500-524`; `error_matrix.rs:141-180` | `VERIFIED` |
| Context-window metadata parsed but unused; estimator bytes/4 | `dto.rs:892-907`; `provider_executor.rs:436-442`; grep no consumer | `VERIFIED` |
| Parent completion does not check descendants; queue drained after aggregation | `execution.rs:2973-3030,1486-1490,1069-1073`; `02:707` | `VERIFIED` |
| No run deadline/stall | `execution.rs:1245-1307`; `config.rs:190-195`; `02:717` | `VERIFIED` |
| Clamps: 64/8/3/40/400k/$0.50, limiter 8 | `caps.rs:8-15`; `budget.rs:24-43`; `provider_executor.rs:104-107` | `VERIFIED` |
| Event bus 1,024/256 and per-fragment delta | `bus.rs:18-20`; `execution.rs:1764-1795` | `VERIFIED` |
| DB one mutex connection; WAL/busy 5 s | `db.rs:295-350` | `VERIFIED` |
| Session loop sends inline; lag → resync | `ws.rs:401-462,1013-1024` | `VERIFIED` |
| UI renders all nodes; no LOD/culling | `Canvas.tsx:351-405`; `graph.ts:143-165` | `VERIFIED` |
| Scale target 50–150/300–600 is impossible today | derived from the above + `03:88-106,645-652` | `INFERRED` (runtime `UNVERIFIED`) |
| All proposed default numbers (deadlines, permits, batch sizes) | design recommendations, not measured facts | `UNVERIFIED` until S0/S1 |
| Published docs' line numbers are current in all cases | several drift (`03:101-103` vs actual `bus.rs:18-20`, `execution.rs:1764-1795`) | `READ` (claims true, citations stale) |

### 9.2 Non-claims and open unknowns

**Do not claim from this lane:** that any P0 is fixed (no code written); that 25/150/600 concurrency works; that the proposed defaults are operator-approved; that Host/token/contract changes are accepted; that the dirty persona fix is tested/committed; that the budget mutex/AIMD/DB actor design has been compiled or load-tested; that SQLite bundled version supports the read pool; that the UI reaches 50 fps at 600 nodes; that no new attack exists in the proposed middleware.

**Open unknowns:** actual bundled SQLite version (`SELECT sqlite_version()`, U6); runtime stream rate and 429 headers; whether `client.complete`/`stream` can be made reservation-based without breaking existing tests; whether the current `RunEngine` APIs (`execute_task`, `complete_task`) are private enough to change (they are private, same module; tests may access them); actual operator policy for Q1/Q5/Q17; whether any other lane is already editing `provider_executor.rs` (the dirty diff proves one lane was); whether a second run may exist during S1 fairness tests (simulate at scheduler level).

### 9.3 Handoff to Lead

1. **Resolve Q1/Q5/Q17 before C1/C5/C15:** spawn approval shape, contract-delta acceptance, budget defaults. These are product/security decisions, not code details.
2. **Make `09:247` the union P0 list with owners**; assign the two orphaned P0s (`ask_parent`, planner blank retry) explicitly; otherwise the first wave's own gate cannot close.
3. **Land C0 immediately:** commit the dirty persona fix with its test; otherwise every later commit is built on an untracked P0.
4. **Order:** P0 waves C1–C8 → S0 harness C9 → S1 C10 → S2 C11–C12 → S3 C13 → S4 C14–C15 → S5 C16–C17. Do not start C10 before C5/C6.
5. **Do not change** the frozen contract (beyond C1), caps, DB schema (beyond additive v11/v12), or single-run semantics until the relevant gate in §7 passes.
6. **Evidence:** every acceptance test above is offline; no paid call, no credential, no product edit in this lane.

**Read-only statement:** this lane wrote **only** `docs/research/hmars/17-scale-security-impl.md`. No product source, tests, config, gate, git state, credential, server process, provider call or paid call was touched; no build/test was executed. Runtime claims remain unverified.
