# GAP AUDIT 1/2 — backend / build / persistence / API / tests

**Auditor:** Gap Auditor 1/2 (read-only lane).
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` — HEAD `6a41250`, branch `master`; `git status --porcelain` = 2 paths (`?? docs/analysis/`, `?? docs/plans/HIVEMIND-FINISH-PLAN-v3.md`).
**Inputs read:** all seven lane reports, `WATCHDOG-LEDGER.md` + `WATCHDOG-LEDGER.jsonl` (100 rows), `CODEBASE-CENSUS.md`/`.json`/`.mmd`, `LEAD-RESEARCH-PSEUDOCODE-LANGUAGES.md`, plus every file named in the task.
**Execution:** none. No `cargo`, `npm`, `node`, server, browser, provider, or DB open. Every "passes" statement below is a **read**, not a run.
**Writes:** this file and `docs/analysis/GAP-PATCH-1.md` only. No source, test, config, ledger, or plan was modified.

---

## 0. Verdict and counts

This audit **confirms the watchdog's core corrections** (L2 `finish`/`report_result`, L5 header 401, route count, `hivemind-cli.mjs` size) but finds **three watchdog corrections that are themselves wrong or incomplete**, and **five new gaps no lane covered**:

| Class | Count | Headline |
|---|---:|---|
| Watchdog/lane claims re-verified | 14 | 11 CONFIRMED, 3 need correction |
| New gaps (no lane owns) | 5 | daemon env inheritance; plan-vs-caps ordering; history epoch fallback; SPA `/api` fallback; handshake dead module |
| Test bodies now read | 5 files, 12 tests | 4 support their citation; `scale_s1.rs` does **not** assert the bound |
| Mandatory files read | 16 | build.rs, handshake.rs, approval_wait.rs, bus.rs, history.rs, static_ui.rs, caps.rs, config.rs, main.rs, lib.rs, planner.rs, remote_supervisor.rs, daemon.rs, remote.rs, context.rs (+ test bodies) |

**Claim ledger for this report:** `G-01` … `G-30` in §7. Every row has a status from the watchdog's own vocabulary plus `NEW` for gaps the lanes did not report.

---

## 1. Mandatory gap targets (read, not cited from census)

### 1.1 `backend/crates/hiverouter-core/build.rs` — the census/watchdog "unowned" file

Behavior, exactly (`build.rs:14-40`): build script runs `${RUSTC:-rustc} --version` (:18-24), parses the **major.minor** only (:29-31, :43-48), panics with `E_TOOLCHAIN_MSRV` if `major < 1 || (major == 1 && minor < 89)` (:32-37), and injects `cargo:rustc-env=HIVEROUTER_BUILD_RUSTC=<full version line>` + `HIVEROUTER_MIN_RUST_MINOR=89` into the crate (:38-39). `checks.rs:58-68` consumes both via `env!`; `msrv_check()` is hardcoded `Check::pass` (:59-60). No network, no file writes, no asset mutation — the census's implied "does it rewrite embedded assets" risk is **not present**.

Real risks found:

1. **Nightly/beta/RC toolchains are rejected although new enough.** `rustc 1.93.0-nightly (…)` parses `minor` as `"0-nightly"` → `parse::<u32>()` fails → `E_TOOLCHAIN_MSRV: could not parse` panic (`build.rs:29-31,44-48`). Only stable `X.Y.Z` works. Not documented in `README.md`/`rust-toolchain.toml` (not read to conclusion).
2. **Patch level is not enforced.** `1.89.0` is required in the message (:34-36) but `1.89.5` and a hypothetical `1.89.0-alpha` both pass/fail inconsistently; only the minor is compared.
3. **The full version string (compiler hash/date) is embedded** (`build.rs:38`) and printed by `/health` via `checks.rs:64`; the same source rebuilt on a different patch toolchain yields a different binary/report → **release reproducibility is impossible by construction** (this is a real finding for the release gate, which hashes the binary).
4. **`RUSTC` is executed from the environment** (`build.rs:18-24`) with `rerun-if-env-changed=RUSTC` only (:16). That is standard cargo trust, but it means a poisoned `RUSTC` gets arbitrary build-time execution; not a product-boundary bug, a CI hygiene item.
5. **Ownership contradiction:** census §A lists `build.rs` as lane `L4` (`CODEBASE-CENSUS.md:782`), but L4 never mentions `build.rs` (grep: 0 hits). Census §15.3's "25 unowned paths" excludes it because it is not a product asset (`CODEBASE-CENSUS.md:522,772-785`). The watchdog's statement "unowned, read by no lane" is **CONFIRMED in practice**; the census table is aspirational. This file has no behavioral test either.

**Do not delete**: `checks.rs` uses the two `env!` outputs; removing the build script breaks the crate.

### 1.2 `src/handshake.rs` — dead production module

`HandshakeFrame`/`HandshakeLog` are re-exported by `lib.rs:19,49` and exercised only by `tests/handshake.rs` (25 grep hits). **No production caller exists** — not `ws.rs`, not `http.rs`, not `execution.rs`. The module doc claims "`HandshakeFrame` is carried by the single Phase 2 transport" (`handshake.rs:3-7`); that transport does not carry it. Consequences:

- The claimed depth cap is duplicated and unenforced by this module: `MAX_HANDSHAKE_DEPTH = 3` (:19) vs `caps.rs:9` vs `planner.rs:67` (literal `3`). Three copies, one authority.
- Stale doc/code drift: `HandshakeLog::new` says "oldest ids forgotten … via the replay-order vector" (:147-149); there is **no vector** — eviction is `seen.keys().min()` (:169-174), which is "oldest" only if every correlation id is UUID v7; `Uuid` is not constrained to v7 in the type (`:69`), and `tests/handshake.rs` may use v4.
- Idempotency is per-process only; a restart forgets every correlation id (no persistence, no ledger), so a replayed child frame after boot is `Processed`.
- `validate()` does not check `capability_set` against `ToolRegistry`/tier (`:80-94`), does not compare `depth`/`parent_task_id` to the stored parent row (doc says "caller derives" but the caller doesn't exist), and accepts any non-nil `correlation_id`.
- `ChildResult` only has `status` + `text` (`:121-128`); no result schema id, no usage, no tool turns.

**Gap:** either wire the handshake or delete it and its test; leaving a public, documented-but-dead protocol is a false capability claim.

### 1.3 `approval_wait.rs` — used, but cancellation arm is dead code

Used by `execution.rs:153,704,779,997,1032,3193-3200` (the wait arm). Findings:

- `wake_cancelled` (`approval_wait.rs:63-65`) has **no production caller** (grep: only its own tests). Cancellation reaches the wait through `token.cancelled()` in the `select!` (`execution.rs:3240-3244`), so `ApprovalWake::Cancelled` is only constructible from tests.
- `arm` is correctly before publish (`execution.rs:3192-3200`); the drop guard disarms. But the map is keyed by approval id only; the "one run = one live wait" invariant comes from `run_state::Suspend` being legal only from `Running` (`run_state.rs:510-512`) + the single run status, **not** from the waiter map. A second concurrent suspend in another task gets `E_SCHEMA_INVALID` and the run fails (`execution.rs:3106-3112` per L1). That is L1's HIGH finding and remains **CONFIRMED by code**.
- `wake_resolved` returns `false` if the receiver dropped; `take_and_send` removes the entry even on send failure (`:88-95`). A late decision after a dropped receiver is silently lost; the timeout path then sees `decision` set and executes the committed label (`execution.rs:3447-3471`). This is the same fail-open shape as §3.2.
- Poisoned mutex → `InternalError`; tests do not cover poisoning.

### 1.4 `bus.rs` — sequencing is sound; replay/backpressure has holes

- Sequence assignment and replay insertion are serialized by `publish_lock` (`bus.rs:110-133`), serialization happens **before** `fetch_add` (:116-117) so a failed publish cannot create a gap. `current_seq` reads the replay tail, not the counter (:171-184) — correct.
- `publish_with_correlation` and `publish_for_run` are near-duplicates (:105-169); a future fix in one will drift.
- Poison handling is asymmetric: `publish*` returns `InternalError` if the replay mutex is poisoned (:124,:161), while `replay_after` recovers with `poisoned.into_inner()` (:209-212). A panic in one publisher can therefore stop all future publishes but not reads.
- `REPLAY_CAPACITY = 256` vs `FANOUT_CAPACITY = 1_024` (:18-20). A subscriber lagging more than 1,024 events gets `Lagged`; whether `ws.rs` maps that to `E_RESYNC_REQUIRED` was not read (residual unknown).
- No retention of "run start" pseudo-events; after restart `seq` restarts at 1, so any client holding `last_seq > 0` gets `Replay::Gap` (expected, but the UI must resnapshot — `ws.rs` not audited).
- Tests cover ordered sequence, replay gap, subscribers (:245-303); no test for poisoned-lock behavior, no test for `Lagged`.

### 1.5 `history.rs` — silent epoch fallback and O(n) disk reads

- **Silent data fabrication:** `Timestamp::parse(&created_at).unwrap_or_else(|_| Timestamp::from_unix_secs(0))` (`history.rs:79`). A corrupt/unparseable `created_at` in the SQLite index is rendered as **1970-01-01** instead of `E_SNAPSHOT_CORRUPT`/`E_INTERNAL_ERROR`. This contradicts the project's fail-closed posture and is a UI-visible lie. It is untested.
- `revisions()` reads **every snapshot file from disk** per HTTP call (`:75-88`): one `snapshots.read` per indexed revision, no cap on revision count, no cache. A run with hundreds of revisions makes `/api/history/runs/{id}/revisions` O(n) file opens; there is no limit/cursor on the history endpoints (L4 noted fixed 200-row runs).
- `state_at` does `list_revisions` then `read` (`:109-115`) — TOCTOU with `prune_run`; the read still refuses, but the error is `E_HISTORY_GAP` vs `E_SNAPSHOT_CORRUPT` depending on timing (minor).
- `TimeTravel::revision_count` is `revisions.len()` (real, good).

### 1.6 `static_ui.rs` — traversal claim holds; one route-shape hole

- `WebAssets::get(requested)` with rust-embed exact-path lookup (:52-59) and the fallback only serving `index.html` for paths without `.` (:61-70) — the module's "path traversal cannot escape `web/dist`" claim (:3-7) is **sound for exact-path lookup**; no filesystem call exists.
- **`/api` (no trailing slash) is not caught by `path.starts_with("api/")`** (`:48`), has no dot, so it serves `index.html` with **200** instead of the honest JSON 404. Concrete: `GET /api` → SPA HTML. Unknown `/api/x` → JSON 404 (:49).
- Non-GET/HEAD returns `405 text/plain` (:43-45), not the JSON refusal envelope other paths use (:74-82) — a contract inconsistency.
- HEAD is allowed and returns the body bytes; hyper normally suppresses the body, but this is not tested.
- Compile-time embed of `../../../web/dist` (:14-16) means a backend rebuild silently pins the UI; there is no build-time assertion that `web/dist/index.html` matches the current web source (test only checks `<div id="root">` exists, :98-103).

### 1.7 `caps.rs` / `planner.rs` — the cap arithmetic, re-derived

- `Caps::check_calls` refuses only `calls > 64` (`caps.rs:69-74`); `check_spawn` uses `siblings >= 8` / `agents >= 64` (:56-61). The `>` vs `>=` is intentional for a 1-based call counter incremented before check (`execution.rs:796-804`) but undocumented — an easy off-by-one trap.
- `MAX_CALLS = 64` counts **provider calls**; `MAX_CALLS_PER_RUN = 40` (`tool_loop.rs:16`) counts **tool calls** — two different caps, same word "call". `planner.rs:9` says "40-call ceiling" for **40 plan nodes**, adding a third meaning. This terminology collision is the root of L1's imprecise arithmetic (see §4.1).
- **New gap: plan validation does not enforce spawn caps.** `Plan::validate` checks ≤40 nodes, unique ids, deps, parent depth/cycles (`planner.rs:50-129`), but **not fan-out ≤8 per parent or total agents ≤64**. Those are checked only during dispatch (`execution.rs:2025-2029`), so a plan that validates can still fail the run with `E_FANOUT_LIMIT`/`E_AGENT_LIMIT` **after** the planning provider call and after earlier task rows were inserted. Fail-at-validate is available and cheap.
- `planner.rs:67` hardcodes `node.depth > 3` instead of `caps::MAX_SPAWN_DEPTH`; three copies of "3" (§1.2).
- `topo_order` is Kahn with `ready.pop()` (LIFO, :146-165); deterministic for a given input order, not sorted. `plan.validate` calls `topo_order` then compares lengths (:125-128) — correct cycle refusal.

### 1.8 `config.rs` / `main.rs` / `lib.rs`

- `default_workspace_root` = process CWD (`config.rs:428-432`); default `data_dir` = `"data"` relative to the same CWD (`:12,487-488,566-570`). Therefore by default **`data/` is inside the workspace jail** — the precondition for the jail asymmetry in §3.3.
- Bind is loopback-only by IP (`config.rs:558-565`); `allowed_origins` derives from the bound port and is empty for port 0 (`:434-444`). `--bind/--data-dir/--db/--workspace-root` accept both `--x v` and `--x=v`; unknown args error (`:494-525`). `--help/--version` are intercepted in `main.rs:33-47` **before** config parsing, so `hiverouter --bogus --help` exits 0 and hides the bad arg (minor UX).
- `--db` may point outside `data_dir`; snapshots and the approval ledger still live in `data_dir` (`snapshot.rs:55-57`, `tools_bridge.rs:341-342`) — a split-brain config that is neither refused nor warned.
- `Config::resolve` (test helper) forces `provider.enabled = false` (`config.rs:539-544`); production uses `from_args` (main.rs:49). Good test isolation.
- `main.rs`: audit guard is initialised before DB open (`:57-63`) and held for the process; DB open/migrate failures are fatal with refusal codes (:65-87); remote boot reconciliation is fatal (:91-102); `recover_on_boot` is fatal (:105-124); environment checks are only **warned** (:126-145) — a missing key/low disk does not stop boot. That matches "checks are diagnostics", but the README's "fail loudly" could be read otherwise; state it.
- Tracing is initialised by `hiverouter_tools::init_json_audit` (`tools/audit.rs:73-82`), so the `RUST_LOG` line in `main.rs:246` is **true**, not a dead promise (this was a candidate finding; refuted by `audit.rs`).
- Shutdown: only `ctrl_c()` is installed (`main.rs:217-223`) plus the supervisor token; Windows service stop / console close are not handled. The supervisor is awaited without a timeout (`:203-204`), but its loop observes the token (`remote_supervisor.rs:64-75`).
- `lib.rs` re-exports `HandshakeFrame/HandshakeLog` (`:49`) — the dead module is public API; removing it is semver-visible (1.2.0-dev, so allowed).
- `lib.rs:5` `#![forbid(unsafe_code)]` — good; `missing_debug_implementations` warn.

