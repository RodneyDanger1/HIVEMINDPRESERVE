# HiveMind 1.0 — 8-seat analysis program (post-integration baseline)

Baseline: HEAD includes the provider wiring, live server proof (`docs/evidence/live-provider-smoke.md`), replay backend arm, and 16-state web vocabulary. Product remains under active implementation; seats are **read-only** and write only under `plans/seats/SEAT-XX/`.

## Identical mandatory protocol (all seats)

1. Read `audit/HIVEMIND-AUDIT-PAPER.md` **in full, first**.
2. Read the 4 official DeepSeek docs pages in full (quick start, pricing, thinking mode, chat completions).
3. Read every assigned audit file **one at a time**, then write a READLOG entry (path, bytes, sha256, lines, timestamp, verified facts).
4. Read every product file in the assigned slice **one at a time**, then immediately write that file’s analysis section.
5. No skipping, no filename inference, no writes to product/git/config; no key access; no paid calls.
6. Every claim labelled VERIFIED / READ / INFERRED / UNVERIFIED with path:line, command, or URL.
7. Every step: 3 cycles of `/forgewright-recon` → `/forgewright-plan` → `/forgewright-lead` in that exact order; record in `GATE-CY-{1,2,3}.md`; a failed cycle stops the seat and reports blocked.
8. Output: `REPORT.md` (partition), `READLOG.md`, `EVIDENCE.md`, gate files. Timebox 30 minutes; interim report after 10 minutes.

## Skills each seat must know/use

`/forgewright-recon`, `/forgewright-plan`, `/forgewright-lead`, `/integration-logic`, `/research`, `/auditor`, `/logic-debugger`, `/trace`, `/weaver`, `/visuals`, `/agent-experience`, `/capability-auditor`, plus truthseeker discipline. Use domain-appropriate skill; cite evidence; no fabricated success.

## Assignments (all 19 non-paper audit files assigned once; paper read by all)

| Seat / teammate | Audit files (after paper) | Product slice | Output |
|---|---|---|---|
| S1 b-core | FINDINGS-REGISTER; diagrams/03-module-dependencies.mmd | core Cargo/config/http/execution/main/checks/ws/provider_executor; provider client/dto/secret/budget/pricing/error; core tests | plans/seats/SEAT-01/REPORT.md |
| S2 b-provider | WAVE-SHEET; diagrams/01-system-topology.mmd; 02-data-flow.mmd | contract command/event/state/refusal/schema; core ws/http; tools settings/health; ws tests | plans/seats/SEAT-02/REPORT.md |
| S3 watchdog-1 | diagrams/04-state-inventory.mmd | Canvas, store, agentState, strings, SettingsPanel/Chat, api contract/index/client, types, UI tests | plans/seats/SEAT-03/REPORT.md |
| S4 watchdog-2 | diagrams/05-dependency-lineage.mmd; tools/build_roadmap.py | e2e specs/config/helpers; App/panels/hooks; tools/qa | plans/seats/SEAT-04/REPORT.md |
| S5 figcraft-a | diagrams/06-migration-target.mmd; tools/build_inventory.py; tools/enrich_manifest.py | root/crate Cargo files, Cargo.lock, .gitignore, runbook/build-release/release-manifest, dist, package.json | plans/seats/SEAT-05/REPORT.md |
| S6 figcraft-b | tools/census_ignored_deliverables.py; tools/complete_figcraft_a.py; tools/dedupe_figcraft_a.py | provider secret/client, tools settings/health, docs/claims, PHASE-B verification, .gitignore, docs packet | plans/seats/SEAT-06/REPORT.md |
| S7 ui-watchdog | tools/finalize_audit.py; tools/merge_quality.py; tools/quality_code_stats.py | all test trees, QUALITY-MATRIX, VERIFICATION, CI/release scripts | plans/seats/SEAT-07/REPORT.md |
| S8 strip-watchdog | tools/build_appendix.py; tools/synthesize_paper.py | docs release/claims/runbooks, ROADMAP/tasks, new RELEASE.md, README | plans/seats/SEAT-08/REPORT.md |

Paper: all 8 read `audit/HIVEMIND-AUDIT-PAPER.md` first. No seat edits product files; Lead owns all writes.
