# Round 95 / 02 — Plan crosswalk: HIVEMIND-FINISH-PLAN-v3 vs shipped code

> **Post-audit correction (final tree):** the `http.rs` `/api/evidence/ledger` candidate
> observed during this audit was **discarded**. The current product tree has no such
> modification; the installed build remains `806251b`. Any “uncommitted” or “working
> tree dirty” language below describes the audit-time tree, not final publication.

**Researcher:** plan-crosswalk (task-25) · **Observed:** 2026-11-10T23:21–23:40-06:00
**HEAD:** `1129349` (during the audit a `http.rs` `/api/evidence/ledger` candidate was present; it was **discarded before final publication**, so the final product tree has no such change)
**Installed build:** `806251b` (`dist/HiveMind/BUILD.json`, `dirty:false`) — two commits behind HEAD.
**Verdict up front:** the plan's *security, approval, schema and root-instruction* core is materially
implemented and tested; **WP-SEC-3, GOV-2, GOV-3, HARDEN-1, STATE-1, API-1, UI-2, TEST-1(strict),
DOCS-1, MISSION-1 are not complete**. Treat the uncommitted http.rs change and HEAD-only receipts as
**unshipped**.

Evidence classes used below: **[T]** executed by this session, **[R]** read from file at cited line,
**[C]** commit message/diff, **[U]** unverified (stated as such).

---

## 1. Base evidence read

| Item | Result | Class |
|---|---|---|
| `git rev-parse HEAD` | `1129349`; final product tree clean (the `http.rs` ledger candidate was discarded after the audit) | [T] |
| `git log 73e5fad..HEAD` | 16 commits: approval serialization tests (93e40fb, be03094, 195fa1c), monitor review (806251b), EVO-GATE (545d676, f7c8463, 92daed7), install receipts (0af9ed6, 8d8fa9d), battery count doc (002c16d), plan baseline update (308c008), crash/scale receipts (1129349) | [T] |
| `docs/evidence/live/full-battery-20261110-225940.json` | 8/8 PASS, 0 failed/skipped; `source_commit c49f90c`; `working_tree_dirty:true`; backend 406s; offline, no provider call | [R] |
| `docs/evidence/live/install-1.2.0-dev.json` | built from `806251b`, health `1.2.0-dev` schema 24, shortcuts, 1440/375 screenshots; **explicitly says monitor is review-only** (`:38`) | [R] |
| `tools/qa/check-docs-truth.mjs` | run now: `checks: 19/19 pass; violations: 0` / `DOCS TRUTH: PASS` | [T] |
| Rust test declarations | **663** `#[test]`/`#[tokio::test]` sites (plan said 639) | [T] grep |
| Web test sites | **391** `it(`/`test(` sites (plan said 231) | [T] grep |
| Playwright | 16 spec files in `e2e/specs/` (plan said 16; 81 declarations not re-counted) | [T] |
| Crash / scale receipts | committed at HEAD `1129349`: crash drill `ok:true`, boot id changed, data preserved; scale drill exit 0, 41-task serial+parallel, **"not a live 40-node provider run"** | [R] |

Not read within budget: full `execution.rs`, `store.rs`, `snapshot.rs` internals, launcher C#,
`run-gates.ps1` end-to-end, the 41-page truth reports in full (headings + targeted sections only).

---

## 2. Work-package matrix

Legend: **DONE** = required behavior exists and has a committed test/receipt that proves it.
**PARTIAL** = part shipped, at least one required bullet absent. **NOT DONE** = no implementation.
**N/A** = explicitly optional/deferred by the plan.

