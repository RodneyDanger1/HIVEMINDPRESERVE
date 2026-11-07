# DUMP-DELTA — historical Git-ingest dump vs current `hiverouter-v1` tree

Read-only reconciliation of the two supplied dump files against this repository at
`HEAD aa16e1347cccc4c1d079e8c51d505ae08feb8a1c` (823 tracked files).
No product file was modified; this document is the only write.

Generated: 2026-11-07. Scope: `git ls-files` (tracked only), plus a noted check of the
untracked `docs/CODEMAP/13-15` files. Binary/UTF-16 payloads are classified, not compared.

---

## 1. Duplicate proof

The two inputs are the same byte sequence, not two sources.

| Input | Bytes | SHA-256 | Lines |
|---|---:|---|---:|
| `C:\Users\Riley\Downloads\Gitingesthivemind.txt` | 5,007,714 | `f681abd04638ef394a9625eefb14e3b4b7bb19e532da56e769cee620aefff761` | 102,926 |
| `C:\Users\Riley\Downloads\rodneydanger1-hivemind-8a5edab282632443.txt` | 5,007,714 | `f681abd04638ef394a9625eefb14e3b4b7bb19e532da56e769cee620aefff761` | 102,927 |

- `Get-FileHash -Algorithm SHA256` on both files: `F681ABD04638EF394A9625EEFB14E3B4B7BB19E532DA56E769CEE620AEFFF761`.
- Byte-for-byte stream comparison of the full length: **5,007,714 / 5,007,714 bytes compared, 0 differing bytes**.
- Hashes equal **and** bytes equal: the second file is a pure duplicate. A single copy is analysed below.

### Entry count: 635, not 636

The dump writes each file as `"="*48` / `FILE: <path>` / `"="*48` / content. Parsing that
structure yields **635 sections**, matching both the embedded directory tree and
unique-path check:

| Measure | Count |
|---|---:|
| Delimited `FILE:` sections (regex `^={48}\nFILE: .+\n={48}`) | **635** |
| Files listed in the header directory tree | **635** |
| Tree paths having no content section | 0 |
| Duplicate paths | 0 |
| Lines containing the substring `FILE:` | 636 |

The 636th hit is a Rust constant inside a captured source file
(`pub const DEFAULT_DB_FILE: &str = "hiverouter.db";`), not a section. The earlier
`CODEMAP/15-INPUT-PROVENANCE.md` table (sum 636, `audit/` = 70) over-counts `audit/` by one;
actual `audit/` sections are **69**. `635` is the authoritative count everywhere below.

## 2. What the dump actually is (provenance, corrected)

The dump mixes documents of three vintages. Only one describes the capture point.

| Embedded claim | Where it comes from | Verdict |
|---|---|---|
| "HEAD `3988622` · 507 tracked files · tag `v1.0.0`" | `audit/HIVEMIND-AUDIT-PAPER.md` (dump line 909, 989) | **Historical audit snapshot, superseded in its own text** ("Post-release supersession 2026-11-06") |
| "sealed release 1.1.0" | `README.md` (dump line 772) | Version-truth label for the released 1.1.0; the same README says the tree is on an unreleased 1.2 pre-release |
| "Commit `e021b12e5bb7...`, source version `1.2.0-dev`, 805 tracked, `git status` = `?? docs/CODEMAP/`" | `docs/CODEMAP/01-SCOPE-AND-CENSUS.md` (dump line 45759) | **Actual capture point** |

Corroboration (`git` against this repository):

- `git cat-file -t e021b12e5bb7e5334c71012bcf7a8810d91b9e58` → `commit` (exists locally).
- `git ls-tree -r --name-only e021b12...` → **805** files; four crate manifests and
  `web/package.json` at that commit all read `version = "1.2.0-dev"`.
- `git diff --name-status e021b12..HEAD` → exactly **18 additions**, all under
  `docs/CODEMAP/`; 0 deletions, 0 modifications of previously tracked files.
- `805 + 18 = 823` = current `git ls-files` count, so nothing else changed.
- At `e021b12` the `docs/CODEMAP/` directory existed on disk but was **untracked**; the dump
  includes 15 of its 18 files, which is only possible for an untracked-at-capture source.
- `8a5edab282632443` (from the second filename) is **not an object in this repository**
  (`fatal: Not a valid object name`); it is the remote/rewritten identity, not a local commit.

**Content fidelity proof:** for all **572** captured files that exist in the `e021b12`
tree and carry real content (the other 48 are the placeholder sections of section 4), the
normalized SHA-256 equals the committed blob.
Computed differences: **0**. The dump's content is a faithful copy of the `e021b12`
working tree; only its *selection* is incomplete.

## 3. Path sets — missing / extra / changed, by area

Normalization used for comparison: decode UTF-8, strip a leading BOM, convert `CRLF`/`CR`
to `LF`, strip leading/trailing newlines, then SHA-256. Line-ending-only and BOM-only
differences therefore never register as changes.

### 3.1 Set arithmetic

