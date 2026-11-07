# FORESIGHT — Approval UX plan: reconnect replay · safe args preview · resolve retry

**Seat:** FORESIGHT PLANNER 3 · **Mode:** READ-ONLY (no code, no build, no product edits) · **Date:** 2026-11-06
**Targets:** items **10** (pending approval replay after reconnect), **11** (safe args preview privacy), **15** (resolve retry/refusal UX).
**Stack:** TypeScript 7 / React 19 / zustand 5; frozen contract `docs/contracts/UI-COMMANDS-v1.md` (14 commands, 23 events, 65 refusals; 1.2 additive approval fields only).
**Evidence basis:** files read in this session (paths+lines below); no execution claims beyond reads.

---

## 1. Verified current state (evidence table)

| # | Fact | Evidence |
|---:|---|---|
| E1 | `ApprovalRecord` carries `approvalId, tool, nonce, decision, requestedAt, runId, taskId, toolCallId, argsSha256, expiresAt`; all six 1.2 fields are nullable | `web/src/state/store.ts:61-73` |
| E2 | `APPROVAL_REQUEST` reducer replaces by `approvalId` and populates the six fields null-safely; `APPROVAL_RESOLVED` removes by id | `web/src/state/store.ts:484-511` |
| E3 | `resolveApproval` requires nonce (else `E_SCHEMA_INVALID` toast), guards duplicate in-flight via `resolvingApprovals`, sends `APPROVAL_RESOLVE{approval_id,nonce,decision,reason?}`, clears the guard in `finally`; no optimistic mutation | `web/src/state/store.ts:739-777` |
| E4 | `run()` returns boolean, toasts only transport/jail/file refusals; typed server ERROR for APPROVAL_RESOLVE is toasted by `applyEvent` but its code is not returned to `resolveApproval` | `web/src/state/store.ts:565-596, 248-254` |
| E5 | The overlay keeps its **own** `pendingResolution` lock, pruned only when the approval disappears (`APPROVAL_RESOLVED`); the comment explains store `resolvingApprovals` clears on ACK, so it "cannot be the sole disabled source" | `web/src/panels/ApprovalOverlay.tsx:181-190, 196-215` |
| E6 | Overlay renders tool, truncated `args_sha256` (full in `title`), run/task/tool-call/approval ids, expiry countdown, resolving status; actions gated by `canResolveApproval` (nonce present, not resolving, not expired) | `web/src/panels/ApprovalOverlay.tsx:236-288, 110-120` |
| E7 | **No retry/refusal rendering exists in the card**: on a typed ERROR after send, `resolvingApprovals` clears (store) but the panel lock stays `true` because the approval was not removed → Approve/Deny remain disabled with no Retry control (stuck card). Only the global toast shows the code | `ApprovalOverlay.tsx:181-190, 203-215, 290-294`; `store.ts:764-776` |
| E8 | No args preview: only `args_sha256` is rendered; `args_preview` is absent from the contract; W11 design recorded hash-only as the accepted v1.2 UI and preview as conditional on a future patch | `ApprovalOverlay.tsx:227,249-252`; `plans/waves/W11-UI-TOOL-ACTIVITY.md:107,220`; `docs/contracts/UI-COMMANDS-v1.md:62-65` |
| E9 | Frozen `APPROVAL_RESOLVE` preconditions/refusals: `E_APPROVAL_NOT_FOUND, E_APPROVAL_REPLAY, E_APPROVAL_TIMEOUT, E_APPROVAL_DECISION_INVALID, E_RESYNC_REQUIRED`; ACK + `APPROVAL_RESOLVED` on success; no command/event to list pending approvals | `docs/contracts/UI-COMMANDS-v1.md:37,54-65` |
| E10 | Server truth: pending requests persisted with single-use `nonce`, `args_sha256`, TTL 120 s; replay/expired refusals; decisions ledgered via `resolve_approval_ledgered` → `append_approval_decision`; `pending_approvals(run_id)` and `pending_approval_events(run_id)` exist and are documented **"safe for UI replay"** (read-only, no nonce consumption) | `backend/crates/hiverouter-tools/src/approval.rs:26,111-118,415-505,561`; `backend/crates/hiverouter-core/src/tools_bridge.rs:217-239,241-260,342-358` |
| E11 | **No production caller** wires those projections into the socket: `send_hello_and_replay` only replays the event bus (`SERVER_HELLO` / `RESYNC_REQUIRED` / bus events); approval requests are not re-emitted on connect | `backend/crates/hiverouter-core/src/ws.rs:132-197,225-295`; caller grep: only `tools_bridge` tests call `pending_approval_events` |
| E12 | Producer seam exists but is not in the live run path yet: `request_model_tool_approval` creates/idempotently returns a pending request and the full wire event (nonce/run/task/tool-call/args/expiry) | `tools_bridge.rs:271-332`; W10: `ApprovalGate::request` had no production caller (`plans/waves/W10-TOOL-LOOP-DECISION.md:48`) |
| E13 | Reconnect semantics today: store keeps `approvals` across socket reconnect (no reset), but a page reload creates a new store and loses them; no persistence; `init()` just re-subscribes and opens | `store.ts:636-665`; `ApprovalOverlay` reads store only |
| E14 | Restart policy: runs waiting on approval are transitioned to terminal (`E_APPROVAL_TIMEOUT`) and pendings expire by recovery; "pending approvals are never auto-approved or replayed" — replay-after-restart must NOT resurrect them | `plans/waves/W11-SUSPENSION-DESIGN.md:55-56,177,311`; `W10-TOOL-LOOP-DECISION.md:164-165` |
| E15 | Current e2e coverage: render/no-send, approve/ACK-stays-open, double-click guard, deny+reason, duplicate request, nonce-less, expired, malformed. **Missing:** replay-after-reconnect/reload, retry after ERROR, ACK-only timeout, args preview privacy | `e2e/specs/approval-flow.spec.ts:113-269` |
| E16 | W11 UI design already specifies the target status machine (`pending→resolving→resolved|denied`, plus `expired/stale/cancelling`), args preview as server-redacted ≤2 KiB plain text, retry forbidden after `E_APPROVAL_REPLAY`, ACK-only→15 s→pending + named toast | `plans/waves/W11-UI-TOOL-ACTIVITY.md:111,200-236,278-290` |

