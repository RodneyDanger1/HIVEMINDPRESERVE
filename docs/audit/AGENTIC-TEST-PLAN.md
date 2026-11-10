# AGENTIC TEST PLAN - HiveMind HMARS live scenarios (task-20)

| Field | Value |
|---|---|
| Task | `task-20` |
| Repo | `C:\Users\Riley\Desktop\hiverouter-v1` |
| Baseline requested | `919c3398c5bf47b4deb9ed9fd2004d2a0916cb9a` (`master`, tree clean when this task started) |
| Observed at final verification | HEAD `197ee28f0ec392018549c36e09826d073a003cbf` (docs-only commit adding live A2A/remote/daemon evidence) plus a **still-moving** uncommitted lane in 6 source/test files - snapshot at 00:44 in §1.1 |
| Date | 2026-11-10 (America/Chicago) |
| Mode | read-only design. No source edit, no commit, no provider key used, no paid run by this task. |
| Only write by this task | `docs/audit/AGENTIC-TEST-PLAN.md` (the other untracked/modified files in §1.1 are another lane's, not this task's) |
| Deliverable | 6 distinct live scenarios (S1-S6) + 1 safe no-key refusal probe (P0) |
| Verdict | **DESIGN ONLY.** No scenario below was executed by this task. Prior live results are cited; all new assertions are UNVERIFIED until the Lead runs them. |

This plan is written so the Lead can run one scenario at a time, stop on a hard fail, and keep the
total spend bounded. Each scenario is 1-3 minutes of wall time after server start, uses a fresh
temporary data directory, and writes evidence only under `docs/evidence/agentic/task-20/`.

## 0. Lead run card

| Order | ID | What it proves | Prior live? | Provider run | Expected tokens | Ceiling | Wall cap |
|---|---|---|---|---|---|---|---|
| 0 | P0 | keyless submit refuses, creates no run, spends nothing | no | no | 0 | 0 | 60 s |
| 1 | S6 | duplicate action is idempotent; late action after revoke is refused | no | no | 0 | 0 | 90 s |
| 2 | S5 | a live HyperAgent->Worker spawn is refused by name | no | yes (2 tasks) | ~1.7 k | 5 k | 90 s |
| 3 | S2 | one `ask_parent`, one answer, child continues, duplicate answer refused | partial (Test B: parent answer) | yes (3 tasks) | ~7 k | 15 k | 120 s |
| 4 | S1 | exact two-branch topology; root cannot add a third branch | yes (Test A) | regression only | ~15.5 k | 30 k | 180 s |
| 5 | S3 | run/hive/branch memory read sets do not leak across branches | no | yes (5 tasks) | ~10 k | 20 k | 150 s |
| 6 | S4 | root control creates a detached remote + daemon; offline action is refused | partial (Test C: creation) | yes (3 tasks) | ~6 k | 20 k | 180 s |

**Safest first three: P0, S6, S5.** They are first because they have the smallest blast radius and
the least model dependence: P0 cannot call a provider, S6 is a pure control-plane probe, and S5
only needs a two-task run whose root task appears before dispatch. If the probe is not counted as
a scenario, the safest three scenarios are **S6, S5, S3** (the net-new scenarios). S1 is a
regression repeat because Test A already passed; S2 and S4 are net-new only for their unproven
branches (duplicate answer / offline action + daemon tick).

Hard stop: 5 live submissions maximum, 65 k cumulative expected tokens, 90 k absolute ceiling,
20 minutes total wall time. Any stop condition in §11.2 ends the session; partial evidence is
recorded as `inconclusive`, never as pass.

## 1. Evidence base and status labels

Status labels used in this plan, following the repo's own claim discipline:

- **VERIFIED (prior live)** - a runtime result already recorded in this repo, cited by file.
- **READ (source)** - read in the HEAD source or docs; not executed in this task.
- **UNVERIFIED** - not proven by any runtime result here.

| Evidence | What it supports | Gap this plan closes |
|---|---|---|
| `docs/audit/AGENTIC-LIVE-TESTS.md:11-40` and `docs/evidence/live/live-tokenlab-hierarchy.json` | **VERIFIED (prior live, 197ee28):** Test A exact-two: prompt recorded, 7 tasks, class matrix at every depth (hyper/super/agent/subagent), 15,551 tokens, ~106 s; root checkpoint requested `spawn_superagent` and did not add a task. | Prompt differs from S1's stricter prompt; no repeated run; only one sample. S1 is a regression repeat with an explicit root attempt, not first proof. |
| `docs/audit/AGENTIC-LIVE-TESTS.md:54-98` and `docs/evidence/live/live-tokenlab-a2a.json` | **VERIFIED (prior live, 197ee28):** Test B `ask_parent`: 3 tasks, one message, `status=answered`, `attempts=1`, `answer_actor=parent:<super>`, 6,924 tokens. | Parent path only; no duplicate-answer or operator-fallback assertion. S2 adds those. |
| `docs/audit/AGENTIC-LIVE-TESTS.md:100-138` and `docs/evidence/live/live-tokenlab-remote-daemon.json` | **VERIFIED (prior live, 197ee28):** Test C root control created `Porch` (light, in_process, `enrolled`, device_count 1, grant_count 1, created by hyper root) and `Pulse` (watchdog, job `{"check":"db"}`, `enrolled`), 3 tasks, 5,555 tokens, 162 s. | Creation only; no action, no offline refusal, no heartbeat, no daemon tick. S4 adds those. |
| `docs/audit/AGENTIC-LIVE-TESTS.md:140-157` and `docs/evidence/live/live-tokenlab-rate-limit.json` | **VERIFIED (prior live, 197ee28):** local RPM governor returned `E_LLM_RATE_LIMIT` in 0 s with 0 tasks and no provider request; retry after one window completed. | Not a scenario here; used only as an environment stop rule. |
| `docs/audit/AGENTIC-LIVE-TESTS.md:175-183` | **READ (prior live doc):** explicit next tests: parent scheduling, lower-level remote request, memory Venn, repeat exact-two x3, 40-node probe. | This plan covers memory (S3), repeat exact-two (S1), and a remote refusal/action path (S4/S6); it does not cover parent scheduling, lower-level remote approval, or 40-node scale. |
| `docs/audit/REMOTE-PERMISSION-GAPS.md` (untracked, task-19, pinned 919c339) | **READ (source audit):** G1-G10. Most relevant: duplicate remote name -> `E_INTERNAL_ERROR`; duplicate daemon accepted; same device idempotency key with a conflicting payload returns the prior action; A2A answer after deadline can be accepted; root branch gate is an unconditional refusal with no durable row; root creation has no approval/idempotency; refused remote actions are not audited (`:323-361`, `:425-433`, `:445-474`, `:519-576`). | Supplies the negative probes for S6-G and prevents S1/S4 from overclaiming a conditional gate, approval, or idempotency. |
| `docs/audit/AGENT-TAXONOMY-VERIFICATION.md` (untracked, task-17, pinned 919c339) | **READ (source audit):** G04 root flat plan can add up to 4 branches; G05 root tools bypass tool audit/cap; G17 root final memory may land in an orphan branch bank; role spelling changes persona/tool authority (`:477-493`). | Constrains how S1/S3/S4 results are interpreted; not a runtime result. |
| `docs/evidence/HMARS-BUGFIX-REPORT.md:204-222` and `docs/evidence/live/live-tokenlab-hierarchy.json` | **VERIFIED (prior live):** exact-two run completed, 7 tasks, 14,407 prompt + 1,144 completion = 15,551 tokens, ~106 s; earlier pre-fix run produced 10 tasks. | Superseded by the richer Test A record at 197ee28; retained for the pre-fix 10-task reproduction. |
| `docs/evidence/HMARS-BUGFIX-REPORT.md:211` and `docs/evidence/live/live-tokenlab-simple.json` | **VERIFIED (prior live):** 2-task run completed in ~26 s, 1,701 tokens. | Same gap; no reusable template for the hierarchy probe. |
| `docs/evidence/live/live-tokenlab-kill-usage.json` | **VERIFIED (prior live):** hard kill after checkpoint recovered 7 tasks as `interrupted`, 449 tokens retained. | Not used here; crash/recovery is out of scope for this plan. |
| `docs/evidence/HMARS-BUGFIX-REPORT.md:60-79, 201-202, 243-274` | **READ (report):** A2A answer/CAS/deadline/attempts, memory bank tree, remote/daemon rules, and the explicit remaining-unverified list. | Report is prose; this plan turns the claims into assertions against live routes. |
| `docs/audit/hivemind-memory-a2a/TEST-PLAN.md` and `AUDIT.md` | **READ (source audit, pre-fix):** test intent for A2A answer, sibling isolation, terminal recipients, budget. Audit baseline was `7610b86`; some findings are stale after the repair wave. | Source-level tests, not live scenarios; this plan uses the intent only. |
| `backend/crates/hiverouter-core/src/execution.rs:3860-4000` | **READ (source):** one root control turn, max 4 tools, `spawn_superagent` refused when the planner already supplied a depth-1 branch, `spawn_remote_agent` and `spawn_daemon` create detached records. | Needs live proof that a real provider run keeps 7 tasks and creates no task row for remotes/daemons. |
| `backend/crates/hiverouter-core/src/execution.rs:3193-3440` | **READ (source):** `ask_parent` durable open row, 120 s deadline, duplicate open question refused with `E_CALL_LIMIT`, parent answer attempted once (`attempts` capped at 2), operator fallback waiter, CAS answer. | Needs live proof of exactly one answer and continuation. |
| `backend/crates/hiverouter-core/src/execution.rs:4337-4364` and `3683-3728` | **READ (source):** task text is auto-written to a bank by role (`super` -> `bank:hive:<task>`, `agent`/`subagent` -> `bank:branch:<task>`); read set is run + nearest hive + parent branch + own branch; context snapshot records bank ids and entry ids. | Needs live bank/read-set inspection. |
| `backend/crates/hiverouter-core/src/http.rs:600-670, 1016-1075, 1745-2128, 2249-2418` | **READ (source):** the routes used below exist; answer CAS maps `E_TASK_TERMINAL` to HTTP 409; remote refusals map `E_REMOTE_OFFLINE` to 409 and `E_REMOTE_UNKNOWN` to 404. | Needs live status-code proof. |
| `backend/crates/hiverouter-core/src/remote.rs:312-370, 633-674, 898-1131` | **READ (source):** remotes start `enrolled` (offline), heartbeat sets `online` with a 120 s lease, `request_action` refuses offline/revoked/capability/in-flight, idempotency key returns the recorded action. | Needs live duplicate/late proof. |
| `backend/crates/hiverouter-core/src/daemon.rs:220-331, 500-540` | **READ (source):** enable schedules a tick now; a `watchdog` job really checks the DB and reports `{"executed":true,"check":"db"}`; an unimplemented job reports `degraded` with `executed:false`. | Needs live report proof. |
| `backend/crates/hiverouter-core/src/execution.rs:195-226` and `backend/crates/hiverouter-tools/src/tool.rs:394-445` | **READ (source):** strict class matrix (Hyper->Super only, Super->Agent only, Agent->Subagent only); `ask_parent` is on Super, Worker, and Subagent allowlists. | Needs live refusal proof for the spawn gate. |
| `docs/AGENT-CLI.md` and `skills/hivemind/SKILL.md` | **READ (docs):** 50-command CLI, JSON envelope `{ok, command, data, error}`, exit codes 0/2/3/4, `spawn-subhive`, `memory`, `roster`, `state`, `agent-messages`. | Runtime verification is explicitly not claimed by the docs; this plan supplies it. |
| `e2e/README.md:12-24, 75-79` | **READ (e2e):** 79 passed / 14 explicitly skipped on the shipped binary; hard rule "No live paid API" for e2e. Node cards/streaming/terminal-run truth remain unverified. | This plan is a separate headless CLI/HTTP drill, not a Playwright suite; it does not change the e2e result. |
| `runtime-test/s9.out`, `runtime-test/s9.err` | **READ (old runtime log):** server booted on `127.0.0.1:8799` with schema v3 on 2026-11-06; stderr empty. | Pre-schema-23 smoke, not evidence for current behavior; superseded by the live JSON above. |

