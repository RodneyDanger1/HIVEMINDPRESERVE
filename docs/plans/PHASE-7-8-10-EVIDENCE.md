# PHASE 7 / 8 / 10 — FRONTEND EVIDENCE

Date: 2026-11-06 · Owner: FRONTEND seat · Scope: `web/**` + this evidence doc + claim register update.
Blueprint: Phase 7/8/10 + §5 (sha `501BA198…C5B1`), UI-WIRING-GUIDE (S01–S28, R-U1..R-U15, G-UI),
`docs/contracts/UI-COMMANDS-v1.md` (patched 2026-11-06), UI watchdog `ledgers/ui-watchdog.json`
(PASS-WITH-NOTES, allow_frontend=true, FAIL 0).

## 1. Commands run and observed output (quoted)

```text
> cd web
> npm.cmd install --no-fund --no-audit
added 86 packages in 2m                                         (exit 0; disk after: 7.43 GB -> later 16.59 GB free)

> npm.cmd run typecheck
> hiverouter-web@0.1.0 typecheck
> tsc --noEmit                                                  (exit 0; 0 errors)

> npm.cmd run test
 ✓ tests/apiClient.test.ts (5 tests)
 ✓ tests/noHardcoded.test.ts (4 tests)
 ✓ tests/store.test.ts (7 tests)
 ✓ tests/tokens.test.ts (6 tests)
 ✓ tests/agentState.test.ts (5 tests)
 ✓ tests/keyboard.test.ts (6 tests)
 ✓ tests/metrics.test.ts (3 tests)
 Test Files  8 passed (8) · Tests  44 passed (44)              (exit 0)

> npm.cmd run build
> tsc --noEmit && vite build
vite v8.3.6 building client environment for production...
✓ 500 modules transformed.
dist/index.html                   0.52 kB │ gzip:   0.32 kB
dist/assets/index-BWOl__uM.css   35.75 kB │ gzip:   7.38 kB
dist/assets/index-_C6E8rly.js   669.20 kB │ gzip: 213.84 kB
✓ built in 1.15s                                               (exit 0)

> npm.cmd ls --depth=0   (exact installed pins)
@dagrejs/dagre@3.1.3 · @tailwindcss/vite@4.3.4 · @types/react@19.3.0 · @types/react-dom@19.3.0 ·
@vitejs/plugin-react@6.2.1 · @xyflow/react@12.12.1 · motion@14.9.1 · partysocket@1.3.0 ·
react@19.3.1 · react-dom@19.3.1 · tailwindcss@4.3.4 · typescript@7.0.2 · vite@8.3.6 · vitest@5.1.0 · zustand@5.0.16
Node v26.7.0 · npm 11.19.0 (blueprint floor Node >=22.12.0 satisfied)
```

Build note: one 500 kB chunk-size warning (React Flow + motion + app, 669 kB / 213 kB gzip). Not a failure;
code-splitting is a Phase-12/performance follow-up (recorded below), not hidden.

## 2. 28-surface implementation matrix (S01–S28)

