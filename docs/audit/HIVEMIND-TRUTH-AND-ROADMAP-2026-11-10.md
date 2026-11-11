# HiveMind — Truth, capability boundary, and roadmap (2026-11-10)

**Audit HEAD:** `1129349`; **final docs HEAD:** `44e0535` (audit corpus committed) · **Installed product build:** `806251b` (`1.2.0-dev`, schema 24)
**Working tree (final):** the diagnostic `GET /api/evidence/ledger` candidate was reviewed and **discarded** after the audit; it had no UI consumer and no route-contract test. The tree is now clean apart from this audit's docs. The installed build remains `806251b`.
**Installed health:** `degraded` / `E_KEY_MISSING`; **no provider key configured**. UI, templates, settings, reads and operator surfaces work; every live prompt refuses by name.
**Verdict:** HiveMind is a real, bounded, local, approval-gated HMAS prototype. It is **not** a governor in the continuous sense, not a learning system in the weight/skill sense, not a vector-memory product, not HiveWire, not AGI, not a Gödel machine, and not a hands-off autonomous coding employee.

This is the master report. It links the six independent seats and the UI/brand work; all detailed lists are in the linked files and were checked by a separate watchdog (section 15).

---

## 1. What this is, in one truthful paragraph

HiveMind is a Windows-local Rust/Axum + React control room for one provider. It accepts a prompt, makes one planning call, builds a DAG of up to 40 nodes at depth ≤3, runs two bounded root-control turns (creation tools only in turn 1), dispatches SuperAgent → Agent → Subagent tasks under hard caps, requires operator approval for file writes, shell execution and web fetches, settles unfinished tasks, makes **one read-only post-dispatch review** and a tool-less final synthesis, and persists the run in SQLite with event and snapshot evidence. The root can record proposals and convene a five-seat evolution vote; a lesson can be applied only after authorization and a held-out receipt gate. It cannot modify its own code, learn weights, run continuously, act as a network satellite, or keep working while you walk away.

---

## 2. Evidence identity and the dirty-tree caveat

| Item | Value | Source |
|---|---|---|
| Git HEAD | `44e0535` docs/audit commit on top of `1129349` evidence/QA commit and product `806251b` | `git log` |
| Installed build | `806251b`, `dirty:false`, server SHA `d9d76902…2c78`, package SHA `B4BAC18D…11061` | `docs/evidence/live/install-1.2.0-dev.json` |
| Installed code vs HEAD | Product code is code-identical; HEAD adds two evidence/QA commits (`crash-drill`, `scale-drill`) | `git log 806251b..1129349` |
| Diagnostic candidate | `/api/evidence/ledger` summary was uncommitted during the audit; **discarded** because it had no consumer/route test. Not installed. | `git diff` then reverted |
| Offline battery | **PASS** 8/8 at `1129349`; backend PASS, web typecheck/tests/build/docs/QA/health PASS | `docs/evidence/live/full-battery-20261110-234515.json` (docs tree dirty during run) |
| Caveat | Battery receipt says `working_tree_dirty:true`; it proves the dirty tree, not a clean released HEAD | receipt |
| Crash drill | process kill → restart, boot_id changed, schema 24, data preserved, both processes exited | `docs/evidence/live/crash-drill-20261110-230629.json` |
| Scale drill | 41-task serial and bounded-parallel probes pass; **not a live 40-node provider run** | `docs/evidence/live/scale-drill-20261110-230653.json` |
| Live provider | **not tested**; no key; `E_KEY_MISSING` | `/health`, install receipt |
| Strict release | **no 1.2 tag/release seal**; `-Release` remains PRE-RELEASE exit 3 | `docs/RELEASE-EVIDENCE.md` |

**Do not present the dirty-tree battery as a clean release proof.** The clean product build is `806251b`; the current tree has an optional diagnostic change.

---

## 3. Roles and hierarchy: what each “agent” really is

Evidence: `execution.rs:250-310` (spawn edges/fallback), `tool.rs:462-518` (allowlists), `caps.rs:7-18`.

