# 03 — UI & action audit: provider surfaces, prompt/stream, controls, responsive, a11y

**Status:** COMPLETE for the audited build (build A). Read-only pass: no product source was edited. Only this file and screenshots under `docs/trace/provider/shots/` were written. A **post-audit build B smoke** is appended in §1.1; it changes three findings and is labelled separately, not merged into the build-A evidence.
**Audit date:** 2026-11-07 18:08–18:27 America/Chicago (2026-11-08 00:08–00:27 UTC).
**Repository:** `hiverouter-v1`.
**Companion docs:** `01-deepseek-capability-matrix.md` (backend/provider truth), `02-agent-runtime-wiring.md` (prompt → planner → spawn → tool loop), `05-figcraft-ux-gaps.md` (visual gaps). This document does not repeat their backend analysis; it audits what the live UI shows and what its controls actually do.

> **No pixel-parity claim.** The four FigCraft boards (`hivemind-spatial-canvas.png`, `hivemind-controls-and-hud.png`, `hivemind-agent-cards.png`, `hivemind-run-detail-and-files.png`) were used only as the expectation baseline for which surfaces should exist. No pixel diff, image similarity, or visual-regression score was computed or is claimed.

---

## 0. One-screen verdict

The provider **read path is real**: the live app showed the configured provider/model, live account balance, the provider catalogue, and persisted per-run token usage, all from real localhost endpoints backed by the DeepSeek key.

The provider **control path does not exist** on build A: there is no model selector, no thinking/effort control, no refresh on Models, no usage history, no stop-in-chat, no retry-run, and no balance timestamp/availability. The only two provider controls on screen are Settings' `Refresh` and `Re-check health`, both 26 px tall. Build B (§1.1) adds a chat Stop button and per-model `WIRE-SUPPORTED` badges, but not a selector or a Models refresh.