Artifact identity used by this plan (verified by file hash at design time):

| Item | Value |
|---|---|
| `target/release/hiverouter.exe` | 36,340,736 bytes; SHA-256 `EAF16EB85B2FAA8648F794466643B79D4A1E2F212F7581668FCFE4E4044E5CCC` |
| `dist/HiveMind/hiverouter.exe` | same size/hash as above |
| Source after release | `git log f988ef2..197ee28 -- backend web tools` is empty; both `919c339` and `197ee28` are docs-only after the release build. The **uncommitted** edits in §1.1 are not in either commit. |
| Inconsistent older hash | `docs/evidence/HMARS-BUGFIX-REPORT.md:150` lists a different build hash (`E1FB...`) than line 8 and the current file. Use the hash above for this run; record the actual preflight hash and flag the report inconsistency in the evidence. |

### 1.1 Workspace drift observed at final verification (read this before running)

This design started at `919c339` with `git status --porcelain` empty. At final read-back the tree had
moved:

```text
HEAD 197ee28  docs: record live A2A, remote/daemon, and rate-governor tests
  added by that commit:
    docs/audit/AGENTIC-LIVE-TESTS.md
    docs/evidence/live/live-tokenlab-a2a.json
    docs/evidence/live/live-tokenlab-remote-daemon.json
    docs/evidence/live/live-tokenlab-rate-limit.json
    docs/evidence/HMARS-BUGFIX-REPORT.md (modified)
  (committed; docs-only)

Uncommitted, another lane (NOT this task; tree was still moving at the 00:44 snapshot):
  M  backend/crates/hiverouter-core/src/execution.rs        (+55/-2: root duplicate-name guards,
                                                             "DIRECT CHILD RESULT" prompt label)
  M  backend/crates/hiverouter-core/src/remote.rs           (+18/-4: hyper-created remote actor_kind/audit)
  M  backend/crates/hiverouter-core/src/store.rs            (+1: class "hyper" -> bank:run)
  M  backend/crates/hiverouter-core/src/provider_executor.rs (appeared after 00:42; inspect diff)
  M  backend/crates/hiverouter-core/src/ws.rs                (appeared after 00:42; inspect diff)
  M  backend/crates/hiverouter-core/tests/run_lifecycle.rs  (+20: grant/audit assertions)
  ?? docs/audit/AGENT-TAXONOMY-VERIFICATION.md
  ?? docs/audit/REMOTE-PERMISSION-GAPS.md
  ?? docs/audit/CAPABILITY-MATRIX.md
```

Consequences for the Lead:

1. The plan's `file:line` references are to the `919c339` source. The committed `197ee28` is docs-only,
   so they are still valid there. The **uncommitted** edits shift lines: `remote.rs` below `:351` by
   roughly +9, `execution.rs` after `:3911` by roughly +33, `store.rs` after `:1346` by +1, and
   `provider_executor.rs`/`ws.rs` were still being edited at the snapshot. Do not use the dirty line
   numbers to judge a refusal; re-read the file at the frozen commit.
2. The pinned binary `DB0183...` predates both docs commits and the uncommitted lane. It is still the
   release-built binary for the unchanged `f988ef2` source. If the Lead builds after the dirty lane,
   the hash will change; record the new hash and re-read the remote/daemon assertions.
3. Do not run S4/S6 against the dirty tree until it is frozen or reverted. The dirty lane changes
   remote audit actor attribution (`system` vs `operator`), grant `granted_by`, root duplicate-name
   guard text, and hyper memory bank selection. S4 does not assert actor_kind; S6's operator-created
   path may still behave as task-19 describes, but a mixed tree is not a clean claim. Freeze first.
4. The dirty `execution.rs` duplicate-name guards are a partial, unreviewed response to task-19 G3;
   they cover the **root control** path, not the operator `POST /api/remotes` duplicate in S6-G.
   Do not count them as a fix for S6.
5. The new Test A/B/C records are **prior live evidence at `197ee28`**, not results of this task.
   They reduce the novelty of S1/S2/S4 creation; the plan keeps them as re-confirmation/extension.

## 2. Shared harness and safety rules

### 2.1 Rules (non-negotiable)

1. **No source changes, no commits.** The run may create only: temporary data under
   `%TEMP%\hmars-agentic\task-20\`, evidence under `docs/evidence/agentic/task-20/`, and this plan.
2. **No key in any command, log, or evidence file.** Paid scenarios use the Lead's already
   authorized provider environment. If `runtime.provider.key_present` is not `true`, stop and mark
   the scenario `blocked`; do not prompt for a key, do not put a key on a command line.
3. **No launcher token for this harness.** Start the server through the CLI (not
   `start-hivemind.ps1`), with `HIVEROUTER_LAUNCH_TOKEN` removed from the process environment.
   Then `/api/session` and the GET routes are reachable loopback-only, and POSTs use the per-boot
   session token. Never persist the `/api/session` response; keep the token in memory only.
4. **One fresh server and one fresh data directory per scenario.** Do not reuse a DB between
   scenarios; a stale remote/daemon/run makes assertions ambiguous.
5. **No retry without Lead approval.** A model that ignores the prompt is `inconclusive`, not a
   product failure. A named refusal is evidence; a fabricated success is a failure.
6. **Evidence is redacted.** Before saving, scan for `sk-`, `Bearer `, `Authorization`,
   `DEEPSEEK_API_KEY=`, `HIVEROUTER_API_KEY=`, `launch=`, and `"token"`. A hit fails the run.

### 2.2 Start / stop / preflight (copy once per scenario)

```powershell
$repo = 'C:\Users\Riley\Desktop\hiverouter-v1'
Set-Location $repo
$cli  = Join-Path $repo 'HiveMind-Agent.cmd'
$exe  = Join-Path $repo 'target\release\hiverouter.exe'
$port = 8098
$url  = "http://127.0.0.1:$port"
$base = Join-Path $env:TEMP 'hmars-agentic\task-20'
$ev   = Join-Path $repo 'docs\evidence\agentic\task-20'
New-Item -ItemType Directory -Force -Path $base, $ev | Out-Null

# A scenario gets its own data dir, e.g. s1-data, s2-data...
$data = Join-Path $base 's1-data'
$ws   = Join-Path $base 'ws'
New-Item -ItemType Directory -Force -Path $data, $ws | Out-Null

# Keep the harness tokenless and let the Lead's provider environment be inherited.
Remove-Item Env:HIVEROUTER_LAUNCH_TOKEN -ErrorAction SilentlyContinue

& $cli start --json --port $port --data-dir $data --workspace-root $ws --exe $exe
& $cli health  --json --url $url
& $cli runtime --json --url $url
& $cli doctor  --json --url $url
& $cli runs    --json --url $url --limit 5

# Keep the per-boot token in memory for the HTTP probes; never save this object.
$session = Invoke-RestMethod "$url/api/session"
$H = @{ Authorization = "Bearer $($session.token)" }
```

Preflight gate for S1-S5 (all must hold before submitting):

- `runtime.provider.key_present` is `true`.
- `runtime.provider.billing` is recorded (`external_unverified` for TokenLab, `deepseek_usd` for
  DeepSeek). No USD/points total is claimed from this field.
- `runs` is empty or no run is active.
- `health.database.schema_version` is recorded (prior live evidence reports 23).

If the gate fails: `stop --yes`, mark `blocked`, stop the session.

Stop and clean (after evidence is copied):

```powershell
& $cli stop --json --yes --url $url
Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue   # must be empty
Remove-Item -Recurse -Force $data, $ws
git -C $repo status --porcelain   # only the plan/evidence additions, never source
```

### 2.3 HTTP helper (for S2, S3, S4, S6, P0)

```powershell
$session = Invoke-RestMethod "$url/api/session"   # may be repeated after 2.2; do NOT save it
$H = @{ Authorization = "Bearer $($session.token)" }

