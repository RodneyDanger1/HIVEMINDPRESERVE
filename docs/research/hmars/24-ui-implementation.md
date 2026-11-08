# 24 — UI/UX implementation audit: the running app against the operator's answers and the wave-2 goal

> **Evidence-gate block (replacement seat, 2026-11-07 22:20 CST).** The audit body was written by the
> original seat 21:52–22:12; the replacement appended §16–§21 at 22:25. This block covers the whole
> file; the body's read set is hashed in §14.2, and every file the appended section cites is hashed
> below. The server was up at 22:07 and refused TCP at 22:18; no restart was performed.

```yaml
gate: EVIDENCE-GATE-v1
lane: 24 — HMARS UI implementation (current truth, operator matrix, change set)
lane_id: wave3-seat-24-ui-implementation
date_local: 2026-11-07T22:25:00-06:00
date_utc: 2026-11-08T04:25:00Z
workspace: C:\Users\Riley\Desktop\hiverouter-v1
head: bd575ba7adf3a5f2a9bdbc69f9bbf7084fe6fe65
working_tree: dirty
working_tree_hashes:
  backend/crates/hiverouter-core/src/execution.rs: F02F032A99CAD91E
  backend/crates/hiverouter-core/src/snapshot.rs: 2954FF374BE2DBC6
  backend/crates/hiverouter-core/src/http.rs: F52B83B0AD889DF6
  backend/crates/hiverouter-core/src/db.rs: 8EAD70812ED67D47
  backend/crates/hiverouter-core/src/tool_loop.rs: 8D924F9E969B9F95
  backend/crates/hiverouter-core/src/caps.rs: D21061F2FA3C5A53
  backend/crates/hiverouter-core/src/graph.rs: 8B3A24A29E425EE4
  backend/crates/hiverouter-core/src/recovery.rs: B35493C23123EAB3
  backend/crates/hiverouter-core/src/ws.rs: F036F47123ACAA94
  backend/crates/hiverouter-tools/src/approval.rs: 0B6FA92801FEA59B
  backend/crates/hiverouter-contract/src/event.rs: 939FDECADBD7F003
  backend/crates/hiverouter-contract/src/refusal.rs: 9C99C4475DF6CBCB
  backend/crates/hiverouter-provider/src/client.rs: 5CD021A02665D6A2
  backend/crates/hiverouter-provider/src/budget.rs: 0B685A5ED36B97DC
  web/src/state/store.ts: 3C222E738E668DFA
  web/src/state/strings.ts: 2ACDB02DC770BCBC
  web/src/api/contract.ts: 4E66EB111F8B7FB2
  web/src/api/refusals.ts: DC715B3AE2480144
  web/src/panels/ApprovalOverlay.tsx: 2E360B16AE719FE1
  web/src/panels/AgentMessagesPanel.tsx: FF9C7787CCA21DB3
  web/src/panels/AgentsPanel.tsx: 9AC74F55E52FF6EA
runtime:
  url: http://127.0.0.1:8099
  reachable_at: 2026-11-08T04:07:00Z
  last_good_probe_at: 2026-11-08T04:16:00Z
  write_time_at: 2026-11-08T04:18:02Z
  last_seq: 18
  graph_nodes: 35
  schema_version: 10
  active_run_id: null
  active_run_status: null
  paid_calls: none
  credentials_read: none
  product_files_edited: none
  files_written: [docs/research/hmars/24-ui-implementation.md]
```

**File:** `docs/research/hmars/24-ui-implementation.md`
**Author:** wave-3 seat 24 (UI/UX implementation audit), read-only seat
**Date/time:** 2026-11-07, 21:52–22:12 America/Chicago (all capture timestamps below are local)
**Repository:** `C:\Users\Riley\Desktop\hiverouter-v1`, HEAD `bd575ba` ("feat: real SuperAgent/Subagent personas and persona prompt execution", 2026-11-07 19:28:54 -0600) **plus concurrent uncommitted edits** (§1.2)
**Live app under audit:** `http://127.0.0.1:8099` — `hiverouter.exe` built 22:00:18, process started 22:00:32, serves the SPA from `web/dist` bundle `assets/index-DzPltEYs.js`
**Deliverable scope:** this file only. No product file, test, bundle, binary, database row, or server process was modified or restarted by this seat.

---

## 0. Method, discipline, and claim classes

### 0.1 Primary discipline statement

`/deep-inspection` is **not installed** in this session's skill catalog (checked `~/.dsh/skills`: the available `deep-*` skills are `deep-investigate` alone, plus `trace`). Per the seat instruction I used the closest available pair and state the substitution here:

- **`/deep-investigate`** — baseline → ≥3 ranked hypotheses → cheapest discriminating probe per hypothesis → reproduce → confirm mechanism. Applied to four runtime questions this pass (§0.4 lists the four, with the elimination evidence and the two **probe corrections** that changed my conclusion).
- **`/trace`** — 11-hop ladder (render → handler → client → request → route → service → store → provider → response → render-back → re-entry) applied to every surface I claim a verdict for; plus the ladder's event/payload/terminal additions (emitter↔subscriber parity, payload-shape parity, terminal-record truth, derived-value provenance, truncation meaning, error identity) and its runtime/deployment-truth additions (source identity, mount completeness, artifact persistence, authz boundary, provider failure identity, latest-state ordering, partial-tool completeness).

### 0.2 Claim classes used in this file

| Tag | Meaning here |
|---|---|
| **MEASURED** | I ran the probe and quote its output (HTTP JSON, DOM counts, Playwright actionability log, DB-backed route response). |
| **VERIFIED** | Exact `path:line` read in this pass from the file at the recorded hash (§14.2); a source fact, not a runtime fact. |
| **READ** | Read from `OPERATOR-DECISIONS.md`, `05`, `08`, `09`, `10`, `15`, a FigCraft board, or an existing repo artifact; not re-measured. |
| **INFERRED** | Follows from ≥2 verified facts but was not directly observed. |
| **UNVERIFIED** | Not observed; the named experiment would settle it. |

### 0.3 Inputs read first (required set)

`OPERATOR-DECISIONS.md` (107 lines, full), `09-synthesis-and-goal.md` (268 lines, full), `15-ui-ux-goal.md` (838 lines, full), `05-ui-ux-patterns.md` (§0–§5, headings + §3.10–§4.5 full), `08-code-ui.md` (§0, §1, §2.1–§2.3, §6 full), `10-gap-meta.md` (heading map + §2–§3 framing), current `web/src` (24 files read in full or by region, §14.2), Rust contract/core/provider seams (`http.rs`, `execution.rs`, `snapshot.rs`, `graph.rs`, `caps.rs`, `tool_loop.rs`, `budget.rs`, `event.rs`), FigCraft boards **05, 07, 08, 11, 12** read as images (`docs/ui/figcraft/ASSETS/*.png`), and the live app at 8099 (GET-only + headless browser DOM reads).

Wave-3 sibling lanes are writing to the same working tree during this seat (see §1.2). Their reports (`19`–`27`) existed as files at 22:05 but were **not** read for content by this seat and are not relied on here.

### 0.4 The four `/deep-investigate` questions this pass

| # | Symptom | Hypotheses (ranked) | Discriminating probe | Outcome |
|---|---|---|---|---|
| I1 | "Does the live app show a truthful run?" | H1 graph is global across runs; H2 UI fabricates nodes; H3 snapshot is empty | `GET /api/history/runs/{id}/state?revision=0` and `=7` for the newest run; compare node ids with the 5 run ids | **H1 confirmed** — 35 nodes, 9 distinct task-id time-buckets, identical node set at rev 0 and rev 7 (only `graph.revision` 82→89). H2 partially true as a consequence (labels fabricated because the snapshot carries no kind) |
| I2 | "Is the History rail's LIVE control usable?" | H1 the button does not exist; H2 it is disabled; H3 it is occluded by the bottom bar | Playwright `click()` on `aside button` last child, then `elementFromPoint` at its centre; then click the footer LIVE | **H3 confirmed** — Playwright: "`<footer>…</footer> from <div class="absolute inset-x-0 bottom-0 z-30"> subtree intercepts pointer events`"; footer LIVE works |
| I3 | "Is the deployed app the code I am reading?" | H1 stale `web/dist`; H2 stale binary; H3 current | served bundle name vs `web/dist` bundle name; `Host`-guard probe; `/health` version/schema | **H3 (current)** — served `index-DzPltEYs.js` equals the current `web/dist` name and contains `args_preview`; server rejects `Host: evil.example` with 421 `E_ORIGIN_REJECTED` (the middleware added to the tree at 21:52), so the 22:00 binary post-dates the tree edits |
| I4 | "Are the seven budget meters renderable today?" | H1 route exists; H2 route absent but caps exposed; H3 nothing exposed | `GET /api/runs/{id}/budget`, `GET /api/approvals`, `GET /diagnose` | **H2 + route absent** — `/api/runs/{id}/budget` → 404, `/api/approvals` → 404; `/diagnose` exposes `run_cost_cap_usd:0.50`, `token_budget_combined:400000`, `call_cap:40`, `max_attempts:3` |

**Probe corrections (recorded because they changed a conclusion):**

1. `GET /api/history/runs/{id}/state` **without** `?revision=` returns `400 E_SCHEMA_INVALID`. That is *not* a defect: `history_state` requires the query parameter (`http.rs:673-675`), and the web client always sends it (`store.ts:437-439`). My first probe omitted it. Verdict changed from "replay route broken" to "replay route works; payload is wrong" (D1).
2. `git show HEAD:…/http.rs | Select-String 'host_guard' | Select-Object -First 8` printed only `.route(...)` lines and I briefly concluded the guard was in HEAD; the `-First 8` truncated the match list. Re-run without truncation: **HEAD contains zero** `host_guard`/`host_allowed`/`loopback_host` lines; the guard is a concurrent uncommitted change (§1.2).
3. The server **restarted mid-audit** (process 9752, started 21:44:12, uptime 542 s at 21:53; process 16616, started 22:00:32, uptime 356 s at 22:06). Browser probes 1–2 straddle the restart; probes 3–4 and the `Host` probe run against the new process. DB rows, bundle name, schema version, and graph node count are identical across both instances (`5` runs, `index-DzPltEYs.js`, schema `10`, `35` nodes), so the measurements are comparable — but anything that a sibling lane rebuilt after 22:00 is **not** covered by the 21:5x probes.

---

## 1. Baseline and identity (trace hop 0)

### 1.1 Repository

```
$ git rev-parse --short HEAD
bd575ba
$ git log -1 --format='%H %ci %s'
bd575ba7adf3a5f2a9bdbc69f9bbf7084fe6fe65 2026-11-07 19:28:54 -0600 feat: real SuperAgent/Subagent personas and persona prompt execution
```

### 1.2 Concurrent working-tree edits (not authored by this seat)

At 21:52:07 `git status --porcelain` listed 2 modified `web/src` files and 4 modified backend files. At 22:06 the same query listed **`http.rs` and `hiverouter-provider/src/client.rs` as newly modified**, and five sibling wave-3 reports (`19`, `20`, `21`, `23`, `26`, `27`) appeared under `docs/research/hmars/` at 22:05. File mtimes:

| Path | mtime | Meaning for this audit |
|---|---|---|
| `web/src/panels/ApprovalOverlay.tsx` | 21:39:44 | `args_preview` render (uncommitted, **in the live bundle**) |
| `web/src/state/store.ts` | 21:39:39 | `argsPreview` mapped from `APPROVAL_REQUEST` (uncommitted) |
| `backend/…/execution.rs` | 21:31:35 | uncommitted; runtime effect UNVERIFIED |
| `backend/…/tool_loop.rs` | 21:34:26 | uncommitted; runtime effect UNVERIFIED |
| `backend/…/http.rs` | 21:52:53 | `host_guard` middleware added by a sibling lane; **live 22:00 binary contains it** |
| `backend/…/provider/src/client.rs` | (new in 22:06 status) | uncommitted; not exercised by this audit |
| `web/dist/index.html`, `web/dist/assets/index-DzPltEYs.js` | 21:56:24 | built after the ApprovalOverlay patch; served by the live app |

**Consequence for every anchor in this file:** line numbers are as read between 21:52 and 22:12 against the hashes in §14.2. Re-hash before implementing. This seat wrote no source file.

### 1.3 Live server identity (MEASURED)

```
GET /health  (22:06:31)
{"status":"ok","version":"1.2.0-dev","schema":"hiverouter.v1","uptime_secs":356,
 "checks":[{"name":"rustc","required":true,"ok":true,"detail":"rustc 1.99.1 …"},
           {"name":"node","required":false,"ok":true,"detail":"v26.7.0 (>= 22.12.0)"},
           {"name":"deepseek_api_key","required":false,"ok":true,"detail":"DEEPSEEK_API_KEY is present (value never logged)"}],
 "database":{"path":"C:\\Users\\Riley\\AppData\\Local\\HiveMind\\hiverouter.db","schema_version":10,"error_code":null},
 "transport":{"last_seq":0,"graph_nodes":35,"allowed_origins":2},
 "prompt_assets":{"loaded":true,"error_code":null}}
```

- **Process:** `hiverouter.exe` (PID 16616) from `C:\Users\Riley\Desktop\HiveMind-Portable\hiverouter.exe`, file mtime 22:00:18, 15,127,040 bytes; started 22:00:32. Earlier instance: PID 9752, started 21:44:12.
- **DB schema 10** → memory-bank migration `v11` has **not** run; there is no bank tree, no `settings` table, no `provider_calls` table, no `watchdogs` table (§7 contract list).
- **`graph_nodes: 35`** with **`last_seq: 0`** → the in-memory graph holds every task of every run in this data dir; the boot recovery published no catch-up frames for this instance.
- **`allowed_origins: 2`**; `Host` guard present at runtime (421 for `Host: evil.example`, §0.4 I3).

### 1.4 Workspace data (MEASURED)

```
GET /api/history/runs  → 5 runs
01a1b931-941a-7171-a209-1c05777354f8  completed  2026-11-08T03:26:36Z
01a1b931-2a4f-75c2-9596-7c447f78b986  completed  2026-11-08T03:26:09Z
01a1b930-9ca0-70b9-b4f0-1450f3d1643d  failed  E_BUDGET_EXCEEDED
01a1b92f-f31f-747e-a63f-58a8fbc0998b  failed  E_BUDGET_EXCEEDED
01a1b92e-7350-7119-8c70-f4d0950cec1d  failed  E_BUDGET_EXCEEDED
GET /api/projects   → 1 project: {"id":"01a1b85f-…","name":"01a1b85f-71f4-7e9f-9a79-22729491030f"}
GET /api/agents/profiles → 7 profiles (chatbot, code-reviewer, content-writer, data-analyst, planner, subagent, superagent)
GET /api/agents/memory   → 7 profiles, every memories[] empty
GET /api/provider/models → 2 models (deepseek-flash, deepseek-v4-pro), ctx 1048576, max_out 393216
GET /api/provider/balance→ USD 3.23 (3.23 topped-up), is_available true
GET /api/usage/runs/01a1b931-… → prompt 6964, completion 1093, total 8057, cache hit 3968, miss 2996, reasoning null
```

This **replaces** the wave-1/2 baseline. `08-code-ui.md:36-40` recorded an empty database (`runs: []`, `projects: []`); `15` carried that forward. The app under audit is **not** in an empty state: it boots with a completed run, a failed-run history, and 35 graph nodes. Everything in §2 below was measured against that non-empty state.

---

## 2. Measured surface inventory (the app as the operator sees it)

All numbers below are MEASURED with headless Chromium (Playwright 1.65 from `e2e/node_modules`) at 1440×900 unless stated. Probe scripts lived in `%TEMP%`; no repo file was written. The probes were read-only: navigation, tab switches, one revision-row click (a GET), and no submit/approve/deny/kill/cancel action.

### 2.1 First paint (Workspace)

```
visible controls: 79  (69 button, 4 role=tab, 2 a[href], 1 textarea, 2 select, 1 input)
disabled controls: 8
canvas nodes: 35   edges: 26   minimap: 1
elements with role="application": 2
aria-live regions: 7
running CSS/WAAPI animations: 72
leaf text nodes < 9.5px: 6   (all six nav labels at 8px)
```

- Every one of the 35 cards carries `data-node-kind="worker"` and an accessible name of the form **`Agent d53c65f4 — WORKER —`**. There is no HyperAgent card, no SuperAgent card, no tool card: `graph.ts:143-165` filters `hyperagent`, and the snapshot generator wrote `kind:'worker'` for all nodes (`store.ts:611-623`).
- **First-8 identity collision is visible in the live UI.** The Agents grid's id column is `task.id.slice(0,8)` (`AgentsPanel.tsx:58`), and across the 35 cards it prints only **7 distinct values** — `01A1B930` ×12, `01A1B931` ×9, `01A1B92E` ×6, and `01A1B86D`/`01A1B88B`/`01A1B88D`/`01A1B893` ×2 each (measured). The UUIDv7 timestamp bucket is shared by every task created in the same moment, which is why `15` and `05` ban the first-8 as identity; the *label* column uses the tail-8 (`store.ts:329-332`) and happens to be unique in this dataset, so the two id columns disagree about which fragment identifies a card.
- **HUD text: `HYPERAGENT · 0 SUPERAGENTS · 35 WORKERS · 45.7% GREEN`.** The `0 SUPERAGENTS` is not an engine fact: the snapshot-projected tasks have no `hyperagent` kind at all, so `deriveHud` counts zero (`metrics.ts:41-47`). `45.7%` = 16 completed / 35 tasks — but the selected run is **completed** and 17 of those 35 nodes belong to **other, failed runs** (§2.3).
- Project chrome reads **`Project: HiveMind AI`** (`LibraryPanel.tsx:48-52` + `store.ts:334-363`, a localStorage-invented id and the constant `projectFallback`), while the real registry's only project has `name` equal to its UUID and is rendered as such by `ProjectsPanel.tsx:71-74` on the Swarm tab. Two project identities are on screen at once.
- Tab order from a cold load (12 stops measured): `Skip to canvas` → `Search agents` → nav `Workspace` → library tab `WORKSPACE` → **8 agent chips** (`Agent d53c65f4 (Completed)` …) → canvas cards. The composer is not among the first 12 tabs.

### 2.2 Per-surface control counts (measured)

| Surface (as navigated) | Visible controls | Disabled | Measured content |
|---|---|---|---|
| Workspace | 79 | 8 | 35 worker cards, 26 edges, HUD, chat, ControlBar, Scrubber |
| Agents | 98 | 0 | `Agents · 35`, hierarchy of 35 rows, `Tool activity · 0`, 7 profiles, memory empty, messages empty |
| Data | 47 | 0 | file tree empty state + preview hint |
| Models | 25 | 0 | usage six counters, balance USD 3.23, two `WIRE-SUPPORTED` models |
| Routing | 24 | 0 | parent-edge table (debug table exposed as a destination) |
| Settings | 27 | 0 | provider/model/thinking/effort read-only, health, reduced-motion override, DevGallery (dev only) |
| Workspace→Swarm tab | 61 | 0 | 5 stat tiles, hero empty state, hierarchy, "Surfaces" card, Projects |
| Workspace→History tab | 78 | 3 | revision rail (8 revisions), canvas in replay, Scrubber |

The library tabs (`WORKSPACE SWARM ROUTING HISTORY`) are painted on **every** destination: the Models-tab body text measured on 22:0x still begins `… Workspace Agents Data Models Routing Settings Project: HiveMind AI WORKSPACE SWARM ROUTING HISTORY ACTIVE AGENTS …` — the global-tab defect of `15` §7.11 is live.

### 2.3 Replay and time travel (MEASURED)

Clicking revision 1 in the History rail produced:

```
banner:  "REPLAY (READ-ONLY) · ROUTING STATE 1"
footer:  "ROUTING STATE 8 OF 8 · LIVE"
labels:  "task 01a1b86d — WORKER —", "task 01a1b86d — WORKER —", "task 01a1b88b — WORKER —", …
```

- The replay labels are **first-8 of foreign tasks** (`task 01a1b86d`), not labels of the run whose revision is being replayed (`01a1b931`). `buildReplayTasks` fabricates them (`graph.ts:196-234`).
- Revision 0 and revision 7 of run `01a1b931` return **the same 35 nodes**; only `graph.revision` changes (82 → 89):

```
$ GET /api/history/runs/01a1b931-…/state?revision=0   → 200, nodes=35, graph.revision=82
$ GET /api/history/runs/01a1b931-…/state?revision=7   → 200, nodes=35, graph.revision=89
$ states: completed=16 cancelled=2 failed=17
$ node-id buckets: 01a1b86d, 01a1b88b, 01a1b88d, 01a1b893, 01a1b92e, 01a1b930, 01a1b931   (7 time buckets, 9 distinct task ids)
```

Root cause (VERIFIED mechanism, §5 D1): `snapshot::capture` stores `graph.snapshot()`, i.e. the whole process graph (`snapshot.rs:258-272`; `graph.rs:216-233`), not a per-run projection; `TaskNode` carries only `{task_id, parent_task_id, state, rev}` (`graph.rs:18-26`), so no card class, role, or label can survive a reload.
- The History rail's own **`LIVE` exit button is pointer-occluded** by the absolutely positioned bottom bar at 1440×900 (Playwright actionability log quoted in §0.4 I2; footer rect `{x:312, y:857, w:1128, h:43}`). The **footer** `LIVE` button works (banner cleared, labels returned to `Agent …`).

### 2.4 Breakpoints (MEASURED)

| Viewport | Library rail | Minimap element | Visible buttons |
|---|---|---|---|
| 1280×800 | 240 px visible | present | 73 |
| 1024×800 | 240 px visible | present | 73 |
| 1000×800 | **0 px (display:none)** | present | 57 |
| 900×800 | **0 px** | present | 57 |

`LibraryPanel.tsx:45` uses `hidden … lg:flex`, so below 1024 px the project identity, agent chips, template insert, state legend and settings readout disappear with **no keyboard or menu alternative**. The minimap element is present at every width (its CSS hiding is not an accessibility path).

### 2.5 Agents surface detail (MEASURED body text)

```
Agents · 35
HIERARCHY  ◆ COMPLETED Agent 18c20b01 WORKERAGENT   └ COMPLETED Agent 03f0b97c WORKERAGENT
           ◆ FAILED Agent 2329b523 WORKERAGENT      └ FAILED Agent 4d3c18f7 WORKERAGENT …
Tool activity · 0 — No tool activity yet — tool calls appear here when the model invokes a tool.
AGENT PROFILES: Chatbot WORKER CHATBOT · 4 TOOLS · REPORT | Code Reviewer … | Content Writer … |
                Data Analyst … | Hive Planner HYPER PLANNER · 0 TOOLS · STRICT |
                HiveMind Subagent WORKER SUBAGENT · 6 TOOLS · STRICT |
                HiveMind SuperAgent SUPER SUPERAGENT · 7 TOOLS · STRICT · LIMITS: 8 CHILDREN · 3 DEPTH
AGENT MEMORY: No durable memories for the boot profiles.
AGENT MESSAGES: No agent-to-agent messages in this run.
```

The hierarchy is a flat `<ul>` with `marginLeft: depth*14px` (`AgentHierarchy.tsx:55-78`), rows sorted by **label** (`:30-31`), no expand/collapse, no `treegrid`, no columns, no root summary, no actions. Board 08 requires a hierarchy with `label · kind · N workers · state` rows and confirmation-first actions; none of that is present.

### 2.6 Models and Settings as shipped (MEASURED body text)

```
Models: PROVIDER deepseek | MODEL deepseek-flash | PROMPT TOKENS 6964 | COMPLETION TOKENS 1093 |
        TOTAL TOKENS 8057 | CACHE HIT / MISS 3968 / 2996 | REASONING TOKENS Not reported |
        PROVIDER ACCOUNT REFRESH BALANCE USD 3.23 | AVAILABLE MODELS
        deepseek-flash WIRE-SUPPORTED DEEPSEEK-V4.1-FLASH · 1048576 CTX · 393216 MAX OUT
        deepseek-v4-pro WIRE-SUPPORTED DEEPSEEK-V4-PRO · 1048576 CTX · 393216 MAX OUT
        CONTRACT REFERENCE (FACT-REGISTER): DEEPSEEK-FLASH; DEEPSEEK-V4-PRO IS OPTIONAL.
Settings: PROVIDER deepseek | MODEL deepseek-flash | THINKING MODE enabled | REASONING EFFORT high |
        PROVIDER KEY STATUS Key configured | SERVER HEALTH ok | RUSTC:OK NODE:OK DEEPSEEK_API_KEY:OK |
        Reduced motion override
```

No cost figure anywhere; no spend cap; no `as of` time on balance; no cache ratio; `WIRE-SUPPORTED` is a constant because `WIRE_MODELS` lists exactly the two models the route returns (`ModelsPanel.tsx:31`, rendered `:241-243`); Settings has no editable field at all — not autonomy, not approvals, not budget mode, not A2A, not memory scopes, not density, not tutorial.

### 2.7 Reduced motion (MEASURED)

| Context | `getAnimations().length` at 3–3.5 s | `data-reduced-motion` | Canvas nodes |
|---|---|---|---|
| `prefers-reduced-motion: no-preference` | **72** | `false` | 35 |
| `prefers-reduced-motion: reduce` | **0** | `true` | 35 |

Reduced motion is correctly exhaustive (`index.css:101-119`, `motion.css:399-442`). The **default** motion budget fails the design's own gate (`05` §3.11: ≤3 concurrent, one infinite loop): 72 concurrent animations for 35 static cards, from three ambient bees, `hive-scan`, the honeycomb field, and per-card pulses/sheens (`Canvas.tsx:295-307`, `index.css:169-331`, `motion.css:42-348`).

---

## 3. Operator decisions vs the running app

Rows are the operator's requirements from `OPERATOR-DECISIONS.md`; "now" is MEASURED/VERIFIED state. Verdict keys: **MET** / **PARTIAL** / **MISSING** / **LIES**.

| # | Operator decision (source line) | Requirement | Now | Verdict |
|---|---|---|---|---|
| 1 | Every agent, incl. tiny Watchdog/Remote, must have a UI card (OP:31) | Total card registry | 5 renderers, all but one reachable only as `worker`; no Subagent/Remote/Watchdog/Hive/Branch/Cluster card; hyperagent card dead (`nodes.tsx:269-275`, `graph.ts:151`) | **MISSING** |
| 2 | Remote/Watchdog = simple switches/daemons with reports (OP:32-33) | Daemon entity + card + cadence/last report | No table, route, event, card, or scheduler (`grep` over `backend/`: no watchdog agent code) | **MISSING** |
| 3 | Agent cards show role, task, state, cost, tools, branch by default (OP:83-84) | Six facts at standard density | Cards show fallback label + `WORKER` + state + permanent `—`; no role, task text, cost, tools, branch | **MISSING** |
| 4 | Autonomy settings/modes 1–4, default 1 (OP:19-20) | Mode selector + policy | No mode concept in UI, contract, or DB | **MISSING** |
| 5 | Approval policy options in settings; default side effects only (OP:20-21) | Policy editor | No settings route; policy is env/config; `spawn_agent` consumes no approval (`execution.rs:1684-1691`) | **MISSING** |
| 6 | Agent capabilities editable in settings, audited (OP:22) | Capability editor | Profile import/delete only (`AgentProfilesPanel`); capabilities read-only from server | **PARTIAL** |
| 7 | No hard default budget; caching is the goal; soft guard + emergency stop allowed (OP:23-25) | Visible soft guard | Hard `$0.50/400k/40-call` caps enforced server-side (`budget.rs:25-52`) with **zero** UI meters; no soft guard, no stop button for spend | **LIES** (caps exist and are invisible) |
| 8 | Retention/visibility/blindness are human-only and auditable (OP:26-27) | Audited human-only controls | Nothing in UI/DB (`schema 10`) | **MISSING** |
| 9 | Memory scopes global/run/hive/branch/role as a Venn cluster (OP:63-65) | Bank tree + inspector | Role-only list, 8-row recall, no scopes, no provenance/trust columns (`AgentMemoryPanel.tsx:86-123`; live payload has only `key`/`value`) | **MISSING** |
| 10 | HyperAgent may read all banks only if leakage prevented (OP:66-68) | Policy + read-set display | No bank model, no read-set, no policy | **MISSING** |
| 11 | Thought storage summaries default (OP:71-72) | Settings + display | `reasoning_tokens` route exists but UI shows "Not reported" (live); no storage policy | **MISSING** |
| 12 | A2A inspector by default; drawer/canvas/chat as settings (OP:79-80) | A2A inspector | Mailbox is a read-only list with no status/count/roster/reply (`AgentMessagesPanel.tsx:86-108`) | **MISSING** |
| 13 | Main view: hierarchy + live execution + memory/A2A edges; projections as settings (OP:81-82) | Hive/Tree projections + edges | One canvas; no treegrid; no memory/A2A edges; no projection setting | **MISSING** |
| 14 | All tiers get cards; metric density is a setting (OP:83-84) | Density presets | No density state; one card size per kind | **MISSING** |
| 15 | Extend FigCraft, don't replace (OP:85-86) | Board parity | Canvas/card/chat geometry match boards 05/07; hierarchy, run detail, approvals, HUD budget violate boards 08/09/12 | **PARTIAL** |
| 16 | DeepSeek only; token caching measured, never claimed (OP:90-98) | Cache metrics per run/agent | Six counters on Models only; no ratio, no per-agent attribution, no cache goal checklist | **PARTIAL** |
| 17 | Scheduler/fair admission; retry then branch failure (OP:102-103) | Queue visibility | No admission queue, no queue UI, single active run | **MISSING** |
| 18 | Persist everything: runs, events, messages, memory, approvals, artifacts (OP:104-105) | Durable ledger | Runs/messages/usage persist; approvals + refusals are session-only (no route, `/api/approvals` → 404) | **PARTIAL** |
| 19 | 50–150 active, staged to 300–600, no overclaim (OP:13-14) | LOD/culling/virtualization | 35 nodes → 72 animations, no culling, layout recomputed on selection (`Canvas.tsx:154-163`), no virtualization anywhere | **MISSING** |
| 20 | HiveWide collision/usage daemon or equivalent (OP:50-51) | Load/collision surface | No daemon, no in-flight chip, no capacity display (`IN_FLIGHT_LIMIT=8` never shown) | **MISSING** |

**Score of the current UI against the operator's own list: 0 MET, 4 PARTIAL, 15 MISSING, 1 LIES.** The wave-2 goal document (`15`) is not stale — it is *unimplemented*. Two places where reality did move since `15` are recorded in §4.2.

---

## 4. Delta against the wave-2 goal (`15`) and first-wave truth (`08`)

### 4.1 What `15` got right and is still true today

Every one of `15`'s §7 removal/repair items was re-checked against the current tree; all still exist except the two noted in §4.2. Load-bearing examples with **current** anchors (they drifted from `15`'s):

| `15` item | Current anchor (VERIFIED) | Live proof |
|---|---|---|
| Swarm stat tiles + hero | `SwarmPanel.tsx:38-51`, `:53-107` | Swarm tab: 61 controls, 5 tiles, hero |
| Chat ACTIONS table | `ChatCore.tsx:274-308` | present (only when toolActivity non-empty) |
| Duplicate tool ledger in Agents | `AgentsPanel.tsx:65-72` | `Tool activity · 0` region |
| Dead HyperAgent card | `nodes.tsx:245-267`; filtered `graph.ts:151` | 0 hyperagent cards live |
| Permanent `—` chrome | `nodes.tsx:183-189`, `:214-219` | every card shows `—` |
| `WorkerAgent` asserted from defaulted kind | `store.ts:1448-1451`, `:325-331` | all 35 cards `— WORKER —` |
| Replay flattening | `graph.ts:196-234` | `task 01a1b86d — WORKER —` in replay |
| Stop disabled during planning/thinking | `ChatCore.tsx:343` | predicate is `streamingRunId===null && runStatus!=='running'` |
| Stale approvals undismissable | `ApprovalOverlay.tsx:312-341` | dismiss renders only inside refusal block |
| `CHAT_PIN` with no control | `store.ts:2105-2108`; no caller in components | grep: no pin control |
| Global library tabs | `LibraryPanel.tsx:15-20`; `App.tsx:50-102` | tabs painted on Models/Data/etc. |
| Routing destination | `NavRail.tsx:9-16`; `App.tsx:96-97` | 6th nav item |
| Local-UUID project chrome | `store.ts:334-363` | `Project: HiveMind AI` |
| HUD `GREEN %` | `HudBadge.tsx:36-51`; `metrics.ts:22,45-46` | `45.7% GREEN` |
| Motion excess | 23 keyframes; `Canvas.tsx:295-307` | 72 concurrent animations |
| 8 px micro-labels | 19 occurrences / 10 files; live nav 8 px; Models 7.5/8 px; History 8/9 px | measured |
| `role="application"` without key model | `Canvas.tsx:281` (+ React Flow viewport) | 2 elements |
| Library tabs/templates without run guard | `LibraryPanel.tsx:133-151`; `TEMPLATE_INSERT` needs active run | click at idle → refusal |

### 4.2 What changed since `15` (measured, so `15` is partially stale here)

1. **Approval `args_preview` now reaches the screen.** `15` §2.2 row 6 and §5.1 said the UI ignores the field. The uncommitted patch adds `argsPreview` to the store record (`store.ts:106`, `:1589`), widens the view (`ApprovalOverlay.tsx:29`), and renders it (`:262-268`); the served bundle contains `args_preview` (MEASURED string search of the live `/assets/index-DzPltEYs.js`). **The gap is no longer "field exists, UI ignores it"; it is quality (raw key, no redaction legend, no from→to) and the undismissable/stale path.** `15` §10 R5 also needs revision: live payloads now carry a preview *when the producer attaches one*; replay after hello still may not.
2. **`/api/projects` exists and is consumed** (`ProjectsPanel.tsx:27`), while the chrome still invents a localStorage project (`store.ts:334-363`). `15` §7.13 ("ProjectsPanel is read-only; fetch `/api/projects`") is half-done: the fetch exists, the *identity* used by Header/Library does not.
3. **A Playwright e2e harness exists** (`e2e/`, `@playwright/test 1.65.0`, installed Chromium), so `15` U8 ("no browser e2e decision") is answered by reality — but `e2e/data/control-matrix.json` is **empty and stale**: 29 surfaces × 0 controls with `_meta.status = "PENDING_APP_NOT_BUILT: web/src is empty and no dev server exists"`, generated 2026-11-06. That artifact is now a false statement about this repo (D26).
4. **The Host allowlist landed** (uncommitted, `http.rs:533`, `:542-593`; live 421 proof) — `09` Q18's "no Host allowlist" is resolved in the working tree; the audit implication is only that the running binary is newer than HEAD.
5. **`MAX_TOOL_TURNS` is 12, not 4** (`tool_loop.rs:14`); `15` §5.4's "turns 4" is stale. Secondary tool-loop meters today: turns 12, result 64 KiB, approval waits 8 (`tool_loop.rs:14,18,21`).
6. **`09` Q17's spawn-approval bypass is still real** (VERIFIED `execution.rs:1684-1691`: "No approval is consumed: spawning is loop-owned"), and the UI has no way to explain a missing spawn approval.

### 4.3 Where the first-wave control matrix (`08` §6.2) is still the best inventory, and what it now misses

`08` §6.2 remains accurate on every control it lists (stop gating, kill/rewire gating, approval a11y, missing model/thinking/spend/profile/topology/A2A/compose/retry, replay flattening, template refusal). It misses, because the runtime state changed:

- 35 live cards and a HUD with a derived percentage now actually render (the defect became visible, not theoretical);
- the History rail `LIVE` control is pointer-occluded;
- the Scrubber and replay banner call a snapshot revision a **"Routing state"** (`strings.ts:132-138`, `:429`);
- replay exit via the footer works, so "replay trap" is limited to the rail;
- the refusal toast stack is capped at 4 with no ledger (`store.ts:784-795`).

---

## 5. Defect register: dead, lying, and missing surfaces

Every row was reached by a trace, not by reading `15`'s prose. Severity: **P0** = truth/safety, **P1** = required HMARS representation, **P2** = craft/perf. "Class" uses the `trace` verdict vocabulary plus **LYING** for a rendered claim with no producer.

### 5.0 Index

| ID | Sev | Class | Surface | One-line |
|---|---|---|---|---|
| D1 | P0 | LYING | Replay/history | Revision replay shows the whole process graph, not the run's graph |
| D2 | P0 | LYING | First paint | Snapshot tasks are fused into the live task map as `worker` "Agent …" cards |
| D3 | P0 | MISSING | All-agent cards | No agent class on the wire; Subagent/Remote/Watchdog/Hive/Branch have no card |
| D4 | P0 | LYING | HUD | `45.7% GREEN` and `0 SUPERAGENTS` are derived from a broken census |
| D5 | P0 | WIRED-BUT-DEAD | Approvals | Dismissable states have no Dismiss control; no inbox; no focus/Escape; 1 s live-region ticker |
| D6 | P0 | MISSING | Refusals | Refusals are toast-only, capped at 4, no ledger, no run-level failure record |
| D7 | P0 | MISSING | Destructive actions | Kill/rewire/clear execute with no confirmation (board 08 says confirmation-first) |
| D8 | P0 | LYING | Project identity | Chrome shows invented UUID + `HiveMind AI`; registry says the name is the UUID |
| D9 | P0 | WIRED-BUT-DEAD | History rail | `LIVE` exit is occluded by the bottom bar; revision copy says "Routing state" |
| D10 | P1 | MISSING | Tree projection | No treegrid, no columns, no expand/collapse, label-sorted rows |
| D11 | P1 | MISSING | LOD/culling/density | No bands, no culling, layout recomputed per selection; 72 animations at 35 nodes |
| D12 | P1 | MISSING | Memory inspector | Role-only list; no scopes/trust/provenance/read-set/retention; unconfirmed clear |
| D13 | P1 | MISSING | A2A inspector | No status/count/cap/grouping/roster/reply; live append missing in the store |
| D14 | P1 | MISSING | Seven meters | No budget route, no cost persistence, no meter UI; caps invisible |
| D15 | P1 | PARTIAL/LYING | Provider & cache | Hardcoded `WIRE_MODELS`, no `as_of`, no cost, no ratio, sub-floor labels |
| D16 | P1 | MISSING | Settings/autonomy/tutorial | Settings is a read-only status page; no route to change anything |
| D17 | P1 | MISSING | Run detail | No destination; board-12 sections split across Agents/Models/chat; no past-run list |
| D18 | P1 | MISSING | Card registry totality | Only 5 kinds; no density; no class-source honesty |
| D19 | P2 | LYING | Motion | 23 keyframes, 3 ambient loops, 72 concurrent animations |
| D20 | P2 | LYING | Type floor | 19 sub-11 px labels (7.5–10 px), live on every surface |
| D21 | P2 | MISSING | Canvas a11y | `role="application"` ×2 with no keyboard model; card names are id soup |
| D22 | P2 | LYING | Card chrome | Permanent `—`/`ctx`/`score` slots that no producer can fill |
| D23 | P2 | MISLEADING | Library rail | Hidden <1024 px with no alternative; template click at idle → refusal |
| D24 | P2 | DUPLICATE | Tool activity | Two tables (chat + Agents) and no run-detail ledger |
| D25 | P2 | MISSING | Density/inspector | No inspector for a selected agent; selection has no detail surface |
| D26 | P1 | STALE | e2e control matrix | `e2e/data/control-matrix.json` claims "web/src is empty", 0 controls |