**Proof of item-15 defect (deterministic):** send APPROVAL_RESOLVE → server ERROR `E_APPROVAL_REPLAY` (or transport timeout) → store guard clears, approval stays, overlay `pendingResolution[id]` stays `true` (E5/E7) → both actions permanently disabled; the only recovery is page reload (card not replayed today). This is why reconnect replay (item 10) and retry UX (item 15) must be designed together.

---

## 2. Goals / acceptance

- **G10 Reconnect replay:** after a dropped/restored WebSocket (same server process) or a page reload, the operator sees the still-pending, still-valid approvals again, deduped by `approval_id`, without auto-approving anything and without resurrecting expired/decided/restart-orphaned requests.
- **G11 Safe args preview:** the operator can judge *what* is being approved without exposing restricted data, without client-side reconstruction, and without leaking into storage/logs/DOM metadata; when the contract lacks a preview, the UI says so honestly and shows the hash binding.
- **G15 Resolve retry/refusal:** every failure mode after a decision gesture is visible, classified and recoverable only when safe; no stuck disabled card, no optimistic resolution, no retry of a consumed nonce.

---

## 3. Architecture / data flow

```
WS EVENT APPROVAL_REQUEST ──▶ contract decoder (strict, optional 1.2 fields) ──▶ store.approvals
WS EVENT APPROVAL_RESOLVED ─▶ remove by id + clear state/persistence            │
SERVER_HELLO/RESYNC_REQUIRED ─▶ replay/settle window ──▶ hydrate by approval_id │
                                                                                ▼
ApprovalOverlay ──▶ ApprovalCard (status machine) ──▶ store.resolveApproval(id, decision)
      ▲                                              │
      └──── status/refusal/ack-only/retry state ◀────┘ (ACK alone never closes or approves)
```

The store is the single owner of approval lifecycle state; the card is a pure projection. The server remains the only authority for pending/nonce/TTL/resolution.

