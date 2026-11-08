# 05 — FigCraft / UI gap audit (post-rebuild)

**File:** `docs/trace/provider/05-figcraft-ux-gaps.md`
**Status:** AUDITED — live build measured; **P0 = 0, P1 = 5, P2 = 11**
**Date:** 2026-11-07, 18:17–18:25 CST (America/Chicago)
**Claim boundary:** no pixel-parity claim. Board coordinates below are read from the exported PNGs and are **ESTIMATED**; live geometry is `getBoundingClientRect()` and is **MEASURED**. Severity is a product judgement, not a measurement.

---

## 0. Runtime snapshot and method

| Item | Value | Class |
|---|---|---|
| Live URL | `http://127.0.0.1:8099` — `GET /health` 200, `status:ok`, `version:1.2.0-dev`, `schema_version:10` | MEASURED |
| Server | `C:\Users\Riley\Desktop\HiveMind-Portable\hiverouter.exe --bind 127.0.0.1:8099 --data-dir C:\Users\Riley\AppData\Local\HiveMind`; PID 11920 at first probe, replaced by a new process during the audit (`uptime_secs: 11` at the first post-restart probe, ≈18:18) | MEASURED |
| Build under test | **`/assets/index-B_KOpz15.js`**, 756,490 B, mtime **2026-11-07T18:12:00-06:00**, SHA-256 `6F4F2D346A40D62811810EDD7B3EFE4EF6E3DBD5B894E33A51706A9E4D99B614`; CSS `index-Br9J9AfU.css`, 59,534 B | MEASURED |
| Source freshness | Latest `web/src` edits: `ChatCore.tsx` 18:09:29, `LibraryPanel.tsx` 18:09:08, `store.ts` 18:07:31 — **all older than the 18:12 build**, so this run includes the latest working-tree UI changes (tool-activity block, short agent labels, provider account) | MEASURED (mtimes) |
| Build churn during audit | Served bundle changed `index-3BVimmOv.js` (17:57 build) → `index-B_KOpz15.js` (18:12 build) when the portable server was restarted at ≈18:18. The 17:5x captures in `docs/trace/figma-parity/` are **stale** for chat/library/models | MEASURED |
| Keyed state | `DEEPSEEK_API_KEY present`; 1 committed run `01a1b86d-8b15…` (status `completed`), 2 chat messages (`Reply with exactly HIVE_UI_OK` → `HIVE_UI_OK`), 4 committed revisions, `run_usage` 88/5/93 tokens, **`tool_turns = 0`, `tool_calls = 0`, approvals = 0, agent_messages = 0** | MEASURED (SQLite read-only + `/health`) |
| Provider routes | `/api/provider/models` → 2 models (`deepseek-flash`, `deepseek-v4-pro`, 1048576 ctx, 393216 max out); `/api/provider/balance` → `USD 9.50` (live panel later read 9.17/9.19 as the account spent) | MEASURED |
| Browser / method | existing headless Chrome 156, CDP `:9222`; new target per pass, `Page.navigate`, fonts ready, viewport emulation, DOM read, `Page.captureScreenshot`; no app restart, no source edit | MEASURED |

Screenshots and raw DOM dumps: [`shots-figcraft/`](shots-figcraft/) — `workspace-1440x900.png`, `models-1440x900.png`, `models-1024x800.png`, `swarm-1440x900.png`, `history-1440x900.png`, `agents-1440x900.png`, `history-1280x900.png`, `history-1024x800.png`, `measure.json`, `measure-tabs.json`.

> Note: `workspace-1280x900.png` / `workspace-1024x800.png` from the second pass actually rendered the History tab; they were renamed `history-1280x900.png` / `history-1024x800.png` before this report so they are not cited as workspace evidence.

---

## 1. Provider / model / balance / usage panel

### 1.1 What is live (MEASURED, 1440×900)

[`models-1440x900.png`](shots-figcraft/models-1440x900.png) and `measure-tabs.json` show four stacked blocks in a 760 px column starting at x≈513:

