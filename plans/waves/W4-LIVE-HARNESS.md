# W4 — LIVE e2e HARNESS (plan + Phase-1 gate)

Seat: **W4 (live e2e harness)** · WRITE SCOPE: **only new files** `e2e/helpers/live-server.ts` and `e2e/specs/live-*.spec.ts`; plus this plan doc. No edits to existing e2e/config/package, no product edits, no commit (Lead), no key printing.
Checkpoint: repo `C:\Users\Riley\Desktop\hiverouter-v1` · observed HEAD `c9390f5` · debug binary `target/debug/hiverouter.exe` (45,447,523 B, mtime 2026-11-06 15:53:26) · release binary `target/release/hiverouter.exe` (13,253,120 B, 11:24:14) · `web/dist` build `index-CVJmpokg.js` (675,204 B, 14:25:25).
Skills used: `forgewright-recon`, `forgewright-lead`, `logic-gatekeeper`, `integration-logic`, `logic-debugger`.

## Recon (forgewright-recon ledger)

- **Target behavior:** a deterministic Playwright harness that boots the real Rust binary on an ephemeral loopback port + temp data dir, waits for `/health` + `/api/session`, exposes `baseURL`, and kills the process + removes the temp tree on teardown; live specs prove ordered streaming (`REPLY_DELTA…` → `REPLY_DONE` → `RUN_STATUS completed`) and live replay (`HISTORY_REVISION_SET` → `HISTORY_SNAPSHOT` with ≥2 nodes → read-only UI), gated so **no paid provider call happens unless `LIVE_ONE_PROMPT=1` and a key is set**; missing server/key ⇒ **named skip/official failure, never fake pass**.
- **Entry points / protocol (read):**
  - Config: `backend/crates/hiverouter-core/src/config.rs:251-291` (`HIVEROUTER_BIND`, `HIVEROUTER_DATA_DIR`/`--bind`/`--data-dir`/`--workspace-root`), `:199-209` (default Origin allowlist derives from bound port; empty for port 0), `:371-380` (`HIVEROUTER_ALLOWED_ORIGINS` appended).
  - Boot: `main.rs:28-160` — parses config, opens SQLite, migration, recovery, env checks, binds loopback, prints `hiverouter <ver> listening on http://<bound> (…schema v3, contract hiverouter.v1)`; key check is a **warning** (`main.rs:102-120`), so the server boots without a key; provider enabled defaults true (`config.rs:56-68`) → live prompts refuse by name (`E_KEY_MISSING`).
  - HTTP: `http.rs:274-347` (`/api/history/runs`, `/revisions`, `/state?revision=`), `http.rs:349-445` (`/health` body: status/version/schema/uptime/checks/database/transport{last_seq,graph_nodes,allowed_origins}; `/api/session` body: token/schema/last_seq/ws), `http.rs:245-253` router; SPA served from embedded `web/dist` (`static_ui.rs:1-8,41-66`).
  - WS: `ws.rs:4-8` raw `{"type":"hello","last_seq":n}` then frozen envelopes; `ws.rs:93-117` Origin + token gate before upgrade; `ws.rs:199-240` hello/SERVER_HELLO; `ws.rs:423-441` `HISTORY_REVISION_SET` → ACK + `HISTORY_SNAPSHOT{run_id,revision,graph}`; graph serializes `TaskNode{task_id,parent_task_id,state,rev}` (`core/src/graph.rs:18-33`), matching `web/src/flow/graph.ts:buildReplayTasks`.
  - Existing e2e: `e2e/playwright.config.ts` (installed Chrome/Edge channels, no download, no `webServer`, baseURL from `E2E_BASE_URL`), `e2e/run-guarded.mjs` (skip-guard: exits non-zero when **only skips ran**), `e2e/helpers/replay-harness.ts` (dist + `page.routeWebSocket` double — **not** a live server), `e2e/helpers/ws-tap.ts` (frame capture), `e2e/helpers/app.ts` (`/health`, `/api/session`, ws URL), `e2e/specs/live-app.spec.ts` (real-server selectors/frame patterns), `e2e/GATE-EXCLUSIONS.md:19` (skip-guard rule), `e2e/README.md` (live binary workflow).
