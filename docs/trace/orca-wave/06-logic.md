# 06 — LOGIC & STATE AUDIT (SEAT F)

Status: **AUDIT ONLY — no product file edited, no process restarted, no provider called, no key read.**
Owner: SEAT F (`/logic-debugger`, input to `/orca-remedy`).
Date: 2026-11-07T23:22–23:37 -06:00.
Workspace: `C:\Users\Riley\Desktop\hiverouter-v1`.
Report path: `docs/trace/orca-wave/06-logic.md` — the only file this seat wrote.
Queue state: **candidate queue, locked against the hashes in §2.2**. The root Lead owns the manifest lock; do not hand-edit this queue after the tree changes without re-pinning. The tree moved **twice** while this audit ran (other seats): §2.2 pins the audited HEAD and the 23:36 working-tree freeze; §0.1 records which queue items were already being fixed at the freeze. **Read §0.1 and §2.2.1 before compiling the manifest** — three items (LOG-02, LOG-06, the D3 half of LOG-07) have in-flight fixes and must not be re-implemented.

---

## 0. Verdict (what the assigned matrix actually shows)

| Audited area | Verdict | The defect(s) |
|---|---|---|
| 4-node scoped snapshot | **FIX-LANDING at 23:36 (uncommitted); recovery + tests still open** | At HEAD: WS replay serialises the raw snapshot (`ws.rs:832-849`); recovery re-writes the whole graph with the unscoped `capture` (`recovery.rs:279`); the HTTP boundary **fails open** when the task index is empty/unavailable (`http.rs:682-691`). The 4-task live run's rev 7 contains **35 nodes / 26 edges, 31 foreign** (§3.1). At 23:33–23:36 another seat extracted `history_graph_value()` and routed the WS read through it, and made the filter unconditional (fail-closed). **Remaining:** a DB error now renders as a silent empty graph; the existing `snapshot_history.rs::history_http_api` test (no task rows, expects 2 nodes) will go red; no test/commit yet. See §0.1 and §2.2.1. |
| History hydration | **UNSAFE TRANSITION** | A resync/reconnect hydrates over a live run: streamed partial reply is destroyed, `streamingRunId` is left set, then the next delta creates a second assistant row; live-only nodes disappear; selection dangles (`store.ts:1038-1237`, `1431-1448`). Probes P1/P6. |
| Run terminal vs replay | **TWO DEFECTS** | ① `RUN_STATUS terminal` is published **before** the terminal snapshot is committed, so the revision refresh can miss the final revision (`execution.rs:1281-1312` vs `store.ts:1423-1428`). ② `HISTORY_SNAPSHOT` ignores the event's `run_id`, so a foreign/stale replay enters this client and poisons `history.revisions` (Probes P3/P4; **fixed uncommitted at 23:36** — see §0.1; the terminal-commit ordering remains open). |
| Graph revision / drag / reparent CAS | **ONE DEFECT, ONE SOUND GUARD** | A committed snapshot rev can be **older** than the live graph rev; after hydration the client sends that stale `expected_rev`, the server CAS refuses `E_RESYNC_REQUIRED`, and the resync returns the same stale value (Probe P2). Cycles are correctly refused by `graph.rs:161-193` — the client can rely on it. |
| Show-chat persistence | **STATE SOUND; AFFORDANCE DEFECT** | `initialChatVisible`/`setChatVisible` round-trip correctly (`store.ts:395-402`, `2200-2207`); the hidden state renders the reopen control, and all three toggles agree. The "removes chat forever" perception is three occlusions, captured live by **02-weaver F2**: the in-canvas reopen pill sits under the HUD, the header toggle sits under the refusal toasts, and the toggle is global while ChatCore is workspace-only. Two of those are already being fixed in the working tree at 23:34 (`ChatCore.tsx:114` z-30; `ToastRegion.tsx:16` top-16). `chat.pinned` is dead state (no caller, no persistence) — appendix P2-2. No persistence defect found; **do not queue a show-chat fix**. |
| Active-agent filtering | **COVERED BY FORGE D2 + WEAVER F3/F5, one interaction** | `hideFinished=true` + terminal reconcile = empty canvas (forge D2). Weaver F3/F5 prove `LibraryPanel`'s "Active Agents" uses a different predicate, silently caps at 8, and reads **live** tasks in replay; the HUD says `3 WORKERS` while the canvas says `0/4`. My unique addition: the HUD and the canvas filter disagree, and the HUD is never told it is showing live numbers under a replay banner (LOG-05). |
| HUD counts | **ONE DEFECT, THREE ADJACENCIES** | HUD uses the **live** `tasks`/`metrics` while `history.replayGraph` is displayed (`HudBadge.tsx:8-11`; `Canvas.tsx` HEAD:500), and hydration across a run switch does not reset `metrics`/`toolActivity`/`approvals` (`store.ts:1169-1206`). Weaver F5 is adjacent (unlabeled derived, root not counted, tools counted as workers); the working tree added a `DERIVED` badge at 23:34, which fixes only the label half. Forge P2-5 (root not counted) is also adjacent. |
| Refresh / WS reconnect | **ONE SEVERE DEFECT** | Reconnect re-hellos the **same boot token**; a restarted server mints a new token every process (`http.rs:337-340`), so the client is permanently `E_WS_DISCONNECTED` with no re-bootstrap (`client.ts:238-345`, Probe P7). A transport `error` after open is a second dead end (Probe P8), and 05-driver **observed it live** (seq 17: `CONNECTION FAILED` / `WEBSOCKET TRANSPORT ERROR.` while `GET /health` returned 200). The full server-restart-recovery half remains unverified (see §7). |
| Empty / error states | **LIES BY OMISSION** | `snapshot.status/incomplete/refusal` and `history.refreshStatus/refreshRefusal` have **zero UI consumers** (grep, §3.4); a failed boot/resync hydration renders as Connected + "No run yet"; a failed terminal-revision refresh renders as "No committed revisions yet". Weaver F4 independently lists `DerivedBadge`, `ReplayBanner`, `healthVersion`, `fileTree.refusal`, `history.refreshRefusal` as stored/exported but never rendered. |
| commit `240fe0f` | **REVIEWED** | The only product change is `hideFinished: false → true` (`Canvas.tsx:112` HEAD); it is the direct cause of forge D2 (finished run = 0 cards) and it is not the fix for the reported "hierarchy" symptom. No logic change in the other audited areas was introduced by this commit; the live runtime was serving an older bundle (§2.3). |

**Top-line answer for the parent:** the scoping fix that commit `678748e` claims is real only on the snapshot-write path and the HTTP read; the WS replay path (the one the Scrubber uses) and recovery still disagree with it, and the HTTP path silently fails open. Independently, hydration can destroy a live stream, and reconnect cannot survive a server restart. The rest of the assigned matrix is either sound or already covered by forge/weaver/capability/driver; the exact merges/conflicts are in §5. **At the 23:36 freeze, LOG-02 and LOG-06 are already fixed in the working tree**; read §0.1 before compiling.

### 0.1 Post-freeze correction (23:36) — do not re-implement three items

The tree moved while this report was being written. At the **23:36 freeze** (`store.ts` `696C46BF…`, `http.rs` `A3E52E55…`, `ws.rs` `6B644242…`), other seats had landed:

| Item | Freeze status | Evidence | What is left |
|---|---|---|---|
| **LOG-02** (WS unscoped / HTTP fail-open) | **fix landed uncommitted** | `http.rs` `history_graph_value()` filters unconditionally; `ws.rs:832-839` calls it; a first version of the function with `if !allowed.is_empty()` was replaced. | DB error now becomes a **silent empty graph** (still “unavailable” rendered as “empty”); the HTTP test with no task rows will go red; no committed test/commit. Keep the item as **verify + tests**, not re-implement. |
| **LOG-06** (`HISTORY_SNAPSHOT` run scope) | **fix landed uncommitted** | `store.ts:1702-1708` now gates on `event.payload.run_id`; `store.ts:1318-1328` gates `NODE_*` (and `HISTORY_SNAPSHOT`) on `envelope.run_id`. | My P3/P4/P9 flipped from pass→fail at 23:36, i.e. the gate works (see §3.2). The no-run edge (`activeRunId === null`) still accepts; the envelope gate is dead for `HISTORY_SNAPSHOT` on the real wire (server publishes it with `run_id: None`, `ws.rs:934-940`) but the payload gate covers it. Keep as **verify + GREEN tests**. |
| **LOG-07** (stale rewire rev) | **D3 half landed; rev-source half open** | `store.ts:1912-1917` no longer calls `guardActiveRun`; it now uses `get().run` + `rewireBlocker`, so a terminal run refuses by name (forge D3). | The committed-vs-live rev source is unchanged: Probe P2 still passes at 23:36 (`expected_rev:1` after live rev 5). This item stays open. |

Still fully open at the freeze: **LOG-01** (P1/P6 still red), **LOG-04** (P7/P8 still red), **LOG-05** (replay live counts + run-switch reset; only the `DERIVED` label landed), **LOG-07** (rev source; P2 red), **LOG-08**, **LOG-09**, **LOG-10**, **LOG-11** (additional, live-verified). Plus **LOG-03** (recovery) untouched.

The probe command/result at the freeze is in §3.2; the earlier 8/8 run is superseded by the 6-pass/3-fail run (the 3 failures are the newly fixed items, now GREEN).

---

## 1. Top 10 — locked candidate fix queue (summary)

| # | ID | P | Defect (one line) | Primary file:line (HEAD) | Fix class | Overlap / conflict |
|---|---|---|---|---|---|---|
| 1 | **LOG-01** | P0 | Hydration over a live run destroys the streamed reply, leaves `streamingRunId` set, drops live-only nodes and dangles selection | `web/src/state/store.ts:1038-1237`, `1169-1206`, `1431-1448` | make hydration a guarded/merged transition; explicit live-gap marker | follows D1/D6/D7; do not weaken resync |
| 2 | **LOG-02** | P0 | WS replay is unscoped and topology-only; HTTP scoping fails open on empty task rows / DB error | `backend/.../ws.rs:832-849`; `http.rs:682-691`; `snapshot.rs:264-285` | one projection for both reads; fail closed, never unfiltered | **merge forge D1+D6**; **fix landed uncommitted at 23:36 — verify + tests, not re-implement** |
| 3 | **LOG-03** | P0 | Recovery writes the process/restored graph as every changed run's new revision; can tag another run's image as this run's revision | `backend/.../recovery.rs:139-177`, `273-303` (`:279`), `389-414` (`:408`) | `capture_scoped` + per-run allowed set; never invent a top-level parent; report not-restored runs | **merge forge D5** |
| 4 | **LOG-04** | P0 | Reconnect cannot survive a server restart: token rotates per process, client never re-reads `/api/session`; `error` state is a dead end | `web/src/api/client.ts:238-345`; `backend/.../http.rs:337-340`; `ws.rs:382-384` | bounded re-bootstrap on close/error; explicit retry state | no forge overlap; coordinate e2e reconnect |
| 5 | **LOG-05** | P1 | HUD shows live counts/metrics while a replay graph is displayed; hydration over a run switch leaves stale metrics/tool/approvals | `web/src/panels/HudBadge.tsx:8-11`; `web/src/state/metrics.ts:24-45`; `store.ts:1169-1206`, `1518-1521` | tie HUD to the displayed projection or label it live; reset per-run surfaces on switch | adjacent forge P2-5 (root count) |
| 6 | **LOG-06** | P1 | `HISTORY_SNAPSHOT` ignores `payload.run_id`: foreign/stale replay enters this client and its revision is inserted into the current run's list | `store.ts:1691-1706`; `1374-1396`; `ChatCore.tsx:337,343,356` | gate by run id; ignore stale responses | extends forge D7 (NODE_* gate) |
| 7 | **LOG-07** | P1 | Rewire sends a committed rev that can be older than the live graph rev; CAS refusal → resync → same stale rev (loop) | `store.ts:1282-1310`, `1495-1497`, `1895-1947`; `graph.rs:147-158` | use the live confirmed rev, or refuse without sending / re-read after refusal | **merge forge D3** (`rewireSelected` rewrite) |
| 8 | **LOG-08** | P1 | Hydration/revision-read failures are invisible; empty states present as the truth | `store.ts:1211-1231`, `1016-1027`, `173-198`; `HistoryPanel.tsx:20-24`; `Scrubber.tsx:30,62-64` | surface the named refusal in the existing banner/toast/empty state | "no new UI" style; visual gates |
| 9 | **LOG-09** | P1 | Terminal `RUN_STATUS` is published before the terminal snapshot commit; the client's revision refresh can miss the final revision | `execution.rs:1281-1312`; `store.ts:1423-1428` | commit before publish, or retry the revision read once | lifecycle-lock ordering; `run_lifecycle.rs` |
| 10 | **LOG-10** | P1 | `messages-unavailable` is asserted without evidence; history truncation is silent | `store.ts:1107-1143` (`1121`, `1137`); `http.rs:777-792`; `store.ts:505-529` | only report what was observed; surface `truncated`/paginate | `http_boot.rs` transcript test |

