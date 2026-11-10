# IMPROVE-TRUTHSEEKER — request_id lane, audit attribution, governor/checkpoint, TokenLab points

Scope: read-only audit of HEAD `8da4c06` (`feat: idempotent detached admission and final release artifacts`).
Grades: **VERIFIED** = reproduced/observed via tool in this session; **READ** = read from named file:line;
**INFERRED** = follows from read code + unexercised branch; **UNVERIFIED** = not proven, and what would prove it.

No source was edited and no paid provider was called. TokenLab points are an external unit with **no
conversion rate in this audit**; no USD figure is invented anywhere below.

## 1. Claim ledger (10 rows)

| ID | Claim | Grade | Evidence |
|---|---|---|---|
| C1 | Idempotent replay resolves on `(device_id, idempotency_key)` only. A key reused with different `capability_id`, `action`, or payload silently returns the **earlier** action and never re-actuates. | READ | `remote.rs:1161-1179` (`find_action` WHERE device_id+idempotency_key); `remote.rs:981-983` returns `Ok(existing)` before params are parsed; `db.rs:745` UNIQUE(remote_id,device_id,idempotency_key) |
| C2 | `params_sha256` is written on create and on the audit row, but is **never compared** on replay or anywhere else (write-only field). | VERIFIED | `remote.rs:986`, `remote.rs:1102-1118`, `remote.rs:1145`; repo-wide grep for `params_sha256` returns insert/select-write sites only, no equality test |
| C3 | Refusal branches inside `request_action` (unknown/offline/revoked remote, revoked device, missing grant, in-flight) commit **no** `remote_audit` row; only the success path calls `audit(...)`. | READ | `remote.rs:1012-1069` return `Ok(Err(..))` before `remote.rs:1136 audit(...)`; schema `db.rs:750-767` |
| C4 | `request_id` is not a caller-controllable identity: the action id is generated server-side and inserted into both `action_id` and `request_id`; dedupe rests entirely on `idempotency_key`. | READ | `remote.rs:991`, `remote.rs:1102` (VALUES ?9 = `action_id`), `remote.rs:1117` |
| C5 | Two concurrent first-time creates with the same key are serialized only by an in-process `Mutex`; a second process/instance loses the SQLite UNIQUE insert and surfaces it as `InternalError`, not as a replay/conflict. | INFERRED | `remote.rs:977-983` `action_lock` (process-local); `db.rs:745`; no `ON CONFLICT`/retry-query-on-conflict path in `remote.rs` |
| C6 | A device-wide in-flight guard (`status IN requested/awaiting_approval/executing`) blocks every new action on that device, regardless of capability/action, for as long as any row stays in those states. | READ (blocking) / UNVERIFIED (duration) | `remote.rs:1058-1069`. No sweeper/expiry for `executing` was proven here; proving it needs a test that leaves `executing` and asserts the action later proceeds or is refused by name |
| C7 | The request-rate governor is per-process, in-memory, and monotonic-clock based; it resets to an empty window on restart, so a hard-kill/restart burst is not rate-limited by prior grants. Cross-instance limits do not exist locally. | READ (mechanism) / INFERRED (burst) | `budget.rs:536-541` (`VecDeque<Duration>` + `Instant` epoch), `budget.rs:584-608` (window prune/grant). Proving the burst needs a restart test |
| C8 | `add_run_usage` checkpoints **after** mutating memory and swallows a checkpoint failure with `tracing::error!`, leaving the run eligible to settle as if complete. | READ | `execution.rs:1001-1025` (memory `checked_add`, then `checkpoint_run_usage`, error logged, no status change) |
| C9 | `persist_run_usage` removes the run's totals from memory **before** the durable write; a failed write therefore drops the totals permanently with no retry queue. | READ | `execution.rs:1027-1039` (`remove(&run_id)` then `record_run_usage`) |
| C10 | TokenLab/external endpoints are accounted as measured tokens with **no USD settlement**: `CostMode::ExternalUnpriced` → `CostStatus::Unpriced`, `Provenance::Unpriced` ("always zero"), and `settled_usd` is null by design. No points→USD rate is applied. | VERIFIED (code path) | `accounting.rs:38-42`, `accounting.rs:61-69`, `accounting.rs:88-111`, `accounting.rs:182`; grep `points` shows no conversion site. That a points→USD rate exists in an external source is **UNVERIFIED** — it would need a cited vendor price sheet |

Unverified stores: (a) hard-kill durability of a checkpoint written between provider reply and crash;
(b) remote device/grant/action partial creation atomicity (`create_grant` at `remote.rs:592` was not
inspected); (c) `executing`-state sweep; (d) any external TokenLab points tariff.

## 2. Five improvement proposals (before → after)

