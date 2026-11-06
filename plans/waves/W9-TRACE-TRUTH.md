# W9 — TRACE TRUTH (plan; forgewright-plan sections)

**Seat:** W9B trace-truth · **Write scope:** `docs/trace/**`, `plans/waves/**` only · **HEAD:** `8a5b583`
(2026-11-06T14:05:32-06:00) · **Mode:** read-only on product/tests/e2e/tools/audit; no commits.
**Recon sources read:** `docs/trace/controls.json` (1,936 lines), `docs/trace/controls-verifier-skeleton.json`
(**path discrepancy:** it lives at `docs/audit/evidence/controls-verifier-skeleton.json`, not `docs/trace/`;
read there), `docs/trace/TRACE-SCHEMA.md`, `docs/trace/QA-HARNESS.md`, `docs/trace/live/README.md`,
`docs/evidence/live-provider-smoke.md`, plus `docs/RELEASE-EVIDENCE.md`, `plans/waves/W3-STREAMING.md`,
`plans/waves/W4-REPLAY-PROOF.md`, `e2e/specs/replay-state.spec.ts`, `e2e/test-results/.last-run.json`,
`docs/audit/evidence/final/live-probe-final.json`, `tools/qa/{verify-harness,apply-live-trace}.mjs`.

## 1. Outcome
`docs/trace/**` states the truth about what is proven at the 1.1 wave-2 baseline: every non-PENDING hop carries a
repo pointer that exists, stale/polluted verdicts are removed or downgraded, and the recorded totals are reproducible
from a quoted command. No status inflation and no fabricated captures.

## 2. Current behavior (evidence)
- Before W9, `controls.json` counted `43 WIRED-AND-WORKS / 270 PENDING / 6 MISSING` (319 hops), but its own
  `_meta.live_capture.hops_evidence_backed = 48`; the capture file gives `43 WIRED / 260 UNVERIFIED / 5 MISSING`
  (308 hops) + `S-APPROVAL` 11 PENDING. The mismatch is a stale `S22 6_service = MISSING` from a pre-fix capture
  (`docs/trace/live/trace-capture.json#S22` now: present, ACK+FILE_TREE_RESULT received; `live-app/14-file-tree.json`:
  entries rendered). `TRACE-SCHEMA.md` and `live/README.md` repeated the stale counts.
- Wave-2 evidence was never applied: streaming (`plans/waves/W3-STREAMING.md:143-150`: provider SSE 8 passed,
  core provider_streaming 6 passed, run_lifecycle 7 passed, workspace 261 passed/0 failed); replay
  (`e2e/specs/replay-state.spec.ts` + `e2e/helpers/replay-harness.ts`: HISTORY_SNAPSHOT projection inert/no drag/no
  select; `web/tests/graph.replay.test.ts`, `web/tests/state.parity.test.ts`; `e2e/test-results/.last-run.json` =
  `{"status":"passed","failedTests":[]}`); release-binary live smoke (`docs/RELEASE-EVIDENCE.md:21-23`:
  SETTINGS_STATUS, SUBMIT_PROMPT -> REPLY_DONE usage 99, HISTORY_REVISION_SET revision 0 -> 2-node HISTORY_SNAPSHOT);
  tracked browser-origin roundtrips (`docs/audit/evidence/final/live-probe-final.json`).

## 3. Decisions
1. **Only repo pointers can back a verdict.** Two new evidence classes are accepted because the schema allows
   `log`/`file` kinds: (a) observed test output quoted in a committed wave plan, (b) integration/browser spec source +
   run-status file. No prose-only claim became WIRED.
2. **Do not upgrade browser render hops without a captured DOM artifact.** `S10 10_render_back` is set `UNVERIFIED`
   (not WIRED, not PENDING) because the replay browser proof exists and passes but no DOM/screenshot names it.
3. **Fix stale truth instead of preserving history:** `S22 6_service MISSING -> WIRED`; historical capture notes keep a
   `W9 HISTORY:` marker; counts are recomputed, not copied.
4. **`events.json.delivered[]` stays empty** — `tools/qa/verify-harness.mjs:111` fails on fabricated deliveries; only
   source-derived `emitted`/`subscribed` pointers are added, parity stays NOT_COLLECTED.
