# W8 — DOCS TRUTH (docs/**, audit/**, README.md; dated supersession only)

Seat: W8 Docs truth (write-authorized: `docs/**`, `audit/**`, `README.md`). Repo `C:\Users\Riley\Desktop\hiverouter-v1`, audit HEAD **`0dfe23b`** (tag `hivemind-1.0.0` → `23ba74a`; `v1.0.0` → `3988622`; `git status` clean at start). No product/git/tag writes, no key access, no paid calls. Commit reserved to Lead.

> **Scope note (recorded contradiction in the assignment):** the instruction says "do NOT edit … plans" but also requires this file at `plans/waves/W8-DOCS-TRUTH.md`. Resolved by writing **only** this new wave record under `plans/waves/` as the explicitly named deliverable; no existing plan content was modified. All content edits are confined to `docs/**` and `README.md`.

## Outcome

**Done — current-facing docs now state the current truth; historical records are preserved with dated supersession blocks, not rewritten.** 11 files changed (all docs/README); 19/19 read-only truth checks pass; no contradictions remain in current-facing claims. `tools/qa/check-docs-truth.mjs` was **not** created — `tools/**` is explicitly outside the W8 write scope (blocker documented below); an equivalent inline checker was executed read-only and its output is quoted in §Tests.

## Current (verified at HEAD `0dfe23b`, 2026-11-06)

| Fact | Value | Evidence |
|---|---|---|
| Release identity | tag `hivemind-1.0.0` = annotated object → commit `23ba74a`; old `v1.0.0` = `3988622` (pre-fix private checkpoint) | `git rev-parse 'hivemind-1.0.0^{commit}'` |
| Server artifact | `target/release/hiverouter.exe` 13,253,120 B sha256 `b17eb3d6469c347f623a4dedc94537b15bb633a76215828742624fa6fe062a9a` | `Get-FileHash` |
| CLI artifact | `target/release/hiverouter-cli.exe` 5,373,952 B sha256 `49f46e2be488dd675280f3dae7e8d455ac5eeec2aa46d7fd44f1c2557c386623` | `Get-FileHash` |
| Package | `dist/HiveMind-1.0.0-win64.zip` 8,444,021 B sha256 `46305fcb1fe1bee23d3645a94a82635a68ee47fbcd3ce0537df747de3c6105ef`; sidecar matches | `Get-FileHash` + sidecar |
| Web dist | `index.html` 8264ae92…, `index-6T84PJfk.js` b771bc2e…, `index-BWOl__uM.css` 96ffa873… (manifest `web_dist_files`, algorithm documented) | `tools/release-manifest.json` |
| UI state vocabulary | **16 states including `interrupted`**; `isTerminal()` includes `failed` | `web/src/state/agentState.ts` (count 16) |
| Trace ladder | **43 WIRED-AND-WORKS / 270 PENDING / 6 MISSING** (29 controls, 319 hops) | `docs/trace/controls.json` (parsed) |
| Backend gate | **237 passed / 0 failed / 2 ignored** on sealed source `6a2b1c6` | `docs/evidence/final-gate-tests.log` (HEAD line + sums) |
| Web gate | 46/46 unit, typecheck/build exit 0 | `docs/evidence/final-web-tests.log`, `final-web-build.log` |
| Live provider | key validation HTTP 200 `HIVE-KEY-OK` usage 20; live server run `completed` reply `HIVE-LIVE-OK` usage 95 (READ-level; not independently re-run) | `docs/evidence/key-validation.md`, `live-provider-smoke.md` |
| Provider wiring | core→provider dep, `build_executor` adapter/fallback, child spawns remove key | `core/Cargo.toml:21`, `http.rs:67,72,79`, `shell.rs:203`, `cli_support.rs:42` |
| Outdated rows corrected | 15→16 states; 260/5→270/6; 233→237/2; old artifacts `184D51A7/9B2ED5A7/4,065,104/6,571,928` superseded by `b17eb3d6/46305fcb/8,444,021`; "no paid provider run" superseded by live evidence | see File-by-file |

