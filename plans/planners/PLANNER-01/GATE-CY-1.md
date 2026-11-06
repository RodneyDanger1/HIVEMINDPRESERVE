# GATE-CY-1 — recon → plan → lead (PLANNER P1, cycle 1: 8-seat evidence corpus)

Read-only planner seat. Scope: all 8 seat reports (`plans/seats/SEAT-01..08/{REPORT,READLOG,EVIDENCE,GATE-CY-1..3}.md`,
48 files read fully), then paper + 4 official docs, then plan synthesis. Skills loaded: forgewright-recon/plan/lead,
integration-logic, research, auditor, logic-debugger, trace, weaver, agent-experience, capability-auditor, truthseeker.

## 1. RECON — what the 8 seats actually establish (facts, then contradictions)

Seat-consensus facts (evidence class: VERIFIED at HEAD `5c69b66`/source unless noted):
- Provider IS wired at HEAD: core→provider dep (`core/Cargo.toml:21`), `build_executor` (`core/src/http.rs:67-82`),
  adapter (`core/src/provider_executor.rs`); honest `UnconfiguredExecutor` → `E_KEY_MISSING` fallback (SEAT-01, SEAT-02,
  SEAT-05, SEAT-08 all independently report this). **Paper A-F01 is stale** (paper audits `3988622`).
- Replay arm + 16-state parity ARE in HEAD source (SEAT-03 V1/V2; SEAT-08 §5; SEAT-04 V3/V4). Paper A-F02/A-F03 stale.
- Remaining provider-runtime defects: S1-R1 non-streaming leaf (one delta), S1-R2 planner empty-content hard fail, S1-R3
  template registry `default()` still used at boot (`http.rs:102,142,179`), S1-R4 per-run `HashMap<RunId,RunBudget>`
  never evicted, S1-R5 engine `check_calls(1)` only for planning vs provider-side call cap.
- Security: S6-F-01 (P1) tool subprocesses inherit env incl. `DEEPSEEK_API_KEY` (`tools/src/shell.rs:201`,
  `cli_support.rs:41`, `jail.rs:389`; 0 `env_clear` in backend). S2 also flags non-constant-time token compare
  (`core/src/ws.rs:113`), single shared boot token, `/health` absolute path.
- Release truth: tag `v1.0.0` = `3988622`; HEAD `5c69b66` is 3 commits later and the tree is dirty; shipped
  `dist/` binaries (server `184D51A7…` 09:10:59, zip `BC7A2BAD…` 09:33:17) predate the provider commit (10:59:06);
  `tools/release-manifest.json` was missing at 11:18 then appeared at 11:41 with hashes (`6b6f1d0b…`) contradicting
  `dist/CHECKSUMS.txt` (SEAT-05 S5-01/02/03/12, SEAT-08 S-1..S-14, SEAT-07 S7-F01..F04).
- Verification truth: no gate log is bound to HEAD (no HEAD/exit code/JUnit); audit verifier is rebuilt by
  `finalize_audit.py` (self-attestation); e2e 79/0/14 with 14 explicit skips; trace 43/319 W&W (SEAT-07).
- Docs: README/PHASE-12/CLAIM-REGISTER C-340 carry stale zip hash `9B2ED5A7…`/4,065,104 B vs actual
  `BC7A2BAD…`/6,571,928 B; FINAL-REPORT says 15 states / "no paid run" / pre-integration test counts (SEAT-08).

**Contradictions the planner must resolve (not inherit):**
1. Paper "provider never wired" vs HEAD source (resolved: paper = artifact era).
2. SEAT-04 says "no live server" and only READ evidence; SEAT-06 F-02 notes PHASE-B says key absent while
   `docs/evidence/*` claim live calls — both temporally consistent but **no artifact/commit binding** → live claims
   remain READ, never VERIFIED.
3. SEAT-05 S5-02 (manifest missing) superseded by S5-12 (manifest present but contradicts dist) — plan must treat
   release manifest as untrusted until regenerated with build metadata.
4. SEAT-03 D1 (`failed` not terminal) and S2 F-S2-02 (NODE_REWIRE hardcoded running) are real code defects but
   outside planner P1's provider/budget scope → route to Lead as owner-assigned, not silently dropped.

## 2. PLAN — planning decisions for P1

- **D1:** plan against current HEAD source `5c69b66` (provider wired), not the paper's artifact era; label every
  artifact-era claim "shipped artifact, pre-integration".
- **D2:** fix order by user impact: (a) subprocess key leak (security), (b) streaming + planner retry (core UX/robustness),
  (c) budget lifecycle/call-cap unification, (d) template registry, (e) acceptance-gate hardening.
- **D3:** no paid call in any acceptance test **except** the explicitly gated operator live-smoke (env-key, `--yes`,
  budget cap), and no key read by the planner/CI.
- **D4:** one writer per file, disjoint scopes: provider crate (adapter/client/budget) vs tools crate (shell/jail/spawn
  env) vs core HTTP boot/template loading; Lead integrates and runs the gate.
- **D5:** every fix lands with a discriminating non-paid test (wiremock/fixture) that fails before and passes after;
  live proof is a separate, human-authorized receipt.

## 3. LEAD — cycle gate

- Read gate: 48 seat files read fully, one seat at a time; paper (244 lines, `HEAD 3988622` line 4) and 4 official docs
  re-fetched HTTP 200 this cycle; seat contradictions enumerated above. PASS.
- Evidence discipline: planner writes only under `plans/planners/PLANNER-01/`; no product/git/key/paid action. PASS.
- **Cycle 1 verdict: PASS** — evidence corpus sufficient for a decision-complete plan; no blocker.
