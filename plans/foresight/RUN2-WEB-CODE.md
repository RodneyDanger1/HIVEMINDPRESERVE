# RUN2-WEB-CODE — WP-06 web approval replay + retry/refusal implementation plan

**Planner:** RUN 2 · PLANNER 3 · **Mode:** READ-ONLY for product code — **this file is the only write**; no source edits, no build, no test run, no paid call.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `90cb7a7` (`feat(1.2): reasoning passback, approval boot expiry, recovery timeout; foresight run-1 plans/ledger`) · **Run-1 tree:** clean working tree; **web files are the pre-Run-2 baseline** (RUN-1.md records the half-integrated UI edits were reverted, so nothing of WP-06 exists yet).
**Inputs read:** `plans/foresight/COMPILED-LEDGER.md` §WP-06 (+ WP-05, WP-07, C9/C10, WAVE-WEB ordering) · `plans/foresight/CODE-APPROVAL-UX.md` (all 822 lines) · `plans/foresight/RUN-1.md` · current `web/src/state/store.ts`, `ApprovalOverlay.tsx`, `strings.ts`, `api/contract.ts`, `api/client.ts`, `api/refusals.ts`, `web/tests/*` · `e2e/helpers/replay-harness.ts`, `e2e/specs/approval-flow.spec.ts`, `tool-loop.spec.ts` · backend anchors `core/src/ws.rs`, `core/src/tools_bridge.rs`, `tools/src/approval.rs`.
**Skills applied:** `/research` (read-first facts), `/forgewright-lead` (blueprint + unverified split), `/forgewright-plan` (decision-complete sections), `/deep-naming` (symbol vocabulary), `/truthseeker` (fail-closed state honesty), `/scientific-thinking` (separate observed vs inferred), `/forge` (defect classification + RED tests), `/evidence-check` (claim table §10).

---

## 0. Verified current behavior (baseline anchors)

| # | Fact | Evidence (`path:line`, read this run) |
|---|---|---|
| B1 | `ApprovalRecord` has `approvalId/tool/nonce/decision/requestedAt` + nullable `runId/taskId/toolCallId/argsSha256/expiresAt` | `web/src/state/store.ts:61-73` |
| B2 | `UiState` has `approvals` and `resolvingApprovals`; `resolveApproval(...)` returns `Promise<void>` | `store.ts:160-162,183,739-777` |
| B3 | `run()` returns `boolean` and **discards** the typed `Refusal` (only toasts 3 local codes); all 12 call sites are `store.ts:679,696,709,722,736,766,780,787,800,851,890,910` | `store.ts:565-596` |
| B4 | `APPROVAL_REQUEST` replace-by-id; `APPROVAL_RESOLVED` removes by id; both have no replay/epoch/attempt state | `store.ts:484-509` |
| B5 | `SERVER_HELLO` only sets `lastSeq`/`connection='open'`; `RESYNC_REQUIRED` only sets `connection='reconnecting'`; `subscribeStatus` only records connection/refusal | `store.ts:310-317,643-646` |
| B6 | Overlay owns a **panel-local `pendingResolution` lock** (`useState`), pruned only when the approval disappears; both actions derive from `canResolveApproval(nonce, resolving, expired)`; submit swallows throws | `web/src/panels/ApprovalOverlay.tsx:167,183-190,196-215,221-225` |
| B7 | Overlay exports `canResolveApproval` and `prunePendingResolutions` (tests import them); DOM: `section[role=dialog][aria-label="Tool approval requested: <tool>"]`, `dd[title=…]`, `dd[aria-live=polite]`, `button[aria-label="Approve/Deny tool call for <tool>"]` | `ApprovalOverlay.tsx:114-153,237-243,296-319` |
| B8 | Strings approval block: `title/toolLabel/resolving/resolveHint/fields/expiry/unknownValue/nonceMissing`; no status/retry/dismiss/replay copy | `web/src/state/strings.ts:244-267` |
| B9 | Contract has `ApprovalResolvePayload {approval_id,nonce,decision,reason?}`; `Refusal {code,message,how_to_fix,retryable}`; **no `args_preview`**; event count 23 | `web/src/api/contract.ts:70-75,366-371` |
| B10 | Client reconnects via partysocket; sends hello on first open and on every reopen with `this.lastSeq`; status reaches `'reconnecting'`; default command timeout 15 s | `web/src/api/client.ts:5-11,87-91,293,299-305,9-10` |
| B11 | Mock harness auto-ACKs COMMAND frames; exposes `route()/sent/sentCommands/commands/waitForCommand/approvalResolveCommands/sendRaw`; **no hello counter, no close/reconnect, no server-hello/error-for-command helper** | `e2e/helpers/replay-harness.ts:281-307,312-390` |
| B12 | Existing approval tests: 9 integration cases + 10 static mount cases from a static-source scan; e2e has 8 approval cases incl. ACK-stays-open and double-click | `web/tests/approval.integration.test.ts:113-274`; `web/tests/agentsToolActivityMount.test.ts:116-305`; `e2e/specs/approval-flow.spec.ts` |
| B13 | TypeScript is strict with `noUnusedLocals`/`noUnusedParameters`; scripts are `typecheck: tsc --noEmit`, `test: vitest run`, `build: tsc --noEmit && vite build` | `web/tsconfig.json`; `web/package.json` |
| B14 | Backend replay projection does **not exist**: grep for `pending_approval_events_all`/`pending_ids_for_active_runs`/`publish_pending_approval_replays`/`send_hello_replay_and_approvals` returns 0 hits; existing pieces: `pending_for_run` (`tools/src/approval.rs:710`), private `load_request` (`approval.rs:213`), `ToolsBridge::pending_approval_events` (`core/src/tools_bridge.rs:432`), `approval_request_event` (`:410`), hello call sites `ws.rs:147,316` | grep/read this run |
| B15 | Ledger WP-06 declares: new `web/src/state/approval.ts`, typed `CommandOutcome`, `approvalUiState` (`idle|sending|awaiting|retryable|terminal`), remove overlay lock, e2e helpers, flag `VITE_UI_APPROVAL_REPLAY`, depends on WP-05; R3 refines to the exact state/availability/classifier tables used below | `COMPILED-LEDGER.md:96-105`; `CODE-APPROVAL-UX.md:222-355,589-757` |

