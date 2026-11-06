# EVIDENCE — watchdog-2

Read-only audit. No product/test/git writes; analysis script kept outside the repo in TEMP.

## Commands run (read-only)

- **Read wave inputs:** Get-ChildItem audit; read audit/WAVE-SHEET.md; read audit/inventory/DEEP-LIST.md; node -e parse audit/inventory/ASSIGNMENTS.json
- **Partition extraction:** node script: assignments["watchdog-2"] → 122 paths (expected 122)
- **Hash verification:** sha256 recomputed per file via node:crypto; compared to FILE-MANIFEST.csv
- **Tracked check:** git ls-files (read-only) → tracked set
- **Content scan:** node regex scan: LOC, imports, TODO/FIXME, console.log, any, test.skip/fixme, mock, process.env, 127.0.0.1, fs writes, secret-ish literals, empty catch, generated markers
- **Deep reads:** read fully: e2e/playwright.config.ts (47 lines), e2e/specs/live-app.spec.ts (334), e2e/specs/trace-hops.spec.ts (318), tools/qa/verify-harness.mjs (131)
- **No writes outside seat:** all generated files written under audit/parts/watchdog-2/; temp analysis script in %TEMP% (outside repo)

## Measured totals

- assigned files: 122 (4 deep, 118 standard)
- files on disk: 122/122; manifest-hash matches: 122/122; mismatches: []
- git-tracked: 122/122; untracked: []
- total tracked bytes in partition: 55196233 (~52.6 MiB); largest: runtime-test/pkg-cli-final/hiverouter.exe=9329152B, runtime-test/pkg-final/hiverouter.exe=9329152B, tools/bin/hiverouter.exe=9329152B, runtime-test/pkg-smoke/hiverouter.exe=9099776B
- unreadable as text (binary/DB/exe): ["runtime-test/data/hiverouter.db","runtime-test/data/hiverouter.db-shm","runtime-test/data/hiverouter.db-wal","runtime-test/data2/hiverouter.db","runtime-test/data2/hiverouter.db-shm","runtime-test/data2/hiverouter.db-wal","runtime-test/data3/hiverouter.db","runtime-test/data3/hiverouter.db-shm","runtime-test/data3/hiverouter.db-wal","runtime-test/data4/hiverouter.db","runtime-test/data4/hiverouter.db-shm","runtime-test/data4/hiverouter.db-wal","runtime-test/data5/hiverouter.db","runtime-test/data5/hiverouter.db-shm","runtime-test/data5/hiverouter.db-wal","runtime-test/data6/hiverouter.db","runtime-test/data6/hiverouter.db-shm","runtime-test/data6/hiverouter.db-wal","runtime-test/data7/hiverouter.db","runtime-test/data7/hiverouter.db-shm","runtime-test/data7/hiverouter.db-wal","runtime-test/data8/hiverouter.db","runtime-test/data8/hiverouter.db-shm","runtime-test/data8/hiverouter.db-wal","runtime-test/data9/hiverouter.db","runtime-test/data9/hiverouter.db-shm","runtime-test/data9/hiverouter.db-wal","runtime-test/pkg-cli-final/data/hiverouter.db","runtime-test/pkg-cli-final/data/hiverouter.db-shm","runtime-test/pkg-cli-final/data/hiverouter.db-wal","runtime-test/pkg-cli-final/hiverouter-cli.exe","runtime-test/pkg-cli-final/hiverouter.exe","runtime-test/pkg-final/data/hiverouter.db","runtime-test/pkg-final/data/hiverouter.db-shm","runtime-test/pkg-final/data/hiverouter.db-wal","runtime-test/pkg-final/err.txt","runtime-test/pkg-final/hiverouter.exe","runtime-test/pkg-final/smoke-data/hiverouter.db","runtime-test/pkg-final/smoke-data/hiverouter.db-shm","runtime-test/pkg-final/smoke-data/hiverouter.db-wal","runtime-test/pkg-final/smoke.err","runtime-test/pkg-smoke/data/hiverouter.db","runtime-test/pkg-smoke/data/hiverouter.db-shm","runtime-test/pkg-smoke/data/hiverouter.db-wal","runtime-test/pkg-smoke/err.txt","runtime-test/pkg-smoke/hiverouter.exe","runtime-test/s5.err","runtime-test/s6.err","runtime-test/s7.err","runtime-test/s8.err","runtime-test/s9.err","runtime-test/server.err.txt","runtime-test/server2.err.txt","runtime-test/server3.err.txt","runtime-test/server4.err.txt","tools/bin/hiverouter-cli.exe","tools/bin/hiverouter.exe","tools/cc-test/r.exe","tools/cc-test/t.exe"]
- duplicate text hashes: []
- external import specifiers: ["@playwright/test","node:crypto","node:fs","node:path","node:url"]

