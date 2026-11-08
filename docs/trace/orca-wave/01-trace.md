# 01 — TRACE audit (SEAT A)

Status: **AUDIT ONLY.** This seat edited no product file, restarted no process, called no provider, and read no key.
The only file written by this seat is this report.

- Seat: A — `/trace` (+ `/weaver` for control truth). Both skills loaded from `~/.dsh/skills/`.
- Window: 2026-11-07 23:10–23:50 America/Chicago (UTC−6).
- Workspace: `C:\Users\Riley\Desktop\hiverouter-v1`.
- App: `http://127.0.0.1:8099` — keyless (`DEEPSEEK_API_KEY` absent; `/health` `E_KEY_MISSING`).
- Question: trace the **current UI → handler → client → route → store → render-back** chain for
  (1) chat composer and send/ACK/refusal, (2) Hide/Show chat and Return live, (3) Active Agents rail,
  (4) History replay and revision scrubber, (5) graph cards / ownership links / visibility,
  (6) drag target and confirmation. Find actual bugs, dead controls, lying labels and missing wiring.
- Report path: `docs/trace/orca-wave/01-trace.md`.

> **Audit target is B2.** The tree moved repeatedly while this seat was reading it. The only live build
> that contains the 23:30–23:39 fixes is **B2** (bundle `index-CmOwogPv.js`, binary built 23:43:15,
> server pid 21324 at 23:43:19). B2 is built from the source freeze pinned in §0.1. Earlier live captures
> (B0/B1) are labelled and are used only where the cited code is unchanged between builds.
> The server was down again at 23:48 (not stopped by this seat); all B2 evidence was captured before that.

---

## 0. Snapshot, runtime and evidence rules

### 0.1 Source freeze (W1) — hash-pinned

The tree stopped changing at **23:38:48**. These are the hashes of the files this report cites; re-pin
before patching because concurrent seats had edited the same files earlier.

| File | SHA-256 (W1) | Lines | Last write |
|---|---|---|---|
| `web/src/state/store.ts` | `696C46BFC2F7A52B5B48CF7F50FF2F7DD8558C15AB5A3CC100D6F87E05A61D8A` | 2332 | 23:35:53 |
| `web/src/flow/Canvas.tsx` | `E9C28794CC00181D3B9B50F75E1768AF8A5D454DCD7A728E7A2FF2AF124E81B1` | 807 | 23:35:12 |
| `web/src/flow/graph.ts` | `441800DD59E16FEF6348A23FB661487A097EFDEFC128F0E0B8334F967DE861A7` | 344 | 23:31:20 |
| `web/src/flow/nodes.tsx` | `1EA92BB104178E27D68D997B8FABE9E4249D6FA47F31A4A2678D9D42A3C1DEFB` | 284 | 22:47:48 |
| `web/src/chat/ChatCore.tsx` | `E89D7DB72F3119A0242000BADE81690F01BB873D6B5DC50F7E86FA61C46490D4` | 381 | 23:38:48 |
| `web/src/panels/Header.tsx` | `8B32C46DF2914138B1A45738F31405D3C293150FAA2C5371939CD4D817999363` | 145 | 22:26:44 |
| `web/src/panels/ControlBar.tsx` | `39B4FF73F92BEDD312E9A7A9BF36D9DDE101451EBAA3BBFDCCD87ABC847E96F2` | 127 | 16:13:02 |
| `web/src/panels/LibraryPanel.tsx` | `0B274428EADDC4547D2F8742A2A80633C522CB0AA3DBAF2E07DCF4DAA36A3B94` | 193 | 22:26:47 |
| `web/src/panels/HistoryPanel.tsx` | `72CB64CEED45DD7FE9596B9EB2DE80673076A1AB8C580FE1434EE4509B1C9193` | 69 | 16:24:33 |
| `web/src/panels/Scrubber.tsx` | `3329167DD0A8CF929E5991959161E6934338E16165E838481BA7269E9D9ACAEB` | 76 | 21:54:07 |
| `web/src/panels/HudBadge.tsx` | `07EC7C267F68A22AAF9441CB1DE710A976B33E8103744A1BB0CD769449138CF2` | 63 | 23:36:00 |
| `web/src/panels/ToastRegion.tsx` | `CB4AC99C047AC7A0AC54904920F919B09E582CC33A43C1A965C0F6AD2DEE1BBB` | 49 | 23:35:00 |
| `web/src/state/strings.ts` | `2ACDB02DC770BCBC8A3E7A69EFAE2304539031B95D3B47F958911DB3BA044C7F` | 440 | 17:32:22 |
| `backend/.../hiverouter-core/src/http.rs` | `A3E52E55AC387B39B8A7336D2A7572E3E12B5315F5AA31EB6C008ABA3EC431CC` | 1251 | 23:33:53 |
| `backend/.../hiverouter-core/src/ws.rs` | `6B64424229EDDB72EA63CB012408683197035E604FC054F8A937D5A5249005AA` | 1366 | 23:30:27 |
| `backend/.../hiverouter-core/src/execution.rs` | `F02F032A99CAD91E593CCB0140423AB990112515C86F7F417D9345E945A53AC0` | 3069 | 22:16:37 |
| `backend/.../hiverouter-core/src/graph.rs` | `8B3A24A29E425EE41ABD2F5DF7CBEA46E3326797185C8D2EDA5620E9FC1680C3` | 411 | 21:44:36 |
| `backend/.../hiverouter-core/src/snapshot.rs` | `2954FF374BE2DBC6E4E42D0D396795DF35864D8FFDA2F57E97413EC29C9F3398` | 305 | 22:16:44 |
| `backend/.../hiverouter-core/src/history.rs` | `FC236BCBB0D4DCE3B6C49F18A15CDC40007EDF842CB4BE8C1B971997AD0ED026` | 116 | 05:52:09 |

HEAD: `240fe0fd5d9b5a0845e670da51e7d75bfb1e934a` (2026-11-07 23:10:02).
Working tree at freeze: 11 modified files (`http.rs`, `ws.rs`, `ChatCore.tsx`, `Canvas.tsx`, `graph.ts`,
`store.ts`, `HudBadge.tsx`, `ToastRegion.tsx`, `ApprovalOverlay.tsx`, `AgentProfilesPanel.tsx`,
`tools/desktop-launcher/HiveMind.exe`) plus this report's directory.

### 0.2 Build / served identity