| WP | Verdict | What is proven (file:line, commit, test, receipt) | What is missing / not proven |
|---|---|---|---|
| **SEC-1 containment** | **PARTIAL** | `cmd.exe` absent from `conservative_default` (now `git.exe`,`cargo.exe`) [R `shell.rs:179-185`]; interpreter hosts refused regardless of policy [R `shell.rs:54-62,89-99`]; `data_dir`+ledger denied in `ToolsBridge::new_with_data_dir` [R `tools_bridge.rs:94-107,144`]; engine jail denies data dir [R `http.rs:347-351`]; daemon stdio env sanitized [R `daemon.rs:444`] with a leak test asserting `CLEAN` [R `daemon.rs:827-845`] | Plan L100-101 required **db_path/+wal/+shm** deny too. `new_with_data_dir(workspace, data_dir, db)` [R `tools_bridge.rs:104`] never receives `db_path`; deny set is only `data_dir`+ledger [R `:94-95,106-107`]. A custom `--db` outside the data root is not denied. `file_read/file_write` refusal tests not verified. |
| **SEC-2 approval integrity** | **PARTIAL (near done)** | `allow_once` TTL is mandatory and checked [R `approval.rs:10-14,108,117`]; verification binds `approval_id` [R `:14`]; per-run single suspension + both queued approvals execute one write each [C 93e40fb, be03094, 195fa1c]; waiter lifecycle arm/wake/disarm [R `approval_wait.rs:1-65`] | ① commit `be03094` itself records an **execution-accounting gap** (not fixed). ② Plan required a *bounded waiter queue*; implementation is "one live waiter per run" (refuses a second with `E_APPROVAL_REPLAY`, [R `approval_wait.rs:40-51`]) — satisfies serialization but means the second suspended task is not queued; verify at engine level [U]. ③ 14 `verify_grant` call sites and two-identical-grant shadow test [U]. |
| **SEC-3 root tool truth** | **PARTIAL** | Root schemas now advertise `spawn_superagent` and `assign_task` with parameters [R `provider_executor.rs:592-608`]; root is two bounded turns with feedback [R plan-adjacent `HYPERAGENT-TRUTH-REPORT.md:271` + `provider_executor.rs:709-738`] | Census 35→28 grant recount and its tests [U]; removal of unreachable ToolAgent/HyperAgent grants [U]; explicit root-control schema-validation/refusal tests [U]. |
| **SCHEMA-1 v24** | **DONE** | v24 = `task_instruction`, `ALTER TABLE tasks ADD COLUMN instruction TEXT` + down [R `db.rs:859-865`]; migration idempotence/schema 24 tests [R `db.rs:1481-1537`]; installed DB reports schema 24 [R install receipt:23] | Conditional approval unique index was to remain post-migration — untouched, not re-read [U]; pre-v19 duplicate precheck not verified [U]. |
| **GOV-1 instruction + root objective** | **DONE** | Child prompt includes the task instruction [R `execution.rs:4190-4196`]; `spawn_superagent {objective,...}` and `assign_task {task_id, objective}` schemas [R `provider_executor.rs:592-608`]; v24 column above; ROOT_CONTROL system text advertises both [R `provider_executor.rs:40-46`] | API detail/roster bounded instruction, auto-voter instruction, and the explicit provider-conformance test [U]. Strikethrough only the core lines (below). |
| **GOV-2 monitor/review** | **PARTIAL** | One answer-only post-dispatch review turn runs before final synthesis [C `806251b`; R `provider_executor.rs:48-53,709-738`]; receipt says "one bounded read-only post-dispatch monitor review" [R install receipt:32]; run_lifecycle tests added (+144 lines) [C] | **Required `send_instruction {task_id,text}` and `cancel_task {task_id,reason}` tools do not exist**; monitor prompt explicitly says "do not call tools" [R `:48-53`]; no bounded repair branch; install receipt self-declares "review-only, cannot send_instruction, cancel_task or repair" [R :38]. This is a review, not the WP's monitor. |
| **GOV-3 caps/settlement** | **PARTIAL / UNVERIFIED** | 41-task serial and bounded-parallel scale probes pass, exit 0 [R `scale-drill-20261110-230653.json`]; the receipt honestly says not a 40-node provider run; crash drill settles runs [R crash receipt] | Atomic tool-call **reservation**: grep for `reserve|reservation|fetch_add|increment` over `execution.rs` found nothing in call budget context [T]; plan's "parallel tool budget ≥2 tasks ≤40" and provider-call formula test [U]; single-node plans execute root [U]; structural `Completed` without execution [U]. |
| **HARDEN-1 boot/edge truth** | **PARTIAL** | Exact `/api` now 404s rather than falling to SPA; non-GET/HEAD JSON policy documented [R `static_ui.rs:41-61`] | ① `history.rs:79` **still** maps an unparsable timestamp to `Timestamp::from_unix_secs(0)` (1970) [R] — required refusal absent. ② `handshake.rs` exists (8,430 B) and is still re-exported [R `lib.rs:20,51`; `tests/handshake.rs`] — plan required deletion. ③ `Plan::validate` checks node count/depth/deps/cycles only [R `planner.rs:50-80`]; **no fan-out ≤8** before the planning call. ④ `build.rs` is still a build script (2,097 B), not extracted to a testable `src` module [R]; patch-level version defect unverified [U]. ⑤ supervisor due-time honor [U]. |
| **STATE-1 migration/state** | **PARTIAL / UNVERIFIED** | Recovery is boot-time owner, marks all active runs interrupted, settles tasks and restores newest snapshot [R `recovery.rs:4,80,116`]; snapshot revision uses shared max [R `snapshot.rs:490`] | Memory-clear in same transaction as delete, role-clear via `source_task_id→tasks.role`, strict writer for live memory, snapshot collision/regression test [U]. No commit in `73e5fad..HEAD` names STATE-1. |
| **API-1 CLI/API truth** | **NOT DONE** | WS-only command surface exists and QA/Playwright `ws-contract.spec.ts` exists [R e2e listing] | `docs/contracts/HTTP-API-v1.md` **does not exist** [T]; CLI has no `HIVEROUTER_LAUNCH_TOKEN` / `launch.token` source — grep over `tools/hivemind-cli.mjs` found only `LOCALAPPDATA` at `:46` [T]; CLI token-never-logged test and route-contract test [U]. |
| **UI-1 repair** | **DONE (mostly)** | Components exist: `SnapshotBanner.tsx`, `ConfirmDialog.tsx`, `ToastRegion.tsx`, `ToolActivityPanel.tsx`; Proposals writes use the single proven `x-hivemind-token` header [R `ProposalsPanel.tsx:124-131`]; spawn gate imported/wired in Agents surface [R `AgentsPanel.tsx:12`]; 375/1440 screenshots exist [R install receipt:16-17, :33]; web unit tests + typecheck + build PASS in battery [R battery:18-33] | Synthetic FigCraft fallback/NaN removal, mock-timer close, `__APP_VERSION__` read, and the 375px **assertion in the harness** (vs screenshots) [U]. Test count grew 231→391 but per-item mapping not proven. |
| **UI-2 polish** | **NOT DONE** | Alpha token/base exists | `HivePulse.tsx`, `ToolsMatrixPanel.tsx`, `FirstRunNotice.tsx` **absent** [T]; governance timeline in Proposals and indeterminate splash not implemented [U]. |
| **INST-1 installer** | **DONE (mostly)** | `install-desktop.ps1`, `uninstall-desktop.ps1`, `set-provider-key.ps1` exist [T]; `BUILD.json` has commit/version/built_at/server+launcher sha256/dirty=false [R]; install receipt: health `1.2.0-dev`, schema 24, desktop+start-menu shortcuts, 1440/375 screenshots [R] | Acceptance said **clean profile** (`-InstallRoot`+`-DataRoot`+no pre-existing provider.key). Receipt uses default `%LOCALAPPDATA%` roots [R :11-13]; a true clean-profile run is not evidenced. Health is `degraded`/`missing_key` (expected offline, but not a fully clean-green install). |
| **TEST-1 gate/battery** | **PARTIAL** | `node --test tools/qa/tests/*.test.mjs` is wired dynamically into `run-gates.ps1` [R `:538-540`]; `run-full-battery.ps1` exists with 8 named steps and SKIPPED≠PASS semantics [R `:1-8,48-77`]; battery PASS at `c49f90c` [R receipt] | ① e2e guard is **still fail-open** when `results.json` is missing/unparsable/stale: `if (stats && ...)` skips the check when `report` is null [R `e2e/run-guarded.mjs:16-35`] — plan L356-358 explicitly required failure. ② Battery was not re-run at HEAD `1129349`; its `source_commit` is `c49f90c` and tree was dirty [R]. ③ Strict gate remains expected RED at G7 / `-Release` exit 3; no green-seal claim is made (correctly). |
| **DOCS-1 truth/release** | **PARTIAL** | Docs checker fix landed: 19/19 PASS now [T]; battery step PASS [R battery:36-40] | `docs/evidence/live/INDEX.md` **missing**, `docs/DOWNLOADS.md` **missing** [T]; claims-register/provider-truth/CODEMAP banners, README web_fetch receipt scope, stale web_dist hashes, `e2e/README.md` corrections [U]. |
| **MISSION-1 gallery** | **NOT DONE** | LibraryPanel exists and installed UI "shows all eight templates" [R install receipt:33] | No `MissionGallery.tsx` [T]; no gallery launch e2e, no card/empty/error states, no `GET /api/templates`-backed mission cards [U]. |
| **OPT-1 verifier** | **N/A (not started)** | — | Plan defers it until after SEC-1/2; no local verifier tool exists [U]. |

