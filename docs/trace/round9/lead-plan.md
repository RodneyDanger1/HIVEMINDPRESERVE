# Round 9 — Lead plan (deep-solid + forgewright-recon)

Status: PLAN ONLY. No product edits in this stage. App stopped; provider key absent.

## 1. Framing

**Target behavior changed by this work:** HiveMind must remain truthful and safe while the remaining HMARS gaps are filled: scoped history/replay, live hydration, A2A/memory/scale surfaces, and a hierarchy UI that cannot lie about which run it is showing.

**Do not break:** the existing 187 web tests, the core test suites, replay run-scoping, keyless refusal, recovery scoping, and the subagent card distinction.

## 2. Recon ledger — live hydration / stale CAS

- Target behavior: a committed snapshot refresh must not overwrite live streaming text or downgrade a live node revision; `REWIRE_TASK` must use the newest authoritative node revision.
- Entry point: `web/src/state/store.ts:1038` `performSnapshotRefresh`, state write at `:1169`.
- Callers: `refreshRunSnapshot` (`:2221`), resync/reconnect paths, `RUN_STATUS` terminal refresh.
- Tests + command: `npm test -- --run`; no dedicated live-streaming hydration fixture exists (gap).
- Data flow: `/api/history/runs` -> choose run -> `/state` + `/messages` -> projection -> `set({messages,tasks,...})`; failure path -> `snapshot.status='failed'`, refusal toast only for resync/refusal/manual.
- Failure path: active run refresh replaces `messages` mid-stream, loses partial assistant text; active graph nodes can be replaced by older snapshot revs; rewire then submits a stale `expected_rev`.
- Evidence still missing: a live streaming fixture test that proves the preserve branch.
- Files expected to touch: `web/src/state/store.ts`, `web/tests/*`; no backend change for this slice.

## 3. SOLID findings

| Principle | file:line | Evidence | Edit | Call sites | Risk |
|---|---|---|---|---|---|
| SRP | `web/src/state/store.ts:1038` (2358 lines) | One store owns transport events, history hydration, commands, chat, approvals, settings | Extract hydration reducer only after a fixture; do not split this round | store consumers across panels | HIGH — defer |
| SRP | `web/src/flow/Canvas.tsx:98` (809 lines) | Canvas owns layout, drag, replay projection, chat overlay, owner links, controls | Extract owner-link projection and drag-target logic into pure helpers | Canvas only; tests | MEDIUM — do after UI reports |
| OCP | `web/src/state/agentState.ts:126`, `api/contract.ts:213`, `flow/nodes.tsx:277` | Adding `subagent` required edits in 6 places; another rank repeats the shotgun | Introduce one registry descriptor (kind -> size, renderer, label) and derive maps | graph, nodes, HUD, contract | MEDIUM — high benefit, broad test surface |
| OCP | `backend/.../http.rs:743` `node_kind_for_role` | Role-to-kind string switch is duplicated conceptually with UI tier rules | One role/tier normalization helper with tests | history route, roster | LOW |
| DIP | `web/src/state/store.ts:1058` | Store constructs HTTP hydration calls inline | Inject a `SnapshotReader` port after a fixture | store tests | HIGH — defer |
| ISP | `web/src/api/client.ts:49` `UiApi` | Transport interface is small; okay | None | — | — |
| LSP | `web/src/state/metrics.ts:9` | `TaskLike` union is manually extended when a kind is added | Derive from `NodeKindLabel` | metrics, HUD | LOW |
| Truthfulness | `web/src/panels/HudBadge.tsx` | replay hides live metrics now; other replay counters still live-scoped | Derive replay counts from `replayTasks` | Canvas/Library/Minimap | MEDIUM |

## 4. Ordered plan (smallest blast radius first)

1. **Finish read-only audit intake**: collect 2 recon + 3 skill reports; merge only evidence-backed bugs.
2. **Add the missing hydration fixture** (test first): streaming partial + newer live rev + committed snapshot; assert transcript/tasks preserved. No product behavior change until the fixture is red.
3. **Fix hydration preserve path** (only if fixture red).
4. **Unify replay counters** (Canvas badge, Library, minimap) to replay tasks.
5. **Error UI**: surface `history.refreshRefusal` / `snapshot.refusal` with Retry; no fabricated empty state.
6. **Approval keyboard/focus/Escape** (deep-frontend findings).
7. **Registry consolidation** for node kinds only if the test matrix stays green.
8. **A2A roster endpoint** (already staged) + UI consumption if reports justify.
9. **Do not touch** memory banks, scheduler, provider APIs, or full HMARS modeling in this pass without a separate design.

## 5. Truthseeker ledger (pre-code)

| ID | Claim | Grade | Evidence |
|---|---|---|---|
| T1 | App is stopped and key is absent | P | `hiverouter=0`; `key_active=False` |
| T2 | Web baseline is 187/187 | P | `npm test -- --run` output |
| T3 | Snapshot history baseline is 4/4 | P | `cargo test -p hiverouter-core --test snapshot_history` |
| T4 | Replay is run-scoped | P | `history_graph_value` at `http.rs:701`; WS call `ws.rs:846` |
| T5 | Live hydration preserve branch exists but is untested | P | `store.ts:1169` source; test search found no streaming fixture |
| T6 | Stale CAS can still occur | U | No reproducible fixture run this round; do not claim fixed |
| T7 | Replay counters still use live tasks | U | Source read only until reports confirm |
| T8 | Remote/Watchdog/memory/scale/A2A-answer are absent | P | No implementing module/call path found; to be rechecked by recon |

## 6. Evidence-check (pre-code)

| Claim | Evidence | Status |
|---|---|---|
| The staged roster endpoint compiles and returns 2 agents | `snapshot_history` 4/4 includes roster assertions | VERIFIED |
| Hydration fix prevents full LOG-01 | no streaming fixture | UNVERIFIED |
| CAS stale rev fixed | no RED/GREEN | UNVERIFIED |
| Current baseline is green | web 187/187, snapshot 4/4 | VERIFIED |
| Visual hierarchy is correct | app stopped | UNVERIFIED |
| Memory/A2A/scale complete | absent | NOT DONE |

## 7. Guardrails

- No provider/key action.
- No server/browser start.
- One writer per file; check GitNexus impact before editing critical symbols.
- Tests must stay at 187 web + green core, or the change is rejected.