**Naming decision (deep-naming):** adopt the ledger name `CommandOutcome` for the new `run()` result (`{ok:true} | {ok:false; refusal:Refusal}`); R3's `CommandResult` is not exported. Keep R3's `ApprovalResolveOutcome = {ok:true} | {ok:false; kind:ApprovalFailureKind; refusal:Refusal}` and `ApprovalResolveState`. `resolvingApprovals` stays as a compatibility projection (source of truth `approvalResolve`); do **not** introduce a second disabled-state name.

---

## 1. Phase 1 (logic-gatekeeper record — planning only, no mutation)

```json
{
  "phase1": {
    "architecture": "Server truth: approvals table (hiverouter-tools) + ToolsBridge read-only projections (pending_for_run, later pending_approval_events_all) + ws.rs hello/replay. Client truth: WsApiClient (contract frames, ACK-only settling), store.ts (approvals, approvalResolve state machine, replay window, typed CommandOutcome), pure web/src/state/approval.ts (classifier/display/availability/replay reducers), ApprovalOverlay list + ApprovalCard/ApprovalStatusStrip presentational components. This plan edits only client files; the replay producer is WP-05 (backend).",
    "dataFlow": "Untrusted inputs are WS EVENT payloads (APPROVAL_REQUEST / APPROVAL_RESOLVED / SERVER_HELLO / RESYNC_REQUIRED / ERROR) and operator gestures. Contract decoder -> store.applyEvent -> approvalResolve reducer + approvals upsert. Gesture -> store.resolveApproval -> run() (now CommandOutcome) -> one APPROVAL_RESOLVE COMMAND -> ACK (awaiting_server) or typed ERROR refusal (classified retryable/stale/expired). Only APPROVAL_RESOLVED removes a record; reconnect replay re-emits APPROVAL_REQUEST after SERVER_HELLO and only confirms records during a bounded window; nothing in hydration/replay/settle may send.",
    "edgeCases": [
      "reconnect with a pending un-decided approval: hello opens a replay window; replayed request confirms it once, actions enabled only after settle; zero commands before a click",
      "approval resolved while the client was away: server replays nothing; pre-reconnect card becomes unconfirmed read-only after settle; dismiss removes locally only",
      "page reload/new tab: fresh store has no records; replay renders a pending card; no storage/nonce persistence; no auto-send",
      "typed ERROR after a gesture: ACK-only status becomes retryable after the ACK settle budget; E_APPROVAL_REPLAY/NOT_FOUND -> stale, E_APPROVAL_TIMEOUT/REQUIRED -> expired; retry reuses the same single-use nonce and is bounded; card never gets permanently stuck",
      "ACK received but APPROVAL_RESOLVED lost: awaiting_server -> retryable after settle timer; only a click resends",
      "duplicate gesture while resolving/awaiting: ignored (no send, no toast)",
      "nonce-less request, client-clock expired request, disconnected connection: actions disabled, named copy, no command",
      "server without WP-05 (old backend): after a reconnect pre-existing cards become unconfirmed read-only (fail-closed) instead of wrong-actionable; flag VITE_UI_APPROVAL_REPLAY=0 keeps legacy behavior for rollout",
      "timers: replay settle, ACK settle, per-card TTL display timer and retry attempts must be cleared on APPROVAL_RESOLVED/dismiss/dispose; advancing fake time after dispose changes nothing"
    ],
    "rootCause": "The store has no typed command outcome (run() swallows the Refusal), no per-approval resolve state machine (so an ERROR or lost APPROVAL_RESOLVED leaves the overlay's panel-local lock permanently armed), and no reconnect/replay hydration (so pending approvals vanish on reconnect and a reload loses them entirely). The server will only be able to replay rows (WP-05) if the client can (a) receive the replay after hello without auto-sending, (b) distinguish confirmed/replayed records from stale ones, and (c) surface retryable vs terminal refusal classes; hence the fix belongs in store state + pure reducer + presentational split, not in another local useState lock."
  }
}
```

