# DEPENDENCIES — FigCraft-B partition (documentation/evidence tier)

Scope: 91 assigned files. This tier contains no executable product code; edges are **document citations and evidence references**, verified by text scan of each assigned file (scripts in `tools/extract.py`).

## 1. Internal citation graph (assigned → assigned, top edges)

| From | Cites (assigned files) |
|---|---|
| `audit/BASELINE-VERIFICATION.md` | `audit/evidence/controls-verifier-skeleton.json`, `audit/evidence/toolchain/hello.c`, `audit/evidence/toolchain/hello.exe`, `audit/evidence/toolchain/hello.rs`, `audit/evidence/toolchain/hello_rs.exe`, `claims/CLAIM-REGISTER.csv`, `contracts/UI-COMMANDS-v1.md`, `plans/HASH-VERIFICATION.md` | … |
| `audit/FINAL-CLOSEOUT.md` | `audit/FINAL-DELTA-VERIFICATION.md`, `audit/evidence/final/pw-out/.last-run.json` | |
| `audit/FINAL-DELTA-VERIFICATION.md` | `audit/evidence/final/cli-diagnose-final.out`, `audit/evidence/final/cli-health-final.out`, `audit/evidence/final/live-probe-final.json`, `audit/evidence/final/pw-out/.last-run.json`, `claims/CLAIM-REGISTER.csv`, `claims/CLAIM-REGISTER.json` | |
| `audit/PHASE-B-VERIFICATION.md` | `audit/evidence/final/pw-out/.last-run.json`, `audit/evidence/phase-b/cli/preflight.err`, `audit/evidence/phase-b/live-probe-verifier-v2.json`, `audit/evidence/phase-b/live-probe-verifier-v2.out.txt`, `audit/evidence/phase-b/live-probe-verifier.json`, `claims/CLAIM-REGISTER.csv`, `claims/CLAIM-REGISTER.json` | |
| `audit/QA-ENV.md` | `claims/SKILL-USE.md`, `contracts/UI-COMMANDS-v1.md` | |
| `audit/evidence/controls-verifier-skeleton.json` | `contracts/UI-COMMANDS-v1.md` | |
| `audit/evidence/final/live-probe-final.mjs` | `audit/evidence/final/live-probe-final.json` | |
| `audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/error-context.md` | `contracts/UI-COMMANDS-v1.md` | |
| `audit/evidence/gen_controls.py` | `audit/evidence/controls-verifier-skeleton.json`, `contracts/UI-COMMANDS-v1.md` | |
| `audit/evidence/phase-b/live-probe-verifier.mjs` | `audit/evidence/phase-b/live-probe-verifier.json` | |
| `audit/evidence/toolchain/hello.exe` | `audit/evidence/toolchain/hello.c` | |
| `claims/CLAIM-REGISTER.csv` | `plans/D2-D3-WIRING-EVIDENCE.md`, `plans/FRONTEND-STATUS.md`, `plans/HASH-VERIFICATION.md`, `plans/PHASE-1-EVIDENCE.md`, `plans/PHASE-11-EVIDENCE.md`, `plans/PHASE-12-EVIDENCE.md`, `plans/PHASE-2-EVIDENCE.md`, `plans/PHASE-3-EVIDENCE.md` | … |
| `claims/CLAIM-REGISTER.json` | `audit/BASELINE-VERIFICATION.md`, `plans/D2-D3-WIRING-EVIDENCE.md`, `plans/PHASE-1-EVIDENCE.md`, `plans/PHASE-3-EVIDENCE.md`, `plans/PHASE-4-EVIDENCE.md`, `plans/PHASE-7-8-10-EVIDENCE.md`, `plans/PHASE-9-EVIDENCE.md`, `plans/TOOLCHAIN.md` | |
| `claims/SKILL-USE.md` | `claims/CLAIM-REGISTER.csv`, `contracts/ADAPTERS.md`, `plans/PHASE-0-RECON.md` | |
| `contracts/ADAPTERS.md` | `plans/PHASE-11-EVIDENCE.md` | |
| `plans/D2-D3-WIRING-EVIDENCE.md` | `contracts/UI-COMMANDS-v1.md` | |
| `plans/FRONTEND-STATUS.md` | `plans/PHASE-7-8-10-EVIDENCE.md`, `plans/PHASE-7-8-10-RECON.md` | |
| `plans/PHASE-1-EVIDENCE.md` | `plans/PHASE-1-RECON.md` | |
| `plans/PHASE-1-RECON.md` | `contracts/UI-COMMANDS-v1.md`, `plans/PHASE-1-EVIDENCE.md` | |
| `plans/PHASE-11-RECON.md` | `plans/PHASE-11-EVIDENCE.md` | |
| `plans/PHASE-12-EVIDENCE.md` | `contracts/UI-COMMANDS-v1.md` | |
| `plans/PHASE-2-EVIDENCE.md` | `contracts/UI-COMMANDS-v1.md`, `plans/PHASE-1-EVIDENCE.md`, `plans/PHASE-2-RECON.md` | |
| `plans/PHASE-2-RECON.md` | `contracts/UI-COMMANDS-v1.md` | |
| `plans/PHASE-3-EVIDENCE.md` | `plans/PHASE-3-RECON.md` | |
| `plans/PHASE-4-EVIDENCE.md` | `plans/PHASE-4-RECON.md` | |
| `plans/PHASE-5-EVIDENCE.md` | `plans/PHASE-5-RECON.md` | |
| `plans/PHASE-5-RECON.md` | `contracts/ADAPTERS.md` | |
| `plans/PHASE-6-EVIDENCE.md` | `contracts/ADAPTERS.md` | |
| `plans/PHASE-6-RECON.md` | `contracts/ADAPTERS.md` | |
| `plans/PHASE-7-8-10-EVIDENCE.md` | `contracts/UI-COMMANDS-v1.md` | |
| `plans/PHASE-7-8-10-RECON.md` | `contracts/UI-COMMANDS-v1.md` | |
| `plans/PHASE-9-EVIDENCE.md` | `plans/PHASE-9-RECON.md` | |
| `plans/SESSION-PLAN.md` | `contracts/UI-COMMANDS-v1.md` | |
| `plans/TOOLCHAIN.md` | `audit/evidence/toolchain/hello.rs` | |
| `reports/FINAL-REPORT.md` | `audit/PHASE-B-VERIFICATION.md`, `claims/CLAIM-REGISTER.csv`, `plans/D2-D3-WIRING-EVIDENCE.md` | |