---

## 4. Item 10 — replay pending approvals after reconnect

### 4.1 Options and verdict

| Option | Contract change? | Verdict |
|---|---|---|
| **R1. Server re-emits existing `APPROVAL_REQUEST` events on hello** for still-pending, unexpired requests (using `pending_approval_events`; no new event/command/refusal) | **No contract change** (reuses frozen event + existing optional fields) | **Preferred.** Requires backend wiring only (`ws.rs` after `SERVER_HELLO`, or bus publish with new seq); dedupe is already replace-by-id (E2). Run selection policy is the only open decision. |
| R2. Client `sessionStorage` snapshot of pending approvals (nonce included) restored after `/api/session` | No | **Fallback only**, behind an explicit flag. Survives backend as-is/page reload, but puts a single-use capability secret at rest in the browser; XSS/extension exposure; stale after restart must be discarded by TTL/recovery refusal. Default **off**. |
| R3. New `APPROVALS_LIST` command / REST endpoint | **Yes** (frozen-contract change: new command/result event or HTTP route) | Not needed for v1.2; only if run-scoped, multi-client listing is required. Defer. |
| R4. Replay from the event-bus history only | No | Insufficient: bus ring may evict, page reload starts at `last_seq=0` (gap → RESYNC, no approvals), and restart history must not resurrect (E14). Not reliable alone. |

### 4.2 Preferred design (R1, no contract change)

1. **Backend (core owner):** after sending `SERVER_HELLO` on a fresh connection, if the connection reported `last_seq` that does not already cover the request, publish/send one `APPROVAL_REQUEST` per pending request. Reuse `api.subscribe`-delivered envelopes; send directly to the reconnecting socket to avoid duplicate fan-out, or publish one bus event per approval (new sequence number) and rely on UI dedupe.
2. **Run scope:** the frozen hello has no run id. Deterministic v1.2 policy: replay pending approvals for **all non-terminal runs known to the store** (bounded: ≤5 runs / ≤20 requests, oldest first), because the local server is single-user and the UI already replaces by `approval_id`. Open question for Lead/backend owner; alternative is "latest active run only".
3. **Restart boundary:** recovery already expires pending approvals and terminates awaiting runs (E14); therefore replay after a *process restart* finds nothing. If recovery has not yet run for some row, the server must not replay terminal/expired requests. UI additionally drops any replay whose `expires_at` is past.
4. **Client (store):**
   - `hydrateApprovals(events, source)` action: dedupe by `approvalId`; when the id already exists, **merge** (refresh nonce/args/expiry identity) but preserve `approvalResolve` state; never reset an in-flight resolution; never auto-send.
   - Store fields: `approvalsStale: boolean` (set on `reconnecting`/`RESYNC_REQUIRED`, cleared on `SERVER_HELLO` after replay window: e.g. `APPROVAL_REPLAY_SETTLE_MS = 250`), `approvalReplaySource: 'hello'|'session'|null`, `lastApprovalReplayAt: number|null`.
   - On `APPROVAL_RESOLVED`: remove record + clear resolve state + delete persisted entry.
   - Timer: expire local cards when `expiresAt<=now` (existing overlay timer today; move to store or shared hook) → status `expired`, no action.
5. **UI:** while `approvalsStale`, card shows "Reconnected — awaiting pending-approval replay" and both actions are disabled; cards arriving from replay appear exactly once; a replay for an already-resolved id is dropped (server pending set excludes it).
6. **Reload path:** with R1 only, a full page reload reconnects and receives replay; no local persistence needed → best privacy. If R1 is not yet shipped, R2 fallback keeps the same reducer/UI semantics.

### 4.3 Store fields/actions (exact names)

```ts
readonly approvalsStale: boolean;
readonly lastApprovalReplayAt: number | null;
readonly approvalReplaySource: 'hello' | 'session' | null;
hydrateApprovals(events: readonly ApprovalRequestPayload[], source: 'hello' | 'session'): void;
markApprovalsStale(): void;
clearApprovalsStale(): void;
```
Reducer: `APPROVAL_REQUEST` continues replace-by-id; add merge rule only inside `hydrateApprovals` (avoid resetting `approvalResolve`).

