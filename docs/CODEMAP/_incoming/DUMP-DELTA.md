# DUMP-DELTA — historical Git-ingest dump vs current `hiverouter-v1` tree

Read-only reconciliation of the two supplied dump files against this repository.
**No product file was modified; this document is the only write.**

Anchor: **`aa16e13`** — the 823-tracked-file state named in the task. The live
checkout has since advanced to `5db2f21` (829 tracked) through documentation-only
commits during this analysis; every difference that drift introduces is listed in §3.4 so
none of it can be mistaken for a dump discrepancy.

---

## 1. Duplicate proof

The two inputs are the same byte sequence, not two sources.

| Input | Bytes | SHA-256 | Newlines |
|---|---:|---|---:|
| `C:\Users\Riley\Downloads\Gitingesthivemind.txt` | 5,007,714 | `f681abd04638ef394a9625eefb14e3b4b7bb19e532da56e769cee620aefff761` | 102,926 |
| `C:\Users\Riley\Downloads\rodneydanger1-hivemind-8a5edab282632443.txt` | 5,007,714 | `f681abd04638ef394a9625eefb14e3b4b7bb19e532da56e769cee620aefff761` | 102,926 |

- `Get-FileHash -Algorithm SHA256` on both: `F681ABD04638EF394A9625EEFB14E3B4B7BB19E532DA56E769CEE620AEFFF761`.
- Full-length byte compare: **5,007,714 / 5,007,714 bytes compared, 0 differing bytes**.
- Equal hash **and** equal bytes: the second file is a pure duplicate. One copy is analysed.

### Entry count: 635, not 636

Each file is written as `"="*48` / `FILE: <path>` / `"="*48` / content. That structure
yields **635 sections**, confirmed four ways:

| Measure | Count |
|---|---:|
| Delimited `FILE:` sections (regex `^={48}\nFILE: .+\n={48}`) | **635** |
| Files listed in the header directory tree | **635** |
| Tree paths with no content section | 0 |
| Duplicate paths | 0 |
| Lines containing the substring `FILE:` | 636 |

The 636th hit is a Rust constant inside a captured source file
(`pub const DEFAULT_DB_FILE: &str = "hiverouter.db";`), not a section. The earlier
`CODEMAP/15-INPUT-PROVENANCE.md` table sums to 636 with `audit/` = 70; the actual `audit/`
count is **69**. **635** is used everywhere below.

## 2. What the dump actually is (provenance, corrected)

The dump bundles documents of three vintages. Only one describes the capture point.

| Embedded claim | Source in the dump | Verdict |
|---|---|---|
| "HEAD `3988622` · 507 tracked · tag `v1.0.0`" | `audit/HIVEMIND-AUDIT-PAPER.md` (dump lines 909, 989) | Historical audit snapshot; its own text marks it superseded ("Post-release supersession 2026-11-06") |
| "sealed release 1.1.0" | `README.md` (dump line 772) | Release label for 1.1.0; the same README says the tree is an unreleased 1.2 line |
| "Commit `e021b12e5bb7...`, source `1.2.0-dev`, 805 tracked, `git status` = `?? docs/CODEMAP/`" | `docs/CODEMAP/01-SCOPE-AND-CENSUS.md` (dump line 45759) | **Actual capture point** |

Corroboration in this repository:

- `git cat-file -t e021b12e5bb7e5334c71012bcf7a8810d91b9e58` → `commit` (local object).
- `git ls-tree -r --name-only e021b12` → **805** files; the four crate manifests and
  `web/package.json` at that commit all read `version = "1.2.0-dev"`.
- `git diff --name-status e021b12..aa16e13` → exactly **18 additions** (all `docs/CODEMAP/`),
  0 deletions, 0 modifications. `805 + 18 = 823` = tracked files at the anchor.
- At `e021b12`, `docs/CODEMAP/` existed on disk but was **untracked**; the dump carries 15
  of its 18 files, which is only possible for an untracked-at-capture source.