1. `PROVIDER deepseek` / `MODEL deepseek-flash` (2-row definition list).
2. `PROMPT TOKENS 88 · COMPLETION 5 · TOTAL 93 · CACHE HIT/MISS 0 / 88 · REASONING Not reported` — **five** rows, one label column (140 px).
3. `PROVIDER ACCOUNT` — two columns: `BALANCE USD 9.17` left, `AVAILABLE MODELS` right with one card per model (`deepseek-flash`, `DEEPSEEK-V4.1-FLASH · 1048576 CTX · 393216 MAX OUT`).
4. `CONTRACT REFERENCE (FACT-REGISTER): DEEPSEEK-FLASH; DEEPSEEK-V4-PRO IS OPTIONAL.`

Both provider routes are real and the numbers are the server's; nothing is fabricated. That is correct and should not be changed.

### 1.2 Gaps against board 12 (MODEL STATUS & USAGE)

| Board 12 says | Live | Class |
|---|---|---|
| One `MODEL STATUS & USAGE` card: provider, model, **key status**, **server health** | Provider + model only; key status and server health are absent from Models (they exist in Settings) | MEASURED |
| `USAGE / SIX COUNTERS` as a 2-column grid (prompt, completion / total, cache hit / cache miss, reasoning) | 5-row single-column list; reasoning is the only "not reported" value | MEASURED |
| Copy: “values arrive from RUN_METRICS; none are fabricated here.” | Same intent is honoured | MEASURED |
| `TOOL ACTIVITY` panel belongs to run detail | Tool activity is on the Agents surface, not near usage | MEASURED |
| Nothing about balance, model catalogue, context window, or an account surface | Two new atoms with no board: **money row** and **model option row** | ESTIMATED (coverage) |

Visible layout defects in the account block:

- The first model card is ~68 px tall, the second ~59 px: `… · 393216 MAX OUT` wraps onto a second line inside an 8 px meta line, so the two rows in the same list disagree in height ([`models-1440x900.png`](shots-figcraft/models-1440x900.png)). **MEASURED (screenshot pixels)**.
- Hierarchy is inverted: the machine `id` is the primary line, the human `name` is the micro-meta line. Board 12's model row is a single labelled value. **ESTIMATED** (design intent).
- Balance has no "as of" time and is fetched exactly once on mount (`useEffect(…, [])`, [`ModelsPanel.tsx:52-82`](../../../web/src/panels/ModelsPanel.tsx)), so it is stale after a spent run until remount — observed values 9.50 → 9.19 → 9.17 across reloads. **MEASURED**.
- `Promise.all` means a balance failure blanks the model catalogue too, and the error text is the raw `provider metadata HTTP 200/500` string. **MEASURED** (code path), unexercised live.
- `is_available` from `/api/provider/balance` is dropped; an unavailable account renders "No balance rows reported." **MEASURED** (code).

### 1.3 Is a new FigCraft board justified?

**Yes — a component sheet, not a new full artboard.** Recommendation: `13 · Provider & account`, 1440×760, four specimens:

1. usage six-counter grid (lift board 12's layout verbatim, 2 columns);
2. balance row: label · currency · amount · `as of <time>` · stale/unknown state;
3. model option row: human name primary, id secondary, `ctx`/`max out` as one meta line with **no wrap** at 1440;
4. provider-unavailable state: key missing (`E_KEY_MISSING`), balance call failed, catalogue call failed — each independently, because they are independent routes.

Why not a full artboard: the surface is a settings page with no spatial behaviour, and board 12 already owns the run-scoped half. Why not "no board": the two new atoms are money and model-selection primitives; without a spec the next change re-invents them, and the current wrap defect is exactly what a board would have pinned. **ESTIMATED** (design judgement).

---

## 2. Chat: text → tool call → result → approval

Live chat is 400×480 at (676, 204) — the board 11 *app variant* size, 16 px lower than board 07's ≈188 top. It contains: header (bee + title + `×`), message list, composer with a single submit icon, hint line. No mic, no attachment, no header progress ring (all three are on boards 05/06/07/11). **MEASURED (geometry/DOM), ESTIMATED (board reading)**.

| Step | Representation today | Board evidence | Gap | Severity |
|---|---|---|---|---|
| **Text** | Streaming assistant rows with cursor + typing dots + `WorkingBee`; `Streaming` status line; code blocks via `MessageBody` | 07 chat; 11 chat app variant (400×480) | none material; the empty state adds starter chips + hex mark + `READY · NO RUN YET` that no board shows (app addition) | P2 (addition) |
| **Tool call** | Only in **Agents → Tool activity** (`TOOL_CALL` → `pending` row). The chat's new `ACTIONS` block ([`ChatCore.tsx:273-307`](../../../web/src/chat/ChatCore.tsx)) is in the **current bundle** but has never rendered: the only run has `tool_turns = 0`, and the block is hidden when the list is empty | 12 `TOOL ACTIVITY` (table + empty-state copy "No tool calls yet — submit a prompt to create the first run.") | no in-chat call/result row; the board's table lives on a different surface; chat shows only "Streaming" while tools run | **P1** |
| **Result** | `TOOL_RESULT` only carries `outcome`, `code`, `bytes`, `sha256` ([`contract.ts:304-313`](../../../web/src/api/contract.ts)); the store keeps `outcome` + `code` ([`store.ts:1525-1563`](../../../web/src/state/store.ts)) and **drops `bytes`/`sha256`**. `ToolActivityRecord.text` exists but is never written, so the chat's `entry.text` span and the panel's text line are dead branches | 12 columns `TOOL · OUTCOME · HASH / LEDGER`, legend `SUCCEEDED / FAILED / REFUSED` | result evidence is limited to an outcome word + args hash; no result hash/bytes, no legend, no column alignment | **P1** |
| **Approval** | Global blocking modal ([`ApprovalOverlay.tsx:243-359`](../../../web/src/panels/ApprovalOverlay.tsx)): tool, `args_sha256`, run/task/tool-call/approval ids, expiry countdown, Deny/Approve. No from→to branch, no message subject, no "single-use grant / TTL 120 s / approve executes no effect by itself" line | 09 `MESSAGE APPROVAL`: `Backend Architecture → Frontend Architecture`, *schema.prisma v3 ready to merge*, `APPROVAL_RESOLVE · nonce · single-use grant · TTL 120 s`, "Approve executes no effect by itself" | the operator cannot tell **who is asking whom, or what for**, without decoding UUIDs; the board's consequence copy is missing | **P1** |

The new chat `ACTIONS` block, even when populated, shows the **last 4** entries sorted by `turn` then `toolCallId` with no turn grouping, no result body, and no approval state — a `pending` tool call and a tool call awaiting an approval are visually identical. It also duplicates the Agents panel's list. **MEASURED (code), UNVERIFIED (render)**. Decide one home for tool activity (board 12 says run detail) before adding a second.

Board references: [agent-cards (05)](../../ui/figcraft/ASSETS/hivemind-agent-cards.png) for chat/card anatomy, [motion-and-live (11)](../../ui/figcraft/ASSETS/hivemind-motion-and-live.png) for the app-variant chat, [hmas-governance (09)](../../ui/figcraft/ASSETS/hivemind-hmas-governance.png) for approval, [run-detail-and-files (12)](../../ui/figcraft/ASSETS/hivemind-run-detail-and-files.png) for tool activity and model status.

---

## 3. Swarm / history / projects

### 3.1 Swarm (1440×900, [`swarm-1440x900.png`](shots-figcraft/swarm-1440x900.png))

Live structure: title `Swarm` + the board-08 intro sentence (verbatim, good), a **5-tile stat row** (`SUB-HIVES 0 · WORKERS 2 · TOOLS 0 · ERRORS 0 · ACTIVE 0`), `HIERARCHY / WORKSPACE-JAILED` panel with a repeated `HIERARCHY` heading and two one-line rows, a `SURFACES` prose panel with `OPEN AGENTS` / `OPEN FILES` and 2 task cards, then `PROJECTS`. **MEASURED (DOM text + screenshot)**.

| Board 08 shows | Live | Severity |
|---|---|---|
| Rows with name, `SUPERAGENT · 3 workers · running` meta, state dot and progress | one line: state word + label + kind; no counts, no progress, no per-row meta; depth via 14 px indent + `◆`/`└` glyphs ([`AgentHierarchy.tsx:55-80`](../../../web/src/panels/AgentHierarchy.tsx)) | **P1** |
| `TOPOLOGY / LIVE EDGES` panel (honeycomb, bee-faceted dashed edges) | absent | **P1** |
| `STATES / SHAPE + TEXT CUE` legend (`colour is never the only channel`) | absent | P2 |
| `Spawn sub-hive` (filled) and `Kill worker` (crimson) inside the surface | absent — `ControlBar` mounts only on `workspace` + `workspace` ([`App.tsx:121-134`](../../../web/src/App.tsx)), so the swarm view has no actions | **P1** |
| HyperAgent root row with `root · 4 superagents · healthy` | no root row concept; both tasks are top-level rows | P2 |
| Nothing like a stat-tile dashboard | 5 display-serif numeral tiles, duplicating the HUD census | P2 (AI-slop signal, §4) |

### 3.2 History (1440×900, [`history-1440x900.png`](shots-figcraft/history-1440x900.png))

Live: 300 px revision rail (`rev 0`…`rev 3`, `4 REVISIONS`, `LIVE` button) + canvas + scrubber `ROUTING STATE … 4 OF 4 … LIVE`. **MEASURED**. There is no board for the revision list; the only history component on the boards is board 06's `TIME-TRAVEL SCRUBBER` (`09:41:07 · routing state 12 of 38`, six tick dots, handle). Live omits the wall-clock time and the ticks, and shows `N OF N` instead of a routing-state phrase — a deliberate simplification but undocumented. **ESTIMATED (board reading)** → P2.

The rail is honest (read-only, LATEST badge, explicit LIVE exit) and the 4 revisions are real (`snapshots` table). No fake data. **MEASURED**.

### 3.3 Projects

Live row: `01A1B86D-6F38-7377-AB83-1A2A1F5B147E` + `COMMITTED` across the full width ([`ProjectsPanel.tsx:60-77`](../../../web/src/panels/ProjectsPanel.tsx)). Board 09 shows `HiveMind AI · active`, a second `sandbox · idle` row, and the footnote `project_id · localhost only · PDF:256,275-282`. **MEASURED (live), ESTIMATED (board)**. A raw UUID as the display name is a machine-output-as-copy signal → P2. The project is never named anywhere the operator can read.

### 3.4 Tab scope (cross-cutting)

The library tab row is painted as a global selector but only affects the workspace ([`App.tsx:89-102`](../../../web/src/App.tsx) checks `nav` first). `models-1440x900.png` shows **HISTORY highlighted while the Models screen is displayed**; the earlier `final-swarm.png` and `final-history.png` are byte-different captures that render the *same* Agents panel with different tab highlights. **MEASURED**. Either scope the tabs visually to Workspace or make the tab win over nav. P2 (confusing), P1 if treated as a lying control (11-control-wiring claims these are `WIRED-AND-WORKS`).

---

## 4. Motion / live-feel and AI-slop signals

### 4.1 Motion

| Check | Live (1440 workspace) | Board 11 spec | Class |
|---|---|---|---|
| Running animations | **12** entries in `document.getAnimations()` | “one calm movement at a time” (repo comment, [`motion.css:9-10`](../../../web/src/styles/motion.css)) | MEASURED |
| Ambient/glow | 16 s (`motion-honeycomb-travel`) | 18 s ambient travel (PROPOSED) | MEASURED / ESTIMATED |
| Sheen | 12 s linear | **6 s** ease-in-out, 5 lobes | MEASURED / ESTIMATED |
| Bee flow | 1.4 s dash + 13/16/18 s ambient bees | 2.4 s `cubic-bezier(.2,.8,.2,1)`; static arrowheads only | MEASURED / ESTIMATED |
| Working-bee spinner | 2.2 s linear | **1.2 s** linear | MEASURED / ESTIMATED |
| Running pulse | 1.8 s ease-in-out | 1.6 s ease-out | MEASURED / ESTIMATED |
| Reduced motion | CSS media + `[data-reduced-motion]` present; earlier QA measured 0 animations | loops off, ≤120 ms opacity | MEASURED (source), UNVERIFIED (this build) |
| Live feel during a run | Not observable in this build: the only run is `completed`; no streaming/tool frames exist in the DB | board 11 chats/streams | UNVERIFIED |

**Gap:** three independent ambient loops (glow, sheen, scan) plus three free-flying bees plus a hidden pulse — all decorative — against a board that specifies one ambient travel and static arrowheads. The new build runs **more** loops than the 09 audit measured (7 → 12). P2 craft/perf, still the largest “live-feel” defect: motion volume substitutes for event-driven feedback (no card state transitions visible at 55% zoom, no progress sheen because every `progress` is null).

### 4.2 New AI-slop signals (not in 09)

| Signal | Evidence | Verdict |
|---|---|---|
| Dashboard stat-tile row | `SwarmPanel.tsx:38-51` — 5 tiles, 26 px display numerals, `0/2/0/0/0` | not on board 08; duplicates the HUD; dashboard-generic |
| Hero empty panel + 3 numbered onboarding steps | `SwarmPanel.tsx:53-107` — “The hive is idle”, radial-gradient backdrop, `STEP 01/02/03` cards, two CTAs | pattern-slop; boards use a one-line honest empty state |
| Card-stack monotony | Models = 4 identical `surface-panel` blocks with `mt-3`; Agents = hierarchy + grid + activity + profiles + memory + messages, same recipe | settings/admin page with no grouping hierarchy vs board 09's 2×3 operator grid |
| Asserted machine kind | every task renders `WorkerAgent …` although `kind` defaults to `'worker'` when the server never sent one (`store.ts:605-620`) | **fake precision** — the UI states a fact it does not know (P1, §5) |
| UUID/machine output as copy | Projects row, Agent ids, approval ids | machine output promoted to primary copy |
| Identical truncated labels | library chips both read `#01A1B…`; Target select has two identical `WorkerAgent 01a1b86d` options | a list you cannot disambiguate |
| 8 px micro-labels | 14 occurrences of `text-[8px]`/`[8.5px]`/`[9px]` across panels/chat; board 05 pins meta mono **9.5** | below the design's own floor; still P2 |
| Text glyphs as icons | `◆`, `└`, `▲`, `×` | glyph-vs-line-icon inconsistency (carried from 09) |

No emoji, no neon, no generic purple blobs, no fabricated metrics, no stock imagery — those remain clean. **MEASURED**.

---

## 5. Top seven fixes (seam + board evidence)

Ordered by risk to the operator, then design drift. Effort is **ESTIMATED**; everything else is marked.

| # | Sev | Fix | Exact UI seam | Board evidence | Class |
|---|---|---|---|---|---|
| 1 | **P1** | Make task identity unique and stop asserting a kind the server never sent. Use role/depth + a distinguishing id suffix (or a server label), and render an explicit “Task / kind unknown” instead of `WorkerAgent`. Widen the 48 px chip. | [`store.ts:327-330`](../../../web/src/state/store.ts) (`fallbackTaskLabel`), [`api/ids.ts:40-42`](../../../web/src/api/ids.ts) (`shortId` = first 8 chars), [`store.ts:605-620`](../../../web/src/state/store.ts) (`kind:'worker'`), [`LibraryPanel.tsx:24-30`](../../../web/src/panels/LibraryPanel.tsx), chips `w-12` at `LibraryPanel.tsx:104-129`; also feeds the `TARGET` select in [`ControlBar.tsx:43-56`](../../../web/src/panels/ControlBar.tsx) | board 05 chips are named `Nova / Vector / Spark`; board 08 rows read `Backend Architecture` / `tailwind-builder` | **MEASURED**: both chips `#01A1B…`; both select options `WorkerAgent 01a1b86d`; DB roles are `planner` + `worker`. Root cause: first 8 hex of a UUIDv7 are the top 32 bits of the ms timestamp (a ~65 s bucket), so **all tasks in one run collide**; the 48 px chip truncates to 6 characters and collides again |
| 2 | **P1** | Decide the tool/result home. If run-detail (board 12) is authoritative, move the activity table there and keep the chat to text; if chat is authoritative, give each call a result row with turn grouping, and surface approval state in that row. | [`ChatCore.tsx:273-307`](../../../web/src/chat/ChatCore.tsx), [`ToolActivityPanel.tsx:137-181`](../../../web/src/panels/ToolActivityPanel.tsx), [`store.ts:1493-1564`](../../../web/src/state/store.ts) | board 12 `TOOL ACTIVITY` (columns + legend + “no fabricated calls”) | **MEASURED**: `tool_turns=0`; block present in bundle, never rendered; `entry.text` unreachable |
| 3 | **P1** | Carry result evidence through the store and render it: keep `bytes`/`sha256` from `TOOL_RESULT` (and only `text` if the contract ever adds it), show `args → result` hash + bytes, and add the outcome legend. | [`contract.ts:304-313`](../../../web/src/api/contract.ts), [`store.ts:110-118`](../../../web/src/state/store.ts), [`store.ts:1525-1563`](../../../web/src/state/store.ts), [`ToolActivityPanel.tsx:145-181`](../../../web/src/panels/ToolActivityPanel.tsx) | board 12 `HASH / LEDGER`, `SUCCEEDED / FAILED / REFUSED` chips | **MEASURED** (fields exist upstream, dropped in store); unexercised |
| 4 | **P1** | Give the approval modal a branch narrative and the board's consequence copy: `from → to`, message kind/subject, `args_sha256`, nonce single-use + TTL line, “Approve grants the engine one consume.” | [`ApprovalOverlay.tsx:243-285`](../../../web/src/panels/ApprovalOverlay.tsx), actions `334-357` | board 09 `MESSAGE APPROVAL` | **MEASURED** (rendered fields); approval unexercised live |
| 5 | **P1** | Close the swarm surface against board 08: per-row meta (`kind · workers · state`), progress, HyperAgent root, live-edges/topology panel, state legend, and the two actions (or a link that opens the workspace control bar). Drop the 5 stat tiles. | [`SwarmPanel.tsx:30-51`](../../../web/src/panels/SwarmPanel.tsx), `109-146`, [`AgentHierarchy.tsx:55-80`](../../../web/src/panels/AgentHierarchy.tsx), [`App.tsx:121-134`](../../../web/src/App.tsx) | board 08 (hierarchy + topology + legend + Spawn/Kill) | **MEASURED** (DOM/screenshot); appearance scored ESTIMATED |
| 6 | **P2** | Align the provider/usage page to board 12 and spec the two new atoms: six-counter 2-column grid, key status + server health, balance with as-of/stale state, model row that cannot wrap, independent error states. Then add board `13 · Provider & account` (component sheet). | [`ModelsPanel.tsx:136-212`](../../../web/src/panels/ModelsPanel.tsx), fetch `52-82` | board 12 `MODEL STATUS & USAGE`; boards have no balance/catalogue atom | **MEASURED** (screenshot + text); board decision ESTIMATED |
| 7 | **P2** | Fix the two lie-shaped affordances: (a) scope the library tabs to Workspace or make tab override nav so `HISTORY` never highlights on Models; (b) give the minimap node rectangles + viewport outline (it is now textured but empty) and stop letting it vanish at 1024 without an alternative overview. | [`LibraryPanel.tsx:54-96`](../../../web/src/panels/LibraryPanel.tsx) + [`App.tsx:89-102`](../../../web/src/App.tsx); [`Canvas.tsx:407-433`](../../../web/src/flow/Canvas.tsx) | board 06 TABS (one row) and board 07 minimap (nodes + viewport) | **MEASURED**: `HISTORY` selected on Models; minimap `220×150` with no nodes; `0×0` at 1024 |

Runner-up (P2, cheap): composer mic/attachment + header progress ring on boards 05/06/07/11 (`ChatCore.tsx:183-208`, `318-348`) and the 2×2 tab grid vs board 06's single row (`LibraryPanel.tsx:54-57`).

---

## 6. P0 / P1 / P2 register

**P0 — 0.** Page loads, `CONNECTED`, renders and answers every surface reached; no fabricated data was found, and no enabled mutation path was observed to be dead. Console/network instrumentation was **not** repeated on this build, so “no console errors” is not claimed here (see §8).

**P1 — 5**

1. Task identity is non-unique and the kind is asserted (`WorkerAgent`) from a default — the `TARGET` select that drives `KILL WORKER`/`RE-ROUTE` shows two indistinguishable options.
2. Chat has no live tool-call/result representation; the implemented `ACTIONS` block has never rendered (0 tool turns in the only run).
3. Tool result evidence is dropped in the store (`bytes`/`sha256`); board 12's hash/ledger and legend are absent.
4. Approval modal has no from→to/subject/consequence; ids only.
5. Swarm surface omits board 08's topology, legend and actions, and adds an off-board stat dashboard.

**P2 — 11**

1. Models/usage layout diverges from board 12 (5 rows vs six-counter grid; no key status/health); model rows wrap to unequal heights; id/name hierarchy inverted; balance stale after a run.
2. No board covers balance/model-catalogue atoms (component sheet recommended).
3. Library tabs are workspace-scoped but painted globally; stale highlight on Models/Agents.
4. Minimap textured but has no node content; hidden (`0×0`) below 1280.
5. Composer lacks mic/attachment; chat header lacks board 11's progress ring.
6. Library tabs wrap to 2×2; board 06 shows one row; board 07 has no tabs in the library at all.
7. Motion: 12 concurrent animations, durations drift from board 11 (sheen 12 s vs 6 s, spinner 2.2 s vs 1.2 s, bee 1.4 s vs 2.4 s).
8. 8 px micro-labels vs board 05's 9.5 px meta floor.
9. Projects/agents/approvals show UUIDs as primary copy; project has no human name.
10. Swarm empty state = hero + 3 numbered steps (pattern-slop); stat tiles duplicate the HUD.
11. Scrubber drops board 06's wall-clock time and tick marks.

---

## 7. Reconciliation with 09 / 10 / 11

These earlier findings are **now fixed or changed** in the 18:12 build — do not re-fix them from the older docs:

| Earlier claim | Now (MEASURED) |
|---|---|
| 09 P1: chat at y=236, too low, 20 px above the bar at 1024 | chat `(676,204,400,480)`, bottom 684, control bar top 804 → **120 px** gap; 1024 chat `(468,154)` with a 712×101 wrapped bar |
| 09 P1: 30 px action buttons, all outline | action buttons are `h-10` (40 px), Spawn is filled honey; but **19/33** visible buttons are still < 40 px (nav/chips/zoom) |
| 09 P1: minimap blank white (inline `background` reset) | dot-grid texture restored (`radial-gradient` computed); **still no node rectangles**; hidden at 1024 |
| 09 P2: minimap occludes chat at 1280/1024 | removed: minimap moved to right inset 40 (x=1180 at 1440), and `0×0` at 1024 |
| 10 WEAK: Models renders only a subset, no refresh on REPLY_DONE | all six usage fields read and re-fetched on `runStatus`; provider account added |
| 11: “Swarm tab … WIRED-AND-WORKS” | true only while `nav === 'workspace'`; inert (highlight-only) on Agents/Models |
| 11: “Projects … read-only” | true; but renders the UUID as the name |

Note the board-size conflict is unchanged: board 07 draws library 240 / minimap 220×150; board 06 captions 260 / 260×170. Live uses the board-07 drawings (tokens 240 / 220×150). **UNVERIFIED** which caption is authoritative.

---

## 8. UNVERIFIED

| Item | Why | What would prove it |
|---|---|---|
| Streaming text, typing dots, working-bee spin, card/message entrance | only run is `completed`; no live frames | one keyed prompt on the current build, capture during `streaming` |
| Tool-call/result/approval rendering (chat `ACTIONS`, tool activity rows, modal) | `tool_turns=0`, `approvals=0`; the bound tools may not produce calls | a run that invokes a tool and one that requests approval |
| Approval TTL/expiry and deny/approve transitions | no approval rows | same |
| History replay of a *populated* graph and the LIVE exit | revisions exist (4) but replay was not exercised | click a revision, capture, exit |
| Motion/perf at 100 % zoom and reduced-motion on this build | captured at fit zoom (55 %); reduced motion not re-sampled | frame trace + `prefers-reduced-motion` capture |
| Console/network cleanliness for this build | not instrumented in this pass (09 covered the previous build) | `Log`/`Runtime`/`Network` listeners on load |
| Board caption authority (240 vs 260 library; 220×150 vs 260×170 minimap) | two boards disagree | a single spec source |
| Hover/focus/active and enabled Spawn/Re-route/Cancel/Kill | controls disabled by run state | a running-run capture |

---

## 9. Guardrails observed

- No product source, FigCraft file, design asset, or git state was modified. Only this report, `shots-figcraft/*.png`, and the two raw JSON dumps were written.
- The `:8099` server was not started, stopped, or signalled by this audit; it was restarted externally at ≈18:18 and every claim above is tied to the 18:12 build.
- Read-only SQLite access (`mode=ro`) and `GET`-only HTTP; no prompt was submitted, so no new run, tool call, or approval exists.
- No pixel-parity claim is made; board dimensions and positions are read from PNG exports and marked ESTIMATED wherever they are used as a target.