Details, acceptance tests, conflicts and residuals are in §4. The appendix (§6) records minors and dead paths.

**Freeze status at 23:36 (read this before compiling):** LOG-01 OPEN · LOG-02 **IN-FLIGHT FIX (verify/tests only)** · LOG-03 OPEN · LOG-04 OPEN · LOG-05 OPEN (partial: `DERIVED` label landed) · LOG-06 **IN-FLIGHT FIX (verify/GREEN tests only)** · LOG-07 **PARTIAL** (D3 guard landed; rev source OPEN, P2 red) · LOG-08 OPEN · LOG-09 OPEN · LOG-10 OPEN. LOG-02/LOG-06 must not be implemented twice; see §0.1 and §2.2.1.

**Additional P1 beyond the top 10 — live-verified by 05-driver (LOG-11):** a keyless `SUBMIT_PROMPT` is accepted (run created), then the run fails `E_KEY_MISSING`; ChatCore's settlement heuristic sees no command refusal, so it **clears the draft and keeps the optimistic prompt** while the failure is only the run-error panel. This is the "run failure is not a submit success" error-path defect; details in §4 (LOG-11). It is not a duplicate of LOG-08 (that is invisible hydration failure) or LOG-10 (fabricated incomplete markers).

---

## 2. Pinned artifacts, method, concurrent edits

### 2.1 Method

- Read the current source at HEAD + the working tree, in the order: store state machine → WS/HTTP read paths → graph/CAS → recovery → UI consumers.
- Re-derived the state machines as invariants (§4, each item states one falsifiable sentence).
- Built a **deterministic, read-only probe harness outside the repo** that imports the real `web/src` modules (no product edit). Nine probes, one per logic edge; at 23:29 eight of them passed on the audited code (RED leg), and at 23:36 the ninth plus the two run-id probes were GREEN after the in-flight gate (§3.2).
- Took read-only live probes (`GET /health`, `GET /api/history/runs`) against the first server process (pid 15484); analyzed the live data directory offline (DB copy + snapshot JSON) for the 4-node fixture; a new read-only `/health` was taken at 23:36 against the replacement process (pid 22380).
- Reviewed the sibling reports **as they appeared**, `02-weaver.md`, `03-forge.md`, `04-capability.md`, `05-driver.md`, and searched specifically for interactions none of them covers (§5). The report was re-pinned at 23:36 after the working tree changed; §2.2.1 is the delta.

### 2.2 Hashes and the concurrent-edit warning (material)

| File | SHA-256 at analysis | Note |
|---|---|---|
| `web/src/state/store.ts` | `E4F163176B6E44D44831A8C5426F1F7A6AE26BE4922F40F75E43C4450A634C2E` | unchanged by any seat |
| `web/src/api/client.ts` | `745FC57A06EBA0ABE460721CAC309B0DB0CA02B5367A821A04D7144A2A12E238` | unchanged |
| `web/src/panels/HudBadge.tsx` | `BE7FD1C62FF60CAA5BDC451E43E48F96AC4563CAA1B3A67333501EA02E7263B9` | unchanged |
| `web/src/panels/HistoryPanel.tsx` | `72CB64CEED45DD7FE9596B9EB2DE80673076A1AB8C580FE1434EE4509B1C9193` | unchanged |
| `web/src/panels/Scrubber.tsx` | `3329167DD0A8CF929E5991959161E6934338E16165E838481BA7269E9D9ACAEB` | unchanged |
| `web/src/chat/ChatCore.tsx` | `0C4E2DD746EA48E1639A8FCB1396371DC9BBF13B327C5853A7A02083315F49CB` | unchanged |
| `web/src/state/metrics.ts` | `1B264CDD370F04C99DE8FDC9D83DD2300A2A896987C03693C597D7645DA8FCC9` | unchanged |
| `backend/.../ws.rs` | `F036F47123ACAA942CC399A913484626741997B7CFFB86F589B5F652F1553AA2` | unchanged at analysis |
| `backend/.../graph.rs` | `8B3A24A29E425EE41ABD2F5DF7CBEA46E3326797185C8D2EDA5620E9FC1680C3` | unchanged |
| `backend/.../snapshot.rs` | `2954FF374BE2DBC6E4E42D0D396795DF35864D8FFDA2F57E97413EC29C9F3398` | unchanged |
| `backend/.../recovery.rs` | `B35493C23123EAB3656689566B9002B6A44A52186DF3F209AC60D72E00F6D1AE` | unchanged |
| `backend/.../execution.rs` | `F02F032A99CAD91E593CCB0140423AB990112515C86F7F417D9345E945A53AC0` | unchanged |
| `backend/.../http.rs` | **HEAD** `AC5F9B96AAC81990DFF96B6D6884CC5FAF812D7CD274ABFC3DCA60A2565F77A4`; **working tree** `9E1B1CA7AB695EC1A412F4D3C5AC02BF94C22E28FE094E2F7F26359D617F77DF` | **being edited by another seat at 23:30** — extracts `history_graph_value()` for the WS path (forge D1/D6 fix). Keeps `unwrap_or_default()` + `if !allowed.is_empty()` — LOG-02's fail-open sub-defect survives the extraction. |
| `web/src/flow/Canvas.tsx`, `graph.ts` | working tree modified | another seat's layout/fit work (column layout, 0.75 clamp, replay anchor); HEAD line numbers in this report are for `240fe0f` (747-line Canvas) and do **not** match the 805-line working copy. |
| HEAD | `240fe0fd5d9b5a0845e670da51e7d75bfb1e934a` | audited commit |
| `git status --porcelain` at 23:31 | `M backend/.../http.rs`, `M tools/desktop-launcher/HiveMind.exe`, `M web/src/flow/Canvas.tsx`, `M web/src/flow/graph.ts`, `?? .claude/ AGENTS.md CLAUDE.md docs/trace/orca-wave/` | none of these edits is mine |

#### 2.2.1 Working-tree freeze at 23:36 (other seats; not the audited HEAD)

The tree changed **while this audit ran**. The audited claims above are against HEAD; these are the deltas as of the freeze, and they change the status of three queue items (§0.1):

| File | Hash at 23:36 | Δ / in-flight change | Item affected |
|---|---|---|---|
| `web/src/state/store.ts` | `696C46BFC2F7A52B5B48CF7F50FF2F7DD8558C15AB5A3CC100D6F87E05A61D8A` | +18 net lines: envelope-run gate at `1318-1328`, payload-run gate in `HISTORY_SNAPSHOT` at `1702-1708`, `rewireSelected` guard now `get().run` + `rewireBlocker` at `1912-1917` | LOG-06 fixed; LOG-07 D3 half fixed; rev source still open |
| `backend/.../http.rs` | `A3E52E55AC387B39B8A7336D2A7572E3E12B5315F5AA31EB6C008ABA3EC431CC` | `history_graph_value()` extracted; unconditional `retain` (fail-closed), no more `if !allowed.is_empty()` | LOG-02 fixed (uncommitted) |
| `backend/.../ws.rs` | `6B64424229EDDB72EA63CB012408683197035E604FC054F8A937D5A5249005AA` | `HistoryRevisionSet` now calls `crate::http::history_graph_value(...)` instead of `serde_json::to_value(envelope.graph)` | LOG-02 fixed (uncommitted) |
| `web/src/chat/ChatCore.tsx` | `B4EC210E4311914C492FD2B0338657A43FA0E7EB55D099EFFE0CB6EAAB43A2F0` | reopen pill `z-30` (`:114`) | weaver F2 (show-chat occlusion) |
| `web/src/panels/HudBadge.tsx` | `07EC7C267F68A22AAF9441CB1DE710A976B33E8103744A1BB0CD769449138CF2` | `DERIVED` badge (`:45-49`) | LOG-05 label half only |
| `web/src/panels/ToastRegion.tsx` | `CB4AC99C047AC7A0AC54904920F919B09E582CC33A43C1A965C0F6AD2DEE1BBB` | `top-16` (below header) | weaver F2 |
| `web/src/flow/Canvas.tsx`, `graph.ts`, `AgentProfilesPanel.tsx`, `ApprovalOverlay.tsx` | modified | layout/fit/anchor + profile/dismiss fixes | not this seat's audit; forge/weaver scope |
| current `git status` at 23:36 | `M` on the above + `?` on `.claude/ AGENTS.md CLAUDE.md docs/trace/orca-wave/` | none of these edits is mine | — |

**Line drift:** the store's HEAD references after `~1315` are no longer current. Roughly: HEAD `1318-1701` → +11, HEAD `>=1702` → +17 (plus the rewire edit). The §4 references are HEAD lines by design; re-pin against the freeze hash before applying. `http.rs`/`ws.rs`/Canvas/graph line numbers are also moving — use the function names in the fixes where possible.

**Re-pin before applying LOG-02 (http.rs), LOG-05 (Canvas line 500/663-669) and any UI line numbers.** Store/client/backend logic lines in LOG-01/04/06/07/08/09/10 are stable as of the hashes above.

### 2.3 Live runtime (read-only, captured before it exited)

The packaged server was running as PID 15484 (`dist\HiveMind\hiverouter.exe --bind 127.0.0.1:8099 --data-dir C:\Users\Riley\AppData\Local\HiveMind --workspace-root <repo>`) when this audit began. **At 23:29 the app server was replaced by another actor (old PID 15484 → new PID 22380, new bundle built 23:25:52)** — this is recorded by `05-driver.md` §Environment; this seat did not start, stop or restart it. At the 23:36 freeze the live process is PID 22380 on 8099 (`/health` below); a second, fixture server (PID 19176, port 8097, temp data dir) belongs to another seat and was not touched. Captured from PID 15484 before the replacement:

