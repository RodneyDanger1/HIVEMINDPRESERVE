# L3 — State, Persistence & Memory Lane

**Repository:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Lane:** database schema/migrations · stores · task/run entities · memory banks/entries/journal/snapshots · A2A message lifecycle · remotes · daemons · approvals persistence · usage · recovery state
**Author:** Codebase Analyst Engineer 3/7 (HiveMind)
**Mode:** READ-ONLY analysis. No source, config, test, or migration was modified; no server started; no provider called; no key printed. The only file written is this report.
**Evidence rules:** every claim is tagged **VERIFIED** (read from the cited file/lines), **UNVERIFIED** (asserted but not executed/observed), or **PROPOSED** (not in the codebase). Line numbers are from the current working tree.

> **Test-execution status:** this lane did **not** run `cargo test`, `cargo build`, migrations, or the server (read-only mandate). Therefore "test X proves Y" below means the test *source* asserts Y (**VERIFIED as source**), not that the test passed (**UNVERIFIED at runtime**).

---

## 1. Executive verdict

1. **Schema discipline is good for forward migrations.** 23 ordered, append-only migrations run inside one transaction each, with a `VACUUM INTO` + `PRAGMA integrity_check` backup before a pending upgrade (`db.rs:1011-1099`). Every migration carries a `down` (`db.rs:19-30`, `32-858`). **VERIFIED.**
2. **Migration safety is not uniform.** Backups happen only on the *up* path: `migrate_down_to` performs destructive down-SQL with **no backup** (`db.rs:1106-1135`). v19 can permanently block boot on a DB that holds two `hyper` tasks in one run (`db.rs:573-575`, `637-638`), and each failed boot writes another `.bak` (`db.rs:1025-1027`, `1051-1099`). **VERIFIED.**
3. **Model-facing memory *read* isolation is real and tested** — bounded bank set, sibling exclusion, trust filter, run-scoped bank read (`store.rs:2181-2229`, `2299-2342`; `tests/mem_isolation.rs:76-115`; `store.rs:3502-3525`). **VERIFIED.**
4. **Model-facing memory *write* control is not wired.** The strict, caller-bank, vocabulary-validated writer `record_memory_entry` (`store.rs:1513-1619`) has **zero production callers** — only `tests/mem_isolation.rs`. The engine writes through the permissive `record_agent_memory_scoped` (`execution.rs:1955-1971`, `5572-5578`; `store.rs:1329-1494`), which accepts an arbitrary `scope_id`, hard-codes `kind='observation'`/`trust='unverified'`, and is **not transactional**. **VERIFIED.**
5. **Governance/lesson records can be orphaned** in at least three ways: the clear-guard scans only the newest 256 run/hive entries and only two key prefixes (`http.rs:1497-1507`), the applied lesson lives in `evolution/LESSONS.jsonl` with no DB FK/transaction (`http.rs:2692-2694`, `2818-2841`), and `clear_run_memory` deletes the only rollback key while leaving the lesson file (`store.rs:1781-1785`; `http.rs:2870-2877`). **VERIFIED.**
6. **Retention/erasure claims are false in one operator path.** `DELETE /api/agents/memory/{agent_key}` deletes only the flat `agent_memory` table (`store.rs:2626-2637`); the authoritative `memory_entries` bank row remains model-readable and cannot even be selected by agent key (no such column, `db.rs:411-431`). **VERIFIED.**

### Direct answers to the mandate

| Question | Answer | Basis |
|---|---|---|
| Can governance/lesson records be orphaned? | **Yes.** File-vs-DB split, bounded prefix guard, and pre-clear evidence loss. | §6.8 |
| Are migrations safe / additive / reversible? | Safe-with-caveats; forward-additive except v11 rebuild; schema-reversible; **data-destructive** and **down has no backup**; v19 is a hard-boot risk. | §14 |
| Is memory access truly controlled? | **Partly.** Model↔model read isolation: yes. Model→memory write: no (unused strict path). Operator→memory read: no (GET routes unauthenticated beyond Host/launcher token). Erasure: no. | §15 |

---

## 2. Coverage ledger (every covered file)

| File | Lines read | What this lane verified |
|---|---|---|
| `backend/crates/hiverouter-core/src/db.rs` | 1-1806 (full) | Migration struct/engine, all 23 migrations + down, pragmas, backup, down path, tool-turn persistence, 11 unit tests |
| `backend/crates/hiverouter-core/src/store.rs` | 1-3653 (full) | `RunStore` types, journal chain, all ~60 public methods, 4 private helpers, 30 unit tests |
| `backend/crates/hiverouter-core/src/entities.rs` | 1-99 (full) | `Run`, `TaskRecord`, `ChatRole`, `ChatMessage` |
| `backend/crates/hiverouter-core/src/history.rs` | 1-116 (full) | `HistoryReadModel`, `TimeTravel`, `state_at` |
| `backend/crates/hiverouter-core/src/snapshot.rs` | 1-355 (full) | v1/v2 schemas, atomic write, prune, index, migrations, `capture_scoped` |
| `backend/crates/hiverouter-core/src/recovery.rs` | 1-974 (full) | Boot recovery, settlement, graph repair, revision selection, 10 tests |
| `backend/crates/hiverouter-core/src/daemon.rs` | 1-837 (full) | Daemon plane, script allowlist, tick/report, 7 tests |
| `backend/crates/hiverouter-core/src/remote.rs` | 1-1717 (full) | Remote/device/grant/action/audit, approval sweep, boot reconcile, 7 tests |
| `backend/crates/hiverouter-core/src/remote_supervisor.rs` | 1-249 (full) | Supervisor lane, supervisor state row, backoff, 2 tests |
| `backend/crates/hiverouter-core/src/approval_wait.rs` | 1-174 (full) | In-process waiters only (no DB state) |
| `backend/crates/hiverouter-core/src/run_state.rs` | 56-145, grep of apply/restart tests | Run/task state sets, terminal sets, `Restart` edge |
| `backend/crates/hiverouter-core/src/tools_bridge.rs` | 80-219 (approval surface) | Approval table migrate, boot reaper, expire/cancel/decision/count |
| `backend/crates/hiverouter-core/src/execution.rs` | 1305-1344, 1732-1799, 1840-1971, 4100-4169; grep of memory/governance call sites | Prompt memory injection, snapshot revision, memory writes, usage checkpoints |
| `backend/crates/hiverouter-core/src/http.rs` | 175-263, 596-827, 1440-1730, 2340-2852, 2856-3234; grep of all routes | Router, auth middleware, memory/a2a/remote/daemon/approval/evolution handlers |
| `backend/crates/hiverouter-core/src/main.rs` | 55-194 | Boot order: open → migrate → remote reconcile → recovery → bind → supervisor |
| `backend/crates/hiverouter-core/src/lib.rs` | 1-73 (full) | Module graph (there is **no** `memory.rs`; memory lives in `store.rs`) |
| `backend/crates/hiverouter-tools/src/approval.rs` | 120-449 | `approvals` table DDL/indexes (outside the migration engine), insert/resolve/session grant |
| `backend/crates/hiverouter-contract/src/*` | `is_terminal` only (grep) | `Killed` is terminal for the contract |
| `tests/mem_isolation.rs` | 1-247 (full) | 4 tests: fail-closed identity, cross-run bank refusal, validated write, vocabulary refusal |
| `tests/snapshot_history.rs` | 1-290 (full) | Snapshot v1↔v2 round-trip, corruption refusal, history HTTP API, prune |
| `tests/daemon_tick_degraded.rs` | 1-98 (full) | Degraded tick, no fake alive, no task row |
| `tests/remote_control_plane.rs` | 1-382 (full) | Post-request revocation/lease/device races, approval sweep, audit, boot reconcile |
| `tests/kill_restore.rs` | 1-117 (full) | Hard-kill writer → recovery interrupts + restores; corrupt-at-boot |
| `tests/chat_store.rs` | 1-149 (full) | Chat/run/task persistence, receipts, v2 down/up |
| `tests/http_boot.rs` | 110-489, 700-806, 1180-1269 (selected) | Memory journal/summary/banks/entries/clear/context, A2A pending+answer, role memory clear |
| `tests/approval_ledger.rs` | 1-120 (partial) | One ledger row per committed decision; replay writes none |

### Files/tests explicitly **not** covered (gaps)

- `tests/recovery_gates.rs`, `tests/recovery_multi_run.rs`, `tests/hyperagent_governance.rs`, `tests/suspension.rs`, `tests/run_lifecycle.rs`, `tests/ws_transport.rs`, `tests/tool_loop_*`, `tests/scale_s*`, `tests/graph_authority.rs`, `tests/proposal_authorization.rs` — **not read**. Multi-run recovery and governance-under-suspension therefore remain **UNVERIFIED** here.
- `tests/http_boot.rs:490-700`, `806-1180`, `1270-1408` — remote/daemon HTTP bodies only skimmed via grep; route existence VERIFIED, exact assertions partially UNVERIFIED.
- `backend/crates/hiverouter-core/src/execution.rs` beyond the cited windows; `http.rs` 1-175, 405-595, 828-1440, 1730-2340, 3235-3715; `ws.rs`; `graph.rs`; `bus.rs`; `config.rs` — not read.
- `backend/crates/hiverouter-tools/src/approval.rs:450-1615` (remaining functions/tests) — not read.
- No live DB file, WAL, `.bak`, `snapshots/` tree, or `evolution/LESSONS.jsonl` was inspected.
- No migration was executed against a real DB; no test/build was run.

---

## 3. Migration inventory (v1-v23)

All rows: `MIGRATIONS` is `pub const` in `db.rs:31-858`; each `up` executes inside one `transaction` and the version row is inserted in that same transaction (`db.rs:1028-1043`). `down` is executed by `migrate_down_to` in its own transaction (`db.rs:1118-1134`). No migration edits an earlier one (header contract, `db.rs:29-31`). Status column is the safety assessment.

