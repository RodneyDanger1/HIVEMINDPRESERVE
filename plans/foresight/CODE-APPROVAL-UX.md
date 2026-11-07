# CODE-APPROVAL-UX — TypeScript/React implementation blueprint

**Seat:** CODE RESEARCH PLANNER 2 · **Mode:** READ-ONLY for product code (this file is the only write; no source edits, no build, no test run)
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `37f0e7c wip(1.2): compile-green read-only tool loop, approval command/UI, 23-event contract, e2e tool/approval specs`
**Skills applied:** `/forgewright-lead` (read → root cause → blueprint → unverified split) and `/logic-gatekeeper` (Phase 1 below; no product mutation).
**Scope:** (a) pending-approval replay on reconnect/reload **without contract change**; (b) args preview **only if the contract patch is approved**; (c) retry/refusal state machine + removal of the stuck panel lock.
**Evidence basis:** files read in this session; all citations are `path:line`. No runtime claims beyond reads. Working tree contains in-flight backend changes (see §0.1).

---

## 0. Evidence base (READ, this session)

### 0.1 Repo state

- HEAD `37f0e7c`; dirty working tree: `backend/crates/hiverouter-core/src/{execution,http,provider_executor,recovery,tools_bridge}.rs`, `backend/crates/hiverouter-core/tests/{run_lifecycle,tool_loop_engine}.rs`, `backend/crates/hiverouter-tools/src/approval.rs`, plus untracked `plans/foresight/`.
- `plans/foresight/APPROVAL-UX-PLAN.md` (Lead research, 228 lines) defines items 10/11/15, options R1–R4, and the acceptance goals this blueprint implements.
- No `CHANGE-LEDGER-IMPL.md` was found by filename scan (`docs/**`): the frozen-contract rule in `docs/contracts/UI-COMMANDS-v1.md:1-16` names it; §6.2 flags how to satisfy it. **UNVERIFIED**: the ledger file's actual location/process may exist under another name.

### 0.2 Server truth

| Fact | Evidence |
|---|---|
| Approvals table: `approval_id, run_id, task_id, tool_call_id, tool_name, args_sha256, permission_class, created_at, expires_at, nonce UNIQUE, decision, decided_at, reason, executed_at` | `backend/crates/hiverouter-tools/src/approval.rs:141-165` |
| Domain request carries `nonce`, `args_sha256`, TTL; `is_expired(now)` exists | `approval.rs:83-117` |
| Read-only replay-safe projections exist: `pending_for_run` (decision NULL, oldest first) and `pending_approval_events(run_id)`; documented "never consumes a nonce … safe for UI replay" | `approval.rs:557-579`; `backend/crates/hiverouter-core/src/tools_bridge.rs:241-252,350-367` |
| Producer seam creates the committed row + full wire event (`nonce/run/task/tool_call/args/expiry`) | `tools_bridge.rs:260-348`; `approval.rs:771-812` |
| Resolution must consume the nonce exactly once; replay/expiry/not-found/decision-invalid are named refusals; ledger failure → ERROR (never fake ACK) | `approval.rs:301-520,668-752`; `tools_bridge.rs:198-239`; `ws.rs:465-483` |
| TTL reaper at boot marks TTL-elapsed pendings `decision='expired'`; recovery marks active runs `interrupted` (+`E_APPROVAL_TIMEOUT` for `awaiting_approval`) | `backend/crates/hiverouter-core/src/http.rs:107-130`; `recovery.rs:50-63` |
| **No production caller replays pending approvals**: `send_hello_and_replay` sends `SERVER_HELLO` + bus replay only; bus published events carry new seq | `backend/crates/hiverouter-core/src/ws.rs:132-152,225-295`; grep: `pending_approval_events` only in tests |
| Runs are persisted with the frozen status strings (`queued/planning/running/awaiting_approval/completed/failed/cancelled/killed/interrupted`) | `backend/crates/hiverouter-contract/src/state.rs:76-93`; `db.rs:734-736` |

### 0.3 Client truth

| Fact | Evidence |
|---|---|
| `ApprovalRecord` has `approvalId/tool/nonce/decision/requestedAt` + nullable `runId/taskId/toolCallId/argsSha256/expiresAt` | `web/src/state/store.ts:61-73` |
| `UiState` has `approvals` + `resolvingApprovals` (in-flight id list); `resolveApproval(approvalId, decision, reason?)` returns `Promise<void>` | `store.ts:160-164,183` |
| `APPROVAL_REQUEST` replace-by-id; `APPROVAL_RESOLVED` removes by id | `store.ts:484-509` |
| `run()` returns `boolean` and drops the typed refusal (only toast); ERROR frame handling | `store.ts:303-309,565-596`; `ws.ts` refusals `web/src/api/client.ts:370-389` |
| Client commands resolve on ACK, reject on typed ERROR/timeout; default command timeout **15000 ms**; reconnect uses Partysocket + a hello control frame with `last_seq` | `web/src/api/client.ts:199,299-315,431-477` |
| `resolveApproval` guards duplicates via `resolvingApprovals`, requires nonce, clears the guard in `finally`, never optimistically mutates | `store.ts:739-777` |
| Overlay keeps a **panel-local `pendingResolution` lock** pruned only when the approval disappears; both actions disabled from `canResolveApproval(nonce, resolving, expired)`; renders resolving line only while in-flight | `web/src/panels/ApprovalOverlay.tsx:163-215,290-319` |
| Helpers: `formatExpiry` (unknown/expired sentinels), `isApprovalExpired`, `canResolveApproval`, `prunePendingResolutions`, `truncateId`, `approvalAriaLabel`, `approvalActionAriaLabel` | `ApprovalOverlay.tsx:37-153` |
| Strings: `STRINGS.approval.{title,toolLabel,resolving,resolveHint,fields,expiry,unknownValue,nonceMissing}`; `STRINGS.controls.{approve,deny,retry,dismiss}` | `web/src/state/strings.ts:95-145,244-267` |
| Contract `ApprovalRequestPayload` optional 1.2 fields only; no `args_preview`; decoder null-safe per optional field | `web/src/api/contract.ts:223-239,693-723` |
| Frozen contract rules: any name change needs a change-ledger row + matrix test; `APPROVAL_RESOLVE` refusals list; event count 23 | `docs/contracts/UI-COMMANDS-v1.md:1-16,37,54-65` |

### 0.4 Tests today

| Fact | Evidence |
|---|---|
| Store/panel approval suite: extended fields, expiry gating, ACK-keeps-lock, nonce-less, malformed, mount | `web/tests/approval.integration.test.ts:112-275` |
| View-model suite: truncation, TTL formatting, disabled gate, labels | `web/tests/toolActivityPanels.test.ts:36-70` |
| Static render/mount suite (mine): selector pinning, expiry/nonce disabled counts | `web/tests/agentsToolActivityMount.test.ts:231-285` |
| E2E (mock WS): render-no-send, approve/ACK-stays-open, double-click, deny+reason, duplicate-id, nonce-less, expired read-only, malformed | `e2e/specs/approval-flow.spec.ts:114-269` |
| Mock harness: `installMockWs` (auto-ACK COMMANDs, `route()/sent/sentCommands/waitForCommand/sendRaw`), `sendApprovalRequest`, `eventFrame`, `startDistHarness` (`/api/session` last_seq 0) | `e2e/helpers/replay-harness.ts:39-108,110-140,255-261,281-305,312-390` |
| **Missing coverage** (the gap this blueprint fills): reconnect/reload replay, ERROR-after-gesture retry, ACK-only stall, args preview privacy | `APPROVAL-UX-PLAN.md:28,185-191` |

---

## 1. Phase 1 (logic-gatekeeper record)

