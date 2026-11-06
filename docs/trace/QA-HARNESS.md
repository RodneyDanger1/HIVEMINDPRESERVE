# QA HARNESS — HiveMind v1 (QA / trace engineer)

**Status: HARNESS BUILT + STATIC CHECKS RUNNING; BROWSER EXECUTION BLOCKED (wired UI not delivered).**
No pass is claimed for any UI control. Every browser spec skips with `APP_NOT_BUILT` until the wired app
boots; the static harness (24/24 checks) validates the 28-surface matrix against the frozen contract.

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
| `web/` | `package.json`, `vite.config.ts` (dev `127.0.0.1:5173` strictPort; preview `:4173`; prod bundle served by Rust core at `:8080`), `node_modules/` installed, `package-lock.json`; `src/` = 9 files across `a11y, api, chat, flow, panels, state, styles, tokens`; `App.tsx` self-describes as **milestone-1 chrome only** ("the wired 28-surface implementation lands next") |
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
  verify-harness.mjs           static validator: matrix ↔ frozen contract ↔ trace seeds (RUNS NOW, 24/24)
  build-trace-seeds.mjs        regenerates docs/trace/{controls,traces,events}.json (RUNS NOW)
  figcraft-assets.mjs          exact FigCraft export inventory + manifest/A-design presence (RUNS NOW)
  check-env.ps1                host/toolchain/pin/browser/disk/project evidence (RUNS NOW)
  README.md                    tool index
docs/trace/
  QA-HARNESS.md                this plan
  TRACE-SCHEMA.md              11-hop data contract (controls/traces/events/terminal-truth)
  A11Y-CONTRAST-PLAN.md        G-UI-1..12 plan + recomputed contrast table
  controls.json                28 controls × 11 hop slots, all PENDING (generated)
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
Seeds generated: `node tools/qa/build-trace-seeds.mjs` → 28 controls × 11 PENDING hops; `traces.json` 0 runs.

## 7. A11y/contrast plan

Full plan: `docs/trace/A11Y-CONTRAST-PLAN.md`. Static arithmetic recomputed today (WCAG 2.1):
14.6677 / 8.7427 / 12.6391 / 3.3167 / 5.4079 / 4.6364 for the six printed rows; recorded failures
4.2115, 3.8367, 1.6989, 1.5477. All ten values reproduce the UI-REQUIREMENTS expectations. G-UI-1..12 DOM gates
are planned per gate with commands/fixtures and are blocked on the wired UI; no live paid API is used anywhere
(mock/wiremock only). Axe integration decision (inject axe-core vs add `@axe-core/playwright`) is open with the Lead.

## 8. FigCraft evidence (verified files; read-only)

`docs/ui/figcraft/ASSETS/` currently holds **10 exports** (PNG+SVG pairs); hashes from `tools/qa/figcraft-assets.mjs`
run 2026-11-06T10:24:57Z:

| File | Bytes | SHA-256 |
|---|---|---|
| a-brand.png | 628,963 | `47680984AA5DACB1BF6A3576B42A11B01864C646068F33E5CA4ACFE124F4A918` |
| a-brand.svg | 940,908 | `94CF946703C14355601813367F9A5FA11105D4E744990DD66AC7B0E5AD0A9522` |
| a-components.png | 2,044,181 | `BC71B15999593B6081FDBD2087432D9C80DFAA2C1430CA5A8DD4173BDA786C58` |
| a-components.svg | 3,706,375 | `553E64308A50162F95F22F29C16DDB5555AD543CAC426E248A95D1ADAA53D0CE` |
| a-foundations.png | 1,054,806 | `E49B788A812E8A4962A29A926D25970B0A39218ADE8DD34EE9C8C7079BDC53BC` |
| a-foundations.svg | 1,768,930 | `24536354E6FFE9BA4DDA2C9682B526BB44BEDEA6AEA7F1D21FD4E3AEB75E551A` |
| a-specimens.png | 894,483 | `109CB256A8D8C0F26D0717EEDC495C3829ED619EFCA968B61CE1469D2FDEC32F` |
| a-specimens.svg | 1,802,175 | `2A75F9BE19816D9BD2E742CE14B2841E5ADC17DF4975F6C732316B3F1DF65D16` |
| b-canvas-default.png | 1,028,183 | `CCA6B5E2726475B7928D6DC5A83B788F707A69C9AB0BD7E0C3F95E501F8898BA` |
| b-canvas-default.svg | 1,223,210 | `FEA7E5CC208DBE57E12D01E909D1DD8315CBBE7BB6D9FD07785476822360FBA3` |