function Invoke-Api {
    param([string]$Method, [string]$Path, [object]$Body)
    $p = @{ Method = $Method; Uri = "$url$Path"; Headers = $H; ErrorAction = 'Stop' }
    if ($null -ne $Body) {
        $p.ContentType = 'application/json'
        $p.Body = ($Body | ConvertTo-Json -Depth 8 -Compress)
    }
    try {
        $r = Invoke-WebRequest @p
        [pscustomobject]@{ status = [int]$r.StatusCode; body = $r.Content }
    } catch {
        $resp = $_.Exception.Response
        $text = ''
        if ($resp) {
            $stream = $resp.GetResponseStream()
            if ($stream) {
                $reader = New-Object System.IO.StreamReader($stream)
                $text = $reader.ReadToEnd()
            }
        }
        if (-not $text -and $_.ErrorDetails) { $text = $_.ErrorDetails.Message }
        [pscustomobject]@{ status = if ($resp) { [int]$resp.StatusCode } else { 0 }; body = $text }
    }
}

function Wait-Run {
    param([string]$RunId, [int]$Seconds)
    $end = (Get-Date).AddSeconds($Seconds)
    do {
        Start-Sleep -Seconds 2
        $r = (& $cli runs --json --url $url --limit 20 2>$null | ConvertFrom-Json).data.runs |
             Where-Object { $_.run_id -eq $RunId }
    } while ($r -and $r.status -notin @('completed','failed','cancelled','interrupted') -and (Get-Date) -lt $end)
    $r
}
```

CLI reads return `{ok, command, data, meta}`; a refusal returns `{ok:false, error:{code,...}}` and
exit 3. Use the CLI's exit code, not only the parsed body.

### 2.4 Evidence layout and `result.json`

Per scenario: `docs/evidence/agentic/task-20/<id>/` with the raw JSON captures and one
`result.json`:

```json
{
  "task": "task-20",
  "scenario": "S1",
  "head": "919c3398c5bf47b4deb9ed9fd2004d2a0916cb9a",
  "binary_sha256": "EAF16EB85B2FAA8648F794466643B79D4A1E2F212F7581668FCFE4E4044E5CCC",
  "started_at": "2026-11-10T...Z",
  "wall_seconds": 0,
  "prompt_sha256": "64-hex",
  "prompt_text_path": "prompt.txt",
  "run_id": "uuid-or-null",
  "status": "pass|fail|inconclusive|blocked",
  "assertions": [
    { "name": "task_count", "expected": 7, "observed": 7, "ok": true }
  ],
  "usage": { "total_tokens": 0 },
  "refusals": [ { "code": "E_TOOL_NOT_ALLOWED", "status": 0, "where": "..." } ],
  "cleanup": { "server_stopped": true, "port_closed": true, "data_dir_removed": true },
  "notes": []
}
```

Never copy `$session.token` or any `Authorization` header into evidence. A prompt is stored as
`prompt.txt` plus its SHA-256; it contains no secrets.

## 3. S1 - Exact-two hierarchy and root branch governance

**Covers:** two branch leads only; exact 7-task tree; HyperAgent root control cannot add a branch
when the planner already supplied one; no detached records appear as tasks.

**Prior evidence:** **VERIFIED (prior live, 197ee28):** Test A already recorded this topology with
its prompt, 7 tasks, the full class matrix, 15,551 tokens, ~106 s, and a root checkpoint request
that added no task (`AGENTIC-LIVE-TESTS.md:11-40`; `live-tokenlab-hierarchy.json`). S1 is therefore
a **regression/repeat**, not first proof. Run it only if a fresh-hash regression or a second sample
is required; if Test A is accepted, skip S1 and go to S2/S3/S5/S6. The active root attempt here is
still unobservable as text (the control result is not persisted), so the live pass is outcome-level.

**Preconditions:** paid preflight passes; no active run; fresh `s1-data`.

**Exact prompt (verbatim):**

```text
Build exactly two independent branches and no more.
Topology: exactly two SuperAgents at depth 1; exactly one Agent at depth 2 under each SuperAgent; exactly one Subagent at depth 3 under each Agent. That is six planned nodes under the HyperAgent root; add no other node.
Each Subagent must return exactly one line "<branch>:<role> ready", where <branch> is A or B.
In the HyperAgent root control turn, call spawn_superagent exactly once even though the plan already supplies two branch leads, then call finish. Do not add a third branch.
```

**Expected path (pseudocode):**

```text
SUBMIT(P1) -> run queued, RUN_CREATED
plan = provider.plan(P1)
expect 6 nodes: super A/B depth1, agent A/B depth2, subagent A/B depth3
plan.validate() -> parent/depth/class ok
insert root HyperAgent            (task 1)
insert 6 planned tasks            (tasks 2..7), publish NODE_SPAWN x7
root control (one turn, <=4 tools):
  spawn_superagent -> has_planned_branch=true -> refusal text only
  finish -> return
dispatch ready nodes in bounded parallel batches
root final synthesis -> RUN_STATUS completed
```

**Run:**

```powershell
$p1 = @'
Build exactly two independent branches and no more.
Topology: exactly two SuperAgents at depth 1; exactly one Agent at depth 2 under each SuperAgent; exactly one Subagent at depth 3 under each Agent. That is six planned nodes under the HyperAgent root; add no other node.
Each Subagent must return exactly one line "<branch>:<role> ready", where <branch> is A or B.
In the HyperAgent root control turn, call spawn_superagent exactly once even though the plan already supplies two branch leads, then call finish. Do not add a third branch.
'@
New-Item -ItemType Directory -Force -Path "$ev\s1" | Out-Null
$r1 = (& $cli submit --prompt $p1 --project agentic-task20 --json --url $url 2>$null | ConvertFrom-Json)
$run1 = $r1.data.run_id
$s1 = Wait-Run -RunId $run1 -Seconds 180
Start-Sleep -Seconds 2
& $cli state  --run $run1 --json --url $url | Set-Content -Encoding utf8 "$ev\s1\state.json"
& $cli roster --run $run1 --json --url $url | Set-Content -Encoding utf8 "$ev\s1\roster.json"
& $cli usage  --run $run1 --json --url $url | Set-Content -Encoding utf8 "$ev\s1\usage.json"
(Invoke-Api GET "/api/remotes").body | Set-Content -Encoding utf8 "$ev\s1\remotes.json"
(Invoke-Api GET "/api/daemons").body | Set-Content -Encoding utf8 "$ev\s1\daemons.json"
```

**Evidence to capture:** run id/status/error_code; roster `agents[]` with `task_id`,
`parent_task_id`, `role`, `class`, `depth`; `count`; state graph nodes; usage total; wall seconds;
prompt text + SHA-256; server binary hash; `/api/remotes` and `/api/daemons` counts (expected 0,
recorded as observation).

**Pass (all required):**

- P1.1 run status `completed`, `error_code` null.
- P1.2 roster count exactly 7.
- P1.3 exactly 2 tasks at depth 1 with class `super`; exactly 2 at depth 2 with class `agent`;
  exactly 2 at depth 3 with class `subagent`; exactly 1 root with `parent_task_id:null` and depth 0.
- P1.4 each depth-2 task's parent is one of the depth-1 tasks; each depth-3 task's parent is one of
  the depth-2 tasks; all 7 ids unique.
- P1.5 no depth >3 and no third depth-1 branch.
- P1.6 total tokens <= 30,000 (prior live: 15,551).
- P1.7 no remote or daemon id appears in the roster (record remotes/daemons as observation; an
  unexpected control-plane row is not a topology failure but is reported).

**Falsifiers / fail classification:**

- **F1 (product, block):** count >= 8 or three or more depth-1 SuperAgents -> root branch
  governance or plan expansion regressed.
- **F2 (inconclusive, not product):** count < 7 or wrong classes while the run completed ->
  planner/model drift. Record the prompt and roster; do not call this a governance pass.
- **F3 (inconclusive):** run failed `E_SCHEMA_INVALID`/planner repair exhausted -> malformed plan,
  not a topology verdict.
- **F4 (environment):** provider transport/auth/rate-limit error -> no product verdict.
- **Limit:** the control-turn refusal text is not persisted or fed back
  (`execution.rs:3996-3999`), so a live pass is **outcome-level** (no 8th task). The refusal path
  itself is covered by the existing unit test
  `hyperagent_checkpoint_refuses_an_extra_branch_when_the_planner_has_one`
  (`backend/crates/hiverouter-core/tests/run_lifecycle.rs:599-630`). Do not claim live observation
  of the refusal text.
- **Gate reality (task-19 audit):** every valid plan has a depth-1 node, so `has_planned_branch` is
  always true and the root branch gate is an **unconditional refusal**; the allow path is
  unreachable (`REMOTE-PERMISSION-GAPS.md:376-401`, `:435-443`). S1 tests the refusal outcome, not a
  conditional decision. There is no durable gate row, so a pass must not claim one.

**Budget:** 1 live submission; expected 15.5 k tokens; ceiling 30 k; 1 point-budget unit; 180 s.
This is a regression repeat: if Test A is accepted, S1 costs 0 and is skipped.

**Cleanup:** if still active `& $cli cancel --run $run1 --yes --json --url $url`; stop; remove
`s1-data`; keep evidence.

**Meaning of failure:** F1 means HMARS topological authority is broken at the root checkpoint; this
is a release blocker. F2/F3 mean the provider/model did not honor an exact topology, so HMARS
topology control is unproven, not disproven.

## 4. S2 - A2A `ask_parent` with exactly one answer

**Covers:** one question, one durable answer, the child continues with the answer, the operator
fallback CAS works, and a duplicate answer is refused.

**Prior evidence:** **VERIFIED (prior live, 197ee28):** Test B already proved the parent-answer
path: 3 tasks, one `ask_parent` row, `status=answered`, `attempts=1`, `answer_actor=parent:<super>`,
6,924 tokens (`AGENTIC-LIVE-TESTS.md:54-98`; `live-tokenlab-a2a.json`). This scenario re-confirms
that path and adds the two branches Test B did not cover: operator fallback (if the parent answer is
unavailable) and a duplicate answer CAS refusal (HTTP 409 `E_TASK_TERMINAL`).

**Preconditions:** paid preflight; fresh `s2-data`.

**Exact prompt (verbatim):**

```text
Plan exactly two nodes: one SuperAgent at depth 1 and one Agent at depth 2 under it. Add no other node.
The branch code for this run is BEE-42. The Agent must call ask_parent exactly once with the text "What is the branch code for this run?" before it finishes, then include the answer it receives in its result. The SuperAgent must answer from the run goal. Neither task may spawn another agent.
```

**Expected path (pseudocode):**

```text
SUBMIT(P2) -> 3 tasks (root, super, agent)
agent.execute:
  ask_parent(text="What is the branch code for this run?")
  find_open_agent_message -> none
  record_agent_message_limited -> open row; deadline = now + 120 s
  record_agent_message_attempt(actor=parent:<super>, action=parent_answer) -> attempts=1
  executor.answer_escalation(parent=super, run_prompt contains BEE-42)
    -> {"answer":"BEE-42","needs_operator":false}
  answer_agent_message CAS open -> answered, actor=parent:<super>
  TOOL_RESULT {status:"answered", answer:"BEE-42"}