---

## 5. Item 11 — safe args preview (privacy)

### 5.1 What is possible without a contract change

- **Only `args_sha256` + tool + identity + expiry** (today). The UI cannot show argument values: `APPROVAL_REQUEST` has no args body and `TOOL_CALL` only has the hash (E8, E9). Client-side reconstruction from model JSON is forbidden (untrusted data; W11 D4).
- Honest copy: "Arguments are bound by `args_sha256`; the frozen approval request does not include argument bodies. Compare the hash/tool_call_id with tool activity before deciding." Keep the full hash copy affordance.

### 5.2 With a contract patch (recommended v1.3 addition; optional field, additive)

```ts
// APPROVAL_REQUEST optional addition (proposal)
readonly args_preview?: {
  readonly text: string;        // server-redacted, plain text, <= 2048 chars
  readonly truncated: boolean;  // producer truncated the preview
  readonly redacted: boolean;   // producer masked >=1 secret-shaped value
} | null;
```
Contract/schema impact: event payload change → `CHANGE-LEDGER-IMPL.md` row + contract matrix + Rust/TS/to-schema tests (frozen-doc rule). No new command/event/refusal code required.

**Producer rules (backend):** preview is server-redacted, never raw args; bounded ≤2 KiB; plain text only; mask at least `sk-…`, `Bearer …`, `Authorization:`, `password|token|secret|api_key` values, JWT-shaped strings, env-like assignments; tool-specific policy: `file_write` → show path + content length/hash, not content; `shell_exec` → program + args with secret masking; unknown tools → key names only. The redaction result is generated once and stored only transiently for the event (never logged/ledgered; `args_sha256` remains the binding).

**UI rules:** store keeps `argsPreview` inside `ApprovalRecord` (memory only); render in a labelled `<pre>`/text block with `aria-describedby`, max-height, scroll; React text interpolation only (no `dangerouslySetInnerHTML`); never put preview text in `title`, `data-*`, URL, localStorage/sessionStorage, console, toasts, error messages, trace/evidence captures; "redacted"/"truncated" labels shown; no copy button for values (hash copy only); `VITE_UI_TOOL_ACTIVITY` flag gates the section until the contract lands.
Naming: component `ApprovalArgsPreview`; fields `argsPreviewText: string | null`, `argsPreviewRedacted: boolean`, `argsPreviewTruncated: boolean` (flattened for zustand simplicity) + selectors `selectApprovalArgsPreview`.

**Tests (privacy):** fixture previews containing `sk-live…`, `Bearer abc`, `password=…`, `token:…`, JWT → rendered text contains `[redacted]`/none of the secrets; DOM `title`/attributes/console/sessionStorage contain no preview or secret; over-2 KiB preview truncated; absent/invalid preview → honest unknown, no fabricated args; e2e `approval-flow` asserts preview absent while field not in contract (current state) and present+redacted after patch.

---

## 6. Item 15 — resolve retry/refusal UX

### 6.1 State machine (per approval, store-owned)

```
pending ──gesture──▶ resolving ──ACK──▶ awaiting_server ──APPROVAL_RESOLVED──▶ closed
   │                     │                    │
   │                     │                    ├─ERROR E_WS_DISCONNECTED/E_RESYNC_REQUIRED → retryable(connection)
   │                     │                    ├─ERROR E_APPROVAL_DECISION_INVALID/E_SCHEMA_INVALID → retryable(operator)
   │                     │                    ├─ERROR E_APPROVAL_REPLAY/E_APPROVAL_NOT_FOUND → stale (no retry)
   │                     │                    └─ERROR E_APPROVAL_TIMEOUT → expired (read-only)
   └─expiry timer────────────────────────────────────────────────────────────▶ expired (read-only)
stale/expired: dismiss only; no resend; nonce never reused on stale
```