### 5.1 P0 — truth and safety

#### D1 — Revision replay renders the whole process graph, not the run's graph — **P0 LYING**

**Chain (trace hops 5→7→10):** route `GET /api/history/runs/{id}/state?revision=N` → `HistoryReadModel::state_at` (`history.rs:109-115`) → `SnapshotStore::read` (`snapshot.rs:110-119`) → the stored file.

- **Producer:** `snapshot::capture` builds the envelope from `graph.snapshot()` (`snapshot.rs:258-272` at `:268`), and `GraphAuthority::snapshot` (`graph.rs:220`) serializes the **process-wide** graph. `TaskNode` has four fields only: `task_id`, `parent_task_id`, `state`, `rev` (`graph.rs:18-26`).
- **MEASURED:** run `01a1b931…` (completed) returns 35 nodes whose ids span seven task-id time buckets / nine distinct task ids (`01a1b86d`, `01a1b88b`, `01a1b88d`, `01a1b893`, `01a1b92e`, `01a1b930`, `01a1b931`); state census `completed=16, cancelled=2, failed=17`; revisions 0 and 7 differ only in `graph.revision` (82 vs 89).
- **Render-back:** `buildReplayTasks` invents `kind:'worker'` and `label:'task ' + id.slice(0,8)` (`graph.ts:196-234`), so the operator sees a completed run's replay painted with 17 failed foreign nodes and labels from the wrong run ("`task 01a1b86d`").
- **Removal of a false claim:** `graph.ts:190-195`'s comment ("The snapshot records only id/parent/state/rev; no role or progress is invented") is true about fields and false about *scope*; the scope error is what makes the surface lie.

**Required change:** snapshot capture becomes a **per-run projection** (`graph.nodes_for_run(run_id)` filtered by `tasks_for_run`), snapshot schema bumps to `v3` with `nodes[].{kind, role, label, progress?, agent_class?, depth}`, `HISTORY_SNAPSHOT`/`/state` serve the projection, and `buildReplayTasks` is deleted in favour of `buildAgentView(snapshot)` (§6). Gate: same-run node set only; rev 0 ≠ rev 7 whenever the run changed; a completed run's replay contains 0 `failed` nodes that are not that run's.

#### D2 — Snapshot tasks are fused into the live task map at boot — **P0 LYING**

- `performSnapshotRefresh` reads `/api/history/runs` → picks the newest/active run → reads its latest snapshot → `set({ tasks: projection.tasks })` (`store.ts:1150-1163`, tasks at `:1156-1158`).
- `recordsFromSnapshotGraph` writes `kind:'worker'`, `label: fallbackTaskLabel(null, id)` = `Agent <tail8>` (`store.ts:609-623`, `:329-332`).
- **MEASURED:** first paint has 35 cards, `data-node-kind="worker"` on all 35, names `Agent d53c65f4 — WORKER —`, HUD `0 SUPERAGENTS · 35 WORKERS · 45.7% GREEN` (§2.1). The root planner is absent from the canvas because no node carries `kind:"hyperagent"` (`graph.ts:151` filters it anyway).
- Consequence: the workspace is not a live view. It is a **replayed history graph presented as live**, with live mutations gated only by `run.status` (the run is `completed`, so the ControlBar disables them — by accident of state, not by design).

**Required change:** two task maps, `liveTasks` (from `NODE_*` only) and `historyTasks` (from snapshots), with an explicit `source: 'live' | 'history' | 'empty'` banner ("No live run — showing the last committed run (read-only)"). Canvas/ControlBar/ChatCore read the mode, not a mixed map. Gate: on a fresh boot with no live run, the canvas shows 0 live cards and a `read-only history` banner; controls are disabled by mode.

#### D3 — There is no agent *class* on the wire, so Subagent/Remote/Watchdog/Hive/Branch cannot have cards — **P0 MISSING**

- The contract's node kinds are `["hyperagent","superagent","worker","tool"]` (`event.rs:240`); `NodeEvent` serializes only `task_id`, `state`, `rev?`, `kind?`, `parent_task_id?` (`event.rs:248-271`) — no `label`, `progress`, `role`, `agent_class`, `persona_id`, `depth`.
- Web payload tolerates `label`/`progress`/`error_code` (`contract.ts:237-246`) but the server never sends them (VERIFIED producer above; MEASURED `progress` chrome always `—`).
- `subagent` is only a role string mapped to `AgentTier::WorkerAgent` (`execution.rs:134`, `:151-162`), so the kind is `worker`; there is no `remote`/`watchdog` anywhere in `backend/`.
- The renderer registry has five entries (`nodes.tsx:269-275`) keyed on that five-value union (`agentState.ts:126-135`); `graph.ts:30-36` hard-codes size per kind. **Every card the operator asked for except HyperAgent/SuperAgent/Worker/Tool does not exist.**
- MEASURED consequence: 35/35 cards are worker; "HiveMind Subagent" and "HiveMind SuperAgent" exist only as *profile rows* on the Agents surface (`AgentProfilesPanel`), never as agents in the graph.

**Required change:** additive wire fields `agent_class?: 'subagent'|'remote'|'watchdog'|null`, `role?: string`, `persona_id?: string`, `depth?: number`, `label?: string`, `progress?: number|null` on `NodeEvent` + snapshot v3 + `TaskRecord`; a **total** card registry over `AgentClass` enforced by a `satisfies` exhaustive map (§6.4); honest "class not reported" rendering. Gate: `agentCards.test.ts` proves totality; a synthetic `subagent` frame renders `SubagentCard`, a `watchdog` frame renders `WatchdogCard` with an honest capability gate.

#### D4 — The HUD's `GREEN %` and `SUPERAGENTS` are not engine facts — **P0 LYING**

- Server `RUN_METRICS.green_pct = completed_tasks / all_tasks` (`execution.rs:657-695`, `:669-677`); `superagents` counts only tier `SuperAgent` (`:661-664`), `workers` counts tier `WorkerAgent` (so subagents are counted as workers).
- Client, when no metric frame arrived (the normal boot case: `last_seq 0`, MEASURED), derives counts from tasks and includes `hyperagent` in "superagents" (`metrics.ts:41-47`), then labels a percentage `GREEN` (`strings.ts:37-42`; `HudBadge.tsx:36-51`).
- **MEASURED:** `HYPERAGENT · 0 SUPERAGENTS · 35 WORKERS · 45.7% GREEN` on a completed run, because the snapshot-derived tasks contain no hyperagent at all. The number is arithmetically real and semantically false: the denominator mixes five runs and the numerator is `completed`.
- The literal `98.2% GREEN` originates in the **design board** (`hivemind-controls-and-hud.png`, board 07), not in a metric definition — board 12 says usage "values arrive from RUN_METRICS; none are fabricated".

**Required change:** replace GREEN/delta with (a) an explicit census `N hives · M agents · K running` labelled `derived` when no server frame, (b) the seven-meter chips of §5 D14, (c) if GREEN is kept, rename it `completed` and define the denominator in the tooltip plus `RUN_METRICS` doc. Gate: with one run of 4 completed / 4 total, HUD reads `1 hive · 4 agents · completed 4/4`; with a mixed graph it never shows a percentage that includes other runs (D2 fixes the input).

#### D5 — The approval surface is blind by layout and dead where it matters — **P0 WIRED-BUT-DEAD**

The nonce/TTL state machine is good. The *surface* is not.

| Hop | Fact | Anchor |
|---|---|---|
| Render | One `fixed inset-0` overlay **per pending approval**, each `role="dialog" aria-modal="true"`; with 2 pending, two stacked modals | `ApprovalOverlay.tsx:221-244` |
| Render | Dismiss/Retry exist **only** inside the `resolveState?.refusal` block | `:312-341` |
| State | `dismiss` is enabled for every terminal status (`expired`/`stale`/`unconfirmed`) | `approval.ts:254-267`, `TERMINAL_STATUSES` `:136-140` |
| Defect | An approval that client-expired, or that arrived `unconfirmed` after a replay settle, has `actions.dismiss === true` and **no rendered control** | combination of the two rows above |
| Render | Expiry `<dd aria-live="polite">` re-announces every second while the 1 s timer runs | `:174-185`, `:290-293` |
| A11y | No focus trap, no initial focus, no `Escape` handler, no focus return; no keydown listener in the file | `:165-373` (absence) |
| Copy | `args_preview` is now rendered but labelled with the raw field name, outside `STRINGS`; no redaction legend | `:262-268` |
| Copy | No from→to branch narrative, no nonce single-use copy, no consequence line ("approve grants one consume") | `:253-304` |
| Copy | Nothing links an approval to its task/branch beyond truncated ids | `:275-283` |
| Inbox | No queue, no count chip, no decided ledger; `GET /api/approvals` → **404** (MEASURED) | `HudBadge.tsx:13-53`; route table `http.rs:514-531` |

**Required change:** `ApprovalInbox` destination + `HudBadge` count; a single focused `alertdialog` with deny-first order, focus trap, `Escape` = close-without-decision, focus return, ≥10 s announcement granularity (or announce only at 60/30/10 s), `args_preview` + redaction legend + TTL + nonce + consequence + from→to; recovery controls (Dismiss/Retry) always reachable when the state machine enables them; decided ledger from a new `GET /api/runs/{id}/approvals`. Gate: `approvalInbox.test.ts` asserts (a) dismiss button exists whenever `approvalActionsEnabled().dismiss`, (b) `getAnimations()`-independent live-region throttling, (c) Escape closes without sending a command, (d) exactly one `role="dialog"` for N pending approvals.

#### D6 — A refusal that scrolls away is lost; there is no run-level failure record — **P0 MISSING**

- `pushRefusal` dedupes and truncates to **4** entries (`store.ts:784-795`); `ToastRegion` renders them as dismissible toasts with a single `aria-live="assertive"` region (`ToastRegion.tsx:12-47`).
- MEASURED: 5 runs in the DB, two categories (`completed`, `failed E_BUDGET_EXCEEDED`), yet no screen lists run failures. The only run-error rendering is a chat card for the *current* run (`ChatCore.tsx:211-221`) and a ControlBar status span (`ControlBar.tsx:118-124`).
- `05` §3.12 requires a run-detail error card + ledger row for run-level failures, and "never a refusal only in the console". Board 12 draws the FAILURE / REFUSAL LEDGER panel with `retryable`/`not retryable` chips; nothing equivalent exists.
- `/api/approvals` and any refusals route are absent (MEASURED 404), so a ledger is client-session-only until a route lands — it must be **labelled** as such rather than omitted.

**Required change:** `refusals` slice keeps an append-only session ledger (no 4-cap on the ledger; the toast stack may stay capped) with `{at, code, message, how_to_fix, retryable, runId, taskId, surface}`; `RunDetail` renders failures from `run.errorCode` + the ledger; rows say `session-only` until a read route exists. Gate: 10 refusals in a session are all visible in the ledger after the toasts are dismissed; a failed run shows its code, message, `how_to_fix`, and retryable flag.

#### D7 — Destructive mutations have no confirmation — **P0 MISSING**

- `Kill worker` sends `KILL_WORKER` on click when enabled (`ControlBar.tsx:107-116`); `Re-route` sends `REWIRE_TASK` on click (`:83-95`); dragging a worker card onto a superagent re-parents live (`Canvas.tsx:368-397`).
- `Clear` role memory sends `DELETE /api/agents/memory/{key}` immediately (`AgentMemoryPanel.tsx:75-84`, button `:105-111`).
- Board 08 states the product rule: "re-parent only while not streaming; **kill/rewire are confirmation-first**" (board 08 hierarchy panel). `05` §3.10 adds: "no drag-only action … destructive actions always confirm".
- The canvas drop rewire also has **no streaming guard**: its only precondition is `runActive` + `kind==='worker'` (`Canvas.tsx:380-396`), whereas the ControlBar rewire path requires a non-streaming branch per the board.

**Required change:** a shared `ConfirmDialog` (focus trap, deny-first, named consequence + target identity); kill/rewire/route-change/memory-clear route through it; keyboard alternatives `R`/`K` open the same dialogs (`05` §3.10); canvas drag opens the confirmation instead of mutating. Gate: `destructive.test.ts` — clicking Kill opens a dialog and sends nothing until confirm; Escape returns focus and sends nothing; drag-drop with `state==='streaming'` is refused by name.

#### D8 — Two project identities are rendered at once — **P0 LYING**

- Chrome: `Project: HiveMind AI` from `store.ts:334-363` (localStorage `hivemind.project-id`, name `STRINGS.library.projectFallback`), rendered by `LibraryPanel.tsx:48-52`; Header shows `owner` from the same fake record (`Header.tsx:62-68`).
- Registry: `GET /api/projects` → `[{id: 01a1b85f…, name: "01a1b85f-71f4-7e9f-9a79-22729491030f"}]` (MEASURED); `ProjectsPanel` renders `project.name || project.id` with a `CURRENT` chip against the **local** id (`ProjectsPanel.tsx:60-77`), which can never match — the local id is random per browser.
- Net: one screen shows a project called `HiveMind AI`, another shows the same project as a UUID, and neither is marked as `current` because the ids differ.

**Required change:** one `project` source from `GET /api/projects` (server name, falling back to a **display** short id labelled `unnamed project`); delete the localStorage identity; ProjectsPanel takes the server id. Gate: `projectIdentity.test.ts` — chrome and registry render the same id/name; no localStorage read for identity; `CURRENT` chip appears on the registry row when ids match.

#### D9 — The History rail's `LIVE` exit is pointer-occluded, and the scrubber mislabels revisions — **P0 WIRED-BUT-DEAD**

- `App.tsx:131-135` mounts the Scrubber in an `absolute inset-x-0 bottom-0 z-30` wrapper on the History tab; `HistoryPanel` renders its `LIVE` button at the bottom of a full-height `<aside>` (`HistoryPanel.tsx:57-64`).
- **MEASURED (Playwright):** clicking `aside … button` last child fails with `<footer …> from <div class="absolute inset-x-0 bottom-0 z-30"> subtree intercepts pointer events`; footer rect `{x:312,y:857,w:1128,h:43}` at 1440×900; the footer's own `LIVE` button works.
- Copy: the scrubber and replay banner print **"Routing state N of M"** (`strings.ts:132-138`, `:429`; MEASURED banner `REPLAY (READ-ONLY) · ROUTING STATE 1`). A committed snapshot revision is not a routing state; there is no "routing" object in the contract.
- The scrubber input itself is a range with `aria-valuetext` (`Scrubber.tsx:46-60`), which is good; `onPointerUp`/`onKeyUp` commit (`:58-59`), so keyboard users commit on key-up — acceptable but untested for arrow-key granularity.

