# Round 9 — Truthseeker + evidence-check (pre-code gate)

Status: PRE-CODE. No product edits in this stage. App stopped; provider key absent.

## 1. Truthseeker claim ledger

| ID | Claim | Grade | Evidence |
|---|---|---|---|
| TS-01 | HUD renders a live delta while it says replay metrics are hidden | P | `HudBadge.tsx:39` replay branch; `:57-59` unconditional delta |
| TS-02 | Library rail reads live tasks during replay | P | `LibraryPanel.tsx:35` `state.tasks`; no `replayGraph` reference in file |
| TS-03 | Canvas count badge and minimap use live tasks during replay | P | `Canvas.tsx:640` `visibleTasks/tasks`; `:696` same |
| TS-04 | Canvas nodes memo omits `interactiveGraph` | P | `Canvas.tsx:206` uses it; deps `:221` omit it |
| TS-05 | Foreign-run `REPLY_DELTA` can create a second assistant row | P | store run gate covers `NODE_*`/`HISTORY_SNAPSHOT`; `REPLY_DELTA` arm at `:1453` has no gate |
| TS-06 | Hydration merge lets stale live tasks beat newer snapshot revs | P | `store.ts:1175-1194` spread order `{...projection.tasks, ...liveTasks}` with no rev compare |
| TS-07 | Reconnect recovery timer can fire after success | P | `client.ts` `refreshSessionAndOpen` does not clear `recoveryTimer` on success |
| TS-08 | Roster handler formatting is malformed but compiles | P | `http.rs:985` `{    let mut profiles`; `snapshot_history` 4/4 compiles |
| TS-09 | ApprovalOverlay has no focus management/Escape and N stacked modals | P | source has no `useRef`/keydown; `approvals.map` one dialog each |
| TS-10 | History/snapshot error state has zero renderers | P | grep over `web/src/**/*.tsx` found no `snapshot.refusal`/`history.refreshRefusal` consumer |
| TS-11 | Server restart can leave phantom run because store skips refresh on ready snapshot | U | source mechanism plausible; no live restart test |
| TS-12 | Memory is role-scoped and cross-run; rewire row keeps stale parent | P | `execution.rs:3029`, `store.rs:612-671`; no `UPDATE tasks SET parent_task_id` found |
| TS-13 | Keyless WS guard depends on a check that can be absent | P | `ws.rs` guard checks `state.checks`; `checks.rs` suppresses when provider disabled |
| TS-14 | Baseline is green | P | web 187/187; `snapshot_history` 4/4 |
| TS-15 | Visual layout is correct | U | app stopped; no render |

## 2. Evidence-check before coding

| Claim | Evidence | Status |
|---|---|---|
| HUD replay delta is a live-metric leak | source lines above | VERIFIED |
| Library/Canvas replay counters are live-scoped | source lines above | VERIFIED |
| Canvas memo dependency is missing | source lines above | VERIFIED |
| Foreign-run reply can mutate chat | source gate inspection | VERIFIED |
| Hydration merge lacks rev arbitration | source lines above | VERIFIED |
| Reconnect timer not cancelled | source inspection | VERIFIED |
| Approval focus contract is absent | source + no refs/keydown | VERIFIED (fix testability UNVERIFIED) |
| History error UI absent | grep result | VERIFIED |
| Memory/rewire authority split | source; no live reproduction | VERIFIED at source, **NOT FIXED** |
| Restart phantom run | mechanism only | UNVERIFIED |
| Prior "all fixed" claims | no new evidence | NOT CLAIMED |

## 3. Safe implementation queue (this pass only)

1. Format `http.rs` roster/memory boundary; no behavior change.
2. HUD: suppress delta in replay.
3. Library: use replay tasks when replaying.
4. Canvas: derive count/minimap from replay tasks; add `interactiveGraph` to memo deps.
5. Store: rev-aware hydration merge; run-gate `REPLY_DELTA`/`REPLY_DONE`.
6. Client: clear recovery timer on successful reopen.
7. Tests: add focused web tests where the existing harness allows; rerun 187+.
8. Do **not** fix memory, rewire parent row, approval focus, server hello watermark, or scale in this pass — they need dedicated design/tests and would expand blast radius.

## 4. Risk gates

- GitNexus impact: `CanvasInner` HIGH (Canvas → App), `LibraryPanel` LOW, `applyEvent` LOW.
- Backend behavior change limited to roster unknown-run 404 plus formatting.
- Keep key off and servers stopped.
- Every changed file must return to a green suite.

## 5. Post-code verification (2026-11-08)

| Change | Evidence | Status |
|---|---|---|
| HUD suppresses delta in replay | `HudBadge.tsx` replay branch now gates delta; source | VERIFIED at source; no render |
| Library rail uses replay tasks | `LibraryPanel.tsx` imports `buildReplayTasks` + memo; source | VERIFIED at source; no render |
| Canvas count/minimap use replay tasks | `Canvas.tsx` `displayedVisibleTasks`/`displayedTasks`; source | VERIFIED at source |
| Canvas memo includes `interactiveGraph` | dependency array updated | VERIFIED at source |
| Hydration merge is rev-aware | `mergeSnapshotTasks`; new unit test | VERIFIED |
| Foreign run replies ignored | new streaming test | VERIFIED |
| Recovery timer cancelled on success | `refreshSessionAndOpen`; source | VERIFIED at source; no live restart |
| Roster formatting repaired | `cargo test snapshot_history` compiles | VERIFIED |
| Roster unknown run returns 404 | new HTTP assertion | VERIFIED |
| Web suite | 189/189 tests, `tsc` clean | VERIFIED |
| Backend snapshot suite | 4/4 | VERIFIED |
| Approval focus, memory authority, rewire row, scale | not changed | NOT DONE |
| Visual verification | app stopped | UNVERIFIED |

