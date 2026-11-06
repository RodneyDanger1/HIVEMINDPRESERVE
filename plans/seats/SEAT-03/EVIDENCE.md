# EVIDENCE — SEAT S3 (UI-State/Replay)

Read-only seat. **No product/git/config edits, no rebuilds, no keys, no paid calls, no test execution.** Every output written only under `plans/seats/SEAT-03/`. Evidence below is quoted command output or `path:line` from files read in this session (READLOG has per-file bytes/sha256/lines).

## 1. Environment / quiescence

```
> git -C <repo> rev-parse HEAD
5c69b66e363e9f1d0801ea5c2e66a1fc12c8943c
> git -C <repo> status --short -- web
 M web/package.json
```

- HEAD `5c69b66` = “feat: wire DeepSeek provider into core; live smoke, replay arm, 16-state parity, settings”.
- **Tree is not fully quiescent**: `web/package.json` is modified vs HEAD (observed `version 0.1.0→1.0.0` + UTF-8 BOM + an em-dash re-encoded in `description`; it is **outside this seat's assigned slice** and was not touched). Every file in the assigned product slice matches HEAD content (hashes in READLOG §B).
- The HEAD commit message claims the replay arm / 16-state parity / settings work; this seat verifies the source, not the commit claim.

## 2. Static verification commands and observed output

**16-state parity:** READLOG rows 10-11 + shipped-bundle probe below. Enum is 16 values (`agentState.ts:9-26`), labels are 16 (`strings.ts:166-183`), `interrupted` present in both; diagram inventory (16, five terminal) matches.

**Shipped production bundle probe (read-only, `Select-String -AllMatches`):**

```
dist/assets/index-DzPE48Wh.js  bytes=672230  mtime=11/6/2026 10:58:47 AM
sha256=1A0BBFF38DD37526E53018922E55782D1D1C17546481930F0CE27494E6CBDCFF
dist/assets/index-BWOl__uM.css bytes=35753  sha256=96FFA8737F8A6123F364EB2E4F7082D448B470C85FBC7E1A2656A2629443A748

pattern 'dev-mock-session-token' occurrences: 0
pattern 'MockApiClient is dev-only' occurrences: 0
pattern 'DEV MOCK' occurrences: 1        (the STRINGS.connection.mock label, strings.ts:54)
pattern 'interrupted' occurrences: 5
pattern 'HISTORY_SNAPSHOT' occurrences: 4
```

Interpretation: **mock module is excluded from this shipped bundle** (no mock-only markers); 16-state vocabulary and replay event handling are present in shipped code. This is bundle-level evidence for the checked artifact only.

**Tests/commands cited (not re-run):**

```
docs/reports/FINAL-REPORT.md:37  | Frontend unit tests | 46 passed / 8 files, typecheck + build exit 0 | web/ (vitest) |
docs/reports/FINAL-REPORT.md:72  cd ..\web; npm.cmd ci; npm.cmd run typecheck; npm.cmd test; npm.cmd run build
```

Source `it(` count across `web/tests/*.ts` = **46** (8 files), consistent with the log; hash list for all 8 test files is in READLOG §C.

**Replay/read-only source evidence (path:line):**
- `web/src/flow/Canvas.tsx:113-136` — replay tasks/nodes/edges computed from `history.replayGraph?.graph` and passed to ReactFlow.
- `web/src/flow/Canvas.tsx:146` — `onNodeClick` always calls live `handleSelect` (also during replay).
- `web/src/flow/graph.ts:64-72` — `buildNodes` sets `draggable: true, selectable: true` for replay nodes too.
- `web/src/flow/graph.ts:93-131` — `buildReplayTasks` accepts `unknown`, silently returns `{}` on malformed input.
- `web/src/api/contract.ts:235-239` — `HistorySnapshotPayload.graph: unknown` (no frozen shape).
- `web/src/state/store.ts:385-393` — `HISTORY_SNAPSHOT` stores the graph but the store does no validation.

**Settings model/thinking/effort/key evidence:**
- `web/src/api/contract.ts:242-248` — result payload has required provider/model/key_present, optional thinking/reasoning_effort.
- `web/src/state/store.ts:78-80, 346-361` — SettingsState carries thinking/reasoningEffort; event handler maps optional fields to null-safe values.
- `web/src/state/store.ts:662-684` — one SETTINGS_STATUS per open (pending guard) + failure refusal attributed to a *new* toast only.
- `web/src/panels/SettingsPanel.tsx:65-78` — renders provider/model/thinking/effort/key.
- `web/src/tests/store.test.ts` equivalent (`web/tests/store.test.ts:112-137`) — single-command guard test.

**HTTP timeout evidence:**
- `web/src/api/client.ts:235-238` — `fetch('/api/session')` with no `AbortSignal`/timeout; `waitForOpen` timeout (line 389-419) only starts after the fetch resolves and a transport exists (line 282).
- `web/src/api/client.ts:443-452` — WS command ACK timeout exists (default 15,000 ms).
- `web/src/api/health.ts:18-21` — `/health` fetch with no timeout/abort.

**Failed-terminal parity evidence:**
- `web/src/state/agentState.ts:111-118` — `isTerminal` returns true only for `completed|cancelled|killed|interrupted`; `failed` absent.
- `audit/diagrams/04-state-inventory.mmd:5` — declared terminal set includes **`failed`** (five total).
- `web/src/state/agentState.ts:120-122` + `web/src/state/store.ts:434-441, 550-557` — `isRunActive('failed')` is true, so spawn/rewire/cancel local gates treat a failed run as active.
- `web/src/state/store.ts:254` — streaming indicator cleared only when `isTerminal(status)`, so `failed` leaves stale streaming state.

## 3. Gate cycles (mandated order)

`GATE-CY-1.md`, `GATE-CY-2.md`, `GATE-CY-3.md` each record **Recon → Plan → Lead decision** in that order.
- CY-1: PASS (inputs/read-only guard).
- CY-2: PASS-WITH-NOTES (static verification approved; runtime delegated).
- CY-3: PASS-WITH-NOTES (final findings/UNVERIFIED accepted). **No cycle failed, so the seat is not BLOCKED.**

## 4. Constraints honoured / not honoured

Honoured: read-only repo operations; no browser; no key access; no paid call; no test/build re-run; outputs only in `plans/seats/SEAT-03/`.
Not possible under the mandate: runtime replay e2e, live Settings probe against a key, production-bundle rebuild comparison, test execution — all recorded as **UNVERIFIED** in `REPORT.md §6`.

Seat directory (uncommitted, as required for a read-only seat): `?? plans/seats/SEAT-03/` containing `REPORT.md`, `READLOG.md`, `EVIDENCE.md`, `GATE-CY-1.md`, `GATE-CY-2.md`, `GATE-CY-3.md`.