**Required change:** give the main region a bottom padding equal to the overlay height (or move the rail's exit control into the Scrubber only); rename the copy to `revision N of M` + wall-clock time per board 07/`05` P2-1; keep `LIVE` as the single exit affordance, reachable by keyboard and pointer. Gate: `historyRail.test.ts` — `elementFromPoint` at the exit button centre resolves to the button; the label text matches `/revision/i`; Escape/Enter works.

### 5.2 P1 — required HMARS representation

#### D10 — No tree projection (the accessibility and density peer) — **P1 MISSING**

`AgentHierarchy.tsx:21-53` walks parent links into a flat array ordered by `label.localeCompare` (`:30-31`), then renders `<li><button style={{marginLeft: depth*14}}>` (`:55-78`). No `treegrid`, no `aria-expanded`, no roving tabindex, no columns (state/workers/progress/cost/tools/branch), no root summary, no expand/collapse, no virtualization. Used by both Swarm (`SwarmPanel.tsx:110-113`) and Agents (`AgentsPanel.tsx:37-39`), so both surfaces inherit the defect. Board 08 shows rows carrying `label · kind · N workers · state` and a root row `HyperAgent · root · 4 superagents · healthy`; `05` §3.3 specifies an APG treegrid.

**Required change:** new `web/src/panels/AgentTree.tsx` (§7 W3) driven by `AgentViewNode[]`, virtualized > 200 rows, root summary row, columns per §6.5, keyboard map per APG (Up/Down/Right/Left/Home/End/Enter/Space), `aria-rowcount`/`aria-rowindex`. Gate: `treegrid.test.ts`.

#### D11 — No LOD, no culling, no density; layout is recomputed on selection — **P1 MISSING**

- `Canvas.tsx:154-163` memoizes `buildNodes(visibleTasks, selectedTaskId, …)`; `buildNodes` calls `layoutTasks` → `dagre.layout` (`graph.ts:143-165` → `:42-141`), and the memo **depends on `selectedTaskId`**, so every selection re-runs the full dagre layout for all visible nodes.
- No `onlyRenderVisibleElements` on `<ReactFlow>` (`Canvas.tsx:351-405`), no band logic, no cluster layer, no animated-edge cap (edges are `animated:false`, good, but 72 card animations run).
- MEASURED: 35 nodes → **72** WAAPI animations with no run active; `03` §9.4 gates are ≥50 fps at 600 nodes, ≤120 mounted nodes, ≤50 animated edges.
- Filters are component-local state (`Canvas.tsx:110-111`), not store state, so Swarm/Agents cannot share them (`15` U6).

**Required change:** `web/src/flow/lod.ts` (bands + ±0.04 hysteresis + culling helper), `view.filters`/`view.band`/`view.density` in the store, layout memo keyed by `(runId, sorted id:class:parentId)` only, `onlyRenderVisibleElements`, batched B0 cluster layer, animated-edge budget. Gate: `lod.test.ts` + a mounted-node count assertion at 600 synthetic nodes.

#### D12 — Memory inspector is a role list with an unconfirmed destructive action — **P1 MISSING**

`AgentMemoryPanel.tsx:20-84` fetches `/api/agents/memory` directly, keeps only profiles with non-empty `memories` (`:99-101`), renders `key: value` (`:113-119`), and clears with `DELETE` + no confirmation (`:75-84`). There is no scope ladder (project/run/hive/branch/role), no trust, no `as_of_rev`, no read-set ("own + ancestors"), no truncation counter, no promotion/purge, no provenance (`run_id`/`task_id` are not even returned by the route — live payload has `id/display_name/memories[{key,value}]`), no honest "bank tree not available on this build" banner. DB schema is 10 (MEASURED), so the bank tree does not exist yet; Phase A must say so.

**Required change:** `MemoryInspector.tsx` Phase A: role scope only + explicit banner + provenance columns when the route carries them (`GET /api/agents/memory?provenance=1`); confirmation + audit note on clear; empty roles still listed with `0 entries` so the operator can see the seven banks. Phase B after migration: five scopes from `04`. Gate: `memoryInspector.test.ts`.

#### D13 — A2A mailbox has no status, count, cap, grouping, roster, or reply — **P1 MISSING**

`AgentMessagesPanel.tsx:21-108` fetches per run, renders `kind · from8 → to8 · text`, refetches when `agentMessageVersion` changes. The store only bumps that integer on `AGENT_MESSAGE` (`store.ts:1495-1497`), so there is no buffered row and no live append in the panel until the refetch resolves. No `status`/`in_reply_to`/`answered_at` exist in the payload (`15` U10 READ; mine: live payload has `id/from_task_id/to_task_id/kind/text/created_at`), no `messages N / 32` header (`execution.rs:70` `MAX_AGENT_MESSAGES_PER_RUN=32`), no branch grouping, no roster, no operator reply (no route/command), no approval link for cross-branch messages even though `send_to_sibling` across branches *does* suspend for approval (`execution.rs:1695-1718`).

**Required change:** `MailboxPanel` + `roster` slice; header `messages N / 32 · status: not reported`; rows grouped by `hive/branch` pair with verb (Escalate/Notify) and approval link; roster list (id, label, class, parent, depth, state) from `GET /api/agents/runs/{id}/roster`; reply only after `AGENT_MESSAGE_REPLY` exists. Gate: `mailbox.test.ts`.

#### D14 — The seven meters do not exist anywhere in the UI, and three of seven have no producer route — **P1 MISSING**

Verified denominators (source) vs producers (routes):

| # | Meter | Denominator (VERIFIED) | Numerator today | Renderable? |
|---|---|---|---|---|
| 1 | Agents | 64 (`caps.rs:13`, `budget.rs:40`) | `Object.keys(tasks).length` | yes (derived) |
| 2 | Depth | 3 (`caps.rs:9`, `budget.rs:34`) | max parent-chain depth | yes (derived) |
| 3 | Fan-out | 8 (`caps.rs:11`, `budget.rs:37`) | max children/parent | yes (derived) |
| 4 | Provider calls | 40 (`budget.rs:31`, `caps.rs:15`, `tool_loop.rs:16`) | in-memory only; no route | **no** |
| 5 | Tokens | 400 000 (`budget.rs:28`) | `GET /api/usage/runs/{id}` (six counters, MEASURED) | yes |
| 6 | Cost | $0.50 (`budget.rs:25`) | not persisted, not exposed | **no** |
| 7 | Messages | 32 (`execution.rs:70`) | list route only, no count | partial |

- `/api/runs/{id}/budget` → **404** (MEASURED); `/api/approvals` → 404; `/diagnose` does expose `run_cost_cap_usd:0.50`, `token_budget_combined:400000`, `call_cap:40`, `max_attempts:3` (MEASURED) — so **caps are known to the server and invisible to the operator**.
- `grep` over `web/src` for `budget` returns only refusal-code strings and an approval-settle constant: no meter component, no cap display, no 80 % warning, no cap-hit linking.
- Secondary tool-loop meters (`tool_loop.rs:14,18,21`): turns **12**, result 64 KiB, approval waits 8 — also absent.

**Required change:** `GET /api/runs/{id}/budget` (seven meters + `asOf` + provenance), `runs.calls_attempted/cost_usd/cost_status` migration, `BudgetStrip.tsx` in Run detail + HUD chips for agents/calls; `—` for unknown, never 0; cap-hit links the named refusal; `IN_FLIGHT_LIMIT=8` (`budget.rs:43`) shown as a status chip, never a run meter. Gate: `budgetMeters.test.ts` + `budget_read.rs`.

#### D15 — Provider & cache panel is a proxy view with a constant badge and no cost — **P1 PARTIAL/LYING**

- `WIRE_MODELS = new Set(['deepseek-flash','deepseek-v4-pro'])` (`ModelsPanel.tsx:31`), used to print `WIRE-SUPPORTED`/`NOT ACCEPTED` (`:241-243`). The route returns exactly those two models (MEASURED), so the badge is a constant: it can never say anything else, and it is not derived from server capability.
- No `as of` timestamp on the account read (`:196-256`), no cost, no cache ratio (raw hit/miss only, `:182-185`), no per-agent/call attribution, no cache-goal checklist, no "settled cost unavailable" state; labels at `7.5px`/`8px` (`:241`, `:245`) are below the design floor.
- Balance is fetched on mount and on `Refresh` (`:68-103`) — fine and bounded.

**Required change:** derive the supported set from a server field (`GET /api/provider/models` → `supported_by_build: bool` or a `capabilities` block); add `as_of` to balance/models/usage; add cost only when `CostStatus::Complete`; cache ratio only when hit+miss known; per-call table after the `provider_calls` migration; type floor respected. Gate: `providerCache.test.ts`.

#### D16 — Settings is a read-only status page, not settings — **P1 MISSING**

`SettingsPanel.tsx:32-118` renders: provider/model/thinking/effort (from `SETTINGS_STATUS`, a WS command — `contract.ts:107`, `store.ts:2202-2210`), provider-key status, `/health` summary, a reduced-motion override checkbox, and `DevGallery`. There is **no** autonomy mode, approval policy, budget mode, A2A policy, memory scope/retention, project/retention control, density, projection, tutorial, or model selection; `GET/PUT /api/settings` → 404 (MEASURED). The values shown are boot-configured and the panel does not say so (it says "Key configured", not "applies to the next run after restart").

**Required change:** §7 W7 (settings schema + route + sections), with human-only rows disabled/confirmed/audited and env-only values labelled `boot-configured — restart required`. Gate: `settingsModes.test.ts`.

#### D17 — There is no run detail destination, and past runs are invisible — **P1 MISSING**

- IA: 6 nav destinations, none of them a run (`NavRail.tsx:9-16`); tabs are `workspace|swarm|routing|history` (`store.ts:63`), no `run`.
- `/api/history/runs` returns **5** runs (MEASURED); `decodeHistoryRuns` is consumed only by `chooseHistoryRun` (`store.ts:1040-1044`, `:643-655`) and never stored, so the operator cannot see or switch runs. `HistoryState` has revisions but no run list (`store.ts:147-159`).
- Board 12 wants six stacked panels (summary, stages, tool ledger, model/usage, failure/refusal ledger, settings). Today: tool activity lives in **Agents** (`AgentsPanel.tsx:65-72`), model usage in **Models**, failures nowhere, files in **Data**, settings in **Settings** — five destinations for one run's evidence.

**Required change:** `RunDetailPanel.tsx` + `nav:'run'` + `runs` slice + `selectedRunId`; sections in board-12 order; the tool ledger moves here (§8 removals); failure ledger from D6. Gate: `runDetail.test.ts` (all six sections render with a fixture; empty/error/refusal states).

#### D18 — Card registry is not total, density does not exist, class source is not honest — **P1 MISSING**

`NODE_RENDERERS` has five entries and `nodeSize` five sizes (`nodes.tsx:269-275`; `graph.ts:30-36`). No density setting; no `classSource`; no "class not reported" affordance; hyperagent card unreachable (`graph.ts:151` filters it while `Canvas.tsx:167-189` adds an invisible anchor); error card unreachable (contract kind vocabulary excludes `error`, `event.rs:240`; store defaults to worker `store.ts:1448-1451`). The operator's "all tiers get cards, metric density is a setting" is entirely unimplemented.

**Required change:** §6 card registry + `view.density`; `HiveChatCard` remains the hyperagent representation (board 05), the canvas hyperagent card is deleted, Error becomes a **state render**, not a kind. Gate: `agentCards.test.ts`.

#### D19 — Motion is 2.4× the design budget before any run starts — **P2 LYING**

MEASURED 72 concurrent animations at 35 static cards; `prefers-reduced-motion` correctly yields 0 (`§2.7`). `05` §3.11 and board 11 fix: ambient 18 s (one), sheen 6 s, pulse 1.6 s, working-bee 1.2 s, hover 120 ms, focus 160 ms, card 120–160 ms, no more than one infinite loop. The tree declares 23 keyframes across `motion.css` (12) and `index.css` (11), including three ambient bees (`Canvas.tsx:295-307`), `hive-scan` (`:296`), and the honeycomb field (`HiveField`). Nothing measures the budget in tests.

**Required change:** one ambient loop maximum (recommend the 18 s honeycomb travel), delete the free-flying bees and the scan sweep, drive state/pulse only from events, add the gate test (`getAnimations()` ≤3 normal, 0 reduced). Gate: `designParity.test.ts` update.

#### D20 — Type floor is violated 19 times (live: nav labels at 8 px) — **P2 LYING**

`tokens.ts:91-100` declares the smallest printed step `mono-meta 11px`; board 05 prints card meta at 9.5 px. Measured live: nav rail labels 8 px; Models model-meta 7.5/8 px; History rail 8/9 px; Swarm tiles 8.5 px; Projects chips 8/10 px; Library chips 8 px / tabs 9 px. Total `text-[7.5..10px]` occurrences: 19 across 10 files.

**Required change:** a `TYPE_FLOOR_PX = 11` constant + a test that scans `web/src/**/*.tsx` for `text-[<11px]` and fails; replace with `type-mono-meta` (11) or a documented 9.5 only where board 05 prints 9.5 (card meta). Gate: `tokens.test.ts` addition.

#### D21 — `role="application"` twice with no keyboard model; card names are id soup — **P2 MISSING**

`Canvas.tsx:278-284` sets `role="application"` + `aria-label` + `tabIndex={-1}` and React Flow's viewport adds its own (`MEASURED: 2`). No composite-widget keyboard contract is implemented (cards are buttons inside; arrow navigation absent). `graph.ts:160` produces `ariaLabel = `${label} (${kind}, ${state})``, which for snapshot-derived cards is "Agent d53c65f4 (worker, completed)" — a random 8-hex id, not an identity. Skip link targets `#hive-canvas` (`App.tsx:106-108`) but tab order still walks nav → chips → cards first.

**Required change:** canvas becomes `role="region"` + `aria-roledescription="hive canvas"` with the Tree as the accessible peer, or implement the full APG grid and test it (`05` §2.11). Card names use `class · role · depth · short-id` (§6.2). Gate: `a11y.test.ts` snapshot of roles/names.

#### D22 — Cards ship permanent empty chrome — **P2 LYING**

Worker card prints `—` for progress and `formatCtx(null)` (`nodes.tsx:183-189`); Tool card prints "no metrics"/`score`/attempt with `formatCount(null)` (`:214-219`, `:236-239`); SuperAgent prints a static role caption "decomposes intent, owns the sub-hive" as if it were the agent's role (`:160-162`, `strings.ts:281`). None of these has a producer.

**Required change:** delete the slots; render only fields present in `AgentViewNode` (null means *absent*, §6.3); keep one honest `—` affordance per card only where the field is contract-defined and currently unknown (progress).

#### D23 — Library rail disappears below 1024 px; template click at idle refuses — **P2 MISLEADING**

`LibraryPanel.tsx:45` `hidden … lg:flex` → 0 px at 1000/900 (MEASURED); the rail holds project identity, active-agent chips, templates, state legend, settings readout. No menu/drawer alternative exists (Header has search only). Templates dispatch `TEMPLATE_INSERT` (`:133-151`) although the command requires an active run (`ws.rs:797-831` READ via `15`), so at idle it produces a refusal toast.

**Required change:** promote the rail to a drawer below `lg` with a Header toggle (stable selector + `aria-expanded`), disable templates with an inline reason when `run === null`, keep state legend reachable in the drawer. Gate: `responsive.test.ts` (1000 px: drawer toggle exists, opens, contains project/chips/templates).

#### D24 — Tool activity has two homes and no ledger — **P2 DUPLICATE**

`ChatCore.tsx:274-308` renders the last 4 tool entries as `ACTIONS`; `AgentsPanel.tsx:65-72` renders the full `ToolActivityPanel` (`ToolActivityPanel.tsx:131-184`, 6 outcomes, hashes, turn). There is no run-detail table, no branch grouping, no result bytes/sha, no filters, no 200-cap footer (board 12's TOOL ACTIVITY panel).

**Required change:** run-detail ledger is the only table (§7 W4); chat keeps `2 tools running · 1 awaiting approval → Run detail`; Agents keeps a link + count. Gate: `toolActivityHomes.test.ts`.

#### D25 — Selection has no detail surface — **P2 MISSING**

`selectedTaskId` is consumed by Canvas, AgentHierarchy, AgentsPanel, ControlBar, SwarmPanel, graph (`grep`: 6 files + store) — and by **no inspector**. Clicking a card highlights it and populates the Target select; nothing shows the agent's identity, parent, children, bank, A2A, counters, or raw state history. There is no `Inspector|Detail|Summary` component in `web/src` (glob: none).

**Required change:** `AgentInspector.tsx` (right rail or overlay, board-05-adjacent) with `identity / class+classSource / role / parent+children / branch / state family + last event / metrics / bank hint / A2A / actions`. Gate: `agentInspector.test.ts`.

#### D26 — The only control matrix in the repo is empty and false — **P1 STALE**

`e2e/data/control-matrix.json`: `_meta.status = "PENDING_APP_NOT_BUILT: web/src is empty and no dev server exists; no evidence collected"`, `generated_at 2026-11-06`, 29 surfaces with `controls: 0`, all `PENDING_APP_NOT_BUILT` except `S25 DECISION_GATED`. The app has 79 controls on Workspace alone (MEASURED). A QA artifact that says the app does not exist will silently pass any gate that trusts it.

**Required change:** regenerate the matrix from `e2e` runs (the harness exists: `e2e/playwright.config.ts`, `helpers/control-matrix` shape, `helpers/live-server.ts`), or delete the file rather than ship a false inventory; add a check that fails when `_meta.status` contradicts `web/src` existence. Gate: `matrix.test.mjs` freshness assertion.

---

## 6. State and view-model design (the single record every surface reads)

The design goal from `15` §3.2 is adopted, but with **two records, not one**, because D1/D2 prove one record cannot serve both live and history truthfully. The split is the correction this audit adds.

### 6.1 Two sources, one projection type

```ts
// web/src/state/agentView.ts  (NEW, pure, no React, no store import)
export type RunSource = 'live' | 'history' | 'none';
export type AgentClass =
  | 'hyperagent' | 'superagent' | 'worker' | 'subagent'
  | 'remote' | 'watchdog' | 'tool' | 'hive' | 'branch' | 'cluster';
export type StateFamily = 'live' | 'steady' | 'waiting' | 'terminal-negative' | 'system';
export type Density = 'compact' | 'standard' | 'detailed';
export type Band = 'B0' | 'B1' | 'B2' | 'B3' | 'B4';

export interface AgentViewNode {
  readonly taskId: string;
  readonly source: RunSource;              // 'history' nodes are never mutated
  readonly class: AgentClass;
  readonly classSource: 'wire' | 'role' | 'kind' | 'fallback';
  readonly label: string;                  // wire label, else `${roleLabel} · d${depth} · ${tail4}`
  readonly role: string | null;            // wire role/persona; never invented
  readonly personaId: string | null;
  readonly depth: number;
  readonly parentId: string | null;
  readonly hiveId: string | null;          // nearest superagent ancestor (self for a superagent)
  readonly branchId: string;               // `${hiveId ?? 'root'}/${taskId}`
  readonly state: AgentState;              // 16-state vocabulary, unchanged
  readonly family: StateFamily;
  readonly progress: number | null;        // null = absent, never 0
  readonly errorCode: string | null;
  readonly isHiveRoot: boolean;
  readonly childCount: number;
  readonly runningChildren: number;
  readonly metrics: {
    readonly tokens: number | null;
    readonly costUsd: string | null;       // settled Decimal as string; null = unknown
    readonly toolCalls: number | null;
    readonly messages: number | null;
  };
  readonly bankHint: { scope: 'role'|'hive'|'branch'|'run'|'project'|'unavailable'; count: number | null };
  readonly watchdog: { cadenceSecs: number; lastReportAt: string | null; nextTickAt: string | null; ownerTaskId: string | null } | null;
  readonly lastEventAt: string | null;
}
```

Builders: `buildAgentView(tasks, { source, budgets, metricsByTask })` and `treeRows(view)`, `clusterProjection(view, band)`, `bandFor(zoom, previousBand)`, `densityFor(band, setting)`.

### 6.2 Derivation rules (total, testable)

1. **class**: wire `agent_class` if in the union → `classSource:'wire'`; else `role==='subagent'` → `subagent` (`'role'`); else `kind==='hyperagent'|'superagent'|'tool'` → same (`'kind'`); else `kind==='worker'` → `worker` (`'kind'`); else `worker` + `'fallback'`. A `watchdog`/`remote` **only** from `agent_class` — never inferred. `classSource==='fallback'` renders a "class not reported" badge at Standard+ and a footnote in the inspector.
2. **label**: wire `label` if non-empty; else `${role ?? CLASS_LABEL[class]} · d${depth} · ${taskId.slice(-4)}`. The **first 8 hex is banned as identity** (MEASURED collision, §2.1); last-4 is the only id fragment allowed in a visible label, and it is `aria-describedby`-backed by the full id.
3. **progress**: wire number 0–100 or `null`. `null` renders **no track** (not an empty track). A `0` is a real measured zero.
4. **hiveId**: nearest ancestor with class `superagent`; a superagent's own hiveId is itself; a HyperAgent's is `null`; `branchId` = the same path key used by memory scope and A2A branch pairs.
5. **family**: `live` = running/thinking/streaming/planning; `steady` = healthy/completed; `waiting` = queued/awaiting_approval; `terminal-negative` = failed/killed/interrupted/cancelled; `system` = reconnecting/replay/empty/loading — system states render as **banners/skeletons**, never painted on an agent card.
6. **order**: stable topological (parent before child, then `spawn_seq`/created order). Never sort by label (`AgentHierarchy.tsx:30-31` is the current counter-example: it produces a shuffled tree).
7. **history source**: every node from a snapshot has `source:'history'`, `classSource` may be `'wire'` after snapshot v3; a history node never receives metrics from the live run and never appears in the live ControlBar target list.

### 6.3 Store slices (zustand, additive)

```ts
view: {
  nav: 'workspace'|'swarm'|'run'|'agents'|'data'|'models'|'settings';
  tab: 'workspace'|'swarm'|'history';            // Workspace-scoped only
  projection: 'hive'|'tree';                     // persisted
  density: Density;                              // persisted
  band: Band;                                    // runtime, hysteresis-guarded
  inspector: null | { kind:'agent'|'run'|'approval'|'memory'|'mailbox'; id: string };
  filters: { hideFinished: boolean; focusBranch: boolean; query: string; class: AgentClass|null; family: StateFamily|null };
  selectedRunId: string | null;                  // history destination
}
runs: { live: RunRecord|null; history: RunRecord[]; liveTasks: Record<string,TaskRecord>; historyTasks: Record<string,TaskRecord>; source: RunSource; }
settings: { …existing…; autonomyMode:1|2|3|4; approvalPolicy; budgetMode:'hard'|'soft'; a2aMode; memoryScopes; retention; density; projection; tutorial; humanOnly: {…}; asOf: number|null; writable: boolean }
budget: { meters: Meter[7]; secondary: Meter3; inFlight:{current:number;limit:8}; asOf: number|null; status:'idle'|'loading'|'ok'|'error'; refusal: Refusal|null }
providerCache: { account; catalogue; usage; cache; asOf; errors: {account?:string; catalogue?:string; usage?:string} }
memoryBank: { phase:'role-only'|'banks'; scope: 'project'|'run'|'hive'|'branch'|'role'; rows; readSet; truncation; refusal }
mailbox: { rows; count; cap:32; status:'not-reported'|'open/answered/expired'; roster; refusal }
approvals: { pending; inbox; ledger; stale }
refusals: { toasts; ledger }                      // ledger append-only, session-labelled
```

**Memo keys** (the only inputs that may trigger a projection): `(source, graphRev, sorted id:class:parentId, filters.class, filters.family, filters.query)`. **Selection, density, band and inspector are never in the key** — that is the fix for D11's per-selection dagre run (`Canvas.tsx:154-163`).

### 6.4 Card registry (total over `AgentClass`)

| Class | Component | Where | Required content (Standard density) | Honest-unknown rule |
|---|---|---|---|---|
| `hyperagent` | `HiveChatCard` (exists as `ChatCore`) | Workspace centre overlay | title, run state, plan `n steps`, model/thinking if reported, budget chip, open-run-detail | no model chip when unreported |
| `superagent` / `hive` | `SuperAgentNode` (+ hive outline) | Canvas, tree, inspector | identity, `SUPERAGENT`, role chip, state, `N agents · M running`, bank badge, progress if reported | bank badge `—` until banks exist |
| `worker` | `WorkerAgentNode` | Canvas, tree, inspector | identity, `WORKER`, role chip, state, branch chip, task line, cost/tools strip | cost `—` until D14 route |
| `subagent` | `SubagentCard` (NEW) | Canvas, tree, inspector | identity, `SUBAGENT` + "worker tier (subagent role)" chip, parent link, ask-parent status if reported, state | tier chip is factual (wire role) |
| `watchdog` / `remote` | `WatchdogCard` (NEW) | Canvas daemon band, Agents registry, Settings | identity, `WATCHDOG`, cadence, last report, next tick, owner | capability gate: "not available on this build" until daemon APIs exist |
| `tool` | `ToolAgentNode` | Canvas (only if a tool-tier task exists) | identity, `TOOL`, state, args sha/preview when sent | no args box when absent |
| **error** (state, not class) | `ErrorCard` replacement | Canvas card replacement | identity, real `error_code` or "error state", halted edge | never `Compile failed` |
| `cluster` | `ClusterCard` | Canvas B0 | `N agents · state` | not selectable as an agent |
| `branch` | `BranchBand` | Canvas band, tree group | branch label, `N agents · M running`, aggregate state | — |

Enforcement: `const CARD_FOR: Record<AgentClass, ComponentType<CardProps>>` with a `satisfies` check and a unit test asserting the key set equals `AGENT_CLASSES`; a class with no producer still renders (capability-gated card), never a missing node.

### 6.5 LOD + density (adopted from `05`, pinned by test)

| Band | Zoom | Renders | Effective density (min(band, setting)) |
|---|---|---|---|
| B0 | 0.10–0.34 | batched SVG clusters + `ClusterCard`; no card DOM | cluster only |
| B1 | 0.35–0.59 | header + state + identity (1 line) | compact |
| B2 | 0.60–1.19 | full card | standard (default) |
| B3 | 1.20–2.19 | + role/branch chip, last event | detailed |
| B4 | 2.20–3.00 | + tool args/preview, error code, bank badge | detailed + evidence |

Hysteresis ±0.04, band in a small store slice (not per-frame React state), `onlyRenderVisibleElements`, layout memo on topology key, animated-edge cap (≤50), minimap class/state rectangles + viewport outline + "Overview" above 1280.

### 6.6 Event → state contract (what must be subscribed)

| Event | Slice | Change from today |
|---|---|---|
| `NODE_SPAWN`/`NODE_STATE`/`NODE_REWIRE`/`NODE_DRAG` | `runs.liveTasks` | consume `label`/`progress`/`agent_class`/`role`/`persona_id`/`depth`; do **not** default kind to `worker` (`store.ts:1448-1451`) |
| `APPROVAL_REQUEST` | `approvals.pending` | keep `args_preview` (now wired), add `taskLabel`/`branchId` from the projection |
| `APPROVAL_RESOLVED` | `approvals.ledger` | append decided row (nonce consumed yes/no, decision, at) |
| `AGENT_MESSAGE` | `mailbox.rows` | append row, not just `agentMessageVersion++` (`store.ts:1495-1497`) |
| `TOOL_CALL`/`TOOL_RESULT` | `toolActivity` + run-detail ledger | keep; add branch/turn grouping keys |
| `RUN_METRICS` | `hud.census` | stop treating `green_pct` as "GREEN"; use `superagents/workers/completed/total` |
| `ERROR` | `refusals.ledger` | append ledger row (D6) |
| `BUDGET_UPDATE` (new, optional) | `budget` | prefer polling `GET /api/runs/{id}/budget` every 2 s while active |
| `WATCHDOG_STATE`/`WATCHDOG_REPORT` (new) | `roster`/watchdog view | only after the daemon table exists |

**Subscriber parity requirement:** every event name emitted by `Event::` variants must appear in the client union and in a handler; today `AGENT_MESSAGE` is handled only as a version bump, and there is no handler for anything the server cannot emit (correct). The test derives both sets from source (`events.json` in `trace` terms).

### 6.7 Contract additions required by the UI (additive, ordered)

1. `NodeEvent`: `label?`, `progress?`, `agent_class?`, `role?`, `persona_id?`, `depth?` (`event.rs:248-271`); contract matrix + golden schema regeneration.
2. Snapshot `v3` per-run projection with the same node fields (`snapshot.rs:26-41`, `graph.rs:18-26`), plus `run_status`, `captured_at`, `revision`.
3. `GET /api/history/runs/{id}/state?revision=N` → per-run projection; add `?schema=v3` tolerance and keep v2 decode (a mixed data dir must render both).
4. `GET /api/runs/{id}/budget` → seven meters + `asOf` + `provenance` + `secondary` + `inFlight`.
5. `GET /api/runs/{id}/approvals` → pending + decided ledger (nonce consumed, decision, at, tool, args sha/preview when available).
6. `GET /api/agents/memory?provenance=1` → add `run_id`/`task_id`/`written_at`; `POST /api/memory/purge` + `promote` after migration v11.
7. `GET /api/agents/runs/{id}/messages` → add `count`, `cap`, `status`; `POST /api/agents/runs/{id}/messages` (reply) only after delivery semantics exist.
8. `GET/PUT /api/settings` (DB-backed, audited) + `GET /api/settings/effective`.
9. `GET/POST/PATCH /api/watchdogs` + `WATCHDOG_*` events after the daemon table exists.
10. `GET /api/provider/usage?as_of` and `supported_by_build` on `/api/provider/models`.

Every mutating HTTP route returns the same refusal shape (`code/message/how_to_fix/retryable`) — the live server already does for typed 4xx (`E_RUN_NOT_FOUND` measured) but **not** for unknown API paths, which return `{"error":"not found","schema":"hiverouter.v1"}` (MEASURED) — unify that.

---

## 7. Exact change list (work packages, anchors, gates)

Each row is a unit of work with a named gate. Anchors are current as of the hashes in §14.2.

### W1 — Snapshot and boot truth (P0, unblocks everything else)

| File | Change | Anchor |
|---|---|---|
| `backend/…/graph.rs` | add `nodes_for_run(run_id)` projection; extend `TaskNode` with `kind/role/label/progress/agent_class/depth` optional fields | `graph.rs:18-26`, `:220` |
| `backend/…/snapshot.rs` | `capture` takes a run projection; bump `SNAPSHOT_SCHEMA_V3`; migrate v2→v3 (fill `kind:null`) | `snapshot.rs:20-41`, `:211-252`, `:258-272` |
| `backend/…/execution.rs` | pass the run's task list to `capture`; fill `label`/`kind`/`role`/`agent_class` on `NodeEvent` publish | `execution.rs:743-779` (`publish_node`), `:765-777` (`publish_node_spawn`), `:97-106`, `:151-162` |
| `web/src/state/store.ts` | split `liveTasks`/`historyTasks`; delete `recordsFromSnapshotGraph` fusion; keep `snapshot` slice for the history banner | `store.ts:545-626`, `:1150-1163` |
| `web/src/flow/graph.ts` | delete `buildReplayTasks`; replay renders `buildAgentView(historyTasks,{source:'history'})` | `graph.ts:190-234` |
| `web/src/panels/HistoryPanel.tsx`, `Canvas.tsx` | read-only mode from `runs.source`, not from `run!==null` | `HistoryPanel.tsx:20-24`; `Canvas.tsx:266-275`, `:352-353` |

Gate: `agent_view.rs` (producer fields), `graph.replay.test.ts` (per-run set, no foreign ids), live: replay revision 1 ≠ revision 7 when the run advanced.

### W2 — Agent identity and card totality (P0/P1)

| File | Change | Anchor |
|---|---|---|
| `web/src/state/agentView.ts` | **NEW** projection (§6.1–6.3) | — |
| `web/src/flow/nodes.tsx` | card registry: add `SubagentCard`, `WatchdogCard`, `ClusterCard`, `BranchBand`; delete `HyperAgentNode`; strip permanent `—` chrome | `nodes.tsx:141-275` |
| `web/src/flow/graph.ts` | nodes built from `AgentViewNode`; sizes from class; keep the anchor edge | `graph.ts:30-40`, `:143-165` |
| `web/src/panels/AgentsPanel.tsx` | cards read the projection; add class/state filters; link to inspector | `AgentsPanel.tsx:40-64` |
| `web/src/panels/AgentInspector.tsx` | **NEW** inspector (D25) | — |
| `web/src/state/agentState.ts` | add `stateFamily()`; keep 16 states; add `CLASS_LABELS` | `agentState.ts:9-26`, `:111-135` |

Gate: `agentView.test.ts`, `agentCards.test.ts` (totality + honest class), `agentInspector.test.ts`.

### W3 — Tree projection and IA (P1)

| File | Change | Anchor |
|---|---|---|
| `web/src/panels/AgentTree.tsx` | **NEW** APG treegrid, root summary, columns, virtualization | supersedes `AgentHierarchy.tsx:21-81` |
| `web/src/panels/SwarmPanel.tsx` | delete 5 tiles + hero; add Hive/Tree toggle, filters, legend, root summary | `SwarmPanel.tsx:38-107` |
| `web/src/App.tsx` / `NavRail.tsx` | nav `workspace|swarm|run|agents|data|models|settings`; tabs Workspace-scoped; add Run detail | `App.tsx:50-102`, `:125-135`; `NavRail.tsx:9-16` |
| `web/src/panels/RoutingPanel.tsx` | delete destination; move `Parent` into the tree's developer disclosure | file + `App.tsx:96-97` |
| `web/src/panels/LibraryPanel.tsx` | drawer below `lg`; project from registry; tablist only on Workspace | `LibraryPanel.tsx:15-20`, `:45`, `:48-52` |

Gate: `treegrid.test.ts`, `ia.test.ts`, `responsive.test.ts`.

### W4 — Run detail as the evidence surface (P1)

| File | Change | Anchor |
|---|---|---|
| `web/src/panels/RunDetailPanel.tsx` | **NEW**: summary, stages, tool ledger, model/usage, failures/refusals, files, revisions (board 12) | — |
| `web/src/panels/ToolActivityPanel.tsx` | becomes the ledger: branch/turn grouping, outcomes, code, sha, bytes, filters, cap footer | `ToolActivityPanel.tsx:131-184` |
| `web/src/chat/ChatCore.tsx` | delete ACTIONS table; one-line strip → Run detail | `ChatCore.tsx:274-308` |
| `web/src/panels/AgentsPanel.tsx` | replace the inline ledger with a link + count | `AgentsPanel.tsx:65-72` |
| `web/src/panels/ApprovalInbox.tsx` | **NEW** inbox + decided ledger (D5) | complements `ApprovalOverlay.tsx:221-244` |
| `web/src/state/store.ts` | `runs.history`, `selectedRunId`, `refusals.ledger`, `approvals.ledger` | `store.ts:147-159`, `:784-795` |

Gate: `runDetail.test.ts`, `approvalInbox.test.ts`, `refusalLedger.test.ts`.

### W5 — Seven meters, provider/cache, cost (P1)

| File | Change | Anchor |
|---|---|---|
| `backend/…/db.rs` + migration | `runs.calls_attempted/cost_usd/cost_status/ended_at`; `provider_calls` table (anchor not re-read this pass; schema 10 is MEASURED via `/health`) | `db.rs` migrations |
| `backend/…/http.rs` | `GET /api/runs/{id}/budget`; extend `/api/usage/runs/{id}` with `cost_status` | `http.rs:527` |
| `backend/hiverouter-provider/src/budget.rs` | persist `RunAccounting::report()` at terminal; expose attempts | `budget.rs:25-52` |
| `web/src/panels/BudgetStrip.tsx` | **NEW** seven meters + secondary group + cap-hit refusal link | — |
| `web/src/panels/HudBadge.tsx` | census + 2 budget chips; remove GREEN | `HudBadge.tsx:36-51` |
| `web/src/panels/ModelsPanel.tsx` | provider & cache panel: `as_of`, ratio, settled cost, server-derived support, type floor | `ModelsPanel.tsx:31`, `:196-256` |
| `web/src/state/metrics.ts` | replace `greenPct` derivation with explicit census | `metrics.ts:22-47` |

Gate: `budgetMeters.test.ts`, `providerCache.test.ts`, `budget_read.rs`.

### W6 — Memory and A2A inspectors (P1)

| File | Change | Anchor |
|---|---|---|
| `web/src/panels/MemoryInspector.tsx` | **NEW** phased scopes, trust, read-set, truncation, promote/purge (no free-text), confirmation on clear | replaces `AgentMemoryPanel.tsx:20-123` |
| `web/src/panels/MailboxPanel.tsx` | **NEW** branch-pair ledger, count/32, status honesty, approval link, roster tab | replaces `AgentMessagesPanel.tsx:21-108` |
| `web/src/state/store.ts` | `memoryBank`, `mailbox`, `roster` slices; `AGENT_MESSAGE` append | `store.ts:1495-1497` |
| `backend/…/http.rs` | `?provenance=1`, `count`, `roster` route; later `promote/purge/reply` | `http.rs:521-526` |

Gate: `memoryInspector.test.ts`, `mailbox.test.ts`, `memory_banks.rs` (after v11).

### W7 — Settings, autonomy modes, tutorial (P1)

| File | Change | Anchor |
|---|---|---|
| `backend/…/settings.rs` + migration | settings table + audit rows + `GET/PUT /api/settings` | new; `http.rs:514-531` |
| `web/src/panels/SettingsPanel.tsx` | sections (General/UI, Autonomy, Approvals, Agents & templates, Memory*, A2A, Provider & budget, Data & retention*, Advanced*), human-only disabled+confirmed, env-only labelled | `SettingsPanel.tsx:32-118` |
| `web/src/panels/TutorialOverlay.tsx` | **NEW** 5-step first-run guide, resumable, skippable, focus-trapped, no feature claims beyond capability status | — |
| `web/src/state/store.ts` | `settings` slice with `writable`, `asOf`, `humanOnly` | `store.ts:213-224` |

Gate: `settingsModes.test.ts`, `tutorial.test.ts`, `http_boot.rs` (origin/refusal on PUT).

### W8 — Canvas performance, LOD, motion (P1/P2)

| File | Change | Anchor |
|---|---|---|
| `web/src/flow/lod.ts` | **NEW** bands/hysteresis/culling | — |
| `web/src/flow/Canvas.tsx` | `onlyRenderVisibleElements`, band subscription, cluster layer, panning perf, `role` change or APG grid | `Canvas.tsx:154-163`, `:278-284`, `:351-405` |
| `web/src/flow/HiveField.tsx`, `styles/index.css`, `styles/motion.css` | one ambient loop, remove scan + free bees, event-driven only | `index.css:169-331`; `motion.css:42-348` |
| `web/src/tokens/tokens.ts` | `TYPE_FLOOR_PX`, density spacing, card sizes for subagent/watchdog | `tokens.ts:91-100`, `:166-179` |

Gate: `lod.test.ts`, `designParity.test.ts` (animations ≤3/0), `tokens.test.ts` (type floor).

### W9 — Session truth surfaces (P0 leftovers)

| File | Change | Anchor |
|---|---|---|
| `web/src/state/store.ts` | project identity from registry; delete localStorage id | `store.ts:334-363` |
| `web/src/panels/HistoryPanel.tsx`, `App.tsx` | bottom-padding fix for the rail's LIVE button; rename copy to `revision N of M` + time | `App.tsx:131-135`; `strings.ts:132-138`, `:429` |
| `web/src/panels/ControlBar.tsx`, `Canvas.tsx`, `AgentMemoryPanel`→inspector | `ConfirmDialog` on kill/rewire/route/clear; streaming guard | `ControlBar.tsx:83-116`; `Canvas.tsx:368-397` |

Gate: `historyRail.test.ts`, `projectIdentity.test.ts`, `destructive.test.ts`.

### W10 — Harness truth (P1)

| File | Change | Anchor |
|---|---|---|
| `e2e/data/control-matrix.json` | regenerate from the live app (79 controls on Workspace is the current truth) or delete | `_meta.status` |
| `e2e/tests/ui-audit.spec.ts` (+ helpers) | encode §10 tests T1–T20 as browser assertions where they are DOM-level | `e2e/playwright.config.ts` |

Gate: matrix freshness test; e2e suite green on a fake-provider fixture.

---

## 8. Removal list (delete or demote — do not restyle)

| # | Remove | Anchor | Replaced by |
|---|---|---|---|
| R1 | 5 Swarm stat tiles (`SUB-HIVES/WORKERS/TOOLS/ERRORS/ACTIVE`) | `SwarmPanel.tsx:38-51` | root summary + census (W3) |
| R2 | Hero empty state + 3 numbered steps | `SwarmPanel.tsx:53-107` | one honest empty line + "Start in Workspace" |
| R3 | Chat `ACTIONS` table | `ChatCore.tsx:274-308` | one-line tool strip → Run detail |
| R4 | Agents-panel tool table | `AgentsPanel.tsx:65-72` | link + count |
| R5 | `HyperAgentNode` canvas card | `nodes.tsx:245-267` (+ filter `graph.ts:151`) | `HiveChatCard` only |
| R6 | `buildReplayTasks` flattening | `graph.ts:190-234` | snapshot projection (W1) |
| R7 | Permanent `—`/`ctx`/`score`/`attempt` slots | `nodes.tsx:164-168`, `:183-189`, `:214-219`, `:236-239` | fields that exist, or nothing |
| R8 | Static role caption "decomposes intent, owns the sub-hive" | `nodes.tsx:160-162`; `strings.ts:281` | wire role/persona chip |
| R9 | `HUD GREEN%` + `delta` | `HudBadge.tsx:36-51`; `metrics.ts:22-47` | census + budget chips |
| R10 | Routing destination | `NavRail.tsx:14`; `App.tsx:96-97`; `RoutingPanel.tsx` | tree `Parent` disclosure |
| R11 | Global library tabs on non-Workspace destinations | `LibraryPanel.tsx:15-20`, `:54-96`; `App.tsx:50-102` | Workspace-scoped tablist |
| R12 | localStorage project identity (`hivemind.project-id`, `HiveMind AI`) | `store.ts:334-363`; `Header.tsx:62-68` | `GET /api/projects` |
| R13 | Two of three ambient loops + `hive-scan` + free-flying bees | `Canvas.tsx:295-307`; `index.css:169-331` | one 18 s ambient |
| R14 | Sub-11 px type (19 occurrences) | tokens floor `tokens.ts:99`; live nav `NavRail.tsx:61`; Models `:241,245` | `type-mono-meta` (11) / board 9.5 |
| R15 | `WIRE_MODELS` constant badge | `ModelsPanel.tsx:31`, `:241-243` | server `supported_by_build` |
| R16 | `CHAT_PIN` command path with no control (or add the control) | `store.ts:2105-2108` | remove `pinChat` + keep ACK no-op |
| R17 | Stale e2e matrix claim | `e2e/data/control-matrix.json` `_meta` | regenerated or deleted |
| R18 | `Routing` wording for revisions | `strings.ts:132-138`, `:429`; `Scrubber.tsx:43-45` | "revision N of M" + time |

**Do not remove:** the spatial canvas, the chat core, the FigCraft tokens, the 16-state vocabulary, the approval nonce/TTL state machine, the refusal copy style (`code/message/how_to_fix`), the mock API boundary (`web/src/api/mock.ts`), the replay read-only intent.

---

## 9. Accessibility and motion rules (normative, testable)

### 9.1 Roles and labels

1. The canvas is **`role="region"` + `aria-roledescription="hive canvas"`** with an accessible name, or a full APG composite grid with arrow-key navigation and tests. `role="application"` is removed until the grid contract exists (`Canvas.tsx:281`; MEASURED 2 such elements today).
2. The Tree projection is the accessible peer: `role="treegrid"`, `aria-rowcount`, `aria-rowindex`, `aria-expanded`, roving tabindex, `Up/Down/Left/Right/Home/End/Enter/Space`, root summary row.
3. Agent cards are **one** interactive element each (a `button` or a `role="option"` in the grid), never a button containing buttons; accessible name = `class · role · depth · short-id · state` (no raw 8-hex prefix; `graph.ts:160` today).
4. Ledgers (tools, approvals, refusals, messages) are `table`/`grid` with header association or a definition list; row actions have stable `aria-label`s including the row identity.
5. Live regions: at most **one** assertive region (toasts); status changes polite; **no live region may update more than once every 10 s** — the approval TTL ticker (`ApprovalOverlay.tsx:291`) must announce at 60/30/10 s or on state change only.
6. Dialogs: `role="alertdialog"` only for approvals (decision + consequence), `role="dialog"` elsewhere; focus moves in, is trapped, returns on close; **`Escape` closes without deciding**; deny is first in DOM and focus order; one modal at a time (queue the rest, D5).
7. Skip link goes to the main content region, not to a `role=application`; tab order budget: composer reachable within the first 10 stops on Workspace (today: not within 12, §2.1).

### 9.2 Targets, focus, contrast

8. Minimum target 24×24 px; primary actions 40 px (`05` P1-11); the 8 px nav labels stay inside their 48 px button, so size is met — the **text** floor is the failure, not the target.
9. Focus ring: 2 px `ink-700`, visible on canvas cards, rail buttons, tabs, and dialog actions; never removed without replacement (`tokens.ts:72-79`).
10. Colour is never the only channel: state = dot/glyph **+ text** (`agentState.ts:58-84` already provides shapes; cards must print the label, not only the dot — `WorkerAgentNode` prints only a dot, `nodes.tsx:180`).
11. Amber is large-text only (`tokens.ts:77-78`); meta text uses `ink-500` (declared AA) or darker.

### 9.3 Motion

12. **One ambient loop maximum** (18 s honeycomb travel). Sheen 6 s optional, pulse 1.6 s only while a run is live, working-bee 1.2 s only while streaming. Remove the scan sweep and the three free-flying bees (`Canvas.tsx:295-307`).
13. Event-driven transitions only: card state 120–160 ms, edge re-route 200–240 ms, toast 150 ms, hover lift 120 ms, focus 160 ms. No idle animation on a static graph.
14. `prefers-reduced-motion: reduce` **and** the in-app override produce `document.getAnimations().length === 0` for loops (today: 0 — keep it) and static glyphs for the working bee.
15. Gate: `getAnimations()` ≤3 concurrent with no run, 0 under reduced motion, ≤1 infinite loop; assert in `designParity.test.ts` for both preference states.
16. No auto-playing video/parallax; no motion that cannot be paused; no layout shift from animation.

### 9.4 Keyboard map (adopt `05` §3.10, minimum)

`Tab/Shift+Tab` move focus; `Arrows` move selection in tree/grid; `Enter` opens inspector; `Space` toggles selection; `Ctrl/Cmd+0` recentre; `+/-/0/F` zoom/reset/fit; `R` opens the re-route dialog; `K` opens the kill confirmation; `Escape` closes dialog without deciding; `?` help. Shortcuts never fire while a text field has focus; no action is drag-only (today rewire is drag-or-select, so this holds only after W9's dialog).

---

## 10. Twenty acceptance tests (none run in this pass)

Type: **U** unit/pure, **D** DOM (testing-library/vitest), **R** Rust crate test, **L** live/browser e2e, **H** HTTP route probe.

| # | Test file | Type | Assertion (must fail before the change, pass after) |
|---|---|---|---|
| T1 | `web/tests/agentView.test.ts` (new) | U | `buildAgentView` is total over `AgentClass`; `classSource` is `'wire'` only when `agent_class` was present; `label` never contains `taskId.slice(0,8)`; `progress:null` yields `null` (not 0); parent/depth/hive/branch derive from parent links; 600 synthetic nodes produce deterministic order |
| T2 | `web/tests/agentCards.test.ts` (new) | D | Every `AgentClass` maps to exactly one card (`CARD_FOR` key set equals `AGENT_CLASSES`); `subagent` renders `SubagentCard` with the tier chip; `watchdog`/`remote` render a capability-gated card with "not available on this build"; no card prints `—` for a field absent from the model |
| T3 | `web/tests/graph.replay.test.ts` (update) | U | A v3 snapshot fixture for run A with nodes `{A1,A2}` produces exactly 2 replay nodes labelled from the wire; a foreign node id in the fixture is **absent**; a completed run's replay has zero `family==='terminal-negative'` nodes not present in its own run rows |
| T4 | `web/tests/history.test.ts` (new) | D | Boot with 5 runs and no live run renders the "read-only history" banner, 0 live cards, and disabled mutations; selecting a run changes `selectedRunId`; revision 1 vs 7 differ when the fixture differs |
| T5 | `web/tests/treegrid.test.ts` (new) | D | `role="treegrid"`, `aria-rowcount`, root summary row; `ArrowDown/Up/Left/Right/Home/End` move/expand/collapse; `Enter` opens the inspector; virtualization reports correct `aria-rowindex` at 600 rows; selection syncs to the canvas |
| T6 | `web/tests/lod.test.ts` (new) | U | Band thresholds B0–B4 with ±0.04 hysteresis (0.34→B0, 0.35→B1, 0.339→B0); effective density = min(band, setting); B0 emits clusters only; culling helper returns ≤120 nodes at 600; **layout key unchanged by selection** |
| T7 | `web/tests/budgetMeters.test.ts` (new) | D | Exactly seven run meters in the fixed order (agents 64, depth 3, fan-out 8, calls 40, tokens 400 000, cost $0.50, messages 32); `null` renders `—`; 80 % renders a text+icon warning; a cap-hit renders the named refusal and highlights the meter; calls/cost/messages read "not reported" when the route is absent; the tool-loop group is separate; in-flight is a chip, not a meter |
| T8 | `web/tests/approvalInbox.test.ts` (new) | D | N pending approvals produce **one** `role=alertdialog` plus N inbox rows; `args_preview` renders with `[redacted]` markers preserved and "preview unavailable" when absent; deny precedes approve in DOM; `Escape` closes with no command; dismiss button exists **iff** `approvalActionsEnabled().dismiss`; expired/stale rows are dismissible outside the failure block; decided ledger row records decision + nonce consumed |
| T9 | `web/tests/refusalLedger.test.ts` (new) | D | 10 refusals leave 10 ledger rows after all toasts are dismissed; each row shows code/message/how_to_fix/retryable and a `session-only` label; a failed run renders its code + message + how_to_fix in Run detail |
| T10 | `web/tests/memoryInspector.test.ts` (new) | D | Phase A renders `bank tree not available on this build`, lists all 7 role banks (0-entry ones included), columns `key/value/written_at` when the route supplies them, clear requires confirmation and posts once; read error renders a refusal, never an empty success |
| T11 | `web/tests/mailbox.test.ts` (new) | D | Rows group by branch pair; header shows `messages N / 32`; status renders `not reported` while the payload lacks it; `AGENT_MESSAGE` appends a row without a refetch; no compose control until `AGENT_MESSAGE_REPLY` exists; roster tab lists id/label/class/parent/depth/state |
| T12 | `web/tests/providerCache.test.ts` (new) | D | Support badge is derived from a server field (a fixture with `supported_by_build:false` renders "not accepted"); balance shows an `as of` time; cache ratio renders only when hit+miss are numbers; cost renders only when `cost_status==='complete'`, else "settled cost unavailable"; no key/DOM leak; independent error states per card |
| T13 | `web/tests/settingsModes.test.ts` (new) | D | Four autonomy modes with the policy matrix; default 1; human-only rows (retention/visibility/blindness) are disabled with a tooltip and require confirmation; a PUT refusal renders `code/message/how_to_fix`; when `settings.writable===false` no input is interactive and the panel says `boot-configured — restart required` |
| T14 | `web/tests/tutorial.test.ts` (new) | D | Five steps link to real surfaces (`Workspace`, `Swarm`, `Run detail`, `Approval inbox`, `Memory/Mailbox`); skip/resume persist; `Don't show again` persists; focus trapped; `Escape` closes without navigation; a step that names an unavailable capability renders the capability gate, not a promise |
| T15 | `web/tests/designParity.test.ts` (update) | D | Card sizes/radii pinned (super 420×150 r14, worker 320×96 r12, tool 420×240 r14, error 320×110 r12, subagent 280×88, watchdog 320×120 — the last two OPINION); HUD shape 460×44; **`getAnimations()` ≤3 with no run and 0 under `reduce`**; at most one infinite loop; meta type ≥9.5 px on cards |
| T16 | `web/tests/tokens.test.ts` (update) | U | `TYPE_FLOOR_PX = 11`; a scan of `web/src/**/*.tsx` finds zero `text-[<11px]` outside a documented allowlist of exactly two 9.5 px card-meta rules; 20 swatches; contrast table unchanged |
| T17 | `web/tests/a11y.test.ts` (new) | D | Canvas exposes `role="region"` (or a tested grid); exactly one assertive live region; the approval TTL region updates no more than once per 10 s; every interactive card has a name containing class + state and no raw 8-hex prefix; focus ring computed style is `ink-700` 2 px |
| T18 | `web/tests/responsive.test.ts` (new) | D | At 1000 px a drawer toggle (`aria-expanded` stable selector) exposes project, chips, templates and legend; templates are disabled with an inline reason when `run===null`; the minimap has an "Overview" action above 1280 |
| T19 | `backend/crates/hiverouter-core/tests/agent_view.rs` (new) | R | `NODE_SPAWN`/`NODE_STATE` publish `label/kind/role/agent_class/depth` when known and **omit** them when unknown; no guessed kind; snapshot v3 contains the same fields and only the run's own tasks; v2 snapshots still decode (migration path) |
| T20 | `backend/crates/hiverouter-core/tests/budget_read.rs` (new) | R | `GET /api/runs/{id}/budget` returns seven meters with `null` (never 0) for unknown, `asOf`, provenance per meter, cost only when `cost_status==='complete'`, attempts persisted across restart; a cap hit correlates with the named refusal row; `GET /api/runs/{unknown}/budget` → 404 named refusal with the standard shape |

**Live verification (not a unit test) that must accompany W1 before any UI claim:** boot the app against the current DB, open History, select revision 1 and 7, and compare the rendered node set and labels; the run's own task ids must be the only ids on screen (§2.3 is the RED state).

---

## 11. Twenty new questions (implementation forks this audit surfaces)

These are **new** relative to `15` §8's twenty; each is a decision the implementation needs. Recommendation is OPINION unless a source is cited.

1. **Snapshot scope vs storage cost.** (a) per-run projection written at each revision (bigger write, correct replay); (b) keep global snapshot + filter at read time by `tasks_for_run`; (c) both (global file + per-run index). **Recommend (a)** with a `run_tasks` index row per revision; (b) is cheaper now but keeps a whole-graph file per run and tempts future misuse. Evidence: `snapshot.rs:258-272`, MEASURED §2.3.
2. **Where does the live/history split live?** (a) two task maps + `source` flag (proposed); (b) one map with a per-node `source`; (c) separate store instance for history. **Recommend (a)** — the ControlBar/Chat gating is per-mode, not per-node.
3. **Naming of the class field.** (a) `agent_class`; (b) `tier` (reuses runtime vocabulary); (c) `kind` + `role` only. **Recommend (a)** — `tier` already means `AgentTier` and would invite a false claim that Subagent is a tier.
4. **Does `role` become the persona id or a free string?** (a) free string + separate `persona_id`; (b) persona id only; (c) normalized enum. **Recommend (a)** — planners emit free-string roles today (`execution.rs:151-162`).
5. **How is "class not reported" rendered?** (a) a small `?` badge with tooltip at standard, footnote at detailed; (b) a full card variant; (c) hide the class row. **Recommend (a)**; never assert worker.
6. **Does the history destination replace the History tab or sit beside it?** (a) `Run detail` destination holds a run picker + revisions; History tab becomes a read-only banner state; (b) keep both. **Recommend (a)** — one evidence surface (board 12).
7. **Where does the past-run list live?** (a) Run detail dropdown fed by `/api/history/runs`; (b) separate "Runs" destination; (c) History rail list of runs (not revisions). **Recommend (a)** now, (b) at multi-hive scale.
8. **Budget polling vs event.** (a) `GET /api/runs/{id}/budget` every 2 s while active; (b) new `BUDGET_UPDATE` event; (c) both. **Recommend (a)** first (bounded, no WS freeze), (b) only if measurements show the poll is visible.
9. **Cost settlement UI.** (a) show `cap · spend not settled` until `complete`; (b) hide the row until settled; (c) show an estimate marked `estimated`. **Recommend (a)**; never display an estimate as spend (`06` §6.1).
10. **Approval inbox placement.** (a) right rail, persistent count in HUD; (b) Run detail section only; (c) both. **Recommend (a)** with a link from Run detail; a blocking approval must be visible from any destination.
11. **Approval TTL announcements.** (a) 60/30/10 s + on state change; (b) never announce time, announce only state; (c) every 10 s. **Recommend (a)** — satisfies "no >10 s live-region churn" while staying useful.
12. **Refusal ledger retention.** (a) session-only with a label; (b) persist in localStorage keyed by run; (c) new DB table. **Recommend (a)** now, (c) when the approvals ledger route lands (same migration).
13. **Destructive confirmations.** (a) one shared `ConfirmDialog` with typed target identity; (b) inline two-step button; (c) undo toast. **Recommend (a)** + no undo (the effects are real).
14. **Memory clear vs purge.** (a) disable Clear in Phase A until confirmation + audit exist; (b) keep with confirmation; (c) replace with per-entry purge. **Recommend (b) now, (c) after v11**; the current one-click DELETE is the worst option.
15. **Sources for the memory scanner.** (a) role-only until v11; (b) show all five scopes disabled with counts; (c) hide the panel. **Recommend (b)** — an operator who cannot see the gap will assume the feature exists.
16. **Tree virtualization threshold.** (a) 200 rows; (b) 100; (c) always virtualize. **Recommend (a)**; measure first at 600.
17. **Density persistence.** (a) per operator localStorage; (b) settings DB; (c) per run. **Recommend (a)** for density/projection, (b) for autonomy/policy (audited).
18. **Tutorial trigger.** (a) first run of a fresh profile only; (b) every new build; (c) manual from Help. **Recommend (a)** + Help entry; never auto-open when an approval is pending.
19. **e2e matrix ownership.** (a) regenerate from Playwright and commit; (b) generate on demand and gitignore; (c) delete. **Recommend (a)** with a freshness test — the current empty matrix is worse than none.
20. **Do we keep `RUN_METRICS.green_pct`?** (a) keep, rename to `completed_pct`, define the denominator in the contract; (b) delete the field; (c) keep for HUD only as a sparkline. **Recommend (a)** — it is a real measured ratio, only the label and denominator are wrong.

---

## 12. Top ten changes, ranked by operator-visible truth gained per unit of work

1. **Per-run snapshots + live/history split (W1).** Kills the two worst lies at once: a completed run painted with 17 foreign failed nodes, and 35 anonymous "Agent …" cards presented as live. Everything else in this report is measured against that surface.
2. **Agent class on the wire + total card registry (W2).** Subagent, Remote/Watchdog, Hive, Branch, Cluster become representable; the operator's "every agent gets a card" stops being false.
3. **Run detail as the single evidence surface (W4).** One place for summary, tool ledger, approvals, failures, files, revisions; deletes two duplicate tables and gives the failure ledger a home.
4. **Seven meters + provider/cache panel with real provenance (W5).** Caps the server enforces become visible; cost stays honest (`—` or "not settled") instead of absent; the `WIRE-SUPPORTED` constant stops pretending to be a capability check.
5. **Approval inbox + honest decision dialog (D5).** Render the preview that already exists, expose the Dismiss that the state machine already allows, one modal, focus trapped, Escape safe, throttled announcements.
6. **Boot truth and HUD census (D2/D4).** No percentage that spans five runs; no `0 SUPERAGENTS` because the root is unrepresentable; a banner that says "read-only history".
7. **Treegrid peer + IA restructure (W3).** Accessibility path at scale and the operator's density view; Routing demoted; tabs scoped; Run detail added.
8. **LOD/culling/density + motion budget (W8).** 72 concurrent animations → ≤3; selection stops re-laying out; the canvas becomes survivable at 150–600 agents.
9. **Memory/mailbox inspectors with capability honesty (W6).** The two HMARS surfaces that currently say "empty" while hiding scope gaps; add confirmation to the destructive clear.
10. **Session truth leftovers (W9/D6/D8/D9):** one project identity, a working replay exit, refusals that survive the toast, and a `ConfirmDialog` in front of kill/rewire/route/clear.

---

## 13. Risks, limitations, and what is not verified

| # | Risk / limit | Impact | What settles it |
|---|---|---|---|
| R1 | The tree is **actively edited by sibling wave-3 lanes** (`http.rs` at 21:52:53, provider client, five sibling reports at 22:05) | All `path:line` anchors can drift; a landed W1/W2 by another lane may duplicate this plan | Re-hash (§14.2) before implementing; coordinate on the W-numbered packages |
| R2 | The live server was **restarted at 22:00:32** during the audit | Probes 1–2 hit the 21:44 binary; probes 3–4 and the Host probe hit the 22:00 binary | Repeat the probes on one instance and record the PID in the evidence |
| R3 | Approvals were **not exercised live** (none pending, and approving would be a paid side effect) | D5's a11y claims are source-verified only; the "dismiss dead" claim is a source-composition proof | A fake-provider fixture run that raises `file_write` approval, then browser assertions T8 |
| R4 | No run was started, so live node events, streaming, and provider calls were not observed | `NODE_*` field claims are source-verified; terminal-state convergence untested here | A single fabricated-fixture run in a test harness (no paid call) |
| R5 | LOD/perf targets (≥50 fps, ≤120 mounted) are `03` §9.4 READ, not measured | The plan may over- or under-fit the real canvas | Synthetic 600-node harness with `mountedNodes`/`fps` capture |
| R6 | The DB is at schema **10**; memory banks/`settings`/`provider_calls`/`watchdogs` do not exist | W5/W6/W7 are design + contract only until migrations land; UI must gate | Migration + route + one round-trip per route |
| R7 | `cost_usd`/`cost_status` are not persisted; `/api/runs/{id}/budget` is absent | The cost meter can only show a cap, never spend, until W5 | `provider_calls` + `runs.cost_*` migration and a settled run |
| R8 | I did not read sibling reports `19`–`27` | Overlap or contradiction with their findings is possible | Read them before approving this plan; this file's evidence stands alone |
| R9 | Board 12's SETTINGS panel and board 09's governance panel were read as images, not as specs | Widths/spacing in W4/W7 are approximate | Use `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md` for exact geometry |

**Explicitly not verified by this seat:** any new component's rendering; any route added by W1–W10; approval/decision behaviour at runtime; provider cost settlement; memory bank behaviour; watchdog behaviour; 150/600-agent performance; screen-reader output (no AT available); contrast of the 8 px labels (they are removed rather than measured).

---

## 14. Evidence appendix

### 14.1 Probes run (all read-only; no file written outside this report)

| Time | Probe | Result quoted in |
|---|---|---|
| 21:53 | `GET /health`, listeners, process identity | §1.3 |
| 21:54 | `GET /api/session`, `/api/history/runs`, `/api/projects`, `/api/agents/{profiles,memory}`, `/api/provider/models`, `/diagnose`, `/api/settings` (404), `/api/runs/{id}/budget` (404), `/api/approvals` (404) | §1.4, §5 D14 |
| 21:55 | `GET /api/history/runs/{id}/usage`, `/messages`, `/state?revision={0,7,99,latest}` | §2.3, §0.4 I1 |
| 21:56 | Served bundle fetch + `args_preview` string search; `web/dist` timestamps | §1.2, §4.2 |
| 21:57–21:59 | Playwright probe 1: first paint inventory, 6 nav destinations, 4 tabs, reduced-motion pass, 1100 px pass | §2.1, §2.2, §2.7 |
| 22:00–22:01 | Playwright probe 2: replay revision click, label capture, surface copy (Settings/Agents/Models), breakpoints 1280/1024/1000/900 | §2.3, §2.4, §2.5, §2.6 |
| 22:03–22:04 | Playwright probe 3: rail LIVE occlusion (actionability log), footer LIVE exit | §2.3, §0.4 I2 |
| 22:05 | Playwright probe 4: tab order, agent headings, full Agents text | §2.1, §2.5 |
| 22:06 | `Host: evil.example` → 421; `Host: localhost:8099` → 200; uptime/PID re-read | §0.4 I3, §1.3 |
| 22:07–22:11 | `git status`, `git show HEAD:…http.rs` (host guard absent in HEAD), file mtimes, SHA-256 prefixes, `control-matrix.json` `_meta` | §1.2, §4.2, §14.2, D26 |

No probe wrote to the repo, the DB, the vault, or the provider. No prompt, approval, kill, cancel, delete, or settings mutation was issued.

**One observation this seat did not cause and cannot attribute:** `/api/provider/balance` returned `USD 3.80` at 21:54 and `USD 3.23` at 22:02 (a 0.57 decrease) with no session of this seat issuing a provider call. The live provider account is therefore being spent down by some other actor (another lane, the launcher, or an earlier run) while this audit reads it; it is recorded here so the figure in §1.4 is not mistaken for an idle-account baseline. This seat's probes were GET-only and no prompt was submitted by this seat.

### 14.2 File hashes read this pass (SHA-256 prefix, drift check)

| File | Prefix | File | Prefix |
|---|---|---|---|
| `web/src/App.tsx` | `0F7CA3156B61` | `web/src/panels/AgentsPanel.tsx` | `9AC74F55E52F` |
| `web/src/flow/Canvas.tsx` | `05D3BC61CCA7` | `web/src/panels/AgentMemoryPanel.tsx` | `3F9F332F54C3` |
| `web/src/flow/graph.ts` | `543B850DE91E` | `web/src/panels/AgentMessagesPanel.tsx` | `FF9C7787CCA2` |
| `web/src/flow/nodes.tsx` | `41A5DA7EDCE8` | `web/src/panels/ModelsPanel.tsx` | `D4AC15D4E124` |
| `web/src/state/store.ts` | `3C222E738E66` | `web/src/panels/SettingsPanel.tsx` | `102C5B6BBCBF` |
| `web/src/state/agentState.ts` | `4CFC0735A6E0` | `web/src/panels/LibraryPanel.tsx` | `B0E99B5E1F07` |
| `web/src/state/approval.ts` | `94311064DD1A` | `web/src/panels/Scrubber.tsx` | `3329167DD0A8` |
| `web/src/state/metrics.ts` | `15FC2EB17290` | `web/src/panels/HistoryPanel.tsx` | `72CB64CEED45` |
| `web/src/chat/ChatCore.tsx` | `0C4E2DD746EA` | `web/src/panels/ToolActivityPanel.tsx` | `11489E87E25C` |
| `web/src/panels/ApprovalOverlay.tsx` | `2E360B16AE71` | `web/src/panels/NavRail.tsx` | `DC1E75970123` |
| `web/src/panels/SwarmPanel.tsx` | `E3D67DD7E860` | `web/src/panels/ControlBar.tsx` | `39B4FF73F92B` |
| `web/src/panels/HudBadge.tsx` | `BE7FD1C62FF6` | `web/src/panels/Header.tsx` | `D18A493F57E3` |
| `web/src/panels/AgentHierarchy.tsx` | `F671771EFEE7` | `web/src/api/contract.ts` | `4E66EB111F8B` |
| `web/src/tokens/tokens.ts` | `60AFFD1871C3` | `backend/…/http.rs` | `F52B83B0AD88` |
| `backend/…/execution.rs` | `8218C81ED167` | `backend/…/snapshot.rs` | `4811CC72A354` |
| `backend/…/graph.rs` | `8B3A24A29E42` | `backend/…/caps.rs` | `D21061F2FA3C` |
| `backend/…/tool_loop.rs` | `8D924F9E969B` | `backend/hiverouter-provider/src/budget.rs` | `0B685A5ED36B` |
| `backend/…/contract/src/event.rs` | `939FDECADBD7` | `web/src/panels/ProjectsPanel.tsx` | (read, not hashed) |

### 14.3 Boards read as images (design authority for this file)

| Board | File | Facts used |
|---|---|---|
| 05 Agent cards | `hivemind-agent-cards.png` | five kinds/one anatomy; Hyper 420×500, Super 420×150, Worker 320×96, Tool 420×240, Error 320×110 `#FFF7F5`; meta mono 9.5; state = dot **or** pill; bee = routing arrow |
| 07 Controls & HUD | `hivemind-controls-and-hud.png` | composer, 40 px targets, tabs, `4 SUPERAGENTS · 12 WORKERS · 98.2% GREEN` (the un-owned metric), scrubber with times, 72 px rail, 260 px library, minimap |
| 08 Hierarchy | `hivemind-hmas-hierarchy.png` | root row `root · 4 superagents · healthy`; per-row `kind · N workers · state`; "re-parent only while not streaming; kill/rewire are confirmation-first" |
| 11 Motion | `hivemind-motion-and-live.png` | ambient 18 s, sheen 6 s, bee flow 2.4 s, pulse 1.6 s, spinner 1.2 s, rise 240 ms, hover 120 ms, focus 160 ms; reduced-motion table (loops off, static glyph) |
| 12 Run detail & files | `hivemind-run-detail-and-files.png` | six stacked panels; tool ledger append-only with outcome chips; failure/refusal ledger with retryable chips; usage "values arrive from RUN_METRICS; none are fabricated" |

### 14.4 Cross-reference to the wave-2 goal

`15-ui-ux-goal.md` remains the design authority for the *target*; this file supersedes it on (a) the snapshot/live split, (b) the now-wired `args_preview`, (c) the project-registry half-fix, (d) the existence of a Playwright harness, (e) `MAX_TOOL_TURNS = 12`, (f) the Host guard having landed. Where they conflict, the measurements in §2 and §5 win.

---

## 15. Done / not done / unverified

### Done

| Item | Evidence |
|---|---|
| Baseline and runtime identity recorded (HEAD, dirty files, live binary/PID/DB schema, bundle) | §1.1–§1.4, quoted JSON and `git` output |
| Live measurement of every reachable surface (counts, labels, HUD, replay, breakpoints, reduced motion, tab order, occlusion) | §2.1–§2.7, quoted probe output |
| Operator-decision coverage matrix (20 rows) with verdicts | §3 |
| Delta against `15`/`08`/`05` including four places where the wave-2 text is stale | §4.1–§4.3 |
| Defect register D1–D26 with class, severity, chain, and required change, each anchored | §5.0–§5.2 |
| State/view-model design (two sources, one projection, total card registry, LOD/density, store slices, memo keys, event contract, contract additions) | §6.1–§6.7 |
| Exact change list as ten work packages with anchors and gates | §7 W1–W10 |
| Removal list (18 rows) | §8 |
| Accessibility and motion rules (16 numbered rules) with testable gates | §9 |
| 20 acceptance tests named by file and assertion | §10 T1–T20 |
| 20 new implementation questions with options and recommendations | §11 |
| Top ten ranked changes | §12 |

### Not done

| Item | Concrete reason |
|---|---|
| Any product edit, build, bundle, migration, or test addition | Seat is read-only by instruction; the deliverable is this file |
| Live approval exercise (approve/deny/dismiss) | Would require a pending approval and a real side effect; no paid run allowed in this seat |
| Starting a run to observe `NODE_*` streaming and terminal convergence | Explicitly out of scope ("no paid runs"); the `NODE_*` claims are source-verified instead |
| Implementing the W1–W10 packages | Design-and-audit seat; see §7 for the seams and §10 for the gates |
| Reading sibling wave-3 reports `19`–`27` | They appeared while this seat was measuring; cross-lane reconciliation is the parent's job (R8) |
| Regenerating `e2e/data/control-matrix.json` | That is a write to a test artifact; W10 specifies the fix |

### Unverified

| Claim or change | Why unverified | What would prove it |
|---|---|---|
| The exact runtime effect of the uncommitted backend edits (execution/tool_loop/provider client) | Not exercised; live binary contains them but no run was started | A fixture run + event capture, or a diff-level test |
| D5's focus-trap/Escape absence at runtime | No approval was on screen; claim is source composition | T8 in a fake-provider harness |
| The occlusion defect at 1280/1024 as well as 1440 | Only 1440 was measured for occlusion (breakpoints measured for layout only) | Repeat the `elementFromPoint` probe at 1280 and 1024 |
| Cost/calls/messages meter availability after any sibling migration | DB is schema 10, routes absent | Re-probe `/api/runs/{id}/budget` after W5 |
| 150–600-agent perf claims | Only 35 nodes measured (72 animations) | Synthetic 600-node harness with mounted-node/fps capture |
| Screen-reader behaviour of the current tree/canvas | No AT available in this environment | VoiceOver/NVDA pass on Workspace + History |
| That the 22:00 binary matches every uncommitted file | Only web bundle and Host guard were cross-checked | Rebuild-and-diff or a version stamp in `/health` |

---

# Replacement seat 24 — verification, corrections, and completion

**Why this section exists.** The brief for the replacement seat said “the previous UI seat produced no
report”. That premise was false by the time the replacement started: the original seat wrote this file at
**22:14:08** (117,673 B) and was stopped before it reported. The replacement did **not** overwrite the
audit; it independently verified it, corrected three claims, added the missing 45-bullet operator matrix,
and recorded the 22:16–22:18 runtime/tree changes. Everything below is additive to §1–§15; where a
correction conflicts with the body above, the correction wins and names the body section.

**Replacement seat method.** `/deep-inspection` is not installed (same substitution as §0.1):
`/deep-investigate` + `/trace`. Runtime was probed read-only at 22:05–22:16; **the server went down at
~22:17** (TCP connect refused at 22:18:02, no `hiverouter`/`HiveMind` process). No restart, no prompt,
no paid call, no credential read, no product edit. The web bundle served was `assets/index-DzPltEYs.js`
(21:56 build, same as §1.3). The replacement read files at the hashes in the header block; SQLite was
opened **read-only** with Node's `node:sqlite` against
`C:\Users\Riley\AppData\Local\HiveMind\hiverouter.db` (+ its WAL).

## 16. Independent verification of the load-bearing claims

**Hash drift since §14.2 (time-scoped).** `execution.rs` changed from `8218C81ED167…` (body read) to
`F02F032A99CAD91E` (22:16, run-scoped snapshot) and `snapshot.rs` from `4811CC72A354…` to
`2954FF374BE2DBC6` (22:16, `capture_scoped`). Both are uncommitted and unbuilt; anchors in §5 that fall
after `execution.rs:1193` shift by +10 in the working tree. The body's line anchors remain valid for the
`bd575ba` code it read; the replacement's anchors are the working-tree ones in the header.

| # | Claim from the body | Independent result | Class |
|---|---|---|---|
| V1 | Replay shows the process graph, not the run's (D1) | **CONFIRMED.** `GET /api/history/runs/01a1b931-941a-7171-a209-1c05777354f8/state?revision=7` → 35 nodes, 7 id-time-buckets; `graph.revision=89`; `history.rs:109-115` → `snapshot.rs:110-119`; `snapshot::capture` snapshots `graph.snapshot()` (`snapshot.rs:258-272`, `graph.rs:220-223`). | VERIFIED |
| V2 | First paint fuses snapshot tasks into live tasks (D2) | **CONFIRMED** in source: `performSnapshotRefresh` sets `tasks: projection.tasks` (`store.ts:1150-1163`); `recordsFromSnapshotGraph` forces `worker` + fallback label (`store.ts:609-626`); no runId on `TaskRecord` (`store.ts:67-85`). Rendered numbers are the original seat's Playwright measurement, not re-run here. | VERIFIED (source) / READ (render) |
| V3 | Identity by first-8 collides (D8/§2.1) | **CONFIRMED** from the live snapshot: every task of one run shares `01a1b93x`; the 35 nodes have 7 distinct first-8 buckets and 9 distinct task ids. First-8 sites: `AgentsPanel.tsx:58`, `HistoryPanel.tsx:26`, `ControlBar.tsx:122`, `AgentMessagesPanel.tsx:102` (both sides). `LibraryPanel.tsx:29` is a **tail-8** chip (unique here, still an opaque fragment, not a collision). | VERIFIED |
| V4 | No agent class/role/label on the wire (D3) | **CONFIRMED.** `NODE_KIND_VOCABULARY` has 4 values (`event.rs:240`); `NodeEvent` fields are task_id/state/rev/kind/parent only (`event.rs:248-272`); producer fills exactly those (`execution.rs:743-779`). | VERIFIED |
| V5 | `args_preview` reaches the screen | **CONFIRMED.** Store maps it (`store.ts:1589`), overlay renders it (`ApprovalOverlay.tsx:262-268`), contract bounds it at 512 UTF-16 (`contract.ts:26`, `:762-769`). Served bundle includes the change (21:56 > 21:39). | VERIFIED |
| V6 | Approvals are session-only / no ledger | **CORRECTED — approvals are durable in SQLite.** See §17.1. | CONTRADICTED (body) |
| V7 | Tool ledger is unrendered after reload (D24) | **CONFIRMED and stronger:** `tool_turns` has **22 rows** for the 3 failed runs (9/7/6), while the UI initialises `toolActivity: []` (`store.ts:1762`) and no route reads the table (`http.rs:513-531`). | VERIFIED |
| V8 | Two of seven meters have no producer (D14) | **CONFIRMED:** `runs` columns are `id, project_id, prompt, status, error_code, created_at, updated_at` (+`tool_turns`,`tool_calls`) — no cost/attempts/ended_at (`db.rs:53-61`); no `provider_calls` table. `/diagnose` exposes caps only. | VERIFIED |
| V9 | Refusal copy is thin (D6) | **QUANTIFIED — 16 of 65 codes.** `strings.ts:312-393` vs `contract.ts:426-492`; fallback copy `refusals.ts:38-45`. | VERIFIED |
| V10 | Reduced motion is exhaustive (good) | **READ** from the original seat's measurement (`getAnimations()=0`); CSS is `index.css:101-119` + `motion.css:399-442`. Not re-measured here. | READ |
| V11 | `E_BUDGET_EXCEEDED` is a lying code | **CONFIRMED from the live DB and code.** Three runs failed with `tool_turns=4` and `tool_calls=5/5/8`; the only measurable one spent 3,268 tokens. `Refusal::BudgetExceeded` renders “exceeded the $0.50 cost cap” (`refusal.rs:152-154`). The 21:31 working-tree change returns `E_CALL_LIMIT` (`execution.rs:2856-2859`) and raises caps (`tool_loop.rs:14-16`) — **unbuilt**. | VERIFIED |
| V12 | `/api/provider/models|balance` were live | **READ** from §1.4 (the original seat called them: USD 3.23, 2 models). Not re-called here (external account metadata; avoid duplicate writes/rate coupling). | READ |

**Probe correction (recorded so no lane inherits a false performance finding).** PowerShell
`Invoke-WebRequest` reported 10.3 s / 30.3 s / 45.5 s for `/api/agents/memory` (twice) and ~12 s for
`/` and `/diagnose`; `curl.exe` reported 3–40 ms for the same endpoints within the same minute. The
latency was **client-side probe overhead, not a product defect**. No “slow panel” finding is made.

## 17. Corrections to §1–§15

### 17.1 CORRECTION — approvals are persisted, and two granted side effects never executed

The body (§D5, §D6, §3 row 18) treats approvals as session-only because `GET /api/approvals` is 404.
Persistence is not the missing piece: the tools crate owns a durable `approvals` table and migrates it
(`backend/crates/hiverouter-tools/src/approval.rs:149-205`), with
`approval_id, run_id, task_id, tool_call_id, tool_name, args_sha256, permission_class, created_at,
expires_at, nonce, decision, decided_at, reason, executed_at` (`:151-166`), a UNIQUE nonce (`:161`),
and a one-pending-per-call partial index (`:190-193`). The **live DB has 2 decided rows**:

```
approval 01a1b930-c864-72c5-8133-b849605ea79a  run 01a1b930-…  task 01a1b930-b226-…  shell_exec
  decision=allow_once  created 03:25:44Z  expires 03:27:44Z  decided 03:25:47Z  executed_at=null
approval 01a1b930-d588-71c0-9d83-c54fd658723a  run 01a1b930-…  same task  shell_exec
  decision=allow_once  created 03:25:47Z  expires 03:27:47Z  decided 03:25:49Z  executed_at=null
```

Consequences for the change set:

1. The run-detail approvals ledger needs **`GET /api/runs/{id}/approvals` over the existing table** —
   no schema migration, no in-memory ledger. The row set gives decision, decided_at, nonce, reason,
   `executed_at`, tool, args hash. The body's W4/W5 plan can drop “session-only” copy for approvals
   (keep it for refusals, which have no table).
2. **Receipt truth:** `executed_at = null` for both granted approvals. “Approve” granted exactly one
   consume and the side effect did **not** run (the run died at the turn cap immediately after). The
   focused dialog must show `granted · not executed` / `executed at …`, not imply the effect happened.
   This is the strongest argument for the consequence line the body already requires.
3. The body's D5 a11y findings (one `aria-modal` per pending approval, no focus trap/Escape/return,
   dismiss only inside the refusal block, 1 s live-region ticker) remain **VERIFIED** by re-reading
   `ApprovalOverlay.tsx:221-370` and `approval.ts:254-267`; they are unchanged.

### 17.2 CORRECTION — D1's root cause is worse: 8 of 35 snapshot nodes belong to runs that do not exist

The body says the snapshot mixes “other runs”. The live DB is sharper: `tasks` has **27 rows across 5
runs** (6/6/6/5/4), `runs` has those same 5 ids, but the run's revision-7 snapshot has **35 nodes: 27
belong to live task rows and 8 are orphans**. `snapshots/` on disk contains **10 run directories**, five
of which (`01a1b86d…`, two `01a1b88b…`, `01a1b88d…`, `01a1b893…`) have **no `runs` row at all**
(`app_meta.bootstrapped_at = 2026-11-08T00:48:36Z` — the DB was recreated while old snapshot files
survived). Boot recovery restores **every snapshot on disk regardless of DB existence**
(`recovery.rs:139-177`; `graph.rs:restore_from_snapshot` replaces the graph), so the old run images are
in the process graph, and the next global capture writes them into the new run's snapshot.

Therefore the complete fix is four parts, not one:

- **R1 (landed in the working tree, unbuilt):** scope capture by the run's task ids — `capture_scoped`
  (`snapshot.rs:255-286`), called from `commit_run_snapshot` (`execution.rs:1189-1216`). Note it filters
  by `tasks_for_run`; a graph node without a task row is dropped (honest, but test the spawn→capture race).
- **R2 (missing):** quarantine snapshot directories whose run is absent from `runs` (or filter them at
  recovery) so a deleted run cannot resurrect.
- **R3 (missing):** filter at the read boundary — `GET /state` and `HISTORY_SNAPSHOT` must serve only
  `tasks_for_run(run_id)` ids. This is what makes **legacy** snapshots honest; R1/R2 do not rewrite
  files already on disk.
- **R4 (missing):** the UI needs a run id per node (wire + snapshot) and must show
  `run source: live | last committed | replay` as a first-class banner (body §6.1 already designs this).

Add an acceptance test: for run `01a1b931…`, revision 7 must return **4** nodes (its task count), zero
`failed` nodes, and zero ids outside `tasks WHERE run_id = 01a1b931…`; a legacy fixture with orphan nodes
must render 0 orphans after R3.

### 17.3 CORRECTION — “runs have no measured usage” is true for 2 of 5, and the spend is visible in `tool_turns`

`run_usage` has rows only for `01a1b92e` (3,268 tokens), `01a1b931-2a4f` (6,894), `01a1b931-941a`
(8,057). The failed runs `01a1b930` (6 tool-turn rows, 2 granted approvals) and `01a1b92f` (7 rows) have
**no usage row**. So the UI's “Not reported” is honest for those two, but the operator cannot tell
“not measured” from “measured and lost”; the run did execute (approvals + tool turns prove work). The
tool ledger row count and the `runs.tool_calls` counter also disagree (6 rows vs 5; 7 vs 5; 9 vs 8), and
`runs.tool_turns` is 4 for all three — **three different “calls” numbers exist** (`runs.tool_calls`,
`tool_turns` rows, provider `CALL_CAP`). The seven-meter design must name its source per meter (§17.4).

### 17.4 NEW FINDING (P0) — the run-level failure banner loses the failure reason

`RUN_STATUS` carries only `status` + `error_code` (`contract.ts:199-203`); `ChatCore` rebuilds copy from
the code with `clientRefusal(runErrorCode)` (`ChatCore.tsx:88-90`, `:211-221`). The client table has no
entry for `E_BUDGET_EXCEEDED`, `E_CALL_LIMIT`, `E_PROVIDER_AUTH`, `E_PROVIDER_RATE_LIMIT`,
`E_PROVIDER_TIMEOUT`, `E_PROVIDER_INCOMPLETE`, `E_PROVIDER_INVALID_RESPONSE`, `E_SPAWN_DEPTH`,
`E_FANOUT_LIMIT`, `E_AGENT_LIMIT`, `E_APPROVAL_TIMEOUT`, `E_APPROVAL_REQUIRED`, `E_TOKEN_BUDGET_EXCEEDED`,
`E_HISTORY_GAP`, `E_FILE_TOO_LARGE`, `E_TEMPLATE_*`, `E_TOOL_*`, `E_PATH_*`, `E_SNAPSHOT_CORRUPT`,
`E_USAGE_INCOMPLETE` (`strings.ts:312-393`), so those runs render **“Unrecognised server refusal (CODE)”**
with a generic how-to-fix (`refusals.ts:38-45`). Meanwhile the provider 402 billing sentence now survives
the HTTP boundary in the client (`backend/crates/hiverouter-provider/src/client.rs:508-516`, working
tree) but has no field to travel in: `runs.error_code` is a code only (`db.rs:53-61`), and the run frame
has no `message`. **Fix:** add `error_message` (bounded) to `RUN_STATUS` + `runs`, render it, and generate
the 65-code copy table with a coverage test (every `KNOWN_REFUSAL_CODES` entry must have copy).
Related: `E_CALL_LIMIT`'s text says “the 40-call cap or the 40-super-step plan ceiling”
(`refusal.rs:157-158`) while the cap that actually fired was the 4-turn loop (now 12, `tool_loop.rs:14`);
server copy also misnames the binding limit — split `E_TURN_LIMIT` from `E_CALL_LIMIT` or generate the
number.

### 17.5 NEW FINDING (P2) — `E_HISTORY_GAP`/`formatStateOf` copy and replay timestamps

The body already flags “Routing state N of M” (D9). Two smaller copy defects remain: `formatStateOf`
returns `Routing state N of M` (`strings.ts:428-430`) and the replay banner prints `history.selected`
(a revision index) as “Routing state 7” (`Canvas.tsx:437-443`), so a revision is still called a routing
state in two places. Board 07/12 want a wall-clock time per revision; `HistoryRevisionList` carries
`created_at` per revision (`http.rs:653-657`) but the UI decodes revisions as numbers only
(`store.ts:520-543`). Add `{revision, created_at, run_status}` to `HistoryState.revisions`.

## 18. The missing matrix: all 45 operator decisions (the “36” is 45 bullets)

`OPERATOR-DECISIONS.md:3` says 36 planning questions; the file contains **45 bullet decisions and no
question ids** (counted: `^- ` = 45). The body's §3 maps 20 grouped rows. The matrix below uses stable
ids `OD-01…OD-45` in bullet order (line numbers from that file) and gives the **UI-only** verdict against
the measured/source app. `MISSING` means no surface; `PARTIAL` means a surface exists but cannot satisfy
the sentence; `LIES` means a rendered claim has no producer; `MET` means the sentence is satisfied today.

| id | line | operator sentence (short) | UI now | Verdict |
|---|---|---|---|---|
| OD-01 | 9-10 | polished local desktop, one operator, Rust localhost, portable Windows | Vite SPA served by the Rust binary; portable exe launcher present; no desktop-shell UX (window/menu/update) | PARTIAL |
| OD-02 | 11-13 | HMARS = Hyper → Super → Agents → Subagents + Remote/Watchdog, A2A, banks, observable UI | 5 kinds, 6 nav; Subagent has no kind/card; Watchdog absent; banks not a surface | MISSING |
| OD-03 | 13-14 | 50–150 active first, 300–600 total, never overclaim | no LOD/culling/virtualization; 35 nodes → 72 animations (measured) | MISSING |
| OD-04 | 15 | hierarchy + memory/A2A first, then provider/scale/UI | UI has no hierarchy tree, no memory bank, no A2A inspector | MISSING |
| OD-05 | 19-20 | autonomy modes 1–4, default 1 | no mode in UI/store/contract/DB | MISSING |
| OD-06 | 21-22 | approval policy as settings; default side effects only; no silent bypass | no policy UI; `spawn_agent` consumes no approval (`execution.rs:1684-1691`) | MISSING |
| OD-07 | 23 | agent capabilities editable, validated/audited, default enabled | import/delete profiles only (`AgentProfilesPanel.tsx:88-127`); no edit/audit/default | PARTIAL |
| OD-08 | 24-26 | no hard default budget; caching goal; soft guard/emergency stop | hard $0.50/400k/40 caps invisible; no meters/stop/soft guard | LIES |
| OD-09 | 27 | retention/visibility/blindness human-only, audited | no controls, no audit table/route | MISSING |
| OD-10 | 31 | every agent incl. tiny Watchdog/Remote has a card | 5 renderers; 35/35 cards `worker`; no Subagent/Remote/Watchdog/Hive/Branch/Cluster card | MISSING |
| OD-11 | 31-33 | Remote/Watchdog simple switch, one job, report to owner | no entity, cadence, report, card, kill path | MISSING |
| OD-12 | 34-36 | default agents code-defined; templates in settings; custom via settings/chat; no marketplace | embedded profiles + import; no settings/templates editor; library has 4 insert buttons | PARTIAL |
| OD-13 | 37-38 | Hyper/Super may choose/spawn built-in persona, subject to settings | spawn role is free-form string; no persona picker/setting; persona lookup by exact/normalized role | PARTIAL |
| OD-14 | 39-40 | default hierarchy + Remote under Hyper/Super | hierarchy is parent links only; no tier/role labels; Remote absent | PARTIAL |
| OD-15 | 41-42 | SuperAgent ownership switchable (dept/domain, branch lead, single task) | no ownership field or UI | MISSING |
| OD-16 | 43-44 | chat is operator UI; conversational SuperAgent deferred | chat is fixed overlay “Hive AI Assistant”, not an agent; no deferral record in UI | PARTIAL |
| OD-17 | 48-49 | workspace read/list/write/shell/network/browser tools | UI shows only file tree/read (`DataPanel.tsx`); no tool catalogue/capability view | PARTIAL |
| OD-18 | 50-51 | HiveWide collision/usage daemon; warn/reject duplicates; CPU/RAM | no load/collision/in-flight surface (`IN_FLIGHT_LIMIT=8` never shown) | MISSING |
| OD-19 | 52 | A2A mute/blind/invisible; tree communication can be disabled | no visibility controls; mailbox is read-only | MISSING |
| OD-20 | 53-54 | any-agent mesh by settings; default safest tree/sibling; topics optional | no settings; mailbox shows only same-run rows; no topology inspector | MISSING |
| OD-21 | 55-56 | cross-hive SuperAgent messaging by settings; default audited bridge | cross-branch sends suspend for approval (`execution.rs:1695-1718`) but no bridge UI/policy | PARTIAL |
| OD-22 | 57 | bounded event/notice bus, optional pub/sub topics, no group chat | no topic/bus surface; events are a transport log, not observable | MISSING |
| OD-23 | 58-59 | hive-crossing/sensitive messages need approval; ordinary chatter not | no sensitivity/hive-boundary indicator in mailbox | PARTIAL |
| OD-24 | 63-66 | memory scopes global/run/hive/branch/role as a Venn cluster | role-only list, 8-row recall, no scope tabs (`AgentMemoryPanel.tsx:86-123`) | MISSING |
| OD-25 | 65-66 | SuperAgents own banks; children tap their tree; cross-branch sharing useful | no bank model | MISSING |
| OD-26 | 67-68 | HyperAgent all-bank read only if leakage prevented; summaries first, raw setting | no read-set/summary/raw setting | MISSING |
| OD-27 | 69-70 | SQLite rows+summaries first; local/external vector future; design all four | UI shows no storage mode or vector status | MISSING |
| OD-28 | 71-72 | thought storage: summaries default; raw/none settings | `reasoning_tokens` shown as “Not reported”; no storage policy | MISSING |
| OD-29 | 72-73 | retention run-lifetime + human-approved project persistence | no retention UI | MISSING |
| OD-30 | 74-75 | memory writes provenance/trust aware; injected content not authoritative | no trust/provenance shown; `context.rs` header says authoritative (body §3) | MISSING |
| OD-31 | 79-80 | A2A topology/memory inspector default; drawer/canvas/chat by settings | mailbox list only; no inspector; no settings | MISSING |
| OD-32 | 81-82 | main view = hierarchy + live execution + memory/A2A edges; all projections on by default | one canvas, one list; no edges for memory/A2A; no projection toggle | MISSING |
| OD-33 | 83-84 | agent cards: role, task, state, cost, tools, branch; density setting | cards show fallback identity + kind + state + `—`; no role/task/cost/tools/branch/density | MISSING |
| OD-34 | 85-86 | extend FigCraft; research option 4 but default extension | boards 05/09/12 partially matched; hierarchy/governance/run-detail not built | PARTIAL |
| OD-35 | 90 | DeepSeek only; no Anthropic key | provider panel shows deepseek only; no Anthropic path | MET |
| OD-36 | 91 | Files default yes only for vision/image, not repo files | Data panel is repo-file read (jail), not provider Files; no provider-file surface | PARTIAL |
| OD-37 | 92-93 | Search & Fetch by approval/budget | no search/fetch tool or cost surface | MISSING |
| OD-38 | 94 | Responses API research; Chat Completions likely canonical | no Responses surface (correct); no decision record in UI | PARTIAL |
| OD-39 | 95 | FIM/Prefix not required now | absent (correct) | MET |
| OD-40 | 96 | Vision important later, text/code first | no vision/Files surface; deferred correctly | PARTIAL |
| OD-41 | 97-98 | token caching first-class: stable prefixes, measured hits, cross-agent reuse, no unmeasured claim | six counters only; no ratio, no prefix contract, no per-agent cache, no claim copy | PARTIAL |
| OD-42 | 102 | per-hive fair admission queue default; modes settings | single active run (`E_RUN_ACTIVE`), no queue/capacity surface | MISSING |
| OD-43 | 103 | bounded retry then branch failure; HyperAgent decides; operator escalation | no retry/failure decision UI; no escalation control | MISSING |
| OD-44 | 104-105 | persist everything incl. approvals/artifacts; ephemeral as setting | runs/messages/usage/approvals/tool_turns persist in SQLite but approvals/tool_turns have **no read route or UI**; no artifact store; no ephemeral setting | PARTIAL |
| OD-45 | 106-107 | 300–600 needs hierarchical planning + real infra | UI cannot show hierarchy depth/tier; planner schema has no `parent` (READ `10` §5) | MISSING |

**Score: 2 MET (OD-35, OD-39), 13 PARTIAL, 29 MISSING, 1 LIES (OD-08).** This is stricter than §3's
20-row grouping because it counts the operator's exact sentences, including the daemon, bus, overlay and
retention lines that a UI seat cannot satisfy alone. It is the honest denominator for “vs all 36 answers”.

## 19. Additions to the work packages and gates (delta to §7–§10)

The body's W1–W10 remain valid. Add these three, and amend two:

| id | New/amended work | Why | Gate |
|---|---|---|---|
| **W11** | **Durable evidence read models over existing tables.** `GET /api/runs/{id}/approvals` (table `approvals`, `approval.rs:151-166`), `GET /api/runs/{id}/tool-turns` (table `tool_turns`, `db.rs:114-141`), `GET /api/runs` already exists. No migration. | Approvals and tool turns are persisted but unrenderable; run detail cannot be honest without them. | `runEvidence.test.ts`: a fixture run with 2 approvals + 9 tool rows renders both after reload; session-only label only on refusals. |
| **W12** | **Legacy snapshot scoping + recovery quarantine.** R2/R3 of §17.2: filter `GET /state` + `HISTORY_SNAPSHOT` by `tasks_for_run`; do not restore snapshot dirs of unknown runs; run id on snapshot nodes (schema v3). | The working-tree `capture_scoped` does not fix files already on disk or the boot recovery path. | `snapshotScope.test.ts` + `snapshot_read.rs`: 35-node legacy fixture returns only the run's 4 nodes; boot with an orphan dir does not grow the graph. |
| **W13** | **Run-failure detail + copy coverage.** `error_message` on `RUN_STATUS`/`runs`; generated 65-code copy table with coverage test; split `E_TURN_LIMIT`/`E_CALL_LIMIT`; render the provider stop reason as provider, not HiveMind. | §17.4: failed runs show “Unrecognised server refusal”; server copy misnames the binding cap. | `refusalCopy.test.ts`: every `KNOWN_REFUSAL_CODES` has message/how_to_fix; `runFailure.test.ts`: a 402 run shows the billing sentence. |
| **W4 amend** | Tool ledger read model is `tool_turns` (W11), not an in-memory slice; keep the live `TOOL_CALL/RESULT` path for the current run and reconcile rows by `(run_id, tool_call_id)` (unique in the table). | Two sources must not double-render; the table is authoritative for history. | `toolLedger.test.ts`: same run live+reload shows N rows, not 2N; `tool_calls` counter vs row count is labelled. |
| **W5 amend** | Seven-meter sources: agents/depth/fanout from `tasks`; tokens from `run_usage`; **calls from persisted attempts (new column) and tool calls from `tool_turns`/`runs.tool_calls` as two labelled meters**; cost from new `cost_usd/cost_status`; messages from `agent_messages` count. | §17.3: three “call” numbers exist today; a meter that mixes them lies. | `budgetMeters.test.ts`: each meter names its source in the DOM; “calls (provider attempts)” and “tool calls” are distinct rows. |

**Additional acceptance tests** (the body's T1–T20 stand; these four are added for the deltas):

| id | File | Assertion |
|---|---|---|
| T21 | `web/tests/replayScope.test.ts` (new) | A legacy 35-node snapshot for a 4-task run renders exactly 4 cards, none `failed`, and a `last committed run · read-only` banner; switching Live clears history tasks. |
| T22 | `web/tests/approvalLedger.test.ts` (new) | After reload, a decided approval renders decision, decided_at, nonce-consumed, and `executed_at` state (`granted · not executed` vs `executed at`); dismiss exists whenever `approvalActionsEnabled().dismiss`; exactly one dialog for N pending. |
| T23 | `web/tests/toolLedgerReload.test.ts` (new) | A run with 9 `tool_turns` rows shows 9 rows after reload from the read model, grouped by branch/turn, with outcome/args hash/result bytes; live + reload does not duplicate. |
| T24 | `web/tests/refusalCopyCoverage.test.ts` (new) | For every `KNOWN_REFUSAL_CODES`, `clientRefusal(code).message` is not the unknown fallback; `E_CALL_LIMIT`/`E_TURN_LIMIT` name the binding cap; provider codes render as provider stops. |

## 20. Replacement seat's top 10 (handoff)

1. **Fix the graph's scope before any surface work** — R1 (in tree) + R2 recovery quarantine + R3 read
   filter + run id on nodes; legacy snapshots are already contaminated (§17.2). This is the difference
   between a monitor and a lie.
2. **Make identity real** — `agent_class`/`role`/`persona_id`/`depth`/`label`/`progress` on the wire
   (D3), and replace the first-8 displays (`AgentsPanel.tsx:58`, `HistoryPanel.tsx:26`,
   `ControlBar.tsx:122`, `AgentMessagesPanel.tsx:102`) plus the tail-8 chip (`LibraryPanel.tsx:29`)
   with `class · role · depth · short-id`.
3. **Ship Run detail as the one evidence destination** with ledger rows from `tool_turns` (22 rows exist
   today, unrendered), approvals from the existing `approvals` table, failure/refusal ledger, usage and
   cost.
4. **Approval inbox + receipt truth** — one focused `alertdialog`, deny-first, args preview + redaction
   legend, from→to, TTL/nonce/consequence, dismiss/retry whenever enabled, decided ledger, and
   `granted · not executed` (live rows prove the distinction).
5. **Seven meters with named sources** — separate provider attempts from tool calls; cost only when
   settled; unknown stays `—`; cap-hit links the named refusal; kill `E_BUDGET_EXCEEDED`'s $0.50 text.
6. **Refusal system** — 65-code copy coverage, `error_message` on run failure, provider stop reasons
   labelled as provider, append-only session refusal ledger (no read table exists).
7. **One projection model** (`agentView.ts`) + total card registry + Hive/Tree toggle + LOD/culling;
   delete the four independent derivations and the label-sorted hierarchy.
8. **Memory/A2A inspectors** — Phase A role-only with an honest “bank tree not available on this build”
   banner and confirmation on clear; mailbox with `messages N/32`, branch grouping, “status not
   reported”; roster from `GET /api/runs/{id}/tasks` (new) before any reply feature.
9. **Settings/autonomy/tutorial only behind a real route** — DB-backed, audited, human-only rows; until
   then label env values `boot-configured — restart required`; no fake selectors.
10. **Remove the slop and restore the motion/type budgets** — Swarm tiles/hero, chat ACTIONS table,
    duplicate tool table under Agents, dead HyperAgent card, `CHAT_PIN`/`diffOpen`, Routing destination,
    global library tabs, local-UUID project chrome, 7.5–10 px labels, and all but one ambient loop.

**Operational handoff.** The server died at ~22:17 and was **not restarted** by this seat. Any live
verification of the above needs the parent's restarted-server gate (`cargo build --release` then the
portable launcher), a fresh `GET /health` capture, and a re-hash of the files in the header block — the
tree moved at 22:16 (`execution.rs`, `snapshot.rs`) and may move again. The report file is
`docs/research/hmars/24-ui-implementation.md`; this replacement wrote no other file and made no product
edit.

## 21. Replacement-seat gate record

```json
{
  "gate": "EVIDENCE-GATE-v1",
  "lane": "wave3-seat-24-ui-implementation (replacement)",
  "decision": "ACCEPT-WITH-UNVERIFIED",
  "unsupported_verified": 0,
  "uncited_numbers": 0,
  "stale_claims": 3,
  "contradictions_open": [],
  "blocked": ["live re-measurement after 22:17 (server down, restart forbidden)"],
  "corrections_to_body": ["approvals persisted (D5)", "D1 root cause + orphan runs", "usage row count per run"],
  "files_written": ["docs/research/hmars/24-ui-implementation.md"],
  "at": "2026-11-08T04:25:00Z"
}
```