**Invariants (all hunks):** (1) no auto-send from hydration/replay/settle; (2) ACK is never terminal and never removes/enables; only `APPROVAL_RESOLVED` removes a record; (3) single-use nonce — retry reuses `approval_id`+`nonce` and a new correlation id; (4) bounded retries (`MAX_APPROVAL_RESOLVE_ATTEMPTS=3`) and bounded replay (`MAX_APPROVAL_REPLAY_EVENTS=20`); (5) no secret/argument-body storage or DOM attributes (nonce never rendered); (6) existing selectors stay valid (`role=dialog`+aria-label, `dd[title]`, TTL `dd[aria-live]`, approve/deny aria-labels + disabled semantics) — new attributes are additive only; (7) all copy from `STRINGS` (no literals); (8) fail-closed: unknown status/refusal → disabled.

---

## 2. Ordered hunks (implement in this order; each independently verifiable)

### H1 — new pure module `web/src/state/approval.ts` (no JSX, no store import cycles)

Create with exported constants/types/functions exactly as `CODE-APPROVAL-UX.md:224-304` (the R3 design is adopted, not re-invented):
`APPROVAL_REPLAY_SETTLE_MS=400`, `MAX_APPROVAL_REPLAY_EVENTS=20`, `MAX_APPROVAL_RESOLVE_ATTEMPTS=3`, `APPROVAL_ACK_SETTLE_MS=2000`, `APPROVAL_RESOLVE_TIMEOUT_MS=15000`; `ApprovalDisplayStatus` = `pending|resolving|awaiting_server|retryable|expired|stale|unconfirmed`; `ApprovalFailureKind` = `transport|operator|stale|expired|terminal|ignored`; `ApprovalResolveState {status,attempts,lastDecision,refusal,lastAttemptAt}`; `ApprovalResolveOutcome`; `initialApprovalResolveState()`; `classifyApprovalRefusal(refusal)` per the R3 classifier matrix (`CODE-APPROVAL-UX.md:278-288`); `approvalDisplayStatus(record, resolve, nowMs)` (precedence: resolve status, else expired, else pending); `approvalActionsEnabled({record,resolve,nowMs,connection})` returning `{approve,deny,retry,dismiss}` per the R3 availability table (`:290-304`); replay helpers `beginReplayWindow(state)`, `stampReplayEpoch(...)`, `settleReplay(...)` as pure functions if store ergonomics allow (keep timers in store).
**Acceptance:** pure functions; no DOM/store/timer side effects; exported types used by store/components/tests. **Tests:** H8 cases 1–6.

### H2 — typed command outcome in `web/src/state/store.ts`

Replace `run()` (`store.ts:565-596`) with:

```ts
export type CommandOutcome =
  | { readonly ok: true }
  | { readonly ok: false; readonly refusal: Refusal };

const run = async <N extends ServerCommandName>(
  name: N, payload: CommandPayloads[N],
  options?: { readonly timeoutMs?: number },
): Promise<CommandOutcome> => { /* same body; return {ok:true} / {ok:false,refusal}; toast policy unchanged */ };
```

Call-site edits (exact current lines): keep `await run(...)` at `679,696,709,722,736,766(inside new resolve),787,800`; change to `const outcome = await run(...)` at **780** (`CHAT_PIN`: `if (outcome.ok)`), **851** (`SETTINGS_STATUS`), **890** (`FILE_TREE_LIST`), **910** (`FILE_READ`); update their `if (ok)` bodies to `outcome.ok`. No other file may consume `run()`.
**Acceptance:** typecheck with `noUnusedLocals`; server-error refusals preserved verbatim; toast behavior unchanged; `resolveApproval` can now distinguish refusal kinds. **Tests:** H9 case 9.

### H3 — replay state + hydrate/confirm/settle actions

