# 26 — Scale/scheduler/security implementation audit (wave 3, seat 26)

**Seat:** Wave-3 independent audit of the scale/security implementation plan. **Read-only.**
**Date/time:** 2026-11-07 21:53–22:05 America/Chicago (CST).
**Repository:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `bd575ba` `feat: real SuperAgent/Subagent personas and persona prompt execution` (unchanged during this pass).
**Working tree:** dirty and moving. At the start of this pass 17 tracked paths were modified (12 product source/test/UI/launcher paths plus docs and build artifacts) and four paths were untracked; the important ones are `execution.rs`, `tool_loop.rs`, `provider_executor.rs`, `http.rs`, `tests/http_boot.rs`, `tests/tool_loop_engine.rs`, `tests/tool_loop_matrix.rs`, `HiveMindLauncher.cs`, `web/src/panels/ApprovalOverlay.tsx`, `web/src/state/store.ts`. None of these edits were made by this seat.
**Live runtime:** HiveMind was running from `C:\Users\Riley\Desktop\HiveMind-Portable` with `--bind 127.0.0.1:8099 --data-dir C:\Users\Riley\AppData\Local\HiveMind --workspace-root C:\Users\Riley\Desktop\hiverouter-v1` (process 9752 at 21:53). It answered read-only probes at **21:54:59 CST** and was **down by 21:58:45 CST**. **Late-breaking (22:00–22:04):** another lane rebuilt `dist\HiveMind\hiverouter.exe` at **22:00:18**, restarted the app at **22:00:32**, and the restarted live server now returns **421 `E_ORIGIN_REJECTED`** for the hostile `Host` probe (verified 22:03:54); the earlier 21:54:59 200-token observation remains the deployment counterexample. The provider `client.rs` was also edited at **22:01** (after that binary), so live is still a mixed build. All runtime claims are timestamped observations, not a stable baseline.
**Primary skill:** `/deep-inspection` is **not installed** in this profile. This seat used **`/deep-investigate`** (baseline → three hypotheses → discriminating probe/elimination → root cause) and **`/trace`** (UI/control → handler → client → route → service/store → provider → response/render-back) as the closest available disciplines. The substitution is recorded because the task named a primary skill.
**What this seat did not do:** no product edit, no test/build, no server restart, no paid provider call, no credential read, no browser action, no git write. All provider/status claims are source-read; all live claims are GET-only.
**Inputs read first:** `OPERATOR-DECISIONS.md`, `09-synthesis-and-goal.md`, `10-gap-meta.md`, `17-scale-security-impl.md`, `03-scale-concurrency.md`, `06-security-cost.md`, `07-code-backend.md`, plus current `backend/crates/hiverouter-core/src/{execution,http,provider_executor,tool_loop,db,graph,snapshot,history,recovery,tools_bridge,ws}.rs`, `hiverouter-provider/src/{client,dto,budget}.rs`, `hiverouter-contract/src/{refusal,event}.rs`, `HiveMindLauncher.cs`, `web/src/state/store.ts`, `web/src/panels/ApprovalOverlay.tsx`, and the live app at `127.0.0.1:8099` (GET-only).

---

## 0. One-screen answer: the top 10 audit findings

1. **The Host fix was absent for most of the audit and appeared only via a late, unverified rebuild.** At **21:54:59 CST**, `GET /api/session` with `Host: rebind.evil.example:8099` returned **HTTP 200 and a token**; the then-running binary (21:43:54) predated the Host source (21:52:53). At **22:00:18** another lane rebuilt the binary and restarted the app; at **22:03:54** the same probe returned **421 `E_ORIGIN_REJECTED`** (with no `retryable` field). So the live gate is currently present, but the process proved a source/deploy skew, no Host test exists, and the token/read endpoints remain unauthenticated (`/api/agents/memory`, `/api/history/runs`, `/api/agents/profiles`, `/api/session` returned 200 at 22:03:54). `VERIFIED (live, two timestamps)`.
2. **The source Host fix is itself untested and incomplete.** It returns a custom **HTTP 421** JSON body with `code/message/how_to_fix` but **no `retryable`** (confirmed live at 22:03:54: `hostile_code=E_ORIGIN_REJECTED`, `retryable_present=False`), unlike `refusal_response`; there is **no `http_origin.rs`/Host test** in `backend/crates/hiverouter-core/tests`, and the port-0 test branch accepts any `127.0.0.1:<anything>` authority (`http.rs:570-592`). The live token/bootstrap endpoint remains loopback-without-Origin (`http.rs:1080-1104`). `READ + VERIFIED (live body) `.
3. **Snapshot scope is a P0 defect, not a history nicety — and wave 2's report 17 omitted it.** `snapshot::capture` stores `graph.snapshot()` — the process-global graph (`snapshot.rs:258-271`) — and `TaskNode` has no `run_id` (`graph.rs:18-27`). Live per-run snapshots grow monotonically with disjoint runs: **14, 20, 26, 31, 35 nodes** and **5, 6, 7, 8, 9 roots**; the newest completed run's `state` contains **35 nodes/9 roots**. The client decodes every node (`web/src/state/store.ts:571-622`), the history route is unauthenticated, and boot recovery restores the global image (`recovery.rs:149-177`). `VERIFIED (live + source)`; this is also `10-gap-meta.md` top finding #1 and D1 (`10:35,326,522`), which 17 never incorporates.
4. **The 12/40 tool-cap fix is still the wrong accounting unit.** `MAX_TOOL_TURNS` is documented as "one task" (`tool_loop.rs:13-14`) but `execute_tool_turn` reads **per-run** counters (`execution.rs:1590-1595`) from `runs.tool_turns` (`db.rs:652-705`). A multi-task run shares one 12-turn budget: later tasks can hit the cap on their first tool call. Raising 4→12 does not fix that. `VERIFIED (source)`.
5. **`E_CALL_LIMIT` still lies about which limit fired.** It is returned for the 12-turn cap (`execution.rs:2845-2849`), the 40-tool-call cap (`execution.rs:1640-1650`), the provider 40-attempt cap, and the **8-approval-wait cap** (`execution.rs:2023-2033`); the message says "the 40-call cap or the 40-super-step plan ceiling" (`refusal.rs:158-160`) and never names turns/approvals/task. `10` found the old `E_BUDGET_EXCEEDED` lie; the working fix only renamed the lie. `VERIFIED`.
6. **Approval args preview is lost on reconnect/replay.** The new UI renders `args_preview` (`ApprovalOverlay.tsx` diff; `store.ts` diff), and fresh requests attach it (`tools_bridge.rs:439-505`), but the replay projection deliberately sets `args_preview: None` (`tools_bridge.rs:502-518`; `pending_approval_events` at `:528-538`, `pending_approval_events_all` at `:550+`) and WS replay uses that path (`ws.rs:567-589`). After reload/reconnect the operator sees `UNKNOWN`, no preview, only a digest. `VERIFIED (source)`.
7. **Spawn approval is still bypassed.** The working tree did not add an approval branch: `spawn_agent` is still handled before `suspend_for_approval` (`execution.rs:1684-1691`), and the new approval-wait budget is 8 per run (`tool_loop.rs:21`; `execution.rs:2023-2033`). A fix that gates spawn per call will be constrained by that 8-wait cap, not by 12 turns. `VERIFIED (source)`.
8. **402/400/422 codes are still wrong; a late 22:01 edit only improved the text.** At the audit read, `classify_status` mapped `401 | 402` → `E_PROVIDER_AUTH` and 400/422 → `E_PROVIDER_INVALID_RESPONSE` (`client.rs:500-524`); a concurrent lane changed `client.rs` at 22:01 (after the 22:00 binary) to give 402 a balance-specific **message** but still `Refusal::ProviderAuth`, and 400/422 a request-shape **message** but still `Refusal::ProviderInvalidResponse`. The wrong 402 test still asserts `E_PROVIDER_AUTH` (`provider/tests/error_matrix.rs:161-180`), and live predates even that edit. Context-window preflight is still absent (`provider_executor.rs:436-442`; no `context_window` consumer). `VERIFIED (source) + time-scoped`.
9. **The launcher's `--workspace-root` fix is dev-machine-specific and unsafe at the argument boundary.** `DefaultRoot` is hard-coded to this checkout (`HiveMindLauncher.cs:17`); a portable copy falls back to `Directory.GetCurrentDirectory()` (`:163-178`), which is the original "agents analysed `dist\HiveMind`" failure (`10:37,529`). The path is concatenated into an `Arguments` string in quotes (`:61`) rather than `ArgumentList`; a quote/trailing backslash in `HIVEMIND_WORKSPACE_ROOT` can alter arguments. The launcher binary at 21:43 and the live process use the flag, but portable fallback and argument safety are untested and absent from the 17 plan. `VERIFIED (source + live command) + UNVERIFIED (portable behavior)`.
10. **17 contains three design contradictions that will fail at implementation time.** (a) Per-model AIMD is impossible with one configured model: `ProviderSettings.model` is the same model for planner and leaf (`provider_executor.rs:298,331,416`; `config.rs:40-70`), so "Flash lease vs Pro lease" is one bucket. (b) The C5 parent-completion deferral `drain_children_of` can execute a child directly while C10's concurrent scheduler also owns it; no claim/lease is specified (17:315-350, C10 in 17:571). (c) The DB-actor plan omits `SnapshotStore::write`, which performs its own direct `db.with_connection` index write (`snapshot.rs:75-102`), so the actor would not cover all writes. `VERIFIED (source against 17)`.

