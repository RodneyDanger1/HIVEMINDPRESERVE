# FILES — watchdog-1 partition (`web/**`)

Audit checkpoint: HEAD `3988622`; partition: **55 git-tracked `web/**` files**; deep: **16** (DEEP-LIST.md); standard: **39**. Manifest: `audit/inventory/FILE-MANIFEST.csv`. Read-only: no product/git/rebuild writes.

**Method.** Every row below carries the manifest bytes/sha256, re-hashed in this audit (55/55 match). Behavioural claims cite ``path:line`` from content reads (full-file line scans for all 55; symbol maps for the 16 deep files; imports resolved for the dependency graph). Deep files get 8 paragraphs (mechanics → logic/algorithm → edge/failure → dependency interactions → quality/evidence); standard files get a structured 1–2 paragraph record.

**Scope limits (per WAVE-SHEET).** Generated/ignored trees are censused, not analyzed: ``web/node_modules`` (3,976 files/110 MB), ``web/dist`` (3 files, shipped artifact), plus repo-wide ``target/``, ``e2e/node_modules``, ``tools/w64devkit``, ``runtime-test/``. Binary/generated files: ``web/package-lock.json`` is verified by hash only. No browser was run by this seat (e2e belongs to the e2e seat); UI runtime claims are therefore ``READ/INFERRED`` unless backed by unit tests or the FINAL-REPORT gate row.

| # | Path | Bytes | Depth | Language |
|---:|---|---:|---|---|
| 1 | `web/index.html` | 676 | standard | HTML |
| 2 | `web/package.json` | 929 | deep | JSON |
| 3 | `web/package-lock.json` | 78338 | standard | JSON |
| 4 | `web/src/a11y/keyboard.ts` | 2976 | deep | TypeScript |
| 5 | `web/src/a11y/reducedMotion.ts` | 1051 | standard | TypeScript |
| 6 | `web/src/api/client.ts` | 15930 | deep | TypeScript |
| 7 | `web/src/api/contract.ts` | 28868 | deep | TypeScript |
| 8 | `web/src/api/health.ts` | 1885 | standard | TypeScript |
| 9 | `web/src/api/ids.ts` | 1607 | standard | TypeScript |
| 10 | `web/src/api/index.ts` | 751 | standard | TypeScript |
| 11 | `web/src/api/mock.ts` | 12437 | standard | TypeScript |
| 12 | `web/src/api/refusals.ts` | 1582 | standard | TypeScript |
| 13 | `web/src/App.tsx` | 3961 | deep | TypeScript/TSX |
| 14 | `web/src/assets/manifest.ts` | 4888 | standard | TypeScript |
| 15 | `web/src/chat/ChatCore.tsx` | 6270 | deep | TypeScript/TSX |
| 16 | `web/src/flow/BeeEdge.tsx` | 1686 | standard | TypeScript/TSX |
| 17 | `web/src/flow/Canvas.tsx` | 8823 | deep | TypeScript/TSX |
| 18 | `web/src/flow/graph.ts` | 3232 | standard | TypeScript |
| 19 | `web/src/flow/nodes.tsx` | 6451 | deep | TypeScript/TSX |
| 20 | `web/src/main.tsx` | 404 | standard | TypeScript/TSX |
| 21 | `web/src/panels/AgentsPanel.tsx` | 2038 | standard | TypeScript/TSX |
| 22 | `web/src/panels/ApprovalOverlay.tsx` | 2857 | standard | TypeScript/TSX |
| 23 | `web/src/panels/ControlBar.tsx` | 5023 | standard | TypeScript/TSX |
| 24 | `web/src/panels/DataPanel.tsx` | 3077 | deep | TypeScript/TSX |
| 25 | `web/src/panels/DevGallery.tsx` | 1044 | standard | TypeScript/TSX |
| 26 | `web/src/panels/DiffDecisionModal.tsx` | 1265 | standard | TypeScript/TSX |
| 27 | `web/src/panels/Header.tsx` | 3268 | standard | TypeScript/TSX |
| 28 | `web/src/panels/LibraryPanel.tsx` | 2994 | standard | TypeScript/TSX |
| 29 | `web/src/panels/ModelsPanel.tsx` | 1195 | standard | TypeScript/TSX |
| 30 | `web/src/panels/NavRail.tsx` | 2251 | standard | TypeScript/TSX |
| 31 | `web/src/panels/RoutingPanel.tsx` | 1888 | standard | TypeScript/TSX |
| 32 | `web/src/panels/Scrubber.tsx` | 1782 | standard | TypeScript/TSX |
| 33 | `web/src/panels/SettingsPanel.tsx` | 5186 | deep | TypeScript/TSX |
| 34 | `web/src/panels/StatePack.tsx` | 1816 | standard | TypeScript/TSX |
| 35 | `web/src/panels/ToastRegion.tsx` | 2072 | standard | TypeScript/TSX |
| 36 | `web/src/state/agentState.ts` | 4509 | deep | TypeScript |
| 37 | `web/src/state/context.ts` | 811 | standard | TypeScript |
| 38 | `web/src/state/metrics.ts` | 1710 | standard | TypeScript |
| 39 | `web/src/state/store.ts` | 23407 | deep | TypeScript |
| 40 | `web/src/state/strings.ts` | 12692 | deep | TypeScript |
| 41 | `web/src/state/useAppReducedMotion.ts` | 388 | standard | TypeScript |
| 42 | `web/src/styles/index.css` | 3601 | standard | CSS |
| 43 | `web/src/styles/tokens.css` | 2858 | standard | CSS |
| 44 | `web/src/tokens/contrast.ts` | 1683 | deep | TypeScript |
| 45 | `web/src/tokens/tokens.ts` | 7070 | deep | TypeScript |
| 46 | `web/tests/agentState.test.ts` | 1819 | standard | TypeScript |
| 47 | `web/tests/apiClient.test.ts` | 4998 | standard | TypeScript |
| 48 | `web/tests/contract.test.ts` | 4472 | standard | TypeScript |
| 49 | `web/tests/keyboard.test.ts` | 2409 | standard | TypeScript |
| 50 | `web/tests/metrics.test.ts` | 1484 | standard | TypeScript |
| 51 | `web/tests/noHardcoded.test.ts` | 2228 | standard | TypeScript |
| 52 | `web/tests/store.test.ts` | 7331 | standard | TypeScript |
| 53 | `web/tests/tokens.test.ts` | 2170 | standard | TypeScript |
| 54 | `web/tsconfig.json` | 618 | standard | JSON |
| 55 | `web/vite.config.ts` | 798 | deep | TypeScript |

## web/index.html

- **Metadata:** 676 bytes · sha256 `BC8E7E4D9AFEFAC2A39F20E20AB3DFB77E68AB1585D86CAB5EA1127FC1702D2A` (manifest; re-hashed in audit: MATCH) · HTML · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Vite HTML shell mounting #root, meta/theme/title and a W4-added data-URI favicon.

- **Mechanics.** Vite HTML shell mounting #root, meta/theme/title and a W4-added data-URI favicon. It imports `none (no ES imports)` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 3/5 · security 3/5 · docs 3/5 · evidence 4/5 — read; shell only; favicon hex outside token module (F-07).
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/package.json

- **Metadata:** 929 bytes · sha256 `A80E90B9F0182EDA2AAA5AAFF138F99A4BCDD7794D717696291E1F9BF2353D36` (manifest; re-hashed in audit: MATCH) · JSON · config-manifest/build/dependency/config · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** Pins the entire frontend toolchain and scripts for dev/typecheck/build/test.

**Mechanics (deep analysis).**
1. The manifest pins every runtime dependency to an exact version (react 19.3.1, @xyflow/react 12.12.1, zustand 5.0.16, @dagrejs/dagre 3.1.3, motion 14.9.1, partysocket 1.3.0) and every dev dependency (vite 8.3.6, typescript 7.0.2, tailwindcss 4.3.4, vitest 5.1.0), matching the blueprint/fact-register stack; ranges are absent, so installs are reproducible.
2. Scripts encode the gate: `typecheck` is `tsc --noEmit`, `build` is `tsc --noEmit && vite build`, `test` is `vitest run`, `dev`/`preview` bind strict localhost ports (5173/4173). This makes the green build/test evidence auditable from package.json:12-15.
3. `engines.node >=22.12.0` matches the blueprint floor (host Node v26.7.0). `type: module` and private:true prevent accidental publish; no `lint`, `format`, or `check:stale` scripts exist, so style drift is not machine-gated.
4. Dependencies are deliberately minimal for a 28-surface UI: React, React Flow, dagre layout, motion, zustand, partysocket. Font packages are absent even though `styles/tokens.css:30-32` names Playfair Display/Inter, which is the root cause of the font fallback finding F-01.
5. There is no `overrides`/`resolutions` block and no audit exceptions; package-lock.json (78 KB, manifest-verified) is committed, so transitive resolution is frozen. No dev server proxy is configured in the manifest â€” VITE_API_BASE/defaultBaseUrl in `src/api/client.ts` handles dev cross-origin instead.
6. Risk: the six runtime packages include three large code contributors (React Flow, motion, dagre), consistent with the single 670 kB bundle observed in the build log; the manifest offers no bundling/chunking options because those belong to vite.config.ts.
7. Quality classification: correctness 5 (exact pins, verified installs), maintainability 4 (clear scripts, but no lint/format), modularity 4 (single app package, no workspaces split), error handling 3 (nothing here), security 4 (no secrets/scripts with side effects), docs 4 (descriptions absent but self-evident), evidence 5.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 4/5 · error handling 3/5 · security 4/5 · docs 4/5 · evidence 5/5 — exact-pin manifest; scripts tsc/vite/vitest; no lint/format script.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/package-lock.json

- **Metadata:** 78338 bytes · sha256 `2F141BD0E8CBE08ABCB6F004FC9C08657331F23BC5AC1E372328777A9373D3BB` (manifest; re-hashed in audit: MATCH) · JSON · config-manifest/build/dependency/config · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Committed npm lockfile freezing the exact transitive dependency tree (78 KB).

- **Mechanics.** Committed npm lockfile freezing the exact transitive dependency tree (78 KB). It imports `none (no ES imports)` and is imported by `none in web/src or web/tests`. Classification: config-manifest.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 4/5 · error handling 3/5 · security 4/5 · docs 3/5 · evidence 5/5 — manifest sha verified; generated file, not hand-edited.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/a11y/keyboard.ts

- **Metadata:** 2976 bytes · sha256 `72BAE334D9D7A4AAD2CDD4C54746774F89165AA985C46DAEC540428D9A5BA967` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** Pure keyboard/focus helpers: Ctrl+0 recentre, zoom shortcuts, zoom clamp, bee angle, roving focus.

