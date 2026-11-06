# W5B-UI-TESTS — UI lifecycle / streaming tests (SEAT 7)

**Seat:** 7 (UI lifecycle/streaming tests) · **Date:** 2026-11-06 · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**HEAD at work start:** `c9390f5` · **Write scope:** new `web/tests/apiLifecycle.test.ts`, new `web/tests/streamingRender.test.ts`, this document. **No product edits, no existing-test edits, no commit.**
**Skills used:** `/forgewright-recon` (file/caller walk), `/forgewright-lead` (run-verified acceptance), `/logic-gatekeeper` (Phase 1 below), `/integration-logic` (factory → store call chain), `/logic-debugger` (streaming state-machine edges).

## 0. Recon (READ, `path:line`)

| File | Facts |
|---|---|
| `web/src/api/index.ts:40-159` | `DevAwareApi` lazy wrapper: `kind` compile-time (`USE_DEV_MOCK`), `ensure()` creates `WsApiClient` (or dynamic mock) on first use, `flushSubscriptions()` replays queued listeners, and `if (this.closed) api.close()` after creation. `close()` (103-106) only sets `closed=true` and closes the delegate **if it exists** — close-before-open therefore depends entirely on the post-create `if (this.closed)` branch. No test exercised this branch. |
| `web/src/api/http.ts:35-104` | `fetchWithBoundTimeout`: `Promise.race([fetch, timeout, callerAbort])`, timeout default `DEFAULT_HTTP_TIMEOUT_MS=8000`, external signal handled by a rejecting `callerAbort` promise (works even if fetch ignores the signal); all failures are `RefusalError` with frozen code `E_WS_DISCONNECTED` + explicit message. |
| `web/src/api/health.ts:36-62` | `fetchHealth` uses the bounded helper with label `GET /health`; invalid JSON → `E_SCHEMA_INVALID`. |
| `web/src/state/store.ts:236-288` | `RUN_CREATED` sets run; `REPLY_DELTA` appends to the last **assistant, same-run, streaming** message else **pushes** a new message; sets `streamingRunId`; `REPLY_DONE` maps assistant messages of that run to `streaming:false` and clears `streamingRunId`. No dedupe by `seq`; ordered accumulation is the only guarantee. |
| `web/src/state/store.ts:470-499` | `init()` subscribes **before** `api.open()`; `dispose()` unsubscribes and closes. |
| `web/src/panels/SettingsPanel.tsx:16-24,82-96` | Mount effect calls `probeSettings()` + `probeHealth()` once; renders `settings.refusal` code/message/how_to_fix and `settings.health`. No DOM test environment is configured (`vite.config.ts` → `test.environment: 'node'`, no Testing Library), so panel rendering is out of scope for this seat; the store paths it consumes are covered. |
| `web/tests/store.test.ts:7-80` | Existing helper pattern: local `FakeUiApi implements UiApi` + `connectedStore()`; no exported helpers, so the new streaming file reuses the **same pattern** (not imports) to stay within "no existing-test edits". |

## 1. PHASE 1 (logic-gatekeeper — written before any test file)

