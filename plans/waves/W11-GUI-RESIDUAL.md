# W11-GUI-RESIDUAL — G-UI-5 residual states + axe decision (Seat E)

**Write scope:** new `e2e/specs/gui-residual.spec.ts`, `e2e/GATE-EXCLUSIONS.md`, this plan only.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD at start:** `9651722` (dirty from other lanes; this seat touches no product/web/tools files).
**Skills used (mandated):** `/forgewright-recon` → `/forgewright-lead` → `/logic-gatekeeper` → `/integration-logic` → `/logic-debugger` (all loaded and applied below).

## 1. RECON (forgewright-recon ledger)

- **Target behaviour:** execute the four residual G-UI-5 substates (`loading`, `thinking`, `streaming`, `reconnecting`) deterministically against the current `web/dist` + mocked WebSocket transport, and decide axe explicitly (no unapproved install).
- **Entry point:** `e2e/specs/gui-gates.spec.ts` (G-UI-5 implemented partially at :146-189; registry test :335-363), `e2e/GATE-EXCLUSIONS.md` (G-UI-5 residual row line 13; G-UI-8 axe row line 12), `e2e/package.json` (only `@playwright/test 1.65.0`; guarded runner `run-guarded.mjs`), `e2e/helpers/replay-harness.ts` (`startDistHarness`, `installMockWs`, `eventFrame`; mock records COMMANDs and auto-ACKs).
- **Callers/contract:** state arrives via WS `EVENT` envelopes decoded by `web/src/state/store.ts`: `RUN_CREATED` :236-241, `RUN_STATUS` :243-257, `REPLY_DELTA` :259-276, `REPLY_DONE` :278-287, `SERVER_HELLO` :228-230, `RESYNC_REQUIRED` :232-234. UI: `ChatCore` streaming indicator, `SettingsPanel` `LoadingState`, `App` reconnect banner, `AgentsPanel`/`StateGlyph` state-label rendering (`web/src/flow/nodes.tsx:21-41`, aria-label :71).
- **Tests + command:** `node e2e/run-guarded.mjs --project=chrome specs/gui-gates.spec.ts specs/gui-residual.spec.ts` (guard exits non-zero on all-skip green).
- **Data flow (success/failure):** mocked WS route established before navigation → tests push frozen `EVENT` envelopes (`eventFrame`) → store reducer sets UI state → DOM assertions. Failure: mock sends nothing / wrong run_id → reducer ignores (`RUN_STATUS` requires matching run) → test times out honestly; no live server, no key, no paid call.
- **Unknowns / evidence still missing:** loading/thinking/streaming/reconnecting had no deterministic execution (excluded W9A). Axe (`axe-core`/`@axe-core/playwright`) is **not installed** (`MODULE_NOT_FOUND`) and adding it needs Lead approval; version selection must come from official npm at approval, not invented here.
- **Files expected to touch:** `e2e/specs/gui-residual.spec.ts` (new), `e2e/GATE-EXCLUSIONS.md` (close G-UI-5 residual row with evidence; make axe decision explicit), `plans/waves/W11-GUI-RESIDUAL.md` (this plan).

## 2. LEAD CONTRACT (forgewright-lead)

- Root cause stated before edits; no claim without a command. Evidence classes: VERIFIED (run this seat) / READ (file) / INFERRED / UNVERIFIED.
- No skip-as-green: the new spec has no `test.fixme`/`test.skip`; guarded runner fails all-skip runs.
- Axe decision: **explicit exclusion, not an install** — no unapproved dependency; exact pinned versions are the Lead's approval decision (none fabricated).

## 3. LOGIC-GATEKEEPER Phase 1 (recorded before creating/editing spec or exclusions)

```json
{
  "phase1": {
    "architecture": "e2e specs run against a loopback static server serving web/dist; replay-harness installMockWs intercepts /ws, records COMMAND frames and auto-ACKs; the React store reducer turns frozen EVENT envelopes into run/message/connection/settings UI state; GATE-EXCLUSIONS.md is the registry read by gui-gates.spec.ts's registry test.",
    "dataFlow": "test -> eventFrame(name,payload,seq) -> mocked WS route.send -> WsApiClient decode -> store applyEvent -> React render -> locator assertion. Failure path: missing envelope or mismatched run_id leaves UI unchanged; test fails by timeout, not a false pass.",
    "edgeCases": ["RESYNC_REQUIRED must set connection=reconnecting and disable composer; a following SERVER_HELLO must return to open without losing run state", "REPLY_DELTA without prior RUN_CREATED must still create a streaming assistant message keyed to run_id (store :259-276), and REPLY_DONE must clear only that run's streaming flag", "Settings loading: probeSettings sends SETTINGS_STATUS but the mock returns only ACK (no SETTINGS_STATUS_RESULT) -> settings.status stays pending -> LoadingState visible", "NODE_SPAWN with state='thinking' must render the enum label 'Thinking' (not a colour-only glyph) on the agent card"],
    "rootCause": "The residual G-UI-5 substates were never executed because the prior gate only drove static/empty/refusal/replay paths, so loading/thinking/streaming/reconnecting are unproven even though the mocked transport can deterministically emit the exact frozen events that drive them; the contradiction is that excluded-but-drivable states were treated as environment-blocked."
  }
}
```

Phase 1 accepted (all fields non-empty; edge cases concrete; root cause is a full sentence).

## 4. INTEGRATION-LOGIC — call-chain proof per substate