agent continues; result includes BEE-42
run completed
```

Operator fallback (only if the parent answer fails or returns `needs_operator:true`):

```text
message stays open
operator POST /api/agents/runs/{run}/messages/{id}/answer {"answer":"BEE-42"}
CAS open -> answered, actor=operator
wake_a2a_waiter -> child resumes with the answer
```

**Run:**

```powershell
$p2 = @'
Plan exactly two nodes: one SuperAgent at depth 1 and one Agent at depth 2 under it. Add no other node.
The branch code for this run is BEE-42. The Agent must call ask_parent exactly once with the text "What is the branch code for this run?" before it finishes, then include the answer it receives in its result. The SuperAgent must answer from the run goal. Neither task may spawn another agent.
'@
New-Item -ItemType Directory -Force -Path "$ev\s2" | Out-Null
$r2 = (& $cli submit --prompt $p2 --project agentic-task20 --json --url $url 2>$null | ConvertFrom-Json)
$run2 = $r2.data.run_id

# Observe the question; answer only if still open at ~10 s.
$msg = $null
for ($i=0; $i -lt 30; $i++) {
    Start-Sleep -Seconds 1
    $m = (& $cli agent-messages --run $run2 --json --url $url 2>$null | ConvertFrom-Json).data.messages
    $msg = $m | Where-Object { $_.kind -eq 'ask_parent' } | Select-Object -First 1
    if ($msg) { break }
}
if ($msg -and $msg.status -eq 'open') {
    $answer = Invoke-Api POST "/api/agents/runs/$run2/messages/$($msg.id)/answer" @{ answer = 'BEE-42' }
    $answer | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 "$ev\s2\operator-answer.json"
}
$s2 = Wait-Run -RunId $run2 -Seconds 120
& $cli agent-messages --run $run2 --json --url $url | Set-Content -Encoding utf8 "$ev\s2\messages.json"
& $cli messages      --run $run2 --json --url $url | Set-Content -Encoding utf8 "$ev\s2\chat.json"
& $cli roster        --run $run2 --json --url $url | Set-Content -Encoding utf8 "$ev\s2\roster.json"

# Duplicate answer probe (after the row is answered), expected 409 E_TASK_TERMINAL.
if ($msg) {
    $dup = Invoke-Api POST "/api/agents/runs/$run2/messages/$($msg.id)/answer" @{ answer = 'BEE-42' }
    $dup | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 "$ev\s2\duplicate-answer.json"
}
```

**Evidence to capture:** the one `ask_parent` message object (`id`, `from_task_id`,
`to_task_id`, `status`, `deadline_at`, `answered_at`, `answer_text`, `answer_actor`, `attempts`,
`last_attempt_at`); child assistant message text; run status; operator/duplicate HTTP status+body;
prompt hash.

**Pass (all required):**

- P2.0 roster count is exactly 3 (root, SuperAgent, Agent); otherwise mark plan drift as
  `inconclusive` before judging A2A.
- P2.1 exactly one row with `kind:"ask_parent"` for the run.
- P2.2 its `status` is `answered`; `answer_text` trims to `BEE-42`; `answered_at` and
  `deadline_at` are non-null.
- P2.3 `answer_actor` is `parent:<super-task-id>` or `operator`; `attempts` is 1 (or 2 only if a
  parent attempt was audited and then the operator answered).
- P2.4 the Agent's result / assistant message contains `BEE-42`; the run completed; no open or
  expired `ask_parent` row remains.
- P2.5 duplicate answer POST returns HTTP 409 with `E_TASK_TERMINAL`; the row is still `answered`
  and `answer_text` is unchanged.

**Falsifiers / fail classification:**

- **F1 (product, block):** row remains `open`/`expired` at terminal, or `answer_text` is empty, or
  the child result claims `BEE-42` with no answered row -> fabricated answer or broken wake.
- **F2 (product, block):** two `ask_parent` rows, or duplicate answer returns 200 and changes the
  row -> CAS/duplicate guard failed.
- **F3 (inconclusive):** zero rows -> the Agent never called `ask_parent`; model steering, not an
  A2A verdict.
- **F4 (environment):** provider error before the tool call -> no verdict.
- **Note:** if the parent answers with a different string, record it; a wrong answer from the
  parent is a model-context failure, while an empty/forged answer is a product failure. The child
  must not invent the code.

**Budget:** 1 live submission; expected ~7 k tokens (Test B: 6,924); ceiling 15 k; 1 point-budget
unit; 120 s. If Test B is accepted and only the duplicate-answer/operator branches matter, the
parent-answer path is still exercised by the same run; there is no cheaper split.

**Cleanup:** cancel if active; stop; remove `s2-data`; keep evidence.

**Meaning of failure:** F1/F2 mean the core HMARS A2A promise (`ask -> answer -> continue`) is not
live; a release blocker. F3 means the prompt/steering did not exercise A2A, so the claim stays
unverified rather than failed.

**Known A2A gaps (not all live-tested here):** task-19 records that a duplicate **open** question
returns `E_CALL_LIMIT` without an `a2a_audit` row, an answer after `deadline_at` but before the
expiry sweep is accepted, and an open message on a terminal run can be answered
(`REMOTE-PERMISSION-GAPS.md:337-340`, `:445-454`). S2's duplicate-after-answered probe expects the
safe 409; it does not wait out the 120 s deadline or force a terminal-run open row, because both
would exceed the 1-3 minute bound. Those remain source-level test proposals.

## 5. S3 - Memory / branch scoping

**Covers:** run bank, two hive banks, two branch banks; a task's read set excludes the sibling
hive and sibling branch; the HyperAgent summary projection has no raw branch bank; scoped reads
return only their scope.

**Prior evidence:** source-read (`execution.rs:4337-4364`, `3683-3728`; `store.rs:1324-1353`,
`2180-2228`, `2784-2815`). None of the 197ee28 live tests A-D exercises memory; `AGENTIC-LIVE-TESTS.md:181`
lists "memory Venn: prove no sibling/foreign-run bleed" as a next test. The pre-fix audit's
M-02/M-06 findings motivated the bank tree; this scenario checks the post-fix live behavior.

**Preconditions:** paid preflight; fresh `s3-data`. This run is independent of S1: the markers below
are what make the read-set evidence unambiguous, and S1's prompt does not create them.

**Exact prompt (verbatim):**

```text
Plan exactly four nodes: two SuperAgents at depth 1, and exactly one Agent at depth 2 under each SuperAgent. Add no Subagent and no other node.
The first SuperAgent and its Agent are branch A; the second SuperAgent and its Agent are branch B. The Agent in branch A must end its result with the exact marker MEM-ALPHA; the Agent in branch B must end its result with the exact marker MEM-BETA. The SuperAgent in branch A must end its result with HIVE-ALPHA; the SuperAgent in branch B must end its result with HIVE-BETA. Never repeat another branch's marker.
```

**Expected path (pseudocode):**

```text
SUBMIT(P3) -> 5 tasks (root, super A/B, agent A/B)
complete_task(super A) -> record_agent_memory_scoped(role=super, key=task:<id>, text, scope=task.id)
  -> bank:hive:<super A>; value ends HIVE-ALPHA
complete_task(super B) -> bank:hive:<super B>; value ends HIVE-BETA
complete_task(agent A) -> bank:branch:<agent A>; value ends MEM-ALPHA
complete_task(agent B) -> bank:branch:<agent B>; value ends MEM-BETA
agent A at execution start:
  memory_read_set(run, agent A, parent=super A) ->
    bank:run:<run>, nearest hive bank=hive:<super A>, own branch=branch:<agent A>
  the own branch has no entries yet (the Agent has not completed), so the recorded
  context snapshot normally contains bank:run and bank:hive:<super A> only
  record_memory_context_snapshot(banks, entry_ids, rendered_chars, prompt_sha256)
```

**Run (independent variant):**

```powershell
$p3 = @'
Plan exactly four nodes: two SuperAgents at depth 1, and exactly one Agent at depth 2 under each SuperAgent. Add no Subagent and no other node.
The first SuperAgent and its Agent are branch A; the second SuperAgent and its Agent are branch B. The Agent in branch A must end its result with the exact marker MEM-ALPHA; the Agent in branch B must end its result with the exact marker MEM-BETA. The SuperAgent in branch A must end its result with HIVE-ALPHA; the SuperAgent in branch B must end its result with HIVE-BETA. Never repeat another branch's marker.
'@
New-Item -ItemType Directory -Force -Path "$ev\s3" | Out-Null
$r3 = (& $cli submit --prompt $p3 --project agentic-task20 --json --url $url 2>$null | ConvertFrom-Json)
$run3 = $r3.data.run_id
$s3 = Wait-Run -RunId $run3 -Seconds 150
& $cli roster --run $run3 --json --url $url | Set-Content -Encoding utf8 "$ev\s3\roster.json"

$roster = (& $cli roster --run $run3 --json --url $url | ConvertFrom-Json).data.agents
$superA = ($roster | Where-Object { $_.depth -eq 1 } | Select-Object -First 1).task_id
$superB = ($roster | Where-Object { $_.depth -eq 1 } | Select-Object -Last 1).task_id
$agentA = ($roster | Where-Object { $_.depth -eq 2 -and $_.parent_task_id -eq $superA }).task_id
$agentB = ($roster | Where-Object { $_.depth -eq 2 -and $_.parent_task_id -eq $superB }).task_id
if (-not $agentA -or -not $agentB) {
    throw 'S3 inconclusive: expected exactly two depth-2 agents; see roster.json'
}