## Decisions

1. **Dated supersession over rewrite.** Historical evidence files keep their body; each current-facing stale file gets a dated (2026-11-06) correction block stating current values and explicitly labelling the body as historical. This preserves the audit trail.
2. **Current release record of authority** = `docs/RELEASE-EVIDENCE.md` + `tools/release-manifest.json`; `docs/reports/FINAL-REPORT.md`, `docs/plans/PHASE-12-EVIDENCE.md` and the trace live/QA records are historical snapshots for the pre-1.0.0 artifact.
3. **Claims register kept at 80 rows / 72 VERIFIED / 8 READ**: only row text/notes were corrected (C-020 16 states; C-026/C-340 current artifact references); JSON `decision_critical_unverified` emptied because C-026 is VERIFIED, and stale `DE676AFB` note replaced. No status silently promoted.
4. **Key-scan phrasing made precise**: strict credential pattern (`sk-[A-Za-z0-9]{32,}`) = 0 over 677 tracked non-vendor files; literal `sk-` substrings exist only in obvious unit-test fixtures/embedded binary strings. This removes a broad-pattern overclaim without hiding anything.
5. **Web-dist hash fixed to per-file** (already done at release commit `87dada4`): `docs/RELEASE-EVIDENCE.md` no longer cites the retired opaque aggregate `86a1c088…` as current; it points at `web_dist_files`.
6. **`tools/qa` checker not added**: outside scope. Documented as a blocker with a drop-in inline equivalent.

## File-by-file (all edits are dated corrections; no historical body rewritten)

| File | Change | Why |
|---|---|---|
| `docs/reports/FINAL-REPORT.md` | Prepended dated supersession block: current 1.0.0 artifacts, 16 states, 237/0/2, 270/6, live-provider evidence; body labelled historical (old hashes/15-state/260/5/"no paid run") | Was shipping pre-1.0.0 artifact hashes and stale counts as if current |
| `docs/plans/PHASE-12-EVIDENCE.md` | Prepended dated supersession block with current artifact hashes/sizes + 237/2 + web-dist pointer; `:136-143` body labelled pre-1.0.0 | Final-artifact block cited `184D51A7/9B2ED5A7/4,065,104` |
| `docs/plans/FRONTEND-STATUS.md` | Prepended dated correction: 15→**16** states incl `interrupted`; contract counts updated to 14/21/65; line 73 body labelled historical | Milestone snapshot asserted 15-state enum |
| `docs/trace/live/README.md` | Prepended dated correction: current artifacts (`b17eb3d6…/46305fcb…`), current assets (`index-6T84PJfk.js`), current trace totals 43/270/6, live-provider evidence; body historical (`184D51A7/Cbfm1IuM/260/5`) | Live-QA record from pre-1.0.0 artifact |
| `docs/trace/QA-HARNESS.md` | Prepended dated correction (same facts + final e2e must be re-established on rebuilt artifact); old status line marked historical | 260/5 overclaim + old artifact |
| `docs/trace/TRACE-SCHEMA.md` | Prepended dated correction to 43/270/6 | 260/5 stale |
| `docs/claims/CLAIM-REGISTER.csv` | C-020 15→16 states; C-026 note updated (1.0.0 artifact `b17eb3d6…`, live provider evidence, remaining 1.1 rendering); C-340 note replaced with 1.0.0 artifacts `b17eb3d6/49f46e2b/46305fcb` + manifest | Old rows cited `184D51A7/9B2ED5A7/4,065,104` and "no paid run" |
| `docs/claims/CLAIM-REGISTER.json` | `decision_critical_unverified: []`; notes refreshed (1.0.0 artifacts, READ-level live evidence, 43/270/6 trace) | Stale `DE676AFB` note; C-026 still listed unverified while counts say 0 |
| `docs/RELEASE-EVIDENCE.md` | Web-dist reference → per-file `web_dist_files` (former aggregate retired); key-scan wording made strict-pattern precise | N1 follow-through; broad-scan wording |
| `docs/RELEASE.md` | Key-scan wording made strict-pattern precise | Same overclaim |
| `README.md` | Added current 1.0.0 artifact hashes/sizes + note that `FINAL-REPORT.md` is historical/superseded | Prevent old-hash confusion |
| `plans/waves/W8-DOCS-TRUTH.md` (new) | This wave record | Explicit deliverable (see scope note) |