## Deep-file citations used

- `e2e/playwright.config.ts:17-46` (baseURL, workers/retries/reporters/projects)
- `e2e/specs/live-app.spec.ts:14-334` (15 tests; evidence writes; soft assertions at 240-242; fixture paths 282-283/307-312; bundle fetch 250-254/313-317)
- `e2e/specs/trace-hops.spec.ts:75-318` (afterAll writer 76-87; 28-surface loop 89-317; S-APPROVAL absent from SURFACES 28-57)
- `tools/qa/verify-harness.mjs:40-131` (contract parse, matrix checks, seeds, warning-only FigCraft; exit codes)

## Not verified / limits

- Binary `tools/bin/hiverouter.exe` and runtime DB bytes were hashed but not decompiled/inspected at the byte/behaviour level; runtime semantics inferred from prior live runs and are marked UNVERIFIED here.
- `tools/w64devkit/**` (9,452 files/717 MB) and node_modules/target/dist are ignored/generated classes per wave sheet §4 — censused, not analysed; not part of the 122 tracked assignment.
- No live server/suite was executed for this audit; test-result claims in FILES/QUALITY come from previously captured final-run evidence (`e2e/test-results/run-live-FINAL-artifact.log`, 79 passed/0 failed/14 skipped) and are labelled as such.

## Runtime-artifact census (measured)

- Category counts from manifest: `runtime-artifact` 82, `generated-artifact` 4, `source` 28, `config-manifest` 4, `other` 4 (total 122).
- Partition total tracked bytes: 33,867,525 (~32.3 MiB); largest: three identical 9,329,152 B `hiverouter.exe` copies (tools/bin + runtime-test/pkg-final + runtime-test/pkg-cli-final), then 5,364,224 B `hiverouter-cli.exe` copies, 9,099,776 B stale `pkg-smoke/hiverouter.exe`.
- Binary sha256 groups (measured): `184D51A7…C9A9` ×3 (shipped server), `71B229B5…C38FC` ×1 (stale), `51E1ABA6…B9AD` ×2 (CLI), plus `tools/cc-test/r.exe`/`t.exe` unique.
- Runtime DB/WAL/SHM: 11 (`.db`, `.db-shm`, `.db-wal`) at `runtime-test/data*` + pkg dirs + smoke-data; WAL sizes 94,792–185,432 B; snapshots 271 B each.
- Logs/outputs: `s5..s9`, `server{,2..4}`, `pkg-*/*.out|err|smoke.*`, `server.out.txt` etc; 12 zero-byte `.err` files (empty capture scaffolding).
- Files unreadable as text (binary): 8 PE exes, 13 `.db`, 13 `.db-shm`, 13 `.db-wal` (39 binary files) — hashed/metadata only; content behaviour UNVERIFIED.
- Quality totals (from QUALITY.csv, n=122, measured): correctness sum=456 avg=3.74 (min 2, max 5, 9×five); maintainability 393/3.22 (max 4); modularity 393/3.22; error_handling 393/3.22; security 423/3.47; docs 306/2.51; evidence 427/3.50 (7×five). Runtime artifacts and ad-hoc probes pull the averages down; deep specs/tools score 4–5.
- No product/test/git writes were made; outputs written only under `audit/parts/watchdog-2/`. Analysis generator lived in `%TEMP%` outside the repo.