(Invoke-Api GET "/api/memory/banks?run_id=$run3").body          | Set-Content -Encoding utf8 "$ev\s3\banks.json"
(Invoke-Api GET "/api/memory/summary?run_id=$run3").body        | Set-Content -Encoding utf8 "$ev\s3\summary.json"
(Invoke-Api GET "/api/memory/context/$agentA").body             | Set-Content -Encoding utf8 "$ev\s3\context-a.json"
(Invoke-Api GET "/api/memory/context/$agentB").body             | Set-Content -Encoding utf8 "$ev\s3\context-b.json"
(Invoke-Api GET "/api/agents/runs/$run3/memory?scope=$agentA").body | Set-Content -Encoding utf8 "$ev\s3\scope-a.json"
(Invoke-Api GET "/api/memory/banks/bank:hive:$superA/entries?run_id=$run3").body | Set-Content -Encoding utf8 "$ev\s3\entries-hive-a.json"
(Invoke-Api GET "/api/memory/banks/bank:hive:$superB/entries?run_id=$run3").body | Set-Content -Encoding utf8 "$ev\s3\entries-hive-b.json"
(Invoke-Api GET "/api/memory/banks/bank:branch:$agentA/entries?run_id=$run3").body | Set-Content -Encoding utf8 "$ev\s3\entries-branch-a.json"
(Invoke-Api GET "/api/memory/banks/bank:branch:$agentB/entries?run_id=$run3").body | Set-Content -Encoding utf8 "$ev\s3\entries-branch-b.json"
```

Map branch A/B by marker value, not only by first/last roster order: if the planner emits the two
branches in the opposite order, the `HIVE-ALPHA`/`HIVE-BETA` values identify A/B unambiguously.
Record the mapping in `result.json`.

**Evidence to capture:** bank list (`id`, `kind`, `scope_id`, `owner_task_id`, `run_id`);
summary `summaries[].bank_id`; context `banks` array, `entry_ids`, `rendered_chars`, `truncated`,
`prompt_sha256`; bank entries (`id`, `bank_id`, `key`, `value`); scoped memory rows (`scope_id`,
`value`); roster ids used to join.

**Pass (all required):**

- P3.1 bank list contains `bank:run:<run>`, `bank:hive:<superA>`, `bank:hive:<superB>`,
  `bank:branch:<agentA>`, `bank:branch:<agentB>`; every bank's `run_id` is the current run.
- P3.2 `bank:hive:<superA>` owner is `superA`; `bank:branch:<agentA>` owner is `agentA`; same for B.
- P3.3 context A `banks` contains `bank:run:<run>` and `bank:hive:<superA>`; it does **not**
  contain `bank:hive:<superB>` or `bank:branch:<agentB>`. An own `bank:branch:<agentA>` entry is
  allowed but not required, because the Agent's own row is written only when it completes. If the
  Agent started before its SuperAgent completed, `hive:<superA>` may be absent; record the task
  start/complete times and mark only the hive-inclusion check inconclusive, not the sibling check.
- P3.4 context B is symmetric (contains B's hive/run, excludes A's hive and branch).
- P3.5 the four task-bank entries contain only their own marker: hive A `HIVE-ALPHA`, hive B
  `HIVE-BETA`, branch A `MEM-ALPHA`, branch B `MEM-BETA`. The **run bank is exempt**: the engine
  stores the operator prompt there (`execution.rs:1798-1805`), so its value necessarily contains
  all markers. The scoping claim is about task banks, not prompt text.
- P3.6 `/api/memory/summary` has only `bank:run:` and `bank:hive:` bank ids; no `bank:branch:` id.
- P3.7 scoped read for `agentA` returns only rows whose `scope_id` is `agentA`, and its values
  contain `MEM-ALPHA`, not `MEM-BETA`; the `agentB` scoped read is the inverse.
- P3.8 each context has `prompt_sha256` of 64 hex chars and `truncated` boolean; `rendered_chars` > 0.
- P3.9 context A's `entry_ids` is disjoint from the entry ids in branch B (and hive B); context B is
  disjoint from branch A (and hive A). The context endpoint does not return values, so this id
  comparison is the value-level isolation check; the bank entries in P3.5 prove the values.

Optional S3b (branch-inheritance proof, not required for S3 pass): add one Subagent under each
Agent (2 more nodes, 7 tasks, +~6 k tokens, same ceiling 20 k). A Subagent's context should include
its parent Agent's `bank:branch:` value (`MEM-ALPHA` for subagent A) and exclude the other branch.
Run S3b only if the Lead wants the parent-branch read path exercised live; the 5-task S3 already
proves bank creation, run/hive isolation, and scoped-read separation.

**Falsifiers / fail classification:**

- **F1 (product, security, block):** a sibling **task bank** id (`bank:hive:<superB>` or
  `bank:branch:<agentB>`) or one of its entry ids appears in context A/B, or the scoped read returns
  the other agent's row -> cross-branch memory leak. The run-bank prompt entry is exempt (it holds
  the operator prompt, which intentionally names both branches).
- **F2 (product, block):** `memory/summary` includes a `bank:branch:` bank -> root aggregate reads
  raw branch rows.
- **F3 (inconclusive):** banks missing because the run used fewer than two Supers/Agents (plan
  drift) -> the scoping claim was not exercised.
- **F4 (environment):** run failed before task completion -> no verdict.

**Budget:** 1 live submission (5 tasks), expected ~10 k tokens, ceiling 20 k, 1 point-budget unit,
150 s. The optional S3b variant is 7 tasks and must stay under the same 20 k ceiling.

**Cleanup:** cancel if active; stop; remove `s3-data`; keep evidence.

**Meaning of failure:** F1/F2 mean the HMARS memory topology is not safe: sibling/raw branch data
reaches a task or the root. That is a data-isolation release blocker. F3 means the scenario did
not create the banks, so the claim remains unverified.

## 6. S4 - Root-control remote + daemon creation, with an offline-action refusal

**Covers:** the HyperAgent root control turn creates a detached remote (with device + narrow grant)
and a detached daemon (watchdog job), neither becomes a task row; an action before heartbeat is
refused; after heartbeat it runs; the daemon reports what actually ran.

**Prior evidence:** **VERIFIED (prior live, 197ee28):** Test C already proved root creation of a
detached `Porch` light (in_process, enrolled, 1 device, 1 grant, hyper actor) and a `Pulse` watchdog
(job `{"check":"db"}`, enrolled), 3 tasks, 5,555 tokens, 162 s
(`AGENTIC-LIVE-TESTS.md:100-138`; `live-tokenlab-remote-daemon.json`). Source-read for the action
path (`execution.rs:3929-3972`, `4005-4079`; `remote.rs:312-370`, `898-1131`;
`daemon.rs:304-331`). This scenario adds the unproven branches: offline-action refusal,
heartbeat/action, and a real daemon tick report.

**Preconditions:** paid preflight; fresh `s4-data`; no active run.

**Exact prompt (verbatim):**

```text
Plan exactly two nodes: one SuperAgent at depth 1 and one Agent at depth 2 under it. Add no other node.
In the HyperAgent root control turn, call spawn_remote_agent exactly once with display_name "Desk Lamp" and kind "light", and call spawn_daemon exactly once with display_name "Hive Watchdog", kind "watchdog", interval_secs 5. Do not call spawn_superagent. Then call finish.
The Agent must return exactly one line: "branch ok".
```

**Expected path (pseudocode):**

```text
SUBMIT(P4) -> 3 tasks
root control (one turn):
  spawn_remote_agent("Desk Lamp","light")
    -> remote_agents state=enrolled, owner_kind=hive, created_by=hyper:<root>
    -> one device class=light, one grant (power,toggle); no task row
  spawn_daemon("Hive Watchdog","watchdog",5)
    -> daemons state=enrolled, owner_kind=hive, job={"check":"db"}; no task row
  finish
dispatch 2 tasks -> completed
POST /api/devices/{device}/actions {"action":"toggle"} before heartbeat
  -> 409 E_REMOTE_OFFLINE; no device_actions row
POST /api/remotes/{id}/heartbeat -> state=online; lease=now+120 s
POST action toggle -> 200 status=confirmed
POST /api/daemons/{id}/switch {"enabled":true} -> state=running, next_tick=now
supervisor tick (2 s cadence) -> report outcome=ok, detail.executed=true, check=db
```

**Run:**

```powershell
$p4 = @'
Plan exactly two nodes: one SuperAgent at depth 1 and one Agent at depth 2 under it. Add no other node.
In the HyperAgent root control turn, call spawn_remote_agent exactly once with display_name "Desk Lamp" and kind "light", and call spawn_daemon exactly once with display_name "Hive Watchdog", kind "watchdog", interval_secs 5. Do not call spawn_superagent. Then call finish.
The Agent must return exactly one line: "branch ok".
'@
New-Item -ItemType Directory -Force -Path "$ev\s4" | Out-Null
$r4 = (& $cli submit --prompt $p4 --project agentic-task20 --json --url $url 2>$null | ConvertFrom-Json)
$run4 = $r4.data.run_id
$s4 = Wait-Run -RunId $run4 -Seconds 180
& $cli roster --run $run4 --json --url $url | Set-Content -Encoding utf8 "$ev\s4\roster.json"

$remotes = (Invoke-Api GET "/api/remotes").body | ConvertFrom-Json
$daemons = (Invoke-Api GET "/api/daemons").body | ConvertFrom-Json
$remotes | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 "$ev\s4\remotes.json"
$daemons | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 "$ev\s4\daemons.json"