The rest of this file is the evidence, the 20 hardest gaps, the corrected implementation order, the five required P0 patches, and 20 questions the waves still have not answered.

---

## 1. Method, baseline, and the current-state delta

### 1.1 `/deep-investigate` baseline and hypotheses

Baseline recorded before any interpretation:

| Fact | Value | Evidence |
|---|---|---|
| HEAD | `bd575ba` | `git rev-parse HEAD` |
| Dirty product files | At first read: 12 product source/test/UI/launcher paths under 17 tracked modified paths; **late addition:** `backend/crates/hiverouter-provider/src/client.rs` modified 22:01 (now 18 tracked modified paths) | `git status --porcelain` at 21:53 and 22:03 |
| Dist binary (first read) | `dist\HiveMind\hiverouter.exe` 15,107,072 B, mtime **2026-11-07 21:43:54** | `Get-Item` |
| Dist binary (late rebuild) | 15,127,040 B, mtime **22:00:18**; launcher 9,216 B, mtime **22:00:22** | `Get-Item` at 22:03 |
| Launcher binary (first read) | `dist\HiveMind\HiveMind.exe` 9,216 B, mtime **21:43:59** | same |
| Source mtimes | `execution.rs` 21:31:35, `tool_loop.rs` 21:34:26, `provider_executor.rs` 21:16, `http.rs` **21:52:53**, `client.rs` **22:01:00**, `ApprovalOverlay.tsx` 21:39, `HiveMindLauncher.cs` 21:31 | `Get-Item` |
| Live process | `hiverouter.exe --bind 127.0.0.1:8099 --data-dir C:\Users\Riley\AppData\Local\HiveMind --workspace-root C:\Users\Riley\Desktop\hiverouter-v1` (PID 9752 at 21:53; PID 16616 after 22:00 restart) | `Get-CimInstance Win32_Process` |
| Live health at 21:54:59 | status ok, version 1.2.0-dev, DB schema 10, `last_seq:0`, `graph_nodes:35`, 2 origins, DB `C:\Users\Riley\AppData\Local\HiveMind\hiverouter.db` | GET `/health` |
| Live health at 22:03:31 | status ok, `last_seq:12`, `graph_nodes:35`; app restarted at 22:00:32 | GET `/health` |
| Live `/diagnose` at 21:54 | keys only; `side_effects_enabled=true`; no workspace/root field | GET `/diagnose` |
| Live Host test at 21:54:59 | normal Host 200 + token; **hostile Host 200 + token** | `curl -H 'Host: rebind.evil.example:8099' /api/session` |
| Live Host test at 22:03:54 | normal Host 200; **hostile Host 421**, `code=E_ORIGIN_REJECTED`, `retryable` absent, no token | same probe after rebuild |
| Live unauth reads at 21:54 / 22:03 | `/api/agents/memory` 200 (7 profiles), `/api/history/runs` 200 (5 runs), `/api/projects` 200 (1), `/api/agents/profiles` 200 (7), a run's messages 200 (4); host gate does not authenticate | GET, status/count only |
| Live per-run snapshot | 14/20/26/31/35 nodes; 5/6/7/8/9 roots; latest run completed at 35/9 | `/api/history/runs/{id}/state?revision=latest` |
| Live down | `/health` 000, no process, at 21:58:45 | `curl`, `Get-CimInstance` |

The live application is **not quiescent** and **not rebuilt atomically from current source**: the Host source changed 9 minutes after the first binary, and `client.rs` changed 41 seconds after the late binary. The late rebuild proves the process can move under a report; it does not prove the source fixes are tested. Three hypotheses were tested for each symptom; the elimination is recorded here so the root causes are mechanisms, not correlations:

| Symptom | H1 | H2 | H3 | Eliminating probe | Confirmed root cause |
|---|---|---|---|---|---|
| Host rebinding returns a token | Provider/Origin layer | **Binary predates Host middleware** | Browser/CORS | Source mtime 21:52 vs binary 21:43 + live hostile Host 200 at 21:54; after 22:00 rebuild hostile Host 421 | Deployment/source skew for the first probe; the late rebuild fixes the Host gate but not the absence of tests/token; live/source still mixed because `client.rs` is newer than the binary. |
| Tool-task fails with `E_CALL_LIMIT` after few turns | Provider 40-call cap | **Per-run tool-turn counter** | Tool call cap | `execution.rs:1590-1595` reads `run_tool_counters`; `db.rs:652-705` are per-run; no task filter; live old runs failed at 4 total turns | Per-run counter with a per-task constant. |
| Snapshot contains foreign nodes | Client projection | DB run scoping | **Global graph capture** | `snapshot.rs:258-271` `graph.snapshot()`; live node counts grow 14→35 across disjoint runs | `capture` stores the process graph; no `run_id` in `TaskNode`. |
| Approval preview disappears after reconnect | UI type | **Replay projection drops preview** | Contract | `tools_bridge.rs:502-518` sets `None`; WS replay uses `pending_approval_events_all` (`ws.rs:580`) | Preview is only attached when live args are in memory. |
| Spawn has no approval | Provider | **Loop dispatch bypass** | UI | `execution.rs:1684-1691` branch precedes approval | Unchanged P0. |
| Context overrun after send | Provider rejects | **No context-window consumer** | Estimator only | `grep context_window` finds only DTO/tests; `estimate_prompt_tokens` bytes/4 | Missing preflight. |

**No live reproduction of a paid/provider path was possible or permitted.** Runtime claims below are `UNVERIFIED`; source-path claims are `VERIFIED`/`READ`. The live app went down during the pass and was restarted by another lane at 22:00; the Host gate changed behavior between the two observations. Any statement below without a timestamp is source-only.

### 1.1a Late-breaking rebuild (observed 22:00–22:04)

| Time (CST) | Observation | Evidence |
|---|---|---|
| 22:00:18 | `dist\HiveMind\hiverouter.exe` rebuilt to 15,127,040 B | `Get-Item` |
| 22:00:22 | `dist\HiveMind\HiveMind.exe` rebuilt to 9,216 B | `Get-Item` |
| 22:00:32 | `hiverouter.exe` restarted (PID 16616) with the same 8099/data-dir/workspace-root command | `Get-CimInstance Win32_Process` |
| 22:01:00 | `backend/crates/hiverouter-provider/src/client.rs` modified **after** the 22:00 binary: 402 still `Refusal::ProviderAuth` with a balance message; 400/422 still `Refusal::ProviderInvalidResponse` with a request-shape message | `git diff client.rs` |
| 22:03:31 | `GET /health` 200, `last_seq:12`, `graph_nodes:35` | GET |
| 22:03:54 | hostile `Host` now **421 `E_ORIGIN_REJECTED`**, `retryable` absent, no token | `curl` |
| 22:03:54 | `/api/session`, `/api/agents/memory`, `/api/history/runs`, `/api/agents/profiles` still **200** without a session token | GET status only |

**Implications:** (a) the Host gate is currently deployed, but only because a concurrent lane rebuilt and restarted; the 21:54 200-token observation remains the proof that source presence is not deployment; (b) live is still a **mixed build** because `client.rs` is newer than the binary; (c) the missing-token and 421-body gaps remain; (d) no live tool-cap/approval/provider behavior was exercised in either build.

### 1.2 `/trace` verdicts on the specific UI/claim surfaces

| Surface | Render | Handler/client | Route | Service/store | Response/render-back | Verdict |
|---|---|---|---|---|---|---|
| Host guard | n/a | n/a | live 200/token at 21:54; live 421 at 22:03 | binary 21:43 lacked middleware; 22:00 rebuild includes it | hostile rejected after 22:00, but `retryable` missing and token/read paths still 200 | **PRESENT AFTER 22:00; STILL WRONG-SHAPE + UNAUTHENTICATED READS** |
| Approval preview | UI field added (`ApprovalOverlay.tsx` diff) | event store reads `args_preview` | `APPROVAL_REQUEST` event | live request has preview; replay has None (`tools_bridge.rs:502-518`) | after reconnect `UNKNOWN` | **WIRED-BUT-DEGRADED** |
| Tool caps | run failure shows E_CALL_LIMIT | engine returns code | WS/run status | counters per run | UI reports one code for four limits | **WRONG ATTRIBUTION** |
| Snapshot/replay | canvas renders all snapshot nodes | `recordsFromSnapshotGraph` decodes all | `/api/history/.../state` 200, no auth | global graph | foreign roots visible | **WIRED-BUT-WRONG-SCOPE** |
| Workspace root | no header shows resolved root | launcher passes flag | server accepts | live process shows repo root | operator cannot verify on another machine | **PARTIAL** |
| Live/source identity | `/health` version only | no build hash | no route | binary mtime/source mtime disagree | no proof | **MISSING-GATE** |