| Surface | Component / file | State |
|---|---|---|
| S01 canvas + honeycomb | `flow/Canvas.tsx`, `styles/index.css` (`.hive-canvas-field`) | implemented |
| S02 pinned chat core | `chat/ChatCore.tsx` | implemented (fixed overlay; pan does not move it) |
| S03 composer | `chat/ChatCore.tsx` (form, textarea, submit) | implemented (Enter/Shift+Enter, disabled preconditions) |
| S04 Spawn sub-hive | `panels/ControlBar.tsx` | implemented (`SPAWN_SUB_HIVE`) |
| S05 Re-route | `panels/ControlBar.tsx` + target select | implemented (`REWIRE_TASK`, expected_rev) |
| S06 Cancel | `panels/ControlBar.tsx` | implemented (`CANCEL_RUN`) |
| S07 Kill worker | `panels/ControlBar.tsx` + worker guard | implemented (`KILL_WORKER`) |
| S08 tabs | `panels/Header.tsx` | implemented (4 tabs, workspace modes) |
| S09 HUD | `panels/Header.tsx` + `state/metrics.ts` | implemented (server values; derived fallback labelled; null→"—") |
| S10 time-travel scrubber | `panels/Scrubber.tsx` | implemented (`HISTORY_REVISION_SET`, replay) |
| S11 nav rail 72px | `panels/NavRail.tsx` | implemented (6 destinations, roving arrows) |
| S12 library 260px | `panels/LibraryPanel.tsx` | implemented (derived active agents + templates) |
| S13 minimap | `flow/Canvas.tsx` (`MiniMap`) | implemented (aria label, pan/zoom) |
| S14 refusal toast | `panels/ToastRegion.tsx` | implemented (code+message+how_to_fix, motion, dismiss) |
| S15 SuperAgent card | `flow/nodes.tsx` | implemented |
| S16 WorkerAgent card | `flow/nodes.tsx` | implemented |
| S17 ToolAgent card | `flow/nodes.tsx` | implemented |
| S18 broken-route error card | `flow/nodes.tsx` | implemented (4 types + 1 error, R-U5) |
| S19 agent chips/state dot | `flow/nodes.tsx` (`StateGlyph`), `panels/AgentsPanel.tsx` | implemented |
| S20 zoom controls + Ctrl+0 | `flow/Canvas.tsx`, `a11y/keyboard.ts` | implemented (0.1–3.0 explicit; Meta alias only) |
| S21 settings/key status | `panels/SettingsPanel.tsx` + `api/health.ts` | implemented (`SETTINGS_STATUS`, `/health`; unprobed labeled) |
| S22 file/code viewer | `panels/DataPanel.tsx` | implemented (patched `FILE_TREE_LIST`/`FILE_READ`; jail pre-check; honest unavailable/error states) |
| S23 brand lockup | `styles/tokens.css` + inline PROPOSED-ADDITION mark | partial: token/brand values from PDF; FigCraft exports recorded, not bundled (Phase-12 decision) |
| S24 elevation e1/e2/e3 | `tokens/tokens.ts` + `styles/tokens.css` | implemented (one printed value; tier differentiation UNVERIFIED) |
| S25 code-diff modal | `panels/DiffDecisionModal.tsx` | DECISION-GATED (no DIFF_* command; explicit gap copy) |
| S26 state pack | `panels/StatePack.tsx`, `panels/DevGallery.tsx` (DEV only) | implemented (loading/empty/error/offline/replay/refusal) |
| S27 streamed reply | `chat/ChatCore.tsx` (`REPLY_DELTA`/`REPLY_DONE`) | implemented |
| S28 project identity | `panels/Header.tsx`, `panels/LibraryPanel.tsx` | implemented (client-generated project id; no invented identity) |

Approval overlay (patched contract) `panels/ApprovalOverlay.tsx`: `APPROVAL_RESOLVE` with server nonce; without a
nonce it shows a named contract-gap state and disables resolution instead of inventing one.

## 3. Contract wiring (frozen `hiverouter.v1` + 2026-11-06 patch)

