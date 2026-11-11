# HiveMind Finish Plan v3 — execution status (2026-11-10)

> **Revision 3.1 note (2026-11-11):** capability expansion is now the **mandatory first work**
> in [HIVEMIND-FINISH-PLAN-v3.md](HIVEMIND-FINISH-PLAN-v3.md) §0, with detailed reference in
> [HIVEMIND-CAPABILITY-EXPANSION-PLAN.md](HIVEMIND-CAPABILITY-EXPANSION-PLAN.md). This file
> remains the historical/current v3 crosswalk; the capability plan does not create a Code mode.

**Companion to:** [HIVEMIND-FINISH-PLAN-v3.md](HIVEMIND-FINISH-PLAN-v3.md) (the plan is not rewritten here).
**Observed HEAD:** `1129349`; **final docs HEAD:** `44e0535`; **installed build:** `806251b`; **working tree:** product code clean (the ledger endpoint candidate was discarded).
**Verdict:** the P0/conformance core is materially shipped and tested; the plan is **not complete**.
**Truth rule:** a line is struck through only when the required behavior exists **and** a committed test or receipt proves it. "Unshipped" and "read-only review" are not marked done.

---

## 1. Strikethrough — completed / proven

Legend: ~~struck~~ = proven; evidence follows. Partial items are in §2.

### Security and approval

- ~~**WP-SEC-1 (partial core): `cmd.exe` removed from the default shell allowlist; interpreter hosts refused even if explicitly allowed.**~~ Evidence: `backend/crates/hiverouter-tools/src/shell.rs:54-99,169-185`; tests in `shell.rs`/`suspension.rs`.
- ~~**WP-SEC-1 (core): the tool jail denies the server data directory and the approval-ledger directory; the engine jail matches.**~~ Evidence: `backend/crates/hiverouter-core/src/tools_bridge.rs:94-107`; `http.rs:347-351`; full backend battery PASS.
- ~~**WP-SEC-1 (core): daemon child stdio/env is sanitized so `DEEPSEEK_API_KEY` is not inherited.**~~ Evidence: `daemon.rs:444,827-845`; leak test asserts `CLEAN`.
- ~~**WP-SEC-2 (core): `allow_once` approval requires an unexpired TTL and binds `approval_id`.**~~ Evidence: `approval.rs:10-14,108,117`.
- ~~**WP-SEC-2 (concurrency): one per-run approval suspension at a time; two queued approvals each execute exactly one `file_write`; both files and two `tool_turns` rows exist.**~~ Evidence: `backend/crates/hiverouter-core/tests/suspension.rs` `concurrent_tasks_do_not_double_suspend_the_run`; commits `93e40fb`, `195fa1c`; suspension suite 17/17.
- ~~**WP-SEC-2 (regression): duplicate/concurrent resolves have one winner and one execution.**~~ Evidence: `suspension.rs`; full battery PASS.

### Schema, root instructions, and root review

- ~~**WP-SCHEMA-1: v24 adds `tasks.instruction TEXT` with a down migration; schema roundtrip/current-version tests are updated.**~~ Evidence: `db.rs:859-865`; migration tests; installed server reports `schema_version: 24` (`docs/evidence/live/install-1.2.0-dev.json`).
- ~~**WP-GOV-1 (core): child task prompts include the task instruction; root schemas advertise `spawn_superagent {objective, persona?}` and `assign_task {task_id, objective}`; `ROOT_CONTROL_SYSTEM` is updated.**~~ Evidence: `execution.rs:4190-4196`; `provider_executor.rs:40-46,592-608`; run lifecycle tests.
- ~~**WP-GOV-2 (partial): one bounded, read-only post-dispatch monitor review runs before final synthesis; its text is labelled data, not authority.**~~ Evidence: `execution.rs:2342-2398`; `provider_executor.rs:48-53,709-738`; `run_lifecycle.rs` `post_dispatch_monitor_review_is_one_bounded_validated_turn`; commit `806251b`.
- ~~**EVO-GATE (partial but real): lesson apply requires an operator-placed held-out receipt; boot rejects receipt-less lesson lines; oversized/weak receipts are refused.**~~ Evidence: `evolution_gate.rs`; `http.rs` apply call; `proposal_authorization.rs`; `execution.rs` `load_authorized_lessons`; commit `f7c8463`/`92daed7`; full battery PASS.
- ~~**Proposal evidence shape: `expected_effect`, `verification_plan`, and `rollback` are accepted and persisted.**~~ Evidence: `provider_executor.rs` schema; `execution.rs` payload; `run_lifecycle.rs`; commit `545d676`.

