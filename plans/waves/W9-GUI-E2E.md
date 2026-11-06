# W9-GUI-E2E — G-UI / e2e truth plan (Seat W9A)

Status: draft for execution · Repo `C:\Users\Riley\Desktop\hiverouter-v1` · base HEAD `8a5b583` (clean at start)
Write scope: `e2e/**` + this plan file. No backend/web/src/docs/tools edits, no key, no paid calls, no commit.

## 1. Target behaviour (one sentence)

The e2e suite must stop shipping 14 permanently-skipped G-UI/WS gates and instead **execute every static-dist-verifiable G-UI gate deterministically**, converting only genuinely server-bound gates into named, owned, acceptance-tracked product exclusions — with a guard that fails the run if it is green only because everything app-bound skipped.

## 2. Entry points (read)

- `e2e/specs/a11y-gates.spec.ts` — 2 static contrast tests run; 12 `test.fixme` DOM gates (lines 26-51).
- `e2e/specs/ws-contract.spec.ts` — 1 real health/session test; 2 `test.fixme` stubs (lines 18-28, 30-43).
- `e2e/helpers/matrix.ts` — `appMissing()` gate (43-45) and stale `skipReason()` copy (47-52).
- `e2e/playwright.config.ts` — installed-Chrome channel, workers=1, no `webServer`, baseURL default :8080 (17), list+html reporters.
- `e2e/specs/replay-state.spec.ts` + `e2e/helpers/replay-harness.ts` — existing deterministic fixture: loopback static server for `web/dist`, `/health` + `/api/session`, Playwright `routeWebSocket` mock with automatic ACK. No key/server. This is the fixture to reuse.

## 3. Callers / dependencies

- Every spec imports `@playwright/test`; matrix-driven specs import `loadMatrix`, `appMissing`, `skipReason`.
- `live-app.spec.ts` / `trace-hops.spec.ts` require a live server (`E2E_APP_READY=1`, `E2E_BASE_URL`); they are out of W9A edit intent except the shared helpers/config they depend on.
- Reporter chain is declared only in `playwright.config.ts`; adding a custom reporter there affects every run, so the guard must be opt-out-only (`E2E_ALLOW_ALL_SKIPS=1`).

## 4. Gate enumeration and static-dist feasibility (no key/server)

| Gate | Requirement | Can run against static `web/dist` + mocked WS? | W9A action |
|---|---|---|---|
| G-UI-1 | contrast table on live token pairs | YES — computed styles exist; mock WS gives connected UI; static math already proven | Implement computed-style + contrast assertions (body/panel/meta/focus) |
| G-UI-2 | Ctrl+0 recentre; text-field focus not hijacked | YES — React Flow + keyboard events work without backend | Implement: focus composer, type, Ctrl+0; assert focus/value retained and viewport transform changes |
| G-UI-3 | foreign Origin → `E_ORIGIN_REJECTED`; token in first frame | **NO** — Origin policy is enforced by the Rust WS server; a page-level `routeWebSocket` mock cannot prove server-based refusal | Named product exclusion (owner b-core/Lead, acceptance: live-server test) |
| G-UI-4 | ≥13-state enum, enum↔token↔label total/injective | YES — vocabulary is source/bundle state; deterministic module import | Implement source-import assertions (AGENT_STATES ≥13 unique incl interrupted; every state has label + visual; transitions cover all keys; 4 dot states 1:1) |
| G-UI-5 | missing-state coverage (loading/empty/error/refused/streaming/replay/offline/missing-key/thinking) | PARTIAL — empty/offline (no mock), refused/missing-key (inject ERROR), replay (HISTORY_SNAPSHOT), streaming/loading/thinking need run lifecycle modelling | Implement empty+offline+refused+missing-key+replay; record loading/thinking/reconnecting residual as named exclusion with owner/acceptance |
| G-UI-6 | reduced motion: 0 animations, state still updates | YES — `emulateMedia` + dataset + `getAnimations()`; state update via tab click | Implement |
| G-UI-7 | zoom clamps exactly 0.1/3.0, both reachable | YES — controls are client-only; reduced-motion makes zoom instant | Implement 25×zoom-out→0.1, 30×zoom-in→3.0 (transform matrix scale) |
| G-UI-8 | focus ring ≥3:1; axe serious = 0 | PARTIAL — focus ring is computed (`:focus-visible outline 2px var(--focus-ring)`); axe-core is not an e2e dependency | Implement focus-ring contrast over tabbed controls; named exclusion for axe-serious subcheck (owner Lead/tooling, acceptance: pinned axe dep then enable) |
| G-UI-9 | refusal toast code+how_to_fix+aria-live, pre-mutation | YES — empty-prompt refusal is client-side; mock route records sent commands → prove zero SUBMIT_PROMPT | Implement + assert no command emitted |
| G-UI-10 | kill socket → hello/resnapshot/Reconnecting; stale `E_RESYNC_REQUIRED` | **NO** — needs a real server-side resync contract; route mock cannot prove server snapshot semantics | Named product exclusion (owner b-core/Lead, acceptance: live kill-socket test) |
| G-UI-11 | no hex outside token module; every token has PDF page:line | YES — static source scan (tokens.css/tokens.ts) | Implement: CSS↔TS hex equality, every swatch pdfLine>0, zero hex outside token files in `web/src` |
| G-UI-12 | P95 frame time measured or UNVERIFIED | YES — rAF sampler in browser | Implement: 120-frame sampler, P95 < 50 ms, <30 frames = fail (no unmeasured pass) |