In `UiState` after `resolvingApprovals` (`:162`) add (R3 `CODE-APPROVAL-UX.md:306-337`): `approvalResolve: Record<string, ApprovalResolveState>`, `approvalSeenEpoch: Record<string, number>`, `approvalsStale: boolean`, `approvalReplayEpoch: number`, `approvalReplayWindow: boolean`, `lastApprovalReplayAt: number|null`, `approvalReplaySource: 'hello'|'session'|null`. Actions: `hydrateApprovals(events, source)`, `markApprovalsStale()`, `beginApprovalReplay(source)`, `settleApprovalReplay()`, `expireApproval(id)`, `dismissApproval(id)`, `retryApproval(id, decision?, reason?)`, plus `clearApprovalResolveFailure` is **not** needed (superseded by state machine). Init state `{}`, `false`, `0`, `false`, `null`, `null`.
Reducer wiring:
- `subscribeStatus` (`:643`): on `'reconnecting'` → `approvalsStale=true; approvalReplayEpoch+=1; approvalReplayWindow=false; clear replay settle timer`; keep records.
- `SERVER_HELLO` (`:310`): set `lastSeq/connection` as today, then if flag enabled and (`approvalsStale` or fresh connect with records) `beginApprovalReplay('hello')` (window=true, arm 400 ms settle timer).
- `RESYNC_REQUIRED` (`:314`): mark stale as above.
- `APPROVAL_REQUEST`: extract upsert into `hydrateApprovals([payload], 'session')`-compatible rule: stamp `approvalSeenEpoch[id]=approvalReplayEpoch`; if status ∈ `{unconfirmed,stale,expired}` and payload expiry future → `pending`, refusal null; preserve `resolving|awaiting_server`; never touch timers.
- `APPROVAL_RESOLVED`: existing removal + delete `approvalResolve[id]`, `approvalSeenEpoch[id]`, clear that id's timers.
- settle timer: `window=false; stale=false`; records whose seen epoch !== current and status ∈ `{pending,retryable}` → `unconfirmed`; dispose clears timer.
Flag: single read point `const approvalReplayEnabled = import.meta.env.VITE_UI_APPROVAL_REPLAY !== '0'` in `createUiStore` closure (never in reducers; when disabled, skip stale/unconfirmed marking → legacy behavior).
**Acceptance:** no command in any hydrate/settle path; replay of same id produces exactly one record; ACK never implies confirmation; old-backend fallback documented. **Tests:** H8/H9 replay cases.

### H4 — resolve state machine, typed outcome, retry/dismiss (replace `resolveApproval` `:739-777`)

Implement exactly `CODE-APPROVAL-UX.md:589-687`:
- `resolveApproval(...): Promise<ApprovalResolveOutcome>` — not-found/nonce-less/expired/retry-cap early returns; set `{status:'resolving', attempts+1, lastDecision, refusal:null}`; `run('APPROVAL_RESOLVE', …, {timeoutMs: APPROVAL_RESOLVE_TIMEOUT_MS})`; on ok → `awaiting_server` + arm ACK settle timer + `resolvingApprovals` projection; on refusal → `classifyApprovalRefusal` → status `retryable|stale|expired`, store verbatim `refusal`; stale/expired clear timers.
- `retryApproval(...)`: only from `retryable`, `attempts<3`, `connection==='open'`; reuses last decision; delegates to `resolveApproval` (new correlation id; same nonce).
- `expireApproval(id)`: `pending|retryable` → `expired` (display; server decides truth); `dismissApproval(id)`: allowed only `expired|stale|unconfirmed`, removes record+resolve+epoch locally, **no command**.
- ACK settle timer: timeout → if still `awaiting_server` → `retryable` with synthetic `clientRefusal('E_WS_DISCONNECTED', {message:'No APPROVAL_RESOLVED within 2000 ms; retry is safe, the nonce is single-use.'})`; cleared on resolved/dismiss/dispose.
- `resolvingApprovals`: maintain as projection of `resolving|awaiting_server` statuses; update doc comment at `store.ts:161-162`; existing double-click test semantics preserved.
**Acceptance:** ACK-only card shows awaiting then retryable after 2 s; ERROR codes classify per matrix; card never removed without APPROVAL_RESOLVED; retries bounded; no optimistic mutation. **Tests:** H9 cases 1–10.

### H5 — strings (`web/src/state/strings.ts` approval block `:244-267`)

Add exactly R3 §3.5/§5.5: `replayPending`, `status.{resolving,awaiting,retryable,expired,stale,unconfirmed}`, `retryLabel:'Retry approval'`, `dismissLabel:'Dismiss approval'`, `refusalTitle:'Approval decision refused'`, `retryCapReached:'Retry limit reached (3) — dismiss this card or cancel the run.'`; reuse `STRINGS.controls.retry/dismiss`. No literal copy in components.
**Acceptance:** static-source scan in tests finds no hardcoded status/refusal strings.

### H6 — overlay split; delete the panel lock