### P1 — Payload-bound idempotency (closes key-reuse aliasing, C1/C2/C5)
```text
BEFORE: find_action(device, key) -> return existing action, whatever its params/capability
AFTER:  create_action(...):
          fingerprint = sha256(capability|action|canonical(params)|canonical(expected))
          BEGIN IMMEDIATE
            row = SELECT ... WHERE device_id=? AND idempotency_key=?
            if row and row.params_sha256 != fingerprint: REFUSE E_IDEMPOTENCY_CONFLICT (named; no mutation)
            if row and row.params_sha256 == fingerprint: audit(REPLAYED); return row
            INSERT action (fingerprint, request_id) ; INSERT audit(same tx) ; COMMIT
          on UNIQUE violation: re-SELECT and re-run the same fingerprint comparison (never InternalError)
        index: UNIQUE(remote_id, device_id, idempotency_key, params_sha256)
```
Test: same key + different params → named conflict and zero new rows; same key + same params → one row, two audit rows (`confirmed`, `replayed`).

### P2 — Refusals and replays are first-class audit rows (closes attribution gap, C3/C4)
```text
BEFORE: audit() only on the success INSERT; refusal paths return Err with no row
AFTER:  decide() returns Decision{outcome, refusal_code}
        every return path (pre-flight AND in-tx) opens its own tx:
          INSERT remote_audit(actor_kind, actor_id, remote_id, device_id, action,
                              request_id, idempotency_key, params_sha256,
                              outcome, refusal_code, at)
        success path keeps audit in the same tx as the action+device UPDATE
```
Invariant: `count(device_actions) == count(remote_audit WHERE outcome='confirmed')` and every 4xx remote refusal is attributable to an actor. Note `actor_kind` CHECK currently allows `operator/system/remote` while `requested_by_kind` allows `operator/task/remote` — reconcile (`task` → `system`) or the first task-originated refusal fails the insert.

### P3 — Crash-durable, boot-aware rate governor (closes restart burst, C7)
```text
BEFORE: in-memory VecDeque<Instant>; empty after restart; permit consumed at acquisition
AFTER:  persist grants as rows (grant_id, at_unix_ms, source) in the same SQLite the app already owns;
        acquire(): DELETE expired; INSERT grant atomically; if window full -> bounded wait or LlmRateLimit
        on boot: if last persisted grant < window old, still pre-load it (no free burst);
                 expose accounting counter rate_governor_state = {durable, instance_scoped}
        multi-instance: single-writer lease row OR report UNVERIFIED cross-instance enforcement
```
Test: fill window, hard-kill, restart, next acquire reports the remaining wait instead of granting.

### P4 — Checkpoint that cannot silently under-count (closes C8/C9)
```text
BEFORE: memory += usage; store write; error only traced; terminal path removes memory first
AFTER:  settle_call(): within one tx { INSERT call_usage; UPDATE run_usage_totals }
        checkpoint_failed -> mark run accounting_dirty=1 (+ audit row), keep totals in memory, retry with backoff
        persist_run_usage(): write durable first; remove from memory only after Ok; on Err keep+dirty
        status derivation: accounting_dirty=1 OR any NULL usage -> CostStatus::INCOMPLETE
```
Test: inject store failure at checkpoint → run lands INCOMPLETE with totals retained; restart does not lose the reply's usage.

### P5 — Points are a unit, never a price (closes fabricated USD, C10)
```text
BEFORE: external endpoint -> Unpriced (good) but no explicit unit/quantity; any future report may multiply
AFTER:  usage row carries billing_unit ('tokens'|'points'), quantity, provenance='measured',
        settled_usd = NULL, cost_status='UNPRICED'
        conversion(points -> USD) is refused unless a cited, dated rate source id is supplied;
        every conversion writes remote/accounting audit with the rate source id and unit arithmetic
```
Invariant: no persisted row has `billing_unit='points' AND settled_usd IS NOT NULL` without a source id. Report renders points as `unknown in USD`, never as 0 or an estimate.

## 3. Corrections log
| Claim | Before | After | Why |
|---|---|---|---|
| C2 `params_sha256` enforced | assumed enforced (field exists in schema + audit) | READ: write-only | grep shows no comparison site |
| C10 points priced | did not assume | VERIFIED null-USD path | accounting contract explicitly `Unpriced`; no rate found |
| C6 in-flight guard | assumed capability-scoped | READ: device-scoped | SQL filters device_id only |

## 4. Unverified / not delivered
- No runtime tests executed (timebox); all code claims are file-graded, not run-graded.
- Partial remote/device/grant creation atomicity not inspected (`create_grant` unreviewed).
- No external TokenLab price sheet retrieved; points remain unvalued by design of this audit.