- `8a5edab282632443` is **not an object in this repository** (`fatal: Not a valid object
  name`); it is the remote/rewritten identity, not a local commit.
- `3988622` is a local object = the audit paper's older `v1.0.0` HEAD, not the dump's base.

**Content fidelity:** every one of the **587** captured sections that carries real text is
byte-identical (after BOM/line-ending normalization) to its blob at the anchor commit;
**0 differ**. The dump is a faithful copy of what it chose to include — only its *selection*
is partial.

## 3. Path sets — missing / extra / changed, by area

Normalization for every comparison: decode UTF-8 → strip leading BOM → `CRLF`/`CR` → `LF`
→ strip edge newlines → SHA-256. Line-ending-only and BOM-only differences never register
as changes; a whitespace-insensitive pass was also run and found **0** additional deltas.

### 3.1 Set arithmetic (dump vs `aa16e13`, 823 tracked)

| Direction | Count | Meaning |
|---|---:|---|
| Dump sections → tree, **missing** | **0** | Every one of the 635 sections still exists at its exact path |
| Renamed between dump and tree | **0** | No path exists only in the dump, so no rename can be hiding |
| Tracked files with **no dump section** ("extra") | **188** | 89 text + 99 binary/UTF-16 |
| Dump sections whose content the dump did not capture | **48** | 28 `[Empty file]` + 20 `[Binary file]` |
| Dump sections identical to `aa16e13` after normalization | **587** | Content-equal to the tracked file |
| Dump sections materially changed | **0** | No captured text differs from the anchor tree |

`587 + 28 + 20 = 635`. The 28 `[Empty file]` sections point at files that are also
0 bytes in the tree, so they are *not* deltas: the effective unchanged set is **615**.

### 3.2 Sections by area

| Area | Dump sections | Tracked | Identical | Changed | Empty sentinel | Binary sentinel | Extra (no section) | …text | …binary |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| docs | 174 | 278 | 152 | 0 | 16 | 6 | 104 | 47 | 57 |
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
| **Total** | 635 | 823 | 587 | 0 | 28 | 20 | 188 | 89 | 99 |

### 3.3 The 188 tracked files with no dump section ("extra")

89 are valid UTF-8 text; 99 are binary or UTF-16 and are not meaningfully diffable. Full
list by area (`text`/`binary`/`utf16` = encoding read from the tree, bytes = current size):

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

| Path | Bytes | Why the dump has no section |
|---|---:|---|
| `backend/crates/hiverouter-core/src/execution.rs` | 96,383 | text above the ~50 KB capture ceiling |
| `backend/crates/hiverouter-core/src/ws.rs` | 53,201 | text above the ~50 KB capture ceiling |
| `backend/crates/hiverouter-tools/src/approval.rs` | 58,434 | text above the ~50 KB capture ceiling |
| `web/src/state/store.ts` | 76,595 | text above the ~50 KB capture ceiling |
| `backend/crates/hiverouter-contract/schemas/contract-v1.json` | 61,221 | text above the ~50 KB capture ceiling |
| `backend/Cargo.lock` | 100,177 | text above the ~50 KB capture ceiling |
| `backend/crates/hiverouter-tools/src/bin/hiverouter.rs` | 579 | **unexplained** — the only small text omission with no matching rule (§5) |

### 3.4 Live-tree drift after the anchor (not a dump discrepancy)

During this analysis the checkout advanced from `aa16e13` to `5db2f21`
(829 tracked) through documentation commits. At `aa16e13` the dump matched
every captured text section; the live tree now differs in three CODEMAP documents:

| Path | Dump = `aa16e13`? | Live lines | Removed / added | Similarity | Cause |
|---|---|---:|---:|---:|---|
| `docs/CODEMAP/00-START-HERE.md` | identical | 46 | 1 / 5 | 0.932 | CODEMAP documentation rewritten after the capture (commit `5db2f21`) |
| `docs/CODEMAP/09-RISKS-GAPS.md` | identical | 59 | 0 / 24 | 0.745 | CODEMAP documentation rewritten after the capture (commit `5db2f21`) |
| `docs/CODEMAP/12-GITHUB-PUSH.md` | identical | 60 | 0 / 10 | 0.909 | CODEMAP documentation rewritten after the capture (commit `5db2f21`) |

Against the live tree: **584** identical, **3** changed (the three above), 48 sentinels.
The drift files are documentation only; the product-code conclusions are unaffected.

## 4. Sections whose content the dump does not carry (48)

The dump writes `[Empty file]` / `[Binary file]` instead of content. These are
**content-availability gaps, not deltas**, and must not be read as evidence about the file.

### 4.1 `[Empty file]` — 28 sections

All 28 paths are 0 bytes in the tree too, so the placeholder is semantically correct:

`docs/audit/evidence/final/cli-diagnose-final.err`, `docs/audit/evidence/phase-b/cli/census.err`, `docs/audit/evidence/phase-b/cli/diagnose_live.err`, `docs/audit/evidence/phase-b/cli/health_dryrun.err`, `docs/audit/evidence/phase-b/cli/health_live.err`, `docs/audit/evidence/phase-b/cli/preflight-path.err`, `docs/audit/evidence/phase-b/cli/preflight.err`, `docs/audit/evidence/phase-b/cli/provider_offpeak.err`, `docs/audit/evidence/phase-b/cli/provider_peak.err`, `docs/audit/evidence/phase-b/cli/schema.err`, `docs/audit/evidence/phase-b/cli/tool_missing.err`, `docs/audit/evidence/phase-b/cli/tools_describe.err`, `docs/audit/evidence/phase-b/cli/tools_list.err`, `docs/audit/evidence/phase-b/cli/unknown.err`, `docs/audit/evidence/phase-b/cli/usage_error.err`, `docs/audit/evidence/phase-b/cli/version.err`, `runtime-test/pkg-final/err.txt`, `runtime-test/pkg-final/smoke.err`, `runtime-test/pkg-smoke/err.txt`, `runtime-test/s5.err`, `runtime-test/s6.err`, `runtime-test/s7.err`, `runtime-test/s8.err`, `runtime-test/s9.err`, `runtime-test/server.err.txt`, `runtime-test/server2.err.txt`, `runtime-test/server3.err.txt`, `runtime-test/server4.err.txt`.

### 4.2 `[Binary file]` — 20 sections

19 are genuinely binary/UTF-16 payloads (Playwright `trace.zip`, SQLite `*.db-shm`, lossy
`.out.txt`, a UTF-16 `.err`, a UTF-16 `census-verify.json`). The 20th is a **false positive
on valid UTF-8 text**:

- `plans/waves/W1-CORE-RECOVERY.md` — 10,001 B, 120 lines, 86 non-ASCII bytes, **no NUL, no
  control bytes, strict UTF-8 decode succeeds**. The dump still replaced its body with
  `[Binary file]`, so that content is unrecoverable from the dump (the file itself is intact).

Paths:

`docs/audit/evidence/final/live-probe-final.out.txt`, `docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/trace.zip`, `docs/audit/evidence/phase-b/cli/census-verify.json`, `docs/audit/evidence/phase-b/cli/version-human.err`, `docs/audit/evidence/phase-b/live-probe-verifier-v2.out.txt`, `docs/audit/evidence/phase-b/live-probe-verifier.out.txt`, `plans/waves/W1-CORE-RECOVERY.md`, `runtime-test/data/hiverouter.db-shm`, `runtime-test/data2/hiverouter.db-shm`, `runtime-test/data3/hiverouter.db-shm`, `runtime-test/data4/hiverouter.db-shm`, `runtime-test/data5/hiverouter.db-shm`, `runtime-test/data6/hiverouter.db-shm`, `runtime-test/data7/hiverouter.db-shm`, `runtime-test/data8/hiverouter.db-shm`, `runtime-test/data9/hiverouter.db-shm`, `runtime-test/pkg-cli-final/data/hiverouter.db-shm`, `runtime-test/pkg-final/data/hiverouter.db-shm`, `runtime-test/pkg-final/smoke-data/hiverouter.db-shm`, `runtime-test/pkg-smoke/data/hiverouter.db-shm`.