- **Manual boot smoke (run, verified):** debug binary on an ephemeral port + temp data dir:
  `boot_ok=True port=62661 health.status=degraded session.schema=hiverouter.v1 session.last_seq=0 ws=/ws token_present=True`; stdout `hiverouter 1.1.0 listening on http://127.0.0.1:62661 (db …\\hiverouter.db, schema v3, contract hiverouter.v1)`; key check warning `E_KEY_MISSING` (no key present in this environment). Process killed; temp dir removed. No paid call.
- **Data flow (one path + one failure):** spawn child (env: bind/data-dir/workspace-root, inherits key only when paid gate allows) → poll `/health` (accept `ok|degraded`) → `/api/session` → page/app or WS probe → tests drive real commands/events → teardown kills child, waits exit, removes temp dir. **Failure:** binary missing / bind fails / health never ready / key missing → named error or named `test.skip`; no fabricated pass; skip-guard makes an all-skip run exit non-zero unless `E2E_ALLOW_ALL_SKIPS=1`.
- **Unknowns / evidence still missing:** no `DEEPSEEK_API_KEY` in this environment (`False`) and `LIVE_ONE_PROMPT` unset → paid streaming/replay cannot be executed here; no live server currently running on `:8799/:8080` (refused). Browser channel Chrome exists. Node has global `WebSocket` (v26) but sends **no Origin** header, so the harness's protocol probe runs inside the page context (page origin is allowlisted by the server).
- **Files I expect to touch (new only):** `e2e/helpers/live-server.ts`, `e2e/specs/live-streaming.spec.ts`, `e2e/specs/live-replay.spec.ts`, this plan doc.

## Phase-1 gate (logic-gatekeeper, required before writes)

```json
{
  "phase1": {
    "architecture": "Playwright/Node test process owns a child Rust server process (spawn) with a per-test temp data dir and a reserved ephemeral loopback port; the server owns SQLite/snapshots/WS bus; the browser page (served by that server) owns the real WS session + Origin header; a second in-page WebSocket probe drives frozen COMMAND envelopes and reads EVENT envelopes; teardown is owned by the test process (kill child -> wait exit -> rm temp dir).",
    "dataFlow": "untrusted input = process env + CLI args + test-provided prompt; harness reads DEEPSEEK_API_KEY presence only (never logs it), strips it for no-key tests, inherits it only when LIVE_ONE_PROMPT=1; server output/logs are captured, redacted, and only surfaced in failures; ports are reserved by a short-lived net server, then passed via --bind.",
    "edgeCases": [
      "No server binary: test.skip with a named reason (never pass)",
      "Port stolen between reserve and bind: bounded retry with a fresh port; fatal named error after retries",
      "Server exits during boot (bind/db/migration failure): fail with redacted stderr/stdout tail, not a timeout-only message",
      "No DEEPSEEK_API_KEY in child: SUBMIT_PROMPT must produce a typed E_KEY_MISSING failure and zero REPLY_DELTA (fail-closed, no paid call)",
      "Paid run requested without LIVE_ONE_PROMPT=1: named skip before any server start, so no provider call is possible",
      "HISTORY_REVISION_SET for an unknown run/revision: typed E_HISTORY_GAP refusal (no fabricated snapshot)",
      "Teardown while WS/page still open: child killed, temp dir removed; failures in cleanup are logged, not hidden"
    ],
    "rootCause": "The repository has deterministic mocked-transport e2e but no live-server harness that can boot the real binary on an ephemeral port with an isolated data dir and drive real WS commands under a paid-call gate; without it, streaming and replay cannot be proven live, and a naive harness could either leak the provider key into logs or skip-as-green."
  }
}
```

Phase 2 entry checks: binary existence checked before spawn; port parsed as loopback-only; data dir is a freshly created temp dir; health status must be `ok|degraded` and schema must equal `hiverouter.v1` before tests run; `withProviderKey:false` deletes the key from the child env; all captured output passes through `redactSecrets()` before being stored/thrown. Callers (existing specs/config) stay untouched.