```json
{
  "phase1": {
    "architecture": "Server truth: hiverouter-tools approval.rs owns the approvals table + nonce/TTL/resolution; core tools_bridge.rs exposes read-only pending_approvals / pending_approval_events projections; core ws.rs owns SERVER_HELLO + bus fan-out; web store.ts owns approvals/resolvingApprovals and is fed by the contract decoder; ApprovalOverlay.tsx owns resolve rendering + a panel-local pendingResolution lock.",
    "dataFlow": "Untrusted input: WS EVENT payloads (APPROVAL_REQUEST/APPROVAL_RESOLVED/ERROR/SERVER_HELLO/RESYNC_REQUIRED) and operator clicks. They reach the strict contract decoder, then store.approvals/resolvingApprovals, then the overlay's canResolveApproval gate and resolveApproval() -> one APPROVAL_RESOLVE command. Replay input additionally flows server DB pending rows -> pending_approval_events -> bus -> same decoder/store path, and must never auto-send.",
    "edgeCases": [
      "reconnect/reload: a pending request appeared, socket drops, no APPROVAL_RESOLVED arrived; after reconnect the same approval_id is replayed exactly once and stays actionable; an approval resolved while away is not replayed and must not stay actionable",
      "ACK-only: command gets ACK but APPROVAL_RESOLVED never arrives; the current panel-local pendingResolution lock stays true forever and the card is permanently disabled",
      "typed ERROR after a gesture: store resolvingApprovals clears in finally but the panel lock persists (stuck card) — the retry state machine must clear it and classify retryable vs stale vs expired",
      "args_preview absent (current contract): UI must show hash-only honest copy, never reconstruct args client-side",
      "args_preview present but malformed/wrong type after a contract patch: decoder treats it as absent, never fabricates text",
      "page reload/new tab with no persistence: replay must be server-driven; no nonce at rest"
    ],
    "rootCause": "The store has no reconnect-replay or per-approval resolve state machine: pending approvals are lost on reload and can be starved by an ACK/ERROR because the overlay's local lock is released only by APPROVAL_RESOLVED, while the server already exposes read-only pending projections and the frozen APPROVAL_REQUEST event that could replay them without any contract change; the code blueprint must add those states/actions, remove the local lock, and specify the args-preview contract patch as the only path to showing argument bodies."
  }
}
```

**Why the contradiction fails closed after implementation (Phase 2 statement):** a replay can only re-deliver an existing, still-pending `APPROVAL_REQUEST` (read-only projection; no nonce consumption, no auto-command); an unconfirmed or stale record is rendered read-only and dismissable; every failure of a resolve gesture is classified from the existing frozen refusal code and either retried with the same single-use nonce (bounded) or made read-only; and the panel lock is deleted, so no ACK/ERROR path can wedge the card.

---

## 2. Non-negotiable invariants (all items)

1. **No auto-send.** Hydration/replay/settle never calls `APPROVAL_RESOLVE`; only an explicit operator gesture does.
2. **No optimistic removal/decision.** The card closes only on `APPROVAL_RESOLVED` (server truth) or explicit local `dismissApproval` for read-only states.
3. **Single-use nonce.** Retry reuses the same `approval_id`+`nonce`; a consumed/expired/decided nonce is never retried (`E_APPROVAL_REPLAY|E_APPROVAL_TIMEOUT|E_APPROVAL_NOT_FOUND` → no retry).
4. **No client-side args reconstruction.** Only `args_sha256` today; preview text only from the server when the contract patch lands.
5. **No persistence of approval state/nonce** in localStorage/sessionStorage in this wave (privacy default; the R2 fallback stays off).
6. **No new event/command/refusal code without a contract patch.** (a) and (c) reuse the frozen vocabulary; (b) is additive/optional.
7. **Bounded everything:** replay batch ≤20, bounded retries (3), bounded strings (preview ≤2048 chars), no unbounded maps (resolve states pruned with records).

---

## 3. Item (a) — pending-approval replay on reconnect/reload, no contract change

### 3.1 Chosen design (R1 from the UX plan, refined with a deterministic window)

- **Server:** after `SERVER_HELLO` is sent on a connection, publish one frozen `APPROVAL_REQUEST` per committed, unexpired, undecided approval whose run is non-terminal (bounded 20, oldest first) through the existing `EventBus`. This is the same event with the same six optional fields — **no contract change**.
- **Client:** on `reconnecting`/`RESYNC_REQUIRED` mark approvals `stale`; on `SERVER_HELLO` open a short replay window and stamp every `APPROVAL_REQUEST` received in it with the current epoch; after `APPROVAL_REPLAY_SETTLE_MS` settle: confirm replayed ids as actionable, mark never-reconfirmed pre-reconnect ids `unconfirmed` (read-only, dismiss-only). **No pruning** is required for safety: an approval resolved while the client was away is simply not replayed → becomes `unconfirmed` (cannot be actioned); if the user dismisses it locally only, the server row is untouched; if replay is unsupported by an old backend, pre-existing approvals become read-only after a reconnect rather than wrong-actionable — fail-closed.
- **Reload/new tab:** fresh store has no approvals; replay arrives after hello and renders as pending. No persistence.
- **Restart:** boot reaper marks TTL-elapsed rows `expired`, recovery interrupts awaiting runs; the pending projection excludes `decision IS NOT NULL` and expired rows → **nothing is replayed after a process restart** (no resurrection).
- **Multi-client:** bus broadcast delivers the replay to every subscribed socket; replace-by-id upsert makes this idempotent; no auto-send anywhere.
- Why not hello-before/after ordering barrier: the server sends `SERVER_HELLO` directly and replays via the same bus the socket already subscribes to (`ws.rs:132-152`), so ordering `hello → replays` is deterministic but there is **no "replay complete" frame** in the frozen contract; a time window is the only contract-free barrier. It gates display (read-only until confirmed), never writes.

### 3.2 Backend changes (exact)

New crate-internal helper in `backend/crates/hiverouter-tools/src/approval.rs` (after `pending_for_run`):

```rust
/// Every committed, unexpired, undecided approval whose run is still non-terminal,
/// oldest first, bounded. Read-only: never consumes a nonce, resolves, executes or
/// ledgers. `load_request` below is made `pub` so the core owner can project rows
/// for all runs without widening this module's run/table coupling.
pub fn pending_ids_for_active_runs(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<String>, ToolError> {
    let capped = i64::try_from(limit).unwrap_or(i64::MAX);
    let mut statement = conn.prepare(
        "SELECT a.approval_id
           FROM approvals a
           JOIN runs r ON r.id = a.run_id
          WHERE a.decision IS NULL
            AND r.status NOT IN ('completed','failed','cancelled','killed','interrupted')
          ORDER BY a.created_at ASC
          LIMIT ?1",
    )?;
    let ids = statement
        .query_map(params![capped], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

// change: `fn load_request` -> `pub fn load_request` (same body/behaviour;
// existing internal callers keep working, no signature change).
```

New core projection in `tools_bridge.rs` (near `pending_approval_events`):

```rust
/// Replay-safe `APPROVAL_REQUEST` payloads for every committed, unexpired,
/// undecided approval of a non-terminal run, oldest first, bounded by `limit`.
/// Read-only: no nonce consumption, no execution, no ledger write.
pub fn pending_approval_events_all(
    &self,
    limit: usize,
) -> Result<Vec<event::ApprovalRequest>, Refusal> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let now_unix_secs = Timestamp::now().unix_secs().unwrap_or(0);
    self.db.with_connection_refusal(|conn| {
        let ids = hiverouter_tools::approval::pending_ids_for_active_runs(conn, limit)
            .map_err(|error| error.refusal())?;
        let mut events = Vec::with_capacity(ids.len());
        for id in ids {
            let Some(request) = hiverouter_tools::approval::load_request(conn, &id)
                .map_err(|error| error.refusal())?
            else {
                continue; // row vanished between query and load: never fabricate
            };
            if request.is_expired(now_unix_secs) {
                continue; // TTL elapsed before the reaper ran: never replay it
            }
            events.push(Self::approval_request_event(&request));
        }
        Ok(events)
    })
}
```

In `ws.rs`, add one module helper and call it from **both** hello paths:

```rust
const MAX_APPROVAL_REPLAY_EVENTS: usize = 20;

/// Publishes the bounded replay batch after SERVER_HELLO. The socket is already
/// subscribed to the bus (session_loop line 133), so the events are delivered to
/// this connection's loop immediately after hello and to other clients as
/// idempotent replace-by-id upserts. Never publishes on a read-only session.
fn publish_pending_approval_replays(state: &Arc<AppState>) -> usize {
    match state.tools.pending_approval_events_all(MAX_APPROVAL_REPLAY_EVENTS) {
        Ok(events) => {
            let mut published = 0_usize;
            for approval in events {
                if state.bus.publish(Event::ApprovalRequest(approval)).is_ok() {
                    published += 1;
                }
            }
            if published > 0 {
                tracing::info!(published, "replayed pending approvals after hello");
            }
            published
        }
        Err(error) => {
            // Approval replay is best-effort: the session still works, the UI
            // shows a stale/unconfirmed card instead of a fabricated approval.
            tracing::warn!(code = error.code(), "pending-approval replay refused");
            0
        }
    }
}

async fn send_hello_replay_and_approvals(
    socket: &mut WebSocket,
    state: &Arc<AppState>,
    reported: Option<u64>,
) -> Result<(), ()> {
    send_hello_and_replay(socket, &state.bus, reported).await?;
    let _ = publish_pending_approval_replays(state);
    Ok(())
}
```

