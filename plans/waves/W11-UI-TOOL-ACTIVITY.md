# W11 — UI APPROVAL / TOOL ACTIVITY DESIGN (SEAT G)

**Type:** design plan only — **no product code written, no tests executed by this seat, no commit.**
**Write scope:** this file only (`plans/waves/W11-UI-TOOL-ACTIVITY.md`). Read-only everywhere else.
**Repo/HEAD:** `C:\Users\Riley\Desktop\hiverouter-v1` · `9651722` at work start.
**Upstream decision this plan depends on:** `plans/waves/W10-TOOL-LOOP-DECISION.md` (decision **NO for 1.1**; B1–B7 blockers; §4.3/§4.4 target design for 1.2 Phase B). This seat designs the **UI half** of B6 (approval UX) and the tool-activity surface for B1/B4, so that when the contract patch lands the frontend is decision-complete.
**Skills used:** `/forgewright-recon` (§2), `/forgewright-lead` (§12 acceptance gates), `/logic-gatekeeper` (Phase 1 + Phase 2, §1), `/integration-logic` (§3 call chain), `/logic-debugger` (§8 state machine + failure matrix).

---

## 1. PHASE 1 — logic-gatekeeper (written before any design/file work)

```json
{
  "phase1": {
    "architecture": "Three layers hold the state: (a) the frozen contract decoder in web/src/api/contract.ts (EventName union + decodePayload) turns wire frames into DecodedEvent; (b) the zustand UiState in web/src/state/store.ts holds `approvals`, `messages`, `tasks`, and would hold the new `toolActivity` list; (c) presentation components (ApprovalOverlay.tsx today; planned ApprovalCard.tsx + ToolActivityPanel.tsx) render state and call store actions. The server side (hiverouter-tools approval.rs, hiverouter-core adapter) owns the truth: approval row, nonce, TTL, args_sha256, execution. The UI never decides; it renders and requests.",
    "dataFlow": "Untrusted data enters as (1) WS EVENT frames: APPROVAL_REQUEST (extended), TOOL_CALL, TOOL_RESULT, APPROVAL_RESOLVED and arbitrary malformed payloads; (2) operator input: approve/deny clicks, keyboard, TTL expiry wall-clock; (3) stale/resync frames. The decoder validates → store reducers mutate → components render → operator decision becomes APPROVAL_RESOLVE {approval_id, nonce, decision} and nothing else. The UI must never send args, never fabricate outcomes, and never treat ACK as approval.",
    "edgeCases": [
      "Approval arrives without nonce (legacy/back-compat frame) while the frozen APPROVAL_RESOLVE requires one: card must stay read-only with the existing named contract-gap copy; no invented nonce, buttons disabled.",
      "Operator double-clicks Approve: exactly one APPROVAL_RESOLVE frame; second click is a no-op because status is `resolving`.",
      "ACK for APPROVAL_RESOLVE arrives but APPROVAL_RESOLVED never does (drop): card stays `resolving` until the 15 s command timeout, then returns to `pending` with a named E_WS_DISCONNECTED toast — never a fake approval.",
      "TTL expires while the card is open or while `resolving`: card becomes `expired` read-only with E_APPROVAL_TIMEOUT copy; no automatic approve/deny, no retry of a consumed nonce.",
      "Replayed nonce / already-resolved elsewhere: server ERROR E_APPROVAL_REPLAY or APPROVAL_RESOLVED for an id we do not show → toast by name; unknown-id events never create ghost cards or duplicate activity rows.",
      "TOOL_RESULT arrives before its TOOL_CALL (out-of-order fan-out): render an `unmatched` row (do not drop), then reconcile on the late TOOL_CALL; duplicate TOOL_CALL/TOOL_RESULT for the same tool_call_id must upsert, never duplicate.",
      "TOOL_RESULT has an outcome token this build does not know: the wire enum is closed (`ToolOutcome` snake_case), so an unknown token is a **schema violation** — decode rejects it as E_SCHEMA_INVALID (contract-drift toast), never mapped to success and never a partial store mutation.",
      "TOOL_RESULT arrives without code/bytes/sha256 (all optional on the wire): absent stays absent (render '—'), never fabricated 0/empty-hash.",
      "Cancel/kill while awaiting approval: card moves to `cancelling` (buttons disabled) and is removed only on a server terminal signal (APPROVAL_RESOLVED / TOOL_RESULT cancelled / RUN_STATUS terminal) — no optimistic cancellation claim.",
      "Resync/reconnect while activity is on screen: rows are kept but marked `stale` until SERVER_HELLO/resnapshot; replayed APPROVAL_REQUEST replaces the same approval_id instead of stacking a second modal.",
      "Tool activity exceeds the retention cap (200): oldest rows evicted, footer marks truncation; no silent unbounded growth."
    ],
    "rootCause": "The current UI has no model-tool vocabulary on the wire and no state machine for model-originated side effects: the decoder rejects unknown TOOL_CALL/TOOL_RESULT events as E_SCHEMA_INVALID, APPROVAL_REQUEST carries only {approval_id, tool, decision?, nonce?} so a card cannot show run/task/tool-call identity, args binding, or TTL, the ApprovalOverlay has no resolving/expired/stale states or focus management, and nothing renders tool activity at all — so approval UX cannot be truthful until the contract patch (W10 B1) lands and the UI adds a validated decode → store state machine → card/log surface with no optimistic success."
  }
}
```