## 5. Omission rule reconstructed from the 188 gaps

The dump is not a random subset; its selection rule is reproducible from the data.

| Rule (inferred from the data) | Files | Evidence |
|---|---:|---|
| Binary / invalid UTF-8 payload (no section at all) | 82 | `.exe`, `.png`, `.pyc`, `.figcraft`, `.db`, `.db-wal` |
| UTF-16 payload (PowerShell redirects) | 17 | `docs/audit/evidence/**/*.out` decode as UTF-16LE |
| Text above the capture ceiling | 34 | largest captured section = 49,753 B; smallest omitted over-ceiling text = 52,448 B |
| `docs/ui/figcraft/**/build/` artifacts | 29 | generated `.py` / `.json` build scripts |
| `*.svg` (text assets) | 11 | contracted SVG diagrams |
| `.out` captures | 7 | `runtime-test/s5-s9.out`, `*.out` |
| `.gitignore` | 2 | root and `e2e/` |
| `package-lock.json` | 2 | `e2e/`, `web/` |
| `*.sha256` | 2 | `dist/` release sums |
| `.log` | 1 | `docs/evidence/live-1.1-paid.log` |
| **Unexplained** | 1 | `backend/crates/hiverouter-tools/src/bin/hiverouter.rs` (579 B) |
| **Total** | 188 |  |

### 5.1 Why the three core files are absent — dump omission, not a pre-1.2 state

Decidable from the dump itself, without trusting the current tree:

1. **The dump's own module graph requires them.** `backend/crates/hiverouter-core/src/lib.rs`
   (captured in full, 2,174 B) declares `pub mod execution;` (line 16) and `pub mod ws;`
   (line 31), and re-exports `pub use execution::{…}` / `pub use ws::SessionAuth;`.
   `backend/crates/hiverouter-tools/src/lib.rs` declares `pub mod approval;` (line 31).
2. **Captured files import them.** `core/src/http.rs:23` `use crate::execution::{ProviderExecutor, RunEngine, UnconfiguredExecutor};`,
   `core/src/http.rs:30` `use crate::ws::{refusal_response, SessionAuth, ws_handler};`,
   `core/src/provider_executor.rs:21` `use crate::execution::{…}`,
   `core/src/tools_bridge.rs:26` `use hiverouter_tools::approval::{…}` — 94 reference hits in the dump.
   A snapshot that captures `lib.rs` and `http.rs` yet omits `execution.rs`, `ws.rs` and
   `approval.rs` is internally inconsistent and could not compile. They existed at the capture commit.
3. **They predate the dump by two releases.** Both `v1.0.0` and `hivemind-1.1.0` trees contain
   all three files. The capture commit `e021b12` is `1.2.0-dev`, *after* both tags, so
   "pre-1.2 state" is refuted: this is an incomplete capture of a later tree, not an earlier one.
4. **The mechanism is size.** Blob sizes at `e021b12`: 96,383 / 53,201 / 58,434 B — all above
   the 49,753 B maximum of any captured section; they sit in the 34-file over-ceiling bucket.

**Verdict: dump omission (capture-rule artefact).** Any claim of the form "the 1.1/1.2 backend
has no execution engine / WS transport / approval module" is false. Note also that the task's
path `backend/core/src/…` exists in neither source; the real paths are
`backend/crates/hiverouter-core/src/…` and `backend/crates/hiverouter-tools/src/approval.rs`.