### 1.3 Working-tree fixes: what changed, and what is actually proven

| Fix | Source change | Committed? | Test updated? | Build evidence | New defect exposed by this audit |
|---|---|---|---|---|---|
| Tool-turn/call caps 4/16 → **12/40** | `tool_loop.rs:13-21,250-269`; tests `tool_loop_engine.rs`, `tool_loop_matrix.rs` | **No** | Updated, no pass evidence in this pass | Dist binary 21:43 is after source 21:34, so *likely* included (`INFERRED`, not proven) | Cap is **per run**, not per task; `E_CALL_LIMIT` text does not say turn cap; no multi-task test exists. |
| `E_BUDGET_EXCEEDED` → **`E_CALL_LIMIT`** | `execution.rs:2845-2849`; `tool_loop.rs:254-269`; `refusal.rs:158-160` | No | Updated, no pass evidence in this pass | likely included | One code covers turns, tool calls, provider attempts, approval waits; message says "40-call cap or 40-super-step plan ceiling". |
| **Host allowlist** middleware | `http.rs:509-594`; new `host_guard`/`host_allowed`/`loopback_host` | No | **No test added** (`http_origin.rs` absent) | Absent from the 21:43 binary; **included in the 22:00 rebuild** (hostile 421 at 22:03); source tests still absent | 421 custom body lacks `retryable`; test port-0 wildcard; token/read auth still absent; live 21:54 bypass is the counterexample for source→deploy. |
| Provider status mapping (late) | `client.rs` 22:01 diff | No | Wrong 402 test still asserts `E_PROVIDER_AUTH` (`error_matrix.rs:161-180`) | **Newer than the 22:00 binary** | 402 still `E_PROVIDER_AUTH`; 400/422 still `E_PROVIDER_INVALID_RESPONSE`; messages improved only. |
| Launcher `--workspace-root` | `HiveMindLauncher.cs:57-61,163-178`; binary 21:43 | No | No test evidence | Live process shows the flag and repo root; portable fallback untested | Hard-coded dev root; portable fallback cwd; string argument concatenation; no visible resolved root in run header. |
| **Approval args preview** | `web/src/panels/ApprovalOverlay.tsx`, `store.ts`; backend already had `redact_args_preview` (`event.rs:286-370`, `tools_bridge.rs:439-505`) | No | No dedicated test | likely in web bundle 21:43 | Replay projection omits preview (`tools_bridge.rs:502-518`); preview not persisted; no test for reconnect. |
| Persona normalization/continuation | `provider_executor.rs:82-92,375` | No | No dedicated test in tree (17 proposes `tool_turn_persona.rs`, absent) | likely included | Still uncommitted; HEAD behavior differs; any doc claiming fixed at HEAD is false. |
| Test expectations updated for 7 personas | `http_boot.rs`, `prompt_assets.rs`, `prompt_assets_wiring.rs` | No | Yes, unrun | not run | Tests are part of dirty tree; a clean checkout does not contain them. |

**Delta against 17 and 10.** `10` (lane 1, written 21:20–21:40) already found the snapshot scope, the false budget error, the wrong workspace, and the dirty-tree hash problem (`10:35-46,92-113,312-435,1019-1138`). `17` (lane 8, written earlier) **does not incorporate any of those** and still treats the HEAD-era 4-turn/tool-cap world, the Host fix as a 403 test, and a per-model AIMD that the configuration cannot express. 26 must therefore supersede those specific claims, not restate 17's plan.

---

## 2. The 20 hardest gaps (ranked by blocking power)

Severity: **P0** = truth/security/correctness gate; **P1** = required before any scale claim; **P2** = design debt. Each row names the exact seam and the minimum test/rollback; the detailed patches are in §4–§5.