- `ACK` is not terminal and never enables actions (current e2e already asserts the card stays open on ACK; E15).
- **ACK-only timeout:** after client command timeout (existing 15 s `resolveApproval` request timeout) → `retryable` with `E_WS_DISCONNECTED` named copy. Server may have committed; a retry then returns `E_APPROVAL_REPLAY` → transition to `stale`, show "already decided by this or another client", no second side effect.
- **Retry:** manual button only; same `approvalId`+`nonce`, new `correlation_id`; max 3 attempts per approval; disabled while connection not `open`; never auto-retry; `E_APPROVAL_REPLAY`/`NOT_FOUND`/`TIMEOUT` are non-retryable terminal/stale classes.
- **Race:** gesture while `resolving`/`awaiting_server`/`retryable-after-send` is a no-op (store guard + card disabled).
- **No optimistic close:** removal only on `APPROVAL_RESOLVED`; refusal keeps card with inline `code/message/how_to_fix` strip (`role="alert"`), plus retry when allowed. Fixes the current stuck-lock defect (E7) by deleting the panel-local `pendingResolution` lock and deriving disabled state from store.

### 6.2 Exact store fields/actions

```ts
type ApprovalResolveStatus =
  | 'pending' | 'resolving' | 'awaiting_server' | 'retryable' | 'expired' | 'stale';
interface ApprovalResolveState {
  readonly status: ApprovalResolveStatus;
  readonly attempts: number;              // manual + automatic? manual only; bound 3
  readonly lastDecision: 'approve' | 'deny' | null;
  readonly refusal: Refusal | null;       // exact named code/message/how_to_fix
  readonly lastAttemptAt: number | null;
}
// UiState
readonly approvalResolve: Readonly<Record<string, ApprovalResolveState>>;
resolveApproval(id, decision, reason?): Promise<ApprovalResolveOutcome>; // signature-compatible: extra return
retryApproval(id, decision, reason?): Promise<ApprovalResolveOutcome>;
applyApprovalRefusal(id: string, refusal: Refusal): void;  // classification matrix above
expireApproval(id: string): void;
dismissApproval(id: string): void;       // expired/stale/nonce-missing only
```
`ApprovalResolveOutcome = { ok: true } | { ok: false; kind: 'retryable'|'terminal'|'transport'; refusal: Refusal }`; `run()` must return the caught `Refusal` (today it returns boolean and drops it, E4); no new wire types.

### 6.3 UI/component plan

- Replace inline markup in `ApprovalOverlay.tsx` with `ApprovalCard.tsx` (overlay keeps map + focus management) and `ApprovalStatusStrip.tsx`; add `ApprovalRetryButton` (only `retryable`); `ApprovalArgsPreview` (contract-gated §5). Preserve `role="dialog"`/`aria-modal` and labels; add/verify focus trap, initial focus policy (Deny-first per W11 D11) and return-focus on close in this work — the current file shows the dialog attributes but focus-trap/Escape behaviour is **not evidenced** by this read, so it must be verified alongside these changes.
- Actions enabled only when `status === 'pending'` (or `retryable` for the Retry control); `awaiting_server` shows resolving with `aria-busy`; refusal strip shows `refusal.code + message + how_to_fix` verbatim; `stale/expired` read-only + dismiss.
- Strings: all new copy in `STRINGS.approval` (`status.*`, `refusalTitle`, `retry`, `reconnectedAwaitingReplay`, `alreadyDecided`, `argsBoundByHash`, `argsPreview.label/redacted/truncated`); no hardcoded strings.

---

## 7. Tests plan (no code now)

**Store unit (`web/tests/approval*.test.ts`, extend existing `toolActivity.store.test.ts` harness):**
1. `hydrateApprovals` dedupes by id across hello/session sources; merge preserves `resolving`/`retryable` state; unknown replay after resolved ignored.
2. reconnect: `reconnecting` → stale; hello replay → stale false; no command sent by hydration.
3. refusal matrix: transport → retryable; DECISION_INVALID → retryable; REPLAY/NOT_FOUND → stale; TIMEOUT → expired; attempt cap; ACK-only → awaiting_server then retryable on timeout.
4. no optimistic removal/lock; `APPROVAL_RESOLVED` clears record+state; expiry timer.
5. persistence fallback (if enabled): safe field set only, nonce stored only for pending, deleted on resolve/expire, TTL-filtered restore.