**Intentionally not edited (historical evidence, correctly dated):** `docs/audit/FINAL-CLOSEOUT.md`, `FINAL-DELTA-VERIFICATION.md`, `QA-ENV.md`, `PHASE-B-VERIFICATION.md`, `docs/plans/PHASE-7-8-10-{EVIDENCE,RECON}.md`, `docs/trace/A11Y-CONTRAST-PLAN.md` — their `184D51A7/9B2ED5A7/260 UNVERIFIED` values are records of the pre-1.0.0 state. `audit/HIVEMIND-AUDIT-PAPER.md` already carries the post-release supersession block (A-F01/A-F02/A-F03 historical at `6a2b1c6`/`hivemind-1.0.0`); no change needed.

## Failure

- **No test-failing contradiction remains** in the current-facing set (19/19 checks pass).
- One checker revision was needed: the first run treated any `86a1c088` occurrence as failure, but the corrected text intentionally cites it once as retired history ("former opaque aggregate … retired"); condition amended to allow the historical mention. No doc change resulted from the false failure.
- **Blocker:** `tools/qa/check-docs-truth.mjs` cannot be added — `tools/**` is outside the W8 write scope; adding it would need the Lead or a tools-scope owner. Drop-in content: the inline checker below (read-only; exits non-zero on any failure) can be saved as that file unchanged.
- **Concurrent work noted:** `git status` during this task also shows non-W8 modifications (`backend/**`, `tools/build-release.ps1`, `web/tests/*`) from other active seats; W8 did not touch them, and no conflict was created. Re-run the checker after those land.

## Tests (executed read-only; quoted output)

`tools/qa` not written (out of scope) — equivalent inline PowerShell checker run at HEAD `0dfe23b`:

```text
=== CHECK-DOCS-TRUTH (v2 — inline; tools/qa is outside W8 write scope) ===
PASS  artifact-exe  b17eb3d6… / 13,253,120 B
PASS  artifact-cli  49f46e2b… / 5,373,952 B
PASS  artifact-zip  46305fcb… / 8,444,021 B
PASS  states-16  agentState states=16
PASS  trace-counts  W^W=43 PENDING=270 MISSING=6
PASS  final-report-supersession  dated supersession with corrected counts
PASS  phase12-supersession  dated correction present
PASS  frontend-16-states  dated correction present
PASS  trace-doc docs/trace/live/README.md  270 PENDING + dated correction
PASS  trace-doc docs/trace/QA-HARNESS.md  270 PENDING + dated correction
PASS  trace-doc docs/trace/TRACE-SCHEMA.md  270 PENDING + dated correction
PASS  release-evidence-webdist  aggregate hash retired; per-file manifest reference present
PASS  readme-artifacts  current 1.0.0 hashes present
PASS  release-md-keyscan  precise key-scan wording
PASS  claims-csv-parses  rows=80
PASS  claim-C020-16states  C-020 text updated
PASS  claim-C340-final-artifacts  C-340 notes updated to final artifacts
PASS  claims-json-parses  total=80 verified=72 read=8
PASS  claims-json-decisions-empty  decision_critical_unverified=[]
PASS  claims-json-notes-refreshed  current artifact + trace counts in notes

RESULT: ALL CHECKS PASS (19/19)
```