| Role | Actual | Can | Cannot / not proven |
|---|---|---|---|
| **HyperAgent** | Root planner task, depth 0; two pre-dispatch control turns; one read-only post-dispatch review; final synthesis | Plan, create one extra SuperAgent branch only if planner supplied none, create remotes/daemons, read memory summary, propose improvement, convene vote, review results, write final answer | Continuous supervision; mid-flight intervention; send instructions/cancel/repair after dispatch; self-modify; approve arbitrary lower-level requests |
| **SuperAgent** | Depth-1 branch lead, `super`, hive bank owner | Spawn Agents; aggregate children; ask parent; same-parent A2A; read/list; request detached; web fetch; vote evolution | Write/shell in the effective persona surface (tier∩persona can hide them); create remotes/daemons directly; spawn Subagents directly |
| **Agent (Worker)** | Depth-2 task executor, branch bank | Spawn Subagents; approval-gated write/shell; read/list; A2A; request detached; web fetch | Create remotes/daemons; act as root; escape caps/jail |
| **Subagent** | Depth-3 temporary leaf | Read/list, ask parent, report, finish | Spawn, write, shell, network, detached admission |
| **Remote** | In-process fixture principal + device + grant | Appear in the remote lane; be actuated through a grant | Run a model, join the tree, use the internet, act as real network satellite |
| **Daemon** | Enrolled tree-free record; watchdog has a DB-check job | Be enrolled; report a DB check when ticked | Continuous OS scheduler / live tick proof |
| **ToolAgent** | Census tier only | Nothing in production | No shipped producer; label only |

**Caps:** depth 3, fan-out 8, 64 agents/run, 64 provider calls/run, 40 plan nodes, 12 tool turns, 40 tool calls/run, 8 approval waits/run, 64 KiB tool result, $0.50/run, 400k combined tokens/run (`caps.rs`, `tool_loop.rs`). Root control: 2 turns, ≤4 calls each. All refusals are named, not silent clamps.

**“SuperAgents/Agents/trees can do more for less”** is not proven. They are a single-run hierarchy with a shared provider and caps; there is no measured cost/quality advantage over a single agent. The 41-task test proves the harness can enumerate tasks, not that the hive is more efficient.

---

## 4. Learning loop, memory, vector search, HiveWire

### 4.1 Learning loop — lesson artifact, not behavioral learning

Evidence: `execution.rs:5070-5160` (votes/seat), `http.rs:2827-2965` (apply/rollback), `evolution_gate.rs`, `execution.rs:89-128` (boot loader).

```text
root propose_improvement → run-bank row applied:false
four designated SuperAgent votes + operator authorization
apply requires held-out receipt (EVO-GATE) → append ONE JSONL lesson
boot loads lessons as bounded labelled data into planner/root/task prompts
```

What changes: prompt text only. What does **not** change: code, schema, tool grants, prompts as files, weights, skill library, gates, or running agents. There is no automatic promotion, no cross-run evaluation execution, no reward loop, no population/archive. A well-formed hand-authored receipt can pass the shape gate; `head_commit` is not compared to HEAD, artifact hashes are not bound to files, and the evaluator is not run.

### 4.2 Memory — SQLite banks, recency, no vector

Evidence: `db.rs:397-436`; `store.rs:2217-2286`.

- Tables: `memory_banks`, `memory_entries`, `memory_journal`, `memory_context_snapshots`, legacy `agent_memory`.
- Banks: HyperAgent → run bank; SuperAgent → hive bank; Agent/Subagent → branch bank; fixed scope set (run + own branch + parent branch + nearest ancestor hive).
- Retrieval: recency `ORDER BY created_at DESC LIMIT`. Trust filter admits `unverified` entries. Siblings/foreign runs not addressable.
- **No vector database, no embeddings, no ANN/HNSW, no semantic/cosine recall; no agent has it.** Repo-wide search found zero memory/embedding implementation.
- Memory is durable text/context, not similarity search. A vector store would be a new subsystem, not a setting.

### 4.3 HiveWire

**HiveWire does not exist in this repository.** Zero case-insensitive `HiveWire` matches in code/docs/web/e2e/plans. It cannot be used as an agent harness today. Do not confuse it with:
- `ws.rs`: UI WebSocket/event transport;
- `remote.rs`: in-process fixture principal;
- `daemon.rs`: enrolled record, no live scheduler.