| Artifact | Value | Evidence |
|---|---|---|
| Served asset | `assets/index-CmOwogPv.js` | captured from B2 served HTML (`probeB2.json`) |
| Bundle B2 | 767,247 B, 2026-11-07 23:39:33, SHA-256 `5B0E9BD0A56CBA14CBD08A27F360673CBD5D87B4E56854306C46B21AEDF7B57A` | `Get-Item`/`Get-FileHash` on `web/dist` |
| Binary B2 | 15,148,544 B, 23:43:15, SHA-256 `209A79DA9C4527048AD94E2AE1693C60C36344965B8ADE8479D4BF6421492A92` | `dist/HiveMind/hiverouter.exe` |
| `/health` B2 | 200 `degraded`, version `1.2.0-dev`, schema `hiverouter.v1`, db schema 10, graph_nodes 35, key `E_KEY_MISSING` | `probeB2` at ~23:44:37 |
| Run set | 6 runs; newest `01a1b9a8-89a7-71c4-baf2-a45b7e6119d1` **failed/E_KEY_MISSING**, then `01a1b931-…` completed (8 revisions), `01a1b931-2a4f-…` completed (10 revs), `01a1b930-…` failed/E_BUDGET_EXCEEDED, etc. | `/api/history/runs` |

B2 was built from W1: latest source write 23:38:48 → bundle 23:39:33 → binary 23:43:15. This makes B2
the first build in this wave that can legitimately be compared to the current source. The earlier B1 build
(bundle `index-CKsw0WPr.js` 23:25:52, binary 23:29:30) and B0 build (23:06/23:08) are superseded; their
captures are used only where the cited lines are unchanged.

### 0.3 Live window / captures (read-only)

| Time (local) | Build | Probe | Evidence |
|---|---|---|---|
| 23:16–23:22 | B0 | chat hide/show, reload persistence, first History replay | `%TEMP%\orca-a\probe1/probe2` |
| 23:29–23:39 | B1 | `Hide finished` boot, skipped drag locator, keyless run | pid 22380 |
| 23:33:59 | B1 | empty-prompt Enter → `E_PROMPT_EMPTY` toast; no `SUBMIT_PROMPT` frame | probe5 |
| 23:36:32 | B1 | driver seat's keyless submit created run `01a1b9a8` (not this seat) | `05-driver.md` seq 10–11; `/api/history/runs` |
| 23:44:37 | **B2** | health, boot, hide chat, History tab | `probeB2` |
| 23:45–23:46 | **B2** | old-run `HISTORY_REVISION_SET` (4 nodes/3 edges), current-run replay 0, Return live | `probeB2b` |
| 23:48 | — | server down (not this seat) | `Get-Process`/port probe |

