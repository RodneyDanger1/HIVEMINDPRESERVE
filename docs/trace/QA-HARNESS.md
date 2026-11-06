# QA HARNESS — HiveMind v1 (QA / trace engineer)

> **Dated correction — W9 trace-truth (2026-11-06T14:10:40-0600, HEAD `8a5b583`).** Recomputed from `docs/trace/controls.json`:
> **49 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 1 UNVERIFIED of 319 hops**; 55 hops carry evidence
> (non-PENDING), 54 evidence-backed excluding the UNVERIFIED marker. This supersedes the earlier
> `43 WIRED / 270 PENDING / 6 MISSING` figures (which counted a stale pre-fix `S22 6_service=MISSING` and did not
> include the 1.1 wave-2 evidence). Applied evidence: W3 streaming tests (S27 6_service/7_store/8_provider/9_response),
> W4 replay browser proof + source (S10 2_handler WIRED, 10_render_back UNVERIFIED), S22 6_service fixed from the
> tracked live probe. No hop was upgraded without a repo pointer; `delivered[]` stays empty (no fabricated frames).
> Recompute: `python -c "import json,collections;d=json.load(open('docs/trace/controls.json'));c=collections.Counter(h['verdict'] for x in d['controls'] for h in x['hops'].values());print(dict(c))"`




**Status (historical):** HARNESS LIVE — FINAL RUN 79 PASSED / 0 FAILED / 14 SKIPPED (exit 0, 2026-11-06 09:21).
Playwright **1.65.0** is installed in `e2e/` (package only; installed Chrome channel — no browser download).
Full Chrome suite against `http://127.0.0.1:8799` (binary `184D51A7…C9A9`, asset `index-Cbfm1IuM.js`, PID 33580): **93 tests =
79 passed, 0 failed, 14 skipped** (run log `e2e/test-results/run-live-FINAL-artifact.log`, HTML report
`e2e/playwright-report/index.html`). Earlier pre-fix failures (SETTINGS_STATUS E_INTERNAL_ERROR, stale
Data/Approval copy, FILE_TREE_RESULT rendering) were fixed by the build owners and re-verified. Live evidence:
`docs/trace/live/` (see `docs/trace/live/README.md`); trace capture refreshed 15:03Z and evidence-backed verdicts
applied to `controls.json` (49 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 1 UNVERIFIED of 319 hops (54 evidence-backed excluding the UNVERIFIED marker)). Static harness still 27/27.
Skipped/fixme tests are explicitly **not** passes; node-card and streaming surfaces stay UNVERIFIED (no run).

## 1. Role & scope

| Item | Value |
|---|---|
| Role | QA / trace engineer (legacy watchdog task closed) |
| Workspace | `C:\Users\Riley\Desktop\hiverouter-v1` (git `master` @ `d1638f2`) |
| Write scope | `e2e/**`, `docs/trace/**`, `docs/audit/QA-*.md`, `tools/qa/**` — nothing else |
| Explicitly NOT edited | `web/src/**`, `backend/**`, FigCraft assets (`docs/ui/figcraft/**`), blueprint (`hivemind-blueprint/**`) |
| Skills loaded | `trace`, `capability-auditor`, `evidence-check`, `mermaid-skill` (all loaded; no substitution). The program's `docs/claims/SKILL-USE.md` is outside this write scope — the Lead/owner must add the four skill-load lines there. |

## 2. Sources read (this session)

`EXECUTION-PROMPT.md` §0-§11 (incl. §7 mandatory verification at :173-185),
`blueprint/HIVEMIND-BLUEPRINT-v1.md` Phases 7/8/10/11 + §5/§6/§7 (lines 455-963),
`blueprint/UI-WIRING-GUIDE.md` R-U1..R-U15 + S01-S28 + §3/§4/§5 (94 lines),
`reports/ui/UI-REQUIREMENTS.md` (167 lines; 52 requirements, G-UI-1..12, K-01..K-16),
`docs/contracts/UI-COMMANDS-v1.md` (71 lines; frozen vocabulary),
`docs/plans/TOOLCHAIN.md`, `docs/plans/SESSION-PLAN.md`,
`backend/crates/hiverouter-core/src/{main,http,config}.rs` (loopback bind, routes),
`web/{package.json,vite.config.ts}`, `web/src/App.tsx` header, `web/src/styles/tokens.css`,
`docs/claims/SKILL-USE.md`, `docs/audit/BASELINE-VERIFICATION.md` (read-only scan).

