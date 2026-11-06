# QA-ENV — QA/trace environment & artifact evidence audit

| Field | Value |
|---|---|
| Seat | QA / trace engineer (role changed from Watchdog 2; watchdog task closed) |
| Date | 2026-11-06, evidence captured 04:15–04:36 America/Chicago |
| Scope | read-only probes + harness files under `e2e/**`, `docs/trace/**`, `docs/audit/QA-*.md`, `tools/qa/**` |
| Verdict | **HARNESS READY (static); EXECUTION BLOCKED** — no fabricated pass claims |
| Skills loaded | `trace`, `capability-auditor`, `evidence-check`, `mermaid-skill` (loaded; docs/claims/SKILL-USE.md is outside QA write scope) |

## Claim / evidence / status

| # | Claim | Evidence (command → observed) | Status |
|---|---|---|---|
| 1 | Node available and satisfies >=22.12.0 | `node --version` → `v26.7.0` | VERIFIED |
| 2 | npm available (PowerShell `.ps1` blocked by execution policy; use cmd) | `cmd /c npm --version` → `11.19.0`; `npm.ps1 cannot be loaded` under pwsh | VERIFIED (with policy caveat) |
| 3 | Playwright CLI reachable via npx cache, version 1.62.1 | `npx --yes playwright --version` → `Version 1.62.1` | VERIFIED |
| 4 | `@playwright/test` NOT installed in project; blueprint pin 1.65.0 | probes: root/`web`/`e2e` `node_modules` absent for playwright (`web/node_modules/@playwright` absent); `npm ls -g playwright` → empty; blueprint EXT-R4-06 pin `@playwright/test 1.65.0` | VERIFIED (absence) / pin READ |
| 5 | Installed Chrome channel available | `C:\Program Files\Google\Chrome\Application\chrome.exe` ProductVersion `156.0.8078.75` | VERIFIED |
| 6 | Installed Edge channel available | `C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe` ProductVersion `156.0.4314.58` | VERIFIED |
| 7 | No browser download used/needed | ms-playwright cache exists (`chromium-1234/1248`, headless shell) from unrelated tools — **not used**; e2e config uses `channel` only; no `playwright install` run | VERIFIED (policy + config) |
| 8 | Blueprint implementation pins match EXECUTION-PROMPT §0 | sha256 computed: `blueprint/HIVEMIND-BLUEPRINT-v1.md` = `4921ABE4…716B` ✅; `UI-WIRING-GUIDE.md` = `3911C3E5…8104` ✅; `EXTERNAL-DEPENDENCIES.md` = `0C933FBF…0308` ✅ | VERIFIED |
| 9 | Frozen source unchanged | `hivemind-blueprint/source/HIVEMIND POSSIBLITY.pruned.txt` sha256 = `7B85760E…D713` (frozen value) | VERIFIED |
| 10 | Frozen contract vocabulary extent | parse `docs/contracts/UI-COMMANDS-v1.md`: 10 frozen commands, 3 decision-gated DIFF, 18 events, 40 refusal codes | VERIFIED (parser in `tools/qa/verify-harness.mjs`) |
| 11 | 28-surface matrix exists and is contract-consistent | `node tools/qa/verify-harness.mjs` → 24 checks, **24 PASS, 0 FAIL**, WARN `ASSETS-INDEX.json absent`, exit 0 | VERIFIED |
| 12 | Trace seeds generated with no fabricated verdicts | `node tools/qa/build-trace-seeds.mjs` → controls 28×11 all PENDING; traces 0 runs; events delivered=[] | VERIFIED |
| 13 | Static contrast math reproduces the six printed rows | node one-liner: 14.6677 / 8.7427 / 12.6391 / 3.3167 / 5.4079 / 4.6364; failures 4.2115 / 3.8367 / 1.6989 / 1.5477 | VERIFIED (arithmetic; no DOM) |
| 14 | FigCraft-A exports present; exact files/hashes | `node tools/qa/figcraft-assets.mjs` → 10 assets (a-brand, a-foundations, a-components, a-specimens, b-canvas-default; PNG+SVG each) with sha256 table; output captured | VERIFIED (point-in-time; assets were still being written — see #15) |
| 15 | FigCraft assets are in flux during session | first listing 04:14 showed 6 assets (a-components.png 1,913,032 B); 04:25 listing shows 10 assets (a-components.png 2,044,181 B, plus a-specimens, b-canvas-default) | VERIFIED (timestamps 04:24–04:25) |
| 16 | A manifest (`ASSETS-INDEX.json`) absent | `Test-Path docs/ui/figcraft/ASSETS-INDEX.json` → False; validator WARN | VERIFIED (absence) |
| 17 | `A-design.figcraft` absent → `figcraft-cli info` not re-runnable | searched `docs/ui/figcraft`, `A`, `build` → not found; no `.figcraft` named A-design | VERIFIED (absence) |
| 18 | `figcraft-cli.exe` present but not invoked | `C:\Program Files\FigCraft\figcraft-cli.exe`, 81,590,128 B; `--help` produced no stdout (GUI-class) | VERIFIED (presence) / uninvoked |
| 19 | Frontend scaffold exists but is not the wired 28-surface app | `web/package.json` + `vite.config.ts` (`127.0.0.1:5173` strictPort, preview 4173), `node_modules/` installed, 9 `src/` files; `App.tsx` header: "milestone 1 — chrome only … wired 28-surface implementation lands next" | VERIFIED |
| 20 | Backend loopback surface exists | `backend/.../main.rs`: default bind `127.0.0.1:8080`; `http.rs` routes `/health`, `/api/session`, `/ws`; per-session token + Origin allowlist | READ source (not executed) |
| 21 | No browser e2e executed | no Playwright install; no dev server running; `e2e/test-results` absent | VERIFIED (absence) |
| 22 | Disk headroom re-measured | `(Get-PSDrive C).Free` → 8.77 GB (04:13), 7.97 GB (04:25), 7.51 GB (04:27) during active builds | VERIFIED (dynamic; Lead's ~4.1 GB not reproducible at these times) |
| 23 | QA writes stayed in scope | new paths only `e2e/**`, `docs/trace/**`, `docs/audit/QA-HARNESS.md`-style `QA-*.md`, `tools/qa/**`; `web/`, `backend/`, `docs/ui/figcraft/`, blueprint untouched by QA | VERIFIED (git status review; other agents' changes exist independently) |

## Notes / discrepancies recorded (not defects fixed here)

1. **Playwright pin drift:** npx cache 1.62.1 vs blueprint devDependency pin `@playwright/test 1.65.0`. Install must pin 1.65.0; no install performed (Lead approval required, no browser download).
2. **Blueprint/contract drift to reconcile (owned by Lead/Frontend):** UI-WIRING-GUIDE S03 writes `{type:"SUBMIT_PROMPT"}` while frozen envelope is `type:"COMMAND"` + `name:"SUBMIT_PROMPT"`; S22 file/code hierarchy has no frozen command. Recorded in `docs/trace/QA-HARNESS.md` §5; QA did not edit those docs.
3. **Font bundling risk:** `web/src/styles/tokens.css` names Playfair/Inter faces but `web/package.json` has no `@fontsource/*` dependency (K-09/R-U12 risk) — frontend owner.
4. **Elevation tiering risk:** `tokens.css:45-47` uses three distinct blur/alpha values while the PDF prints one identical value and UI-R-10/K-13 say tier differentiation stays UNVERIFIED — frontend owner to confirm.
5. **Assets in flux:** FigCraft-A/B were actively writing during this audit (04:24–04:25); hashes in `QA-HARNESS.md` §8 are point-in-time. Re-run `tools/qa/figcraft-assets.mjs` after FigCraft settles and after `ASSETS-INDEX.json` appears.

## Not verified / open

- Any UI control behaviour, WS flow, ACK/refusal, trace hop, G-UI gate, screenshot, or performance measurement.
- `figcraft-cli info` output (A-design.figcraft absent) and FigCraft asset→surface mapping (manifest absent).
- Backend boot/`/health` response; no server was started by QA (avoided interfering with active build agents).
- Disk/free-space guarantees; measurements are snapshots.