| Direction | Count | Meaning |
|---|---:|---|
| Dump sections → current tree, **missing** | **0** | Every one of the 635 sections still exists at its exact path |
| Renamed between dump and current | **0** | No path exists only in the dump, so no rename can be hiding |
| Current tracked files with **no dump section** ("extra") | **188** | 89 text + 99 binary/UTF-16 |
| Dump sections whose content the dump did not capture | **48** | 28 `[Empty file]` + 20 `[Binary file]` |
| Dump sections whose text materially changed | **1** | `docs/CODEMAP/00-START-HERE.md` |
| Dump sections identical after normalization | **586** | Content-equal to the current file |

`586 + 28 + 20 + 1 = 635`. The 28 `[Empty file]` sections point at files that are also
0 bytes today, so they are *not* deltas: the effective unchanged set is **614**
(586 identical + 28 empty).

### 3.2 Sections by area

| Area | Dump sections | Current tracked | Identical | Changed | Empty sentinel | Binary sentinel | Extra (no section) | …text | …binary |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| docs | 174 | 278 | 151 | 1 | 16 | 6 | 104 | 47 | 57 |
| plans | 109 | 114 | 108 | 0 | 0 | 1 | 5 | 5 | 0 |
| audit | 69 | 93 | 69 | 0 | 0 | 0 | 24 | 18 | 6 |
| backend/hiverouter-core | 50 | 52 | 50 | 0 | 0 | 0 | 2 | 2 | 0 |
| runtime-test | 46 | 82 | 21 | 0 | 12 | 13 | 36 | 6 | 30 |
| web/src | 44 | 45 | 44 | 0 | 0 | 0 | 1 | 1 | 0 |
| e2e | 37 | 39 | 37 | 0 | 0 | 0 | 2 | 2 | 0 |
| backend/hiverouter-tools | 23 | 25 | 23 | 0 | 0 | 0 | 2 | 2 | 0 |
| web/tests | 21 | 21 | 21 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools | 19 | 23 | 19 | 0 | 0 | 0 | 4 | 0 | 4 |
| backend/hiverouter-provider | 17 | 17 | 17 | 0 | 0 | 0 | 0 | 0 | 0 |
| backend/hiverouter-contract | 15 | 16 | 15 | 0 | 0 | 0 | 1 | 1 | 0 |
| web | 4 | 5 | 4 | 0 | 0 | 0 | 1 | 1 | 0 |
| (root) | 3 | 4 | 3 | 0 | 0 | 0 | 1 | 1 | 0 |
| ledgers | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| .cargo | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| backend | 1 | 2 | 1 | 0 | 0 | 0 | 1 | 1 | 0 |
| dist | 0 | 4 | 0 | 0 | 0 | 0 | 4 | 2 | 2 |
| **Total** | 635 | 823 | 586 | 1 | 28 | 20 | 188 | 89 | 99 |

### 3.3 The 188 current files with no dump section ("extra")

89 are valid UTF-8 text; 99 are binary or UTF-16 and are not meaningfully diffable.
Full list, grouped by area (`text`/`binary`/`utf16` = encoding as read from the current tree):

**`docs` — 104**

