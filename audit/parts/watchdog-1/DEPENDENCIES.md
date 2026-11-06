# DEPENDENCIES — watchdog-1 partition (`web/**`)

Checkpoint HEAD `3988622` · 55 files · read-only audit. Scope: npm externals, local ES-module graph, cross-partition links, dependency risks.

## 1. External npm dependencies (pinned, from `web/package.json`; lockfile `web/package-lock.json` verified by manifest hash)

| Package | Scope | Version | Import sites in web/src+tests |
|---|---|---|---|
| `@dagrejs/dagre` | runtime | `3.1.3` | 1 |
| `@xyflow/react` | runtime | `12.12.1` | 2 |
| `motion` | runtime | `14.9.1` | 1 |
| `partysocket` | runtime | `1.3.0` | 1 |
| `react` | runtime | `19.3.1` | 14 |
| `react-dom` | runtime | `19.3.1` | 1 |
| `zustand` | runtime | `5.0.16` | 3 |
| `@tailwindcss/vite` | dev | `4.3.4` | 0 |
| `@types/react` | dev | `19.3.0` | 0 |
| `@types/react-dom` | dev | `19.3.0` | 0 |
| `@vitejs/plugin-react` | dev | `6.2.1` | 0 |
| `tailwindcss` | dev | `4.3.4` | 0 |
| `typescript` | dev | `7.0.2` | 0 |
| `vite` | dev | `8.3.6` | 0 |
| `vitest` | dev | `5.1.0` | 8 |

No dependency is imported but undeclared; no declared dependency is unused except dev tooling used only by config/CLI (`typescript` via script, `tailwindcss` via its Vite plugin). `@types/react`/`@types/react-dom` are type-only.

## 2. npm import matrix (module → sites)

- `@dagrejs/dagre` → web/src/flow/graph.ts:7
- `@xyflow/react` → web/src/flow/graph.ts:8, web/src/flow/nodes.tsx:6
- `motion` → web/src/panels/ToastRegion.tsx:2
- `partysocket` → web/src/api/client.ts:13
- `react` → web/src/App.tsx:6, web/src/main.tsx:1, web/src/a11y/reducedMotion.ts:6, web/src/chat/ChatCore.tsx:7, web/src/flow/Canvas.tsx:8, web/src/flow/nodes.tsx:7, web/src/panels/ControlBar.tsx:6, web/src/panels/DataPanel.tsx:2, web/src/panels/Header.tsx:2, web/src/panels/NavRail.tsx:2, web/src/panels/Scrubber.tsx:2, web/src/panels/SettingsPanel.tsx:2, web/src/panels/StatePack.tsx:2, web/src/state/context.ts:4
- `react-dom` → web/src/main.tsx:2
- `vitest` → web/tests/agentState.test.ts:1, web/tests/apiClient.test.ts:1, web/tests/contract.test.ts:1, web/tests/keyboard.test.ts:1, web/tests/metrics.test.ts:1, web/tests/noHardcoded.test.ts:1, web/tests/store.test.ts:1, web/tests/tokens.test.ts:1
- `zustand` → web/src/state/context.ts:5, web/src/state/context.ts:6, web/src/state/store.ts:7

## 3. Local ES-module graph (edges as written in source; resolution targets verified on disk)