5. **`traces.json.traces[]` stays empty** — no captured TraceRun envelope exists; the index of available evidence is
   recorded in `_meta`, not as fake runs.

## 4. File-by-file change
- `docs/trace/controls.json`: `_meta.counts` recomputed (49 WIRED / 264 PENDING / 5 MISSING / 1 UNVERIFIED / 319),
  `wave2_evidence` block with pointers, `verification` block with command + validator; S27 6_service/7_store/8_provider/
  9_response -> WIRED (integration-test logs/files); S10 2_handler -> WIRED (source) and 10_render_back -> UNVERIFIED
  (run-status only); S22 6_service stale MISSING -> WIRED (tracked live-probe roundtrip + live-app capture); notes
  normalized for S21/S22/S15-S19/S-APPROVAL with exact missing evidence.
- `docs/trace/traces.json`: `_meta` updated — NO_RUNS with evidence index; `traces[]` remains `[]`.
- `docs/trace/events.json`: `_meta.status` partial source-derived; one emitted (REPLY_DELTA) + one subscribed
  (HISTORY_SNAPSHOT) pointer; `delivered`/`payload_parity`/`terminal_records` stay empty.
- `docs/trace/terminal-truth.md`: dated W9 note — still NOT_COLLECTED; names partial READ-level evidence and the
  missing DB/snapshot/persisted-events capture.
- `docs/trace/{QA-HARNESS.md,TRACE-SCHEMA.md,live/README.md}`: W9 dated correction block replaces the stale W8
  counts with the recomputed truth + command.
- This plan.

## 5. Failure modes
- Evidence pointer rots (file moves): validator does not check pointers; mitigation = pointers name line ranges/files
  and a future gate can hash them. Recorded as residual.
- Over-claiming: rejected by keeping browser-render wins at UNVERIFIED; only test-observed/log-backed hops moved.
- Count drift: single JSON source; recompute command quoted in docs and re-run at validation.
- `events.json` fabrication: prevented by leaving `delivered[]` empty (validator rule).

## 6. Tests / validation
- `node tools/qa/verify-harness.mjs` — static contract checks; must end `fail=0` (delivered[] empty, 11 hop keys,
  29 controls, non-PENDING hops evidence-backed).