- **A manifest (`docs/ui/figcraft/ASSETS-INDEX.json`) is ABSENT** — asset→surface (S01-S28) mapping blocked; watch this path.
- **`A-design.figcraft` is ABSENT** (searched `docs/ui/figcraft/{,A,build}`) — `figcraft-cli info` cannot be re-run on it.
- `C:\Program Files\FigCraft\figcraft-cli.exe` present (81,590,128 B); **not invoked** (no A-design file; GUI-class binary
  produced no stdout on `--help`). No FigCraft file was modified by QA.

## 9. What can run now (verified today) vs blocked

**Can run now (evidence captured):**

| Command | Observed result |
|---|---|
| `node tools/qa/verify-harness.mjs` | 24 checks, 24 PASS, 0 FAIL; WARN `ASSETS-INDEX.json absent`; exit 0 |
| `node tools/qa/build-trace-seeds.mjs` | wrote controls.json (28×11 PENDING), traces.json (0 runs), events.json (18 event vocabulary) |
| `node tools/qa/figcraft-assets.mjs` | 10 exports hashed; manifest/A-design absent; status BLOCKED_PARTIAL |
| `powershell -File tools/qa/check-env.ps1` | node/npm/browsers/pins/project state/disk printed; pins match; exit 0 |
| node WCAG recompute (one-liner; recorded §7) | six rows + four failures reproduce within ±0.01 |

**Blocked (named, with owner/next action):**

| Blocker | Why | Owner / unblock |
|---|---|---|
| Wired UI not delivered | `App.tsx` milestone-1 chrome; 28 surfaces not implemented | Frontend milestone 2 |
| Browser tests not executable | `@playwright/test` absent; npx cache 1.62.1 ≠ pin 1.65.0 | Lead approval for one package install (`PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`; no browser download) |
| FigCraft asset manifest/`A-design.figcraft` absent | ASSETS-INDEX not delivered; no design master to `info` | FigCraft-A owner; re-run `tools/qa/figcraft-assets.mjs` when they land |
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
3. **S03 envelope drift** and **S22 missing command** recorded in §5 — contract owner to reconcile.
4. **Playwright pin drift:** local npx cache 1.62.1 vs blueprint 1.65.0 — install must use `@playwright/test@1.65.0`.

## 11. Next steps when the wired app lands

1. Install Playwright package only (Lead approval; `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`), verify `npx playwright --version` = 1.65.0.
2. Boot app (prod-shaped `http://127.0.0.1:8080` preferred; dev :5173 only after Origin allowlist).
3. `E2E_APP_READY=1 npm run e2e:chrome`; capture failures as defects, never as passes.
4. Fill `docs/trace/controls.json` verdicts only from `traces.json` evidence; run events parity + terminal truth.
5. Update `e2e/README.md`/this doc with measured status; report Done/Not done/Unverified to Lead.

## 12. Unverified (honest limits)

- No UI control has been exercised; all 28×11 hop verdicts are PENDING/UNVERIFIED.
- No Playwright run exists; no screenshots/traces exist; no G-UI gate has been executed.
- No `/api/session` token was captured from a running server.
- FigCraft asset→surface mapping is unknown until `ASSETS-INDEX.json` lands; pixel-level visual fidelity remains UNVERIFIED.
- Disk/time budget claims are dynamic measurements, not guarantees; the Lead's ~4.1 GB figure was not reproducible later (measured 7.5-8.8 GB).
