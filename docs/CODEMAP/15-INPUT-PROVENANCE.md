# 15 — Supplied-Input Provenance and Reconciliation

## The three supplied artifacts

| Input | Size | SHA-256 | What it is |
|---|---:|---|---|
| `Gitingesthivemind.txt` | 5,007,714 B | `F681ABD04638EF394A9625EEFB14E3B4B7BB19E532DA56E769CEE620AEFFF761` | Git-ingest capture, 102,926 physical lines, **635 real `FILE:` sections** |
| `rodneydanger1-hivemind-8a5edab282632443.txt` | 5,007,714 B | same hash | **Byte-identical duplicate** |
| `rodneydanger1_hivemind architecture diagram_ how it works.html` | 171,501 B | `0B857D60E8457FE06D15C783F4985728014BEF918CD181CC3CF9E5437C3FDC4B` | Saved GitDiagram page: 25 components / 30 connections / 40 sampled files |

The two text files are one dump plus one duplicate, not two independent sources.

## What the dump is — corrected

The dump is a **capture of the `e021b12` working tree**, not a 1.0-era repository and not the current `aa16e13`.

| Evidence | Result |
|---|---|
| `docs/CODEMAP/01-SCOPE-AND-CENSUS.md` inside dump | Records commit `e021b12`, source `1.2.0-dev`, 805 tracked files, `git status ?? docs/CODEMAP/` |
| `git cat-file -t e021b12` | commit exists locally |
| `git ls-tree -r e021b12` | 805 files |
| `git diff --name-status e021b12..HEAD` | 18 additions under `docs/CODEMAP/`, 0 deletions/modifications |
| `805 + 18 = 823` | current `git ls-files` |
| Captured content parity | for 620 captured files that exist at `e021b12`, normalized SHA-256 differences = **0** |
| `8a5edab282632443` from the second filename | not an object in local history; remote/rewritten identity |

The **audit paper inside the dump** is a historical `v1.0.0` audit at `3988622`/507 files. That is a carried-forward document, not the capture identity. It carries its own supersession note for the P0 findings.

## Dump section inventory

| Measure | Value |
|---|---:|
| Delimited `FILE:` sections | **635** |
| Header tree files | 635 |
| Duplicate paths | 0 |
| Lines containing `FILE:` | 636 — the extra is `DEFAULT_DB_FILE: &str` in source |
| Payload | 4,881,819 B / 98,380 lines |
| Sections still present at HEAD | **635 / 635** |
| Dump-only paths | 0 |
| Current tracked files with no dump section | **188** (89 text, 99 binary/UTF-16) |
| Content identical after normalization | 586 |
| Content materially changed | 1 (`docs/CODEMAP/00-START-HERE.md`) |
| Empty sentinels | 28 |
| Binary sentinels | 20 |

By area: docs 174, plans 109, backend 106, audit **69**, web 69, runtime-test 46, e2e 37, tools 19, root 3, ledgers 2, `.cargo` 1 = 635.

## What the dump does **not** contain

- 188 current tracked files, including 99 binaries/UTF-16 and 89 text files.
- `backend/crates/hiverouter-core/src/execution.rs`, `ws.rs`, `tools/src/approval.rs`, and other current core files are among the 188 extras. The dump is a **strict subset**; this is a selection/export limitation, not evidence those files were absent.
- The audit’s own `FILE-MANIFEST*`, `QUALITY-MATRIX.csv`, `APPENDIX-FILE-CENSUS.md`, and `backend/Cargo.lock` are absent, so its 507-file hash claim is read, not self-verified.
- 49 bodies are placeholders (28 `[Empty file]`, 20 `[Binary file]`); one “binary” is actually Markdown (`plans/waves/W1-CORE-RECOVERY.md`).
- 4 control characters corrupt identifiers inside the dump’s copy of `plans/foresight/RUN-3.md`.

## GitDiagram provenance

- Generated `2026-11-07T05:48:53.259Z`, 7m36s after local HEAD `aa16e13`; likely the same tree, but the exact commit is not exposed.
- 25 components / 30 connections / 40 source files read; 22/30 connections with evidence.
- All 21 hinted paths exist at HEAD; the diagram’s problems are role labelling and omissions, not renames.
- It omits 90 of 111 tracked source files and all tests; `Run Execution` has no inbound edge, `Run Planning` no outbound edge.
- See [`14-GITDIAGRAM-ARCHITECTURE.md`](14-GITDIAGRAM-ARCHITECTURE.md) for the full component/edge map.

## Reconciliation rules

1. Current truth = source tree + frozen `_data/inventory.csv` + committed gates, not the dump or diagram.
2. The dump is evidence for content at `e021b12`; it is not a current census and not the 1.0 audit.
3. The audit paper inside it is historical; its 30 findings are dispositioned in [`13-HISTORICAL-AUDIT.md`](13-HISTORICAL-AUDIT.md).
4. The diagram is a near-current sampled cross-check; component file labels are corrected in doc 14.
5. No dump count (507 files, 233/79 tests, 63.98 MiB) is presented as current. Current frozen counts: 805 tracked / 36,480 working-tree / 475 backend / 179 web / 106 mocked E2E.
6. After CODEMAP publication the repo has 823 tracked files; the census is frozen before `docs/CODEMAP` existed and both counts are stated with their scope.

## Supporting reports

- [`_incoming/DUMP-ANALYSIS.md`](_incoming/DUMP-ANALYSIS.md) — 635-section parse, 30 findings, 9 fixed / 12 open / 2 partial / 6 unrechecked.
- [`_incoming/DUMP-DELTA.md`](_incoming/DUMP-DELTA.md) — duplicate proof, path-set delta, 188 extras, content parity.
- [`_incoming/DIAGRAM-ANALYSIS.md`](_incoming/DIAGRAM-ANALYSIS.md) — SVG recovery, 25/30/22 counts, mapping conflicts.

## Unverified

- Exact GitDiagram commit; only the generation timestamp and path-existence check support “same tree”.
- `cargo tree -d`; duplicate majors are confirmed by lockfile grep, not the gate.
- Recorded historical test counts; not re-executed in read-only analysis.
- Full hardlink/symlink scan and DB payload contents; not read here.
