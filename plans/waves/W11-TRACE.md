# W11 — LIVE TRACE CAPTURE (S10 replay / S27 streaming)

Seat: **D (live trace capture)** · WRITE SCOPE: new `docs/evidence/live-trace-1.1.md`, `docs/trace/capture-live-1.1.json`, `plans/waves/W11-TRACE.md` only; READ-ONLY elsewhere. No commit (Lead). No key material may be written or logged.
Checkpoint: HEAD `9651722a8e4b709fef09e58ae0b7804277cfdcc9`; release binary `target/release/hiverouter.exe` 13,624,320 B, sha256 `9C3174FB7C118143139FF283C08A3190C8E146A831717FB3FF8E25D6468D7207`; UTC capture attempt `2026-11-06T22:46:32.102Z`.
Skills applied: `forgewright-recon`, `forgewright-lead`, `logic-gatekeeper`, `integration-logic`, `logic-debugger`.

## Recon (forgewright-recon ledger)

- **Target behavior:** with explicit operator approval (`LIVE_ONE_PROMPT=1` + `DEEPSEEK_API_KEY`), run the existing live Playwright specs against the **release** binary and capture raw evidence for S27 streaming and S10 replay (event order/UTC timestamps/run id/revision/node count, UI read-only assertions) into `docs/evidence/live-trace-1.1.md` + `docs/trace/capture-live-1.1.json`, then reconcile only the surfaces actually captured with `docs/trace/capture-registry.json` / `controls.json`.
- **Entry points:** `e2e/specs/live-streaming.spec.ts` (sha `90D59D24…`, 5 tests: boot, no-key fail-closed, paid streaming), `e2e/specs/live-replay.spec.ts` (sha `AA007363…`, paid replay + no-paid unknown-run refusal), harness `e2e/helpers/live-server.ts` (sha `4AB55C89…`; `E2E_SERVER_BIN` override, ephemeral port/temp dir, redacted logs, paid gate requiring `LIVE_ONE_PROMPT=1` AND key).
- **Truth baseline (read):** `docs/trace/controls.json` = 29 controls / 319 hops → **1 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 49 UNVERIFIED**; `docs/trace/live/trace-capture.json` = `surfaces: []` (286 B, `generated_at 2026-11-06T20:25:04.044Z`); `docs/trace/capture-registry.json` = `CAPTURE_PLAN_ONLY`, rows `CAP-S10-REPLAY` (UNVERIFIED) and `CAP-S27-STREAMING` (UNVERIFIED) both **NOT_CAPTURED**; `traces.json` has no runs; `events.json` has no delivered frames.
- **Registry rules read:** raw artifact must be committed under `docs/trace/live/captures/<wave>/` with UTC timestamps + producing command; a verdict may move to WIRED only when a raw artifact supports that specific hop (prose/source/test-summary alone does not qualify); **no paid call without explicit operator approval and budget cap**; capture execution belongs to the e2e/QA lane, the trace seat validates (`CAPTURE-REGISTRY.md:9-14,25-27`).
- **Data flow:** operator env gate → harness boots release binary on ephemeral loopback port + temp data dir → live Playwright spec drives one short prompt / `HISTORY_REVISION_SET` → WS frames + DOM needed for S27/S10 traces → evidence files + capture JSON; registry promotion only where raw artifacts exist. **Failure path:** gate closed → specs skip with named reasons; no raw capture exists; **no promotion and no fabricated frame is written**.
- **Unknowns / blocker (verified this seat):** `DEEPSEEK_API_KEY` is **not present** in this process environment and `LIVE_ONE_PROMPT` is **unset**; the operator key was never provided to this seat. Therefore the paid specs cannot run (the harness gate would skip them) and the required raw S10/S27 artifacts cannot be produced. This is a hard blocker, not a test failure to paper over.
- **Files I expect to touch (new only):** `plans/waves/W11-TRACE.md`, `docs/evidence/live-trace-1.1.md`, `docs/trace/capture-live-1.1.json`.

## Phase-1 gate (logic-gatekeeper, before any write)

```json
{
  "phase1": {
    "architecture": "Read-only trace seat: evidence + capture record files are owned by this seat; raw capture execution + artifacts belong to the e2e/QA lane under docs/trace/live/captures/ (not writable in this scope), and verdict promotion is a registry operation backed only by committed raw artifacts. The release binary is the system under test; the live Playwright specs are the drivers; controls.json/capture-registry.json are the truth baseline.",
    "dataFlow": "Operator env (LIVE_ONE_PROMPT=1 + DEEPSEEK_API_KEY) -> harness gate -> release binary on ephemeral port/temp data dir -> one short SUBMIT_PROMPT / HISTORY_REVISION_SET -> raw WS frames + DOM assertions -> capture evidence + registry reconciliation. Without the gate, the flow stops at the gate and the deliverable is a named blocker; no event/frame/hop is invented.",
    "edgeCases": [
      "No operator key / LIVE_ONE_PROMPT unset: deliver exact blocker + zero promotions (this case)",
      "Paid call attempted without approval: forbidden by CAPTURE-REGISTRY rule 4 and by harness gate",
      "Prose evidence exists (docs/evidence/live-provider-smoke.md): registry rule 2 forbids promotion without raw artifact; do not reuse it as capture",
      "Release binary present but no capture run: record binary sha/mtime, do NOT claim it was exercised by a paid run",
      "Existing trace-capture.json is empty (surfaces:[]): do not resurrect old counts or fabricate surfaces",
      "Trace seat cannot write docs/trace/live/captures/: record required-artifact paths; e2e/QA lane owns creation"
    ],
    "rootCause": "The trace-capture task requires an operator-approved paid provider run to produce raw S10/S27 artifacts, but this seat's process has neither DEEPSEEK_API_KEY nor LIVE_ONE_PROMPT set, and prose/source pointers are explicitly insufficient for registry promotion; therefore the only truthful deliverable is a blocked capture record with zero promotions and an exact unblock recipe."
  }
}
```