On the audited build A, three visible states are not true when they are shown (build B's smoke did not re-test the first two; its source around the Models loading render and the `REPLY_DONE` state handling was not changed in the areas concerned):

1. **Models says "No balance rows reported." and shows an empty Available Models list while its fetches are still in flight** (observed; about 0.6–1 s after opening). It is a false empty state, not a loading state.
2. **Cancelling a streaming run leaves the assistant bubble labelled `· Loading` with a streaming caret forever.** The run is terminal (`RUN_STATUS cancelled`); the bubble is not.
3. **History-replayed nodes are all classified `WorkerAgent`.** The snapshot carries no kind, and the store asserts `kind: 'worker'` "none is invented" (`store.ts:615-617`), so a root node can be shown as a worker and the HUD can report `0 SUPERAGENTS · 2 WORKERS` from an invented classification.

A fourth gap is silence rather than a lie: **a cancelled run's spend is invisible.** `/api/usage/runs/{cancelled}` returned `usage: null`; Models renders no usage block at all (not even "Not reported"), even though the run had generated a partial reply.

| # | Severity | Verdict | Finding |
|---|---|---|---|
| F1 | **High** | **LYING** | History graph asserts every node is `worker`; observed root displayed as `WorkerAgent`, HUD `0 SUPERAGENTS · 2 WORKERS`. Build B fixes only the duplicate label; the asserted kind and HUD count remain. |
| F2 | **High** | **LYING** | Models shows `No balance rows reported.` + empty model list during loading; no spinner. |
| F3 | **High** | **LYING** | Mid-stream Cancel leaves the assistant message `· Loading` with caret after the run is terminal. |
| F4 | **High** | **MISSING-UI** | No model/provider selection anywhere; catalogue is read-only. Configured model is fixed to `deepseek-flash`. |
| F5 | **Medium-High** | **MISSING-UI** | Usage is current-run only and disappears when `usage === null`; cancelled-run spend invisible; no usage history/cost/timestamp. |
| F6 | **Medium** | **MISSING-UI** (latent **LYING**) | Balance ignores `is_available`, `granted_balance`, `topped_up_balance`, and fetch time; no refresh. |
| F7 | **Medium** | **MISSING-UI** | Provider fetch errors are raw `PROVIDER METADATA HTTP 503/503`; no named code, `how_to_fix`, retry, or `role=alert`. |
| F8 | **Medium** | **WIRED-BUT-DEAD** (build A) | No `TOOL_CALL`/`TOOL_RESULT`/`APPROVAL_REQUEST` can occur; tool activity and approvals cannot be reached. Build B makes the tool path reachable (§1.1); approvals remain unexercised. |
| F9 | **Medium** | **LYING** (soft) | HUD shows terminal-run percentages as throughput with no textual `derived`/`server` source and no run-status context. |
| F10 | **Low-Med** | **MISSING-UI** (build A) | Chat has no stop control on build A; ControlBar Cancel is the only stop, far from the composer. No retry-run control anywhere. Build B adds a 32×32 chat Stop button. |
| F11 | **Low-Med** | **MISSING-UI** | No provider metadata loading/empty states; no "usage not reported" row when usage is null. |
| F12 | **Low-Med** | **Accessibility** | Provider controls are 26 px high; provider errors are not announced; provider lists have no group labels; duplicate agent names in the audited build. |
| F13 | **Low** | **WIRED-AND-WORKS** | Responsive 1440/1280/1024: no control clipping or overlap; minimap intentionally hidden at 1024; control bar wraps to 2 rows with a 22 px gap to chat. |

---

## 1. What the audited build actually is

| Fact | Evidence |
|---|---|
| Live URL | `http://127.0.0.1:8099/` (portable, keyed local server) |
| Server | `C:\Users\Riley\Desktop\HiveMind-Portable\hiverouter.exe`, 15,091,200 bytes, started **18:18:02** (PID 2744) |
| Launcher | `HiveMind.exe` 8,704 bytes, started **18:17:56** (PID 5432) |
| Health | `GET /health` 200 at 18:18:13: `version 1.2.0-dev`, `schema hiverouter.v1`, `uptime_secs 10`, `deepseek_api_key: ok`, `database.schema_version 10` |
| UI bundle | `index-B_KOpz15.js` 756,490 bytes + `index-Br9J9AfU.css` 59,534 bytes, built **18:12:00** |
| Data/key | `%LOCALAPPDATA%\HiveMind\hiverouter.db` (+ 494 KB WAL) and `provider.key` (DPAPI, 262 bytes); key never appears in the UI or responses |
| Provider | `SETTINGS_STATUS_RESULT` → `{provider:"deepseek", model:"deepseek-flash", key_present:true, thinking:"enabled", reasoning_effort:"high"}` |
| Browser | installed Chrome 156.0.8078.75 via Playwright, headless, no `--virtual-time-budget` |

**Build-motion warning.** The app went down twice during the audit, both times immediately before/while the source tree was rebuilt (crash/exit at ~18:11; `cargo build --release` at 18:12 and again at 18:28). The parent relaunched the portable app at **18:17:56**; all screenshots in §8 except the two `buildB` files are from that instance (`index-B_KOpz15.js`). The working tree changed *mid-audit but outside the audited binary*: chat Stop control (`ChatCore.tsx`, **18:26:10**), model `WIRE-SUPPORTED` badges (`ModelsPanel.tsx`, **18:19:56**), label-only history fix (`store.ts`, **18:27:44**), and the `02` tools fix (**18:19:33**, not in the 18:18 binary). A new build was relaunched at **18:33:37** (`index-CE7QVIAu.js`); it is smoke-checked in §1.1 and is **not** the basis of the findings below.

### 1.1 Post-audit build B smoke (look-ahead, 18:33:37) — not part of the evidence set

After the 18:28 rebuild, the app came back on `index-CE7QVIAu.js` (757,557 B, built 18:28:25), server PID 15572 started 18:33:37. A no-cost reload (no prompt submitted by this audit) showed:

| Change vs build A | Build B observation | Effect on findings |
|---|---|---|
| History labels | Unique `Agent d53c65f4` … `Agent 18c20b01` (8 tasks), Target options unique | Duplicate-label part of F1 **fixed**; kind is still asserted `worker` and HUD still `0 SUPERAGENTS · 8 WORKERS` |
| Chat stop | A 32×32 `button[aria-label="Stop the active run"]` now sits left of Submit; it calls `cancelRun`; disabled at terminal | "No stop-in-chat" part of F10 **fixed** (32 px, below 40 target) |
| Tool path | Chat showed a completed `file_read` prompt/answer (`HiveMind portable desktop build`) from another session, with 8 worker nodes | Tool path is **reachable** on B; F8's "dead" verdict is build-A-only. No `TOOL_CALL` frames were captured by this late-joining smoke, so the wire sequence is **not** verified |
| Models | Adds `WIRE-SUPPORTED` / `NOT ACCEPTED` per model (`ModelsPanel.tsx:31, 208`) | "Available models" overclaim **partly fixed**; still no selector/refresh/loading state |
| Models usage | `1950 / 42 / 1992`, cache `1664 / 286`, reasoning `Not reported`; balance `8.74` | Usage read path still works; the null-usage/cancelled-run gap is unchanged in source |
| Models controls | still no control inside `<main>` on Models | F4 (no selector), F6 (no refresh/availability/time) persist |

Screens: `workspace-buildB-1440x900.png`, `models-buildB-1440x900.png`. These are **not** counted in the 1440/1280/1024 evidence matrix; they are a dated look-ahead. The app state on B includes runs created by other clients sharing the port; no claim is made that those runs belong to this audit.

**Truth labels used in this document**

| Label | Meaning |
|---|---|
| **WIRED-AND-WORKS** | The visible control/state was exercised or directly observed end-to-end against the live server in this pass. |
| **WIRED-BUT-DEAD** | Code path exists, but no live state can reach it on the audited build (or it is unreachable without a product change). |
| **MISSING-UI** | No control/state is exposed for a documented/claimed capability. |
| **LYING** | The UI asserts something the live evidence contradicts, or shows a state that is not true at that moment. Latent versions are marked *(latent)* and are code-only. |

**Screenshot methodology note.** Screenshots are captured with Playwright `animations: 'disabled'`. With default animation capture, `.chat-message` (`soft-in`) sometimes paints at its initial `opacity:0` in headless Chrome; the DOM and computed styles were fully populated (`opacity:1`, dark text, correct rects) and the same frames render when animations are disabled. That is an evidence-capture artifact, **not** a product defect and not reported as one.

---

## 2. Provider / model / balance / usage surfaces

### 2.1 What is visible, and where

| Surface | What the live UI shows | Data path | Wired? |
|---|---|---|---|
| Header | `CONNECTED` dot only. No provider, model, balance, or spend. | WS status | WIRED-AND-WORKS |
| Models → identity | `PROVIDER deepseek`, `MODEL deepseek-flash` | `SETTINGS_STATUS` (WS, only if settings unknown) | WIRED-AND-WORKS |
| Models → usage | `PROMPT TOKENS 88`, `COMPLETION 5`, `TOTAL 93`, `CACHE HIT/MISS 0 / 88`, `REASONING Not reported` (23:52 run) | `GET /api/usage/runs/{run_id}` | WIRED-AND-WORKS |
| Models → provider account | `BALANCE USD 9.17` (observed values 9.50 → 9.17 → 9.04 → 8.92 → 8.97 across the window), `AVAILABLE MODELS` with `deepseek-flash` and `deepseek-v4-pro`, `1048576 CTX · 393216 MAX OUT` | `GET /api/provider/balance` + `GET /api/provider/models` in one `Promise.all` | WIRED-AND-WORKS for the happy path |
| Models → contract note | Static `CONTRACT REFERENCE (FACT-REGISTER): DEEPSEEK-FLASH; DEEPSEEK-V4-PRO IS OPTIONAL.` | `STRINGS.models.contractNote` | WIRED-AND-WORKS |
| Settings → provider block | `PROVIDER deepseek`, `MODEL deepseek-flash`, `THINKING MODE enabled`, `REASONING EFFORT high`, `PROVIDER KEY STATUS Key configured` | `SETTINGS_STATUS` (WS) + `GET /health` on mount | WIRED-AND-WORKS |
| Settings → health | `ok`, `RUSTC:OK`, `NODE:OK`, `DEEPSEEK_API_KEY:OK` | `GET /health` | WIRED-AND-WORKS |
| Library → State & settings (disclosure) | provider/model/key, only after a probe; dashes until then | store `settings` (no probe of its own) | WIRED-AND-WORKS (stale until probed) |
| HUD | throughput counts only; no provider/model/tokens/cost | `RUN_METRICS` or derived from tasks | WIRED-AND-WORKS |
| Chat | no per-message model, token, cost, or provider label | WS `REPLY_*` | MISSING-UI (metadata) |

**Observed live endpoint values**

```
GET /api/provider/models    200 in 643 ms
{"object":"list","models":[
 {"id":"deepseek-flash","name":"DeepSeek-V4.1-Flash","context_window":1048576,"max_output_tokens":393216},
 {"id":"deepseek-v4-pro","name":"DeepSeek-V4-Pro","context_window":1048576,"max_output_tokens":393216}]}

GET /api/provider/balance   200 in 399 ms (first probe)
{"is_available":true,"balances":[{"currency":"USD","total_balance":"9.50",
  "granted_balance":"0.00","topped_up_balance":"9.50"}]}

GET /api/usage/runs/01a1b86d-8b15-748e-abde-2c97f347d90c  200
{"usage":{"prompt_tokens":88,"completion_tokens":5,"total_tokens":93,
  "prompt_cache_hit_tokens":0,"prompt_cache_miss_tokens":88,"reasoning_tokens":null}}
```

Both provider proxies are real calls through `hiverouter-provider` (`client.rs:261-313`, `dto.rs:881-933`), not client fixtures. The UI's 1,048,576 and 393,216 are **provider-reported** values, not invented by the UI — but they are **not version-pinned** (see §9) and the UI gives no source/timestamp for them.

### 2.2 Models surface, by viewport

| Viewport | Layout observed | Verdict |
|---|---|---|
| 1440×900 | content column 726 px wide at x≈514; identity, usage, provider account stacked; no clipping | WIRED-AND-WORKS |
| 1280×900 | panel 728 px wide; identity + provider account fit; no clipping | WIRED-AND-WORKS |
| 1024×800 | content ≈680 px wide inside 712 px main; identity, usage, account all fit; two-column account retained; no horizontal scroll | WIRED-AND-WORKS |

Screens: `models-1440x900.png`, `models-1280x900.png`, `models-1024x800.png`, `models-loading-1440x900.png`, `models-after-run-1440x900.png`, `models-provider-error-1280x900.png`.

**The false-empty finding (F2), with evidence.** Immediately after clicking Models:

```
Models
PROVIDER deepseek
MODEL deepseek-flash
PROMPT TOKENS 88 ... TOTAL TOKENS 93 ...
PROVIDER ACCOUNT
BALANCE
NO BALANCE ROWS REPORTED.
AVAILABLE MODELS
CONTRACT REFERENCE ...
```

That text was read at t≈0 ms after navigation while `GET /api/provider/models` and `/api/provider/balance` were still in flight; ~0.6–1 s later the same page showed `USD 9.04`, `deepseek-flash`, `deepseek-v4-pro`. `ModelsPanel` initialises the arrays to `[]`, has no `loading` state, and renders `providerBalances.length === 0 ? "No balance rows reported."` (`ModelsPanel.tsx:180-190`). There is no spinner and no "checking provider…" label. The screenshot `models-loading-1440x900.png` captures the false empty state. This is a **transient lie**, not a slow-load cosmetic issue: the sentence is false for the interval in which it is shown.

**No controls on Models.** A live DOM query of `<main>` on the Models surface found **0 buttons and 0 links** (`models1440` measure). The only way to refresh balance/models is to navigate away and back, which remounts the panel and re-issues all three requests. There is no retry for the provider fetches and no fetch timestamp.

**Error path (fault-injected).** With `GET /api/provider/**` fulfilled as HTTP 503 and `GET /api/usage/**` as HTTP 500 (Playwright route, no product edit), the UI showed:

```
PROVIDER deepseek
MODEL deepseek-flash
USAGE UNAVAILABLE; NO TOKEN COUNT INVENTED.
PROVIDER ACCOUNT
PROVIDER METADATA HTTP 503/503
CONTRACT REFERENCE (FACT-REGISTER): ...
```

Findings:
- the error is the raw concatenated HTTP status (`ModelsPanel.tsx:59-65`), not a named refusal from the contract table;
- no `how_to_fix`, no retry button, no named `E_PROVIDER_*` code;
- the message is a plain `<p class="text-signal">` (`ModelsPanel.tsx:174-176`) and is **not** `role="alert"`; the DOM query on error returned `alerts: []`, so an async failure is not announced;
- the usage error sentence is honest ("no token count invented") but sits alone with no retry or named cause.

Screen: `models-provider-error-1280x900.png`. Requests were recorded with 503/503/500; the UI recovered only after a manual nav away/back (and after routes were removed), which is the practical "retry".

### 2.3 Settings surface

Live Settings (1440×900) shows provider/model/thinking/effort/key and health. `Refresh` (provider key status) sends:

```
{"schema":"hiverouter.v1","type":"COMMAND","name":"SETTINGS_STATUS",...}
→ ACK {command:"SETTINGS_STATUS"} → SETTINGS_STATUS_RESULT {provider:"deepseek",model:"deepseek-flash",
  key_present:true,thinking:"enabled",reasoning_effort:"high"}
```

`Re-check health` issues `GET /health`. Both work. The buttons are 66.5×26 and 121.9×26 px — below a 40×40 target. The key label is precise ("Key configured"), not "key valid"; a present-but-invalid key would still read "configured" and only a run would surface `E_PROVIDER_AUTH`. That is defensible wording, not a numeric lie, but the UI does not test validity on demand.

**Copy seam:** `STRINGS.settings.keyUnprobedHint` says "Submit a prompt to surface E_KEY_MISSING if no key is configured." This is honest, but it makes the only key-validity test a paid run.

Screen: `settings-1440x900.png`; focus evidence `settings-focus-1280x900.png`.

### 2.4 Library, HUD, chat

- **Library → State & settings** (`LibraryPanel.tsx:159-188`) exposes provider/model/key behind a `<details>`; it reads the store's `settings`, and **does not probe** on its own. On a fresh boot the values are `—` until Settings or Models is opened (or a `SETTINGS_STATUS_RESULT` arrives). It is honest but stale; the disclosure is below the fold at 800 px height and the whole library is hidden below the `lg` breakpoint (<1024 px).
- **HUD** (`HudBadge.tsx:38-51`) never says `derived` vs `server` in text; the only cue is the dot colour (honey for derived, green for server). The strings `sourceDerived`/`sourceServer` exist but are not rendered. A terminal run can therefore show `100.0% GREEN` as "swarm throughput" with no terminal/derived context. On the audited boot the historical completed run showed `0 SUPERAGENTS · 2 WORKERS · 100.0% GREEN`; that "2 workers" comes from the invented `kind` in §2.6/F1, not from the server.
- **Chat** has no provider/model/tokens/cost label per message and no run id per message; the chat transcript survives across runs (observed: messages from the 23:52 run, the 18:25 run, and the cancelled runs in one list).

### 2.5 Usage lifecycle (the money view is incomplete)

Observed runs in the audited session:

| Run | Status | `GET /api/usage/runs/{id}` | UI |
|---|---|---|---|
| `01a1b86d…` (23:52) | completed | `88 / 5 / 93`, cache `0/88`, reasoning `null` | Models showed all six counters, reasoning `Not reported` |
| `01a1b88b-53c2…` (18:25:01) | completed | `135 / 6 / 141`, cache `0/135`, reasoning `null` | Models showed `135 / 6 / 141`; balance changed 9.04 → 8.97 |
| `01a1b88b-7190…` (18:25:08) | cancelled (queued) | `usage: null` | No usage block at all |
| `01a1b88d-0665…` (18:26:52) | cancelled mid-stream | `usage: null` | No usage block; partial assistant text remains |

The usage surface is tied to **the current run only** (`ModelsPanel.tsx:84-134`). When the current run has no usage row, the block is not rendered; there is no "Not reported", no run id, no "cancelled before usage" note. `models-1280x900.png` was captured while the current run was the cancelled one: the page has identity and account, **no usage block**, while the completed run's 135-token row existed in the API. The History tab lets an operator inspect committed revisions but does not change the Models run selection; there is no per-run usage browser.

A cancelled mid-stream run **has consumed tokens** (the audit saw ~9 numbered facts stream before cancel) yet reports `usage: null`. That is a backend accounting gap surfaced as a UI omission; from the operator's point of view, spend is invisible. If usage is meant to be persisted only for terminal runs with a provider `REPLY_DONE`, that policy must be visible; it is not.

**No cost view.** The backend has `pricing.rs` and Decimal accounting, but the UI shows token counts only; no USD cost per run, no account spend, no budget. This is not necessarily a defect (the product may choose token-only), but it is a missing surface relative to "balance/usage".

### 2.6 Provider-adjacent state findings

**F1 — history graph invents `kind: 'worker'` (High, observed).**

The server snapshot's `TaskNode` carries `task_id`, `parent_task_id`, `state`, `rev` — no kind, label, progress. The store maps every node to:

```ts
// store.ts:612-617
// `GraphSnapshot::TaskNode` carries no kind/label/progress: none is invented.
kind: 'worker',
label: fallbackTaskLabel(null, node.id),
```

The comment says "none is invented" while the code asserts a kind. Live effect on the audited bundle:

- `GET …/state?revision=3` for the 23:52 run: two nodes, root `parent_task_id: null`, child under it.
- Workspace showed **both** as `WorkerAgent 01a1b86d`; the Target select offered two identical `WorkerAgent 01a1b86d` options; Library showed two `#01A1B86D` chips; HUD rendered `0 SUPERAGENTS · 2 WORKERS`.
- Agents surface showed both as `WORKERAGENT`.

Even without knowing the root's true kind, the UI cannot justify "worker" from the snapshot; it should render "kind unknown" or omit the kind. The HUD then repeats the invented classification as a factual count. This is a **LYING** surface, not merely missing data.

**Post-audit source movement (not in the audited bundle):** the working tree at 18:27:44 changed `fallbackTaskLabel` to use the random UUID tail and `Agent …` when kind is null (`store.ts:328-331, 617`), which fixes only the *duplicate label*; `kind: 'worker'` and the HUD consequence remain. Build B confirms this: labels are now unique (`Agent d53c65f4` …) but HUD still reads `0 SUPERAGENTS · 8 WORKERS`, and every node still carries the hardcoded `worker` kind.

**F9 — HUD derived/terminal context (Medium, soft lie).** `deriveHud` (`metrics.ts:24-47`) labels derived data internally, but `HudBadge` renders only numbers and a coloured dot. On a boot showing a completed historical run, the badge read `100.0% GREEN` with no "derived", "completed", or "historical" word; the scrubber said `4 OF 4` and the control bar said `UNAVAILABLE`, so the screen carries contradictory live/terminal signals. Severity is medium because the dot is honey (the only cue) and the numbers are derived from real states, not fabricated.

**F6 — balance fields dropped (Medium; latent lie).** The proxy returns `is_available`, `granted_balance`, `topped_up_balance`; `ModelsPanel` reads only `currency` and `total_balance` (`ModelsPanel.tsx:180-191`). If `is_available` were false, the page would still render `USD 9.50` with no unavailable marker. This was **not** exercised (the live account was available), so it is a latent truthfulness risk, not an observed lie. The same code drops the grant/top-up split and any "as of" time.

**F4 — no model/provider selector (High, missing).** There is no `<select>`, input, or button anywhere in the app that sets provider, model, thinking, or effort. The only occurrences are read-only `dd` values. The configured model is fixed by server config (`deepseek-flash` per `SETTINGS_STATUS`). The provider catalogue lists `deepseek-v4-pro`, but selecting it is impossible from the UI; the label `AVAILABLE MODELS` overstates availability *to this app*. This matches `02`'s finding; the UI has no control, not even a disabled one.

**Unverified-number note.** `393216 MAX OUT` is provider-reported by live `/models`, and the UI faithfully displays it; it is **not** a UI fabrication. But it is the same 393,216 number the workspace contract flags as unconfirmed by a version-pinned official source. The UI presents it as a fact with no "reported by provider at …" provenance. Treat it as **provider-sourced, version-unpinned**, not as verified product capability.

---

## 3. Prompt → streaming → tool/action states

### 3.1 Observed sequence (real keyed run)

Run `01a1b88b-53c2-72dd-8d36-a2b8902aa0ca`, prompt `Reply with exactly HIVE_AUDIT_OK`, captured from the live WebSocket:

| Wall time (from submit) | Frame / state | UI consequence |
|---|---|---|
| +0 ms | `SUBMIT_PROMPT {text, project_id}` sent | user bubble appended optimistically; draft retained until ACK (`ChatCore.tsx:132-169`) |
| +7 ms | `RUN_CREATED {status:"queued"}` then `ACK {command:"SUBMIT_PROMPT"}` | run status `QUEUED`; graph reset; composer cleared on ACK |
| +0.3 s | `RUN_METRICS sup=0 work=0 green=0.0 delta=0.0` | HUD `0 SUPERAGENTS · 0 WORKERS · 0.0% GREEN` (no "no metrics" because a server frame exists) |
| +1.3 s | `RUN_STATUS running`, `RUN_METRICS work=1` | Cancel enabled; Spawn enabled; Kill/Re-route still gated |
| +1.81 s | first `REPLY_DELTA` (`"H"`) | assistant bubble with `HIVE AI ASSISTANT · LOADING` and caret; chat status `STREAMING` |
| +1.81…1.94 s | six deltas: `"H"`, `"IVE"`, `"_A"`, `"UD"`, `"IT"`, `"_OK"` | text grows in place; `aria-live="polite"` on the list |
| +1.94 s | `REPLY_DONE {usage:{135,6,141,cache 0/135,reasoning null}}` | caret/label removed; usage persisted |
| +1.95 s | `RUN_METRICS green=50.0 → 100.0`, `RUN_STATUS completed` | status `UNAVAILABLE`; Cancel/Kill/Spawn disabled; scrubber becomes `4 OF 4` |

Total prompt→terminal ≈ **1.94 s**. The stream was real (six deltas), not a single fabricated message. Screens: `chat-submitted-1440x900.png` (queued), `chat-streaming-live-1280x900.png` (mid-stream), `workspace-after-run-1440x900.png` (terminal).

**Draft/refusal behavior (code + observed):** on ACK, the draft is cleared only if it still equals the submitted text (`ChatCore.tsx:159`); on refusal, the optimistic user row is retracted and the draft restored (`ChatCore.tsx:162-168`). Empty submit produced the named toast `E_PROMPT_EMPTY` with `TYPE A PROMPT BEFORE SUBMITTING (1-8000 CHARACTERS).` and sent no WS command (`empty-prompt-refusal-1440x900.png`).

### 3.2 Controls by phase

| Phase | Composer | Submit | Cancel | Kill | Spawn | Re-route |
|---|---|---|---|---|---|---|
| Terminal/boot | enabled | disabled (empty) | disabled | disabled | disabled (title `No active run`) | disabled |
| Queued | enabled | disabled (pending) | **enabled** | disabled | **enabled** | disabled |
| Running/streaming | enabled | disabled | **enabled** | disabled unless a worker is selected | enabled | requires selection + target |
| Completed/cancelled | enabled | disabled (draft empty) | disabled | disabled | disabled | disabled |

Observed values: before submit all four control-bar buttons were `disabled` with `opacity:0.5`; in the queued screenshot `SPAWN SUB-HIVE` and `CANCEL` were visually enabled; at terminal all were disabled again. `composerDisabled` was `false` throughout because the connection stayed open (the composer is disabled only when disconnected or replaying, `ChatCore.tsx:337`).

**Spawn at QUEUED (Low, observed).** `canSpawn = connected && runActive && !anyPending && !replaying` (`ControlBar.tsx:33`) is true while the run is `queued` with **0 tasks**. Clicking it then takes the `parentTask` fallback, finds no task, and pushes `E_TASK_NOT_FOUND` (`store.ts:1850-1855`). The button is enabled for an action that cannot succeed until a node exists. Not exercised by click; code path is deterministic.

**Target select and agent chips (Low-Med, observed).** On the audited bundle two tasks in the same run share the same visible label because their UUIDv7 prefixes share the run timestamp bucket (`WorkerAgent 01a1b86d` twice); the Target `<select>` has two identical option labels and the Library two identical accessible names. Selecting either is ambiguous. This is a label/identity problem, not a data problem. The 18:27 working-tree change mitigates the label only.

### 3.3 Cancellation — works at the run level, lies in the transcript

Two cancellation paths were exercised:

**(a) Cancel while QUEUED.** Prompt at 18:25:08; `RUN_CREATED status=queued`, ACK; `CANCEL_RUN` sent ~0.1 s later; **`ACK {command:"CANCEL_RUN"}` in 104 ms**, then `RUN_STATUS status=cancelled`. No nodes ever spawned; Workspace showed `0/0`, HUD `NO METRICS`, status `01A1B88B · UNAVAILABLE`, scrubber `NO RUN HISTORY YET`. Screen: `workspace-cancelled-1440x900.png`. This path is **WIRED-AND-WORKS**.

**(b) Cancel mid-stream (the defect).** Prompt `List ten short facts about bees, one per line.`; after the first assistant text and with `Cancel` enabled, cancel was clicked:

```
ACK {command:"CANCEL_RUN"}
RUN_METRICS sup=0 work=1 green=0.0 delta=0.0
RUN_STATUS status=cancelled
```

**No `REPLY_DONE` was sent.** The store clears `streamingRunId` on terminal `RUN_STATUS` (`store.ts:1377-1393`) but clears each message's `streaming` flag **only** on `REPLY_DONE` (`store.ts:1423-1429`). `ChatCore` renders the label from `message.streaming` (`ChatCore.tsx:267-270`). Result, still present 1.5 s after cancel:

```
HIVE AI ASSISTANT · LOADING
1. Bees are flying insects...
2. There are over 20,000 known species...
... 9. Honey bees
[blinking caret]
```

The run is terminal and the Cancel button is disabled, but the transcript says a reply is still loading. On reload the partial bubble may vanish (messages are rehydrated from `/messages`), but in-session it is a permanent false loading state until navigation/reload. Screen: `chat-cancelled-midstream-1280x900.png`. This is **LYING** and is the most user-visible action defect in the audit.

The reason no `REPLY_DONE` arrives is outside the UI (server cancel path); the UI nevertheless owns the stale `message.streaming` state. A terminal `RUN_STATUS` should end every streaming bubble for that run (or the server must emit a terminal reply frame).

### 3.4 Retry and stop

- **No retry-run control.** After a refusal/terminal error, the only actions are Dismiss on the toast and Submit again; the draft is restored on send refusal, so a manual resubmit is possible. There is no `Retry` for a run, model call, or provider error. The only `Retry` in the UI is inside the approval overlay (`ApprovalOverlay.tsx:310-320`), which is unreachable on this build. **MISSING-UI**.
- **No stop-in-chat on build A.** The composer has no stop/square button; `Cancel` lives in the ControlBar at the bottom of the canvas (`ControlBar.tsx:97-106`), 40 px high, far from the chat core. On cancellation, focus is not moved to the streaming transcript, and no confirmation is asked. **MISSING-UI** relative to chat-centered generation controls. **Build B** adds a 32×32 `Stop the active run` button in the composer (source `ChatCore.tsx:339-351`, 18:26:10), which was not in the audited bundle; its disabled logic is `streamingRunId === null && runStatus !== 'running'`, so it is enabled only while a run is active.
- **No confirmation for Cancel/Kill.** Single click, immediate command. Cancel is reversible only by resubmitting; Kill is destructive. The reference's kill confirmation state is not implemented (see 05). Not a defect by contract, but a UX risk.

### 3.5 Tool and approval states — unreachable on the audited build

Across three live runs (one completed, one queued-cancel, one mid-stream-cancel):

- **0** `TOOL_CALL`, **0** `TOOL_RESULT`, **0** `APPROVAL_REQUEST` frames.
- Chat never rendered the `ACTIONS` block (`ChatCore.tsx:273-307`); Agents showed `Tool activity · 0` + `No tool activity yet — tool calls appear here when the model invokes a tool.`; no `role="dialog"` approval overlay appeared.
- The empty state is honest (no fabricated tool rows).

Root cause is backend, recorded in `01`/`02`: on the pre-18:19 bytes the first leaf request never advertised `tools`, so a compliant model could not emit a tool call; `02` records a fix at 18:19:33 (`provider_executor.rs:306-322`) that is **uncommitted, untested, and not in the audited 18:18 binary**. Therefore, on the audited build:

- `APPROVAL_REQUEST`-driven Approve/Deny/Retry/Dismiss: **WIRED-BUT-DEAD** (code complete, no reachable producer).
- `TOOL_CALL`/`TOOL_RESULT` activity rows: **WIRED-BUT-DEAD** (ditto).
- The approval dialog's `role="dialog" aria-modal="true"` has no focus trap or initial focus in code; this was **not exercised** and must be re-tested once approvals are reachable.

**Build B look-ahead (not evidence-set):** on the relaunched 18:33:37 build, the chat showed a completed `file_read` prompt/answer (`HiveMind portable desktop build`) and 8 worker nodes. That is consistent with the `02` tools fix being reachable, so the "dead" verdict must not be carried forward to build B. The smoke did **not** capture `TOOL_CALL`/`TOOL_RESULT` frames (it joined after the run), and no `APPROVAL_REQUEST` was seen, so the approval overlay and the chat `ACTIONS` block remain **unverified** on B. Re-run a live tool prompt with a frame tap before claiming either works.

### 3.6 Empty / loading / error inventory

| State | Where | Observed | Verdict |
|---|---|---|---|
| Empty chat | ChatCore empty state + three starter prompts | not reachable in the audited session because a run existed; code present (`ChatCore.tsx:228-254`) | WIRED-AND-WORKS (code), not exercised |
| No run | HUD `NO METRICS`, ControlBar `UNAVAILABLE`, scrubber `NO RUN HISTORY YET` | observed after cancel | WIRED-AND-WORKS |
| Loading (prompt ACK) | status `QUEUED`/`RUNNING`; submit disabled | observed | WIRED-AND-WORKS |
| Streaming | `· LOADING` + caret + `STREAMING` status | observed (six deltas) | WIRED-AND-WORKS |
| Empty prompt | toast `E_PROMPT_EMPTY` + how-to-fix | observed | WIRED-AND-WORKS |
| Provider loading | Models account | false `No balance rows reported.`; no spinner | **LYING** (F2) |
| Provider error | Models account | raw `HTTP 503/503`; no role alert; no retry | **MISSING-UI** (F7) |
| Usage error | Models usage | `Usage unavailable; no token count invented.` | WIRED-AND-WORKS (honest), no retry |
| Usage empty | Models usage | block omitted when `usage === null` | **MISSING-UI** (F5/F11) |
| Cancelled transcript | Chat | stuck `· LOADING` after `RUN_STATUS cancelled` | **LYING** (F3) |
| Offline | App banner + toast, mutations blocked | not exercised (did not kill the server) | WIRED (code), unverified |

---

## 4. Responsive geometry — 1440 / 1280 / 1024

Measured with `getBoundingClientRect` in the live page. No CSS assumptions.

| Viewport | Nav | Library | Main | Chat core | HUD | Mini-map | Control bar | Result |
|---|---|---|---|---|---|---|---|---|
| **1440×900** | 72 | 240 | 1128 | 400×480 at (676,204), bottom 684 | 460×44 at (646,64) | 220×150 at (1180,630), visible | 1128×53 at y804; scrubber y877 | no overlap; 104 px gap chat→minimap; doc 1440×900 |
| **1280×900** | 72 | 240 | 968 | 400×480 at (596,204) | 460×44 at (566,64) | 220×150 at (1020,630), visible | 968×53 | 24 px gap chat→minimap; no overlap; no scroll |
| **1024×800** | 72 | **240 (still visible at lg)** | 712 | 400×480 at (468,154), bottom 634 | 460×44 at (438,64) | **`display:none`** (`index.css:137-142`, max-width 1279) | 712×101 at y656 (wraps to 2 rows); 22 px below chat | no overlap; `Kill worker` right edge 546.6 < 1024; no horizontal scroll; minimap intentionally absent |

**Findings**

- **No clipped controls and no surface overlap at any tested width.** The 1024 control bar wraps to two rows (`TARGET … SPAWN … RE-ROUTE`, then `CANCEL KILL WORKER` + status) and ends at y757; chat ends at y634 and the bar starts at y656, so there is a 22 px gap and no overlap. The canvas shrinks, but controls remain whole and reachable.
- **Mini-map is hidden below 1280 by design** (`@media (max-width: 1279px)`). At 1024 there is no mini-map at all — not clipped, not broken, but a missing reference surface at that width. If 1024 is a supported width, this is a deliberate omission to document, not a bug.
- **Library is hidden below 1024** (`hidden … lg:flex`, `LibraryPanel.tsx:45`), so at widths <1024 there is no tab strip, no template list, and no State & settings disclosure. At exactly 1024 it is visible (lg = 1024). No 1024-specific defect; the 1023 breakpoint is untested.
- **Chat stays 400 px wide at 1024** (56% of the 712 px main); the graph's two WorkerAgent cards extend behind/around it and one card is partly off-canvas at the right edge. Panning/zoom is available; this is canvas content, not a clipped control.
- **Models** fits at all three widths (see §2.2); the account two-column layout remains readable at 1024 (≈680 px content).
- **Under-40 px interactive targets** (not provider-only): search 32×32, library tabs 26 px high, template rows 30 px high, canvas pills 26 px high, zoom buttons 32×32, chat close 24.9×22, submit 32×32, selects 36 px high, Settings refresh 26 px high. The provider-specific ones are the two Settings buttons (26 px) and the absent Models controls. Target-size is an accessibility gap, not a clipping gap.
- **Clipped text** is confined to intentional truncation (`w-12 truncate` on agent chips: 62 px content in 48 px; worker card 322 px content in 318 px). No provider value is truncated at any tested width.

Screens: `workspace-1440x900.png`, `workspace-1280x900.png`, `workspace-1024x800.png`, `models-1024x800.png`; control crops `controls-1440x900.png` (1128×200) and `controls-1024x800.png` (712×180).

---

## 5. Keyboard, focus, accessibility

**Keyboard evidence (live, 1440×900)**

- First `Tab` reaches the skip link (149.1×44) with a visible ring; then Search, the active nav item, the active library tab, the two agent chips, and the four template buttons. All had `outline: rgb(78,58,21) solid 2px; outline-offset: 2px` (global `:focus-visible`, `index.css:27-30`). **Focus visibility passes.**
- Nav rail is a roving tablist: only the current item is `tabIndex=0`; `ArrowDown` moved focus from Workspace to Agents and `Enter` navigated to `Agents · 2`. **Keyboard navigation works.**
- Library tabs use ArrowLeft/Right/Home/End and activate on focus (code `LibraryPanel.tsx:54-76`); not separately exercised beyond the roving check.
- Models surface has **no focusable controls** in `<main>`; tab stops remain header/search, active nav, library controls. That is correct for read-only data, but it means provider state cannot be reached/refreshed from the keyboard except by re-navigating.
- Settings has two native buttons, reachable and operable by Enter/Space; focus ring observed on `REFRESH` (`settings-focus-1280x900.png`).

**Accessibility findings**

1. **Target size:** `REFRESH` 66.5×26, `RE-CHECK HEALTH` 121.9×26 — below 40 px height. Other under-40 controls listed in §4. (WCAG 2.5.8 target size minimum is 24×24, so these pass that; the reference's 40 px "min touch target" is not met. State this precisely: 26 px > 24, but below the product's own 40 px note.)
2. **Provider error is not announced:** `ModelsPanel` renders the error in a plain `<p>` (`:174-176`); the live error DOM had `role=alert` count 0. Usage error likewise plain text. A screen-reader user who opens Models and waits gets no announcement when the fetch fails.
3. **Provider lists lack group labels:** the `<ul>` for balances and models are preceded by visual `<div>` labels ("Balance", "Available models") but no `<h*>`, `aria-labelledby`, or `aria-label` on the lists. The enclosing `section` has `aria-label="Provider account"`, which helps, but the two lists are not individually named.
4. **Duplicate accessible names (audited build):** two agent chips both named `WorkerAgent 01a1b86d (Completed)`; two identical options in the Target select. This makes the selection ambiguous for screen-reader and speech-input users. The 18:27 label-only fix may make the tail unique; the kind assertion remains.
5. **Approval dialog** (`role="dialog" aria-modal="true"`) has no focus trap, no initial focus, no Escape handler in code. Unreachable on this build; mark unverified and re-test when approvals are reachable.
6. **Balance/model updates** are not `aria-live`; after a manual navigation, a screen reader reads the page on demand. Models has no refresh, so a quoted balance can go stale in the DOM without any announcement. This is minor.
7. **Contrast:** not measured in this pass; the existing token/contrast system is out of scope. No contrast claim is made.

---

## 6. Control → endpoint / store hop matrix

Scope: every visible control on Workspace, Models, Settings, Agents, and the chat core, plus provider-relevant read paths. "Observed" means seen in this pass, not inferred from code alone.

| # | Control / surface | Action | Endpoint / store hop | Observed evidence | Verdict |
|---|---|---|---|---|---|
| 1 | Nav **Workspace/Agents/Data/Models/Routing/Settings** | sets `view.nav` | local store `setNav`; panels then fetch | `SETTINGS_STATUS`, `/health`, `/api/provider/*`, `/api/agents/*`, `/api/history/*` all fired on the expected nav | WIRED-AND-WORKS |
| 2 | **Models** surface (read) | mounts | `GET /api/provider/models`, `GET /api/provider/balance`, `GET /api/usage/runs/{id}`, `SETTINGS_STATUS` if unknown | 200s; values above | WIRED-AND-WORKS |
| 3 | **Models** account loading | — | none | false `No balance rows reported.` while requests in flight | **LYING** (F2) |
| 4 | **Models** account error | — | raw `fetch` failure | `PROVIDER METADATA HTTP 503/503`; no role alert; no retry | **MISSING-UI** (F7) |
| 5 | **Models** usage | mounts / run changes | `GET /api/usage/runs/{id}` | 88/5/93 and 135/6/141 shown; `usage:null` → no block | WIRED-AND-WORKS (data) / **MISSING-UI** (empty/history) |
| 6 | **Settings** provider block | mount | `SETTINGS_STATUS` (WS) | ACK + RESULT; provider/model/thinking/effort/key | WIRED-AND-WORKS |
| 7 | **Settings** `Refresh` (provider) | click | `SETTINGS_STATUS` (WS) | command + ACK + RESULT observed; 26 px high | WIRED-AND-WORKS |
| 8 | **Settings** `Re-check health` | click | `GET /health` | request observed; 26 px high | WIRED-AND-WORKS |
| 9 | **Settings** Reduced motion override | toggle | local store + `data-reduced-motion` | not exercised; code | WIRED-AND-WORKS (code) |
| 10 | **Library** State & settings disclosure | toggle | local; reads store settings | provider/model/key behind disclosure; dashes until probe | WIRED-AND-WORKS |
| 11 | **Library** tabs Workspace/Swarm/Routing/History | click | local `setTab` | code + keyboard | WIRED-AND-WORKS |
| 12 | **Library** agent chip | click | local `selectTask` | two identical names on audited build | WIRED-AND-WORKS (identity defect) |
| 13 | **Library** template buttons ×4 | click | `TEMPLATE_INSERT` (WS) | not exercised; code `insertTemplate` | WIRED-AND-WORKS (code) |
| 14 | **Chat** composer | type | local | draft/persistence observed | WIRED-AND-WORKS |
| 15 | **Chat** Submit (arrow) | click/Enter | `SUBMIT_PROMPT` (WS) | two runs observed; optimistic row; draft clear on ACK | WIRED-AND-WORKS |
| 16 | **Chat** empty submit | Enter | local `E_PROMPT_EMPTY` toast; no WS send | toast observed with code/how-to-fix | WIRED-AND-WORKS |
| 17 | **Chat** close/open | click | local `chat.visible` | code | WIRED-AND-WORKS |
| 18 | **Chat** starter prompts (empty state) | click | fills draft | code only (a run existed) | WIRED-AND-WORKS (code) |
| 19 | **Chat** stop generation | click | `CANCEL_RUN` (build B; none on build A) | build A had no control; build B adds `aria-label="Stop the active run"` calling `cancelRun`, 32×32, not exercised | **MISSING-UI** (F10, build A) / **WIRED-AND-WORKS** (build B, unverified) |
| 20 | **Chat** retry run | — | none | no control | **MISSING-UI** |
| 21 | **ControlBar** Target select | change | local `selectTask` | two identical options on audited build | WIRED-AND-WORKS (identity defect) |
| 22 | **ControlBar** Spawn sub-hive | click | `SPAWN_SUB_HIVE` (WS) | disabled terminal; **enabled queued with 0 tasks**; store would refuse `E_TASK_NOT_FOUND` | WIRED-AND-WORKS (gate) / **MISSING-UI** (queued guard) |
| 23 | **ControlBar** Re-route select + button | click | `REWIRE_TASK` (WS) | disabled without running worker + target; not exercised | WIRED-AND-WORKS (gate) |
| 24 | **ControlBar** Cancel | click | `CANCEL_RUN` (WS) | ACK in 104 ms; `RUN_STATUS cancelled` (queued and mid-stream) | WIRED-AND-WORKS |
| 25 | **ControlBar** Kill worker | click | `KILL_WORKER` (WS) | disabled terminal; not exercised | WIRED-AND-WORKS (gate) |
| 26 | **ControlBar** status text | — | store `run` | `QUEUED`, `RUNNING`, `01A1B88B · UNAVAILABLE` observed | WIRED-AND-WORKS |
| 27 | **Canvas** Hide finished / Focus branch / Graph only | click | local filters / chat visibility | code; disabled states observed | WIRED-AND-WORKS (code) |
| 28 | **Canvas** zoom +/−/recenter/fit | click | React Flow local | code; 32×32 | WIRED-AND-WORKS (code) / target-size defect |
| 29 | **Canvas** minimap pan/zoom | drag | React Flow local | visible at 1280/1440; hidden at 1024 | WIRED-AND-WORKS |
| 30 | **Canvas** node select / drag-to-rewire | click/drag | local select; `REWIRE_TASK` on drop | code; not exercised | WIRED-AND-WORKS (select) / unverified (rewire) |
| 31 | **Scrubber** range + `live` | change/click | `HISTORY_REVISION_SET`; `exitHistoryReplay` | 4 of 4 shown; not exercised | WIRED-AND-WORKS (code) |
| 32 | **History** revision rows + `live` | click | `HISTORY_REVISION_SET` / local exit | code; not exercised | WIRED-AND-WORKS (code) |
| 33 | **Header** Search agents | toggle/type/select | local; `selectTask` | code; focus ring observed | WIRED-AND-WORKS (code) |
| 34 | **Data** refresh / dir / file | click | `FILE_TREE_LIST`, `FILE_READ` (WS) | code; not exercised | WIRED-AND-WORKS (code) |
| 35 | **Agents** profile import/delete, memory clear, messages | click | `POST /api/agents/profiles/import`, `DELETE /api/agents/profiles/{id}`, `DELETE /api/agents/memory/{key}`, `GET /api/agents/runs/{id}/messages` | requests observed on Agents mount; mutations not exercised | WIRED-AND-WORKS (read) / unverified (mutations) |
| 36 | **Approval** Approve/Deny/Retry/Dismiss | click | `APPROVAL_RESOLVE` (WS) | no `APPROVAL_REQUEST` frames; overlay never shown | **WIRED-BUT-DEAD** (build A; unverified on build B) |
| 37 | **Tool activity** rows | read | WS `TOOL_CALL`/`TOOL_RESULT` | 0 frames; `Tool activity · 0` | **WIRED-BUT-DEAD** (build A; reachable-but-unverified on build B) |
| 38 | **HUD** source/terminal context | read | `RUN_METRICS` or derived | 100% green on terminal; no text source | **LYING** (F9) |
| 39 | **History graph** node kind | read | snapshot → store | all `worker`; root shown `WorkerAgent` | **LYING** (F1) |

**Store hop summary:** all provider reads bypass the WebSocket store except `SETTINGS_STATUS`; `ModelsPanel` calls `fetch` directly and keeps results in local component state, so provider data is not in the app store, is not replayable, is not shared between panels, and disappears on unmount. Usage is likewise panel-local. Only provider/model/key are mirrored into `state.settings` via `SETTINGS_STATUS_RESULT` (`store.ts:1624-1634`). That split explains why Library/Header/HUD cannot show balance or usage: the data never reaches them.

**Server seam (for the hops above):** routes registered at `backend/crates/hiverouter-core/src/http.rs:494-497`; `run_usage` `:811-844`; `provider_models` `:846-866`; `provider_balance` `:868-885`; provider client `GET /models` / `GET /user/balance` at `hiverouter-provider/src/client.rs:261-277`, DTOs `dto.rs:881-933`. The proxies are key-safe but unauthenticated (documented in `02`), and the UI cannot edit model/thinking/effort.

---

## 7. Lies / missing / dead — consolidated

| Verdict | Item | Severity | Evidence |
|---|---|---|---|
| **LYING** | All snapshot nodes asserted `worker`; root shown as WorkerAgent; HUD counts follow | High | `store.ts:615-617`; live `0 SUPERAGENTS · 2 WORKERS`, two `WorkerAgent 01a1b86d` cards |
| **LYING** | `No balance rows reported.` during fetch | High | `models-loading-1440x900.png`; DOM read at t≈0; `ModelsPanel.tsx:180-190` |
| **LYING** | Cancelled stream stays `· Loading` with caret | High | WS `RUN_STATUS cancelled`, no `REPLY_DONE`; `ChatCore.tsx:267-270`; screenshot |
| **LYING** (latent) | Balance renders without `is_available` | Medium | API has `is_available`; `ModelsPanel.tsx:180-191` drops it; not exercised false |
| **LYING** (soft) | HUD shows terminal 100% green as throughput, no source label | Medium | `HudBadge.tsx:38-51`; `workspace-1440x900.png` |
| **MISSING-UI** | Model/provider/thinking/effort selector | High | 0 controls in `main`; `SETTINGS_STATUS` read-only |
| **MISSING-UI** | Usage empty state / cancelled spend / history / cost | Med-High | API `usage:null`; no block in `models-1280x900.png`; `ModelsPanel.tsx:84-134` |
| **MISSING-UI** | Balance as-of time, availability, grant/top-up, refresh | Medium | same render block; no button |
| **MISSING-UI** | Named provider error + how-to-fix + retry | Medium | `PROVIDER METADATA HTTP 503/503`; no role alert |
| **MISSING-UI** | Retry run; stop in chat | Low-Med | no retry control; build A has no chat Stop (Cancel only in the bar). Build B adds a 32×32 chat Stop (look-ahead, not exercised) |
| **MISSING-UI** | Queued-state guard for Spawn | Low | `ControlBar.tsx:33`; `store.ts:1850-1855` |
| **WIRED-BUT-DEAD** | Approval overlay actions | Medium | no `APPROVAL_REQUEST`; `01`/`02` tool-loop seam; unverified post-fix |
| **WIRED-BUT-DEAD** | Tool activity rows | Medium | no `TOOL_CALL` frames; `Tool activity · 0` |
| **WIRED-AND-WORKS** | Provider identity/key/health read path | — | 200s + WS ACK; Settings screenshot |
| **WIRED-AND-WORKS** | Prompt → queued → stream → completed | — | six `REPLY_DELTA`, `REPLY_DONE`, `RUN_STATUS completed`; usage 135/6/141 |
| **WIRED-AND-WORKS** | Cancel (queued and mid-stream) at run level | — | `CANCEL_RUN` ACK + `RUN_STATUS cancelled` |
| **WIRED-AND-WORKS** | Empty-prompt refusal | — | `E_PROMPT_EMPTY` toast, no WS send |

---

## 8. Screenshot index (1440×900 / 1280×900 / 1024×800; two control-strip crops; two build-B look-ahead shots)

| File | True size | What it proves |
|---|---|---|
| `workspace-1440x900.png` | 1440×900 | Boot after the 23:52 keyed run: 2/2 nodes, HUD `0 SUPERAGENTS · 2 WORKERS · 100.0% GREEN`, chat `HIVE_UI_OK`, control bar `01A1B86D · UNAVAILABLE`, scrubber 4/4 |
| `workspace-1280x900.png` | 1280×900 | 1280 layout: minimap visible at (1020,630), 24 px from chat; no overlap |
| `workspace-1024x800.png` | 1024×800 | 1024 layout: minimap `display:none`, library still visible, 2-row control bar ends 22 px below chat; no clipping |
| `workspace-after-run-1440x900.png` | 1440×900 | Real keyed run `HIVE_AUDIT_OK` completed; HUD 1 worker/100%; usage persisted 135/6/141 |
| `workspace-cancelled-1440x900.png` | 1440×900 | Queued cancel: `0/0`, `NO METRICS`, `01A1B88B · UNAVAILABLE` |
| `chat-submitted-1440x900.png` | 1440×900 | Prompt queued: user row, `QUEUED`, Cancel enabled, Spawn enabled, 0/0, no metrics |
| `chat-streaming-live-1280x900.png` | 1280×900 | Genuine mid-stream: `HIVE AI ASSISTANT · LOADING`, caret, `STREAMING`, partial fact list, Cancel enabled |
| `chat-cancelled-midstream-1280x900.png` | 1280×900 | After mid-stream Cancel: run `UNAVAILABLE`, partial text still present with streaming caret; label stuck `LOADING` |
| `empty-prompt-refusal-1440x900.png` | 1440×900 | `E_PROMPT_EMPTY` toast with how-to-fix |
| `models-1440x900.png` | 1440×900 | Settled Models: provider/model, usage 88/5/93, balance 9.04, both models |
| `models-1280x900.png` | 1280×900 | Models at 1280 with **no usage block** while the current run has `usage:null`; account still shown |
| `models-1024x800.png` | 1024×800 | Models at 1024: fits, 88-token usage, balance 9.17, no clipping |
| `models-loading-1440x900.png` | 1440×900 | **False empty**: `NO BALANCE ROWS REPORTED.` + empty Available Models before data arrives |
| `models-after-run-1440x900.png` | 1440×900 | Usage updated to 135/6/141; balance 8.97 |
| `models-provider-error-1280x900.png` | 1280×900 | Injected 503/503 + 500: raw `PROVIDER METADATA HTTP 503/503`, usage error, no retry/alert |
| `settings-1440x900.png` | 1440×900 | Provider/model/thinking/effort/key + health; both refresh buttons 26 px |
| `settings-focus-1280x900.png` | 1280×900 | Keyboard focus ring on `REFRESH` (2 px, offset 2) |
| `agents-1440x900.png` | 1440×900 | Agents/hierarchy before the later runs; duplicate labels visible |
| `agents-after-run-1440x900.png` | 1440×900 | `Tool activity · 0` empty state, no approvals; Agents lists 4 tasks from two run prefixes (see §9) |
| `controls-1440x900.png` | 1128×200 | Disabled ControlBar at terminal: titles, 0.5 opacity, status `UNAVAILABLE` |
| `controls-1024x800.png` | 712×180 | 2-row ControlBar at 1024: all controls whole, Kill not overflowing |
| `workspace-buildB-1440x900.png` | 1440×900 | **Look-ahead only:** build B labels `Agent d53c65f4`, chat Stop button, 8 worker nodes, `0 SUPERAGENTS · 8 WORKERS · 75.0% GREEN` |
| `models-buildB-1440x900.png` | 1440×900 | **Look-ahead only:** build B adds `WIRE-SUPPORTED` badges; usage 1950/42/1992; still no selector/refresh |

No screenshots of the approval overlay or tool rows exist from build A because no live frame produced them. That absence is itself the evidence for F8 on build A. Build B's `workspace-buildB-1440x900.png` shows a completed tool-style answer from another client's run, but not the `TOOL_CALL` frames; it is a look-ahead, not approval evidence.

---

## 9. Not verified, limits, and competing explanations

| Item | Why it is not claimed | What would prove it |
|---|---|---|
| Build B (18:33:37) | The app was rebuilt after capture (`cargo build --release` at 18:28; working tree changed 18:19–18:27) and relaunched. Only a no-cost smoke was performed (§1.1); the tools fix at 18:19:33 was **not** in the 18:18 build A binary. | A live tool prompt with a WS frame tap on build B; check `TOOL_CALL`/`APPROVAL_REQUEST` frames, then re-test the chat Stop and cancel transcript. |
| Approval overlay a11y | Unreachable (no approvals). | A live tool call with approval; then test focus trap/Escape/initial focus. |
| Balance `is_available:false` | Live account was available. | Mock/route the balance proxy to `is_available:false`; observe whether UI warns (it currently cannot). |
| Balance attribution | Balance moved 9.50 → 9.17 → 9.04 → 8.92 → 8.97; the app's only completed runs were 88/5/93 and 135/6/141, and the account may be shared. The UI has no timestamp, so app spend cannot be reconciled from it. | A single-client account, before/after balance reads, and a cost row in the UI. |
| Contrast ratios | Not measured. | Token contrast audit; 05 covers visual tokens. |
| Agents `· 4` cross-run tasks | Observed: Agents listed tasks from prefixes `01a1b86d` ×2 and `01a1b88b` ×2 while adjacent Workspace captures showed 2/2 and the current run was cancelled with no committed graph. Build B similarly listed 8 tasks across mixed completed/cancelled states. This is evidence of cross-run task retention/inconsistency, but the exact race/root cause was not established; do not treat it as diagnosed. | A clean sequence with one client, WS frame log, and store snapshot at each run boundary. |
| Wide-screen >1440 / narrow <1024 | Not tested (requested widths only). | Additional viewports. |
| 1023 px library breakpoint | Not tested; at 1024 the library is visible. | 1023×800 capture. |
| Pixel parity | Explicitly not claimed. | — |

**Known environment events (not product defects asserted here):** the live app was found dead at ~18:11 (no `HiveMind`/`hiverouter` process, port closed, WebView profile Crashpad metadata at 18:11:04) and was relaunched at 18:17:56 by the parent. It went down again at ~18:28 during a rebuild and came back at 18:33:37 on build B. No restart/kill was performed by this audit. The evidence set covers build A (18:17:56–18:28) plus the two explicitly-labelled build-B look-ahead screenshots.

**Original-request coverage check**

- Provider/model/balance/usage visible/missing/lying: §2, §7.
- Prompt → streaming → tool/action states, approved controls, cancellation, retry, empty/loading/error: §3, §7.
- Responsive 1280/1024 overlaps/clipping/library/HUD/chat/minimap: §4.
- Keyboard/focus/a11y of new provider controls: §5.
- Every control's endpoint/store hop with verdicts: §6.
- Screenshots of Models, Workspace after real keyed run, 1024: §8.
- No product edits; only this file + shots: confirmed by the write set (`docs/trace/provider/03-ui-actions.md`, `docs/trace/provider/shots/*.png`). No pixel-parity claim: stated at top.

---

## Appendix A — key evidence quotes

```
WS  RUN_CREATED {"run_id":"01a1b88b-53c2-...","status":"queued"}
WS  ACK        {"command":"SUBMIT_PROMPT","correlation_id":"01a1b88b-..."}
WS  RUN_STATUS {"status":"running","error_code":null}
WS  REPLY_DELTA "H" / "IVE" / "_A" / "UD" / "IT" / "_OK"
WS  REPLY_DONE  usage {prompt_tokens:135, completion_tokens:6, total_tokens:141,
                      prompt_cache_hit_tokens:0, prompt_cache_miss_tokens:135, reasoning_tokens:null}
WS  RUN_STATUS {"status":"completed"}
```

```
Live Models at t≈0 after nav:
BALANCE
NO BALANCE ROWS REPORTED.
AVAILABLE MODELS
[empty]
```

```
Mid-stream cancel:
WS  ACK        {"command":"CANCEL_RUN"}
WS  RUN_METRICS sup=0 work=1 green=0.0 delta=0.0
WS  RUN_STATUS {"status":"cancelled"}
    (no REPLY_DONE)
DOM chat last bubble: "HIVE AI ASSISTANT · LOADING\n1. Bees are ...\n9. Honey bees"
```

```
Settings refresh:
CMD SETTINGS_STATUS → ACK → RESULT {"provider":"deepseek","model":"deepseek-flash",
 "key_present":true,"thinking":"enabled","reasoning_effort":"high"}
```

## Appendix B — reproduction notes

- Playwright: `C:\Users\Riley\Desktop\hiverouter-v1\e2e\node_modules\playwright`, installed Chrome channel `chrome` (156.0.8078.75), headless, `deviceScaleFactor: 1`.
- No `--virtual-time-budget`; no server start/stop by the audit.
- Screenshots: `animations: 'disabled'`; viewports 1440×900, 1280×900, 1024×800; full-page `false`.
- Error injections used Playwright `page.route` for `/api/provider/**` and `/api/usage/**` only; no product file was touched.
- Harness scripts live outside the repo (temp), so this workspace contains only the deliverable and its screenshots.