- `docs/CODEMAP/08-FILE-CATALOG.md` — text, 152,428 B
- `docs/CODEMAP/_data/inventory.csv` — text, 3,334,920 B
- `docs/CODEMAP/_data/inventory.json` — text, 6,230,951 B
- `docs/audit/evidence/final/cli-diagnose-final.out` — utf16, 6,246 B
- `docs/audit/evidence/final/cli-health-final.out` — utf16, 1,452 B
- `docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/test-failed-1.png` — binary, 4,254 B
- `docs/audit/evidence/phase-b/cli/census.out` — utf16, 4,968 B
- `docs/audit/evidence/phase-b/cli/diagnose_live.out` — utf16, 964 B
- `docs/audit/evidence/phase-b/cli/health_dryrun.out` — utf16, 568 B
- `docs/audit/evidence/phase-b/cli/health_live.out` — utf16, 1,450 B
- `docs/audit/evidence/phase-b/cli/preflight-path.out` — utf16, 1,374 B
- `docs/audit/evidence/phase-b/cli/preflight.out` — utf16, 1,640 B
- `docs/audit/evidence/phase-b/cli/provider_offpeak.out` — utf16, 1,824 B
- `docs/audit/evidence/phase-b/cli/provider_peak.out` — utf16, 1,816 B
- `docs/audit/evidence/phase-b/cli/schema.out` — utf16, 4,000 B
- `docs/audit/evidence/phase-b/cli/tool_missing.out` — utf16, 834 B
- `docs/audit/evidence/phase-b/cli/tools_describe.out` — utf16, 1,934 B
- `docs/audit/evidence/phase-b/cli/tools_list.out` — utf16, 2,432 B
- `docs/audit/evidence/phase-b/cli/unknown.out` — utf16, 824 B
- `docs/audit/evidence/phase-b/cli/usage_error.out` — utf16, 704 B
- `docs/audit/evidence/phase-b/cli/version-human.out` — text, 0 B
- `docs/audit/evidence/phase-b/cli/version.out` — utf16, 818 B
- `docs/audit/evidence/toolchain/hello.exe` — binary, 49,876 B
- `docs/audit/evidence/toolchain/hello_rs.exe` — binary, 5,088,192 B
- `docs/diagrams/agent-lifecycle-state.png` — binary, 136,871 B
- `docs/diagrams/agent-lifecycle-state.svg` — text, 72,503 B
- `docs/diagrams/architecture.png` — binary, 120,208 B
- `docs/diagrams/architecture.svg` — text, 41,481 B
- `docs/diagrams/er-schema.png` — binary, 119,367 B
- `docs/diagrams/er-schema.svg` — text, 218,998 B
- `docs/diagrams/run-sequence.png` — binary, 149,891 B
- `docs/diagrams/run-sequence.svg` — text, 47,817 B
- `docs/diagrams/ui-control-event-sequence.png` — binary, 98,011 B
- `docs/diagrams/ui-control-event-sequence.svg` — text, 43,918 B
- `docs/evidence/live-1.1-paid.log` — text, 1,586 B
- `docs/trace/controls.json` — text, 119,024 B
- `docs/trace/live/app-boot-full.png` — binary, 112,923 B
- `docs/trace/live/app-boot.png` — binary, 112,876 B
- `docs/trace/live/captures/w12/CAP-S22-RECHECK/shot.png` — binary, 99,422 B
- `docs/trace/live/live-app/boot.png` — binary, 99,367 B
- `docs/trace/live/live-app/live-security-foreign-browser.png` — binary, 5,960 B
- `docs/trace/live/live-app/surface-census.png` — binary, 99,367 B
- `docs/ui/figcraft/A/build/build-state.json` — text, 103,383 B
- `docs/ui/figcraft/A/build/c4.png` — binary, 86,238 B
- `docs/ui/figcraft/A/build/fig.py` — text, 3,380 B
- `docs/ui/figcraft/A/build/gen_build.py` — text, 10,259 B
- `docs/ui/figcraft/A/build/gen_common.py` — text, 21,202 B
- `docs/ui/figcraft/A/build/gen_manifest.py` — text, 17,760 B
- `docs/ui/figcraft/A/build/page00.figcraft` — binary, 78,099 B
- `docs/ui/figcraft/A/build/page00g.figcraft` — binary, 78,540 B
- `docs/ui/figcraft/A/build/page01.figcraft` — binary, 88,448 B
- `docs/ui/figcraft/A/build/page01g.figcraft` — binary, 88,784 B
- `docs/ui/figcraft/A/build/page02.figcraft` — binary, 115,696 B
- `docs/ui/figcraft/A/build/page02g.figcraft` — binary, 116,787 B
- `docs/ui/figcraft/A/build/page03.figcraft` — binary, 126,837 B
- `docs/ui/figcraft/A/build/page03g.figcraft` — binary, 127,191 B
- `docs/ui/figcraft/A/build/pages_brand.py` — text, 16,379 B
- `docs/ui/figcraft/A/build/pages_components.py` — text, 43,816 B
- `docs/ui/figcraft/A/build/pages_foundations.py` — text, 13,873 B
- `docs/ui/figcraft/A/build/pages_specimens.py` — text, 18,950 B
- `docs/ui/figcraft/A/build/probe-s04c.png` — binary, 23,088 B
- `docs/ui/figcraft/A/build/stage0.figcraft` — binary, 1,086 B
- `docs/ui/figcraft/A/build/stage1.figcraft` — binary, 4,967 B
- `docs/ui/figcraft/A/build/styles.figcraft` — binary, 80,405 B
- `docs/ui/figcraft/A/scripts/02_foundations.json` — text, 76,384 B
- `docs/ui/figcraft/A/scripts/07_components.json` — text, 200,344 B
- `docs/ui/figcraft/A/scripts/09_specimens.json` — text, 68,015 B
- `docs/ui/figcraft/B/build/__pycache__/board_default.cpython-314.pyc` — binary, 29,494 B
- `docs/ui/figcraft/B/build/__pycache__/board_hero.cpython-314.pyc` — binary, 15,308 B
- `docs/ui/figcraft/B/build/__pycache__/board_interaction.cpython-314.pyc` — binary, 25,009 B
- `docs/ui/figcraft/B/build/__pycache__/board_zoom.cpython-314.pyc` — binary, 16,839 B
- `docs/ui/figcraft/B/build/__pycache__/boards_common.cpython-314.pyc` — binary, 26,082 B
- `docs/ui/figcraft/B/build/__pycache__/fig_b.cpython-314.pyc` — binary, 4,996 B
- `docs/ui/figcraft/B/build/_run.json` — text, 17,355 B
- `docs/ui/figcraft/B/build/board_default.py` — text, 22,086 B
- `docs/ui/figcraft/B/build/board_hero.py` — text, 11,816 B
- `docs/ui/figcraft/B/build/board_interaction.py` — text, 19,172 B
- `docs/ui/figcraft/B/build/board_zoom.py` — text, 11,922 B
- `docs/ui/figcraft/B/build/boards_common.py` — text, 19,072 B
- `docs/ui/figcraft/B/build/build-state.json` — text, 29,301 B
- `docs/ui/figcraft/B/build/build_b.py` — text, 9,962 B
- `docs/ui/figcraft/B/build/fig_b.py` — text, 2,776 B
- `docs/ui/figcraft/B/build/info-B.json` — text, 1,468 B
- `docs/ui/figcraft/B/build/make_manifest.py` — text, 14,329 B
- `docs/ui/figcraft/B/build/patch3.py` — text, 15,487 B
- `docs/ui/figcraft/B/build/patch4.py` — text, 8,394 B
- `docs/ui/figcraft/B/build/patch5.py` — text, 2,467 B
- `docs/ui/figcraft/B/build/patch_default.py` — text, 7,268 B
- `docs/ui/figcraft/B/build/patch_default2.py` — text, 3,177 B
- `docs/ui/figcraft/B/build/patch_manifest.py` — text, 832 B
- `docs/ui/figcraft/B/build/patch_manifest2.py` — text, 1,387 B
- `docs/ui/figcraft/B/build/patch_verify.py` — text, 2,801 B
- `docs/ui/figcraft/B/build/tmp/board00.figcraft` — binary, 96,529 B
- `docs/ui/figcraft/B/build/tmp/board01.figcraft` — binary, 154,725 B
- `docs/ui/figcraft/B/build/tmp/board02.figcraft` — binary, 184,409 B
- `docs/ui/figcraft/B/build/tmp/board03.figcraft` — binary, 220,688 B
- `docs/ui/figcraft/B/build/tmp/effects.figcraft` — binary, 226,734 B
- `docs/ui/figcraft/B/build/tmp/step0.figcraft` — binary, 479 B
- `docs/ui/figcraft/B/build/tmp/step1.figcraft` — binary, 3,500 B
- `docs/ui/figcraft/B/build/verify_b.py` — text, 8,954 B
- `docs/ui/figcraft/B/scripts/02_canvas_default.json` — text, 98,178 B
- `docs/ui/figcraft/B/scripts/03_zoom_states.json` — text, 104,367 B
- `docs/ui/figcraft/B/scripts/04_interaction.json` — text, 83,317 B
- `docs/ui/figcraft/B/scripts/05_hero.json` — text, 74,872 B

