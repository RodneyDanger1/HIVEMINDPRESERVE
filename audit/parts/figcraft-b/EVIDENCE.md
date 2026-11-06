# EVIDENCE — FigCraft-B seat (read-only audit commands and outputs)

Wave: HEAD `3988622` (short; `git rev-parse HEAD` = 3988622…), tag `v1.0.0` on HEAD.
Seat `figcraft-b` = 91 files, 0 deep files (wave sheet). All commands were read-only;
writes were confined to `audit/parts/figcraft-b/`.

## E1. Inventory + seat resolution
- `audit/inventory/ASSIGNMENTS.json` → key `figcraft-b` = 91 entries (26 plans, 59 audit, 3 claims, 2 contracts, 1 report).
- `audit/inventory/FILE-MANIFEST.csv` → 507 rows; every assigned path present (0 missing).

## E2. Hash verification (raw)
- Tool: `tools/extract.py` (SHA-256 over bytes, compared to manifest).
- Result: **91/91 hash MATCH, 91/91 size MATCH, 0 missing, 0 unreadable**.
- Total assigned bytes: 5,569,998 (dominated by `hello_rs.exe` 5,088,192 B; 4 binary files in seat).

## E3. Repository state (context for provenance findings)
- `git rev-parse --short HEAD` → `3988622`
- `git log --oneline -6` → `3988622 closeout: sync FINAL-REPORT …`; `2817326 closeout: final landing checks`; `6815dbe package: ship hiverouter-cli.exe`; `6fe05e8 docs: final report claim-register count synced`; `ff24707 docs: C-026 VERIFIED`; `7fb0495 docs: restore claim register 79->80 rows`
- `git tag -l "v1*"` → `v1.0.0`; `git tag --points-at HEAD` → `v1.0.0`
- `git status --porcelain` → only untracked `audit/parts/` (this audit wave).

## E4. Claim-register counts (recomputed, not trusted)
- CSV: 80 data rows; status histogram `{VERIFIED: 72, READ: 8}`; ids C-001..C-335 + C-340.
- JSON `counts`: total 80 / verified 72 / read 8 / unverified 0 / contradicted 0 → agrees with CSV.
- JSON body still lists C-026 under `decision_critical_unverified` (F-01).

## E5. Artifact hashes re-computed (read-only)
- `tools/bin/hiverouter.exe` → `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9`, 9,329,152 B (matches FINAL-REPORT/FINAL-CLOSEOUT).
- `tools/bin/hiverouter-cli.exe` → `51E1ABA6DC7349BFB8904014D7007015E13642EA9710F04D380525DCCBD9B9AD`.
- `dist/HiveMind-v1-win64.zip` → `BC7A2BAD98FAB1C849E3340D195C26F165581350B630F7BA37D89EE7CB27FEE7`.

## E6. Trace census (cross-document check)
- `docs/trace/controls.json`: 29 controls, statuses 23 PARTIALLY_VERIFIED + 6 PENDING_APP_NOT_BUILT;
  hop verdicts 43 WIRED-AND-WORKS + 270 PENDING + 6 MISSING = 319 (F-02).

## E7. Duplicate-content detection
- Identical-sha groups inside the seat: `live-probe-verifier.json` = `live-probe-verifier-v2.json` (19D660FB…);
  `census-verify.json` = `census.out` (1E0139F3…); 17 zero-byte `.err`/`.out` captures (expected clean stderr).

## E8. Cited-evidence tracking check
- `docs/audit/evidence/final-gate-tests.log` → untracked
- `e2e/test-results/run-live-FINAL-artifact.log` → untracked
- `docs/audit/evidence/final/cargo-test-final.log` → untracked
- `docs/audit/evidence/final/e2e-verifier-independent.log` → untracked
- (F-05; these are not rows of the 507-file manifest.)

## E9. Secret-shaped-string scan (all 87 text files in seat)
- Pattern: `sk-…`, `Bearer …`, `DEEPSEEK_API_KEY=…`, `password[:=]…`, `token[:=]` (≥20 chars).
- Result: "CLEAN (no credential-shaped strings)"
- Note: live-probe logs contain event names like `SETTINGS_STATUS` and refusal codes, no credential values.

## E10. Unreadable / UNVERIFIED files
- Unreadable: **none** (all non-binary files decoded UTF-8 with replacement; 0 read errors).
- UNVERIFIED content: the 4 binaries (`hello.exe` 49,876 B; `hello_rs.exe` 5,088,192 B;
  `test-failed-1.png` 4,254 B; `trace.zip` 11,204 B) — metadata/hash only; semantic content not analysed.
- `docs/audit/evidence/controls-verifier-skeleton.json` (39,735 B) carries 3 UNVERIFIED markers by design
  (it is a skeleton to be filled by a verifier), treated as an evidence scaffold, not a verification result.

## E11. Tools used (paths under audit/parts/figcraft-b/)
- `tools/extract.py` — read-only per-file digest (`_digest.json`): manifest compare, headings, marker counts, dates.
- `tools/gen_outputs.py` — writes FILES.md, DEPENDENCIES.md, QUALITY.csv from the digest.
- This file's sibling `tools/write_findings.py` — writes FINDINGS.md/EVIDENCE.md + quality binary rows + secret scan.
