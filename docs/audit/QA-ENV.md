# QA-ENV — QA/trace environment & artifact evidence audit

| Field | Value |
|---|---|
| Seat | QA / trace engineer (role changed from Watchdog 2; watchdog task closed) |
| Date | 2026-11-06, evidence captured 04:15–04:46 America/Chicago |
| Scope | read-only probes + harness files under `e2e/**`, `docs/trace/**`, `docs/audit/QA-*.md`, `tools/qa/**` |
| Verdict | **HARNESS READY (static, 27/27 checks); EXECUTION BLOCKED** — no fabricated pass claims |
| Skills loaded | `trace`, `capability-auditor`, `evidence-check`, `mermaid-skill` (docs/claims/SKILL-USE.md is outside QA write scope) |

## Claim / evidence / status

| # | Claim | Evidence (command → observed) | Status |
|---|---|---|---|
| 1 | Node available and satisfies >=22.12.0 | `node --version` → `v26.7.0` | VERIFIED |
| 2 | npm available (PowerShell `.ps1` blocked by execution policy; use cmd) | `cmd /c npm --version` → `11.19.0`; `npm.ps1 cannot be loaded` under pwsh | VERIFIED (with policy caveat) |
| 3 | Playwright CLI reachable via npx cache, version 1.62.1 | `npx --yes playwright --version` → `Version 1.62.1` | VERIFIED |
| 4 | `@playwright/test` NOT installed in project; blueprint pin 1.65.0 | probes: no playwright in root/web/e2e node_modules; `npm ls -g playwright` empty; blueprint EXT-R4-06 pins `@playwright/test 1.65.0` | VERIFIED (absence) / pin READ |
| 5 | Installed Chrome channel available | `C:\Program Files\Google\Chrome\Application\chrome.exe` ProductVersion `156.0.8078.75` | VERIFIED |
| 6 | Installed Edge channel available | `C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe` ProductVersion `156.0.4314.58` | VERIFIED |
| 7 | No browser download used/needed | e2e config uses `channel` only; ms-playwright cache (chromium-1234/1248 etc.) is from unrelated tools and unused; no `playwright install` run | VERIFIED (policy + config) |
| 8 | Blueprint implementation pins match EXECUTION-PROMPT §0 | sha256: `blueprint/HIVEMIND-BLUEPRINT-v1.md` = `4921ABE4…716B` ✅; `UI-WIRING-GUIDE.md` = `3911C3E5…8104` ✅; `EXTERNAL-DEPENDENCIES.md` = `0C933FBF…0308` ✅ | VERIFIED |
| 9 | Frozen source unchanged | `hivemind-blueprint/source/HIVEMIND POSSIBLITY.pruned.txt` sha256 = `7B85760E…D713` (frozen value) | VERIFIED |
| 10 | Frozen contract extent (updated mid-audit) | `docs/contracts/UI-COMMANDS-v1.md` mtime `2026-11-06T04:30:24-06:00`, sha `FE6EE89EF2DE905A747D6EFEAC9749D0CFFCCF0B6FE7A524892626486DF5CD7B`; parse → **14 frozen commands, 3 decision-gated DIFF, 21 events, 45 refusal codes** | VERIFIED |
| 11 | 28-surface matrix + supplemental control contract-consistent | `node tools/qa/verify-harness.mjs` → **27 checks, 27 PASS, 0 FAIL**, WARN canonical `ASSETS-INDEX.json` absent, exit 0 | VERIFIED |
| 12 | Trace seeds generated with no fabricated verdicts | `node tools/qa/build-trace-seeds.mjs` → controls 29 (28 guide + `S-APPROVAL`) ×11 all PENDING; traces 0 runs; events delivered=[]; event vocabulary 21 | VERIFIED |
| 13 | Static contrast math reproduces the six printed rows | node WCAG 2.1 one-liner: 14.6677 / 8.7427 / 12.6391 / 3.3167 / 5.4079 / 4.6364; failures 4.2115 / 3.8367 / 1.6989 / 1.5477; `STATIC_CONTRAST_OK=6/6` | VERIFIED (arithmetic; no DOM) |
| 14 | `A-design.figcraft` present; its `figcraft-cli info` re-run captured | file `docs/ui/figcraft/A/A-design.figcraft`, 126,928 B (04:45:49), sha256 `93226DDFC3297274AAD2B649852B53A60321D8A2F4FF280DE05349858276CD74`; `figcraft-cli info` → **exit 0**, stderr 0 bytes, stdout 493 bytes: "HiveMind v1 Design System (1368 nodes, 31 styles, 20 variables, 0 images)"; pages 00 Foundations (253 nodes), 01 Brand (176), 02 Components (690), 03 Specimens (245) | VERIFIED (exit + output) |
| 15 | FigCraft CLI present | `C:\Program Files\FigCraft\figcraft-cli.exe`, 81,590,128 B; invoked once for `info` (read-only) | VERIFIED |
| 16 | FigCraft asset exports present | `node tools/qa/figcraft-assets.mjs` → **16 exports** = 8 boards × PNG+SVG: a-brand, a-foundations, a-components, a-specimens, b-canvas-default, b-hero, b-interaction, b-zoom-states; per-file sha256 listed in `QA-HARNESS.md` §8 | VERIFIED (point-in-time; A/B were writing at 04:24–04:37 — re-run after settle) |
| 17 | A manifest candidate present; canonical path absent | `docs/ui/figcraft/A/A-ASSETS.json` exists (18,534 B, sha256 `011D5AC08C1B6E2875F2E5CF643DB6624E211B9B0AA9556A0CFAB5868057C1B3`): schema `hivemind.ui-assets.v1`, `document_sha256` matching A-design, `figcraft_cli_version 0.11.2`, 28 assets mapped S01-S28 with export sha256/dimensions/PDF provenance. Canonical `docs/ui/figcraft/ASSETS-INDEX.json` (EXECUTION-PROMPT §6 path) **absent** | VERIFIED (presence/absence); mapping not independently audited |
| 18 | A self-check/self-report present | `A-SELF-CHECK.json` sha256 `B538C427341F7B592233D4B4FED2116A066A6D786916F59EAE9A9132F11B471E`: `ok:true`, `surfaces.expected 28 / mapped 28 / missing []`, exports hashes/dimensions; `A-EPISODES.json` sha256 `BB3A7AFD2CE4FE103D279656D374D9B3501A28C4F1401554ECAC99B09D7547C6` present | VERIFIED (file/parse) — self-reported by FigCraft-A, not QA-verified |
| 19 | Frontend scaffold exists but is not verified as the wired app | `web/package.json`, `vite.config.ts` (`127.0.0.1:5173` strictPort, preview 4173), `node_modules/`, `package-lock.json`; `src/` grew **9 → 41 files** during this audit (dirs a11y/api/chat/flow/panels/state/styles/tokens); `App.tsx` at 04:26 self-described as "milestone 1 — chrome only"; frontend still writing at 04:55 — no dev server started by QA, no browser run | VERIFIED (snapshot; app completeness not verified) |
| 20 | Backend loopback surface exists | `backend/.../main.rs`: default bind `127.0.0.1:8080`; `http.rs`: `GET /health`, `GET /api/session`, `GET /ws`; per-session token + Origin allowlist | READ source (not executed) |
| 21 | No browser e2e executed | no Playwright install; no dev server started by QA; no `e2e/test-results` | VERIFIED (absence) |
| 22 | Disk headroom re-measured | `(Get-PSDrive C).Free` → 8.77 GB (04:13), 7.97 GB (04:25), 7.51 GB (04:27), 3.71 GB (04:52) during active builds | VERIFIED (dynamic; Lead's ~4.1 GB brief figure not reproduced at earlier times) |
| 23 | QA writes stayed in scope | new paths only `e2e/**`, `docs/trace/**`, `docs/audit/QA-*.md`, `tools/qa/**`; no writes to `web/`, `backend/`, `docs/ui/figcraft/`, `hivemind-blueprint/` by QA; repo HEAD moved to `ce24525` (other agents' commits) | VERIFIED (git status review; other agents' changes exist independently) |
| 24 | Contract changed mid-audit (drift handled) | `docs/contracts/UI-COMMANDS-v1.md` updated 04:30 adding APPROVAL_RESOLVE / SETTINGS_STATUS / FILE_TREE_LIST / FILE_READ (sha `FE6EE89E…CD7B`); matrix updated (S21/S22 + `S-APPROVAL`) and validator re-run to **27/27 PASS** | VERIFIED |
| 25 | FigCraft assets in flux | exports went 6 → 10 → 16 during the audit; b-* boards re-exported at 04:24 / 04:46 / 04:55; A-design/A-ASSETS regenerated 04:45–04:46. Hashes in `QA-HARNESS.md` §8 are point-in-time 04:55 | VERIFIED (timestamps/hashes; re-run `tools/qa/figcraft-assets.mjs` after settle) |

## Discrepancies / notes (owned elsewhere; not fixed by QA)

1. **Playwright pin drift:** npx cache 1.62.1 vs blueprint pin `@playwright/test 1.65.0`. Install must pin 1.65.0; no install performed (Lead approval required; no browser download).
2. **S03 envelope drift (Lead/contract owner):** UI-WIRING-GUIDE S03 writes `{type:"SUBMIT_PROMPT"}`; frozen envelope is `type:"COMMAND"` + `name:"SUBMIT_PROMPT"`. Harness asserts the frozen envelope.
3. **Manifest path mismatch (Lead/FigCraft-A):** canonical `ASSETS-INDEX.json` missing; candidate `A/A-ASSETS.json` exists with 28/28 surface mapping. Needs acceptance or copy/alias before the frontend consumes it; `A-SELF-CHECK ok:true` is self-report, not independent audit.
4. **Font bundling risk:** `web/src/styles/tokens.css` names Playfair/Inter faces but `web/package.json` has no `@fontsource/*` dependency (K-09/R-U12) — frontend owner.
5. **Elevation tiering risk:** `tokens.css:45-47` uses three distinct blur/alpha values while PDF prints one identical value and UI-R-10/K-13 say tier differentiation stays UNVERIFIED — frontend owner.
6. **Assets in flux:** 6 → 10 → 16 asset files during this session (04:14/04:25/04:52); b-* boards re-exported; hashes are point-in-time; re-run `tools/qa/figcraft-assets.mjs` after FigCraft settles.
7. **`figcraft-cli info` exit code** was captured on the final run via `cmd /c` → **EXIT=0**, stderr 0 bytes (earlier `Start-Process` wrapper did not surface the exit code; the output was already valid at that time).

## Not verified / open

- Any UI control behaviour, WS flow (ACK/refusal/no-fake-success), trace hop, G-UI gate, screenshot, or performance measurement.
- FigCraft asset→surface mapping against the PDF/UI requirements (candidate only; UI-watchdog/Lead audit pending).
- Backend boot/`/health` response (no server started by QA to avoid interfering with active build agents).
- Disk/free-space guarantees (dynamic measurements).