**`runtime-test` — 36**

- `runtime-test/data/hiverouter.db` — binary, 4,096 B
- `runtime-test/data/hiverouter.db-wal` — binary, 94,792 B
- `runtime-test/data2/hiverouter.db` — binary, 4,096 B
- `runtime-test/data2/hiverouter.db-wal` — binary, 94,792 B
- `runtime-test/data3/hiverouter.db` — binary, 4,096 B
- `runtime-test/data3/hiverouter.db-wal` — binary, 94,792 B
- `runtime-test/data4/hiverouter.db` — binary, 4,096 B
- `runtime-test/data4/hiverouter.db-wal` — binary, 152,472 B
- `runtime-test/data5/hiverouter.db` — binary, 4,096 B
- `runtime-test/data5/hiverouter.db-wal` — binary, 94,792 B
- `runtime-test/data6/hiverouter.db` — binary, 4,096 B
- `runtime-test/data6/hiverouter.db-wal` — binary, 152,472 B
- `runtime-test/data7/hiverouter.db` — binary, 4,096 B
- `runtime-test/data7/hiverouter.db-wal` — binary, 127,752 B
- `runtime-test/data8/hiverouter.db` — binary, 4,096 B
- `runtime-test/data8/hiverouter.db-wal` — binary, 127,752 B
- `runtime-test/data9/hiverouter.db` — binary, 4,096 B
- `runtime-test/data9/hiverouter.db-wal` — binary, 185,432 B
- `runtime-test/pkg-cli-final/data/hiverouter.db` — binary, 4,096 B
- `runtime-test/pkg-cli-final/data/hiverouter.db-wal` — binary, 127,752 B
- `runtime-test/pkg-cli-final/hiverouter-cli.exe` — binary, 5,364,224 B
- `runtime-test/pkg-cli-final/hiverouter.exe` — binary, 9,329,152 B
- `runtime-test/pkg-final/data/hiverouter.db` — binary, 4,096 B
- `runtime-test/pkg-final/data/hiverouter.db-wal` — binary, 127,752 B
- `runtime-test/pkg-final/hiverouter.exe` — binary, 9,329,152 B
- `runtime-test/pkg-final/smoke-data/hiverouter.db` — binary, 4,096 B
- `runtime-test/pkg-final/smoke-data/hiverouter.db-wal` — binary, 127,752 B
- `runtime-test/pkg-final/smoke.out` — text, 583 B
- `runtime-test/pkg-smoke/data/hiverouter.db` — binary, 4,096 B
- `runtime-test/pkg-smoke/data/hiverouter.db-wal` — binary, 94,792 B
- `runtime-test/pkg-smoke/hiverouter.exe` — binary, 9,099,776 B
- `runtime-test/s5.out` — text, 553 B
- `runtime-test/s6.out` — text, 4,836 B
- `runtime-test/s7.out` — text, 553 B
- `runtime-test/s8.out` — text, 553 B
- `runtime-test/s9.out` — text, 553 B

**`audit` — 24**

