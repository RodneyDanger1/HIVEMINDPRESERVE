# FINDINGS — FigCraft-B seat (documentation/evidence tier)

Scope: 91 files (`docs/plans/**` 26, `docs/audit/**` 59, `docs/claims/**` 3,
`docs/contracts/**` 2, `docs/reports/**` 1). Audit HEAD `3988622` (short), read-only.
Manifest integrity is clean: **91/91 sha256 match**, so every finding below is about
**content/claim truth**, not file corruption. Severity is for the audit paper.

## F-01 — Claim register JSON is internally inconsistent (stale "unverified" block) — HIGH
- `docs/claims/CLAIM-REGISTER.json:3-9` declares `total 80 / verified 72 / read 8 / unverified 0 / contradicted 0`.
- Same file `:10-16` keeps `decision_critical_unverified: [{claim_id: C-026, ...}]` and `:86` says
  "C-026 (browser e2e) remains the only open UNVERIFIED item".
- `docs/claims/CLAIM-REGISTER.csv:36` records C-026 status **VERIFIED** ("79 passed / 0 failed / 14 explicit fixme skips"),
  and `docs/reports/FINAL-REPORT.md:46` states "0 UNVERIFIED".
- Resolution (docs-only): delete/retire the `decision_critical_unverified` block or mark it resolved in
  `:86`. Git history shows C-026 was promoted in `ff24707` and the register restored to 80 rows in `7fb0495`;
  the JSON body was not fully re-synchronised. Evidence: CSV row counts computed this audit
  (80 rows = 72 VERIFIED + 8 READ); `git log --oneline` heads `7fb0495 → ff24707 → 6fe05e8 → 3988622`.

## F-02 — FINAL-REPORT trace counts contradict the trace census — MEDIUM
- `docs/reports/FINAL-REPORT.md:45`: "43 `WIRED-AND-WORKS` hops with captured evidence; **260 UNVERIFIED; 5 MISSING**".
- `docs/trace/controls.json` (read this audit): 29 controls = 23 `PARTIALLY_VERIFIED` + 6 `PENDING_APP_NOT_BUILT`;
  hop verdicts = **43 WIRED-AND-WORKS + 270 PENDING + 6 MISSING = 319**.
- `docs/audit/FINAL-DELTA-VERIFICATION.md:50` (O-4) already states "43 WIRED-AND-WORKS / 270 PENDING / 6 MISSING",
  so FINAL-REPORT is the stale document (260 vs 270; 5 vs 6). No product claim changes: all are honest-unverified.

## F-03 — FINAL-CLOSEOUT cites a passing `.last-run.json`, but the committed artifact says failed — MEDIUM
- `docs/audit/FINAL-CLOSEOUT.md:25`: "`.last-run.json` passed".
- `docs/audit/evidence/final/pw-out/.last-run.json:2` = `"status": "failed"` (1 failed test id);
  `docs/audit/evidence/final/pw-out/ws-contract-.../error-context.md:14` = "Timeout 1ms exceeded while waiting on the predicate".
- This is the known **test-code flake** documented in `docs/audit/FINAL-DELTA-VERIFICATION.md:49` (O-3:
  `e2e/specs/ws-contract.spec.ts:38` `expect.poll(…, {timeout: 1})` before its own `test.fixme`), not a product failure —
  but the closeout should cite `e2e/test-results/run-live-FINAL-artifact.log` (79 passed/0 failed) instead, and
  `evidence/final/pw-out/` should carry a marker that it stores the independent verifier's flaky run.

## F-04 — FINAL-DELTA's FAIL verdict on the claim register is stale at current HEAD — MEDIUM
- `docs/audit/FINAL-DELTA-VERIFICATION.md:12,23,46` (G6/O-9): final register FAIL (CSV 34 rows vs JSON 79) at `01bfff4`.
- At current HEAD `3988622`: `CLAIM-REGISTER.csv` = **80 rows** (72 VERIFIED/8 READ, C-001..C-340) and
  `CLAIM-REGISTER.json` = **80** → the deliverable FAIL is repaired; commits `7fb0495` ("restore claim register 79->80"),
  `ff24707` (C-026 VERIFIED), `6fe05e8` (count sync), `3988622` (FINAL-REPORT sync).