```json
{
  "phase1": {
    "architecture": "Three state holders: (a) DevAwareApi in web/src/api/index.ts owns delegate resolution, the closed flag and queued listener sets; (b) fetchWithBoundTimeout in web/src/api/http.ts owns the timeout/abort race and refusal mapping; (c) the zustand UiState messages/streamingRunId in web/src/state/store.ts owns streamed text accumulation. Tests only observe these holders; no product state is written.",
    "dataFlow": "Untrusted input enters as (1) a fetch implementation/signal that may never resolve or abort late, (2) a WS/event sequence REPLY_DELTA*→REPLY_DONE whose text_delta strings are concatenated, and (3) user lifecycle calls close()/open() in either order. The tests drive these entries and assert on returned refusals / store snapshots.",
    "edgeCases": [
      "close() called BEFORE the first open()/ensure(): delegate is created later; queued subscription must be flushed exactly once and the deferred close must run BEFORE the delegate's open() (no leaked live socket).",
      "Caller aborts a bounded request while the fetch implementation ignores the abort signal: must reject with a named E_WS_DISCONNECTED refusal 'aborted by the caller' (not hang until timeout).",
      "Signal is already aborted before the call: must refuse immediately.",
      "REPLY_DONE arrives before any REPLY_DELTA: no ghost assistant message; streamingRunId stays null.",
      "REPLY_DELTA for a second run after the first REPLY_DONE: must create a separate assistant message, never append to or duplicate the finished run's text.",
      "Duplicate REPLY_DONE: idempotent — text unchanged, single assistant message, streamingRunId null.",
      "Production entry statically importing ./mock: tests read raw sources and assert no static import/re-export and a single dev-gated dynamic import."
    ],
    "rootCause": "The current suite never observes DevAwareApi's deferred-close branch, the shared HTTP bound's caller-abort branch, or the ordered REPLY_DELTA→REPLY_DONE concatenation invariant, so a regression that leaks a socket, hangs on abort, or duplicates streamed text would still pass every existing test."
  }
}
```

## 2. Test design (new files only)

`web/tests/apiLifecycle.test.ts`
1. **close-before-open** — `vi.mock('../src/api/client')` replaces `WsApiClient` with a recorder (via `vi.hoisted`); `createUiApi()` → subscribe → `close()` → `open()`; assert call order `construct → subscribe → close → open` (deferred close precedes open), subscribe flushed once, unsubscribe forwarded.
2. **open-without-close** — no spurious close; listener attached once; `kind` stays `'ws'`.
3. **timeout refusal** — never-resolving fetch + `timeoutMs: 20` → `RefusalError` code `E_WS_DISCONNECTED`, label + "timed out after 20 ms", non-empty `how_to_fix`, bounded wall time.
4. **caller-abort refusal** — in-flight abort and pre-aborted signal both reject with named `E_WS_DISCONNECTED` "aborted by the caller"; fetch receives an `AbortSignal`.
5. **network-failure naming** — rejected fetch surfaces label + cause, not a bare Error.
6. **no static mock import** — scan raw `src/api/*.ts` + `src/App.tsx`: zero static `./mock` imports/re-exports, exactly one dynamic `import('./mock')` inside the dev flag; mock guard requires `DEV && VITE_UI_MOCK`; `createUiApi()` resolves real client.

`web/tests/streamingRender.test.ts`
1. **ordered REPLY_DELTA → REPLY_DONE** — three deltas concatenate into exactly one assistant message; `streamingRunId` tracks the run; REPLY_DONE flips `streaming:false`, keeps text byte-identical, clears id; user message untouched; assistant count stays 1 (no duplicate text).
2. **REPLY_DONE with no deltas** — no ghost message.
3. **cross-run isolation** — a second run's deltas create a second message; first run's text unchanged.
4. **duplicate REPLY_DONE idempotent** — text and counts unchanged.
5. **empty final delta** — zero-length delta does not duplicate or reset accumulated text.

## 3. Execution evidence

**Files created (only writes outside this doc):** `web/tests/apiLifecycle.test.ts` (227 lines, 13 tests), `web/tests/streamingRender.test.ts` (179 lines, 5 tests). **No `web/src/**` edits; no existing-test edits; no commit** (`HEAD c9390f5` unchanged; `git status --porcelain -- web/src` empty).

**RED first (proves the tests are discriminating, not tautologies):**

```
 ❯ tests/apiLifecycle.test.ts (13 tests | 1 failed) 237ms
   × keeps the app entry chain free of direct mock imports 14ms
AssertionError: expected 'import { StrictMode } from 'react';…' to contain 'createUiApi'
 Test Files  1 failed | 14 passed (15)
      Tests  1 failed | 92 passed (93)
```

Fix: the factory lives in `App.tsx`, not `main.tsx` (recon correction); the assertion now checks no mock import in both entry files and `createUiApi` in `App.tsx`.

**GREEN (quoted verbatim):**