If “HiveWire” is a separate product/spec, it must be supplied before it can be audited or integrated.

---

## 5. Is it AGI? A Gödel machine? A meta-HyperAgent?

**No to all three.** Evidence: README claim register; `evolution_gate.rs`; no weight updates; no self-rewriting program; no variant archive; no continuous goal loop.

| Axis | Status |
|---|---|
| General intelligence / held-out breadth | Not demonstrated; one prompt → one bounded run |
| Continual learning | Lesson prompt injection only; no weights/skills |
| Self-model / open-ended goals | No |
| Gödel machine (self-referential proof search/self-rewrite) | No |
| Meta-HyperAgent (improves the procedure that produced it) | No; lessons are operator-gated data |
| Safety envelope | Human approvals, jail, caps; shell is still arbitrary code, not OS sandbox |
| Scale | Caps permit 40 nodes; live 40-node provider run not done |

It is a **narrow, capped, single-run HMAS**. That is a legitimate engineering category; it is not a lesser name for AGI.

---

## 6. What HiveMind can do — 100 use cases

The complete, numbered, evidence-grounded list is in
[`round-95/06-use-cases.md`](round-95/06-use-cases.md) — **100 use cases**, each labelled SAFE or APPROVAL-GATED, with key/run conditions. The companion “cannot” list is in the same file (§2, 10×10).

Best 10 things usable **today with no key** (read/operator surface):
1. `GET /health` / `GET /diagnose` liveness and secrets-free checks.
2. `GET /api/runtime` effective caps.
3. `GET /api/templates` eight templates and insertability.
4. `GET /api/session` provider identity without secrets.
5. `GET /api/approvals` pending approval surface.
6. `GET /api/memory/journal/verify` hash-chain check.
7. `GET /api/agents/runs/{id}/roster` hierarchy/roster.
8. `GET /api/history/runs` / revisions / snapshots.
9. `tools.list` / `tools.describe` / `capabilities.census` CLI.
10. `db.backup` (operator-tier, approval-gated).

With a key, the model-dependent uses are the 12 registry tools under approval/caps; they are not hands-off coding. The full list makes the boundary explicit.

---

## 7. What HiveMind cannot do — 100 items

Two independent lists exist:
- [`round-95/05-operations.md`](round-95/05-operations.md) §4 — 100 install/runtime/operability cannots.
- [`round-95/06-use-cases.md`](round-95/06-use-cases.md) §2 — 100 capability cannots.

Top categories: no autonomous/self-modifying coding; no vector/semantic memory; no HiveWire; no monitor actions/repair; no continuous daemon/network remotes; no live 40-node proof; no clean release/1.2 seal; no clean-profile install proof; no keyless model run; no AGI/Gödel.

---

## 8. Tool surface — what it can and cannot use

**12 registry tools, 5 permission classes, 35 tier grants** (`tool.rs:115-140,462-518`). Effective surface is **tier ∩ persona**, so some grants are unreachable.

| Tier | Registered allowlist |
|---|---|
| HyperAgent | spawn_agent, report_result, finish (plus separate root-control vocabulary) |
| SuperAgent | spawn_agent, ask_parent, send_to_sibling, report_result, finish, file_read, file_list, file_write, shell_exec, request_detached, web_fetch, vote_evolution |
| WorkerAgent | same minus vote_evolution |
| Subagent | ask_parent, report_result, finish, file_read, file_list |
| ToolAgent | file_read, file_list, file_write, shell_exec — **no producer** |

Root-control vocabulary (not registry): spawn_superagent, assign_task, spawn_remote_agent, spawn_daemon, read_memory_summary, propose_improvement, convene_evolution_vote, finish.

Hard limits: approvals for Mutate/Execute/Network; 120s one-shot TTL; `AllowSession` non-expiring; shell allowlist **git.exe + cargo.exe only**, interpreters refused, **not a sandbox**; file read/write 1 MiB jail-contained; `web_fetch` HTTPS-only, no redirects/private nets, 1 MiB. Tools it **cannot** use: no browser automation, no unrestricted HTTP, no SMTP/DB/cloud APIs, no arbitrary language runtimes, no network remotes, no vector search, no code self-patch.

---