Total intra-partition citation edges: **86** across **35** files.

## 2. Cross-partition dependency edges (documentation tier → code/other seats)

| From | References |
|---|---|
| `audit/BASELINE-VERIFICATION.md` | backend (b-core / b-provider seats), blueprint corpus, docs/trace (ui-watchdog seat), e2e (watchdog-2 seat), external blueprint corpus (read-only), web (watchdog-1 seat) |
| `audit/FINAL-CLOSEOUT.md` | blueprint corpus, dist artifact, tools (watchdog-2 seat) |
| `audit/FINAL-DELTA-VERIFICATION.md` | backend (b-core / b-provider seats), docs/trace (ui-watchdog seat), e2e (watchdog-2 seat), runtime-test (watchdog-2 seat), web (watchdog-1 seat) |
| `audit/PHASE-B-VERIFICATION.md` | backend (b-core / b-provider seats), dist artifact, docs/trace (ui-watchdog seat), docs/ui/figcraft (figcraft-a seat), e2e (watchdog-2 seat), ledgers (figcraft-a seat), runtime-test (watchdog-2 seat), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `audit/QA-ENV.md` | backend (b-core / b-provider seats), blueprint corpus, docs/trace (ui-watchdog seat), e2e (watchdog-2 seat), external blueprint corpus (read-only), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `audit/evidence/controls-verifier-skeleton.json` | docs/trace (ui-watchdog seat), docs/ui/figcraft (figcraft-a seat), external blueprint corpus (read-only), web (watchdog-1 seat) |
| `audit/evidence/final/live-probe-final.mjs` | e2e (watchdog-2 seat) |
| `audit/evidence/gen_controls.py` | docs/trace (ui-watchdog seat), docs/ui/figcraft (figcraft-a seat), external blueprint corpus (read-only), web (watchdog-1 seat) |
| `audit/evidence/phase-b/live-probe-verifier.mjs` | e2e (watchdog-2 seat) |
| `claims/CLAIM-REGISTER.csv` | backend (b-core / b-provider seats), blueprint corpus, dist artifact, docs/ui/figcraft (figcraft-a seat), e2e (watchdog-2 seat), ledgers (figcraft-a seat), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `claims/CLAIM-REGISTER.json` | backend (b-core / b-provider seats), docs/ui/figcraft (figcraft-a seat), ledgers (figcraft-a seat), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `claims/SKILL-USE.md` | docs/trace (ui-watchdog seat), docs/ui/figcraft (figcraft-a seat) |
| `contracts/ADAPTERS.md` | backend (b-core / b-provider seats), blueprint corpus, tools (watchdog-2 seat) |
| `contracts/UI-COMMANDS-v1.md` | blueprint corpus |
| `plans/D2-D3-WIRING-EVIDENCE.md` | backend (b-core / b-provider seats), web (watchdog-1 seat) |
| `plans/FRONTEND-STATUS.md` | backend (b-core / b-provider seats), blueprint corpus, dist artifact, docs/ui/figcraft (figcraft-a seat), ledgers (figcraft-a seat), web (watchdog-1 seat) |
| `plans/HASH-VERIFICATION.md` | blueprint corpus, tools (watchdog-2 seat) |
| `plans/PHASE-0-RECON.md` | backend (b-core / b-provider seats), external blueprint corpus (read-only), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `plans/PHASE-1-EVIDENCE.md` | backend (b-core / b-provider seats), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `plans/PHASE-1-RECON.md` | backend (b-core / b-provider seats), blueprint corpus, external blueprint corpus (read-only), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `plans/PHASE-11-EVIDENCE.md` | backend (b-core / b-provider seats), tools (watchdog-2 seat) |
| `plans/PHASE-11-RECON.md` | backend (b-core / b-provider seats), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `plans/PHASE-12-EVIDENCE.md` | backend (b-core / b-provider seats), dist artifact, e2e (watchdog-2 seat), tools (watchdog-2 seat) |
| `plans/PHASE-12-RECON.md` | backend (b-core / b-provider seats), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `plans/PHASE-2-EVIDENCE.md` | web (watchdog-1 seat) |
| `plans/PHASE-2-RECON.md` | backend (b-core / b-provider seats), web (watchdog-1 seat) |
| `plans/PHASE-3-EVIDENCE.md` | backend (b-core / b-provider seats) |
| `plans/PHASE-3-RECON.md` | backend (b-core / b-provider seats), web (watchdog-1 seat) |
| `plans/PHASE-4-EVIDENCE.md` | backend (b-core / b-provider seats) |
| `plans/PHASE-4-RECON.md` | backend (b-core / b-provider seats), web (watchdog-1 seat) |
| `plans/PHASE-5-EVIDENCE.md` | backend (b-core / b-provider seats), tools (watchdog-2 seat) |
| `plans/PHASE-5-RECON.md` | backend (b-core / b-provider seats), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `plans/PHASE-6-EVIDENCE.md` | backend (b-core / b-provider seats), tools (watchdog-2 seat) |
| `plans/PHASE-6-RECON.md` | backend (b-core / b-provider seats), tools (watchdog-2 seat), web (watchdog-1 seat) |
| `plans/PHASE-7-8-10-EVIDENCE.md` | dist artifact, docs/ui/figcraft (figcraft-a seat), ledgers (figcraft-a seat), web (watchdog-1 seat) |
| `plans/PHASE-7-8-10-RECON.md` | backend (b-core / b-provider seats), blueprint corpus, web (watchdog-1 seat) |
| `plans/PHASE-9-EVIDENCE.md` | backend (b-core / b-provider seats) |
| `plans/PHASE-9-RECON.md` | web (watchdog-1 seat) |
| `plans/SESSION-PLAN.md` | blueprint corpus |
| `reports/FINAL-REPORT.md` | backend (b-core / b-provider seats), blueprint corpus, docs/trace (ui-watchdog seat), docs/ui/figcraft (figcraft-a seat), e2e (watchdog-2 seat), external blueprint corpus (read-only), ledgers (figcraft-a seat), runtime-test (watchdog-2 seat), tools (watchdog-2 seat), web (watchdog-1 seat) |

## 3. Dependency shape

- `docs/contracts/UI-COMMANDS-v1.md` is the frozen UI vocabulary consumed by `docs/plans/PHASE-*` evidence and the UI assets tracked by the figcraft-a seat.
- `docs/plans/**` (26 files) are the phase recon/evidence ledgers; they cite backend crates, `e2e/`, `web/` and the blueprint corpus.
- `docs/claims/**` cite plan evidence and artifact paths; `docs/reports/FINAL-REPORT.md` cites the audit verification files and the claim register.
- `docs/audit/**` (59 files) forms the independent verification tier: reports cite `docs/audit/evidence/**` raw logs and `docs/trace/**` (other seat).
- External: `C:\Users\Riley\Desktop\hivemind-blueprint` corpus (read-only, not in repo manifest).