- `audit/APPENDIX-FILE-CENSUS.md` — text, 53,003 B
- `audit/QUALITY-MATRIX.csv` — text, 62,493 B
- `audit/diagrams/01-system-topology.png` — binary, 65,034 B
- `audit/diagrams/01-system-topology.svg` — text, 36,642 B
- `audit/diagrams/02-data-flow.png` — binary, 51,310 B
- `audit/diagrams/02-data-flow.svg` — text, 35,069 B
- `audit/diagrams/03-module-dependencies.png` — binary, 31,591 B
- `audit/diagrams/03-module-dependencies.svg` — text, 20,480 B
- `audit/diagrams/04-state-inventory.png` — binary, 27,216 B
- `audit/diagrams/04-state-inventory.svg` — text, 13,065 B
- `audit/diagrams/05-dependency-lineage.png` — binary, 91,969 B
- `audit/diagrams/05-dependency-lineage.svg` — text, 30,009 B
- `audit/diagrams/06-migration-target.png` — binary, 29,406 B
- `audit/diagrams/06-migration-target.svg` — text, 15,414 B
- `audit/inventory/ASSIGNMENTS.json` — text, 82,443 B
- `audit/inventory/FILE-MANIFEST-ENRICHED.csv` — text, 88,322 B
- `audit/inventory/FILE-MANIFEST-ENRICHED.json` — text, 202,470 B
- `audit/inventory/FILE-MANIFEST.csv` — text, 83,922 B
- `audit/inventory/FILE-MANIFEST.json` — text, 180,844 B
- `audit/parts/figcraft-a/FILES.md` — text, 133,972 B
- `audit/parts/figcraft-b/FILES.md` — text, 91,324 B
- `audit/parts/figcraft-b/_digest.json` — text, 268,759 B
- `audit/parts/watchdog-1/FILES.md` — text, 102,832 B
- `audit/parts/watchdog-2/FILES.md` — text, 114,438 B

**`plans` — 5**

- `plans/foresight/APPROVAL-SUSPENSION-PLAN.md` — text, 58,163 B
- `plans/foresight/CODE-APPROVAL-ENGINE.md` — text, 102,091 B
- `plans/foresight/CODE-APPROVAL-UX.md` — text, 66,041 B
- `plans/foresight/COMPILED-LEDGER.md` — text, 53,097 B
- `plans/foresight/RUN2-ENGINE-CODE.md` — text, 67,831 B

**`dist` — 4**

- `dist/HiveMind-1.0.0-win64.zip` — binary, 8,444,021 B
- `dist/HiveMind-1.0.0-win64.zip.sha256` — text, 66 B
- `dist/HiveMind-1.1.0-win64.zip` — binary, 8,613,122 B
- `dist/HiveMind-1.1.0-win64.zip.sha256` — text, 65 B

**`tools` — 4**

- `tools/bin/hiverouter-cli.exe` — binary, 5,364,224 B
- `tools/bin/hiverouter.exe` — binary, 9,329,152 B
- `tools/cc-test/r.exe` — binary, 5,088,156 B
- `tools/cc-test/t.exe` — binary, 49,876 B

**`backend/hiverouter-core` — 2**

- `backend/crates/hiverouter-core/src/execution.rs` — text, 96,383 B
- `backend/crates/hiverouter-core/src/ws.rs` — text, 53,201 B

**`backend/hiverouter-tools` — 2**

- `backend/crates/hiverouter-tools/src/approval.rs` — text, 58,434 B
- `backend/crates/hiverouter-tools/src/bin/hiverouter.rs` — text, 579 B

**`e2e` — 2**

- `e2e/.gitignore` — text, 125 B
- `e2e/package-lock.json` — text, 1,748 B

**`(root)` — 1**

- `.gitignore` — text, 258 B

**`backend` — 1**

- `backend/Cargo.lock` — text, 100,177 B

**`backend/hiverouter-contract` — 1**

- `backend/crates/hiverouter-contract/schemas/contract-v1.json` — text, 61,221 B

**`web` — 1**

- `web/package-lock.json` — text, 78,346 B

**`web/src` — 1**

- `web/src/state/store.ts` — text, 76,595 B

**Product-code files in this set** (the material part):

| Path | Bytes @ e021b12 | Why absent from the dump |
|---|---:|---|
| `backend/crates/hiverouter-core/src/execution.rs` | 96,383 | content exceeds the ~50 KB capture ceiling |
| `backend/crates/hiverouter-core/src/ws.rs` | 53,201 | content exceeds the ~50 KB capture ceiling |
| `backend/crates/hiverouter-tools/src/approval.rs` | 58,434 | content exceeds the ~50 KB capture ceiling |
| `web/src/state/store.ts` | 76,595 | content exceeds the ~50 KB capture ceiling |
| `backend/crates/hiverouter-contract/schemas/contract-v1.json` | 61,221 | content exceeds the ~50 KB capture ceiling |
| `backend/Cargo.lock` | 100,177 | content exceeds the ~50 KB capture ceiling |
| `backend/crates/hiverouter-tools/src/bin/hiverouter.rs` | 579 | **single unexplained omission** (see §5.1) |

### 3.4 The one materially changed text file