## 1. Outcome
`npm.cmd --prefix e2e run e2e:chrome -- specs/live-*.spec.ts` boots a real server itself, proves boot/WS/no-key refusal deterministically today, and can prove live streaming + live replay when `LIVE_ONE_PROMPT=1 DEEPSEEK_API_KEY=…` is explicitly set; otherwise those tests are **named skips**, and the skip-guard prevents an all-skip green.

## 2. Current behavior (evidence)
See Recon: no live harness exists (`e2e/helpers/*` has only dist/mock harness + probe helpers); live specs require an externally started server (`E2E_APP_READY=1`) and skip otherwise (`matrix.ts appMissing/skipReason`). Manual boot smoke confirms the binary is self-contained on an ephemeral port with temp data dir and no key.

## 3. Decisions
1. **Self-owned process lifecycle** (not Playwright `webServer`): config must not be edited, and tests need a per-test temp dir/port; `node:child_process.spawn` + explicit `stop()` in `afterEach` is deterministic.
2. **Reserve-then-bind ephemeral port** (net server listen 0 → close → `--bind`), bounded retry; port 0 would leave the Origin allowlist empty (`config.rs:199-209`), so a concrete port is required for browser Origin acceptance.
3. **Key policy:** never log/echo the key; no-key tests start the child with `DEEPSEEK_API_KEY` removed (fail-closed proof); paid tests inherit it only when `LIVE_ONE_PROMPT=1`, otherwise `test.skip` with a named reason *before* spawning.
4. **Protocol probe runs in the page context** (`page.evaluate` + `WebSocket`) because the server requires an allowlisted `Origin`; Node's global WebSocket sends none (`ws.rs:93-117`). This also keeps tests at the real browser/transport boundary.
5. **No edits to `run-guarded.mjs`/`package.json`/config**; the new specs work under the existing `--project=chrome` runner and the skip-guard.
6. **Non-paid tests are executed now** (boot, no-key refusal, unknown-run history refusal); paid tests are named skips in this environment and only run on explicit env.

## 4. File-by-file change
- **NEW `e2e/helpers/live-server.ts`** — binary discovery (`E2E_SERVER_BIN` → `target/debug` → `target/release`), `paidGate()`, `serverSkipReason()`, `redactSecrets()`, `reservePort()`, `startLiveServer({withProviderKey?})` (temp data dir, child env, redacted log capture, health/session wait, embedded-UI probe), `stop()`, `openWsProbe(page)` (in-page WebSocket: hello, sendCommand, waitFor(predicate), frames, close).
- **NEW `e2e/specs/live-streaming.spec.ts`** — (a) boot/health/session/UI/WS-hello (no key, runs now); (b) no-key `SUBMIT_PROMPT` ⇒ typed `E_KEY_MISSING`, zero `REPLY_DELTA` (runs now); (c) paid-gated ordered streaming: `RUN_CREATED` → ≥1 `REPLY_DELTA` with non-empty deltas and monotonic `seq` → `REPLY_DONE` → `RUN_STATUS completed`, none out of order, no `failed`; skip named when gate closed.
- **NEW `e2e/specs/live-replay.spec.ts`** — (a) `HISTORY_REVISION_SET` unknown run ⇒ typed `E_HISTORY_GAP` (runs now, no paid); (b) paid-gated: one prompt → completed run → iterate revisions via `/api/history/runs/{id}/revisions` → `HISTORY_REVISION_SET` → `HISTORY_SNAPSHOT` with **≥2 nodes** → UI replay banner, `.react-flow__node` count, `0 × .draggable`, `0 × .selectable` (read-only).
- **NEW this doc** `plans/waves/W4-LIVE-HARNESS.md`.

## 5. Failure modes
- Missing binary → `serverSkipReason()` named skip; no spawn.
- Spawn exit/bind failure → throw with redacted stdout/stderr tail; afterEach no-ops safely.
- Health never `ok|degraded` → timeout error with last health body + redacted logs; temp dir removed.
- No key path: asserts typed `E_KEY_MISSING`/`failed`; **fails** if a `REPLY_DELTA` appears (would mean fake/live success).
- Paid gate closed: named skip; skip-guard exits non-zero when *only* skips ran (unless explicit override), so CI cannot quietly pass.
- Paid run error (provider/auth/rate/budget): test fails with captured typed ERROR code + redacted logs; no retry storm.
- Teardown failure: logged via returned stop error; test result already recorded.

