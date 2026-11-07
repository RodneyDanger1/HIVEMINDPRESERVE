# HiveMind Codebase Encyclopedia

Frozen at commit [`e021b12e5bb7`](https://github.com/RodneyDanger1/HiveMind) — clean worktree, source version `1.2.0-dev`.

This is the map of **every path in the working tree**, not a marketing overview.

## The partition (exact)

| Universe | Count | Bytes |
|---|---:|---:|
| Git-tracked project files | **805** | **85.61 MiB** |
| Ignored/generated/vendor files | **35,675** | **18.60 GiB** |
| Working tree (excl. `.git`) | **36,480** | **18.68 GiB** |
| `.git` internals | **226** | volatile, VCS state |
| **Total including `.git`** | **36,706** | — |

`805 + 35,675 = 36,480`. That is the exact partition; no path is uncounted.

## Read in this order

1. [`01-SCOPE-AND-CENSUS.md`](01-SCOPE-AND-CENSUS.md) — exact counts, per-directory and extension tables, text/binary rule, CRLF/blob nuance.
2. [`02-ARCHITECTURE.md`](02-ARCHITECTURE.md) — process topology and the full request/run/tool/approval flow.
3. [`03-BACKEND.md`](03-BACKEND.md) — contract, provider, tools, core.
4. [`04-WEB.md`](04-WEB.md) — React UI, store, API, panels, replay/approval UX.
5. [`05-E2E-TOOLS.md`](05-E2E-TOOLS.md) — Playwright, QA gates, release/seal tooling.
6. [`06-DOCS-PLANS-AUDIT.md`](06-DOCS-PLANS-AUDIT.md) — docs/plans/audit/claims corpus and supersession.
7. [`07-RUNTIME-BINARIES.md`](07-RUNTIME-BINARIES.md) — runtime-test, dist, DB/WAL, binaries, assets.
8. [`08-FILE-CATALOG.md`](08-FILE-CATALOG.md) — every tracked file with hash/encoding/role; deep entries flagged.
9. [`09-RISKS-GAPS.md`](09-RISKS-GAPS.md) — verified gaps and unverified claims.
10. [`10-VERIFICATION.md`](10-VERIFICATION.md) — commands, acceptance gates, coverage proof.
11. [`11-AUDITOR-FINDINGS.md`](11-AUDITOR-FINDINGS.md) — the three independent audits and the corrections they forced.

Machine data:
- [`_data/inventory.csv`](_data/inventory.csv) — **36,480 rows**, every path (tracked and ignored), size, area, extension, tracked flag, and SHA-256 for tracked files.
- [`_data/summary.json`](_data/summary.json) — recomputable tables.
- [`_data/inventory.json`](_data/inventory.json) — same data as JSON.

## What is proven and what is not

**Proven:** the path partition, per-directory counts, per-extension counts, tracked SHA-256 values, encoding census, line-ending caveat, largest files, and Git state.

**Not claimed:** that every generated/vendor file was semantically read line by line. Tracked authored files are indexed and the architecture-critical ones are deeply read; generated files are fully enumerated and family-explained. See [`01-SCOPE-AND-CENSUS.md`](01-SCOPE-AND-CENSUS.md) for the exact depth labels.