**Phase 2 (constraints for the future implementation):**
- **Entry checks at the decoder**, before any store mutation: `TOOL_CALL` requires strings (`run_id`, `task_id`, `tool_call_id`, `tool`, `args_sha256`) + numeric `turn`; `TOOL_RESULT` requires the same identity + closed `outcome` enum; extended `APPROVAL_REQUEST` fields are **optional for legacy compat** but each is validated when present (`nonce`/`args_sha256` strings, `expires_at` RFC3339), and a frame without nonce stays non-resolvable (existing honest gap). Bounded preview (≤2 KiB if the field lands), no unknown extra semantics. Malformed → existing `E_SCHEMA_INVALID` path, state untouched.
- **Callers stay stable:** `ApprovalOverlay` remains the mount point in `App.tsx`; `resolveApproval(approvalId, decision, reason?)` keeps its signature (its internals gain the pending guard); `contract.ts` additions are additive; no backend change in this plan.
- **Why the same contradiction now fails closed:** a nonce-less, TTL-less, args-unbound request can no longer be rendered as an actionable approval (it stays read-only), and an unacknowledged decision can no longer disappear optimistically (card stays `resolving` until the server says otherwise or a named timeout fires).

---

## 2. RECON — forgewright-recon ledger (READ this session)

| # | File | Facts (path:line) |
|---|---|---|
| R1 | `backend/crates/hiverouter-contract/src/event.rs` | `EventName` = **21** variants (18-64); `ApprovalRequest` wire payload = `{approval_id, tool, decision?: Option<String>}` only (242-254) — **no nonce/run/task/tool_call/args/expiry**; no `ToolCall`/`ToolResult` variants (enum ends at `Pong`, 60); decode dispatch lacks tool events (334-463). |
| R2 | `backend/crates/hiverouter-tools/src/approval.rs` | Server-side truth already exists: `ApprovalRequest {approval_id, run_id, task_id, tool_call_id, tool: ToolId, permission_class, args_sha256, created_at, expires_at, nonce}` (85-106); `is_expired()` (108-118); TTL/nonce/replay machinery. The wire gap is transport, not logic. |
| R3 | `web/src/api/contract.ts` | `EventName` union/`EVENT_NAMES` = 21 (268-316); unknown event → `E_SCHEMA_INVALID` (`validateEnvelope` 468, `parseEventFrame` 786) — new events **must** be added to the union or they are dropped as schema errors; `ApprovalRequestPayload` = `{approval_id, tool, decision?, nonce?}` (223-231); decode only reads those four fields (649-671, nonce optional, `null` when absent); `ApprovalResolvePayload` = `{approval_id, nonce, decision: 'approve'|'deny', reason?}` (70-76). |
| R4 | `web/src/state/store.ts` | `ApprovalRecord` = `{approvalId, tool, nonce, decision, requestedAt}` (55-61) — no run/task/tool-call/args/expiry/status; `approvals` list in `UiState` (123); `APPROVAL_REQUEST` replaces by `approvalId` (325-336), `APPROVAL_RESOLVED` filters it out (338-345); `resolveApproval` (573-599): finds record, refuses when `nonce` missing with `E_SCHEMA_INVALID` contract-gap message, then `run('APPROVAL_RESOLVE', {approval_id, nonce, decision, reason?})`; **no `resolving` guard → double click sends two frames**; `run()` (400-433) sets `pendingCommands`, awaits ACK, toasts only transport-class refusals; ACK does not dismiss the card (dismissal depends on APPROVAL_RESOLVED). |
| R5 | `web/src/panels/ApprovalOverlay.tsx` | Maps `approvals` to a modal per record (12-64); shows tool + `approval_id` prefix + nonce-missing copy; Approve/Deny buttons disabled only when `nonce === null` (43, 53); no TTL, no args hash, no run/task/tool identity, no resolving/expired/stale state, no focus trap/return focus, no Escape handling, no double-submit guard. |
| R6 | `web/src/panels/StatePack.tsx` + `ToastRegion.tsx` | Existing a11y primitives to reuse: `role="status"` loading (17-23), `role="alert"` error (25-32), toast region `aria-live="assertive"` + named refusal code/message/how_to_fix (14-46). |
| R7 | `web/src/panels/DiffDecisionModal.tsx`, `SettingsPanel.tsx` | Decision-gated modal pattern (role=dialog, close-only); Settings shows diff gate (119-121) and health/probe states — no tool/approval activity surface exists anywhere. |
| R8 | `web/src/state/strings.ts` | Approval copy is minimal: `approval.title/toolLabel/nonceMissing` (215-221); approve/deny labels (100-101); no tool-activity strings, outcome labels, TTL copy, or aria labels. |
| R9 | `plans/waves/W10-TOOL-LOOP-DECISION.md` | Decision NO for 1.1; blockers B1 (contract vocabulary + approval nonce/args), B2 (suspension), B3 (persistence), B4 (assembly/execution), B5 (caps), B6 (**approval UX/security undefined — this plan's owner**), B7 (deterministic fake-provider harness); target payloads §4.4 (TOOL_CALL {run_id, task_id, tool_call_id, tool, args_sha256, turn}; TOOL_RESULT {run_id, task_id, tool_call_id, outcome, code?, bytes?, sha256?}; APPROVAL_REQUEST + nonce/run_id/task_id/tool_call_id/args_sha256/expires_at + optional redacted args preview); lifecycle §4.3; failure modes §5; UI tests §6 items 6-8; e2e items 9-11. |
| R10 | `e2e/` | No tool-loop/approval spec and no fake-provider helper (W10 B7); existing e2e only asserts absence of stale approval copy (`live-app.spec.ts:245-264`). |

---

## 2a. ALIGNMENT WITH THE IN-FLIGHT B1 CONTRACT PATCH (verified read-only)

While this seat was planning, another seat's **uncommitted** B1 worktree patch was visible (not authored or touched by this seat; committed HEAD still reads 21 events). Read-only inspection shows the patch matches W10 §4.4 and this plan field-for-field:

| Wire item | Patch evidence (worktree, uncommitted) | UI mapping |
|---|---|---|
| `TOOL_CALL` name | `event.rs:62` (`ToolCall`), `:120` (`"TOOL_CALL"`) | add to `EventName`/`EVENT_NAMES` |
| `ToolCallPayload` | `event.rs:327-344`: `{run_id, task_id, tool_call_id, tool, args_sha256, turn: u32}` | exact fields in §5 |
| `TOOL_RESULT` name | `event.rs:64` (`ToolResult`), `:121` (`"TOOL_RESULT"`) | add to union |
| `ToolResultPayload` | `event.rs:346-366`: `{run_id, task_id, tool_call_id, outcome: ToolOutcome, code?: Option<String>, bytes?: Option<u64>, sha256?: Option<String>}` | exact fields; optional stays `| null`, never 0/empty |
| `ToolOutcome` | `event.rs:286-306`, `#[serde(rename_all="snake_case")]`, **closed five-value enum** (`ok/refused/denied/timeout/cancelled`) | D3: unknown token = decode error, no normalisation |
| `APPROVAL_REQUEST` extension | `event.rs:252-282`: existing `approval_id`, `tool`, `decision?` + **Option** fields `nonce`, `run_id`, `task_id`, `tool_call_id`, `args_sha256`, `expires_at: Option<Timestamp>` (all `skip_serializing_if = "Option::is_none"`) | legacy frames still valid; missing → legacy read-only card (current behavior preserved) |
| `expires_at` wire format | `Timestamp` = RFC3339 `YYYY-MM-DDTHH:MM:SSZ` string, second precision (`timestamp.rs:29,50,89-112`) | D2 confirmed: `Date.parse`, countdown in whole seconds |
| Event census | patch adds exactly the two event names → **23** events (committed HEAD = 21) | parity/golden/matrix/`EVENTS` count updated in same patch |
| Residual gap | **No `args_preview`**; no `permission_class` on `TOOL_CALL`; no `turn` on `TOOL_RESULT` (turn comes from the matching `TOOL_CALL`) | D4 hash-only accepted; group by `run_id`+`tool_call_id`; do not invent fields |

**Plan revision adopted from this check:** D3 is strict closed-enum decode; `args_preview` is conditional (hash-only default); `outcome_raw` is removed from the TS contract. Final field names/types must be reconciled once the patch is committed — if the merged patch differs, this table is the diff checklist.

---

## 3. INTEGRATION-LOGIC — call chain and where it breaks today

| Hop | Control / value | Current (evidence) | Required for approval/tool activity |
|---|---|---|---|
| 1 Render | approval card / activity row | `ApprovalOverlay` modal (R5); no activity surface (R7) | `ApprovalCard` + `ToolActivityPanel` rendering `approval`/`toolActivity` state |
| 2 Handler | Approve / Deny buttons | `resolveApproval(id,'approve'\|'deny')` (R5) | same action, now guarded by `status === 'pending'` |
| 3 Client call | `APPROVAL_RESOLVE {approval_id, nonce, decision, reason?}` | R4 573-599 | unchanged shape; one frame per decision |
| 4 Request/route | WS command → server gate | frozen command exists; server approval machinery exists (R2) | unchanged; server returns ACK then APPROVAL_RESOLVED/ERROR |
| 5 Service | suspend → verify_grant → execute once | **missing** (W10 B2/B4) | backend wave; UI only reflects `awaiting_approval` → result |
| 6 Store | approvals list mutation | replaces by id, removes on APPROVAL_RESOLVED (R4) | extend record; add resolving/expired/stale; activity upsert |
| 7 Events | APPROVAL_REQUEST/APPROVAL_RESOLVED on wire | 21-event contract; approvals have no producer yet (W10 B1/B2) | contract patch: extended APPROVAL_REQUEST + new TOOL_CALL/TOOL_RESULT |
| 8 Provider | tool call args/outcome | absent from wire (R1/R3) | server publishes TOOL_CALL then TOOL_RESULT per call |
| 9 Response | named refusals | E_APPROVAL_NOT_FOUND/REPLAY/TIMEOUT exist in copies (strings 316+) | card/activity show code + how_to_fix; no spinner |
| 10 Render-back | card closes only on server truth | ACK does not dismiss (correct today) | keep; add resolving state so ACK ≠ success visually |
| 11 Re-entry | reload/resync | `approvals` rebuilt only from replayed events (no fetch) | mark stale on reconnect; replace by approval_id; never ghost |

**Integration verdict:** the break is **at hop 7 (contract) and hop 5 (server execution)** — not in the UI store contract. The UI work in this plan is therefore *blocked on the W10 B1 patch* and can be staged so it degrades safely if events are absent.

---

## 4. DECISIONS (decision-complete; alternatives rejected)

| ID | Decision | Alternative rejected — why |
|---|---|---|
| D1 | **Wire payloads exactly as W10 §4.4, confirmed against the in-flight B1 patch (§2a)**: `TOOL_CALL {run_id, task_id, tool_call_id, tool, args_sha256, turn}`; `TOOL_RESULT {run_id, task_id, tool_call_id, outcome, code?, bytes?, sha256?}`; `APPROVAL_REQUEST` += optional `{nonce, run_id, task_id, tool_call_id, args_sha256, expires_at}`. No extra fields are invented; `args_preview` remains conditional and is absent from the patch today. | Inventing UI-only fields (e.g. `permission_class` in TOOL_CALL) would force a second contract patch; server already has permission class but W10 keeps it off the wire for 1.2 — defer. |
| D2 | `expires_at` is an **RFC3339 UTC string**, matching the envelope `ts` convention; UI parses with `Date.parse`, treats unparseable as `expired` (fail closed). | Unix-seconds number: mixes units with the rest of the wire; local clock formatting: untestable. |
| D3 | **Closed-enum decode, no normalisation:** `ToolOutcome` is a fixed five-value wire enum (`ok\|refused\|denied\|timeout\|cancelled`); an unknown token or any malformed tool/approval payload is rejected by the decoder (`E_SCHEMA_INVALID`, contract-drift toast) with **no store mutation**. Forward-compatible `unknown` mapping is only for a future vocabulary expansion and must land with the contract patch, not silently in the UI. | Ignoring unknown events (contract says unknown event name = schema error; silently dropping hides contract drift); coercing unknown rows to `ok` (fake success). |
| D4 | **The in-flight B1 patch carries no `args_preview`** (verified §2a). Accepted v1.2 UI: render `args_sha256` (12-hex prefix + full-hash copy action) and nothing derived; **if** the final patch adds a server-redacted preview (≤2 KiB, security-reviewed) render it as plain text. The UI never reconstructs args from the model, never parses model JSON for display, and never sends args back. | Reconstructing args client-side (untrusted input, leak risk); showing hash-only *after* a preview field exists (missed operator context); inventing a preview field outside the contract patch (frozen-contract violation). |
| D5 | Activity retention: **cap 200 entries FIFO**, dedupe by `tool_call_id` (upsert), mark `truncated` when eviction occurred, render footer note. | Unbounded list (memory/DoS); dropping oldest silently (hides evidence). |
| D6 | Ordering: attach envelope `seq` when available; render **newest-first**; within one run/turn use `(turn, seq, arrival)` ascending. Requires the store subscription to receive the envelope (`api.subscribe((event, envelope) => …)`), which it currently discards. | Sorting only by arrival (no seq) loses deterministic order after resync; needless full re-sort on every event. |
| D7 | Placement: **Agents panel gets a "Tool activity" section** (`ToolActivityPanel`), approval card stays a global overlay in `App.tsx`. Chat transcript is **unchanged** in 1.2 (no tool bubbles) — chat rendering moves in a later wave only if product asks. | Chat-inline tool chips: larger blast radius in the most-tested component and duplicates the panel; new dock: more layout risk than value for 1.2. |
| D8 | Approval status machine: `pending → resolving → resolved\|denied`, plus `expired`, `stale`, `cancelling`; buttons enabled **only** in `pending`; `resolving` shows `aria-busy`; expired/stale are read-only (dismiss only). `ACK` is not a terminal state; only `APPROVAL_RESOLVED`/`ERROR`/timeout transitions the card. | Closing on ACK (fake success); auto-deny on TTL (destructive without operator); allowing retry after `E_APPROVAL_REPLAY` (nonce consumed). |
| D9 | Cancel/kill while waiting: on ACK of `CANCEL_RUN`/`KILL_WORKER` set card `cancelling`; remove only on server terminal event (`APPROVAL_RESOLVED` deny/cancelled, `TOOL_RESULT outcome=cancelled`, or `RUN_STATUS` terminal). | Optimistically removing the card (claims cancellation before server proves it). |
| D10 | Feature gate: new surface behind `VITE_UI_TOOL_ACTIVITY` (`'1'` enables panel + extended states; unset/`'0'` keeps today's overlay behavior and decoders simply store nothing beyond legacy fields). Contract patch ships separately. | Always-on (breaks 1.1 UI if server is old); runtime capability probe (backend has no `/diagnose` capability field for this yet). |
| D11 | Safe-default focus: card is `role="alertdialog"`, initial focus on **Deny** (destructive-avoidance), focus trap, return focus on close; `Escape` never approves and never denies — it only dismisses an expired/stale card. | Focusing Approve by default (accidental side effect); Escape = deny (destructive without explicit intent). |
| D12 | Live semantics: approval card is announced once (`alertdialog`); TTL countdown is visual (`aria-live="off"`) with polite announcements at 30 s and 10 s; activity rows are in a `role="log" aria-live="polite"` region; outcome is text+shape, never colour-only; reduced motion disables pulse/countdown animation. | Fully live countdown (screen-reader spam); colour-only chips (WCAG 1.4.1); motion-only progress (reduced-motion gate). |

---

## 5. DATA CONTRACTS (TypeScript, additive)

```ts
// web/src/api/contract.ts — additive to EventName / EVENT_NAMES / EventPayloads
export type ToolOutcome = 'ok' | 'refused' | 'denied' | 'timeout' | 'cancelled';

export interface ToolCallPayload {
  readonly run_id: string;
  readonly task_id: string;
  readonly tool_call_id: string;
  readonly tool: string;
  readonly turn: number;
  readonly args_sha256: string;
}
export interface ToolResultPayload {
  readonly run_id: string;
  readonly task_id: string;
  readonly tool_call_id: string;
  readonly outcome: ToolOutcome;      // closed five-value wire enum (strict decode)
  readonly code?: string | null;
  readonly bytes?: number | null;
  readonly sha256?: string | null;
}
// APPROVAL_REQUEST becomes (existing fields kept):
export interface ApprovalRequestPayload {
  readonly approval_id: string;
  readonly tool: string;
  readonly decision?: string | null;
  readonly nonce?: string | null;         // kept optional for 1.1 back-compat
  readonly run_id?: string | null;
  readonly task_id?: string | null;
  readonly tool_call_id?: string | null;
  readonly args_sha256?: string | null;
  readonly expires_at?: string | null;    // RFC3339 UTC
  readonly args_preview?: string | null;  // conditional: absent from the in-flight B1 patch; render only if it lands (server-redacted, ≤2 KiB)
}
```
Event count becomes **23** — committed HEAD has 21; the in-flight B1 patch (§2a) adds exactly `TOOL_CALL`/`TOOL_RESULT`. Golden schemas + matrix rows + contract-test census must be updated in the same contract-patch commit; command count 17 variants / 14 frozen unchanged; refusal count unchanged unless `E_TOOL_LOOP_LIMIT` is also added — W10 keeps that optional.

```ts
// web/src/state/store.ts — additive types
export type ApprovalStatus = 'pending' | 'resolving' | 'resolved' | 'denied' | 'expired' | 'stale' | 'cancelling';
export interface ApprovalRecord {
  readonly approvalId: string;
  readonly tool: string;
  readonly nonce: string | null;
  readonly decision: string | null;
  readonly requestedAt: number;
  readonly runId: string | null;
  readonly taskId: string | null;
  readonly toolCallId: string | null;
  readonly argsSha256: string | null;
  readonly argsPreview: string | null;
  readonly expiresAtMs: number | null;   // null = unknown TTL → no countdown, never auto-expire
  readonly status: ApprovalStatus;
  readonly refusal: Refusal | null;      // last named failure for this card
}

export type ToolActivityStatus = 'requested' | 'running' | ToolOutcome;
export interface ToolActivityEntry {
  readonly toolCallId: string;
  readonly runId: string;
  readonly taskId: string;
  readonly tool: string;
  readonly turn: number;
  readonly argsSha256: string;
  readonly status: ToolActivityStatus;
  readonly code: string | null;
  readonly bytes: number | null;
  readonly sha256: string | null;
  readonly seq: number | null;            // envelope seq when present
  readonly requestedAt: number;
  readonly updatedAt: number;
}

// UiState additions
readonly toolActivity: readonly ToolActivityEntry[];
readonly toolActivityTruncated: boolean;
readonly approvalStale: boolean;          // set on reconnect, cleared on SERVER_HELLO/resnapshot
```

New store actions (signatures): `dismissApproval(approvalId: string): void`; `expireApproval(approvalId: string): void` (internal timer + deterministic test seam); `markApprovalsStale(): void` (reconnect); `clearApprovalStale(): void` (SERVER_HELLO). `resolveApproval` signature unchanged.

**Reducer rules (logic-debugger invariants):**
- `TOOL_CALL`: upsert by `tool_call_id` (`requested`, fill identity fields); duplicate id with same payload → no-op; conflicting turn/tool → last event wins but row keeps a `seq` (no duplicate row).
- `TOOL_RESULT`: upsert by `tool_call_id`; if no row, create `unmatched` row (**never dropped**); terminal status is sticky — a second terminal result for the same call is ignored (counted, not rendered twice); an unknown outcome token never reaches the reducer (the decoder rejects the frame as `E_SCHEMA_INVALID`).
- `APPROVAL_REQUEST`: replace-by-`approval_id` (no stacking); a different nonce for an existing id resets status to `pending` only if current status is not `resolving`.
- `APPROVAL_RESOLVED`: if id known → `status='resolved'|'denied'` from decision then remove from `approvals` after one render tick (card closing), regardless keep a transient activity-free toast; if id unknown → ignore silently (no ghost card).
- TTL: per-record timer scheduled from `expiresAtMs`; when it fires → `status='expired'`, toast `E_APPROVAL_TIMEOUT` copy; timers cleared on resolve/remove/dispose (no leaks).
- Reconnect: `subscribeStatus('reconnecting')` → `approvalStale = true` (card shows "connection lost — decision cannot be sent" and buttons disabled); `SERVER_HELLO` → `approvalStale = false`.

---

## 6. UI SPECIFICATION

### 6.1 ApprovalCard (replaces inline markup in `ApprovalOverlay.tsx`; overlay keeps map+key)
```
role="alertdialog" aria-modal="true" aria-labelledby="approval-title-<id>" aria-describedby="approval-summary-<id>"
Title: "Tool approval requested"            (STRINGS.approval.title)
Summary line (aria-describedby): "<tool> · task <task_id short> · turn <turn>"
Fields (definition list): Tool, Task, Tool call (short id), Args sha256 (12-hex prefix + copy), TTL remaining (mm:ss)
Args preview: server-redacted <pre> (max 2 KiB, text only, scrollable, labelled "Arguments preview (redacted)")
Actions: [Deny] (left, safety-first default focus) [Approve]
Status strip (role="status"): resolving "Sending decision…" / expired "Approval expired — no action taken (E_APPROVAL_TIMEOUT)" / stale "Connection lost — decision disabled" / cancelling "Run is being cancelled…"
Nonce-missing legacy block: existing STRINGS.approval.nonceMissing; buttons disabled; no invented nonce.
```
**State × action matrix**

| status | Approve | Deny | TTL text | Card dismiss |
|---|---|---|---|---|
| `pending` | enabled → `resolving` | enabled → `resolving` | counts; announces 30 s/10 s | only via server resolution / expire |
| `resolving` | disabled (`aria-busy`) | disabled | counts | waits for APPROVAL_RESOLVED/ERROR/15 s timeout |
| `expired` | disabled | disabled | 00:00 | Dismiss button (safe), no auto-action |
| `cancelling` | disabled | disabled | frozen | removed on terminal server event |
| `stale` | disabled | disabled | frozen | Dismiss; reconnect may replay |
| nonce missing | disabled | disabled | n/a | Dismiss (existing copy) |

**Named refusal rendering:** `ERROR.code + message + how_to_fix` shown verbatim in the card's `role="alert"` strip (reuse `ErrorState` semantics); `E_APPROVAL_REPLAY`/`E_APPROVAL_TIMEOUT`/`E_APPROVAL_NOT_FOUND` → read-only expired/stale state + toast; `E_RESYNC_REQUIRED` → card stays `pending`, store marks graphs resyncing; `E_APPROVAL_DECISION_INVALID` → card returns to `pending` with the error strip (operator may choose again only if the nonce was not consumed — server decision, UI does not assume).

### 6.2 ToolActivityPanel (`web/src/panels/ToolActivityPanel.tsx`, rendered inside `AgentsPanel.tsx`)
```
<section aria-labelledby="tool-activity-title">
  <h3 id="tool-activity-title">Tool activity</h3>
  <p role="status" aria-live="polite">Showing N calls · <run label> · newest first</p>
  <ol role="log" aria-live="polite" aria-relevant="additions text">
    <li> <span class="type-mono-meta">#turn</span> <span>tool</span>
         <span class="outcome-chip outcome-ok|refused|denied|timeout|cancelled|requested|running">
           label (+ shape cue: ✓/✕/⊘/⏱/—/…)
         </span>
         <span>code (if any)</span> <span>bytes</span> <span>sha256 (8-hex)</span>
         <button>Select task</button> (focusable; calls selectTask(task_id))
    </li>
  </ol>
  empty: <EmptyState "No tool activity yet — tool calls appear here when a run uses tools." />
  truncated: footer "Earlier activity not retained (cap 200)."
</section>
```
Rules: no spinner is ever used as a stand-in for an outcome; `requested`/`running` show a static dot/text label (not an endless animation — reduced motion; if no result arrives the run's terminal state or `E_TOOL_CANCELLED`/timeout ends it); rows for `refused|denied|timeout|cancelled` always show code + how_to_fix (from refusals table when the code is known, raw code otherwise); rows are keyboard reachable and select the owning task; `unknown` outcome row shows the raw token and a "contract drift" hint.

---

## 7. A11Y SPECIFICATION

| Area | Requirement |
|---|---|
| Dialogs | `role="alertdialog"`, `aria-modal`, labelled title + described summary; focus moves to **Deny** on open; focus trap within card; focus returns to the triggering element on close; page cannot scroll behind (`overflow` lock as today). |
| Keyboard | Tab cycles card controls only; Enter/Space activate focused button; **Escape never approves/denies** — it dismisses only `expired`/`stale`/nonce-missing cards, otherwise it is a no-op (no hidden action). |
| Announced state | status strip `role="status"` for resolving/expired/stale; TTL countdown visible but `aria-live="off"`, thresholds 30 s/10 s announced politely; when the card closes after server resolution announce "Approval approved/denied by operator". |
| Tool log | `role="log"`, `aria-live="polite"`, additions only; each row is a list item with an accessible name including tool + outcome text; outcome never colour-only (text + shape cue); "Select task" buttons have `aria-label="Select task <task_id>"`. |
| Contrast | Use existing tokens only: approve = honey-300 on ink-900 (AA), deny = signal border on panel (AA), meta text ink-500 (5.13), focus ring ink-700 ≥3:1; never amber small text (UI-R-05) and never colour-only status. |
| Motion | No pulsing/animating indicator for `running`; countdown is textual; `prefers-reduced-motion` and the in-app override disable any transition in the card/log (reuse `useAppReducedMotion`). |
| Screen-reader copy | All new strings live in `STRINGS` (no hardcoded), include aria labels; no secrets in labels (`args_sha256` only). |

---

## 8. LOGIC-DEBUGGER — state machine, stale/refuse matrix (each row becomes a test)

| # | Situation | Expected UI behaviour | Must-not |
|---|---|---|---|
| A1 | APPROVAL_REQUEST (full payload) | card `pending`, countdown from `expires_at`, args hash + redacted preview, one modal per approval_id | stack on duplicate frame |
| A2 | duplicate APPROVAL_REQUEST same id | replace fields; status unchanged unless nonce changed | second modal / reset countdown silently |
| A3 | Approve click | status `resolving`, `aria-busy`, exactly one `APPROVAL_RESOLVE` frame | second frame on double click |
| A4 | ACK only, no APPROVAL_RESOLVED | stay `resolving`; after 15 s → `pending` + E_WS_DISCONNECTED toast | dismiss/claim approved |
| A5 | APPROVAL_RESOLVED `approve`/`deny` | close card, announce outcome, activity row updates only on TOOL_RESULT | fabricate success row |
| A6 | ERROR E_APPROVAL_REPLAY | card `expired` read-only + toast code/how_to_fix; no retry | re-send nonce |
| A7 | TTL expiry (client timer) | card `expired`, toast E_APPROVAL_TIMEOUT copy; server truth awaited | auto deny/approve |
| A8 | ERROR E_APPROVAL_NOT_FOUND / APPROVAL_RESOLVED else | remove card + named toast | ghost card |
| A9 | CANCEL_RUN/KILL_WORKER while pending/resolving | card `cancelling`; removed on server terminal; activity row cancelled on TOOL_RESULT | optimistic cancel success |
| A10 | reconnect while card/activity visible | `stale` banner, buttons disabled; rows kept | drop evidence or pretend live |
| A11 | TOOL_RESULT before TOOL_CALL | `unmatched` row shown; reconcile on late call | drop event |
| A12 | duplicate TOOL_CALL/TOOL_RESULT | upsert/dedupe; second terminal ignored | duplicate rows/text |
| A13 | unknown tool outcome token | decoder rejects the frame (`E_SCHEMA_INVALID`, contract-drift toast); no row mutation | map to ok / render a fabricated chip |
| A14 | malformed tool/approval payload | decoder `E_SCHEMA_INVALID` toast; state untouched | partial mutation |
| A15 | cap exceeded | oldest evicted, truncation footer | unbounded growth |
| A16 | args_preview contains secret-shaped text | mask `sk-…`, `Bearer …`, `DEEPSEEK_API_KEY=…` before render; assert no key text in DOM | echo secret |

---

## 9. FAILURE MODES (design-level; each needs RED before GREEN in the wave)

| ID | Failure | Expected |
|---|---|---|
| F1 | Contract events absent (old server, flag on) | decoders unchanged for legacy events; no tool rows; panel empty state; never error loop |
| F2 | Malformed/partial tool payloads | `E_SCHEMA_INVALID` toast, no state mutation, socket stays open |
| F3 | Nonce missing but approve allowed by bug | forbidden by D8 matrix + test "buttons disabled when nonce null" |
| F4 | Double submit / lost ACK | one frame; resolving→timeout→pending with named refusal |
| F5 | Replayed/expired approval | read-only card + named code; no second execution possible from UI |
| F6 | Cancel race (approve clicked, cancel arrives) | whichever server processes first is truth; UI shows server event; no optimistic claim; activity row reflects TOOL_RESULT outcome |
| F7 | Reconnect with pending approval | stale read-only; on resnapshot replayed APPROVAL_REQUEST replaces by id |
| F8 | Oversize/secret args preview | server truncates/redacts; UI additionally masks + hard caps render at 2 KiB |
| F9 | Activity cap eviction while card open | card unaffected (approvals separate list); footer marks truncation |
| F10 | Renderer throws on unexpected value | decoder is the only place enum values are normalised/rejected (closed enum, E_SCHEMA_INVALID); panel-level ErrorBoundary still wraps the surface so a renderer bug shows ErrorState, not a blank app |

---

## 10. FILE-BY-FILE CHANGE (future implementation; no code in this seat)

| File | Change |
|---|---|
| `web/src/api/contract.ts` | Add `ToolCallPayload`, `ToolResultPayload`; extend `ApprovalRequestPayload`; add `TOOL_CALL`/`TOOL_RESULT` to `EventName` union, `EVENT_NAMES`, `EventPayloads`, `DecodedEvent`, `decodePayload` (+ validators: required strings/enums, RFC3339 parse, preview cap, unknown-outcome normalisation). Keep unknown **event** names a schema error. |
| `web/src/state/store.ts` | Add types/state from §5; extend `APPROVAL_REQUEST`/`APPROVAL_RESOLVED` reducers; add `TOOL_CALL`/`TOOL_RESULT` reducers; add `resolving` guard + 15 s timeout handling in `resolveApproval`; TTL timers + dispose cleanup; reconnect stale flags; pass envelope `seq` through `applyEvent` (change `api.subscribe((event) => …)` → `(event, envelope) => …`). |
| `web/src/state/strings.ts` | Add `toolActivity.{title,empty,truncated,outcome.*,selectTaskAria,driftHint}` and `approval.{summary,argsHash,argsPreview,ttlRemaining,resolving,expired,stale,cancelling,approvedAnnounce,deniedAnnounce,copyHash}`; all aria labels. |
| `web/src/panels/ApprovalCard.tsx` (new) | Card per §6.1 (+ focus management helper). |
| `web/src/panels/ApprovalOverlay.tsx` | Keep list/portal; render `ApprovalCard`; no inline field markup. |
| `web/src/panels/ToolActivityPanel.tsx` (new) | Log per §6.2 (+ empty/truncated/stale/unknown). |
| `web/src/panels/AgentsPanel.tsx` | Mount `ToolActivityPanel` below agent cards (flag-gated). |
| `web/src/App.tsx` | No structural change; overlay mount stays; feature flag read inside new components. |
| `web/tests/` | New `contract.toolEvents.test.ts`, `toolActivity.store.test.ts`, `approvalCard.state.test.ts`; extend `contract.test.ts`/`store.test.ts` only with additive cases. |
| `e2e/` (other owner) | New `specs/tool-loop.spec.ts`, `specs/approval-flow.spec.ts`; fake-provider helper per W10 B7. |

**No backend, docs, or release changes in this plan; contract patch is a separate owner (CONTRACT + Lead) with its own CHANGE-LEDGER row.**

---

## 11. TESTS (named; commands; acceptance)

### 11.1 Vitest (deterministic, offline; `cd web && npm.cmd run typecheck && npm.cmd test -- --run`)
| Test file | Cases (names) | Acceptance |
|---|---|---|
| `tests/contract.toolEvents.test.ts` (new) | `decodes TOOL_CALL with required fields`; `rejects TOOL_CALL without args_sha256`; `decodes all five outcomes`; `rejects an unknown outcome token as E_SCHEMA_INVALID`; `decodes extended APPROVAL_REQUEST and keeps legacy frame valid`; `rejects APPROVAL_REQUEST without approval_id/tool`; `rejects malformed expires_at`; `event vocabulary is 23 and unknown event names stay E_SCHEMA_INVALID` | all pass; no state; parity test still green |
| `tests/toolActivity.store.test.ts` (new) | `TOOL_CALL upserts one row per tool_call_id`; `TOOL_RESULT maps ok/refused/denied/timeout/cancelled`; `result before call renders unmatched then reconciles`; `duplicate terminal results are ignored`; `unknown outcome token is rejected at decode and mutates nothing`; `newest-first order with seq fallback`; `cap 200 evicts oldest and sets truncated`; `approval request replaces by id (no stacking)`; `double approve sends exactly one frame`; `ACK alone does not dismiss the card`; `command timeout returns card to pending with E_WS_DISCONNECTED`; `APPROVAL_RESOLVED closes and announces`; `ERROR E_APPROVAL_REPLAY/ TIMEOUT / NOT_FOUND → read-only + toast`; `TTL timer expires card (fake timers)`; `reconnect marks stale and disables actions`; `cancel while pending → cancelling until terminal event`; `preview masking leaks no key-shaped text` | all pass; zero sends on disabled paths |
| `tests/approvalCard.state.test.ts` (new, logic-only) | exported pure `approvalButtonState(record)` and `formatTtl(ms, now)` helpers: `buttons enabled only in pending with nonce`; `resolving/expired/stale/cancelling disable both`; `Escape action is dismiss only for expired/stale`; `ttl formatting mm:ss clamps at 00:00` | all pass |
| existing `tests/contract.test.ts`, `tests/store.test.ts` | additive only (event count/union parity; legacy approval frame still decodes) | unchanged green |

### 11.2 E2E (fake provider; no paid call; `cd e2e`, `E2E_APP_READY=1`, base URL recorded)
| Spec | Scenario | Acceptance |
|---|---|---|
| `specs/tool-loop.spec.ts` (new, Phase A) | submit → fake provider turn1 returns `file_read` tool call → tool executes in temp jail → turn2 final text | activity rows `requested → ok` with tool + sha prefix; final answer rendered; no raw args leak; panel empty state before run |
| `specs/approval-flow.spec.ts` (new, Phase B) | (a) request card shows tool/args hash/TTL; Deny → activity `denied` row, temp workspace unchanged; (b) Approve → exactly one execution (file created once), card closes only after server event; (c) TTL path (short TTL fixture) → `E_APPROVAL_TIMEOUT`, no execution; (d) replay → `E_APPROVAL_REPLAY`, no second execution; (e) cancel while waiting → `E_TOOL_CANCELLED`; keyboard: initial focus on Deny, Escape does not approve, Tab trap works, `alertdialog`/`log` roles and accessible names assertable via Playwright roles | all assertions pass; requests captured show one APPROVAL_RESOLVE per decision; gate log records HEAD + command + exit code |
| gate evidence | run log must carry HEAD, command, exit code, JUnit/lists (S7 finding) | head-bound log archived |

**Commands (acceptance gate):**
```powershell
cd C:\Users\Riley\Desktop\hiverouter-v1\web
npm.cmd run typecheck
npm.cmd test -- --run
# e2e (requires the B7 fake-provider helper + a server built with the B1 contract patch)
cd ..\e2e
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'
npx playwright test --project=chrome specs/tool-loop.spec.ts specs/approval-flow.spec.ts
```
**forgewright-lead acceptance gate:** contract patch green (`cargo test -p hiverouter-contract`) + B2/B4/B7 cleared + all §11.1/§11.2 green + no skips + no paid call + gate logs HEAD-bound → then, and only then, this UI plan may be implemented as a wave. Any red gate = blocked, report by name.

---

## 12. ROLLBACK

- **UI wave:** behind `VITE_UI_TOOL_ACTIVITY` (D10). Flag off → legacy overlay/store behaviour exactly as today; new decoders are additive and harmless (events simply drive nothing visible), new files unreferenced.
- **Contract patch:** separate commit with `CHANGE-LEDGER-IMPL.md` row + regenerated golden schemas; revert restores the 21-event vocabulary and old `ApprovalRequest` struct; UI decoders must tolerate the absence (legacy path tested).
- **Server loop:** W10 §8 flags (`HIVEROUTER_TOOL_LOOP=off`) remain the backend rollback; UI shows no activity when no tool events exist (F1).
- **No DB migration in this plan** (persistence is W10 B3/CORE); no data loss path introduced by the UI.
- **Kill switch behaviour:** flag off + new server events present → decoders still parse (no crash), store ignores tool state, card renders legacy fields only; acceptable degraded mode.

---

## 13. BLOCKERS / OUT OF SCOPE / UNVERIFIED

**Hard dependencies (must land first):** W10 B1 (contract patch — this plan's event/payload design), B2 (run suspension/resume), B4 (assembly/execution seam), B7 (deterministic fake-provider harness for e2e); B3/B5 (persistence/caps) are backend prerequisites for full evidence but UI can render from wire events alone. B6 (approval UX/security) is **this plan**; B8 (Tauri) unaffected.

**Out of scope (non-claims):** no product code/tests/commands executed by this seat; no pixel layout beyond PDF-derived patterns; chat-inline tool bubbles (deferred, D7); session grants (`AllowSession` remains internal per W10 §4.3); Tauri shell; live paid provider run; backend changes.

**UNVERIFIED:** real provider tool-call ordering/`finish_reason` behaviour (W10 §9); **B1 contract patch is uncommitted in another seat's worktree at plan time** (§2a aligned, but final committed field names/types must be reconciled before UI implementation); `args_preview` absent from the in-flight patch — hash-only is the accepted UI unless a security-reviewed preview field is added; screen-reader behaviour on the target browsers (needs e2e + manual AT pass); exact TTL values beyond W10's 120 s (configurable; UI must consume `expires_at`, never hardcode); `expires_at` second precision (countdown coarse by design); whether the Lead wants the activity panel under Agents vs another surface (D7 is the plan's decision, reversible by flag/layout); model-generated JSON quality (not a UI concern).

---

## 14. RECEIPT (this seat)

Read-only seat: **only this file written**; no code, tests, commands, builds, key access, paid calls, or git writes. Decision-complete plan for TOOL_CALL/TOOL_RESULT rendering, approval card states, tool activity log, a11y, stale/refuse handling, tests (vitest + e2e), rollback; Phase 1 JSON recorded before design per `logic-gatekeeper`; five mandated skills applied (§2 recon, §3 integration, §1 gatekeeper, §8 logic-debugger, §12 lead gate).