### 1.9 `remote_supervisor.rs`

- Tick = expire leases + expire device approvals + `daemons.tick_due(64)` (`:96-101`); 64 is a magic literal, no config.
- Backoff doubles 2s→30s (`:168-174`); on success records `backoff_ms=0` and `error_code=NULL` (`:77-91`).
- `record_state` computes `next_due_at = now + max(backoff_ms/1000, 2)` **regardless of the configured interval** (`:116-126`); with the production 2s interval it happens to agree, with any other interval the UI's "next due" is wrong.
- `record_state` is an `UPDATE … WHERE id = 1`; if the singleton seed row is missing, it silently updates 0 rows (`:127-139`) and `state()` returns `QueryReturnedNoRows` (`:145-165`). No test covers a missing row.
- All `record_state` calls in `run()` are `let _ = …` (:61,80,84) — durable supervisor observability can fail silently while the lane keeps running.
- `lane_started_at` is set once and never cleared on shutdown; there is no "stopped" state, so the UI cannot distinguish "lane dead" from "lane idle" (the `last_tick_at` staleness is the only signal). This is a real observability gap.

### 1.10 `daemon.rs` — **new HIGH: script daemons inherit the server environment**

- `DaemonStore::create` validates name/owner/kind/interval but **not the job JSON** (`daemon.rs:148-187`); a script job is `{"kind":"script","program":"x.ps1","args":[…]}`.
- `execute_script_job` canonicalises a workspace-root allowlist, requires a plain filename, allows only `.exe`/`.ps1`, ≤16 args, each ≤512 chars, timeout 1-60s, capped output (:333-498). Path escape tests exist (:756-814).
- **`.ps1` runs via `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File <path>` (`:431-433`); `.exe` runs directly (`:428-429`). The child is spawned with `.args(...).stdin(null).stdout/stderr(piped)` but **no `.env_clear()`/allowlist** (`:436-445`). It therefore inherits the server process environment, which per `dist/HiveMind/README.txt:11-13` includes the DPAPI-decrypted `DEEPSEEK_API_KEY` injected into the server child.** A workspace `.ps1` (which an approved `file_write` can create, or any local file) can read the provider key. This is a direct contradiction of the shell side-effect path's env allowlist (`env_safety.rs` used by `shell.rs`/`side_effects.rs`; daemon.rs does not import it).
- Mitigations that bound reachability: script jobs are created only via operator HTTP `POST /api/daemons` (`http.rs:2127`) with the session token; the model's root `spawn_daemon` is forced to `{"check":"db"}` for `watchdog` and `{}` otherwise (`execution.rs:4507-4511`). So this is **operator-reachable, not model-reachable** — HIGH residual, not CRITICAL model escape. It is still a new gap: no lane read `daemon.rs` execution deeply, and L6's install/ops surface does not mention it.
- `kill_process_tree` shells out to `taskkill` (`:614-630`) — if `taskkill` is missing, the timeout path falls back to `child.wait()` after `kill_process_tree`, which may hang; residual.