**Mechanics (deep analysis).**
1. `handleCanvasShortcut` (line 52) is a pure decision function: it rejects editable targets via `isEditableElement` (line 25), ignores Alt, then maps Ctrl/Meta+`0`/`Digit0`/`Numpad0` to recenter, Ctrl+= / Ctrl+- to zoom, and Ctrl+1/Ctrl+f to fit. Returning a boolean lets the caller preventDefault/stopPropagation deterministically.
2. Editable-target detection is duck-typed rather than `instanceof HTMLElement` (line 25-49), which works both in browsers and in the Node test environment; inputs, textareas, selects and contenteditable (attribute or property) are protected so typing is never hijacked â€” this is the fix that made Ctrl+0 testable and behaviourally safe.
3. `clampZoom` (line 83) enforces the frozen 0.1â€“3.0 range and treats non-finite input as MIN_ZOOM; it is imported by no other module directly, but provides the single clamp primitive paired with contract.ts `validateZoom` used by the API layer.
4. `beeAngle` (line 96) computes `atan2(dy,dx)` for the sourceâ†’target data direction, giving the BeeEdge glyph its routing orientation (R-U6); it is pure and direction-tested with cardinal vectors.
5. `nextRovingIndex` (line 101) implements wrap-around focus order for the nav rail; NavRail.tsx consumes it for ArrowUp/ArrowDown with `refs` and tabIndex management.
6. Edge behaviour: `code` is accepted in addition to `key` so Playwright/automation and non-US layouts still trigger the shortcut; shift/alt modifiers are rejected, and no handler runs inside editable controls. Missing: Escape/command-palette shortcuts are intentionally out of scope.
7. The module imports only `MAX_ZOOM/MIN_ZOOM` from `api/contract`, keeping a11y independent of React and of the store; tests exercise 7 cases, including the physical Digit0 path added for live e2e.
8. Quality: correctness 5 (pure, tested), maintainability 5 (small), modularity 5, error handling 5 (invalid zoom clamped), security 4 (no global state), docs 4, evidence 5 (tests + read).

- **Dependencies:** imports → web/src/a11y/keyboard.ts:5: ../api/contract
- **Importers/callers:** web/src/flow/BeeEdge.tsx:11; web/src/flow/Canvas.tsx:23; web/src/panels/NavRail.tsx:6; web/tests/keyboard.test.ts:2
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read + tests/keyboard.test.ts (7 tests).
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/a11y/reducedMotion.ts

- **Metadata:** 1051 bytes · sha256 `45231D4FEB659362CDE475D94AB32281B225979ACB68532B0CE892B89F10BC52` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** prefers-reduced-motion detection hook with safe Node fallback.

- **Mechanics.** prefers-reduced-motion detection hook with safe Node fallback. It imports `web/src/a11y/reducedMotion.ts:6: react` and is imported by `web/src/state/useAppReducedMotion.ts:5`. Classification: source.

- **Dependencies:** imports → web/src/a11y/reducedMotion.ts:6: react
- **Importers/callers:** web/src/state/useAppReducedMotion.ts:5
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; used by useAppReducedMotion + CSS.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/api/client.ts

- **Metadata:** 15930 bytes · sha256 `6EFA1C7D1DEE8D045C31AA3F326DA1E6B9080DEDB913973ECF03A5F3082E8CA7` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** Typed WS client: /api/session token+hello handshake, ACK-only success, typed ERROR rejection, reconnect/resync.