`ApprovalOverlay.tsx`: keep exports `APPROVAL_DECISIONS, ApprovalDecision, ApprovalView, UNKNOWN_VALUE, truncateId, formatArgsSha256, ExpiryLabels, formatExpiry, isApprovalExpired, approvalAriaLabel, approvalActionAriaLabel`; **delete** `canResolveApproval`, `prunePendingResolutions`, `pendingResolution` state/effect and the local `submit` lock (`:114-153,167,183-215,221-225`); component becomes list/mount only (still the single `App` mount at `web/src/App.tsx:100`; no App change).
New `web/src/panels/ApprovalCard.tsx`: props per `CODE-APPROVAL-UX.md:701-731`; keeps the existing `section role=dialog aria-modal aria-label`, `<dl>` evidence, nonce-less copy, TTL `dd[aria-live=polite]`; enables actions only from `approvalActionsEnabled(...)`; adds additive DOM contract: `data-approval-status={status}` on dialog, `aria-busy` while `resolving|awaiting_server`, stale banner `[role=status][data-testid=approval-replay-banner]`, `[data-testid=approval-status-strip]`, refusal `role=alert` verbatim `code · message · how_to_fix`, retry `button[aria-label="Retry approval for <tool>"]`, dismiss `button[aria-label="Dismiss approval for <tool>"]`. New `ApprovalStatusStrip.tsx` per `:717-727`; new `ApprovalArgsPreview.tsx` is **out of scope** (WP-07/1.3-gated; item 11 ships hash-only copy from R3 §4.1 — adopt `STRINGS.approval.argsBoundByHash` text only if Lead schedules it; not needed for WP-06 acceptance).
**Acceptance:** all existing approval e2e selectors still match; no local `useState` disabled source; retry/dismiss/refusal visible only for their statuses. **Tests:** H10.

### H7 — `web/src/App.tsx`
No change required (overlay remains mounted once, `App.tsx:100`); test keeps pinning single mount.

### H8 — new unit suite `web/tests/approval.state.test.ts`
Exact cases (vitest, node): `classifies every refusal code into the retryability matrix and defaults unknown codes to bounded operator-retry`; `availability truth table never enables stale/expired/unconfirmed/nonce-less/disconnected cards`; `display status precedence is resolve status then expired then pending`; `retry cap helper allows retries only below MAX_APPROVAL_RESOLVE_ATTEMPTS`; `initial state has idle/pending defaults`; `pure replay reducers stamp epochs and settle only unconfirmed records`.

### H9 — extend `web/tests/approval.integration.test.ts` (existing `FakeUiApi` harness `:43-100`)
Add `emitStatus(status, detail?)`, `rejectNext(refusal)`, `hello(lastSeq)`, `replay(payload)` helpers. Exact new/extended cases (names at `CODE-APPROVAL-UX.md:391-415,740-751`): reconnect-mark-stale + hello replay confirms once; pre-reconnect record not replayed → unconfirmed + dismiss only, no commands; APPROVAL_RESOLVED during window removes record and late replay can't resurrect; ACK → awaiting_server keeps card; ACK-settle timer → retryable + one retry command (same id/nonce, new correlation); transport refusal → retryable bounded cap; `E_APPROVAL_REPLAY` → stale no retry; `E_APPROVAL_TIMEOUT` → expired; `run()` typed `CommandOutcome` preserves exact refusal code; `dispose` clears ack/replay timers (fake timers; advance after dispose → no change). Update the two tests that assert the old panel lock (`:155-199`) to assert store state machine + card DOM contract instead.

### H10 — static render + selector suites
New `web/tests/approvalPanels.test.ts` per `CODE-APPROVAL-UX.md:751` (each status renders expected strip/buttons/aria-busy/refusal verbatim; `[data-testid=approval-status-strip]`, retry/dismiss aria-labels; no hardcoded copy). Update `agentsToolActivityMount.test.ts:249-284` (currently drives buttons from pending lock/resolving; rewrite to store-driven statuses while keeping selectors) and `toolActivityPanels.test.ts:57-70` (imports `canResolveApproval` — replace with `approvalActionsEnabled`/`approvalDisplayStatus`).
**Commands:** `cd web; npm.cmd run typecheck` (exit 0, no unused symbols); `npm.cmd test -- --run tests/approval.state.test.ts tests/approval.integration.test.ts tests/approvalPanels.test.ts` (all pass); `npm.cmd test -- --run` (existing 147 + new, 0 failed); `npm.cmd run build`.

### H11 — e2e harness helpers (`e2e/helpers/replay-harness.ts`)

Implement R3 §3.7 (`CODE-APPROVAL-UX.md:417-432`): track **every** routed connection (`const routes: WebSocketRoute[]`, `current=routes.at(-1)`) and count hello control frames (`helloFrameCount(ws)`; increment on `parsed.type==='hello'` inside `route.onMessage`, before ACK logic); add `closeMockSocket(ws)` (`ws.route().close()` — Playwright `WebSocketRoute.close` is the R3 UNVERIFIED surface; if unavailable in 1.65.0, fall back to page-side close via the app socket: expose `ws.sent` hello count and use `page.reload()` path only, marking the socket-close case UNVERIFIED); `waitForReconnect(ws, priorHelloCount, timeoutMs=5000)` polls `helloFrameCount > prior` and returns latest route; `sendServerHello(ws, payload, seq)` sends a validated SERVER_HELLO EVENT; `sendErrorForCommand(ws, command, code, seq)` sends an EVENT `ERROR` with `correlation_id = command.correlationId` and a frozen-copy payload fixture (`message/how_to_fix/retryable` supplied by the spec, documented). Keep auto-ACK and malformed/no-fake behavior unchanged.
**Acceptance:** helper itself never auto-sends server events; reconnect capture deterministic; timeout throws with captured hello counts (no silent pass). **Note:** Playwright `routeWebSocket` re-invocation on reconnect is UNVERIFIED (§8); the e2e replay cases are gated on the H11 spike result.