```
GET /health -> status "degraded", version "1.2.0-dev", schema "hiverouter.v1",
  deepseek_api_key ok:false code:E_KEY_MISSING,
  database {path "...\HiveMind\hiverouter.db", schema_version 10},
  transport {last_seq 13, graph_nodes 35, allowed_origins 2}
GET /api/history/runs -> 5 runs, newest first:
  01a1b931-941a-7171-a209-1c05777354f8 completed 2026-11-08T03:26:36Z  (the 4-task run)
  01a1b931-2a4f-75c2-9596-7c447f78b986 completed
  01a1b930-9ca0-70b9-b4f0-1450f3d1643d failed
  01a1b92f-f31f-747e-a63f-58a8fbc0998b failed
  01a1b92e-7350-7119-8c70-f4d0950cec1d failed
```

New process, read-only probe at 23:36: `GET /health -> status degraded, version 1.2.0-dev, graph_nodes 35, last_seq 4` (keyless). The new binary/bundle pre-dates the 23:30–23:36 working-tree fixes (dist exe mtime 23:24:39; driver-recorded bundle 23:25:52), so **do not treat the live process as proof of LOG-02/LOG-06 fixes** — those are uncommitted source fixes only.

A second GET (`/state?revision=7`) was attempted across the replacement and failed. The 4-node state result quoted in §3.1 is therefore derived from the on-disk snapshot + DB task rows (this seat) and is corroborated by forge's live capture against PID 15484 (`nodes=4`, §4 of `03-forge.md`) — not claimed as this seat's live HTTP probe.

---

## 3. Evidence pack

### 3.1 The 4-node scoped snapshot (the live fixture)

Run `01a1b931-941a-7171-a209-1c05777354f8` (newest, completed) has exactly **4 task rows**:

| task id | role | db state/rev | parent |
|---|---|---|---|
| `01a1b931-9eae-731b-ab0e-29d53d23430f` | planner | completed / 1 | — |
| `01a1b931-9eae-731b-ab0e-29d215f5a82c` | worker | completed / 2 | planner |
| `01a1b931-9eae-731b-ab0e-29d3b087b93d` | worker | completed / 2 | planner |
| `01a1b931-9eae-731b-ab0e-29d4e6db997e` | worker | completed / 2 | planner |

Offline read of `C:\Users\Riley\AppData\Local\HiveMind\snapshots\01a1b931-941a-…\0000000007.json` (the latest committed revision, `run_status: completed`):

```
envelope_revision 7, graph_revision 89
nodes 35, edges 26
own nodes 4, own edges 3
foreign nodes 31, foreign edges 23
node fields: ["parent_task_id","rev","state","task_id"]   (no kind, no role, no label)
own ids == snapshot ids ∩ tasks_for_run  -> true
```

**Therefore:**
- The scoped projection of rev 7 must be exactly **4 nodes / 3 edges** (the planner root + 3 workers). That is what the HTTP read computes today when `tasks_for_run` succeeds (`http.rs:682-691` HEAD).
- The raw file contains **35 nodes / 26 edges** and only topology — the WS `HISTORY_REVISION_SET` path sends this raw graph (`ws.rs:832-849`), so the Scrubber replay of this 4-node run paints 31 foreign agents and cannot know the root is a hyperagent. Forge's live probe captured exactly this (`node_count 35`, `roots 9`, fields topology-only) while the HTTP read of the same revision returned `nodes=4` with `kind:"hyperagent" role:"planner"` (`03-forge.md` §D1/D6).
- All eight on-disk revisions of this run are 35 nodes (pre-`678748e` capture); the fix's read-boundary filter is the only thing standing between that file and the screen.

The **fail-open** edge: if `tasks_for_run()` errors, `unwrap_or_default()` produces an empty `allowed` set and `if !allowed.is_empty()` skips all filtering — the HTTP route then returns the entire 35-node graph for this 4-node run. The same shape occurs for any run whose task rows were pruned/lost (the workspace has five such snapshot dirs from an earlier DB lifetime). The correct behaviour is an honest empty/refusal, never the unfiltered graph.

### 3.2 Deterministic probes (outside the repo, real source, read-only)