## 6. Tests / commands
- Static/no-paid now: `cd e2e; .\node_modules\.bin\playwright.cmd test specs/live-streaming.spec.ts specs/live-replay.spec.ts --project=chrome --reporter=list` (expect passes + named skips).
- Compile/list: `… --list`.
- Paid (Lead-only): set `LIVE_ONE_PROMPT=1` and `DEEPSEEK_API_KEY` explicitly, same command (one short prompt per spec; ≤2 paid prompts total).
- Skip-guard behavior: the existing `run-guarded.mjs` remains unchanged and is the owner of the all-skip refusal.

## 7. Rollback
Delete the two new spec files + helper + this doc. No existing file, config, product code, DB or git history is touched; temp data dirs are per-test and removed.

## 8. Out of scope / blockers
- Out of scope: editing `e2e/playwright.config.ts`, `e2e/package.json`, `e2e/run-guarded.mjs`, existing helpers/specs, any `web/src`/`backend`/docs/audit/tools file; commits.
- Blockers for the paid proofs here: `DEEPSEEK_API_KEY` absent and `LIVE_ONE_PROMPT` unset in this environment; no live server running on `:8799/:8080`. The specs will skip with exact reasons; the non-paid live proofs run and are quoted in the final report.

## 9. Observed results (this environment — no key, no paid call)

Command: `cd e2e; .\node_modules\.bin\playwright.cmd test specs/live-streaming.spec.ts specs/live-replay.spec.ts --project=chrome --reporter=list`

```
Running 5 tests using 1 worker
  ok 1 … live-replay.spec.ts:37  HISTORY_REVISION_SET for an unknown run refuses by name, no snapshot (no paid call) (5.2s)
  -  2 … live-replay.spec.ts:59  live replay: HISTORY_REVISION_SET -> HISTORY_SNAPSHOT >=2 nodes, UI read-only (paid gate)
  ok 3 … live-streaming.spec.ts:39  boots real server: /health, /api/session, embedded UI, WS hello (no key, no paid call) (3.0s)
  ok 4 … live-streaming.spec.ts:67  SUBMIT_PROMPT without a key fails closed: typed E_KEY_MISSING, no REPLY_DELTA (no paid call) (2.1s)
  -  5 … live-streaming.spec.ts:99  live streaming: ordered REPLY_DELTA -> REPLY_DONE -> RUN_STATUS completed (paid gate)
  2 skipped · 3 passed (1.0m) · exit 0
```

Named skip reason captured from the JSON reporter: `LIVE_ONE_PROMPT is not 1: paid provider call disabled. Set LIVE_ONE_PROMPT=1 AND DEEPSEEK_API_KEY explicitly to run the live streaming/replay proof (one short prompt each).`

Skip-guard proof (only the two gated tests, via the existing unchanged wrapper):
`node run-guarded.mjs specs/live-streaming.spec.ts specs/live-replay.spec.ts --project=chrome -g "paid gate"` → `2 skipped`, **exit 1**, stderr `[skip-guard] all 2 tests were skipped and none passed; set E2E_ALLOW_ALL_SKIPS=1 only with a written reason`.

Server/teardown evidence: each non-paid test booted `target/debug/hiverouter.exe` on a reserved ephemeral loopback port with a fresh temp data dir, observed redacted stdout `listening on http://127.0.0.1:<port>`, health `degraded` (key absent, expected), `/api/session` token/schema/last_seq/ws present, embedded UI at `/` (`<div id="root">`), WS `SERVER_HELLO` received on a real socket, then `stop()` killed the child and removed the temp tree; `Get-Process hiverouter` = none afterwards. No paid call was made (`DEEPSEEK_API_KEY` absent; child env explicitly stripped for the no-key tests).

