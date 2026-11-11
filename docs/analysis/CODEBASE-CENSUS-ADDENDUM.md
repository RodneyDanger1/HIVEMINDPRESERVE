# Census addendum — analysis artifacts added after the 6a41250 pin

The main census remains pinned to `6a41250` and indexes **1,573** tracked paths. The
analysis commit `436fa4d` adds these 20 evidence/plan artifacts to the tracked tree;
none is a product runtime file. Current `git ls-files` at `436fa4d` is **1,593**.
This addendum closes the path-count gap without rewriting the pinned census.

| Path | Kind | Purpose |
|---|---|---|
| `docs/analysis/CODEBASE-CENSUS.json` | census | machine index of the 1,573 pinned paths |
| `docs/analysis/CODEBASE-CENSUS.md` | census | human census + appendix |
| `docs/analysis/CODEBASE-CENSUS.mmd` | census | subsystem Mermaid |
| `docs/analysis/GAP-AUDIT-1.md` | audit | backend/build gap audit |
| `docs/analysis/GAP-AUDIT-2.md` | audit | UI/docs/evidence gap audit |
| `docs/analysis/GAP-PATCH-1.md` | plan input | backend patch section |
| `docs/analysis/GAP-PATCH-2.md` | plan input | UI/docs patch section |
| `docs/analysis/LEAD-RESEARCH-PSEUDOCODE-LANGUAGES.md` | research | language/pseudocode research |
| `docs/analysis/PLAN-FACTCHECK-CORE.md` | audit | core plan fact-check |
| `docs/analysis/PLAN-FACTCHECK-UX.md` | audit | UI/install plan fact-check |
| `docs/analysis/WATCHDOG-LEDGER.jsonl` | ledger | 100-row claim audit |
| `docs/analysis/WATCHDOG-LEDGER.md` | ledger | watchdog report |
| `docs/analysis/lanes/L1-runtime-governance.md` | lane report | runtime/governance |
| `docs/analysis/lanes/L2-tools-security.md` | lane report | tools/security |
| `docs/analysis/lanes/L3-state-memory.md` | lane report | state/memory |
| `docs/analysis/lanes/L4-api-provider.md` | lane report | API/provider |
| `docs/analysis/lanes/L5-ui-ux.md` | lane report | UI/UX |
| `docs/analysis/lanes/L6-tests-release-install.md` | lane report | tests/release/install |
| `docs/analysis/lanes/L7-docs-claims.md` | lane report | docs/claims |
| `docs/plans/HIVEMIND-FINISH-PLAN-v3.md` | plan | 40-round execution plan |

The feature census is unchanged: it counts product surfaces derived from source, not
these new analysis files. The census's own arithmetic caveat still stands: its 216
verdict scope is not the 249-row table total; see `GAP-AUDIT-2.md` F1.