Harness: `C:\Users\Riley\AppData\Local\Temp\seatf\probe` (copy of `web/src` + junction `node_modules` → repo's; no repo write). Command:

```
cd C:\Users\Riley\AppData\Local\Temp\seatf\probe
node "C:\Users\Riley\Desktop\hiverouter-v1\web\node_modules\vitest\vitest.mjs" run --root .
```

Result at 23:29 (audited code, 8 tests):

```
 ✓ tests/logic-probe.test.ts (8 tests) 252ms
 Test Files  1 passed (1)
      Tests  8 passed (8)
EXIT=0
```

Result at the **23:36 freeze** (9 tests; the extra P9 targets the in-flight gate):

```
 ❯ tests/logic-probe.test.ts (9 tests | 3 failed) 150ms
   ✓ P1 hydration during a stream drops the streamed partial reply and splits the assistant row
   ✓ P2 hydration downgrades the live server rev; a later rewire sends the stale expected_rev
   × P3 accepts a HISTORY_SNAPSHOT for another run: its revision is inserted into this run list
   × P4 a stale HISTORY_SNAPSHOT for the old run survives RUN_CREATED and disables the composer state
   ✓ P5 run-changed guard reports messages-unavailable even though the messages read succeeded
   × P9 the new envelope-run gate covers NODE_* but not HISTORY_SNAPSHOT (server publishes it with run_id None)
   ✓ P7 the WS client never re-bootstraps /api/session after a reconnect (stale boot token)
   ✓ P8 a transport error after open is a dead end: open does not clear error/status and send stays refused
   ✓ P6 hydration drops a live node that was spawned after the last committed snapshot
 Test Files  1 failed (1)
      Tests  3 failed | 6 passed (9)
EXIT=1
```

**Reading the 23:36 run:** P3/P4/P9 failing means the *foreign-snapshot gate landed* (the snapshot is now ignored, so the probes that asserted it was accepted now fail) — i.e. LOG-06 is GREEN in the working tree. P1/P2/P5/P6/P7/P8 still pass, so LOG-01/04/05/07(rev)/08/10 are still red. The tests are the RED→GREEN leg: port P1/P2/P5/P6/P7/P8 as failing tests into `web/tests/`; port P3/P4/P9 as GREEN regression tests with the corrected gate assertions.

| Probe | Falsifiable claim proven (P1–P8 pass at 23:29 = RED; P3/P4/P9 green at 23:36 = fix verified) |
|---|---|
| **P1** | Stream `REPLY_DELTA 'PARTIAL'` → `refreshRunSnapshot('resync')` → messages become `['user:hi']`, `streamingRunId` is still set, next `REPLY_DELTA '-TAIL'` appends a second assistant row whose text is only `-TAIL`. Server persisted `PARTIAL-TAIL`. |
| **P2** | Live `NODE_STATE rev=5` → hydration from committed `rev=1` → `rewireSelected` sends `REWIRE_TASK{expected_rev:1}` (the prior send, before hydration, was `5`). The server CAS compares live 5 (`graph.rs:151-155`) and refuses. |
| **P3** | `HISTORY_SNAPSHOT{run_id:'another-run', revision:9}` while current run is `run-1` → `history.replayGraph` set, `history.selected=9`, `revisions` now contains 9 (the server never listed 9 for `run-1`). Clicking it sends an impossible revision. |
| **P4** | `RUN_CREATED run-2` (resets history) then stale `HISTORY_SNAPSHOT{run_id:'run-1'}` → replayGraph set while `run.id === 'run-2'`; composer is disabled by `replaying` (`ChatCore.tsx:337,343,356`). |
| **P5** | Run-changed guard: `/messages` read **succeeded** (fetch count +1) yet `snapshot.incomplete` contains `'messages-unavailable'` (and `'run-changed'`). |
| **P6** | Live `NODE_SPAWN live-3` after the last commit, select it, hydrate → `tasks['live-3']` is gone, `selectedTaskId` still names it. |
| **P7** | `WsApiClient.open()` → close → transport reopen: `/api/session` fetch count stays **1**; status returns `open` on the same boot token. A restarted server mints a new token (`http.rs:337-340`), so this URL can never authenticate again. |
| **P8** | After open, a transport `error` sets status `error`; a subsequent transport `open` does not send hello and does not clear `error`; `send()` keeps rejecting `E_WS_DISCONNECTED` even though the socket is open. |
| **P9** | The new `store.ts:1318-1328` gate ignores a foreign `NODE_*` by `envelope.run_id` (works), but `HISTORY_SNAPSHOT` is published with `run_id: None` (`ws.rs:934-940`), so the envelope gate cannot see it — the separate `payload.run_id` gate at `store.ts:1702-1708` is the one that matters. At 23:36 the payload gate ignores the foreign snapshot (P9 fails = fixed); if the envelope gate were the only one it would pass = still broken. |

These are the cheapest deterministic repros; the implementing seat should port them into `web/tests/` (acceptance tests in §4).

### 3.3 What was verified sound (no fix needed; do not re-open)

| Area | Evidence |
|---|---|
| Graph cycle prevention | `graph.rs:160-193`: self-rewire refused, `has_path_connecting(task,new_parent)` refused, defensive `is_cyclic_directed` revert; `add_task` refuses unknown parent. The client may rely on `E_CYCLE`; no client-side cycle walk is required. |
| Rewire CAS check | `graph.rs:147-158` compares the live node rev; `begin_rewire`/`rewire` set/clear the in-flight marker; a stale rev is refused by name (`E_RESYNC_REQUIRED`), never applied. The bug is the **client's source** of `expected_rev` (LOG-07), not the server guard. |
| Show-chat persistence | `initialChatVisible()` reads `hivemind.chat.visible.v1`; `setChatVisible` writes it; hidden chat renders the reopen button; Header (`aria-pressed`), Canvas ("Graph only"/"Show chat") and ChatCore close/reopen all agree. `CHAT_PIN` is a deliberate client-only ACK (`ws.rs:737-742`), and no surface calls `pinChat` (dead state, appendix P2). |
| Replay revision-count clamp | `Scrubber.tsx:16-32` clamps by `revisions.length`, not `latest+1`; `formatSelectedRevision` maps by index of the exact revision; sparse revision numbers cannot render "state 7 of 4". |
| History decoder strictness | `decodeHistoryRevisions` refuses non-integer/negative revisions and a `latest_revision` that disagrees with the list; `decodeHistoryState` refuses a missing/negative revision; `recordsFromSnapshotGraph` refuses duplicate ids/malformed nodes. Fails closed, not fabricated. |
| Evidence availability (`E_RESYNC_REQUIRED`) | `raiseResyncGap`/`resyncSerial` prevents an old in-flight snapshot from clearing a newer gap (`store.ts:1168-1210`, `1252-1265`); `SERVER_HELLO` correctly refuses to present a gap as Connected (`1339`). |
| Tool-result ordering | `TOOL_RESULT` before `TOOL_CALL` is buffered (bounded/deduped), first terminal outcome wins, terminal runs drop late results (`store.ts:760-800`, `1522-1593`). |
| `RUN_CREATED` fresh-run reset | Clears tasks/selection/toolActivity/metrics/streaming/history/snapshot for a *new* run id (`store.ts:1374-1396`); the chat transcript is deliberately kept. Missing: approvals (appendix P2-7). |

### 3.4 Grep receipts for the invisible-error-state finding

```
grep -n "state\.snapshot|isSnapshotDegraded|refreshStatus|refreshRefusal|resyncGap" web/src
```

- `snapshot.status / source / runId / revision / incomplete / refusal / attempts / updatedAt`: **store.ts only** — no `.tsx` consumer (0).
- `isSnapshotDegraded`: exported at `store.ts:749-756`, **0 call sites**.
- `history.refreshStatus` / `history.refreshRefusal`: set at `store.ts:1005,1023-1024,993-994,1190-1191`; **0 render sites**.
- `resyncGap`: only the connection-state gate (`store.ts:1339,1805,1827`); no visible gap text.
- `approvalsStale` **is** consumed (`ApprovalOverlay.tsx:167,212`) — do not lump it in.

---

## 4. Candidate fix queue (detailed)

Legend: each item has one falsifiable sentence, the exact HEAD line, the minimal fix, the acceptance test (command + failing assertion on today's code), conflicts, residual. IDs are stable for the manifest.

### LOG-01 — P0 — Hydration is not a safe transition; it destroys a live stream and live-only graph state

**Falsifiable sentence.** If a run is live (a `REPLY_DELTA` row is streaming or a `NODE_*` arrived after the latest commit), a `performSnapshotRefresh('resync'|'reconnect'|'refusal'|'rewire')` replaces `messages` and `tasks` wholesale, so (a) the streamed partial assistant text disappears, (b) `streamingRunId` remains non-null, (c) the next `REPLY_DELTA` opens a second assistant row, (d) a task spawned after the last commit disappears, and (e) `selectedTaskId` can name a task no longer in `tasks`.

**Evidence.** Probes P1/P6; code: `store.ts:1169-1206` (the set block), `1171` (`messages: historyMessages`), `1173-1177` (`tasks: projection.tasks`), `1182` (`graphRev`), `1431-1448` (`REPLY_DELTA` row selection), `1413` (`streamingRunId` only cleared on terminal). Server persists **only the final reply** — `execution.rs:3011-3014` appends the final text once at completion; `REPLY_DELTA` is published, never stored (`execution.rs:1779-1799`). Hydration sources: `SERVER_HELLO` when snapshot not ready (`store.ts:1352-1361`), `RESYNC_REQUIRED` (`1364-1372`), `ERROR E_RESYNC_REQUIRED` (`1326-1330`), foreign `NODE_REWIRE` without parent (`1503-1511`), manual (`2147`).

**Minimal fix.** Make hydration a guarded transition, not a blind replace:
- When `get().run` is **active** and `streamingRunId !== null`, keep the local stream row(s); hydrate only `run`/`history` (and merge any task not present locally? do **not** delete local tasks whose state is newer than `snapshot.revision`).
- Clear `streamingRunId` (and mark the transcript row non-streaming) when hydration legitimately replaces the transcript, or keep it and keep the row — never leave a stale non-null value.
- Add an explicit `SnapshotIncomplete` value such as `'live-gap'` and set it when a resync hydrates over an active run; the UI must then say the graph is committed-state + subsequent events, not the live truth.
- Clear `selectedTaskId` when it is not in the new `tasks` map (or keep a dangling flag and render "no longer in graph").
Do not change the boot path (no live run) or the no-commit degraded path.

**Acceptance.**
- `web/tests/logic.hydration.test.ts` (new; port P1/P6):
  - `hydration_during_stream_keeps_one_assistant_row`: `REPLY_DELTA PARTIAL` → `refreshRunSnapshot('resync')` → next delta → assert exactly one assistant row and the final text is not regressed; `streamingRunId` equals the live run id or null, never a stale non-null.
  - `hydration_does_not_drop_a_live_spawned_node`: `NODE_SPAWN live-3` after the last commit → hydrate → assert `live-3` present (or `snapshot.incomplete` includes `live-gap` and the UI explains it).
  - `hydration_clears_dangling_selection`.
  - RED today: the P1/P6 assertions (proven).
- Command: `cd web; npm test -- tests/logic.hydration.test.ts`.
- Regression: `streamingRender.test.ts` must stay green (normal stream path); `live-resync.spec.ts` paid path must not duplicate reply text.

**Conflicts.** Must be sequenced **after** LOG-02/LOG-06 (a scoped/enriched WS replay can otherwise still trigger a replace with a different projection). It must not weaken the resync guarantee: a client with a real event gap cannot simply trust local state; the `live-gap` marker is the honest compromise. No conflict with forge D7, but the same `envelope.run_id` gate should be shared.

**Residual.** A resync during an active run can never reconstruct events lost before the committed snapshot; the marker makes that explicit, it does not make it whole.

---

### LOG-02 — P0 — The WS replay read is unscoped/topology-only, and the HTTP scoping fails open

> **STATUS at 23:36 — FIX LANDED UNCOMMITTED; do not re-implement.** `http.rs` now exposes `history_graph_value()` with unconditional `retain` (fail-closed) and `ws.rs:832-839` calls it. What remains is (a) a DB error is still silently rendered as an empty graph (no named refusal), (b) the existing `snapshot_history.rs::history_http_api` test has no task rows and now returns 0 nodes (it will go red), (c) the live e2e still asserts `nodes >= 2`, and (d) no committed test. Turn §4's acceptance into GREEN tests and close only those gaps.

**Falsifiable sentence.** For the same run+revision, `HISTORY_SNAPSHOT` and `GET /api/history/runs/{id}/state` disagree on the node/edge set (35/26 vs 4/3 on the live 4-node run), the WS payload has no `kind/role/label`, and the HTTP read returns the **unfiltered** graph whenever `tasks_for_run` errors or is empty.

**Evidence.**
- WS: `ws.rs:832-849` — `Command::HistoryRevisionSet` does `serde_json::to_value(envelope.graph)` and publishes it; no filter, no enrichment.
- HTTP: `http.rs:676-691` HEAD filters only **if** `allowed` is non-empty; `let run_tasks = state.engine.store.tasks_for_run(run_id).unwrap_or_default();` silently turns a DB refusal into an empty allow-set and then **returns the whole process graph**. The 23:33 working-tree extraction `history_graph_value()` made the `retain` unconditional (fail-closed), so the leak is fixed; the DB-error path still degrades to a silent empty graph (see §0.1).
- Write path is scoped: `execution.rs:1193-1215` (`tasks_for_run` → `capture_scoped`), `snapshot.rs:264-285`.
- Data: §3.1 (35 vs 4); forge live probe (§D1/D6) for the same disagreement.
- Existing test gap: `snapshot_history.rs::history_http_api` creates **no task rows**, so `allowed` is empty and the filter is silently skipped — the test passes because of the fail-open path. `e2e/specs/live-replay.spec.ts:126` only asserts `nodes >= 2`, so 35 nodes for a 4-node run passes.

**Minimal fix.** One projection function used by both routes (the working-tree `history_graph_value` is the right shape):
- `ws.rs:832-849` calls it instead of `serde_json::to_value(envelope.graph)` (this also fixes forge D1's missing kind/role/label).
- Make the projection **fail closed**: if `tasks_for_run` returns `Err`, return a named refusal (503 / `E_INTERNAL_ERROR` or a new honest code) — never the unfiltered graph. If the run has no task rows (a pruned/legacy run), return an empty graph with `nodes: []` plus an explicit marker, or refuse `E_HISTORY_SCOPE_UNKNOWN`; do **not** return the process graph.
- Keep `capture_scoped` on the write path; do not duplicate the role→kind table.

**Acceptance.**
- `backend/crates/hiverouter-core/tests/snapshot_history.rs`:
  - `ws_replay_and_http_state_agree`: insert 4 task rows + write an unscoped snapshot containing a second run's nodes; send `HISTORY_REVISION_SET`; assert WS `graph.nodes` ids == HTTP ids == `tasks_for_run(run).ids` and == 4; assert each node has non-empty `kind/role/label` and the planner maps to `kind:"hyperagent"`.
  - `history_state_without_task_rows_never_leaks`: same snapshot, no task rows; assert the response contains **zero** foreign nodes (empty or a named refusal), not 35.
  - `history_state_db_error_fails_closed`: force `tasks_for_run` error; assert non-200 refusal, no graph.
- Commands: `cd backend; cargo test -p hiverouter-core --test snapshot_history`.
- Tighten `e2e/specs/live-replay.spec.ts` to assert `nodes == task_count(run)` (not `>= 2`) and to check `kind`/`role`.

**Conflicts.** **Merges with forge D1 + D6** — one implementation. The in-flight `history_graph_value` extraction is the vehicle; do not let it keep the empty-allowed bypass. Changing the HTTP no-task-rows behaviour may break `history_http_api`'s assumptions (it expects 2 nodes with no task rows) — update that test deliberately to seed task rows; that is the point of the gate.

**Residual.** Role is free-form; the kind derivation silently maps unknown roles to `worker` (forge D1 residual). A snapshot whose task rows are gone becomes empty rather than wrong — honest, and the client must render an empty replay, not a crash (`buildReplayTasks` returns `{}`).

---

### LOG-03 — P0 — Recovery writes the unscoped restored graph as every changed run's new revision

**Falsifiable sentence.** Boot recovery restores the newest run's snapshot into the one process-wide graph, then for every changed run writes `snapshot::capture(graph, run_id, ...)` (unscoped). If a changed run's nodes are not in that restored image, its new revision contains **another run's nodes and not its own**; if the parent node is absent, `settle_graph_node` invents a top-level node (`add_task(task.id, None)`).

**Evidence.** `recovery.rs:139-177` (restore loop, last snapshot wins), `263-303` (`changed_runs`, `:279` unscoped capture), `389-414` (`:408` parentless `add_task`), `417-431` (`next_revision`). Compare the fixed write path (`execution.rs:1203` `capture_scoped`). Forge D5 covers the unscoped capture; the **tagging** interaction and the empty-projection consequence are new here. On this workspace all on-disk snapshots are pre-fix and unscoped, so a recovery write would immediately create a cross-run revision that the new scoped read would then filter to zero nodes (HTTP) while the WS path shows foreign nodes (§3.1).

**Minimal fix.**
- Capture with `capture_scoped(graph, run_id, revision, status, &allowed)` where `allowed = tasks_for_run(run_id)`; if the allowed set is empty, skip the write and record `snapshot_write_failures` (do not write an empty/foreign revision silently — or write an explicit empty revision only if the run genuinely has no tasks).
- Do not call `capture` for a run whose nodes are not the restored image. Either restore per-run images, or only write the recovery revision for the run that owns the restored graph and report the others as not-restored (forge D5's (c)).
- At `settle_graph_node`, when the parent is absent from the graph, record `graph_settle_failures` and leave the node out; do not `add_task(id, None)` (that invents a flattened topology).

**Acceptance.**
- `backend/crates/hiverouter-core/tests/recovery_gates.rs` (new test): two runs A/B each with task rows + snapshots; crash state leaves an interrupted run with a task row; `recover_on_boot`; assert every node id in the newest revision of each changed run ∈ `tasks_for_run(run)`; assert no `add_task(None)` flattening (parent set or recorded failure).
- Keep `kill_restore.rs` (single-run, 2 nodes) green.
- Command: `cd backend; cargo test -p hiverouter-core --test recovery_gates --test kill_restore`.

**Conflicts.** **Merges with forge D5**; one implementation. Fixing (c) changes which run is visible after a crash — release-note it (forge D5 residual). Coordinate with LOG-02: the read boundary must fail closed while recovery transitions to scoped writes.

**Residual.** With one process-wide graph, recovery cannot reconstruct two runs' concurrent images; the honest outcome is "one restored run + the others readable through history", not a merged revision.

---

### LOG-04 — P0 — Reconnect cannot survive a server restart; `error` status has no recovery edge

**Falsifiable sentence.** `WsApiClient` fetches `/api/session` exactly once; every reconnect re-sends hello on the **same URL with the boot token**. `AppState` mints a new UUID token on every process (`http.rs:337-340`), and `/ws` rejects the old token (`ws.rs:382-384`), so after a server restart the client retries a permanently invalid URL (max 12 times), stays `reconnecting`, and never re-bootstraps. Separately, a transport `error` after a successful open sets status `error`; a later `open` event does nothing (`client.ts:322-345`), leaving `send()` permanently `E_WS_DISCONNECTED` on an open socket.

**Evidence.** Probe P7 (session fetch count stays 1 across close→reopen; status returns `open` on the stale token), P8 (error→open dead end); `client.ts:238-249` (cached `open` returns the session when open), `258-320` (single bootstrap), `322-345` (reconnect only re-hellos; error sets status), `107-144` (`PartyTransport` `maxRetries:12`, same URL); `http.rs:337-340` (fresh token per `AppState`); `ws.rs:382-384` (token check). The launcher opens the system browser and never reloads it, so a server restart leaves the tab dead.

> **LIVE CORROBORATION (05-driver, 23:40):** after the keyless run/replay churn the driver saw `CONNECTION FAILED` / `WEBSOCKET TRANSPORT ERROR.` while `GET /health` still returned 200; `RETURN LIVE`/`replay 1 OF 1` remained, `HIDE FINISHED 0/0`, no nodes, and the click did not execute. That is exactly P8: the transport error sets a dead-end status, the UI cannot recover the socket, and the only path is a page reload. This is live, not just a probe.

**Minimal fix.**
- On `close`/`error` after a retry threshold (or immediately on a 401/403 upgrade failure), invalidate `this.session` and re-run the `openInternal` bootstrap (fetch `/api/session`, rebuild the URL, create a new transport). Bound the attempts and surface a named "server restarted, reconnect" banner.
- Treat `error` as recoverable when the transport remains open: the next `open` must send hello and set `open` (do not gate only on `status === 'reconnecting'`).
- Keep `pullPendingApprovals`' hello semantics unchanged (it deliberately reuses the current socket).
- Alternatively expose a user-visible Retry that calls the same bootstrap.

**Acceptance.**
- `web/tests/apiLifecycle.test.ts` (extend): `reconnect_after_token_rotation_rebootstraps`: fake transport + fetch returns token1 then token2; close, fire open on the old transport (fails), assert `/api/session` fetched twice and the new transport URL contains token2; assert status `open` only after the new hello.
- `error_status_recovers_on_next_open`: error then open → hello sent, status `open`, a subsequent `send` reaches the transport.
- e2e: restart the test server with a new token while the page is open (or simulate with a token check) and assert the UI returns to Connected without a manual reload; keep `live-resync.spec.ts` green (normal replays must not double-fetch).
- Command: `cd web; npm test -- tests/apiLifecycle.test.ts`.

**Conflicts.** No sibling overlap. Must not re-fetch `/api/session` on every reconnect when the token is still valid (the session may be reused if the status is open). Coordinate with the store's `init()` (only bootstrap caller) and the `dispose()` path.

**Residual.** A server restart also loses the in-memory graph/approvals; the re-bootstrap must hydrate from history (the existing `SERVER_HELLO` → snapshot path) — LOG-01/LOG-08 govern what that looks like.

---

### LOG-05 — P1 — HUD shows live counts under a replay banner; hydration across a run switch leaves per-run surfaces stale

> **STATUS at 23:36 — OPEN (label half fixed).** The working tree adds a `DERIVED` badge (`HudBadge.tsx:45-49`); it does not fix the replay source or the run-switch reset. Weaver F3/F5 cover the same family from the panel side (`LibraryPanel` live-in-replay, capped list, HUD vs canvas count); merge the fix into one census selector as weaver F3 proposes, and add the two conditions below that weaver does not state (replay source; run-switch reset).

**Falsifiable sentence.** With `history.replayGraph !== null`, `HudBadge` derives from the **live** `tasks` and `metrics` (`HudBadge.tsx:8-11`) and Canvas still renders it (`Canvas.tsx` HEAD:500), so the HUD counts do not describe the replayed revision (and may count finished agents the canvas hides). If `performSnapshotRefresh` replaces the run/tasks, `metrics`, `toolActivity` and `approvals` are not reset (`store.ts:1169-1206`), so the HUD can show the previous run's server metrics.

**Evidence.** Probes not needed to see the wiring; code + the live 4-node fixture: replay rev 7 is 4 nodes (when scoped) while the HUD is fed the live task map and the last server RUN_METRICS (`store.ts:1518-1521`). `deriveHud` returns server values whenever `metrics !== null`, regardless of `history.replayGraph` (`metrics.ts:24-36`). The run-changed guard only protects an **active** live run (`store.ts:1107-1144`), so a terminal run can be replaced by the chosen run; `metrics`/`toolActivity`/`approvals` survive the replacement. Also see forge P2-5 (root counted by neither super nor worker) — the HUD's own total is already wrong; the replay condition makes it misleading, not merely incomplete.

**Minimal fix.** One census selector (the fix shape 02-weaver F3 proposes) so HUD, LibraryPanel, Swarm and the Canvas count agree; then:
- In replay, either derive the HUD/census from `buildReplayTasks(history.replayGraph.graph)` (read-only counts, clearly labelled "rev N") or hide the HUD / mark it "live" — do not show live counts under a replay banner. The server metrics belong to the live run, so the derived projection is the honest choice in replay.
- When `performSnapshotRefresh` changes `run.id`, reset `metrics`/`toolActivity`/`approvals` (and `approvalResolve`/`approvalSeenEpoch`) in the same set; when it keeps the run, leave them.
- Count the root (forge P2-5) and keep the `DERIVED` label that just landed.

**Acceptance.**
- `web/tests/metrics.test.ts` + a small render/store test:
  - `replay_hud_uses_replay_projection`: tasks A (live, 3 workers) + replay graph B (1 node); assert the HUD source is derived from B (or the live HUD is not rendered in replay), not from A.
  - `run_switch_resets_per_run_surfaces`: hydrate run-1, emit RUN_METRICS/TOOL_CALL/APPROVAL_REQUEST, then hydrate run-2 (terminal switch) → assert metrics/toolActivity/approvals are empty.
  - `hud_counts_sum_to_displayed_total` (forge P2-5).
- Command: `cd web; npm test -- tests/metrics.test.ts tests/logic.hud.test.ts`.

**Conflicts.** Forge P2-5 is adjacent; one HUD helper should solve both. The "server is authoritative when present" comment (`metrics.ts:1-5`) must be reconciled: authoritative **for the live run**, not for a replay. Visual gates (`gui-gates.spec.ts` / screenshots) may assert HUD text.

**Residual.** A pure "hide the HUD in replay" fix loses throughput context on the history tab; a "show replay counts" fix must not imply the revision is live. State the chosen contract in the UI text.

---

### LOG-06 — P1 — `HISTORY_SNAPSHOT` is not run-scoped; a foreign/stale replay enters this client and poisons the revision list

> **STATUS at 23:36 — FIX LANDED UNCOMMITTED; do not re-implement.** The working tree now gates on `event.payload.run_id` inside the `HISTORY_SNAPSHOT` arm (`store.ts:1702-1708`) and on `envelope.run_id` for `NODE_*` (`store.ts:1318-1328`). Probes P3/P4/P9 flipped from pass→fail at 23:36, i.e. the gate works. Remaining: the `activeRunId === null` edge still accepts a foreign snapshot; use the payload gate for `HISTORY_SNAPSHOT` (the envelope has `run_id: None` on the real wire, `ws.rs:934-940`) and add GREEN tests.

**Falsifiable sentence.** `HISTORY_SNAPSHOT` carries `run_id`, but the store ignores it (`store.ts:1691-1706`): it sets `replayGraph`/`selected`/`latest` and inserts the revision into `history.revisions`, so a broadcast from another tab's replay (or a stale response) can flip this client into replay of another run, disable the composer (`ChatCore.tsx:337,343,356`), and offer a revision that this run does not have (click → `E_HISTORY_GAP`). `RUN_CREATED` resets `history` (`store.ts:1374-1396`), but a stale snapshot arriving afterwards re-poisons it.

**Evidence.** Probes P3/P4 (exact assertions). The server publishes `HISTORY_SNAPSHOT` to the whole bus (`ws.rs:847-848` `publish_with_correlation`), so two browser tabs on one local server see each other's replays. The command itself is run-scoped (`ws.rs:832-849`), the event is not consumed that way. **Live corroboration (05-driver):** TD-02's own note records "concurrent client had forced replay 7/8", and TD-04's note records "replay is server-broadcast so concurrent client may also apply it" — the exact multi-tab class, observed before the 23:36 gate landed.

**Minimal fix.** At the top of the `HISTORY_SNAPSHOT` arm: `if (event.payload.run_id !== (get().run?.id ?? null)) return;` (optionally push a quiet refusal only for a manual request). Do **not** filter replay nodes by the live task map on the client — `replay-state.spec.ts` deliberately sends synthetic node ids with a matching run id; the server projection is the authority for the node set (LOG-02).

**Acceptance.**
- `web/tests/store.test.ts` (extend): `history_snapshot_for_another_run_is_ignored` (run id `another-run` → `replayGraph === null`, `revisions` unchanged); `stale_history_snapshot_after_run_created_is_ignored` (emit RUN_CREATED run-2, then HISTORY_SNAPSHOT run-1 → still null); `history_revisions_not_poisoned_by_foreign_revision`.
- Keep `replay-state.spec.ts` green (it sends `run_id: 'run-live'`, matching the current run).
- Command: `cd web; npm test -- tests/store.test.ts tests/graph.replay.test.ts`.

**Conflicts.** Extends forge D7 (which gates `NODE_*` on `envelope.run_id`); implement one shared `eventRunMatchesCurrent(envelope, payload)` helper rather than two copies. `HistorySnapshotPayload` has `run_id` (contract.ts) and `Envelope` has `run_id`; prefer the payload for this event (the envelope may be unset in tests).

**Residual.** A legitimate replay requested by this client always matches its own run id; the gate cannot break the History tab. If the UI ever supports browsing other runs, the gate must become a per-request correlation id, not just the current run id.

---

### LOG-07 — P1 — Rewire CAS uses a committed rev that can be older than the live graph rev

> **STATUS at 23:36 — PARTIAL.** The D3 half landed: `rewireSelected` now uses `get().run` + `rewireBlocker` (`store.ts:1912-1917`), so a terminal run refuses by name. The rev-source half is unchanged: Probe P2 still reproduces (`expected_rev:1` after live rev 5). Keep this item open for the rev source; do not re-do the guard.

**Falsifiable sentence.** `fetchNodeServerRev` reads the **latest committed** revision (`store.ts:1283-1310`) and `performSnapshotRefresh` copies committed node revs into the live `tasks` map (`store.ts:1175-1177`, `recordsFromSnapshotGraph:582-645`). The server CAS compares the **live** node rev (`graph.rs:147-158`), so after a hydration the client sends `expected_rev` values that are lower than the server's; the rewire is refused `E_RESYNC_REQUIRED`, the resync returns the same committed (stale) value, and a retry repeats. Probe P2: live rev 5 → hydration rev 1 → `REWIRE_TASK{expected_rev:1}`.

**Evidence.** Probe P2; `store.ts:1495-1497` (live `NODE_*` rev, `NODE_DRAG` preserves; snapshot rev copied), `1283-1310` (fetch), `1922-1947` (send), `graph.rs:151-155` (CAS). The store's own comment calls the committed revision "the only authority" (`store.ts:1888-1893`) — that is true for the *history* render, not for a CAS against the live graph.

**Minimal fix.** Distinguish confirmed-live rev from committed rev in `TaskRecord` (e.g. `rev` = live-confirmed; `committedRev` = history), or:
- On a CAS `E_RESYNC_REQUIRED` refusal, re-read the node rev from the **live** event stream only; never retry with the same committed value (no loop).
- If the live rev is unknown, keep the rewire control disabled with `unknown-server-rev` (the existing honest state) instead of fetching a committed value and pretending it is confirmed.
- Coordinate with forge D3: `rewireSelected` should funnel through `rewireBlocker` (terminal → `E_RUN_TERMINAL`, not `E_RUN_NOT_FOUND`) and must not reintroduce a guessed rev.

**Acceptance.**
- `web/tests/store.test.ts`: `rewire_uses_live_confirmed_rev_not_committed_rev`: `NODE_STATE rev=5` → hydrate committed rev=1 → `rewireSelected`; assert either (a) the sent `expected_rev` is 5, or (b) no command is sent and a named `E_RESYNC_REQUIRED`/unknown-rev refusal is shown — never `expected_rev:1`.
- `rewire_after_refusal_does_not_repeat_the_stale_value`: a server `E_RESYNC_REQUIRED` response must not trigger a second send with the same rev; assert the command count.
- Keep the live e2e stale-rev probe (forge §0.3) refusing.
- Command: `cd web; npm test -- tests/store.test.ts`.

**Conflicts.** **Merge with forge D3** (same function); D3's test expects `E_RUN_TERMINAL` for a terminal run — preserve it. The comment at `store.ts:1888-1893` must be rewritten, not ignored. A fix that always uses the latest committed rev (the current intent) cannot pass the live CAS; a fix that always uses the last live event rev needs a "rev is live-confirmed" bit so a snapshot doesn't downgrade it.

**Residual.** If a node's rev changes between the live read and the send, the CAS still refuses (correct); the UI must show the refusal and refresh, not loop.

---

### LOG-08 — P1 — Hydration and revision-read failures are invisible; empty states present as the truth

**Falsifiable sentence.** A failed `performSnapshotRefresh('boot'|'reconnect')` or a failed `refreshCommittedRevisions` leaves a named refusal in `snapshot.refusal`/`history.refreshRefusal`, but no component reads either field; the UI renders as Connected with "No run yet" / "No committed revisions yet", which is indistinguishable from a genuinely empty workspace, and the operator can submit a prompt that the server then refuses with `E_RUN_ACTIVE`.

**Evidence.** §3.4 grep receipts; **02-weaver F4 independently lists the same unused state** (`DerivedBadge`, `ReplayBanner`, `healthVersion`, `fileTree.refusal`, `history.refreshRefusal` stored/exported but never rendered). `store.ts:1211-1231` (catch sets failure; `pushRefusal` only for `resync|refusal|manual`, so `boot`/`reconnect` are silent; connection unchanged except for resync/refusal), `store.ts:1016-1027` (history refresh error), `HistoryPanel.tsx:20-24` and `Scrubber.tsx:30,62-64` (empty copy), `isSnapshotDegraded` 749-756 (dead), `App.tsx:43-48` (banner only from `connection`/`connectionRefusal`).

**Minimal fix.** Use the existing `ToastRegion`/banner (no new component):
- On a failed hydration with source `boot|reconnect`, push the named refusal (or set `connectionRefusal`) and render a distinct "history unavailable" state in `HistoryPanel`/`Scrubber` instead of the empty copy.
- On `history.refreshStatus === 'error'`, render the refusal next to the scrubber / in the history panel and offer Retry (calls `refreshRunSnapshot('manual')`); a terminal run's final revision must not be silently absent.
- Remove or wire `isSnapshotDegraded`; do not leave dead state.

**Acceptance.**
- `web/tests/logic.errors.test.ts`: `boot_hydration_failure_is_visible`: fake fetch 503 for `/api/history/runs`, init → assert a refusal toast/banner exists and the "No run yet" empty state is replaced by the failure copy; `reconnect_hydration_failure_is_visible`; `terminal_revision_refresh_failure_is_visible`: `refreshCommittedRevisions` 503 after RUN_STATUS completed → assert `history.refreshStatus==='error'` is rendered + retry control.
- Command: `cd web; npm test -- tests/logic.errors.test.ts`.

**Conflicts.** The `refreshStatus` field is documented "quiet (no toast)" (`store.ts:999`); this fix changes that for a terminal run whose scrubber would otherwise lie. Keep the quiet path for a successful mid-run refresh. Visual gates may need a new empty-state string; use `STRINGS`/`StatePack.EmptyState`.

**Residual.** A boot-time server-down still needs a reload (or LOG-04's re-bootstrap); this item only makes the state visible, not self-healing.

---

### LOG-09 — P1 — Terminal status is published before the terminal snapshot is committed

**Falsifiable sentence.** `dispatch()` publishes the terminal `RUN_STATUS` inside the lifecycle lock (`execution.rs:1282-1303`) and only afterwards calls `commit_run_snapshot(..., final_status)` (`execution.rs:1312`). The client reacts to the status by calling `refreshCommittedRevisions` (`store.ts:1423-1428`), which can read the revision index before the terminal revision exists; nothing retries, so the scrubber can permanently lack the final revision until a reload or another revision-changing event.

**Evidence.** Code order is decisive: publish at 1293-1301, commit at 1312; `snapshot_revision` is computed at commit time (`execution.rs:1174-1180`), i.e. the terminal revision is not even indexed before the publish. `http.rs:639-663` reads the index. The race window is small (the commit is synchronous right after the publish) but it is a real ordering bug; this seat did not reproduce it at runtime (server exited) — label **code-order defect, runtime race unverified**.

**Minimal fix.** Commit the terminal snapshot **before** publishing the terminal `RUN_STATUS` (or publish a `RUN_STATUS` carrying `latest_revision` and have the client use it), keeping the single-owner lifecycle semantics:
- move `commit_run_snapshot(run, snapshot_revision(run.id), final_status)` before the `with_lifecycle_lock` terminal publish; the snapshot status is already known (`final_status`); no double publish.
- Alternatively, make the client's terminal refresh retry once when `history.count` did not increase after a terminal status (bounded, no loop).

**Acceptance.**
- `backend/crates/hiverouter-core/tests/run_lifecycle.rs` (new): subscribe to the bus, capture the `RUN_STATUS` frame, then immediately read `indexed_revisions(run_id)`; assert the terminal revision is already present at the moment the frame is observable (or assert the client-side retry).
- `web/tests/store.test.ts`: `terminal_refresh_retries_once_when_no_revision_was_added`: fake revisions response missing the new revision on the first read, present on the second; assert one bounded retry and a populated scrubber.
- Command: `cd backend; cargo test -p hiverouter-core --test run_lifecycle`; `cd web; npm test -- tests/store.test.ts`.

**Conflicts.** Reordering the terminal publish may affect tests that assert frame order (NODE_STATE settlements before RUN_STATUS, `run_lifecycle.rs`); the terminal snapshot commit is currently after the lifecycle lock to avoid holding the lock across fs I/O — keep the lock ordering but move the commit only if it cannot interleave with a second terminal publish. A client-side retry is the lower-risk half; do both only if the ordering test proves safe.

**Residual.** If the snapshot write fails, the run is still terminal; the client must show the failure (LOG-08) rather than retry forever.

---

### LOG-10 — P1 — `messages-unavailable` is fabricated in the run-changed guards; truncation is silent

**Falsifiable sentence.** Both run-changed guard branches set `incomplete: ['run-changed','messages-unavailable']` (`store.ts:1121`, `1137`) even when the `/messages` read succeeded just before the guard (and even when it was never attempted); separately `history_messages` returns the **oldest** 1000 rows (`http.rs:777-792`, `messages.into_iter().take(MAX_MESSAGES)`) and reports `truncated`, but the client decoder ignores `truncated` (`store.ts:505-529`), so a long run's transcript is silently the wrong 1000 messages.

**Evidence.** Probe P5 (messages fetch count +1, read 200, yet `messages-unavailable`); code lines above. `http_boot.rs::history_messages_route_returns_persisted_transcript` only checks a 2-message happy path and `truncated:false`; no test for truncation or the guard.

**Minimal fix.**
- Push `messages-unavailable` only when the messages read actually threw (the `catch` at `store.ts:1082-1086` already exists and sets `messagesUnavailable=true`); the guard branches must carry the **observed** flags, not a hardcoded pair. If the guard means "the messages are for another run", use a distinct honest marker (or omit it).
- Decide truncation semantics: return the **newest** N with a `truncated` flag and either (a) show a named "transcript truncated" notice in the chat, or (b) refuse to hydrate an incomplete transcript and surface `E_HISTORY_TRUNCATED`. Do not silently claim completeness. The client decoder must not reject old payloads (additive handling).

**Acceptance.**
- `web/tests/store.test.ts`: `run_changed_guard_reports_only_observed_incomplete`: successful messages read → `incomplete` does **not** contain `messages-unavailable`; failed read → it does.
- `web/tests/logic.errors.test.ts`: `truncated_history_is_surfaced`: `/messages` returns `truncated:true`; assert a visible notice/refusal and that the messages are the newest N (or that hydration fails closed).
- Backend `http_boot.rs`: extend the test with 1001 messages, assert `truncated:true` and that the returned window is the newest (decide and pin).
- Command: `cd web; npm test -- tests/store.test.ts tests/logic.errors.test.ts`; `cd backend; cargo test -p hiverouter-core --test http_boot`.

**Conflicts.** Changing the truncation window is a contract-visible behavior change; update `http_boot` and the chat UI together. The `messages-unavailable` marker is used by any future UI (LOG-08); keep the type stable.

**Residual.** 1000 messages is already a bounded UI; a cursor/pagination design is larger than this audit.

---

### LOG-11 — P1 (additional, live-verified) — A run failure is settled as a successful submit: draft cleared, optimistic prompt retained, failure only in the run panel

**Falsifiable sentence.** For a keyless `SUBMIT_PROMPT`, the server ACKs and creates a run, then the run fails `E_KEY_MISSING`; the client's submission settlement only looks for a **command** refusal (`findSubmissionRefusal`), so it clears the draft and keeps the optimistic user row while the only failure surface is the run-error panel.

**Evidence (live, 05-driver seq 10-11, 23:36-23:37).** `optimisticMessageEcho=True`, `composerValueAfter=[]`, toast `RUN REFUSED / E_KEY_MISSING`, `runLine=[]`; then "new run id 01A1B9A8, scrubber 1 OF 1, HUD NO METRICS, graph 0/0, RUN REFUSED E_KEY_MISSING panel, chat contains user message keyless check, composer empty". Code: `ChatCore.tsx:132-169` (settlement heuristic), `store.ts:1860-1875` (`submitPrompt` appends the optimistic row then `run()`), `store.ts:1408-1440` (`RUN_STATUS` failed writes `run.errorCode` and flips streaming off, but does not publish a toast), server `submit_prompt` ACKs before dispatch and the planning failure publishes a run status, not an ERROR frame (`execution.rs:1255-1312`; `E_KEY_MISSING` path in the provider/planning call).

**Minimal fix.** Settle the submission on the **run** outcome as well as the command: if the run created by this submission fails before any node executes (no `NODE_SPAWN` for the new run, `errorCode` present), restore the draft (or mark the optimistic message refused and offer retry) and surface a toast tied to the prompt. Keep the persisted user message (it is real) but do not present it as accepted-and-queued success. Do not weaken the normal ACK-before-mutation rule.

**Acceptance.** `web/tests/logic.submit.test.ts`: `keyless_run_failure_restores_draft_or_marks_message`: submit a prompt, emit `RUN_CREATED` then `RUN_STATUS failed/E_KEY_MISSING`; assert the draft is restored **or** the optimistic row carries a failed marker and a visible named refusal; assert the draft is not silently empty while the row reads as accepted. Driver-level: repeat TD-03 against the new build and read back composer/row/toast.

**Conflicts.** `ChatCore.tsx` is being edited by another seat for the show-chat occlusion (z-index only); the settlement logic is untouched, but re-pin. Do not turn every run failure into a draft restore (e.g. a failed tool turn after work executed should not re-open the composer with the original prompt); the "no node executed" predicate is the boundary.



Sibling reports reviewed as they appeared: `02-weaver.md` (B), `03-forge.md` (C), `04-capability.md` (D), `05-driver.md` (E). This section maps my queue to theirs, records the new interactions, and gives the manifest sequencing.

### 5.1 Direct merges (one fix, several reports)

| My item | Sibling item(s) | Merge instruction |
|---|---|---|
| LOG-02 | **Forge D1 + D6**; **Capability CAP-P2-5** (replay snapshot drops kind) | One `history_graph_value`-style projection used by both reads. My additions: the **fail-open** `unwrap_or_default` + `if !allowed.is_empty()` (now fixed in the working tree), the test-gap evidence (the existing HTTP test passes because its task set is empty; the live e2e only asserts `>= 2`), and the DB-error-silent-empty residual. |
| LOG-03 | **Forge D5** (recovery captures unscoped; parentless `add_task`) | One recovery fix. My additions: the **tagging** interaction (a changed run whose nodes are absent from the restored image gets another run's graph as its new revision; then the scoped HTTP read filters it to zero while the WS read shows foreign nodes) and the explicit `changed_runs` set. |
| LOG-07 | **Forge D3** (terminal-run `E_RUN_NOT_FOUND`; drag position persisted) | Same `rewireSelected` rewrite; the D3 guard half is already in the working tree (`store.ts:1912-1917`). LOG-07 fixes the **rev source** after hydration. Neither alone makes a live reparent reliably succeed. |
| LOG-06 | **Forge D7** (client accepts `NODE_*` from any run; role lost) | One run-scoped event gate; use a shared helper for `NODE_*` (envelope `run_id`) and `HISTORY_SNAPSHOT` (payload `run_id` — the envelope is `None` on the real wire). The working tree now has both gates; verify with P3/P4/P9 as GREEN. |
| LOG-05 | **Forge P2-5** (HUD total > displayed counts); **Weaver F3** (Active Agents predicate/cap) + **F5** (HUD/panel/canvas count inconsistencies) | One census selector (weaver F3's shape) used by HUD, LibraryPanel, Swarm and the Canvas count; then add the replay source and the run-switch reset (my two conditions). The `DERIVED` label half already landed. |
| LOG-08 | **Weaver F4** (`history.refreshRefusal`, `DerivedBadge`, `ReplayBanner`, `healthVersion`, `fileTree.refusal` unused) | One visible error/empty state; do not add a second unused badge. |
| P2-2 (`chat.pinned` dead) | **Weaver F4** (`pinChat`, diff modal have no caller) | Remove or wire; do not list it as a capability. |
| Show-chat (no queue item) | **Weaver F2** | Weaver proves the state toggle works and the "forever" perception is occlusion. The working tree is already fixing the two controllable occlusions (`ChatCore.tsx:114` z-30, `ToastRegion.tsx:16` top-16); only the workspace-only contextual toggle remains as a UX note. **Do not queue a persistence fix.** |

### 5.2 New interactions the sibling reports did not record (beyond LOG-01..11)

1. **Hydration × forge D2.** Forge D2 wants `hideFinished` false so a finished run is visible. LOG-01's hydration can drop the very nodes D2 is meant to show (post-commit spawns). Fix D2 and LOG-01 in the same slice or the canvas can still go empty on a resync.
2. **Hydration × forge D1/D6.** The moment WS replay is scoped/enriched, a resync that calls `performSnapshotRefresh` will still overwrite the live graph with a *committed* projection; scoping the replay does not make hydration safe. LOG-01 is not obsoleted by D1/D6.
3. **`RUN_CREATED` × approvals.** A fresh run resets tasks/metrics/history, but leaves `approvals`, `approvalResolve`, `approvalSeenEpoch` and `approvalsStale` untouched (`store.ts:1374-1396`). A run cancelled while awaiting approval leaves an overlay card; `cancel` only calls `tools.cancel_approval` and publishes no `APPROVAL_RESOLVED` (`execution.rs:2402-2415`), so the card can block the next run until TTL/dismiss. Cross-run UI leak; put it in the manifest backlog (P2-7).
4. **Replay × HUD.** Fixing forge D2 (show cards) without LOG-05 leaves the HUD counting live metrics under a replay banner; fixing LOG-05 without D2 leaves a replay graph whose kind is missing (D1). None is sufficient alone.
5. **Terminal status × scoped read.** LOG-09's race is *more* visible after LOG-02: a scoped HTTP read that correctly returns 4 nodes but misses the final revision still offers a stale scrubber. Keep the terminal-revision invariant in the acceptance of both.
6. **`HISTORY_SNAPSHOT` × envelope run id (forge D7).** The server publishes `HISTORY_SNAPSHOT` through `publish_with_correlation` → `publish_for_run(event, None, ...)` (`ws.rs:934-940`), so the envelope `run_id` is **None**; only `payload.run_id` can gate it. Probe P9 proves the envelope gate ignores a foreign `NODE_*` but cannot see the replay event. The working tree now has both gates; keep the payload one even if the envelope later carries the run id.
7. **`/api/session` × reconnect.** Forge's D7/§0.2 and the driver's environment note both show the served bundle can be older than the source. LOG-04's restart path is only exercised when the page outlives the server; the e2e harness restarts its own server per test, so a browser-level restart test is needed. The 05-driver observed a real replacement at 23:29 (pid 15484 → 22380) while a browser was open — the exact condition, though it re-navigated instead of asserting recovery.
8. **Scoped snapshot × `health.graph_nodes`.** After a restart with a scoped snapshot, the process graph holds only the newest run (35 → 4 in this workspace), so `/health.transport.graph_nodes` changes meaning; if any UI/claim uses it as a run counter, it is now wrong. Not audited in depth (no UI consumer found); record as a cleanup.
9. **`history_messages` × run switch.** The store hydrates messages only for the chosen run (LOG-10/LOG-01); after a run switch the transcript is replaced even if the user considers it "their chat". LOG-01's guard should keep the operator's transcript when the run id changes (the transcript is already kept on live RUN_CREATED), but the committed transcript of another run must not silently replace it.
10. **Show-chat state vs affordance (weaver F2).** The persistence state is sound; the "forever" perception is three occlusions (reopen pill under the HUD, header toggle under toasts, global toggle with workspace-only ChatCore). The working tree is fixing the first two. This is a UX/affordance item, **not** a persistence defect; do not spend a queue slot on it.
11. **Active-agent census (weaver F3/F5).** LibraryPanel's predicate/cap and the HUD/canvas inconsistencies are the same family as LOG-05. One selector; include the replay source and the run-switch reset that weaver does not state.
12. **Capability CAP-P2-5 (replay drops kind).** Same root as LOG-02; the projection fixes it. Do not create a separate "add kind to the snapshot file" change unless the durable format is being versioned anyway.
13. **Driver environment + live error paths (05-driver).** The server was replaced at 23:29 with a bundle built 23:25:52, which predates the 23:30–23:36 source fixes. The driver's baseline (0/4 cards on a completed run, HUD `3 WORKERS` vs minimap `NO NODES YET`) is live proof of forge D2/LOG-05; it is **not** validation of the uncommitted scoping/gate fixes. The driver also produced live evidence for two of my items: **LOG-04's dead-end transport error** (`CONNECTION FAILED` / `WEBSOCKET TRANSPORT ERROR.` while `/health` 200; no recovery) and **LOG-11** (keyless run failure settled as a successful submit: draft cleared, optimistic row retained, failure only in the run panel). Also: the driver's "Show chat" click → the chat core unmounted and the header toggle flipped to Show — consistent with my state-sound verdict.
14. **In-flight fail-closed × existing backend test.** `history_graph_value` now returns an empty graph when the run has no task rows. `snapshot_history.rs::history_http_api` (`:150-184`) creates no task rows and asserts 2 nodes → it will go red. Update it to seed task rows (that is the point of the scope gate); keep `history_api_returns_real_revisions` (read-model only, still 2 nodes) as-is. Add this as a **test-update** item in the manifest, not a product change.
15. **In-flight gate × no-run edge.** The payload gate is `activeRunId !== null && payload.run_id !== activeRunId`; with no run it still accepts a replay snapshot. Decide whether replay without a run is legal (App renders the Canvas only when `run !== null`, so it is invisible today) or ignore it; document either way.

### 5.3 Conflict/sequencing rules for the manifest

1. **LOG-02 and LOG-06 are already in the working tree at the 23:36 freeze** — carry them as **verify + tests**, not new fixes. LOG-03 remains first for new work (server truth), then LOG-01/LOG-07/LOG-08/LOG-09/LOG-10, then LOG-05/UX. A client-side scope fix before the server projection would require a second authority and would be thrown away.
2. **Never gate replay nodes by the live task map on the client** — `replay-state.spec.ts` sends synthetic node ids with a matching run id; gate only on the event run id (LOG-06).
3. **LOG-07 and forge D3 touch the same function**; land one patch. Preserve D3's `E_RUN_TERMINAL` assertion and do not reintroduce a guessed rev.
4. **LOG-09 reordering** must not move the terminal publish outside the single-owner lifecycle lock; if the commit cannot be moved, implement the bounded client retry only.
5. **LOG-02's fail-closed decision** must be reflected in the client: an empty/refused replay is an empty replay, not a crash; `buildReplayTasks` already returns `{}` for malformed input. Also update the backend test that assumes the old fail-open.
6. **Line-number drift**: the store grew by +11/+17 after the 23:36 gates; `http.rs`, `ws.rs`, `Canvas.tsx`, `graph.ts`, `ChatCore.tsx`, `HudBadge.tsx`, `ToastRegion.tsx` were edited concurrently. Re-pin before applying; prefer the function names in §4 over raw lines for the moving files.

---

## 6. Appendix — P2 minors and dead paths (recorded, not in the top 10)

| ID | Defect | Cause | Minimal fix / decision |
|---|---|---|---|
| P2-1 | A background snapshot refresh silently ends replay (`replayGraph: null`, `selected: latest`) without the explicit `exitHistoryReplay` gesture; the operator loses their place and the only clue is the banner disappearing | `store.ts:1183-1192` vs `2161-2175` | Either preserve the replay revision if it still exists (like `applyHistoryRevisionList` does) or push a named "Replay ended: reconnected" toast; document the contract |
| P2-2 | `chat.pinned` is dead state: `pinChat` has no caller, nothing persists it, and `CHAT_PIN` is a server ACK-only command | `store.ts:1786`, `2124-2129`; `ws.rs:737-742` | Remove `pinned`/`pinChat` or wire a real pin control + persistence; do not leave an implied capability |
| P2-3 | `graphRev` is a write-only counter (incremented 7×, read nowhere); `isSnapshotDegraded` is exported and unused | `store.ts:254`, `1182,1393,1421,1501`, `749-756` | Remove or consume; do not present it as a revision authority (the comments correctly say it is not a CAS rev) |
| P2-4 | `NODE_DRAG` is decoded/handled but never produced; positions are client-local only (forge P2-4) | `ws.rs:969` (match arm only); `store.ts:1461-1512` | Remove/allowlist; the drag is a local layout gesture until a rewire lands |
| P2-5 | A stale `NODE_REWIRE` from another client can consume `pendingRewire` and attribute the wrong parent when the payload omits `parent_task_id` | `store.ts:1273-1280`, `1474-1498` | Match the pending intent to the correlation/ACK or expire it; do not attribute a foreign rewire |
| P2-6 | `HistoryPanel`/Scrubber show "No committed revisions yet" for a run whose revision read failed (see LOG-08) | `HistoryPanel.tsx:20-24`; `Scrubber.tsx:30` | Covered by LOG-08; do not duplicate |
| P2-7 | `RUN_CREATED` resets tasks/metrics/history but **not** approvals/approvalResolve/approvalSeenEpoch/approvalsStale; a cancelled awaiting-approval run can leave a blocking card into the next run | `store.ts:1374-1396`; `execution.rs:2402-2415` (cancel does not publish `APPROVAL_RESOLVED`) | Clear/mark stale approvals on a fresh run, or publish the resolution on cancel; add a store test |
| P2-8 | `refreshCommittedRevisions` is fire-and-forget and has no retry; a transient failure leaves the cursor stale until another event | `store.ts:1423-1428`, `1000-1028` | Covered by LOG-08/LOG-09; add a bounded retry if LOG-09's ordering fix is not taken |
| P2-9 | `history_messages` returns the **oldest** 1000 (`take`) while the client ignores `truncated`; a long transcript is silently wrong | `http.rs:777-792`; `store.ts:505-529` | Covered by LOG-10 |
| P2-10 | A transport `error` after open blocks all commands until close/reload even if the socket is open (P8) | `client.ts:340-345` | Covered by LOG-04's error-recovery edge |

**Show-chat audit result (explicit).** `initialChatVisible`/`setChatVisible` are correct and symmetric; the server's `CHAT_PIN` no-op is intentional and harmless while no UI calls it; no persistence defect was found. The only related defect is the dead `pinned` field (P2-2) and the Header/Canvas/ChatCore toggles are consistent. Do not spend a fix slot on show-chat.

---

## 7. Receipt

### Done (evidence or it is not done)

| Item | Evidence |
|---|---|
| Read the assigned surfaces and state machines (store, Canvas/graph, ChatCore, Header/Hud/History/Scrubber/Library/Agents, client, contract, metrics; ws/http/graph/snapshot/recovery/execution) | hashes §2.2; line citations in §4; HEAD `240fe0f` |
| Built a deterministic RED harness for the store/client edges (no product edit) | `%TEMP%\seatf\probe`; 8/8 pass at 23:29 (audited code); 9 tests at the 23:36 freeze with 6 pass / 3 flipped green by the in-flight gate; both outputs §3.2 |
| Re-pinned after the working tree changed and recorded the delta | §2.2.1 (freeze hashes, in-flight fixes, line drift) and §0.1 (status corrections); live `/health` re-probe at 23:36 |
| Proved the 4-node scoped-snapshot disagreement and the fail-open boundary | §3.1 (35/26 vs 4/3; 31 foreign; topology-only fields) + `http.rs:682-691` + `ws.rs:832-849` |
| Pinned exact file:line for the 10 queue items and 10 P2 minors | §1, §4, §6 |
| Produced acceptance tests (RED on today's code) per item, with commands | §4; a ready RED harness exists outside the repo |
| Reviewed every sibling report as it appeared and recorded merges/conflicts/missed interactions | §5 (forge D1/D3/D5/D6/D7/P2-5; weaver F2/F3/F4/F5; capability CAP-P2-5; driver environment; 15 new interactions) |
| Live corroboration from 05-driver for two findings | `05-driver.md` seq 10-11 (LOG-11: keyless run failure settled as submit success) and seq 17 (LOG-04's dead-end transport error, `/health` still 200); baseline seq 2 for forge D2/LOG-05 |
| Verified sound areas and recorded them (no false positives) | §3.3 (cycles, CAS guard, chat persistence, revision clamp, decoders, resync serial, tool ordering, RUN_CREATED reset) |
| Kept the repo clean of my writes | only `docs/trace/orca-wave/06-logic.md`; `git status` shows only other seats' edits |

### Not done (and why)

| Item | Reason |
|---|---|
| No product fix applied | This seat is audit-only by instruction ("only write `docs/trace/orca-wave/06-logic.md`, no product edits/restarts"). The queue is a candidate for the Lead's manifest. |
| No backend RED run (`cargo test` with new tests) | Adding tests would require a repo write; this seat may only write the report. The tests are specified for the implementing seat. |
| No deterministic reproduction of LOG-09 (terminal revision race) | Requires an instrumented snapshot store / live server; code order is decisive but the runtime race is unproven (see Unverified). |
| No live WS `HISTORY_REVISION_SET` re-probe | The first server was replaced mid-audit (pid 15484 → 22380 at 23:29); sending it would also have changed the live UI. The raw snapshot + forge's captured live result + the code path are the evidence. |
| No provider call, no key read | Keyless by design (`E_KEY_MISSING`); out of scope. |

### Unverified (non-empty)

| Claim | What would prove it |
|---|---|
| LOG-09's race is reachable at runtime (terminal revision missing from a client's scrubber) | Instrumented `commit_run_snapshot` delay + a WS client observing RUN_STATUS then reading `/revisions`; or a repeated live run under load with a revision-count assertion |
| LOG-03's cross-run recovery tagging on this exact workspace | A two-run recovery fixture (or a crash/restart with a second interrupted run) asserting every node in the new revision ∈ `tasks_for_run` |
| LOG-04's restart path end-to-end | Restart the server while a browser page stays open (token rotation) and assert auto-recovery; P7/P8 prove the client state machine, not the browser network path |
| LOG-02's HTTP result on the current binary | The first state GET failed across the 23:29 server replacement; forge's 23:22 capture (`nodes=4`) and the on-disk 35 are the two halves. The new process/bundle (23:25:52) predates the 23:33–23:36 fix, so it cannot prove the fix; a re-run of the acceptance tests is required. |
| LOG-05's HUD rendering | A rendered-browser test with a replay graph + live tasks; this seat proved the data wiring, not the pixels |
| LOG-10's truncation UI contract | A product decision (newest-N + notice vs fail closed) must be made before the test is written |
| The working tree equals the audited hashes at fix time | At the 23:36 freeze the store is `696C46BF…`, http `A3E52E55…`, ws `6B644242…`; they were still moving. Re-hash before applying; use §2.2.1. |
| The in-flight LOG-02/LOG-06 fixes compile and their tests pass | The working tree has not been built/tested by this seat (audit-only; no product edits). Run `cargo test -p hiverouter-core --test snapshot_history` and `npm test` after re-pinning; the first will expose the no-task-rows HTTP test. |

### Stub sweep (§23)

```
Select-String -Path "docs\trace\orca-wave\06-logic.md" `
  -Pattern 'TODO|FIXME|XXX|HACK|placeholder|NotImplemented|not implemented|mock|stub|dummy|lorem|simplified for|for illustration'
```

Result at 23:37: **3 hits, all self-references** — the `### Stub sweep` heading, the pattern line above, and the word "stub" in the result sentence. Classification: **permitted** (documentation self-reference). Product files were not modified by this seat, so no product-code sweep scope applies: **zero product-code hits**.

---

## 8. Re-run instructions (for the implementing/gate seats)

1. Re-pin: `git rev-parse HEAD`; hash the files in §2.2; if `http.rs`/`ws.rs`/store/Canvas/graph/UI changed, re-read the cited lines. The 23:36 freeze hashes are in §2.2.1.
2. Red leg: re-run the probe harness:

   ```
   cd C:\Users\Riley\AppData\Local\Temp\seatf\probe
   node "C:\Users\Riley\Desktop\hiverouter-v1\web\node_modules\vitest\vitest.mjs" run --root .
   ```

   At the 23:36 freeze: **9 tests, 6 pass / 3 fail** — the 3 failures are P3/P4/P9, i.e. the LOG-06 gate now works (they were RED before it). P1/P2/P5/P6/P7/P8 still pass = still RED for LOG-01/04/05/07(rev)/08/10. Port those six into `web/tests/` as the failing assertions; port P3/P4/P9 as GREEN regression tests with corrected gate assertions. After the remaining fixes, all 9 should pass in the fixed direction (assert the *fixed* behavior, not the original bug).
3. Green leg: run the acceptance command in the item, then the project gates (`cd web; npm test; cd backend; cargo test -p hiverouter-core`), then the e2e replay/resync specs. Remember `snapshot_history.rs::history_http_api` must be updated to seed task rows for the new fail-closed projection.
4. Do not hand-edit this report's queue after the manifest lock; the Lead owns the manifest and the ordering in §5.3.

**One-line answers for the integrator**

- **4-node scoped snapshot:** at HEAD, scoped on write + HTTP read, unscoped on WS replay + recovery, fail-open at the HTTP boundary; at the 23:36 freeze the WS read + fail-closed projection are fixed uncommitted, recovery is still open (LOG-02/03; merge D1/D5/D6; verify + tests).
- **History hydration:** can silently destroy a live stream and live-only graph nodes; it needs a guarded transition + a live-gap marker (LOG-01).
- **Run terminal vs replay:** the foreign-replay entry was fixed uncommitted at 23:36 (LOG-06, verify/GREEN tests); the terminal snapshot is still published before its commit (LOG-09).
- **CAS/drag/reparent:** the server cycle/CAS guard is sound; the client can still feed it a stale committed rev (LOG-07); forge D3's terminal-refusal half landed in the working tree.
- **Show-chat:** state sound (weaver F2); the occlusion fixes are landing; `pinned` is dead (P2-2).
- **Active filtering/HUD:** forge D2 + weaver F3/F5 are the panel side; the HUD still shows live counts in replay and across run switches (LOG-05).
- **Refresh/WS reconnect:** cannot survive a server restart (token rotates, no re-bootstrap) (LOG-04); the 05-driver observed a real replacement at 23:29.
- **Empty/error states:** hydration failures are invisible because no component reads `snapshot`/`refreshStatus` (LOG-08); weaver F4 found the same unused state.
- **Submit refusal semantics:** a keyless run failure is settled as a successful submit (draft cleared, optimistic row retained, no toast) — LOG-11, live-verified by 05-driver.