### H12 — e2e spec `e2e/specs/approval-flow.spec.ts` (extend; existing 8 cases stay)

New exact cases/selectors/expected:
1. `'reconnect replays a pending approval exactly once with zero auto-commands'` — open card; capture `helloFrameCount`; `closeMockSocket`; `waitForReconnect`; `sendServerHello(ws,{last_seq:12,schema:SCHEMA_VERSION},12)`; `sendApprovalRequest(ws,payload,13)`; expect one `getByRole('dialog',{name:/file_write/})`, `[data-testid=approval-status-strip]` absent/pending, both action buttons enabled after settle, `ws.approvalResolveCommands()` length 0; banner `[data-testid=approval-replay-banner]` disappears.
2. `'approval resolved while away stays read-only (unconfirmed) and sends nothing'` — pre-existing card; close+reconnect+hello without replay; expect approve/deny disabled, `button[aria-label="Dismiss approval for file_write"]` visible, status strip unconfirmed, 0 commands.
3. `'page reload hydrates a replayed pending card with zero commands'` — `page.reload()`; wait Connected; hello+replay; card enabled; 0 commands.
4. `'ACK-only stall becomes retryable and retry issues exactly one more command'` — click approve (auto-ACK), no APPROVAL_RESOLVED; expect `data-approval-status="awaiting_server"` then within ~3 s `retryable` + retry enabled; click retry → `approvalResolveCommands()` length 2 with identical `approval_id`/`nonce`, different `correlationId`; then `APPROVAL_RESOLVED` closes card; count stays 2.
5. `'typed ERROR E_APPROVAL_REPLAY renders stale, disables actions, dismiss-only and never resends'` — approve → `sendErrorForCommand(ws, ws.approvalResolveCommands()[0], 'E_APPROVAL_REPLAY', seq)` → status strip shows code verbatim, approve/deny disabled, dismiss visible, count 1.
6. `'E_APPROVAL_DECISION_INVALID is retryable: retry sends a second command, APPROVAL_RESOLVED closes'` — variant of 5 with retry enabled; retry sends same id/nonce new correlation; count 2; resolve closes; count 2.
7. Update old assertions that relied on the panel pending lock (`approve keeps card open`, `deny`, double-click) to assert `data-approval-status` transitions and unchanged command counts; keep exact dialog/action selectors.
**Commands:** rebuild `web/dist` first (`cd web; npm.cmd run build`), then `cd e2e; node run-guarded.mjs --project=chrome specs/approval-flow.spec.ts specs/tool-loop.spec.ts` → exit 0 with passed>0 (skip-guard active); full `npm run e2e:guarded -- --project=chrome` later.

### H13 — rollout flag + compatibility
Read `VITE_UI_APPROVAL_REPLAY` once (H3): default enabled; `'0'` restores pre-WP-06 behavior (records remain actionable across reconnect; no stale/unconfirmed marking) as the old-backend fallback. Browser build must be rebuilt for the flag (Vite compile-time); document in the wave report. No contract/event/command/refusal change in WP-06; args preview remains deferred (WP-07, 1.3-gated per ledger `:107-118`).

---

## 3. Test matrix (implementation wave)

| Gate | Command | Expected |
|---|---|---|
| Types | `cd web; npm.cmd run typecheck` | exit 0; no unused locals/params |
| Pure state | `cd web; npm.cmd test -- --run tests/approval.state.test.ts` | all pass (H8) |
| Store integration | `cd web; npm.cmd test -- --run tests/approval.integration.test.ts` | existing+new pass; zero-command assertions |
| Panels | `cd web; npm.cmd test -- --run tests/approvalPanels.test.ts tests/agentsToolActivityMount.test.ts tests/toolActivityPanels.test.ts` | all pass |
| Full web | `cd web; npm.cmd test -- --run` | 147 baseline + new, 0 failed |
| Build | `cd web; npm.cmd run build` | exit 0; `web/dist` rebuilt for e2e |
| E2E mocked | `cd e2e; node run-guarded.mjs --project=chrome specs/approval-flow.spec.ts specs/tool-loop.spec.ts` | exit 0; passed>0; new cases 1–6 pass |

No paid calls, no live server, no backend edits in WP-06. **This planning task ran nothing.**

---

## 4. What WP-06 needs from backend WP-05 (blocking dependency)

WP-05 must exist for real reconnect replay; mocked e2e (H12) can run before it. Required server behavior (from R3 §3.2, adopted):