**Blueprint/contract pins re-verified (match EXECUTION-PROMPT §0):**

| Artifact | sha256 | Match |
|---|---|---|
| `blueprint/HIVEMIND-BLUEPRINT-v1.md` | `4921ABE45746D5932FE5EF681D86C0D8F6217E9607F89D6AEBA9F8E9BBBD716B` (126,400 B / 1,013 lines) | ✅ |
| `blueprint/UI-WIRING-GUIDE.md` | `3911C3E5DED6ED4728E00811126E1639C10BDEA281587997837A302A5F1E8104` (14,838 B) | ✅ |
| `blueprint/EXTERNAL-DEPENDENCIES.md` | `0C933FBFDD84B68AF89CCD34C2FC27DFC3BC3878A23CD50551870235FBF10308` | ✅ |

## 3. Current build state (measured 2026-11-06 04:15-04:27)

| Area | State |
|---|---|
| `web/` | `package.json`, `vite.config.ts` (dev `127.0.0.1:5173` strictPort; preview `:4173`; prod bundle served by Rust core at `:8080`), `node_modules/` installed, `package-lock.json`; `src/` grew **9 → 41 files** during this audit (dirs `a11y, api, chat, flow, panels, state, styles, tokens`); `App.tsx` self-described as **milestone-1 chrome only** at 04:26 ("the wired 28-surface implementation lands next"); frontend was still writing at 04:55 — no dev server or browser run by QA |
| `backend/` | 4 crates (`hiverouter-contract/core/provider/tools`); core routes `GET /health`, `GET /api/session`, `GET /ws`; loopback-only bind default `127.0.0.1:8080`; WS session token + Origin allowlist implemented in source |
| Playwright | `@playwright/test` **not installed** (no root/web/e2e `node_modules` copy); `npx playwright --version` = 1.62.1 (cache) vs blueprint pin **1.65.0** |
| Browser channels | Chrome 156.0.8078.75 (`C:\Program Files\Google\Chrome\Application\chrome.exe`); Edge 156.0.4314.58 (`C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe`) — used via Playwright `channel`, no download |
| Dev server | not running; no browser test executed |
| Disk | ~7.5-8.8 GB free observed during active builds (Lead brief said ~4.1 GB; re-measured — treat as dynamic) |
| Python | 3.14.7 present (FigCraft helper only; product runtime has no Python per D1 — not used by this harness) |

## 4. Harness layout (all new; inside write scope)

```
e2e/
  README.md                    run/install rules + current blockers
  package.json                 @playwright/test 1.65.0 (pinned; NOT installed)
  playwright.config.ts         installed-Chrome/Edge channels only; no browser download; baseURL :8080
  tsconfig.json                spec typecheck settings
  .gitignore                   test-results/playwright-report ignored
  data/control-matrix.json     28 surfaces S01-S28 × frozen commands/events/refusals × ACK rule × status
  helpers/matrix.ts            matrix loader, APP_NOT_BUILT skip gate, frozen command list
  helpers/app.ts               /health + /api/session probe (token, schema, last_seq)
  helpers/ws-tap.ts            Playwright websocket frame capture (ACK/refusal/no-fake-success)
  helpers/contrast.ts          WCAG 2.1 ratio math + expected rows + known failures
  helpers/trace.ts             appends captured TraceRun into docs/trace/traces.json
  specs/control-matrix.spec.ts matrix integrity (static) + 28 per-surface probes (blocked)
  specs/ws-contract.spec.ts    origin/token/hello/ACK/resync (blocked)
  specs/a11y-gates.spec.ts     G-UI-1..12 (static contrast math runnable; DOM blocked)
  specs/trace-hops.spec.ts     11-hop capture per surface (blocked)
tools/qa/
  verify-harness.mjs           static validator: matrix ↔ frozen contract ↔ trace seeds (RUNS NOW, 27/27)
  build-trace-seeds.mjs        regenerates docs/trace/{controls,traces,events}.json (RUNS NOW)
  figcraft-assets.mjs          exact FigCraft export inventory + manifest/A-design presence (RUNS NOW)
  check-env.ps1                host/toolchain/pin/browser/disk/project evidence (RUNS NOW)
  README.md                    tool index
docs/trace/
  QA-HARNESS.md                this plan
  TRACE-SCHEMA.md              11-hop data contract (controls/traces/events/terminal-truth)
  A11Y-CONTRAST-PLAN.md        G-UI-1..12 plan + recomputed contrast table
  controls.json                29 controls × 11 hop slots, all PENDING (generated)
  traces.json                 empty (0 runs) — no fabricated traces
  events.json                 event-parity skeleton, delivered=[] (not collected)
  terminal-truth.md           terminal-run truth template (NOT_COLLECTED)
docs/audit/QA-ENV.md          environment evidence audit (claim/evidence/status)
```