$remote = $remotes.remotes | Where-Object { $_.remote.display_name -eq 'Desk Lamp' } | Select-Object -First 1
$daemon = $daemons.daemons | Where-Object { $_.display_name -eq 'Hive Watchdog' } | Select-Object -First 1
if (-not $remote -or -not $daemon) {
    throw 'S4 inconclusive: the root control turn did not create both records; see remotes.json/daemons.json'
}
$detail = (Invoke-Api GET "/api/remotes/$($remote.remote.remote_id)").body | ConvertFrom-Json
$detail | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 "$ev\s4\remote-detail.json"
if ($detail.devices.Count -eq 0 -or $detail.grants.Count -eq 0) {
    throw 'S4 inconclusive: remote has no device/grant; root creation was partial'
}
$device = $detail.devices[0].device_id
$auditBefore = Invoke-Api GET "/api/remotes/$($remote.remote.remote_id)/audit"
$auditBefore | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 "$ev\s4\audit-before.json"

$pre = Invoke-Api POST "/api/devices/$device/actions" @{ capability_id='power'; action='toggle'; idempotency_key='s4-pre' }
$pre | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 "$ev\s4\pre-heartbeat-action.json"

$hb = Invoke-Api POST "/api/remotes/$($remote.remote.remote_id)/heartbeat" @{}
$hb | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 "$ev\s4\heartbeat.json"
$act = Invoke-Api POST "/api/devices/$device/actions" @{ capability_id='power'; action='toggle'; idempotency_key='s4-on' }
$act | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 "$ev\s4\action.json"
$auditAfter = Invoke-Api GET "/api/remotes/$($remote.remote.remote_id)/audit"
$auditAfter | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 "$ev\s4\audit-after.json"

$sw = Invoke-Api POST "/api/daemons/$($daemon.daemon_id)/switch" @{ enabled = $true }
$sw | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 "$ev\s4\daemon-switch.json"
Start-Sleep -Seconds 6
$rep = Invoke-Api GET "/api/daemons/$($daemon.daemon_id)/reports"
$rep | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 "$ev\s4\daemon-reports.json"
```

**Evidence to capture:** run status/roster; remotes list and detail (`owner_kind`, `created_by`,
`state`, device `class`, grant `action`); daemons list (`kind`, `job`, `state`, `created_by`);
pre-heartbeat action status/body (expect 409 `E_REMOTE_OFFLINE`); audit before/after; post-heartbeat
action; daemon reports (`outcome`, `detail.executed`, `detail.check`); wall times.

**Pass (all required):**

- P4.1 run completed; roster count exactly 3; no remote/daemon id in the roster.
- P4.2 exactly one remote named `Desk Lamp`, `owner_kind:"hive"`, `created_by:"hyper:<root>"`,
  adapter/class consistent with `light`.
- P4.3 its detail has exactly one device (class `light`) and at least one grant for
  `capability_id:"power"`, `action:"toggle"`. Note the root tool description says "has no grant"
  (`provider_executor.rs:556-558`) while `execution.rs:4043-4079` creates one; the code and this
  test expect a grant. Record the wording mismatch, do not fail on it.
- P4.4 exactly one daemon named `Hive Watchdog`, `kind:"watchdog"`, job `{"check":"db"}`,
  `state:"enrolled"` before the switch (source insert uses `enrolled`; the root tool description
  says "starts stopped" - record the actual value and flag the wording mismatch, do not fail on
  the wording alone).
- P4.5 pre-heartbeat action is HTTP 409 with `E_REMOTE_OFFLINE`; the remote audit has no
  `device_action` row for the pre-heartbeat key.
- P4.6 heartbeat returns `state:"online"` with a non-null lease; the action then returns HTTP 200
  `status:"confirmed"`; the device state version increments once.
- P4.7 switch returns `state:"running"`; within 6 s at least one report has `outcome:"ok"` and
  `detail.executed:true` and `detail.check:"db"`; no report claims `ok` with `executed:false`.

**Falsifiers / fail classification:**

- **F1 (product, block):** a remote or daemon becomes a task row / graph node, or roster count >3
  because of it -> detached-record invariant broken.
- **F2 (product, security, block):** pre-heartbeat action returns 200, or the offline remote is
  treated as online -> fabricated authority.
- **F3 (product, block):** daemon reports `ok` without `executed:true`, or reports `alive` for an
  unimplemented job -> fake liveness.
- **F4 (inconclusive):** the root made 0 or 1 tool calls (no remote/daemon rows) -> model steering,
  not a control-plane verdict. Do not retry without Lead approval.
- **F5 (inconclusive):** missing device/grant while the remote exists -> root creation partial; this
  is closer to a product defect, but distinguish it from a model that never called the tool.

**Budget:** 1 live submission (3 tasks), expected ~6 k tokens (Test C: 5,555), ceiling 20 k,
1 point-budget unit, 180 s. Test C took 162 s, so 150 s was too tight; 180 s covers the run plus the
action/tick checks. Creation alone is already prior-proven; the budget buys the offline refusal,
action, and daemon tick assertions.

**Cleanup:** `POST /api/daemons/{id}/retire`; `POST /api/remotes/{id}/revoke`; stop; remove
`s4-data`; keep evidence.

**Meaning of failure:** F1-F3 mean the detached control plane is not truthful or not detached; those
are release blockers for the remote/daemon claim. F4 means root tool steering is unverified, not
that creation is broken.

**Known control-plane gaps (record, do not overclaim):** task-19 finds root creation has no
approval/request/idempotency record, daemon creation has no audit row, and a refused remote action
is not audited (`REMOTE-PERMISSION-GAPS.md:415-433`, `:455-474`). S4 therefore asserts **creation,
detachment, and the offline refusal**, not an approval flow. P4.5 checks that no `device_action` row
exists; the absence of a **refusal audit** row is a known gap to report, not an S4 pass claim.

## 7. S5 - Live hierarchy violation refused by the spawn gate

**Covers:** while a real run is active, a `SPAWN_SUB_HIVE` from the HyperAgent root requesting a
Worker is refused by name (`E_TOOL_NOT_ALLOWED`) and inserts no task.

**Prior evidence:** source-read (`execution.rs:195-226`, `1440-1486`; `ws.rs:956-964`); unit test
`strict_hierarchy_refuses_direct_hyper_agent_and_super_subagent_edges`
(`run_lifecycle.rs:537-562`) and the Worker->Super refusal at `run_lifecycle.rs:1155-1166`.
There is no existing live probe for the WS gate.

**Preconditions:** paid preflight; fresh `s5-data`; no active run.

**Exact prompt (verbatim):**

```text
Reply with exactly: OK.
```

This is intentionally the cheapest 2-task plan (prior live: 1,701 tokens, ~26 s). The test target
is the spawn gate, not the model.

**Expected path (pseudocode):**

```text
SUBMIT(P5) -> 2 tasks, run active
poll GET /api/agents/runs/{run}/roster until a root task exists
  root: parent_task_id=null, depth=0, role=planner
CLI spawn-subhive --parent <root> --role worker
  WS SPAWN_SUB_HIVE -> engine.spawn_sub_hive(root,"worker")
  validate_spawn_role(HyperAgent,"worker") -> Err(ToolNotAllowed)
  publish_error E_TOOL_NOT_ALLOWED
  no task row inserted; roster count unchanged
run continues to completion
```

**Run:**

```powershell
New-Item -ItemType Directory -Force -Path "$ev\s5" | Out-Null
$r5 = (& $cli submit --prompt 'Reply with exactly: OK.' --project agentic-task20 --json --url $url 2>$null | ConvertFrom-Json)
$run5 = $r5.data.run_id

$rootTask = $null
for ($i=0; $i -lt 80; $i++) {
    Start-Sleep -Milliseconds 250
    $agents = (& $cli roster --run $run5 --json --url $url 2>$null | ConvertFrom-Json).data.agents
    $rootTask = $agents | Where-Object { $null -eq $_.parent_task_id } | Select-Object -First 1
    if ($rootTask) { break }
}
if (-not $rootTask) {
    throw 'S5 inconclusive: no root task observed before the run ended; see result.json'
}
$before = (& $cli roster --run $run5 --json --url $url | ConvertFrom-Json).data
$beforeStatus = ((& $cli runs --json --url $url --limit 20 | ConvertFrom-Json).data.runs |
                 Where-Object { $_.run_id -eq $run5 }).status
$spawn  = & $cli spawn-subhive --parent $rootTask.task_id --role worker --json --url $url 2>$null
$spawnExit = $LASTEXITCODE
$spawn | Set-Content -Encoding utf8 "$ev\s5\spawn.json"
@{ exit = $spawnExit; status_before = $beforeStatus; root_task_id = $rootTask.task_id } |
    ConvertTo-Json -Depth 4 | Set-Content -Encoding utf8 "$ev\s5\spawn-meta.json"