- web/src/a11y/keyboard.ts -> ../api/contract
- web/src/api/client.ts -> ./ids
- web/src/api/client.ts -> ./refusals
- web/src/api/contract.ts -> ../state/agentState
- web/src/api/health.ts -> ./client
- web/src/api/health.ts -> ./refusals
- web/src/api/index.ts -> ./client
- web/src/api/index.ts -> ./mock
- web/src/api/mock.ts -> ../state/agentState
- web/src/api/mock.ts -> ./ids
- web/src/api/mock.ts -> ./refusals
- web/src/api/refusals.ts -> ../state/strings
- web/src/api/refusals.ts -> ./contract
- web/src/App.tsx -> ./api
- web/src/App.tsx -> ./chat/ChatCore
- web/src/App.tsx -> ./flow/Canvas
- web/src/App.tsx -> ./panels/AgentsPanel
- web/src/App.tsx -> ./panels/ApprovalOverlay
- web/src/App.tsx -> ./panels/ControlBar
- web/src/App.tsx -> ./panels/DataPanel
- web/src/App.tsx -> ./panels/DiffDecisionModal
- web/src/App.tsx -> ./panels/Header
- web/src/App.tsx -> ./panels/LibraryPanel
- web/src/App.tsx -> ./panels/ModelsPanel
- web/src/App.tsx -> ./panels/NavRail
- web/src/App.tsx -> ./panels/RoutingPanel
- web/src/App.tsx -> ./panels/Scrubber
- web/src/App.tsx -> ./panels/SettingsPanel
- web/src/App.tsx -> ./panels/ToastRegion
- web/src/App.tsx -> ./state/context
- web/src/App.tsx -> ./state/store
- web/src/App.tsx -> ./state/strings
- web/src/App.tsx -> ./state/useAppReducedMotion
- web/src/chat/ChatCore.tsx -> ../state/context
- web/src/chat/ChatCore.tsx -> ../state/strings
- web/src/chat/ChatCore.tsx -> ../state/useAppReducedMotion
- web/src/flow/BeeEdge.tsx -> ../a11y/keyboard
- web/src/flow/Canvas.tsx -> ../a11y/keyboard
- web/src/flow/Canvas.tsx -> ../state/context
- web/src/flow/Canvas.tsx -> ../state/strings
- web/src/flow/Canvas.tsx -> ../state/useAppReducedMotion
- web/src/flow/Canvas.tsx -> ./BeeEdge
- web/src/flow/Canvas.tsx -> ./graph
- web/src/flow/Canvas.tsx -> ./nodes
- web/src/flow/graph.ts -> ../state/agentState
- web/src/flow/graph.ts -> ../state/store
- web/src/flow/nodes.tsx -> ../state/agentState
- web/src/flow/nodes.tsx -> ../state/strings
- web/src/flow/nodes.tsx -> ./graph
- web/src/main.tsx -> ./App
- web/src/panels/AgentsPanel.tsx -> ../flow/nodes
- web/src/panels/AgentsPanel.tsx -> ../state/agentState
- web/src/panels/AgentsPanel.tsx -> ../state/context
- web/src/panels/AgentsPanel.tsx -> ../state/strings
- web/src/panels/AgentsPanel.tsx -> ./StatePack
- web/src/panels/ApprovalOverlay.tsx -> ../state/context
- web/src/panels/ApprovalOverlay.tsx -> ../state/strings
- web/src/panels/ControlBar.tsx -> ../state/agentState
- web/src/panels/ControlBar.tsx -> ../state/context
- web/src/panels/ControlBar.tsx -> ../state/strings
- web/src/panels/DataPanel.tsx -> ../state/context
- web/src/panels/DataPanel.tsx -> ../state/strings
- web/src/panels/DataPanel.tsx -> ./StatePack
- web/src/panels/DevGallery.tsx -> ../flow/nodes
- web/src/panels/DevGallery.tsx -> ../state/agentState
- web/src/panels/DevGallery.tsx -> ../state/agentState
- web/src/panels/DevGallery.tsx -> ../state/strings
- web/src/panels/DiffDecisionModal.tsx -> ../state/context
- web/src/panels/DiffDecisionModal.tsx -> ../state/strings
- web/src/panels/Header.tsx -> ../state/context
- web/src/panels/Header.tsx -> ../state/metrics
- web/src/panels/Header.tsx -> ../state/store
- web/src/panels/Header.tsx -> ../state/strings
- web/src/panels/Header.tsx -> ./StatePack
- web/src/panels/LibraryPanel.tsx -> ../assets/manifest
- web/src/panels/LibraryPanel.tsx -> ../state/agentState
- web/src/panels/LibraryPanel.tsx -> ../state/context
- web/src/panels/LibraryPanel.tsx -> ../state/strings
- web/src/panels/ModelsPanel.tsx -> ../state/context
- web/src/panels/ModelsPanel.tsx -> ../state/strings
- web/src/panels/ModelsPanel.tsx -> ./StatePack
- web/src/panels/NavRail.tsx -> ../a11y/keyboard
- web/src/panels/NavRail.tsx -> ../state/context
- web/src/panels/NavRail.tsx -> ../state/store
- web/src/panels/NavRail.tsx -> ../state/strings
- web/src/panels/RoutingPanel.tsx -> ../state/agentState
- web/src/panels/RoutingPanel.tsx -> ../state/context
- web/src/panels/RoutingPanel.tsx -> ../state/strings
- web/src/panels/RoutingPanel.tsx -> ./StatePack
- web/src/panels/Scrubber.tsx -> ../state/context
- web/src/panels/Scrubber.tsx -> ../state/strings
- web/src/panels/SettingsPanel.tsx -> ../state/context
- web/src/panels/SettingsPanel.tsx -> ../state/strings
- web/src/panels/SettingsPanel.tsx -> ./DevGallery
- web/src/panels/SettingsPanel.tsx -> ./StatePack
- web/src/panels/StatePack.tsx -> ../state/strings
- web/src/panels/ToastRegion.tsx -> ../state/context
- web/src/panels/ToastRegion.tsx -> ../state/strings
- web/src/panels/ToastRegion.tsx -> ../state/useAppReducedMotion
- web/src/state/agentState.ts -> ./strings
- web/src/state/context.ts -> ./store
- web/src/state/metrics.ts -> ../api/contract
- web/src/state/metrics.ts -> ./agentState
- web/src/state/store.ts -> ../api/client
- web/src/state/store.ts -> ../api/contract
- web/src/state/store.ts -> ../api/contract
- web/src/state/store.ts -> ../api/health
- web/src/state/store.ts -> ../api/ids
- web/src/state/store.ts -> ../api/refusals
- web/src/state/store.ts -> ./metrics
- web/src/state/store.ts -> ./strings
- web/src/state/useAppReducedMotion.ts -> ../a11y/reducedMotion
- web/src/state/useAppReducedMotion.ts -> ./context
- web/tests/agentState.test.ts -> ../src/state/strings
- web/tests/apiClient.test.ts -> ../src/api/client
- web/tests/apiClient.test.ts -> ../src/api/contract
- web/tests/apiClient.test.ts -> ../src/api/refusals
- web/tests/contract.test.ts -> ../src/api/ids
- web/tests/keyboard.test.ts -> ../src/a11y/keyboard
- web/tests/metrics.test.ts -> ../src/state/metrics
- web/tests/metrics.test.ts -> ../src/state/store
- web/tests/noHardcoded.test.ts -> ../src/tokens/tokens
- web/tests/store.test.ts -> ../src/api/client
- web/tests/store.test.ts -> ../src/api/contract
- web/tests/store.test.ts -> ../src/api/refusals
- web/tests/store.test.ts -> ../src/state/store
- web/tests/tokens.test.ts -> ../src/tokens/contrast