## 5. Frozen contract basis (counts parsed from `docs/contracts/UI-COMMANDS-v1.md`)

| Item | Count | Notes |
|---|---|---|
| UI guide surfaces | **28** | S01-S28 = 24 PDF surfaces + 4 additions; `e2e/data/control-matrix.json` |
| Supplemental contract-named control | **1** | `S-APPROVAL` (UI-COMMANDS-v1 §6 "approval overlay → APPROVAL_RESOLVE"; no UI-WIRING-GUIDE S-number) |
| Frozen commands | **14** | SUBMIT_PROMPT, SPAWN_SUB_HIVE, REWIRE_TASK, CANCEL_RUN, KILL_WORKER, APPROVAL_RESOLVE, SETTINGS_STATUS, FILE_TREE_LIST, FILE_READ, CHAT_PIN, TEMPLATE_INSERT, HISTORY_REVISION_SET, VIEWPORT_SET, VIEWPORT_RECENTER — contract updated **2026-11-06T04:30:24-06:00** (sha `FE6EE89E…CD7B`) |
| Decision-gated (NOT frozen) | **3** | DIFF_PROPOSE / DIFF_APPLY / DIFF_REJECT → S25 stays `DECISION_GATED` |
| Frozen events | **21** | incl. new SETTINGS_STATUS_RESULT, FILE_TREE_RESULT, FILE_CONTENT, APPROVAL_REQUEST/RESOLVED; ACK/ERROR/HELLO/RESYNC/RUN_*/REPLY_*/NODE_*/RUN_METRICS/HISTORY_SNAPSHOT/PING/PONG |
| Named refusal codes | **45** | matrix refusals are a validated subset (incl. new E_APPROVAL_*, E_PATH_*, E_FILE_TOO_LARGE) |
| Mutating controls with ACK requirement | **9** | S02 CHAT_PIN, S03 SUBMIT_PROMPT, S04 SPAWN, S05 REWIRE, S06 CANCEL, S07 KILL, S10 HISTORY_REVISION_SET, S12 TEMPLATE_INSERT, S-APPROVAL APPROVAL_RESOLVE |
| Controls with a command | **12** | incl. S20 client-only VIEWPORT_SET/VIEWPORT_RECENTER (no server ACK) and read commands SETTINGS_STATUS / FILE_TREE_LIST+FILE_READ |

**ACK / no-fake-success rule enforced by the harness:** for every mutating control, the WS tap must observe
`ACK` (or a typed `ERROR` with code/how_to_fix) **before** any optimistic UI mutation; a refused command must leave
state unchanged; S20 viewport commands are client-only and must emit no server mutation.

**Frozen-contract notes (recorded, not invented):**
- **S22 file/code hierarchy gap CLOSED** by the 04:30 contract revision: `FILE_TREE_LIST` / `FILE_READ` are now frozen
  (`E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`, `E_FILE_TOO_LARGE`).
- **Approval overlay** has no UI-WIRING-GUIDE S-number; the contract names it, so the harness adds supplemental
  control `S-APPROVAL` → `APPROVAL_RESOLVE` (nonce replay/expiry refusals included).
- **S03 wording drift:** UI-WIRING-GUIDE writes `{type:"SUBMIT_PROMPT", …}`; the frozen envelope is
  `{schema, type:"COMMAND", name:"SUBMIT_PROMPT", …}`. Harness asserts the frozen envelope; Lead to reconcile the guide.
