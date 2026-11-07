# 07 — UI stack: language, framework, renderer and wiring for the FigCraft spatial UX

| Field | Value |
|---|---|
| Document | `docs/trace/ui-rebuild/07-ui-stack.md` (series item 07) |
| Seat | Frontend architecture analysis — stack + canvas engine decision |
| Date | 2026-11-07 (analysis window 15:00–15:15 CST; machine facts recorded 15:11:34 CST) |
| Status | **PLAN — decision-complete.** No product source was edited. Nothing was installed. The only file this seat wrote is this document. |
| Question answered | What language/framework and wiring should the HiveMind UI use to faithfully implement the FigCraft spatial UX while preserving the verified Rust backend? |
| Inputs read | `web/package.json`, `web/vite.config.ts`, `web/tsconfig.json`, `web/index.html`, `web/src/main.tsx`, `web/src/App.tsx`, `web/src/tokens/tokens.ts`, `web/src/styles/{index,tokens}.css`, `web/src/state/{store,context,agentState,strings,metrics}.ts`, `web/src/api/{client,contract,index,refusals}.ts`, `web/src/flow/{Canvas,graph,nodes,BeeEdge}.ts(x)`, `web/src/a11y/keyboard.ts`, `web/src/assets/manifest.ts`, `web/src/panels/*` (read-only); `backend/crates/hiverouter-core/src/{http,ws,graph,caps,static_ui}.rs` (read-only); `docs/contracts/UI-COMMANDS-v1.md`; `e2e/package.json`, `e2e/playwright.config.ts`, `e2e/README.md`, `e2e/specs/*` (grep census); `docs/ui/figcraft/HiveMind/README.md`, `HiveMind-ASSETS.json`; `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md`; `C:\Users\Riley\Desktop\SPATIAL CANVAS ENGINE MAP.txt` (1 337 lines, read in full for D1–D7 + Appendices C/D/F); `C:\Users\Riley\Desktop\graphcontract.json` |
| Source of truth (requirements) | `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md` §2, §5 (Req. 1/3), §4 prompts 1–10 |
| Source of truth (contract) | `docs/contracts/UI-COMMANDS-v1.md` (frozen 2026-11-06, 1.2 additive) + `web/src/api/contract.ts` + `backend/crates/hiverouter-contract` |
| Implementation target asserted here | TypeScript 7.0.2 strict + React 19.3.1 + Vite 8.3.6 + Tailwind 4.3.4 + `@xyflow/react` 12.12.1, **no renderer replacement, no new runtime dependency required** (one OFL font package proposed; see §4) |

---

## 0. Scope, method, and how to read the claims

### 0.1 Claim classes (used on every load-bearing statement)

| Tag | Means | How it was established |
|---|---|---|
| **M-nn** | MEASURED, this session, this machine | a command was run and its output is quoted (Appendix A) |
| **R-nn** | READ from a file, this session | `path:line` is cited; the file content is the evidence |
| **G-nn** | GREPPED census, this session | a search whose pattern and zero/non-zero result is quoted |
| **E-nn** | ESTIMATE | arithmetic or design counting, never presented as observation |
| **U-nn** | UNVERIFIED | not established; §7 names the decisive experiment |
| **P-nn** | PROPOSED by this document | a name/shape this plan defines; it is not built and not authoritative until the owning seat accepts it |

There is **no frame-rate claim anywhere in this document** (M-nn numbers are Node microbenchmarks of pure functions plus repository facts, not browser rendering measurements). The phrase "smooth" always means "inside the structural caps of §3", never a measured fps.

Reference machine for every M-nn: `win32 x64`, Windows 10 `10.0.19045`, **Intel Core i5-3470S @ 2.90 GHz (4 logical CPUs), 7.9 GB RAM**, Node **v26.7.0**, npm **11.19.0** (M-01). This is the same machine class the SPATIAL CANVAS ENGINE MAP used (its Appendix D.1), so its numbers and mine are comparable.

### 0.2 The workspace moved during this analysis — the pinned baseline

Other rebuild seats are editing `web/src/**` concurrently. `web/src/flow/Canvas.tsx` changed at **15:04:45** — *during* this read — and `web/src/tokens/tokens.ts` at **15:01:13**. Every statement in §1 is pinned to these digests (G-01); a later document that contradicts §1 should be re-run against the then-current tree, not argued from memory.

| File | SHA-256 (first 16 hex) | Bytes | mtime |
|---|---|---|---|
| `web/src/flow/Canvas.tsx` | `1B98B49BC30B5854` | 16 892 | 15:04:45 |
| `web/src/flow/graph.ts` | `2842E1C084B66AFE` | 6 112 | 15:01:19 |
| `web/src/flow/nodes.tsx` | `B69DB7E9AF52B341` | 9 435 | 14:47:04 |
| `web/src/flow/BeeEdge.tsx` | `EFAFC8F82E642AFB` | 1 853 | 15:00:53 |
| `web/src/tokens/tokens.ts` | `00976BBD4343C933` | 7 722 | 15:01:13 |
| `web/src/App.tsx` | `96E275A44C35CB24` | 3 967 | 14:46:04 |
| `web/src/state/store.ts` | `27A2E6D4CA069F51` | 78 483 | 10:21:32 |
| `web/src/api/client.ts` | `745FC57A06EBA0AB` | 19 031 | 21:51:58 (11-06) |
| `web/src/api/contract.ts` | `4E66EB111F8B7FB2` | 38 086 | 11:18:30 (11-06) |
| `web/package.json` | `434960F0E97847F1` | 948 | 17:14:55 (11-06) |

### 0.3 What this document is not

- It is **not** a green light to change the frozen wire contract, the Rust backend, or the token module.
- It is **not** a mandate to replace React Flow. §2.5 rejects replacement on evidence.
- It is **not** an implementation. §6 is a sequence a later seat executes; this seat wrote only this file.

---

## 1. Current stack truth

### 1.1 Language, runtime, toolchain

| Item | Value | Evidence |
|---|---|---|
| Language | TypeScript **7.0.2**, `strict: true` plus `noUnusedLocals`, `noUnusedParameters`, `noImplicitOverride`, `noFallthroughCasesInSwitch`, `isolatedModules`, `noEmit` | R-01 `web/tsconfig.json:9-19` |
| Compile target | `ES2022`, `module ESNext`, `moduleResolution bundler`, `jsx react-jsx`, libs ES2022+DOM+DOM.Iterable | R-01 `web/tsconfig.json:3-20` |
| UI framework | React **19.3.1** + react-dom **19.3.1** (MIT) | R-02 `web/package.json:22-23`; M-02 installed versions match |
| Bundler | Vite **8.3.6** (`@vitejs/plugin-react` 6.2.1, MIT) | R-02 `web/package.json:30,33`; M-02 |
| Styling | Tailwind **4.3.4** via `@tailwindcss/vite` 4.3.4, tokens declared in a Tailwind `@theme` block | R-03 `web/src/styles/tokens.css:6-48`; R-04 `web/src/styles/index.css:1-2` |
| State | zustand **5.0.16** (vanilla `createStore`, one store per App mount) | R-02; R-05 `web/src/state/context.ts:9-17`; R-06 `web/src/state/store.ts:7` |
| Canvas | `@xyflow/react` **12.12.1** (MIT) + `@xyflow/system` 0.0.84, `@dagrejs/dagre` **3.1.3** (MIT) | R-02 `web/package.json:18-19`; M-02 |
| Reconnect | `partysocket` **1.3.0** (MIT) | R-07 `web/src/api/client.ts:13,107-144` |
| Animation | `motion` **14.9.1** (MIT) — used in exactly one module | G-02 `motion/react` appears only in `web/src/panels/ToastRegion.tsx:2` |
| Tests | vitest **5.1.0**, `environment: 'node'`, `include: ['tests/**/*.test.ts']` | R-08 `web/vite.config.ts:26-30` |
| Node floor | `>=22.12.0`; host is **v26.7.0** | R-02 `web/package.json:7-9`; M-01 |
| npm | **11.19.0** (host; `npm.ps1` blocked by execution policy — use `npm.cmd`) | M-01, M-03 |
| Licences | every installed dependency above is **MIT**, except TypeScript (**Apache-2.0**) | M-02 |

No `@fontsource/*`, `jsdom`, `happy-dom`, `@testing-library/*`, Playwright, or coverage provider is installed in `web/` (G-03, M-04) — relevant to §5 and §1.6.

### 1.2 Build and delivery pipeline

- `npm run build` = `tsc --noEmit && vite build` (R-02 `web/package.json:13`). `build.target: es2022`, `outDir: dist`, `sourcemap: false`, `emptyOutDir: true` (R-08 `web/vite.config.ts:20-25`).
- Dev server pinned to `127.0.0.1:5173` `strictPort`; preview `127.0.0.1:4173` (R-08 `:10-19`). When the page is served from port 5173 the client falls back to `http://127.0.0.1:8080` for `/api/*`; otherwise same-origin (R-07 `client.ts:146-155`).
- **The production bundle is embedded in the Rust binary at compile time**: `#[derive(RustEmbed)] #[folder = "../../../web/dist"]` (R-09 `backend/crates/hiverouter-core/src/static_ui.rs:13-16`), served by the router fallback (R-10 `http.rs:496`). A UI change therefore needs a `vite build` **and** a Rust rebuild to reach the shipped binary. This is the single most important build constraint in the project and §6 honours it.
- The current `web/dist` artifact is **one JS chunk of 724 133 B** plus 40 640 B CSS (M-05 `web/dist/assets/*`). There is **no code splitting**: `App.tsx` statically imports `./flow/Canvas` (R-11 `App.tsx:24`), so `@xyflow/react` is in the initial chunk. Vite's default `chunkSizeWarningLimit` is 500 kB, so this build warns (E-01; the warning was already recorded in `docs/plans/FRONTEND-STATUS.md:91`).
- **Typographic faces are declared but never loaded.** `tokens.css` asks for `"Playfair Display Variable"` / `"Inter Variable"` (R-03 `:30-31`), while `index.html` has no `<link>` to a font and neither `package.json` nor `package-lock.json` mentions `fontsource` (R-12 `web/index.html:1-15`; G-03). The shipped UI therefore renders in the fallback stack (`Georgia` / `system-ui`). This is a **verified fidelity gap**, already self-reported at `docs/plans/FRONTEND-STATUS.md:50,91`.

### 1.3 State architecture

One zustand store per `App` mount (R-11 `App.tsx:107`), provided by context, read through a selector hook (R-05). `UiState` is a **78-field interface** (R-06 `store.ts:241-319`) covering: connection/session, `run`, `tasks`, `graphRev`, `selectedTaskId`, chat messages, metrics, history/replay, snapshot/rehydration, resync gap, approvals (5 sub-fields), tool activity, refusals, pending commands, view, **viewport**, chat pin, settings, file tree, project, reduced-motion override, diff modal — plus 34 action methods.

The load-bearing rules, all verified in code and worth preserving verbatim:

1. **ACK-before-mutation.** `run(name, payload)` registers a pending command, awaits the transport, and on failure routes a typed refusal (R-06 `store.ts:1650-1681`). Commands resolve **only** on `ACK`; they reject only on a typed `ERROR` frame or a transport refusal (R-07 `client.ts:373-501`).
2. **Server events are the only mutation source for graph state.** `applyEvent` switches on all 24 event names (R-06 `store.ts:1259-1660`); `RUN_CREATED` resets tasks/history for a fresh run (R-06 `:1321-1342`).
3. **`rev` is the server CAS revision, never a client counter.** `TaskRecord.rev` is documented as `graph.rs TaskNode.rev`; a live `NODE_*` frame with no `rev` sets it to `null` ("unconfirmed"), and `NODE_DRAG` preserves the last confirmed value (R-06 `store.ts:74-84, 1431-1438`).
4. **`rewireSelected` refuses before sending.** `rewireBlocker` returns `no-run | run-terminal | no-selection | target-not-in-graph | unknown-server-rev`; `unknown-server-rev` triggers exactly one bounded history read, then a named refusal — never a guessed `expected_rev` and never an `E_RESYNC_REQUIRED` retry loop (R-06 `store.ts:647-700, 1822-1875`).
5. **Reconnect is a replay window, not a reset.** `SERVER_HELLO` marks approvals stale and rehydrates only when the snapshot is not `ready` (R-06 `:1280-1310`); `RESYNC_REQUIRED` forces `connection: 'reconnecting'` even if the socket is open, until a snapshot settles (R-06 `:1311-1319, 1734-1741`).
6. **Nothing is fabricated.** Missing metrics render `—` through `formatCount/formatPercent/formatCtx` (R-13 `web/src/state/strings.ts`), and every failure path raises a named `Refusal` toast (R-14 `web/src/panels/ToastRegion.tsx`).

### 1.4 Event transport

| Step | Wire fact | Evidence |
|---|---|---|
| Session | `GET /api/session` → `{token, schema, last_seq, ws}`; shape validated field-by-field, mismatch → `E_SCHEMA_INVALID` | R-07 `client.ts:258-306`; R-10 `http.rs:482,882-892` |
| Connect | `ws://…/ws?token=<uuid-v7>&last_seq=<n>` | R-07 `client.ts:308-313` |
| Handshake | first client frame is **raw** `{"type":"hello","last_seq":n}` (not an envelope); server answers `SERVER_HELLO` and replays only the missing range | R-07 `client.ts:317`; R-15 `ws.rs:495-533` |
| Frames | envelopes `{schema,type,name,id,correlation_id,run_id,task_id,seq,ts,payload}` | R-16 `contract.ts:30-41`; `docs/contracts/UI-COMMANDS-v1.md:19-26` |
| Correlation | `ACK` matched by `correlation_id`; a mismatched `command` in the ACK is a client-visible `E_SCHEMA_INVALID` | R-07 `client.ts:373-400` |
| Reconnect | partysocket, `maxRetries 12`, backoff 500→8000 ms ×1.6, `connectionTimeout 6000`, `maxEnqueuedMessages 0`; every reopen re-sends hello with the current `last_seq` | R-07 `client.ts:111-118, 322-328` |
| Refusal on close | pending commands are failed with `E_WS_DISCONNECTED`; status becomes `reconnecting` | R-07 `client.ts:329-339` |
| Pull | `pullPendingApprovals()` re-sends the same hello frame; the server re-publishes open approvals — **no new command** | R-07 `client.ts:524-558`; R-15 `ws.rs:600-608` |
| Vocabulary | **14 frozen commands** (12 server + `VIEWPORT_SET`/`VIEWPORT_RECENTER` client-only), **24 events**, **16 agent states**, **65 refusal codes** | R-16 `contract.ts:119-157, 385-410`; R-17 `web/src/state/agentState.ts:9-26`; M-06 (counted 65 codes in `contract.ts:426-493`) |
| Transport shape | `VIEWPORT_SET`/`VIEWPORT_RECENTER`/`CHAT_PIN` are **explicit no-ops** server-side | R-15 `ws.rs:737-741`; `docs/contracts/UI-COMMANDS-v1.md:44` |
| Drag | **the server never emits `NODE_DRAG`** — zero emitters in the whole backend | G-04 (`Select-String` over `backend/crates/**/*.rs` → 0 matches) |

### 1.5 The canvas as it exists today

`web/src/flow/Canvas.tsx` (461 lines, R-18) is a `ReactFlowProvider` + `ReactFlow` with custom `nodeTypes` (5 renderers) and one custom `edgeTypes` (`bee`). Its behaviour, verified line by line:

| Behaviour | Implementation | Consequence |
|---|---|---|
| Pan/zoom | React Flow built-in; `minZoom={0.1} maxZoom={3}` overrides the library defaults | matches `MIN_ZOOM/MAX_ZOOM` (R-16 `contract.ts:17-18`) and `ZOOM` tokens (R-19 `tokens.ts:134-138`) |
| Recentre | document-level capture `keydown`, Ctrl/Meta+0, instant `setCenter(0,0)` + rAF re-apply (R-18 `:176-203`) | survives focus and React Flow key handling |
| Zoom chrome | +/−/0/fit buttons, live `%` label (R-18 `:396-439`) | S20 |
| Minimap | `<MiniMap>` 220×150 from `LAYOUT.minimap` (R-18 `:359-385`), `aria-label` patched onto `.react-flow__minimap` (R-18 `:205-220`) | S13 |
| Background | static SVG `<pattern>` honeycomb painted in **screen space** (R-18 `:236-243`), plus four blurred ellipse "lobes" (R-18 `:244-268`) | does **not** pan/zoom with the world — the lattice is fixed to the viewport |
| Layout | `dagre` LR with `nodesep 40 / ranksep 90`, root-centred at world `(0,0)` (R-20 `graph.ts:38-77`) | deterministic, but **recomputed inside `buildNodes`** (`graph.ts:85`) |
| Node positions | dagre positions, overridden per node by `localStorage` entries keyed `hivemind.canvas.positions.v1:<runId>` (R-18 `:34-80, 146-155`) | drag survives reload; no server persistence |
| Drag | React Flow node drag; on drop, position saved and, for `worker` nodes, `getIntersectingNodes` picks a `superagent`/`hyperagent` and calls `selectTask(node.id)` then `rewireSelected(target.id)` (R-18 `:320-349`) | **bounding-box overlap**, not a 150-px radius gate; no preview, no ranking, no slot displacement |
| Edges | parent→child `bee` edges; `data.broken` when child state ∈ {failed, killed, interrupted} (R-20 `graph.ts:101-114`) | crimson dashed wire for broken routes (R-21 `BeeEdge.tsx:27-53`) |
| Bee glyph | static SVG at the Bézier midpoint, rotated to the straight source→target angle (R-21 `:22-31, 39-54`) | not animated, not lattice-routed |
| Culling | **none** — `onlyRenderVisibleElements` is not passed (R-18, full prop list) | every node is mounted |
| LOD | **none** — one card anatomy at every zoom | 0.1× renders full cards |
| Filters | `hideFinished` / `focusBranch` local toggles producing `visibleTasks`, plus an `N/M` counter (R-18 `:99-144, 270-302`) | newest edit; not a LOD band |
| Replay | replay nodes/edges built from the snapshot with `interactive=false`, `fitView` on (R-18 `:223-232, 355-356`) | read-only time travel |
| Viewport state | `onMoveEnd` writes `{x,y,zoom}` into the zustand store via `setViewport` (R-18 `:351-354`) | see D-2 |
| A11y | `role="application"`, `aria-label`, skip-link target `#hive-canvas`, `aria-live` node announcement, focusable nodes/edges (R-18 `:235, 311-312, 445-460`) | G-UI-2 gates depend on `#hive-canvas` (`e2e/specs/live-app.spec.ts:23-36`) |

### 1.6 Gate status measured now

| Gate | Command | Observed result |
|---|---|---|
| Typecheck | `npm.cmd run typecheck` (M-07) | `tsc --noEmit`, **exit 0**, no diagnostics |
| Unit tests | `npm.cmd run test` (M-08) | **22 files / 182 tests passed**, 12.71 s wall |
| Build | not run by this seat (would rewrite `web/dist`, which `rust-embed` reads and other seats are actively working against) | **U-01**; last recorded numbers are the 724 133 B artifact (M-05) |
| Browser e2e | not run by this seat (requires a built binary on `127.0.0.1:8799`, `E2E_APP_READY=1`) | **U-02**; the harness itself is **R-22**: Playwright **1.65.0**, installed **Chrome/Edge channels only**, `workers: 1`, `retries: 0`, `forbidOnly: true`, no `webServer`, base URL from `E2E_BASE_URL` |
| Canvas-specific gates | — | **absent**: no spec asserts a snap radius, a slot preview, LOD banding, or a frame budget (G-05) |

### 1.7 Structural defects this analysis found (the gap list, numbered for §6)

| # | Defect | Evidence | Impact |
|---|---|---|---|
| **D-1** | **Layout runs inside the render projection.** `buildNodes()` calls `layoutTasks()` on every invocation (`graph.ts:85`), and the `nodes` memo depends on `selectedTaskId` (`Canvas.tsx:146-155`) — so **selecting a card re-runs dagre for the whole graph**. | M-09: dagre p50 **23.5 ms @50**, **39.6 ms @100**, **93.4 ms @200**, **319.9 ms @500** nodes; the pure projection alone is **0.04–0.29 ms** | a dropped frame on every click at 50 nodes; a 320 ms freeze at 500 |
| **D-2** | **Viewport is written to the app store and read by nobody.** `setViewport` is called on every `onMoveEnd`; `state.viewport` has **zero readers** in `src` | G-06 (`viewport` census: 15 hits, all writers/type/initial value; no reader) | pointless store churn + a second source of truth for the camera |
| **D-3** | **The node object identity churns.** `data` is rebuilt for every node on every projection, and `selected` lives *inside* `data` | R-18 `:146-155`; R-20 `:79-99` | all N node components re-render on any selection or single-node state change |
| **D-4** | **No viewport culling and no LOD.** | R-18 (no `onlyRenderVisibleElements`); §1.5 | full DOM at 0.1×; the FigCraft macro band (Prompt 9) is unreachable |
| **D-5** | **Rewire is triggered by card intersection, not by the spec'd 150-screen-px radius gate**, and fires immediately on drop with no preview | R-18 `:336-348`; spec `HIVEMIND-TARGET-SPEC.md:68-75` | accidental re-parenting; no "glowing target circle + guidelines + displacement preview" (Prompt 5) |
| **D-6** | **The honeycomb is screen-fixed, not world-anchored** and its scale is a literal 48×42 tile | R-18 `:236-243` | zooming does not zoom the lattice; the "crisp vector honeycomb" promise is cosmetic only |
| **D-7** | **Bee is static and the wire is a Bézier curve**, not a two-segment route on the 0°/±60° lattice | R-21 `:14-31`; spec `:63-66, 122-125` | the strongest visual signature of the design is missing |
| **D-8** | **Fonts are declared but never loaded** | §1.2, G-03 | premium typography renders as Georgia/system-ui |