Call-site changes (exact): `ws.rs:147-152` and `ws.rs:316-318` become `send_hello_replay_and_approvals(&mut socket, &state, reported/last_seq).await`; the initial-hello handler also currently passes `reported`; keep `query_last_seq` semantics unchanged. The existing `send_hello_and_replay` keeps its behaviour (unchanged tests).

**Ordering guarantee (READ):** `session_loop` subscribes the bus *before* the hello exchange (`ws.rs:133`), and `send_hello_and_replay` awaits the direct hello send before returning, so the replay is always emitted after `SERVER_HELLO` on the socket's outbound path.

### 3.3 Client state (exact fields/actions/selectors)

New pure module `web/src/state/approval.ts` (state machine + replay reducer; no JSX):

```ts
import type { ApprovalRequestPayload, Refusal } from '../api/contract';
import type { ApprovalRecord } from './store';

/** Time window after SERVER_HELLO during which APPROVAL_REQUEST frames are attributed to replay. */
export const APPROVAL_REPLAY_SETTLE_MS = 400;
/** Bounded replay batch mirror of the server constant (diagnostics/tests only). */
export const MAX_APPROVAL_REPLAY_EVENTS = 20;
/** Manual retry attempts per approval (nonce stays single-use). */
export const MAX_APPROVAL_RESOLVE_ATTEMPTS = 3;
/** ACK→APPROVAL_RESOLVED settlement budget before a retry is offered. */
export const APPROVAL_ACK_SETTLE_MS = 2_000;
/** Explicit command timeout for APPROVAL_RESOLVE (matches WsApiClient default, named). */
export const APPROVAL_RESOLVE_TIMEOUT_MS = 15_000;

export type ApprovalReplaySource = 'hello' | 'session';
export type ApprovalDisplayStatus =
  | 'pending' | 'resolving' | 'awaiting_server' | 'retryable'
  | 'expired' | 'stale' | 'unconfirmed';
export type ApprovalFailureKind = 'transport' | 'operator' | 'stale' | 'expired' | 'terminal' | 'ignored';

export interface ApprovalResolveState {
  readonly status: ApprovalDisplayStatus;
  readonly attempts: number;                         // manual gestures only, 0..3
  readonly lastDecision: 'approve' | 'deny' | null;
  readonly refusal: Refusal | null;                  // exact named code/message/how_to_fix
  readonly lastAttemptAt: number | null;
}

export type ApprovalResolveOutcome =
  | { readonly ok: true }
  | { readonly ok: false; readonly kind: ApprovalFailureKind; readonly refusal: Refusal };

export function initialApprovalResolveState(): ApprovalResolveState;
export function classifyApprovalRefusal(refusal: Refusal): {
  readonly kind: ApprovalFailureKind;
  readonly status: ApprovalDisplayStatus;
  readonly retryable: boolean;
};
export function approvalDisplayStatus(
  record: ApprovalRecord,
  resolve: ApprovalResolveState | undefined,
  nowMs: number,
): ApprovalDisplayStatus;
export function approvalActionsEnabled(args: {
  readonly record: ApprovalRecord;
  readonly resolve: ApprovalResolveState | undefined;
  readonly nowMs: number;
  readonly connection: ConnectionStatus; // from web/src/api/client.ts
}): { readonly approve: boolean; readonly deny: boolean; readonly retry: boolean; readonly dismiss: boolean };
```

Classifier matrix (exact, from the frozen refusal list `web/src/api/contract.ts:398-421`, `docs/contracts/UI-COMMANDS-v1.md:37`):

| refusal code | kind | next status | retry allowed |
|---|---|---|---|
| `E_WS_DISCONNECTED`, `E_RESYNC_REQUIRED`, client timeout refusal | `transport` | `retryable` | yes |
| `E_APPROVAL_DECISION_INVALID`, `E_SCHEMA_INVALID` | `operator` | `retryable` | yes (same nonce not consumed by validation refusal) |
| `E_DB_BUSY`, `E_INTERNAL_ERROR` | `transport` | `retryable` | yes (bounded) |
| `E_APPROVAL_REPLAY`, `E_APPROVAL_NOT_FOUND` | `stale` | `stale` | **no** |
| `E_APPROVAL_TIMEOUT`, `E_APPROVAL_REQUIRED` | `expired` | `expired` | **no** |
| any other code | `operator` | `retryable` | yes (bounded; card shows the named code) |
| duplicate gesture while `resolving`/`awaiting_server` | `ignored` | unchanged | no |

Availability truth table (replaces the overlay-local `canResolveApproval` disabled logic for cards):

| status | nonce | expired (client clock) | connection | approve/deny | retry | dismiss |
|---|---|---|---|---|---|---|
| `pending` | present | false | `open` | **enabled** | — | — |
| `pending` | present | false | not `open` | disabled | — | — |
| `pending` | null | any | any | disabled (named gap copy) | — | — |
| `resolving` | any | any | any | disabled | — | — |
| `awaiting_server` | any | any | any | disabled | — | — |
| `retryable` | present | false | `open` | disabled | **enabled** (<3 attempts) | — |
| `retryable` | present | false | not `open` | disabled | disabled | — |
| `expired` | any | true | any | disabled | — | **enabled** |
| `stale` / `unconfirmed` | any | any | any | disabled | — | **enabled** |

`approvalDisplayStatus` precedence: `resolve.status` for `resolving|awaiting_server|retryable|stale|unconfirmed`; else `expired` if client-clock expired; else `pending`.

Store additions (exact; `web/src/state/store.ts`):

```ts
// UiState additions (after `resolvingApprovals`, before toolActivity)
readonly approvalResolve: Readonly<Record<string, ApprovalResolveState>>;
readonly approvalSeenEpoch: Readonly<Record<string, number>>;
readonly approvalsStale: boolean;
readonly approvalReplayEpoch: number;
readonly approvalReplayWindow: boolean;
readonly lastApprovalReplayAt: number | null;
readonly approvalReplaySource: ApprovalReplaySource | null;

// Actions
hydrateApprovals(events: readonly ApprovalRequestPayload[], source: ApprovalReplaySource): void;
markApprovalsStale(): void;
beginApprovalReplay(source: ApprovalReplaySource): void;
settleApprovalReplay(): void;
expireApproval(approvalId: string): void;
dismissApproval(approvalId: string): void;
retryApproval(approvalId: string, decision?: 'approve' | 'deny', reason?: string): Promise<ApprovalResolveOutcome>;
// changed return type (call sites updated):
resolveApproval(approvalId: string, decision: 'approve' | 'deny', reason?: string): Promise<ApprovalResolveOutcome>;
```

Selectors (exported from `approval.ts`, wired with `useUiStore` in components; keep store reads single-field):

```ts
export const selectApprovals = (s: UiState) => s.approvals;
export const selectApprovalsStale = (s: UiState) => s.approvalsStale;
export const selectApprovalResolve = (id: string) => (s: UiState) => s.approvalResolve[id];
export const selectApprovalResolveMap = (s: UiState) => s.approvalResolve;
```

Reducer transitions (store `applyEvent`/status/timer events; exact):

| trigger | transition |
|---|---|
| `subscribeStatus('reconnecting')` or `RESYNC_REQUIRED` | `approvalsStale=true`; `approvalReplayEpoch+=1`; `approvalReplayWindow=false`; clear settle timer; keep records |
| `SERVER_HELLO` while `approvalsStale` or after a fresh connect | `beginApprovalReplay('hello')`: `window=true`; `lastApprovalReplayAt=Date.now()`; arm `setTimeout(settleApprovalReplay, APPROVAL_REPLAY_SETTLE_MS)`; `connection='open'` as today |
| `APPROVAL_REQUEST` while `approvalReplayWindow` | upsert by id; `approvalSeenEpoch[id]=approvalReplayEpoch`; if status ∈ {`unconfirmed`,`stale`,`expired`} and new `expires_at` is future → status `pending`, refusal null; preserve `resolving`/`awaiting_server`; never re-arm timers |
| `APPROVAL_REQUEST` outside window (live arrival) | upsert by id; if existing status ∈ {`resolving`,`awaiting_server`} preserve it; otherwise status `pending` (fresh nonce/expiry); `approvalSeenEpoch[id]=approvalReplayEpoch` |
| `APPROVAL_RESOLVED` | remove approval + `approvalResolve[id]` + `approvalSeenEpoch[id]`; clear this id's settle timer; `resolvingApprovals` projection removes id |
| settle timer fires | `window=false`; `stale=false`; for each record: if `approvalSeenEpoch[id] === approvalReplayEpoch` → ensure status not `unconfirmed`; else if status ∈ {`pending`,`retryable`} → `unconfirmed` (read-only) |
| `expireApproval(id)` (overlay timer) | if status ∈ {`pending`,`retryable`} → `expired`; clear attempts? no (display only) |
| `dismissApproval(id)` | allowed for `expired`,`stale`,`unconfirmed`; removes record + resolve + seen epoch locally (no command; server row unchanged) |
| `dispose()` (existing line 659) | clear the settle timer; no persistence to clear |

