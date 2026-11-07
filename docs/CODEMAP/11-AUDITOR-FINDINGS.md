# 11 — Independent Auditor Findings and Corrections

Three read-only auditors were run before this document: evidence-check, evidence-gap-audit, research (research was stopped before final report; its partial findings are marked).

| Claim from draft plan | Auditor result | Correction applied |
|---|---|---|
| 805 tracked | VERIFIED | kept |
| 36,480 working tree | VERIFIED | kept |
| 36,706 incl. `.git` | VERIFIED | kept |
| 662 text / 177,861 lines | Numerically reproducible, but “text” was an undisclosed extension rule; lines were non-empty only | Catalog records encoding; no single line-count claim; 33 outliers named |
| 143 binary / 80.76 MB | Numerically reproducible, but 29 are plain-text `.out/.err` | Treated as evidence text, not structural binaries |
| Area table root 7 | CONTRADICTED: actual root 4 + `.cargo` 1; total 805 not 807 | corrected table |
| Backend 13/16/23/46 `.rs` | CONTRADICTED: raw 14/16/24/48; omitted 4 `.rs` | catalog includes all 102 Rust files |
| Ignored components 17,244/9,475/4,141 | VERIFIED total but components mixed tracked/total; 4,909 unnamed | full 35,675 family table with sum |
| Tier-3 “all files” | CONFLICT: 35,675 generated files cannot all get prose | depth labels: DEEP/INDEXED/STRUCTURAL/FAMILY; every path in inventory |
| No manifest/oracle | MISSING | `_data/inventory.csv` + summary + rebuild script |
| Encoding/CRLF/dedupe | MISSING | encoding field + core.autocrlf caveat + duplicate groups |
| `.git` internals | MISSING | separate scope, 226 files counted |
| Secrets/license/release | MISSING | risk section + no-first-party-license note |
| Research auditor | Stopped before final report | findings not claimed; gap remains documented |

## Auditor-verified numbers used

- 0 untracked, clean at `e021b12`, 49 commits, 3 tags.
- 5,571 directories, 942 empty.
- 149 paths with spaces; 0 reparse points/symlinks.
- 40 non-UTF-8 tracked files (22 UTF-16LE + 18 ZIP `.figcraft`), 31 UTF-8 BOM, 41 NUL-containing text; 551 LF / 87 CRLF / 24 mixed / 12 no-newline.
- 433 empty files (29 tracked); 8 duplicate hash groups / 22 redundant files / 23.07 MB.
- 18 ignored design deliverables = 22,009,234 B; 18 tracked build `.figcraft`; 6 tracked `.pyc`.
- 13 DB + 13 WAL + 13 SHM; 3 identical EXEs; 13 identical 4 KB DBs.
- 7,236 extensionless files (auditor total incl. different scope); this inventory records 7,051 extensionless in the frozen working tree.
- 99 dependency license files; no first-party LICENSE/NOTICE.
- 11 fixture-token hits, 0 real secrets; `.env*` absent.
- Prior audit FILE-MANIFEST 507 tracked at older HEAD vs current 805.