$after = (& $cli roster --run $run5 --json --url $url | ConvertFrom-Json).data
$s5 = Wait-Run -RunId $run5 -Seconds 90
```

**Evidence to capture:** run id; root task id/depth; `spawn.json` (error code or ACK);
`spawn-meta.json` (exit code, run status at attempt, root id); roster count before/after; run status
after the attempt; wall time. Save the exact role requested.

**Pass (all required):**

- P5.1 the root task was observed while the run was still active (not terminal).
- P5.2 `spawn-subhive` exits 3 with `error.code:"E_TOOL_NOT_ALLOWED"`; there is no
  `"spawned":true` and no ACK payload.
- P5.3 roster count is unchanged after the attempt, and no child task with `role:"worker"` and
  `parent_task_id:<root>` appears.
- P5.4 the run still reaches a terminal state (completed preferred) with no error caused by the
  refused attempt.

**Falsifiers / fail classification:**

- **F1 (product, critical, block):** exit 0 / `spawned:true` / a new Worker child appears ->
  hierarchy authority bypass.
- **F2 (product, gate-order defect):** a different code (`E_SPAWN_DEPTH`, `E_PERSONA_UNKNOWN`,
  `E_RUN_TERMINAL`) -> the wrong gate answered. `E_RUN_TERMINAL` while the run was already
  terminal makes the probe **inconclusive**, not a bypass.
- **F3 (inconclusive):** root task never observed within 20 s, or the run completed before the
  attempt -> race; retry once only if budget remains and the Lead approves, otherwise mark
  `inconclusive`.

**Budget:** 1 live submission (2 tasks), expected ~1.7 k tokens, ceiling 5 k, 1 point-budget unit,
90 s.

**Cleanup:** cancel if active; stop; remove `s5-data`; keep evidence.

**Meaning of failure:** F1 is the most serious result in this plan: a non-Hyper edge was accepted,
so the class matrix is not enforcing authority. F2 means the edge was refused but by a different
gate; that is still a refusal, but it does not prove the hierarchy gate. F3 is a timing miss.

**Known boundary:** task-19 records that a leaf trying a root-only tool such as
`spawn_remote_agent` gets `E_TOOL_NOT_FOUND` because the tool is absent from its allowlist, with no
request/approval row (`REMOTE-PERMISSION-GAPS.md:348`, `:405-413`). That path is **not** driven live
here: the tool is not advertised, so a model cannot call it reliably. S5 tests the server-side
`SPAWN_SUB_HIVE` gate that the CLI/UI can actually invoke.

## 8. S6 - Late / duplicate remote request (idempotency + revoke)

**Covers:** a duplicate action with the same idempotency key returns the recorded action and does
not re-actuate; an action after remote revoke is refused; two known-gap probes record whether a
duplicate remote name and a conflicting idempotency payload are safely refused. No provider key,
no run, no points.

**Prior evidence:** source-read (`remote.rs:898-1131`, `414-446`; `http.rs:1943-1998`,
`1705-1718`). Existing `http_boot.rs:980` covers tokenless remote creation; task-19 documents the
current duplicate/late gaps (`REMOTE-PERMISSION-GAPS.md:323-361`, `:425-433`, `:465-474`). This
scenario covers the safe ordering/idempotency behavior and adds two known-gap probes (S6-G).

**Preconditions:** fresh `s6-data`; no provider key needed; server running tokenless.

**Exact prompt:** none - this is a control-plane probe, not a model run. The exact HTTP request
sequence below is the test input.

**Exact request sequence (no model prompt):**

```text
POST /api/remotes                 {"display_name":"S6 Probe","owner_kind":"operator","owner_id":"operator"}
POST /api/remotes/{id}/devices    {"class":"light","name":"S6 Lamp","safety_class":"reversible"}
POST /api/remotes/{id}/grants     {"device_id":"<dev>","capability_id":"power","action":"on"}
POST /api/devices/{dev}/actions   {"capability_id":"power","action":"on","idempotency_key":"s6-once"}
  before heartbeat -> 409 E_REMOTE_OFFLINE
POST /api/remotes/{id}/heartbeat  {}
POST /api/devices/{dev}/actions   {"capability_id":"power","action":"on","idempotency_key":"s6-once"}
  -> 200 confirmed, action A
POST /api/devices/{dev}/actions   {"capability_id":"power","action":"on","idempotency_key":"s6-once"}
  -> 200 action A, same action_id; no second row
POST /api/devices/{dev}/actions   {"capability_id":"power","action":"off","idempotency_key":"s6-once"}
  S6-G: safe = 409 conflict; current expected 200 prior action A (G7)
POST /api/remotes                 {"display_name":"S6 Probe","owner_kind":"operator","owner_id":"operator"}
  S6-G: safe = named duplicate or same id; current expected 503 E_INTERNAL_ERROR (G3)
POST /api/remotes/{id}/revoke     {}
POST /api/devices/{dev}/actions   {"capability_id":"power","action":"on","idempotency_key":"s6-late"}
  -> 404 E_REMOTE_UNKNOWN
```

**Expected path (pseudocode):**

```text
POST remote -> 201, state=enrolled, no lease
POST device -> 201; POST grant(power,on) -> 201
POST action(key=s6-once) before heartbeat
  -> state != online -> 409 E_REMOTE_OFFLINE; no device_actions row; state_version 0
POST heartbeat -> 200, state=online, lease=now+120
POST action(key=s6-once, action=on) -> 200 confirmed A; state_version 1; one audit row
POST action(same key, action=on) -> 200 A; no second row; state_version 1
POST action(same key, action=off) -> current: 200 A (G7 gap); safe: named conflict
POST remote(same display_name) -> current: 503 E_INTERNAL_ERROR (G3 gap); safe: named duplicate/same id
POST revoke -> 200 state=revoked
POST action(key=s6-late) -> 404 E_REMOTE_UNKNOWN; no row; audit unchanged
```

**Run:**

```powershell
New-Item -ItemType Directory -Force -Path "$ev\s6" | Out-Null
$rem = (Invoke-Api POST '/api/remotes' @{ display_name='S6 Probe'; owner_kind='operator'; owner_id='operator' }).body | ConvertFrom-Json
$dev = (Invoke-Api POST "/api/remotes/$($rem.remote_id)/devices" @{ class='light'; name='S6 Lamp'; safety_class='reversible' }).body | ConvertFrom-Json
$gr  = (Invoke-Api POST "/api/remotes/$($rem.remote_id)/grants" @{ device_id=$dev.device_id; capability_id='power'; action='on' }).body | ConvertFrom-Json

$pre = Invoke-Api POST "/api/devices/$($dev.device_id)/actions" @{ capability_id='power'; action='on'; idempotency_key='s6-once' }
$audit0 = Invoke-Api GET "/api/remotes/$($rem.remote_id)/audit"
$dev0 = Invoke-Api GET "/api/devices/$($dev.device_id)"
$hb  = Invoke-Api POST "/api/remotes/$($rem.remote_id)/heartbeat" @{}
$one = Invoke-Api POST "/api/devices/$($dev.device_id)/actions" @{ capability_id='power'; action='on'; idempotency_key='s6-once' }
$two = Invoke-Api POST "/api/devices/$($dev.device_id)/actions" @{ capability_id='power'; action='on'; idempotency_key='s6-once' }
$conflict = Invoke-Api POST "/api/devices/$($dev.device_id)/actions" @{ capability_id='power'; action='off'; idempotency_key='s6-once' }
$dupRemote = Invoke-Api POST '/api/remotes' @{ display_name='S6 Probe'; owner_kind='operator'; owner_id='operator' }
$audit1 = Invoke-Api GET "/api/remotes/$($rem.remote_id)/audit"
$dev1 = Invoke-Api GET "/api/devices/$($dev.device_id)"
$rev = Invoke-Api POST "/api/remotes/$($rem.remote_id)/revoke" @{}
$late = Invoke-Api POST "/api/devices/$($dev.device_id)/actions" @{ capability_id='power'; action='on'; idempotency_key='s6-late' }
$audit2 = Invoke-Api GET "/api/remotes/$($rem.remote_id)/audit"

[ordered]@{
    pre = $pre; audit_before = $audit0; device_before = $dev0; heartbeat = $hb; first = $one;
    duplicate = $two; conflict = $conflict; duplicate_remote = $dupRemote;
    audit_first = $audit1; device_after = $dev1; revoke = $rev;
    late = $late; audit_after = $audit2
} | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 "$ev\s6\s6.json"
```

**Evidence to capture:** HTTP status + body for every step; remote id/state; device id/state/version;
grant id; action id/status for the first and duplicate; audit rows with `action:"device_action"`
and `request_id`; conflicting-key response; duplicate-remote response; late action status/code;
revoke response.

**Pass (all required):**

- P6.1 create/device/grant return 201 and the expected ids; remote starts `enrolled` (offline).
- P6.2 pre-heartbeat action returns 409 `E_REMOTE_OFFLINE`; remote audit has zero
  `device_action` rows for `s6-once`; device state version is 0.
- P6.3 heartbeat returns `state:"online"` and a non-null lease.
- P6.4 first online action returns 200 `status:"confirmed"` with action id A.
- P6.5 duplicate action returns 200 with the **same** action id A; the device state version is 1
  (not 2); the audit has exactly one `device_action` row with `request_id:A`.
- P6.6 revoke returns 200 `state:"revoked"`; the late action returns 404 `E_REMOTE_UNKNOWN`;
  the audit still has exactly one `device_action` row and the device state version is unchanged.

**Falsifiers / fail classification:**

- **F1 (product, block):** duplicate creates a second action or increments state version to 2 ->
  idempotency broken.
- **F2 (product, security, block):** late action returns 200 or 409 `E_REMOTE_OFFLINE` instead of
  404 `E_REMOTE_UNKNOWN` -> revoke does not invalidate the principal; or the code is
  `E_DEVICE_UNKNOWN` (different gate; record as gate-order, likely inconclusive for this claim).
- **F3 (inconclusive):** pre-heartbeat action returns 409 for a different reason (e.g. capability)
  -> setup failure, not an offline test.

**S6-G known-gap probes (0 tokens; expected to fail on 919c339, report as gaps):**

- **G3 duplicate remote name:** safe behavior is a named duplicate refusal (for example
  `E_DUPLICATE_REMOTE`) or the same remote id; task-19 says the current code returns 503
  `E_INTERNAL_ERROR` after committing the first remote (`REMOTE-PERMISSION-GAPS.md:329`, `:425-433`).
  Record the actual status/code. A 503/500 `E_INTERNAL_ERROR` is a **gap**, not an acceptable
  refusal.
- **G7 conflicting idempotency payload:** safe behavior is a named conflict (for example 409
  `E_DEVICE_IDEMPOTENCY_CONFLICT`); task-19 says the current code returns the prior action as a
  success and ignores the conflicting `off` (`REMOTE-PERMISSION-GAPS.md:334`, `:465-474`). Record
  whether the response is 200 prior-A (gap), or a named conflict (fixed).
- These probes do not change S6's core pass (P6.1-P6.6); they are release-gate findings for the
  duplicate/late request contract. If either already returns the safe result, record the improvement
  and do not claim the gap.

**Budget:** 0 provider tokens; 0 point-budget units; <90 s; no run.

**Cleanup:** remote already revoked; stop; remove `s6-data`; keep evidence.

**Meaning of failure:** F1/F2 mean a retried or revoked control-plane request can still act; that
is a safety defect in the remote plane. Because the transport is the in-process fixture, this does
**not** prove network remote behavior (still unimplemented per the report).

## 9. P0 - Safe no-key refusal probe (run locally, zero spend)

**Purpose:** prove admission control and key policy without a provider key and without a run.
This is the mandatory first probe; it cannot spend points.

**Preconditions:** no server on 8098; fresh `p0-data`; both key variables removed from the
process environment.

**Run:**

```powershell
$repo = 'C:\Users\Riley\Desktop\hiverouter-v1'; Set-Location $repo
$cli = Join-Path $repo 'HiveMind-Agent.cmd'
$port = 8098; $url = "http://127.0.0.1:$port"
$data = Join-Path $env:TEMP 'hmars-agentic\task-20\p0-data'
$ws   = Join-Path $env:TEMP 'hmars-agentic\task-20\ws-p0'
$ev   = Join-Path $repo 'docs\evidence\agentic\task-20\p0'
New-Item -ItemType Directory -Force -Path $data, $ws, $ev | Out-Null
Remove-Item Env:HIVEROUTER_LAUNCH_TOKEN, Env:DEEPSEEK_API_KEY, Env:HIVEROUTER_API_KEY -ErrorAction SilentlyContinue