```
$ npm.cmd test -- --run
 ✓ tests/apiClient.test.ts (7 tests) 178ms
 ✓ tests/health.test.ts (5 tests) 112ms
 ✓ tests/apiLifecycle.test.ts (13 tests) 133ms
 ✓ tests/streamingRender.test.ts (5 tests) 29ms
 ✓ tests/store.test.ts (8 tests) 30ms
 ✓ tests/graph.replay.test.ts (7 tests) 75ms
 ✓ tests/contract.test.ts (8 tests) 24ms
 ✓ tests/tokens.test.ts (6 tests) 17ms
 ✓ tests/agentState.test.ts (5 tests) 14ms
 ✓ tests/apiBoundary.test.ts (5 tests) 18ms
 ✓ tests/state.parity.test.ts (8 tests) 16ms
 ✓ tests/noHardcoded.test.ts (4 tests) 12ms
 ✓ tests/refusalParity.test.ts (2 tests) 9ms
 ✓ tests/keyboard.test.ts (7 tests) 13ms
 ✓ tests/metrics.test.ts (3 tests) 10ms

 Test Files  15 passed (15)
      Tests  93 passed (93)
   Duration  16.74s
```

```
$ npm.cmd run typecheck
> tsc --noEmit
typecheck exit: 0
```

Baseline before this seat: 75 tests / 13 files → **+18 tests / +2 files, 93/93 passing**.

**Coverage of the requested cases:** close-before-open (deferred `close` observed strictly before `open`: `['construct','subscribe','close','open']`; queued subscription flushed exactly once; unsubscribe forwarded) · timeout refusal (hung fetch → `E_WS_DISCONNECTED` "timed out after 20 ms", bounded wall time) · abort refusal (in-flight + pre-aborted signal → "aborted by the caller"; `AbortSignal` passed to fetch) · no static mock import (raw-source scan of `src/api/**`, single dev-gated dynamic import, app entry chain, constructor guard) · ordered `REPLY_DELTA` → `REPLY_DONE` (one assistant message, concatenated text, finalized once, no duplicate occurrence, no ghost/empty-message, cross-run isolation, idempotent duplicate DONE).

**Logic-gatekeeper Phase 2 (why the constraints now fail closed):** entry checks are at the boundary — `fetchWithBoundTimeout` rejects named refusals for timeout/abort/network before any caller can treat the result as success; `DevAwareApi` guarantees a deferred close is applied before the delegate opens, so a close-before-open cannot leak a live socket; store invariants pin the ordered-delta contract. Callers stay stable: `WsApiClientOptions` gained only an optional `httpTimeoutMs`, `fetchHealth()` keeps its no-arg call site, `createUiApi()` keeps its synchronous signature.

**Skill usage:** `/forgewright-recon` (§0 file/caller walk), `/forgewright-lead` (run-verified acceptance, quoted outputs), `/logic-gatekeeper` (Phase 1 before writes, Phase 2 above), `/integration-logic` (factory → store → panel chain and boundary refusals), `/logic-debugger` (streamed-state edge cases: ghost message, cross-run mixing, duplicate DONE, empty delta).

## 3.1 UNVERIFIED / not covered

- React `SettingsPanel` render assertions: no DOM test environment (README: node env, no jsdom/Testing Library) — render layer UNVERIFIED; the store mutations the panel consumes (`probeSettings`/`probeHealth`) are covered by existing tests + lifecycle tests here.
- Real WebSocket/network behaviour under a live server (no server started; out of this seat's scope).
- `dev`-mode mock path (VITE_UI_MOCK=1) not executed end-to-end (compile-time flag cannot be toggled per-test); the guard, dynamic-import boundary and constructor refusal are asserted.

## 4. Boundaries

- No `web/src/**` edits, no existing-test edits, no commit.
- `SettingsPanel` render tests are **not possible** in this repo's current test environment (node, no DOM/Testing Library, no jsdom dependency) — recorded as UNVERIFIED for panel rendering; the store paths it calls are covered here and in existing tests.