### UI, docs, install, and evidence

- ~~**WP-UI-1 (mostly): 8-template library gallery, snapshot banner, approval queue, tool attribution, auth-header consistency, mock-timer/version fixes, 1440 + 375 screenshots.**~~ Evidence: `web/src/panels/LibraryPanel.tsx`; `web/src/state/templateCatalog.ts`; `web/tests/templateGallery.test.ts`; [1440 screenshot](../evidence/live/install-ui-1440x900.png), [375 screenshot](../evidence/live/install-ui-375x812.png); web 265 tests; typecheck/build PASS.
- ~~**WP-INST-1 (core): reproducible package build, per-user install, Desktop/Start Menu shortcuts, DPAPI provider key path, uninstall, BUILD.json identity.**~~ Evidence: `tools/build-desktop.ps1`; `tools/install-desktop.ps1`; `tools/uninstall-desktop.ps1`; current install receipt.
- ~~**WP-TEST-1 (partial): full offline battery wired with SKIPPED≠PASS semantics; docs checker; QA tests.**~~ Evidence: `tools/qa/run-full-battery.ps1`; `tools/qa/check-docs-truth.mjs`; current battery receipt `docs/evidence/live/full-battery-20261110-232942.json` (8/8 PASS; **dirty tree**, see caveat).
- ~~**Evidence drills: real process kill/restart receipt and 41-task scale receipt.**~~ Evidence: `docs/evidence/live/crash-drill-20261110-230629.json`; `docs/evidence/live/scale-drill-20261110-230653.json`; scripts `tools/qa/run-crash-drill.ps1`, `run-scale-receipt.ps1`.
- ~~**Docs truth gate: 19/19 checks, 0 violations; README/tool-surface/count corrections.**~~ Evidence: `node tools/qa/check-docs-truth.mjs` (run 2026-11-10); README/audit corrections.

---

## 2. Partial — implemented but not at plan acceptance

| Plan item | What exists | What is missing |
|---|---|---|
| WP-SEC-1 db-path containment | data_dir + ledger denied | `db_path/+wal/+shm` are not passed to `ToolsBridge`; custom `--db` outside data root is not denied (`tools_bridge.rs:104`) |
| WP-SEC-2 approval integrity | TTL, binding, single suspension, one-write test | session approvals are **by design** non-expiring (one-shot TTL 120s; scoped by run/tool/args); a bounded waiter queue is not implemented; all `verify_grant` paths not proven |
| WP-SEC-3 root tool truth | root schemas updated; two-turn control | **superseded by v3.1 §0/§4:** keep all 35 grants; do not adopt the 35→28 removal; only root schema/audit tests remain |
| WP-GOV-2 monitor | one read-only review turn | **no `send_instruction`, `cancel_task`, or repair turn**; tool calls in review fail the run (`execution.rs:2384-2387`) |
| WP-GOV-3 caps/settlement | 41-task probes; crash settle | no live 40-node provider run; no atomic provider-call reservation proof; structural settlement remains a sequential loop |
| WP-HARDEN-1 | `/api` fall-through fixed; interpreter policy | history 1970 bug (`history.rs:79`), handshake dead API, planner fan-out validation, `build.rs` extraction |
| WP-UI-2 / MISSION-1 | library gallery exists | HivePulse, ToolsMatrix, FirstRunNotice, MissionGallery absent; no gallery launch e2e |
| WP-API-1 | WS/UI command surface | `docs/contracts/HTTP-API-v1.md` absent; CLI launch-token source absent |
| WP-DOCS-1 | checker PASS; README corrected | `docs/evidence/live/INDEX.md` and `docs/DOWNLOADS.md` absent; some CODEMAP/release banners stale |
| WP-TEST-1 strict | battery, QA, docs | e2e guard still fails open (`e2e/run-guarded.mjs:16-35`); battery receipt is dirty-tree; strict `-Release` remains PRE-RELEASE exit 3 |
| WP-INST-1 acceptance | default install, shortcuts, receipt | clean-profile `-InstallRoot/-DataRoot` proof absent; health is degraded without a key |