| Substate | Trigger envelope (mocked) | Store effect (file:line) | DOM evidence asserted |
|---|---|---|---|
| loading | open Settings (probeSETTINGS_STATUS; mock ACK only) | `settings.status='pending'` → SettingsPanel `LoadingState` `role=status` | `main` contains a `role=status` "Loading" |
| thinking | `NODE_SPAWN {task_id, state:'thinking', kind:'superagent', label}` | tasks record + `visualFor('thinking').label` | Agents panel "Thinking" text visible; canvas node aria-label contains `Thinking` |
| streaming | `RUN_CREATED` → `RUN_STATUS streaming` → `REPLY_DELTA` → `REPLY_DONE` | streaming message + `streamingRunId` cleared by REPLY_DONE | assistant text visible; `role=status` "Streaming" appears then disappears; text persists |
| reconnecting | `RESYNC_REQUIRED` then `SERVER_HELLO` | connection reconnecting→open (store :228-234) | reconnect banner visible, composer disabled; after hello banner hidden, composer enabled |

Refusal boundary: none of these actions starts a provider call; no key or live server involved.

## 5. LOGIC-DEBUGGER — invariant, RED, GREEN

- **Invariant:** each G-UI-5 substate must be reachable from a frozen envelope sequence through the real reducer and must be asserted on DOM text/state, not inferred from source; the exclusion registry may only retain rows that are genuinely blocked.
- **RED (before):** `e2e/specs/gui-residual.spec.ts` does not exist; `GATE-EXCLUSIONS.md:13` lists G-UI-5 residual as blocked although all four triggers are available in the mocked transport; axe has no dependency and no explicit decision record beyond a generic row.
- **GREEN (after):** new spec executes the four deterministic scenarios; G-UI-5 residual exclusion is closed with the evidence command; axe row states the explicit decision (excluded; pinned dependency requires Lead approval); guarded two-spec run is green with >0 passed.
- Residual: axe-serious remains unmeasured (owned exclusion); no live-server resync/origin proof.

## 6. Decisions

1. Implement `gui-residual.spec.ts` with four tests + a registry-hygiene test (asserts the four substate names are present, GATE-EXCLUSIONS no longer lists G-UI-5 residual, and no `test.fixme(`/`test.skip(` calls in the new spec).
2. Do **not** install axe. Explicit exclusion chosen: `axe-core`/`@axe-core/playwright` absent (`MODULE_NOT_FOUND`); pinned version must be approved by the Lead from official npm (no fabricated version), then a serious=0 assertion can be added. W11 records the decision, not an install.
3. G-UI-5 residual row is converted from exclusion to implemented evidence in `GATE-EXCLUSIONS.md`; other genuinely blocked rows (G-UI-3, G-UI-10, axe, W1, W2) remain.
4. No product/e2e-helper/config edits beyond the allowed files; no commit.

## 7. Files

- `e2e/specs/gui-residual.spec.ts` — new deterministic spec (dist harness + mocked WS; no key/live server).
- `e2e/GATE-EXCLUSIONS.md` — G-UI-5 residual row closed with evidence; axe decision explicit; other rows unchanged.
- `plans/waves/W11-GUI-RESIDUAL.md` — this plan, with observed validation and Done/Not done/Unverified.

## 8. Validation (observed, quoted)

```
$ cd e2e; node run-guarded.mjs --project=chrome specs/gui-gates.spec.ts specs/gui-residual.spec.ts
Running 16 tests using 1 worker
  ok 1..11  gui-gates.spec.ts (G-UI-1,2,4,5,6,7,8,9,11,12 + gate registry)
  ok 12     gui-residual: loading: pending SETTINGS_STATUS probe renders the loading state
  ok 13     gui-residual: thinking: NODE_SPAWN thinking renders the enum label on agent + canvas cards
  ok 14     gui-residual: streaming: deltas render progressively and clear on REPLY_DONE without losing text
  ok 15     gui-residual: reconnecting: RESYNC_REQUIRED blocks mutations, SERVER_HELLO restores without losing the run
  ok 16     gui-residual: registry hygiene: residual states covered, no skip-as-green rows
  16 passed (3.1m)   EXIT=0

$ node run-guarded.mjs --project=chrome specs/ws-contract.spec.ts
  1 skipped
[skip-guard] all 1 tests were skipped and none passed; set E2E_ALLOW_ALL_SKIPS=1 only with a written reason
  ALLSKIP_EXIT=1   (guard negative control proven; no skip-as-green)
```

`GATE-EXCLUSIONS.md` updated: G-UI-5 residual row removed/closed with evidence pointer to `specs/gui-residual.spec.ts`; G-UI-8 axe subcheck now records the **W11 explicit-exclusion decision** (no install; pinned version requires Lead approval); G-UI-3, G-UI-10, W1, W2 remain named exclusions. No product/web/src/tools edits; no commit.

## 9. Rollback

Delete the new spec and revert `GATE-EXCLUSIONS.md` to HEAD content (`git show 9651722:e2e/GATE-EXCLUSIONS.md`); remove this plan. No product files touched, no commit.

## 10. Done / Not done / Unverified

- **Done (verified 2026-11-06):** deterministic G-UI-5 residual coverage (loading/thinking/streaming/reconnecting) executed against `web/dist` + mocked WS; G-UI-5 residual exclusion closed with evidence; explicit axe decision recorded (no unapproved install); guarded GUI specs **16 passed / 0 failed, exit 0**; guard negative control (all-skip) **exit 1** with `[skip-guard]` message; registry hygiene test added.
- **Not done:** axe serious=0 remains unmeasured and uninstalled by design; G-UI-3 foreign-Origin and G-UI-10 live resync remain product exclusions; no live server run; no paid call.
- **Unverified:** live-server versions of reconnect/resnapshot; axe bug classes; visual regressions beyond text/ARIA/state assertions in `web/dist`.
