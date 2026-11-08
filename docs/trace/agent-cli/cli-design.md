# HiveMind production CLI — agent-native design (audit-only)

**Status:** design only. No product code, test, configuration, launcher, or state file was edited; HiveMind was not started, stopped, signalled, or probed; no browser was launched; no provider key was read, decrypted, printed, or stored.

**Date:** 2026-11-08 (America/Chicago). **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`, HEAD `36ce7d3`.

**Seat / skills applied:** `agent-native-design` (P0–P7, the 14-criterion rubric, `references/design-patterns.md`, `references/testing.md`, `references/hybrid-mcp-cli.md`) and the `agent-experience` duties. There is **no standalone `/agent-experience` skill installed** in this profile; the duties applied are the TestDrive Fixer 3 definition — "agent/user flow, discoverability, error context, UX" (`C:\Users\Riley\.agents\skills\testdrive\SKILL.md:50`) — plus the agent-native principles. That substitution is stated, not implied.

**Evidence basis.** This design sits on two audit-only recon maps produced in the same workspace:

- `docs/trace/agent-cli/api-map.md` — callable HTTP/WS surface, transport gates, command/event/refusal tables, and gaps.
- `docs/trace/agent-cli/lifecycle-map.md` — launch/boot/stop reality, PID/state gaps, readiness and port policy, and a snapshot assessment of the in-flight CLI.

Both are read-only recon, not runtime verification. Where this document says "verified", it means verified by reading the cited source, not by running HiveMind.

**Repo reality at design time.** Two candidate CLI surfaces exist, neither complete:

| Surface | State | Scope | Score |
|---|---|---|---|
| `hiverouter-cli` (`backend/crates/hiverouter-tools/src/cli.rs`, artifact `hiverouter-cli.exe`) | shipped in 1.0/1.1 release lineage; 9 commands; no mutating command by design (`cli.rs:16–18`) | read-only diagnostics (`version`, `preflight`, `schema`, `tools.*`, `capabilities.census`, `provider.cost`, `health`, `diagnose`) | **19/28** — partially agent-native for the narrow surface |
| `hivemind-cli` (`tools/hivemind-cli.mjs`, untracked; wrappers `tools/hivemind.cmd\|ps1`; usage doc `docs/AGENT-CLI.md`) | in-flight candidate, changed repeatedly during this seat; not in `tools/release-manifest.json`; requires Node | broad: lifecycle, reads, mutations, approvals, templates, files, watch, skill | **15/28** — broad; latest snapshot aligns the envelope basics; contract gaps remain |
| **target production CLI** | this document | one contract, one owner, full control plane | **28/28** |

---

## 0. Verdict

**Current verdict: partially agent-native.** The repo does not lack a CLI. It lacks **one production CLI contract**. The shipped Rust CLI is a credible agent-native *diagnostic* surface, and the in-flight Node CLI is a credible *field prototype*, but neither is a production HiveMind control plane:

- The Rust CLI cannot start/stop the server, cannot submit/cancel/approve, cannot stream events, has no typed argument schema, no lifecycle identity, and no idempotency beyond one server-side receipt.
- The Node prototype covers most operations and its latest snapshot now emits a top-level `schema`, human data output on stderr, and the shipped `0/2/3/4` exit classes. It still writes human help to stdout, collapses transport/timeout/safety into 2/3, has no lock/instance identity, no idempotency, no resync, no tests, and is not in the release.
- The server side is missing the read routes and instance identity a lifecycle CLI needs: `/api/approvals`, `/api/templates`, `/api/workspace`, `/api/runs/active`; `/health` has no `pid`/`boot_id`/`data_dir`; `/api/*` is not token-authenticated; only `SUBMIT_PROMPT` is receipt-idempotent; no route shuts down the server.
- The two surfaces now **partly share** the `0/2/3/4` classes, but the Node aliases collapse distinct failures (`NOT_RUNNING`/`AUTH`/`TIMEOUT` → 3, `UNSAFE` → 2), and there is still no single owner. The production table in §6 adds semantic 5/6/7/8/9/130; agents keyed on the old Node `1/3/4/5/6` snapshot still break.

**Recommendation.** Build one production CLI — `hivemind` — as a **Rust binary** (extend `hiverouter-cli`, or add a `hivemind` binary beside it), keep `hiverouter-cli` as a deprecated alias for one release, and keep the Node prototype as a **non-shipped behavioral reference** until parity. Do **not** ship the Node prototype as the release artifact unless Node >= 22.12 is explicitly declared a runtime prerequisite, bundled, and the Rust CLI's exit table is changed in the same release. Never ship two CLI contracts.

**Target verdict if the plan lands: agent-native (28/28).** The design is deliberately local-first and CLI-first; MCP is deferred (see §9.5).

---

## 1. Three-audience contract

The contract is one CLI serving three readers simultaneously. The channel split is load-bearing; it is not stylistic.

| Channel | Audience | Rule |
|---|---|---|
| `stdout` | machines / agents | Exactly one JSON envelope per invocation in `json` mode; exactly one JSON object per line in `jsonl` mode; no prose, no spinner, no human table unless `--format table` was explicitly requested. In human mode, **stdout is empty**. |
| `stderr` | humans | Human diagnostics, progress lines, and the human view of errors. Never the machine envelope. In `jsonl` watch mode, stderr carries structured progress/liveness only. |
| exit code | systems / orchestrators | Stable class per failure family; documented in `schema` and `--help`; never one code for everything. |
| environment | trust boundary | Operator/system sets stable configuration (`HIVEROUTER_*`, token file, executable path, workspace root). The agent supplies only runtime parameters. |

### 1.1 Format and TTY rules (P1)

- At startup, detect `stdout.is_terminal()` and `stdin.is_terminal()`.
- Default: **JSON on stdout when stdout is not a TTY**; **human on stderr when stdout is a TTY**.
- Overrides: `--format json`, `--format jsonl`, `--format table`, `--format human`, `--json`, `--human`.
- `--format table` is **explicit-only**; it prints an ASCII table to stdout (no pager, no color when `NO_COLOR`/not a TTY). It never auto-selects.
- `NO_COLOR`, `TERM=dumb`, and `--no-color` disable ANSI. No pager is ever invoked; `PAGER` is ignored.
- `--format human` under a non-TTY is allowed for humans redirecting stderr, but stdout stays empty. The Node prototype's latest snapshot writes human data output to stderr (good), but its `humanHelp` still writes prose to stdout; `--help --json` must return an envelope, not prose.
- All timestamps are UTC ISO-8601 with `Z`; all dates ISO-8601; decimal point `.`; no thousands separators; JSON is emitted with deterministic key order where practical. Internal subprocesses that can be locale-sensitive run with `LC_ALL=C` on Unix; Windows output is UTF-8 and does not depend on the console code page.

### 1.2 Trust is directional (P2)

- CLI arguments are **untrusted** and may be hallucinated or prompt-injected. Validate once at the entry point; never let an argument select an arbitrary executable, browser, token, or data directory without an explicit operator grant.
- `--exe` is a **trusted operator flag**, not an agent convenience. Default resolution is the installed server beside the CLI; if `--exe` is supplied, it is tier 3 and requires `--yes`, and the resolved path/hash is recorded in the envelope and state. Agents should use `HIVEROUTER_SERVER_EXE` set by the operator, not invent a path.
- `--token` is **forbidden**. Session tokens come from the local server (`GET /api/session`) or from `HIVEROUTER_SESSION_TOKEN` / `--token-file` set by the operator. A token in argv is visible in the process list and shell history.
- `--url` accepts only `http(s)://127.0.0.1:<port>`, `http://localhost:<port>`, or `http://[::1]:<port>`; a remote URL is `E_CLI_UNSAFE` (exit 7), not a transport error.
- `--workspace-root` must be an existing absolute directory; the server otherwise falls back to its own cwd (`config.rs:197–201,367–369`), which silently jails file tools to the wrong tree. Refuse before spawn.

### 1.3 Authentication is delegated (P7)

- The provider key belongs to the **server process environment** (or the DPAPI `provider.key` injected by a human launcher). The CLI never reads, decrypts, prints, stores, or forwards the key. It reports `provider_key_present: true|false` only, with the source named (`/diagnose.provider_key_present` or the `deepseek_api_key` health check).
- The CLI obtains the per-boot WS/API session token from `GET /api/session` over loopback and holds it **in memory only**. It never prints it; `session`/`doctor` redact it; errors never include it.
- For orchestrators, the token may be supplied by `HIVEROUTER_SESSION_TOKEN` or `--token-file <path>` (ACL-restricted, mode 0600). The CLI does not run a login flow, browser OAuth, or token refresh; those belong to the operator/system.
- Server-side requirement: `/api/*` (except `/health` and `/api/session`) must require the token. Today only `/ws` does (`ws.rs:382–384`); `/api/*` is guarded only by the Host header (`http.rs:543–564`), and `DELETE /api/agents/profiles/{id}` and `DELETE /api/agents/memory/{key}` are reachable by any loopback client. That is a P0 security gap for a mutating CLI.

### 1.4 No secret output

- Redact `token`, `authorization`, `api_key`, `DEEPSEEK_API_KEY`, `sk-*` patterns, and the literal session token from all envelopes, human lines, logs, state files, and test fixtures.
- `settings` returns only `provider`, `model`, `key_present`, `thinking`, `reasoning_effort`; never key material.
- `provider balance` is sensitive account data, not a credential; still never write it to persistent state or logs.
- `--debug`/`--verbose` do not lift redaction.
- A test must inject a fake key and assert zero occurrences across stdout, stderr, state, and log tail.

### 1.5 Windows-first behavior

The product ships on Windows and agents drive it through PowerShell/cmd. The CLI must be correct under both:

- `--flag=value` and `--flag value` are both accepted for scalar flags; `--` terminates option parsing.
- Prompt text uses `--text-file <path>` or `--text -` (stdin). `--text` is for short, shell-safe strings only. Never build a prompt by concatenating quoted arguments inside a shell wrapper.
- Paths with spaces, Unicode, and `\` are handled as `PathBuf`/`os.PathLike`, not string concatenation.
- `HiveMind.cmd` must `exit /b %ERRORLEVEL%` and must not `pause` on error; an agent must not block on "Press any key". `--pause-on-error` is human-only.
- The launcher must not force a browser open. Headless `start` is the default; `--open`/`open` is an explicit human action.

---

## 2. Interaction-loop coverage

The skill's loop is bootstrap → discovery → planning → execution → interpretation → recovery. Today's coverage:

| Phase | Rust CLI | Node prototype | Target | Gap |
|---|---|---|---|---|
| 0. Bootstrap | key presence only; no session token | fetches `/api/session` internally; no token env/file; inherits env | operator sets key/exe/workspace; CLI fetches token; `HIVEROUTER_SESSION_TOKEN`/`--token-file` for orchestrators | `/api/*` auth; no token delegation; no lock identity |
| 1. Discovery | top/resource help, `schema`, 9 commands | flat help, `commands`, `schema`, 32 commands | progressive `help`, `commands`, `schema`, `doctor`, `completion`, `skill` | no action-level typed help; Node help is prose on stdout; no typed schema |
| 2. Planning | `--dry-run` on health/diagnose/provider.cost | `--dry-run` broadly; latest snapshot checks before `start` writes; WS dry-run is still generic | every mutating command previews the transport payload/process plan with no side effect | no `would_request` contract for WS; verify side-effect freedom with a fingerprint test |
| 3. Execution | HTTP reads only | HTTP + manual WS client; not runtime-verified | one HTTP read client + one WS command/client; bounded waits; correlation filtering | WS client must set `Origin`; Node now does manually; no tests; no resync |
| 4. Interpretation | `hiverouter.cli.v1` envelope with `schema` | latest snapshot has `schema`; `doctor` can still be `ok:false` with no `error`; help is prose | stable envelope, `meta`, `page`, `next`, error catalog | Node doctor error shape; human help; no `page`/`next` |
| 5. Recovery | named `retryable`; no idempotency key | timeout/exit classes; no resync; no idempotency | exit table + `retryable` + `--idempotency-key` + resync policy | only `SUBMIT_PROMPT` server-idempotent; no resync; no lock/stale identity |

**Acceptance for the loop:** an agent can go from `hivemind doctor` (or `start`) to `hivemind commands` → `hivemind schema runs.submit` → `hivemind runs submit --dry-run` → execute → parse `data`/`error` → recover with `runs show`/`watch`/`runs state`, without reading a README and without a second source of truth.

---

## 3. Rubric and principles

### 3.1 14-criterion rubric (current vs target)

Scores are 0–2 per criterion (max 28). "Rust" is the shipped narrow CLI; "Node" is the untracked candidate at its last observed form. Target is the design in this document. The `anc` linter is not installed on this host, so this is a rubric-only review (the skill's fallback), not an `anc` scorecard.

| # | Criterion (principle) | Rust | Node | Target | Evidence / issue |
|---|---|---:|---:|---:|---|
| 1 | Three-audience support (P0) | 2 | 1 | 2 | Rust splits channels correctly for read-only; Node human data is stderr but help is stdout; no lifecycle parity |
| 2 | Stdout contract (P1) | 2 | 1 | 2 | Rust JSON non-TTY and TTY override; Node `--help` prose, `--format table` incomplete, no `page`/`next` |
| 3 | Stderr separation (P1) | 2 | 2 | 2 | Node `printHuman` now writes stderr; failures use stderr; help is the remaining stdout leak |
| 4 | Exit-code semantics (P1/P2) | 1 | 1 | 2 | Rust 0/2/3/4; Node aliases to 0/2/3/4 but collapses transport/timeout/safety into 3/2; no 5/6/7/8/9 |
| 5 | Self-description/help (P3) | 1 | 1 | 2 | Rust top/resource but no action-level flags; Node flat list, prose help; no progressive tree |
| 6 | Schema introspection (P6) | 1 | 1 | 2 | Rust `schema` returns command specs + hand-written response placeholder; Node returns registry args but no typed JSON Schema/output/errors/examples |
| 7 | Dry-run (P3/P4) | 1 | 1 | 2 | Rust only health/diagnose/provider.cost; Node broad; latest snapshot checks `start` dry-run before writes, WS dry-run still generic |
| 8 | Idempotent retries (P1) | 1 | 0 | 2 | Only `SUBMIT_PROMPT` has a server receipt (`store.rs:960–1005`, `execution.rs:1031–1073`); no CLI key; no receipts for spawn/template/rewire/kill/cancel/approval |
| 9 | Non-interactive operation (P0) | 2 | 2 | 2 | Neither prompts; Node has `--yes` for most destructive commands; no `--no-input`; `restart` bypasses `--yes` |
| 10 | Safety tiers (P4) | 1 | 1 | 2 | Rust has tier metadata, no mutating; Node has safety strings/`--yes` but inconsistent (`restart`, `open`), no dispatch enforcement |
| 11 | Boundary validation (P5) | 2 | 1 | 2 | Rust central parse/validation; Node central but unknown flags ignored, integer-only zoom, prompt length in UTF-16 units |
| 12 | Auth delegation (P7) | 1 | 1 | 2 | Rust no token; Node fetches token internally/redacts; `/api/*` unauthenticated; no token env/file |
| 13 | Error recoverability (P1) | 1 | 1 | 2 | Both code/message/how_to_fix/retryable; neither has category/exit/correlation/details; Node `ok:false` can lack `error` |
| 14 | Trust boundary (P2) | 1 | 1 | 2 | Rust args runtime-only but no lifecycle; Node allows arbitrary `--exe`/`--browser`, loopback URL guard only |
| | **Total** | **19** | **15** | **28** | Rust = 17–25 band (partially agent-native); Node below band; target = agent-native |

### 3.2 Seven-principle review

| Principle | Status | Evidence | Issue | Recommendation |
|---|---|---|---|---|
| P0 One CLI, three audiences | partial | Rust is human/agent/system for reads; Node is broad but help is stdout | two surfaces, one owner unresolved | one `hivemind` binary; Node prototype not shipped; one table |
| P1 Structured output is the interface | partial | Rust `hiverouter.cli.v1` with `schema`; latest Node snapshot added `schema` | two envelope shapes; human help still on stdout; no `page`/`next` | keep `schema`, `ok`, `command`, `data\|error`, `meta`; add `page`/`next`; never prose on stdout |
| P2 Trust is directional | partial | loopback URL guard, no key args; Node `--exe`/`--browser` | agent can choose arbitrary executable/browser | operator-only `--exe`; require `--yes`/hash; `--browser` human-only |
| P3 CLI describes itself | partial | `commands`/`schema`; Rust top/resource help | no action-level typed help; Node flat prose help | progressive `help` → resource → action → `schema`; top help < 500 tokens |
| P4 Safety through graduated visibility | partial | tiers/`--yes`; Node exposes UI-only commands and `restart` bypass | destructive actions not consistently gated | tier table + dispatch enforcement; `restart`/`stop` confirm; UI-only hidden |
| P5 Validate at the boundary | partial | central parse; Node ignores unknown flags | invalid inputs reach handlers; JS UTF-16 vs Rust `chars()` | one typed spec; reject unknown flags; Unicode scalar count; test pass/reject |
| P6 Schema is source of truth | partial | Rust command table; Node registry | schema/help/validation drift; no schema version/deprecation | one `CommandSpec` + `ArgSpec`; derive help/schema/completion/skill; `schema_version`/`since`/`deprecated` |
| P7 Auth is delegatable | partial | WS token; `/api/*` unauthenticated; Node internal token | no operator-supplied token; local API writes unauthenticated | token middleware; `HIVEROUTER_SESSION_TOKEN`/`--token-file`; never `--token` |

### 3.3 What the scores mean

- The shipped Rust CLI's 19/28 is **not** a claim that it is production-complete; it scores well on output discipline and poorly on control-plane scope.
- The Node prototype's 15/28 is **not** a dismissal of its coverage; it is a refusal to call a surface agent-native while human help is prose on stdout, the exit table collapses distinct failure classes, and lifecycle/idempotency/resync are absent.
- The target's 28/28 is conditional on the P0 backend changes in §13. Without them, the CLI can be at most a read-only wrapper (roughly 19–21/28).

---

## 4. Command tree

### 4.1 Artifact and naming

- Production binary: `hivemind` (`hivemind.exe` on Windows). It is a Rust binary; it may live in `hiverouter-tools` as a second `[[bin]]` or in a new `hiverouter-cli` crate.
- Deprecated alias: `hiverouter-cli` for one release; it prints a one-line stderr warning in human mode and sets `meta.deprecated:true` in JSON. The server binary `hiverouter` is unchanged.
- Flat aliases for the Node prototype's ergonomics: `hivemind start`, `hivemind submit`, `hivemind runs`, `hivemind watch`, etc. map to the hierarchical commands below. Aliases are documented but not a second schema.
- `HiveMind.cmd` should become `hivemind start --open` (or `hivemind open`), not the old `tools/start-hivemind.ps1` path; `HiveMind-Agent.cmd` stays headless and uses `hivemind`.
- Do not expose `chat-pin`, `viewport`, `recenter`, or `DIFF_*` on the production agent surface. They are UI/client-only or decision-gated (`command.rs:442–447,484–486`); hiding them is a safety-tier decision, not an omission.

### 4.2 Global flags

```
--format json|jsonl|table|human   # auto: json when stdout is not a TTY; human when TTY
--json | --human | --no-color
--url <loopback-base>             # default http://127.0.0.1:8080
--data-dir <dir>                  # default %LOCALAPPDATA%\HiveMind
--workspace-root <dir>            # required for file tools; validated before spawn
--timeout <duration>              # one control operation (default 30s; health 10s)
--wait-timeout <duration>         # --wait-for predicate (default 300s)
--wait-for <predicate>            # e.g. run.created, run.terminal, health.ok, approval.pending
--dry-run                         # no side effects, on every tier 2/3 command
--yes                             # confirm a tier 2/3 action in non-interactive mode
--no-input                        # never prompt; missing confirmation is an error
--idempotency-key <string>        # deterministic correlation for safe retry (see §8.3)
--correlation-id <uuid>           # operator/orchestrator trace id; optional
--limit <n> --cursor <c>          # bounded lists
--max-bytes <n>                   # response/stream cap (default 1 MiB)
--fields <csv> --full             # compact default vs full detail
--stream                          # list output as JSONL (where supported)
--token-file <path>               # operator-managed token; never --token
--verbose | --debug               # redacted diagnostics only
--open                            # human-only for start; opens UI
--fix                             # doctor only; safe repairs
--require-key                     # start/doctor gate; absent key is exit 3
--force                           # stop/restart/unsafe; requires --yes
```

### 4.3 Tree

```
hivemind
├── version
├── commands [--format ...]
├── schema [<command>] [--all]
├── doctor [--fix] [--dry-run] [--online] [--require-key]
├── completion <powershell|bash|zsh|fish>
├── skill [--format markdown|json]
├── mcp serve [--read-only]                 # future, not v1
│
├── server
│   ├── start [--bind|--port] [--data-dir] [--workspace-root] [--exe]
│   │         [--ready-timeout] [--foreground] [--open] [--require-key]
│   ├── status [--watch] [--interval]
│   ├── stop [--timeout] [--force]
│   ├── restart [--timeout] [--drain] [--force]
│   ├── health [--url] [--timeout]
│   ├── diagnose [--url] [--timeout]
│   ├── logs [--lines N] [--stream stdout|stderr|both] [--follow]
│   └── open [--print-url] [--browser <path>] [--no-app]
│
├── runs
│   ├── submit (--text <s> | --text-file <p> | --text -) --project <id>
│   │         [--wait-for run.created|run.terminal] [--wait-timeout]
│   ├── list [--limit] [--cursor] [--status] [--project] [--since]
│   ├── show <run_id>
│   ├── cancel <run_id> [--wait-for run.terminal]
│   ├── revisions <run_id> [--limit] [--cursor]
│   ├── state <run_id> [--revision N | --latest]
│   ├── messages <run_id> [--limit] [--cursor] [--role]
│   └── usage <run_id>
│
├── agents
│   ├── roster <run_id> [--limit] [--cursor]
│   ├── messages <run_id> [--limit] [--cursor]
│   ├── profiles [list | show <id> | import <file> | delete <id>]
│   ├── memory [list [--agent <key>] | clear <agent_key>]
│   ├── spawn <parent_task_id> --role <role>
│   ├── rewire <task_id> --parent <task_id> --expected-rev <n>
│   └── kill <task_id>
│
├── approvals
│   ├── list [--run <run_id>] [--wait-for approval.pending]
│   ├── show <approval_id>
│   ├── approve <approval_id> --nonce <nonce> [--reason <s>]
│   └── deny <approval_id> --nonce <nonce> [--reason <s>]
│
├── templates
│   ├── list [--limit] [--cursor]
│   ├── show <template_id>
│   └── insert <template_id> --project <id>
│
├── workspace
│   ├── info
│   ├── projects [--limit] [--cursor]
│   ├── tree [--path <dir>] [--limit]
│   └── read <path> [--max-bytes]
│
├── settings
│   ├── show
│   ├── models
│   └── balance
│
├── history
│   └── replay <run_id> [--from <rev>] [--to <rev>] [--format jsonl]
│
└── watch [--run <run_id>] [--event <NAME>] [--since-seq N]
          [--max-events N] [--timeout] [--follow] [--format jsonl]
```

### 4.4 Command → backing surface

| Command | Transport / route | Existing? | Notes |
|---|---|---|---|
| `server.start` | process spawn + `/health` | Node prototype only | needs lock/identity/child-exit |
| `server.status` | state + `/health` | Node prototype only | read-only; no state write |
| `server.stop` | process + signal/kill | Node prototype only | no working graceful stop today |
| `server.restart` | stop + start | Node prototype only | must inherit state identity, not caller flags |
| `server.health` | `GET /health` | yes (`http.rs:1153`) | parse body on 503 |
| `server.diagnose` | `GET /diagnose` | yes (`http.rs:597`) | no secrets |
| `server.logs` | local files | Node prototype only | bounded tail, no whole-file read |
| `server.open` | local browser | Node prototype only | human-only; `--print-url` for agents |
| `runs.list` | `GET /api/history/runs` | yes (`http.rs:621`) | server cap 200; add cursor |
| `runs.show` | scan list + `runs state` | partial | no single-run route |
| `runs.submit` | WS `SUBMIT_PROMPT` | yes (`ws.rs:763`) | keyless refusal before run |
| `runs.cancel` | WS `CANCEL_RUN` | yes (`ws.rs:793`) | no receipt |
| `runs.revisions` | `GET .../revisions` | yes (`http.rs:640`) | valid-unknown returns `200 []` |
| `runs.state` | `GET .../state?revision=` | yes (`http.rs:666`) | revision required server-side; CLI defaults latest |
| `runs.messages` | `GET .../messages` | yes (`http.rs:772`) | 1000 hard cap + `truncated` |
| `runs.usage` | `GET /api/usage/runs/{id}` | yes (`http.rs:1031`) | nullable usage |
| `agents.roster` | `GET .../roster` | yes (`http.rs:951`) | unpaginated |
| `agents.messages` | `GET .../messages` | yes (`http.rs:817`) | A2A, distinct from chat |
| `agents.profiles` | `GET/POST/DELETE /api/agents/profiles*` | yes (`http.rs:523–525`) | mutations unauthenticated |
| `agents.memory` | `GET/DELETE /api/agents/memory*` | yes (`http.rs:526–527`) | clear is destructive |
| `agents.spawn` | WS `SPAWN_SUB_HIVE` | yes (`ws.rs:783`) | no receipt; caps |
| `agents.rewire` | WS `REWIRE_TASK` | yes (`ws.rs:650`) | CAS `expected_rev` |
| `agents.kill` | WS `KILL_WORKER` | yes (`ws.rs:800`) | no receipt |
| `approvals.list` | **no HTTP route**; WS hello replay | only via `hello` (`ws.rs:569–598`) | add read route or bounded replay |
| `approvals.show` | same | no | nonce from `APPROVAL_REQUEST` |
| `approvals.approve/deny` | WS `APPROVAL_RESOLVE` | yes (`ws.rs:899`) | nonce single-use |
| `templates.list/show` | **no HTTP route** | no | add `GET /api/templates` |
| `templates.insert` | WS `TEMPLATE_INSERT` | yes (`ws.rs:827`) | needs active run/project |
| `workspace.info` | `/diagnose` + state | partial | add `GET /api/workspace` or fields |
| `workspace.projects` | `GET /api/projects` | yes (`http.rs:1136`) | read-only |
| `workspace.tree/read` | WS `FILE_TREE_LIST`/`FILE_READ` | yes (`ws.rs:879,889`) | jailed |
| `settings.show` | WS `SETTINGS_STATUS` | yes (`ws.rs:807`) | key presence only |
| `settings.models` | `GET /api/provider/models` | yes (`http.rs:1070`) | keyless refusal |
| `settings.balance` | `GET /api/provider/balance` | yes (`http.rs:1093`) | keyless refusal |
| `history.replay` | HTTP revisions + state | yes (composition) | deterministic; no live bus |
| `watch` | WS events | yes (bus) | needs resync/reconnect |
| `doctor` | composition | partial | no fix/identity |
| `commands/schema/version` | local | yes (Rust) | needs typed schema |
| `completion/skill` | local generated | no | add |
| `mcp.serve` | new | no | defer |

### 4.5 Hidden / not exposed

- `DIFF_PROPOSE`, `DIFF_APPLY`, `DIFF_REJECT`: decision-gated, not frozen; server refuses `E_DECISION_REQUIRED` (`command.rs:442–447`). Do not expose.
- `CHAT_PIN`, `VIEWPORT_SET`, `VIEWPORT_RECENTER`: client-only; no server effect except ACK (`ws.rs:757–762`). Do not expose on the agent surface; optionally a hidden `ui` group for parity tests.
- Raw `/api/session` token: never a command output; `session`/`doctor` redact.
- Arbitrary PID kill: `server.stop` only kills the recorded, identity-checked instance; `--force` still prints and confirms identity.
- Provider key set/read: no command; server environment/DPAPI only.
- `reset`/`purge-data`: not in v1; if added, tier 3, `--purge-data --yes`, dry-run lists exact paths, never reachable from `stop`.

---

## 5. JSON envelopes and errors

### 5.1 Envelope identity

Keep the established envelope id **`hiverouter.cli.v1`** for compatibility. The wire contract is `hiverouter.v1` (`schema.rs:20`). Add a semantic CLI version and schema version in `meta`; a new id (`hivemind.cli.v2`) is reserved for a breaking envelope change. This follows P6's "append-mostly" rule and avoids breaking agents cached on the shipped CLI.

Every response carries a top-level `schema`; the Node prototype's latest snapshot added it, but earlier snapshots omitted it (lifecycle-map §9.7). Keep the field mandatory and tested.

### 5.2 Success

```json
{
  "schema": "hiverouter.cli.v1",
  "ok": true,
  "command": "runs.list",
  "data": {
    "runs": [
      {"run_id": "019a...", "project_id": "workspace", "status": "completed",
       "error_code": null, "created_at": "2026-11-08T07:00:00Z"}
    ]
  },
  "page": {"limit": 20, "count": 1, "has_more": false, "next_cursor": null,
           "truncated": false, "bytes": 312},
  "next": ["hivemind runs show 019a..."],
  "meta": {
    "cli_schema": "hiverouter.cli.v1",
    "schema_version": "2.0.0",
    "cli_version": "2.0.0",
    "contract_schema": "hiverouter.v1",
    "server_version": "1.2.0-dev",
    "command": "runs.list",
    "safety": {"tier": 1, "mutating": false, "destructive": false, "requires_yes": false},
    "correlation_id": "019a...",
    "request_id": "019a...",
    "server": {"url": "http://127.0.0.1:8080", "boot_id": "019a...",
               "pid": 1234, "last_seq": 42},
    "latency_ms": 12,
    "cached": false
  }
}
```

Rules:

- `data` is always an object or array; never `null` for a list. An empty list is `[]` with `count: 0`.
- `page` is present on every list/collection response. `has_more` is only true when the server or a trustworthy cursor says so; client-side slicing with a server cap must set `truncated:true` and `truncation_reason:"server_cap"`.
- `next` is optional, copy-pasteable, and never contains secrets.
- `meta.correlation_id` is always present. `meta.request_id` is server-assigned when available, otherwise CLI-assigned and sent as `X-Request-Id`; today the server ignores it, so add echo support.
- `meta.safety` is derived from the same `CommandSpec` as `schema`, not hand-written.
- `meta.server` is present when the command contacted the server; `boot_id` is added by the server change in §13.
- `meta.schema_version` is the CLI's semantic version; `meta.contract_schema` is the server wire contract. An agent compares both before caching.

### 5.3 Failure

```json
{
  "schema": "hiverouter.cli.v1",
  "ok": false,
  "command": "runs.submit",
  "error": {
    "code": "E_KEY_MISSING",
    "category": "refusal",
    "message": "No DeepSeek API key is configured.",
    "how_to_fix": "Set DEEPSEEK_API_KEY in the server environment and restart the server.",
    "retryable": false,
    "exit_code": 3,
    "correlation_id": "019a...",
    "details": {"field": null, "run_id": null, "http_status": null, "server_code": null}
  },
  "meta": { "...": "same shape as success; safety and server are included when known" }
}
```

Rules:

- `error.code` is always present; `E_*` codes are stable identifiers, messages may change (refusal.rs:5–7). CLI-local codes use `E_CLI_*` and are documented as not wire refusals.
- `error.category` is one of `usage|refusal|auth|conflict|transport|timeout|safety|internal|protocol|partial|interrupted`.
- `error.exit_code` duplicates the process exit class so a wrapper that loses the process status can still route.
- `error.details` is structured and bounded; no payload dumps, no token, no key, no approval nonce (nonce is single-use but still not logged).
- A failure never has `ok:true`; `doctor` is the one deliberate exception (it reports a completed diagnosis; see §5.7).

### 5.4 Partial success

```json
{
  "schema": "hiverouter.cli.v1",
  "ok": "partial",
  "command": "agents.kill",
  "data": {
    "succeeded": [{"task_id": "019a...", "state": "killed"}],
    "failed": [
      {"task_id": "019b...", "error": {"code": "E_TASK_TERMINAL", "message": "...",
        "retryable": false, "exit_code": 6}}
    ]
  },
  "page": {"limit": 2, "count": 2, "has_more": false, "next_cursor": null,
           "truncated": false, "bytes": 420},
  "meta": { "...": "..." }
}
```

Use `ok:"partial"` only for real batch commands (`--all` variants, multi-file/profile operations) and exit 8. A single-item command must not collapse a per-item failure into top-level `ok:false` if some items succeeded; conversely, do not invent batch commands just to use this shape.

### 5.5 Error catalog mapping

| Error code | Source | Category | Exit | Retryable | Notes |
|---|---|---|---:|---|
| `E_CLI_USAGE` | CLI | usage | 2 | no | unknown command/flag, missing/invalid value, bad format |
| `E_SCHEMA_INVALID` | CLI parse | usage | 2 | no | validated at boundary; server-origin is 4 |
| `E_CLI_CONFIRMATION_REQUIRED` | CLI | safety | 7 | no | `--yes`/`--no-input` path |
| `E_CLI_HUMAN_ONLY` | CLI | safety | 7 | no | `open` under non-TTY without `--print-url` |
| `E_CLI_UNSAFE` | CLI | safety | 7 | no | non-loopback URL, `..`/unsafe path, arbitrary exe without `--yes` |
| `E_CLI_SERVER_UNREACHABLE` | CLI | transport | 5 | yes | connection refused/timeout before response |
| `E_CLI_TIMEOUT` | CLI | timeout | 5 | yes | per-command / wait-for deadline; `outcome_unknown` for mutations |
| `E_CLI_SERVER_START_FAILED` | CLI | process | 1 | no | child exited during boot; include stderr tail |
| `E_CLI_PORT_IN_USE` | CLI | conflict | 6 | no | non-HiveMind listener or wrong instance |
| `E_CLI_STALE_LOCK` / `E_CLI_PID_REUSED` | CLI | conflict | 6 | no | identity mismatch; never kill |
| `E_CLI_INSTANCE_CONFLICT` | CLI | conflict | 6 | no | same data dir, different port/instance |
| `E_CLI_SCHEMA_VERSION_MISMATCH` | CLI | protocol | 9 | no | CLI/server schema drift; re-discover/upgrade |
| `E_CLI_INTERRUPTED` | CLI | interrupted | 130 | yes | SIGINT/Ctrl+C; final envelope/summary emitted |
| `E_WS_DISCONNECTED` | server/CLI | transport | 5 | yes | mapped from old 3; migration note |
| `E_ORIGIN_REJECTED` / `E_TOKEN_INVALID` | server | auth | 3 | no | token/Origin/Host; never echo token |
| `E_KEY_MISSING` | server | refusal | 3 | no | named refusal; no run created |
| `E_PROVIDER_AUTH` | server | auth/refusal | 3 | no | key rejected; never retry |
| `E_PROVIDER_RATE_LIMIT` / `E_LLM_RATE_LIMIT` | server | refusal | 3 | yes | backoff |
| `E_PROVIDER_TIMEOUT` | server | refusal | 3 | yes | provider timeout |
| `E_RUN_ACTIVE` / `E_RUN_TERMINAL` | server | conflict | 6 | no | state conflict; read before retry |
| `E_TASK_TERMINAL` / `E_KILL_DENIED` | server | conflict | 6 | no | state conflict |
| `E_REWIRE_IN_FLIGHT` / `E_RESYNC_REQUIRED` / `E_DB_BUSY` | server | conflict | 6 | yes | retry after read/backoff |
| `E_APPROVAL_REQUIRED` / `E_DECISION_REQUIRED` | server | safety | 7 | no | pending approval/decision-gated |
| `E_APPROVAL_REPLAY` / `E_APPROVAL_TIMEOUT` | server | refusal | 3 | no/yes | timeout retryable; replay not |
| `E_APPROVAL_NOT_FOUND` / `E_APPROVAL_DECISION_INVALID` | server | refusal | 3 | no | read pending approvals |
| `E_PROMPT_EMPTY` / `E_PROMPT_TOO_LONG` | server | usage/refusal | 3 | no | boundary also validates; server remains authority |
| `E_BUDGET_EXCEEDED` / `E_CALL_LIMIT` / `E_TOKEN_BUDGET_EXCEEDED` | server | refusal | 3 | no | start a new run |
| `E_PATH_OUTSIDE_JAIL` / `E_PATH_NOT_FOUND` / `E_FILE_TOO_LARGE` | server | refusal | 3 | no | workspace-relative |
| `E_FILE_TREE_UNAVAILABLE` | server | refusal | 3 | no | `--workspace-root` invalid |
| `E_TOOL_*` / `E_JAIL_ESCAPE` | server | refusal | 3 | mixed | preserve server `retryable` |
| `E_TEMPLATE_*` | server | refusal | 3 | no | template/licence/ref |
| `E_NODE_MISSING` / `E_TOOLCHAIN_MSRV` | server | refusal | 3 | no | environment |
| `E_INTERNAL_ERROR` | server/CLI | internal | 4 | yes | invariant failure; include correlation id |
| HTTP 404/405/421/SPA shapes | server | protocol/refusal | 4 or 3 | no | tolerate all body variants (api-map §2) |

Full wire code list remains `Refusal::ALL` (`refusal.rs:73–282`, count 64 in the current census). The CLI must map by code, never by message. `how_to_fix` is displayed but not parsed.

### 5.6 Pagination and truncation

- Every list command accepts `--limit` (default 20–50, max 200), `--cursor` (opaque), `--fields`, `--full`, `--max-bytes`.
- Response `page`: `{limit, count, has_more, next_cursor, truncated, truncation_reason, bytes}`.
- `has_more` is authoritative only when the server returns a cursor or a total. Until the server supports cursors, commands over hard-capped routes must report `truncated:true` and `truncation_reason:"server_cap"`, and must not claim `has_more:false` when the cap was hit.
- Stable sort keys: runs by `created_at DESC, run_id`; messages by `created_at ASC, id`; revisions by `revision ASC`; roster by `spawn_seq, task_id`; memory by `agent_key, key`; projects by `created_at, id`; approvals by `expires_at, approval_id`.
- `--max-bytes` truncates the `data` payload, not the envelope; it sets `truncated:true`, `truncation_reason:"max_bytes"`, and preserves IDs/status/`next_cursor`. It never silently drops an error.
- Server gaps: `/api/history/runs` hard-caps 200 (`http.rs:622`); messages hard-cap 1000 (`http.rs:782`); roster/revisions/projects/profiles/memory are unpaginated. Add cursor support (P1) rather than hiding it client-side.

### 5.7 Schema, commands, and doctor envelopes

`schema <command>` returns a typed contract, not a registry dump:

```json
{
  "schema": "hiverouter.cli.v1",
  "ok": true,
  "command": "schema",
  "data": {
    "command": "runs.submit",
    "since": "2.0.0",
    "deprecated": false,
    "replaced_by": null,
    "safety": {"tier": 2, "mutating": true, "destructive": false,
               "requires_yes": true, "dry_run": true, "idempotent": true},
    "params": {
      "text": {"type": "string", "minLength": 1, "maxLength": 8000, "required": false},
      "text_file": {"type": "string", "format": "path", "required": false},
      "project": {"type": "string", "required": true, "default": "workspace"},
      "wait_for": {"type": "string", "enum": ["run.created", "run.terminal", "none"],
                   "default": "run.created"},
      "wait_timeout": {"type": "string", "default": "300s"}
    },
    "request": {"transport": "ws", "name": "SUBMIT_PROMPT",
                "payload": {"text": "text", "project_id": "project"}},
    "response": {"data_schema": "runs.submit.v1"},
    "errors": ["E_PROMPT_EMPTY", "E_PROMPT_TOO_LONG", "E_RUN_ACTIVE", "E_KEY_MISSING"],
    "examples": ["hivemind runs submit --text-file prompt.txt --project workspace --yes"],
    "meta": {"schema_version": "2.0.0", "contract_schema": "hiverouter.v1"}
  }
}
```

`commands` returns a compact list: `{name, summary, tier, mutating, destructive, transport, since}`. It does **not** dump full schemas; that is `schema`. Top-level `--help` must stay under ~500 tokens and never dump the command table in JSON; agents that loop must not pay an eager schema tax (P3, `hybrid-mcp-cli.md`).

`doctor` reports a completed diagnosis, not a refusal envelope:

```json
{
  "schema": "hiverouter.cli.v1",
  "ok": true,
  "command": "doctor",
  "data": {
    "ready": false,
    "status": "blocked",
    "checks": [
      {"name": "cli", "ok": true, "detail": "hivemind 2.0.0"},
      {"name": "server", "ok": true, "detail": "http://127.0.0.1:8080"},
      {"name": "instance", "ok": true, "detail": "boot_id ... data_dir ..."},
      {"name": "workspace", "ok": true, "detail": "C:\\work"},
      {"name": "provider_key", "ok": false, "code": "E_KEY_MISSING",
       "detail": "absent (server environment)"},
      {"name": "lock", "ok": true, "detail": "managed pid 1234"}
    ],
    "blocking": ["E_KEY_MISSING"],
    "warnings": [],
    "fixes": []
  },
  "meta": { "...": "..." }
}
```

If `doctor` itself cannot run (unreadable state, invalid options), it returns `ok:false` with an error. If checks ran and are blocked, exit 3 and keep `ok:true`; `data.ready` is the routing field. `doctor --fix` returns `ok:"partial"` if some repairs fail, with `fixes[]` containing `{action, status, detail}`; it never touches the key, DB, runs, or approvals.

### 5.8 JSONL event stream

`watch --format jsonl` (or non-TTY default) writes one JSON object per line to stdout; no prose; final summary line:

```json
{"schema":"hiverouter.event.v1","type":"EVENT","seq":42,"name":"RUN_STATUS",
 "run_id":"019a...","task_id":null,"correlation_id":"019a...",
 "ts":"2026-11-08T07:00:00Z","payload":{"run_id":"019a...","status":"completed"}}
...
{"schema":"hiverouter.cli.v1","ok":true,"command":"watch",
 "summary":{"events":12,"last_seq":54,"resyncs":0,"timed_out":false,
            "truncated":false,"interrupted":false}}
```

- `--event NAME` filters; `--run` filters by `run_id`; `--since-seq` starts replay; `--max-events` bounds; `--timeout` bounds wall time.
- On `RESYNC_REQUIRED`, emit a `resync` record, fetch the latest snapshot, emit it, and continue from `last_seq`; never skip silently. On repeated failure, exit 5 with `E_CLI_RESYNC_EXHAUSTED`.
- On SIGINT, flush the final summary with `ok:false`, `error.code:"E_CLI_INTERRUPTED"`, `interrupted:true`, exit 130. No truncated line.
- Human mode writes readable lines to stderr; stdout stays empty.

### 5.9 Correlation IDs

- Every invocation generates a UUIDv7 `correlation_id`, exposed in `meta` and `error`.
- `--correlation-id` overrides it; `--idempotency-key` derives a deterministic UUIDv5 (namespace = CLI/schema id) so retries reuse the same correlation.
- WS COMMAND envelopes carry `correlation_id`; ACK/ERROR/results share it (`envelope.rs:98–103`). Filter by `correlation_id` and, for run-scoped results, `run_id`; the bus broadcasts every client's ACK/ERROR (`bus.rs:120`, api-map §3).
- HTTP requests send `X-Request-Id`; the server should echo it in `meta.request_id`/logs (additive).
- `meta.server.boot_id` distinguishes instances; `last_seq` anchors the event watermark.

### 5.10 Command timeouts

- `--timeout` default 30s for control operations, 10s for `health`/`diagnose`, 300s for `--wait-for run.terminal` unless `--wait-timeout` overrides.
- On timeout: exit 5, `E_CLI_TIMEOUT`, `retryable:true`, `details:{scope,seconds,correlation_id}`.
- **Mutating timeout**: set `details.outcome_unknown:true`; do not auto-retry. Tell the agent to read `runs show`/`approvals list` first; retry only with the same `--idempotency-key`.
- `--wait-for` failure is `E_CLI_WAIT_TIMEOUT` (exit 5) with `details.last_state`, not a generic refusal.

---

## 6. Exit-code table

The production CLI must ship **one** table. This design preserves the shipped Rust classes `0/2/3/4` and adds semantic classes; the Node prototype's latest snapshot already aliases to `0/2/3/4`, but it collapses `NOT_RUNNING`/`AUTH`/`TIMEOUT` into 3 and `UNSAFE` into 2. Add 5/6/7/8/9/130 in the same release; never ship two tables (lifecycle-map §9.7).

| Code | Class | Meaning | Typical `error.code` | Retryable |
|---:|---|---|---|---|
| 0 | success | requested end state holds; also idempotent no-ops (`start` already running, `stop` already stopped, `status` not running) and `doctor` ready | — | n/a |
| 1 | process failure | server child exited/spawn failed; not a named refusal | `E_CLI_SERVER_START_FAILED` | no |
| 2 | usage | CLI parse/validation, missing required, bad enum/format, unsafe URL | `E_CLI_USAGE`, `E_SCHEMA_INVALID` (CLI parse) | no |
| 3 | named refusal | server/domain refusal: key, auth, approval, jail, provider, template, prompt, budget | `E_KEY_MISSING`, `E_PROVIDER_AUTH`, `E_APPROVAL_REPLAY`, ... | mixed (`retryable` from server) |
| 4 | internal | CLI bug, unexpected server protocol shape, invariant failure | `E_INTERNAL_ERROR`, `E_PROTOCOL` | yes |
| 5 | transport/timeout | server unreachable, WS dropped, deadline, resync exhausted | `E_CLI_SERVER_UNREACHABLE`, `E_CLI_TIMEOUT`, `E_WS_DISCONNECTED` | yes |
| 6 | conflict/state | port/lock/instance conflict, stale PID, run/task terminal, rewire in flight, DB busy | `E_CLI_PORT_IN_USE`, `E_CLI_STALE_LOCK`, `E_RUN_TERMINAL`, `E_REWIRE_IN_FLIGHT` | mixed |
| 7 | safety | confirmation missing, human-only action, approval/decision required, unsafe path/exe | `E_CLI_CONFIRMATION_REQUIRED`, `E_CLI_HUMAN_ONLY`, `E_APPROVAL_REQUIRED` | no |
| 8 | partial | some batch items succeeded, some failed | per-item errors | per-item |
| 9 | protocol/schema | CLI/server schema drift, incompatible envelope/command | `E_CLI_SCHEMA_VERSION_MISMATCH` | no |
| 130 | interrupted | SIGINT/Ctrl+C; final envelope/summary emitted | `E_CLI_INTERRUPTED` | yes |

**Server exit codes** are separate: `hiverouter` currently returns 0 for help/version and 1 for every boot failure (`main.rs:18–24,44–50,60–71,72–82,87–106,136–145`). A supervisor cannot classify port/DB/config from 1. P2 recommendation: add distinct server codes (e.g. 10 bind, 11 DB, 12 config, 13 recovery) or a machine-readable boot error file; the CLI must not infer from server exit 1 alone—use the child stderr tail and `/health` checks.

**Compatibility and migration:**

- `hiverouter-cli` 1.x: `E_WS_DISCONNECTED` was exit 3; the new CLI maps it to 5. This is the only breaking remap. Publish it in the release notes and in `schema`; optionally honor `HIVEROUTER_CLI_LEGACY_EXIT=1` for one release, removed at 3.0.
- Node prototype: the latest snapshot already aliases to `0/2/3/4`; keep those, then add `NOT_RUNNING/TIMEOUT → 5`, `UNSAFE → 7`, conflict → 6, partial → 8, protocol → 9, interrupted → 130. Do all of this in the same release that removes the prototype as a shipped surface.
- `retryable` is the server's field when present; CLI-local failure classes define it. An agent must use both the code and the flag, not the exit code alone.
- Exit codes are documented in `schema`, `commands`, and `--help`; a test pins every class.

---

## 7. Safety matrix

### 7.1 Tiers

| Tier | Name | Exposure | Examples |
|---:|---|---|---|
| 0 | read-local | fully discoverable | `version`, `commands`, `schema`, `doctor` local checks, `completion`, `skill` |
| 1 | read-server | fully discoverable | `status`, `health`, `diagnose`, `runs list/show/revisions/state/messages/usage`, `agents roster/messages/profiles/memory list`, `approvals list/show`, `templates list/show`, `workspace info/projects/tree/read`, `settings show/models/balance`, `history replay`, `watch` |
| 2 | mutating | discoverable; help carries warning; `--dry-run` required | `runs submit`, `agents spawn/rewire`, `templates insert`, `workspace` writes (none), `profiles import/delete`, `memory clear`, `server start` |
| 3 | destructive / safety-critical | hidden from generated skills or gated separately; `--yes` + explicit target; no batch default | `runs cancel`, `agents kill`, `approvals approve/deny`, `server stop/restart`, `doctor --fix`, `--exe` selection, `--force` |

Tiers are prompt-side defense only. The runtime sandbox (filesystem, network, process) is the second layer; destructive commands must fail closed inside it. Approval fatigue degrades visibility-only defenses, so the CLI must **also** require the explicit semantics below.

### 7.2 Matrix

Legend: `Y` = required; `—` = not applicable; `opt` = optional via `--dry-run`; `read` = no mutation. "Idem" is per §8.3, not a claim of blanket retry safety.

| Command | Tier | Mutates | Destructive | `--yes` | `--dry-run` | Idem | Approval / key |
|---|---:|---|---|---|---|---|---|
| `version`, `commands`, `schema`, `completion`, `skill` | 0 | no | no | — | — | n/a | none |
| `doctor` | 0 | no | no | — | — | n/a | key presence only |
| `doctor --fix` | 3 | local | no (safe repairs only) | Y | Y | yes | never key/DB/runs |
| `server start` | 2 | process | no (spawns) | no | Y | yes (lock/health) | key inherited; `--require-key` optional |
| `server status` | 1 | no | no | — | — | n/a | key presence only |
| `server stop` | 3 | process | Y (interrupts runs) | Y | Y | yes (already stopped) | none |
| `server restart` | 3 | process | Y | Y | Y | no (stop+start) | `--drain`/`--force` |
| `server health` / `diagnose` | 1 | no | no | — | opt | n/a | key presence only |
| `server logs` | 1 | no | no | — | — | n/a | redacted |
| `server open` | 3 | process | no | Y (or human TTY) | Y | n/a | human-only |
| `runs submit` | 2 | yes | no (spends money) | Y (non-TTY) | Y | **yes** (server receipt) | key absence → `E_KEY_MISSING` before run |
| `runs list/show/revisions/state/messages/usage` | 1 | no | no | — | — | n/a | none |
| `runs cancel` | 3 | yes | Y (interrupts) | Y | Y | partial (terminal read) | none |
| `agents roster/messages/profiles list/memory list` | 1 | no | no | — | — | n/a | none |
| `agents profiles import` | 2 | yes | no | Y (non-TTY) | Y | yes (upsert) | none |
| `agents profiles delete` | 3 | yes | Y | Y | Y | yes (absent = no-op) | none |
| `agents memory clear` | 3 | yes | Y | Y | Y | yes (absent = no-op) | none |
| `agents spawn` | 2 | yes | no | Y | Y | **no** until receipt | caps; key not needed |
| `agents rewire` | 2 | yes | no | Y | Y | partial (CAS) | `expected_rev` |
| `agents kill` | 3 | yes | Y | Y | Y | partial (terminal read) | none |
| `approvals list/show` | 1 | no | no | — | — | n/a | nonce returned for live approvals |
| `approvals approve` | 3 | yes | Y (grants side effect) | Y | Y | partial (nonce replay) | nonce + authority |
| `approvals deny` | 3 | yes | Y | Y | Y | partial (nonce replay) | nonce + authority |
| `templates list/show` | 1 | no | no | — | — | n/a | licence info |
| `templates insert` | 2 | yes | no | Y | Y | **no** until receipt | active run/project |
| `workspace info/projects/tree/read` | 1 | no | no | — | — | n/a | jail |
| `settings show/models/balance` | 1 | no | no | — | — | n/a | key presence; no key error |
| `history replay` | 1 | no | no | — | — | n/a | none |
| `watch` | 1 | no | no | — | — | n/a | none |

**Dispatch enforcement.** The tier/`--yes`/`--dry-run` checks live in one middleware around the command handler, derived from `CommandSpec`. A command cannot "forget" its own safety check; the Node prototype's per-handler `requireYes` is insufficient because `restart` calls `stopServer({yes:true})` internally and bypasses the check.

### 7.3 Approval authority

- The local server has a **single operator** and no user identity. Approval authority is therefore the operator's possession of the single-use `nonce` plus an explicit `--yes`. The CLI must not invent an identity or an "approve all" mode.
- `approvals approve` requires `--approval-id`, `--nonce`, and `--yes` in non-interactive mode. `--reason` is optional. `approvals deny` has the same shape. There is no `--all`, no `--auto`, and no approval inferred from `runs submit --yes`.
- The nonce comes only from a live `APPROVAL_REQUEST` frame (`event.rs:488–493`; replay includes it at `tools_bridge.rs:502–517`). A request with no nonce cannot be resolved from the CLI; say so, do not fabricate one.
- TTY confirmation shows tool, run/task, `args_sha256`, bounded `args_preview`, expiry, and the exact decision; it never prints the nonce to the human if it can avoid it (the human already has it from the event; the CLI may show a short fingerprint).
- `approve` is the dangerous direction: require `--yes` even in a TTY when stdin is not a TTY, and never auto-approve on timeout (`E_APPROVAL_TIMEOUT`). `deny` is still gated but is the safe default.
- A replay after a decision returns `E_APPROVAL_REPLAY`; the CLI may read `approvals show` and report `already_resolved:true` only when the decision matches, never as a fake success for a different decision.

### 7.4 Provider-key absence is a named refusal

- The server is the authority: `SUBMIT_PROMPT` checks the key before `create_run` (`ws.rs:763–773`), so a keyless submit **creates no run** and returns `E_KEY_MISSING`.
- The CLI must not treat that as a server defect, a usage error, or a generic HTTP 503. It is exit 3, `category:"refusal"`, `retryable:false`, `how_to_fix` = "Set DEEPSEEK_API_KEY in the server environment and restart."
- `doctor` reports `provider_key_present:false` and `status:"blocked"` only when `--require-key` is set; by default a keyless server is `ready:false` with a warning, because the server itself can run and serve reads without a key.
- A keyless server with the provider enabled boots `degraded` with HTTP 200 (`http.rs:1167–1202`); a CLI that treats non-200 as "not ready" will misclassify `failing`; parse the body on 503 and the `status` string on 200.
- `start --require-key` is the opt-in gate; it refuses before spawn only when it can prove the key is absent (server env or DPAPI), and it never decrypts/prints the key itself.

### 7.5 No secret output

The CLI's redaction layer applies to `--verbose`, errors, state files, logs, and crash tails. The literal session token is never emitted. `session` returns `{schema,last_seq,ws,token:"[redacted]"}`; `doctor` reports `provider_key_present` only; `settings` never includes the key. A fake-key test scans stdout/stderr/state/logs.

### 7.6 Dry-run and confirmation semantics

- `--dry-run` is required on every tier 2/3 command. It prints the exact transport request/process plan (WS name+payload, HTTP method+path+body, child argv+env allowlist) and `dry_run:true`; it performs **no** state write, spawn, health probe with side effects, or server mutation.
- `--no-input` (default when stdin is not a TTY) means "never prompt". A missing `--yes` yields `E_CLI_CONFIRMATION_REQUIRED` (exit 7) with a copy-pasteable re-run; the process never blocks.
- `--yes` is required for tier 2/3 in non-interactive mode. In a human TTY, the CLI may prompt once with a summary; `--yes` bypasses. `--yes` does not bypass `--nonce`, `--approval-id`, `--expected-rev`, or `--require-key`.
- `--force` is an escalation, not a confirmation. It requires `--yes`, prints the exact PID/path/identity it will affect, and refuses if identity checks fail.

---

## 8. Lifecycle, recovery, and retry

### 8.1 Lifecycle state and identity

There is no service registration, installer, supervisor, PID file, lock file, or instance registry in product code today (lifecycle-map §6). Both the `.cmd` path and the launcher keep the server handle only in memory; the printed PID is console text. A production CLI must persist an identity record.

**Per-data-directory files:**

- `<data-dir>/hivemind.lock` — CLI-owned exclusive lock + instance identity. JSON, atomic write (temp + fsync + rename), mode 0600, Windows ACL current user.
- `<data-dir>/server.json` — server-written boot manifest (or the same fields in the lock after readiness). It allows `status` to identify an unmanaged server and lets `doctor` verify the instance without guessing.
- `<data-dir>/agent-cli-state.json` — **not** a lock; legacy prototype state. Migrate to `hivemind.lock`; do not keep two.

**Lock schema:**

```json
{
  "schema": "hivemind.lock.v1",
  "state": "starting|ready|stopping",
  "pid": 1234,
  "pid_created_at": "2026-11-08T07:00:00Z",
  "boot_id": "019a...",
  "exe": {"path": "C:\\...\\hiverouter.exe", "size": 13253120,
          "mtime": "2026-11-07T23:43:00Z", "sha256": "..."},
  "bind": "127.0.0.1:8080",
  "data_dir": "C:\\Users\\...\\HiveMind",
  "workspace_root": "C:\\work",
  "server_version": "1.2.0-dev",
  "contract_schema": "hiverouter.v1",
  "db_schema_version": 11,
  "provider_key_present": false,
  "started_at": "2026-11-08T07:00:00Z",
  "logs": {"stdout": "...", "stderr": "..."}
}
```

**Server additions needed (P0):** `/health` must expose `pid`, `boot_id`, `data_dir`, `workspace_root`, and a build id (or `server.json` must be authoritative). Today `/health` has none of these (`http.rs:456–476`), so an unmanaged server's PID cannot be known from HTTP; `status` must honestly report `pid:null` and `managed:false` until the addition lands.

**Identity check:** PID alive **and** process creation time matches `pid_created_at` **and** image path/hash matches **and** `/health.boot_id`/`database.path` matches the lock. Any mismatch is a stale/foreign instance; never kill it. PID reuse is a real risk on Windows; `processAlive(pid)` alone is insufficient.

### 8.2 Start / status / stop / restart

**`server start`** is idempotent:

1. Validate loopback bind (refuse `0.0.0.0`, port 0 with a clear reason), absolute existing `--data-dir`, absolute existing `--workspace-root`, server exe (installed beside CLI or operator `--exe`; do not search `backend/target` first like the prototype), and port.
2. Acquire the data-dir lock exclusively. If held by a live, identity-matching instance: return `already_running:true, managed:true`, exit 0, no spawn. If held by a live but different instance: `E_CLI_INSTANCE_CONFLICT` (6). If held by a dead/stale record: clean it only under `doctor --fix` or `start --force --yes`.
3. Probe `/health` with a 2s timeout and validate `schema == "hiverouter.v1"` and `database.path == <data-dir>/hiverouter.db`. A healthy non-HiveMind listener is `E_CLI_PORT_IN_USE` (6), never "already running".
4. Spawn detached, `windowsHide`, no inherited stdio; stdout/stderr to append-only files; cwd = workspace root; args `--bind --data-dir --workspace-root`; env = parent env + optional DPAPI key (if the CLI implements it).
5. Write `state:"starting"` with pid + creation time + deadline **before** waiting. Poll every 250ms, deadline 30s (`--ready-timeout`), HTTP probe timeout 2s; abort immediately on child exit with the last 20 stderr lines. A `degraded` 200 is ready with a warning; a 503 `failing` is not ready.
6. On ready, rewrite `state:"ready"` atomically with full identity. On failure, clear only the state for the child it spawned; kill only that child; report the reason.
7. Envelope: `{started|already_running, managed, pid, boot_id, url, data_dir, workspace_root, health, provider_key_present, logs}`.

**`server status`** is read-only; it writes no state and spawns nothing. Fields: `{running, managed, health|error, pid, url, data_dir, workspace_root, server_version, contract_schema, db_schema_version, db_path, uptime_secs, provider_key_present, logs, stale_state, pid_reused}`. If the server is down, `ok:true, data.running:false`, exit 0 (a state, not an error); `--strict` exits 5. If the server is unmanaged, `pid:null, managed:false` and explain why.

**`server stop`** is idempotent and identity-checked:

1. No `--yes` ceremony for the CLI's own recorded instance is defensible (lifecycle-map §9.5), but this design requires `--yes` for any stop that could interrupt an active run or kill an unmanaged PID. `stop --dry-run` previews.
2. Read lock; absent + `/health` unreachable → `already_stopped:true`, exit 0. Absent + healthy → `E_CLI_NOT_MANAGED` (6); `--force --yes --pid <exact>` may stop it after identity print.
3. Verify PID + creation time + image + boot_id + DB path. Mismatch → stale cleanup only; never kill.
4. Graceful stop: send the platform signal the server actually handles (see §8.4); wait `--timeout` (default 10s) for process exit and `/health` unreachable.
5. Timeout: with `--force --yes`, `taskkill /F /PID` (plus `/T` only if intentionally reaping tool children) after printing identity; report `graceful:false, escalated:true`. Without `--force`, exit 5/6 `E_CLI_STOP_TIMEOUT`; do not silently force.
6. Delete lock only after process exit and health down. Never delete data.

**`server restart`** is `stop` + `start` with the **recorded identity** (port/data-dir/workspace/exe), not the caller's defaults. If a run is active, refuse with `active_runs:[...]` and `destructive:true`; `--force --yes` restarts, `--drain` waits for terminal. It must not silently pass `yes:true` like the prototype.

**Port policy:** one default. The server default is `127.0.0.1:8080` (`config.rs:10`); the `.cmd`/launcher use `8099` (`start-hivemind.ps1:4`, `HiveMindLauncher.cs:16`). This design chooses **8080** for the CLI and all launchers, with explicit `--port 8099` for anyone who needs the old behavior; or choose 8099 everywhere and pass `--bind 127.0.0.1:8099` to the server. Do not ship the current split. Port 0 is test-only: the server's Origin allowlist is empty for port 0 (`config.rs:205–208`), so product start refuses it.

### 8.3 Idempotency and retry

The contract claims every mutating command is idempotent by `correlation_id` for 24h (`docs/contracts/UI-COMMANDS-v1.md:24`), but the implementation only does this for `SUBMIT_PROMPT`: `command_receipts` stores `correlation_id → run_id` with a 24h check (`store.rs:960–1005`), and `submit_prompt` consults/records it (`execution.rs:1031–1073`). There are no receipts for spawn, rewire, kill, cancel, template insert, or approval.

| Command | Current server behavior | CLI retry policy | Required change |
|---|---|---|---|
| `runs submit` | idempotent by correlation (24h) | safe with `--idempotency-key`; reuse deterministic UUIDv5 | none for this command |
| `runs cancel` | second cancel returns `E_RUN_TERMINAL`; no receipt | read `runs show`; if terminal, report `already_terminal`; do not blindly retry | generalized receipt (P1) |
| `agents kill` | second kill returns `E_TASK_TERMINAL`/`E_KILL_DENIED` | read roster; if terminal, `already_terminal`; no blind retry | receipt |
| `agents rewire` | CAS `expected_rev`; retry after success is stale/resync | re-read roster; retry only with fresh `expected_rev` | receipt or CAS result cache |
| `approvals approve/deny` | nonce single-use; replay `E_APPROVAL_REPLAY` | read `approvals show`; same decision → `already_resolved`; different → error | receipt keyed by approval id |
| `agents spawn` | no dedupe; retry duplicates tasks | **do not retry**; require operator/new request | receipt (P0 for batch automation) |
| `templates insert` | no dedupe; retry duplicates tasks | **do not retry**; require active run + receipt | receipt |
| `profiles import` | upsert by id | safe (idempotent) | none |
| `profiles delete` / `memory clear` | delete absent = no-op | safe | none |
| `server start/stop` | no server receipt; CLI lock/health | idempotent by state/health | lock + identity (P0) |
| `doctor --fix` | safe repairs | idempotent | none |

**Recommendation (P0/P1):** generalize `command_receipts` to `(correlation_id, command_name, result_json, created_at)` with the same 24h window and a unique `(correlation_id, command_name)`. The WS dispatcher records a receipt for every mutating command before publishing ACK; a retry returns the stored ACK/result. Until that lands, the CLI must not claim blanket idempotency and must expose `idempotent:false` in `schema`/`meta.safety` for commands without it.

`--idempotency-key` semantics:

- `runs submit`: `correlation_id = UUIDv5(namespace, key)`; server receipt makes retry return the original run.
- Other mutating commands: the key is a trace/coordination id; the CLI refuses `--idempotency-key` on commands that are not server-idempotent unless `--allow-nonidempotent` is set, with a warning in `meta`.
- Read commands ignore it.

**Retry rules:** automatic retry is allowed only for reads and transport failures (exit 5), with bounded exponential backoff (250ms → 5s, max 3 attempts). Mutating commands never auto-retry; a timeout sets `outcome_unknown:true`. A retry after a mutating failure requires the same `--idempotency-key` and a read-back first. `retryable:false` is terminal.

### 8.4 Signals and graceful shutdown

The server handles only `tokio::signal::ctrl_c()` (`main.rs:178–184`); it does not handle `CTRL_BREAK`, `CTRL_CLOSE`, `CTRL_SHUTDOWN`, `SIGTERM`, or an HTTP shutdown route. For a detached, `windowsHide` child, `taskkill` without `/F` was measured to report success and do nothing (lifecycle-map §7). Therefore "graceful stop" is **not currently available**; the CLI must not claim it.

Recommended order:

1. **P0 (truthful now):** stop reports `graceful:false` when it must escalate, and performs one-command escalation after the grace window. Do not require a second `stop --force` invocation. The server's boot recovery marks interrupted runs (`recovery.rs:85–191`), so a forced stop is safe but loses unflushed audit lines.
2. **P1 (small server change):** install `tokio::signal::windows::ctrl_break()` and `SIGTERM` in the shutdown future; spawn the server with `CREATE_NEW_PROCESS_GROUP`; `stop` attaches to its console and sends `CTRL_BREAK_EVENT`, or uses an authenticated `POST /api/admin/shutdown` (token + Host/Origin) if remote supervisors need it. Test with a real server and assert audit flush + exit 0.
3. `stop` reports `{stopped, graceful, escalated, exit_code}`; `restart` inherits.
4. CLI signal handling: `watch`/`--follow` flushes the final summary and exits 130; `start --foreground` proxies signals to the child; the CLI itself never leaves a half-written JSON line.

### 8.5 Reconnect and resync

- `watch` connects, sends `{"type":"hello","last_seq":N}` within 10s, reads `SERVER_HELLO`, then replays/streams. The server's replay buffer is 256 events and fan-out 1024 (`bus.rs:18–20`); a lagging session gets `RESYNC_REQUIRED` and the server closes (`ws.rs:451–461,539–562`).
- On `RESYNC_REQUIRED`: emit a `resync` record, fetch `runs state --latest` (HTTP) and emit a snapshot record, then reconnect with `last_seq` from the hello; do not silently skip events. `--resync auto|fail` (default auto); max 3 attempts; then `E_CLI_RESYNC_EXHAUSTED` (5).
- Reconnect backoff 250ms → 5s; dedupe by `seq`; drop duplicates; a gap without `RESYNC_REQUIRED` is a protocol error (exit 9).
- For one-shot mutations, `wsCommand` waits for `ACK`/`ERROR` matching `correlation_id`; for results (`RUN_CREATED`, `RUN_STATUS`, `APPROVAL_RESOLVED`, `HISTORY_SNAPSHOT`, `SETTINGS_STATUS_RESULT`, `FILE_*`), filter by `correlation_id` and `run_id` where present. The bus broadcasts to all sessions, so name-only matching is a bug (Node prototype `wsCommand` currently matches by name; fix).
- `runs submit --wait-for run.created` must use the same connection that sent the command, or read `runs list`/`show` after ACK; opening a second WS after ACK can miss the run-created event (Node prototype `run` does this).

### 8.6 Timeout and `--wait-for`

`--wait-for` is a predicate over observed state, not a sleep:

- `run.created`: ACK + `RUN_CREATED` (or a `runs show` fallback).
- `run.terminal`: `RUN_STATUS` with `completed|failed|cancelled|interrupted`, or `runs show` terminal.
- `health.ok`: `/health.status` in `ok|degraded` and schema match.
- `approval.pending`: a live `APPROVAL_REQUEST` with `approval_id`/nonce.
- `seq>=N`: event sequence watermark.
- `event=NAME`: first matching event.
- `status=<value>`: run/agent state.

`--wait-timeout` default 300s; on expiry, `E_CLI_WAIT_TIMEOUT` (5) with `details.last_state` and `next` suggestions. A timeout is not a failure of the command that was already ACKed; the envelope says `accepted:true, wait:{timed_out:true}` where appropriate.

---

## 9. Discovery, self-description, and schema

### 9.1 Progressive help (P3)

- Level 0: `hivemind --help` — one line per resource, global flags, exit-code summary, discovery commands. Target < 500 tokens; no full schema dump.
- Level 1: `hivemind <resource> --help` — actions with summaries and tiers.
- Level 2: `hivemind <resource> <action> --help` — flags with types, required/default/enum, safety, dry-run, idempotency, examples.
- Level 3: `hivemind schema <resource.action>` — full typed JSON Schema, request/response mapping, errors, `since`/`deprecated`/`replaced_by`, examples. `schema` with no argument returns a compact command list; `schema --all` is for tooling only.
- `hivemind commands` returns the compact list for agents that need a menu, not full schemas.
- Help output follows the same channel rules: human mode to stderr; `--format json` yields a `help` envelope; `--help --format json` must not print prose to stdout. The Node prototype's `humanHelp` always writes prose to stdout and ignores `--json`; fix.

### 9.2 Schema is the source of truth (P6)

One `CommandSpec` + `ArgSpec` table drives parsing, validation, help, `schema`, `commands`, completions, and the generated skill:

```
CommandSpec {
  name, resource, action, summary, tier, mutating, destructive,
  requires_yes, dry_run, idempotent, transport, since, deprecated,
  args: [ArgSpec], output_schema, errors, examples, handler
}
ArgSpec {
  name, long, short, value_name, type, required, default, enum, min, max,
  format, repeatable, hidden, deprecated, replaced_by, removed_in, help
}
```

- Build the parser (clap dynamic `Command` or hand-rolled) from this table; do not maintain a second flag list.
- `schema <command>` serializes the table; `commands` serializes the compact projection; `completion` uses the same `clap::Command`; `skill` renders a checked-in markdown file from it.
- Contract tests assert: every command has a handler; every flag in the table is accepted; every handler flag is in the table; no orphan; `schema --all` matches a golden snapshot; `commands` count matches; `help` sizes bounded.
- Schema versioning: `meta.schema_version`; each command `since`; deprecated fields have `deprecated`/`replaced_by`/`removed_in`. An agent that cached a schema compares `meta.schema_version` and re-discovers on mismatch; it does not fail silently. The schema id stays `hiverouter.cli.v1` for additive changes; a breaking change requires a new id and a migration note.

### 9.3 Doctor

`doctor` composes, in order:

1. CLI version/schema and executable path/hash.
2. Server reachability and `/health` schema (`hiverouter.v1`); parse the body on 503.
3. Instance identity: `pid`, `boot_id`, `data_dir`, `workspace_root`, `database.path` vs lock; `stale_state`/`pid_reused`.
4. Lock/port ownership: port listening but not HiveMind; lock held but dead; foreign instance.
5. Data dir writable; DB readable; schema version; audit dir exists; snapshot dir.
6. Workspace root exists and is a directory; jail can open; `E_FILE_TREE_UNAVAILABLE` otherwise.
7. Provider key presence (boolean) from `/diagnose` or the `deepseek_api_key` check; `--require-key` makes absence blocking.
8. UI asset served (`GET /` contains `<div id="root">`) — optional.
9. Provider models/balance only with `--online` and a key; absence is a named refusal, not a doctor failure.
10. Version/schema drift and release artifact mismatch (exe hash vs state).

`doctor --fix` is a **tier 3**, `--dry-run`-first, `--yes`-gated repair of local, non-data state only:

- create a missing data dir (mode 0600/ACL);
- remove a stale lock/state only if PID is dead or identity mismatches (never kill);
- normalize log file names/append mode; optionally rotate by size if `--rotate-logs`;
- write/repair `server.json` only from a live `/health` identity;
- never touch the DB, runs, approvals, memory, workspace files, provider key, or server binary;
- report `fixes:[{action,status,detail}]`; `ok:"partial"` + exit 8 if any fix fails.

### 9.4 Completions and agent skill

- `hivemind completion powershell|bash|zsh|fish` emits a completion script from the same `CommandSpec`; no network, no state. A golden test pins each shell.
- `hivemind skill --format markdown` emits a compact agent skill (discovery, safety, examples, exit codes, no secrets). Check in `skills/hivemind/SKILL.md` generated from the table; CI fails if it drifts. The existing untracked `skills/hivemind/SKILL.md` is a good seed but documents the Node prototype's exit table; regenerate it after the unified contract.
- The skill must not include the auth lifecycle, raw token, key handling, or `--exe` as an agent recipe. It should include `doctor`/`start`, `commands`/`schema`, `--dry-run`, `--yes`, `watch`, and the error table.

### 9.5 MCP wrapper decision

**Not in v1.** HiveMind is a single-user localhost tool; the CLI process model is cheap, auth is a local boot token, and progressive help is the token-efficient discovery path. An MCP server would eagerly expose ~30 tools, duplicate the schema, add a second auth/lifecycle surface, and (for mutations) need the same approval authority. The hybrid guidance is explicit: CLI for local/scriptable/infrastructure work; MCP for multi-tenant SaaS, per-user OAuth, and network-attached access (`references/hybrid-mcp-cli.md` decision matrix). HiveMind is the former.

If an MCP host requires it later, add `hivemind mcp serve --read-only` as a thin adapter over the same `CommandSpec`: 8–10 read tools (`health`, `runs.list`, `runs.show`, `runs.state`, `agents.roster`, `workspace.tree`, `workspace.read`, `settings.show`), no secrets, no mutations, no separate backend. Do not expose 30 tools or a second schema. This is P2, not a release blocker.

---

## 10. Windows, headless, and app-open parity

### 10.1 Quoting and input

- `--text-file <path>` and `--text -` (stdin) are the prompt paths. `--text` is for short strings; PowerShell examples use single quotes or a file.
- Read UTF-8, strip a BOM, reject NUL and invalid UTF-8, preserve newlines, and count **Unicode scalar values** (`chars().count()`), matching the server's `MAX_PROMPT_CHARS` (`command.rs:34–35,395–402`). The Node prototype's `text.length` counts UTF-16 units and rejects/accepts differently for emoji; fix.
- `--` terminates options, so IDs/paths beginning with `-` are safe. `--flag=value` is canonical; `--flag value` is accepted for scalars; never split a value on spaces in a wrapper.
- `--exe`/`--browser` paths with spaces are passed as one argv element; no `cmd /c` string interpolation.
- A secret is never an argv value. The prompt may contain user secrets; do not log prompt text, only length/hash if needed. `--text-file` avoids shell history.

### 10.2 Wrappers

- `HiveMind.cmd` (human launcher): `hivemind start --open`, no `pause` on error, `exit /b %ERRORLEVEL%`; `--pause-on-error` optional.
- `HiveMind-Agent.cmd` / `HiveMind-Agent.ps1` (headless): `hivemind` with passed args, no PowerShell `Start-Process`, no browser, no `pause`.
- `tools/hivemind.cmd` / `.ps1`: thin wrapper over the production binary; do not invoke `node` once the Rust binary exists. If the Node prototype remains for development, label it `hivemind-dev` and do not ship it.
- The old `tools/start-hivemind.ps1` is deprecated: it always opens a browser, truncates logs on each start, and uses 8099. It now forwards `--workspace-root $root` (fixed during this seat), but it still has no headless mode, no lock/PID identity, and no graceful stop. Keep it only as a compatibility shim that calls `hivemind start` with explicit flags, or delete it in the release that ships `hivemind`.

### 10.3 Headless vs app-open parity

- `start` is headless by default: no browser, stdout redirected to logs, JSON envelope. `--open` is the only start path that opens a browser; `open` is a separate human command.
- `HiveMind.cmd` may open; `HiveMind-Agent.cmd` must not. Both use the same port/data-dir/workspace defaults and the same server binary.
- `open` under non-TTY without `--print-url` is `E_CLI_HUMAN_ONLY` (7); `--print-url` prints `{url, opened:false}` and exits 0. `open` never runs in an agent loop by accident.
- Deep link parity: HiveMind's web app is a single embedded SPA; there is no verified `?run=` route. `open` opens the base URL only. Do not promise `runs show --open` until the web app supports a run route and a test proves it. UI-only commands (`chat-pin`, `viewport`, `recenter`) are not part of headless parity.
- Test: start headless, assert `/health` + `/api/session` + `GET /` asset; then `open --print-url`; then `start --open` under a TTY harness; compare URL/data-dir/pid. No browser automation required for the headless half.

---

## 11. Test harness and acceptance

### 11.1 What must be tested

The CLI's agent-native claim is falsifiable. These tests are cheap and belong in CI, not in a manual smoke log. The current repo has only unit tests inside `cli.rs` (`cli.rs:829–1021`) and a recorded smoke log (`backend/cli-smoke.log`); the Node prototype has no tests.

| Gate | What it proves | Harness |
|---|---|---|
| Envelope shape | every command success/failure/partial is one valid JSON object with `schema`, `ok`, `command`, `data\|error`, `meta`; no prose on stdout | `assert_cmd` + JSON Schema snapshot; iterate `commands` |
| Stdout/stderr split | stdout parses as one JSON doc in `json`/`jsonl`; stderr never contains the envelope; human mode stdout empty | capture both streams; `serde_json::from_str`; assert stderr no `"schema"` |
| TTY behavior | non-TTY defaults to JSON; TTY defaults to human-on-stderr; `--format` overrides; no pager; no prompt | `portable-pty`/ConPTY on Windows; fallback explicit `--format` tests; `PAGER=false` |
| Exit matrix | each class 0/2/3/4/5/6/7/8/9/130 | synthesize each failure (bad flag, keyless submit, unreachable, stale lock, timeout, omitted `--yes`, partial, schema mismatch, SIGINT) |
| Dry-run safety | every tier 2/3 command has `dry_run:true` and `would_request`; state/store fingerprint unchanged | run `--dry-run` for every command; compare lock/state/DB/log hashes; fake server records zero mutations |
| Idempotency | `runs submit` twice with same key returns same run and one run row; commands without receipt reject `--idempotency-key` honestly | fake server + temp DB; replay; count rows |
| Confirmation | no `--yes` under non-TTY returns `E_CLI_CONFIRMATION_REQUIRED` within 1s, never hangs; `--yes` proceeds | closed stdin + timeout; assert no process hang |
| Schema drift | `schema --all` and `commands` match golden snapshots; every command has a handler; no orphan flags; help size < budget | snapshot diff; compile-time table assertions |
| Lock/identity | two starts on one data dir cannot both spawn; stale PID is cleared, not killed; PID reuse refuses; port non-HiveMind refuses | fake process identity; temp data dir; OS process fixture; `doctor --fix` |
| Start/stop | headless start, `/health`, `/api/session`, `GET /`, status, stop; no browser; child-exit aborts fast | fake server binary (axum) + a small child process fixture; real binary optional |
| Resync | `RESYNC_REQUIRED` triggers snapshot + reconnect; duplicate seq dropped; no silent gap | fake WS server sends sequence with gap; assert resync records and final summary |
| Approval authority | approve without nonce/`--yes` fails; nonce replay maps correctly; expired approval fails; no `--all` | fake WS `APPROVAL_REQUEST` + `APPROVAL_RESOLVED` |
| Secret output | fake key/token never appears in stdout/stderr/state/logs; `session` redacts token; `settings` only boolean | inject `sk-test-...`; scan all artifacts; assert `sk-` absent |
| Windows quoting | paths with spaces/Unicode, `--text-file`, `--text -`, `--` terminator, CRLF, BOM | `cmd`/PowerShell wrapper tests; `std::process::Command` argv vectors; no shell strings |
| Locale/time | bytes identical under `LC_ALL=C`/`TZ=UTC` and a divergent locale/TZ (Windows culture) | run twice, diff; timestamps UTC; numbers `.` |
| Log bounds | `logs --lines` reads a bounded tail; large file does not exhaust memory; no whole-file read | large temp log; assert cap and no OOM |
| Open/headless parity | `start` never opens a browser; `open --print-url` works non-TTY; `open` non-TTY fails `E_CLI_HUMAN_ONLY`; TTY path opens once | process spy on browser; no real browser |
| Contract test | no CLI command exists that the server rejects as unknown; every WS name in `CommandName::ALL` that is exposed is handled | table-driven against `command.rs`/`schema.rs`; no live server required |

### 11.2 Harness shape

- **Rust integration tests** in `backend/crates/hiverouter-tools/tests/cli_contract.rs` (or a new `hiverouter-cli` crate): `assert_cmd`, `predicates`, `tempfile`, `serde_json`, `jsonschema`. Use a fake HTTP/WS server built with the existing `axum` dependency and a WS test client (`tokio-tungstenite` dev-dependency) rather than the real server. This keeps tests offline and deterministic.
- **Fake server** must reproduce: Host gate, `/api/session` token, `/health` 200/503 bodies, all read routes, WS hello/ACK/ERROR/broadcast, `RESYNC_REQUIRED`, approval replay, keyless `E_KEY_MISSING` before run creation, and the three HTTP error-body variants (api-map §2). It must not fabricate success for a real-path failure.
- **Real-server smoke** remains a separate, opt-in gate (no key for reads; `E_KEY_MISSING` for submit), using the existing `e2e/helpers/live-server.ts` pattern (ephemeral port, temp data dir, child-exit abort, redacted logs). Do not put a paid call in CI.
- **`tools/test.ps1`** adds a CLI gate: `cargo test -p hiverouter-tools --locked` is already there; add `cargo test --test cli_contract` and a `node`/`pwsh` wrapper test for `HiveMind-Agent.cmd` if the wrapper remains. A skipped test is not green; print `SKIP` and fail the release if the CLI gate is required.
- **No mocks in product paths.** Test doubles are confined to tests/fake servers; production code must not branch on `NODE_ENV=test`/`cfg(test)` to fabricate data.

### 11.3 Acceptance for the design itself

The design is accepted when a reviewer can answer "yes" to all of these from the document and the repo:

1. Is there exactly one production CLI owner and one exit/envelope table? (§0, §6)
2. Does every command have a tier, `--dry-run` if mutating, `--yes` if destructive, and a typed schema? (§4, §7, §9)
3. Does `stdout` stay machine-only in `json`/`jsonl`, with human output on stderr? (§1, §5)
4. Does the lifecycle have lock/identity/stale checks and a truthful stop? (§8)
5. Is every mutation's retry story explicit, including the ones that are **not** safe? (§8.3)
6. Does discovery work without a README, and is the top-level help token-bounded? (§9)
7. Are key/token/approval authority and secret output handled? (§7.3–7.5)
8. Is the Windows/headless path specified, including quoting and no browser? (§10)
9. Is there a test harness that proves all of the above? (§11)
10. Are the genuinely missing server changes listed with priorities? (§12–13)

---

## 12. What is genuinely missing from the repo

### 12.1 What is not missing

Be fair to the existing work:

- 19 HTTP routes including history, messages, agent messages, roster, profiles, memory, usage, projects, models, balance (`http.rs:511–536`; api-map §2).
- The full WS command dispatcher for submit/spawn/rewire/cancel/kill/approve/settings/files/template/history (`ws.rs:610–937`).
- A per-boot session token and Host/Origin gates (`ws.rs:364–387`, `http.rs:543–564,1206–1233`).
- A stable refusal table with `retryable` and `how_to_fix` (`refusal.rs:73–282`).
- `SUBMIT_PROMPT` idempotency by correlation for 24h (`store.rs:960–1005`, `execution.rs:1031–1073`).
- A shipped Rust CLI with a good envelope and exit tests for its narrow surface (`cli.rs:81–172,829–1021`).
- A broad untracked Node prototype and a usage doc/skill (`tools/hivemind-cli.mjs`, `docs/AGENT-CLI.md`, `skills/hivemind/SKILL.md`).
- An e2e live-server harness that is better behaved than the product launchers (`e2e/helpers/live-server.ts`).

The gap is not "nothing exists"; it is "the existing pieces do not add up to one production contract."

### 12.2 Missing or insufficient — ranked

| # | Missing | Evidence | Impact | Required change | Priority |
|---:|---|---|---|---|---|
| 1 | One CLI owner and one exit/envelope contract | Rust `0/2/3/4` (`cli.rs:51–57`); Node aliases to `0/2/3/4` but collapses transport/timeout/safety; both have top-level `schema` now | no single owner; semantic classes are lost; scripts keyed on old Node `1/3/4/5/6` still break | choose one binary; unified table; deprecate the other | **P0** |
| 2 | Typed schema as source of truth | Rust `CommandSpec` has only flag names (`cli.rs:59–78,81–172`); `schema` emits a hand-written response placeholder (`cli.rs:558–591`); Node registry args are not JSON Schema | help/validation/schema drift; agents guess flags | `CommandSpec` + `ArgSpec`; derive help/schema/completion/skill; `since`/deprecated | **P0** |
| 3 | Lifecycle lock, identity, stale-PID safety | no lock/PID/instance record in product (`lifecycle-map §6`); prototype state file is non-atomic, no creation time, no lock | two servers on one data dir; wrong-instance adoption; kill unrelated PID | `hivemind.lock` + `server.json`, atomic, PID creation time, exe hash, boot_id | **P0** |
| 4 | Instance-identifying `/health` | `/health` has no `pid`/`boot_id`/`data_dir`/`workspace_root`/build (`http.rs:456–476`) | CLI cannot prove which server it talks to | add fields (or make `server.json` authoritative) | **P0** |
| 5 | Truthful graceful stop | server only handles Ctrl+C (`main.rs:178–184`); `taskkill` without `/F` no-ops on headless children (lifecycle-map §7) | `stop` may force-kill after the grace window; audit loss | `stop` reports `graceful:false` + one-command escalation now; add Ctrl+Break/SIGTERM or token-authenticated shutdown P1 | **P0** (truth) / **P1** (grace) |
| 6 | `/api/*` token authentication | only `/ws` checks token (`ws.rs:382–384`); `/api/*` only Host-guarded (`http.rs:543–564`); DELETE profile/memory unauthenticated | any loopback client can mutate profiles/memory; CLI token is only used for WS | token middleware on `/api/*` except `/health`/`/api/session`; `Cache-Control: no-store` on session | **P0** |
| 7 | General mutation idempotency | only `SUBMIT_PROMPT` receipt (`execution.rs:1031–1073`); contract claims all (`UI-COMMANDS-v1.md:24`) | retries duplicate spawn/template/rewire/kill/cancel/approval | generalize `command_receipts` to all mutating commands with result JSON + 24h TTL | **P0** for spawn/template, **P1** for rest |
| 8 | WS client with Origin/correlation/resync | server requires `Origin` (`ws.rs:75–83,376–381`); Node built-in WebSocket sends none (observed; candidate now has a manual upgrade); no correlation filtering/resync | commands/watch fail or pick another client's event | Rust WS client that sets Origin, filters by correlation/run, handles `RESYNC_REQUIRED` | **P0** |
| 9 | `--wait-for`, `--idempotency-key`, `--limit`, `--cursor`, `--max-bytes`, `--fields`, `--full`, `--no-input`, `--timeout` | Rust has only `--format`, `--dry-run`, `--url`, `--timeout` on health/diagnose; Node has `--timeout-ms`, no wait/idempotency/cursor | agents write their own polling/retry; unbounded output | global flags + per-command validation; bounded page contract | **P0/P1** |
| 10 | Pagination/truncation server-side | runs hard-cap 200 (`http.rs:622`); messages 1000 (`http.rs:782`); roster/revisions/projects/profiles/memory unpaginated | `has_more`/cursor cannot be truthful; agents over-fetch | cursor + stable sort + `page` on list routes | **P1** |
| 11 | `GET /api/approvals`, `GET /api/templates`, `GET /api/workspace`, `GET /api/runs/active` | no routes; approvals only via WS hello replay (`ws.rs:569–598`); templates only insert; workspace only `/diagnose`/state; active run internal | CLI cannot list approvals/templates/workspace/active run cleanly | read-only routes (token-gated, nonce only for live approvals) | **P1** |
| 12 | `doctor --fix` | no composed doctor; prototype doctor is read-only and partial | stale lock/state requires manual repair | safe repairs only, `--dry-run`, `--yes`, never DB/key/runs | **P1** |
| 13 | Watch resync/reconnect and JSONL summary | Node `watch` collects events; no `RESYNC_REQUIRED` handling; `--follow` writes raw frames without summary; no seq dedupe | silent gaps; agent cannot tell progress/terminal | resync policy, seq watermark, final summary, SIGINT flush | **P0/P1** |
| 14 | `hivemind completion` and generated `skill` | no completions; skill exists untracked but documents Node exit table | discoverability/ergonomics; drift | generate both from `CommandSpec`; CI drift gate | **P2** |
| 15 | MCP wrapper | none | not needed for local CLI; some hosts want tools | defer; optional read-only adapter P2 | **P2** |
| 16 | CLI contract test harness | only `cli.rs` unit tests + `backend/cli-smoke.log`; no TTY/exit/dry-run/lock/resync/secret tests | regressions are invisible; "should work" is not evidence | add `cli_contract` integration suite + fake server + optional real smoke | **P0** |
| 17 | Windows launcher parity | `HiveMind.cmd` opens browser always and pauses on error; `start-hivemind.ps1` still truncates logs and uses 8099; `HiveMind-Agent` is headless | agent blocks on pause; port split; log loss | wrappers call `hivemind`; no pause; one port; bounded logs | **P1** |
| 18 | Log policy | `.cmd` truncates each start; Node appends forever; `logs` reads whole file | evidence loss/OOM | append + size cap + rotation + bounded tail | **P1** |
| 19 | Server boot exit taxonomy | every boot failure exit 1 (`main.rs:18–24`) | supervisor cannot classify port/DB/config | distinct server codes or boot error file | **P2** |
| 20 | Doc truth and drift | `docs/CODEMAP/03-BACKEND.md:67` and `09-RISKS-GAPS.md:5` say no chat-history endpoint, but `http.rs:520,772` exposes it; `runbook.md:3–8` says two binaries while a Node CLI exists | agents/docs disagree; false "missing" claims | update docs in the same change; add a docs-truth check | **P2** |

### 12.3 Candidate-specific defects to fix before calling it done

From the lifecycle snapshot (`lifecycle-map §11`) and this review; line numbers in the untracked Node file move, so function names are the stable anchors:

- **Human help on stdout** — `humanHelp` still writes prose to stdout and ignores `--json`; human data output is now stderr (fixed), but help must return an envelope in JSON mode.
- **`ok:false` without `error`** — `doctor` ready=false can return `ok:false` with only `data`; the envelope contract requires `error` on `ok:false`.
- **Coarse exit table** — the latest snapshot aliases `NOT_RUNNING`/`AUTH`/`TIMEOUT` to 3 and `UNSAFE` to 2; it matches Rust but loses the semantic classes 5/6/7/8/9/130 that the design requires.
- **No lock / non-atomic state** — `agent-cli-state.json` write is not atomic, no PID creation time, no boot_id, no exclusive lock; two starts can share a data dir.
- **Stop cannot be graceful** — `taskkill /T` without `/F` no-ops on headless children; the latest snapshot auto-escalates to `/F`, which still loses the audit flush; `restart` passes `yes:true` internally.
- **`--dry-run` side effects (earlier snapshot)** — the earlier `start` path wrote state before checking dry-run; the latest snapshot checks first. Verify with a state/log fingerprint test before calling it fixed.
- **No resync** — `watch` treats `RESYNC_REQUIRED` as an ordinary event; no reconnect/snapshot.
- **Name-only event matching** — `wsCommand` waits for any event with the awaited name, not the `correlation_id`/`run_id`; the bus broadcasts all clients.
- **Submit/watch race** — `run` opens a second WS after ACK; it can miss `RUN_CREATED`/terminal events; use the same connection or read back.
- **Integer-only `viewport`** — `readNumber` rejects fractional zoom/x/y; server accepts f64; the command is UI-only and should be hidden anyway.
- **UTF-16 prompt length** — `text.length` differs from server `chars().count()` for non-BMP characters.
- **Unknown flags ignored** — parse accepts any `--name` into options; boundary validation must reject unknown flags per command.
- **Executable search still includes `backend/target` fallbacks** — `target/release` is first now, but a missing target can fall through to a stale `backend/target/debug` build (lifecycle-map D2); prefer the installed binary and record its hash.
- **No instance identity** — "already running" = 200 + same URL + PID alive; a foreign HiveMind can be adopted (D3).
- **Node runtime dependency** — the prototype requires Node >= 22.12; the release/runbook says Node is only for web build, not server runtime. Decide before shipping.

### 12.4 What the design does **not** change

- The frozen wire contract (`hiverouter.v1`), 17 command names (including 3 decision-gated `DIFF_*`), 24 events, 16 agent states, and 64 refusal codes stay as published. This design consumes them; it does not add a command or refusal without a `CHANGE-LEDGER` row and the contract-matrix re-run.
- The server remains the sole authority; the CLI does not read the DB directly, does not write run state, and does not duplicate the engine.
- The embedded UI is unchanged; `open` is a convenience, not a second client.

---

## 13. Prioritized build plan

### 13.1 P0 — must land before the CLI is called production

**P0-1. Contract and ownership (S/M).**
Decide: Rust `hivemind` owns production; Node prototype is `hivemind-dev`/reference, not shipped. Publish the unified envelope/exit table and the deprecation policy for `hiverouter-cli`. Acceptance: one document, one table, one binary in `release-manifest.json`; no second exit table.

**P0-2. Schema-first command model (M).**
`CommandSpec`/`ArgSpec`; derive parser, help, `commands`, `schema`, completions (stub), skill; add `since`/`deprecated`/`replaced_by`; top-level help < 500 tokens; `schema --all` snapshot. Acceptance: every command has a handler and a typed schema; no orphan flags; golden snapshot green.

**P0-3. Transport (M).**
HTTP client with Host/Origin correctness and token; WS client that sets `Origin`, sends hello, filters ACK/ERROR/results by `correlation_id`/`run_id`, handles `SERVER_HELLO`/`RESYNC_REQUIRED`; bounded timeouts; `X-Request-Id`; token redaction. Acceptance: fake WS integration tests for every exposed command; no name-only matching; no secret in logs.

**P0-4. Lifecycle (M).**
`hivemind.lock` + `server.json` + atomic writes; start/status/stop/restart; PID creation time + exe hash + boot_id identity; child-exit abort; 250ms/30s readiness; stale/port/instance states; no browser by default; logs append + bounded tail; stop truthfully reports graceful/escalated. Acceptance: two-start conflict, stale PID, PID reuse, non-HiveMind port, child-crash; no data deletion.

**P0-5. Read surface (M).**
`server status/health/diagnose/logs`; `runs list/show/revisions/state/messages/usage`; `agents roster/messages/profiles/memory list`; `workspace info/projects/tree/read`; `settings show/models/balance`; `history replay`; `approvals list/show`; `templates list/show`; `watch`. Backing routes exist except the four additions in §12; use composition/`/diagnose` where honest, and mark pending routes as `E_CLI_NOT_IMPLEMENTED` (9/4) rather than faking. Acceptance: every read works against a fake server; keyless provider reads refuse `E_KEY_MISSING`.

**P0-6. Safety and secrets (S/M).**
Central tier/`--yes`/`--dry-run` middleware; no `--token`; key presence only; redaction scan; approval nonce/authority; `open` human-only; `--exe` operator-gated; unknown flags rejected; Unicode scalar prompt count; Windows `--text-file`/`--`. Acceptance: secret scan zero; dry-run fingerprint unchanged; confirmation returns in <1s under closed stdin.

**P0-7. Test harness and CI (M).**
`cli_contract` integration suite with fake HTTP/WS server; TTY tests; exit matrix; schema snapshot; lock/identity; resync; approval; secret; Windows quoting; locale; log bounds. Wire into `tools/test.ps1`; no skip-as-green. Acceptance: all gates green on a clean checkout; failure-path tests exercised.

**P0 exit criteria:** `hivemind doctor` (server down) → JSON exit 5; `hivemind server start` → headless JSON; `status`; `runs submit` keyless → `E_KEY_MISSING` and no run; `watch` survives a fake resync; `stop` truthful; no secret in any artifact; one exit table.

### 13.2 P1 — control-plane completeness

**P1-1. Mutations with receipts (M/L).**
Submit/cancel/kill/rewire/spawn/template insert/approval/profile/memory; generalized `command_receipts` for all mutating commands (result JSON, 24h TTL); `--idempotency-key`; `--wait-for`; batch partial where real. Acceptance: replay returns the original result; duplicate spawn/template is impossible; timeout sets `outcome_unknown`.

**P1-2. Server read additions (M).**
`GET /api/approvals?run_id=`, `GET /api/templates`, `GET /api/workspace`, `GET /api/runs/active`; token middleware on `/api/*` except `/health`/`/api/session`; `Cache-Control: no-store` on session; `/health` `pid`/`boot_id`/`data_dir`/`workspace_root`/build. Acceptance: CLI list commands are first-class, not WS-replay hacks; tokenless `/api/*` is 401.

**P1-3. Pagination/truncation (M).**
Cursor + stable sort for runs/messages/revisions/roster/projects/profiles/memory/approvals; `page` block; `--limit`/`--cursor`/`--max-bytes`/`--fields`/`--full`. Acceptance: `has_more` truthful; no silent cap.

**P1-4. Watch robustness (M).**
Resync snapshot + reconnect + seq dedupe + final summary + SIGINT 130; event filters; `--since-seq`. Acceptance: fake gap test; no duplicate/lost events; summary matches count.

**P1-5. Lifecycle hardening (M).**
Ctrl+Break/SIGTERM or token-authenticated shutdown; `--drain`; restart active-run refusal; log rotation/size cap; workspace validation; DPAPI key injection optional; port policy unified; wrapper parity. Acceptance: one graceful stop path proven; `HiveMind.cmd`/`HiveMind-Agent` no pause/always-open divergence.

**P1-6. Doctor `--fix` (S/M).**
Safe repairs only; dry-run; `--yes`; partial; never DB/key/runs. Acceptance: stale lock/state repaired; live process never killed; no data deletion.

### 13.3 P2 — ergonomics and long-term

- `--format table` + `--fields`/`--full` polish; shell completions; generated skill drift gate.
- `mcp serve --read-only` adapter (8–10 read tools, no mutations, same schema); do not expose the full surface.
- Deep link `open --run <id>` only after the web app has a verified route.
- Server boot exit taxonomy; auto-restart policy (rate-limited, recorded) if ever; log rotation daemon.
- Remove `hiverouter-cli` alias after the deprecation window.

### 13.4 Dependencies and risks

| Risk | Mitigation |
|---|---|
| WS protocol drift (frozen contract) | pin to `hiverouter.v1`; contract tests; no new command without ledger |
| `Origin`/Host gates | always set Origin for WS; never override Host; bind 127.0.0.1; test with `localhost` and `[::1]` |
| Windows graceful stop | do not claim grace until a real signal test passes; report `graceful:false` now |
| PID reuse / stale lock | creation time + exe hash + boot_id; never kill on mismatch |
| Token leakage | memory-only; redaction; no `--token`; fake-key/token scan |
| Idempotency overclaim | per-command table; refuse `--idempotency-key` where unsupported |
| Node runtime | choose Rust production; if Node, declare/bundle runtime and unify exit table |
| Test flakiness | fake server, no network, no paid key; TTY tests isolated; real smoke opt-in |
| Schema drift | one `CommandSpec`; golden snapshots; `since`/deprecation |

### 13.5 Suggested sequencing

1. P0-1/P0-2 (contract/schema) — no code beyond a table and tests.
2. P0-3/P0-4 (transport/lifecycle) — the highest-risk, highest-value pair.
3. P0-5/P0-6 (read/safety) — proves the contract on real routes.
4. P0-7 (tests) in parallel; gate each step.
5. P1-1/P1-2 (mutations/routes) — only after P0 exit criteria.
6. P1-3/P1-4/P1-5/P1-6.
7. P2.

---

## 14. Open decisions

1. **Owner:** Rust `hivemind` (recommended: single binary, existing envelope/exit tests, no runtime dependency) vs Node prototype (faster, broad, but adds Node runtime and a coarse/duplicated table). If Node, bundle Node >=22.12 and remove the Rust table in the same release.
2. **Default port:** 8080 (server/docs) vs 8099 (launchers). Recommend 8080 everywhere; explicit `--port 8099` for legacy. Change `start-hivemind.ps1`/launcher in the same release.
3. **`/api/*` auth:** add token middleware (recommended) or accept loopback-only Host guard (not safe for mutations). The CLI design assumes token.
4. **`stop --yes`:** require it always (safe, simpler) vs only when active run/unmanaged (lifecycle-map §9.5). This design requires it for any stop that can interrupt a run; document the exact policy.
5. **`--exe` trust:** operator-only env/config (recommended) vs agent-settable with `--yes`; if the latter, record hash + path and treat as tier 3.
6. **Approvals read:** add `GET /api/approvals` (recommended) vs WS hello replay (works but noisy/broadcast). If replay, bound to 20 and filter by run; never use it as the only source.
7. **MCP:** defer (recommended); if required, read-only 8–10 tools, no mutations, same schema.
8. **Schema id:** keep `hiverouter.cli.v1` additive (recommended) vs new `hivemind.cli.v2`; only a breaking change justifies a new id.
9. **Node prototype disposition:** keep as `hivemind-dev` reference (recommended) or delete after parity; do not ship both.
10. **Server build id:** add to `/health`/`server.json` (recommended) so `status` can prove artifact identity; otherwise rely on exe hash in the lock.

---

## 15. Citations and evidence

### 15.1 Principles applied

- **P0 One CLI, Three Audiences** — channel split, one binary, one table (§1, §4).
- **P1 Structured Output Is the Interface** — stdout JSON/stdout-empty-human, envelope, exit codes, idempotency, bounded output (§1, §5, §6, §11).
- **P2 Trust Is Directional** — args untrusted, operator-owned token/exe/workspace, no `--token`, no CLI key (§1.2, §7).
- **P3 The CLI Must Describe Itself** — progressive help, `commands`, `schema`, `doctor`, top help < 500 tokens, no README (§9).
- **P4 Safety Through Graduated Visibility** — tiers 0–3, hidden destructive/UI-only commands, `--yes`, fail-closed sandbox (§7).
- **P5 Validate at the Boundary** — one typed `CommandSpec`, reject unknown flags, Unicode scalar count, Windows input (§1.5, §9.2, §10).
- **P6 The Schema Is the Source of Truth** — one table drives parse/help/schema/completion/skill; version/deprecation (§9.2).
- **P7 Authentication Must Be Delegatable** — token env/file, no login flow, key presence only (§1.3).

Skill references: `agent-native-design/SKILL.md` (seven principles, interaction loop), `references/rubric.md` (14 criteria), `references/design-patterns.md` (output envelopes, idempotency, non-interactive, long-running, help, safety, auth, locale), `references/testing.md` (contract gates), `references/hybrid-mcp-cli.md` (CLI vs MCP). `agent-experience` is not a standalone installed skill; the applied duties are the TestDrive Fixer 3 definition (`testdrive/SKILL.md:50`).

### 15.2 Repo evidence

| Claim | Evidence |
|---|---|
| Shipped Rust CLI command table / exit codes / help / schema / HTTP only | `backend/crates/hiverouter-tools/src/cli.rs:51–57,81–172,304–341,389–409,411–462,558–591,778–815` |
| Rust CLI tests | `backend/crates/hiverouter-tools/src/cli.rs:829–1021`; `backend/cli-smoke.log` |
| HTTP route table, Host guard, session token, history/messages/roster/memory/usage/projects/models/balance/health | `backend/crates/hiverouter-core/src/http.rs:511–536,543–564,621–694,772–845,951–1064,1070–1151,1153–1233` |
| WS auth, hello/replay/resync, command dispatch, approval replay | `backend/crates/hiverouter-core/src/ws.rs:364–387,495–598,610–937` |
| Command names/payloads/validation, refusal table | `backend/crates/hiverouter-contract/src/command.rs:45–164,166–287,393–448`; `refusal.rs:73–282` |
| `SUBMIT_PROMPT` receipt/idempotency | `backend/crates/hiverouter-core/src/store.rs:960–1005`; `execution.rs:1031–1073` |
| Contract claim "all mutating idempotent 24h" | `docs/contracts/UI-COMMANDS-v1.md:24–26` |
| Server boot/args/help/ctrl-c | `backend/crates/hiverouter-core/src/main.rs:18–24,44–50,136–158,178–184`; `config.rs:249–296` |
| Launcher behavior/port/workspace/browser/pause | `tools/start-hivemind.ps1:1–74`; `HiveMind.cmd:1–5`; `tools/runbook.md:3–8,20–28,59–74` |
| Release artifacts | `tools/release-manifest.json` (`hiverouter.exe`, `hiverouter-cli.exe`) |
| Candidate CLI / doc / skill | `tools/hivemind-cli.mjs`; `docs/AGENT-CLI.md`; `skills/hivemind/SKILL.md` |
| Surface/gap recon | `docs/trace/agent-cli/api-map.md` §2, §11, §12; `docs/trace/agent-cli/lifecycle-map.md` §3, §6, §7, §9, §10, §11 |
| Doc drift (chat route) | `docs/CODEMAP/03-BACKEND.md:65–71`; `docs/CODEMAP/09-RISKS-GAPS.md:5–12` vs `http.rs:520,772` |

### 15.3 Unverified / not claimed

- No HiveMind process was started, stopped, signalled, or probed; no HTTP/WS request was sent to it; no browser was launched; no provider key was read/decrypted.
- The Node prototype was observed only for local `commands`/`--help`/`schema` behavior; its WS/HTTP commands are **not** runtime-verified in this seat.
- A direct probe of Node's built-in `WebSocket` showed no `Origin` header; the candidate now implements a manual upgrade (`MiniWebSocket`), but that path is also not live-tested here.
- Graceful stop via console-control injection is unproven; the lifecycle map measured `taskkill` without `/F` as a no-op on headless servers. Treat `graceful:false` as the only currently truthful claim.
- The `/health` 503 body shape for a `failing` boot is source-read, not observed.
- `docs/CODEMAP` says no chat-history endpoint; the source now has one. The source is authoritative; the doc is stale.
- No `anc` linter is installed; the rubric score is a manual review, not an `anc` scorecard.

---

## 16. One-paragraph summary

HiveMind already has the hard parts — a real HTTP/WS contract, a frozen command/refusal vocabulary, a session token, a receipted submit path, and a broad in-flight prototype. What it lacks is a **single production CLI contract**: one owner, one envelope with `schema`, one exit table, one typed schema, one lifecycle identity, one idempotency story per mutation, one bounded-output/pagination contract, and a test harness that proves it. The recommended build is a Rust `hivemind` binary that keeps the shipped `0/2/3/4` classes, adds semantic 5/6/7/8/9/130, uses progressive help and typed schemas, sets `Origin` on WS, owns a lock with PID/exe/boot identity, never prints the key/token, and reports `graceful:false` until a real graceful-stop path exists. The Node prototype is the behavioral reference, not the release artifact. MCP is deferred. Without the P0 server changes — token-gated `/api/*`, instance-identifying `/health`, generalized receipts, and the four read routes — the CLI can be a good read-only diagnostic tool, but not a production control plane.

---
