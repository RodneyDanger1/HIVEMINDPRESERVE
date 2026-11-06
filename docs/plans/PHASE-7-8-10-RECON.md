# PHASE 7 / 8 / 10 — FRONTEND RECON LEDGER

Date: 2026-11-06 · Owner: FRONTEND seat · Scope: `web/**` only (no backend/blueprint/ledger edits)

## Recon (forgewright-recon format)

- **Target behavior.** Deliver the proposed HiveMind v1 spatial UI (blueprint Phases 7/8/10 + UI-WIRING-GUIDE
  S01–S28) as a localhost React 19 bundle that speaks ONLY the frozen `hiverouter.v1` contract
  (`docs/contracts/UI-COMMANDS-v1.md`, patched 2026-11-06), with one agent-state vocabulary, PDF-derived
  tokens, an honest dev-only mock, and named refusals — no fake success on any path.
- **Entry points.** `web/index.html` → `web/src/main.tsx` → `web/src/App.tsx` (store provider + chrome);
  contract boundary: `web/src/api/contract.ts` + `web/src/api/client.ts`; state: `web/src/state/store.ts`;
  token module: `web/src/tokens/tokens.ts` + `web/src/styles/tokens.css`.
- **Callers/consumers (verified by import graph).**
  - `App.tsx` → NavRail, Header, LibraryPanel, ControlBar, Scrubber, ToastRegion, Canvas, ChatCore,
    SettingsPanel, DataPanel, AgentsPanel, ModelsPanel, RoutingPanel, ApprovalOverlay, DiffDecisionModal.
  - `Canvas.tsx` → `flow/graph.ts` (dagre layout) → `flow/nodes.tsx` (4 node types + broken-route error card),
    `flow/BeeEdge.tsx` (source→target bee direction).
  - `state/store.ts` → `api/client.ts` (ACK-before-mutation, typed ERROR rejection) | `api/mock.ts` (dev only) |
    `api/health.ts` (`GET /health`) | `state/metrics.ts` (HUD null-aware derivation).
  - `ChatCore.tsx` → `SUBMIT_PROMPT`; `ControlBar.tsx` → `SPAWN_SUB_HIVE`/`REWIRE_TASK`/`CANCEL_RUN`/`KILL_WORKER`;
    `LibraryPanel.tsx` → `TEMPLATE_INSERT`; `Scrubber.tsx` → `HISTORY_REVISION_SET`; `ApprovalOverlay.tsx` →
    `APPROVAL_RESOLVE`; `SettingsPanel.tsx` → `SETTINGS_STATUS`; `DataPanel.tsx` → `FILE_TREE_LIST`/`FILE_READ`.
- **Tests + command.** `npm.cmd run test` (vitest 5.1.0, node env): 8 files / 44 tests — tokens+contrast,
  state enum, frozen contract (14 commands/21 events/40 refusal codes), WS client ACK/ERROR/hello/malformed/jail,
  store ACK-before-mutation + refusal + approvals, keyboard Ctrl+0/zoom/clamp, HUD null derivation,
  token-isolation + no-hardcoded copy + asset-seam scan. Plus `npm.cmd run typecheck` and `npm.cmd run build`.
- **Data flow (happy).** Prompt → `SUBMIT_PROMPT` COMMAND (uuid-v7 id/correlation) → WS `?token&last_seq` →
  raw `{type:"hello",last_seq}` → `ACK` resolves → `RUN_CREATED`/`NODE_*`/`REPLY_*`/`RUN_METRICS` update the store →
  canvas cards, HUD, chat stream, scrubber. **Failure path.** Named `ERROR` frame → promise rejects (no state
  mutation) → `REFUSED BY NAME · NOT SILENT` toast with code + how_to_fix; malformed frame → client
  `E_SCHEMA_INVALID`; socket loss → reconnecting banner + mutations blocked.
- **Unknowns / evidence still missing (blocking e2e only, not implementation).**
  - Backend `/ws`, `/api/session`, `SETTINGS_STATUS`, `FILE_*`, `APPROVAL_RESOLVE` implementations are Phase 2/6
    work in progress (frozen contract + Phase-2 recon handshake implemented client-side; live e2e not yet runnable).
  - NODE_* frozen payload carries only `task_id,state`; kind/label/parent/progress are read as optional extras
    (forward-compatible). Absent metadata renders a generic task card — recorded as a contract-extension request.
  - Font bundling: display face Playfair + UI face Inter chosen from PDF; @fontsource bundling is not installed at
    W1 (explicit fallback stack; UNVERIFIED — see evidence).
  - FigCraft exports accepted by the watchdog; assets are reference boards, not copied into the web bundle
    (Phase-12 packaging decision). Pixel-layout fidelity stays UNVERIFIED/needs_human.
- **Files expected to touch (all inside `web/**`)** plus `docs/plans/PHASE-7-8-10-{RECON,EVIDENCE}.md` and the
  claim register: `package.json`, config files, `src/**`, `tests/**`.

## Scope check against the blueprint cards

- **Phase 7 (Canvas/Rendering/Design):** 4 node types + broken-route error state; 20-swatch token module + corrected
  contrast; FigCraft gate wired; 0.1–3.0 zoom with library defaults overridden; one snapping/renderer/edge concept
  represented by single implementations (no duplicate renderers).
- **Phase 8 (Interaction/Animation/Navigation):** Ctrl+0/Meta alias recentre, zoom controls and shortcuts, bee
  direction marker, reduced-motion path, keyboard/focus a11y, nav rail + tabs owner rule.
- **Phase 10 (UI Surfaces/Controls/A11y):** 28 surfaces (S01–S28) implemented or honestly gated (S25 decision-gated,
  S22 unavailable state if server route absent), 15-state enum mapped to 4 PDF dots + shape/text cues, refusal
  toast, approval overlay, state pack, settings/key status, HUD null-aware, scrubber/replay.