### 1.11 `remote.rs` / `context.rs` — read, no blocking defect

- `remote.rs`: only `transport_kind == "in_process"` is accepted (`:312-337`); `heartbeat` sets a 120s lease (`:688-730`); `expire_approvals`/`expire_leases` are transactional with audit rows (`:733-830`); `reconcile_on_boot` is fatal in `main.rs:91-102`. The watchdog's "detached remote principals are in-process fixtures" is **CONFIRMED** by `remote.rs:5-6,316-337` and `execution.rs:5176-5182`.
- `context.rs`: scoped facts keyed by `parent:<uuid>:<key>` (`:15-118`), sensitive-key filtering (`:118-148`), truncation, and rewire clearing (`:161-210`). No blocking defect found; the only residual is that `token_set`/prompt rendering is string-based and relies on the caller not injecting `parent:` prefixes (keys are generated internally: `:110-117`).

---

## 2. The test bodies behind L1's test-name-only citations

All five files were read end-to-end. Results:

| Test (file:line) | L1's claim | Body verdict |
|---|---|---|
| `scale_s1.rs:101` | "41 tasks at bound 32" | **PARTIALLY SUPPORTED / correction.** Body asserts status completed, `tasks == 41`, `calls == tasks`, `max_active > 1`, `<60s` (`:136-140`). It **never asserts `max_active <= 32`**; the "bound 32" is only `with_parallel_dispatch(32)` (`:120`) and the test name. The file header still says "the engine is serial today" (`:5-8`) — stale. It is an in-process provider double, not a server test. So the test proves overlap and counts, **not** the bound. |
| `snapshot_history.rs` | kind/role, migration, corruption, history API, prune | **SUPPORTED.** kind/role `:43-70`; v2↔v1 round-trip + unknown version `:72-104`; corrupt file refused + row survives `:106-128`; history API 200/404 + roster `:130-261`; prune keeps newest 2 and index row `:263-289`. No revision-collision or corrupt-index-row test (L3 already said so). |
| `graph_authority.rs` | cycle, self-rewire, stale rev, in-flight, unknown, toposort, dupes | **SUPPORTED.** Exactly those seven tests at `:19,39,48,65,80,93,114`. |
| `kill_restore.rs` | crash restore; corrupt-at-boot refused | **SUPPORTED.** Helper aborts via `std::process::abort()` (`:22-45`); parent asserts non-success, `interrupted_runs` + `restored_runs`, status Interrupted, 2 nodes restored (`:47-89`); corrupt snapshot refused and row still recovered (`:91-117`). The helper's `#[ignore]` is required for the spawn (`:20-21`). |
| `recovery_multi_run.rs` | two interrupted runs both represented, degradation named | **SUPPORTED.** Both runs interrupted, exactly one restored + one degraded (`:94-105`), every task has a node, terminal task preserved, 3 settled tasks, no failures (`:107-160`). |

