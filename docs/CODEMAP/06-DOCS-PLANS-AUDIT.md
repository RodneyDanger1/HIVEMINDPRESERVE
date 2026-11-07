# 06 — Documentation, Plans, and Audit Corpus

This repository carries its own history. Treat these as **claims**, not authority; source code wins on conflict.

| Area | Tracked files | What it is |
|---|---:|---|
| `docs/` | 260 | Product/architecture reports, contracts, trace registry, evidence, UI/Figma-craft assets, diagrams |
| `plans/` | 114 | Foresight runs, seat reports, wave plans, verification notes, compiled ledgers |
| `audit/` | 93 | Prior audit inventories, parts, diagrams, tools; includes stale file manifests |

Important subareas:
- `docs/contracts/` — UI command/event contract docs; counts must match 17/23.
- `docs/trace/` — 319-hop control registry (1 WIRED / 49 UNVERIFIED / 5 MISSING / 264 PENDING) plus live captures; raw capture is empty.
- `docs/evidence/` — local command logs; many ignored, so a fresh clone lacks them.
- `docs/ui/` — FigCraft design sources/assets; 18 ignored canonical deliverables + tracked build `.figcraft` ZIPs.
- `docs/plans/` and `plans/` — design history; some docs describe earlier 1.1/1.2 states and are superseded by later commits.
- `audit/inventory/FILE-MANIFEST*.json` — prior census at a different HEAD (reported 507 tracked vs 805 now); do not use as current oracle.

## Truth rules for this corpus

1. Sealed 1.1.0 artifacts are authoritative only for 1.1.0.
2. Current source is 1.2.0-dev; no 1.2 tag/artifact/seal exists.
3. A plan/report is not proof; it is a claim with a date/commit.
4. Old counts (507 files) are historical, not current.
5. The current oracle is `_data/inventory.csv` frozen at `e021b12`.
