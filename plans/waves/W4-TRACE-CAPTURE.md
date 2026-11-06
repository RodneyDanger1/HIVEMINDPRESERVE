# W4-TRACE-CAPTURE — trace truth after raw-capture loss (Seat 4)

**Seat:** Trace truth/captures · **Write scope:** `docs/trace/**`, `plans/waves/**` only · **HEAD at start:** `c9390f5`
(dirty from other lanes; this seat touched no product/web/e2e/tool files) · **Mode:** read-only on everything outside scope; no commit.
**Skills used (all five, in order):** `/forgewright-recon` → `/forgewright-lead` → `/logic-gatekeeper` → `/integration-logic` → `/logic-debugger`.

## 1. RECON (forgewright-recon ledger)

- **Target behaviour:** the trace corpus must state only what committed raw capture supports; no hop may be `WIRED-AND-WORKS` while its cited raw capture is missing/empty.
- **Entry point:** `docs/trace/controls.json` (`_meta.counts`, `_meta.live_capture`, per-hop `verdict`/`evidence[]`); `docs/trace/traces.json`; `docs/trace/events.json`; `docs/trace/terminal-truth.md`; consumers `docs/trace/{QA-HARNESS,TRACE-SCHEMA}.md`, `docs/trace/live/README.md`, `plans/waves/W9-TRACE-TRUTH.md`.
- **Callers/consumers (path:line):** `tools/qa/verify-harness.mjs:94-111` (validates non-PENDING evidence presence), `tools/qa/apply-live-trace.mjs` (writes controls from a capture — outside this seat's write scope), `e2e/specs/trace-hops.spec.ts` (produces `trace-capture.json` — outside scope), audit/paper claims in `audit/HIVEMIND-AUDIT-PAPER.md` and `plans/waves/W9-TRACE-TRUTH.md`.
- **Tests + command:** `node tools/qa/verify-harness.mjs` (static contract/trace-seed gate; does **not** check evidence-pointer existence). Count command: `node -e "…Counter(h['verdict'])…"`.
- **Data flow (one success, one failure):**
  - success: `e2e/specs/trace-hops.spec.ts` afterAll → `docs/trace/live/trace-capture.json` (28 surfaces, 43 WIRED/260 UNVERIFIED/5 MISSING) → `apply-live-trace.mjs` → `controls.json` (43 WIRED) → docs claims.
  - failure (observed): a later e2e run with no app records wrote `trace-capture.json` = `{"surfaces":[]}` at `2026-11-06T20:25:04.044Z` (286 B) → the 43 WIRED verdicts + 48 evidence pointers silently dangle; `controls.json` still reports `WIRED-AND-WORKS: 49`, `verbose live_capture.generated_at: 15:24:24Z`, and docs repeat 49/264/5/1.
- **Unknowns/evidence missing:** no raw per-hop capture for S10 live round-trip, S15-S19 node cards, S27 progressive UI, S-APPROVAL overlay; no DB/WS/persisted-events raw artifacts. Current server not running (checked earlier lanes); capture commands cannot execute in this seat (e2e write scope excluded).
- **Files expected to touch (scope-checked):** `docs/trace/controls.json`, `docs/trace/traces.json`, `docs/trace/events.json`, `docs/trace/terminal-truth.md`, `docs/trace/{QA-HARNESS,TRACE-SCHEMA}.md`, `docs/trace/live/README.md`, new `docs/trace/{CAPTURE-REGISTRY.md,capture-registry.json}`, this plan; append correction note to `plans/waves/W9-TRACE-TRUTH.md`. No product/web/e2e/tools edits.

## 2. LEAD CONTRACT (forgewright-lead)

- Root cause must be proven before edits: **raw capture loss is silent** — the file exists (non-null) but has `surfaces: []`, so existence-only checks pass while 48 evidence pointers are semantically dead. This is a content-validity flaw, not a missing-file flaw.
- Evidence classes: VERIFIED (re-read by this seat) / READ (file content) / INFERRED / UNVERIFIED. No "should be".
- Gate: `logic-gatekeeper` Phase 1 is recorded below before any mutation. Validation by running commands, quoted, not prose.

## 3. LOGIC-GATEKEEPER Phase 1 (recorded before any write to docs/trace)

```json
{
  "phase1": {
    "architecture": "docs/trace/controls.json stores 29 controls × 11 hop verdicts with evidence[] pointers; docs/trace/live/trace-capture.json is the raw browser capture that e2e/specs/trace-hops.spec.ts regenerates on every run; docs pages and W9 plan repeat the derived counts; tools/qa/verify-harness.mjs validates shape/evidence-presence but not pointer content-existence.",
    "dataFlow": "e2e afterAll writes trace-capture.json (surface records) → tools/qa/apply-live-trace.mjs copies non-UNVERIFIED hops + evidence into controls.json → W9 further upgraded 6 hops from source/test-log pointers → docs quote counts. Failure: a run with no captured app records rewrites trace-capture.json as {surfaces:[]}; controls keeps old WIRED verdicts and docs keep old counts with no invalidation.",
    "edgeCases": ["empty-but-existing capture file (observed: 286 B, surfaces=[] at 2026-11-06T20:25:04.044Z) — existence check passes, content supports nothing", "non-capture W9 upgrades (S10 2_handler source file; S27 6-9 test-output/log pointers) — PENDING→WIRED without raw capture", "evidence pointer whose file exists but whose referenced section is gone (live-probe roundtrip has FILE_TREE_LIST ACK+RESULT, still valid raw capture)", "S15-S19 MISSING hops with no DOM/card capture, and S-APPROVAL with no overlay capture"],
    "rootCause": "controls.json and the docs derive WIRED counts from a raw capture file whose content can be silently replaced by an empty run, and W9 additionally promoted six hops from non-raw pointers, so the reported 49 WIRED / 54-evidence-backed counts are not reproducible from any current raw capture; the failure closes only by validating evidence content (not mere file existence) and downgrading unsupported verdicts with a named correction."
  }
}
```

Phase 1 accepted: all fields non-empty, edge cases concrete, rootCause is a full contradiction sentence. Phase 2 (the correction itself) is below.

## 4. INTEGRATION-LOGIC — evidence provenance chain

| Hop family | Claimed (before) | Pointer class | Pointer exists? | Content supports it? | Correct verdict |
|---|---|---|---|---|---|
| 43 hops across S01-S14/S20-S28 (excl. A) | WIRED-AND-WORKS | `docs/trace/live/trace-capture.json#Sxx` | file exists (286 B) but `surfaces: []` | **NO** — empty capture | `UNVERIFIED` + W4 correction note |
| S10 2_handler | WIRED (W9) | `web/src/flow/Canvas.tsx`, `web/src/flow/graph.ts`, replay spec/run-status | yes | source handler only; no raw browser capture | `UNVERIFIED` (rule: no PENDING→WIRED without raw capture) |
| S27 6_service/7_store/8_provider/9_response | WIRED (W9) | `plans/waves/W3-STREAMING.md` quoted test output; backend test files | yes | observed test output, **not** a captured run/UI artifact | `UNVERIFIED` (no raw capture) |
| S22 6_service | WIRED (W9) | `docs/audit/evidence/final/live-probe-final.json` (tracked WS roundtrip: FILE_TREE_LIST → ACK+FILE_TREE_RESULT) + `docs/trace/live/live-app/14-file-tree.json` (DOM evidence) | yes | **YES** — tracked raw WS roundtrip + browser DOM evidence | keep `WIRED-AND-WORKS` |
| S15-S19 1_render | MISSING | none | — | no run/cards captured | keep `MISSING` |
| S-APPROVAL all hops | PENDING | none | — | no overlay capture | keep `PENDING` |

Failure path proven by the recon: `apply-live-trace` trusts capture verdicts without validating `surfaces.length`; the validator trusts evidence arrays without file-content hashes.

## 5. LOGIC-DEBUGGER — invariant, RED, GREEN

- **Invariant:** *No hop may be `WIRED-AND-WORKS` unless at least one evidence pointer resolves to a committed capture artifact whose content supports that hop; an empty capture invalidates every verdict derived from it.*
- **RED (observed before correction, read-only):** `node tools/qa/verify-harness.mjs` passes 27/27 while `docs/trace/live/trace-capture.json` has `surfaces: []` and `controls.json` reports 49 WIRED / 54 evidence-backed; 48 hops point at that empty file; 5 W9-promoted hops have no raw capture. The validator cannot see the contradiction (it checks evidence-array non-emptiness only).
- **Fix at the boundary (this seat, docs only):** validate capture *content* before trusting it; downgrade unsupported verdicts to `UNVERIFIED` with a named `w4_correction` (previous verdict + previous evidence preserved), recompute counts from a single command, and publish a capture registry that defines the exact raw artifacts required to re-promote each family.
- **GREEN (post-fix target):** every non-PENDING hop carries either (a) a pointer to a committed raw artifact whose content was re-read this session, or (b) a `w4_correction` note that explicitly downgraded it; recomputed counts match the documented command; validator still exits 0 (shape/evidence-presence) and the docs state that capture-content validation is a residual gap in `tools/qa` (out of this seat's write scope).
- Regression bound: `apply-live-trace.mjs`/`trace-hops.spec.ts` remain unchanged; a future empty-capture run will now be visibly inconsistent because controls' `live_capture` block records the observed empty capture and zero evidence-backed hops.

## 6. Decisions

1. **Downgrade, do not delete:** capture-backed WIRED hops → `UNVERIFIED`, preserving `w4_correction.previous_verdict/previous_evidence` so history is auditable; `S22 6_service` keeps WIRED on tracked raw WS/DOM evidence.
2. **W9 promotions without raw capture are not WIRED:** S10 `2_handler` and S27 `6/7/8/9` → `UNVERIFIED` (source/test-log pointers preserved as `w4_correction` evidence).
3. **MISSING/PENDING untouched:** S15-S19 remain `MISSING`; `S-APPROVAL` remains `PENDING`; no promotion without raw capture.
4. **Counts derive from one quoted command** in `_meta.counts`, and `_meta.live_capture` now reports `surfaces 0 / status EMPTY_RAW_CAPTURE / prior 43 WIRED+5 MISSING invalidated`.
5. **Capture registry is explicit** (`docs/trace/CAPTURE-REGISTRY.md` + machine-readable `.json`): S10, S15-S19, S27, S-APPROVAL with owner, exact command recipe, artifact format/path, acceptance, and current blocker.
6. **Residual tooling gap:** `tools/qa/verify-harness.mjs` (outside scope) does not validate pointer content/hash; recorded for the Lead/tooling owner rather than edited here.

## 7. Capture registry (summary; full detail in docs/trace/CAPTURE-REGISTRY.md)

| Surface(s) | Hops to capture | Required raw artifacts | Recipe (executed by e2e/live lane, not this seat) | Acceptance |
|---|---|---|---|---|
| S10 replay | 10_render_back (+ live round-trip hops) | WS frame JSONL with UTC ms + correlation ids; DOM snapshot/animation timestamp; screenshot; `.last-run.json` with named test | Dist harness + mocked WS replay spec run, plus live server: HISTORY_REVISION_SET → HISTORY_SNAPSHOT → replay DOM | replay nodes inert, banner + revision visible, frames captured with timestamps |
| S15-S19 node cards | 1_render (cards), plus state/derived hops | DOM snapshot of cards, screenshot, WS frames RUN_CREATED/NODE_SPAWN/NODE_STATE | Mock-run or live run with ≥2 nodes; agents panel + canvas capture | one card per kind renders with state text; no fabricated data |
| S27 streaming reply | 6_service/7_store/8_provider/9_response/10_render_back | ordered REPLY_DELTA frames with seq/ts, REPLY_DONE usage payload, run row snapshot, rendered reply DOM/screenshot | Live or wiremock-driven run with capture tap; persisted run/usage row read | deltas ordered, usage matches run, no fake completion |
| S-APPROVAL | 1_render..10_render_back | APPROVAL_REQUEST frame, overlay DOM, APPROVAL_RESOLVE ACK/ERROR frames, audit row | Mock/live tool approval flow (nonce), capture overlay + resolution | overlay visible with nonce; approve/deny ACK or named refusal; no silent effect |
| S10/S22 re-promotion | — | tracked live-probe JSON + live-app DOM (already exist) | — | S22 6_service stays WIRED; re-promote only with new captures |

## 8. Validation (observed, quoted)

```
$ node tools/qa/verify-harness.mjs
... (27 checks)
checks=27 pass=27 fail=0
OK: QA harness structure is consistent with the frozen contract; no execution claims made.

$ node -e "const c=require('./docs/trace/controls.json');const h=c.controls.flatMap(x=>Object.values(x.hops));const n={};for(const v of h)n[v.verdict]=(n[v.verdict]||0)+1;console.log(JSON.stringify(n))"
{"UNVERIFIED":49,"PENDING":264,"MISSING":5,"WIRED-AND-WORKS":1}

$ node -e "... non-PENDING hops with empty evidence[] ..."
non-PENDING without evidence: 0

$ node -e "JSON.parse controls/traces/events"
ok docs/trace/controls.json
ok docs/trace/traces.json
ok docs/trace/events.json

$ git status --porcelain docs/trace plans/waves
 M docs/trace/QA-HARNESS.md / TRACE-SCHEMA.md / controls.json / events.json / live/README.md / terminal-truth.md / traces.json
?? docs/trace/CAPTURE-REGISTRY.md / docs/trace/capture-registry.json
?? plans/waves/W4-TRACE-CAPTURE.md
 M plans/waves/W9-TRACE-TRUTH.md   (W4 correction addendum appended)
```

Raw-capture state re-read for the correction: `docs/trace/live/trace-capture.json` = **286 B**, `generated_at 2026-11-06T20:25:04.044Z`, `surfaces: []` (the prior 28-surface/43-WIRED capture is gone; 48 WIRED verdicts downgraded).

## 9. Rollback

Revert touched files from HEAD `c9390f5` content (`git show c9390f5:docs/trace/...` into working file) — this seat created no commits and touched no product code.

## 10. Done / Not done / Unverified

- **Done (this seat):** raw-capture loss detected; 48 capture-backed WIRED hops downgraded with audit-preserving `w4_correction`; 5 non-capture W9 promotions downgraded; S22 6_service retained on tracked raw evidence; counts recomputed and quoted; `live_capture`/`wave2_evidence` corrected; `traces.json`/`events.json`/`terminal-truth.md` corrected notes; docs count blocks replaced; concrete capture registry added (MD+JSON) for S10/S15-S19/S27/S-APPROVAL; validator run quoted.
- **Not done (by design / out of scope):** no new raw captures executed (server not running; e2e/tools write scope excluded; paid/mock run needs a different lane); no WIRED promotion for PENDING/MISSING; no edit to `tools/qa` validator to add pointer-content/hash gate; no edit to e2e capture writer.
- **Unverified:** the historical 43-WIRED capture cannot be re-verified (file overwritten empty); S22 raw live-probe JSON was READ and its FILE_TREE_LIST roundtrip verified, but `live-probe-final.json` predates the current HEAD (`c9390f5`) — its S22 6_service WIRED verdict is conditional on that artifact being the same product behaviour (re-capture recommended); `terminal-truth.md` remains NOT_COLLECTED; events parity `delivered=[]` remains uncollected; capture commands in the registry are proposals, not executed.
