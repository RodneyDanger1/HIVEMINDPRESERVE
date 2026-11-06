# Live trace capture — 1.1 (BLOCKED: no operator key; no fabricated trace)

Seat: **D (live trace capture)** · Date: 2026-11-06 (America/Chicago) · Capture attempt (UTC): `2026-11-06T22:46:32.102Z`
Checkpoint: HEAD `9651722a8e4b709fef09e58ae0b7804277cfdcc9`.
Machine-readable record: `docs/trace/capture-live-1.1.json`. Plan/recon/Phase-1: `plans/waves/W11-TRACE.md`.
Write scope: only these three new files. No existing registry/trace/e2e/config/product file edited. No commit.

## 1. Exact blocker (verified in this environment)

| Gate | Observed | Consequence |
|---|---|---|
| `LIVE_ONE_PROMPT` | **unset** (empty) | The paid streaming/replay specs are gated and skip; per the task, the live specs may run only when this is `1` |
| `DEEPSEEK_API_KEY` | **not present** (boolean check only; value never read) | No paid provider run is possible; the harness `paidGate()` would skip with the named reason “LIVE_ONE_PROMPT is not 1…” |
| Operator key provided to this seat | **no** | The S10 replay and S27 streaming raw captures cannot be produced here |

**No paid call was made. No raw frame, no capture surface and no trace hop was fabricated or promoted.** The existing prose live-run narrative (`docs/evidence/live-provider-smoke.md`) is **not** accepted as capture evidence by registry rule 2 (raw artifact required), so it was not used to promote anything.

Blocker code: **`E_LIVE_TRACE_BLOCKED_NO_OPERATOR_KEY`**.

## 2. Preconditions verified (so the blocker is the only reason for no capture)

- Release binary present and hashed (not executed in a paid run):
  `target/release/hiverouter.exe` — 13,624,320 B, sha256 `9C3174FB7C118143139FF283C08A3190C8E146A831717FB3FF8E25D6468D7207`, mtime 2026-11-06T16:42:37-06:00.
- Drivers present and hashed:
  - `e2e/specs/live-streaming.spec.ts` — 6,553 B, 172 lines, sha256 `90D59D2416F62730F600563F22D977E5782321F595BE2DCBB421D771815F6B3C`
  - `e2e/specs/live-replay.spec.ts` — 5,271 B, 129 lines, sha256 `AA00736300A47D6B2E829658247FAD980624A5929AB5FE42E38C6CB09EFF1079`
  - `e2e/helpers/live-server.ts` — 19,087 B, 555 lines, sha256 `4AB55C89C1326476A80A988E9DC8406FCF162D2BF936A26A2F2F56260C68D5BB`
- Spec discovery (no execution, no browser prompt):
  `playwright test specs/live-streaming.spec.ts specs/live-replay.spec.ts --list --project=chrome` → **5 tests in 2 files, exit 0** (3 streaming: boot / no-key fail-closed / paid streaming; 2 replay: unknown-run refusal / paid replay).
- Harness truth baseline (read-only recompute):
  `docs/trace/controls.json` → 29 controls, 319 hops: **`{"UNVERIFIED":49,"PENDING":264,"MISSING":5,"WIRED-AND-WORKS":1}`**
  `docs/trace/live/trace-capture.json` → `surfaces: []` (286 B, `generated_at 2026-11-06T20:25:04.044Z`).
  `docs/trace/capture-registry.json` → `CAPTURE_PLAN_ONLY`; `CAP-S10-REPLAY` and `CAP-S27-STREAMING` are `NOT_CAPTURED`/`UNVERIFIED_NO_CAPTURE`.

## 3. Harness verification (required run)

```
> node tools/qa/verify-harness.mjs
...
PASS  events.json has no fabricated deliveries
--- summary ---
checks=27 pass=27 fail=0
OK: QA harness structure is consistent with the frozen contract; no execution claims made.
(exit 0)
```

## 4. Registry reconciliation — only captured surfaces may move; none were captured

From `docs/trace/CAPTURE-REGISTRY.md` / `capture-registry.json`:

| Registry ID | Surfaces | Current verdict | Required raw artifacts | This run |
|---|---|---|---|---|
| `CAP-S10-REPLAY` | S10 (`2_handler`, `10_render_back`) | UNVERIFIED | ① WS frame JSONL `HISTORY_REVISION_SET` → ACK + `HISTORY_SNAPSHOT` with UTC/correlation ids; ② post-replay DOM (banner/revision/non-draggable+non-selectable classes); ③ timestamped PNG; ④ named test-result JSON | **BLOCKED — not captured**; no promotion |
| `CAP-S27-STREAMING` | S27 (`6_service`,`7_store`,`8_provider`,`9_response`,`10_render_back`) | UNVERIFIED | ① ordered `REPLY_DELTA` frames with seq/UTC + `REPLY_DONE` usage; ② rendered reply DOM/screenshot; ③ persisted run/usage row; ④ provider metadata without key | **BLOCKED — not captured**; no promotion |

`promotions: []`, `surfaces: []`, `captures: []`; truth baseline unchanged (**1 WIRED / 264 PENDING / 5 MISSING / 49 UNVERIFIED**). No registry or trace JSON was edited (truth files and the raw-capture directory `docs/trace/live/captures/**` are outside this seat's write scope; capture execution belongs to the e2e/QA lane per `CAPTURE-REGISTRY.md:25`).

## 5. What would unblock (exact recipe for the e2e/QA lane + operator)

1. Operator sets, in the process environment (explicit approval + budget cap; value never written/logged):
   `LIVE_ONE_PROMPT=1` and the operator key variable `DEEPSEEK_API_KEY` (value supplied by the operator, never written here).
2. Point the harness at the release binary: `E2E_SERVER_BIN=<repo>\target\release\hiverouter.exe`.
3. Run the two live specs with the WS tap / DOM evidence:
   `cd e2e; npx playwright test specs/live-streaming.spec.ts specs/live-replay.spec.ts --project=chrome --reporter=json`
   (one short prompt per paid test; the harness starts the release binary on an ephemeral loopback port + fresh temp data dir and kills it on teardown).
4. Persist raw artifacts under `docs/trace/live/captures/w11/` (WS frame JSONL with UTC ms + run id; post-replay DOM snapshot incl. banner/revision and `0 × .draggable`/`0 × .selectable`; timestamped PNG; named test-result JSON; run/revision/node count; persisted run/usage row for S27), plus sha256 manifest and the producing command line.
5. Trace seat validates those raw artifacts and promotes only the captured hops; reconcile `docs/trace/capture-live-1.1.json` (`surfaces`/`captures`/`promotions`) against the registry rows.

## 6. Non-fabrication statement

- `docs/trace/capture-live-1.1.json` records `status: BLOCKED_NO_OPERATOR_KEY`, `surfaces: []`, `captures: []`, `delivered_frames: 0`, `promotions: []`.
- No frames, timestamps, run ids, revisions, node counts or UI assertions were invented. Nothing in this file claims S10/S27 were captured.
- `node tools/qa/verify-harness.mjs` passes 27/27 with `delivered: []` — the harness itself rejects fabricated deliveries.