### 40-round table status (plan §3)

R2 done (checker + QA tests) · R3–R5 mostly (SEC-1/2 shipped; db-path sub-item open) · R6 done · R7–R9 done ·
R10–R11 **partial** (read-only review only) · R12–R13 **partial/unverified** (scale receipts, no atomic reservation proof) ·
R14–R16 **unverified** · R17–R19 **partial** · R20 **not done** · R21–R23 mostly · R24–R27 mostly (no clean-profile proof) ·
R28–R30 **not done** · R31–R34 **partial** · R35 **not run** (battery is explicitly offline; no live provider receipt in this set) ·
R36–R38 **partial** · R39–R40 **not done** (no independent verifier pass / final installed screenshot at HEAD).

---

## 3. Lines/headings safe to mark done with strikethrough

Do **not** edit the original plan file; the list below is for a later plan owner to apply `~~...~~`.
Only lines whose *entire* claim is now proven are listed.

| Plan line(s) | Text (anchor) | Why safe |
|---|---|---|
| **156** | `### WP-SCHEMA-1 — freeze and ship v24 once` | v24 up/down shipped `db.rs:859-865`, migration tests, installed schema 24. |
| **161** | "freeze **v24 = `tasks.instruction TEXT` only** (implemented in this pass)." | Same evidence; memory columns are still deferred (no later edits observed). |
| **173** | `### WP-GOV-1 — task instruction and root objective` | Core requirement shipped: column, child prompt `execution.rs:4190-4196`, advertised schemas `provider_executor.rs:592-608`. |
| **179–183** | "use the frozen v24 … insert/select/update; update `provider_executor.rs` root schemas … `spawn_superagent {objective, persona?}` … `assign_task {task_id, objective}` … update `ROOT_CONTROL_SYSTEM`" | Same evidence. **Do not strike 184** (provider-conformance test unverified) or the auto-voter clause. |
| **393–394 (first clause only)** | "Acceptance: after the checker fix, `node tools/qa/check-docs-truth.mjs` exits 0 with 19/19" | Re-ran this session: `19/19 pass; violations: 0`, exit PASS. Strike the checker clause only; **keep** "`verify-harness.mjs`; named link/path scan; no stale claims remain" (unverified). |
| **425** | `| R2 | WP-TEST-1: wire all … | 8/8; checker measured; no green seal claim |` | QA tests wired `run-gates.ps1:538-540`; checker 19/19 measured; no green seal claimed. |