| v | name | Up creates / changes | Indexes / constraints | Down | Verdict |
|---|---|---|---|---|---|
| 1 | `bootstrap` (`db.rs:33-48`) | `app_meta(key PK, value)` + `schema_version`, `bootstrapped_at` | PK `key` | `DROP app_meta` | Additive; schema_version starts as string `hiverouter.v1` |
| 2 | `runs_tasks_chat` (`db.rs:50-97`) | `runs`, `tasks`, `chat_messages`, `command_receipts` | `idx_runs_status`; `idx_tasks_run`; `idx_messages_run`; task→run FK CASCADE; chat→run FK CASCADE | Drops all four | Additive; **down loses every run** |
| 3 | `snapshot_index` (`db.rs:98-113`) | `snapshots(run_id, revision, path, created_at, schema_version)` | PK `(run_id, revision)` | Drop | Additive; path is absolute (`snapshot.rs:109`) → not portable across data-dir moves |
| 4 | `tool_turns` (`db.rs:114-141`) | `ALTER runs ADD tool_turns, tool_calls`; `tool_turns` | `UNIQUE(run_id, tool_call_id)`; PK `id` | Drop table + 2 columns | Additive; down supported (SQLite ≥3.35) |
| 5 | `task_results` (`db.rs:142-158`) | `task_results(task_id PK, run_id, text, created_at)` | `idx_task_results_run`; FK CASCADE | Drop | Additive |
| 6 | `agent_messages` (`db.rs:159-178`) | `agent_messages` | `idx_agent_messages_run`; FK CASCADE | Drop | Additive |
| 7 | `agent_memory` (`db.rs:179-197`) | flat `agent_memory(agent_key,key,value,run_id,task_id,created_at)` | PK `(agent_key,key)`; `idx_agent_memory_agent` | Drop | Superseded by v11; global key space (no run in PK) |
| 8 | `run_usage` (`db.rs:198-217`) | `run_usage` 6 nullable counters | PK `run_id`; FK CASCADE | Drop | Additive; NULL-preserving |
| 9 | `agent_profiles` (`db.rs:218-232`) | `agent_profiles(id PK, profile_json, source, imported_at)` | PK only | Drop | Additive; JSON not validated by DB |
| 10 | `projects` (`db.rs:233-248`) | `projects`; backfill from `runs` | PK `id` | Drop | Additive; backfill idempotent (`INSERT OR IGNORE`) |
| 11 | `agent_memory_scoped` (`db.rs:249-274`) | New `agent_memory_scoped(run_id,scope_id,agent_key,key,...)`; copy flat rows with `scope_id='legacy'`; **DROP old table; RENAME**; recreate 3 indexes | PK `(run_id,scope_id,agent_key,key)`; 3 indexes | `DROP agent_memory` | **Rebuild migration**; non-`IF NOT EXISTS` DDL is safe only because of the version gate + transaction. Down drops scoped table and does **not** recreate the v7 shape |
| 12 | `a2a_answer_state` (`db.rs:275-297`) | 6 `ALTER agent_messages ADD` columns (`status` default `open`, `deadline_at`, `answered_at`, `answer_text`, `answer_actor`, `in_reply_to`) | `idx_agent_messages_status(run_id,status)` | Drop index + 6 columns | Additive; `status` has **no CHECK** constraint |
| 13 | `task_identity` (`db.rs:298-392`) | `tasks.class/rank/profile_id` + backfill by role vocabulary; `AFTER INSERT` trigger duplicating the mapping | `idx_tasks_identity(run_id,class,profile_id)` | Drop index/trigger/columns | Additive but **duplicated vocabulary** (also v17 and Rust `task_agent_tier`) → drift risk |
| 14 | `memory_banks` (`db.rs:393-514`) | `memory_banks(kind CHECK 6, scope_id, ..., UNIQUE(kind,scope_id))`; `memory_entries(kind CHECK 8, source_kind CHECK 7, trust CHECK 4, status CHECK 3)`; backfill global/run/agent_role/branch banks + entries | `idx_memory_entries_active` unique partial on `(bank_id,entry_key) WHERE status='active'`; `idx_memory_banks_run`, `idx_memory_banks_kind`, `idx_memory_entries_bank(bank_id,created_at)`; FK entries→banks CASCADE | Drop both | Additive but **agent_role banks get `run_id NULL`** (`db.rs:454-465`) → unreachable by run scope (see §6.8/§16) |
| 15 | `memory_journal` (`db.rs:515-543`) | `memory_journal(seq AUTOINCREMENT PK, run_id, bank_id, entry_id, action, actor_kind, actor_id, detail_json, created_at)`; backfill one `entry_written` per migration entry | `idx_memory_journal_run(run_id,seq)`, `idx_memory_journal_bank(bank_id,seq)` | Drop | Additive; **no FKs** (dangling refs by design, but no guard) |
| 16 | `memory_context_snapshots` (`db.rs:544-565`) | Snapshot of the bank/entry set that shaped a prompt | `idx_memory_snapshots_task`, `idx_memory_snapshots_run`; CHECKs on `rendered_chars`, `truncated` | Drop | Additive; **never pruned** (no DELETE anywhere) |
| 17 | `task_ownership` (`db.rs:566-618`) | `ownership_mode/scope/owner_task_id` + per-class backfill + `AFTER INSERT` trigger | `idx_tasks_owner(run_id,owner_task_id)` | Drop | Additive; trigger duplicates class vocabulary again |
| 18 | `memory_journal_hash_chain` (`db.rs:619-632`) | `prev_hash`, `event_hash` | `idx_memory_journal_hash(run_id,event_hash)` | Drop index + 2 columns | Additive; existing rows stay NULL (`unchained`) |
| 19 | `single_run_owner` (`db.rs:633-643`) | Unique partial index `tasks(run_id) WHERE ownership_mode='run'` | unique | Drop index | **Hard failure risk** on legacy DBs with 2 hyper rows; no pre-check/dedupe |
| 20 | `memory_journal_head` (`db.rs:644-658`) | `runs.memory_journal_head` + backfill `MAX(seq)` hash per run | none | Drop column | Additive; backfill uses `seq DESC` but not `event_hash` chain order check |
| 21 | `remote_control_plane` (`db.rs:659-790`) | `remote_agents`, `devices`, `remote_grants`, `device_actions`, `remote_audit`, `remote_supervisor_state` (+ singleton row) | `idx_remote_state`; `ux_remote_name` (partial, `revoked_at IS NULL`); `idx_devices_remote`; `ux_device_remote_ref`; `ux_remote_grant`; `UNIQUE(remote_id,device_id,idempotency_key)`; `idx_device_actions_status/device`; `idx_remote_audit_remote/device`; FKs devices→remote, grants→remote+device, actions→remote+device | Drop 6 tables | Additive; `remote_audit` has **no FKs** |
| 22 | `daemon_plane` (`db.rs:791-831`) | `daemons`, `daemon_reports` | `idx_daemons_state(state,next_tick_at)`, `idx_daemons_owner`; FK reports→daemons CASCADE; CHECKs on owner/kind/state/interval | Drop both | Additive; `restart_attempts`/`restart_window_at` are **never written** (dead columns) |
| 23 | `a2a_attempt_audit` (`db.rs:832-857`) | `ALTER agent_messages ADD attempts, last_attempt_at`; `a2a_audit` | `idx_a2a_audit_run`, `idx_a2a_audit_message`; CHECKs on actor/action length | Drop indexes/table/columns | Additive; audit rows have **no FK** to messages |

**Engine invariants (VERIFIED):**
- Pragmas: WAL, `synchronous=NORMAL`, `foreign_keys=ON`, `busy_timeout=5s` (`db.rs:958-966`). FK enforcement is per-connection, set for both open paths (`db.rs:931-956`).
- `current_version = MAX(version)` (`db.rs:987-996`). **No contiguity check** and **no name validation** of `schema_migrations` rows (inserted at `db.rs:1034-1041`, read at `db.rs:1146-1150`): a gap or renamed row is accepted silently.
- Backup: only when `current > 0` and a newer migration exists (`db.rs:1025-1027`); `VACUUM INTO` then `integrity_check` (`db.rs:1085-1097`); name `{stem}.pre-v{version}-{stamp}.bak` with numeric suffix on collision (`db.rs:1075-1084`). **No pruning, no restore API, no backup on the down path.**
- `SchemaNewer` refuses without running anything (`db.rs:1022-1024`), tested at `db.rs:1733-1757`.
- Each migration is atomic; a failed migration is not recorded (`db.rs:1032-1042`), but a failing migration that is *deterministically* failing (v19) makes every boot exit (`main.rs:77-87`).

---

## 4. Table registry (schema · writers · readers · failure modes)

`FK` = declared foreign key; `—` = none. Writer/reader columns name the store methods (all `store.rs` unless noted).