**Net correction to watchdog `L1-025`:** four of five citations are now body-verified; `scale_s1.rs:101` supports overlap/counts but **not** the bound. Also L1's §1 line counts for these files are **non-blank** counts (`snapshot_history 265`, `scale_s1 181`, `kill_restore 109`, `recovery_multi_run 149`, `graph_authority 109`), not total lines — the ledger said only `db.rs`/`store.rs` mixed counts; **five test rows do too** (see §4.4).

---

## 3. Watchdog corrections — independently re-verified

### 3.1 `finish` / `report_result` (ledger C1 / `L2-001`) — **watchdog correct; L2 R6 refuted**

- Task tool results: `finish`/`report_result` are executed by the read executor (`tools/executor.rs:60-61,78-79`; `execute_finish`/`execute_report_result` at `:89-120`) and return an `Ok` `ExecutedTool`.
- The engine then inspects `turn.executed` **after** the turn: `execution.rs:5416-5430` finds a successful `finish` and `return self.complete_task(...)`; `:5437-5462` does the same for `report_result`, including `success=false` labelling. The dispatcher really has no `finish` branch (`:2652-2709`) — that literal part of L2 is true — but the engine effect exists, so **"finish cannot finish" / "functional deception" is false**. No test covers this (L2's own missing-coverage list says so; still true).

### 3.2 Headers (ledger C2 / `L5-004`) — **watchdog correct**

`http.rs:767-779`: mutation guard accepts `Authorization: Bearer <token>` **or** `x-hivemind-token`, then compares to the per-boot session token (`:780`). GET/HEAD/OPTIONS bypass (`:764-766`). The Proposals panel's Bearer choice cannot 401. L5's table already hedged ("which one the server accepts is UNVERIFIED", line 402), so the defect is the unhedged P0/S2 wording at §9 item 6 / §10 item 4, not the observation.

### 3.3 Data-dir jail (ledger §5 item 2) — **CONFIRMED, with the missing link now proven**

Two jails are constructed:

- Engine read-only model tools: `WorkspaceJail::open(&config.workspace_root).deny_path(&config.data_dir)` (`http.rs:347-351`); `execute_tool_plan` uses it (`execution.rs:3009-3015`).
- Bridge (UI `FILE_READ`/`FILE_TREE_LIST`, and — the important part — **approved side effects**): `ToolsBridge::new(&config.workspace_root, db)` (`http.rs:259-262`) opens the jail with **no `deny_path`** (`tools_bridge.rs:53-75`); `execute_approved` uses `self.jail` for preflight/commit (`tools_bridge.rs:213-253`), and `resume_and_execute` reaches it at `execution.rs:3392-3402`.

Because the default `data_dir` is `data/` under the CWD (`config.rs:12,428-432,487-488`), the bridge jail can read and write `data/hiverouter.db`, `data/snapshots/**`, and `data/ledger/**`. `approval_ledger_dir(data_dir) = data_dir.join("ledger")` (`tools_bridge.rs:341-342`). This is an **approval-bound but data-dir-unbound** write surface: approved `file_write` can corrupt the SQLite DB or truncate the approval ledger. **New precision for the plan: the missing call is one line — `.deny_path(&config.data_dir)` on the `ToolsBridge` jail — not a new jail.**

### 3.4 Approval TTL / ledger (ledger §5 items 4-5) — **CONFIRMED**

- `APPROVAL_TTL_SECS = 120` (`approval.rs:26`); `resolve` enforces expiry and marks `expired` (`:562-577`); `cancel_pending`/`expire` only win `decision IS NULL` CAS (`:618-646`).
- `ApprovalGate::verify_grant` **discards the clock** (`approval.rs:935 let _ = now_unix_secs;`), then looks up `allow_once` by `(run_id, tool_name, args_sha256)` only (`:942-951`). No approval id, no tool_call id, no `expires_at`, no nonce. So a decision made inside TTL can be executed arbitrarily later, and any same-args call can consume another call's single-use grant. **CONFIRMED.**
- `resolve_approval_ledgered` commits the decision, then tries to append to the ledger; on failure it calls `cancel_approval` (`tools_bridge.rs:387-394`). But `cancel_pending` CAS is `decision IS NULL` (`approval.rs:640-643`) — the decision is already set, so **the "fail closed" cancel is a no-op**. The waiting executor later times out, `expire_approval` loses the CAS (decision set), `approval_decision` returns `allow_once`, and `apply_committed_decision` executes it (`execution.rs:3447-3471`). The comment at `tools_bridge.rs:387-391` is therefore **false as written**. **CONFIRMED; this is a code-comment lie, not just a gap.**
- Boot orphans are expired unconditionally (`expire_orphaned_pending`, `:694-707`); live TTL reaper `expire_pending` (`:718+`). Good.

### 3.5 Route and refusal counts — **one watchdog correction wrong**

- Routes: 60 `.route()` calls (`http.rs:609-699`); dual-method registrations at `:655` (`/api/remotes`), `:668` (`/api/daemons`), `:695-697` (`/api/provider/profile`) → **63 method+path pairs including `GET /ws`; 62 excluding `/ws`**. Ledger's counting rule is correct; L4's "62" needs the rule stated.
- Refusals: the macro table has **78 code rows** (`refusal.rs:73-…`; `codes_are_unique_and_well_formed` at `:346`). The 79th `E_*` literal is the **negative test fixture** `"E_NOT_A_REAL_CODE"` (`:366`). The ledger's correction "79 distinct E_ strings" counts a test literal; **L4's original 78 is correct**, and the watchdog correction `L4-009` must be reverted. (Raw `Select-String` returns 79 because it scans the test module too.)

### 3.6 File/CLI sizes and line counts — **L6 systematically off by +1; CLI size confirmed**

- `tools/hivemind-cli.mjs` = **80,007 bytes** (filesystem length); L6's 80,075 is off by 68 (ledger right).
- L6's per-test-file line counts are **total+1**: `snapshot_history` 291 (actual 290), `scale_s1` 190 (189), `kill_restore` 118 (117), `recovery_multi_run` 162 (161), `graph_authority` 121 (120). The same +1 pattern presumably applies to the whole table; state the counting method or fix.
- L1's test-file rows are **non-blank** counts (§2), not total.

### 3.7 Strict-gate dirty tree — **L6's filename stale; ledger right, but the real cause is worse**

Current `git status --porcelain` = `?? docs/analysis/` + `?? docs/plans/HIVEMIND-FINISH-PLAN-v3.md`. `run-gates.ps1:181,224-227` refuses **any** dirty path in strict mode. So the strict gate cannot be green while the swarm's own reports and the finish plan exist. L6's `docs-analysis-tmp-index.txt` is gone; the file to delete is not the cause, and deleting it would not help. The plan needs an explicit decision: commit the reports/plan, add them to a documented gate exclusion, or run `-Rehearsal` (which prints REHEARSAL, not PASS).

### 3.8 Census vs watchdog on `build.rs` — **contradiction inside the census**

The watchdog §4 "only tracked code file named by no lane" is true (L4 has 0 `build.rs` hits), but census §A assigns `build.rs` to L4 (`CODEBASE-CENSUS.md:782`) and its "unowned paths = 25" list (`:522,772-785`) contains only UI assets/lockfiles/gitignore. The census's "0 product/UI/API/tool features without an owner" is scoped to features, not build scripts, so it is not literally false — but the ownership table overstates coverage. Keep `build.rs` as unowned in the plan.

---

## 4. Lane high-impact claims audited on top of the watchdog

### 4.1 L1's provider-call arithmetic (`L1-008`) — **needs correction**

L1 §2.3 says the 64-call cap is "arithmetically inconsistent with a 40-node plan plus planner + repair + 2 root-control turns + root final + up to 12 tool-loop continuations per task + parent escalation answers" — that is defensible. The watchdog's compressed form, **"64 provider calls cannot cover a legal 40-node plan with task sends and continuations"**, is **not** supported for a no-tool plan: the minimum is planner 1 (+1 repair) + root control ≤2 + 40 task turns + root final 1 = **44-45**, under 64. It becomes false only because each follow-up tool turn claims another provider call (`execution.rs:5469` loop; `MAX_TOOL_TURNS = 12`), so **two tool-using tasks with ~10 turns total exhaust the cap**. Correct statement for the plan: the 64-cap is inconsistent with the *per-task 12-turn allowance*, not with the 40-node topology itself; and the 40 tool-call cap (`tool_loop.rs:16`) is a separate budget.

### 4.2 L1's parallel "bound 32" (`L1-286`, `L1-524`) — **name-level only**

Body read (§2): no `max_active <= 32` assertion. Also `http.rs:365-379` clamps the env override to 1..64 and external endpoints to 2; `with_parallel_dispatch(32)` is test configuration (`scale_s1.rs:120`). So "measures 41 tasks at bound 32" overstates what the test proves.

### 4.3 L1's approval race — **CONFIRMED by code, no test**

`run_state.rs:510-512` allows `Suspend` only from `Running`; `persist_approval_transitions` applies it directly; parallel default 32 (`http.rs:365-379`). A second concurrent suspend loses the CAS and fails the run. L1 labels reachability "VERIFIED by parallel default, test UNVERIFIED" — accurate. No `suspension.rs` read here (L1 disclaimed it); residual.

### 4.4 L1 §1 / L6 §2 line counts — **both mixed**

Already stated: L1 test rows are non-blank counts; L6 all rows +1. This matters because the ledger's `L1-022` correction ("only db/store used non-blank") is incomplete: five test-file rows did too.

### 4.5 L2's dead-grant count (7/35) — **CONFIRMED**

`role_class("tool")` returns `None` (`execution.rs:201-214`); planned roles are validated with `role_class(...).ok_or(PersonaUnknown)` before a row is created (`execution.rs:1996-2003`); `validate_spawn_role` never permits `tool` (`:244-275`); `task_agent_tier` supports `"tool"` (`:293`) but nothing can create it. ToolAgent's 4 grants + HyperAgent's 3 (root never enters the tool loop; `orchestrate_root` is a separate path) = **7/35 dead**. Registry counts confirmed: 5 tiers, 12 tools, 35 grants (`tool.rs:460-516`; 3+12+11+5+4).

### 4.6 L2's root vocabulary — **CONFIRMED with a prompt/dispatch drift note**

Engine prompt advertises six tools (`execution.rs:4300-4307`) and omits `spawn_daemon`; dispatch handles it (`:4464-4525`); `provider_executor.rs:39-45` `ROOT_CONTROL_SYSTEM` lists it. So the vocabulary is seven names across three inconsistent lists, none registry-backed. L1-002's correction already notes this; the plan should treat `spawn_daemon` as advertised-by-provider-only.

### 4.7 L2's `resolve` unreachable-`E_TOOL_NOT_FOUND` note — **CONFIRMED**

Every enum variant gets a contract (`tool.rs:456-458`), so `contracts.get` cannot miss in v1; only `resolve_call`'s `from_id` can return `ToolNotFound` (`tool.rs:579-583`). No action needed; fits the census self-reference finding.

### 4.8 L5's `/health`+`/diagnose` launch-token gap — **CONFIRMED as code, impact not proven**

`main.tsx:30` shim predicate is `/api/` only; `http.rs:764-766` lets GET through to the router. `host_guard`/launch-token middleware is separate (not fully read); in a launch-token deployment `/health`/`/diagnose` would be unauthenticated but bound to loopback. The ledger already marks server impact UNVERIFIED; keep it out of P0.

---

## 5. Gates, install identity, and unrun unit tests

### 5.1 No gate runs `node --test` — **CONFIRMED**

`tools/qa/run-gates.ps1` runs cargo, npm typecheck/test/build, guarded e2e, key scan, manifest/hashes (`:437-441, 505-526, 536-580`); grep for `node --test` returns **zero** hits in `run-gates.ps1`, `tools/test.ps1`, `e2e/package.json`, `web/package.json`. The two safety-critical script test files exist and are unused:

- `tools/qa/tests/seal-topology.test.mjs` — 5 `node:test` cases (`:1-6,54-106`);
- `tools/qa/tests/trace-tools.test.mjs` — 3 cases (`:69,83,118`).

`tools/test.ps1` invokes `cargo test` for provider/tools/contract/core only (`:43-58`). So the seal predicate that `run-gates.ps1` uses for `-Release` (`:326-352`) is itself untested by the gate. **HIGH, one-line fix** (add `node --test tools/qa/tests/` as G5b; L6 already proposed it).

### 5.2 `e2e/run-guarded.mjs` — the guard is fail-open when the report is absent

`run-guarded.mjs:9-14` spawns Playwright and writes `test-results/results.json` (cwd-relative); `:16-24` parses it if it exists, else `report = null`; `:25` guards only `if (stats …)`. Therefore:

- If Playwright exits 0 but writes no parsable report, the all-skipped guard **does not run**, and the wrapper exits 0. The file's own premise ("Playwright reporter/teardown errors do not reliably change exit code") is exactly the case where the report is missing — the guard fails open in its threat model.
- Stale `results.json` from a previous run is read if the new run writes nothing; `passed>0` from stale stats would mask an all-skip run.
- `flaky` is counted as failure (`:27`) — arguably right; `expected` is Playwright's passed count.
- The script depends on cwd (`test-results/…`, `node_modules/…`), and `run-gates.ps1` calls it with cwd `e2e` (`:143,526`), so the current call site is fine; direct invocation from the repo root is not.

**Fix:** require a fresh report (delete before run / mtime check), and fail when `report === null` unless an explicit `E2E_ALLOW_MISSING_REPORT=1` is set. No test covers this wrapper.

### 5.3 Gate and install identity — **CONFIRMED mismatches**

- `tools/release-manifest.json` is `schema: hivemind.release-manifest.v1`, `version: 1.1.0`, `status: SEALED_1.1.0`; `web/package.json` and all four crate manifests are **1.2.0-dev**. `run-gates.ps1:183,211-213` computes `$version` from `web/package.json` and treats `-` in the version as a pre-release candidate, so `-Release` on this tree takes the **exit 3 PRE-RELEASE** path (`:313-359, 453, 744`), never green.
- The sealed branch requires `schema == hivemind.release-manifest.v2` (`:246-248`), but the only manifest on disk is v1. So this tree cannot certify as sealed even after a rebuild unless the manifest is migrated to v2. This is a **gate/identity defect**, not just staleness.
- `dist/HiveMind/` is the **dev** package: `dist/HiveMind/hiverouter.exe` = 36,540,416 bytes = `target/release/hiverouter.exe`, while the sealed 1.1.0 binary is 13,624,320 (`release-manifest.json` artifacts). `dist/HiveMind/README.txt:1-3` says "portable desktop build" with no version/commit (L7 finding, confirmed).
- L7's citation for the key instruction is wrong: the instruction is at `dist/HiveMind/README.txt:14-15`, not `:18-19` (`:18-20` is the windowless-launcher paragraph). L7-004's correction is right.
- `README.txt` claims the key is injected "into the server child only" (`:11-13`); daemon children inherit it (§1.10) — a **documentation-vs-code contradiction**.
- `run-gates.ps1` G7 verifies `release-manifest.json` hashes against disk (`:574+`); because `dist/HiveMind` and `target/release` are dev builds, G7 will fail in strict mode (L6 #2, confirmed by sizes). This is expected for a dev tree but must be stated in the finish plan, not papered over.

### 5.4 `tools/qa/check-seal-topology.mjs` predicate

Not re-read in full (L6 read it; it is 435 lines). Its tests cover: unsealed, sealed allowlist subset, source-file in diff, HEAD≠tag, artifact-hash drift (`seal-topology.test.mjs:54-106`). Missing from the test set: empty/duplicate allowlist entries, case/path separator normalization (`..\\` vs `../`), symlink tags, and a `seal_diff.files` entry that is allowlisted but does not exist in the actual diff. Residual until read.

---

## 6. What this audit did not establish (residual unknowns)

1. **No execution.** Every test body was read, not run. `scale_s1`'s bound, the approval race, the daemon env leak, and the e2e guard's fail-open are code-derived; running them is the proof.
2. **`ws.rs` backpressure mapping** (`Lagged` → `E_RESYNC_REQUIRED`) and `host_guard`/launch-token path were not read; bus/static-UI findings are scoped accordingly.
3. **Snapshot index ordering** (`indexed_revisions`) was not read; `history.latest_revision` assumes ascending order. Prove it or sort explicitly.
4. **`census.rs`** was not re-read (L2 read it fully); the "PROMPT_CLAIMED_TOOL_IDS is a literal" claim is taken from L2 and its cited lines, not independently reopened.
5. **`side_effects.rs` / `shell.rs` env allowlist** was not fully read; §1.10's contrast with daemon.rs rests on the imports and L2's line citations.
6. **`trace-tools.test.mjs`** only its test names were read, not bodies.
7. **Release-manifest v2 template** does not exist on disk; the migration path is unspecified.
8. **`docs/analysis` untracked status** could be resolved by committing the swarm artifacts; the plan must choose commit vs exclude.

---

## 7. Claim ledger (verified / unsupported / new)

Status vocabulary: `CONFIRMED`, `CORRECTED`, `UNSUPPORTED`, `NEW`, `UNVERIFIED`.

| ID | Claim | Evidence | Status |
|---|---|---|---|
| G-01 | build.rs runs `rustc --version`, enforces minor ≥89, embeds version | `build.rs:14-40`; `checks.rs:58-68` | CONFIRMED |
| G-02 | build.rs rejects nightly/beta and ignores patch level | `build.rs:29-31,43-48` | NEW (code-derived) |
| G-03 | build.rs is lane-unowned; census assigns it to L4 but L4 never names it | `CODEBASE-CENSUS.md:782`; L4 grep 0 | CONFIRMED (watchdog) |
| G-04 | handshake.rs has no production caller; doc claim false | `lib.rs:19,49`; grep 0 prod hits | NEW |
| G-05 | handshake idempotency resets on restart; eviction assumes v7 | `handshake.rs:140-191` | NEW |
| G-06 | `wake_cancelled` has no production caller | `approval_wait.rs:63-65`; grep | NEW |
| G-07 | bus publish/replay sequencing correct; poison asymmetry | `bus.rs:105-133,171-231` | NEW (low) |
| G-08 | history maps corrupt `created_at` to 1970 | `history.rs:79` | NEW (medium) |
| G-09 | history reads one file per revision per call | `history.rs:75-88` | NEW (low) |
| G-10 | `/api` exact path serves SPA 200 | `static_ui.rs:46-70` | NEW (low) |
| G-11 | plan validation omits fanout/agent caps | `planner.rs:50-129`; `execution.rs:2025-2029` | NEW (medium) |
| G-12 | three copies of depth 3 | `caps.rs:9`; `planner.rs:67`; `handshake.rs:19` | NEW (low) |
| G-13 | daemon `.ps1` inherits server env (incl. provider key), bypass policy | `daemon.rs:428-445`; `README.txt:11-13` | NEW (HIGH residual) |
| G-14 | supervisor `next_due_at` ignores interval; state errors ignored | `remote_supervisor.rs:61,80,84,116-141` | NEW (low) |
| G-15 | `scale_s1` does not assert concurrency bound; header stale | `scale_s1.rs:1-8,101-145` | CORRECTED (L1) |
| G-16 | snapshot/graph/kill/recovery test bodies support citations | files cited §2 | CONFIRMED (L1-025 upgraded) |
| G-17 | finish/report_result complete the task; L2 R6 false | `execution.rs:5416-5462`; `tools/executor.rs:78-79` | CONFIRMED (watchdog C1) |
| G-18 | server accepts both auth headers | `http.rs:767-779` | CONFIRMED (watchdog C2) |
| G-19 | bridge jail omits data-dir deny; approved effects use it | `http.rs:259-262`; `tools_bridge.rs:53-75,213-253`; `http.rs:347-351` | CONFIRMED (watchdog §5.2, with new link) |
| G-20 | grant verification ignores TTL/identity | `approval.rs:924-951` | CONFIRMED |
| G-21 | ledger-failure "cancel" is a no-op; timeout executes committed label | `tools_bridge.rs:387-394`; `approval.rs:640-643`; `execution.rs:3447-3471` | CONFIRMED |
| G-22 | 60 route calls → 63 pairs incl. `/ws`, 62 excl. | `http.rs:609-699` | CONFIRMED (watchdog) |
| G-23 | 78 refusal table codes; 79th is a test literal | `refusal.rs:73-…,346,366` | CORRECTED (watchdog L4-009) |
| G-24 | `hivemind-cli.mjs` 80,007 B | filesystem | CONFIRMED (watchdog L6-007) |
| G-25 | L6 per-file line counts are total+1 | 5 files measured | CORRECTED (L6) |
| G-26 | L1 test rows are non-blank counts | 5 files measured | CORRECTED (L1-022 scope) |
| G-27 | no gate runs `node --test` | grep across gates/package files | CONFIRMED (L6 D2) |
| G-28 | guarded e2e fails open if report missing/unparsable | `run-guarded.mjs:16-35` | NEW (HIGH for test trust) |
| G-29 | manifest v1/1.1.0 vs tree 1.2.0-dev → pre-release exit 3; sealed branch needs v2 | `release-manifest.json:2-4`; `run-gates.ps1:183,211-213,246-248,313-359` | CONFIRMED |
| G-30 | key instruction at `README.txt:14-15`, not `18-19` | `dist/HiveMind/README.txt:11-20` | CONFIRMED (watchdog L7-004) |
| G-31 | 7/35 dead grants (ToolAgent 4 + HyperAgent 3) | `tool.rs:460-516`; `execution.rs:1996-2003,244-275` | CONFIRMED (L2) |
| G-32 | root control vocabulary is 7 names across 3 lists, registry-free | `execution.rs:4300-4307,4381-4811`; `provider_executor.rs:39-45` | CONFIRMED (L1/L2) |

---

## 8. Priority envelope for the finish plan

**P0 (data/secret integrity):**
1. `data/` read/write through the bridge jail (G-19) — one `deny_path` call; add a test that `file_read data/hiverouter.db` and `file_write data/…` refuse.
2. Approval grant identity/TTL (G-20) and the false "fail closed" cancel (G-21) — bind `(approval_id, tool_call_id, args_sha256, expires_at)`; make ledger failure abort before the decision is irrevocable or mark the row `cancelled` with a post-decision CAS.
3. Daemon script env (G-13) — `.env_clear()` + explicit allowlist, or refuse script jobs that need env; document operator-only reachability.

**P1 (build/release/test trust):**
4. `node --test tools/qa/tests/` in the gate (G-27).
5. Guarded e2e must fail on a missing report (G-28).
6. Manifest/gate identity: migrate to v2 or explicitly stage the 1.2.0 seal; decide the untracked `docs/analysis` gate conflict (§3.7).
7. `scale_s1` bound assertion and stale header (G-15); L6 line-count convention (G-25).

**P2 (robustness/observability):**
8. history epoch fallback + O(n) reads (G-08/G-09); plan-time cap validation (G-11); supervisor `next_due_at`/state errors (G-14); `/api` SPA fallback (G-10); build.rs prerelease parsing + patch policy (G-02); handshake wire-or-delete (G-04/G-05).

---

*End of GAP-AUDIT-1. No repository file other than this report and GAP-PATCH-1.md was written.*