`hydrateApprovals(events, source)` is the public/testable form: dedupe by `approval_id`, call the same upsert rule with the current epoch, and set `approvalReplaySource=source`/`lastApprovalReplayAt=Date.now()`. It never sends commands.

**Timers:** one settle `setTimeout` handle owned by the store closure (cleared on `beginApprovalReplay`, `settle`, `dispose`); per-approval ACK timers live in `approvalResolve` handling (see §5.4) inside a `Map<string, ReturnType<typeof setTimeout>>` closure, cleared on resolve/removal/dispose. Overlay keeps its 1 s display interval for TTL text only.

### 3.4 UI behavior (replay)

- While `approvalsStale` (between reconnect and settle) the card renders a `role="status"` banner `STRINGS.approval.replayPending` and both actions are disabled (availability table: connection not `open` plus status handling).
- A replayed card appears exactly once (replace-by-id; no dialog stacking).
- After settle, never-reconfirmed cards show `STRINGS.approval.status.unconfirmed` strip, both actions disabled, `Dismiss` only.
- No storage writes anywhere; no `title`/`data-*` attributes carry nonce (existing rendering already only carries `title` for ids/hash, never nonce).

### 3.5 Strings (exact additions under `STRINGS.approval`)

```ts
replayPending: 'Reconnected — waiting for the server to replay pending approvals.',
status: {
  resolving: 'Resolving approval…',
  awaiting: 'Waiting for the server to confirm the decision…',
  retryable: 'Decision not confirmed — retry is safe (the nonce is single-use).',
  expired: 'Expired — no action was taken.',
  stale: 'Already decided or no longer pending — dismiss this card.',
  unconfirmed: 'Reconnected — the server did not re-confirm this approval; it is read-only.',
},
retryLabel: 'Retry approval',        // button aria-label prefix, tool appended
dismissLabel: 'Dismiss approval',    // button aria-label prefix, tool appended
refusalTitle: 'Approval decision refused',
```

### 3.6 Tests — item (a)

Unit (`web/tests/approval.state.test.ts`, new; pure, no render):
1. `beginApprovalReplay` sets window/epoch/source and `settleApprovalReplay` clears them.
2. hydration dedupes by id across two batches; second batch updates nonce/expiry but never resets `resolving`/`awaiting_server`.
3. hydration stamps `approvalSeenEpoch`; a pre-existing record not re-confirmed becomes `unconfirmed` after settle; a replayed one stays `pending`.
4. hydration never appends a command to a `FakeUiApi` (no send).
5. `APPROVAL_REQUEST` for an id already resolved (no record) is added only if replayed; no `APPROVAL_RESOLVE` occurs.
6. `dispose()` clears the settle timer (`vi.useFakeTimers`; advancing time after dispose changes nothing).

Store integration (`web/tests/approval.integration.test.ts`, extend existing harness): `FakeUiApi` gains the exact members below; cases 7-10 follow.

```ts
// add to class FakeUiApi (approval.integration.test.ts:43-100); import RefusalError from '../src/api/refusals'
emitStatus(status: ConnectionStatus, detail?: Refusal): void {
  for (const listener of this.statuses) listener(status, detail);
}
rejectNext(refusal: Refusal): void {
  const pending = this.sends.shift();
  pending?.reject(new RefusalError(refusal));
}
hello(lastSeq: number): void {           // helper for replay tests: emit a SERVER_HELLO event
  this.emit({ name: 'SERVER_HELLO', payload: { last_seq: lastSeq, schema: SCHEMA_VERSION } });
}
```

7. emit `reconnecting` → `approvalsStale=true`; emit `SERVER_HELLO` then replayed `APPROVAL_REQUEST` → after settle record pending/actions logic unavailable in store, assert `approvalSeenEpoch` and `approvalsStale=false`, `approvalReplaySource='hello'`.
8. record present before reconnect, not replayed → after settle `approvalResolve[id].status === 'unconfirmed'`; `dismissApproval` removes it; no commands sent.
9. `APPROVAL_RESOLVED` during the window removes record and clears `approvalResolve`.
10. hydration with malformed optional fields (missing nonce/expiry, wrong types dropped by decoder) never fabricates.

E2E (`e2e/specs/approval-flow.spec.ts`, extend; harness helpers in §3.7): cases 11-13:
11. **Reconnect replay, exactly once:** open card; `await closeMockSocket(ws)`; `await waitForReconnect(ws)` (new hello captured); `sendServerHello(ws, {last_seq: 12, schema: SCHEMA_VERSION}, 12)`; `sendApprovalRequest(ws, payload, 13)`; expect exactly one `getByRole('dialog', {name:/file_write/})`, both actions enabled, `approvalResolveCommands()` length 0; banner disappears.
12. **Resolved while away is not actionable:** close, reconnect + hello, do not replay; after the settle window the card is read-only (`button[aria-label="Approve tool call for file_write"]` disabled), `Dismiss approval for file_write` visible, zero commands.
13. **No auto-send on reload:** `page.reload()`; wait Connected; send hello + replay; card appears, actions enabled, `approvalResolveCommands()` length 0.

### 3.7 E2E helper additions (`e2e/helpers/replay-harness.ts`; exact signatures)

```ts
/** Closes the currently routed page-side socket; the app's ReconnectingWebSocket reconnects. */
export async function closeMockSocket(ws: MockWsHarness): Promise<void>;
/** Resolves once a NEW hello control frame arrived after a close (count increases). */
export async function waitForReconnect(ws: MockWsHarness, priorHelloCount: number, timeoutMs?: number): Promise<WebSocketRoute>;
/** Number of hello control frames captured so far (reconnect diagnostics). */
export function helloFrameCount(ws: MockWsHarness): number;
/** Sends a validated SERVER_HELLO event on the current route (reconnect tests). */
export function sendServerHello(ws: MockWsHarness, payload: { last_seq: number; schema: string }, seq: number): void;
/** Sends an ERROR frame correlated to a previously captured COMMAND (retry tests). */
export function sendErrorForCommand(ws: MockWsHarness, command: MockSentCommand, code: string, seq: number): void;
```

Implementation notes: `MockWsHarness` gains `readonly helloCount: () => number`; the route handler increments on `type:"hello"` control frames; `sendErrorForCommand` emits `{"schema","type":"EVENT","name":"ERROR","correlation_id": command.correlationId ?? fallback, "seq", "ts", "payload":{code,message,how_to_fix,retryable}}` using `clientRefusal`-shaped copy from `web/src/state/strings.ts`? No — e2e is outside browser; build the payload from the frozen code table with explicit `message/how_to_fix/retryable` fixtures (documented in spec).

---

## 4. Item (b) — args preview, contract patch required

### 4.1 Without a contract patch (ships now)

The UI cannot show argument values: `APPROVAL_REQUEST` has no body and `TOOL_CALL` only carries `args_sha256` (`contract.ts:223-239`; `docs/contracts/UI-COMMANDS-v1.md:62-65`). Current honest copy stays and gains a cross-check affordance:

- `STRINGS.approval.argsBoundByHash = 'Arguments are bound by args_sha256; the approval request carries no argument bodies. Compare the hash and tool_call_id with the tool activity panel before deciding.'`
- Render it under the `args_sha256` row (plain text; existing `<dd title>` full hash stays).
- **Forbidden:** client-side reconstruction from model JSON, showing `args_sha256` as if it were args, any "preview" fabricated from `tool_call_id`.
- Test: card contains the hash-binding copy and no `approval-args-preview` element while `args_preview` is absent.

### 4.2 Contract patch delta (additive, optional; same pattern as the 1.2 additive fields)