None of these is a reason to change framework. Every one of them is fixable inside the current stack (§2.5, §6).

---

## 2. Fit and gap for a Figma-like canvas

### 2.1 Requirement inventory (what "faithfully implement the FigCraft spatial UX" actually demands)

From `HIVEMIND-TARGET-SPEC.md` §2/§5 and prompts 1–10, plus the SPATIAL CANVAS ENGINE MAP decisions D1–D7, the canvas must provide:

| # | Requirement | Source |
|---|---|---|
| Q1 | Infinite pan, zoom **0.1×–3.0×**, zoom **anchored at the pointer** | spec `:46-51` |
| Q2 | World origin = centre of the HyperAgent hive chat card; **Cmd/Ctrl+0 always returns to it** | spec `:49-51` |
| Q3 | World-anchored honeycomb lattice, crisp at every zoom (banded tile radius) | spec `:47-48`; CANVAS-MAP D4 |
| Q4 | Text-heavy premium cards that use the token system (no canvas text) | CANVAS-MAP D1 |
| Q5 | Two-segment lattice routes (0°, ±60°), directional **bee** facing the flow, SMIL dash flow | CANVAS-MAP D5 |
| Q6 | **150 screen-px radius gate**, path-prioritised ranking, **displacement preview**, one atomic rewire | spec `:68-75`; CANVAS-MAP D6 |
| Q7 | Zoom-banded LOD (5 bands), macro view "hundreds of silhouettes, batched wires, brighter clusters" | spec Prompt 9; CANVAS-MAP D7/App. C |
| Q8 | Minimap with viewport rectangle; HUD; time-travel scrubber that dims inactive lines | spec Prompts 6/10 |
| Q9 | Full keyboard + screen-reader access (cards are real elements, focusable, named) | CANVAS-MAP D1/§4.7; existing G-UI-2 gates |
| Q10 | Reduction of motion honoured (`prefers-reduced-motion` **and** an in-app switch) | `web/src/a11y/reducedMotion.ts`; R-04 `index.css:100-118` |
| Q11 | Graph topology decoupled from coordinates; viewport never enters graph state | spec Req. 1 `:190-195` |
| Q12 | Failure rendering by name: crimson card, broken dashed crimson wire, paused bee, refusal toast with `code` + `how_to_fix` | spec Prompt 7; `ToastRegion.tsx` |

### 2.2 The scale premise must be corrected first: the backend caps a run at **64 nodes**

Every "500-node / 1 000-node / macro-swarm of hundreds" argument in the SPATIAL CANVAS ENGINE MAP and in Prompt 9 is **unreachable against the verified Rust backend**:

| Cap | Value | Evidence |
|---|---|---|
| Max spawn depth | **3** (Hyper → Super → Worker) | R-23 `backend/crates/hiverouter-core/src/caps.rs:9` |
| Max children per parent | **8** | R-23 `caps.rs:11` |
| **Max agents (tasks) per run** | **64** | R-23 `caps.rs:13`; enforced at `caps.rs:49-60`, refusal `E_AGENT_LIMIT` |
| Max provider calls per run | 40 | R-23 `caps.rs:15` |
| One run at a time in the UI | `RUN_CREATED` for a new run clears `tasks` | R-06 `store.ts:1321-1342` |

The worst case the server can ever deliver to the canvas is therefore **64 task nodes**, and the UI never holds two runs' graphs at once. This single fact dissolves most of the React Flow performance debate:

- 64 DOM cards ≈ 640–900 elements (E-02, using the 10–14 elements/card anatomy of `nodes.tsx`), which is an order of magnitude below any browser DOM budget.
- A renderer whose justification is "500+ nodes" is answering a question this product does not ask.
- LOD bands remain **required** — not for headroom, but because Prompt 9 asks for the *look* of a macro swarm, and because 64 full cards at 0.1× are unreadable mush (E-03).
- `dagre` at 64 nodes is still ~25–30 ms (M-09, interpolating 23.5 ms @50 → 39.6 ms @100), i.e. **still a dropped frame per selection** — so D-1 must be fixed regardless of renderer.

### 2.3 Candidate renderers, scored against Q1–Q12