- **S25** code-diff modal is decision-gated; `DIFF_*` must not ship before user approval.

## 6. 11-hop trace data contract

Full schema: `docs/trace/TRACE-SCHEMA.md`. Hops (exact): `1_render`, `2_handler`, `3_client_call`,
`4_request`, `5_route`, `6_service`, `7_store`, `8_provider`, `9_response`, `10_render_back`, `11_reentry`.
Verdicts: `WIRED-AND-WORKS | WIRED-BUT-DEAD | MISSING | REFUSED-BY-NAME | UNVERIFIED`; seed-only `PENDING`.
Evidence kinds: `dom | request | ws-frame | db-row | log | file` with pointer+quote. Event evidence additionally
requires emitted==subscribed==delivered parity, live-vs-persisted payload shape equality, terminal-record truth and
NULL-not-zero usage for failed/budget runs (`docs/trace/events.json` parity_rules; `terminal-truth.md` template).
Current seed state (W9 recompute): 29 controls × 11 hop slots; **49 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 1 UNVERIFIED of 319 hops (54 evidence-backed excluding the UNVERIFIED marker)** — `node tools/qa/build-trace-seeds.mjs` would regenerate PENDING seeds (do not run: it would drop the captured + test-backed verdicts); `traces.json` 0 runs (no captured TraceRun).

## 7. A11y/contrast plan

Full plan: `docs/trace/A11Y-CONTRAST-PLAN.md`. Static arithmetic recomputed today (WCAG 2.1):
14.6677 / 8.7427 / 12.6391 / 3.3167 / 5.4079 / 4.6364 for the six printed rows; recorded failures
4.2115, 3.8367, 1.6989, 1.5477. All ten values reproduce the UI-REQUIREMENTS expectations. G-UI-1..12 DOM gates
are planned per gate with commands/fixtures and are blocked on the wired UI; no live paid API is used anywhere
(mock/wiremock only). Axe integration decision (inject axe-core vs add `@axe-core/playwright`) is open with the Lead.

## 8. FigCraft evidence (verified files; read-only)

`docs/ui/figcraft/ASSETS/` holds **16 exports** (8 boards × PNG+SVG); hashes are **point-in-time 2026-11-06T04:55**
from `node tools/qa/figcraft-assets.mjs`. A/B agents were still writing during this audit (the b-* boards were re-exported
several times: 04:24, 04:46, 04:55) — re-run the inventory after FigCraft settles:

| File | Bytes | SHA-256 |
|---|---|---|
| a-brand.png | 628,963 | `47680984AA5DACB1BF6A3576B42A11B01864C646068F33E5CA4ACFE124F4A918` |
| a-brand.svg | 940,908 | `94CF946703C14355601813367F9A5FA11105D4E744990DD66AC7B0E5AD0A9522` |
| a-components.png | 2,044,181 | `BC71B15999593B6081FDBD2087432D9C80DFAA2C1430CA5A8DD4173BDA786C58` |
| a-components.svg | 3,706,375 | `553E64308A50162F95F22F29C16DDB5555AD543CAC426E248A95D1ADAA53D0CE` |
| a-foundations.png | 1,054,806 | `E49B788A812E8A4962A29A926D25970B0A39218ADE8DD34EE9C8C7079BDC53BC` |
| a-foundations.svg | 1,768,930 | `24536354E6FFE9BA4DDA2C9682B526BB44BEDEA6AEA7F1D21FD4E3AEB75E551A` |
| a-specimens.png | 886,753 | `99321E1FDEB2446CA2457BABF865D6353FCC693C3AB3E29361A5D8E4D1027FE7` |
| a-specimens.svg | 1,800,816 | `D8128E43D7726A39D3BE07CF5D579A050FE50945296098E717D2BF346AF61563` |
| b-canvas-default.png | 1,027,268 | `AA6066D38F73D391B45C623C62E285BDD19D541F3443DB984279627F2DF7A694` |
| b-canvas-default.svg | 1,223,908 | `B5A5A824145A85848247A10A663FAB2D4783CC120D04DE51FB93316A9EE33D9F` |
| b-hero.png | 941,929 | `719A28BE8DE4926612ECFE5537EC7BDB98E04EAC5338D34FCC6EFBB62B54834E` |
| b-hero.svg | 780,682 | `2BF8D20FACBA986B2C148702A774187538886658C680F7D323AA0E785A678477` |
| b-interaction.png | 1,025,547 | `DE50112439898E88B8BBF334DD8427F0F8AF2FA2FBCC6BED338334F34F0D00A4` |
| b-interaction.svg | 1,665,849 | `927514772AC5321C3771ED7293AE72B67E53526CB544222B21AF408550A2E945` |
| b-zoom-states.png | 1,114,713 | `A02CB4A1963A8C3588AE3CC66FA74C0F7D415E1A46E95089D191794D4F79141C` |
| b-zoom-states.svg | 966,005 | `AB1EE97AE2B9D042B9D8B78EF22F10F1D4B594041FBC51ACD27C32112A9493AE` |

