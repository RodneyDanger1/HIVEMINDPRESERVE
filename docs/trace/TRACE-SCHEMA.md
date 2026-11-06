# TRACE-SCHEMA — 11-hop control trace data contract (v1)

> **Dated correction 2026-11-06 (W8 docs-truth; HEAD `0dfe23b`).** Current evidence-backed trace totals are **43 WIRED-AND-WORKS / 270 PENDING / 6 MISSING of 319 hops** in `docs/trace/controls.json` (the `260 UNVERIFIED / 5 MISSING` figures below are the historical capture). Status line: live capture applied; no fabricated verdicts.

Owner: QA / trace engineer (`docs/trace/**`). Sources: skill `trace` (11-hop ladder, event/payload/terminal
trace), `blueprint/UI-WIRING-GUIDE.md` §1-§3, `docs/contracts/UI-COMMANDS-v1.md`, EXECUTION-PROMPT §7.
**Status: LIVE CAPTURE APPLIED (2026-11-06 15:03Z).** Captured evidence-backed verdicts are applied to
`controls.json` (43 WIRED-AND-WORKS hops, 260 UNVERIFIED, 5 MISSING, 0 WIRED-BUT-DEAD; 23 controls partially
verified); a verdict may only leave PENDING when its `evidence[]` is non-empty and points at a captured artifact.

## 1. Files

| File | Role | Current state |
|---|---|---|
| `controls.json` | 29 control records (28 guide surfaces + supplemental `S-APPROVAL`) × 11 hop slots (initial verdicts) | live capture applied: 23 `PARTIALLY_VERIFIED`, 6 `PENDING_APP_NOT_BUILT` (node cards S15-S19 + S-APPROVAL); 48 hop verdicts evidence-backed |
| `traces.json` | append-only captured runs (`TraceRun[]`) | empty (`0 runs`) |
| `events.json` | emitted/subscribed/delivered parity, payload parity, terminal records | empty arrays; event vocabulary 21 |
| `terminal-truth.md` | terminal-run truth template (per-run evidence block) | template only, `NOT_COLLECTED` |

Regenerate controls/events from the matrix: `node tools/qa/build-trace-seeds.mjs`.
Validate: `node tools/qa/verify-harness.mjs`.

## 2. The 11 hops (exact keys, in order)

| Hop key | Required proof |
|---|---|
| `1_render` | Control exists in DOM with right role/label; enabled/disabled is real (not CSS-faked) — `dom` evidence (snapshot/screenshot/selector + quote) |
| `2_handler` | Real handler bound (no `href="#"`/no-op) — `file` evidence `web/src/...:line` or DOM listener proof |
| `3_client_call` | Payload/validation matches the frozen command contract — `request`/`ws-frame` + `file` pointer |
| `4_request` | Exact method/path/body/headers; `correlation_id`; idempotency for mutating commands — captured request |
| `5_route` | Route exists, validates, authorises, names refusals — `file` route pointer + captured response |
| `6_service` | Real service/pipeline execution; no fake success/sleep — `log`/`db-row`/`file` pointer |
| `7_store` | Transaction/CAS/idempotency + actual write (SQLite row, snapshot, artifact) — `db-row`/`file` with query+row |
| `8_provider` | Mock/fixture provider path (NOT a live paid call): timeout/refusal mapping, no key leakage, no fake success — `log`/`request` |
| `9_response` | Status/code/body truthfully represent result (typed refusal vs success) — captured response |
| `10_render_back` | UI state changes per ACK/refusal; stale state cleared; no optimistic fake success — `dom` before/after |
| `11_reentry` | Reload/back/duplicate/cancel/terminal truth (run row + persisted events after SSE off) — `db-row`/`request`/`dom` |

## 3. controls.json

```json
{
  "_meta": {"schema": "hiverouter.trace.controls/v1", "source": "e2e/data/control-matrix.json",
            "status": "PENDING_APP_NOT_BUILT", "generated_at": "RFC3339"},
  "controls": [{
    "surface_id": "S01", "surface": "...", "kind": "client-state|consume-only|derived|mutating-command|static-asset|decision-gated|consume+client-state",
    "command": "SUBMIT_PROMPT|null", "owner": "Phase/WP", "acceptance": "...",
    "status": "PENDING_APP_NOT_BUILT",
    "hops": {"1_render": {"verdict": "PENDING", "evidence": []}, "... 11 keys ...": {}}
  }]
}
```