**Wire payload** (both Rust and TS must match; no new event/command/refusal):

```rust
// backend/crates/hiverouter-contract/src/event.rs
/// Server-redacted, plain-text argument preview for an approval request.
/// Bounded (<= 2048 chars) and produced only by the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ArgsPreview {
    /// Redacted, plain-text rendering (never raw argument bytes).
    pub text: String,
    /// Producer truncated the preview to the bound.
    pub truncated: bool,
    /// Producer masked at least one secret-shaped value.
    pub redacted: bool,
}

pub struct ApprovalRequest {
    // ... existing fields unchanged ...
    /// 1.3 additive: server-redacted args preview; absent on old producers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args_preview: Option<ArgsPreview>,
}
```

```ts
// web/src/api/contract.ts
export interface ApprovalArgsPreview {
  readonly text: string;
  readonly truncated: boolean;
  readonly redacted: boolean;
}
export interface ApprovalRequestPayload {
  // ... existing unchanged ...
  /** 1.3 additive: server-redacted preview; absent means "not provided". */
  readonly args_preview?: ApprovalArgsPreview | null;
}
```

Decoder rule (in the existing `APPROVAL_REQUEST` arm, after `expiresAt`): accept only a well-formed object with `text: string`, `truncated: boolean`, `redacted: boolean` (enforce `text.length <= 2048`); anything else → `args_preview: null` (never a decoder refusal, matching the current null-safe optional-field policy; a wrong-typed optional field cannot fabricate data). `args_sha256` remains the binding.

**Schema/contract process (exact steps for the owner):**
1. `UPDATE_GOLDEN=1 cargo test -p hiverouter-contract golden_schemas` then run the golden test normally (`backend/crates/hiverouter-contract/tests/golden_schemas.rs:7-9`).
2. Add the additive row to `docs/contracts/UI-COMMANDS-v1.md` (§3 events + new "1.3 additive" note; the file requires a change-ledger row for changes — create/update the ledger named in line 5 and record: field, producer, optionality, compat).
3. Update `backend/crates/hiverouter-contract/tests/common/mod.rs` wire samples + `web/tests/contract.test.ts` decode test (add preview-present and preview-absent cases; event count stays 23).
4. AgentState/command/refusal vocabularies unchanged.

**Persistence (server must replay the same preview):** approvals need the preview at replay time, and the DB stores only the hash today (`approval.rs:141-165`). Add one tools migration step:

```sql
-- idempotent guard in approval::migrate (after the CREATE TABLE batch):
ALTER TABLE approvals ADD COLUMN args_preview TEXT; -- JSON: {"text","truncated","redacted"}
```

Exact implementation: `ApprovalRequest` gains `pub preview: Option<Preview>` (`Preview` serde struct = wire shape); `migrate` checks `pragma_table_info('approvals')` for the column and only then `ALTER TABLE`; `insert_request` writes `serde_json::to_string(preview)` (NULL when none); `load_request` parses the nullable JSON (malformed stored row → `E_INTERNAL_ERROR`, never a fabricated preview); `approval_request_event` maps it to the wire field.

**Producer redaction (server only; new module `backend/crates/hiverouter-tools/src/preview.rs`):**

```rust
pub const MAX_PREVIEW_CHARS: usize = 2048;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preview { pub text: String, pub truncated: bool, pub redacted: bool }

/// Server-redacted, plain-text preview of the arguments the approval binds.
/// Rules: plain text only; never log/ledger; <= MAX_PREVIEW_CHARS; report flags.
pub fn redact_args_preview(tool: ToolId, args: &serde_json::Value) -> Preview;
```

Redaction rules (implement + test): mask secret-shaped values (`sk-[A-Za-z0-9_-]{8,}`, `Bearer <token>`, `Authorization:`, values of keys matching `(?i)password|token|secret|api[_-]?key`, JWT-shaped triples, `KEY=VALUE` env assignments) with `[redacted]` and set `redacted=true`; per tool: `file_write` → path + content length + content sha256 (never content), `shell_exec` → program + argv with masking, `spawn_agent`/`network` → key names + types, unknown tool → top-level key names only; serialize as stable `key: value` lines (sorted keys, deterministic); truncate at a UTF-8 char boundary (≤2048) and set `truncated`; empty args → `{ text: "(empty arguments)", truncated:false, redacted:false }`.
Wiring: `request_model_tool_approval` computes `let preview = redact_args_preview(id, args);` and passes it into `ApprovalGate::request(..., preview: Option<Preview>)` (signature change; tests updated) → stored with the row → returned event carries it. No raw args are stored.

### 4.3 Store + UI (exact)

`ApprovalRecord` additions (flattened per the UX plan, memory-only):

```ts
readonly argsPreviewText: string | null;      // server-redacted plain text
readonly argsPreviewRedacted: boolean;
readonly argsPreviewTruncated: boolean;
```

Populated in the `APPROVAL_REQUEST` reducer + `hydrateApprovals` from `event.payload.args_preview` (null when absent; enforce `length <= 2048` at the store boundary too, dropping over-long payloads). Never written to storage.

Component (new `web/src/panels/ApprovalArgsPreview.tsx`):

```ts
export interface ApprovalArgsPreviewProps {
  readonly text: string;          // already server-redacted
  readonly truncated: boolean;
  readonly redacted: boolean;
  readonly boundHash: string;     // full args_sha256, shown as the binding line
}
export function ApprovalArgsPreview(props: ApprovalArgsPreviewProps): JSX.Element;
```

DOM contract: `<section aria-labelledby="approval-args-preview-label" data-testid="approval-args-preview">` + `<h3 id="approval-args-preview-label">STRINGS.approval.argsPreview.label</h3>` + `<pre>` with React text interpolation only (no `dangerouslySetInnerHTML`) + labels `redacted`/`truncated` when flags + hash-binding line `args_sha256 <full>`. Max height with `overflow-auto`, `whitespace-pre-wrap`. When `argsPreviewText === null` render the §4.1 hash-only copy (no empty `<pre>`).

Privacy rules (must be enforced in review + tests): never in `title`, `data-*`, URL, `localStorage`/`sessionStorage`, console, toasts, trace/evidence captures, ledger. Do not add a copy button for the preview (hash copy only). A build-time flag `VITE_UI_ARGS_PREVIEW` (default `'1'` once the contract ships; `'0'` renders hash-only) gates the section during rollout.

### 4.4 Tests — item (b)

Backend (`hiverouter-tools` unit tests): redaction fixtures for `sk-…`, `Bearer …`, `Authorization:`, `password=…`, `token: …`, JWT, `API_KEY=…` → none present in `preview.text`, `redacted=true`; 3 KiB input → `truncated=true` and `text.chars().count() <= 2048`; per-tool policy for `file_write` (no content bytes) and `shell_exec`; empty args; deterministic output for the same args; stored preview round-trips through insert/load; `approval_request_event` carries the same preview; replay projection carries it.
Contract: golden schema test passes after deliberate regeneration; decode tests for present/absent/malformed.
Store: `APPROVAL_REQUEST` with/without preview populates/omits the flattened fields; over-long text dropped; hydration preserves preview.
UI (static render, node env): preview present → `<pre>` text exactly the redacted text, flags visible, no secret substring in the whole markup, `title`/`data-*` attributes do not contain the preview; preview absent → hash-only copy and no `[data-testid="approval-args-preview"]`.
E2E (after patch): fixture approval with a preview containing `sk-live-…` and `Bearer abc`; assert visible text contains no secret, contains `[redacted]`; assert `page.evaluate(() => JSON.stringify(localStorage) + JSON.stringify(sessionStorage))` has no preview; assert console messages contain no preview; absent-case keeps hash-only copy.

---

## 5. Item (c) — retry/refusal state machine; delete the stuck panel lock

### 5.1 Defect proof (deterministic, from code)

1. Operator clicks: overlay sets `pendingResolution[id]=true` (`ApprovalOverlay.tsx:203-215`), store appends `resolvingApprovals` and sends (`store.ts:764-771`).
2. Server ERROR (e.g. `E_APPROVAL_REPLAY`/`E_APPROVAL_TIMEOUT`): `client.ts:370-389` rejects the command promise; store `finally` clears `resolvingApprovals` (`store.ts:772-776`); the approval row stays in `approvals` (only `APPROVAL_RESOLVED` removes it).
3. Overlay's `pendingResolution` survives: it is pruned only when the id leaves `approvals` (`ApprovalOverlay.tsx:181-190`). Result: both actions stay disabled forever; the card is stuck (only page reload recovers, and today a reload loses the card entirely — item (a) fixes that).
4. ACK-only: store clears the guard on ACK, panel keeps the lock "until APPROVAL_RESOLVED" — if that event is lost, the same permanent disable (the e2e only checks that the card stays open, not recovery).