- Residual is F-01 (JSON body not fully synced). The report is honest but chronological: label it superseded.

## F-05 — Headline-gate evidence cited by the final reports is untracked by git — MEDIUM
- `docs/reports/FINAL-REPORT.md:36` cites `docs/audit/evidence/final-gate-tests.log` — **untracked**.
- `docs/reports/FINAL-REPORT.md:38` cites `e2e/test-results/run-live-FINAL-artifact.log` — **untracked**.
- `docs/audit/FINAL-DELTA-VERIFICATION.md:8` cites `cargo-test-final.log` and `e2e-verifier-independent.log` — both **untracked**.
- The wave inventory covers the **507 tracked** files; these logs exist on disk only, so the audit paper cannot
  hash-verify the most important gates through `FILE-MANIFEST.csv`. Fix: commit the logs or record them as
  untracked evidence with hashes.

## F-06 — Duplicate evidence files imply supersession that did not happen — LOW
- `docs/audit/evidence/phase-b/live-probe-verifier.json` and `…-v2.json` are **byte-identical**
  (sha256 19D660FB…): the "v2" capture adds no evidence.
- `docs/audit/evidence/phase-b/cli/census-verify.json` and `census.out` are **byte-identical** (sha256 1E0139F3…).
- 17 `.err` captures are 0 B (clean stderr — expected); `version-human.out` and `version.err` are both 0 B
  (`version-human.err` 726 B holds the human line, matching the documented JSON-stdout/human-stderr split).

## F-07 — FINAL-CLOSEOUT records HEAD `6815dbe`; actual HEAD/tag is `3988622` — LOW
- `docs/audit/FINAL-CLOSEOUT.md:9`: "HEAD / tag | `6815dbe` + closeout docs commit · `v1.0.0`".
- `git rev-parse HEAD` = `3988622`; `git tag --points-at HEAD` = `v1.0.0`; `6815dbe` is 3 commits back
  (artifact packaging). The phrase is defensible but the table value is not the actual frozen HEAD.

## F-08 — Claim-register notes keep the pre-freeze binary hash — LOW
- `docs/claims/CLAIM-REGISTER.json:86` cites "release build exit 0 with hiverouter.exe sha256 `DE676AFB...`".
- Final artifact is `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9`
  (9,329,152 B, re-hashed this audit: MATCH); CLI `51E1ABA6…` matches. C-334 (`CLAIM-REGISTER.csv:79`)
  keeps the historical hash; C-340 (`:81`) carries the final hash — the JSON `notes` should state the supersession.

## F-09 — Cleared pseudo-anomaly: "2026-10-13" dates are toolchain build dates — INFO (no defect)
- `docs/audit/BASELINE-VERIFICATION.md:35`, `docs/plans/TOOLCHAIN.md:7`, `PHASE-1-EVIDENCE.md:13`,
  `PHASE-5-EVIDENCE.md:10`, `PHASE-6-EVIDENCE.md:9`, `PHASE-12-EVIDENCE.md:49` contain `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
  — that is the compiler build date, not project chronology. No stale-date defect.

## F-10 — Clean-tree claim holds for product code, not for the current workspace — INFO
- `docs/audit/FINAL-CLOSEOUT.md:13,19` claim empty `git status --porcelain`. At audit time the only dirty path is
  untracked `audit/parts/` produced by this audit wave; `git status --porcelain` filtered to product areas
  (`backend/`, `web/`, `e2e/`, `docs/` tracked files) is empty. So the claim is true for product scope and the
  difference is audit output, not a contradiction.

## Cross-partition open checks (not defects until other seats report)
- `docs/reports/FINAL-REPORT.md:19` claims "14 commands, 21 events, 65 refusal codes" for the backend contract;
  `docs/contracts/UI-COMMANDS-v1.md:18-31` lists the UI command vocabulary (10 command rows incl. client-only
  VIEWPORT/DIFF). Both can be true of different layers — needs the b-core contract crate to confirm (OUT OF SEAT).
- `docs/reports/FINAL-REPORT.md:30` cites figcraft A/B asset counts (55 rows / 28 surfaces, watchdog PASS 20/20);
  owned by the figcraft-a seat's manifest — cross-check there.