Commands used: only GETs to `/`, `/health`, `/api/session`, `/api/history/runs`,
`/api/history/runs/{id}/revisions`, `/api/history/runs/{id}/state?revision=…`; DOM/ARIA reads;
one second WebSocket that sent two read-only `HISTORY_REVISION_SET` commands (a replay read).
No prompt was submitted, no Connect/Template/Approval/Cancel/Stop/Delete control was clicked.
Playwright probe scripts live outside the repo in `%TEMP%\orca-a\` and mutate nothing durable.

### 0.4 Evidence classes

- **VERIFIED** — observed on B2 with the quoted DOM/WS/HTTP result.
- **VERIFIED-B1** — observed on B1; only used where the cited source lines are identical in W1/B2.
- **READ** — read from W1 source; not exercised live.
- **INFERRED** — follows from source + a captured fact, but the final control was not exercised.
- **UNVERIFIED** — no safe live path was available; listed in §6.

---

## 1. Top 10 (ranked)

| # | P | ID | Defect | Status | Primary seam |
|---|---|---|---|---|---|
| 1 | **P0** | TR-01 | Keyless `SUBMIT_PROMPT` is ACKed and creates a run that later fails `E_KEY_MISSING`; the frozen contract makes `E_KEY_MISSING` a command refusal. The new client guard is dead on boot (`keyPresent === null`). | VERIFIED B2 | `execution.rs:1016-1053`; `ws.rs:743-751`; `ChatCore.tsx:138`; `store.ts:372,1334,1408-1440,1670` |
| 2 | **P0** | TR-02 | A terminal run failure is silent when chat is hidden: no toast/alert, the canvas says "No agents yet — submit a prompt to create the first run", and ControlBar shows "Unavailable". Only ChatCore's banner names the failure. | VERIFIED B2 | `store.ts:1408-1440`; `ChatCore.tsx:225-234`; `Canvas.tsx:785-790`; `ControlBar.tsx:118-124`; `strings.ts:197,193` |
| 3 | **P1** | TR-03 | History is single-run only: no run picker; after `01a1b9a8` appeared, the completed `01a1b931` (8 revisions) became unreachable. `HistoryPanel` drops `created_at`/`run_status`, so a failed revision has no failure banner and the panel reads "1 revisions". | VERIFIED B2 | `store.ts:531-554,662-674,1059`; `HistoryPanel.tsx:20-52`; `Scrubber.tsx:61-65` |
| 4 | **P1** | TR-04 | Replay is presented with live-scoped counts: Canvas count badge and minimap-empty class use `visibleTasks`/`tasks`, and the Active Agents rail uses live `tasks`. HUD is fixed, these three are not. | READ W1 + VERIFIED-B1 | `Canvas.tsx:640,696`; `LibraryPanel.tsx:40-42,106`; `HudBadge.tsx:9,39` (fixed) |
| 5 | **P1** | TR-05 | Chat Hide/Show is global and remains active on the History tab, where `ChatCore` is not mounted: toggling it changes state and the header label but no chat can appear. | VERIFIED B2 | `App.tsx:55-87`; `Header.tsx:50-67`; `Canvas.tsx:642-649` |
| 6 | **P1** | TR-06 | Read-only replay is not enforced in the store/unguarded controls: Library TEMPLATE_INSERT is clickable during replay, and a `pendingReparent` dialog opened before replay can be confirmed afterwards; store actions have no replay guard. | READ W1 | `LibraryPanel.tsx:137-145`; `store.ts:2149-2154,1912-1966`; `Canvas.tsx:284-292,579-606` |
| 7 | **P1** | TR-07 | Active Agents rail excludes the hyperagent, silently caps at 8, has no selected state, and reads live tasks during replay. | READ W1 + VERIFIED-B1 | `LibraryPanel.tsx:40-42,100-131` |
| 8 | **P1** | TR-08 | Run-terminal refusal identity is lost for most server codes: 16 client-side codes vs 65 server codes; `E_BUDGET_EXCEEDED` / provider codes render "Unrecognised server refusal (…)" and "record the frame". | READ W1 | `strings.ts:312-393`; `refusals.ts:25-45`; `ChatCore.tsx:91`; `store.ts:1422` |
| 9 | **P1** | TR-09 | Canvas filter state is not durable/coherent: "Hide finished" hides every terminal state and resets on tab change; after a pane-deselect `focusBranch` stays `aria-pressed=true` and disabled, with no way to clear it. | VERIFIED-B1 + READ W1 | `Canvas.tsx:112,172-200,624-638,684` |
| 10 | **P1** | TR-10 | Drag/confirm (active runs only in W1): confirmation copy is false about scope, source/target kinds and the contract's "worker not streaming" precondition are unenforced, the dropped offset persists before confirm, and the dialog has no role/focus/Escape. | READ W1; live UNVERIFIED | `Canvas.tsx:150-170,249-292,579-606,674-681`; `store.ts:704-721,1912-1966`; `ws.rs:650-735,340-357`; contract `UI-COMMANDS-v1.md:34` |

P0 = 2, P1 = 8. The P2 appendix is in §4.

---

## 2. Trace ladders

Each ladder records the true value reaching the next hop. A chain is "wired" only when the last hop renders
what the store actually received — not merely that a control exists.

### 2.1 Chat composer and send / ACK / refusal

| Hop | Where (W1) | What actually happens |
|---|---|---|
| Render | `ChatCore.tsx:337-352` textarea; `:353-375` stop/submit; `:333` form; `:344-349` Enter | Composer exists; Submit disabled on `connection !== open || replaying || submitting || draft.trim()===''` (`:370`); Stop disabled unless running/streaming (`:357`) |
| Handler | `ChatCore.tsx:133-184` `submit()`; `:186-189` `onSubmit` | Synchronous `inFlightRef` guard; `keyPresent===false` pre-check (`:138`); local prompt validation (`:151`); baseline before dispatch (`:158`); ACK/refusal settlement (`:165-183`) |
| Client | `store.ts:1873-1886` `submitPrompt` | Validates, appends optimistic user row (`:1879-1884`), awaits `run('SUBMIT_PROMPT',…)`; **returns void, so ACK and refusal look the same to the caller** |
| Transport | `api/client.ts:455-501` `send`; `:373-400` `routePending` | Resolves only on a matching `ACK`; rejects on typed `ERROR` or 15 s timeout. No fake success. |
| Request | `SUBMIT_PROMPT` `{text, project_id}` | ACK is published before/independent of provider work |
| Route | `ws.rs:743-751` | Calls `engine.submit_prompt`; `Ok(_run)` → ACK; `Err(refusal)` → ERROR. **No key preflight.** |
| Service | `execution.rs:1016-1053` | `create_run` (`:1035`), receipt, user message, `set_active`, `RUN_CREATED`, spawn `dispatch`. `UnconfiguredExecutor` refuses `KeyMissing` only later, inside `dispatch` (`execution.rs:287-313`, `1267-1272`, `1294-1299`). |
| Store | `store.ts:1312-1332` ACK; `:1332-1343` ERROR; `:1408-1440` RUN_STATUS | ERROR → toast. RUN_STATUS failure → `run.errorCode` only; **no toast**. `settings.keyPresent` is not set on RUN_STATUS. |
| Response | live `01a1b9a8` | Run row `failed`, `error_code=E_KEY_MISSING`, created `2026-11-08T05:36:32Z`; revision 0 snapshot empty; no tasks; no metrics. |
| Render-back | `ChatCore.tsx:225-234` banner; B2 DOM | Chat shows **RUN REFUSED / E_KEY_MISSING / No provider key… / how_to_fix**; user message "keyless check" remains; draft is cleared (ACK path). Hide chat → **no alert anywhere** (TR-02). |
| Re-entry | B2 boot | `performSnapshotRefresh` loads the failed run + messages + 1 revision; banner returns only when chat is visible. |

**Verdict: WIRED-BUT-LYING (ACK ≠ acceptable).** The transport is honest (ACK on accept, ERROR on refusal),
but `SUBMIT_PROMPT` accepts a keyless prompt and only fails asynchronously. The frozen contract lists
`E_KEY_MISSING` in the `SUBMIT_PROMPT` named-refusal column (`docs/contracts/UI-COMMANDS-v1.md:32`);
`execution.rs:1015` itself omits it. The live failed run is the proof.

**Negative paths exercised**
- Empty prompt via Enter: VERIFIED B1 — `E_PROMPT_EMPTY` toast, zero `SUBMIT_PROMPT` frames (`submitFrames.before=0, after=0`).
- Whitespace-only + Enter: READ — `validatePromptText` (`contract.ts:1051-1060`) only rejects length 0; the Submit
  button is disabled on `draft.trim()===''`, but the Enter handler calls `submit()` directly, so `"   "` would pass
  client validation and be sent. P2.
- Double submit: guarded by `inFlightRef` (`ChatCore.tsx:134-136,159-164`); the store has no duplicate guard.
- Server `E_RUN_ACTIVE` / `E_PROMPT_TOO_LONG`: READ; ERROR path is wired and toastable; not exercised.

**Fix seam:** preflight key availability before `create_run` in `execution.rs:1029-1035` (or in `ws.rs:743`
before calling the engine) and return `Refusal::KeyMissing`; on the client, set
`settings.keyPresent=false` in the `RUN_STATUS` handler when `error_code==='E_KEY_MISSING'` so the new
`ChatCore.tsx:138` guard works on the first submit. Keep the draft on refusal.

### 2.2 Hide/Show chat and Return live

| Hop | Where (W1) | What actually happens |
|---|---|---|
| Render | `Header.tsx:50-67` button; `Canvas.tsx:642-649` pill; `ChatCore.tsx:113-123` floating reopen; `ChatCore.tsx:217` × | Three affordances for one boolean; text/aria switch Hide↔Show; `aria-pressed` tracks it |
| Handler | `setChatVisible` `store.ts:2218-2225` | Writes `localStorage['hivemind.chat.visible.v1']`, then state |
| Store init | `store.ts:395-402,1786` | Boots from localStorage; default true; persists |
| Render-back | B1 reload | Hide → no chat section, header "Show chat", Canvas "Show chat"; reload → still hidden; Show → chat returns. VERIFIED B1 (source unchanged in B2). |
| Return live | `Header.tsx:41-49`; `Scrubber.tsx:66-73`; `HistoryPanel.tsx:57-64`; `store.ts:2179-2193` | Three exits from replay; `exitHistoryReplay` clears `replayGraph`, sets `selected=latest`, sends no command |
| Re-entry | B2 | Header "RETURN LIVE" appeared during replay; click → banner gone, HUD back to "NO METRICS"; VERIFIED B2 in the current run's replay. Scrubber values restore per run (`1 of 1` for the failed run). |

**Verdict: WIRED-AND-WORKS on Workspace; WIRED-BUT-NO-OP on History; one visual overlap in B2.**

- **TR-05 (P1):** `App.tsx:55-87` mounts `ChatCore` only in `workspaceView`; the History branch (`:55-81`)
  mounts `HistoryPanel + Canvas`. The Header toggle (`Header.tsx:50-67`) and Canvas toggle
  (`Canvas.tsx:642-649`) are always rendered. B2 History DOM: `chat:false` (no section), Header
  "Hide chat" `aria-pressed=true`; toggling changes the label to "Show chat" but no chat exists. A user can
  "hide" a chat that is already absent, or "show" it with no visible result; the state then affects the
  Workspace after a tab switch. Fix: condition both toggles on `nav==='workspace' && tab==='workspace'`, or
  render a read-only chat placeholder in History; don't let a view-scoped surface claim a global toggle.
- **P2-1 (B2 visual):** when chat is hidden, `ChatCore.tsx:115` renders the reopen pill at `top-4 z-30`,
  while `HudBadge.tsx:15` renders at `top-3 z-20`; both are centered. B2 screenshot `b2-hidden.png` shows
  "SHOW CHAT" covering the right half of `HYPERAGENT · NO METRICS`. The z-30 fix made the reopen reachable
  but now occludes the HUD. Move the pill below the HUD (`top-20`) or drop it (Header/Canvas already offer Show).
- **P2-3:** `Scrubber.tsx:66-73` and `HistoryPanel.tsx:57-64` both expose a button named `live` (same string
  `STRINGS.scrubber.live`), with no distinct accessible name and the same disabled state. In History there are
  two indistinguishable `live` buttons.
- No persistence defect: localStorage round-trips correctly; `chat.pinned` is dead state (`store.ts:2142-2147`,
  strings `chat.pin/unpin/pinned`), with no caller and no UI. P2-6.

### 2.3 Active Agents rail

| Hop | Where (W1) | What actually happens |
|---|---|---|
| Render | `LibraryPanel.tsx:100-131` | Heading `ACTIVE AGENTS`; up to 8 hex chips with a state dot and a short label |
| Data | `LibraryPanel.tsx:40-42` | `Object.values(tasks).filter(task => task.kind !== 'hyperagent' && !isTerminal(task.state))` |
| Handler | `:111` → `selectTask` (`store.ts:2196-2198`) | Sets `selectedTaskId` only |
| Render-back | Canvas `Frame` (`nodes.tsx:84-94`) | Selected border/shadow on the canvas card; the rail chip itself has no `aria-pressed` and no selected class (`:108-127`) |
| Replay | Canvas `:201-219,436-498` | Replay uses `replayTasks`; the rail still uses live `tasks` |

**Verdict: WIRED-BUT-PARTIAL.** B1 (3 completed tasks) and B2 (0 tasks) both read "No active agents", which is
correct for the live map. The defects are structural:

- **TR-07 (P1):** the hyperagent is excluded, so a run whose only non-terminal row is the orchestrator shows
  "No active agents" while the run is active. The rail is titled agents, not workers.
- **Silent 8 cap** (`:106`): `MAX_AGENTS=64` (`caps.rs:13`); 9–64 active tasks show 8 with no `+N more`.
- **No selected state**: a chip click selects but provides no visual/a11y confirmation; a terminal run's cards
  are now also non-selectable at the React Flow level in W1 (`interactiveGraph`), so the rail is one of the few
  selection paths — and it gives no feedback.
- **Live-scoped in replay**: B1 replay of 35 nodes showed the rail empty (live tasks were all completed) while
  the canvas showed historical work. Use `history.replayGraph ? replayTasks : tasks`.
- Fix seam: `LibraryPanel.tsx:40-42,106,108-127`; add `aria-pressed`, a selected chip style, `+N more`, and a
  root/owner chip or rename to "Active workers".

### 2.4 History replay and revision scrubber

| Hop | Where (W1) | What actually happens |
|---|---|---|
| Render | `HistoryPanel.tsx:18-68`; `Scrubber.tsx:10-75` | Revision list + range input + live exit; revision labels only number + LATEST/COMMITTED |
| Handler | `store.ts:2156-2176` `setHistoryRevision` | Validates `run` and `revisions.includes(revision)`; if empty, refreshes snapshot; sends `HISTORY_REVISION_SET` |
| Client/Request | `api/client.ts:455-501`; `run()` `store.ts:1730-1763` | ACK/ERROR; then server event |
| Route | `ws.rs:832-843` | `history.state_at`; now `history_graph_value(state, run_id, envelope.graph)`; ACK then `HISTORY_SNAPSHOT` |
| Service | `http.rs:700-740` | **B2**: one run-scoped, identity-enriched projection for HTTP and WS; fail-closed filter (`:709-713`); role→kind + label from task rows (`:718-729`) |
| Store | `store.ts:1702-1712` | Payload `run_id` gate (`:1703-1707`); then inserts revision, sets latest/count/selected/replayGraph |
| Response | live B2 | Old run `01a1b931` rev 6 now returns **4 nodes, 3 edges**, kinds `worker,worker,worker,hyperagent`, labels `worker 15f5a82c / b087b93d / e6db997e / HyperAgent 3d23430f`, parents all root/null. VERIFIED B2. |
| Render-back | `Canvas.tsx:431-498,690` | `replayNodes` + `HIVE_ANCHOR` when a hyperagent is present; `replayOwnerLinks`; auto-fit 0.75; banner |
| Re-entry | B2 | Current run rev 0 → "Replay (read-only) · Routing state 0", HUD "REPLAY · LIVE METRICS HIDDEN", 0 nodes; Return live restores. VERIFIED B2. |

**Verdict: WIRED-AND-WORKS for the projection and replay mechanics; PARTIAL for history as a feature.**

The 23:30–23:36 fix is real and live: the WS replay now returns the same scoped/enriched graph as HTTP.
The B1 failure (35 nodes/26 edges/0 owner links/all "Agent …") was the old raw `ws.rs` serialization and is
not an open source defect in B2. The remaining defects are:

- **TR-03 (P1) — no run picker.** `/api/history/runs` is fetched only inside `performSnapshotRefresh`
  (`store.ts:1060`); `chooseHistoryRun` (`:662-674`) picks the active run, else the newest, and nothing renders
  the list. B2 live: the new failed run `01a1b9a8` owns the History tab (1 revision); the completed
  `01a1b931` run with 8 revisions is in the API but unreachable in the UI. I could reach it only by a raw WS
  command; the payload gate (`store.ts:1702-1708`) ignored that foreign snapshot live: the old 4-node revision
  arrived, but the app's history stayed at `1 of 1` (`max=0`) and the replay stayed on the current run. This is
  not a cosmetic gap: any new run
  (even a keyless failure) makes all prior graphs and transcripts unreachable.
- **TR-03 — metadata is dropped.** `decodeHistoryRevisions` (`store.ts:531-554`) keeps only revision numbers;
  the API sends `created_at` and `run_status`. `HistoryState` (`:149-162`) has no metadata; `HistoryPanel`
  (`:26-52`) and `Scrubber` (`:61-65`) render only a position. B2 live failed-run History reads
  `History 01a1b9a8 1 revisions rev 0 LATEST live` — no `failed`, no `E_KEY_MISSING`, no timestamps.
  A failed revision looks like a committed one; replaying it shows an empty canvas with no failure banner.
- **P2-2:** "1 revisions" is not pluralized (`HistoryPanel.tsx:28`).
- **P2-3:** duplicate `live` buttons (Scrubber + HistoryPanel).
- **P2-9:** `snapshot.status/refusal` and `history.refreshStatus/refreshRefusal` have no UI consumers
  (`store.ts:160-161,187-198`; `grep` across `web/src` returns only definitions). A failed boot/resync
  hydration renders as Connected + "No run yet"; a failed revision refresh renders as "No committed revisions".
- **P2-10:** `history_graph_value` uses `tasks_for_run(run_id).unwrap_or_default()` (`http.rs:705`), so a DB
  error becomes an empty allow-set and a silently empty graph, not a refusal.
- **P2-11:** `HISTORY_SNAPSHOT` is published with `publish_with_correlation` → `publish_for_run(event, None, …)`
  (`ws.rs:934-937`), so the envelope `run_id` is `null`. The outer guard `store.ts:1318-1328` therefore never
  fires for `HISTORY_SNAPSHOT`; the payload gate `:1702-1708` does. Keep the payload gate; either set the
  envelope run id or delete the dead branch to avoid a future "guard exists" false assumption.
- The replay auto-fit is pinned to 0.75 in both `Canvas.tsx:365-366` and `:690`. With the new run-scoped
  projection (4 nodes) this is fine; for a 64-agent run it cannot fit (manual "Fit swarm" still can, because it
  calls `fitView` without the pin). P2-15.

### 2.5 Graph cards, ownership links, visibility

| Hop | Where (W1) | What actually happens |
|---|---|---|
| Data | `Canvas.tsx:172-200` | `visibleTasks` = all, branch, or non-terminal depending on `hideFinished`/`focusBranch` |
| Layout | `graph.ts:43-127` | `columnHiveLayout` when a root hyperagent exists; otherwise dagre; orphan slots (`:104-115`) |
| Nodes | `Canvas.tsx:202-247`; `graph.ts:226-248` | Hyperagent is filtered out of cards; `liveNodes` adds a 1×1 `HIVE_ANCHOR` only when a hyperagent is visible; `draggable/selectable = interactiveGraph` |
| Edges | `graph.ts:250-273` | Parent→child; root children are redirected to `HIVE_ANCHOR_ID` |
| Ownership links | `Canvas.tsx:314-351` (`ownerLinks`), `:458-498` (`replayOwnerLinks`), `:498,521-548` | Fixed-center radial dashed lines to **direct root children only**; replay now has its own copy; hidden when no root is visible |
| Cards | `nodes.tsx:141-243` | SuperAgent/Worker/Tool/BrokenRoute; metrics absent render `—` |
| HUD | `HudBadge.tsx:8-46` | Uses live `tasks`/`metrics`; B2 shows `REPLAY · LIVE METRICS HIDDEN` during replay (fixed) |
| Count / minimap | `Canvas.tsx:640,696` | `visibleTasks/tasks` and `hive-minimap-empty` use live tasks — **not replay-scoped** |
| Empty | `Canvas.tsx:785-790` | `tasks.length===0` → `STRINGS.panels.agentsEmpty` ("No agents yet — submit a prompt…") regardless of `run`/`snapshot` |

**Verdict: WIRED for live cards/edges/links; TR-04/TR-09 for replay and filter truth.**

- **B1 live (same count/minimap code):** replay rev 6 rendered **35 cards**, count badge `0/4`, minimap
  "NO NODES YET", HUD `3 WORKERS · 100% GREEN`. The screenshot `replay6b.png` captures it. B2 fixed the graph
  and HUD, but the badge/minimap/rail selectors are unchanged.
- **B1 live (same owner-link code):** live 3-worker run: 4 nodes, 3 edges, 3 owner links. The live ownership
  links are real. In B1 replay there were 0 because the raw graph had no root/kind; W1 adds
  `replayOwnerLinks` + `HIVE_ANCHOR`. **UNVERIFIED** live on B2 because no non-empty replay is reachable
  through the UI (see TR-03), and the payload gate (`store.ts:1702-1708`) ignored the foreign run live.
  Do not claim the replay link rendering as proven until a run picker exists.
- **TR-04:** use the displayed projection:
  `const display = history.replayGraph ? replayTasks : visibleTasks;` for `:640`, and
  `Object.keys(history.replayGraph ? replayTasks : visibleTasks).length===0` for `:696`.
- **TR-09:** `hideFinished` (`:112`) filters `isTerminal` (`:196-199`), i.e. failed/killed/cancelled/interrupted
  as well as completed, under a label "Hide finished" (`:622`). It is local component state, so a tab switch
  unmounts `Canvas` and resets it (B1: toggle off → History → Workspace → `0/4` again). `focusBranch`
  (`:113,624-638`) is disabled when `selectedTaskId===null` but `aria-pressed` keeps its value; B1 proved:
  select node → Focus branch (pressed, 1 card) → pane click deselects → button `pressed=true disabled=true`,
  all 4 tasks visible again, and the user cannot turn focus off without re-selecting a node. Fix: persist
  filter state in the store/localStorage, reset `focusBranch` when selection clears (or keep the toggle
  enabled), rename to "Hide terminal" or filter only completed/cancelled.
- **Failed card representation:** `BrokenRouteNode` (`nodes.tsx:226-243,282`) has no producer. The contract's
  `NODE_KIND_VOCABULARY` excludes `error` (`event.rs:240`, test `:915-923`); `NODE_*` has no `error_code`
  (`event.rs:248-271`). A failed task renders as a Worker/SuperAgent card with a signal dot and no code.
  P2-7.
- **SuperAgent "workers" row** (`nodes.tsx:165-168`) labels a progress percentage as `workers`; `progress` is
  not on the wire (`event.rs` has no progress field), so it currently renders `workers —`. Latent mismatch,
  P2-8.
- **Ownership depth:** links only for `task.parentId === root.id` (`Canvas.tsx:325-328`), so workers under a
  SuperAgent get normal bee edges but not ownership lines. P2-13.

### 2.6 Drag target and confirmation

| Hop | Where (W1) | What actually happens |
|---|---|---|
| Render | `Canvas.tsx:651-684` ReactFlow with `onNodeDrag`/`onNodeDragStop`; `:579-606` dialog | Cards are draggable only when `interactiveGraph` is true (`:125,206`) — i.e. an active run; replay/terminal cards are inert |
| Handler | `Canvas.tsx:150-170` `findDropTarget`; `:249-280` drag handlers; `:284-292` `confirmReparent` | On drag: `getIntersectingNodes` overlap, target highlight + green line; on drop: persist `positionOverrides` **before** the dialog; if a target, open `pendingReparent` |
| Client | `store.ts:1912-1966` `rewireSelected` | `rewireBlocker` checks run/selection/target/rev (`:704-721`); terminal run → `run-terminal` refusal; unknown rev → one bounded HTTP read; sends `REWIRE_TASK` with `expected_rev` |
| Route | `ws.rs:650-735` | `graph.rewire` (cycle/rev) then `apply_rewire_context`; ACK + `NODE_REWIRE`; a context failure publishes ERROR after the graph committed (fails loudly) |
| Service | `graph.rs:141-201`; `ws.rs:340-357`; `context.rs:137-186` | Cycle/rev checks; **no kind/streaming check**; clears old parent's scoped facts, injects new parent's facts, keeps the task's own allowed tools/schema |
| Store | `store.ts:1472-1525` | `NODE_REWIRE` updates parent/rev; `pendingRewire` fallback only if the payload omits the parent |
| Render-back | `Canvas.tsx:207-220,521-548` | Card stays where dropped (override), edge re-parents, owner links recompute; selection set to the dragged task |
| Re-entry | B1/B2 | B1 live: 3 terminal cards; dragging terminal cards was possible in B1, but W1 `interactiveGraph` disables it. B2 latest run has 0 cards. **No live drag was performed** (see §6). |

**Verdict: source-wired for active runs; confirmation exists; the semantic contract is still unenforced.**

- **TR-10 (P1):**
  - **Copy is false.** The dialog (`Canvas.tsx:586`) says "Memory, tool and message scope changes are not
    applied yet; the server rewire is the first step." `ws.rs:340-357` **does** clear the old parent's scoped
    facts and inject the new parent's facts (and `context.rs:137-186` switches the parent pointer). Tool ids/
    schema are kept (that half is true), but "memory … not applied" is not. "The card moves with this
    connection" is also false: the position override is saved before the confirm (`Canvas.tsx:674-681`), so the
    card stays at the drop point.
  - **Kinds/precondition unenforced.** `findDropTarget` accepts `superagent|hyperagent|worker` targets and any
    non-hyperagent source; `ControlBar.canRewire` requires a worker (`ControlBar.tsx:33-35`); `rewireBlocker`
    does not check kind/state; the server `graph.rewire` (`graph.rs:141-201`) checks only existence/rev/cycle.
    The frozen contract says "worker not streaming" (`docs/contracts/UI-COMMANDS-v1.md:34`). A streaming worker
    can be rewired; a superagent can be dragged under a worker via the canvas path.
  - **Target selection is overlap, not a radius/path gate** (`getIntersectingNodes`), so ordinary repositioning
    that overlaps a sibling opens a re-parent confirmation. Confirmation mitigates; still a false affordance.
  - **Dialog semantics:** no `role="dialog"`, no `aria-modal`, no focus move/trap, no Escape, Connect not
    disabled in flight. `aria` users get a plain div.
  - **Replay hole:** `pendingReparent` is not cleared by `exitHistoryReplay` or `setHistoryRevision`; a dialog
    opened during an active run can be confirmed after entering replay. `confirmReparent` has no replay guard.
  - **Fix seam:** add a `replayBlocker`/`rewireBlocker` kind+state check used by both ControlBar and Canvas;
    enforce "worker & !streaming" in `graph.rewire`/`ws.rs` (server authority); restrict `findDropTarget` to
    superagent/hyperagent; only persist the drop position after a successful rewire (or revert on cancel);
    replace the copy with the real effect; add dialog role/focus/escape and a Connect in-flight guard.
  - **Status:** READ; the live drag/confirm path was **not** exercised because B2 had no active run and no
    cards. Do not close this item on a screenshot alone.

---

## 3. Event, payload and terminal truth

### 3.1 Emitter → subscriber parity

The app uses one WebSocket with a single JSON dispatcher (`api/client.ts:346-370`), not named SSE frames.
For the six families the server events are `ACK`, `ERROR`, `RUN_CREATED`, `RUN_STATUS`, `REPLY_DELTA`,
`REPLY_DONE`, `NODE_SPAWN/DRAG/REWIRE/STATE`, `HISTORY_SNAPSHOT`, `SERVER_HELLO`, `RESYNC_REQUIRED`.
All have handlers in `store.ts:1318-1727` (ACK ignored, ERROR toasted, RUN_STATUS/REPLY/NODE/HISTORY switch).
No dropped named event was found in these families.

### 3.2 Payload-shape parity

- `HISTORY_SNAPSHOT` B2 payload: `graph.nodes[]` now contain `task_id,parent_task_id,state,rev,role,kind,label`
  (verified live: 4 nodes, 3 edges, kinds/labels above). The client `buildReplayTasks` reads `kind`/`label`
  (`graph.ts:279-313`), so B2 replay identity is correct. `graph.edges` is not used by the client; it rebuilds
  from parents, which is consistent for the scoped graph.
- `NODE_*` payloads carry `rev`,`kind`,`parent_task_id` (`event.rs:248-271`); `store.ts:1480-1498` consumes
  them. No `progress`/`error_code` fields exist on the wire, which is why the cards show `—`.
- The `HISTORY_SNAPSHOT` **envelope** `run_id` is `null` (`ws.rs:934-937` → `bus.rs:129-138`), while the
  **payload** `run_id` is correct. The store has two gates: the envelope gate (`:1318-1328`) is dead for this
  event; the payload gate (`:1702-1708`) works. See P2-11.

### 3.3 Terminal record truth — run `01a1b9a8`

Captured at 23:36–23:46 local / `2026-11-08T05:36–05:46Z`.

| Field | Value | Source |
|---|---|---|
| `runs.status` / `error_code` | `failed` / `E_KEY_MISSING` | `/api/history/runs` |
| created_at | `2026-11-08T05:36:32Z` | same |
| revisions | 1 (`revision 0`, `run_status: failed`) | `/revisions` |
| snapshot graph | `nodes:[]`, `edges:[]`, `toposort:[]` | snapshot file / `/state?revision=0` |
| tasks / metrics | 0 tasks; HUD `NO METRICS`; graph `0/0` | B2 DOM |
| messages | user "keyless check"; no assistant | B2 chat DOM |
| chat banner | `RUN REFUSED / E_KEY_MISSING / No provider key… / how_to_fix: Set DEEPSEEK_API_KEY…` | B2 DOM, screenshot `keyless-boot.png` |
| toast region | empty (the banner is inside ChatCore, also `role=alert`) | B2 DOM |
| chat hidden | **no error anywhere**, canvas says "No agents yet — submit a prompt to create the first run", ControlBar `01a1b9a8 · Unavailable` | B2 DOM, screenshot `b2-hidden.png` |
| scrubber | `1 of 1` enabled | B2 DOM |
| History panel | `History 01a1b9a8 1 revisions rev 0 LATEST live` — no failure marker | B2 DOM |

This one run simultaneously proves TR-01 (the run exists despite the key refusal), TR-02 (silent when chat
hidden), TR-03 (history has no run picker/status), and P2-2 (grammar).

### 3.4 Refusal identity

The transport carries full server refusals for ERROR frames (code/message/how_to_fix/retryable), and
`applyEvent` uses them directly (`store.ts:1332-1343`). The loss is on **run-terminal** errors:
`RUN_STATUS` carries only `error_code` (`event.rs` / `store.ts:1422`), and `ChatCore` builds the copy locally
with `clientRefusal(runErrorCode)` (`ChatCore.tsx:91`). The local table has 16 codes
(`strings.ts:312-393`); the contract has 65 product codes (`refusal.rs:75-279`). For
`E_BUDGET_EXCEEDED`, `E_CALL_LIMIT`, `E_PROVIDER_AUTH`, `E_PROVIDER_TIMEOUT`, `E_AGENT_LIMIT`,
`E_FANOUT_LIMIT`, `E_SPAWN_DEPTH`, `E_CYCLE`, `E_REWIRE_IN_FLIGHT`, `E_TASK_TERMINAL`, `E_APPROVAL_*`,
`E_TEMPLATE_*`, `E_PATH_*`, `E_FILE_*`, `E_SNAPSHOT_CORRUPT`, `E_HISTORY_GAP`, `E_INTERNAL_ERROR`, etc.,
the banner reads "Unrecognised server refusal (CODE)" with "Record the frame and report a contract gap; do not
retry blindly." That is a lying how-to for a known product failure. The live DB already contains
`E_BUDGET_EXCEEDED` failed runs. TR-08; fix in `strings.ts` and/or carry the message on `RUN_STATUS`.

---

## 4. P2 appendix (recorded, not in the top 10)

| ID | Defect | Where | Status |
|---|---|---|---|
| P2-1 | Chat reopen pill (`top-4 z-30`) covers the HUD (`top-3 z-20`) when chat is hidden | `ChatCore.tsx:115`; `HudBadge.tsx:15` | VERIFIED B2 (`b2-hidden.png`) |
| P2-2 | "1 revisions" grammar; no timestamps/status in History | `HistoryPanel.tsx:28`; `store.ts:531-554` | VERIFIED B2 |
| P2-3 | Two buttons named `live` (Scrubber + HistoryPanel) with no distinct accessible name | `Scrubber.tsx:66-73`; `HistoryPanel.tsx:57-64` | READ |
| P2-4 | Chat Stop disabled during `planning/thinking/queued/awaiting_approval` unless streaming; only `running`/streaming pass | `ChatCore.tsx:357`; `agentState.ts:121-123` | READ |
| P2-5 | Whitespace-only prompt passes client validation via Enter while Submit is disabled on `trim()` | `contract.ts:1051-1060`; `ChatCore.tsx:344-349,370` | READ |
| P2-6 | `pinChat`/`chat.pinned`/pin strings are dead code (no caller, no UI) | `store.ts:2142-2147`; `strings.ts:79-82` | READ |
| P2-7 | `BrokenRouteNode` is unreachable; failed cards show no error code; server has no `error` kind/error_code | `nodes.tsx:226-243,282`; `event.rs:240,248-271` | READ |
| P2-8 | SuperAgent card labels progress as `workers`; progress is not on the wire (renders `—`) | `nodes.tsx:165-168` | READ |
| P2-9 | Snapshot/history error states have zero UI consumers; failed hydration renders as Connected + "No run yet"/"No committed revisions" | `store.ts:160-161,187-198`; `HistoryPanel.tsx:20-24` | READ |
| P2-10 | DB error in `history_graph_value` becomes an empty allow-set → silently empty graph, not a refusal | `http.rs:705-713` | READ; corroborated by 06-logic LOG-02 |
| P2-11 | `HISTORY_SNAPSHOT` envelope `run_id` is null; the outer store guard is dead for the event (payload gate covers and was verified live) | `ws.rs:934-937`; `store.ts:1318-1328` | READ (envelope); payload gate VERIFIED B2 |
| P2-12 | ChatCore keyless toast bypasses `pushRefusal` dedupe/4-item cap and appends newest last | `ChatCore.tsx:141-149` vs `store.ts:803-813` | READ |
| P2-13 | Ownership links are depth-1 only; workers under a SuperAgent get bee edges but no owner line; replay links unverified | `Canvas.tsx:325-328,458-497` | READ |
| P2-14 | Disabled Spawn/Kill keep a strong honey/red fill at 50% opacity and read as enabled | `ControlBar.tsx:60-65,109-116` | VERIFIED B2 screenshot |
| P2-15 | Auto-fit pinned to 0.75; a large live graph can clip (manual Fit swarm still works) | `Canvas.tsx:365-366,690` | READ |
| P2-16 | HistoryPanel/Scrubber/Active rail are live-scoped in replay (see TR-04/TR-07) | see TR-04/07 | READ/B1 |

---

## 5. Exact fix seams (for the forge/lead)

1. **Keyless ACK (TR-01):** `execution.rs:1029-1035` — check the configured provider/key before `create_run`
   and return `Refusal::KeyMissing`; keep the receipt/idempotency path intact. Alternatively preflight in
   `ws.rs:743-747`. Client: `store.ts:1408-1440` set `settings.keyPresent=false` on
   `error_code==='E_KEY_MISSING'`; then `ChatCore.tsx:138` works on first submit. Add a server test: no key →
   ERROR, zero run rows; a client test: key null/true/false.
2. **Silent terminal failure (TR-02):** `store.ts:1408-1440` — `pushRefusal(clientRefusal(error_code))` (or a
   dedicated run banner) for terminal runs with an `error_code`; `ControlBar.tsx:118-124` render the real
   status/error, not `Unavailable`; `Canvas.tsx:785-790` branch on `run===null` vs `run.errorCode` vs
   `snapshot.status`, and use `snapshot.refusal`/`history.refreshRefusal`; `ChatCore.tsx:225-234` can stay as
   the detailed surface.
3. **History run picker/status (TR-03):** keep `created_at`/`run_status` in `HistoryRevisionList`/`HistoryState`
   (`store.ts:531-554,149-162`), render them in `HistoryPanel.tsx:26-52`/`Scrubber.tsx:61-65`, and add a run
   selector from `GET /api/history/runs` (the data is already fetched at `store.ts:1060`); a failed revision
   must say `failed` and, when it has a run error, name the code. Add a multi-run fixture.
4. **Replay counters (TR-04):** `Canvas.tsx:640,696` and `LibraryPanel.tsx:40-42,106` should select from
   `history.replayGraph ? replayTasks : visibleTasks|tasks`; add a `displayedTasks` memo. The HUD fix
   (`HudBadge.tsx:9,39`) is already correct; keep it.
5. **History chat toggles (TR-05):** `App.tsx:55-87` and `Header.tsx:50-67`/`Canvas.tsx:642-649` — gate the
   controls on the workspace view, or mount a read-only history chat; do not let the History view own a
   global chat toggle.
6. **Replay read-only (TR-06):** add one `if (get().history.replayGraph !== null) { pushRefusal(...); return; }`
   at the top of `spawnSubHive/rewireSelected/cancelRun/killWorker/insertTemplate` (`store.ts:1888-1989,2149-2154`);
   disable the Library template buttons (`LibraryPanel.tsx:137-145`); clear `pendingReparent` on replay entry
   (`store.ts:2156-2193` or `Canvas.tsx:284-292`).
7. **Active rail (TR-07):** `LibraryPanel.tsx:40-42` include/denote the root or rename; `:106` add `+N more`;
   `:108-127` add `aria-pressed` + selected style; use the replay projection.
8. **Refusal copy (TR-08):** add the run-terminal codes to `strings.ts:312-393` with real how_to_fix; or add
   `message`/`how_to_fix` to `RUN_STATUS` and render that; unit-test every `Refusal::ALL` code has copy.
9. **Filter truth (TR-09):** persist `hideFinished`/`focusBranch` per run (store/localStorage) instead of
   component state; reset/disable `focusBranch` coherently when selection clears; rename the filter to
   "Hide terminal" or filter only `completed|cancelled`.
10. **Drag (TR-10):** enforce `worker && !streaming` in `rewireBlocker` and `graph.rewire`; restrict
    `findDropTarget` to `superagent|hyperagent`; persist the drop position only after a successful rewire;
    correct the dialog copy; add `role="dialog"`, `aria-modal`, focus/Escape, and a Connect in-flight guard;
    add a server regression for a streaming/kind rewire.

**Verification plan after a fix build:** `/health` keyless; keyless submit must produce ERROR + no run; hide
chat after a failed run must still show a named toast; the History tab must list at least two runs and a failed
revision; replay of a 4-node revision must show 4/4 in the badge/minimap and no live metrics; a 35-node legacy
snapshot must be scoped to 4; the replay owner lines must be present with the anchor; a terminal run must not
offer drag; an active run must refuse a streaming/kind rewire; Return live from both Scrubber and Header.

---

## 6. Done / Not done / Unverified

### Done (evidence in this report)

| Item | Evidence |
|---|---|
| Six-family trace ladder with source paths/lines | §2.1–2.6 |
| Live B2 runtime/deployment identity | §0.2; `probeB2` served asset, `/health`, run set |
| Keyless terminal truth (run `01a1b9a8`) | §3.3; `/api/history/runs`; B2 DOM/screenshots |
| Replay projection fix independently verified | B2 old-run rev6: 4 nodes/3 edges/kinds/labels/parents |
| Cross-run replay guard VERIFIED B2 | Old-run rev6 snapshot (4 nodes) broadcast; app history stayed `1 of 1` (`max=0`) and replay stayed on the current run — the foreign revision was not inserted (`store.ts:1702-1708`) |
| HUD replay label verified | B2 `REPLAY · LIVE METRICS HIDDEN`; `HudBadge.tsx:39` |
| Hide/Show chat + Return live exercised | B1 reload persistence; B2 Return live from rev0 |
| Empty-prompt local refusal exercised | B1 `E_PROMPT_EMPTY` toast; zero `SUBMIT_PROMPT` frames |
| P0/P1/P2 ranked list with exact seams | §1, §4, §5 |

### Not done

| Item | Reason |
|---|---|
| Live drag/reparent and confirmation click | B2 latest run has **0 cards**; the only 3-card run is terminal, and W1 disables drag on terminal runs (`Canvas.tsx:125,206`). No active keyed run exists; no run picker to load the old run. |
| Live non-empty replay (owner links, badge/minimap) | The only non-empty run (`01a1b931`) is not reachable in the UI; the payload gate ignored its foreign snapshot live, which is the correct behavior. A run picker is required to exercise the non-empty replay. |
| Live active-Agents rail (selection/cap) | No active run; the keyless run has no tasks. |
| Provider/keyed paths, approvals, cancel/stop, template insert | Seat rules: no provider, no key, no mutating click. |
| Server restart/reconnect truth | Not in this seat's six surfaces; 06-logic owns it. |
| A fix build after this report | The tree/server moved after B2; the Lead owns the rebuilt gate. |

### Unverified

| Claim | What would prove it |
|---|---|
| Replay owner links + anchor in a non-empty replay | A run picker or a 4-node current run; count `[data-testid="owner-links"] line` and `.hive-node` cards in replay |
| Drag confirmation copy/behavior on an active run | A keyed or seeded active run; drag a worker onto a superagent; capture the dialog, Connect, ACK/NODE_REWIRE |
| Active rail selection/cap | A run with >1 non-terminal tasks; count chips, `aria-pressed`, `+N` |
| Empty state after a **real** history read failure | Force `/api/history/*` 500 and observe the UI (currently no consumer) |
| `history_graph_value` DB-error path | Force a DB failure and call `/state` (currently `unwrap_or_default`) |
| Keyless submit after the B2 rebuild (would create another run) | Not run by this seat; the server-side fix is the proper proof |

---

## 7. Evidence index

- Raw probe outputs (outside the repo): `%TEMP%\orca-a\probe5.out.json`, `probeB2.json`, `probeB2b.json`,
  `probe10.out.json`; probe scripts `probe5.mjs`, `probeB2.mjs`, `probeB2b.mjs`, `probe10.mjs`.
- Screenshots (outside the repo): `%TEMP%\orca-a\keyless-boot.png` (B1 chat banner),
  `keyless-hidden.png` (B1 silent empty), `b2-hidden.png` (B2 false empty + HUD overlap + `Unavailable`),
  `replay6b.png` (B1 35-card replay vs `0/4` and "NO NODES YET"), `b2-final.png`.
- Live HTTP: `/health` (keyless/degraded), `/api/history/runs` (6 runs; newest failed `01a1b9a8`),
  `/api/history/runs/01a1b9a8-…/revisions` (1 revision, `run_status: failed`),
  `/api/history/runs/01a1b931-…/state?revision=6` (B2 WS equivalent: 4 nodes/3 edges).
- Paired reports: `03-forge.md` (D1–D10), `02-weaver.md` (control census, F2–F5), `05-driver.md` (keyless
  submit seq 10–11), `06-logic.md` (LOG-01…LOG-11). This report independently reproduces the trace-level
  facts and records the B2 fixes; it does not re-audit their root-cause detail.

---

### Final line

**Report path:** `docs/trace/orca-wave/01-trace.md`.
**Top 10:** TR-01 keyless ACK phantom run (P0), TR-02 silent terminal failure/false empty/"Unavailable" (P0),
TR-03 no run picker + dropped run status (P1), TR-04 replay counters/rail live-scoped (P1),
TR-05 chat toggles no-op in History (P1), TR-06 replay read-only holes (P1), TR-07 Active Agents rail
partial/live-scoped (P1), TR-08 unknown refusal copy (P1), TR-09 filter state trap/reset (P1),
TR-10 drag/confirm semantics (P1, live UNVERIFIED).

