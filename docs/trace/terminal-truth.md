# terminal-truth.md — terminal-run truth template (NO RUNS CAPTURED)

**Status: NOT_COLLECTED.** The app does not exist yet; this file is the template the trace seat
fills from captured evidence once a run is executed. An empty template is not a pass.

> **W9 trace-truth note (2026-11-06T14:10:40-0600).** Status remains **NOT_COLLECTED**: no persisted run row + snapshot + persisted
> `/events` capture is committed. Available partial evidence is READ-level only:
> `docs/RELEASE-EVIDENCE.md:22` (release binary: SUBMIT_PROMPT -> RUN_CREATED -> nodes/reply events -> REPLY_DONE
> usage `{prompt:91, completion:8, total:99}` -> RUN_STATUS completed) and `docs/evidence/live-provider-smoke.md:8-11`
> (event sequence + completed run). The new streaming integration tests (`plans/waves/W3-STREAMING.md:143-150`)
> prove engine emission order but are not a run record. Missing evidence: SQLite `runs`/`usage`/`traces` rows,
> snapshot file, and a persisted-events capture for the same run. Do not fill this template from those notes.

Source of rules: skills `trace` (event/payload/terminal-state trace) + `evidence-check`;
blueprint Phase 11; `docs/contracts/UI-COMMANDS-v1.md` §3.

## Per terminal run (one block per run; copy the block)

```
run_id: <uuid-v7 | UNVERIFIED>
surface: <S03/S27/... | UNVERIFIED>
terminal_state: <completed | failed | cancelled | killed | INCOMPLETE | UNVERIFIED>
captured_at: <RFC3339 | UNVERIFIED>
capture_source: <live DOM + WS frames + /health + SQLite row | UNVERIFIED>

| Check | Expected | Observed | Evidence | Verdict |
|---|---|---|---|---|
| run row status/error | status = terminal_state; error_code NULL on success, typed code on failure | <observed> | <sqlite file:row or API pointer> | PENDING |
| terminal event payload | RUN_STATUS/REPLY_DONE/ERROR payload matches run row | <observed> | ws-frame timestamp + JSON | PENDING |
| snapshot status/steps/cost/tokens | snapshot agrees with run row; cost/tokens NULL when incomplete (V1-V6) | <observed> | snapshot file:line | PENDING |
| usage vs runs.cost_usd | Decimal equality for completed runs; NULL + INCOMPLETE for failed/budget | <observed> | usage JSON + row | PENDING |
| live vs persisted event source | persisted /events equals live SSE/WS stream (same envelopes) | <observed> | two captures | PENDING |
| header/actions | terminal header + actions match terminal_state (no fake running) | <observed> | DOM snapshot | PENDING |
| re-entry (reload) | reload shows same terminal truth; no resurrection of run | <observed> | reload capture | PENDING |
| refusal accounting | every refused command has code + how_to_fix + audit row | <observed> | audit log line | PENDING |
| no key/token leakage | audit/log/error surfaces contain no API key or session token | <observed> | grep of captured text | PENDING |

Timestamped screenshot: <path or UNVERIFIED — never a mid-run capture mislabelled as terminal>
```

## Rule reminders

1. A **mid-run** capture is not a convergence defect; only a timestamped terminal capture counts.
2. Failed/budget-exceeded runs must show `cost_usd`/tokens as NULL or INCOMPLETE — **never 0**.
3. A stale/missing persisted `/events` read after SSE is disabled is a defect, not "probably fine".
4. Every row above stays `PENDING`/`UNVERIFIED` until the evidence pointer is filled and checked.

> **W4 trace-capture correction (2026-11-06T2026-11-06T21:57:09.616Z).** The raw browser capture `docs/trace/live/trace-capture.json` was overwritten with `surfaces: []` at `2026-11-06T20:25:04.044Z`. Status remains **NOT_COLLECTED**; any WIRED trace verdict that depended on that capture was downgraded (see `docs/trace/controls.json` `_meta.wave2_evidence.w4_correction`). Required terminal artifacts are unchanged: persisted run row (`runs`/`usage`/`traces`), snapshot file, and persisted-events read for the same run — plus a timestamped terminal DOM capture. See `docs/trace/CAPTURE-REGISTRY.md`.