| Path | Dump lines → current lines | Lines removed / added | Similarity | Nature of change |
|---|---:|---:|---:|---|
| `docs/CODEMAP/00-START-HERE.md` | 42 → 46 | 1 / 5 | 0.932 | index extended from 11 to 15 entries (adds `12-GITHUB-PUSH`, `13-HISTORICAL-AUDIT`, `14-GITDIAGRAM-ARCHITECTURE`, `15-INPUT-PROVENANCE`) and item 11 reworded |

No other captured text section differs from the current file beyond line endings/BOM.

## 4. Sections whose content the dump does not carry (48)

The dump writes `[Empty file]` / `[Binary file]` in place of content. These are
**content-availability failures, not deltas**, and must not be read as evidence about the file.

### 4.1 `[Empty file]` — 28 sections

All 28 target paths are 0 bytes in the current tree, so the placeholder is semantically
correct. Paths: `docs/audit/evidence/final/cli-diagnose-final.err`, `docs/audit/evidence/phase-b/cli/census.err`, `docs/audit/evidence/phase-b/cli/diagnose_live.err`, `docs/audit/evidence/phase-b/cli/health_dryrun.err`, `docs/audit/evidence/phase-b/cli/health_live.err`, `docs/audit/evidence/phase-b/cli/preflight-path.err`, `docs/audit/evidence/phase-b/cli/preflight.err`, `docs/audit/evidence/phase-b/cli/provider_offpeak.err`, `docs/audit/evidence/phase-b/cli/provider_peak.err`, `docs/audit/evidence/phase-b/cli/schema.err`, `docs/audit/evidence/phase-b/cli/tool_missing.err`, `docs/audit/evidence/phase-b/cli/tools_describe.err`, `docs/audit/evidence/phase-b/cli/tools_list.err`, `docs/audit/evidence/phase-b/cli/unknown.err`, `docs/audit/evidence/phase-b/cli/usage_error.err`, `docs/audit/evidence/phase-b/cli/version.err`, `runtime-test/pkg-final/err.txt`, `runtime-test/pkg-final/smoke.err`, `runtime-test/pkg-smoke/err.txt`, `runtime-test/s5.err`, `runtime-test/s6.err`, `runtime-test/s7.err`, `runtime-test/s8.err`, `runtime-test/s9.err`, `runtime-test/server.err.txt`, `runtime-test/server2.err.txt`, `runtime-test/server3.err.txt`, `runtime-test/server4.err.txt`.

### 4.2 `[Binary file]` — 20 sections

19 point at genuinely binary/UTF-16 payloads (Playwright `trace.zip`, SQLite `*.db-shm`,
lossy `.out.txt`, a UTF-16 `.err`, a UTF-16 `census-verify.json`). The 20th is a
**false positive on valid UTF-8 text**:

- `plans/waves/W1-CORE-RECOVERY.md` — 10,001 B at `e021b12`, 120 lines, 86 non-ASCII bytes,
  **no NUL, no control bytes, strict UTF-8 decode succeeds**. The dump replaced its content
  with `[Binary file]`, so its body is unrecoverable from the dump (the current file is intact).

Paths: `docs/audit/evidence/final/live-probe-final.out.txt`, `docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/trace.zip`, `docs/audit/evidence/phase-b/cli/census-verify.json`, `docs/audit/evidence/phase-b/cli/version-human.err`, `docs/audit/evidence/phase-b/live-probe-verifier-v2.out.txt`, `docs/audit/evidence/phase-b/live-probe-verifier.out.txt`, `plans/waves/W1-CORE-RECOVERY.md`, `runtime-test/data/hiverouter.db-shm`, `runtime-test/data2/hiverouter.db-shm`, `runtime-test/data3/hiverouter.db-shm`, `runtime-test/data4/hiverouter.db-shm`, `runtime-test/data5/hiverouter.db-shm`, `runtime-test/data6/hiverouter.db-shm`, `runtime-test/data7/hiverouter.db-shm`, `runtime-test/data8/hiverouter.db-shm`, `runtime-test/data9/hiverouter.db-shm`, `runtime-test/pkg-cli-final/data/hiverouter.db-shm`, `runtime-test/pkg-final/data/hiverouter.db-shm`, `runtime-test/pkg-final/smoke-data/hiverouter.db-shm`, `runtime-test/pkg-smoke/data/hiverouter.db-shm`.

## 5. Omission rule reconstructed from the 188 gaps

The dump is not a random subset. Its selection rule is reproducible from the data:

| Rule (inferred) | Files | Evidence |
|---|---:|---|
| Payload is binary / invalid UTF-8 (no section at all) | 82 | `.exe`, `.png`, `.pyc`, `.figcraft`, `.db`, `.db-wal` payloads |
| Payload is UTF-16 (PowerShell redirects) | 17 | `docs/audit/evidence/**/*.out` decode as UTF-16LE |
| Text content above the capture ceiling | 34 | largest captured section = 49,753 B; smallest omitted over-ceiling text = 52,448 B |
| `docs/ui/figcraft/**/build/` artifacts | 29 | generated `.py`/`.json` build scripts |
| `*.svg` (text assets) | 11 | contracted SVG diagrams |
| .out captures | 7 | `runtime-test/s5-s9.out`, `*.out` |
| .gitignore | 2 | root and `e2e/` |
| `package-lock.json` | 2 | `e2e/`, `web/` |
| `*.sha256` | 2 | `dist/` release sums |
| .log | 1 | `docs/evidence/live-1.1-paid.log` |
| **Unexplained small text** | 1 | `backend/crates/hiverouter-tools/src/bin/hiverouter.rs` (579 B) — no path, extension or size rule explains it |
| **Total** | 188 |  |