| Table | Schema source | Indexes/constraints | Writers | Readers | Failure / recovery notes |
|---|---|---|---|---|---|
| `app_meta` | `db.rs:36-43` | PK `key` | v1, `migrate()` | `app_meta()` `db.rs:1162-1170` | Written once; no update path; absent after down-to-0 (`db.rs:1504-1516`) |
| `schema_migrations` | `db.rs:1013-1019`, `1111-1117` | PK `version` | `migrate`, `migrate_down_to` | `applied_migrations` `db.rs:1141-1156` | No contiguity/name validation |
| `runs` | `db.rs:53-61` (+v4 counters, +v20 head) | PK `id`; `idx_runs_status` | `create_run` `352-390`; `set_run_status` `479-513`; `increment_run_tool_counters` `1353-1376`; journal head `97-100`; recovery `recovery.rs:134` | `get_run` `417-432`; `latest_active_run` `438-452`; `list_runs` `458-473`; tool counters `1383-1402`; `verify_memory_journal` `2113-2120`; history `history.rs:67-69` | Terminal guard is a hard-coded SQL list (`479-513`) that differs from `run_state::RUN_TERMINAL_STATES`; `Killed` is terminal in the contract but not in `RUN_TERMINAL_STATES` (`run_state.rs:76-81`) |
| `tasks` | `db.rs:63-73` (+v13 identity, +v17 ownership) | PK `id`; FK run CASCADE; `idx_tasks_run`; `idx_tasks_identity`; `idx_tasks_owner`; unique partial `idx_tasks_single_run_owner` | `insert_task` `519-540` (+`ensure_bank_for_inserted_task` `2690-2751`); `update_task_state` `576-633`; `update_task_parent` `644-666`; triggers v13/v17 | `get_task` `546`; `tasks_for_run` `672`; `task_count` `693`; `task_identity` `1189`; `task_ownership` `1162`; recovery `recovery.rs:206` | Terminal-sticky by SQL; identity/ownership are DB-trigger derived, never recomputed on role change; `depth`/`spawn_seq` written with saturating casts (`531-533`) |
| `chat_messages` | `db.rs:75-82` | PK `id`; FK run CASCADE; `idx_messages_run` | `append_message` `711-741` | `messages` `747-780` | Row decode **fabricates** `Uuid::nil()`, `RunId(nil)`, `ChatRole::System`, epoch timestamp on malformed data (`757-771`) |
| `command_receipts` | `db.rs:84-88` | PK `correlation_id` | `record_receipt` `2639` (`INSERT OR REPLACE`) | `receipt_run` `2657-2684` | 24 h window enforced in Rust; a corrupted timestamp returns `None` (idempotency silently lost) |
| `snapshots` | `db.rs:101-108` | PK `(run_id,revision)` | `SnapshotStore::write` `snapshot.rs:101-116` (`INSERT OR REPLACE`); `prune_run` deletes `snapshot.rs:220-228` | `indexed_revisions` `snapshot.rs:237-255`; history `history.rs:75-88` | Path is absolute; index/file divergence possible (write file then index); prune deletes file then index |
| `tool_turns` | `db.rs:120-134` | PK `id`; `UNIQUE(run_id,tool_call_id)` | `record_tool_turn` `1212-1247` | `tool_turns_for_run` `1269-1346`; `task_tool_turns` `1253-1262` | Duplicate call = SQLite constraint → mapped to `E_INTERNAL_ERROR` (`db.rs:897-911`), not a named tool refusal |
| `task_results` | `db.rs:146-152` | PK `task_id`; FK run CASCADE; `idx_task_results_run` | `record_task_result` `790-814` (upsert) | `task_result` `820-832` | Retry replaces text; no history |
| `agent_messages` | `db.rs:163-172` (+v12, +v23) | PK `id`; FK run CASCADE; `idx_agent_messages_run`; `idx_agent_messages_status` | `record_agent_message` `1110`; `record_agent_message_limited` `841`; `answer_agent_message` `983`; `record_agent_message_attempt` `919`; `expire_agent_message` `1093`; `expire_open_agent_messages` `1849`; `set_agent_message_deadline` `1049` | `agent_messages_for_run` `1214`; `..._for_task` `1258`; `pending_agent_messages` `1805`; `find_open_agent_message` `882`; `agent_message_status` `1071` | `status` has no CHECK; `in_reply_to` is **never written** (dead column, v12 `db.rs:284`); expire/deadline report `Ok` for a missing row |
| `agent_memory` (flat, post-v11) | `db.rs:253-269` | PK `(run_id,scope_id,agent_key,key)` | `record_agent_memory`/`_scoped` `1313-1494`; `clear_run_memory` `1777-1780`; `clear_agent_memory` `2626-2637` | `agent_memory_for_run` `1662`; `..._for_scope` `2349`; `..._for_scope_any_role` `1697`; `agent_memory` `2450`; `agent_memory_entries_*` `2387-2444` | Legacy flat read path; `clear_agent_memory` touches only this table |
| `memory_banks` | `db.rs:397-410` | PK `id`; `UNIQUE(kind,scope_id)`; CHECK kind; indexes `434-435` | v14 backfill; `ensure_bank_for_inserted_task`; `record_agent_memory_scoped` `1377-1457`; `record_memory_entry` `1565-1580`; `write_run_digest` `1737-1742` | `memory_banks_for_run` `1625-1656`; `memory_entries_for_bank_scoped` join `2311`; `hyper_summary_rows` `2793` | No FK on `run_id`/`owner_task_id`/`parent_bank_id`; agent_role banks have `run_id NULL` |
| `memory_entries` | `db.rs:411-431` | PK `id`; FK bank CASCADE; unique active partial `432-433`; CHECKs kind/source_kind/trust/status; index `436` | `record_agent_memory_scoped` `1458-1478`; `record_memory_entry` `1581-1602`; `write_run_digest` `1743-1761`; v14 backfill; `prune_inactive_memory` deletes `2153-2158`; `clear_run_memory` deletes `1781-1785` | `memory_read_set` `2197-2221`; `hyper_summary_rows` `2790-2809`; `memory_entries_for_bank` `2251`; `..._scoped` `2299`; `verify`/chain via journal only | **No `agent_key` column** → cannot select by role for erasure; `trust` nullable for migration notes |
| `memory_journal` | `db.rs:519-529` (+v18) | PK `seq` autoincrement; `idx_memory_journal_run/bank/hash`; **no FKs** | `append_memory_journal` `47-102` (called from scoped write `1479-1489`, validated write `1603-1613`, promote `1896-1906`, prune `2159-2169`) | `memory_journal_for_run` `1918-1941`; `memory_journal_chain` `2020-2041`; `verify_memory_journal` `2047-2131` | Journal is append-only by convention, not by trigger/privilege; `write_run_digest`, `clear_run_memory`, and flat-only writes are **not journaled** |
| `memory_context_snapshots` | `db.rs:548-558` | PK `id`; CHECKs; indexes `559-560` | `record_memory_context_snapshot` `1947-1980` (only caller `execution.rs:4139-4147`) | `memory_context_snapshot` `1986-2014`; HTTP `http.rs:3189-3219` | **No pruning**; unbounded per task refresh; `truncated` always written `false` (`execution.rs:4145`) even when the 16-entry LIMIT truncated the set |
| `run_usage` | `db.rs:202-212` | PK `run_id`; FK CASCADE | `record_run_usage` `2479-2510` (checkpoints `execution.rs:1128`, `1144`) | `run_usage` `2516-2544`; HTTP `http.rs:3221-3234` | Upsert replaces totals; NULLs preserved (test `store.rs:3636-3652`); no per-turn ledger |
| `agent_profiles` | `db.rs:222-227` | PK `id` | `upsert_agent_profile` `2584-2606` | `agent_profiles` `2550-2578`; HTTP import/list/delete `http.rs:684-686` | Invalid stored JSON → `E_SCHEMA_INVALID` for the whole list (`2574`), boot falls back to embedded only (`http.rs:180-189`) |
| `projects` | `db.rs:237-243` | PK `id` | `create_run` (`INSERT OR IGNORE`, `371-373`); v10 backfill | `projects` `396-411`; HTTP `http.rs:692` | Backfilled names are the id (`242-243`); no rename path |
| `remote_agents` | `db.rs:663-687` | PK; CHECK state/owner/transport; `idx_remote_state`; partial unique name | `create_remote` `316-380`; `heartbeat` `688-729`; `revoke_remote` `454-486`; `expire_leases` `784-827`; `reconcile_on_boot` `831-946` | `get_remote` `406`; `list_remotes` `386`; `find_by_request_id` `424-448`; supervisor | `find_by_request_id` is **never called by `create_remote`** → retry with same request id creates a duplicate |
| `devices` | `db.rs:689-705` | PK; FK remote; CHECK class/safety; partial unique `(remote_id,adapter_ref)` | `create_device` `492-555`; `request_action` state update `1136-1142`; `resolve_action` `1323-1329` | `get_device` `581`; `list_devices` `561`; HTTP device JSON `1683-1700` | `state_json` is free-form; state hash is the only integrity check |
| `remote_grants` | `db.rs:707-721` | PK; FKs remote+device; partial unique `(remote,device,capability,action)` | `create_grant` `599-660` | `list_grants` `666`; authorization queries `1049-1064`, `1278-1291` | Grant is not checked against `devices.capabilities_json` → over-grant until execution refuses |
| `device_actions` | `db.rs:723-748` | PK; FKs; CHECK status vocabulary; `UNIQUE(remote,device,idempotency_key)` | `request_action` `1106-1135`; `resolve_action` `1306-1319`; `expire_approvals` `753-759`; `reconcile_on_boot` `876-892` | `get_action` `1357`; `find_action` `1168`; in-flight count `1065-1070` | Reversible action commits device state in the same tx (`1136-1142`); deny records a `state_after_hash` of the unchanged state (`1296-1318`) — misleading |
| `remote_audit` | `db.rs:750-767` | PK; indexes; **no FKs** | `audit()` helper `263-300` from every mutation/sweep; `record_control_decision` `1377-1402`; `record_admission` `1410-1434` | `list_audit` `1440-1475`; HTTP `http.rs:2044` | Audit is the control-plane's durable truth; `detail_json` is free-form (no redaction pass beyond caller discipline) |
| `remote_supervisor_state` | `db.rs:769-780` | PK `id=1` CHECK | `record_state` `remote_supervisor.rs:108-141` (errors ignored) | `state()` `104`; `read_state` `145-166`; HTTP `http.rs:673` | Singleton row never created if v21 down/up interplay; `lane_started_at` only at lane start |
| `daemons` | `db.rs:795-817` | PK; CHECKs; `idx_daemons_state`, `idx_daemons_owner` | `create` `148-187`; `set_enabled` `220-246`; `retire` `249-265`; `record_daemon_tick` `500-549` | `get` `189`; `list` `203`; `tick_due` select `277-296` | `restart_attempts`/`restart_window_at` never written; `owner_id` not verified against a real task/run |
| `daemon_reports` | `db.rs:818-825` | PK; FK daemon CASCADE; index | `record_daemon_tick` `533-544` | `reports` `550-577` | Report insert is inside the tick tx; script stdout/stderr capped at 8 KiB (`589-612`) |
| `a2a_audit` | `db.rs:838-848` | PK; CHECKs; indexes; **no FK** | `record_agent_message_attempt` `949-961`; `answer_agent_message` `1008-1019` | No store reader; only direct SQL/tests | `expire_agent_message`/`expire_open_agent_messages` write **no** audit row → asymmetric lifecycle evidence |
| `approvals` **(outside v1-v23)** | `hiverouter-tools/src/approval.rs:151-170` | PK `approval_id`; `nonce UNIQUE`; partial unique `(run_id,tool_call_id) WHERE decision IS NULL` (`190-192`); 2 identical indexes (`167-170`) | `insert_request` `211-242`; `resolve` `357-430`; expire/cancel in tools crate | `load_request` `258-347`; `pending_approvals` `tools_bridge.rs:405`; HTTP `/api/approvals` `http.rs:1356` | Created by `ToolsBridge::migrate` from a **health check** (`http.rs:223-241`), not by `db.rs`; no `schema_migrations` row, no backup, no down |

---

## 5. `RunStore` method inventory

`RunStore` is `Arc<Database>` + methods (`store.rs:335-2685`). Every method runs under the single `Mutex<Connection>` (`db.rs:916-918`, `968-970`), so intra-process writes are serialized; cross-process safety relies on WAL + 5 s busy timeout only.