- `python -c "import json,collections;d=json.load(open('docs/trace/controls.json'));..."
  -> `{'WIRED-AND-WORKS': 49, 'PENDING': 264, 'MISSING': 5, 'UNVERIFIED': 1}` (quoted in the report).
- JSON parse check for controls/traces/events (json.load) and Python assertion that every non-PENDING hop has evidence.

## 7. Rollback
All edits are confined to `docs/trace/**` (+ this plan). Revert the files with `git checkout -- docs/trace plans/waves/W9-TRACE-TRUTH.md`
(no product/test/e2e/tool changes, no commits by this seat).

## 8. Out of scope / blockers
- No new captures: paid provider runs, live server capture, Playwright re-runs are not performed here (read-only lanes);
  hence S10 live round-trip hops, S15-S19 node cards, S27 progressive rendering and S-APPROVAL overlay stay
  PENDING/MISSING with named missing evidence.
- Working tree was **not quiescent**: `git status` shows `M web/src/api/index.ts` and `?? web/src/api/http.ts` from
  another lane (uncommitted product work). W9 does not touch them; any live capture made now would be non-reproducible.
- `controls-verifier-skeleton.json` is not at the path named in the brief (`docs/trace/`), it is committed under
  `docs/audit/evidence/`; recorded, not moved (audit lane owns it).

## 9. Execution log / observed commands
```
$ python -c "...Counter(h['verdict'] ...)"
{'WIRED-AND-WORKS': 49, 'PENDING': 264, 'MISSING': 5, 'UNVERIFIED': 1}, total 319 hops
$ node tools/qa/verify-harness.mjs
checks=27 pass=27 fail=0
OK: QA harness structure is consistent with the frozen contract; no execution claims made.

$ python -c "import json,collections;d=json.load(open('docs/trace/controls.json'));c=collections.Counter(h['verdict'] for x in d['controls'] for h in x['hops'].values());print(dict(c))"
{'WIRED-AND-WORKS': 49, 'PENDING': 264, 'MISSING': 5, 'UNVERIFIED': 1}

$ python -c "... assert every non-PENDING hop has non-empty evidence[]"
non-PENDING without evidence: []   (29 controls, 319 hops)

$ git status --porcelain docs/trace plans/waves
 M docs/trace/QA-HARNESS.md / TRACE-SCHEMA.md / controls.json / events.json / live/README.md / terminal-truth.md / traces.json
?? plans/waves/W9-TRACE-TRUTH.md (+ W9 apply/verify scripts; no product files touched by this seat)
```
Applied evidence pointers (all exist at HEAD `8a5b583`):
`plans/waves/W3-STREAMING.md:143-150`; `backend/crates/hiverouter-provider/tests/streaming.rs`;
`backend/crates/hiverouter-core/tests/provider_streaming.rs`; `e2e/specs/replay-state.spec.ts`;
`e2e/test-results/.last-run.json`; `docs/audit/evidence/final/live-probe-final.json`;
`docs/trace/live/live-app/14-file-tree.json`; `web/src/flow/Canvas.tsx:114-150`; `web/src/flow/graph.ts:96`;
`docs/RELEASE-EVIDENCE.md:21-23`.


## 10. Observed validation (quoted; run by this seat)
- `node tools/qa/verify-harness.mjs` -> **checks=27 pass=27 fail=0**; `events.json delivered[]` empty; `traces.json []`.
- Recomputed counts (see §9): **49 WIRED / 264 PENDING / 5 MISSING / 1 UNVERIFIED = 319 hops**; 54 evidence-backed
  (excludes the single UNVERIFIED hop). Every non-PENDING hop has non-empty `evidence[]` (asserted in the apply script).
- `controls-verifier-skeleton.json`: the brief named `docs/trace/controls-verifier-skeleton.json`, which does not exist
  in this tree; the committed file is `docs/audit/evidence/controls-verifier-skeleton.json` (baseline `d1638f2`).
  Sections read: `counts` (28 surfaces / 0 WIRED at Phase-A baseline), `verdict_definitions`, `gaps` (GAP-UI-01..05,
  pre-UI), `expected_probe`, `frontend_state_at_baseline`. It is a Phase-A skeleton, not a current count source;
  W9 does not edit it (audit lane owns it).

## 11. Report shape (seat receipt)
- **Done:** `controls.json` corrected and recomputed (49/264/5/1 of 319, 54 evidence-backed) with `wave2_evidence` +
  `verification` blocks; `traces.json` NO_RUNS + evidence index; `events.json` partial source-derived (delivered empty);
  `terminal-truth.md` W9 note (still NOT_COLLECTED); W9 dated corrections replace stale counts in QA-HARNESS /
  TRACE-SCHEMA / live README; this plan; validator 27/27; counts reproducible via the quoted command.
- **Not done (by design):** no new captures — S10 live round-trip hops, S15-S19 node cards, S27 UI progressive
  rendering, S-APPROVAL overlay stay PENDING/MISSING with named missing evidence; skeleton file not relocated.
- **Unverified:** `e2e/test-results/.last-run.json` does not name the replay test (only status=passed at
  2026-11-06T20:02:54Z); the browser replay proof used a mocked WS transport (not a live-run trace); the release-smoke
  replay claim (`docs/RELEASE-EVIDENCE.md:23`) is READ-level with no committed raw frames/DB row.


---

## W4 CORRECTION (supersedes W9 counts) — 2026-11-06T21:57:09.616Z
- W9 reported 49 WIRED / 264 PENDING / 5 MISSING / 1 UNVERIFIED (54 evidence-backed). The underlying raw capture `docs/trace/live/trace-capture.json` was later overwritten by an e2e run with `surfaces: []` (generated_at 2026-11-06T20:25:04.044Z, 286 B), so 43 capture-derived WIRED verdicts and 5 non-raw W9 promotions (S10/2_handler, S27/6-9) are not supported by current raw evidence. W4 downgraded them to UNVERIFIED with `w4_correction` history; S22/6_service remains WIRED on tracked raw live-probe + DOM evidence. Corrected counts: **1 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319 hops**. See `plans/waves/W4-TRACE-CAPTURE.md` and `docs/trace/CAPTURE-REGISTRY.md`.
