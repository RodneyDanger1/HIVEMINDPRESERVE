# WAVE SHEET — HiveMind v1 exhaustive audit (scientific paper)

**Checkpoint:** HEAD `3988622` (git clean) · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Inventory:** `audit/inventory/FILE-MANIFEST.csv|json` (507 tracked files, sha256 each), `ASSIGNMENTS.json` (per-seat file list; read your seat key), `DEEP-LIST.md` (56 deep files), `DIRECTORY-CENSUS.md`, `CODE-STATS.md`.
**Rule:** this is a READ-ONLY audit of product code. No edits outside `audit/` (and no edits to product, tests, docs, ledgers, git). No installs, no rebuilds, no paid calls.

## Scope definition (exhaustive but honest)

1. **All 507 git-tracked files** are in the manifest and assigned to exactly one seat. Each must appear in that seat's `FILES.md`.
2. **Deep files (56)** get an 8–12 paragraph analysis each (functional mechanics, logic/algorithm, edge/failure behavior, dependency interactions, quality/evidence). The list is in `DEEP-LIST.md`; depth per seat: b-core 21, b-provider 15, watchdog-1 16, watchdog-2 4.
3. **Standard files** get a structured record: metadata, purpose, mechanics (2–5 sentences), exact dependencies/callers where applicable, quality classification, evidence. For pure docs/assets/runtime artifacts, the record covers purpose/provenance/role/defects instead of fictional code mechanics.
4. **Ignored/generated classes are censused, not prose-analyzed**: `target/` 13,525 files/8,087 MB; `web/node_modules` 3,976/110 MB; `e2e/node_modules` 194/18 MB; `tools/w64devkit` 9,452/717 MB; `dist/` 6/20 MB; `runtime-test/` 82 tracked/34 MB; `.git/` excluded by convention. State this limitation in the paper.
5. Every path/hash must match the manifest; every behavioral claim needs `path:line` evidence from reading the file. If something cannot be verified, write `UNVERIFIED` + what would prove it. Never infer a function's behavior from its filename.

## Seat assignments (8 seats, disjoint write scopes)

| Seat / teammate | Files | Deep | Partition (from ASSIGNMENTS.json) | Write scope |
|---|---:|---:|---|---|
| **b-core** | 54 | 21 | `backend/crates/hiverouter-contract/**`, `hiverouter-core/**` | `audit/parts/b-core/**` |
| **b-provider** | 33 | 15 | `backend/crates/hiverouter-provider/**`, `hiverouter-tools/**`, `backend/Cargo.toml`, `backend/Cargo.lock` | `audit/parts/b-provider/**` |
| **watchdog-1** | 55 | 16 | `web/**` | `audit/parts/watchdog-1/**` |
| **watchdog-2** | 122 | 4 | `e2e/**`, `tools/**`, `runtime-test/**` | `audit/parts/watchdog-2/**` |
| **figcraft-a** | 103 | 0 | `docs/ui/**`, `ledgers/**` | `audit/parts/figcraft-a/**` |
| **figcraft-b** | 91 | 0 | `docs/plans/**`, `docs/contracts/**`, `docs/claims/**`, `docs/reports/**`, `docs/audit/**` | `audit/parts/figcraft-b/**` |
| **ui-watchdog** | 49 | 0 | `docs/trace/**`, `docs/diagrams/**`, root configs; + dependency provenance research | `audit/parts/ui-watchdog/**` |
| **strip-watchdog** | 0 primary | — | independent verification of all seats + fact-check | `audit/parts/strip-watchdog/**` |

## Required outputs per seat

- `audit/parts/<seat>/FILES.md` — every assigned file, one section per file: `## <path>` then metadata block (bytes, sha256 from manifest, language, category, seat, depth), purpose, mechanics, dependency/caller matrix (`path:line` when known), quality classification, evidence/UNVERIFIED. Deep files 8–12 paragraphs; standard files a structured 1–2 paragraph record.
- `audit/parts/<seat>/DEPENDENCIES.md` — local import/call graph for the partition, external crates/npm in use (from manifests/lockfiles you read), and cross-partition links.
- `audit/parts/<seat>/QUALITY.csv` — columns `path,correctness,maintainability,modularity,error_handling,security,docs,evidence,notes` (1–5 each; evidence = path:line/test/log).
- `audit/parts/<seat>/FINDINGS.md` — defects, debt, risks, provenance/donor evidence, and items for the migration roadmap; each with evidence and severity.
- `audit/parts/<seat>/EVIDENCE.md` — commands run (recon/grep/read/hash checks) and their outputs; list any file you could not read.
- Optional `audit/parts/<seat>/diagrams/*.mmd` if useful; final diagrams are compiled by Lead/weaver.
- Final chat receipt ≤20 lines: files covered / total assigned, deep count, quality totals, top findings, unverified list.

## Skills to load (use the skill tool)

- b-core, b-provider: `forgewright-lead`, `integration-logic`, `smartfix`, `forgewright-recon`.
- watchdog-1: `forgewright-lead`, `integration-logic`, `smartfix`, `forgewright-recon`.
- watchdog-2: `trace`, `smartfix`, `auditor`, `forgewright-recon`.
- figcraft-a: `weaver`, `forge`, `truthseeker`, `forgewright-recon`.
- figcraft-b: `forgewright-recon`, `auditor`, `weaver`.
- ui-watchdog: `research` (forgewright-research), `truthseeker`, `forgewright-recon`, `weaver`.
- strip-watchdog: `auditor`, `truthseeker`, `integration-logic`, `trace`.

## Timebox and loop

- Write incrementally; do not wait for a polished final file. At ~25 minutes, finish the current file, write receipts, return. Lead checks every ≤2 min and keeps compiling.
- No product writes, no git writes, no rebuilds. If a claim needs execution, perform read-only commands only (`Get-FileHash`, `Select-String`, `git show`, `cargo metadata --no-deps` is read-only but do not rebuild).
- If a file is unreadable/binary: record metadata + class + `UNVERIFIED (binary/generated)`; do not invent content.

## Paper integration (Lead)

The Lead compiles: `audit/HIVEMIND-AUDIT-PAPER.md` (Abstract, Introduction, Architecture & Topology, Empirical File Analysis, Upstream Provenance & Donor Lineage, Code Quality & Mathematical/Logical Integrity, Refactoring & Migration Roadmap, Conclusion), `audit/QUALITY-MATRIX.csv`, `audit/ROADMAP-TASKS.csv` (101 tasks across 8 seats / 6 phases), `audit/diagrams/*.mmd`, `audit/APPENDIX-FILE-CENSUS.md`, and `audit/VERIFICATION.md` (strip-watchdog). Seats' `FILES.md` files are the empirical appendix evidence.