| Need | Exact server surface (R3 design) | Current state |
|---|---|---|
| Read-only pending projection | `hiverouter-tools/src/approval.rs`: `pub fn pending_ids_for_active_runs(conn, limit)` (join `runs`, `decision IS NULL`, non-terminal statuses, oldest first, LIMIT); make `load_request` `pub` | absent (`approval.rs:213` private; no ids fn) |
| Bridge projection | `hiverouter-core/src/tools_bridge.rs`: `pub fn pending_approval_events_all(&self, limit) -> Result<Vec<event::ApprovalRequest>, Refusal>` reusing `approval_request_event` (`:410`); skip expired rows; never consume nonce | absent (`:432` is per-run only) |
| WS replay after hello | `core/src/ws.rs`: `const MAX_APPROVAL_REPLAY_EVENTS: usize = 20`; `publish_pending_approval_replays(state)`; `send_hello_replay_and_approvals(...)`; call from **both** hello paths (`ws.rs:147` initial, `:316` re-hello/resync) | absent; current call sites direct `send_hello_and_replay` |
| Ordering/seq semantics | replay must be published **after** SERVER_HELLO on the same socket (bus subscribed at `ws.rs:133` before hello); replayed events get new bus seqs — client `last_seq` advances; confirm no `RESYNC_REQUIRED` loop and that replay events do not break `hello{last_seq}` ordering | decision needed (U6) |
| Scope policy | bounded 20, all active runs, every hello (covers reload) vs reconnect-only/latest-run-only | Lead decision (R3 open Q1) |
| Safety | no replay on read-only session; no nonce consumption; no duplicate for decided/expired rows; multi-client broadcast idempotent by approval_id | acceptance criteria |
| Backend tests | new `backend/crates/hiverouter-core/tests/approval_replay.rs` (bounded, idempotent, no nonce consumption, hello-before-replay ordering) + `ws_transport` reconnect case | not written |
| Client assumptions to honor | replay is the only hydration source; no "replay complete" frame → client uses the 400 ms window; resolved-while-away produces no frame (never resurrect) | documented in H3 |

Also needed from WP-05 owner: the replay events must carry full six optional fields (`nonce/run_id/task_id/tool_call_id/args_sha256/expires_at`) exactly as `approval_request_event` does today; if any field is missing the client stays fail-closed (nonce-less copy) rather than fabricating.

**Not needed from backend for WP-06:** args preview (WP-07), new events/commands/refusals, DB migration, ledger changes.

---

## 5. Edge/failure matrix (implementation truth table)

| Situation | Required client behavior | Proof |
|---|---|---|
| reconnect while card pending | mark stale; hello opens window; replay confirms once; no command | H12-1 |
| replay never arrives (old backend) | after settle → unconfirmed read-only, dismiss only | H12-2 |
| replay arrives twice | replace-by-id; one dialog; no extra command | H9 replay dedupe |
| replay after local decision gesture | preserve `resolving/awaiting_server`; never re-enable | H9 |
| APPROVAL_RESOLVED during window | remove immediately; late replay ignored | H9/H12 |
| ACK only | awaiting_server; after 2 s → retryable; no removal | H12-4 |
| ERROR E_APPROVAL_REPLAY/NOT_FOUND | stale, no retry | H12-5 |
| ERROR E_APPROVAL_TIMEOUT/REQUIRED | expired, dismiss only | H8/H9 |
| ERROR E_APPROVAL_DECISION_INVALID | retryable, bounded | H12-6 |
| duplicate gesture | ignored, no send/toast | existing double-click test |
| nonce-less | disabled + gap copy, no command | existing test |
| expired (client clock) | disabled, expired status; server remains truth | existing test |
| page reload | fresh store, replay hydrates, no storage/nonce at rest | H12-3 |
| dispose | clears replay/ACK timers; advancing time after → no state change | H9-10 |
| multi-tab | APPROVAL_RESOLVED from one tab removes in all via event; replay idempotent | design + store test |

---

## 6. File-change map (WP-06 only)

| File | Change |
|---|---|
| `web/src/state/approval.ts` (new) | constants, types, classifier, display/availability, replay reducers |
| `web/src/state/store.ts` | `CommandOutcome`; replay fields/actions/timers; resolve state machine; retry/dismiss/expire; `resolvingApprovals` projection; dispose timers |
| `web/src/panels/ApprovalOverlay.tsx` | list/mount only; remove local lock + `canResolveApproval`/`prunePendingResolutions` |
| `web/src/panels/ApprovalCard.tsx` (new) | presentational card + additive DOM contract |
| `web/src/panels/ApprovalStatusStrip.tsx` (new) | status/refusal strip |
| `web/src/state/strings.ts` | replay/status/retry/dismiss/refusal copy |
| `web/tests/approval.state.test.ts` (new), `approval.integration.test.ts` (extend), `approvalPanels.test.ts` (new), `agentsToolActivityMount.test.ts` + `toolActivityPanels.test.ts` (update imports/assertions) | H8–H10 |
| `e2e/helpers/replay-harness.ts` | hello count/routes, close/waitForReconnect, sendServerHello, sendErrorForCommand |
| `e2e/specs/approval-flow.spec.ts` | replay/unconfirmed/reload/retry/ACK-only/ERROR cases; update old lock assertions |
| `web/src/App.tsx` | **no change** |

