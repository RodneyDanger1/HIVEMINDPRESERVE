# SEAT S4 — Live UX / E2E report (read-only)

- Repo: `C:\Users\Riley\Desktop\hiverouter-v1` · paper baseline HEAD `3988622`; current HEAD `5c69b66e363e9f1d0801ea5c2e66a1fc12c8943c` (dirty: backend Cargo files/package.json, new docs/RELEASE.md).
- Mandatory order completed: paper (full) → 4 official DeepSeek docs → 2 assigned audit files → product slice, one at a time (READLOG has bytes/sha256/lines/facts per file). Missing assignment directory: `web/src/hooks/**` (does not exist).
- The live server was **not running** at audit time (`curl /health` = 000), so the journey below is evidence-assessed from source + tracked artifacts, not a fresh live run. No key read, no paid call, no product/git writes.

## 1. Verified facts (fresh, read-only)

| # | Fact | Evidence |
|---|---|---|
| V1 | Paper baseline records P0 A-F01 provider unwired, A-F02 dead replay, A-F03 16-vs-15 parity; 79 e2e pass/14 skips; 82 runtime artifacts; skip-as-green/wiped-log/trace-gap P2s | `audit/HIVEMIND-AUDIT-PAPER.md:104-117` (read in full) |
| V2 | Current tree **has provider executor wiring** with honest `UnconfiguredExecutor` fallback when key/provider absent | `backend/crates/hiverouter-core/src/http.rs:21,25,67-73` (READ) |
| V3 | Current tree **builds replay nodes from HISTORY_SNAPSHOT** instead of a hardcoded empty list | `web/src/flow/Canvas.tsx:114-118,135` (READ) |
| V4 | Current tree has **16 states incl `interrupted`** in the UI vocabulary | `web/src/state/agentState.ts:9-28` (READ) |
| V5 | Live-provider smoke (tracked) proves a real non-mock paid call: completed, reply HIVE-LIVE-OK, usage 89/6/95, no error | `docs/evidence/live-provider-smoke.md:7-12` (read in full) |
| V6 | Same docs' remaining gaps: adapter returns the leaf reply as one delta after a non-streaming completion; true SSE delta publishing + replay UI still being completed | `docs/evidence/live-provider-smoke.md:12-13` |
| V7 | Final e2e artifact log: 93 tests = 79 passed / 0 failed / 14 skipped; run 09:21:29; HTML report exists | `e2e/test-results/run-live-FINAL-artifact.log`; `e2e/playwright-report/index.html` |
| V8 | 14 skips are explicit `test.fixme`: 12 G-UI DOM gates + 2 WS-contract stubs | `a11y-gates.spec.ts:26-51`; `ws-contract.spec.ts:18-43` |
| V9 | Trace capture 28 surfaces (43 WW/260 UNV/5 MISSING), controls 23 partial/6 pending, no S-APPROVAL capture (matrix 29 controls) | `docs/trace/live/trace-capture.json`; `docs/trace/controls.json`; `trace-hops.spec.ts:28-57` |
| V10 | Harness default baseURL :8080 vs live evidence :8799; no artifact identity preflight | `playwright.config.ts:17`; `live-app.spec.ts:6`; `trace-hops.spec.ts:14` |
| V11 | Skip-as-green gate unless `E2E_APP_READY=1`; skip copy still says milestone-1 shell | `e2e/helpers/matrix.ts:43-52` |
| V12 | Seed/apply ordering hazard: seeds rewrite controls all-PENDING; apply is what adds evidence; running seeds after apply silently drops verdicts | `tools/qa/build-trace-seeds.mjs:81-86`; `tools/qa/apply-live-trace.mjs:24-58` |
| V13 | Validator uses lower-bound vocabulary thresholds and exact-heading contract parsing | `tools/qa/verify-harness.mjs:30-36,56-59` |
| V14 | Panels are honest/no-fake-success: approval disabled without nonce; diff modal decision-gated close-only; models/agents/routing derive from store | `ApprovalOverlay.tsx:30-55`; `DiffDecisionModal.tsx:5-32`; `ModelsPanel.tsx:11-21`; `AgentsPanel.tsx:8-44`; `RoutingPanel.tsx:8-35` |
| V15 | App root renders skip link, nav, header, alerts, workspace-only ControlBar/Scrubber, chat/overlays; mock banner only when apiKind==='mock' | `web/src/App.tsx:76-102,106-121` |