**Mechanics (deep analysis).**
1. `WsApiClient.open()` fetches `GET /api/session` (baseUrl default: same-origin or http://127.0.0.1:8080 under Vite), validates the four-field session shape, builds `ws?token&last_seq` from `httpToWs`, creates the transport (partysocket `WebSocket` with bounded retries/delays), then awaits socket open and sends the raw `{"type":"hello","last_seq":n}` control frame required by Phase-2 recon.
2. Commands are envelopes built by `buildCommandEnvelope` with uuid-v7 id/correlation, then tracked in a pending map keyed by correlation id with a per-command timer. `routePending` resolves ONLY on a matching `ACK` (command name must match the expected command) and rejects ONLY on a matching `ERROR` frame; malformed frames become a client `E_SCHEMA_INVALID` refusal via `subscribeClientError` instead of a silent drop.
3. Transport failures: close rejects all pending commands with `E_WS_DISCONNECTED` and surfaces `reconnecting`; reopen replays hello with the tracked `last_seq`; `send` refuses to queue while not open, preventing stale mutations. PartyTransport queues zero messages (`maxEnqueuedMessages: 0`), so nothing is silently replayed.
4. Lifecycle defects: `waitForOpen` registers its own `onOpen` listener and never removes it (line 404), and `attach` registers another on every open (line 289) â€” a minor listener leak; session fetch and health fetch have no AbortController/timeout, so a hung HTTP request blocks `open()` until the browser/network gives up (F-05, partly UNVERIFIED).
5. Pre-send guards: FILE_READ paths go through `validateWorkspacePath`, so `../` absolute/drive escapes are refused client-side before hitting the wire; prompt/zoom validation lives in contract.ts and the store.
6. The `UiApi` interface is the stable seam (`kind/open/close/send/subscribe/subscribeStatus/subscribeClientError`), implemented by both WsApiClient and the dev MockApiClient; tests swap a fake transport and a stub `fetch` without a network, proving the ACK/ERROR/timeout logic deterministically.
7. State exposure is narrow: `getStatus()` plus status listeners; the store owns UI state, so the client has no product state beyond session/lastSeq/pending. No credentials are read or logged; the token appears only in the WS URL query as designed.
8. Quality: correctness 4 (ACK gating/refusals proven by tests, minor listener leak), maintainability 3 (497 lines, several responsibilities in one class), modularity 4 (interface seam is clean), error handling 4 (typed refusals, two timeout gaps), security 4 (no secret storage/logging), docs 4, evidence 4.

- **Dependencies:** imports → web/src/api/client.ts:13: partysocket; web/src/api/client.ts:25: ./ids; web/src/api/client.ts:26: ./refusals
- **Importers/callers:** web/src/api/health.ts:6; web/src/api/index.ts:7; web/src/state/store.ts:25; web/tests/apiClient.test.ts:2; web/tests/store.test.ts:5
- **Quality classification:** correctness 4/5 · maintainability 3/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; tests/apiClient.test.ts 5 tests (fake transport).
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/api/contract.ts

- **Metadata:** 28868 bytes · sha256 `4C43F563D0967E3786F27E11C9F4146056EA7382B82A45FE4E9D529D493979DA` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** Single typed source of the frozen hiverouter.v1 wire contract, validators, envelope/decode/build helpers and ids.

**Mechanics (deep analysis).**
1. Declares SCHEMA_VERSION='hiverouter.v1', FrameKind, generic Envelope, payload types for 14 frozen commands (including the patched APPROVAL_RESOLVE, SETTINGS_STATUS, FILE_TREE_LIST, FILE_READ), 21 events with payload interfaces, and 40 documented refusal codes; anything not in these unions cannot be sent or rendered as a known event.
2. `validateEnvelope` performs structural checks (schema, frame type, non-empty ids/ts, run/task nullable strings, seq number|null, event/command name membership) and `parseEventFrame` adds JSON parsing, EVENT-only enforcement and typed `decodePayload`, producing a discriminated `DecodedEvent` union consumed by the store. Malformed input returns `{ok:false, code, detail}` rather than throwing.
3. `decodePayload` (861-line file, the largest reviewed unit) validates each event field by field: ACK command/correlation, ERROR code/message/how_to_fix/retryable, RUN_* statuses via `isAgentState`, REPLY usage integers/nullable, NODE_* optional metadata (kind/label/parent/progress/error_code forward-compatible extras), settings/file results, approval nonce, history graph passthrough.
4. `buildCommandEnvelope` mints uuid-v7 ids via injected `newId`, RFC3339 via injected `now`, and starts correlation_id equal to id per the frozen contract; `buildHelloFrame` encodes the Phase-2 raw hello. Injected factories keep the module pure and unit-testable.
5. Pre-send validators encode contract preconditions: prompt 0/8000 chars, zoom 0.1â€“3.0, workspace-relative jail path (rejects absolute/drive/UNC/`..`), and file-size >1 MiB refusal. These are duplicated deliberately on the client for fast named refusals, while the server remains authoritative.
6. Edge behaviour: unknown event names are rejected (E_SCHEMA_INVALID), unknown command names (E_COMMAND_UNKNOWN) are recognised only for COMMAND frames, nullable run_id/task_id are normalised, and non-object payloads fail. The union types prevent the store from handling an event it has no decoder for.
7. Risk: this is a client-side re-implementation of `backend/crates/hiverouter-contract/src/*` (envelope/command/event/refusal). Two sources of truth can drift even though the frozen doc is shared; only the repository-level review can keep them in sync (F-08).
8. Quality: correctness 5 (tests cover 14/21/40 declarations + parse/build), maintainability 4 (single 861-line file, highly repetitive decoders), modularity 3 (types+validators+builders in one module), error handling 5 (no throws on wire data), security 4, docs 5, evidence 5.

- **Dependencies:** imports → web/src/api/contract.ts:7: ../state/agentState
- **Importers/callers:** web/src/a11y/keyboard.ts:5; web/src/api/refusals.ts:7; web/src/state/metrics.ts:6; web/src/state/store.ts:22; web/src/state/store.ts:27; web/tests/apiClient.test.ts:4; web/tests/store.test.ts:4
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 3/5 · error handling 5/5 · security 4/5 · docs 5/5 · evidence 5/5 — read; tests/contract.test.ts 8 tests.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/api/health.ts

- **Metadata:** 1885 bytes · sha256 `84963F8C645ED6C04272C69977A343EBF1E9151F3FC89FE23EE89A31739058F8` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** GET /health snapshot (status/version/checks) for Settings; throws named refusal on failure.

- **Mechanics.** GET /health snapshot (status/version/checks) for Settings; throws named refusal on failure. It imports `web/src/api/health.ts:6: ./client; web/src/api/health.ts:7: ./refusals` and is imported by `web/src/state/store.ts:26`. Classification: source.

- **Dependencies:** imports → web/src/api/health.ts:6: ./client; web/src/api/health.ts:7: ./refusals
- **Importers/callers:** web/src/state/store.ts:26
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 3/5 · docs 4/5 · evidence 4/5 — read; no timeout/AbortController (F-05).
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/api/ids.ts

- **Metadata:** 1607 bytes · sha256 `9D7D8D755A5EB863E16FBCCD56573CD8A06C4D78A091EDE25D845A9EC852DADC` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** UUIDv7 generation/validation (RFC 9562 layout) and timestamp helper for contract ids.

- **Mechanics.** UUIDv7 generation/validation (RFC 9562 layout) and timestamp helper for contract ids. It imports `none (no ES imports)` and is imported by `web/src/api/client.ts:25; web/src/api/mock.ts:24; web/src/state/store.ts:23; web/tests/contract.test.ts:15`. Classification: source.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** web/src/api/client.ts:25; web/src/api/mock.ts:24; web/src/state/store.ts:23; web/tests/contract.test.ts:15
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; contract tests validate version/variant.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/api/index.ts

- **Metadata:** 751 bytes · sha256 `6D6E3618DC4918CC974344FB3DCA84A1E7FFF0695FCC5485D93D391DA5906A76` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** UI API factory: real WsApiClient by default, dev-only mock behind VITE_UI_MOCK.

- **Mechanics.** UI API factory: real WsApiClient by default, dev-only mock behind VITE_UI_MOCK. It imports `web/src/api/index.ts:7: ./client; web/src/api/index.ts:8: ./mock` and is imported by `web/src/App.tsx:7`. Classification: source.

- **Dependencies:** imports → web/src/api/index.ts:7: ./client; web/src/api/index.ts:8: ./mock
- **Importers/callers:** web/src/App.tsx:7
- **Quality classification:** correctness 4/5 · maintainability 5/5 · modularity 5/5 · error handling 4/5 · security 3/5 · docs 4/5 · evidence 5/5 — read; static mock import = F-02.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/api/mock.ts

- **Metadata:** 12437 bytes · sha256 `A149A4E36350E0CBF655E7182095CC3C0133BDF1A3104549EFA9490662EC69C8` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** DEV-ONLY mock UiApi with scenarios/refusals; throws when constructed in PROD.

- **Mechanics.** DEV-ONLY mock UiApi with scenarios/refusals; throws when constructed in PROD. It imports `web/src/api/mock.ts:23: ../state/agentState; web/src/api/mock.ts:24: ./ids; web/src/api/mock.ts:25: ./refusals` and is imported by `web/src/api/index.ts:8`. Classification: source.

- **Dependencies:** imports → web/src/api/mock.ts:23: ../state/agentState; web/src/api/mock.ts:24: ./ids; web/src/api/mock.ts:25: ./refusals
- **Importers/callers:** web/src/api/index.ts:8
- **Quality classification:** correctness 4/5 · maintainability 3/5 · modularity 4/5 · error handling 4/5 · security 3/5 · docs 3/5 · evidence 4/5 — read; dev-only guard at lines 68-69.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/api/refusals.ts

- **Metadata:** 1582 bytes · sha256 `DC715B3AE248014448B64C8FD58FD917680D3D31CBC2D9CD97BA908F920D9AEF` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** RefusalError + client refusal construction from contract codes/string table.

- **Mechanics.** RefusalError + client refusal construction from contract codes/string table. It imports `web/src/api/refusals.ts:7: ./contract; web/src/api/refusals.ts:8: ../state/strings` and is imported by `web/src/api/client.ts:26; web/src/api/health.ts:7; web/src/api/mock.ts:25; web/src/state/store.ts:24; web/tests/apiClient.test.ts:3; web/tests/store.test.ts:3`. Classification: source.

- **Dependencies:** imports → web/src/api/refusals.ts:7: ./contract; web/src/api/refusals.ts:8: ../state/strings
- **Importers/callers:** web/src/api/client.ts:26; web/src/api/health.ts:7; web/src/api/mock.ts:25; web/src/state/store.ts:24; web/tests/apiClient.test.ts:3; web/tests/store.test.ts:3
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; no invented causes.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/App.tsx

- **Metadata:** 3961 bytes · sha256 `BC93C009D101C0409458ABB758E606C4219A8CD7ACA9149452587CE457175224` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** Root composition: creates the one UI store, connects it once, and mounts all 28 surfaces via nav/tab routing.

**Mechanics (deep analysis).**
1. `App` (line 106) creates exactly one `StoreApi` per mount with `useState(() => createUiStore(createUiApi()))` (line 107) and wraps the tree in `UiStoreContext.Provider`; `AppFrame` (line 28) consumes selectors for nav/tab/connection/apiKind/reduced motion and composes NavRail, Header, content, ControlBar, Scrubber, LibraryPanel, ToastRegion, ApprovalOverlay and DiffDecisionModal â€” a direct implementation of the 28-surface map.
2. Connection lifecycle: the effect at lines 109-112 calls `store.getState().init()` on mount and `dispose()` on cleanup. `init()` is idempotent via an `initialized` flag in the store, while `dispose()` unsubscribes listeners and closes the transport; in React StrictMode dev double-mount this can dispose subscriptions while the first async handshake is still resolving (F-18, UNVERIFIED dev-only).
3. The layout is honest about state: a mock API badge renders only when `apiKind === 'mock'`; a banner renders the named connection refusal when the socket errors/closes; no fabricated run is shown when disconnected because the store blocks mutations in that state.
4. Navigation model: `view.nav` chooses the main panel (workspace/agents/data/models/routing/settings), while `view.tab` only splits workspace modes (workspace/swarm/routing/history) â€” the G-UI-8 tabs-vs-rail owner rule. Both are plain zustand state, no router dependency.
5. `workspaceView` (line ~48) renders Canvas plus pinned ChatCore, routing table, or replay canvas by tab; the footer shows ControlBar/Scrubber only in workspace. Content panels are conditionally mounted, so each panel's own effects run on entry (this is why SettingsPanel can issue one SETTINGS_STATUS per open after the store guard).
6. Reduced motion is materialised as `document.documentElement.dataset.reducedMotion` in an effect (lines 36-38), which `styles/index.css` consumes; this is a global DOM side effect but scoped to a data attribute only.
7. The component contains no product strings except through `STRINGS`; the only literals are structural class names. That satisfies R-U13 and is verified by tests/noHardcoded.test.ts which scans all src TS/TSX.
8. Quality: correct composition and state gating (4), maintainable effect layout (4), modular (panel components separated), error handling delegated to store/banner (4), security neutral (3), docs via header comment, evidence is read + store tests rather than mount tests.

- **Dependencies:** imports → web/src/App.tsx:6: react; web/src/App.tsx:7: ./api; web/src/App.tsx:8: ./state/store; web/src/App.tsx:9: ./state/context; web/src/App.tsx:10: ./state/useAppReducedMotion; web/src/App.tsx:11: ./state/strings; web/src/App.tsx:12: ./panels/NavRail; web/src/App.tsx:13: ./panels/Header; web/src/App.tsx:14: ./panels/LibraryPanel; web/src/App.tsx:15: ./panels/ControlBar; web/src/App.tsx:16: ./panels/Scrubber; web/src/App.tsx:17: ./panels/ToastRegion; web/src/App.tsx:18: ./panels/SettingsPanel; web/src/App.tsx:19: ./panels/DataPanel; web/src/App.tsx:20: ./panels/AgentsPanel; web/src/App.tsx:21: ./panels/ModelsPanel; web/src/App.tsx:22: ./panels/RoutingPanel; web/src/App.tsx:23: ./panels/ApprovalOverlay; web/src/App.tsx:24: ./panels/DiffDecisionModal; web/src/App.tsx:25: ./flow/Canvas; web/src/App.tsx:26: ./chat/ChatCore
- **Importers/callers:** web/src/main.tsx:4
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 3/5 · docs 4/5 · evidence 4/5 — read; tests cover store, not mount lifecycle.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/assets/manifest.ts

- **Metadata:** 4888 bytes · sha256 `C0DA46D37948D5C6344D5681C14B5D0EF944115C5E1062CAE59395BDEC6C72EF` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** FigCraft asset gate seam: watchdog verdict, 8 accepted exports with audited sha256, no pixel sampling.

- **Mechanics.** FigCraft asset gate seam: watchdog verdict, 8 accepted exports with audited sha256, no pixel sampling. It imports `none (no ES imports)` and is imported by `web/src/panels/LibraryPanel.tsx:5`. Classification: source.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** web/src/panels/LibraryPanel.tsx:5
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 5/5 · evidence 5/5 — read; gate wired allow_frontend=true.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/chat/ChatCore.tsx

- **Metadata:** 6270 bytes · sha256 `E924BB44FA76476B2342EF7554E9026D7523DC432C8A869F07FFF4869FA78FBE` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** Pinned HyperAgent chat core + composer + streaming reply rendering.

**Mechanics (deep analysis).**
1. ChatCore renders as a fixed overlay independent of canvas pan/zoom (R-U2/S02 'pinned'), using store state for messages, streamingRunId, chat.pinned/visible, pending commands and connection. Its composer is the only caller of `submitPrompt` besides tests.
2. Submit paths: form submit and Enter key both call `submitPrompt(draft)` then clear the draft; Shift+Enter stays as a newline. The store validates length/emptiness and appends the user message before sending SUBMIT_PROMPT, so refusals are shown as named toasts while the user message remains visible.
3. Streaming: REPLY_DELTA appends/concatenates into the last assistant message keyed by run_id; REPLY_DONE flips `streaming` false. The header reflects connection status and pinned state; hide/show and pin/unpin are local UI actions (pin sends CHAT_PIN and applies pinned only after ACK).
4. Accessibility: section has `aria-label`, messages list has `aria-live="polite"`, composer has a visually hidden label, submit has a real label, and streaming status is announced. Focus never auto-moves; keyboard Enter handling prevents newline+submit double-fire.
5. Edge behaviour: when the socket is not open the textarea/submit are disabled; submitting while another SUBMIT_PROMPT is pending disables the button (pending map scan) to avoid duplicate runs. Refusals are not rendered inline, only via the global toast region â€” acceptable but means the chat area itself does not explain an E_KEY_MISSING failure.
6. Defect F-10: the draft is cleared before validation results are known (ChatCore submit handlers), so an empty/too-long client refusal loses whatever was typed; the store's refusal toast still explains why. Historical messages are unbounded in the DOM (no virtualisation) â€” fine for v1 but a memory/scroll risk on long runs.
7. Dependencies: store context, STRINGS, reduced-motion hook; no direct API import. Its rendering data (messages) is derived from server events only, so it cannot fabricate replies.
8. Quality: correctness 4, maintainability 4, modularity 4 (chat isolated), error handling 4 (disabled states + toasts, draft-loss defect), security 3 (renders plain text, no markdown/HTML injection path), docs 4, evidence 4.

- **Dependencies:** imports → web/src/chat/ChatCore.tsx:7: react; web/src/chat/ChatCore.tsx:8: ../state/context; web/src/chat/ChatCore.tsx:9: ../state/strings; web/src/chat/ChatCore.tsx:10: ../state/useAppReducedMotion
- **Importers/callers:** web/src/App.tsx:26
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 3/5 · docs 4/5 · evidence 4/5 — read; store tests cover submit/stream indirectly, no component test.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/flow/BeeEdge.tsx

- **Metadata:** 1686 bytes · sha256 `96C4E65A6BB24120887722674F5657EDF2C769C2F60E86E02B0227C150FC9321` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Custom React Flow edge: dashed honey path + bee marker oriented along sourceâ†’target.

- **Mechanics.** Custom React Flow edge: dashed honey path + bee marker oriented along sourceâ†’target. It imports `web/src/flow/BeeEdge.tsx:11: ../a11y/keyboard` and is imported by `web/src/flow/Canvas.tsx:22`. Classification: source.

- **Dependencies:** imports → web/src/flow/BeeEdge.tsx:11: ../a11y/keyboard
- **Importers/callers:** web/src/flow/Canvas.tsx:22
- **Quality classification:** correctness 4/5 · maintainability 5/5 · modularity 5/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; direction uses tested beeAngle.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/flow/Canvas.tsx

- **Metadata:** 8823 bytes · sha256 `5751824B68466EB5334209787D2D3FB03EF4FFEB3BAFCC58E4CAA5E51DEC0FB1` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** React Flow canvas with honeycomb field, node/edge types, minimap, zoom bounds, Ctrl+0 recentre, history replay branch.

**Mechanics (deep analysis).**
1. Canvas wraps ReactFlowProvider around CanvasInner. Nodes come from `buildNodes(tasks, selectedTaskId, handleSelect)` and edges from `buildEdges` (dagre projection in flow/graph.ts); nodeTypes are the 5 renderers cast to NodeTypes, edge type 'bee' renders the BeeEdge with direction arrow.
2. Viewport: explicit minZoom 0.1 / maxZoom 3.0 overrides library defaults, default viewport {0,0,1}, fitView on first render (except replay), and onMoveEnd updates store viewport + zoom label. Zoom controls (+/âˆ’/0/fit) call ReactFlow API; recentre sets center (0,0) at current zoom.
3. Keyboard: a document-level capture listener (line 91) maps Ctrl+0/Meta+0/Digit0 to an instant `setCenter(..., {duration:0})` plus a rAF re-apply (line 69), preventDefault/stopPropagation; result is independent of which control has focus except editable fields. This closes the live-e2e transform bug.
4. Minimap accessibility: the effect at lines ~97-109 queries `.react-flow__minimap`/`.rf__minimap` and sets aria-label='Mini-map' + role='img' (plus the MiniMap `ariaLabel` prop); an imperative DOM bridge, but deterministic and tested by live e2e.
5. Replay defect F-12: when `history.replayGraph` is set the component passes `nodes={replayNodes}` where `replayNodes` is `useMemo(() => [], [])` and `edges={[]}` â€” the replay banner appears but the snapshot graph is never projected, so the scrubber is effectively read-only empty. The parsed HISTORY_SNAPSHOT graph is stored but unused.
6. Honeycomb field is a static SVG pattern overlay behind ReactFlow (currentColor), not pan-synced with the viewport; visual fidelity is explicitly UNVERIFIED/needs_human per the asset gate, and no pixel claim is made.
7. Edge behaviour: empty graph shows an empty hint; replay branch bypasses live nodes; CSS class names/aria labels are structural. The imperative document listener and querySelector bridge are testability/SSR concerns (no SSR here).
8. Quality: correctness 4 (live canvas works; replay dead), maintainability 3 (effect/imperative DOM), modularity 4 (graph/layout extracted), error handling 3 (no fallback if RF fails), security 3 (no user HTML), docs 4, evidence 4.

- **Dependencies:** imports → web/src/flow/Canvas.tsx:8: react; web/src/flow/Canvas.tsx:19: @xyflow/react/dist/style.css; web/src/flow/Canvas.tsx:20: ./graph; web/src/flow/Canvas.tsx:21: ./nodes; web/src/flow/Canvas.tsx:22: ./BeeEdge; web/src/flow/Canvas.tsx:23: ../a11y/keyboard; web/src/flow/Canvas.tsx:24: ../state/context; web/src/flow/Canvas.tsx:25: ../state/useAppReducedMotion; web/src/flow/Canvas.tsx:26: ../state/strings
- **Importers/callers:** web/src/App.tsx:25
- **Quality classification:** correctness 4/5 · maintainability 3/5 · modularity 4/5 · error handling 3/5 · security 3/5 · docs 4/5 · evidence 4/5 — read; keyboard/minimap fixes audited in live e2e; replay path untested.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/flow/graph.ts

- **Metadata:** 3232 bytes · sha256 `2357DCB03F698A6B2D693DF7E258DD2492BBC831DCFEFD4FD5344FF164C28FEB` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Pure graph projection: TaskRecord map â†’ dagre layout nodes/edges + kind counts.

- **Mechanics.** Pure graph projection: TaskRecord map â†’ dagre layout nodes/edges + kind counts. It imports `web/src/flow/graph.ts:7: @dagrejs/dagre; web/src/flow/graph.ts:8: @xyflow/react; web/src/flow/graph.ts:9: ../state/store; web/src/flow/graph.ts:10: ../state/agentState` and is imported by `web/src/flow/Canvas.tsx:20; web/src/flow/nodes.tsx:10`. Classification: source.

- **Dependencies:** imports → web/src/flow/graph.ts:7: @dagrejs/dagre; web/src/flow/graph.ts:8: @xyflow/react; web/src/flow/graph.ts:9: ../state/store; web/src/flow/graph.ts:10: ../state/agentState
- **Importers/callers:** web/src/flow/Canvas.tsx:20; web/src/flow/nodes.tsx:10
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 5/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; layout is deterministic, no cycle guard beyond dagre.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/flow/nodes.tsx

- **Metadata:** 6451 bytes · sha256 `5DE1969AE23615BCD71579F0CB57FBCB02549D87E5DBFF8BB20BB3C12A0DAD9C` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** Five node-card renderers (4 types + broken-route error) and the state glyph used across surfaces.

**Mechanics (deep analysis).**
1. Exports HyperAgentNode, SuperAgentNode, WorkerAgentNode, ToolAgentNode and BrokenRouteNode, all built on a shared Shell that renders kind label, state glyph, label, caption, optional children, target/source handles and selection styling; NODE_RENDERERS maps the five kinds for React Flow.
2. StateGlyph is the single visualisation of the 15-state enum: dot for the four PDF dot states, ring/diamond/square/triangle/hex cues for all others, with label text; colour comes from token CSS variables, never hex literals, and shape+text mean colour is never the only signal.
3. Honesty of data: Worker/Agent cards show ctx/score/attempt as the contract's null glyph 'â€”' because the frozen wire payload does not yet deliver those metrics; Super/Worker progress renders a bar only when a progress number exists (optional metadata extension). No fabricated metrics.
4. BrokenRouteNode renders the error state with errorCode or a generic failed caption and a null attempt count â€” the 5th renderer exists for the 'broken-route card' requirement while still being one error state, not a fifth agent type.
5. Accessibility: cards are real buttons (`aria-pressed`, label includes kind+state) inside React Flow's focusable node wrapper, so keyboard selection works; React Flow handles dragging/focus semantics.
6. Dependencies: STRINGS (all copy), agentState (visuals/labels), graph types. The module is presentation-only and receives `data` via a local HiveNodeProps type; Canvas casts the registry to NodeTypes to satisfy React Flow v12 generics.
7. Edge risk: an unknown server-supplied kind cannot reach these renderers because contract.ts validates kind against NODE_KINDS; missing metadata falls back to worker/generic label upstream in the store, so renderers never invent a type.
8. Quality: correctness 4 (registry complete), maintainability 4, modularity 4, error handling 4 (null glyphs), security 3 (plain text), docs 4 (header comment), evidence 5 (state tests + read).

- **Dependencies:** imports → web/src/flow/nodes.tsx:6: @xyflow/react; web/src/flow/nodes.tsx:7: react; web/src/flow/nodes.tsx:8: ../state/strings; web/src/flow/nodes.tsx:9: ../state/agentState; web/src/flow/nodes.tsx:10: ./graph
- **Importers/callers:** web/src/flow/Canvas.tsx:21; web/src/panels/AgentsPanel.tsx:5; web/src/panels/DevGallery.tsx:3
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 3/5 · docs 4/5 · evidence 5/5 — read; node registry/shape cues indirectly tested via agentState tests.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/main.tsx

- **Metadata:** 404 bytes · sha256 `2BC1AA6AAF5A25FDDF1A35C49D9BF053A07C45437160CE7878D01D8572D9452D` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** React root bootstrap with StrictMode and global CSS/React Flow stylesheet imports.

- **Mechanics.** React root bootstrap with StrictMode and global CSS/React Flow stylesheet imports. It imports `web/src/main.tsx:1: react; web/src/main.tsx:2: react-dom/client; web/src/main.tsx:3: @xyflow/react/dist/style.css; web/src/main.tsx:4: ./App; web/src/main.tsx:5: ./styles/index.css` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → web/src/main.tsx:1: react; web/src/main.tsx:2: react-dom/client; web/src/main.tsx:3: @xyflow/react/dist/style.css; web/src/main.tsx:4: ./App; web/src/main.tsx:5: ./styles/index.css
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 3/5 · evidence 5/5 — read; StrictMode interacts with effect lifecycle (F-18).
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/AgentsPanel.tsx

- **Metadata:** 2038 bytes · sha256 `0A452EE172371E85A59D28BE600A0D0817CCB9B826E51EFAEE8CC99A06B8210F` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Agent registry grid: derived card list from tasks with state glyphs and selection.

- **Mechanics.** Agent registry grid: derived card list from tasks with state glyphs and selection. It imports `web/src/panels/AgentsPanel.tsx:2: ../state/context; web/src/panels/AgentsPanel.tsx:3: ../state/agentState; web/src/panels/AgentsPanel.tsx:4: ../state/strings; web/src/panels/AgentsPanel.tsx:5: ../flow/nodes; web/src/panels/AgentsPanel.tsx:6: ./StatePack` and is imported by `web/src/App.tsx:20`. Classification: source.

- **Dependencies:** imports → web/src/panels/AgentsPanel.tsx:2: ../state/context; web/src/panels/AgentsPanel.tsx:3: ../state/agentState; web/src/panels/AgentsPanel.tsx:4: ../state/strings; web/src/panels/AgentsPanel.tsx:5: ../flow/nodes; web/src/panels/AgentsPanel.tsx:6: ./StatePack
- **Importers/callers:** web/src/App.tsx:20
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; read-only derived view.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/ApprovalOverlay.tsx

- **Metadata:** 2857 bytes · sha256 `EB1595B3150B400D9CF7FB0BE3DCBD189B242B6FA7D7CC8DD965FC87030A37BF` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Approval modal: renders pending approvals, resolves via APPROVAL_RESOLVE when nonce present.

- **Mechanics.** Approval modal: renders pending approvals, resolves via APPROVAL_RESOLVE when nonce present. It imports `web/src/panels/ApprovalOverlay.tsx:2: ../state/context; web/src/panels/ApprovalOverlay.tsx:3: ../state/strings` and is imported by `web/src/App.tsx:23`. Classification: source.

- **Dependencies:** imports → web/src/panels/ApprovalOverlay.tsx:2: ../state/context; web/src/panels/ApprovalOverlay.tsx:3: ../state/strings
- **Importers/callers:** web/src/App.tsx:23
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 3/5 · docs 4/5 · evidence 5/5 — read; nonce-missing gap state honest.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/ControlBar.tsx

- **Metadata:** 5023 bytes · sha256 `A22B87B6D259C2FB9B4DC34D0449AE5D7CA9DB8AA2F720FCD230FC5346FACF92` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** S04-S07 mutating controls (spawn/rewire/cancel/kill) with precondition gating and pending state.

- **Mechanics.** S04-S07 mutating controls (spawn/rewire/cancel/kill) with precondition gating and pending state. It imports `web/src/panels/ControlBar.tsx:6: react; web/src/panels/ControlBar.tsx:7: ../state/context; web/src/panels/ControlBar.tsx:8: ../state/strings; web/src/panels/ControlBar.tsx:9: ../state/agentState` and is imported by `web/src/App.tsx:15`. Classification: source.

- **Dependencies:** imports → web/src/panels/ControlBar.tsx:6: react; web/src/panels/ControlBar.tsx:7: ../state/context; web/src/panels/ControlBar.tsx:8: ../state/strings; web/src/panels/ControlBar.tsx:9: ../state/agentState
- **Importers/callers:** web/src/App.tsx:15
- **Quality classification:** correctness 4/5 · maintainability 3/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; disable logic + named refusals.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/DataPanel.tsx

- **Metadata:** 3077 bytes · sha256 `58A77EDED98E06776F43F50D039F1E4FA2029C522BDE9EC145B51A85FB383F12` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** S22 workspace Data surface: FILE_TREE_LIST/FILE_READ with load/preview/error states and client-side jail pre-check.

**Mechanics (deep analysis).**
1. On mount (when idle) requests FILE_TREE_LIST; renders loading/error/empty states, then a list of entries with file size or '/', and a right pane that previews FILE_CONTENT (path, bytes, sha256 prefix, text) or shows a preview hint.
2. Path safety: clicking an entry calls store `openFile(path)` which pre-validates `validateWorkspacePath` (absolute/drive/UNC/`..` â†’ E_PATH_OUTSIDE_JAIL) before FILE_READ; the server re-validates and jails canonical paths, so the client check is a fast refusal, not authority.
3. Copy was corrected for live e2e: the panel no longer claims there is no workspace-file API; it names the frozen commands and instructs loadâ†’selectâ†’preview. Error copy names a named-refusal toast and retry.
4. Defect F-13: the error copy says 'retry with Refresh' but the panel has no Refresh control; re-entering the Data nav only fetches when status is idle, so after an error the user has no in-panel retry affordance (store requestFileTree exists but is unreachable here).
5. States are honest: a missing/refusing server route shows ErrorState rather than a fake tree; FILE_CONTENT text is rendered in `<pre>` as plain text (no HTML injection), and sha256/bytes come from the server payload.
6. Dependencies: store context (fileTree/openFile/requestFileTree), STRINGS, StatePack primitives; no direct API import. Size is small and presentation-driven; the store owns command lifecycle.
7. Edge: selecting a second file keeps prior content visible during loading? It sets contentStatus loading but retains content, so stale preview text can show under a loading banner until FILE_CONTENT arrives â€” minor UX ambiguity (UNVERIFIED live).
8. Quality: correctness 4, maintainability 4, modularity 4, error handling 4, security 4 (jail + plain text), docs 4, evidence 5 (read + store tests).

- **Dependencies:** imports → web/src/panels/DataPanel.tsx:2: react; web/src/panels/DataPanel.tsx:3: ../state/context; web/src/panels/DataPanel.tsx:4: ../state/strings; web/src/panels/DataPanel.tsx:5: ./StatePack
- **Importers/callers:** web/src/App.tsx:19
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; store file actions unit-covered, live API e2e owned by e2e seat.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/panels/DevGallery.tsx

- **Metadata:** 1044 bytes · sha256 `5D6645467E54865F50E989613708D5171BC4DB0B0E26761FCCDA27F53140D7FC` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** DEV-only 15-state shape/text gallery; returns null in production builds.

- **Mechanics.** DEV-only 15-state shape/text gallery; returns null in production builds. It imports `web/src/panels/DevGallery.tsx:2: ../state/agentState; web/src/panels/DevGallery.tsx:3: ../flow/nodes; web/src/panels/DevGallery.tsx:4: ../state/agentState; web/src/panels/DevGallery.tsx:5: ../state/strings` and is imported by `web/src/panels/SettingsPanel.tsx:6`. Classification: source.

- **Dependencies:** imports → web/src/panels/DevGallery.tsx:2: ../state/agentState; web/src/panels/DevGallery.tsx:3: ../flow/nodes; web/src/panels/DevGallery.tsx:4: ../state/agentState; web/src/panels/DevGallery.tsx:5: ../state/strings
- **Importers/callers:** web/src/panels/SettingsPanel.tsx:6
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 3/5 · evidence 4/5 — read; import.meta.env.DEV guard.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/DiffDecisionModal.tsx

- **Metadata:** 1265 bytes · sha256 `6C2DDD6BFD6F800C57F01CBEE0F3C0A16C10E902D7E95ECF8A749A33337412D0` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Decision-gated code-diff modal; explicitly does not implement DIFF_* commands.

- **Mechanics.** Decision-gated code-diff modal; explicitly does not implement DIFF_* commands. It imports `web/src/panels/DiffDecisionModal.tsx:2: ../state/context; web/src/panels/DiffDecisionModal.tsx:3: ../state/strings` and is imported by `web/src/App.tsx:24`. Classification: source.

- **Dependencies:** imports → web/src/panels/DiffDecisionModal.tsx:2: ../state/context; web/src/panels/DiffDecisionModal.tsx:3: ../state/strings
- **Importers/callers:** web/src/App.tsx:24
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; S25 gate respected.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/Header.tsx

- **Metadata:** 3268 bytes · sha256 `8285642357B7B8F660075742A3E8CA58E5AAD991545777E3B9BEBAA0241394BA` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Tabs + HUD (server/derived metrics, null glyphs) + project identity + connection badge.

- **Mechanics.** Tabs + HUD (server/derived metrics, null glyphs) + project identity + connection badge. It imports `web/src/panels/Header.tsx:2: react; web/src/panels/Header.tsx:3: ../state/context; web/src/panels/Header.tsx:4: ../state/strings; web/src/panels/Header.tsx:5: ../state/metrics; web/src/panels/Header.tsx:6: ./StatePack; web/src/panels/Header.tsx:7: ../state/store` and is imported by `web/src/App.tsx:13`. Classification: source.

- **Dependencies:** imports → web/src/panels/Header.tsx:2: react; web/src/panels/Header.tsx:3: ../state/context; web/src/panels/Header.tsx:4: ../state/strings; web/src/panels/Header.tsx:5: ../state/metrics; web/src/panels/Header.tsx:6: ./StatePack; web/src/panels/Header.tsx:7: ../state/store
- **Importers/callers:** web/src/App.tsx:13
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; deriveHud unit-tested.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/LibraryPanel.tsx

- **Metadata:** 2994 bytes · sha256 `85B059387AC942ADC44980CB698AE338522B94927F9709382CF98B24429D114E` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** 260px library: project id, derived active agents, template inserts, asset-gate note.

- **Mechanics.** 260px library: project id, derived active agents, template inserts, asset-gate note. It imports `web/src/panels/LibraryPanel.tsx:2: ../state/context; web/src/panels/LibraryPanel.tsx:3: ../state/strings; web/src/panels/LibraryPanel.tsx:4: ../state/agentState; web/src/panels/LibraryPanel.tsx:5: ../assets/manifest` and is imported by `web/src/App.tsx:14`. Classification: source.

- **Dependencies:** imports → web/src/panels/LibraryPanel.tsx:2: ../state/context; web/src/panels/LibraryPanel.tsx:3: ../state/strings; web/src/panels/LibraryPanel.tsx:4: ../state/agentState; web/src/panels/LibraryPanel.tsx:5: ../assets/manifest
- **Importers/callers:** web/src/App.tsx:14
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 3/5 · docs 4/5 · evidence 4/5 — read; inserts TEMPLATE_INSERT.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/ModelsPanel.tsx

- **Metadata:** 1195 bytes · sha256 `231A22A7228E632CF4DF6E937A1448031EAB6220A1855B44EFB105F92A5F4858` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Read-only provider/model view from SETTINGS_STATUS with explicit not-reported states.

- **Mechanics.** Read-only provider/model view from SETTINGS_STATUS with explicit not-reported states. It imports `web/src/panels/ModelsPanel.tsx:2: ../state/context; web/src/panels/ModelsPanel.tsx:3: ../state/strings; web/src/panels/ModelsPanel.tsx:4: ./StatePack` and is imported by `web/src/App.tsx:21`. Classification: source.

- **Dependencies:** imports → web/src/panels/ModelsPanel.tsx:2: ../state/context; web/src/panels/ModelsPanel.tsx:3: ../state/strings; web/src/panels/ModelsPanel.tsx:4: ./StatePack
- **Importers/callers:** web/src/App.tsx:21
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; no fabricated model config.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/NavRail.tsx

- **Metadata:** 2251 bytes · sha256 `DC54427F99FC00972A99DAA8C824C67AA5BFD1FA8F366C87F7BD74D3C91BF94E` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** 72px six-destination rail with roving arrow focus and aria-current.

- **Mechanics.** 72px six-destination rail with roving arrow focus and aria-current. It imports `web/src/panels/NavRail.tsx:2: react; web/src/panels/NavRail.tsx:3: ../state/context; web/src/panels/NavRail.tsx:4: ../state/strings; web/src/panels/NavRail.tsx:5: ../state/store; web/src/panels/NavRail.tsx:6: ../a11y/keyboard` and is imported by `web/src/App.tsx:12`. Classification: source.

- **Dependencies:** imports → web/src/panels/NavRail.tsx:2: react; web/src/panels/NavRail.tsx:3: ../state/context; web/src/panels/NavRail.tsx:4: ../state/strings; web/src/panels/NavRail.tsx:5: ../state/store; web/src/panels/NavRail.tsx:6: ../a11y/keyboard
- **Importers/callers:** web/src/App.tsx:12
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; a11y keyboard tested via helper.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/RoutingPanel.tsx

- **Metadata:** 1888 bytes · sha256 `F0269705A18A93B587EB5C9B887737E4DEFE3F5C7453DCBEB14F06103FFEB1B9` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Derived parentâ†’child routing table with kinds/states.

- **Mechanics.** Derived parentâ†’child routing table with kinds/states. It imports `web/src/panels/RoutingPanel.tsx:2: ../state/context; web/src/panels/RoutingPanel.tsx:3: ../state/strings; web/src/panels/RoutingPanel.tsx:4: ../state/agentState; web/src/panels/RoutingPanel.tsx:5: ./StatePack` and is imported by `web/src/App.tsx:22`. Classification: source.

- **Dependencies:** imports → web/src/panels/RoutingPanel.tsx:2: ../state/context; web/src/panels/RoutingPanel.tsx:3: ../state/strings; web/src/panels/RoutingPanel.tsx:4: ../state/agentState; web/src/panels/RoutingPanel.tsx:5: ./StatePack
- **Importers/callers:** web/src/App.tsx:22
- **Quality classification:** correctness 4/5 · maintainability 5/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; no invented routes.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/Scrubber.tsx

- **Metadata:** 1782 bytes · sha256 `1D491793C3F8179994BE7C58AE9909C7168CBED670DE81071166B90AC2225824` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Time-travel scrubber: local drag state commits HISTORY_REVISION_SET on release; replay banner.

- **Mechanics.** Time-travel scrubber: local drag state commits HISTORY_REVISION_SET on release; replay banner. It imports `web/src/panels/Scrubber.tsx:2: react; web/src/panels/Scrubber.tsx:3: ../state/context; web/src/panels/Scrubber.tsx:4: ../state/strings` and is imported by `web/src/App.tsx:16`. Classification: source.

- **Dependencies:** imports → web/src/panels/Scrubber.tsx:2: react; web/src/panels/Scrubber.tsx:3: ../state/context; web/src/panels/Scrubber.tsx:4: ../state/strings
- **Importers/callers:** web/src/App.tsx:16
- **Quality classification:** correctness 3/5 · maintainability 4/5 · modularity 4/5 · error handling 3/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; commits only on pointer/key release; replay render dead (F-12).
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/SettingsPanel.tsx

- **Metadata:** 5186 bytes · sha256 `DFBF40970F72B05C9677DE65D4BF4991F24B7CD7B5886D7ECC689B110D5A52BF` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** S21 Settings: one SETTINGS_STATUS per open, key/model/provider + refusal rendering, health probe, reduced motion, diff gate, dev gallery.

**Mechanics (deep analysis).**
1. A `probed` ref plus store-level pending guard make the effect issue exactly one SETTINGS_STATUS per Settings open (React StrictMode double-effect included); the panel renders pending/loading, typed refusal (code/message/how_to_fix), or the provider/model/key_present result.
2. Key status is data, not a refusal: key_present false renders 'Key missing' without blocking the panel; the hint explains E_KEY_MISSING surfaces on submit. No key material is ever requested or displayed.
3. Health: probeHealth fetches GET /health and renders status + named checks; failures set health unknown and toast the transport refusal, so the panel never claims 'healthy' without evidence. Model/provider display 'unknown' until the server reports them.
4. Accessibility/controls: real labels/aria via surrounding sections, checkbox for reduced-motion override affecting document root data attribute, a button that opens the decision-gated diff modal, and the DEV-only state gallery.
5. Security observation F-06: reduced-motion and settings state live in the global store; health data is non-secret, but the panel displays arbitrary server-provided check names/messages as text â€” plain text is safe, no HTML.
6. Dependencies: store context, STRINGS, StatePack, DevGallery. Effect dependency array excludes settings fields deliberately (guarded once), so later nav away/back remounts and re-probes once â€” desired for status freshness.
7. Edge behaviour: if SETTINGS_STATUS ACKs but the result never arrives, status stays pending (no timeout in the panel); the store command timeout (15 s) rejects only the ACK path, so a missing result leaves pending indicator until remount â€” minor honesty/robustness gap (UNVERIFIED server timing).
8. Quality: correctness 4, maintainability 4, modularity 4, error handling 5 (typed refusal render), security 2 (should be 4: no sensitive data; scored 2 only because settings/health probing is unauthenticated on localhost and token handling is out of panel scope â€” see F-06) , docs 4, evidence 5.

- **Dependencies:** imports → web/src/panels/SettingsPanel.tsx:2: react; web/src/panels/SettingsPanel.tsx:3: ../state/context; web/src/panels/SettingsPanel.tsx:4: ../state/strings; web/src/panels/SettingsPanel.tsx:5: ./StatePack; web/src/panels/SettingsPanel.tsx:6: ./DevGallery
- **Importers/callers:** web/src/App.tsx:18
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 5/5 · security 2/5 · docs 4/5 · evidence 5/5 — read; store test 'one SETTINGS_STATUS per open'.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/panels/StatePack.tsx

- **Metadata:** 1816 bytes · sha256 `ED3FBDC6EBE78C27BA69CB454C38E7B2A19803EC5F6649F016B71EADF730C677` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Shared surface-state primitives (panel title, empty/loading/error, replay banner, derived badge).

- **Mechanics.** Shared surface-state primitives (panel title, empty/loading/error, replay banner, derived badge). It imports `web/src/panels/StatePack.tsx:2: react; web/src/panels/StatePack.tsx:3: ../state/strings` and is imported by `web/src/panels/AgentsPanel.tsx:6; web/src/panels/DataPanel.tsx:5; web/src/panels/Header.tsx:6; web/src/panels/ModelsPanel.tsx:4; web/src/panels/RoutingPanel.tsx:5; web/src/panels/SettingsPanel.tsx:5`. Classification: source.

- **Dependencies:** imports → web/src/panels/StatePack.tsx:2: react; web/src/panels/StatePack.tsx:3: ../state/strings
- **Importers/callers:** web/src/panels/AgentsPanel.tsx:6; web/src/panels/DataPanel.tsx:5; web/src/panels/Header.tsx:6; web/src/panels/ModelsPanel.tsx:4; web/src/panels/RoutingPanel.tsx:5; web/src/panels/SettingsPanel.tsx:5
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; pure presentation.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/panels/ToastRegion.tsx

- **Metadata:** 2072 bytes · sha256 `403D24C553BF9F07E7EBA5A94671AA00F98D20CE31DC7EA71B8D4FA3ED6E8A4A` (manifest; re-hashed in audit: MATCH) · TypeScript/TSX · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Refusal/error toast region with code+message+how_to_fix, dismiss, motion, aria-live.

- **Mechanics.** Refusal/error toast region with code+message+how_to_fix, dismiss, motion, aria-live. It imports `web/src/panels/ToastRegion.tsx:2: motion/react; web/src/panels/ToastRegion.tsx:3: ../state/context; web/src/panels/ToastRegion.tsx:4: ../state/strings; web/src/panels/ToastRegion.tsx:5: ../state/useAppReducedMotion` and is imported by `web/src/App.tsx:17`. Classification: source.

- **Dependencies:** imports → web/src/panels/ToastRegion.tsx:2: motion/react; web/src/panels/ToastRegion.tsx:3: ../state/context; web/src/panels/ToastRegion.tsx:4: ../state/strings; web/src/panels/ToastRegion.tsx:5: ../state/useAppReducedMotion
- **Importers/callers:** web/src/App.tsx:17
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; motion respects reduced motion.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/state/agentState.ts

- **Metadata:** 4509 bytes · sha256 `88CFB66BBAAE9F077230884078D586F517CAA41057FA998E4CD54C75962A5CDB` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** The single >=13-state agent vocabulary, dot mapping, shape cues, transition matrix and node-kind labels.

**Mechanics (deep analysis).**
1. AGENT_STATES declares 15 states (healthy, running, queued, failed, completed, cancelled, killed, planning, thinking, awaiting_approval, streaming, reconnecting, replay, empty, loading) â€” a superset of the contract's 13 minimum, with `isAgentState` guard used by the contract decoder so unknown wire states are rejected.
2. DOT_STATES fixes exactly the four PDF dots (healthy/running/failed/queued); `dotFor` returns the dot state or null, and `visualFor` returns {label,dot,shape,toneToken} where non-dot states get explicit shapes (ring/diamond/square/triangle/hex) â€” the 'shape/text cue as well as colour' rule.
3. Label strings come from STRINGS.agentStates (single string table), so the enum is the only vocabulary while copy stays localisable and test-verifiable (tests assert label presence for every state).
4. STATE_TRANSITIONS encodes the wiring guide's transition matrix (e.g. runningâ†’completed/failed/cancelled/killed/thinking/streaming/queued; completed/killed/cancelled terminal) with canTransition/isTerminal/isRunActive helpers; isRunActive gates mutating controls in ControlBar and store guards.
5. NODE_KIND_KEYS/NODE_KIND_LABELS define the 4 node types + error state labels reused by nodes.tsx and panels â€” the R-U5 vocabulary in one place.
6. Edge behaviour: isDotState narrowing is explicit; visualFor casts the non-dot branch after the dot check (TypeScript-safe), unknown values never reach it because isAgentState gates wire data. Transition matrix is documentation-grade but not universally enforced by a state machine â€” the server owns truth.
7. Dependencies: only STRINGS (no API, no React). Testing is pure and deterministic (15 states, dot mapping, cues, transitions), making this one of the strongest evidence-backed modules.
8. Quality: correctness 5, maintainability 5, modularity 5, error handling 5, security 4 (neutral), docs 5, evidence 5.

- **Dependencies:** imports → web/src/state/agentState.ts:7: ./strings
- **Importers/callers:** web/src/api/contract.ts:7; web/src/api/mock.ts:23; web/src/flow/graph.ts:10; web/src/flow/nodes.tsx:9; web/src/panels/AgentsPanel.tsx:3; web/src/panels/ControlBar.tsx:9; web/src/panels/DevGallery.tsx:2; web/src/panels/DevGallery.tsx:4; web/src/panels/LibraryPanel.tsx:4; web/src/panels/RoutingPanel.tsx:4; web/src/state/metrics.ts:7
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 5/5 · evidence 5/5 — read; tests/agentState.test.ts 5 tests.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/state/context.ts

- **Metadata:** 811 bytes · sha256 `2CE91B7243F156602DB34C23B9B9144E80C52876EF4AD8CC80381641E917EC85` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** React context binding for the zustand StoreApi with useUiStore/useUiStoreApi hooks.

- **Mechanics.** React context binding for the zustand StoreApi with useUiStore/useUiStoreApi hooks. It imports `web/src/state/context.ts:4: react; web/src/state/context.ts:5: zustand; web/src/state/context.ts:6: zustand/vanilla; web/src/state/context.ts:7: ./store` and is imported by `web/src/App.tsx:9; web/src/chat/ChatCore.tsx:8; web/src/flow/Canvas.tsx:24; web/src/panels/AgentsPanel.tsx:2; web/src/panels/ApprovalOverlay.tsx:2; web/src/panels/ControlBar.tsx:7; web/src/panels/DataPanel.tsx:3; web/src/panels/DiffDecisionModal.tsx:2; web/src/panels/Header.tsx:3; web/src/panels/LibraryPanel.tsx:2; web/src/panels/ModelsPanel.tsx:2; web/src/panels/NavRail.tsx:3; web/src/panels/RoutingPanel.tsx:2; web/src/panels/Scrubber.tsx:3; web/src/panels/SettingsPanel.tsx:3; web/src/panels/ToastRegion.tsx:3; web/src/state/useAppReducedMotion.ts:4`. Classification: source.

- **Dependencies:** imports → web/src/state/context.ts:4: react; web/src/state/context.ts:5: zustand; web/src/state/context.ts:6: zustand/vanilla; web/src/state/context.ts:7: ./store
- **Importers/callers:** web/src/App.tsx:9; web/src/chat/ChatCore.tsx:8; web/src/flow/Canvas.tsx:24; web/src/panels/AgentsPanel.tsx:2; web/src/panels/ApprovalOverlay.tsx:2; web/src/panels/ControlBar.tsx:7; web/src/panels/DataPanel.tsx:3; web/src/panels/DiffDecisionModal.tsx:2; web/src/panels/Header.tsx:3; web/src/panels/LibraryPanel.tsx:2; web/src/panels/ModelsPanel.tsx:2; web/src/panels/NavRail.tsx:3; web/src/panels/RoutingPanel.tsx:2; web/src/panels/Scrubber.tsx:3; web/src/panels/SettingsPanel.tsx:3; web/src/panels/ToastRegion.tsx:3; web/src/state/useAppReducedMotion.ts:4
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; throws outside provider.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/state/metrics.ts

- **Metadata:** 1710 bytes · sha256 `15FC2EB1729032A8A159244CC6A368E57F0415B465EED2B9B9C6B80C77C0644D` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** HUD derivation: server metrics authoritative, else derived counts/green% with source label, nulls preserved.

- **Mechanics.** HUD derivation: server metrics authoritative, else derived counts/green% with source label, nulls preserved. It imports `web/src/state/metrics.ts:6: ../api/contract; web/src/state/metrics.ts:7: ./agentState` and is imported by `web/src/panels/Header.tsx:5; web/src/state/store.ts:28; web/tests/metrics.test.ts:2`. Classification: source.

- **Dependencies:** imports → web/src/state/metrics.ts:6: ../api/contract; web/src/state/metrics.ts:7: ./agentState
- **Importers/callers:** web/src/panels/Header.tsx:5; web/src/state/store.ts:28; web/tests/metrics.test.ts:2
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; tests/metrics.test.ts 3 tests.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/state/store.ts

- **Metadata:** 23407 bytes · sha256 `6289459A98C9BD6E875100B4D13559E511EBF2EB6E90DA44AD1B638F749990EE` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** The single zustand store: run/task graph, chat stream, approvals/history/settings/files, connection, refusals, pending commands, ACK-before-mutation.

**Mechanics (deep analysis).**
1. State shape covers run, tasks map, graphRev, selected task, messages/streaming, server metrics, history/replay, approvals, refusal toasts, pending commands, nav/tab, viewport, chat pin/visibility, settings, file tree/content, project identity and reduced-motion override â€” every surface reads a selector from here (735 lines, the largest client state unit).
2. Command lifecycle: `run()` mints a uuid-v7 correlation, records a pending entry, awaits `api.send`, and clears pending in finally; refusals are toasted once (server ERROR events are toasted by applyEvent; local transport/pre-send refusals by the catch), never converted into optimistic success. Mutating state changes are applied ONLY from server events (ACK-before-mutation rule).
3. applyEvent is the event switch: RUN_CREATED/STATUS update run; REPLY_* append/finish chat; NODE_* upsert tasks with optional metadata fallback (worker/`Task <id>` when the frozen payload omits kind/label); RUN_METRICS replaces metrics; APPROVAL_* and SETTINGS_STATUS_RESULT/FILE_* fill their panels; HISTORY_SNAPSHOT stores latest/selected/replay; ERROR toasts and flags missing key; PING/PONG no-op.
4. Guards: submitPrompt validates before send (empty/too-long refusals), spawn/rewire/cancel/kill/approval/template/history each check preconditions and refuse by name; `guardActiveRun` blocks no-run mutations; `probeSettings` pending guard gives one command per open.
5. Multi-source honesty: metrics may be server or derived (state/metrics.ts) with source labelled; history replay stores the graph but (per F-12) Canvas does not render it; file content/text sha come from server payloads; nothing is synthesised.
6. Risks: two separate import statements from '../api/contract' (lines 15/22/27) are a maintainability smell; the store is a god-object mixing domain/network/UI concerns and is only unit-tested through a fake API, not with a real socket; HISTORY_SNAPSHOT graph is never parsed (typed as unknown) so replay semantics are UNVERIFIED.
7. Dependencies: api/client (UiApi), api/contract, api/ids, api/refusals, api/health, metrics, strings, zustand/vanilla; consumed by every panel plus tests. No React in the store itself â€” the context module binds it â€” which keeps it testable in Node.
8. Quality: correctness 4 (guards/ACK gating tested), maintainability 3 (size/import smell), modularity 4 (domain types split out), error handling 5 (named refusals), security 4, docs 4, evidence 5.

- **Dependencies:** imports → web/src/state/store.ts:7: zustand/vanilla; web/src/state/store.ts:22: ../api/contract; web/src/state/store.ts:23: ../api/ids; web/src/state/store.ts:24: ../api/refusals; web/src/state/store.ts:25: ../api/client; web/src/state/store.ts:26: ../api/health; web/src/state/store.ts:27: ../api/contract; web/src/state/store.ts:28: ./metrics; web/src/state/store.ts:29: ./strings
- **Importers/callers:** web/src/App.tsx:8; web/src/flow/graph.ts:9; web/src/panels/Header.tsx:7; web/src/panels/NavRail.tsx:5; web/src/state/context.ts:7; web/tests/metrics.test.ts:3; web/tests/store.test.ts:2
- **Quality classification:** correctness 4/5 · maintainability 3/5 · modularity 4/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; tests/store.test.ts 8 tests.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/state/strings.ts

- **Metadata:** 12692 bytes · sha256 `D6C9E9420D9280D1E9142733C4DED11E050E09EDEB9BA256C36D7B19230033C3` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** Single localisable string table for all UI copy plus pure formatting helpers and client-side refusal copy.

**Mechanics (deep analysis).**
1. STRINGS groups copy by surface (app/brand/nav/tabs/hud/connection/chat/composer/controls/library/templates/scrubber/minimap/toast/settings/panels/diff/approval/node/seed/agentStates/clientRefusals) with provenance notes where copy derives from the PDF; components import this table instead of hardcoding strings.
2. Formatting helpers are pure and null-aware: formatCount/formatPercent/formatDelta/formatCtx return the contract null glyph 'â€”' for null/NaN/Infinity, preventing fake zeros; formatStateOf composes scrubber copy; refusalCopy resolves client-side refusal text by code.
3. Client refusal copy covers only codes the UI can raise without a server frame (prompt empty/long, WS disconnected, schema invalid, origin/token, zoom, resync, task/run/approval gaps, kill denied) and an 'unrecognised refusal' fallback that echoes the server code â€” it does not invent causes for unknown server refusals.
4. The 'seed' group carries PDF-illustrative strings (Backend Architecture, tailwind-builder, Prisma Generator, schema.prisma) used only by the DEV mock, so illustrative content cannot leak into production paths as real data (also enforced by R-U13/noHardcoded tests for banned literals).
5. Accessibility: all labels used in aria-label/roles originate here, so the a11y vocabulary is reviewable in one place; dynamic data is interpolated by components, not concatenated into fixed sentences.
6. Maintainability: it is a 12.7 KB single file with nested groups; no i18n library/plurals, and no compile-time key union â€” a wrong key would surface only at runtime (TypeScript infers the object type, so typos are caught when accessed; dynamic keys like STRINGS.connection[connection] are structurally safe).
7. Evidence: tests/noHardcoded.test.ts scans all src TS/TSX for banned illustrative literals and hex outside the token module; tests/agentState.test.ts asserts every state label; the file itself has no side effects.
8. Quality: correctness 5 (pure table), maintainability 4 (size/no i18n), modularity 5 (single concern), error handling 4 (null-aware), security 4 (no secrets), docs 5, evidence 5.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** web/src/App.tsx:11; web/src/api/refusals.ts:8; web/src/chat/ChatCore.tsx:9; web/src/flow/Canvas.tsx:26; web/src/flow/nodes.tsx:8; web/src/panels/AgentsPanel.tsx:4; web/src/panels/ApprovalOverlay.tsx:3; web/src/panels/ControlBar.tsx:8; web/src/panels/DataPanel.tsx:4; web/src/panels/DevGallery.tsx:5; web/src/panels/DiffDecisionModal.tsx:3; web/src/panels/Header.tsx:4; web/src/panels/LibraryPanel.tsx:3; web/src/panels/ModelsPanel.tsx:3; web/src/panels/NavRail.tsx:4; web/src/panels/RoutingPanel.tsx:3; web/src/panels/Scrubber.tsx:4; web/src/panels/SettingsPanel.tsx:4; web/src/panels/StatePack.tsx:3; web/src/panels/ToastRegion.tsx:4; web/src/state/agentState.ts:7; web/src/state/store.ts:29; web/tests/agentState.test.ts:12
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 5/5 · error handling 4/5 · security 4/5 · docs 5/5 · evidence 5/5 — read; noHardcoded test enforces table usage for banned literals.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/state/useAppReducedMotion.ts

- **Metadata:** 388 bytes · sha256 `CB9A8F53F90D2BA173BB273C4872DA9F6E5CD933B6C8E539226D2FC4279E5591` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Combines OS prefers-reduced-motion with in-app override.

- **Mechanics.** Combines OS prefers-reduced-motion with in-app override. It imports `web/src/state/useAppReducedMotion.ts:4: ./context; web/src/state/useAppReducedMotion.ts:5: ../a11y/reducedMotion` and is imported by `web/src/App.tsx:10; web/src/chat/ChatCore.tsx:10; web/src/flow/Canvas.tsx:25; web/src/panels/ToastRegion.tsx:5`. Classification: source.

- **Dependencies:** imports → web/src/state/useAppReducedMotion.ts:4: ./context; web/src/state/useAppReducedMotion.ts:5: ../a11y/reducedMotion
- **Importers/callers:** web/src/App.tsx:10; web/src/chat/ChatCore.tsx:10; web/src/flow/Canvas.tsx:25; web/src/panels/ToastRegion.tsx:5
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; trivial composition.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/styles/index.css

- **Metadata:** 3601 bytes · sha256 `E3925C3E6BFEA2A59D7B2DCEE0640C2CE3BA6BB559C675D64CA382162E102C6F` (manifest; re-hashed in audit: MATCH) · CSS · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Tailwind import, token mirror import, type utility classes, base focus/reduced-motion rules, skip link.

- **Mechanics.** Tailwind import, token mirror import, type utility classes, base focus/reduced-motion rules, skip link. It imports `none (no ES imports)` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; global CSS, no hex outside tokens import.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/styles/tokens.css

- **Metadata:** 2858 bytes · sha256 `43A0B8E4C07E5DB70F5F6DF54F29D4BA19D929128A3EB076E3BC35DCA78B2084` (manifest; re-hashed in audit: MATCH) · CSS · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** CSS mirror of the token module (@theme colours/fonts/radii/shadows + type vars).

- **Mechanics.** CSS mirror of the token module (@theme colours/fonts/radii/shadows + type vars). It imports `none (no ES imports)` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 4/5 · maintainability 3/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 4/5 — read; duplicated hex with tokens.ts (F-04), fonts absent (F-01).
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/src/tokens/contrast.ts

- **Metadata:** 1683 bytes · sha256 `C3193DB4403CC32A0B95AA7D1DA985760E872C548B399AD121E188ABC131725F` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** WCAG 2.1 contrast maths (hexâ†’rgb, linear luminance, ratio, pass thresholds, formatting) for token evidence.

**Mechanics (deep analysis).**
1. hexToRgb validates a 6-digit hex string and throws on malformed input; relativeLuminance applies the WCAG sRGB piecewise linearisation; contrastRatio returns (lighter+0.05)/(darker+0.05) â€” the canonical WCAG 2.1 formula.
2. passesAA (>=4.5), passesAALarge (>=3), passesNonText (>=3) and formatRatio provide the gate predicates used by tests/apps; no colour math is duplicated elsewhere, so correction claims are machine-checked.
3. The module is pure and browser-free (no DOM/canvas sampling) â€” evidence for the 'computed, not claimed' requirement (UI-R-02..05). Tests reproduce the six PDF rows within 0.05 and verify meta/focus/amber corrections.
4. Edge behaviour: NaN/invalid hex throws loudly at development/test time rather than silently producing wrong ratios; ratios are computed from the token module's hex values, not from sampled pixels.
5. Quality: correctness 5 (formula + tests), maintainability 5 (small pure module), modularity 5, error handling 5 (explicit throw), security 4 (neutral), docs 4 (header comment), evidence 5.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** web/tests/tokens.test.ts:10
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 5/5 · security 4/5 · docs 5/5 · evidence 5/5 — read; tests/tokens.test.ts contrast rows/corrections.
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/src/tokens/tokens.ts

- **Metadata:** 7070 bytes · sha256 `078354A37D8AC7549A980E1F7FEE2625868B6ED1E953C0D298CC97145F9CD069` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** The authoritative token module: exactly 20 PDF swatches with pdfLine provenance, type/space/radius/elevation data, contrast rows, corrections, zoom/routing constants.

**Mechanics (deep analysis).**
1. SWATCHES contains exactly 20 entries mapping idâ†’hexâ†’pdfLineâ†’group (surface/honey/ink/state/accent), covering PDF:21-78; the count and uniqueness are asserted by tests, and every value carries its evidence line so G-UI-11 is reviewable.
2. DECLARED_CONTRAST_ROWS reproduces the six PDF measured rows (14.67/8.74/12.64/3.32/5.41/4.64 with grades) and CORRECTIONS records replacements: card meta ink-500 (PDF:223), focus ring ink-700 (PDF:237), amber large-text-only; deliberate deviations are recorded, not silently changed.
3. TYPE_SCALE (8 printed steps with size/line/tracking/pdfLine), SPACING_STOPS [4..96], RADII_STOPS [4,8,12,16,24,48,999] with button/super-card exceptions 10/14, ELEVATION (single printed warm-ink value, tier differentiation UNVERIFIED) and ZOOM/ROUTING constants encode the rest of the design system.
4. `swatch(id)` throws for unknown ids, so components cannot silently fall back to an arbitrary colour; SEMANTIC maps roles to token ids (textPrimary, metaText, focusRing, error, success, bee, node tones) used by components.
5. The token values are duplicated into styles/tokens.css as CSS custom properties (the CSS mirror, the only other file allowed hex by the noHardcoded test); keeping two copies in sync is a manual maintainability risk if either is edited alone.
6. Fonts are declared by name only (Playfair/Inter/Consolas in CSS) while no font package is installed â€” the cause of F-01 (fallback faces, R-U12 not fully met); types/space/radius are otherwise complete.
7. Dependencies: none (pure data/functions). All colour/type/space claims in the UI resolve here or in the CSS mirror; contrast maths lives in the sibling contrast.ts and is tested against these hex values.
8. Quality: correctness 5 (test-verifiable), maintainability 4 (TS/CSS mirror sync), modularity 4 (one token module pair), error handling 4 (throws on unknown ids), security 4, docs 5 (pdfLine provenance), evidence 5.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** web/tests/noHardcoded.test.ts:2
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 5/5 · evidence 5/5 — read; tests/tokens.test.ts (20 swatches/provenance/contrast).
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

## web/tests/agentState.test.ts

- **Metadata:** 1819 bytes · sha256 `D2B3849B5F282D8353298AD83C3DB1BB7945B5899A3B60D62DBBC13BA56CF433` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Pins 15-state enum, dot mapping, shape cues, transitions, guards.

- **Mechanics.** Pins 15-state enum, dot mapping, shape cues, transitions, guards. It imports `web/tests/agentState.test.ts:1: vitest; web/tests/agentState.test.ts:12: ../src/state/strings` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → web/tests/agentState.test.ts:1: vitest; web/tests/agentState.test.ts:12: ../src/state/strings
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; 5 tests pass (FINAL-REPORT).
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/tests/apiClient.test.ts

- **Metadata:** 4998 bytes · sha256 `1BBAB1FE55DEDCA8BFC210098ED60BDD19BD9CE6834BFC347DF982C36D947F75` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Pins token/hello handshake, ACK-only resolve, typed ERROR reject, malformed frame, jail pre-check.

- **Mechanics.** Pins token/hello handshake, ACK-only resolve, typed ERROR reject, malformed frame, jail pre-check. It imports `web/tests/apiClient.test.ts:1: vitest; web/tests/apiClient.test.ts:2: ../src/api/client; web/tests/apiClient.test.ts:3: ../src/api/refusals; web/tests/apiClient.test.ts:4: ../src/api/contract` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → web/tests/apiClient.test.ts:1: vitest; web/tests/apiClient.test.ts:2: ../src/api/client; web/tests/apiClient.test.ts:3: ../src/api/refusals; web/tests/apiClient.test.ts:4: ../src/api/contract
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 4/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; 5 tests pass.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/tests/contract.test.ts

- **Metadata:** 4472 bytes · sha256 `7790F09690295F6FE6693E24B4403D5CB214915949C56AFD90BEE9D10817FDC3` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Pins 14 commands/21 events/40 codes, parse/build, validators, uuidv7 bits.

- **Mechanics.** Pins 14 commands/21 events/40 codes, parse/build, validators, uuidv7 bits. It imports `web/tests/contract.test.ts:1: vitest; web/tests/contract.test.ts:15: ../src/api/ids` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → web/tests/contract.test.ts:1: vitest; web/tests/contract.test.ts:15: ../src/api/ids
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 4/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; 8 tests pass.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/tests/keyboard.test.ts

- **Metadata:** 2409 bytes · sha256 `FC05F6443800BF2C2F192D5319D5F1DB1B2155A43021DC3C24327B517A16A934` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Pins Ctrl+0/Meta/Digit0 recenter, editable ignore, zoom clamp, bee angle, roving focus.

- **Mechanics.** Pins Ctrl+0/Meta/Digit0 recenter, editable ignore, zoom clamp, bee angle, roving focus. It imports `web/tests/keyboard.test.ts:1: vitest; web/tests/keyboard.test.ts:2: ../src/a11y/keyboard` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → web/tests/keyboard.test.ts:1: vitest; web/tests/keyboard.test.ts:2: ../src/a11y/keyboard
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 4/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; 7 tests pass.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/tests/metrics.test.ts

- **Metadata:** 1484 bytes · sha256 `16AAAC10CA47EE6ACE0B5AF9F7EF379DBE0F2CA75CBFAD65230789504210CFB3` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Pins HUD null-vs-derived-vs-server semantics.

- **Mechanics.** Pins HUD null-vs-derived-vs-server semantics. It imports `web/tests/metrics.test.ts:1: vitest; web/tests/metrics.test.ts:2: ../src/state/metrics; web/tests/metrics.test.ts:3: ../src/state/store` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → web/tests/metrics.test.ts:1: vitest; web/tests/metrics.test.ts:2: ../src/state/metrics; web/tests/metrics.test.ts:3: ../src/state/store
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; 3 tests pass.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/tests/noHardcoded.test.ts

- **Metadata:** 2228 bytes · sha256 `45D5B407FBA4108F2857D318AD71F78F2B02F9279A4BECA487084101AD553B66` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Token isolation/no-hex scan, banned illustrative literals, asset-seam-only scan.

- **Mechanics.** Token isolation/no-hex scan, banned illustrative literals, asset-seam-only scan. It imports `web/tests/noHardcoded.test.ts:1: vitest; web/tests/noHardcoded.test.ts:2: ../src/tokens/tokens` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → web/tests/noHardcoded.test.ts:1: vitest; web/tests/noHardcoded.test.ts:2: ../src/tokens/tokens
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 4/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; 4 tests pass; scans src only (F-07).
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/tests/store.test.ts

- **Metadata:** 7331 bytes · sha256 `8945EF17B632EDB3717557406BEA6E0D4F10267576A090E78D8AA3BA38D7C3BC` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Pins ACK-before-mutation, refusals, approvals/nonce, settings single-command guard, metadata fallback.

- **Mechanics.** Pins ACK-before-mutation, refusals, approvals/nonce, settings single-command guard, metadata fallback. It imports `web/tests/store.test.ts:1: vitest; web/tests/store.test.ts:2: ../src/state/store; web/tests/store.test.ts:3: ../src/api/refusals; web/tests/store.test.ts:4: ../src/api/contract; web/tests/store.test.ts:5: ../src/api/client` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → web/tests/store.test.ts:1: vitest; web/tests/store.test.ts:2: ../src/state/store; web/tests/store.test.ts:3: ../src/api/refusals; web/tests/store.test.ts:4: ../src/api/contract; web/tests/store.test.ts:5: ../src/api/client
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 4/5 · error handling 5/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; 8 tests pass.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/tests/tokens.test.ts

- **Metadata:** 2170 bytes · sha256 `80B68A13EA77F61A5423342DE67C5AE178CA5F3E3D770AACFF16CD579E238646` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Pins 20 swatches/provenance, six contrast rows, corrections, scale arrays.

- **Mechanics.** Pins 20 swatches/provenance, six contrast rows, corrections, scale arrays. It imports `web/tests/tokens.test.ts:1: vitest; web/tests/tokens.test.ts:10: ../src/tokens/contrast` and is imported by `none in web/src or web/tests`. Classification: source.

- **Dependencies:** imports → web/tests/tokens.test.ts:1: vitest; web/tests/tokens.test.ts:10: ../src/tokens/contrast
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 5/5 · modularity 5/5 · error handling 4/5 · security 4/5 · docs 4/5 · evidence 5/5 — read; 6 tests pass.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/tsconfig.json

- **Metadata:** 618 bytes · sha256 `A50AC07820F94752D62FBB14FC6CEA5C2CF1A483B28D56C3F7EADF63D682A8BE` (manifest; re-hashed in audit: MATCH) · JSON · config-manifest/build/dependency/config · seat watchdog-1 · depth **standard** · checkpoint HEAD `3988622`
- **Purpose:** Strict TS 7 config: bundler resolution, react-jsx, ES2022, noUnused*, noEmit, tests included.

- **Mechanics.** Strict TS 7 config: bundler resolution, react-jsx, ES2022, noUnused*, noEmit, tests included. It imports `none (no ES imports)` and is imported by `none in web/src or web/tests`. Classification: config-manifest.

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 3/5 · evidence 5/5 — read; typecheck exit 0 evidence.
- **Evidence/UNVERIFIED:** read in audit + imports/callers

## web/vite.config.ts

- **Metadata:** 798 bytes · sha256 `4C0BF654D4C48B24B13C960327C080A5B32544F78E1BE2DC810D07611CDDC09F` (manifest; re-hashed in audit: MATCH) · TypeScript · source/first-party code · seat watchdog-1 · depth **deep** · checkpoint HEAD `3988622`
- **Purpose:** Vite 8 build/dev config: React 19 plugin, Tailwind 4 plugin, localhost dev/preview ports, ES2022 build, vitest node test config.

**Mechanics (deep analysis).**
1. `defineConfig` from `vitest/config` merges Vite and Vitest config in one file: plugins react() and tailwindcss(), server host 127.0.0.1 port 5173 strictPort, preview 4173 strictPort, build target es2022/outDir dist/sourcemap false/emptyOutDir true.
2. The production bundle is designed to be served by the Rust core (rust-embed) from the same origin; the dev server is separate, which is why apiBaseUrl falls back to http://127.0.0.1:8080 when running on port 5173.
3. Vitest is configured for node environment and only tests/**/*.test.ts, matching the pure-logic test strategy (no jsdom/component tests). Sourcemaps are disabled in production, trading debuggability for size; no chunking/manualChunks configuration, so the app ships as one large chunk (F-16).
4. No environment-variable plumbing besides import.meta.env defaults; no proxy, no HTTPS, no PWA/service worker plugin, no bundle analyzer. Those are deliberate minimalism for a localhost v1, but the missing bundling config is a concrete debt.
5. Edge: strictPort makes a busy port fail loudly instead of silently changing origin (important for the server Origin allowlist); emptyOutDir avoids stale assets in dist (which was verified by the closeout hash-identity check).
6. Dependencies: vite, @vitejs/plugin-react, @tailwindcss/vite, vitest/config. The file is build-only and not imported by runtime code, so it cannot affect product behavior except through plugin/transform defaults.
7. Quality: correctness 5 (green typecheck/build), maintainability 4, modularity 4, error handling 4 (strictPort), security 4 (localhost), docs 3 (no comments beyond purpose), evidence 5 (build evidence).

- **Dependencies:** imports → none (no ES imports)
- **Importers/callers:** none in web/src or web/tests
- **Quality classification:** correctness 5/5 · maintainability 4/5 · modularity 4/5 · error handling 4/5 · security 4/5 · docs 3/5 · evidence 5/5 — read; build log evidence (FINAL-REPORT, typecheck/build).
- **Evidence/UNVERIFIED:** read in audit + symbol map + imports