| # | Gap | Severity | Evidence (current source / live) | Why wave 1/17 missed it | Exact seam → required test/rollback |
|---:|---|---|---|---|---|
| 1 | **Host gate was absent for the audit window and only appeared via a late, unverified rebuild** | P0 | 21:54:59 hostile Host → 200 token; binary 21:43:54 < http.rs 21:52:53; 22:00 rebuild then 22:03:54 hostile Host → 421; unauth reads still 200; no Host test | 17/06 treated source presence as fix; no one compared binary mtime to source or probed hostile Host at both times | `http.rs:509-594`; build/deploy gate + fingerprint; test `http_origin.rs::rebound_host_header_is_rejected`; rollback = revert deploy + env off only in dev. |
| 2 | **Host guard is wrong-shape and leaves token/read paths open** | P0 | Live 22:03:54: hostile 421 `E_ORIGIN_REJECTED` with `retryable_present=False`; `/api/agents/memory`, `/api/history/runs`, `/api/agents/profiles`, `/api/session` still 200; no `http_origin.rs`; port 0 accepts any loopback authority `:570-592`; `api_session` still loopback-without-Origin `:1080-1104` | 17's Host section assumed a 403 `refusal_response` and a token gate; no one checked the implemented variant or reads | `http.rs:551-592,1080-1104`; tests foreign/missing Host on every route, no-token reads 401, no mutation; rollback feature env + revert. |
| 3 | **Snapshot scope: global graph captured; cross-run topology in history/replay/UI** | P0 | `snapshot.rs:258-271`; live 14/20/26/31/35 nodes, 5→9 roots; `TaskNode` no `run_id` (`graph.rs:18-27`); `state_at` returns unfiltered (`history.rs:109-115`); UI decodes all (`store.ts:571-622`) | 10 found it; 17 omitted it entirely; first wave assumed run scope (`05:92`, `08:590-592` per 10) | Snapshot v3 + `run_id`/filter; `snapshot.rs`, `graph.rs`, `history.rs`, `recovery.rs`; tests two runs, state has only run's ids; rollback = v2 read compatibility, DB backup. |
| 4 | **Recovery restores global image and can publish foreign graph nodes under one run** | P0 | `recovery.rs:149-177` restores each `latest()` via `graph.restore_from_snapshot`; `:236-257` iterates `graph.snapshot().nodes` and publishes with `restored_run`; live after restart `graph_nodes:35` | 17/10 did not trace recovery; 10 only measured file scope | `recovery.rs:139-177,232-302`; tests `snapshot_scope::recovery_restores_only_run`, no foreign `run_id` in events; rollback = restore backup. |
| 5 | **Tool-turn cap is per-run, not per-task** | P0/P1 | `MAX_TOOL_TURNS=12` comment says one task (`tool_loop.rs:13`); `execute_tool_turn` uses `run_tool_counters(run.id)` (`execution.rs:1590-1595`); counters in `runs` (`db.rs:652-705`); no task filter; tests single-task only | 10 noticed 4-run failures but not the accounting unit; 17 didn't mention tool caps at all; the working fix raised numbers | Add `Db::task_tool_counters(run,task)` using `tool_turns` (`db.rs:573-650`); use it for `next_turn`; test two tasks each get 12; rollback = revert, behavior old per-run. |
| 6 | **`E_CALL_LIMIT` conflates four limits; no attribution** | P0/P1 | `refusal.rs:158-160` text; four producers: turns `execution.rs:2845-2849`, tool calls `:1640-1650`, provider attempts `provider/budget.rs:243-251`, approval waits `execution.rs:2023-2033`; no task id in detail | 10 found old `E_BUDGET` lie; 17 accepted one code; no one counted the four producers in the working tree | Split contract codes or add typed detail; `refusal.rs` + ledger; tests each producer's code; rollback = leave additive codes unused. |
| 7 | **Approval-wait cap 8 is smaller than 12 turns and shares the wrong code** | P0/P1 | `tool_loop.rs:21`; `execution.rs:2023-2033` returns `E_CALL_LIMIT`; `approval_count_for_run` per run | 17's spawn approval plan uses 8 waits; no one reconciled with 12 turns | `execution.rs:2023-2033`, `tool_loop.rs:21`; test 9 approvals; rollback = configurable cap. |
| 8 | **Approval args preview is not durable/replayable** | P1 (P0 UX) | Backend replay `args_preview: None` (`tools_bridge.rs:502-518`); `pending_approval_events_all` (`:550+`); WS replay (`ws.rs:580`); UI new field | 17/10 treated preview as web-only; no one traced replay | Persist bounded redacted preview in approval row or a receipts table; migration; test reconnect + secret redaction; rollback = additive column/read none. |
| 9 | **Launcher workspace root is non-portable and argument-unsafe** | P0 usefulness | `DefaultRoot` hard-coded (`HiveMindLauncher.cs:17`); fallback cwd `:163-178`; `Arguments` string `:61`; no per-run root/header | 10 flagged hard-coded dev path (`10:103`); 17 didn't include launcher | `HiveMindLauncher.cs:57-69,163-198`; test portable copy/no default + quoted/env path; use `ArgumentList`; rollback = remove flag (old bug) not recommended. |
| 10 | **Spawn approval still bypassed** | P0 authority | `execution.rs:1684-1691`; no branch; loop-owned comment | 17 P0-1 remains unimplemented; working tree didn't touch it | `execution.rs:1682-1736`; tests `approval_coverage.rs`, `engine_spawn_approval.rs`; rollback = revert, no release bypass. |
| 11 | **402/400/422 codes still wrong; late 22:01 edit only improved messages** | P0 money/diagnosis | `client.rs:500-524`: 402 → `ProviderAuth` (wrong code), 400/422 → `ProviderInvalidResponse` (wrong code); late diff at 22:01 adds balance/request-shape detail but no new refusal; wrong 402 test `error_matrix.rs:161-180`; no balance freeze; live binary predates the edit | 17 planned it; 26 initially read pre-edit source; late concurrent edit only changed text, not codes | `client.rs:492-524`, `http.rs:937-945`, `refusal.rs`; tests 401/402/400/422; rollback = additive codes. |
| 12 | **Context-window preflight unchanged; estimator under-counts** | P0 availability/cost | `dto.rs:892-907` parses window; `provider_executor.rs:436-442` bytes/4; no consumer; no `HIVEROUTER_CONTEXT_WINDOW` | 17 planned it; no implementation; 01 gap H | `provider_executor.rs:94-151,291-429,436-442`; test zero HTTP above window; rollback env 0. |
| 13 | **Snapshot/history endpoints still unauthenticated at 22:03; Host alone does not authenticate** | P0/P1 | Live `/api/history/*`, `/api/agents/memory`, `/api/agents/profiles` all 200 after the 22:00 Host rebuild; `/api/session` 200; token gate absent | 17/06 mention T12; 17's Host-only P0 understates; no token implementation | `http.rs:511-534`; token middleware + web fetch; tests no token 401, no mutation; rollback token off dev. |
| 14 | **17 per-model AIMD cannot be expressed: one model for planner and leaf** | P1 design blocker | `ProviderSettings.model` (`config.rs:40-70`); `provider_executor.rs:298,331,416` uses one model; no planner/leaf model selector | 17/03 assumed Flash/Pro per-model leases; 17 didn't verify the config | Add `planner_model`/`leaf_model` + request model, or make lease single-model; test two model settings; rollback config defaults. |
| 15 | **17 admission + parent guard can execute a child twice** | P1 design blocker | 17 C5 `drain_children_of` (`17:341-350`) and C10 concurrent scheduler (`17:571`); inline spawn `take_queued_task` (`execution.rs:2670`); no claim/lease in 17 | 17 was written before the parallel design's ownership was examined; 10 didn't review 17 | Single execution claim per task: scheduler-owned `JoinHandle`/reservation; parent guard awaits it; test one child exactly once under race; rollback feature flag sequential. |
| 16 | **17 DB-actor plan omits `SnapshotStore.write` index writes and migrations** | P1 design blocker | `snapshot.rs:75-102` direct `db.with_connection`; `execution.rs:1189-1205` calls snapshot write from async; 17:255-287 only store/DB | 17 didn't trace snapshot writes; 10 found scope but not actor seam | New DB handle must include `SnapshotStore` index writes and migration bootstrap; tests write p99 including snapshot; rollback legacy DB. |
| 17 | **17 event coalescing has no terminal/cancel/replay ordering proof** | P1 design blocker | `reply_delta_sink` per call (`execution.rs:1764-1795`); `REPLY_DONE`/`RUN_STATUS` control; no coalescer code/tests in 17 | 17 describes but does not specify flush-on-terminal vs control seq; no two-run replay test | `execution.rs:1764-1820,2791,2922`; tests no text loss, final flush before REPLY_DONE, cancel/fail flush once, replay seq; rollback env off. |
| 18 | **Live app was non-quiescent, went down and was rebuilt/restarted mid-audit; no build fingerprint** | P1 process | 21:54:59 health ok/graph35; 21:58:45 000/no process; 22:00:18 binary rebuilt, 22:00:32 restart, 22:03:31 health 200/last_seq12; `/health` has version/schema only; `client.rs` 22:01 after binary | No first-wave report compared live binary mtime to source; 9/17/10 use HEAD but not deployment | `/health` add build/worktree fingerprint; deploy gate; report timestamp; rollback none. |
| 19 | **Tool-cap tests were changed but have no pass evidence in this pass; no multi-task/approval/provider interaction** | P1 evidence | Diff `tool_loop_engine.rs`, `tool_loop_matrix.rs`; no tool-loop test execution was observed here; tests assume single task; `MAX_APPROVAL_WAITS` not exercised at 12/40 | Working fixes uncommitted; no CI gate | Run tests on quiet tree; add `multi_task_turn_budget` and `approval_wait_at_eight`; rollback = revert tests+code together. |
| 20 | **17's P0 set and the wave-3 brief conflict: snapshot scope vs parent guard/deadline** | P1 process | 17 P0 list (17:450-551) has spawn/Host/mapping/context/parent/deadline/orphans; wave-3 brief adds **snapshot scope**, omits parent/deadline; 09:247 has parent/deadline; 10:D1 says snapshot is #1 decision | Every wave has a different P0 list; no owner matrix with deployment evidence | Freeze a union P0 list with owner/commit/evidence; add snapshot scope to 17's P0; retain 09's parent/deadline until their gates; rollback n/a. |