## 2. Live journey assessment (submit → plan → reply → replay)

| Hop | Exists today | Evidence | Automation gap |
|---|---|---|---|
| Submit | Composer→store.submitPrompt→SUBMIT_PROMPT; empty prompt → client E_PROMPT_EMPTY with zero WS frames; CHAT_PIN ACK-before-render-back proven | `live-app.spec.ts:157-187`; `ChatCore.tsx`; smoke doc RUN_CREATED→ACK | Valid SUBMIT_PROMPT is never e2e-exercised (would be paid); no mock-provider e2e path → gated live smoke or wiremock-served run required. |
| Plan | Backend emits NODE_SPAWN×2 / NODE_STATE; graph authority + agent cards exist | smoke doc line 8; `AgentsPanel.tsx`; `Canvas.tsx` | No e2e asserts plan event order/content or node cards rendering from a run; S15-S19 trace records are MISSING by design. |
| Reply | REPLY_DELTA→REPLY_DONE render in ChatCore; usage fields exist | smoke doc lines 8-11; `ChatCore.tsx` | Smoke doc states one leaf delta after non-streaming completion; delta ordering/streaming, terminal truth (RUN_STATUS/usage/cost vs run row) not automated. |
| Replay | Scrubber commits HISTORY_REVISION_SET; Canvas builds replay tasks from HISTORY_SNAPSHOT; ReplayBanner exists | `Scrubber.tsx:11-49`; `Canvas.tsx:114-118,135`; `StatePack.tsx:34-40` | **No e2e test** for scrubber→snapshot→read-only replay render (A-F02 fix is READ, not browser-proven). |

## 3. Defects / debt in the assigned slice (path:line, severity)

1. **P1 — skip-as-green + stale gate copy** — `e2e/helpers/matrix.ts:43-52`: all browser tests skip unless `E2E_APP_READY=1`; a bare run reports success with 0 executed tests (observed previously in `.last-run.json`), and `skipReason()` still says "web/src is a milestone-1 shell". Fix: fail closed when CI/execution is requested; refresh copy.
2. **P1 — 14 fixme gates** — `a11y-gates.spec.ts:26-51` (12 G-UI DOM gates) + `ws-contract.spec.ts:18-43` (foreign-Origin refusal, hello/ACK duplicate) are explicitly not executed; they are honest (not passes) but G-UI-1..12 and origin refusal remain unverified by the harness.
3. **P1 — no automated paid-safe live journey** — valid submit/plan/reply exist only in the single manual smoke doc; no mock-provider e2e, no terminal-truth automation, no replay e2e. Largest Live-UX evidence gap.
4. **P2 — baseURL drift / no artifact preflight** — `playwright.config.ts:17` default :8080 vs specs fallback :8799 (`live-app.spec.ts:6`, `trace-hops.spec.ts:14`); no binary-hash/version check that the tested process is the shipped artifact.
5. **P2 — trace completeness / S-APPROVAL** — `trace-hops.spec.ts:28-57` captures 28 but matrix has 29 (S-APPROVAL missing); `afterAll` 76-87 has no count/uniqueness/schema assertion; Settings branch 115-142 leaves `10_render_back` UNVERIFIED despite rendered values.
6. **P2 — seed/apply ordering hazard** — `build-trace-seeds.mjs:81-86` regenerates all-PENDING controls; `apply-live-trace.mjs:24-58` applies evidence; `npm run seeds` after a capture silently erases verdicts (no guard/order in README).
7. **P2 — validator thresholds/heading parsing** — `verify-harness.mjs:30-36,56-59`: heading substring parse and `>=10/>=12/>=30` thresholds allow vocabulary shrink/rename to pass or blank.
8. **P2 — evidence written into product tree** — `e2e/helpers/evidence.ts:5-26` + spec writes put JSON/screenshots in `docs/trace/live/live-app/`; suite runs dirty the repo (observed untracked 14/15 JSON at closeout).
9. **P2 — brittle literals** — `live-app.spec.ts:282-283,307-312` hardcode fixture names; 118-123 hardcode panel copy; fixture/copy edits break tests unrelated to wiring.
10. **P3 — missing assigned directory** — `web/src/hooks/**` does not exist; only hook-like file is `web/src/state/useAppReducedMotion.ts` (outside slice).
11. **P3 — stale docs** — `tools/qa/README.md:1-3,12-13` claims scripts run before the app exists and that seed writing is the only docs/trace writer (apply-live-trace also writes); `build-trace-seeds.mjs:4-5` says 28 controls vs 29.