## 6. `DO-NOT-USE-AS-CURRENT`

| # | Item | Why | Use instead |
|---:|---|---|---|
| 1 | Any code claim resting on the **absence** of `execution.rs`, `ws.rs`, `approval.rs`, `store.ts`, `contract-v1.json`, `Cargo.lock` | All exist; §5.1 proves omission, not absence | Read the files — they are 53–100 KB, above the capture ceiling |
| 2 | All 20 `[Binary file]` sections, especially `plans/waves/W1-CORE-RECOVERY.md` | Content replaced by a placeholder; the W1 case is a false positive on valid text | The tracked file (120 lines, intact) |
| 3 | All 28 `[Empty file]` sections as evidence a file had content | The files are genuinely 0 bytes | The tracked file (also 0 bytes) |
| 4 | Dump `docs/CODEMAP/00-START-HERE.md` as the navigation index | Equals the pre-drift index (11 entries); the live tree lists 15 | The live working-tree file |
| 5 | Every count in `audit/*` (507 tracked, 233 backend tests, 79 live E2E, 63.98 MiB) | `v1.0.0`-era snapshot embedded in the dump; its own text marks it superseded | `e021b12` census (805 tracked) for the capture; **823** at the anchor |
| 6 | The dump's `runtime-test/**/*.db-shm` sections | Binary SQLite SHM captured only as a placeholder | Regenerate from a run, or ignore |
| 7 | `dist/*.zip`, `tools/bin/*.exe`, `docs/ui/figcraft/**` artifacts | Payload omitted by the capture rule | Rebuild from source; the dump cannot reconstruct artifacts |
| 8 | "636 files" as a count, and `audit/` = 70 in the prior provenance table | 635 sections; `audit/` = 69 (§1) | This document |
| 9 | Dump line numbers as if they described the live tree | Content is anchored at `e021b12`; the tree at the anchor is `aa16e13` | Cite `e021b12` with any dump line reference |

## 7. `STILL-VALID-HISTORICAL`

| Item | Validity | Required qualifier |
|---|---|---|
| The **587** sections byte-identical to the anchor tree | Safe as textual evidence; verified equal after normalization | Cite as "at `e021b12` / `aa16e13`" |
| The 28 empty-file sections | Correct — the files are empty | — |
| Dump `docs/CODEMAP/00-START-HERE.md` | Equals the anchor's index (11 entries) | Superseded by the live index (15 entries) |
| `audit/HIVEMIND-AUDIT-PAPER.md`, `audit/FINDINGS-REGISTER.md`, quality/roadmap CSVs | Valid **as the `v1.0.0` audit record** (30 findings: 3 P0 / 10 P1 / 16 P2 / 1 info) | Every finding needs a fresh source read before being called open or fixed — the paper says so itself |
| Dump census of `e021b12` (805 tracked / 36,480 working-tree / 35,675 ignored) | True for that commit | Never quote as current (823 at the anchor, 829 live) |
| Dump `.mmd` diagrams and contract/architecture docs | Source text, unchanged | Only where §3.3 does not list them as omitted |
| Dump copies of the 99 binary/UTF-16 **paths** | The path list is correct even though payloads are absent | Payload must come from the tree |

## 8. Summary table