**Why these are genuinely new (not repeats of 17's ten):** 17's gap list is a design-first inventory of scale/security **without** (a) live deployment truth, (b) the per-run tool-turn accounting unit, (c) approval-preview replay, (d) snapshot scope/recovery/UI, (e) the launcher's portable/injection issues, (f) 17's own per-model-AIMD/scheduler/DB-actor contradictions, and (g) the working-tree fixes' uncommitted/unbuilt status. 10 found (d) and the false budget; this audit extends it with recovery, v3 migration, auth, and the 17 design collisions.

---

## 3. Exact implementation order for admission, fair scheduler, AIMD, DB actor, and event coalescing

The order below is a **dependency order**, not a wish list. The wave-2 C10–C15 sequence is directionally right but must be corrected: snapshot scope and per-task cap accounting are prerequisites; DB actor must include snapshot writes; AIMD must be preceded by the budget-lock split and the model-scope decision; coalescing must prove terminal ordering.

### 3.1 Order (with reasons and gates)

| Step | Work | Depends on | Why this order | Acceptance gate | Rollback |
|---:|---|---|---|---|---|
| **0a** | **Quiet tree/deploy truth**: commit the working-tree fixes individually (tool caps/refusal, Host middleware, launcher, approval preview, persona) after tests; rebuild `dist`; add `/health` build fingerprint. No product behavior change beyond the fixes. | none | Every later measurement is meaningless while source/binary disagree (gap 1/18). | `cargo test --locked` on the quiet tree; `/health` reports the same commit/worktree hash as the build; live hostile-Host probe 403/421 by design. | Revert individual commits; keep old dist for comparison. |
| **0b** | **Snapshot scope v3**: run-scoped capture/filter, history read filter, recovery restore/filter, old-v2 read migration, UI consumes only run nodes. | 0a | History/replay/recovery/UI and all scale arithmetic assume run scope; a scheduler over a global graph would amplify the bug (gap 3/4). | Two-run fixture: run B snapshot/state contains only B task ids; recovery graph has only newest run; state endpoint no foreign ids; UI mounted node count matches run. | Snapshot v2 files remain readable; restore pre-change DB/files backup; feature flag only read-side filter is not enough. |
| **0c** | **P0 security**: Host+token deploy, spawn approval, 402/400/422, context preflight, approval-preview persistence. | 0a | P0 before concurrency; spawn authority before a scheduler can create children. | Targeted tests in §4; live hostile Host 403; no child without approval; zero HTTP over context. | Per-P0 commit revert; token/Host dev env flags; snapshot backup where schema touched. |
| **0d** | **Per-task tool budget + refusal taxonomy**: `task_tool_counters`, per-task 12 turns, per-run 40 calls, separate turn/call/provider/approval refusals or structured detail; fix comments/tests. | 0a/0c | Scheduler parallelizes tasks; a per-run turn counter would starve every task after the first (gap 5/6/7). | Two tasks × 12 turns each; first task cannot consume second's turn budget; each limit returns its own code/detail; approval 9th refusal names approval wait. | Revert; per-run behavior restored (known-bad, feature flag not recommended). |
| **1** | **S0 load harness/baseline**: synthetic provider, current serial engine, record in-flight≤1, latency, DB write, event rates, SQLite version. | 0a–0d | 03/17 demand a baseline artifact before any capacity claim; current 12/40 changes the failure profile. | Harness reproduces ~0.4 calls/s at 20 s synthetic latency; artifacts committed. | n/a (no product). |
| **2** | **S1 admission queue + fair scheduler**: bounded call/task admission, per-hive deficit round-robin, deadlines, scheduler-owned execution claim; integrate parent guard and inline child. | 0b/0c/0d/1 | This is the first change that creates real concurrency; it must not arrive before run-scoped graph and per-task budgets. | S1: 60 pending/25 permits 10 min, in-flight p99=25, wait p99<250 ms, 0 deadline expiries, queue cap named; child-once race test. | `HIVEROUTER_SCHEDULER=off` → sequential dispatch; feature-flag queue only. |
| **3** | **DB writer actor + read pool**, including `SnapshotStore.write` index writes, migration bootstrap, `BEGIN IMMEDIATE`, short read busy timeout, SQLite version gate. | 2 | Parallel tasks expose the single `Mutex<Connection>` and 5 s blocking; actor must be the only writer; snapshot writes are part of the write path (gap 16). | S2: write p99<10 ms at 200 writes/s, ≥20 rows/batch, read-after-write, snapshot build <50 ms, 0 BUSY surfaced, SQLite version gate. | `HIVEROUTER_DB_ACTOR=off` legacy single connection + pre-migration DB backup. |
| **4** | **Event coalescing before the bus** (per run/task 16–33 ms; control direct; flush on terminal/fail/cancel). | 2 | Concurrency multiplies delta events; coalescing must precede sessions/replay and must prove control-sequence ordering (gap 17). | 1,000 fragments → ≤60 flush/s/task; no text loss; one final flush before `REPLY_DONE`; cancel/fail one flush; replay seq intact. | `HIVEROUTER_DELTA_COALESCE=off` direct publish. |
| **5** | **Bounded session outbound queues** (control priority, 256 batches, 2 s send timeout, heartbeat, resync budget). | 4 | Coalesced deltas give the session writer a bounded frame rate; sessions are the last fan-out boundary. | S3 slow-client test: other sessions 0 lag; stalled session resync≤10; control lossless; queue≤256. | `HIVEROUTER_SESSION_QUEUE=off` inline send. |
| **6** | **Budget-lock split + reservation/settlement** (per-run `RunBudget` handle; no mutex held across provider await). | 2/3 | AIMD/permits are useless while one stream holds the budget mutex (06 T26); accounting must be race-safe before leases. | Two 300 ms streams overlap <500 ms; no call after cap; reservation blocks concurrent overspend; missing usage INCOMPLETE. | `HIVEROUTER_BUDGET_RESERVATION=off` legacy lock; revert. |
| **7** | **AIMD lease + two-level cost governor**: per-model vs single-model decision; 429 additive/multiplicative; shared cooldown; run/global caps, observe/soft/hard settings, cost ledger. | 6/3 | Raising permits without AIMD/backpressure and without persistent cost repeats the 429 stampede and denial-of-wallet gaps. | 429 injection: permits halve within one observation, recover; no call after global cap; ledger Decimal/reconcile; 150 soak. | `HIVEROUTER_MODEL_LEASE=off` (8) / `HIVEROUTER_BUDGET_MODE=observe`; DB backup for migration. |
| **8** | **UI LOD/culling + load acceptance**. | 2–7 | Do not hide a bad runtime behind UI polish; only after server bounds hold. | seeded 600-node: ≤120 mounted, ≤50 animated, ≥50 fps, snapshot budgets. | `HIVEROUTER_UI_LOD=off`. |

**Hard ordering rules.** Snapshot scope (0b) before any scheduler; per-task cap (0d) before admission; spawn approval (0c) before model spawn can be scaled; DB actor (3) before S2 load; coalescing (4) before sessions; budget-lock split (6) before AIMD; no cap increase before S0–S4 thresholds; no live deployment claim without the 0a fingerprint.

### 3.2 What 17 must be corrected to say

- **17 C10 admission** and **17 C5 parent guard** need a single **execution claim**: `Scheduler` owns a per-task `JoinHandle`/reservation; `drain_children_of` must await that handle or enqueue and await, never call `execute_task` on a task already owned by the scheduler. Inline model spawn already removes a task from the queue (`execution.rs:2670`), so the claim API must cover both.
- **17 C11 DB actor** must include `SnapshotStore::write` (direct DB index write, `snapshot.rs:75-102`) and `SnapshotStore::read` filter after scope fix; migrations must run before the actor starts and before read-only connections.
- **17 C12 coalescing** must specify terminal flush order and control-lane seq; the current per-call sink (`execution.rs:1764-1795`) makes it possible for `REPLY_DONE` to overtake a buffered delta unless the engine flushes synchronously under the task state lock before `publish`.
- **17 C14 AIMD** must resolve plan/leaf model selection first. Current `ProviderSettings` has one `model` used for planner and leaf (`provider_executor.rs:298,331,416`); a per-model lease is either a single bucket or requires a new `planner_model`/`leaf_model` configuration. Add that decision to the gate.
- **17 C13 sessions** should not carry viewport subscription until the frozen command/event contract is extended; the current 17 text defers it but the S3 acceptance table mentions it (`17:650`).

---

## 4. P0 security patches: exact fixes, tests, rollback

### P0-1 — Host allowlist (and the token/README residual)

**Current state (time-scoped).** The working tree added a middleware at `http.rs:509-594`; the 21:43 live binary lacked it (21:54 hostile Host → 200 token), and the 22:00 rebuild now returns 421 for hostile Host (22:03:54) but still serves the unauth reads. The guard uses `state.config.bind`, returns a custom 421 body without `retryable`, applies to matched routes + fallback (`:533`), and has no test file. It is not a complete T12 fix because no token/Origin gate exists on the HTTP routes.

**Required patch (from the current source, not from scratch).**
1. Keep the middleware but return the standard refusal shape: `refusal_response(StatusCode::FORBIDDEN, &Refusal::OriginRejected)` (or add `retryable` to the current body and update the contract/UI). 06's acceptance expects **403 `E_ORIGIN_REJECTED`**; 421 is not a contract code and the UI parser expects `retryable` (`web/src/api/client.ts` refusal parsing).
2. Resolve the **actual bound port**: production `config.bind` is fixed, so the current code works for 8099; but tests/config with `:0` take the wildcard branch (`http.rs:570-592`). For a real test of production semantics, pass the listener's `local_addr()` into the router (wire `serve` at `http.rs:1111-1122`) and test with the actual port; do not accept `127.0.0.1:anyport` in production.
3. Add token/Origin for the endpoints that remain unauthenticated (live proves 200): `POST /api/agents/profiles/import`, `DELETE /api/agents/profiles/{id}`, `DELETE /api/agents/memory/{key}`, `GET /api/agents/memory`, `GET /api/history/runs*`, `GET /api/agents/runs/{id}/messages` (`http.rs:511-527`). Use `Authorization: Bearer <session token>` or a documented `X-HiveMind-Token`; reject missing/wrong with 401 `E_TOKEN_INVALID`. The web client already holds the token (`web/src/api/client.ts:290-306`); add an auth fetch helper and update the panels. This is the T12 half that a Host-only fix leaves open.
4. `provider_origin_allowed` currently treats absent Origin as allowed (`http.rs:987-996`, `None => true` at `:989`). Require an explicit allowed Origin or the token for `/api/provider/*`.
5. Log one line per rejection: code, path, host/origin, **never** the token/body (`06` L1). Do not log the full `Host` if it can contain secret query data (it cannot, but keep fixed fields).
6. Deployment gate: after rebuild, repeat the live probe; hostile Host must not return a token; normal loopback must; unauth reads/mutations must 401. Record binary/source hashes.

**Tests (new `core/tests/http_origin.rs`, raw TCP helper like `http_boot.rs:69`):**
- `rebound_host_header_is_rejected`: `GET /api/session`, `Host: rebind.evil.example:<port>` → 403 `E_ORIGIN_REJECTED`, body has no token, `retryable` present.
- `loopback_hosts_still_work`: `127.0.0.1:<port>`, `localhost:<port>` → 200.
- `missing_host_is_rejected` (raw HTTP/1.0 or no Host) → 403/421 as pinned.
- `mutating_endpoints_require_token_and_origin`: each route no token/foreign Origin/foreign Host → 401/403, DB row counts unchanged; with valid token+Host → success.
- `read_endpoints_require_token`: memory/history/agent-message/profile reads require token (per the selected policy); no data bytes in rejection.
- `provider_proxy_requires_origin_or_token`.
- `host_guard_uses_the_actual_bound_port` (not only 0 wildcard).
- `cargo test -p hiverouter-core --test http_origin --test http_boot --test ws_transport`.

**Rollback.** `HIVEROUTER_HOST_ALLOWLIST=off` and `HIVEROUTER_HTTP_TOKEN=off` for development only, with loud warnings; production default on. A Host middleware revert alone is acceptable only if the token gate remains; otherwise do not ship.

### P0-2 — Spawn approval (still bypassed)

**Current state.** `execution.rs:1684-1691` handles `spawn_agent` before the approval branch; no source change. The 8-wait cap (`tool_loop.rs:21`) and the new 12-turn cap share `E_CALL_LIMIT` (`execution.rs:2023-2033`).

**Required patch.**
1. Move `spawn_agent` into the approval path: after `authorize_tool_call`/plan validation and before `execute_spawn_agent`, call `suspend_for_approval` with the `ToolCallPlan`. No child row, no `NODE_SPAWN`, no provider call before the decision.
2. Approval args must include the delegation target/limits: `role`, bounded `instruction` preview/hash, current `fanout/depth/agents`, child prompt hash. `args_sha256` already binds exact args; the preview uses `redact_args_preview`. Reject role strings outside the allowed spawn vocabulary before approval so the operator does not approve an impossible child.
3. On approve: run `execute_spawn_agent` exactly once; replay cannot spawn twice (approval CAS). On deny/timeout: return a refused `TOOL_RESULT` with `E_TOOL_NOT_ALLOWED`/`E_APPROVAL_TIMEOUT`; no child.
4. Resolve the 8-wait capacity: either raise/task-scope `MAX_APPROVAL_WAITS_PER_RUN` (per-task/hive, not per-run), or use the **delegation-budget grant** (one approval allows N spawns in one parent turn, audited) for scale. Do not silently exempt spawn.
5. Update the `APPROVAL_EXEMPT` policy to empty for spawn; add `approval_coverage` static test; add audit event with role/depth/fanout, never instruction body.
6. Operator `SPAWN_SUB_HIVE` remains authorized (operator command); model `spawn_agent` is approval-gated by default. Document that distinction in `UI-COMMANDS-v1`.

**Tests.** `approval_coverage.rs::every_approval_tool_is_gated_or_declared_exempt`; `engine_spawn_approval.rs` no-approval/deny/approve/replay; a 9th approval-wait test proving its own code/detail; multi-task approval fairness. **Rollback:** revert only; no release flag that reopens the bypass.

### P0-3 — 401/402/400/422 provider mapping

**Current state.** `client.rs:500-524`: `401 | 402` → `E_PROVIDER_AUTH`; `_` → `E_PROVIDER_INVALID_RESPONSE`. `error_matrix.rs:161-180` asserts the wrong 402 behavior. No balance freeze. `http.rs:937-945` status map has no 402/400/422.

**Required patch.**
- 401 → `ProviderAuth`; 402 → new `E_PROVIDER_BALANCE` (non-retryable; detail "check balance, do not rotate key"); 400/422 → new `E_PROVIDER_REQUEST_REJECTED` (non-retryable; status only, no body/parameter secrets); 429 unchanged; 5xx transient; 501/505 invalid.
- Add a process/client `AccountState` freeze on 402 so subsequent preflights refuse **before HTTP** with `E_PROVIDER_BALANCE` until the operator checks balance/clears; expose boolean/code in `/health`/`/diagnose`.
- HTTP map: balance → 402, request rejected → 400/422, auth → 401.
- Contract delta only with `CHANGE-LEDGER-IMPL.md` rows and `ui_command_matrix`/`golden_schemas`; UI `known codes`, strings, `refusal_matrix`.
- Tests: update `error_matrix.rs` 402 test; add 400/422; 401; 429; freeze-before-HTTP (wiremock request count unchanged); every code's `retryable=false`; no body echo.

**Rollback.** Revert mapping+contract together; additive variants may remain unused. Do not map 402 to `E_BUDGET_EXCEEDED` as a permanent answer (06's interim is explicitly inferior); if contract freeze blocks new codes, use a detail field and schedule the code as a blocker, not a silent lie.

### P0-4 — Context-window preflight

**Current state.** Metadata exists (`dto.rs:892-907`: optional `context_window`, `max_output_tokens`), but no consumer; estimator is bytes/4 (`provider_executor.rs:436-442`); `ProviderSettings` has no window/estimator fields (`config.rs:40-70`). No test.

**Required patch.**
1. `ProviderSettings` adds `context_window` (default documented 1,048,576; env override for tests) and a safety margin (1,024 tokens/2%). A background model catalogue may refresh from `client.list_models()` (`client.rs:266-274`); if absent, documented default + log; never block boot.
2. Add a pure `preflight_context(request, estimated_prompt, max_output, window)` used by `plan`, `execute`, `continue_with_tools`, before the budget lock/HTTP. Refuse with `E_CONTEXT_WINDOW_EXCEEDED` when estimate + max output + safety > window.
3. Replace/ augment the estimator with a **conservative** one (`ceil(json_bytes/3)+64`) documented and env-tunable; keep the provider's reported usage as authoritative for accounting. Do not claim a tokenizer accuracy.
4. Separate from the run token budget; a context refusal is not a run-budget refusal.
5. Tests: zero HTTP above window; exact-fit sent; CJK fixture conservative; env/metadata selection; plan/leaf/continuation/inline child all covered.

**Rollback.** `HIVEROUTER_CONTEXT_WINDOW=0` disables the check (dev); default on; estimator change is a code revert.

### P0-5 — Snapshot scope (new P0; 17/09 did not have it)

**Current state.** `capture` uses the global graph (`snapshot.rs:258-271`); `TaskNode` has no `run_id` (`graph.rs:18-27`); `HistoryReadModel::state_at` returns the envelope unfiltered (`history.rs:109-115`); recovery restores the global image and publishes graph-only nodes under the restored run (`recovery.rs:149-177,232-257`); UI decodes every node (`store.ts:571-622`); live snapshots grow 14→35.

**Required patch (smallest durable design).**
1. Add run ownership in the graph authority without changing the wire `TaskNode`: `GraphState.run_of_task: HashMap<TaskId, RunId>`; change `GraphAuthority::add_task(task_id, parent, run_id)` and all call sites (`execution.rs` root/planned/spawn/template, `ws.rs` template insert, `recovery.rs`, tests). This avoids a frozen-contract field addition while making filtering exact.
2. Add `GraphAuthority::snapshot_for_run(run_id)` that filters nodes by `run_of_task`, edges where both endpoints are in the run, and toposort in-run. `snapshot::capture` calls it, not `graph.snapshot()`.
3. Snapshot schema **v3**: write only run-scoped graph; `SnapshotEnvelope` remains run-identified. Keep `v2` read compatibility: `SnapshotStore::read` migrates v2 by filtering the envelope graph through the run's task ids from `RunStore::tasks_for_run(run_id)` (the DB is the authority; a task id not belonging to the run is dropped, edges/toposort rebuilt). `v3` files are read as-is but validated.
4. `restore_from_snapshot(snapshot, run_id)` must reject/ignore nodes not owned by `run_id` (or restore only after filtering). `recover_on_boot` restores only the newest loadable run image; the live graph is one run image, not the union. The graph-only settlement/publish path must use only nodes owned by `restored_run`; foreign nodes cannot produce events under that run.
5. `HistoryReadModel::state_at` filters the envelope before returning (defense in depth) using the same run-owning set; `history_http_api` and UI see only run nodes. This also closes the live 35-node/9-root replay lie.
6. `SnapshotStore::write` index write remains the only DB write outside the later DB actor; note it for §3 step 3. Old v2 files remain at rest (same-user boundary); document that scope is enforced on read/write, not retroactively rewritten; a migration rewrite is optional and separate.
7. UI validation can additionally reject a node whose `task_id` is not in the run's task set if the server ever sends one; but the server-side filter is the authority.

**Tests (new `core/tests/snapshot_scope.rs`; extend `snapshot_history.rs`):**
- `capture_contains_only_its_run`: graph with run A root/child, run B root; capture A → 2 nodes, A ids only; capture B → 1 node.
- `history_state_filters_foreign_nodes`: write a v2 global envelope for run A; `state_at(A)` returns only A ids; edges/toposort consistent.
- `recovery_restores_only_newest_run`: two run dirs; restart; graph node count equals newest run's tasks; no foreign nodes; no foreign task event.
- `v2_migration_is_read_compatible`: v2 global file → v3 read filter; no panic; `E_SNAPSHOT_CORRUPT` unchanged for malformed.
- `two_run_snapshot_api`: live-equivalent two runs; `/api/history/runs/A/state` nodes == A task ids; roots == A's roots.
- `snapshot_scope_ui` (optional e2e): replay shows only run tasks; no foreign root.
- `cargo test -p hiverouter-core --test snapshot_scope --test snapshot_history --test kill_restore --test recovery_gates`.

**Rollback/migration.** Write v3; old binary cannot read v3, so rollback requires a pre-change backup and binary rollback. Keep read support for v2 in the new binary; migration is forward-only. No product data deletion; old foreign node files are not rewritten in the P0 patch.

### P0-6 — Approval-preview durability (current-fix audit)

The task's "approval args preview" fix is web-visible only; the backend replay path deliberately drops it. Either:
- **Persist** a bounded redacted preview with the approval row (new additive column in the tools approval table/migration; `args_preview` max 512 UTF-16, secret-redacted at insert), and attach it in `approval_request_event`/`pending_approval_events*`; or
- **Do not show a preview** after reconnect; mark it unavailable in UI and rely on the digest, with an explicit operator note. Do not silently display `UNKNOWN` as if it were evidence.
Recommended: persist the preview with the same redaction function (`event.rs:370-414`), then replay it; test live request + reconnect + restart; verify no secret canary and hash still binds. This is P1 but blocks approval truth; it is a direct gap in the current working fix.

---

## 5. Tests/rollback matrix for the five required P0 patches (condensed)

| Patch | Primary seam | New/updated tests (offline) | Rollback |
|---|---|---|---|
| Host allowlist + token | `http.rs:509-594,1080-1122`; web fetch | `http_origin.rs` (all routes, hostile/absent Host, no token, actual port, provider proxies) | env flags dev-only or revert; token remains on if Host reverted. |
| Spawn approval | `execution.rs:1682-1736`; `tool.rs`; approval bridge | `approval_coverage.rs`, `engine_spawn_approval.rs`, `approval_ledger.rs`, 9th wait test | revert only; no release bypass. |
| 402/400/422 | `client.rs:492-524`; `http.rs:937-945`; `refusal.rs`; ledger | `error_matrix.rs` 401/402/400/422/429; freeze-before-HTTP; contract matrix | revert mapping+contract; codes additive. |
| Context preflight | `provider_executor.rs:94-151,291-429,436-442`; `config.rs` | `context_window.rs` (zero HTTP, exact fit, CJK, metadata/env, continuation) | `HIVEROUTER_CONTEXT_WINDOW=0`; revert estimator. |
| Snapshot scope | `snapshot.rs:254-272`; `graph.rs:18-27,73-107,220-327`; `history.rs:109-115`; `recovery.rs:149-177,232-302`; `store.ts:571-622` | `snapshot_scope.rs`, extended `snapshot_history.rs`, `kill_restore.rs`, UI two-run | v2 read compatibility; backup/restore for v3; no in-place destructive migration. |
| Approval preview durability (companion) | `tools_bridge.rs:439-518`, approval table migration; `ws.rs:580` | `approval_preview.rs` live/replay/restart + secret canary | additive column; read none if rollback. |

**No test above contacts the provider or reads a credential.** All use wiremock or in-memory/tempfile. The existing test helpers are `http_boot.rs:69` raw TCP, `prompt_assets_wiring.rs:223`, `snapshot_history.rs:133`; new tests should use the same pattern.

---

## 6. What must not change until the migration/load gates pass (delta from 17)

17's list is directionally correct; 26 adds/strengthens:

1. **Do not deploy a scheduler onto the global graph.** Snapshot scope (P0-5) must land and be proven with two runs before admission parallelizes tasks.
2. **Do not build a per-model AIMD until planner/leaf model selection exists**; otherwise a single model lease is the only truthful implementation (decision Q16).
3. **Do not ship the DB actor without snapshot index writes and migration ownership**; all writes need one owner or the actor is not the writer.
4. **Do not enable event coalescing before proving control-event order / terminal flush**; a coalesced delta must never overtake `REPLY_DONE`/`RUN_STATUS`.
5. **Do not treat the working-tree fixes as committed or tested.** C0 must land them with tests and a rebuilt dist; no live claim before the build fingerprint.
6. **Do not claim Host security from source presence.** The live 200-token probe is the counterexample; deploy-then-probe is the gate.
7. **Do not raise `MAX_AGENTS`/`IN_FLIGHT_LIMIT`/tool caps before per-task accounting, spawn approval, snapshot scope, and the S0–S4 gates.** Current 12/40 caps are still per-run; raising them multiplies the defect.
8. **Do not add approval waits after spawn approval without task/hive scoping**; 8 per run is already too small once each spawn needs a decision.
9. **Do not write a v3 snapshot migration without a backup/restore test**; snapshot files are the only durable history image.
10. **Do not change the frozen contract beyond ledgered P0 additions** (turn/call/provider/approval codes, snapshot schema is not the event contract but still needs a schema/migration note).
11. **Do not open web-content ingestion/Search/Fetch** until Host+token, spawn approval, context preflight, and tool-result fencing are deployed and probe-verified.
12. **Do not remove or replace the live binary during this audit**; deployment/rebuild is a Lead gate, not a research-seat action.

---

## 7. Twenty new unanswered questions (options and impact)

These are deliberately different from 10's 20 questions (`10:589-782`), 17's 20 (`17:604-632`), and 09's 20. They are the questions this audit cannot answer from code/live evidence.

| # | Question | Options | Impact / why it changes implementation |
|---:|---|---|---|
| Q1 | **How is deployment truth proven?** | (a) `/health` build hash + worktree hash; (b) signed manifest in dist; (c) only version/schema; (d) manual mtime check | Without this, every "fixed" claim is ambiguous (live Host 200 while source has middleware). Recommend (a). |
| Q2 | **What is the acceptance status-code for a foreign Host?** | (a) 403 `E_ORIGIN_REJECTED` standard body; (b) 421 `MISDIRECTED_REQUEST` custom body; (c) 400 | 06's acceptance expects 403; the current implementation returns 421 without `retryable`. UI parser/QA depend on it. |
| Q3 | **Is the Host guard's port-0 wildcard acceptable in tests?** | (a) pass actual listener port into router and test exact authority; (b) keep wildcard for tests; (c) both | Production/test divergence can hide a production allowlist bug; recommend (a) plus a unit test. |
| Q4 | **Which HTTP endpoints require the session token?** | (a) all `/api/*`; (b) mutations + read-sensitive endpoints; (c) none beyond Host; (d) per-route policy | Live proves unauthenticated memory/history/messages. Token scope determines web changes and T12 closure. |
| Q5 | **Should approval args preview be persisted at rest?** | (a) yes, redacted/bounded; (b) no, mark unavailable after replay; (c) persist only shell/path metadata | Current replay loses it; operator approves without evidence. Persisting adds PII at rest and a migration. |
| Q6 | **What is the turn-budget unit?** | (a) per task; (b) per run; (c) per task with a run safety cap; (d) per hive | Current constant says per task, counters are per run; already broken for multi-task. Recommend (c). |
| Q7 | **How should tool/call/provider/approval limits be named?** | (a) one `E_CALL_LIMIT` + structured detail; (b) separate codes per limit; (c) codes per budget class; (d) no new codes | 17/10 diverge; UI and operator need attribution; contract additions need ledger. |
| Q8 | **What is the desired behavior when a task exhausts its turn budget?** | (a) fail task and continue run; (b) fail run; (c) requeue with new budget; (d) operator approval to extend | Current fails the task and run; a 12-turn research task may be legitimate; task vs run failure changes UX and recovery. |
| Q9 | **What is the approval-wait budget scope and size?** | (a) 8 per task; (b) 8 per run; (c) per hive; (d) grant-based delegation budget | Spawn approval per-child + 8 waits can stall a branch; 12 turns makes the mismatch worse. |
| Q10 | **How does a portable launcher pick a workspace root?** | (a) per-run picker; (b) per-project stored root; (c) env/default with visible path; (d) package directory | The live fix only works on this dev machine; portable fallback repeats the wrong-workspace run. |
| Q11 | **How are launcher child arguments transported?** | (a) `ArgumentList`; (b) escaped quoted string; (c) config file | A quote/backslash in `HIVEMIND_WORKSPACE_ROOT` can inject/alter args; correctness/security. |
| Q12 | **Should snapshot v3 add `run_id` to nodes or keep a graph-side run map?** | (a) graph map + filtered snapshots; (b) `TaskNode.run_id` + wire addition; (c) per-run graph store; (d) read-only filter only | Scope fix must be durable and backward compatible; 10:D1 recommends (a)/(b) but does not choose. |
| Q13 | **What happens to existing v2 snapshots that already contain foreign nodes?** | (a) filter on read; (b) rewrite v3 in place; (c) delete/retain with audit; (d) mark legacy | Live already has 14→35 files; filtering on read hides but does not purge; disk/ops policy needed. |
| Q14 | **How should boot recovery choose the graph image?** | (a) newest run only; (b) active run only; (c) union for history but not live; (d) refuse if multiple active | Current restores a union/global image; live graph_nodes 35; wrong-run events possible. |
| Q15 | **Can a single configured model support per-model AIMD?** | (a) add planner/leaf model settings; (b) single lease; (c) model per task role; (d) per-request model from planner | 17's AIMD design is unimplementable as written; scale stage acceptance depends on it. |
| Q16 | **What is the admission unit and ownership?** | (a) per-call ticket with scheduler-owned task handles; (b) per-task slot; (c) per-run; (d) hybrid | 17's C10+C5 can double-execute a child; ownership determines one reply/one result. |
| Q17 | **Should the DB actor own snapshot index writes?** | (a) yes, all writes; (b) snapshots outside actor with own mutex; (c) async writer for snapshots | `SnapshotStore::write` does direct DB writes; actor split must decide; otherwise one writer invariant is false. |
| Q18 | **What exactly does event coalescing preserve on terminal/cancel/fail?** | (a) flush all pending then control; (b) drop pending; (c) flush with truncation; (d) coalesce into REPLY_DONE | Text loss/ordering/terminal truth; 17 only says "flush before done" without a state-machine test. |
| Q19 | **When does the live app get rebuilt, and who owns restart?** | (a) Lead after P0 commit; (b) every merge; (c) user-driven; (d) no rebuild until S5 | Source/binary skew invalidates research and security claims; task says no restart by this seat. |
| Q20 | **What artifact proves the next scale stage?** | (a) S0 harness report; (b) S1 25-agent soak; (c) S2/S3 then S4 150; (d) all before any claim | Operator says never claim higher than the stage proves; 03/09 have stages but no committed artifact. |

---

## 8. Evidence register, non-claims, and handoff

### 8.1 Claims (material only; full list in body citations)

| Claim | Evidence | Status |
|---|---|---|
| Live app accepted a hostile `Host` and returned a token at 21:54:59 | `curl -H 'Host: rebind.evil.example:8099' /api/session` → 200, token present at 21:54:59 | `VERIFIED (live, first build)` |
| Live app rejected a hostile Host after the 22:00 rebuild | same probe at 22:03:54 → 421 `E_ORIGIN_REJECTED`, no retryable, no token | `VERIFIED (live, second build)` |
| Unauth reads still 200 after the Host rebuild | `/api/agents/memory`, `/api/history/runs`, `/api/agents/profiles`, `/api/session` at 22:03:54 | `VERIFIED (live)` |
| First live binary predated Host middleware; late binary predates `client.rs` | first binary 21:43:54 vs `http.rs` 21:52:53; second binary 22:00:18 vs `client.rs` 22:01:00 | `VERIFIED (mtime)` |
| Snapshot files are global-graph snapshots | `snapshot.rs:258-271`; live 14/20/26/31/35 nodes, 5–9 roots | `VERIFIED (live + source)` |
| Tool turn cap is per run | `execution.rs:1590-1595`; `db.rs:652-705`; `tool_loop.rs:13` | `VERIFIED (source)` |
| E_CALL_LIMIT covers 4 producers | code points above | `VERIFIED (source)` |
| Approval preview missing on replay | `tools_bridge.rs:502-518`; `pending_approval_events_all`; `ws.rs:580` | `VERIFIED (source)` |
| Spawn approval still bypassed | `execution.rs:1684-1691` | `VERIFIED (source)` |
| 402/400/422 codes still wrong; late edit is message-only | `client.rs:500-524`; 22:01 diff; `error_matrix.rs:161-180` | `VERIFIED (source)` |
| Context preflight absent | `provider_executor.rs:436-442`; no `context_window` consumer | `VERIFIED (source negative)` |
| Launcher hard-coded root / string args | `HiveMindLauncher.cs:17,57-69,163-178` | `VERIFIED (source)` |
| One model used for plan/leaf | `provider_executor.rs:298,331,416`; `config.rs:40-70` | `VERIFIED (source)` |
| 17 snapshot scope absent / P0 conflict | `17:446-551` vs `10:35,326,522`; task brief | `READ` |
| 17 DB actor omits snapshot index write | `snapshot.rs:75-102` vs `17:255-287` | `VERIFIED (source/design)` |
| 17 scheduler+parent guard double-execution risk | `17:315-350,571`; inline claim `execution.rs:2670` | `INFERRED` from design/source |
| Live app went down and was rebuilt mid-audit | `/health` 000 at 21:58:45; binary 22:00:18; restart 22:00:32; health 200 at 22:03:31 | `VERIFIED (live)` |
| Any runtime behavior of proposed fixes | no build/test/restart by this seat; app was rebuilt by another lane | `UNVERIFIED` |

### 8.2 Non-claims

This audit does not claim: that any fix is committed, tested, or that the deployed binary corresponds to a tested commit; that the Host fix was live for the whole audit (it was absent at 21:54:59 and present only after the concurrent 22:00 rebuild — see §1.1a); that the 12/40 caps are correct or tested; that a spawn approval exists; that 402/400/422 codes or context preflight are implemented; that snapshot scope is fixed; that the launcher works on a portable machine; that 17's AIMD/DB actor/coalescing design is executable as written; that the live app is stable or that any post-22:00 probe is repeatable; that a 150/600 claim is supported; that the live binary matches any source commit.

### 8.3 Handoff to the Lead

1. **Do not accept 17 as an implementation plan until C0/snapshot/tool-accounting corrections are incorporated.** 17's structure is usable, but its P0 set and three technical premises are wrong or stale.
2. **Gate 1: quiet tree + rebuilt dist + `/health` fingerprint.** The live 200-token Host probe proves source presence is not deployment.
3. **Gate 2: snapshot scope v3 + recovery filter + two-run history test.** Before any scheduler/concurrency.
4. **Gate 3: per-task turn/call accounting + spawn approval + approval-preview durability.** Before scale.
5. **Gate 4: provider mapping/context preflight.** Before cost/AIMD claims.
6. **Gate 5: S0 harness then admission → DB actor → coalescing → sessions → budget split → AIMD → cost governor → UI LOD** (corrections in §3).
7. **Resolve Q1–Q20 as product/security decisions; Q5/Q6/Q7/Q9/Q15/Q16/Q17 are blockers for the respective commits.**
8. This seat wrote only `docs/research/hmars/26-scale-security-audit.md`; no product code, tests, config, server, credentials or paid call were touched.

---

## 9. Appendix — live probe transcript (timestamped, read-only)

```text
21:53:00 CST  HEAD=bd575ba; dirty product files as listed in §1.1
21:53        process: hiverouter.exe --bind 127.0.0.1:8099 \
             --data-dir C:\Users\Riley\AppData\Local\HiveMind \
             --workspace-root C:\Users\Riley\Desktop\hiverouter-v1
21:54:59     GET /health -> status ok, version 1.2.0-dev, db schema 10,
             last_seq 0, graph_nodes 35, origins 2
21:54:59     GET /diagnose -> side_effects_enabled true; (no workspace field)
21:54:59     GET /api/session (Host 127.0.0.1:8099) -> 200, token present (not printed)
21:54:59     GET /api/session (Host rebind.evil.example:8099) -> 200, token present
21:54:59     GET /api/agents/memory -> 200 (7 profiles)
21:54:59     GET /api/history/runs -> 200 (5 runs)
21:54:59     GET /api/projects -> 200 (1)
21:54:59     GET /api/agents/profiles -> 200 (7)
21:54:59     GET /api/history/runs/<id>/messages -> 200 (4)
21:54:59     per-run history latest: 14/20/26/31/35 nodes; 5/6/7/8/9 roots
21:58:45     GET /health -> connection failure; no hiverouter/HiveMind process
22:00:18     dist\HiveMind\hiverouter.exe rebuilt (15,127,040 B)
22:00:22     dist\HiveMind\HiveMind.exe rebuilt (9,216 B)
22:00:32     hiverouter.exe restarted (PID 16616), same 8099/data-dir/workspace-root
22:01:00     client.rs modified after the binary (402/400/422 messages only)
22:03:31     GET /health -> 200, last_seq 12, graph_nodes 35
22:03:54     GET /api/session (Host rebind.evil.example:8099) -> 421
             body code=E_ORIGIN_REJECTED, retryable_present=False, no token
22:03:54     GET /api/agents/memory, /api/history/runs, /api/agents/profiles,
             /api/session -> 200 without a token
```

No POST/DELETE/approval/provider call was made. No token, key, memory value, message body, or prompt text is quoted in this report.

**Bottom line:** the working tree contains real fixes (tool caps/refusal, Host middleware, launcher root, approval preview), but the Host gate was absent for most of the audit and only appeared through a late unverified rebuild; the tool-cap fix still uses per-run accounting; snapshot scope is a P0 live defect; spawn approval/402-400-422 codes/context preflight remain unimplemented; the HTTP token gate is absent; and 17's scheduler/AIMD/DB-actor design has three implementation-blocking contradictions. Fix the P0s, commit/test/rebuild with a fingerprint, then implement the corrected order in §3.
