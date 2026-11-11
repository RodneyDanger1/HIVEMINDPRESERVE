# HiveMind v1 — Exhaustive Codebase Census

**Artifact class:** census / cartography (read-only evidence; no source, test, config, or doc was edited by this work).
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**HEAD:** `6a4125070186da1bbe3f088b3634f3f6d5402fab` (`6a41250`, branch `master`, `git describe` = `6a41250`).
**Version on disk:** `1.2.0-dev` (working tree) against a sealed `1.1.0` release record (`tools/release-manifest.json:6,49-50`).
**Companion artifacts:** `docs/analysis/CODEBASE-CENSUS.json` (machine index, 1,573 entries), `docs/analysis/CODEBASE-CENSUS.mmd` (subsystem diagrams).
**Generated:** 2026-11-10 (America/Chicago), from `git ls-files` plus on-disk ignored/generated trees.

> **Evidence rule.** Every count in this report is derived by a command that ran in this session against this tree, and the derivation is named. Statements copied from a lane report are attributed to that report. No test was executed; no server was started; no provider was called; no key was read or printed.

---

## 0. Verdict

**`COMPLETE`.**

The two verdict conditions are met:

| Condition | Result | Evidence |
|---|---|---|
| Every tracked path is indexed | **1,573 / 1,573** | `git ls-files` = 1,573 (Section 1); JSON `index` array = 1,573 entries; 0 paths with a fallback subsystem or an `unclassified:*` feature tag |
| Every product / UI / API / tool / table feature has an owner or an explicit `MISSING` row | **216 / 216** | 63 HTTP method+path routes, 12 tools, 28 tables, 17 commands, 24 events, 8 personas, 36 panels, 28 UI surfaces, all with an owner file+lane; the 3 unimplemented `DIFF_*` command names carry explicit `MISSING` rows (Section 12) |

**What `COMPLETE` does not mean.** It does not mean the seven lane reports cover the tree: **only 410 of the 1,573 paths (26.1%) are named literally by any lane report**; 863 are covered only by a directory wildcard of depth ≥ 2, 275 only by a root-level wildcard (`docs/**`, `plans/**`, `audit/**`, `runtime-test/**`, `backend/**`), and **25 paths are absent from all seven reports** (Section 15). The census is complete; the lane-report coverage is not. This distinction is carried as data in `cross_reference` and `omissions` in the JSON.

---

## 1. Method and reproducibility

Commands and reads that produced this report:

1. `git ls-files` → 1,573 paths (the index universe).
2. `git rev-parse HEAD`, `git rev-parse --abbrev-ref HEAD`, `git describe --always --tags --dirty`, `git status --porcelain` → repo identity and dirty set (`?? docs/analysis/` only).
3. `Get-ChildItem` over `.gitignore`d and `.git/info/exclude`d trees (`web/dist`, `backend/target`, `tools/w64devkit`, `web/node_modules`, `e2e/node_modules`, `.gitnexus`, `.claude`, `dist/HiveMind*`) → generated-asset inventory (Section 14).
4. Regex extraction over the cited sources for routes (`backend/crates/hiverouter-core/src/http.rs`), commands/events (`backend/crates/hiverouter-contract/src/{command,event}.rs`), refusals (`refusal.rs`), tools (`backend/crates/hiverouter-tools/src/tool.rs`), tables/migrations (`backend/crates/hiverouter-core/src/db.rs`), personas (`backend/crates/hiverouter-core/assets/prompts/personas.json`), panels (`web/src/panels/*.tsx`), surfaces (`web/src/App.tsx:32-33,109-163`), tests (all tracked `.rs`, `web/tests/*.ts`, `e2e/specs/*.spec.ts`).
5. Lane-report cross-reference: the seven files under `docs/analysis/lanes/` were parsed for backticked path tokens, normalised (`\` → `/`, line suffixes stripped), matched exactly, by unique path suffix, by directory prefix, and by declared wildcard.

**Classification rules.** Each path receives exactly one `area` (12 values), one `subsystem` (91 values), one owner `lane` (L1–L7), one `kind` (11 values), one `feature` tag, and a `feature_kind` (8 values). The rule table is deterministic: backend by crate and module, web by directory, docs by family, plans/audit/runtime-test/dist by subtree. The JSON records `bytes` and `lines` (for text files under 3 MB) per path.

**Lane model.** Lanes are keyed to the seven report filenames: L1 runtime-governance, L2 tools-security, L3 state-memory, L4 api-provider, L5 ui-ux, L6 tests-release-install, L7 docs-claims. `L7-docs-claims.md` uses a **different internal lane numbering** (its §6 maps L1=contract/core, L2=provider, L3=tools/safety, L4=execution/governance/plans, L5=web/UI, L6=e2e/release/tooling, L7=docs). This census never uses L7's internal codes for the index lane field; the conflict is recorded in `method.lane_numbering_note` in the JSON.

**Limits, stated as limits.** 922 documentation files were classified by family, not read line-by-line. 117 `.rs`, 106 `.ts`, 72 `.tsx`, 19 `.mjs`, and 86 `.py` files were classified by path and spot-read only where a feature mapping required a line. Binary artifacts (11 `.exe`, 13 `.db`, 3 `.zip`, 13 `.pyc`) were measured, not opened. Test counts are **declared** test functions/sites read from source; no test was executed.

---

**Companion diagram.** `docs/analysis/CODEBASE-CENSUS.mmd` is one valid Mermaid flowchart whose 13 subgraphs are the required subsystem diagrams: repository topology, frozen contract, core runtime barrel, HTTP/WS transport, persistence, provider adapter, tool registry/safety, memory/A2A, evolution gate, web UI navigation/surfaces, build-install-release, index-only file families, and generated/untracked trees. It is subsystem-level by design: the literal per-file mapping is Appendix A and the JSON `index`. The diagram's `INDEXONLY` and `NODIAG` nodes name the 1,137 documentation/evidence paths that are in the index but intentionally have no individual node.

---

## 2. Totals at a glance

| Metric | Count |
|---|---:|
| Tracked paths (`git ls-files`) | **1,573** |
| Indexed paths (JSON `index`) | **1,573** |
| Unclassified paths | **0** |
| Distinct subsystems | **96** |
| Distinct file extensions | **34** |
| Code files (`.rs` + `.ts` + `.tsx` + `.mjs` + `.py` + `.ps1` + `.cs` + `.c` + `.cmd`) | **416** |
| Rust source + test files | **117** (57,294 lines; 57,292 in the 4 crates + 2 outside) |
| Web source files (`web/src`) | **78** (18,955 lines) |
| Web test files | **32** (31 with tests, 4,572 lines) |
| Product-surface features enumerated | **216** |
| Explicit `MISSING` feature rows | **3** (`DIFF_PROPOSE`, `DIFF_APPLY`, `DIFF_REJECT`) |
| Paths named by at least one lane report | **410** |
| Paths absent from all lane reports | **25** |

---

## 3. Code-language census (derived from `git ls-files`)

| Extension | Files | Language / role |
|---|---:|---|
| `.md` | 363 | Markdown — docs, plans, audits, reports |
| `.png` | 271 | PNG — screenshots, parity baselines, diffs |
| `.json` | 233 | JSON — contracts, evidence receipts, manifests, catalogs |
| `.rs` | 117 | Rust — backend (4 crates) |
| `.ts` | 106 | TypeScript — web API/state/tests, e2e specs/helpers, tools |
| `.py` | 86 | Python — audit extraction/QA scripts |
| `.tsx` | 72 | TypeScript + JSX — React surfaces |
| `.svg` | 55 | SVG — design/figcraft exports and served boards |
| `.figcraft` | 42 | FigCraft document (binary design export) |
| `.err` | 25 | Captured stderr logs |
| `.out` | 24 | Captured stdout logs |
| `.txt` | 20 | Text — checksums, notes, package readmes |
| `.mjs` | 19 | JavaScript ESM — e2e live harness, QA gates, Node CLI |
| `.csv` | 14 | CSV — claims, quality matrices |
| `.db-wal` | 13 | SQLite write-ahead log (test fixtures) |
| `.db` | 13 | SQLite database (test fixtures) |
| `.pyc` | 13 | Python bytecode (committed cache) |
| `.db-shm` | 13 | SQLite shared memory (test fixtures) |
| `.mmd` | 12 | Mermaid diagram sources |
| `.exe` | 11 | Windows PE binaries (CLI, launcher, probes) |
| `.ps1` | 10 | PowerShell — build, release, launcher, QA |
| `.prompty` | 8 | Prompty persona templates |
| `.toml` | 7 | Cargo / rust toolchain / cargo config |
| `.css` | 6 | Stylesheets (tokens, motion, splash, index) |
| `.cmd` | 3 | Windows batch launchers |
| `.log` | 3 | Live-run logs |
| `.zip` | 3 | Release archives |
| `.c` | 2 | C probe source + object |
| `.html` | 2 | `web/index.html` + captured `root.html` |
| `.gitignore` | 2 | Git ignore files |
| `.sha256` | 2 | Release hash sidecars |
| `.lock` | 1 | `backend/Cargo.lock` |
| `.jsonl` | 1 | Lesson ledger (`LESSONS.jsonl`) |
| `.cs` | 1 | C# desktop launcher |

Sum = **1,573**. By `kind`: doc 434, evidence 420, asset 295, source 194, test 92, data 59, artifact 31, config 22, binary 13, diagram 12, schema 1.

Rust volume by crate (tracked `.rs`, line counts via `Get-Content`):

| Crate | `.rs` files | Lines |
|---|---:|---:|
| `hiverouter-contract` | 14 | 3,949 |
| `hiverouter-core` | 59 | 37,635 |
| `hiverouter-provider` | 17 | 6,761 |
| `hiverouter-tools` | 25 | 8,947 |
| **Total** | **115** | **57,292** |

(The table counts only the 115 `.rs` files inside `backend/crates`; the remaining 2 are `docs/audit/evidence/toolchain/hello.rs` and `tools/cc-test/r.rs`, bringing the tracked total to 117 files / 57,294 lines.)

---

## 4. Area census

| Area | Files | Notes |
|---|---:|---|
| `docs` | 922 | 58.6% of the tree; mostly evidence PNG/JSON + design exports + prior audits |
| `web` | 137 | 78 `src`, 32 `tests`, 22 public figcraft SVGs, 5 build files |
| `backend` | 132 | 4 crates + workspace manifests |
| `plans` | 114 | seats 49, waves 23, planners 17, foresight 17, verification 5, root 3 |
| `audit` | 98 | parts 43, root 29, diagrams 18, inventory 8 |
| `runtime-test` | 82 | packages 31, db fixtures 27, logs 22, workspace 2 |
| `e2e` | 39 | specs 16, helpers 8, live harness 7, config 7, data 1 |
| `tools` | 34 | qa 13, launcher 7, release 4, cc-test 4, binaries 2, desktop-launcher 3, cli 1 |
| `root` | 8 | `README.md`, `CHANGE-LEDGER-IMPL.md`, 3 launchers, `.gitignore`, `rust-toolchain.toml`, `.cargo/config.toml` |
| `dist` | 4 | 1.0.0 and 1.1.0 zips + sha256 sidecars |
| `ledgers` | 2 | `ui-watchdog.md` / `.json` |
| `skills` | 1 | `skills/hivemind/SKILL.md` |

Sum = **1,573**.

---

## 5. Subsystem census (96 subsystems; top 30)

| Subsystem | Files | Subsystem | Files |
|---|---:|---|---:|
| `docs/design-figcraft` | 248 | `backend/core-tests` | 29 |
| `docs/trace-figma-parity` | 155 | `docs/plans` | 28 |
| `docs/evidence` | 95 | `runtime-test/db-fixtures` | 27 |
| `docs/trace-ui-rebuild` | 85 | `docs/codemap` | 27 |
| `docs/audit-prior` | 82 | `docs/trace-orca` | 23 |
| `plans/seats` | 49 | `plans/waves` | 23 |
| `audit/parts` | 43 | `backend/tools` | 22 |
| `docs/trace-live` | 40 | `runtime-test/logs` | 22 |
| `docs/trace-provider` | 38 | `web/figcraft-assets` | 22 |
| `docs/research` | 38 | `docs/trace-rounds` | 19 |
| `web/panels` | 36 | `audit/diagrams` | 18 |
| `web/tests` | 32 | `plans/planners` | 17 |
| `runtime-test/packages` | 31 | `plans/foresight` | 17 |
| `audit/root` | 29 | `e2e/specs` | 16 |

The full 96-row table is `subsystem_census` in the JSON. Backend split: contract 14, core 59, provider 17, tools 25 (plus manifests). Web split: panels 36, tests 32, api 9, state 9, flow 6, styles 4, figcraft 3, shell 3, tokens 2, a11y 2, chat 2, figcraft-assets 22, build 5, assets 1, splash 1.

---

## 6. Owner-lane census

| Lane | Files | What it owns | Report |
|---|---:|---|---|
| L1 | 11 | `execution.rs`, `planner.rs`, `caps.rs`, `checks.rs`, `context.rs`, `run_state.rs`, `daemon.rs`, `recovery.rs`, `graph.rs`, `remote.rs`, `remote_supervisor.rs` | `docs/analysis/lanes/L1-runtime-governance.md` |
| L2 | 36 | tools crate (22), core tool modules (4: `tool_loop`, `tools_bridge`, `approval_wait`, `prompts`), prompt assets (10) | `L2-tools-security.md` |
| L3 | 5 | `db.rs`, `store.rs`, `entities.rs`, `history.rs`, `snapshot.rs` | `L3-state-memory.md` |
| L4 | 50 | contract crate, provider crate, HTTP/WS/bus/handshake/static-UI, config/platform, `web/src/api` (9), CLI, contract/provider docs | `L4-api-provider.md` |
| L5 | 379 | `web/**` except `web/src/api` (128), `docs/ui` (250), `docs/UI-UX-ROADMAP.md` (1) | `L5-ui-ux.md` |
| L6 | 682 | backend integration/unit test targets, `e2e/**`, `tools/**`, `runtime-test/**`, `dist/**`, `ledgers/**`, launchers and toolchain files, `docs/trace/**`, `docs/evidence/**`, release docs | `L6-tests-release-install.md` |
| L7 | 410 | `README.md`, `CHANGE-LEDGER-IMPL.md`, `docs/{audit,research,plans,CODEMAP,diagrams,claims,reports,top-level}`, `plans/**`, `audit/**`, `skills/**` | `L7-docs-claims.md` |

Sum = **1,573**. Note the deliberate cross-ownerships: `web/src/api` is L4 by the L4 report's own scope ("Frontend API Client") although L5's report covers `web/src/**`; prompt assets are L2 (persona security surface) although L7 also claims prompts. Both reports' mentions are preserved per path in `report_mentions`.

---

## 7. HTTP route census

Source: `backend/crates/hiverouter-core/src/http.rs:609-703` (`build_router`). **60 path registrations = 63 method+path pairs** (3 paths serve two methods: `/api/remotes`, `/api/daemons`, `/api/provider/profile`): 42 `GET` (including the `/ws` upgrade), 19 `POST`, 2 `DELETE`. All routes are Host-guarded; mutating routes require the per-boot session token (`http.rs:706-793`).

| Method | Path | Handler | Line |
|---|---|---|---|
| GET | `/health` | `health` | 610 |
| GET | `/diagnose` | `diagnose` | 611 |
| GET | `/api/runtime` | `runtime_status` | 612 |
| GET | `/api/session` | `api_session` | 613 |
| GET | `/api/history/runs` | `history_runs` | 614 |
| GET | `/api/history/runs/{run_id}/revisions` | `history_revisions` | 615 |
| GET | `/api/history/runs/{run_id}/state` | `history_state` | 616 |
| GET | `/api/history/runs/{run_id}/messages` | `history_messages` | 617 |
| GET | `/api/agents/runs/{run_id}/messages` | `agent_messages` | 618 |
| POST | `/api/agents/runs/{run_id}/messages/{message_id}/answer` | `answer_agent_message` | 620 |
| GET | `/api/agents/runs/{run_id}/roster` | `agent_roster` | 623 |
| GET | `/api/agents/runs/{run_id}/memory` | `agent_memory_for_run` | 624 |
| GET | `/api/memory/banks` | `memory_banks` | 625 |
| GET | `/api/memory/summary` | `memory_summary` | 626 |
| GET | `/api/memory/banks/{bank_id}/entries` | `memory_bank_entries` | 628 |
| POST | `/api/memory/clear` | `clear_run_memory` | 631 |
| POST | `/api/memory/prune` | `prune_run_memory` | 632 |
| POST | `/api/memory/trust` | `promote_memory` | 633 |
| GET | `/api/memory/journal` | `memory_journal` | 634 |
| GET | `/api/memory/journal/verify` | `verify_memory_journal` | 635 |
| GET | `/api/agents/runs/{run_id}/proposals` | `run_evolution_proposals` | 637 |
| POST | `/api/agents/runs/{run_id}/proposals/{proposal_id}/authorize` | `authorize_evolution_proposal` | 641 |
| POST | `/api/agents/runs/{run_id}/proposals/{proposal_id}/apply` | `apply_evolution_lesson` | 645 |
| POST | `/api/agents/runs/{run_id}/proposals/{proposal_id}/rollback` | `rollback_evolution_lesson` | 649 |
| GET | `/api/evolution/lessons` | `evolution_lessons` | 652 |
| GET | `/api/a2a/pending` | `a2a_pending` | 653 |
| GET | `/api/a2a/messages` | `a2a_messages` | 654 |
| GET | `/api/remotes` | `remote_list` | 655 |
| POST | `/api/remotes` | `remote_create` | 655 |
| GET | `/api/remotes/{remote_id}` | `remote_detail` | 656 |
| POST | `/api/remotes/{remote_id}/revoke` | `remote_revoke` | 657 |
| POST | `/api/remotes/{remote_id}/heartbeat` | `remote_heartbeat` | 659 |
| POST | `/api/remotes/{remote_id}/devices` | `remote_device_create` | 663 |
| POST | `/api/remotes/{remote_id}/grants` | `remote_grant_create` | 666 |
| GET | `/api/remotes/{remote_id}/audit` | `remote_audit_list` | 667 |
| GET | `/api/daemons` | `daemon_list` | 668 |
| POST | `/api/daemons` | `daemon_create` | 668 |
| GET | `/api/daemons/{daemon_id}` | `daemon_detail` | 669 |
| POST | `/api/daemons/{daemon_id}/switch` | `daemon_switch` | 670 |
| POST | `/api/daemons/{daemon_id}/retire` | `daemon_retire` | 671 |
| GET | `/api/daemons/{daemon_id}/reports` | `daemon_reports` | 672 |
| GET | `/api/supervisor` | `supervisor_status` | 673 |
| GET | `/api/devices/{device_id}` | `remote_device_get` | 674 |
| POST | `/api/devices/{device_id}/actions` | `remote_action_create` | 675 |
| POST | `/api/device-actions/{action_id}/resolve` | `remote_action_resolve` | 677 |
| GET | `/api/approvals` | `pending_approvals` | 680 |
| GET | `/api/templates` | `templates` | 681 |
| GET | `/api/workspace` | `workspace` | 682 |
| GET | `/api/runs/active` | `active_run` | 683 |
| GET | `/api/agents/profiles` | `agent_profiles` | 684 |
| POST | `/api/agents/profiles/import` | `import_agent_profile` | 685 |
| DELETE | `/api/agents/profiles/{profile_id}` | `delete_agent_profile` | 686 |
| GET | `/api/agents/memory` | `agent_memory_list` | 687 |
| DELETE | `/api/agents/memory/{agent_key}` | `clear_agent_memory` | 688 |
| GET | `/api/usage/runs/{run_id}` | `run_usage` | 689 |
| GET | `/api/runs/{run_id}/detail` | `run_detail` | 690 |
| GET | `/api/memory/context/{task_id}` | `task_memory_context` | 691 |
| GET | `/api/projects` | `projects` | 692 |
| GET | `/api/provider/models` | `provider_models` | 693 |
| GET | `/api/provider/balance` | `provider_balance` | 694 |
| GET | `/api/provider/profile` | `provider_profile_get` | 696 |
| POST | `/api/provider/profile` | `provider_profile_set` | 696 |
| GET | `/ws` | `ws_handler` | 699 |

Distribution: 3 non-API routes (`/health`, `/diagnose`, `/ws`) and 57 `/api/*` paths (60 method+path pairs under `/api`, because three serve two methods). The fallback is the embedded SPA (`http.rs:700`).

---

## 8. WebSocket command and event census

**Commands — 17 declared, 14 frozen, 14 dispatched.** `backend/crates/hiverouter-contract/src/command.rs:48-105` declares 17 `CommandName` variants; `is_frozen()` (`:143-148`) excludes the three `DIFF_*` names. `ws.rs` has 14 `Command::` dispatch arms (verified by extracting `Command::[A-Za-z]+`): `SubmitPrompt`, `SpawnSubHive`, `RewireTask`, `CancelRun`, `KillWorker`, `ApprovalResolve`, `SettingsStatus`, `FileTreeList`, `FileRead`, `ChatPin`, `TemplateInsert`, `HistoryRevisionSet`, `ViewportSet`, `ViewportRecenter`.

| Command | Frozen | Dispatch | Client-only? |
|---|---|---|---|
| `SUBMIT_PROMPT` | yes | `ws.rs` | no |
| `SPAWN_SUB_HIVE` | yes | `ws.rs` | no |
| `REWIRE_TASK` | yes | `ws.rs` | no |
| `CANCEL_RUN` | yes | `ws.rs` | no |
| `KILL_WORKER` | yes | `ws.rs` | no |
| `APPROVAL_RESOLVE` | yes | `ws.rs` | no |
| `SETTINGS_STATUS` | yes | `ws.rs` | no |
| `FILE_TREE_LIST` | yes | `ws.rs` | no |
| `FILE_READ` | yes | `ws.rs` | no |
| `CHAT_PIN` | yes | `ws.rs` | yes (ACK only) |
| `TEMPLATE_INSERT` | yes | `ws.rs` | no |
| `HISTORY_REVISION_SET` | yes | `ws.rs` | no |
| `VIEWPORT_SET` | yes | `ws.rs` | yes |
| `VIEWPORT_RECENTER` | yes | `ws.rs` | yes |
| `DIFF_PROPOSE` | **no** | **MISSING** | n/a |
| `DIFF_APPLY` | **no** | **MISSING** | n/a |
| `DIFF_REJECT` | **no** | **MISSING** | n/a |

**Events — 24.** `backend/crates/hiverouter-contract/src/event.rs:72-96` (wire names at `:103-126`): `ACK`, `ERROR`, `SERVER_HELLO`, `RESYNC_REQUIRED`, `RUN_CREATED`, `RUN_STATUS`, `REPLY_DELTA`, `REPLY_DONE`, `NODE_SPAWN`, `NODE_DRAG`, `NODE_REWIRE`, `NODE_STATE`, `RUN_METRICS`, `APPROVAL_REQUEST`, `APPROVAL_RESOLVED`, `SETTINGS_STATUS_RESULT`, `FILE_TREE_RESULT`, `FILE_CONTENT`, `HISTORY_SNAPSHOT`, `PING`, `PONG`, `TOOL_CALL`, `TOOL_RESULT`, `AGENT_MESSAGE`. The transport control frame is untyped `{"type":"hello","last_seq":n}` (`command.rs:4-24`, `ws.rs:567-590`); there is no typed `APPROVAL_HYDRATE` — approvals are replayed after `SERVER_HELLO` by design.

**Refusals — 78** named `E_*` codes in `backend/crates/hiverouter-contract/src/refusal.rs:73-323` (extracted 78 `(E_CODE, retryable, message, fix)` rows; the L4 report's count of 78 agrees).

---

## 9. Tool, permission and persona census

**Tools — 12.** Enum at `backend/crates/hiverouter-tools/src/tool.rs:115-140`; wire ids at `:147-158`:

| Tool | Line | Class | Notes |
|---|---|---:|---|
| `spawn_agent` | 147 | control | child spawn; caps enforced by engine |
| `ask_parent` | 148 | control | mailbox question |
| `send_to_sibling` | 149 | control | A2A message |
| `report_result` | 150 | control | terminal result |
| `finish` | 151 | control | terminal |
| `file_read` | 152 | read | jailed, 1 MiB cap (`command.rs:37`) |
| `file_list` | 153 | read | optional path |
| `file_write` | 154 | mutate | approval-gated, atomic |
| `shell_exec` | 155 | execute | allowlisted programs + Job Object |
| `request_detached` | 156 | control | remote/daemon admission |
| `web_fetch` | 157 | network | Worker/Super only |
| `vote_evolution` | 158 | control | SuperAgent voter |

**Permission classes — 5** (`Read`, `Mutate`, `Execute`, `Network`, `Control`; `tool.rs:29-53`). **Agent tiers — 5** (`hyper_agent`, `super_agent`, `worker_agent`, `subagent`, `tool_agent`; `tool.rs:75-98`). **Tier grants — 35** (3+12+11+5+4, `tool.rs:460-516`; the L7 report's independent count of 35 agrees).

**Personas — 8** (`backend/crates/hiverouter-core/assets/prompts/personas.json`), each backed by one of 8 `.prompty` templates plus `manifest.json` (10 assets total):

| Persona | Tier | Template | Tool grants | Output schema |
|---|---|---|---:|---|
| `planner` | hyper | planner | 0 | `plan-v1` |
| `content-writer` | worker | content-writer | 7 | — |
| `superagent` | super | superagent | 10 | — |
| `subagent` | worker | subagent | 6 | — |
| `data-analyst` | worker | data-analyst | 7 | — |
| `code-reviewer` | worker | code-reviewer | 7 | — |
| `chatbot` | worker | chatbot | 4 | — |
| `agent` | worker | agent | 11 | — |

Assets are embedded at compile time (`prompts.rs:38-75`), so a missing asset breaks the build, not boot. The L2 report flags that the root's tool vocabulary bypasses this registry; the L7 report flags the `subagent` persona's `send_to_sibling` grant as inconsistent with the Subagent tier. Both are lane findings; this census records the data, not the verdict.

---

## 10. Database table and migration census

Source: `backend/crates/hiverouter-core/src/db.rs`. **23 ordered migrations** (`MIGRATIONS`, versions 1–23; forward-only append, each with a `down`), **28 distinct tables**, **30 `CREATE TABLE` statements** (`schema_migrations` is created in the engine and re-created in the down path and one test).

| # | Migration | Introduces |
|---:|---|---|
| 1 | `bootstrap` | `app_meta` |
| 2 | `runs_tasks_chat` | `runs`, `tasks`, `chat_messages`, `command_receipts` |
| 3 | `snapshot_index` | `snapshots` |
| 4 | `tool_turns` | `tool_turns` |
| 5 | `task_results` | `task_results` |
| 6 | `agent_messages` | `agent_messages` |
| 7 | `agent_memory` | `agent_memory` |
| 8 | `run_usage` | `run_usage` |
| 9 | `agent_profiles` | `agent_profiles` |
| 10 | `projects` | `projects` |
| 11 | `agent_memory_scoped` | rebuild of `agent_memory` through a new `agent_memory_scoped` table |
| 12 | `a2a_answer_state` | `agent_messages` answer/attempt columns |
| 13 | `task_identity` | task class/profile identity + `idx_tasks_identity` |
| 14 | `memory_banks` | `memory_banks`, `memory_entries` |
| 15 | `memory_journal` | `memory_journal` |
| 16 | `memory_context_snapshots` | `memory_context_snapshots` |
| 17 | `task_ownership` | `tasks.owner_task_id` + index |
| 18 | `memory_journal_hash_chain` | journal hash chain |
| 19 | `single_run_owner` | single-owner index/constraint |
| 20 | `memory_journal_head` | journal head |
| 21 | `remote_control_plane` | `remote_agents`, `devices`, `remote_grants`, `device_actions`, `remote_audit`, `remote_supervisor_state` |
| 22 | `daemon_plane` | `daemons`, `daemon_reports` |
| 23 | `a2a_attempt_audit` | `a2a_audit` |

Tables (28): `app_meta`, `runs`, `tasks`, `chat_messages`, `command_receipts`, `snapshots`, `tool_turns`, `task_results`, `agent_messages`, `agent_memory`, `run_usage`, `agent_profiles`, `projects`, `agent_memory_scoped`, `memory_banks`, `memory_entries`, `memory_journal`, `memory_context_snapshots`, `remote_agents`, `devices`, `remote_grants`, `device_actions`, `remote_audit`, `remote_supervisor_state`, `daemons`, `daemon_reports`, `a2a_audit`, `schema_migrations`.

Note: `agent_memory_scoped` is the transient rebuild target of migration 11 (renamed then dropped); the durable table is `agent_memory`. Schema version is read from `schema_migrations` and a DB newer than the binary refuses boot (`db.rs:876-886,988-1005`).

---

## 11. UI surface census

**Router.** `web/src/App.tsx:32-33`: 7 nav sections (`workspace`, `agents`, `data`, `models`, `routing`, `remote`, `settings`) × 4 tab ids (`workspace`, `swarm`, `routing`, `history`) = **28 addressable hash surfaces** (`#/nav/tab`, `App.tsx:47-81`). Only `workspace` varies content by tab (`App.tsx:109-146`); the other six render their panel for all four tab values (`App.tsx:148-163`), so **10 surfaces render distinct content** and 18 are duplicate addresses.

| Hash | Renders |
|---|---|
| `#/workspace/workspace` | Canvas + ChatCore + ControlBar + Scrubber + LibraryPanel |
| `#/workspace/swarm` | SwarmPanel + AgentHierarchy + ProjectsPanel |
| `#/workspace/routing` | RoutingPanel |
| `#/workspace/history` | HistoryPanel + RunDetailPanel + RunTranscriptPanel + Scrubber + Canvas |
| `#/agents/*` | AgentsPanel + AgentHierarchy, AgentRoster, AgentProfiles, AgentMessages, AgentMemory, A2AInbox, MemoryBanks, Lessons, Proposals, ToolActivity |
| `#/data/*` | DataPanel |
| `#/models/*` | ModelsPanel |
| `#/routing/*` | RoutingPanel |
| `#/remote/*` | RemotePanel + DaemonPanel + ConfirmDialog |
| `#/settings/*` | SettingsPanel + DiagnosticsPanel + FigCraftPanel + DevGallery |

**Panels — 36 components**, all mounted (no orphan): A2AInboxPanel, AgentHierarchy, AgentMemoryPanel, AgentMessagesPanel, AgentProfilesPanel, AgentRosterPanel (all ← AgentsPanel), AgentsPanel ← App, ApprovalOverlay ← App, ConfirmDialog ← MemoryBanksPanel+RemotePanel, ControlBar ← App, DaemonPanel ← RemotePanel, DataPanel ← App, DevGallery ← SettingsPanel, DiagnosticsPanel ← SettingsPanel, FigCraftPanel ← SettingsPanel, Header ← App, HistoryPanel ← App, HudBadge ← Canvas, LessonsPanel ← AgentsPanel, LibraryPanel ← App, MemoryBanksPanel ← AgentsPanel, ModelsPanel ← App, NavIcon ← NavRail, NavRail ← App, ProjectsPanel ← SwarmPanel, ProposalsPanel ← AgentsPanel, RemotePanel ← App, RoutingPanel ← App, RunDetailPanel ← HistoryPanel, RunTranscriptPanel ← HistoryPanel, Scrubber ← App, SettingsPanel ← App, StatePack ← 20 panels, SwarmPanel ← App, ToastRegion ← App, ToolActivityPanel ← AgentsPanel.

**Global chrome:** Header, NavRail, LibraryPanel, ToastRegion, ApprovalOverlay (App), splash `HiveMindSplash` (App). **Dev-only:** `DevGallery` is reachable through Settings → FigCraft (`SettingsPanel.tsx`), i.e. shipped, despite its docstring "Never rendered in production builds" (`web/src/panels/DevGallery.tsx:1`) — a UI-census discrepancy worth a lane check.

**Drift found:** `web/src/panels/NavRail.tsx:1` says "six destinations"; the array at `:9-17` has seven and `App.tsx:32` has seven.

**Client transport:** `web/src/api/client.ts` opens `/api/session` → `/ws`, sends `hello{last_seq}`, correlates ACK/ERROR, reconnects; the dev mock is compile-time gated and dynamically imported (`web/src/api/index.ts:32,62-80`).

---

## 12. Caps, hierarchy and evolution gate

**Caps** (`backend/crates/hiverouter-core/src/caps.rs`): `MAX_SPAWN_DEPTH = 3` (:9), `MAX_FANOUT = 8` (:11), `MAX_AGENTS = 64` (:13), `MAX_CALLS = 64` (:18). Planner: `MAX_PLAN_NODES = 40` (`planner.rs:10`). Tool loop: `MAX_TOOL_TURNS = 12` (`tool_loop.rs:14`), `MAX_CALLS_PER_RUN = 40` (:16), `MAX_TOOL_RESULT_BYTES = 65,536` (:18), `MAX_APPROVAL_WAITS_PER_RUN = 8` (:21). Root control turn: 2 turns × 4 tool calls (L1 report, `execution.rs:4319,4328`).

**Hierarchy:** `hyper_agent` → `super_agent` → `worker_agent` → `subagent` → `tool_agent` (`tool.rs:75-98`), tasks linked by parent id, depth ≤ 3. **Evolution gate:** root `propose_improvement` → four SuperAgent `vote_evolution` voters → operator `POST /api/agents/runs/{run_id}/proposals/{proposal_id}/authorize` → `apply`/`rollback` → `LESSONS.jsonl` + `evolution_lessons` (`http.rs:637-652`, `tool.rs:450-455`). Three surfaces: `ProposalsPanel`, `LessonsPanel`, `ToolActivityPanel`.

**Feature-ownership rows that are not owned by implementation:**

| Feature | Kind | Status | Why |
|---|---|---|---|
| `DIFF_PROPOSE` | command | **MISSING** | Declared `command.rs:78`; `is_frozen()` false; no `ws.rs` arm; no handler |
| `DIFF_APPLY` | command | **MISSING** | Declared `command.rs:80`; no arm; no handler |
| `DIFF_REJECT` | command | **MISSING** | Declared `command.rs:82`; no arm; no handler |
| `APPROVAL_HYDRATE` | command | **ABSENT BY DESIGN** | `command.rs:4-24` documents the untyped `hello{last_seq}` pull; no typed command exists by decision |

---

## 13. Test-suite census (declared, not executed)

| Suite | Files | Declared tests | Method |
|---|---:|---:|---|
| Rust unit (`#[cfg(test)]` in `src`) | 57 | **373** | `#[test]` + `#[tokio::test]` regex per tracked `.rs` |
| Rust integration (`tests/`, `benches/`) | 41 | **266** | same; 43 target files incl. `common/mod.rs`, `support/mod.rs` |
| **Rust total** | **98** | **639** | 2 `#[ignore]` attributes |
| Web (vitest) | 31 (+1 helper) | **231** `it/test` in **58** `describe` | `web/tests/*.ts` |
| E2E (Playwright) | 16 specs | **81** `test()` in **20** `describe` | `e2e/specs/*.spec.ts` |
| QA guard suites | 2 | — | `tools/qa/tests/seal-topology.test.mjs`, `trace-tools.test.mjs` |
| Benchmarks | 1 | — | `backend/crates/hiverouter-tools/benches/engine_benches.rs` |

Largest e2e specs: `live-app.spec.ts` 15, `gui-gates.spec.ts` 11, `approval-flow.spec.ts` 8, `tool-loop.spec.ts` 8. Largest Rust test targets (by declared fns) are in `backend/crates/hiverouter-core/tests/` (186 fns, matching L6's table), with 80 more in contract/provider/tools integration targets that L6's table does not include.

---

## 14. Untracked, ignored and generated locations that affect the product

Measured on disk at census time. These are not in `git ls-files` but are consumed by the build, the shipped binary, or the runtime.

| Path | Present | Files | Bytes | Impact | Why it matters | Lane |
|---|---|---:|---:|---|---|---|
| `web/dist/` | yes | 40 | 21,508,144 | HIGH | Compile-time embedded by `rust-embed` (`static_ui.rs:15-17`); the binary serves exactly these bytes; gitignored (`.gitignore:3`), so a stale dist silently ships an old UI | L5 |
| `backend/target/` | yes | 4,633 | 2,463,924,350 | MEDIUM | Cargo build tree; `target/release/hiverouter.exe` is what the launcher and release scripts consume (`.gitignore:1`) | L6 |
| `tools/w64devkit/` | yes | 9,452 | 751,507,790 | MEDIUM | Vendored MinGW toolchain for the C probe path; gitignored (`.gitignore:4`) | L6 |
| `web/node_modules/` | yes | 8,469 | 248,586,319 | MEDIUM | Required for typecheck/test/build; lockfile is tracked | L5 |
| `e2e/node_modules/` | yes | 194 | 19,066,967 | MEDIUM | Playwright install for `e2e/run-guarded.mjs` | L6 |
| `dist/HiveMind/` | yes | 3 | 36,551,694 | HIGH | Extracted 1.2-dev portable package (`HiveMind.exe`, `hiverouter.exe`, `README.txt`); untracked because `dist/` is ignored and only the 1.0.0/1.1.0 zips are force-added | L6 |
| `dist/HiveMind-1.1.0-win64/` | yes | 6 | 19,010,527 | HIGH | Extracted 1.1.0 package | L6 |
| `dist/HiveMind-v1-win64/` | yes | 5 | 14,699,760 | HIGH | Extracted earlier package | L6 |
| `dist/HiveMind.zip` | yes | 1 | 14,416,165 | HIGH | 1.2-dev portable zip (not in `git ls-files`) | L6 |
| `dist/HiveMind-v1-win64.zip` | yes | 1 | 6,571,928 | HIGH | Earlier portable zip | L6 |
| `dist/.release-manifest-2026*.json` | yes | 3 | 6,477 | HIGH | Three superseded manifest backups | L6 |
| `backend/crates/hiverouter-core/assets/prompts/` | yes | 10 | 9,237 | — | **Tracked** (10 files) despite living under an assets path; embedded at build time | L2 |
| `%LOCALAPPDATA%/HiveMind/provider.json` | no | 0 | 0 | HIGH | Provider profile resolved at `config.rs:312-315`; created by `tools/set-provider-profile.ps1`; runtime state outside the repo | L4 |
| `./data/hiverouter.db` | no | 0 | 0 | HIGH | Default runtime DB (`config.rs:12-14`: data dir `data`, file `hiverouter.db`) created on first boot; absent at census time | L3 |
| `.gitnexus/` | yes | 339 | 90,758,836 | LOW | Code-intelligence index; excluded via `.git/info/exclude:6`; not shipped | L7 |
| `.claude/` | yes | 6 | 41,268 | LOW | Agent skills; excluded `.git/info/exclude:7`; not shipped | L7 |
| `AGENTS.md`, `CLAUDE.md` | yes | 2 | 7,202 | LOW | Agent instructions; excluded `.git/info/exclude:9-10`; not shipped | L7 |
| `docs/analysis/lanes/*.md` | yes | 7 | 509,117 | LOW | The seven lane reports consumed by Section 15; untracked at census time | L1–L7 |

---

## 15. Lane-report cross-reference

### 15.1 What each report names

Parsed from each report's backticked tokens, normalised and matched against the tracked set (exact path, unique path suffix, directory prefix, declared wildcard). `suffix` counts tokens resolved to a unique tracked path from a basename or partial path (e.g. `execution.rs:4328` → `backend/crates/hiverouter-core/src/execution.rs`). `phantom` counts path-like tokens that resolve to nothing tracked; `ambiguous` counts tokens matching more than one tracked suffix.

| Report | Named tracked paths | Suffix-resolved | Scope prefixes | Phantom refs | Ambiguous refs |
|---|---:|---:|---:|---:|---:|
| L1-runtime-governance | 37 | 295 | 0 | 17 | 1 |
| L2-tools-security | 52 | 337 | 3 | 42 | 10 |
| L3-state-memory | 47 | 338 | 0 | 47 | 0 |
| L4-api-provider | 84 | 304 | 7 | 88 | 55 |
| L5-ui-ux | 94 | 96 | 2 | 147 | 136 |
| L6-tests-release-install | 230 | 375 | 17 | 211 | 23 |
| L7-docs-claims | 81 | 146 | 22 | 50 | 16 |

Because a path can be named by several reports, the union is **410 paths**; **134** of those are named by more than one report.

### 15.2 Coverage tiers (every one of the 1,573 paths)

| Tier | Paths | Meaning |
|---|---:|---|
| `named` | **410** | The path (or a unique partial form of it) appears literally in ≥ 1 report |
| `scope-specific` | **863** | Not named, but inside a declared directory wildcard of depth ≥ 2 (e.g. `docs/evidence/**`, `web/src/**`, `backend/crates/hiverouter-tools/**`) |
| `scope-broad` | **275** | Not named, covered only by a single-segment wildcard (`docs/`, `plans/`, `audit/`, `dist/`, `runtime-test/`, `tools/`, `backend/`) |
| `absent` | **25** | No report names or scopes it |

`scope-broad` composition: `plans/seats` 49, `audit/parts` 43, `audit/root` 29, `runtime-test/db-fixtures` 27, `plans/waves` 23, `audit/diagrams` 18, `plans/planners` 17, `runtime-test/logs` 17, `runtime-test/packages` 15, `audit/inventory` 8, `plans/verification` 5, `tools/cc-test` 4, `e2e/config` 3, `dist/release` 3, `plans/root` 3, `backend/build` 2, `docs/release-cli` 2, `runtime-test/workspace` 2, and one each for `root/toolchain`, `skills/hivemind`, `backend/contract`, `backend/contract-tests`, `e2e/helpers`.

### 15.3 Paths absent from all seven report files (25)

These are not product features; they are UI build inputs and the runtime FigCraft asset family. They are nonetheless part of the shipped product and are owned in this census by L5 (assets) or L7 (repo hygiene):

- `.gitignore`
- `web/index.html`
- `web/package-lock.json`
- `web/public/figcraft/SOURCES.json`
- the 21 served board SVGs `web/public/figcraft/*.svg` (`a-*` 4, `b-*` 4, `hivemind-*` 13)

Why this matters: `web/public/figcraft/*.svg` is copied into `web/dist/figcraft/` at build time (all 21 are present in `web/dist/figcraft/`), embedded by `rust-embed`, and served by the binary; `web/src/figcraft/catalog.ts` is the gallery index for them. No lane report names them, so a change to those boards would be reviewed by no one under the current lane split.

### 15.4 Feature-ownership results

| Feature family | Count | Owner file | Owner lane | Missing |
|---|---:|---|---|---:|
| HTTP method+path routes | 63 | `backend/crates/hiverouter-core/src/http.rs:609-703` | L4 | 0 |
| WS commands | 17 | `backend/crates/hiverouter-contract/src/command.rs:48-105` | L4 | 3 |
| WS events | 24 | `backend/crates/hiverouter-contract/src/event.rs:72-96` | L4 | 0 |
| Tools | 12 | `backend/crates/hiverouter-tools/src/tool.rs:115-158` | L2 | 0 |
| Personas | 8 | `backend/crates/hiverouter-core/assets/prompts/personas.json` | L2 | 0 |
| DB tables | 28 | `backend/crates/hiverouter-core/src/db.rs` | L3 | 0 |
| DB migrations | 23 | `backend/crates/hiverouter-core/src/db.rs` | L3 | 0 |
| UI panels | 36 | `web/src/panels/*.tsx` | L5 | 0 |
| UI hash surfaces | 28 | `web/src/App.tsx:32-33` | L5 | 0 |
| Prompt assets | 10 | `backend/crates/hiverouter-core/assets/prompts/*` | L2 | 0 |

The eight families counted by JSON `product_surface_features` are routes (63), tools (12), tables (28), commands (17), events (24), personas (8), panels (36), and surfaces (28): **63 + 12 + 28 + 17 + 24 + 8 + 36 + 28 = 216**. Prompt assets (10) and migrations (23) are listed for completeness; they are owned by the same persona and `db.rs` files and are not added to the 216.

---

## 16. Omission audit

| Audit question | Result |
|---|---|
| Total tracked paths | **1,573** |
| Indexed paths | **1,573** |
| Difference | **0** |
| Paths with a fallback/absent subsystem | **0** |
| Paths with no feature tag | **0** (all 1,573 carry one `feature`; `unclassified:*` = 0) |
| Paths whose feature is documentation-only | **1,137** (`feature_kind: doc`) |
| Paths mapped to a product feature family | **436** (`ui` 92, `build` 139, `test` 115, `tool` 29, `runtime` 29, `api` 27, `db` 5) |
| Unowned paths (no report names or scopes them) | **25** (Section 15.3, listed) |
| Paths covered only by a root-level wildcard | **275** (Section 15.2, listed in JSON `broad_scope_only_paths`) |
| Product/UI/API/tool/table features without an owner | **0** (the 3 `DIFF_*` names carry explicit `MISSING` rows; 1 `ABSENT BY DESIGN` row) |
| Files absent from all seven reports, yet product-bearing | **25** — 22 served FigCraft assets + `web/index.html` + `web/package-lock.json` + `.gitignore` |
| Demo or fabricated data counted as a feature | **0** (none asserted) |

**Unbuilt or unimplemented surfaces found in the census (not ownership gaps, reported for completeness):** the `DIFF_*` command names are wired into the frozen vocabulary enum but not into the protocol (Section 12); `agent_memory_scoped` exists only as a migration-11 rebuild artifact; `docs/trace/controls.json` records its own trace ladder, which L7 recomputed as 1 WIRED / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319 hops (L7 report §0.10) — this census neither confirms nor re-runs that ladder.

---

## 17. Cross-report numeric discrepancies (evidence for the parent)

| # | Claim elsewhere | Census measurement | Evidence |
|---:|---|---|---|
| 1 | L4 report: "62 method+path routes" | **63** pairs from **60** path registrations | `http.rs:610-699`; 3 paths serve GET+POST (`/api/remotes`, `/api/daemons`, `/api/provider/profile`) |
| 2 | L4 report: "12 usable WebSocket commands" | **14** dispatch arms (17 declared, 14 frozen) | `ws.rs` `Command::*` arms; `command.rs:48-105`; `is_frozen()` at `:143-148` |
| 3 | L6 report: "559 declared Rust test functions (373 unit + 186 integration)" | **639** (373 unit + **266** integration) | 186 is exactly the `hiverouter-core/tests/**` count; contract/provider/tools integration targets add 80 |
| 4 | L6 report: "236 declared web tests" | **231** `it/test` sites in 31 files (+1 helper) | `web/tests/*.ts` regex count |
| 5 | L6 report: "95 declared Playwright test sites" | **81** `test()` sites in 16 specs (20 `describe`) | `e2e/specs/*.spec.ts` |
| 6 | L6 report: "three `tools/qa/tests/*.test.mjs` guard suites" | **2** exist | `git ls-files tools/qa/tests` = `seal-topology.test.mjs`, `trace-tools.test.mjs` |
| 7 | `README.md` says 12, 10, and "11th" tools in different places (per L7) | **12 tools / 5 classes / 35 grants** | `tool.rs:115-158,460-516` |
| 8 | `NavRail.tsx` docstring: "six destinations" | **seven** nav sections | `NavRail.tsx:1` vs `:9-17`; `App.tsx:32` |
| 9 | `docs/CODEMAP/00-START-HERE.md` claims 805 tracked files (per L7) | **1,573** tracked; 36,408 working-tree files excluding `.git` (L7's own measurement) | `git ls-files`; L7 §0.7 |
| 10 | L7 report §6 lane codes | A **second, conflicting** numbering | L7 maps L1=contract/core, L2=provider, L3=tools/safety, L4=execution/governance/plans; this census keys lanes to the seven report filenames |

These are reported, not reconciled: resolving them would require editing the lane reports, which this census does not do.

---

## 18. Residual unknowns (stated as unknowns)

1. **Documentation content** — 922 docs were family-classified, not read. Claims inside `docs/audit/**`, `docs/research/**`, and `docs/trace/**` are not re-verified here; they are attributed to L7's report where used.
2. **Binary artifacts** — 11 `.exe`, 13 `.db`, 3 `.zip`, 13 `.pyc`, and 42 `.figcraft` files were measured (size), not opened. Their contents are unknown to this census.
3. **Test execution** — all test counts are declared source sites; no `cargo test`, `npm test`, or Playwright run was performed. L6's gate-freshness findings are the authority on execution state.
4. **Runtime behaviour** — no server was started; route/tool behaviour cited here is static reading, not observation.
5. **Lane-report "phantom" references** — 17–211 path-like tokens per report resolve to no tracked file (e.g. `/health`, `/api/templates`, `graph.replay`, `.state-pulse`). This is partly the reports citing routes, CSS classes, or test titles rather than files; the per-report counts are in the JSON and should not be read as errors by themselves.
6. **`docs/analysis/lanes/*.md` are untracked**, so their line numbers can drift; the citation used here is the file path, and their sizes are recorded in Section 14.
7. **Sibling artifacts appeared during the census.** While this report was being assembled, other agents wrote `docs/analysis/LEAD-RESEARCH-PSEUDOCODE-LANGUAGES.md`, `docs/analysis/WATCHDOG-LEDGER.md`, `docs/analysis/WATCHDOG-LEDGER.jsonl`, and `docs/plans/HIVEMIND-FINISH-PLAN-v3.md` (all untracked). They are not product-bearing and are outside this census's index; `repo.dirty_entries` in the JSON records them as untracked at generation time.

---

## Appendix A — complete per-path index

All 1,573 tracked paths, grouped by subsystem. Each row: `path`, owner lane, kind, feature tag, and the weakest report coverage (`-` = absent from every lane report; `scope>=` = only a depth-≥2 wildcard covers it; `scope*` = only a root-level wildcard covers it).

Coverage tiers across the 1,573 paths: named=410, scope-specific=863, scope-broad=275, absent=25.

Generated from `git ls-files` at HEAD. Each row: path, owner lane, kind, feature tag, and the weakest report coverage (`-` = absent from every lane report; `scope>=` = only a depth-2-or-deeper wildcard covers it; `scope*` = only a root-level wildcard covers it).

### A.audit/diagrams (18 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `audit/diagrams/01-system-topology.mmd` | L7 | diagram | `audit:diagrams` | scope*L7 |
| `audit/diagrams/01-system-topology.png` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/01-system-topology.svg` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/02-data-flow.mmd` | L7 | diagram | `audit:diagrams` | scope*L7 |
| `audit/diagrams/02-data-flow.png` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/02-data-flow.svg` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/03-module-dependencies.mmd` | L7 | diagram | `audit:diagrams` | scope*L7 |
| `audit/diagrams/03-module-dependencies.png` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/03-module-dependencies.svg` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/04-state-inventory.mmd` | L7 | diagram | `audit:diagrams` | scope*L7 |
| `audit/diagrams/04-state-inventory.png` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/04-state-inventory.svg` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/05-dependency-lineage.mmd` | L7 | diagram | `audit:diagrams` | scope*L7 |
| `audit/diagrams/05-dependency-lineage.png` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/05-dependency-lineage.svg` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/06-migration-target.mmd` | L7 | diagram | `audit:diagrams` | scope*L7 |
| `audit/diagrams/06-migration-target.png` | L7 | asset | `audit:diagrams` | scope*L7 |
| `audit/diagrams/06-migration-target.svg` | L7 | asset | `audit:diagrams` | scope*L7 |

### A.audit/inventory (8 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `audit/inventory/ASSIGNMENTS.json` | L7 | doc | `audit:inventory` | scope*L7 |
| `audit/inventory/CODE-STATS.md` | L7 | doc | `audit:inventory` | scope*L7 |
| `audit/inventory/DEEP-LIST.md` | L7 | doc | `audit:inventory` | scope*L7 |
| `audit/inventory/DIRECTORY-CENSUS.md` | L7 | doc | `audit:inventory` | scope*L7 |
| `audit/inventory/FILE-MANIFEST.csv` | L7 | doc | `audit:inventory` | scope*L7 |
| `audit/inventory/FILE-MANIFEST.json` | L7 | doc | `audit:inventory` | scope*L7 |
| `audit/inventory/FILE-MANIFEST-ENRICHED.csv` | L7 | doc | `audit:inventory` | scope*L7 |
| `audit/inventory/FILE-MANIFEST-ENRICHED.json` | L7 | doc | `audit:inventory` | scope*L7 |

### A.audit/parts (43 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `audit/parts/b-core/DEPENDENCIES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/b-core/EVIDENCE.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/b-core/FILES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/b-core/FINDINGS.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/b-core/QUALITY.csv` | L7 | data | `audit:parts` | scope*L7 |
| `audit/parts/b-provider/DEPENDENCIES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/b-provider/EVIDENCE.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/b-provider/FILES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/b-provider/FINDINGS.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/b-provider/QUALITY.csv` | L7 | data | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-a/DEPENDENCIES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-a/diagrams/asset-lineage.mmd` | L7 | diagram | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-a/EVIDENCE.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-a/FILES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-a/FINDINGS.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-a/QUALITY.csv` | L7 | data | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-b/_digest.json` | L7 | data | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-b/DEPENDENCIES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-b/EVIDENCE.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-b/FILES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-b/FINDINGS.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-b/QUALITY.csv` | L7 | data | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-b/tools/extract.py` | L7 | source | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-b/tools/gen_outputs.py` | L7 | source | `audit:parts` | scope*L7 |
| `audit/parts/figcraft-b/tools/write_findings.py` | L7 | source | `audit:parts` | scope*L7 |
| `audit/parts/strip-watchdog/EVIDENCE.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/strip-watchdog/VERIFICATION.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/ui-watchdog/DEPENDENCIES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/ui-watchdog/EVIDENCE.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/ui-watchdog/FILES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/ui-watchdog/FINDINGS.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/ui-watchdog/PROVENANCE.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/ui-watchdog/QUALITY.csv` | L7 | data | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-1/DEPENDENCIES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-1/EVIDENCE.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-1/FILES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-1/FINDINGS.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-1/QUALITY.csv` | L7 | data | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-2/DEPENDENCIES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-2/EVIDENCE.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-2/FILES.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-2/FINDINGS.md` | L7 | doc | `audit:parts` | scope*L7 |
| `audit/parts/watchdog-2/QUALITY.csv` | L7 | data | `audit:parts` | scope*L7 |

### A.audit/root (29 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `audit/APPENDIX-FILE-CENSUS.md` | L7 | doc | `audit:root` | scope*L7 |
| `audit/APPENDIX-IGNORED-DELIVERABLES.md` | L7 | doc | `audit:root` | scope*L7 |
| `audit/audit-after.json` | L7 | data | `audit:root` | scope*L7 |
| `audit/audit-before.json` | L7 | data | `audit:root` | scope*L7 |
| `audit/baseline.light.png` | L7 | asset | `audit:root` | scope*L7 |
| `audit/final.light.png` | L7 | asset | `audit:root` | scope*L7 |
| `audit/FINDINGS-REGISTER.md` | L7 | doc | `audit:root` | scope*L7 |
| `audit/HIVEMIND-AUDIT-PAPER.md` | L7 | doc | `audit:root` | scope*L7 |
| `audit/IGNORED-DELIVERABLES.csv` | L7 | data | `audit:root` | scope*L7 |
| `audit/QUALITY-CODE-STATS.json` | L7 | data | `audit:root` | scope*L7 |
| `audit/QUALITY-CODE-STATS.md` | L7 | doc | `audit:root` | scope*L7 |
| `audit/QUALITY-MATRIX.csv` | L7 | data | `audit:root` | scope*L7 |
| `audit/QUALITY-SUMMARY.json` | L7 | data | `audit:root` | scope*L7 |
| `audit/ROADMAP.md` | L7 | doc | `audit:root` | scope*L7 |
| `audit/ROADMAP-TASKS.csv` | L7 | data | `audit:root` | scope*L7 |
| `audit/tools/build_appendix.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/build_inventory.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/build_roadmap.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/census_ignored_deliverables.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/complete_figcraft_a.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/dedupe_figcraft_a.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/enrich_manifest.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/finalize_audit.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/merge_quality.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/quality_code_stats.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/tools/synthesize_paper.py` | L7 | doc | `audit:root` | scope*L7 |
| `audit/UNSLOP-REPORT.md` | L7 | doc | `audit:root` | scope*L7 |
| `audit/VERIFICATION.md` | L7 | doc | `audit:root` | scope*L7 |
| `audit/WAVE-SHEET.md` | L7 | doc | `audit:root` | scope*L7 |

### A.backend/build (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/Cargo.lock` | L4 | config | `build:workspace` | scope*L6 |
| `backend/Cargo.toml` | L4 | config | `build:workspace` | scope*L6 |

### A.backend/contract (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/Cargo.toml` | L4 | config | `build:crate` | scope*L6 |
| `backend/crates/hiverouter-contract/src/lib.rs` | L4 | source | `core:barrel` | L3 |

### A.backend/contract/command (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/src/command.rs` | L4 | source | `contract:command` | L3,L4,L6 |

### A.backend/contract/envelope (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/src/envelope.rs` | L4 | source | `contract:envelope` | L3,L4 |

### A.backend/contract/event (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/src/event.rs` | L4 | source | `contract:event` | L2,L3,L4,L6 |

### A.backend/contract/ids (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/src/ids.rs` | L4 | source | `contract:ids` | L3,L4 |

### A.backend/contract/refusal (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/src/refusal.rs` | L4 | source | `contract:refusal` | L3,L4,L6 |

### A.backend/contract/schema (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/src/schema.rs` | L4 | source | `contract:schema` | L3,L4 |

### A.backend/contract/schemas (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/schemas/contract-v1.json` | L4 | schema | `contract:schema` | L1 |

### A.backend/contract/state (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/src/state.rs` | L4 | source | `contract:state` | L1,L3,L4,L6 |

### A.backend/contract/timestamp (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/src/timestamp.rs` | L4 | source | `contract:timestamp` | L3,L4,L6 |

### A.backend/contract/usage (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/src/usage.rs` | L4 | source | `contract:usage` | L3,L4,L6 |

### A.backend/contract-tests (4 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-contract/tests/common/mod.rs` | L6 | test | `test:contract:mod` | scope*L6 |
| `backend/crates/hiverouter-contract/tests/golden_schemas.rs` | L6 | test | `test:contract:golden_schemas` | L4 |
| `backend/crates/hiverouter-contract/tests/schema_roundtrip.rs` | L6 | test | `test:contract:schema_roundtrip` | L4 |
| `backend/crates/hiverouter-contract/tests/ui_command_matrix.rs` | L6 | test | `test:contract:ui_command_matrix` | L4 |

### A.backend/core (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/Cargo.toml` | L4 | config | `build:crate` | scope>=L2,L6 |

### A.backend/core-platform (4 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/build.rs` | L4 | source | `build:script` | scope>=L2,L6 |
| `backend/crates/hiverouter-core/src/config.rs` | L4 | source | `config:env` | L2,L3,L4,L6 |
| `backend/crates/hiverouter-core/src/lib.rs` | L4 | source | `core:barrel` | L3,L4 |
| `backend/crates/hiverouter-core/src/main.rs` | L4 | source | `core:bin` | L3,L4 |

### A.backend/core-runtime (12 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/src/caps.rs` | L1 | source | `runtime:caps` | L1,L6,L7 |
| `backend/crates/hiverouter-core/src/checks.rs` | L1 | source | `runtime:checks` | L4,L6 |
| `backend/crates/hiverouter-core/src/context.rs` | L1 | source | `runtime:context` | L1,L6 |
| `backend/crates/hiverouter-core/src/daemon.rs` | L1 | source | `runtime:daemon` | L3,L4,L6 |
| `backend/crates/hiverouter-core/src/execution.rs` | L1 | source | `runtime:execution` | L1,L2,L3,L4,L6,L7 |
| `backend/crates/hiverouter-core/src/graph.rs` | L1 | source | `runtime:graph` | L1,L3,L6 |
| `backend/crates/hiverouter-core/src/planner.rs` | L1 | source | `runtime:planner` | L1,L6 |
| `backend/crates/hiverouter-core/src/provider_executor.rs` | L4 | source | `provider:executor` | L1,L2,L4,L6,L7 |
| `backend/crates/hiverouter-core/src/recovery.rs` | L1 | source | `runtime:recovery` | L1,L3,L6 |
| `backend/crates/hiverouter-core/src/remote.rs` | L1 | source | `remote:control-plane` | L3,L4,L6 |
| `backend/crates/hiverouter-core/src/remote_supervisor.rs` | L1 | source | `remote:supervisor` | L3,L6 |
| `backend/crates/hiverouter-core/src/run_state.rs` | L1 | source | `runtime:state-machine` | L1,L3,L6 |

### A.backend/core-state (5 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/src/db.rs` | L3 | source | `db:schema` | L1,L2,L3,L4,L6 |
| `backend/crates/hiverouter-core/src/entities.rs` | L3 | source | `persistence:entities` | L1,L3 |
| `backend/crates/hiverouter-core/src/history.rs` | L3 | source | `history:read-model` | L3,L4 |
| `backend/crates/hiverouter-core/src/snapshot.rs` | L3 | source | `history:snapshot` | L1,L3 |
| `backend/crates/hiverouter-core/src/store.rs` | L3 | source | `persistence:store` | L1,L2,L3,L4,L6,L7 |

### A.backend/core-tests (29 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/tests/approval_ledger.rs` | L6 | test | `test:core:approval_ledger` | L2,L3,L6 |
| `backend/crates/hiverouter-core/tests/chat_store.rs` | L6 | test | `test:core:chat_store` | L3,L6 |
| `backend/crates/hiverouter-core/tests/context_switch.rs` | L6 | test | `test:core:context_switch` | L6 |
| `backend/crates/hiverouter-core/tests/daemon_tick_degraded.rs` | L6 | test | `test:core:daemon_tick_degraded` | L3,L6 |
| `backend/crates/hiverouter-core/tests/graph_authority.rs` | L6 | test | `test:core:graph_authority` | L1,L3,L6 |
| `backend/crates/hiverouter-core/tests/handshake.rs` | L6 | test | `test:core:handshake` | scope>=L2,L6 |
| `backend/crates/hiverouter-core/tests/http_boot.rs` | L6 | test | `test:core:http_boot` | L2,L3,L4 |
| `backend/crates/hiverouter-core/tests/hyperagent_governance.rs` | L6 | test | `test:core:hyperagent_governance` | L1,L3,L6 |
| `backend/crates/hiverouter-core/tests/kill_restore.rs` | L6 | test | `test:core:kill_restore` | L1,L3,L6 |
| `backend/crates/hiverouter-core/tests/lead_bugcheck_boot.rs` | L6 | test | `test:core:lead_bugcheck_boot` | L2,L6 |
| `backend/crates/hiverouter-core/tests/mem_isolation.rs` | L6 | test | `test:core:mem_isolation` | L1,L3,L6 |
| `backend/crates/hiverouter-core/tests/prompt_assets.rs` | L6 | test | `test:core:prompt_assets` | L1,L2,L6 |
| `backend/crates/hiverouter-core/tests/prompt_assets_wiring.rs` | L6 | test | `test:core:prompt_assets_wiring` | L1,L2,L6 |
| `backend/crates/hiverouter-core/tests/proposal_authorization.rs` | L6 | test | `test:core:proposal_authorization` | L1,L2,L3,L4,L6 |
| `backend/crates/hiverouter-core/tests/provider_adapter.rs` | L6 | test | `test:core:provider_adapter` | L2,L4,L6,L7 |
| `backend/crates/hiverouter-core/tests/provider_live.rs` | L6 | test | `test:core:provider_live` | L4,L6 |
| `backend/crates/hiverouter-core/tests/provider_streaming.rs` | L6 | test | `test:core:provider_streaming` | L4,L6 |
| `backend/crates/hiverouter-core/tests/recovery_gates.rs` | L6 | test | `test:core:recovery_gates` | L1,L3,L6 |
| `backend/crates/hiverouter-core/tests/recovery_multi_run.rs` | L6 | test | `test:core:recovery_multi_run` | L1,L3,L6 |
| `backend/crates/hiverouter-core/tests/remote_control_plane.rs` | L6 | test | `test:core:remote_control_plane` | L2,L3,L6 |
| `backend/crates/hiverouter-core/tests/run_lifecycle.rs` | L6 | test | `test:core:run_lifecycle` | L1,L2,L3,L6,L7 |
| `backend/crates/hiverouter-core/tests/scale_s0.rs` | L6 | test | `test:core:scale_s0` | L1,L6 |
| `backend/crates/hiverouter-core/tests/scale_s1.rs` | L6 | test | `test:core:scale_s1` | L1,L6 |
| `backend/crates/hiverouter-core/tests/snapshot_history.rs` | L6 | test | `test:core:snapshot_history` | L1,L3,L4,L6 |
| `backend/crates/hiverouter-core/tests/support/mod.rs` | L6 | test | `test:core:mod` | L6 |
| `backend/crates/hiverouter-core/tests/suspension.rs` | L6 | test | `test:core:suspension` | L1,L2,L3,L6 |
| `backend/crates/hiverouter-core/tests/tool_loop_engine.rs` | L6 | test | `test:core:tool_loop_engine` | L1,L2,L6 |
| `backend/crates/hiverouter-core/tests/tool_loop_matrix.rs` | L6 | test | `test:core:tool_loop_matrix` | L1,L2,L6 |
| `backend/crates/hiverouter-core/tests/ws_transport.rs` | L6 | test | `test:core:ws_transport` | L2,L3,L4,L6 |

### A.backend/core-tools (4 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/src/approval_wait.rs` | L2 | source | `approvals:wait` | L2,L3,L6 |
| `backend/crates/hiverouter-core/src/prompts.rs` | L2 | source | `prompts:registry` | L1,L2,L7 |
| `backend/crates/hiverouter-core/src/tool_loop.rs` | L2 | source | `tools:loop` | L1,L2,L6,L7 |
| `backend/crates/hiverouter-core/src/tools_bridge.rs` | L2 | source | `tools:bridge` | L2,L3,L4,L6 |

### A.backend/core-transport (5 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/src/bus.rs` | L4 | source | `event:bus` | L3,L4,L6 |
| `backend/crates/hiverouter-core/src/handshake.rs` | L4 | source | `ws:handshake` | scope>=L2,L6 |
| `backend/crates/hiverouter-core/src/http.rs` | L4 | source | `http:api` | L1,L2,L3,L4,L7 |
| `backend/crates/hiverouter-core/src/static_ui.rs` | L4 | source | `ui:embed` | L4,L6 |
| `backend/crates/hiverouter-core/src/ws.rs` | L4 | source | `ws:transport` | L1,L2,L3,L4,L6 |

### A.backend/prompt-assets (10 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/assets/prompts/agent.prompty` | L2 | asset | `persona:agent` | scope>=L2,L6 |
| `backend/crates/hiverouter-core/assets/prompts/chatbot.prompty` | L2 | asset | `persona:chatbot` | scope>=L2,L6 |
| `backend/crates/hiverouter-core/assets/prompts/code-reviewer.prompty` | L2 | asset | `persona:code-reviewer` | scope>=L2,L6 |
| `backend/crates/hiverouter-core/assets/prompts/content-writer.prompty` | L2 | asset | `persona:content-writer` | scope>=L2,L6 |
| `backend/crates/hiverouter-core/assets/prompts/data-analyst.prompty` | L2 | asset | `persona:data-analyst` | scope>=L2,L6 |
| `backend/crates/hiverouter-core/assets/prompts/manifest.json` | L2 | asset | `prompts:manifest` | L1,L2,L7 |
| `backend/crates/hiverouter-core/assets/prompts/personas.json` | L2 | asset | `personas:registry` | L1,L2,L7 |
| `backend/crates/hiverouter-core/assets/prompts/planner.prompty` | L2 | asset | `persona:planner` | L1,L2,L7 |
| `backend/crates/hiverouter-core/assets/prompts/subagent.prompty` | L2 | asset | `persona:subagent` | L7 |
| `backend/crates/hiverouter-core/assets/prompts/superagent.prompty` | L2 | asset | `persona:superagent` | L7 |

### A.backend/provider (10 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-provider/Cargo.toml` | L4 | config | `build:crate` | scope>=L4,L6 |
| `backend/crates/hiverouter-provider/src/accounting.rs` | L4 | source | `provider:accounting` | L4,L6 |
| `backend/crates/hiverouter-provider/src/budget.rs` | L4 | source | `provider:budget` | L4,L6 |
| `backend/crates/hiverouter-provider/src/client.rs` | L4 | source | `provider:client` | L4,L6 |
| `backend/crates/hiverouter-provider/src/dto.rs` | L4 | source | `provider:dto` | L4,L6 |
| `backend/crates/hiverouter-provider/src/error.rs` | L4 | source | `provider:error` | scope>=L4,L6 |
| `backend/crates/hiverouter-provider/src/lib.rs` | L4 | source | `core:barrel` | scope>=L4,L6 |
| `backend/crates/hiverouter-provider/src/pricing.rs` | L4 | source | `provider:pricing` | L4,L6 |
| `backend/crates/hiverouter-provider/src/secret.rs` | L4 | source | `provider:secret` | L4,L6 |
| `backend/crates/hiverouter-provider/src/tool_assembler.rs` | L4 | source | `provider:tool-assembler` | L2 |

### A.backend/provider-tests (8 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-provider/tests/error_matrix.rs` | L6 | test | `test:provider:error_matrix` | L4 |
| `backend/crates/hiverouter-provider/tests/rate_and_provenance.rs` | L6 | test | `test:provider:rate_and_provenance` | L4 |
| `backend/crates/hiverouter-provider/tests/refusal_matrix.rs` | L6 | test | `test:provider:refusal_matrix` | L4 |
| `backend/crates/hiverouter-provider/tests/streaming.rs` | L6 | test | `test:provider:streaming` | L4 |
| `backend/crates/hiverouter-provider/tests/tool_assembler.rs` | L6 | test | `test:provider:tool_assembler` | scope>=L4,L6 |
| `backend/crates/hiverouter-provider/tests/tool_call_transport.rs` | L6 | test | `test:provider:tool_call_transport` | L4 |
| `backend/crates/hiverouter-provider/tests/tool_continuation.rs` | L6 | test | `test:provider:tool_continuation` | L4 |
| `backend/crates/hiverouter-provider/tests/tool_conversation.rs` | L6 | test | `test:provider:tool_conversation` | L4 |

### A.backend/tools (22 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-tools/Cargo.toml` | L2 | config | `build:crate` | scope>=L2,L4,L6 |
| `backend/crates/hiverouter-tools/src/approval.rs` | L2 | source | `approvals:store` | L2,L3,L6 |
| `backend/crates/hiverouter-tools/src/audit.rs` | L2 | source | `tools:audit` | L2,L6 |
| `backend/crates/hiverouter-tools/src/cancel.rs` | L2 | source | `tools:cancel` | L2,L6 |
| `backend/crates/hiverouter-tools/src/census.rs` | L2 | source | `tools:census` | L2,L6 |
| `backend/crates/hiverouter-tools/src/cli.rs` | L2 | source | `cli:rust` | L2,L4,L6 |
| `backend/crates/hiverouter-tools/src/cli_support.rs` | L2 | source | `cli:support` | L2,L4,L6 |
| `backend/crates/hiverouter-tools/src/commands.rs` | L2 | source | `tools:commands` | L2,L6 |
| `backend/crates/hiverouter-tools/src/db_admin.rs` | L2 | source | `tools:db-admin` | L2,L4,L6 |
| `backend/crates/hiverouter-tools/src/env_safety.rs` | L2 | source | `tools:env-safety` | L2,L6 |
| `backend/crates/hiverouter-tools/src/error.rs` | L2 | source | `provider:error` | scope>=L2,L4,L6 |
| `backend/crates/hiverouter-tools/src/executor.rs` | L2 | source | `tools:executor` | L2,L6 |
| `backend/crates/hiverouter-tools/src/health.rs` | L2 | source | `tools:health` | L2,L6 |
| `backend/crates/hiverouter-tools/src/jail.rs` | L2 | source | `tools:jail` | L2,L6 |
| `backend/crates/hiverouter-tools/src/ledger.rs` | L2 | source | `tools:ledger` | L2,L6 |
| `backend/crates/hiverouter-tools/src/lib.rs` | L2 | source | `core:barrel` | scope>=L2,L4,L6 |
| `backend/crates/hiverouter-tools/src/settings.rs` | L2 | source | `tools:settings` | L2,L6 |
| `backend/crates/hiverouter-tools/src/sha256.rs` | L2 | source | `tools:sha256` | L2,L6 |
| `backend/crates/hiverouter-tools/src/shell.rs` | L2 | source | `tools:shell` | L2,L6 |
| `backend/crates/hiverouter-tools/src/side_effects.rs` | L2 | source | `tools:side-effects` | L2,L6 |
| `backend/crates/hiverouter-tools/src/tool.rs` | L2 | source | `tools:registry` | L2,L6,L7 |
| `backend/crates/hiverouter-tools/src/watcher.rs` | L2 | source | `tools:watcher` | L2,L6 |

### A.backend/tools-bench (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-tools/benches/engine_benches.rs` | L6 | test | `bench:tools:engine_benches` | L2 |

### A.backend/tools-cli (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-tools/src/bin/hiverouter.rs` | L4 | source | `cli:rust-bin` | L2,L4 |

### A.backend/tools-tests (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `backend/crates/hiverouter-tools/tests/audit_jsonl.rs` | L6 | test | `test:tools:audit_jsonl` | L2,L4 |
| `backend/crates/hiverouter-tools/tests/lead_bugcheck_wp01.rs` | L6 | test | `test:tools:lead_bugcheck_wp01` | L2,L4 |

### A.dist/release (4 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `dist/HiveMind-1.0.0-win64.zip` | L6 | binary | `artifact:release:HiveMind-1.0.0-win64` | scope*L6,L7 |
| `dist/HiveMind-1.0.0-win64.zip.sha256` | L6 | binary | `artifact:release:HiveMind-1.0.0-win64.zip` | scope*L6,L7 |
| `dist/HiveMind-1.1.0-win64.zip` | L6 | binary | `artifact:release:HiveMind-1.1.0-win64` | L6,L7 |
| `dist/HiveMind-1.1.0-win64.zip.sha256` | L6 | binary | `artifact:release:HiveMind-1.1.0-win64.zip` | scope*L6,L7 |

### A.docs/audit-prior (82 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/audit/AGENTIC-LIVE-TESTS.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/AGENTIC-TEST-PLAN.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/AGENT-TAXONOMY-VERIFICATION.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/BASELINE-VERIFICATION.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/CAPABILITY-MATRIX.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/evidence/controls-verifier-skeleton.json` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/final/cli-diagnose-final.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/final/cli-diagnose-final.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/final/cli-health-final.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/final/live-probe-final.json` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/final/live-probe-final.mjs` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/final/live-probe-final.out.txt` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/final/pw-out/.last-run.json` | L7 | doc | `doc:audit-prior` | L6 |
| `docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/error-context.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/test-failed-1.png` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/trace.zip` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/gen_controls.py` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/census.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/census.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/census-verify.json` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/diagnose_live.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/diagnose_live.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/health_dryrun.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/health_dryrun.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/health_live.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/health_live.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/preflight.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/preflight.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/preflight-path.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/preflight-path.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/provider_offpeak.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/provider_offpeak.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/provider_peak.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/provider_peak.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/schema.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/schema.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/tool_missing.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/tool_missing.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/tools_describe.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/tools_describe.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/tools_list.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/tools_list.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/unknown.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/unknown.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/usage_error.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/usage_error.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/version.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/version.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/version-human.err` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/cli/version-human.out` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/live-probe-verifier.json` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/live-probe-verifier.mjs` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/live-probe-verifier.out.txt` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/live-probe-verifier-v2.json` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/phase-b/live-probe-verifier-v2.out.txt` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/toolchain/hello.c` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/toolchain/hello.exe` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/toolchain/hello.rs` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/evidence/toolchain/hello_rs.exe` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/FINAL-CLOSEOUT.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/FINAL-DELTA-VERIFICATION.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/GOVERNANCE-AUDIT-2.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/GOVERNANCE-TESTS.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/hivemind-memory-a2a/AUDIT.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/hivemind-memory-a2a/FINDINGS.json` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/hivemind-memory-a2a/README.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/hivemind-memory-a2a/TEST-PLAN.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/HYPERAGENT-PSEUDOCODE-REPORT.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/HYPERAGENT-TRUTH-REPORT.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/IMPROVE-DEEPDESLOP.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/IMPROVE-EVIDENCE-GAP.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/IMPROVE-LESSONS.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/IMPROVEMENT-VERIFICATION.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/IMPROVE-RESEARCH.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/IMPROVE-SCIENTIFIC.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/IMPROVE-TRUTHSEEKER.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/PHASE-B-VERIFICATION.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/PSEUDOCODE-P1-META.md` | L7 | doc | `doc:audit-prior` | L7 |
| `docs/audit/PSEUDOCODE-P2-HMARS.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/PSEUDOCODE-P3-AGI.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/QA-ENV.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |
| `docs/audit/REMOTE-PERMISSION-GAPS.md` | L7 | doc | `doc:audit-prior` | scope>=L4,L6,L7 |

### A.docs/claims (3 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/claims/CLAIM-REGISTER.csv` | L7 | doc | `doc:claims` | L7 |
| `docs/claims/CLAIM-REGISTER.json` | L7 | doc | `doc:claims` | scope>=L6,L7 |
| `docs/claims/SKILL-USE.md` | L7 | doc | `doc:claims` | scope>=L6,L7 |

### A.docs/codemap (27 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/CODEMAP/_data/inventory.csv` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/_data/inventory.json` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/_data/summary.json` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/_incoming/DIAGRAM-ANALYSIS.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/_incoming/DUMP-ANALYSIS.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/_incoming/DUMP-DELTA.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/_tools/build_codemap.py` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/_tools/build_docs.py` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/00-START-HERE.md` | L7 | doc | `doc:codemap` | L7 |
| `docs/CODEMAP/01-SCOPE-AND-CENSUS.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/02-ARCHITECTURE.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/03-BACKEND.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/04-WEB.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/05-E2E-TOOLS.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/06-DOCS-PLANS-AUDIT.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/07-RUNTIME-BINARIES.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/08-FILE-CATALOG.md` | L7 | doc | `doc:codemap` | L7 |
| `docs/CODEMAP/09-RISKS-GAPS.md` | L7 | doc | `doc:codemap` | L7 |
| `docs/CODEMAP/10-VERIFICATION.md` | L7 | doc | `doc:codemap` | L7 |
| `docs/CODEMAP/11-AUDITOR-FINDINGS.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/12-GITHUB-PUSH.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/13-HISTORICAL-AUDIT.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/14-GITDIAGRAM-ARCHITECTURE.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/15-INPUT-PROVENANCE.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/16-GITNEXUS-FINDINGS.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |
| `docs/CODEMAP/17-CLAIM-REGISTER.md` | L7 | doc | `doc:codemap` | L7 |
| `docs/CODEMAP/18-HMAS-RESEARCH.md` | L7 | doc | `doc:codemap` | scope>=L6,L7 |

### A.docs/contracts (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/contracts/ADAPTERS.md` | L4 | doc | `doc:contracts` | L4 |
| `docs/contracts/UI-COMMANDS-v1.md` | L4 | doc | `doc:contracts` | L4,L6 |

### A.docs/design-figcraft (248 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/ui/figcraft/A/A-ASSETS.json` | L5 | asset | `doc:design-figcraft` | L6 |
| `docs/ui/figcraft/A/A-EPISODES.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/A-SELF-CHECK.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/build-state.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/c4.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/fig.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/gen_build.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/gen_common.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/gen_manifest.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/page00.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/page00g.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/page01.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/page01g.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/page02.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/page02g.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/page03.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/page03g.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/pages_brand.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/pages_components.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/pages_foundations.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/pages_specimens.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/probe-s04c.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/stage0.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/stage1.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/build/styles.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/00_setup_pages.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/01_frames.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/02_foundations.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/03_foundations_groups.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/04_styles.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/05_brand.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/06_brand_groups.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/07_components.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/08_components_groups.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/09_specimens.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/A/scripts/10_specimens_groups.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/a-brand.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/a-brand.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/a-components.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/a-components.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/a-foundations.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/a-foundations.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/a-specimens.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/a-specimens.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/b-canvas-default.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/b-canvas-default.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/b-hero.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/b-hero.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/b-interaction.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/b-interaction.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/b-zoom-states.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/b-zoom-states.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-agent-cards.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-agent-cards.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-agent-cards@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-brand-lockups.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-brand-lockups.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-brand-lockups@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-colour.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-colour.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-colour@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-contact-sheet.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-controls-and-hud.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-controls-and-hud.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-controls-and-hud@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-cover.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-cover.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-cover@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-hmas-governance.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-hmas-governance.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-hmas-governance@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-hmas-hierarchy.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-hmas-hierarchy.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-hmas-hierarchy@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-motion-and-live.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-motion-and-live.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-motion-and-live@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-run-detail-and-files.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-run-detail-and-files.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-run-detail-and-files@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-space-and-depth.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-space-and-depth.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-space-and-depth@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-spatial-canvas.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-spatial-canvas.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-spatial-canvas@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-states-and-refusals.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-states-and-refusals.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-states-and-refusals@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-typography.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-typography.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS/hivemind-typography@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/ASSETS-INDEX.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/B-ASSETS.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/__pycache__/board_default.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/__pycache__/board_hero.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/__pycache__/board_interaction.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/__pycache__/board_zoom.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/__pycache__/boards_common.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/__pycache__/fig_b.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/_run.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/board_default.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/board_hero.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/board_interaction.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/board_zoom.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/boards_common.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/build_b.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/build-state.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/fig_b.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/info-B.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/make_manifest.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/patch_default.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/patch_default2.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/patch_manifest.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/patch_manifest2.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/patch_verify.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/patch3.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/patch4.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/patch5.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/tmp/board00.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/tmp/board01.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/tmp/board02.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/tmp/board03.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/tmp/effects.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/tmp/step0.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/tmp/step1.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/build/verify_b.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/scripts/00_setup_pages.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/scripts/01_frames.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/scripts/02_canvas_default.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/scripts/03_zoom_states.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/scripts/04_interaction.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/scripts/05_hero.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/B/scripts/06_effects.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/BRIEF-A.md` | L5 | doc | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/BRIEF-B.md` | L5 | doc | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/build/__pycache__/boards_c.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/build/__pycache__/boards_common.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/build/__pycache__/build_c.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/build/__pycache__/fig_c.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/build/_patch_common.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/build/boards_c.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/build/boards_common.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/build/build_c.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/build/fig_c.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/C/SUPERSEDED.md` | L5 | doc | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-00-00_setup_pages.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-01-01_frames.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-02-02_foundations.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-03-03_foundations_groups.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-04-04_styles.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-05-05_brand.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-06-06_brand_groups.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-07-07_components.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-08-08_components_groups.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-09-09_specimens.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-build-10-10_specimens_groups.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/a-a-exports.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/b-00_setup_pages.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/b-01_frames.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/b-02_canvas_default.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/b-03_zoom_states.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/b-04_interaction.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/b-05_hero.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/b-06_effects.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/EPISODES/b-exports.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/__pycache__/build_hive_style.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/__pycache__/publish_hivemind.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/__pycache__/verify_hivemind.cpython-314.pyc` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/_fix_refusal.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/_patch_hivemind.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/_reboard.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/_v2_fix.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/_v2_motion.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/_v3_fix.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/_v3_run_detail.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/base.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/build_hive_style.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/commands_phase1.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/commands_phase2.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/expected_ids.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/prev-assets.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/publish_hivemind.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/run_phase1.err` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/run_phase1.log` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/run_phase2.err` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/run_phase2.log` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/verify_hivemind.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/build/verify-report.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/episodes/01_phase1_nodes.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/episodes/02_phase2_tokens_exports.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-agent-cards.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-agent-cards@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-brand-lockups.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-brand-lockups.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-brand-lockups@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-colour.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-colour@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-controls-and-hud.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-controls-and-hud@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-cover.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-cover@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-hmas-governance.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-hmas-governance@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-hmas-hierarchy.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-hmas-hierarchy@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-motion-and-live.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-motion-and-live@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-run-detail-and-files.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-run-detail-and-files@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-space-and-depth.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-space-and-depth@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-spatial-canvas.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-spatial-canvas@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-states-and-refusals.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-states-and-refusals@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-typography.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/exports/hivemind-typography@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/HiveMind-EPISODES.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/HiveMind-SOURCE-EVIDENCE.json` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/HiveMind-Style-System.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/README.md` | L5 | doc | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/base.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/build_hive_style.py` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-agent-cards.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-agent-cards@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-brand-lockups.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-brand-lockups.svg` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-brand-lockups@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-colour.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-colour@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-controls-and-hud.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-controls-and-hud@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-cover.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-cover@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-space-and-depth.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-space-and-depth@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-spatial-canvas.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-spatial-canvas@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-typography.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/exports/hiverouter-typography@2x.png` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/FIGCRAFT-COMMAND-ANALYSIS.md` | L5 | doc | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/HiveRouter-Style-Kit.md` | L5 | doc | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/HiveRouter-Style-System.figcraft` | L5 | asset | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/source-evidence/SPEC/HIVEROUTER-TARGET-SPEC.md` | L5 | doc | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md` | L5 | doc | `doc:design-figcraft` | scope>=L6,L7 |
| `docs/ui/figcraft/WATCHDOG-BRIEF.md` | L5 | doc | `doc:design-figcraft` | scope>=L6,L7 |

### A.docs/design-system (3 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/ui/HIVEMIND-HONEYGLASS-DESIGN-SYSTEM.md` | L5 | doc | `doc:design-system` | L7 |
| `docs/ui/HIVEMIND-UI-GAP-AUDIT.md` | L5 | doc | `doc:design-system` | L7 |
| `docs/UI-UX-ROADMAP.md` | L5 | doc | `doc:design-system` | L7 |

### A.docs/diagrams (15 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/diagrams/agent-lifecycle-state.mmd` | L7 | diagram | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/agent-lifecycle-state.png` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/agent-lifecycle-state.svg` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/architecture.mmd` | L7 | diagram | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/architecture.png` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/architecture.svg` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/er-schema.mmd` | L7 | diagram | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/er-schema.png` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/er-schema.svg` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/run-sequence.mmd` | L7 | diagram | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/run-sequence.png` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/run-sequence.svg` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/ui-control-event-sequence.mmd` | L7 | diagram | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/ui-control-event-sequence.png` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |
| `docs/diagrams/ui-control-event-sequence.svg` | L7 | asset | `doc:diagrams` | scope>=L6,L7 |

### A.docs/evidence (95 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/evidence/audit-r77-offline.json` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/audit-r79-offline.json` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/gate-033bcde0-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-0ac12aac-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-1d97add6-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-25e33e08-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-285fc7ba-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-3d452cfb-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-3f5ec41a-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-4113713a-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-42cc6885-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-43f2c4f9-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-451cbeab-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-456ef750-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-46ab5528-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-478a9bb2-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-4b28a5f9-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-4b436c58-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-547a7093-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-56579edf-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-574c8ec0-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-71e85ad3-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-75a04da2-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-7969f5b9-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-7d282b24-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-7d54a9e2-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-7ec1299c-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-8a19d0b5-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-8d6e2e66-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-9c0b8fad-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-9ffa60dc-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-a25023c9-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-a4de0b07-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-b09782fc-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-b22cfa5c-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-b94eb773-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-b9518b74-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-bd1f49d5-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-c151d06c-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-cbcbf37b-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-d0dc56cf-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-d27923e5-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-d5e4e403-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-d8afbd54-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-e5961240-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-e6de6824-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-ec45b2bd-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-f4ec96bf-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-f53087ae-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-f56cbb28-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/gate-fa3b0a87-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/HMARS-BUGFIX-REPORT.md` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/key-validation.md` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/live/live-tokenlab-a2a.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-applied-lesson-retrieval.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-authorize-refusal.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-auto-voters-five-seat.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-detached-request.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-exact-two-stability.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-file-write-approval.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-final-exact-two.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-hierarchy.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-idempotent-attempt.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-kill-usage.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-lesson-apply-refusal.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-lesson-retrieval.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-lesson-retrieval-parsed.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-lessons-empty.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-proposal-dedupe.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-proposal-votes.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-rate-limit.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-remote-daemon.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-request-id.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-request-id-final.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-shell-exec.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-simple.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-voter-dissent.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-web-fetch.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/live-tokenlab-web-fetch-killswitch.json` | L6 | evidence | `doc:evidence` | L7 |
| `docs/evidence/live/ui-api-type.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-api-type-connected.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-e2e-provider-save.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-final.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-final2.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-initial.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-polished-final.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-provider-settings.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-settings.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-splash.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live/ui-splash-375.png` | L6 | evidence | `doc:evidence` | scope>=L6,L7 |
| `docs/evidence/live-1.1-paid.log` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/live-provider-smoke.md` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/live-trace-1.1.md` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/scale-s0-9c0b8fad-offline.json` | L6 | evidence | `doc:evidence` | L6 |
| `docs/evidence/scale-s1-cf8bff0f-offline.json` | L6 | evidence | `doc:evidence` | L6 |

### A.docs/plans (28 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/plans/AGENT-WAVE-REPORTS.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/D2-D3-WIRING-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/FRONTEND-STATUS.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/HASH-VERIFICATION.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/HIVEMIND-MASTER-FINISH-PLAN.md` | L7 | doc | `doc:plans` | L4 |
| `docs/plans/PHASE-0-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-11-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-11-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-12-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-12-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-1-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-1-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-2-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-2-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-3-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-3-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-4-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-4-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-5-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-5-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-6-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-6-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-7-8-10-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-7-8-10-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-9-EVIDENCE.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/PHASE-9-RECON.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/SESSION-PLAN.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |
| `docs/plans/TOOLCHAIN.md` | L7 | doc | `doc:plans` | scope>=L4,L6,L7 |

### A.docs/provider (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` | L4 | doc | `doc:provider` | L7 |
| `docs/provider/DEEPSEEK-INTEGRATION.md` | L4 | doc | `doc:provider` | scope>=L6,L7 |

### A.docs/release-cli (4 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/AGENT-CLI.md` | L6 | doc | `doc:release-cli` | scope*L6,L7 |
| `docs/EXTERNAL-DRIVERS.md` | L6 | doc | `doc:release-cli` | scope*L6,L7 |
| `docs/RELEASE.md` | L6 | doc | `doc:release-cli` | L6,L7 |
| `docs/RELEASE-EVIDENCE.md` | L6 | doc | `doc:release-cli` | L6,L7 |

### A.docs/reports (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/reports/FINAL-REPORT.md` | L7 | doc | `doc:reports` | L7 |

### A.docs/research (38 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/research/agent-definitions/DRAFT.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/agent-definitions/VERIFIED.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/01-deepseek-api.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/02-hmars-patterns.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/03-scale-concurrency.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/04-memory-architecture.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/05-ui-ux-patterns.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/06-security-cost.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/07-code-backend.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/08-code-ui.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/09-synthesis-and-goal.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/10-gap-meta.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/11-donor-frameworks.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/12-role-capability-model.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/13-memory-venn.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/14-a2a-bus-roster.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/15-ui-ux-goal.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/16-deepseek-cache-cost.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/17-scale-security-impl.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/18-open-source-agents.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/19-wave3-gap-audit.md` | L7 | doc | `doc:research` | L7 |
| `docs/research/hmars/20-implementation-architecture.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/21-roles-daemons-implementation.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/22-memory-bank-implementation.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/23-a2a-implementation.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/24-ui-implementation.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/25-provider-cache-implementation.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/26-scale-security-audit.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/27-open-source-daemon-catalog.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/hmars/OPERATOR-DECISIONS.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/remote-agents/00-README.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/remote-agents/01-remote-agent-model.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/remote-agents/02-memory-isolation.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/remote-agents/03-device-capabilities.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/remote-agents/04-contracts-api-ws-refusals.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/remote-agents/05-ui-representation.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/remote-agents/06-phased-plan.md` | L7 | doc | `doc:research` | scope>=L6,L7 |
| `docs/research/remote-agents/SOURCES.md` | L7 | doc | `doc:research` | scope>=L6,L7 |

### A.docs/trace-agent-cli (4 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/trace/agent-cli/api-map.md` | L6 | doc | `doc:trace-agent-cli` | scope>=L6,L7 |
| `docs/trace/agent-cli/cli-bugfix-audit.md` | L6 | doc | `doc:trace-agent-cli` | scope>=L6,L7 |
| `docs/trace/agent-cli/cli-design.md` | L6 | doc | `doc:trace-agent-cli` | scope>=L6,L7 |
| `docs/trace/agent-cli/lifecycle-map.md` | L6 | doc | `doc:trace-agent-cli` | scope>=L6,L7 |

### A.docs/trace-figma-parity (155 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/trace/figma-parity/after-hiverouter-style-1.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/after-hiverouter-style-2.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/after-hiverouter-style-3.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/ASSETS-INDEX.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/A-ASSETS.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/A-design.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/A-EPISODES.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/A-SELF-CHECK.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/build-state.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/c4.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/fig.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/gen_build.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/gen_common.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/gen_manifest.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/page00.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/page00g.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/page01.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/page01g.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/page02.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/page02g.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/page03.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/page03g.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/pages_brand.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/pages_components.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/pages_foundations.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/pages_specimens.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/probe-s04c.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/stage0.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/stage1.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/build/styles.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/00_setup_pages.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/01_frames.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/02_foundations.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/03_foundations_groups.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/04_styles.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/05_brand.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/06_brand_groups.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/07_components.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/08_components_groups.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/09_specimens.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-A/scripts/10_specimens_groups.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/B-ASSETS.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/B-canvas.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/_run.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/board_default.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/board_hero.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/board_interaction.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/board_zoom.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/boards_common.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/build_b.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/build-state.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/fig_b.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/info-B.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/make_manifest.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/patch_default.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/patch_default2.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/patch_manifest.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/patch_manifest2.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/patch_verify.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/patch3.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/patch4.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/patch5.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/tmp/board00.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/tmp/board01.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/tmp/board02.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/tmp/board03.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/tmp/effects.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/tmp/step0.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/tmp/step1.figcraft` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/build/verify_b.py` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/scripts/00_setup_pages.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/scripts/01_frames.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/scripts/02_canvas_default.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/scripts/03_zoom_states.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/scripts/04_interaction.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/scripts/05_hero.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/figcraft-B/scripts/06_effects.json` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/SHA256SUMS.txt` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/a11y/keyboard.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/a11y/reducedMotion.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/api/client.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/api/contract.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/api/health.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/api/http.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/api/ids.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/api/index.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/api/mock.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/api/refusals.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/App.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/assets/manifest.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/chat/ChatCore.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/flow/BeeEdge.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/flow/Canvas.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/flow/graph.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/flow/nodes.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/main.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/AgentHierarchy.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/AgentMemoryPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/AgentMessagesPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/AgentProfilesPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/AgentsPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/ApprovalOverlay.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/ControlBar.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/DataPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/DevGallery.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/Header.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/LibraryPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/ModelsPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/NavRail.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/RoutingPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/Scrubber.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/SettingsPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/StatePack.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/ToastRegion.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/panels/ToolActivityPanel.tsx` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/state/agentState.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/state/approval.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/state/context.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/state/metrics.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/state/store.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/state/strings.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/state/useAppReducedMotion.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/styles/index.css` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/styles/tokens.css` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/tokens/contrast.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/baseline/web-src/tokens/tokens.ts` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/cards-seeded.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/final-agents-top.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/final-history.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/final-live.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/final-portable.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/final-swarm.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/final-workspace.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/hierarchy-compact.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/hierarchy-fixture.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/hierarchy-fixture-2.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/hierarchy-fixture-3.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/hierarchy-radial.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/hierarchy-radial-2.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/keyed-live-reply.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/keyless-clean-ui.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/keyless-owner-links.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/keyless-owner-links-2.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/keyless-scoped-ui.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/keyless-window.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/live-diag.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/live-feel-1.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/live-feel-2.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/live-feel-3.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/live-feel-cdp.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/live-final-empty.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/live-final-v2.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/models-keyed.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/models-keyed-final.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |
| `docs/trace/figma-parity/tool-loop-live.png` | L6 | evidence | `doc:trace-figma-parity` | scope>=L6,L7 |

### A.docs/trace-live (40 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/trace/live/app-boot.png` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/app-boot-full.png` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/captures/w11/live-replay.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/captures/w11/live-streaming.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/captures/w12/CAP-S22-RECHECK/dom.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/captures/w12/CAP-S22-RECHECK/frames.jsonl` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/captures/w12/CAP-S22-RECHECK/manifest.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/captures/w12/CAP-S22-RECHECK/shot.png` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/current-ui.png` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/current-ui-left-rail.png` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/inspect.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/01-boot.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/02-skip-link.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/03-console.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/04-census.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/05-tabs.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/06-nav.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/07-zoom.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/08-chat-pin.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/09-empty-prompt.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/10-ws-handshake.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/11-colours.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/12-settings-status.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/13-contract-drift.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/14-file-tree.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/15-data-size.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/boot.png` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/live-resync-gap.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/live-resync-keyless.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/live-security-allowed-origin-option.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/live-security-bad-token.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/live-security-foreign-browser.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/live-security-foreign-browser.png` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/live-security-foreign-origin.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/live-security-missing-origin.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/live-security-positive.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/live-app/surface-census.png` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/README.md` | L6 | doc | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/ROUND8-LIVE.md` | L6 | doc | `doc:trace-live` | scope>=L6,L7 |
| `docs/trace/live/trace-capture.json` | L6 | evidence | `doc:trace-live` | scope>=L6,L7 |

### A.docs/trace-orca (23 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/trace/orca-wave/01-trace.md` | L6 | doc | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/02-weaver.md` | L6 | doc | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/03-forge.md` | L6 | doc | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/04-capability.md` | L6 | doc | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/05-driver.md` | L6 | doc | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/06-logic.md` | L6 | doc | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/baseline-app-window.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/baseline-edge.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-01-baseline.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-01-baseline-newbuild.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-02-chat-hidden.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-02-chat-hidden-cdp.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-02-chat-shown.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-02-return-live.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-03-composer-cleared.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-03-composer-typed.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-03-keyless-submit-refusal.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-04-history-replay-rev0.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-04-history-tab.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-04-return-live.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-05-hide-finished-off.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-05-workspace-before-filter.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |
| `docs/trace/orca-wave/shots/TD-07-agents.png` | L6 | evidence | `doc:trace-orca` | scope>=L6,L7 |

### A.docs/trace-provider (38 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/trace/provider/01-deepseek-capability-matrix.md` | L6 | doc | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/02-agent-runtime-wiring.md` | L6 | doc | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/03-ui-actions.md` | L6 | doc | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/04-audit.md` | L6 | doc | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/05-figcraft-ux-gaps.md` | L6 | doc | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/agents-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/agents-after-run-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/chat-cancelled-midstream-1280x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/chat-streaming-live-1280x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/chat-submitted-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/controls-1024x800.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/controls-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/empty-prompt-refusal-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/models-1024x800.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/models-1280x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/models-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/models-after-run-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/models-buildB-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/models-loading-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/models-provider-error-1280x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/settings-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/settings-focus-1280x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/workspace-1024x800.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/workspace-1280x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/workspace-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/workspace-after-run-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/workspace-buildB-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots/workspace-cancelled-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/agents-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/history-1024x800.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/history-1280x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/history-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/measure.json` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/measure-tabs.json` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/models-1024x800.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/models-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/swarm-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |
| `docs/trace/provider/shots-figcraft/workspace-1440x900.png` | L6 | evidence | `doc:trace-provider` | scope>=L6,L7 |

### A.docs/trace-root (10 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/trace/A11Y-CONTRAST-PLAN.md` | L6 | doc | `doc:trace-root` | scope>=L6,L7 |
| `docs/trace/capture-live-1.1.json` | L6 | evidence | `doc:trace-root` | scope>=L6,L7 |
| `docs/trace/capture-registry.json` | L6 | evidence | `doc:trace-root` | scope>=L6,L7 |
| `docs/trace/CAPTURE-REGISTRY.md` | L6 | doc | `doc:trace-root` | scope>=L6,L7 |
| `docs/trace/controls.json` | L6 | evidence | `doc:trace-root` | L7 |
| `docs/trace/events.json` | L6 | evidence | `doc:trace-root` | scope>=L6,L7 |
| `docs/trace/QA-HARNESS.md` | L6 | doc | `doc:trace-root` | L7 |
| `docs/trace/terminal-truth.md` | L6 | doc | `doc:trace-root` | scope>=L6,L7 |
| `docs/trace/traces.json` | L6 | evidence | `doc:trace-root` | scope>=L6,L7 |
| `docs/trace/TRACE-SCHEMA.md` | L6 | doc | `doc:trace-root` | scope>=L6,L7 |

### A.docs/trace-rounds (19 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/trace/round9/findings-register.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/round9/lead-plan.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/round9/recon-backend.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/round9/recon-ui.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/round9/skill-frontend.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/round9/skill-modernize.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/round9/skill-unslop.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/round9/truthseeker-precode.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round20-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round24-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round25-feature.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round26-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round28-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round30-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round32-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round34-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round36-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round38-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |
| `docs/trace/rounds/round40-bug-sweep.md` | L6 | doc | `doc:trace-rounds` | scope>=L6,L7 |

### A.docs/trace-ui-rebuild (85 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `docs/trace/ui-rebuild/01-shell-parity.md` | L6 | doc | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/02-card-parity.md` | L6 | doc | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/04-hmas-surface-parity.md` | L6 | doc | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/05-visual-qa.md` | L6 | doc | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/07-ui-stack.md` | L6 | doc | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/09-live-visual-qa.md` | L6 | doc | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/10-representation-audit.md` | L6 | doc | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/11-control-wiring.md` | L6 | doc | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-live-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-live-boot.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-live-node-focus-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-live-seeded-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-live-seeded-hide-finished-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-node-focus-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-seeded-hide-finished-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-workspace-empty.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-workspace-empty-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-workspace-empty-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-workspace-seeded-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-workspace-seeded-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-workspace-seeded-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1024x800-workspace-seeded-worker-selected-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1280x900-live-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1280x900-live-boot.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1280x900-workspace-empty.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1280x900-workspace-empty-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1280x900-workspace-empty-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-chat-hidden.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-chat-pin-toggled.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-library-details-open-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-live-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-live-boot.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-live-node-focus-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-live-replay-attempt.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-live-replay-banner-occlusion.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-live-seeded-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-live-seeded-hide-finished-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-live-template-refusal.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-node-focus-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-node-focus-ring-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-node-focus-ring-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-seeded-focus-branch-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-seeded-focus-branch-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-seeded-hide-finished-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-seeded-hide-finished-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-seeded-hide-finished-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-seeded-select-via-library-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-settings-diff-click.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-tab-history.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-tab-swarm.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-tab-swarm-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-tab-swarm-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-tab-swarm-live-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-toast-submit-refusal.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-toast-template-insert.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-toast-template-insert-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-agents.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-agents-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-agents-live-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-data.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-data-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-data-live-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-models.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-models-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-models-live-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-routing.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-routing-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-routing-live-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-settings.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-settings-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-settings-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-view-settings-live-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-workspace-empty.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-workspace-empty-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-workspace-empty-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-workspace-seeded-1520.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-workspace-seeded-live.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-workspace-seeded-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots/1440x900-workspace-seeded-worker-selected-newbuild.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots-live/1024x800.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots-live/1024x800-empty-state.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots-live/1280x900.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots-live/1440x900.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots-live/1440x900-empty-state.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |
| `docs/trace/ui-rebuild/shots-live/1440x900-reduced-motion.png` | L6 | evidence | `doc:trace-ui-rebuild` | scope>=L6,L7 |

### A.e2e/config (7 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `e2e/.gitignore` | L6 | config | `e2e:config:` | scope*L6 |
| `e2e/GATE-EXCLUSIONS.md` | L6 | doc | `e2e:config:GATE-EXCLUSIONS` | L6 |
| `e2e/package.json` | L6 | config | `e2e:config:package` | L6 |
| `e2e/package-lock.json` | L6 | config | `e2e:config:package-lock` | scope*L6 |
| `e2e/playwright.config.ts` | L6 | config | `e2e:config:playwright.config` | L6 |
| `e2e/README.md` | L6 | doc | `e2e:config:README` | L6 |
| `e2e/tsconfig.json` | L6 | config | `e2e:config:tsconfig` | scope*L6 |

### A.e2e/data (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `e2e/data/control-matrix.json` | L6 | data | `e2e:control-matrix-data` | L6 |

### A.e2e/helpers (8 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `e2e/helpers/app.ts` | L6 | source | `e2e-helper:app` | L6 |
| `e2e/helpers/contrast.ts` | L6 | source | `e2e-helper:contrast` | scope*L6 |
| `e2e/helpers/evidence.ts` | L6 | source | `e2e-helper:evidence` | L6 |
| `e2e/helpers/live-server.ts` | L6 | source | `e2e-helper:live-server` | L6 |
| `e2e/helpers/matrix.ts` | L6 | source | `e2e-helper:matrix` | L6 |
| `e2e/helpers/replay-harness.ts` | L6 | source | `e2e-helper:replay-harness` | L6 |
| `e2e/helpers/trace.ts` | L6 | source | `e2e-helper:trace` | L6 |
| `e2e/helpers/ws-tap.ts` | L6 | source | `e2e-helper:ws-tap` | L6 |

### A.e2e/live-harness (7 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `e2e/data-probe.mjs` | L6 | source | `e2e-live:data-probe` | L6 |
| `e2e/live-check2.mjs` | L6 | source | `e2e-live:live-check2` | L6 |
| `e2e/live-inspect.mjs` | L6 | source | `e2e-live:live-inspect` | L6 |
| `e2e/live-probe.mjs` | L6 | source | `e2e-live:live-probe` | L6 |
| `e2e/live-shot.mjs` | L6 | source | `e2e-live:live-shot` | L6 |
| `e2e/live-smoke.mjs` | L6 | source | `e2e-live:live-smoke` | L6 |
| `e2e/run-guarded.mjs` | L6 | source | `e2e-live:run-guarded` | L6 |

### A.e2e/specs (16 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `e2e/specs/a11y-gates.spec.ts` | L6 | test | `e2e:a11y-gates.spec` | L6 |
| `e2e/specs/approval-flow.spec.ts` | L6 | test | `e2e:approval-flow.spec` | L4,L6 |
| `e2e/specs/control-matrix.spec.ts` | L6 | test | `e2e:control-matrix.spec` | L6 |
| `e2e/specs/gui-gates.spec.ts` | L6 | test | `e2e:gui-gates.spec` | L6 |
| `e2e/specs/gui-residual.spec.ts` | L6 | test | `e2e:gui-residual.spec` | L6 |
| `e2e/specs/live-app.spec.ts` | L6 | test | `e2e:live-app.spec` | L6 |
| `e2e/specs/live-replay.spec.ts` | L6 | test | `e2e:live-replay.spec` | L6 |
| `e2e/specs/live-resync.spec.ts` | L6 | test | `e2e:live-resync.spec` | L6 |
| `e2e/specs/live-security.spec.ts` | L6 | test | `e2e:live-security.spec` | L4,L6 |
| `e2e/specs/live-streaming.spec.ts` | L6 | test | `e2e:live-streaming.spec` | L6 |
| `e2e/specs/live-tool-turn.spec.ts` | L6 | test | `e2e:live-tool-turn.spec` | L6 |
| `e2e/specs/live-trace-capture.spec.ts` | L6 | test | `e2e:live-trace-capture.spec` | L6 |
| `e2e/specs/replay-state.spec.ts` | L6 | test | `e2e:replay-state.spec` | L6 |
| `e2e/specs/tool-loop.spec.ts` | L6 | test | `e2e:tool-loop.spec` | L6 |
| `e2e/specs/trace-hops.spec.ts` | L6 | test | `e2e:trace-hops.spec` | L6 |
| `e2e/specs/ws-contract.spec.ts` | L6 | test | `e2e:ws-contract.spec` | L4,L6 |

### A.ledgers/ui-watchdog (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `ledgers/ui-watchdog.json` | L6 | data | `ledger:ui-watchdog` | L6 |
| `ledgers/ui-watchdog.md` | L6 | data | `ledger:ui-watchdog` | L6 |

### A.plans/foresight (17 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `plans/foresight/APPROVAL-SUSPENSION-PLAN.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/APPROVAL-UX-PLAN.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/CODE-APPROVAL-ENGINE.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/CODE-APPROVAL-UX.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/CODE-PROOF.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/CODE-SIDE-EFFECTS.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/COMPILED-LEDGER.md` | L7 | doc | `plan:foresight` | L7 |
| `plans/foresight/PROOF-RELEASE-PLAN.md` | L7 | doc | `plan:foresight` | L7 |
| `plans/foresight/RUN-1.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/RUN-2.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/RUN2-COMPILED-LEDGER.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/RUN2-ENGINE-CODE.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/RUN2-PROOF-CODE.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/RUN2-SIDE-EFFECT-CODE.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/RUN2-WEB-CODE.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/RUN-3.md` | L7 | doc | `plan:foresight` | scope>=L7 |
| `plans/foresight/SIDE-EFFECT-EXECUTOR-PLAN.md` | L7 | doc | `plan:foresight` | scope>=L7 |

### A.plans/planners (17 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `plans/planners/PLANNER-01/GATE-CY-1.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-01/GATE-CY-2.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-01/GATE-CY-3.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-01/PLANNER-01.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-02/GATE-CY-1.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-02/GATE-CY-2.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-02/GATE-CY-3.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-02/PLANNER-02.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-03/GATE-CY-1.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-03/GATE-CY-2.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-03/GATE-CY-3.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-03/PLANNER-03.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-04/GATE-CY-1.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-04/GATE-CY-2.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-04/GATE-CY-3.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-04/PLANNER-04.md` | L7 | doc | `plan:planners` | scope*L7 |
| `plans/planners/PLANNER-04/READLOG.md` | L7 | doc | `plan:planners` | scope*L7 |

### A.plans/root (3 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `plans/1.1-WAVE-1-RECOVERY.md` | L7 | doc | `plan:root` | scope*L7 |
| `plans/HIVEMIND-1.0-EXECUTION-PLAN.md` | L7 | doc | `plan:root` | scope*L7 |
| `plans/HIVEMIND-1.0-SEAT-ASSIGNMENTS.md` | L7 | doc | `plan:root` | scope*L7 |

### A.plans/seats (49 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `plans/seats/SEAT-01/EVIDENCE.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-01/GATE-CY-1.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-01/GATE-CY-2.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-01/GATE-CY-3.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-01/READLOG.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-01/REPORT.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-02/EVIDENCE.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-02/GATE-CY-1.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-02/GATE-CY-2.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-02/GATE-CY-3.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-02/READLOG.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-02/REPORT.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-03/EVIDENCE.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-03/GATE-CY-1.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-03/GATE-CY-2.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-03/GATE-CY-3.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-03/READLOG.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-03/REPORT.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-04/EVIDENCE.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-04/GATE-CY-1.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-04/GATE-CY-2.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-04/GATE-CY-3.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-04/READLOG.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-04/REPORT.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-05/EVIDENCE.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-05/GATE-CY-1.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-05/GATE-CY-2.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-05/GATE-CY-3.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-05/READLOG.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-05/REPORT.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-06/EVIDENCE.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-06/GATE-CY-1.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-06/GATE-CY-2.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-06/GATE-CY-3.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-06/READLOG.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-06/REPORT.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-06/tools/write_seat.py` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-07/EVIDENCE.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-07/GATE-CY-1.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-07/GATE-CY-2.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-07/GATE-CY-3.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-07/READLOG.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-07/REPORT.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-08/EVIDENCE.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-08/GATE-CY-1.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-08/GATE-CY-2.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-08/GATE-CY-3.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-08/READLOG.md` | L7 | doc | `plan:seats` | scope*L7 |
| `plans/seats/SEAT-08/REPORT.md` | L7 | doc | `plan:seats` | scope*L7 |

### A.plans/verification (5 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `plans/verification/FINAL-AUDIT.md` | L7 | doc | `plan:verification` | scope*L7 |
| `plans/verification/LEAD-BUGCHECK.md` | L7 | doc | `plan:verification` | scope*L7 |
| `plans/verification/LEAD-BUGCHECK-RUN2.md` | L7 | doc | `plan:verification` | scope*L7 |
| `plans/verification/RELEASE-1.1-AUDIT.md` | L7 | doc | `plan:verification` | scope*L7 |
| `plans/verification/RELEASE-GATE-AUDIT.md` | L7 | doc | `plan:verification` | scope*L7 |

### A.plans/waves (23 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `plans/waves/README.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W10-TOOL-LOOP-DECISION.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W11-CONTRACT.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W11-GATES.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W11-GUI-RESIDUAL.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W11-SUSPENSION-DESIGN.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W11-TOOL-ASSEMBLER.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W11-TRACE.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W11-UI-TOOL-ACTIVITY.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W12-INTERFACES.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W1-CORE-RECOVERY.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W2-PROVIDER-MATRIX.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W3-STREAMING.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W4-LIVE-HARNESS.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W4-REPLAY-PROOF.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W4-TRACE-CAPTURE.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W5B-UI-TESTS.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W5-PROMPT-ASSETS.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W7-RELEASE-ENGINEERING.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W8-DOCS-CHECK.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W8-DOCS-TRUTH.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W9-GUI-E2E.md` | L7 | doc | `plan:waves` | scope*L7 |
| `plans/waves/W9-TRACE-TRUTH.md` | L7 | doc | `plan:waves` | scope*L7 |

### A.root/docs (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `CHANGE-LEDGER-IMPL.md` | L7 | doc | `doc:root:CHANGE-LEDGER-IMPL` | L7 |
| `README.md` | L7 | doc | `doc:root:README` | L6,L7 |

### A.root/launcher (3 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `HiveMind.cmd` | L6 | source | `launcher:HiveMind` | L6 |
| `HiveMind-Agent.cmd` | L6 | source | `launcher:HiveMind-Agent` | L6 |
| `HiveMind-Agent.ps1` | L6 | source | `launcher:HiveMind-Agent` | L6 |

### A.root/repo (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `.gitignore` | L7 | config | `repo:gitignore` | - |

### A.root/toolchain (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `.cargo/config.toml` | L6 | config | `build:toolchain:config` | scope*L6 |
| `rust-toolchain.toml` | L6 | config | `build:toolchain:rust-toolchain` | L6 |

### A.runtime-test/db-fixtures (27 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `runtime-test/data/hiverouter.db` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data/hiverouter.db-shm` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data/hiverouter.db-wal` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data2/hiverouter.db` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data2/hiverouter.db-shm` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data2/hiverouter.db-wal` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data3/hiverouter.db` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data3/hiverouter.db-shm` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data3/hiverouter.db-wal` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data4/hiverouter.db` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data4/hiverouter.db-shm` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data4/hiverouter.db-wal` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data5/hiverouter.db` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data5/hiverouter.db-shm` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data5/hiverouter.db-wal` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data6/hiverouter.db` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data6/hiverouter.db-shm` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data6/hiverouter.db-wal` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data7/hiverouter.db` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data7/hiverouter.db-shm` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data7/hiverouter.db-wal` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data8/hiverouter.db` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data8/hiverouter.db-shm` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data8/hiverouter.db-wal` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data9/hiverouter.db` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data9/hiverouter.db-shm` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |
| `runtime-test/data9/hiverouter.db-wal` | L6 | data | `artifact:db-fixture` | scope*L6,L7 |

### A.runtime-test/logs (22 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `runtime-test/data4/snapshots/01a1b119-2f34-7228-974e-03b1c27856fc/0000000000.json` | L6 | artifact | `artifact:log:0000000000` | scope*L6,L7 |
| `runtime-test/data6/snapshots/01a1b11d-90f3-73b3-bca9-616a21cd2e54/0000000000.json` | L6 | artifact | `artifact:log:0000000000` | scope*L6,L7 |
| `runtime-test/data9/snapshots/01a1b16d-7530-75e4-b51e-83b6bc3fdcca/0000000000.json` | L6 | artifact | `artifact:log:0000000000` | scope*L6,L7 |
| `runtime-test/root.html` | L6 | artifact | `artifact:log:root` | L6 |
| `runtime-test/s5.err` | L6 | artifact | `artifact:log:s5` | scope*L6,L7 |
| `runtime-test/s5.out` | L6 | artifact | `artifact:log:s5` | scope*L6,L7 |
| `runtime-test/s6.err` | L6 | artifact | `artifact:log:s6` | scope*L6,L7 |
| `runtime-test/s6.out` | L6 | artifact | `artifact:log:s6` | scope*L6,L7 |
| `runtime-test/s7.err` | L6 | artifact | `artifact:log:s7` | scope*L6,L7 |
| `runtime-test/s7.out` | L6 | artifact | `artifact:log:s7` | scope*L6,L7 |
| `runtime-test/s8.err` | L6 | artifact | `artifact:log:s8` | scope*L6,L7 |
| `runtime-test/s8.out` | L6 | artifact | `artifact:log:s8` | scope*L6,L7 |
| `runtime-test/s9.err` | L6 | artifact | `artifact:log:s9` | scope*L6,L7 |
| `runtime-test/s9.out` | L6 | artifact | `artifact:log:s9` | scope*L6,L7 |
| `runtime-test/server.err.txt` | L6 | artifact | `artifact:log:server.err` | scope*L6,L7 |
| `runtime-test/server.out.txt` | L6 | artifact | `artifact:log:server.out` | L6 |
| `runtime-test/server2.err.txt` | L6 | artifact | `artifact:log:server2.err` | scope*L6,L7 |
| `runtime-test/server2.out.txt` | L6 | artifact | `artifact:log:server2.out` | L6 |
| `runtime-test/server3.err.txt` | L6 | artifact | `artifact:log:server3.err` | scope*L6,L7 |
| `runtime-test/server3.out.txt` | L6 | artifact | `artifact:log:server3.out` | L6 |
| `runtime-test/server4.err.txt` | L6 | artifact | `artifact:log:server4.err` | scope*L6,L7 |
| `runtime-test/server4.out.txt` | L6 | artifact | `artifact:log:server4.out` | L6 |

### A.runtime-test/packages (31 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `runtime-test/pkg-cli-final/CHECKSUMS.txt` | L6 | artifact | `artifact:package:CHECKSUMS` | scope*L6,L7 |
| `runtime-test/pkg-cli-final/data/hiverouter.db` | L6 | data | `artifact:package:hiverouter` | L6 |
| `runtime-test/pkg-cli-final/data/hiverouter.db-shm` | L6 | data | `artifact:package:hiverouter` | scope*L6,L7 |
| `runtime-test/pkg-cli-final/data/hiverouter.db-wal` | L6 | data | `artifact:package:hiverouter` | scope*L6,L7 |
| `runtime-test/pkg-cli-final/hiverouter.exe` | L6 | binary | `artifact:package:hiverouter` | scope*L6,L7 |
| `runtime-test/pkg-cli-final/hiverouter-cli.exe` | L6 | binary | `artifact:package:hiverouter-cli` | scope*L6,L7 |
| `runtime-test/pkg-cli-final/README.md` | L6 | doc | `artifact:package:README` | scope*L6,L7 |
| `runtime-test/pkg-cli-final/RUNBOOK.md` | L6 | doc | `artifact:package:RUNBOOK` | scope*L6,L7 |
| `runtime-test/pkg-final/CHECKSUMS.txt` | L6 | artifact | `artifact:package:CHECKSUMS` | L6 |
| `runtime-test/pkg-final/data/hiverouter.db` | L6 | data | `artifact:package:hiverouter` | L6 |
| `runtime-test/pkg-final/data/hiverouter.db-shm` | L6 | data | `artifact:package:hiverouter` | scope>=L6,L7 |
| `runtime-test/pkg-final/data/hiverouter.db-wal` | L6 | data | `artifact:package:hiverouter` | scope>=L6,L7 |
| `runtime-test/pkg-final/err.txt` | L6 | artifact | `artifact:package:err` | scope>=L6,L7 |
| `runtime-test/pkg-final/hiverouter.exe` | L6 | binary | `artifact:package:hiverouter` | scope>=L6,L7 |
| `runtime-test/pkg-final/out.txt` | L6 | artifact | `artifact:package:out` | scope>=L6,L7 |
| `runtime-test/pkg-final/README.md` | L6 | doc | `artifact:package:README` | scope>=L6,L7 |
| `runtime-test/pkg-final/RUNBOOK.md` | L6 | doc | `artifact:package:RUNBOOK` | scope>=L6,L7 |
| `runtime-test/pkg-final/smoke.err` | L6 | artifact | `artifact:package:smoke` | scope>=L6,L7 |
| `runtime-test/pkg-final/smoke.out` | L6 | artifact | `artifact:package:smoke` | L6 |
| `runtime-test/pkg-final/smoke-data/hiverouter.db` | L6 | data | `artifact:package:hiverouter` | scope>=L6,L7 |
| `runtime-test/pkg-final/smoke-data/hiverouter.db-shm` | L6 | data | `artifact:package:hiverouter` | scope>=L6,L7 |
| `runtime-test/pkg-final/smoke-data/hiverouter.db-wal` | L6 | data | `artifact:package:hiverouter` | scope>=L6,L7 |
| `runtime-test/pkg-smoke/CHECKSUMS.txt` | L6 | artifact | `artifact:package:CHECKSUMS` | scope*L6,L7 |
| `runtime-test/pkg-smoke/data/hiverouter.db` | L6 | data | `artifact:package:hiverouter` | L6 |
| `runtime-test/pkg-smoke/data/hiverouter.db-shm` | L6 | data | `artifact:package:hiverouter` | scope*L6,L7 |
| `runtime-test/pkg-smoke/data/hiverouter.db-wal` | L6 | data | `artifact:package:hiverouter` | scope*L6,L7 |
| `runtime-test/pkg-smoke/err.txt` | L6 | artifact | `artifact:package:err` | scope*L6,L7 |
| `runtime-test/pkg-smoke/hiverouter.exe` | L6 | binary | `artifact:package:hiverouter` | scope*L6,L7 |
| `runtime-test/pkg-smoke/out.txt` | L6 | artifact | `artifact:package:out` | scope*L6,L7 |
| `runtime-test/pkg-smoke/README.md` | L6 | doc | `artifact:package:README` | scope*L6,L7 |
| `runtime-test/pkg-smoke/RUNBOOK.md` | L6 | doc | `artifact:package:RUNBOOK` | scope*L6,L7 |

### A.runtime-test/workspace (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `runtime-test/workspace/note.txt` | L6 | data | `artifact:workspace:note` | scope*L6,L7 |
| `runtime-test/workspace/README.md` | L6 | data | `artifact:workspace:README` | scope*L6,L7 |

### A.skills/hivemind (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `skills/hivemind/SKILL.md` | L7 | doc | `doc:skill` | scope*L6 |

### A.tools/binaries (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `tools/bin/hiverouter.exe` | L6 | binary | `launcher:binary:hiverouter` | L6 |
| `tools/bin/hiverouter-cli.exe` | L6 | binary | `launcher:binary:hiverouter-cli` | scope>=L6,L7 |

### A.tools/cc-test (4 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `tools/cc-test/r.exe` | L6 | binary | `build-probe:r` | scope*L6,L7 |
| `tools/cc-test/r.rs` | L6 | source | `build-probe:r` | scope*L6,L7 |
| `tools/cc-test/t.c` | L6 | source | `build-probe:t` | scope*L6,L7 |
| `tools/cc-test/t.exe` | L6 | binary | `build-probe:t` | scope*L6,L7 |

### A.tools/cli (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `tools/hivemind-cli.mjs` | L4 | source | `cli:node` | L4,L6 |

### A.tools/desktop-launcher (3 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `tools/desktop-launcher/HiveMind.exe` | L6 | binary | `launcher:HiveMind` | L7 |
| `tools/desktop-launcher/HiveMindLauncher.cs` | L6 | source | `launcher:HiveMindLauncher` | L6,L7 |
| `tools/desktop-launcher/README.md` | L6 | doc | `launcher:README` | L6 |

### A.tools/launcher (7 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `tools/hivemind.cmd` | L6 | source | `launcher:hivemind` | L6 |
| `tools/hivemind.ps1` | L6 | source | `launcher:hivemind` | L6 |
| `tools/runbook.md` | L6 | doc | `launcher:runbook` | L6 |
| `tools/set-provider-key.ps1` | L6 | source | `launcher:set-provider-key` | L6,L7 |
| `tools/set-provider-profile.ps1` | L6 | source | `launcher:set-provider-profile` | L6 |
| `tools/start-hivemind.ps1` | L6 | source | `launcher:start-hivemind` | L4,L6 |
| `tools/test.ps1` | L6 | source | `launcher:test` | L6 |

### A.tools/qa (13 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `tools/qa/apply_wp01.py` | L6 | config | `qa:apply_wp01` | L6 |
| `tools/qa/apply-live-trace.mjs` | L6 | source | `qa:apply-live-trace` | L6 |
| `tools/qa/build-trace-seeds.mjs` | L6 | source | `qa:build-trace-seeds` | L6 |
| `tools/qa/check-docs-truth.mjs` | L6 | source | `qa:check-docs-truth` | L6 |
| `tools/qa/check-env.ps1` | L6 | source | `qa:check-env` | L6 |
| `tools/qa/check-seal-topology.mjs` | L6 | source | `qa:check-seal-topology` | L6 |
| `tools/qa/figcraft-assets.mjs` | L6 | source | `qa:figcraft-assets` | L6 |
| `tools/qa/README.md` | L6 | config | `qa:README` | L6 |
| `tools/qa/run-gates.ps1` | L6 | source | `qa:run-gates` | L6 |
| `tools/qa/tests/seal-topology.test.mjs` | L6 | source | `qa:seal-topology.test` | L6 |
| `tools/qa/tests/trace-tools.test.mjs` | L6 | source | `qa:trace-tools.test` | L6 |
| `tools/qa/ui-e2e.mjs` | L6 | source | `qa:ui-e2e` | L6 |
| `tools/qa/verify-harness.mjs` | L6 | source | `qa:verify-harness` | L6 |

### A.tools/release (4 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `tools/build_assets_index.py` | L6 | source | `release:build_assets_index` | L6 |
| `tools/build-desktop.ps1` | L6 | source | `release:build-desktop` | L6 |
| `tools/build-release.ps1` | L6 | source | `release:build-release` | L6 |
| `tools/release-manifest.json` | L6 | config | `release:release-manifest` | L6,L7 |

### A.web/a11y (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/a11y/keyboard.ts` | L5 | source | `ui:a11y:keyboard` | L5 |
| `web/src/a11y/reducedMotion.ts` | L5 | source | `ui:a11y:reducedMotion` | L5 |

### A.web/api (9 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/api/client.ts` | L4 | source | `api-client:client` | L4,L5 |
| `web/src/api/contract.ts` | L4 | source | `api-client:contract` | L4,L5 |
| `web/src/api/failure.ts` | L4 | source | `api-client:failure` | L4,L5 |
| `web/src/api/health.ts` | L4 | source | `api-client:health` | L5 |
| `web/src/api/http.ts` | L4 | source | `api-client:http` | L5 |
| `web/src/api/ids.ts` | L4 | source | `api-client:ids` | L5 |
| `web/src/api/index.ts` | L4 | source | `api-client:index` | L5 |
| `web/src/api/mock.ts` | L4 | source | `api-client:mock` | L5,L7 |
| `web/src/api/refusals.ts` | L4 | source | `api-client:refusals` | L5 |

### A.web/assets (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/assets/manifest.ts` | L5 | source | `ui:assets` | L5 |

### A.web/build (5 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/index.html` | L5 | config | `web:build:index` | - |
| `web/package.json` | L5 | config | `web:build:package` | L5,L6,L7 |
| `web/package-lock.json` | L5 | config | `web:build:package-lock` | - |
| `web/tsconfig.json` | L5 | config | `web:build:tsconfig` | L6 |
| `web/vite.config.ts` | L5 | config | `web:build:vite.config` | L5,L6 |

### A.web/chat (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/chat/ChatCore.tsx` | L5 | source | `ui:chat:ChatCore` | L5 |
| `web/src/chat/ChatMotion.tsx` | L5 | source | `ui:chat:ChatMotion` | L5 |

### A.web/figcraft (3 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/figcraft/catalog.ts` | L5 | source | `ui:figcraft:catalog` | L5 |
| `web/src/figcraft/FigCraftGallery.tsx` | L5 | source | `ui:figcraft:FigCraftGallery` | L5 |
| `web/src/figcraft/types.ts` | L5 | source | `ui:figcraft:types` | L5 |

### A.web/figcraft-assets (22 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/public/figcraft/a-brand.svg` | L5 | asset | `ui:figcraft-asset:a-brand` | - |
| `web/public/figcraft/a-components.svg` | L5 | asset | `ui:figcraft-asset:a-components` | - |
| `web/public/figcraft/a-foundations.svg` | L5 | asset | `ui:figcraft-asset:a-foundations` | - |
| `web/public/figcraft/a-specimens.svg` | L5 | asset | `ui:figcraft-asset:a-specimens` | - |
| `web/public/figcraft/b-canvas-default.svg` | L5 | asset | `ui:figcraft-asset:b-canvas-default` | - |
| `web/public/figcraft/b-hero.svg` | L5 | asset | `ui:figcraft-asset:b-hero` | - |
| `web/public/figcraft/b-interaction.svg` | L5 | asset | `ui:figcraft-asset:b-interaction` | - |
| `web/public/figcraft/b-zoom-states.svg` | L5 | asset | `ui:figcraft-asset:b-zoom-states` | - |
| `web/public/figcraft/hivemind-agent-cards.svg` | L5 | asset | `ui:figcraft-asset:hivemind-agent-cards` | - |
| `web/public/figcraft/hivemind-brand-lockups.svg` | L5 | asset | `ui:figcraft-asset:hivemind-brand-lockups` | - |
| `web/public/figcraft/hivemind-colour.svg` | L5 | asset | `ui:figcraft-asset:hivemind-colour` | - |
| `web/public/figcraft/hivemind-controls-and-hud.svg` | L5 | asset | `ui:figcraft-asset:hivemind-controls-and-hud` | - |
| `web/public/figcraft/hivemind-cover.svg` | L5 | asset | `ui:figcraft-asset:hivemind-cover` | - |
| `web/public/figcraft/hivemind-hmas-governance.svg` | L5 | asset | `ui:figcraft-asset:hivemind-hmas-governance` | - |
| `web/public/figcraft/hivemind-hmas-hierarchy.svg` | L5 | asset | `ui:figcraft-asset:hivemind-hmas-hierarchy` | - |
| `web/public/figcraft/hivemind-motion-and-live.svg` | L5 | asset | `ui:figcraft-asset:hivemind-motion-and-live` | - |
| `web/public/figcraft/hivemind-run-detail-and-files.svg` | L5 | asset | `ui:figcraft-asset:hivemind-run-detail-and-files` | - |
| `web/public/figcraft/hivemind-space-and-depth.svg` | L5 | asset | `ui:figcraft-asset:hivemind-space-and-depth` | - |
| `web/public/figcraft/hivemind-spatial-canvas.svg` | L5 | asset | `ui:figcraft-asset:hivemind-spatial-canvas` | - |
| `web/public/figcraft/hivemind-states-and-refusals.svg` | L5 | asset | `ui:figcraft-asset:hivemind-states-and-refusals` | - |
| `web/public/figcraft/hivemind-typography.svg` | L5 | asset | `ui:figcraft-asset:hivemind-typography` | - |
| `web/public/figcraft/SOURCES.json` | L5 | asset | `ui:figcraft-asset:SOURCES` | - |

### A.web/flow (6 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/flow/BeeEdge.tsx` | L5 | source | `ui:flow:BeeEdge` | L5 |
| `web/src/flow/Canvas.tsx` | L5 | source | `ui:flow:Canvas` | L5 |
| `web/src/flow/graph.ts` | L5 | source | `ui:flow:graph` | L5 |
| `web/src/flow/HiveField.tsx` | L5 | source | `ui:flow:HiveField` | L5 |
| `web/src/flow/nodes.tsx` | L5 | source | `ui:flow:nodes` | L5 |
| `web/src/flow/RemoteLane.tsx` | L5 | source | `ui:flow:RemoteLane` | L4,L5 |

### A.web/panels (36 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/panels/A2AInboxPanel.tsx` | L5 | source | `ui:panel:A2AInboxPanel` | L4,L5 |
| `web/src/panels/AgentHierarchy.tsx` | L5 | source | `ui:panel:AgentHierarchy` | L5 |
| `web/src/panels/AgentMemoryPanel.tsx` | L5 | source | `ui:panel:AgentMemoryPanel` | scope>=L4,L5 |
| `web/src/panels/AgentMessagesPanel.tsx` | L5 | source | `ui:panel:AgentMessagesPanel` | L5 |
| `web/src/panels/AgentProfilesPanel.tsx` | L5 | source | `ui:panel:AgentProfilesPanel` | scope>=L4,L5 |
| `web/src/panels/AgentRosterPanel.tsx` | L5 | source | `ui:panel:AgentRosterPanel` | L5 |
| `web/src/panels/AgentsPanel.tsx` | L5 | source | `ui:panel:AgentsPanel` | L5 |
| `web/src/panels/ApprovalOverlay.tsx` | L5 | source | `ui:panel:ApprovalOverlay` | L5 |
| `web/src/panels/ConfirmDialog.tsx` | L5 | source | `ui:panel:ConfirmDialog` | L5 |
| `web/src/panels/ControlBar.tsx` | L5 | source | `ui:panel:ControlBar` | L5 |
| `web/src/panels/DaemonPanel.tsx` | L5 | source | `ui:panel:DaemonPanel` | L4,L5 |
| `web/src/panels/DataPanel.tsx` | L5 | source | `ui:panel:DataPanel` | L5 |
| `web/src/panels/DevGallery.tsx` | L5 | source | `ui:panel:DevGallery` | L5 |
| `web/src/panels/DiagnosticsPanel.tsx` | L5 | source | `ui:panel:DiagnosticsPanel` | L4,L5 |
| `web/src/panels/FigCraftPanel.tsx` | L5 | source | `ui:panel:FigCraftPanel` | L5 |
| `web/src/panels/Header.tsx` | L5 | source | `ui:panel:Header` | L5 |
| `web/src/panels/HistoryPanel.tsx` | L5 | source | `ui:panel:HistoryPanel` | L5 |
| `web/src/panels/HudBadge.tsx` | L5 | source | `ui:panel:HudBadge` | L5 |
| `web/src/panels/LessonsPanel.tsx` | L5 | source | `ui:panel:LessonsPanel` | L5 |
| `web/src/panels/LibraryPanel.tsx` | L5 | source | `ui:panel:LibraryPanel` | L5 |
| `web/src/panels/MemoryBanksPanel.tsx` | L5 | source | `ui:panel:MemoryBanksPanel` | L4,L5 |
| `web/src/panels/ModelsPanel.tsx` | L5 | source | `ui:panel:ModelsPanel` | scope>=L4,L5 |
| `web/src/panels/NavIcon.tsx` | L5 | source | `ui:panel:NavIcon` | L5 |
| `web/src/panels/NavRail.tsx` | L5 | source | `ui:panel:NavRail` | L5 |
| `web/src/panels/ProjectsPanel.tsx` | L5 | source | `ui:panel:ProjectsPanel` | L5 |
| `web/src/panels/ProposalsPanel.tsx` | L5 | source | `ui:panel:ProposalsPanel` | L4,L5,L7 |
| `web/src/panels/RemotePanel.tsx` | L5 | source | `ui:panel:RemotePanel` | L4,L5 |
| `web/src/panels/RoutingPanel.tsx` | L5 | source | `ui:panel:RoutingPanel` | scope>=L4,L5 |
| `web/src/panels/RunDetailPanel.tsx` | L5 | source | `ui:panel:RunDetailPanel` | L4,L5 |
| `web/src/panels/RunTranscriptPanel.tsx` | L5 | source | `ui:panel:RunTranscriptPanel` | L5 |
| `web/src/panels/Scrubber.tsx` | L5 | source | `ui:panel:Scrubber` | L5 |
| `web/src/panels/SettingsPanel.tsx` | L5 | source | `ui:panel:SettingsPanel` | L5 |
| `web/src/panels/StatePack.tsx` | L5 | source | `ui:panel:StatePack` | L5 |
| `web/src/panels/SwarmPanel.tsx` | L5 | source | `ui:panel:SwarmPanel` | L4,L5 |
| `web/src/panels/ToastRegion.tsx` | L5 | source | `ui:panel:ToastRegion` | L5 |
| `web/src/panels/ToolActivityPanel.tsx` | L5 | source | `ui:panel:ToolActivityPanel` | L5 |

### A.web/shell (3 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/App.tsx` | L5 | source | `ui:shell:App` | L5 |
| `web/src/fontsource.d.ts` | L5 | source | `ui:shell:fontsource.d` | L5 |
| `web/src/main.tsx` | L5 | source | `ui:shell:main` | L4,L5 |

### A.web/splash (1 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/splash/HiveMindSplash.tsx` | L5 | source | `ui:splash` | L5 |

### A.web/state (9 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/state/agentState.ts` | L5 | source | `ui:state:agentState` | L5 |
| `web/src/state/approval.ts` | L5 | source | `ui:state:approval` | L5 |
| `web/src/state/chatIntents.ts` | L5 | source | `ui:state:chatIntents` | L5 |
| `web/src/state/context.ts` | L5 | source | `ui:state:context` | L5 |
| `web/src/state/metrics.ts` | L5 | source | `ui:state:metrics` | L5 |
| `web/src/state/spawnMatrix.ts` | L5 | source | `ui:state:spawnMatrix` | L5 |
| `web/src/state/store.ts` | L5 | source | `ui:state:store` | L5 |
| `web/src/state/strings.ts` | L5 | source | `ui:state:strings` | L5 |
| `web/src/state/useAppReducedMotion.ts` | L5 | source | `ui:state:useAppReducedMotion` | L5 |

### A.web/styles (4 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/styles/index.css` | L5 | source | `ui:style:index` | L5 |
| `web/src/styles/motion.css` | L5 | source | `ui:style:motion` | L5 |
| `web/src/styles/splash.css` | L5 | source | `ui:style:splash` | L5 |
| `web/src/styles/tokens.css` | L5 | source | `ui:style:tokens` | L5 |

### A.web/tests (32 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/tests/agents_spawn_matrix.test.ts` | L5 | test | `webtest:agents_spawn_matrix.test` | scope>=L5,L6 |
| `web/tests/agentState.test.ts` | L5 | test | `webtest:agentState.test` | scope>=L5,L6 |
| `web/tests/agentsToolActivityMount.test.ts` | L5 | test | `webtest:agentsToolActivityMount.test` | L5,L6 |
| `web/tests/apiBoundary.test.ts` | L5 | test | `webtest:apiBoundary.test` | L4,L5 |
| `web/tests/apiClient.test.ts` | L5 | test | `webtest:apiClient.test` | L4 |
| `web/tests/apiLifecycle.test.ts` | L5 | test | `webtest:apiLifecycle.test` | L4 |
| `web/tests/approval.integration.test.ts` | L5 | test | `webtest:approval.integration.test` | scope>=L5,L6 |
| `web/tests/approval.state.test.ts` | L5 | test | `webtest:approval.state.test` | scope>=L5,L6 |
| `web/tests/chatMotion.test.ts` | L5 | test | `webtest:chatMotion.test` | L5 |
| `web/tests/client.strictmode.test.ts` | L5 | test | `webtest:client.strictmode.test` | scope>=L5,L6 |
| `web/tests/contract.test.ts` | L5 | test | `webtest:contract.test` | L4,L5,L6 |
| `web/tests/designParity.test.ts` | L5 | test | `webtest:designParity.test` | L5,L6 |
| `web/tests/failure.test.ts` | L5 | test | `webtest:failure.test` | L4,L5 |
| `web/tests/graph.replay.test.ts` | L5 | test | `webtest:graph.replay.test` | L5 |
| `web/tests/health.test.ts` | L5 | test | `webtest:health.test` | L4,L5 |
| `web/tests/helpers/fakeUiApi.ts` | L5 | test | `webtest:fakeUiApi` | L5 |
| `web/tests/keyboard.test.ts` | L5 | test | `webtest:keyboard.test` | L5 |
| `web/tests/layout_hive.test.ts` | L5 | test | `webtest:layout_hive.test` | L5 |
| `web/tests/lessons.test.ts` | L5 | test | `webtest:lessons.test` | L4,L5 |
| `web/tests/metrics.test.ts` | L5 | test | `webtest:metrics.test` | L5 |
| `web/tests/noHardcoded.test.ts` | L5 | test | `webtest:noHardcoded.test` | L5,L6 |
| `web/tests/proposals.test.ts` | L5 | test | `webtest:proposals.test` | L4,L5 |
| `web/tests/refusalParity.test.ts` | L5 | test | `webtest:refusalParity.test` | L4,L6 |
| `web/tests/state.parity.test.ts` | L5 | test | `webtest:state.parity.test` | L5,L6 |
| `web/tests/store.spawn-replay.test.ts` | L5 | test | `webtest:store.spawn-replay.test` | scope>=L5,L6 |
| `web/tests/store.strictmode.test.ts` | L5 | test | `webtest:store.strictmode.test` | scope>=L5,L6 |
| `web/tests/store.test.ts` | L5 | test | `webtest:store.test` | L5 |
| `web/tests/streamingRender.test.ts` | L5 | test | `webtest:streamingRender.test` | scope>=L5,L6 |
| `web/tests/tokens.test.ts` | L5 | test | `webtest:tokens.test` | L5 |
| `web/tests/toolActivity.integration.test.ts` | L5 | test | `webtest:toolActivity.integration.test` | scope>=L5,L6 |
| `web/tests/toolActivity.store.test.ts` | L5 | test | `webtest:toolActivity.store.test` | scope>=L5,L6 |
| `web/tests/toolActivityPanels.test.ts` | L5 | test | `webtest:toolActivityPanels.test` | L6 |

### A.web/tokens (2 paths)

| path | lane | kind | feature | coverage |
|---|---|---|---|---|
| `web/src/tokens/contrast.ts` | L5 | source | `ui:tokens:contrast` | L5 |
| `web/src/tokens/tokens.ts` | L5 | source | `ui:tokens:tokens` | L5 |

