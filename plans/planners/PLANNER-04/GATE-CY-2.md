# GATE-CY-2 — recon → plan → lead (exact order)

**Planner P4** · object: evidence/acceptance reconciliation: logs, JUnit, provider paths, live smoke · 2026-11-06T11:32:00-06:00

## 1. RECON

VERIFIED from seats: Rust gate log `docs/audit/evidence/final-gate-tests.log` (41,644 B) and e2e log `e2e/test-results/run-live-FINAL-artifact.log` (14,217 B) contain counts but no HEAD/commit id, command line, exit code, or JUnit XML (S7-F01/F04). `audit/tools/finalize_audit.py` rebuilds `audit/VERIFICATION.md` while patching paper/register (S7-F02). `e2e/helpers/matrix.ts:43-52` skips all browser tests unless `E2E_APP_READY=1` → skip-as-green (S4-1). `playwright.config.ts:17` default :8080 vs live :8799 (S4-4). Seed/apply ordering can silently drop trace verdicts (S4-6). Provider error paths: 401×7, 402×5, 429×6, timeout×7, budget×24, malformed×3 in `refusal_matrix.rs`; missing-key and usage-absent are unit-only; no run-path e2e maps provider refusals to run row/ledger/UI (S7 §3, S1-S1-R1/R2). Live smoke: `docs/evidence/live-provider-smoke.md` + `key-validation.md` are self-attested prose with no commit/binary hash; no gated script (S1-R8, S6-F02, S4-3). Replay/state proof: source fixed (S3 V1/V2) but no replay e2e and `failed` not terminal (S3 D1/D2/D3, S4 journey table).

## 2. PLAN

Design the acceptance system as a pipeline of falsifiable gates: G0 quiescence+HEAD capture → G1 Rust gate with JUnit XML + exit code → G2 web gate → G3 e2e gate with JUnit, artifact-identity preflight, 0 skips (or skips declared non-gates) → G4 stub-provider run-path e2e (no key) → G5 gated live smoke (explicit flag+key+cap; never default) → G6 release seal (manifest, rebuilt artifacts, tag at built commit). Each gate: command, expected artifact, machine-checkable acceptance, owner seat. Reject: accepting prose logs, accepting skip-as-green, one-off manual smoke as release proof, patching paper counts without supersession note.

## 3. LEAD

PASS-WITH-NOTES. Evidence system is actionable; two constraints accepted: live smoke must stay operator-gated (no CI paid calls), and JUnit output may need adding a test reporter dependency (tooling change, not product).