## 9. If someone installs this today

Full step-by-step, failure modes and 100 cannots: [`round-95/05-operations.md`](round-95/05-operations.md). Short version:

1. They must **build first** (Node/npm web build, Rust release + bundled w64devkit, csc.exe launcher) and then run `tools/install-desktop.ps1`. There is no downloader, MSI, code signing, auto-update or Add/Remove entry.
2. Install is per-user: `%LOCALAPPDATA%\Programs\HiveMind\app`, data `%LOCALAPPDATA%\HiveMind`, workspace `…\workspace`, Desktop/Start Menu shortcuts.
3. First launch picks a port 8099–8199 and opens Edge/Chrome app window. Server boots SQLite/schema 24, recovers interrupted runs, serves UI.
4. **Without a key**, the UI and read surfaces work; `/health` is `degraded`; every prompt refuses `E_KEY_MISSING`. The product does not fake a run. This is the current state.
5. With a key, runs are bounded and approval-gated; shell is arbitrary code under approval; no continuous autonomy.
6. Known install risks: port-8099 probe mismatch, keyless default, stale test DB/WAL residue, disabled key material on disk, no clean-profile receipt.

---

## 10. Does it need rebuild/recommit right now?

**For the requested report: no.** The installed product (`806251b`) is code-identical to HEAD `1129349`; HEAD adds only evidence/QA commits. Rebuilding now would stamp the uncommitted `http.rs` change (`dirty:true`) and add **zero user-facing capability**. The uncommitted `/api/evidence/ledger` endpoint is a diagnostic API, not referenced by the UI.

**If you want that endpoint:** review it, add a route test/consumer, commit it, re-run the battery, then rebuild/install. It changes audit readback, not core capability. It was discarded from the final tree; the code remains available in the audit history/session if a later change needs it.

**If you want “more capabilities”:** rebuilds do not add them. The missing capabilities are code/subsystem work: monitor actions, active-run crash recovery, vector memory, HiveWire, live 40-node proof, clean release. See ranking.

---

## 11. DeepSeek V4.1 Flash 1017 vs Claude Fable 5.2

Both names are **real, currently documented vendor models** (retrieved 2026-11-10):

