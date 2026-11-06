# FRONTEND STATUS — HiveMind v1 web (owned `web/**`)

Updated: 2026-11-06 04:27 CST · Owner: FRONTEND seat · Repo: `C:\Users\Riley\Desktop\hiverouter-v1`

## What exists now (milestone 1 — foundation)

| Area | Files | State |
|---|---|---|
| Toolchain | `web/package.json`, `web/vite.config.ts`, `web/tsconfig.json`, `web/index.html` | Vite 8.3.6 · React 19.3.1 · TS 7.0.2 · Tailwind 4.3.4 · @xyflow/react 12.12.1 · zustand 5.0.16 · @dagrejs/dagre 3.1.3 · motion 14.9.1 · partysocket 1.3.0 (all exact-pinned). Dev server 127.0.0.1:5173 strictPort; build target es2022; vitest included. |
| Tokens | `web/src/tokens/tokens.ts`, `web/src/tokens/contrast.ts`, `web/src/styles/tokens.css` | Exactly **20 PDF swatches** with `pdfLine` provenance (PDF:21-78); 8 printed type steps; 4px spacing stops; radius stops {4,8,12,16,24,48,999} + exceptions 10/14; elevation single warm-ink value with tiering marked UNVERIFIED; six declared contrast rows; corrections **meta → ink-500 #75663F**, **focus ring → ink-700**, **amber large-only**; WCAG contrast maths (pure). |
| State vocabulary | `web/src/state/agentState.ts` | One enum with **15 states** (≥13 contract): healthy/running/queued/failed/completed/cancelled/killed/planning/thinking/awaiting_approval/streaming/reconnecting/replay/empty/loading; dot mapping (4 PDF dots) + shape/text cues for all others; transition matrix; guards. |
| Wire contract | `web/src/api/contract.ts` | Frozen `hiverouter.v1` envelope; 10 frozen commands + 3 decision-gated DIFF names; 18 events; ACK/ERROR payloads; usage fields; **40 refusal codes** from UI-COMMANDS-v1 §5; `validateEnvelope`, `parseEventFrame`, `decodePayload` (typed events), command/envelope builders, prompt/zoom validation, `HelloFrame` for the Phase-2 `ws?token&last_seq` handshake. |
| Strings | `web/src/state/strings.ts` | Single string table (R-U13) incl. nav/tabs/HUD/chat/composer/library/templates/scrubber/settings/panels/states/agent-state labels/client refusal copy; formatters (`formatCount/Percent/Delta/Ctx/StateOf`); no copy hardcoded in components so far. |
| Shell | `web/src/main.tsx`, `web/src/App.tsx`, `web/src/styles/index.css` | Honest chrome: 72px nav rail (6), workspace tabs (4), HUD badge (NULL glyph `—`, never 0), canvas field w/ SVG honeycomb, pinned chat-core placeholder, 260px library, scrubber footer, aria-live toast region, skip link. No fabricated data anywhere yet. |

## Commands run (quoted output)

```text
> cd C:\Users\Riley\Desktop\hiverouter-v1\web
> npm.cmd install --no-fund --no-audit
added 86 packages in 2m
exit: 0

> npm.cmd run typecheck
> hiverouter-web@0.1.0 typecheck
> tsc --noEmit
exit: 0

> npm.cmd run build
> hiverouter-web@0.1.0 build
> tsc --noEmit && vite build
vite v8.3.6 building client environment for production...
transforming...
✓ 17 modules transformed.
dist/index.html                   0.52 kB │ gzip:  0.32 kB
dist/assets/index-ioHt9DQ5.css   15.13 kB │ gzip:  3.96 kB
dist/assets/index-By7KR63R.js   231.62 kB │ gzip: 73.06 kB
✓ built in 453ms
exit: 0
```

Disk free after install: **7.43 GB** (floor 1.2 GB not approached). Node v26.7.0 satisfies the blueprint floor (≥22.12.0); npm 11.19.0.

## Provenance / constraint compliance so far

- FigCraft assets **not consumed**: `ledgers/ui-watchdog.json` does not exist and no `allow_frontend` flag is present, so the shell uses PDF-derived token values only; the `web/src/assets/manifest.ts` seam is the next file in this batch and will mark assets `PENDING-UI-WATCHDOG` with no fabricated provenance.
- No backend/blueprint/ledger edits; all files above are inside `web/**`.
- Display face decision: Playfair Display (PDF:97-120); Inter for UI; mono = Consolas (PDF:127 host-resolved). Font bundling via @fontsource (OFL) will be added with the component batch and recorded in evidence; if unavailable offline the fallback stack is explicit and labelled UNVERIFIED, never silently "resolved".
- No pixel-layout claims: PDF boards are text-layer; visual fidelity stays UNVERIFIED/needs_human.

## Next step (in progress, same run)

1. `web/src/api/{client,mock,index}.ts` — typed WS client: `/api/session` token+`last_seq`, `ws?token&last_seq`, raw `{type:"hello",last_seq}` then envelope frames; ACK-before-mutation promise map; typed ERROR → refusal; reconnect/resnapshot; dev-only mock behind the same interface (no production fake success).
2. `web/src/state/store.ts` + `metrics.ts` — zustand run/task/chat/approval/history/viewport state; command lifecycle gated by ACK; named refusals; derived HUD counts (NULL ⇒ `—`); 13+ state enum applied to nodes.
3. React Flow canvas (@xyflow/react 12.12.1): honeycomb vector field, 4 node types + broken-route error card, bee-direction edges, Ctrl+0 recentre, explicit minZoom 0.1/maxZoom 3.0, minimap, zoom controls, keyboard/focus a11y.
4. Remaining 28 surfaces (S01–S28): chat core + composer + streaming, spawn/re-route/cancel/kill buttons, tabs, HUD, scrubber, nav rail, library, toast/refusal, agent chips, settings/key status, file-tree unavailable state, approval request (contract gap recorded), decision-gated diff modal, state pack, project identity, reduced motion.
5. `web/tests/*.test.ts` — vitest: 20-swatch/token provenance, contrast corrections, enum ≥13 + dot mapping, contract names/refusals/envelope parse, ACK/refusal client paths, keyboard Ctrl+0/zoom clamp, derived HUD nulls, no-hex/no-hardcoded-copy scans.
6. `docs/plans/PHASE-7-8-10-RECON.md` + `PHASE-7-8-10-EVIDENCE.md`; claim-register update; `npm.cmd run typecheck|build|test` quoted; git milestone commit if the repo permits.

## Blockers / open items (not hidden)

- FigCraft A assets cannot be consumed until `ledgers/ui-watchdog.json` `allow_frontend` is true; `docs/ui/figcraft/ASSETS-INDEX.json` does not exist yet. Seam + UNVERIFIED asset state only.
- Approval resolution has **no frozen command** in UI-COMMANDS-v1 (events exist, no response command) — the approval surface will render the request and record the gap; resolving would require a contract change.
- Workspace file-tree API (S22) is absent from the v1 server routes (`/health`, `/api/session`, `/ws` only) — panel renders an honest "unavailable" state, not a fake tree.
- Live provider/key status has no dedicated endpoint; Settings reads `/health` checks and marks unprobed state, never "present" without evidence.