& $cli start --json --port $port --data-dir $data --workspace-root $ws --exe (Join-Path $repo 'target\release\hiverouter.exe')
& $cli health  --json --url $url | Set-Content -Encoding utf8 "$ev\health.json"
& $cli runtime --json --url $url | Set-Content -Encoding utf8 "$ev\runtime.json"
& $cli doctor  --json --url $url | Set-Content -Encoding utf8 "$ev\doctor.json"
& $cli runs    --json --url $url --limit 5 | Set-Content -Encoding utf8 "$ev\runs-before.json"
& $cli active-run --json --url $url | Set-Content -Encoding utf8 "$ev\active-before.json"

& $cli submit --prompt 'This must be refused: no provider key is configured.' --project agentic-task20 --json --url $url 2>$null | Set-Content -Encoding utf8 "$ev\submit-refusal.json"
$submitExit = $LASTEXITCODE

& $cli runs    --json --url $url --limit 5 | Set-Content -Encoding utf8 "$ev\runs-after.json"
& $cli active-run --json --url $url | Set-Content -Encoding utf8 "$ev\active-after.json"
& $cli models  --json --url $url | Set-Content -Encoding utf8 "$ev\models-refusal.json"
$modelsExit = $LASTEXITCODE
& $cli balance --json --url $url | Set-Content -Encoding utf8 "$ev\balance-refusal.json"
$balanceExit = $LASTEXITCODE
@{ submit = $submitExit; models = $modelsExit; balance = $balanceExit } |
    ConvertTo-Json | Set-Content -Encoding utf8 "$ev\exit-codes.json"

& $cli stop --json --yes --url $url
```

**Expected (P0):**

- `health.status` is `degraded` (not `ok`); `database.schema_version` is recorded (prior live: 23).
- `runtime.provider.key_present` is `false`; `runtime.status` is `idle`; `caps.max_calls` is 64 and
  `caps.parallel_dispatch` is 32 (prior live/read values; record the live values).
- `doctor` exits 0 with a key-absent check; no key material appears anywhere.
- `submit` exits **3** with `error.code:"E_KEY_MISSING"`; no `RUN_CREATED`; `runs-after` has the
  same count as `runs-before`; `active-run` is null/absent.
- `models` and `balance` exit 3 with `E_KEY_MISSING`.
- Total provider calls: 0. Point spend: 0. No `hiverouter.exe` child left after `stop`.

**Falsifiers / fail classification:**

- **F1 (product, critical):** `submit` exits 0 or a run row appears -> keyless admission failed.
- **F2 (environment/harness):** `runtime.provider.key_present` is `true` despite clearing the env
  -> the probe is contaminated; stop and fix the environment, do not report a pass.
- **F3 (product, low):** `health.status` is `ok` while no key exists -> health truthfulness defect;
  record the checks array.

**Meaning of failure:** F1 means the product can start a run without a key (or fabricates a
success), which is a trust-boundary blocker. F2 means the probe did not test what it claims.

**Cleanup:** `stop --yes` (already last in the run block); confirm port 8098 is closed; remove
`p0-data` and `ws-p0`; keep the evidence. No provider call or run row exists to cancel.

## 10. Failure meanings (cross-scenario)

| Observation | Implication | Lead action |
|---|---|---|
| S1 count >=8 or 3+ depth-1 | Root/topology governance regressed | Block release; capture roster; file defect |
| S2 open/expired or forged answer | A2A answer/wake broken or fabricated | Block release; capture message row and child text |
| S3 sibling/raw branch read | Memory isolation broken | Block release; security finding |
| S4 detached record becomes a task | HMARS ownership invariant broken | Block release |
| S4 offline action succeeds or daemon fakes alive | Control-plane truth/authority broken | Block release |
| S5 spawn accepted | Hierarchy authority bypass | Critical; stop the session |
| S6 duplicate re-acts or late action succeeds | Idempotency/revocation broken | Block remote claim |
| Plan/model drift (wrong task count, no tool call) | Scenario did not exercise the claim | `inconclusive`; do not retry without Lead approval |
| Provider auth/rate-limit/transport error | Environment, not product | `blocked`; no verdict |
| Budget/wall exceeded | Test incomplete | Stop, record partial evidence, no pass |

## 11. Budget, stop rules, and cleanup

### 11.1 Budget summary

| Scenario | Live submits | Expected tokens | Ceiling | Point-budget units | Wall cap |
|---|---|---|---|---|---|
| P0 | 0 | 0 | 0 | 0 | 60 s |
| S6 | 0 | 0 | 0 | 0 | 90 s |
| S5 | 1 | 1.7 k | 5 k | 1 | 90 s |
| S2 | 1 | 7 k | 15 k | 1 | 120 s |
| S1 | 0 if Test A accepted; else 1 | 15.5 k | 30 k | 0 or 1 | 180 s |
| S3 | 1 | 10 k | 20 k | 1 | 150 s |
| S4 | 1 | 6 k | 20 k | 1 | 180 s |
| **Total** | **4 (5 with S1)** | **~40 k** | **90 k** | **4 (5 with S1)** | **~21 min** |

- A "point-budget unit" is one live submission. TokenLab point pricing is not defined in this repo;
  do **not** convert tokens to points or USD. Record the provider balance only if the UI exposes it,
  and never claim a settled cost for an external endpoint (`billing` stays `external_unverified`).
- Stop before submitting if cumulative expected tokens would exceed 65 k, or if the last run's
  `usage.total_tokens` is more than 2x its ceiling.
- On `E_LLM_RATE_LIMIT`/`E_PROVIDER_TIMEOUT`: stop, do not hot-retry; record the code. The 197ee28
  Test D shows the local governor can refuse with zero provider requests; that is a stop signal,
  not a reason to retry tight.

### 11.2 Stop conditions

Stop the whole session and write a `blocked` result if any of these occurs:

1. The paid preflight says `key_present:false`.
2. A scenario's run is still active after its wall cap and `cancel` does not settle it.
3. `stop --yes` fails and port 8098 remains listening (external process).
4. Any evidence file contains a key-like or token-like string.
5. S5 returns an ACK (authority bypass) - stop immediately; do not run S2/S1/S3/S4 on that build.
6. S3 or S4 reveals a live task row that violates a hierarchy/ownership invariant.

### 11.3 Cleanup (every scenario)

```powershell
# If a run is active:
& $cli cancel --run $runId --yes --json --url $url

# Stop only the CLI-owned server:
& $cli stop --json --yes --url $url
Get-NetTCPConnection -LocalPort 8098 -State Listen -ErrorAction SilentlyContinue   # must be empty

# Remove only the temporary data directory; keep evidence.
Remove-Item -Recurse -Force $data, $ws

# Confirm no source change:
git -C $repo status --porcelain
```

`stop` owns only a CLI-started server. If `start` reported `already_running`, the server was external:
do not force-kill it; change the port and re-run, or ask the Lead.

## 12. What this plan does not prove

- **No execution:** every live result here is UNVERIFIED until the Lead runs it. This document is a
  design, not a test report.
- **No scale:** 7 tasks, one run at a time; not 50-150/300-600 concurrency.
- **No re-proof of Test A/B/C:** exact-two, parent-answer A2A, and root remote/daemon **creation**
  are already live-proven at 197ee28. S1 is a regression repeat; S2/S4 add only their unproven
  branches (duplicate/operator answer; offline action/daemon tick). S3, S5, S6 are net-new.
- **No crash/recovery, no network remotes, no daemon script jobs, no browser UI, no release artifact
  packaging.** Remote transport here is the in-process fixture only (report §8).
- **No live proof of the root-control refusal text:** S1 observes the outcome (no 8th task), not
  the internal refusal, because the control result is not persisted. The unit test covers the path.
- **No A2A `send_to_sibling`, cross-branch approval, or terminal-recipient matrix.** The source
  audit's T-P0-17/T-P0-18 remain source-level tests to write/run.
- **No security exploit run:** no key exfiltration, no DNS-rebinding, no auth bypass beyond P0.
- **No points/USD claim:** TokenLab point pricing is not in-repo; external billing is
  `external_unverified`.

## 13. Lead checklist

- [ ] Confirm HEAD `919c339` (or record the actual HEAD `197ee28`) and the server binary SHA-256.
- [ ] Freeze or revert the uncommitted lane in §1.1 before S4/S6; otherwise record it as a confound.
- [ ] Run P0 first; confirm `E_KEY_MISSING`, zero runs, zero provider calls.
- [ ] Run S6; confirm duplicate/late refusals before any provider spend.
- [ ] Run S5; confirm `E_TOOL_NOT_ALLOWED` and no roster change.
- [ ] Confirm paid preflight (`key_present:true`, billing recorded) before S2/S1/S3/S4.
- [ ] Run S2; capture the message row, child text, and duplicate-answer 409.
- [ ] Skip S1 if Test A is accepted; otherwise run it and capture roster/usage.
- [ ] Run S3; capture banks, summary, contexts, and bank entries.
- [ ] Run S4; capture remote/daemon rows and the offline refusal/action/tick.
- [ ] Redaction scan; write `result.json` per scenario; stop server; remove temp dirs.
- [ ] `git status --porcelain` shows only this plan plus evidence; no source change.
- [ ] Update the HMARS bug-fix report only after evidence exists; no pass claims without the
      captured JSON.