| Group | Methods (line) | Atomic? | Notes / risks |
|---|---|---|---|
| Runs | `create_run` 352, `projects` 396, `get_run` 417, `latest_active_run` 438, `list_runs` 458, `set_run_status` 479 | Single statement each; `create_run` = 2 statements **without** tx (project + run) | A crash between the two leaves a project with no run (harmless); `set_run_status` silently succeeds when replaying the same terminal state, `E_RUN_TERMINAL` otherwise (`501-511`) |
| Tasks | `insert_task` 519, `get_task` 546, `update_task_state` 576, `update_task_parent` 644, `tasks_for_run` 672, `task_count` 693 | `insert_task` = insert + bank create, **no tx** | Task row can exist without its bank if the second statement fails (`536`); identity/ownership depend on triggers, so a role edit never re-derives them |
| Chat | `append_message` 711, `messages` 747 | Single statement | Decode fallbacks fabricate values on corrupt rows |
| Task results | `record_task_result` 790, `task_result` 820 | Upsert | One row per task; retry overwrites |
| A2A | `record_agent_message_limited` 841, `find_open_agent_message` 882, `record_agent_message_attempt` 919, `answer_agent_message` 983, `set_agent_message_deadline` 1049, `agent_message_status` 1071, `expire_agent_message` 1093, `record_agent_message` 1110, `agent_message_count` 1144, `pending_agent_messages` 1805, `expire_open_agent_messages` 1849, `agent_messages_for_run` 1214, `agent_messages_for_task` 1258 | Budget insert is one statement (`INSERT … SELECT … WHERE COUNT < ?`, `854-868`) → atomic; attempt/answer use `unchecked_transaction` (`936`, `993`) | `find_open` does full-text equality on `text` (no index); `expire_*` no audit; deadline/expire return `Ok` for missing rows |
| Identity/ownership | `task_ownership` 1162, `task_identity` 1189 | Read-only | Returns `None` rather than guessing unknown roles (test `store.rs:3333-3367`) |
| Memory write | `record_agent_memory` 1313, `record_agent_memory_scoped` 1329, `record_memory_entry` 1513, `write_run_digest` 1732 | **Scoped write = 5+ statements, no tx** (`1356-1491`); `record_memory_entry` = one tx (`1564-1614`); digest = 3 statements, no tx, **no journal** (`1733-1762`) | Core integrity defect (§16 #4/#5) |
| Memory read | `memory_banks_for_run` 1625, `agent_memory_for_run` 1662, `agent_memory_for_scope_any_role` 1697, `agent_memory_for_scope` 2349, `agent_memory_entries_for_run` 2387, `..._for_scope` 2399, `agent_memory` 2450, `memory_entries_for_bank` 2251, `..._scoped` 2299, `memory_read_set` 2181, `memory_hyper_summary` 2237 | Read-only | `memory_entries_for_bank` is the **unscoped** operator surface (doc comment at `2290-2295` admits it); `memory_read_set` limits to 16 total with no per-bank fairness (`2197-2203`) |
| Journal | `memory_journal_for_run` 1918, `memory_journal_chain` 2020, `verify_memory_journal` 2047 | Read-only | Verify recomputes hashes and compares the head (`2094-2123`); it does **not** prove entries exist or that every write was journaled |
| Retention | `clear_run_memory` 1773, `prune_inactive_memory` 2139, `clear_agent_memory` 2626 | clear/prune = one tx (`1776`, `2152`); role clear = single DELETE | clear deletes entries but not journal/snapshots; role clear misses bank entries |
| Context snapshot | `record_memory_context_snapshot` 1947, `memory_context_snapshot` 1986 | Single statement | `prompt_sha256` is actually the **memory-block** hash (`execution.rs:4138`) |
| Trust | `promote_memory_entry` 1868 | One tx (update + journal) | Operator-only by convention, no DB-level guard; accepts arbitrary `actor` string |
| Usage | `record_run_usage` 2479, `run_usage` 2516 | Upsert | NULL-preserving |
| Profiles | `agent_profiles` 2550, `upsert_agent_profile` 2584, `delete_agent_profile` 2612 | Single statements | List fails wholesale on one bad JSON row |
| Receipts | `record_receipt` 2639, `receipt_run` 2657 | Single statement | 24 h window in Rust; malformed timestamp = cache miss |

Private helpers: `terminal_task_state_sql` 308, `ensure_bank_for_inserted_task` 2690, `nearest_hive_bank` 2754 (8-level walk), `hyper_summary_rows` 2785, row mappers 2818-2877.

---

## 6. Memory subsystem

### 6.1 Bank taxonomy

- Kinds: `global|project|run|hive|branch|agent_role` (`db.rs:399-400`). IDs are string conventions: `bank:global:default`, `bank:run:{run_id}`, `bank:hive:{task_id}`, `bank:branch:{task_id}`, `bank:agent_role:{normalized}` (`db.rs:438-475`; `store.rs:261-268`).
- Creation: v14 seeds the global bank and backfills run/hive/branch/agent_role banks (`db.rs:438-508`); `insert_task` creates the run bank plus hive/branch bank for the spawned class (`store.rs:2690-2751`); memory writers are additional `INSERT OR IGNORE` creators (`store.rs:1377-1457`, `1565-1580`).
- **`UNIQUE(kind, scope_id)`** (`db.rs:409`) means one bank per scope; branch/hive scopes are globally unique task UUIDs.
- Parentage: run→global (`db.rs:441`), hive→run, branch→nearest hive or run (`store.rs:2731-2732`, `2754-2781`). `parent_bank_id` has no FK and is informational only — **no reader walks it**; `memory_read_set` reconstructs the set from the task row instead (`store.rs:2188-2196`). **VERIFIED.**

### 6.2 Entries, trust and visibility

- `kind` CHECK (8 values, `db.rs:415-416`); `source_kind` CHECK (7, `db.rs:419-420`); `trust` CHECK (4, `db.rs:424-425`); `status` CHECK (3, `db.rs:426-427`).
- Read visibility filter: `status='active' AND (trust IN ('unverified','verified','authoritative') OR (trust IS NULL AND kind='note'))` (`store.rs:2199-2202`, duplicated at `2795-2797`). `quarantined` is deliberately unreadable (`store.rs:254-257`).
- The validated writer refuses `quarantined` and unknown kinds so it can never write an unreadable row (`store.rs:1526-1531`; test `mem_isolation.rs:189-247`). The **live** writer hard-codes `'observation'`/`'unverified'` (`store.rs:1467-1468`), bypassing that guarantee.
- Unique-active index makes supersession key-based (`db.rs:432-433`), but no writer ever sets `status='superseded'` — `prune_inactive_memory` therefore only ever deletes rows that a manual/test UPDATE marked (`store.rs:3404-3413`). **VERIFIED:** `grep 'superseded'` finds only migration CHECK, prune SQL, and tests.

### 6.3 Write paths (three, with different guarantees)

| Path | Callers | Bank selection | Atomicity | Journal | Trust/kind validation |
|---|---|---|---|---|---|
| `record_agent_memory_scoped` (`store.rs:1329-1494`) | engine `execution.rs:1955, 1970, 4661, 4910, 5246, 5572`; HTTP evolution `http.rs:2622, 2831, 2969`; `record_agent_memory` wrapper `store.rs:1321` | By persisted `tasks.class`: hyper→run, super→hive, agent/subagent→branch; **missing identity→own branch** (`1342-1354`) | **None** — 5+ sequential `execute` calls (`1356-1491`) | Yes, last (`1479-1489`) | No: hard-codes `observation`/`unverified` |
| `record_memory_entry` (`store.rs:1513-1619`) | **Tests only** (`mem_isolation.rs:132,170,200,212,224,229`) | Strict `caller_bank_for_class`, unknown class refused (`1532-1534`) | One transaction (`1564-1614`) | Yes (`1603-1613`) | Yes (`1523-1531`) |
| `write_run_digest` (`store.rs:1732-1765`) | `execution.rs:2246` | run bank, key `run-digest` | None (3 statements) | **No** | n/a |

Consequences:
- A crash between the DELETE-active and INSERT in the scoped path can drop the previous value; a crash after INSERT but before the journal append yields an entry with no journal row. `verify_memory_journal` still reports `valid=true` because it only hashes journal rows (`store.rs:2072-2123`). **VERIFIED.**
- The `scope_id` column of the flat table is caller-supplied and unvalidated: `scope_id='run'` makes a row legible to `agent_memory_for_scope_any_role` for any role in the run; tests rely on this (`http_boot.rs:718-752` uses `"hive-a"`/`"hive-b"` scopes with random task ids). **VERIFIED.**
- The run prompt is copied into the run bank, truncated to 2,000 chars, as untrusted data (`execution.rs:1955-1964`), and every task in the run reads the run bank (`store.rs:2202`), i.e. every child sees the raw prompt.

### 6.4 Read paths

| Path | Banks | Limit | Callers | Control |
|---|---|---|---|---|
| `memory_read_set` (`store.rs:2181-2229`) | run + immediate parent branch + own branch + nearest hive | 16 (engine injection), 8 at seed | `execution.rs:1323-1337`, `4115-4147` | This is the only model-facing path; sibling banks unaddressable (tests `store.rs:3502-3525`, `mem_isolation.rs:118-187`) |
| `memory_hyper_summary` (`store.rs:2237-2245`, SQL `2785-2816`) | run + hive only | caller (8/64/256) | HyperAgent control turn `execution.rs:4542, 4588, 4734, 4871`; governance HTTP `http.rs:1497, 2379, 2535, 2733, 2866` | Deliberately excludes branch raw entries; values are truncated at the HTTP edge (`http.rs:2357`) |
| Flat role reads (`agent_memory_for_run`, `agent_memory`, `..._for_scope`) | flat table | caller | operator HTTP `/api/agents/memory`, `/api/agents/runs/{id}/memory` | **Not** the bank model; legacy |
| Operator bank reads (`memory_entries_for_bank` unscoped, `..._scoped`) | any named bank | caller | HTTP entries route uses the **scoped** variant (`http.rs:3013`) | Unscoped variant remains public and documented as a gap (`store.rs:2290-2295`; `mem_isolation.rs:106-115`) |

**Starvation risk (VERIFIED by SQL shape):** `ORDER BY created_at DESC, id DESC LIMIT 16` over four banks (`store.rs:2203`) gives no per-bank quota. A branch that writes 16 notes in one turn crowds the run bank and parent branch out of the next prompt. There is no test for this.

### 6.5 Journal hash chain

- Hash input: `prev|run|bank|entry|action|actor_kind|actor_id|detail|created_at` (`store.rs:40-43`), SHA-256.
- Append finds the previous hash by `seq DESC` for that run (`store.rs:58-66`), inserts `prev_hash/event_hash`, then updates `runs.memory_journal_head` (`store.rs:79-100`).
- Verification walks `seq ASC`, skips unchained rows, recomputes and compares `prev_hash`/`event_hash`, then compares the stored head (`store.rs:2072-2123`). Truncation and mutation are detected (tests `store.rs:3214-3241`, `3426-3452`).
- **Limits (VERIFIED):** (a) no trigger/privilege prevents `UPDATE`/`DELETE` on `memory_journal` — tests themselves mutate it (`store.rs:3230-3237`, `3442-3448`); (b) the head can be rewritten by the same code; (c) journal rows are not FK-bound, so pruning entries dangles `entry_id`; (d) `write_run_digest`, `clear_run_memory`, and every flat-table/`agent_memory` write are invisible to the journal. The chain proves *what was written to the journal*, not *what was written to memory*.

### 6.6 Context snapshots

`record_memory_context_snapshot` stores `banks_json`, `entry_ids_json`, `rendered_chars`, `truncated`, `prompt_sha256`, `created_at` (`store.rs:1947-1980`; table `db.rs:548-558`). The only writer is `refresh_memory_bank` (`execution.rs:4139-4147`) and the only reader is HTTP (`http.rs:3189-3219`). Issues:
1. `prompt_sha256` hashes the **rendered memory block only** (`execution.rs:4138`), not the assembled system+user prompt or the injected lessons. The column name overstates what is proven. **VERIFIED.**
2. `truncated` is always `false` (`execution.rs:4145`) even when `memory_read_set`'s LIMIT cut the set. **VERIFIED.**
3. No pruning: one row per task refresh for the life of the DB. **VERIFIED** (no DELETE in the codebase).
4. `entry_ids_json` dangles after `clear_run_memory`/prune; nothing annotates or removes these rows.

### 6.7 Is memory access truly controlled? (per-control assessment)

| Control | Implemented | Tested | Verdict |
|---|---|---|---|
| Foreign-run bank read blocked | `memory_entries_for_bank_scoped` joins on `memory_banks.run_id` (`store.rs:2307-2314`) | `mem_isolation.rs:76-116` | **Controlled** |
| Sibling-branch read blocked | read-set bank allow-list excludes sibling (`store.rs:2202`) | `store.rs:3502-3525` | **Controlled** |
| Unknown identity cannot widen to run bank | fail-closed fallback to own branch (`store.rs:1338-1354`) | `mem_isolation.rs:38-74` | **Controlled** (write side still permissive in shape, but bank choice is safe) |
| Trust gate | `quarantined` filtered; NULL-note exception (`store.rs:2199-2202`) | `mem_isolation.rs:189-247` (writer refusal) | **Controlled for reads**; a direct SQL writer can still insert `quarantined` |
| Model write must use own bank + valid vocab | `record_memory_entry` | tests only | **Not wired** → not in force |
| Operator read requires auth | GETs bypass session token unless a launcher token is configured (`http.rs:738-766`); only Host guard (`701`, `706-730`) | none seen | **Not controlled** (loopback process can read `/api/agents/runs/{id}/memory`, `/api/memory/banks/{id}/entries`) |
| Erasure removes value everywhere | `clear_agent_memory` deletes flat only (`store.rs:2626-2637`); `clear_run_memory` deletes bank entries + flat (`1773-1799`) but not journal/snapshots/lesson file | role clear test only checks flat (`http_boot.rs:1221-1240`) | **Not controlled for role clear**; run clear is partial |

### 6.8 Governance / lesson records: can they be orphaned? **Yes.**

Storage today (all **VERIFIED**):
- Proposal, vote, authorization, apply, rollback records are **memory entries** in the run bank with key prefixes `improvement:`, `evolution_vote:`, `evolution_authorization:`, `evolution_apply:`, `evolution_rollback:` (`http.rs:2383-2407`, writes `2601-2631`, `2745`, `2952`; vote write `execution.rs:4901-4916`).
- The applied lesson artifact is a line in `<data_dir>/evolution/LESSONS.jsonl` (`http.rs:2692-2694`, written atomically `2704-2715`, appended `2815-2820`).
- The DB record stores the artifact hash chain (`http.rs:2821-2830`); rollback **requires** the `evolution_apply:` entry to find the line hash (`http.rs:2870-2880`).

Failure modes:
1. **Cross-store crash window.** Apply writes the lesson file first (`2818`), then the DB entry (`2831-2841`); the compensating rewrite on DB failure is best-effort (`let _ =`, `2839`). A crash between them leaves an applied lesson with no DB apply record → rollback refuses `E_APPROVAL_REQUIRED` (`2875-2877`) and the UI shows `not_applied` while the lesson is in force (`http.rs:2457-2473`). **VERIFIED code order; crash window is UNVERIFIED at runtime but structurally present.**
2. **Clear-guard blind spots.** `clear_run_memory` refuses only when the newest **256** run/hive entries contain `evolution_authorization:` or `evolution_apply:` (`http.rs:1497-1507`). It does **not** guard `evolution_rollback:`, `evolution_vote:`, or `improvement:`. If a run accumulates >256 newer entries after apply (each task result is an entry, `execution.rs:5572`), the guard passes and `clear_run_memory` deletes **all** run-bank entries (`store.rs:1781-1785`) while `LESSONS.jsonl` keeps the lesson → permanent orphan (unrollbackable, authorization evidence destroyed). **VERIFIED.**
3. **v14 legacy banks.** Rows migrated into `bank:agent_role:*` carry `run_id NULL` (`db.rs:454-465`, entries `477-488`), so they are excluded from `memory_read_set` (bank allow-list), `clear_run_memory` (`WHERE run_id = ?`, `1783`), `prune_inactive_memory` (`2144`), and even `memory_entries_for_bank_scoped` (`b.run_id = ?`, `2312`). Any pre-v14 role memory is **unreadable and undeletable** through every public API except the unscoped `memory_entries_for_bank` if the id is known. **VERIFIED.**
4. **No FK/`ON DELETE RESTRICT` from governance to run/bank.** The only protection is the HTTP pre-scan. **VERIFIED.**

---

## 7. A2A message lifecycle

**States (implicit, no CHECK):** `open` (default, `db.rs:279`) → `answered` (`store.rs:996`) | `expired` (`1097`, `1853`). **Transitions (all CAS via `WHERE status='open'`):**

| Operation | Method | Guard | Audit row | Failure mapping |
|---|---|---|---|---|
| Send (budgeted) | `record_agent_message_limited` `841-875` | single `INSERT…SELECT…WHERE COUNT<max` | none | `E_CALL_LIMIT` |
| Send (unbudgeted) | `record_agent_message` `1110-1138` | none | none | DB error only |
| Find duplicate open question | `find_open_agent_message` `882-912` | exact text equality, `status='open'` | none | — |
| Answer attempt | `record_agent_message_attempt` `919-976` | `status='open' AND attempts<max` | `a2a_audit` in same tx (`949-962`) | `TaskNotFound` / `CallLimit` / `TaskTerminal` (`967-975`) |
| Answer | `answer_agent_message` `983-1043` | `status='open'` | `a2a_audit` same tx (`1008-1020`) | `TaskNotFound` / `TaskTerminal` (`1028-1042`) |
| Set deadline | `set_agent_message_deadline` `1049-1065` | `status='open'` | none | **`Ok` even when no row matched** |
| Expire one / all for run | `expire_agent_message` `1093-1104`; `expire_open_agent_messages` `1849-1861` | `status='open'` | **none** | `Ok` regardless |
| Boot expiry | recovery `recovery.rs:141-143` (best-effort `let _ =`) | only runs newly interrupted | none | failure swallowed |

**Tests:** attempt cap + audit count (`store.rs:3064-3118`), atomic budget (`3121-3136`), duplicate-find until answered (`3030-3061`), crash expiry (`3164-3183`), CAS answer (`3186-3212`), HTTP answer 409 on replay (`http_boot.rs:434-489`), HTTP pending list (`http_boot.rs:230-255`).
**Findings:**
- `in_reply_to` is selected (`store.rs:1222`, `1268`) but **never inserted** → always NULL. Dead schema from v12. **VERIFIED.**
- `expired` transitions leave no audit evidence, unlike answered/attempted → lifecycle evidence is asymmetric.
- `pending_agent_messages` filters `status='open'` across all runs (`store.rs:1815`) so `idx_agent_messages_status(run_id,status)` cannot serve it well; no global status index.
- Message text is stored verbatim (no size cap at the store layer); caps live in the tool loop, so a direct store caller can write unbounded text.

---

## 8. Remote control plane

**Tables and transitions.** `remote_agents.state ∈ {enrolled, online, degraded, offline, quarantined, revoked, retired}` (`db.rs:673-674`). Code only writes `enrolled` (`remote.rs:341`), `online` (`700-705`), `offline` (`802-805`, `887-892`), `revoked` (`459-461`), and the DDL allows states no writer produces (`degraded`, `quarantined`) — **schema/code drift** (quarantined_at is likewise never written). **VERIFIED.**

| Operation | Method | Atomicity | Idempotency | Notes |
|---|---|---|---|---|
| Create remote | `create_remote` `316-380` | insert + audit, no tx | **Broken**: `find_by_request_id` exists (`424-448`) but is never called | Retry duplicates a remote; unique name index only blocks an unrevoked same-name duplicate (`db.rs:687`) |
| Heartbeat | `688-729` | update + audit, no tx | repeats extend lease | Increments `generation` and `rev`; lease +120 s (`693-695`) |
| Revoke | `454-486` | update + audit, no tx | second revoke → `RemoteUnknown` | Cascades logically: grants/actions revalidate on use |
| Create device | `492-555` | insert + audit, no tx | unique `(remote,adapter_ref)` partial | Validates class/safety; `capabilities_json` unvalidated |
| Create grant | `599-660` | insert + audit, no tx | unique `(remote,device,cap,action)` partial | **Does not check device capabilities** → grants can exceed device ability; execution then refuses `RemoteCapability` |
| Request action | `953-1166` | one tx incl. authority re-read, in-flight count, device state update, audit (`1003-1157`) | same `(device,idempotency_key)` returns stored row (`988-990`) | Process-local `action_lock` (`984-987`) + single DB mutex; reversible actions marked `confirmed` immediately |
| Resolve critical | `1189-1351` | one tx revalidating remote/lease/device/grant, CAS on status+nonce (`1306-1319`) | duplicate resolve refused | Tests: revocation race `remote_control_plane.rs:84-117`, lease race `120-158`, device revoke `161-190` |
| Expire approvals | `733-780` | tx: select, update, one audit per row | idempotent (status change) | Unlocks device after timeout (test `remote_control_plane.rs:193-264`) |
| Expire leases | `784-827` | tx + per-row audit | idempotent (test `289-293`) | Uses **string comparison** of RFC3339 timestamps (`794`, `804`) — correct only while all writers use the same UTC `Z` format |
| Boot reconcile | `831-946` | one tx: executing→interrupted, expired approval→timeout, online→offline, audits | idempotent (test `remote_control_plane.rs:297-341`) | Called from `main.rs:90-102`; failure **aborts boot** (`96-101`) |

**Findings:** `request_id` in `device_actions` is set to the action id (`remote.rs:1124`), not the caller's request id; deny writes `state_after_hash` of unchanged state (`1296-1318`); `remote_audit` has no FKs so orphan audit rows are structurally allowed; `record_control_decision` ignores its `_action` argument (`1377-1382`).

---

## 9. Daemons

- A daemon never becomes a task (doc `daemon.rs:1-7`; tests `daemon_tick_degraded.rs:66-72`, `daemon.rs:679-684`).
- State: `enrolled → running → degraded` and `retired` (`db.rs:802-803`); `stopped` via switch (`daemon.rs:228`). Retired cannot be re-enabled (`daemon.rs:816-836`).
- Tick: select due `running` rows (`277-296`), run job **outside** the DB lock, then one tx updates the row + inserts one report (`514-546`). `AND state='running'` guard prevents a degraded daemon from re-ticking (test `daemon_tick_degraded.rs:56-60`).
- Jobs: `{"check":"db"}` → SQLite probe (`307-317`); `{"kind":"script", program, args, timeout_secs}` → allowlisted `.exe`/`.ps1` under `script_root` with canonicalization, plain-name check, arg count/length caps, 1-60 s timeout, process-tree kill, 8 KiB capped stdout/stderr (`333-498`); anything else → `degraded` with a named reason (`322-331`).
- Tests: no-runner degraded (`daemon_tick_degraded.rs:16-72`), empty-job reason (`74-98`), create/tick/report (`daemon.rs:645-685`), db check ok (`687-714`), script executes (`716-752`), escape refused (`754-780`), timeout killed (`782-814`), retirement (`816-836`).

**Findings:**
1. **At-least-once script side effects.** A crash after `execute_daemon_job` but before `record_daemon_tick` re-runs the script next boot; there is no per-tick idempotency key or "claimed" state. **VERIFIED by code shape.**
2. **Supervisor stall.** Scripts run synchronously inside `tick_due` (`297-300`), which is called by `tick_once` (`remote_supervisor.rs:96-101`). With `limit=64` and a 60 s cap per script, one tick can block for ~64 minutes, delaying lease expiry and the next tick. **VERIFIED.**
3. `restart_attempts`/`restart_window_at` are never written (`db.rs:808-809`) — dead columns; `set_enabled` always schedules `now` with no backoff (`229-233`).
4. `owner_id` is not verified to exist (no FK, `db.rs:798-799`); `job_json` is stored unvalidated/unbounded (`795-801`).

---

## 10. Approvals persistence

- **Two layers.** (a) In-process waiters keyed by approval id, never persisted, explicitly "never reads or mutates approval database state" (`approval_wait.rs:1-6`, map at `24-96`). (b) Durable `approvals` table in the tools crate (`approval.rs:149-205`): PK `approval_id`, `nonce UNIQUE`, run/task/tool/args hash/TTL/decision/decided_at/reason/executed_at; indexes `approvals_run_pending` and duplicate `approvals_session` on the same columns (`167-170`); partial unique one-pending-per `(run_id,tool_call_id)` created **conditionally**, refusing by name if duplicates exist (`172-204`).
- **Lifecycle.** `insert_request` commits before side effects (`211-242`); `resolve` is CAS (`414-424`), TTL-expires to `decision='expired'` (`397-412`), refuses replay (`377-395`); `session_allowed` supports `allow_session` (`436-449`). Boot reaper consumes every pending row regardless of TTL because no waiter survives restart (`tools_bridge.rs:93-105`, called from `http.rs:243-258`).
- **Out-of-band ledger.** One JSONL row per committed decision (`approval_ledger.rs:80-120`), run-scoped, arg-body-free.
- **Findings (all VERIFIED):**
  1. The table is created by `ToolsBridge::migrate` inside a **health check** (`http.rs:223-241`) — after `recover_on_boot` and outside `MIGRATIONS`, `schema_migrations`, and the pre-migration backup. A DB restored without it is repaired only when `AppState` is built.
  2. The required check can fail while the server still starts: `main.rs:126-145` logs failed checks and continues; `http.rs` records `required: true, ok: false` but nothing gates serve on it.
  3. Down-migrations to v0 drop `runs` (`db.rs:88-93`) but leave `approvals` → approvals whose `run_id` no longer exists. No FK prevents it.
  4. `nonce` is stored plaintext and unique; `approvals` has no `run_id` FK, no expiry index, and no cleanup of decided rows (unbounded growth).

---

## 11. Usage

- Schema `run_usage` (`db.rs:202-212`): six nullable counters keyed by run; FK CASCADE. NULL means "not measured" and stays NULL (`store.rs:2480`, test `3636-3652`).
- Writers: engine checkpoints and final persist (`execution.rs:1111-1145`, `1890`), each an upsert replace (`store.rs:2484-2505`).
- Readers: HTTP `/api/usage/runs/{id}` (`http.rs:3221-3234`) and run detail (`http.rs:3158`).
- Findings: totals are last-writer-wins, not additive; a crash between checkpoint and terminal write can under-report; there is no per-turn/per-provider usage ledger, so cost attribution is lossy. `run_usage` row absence is indistinguishable from "zero usage" at the HTTP layer (`history`/detail use `Option`).

---

## 12. Recovery, snapshots and history

### 12.1 Boot order (VERIFIED, `main.rs:65-193`)
1. `Database::open` (`65`) → `migrate` (`77`) → exit on failure.
2. `RemoteStore::reconcile_on_boot` (`90-102`) → exit on failure.
3. `SnapshotStore::new` + `GraphAuthority::new` → `recover_on_boot` (`103-124`) → exit on failure.
4. Environment checks are logged, not gating (`126-145`).
5. `AppState::with_graph` builds the bus, runs the tools approval migration + boot reaper (`http.rs:223-263`), publishes recovery catch-up frames (`http.rs:384-398`).
6. Bind, then spawn `RemoteSupervisor` every 2 s (`186-193`).

### 12.2 `recover_on_boot` behavior (`recovery.rs:107-336`)
- Active runs → `interrupted`, with `E_APPROVAL_TIMEOUT` set for an `awaiting_approval` run (`129-137`). Already-interrupted runs are still scanned so a crash mid-recovery finishes (`116-124`).
- Open A2A questions of **newly** interrupted runs are expired (`141-143`); failures ignored.
- Snapshots restored oldest-first so the newest image wins (`151-193`); corrupt snapshots are reported, never reset (`169-187`).
- Every non-terminal task of every interrupted run is settled to `interrupted` via the state machine (`205-251`, `385-397`); terminal tasks are repaired in the graph if they disagree (`208-229`).
- Non-terminal nodes of the restored image without task rows are settled in the graph only (`257-283`).
- One new snapshot revision is committed per changed run (`288-333`), revision = `max(index, disk) + 1` (`480-494`).
- Report fields distinguish interrupted/restored/not-restored/corrupt/settled/graph failures/snapshot failures (`41-78`); clean boot yields an empty report (`960-973`).
- Tests: settlement + revision (`recovery.rs:617-687`), one NODE_STATE per settled task (`690-737`), already-interrupted run (`740-763`), graph-only orphans (`766-811`), corrupt image (`814-848`), catch-up publisher (`851-884`), settlement target (`887-913`), idempotent second boot (`916-943`), monotonic revision (`946-957`), clean boot (`960-973`).

**Findings:**
1. **500-run ceiling.** `store.list_runs(500)` (`recovery.rs:119`) silently ignores older non-terminal runs. They stay active forever, keep returning from `latest_active_run` (`store.rs:438-452`), and block run-memory clear with `E_RUN_ACTIVE` (`http.rs:1492-1494`). **VERIFIED.**
2. `expire_open_agent_messages` is best-effort and only for newly interrupted runs; an already-interrupted run's open questions survive another boot. **VERIFIED.**
3. Settlement failures in the graph are recorded but do not fail boot (`233-240`), so DB and graph can diverge until the next boot's repair pass.

### 12.3 Snapshots (`snapshot.rs`)
- Schemas `hivemind.snapshot.v1` / `v2` (`23-25`); v2 adds `run_status` (`38-42`); v1→v2 inserts `run_status: null`; v2→v1 removes it (`262-303`). Tests: round-trip, unknown version refused, metadata preserved (`snapshot_history.rs:43-104`).
- Write: validate schema, `create_dir_all`, temp file, `write_all`, `sync_all`, rename, dir fsync on Unix, then index (`77-118`). Reads validate id/revision (`125-134`), migrate on load, and never reset the workspace.
- Prune keeps newest `keep` (`206-231`), deletes file then index row; called only at terminal commit with keep=16 (`execution.rs:1879-1887`).
- Tests: corruption refusal (`snapshot_history.rs:106-128`), history HTTP (`179-261`), prune (`263-289`), kill-restore hard abort (`kill_restore.rs:20-89`).

**Findings:**
1. **Revision collision.** Runtime revision = number of indexed rows (`execution.rs:1732-1738`), not `max+1`. If a file exists without its index row (index insert failed after the file rename, or index row was pruned/lost), the next write reuses a revision number and **silently overwrites that history file and index row** (`snapshot.rs:83`, `96`, `104`). Recovery's own `next_revision` does it correctly (`recovery.rs:480-494`) — two implementations disagree. **VERIFIED.**
2. **History masks corruption.** `HistoryReadModel::revisions` does `.read(...).ok()` (`history.rs:80`), so a corrupt/missing revision is presented with `run_status: None` instead of the named `E_SNAPSHOT_CORRUPT` — contradicting the module's "never silently reset/refuse" contract. **VERIFIED.**
3. `write` overwrites an existing revision without a monotonic check; only call-site discipline prevents rewrites.

### 12.4 History read model
- `runs` delegates to `RunStore::list_runs` (`history.rs:67-69`); `revisions` from the SQLite index (`75-88`); `state_at` checks the **disk** list then reads (`109-115`), so index/disk disagreement changes behavior between `revisions()` and `state_at()`.
- `E_HISTORY_GAP` for unknown revisions (test `snapshot_history.rs:155-158`), `E_RUN_NOT_FOUND` for unknown roster run (`snapshot_history.rs:254-260`).

---

## 13. Tests: what each proves (source-level) and its blind spots

| Test file | Proves | Blind spot |
|---|---|---|
| `db.rs` unit tests `1462-1805` | Fresh DB v0; migrate idempotent → 23; down→up; file persistence; unknown down target; tool-turn round-trip/dup; rollback+reapply; `SchemaNewer`; backup + integrity | In-memory for most; no test for v19 upgrade collision, backup pruning, down-path backup absence, or contiguity gaps |
| `store.rs` unit tests `2883-3653` | Terminal stickiness; rewire rev; A2A budget/CAS/attempts/expiry; journal chain/tamper/truncate; trust promotion journaling; one durable run owner; identity/ownership triggers; read-set sibling exclusion; digest visibility; branch/run scoping; projects; usage NULLs | No test for scoped-write atomicity, role-clear bank residue, v14 agent_role reachability, digest journaling, read-set starvation |
| `mem_isolation.rs` | Fail-closed identity; cross-run bank refusal; validated write bank routing; vocabulary refusal | Validated path is test-only; no test that the *engine* path obeys the same rules |
| `snapshot_history.rs` | v1↔v2 round-trip; corrupt refusal; HTTP history; terminal prune | No revision-collision test; no corrupt-index-row test |
| `daemon_tick_degraded.rs` | Degraded honesty, no tick-window left, no task row, specific empty-job reason | No crash-between-execute-and-report test |
| `remote_control_plane.rs` | Resolution revalidation (revoke/lease/device), approval sweep + audit, lease audit, boot reconcile idempotence, state chaining | No `create_remote` idempotency test; no over-grant test |
| `kill_restore.rs` | Real hard-kill (abort) → interrupted + restored nodes; corrupt-at-boot refusal | Single run only; no >500-run case |
| `chat_store.rs` | Chat order/durability; run status/active tracking; task migrations; receipts; v2 down/up | Receipt 24 h expiry untested; task bank creation failure untested |
| `http_boot.rs` (memory routes) | Journal contains actions/no values (`123-149`); context metadata (`164-194`); clear scoped + terminal-only (`258-322`); banks/summary/entries (`325-427`); run memory scope (`718-806`); role list/clear flat only (`1190-1241`) | No unauthenticated-GET test; no clear-with-governance test; role clear asserts only the flat table |
| `approval_ledger.rs` | One ledger row per decision; replay writes none; no arg body | Partial read (rest not reviewed); no boot-reaper ledger interaction test |

---

## 14. Migration safety / reversibility assessment

| Dimension | Result | Evidence |
|---|---|---|
| Forward-additive | 22/23 additive; v11 is a rebuild (create/copy/drop/rename) | `db.rs:249-274` |
| Atomic per step | Yes (transaction per migration + version insert) | `db.rs:1032-1043` |
| Backup before upgrade | Yes, when pending and `current>0`, verified with `integrity_check` | `db.rs:1025-1027`, `1051-1099` |
| Backup before downgrade | **No** | `db.rs:1106-1135` has no `backup_before_migration` call |
| Backup retention | **None** (one file per boot attempt that finds pending migrations) | `db.rs:1075-1084` |
| Schema-reversible | Yes — every migration has `down`; round-trip tested | `db.rs:1504-1516`, `1679-1730` |
| Data-reversible | **No** — v2/v7/v11/v13/v14/v15/v16/v17/v18/v20/v23 downs drop data or columns | cited down blocks |
| Forward compatibility | `SchemaNewer` refused by name | `db.rs:1022-1024`, `1733-1757` |
| Deterministic-failure safety | A failing v19 blocks boot permanently (backup exists but no auto-repair) | `db.rs:637-638`, `main.rs:77-87` |
| Version-table integrity | No contiguity/name validation | `db.rs:987-996`, `1141-1156` |
| Tooling outside the engine | `approvals` table unversioned, un-backed-up | `tools_bridge.rs:87-91`, `http.rs:223-241` |

**Verdict:** safe for the normal single-step upgrade on a healthy DB; **not** safe as an operator runbook for downgrade or for a legacy DB that violates the v19 uniqueness assumption.

---

## 15. Memory access-control verdict (summary)

- **Model ↔ model (within a run): controlled.** Bank selection is derived from the persisted task identity, sibling branches are unaddressable, cross-run scoped reads are refused, and all four properties have tests.
- **Model → memory (write): not controlled in practice.** The strict writer is unused; the live writer is permissive, non-transactional, and accepts a caller-supplied `scope_id`. The fail-closed bank fallback prevents the worst leak (unknown identity → own branch), which is why the isolation tests still pass.
- **Operator/host → memory (read): not controlled.** GET memory routes are Host-gated only; session auth applies to mutations, and the launcher token is optional (`http.rs:733-766`). Any local process that can reach the loopback port can read raw memory values, including the copied run prompt.
- **Erasure/retention: not controlled.** Role clear misses bank entries; run clear misses journal, context snapshots, agent_role banks, and the global lesson artifact.

---

## 16. Top-10 bugs / risks

Severity: **S1** data loss or governance integrity, **S2** incorrect durable state or blocked boot, **S3** retention/audit/availability defect. "Fix location" names the exact current file/lines.

| # | Sev | Finding | Evidence | Fix location |
|---|---|---|---|---|
| 1 | S1 | **Governance records can be orphaned by run-memory clear.** Guard scans only newest 256 run/hive entries and only 2 of 5 prefixes; lesson file survives the DB delete, so an applied lesson becomes unrollbackable. | `http.rs:1497-1507`; `store.rs:1781-1785`; rollback dependency `http.rs:2870-2877`; keys `2383-2407`; file `2692-2694` | Move the guard into the same transaction as the delete: add `store.rs` method that `SELECT 1 FROM memory_entries WHERE bank_id IN (…) AND (entry_key LIKE 'evolution\\_%' ESCAPE '\\' OR entry_key LIKE 'improvement:%')` and refuse; guard `evolution_rollback:` too. Long-term: §17 `governance_records` + `ON DELETE RESTRICT`. |
| 2 | S1 | **Role memory clear reports success but leaves model-readable bank entries.** `memory_entries` has no agent key, so entries cannot be selected for deletion. | `store.rs:2626-2637`; schema `db.rs:411-431`; read-set `store.rs:2197-2203`; test asserts flat only `http_boot.rs:1232-1240` | `store.rs:2626`: delete bank entries by `source_task_id IN (SELECT id FROM tasks WHERE role = ?)` **or** add `agent_key TEXT` to `memory_entries` in a new migration and delete by it; journal the erasure. |
| 3 | S1 | **The strict memory writer is dead code; the live writer bypasses vocabulary and bank validation.** | `store.rs:1513-1619` (no prod callers; grep shows tests only); live path `execution.rs:1955-1971`, `5572-5578`; permissive `store.rs:1329-1354`, `1467-1468` | Replace engine calls with `record_memory_entry` (extend it with `kind`/`trust` args and an engine-actor allowance); delete or `pub(crate)`-restrict `record_agent_memory_scoped`; add an integration test that the engine path cannot write `quarantined` or a foreign bank. |
| 4 | S2 | **Scoped memory write is non-atomic** — entry can exist without journal row, or the prior active row can be deleted before the replacement lands. `verify_memory_journal` cannot detect either. | `store.rs:1355-1492` (no transaction) vs `1564`; verify `2072-2123` | Wrap `store.rs:1356-1491` in `connection.transaction()` exactly like `record_memory_entry`; commit after `append_memory_journal`. |
| 5 | S2 | **Journal is not a complete governance record**: `write_run_digest` and `clear_run_memory` are unjournaled; prune journals blank `bank_id`/`entry_id`. | `store.rs:1732-1765`; `1773-1799`; `2159-2169` | Add journal appends (`run_digest_written`, `run_memory_cleared`) and pass real ids for prune; make journal append mandatory inside each write transaction. |
| 6 | S2 | **v19 can permanently block boot** on a DB with two `hyper` tasks in one run; each failed boot writes another unreaped `.bak`. | `db.rs:573-575`, `637-638`, `1025-1043`; boot exit `main.rs:77-87`; no backup pruning `1075-1084` | Before creating the index, pre-check and refuse with a named `E_SCHEMA_CONFLICT` (or keep only the lowest `spawn_seq` and journal the demotion). Add backup retention (keep N newest per version). |
| 7 | S2 | **Snapshot revision = row count**, so an unindexed/removed revision is silently overwritten by `INSERT OR REPLACE` + file rename. | `execution.rs:1732-1738`; `snapshot.rs:83`, `96`, `104`; contrast `recovery.rs:480-494` | Delete `snapshot_revision`; call a shared `next_snapshot_revision` that returns `max(indexed_max, disk_max)+1`, and make `write` refuse a revision `<= latest`. |
| 8 | S2 | **Boot recovery only considers the newest 500 runs**; older active runs stay active, keep `latest_active_run` non-null, and block run-memory clear forever. | `recovery.rs:119`; `store.rs:438-452`; `http.rs:1492-1494` | Add `RunStore::active_run_ids()` (`WHERE status IN (…)`) and use it in `recovery.rs:119`; or loop pages until exhausted. |
| 9 | S3 | **Legacy `agent_role` banks are unreachable and unprunable** (bank `run_id NULL`), so pre-v14 memory is invisible to prompts and immune to retention. | `db.rs:454-465`, `477-488`; `store.rs:1783`, `2144`, `2312`; read-set `2202` | Backfill `memory_banks.run_id` for agent_role banks from `memory_entries.source_run_id` in a new migration, or move those entries into per-run banks; add a `memory_banks.kind='agent_role'` reader for the role surface. |
| 10 | S3 | **Approvals table lives outside the migration engine** (created from a health check, unversioned, un-backed-up, no down path; down-to-0 leaves orphan approvals). | `hiverouter-tools/src/approval.rs:149-205`; `http.rs:223-241`; `db.rs:88-93`; failed check not gating `main.rs:126-145` | Register the approvals DDL as migration v24 in `db.rs` (keeping the conditional unique index as a post-step with named refusal), drop `ToolsBridge::migrate` from the health check, and gate serve on the required check. |

**Secondary defects (not in top 10, all VERIFIED):**
- `history.rs:80` hides `E_SNAPSHOT_CORRUPT` behind `Option::None`.
- `memory_context_snapshots.prompt_sha256` hashes only the memory block (`execution.rs:4138`); `truncated` always false (`4145`); table never pruned.
- `find_by_request_id` unused → duplicate remotes on retry (`remote.rs:424-448` vs `316-380`).
- `in_reply_to` never written (`db.rs:284`; reads only).
- `expire_agent_message`/`set_agent_message_deadline` return `Ok` on no-op (`store.rs:1049-1104`); expired transitions unjournaled.
- Daemon scripts are at-least-once and can block the supervisor ~64 min (`daemon.rs:297-300`; `remote_supervisor.rs:99`).
- `set_run_status` hard-codes terminal states (`store.rs:491`) instead of deriving from `run_state` (`run_state.rs:76-81`) → `killed` run rows become un-updatable and misreported.
- `create_grant` ignores `capabilities_json` (`remote.rs:599-660`).
- `remote_agents.state` CHECK allows `degraded`/`quarantined` that no writer produces (`db.rs:673-674`).
- `daemons.restart_attempts/restart_window_at` never written (`db.rs:808-809`).
- `migrate()` accepts gapped/renamed `schema_migrations` rows (`db.rs:987-996`, `1028-1042`).
- `ChatMessage`/`Run`/`TaskRecord` decoders fabricate nil ids and epoch timestamps on corrupt rows (`store.rs:757-771`, `2825-2831`, `2844-2853`).
- Duplicate index definitions `approvals_run_pending` / `approvals_session` on identical columns (`approval.rs:167-170`).

---

## 17. PROPOSED safe schema change for task instructions

**Problem being fixed (VERIFIED):** the exact instructions a model received are not reconstructible. They are scattered across `runs.prompt` (plaintext, `db.rs:56`), a truncated run-bank copy (`execution.rs:1955-1964`), the global `LESSONS.jsonl` (`http.rs:2692-2694`), and injected memory entries (`execution.rs:4115-4147`). The only audit row claims a `prompt_sha256` but hashes just the memory block (`execution.rs:4138`), and `memory_context_snapshots` is never pruned.

**Proposed migration v24 `instruction_provenance` (additive; no rewrite of existing rows):**

```sql
ALTER TABLE runs ADD COLUMN prompt_sha256 TEXT;               -- hash of the user prompt exactly as accepted
ALTER TABLE memory_context_snapshots ADD COLUMN prompt_sha256_full TEXT;  -- hash of the exact assembled prompt
ALTER TABLE memory_context_snapshots ADD COLUMN truncated_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_entries ADD COLUMN agent_key TEXT;         -- enables role-scoped erasure (finding #2)

CREATE TABLE task_instructions (
    instruction_id  TEXT PRIMARY KEY NOT NULL,
    run_id          TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    task_id         TEXT,
    kind            TEXT NOT NULL CHECK (kind IN
                      ('user_prompt','system_prompt','lesson','memory_block','a2a_delta','tool_result')),
    content         TEXT NOT NULL,
    content_sha256  TEXT NOT NULL,
    source          TEXT NOT NULL CHECK (source IN
                      ('operator','engine','lesson_ledger','memory_bank','provider')),
    source_ref      TEXT,                                      -- lesson sha / entry id / message id
    seq             INTEGER NOT NULL,
    created_at      TEXT NOT NULL
);
CREATE UNIQUE INDEX ux_task_instruction_seq ON task_instructions(run_id, task_id, seq);
CREATE INDEX idx_task_instruction_hash ON task_instructions(run_id, content_sha256);

CREATE TABLE governance_records (
    record_id        TEXT PRIMARY KEY NOT NULL,
    run_id           TEXT NOT NULL REFERENCES runs(id) ON DELETE RESTRICT,
    kind             TEXT NOT NULL CHECK (kind IN
                       ('proposal','vote','authorization','lesson_apply','lesson_rollback')),
    proposal_id      TEXT NOT NULL,
    content_sha256   TEXT NOT NULL,
    artifact_sha256  TEXT,
    actor            TEXT NOT NULL,
    created_at       TEXT NOT NULL
);
CREATE UNIQUE INDEX ux_governance_once ON governance_records(kind, proposal_id)
    WHERE kind IN ('authorization','lesson_apply','lesson_rollback');
CREATE INDEX idx_governance_run ON governance_records(run_id, kind);
```

Down (schema-reversible; SQLite `DROP COLUMN` is already used by v4/v12/v13/v23 downs):
```sql
DROP INDEX IF EXISTS idx_governance_run;
DROP INDEX IF EXISTS ux_governance_once;
DROP TABLE IF EXISTS governance_records;
DROP INDEX IF EXISTS idx_task_instruction_hash;
DROP INDEX IF EXISTS ux_task_instruction_seq;
DROP TABLE IF EXISTS task_instructions;
ALTER TABLE memory_entries DROP COLUMN agent_key;
ALTER TABLE memory_context_snapshots DROP COLUMN truncated_count;
ALTER TABLE memory_context_snapshots DROP COLUMN prompt_sha256_full;
ALTER TABLE runs DROP COLUMN prompt_sha256;
```

**Wiring (PROPOSED, in order):**
1. `create_run` computes `prompt_sha256` and writes it in the same INSERT (`store.rs:370-386`).
2. A single `assemble_prompt()` in `execution.rs` returns `(wire_prompt, Vec<InstructionPart>)`; `record_memory_context_snapshot` receives `sha256(wire_prompt)` and the real truncated count; `task_instructions` rows are inserted in the same DB transaction as the snapshot.
3. Evolution apply/rollback insert `governance_records` in the DB transaction **before** the atomic file rename; a crash then leaves at most a DB row without a file line (detectable and repairable) rather than a live lesson with no record.
4. `clear_run_memory` refuses when `governance_records` has a row for the run (or returns `E_APPROVAL_REQUIRED`), removing the prefix-scan heuristic.
5. `clear_agent_memory` deletes `memory_entries WHERE agent_key = ?` and journals it.
6. Retention: a new `prune_context_snapshots(run_id, keep)` called from the terminal path next to `snapshot.prune_run` (`execution.rs:1879-1887`).

**Verification gates (PROPOSED):** migration round-trip test (down→up); a test that a run with one applied lesson cannot be cleared; a test that role clear removes both flat and bank rows; a test that a missing/removed snapshot index row can never cause revision reuse; a kill-early test between lesson-file write and DB write.

**Risk note:** `ALTER TABLE memory_entries ADD COLUMN agent_key` is additive, but legacy rows will have NULL `agent_key`; the role-clear fix must additionally fall back to `source_task_id → tasks.role` for those rows, or the erasure gap persists for migrated data.

---

## 18. Explicit gaps and UNVERIFIED statements

| Item | Status | What would prove it |
|---|---|---|
| All 23 migrations apply cleanly on a real DB | **UNVERIFIED** — SQL read only | `cargo test -p hiverouter-core --test chat_store` and a fresh `migrate()` run |
| v19 collision actually blocks boot on a legacy DB | **UNVERIFIED** — inferred from DDL + trigger backfill | Seed two `hyper` tasks with `ownership_mode='run'`, run `migrate()` |
| Governance orphan after >256 entries | **UNVERIFIED at runtime** — code path verified | Integration test: apply lesson, write 300 run entries, POST `/api/memory/clear`, then attempt rollback |
| Role-clear leaves bank rows | **VERIFIED in code**; no runtime test | Call `clear_agent_memory`, then `memory_read_set` |
| Memory GET routes reachable without token | **UNVERIFIED at runtime**; verified in middleware code | Start server without launcher token, `curl /api/agents/runs/{id}/memory` |
| Test suite status | **UNVERIFIED** — not run | `cargo test --workspace` |
| Multi-run recovery / governance under suspension | **UNVERIFIED** — `recovery_multi_run.rs`, `recovery_gates.rs`, `suspension.rs`, `hyperagent_governance.rs` not read | Read those files; run them |
| Approval table behavior on restored pre-v21 DB | **UNVERIFIED** | Restore an old DB copy, boot, inspect `sqlite_master` |
| Snapshot/revision collision in production | **UNVERIFIED at runtime**; structural defect verified | Delete one `snapshots` index row, commit another revision, observe overwrite |
| `GitNexus` impact analysis for these findings | **NOT RUN** — read-only lane, no edits; graph tools not in this session | Run `impact` on the proposed change targets before implementing |

---

## 19. Delivery status (report shape)

**Done** — this report: full read of `db.rs`, `store.rs`, `entities.rs`, `history.rs`, `snapshot.rs`, `recovery.rs`, `daemon.rs`, `remote.rs`, `remote_supervisor.rs`, `approval_wait.rs`; targeted read of `http.rs`, `execution.rs`, `main.rs`, `run_state.rs`, `tools_bridge.rs`, `hiverouter-tools/src/approval.rs`; full read of the seven mandated test files (with `http_boot.rs` selected ranges); 23/23 migrations enumerated; all 28 durable tables listed; top-10 risks with exact fix locations; proposed v24 schema.

**Not done** — no test/build/migration executed (read-only mandate); `recovery_multi_run.rs`, `recovery_gates.rs`, `suspension.rs`, `hyperagent_governance.rs`, `run_lifecycle.rs`, and the rest of `approval.rs` unread; no live DB/WAL/snapshot/lesson file inspected; no GitNexus graph queries (not available in this session).

**Unverified** — runtime pass/fail of every cited test; the concrete crash windows (#4, #6, #7, #8) are structural inferences, not reproduced; v19 boot-block and the >256-entry governance bypass are code-path verified but not exercised.