- **`A-design.figcraft` is PRESENT** (`docs/ui/figcraft/A/A-design.figcraft`, 126,928 B as of 04:45:49,
  sha256 `93226DDFC3297274AAD2B649852B53A60321D8A2F4FF280DE05349858276CD74`). QA re-ran
  `figcraft-cli info` (exit 0, stderr 0 bytes; output quoted in `docs/audit/QA-ENV.md`): "HiveMind v1 Design System
  (1368 nodes, 31 styles, 20 variables, 0 images)"; pages 00 Foundations (253 nodes), 01 Brand (176), 02 Components
  (690), 03 Specimens (245). This is CLI output, not a QA conformance verdict.
- **A manifest:** canonical `docs/ui/figcraft/ASSETS-INDEX.json` is **still ABSENT**, but
  `docs/ui/figcraft/A/A-ASSETS.json` **exists** (18,534 B, sha256 `011D5AC08C1B6E2875F2E5CF643DB6624E211B9B0AA9556A0CFAB5868057C1B3`;
  schema `hivemind.ui-assets.v1`, `document_sha256` matching the design, `figcraft_cli_version 0.11.2`, 28 assets mapped
  S01-S28 with export sha256/dimensions/PDF provenance). Also present: `A-SELF-CHECK.json` (`B538C427…471E`; self-reported
  `ok: true`, `surfaces.expected 28 / mapped 28 / missing []`) and `A-EPISODES.json` (`BB3A7AFD…47C6`; command-id
  usage/episodes). **Lead acceptance or a copy/alias to the canonical path is required** before treating it as the
  delivered manifest; QA does not edit FigCraft assets.
- `C:\Program Files\FigCraft\figcraft-cli.exe` present (81,590,128 B); used for `info` only; no FigCraft file modified by QA.
- A-SELF-CHECK `ok:true` is **self-reported by FigCraft-A**, not independently verified by QA; the UI-watchdog/Lead must
  audit it against the PDF/UI requirements.

## 9. What can run now (verified today) vs blocked

**Can run now (evidence captured):**

| Command | Observed result |
|---|---|
| `node tools/qa/verify-harness.mjs` | 27 checks, 27 PASS, 0 FAIL; WARN canonical `ASSETS-INDEX.json` absent (candidate `A/A-ASSETS.json` present); exit 0 |
| `node tools/qa/build-trace-seeds.mjs` | wrote controls.json (29×11 PENDING), traces.json (0 runs), events.json (21 event vocabulary) |
| `node tools/qa/figcraft-assets.mjs` | 16 exports hashed; `A-design.figcraft` present; canonical `ASSETS-INDEX.json` absent but `A/A-ASSETS.json` candidate present (28 surfaces mapped); status `PARTIAL_MANIFEST_CANDIDATE` |
| `powershell -File tools/qa/check-env.ps1` | node/npm/browsers/pins/project state/disk printed; pins match; exit 0 |
| node WCAG recompute (one-liner; recorded §7) | six rows + four failures reproduce within ±0.01 |

**Blocked (named, with owner/next action):**