### 5.2 New state machine (store-owned; per approval)

```
pending (nonce present, not expired, actions enabled)
  ├─ gesture (connection open, attempts < 3) ──▶ resolving   [send APPROVAL_RESOLVE; resolvingApprovals projection +]
  │      ├─ ACK ─────────────────────────────▶ awaiting_server [ACK settle timer armed]
  │      │     ├─ APPROVAL_RESOLVED ─────────▶ removed / closed
  │      │     └─ timer(2000ms) fires ───────▶ retryable (refusal = E_WS_DISCONNECTED synthetic, named copy)
  │      └─ rejection/refusal ───────────────▶ classify:
  │             transport/operator/db ───────▶ retryable  [lock cleared; Retry enabled, attempts preserved]
  │             E_APPROVAL_REPLAY/NOT_FOUND ─▶ stale      [read-only, Dismiss]
  │             E_APPROVAL_TIMEOUT/REQUIRED ─▶ expired    [read-only, Dismiss]
  └─ TTL timer (expiresAt <= now) ──────────▶ expired [read-only, Dismiss]

retryable ── Retry gesture (connection open, attempts < 3) ──▶ resolving (same id/nonce, new correlation id)
retryable ── attempts == 3 ──────────────────────────────────▶ terminal (no send; strip explains cap)
stale/expired/unconfirmed ── Dismiss ───────────────────────▶ removed locally only (no command)
resolving/awaiting_server ── any gesture ───────────────────▶ ignored (no send, no toast)
```

Rules: `ACK` is not terminal and never closes/enables; only `APPROVAL_RESOLVED` removes the approval. `E_APPROVAL_REPLAY`/`NOT_FOUND`/`TIMEOUT` are non-retryable classes. A retry that hits replay becomes `stale` (no second side effect: nonce is single-use server-side). `expireApproval` never fires while `resolving`/`awaiting_server` (the server decides; the card stays disabled until the settle/refusal path lands).

### 5.3 `run()` + `resolveApproval` (exact store code changes)

```ts
// store.ts: replace boolean run() with a result that preserves the refusal
type CommandResult =
  | { readonly ok: true }
  | { readonly ok: false; readonly refusal: Refusal };

const run = async <N extends ServerCommandName>(
  name: N,
  payload: CommandPayloads[N],
  options?: { readonly timeoutMs?: number },
): Promise<CommandResult> => {
  const correlationId = uuidv7();
  set({ pendingCommands: { ...get().pendingCommands, [correlationId]: { name, startedAt: Date.now() } } });
  try {
    await api.send(name, payload, { correlationId, ...(options?.timeoutMs === undefined ? {} : { timeoutMs: options.timeoutMs }) });
    return { ok: true };
  } catch (error) {
    const refusal = error instanceof RefusalError ? error.refusal : clientRefusal('E_WS_DISCONNECTED');
    // Existing toast policy unchanged (applyEvent toasts server ERRORs; run toasts only
    // local transport/jail/file refusals):
    if (refusal.code === 'E_WS_DISCONNECTED' || refusal.code === 'E_PATH_OUTSIDE_JAIL' || refusal.code === 'E_FILE_TOO_LARGE') {
      pushRefusal(refusal);
    }
    if (refusal.code === 'E_KEY_MISSING') set({ settings: { ...get().settings, keyPresent: false } });
    return { ok: false, refusal };
  } finally {
    const pending = { ...get().pendingCommands };
    delete pending[correlationId];
    set({ pendingCommands: pending });
  }
};
```

Call-site updates (exact list from `store.ts`): lines 679, 696, 709, 722, 736, 787, 800 can keep `await run(...)`; lines **780** (`const result = await run('CHAT_PIN'…); if (result.ok) …`), **851** (`if (!result.ok)`), **890**, **910** switch from `const ok` to `const result`; line 766 becomes the `resolveApproval` implementation below.

```ts
async resolveApproval(approvalId, decision, reason?): Promise<ApprovalResolveOutcome> {
  const record = get().approvals.find((candidate) => candidate.approvalId === approvalId);
  if (!record) return refusalOutcome('operator', 'E_APPROVAL_NOT_FOUND'); // toast once
  const previous = get().approvalResolve[approvalId] ?? initialApprovalResolveState();
  if (previous.status === 'resolving' || previous.status === 'awaiting_server') {
    return refusalOutcome('ignored', 'E_APPROVAL_REPLAY'); // no send, no toast
  }
  if (record.nonce === null) { expireApproval(approvalId); return refusalOutcome('operator', 'E_SCHEMA_INVALID', 'nonce missing'); }
  if (isApprovalExpired(record.expiresAt, Date.now())) { expireApproval(approvalId); return refusalOutcome('expired', 'E_APPROVAL_TIMEOUT'); }
  if (previous.status === 'retryable' && previous.attempts >= MAX_APPROVAL_RESOLVE_ATTEMPTS) {
    return refusalOutcome('terminal', 'E_APPROVAL_REPLAY', 'retry cap reached');
  }
  setResolveState(approvalId, { status: 'resolving', attempts: previous.attempts + 1, lastDecision: decision, refusal: null, lastAttemptAt: Date.now() });
  setResolvingProjection(approvalId, true);
  const result = await run('APPROVAL_RESOLVE', { approval_id: approvalId, nonce: record.nonce, decision, ...(reason === undefined ? {} : { reason }) }, { timeoutMs: APPROVAL_RESOLVE_TIMEOUT_MS });
  setResolvingProjection(approvalId, false);
  if (result.ok) { setResolveState(approvalId, { ...get().approvalResolve[approvalId]!, status: 'awaiting_server' }); armAckSettleTimer(approvalId); return { ok: true }; }
  const classified = classifyApprovalRefusal(result.refusal);
  setResolveState(approvalId, { status: classified.status, refusal: result.refusal, lastAttemptAt: Date.now(), attempts: get().approvalResolve[approvalId]!.attempts, lastDecision: decision });
  if (classified.kind === 'stale') { clearApprovalTimers(approvalId); }
  if (classified.kind === 'expired') { clearApprovalTimers(approvalId); }
  return { ok: false, kind: classified.kind, refusal: result.refusal };
}

async retryApproval(approvalId, decision?, reason?): Promise<ApprovalResolveOutcome> {
  const record = get().approvals.find((candidate) => candidate.approvalId === approvalId);
  const state = get().approvalResolve[approvalId];
  if (!record || !state || state.status !== 'retryable') return refusalOutcome('ignored', 'E_APPROVAL_REPLAY');
  if (state.attempts >= MAX_APPROVAL_RESOLVE_ATTEMPTS || get().connection !== 'open') {
    return refusalOutcome('terminal', 'E_APPROVAL_REPLAY', 'Retry limit reached (3).');
  }
  return get().resolveApproval(approvalId, decision ?? state.lastDecision ?? 'approve', reason);
}

// Closure helpers inside createUiStore (exact signatures; single implementation each):
const setResolveState = (id: string, state: ApprovalResolveState): void =>
  set({ approvalResolve: { ...get().approvalResolve, [id]: state } });
const setResolvingProjection = (id: string, inFlight: boolean): void =>
  set({
    resolvingApprovals: inFlight
      ? (get().resolvingApprovals.includes(id) ? get().resolvingApprovals : [...get().resolvingApprovals, id])
      : get().resolvingApprovals.filter((candidate) => candidate !== id),
  });
const refusalOutcome = (
  kind: ApprovalFailureKind,
  code: string,
  message?: string,
): ApprovalResolveOutcome => ({
  ok: false,
  kind,
  refusal: clientRefusal(code, message === undefined ? undefined : { message }),
});
const armAckSettleTimer = (id: string): void;   // setTimeout(APPROVAL_ACK_SETTLE_MS) -> awaiting_server -> retryable
const clearApprovalTimers = (id: string): void; // clear + delete ack settle + any per-approval timer
const armReplaySettleTimer = (): void;          // setTimeout(APPROVAL_REPLAY_SETTLE_MS) -> settleApprovalReplay()
const clearReplaySettleTimer = (): void;        // used by begin/dispose
```