- `api/contract.ts`: 14 frozen commands (10 original + `APPROVAL_RESOLVE`, `SETTINGS_STATUS`, `FILE_TREE_LIST`,
  `FILE_READ`), 21 events (18 + 3 results), 40 documented refusal codes (incl. `E_APPROVAL_NOT_FOUND`,
  `E_APPROVAL_DECISION_INVALID`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`, `E_FILE_TOO_LARGE`), typed
  decode for every event, envelope validation, uuid-v7 ids, prompt/zoom/path/file-size validation.
- `api/client.ts`: `GET /api/session` → token/schema/last_seq/ws → `ws?token&last_seq` → raw
  `{type:"hello",last_seq}` (Phase-2 recon) → envelopes; commands resolve **only on ACK**, reject on typed
  `ERROR`; malformed frames surface `E_SCHEMA_INVALID`; reconnect replays hello with last_seq; no fake success.
- Contract-matrix test evidence: `tests/contract.test.ts` (14 names, 21 events, patched codes, typed payloads,
  preconditions) and `tests/apiClient.test.ts` (hello/token URL, ACK resolve, ERROR reject, malformed, jail).

## 4. G-UI gate status

| Gate | Evidence | State |
|---|---|---|
| G-UI-1 contrast table | `tests/tokens.test.ts` (6 declared rows ±0.05; meta ink-500 ≥4.5; focus ink-700 ≥3; amber large-only) | PASS |
| G-UI-2 Ctrl+0 recentre | `tests/keyboard.test.ts` (Windows chord, Meta alias, editable ignored) + Canvas handler | PASS |
| G-UI-3 WS origin/token | client token query + hello; server Origin allowlist is backend Phase 2 | PARTIAL (client side done; server e2e UNVERIFIED) |
| G-UI-4 one state vocabulary | `tests/agentState.test.ts` (15 states ≥13, 4 dots, cues) | PASS |
| G-UI-5 missing-state coverage | state pack + DEV gallery (15 states rendered) | PASS (DEV-only gallery) |
| G-UI-6 reduced motion | `a11y/reducedMotion.ts`, `state/useAppReducedMotion.ts`, CSS override + settings toggle | PASS (code/CSS; pixel effect UNVERIFIED) |
| G-UI-7 zoom clamp | explicit minZoom/maxZoom + `clampZoom` test + `validateZoom` | PASS |
| G-UI-8 tabs-vs-rail owner rule | rail = destinations; tabs = workspace modes only (never both as peers) | PASS (recorded decision) |
| G-UI-9 font decision | Playfair (display) + Inter (UI) chosen from PDF; bundling not installed at W1; explicit fallback stack | PARTIAL/UNVERIFIED |
| G-UI-10 diff modal decision | decision-gated modal; no DIFF_* command implemented | PASS (decision-gated) |
| G-UI-11 token isolation | exactly 20 swatches; `tests/noHardcoded.test.ts` zero hex outside `tokens/`; token/provenance tests | PASS |
| G-UI-12 no pixel-layout claim | no assertion of pixel fidelity; asset + UI docs mark UNVERIFIED/needs_human | PASS (UNVERIFIED stated) |

## 5. FigCraft asset gate (wired after watchdog PASS-WITH-NOTES)

- Watchdog verdict read: `ledgers/ui-watchdog.json` → overall PASS-WITH-NOTES, **allow_frontend=true, FAIL 0**,
  A 28/28 surfaces + B 27/27 (≥24), A tokens 20/20 exact.
- `web/src/assets/manifest.ts` now records the **8 accepted export groups** (A foundations/brand/components/specimens;
  B canvas-default/zoom-states/interaction/hero) with their audited sha256; `FIGCRAFT_GATE.allowFrontend=true`.
- Rules kept: token values from `src/tokens/*` only (never sampled pixels); the A `Specimens` off-token grey spinner
  is recorded and **not adopted**; pixel-layout fidelity stays UNVERIFIED; S25 stays decision-gated.
- Assets are not copied into the web bundle at W1 (boards remain reference evidence under
  `docs/ui/figcraft/ASSETS/`); bundling is a Phase-12 packaging decision. No fabricated provenance anywhere.

## 6. Known gaps / UNVERIFIED (not hidden)

1. **Live end-to-end not run**: backend `/ws`, `/api/session` flow, patched `SETTINGS_STATUS`/`FILE_*`/
   `APPROVAL_RESOLVE` implementations are Phase 2/6 work in progress. UI is wired against the frozen contract;
   unit tests use a fake transport + fake API, not the real socket.
2. **Node metadata extension**: frozen NODE_* payload = `{task_id,state}` only; kind/label/parent/progress are
   optional extras (forward-compatible). Without them cards are generic — contract-extension request recorded.
3. **Approval nonce**: `APPROVAL_RESOLVE` requires a nonce but the frozen `APPROVAL_REQUEST` shape does not list
   one; UI reads an optional `nonce` field and refuses with a named gap when absent (no invented nonce).
4. **Fonts**: Playfair/Inter chosen but not bundled (no @fontsource install at W1); fallback stack explicit.
5. **Bundle size**: 669 kB JS / 213 kB gzip single chunk (warning, not failure) → code-split follow-up.
6. **No browser e2e / Playwright run** (browser binaries intentionally not installed); no visual screenshot claim.
7. **AI-proposed-only**: the `motion` animation, `dagre` layout and `partysocket` reconnection are exercised in
   unit tests only at the pure-function level; live reconnect timing is UNVERIFIED.
8. `web/src/panels/DevGallery.tsx` renders only when `import.meta.env.DEV`; production build returns `null`
   (verified by the `noHardcoded` scan? no — verified by code inspection and Vite PROD define; live prod check UNVERIFIED).