Layering observed: `tokens` (no deps) → `api/contract` → `api/*` → `state/store` → `state/context` → panels/flow/chat/App; `a11y` is leaf-level; `styles/*.css` imported only by entry/bootstrap. No cycles were found in this edge list (each edge goes from a higher-level module to a lower-level/loader utility; `api/mock` imports `state/agentState` only).

## 4. Cross-partition links

- `web/**` does **not** import any backend, e2e, tools, docs or runtime-test file; the boundary is the frozen HTTP/WS contract.
- Duplicated contract: `web/src/api/contract.ts` re-implements the wire types also owned by `backend/crates/hiverouter-contract/src/{envelope,command,event,refusal,state}.rs`. Shared authority is `docs/contracts/UI-COMMANDS-v1.md`; drift risk remains (finding F-08).
- `e2e/**` drives the shipped `web/dist` through Playwright (owned by the e2e seat); the web partition ships the bundle the e2e seat asserts. `backend/**` serves `web/dist` via rust-embed (closeout hash-identity check PASS).
- Build chain: `web/package.json` → `vite.config.ts` (React + Tailwind plugins) → `web/dist` → served by the Rust core at `http://127.0.0.1:8799/`.

## 5. Runtime/environment dependencies

- Browser APIs: WebSocket (partysocket), `fetch` (/api/session, /health), `crypto.getRandomValues` (uuid-v7), `requestAnimationFrame`, `matchMedia`, `document` (canvas shortcut + minimap aria bridge). No Node-only API in product code.
- Network endpoints (hard-coded in code/config, all localhost): `GET /api/session`, `GET /health`, `ws /ws?token&last_seq`. Base URL resolution: `VITE_API_BASE` env → else same-origin → else `http://127.0.0.1:8080` when dev port 5173 (`web/src/api/client.ts`).
- No secrets, tokens, or credentials in any web file: the WS token is fetched at runtime and used only in the URL query; no storage/logging of it.

## 6. Dependency risks

1. **Contract duplication** (web TS ↔ Rust contract crate) — same vocabulary, two type systems; frozen-doc-driven (F-08).
2. **Runtime coupling to frozen payload extras**: NODE_* kind/label/parent/progress are optional client extensions; absent metadata yields generic cards (honest fallback, but a rendering-quality dependency).
3. **Heavy runtime bundle**: @xyflow/react + motion + dagre dominate the 669 kB single chunk; no code-splitting configured (F-16).
4. **partysocket**: reconnection semantics are delegated to the library; client adds hello replay on reopen. Behaviour under rapid close/open is UNVERIFIED by unit tests (would need live socket).
5. **@dagrejs/dagre layout**: deterministic for a given graph but not width-aware for long labels; nodes may overlap at small zoom (visual UNVERIFIED, no pixel claim).
6. **Tailwind 4 Vite plugin + CSS mirror**: token values duplicated in `tokens.ts` and `tokens.css`; single-edit drift risk (F-04).