---

## 7. Rollback / out of scope

- **Rollback:** `VITE_UI_APPROVAL_REPLAY=0` restores pre-WP-06 connect behavior; revert H1-H6 + tests as one change set (no DB/contract/event changes to undo); `ApprovalCard/StatusStrip` are additive files; overlay exports preserved so e2e selectors survive a partial revert.
- **Out of scope:** backend WP-05 edits (separate owner), args preview (WP-07; contract+DB 1.3-gated; hash-only copy only), live paid proof (WP-07), removal of `resolvingApprovals` compatibility field (later cleanup), persistence/localStorage (rejected by privacy review), Escape-to-dismiss, multi-approval stacking redesign, trace/G-UI work.

---

## 8. Blocking unknowns / UNVERIFIED (resolve before or during implementation)

1. **Playwright `WebSocketRoute.close()` + re-invocation of `page.routeWebSocket` handler on reconnect** — R3 marks UNVERIFIED; H11 must spike first. Fallback (if unsupported in 1.65.0): keep reload-based replay cases + unit-level reconnect simulation in H9; record the socket-close e2e case as UNVERIFIED rather than weakening assertions.
2. **Server replay sequence/ordering policy** (bounded 20, every hello, all active runs) and whether replayed bus events may advance `last_seq`/trigger resync (WP-05/U6) — Lead/backend decision; client H3 is written fail-closed for either policy.
3. **ACK→APPROVAL_RESOLVED settle budget (2000 ms)** — product decision vs real server latency (events are published back-to-back today); keep constant named/overridable for tests.
4. **R3 vs ledger naming drift** — ledger says `approvalUiState` statuses `idle|sending|awaiting|retryable|terminal`; R3 refines to `ApprovalResolveState`/display statuses. This plan adopts R3 (more precise) and exports `CommandOutcome` per ledger; compiler/Lead should reconcile the ledger row rather than merging both vocabularies.
5. **Selectors currently asserted by tests** (`canResolveApproval`, `prunePendingResolutions`, panel-lock static scans) will be removed — H10 enumerates the exact test edits; any other import of those exports (grep before implementation) must be updated in the same change set.
6. **No runtime/build/test evidence in this planning task** — every "must/pass" above is a design requirement; §0 facts are reads only.
7. **Old backend + new client:** after reconnect all pre-existing cards become unconfirmed (read-only) — acceptable fail-closed, but the rollout order must ship WP-05 before enabling the flag in a release build if user-visible regression is unacceptable (Lead decision).

---

## 9. Read references (this session)

`plans/foresight/COMPILED-LEDGER.md` (WP-05/06/07, C9/C10, WAVE-WEB) · `plans/foresight/CODE-APPROVAL-UX.md` §0–§9 · `plans/foresight/RUN-1.md` · `web/src/state/store.ts` · `web/src/panels/ApprovalOverlay.tsx` · `web/src/state/strings.ts` · `web/src/api/contract.ts` · `web/src/api/client.ts` · `web/src/api/refusals.ts` · `web/tests/{approval.integration.test.ts,toolActivityPanels.test.ts,agentsToolActivityMount.test.ts}` · `web/tsconfig.json` · `web/package.json` · `e2e/helpers/replay-harness.ts` · `e2e/specs/approval-flow.spec.ts` · backend `core/src/ws.rs`, `core/src/tools_bridge.rs`, `tools/src/approval.rs`.

## 10. Evidence check (claim → evidence → status)

| Claim | Evidence | Status |
|---|---|---|
| Store swallows typed refusals; overlay lock causes stuck card | `store.ts:565-596`; `ApprovalOverlay.tsx:167,183-215` | VERIFIED (read) |
| No replay projection exists in backend | grep for 4 new symbols → 0 hits; only per-run `pending_approval_events` | VERIFIED (read/grep) |
| Existing selectors/tests will need the enumerated edits | B7, B12, B15 vs R3 §6.1/§5.6 | VERIFIED (read) |
| Reconnect mock support is available in Playwright 1.65 | not tested here | UNVERIFIED (H11 spike) |
| Server replay ordering/seq semantics | R3 §3.2 design only; WP-05 not implemented | UNVERIFIED / DESIGN |
| Current tests will pass after H1–H13 | not run | UNVERIFIED (run in implementation wave) |

**Non-claims:** this plan does not claim WP-06 is implemented, tested, or that reconnect works in the real app; it does not authorise the backend WP-05 edits; no build/test was executed.
