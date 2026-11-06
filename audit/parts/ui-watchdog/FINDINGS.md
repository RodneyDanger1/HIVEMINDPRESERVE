# FINDINGS.md — ui-watchdog seat

**Audited:** 2026-11-06 · 49 files, read-only · evidence paths are repo-relative; manifest sha256 = `audit/inventory/FILE-MANIFEST.csv`.

| ID | Severity | Finding | Evidence | Fix / owner |
|---|---|---|---|---|
| F-01 | low | `.cargo/config.toml` hardcodes absolute host paths (`C:/Users/Riley/Desktop/hiverouter-v1/tools/w64devkit/bin/{gcc,ar}.exe`) → build config is non-portable and leaks host layout | `.cargo/config.toml:5-12` | use relative/`$PWD`-derived or env-var paths; owner: build |
| F-02 | medium | Documentation overstates verification breadth: README/final claims (233 backend tests, 79 e2e passes) coexist with trace evidence showing 260 UNVERIFIED hops and 5-6 MISSING (node cards S15-S19, S-APPROVAL) | `README.md:31-33`; `docs/trace/live/README.md`; `docs/trace/controls.json` (hop verdicts PENDING 270 / WIRED 43 / MISSING 6) | add explicit 'unverified surfaces' caveat to README status; owner: Lead |
| F-03 | medium | Stale status docs: A11Y-CONTRAST-PLAN says 'Wired UI not delivered yet … DOM gates blocked'; QA-HARNESS says harness LIVE with 79-pass final run | `docs/trace/A11Y-CONTRAST-PLAN.md:2,58` vs `docs/trace/QA-HARNESS.md:3` | refresh A11Y plan status to post-ship (DOM gates still blocked only for un-automatable gates); owner: QA |
| F-04 | medium | Terminal-run truth never collected: template says 'app does not exist yet … NOT_COLLECTED' (stale) and all 9 evidence rows PENDING | `docs/trace/terminal-truth.md:3,10-24` | run a terminal capture (no paid API needed: mock/refusal path) and fill template; owner: QA/trace |
| F-05 | medium | Event parity and run traces uncollected: `events.json` emitted/subscribed/delivered/payload_parity/terminal_records all empty (NOT_COLLECTED); `traces.json` 0 runs | `docs/trace/events.json:3-11`; `docs/trace/traces.json:3-5` | capture one full WS/event run and one terminal-run record; owner: QA |
| F-06 | low | `er-schema.mmd` annotation 'v1 has app_meta only' conflicts with shipped backend claiming runs/tasks/events tables (QA-HARNESS §3) | `docs/diagrams/er-schema.mmd:3` vs `docs/trace/QA-HARNESS.md:48` | re-verify against `backend/.../db.rs` (outside this partition) then re-render; owner: b-core/verifier |
| F-07 | low | `controls.json` `_meta.status` still `PENDING_APP_NOT_BUILT` while 23/29 controls are `PARTIALLY_VERIFIED` — metadata lies about the aggregate state | `docs/trace/controls.json` `_meta.status` vs `controls[].status` | derive `_meta.status` from counts; owner: QA |
| F-08 | low | `inspect.json` records one console error (`404 Not Found`) while `03-console.json` reports none — different runs; a 404 during boot is an unexplained resource miss | `docs/trace/live/inspect.json` consoleErrors; `docs/trace/live/live-app/03-console.json` | identify missing resource URL; owner: web |
| F-09 | low | Evidence artifact duplication: `live-app/boot.png` and `live-app/surface-census.png` are byte-identical (sha `9EA23060505B…`), so the census screenshot is not a distinct capture | manifest sha256 for both; `live-app/04-census.json` references `surface-census.png` | re-capture census screenshot after rendering the census state; owner: QA |
| F-10 | low | `CHANGE-LEDGER-IMPL.md` CL-I-003 records FigCraft B as '24 + 2 pending', superseded by final B-ASSETS (27 surfaces, 0 pending) — ledger not closed | `CHANGE-LEDGER-IMPL.md:10` vs final `B/B-ASSETS.json` | update row to applied/closed with final counts; owner: Lead |
| F-11 | low | `architecture.mmd` header still says 'Baseline status: PLANNED TARGET — web/ not implemented at baseline' — accurate as baseline note but misleading as current-state diagram without date | `docs/diagrams/architecture.mmd:3` | add rendered/shipped date line; owner: verifier |
| F-13 | low | FigCraft design files and rendered exports are gitignored (`.gitignore`: `docs/ui/figcraft/*/*.figcraft`, `ASSETS/*.png|svg`) -> the shipped git repo/manifest (507 files) does not contain the UI design assets; provenance relies on the working tree only | `.gitignore`; `audit/inventory/FILE-MANIFEST.csv` has no figcraft asset rows | commit the accepted A/B design files + exports under a release tag, or record the external-asset hash list in the repo; owner: Lead |
| F-12 | info | Honest-limits pattern is strong: trace docs explicitly mark NOT_COLLECTED/UNVERIFIED and refuse fake passes (0 WIRED-BUT-DEAD asserted); diagrams and evidence are hash-pinned | `TRACE-SCHEMA.md:5,2`; `README.md`; manifest hashes | keep as audit strength |

## Notes on method

- No product/git writes were made; only `audit/parts/ui-watchdog/**` was written.
- A claim is marked UNVERIFIED when it needs code outside this partition (e.g. `db.rs`, `web/src/**`) or a live run/number I could not read.
- Severity: medium = affects acceptance evidence or docs truth; low = hygiene/portability/staleness; info = positive observation.