Store closure timer helpers: `armAckSettleTimer(id)` → `setTimeout(() => { if (get().approvalResolve[id]?.status === 'awaiting_server' && get().approvals.some(a => a.approvalId === id)) setResolveState(id, { status:'retryable', refusal: clientRefusal('E_WS_DISCONNECTED', { message:'No APPROVAL_RESOLVED within 2000 ms; the decision may not have been applied — retry is safe, the nonce is single-use.' }) }); }, APPROVAL_ACK_SETTLE_MS)` stored in a `Map` and cleared on `APPROVAL_RESOLVED`/settle/dismiss/dispose. `refusalOutcome(kind, code, message?)` builds `clientRefusal(code, {message})`; it must **not** send.

`resolvingApprovals` compatibility: keep the field and maintain it as a projection of statuses `resolving|awaiting_server` (append/remove), so existing tests/e2e double-click semantics and any external readers keep working; document it as a read-only projection (the source of truth becomes `approvalResolve`). Update the doc comment at `store.ts:161-162`.

### 5.4 Overlay / component split (exact)

`ApprovalOverlay.tsx` keeps ownership of the list and mounts cards; all settle/retry logic is store-driven:

```ts
// web/src/panels/ApprovalOverlay.tsx (kept exports: APPROVAL_DECISIONS, ApprovalDecision,
// ApprovalView, UNKNOWN_VALUE, truncateId, formatArgsSha256, ExpiryLabels, formatExpiry,
// isApprovalExpired, approvalAriaLabel, approvalActionAriaLabel; REMOVED:
// canResolveApproval, prunePendingResolutions, and local pendingResolution/useState lock)
export function ApprovalOverlay(): JSX.Element | null;
```

New `web/src/panels/ApprovalCard.tsx`:

```ts
export interface ApprovalCardProps {
  readonly record: ApprovalView;
  readonly resolve: ApprovalResolveState;          // from selectApprovalResolve(id)
  readonly status: ApprovalDisplayStatus;          // approvalDisplayStatus(record, resolve, nowMs)
  readonly nowMs: number;
  readonly connection: ConnectionStatus;
  readonly onDecide: (decision: ApprovalDecision) => void;
  readonly onRetry: () => void;
  readonly onDismiss: () => void;
}
export function ApprovalCard(props: ApprovalCardProps): JSX.Element;
```

New `web/src/panels/ApprovalStatusStrip.tsx`:

```ts
export interface ApprovalStatusStripProps {
  readonly status: ApprovalDisplayStatus;
  readonly refusal: Refusal | null;   // verbatim code/message/how_to_fix
  readonly attempts: number;
  readonly maxAttempts: number;
}
export function ApprovalStatusStrip(props: ApprovalStatusStripProps): JSX.Element | null;
```

Rendering rules inside `ApprovalCard`: keep the existing `section role="dialog" aria-modal="true" aria-label={approvalAriaLabel(tool)}` and the whole evidence `<dl>` (tool ids hash expiry), the nonce-less copy, `dd[aria-live="polite"]` TTL, `button[aria-label="Approve tool call for <tool>"]`/`Deny…` (existing e2e selectors unchanged). Add: `data-approval-status={status}` on the dialog; stale banner `[role="status"][data-testid="approval-replay-banner"]`; status strip `[data-testid="approval-status-strip"]` with `role="status"` and refusal `role="alert"` text `code · message · how_to_fix`; `button[aria-label="Retry approval for <tool>"]` (only `retryable`, disabled at cap/connection); `button[aria-label="Dismiss approval for <tool>"]` (only `expired|stale|unconfirmed`); resolving line `[role="status"]` with `STRINGS.approval.resolving`; `aria-busy="true"` on the dialog while `resolving|awaiting_server`. Focus policy (W11 D11): initial focus on Deny, focus trap, return focus to the invoking element on close; Escape never resolves (browser default only dismisses if implemented — do not implement dismiss-on-Escape). All copy from `STRINGS.approval`/`STRINGS.controls` (no literals).

Availability derivation in the card comes only from `approvalActionsEnabled(...)` in `approval.ts`; the panel no longer computes disabled state from local state.

### 5.5 Strings for (c) (all under `STRINGS.approval`, no literals)

`status.*` (§3.5), `retryLabel`, `dismissLabel`, `refusalTitle`, `retryCapReached: 'Retry limit reached (3) — dismiss this card or cancel the run.'`, plus reuse `STRINGS.controls.retry/dismiss`. Refusal strip shows the server `Refusal` fields verbatim (never re-worded).

### 5.6 Tests — item (c)

`web/tests/approval.state.test.ts` (pure): classifier matrix (every row above, unknown code default), availability truth table (16 rows), `approvalDisplayStatus` precedence, initial state, retry cap logic helper.
`web/tests/approval.integration.test.ts` (extend `FakeUiApi` with `rejectNext(refusal)`):
1. approve → ACK (`resolveNext`) → `approvalResolve[id].status==='awaiting_server'`, `resolvingApprovals` projection empty, approval still present.
2. `vi.useFakeTimers()`; advance `APPROVAL_ACK_SETTLE_MS` → status `retryable`, refusal code `E_WS_DISCONNECTED`, card not removed; `retryApproval` sends exactly one second command with the same id/nonce and new correlation id.
3. rejection with `E_APPROVAL_REPLAY` → status `stale`, no removal, attempts preserved; further `resolveApproval` returns `ignored`, sends nothing.
4. rejection with `E_APPROVAL_DECISION_INVALID` → `retryable`; retry cap: after 3 attempts `retryApproval` sends nothing and returns non-ok.
5. rejection with `E_APPROVAL_TIMEOUT` → `expired`, no retry.
6. `dismissApproval` removes only covered statuses; expired/stale/unconfirmed allowed; pending/resolving ignored.
7. `APPROVAL_RESOLVED` clears record + resolve + timer (advance time after → no change).
8. no optimistic removal: after send and ACK the record remains until resolved.
9. `run()` result propagation: FakeUiApi rejects with `RefusalError(clientRefusal('E_APPROVAL_REPLAY'))` → outcome kind `stale`; transport rejection (`E_WS_DISCONNECTED`) → kind `transport`.
10. expiry timer: past `expiresAt` → `expireApproval` → `expired`, actions disabled, no command.
`web/tests/approvalPanels.test.ts` (new static-render suite, node env, `renderToStaticMarkup` + store provider like `agentsToolActivityMount.test.ts:37-90`): each status renders the expected strip/buttons/aria; pending enables both actions; `resolving`/`awaiting_server` disabled + `aria-busy`; `retryable` shows Retry enabled and actions disabled; `expired|stale|unconfirmed` dismiss-only + correct copy; refusal strip shows code/message/how_to_fix verbatim; `[data-approval-status]` matches; no hardcoded copy (static-source scan pattern from `approval.integration.test.ts:262-274`); nonce-less still disabled with the gap copy.
E2E `e2e/specs/approval-flow.spec.ts` new cases (uses §3.7 helpers):
- **retry after ERROR:** approve → `sendErrorForCommand(ws, approvalResolveCommands()[0], 'E_APPROVAL_REPLAY', seq)` → `[data-testid="approval-status-strip"]` shows the code, card stays open, both approve/deny disabled; dismiss-only (since replay is stale) and **no second command**.
  - Variant `E_APPROVAL_DECISION_INVALID` → Retry button visible/enabled; click → exactly 2 APPROVAL_RESOLVE commands with identical `approval_id`/`nonce`; then `APPROVAL_RESOLVED` closes the card and commands stay 2.
- **ACK-only stall:** approve (mock ACK auto) → no APPROVAL_RESOLVED; wait `APPROVAL_ACK_SETTLE_MS` (Playwright `page.clock` if enabled; else expect strip within 3 s) → retryable strip + Retry enabled; no card close.
- **expired/nonce-less regressions:** both remain read-only, no commands (existing tests stay).
Assertions must use `ws.approvalResolveCommands()` counts and payload equality via `MockSentCommand` (`replay-harness.ts:138-145`).

---

## 6. File-change map, rollout order, migration/compat

### 6.1 Change map (all additive unless stated)