Safe to annotate (not strikethrough): **L52–53** ("HyperAgent is currently a one-shot controller plus
answer-only final") is now stale — `806251b` adds one bounded monitor review before final, making root
two turns. It is still *not* a governor, so do not strike; replace "one-shot" with "two bounded turns
(review-only)". **L190–203 (WP-GOV-2)** and **L348–365 (WP-TEST-1)** must not be struck.

---

## 4. Known bugs still present vs fixed P0s

### Still present (with code evidence)

1. **P1 — history maps corrupt timestamps to 1970.** `history.rs:79`:
   `Timestamp::parse(&created_at).unwrap_or_else(|_| Timestamp::from_unix_secs(0))`.
   Plan HARDEN-1 required a named refusal. UI/history consumers see a fake 1970 revision.
2. **P1 — e2e skip-guard fails open.** `e2e/run-guarded.mjs:16-35`: when `results.json` is missing or
   unparsable, `report` is `null` and the `if (stats && …)` block is skipped; the process exits with
   Playwright's exit code, which the comment at `:1-3` says is unreliable. Plan TEST-1 required fail.
3. **P1 — custom `--db` outside the data root is not jail-denied.** `ToolsBridge::new_with_data_dir`
   receives only `(workspace_root, data_dir, db)` `tools_bridge.rs:104`; the deny set is `data_dir` and
   `approval_ledger_dir(data_dir)` `:94-107`. `db_path/+wal/+shm` are never passed (plan SEC-1 L100-101).
4. **P2 — planner validates too late and too little.** `Plan::validate` `planner.rs:50-80` enforces
   non-empty/≤40/depth/deps/cycles; **no fan-out ≤8**, so the refusal occurs after the planning call
   (plan HARDEN-1 L228-230).
5. **P2 — monitor cannot act.** `ROOT_MONITOR_SYSTEM` `provider_executor.rs:48-53` says "do not call
   tools … no authority to change the run"; no `send_instruction`/`cancel_task` exists; no repair branch.
   Install receipt `:38` confirms. This is unimplemented WP-GOV-2, not a regression.
6. **P2 — CLI has no documented launch-token source.** No `HIVEROUTER_LAUNCH_TOKEN` or
   `launch.token` read in `tools/hivemind-cli.mjs` (grep); only `LOCALAPPDATA` at `:46`. API-1 open.
7. **P3 — dead public API retained.** `handshake.rs` (8,430 B) + `lib.rs:20,51` exports +
   `tests/handshake.rs` — plan HARDEN-1 required deletion, not wiring.
8. **P3 — `build.rs` is still untestable-by-cargo.** `backend/crates/hiverouter-core/build.rs`
   (2,097 B) not extracted to a `src` module; plan HARDEN-1 L233-236 open. Actual version-parse
   behavior unverified.
9. **Discarded candidate — `/api/evidence/ledger`.** `http.rs` +125 lines adding an 8 MiB-capped ledger
   summary endpoint: no UI consumer, no route-contract test; discarded before final publication. Not a current-tree claim.
10. **Documented gap, not code bug — approval execution accounting.** Commit `be03094` message says it
    "document[s] execution-accounting gap"; tests pin single suspension and one-write-per-approval but
    do not close the accounting gap. Do not present approval integrity as fully closed.

### Fixed P0/P1s (do not re-open)

- `cmd.exe` removed from the default allowlist and every interpreter host refused (`shell.rs:54-99,175-185`).
- Approval `allow_once` requires unexpired TTL and binds `approval_id` (`approval.rs:10-14,108,117`).
- Per-run single suspension; both queued approvals execute one write each (commits 93e40fb…195fa1c).
- Data dir + approval ledger denied by the tool jail (`tools_bridge.rs:94-107`); engine jail matches (`http.rs:347-351`).
- Daemon stdio env sanitized; `DEEPSEEK_API_KEY` leak test asserts `CLEAN` (`daemon.rs:444,827-845`).
- Root schemas advertise real objective args (`provider_executor.rs:592-608`); v24 `instruction` shipped.
- Exact `/api` no longer falls through to SPA (`static_ui.rs:41-61`).
- Docs-truth checker 19/19 and QA tests wired into the strict gate (`run-gates.ps1:538-540`).

---

## 5. Tests that exist vs tests needed

| Residual | Exists now | Needed |
|---|---|---|
| history 1970 | none found for corrupt timestamps | `history_corrupt_timestamp_refuses`: insert malformed `tasks.created_at`, assert `Err(Refusal::HistoryGap/SchemaInvalid)` and no 1970; API-level JSON test. |
| e2e guard fail-open | none for missing results | Node test driving `run-guarded.mjs` with absent results.json, malformed JSON, stale mtime → exit non-zero; keep all-skipped case. |
| custom `--db` jail | `file_read`/jail tests exist for data dir [U exact] | construct bridge with `db_path` outside data dir; assert `file_read`, `file_write`, `send_to_sibling` all refuse; plus canonicalization-failure fail-closed test. |
| planner fan-out | `plan.validate` node/depth/cycle tests `planner.rs:251-313` | `plan_fanout_over_8_refused_before_provider`: scripted provider asserts zero planning calls on refusal; exact refusal code. |
| monitor tools | run_lifecycle review tests (+144 lines, 806251b) | scripted provider: assign → monitor `send_instruction` → child observes text → `cancel_task` refusal after terminal → final tool-free; late-mutation refusal. |
| CLI token/API contract | `ws-contract.spec.ts` | CLI mutation test against real local server with launch token; token never appears in stdout/stderr; route inventory vs `HTTP-API-v1.md`. |
| evidence ledger endpoint | none | route contract + malformed-line + truncation + 8 MiB cap tests, then commit; unshipped until then. |
| build identity | `BUILD.json` + install receipt | CI check that `BUILD.json.commit == git rev-parse --short HEAD` at package time; fail on mismatch. |
| clean-profile install | default-profile install receipt | `-InstallRoot`/`-DataRoot` temp profile → install → uninstall → reinstall, data preserved, screenshot. |
| UI-2 / gallery | 391 web sites, 16 e2e specs | real-API e2e for HivePulse, 8 cards, tools matrix; route-abort snapshot banner; seeded approval fixture; 375px assertion (not screenshot). |

Existing coverage that must not be over-claimed: the full battery is **offline** (`no provider call or
key was used`), so it proves build/unit/QA health, not model behavior; the scale receipt proves repository
probes, not a 40-node provider run; the install receipt proves an installed build from `806251b`, not HEAD.

---

## 6. Pseudocode for remaining work

```text
# R1 history truth
fn revisions(run_id):
  for row in query(...):
    created = Timestamp::parse(row.created_at)
      .map_err(|_| Refusal::HistoryGap)?          # no 1970 fallback
    out.push(...)
  # test: malformed row -> HistoryGap; API returns JSON error, not 1970

# R2 e2e guard fail-closed
run = spawn(playwright, env PLAYWRIGHT_JSON_OUTPUT_NAME)
report = read_json_or_null(results.json)
if report is null:
  fail("results.json missing/unparsable")          # new
elif report.stats is missing:
  fail("results.json malformed")
elif age(results.json) > run window:               # stale
  fail("stale results.json")
elif passed==0 and failed==0 and skipped>0 and !allow_all_skips:
  fail("all skipped")
exit(run.status if run.ok else 1)

# R3 jail closure
fn new_with_data_dir(workspace, data_dir, db_path, db):
  deny = [data_dir, data_dir/ledger]
  for suffix in ["", "-wal", "-shm"]:
    p = canonicalize(db_path + suffix) or fail_closed   # E_JAIL_CANONICAL
    deny.push(p)
  ToolsBridge { workspace, deny, db }
# update 4 constructor sites: http.rs, tools_bridge.rs, lead_bugcheck_boot.rs, suspension.rs
# tests: --db outside data root -> file_read/file_write refuse; workspace read still succeeds

# R4 planner pre-call cap
fn Plan::validate():
  ...existing checks...
  fanout = group nodes by parent, count
  if any(count > 8): return Err(Refusal::CallLimit)   # before provider call
# test: provider mock counts calls; invalid fan-out => 0 calls, CallLimit

# R5 monitor actions (WP-GOV-2)
const ROOT_MONITOR_SYSTEM = "... you may send_instruction/cancel_task; bounded repair ..."
tools = [send_instruction{task_id,text}, cancel_task{task_id,reason}]
after dispatch:
  review = root.monitor_turn(task_states)            # bounded, one turn
  for call in review.tool_calls:
    validate target belongs to run and is non-terminal
    if send_instruction: persist text for task; child reads on next turn
    if cancel_task: run_state -> cancelled (not raw status write)
  if failed_task terminal and caps remain and no replacement exists:
    create exactly one replacement branch (dedupe by parent+objective hash)
finalize_root(): no tools, truthful status/changes/non-changes/evidence
# tests: assign->instruct->final; late mutation refusal; cap refusal before row creation

# R6 API/CLI truth
token = env HIVEROUTER_LAUNCH_TOKEN or read %LOCALAPPDATA%\HiveMind\launch.token
assert token != "" and never print(token)
mutations over WS only; reads over documented HTTP
write docs/contracts/HTTP-API-v1.md from the router route table (generated, then diffed in test)

# R7 remaining docs/release
INDEX.md rows: {observed_at, head_commit, what_proves, what_it_does_not_prove}
DOWNLOADS.md: package path+sha256, install steps, first-run key, uninstall, data retention
README web_fetch: state receipt scope (bytes/body_not_captured) not a blanket trust claim

# R8 evidence ledger (candidate discarded; only if it is revived)
re-add after: route test, malformed-file test, cap test, docs/contracts entry, UI consumer
```

---

## 7. Priority ranking (realistic)

### (a) Time to implement at high quality

| Rank | Item | Estimate |
|---|---|---|
| 1 | e2e guard fail-closed (R2) | 0.5 day |
| 2 | history refusal (R1) | 0.5–1 day (endpoint + UI handling) |
| 3 | handshake deletion + build.rs extraction (R7) | 0.5 day |
| 4 | planner fan-out pre-call (R4) | 0.5–1 day |
| 5 | docs INDEX/DOWNLOADS + claim scopes (R6) | 0.5–1 day |
| 6 | CLI launch-token + HTTP contract doc (R6) | 1–1.5 days |
| 7 | db-path jail closure (R3) | 1–2 days (4 constructors, canonicalization, tests) |
| 8 | monitor actions + repair (R5) | 3–5 days (new tools, engine state, scripted provider tests) |
| 9 | evidence-ledger ship (if kept) | 1 day + battery |
| 10 | UI-2 polish | 3–5 days |
| 11 | Mission gallery + e2e | 2–3 days |

### (b) Easiest → hardest

e2e guard → history refusal → handshake deletion/docs → planner fan-out → CLI/document →
db-path jail → evidence ledger → monitor actions → UI-2 → Mission gallery.
(Rationale: each step adds one behavior with a local test; monitor is the first item that changes
engine state and provider schema, UI-2 the first that needs new screens.)

### (c) Dependency order

1. **Truth gates first**: e2e guard + history + handshake (unblocks trustworthy release evidence).
2. **Containment**: db-path jail (closes SEC-1 before any new tool ships).
3. **Planner cap** (independent; can run parallel with 2).
4. **API contract + CLI token** (needed before monitor tools are exposed through any surface).
5. **Monitor actions/repair** (requires approval path and accounting; after 1–4).
6. **Evidence ledger** (optional; after 1 and 4; do not ship before route tests).
7. **UI-2 / Mission gallery** (no hard dependency, but should follow the API/fixtures being final).
8. **Docs/release closeout** (last, after the claims it records are true).

### (d) User impact

| Rank | Item | Why the user feels it |
|---|---|---|
| 1 | Monitor cannot instruct/cancel/repair | The advertised "governor" behavior is absent; runs that go wrong stay wrong. |
| 2 | History shows 1970 | Visible wrong timestamps in run history; erodes trust in evidence. |
| 3 | e2e guard fail-open | Broken UI can ship with a "passing" guarded run — release-quality risk. |
| 4 | db-path jail gap | Containment claim is narrower than documented (`file_read` of a custom DB). |
| 5 | CLI launch token missing | Operator workflows either rely on env inheritance or fail; security posture. |
| 6 | Planner fan-out refusal timing | A bad plan burns a paid planning call before refusal. |
| 7 | UI-2 polish / gallery | Discoverability and first-run clarity; no correctness impact. |
| 8 | Docs INDEX/DOWNLOADS | Trust in receipts; maintenance burden. |
| 9 | Handshake dead API | No user impact; code hygiene. |

**Recommended next pass (highest value per day):** items 1, 2, 3 from (a) — e2e guard, history,
handshake/build — plus planner fan-out. That converts the plan's "gate truth" and "edge truth" claims
from partial to done without touching engine state. Monitor actions are the largest user-visible gap and
should be the next dedicated 3–5 day block after those four land.

---

## 8. Task-25 record

No shared task-registry tool is mounted in this subagent session (tool surface: filesystem, shell,
web, sessions; no task board). task-25 is therefore claimed by delivering this file and reporting it to
the parent `session-8c72559d-…`; if the parent holds a separate task registry, it should mark
task-25 complete against this path. Nothing in this session edited tracked code; the only write is
this report.

## 9. One-line bottom line

Plan core (containment, approval TTL/binding, v24 instruction, root schemas, docs checker, installer,
battery) is real; **the plan's own hard parts — WP-GOV-2 monitor actions, WP-GOV-3 atomic accounting,
WP-HARDEN-1 edge truth, WP-API-1/UI-2/MISSION-1, and strict gate/battery at HEAD — remain open**, and
the biggest remaining user-facing overclaim is labelling the review-only pass a monitor.