- **DeepSeek V4.1 Flash 1017**: API `deepseek-flash`, released 2026-10-17, 1M context, **384K max output**, tools/JSON/Responses/Anthropic API/vision, 2500 concurrency; list price **$0.15/$0.30 input** (off/peak cache-miss) and **$0.60/$1.20 output**; open-weights link. [Release](https://api-docs.deepseek.com/news/news261017), [pricing](https://api-docs.deepseek.com/quick_start/pricing/).
- **Claude Fable 5.2**: API `claude-fable-5-2`, released 2026-11-09, 1M context, **128K max output**, $8/$40 per MTok; fast mode $16/$80 up to 2.5× output speed. [Announcement](https://www.anthropic.com/claude-fable-and-mythos-5-2), [release notes](https://platform.claude.com/docs/en/release-notes/overview), [models](https://platform.claude.com/docs/en/models/overview).

**Token-price ratio (arithmetic, not per-task):** ~26–53× cheaper input and ~33–67× cheaper output vs Fable standard; fast mode widens it. **Quality/speed/bug-fix parity is UNVERIFIED**: no independent cross-vendor same-task evaluation; benchmark tables are vendor-run. Anthropic itself recommends Opus 5.5 as the daily coding driver, Fable for the hard tail.

**For HiveMind:** the installed route is DeepSeek/flash; **no Anthropic adapter exists**. Using Fable would require a provider integration (key custody, stream/tool-call handling, refusal/cost mapping), not a config toggle. Do not promise “Fable quality at DeepSeek price.” Run a blind 30–50-task acceptance study and judge `$ / successful task`, rework cycles and wall-clock. Full evidence: [`round-95/04-models.md`](round-95/04-models.md).

---

## 12. UI, brand, and design

- Brand system: [`docs/design/HIVEMIND-BRAND-SYSTEM-v1.md`](../design/HIVEMIND-BRAND-SYSTEM-v1.md).
- Application specs: [`docs/design/HIVEMIND-DESIGN-SPECS-v1.md`](../design/HIVEMIND-DESIGN-SPECS-v1.md).
- Screenshots: [1440×900](../evidence/live/install-ui-1440x900.png), [375×812](../evidence/live/install-ui-375x812.png).

Verified UI issues:
1. Header says `CONNECTED` and splash says “The hive is awake” based on socket only, while provider key is missing (`Header.tsx:114-126`, `HiveMindSplash.tsx:28-33`, install receipt).
2. At 375px the 240px Library is hidden (`LibraryPanel.tsx:84`), so the 8-template gallery is unreachable; 72px rail + clipped wordmark; tabs wrap.
3. Gallery has 8 rows/4 insertable; disabled `CATALOG` reason is title-only (keyboard/touch cannot discover).
4. Header search popover lacks click-outside/focus management (`Header.tsx:136-176`); NavRail comment says six destinations but renders seven.
5. Splash progress 18/46/100 is a connection bucket, no skip; reduced-motion supported.
6. Main contrast pairs pass AA; amber is AA-large only; light-only.

The brand direction (cream/honey/forest/ink, HoneyGlass, Playfair/Inter, hex mark) is retained; the new specs are **proposed acceptance rules**, not a claim that every rule is implemented.

---

## 13. Plan v3 status with strikethrough

Full crosswalk: [`docs/plans/HIVEMIND-FINISH-PLAN-v3-STATUS.md`](../plans/HIVEMIND-FINISH-PLAN-v3-STATUS.md). Detailed evidence: [`round-95/02-plan.md`](round-95/02-plan.md).

**Proven / struck:** v24 schema; task instruction + root objective schemas; `cmd.exe` removed and interpreters refused; data-dir/ledger jail deny; daemon env sanitization; approval TTL/binding/single suspension/two-write test; EVO-GATE apply+boot; proposal evidence fields; 8-template gallery; installer/build identity; full offline battery; docs truth; crash/restart and 41-task scale receipts; read-only monitor review.

**Open:** db-path/+wal/+shm jail; monitor **actions**/repair; 35→28 grant cleanup; atomic provider-call accounting; history 1970; e2e fail-open; planner fan-out pre-call; handshake/build hygiene; API contract/CLI token; HivePulse/ToolsMatrix/FirstRunNotice/MissionGallery; clean-profile install; active-run crash; live provider; 40-node live; strict release; independent verifier.

**Plan line caveat:** plan L52–53 “one-shot controller” is stale after `806251b` (two control turns + review), but it is **not** a governor; replace wording, do not strike the limitation.

---

## 14. Priority ranking (what to do next)

From [`round-95/02-plan.md`](round-95/02-plan.md) §7, summarized:

### Fastest high-quality wins (0.5–1 day each)
1. e2e guard **fail-closed** (missing/stale results must fail).
2. History timestamp **refusal** instead of 1970.
3. Handshake dead-API removal + `build.rs` extraction.
4. Planner fan-out ≤8 validation **before** the paid planning call.
5. Docs `INDEX.md`/`DOWNLOADS.md` and claim-scope corrections.

### Next (1–2 days)
6. CLI launch-token source + `docs/contracts/HTTP-API-v1.md`.
7. db_path/+wal/+shm jail closure with 4 constructor updates and tests.
8. Evidence-ledger endpoint only if it gets a route test + consumer.

### Largest (3–5 days)
9. **Monitor actions/repair**: `send_instruction`, `cancel_task`, one deduped repair branch; new engine state + scripted provider tests.
10. UI-2 polish (HivePulse/ToolsMatrix/FirstRunNotice) and MissionGallery.
11. Clean-profile install, active-run crash, keyed live 40-node run, strict release.

**Easiest → hardest:** e2e guard → history → handshake/docs → planner fan-out → CLI/contract → db-path jail → evidence ledger → monitor actions → UI-2 → MissionGallery.
**Impact order:** monitor actions → history truth → e2e guard → db-path jail → CLI token → planner refusal → UI polish.
**Dependency order:** truth gates → containment → planner cap → API contract → monitor actions → ledger → UI → release closeout.

---

## 15. Watchdog verification and Lead reconciliation

Independent watchdog: [`round-95/07-watchdog.md`](round-95/07-watchdog.md)
(12,756 B / 157 lines, read-only, no product edits).

**Mechanical result:** `06-use-cases.md` = exactly **100 use cases + 100 cannots**; `05-operations.md` = exactly **100 cannots**; all sections present, non-empty, fences balanced, endings complete. Fourteen code claims were spot-checked; caps, approval classes, two-turn root control, read-only monitor, EVO-GATE shape-only, no vector memory, no HiveWire, and installed-vs-HEAD identity passed.

**Watchdog final verdict at time of audit: FAIL (conditional), not a capability failure.** The failure was a **stale-tree contradiction**: `02-plan.md`, `05-operations.md` and `06-use-cases.md` described the `/api/evidence/ledger` candidate as still uncommitted; the candidate had been discarded and the tree was clean for product code. The watchdog also flagged 01/02/05 for citing older battery receipts, and 06’s “perform today” headline for including key-blocked items.

**Lead reconciliation applied after the watchdog wrote its report:**
1. 02/05/06 now carry a prominent post-audit correction and their current-tree claims were amended; 06’s cannot #70 now says no ledger route exists; 05’s ledger cannots say the same.
2. 02’s dirty-tree and “unshipped change” lines were corrected to **discarded candidate**.
3. 05’s baseline, G3/G4 and evidence-gap lines were corrected; the latest dirty battery is named, not an older clean one.
4. 01’s older battery citation remains historical; the canonical current receipt is `full-battery-20261110-232942.json` (PASS, dirty tree).
5. 06’s 100/100 counts were preserved; key-blocked items are explicitly labelled, so “100 supported surfaces” is the honest reading, not “100 things usable without a key”.

**Final trust status:** the corpus is mechanically complete and code-claim-verified where spot-checked; it is **conditionally trusted** for technical claims, **not** a release proof, and **not** a runtime proof. The watchdog’s FAIL is retained as history; this reconciliation is the audit trail, not a rewrite of the watchdog.

---

## 16. Claim register (master)

| Claim | Grade | Evidence |
|---|---|---|
| 12 tools/5 classes/35 grants | P/READ | `tool.rs` |
| v24 schema installed | P | DB/health/install receipt |
| approval single suspension + two writes | P | suspension test, commits |
| EVO-GATE apply + boot receipt | P | `evolution_gate.rs`, tests |
| monitor is review-only | P | code + test + receipt |
| no vector memory | P/READ | schema/code search |
| no HiveWire | P/READ | zero matches |
| no automatic learning | P/READ | code + docs |
| not AGI/Gödel | P/READ | capability boundary |
| 806251b installed; 1129349 HEAD | P | BUILD.json/git |
| offline battery PASS dirty | P | receipt |
| no live provider proof | P | health/install receipt |
| 40-node live not proven | P | scale receipt note |
| DeepSeek/Fable real + prices | P | vendor pages |
| quality parity | U | no independent eval |
| clean release seal | U/FALSE | release evidence |

**Unverified:** live provider behavior, installed launcher-window screenshot, active-run crash recovery, 40-node provider run, quality parity, EVO evaluator truth, clean-profile install, strict release.

---

## 17. Files produced by this audit

- [01-core.md](round-95/01-core.md) — roles, learning, memory, HiveWire, AGI.
- [02-plan.md](round-95/02-plan.md) — plan crosswalk, bugs, tests, pseudocode, ranking.
- [04-models.md](round-95/04-models.md) — DeepSeek/Fable primary-source comparison.
- [05-operations.md](round-95/05-operations.md) — install/run truth + 100 cannots.
- [06-use-cases.md](round-95/06-use-cases.md) — 100 use cases + 100 cannots.
- [07-watchdog.md](round-95/07-watchdog.md) — independent verification.
- [HIVEMIND-FINISH-PLAN-v3-STATUS.md](../plans/HIVEMIND-FINISH-PLAN-v3-STATUS.md) — strikethrough status.
- [HIVEMIND-BRAND-SYSTEM-v1.md](../design/HIVEMIND-BRAND-SYSTEM-v1.md), [HIVEMIND-DESIGN-SPECS-v1.md](../design/HIVEMIND-DESIGN-SPECS-v1.md).
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — root architecture addendum.