| Metric | Value |
|---|---:|
| Input bytes (each file) | 5,007,714 |
| Input SHA-256 (both) | `F681ABD04638…EFFF761` |
| Byte differences | 0 |
| Dump lines / sections | 102,926 / 635 |
| Tracked files at the anchor `aa16e13` | 823 |
| Tracked files live at `5db2f21` | 829 |
| Files at capture commit `e021b12` | 805 (+18 untracked `docs/CODEMAP` on disk) |
| Capture completeness | 635 / 823 (77.2 %); 635 / 805 at `e021b12` (78.9 %) |
| Dump paths still present in the tree | 635 / 635 (0 missing) |
| Renames between dump and tree | 0 |
| Tracked files with no dump section | 188 (89 text, 99 binary/UTF-16) |
| Sections identical to the anchor after normalization | 587 |
| Sections equivalent but stored as `[Empty file]` | 28 |
| Sections with content unavailable (`[Binary file]`) | 20 (19 binary + 1 text false positive) |
| Sections materially changed vs the anchor | 0 |
| Live-tree documentation drift (post-anchor commits) | 3 modified + 6 added `docs/CODEMAP` files |
| Product-code files omitted from the dump | 7 (execution, ws, approval, store, contract schema, Cargo.lock, `src/bin`) |
| Core files **absent by omission** | `execution.rs`, `ws.rs`, `approval.rs` |

## 9. Corrections to previously written CODEMAP claims

| Prior statement | Correction | Proof |
|---|---|---|
| `CODEMAP/15-INPUT-PROVENANCE.md`: "636 `FILE:` entries", area table sums to 636, `audit/` = 70 | **635** sections; `audit/` = **69** | §1 — four independent counts agree on 635 |
| `CODEMAP/12-GITHUB-PUSH.md`: "Tracked files on remote `main` = 822" | **823** tracked at `aa16e13`; `e021b12..aa16e13` adds exactly 18 files to 805 | `git diff --name-status e021b12..aa16e13` (18 × `A`) and `git ls-tree -r` count |
| `CODEMAP/15`: commit "contested (`3988622` vs `8a5edab`)" | Capture commit is **`e021b12e5bb7e5334c71012bcf7a8810d91b9e58`** (from the dump's own `01-SCOPE-AND-CENSUS.md`, and a local object); `3988622` is the audit paper's older HEAD; `8a5edab` is not local | §2 |

Note: `docs/CODEMAP/13-HISTORICAL-AUDIT.md`, `14-GITDIAGRAM-ARCHITECTURE.md` and
`15-INPUT-PROVENANCE.md` were untracked when this analysis started; the post-anchor commit
`5db2f21` added them. They are outside the 823-file set for every count above and were
read only for cross-checking.

## 10. Method, reproducibility, limitations

```powershell
$a='C:\Users\Riley\Downloads\Gitingesthivemind.txt'
$b='C:\Users\Riley\Downloads\rodneydanger1-hivemind-8a5edab282632443.txt'
Get-FileHash $a,$b -Algorithm SHA256           # equal
# full-length byte compare: 5,007,714 bytes, 0 differing
git -C C:\Users\Riley\Desktop\hiverouter-v1 ls-tree -r --name-only aa16e13 | Measure-Object  # 823
git -C C:\Users\Riley\Desktop\hiverouter-v1 ls-tree -r --name-only e021b12  | Measure-Object  # 805
git -C C:\Users\Riley\Desktop\hiverouter-v1 diff --name-status e021b12..aa16e13  # 18 x A, all docs/CODEMAP
```

Python: split on `"="*48` / `FILE:` / `"="*48`; decode UTF-8; strip BOM; `CRLF`/`CR` →
`LF`; strip edge newlines; SHA-256 against the anchor blob (`git cat-file --batch` on
`aa16e13:<path>`) and against the live file; `difflib.SequenceMatcher` for line counts.
Gap classification uses the UTF-16 BOM, NUL-in-first-8000-bytes and strict UTF-8 decode
tests, then size and path/extension patterns.

Limitations, stated as such:

- The capture rule in §5 is **inferred** from 188 observations; no generator script or
  configuration was supplied. `src/bin/hiverouter.rs` remains the single unexplained omission.
- The `[Binary file]` false positive on `W1-CORE-RECOVERY.md` has no established cause; only
  its effect (content unavailable) is proven.
- Binary payloads are classified, never compared: no binary equality is claimed.
- The checkout was live during analysis (concurrent CODEMAP commits). The anchor is a real
  commit; live drift is reported separately in §3.4 and is not attributed to the dump.