Stub sweep over all 11 changed files: `stub_hits=10`, every hit **classified permitted/historical** — `test.fixme` references (FINAL-REPORT:40,55; PHASE-12:142), "placeholder" describing the pinned UI chat-core placeholder (FRONTEND-STATUS:16), "not implemented" describing intentional decision-gated/unimplemented 1.1 items (FINAL-REPORT:64, QA-HARNESS:194), and `test.fixme` e2e skips (trace live README:71,73; QA-HARNESS:13; claims C-026). No TODO/FIXME/HACK/mock/fake code was added.

Changed-file sha256: `README.md` E3A50037…; `docs/RELEASE-EVIDENCE.md` 98B07358…; `docs/RELEASE.md` CBA3952D…; `docs/claims/CLAIM-REGISTER.csv` A6D46F6A…; `CLAIM-REGISTER.json` FA13627A…; `docs/plans/FRONTEND-STATUS.md` A697F19E…; `docs/plans/PHASE-12-EVIDENCE.md` 3B434D69…; `docs/reports/FINAL-REPORT.md` DED98235…; `docs/trace/QA-HARNESS.md` 544D196B…; `docs/trace/TRACE-SCHEMA.md` BBABEF32…; `docs/trace/live/README.md` 36A996B8….

## Rollback

- All W8 edits are text-only and reversible with `git restore -- <file>` per file: `git restore -- docs/reports/FINAL-REPORT.md docs/plans/PHASE-12-EVIDENCE.md docs/plans/FRONTEND-STATUS.md docs/trace/live/README.md docs/trace/QA-HARNESS.md docs/trace/TRACE-SCHEMA.md docs/claims/CLAIM-REGISTER.csv docs/claims/CLAIM-REGISTER.json docs/RELEASE-EVIDENCE.md docs/RELEASE.md README.md`, then `rm plans/waves/W8-DOCS-TRUTH.md`. No product/runtime/build state is affected; no data directory, database, artifact, or tag is touched.
- If a dated block is later deemed too prominent, it is a single block per file (all begin with `> **Superseded for current release state` or `> **Dated correction`), removable independently.

## Out of scope (explicitly not done)

- `backend/**`, `web/**`, `tools/**`, `plans/**` (except this required W8 record), release tags, commits — untouched.
- Historical audit/live-QA bodies were not rewritten (dated blocks only); historical files listed above were left as-is.
- No new claims invented; no statuses promoted; live-provider evidence remains READ-level (not independently re-run — no key access).
- Full browser e2e / test rebuild on the 1.0.0 artifact was not run (out of scope; final e2e result still needs to be established by the QA owner on `b17eb3d6…`).

## Done / Not done / Unverified

| Done | Evidence |
|---|---|
| 11 current-facing docs/README files reconciled with dated corrections; historical files preserved | file-by-file table; quoted 19/19 checker §Tests |
| All known stale claims fixed or explicitly labelled historical: 15-state, 260/5, "no paid provider run", old zip/binary hashes, `86a1c088` aggregate, stale claim rows, stale JSON notes | checker passes for each |
| No-scope-escape verified | `git status` shows only docs/README (plus other seats' files) changed by W8; no tag/commit/product write |

| Not done | Reason |
|---|---|
| `tools/qa/check-docs-truth.mjs` | `tools/**` outside W8 write scope → blocker documented with drop-in checker content |
| Final-artifact browser e2e claim on `b17eb3d6…` | QA owner work; historical 79/0/14 was pre-1.0.0 binary — docs say so |
| Editing historical audit/live-QA bodies | Deliberately out of policy: dated supersession, not history rewrite |

| Unverified | What would prove it |
|---|---|
| Live provider claims (`HIVE-ARTIFACT-OK`, usage 99; `HIVE-LIVE-OK`, usage 95) | Re-run with a key on the 1.0.0 artifact and capture events/usage (read-only audit allowed no key) |
| 1.0.0-artifact e2e still green | Run Playwright against `b17eb3d6…` and log result |
| Stale strings in historical files (intentionally kept) | `grep` still finds `184D51A7/9B2ED5A7/260 UNVERIFIED` in dated audit/live records — correct by design |