Rules: 29 controls exactly (28 guide surfaces S01-S28 + supplemental `S-APPROVAL` for `APPROVAL_RESOLVE`); each record
has 11 keys exactly; verdict vocabulary below; `evidence[]` entries are
`{"kind":"dom|request|ws-frame|db-row|log|file","pointer":"...","quote":"..."}`; no verdict may be
`WIRED-AND-WORKS` without evidence for **all** hops that the control exercises.

## 4. traces.json

```json
{"_meta": {"schema": "hiverouter.trace.runs/v1", "status": "NO_RUNS", "generated_at": "RFC3339"},
 "traces": [{
   "run_id": "uuid-v7", "surface_id": "S03", "base_url": "http://127.0.0.1:8080",
   "started_at": "RFC3339",
   "hops": {"1_render": {"verdict": "WIRED-AND-WORKS",
                         "evidence": [{"kind": "dom", "pointer": "playwright trace ...:line", "quote": "..."}],
                         "captured_at": "RFC3339"}, "...": {}},
   "notes": "optional"
 }]}
```

Appended only by `e2e/helpers/trace.ts` (`appendTrace`). A skipped/blocked run appends nothing.

## 5. events.json — parity + payload + terminal truth

```json
{"_meta": {"schema": "hiverouter.trace.events/v1", "status": "NOT_COLLECTED",
           "event_vocabulary_source": "docs/contracts/UI-COMMANDS-v1.md §3", "parity_rules": ["..."]},
 "emitted": [],      // server emitEvent sites (source-derived; file:line)
 "subscribed": [],   // client addEventListener registrations (source-derived; file:line)
 "delivered": [],    // browser-captured frames (ws-frame with timestamp)
 "payload_parity": [], // {event, live_payload, persisted_payload, equal: boolean, pointer}
 "terminal_records": []} // one per terminal run: see terminal-truth.md
```

Required checks before any terminal verdict:
1. **emitted == subscribed == delivered.** Report `emitted-not-subscribed` and `subscribed-never-emitted`
   separately; a named SSE/WS frame that never reaches the registered listener is a dropped event, not omission.
2. **Payload shape parity.** A live envelope (`{schema,type,data,meta}`) persisted as `data` is not the same
   value as the persisted `payload`; unwrap only when the schema marker is present.
3. **Terminal record truth.** Run row status/error, terminal event payload, snapshot status/steps/cost/tokens,
   live-vs-persisted event source, header/actions and persisted `/events` read after SSE disabled must all agree.
   A screenshot must be timestamped; a mid-run capture is not a convergence defect.
4. **Failed/budget terminal usage.** `runs.cost_usd`/`prompt_tokens`/`completion_tokens` must be NULL/INCOMPLETE,
   never 0; compare with `usage` and `traces` rows.

## 6. Verdict vocabulary (exactly these five)

| Verdict | Meaning |
|---|---|
| `WIRED-AND-WORKS` | every hop proven with captured evidence (live DOM/request/DB/log) |
| `WIRED-BUT-DEAD` | code path exists but fakes/echoes/drops; evidence shows the break |
| `MISSING` | no phase/implementation owns the hop |
| `REFUSED-BY-NAME` | control refuses with a typed code + how_to_fix; no silent no-op |
| `UNVERIFIED` | not provable here; what would prove it named |

`PENDING` is a seed-only marker (not a verdict) and may not survive a real run.

## 7. Evidence-label rules

- `VERIFIED` = observed by tool/file this session; quote path:line or captured JSON/frame.
- `READ` = read but not independently confirmed; name the file.
- `INFERRED` = reasoned; state reasoning.
- `UNVERIFIED` = not provable; name what would prove it.
- `NOT DOCUMENTED` = official sources/silent.
A run without evidence quotes is `UNVERIFIED`, not PASS. Never paste a screenshot without timestamp.