| File | Change | Item |
|---|---|---|
| `backend/crates/hiverouter-tools/src/approval.rs` | add `pending_ids_for_active_runs`; make `load_request` public; preview column migration + domain/wire field | a, b |
| new `backend/crates/hiverouter-tools/src/preview.rs` | `Preview`, `redact_args_preview` + redaction tests | b |
| `backend/crates/hiverouter-tools/src/lib.rs` | register `pub mod preview;` | b |
| `backend/crates/hiverouter-core/src/tools_bridge.rs` | `pending_approval_events_all(limit)`; `approval_request_event` maps preview; `request_model_tool_approval` passes preview | a, b |
| `backend/crates/hiverouter-core/src/ws.rs` | `publish_pending_approval_replays`, `send_hello_replay_and_approvals`, two call sites | a |
| `backend/crates/hiverouter-contract/src/event.rs` + `schemas/contract-v1.json` + contract tests | `ArgsPreview`, optional field, golden | b |
| `docs/contracts/UI-COMMANDS-v1.md` + change ledger | 1.3 additive paragraph/row | b |
| `web/src/api/contract.ts` | `ApprovalArgsPreview`, optional payload field, decoder | b |
| new `web/src/state/approval.ts` | replay/resolve state machine, classifier, availability (pure) | a, c |
| `web/src/state/store.ts` | replay fields/actions, resolve state, `CommandResult`, `resolveApproval`/`retryApproval`/dismiss/expire, timer cleanup in `dispose` | a, c |
| `web/src/panels/ApprovalOverlay.tsx` | list + card mount; remove local lock/prune helpers | a, c |
| new `web/src/panels/ApprovalCard.tsx`, `ApprovalStatusStrip.tsx`, `ApprovalArgsPreview.tsx` | status-aware card/retry/dismiss/preview + DOM contract | a, b, c |
| `web/src/state/strings.ts` | replay/status/retry/dismiss/refusal/preview strings (from mockup/UX plan) | a, b, c |
| `web/src/App.tsx` | **no change** (single overlay mount at line 100; tests keep pinning that) | — |
| `web/tests/approval.state.test.ts`, `approval.integration.test.ts`, `approvalPanels.test.ts` | new/extended suites | a, b, c |
| `e2e/helpers/replay-harness.ts` | `helloFrameCount`, `closeMockSocket`, `waitForReconnect`, `sendServerHello`, `sendErrorForCommand` | a, c |
| `e2e/specs/approval-flow.spec.ts` | replay, resolved-while-away, reload, retry, ACK-only cases | a, c |
| new `backend/crates/hiverouter-core/tests/approval_replay.rs` + tools unit tests | replay projection + preview | a, b |

### 6.2 Migration/compat notes

1. **(a)/(c) are wire-compatible**: no command/event/refusal change; old server/new UI (backend not yet replaying) → after a reconnect pre-existing approvals become `unconfirmed` (read-only, dismissable) instead of stuck; fresh reload simply shows no approvals until the next live request (no fabricated cards). New server/old UI → old client still upserts replayed `APPROVAL_REQUEST` by id (existing reducer), so replay is harmless before the client wave lands.
2. **(b) requires the frozen-contract process**: additive/optional field, same `hiverouter.v1` version (pattern of the 1.2 additive fields, `docs/contracts/UI-COMMANDS-v1.md:7-16`), golden regeneration, matrix/roundtrip tests, change-ledger row. Old client ignores the field; new client with absent field shows the hash-only copy. **Do not ship the UI preview section before the producer + decoder exist** (flag default stays off until then).
3. **No persistence**: no localStorage/sessionStorage keys; reload recovery is replay-only. The R2 sessionStorage fallback remains rejected/off (`APPROVAL-UX-PLAN.md:66,203-207`) pending a privacy review; if ever enabled it must store only non-secret fields and delete on resolve/expire.
4. **Restart boundary:** reaper + recovery run at boot before serving (`http.rs:107-130`); replay projection excludes decided/expired/terminal-run rows → nothing is resurrected after a restart. Tests must include this case.
5. **Multi-client/multi-tab:** replay is broadcast to all subscribers; upsert is idempotent; a decision in one client removes the card in all via `APPROVAL_RESOLVED`. No cross-tab storage.
6. **`resolvingApprovals` remains** as a compatibility projection (tests/e2e keep working); its doc comment changes to "projection; source of truth is approvalResolve". Removing it is a later cleanup, not part of this wave.
7. **Selectors stay backward compatible**: dialog `role/aria-label`, `dd[title]`, `dd[aria-live="polite"]`, action aria-labels and disabled semantics are preserved; only additive `data-*`/status elements are new. Existing e2e tests must pass unchanged.
8. **Feature flags:** `VITE_UI_APPROVAL_REPLAY` (default `'1'`; `'0'` keeps records actionable across reconnect for a pre-replay backend — degraded fallback only) and `VITE_UI_ARGS_PREVIEW` (default `'0'` until (b) ships, then `'1'`). Flags live in `createUiStore` initial state or component gates consistently (single read point, documented) — do not read `import.meta.env` in reducers.

---

## 7. Verification plan for the implementing wave (NOT run here — no build/test in this research task)

Backend: `cargo test -p hiverouter-tools approval` (pending_all/redaction/preview round-trip), `cargo test -p hiverouter-core approval` (projection, replay publish helper, boot reaper), `UPDATE_GOLDEN=1 cargo test -p hiverouter-contract golden_schemas` then normal golden test, `cargo test -p hiverouter-core --test approval_replay`.
Frontend: `npm run typecheck`, `npm test -- approval` (state/integration/panels suites), `npm test` full, then `npx playwright test e2e/specs/approval-flow.spec.ts` against a rebuilt `web/dist` (harness `startDistHarness` requires a build; the implementing wave must run it, this blueprint does not).
Acceptance: existing approval e2e passes unchanged + new cases 11-13 (replay) and retry/ACK-only cases; zero-secret privacy assertions for (b); no auto-command in any hydration/replay test; `resolvingApprovals` projection parity test.

---

## 8. Open questions / UNVERIFIED (resolve before implementation)

1. **Backend replay trigger scope:** replay on every hello (chosen: always, bounded 20) vs reconnect-only. Chosen policy is simpler and also covers page reload/new tabs; confirm with Lead/backend owner. Latest-active-run-only is an alternative (UX plan open question 1).
2. **`_meta`/change ledger:** `CHANGE-LEDGER-IMPL.md` not found by filename scan; confirm the actual ledger/claim file to update for (b).
3. **`pending_ids_for_active_runs` uses the core-owned `runs` table from the tools-owned migration context** — verify the join is acceptable with the DB owner (it is a read-only query over the same SQLite file; `load_request` is made public to avoid widening coupling). If rejected, fallback: replay only rows whose `run_id` matches `store.latest_active_run()` / `list_runs` filtered non-terminal in core.
4. **`E_APPROVAL_DECISION_INVALID` retry semantics:** server never consumes the nonce on validation refusal (`approval.rs:672-752` path), so retry is safe; confirm the gate returns that code before any `decided_at` write (READ: `resolve_with_nonce` validates decision before updating — spot-verify in implementation wave).
5. **ACK→APPROVAL_RESOLVED window:** chosen 2000 ms; validate against real server latency (ACK and APPROVAL_RESOLVED are published back-to-back in `ws.rs:474-481`, so a lost event is the only gap). A shorter/longer value is a product decision.
6. **Playwright reconnect support:** `WebSocketRoute.close()` exists; the mock harness must be extended to count hello frames (blueprint §3.7). Validate in the implementing wave; no runtime evidence was gathered here.
7. **`args_preview` JSON-schema shape** (nested object vs three flat fields) must be confirmed with the contract owner before writing the golden; the TS decoder is null-safe either way, the Rust/wire shape above is the recommended one.
8. **No runtime/build/test evidence was produced by this task** — all §0 facts are reads; anything marked "must/expected" is a design requirement for the implementation wave, not a verified result.

---

## 9. Read references (this session)

`plans/foresight/APPROVAL-UX-PLAN.md`; `web/src/state/store.ts`; `web/src/panels/ApprovalOverlay.tsx`; `web/tests/approval.integration.test.ts`; `web/tests/toolActivityPanels.test.ts`; `web/tests/agentsToolActivityMount.test.ts`; `web/src/api/contract.ts`; `web/src/api/client.ts`; `web/src/api/refusals.ts`; `web/src/state/strings.ts`; `backend/crates/hiverouter-contract/src/event.rs`; `backend/crates/hiverouter-contract/src/state.rs`; `backend/crates/hiverouter-core/src/{ws,tools_bridge,http,db,recovery}.rs`; `backend/crates/hiverouter-tools/src/approval.rs`; `backend/crates/hiverouter-contract/tests/golden_schemas.rs`; `docs/contracts/UI-COMMANDS-v1.md`; `e2e/specs/approval-flow.spec.ts`; `e2e/helpers/replay-harness.ts`; `web/src/panels/StatePack.tsx`.