WS stubs:
| Stub | Requirement | Static? | W9A action |
|---|---|---|---|
| W1 | foreign Origin refused with `E_ORIGIN_REJECTED` | NO (same as G-UI-3) | Named product exclusion (owner backend/Lead; acceptance: live-server integration test) |
| W2 | hello{last_seq} + ACK-before-mutation + refusal code/how_to_fix | PARTIAL — hello/connect is exercised by replay harness; real ACK-before-mutation is covered by live-app when server available | Move to exclusion registry referencing existing coverage; do not keep a duplicate fixme test |

## 5. Data flow (happy path / failure path)

- Happy: Playwright run → `startDistHarness()` (loopback static server over `web/dist`, fake `/health` + `/api/session`) → `installMockWs()` intercepts `/ws` and ACKs COMMAND frames → page renders connected UI → G-UI assertions on DOM/computed styles/state/keyboard/rAF → `harness.close()` in afterAll.
- Failure: if `web/dist/index.html` is missing, `startDistHarness()` throws a named error → suite fails (never skips green). If no app-bound test executes, custom reporter exits non-zero unless `E2E_ALLOW_ALL_SKIPS=1`.
- Exclusion path: un-implementable gate → `e2e/GATE-EXCLUSIONS.md` row with id, owner, reason, acceptance, tracking; a registry test asserts every G-UI gate is either implemented in `gui-gates.spec.ts` or present in the exclusion file with owner+acceptance.

## 6. Tests / commands

```powershell
cd C:\Users\Riley\Desktop\hiverouter-v1\e2e
npx playwright test --project=chrome --reporter=list          # expected: static math + DOM gates + control static + replay pass; live-only specs skip
npx playwright test specs/gui-gates.spec.ts --project=chrome  # focused gate run
npx playwright test specs/ws-contract.spec.ts --project=chrome # expected exit 1 via skip-guard (all skipped, no passes)
```
Acceptance: full run exits 0 with ≥10 executed G-UI gates and 0 fixme gate rows; forced all-skip run exits 1 with the guard message; no paid calls; only installed Chrome channel.

## 7. Unknowns / risks

- Importing `web/src/state/agentState.ts` from a Playwright spec relies on Playwright's TS transpile of out-of-e2e TS (expected to work; fallback = parse source text with fs if it fails).
- Focus-ring computation across arbitrary controls: some controls may only have UA outlines; gate asserts `:focus-visible` outline colour/width/contrast where present and counts missing indicators as failures only for the tested set (documented).
- Static dist is a **built artifact**: gates prove the current `web/dist`, not source parity; build reproducibility remains outside e2e.
- Worker count 1: DOM gates must keep total runtime acceptable (<5 min).
- Skip-guard reporter: Playwright must honor `process.exitCode` set in `onEnd`; verify empirically with the all-skip run and fall back to `globalTeardown` throw if needed.

## 8. Files to touch (all in e2e/**)

- Create `e2e/specs/gui-gates.spec.ts` (12-gate implementation + registry test).
- Create `e2e/reporters/skip-guard-reporter.mjs`.
- Create `e2e/GATE-EXCLUSIONS.md` (named exclusions G-UI-3/5-residual/8-axe/10 + W1/W2 with owner+acceptance).
- Modify `e2e/helpers/replay-harness.ts` — expose COMMAND capture (`commands[]`) for pre-mutation assertions; keep ACK behavior.
- Modify `e2e/specs/a11y-gates.spec.ts` — keep static contrast math; delete the 12-fixme loop.
- Modify `e2e/specs/ws-contract.spec.ts` — keep real health/session test; delete the 2 fixme stubs (now exclusions).
- Modify `e2e/playwright.config.ts` — append skip-guard reporter.
- Update `e2e/README.md` — document deterministic GUI gate run, exclusions, and guard.

## 9. Rollback

Restore `e2e/` files from HEAD `8a5b583` content (`git show 8a5b583:e2e/<path>`) — no product code touched, no commit created.