### 5.1 Why the three "core" files are absent — dump omission, not a pre-1.2 state

This is decidable from the dump alone, without reference to the current tree.

1. **The dump's own module graph requires them.** `backend/crates/hiverouter-core/src/lib.rs`
   (captured in full, 2,174 B) declares `pub mod execution;` (line 16) and `pub mod ws;`
   (line 31), and re-exports `pub use execution::{…}` / `pub use ws::SessionAuth;`.
   `backend/crates/hiverouter-tools/src/lib.rs` declares `pub mod approval;` (line 31).
2. **Captured files import them.** `core/src/http.rs:23` `use crate::execution::{ProviderExecutor, RunEngine, UnconfiguredExecutor};`,
   `core/src/http.rs:30` `use crate::ws::{refusal_response, SessionAuth, ws_handler};`,
   `core/src/provider_executor.rs:21` `use crate::execution::{…}`,
   `core/src/tools_bridge.rs:26` `use hiverouter_tools::approval::{…}` — 94 reference hits across the dump.
   A snapshot that captures `lib.rs` and `http.rs` but not `execution.rs`/`ws.rs`/`approval.rs`
   is internally inconsistent and could not compile. The files existed at the capture commit.
3. **They predate the dump by two releases.** Both `v1.0.0` and `hivemind-1.1.0` trees contain
   `backend/crates/hiverouter-core/src/execution.rs`, `…/ws.rs`, `…/approval.rs`.
   `e021b12` is `1.2.0-dev` — *after* both tags. "Pre-1.2 state" is refuted: this is not an
   earlier tree, it is an incomplete capture of a later one.
4. **The mechanism is size.** Blob sizes at `e021b12`: 96,383 / 53,201 / 58,434 bytes, all above
   the 49,753-byte maximum of any captured section. They fall in the 34-file "text over ceiling" bucket.

**Verdict: dump omission (capture-rule artefact).** Any statement of the form "the 1.1/1.2 backend
has no execution engine / WS transport / approval module" is false. Also note the task's path
`backend/core/src/…` does not exist in either source: the real path is
`backend/crates/hiverouter-core/src/…` (`hiverouter-tools/src/approval.rs`).

## 6. `DO-NOT-USE-AS-CURRENT`

| # | Item | Why | Use instead |
|---:|---|---|---|
| 1 | Any code claim resting on the **absence** of `execution.rs`, `ws.rs`, `approval.rs`, `store.ts`, `contract-v1.json`, `Cargo.lock` | All exist; §5.1 proves omission, not absence | Read the files; they are 53–100 KB, over the capture ceiling |
| 2 | All 20 `[Binary file]` sections, in particular `plans/waves/W1-CORE-RECOVERY.md` | Content replaced by a placeholder; the W1 case is a false positive on valid text | Current tree file (120 lines, intact) |
| 3 | All 28 `[Empty file]` sections as "the file had content" | The files are genuinely 0 bytes | Current tree (also 0 bytes) |
| 4 | Dump `docs/CODEMAP/00-START-HERE.md` | Pre-publication index (11 entries); current index has 15 | Current tracked file |
| 5 | Every count in `audit/*` (507 tracked, 233 backend tests, 79 live E2E, 63.98 MiB) | `v1.0.0`-era snapshot embedded in the dump; its own text marks it superseded | `e021b12` census (805 tracked) for the capture; 823 tracked for `HEAD` |
| 6 | The dump's `runtime-test/**/*.db-shm` sections | Binary SQLite SHM captured as a placeholder; zero analytical value | Regenerate from a run, or ignore |
| 7 | `dist/*.zip`, `tools/bin/*.exe`, `docs/ui/figcraft/**` build artifacts | Payload omitted entirely by the capture rule | Rebuild from source; the dump cannot reconstruct artifacts |
| 8 | "636 files" as a count, and `audit/` = 70 in the prior provenance table | 635 sections, `audit/` = 69 (§1) | This document |
| 9 | Dump line numbers for code that moved after `e021b12` | Content is `e021b12`, current tree is `aa16e13`; only `00-START-HERE.md` text actually differs, but the commit anchor still matters | Cite `e021b12` explicitly when using dump line refs |

## 7. `STILL-VALID-HISTORICAL`