| Criterion | **A. `@xyflow/react` 12.12.1 (current)** | **B. Custom DOM + SVG world layer** (CANVAS-MAP D1/D2) | **C. PixiJS / WebGL** | **D. Hand-rolled `<canvas>` 2D** |
|---|---|---|---|---|
| Q1 pan/zoom + pointer anchoring | built in (d3-zoom under `@xyflow/system`), configurable bounds | must be written + tested | library pan/zoom or hand-rolled | hand-rolled |
| Q2 origin recentre | `setCenter(0,0)` (already shipped) | hand-rolled | hand-rolled | hand-rolled |
| Q3 world-anchored comb | **gap** (screen-space today); fixable only by moving the SVG **inside** the flow viewport or by a CSS background on `.react-flow__viewport` | native to the design | native | native |
| Q4 text cards + tokens | **native DOM**, Tailwind classes work today | native DOM | **loses**: canvas text, manual line-breaking, no selection, blur at fractional zoom | loses |
| Q5 lattice routes + SMIL bee | custom edge component (SVG) — full control of `path` | native | must animate sprites; SMIL unavailable | must animate |
| Q6 snap + preview | implement in our own pure module; React Flow gives node rects + drag callbacks | native | native | native |
| Q7 LOD | `nodeTypes` swap per band + `onlyRenderVisibleElements` + a batched SVG macro layer | native (the design's core idea) | natural | natural |
| Q8 minimap/HUD/scrubber | **`<MiniMap>` shipped and gated** | must be written | must be written | must be written |
| Q9 a11y | **real DOM, focusable, `ariaLabel` per node, `nodesFocusable`** | real DOM | **requires a parallel DOM mirror** | requires a parallel DOM mirror |
| Q10 reduced motion | already wired (`useAppReducedMotion`) | same | same | same |
| Q11 viewport ≠ graph state | true if we stop mirroring viewport into the store (D-2) | true by design | true | true |
| Q12 failure rendering | card + custom edge + toast (already shipped) | same | worse | worse |
| **Existing gate compatibility** | **7 e2e specs assert `.react-flow__viewport`, `.react-flow__node(.draggable/.selectable/.selected/.dragging)`, `.react-flow__edge`, `[data-testid="rf__minimap"]`** (R-24) | **breaks all of them** | breaks all | breaks all |
| Bundle cost | **already in the bundle** (0 new bytes) | −~233 kB ESM source if removed | **+≈400–450 kB min** (E-04, unverified) | 0 |
| Delivery risk | low — it is the shipped, gated renderer | **high** — rebuilds gesture/zoom/a11y/minimap/layout | high | high |

**R-24 evidence (the decisive gate coupling):** `e2e/specs/gui-gates.spec.ts:100,120-123` (Ctrl+0 and zoom transform), `e2e/specs/gui-residual.spec.ts:74` (card located by `.react-flow__node`), `e2e/specs/live-app.spec.ts:80,138-153` (minimap `data-testid="rf__minimap"`, viewport transform), `e2e/specs/replay-state.spec.ts:79-148` (node counts, `.draggable`, `.selectable`, `.selected`, `.dragging`), `e2e/specs/live-replay.spec.ts:137-139`, `e2e/specs/live-trace-capture.spec.ts:149,390-391` (node census for traces), `e2e/specs/trace-hops.spec.ts:41-47,253-259` (surface census S13/S15–S19 selectors on `.react-flow__node[data-id]`). Replacing the renderer does not "adjust a test" — it invalidates the surface census that the watchdog and trace registry are built on.

### 2.4 Why not PixiJS, stated with the product's own numbers

1. **Text.** The cards are 90 % typography (`nodes.tsx`: label, role, state label, progress %, meta, code box). Pixi would either rasterise text (blurring at 0.6×/1.3×/2.9×, an **U** in CANVAS-MAP D1) or need a DOM overlay that duplicates the layer.
2. **A11y.** Q9 requires focusable, named elements. Pixi needs a parallel DOM mirror — which is exactly the DOM layer we already have.
3. **Scale.** The backend caps at 64 nodes §2.2. There is no measured or reachable scenario where DOM cannot keep up.
4. **Cost.** A new renderer means a new ticker, a second coordinate space, context-loss handling, and a new test harness — for zero requirements gained.

### 2.5 Verdict

**Keep TypeScript + React 19.3.1 + Vite 8.3.6 + Tailwind 4.3.4 + zustand 5.0.16 + `@xyflow/react` 12.12.1. Do not replace the renderer.** Demote React Flow from "graph authority" to **presentation host** and move every spatial decision into pure, unit-testable modules:

| Concern | Today | After |
|---|---|---|
| Layout | dagre inside `buildNodes` | `canvas/layout.ts`, memoised by topology key, **never called from render** |
| Snap/ranking | `getIntersectingNodes` | `canvas/snap.ts`, pure, 150 screen-px gate, total-order tie-breaks |
| Routing | Bézier | `canvas/routing.ts`, two lattice segments, same math as CANVAS-MAP D5 |
| LOD | absent | `canvas/lod.ts`, 5 bands + hysteresis |
| Viewport | mirrored into zustand, unread | non-React viewport store; React notified at band/commit boundaries only |
| Persistence | ad-hoc `localStorage` v1 | versioned, validated, capped v2 |
| Macro band | absent | batched SVG silhouette layer inside the flow viewport |

Why this is the faithful answer and not a compromise: the spec itself names React Flow as an acceptable rendering layer — *"Implement a canvas rendering layer (WebGL, HTML5 Canvas, or high-performance libraries such as pixi.js or React Flow)"* (R-25 `HIVEMIND-TARGET-SPEC.md:193-195`). The SPATIAL CANVAS ENGINE MAP's rejection of React Flow (its D1, alternative 3) was written against a **different repository** with **zero** render dependencies and a hard "no UI dependency" posture — it explicitly says "`web/package.json` does not exist" (CANVAS-MAP §2.3). That premise is false here: `@xyflow/react` is the shipped renderer, seven gate specs depend on its DOM, and it is MIT. **We adopt CANVAS-MAP's geometry and LOD thinking, and reject its renderer swap** — the math is portable; the gate risk is not.

### 2.6 What React Flow genuinely cannot give us (mandated workarounds)

| Gap | Workaround (owned by us) |
|---|---|
| No radius-gate/path-prioritised snapping | `canvas/snap.ts` pure function over projected rects; called from `onNodeDrag`; preview layer drawn by us |
| Bézier-only default edges | custom `bee` edge already receives `sourceX/Y`, `targetX/Y` → feed `routing.ts` and emit `<path d="M … L … L …">` |
| No LOD | band from `canvas/lod.ts`; swap `nodeTypes` per band; below B2 render one `<svg>` silhouette layer instead of cards |
| Comb is not part of the world | move the honeycomb `<svg>` **inside** `.react-flow__viewport` (or paint it as a CSS background on the viewport element) so it inherits the one transform |
| Viewport changes re-render consumers | read `useStore`/`useReactFlow` imperatively; never `useViewport()` in a component that renders nodes |
| Node identity churn | memoise `ViewNode` per `(task.id, task.rev, kind, attrsHash)`; remove `selected` from `data` |
| Internals are private | pin `12.12.1` exactly (already exact) and add our own `data-surface="S15"` attributes so a future major bump is a *selector* migration, not a rewrite |

---

## 3. Recommended architecture (decision-complete)

### 3.1 Layers and module map

```
 ┌──────────────────────────── server (frozen, do not change) ────────────────────────────┐
 │ axum router (http.rs:478-496) · WS session (ws.rs:401-467) · graph CAS (graph.rs:141)   │
 │ rust-embed of web/dist (static_ui.rs:13-16)                                             │
 └───────────────────────────────────────┬────────────────────────────────────────────────┘
                                         │ envelopes: seq, ACK, ERROR, NODE_*, HISTORY_SNAPSHOT
 ┌───────────────────────────────────────▼────────────────────────────────────────────────┐
 │ api/client.ts + api/contract.ts          — UNCHANGED (frozen vocabulary, ACK correlation)│
 ├────────────────────────────────────────────────────────────────────────────────────────┤
 │ state/store.ts   server truth: run/tasks/history/approvals/refusals   — API UNCHANGED    │
 │                  (+ one additive action: rewireTask(taskId,parentId) — P-1)              │
 ├────────────────────────────────────────────────────────────────────────────────────────┤
 │ canvas/projection   pure, memoised: TaskRecord[] → ViewNode[] / ViewEdge[]               │
 │   ├─ canvas/layout/layout.ts        deterministic slot lattice (replaces dagre in render) │
 │   ├─ canvas/geometry/routing.ts     two-segment lattice route + bee heading               │
 │   ├─ canvas/geometry/snap.ts        150 screen-px gate, ranking, slot choice, preview     │
 │   ├─ canvas/geometry/lod.ts         band(scale) with ±4 % hysteresis                      │
 │   └─ canvas/persistence/positions.ts versioned localStorage v2                            │
 ├────────────────────────────────────────────────────────────────────────────────────────┤
 │ canvas/viewport  band + zoom-label external store (useSyncExternalStore)                  │
 │                  transform is owned by React Flow; we never mirror x/y/zoom into zustand  │
 ├────────────────────────────────────────────────────────────────────────────────────────┤
 │ canvas/components  CanvasHost (ReactFlow) · WorldComb · DragLayer · BeeEdge · nodes/*     │
 │                    chrome: HudBadge · MiniMapPanel · ZoomControls · Scrubber · HintLine    │
 └────────────────────────────────────────────────────────────────────────────────────────┘
```

**File plan.** `P` = proposed new module; `E` = existing file that changes.

| Path | State | Responsibility | Test surface |
|---|---|---|---|
| `web/src/canvas/layout/layout.ts` | P | `computeLayout(tasks) → Record<TaskId, Point2>`; root at `(0,0)`; 6-direction super ring; 12-slot sub-hive; deterministic overflow | unit (determinism, distances, capacity) |
| `web/src/canvas/layout/topologyKey.ts` | P | stable key = runId + sorted `id:kind:parentId` hash | unit |
| `web/src/canvas/geometry/routing.ts` | P | `routeBetween(sourcePort, targetPort) → {points:[p0,p1,p2], angle}`; lattice basis A0/A1/A2; smallest `|a|+|b|` pair; tie-break by deviation then pair priority | unit (2 segments, on-axis, deterministic) |
| `web/src/canvas/geometry/ports.ts` | P | per-kind port anchors (worker left/right, super top/bottom, hyper six) | unit |
| `web/src/canvas/geometry/snap.ts` | P | `findSnapTarget(input) → SnapDecision \| null`; screen-space gate, align-first ranking, hysteresis, slot occupancy, BFS displacement ≤3 | unit + property (shuffle determinism) |
| `web/src/canvas/geometry/lod.ts` | P | `bandFor(scale, previous)`; 5 bands; ±4 % hysteresis | unit (boundaries) |
| `web/src/canvas/viewport/viewportStore.ts` | P | `{ band, zoomPct }` external store; `notifyMoveEnd(viewport)`, `subscribe/getSnapshot`; no x/y/zoom retention | unit |
| `web/src/canvas/persistence/positions.ts` | P | `load(runId)/save(runId, map)`; v1→v2 migration; validation; 64-entry / 10 kB cap; quota + private-mode failures | unit (corrupt, quota, migration) |
| `web/src/canvas/projection/nodes.ts` | P | `projectNodes(tasks, layout, positions)` with memo table; stable object identity | unit (identity stability) |
| `web/src/canvas/projection/edges.ts` | P | `projectEdges(tasks) → {id,source,target,broken}` | unit |
| `web/src/canvas/components/CanvasHost.tsx` | E | `ReactFlow` with `onlyRenderVisibleElements`, band-switched `nodeTypes`, drag pipeline wiring | e2e |
| `web/src/canvas/components/WorldComb.tsx` | P | world-anchored honeycomb tile inside the flow viewport; 5 pre-generated band tiles | visual |
| `web/src/canvas/components/DragLayer.tsx` | P | snap ring, guideline, ghost slots, refusal chip | e2e + visual |
| `web/src/canvas/components/MacroLayer.tsx` | P | one `<svg>` of ≤64 silhouette rects + batched wires for B0/B1 | unit (rect count) |
| `web/src/flow/BeeEdge.tsx` | E | consume `routing.ts`; SMIL dash + `animateMotion` bee; paused when broken/reduced | unit (path string) + visual |
| `web/src/flow/nodes.tsx` | E | band-aware anatomy; selection via context; stable `data`; `data-surface` | visual + a11y |
| `web/src/flow/graph.ts` | E | keeps `buildReplayTasks`/`buildEdges`; **`buildNodes` loses its internal `layoutTasks` call** | existing `graph.replay.test.ts` must stay green |
| `web/src/flow/Canvas.tsx` | E | becomes `<CanvasHost>` shell; exports unchanged (`Canvas`) so `App.tsx` does not move | e2e |

**Non-goal:** do not move `web/src/flow/*` to `web/src/canvas/*` in one sweep. Rename only when the module is rewritten, so `designParity.test.ts:29-32` (which asserts `sourceOf('/flow/graph.ts')` contains `LAYOUT.cards.superagent` and `Canvas.tsx` contains `LAYOUT.minimap.width`) keeps passing during the transition — an intentional pin, see §5.1.

### 3.2 Viewport

**Owner of the transform: React Flow.** `@xyflow/react` writes a single CSS `transform: translate(x,y) scale(k)` on `.react-flow__viewport`; that is exactly the "one transformed world layer" model CANVAS-MAP D2 asks for, implemented by the library. The defect is not the transform, it is **mirroring it into the app store** (D-2).

Rules (each is testable):

| Rule | Statement | Rationale |
|---|---|---|
| V-1 | The transform has **one** owner (React Flow). No React state, no zustand field, holds `x/y/zoom`. | two sources of truth caused D-2 |
| V-2 | `UiState.viewport` and `setViewport` are **deleted**; `Canvas.tsx` stops pushing on `onMoveEnd`. | R-06 `store.ts:276, 307, 1719, 2119-2121`; G-06 (no readers) |
| V-3 | Programmatic camera moves call `setCenter/setViewport/zoomIn/zoomOut/fitView` from `useReactFlow()`; nothing else writes the transform. | R-18 `:96` |
| V-4 | `minZoom = MIN_ZOOM = 0.1`, `maxZoom = MAX_ZOOM = 3.0` are read from `api/contract.ts`, not literals. | R-16 `contract.ts:17-18`; today literals at R-18 `:308-309` |
| V-5 | Zoom is **pointer-anchored**: `zoomAt(sx, sy, s')` keeps the world point under the cursor fixed (`tx' = sx − ((sx−tx)/s)·s'`). | Q1; CANVAS-MAP D3 formula |
| V-6 | Ctrl/Meta+0 recentres world `(0,0)` **without changing zoom**, instantly, at document capture level. | Q2; R-18 `:176-203` (already correct, keep) |
| V-7 | React is notified at **band change** and **move end** only. `onMove` runs at most one rAF-coalesced update of the zoom label; `onMove`/`onMoveEnd` never call `setState` on the store. | D-2, D-3 |
| V-8 | Non-finite or out-of-range camera values are rejected and the previous transform retained. | CANVAS-MAP D2 stability rule; `validateZoom` exists (R-16 `:1063-1068`) |
| V-9 | The camera is **not persisted**; a reload starts at `scale 1`, world origin centred. | spec `:49-51` (Ctrl+0 must always mean "home"); CANVAS-MAP D3 |
| V-10 | `prefers-reduced-motion` **or** `reducedMotionOverride` forces `duration: 0` on every camera animation. | R-04 `index.css:100-118`; R-18 `:172` |

`viewportStore.ts` (P) therefore holds exactly `{ band, zoomPct, hostW, hostH }` and exposes `subscribe/getSnapshot` for `useSyncExternalStore`. Host size comes from a `ResizeObserver` on `#hive-canvas` (needed for `fitView` padding and the minimap rect).

### 3.3 Layout

**One layout per topology change, never per render.** This is the D-1 fix and the highest-value change in this document.

```
computeLayout(tasks)            // pure; no React, no dagre
  root      := the task with parentId === null (the hyperagent hive chat card)
  root.pos  := (-w/2, -h/2)     // world origin = card CENTRE (spec :49-51)
  supers    := children(root) ordered by (spawn order from insertion, then id)   // deterministic
  supers[i] := ringPoint(i, RING_PITCH)   // six lattice directions, repeating rings
  for each super:
     children := its children, ordered deterministically
     slot[j]  := parentPort + SLOT_OFFSETS[j]      // 12 slots, column pitch 440, row pitch 381.0512, odd-row offset 220
  overflow  := next free slot on the parent's next ring (deterministic BFS), never a random/overlap position
```

| Constant | Value | Source |
|---|---|---|
| Sub-hive column pitch | **440** world px | CANVAS-MAP §A.3 (slot lattice) |
| Row pitch | **440·√3/2 = 381.0512…** | CANVAS-MAP §A.3 |
| Odd-row offset | 220 | CANVAS-MAP §A.3 |
| Slots per parent | **12** | CANVAS-MAP D6; sufficient because `MAX_FANOUT = 8` (R-23 `caps.rs:11`) |
| Card sizes | `LAYOUT.cards.{superagent,worker,tool,error}`, `LAYOUT.chat` | R-19 `tokens.ts:166-179` |

**Memoisation contract**

| Rule | Statement |
|---|---|
| L-1 | `computeLayout` is called only from `projectNodes`, and `projectNodes` is memoised on `topologyKey(tasks, runId)` (a stable string hash of `runId` + sorted `id:kind:parentId`). |
| L-2 | The cache is a 2-entry LRU at module scope; entries are `Object.freeze`d. |
| L-3 | **No `useMemo` that produces nodes may depend on `selectedTaskId`.** Selection is not geometry. |
| L-4 | User drag overrides are applied **after** layout, per id, and are not part of `topologyKey` (dragging must not relayout). |
| L-5 | A `NODE_REWIRE`/`NODE_SPAWN`/`RUN_CREATED` changes `topologyKey` → exactly one recompute. |
| L-6 | Layout never throws on hostile input (orphan parent, cycle, self-parent): it re-roots orphans in deterministic order and records the anomaly for the DEV console; it does not invent a node. |

**Why the lattice and not dagre.** dagre answers "rank a DAG left-to-right"; the FigCraft design answers "place a child in its parent's hive slots". At `MAX_FANOUT = 8` the 12-slot lattice is always sufficient, so a general DAG layout engine adds cost (M-09) and produces positions the design does not describe (children of one parent can be spread across ranks). Decision: **remove `@dagrejs/dagre`** from `web/package.json` once WP-2 lands and `graph.replay.test.ts` has been re-pinned to lattice assertions; keep `buildReplayTasks` unchanged.

### 3.4 State boundaries and selector rules

| Data | Home | May React re-render on change? | Notes |
|---|---|---|---|
| run, tasks, history, metrics, approvals, tool activity, refusals | `state/store.ts` (unchanged) | yes, via narrow selectors | server truth; mutations only from events |
| selection (`selectedTaskId`) | `state/store.ts` (unchanged) | only the affected card(s) + ControlBar | shared with the control bar today |
| layout | memo cache (§3.3) | no (recomputed, not state) | pure |
| projection (`ViewNode[]`) | memo | only when `topologyKey` changes | stable identities |
| drag preview (candidate parent, ghost slots, pointer) | module-local store in `canvas/components/dragStore.ts` (P) | only `DragLayer` + the dragged card | **never** in `UiState` — pointer-rate writes must not touch app state |
| band, zoom label, host size | `viewportStore.ts` (P) | `WorldComb`, `MiniMapPanel`, zoom label | band changes ~5 times per session |
| positions | `persistence/positions.ts` + local component state | the moved card only | `localStorage` behind a validated API |
| replay graph | `history.replayGraph` (unchanged) | yes | read-only |

**Selector rules** (enforced by review; a unit test cannot see them):

- S-1 — a component that renders a card subscribes to `tasks[id]` **by id**, never to the whole `tasks` map.
- S-2 — `selected` must not be part of a card's `data`; the card reads `useUiStore(s => s.selectedTaskId === id)`.
- S-3 — handlers (`onSelect`, `onRewire`) come from React context, not from `data`, so `data` identity depends only on server fields.
- S-4 — `useShallow` for array selectors (`refusals`, `toolActivity`) where identity is recreated.
- S-5 — no `useStore(store)` without a selector in any module reachable from `Canvas`.

### 3.5 Node components

| Kind | Size (`LAYOUT`) | Anatomy at B3/B4 | Anatomy at B2 | Marker at B0/B1 |
|---|---|---|---|---|
| `hyperagent` | 420×500 (`LAYOUT.chat`) | hex containment + pinned hive chat overlay (`ChatCore`) | silhouette + label chip | hex outline |
| `superagent` | 420×150 | honey header (name + state pill), role caption, progress bar, worker count | header only + state dot | rect + emphasis stroke |
| `worker` | 320×96 | state glyph + name + `%`, role label, thin progress | name + dot | rect (kind colour) |
| `tool` | 420×240 | name + kind chip, dark result box, progress, score row | header only | rect |
| `error` | 320×110 | crimson border, ▲, error code, "broken route", attempt count | name + ▲ | crimson rect |

Non-negotiable render rules:

1. **Unknown is not zero.** `progress === null` renders no bar and `—`; `formatPercent/formatCount/formatCtx` are the only formatters (R-13). A test already pins "no hardcoded copy/hex" (`web/tests/noHardcoded.test.ts`).
2. **State is never colour-only.** `visualFor(state)` supplies shape + text (`agentState.ts:48-84`); the glyph must keep its `title`/label.
3. **Selection styling** is a border/shadow change (R-18 `nodes.tsx:41-68`), never a size change (size changes alter measured node dims and force edge recompute).
4. **Real elements.** Cards stay `<div>`/`<button>` with `aria-pressed`; `ariaLabel` on the React Flow node = `"<label> (<kind>, <state>)"` (already at R-20 `graph.ts:94`).
5. **`data-surface` attributes** (`S15`..`S19`) are added so the surface census stops depending on React Flow's private class names (§5.4).
6. **Reduced motion** disables entrance transitions and bee/dash animation (V-10).
7. **No `dangerouslySetInnerHTML`; no hex literals** in any card (token module only; `noHardcoded.test.ts` enforces).

### 3.6 Event hydration

The store's `applyEvent` (R-06 `:1259-1660`) stays the only mutation path. The canvas adds a **projection** keyed on `graphRev`, so hydration is a pure function of server state.

| Event | Store effect (existing) | Canvas effect (new) |
|---|---|---|
| `RUN_CREATED` (new run) | clears tasks/history/snapshot (R-06 `:1321-1342`) | layout cache invalidated by `runId`; positions for the new run loaded from storage; camera recentred once |
| `NODE_SPAWN` | insert/merge record (R-06 `:1401-1452`) | node appears; entrance animation only at B2+ and only if motion allowed |
| `NODE_STATE` | merge `state` + `rev` (R-06 `:1420-1438`) | glyph swap; card re-renders only itself (S-1/S-2) |
| `NODE_REWIRE` | `parentId` = payload `parent_task_id` ?? `pendingRewire` ?? existing (R-06 `:1411-1422`); triggers one snapshot refresh when neither is known (R-06 `:1443-1451`) | edge re-routed once on drop; no per-frame re-route (CANVAS-MAP D5.5) |
| `NODE_DRAG` | **never emitted by the server** (G-04) — decoder exists for forward compatibility | no-op; positions are client-owned |
| `HISTORY_SNAPSHOT` / replay | `history.replayGraph` (R-06 `:1626-1641`) | read-only projection; non-interactive nodes (R-18 `:228-232`); inactive wires dimmed (Q8) |
| `RUN_METRICS` | `metrics` (R-06 `:1454-1457`) | HUD only — never node positions |
| `ERROR` | refusal toast (R-06 `:1268-1279`) | if correlated to an in-flight rewire, the card reverts and shows the refusal chip |
| `RESYNC_REQUIRED` | `resyncGap` + `connection: 'reconnecting'` (R-06 `:1311-1319`) | **drag commits disabled**; banner via `App.tsx:39-44`; gap clears only on a settled snapshot |
| `SERVER_HELLO` | replay window, approvals stale (R-06 `:1280-1310`) | no visual change except banner state |
| `ACK` | resolves the command promise (R-07 `:373-392`) | none |

**Sequence discipline (unchanged):** the client never synthesises an event, never advances `lastSeq` beyond what it received (R-06 `:1260-1264`), and treats a gap only when the *server* says so. Do not add a client-side "expected next seq" inference.

### 3.7 Selection and rewire

**Selection model (unchanged semantics, cheaper plumbing)**

- `selectedTaskId: string | null` remains the single selection in `UiState` (R-06 `:251`); `selectTask` unchanged (`:2123-2125`).
- Click a card → select; click pane → clear (R-18 `:350`); `Escape` → clear; keyboard: `Tab` reaches cards (`nodesFocusable`), `Enter`/`Space` selects (React Flow focus behaviour + card button).
- Selection is **not** geometry: it must never trigger layout (§3.3 L-3).

**Drag-to-rewire pipeline (P-2)**

1. `onNodeDragStart` → open a drag session in `dragStore` (dragged id, pointer, smoothed direction, candidate set).
2. `onNodeDrag` (rAF-coalesced) → `findSnapTarget({ draggedRect, candidates, scale, dragDir, currentParentId })`:
   - **gate**: `rectGap(dragged, candidate) ≤ 150 / scale` world px = **150 screen px at every zoom** (`ROUTING.snapRadiusPx`, R-19 `tokens.ts:127-129`);
   - **candidates**: `superagent` and `hyperagent` nodes only, excluding self, descendants, and the current parent (R-18 `:336-344` keeps this rule);
   - **ranking**: `align` first (`dot(unit(parentPort − dragOrigin), dragDir)`), then gap, then `parentId` lexicographic — a **total order**, floats rounded to 1e-6;
   - **hysteresis**: keep the current parent unless a rival wins by `> 0.12` align **and** `> 24` screen px;
   - **gate off below `scale 0.4`** (macro bands) with a hint line, never a silent no-op;
   - **slot choice**: nearest free slot of the chosen parent (12-slot lattice), `occupiedPenalty +2000`; if all 12 occupied → ineligible with `E_SNAP_SLOTS_FULL`-style copy (local, not a wire code);
   - **displacement preview**: BFS ≤3 rings for free slots of siblings, preview ≤3 ghosts via transform transition (120 ms; 0 ms reduced).
3. Preview is **visual only**. No store mutation, no command.
4. `onNodeDragStop` → commit if a target exists: call the additive store action `rewireTask(draggedId, targetId)` (P-1); else keep the new position and persist it.
5. `Escape` / `pointercancel` / window blur → cancel with no event.
6. On refusal (`E_CYCLE`, `E_RESYNC_REQUIRED`, `E_REWIRE_IN_FLIGHT`, `E_TASK_NOT_FOUND`): revert the card to its pre-drag position, show a chip on the card (`data-refusal="<code>"`) **and** the standard toast.

**CAS handshake (must not change).** `rewireTask` reuses `rewireBlocker` exactly as `rewireSelected` does today: `expected_rev` comes from `tasks[id].rev`; if `null`, one bounded `GET /api/history/runs/{id}/state` read fills it; if still unknown, refuse with `E_RESYNC_REQUIRED` and **do not send** (R-06 `:1822-1875`, `:631-700`; backend CAS at R-26 `graph.rs:152`). `rewireSelected` remains for `ControlBar.tsx:88` and delegates to `rewireTask(get().selectedTaskId, parentId)` — one code path, two entry points.

**Why an additive store action is required.** Today's canvas does `selectTask(node.id)` then immediately `rewireSelected(target.id)` and relies on zustand's synchronous `set` (R-18 `:346-347`). That is an implicit ordering dependency between two UI gestures: a future batched update, a concurrent selection from another panel, or a second pointer would silently rewire the wrong node. `rewireTask(taskId, parentId)` removes the coupling without changing the wire behaviour.

### 3.8 Persistence

| Data | Where | Schema | Rules |
|---|---|---|---|
| Node positions | `localStorage` | `hivemind.canvas.positions.v2:<runId>` → `{v:2, runId, updatedAt, positions:{[taskId]:{x,y}}}` | validate every number (`Number.isFinite`, `|v| ≤ 1e7`); cap **64 entries / 10 kB**; unknown ids ignored on read; corrupt JSON → `{}` and a DEV warning; quota/private-mode failure is silent-by-design **but** a `positionsSaved: false` flag drives a one-line hint |
| Viewport / camera | **not persisted** | — | V-9 |
| Graph topology | **server only** | — | never written by the canvas |
| UI preferences (hideFinished, focusBranch) | `localStorage` (P, optional) | `hivemind.canvas.prefs.v1` | validated booleans only |

**Migration.** v1 keys (`hivemind.canvas.positions.v1:<runId>`) are read once, validated, and rewritten as v2; the v1 key is then deleted. A failed migration leaves v1 untouched and starts empty (no data loss beyond the stale key).

**Why positions cannot be persisted server-side (verified).** There is no coordinate table in the Rust backend: `graph.rs` stores `TaskNode {task_id, parent_task_id, state, rev}` only (R-26 `graph.rs:18-28`); `VIEWPORT_SET`/`VIEWPORT_RECENTER` are explicit no-ops (R-15 `ws.rs:737-741`); and the server never emits `NODE_DRAG` (G-04). `graphcontract.json` proposes a `node_positions` table, but it is a **PROPOSED contract for a different stack** — `node:sqlite`, SSE, `/api/graph/workspaces/:id/...`, node kinds `hyper|super|worker|tool` (R-27) — and adopting it would mean adding tables, routes and a second transport to a verified backend. **Rejected**: positions stay client-side in v1. This is a product limitation, stated, not a silent gap.

### 3.9 Failure and refusal rendering

Every failure class has exactly one surface. No class renders as a generic error, and none renders invented state.

| Class | Trigger (verified source) | Surface | Copy source | Mutations |
|---|---|---|---|---|
| Transport down | `connection ∈ {error, closed}` (R-11 `App.tsx:39-44`) | full-width banner under the header | `STRINGS.connection.offlineBanner` (R-13 `strings.ts:55`) | all disabled |
| Reconnecting / resyncing | `connection === 'reconnecting'`, incl. an open socket with `resyncGap ≠ null` (R-06 `:1734-1741`) | same banner, distinct copy | `reconnectBanner` (`:56`) | rewire/cancel/spawn disabled; drag preview enabled, **commit disabled** |
| Command refusal | `ERROR` frame (R-06 `:1268-1279`) | toast, assertive, with `code`, `message`, `how_to_fix`, dismiss (R-14) | server payload + `clientRefusal` fallback | unchanged; the refused command did not mutate |
| Rewire-state refusal | `rewireBlocker` (`E_RESYNC_REQUIRED`, `E_CYCLE`, `E_REWIRE_IN_FLIGHT`) | toast **and** card chip `data-refusal`; card reverts | `rewireRefusal` (R-06 `:670-698`) | rewire stays disabled until a snapshot confirms `rev` |
| Per-node failure | task state ∈ {failed, killed, interrupted} | crimson card + broken crimson dashed wire + **paused** bee (no SMIL) | `STRINGS.node.failed/brokenRoute` | kill/retry stay available per contract |
| Degraded snapshot | `snapshot.status ∈ {loading, degraded, failed}` (R-06 `:183-194`) | thin strip above the canvas: "graph may be stale — resyncing" | new string (P) | drag commits disabled while `loading` |
| Replay mode (not an error) | `history.replayGraph !== null` | pill badge "REPLAY · STATE n OF m" (R-18 `:388-394`) + scrubber | `STRINGS.states.replay` | all mutations disabled; nodes non-draggable (R-18 `:228-232`) |
| Empty graph | `tasks` empty and no run | centred hint, not an error | `STRINGS.panels.agentsEmpty` (`:196`) | spawn disabled until a run exists |

**ARIA roles:** banner `role="alert"`; toast region `aria-live="assertive"` (R-14 `:14`); selection announced through the existing `aria-live="polite"` node (`R-18 :447-450`); the canvas stays `role="application"` with an accessible name.

### 3.10 What must NOT change (the verified seams)

| # | Seam | Evidence | Why it is frozen |
|---|---|---|---|
| N-1 | Envelope + handshake: `GET /api/session` → `ws?token&last_seq` → raw `{"type":"hello"}` → envelopes | R-07 `client.ts:258-320`; R-15 `ws.rs:469-533`; `UI-COMMANDS-v1.md:19-26` | the server's session loop expects hello first and times out otherwise (`ws.rs:404-414`) |
| N-2 | 14 commands / 24 events / 16 states / 65 refusal codes | R-16 `contract.ts:119-157, 385-410`; R-17; M-06 | frozen vocabulary; changing a name requires a `CHANGE-LEDGER-IMPL.md` row |
| N-3 | ACK-before-mutation; commands resolve only on ACK, reject on typed ERROR | R-07 `client.ts:373-501`; R-06 `:1650-1681` | the only honest success signal |
| N-4 | `expected_rev` from the committed snapshot, never a client counter | R-06 `store.ts:74-84, 1859-1871`; R-26 `graph.rs:145-156` | server CAS refuses stale revs with `E_RESYNC_REQUIRED` |
| N-5 | `NODE_REWIRE` parent learning + one bounded snapshot refresh | R-06 `:1411-1451` | the frozen event may omit the new parent; the edge can only be learned from a snapshot |
| N-6 | Resync gap semantics (`resyncGap` outranks an open socket) | R-06 `:258-259, 1311-1319, 1736-1740` | prevents presenting a lost range as Connected |
| N-7 | `/api/history/runs*` read routes | R-10 `http.rs:483-486` | the only rehydration authority |
| N-8 | Router shape + `rust-embed` of `web/dist` | R-10 `http.rs:478-496`; R-09 `static_ui.rs:13-16` | a build that does not land in `web/dist` never ships |
| N-9 | Token module is the only hex source | R-19; `web/tests/noHardcoded.test.ts`; `web/tests/tokens.test.ts` | WCAG corrections live there (`CORRECTIONS`, `tokens.ts:72-79`) |
| N-10 | `LAYOUT` geometry pins | R-19 `tokens.ts:166-179`; `web/tests/designParity.test.ts:16-33` | board-07 fidelity contract |
| N-11 | Dev-only mock boundary (`VITE_UI_MOCK`, dynamic import) | `web/src/api/index.ts:32-80`; `web/tests/apiBoundary.test.ts` | a production bundle must not ship a test double |
| N-12 | Playwright browser policy (installed channels, no download, no live paid run) | R-22; `e2e/README.md:12-22` | licence/cost/host constraints |

**Explicitly *not* frozen:** the canvas renderer's internal structure, `dagre`, the `viewport` store field, the screen-space honeycomb, the intersection-based rewire. Those are §6 targets.

---

## 4. Dependency decision table (no installs performed)

Licences below are **read from the installed package's own `package.json`** (M-02) unless marked otherwise. "Maintenance" is a dated judgement, not a fetched fact (**U-03**).

### 4.1 KEEP (already installed, exact-pinned)

| Package | Version | Licence (verified) | Build risk | Why keep |
|---|---|---|---|---|
| `react`, `react-dom` | 19.3.1 | MIT | low | the app; no server components needed |
| `@xyflow/react` | 12.12.1 | MIT | low–medium: owns pan/zoom/drag/minimap/a11y and 7 e2e specs; private class names pin us to a major | §2.5: cheapest faithful path; replacing it costs more than it saves |
| `@xyflow/system` (transitive) | 0.0.84 | MIT | — | d3-zoom based viewport |
| `zustand` | 5.0.16 | MIT | low | existing store; vanilla `createStore` already used |
| `partysocket` | 1.3.0 | MIT | low | reconnect/backoff already tuned (R-07 `:111-118`) |
| `motion` | 14.9.1 | MIT | low | one toast animation; keep, and reuse for card/snap transitions instead of adding a second animation lib |
| `tailwindcss`, `@tailwindcss/vite` | 4.3.4 | MIT | low | the token `@theme` bridge |
| `vite` | 8.3.6 | MIT | low | already the build |
| `typescript` | 7.0.2 | **Apache-2.0** | medium: a new major line; semantics of strict flags must be re-confirmed on upgrade | pinned; no upgrade without a gate run |
| `vitest` | 5.1.0 | MIT | low | 182 existing tests |
| `@vitejs/plugin-react` | 6.2.1 | MIT | low | — |

### 4.2 KEEP-THEN-REMOVE

| Package | Version | Licence | Decision | Reason |
|---|---|---|---|---|
| `@dagrejs/dagre` | 3.1.3 | MIT | **remove after WP-2** | measured p50 23.5–319.9 ms per call when placed in the render path (M-09); its DAG semantics contradict the hive slot lattice (§3.3); `MAX_FANOUT = 8 ≤ 12` slots makes it unnecessary. Removal must be recorded as a dependency change with the new layout test evidence (§28 of the operating contract). |

### 4.3 ADD (proposed — none installed by this document)

| Package | Licence | Status | Build impact | Why / alternative |
|---|---|---|---|---|
| `@fontsource-variable/inter`, `@fontsource/playfair-display` | OFL-1.1 (**U-04**, not verified offline) | **recommended** | +~120–200 kB woff2 (E-05), hashed assets, `font-display: swap` | the design's display/UI faces are declared but never loaded (§1.2). Alternative — system stack — is free but abandons board-02 typography. Decisive check: licence file present in the published tarball, and a visual diff of the type board. |
| `jsdom` + `@testing-library/react` | MIT (**U-04**) | **conditional, deferred** | dev-only, no runtime bytes | only if a component invariant cannot be proven by Playwright. Current posture (node-env vitest + Playwright for DOM) is deliberate (R-08) and should not be widened without a failing test that *needs* it. |
| `axe-core` | MPL-2.0 (**U-04**) | **rejected for now** | dev-only | cannot be installed offline from a verified source here; §5.6 covers the same ground with explicit ARIA assertions plus the existing parity tests. |

### 4.4 REJECT

| Candidate | Licence (claimed) | Why rejected |
|---|---|---|
| `pixi.js` | MIT | §2.4: text, a11y, context-loss, +≈400–450 kB (E-04), and the backend caps at 64 nodes — no requirement gained |
| `three.js` / `@react-three/fiber` | MIT | 2-D design; no 3-D requirement |
| `konva` / `react-konva` | MIT | same text/a11y problem as Pixi, plus a second scene graph alongside React Flow |
| `fabric` | MIT | canvas-only object model; would duplicate every layer |
| `d3` (direct), `d3-zoom` | ISC | already vendored inside `@xyflow/system`; a second copy risks two competing transform owners |
| `react-zoom-pan-pinch`, `react-flow-renderer` (old name) | MIT | duplicate/replaced by the installed Flow |
| `react-virtualized`, `virtua`, `@tanstack/virtual` | MIT | `onlyRenderVisibleElements` + a 64-node cap make list virtualisation pointless |
| `zod` | MIT | the contract decoder is hand-written and *frozen*; a second validator would drift from `hiverouter-contract` |
| `xstate` | MIT | the interaction model is a handful of booleans plus the server state machine; a statechart adds ceremony without removing a class of bug |
| `@tanstack/react-query` | MIT | transport is push (WS events), not request/response caching |
| `radix-ui` / `shadcn` | MIT | the design system is bespoke tokens (honey/ink) with exact geometry pins; importing a component system fights `designParity.test.ts` |
| `framer-motion` (as a second lib) | MIT | `motion` is already installed; never ship both |

---

## 5. Testing strategy

The current harness is **vitest in a node environment** (R-08) plus **Playwright against the shipped binary** (R-22). That split is correct and should not be widened casually: geometry and state are pure and cheap to test; DOM behaviour belongs to Playwright against Chrome.

### 5.1 Unit / property (vitest, node env, `web/tests/*.test.ts`)

| Target | Tests (P) | Gate |
|---|---|---|
| `layout/layout.ts` | root centre is exactly `(0,0)`; same input → identical positions (10 shuffles of insertion order); no two nodes overlap; fan-out 8 fits; orphan re-rooted; 64-node fixture completes | **≤ 5 ms p50** on the 64-node fixture (M-09 projection was 0.04–0.29 ms; layout must stay in the low ms) |
| `topologyKey.ts` | key changes on spawn/rewire/state-change-of-parent; key is **stable** across pure state changes (state must not invalidate layout) | exact string equality |
| `geometry/routing.ts` | exactly 2 segments; every segment on 0°/±60°; tie-break total order; deterministic under 20 shuffles; degenerate zero-length route returns a single point without NaN | assertion + fuzz over 10 000 random rects |
| `geometry/snap.ts` | gate is **150 screen px at scale 0.1/0.4/1/3** (world threshold scales as `150/scale`); candidate cap 24; hysteresis; slot capacity refusal; displacement BFS ≤3; **determinism under 20 parent-insertion shuffles** (the CANVAS-MAP D6 property) | p95 ≤ **0.5 ms** per query in-browser (§5.5), ≤ 0.05 ms in node |
| `geometry/lod.ts` | band edges 0.20/0.35/0.60/1.50; ±4 % hysteresis prevents flapping at each edge; 0.1 and 3.0 clamp | boundary table |
| `persistence/positions.ts` | v1→v2 migration; corrupt JSON → `{}`; non-finite/oversized values dropped; quota throw handled; cap enforced | no throw on any hostile input |
| `projection/nodes.ts` | identity stability: unchanged task ⇒ **same object reference**; changed `rev` ⇒ new object; selection change ⇒ **no new objects** | reference equality |
| `flow/graph.ts` | existing `graph.replay.test.ts` (7 tests) must keep passing; re-pin the dagre-mutation regression (`:146-157`) to the lattice layout | green |

Run: `npm.cmd run test` (M-08 baseline: 22 files / 182 tests).

### 5.2 Event parity (contract ↔ store ↔ projection)

| Test | What it pins | Evidence base |
|---|---|---|
| Vocabulary freeze | `EVENT_NAMES`, `FROZEN_COMMAND_NAMES`, `SERVER_COMMAND_NAMES`, `AGENT_STATES` lengths (24/14/12/16) and the 65 refusal codes | `contract.test.ts`, `refusalParity.test.ts`, `state.parity.test.ts` (already exist) |
| Golden replay | a recorded envelope log (from `e2e/helpers/ws-tap.ts`) replayed through `applyEvent` produces the same `tasks` + `graphRev` + projection as the live capture | new `canvas.hydration.test.ts` (P) |
| Reconnect | `SERVER_HELLO` after `RESYNC_REQUIRED` **must not** present Connected; a settled snapshot clears the gap | mirrors `store.ts:1280-1319` |
| Rewire learning | `NODE_REWIRE` with and without `parent_task_id`; the `pendingRewire` fallback is consumed once | mirrors `store.ts:1411-1451` |
| No invented frames | the projection never adds a node id absent from `tasks`; every edge endpoint exists | mirrors `graph.replay.test.ts:83-88` |

### 5.3 Visual (Playwright `toHaveScreenshot`, committed baselines)

| Shot set | Viewports | What it proves |
|---|---|---|
| 5 zoom bands (0.1 / 0.25 / 0.5 / 1.0 / 2.0) on a deterministic 12-node fixture | 1440×900 | LOD banding, comb crispness, macro silhouettes |
| Node anatomy × 5 kinds, selected / unselected / failed | 1440×900 | card fidelity, crimson broken route, paused bee |
| Drag preview (ring + guideline + 2 ghosts) | 1440×900 | Prompt-5 affordance |
| Refusal toast + card chip | 1440×900 | failure rendering |
| Replay state (dimmed inactive wires + badge) | 1440×900 | Prompt-10 |
| Responsive shell | 1280, 1440, 1920 | no overlap of HUD/minimap/zoom/scrubber |

Rules: baselines only from a **deterministic fixture** (fixed ids, fixed order, animations disabled via `prefers-reduced-motion` emulation), and a diff must be attached to evidence. **No baseline is accepted from a live provider run** (no paid API in e2e, `e2e/README.md:14-16`).

### 5.4 Playwright behaviour (installed Chrome/Edge only)

| Test | Assertion |
|---|---|
| Pointer-anchored zoom | world point under the cursor is stable within 0.5 px across 0.1↔3.0 (compute via the viewport transform string) |
| Ctrl+0 | returns to origin **without** changing zoom; does not hijack a focused input (existing `gui-gates.spec.ts:98-123` must stay green) |
| Drag-snap | dragging a worker within 150 screen px of a superagent shows the ring + guideline; releasing **outside** the gate does **not** send `REWIRE_TASK` (assert via WS tap) |
| Rewire commit | release inside the gate sends exactly one `REWIRE_TASK` with `expected_rev` equal to the last `NODE_*` rev |
| Refusal path | force `E_CYCLE`/`E_RESYNC_REQUIRED` and assert: toast with code + `how_to_fix`, card reverts, no second command |
| Replay | entering replay disables drag; node counts match the snapshot; exit restores live graph |
| Surface census | S01–S28 + S-APPROVAL present — using **`data-surface`** attributes, not React Flow classes (`e2e/specs/trace-hops.spec.ts` re-pinned) |
| Offline | stop the server → banner, mutations disabled, no fabricated nodes |
| Keyboard | Tab reaches cards, Enter selects, Escape clears, focus ring visible (`2px ink-700`) |

Command: `cd e2e; $env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'; npm.cmd run e2e` (R-22; server started by the build, never by Playwright).

### 5.5 Performance budget (gates, not claims)

All numbers are **ESTIMATES until recorded**; each gate stores the trace that settles it. Reference: the 64-node worst case (§2.2) and a 200-node *synthetic* fixture used only to prove headroom (E-06).

| Gate | Measurement | Threshold | Fallback if failed |
|---|---|---|---|
| **P-1 layout** | `computeLayout` on 64 nodes, node microbench, 200 runs | p95 ≤ **5 ms** | cache by `topologyKey` (already), precompute off the event handler, or reduce slot search |
| **P-2 projection** | `projectNodes` identity + object build | p95 ≤ **0.5 ms** | memo per id |
| **P-3 interaction** | in-browser snap query p95 per pointermove | ≤ **0.5 ms** | candidate cap 24→12→8; drop the displacement BFS scoring pass |
| **P-4 pan** | 3 s continuous pan at B3 and a 0.1↔3.0 sweep: frame interval distribution and long tasks | **0 long tasks > 50 ms** attributable to canvas JS | disable `onlyRenderVisibleElements` churn, batch edges, demote band |
| **P-5 memory** | 10-minute pan/zoom session, heap snapshots | no unbounded growth | shrink comb tile table; dispose drag store on unmount |
| **P-6 build** | `vite build` artifact sizes | initial chunk ≤ **750 kB** *or* Canvas lazy-loaded into its own chunk | `manualChunks`: `vendor-react`, `vendor-flow`, `app`; `React.lazy` the canvas |

**No fps claim.** The UI may be called "smooth" only after P-4's trace exists and is linked in the evidence file.

### 5.6 Accessibility gates

- Contrast comes from `tokens/contrast.ts` + `tokens.test.ts` (existing); the six declared rows must keep their computed ratios (e.g. `amber` on `canvas` = 3.32 → large text only, `tokens.ts:62-79`).
- Every card is keyboard reachable and has an accessible name (`ariaLabel` = label + kind + state).
- Reduced motion: both the OS preference and the in-app switch produce `animation-duration: 0.001ms` (R-04 `:100-118`); the canvas must additionally **not start** SMIL when reduced.
- Focus ring is `ink-700` at 2 px (the corrected value, `tokens.ts:72-79`).
- Live regions: selection (`polite`), refusals (`assertive`), connection banner (`alert`).

### 5.7 What must never be asserted

- A frame rate as a pass/fail.
- A pixel-perfect position of a node against a FigCraft export (boards are text-layer; fidelity is **UNVERIFIED/needs_human**, `web/src/assets/manifest.ts:37-40`).
- A specific number of DOM nodes as a proxy for performance.
- Any behaviour of a live provider (no paid run in e2e).

---

## 6. Implementation sequence (decision-complete)

Each work package (WP) has an exit gate. No WP touches the backend or the frozen contract. "Rollback" is the inverse operation the implementer must be able to perform (§20 of the operating contract).

| WP | Change | Exit gate | Rollback |
|---|---|---|---|
| **WP-0 — Baseline freeze** | Record `git rev-parse HEAD`, the §0.2 hashes, `npm.cmd run typecheck|test|build` output, and a 64-node fixture generator (test-only). No product edit. | artifacts committed under `docs/trace/ui-rebuild/`; dist hash recorded | delete the record |
| **WP-1 — Pure geometry** | Add `canvas/geometry/{ports,routing,snap,lod}.ts` + `canvas/layout/layout.ts`; unit tests per §5.1. Nothing imported by the app yet. | new tests green; `layout` p95 ≤ 5 ms at 64 nodes; snap determinism under 20 shuffles | delete files (no app import) |
| **WP-2 — Layout out of render** | `graph.ts: buildNodes` stops calling `layoutTasks`; `Canvas.tsx` memoises on `topologyKey`; selection removed from `data`; stable projection (§3.3/§3.4). | `graph.replay.test.ts` green; clicking a card no longer re-runs layout (prove with a counter test); p50 selection→paint measurement recorded | restore `layoutTasks` call (one-line revert) |
| **WP-3 — Viewport hygiene** | Delete `UiState.viewport`/`setViewport`; add `viewportStore` (band + zoom label); `minZoom/maxZoom` from `contract.ts`; pointer-anchored zoom test. | typecheck green (no dangling `viewport` refs); e2e Ctrl+0 + zoom still green; no `setState` in `onMove` | re-add `viewport` field (dead-state revert) |
| **WP-4 — LOD + culling** | `lod.ts` wired; `onlyRenderVisibleElements`; `MacroLayer` for B0/B1; band-swapped `nodeTypes`. | visual baselines at 5 bands; P-4 trace ≤ threshold; silhouette rect count == node count | force `band = B3` feature flag |
| **WP-5 — Card fidelity** | Band-aware anatomy, `data-surface`, selection via context, reduced-motion entrance, `LAYOUT` sizes everywhere. | visual + a11y gates; `noHardcoded`/`designParity` green | revert `nodes.tsx` (single file) |
| **WP-6 — Snap/rewire** | Add store action `rewireTask` (P-1); wire the drag pipeline (§3.7); card chip refusal; WS-tap assertions. | pointer-gate e2e (in/out of 150 px), single `REWIRE_TASK` with correct `expected_rev`, refusal path | drop `rewireTask`, restore `selectTask`+`rewireSelected` call site |
| **WP-7 — Routing + bee** | `BeeEdge` consumes `routing.ts`; SMIL dash + `animateMotion`; pause on broken/reduced; world-anchored comb (`WorldComb`). | route unit tests; visual at 3 zooms; reduced-motion freezes bees | revert `BeeEdge.tsx` + comb to screen-space |
| **WP-8 — Persistence v2 + replay dimming** | `persistence/positions.ts`, v1→v2 migration, caps; replay inactive wires at 40 % opacity; degraded-snapshot strip. | persistence unit tests (corrupt/quota/migration); replay visual; offline e2e | keep v1 reader, ignore v2 key |
| **WP-9 — Gates + docs** | Full run: typecheck, 182+ tests, build, e2e, perf traces, visual baselines; update `FRONTEND-STATUS`, claims register, `CHANGELOG`. | every gate quoted with command + output; UNVERIFIED list updated honestly | revert docs commit |

Sequencing constraints: WP-1 → WP-2 → WP-4 (LOD needs the projection), WP-3 independent of WP-4, WP-6 depends on WP-1 (snap) and WP-5 (card chip), WP-7 independent of WP-6, WP-9 last. **No WP may begin before the previous one's exit gate is quoted.**

---

## 7. UNVERIFIED register (and the decisive experiment for each)

| # | Claim | Why unverified | Decisive experiment |
|---|---|---|---|
| U-01 | `web/dist` corresponds to HEAD | no build was run (it would rewrite the embedded artifact other seats are using) | `npm.cmd run build`, compare hashes and bundle sizes, then rebuild the Rust binary |
| U-02 | Browser e2e is green today | requires the built binary on `127.0.0.1:8799` + `E2E_APP_READY=1` | run `e2e/run-guarded.mjs` against a fresh release binary |
| U-03 | Maintenance status of every dependency | not fetched from official registries in this session | check each project's releases/changelog at implementation time |
| U-04 | Licences of the **proposed** packages (`@fontsource*`, `jsdom`, `@testing-library/*`, `axe-core`) | not installed; read from memory only | read the licence file in the published tarball before adding |
| U-05 | Browser frame times at any band | no trace was recorded | Playwright trace / DevTools Performance for a 3 s pan per band |
| U-06 | Text re-raster quality at fractional CSS scale (0.6×/1.3×/2.9×) | visual only | screenshots at each zoom; if blurry, re-render card text at integer screen sizes per band |
| U-07 | `onlyRenderVisibleElements` behaviour with 64 measured nodes | not exercised | count `.react-flow__node` at 0.25× with/without the flag |
| U-08 | SMIL `<animateMotion>` support and cost in the target Chrome/Edge | not measured | feature-detect + trace with bees on/off; fallback = static bee at 55 % of the route |
| U-09 | Whether React Flow 13 will keep `.react-flow__*` class names / `data-id` | future release | pin 12.12.1; adding `data-surface` in WP-5 makes the migration a selector swap |
| U-10 | Whether 64 nodes is the true UI maximum in every path | derived from `caps.rs` only; a template insert could differ | count nodes in a real template-insert run; assert `≤ MAX_AGENTS` |
| U-11 | Runtime cost of the world-anchored comb inside `.react-flow__viewport` | not measured | trace pan/zoom at 0.1× and 3.0×; fallback = screen-space pattern updated on `moveEnd` |
| U-12 | The exact snap feel (150 px + hysteresis 0.12/24 px) matches the design intent | design intent is prose, not a spec number | usability pass with 3 operators; tune constants, keep the pure function's contract |
| U-13 | `graphcontract.json` applicability | it is PROPOSED and targets a different stack (Node/SQLite/SSE) | **reject** unless a backend decision adds coordinate tables; re-read its `status` field |

---

## Appendix A — Benchmark method and raw output

Purpose: separate **layout** cost from **projection** cost, because the current code conflates them (`graph.ts:85`).

Method: Node script that reproduces `layoutTasks` + `buildNodes` from `web/src/flow/graph.ts` exactly (same dagre options `rankdir LR, nodesep 40, ranksep 90, margin 40`, same size table), run against the **installed** `@dagrejs/dagre@3.1.3` (`web/node_modules/@dagrejs/dagre/dist/dagre.esm.js`). Fixture: one root, `round(N/12)` superagents, the rest workers round-robin parented. 15–25 iterations, median reported. Nothing was installed; the script was written to `%TEMP%` and deleted.

```
M-09 (500 nodes, 40 parents, 25 runs)
{ "nodes": 500, "parents": 40, "runs": 25,
  "p50_ms": 324.62, "p95_ms": 464.61, "min_ms": 261.01, "max_ms": 473.91 }

M-09b (scaling, 15 runs each, p50)
nodes   dagre_p50_ms   project_only_p50_ms
   50        23.48              0.04
  100        39.61              0.06
  200        93.38              0.11
  500       319.92              0.29
```

Interpretation (E-07): the projection is ~1 000× cheaper than the layout at 500 nodes. At the **64-node server cap** the layout is ≈ 25–30 ms — one dropped frame **per selection** today, because `buildNodes` is re-invoked whenever `selectedTaskId` changes (`Canvas.tsx:155`). This is the measured justification for WP-2.

Caveat: these are Node microbenchmarks of pure functions on an i5-3470S; they are **not** browser rendering measurements. Browser overhead is **U-05**.

## Appendix B — Proposed module signatures (P)

```ts
// canvas/layout/layout.ts
export interface Point2 { readonly x: number; readonly y: number }
export function computeLayout(tasks: Readonly<Record<string, TaskRecord>>): Readonly<Record<string, Point2>>
export function topologyKey(tasks: Readonly<Record<string, TaskRecord>>, runId: string | null): string

// canvas/geometry/ports.ts
export type Kind = 'hyperagent' | 'superagent' | 'worker' | 'tool' | 'error'
export function portsFor(kind: Kind, rect: Rect): { readonly left: Point2; readonly right: Point2; readonly top: Point2; readonly bottom: Point2 }

// canvas/geometry/routing.ts
export interface Route { readonly points: readonly [Point2, Point2, Point2]; readonly headingRad: number }
export function routeBetween(source: Point2, target: Point2): Route

// canvas/geometry/snap.ts
export interface SnapInput {
  readonly dragged: { readonly id: string; readonly rect: Rect; readonly kind: Kind }
  readonly candidates: readonly { readonly id: string; readonly rect: Rect; readonly kind: Kind; readonly occupied: number }[]
  readonly scale: number
  readonly dragDir: Point2 | null
  readonly currentParentId: string | null
  readonly nowMs: number
}
export interface SnapDecision {
  readonly parentId: string
  readonly slotIndex: number
  readonly ghosts: readonly { readonly taskId: string; readonly to: Point2 }[]
  readonly align: number
  readonly gapScreenPx: number
}
export function findSnapTarget(input: SnapInput): SnapDecision | null
export const SNAP_GATE_SCREEN_PX = 150 // ROUTING.snapRadiusPx

// canvas/geometry/lod.ts
export type Band = 'B0' | 'B1' | 'B2' | 'B3' | 'B4'
export function bandFor(scale: number, previous: Band | null): Band

// canvas/persistence/positions.ts
export function loadPositions(runId: string | null, storage: Storage | null): Record<string, Point2>
export function savePositions(runId: string | null, positions: Record<string, Point2>, storage: Storage | null): boolean

// state/store.ts (additive only)
rewireTask(taskId: string, newParentId: string): Promise<void>
```

## Appendix C — FigCraft board → surface → implementation map

| Board (HiveMind) | Surfaces | Implementation after §6 |
|---|---|---|
| 07 Spatial Canvas | S01 canvas, S02 hive chat core, S13 minimap, S20 zoom | `CanvasHost`, `ChatCore` (pinned overlay), `MiniMapPanel`, `ZoomControls` |
| 05 Agent Cards | S15–S19 | `nodes/*` per band (§3.5) |
| 06 Controls & HUD | S09 HUD, S10 scrubber, S05/S06/S07 controls | `HudBadge`, `Scrubber`, `ControlBar` (unchanged contracts) |
| 10 States & Refusals | S14 refusals, S18 broken route | `ToastRegion`, `error` node, `BeeEdge` broken path |
| 08/09 HMAS hierarchy & governance | S11 agent hierarchy, approval overlay | `AgentHierarchy`, `ApprovalOverlay` (unchanged) |
| A foundations/colour/typography | tokens | `tokens.ts` + (proposal) bundled faces |
| B canvas/zoom/interaction | Q1–Q7 evidence boards | reference only — **no pixel-parity claim** |

## Appendix D — Claims register for this document

| Claim | Class | Source |
|---|---|---|
| Toolchain versions match `package.json` | M-02 | `node_modules/*/package.json` |
| 22 test files / 182 tests pass; typecheck exit 0 | M-07, M-08 | command output |
| dist is one 724 133 B JS chunk + 40 640 B CSS | M-05 | `web/dist/assets` |
| dagre p50 23.5/39.6/93.4/319.9 ms at 50/100/200/500 nodes | M-09 | Appendix A |
| Server caps a run at 64 agents, fan-out 8, depth 3 | R-23 | `caps.rs:9-15` |
| Server never emits `NODE_DRAG` | G-04 | grep over `backend/crates/**/*.rs` |
| `VIEWPORT_*` are server no-ops | R-15 | `ws.rs:737-741` |
| 7 e2e specs assert React Flow DOM classes | R-24 | `e2e/specs/*` |
| Fonts declared but never loaded | G-03, R-12 | `tokens.css:30-31`, `package.json`, `index.html` |
| `viewport` store field has no readers | G-06 | grep over `web/src` |
| `graphcontract.json` targets a different stack and is PROPOSED | R-27 | its `contract`/`status`/`transport`/`node_kinds` fields |
| Browser performance is unknown | U-05 | no trace recorded |