## 1. Outcome
Either raw S10/S27 captures are captured and reconciled, or — as in this environment — a machine-readable blocked capture record and evidence file state the exact blocker, the verified preconditions, the required artifacts, and the unblock command, with **zero fabricated frames, surfaces or promotions**.

## 2. Current behavior (evidence)
See Recon: truth baseline 1/264/5/49; raw capture empty; registry rows S10/S27 NOT_CAPTURED; release binary present (sha `9C3174FB…`) but no paid run possible here (`DEEPSEEK_API_KEY` absent, `LIVE_ONE_PROMPT` unset). `node tools/qa/verify-harness.mjs` → **checks=27 pass=27 fail=0, exit 0** (no fabricated deliveries). Live specs list 5 tests (2 paid-gated) without execution.

## 3. Decisions
1. **Do not run the live specs here** — their paid tests are gated by `LIVE_ONE_PROMPT`; the task allows the run only with the gate, and running only the non-paid portions would not capture S10/S27. Only `--list` (discovery, no execution, no browser prompt) was run.
2. **Do not promote or reuse prose** — `docs/evidence/live-provider-smoke.md` and `docs/RELEASE-EVIDENCE.md` are prose; registry rule 2 forbids promoting hops from them. No existing registry/trace files are edited.
3. **Capture JSON is an honest blocked record**, not an empty pass: `surfaces: []`, `captures: []`, `promotions: []`, with `registry_reconciliation` for CAP-S10-REPLAY / CAP-S27-STREAMING and a named blocker code.
4. **Raw artifacts remain e2e/QA lane output** under `docs/trace/live/captures/w11/` (outside this seat's write scope). Evidence doc lists the exact required artifacts/acceptance from the registry.
5. **No key handling:** only boolean presence checks; no key read, print, write, or log.

## 4. File-by-file change
- **NEW `docs/trace/capture-live-1.1.json`** — capture record: schema `hiverouter.trace.live-capture/v1`, generated_at UTC, head/binary/specs metadata, env gate booleans (no key material), `status: BLOCKED_NO_OPERATOR_KEY`, `surfaces: []`, `captures: []`, registry reconciliation for S10/S27 (`captured:false`, `promoted:false`, required artifacts lists), truth baseline before/after (unchanged), commands run + outputs, blocker object with code/message/how_to_fix, fabrication guard (`delivered: []`).
- **NEW `docs/evidence/live-trace-1.1.md`** — human evidence: exact blocker, env booleans, release binary hash, spec/harness hashes, commands+quoted outputs (`verify-harness` 27/27, counts, `--list`), registry requirements/acceptance for S10/S27, what would unblock, explicit non-fabrication statement, write-scope statement.
- **NEW `plans/waves/W11-TRACE.md`** — this plan/recon/Phase-1 record.

## 5. Failure modes
- No key/gate → named blocker + zero promotion (handled).
- Specs unavailable/changed → hashes recorded; discovery still lists 5 tests (verified).
- verify-harness regression → would fail the seat (it passed 27/27 here).
- Future capture run partially succeeds (e.g., streaming captured, replay blocked) → capture JSON must list per-surface status separately; no all-or-nothing promotion.
- Operator key leaked into evidence → forbidden; evidence writes only boolean presence.

## 6. Tests / commands
- `node tools/qa/verify-harness.mjs` → **27/27 pass, exit 0** (run, quoted).
- Read-only truth baseline recompute → `{"UNVERIFIED":49,"PENDING":264,"MISSING":5,"WIRED-AND-WORKS":1}`, 29 controls.
- `playwright test specs/live-streaming.spec.ts specs/live-replay.spec.ts --list --project=chrome` → 5 tests discovered (no execution, exit 0).
- Unblock command for the e2e/QA lane (after operator approval + budget): set `LIVE_ONE_PROMPT=1` and `DEEPSEEK_API_KEY` in the process env, `E2E_SERVER_BIN=<repo>\target\release\hiverouter.exe`, run the two specs with `--project=chrome --reporter=json`, persist WS frames/DOM/PNG under `docs/trace/live/captures/w11/`, then this trace seat validates and promotes only captured hops.

## 7. Rollback
Delete the three new files. No existing file is modified, so there is nothing else to revert; no product/backend/e2e/config/git writes.

## 8. Out of scope / blockers
- Out of scope: `e2e/**`, `web/**`, `backend/**`, existing trace/registry files, operator key provision, paid calls, commits, raw-artifact directory `docs/trace/live/captures/**` (owned by e2e/QA lane per `CAPTURE-REGISTRY.md:25`).
- **Blocker (this environment):** `DEEPSEEK_API_KEY` absent and `LIVE_ONE_PROMPT` unset ⇒ paid streaming/replay specs cannot execute ⇒ no raw S10/S27 capture ⇒ no trace promotion. Exact code: `E_LIVE_TRACE_BLOCKED_NO_OPERATOR_KEY`.
