# EVIDENCE.md — commands run and outputs (ui-watchdog seat)

**Date:** 2026-11-06 · **Repo HEAD:** `3988622` ('closeout: sync FINAL-REPORT…'), `git status --porcelain` = `?? audit/` only (audit packet untracked; no product changes).
**Skills loaded:** `research` (forgewright-research), `truthseeker`, `forgewright-recon`, `weaver`.

## Commands + results

| # | command (read-only) | result |
|---|---|---|
| 1 | `read audit/WAVE-SHEET.md` | 56 lines; scope, 49-file partition, required outputs, 25-min timebox |
| 2 | `read audit/inventory/ASSIGNMENTS.json` (seat read via python) | `ui-watchdog` = 49 entries; all 49 found in FILE-MANIFEST.csv (0 missing) |
| 3 | python read of `audit/inventory/FILE-MANIFEST.csv` | 507 rows; columns path,bytes,sha256,ext,language,category,subcategory,seat,depth; 49 seat rows extracted with bytes/sha |
| 4 | bulk read of all 49 assigned text files (python, UTF-8) | full content or structured digest; binaries hashed/measured not prose-analysed |
| 5 | `git log --oneline -3` | `3988622 closeout: sync FINAL-REPORT…` / `2817326 closeout: final landing checks PASS…` / `6815dbe package: ship hiverouter-cli.exe…` |
| 6 | `git status --porcelain` | 1 line: `?? audit/` (expected untracked audit packet; repo clean otherwise) |
| 7 | PNG IHDR dimension reads (python struct) | agent-lifecycle 1839×1122; architecture 1733×1676; er-schema 2032×735; run-sequence 2032×1092; ui-control 2032×813; app-boot-full 1440×900; app-boot 1440×900; live boot 1280×720; surface-census 1280×720 |
| 8 | JSON structure parses (python) | `controls.json` 29 controls × hops verdicts {PENDING 270, WIRED-AND-WORKS 43, MISSING 6}; `events.json` 21-event vocabulary, empty parity arrays; `traces.json` 0 runs; `trace-capture.json` 28 surfaces; `inspect.json` boot/WS facts |
| 9 | `backend/Cargo.lock` regex parse | 405 `[[package]]` entries; direct workspace pins read from `backend/Cargo.toml` + 4 crate manifests |
| 10 | `web/package.json` + `web/package-lock.json` parse | 7 runtime deps, 8 devDeps; lockfile v3, 147 entries, 139 top-level installed |
| 11 | crates.io API `https://crates.io/api/v1/crates/<name>` (12 crates) | max_version + repository URL confirmed for each; `license` field returned `null` (recorded as API observation) |
| 12 | npm registry `https://registry.npmjs.org/<pkg>/<version>` (10 packages) | licenses: MIT ×9, Apache-2.0 ×1 |
| 13 | GitHub API `https://api.github.com/repos/<owner>/<repo>` (16 repos) | SPDX: MIT ×11, Apache-2.0 ×3 (petgraph, reqwest, TypeScript, Playwright→4), Unlicense ×1 (w64devkit) — fetched 2026-11-06 |
| 14 | `Select-String` cross-refs in trace docs | QA-HARNESS §3 backend routes; A11Y plan token mirror; live/README evidence paths (web/src/panels/DataPanel.tsx:44) |

## Files that could not be fully read

| file | reason | disposition |
|---|---|---|
| `docs/trace/controls.json` (73,765 B) | size; read as parsed JSON (schema, counts, every control surface_id/status/hops verdict histogram) rather than literal full text | structure + counts VERIFIED; per-hop evidence prose not quoted line-by-line |
| `docs/trace/live/inspect.json` (10,462 B), `trace-capture.json` (22,492 B) | read structure/keys and key facts; not every nested evidence string quoted | counts/facts VERIFIED as recorded |
| 9 PNG assets | binary | dimension + sha256 verified; pixels UNVERIFIED |
| `docs/diagrams/*.svg` (5) | large generated SVG | header/viewBox/labels + sha verified; full 40-218 KB CSS not quoted |

## Hash verification

- All 49 assigned files' current sha256 were compared to `FILE-MANIFEST.csv` during extraction (seat entries supplied bytes+sha; no mismatch surfaced in the extraction join).
- Manifest rows for the seat are the authoritative hashes in `FILES.md` metadata blocks.

## Scope discipline

- Writes confined to `audit/parts/ui-watchdog/{FILES.md,DEPENDENCIES.md,PROVENANCE.md,QUALITY.csv,FINDINGS.md,EVIDENCE.md}` (this file set). No product, test, docs, ledger or git writes.
- Network use was limited to official registries/APIs listed above; no installs, no paid calls, no rebuilds.