| Blocker | Why | Owner / unblock |
|---|---|---|
| Wired UI not delivered | `App.tsx` milestone-1 chrome; 28 surfaces not implemented | Frontend milestone 2 |
| Browser tests not executable | `@playwright/test` absent; npx cache 1.62.1 ≠ pin 1.65.0 | Lead approval for one package install (`PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`; no browser download) |
| FigCraft manifest path | canonical `ASSETS-INDEX.json` absent; `A/A-ASSETS.json` candidate present with S01-S28 mapping and matching design sha | Lead accepts candidate or FigCraft-A copies/aliases to `docs/ui/figcraft/ASSETS-INDEX.json`; then re-run `tools/qa/figcraft-assets.mjs` |
| Axe integration decision | `@axe-core/playwright` not pinned; jest-axe is unit-test-only | Lead decision (inject axe-core 4.14.0 or add pinned dep) |
| G-UI-9 mock-provider fixture for browser | no browser-visible mock mode agreed | Backend/Lead; must not use live paid API |
| Dev-origin allowlist for Vite :5173 | backend allowlist targets :8080 origin; dev-mode cross-origin may be refused | Backend owner confirms `http://127.0.0.1:5173` allowlist or e2e runs against prod-shaped :8080 |
| A2/extended live evidence | no live key/budget authorized | user approval only |

## 10. QA observations for owners (not fixed here — outside write scope)

1. **Elevation tiering:** `web/src/styles/tokens.css:45-47` uses three different blur/alpha values
   (`0 2px 8px /0.09`, `0 6px 20px /0.12`, `0 12px 34px /0.13`) while the PDF prints one identical value
   (`#282010 @7-13%`) and UI-R-10/K-13 require tier differentiation to stay UNVERIFIED. The comment says
   "tiering UNVERIFIED" but the values do differ — frontend owner should confirm this is allowed or flatten/label.
2. **Bundled fonts:** `tokens.css:30-32` names "Playfair Display Variable"/"Inter Variable", but `web/package.json`
   has no `@fontsource/*` dependency and no `@font-face` source is visible in the 9 current files — font fidelity
   (K-09/R-U12) at risk; frontend owner to bundle or document fallback.
3. **S03 envelope drift** recorded in §5 — contract owner to reconcile the guide wording with the frozen envelope.
4. **Playwright pin drift:** local npx cache 1.62.1 vs blueprint 1.65.0 — install must use `@playwright/test@1.65.0`.
5. **Manifest path/self-report:** `A-SELF-CHECK.json` says `ok: true` and `A-ASSETS.json` maps 28/28 surfaces, but this is
   FigCraft-A self-report; canonical `ASSETS-INDEX.json` (EXECUTION-PROMPT §6 path) is still missing. UI-watchdog/Lead
   must independently audit A assets and accept/alias the manifest before the frontend consumes the mapping.

## 11. Next steps when the wired app lands

1. Install Playwright package only (Lead approval; `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`), verify `npx playwright --version` = 1.65.0.
2. Boot app (prod-shaped `http://127.0.0.1:8080` preferred; dev :5173 only after Origin allowlist).
3. Lead accepts `A/A-ASSETS.json` as the FigCraft-A manifest (or FigCraft-A copies it to `docs/ui/figcraft/ASSETS-INDEX.json`); re-run `tools/qa/figcraft-assets.mjs`.
4. `E2E_APP_READY=1 npm run e2e:chrome`; capture failures as defects, never as passes.
5. Fill `docs/trace/controls.json` verdicts only from `traces.json` evidence; run events parity + terminal truth.
6. Update `e2e/README.md`/this doc with measured status; report Done/Not done/Unverified to Lead.

## 12. Unverified (honest limits)

- No UI control has been exercised; all 28 guide ×11 hop verdicts (+1 supplemental control) are PENDING/UNVERIFIED.
- No Playwright run exists; no screenshots/traces exist; no G-UI gate has been executed.
- No `/api/session` token was captured from a running server.
- FigCraft asset→surface mapping is candidate-only until the Lead accepts/aliases `A/A-ASSETS.json`; pixel-level visual
  fidelity and the A-SELF-CHECK `ok:true` claim remain UNVERIFIED (self-report).
- Disk/time budget claims are dynamic measurements, not guarantees; the Lead's ~4.1 GB figure was not reproducible later (measured 7.5-8.8 GB).