---

## 3. Not done — required by the plan

- [ ] Monitor **actions**: `send_instruction`, `cancel_task`, bounded repair after a failed dispatch.
- [ ] Independent held-out evaluator; EVO-GATE receipt is shape-checked, not produced/verified by an evaluator.
- [ ] EVIDENCE-LEDGER target-class counters (the `/api/evidence/ledger` candidate was discarded: no UI consumer, no route-contract test; not installed).
- [ ] Active-run crash drill (kill while `Running`/`AwaitingApproval`, boot recovery assertion).
- [ ] Live keyed provider run; no key is configured; all current live prompts refuse `E_KEY_MISSING`.
- [ ] 40-node live scale receipt; current scale probes are 41 tasks in the test harness.
- [ ] `docs/contracts/HTTP-API-v1.md`; CLI launch-token source.
- [ ] HivePulse/ToolsMatrix/FirstRunNotice/MissionGallery.
- [ ] Clean-profile install proof and strict release seal.
- [ ] UI browser e2e on the installed launcher window; screenshots are release-server UI, not the Edge app window.
- [ ] Independent evaluator pass / final release gate.

---

## 4. Archived 40-round table status (pre-3.1; superseded by v3.1 §5, 52 rounds)

The table below is retained as history. It refers to the archived pre-3.1 plan §3, not the current
52-round capability-first schedule.

| Rounds | Status | Note |
|---|---|---|
| R1–R2 | **DONE** | checker/QA; baseline |
| R3–R5 | **MOSTLY** | SEC-1/2 shipped; db-path sub-item open |
| R6 | **DONE** | schema v24 |
| R7–R9 | **DONE** | root instruction/objective |
| R10–R11 | **PARTIAL** | read-only review, not actions |
| R12–R13 | **PARTIAL/UNVERIFIED** | scale probes, no 40-node provider run |
| R14–R16 | **UNVERIFIED** | caps/settlement/accounting |
| R17–R19 | **PARTIAL** | API/hardening |
| R20 | **NOT DONE** | API truth |
| R21–R23 | **MOSTLY** | UI repair; 375 assertion pending |
| R24–R27 | **MOSTLY** | install; clean profile pending |
| R28–R30 | **NOT DONE** | UI-2/gallery |
| R31–R34 | **PARTIAL** | battery/crash/scale; dirty-tree caveat |
| R35 | **NOT RUN** | live provider evidence |
| R36–R38 | **PARTIAL** | docs/release truth |
| R39–R40 | **NOT DONE** | independent verifier/final seal |

---

## 5. What would make the next lines struck

| Residual | Proof required |
|---|---|
| Monitor actions | test: monitor sends one instruction to a stalled task; cancel one task; repair one failed branch; no authority escalation |
| EVO-GATE independent | evaluator run with task-set hash/artifacts; gate binds `head_commit`/artifact files; test hand-authored receipt still cannot pass |
| EVIDENCE-LEDGER | committed endpoint + route test + UI consumer + target-class counters |
| Active-run crash | kill process during AwaitingApproval; boot does not resume approval; run becomes `interrupted`; no side effect |
| Live provider | keyed run with receipt under `docs/evidence/live/`; no secrets |
| 40-node scale | keyed or scripted 40-node run with provider-call accounting |
| e2e guard | missing/stale results exits non-zero; negative test |
| Clean install | clean profile with `-InstallRoot/-DataRoot`; health, schema, screenshots |
| Release | sealed artifact/tag with matching gate; no pre-release claim |