## 4. File-by-file plan (analysis → next action)

| File(s) | State | Next action |
|---|---|---|
| `live-app.spec.ts` (15 tests) | passes shipped binary; no valid paid submit, no replay | keep; derive fixture literals from payloads; add mock-provider run-path + replay-scrubber e2e. |
| `trace-hops.spec.ts` (28 captures) | real defaults-closed capture | add S-APPROVAL, assert 28/29 completeness in afterAll, set S21 render-back, per-record timestamp/seq. |
| `a11y-gates.spec.ts` | static math only; 12 fixme | implement DOM gates incrementally (axe/keyboard/zoom/reduced motion); keep fixme until executed. |
| `ws-contract.spec.ts` | health real; 2 fixme | implement synthetic foreign-Origin page; move hello/ACK scenario here. |
| `control-matrix.spec.ts` | static + 29 boot probes | add gated live-run probe behind env; keep static. |
| `playwright.config.ts` | deterministic, no webServer | remove :8080 default or fail loudly; artifact-identity preflight; durable run logs outside outputDir. |
| `helpers/{matrix,evidence,app,ws-tap,contrast,trace}.ts` | small, honest | fail-closed env gate/refresh skip copy; configurable evidence dir; trace.ts appendTrace unused — wire or delete. |
| `web/src/App.tsx` | honest shell | no product change from this seat; replay proof belongs to browser tests (Canvas out of slice). |
| `web/src/panels/**` (16) | store-derived, fail-closed approval/diff, em-dash null sizes | add tests for approval nonce-less disable, diff decision-gated, scrubber disabled/live; no product change. |
| `tools/qa/**` (6) | read-only except seeds/controls writers | seed/apply ordering guard + README fix; exact contract-set comparison; stale copy/28→29; capture-integrity check. |
| `docs/evidence/live-provider-smoke.md` | tracked manual evidence with explicit gaps | regenerate after true streaming/replay; gated automated smoke writes machine receipt. |

## 5. Tests / commands (proposed; not executed by this seat)

- Static: `node tools/qa/verify-harness.mjs` (expected 27/27 on current tree), `node tools/qa/figcraft-assets.mjs`.
- Browser (no key): `cd e2e; $env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'; npx playwright test --project=chrome --reporter=list,html` (expected 79/0/14 today).
- **Gated paid live smoke (safety design, NOT run):**
  ```powershell
  # operator supplies the key in-process only; never printed/logged; explicit --yes; hard budget + fixed prompt.
  $env:DEEPSEEK_API_KEY = '<operator-provided>'   # not read/logged by this seat
  $env:HIVEROUTER_LIVE_SMOKE = '1'
  node tools/qa/live-journey-smoke.mjs --yes --budget-usd 0.02 --max-tokens 256 --prompt 'Reply with exactly HIVE-LIVE-OK'
  ```
  Assertions: /health ok; WS hello token; one SUBMIT_PROMPT → ACK + RUN_CREATED → NODE_SPAWN ≥1 → ordered REPLY_DELTA(s) → REPLY_DONE + RUN_STATUS completed; usage non-null and consistent with run row/trace; scrubber HISTORY_REVISION_SET → HISTORY_SNAPSHOT replays read-only; no key/token in logs; abort on budget/timeout; isolated DB per run. Missing key must yield E_KEY_MISSING (no fake success), never skip-green.
- Mocked run path: add wiremock/fixture provider in backend dev path + e2e asserting submit→plan→reply→replay without paid calls (only way to make it reproducible in CI).

## 6. UNVERIFIED

- Live journey was not re-executed here (no server at audit time; read-only, no key). Completed-run claim is READ from the tracked smoke doc, not reproduced.
- True streaming deltas: smoke doc says one leaf delta after non-streaming completion; whether current tree streams is UNVERIFIED (would prove with gated smoke capturing ordered REPLY_DELTA frames).
- Replay UI: Canvas wiring READ only; no browser proof of scrubber→snapshot→render.
- 12 G-UI DOM gates + foreign-Origin refusal are fixme/UNVERIFIED.
- DeepSeek pricing figures: fetched pricing page body lacked the table → UNVERIFIED.
- `web/src/hooks/**` missing; the assignment item is MISSING, not analysed.
- Current HEAD dirty (Cargo/package changes + untracked docs/RELEASE.md): audited tree is not the tagged artifact; hash-level verification pending Lead integration.