**E2E (`e2e/specs/approval-flow.spec.ts`, mocked WS):**
6. replay: close socket → app reconnects (mock accepts new route) → pending request re-sent → one card, actions enabled; resolved id not replayed; no auto-command.
7. retry: approve → server ERROR `E_WS_DISCONNECTED` → card stays with named strip + Retry; retry sends one new command with same id/nonce; success closes only on `APPROVAL_RESOLVED`.
8. ACK-only timeout (Playwright clock) → retryable, not success.
9. expired/stale: no resend, dismiss only.
10. args privacy (after contract patch): redacted preview visible; no secret in DOM/console/storage; absent field → hash-only honest copy.
11. a11y: focus moves to Deny, trap, return focus; Escape never resolves; status strip live region.

**Backend (owner b-core):** hello replay test using `pending_approval_events` (pending replayed once, expired/decided excluded, restart recovery excludes); no contract change.

---

## 8. What can / cannot be done without contract changes

| Capability | No contract change? | Requirement / caveat |
|---|---|---|
| Server replays pending `APPROVAL_REQUEST` events on hello (survives reload/reconnect in same process) | **YES** — existing frozen event/fields | Backend wiring in `ws.rs` + run-scope policy; no new command/event; skipped after restart per recovery |
| Client dedupe/merge, stale banner, no auto-send | **YES** | Store hydration + reducer merge rules |
| Session/local storage fallback for page reload | **YES** (pure UI) | Capability secret at rest; default off; strict TTL/cleanup; privacy review |
| Retry/refusal UX, ACK-only timeout, terminal/stale classes | **YES** | Store `approvalResolve` state machine; remove panel lock; `run()` must return refusal |
| Args **body/preview** display with redaction | **NO** | Requires additive `APPROVAL_REQUEST.args_preview` (or equivalent) + producer redaction + contract patch row |
| Run-scoped replay request | **NO** | Needs new command/REST or hello run hint; v1.2 policy must be "all pending, bounded" |
| Verification that displayed args match `args_sha256` | **NO** | Client has no args; server-side preview/redaction is the only proof |
| Replay after process restart | **NO (by policy)** | Recovery expires pendings/terminates runs; UI must show expired/interrupted, never resurrect |

---

## 9. Rollout / backlog seeds (for the roadmap)

1. **B-backend:** wire `pending_approval_events` into hello replay (same-process only), bounded/deduped, tests. Acceptance: reconnect test sees one replayed pending approval; restart sees none.
2. **F-store:** add `approvalsStale/lastApprovalReplayAt/approvalReplaySource`, `hydrateApprovals`, `approvalResolve` state machine, `retryApproval`, refusal classifier; `run()` returns refusal; remove panel-local lock. Acceptance: store unit matrix + no stuck card.
3. **F-overlay:** split `ApprovalCard`/`ApprovalStatusStrip`/`ApprovalArgsPreview`; wire states/retry/dismiss/focus; copy in `STRINGS.approval`. Acceptance: e2e 6-11.
4. **Contract (only if approved):** `APPROVAL_REQUEST.args_preview` additive field + CHANGE-LEDGER row + contract matrix/schema tests + Rust producer redaction + TS decode. Acceptance: privacy tests, per-tool redaction fixtures.
5. **Security review:** nonce-at-rest decision (R2 flag), redaction rules, XSS/attribute/log/storage scans.

---

## 10. Open questions for Lead

1. Approve server hello replay (R1) with "all pending approvals, bounded" run policy, or prefer latest-active-run only?
2. Approve `args_preview` contract patch for 1.3, and who owns redaction policy (b-core + security)?
3. Is the sessionStorage fallback (R2) allowed at all behind a flag, or is replay-only required?
4. Retry policy bound: 3 manual attempts per approval acceptable? Should `E_APPROVAL_DECISION_INVALID` permit re-choice (nonce not consumed per server command validation) — confirm with backend.
5. Update **items 10/11/15** acceptance in the W12/backlog sheet with the evidence/acceptance rows above.
