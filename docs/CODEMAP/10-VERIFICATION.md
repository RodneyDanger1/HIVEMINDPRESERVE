# 10 — Verification

## Frozen census commands

```powershell
git rev-parse HEAD
git -c core.quotePath=false ls-files -z    # 805
Get-ChildItem -Recurse -File -Force | Measure-Object   # includes .git; exclude it for 36,480
python docs/CODEMAP/_tools/build_codemap.py            # rebuilds _data
```

## Acceptance checks

| Check | Expected | Result |
|---|---|---|
| Tracked | 805 | 805 |
| Working tree excl. `.git` | 36,480 | 36,480 |
| Ignored | 35,675 | 35,675 |
| `.git` files | 226 | 226 |
| Total incl. `.git` | 36,706 | 36,706 |
| Partition | 805 + 35,675 = 36,480 | pass |
| Tracked bytes | 89,771,311 | pass |
| Ignored bytes | 19,971,803,372 | pass |
| Working-tree bytes | 20,061,574,683 | pass |
| Catalog rows | 805 | 805 |
| Inventory rows | 36,480 | 36,480 |
| Catalog hashes | all tracked rows have SHA-256 | pass |
| Encoding census | 655 UTF-8 / 96 NUL / 32 UTF-8-BOM / 22 UTF-16LE | pass |
| Text/binary reconciliation | 33 format-rule outliers named | pass |
| `.git` scope | separate, not claimed as product source | pass |

## Committed product gates (source: `docs/RELEASE-EVIDENCE.md`)

- `cargo check --workspace --tests --locked` exit 0.
- `cargo test --workspace --locked`: **475 passed / 0 failed / 2 ignored**.
- web `tsc --noEmit` exit 0; vitest **179/179**.
- guarded E2E **106 passed / 0 failed / 166 named skips**.
- docs truth **19/19**; QA harness **28/28**.
- No 1.2 release build/tag/seal/live proof.

## What this document does not verify

- Runtime behavior of the current binary (no live server run for this map).
- The content/semantics of every generated/vendor file (family-level only).
- Visual fidelity of design assets (needs human review).
- Hardlink groups beyond duplicate-hash detection.