| Item | Validity | Required qualifier |
|---|---|---|
| The **586** sections whose normalized SHA-256 equals the current file | Content-identical to `HEAD`; safe as textual evidence | Cite as "at `e021b12`, unchanged at `aa16e13`" |
| The 28 empty-file sections | Correct (files are empty) | — |
| Dump `docs/CODEMAP/00-START-HERE.md` as a *pre-publication* index | Historically accurate for the 11-entry state | Must not be cited as the current index |
| `audit/HIVEMIND-AUDIT-PAPER.md`, `audit/FINDINGS-REGISTER.md`, quality/roadmap CSVs | Valid **as the `v1.0.0` audit record** (30 findings: 3 P0/10 P1/16 P2/1 info) | Every finding needs a fresh source read before being called open/fixed — the paper itself says so |
| Dump census of `e021b12` (805 tracked / 36,480 working-tree / 35,675 ignored) | True for that commit | Do not quote as current (823 tracked today) |
| Dump `.mmd` diagrams and contract/architecture docs | Source text, unchanged | Only where §3.3 does not list them as omitted |
| Dump copies of the 99 binary/UTF-16 **paths** | The path list is right even though payloads are absent | Payload must come from the tree |

## 8. Summary table

| Metric | Value |
|---|---:|
| Input bytes (each file) | 5,007,714 |
| Input SHA-256 (both) | `F681ABD0…EFFF761` |
| Byte differences | 0 |
| Dump `FILE:` sections | 635 |
| Dump lines | 102,926 |
| Current tracked files (`git ls-files`) | 823 |
| Files at the dump's capture commit `e021b12` | 805 (+18 untracked `docs/CODEMAP` on disk) |
| Dump paths still present in current tree | 635 / 635 (0 missing) |
| Capture completeness | 635 / 823 tracked today (77.2 %); 635 / 805 at `e021b12` (78.9 %) |
| Renames between dump and current | 0 |
| Current tracked files with no dump section | 188 (89 text, 99 binary/UTF-16) |
| Sections identical after normalization | 586 |
| Sections equivalent but stored as `[Empty file]` | 28 |
| Sections with content unavailable (`[Binary file]`) | 20 (19 binary + 1 text false positive) |
| Sections materially changed | 1 (`docs/CODEMAP/00-START-HERE.md`) |
| Product-code files omitted from the dump | 7 (execution, ws, approval, store, contract schema, Cargo.lock, `src/bin`) |
| Core files **absent by omission** | `execution.rs`, `ws.rs`, `approval.rs` — proven dump omission |

## 9. Corrections to previously written CODEMAP claims

| Prior statement | Correction | Proof |
|---|---|---|
| `CODEMAP/15-INPUT-PROVENANCE.md`: "636 `FILE:` entries", area table sums to 636, `audit/` = 70 | **635** sections; `audit/` = **69** | §1; four independent counts agree on 635 |
| `CODEMAP/12-GITHUB-PUSH.md`: "Tracked files on remote `main` = 822" | Current `git ls-files` = **823**; `e021b12..HEAD` adds exactly 18 files to 805 | `git diff --name-status e021b12..HEAD` (18 × `A`) and `git ls-files \| wc -l` = 823 |
| `CODEMAP/15`: "The dump is historical and external" with commit "contested (`3988622` vs `8a5edab`)" | Capture commit is **`e021b12e5bb7e5334c71012bcf7a8810d91b9e58`**, embedded in the dump's own `01-SCOPE-AND-CENSUS.md`, and present as a local object; `3988622` is the audit paper's own older HEAD; `8a5edab` is not local | §2 |

Note: `docs/CODEMAP/13-HISTORICAL-AUDIT.md`, `14-GITDIAGRAM-ARCHITECTURE.md` and
`15-INPUT-PROVENANCE.md` are **untracked** at `HEAD` (they exist on disk but are outside the
823), so they are outside every set computed here and were read only for cross-checking.

## 10. Method and reproducibility

Commands (PowerShell 7 / Python 3, read-only):

```powershell
$a='C:\Users\Riley\Downloads\Gitingesthivemind.txt'
$b='C:\Users\Riley\Downloads\rodneydanger1-hivemind-8a5edab282632443.txt'
Get-FileHash $a,$b -Algorithm SHA256   # equal
# full-length byte compare: 5,007,714 bytes, 0 differing
git -C C:\Users\Riley\Desktop\hiverouter-v1 ls-files | Measure-Object          # 823
git -C C:\Users\Riley\Desktop\hiverouter-v1 ls-tree -r --name-only e021b12 | Measure-Object  # 805
git -C C:\Users\Riley\Desktop\hiverouter-v1 diff --name-status e021b12..HEAD  # 18 x A, all docs/CODEMAP
```

Python: split the dump on the `"="*48` / `FILE:` / `"="*48` delimiter; decode UTF-8; strip
BOM; `CRLF`/`CR` → `LF`; strip edge newlines; SHA-256; `difflib.SequenceMatcher` for line
counts. Classification of the 188 gaps: UTF-16 BOM / NUL-in-first-8000-bytes / strict UTF-8
decode, then per-file size and path/extension patterns.

Limitations, stated as such:

- The capture rule in §5 is **inferred** from 188 observations; no generator script or
  configuration was supplied. `src/bin/hiverouter.rs` remains the one unexplained omission.
- The `[Binary file]` false positive on `W1-CORE-RECOVERY.md` has no established cause; only
  its effect (content unavailable) is proven.
- Binary payloads are classified, never compared; no binary equality is claimed.
- "Materially changed" means "normalized text differs"; whitespace-insensitive equality was
  also checked and found for **0** additional files, so no formatting-only deltas exist.
