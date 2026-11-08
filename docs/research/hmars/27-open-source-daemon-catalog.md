# 27 — Open-source hyperagent / superagent / agent / subagent / remote TV-remote / watchdog daemon catalog, second pass

**Lane:** wave-3 EXTRA seat 27. **Written:** 2026-11-08 (UTC), sampling window 03:30Z–04:15Z.
**Repository:** `C:\Users\Riley\Desktop\hiverouter-v1`. **HEAD read:** `bd575ba` (working tree dirty;
same dirty files recorded by `12-role-capability-model.md`).
**Scope:** research only. This lane wrote **only this file**. No product source, test, migration,
config, build, server, key, credential, or provider call was touched. No code was copied. No paid
API was called; the only network traffic was GitHub metadata, raw README/LICENSE files, and four
public documentation pages, all named in §15.
**Required reading consumed:** `OPERATOR-DECISIONS.md`, `09-synthesis-and-goal.md`,
`10-gap-meta.md`, `12-role-capability-model.md`, `14-a2a-bus-roster.md`, `18-open-source-agents.md`
(the wave-2 catalog this file extends), plus a live read of the role/daemon code.

---

## 0. One-paragraph answer

The wave-2 catalog answered **which brands** a HiveMind donor list should name; this pass answers
**which mechanics** the Remote/Watchdog tier actually needs, and finds 34 projects wave 2 did not
catalogue (zero overlap). The most important finding is that the operator's "one job forever,
report every five minutes" is not an agent problem at all: it is the **process-supervision**
problem solved decades ago by `s6`/`daemontools`/`runit` — one service, one supervisor, one
readiness signal, a restart policy with intensity limits, and a log. The second finding is that
the operator's "TV-remote" is a **deterministic command table plus a state read**, best evidenced
by `pyatv` and `node-red`, and by `home-assistant`'s entity/service split; it is not an LLM turn.
The third is that the missing pieces wave 2 left unnamed are **enrolment** (Fleet, NanoMDM,
Tailscale, Syncthing), **report lifecycle and dedupe** (Alerta, Karma, Keep, Robusta, OneUptime),
and **bounded delivery** (NSQ's `--mem-queue-size` + disk overflow + ephemeral drop; River,
Temporal, NATS; Rust-native `tower`/`governor`/`flume`). The best single new donor is **`s6`**
(ISC): it supplies the exact daemon grammar HiveMind lacks, with no runtime to adopt. The best
product-shaped donor is **`process-compose`** (Apache-2.0): a single Go binary that supervises
local processes with liveness/readiness probes, restart policy, interval/cron scheduling, a TUI
and a token-authenticated REST control plane — i.e. a HiveMind daemon lane plus the operator's
remote in one design. The catalog's verdict is consistently **adopt the supervision and report
grammar, adapt the local-first control surfaces, reject the agentic runtimes** as daemon hosts:
a daemon with a model in its check loop is neither "one job" nor "forever".

**Top-10 shortlist (full table in §3):**

| # | Donor | Licence | Last push (UTC) | Tier served | Verdict | Why it earns the slot |
|---|---|---|---|---|---|---|
| 1 | [`skarnet/s6`](https://github.com/skarnet/s6) | ISC | 2026-11-07 | Daemon supervisor | **ADOPT (design)** | "One service → one daemon", readiness notification on an fd, supervision tree — the daemon contract itself |
| 2 | [`F1bonacc1/process-compose`](https://github.com/F1bonacc1/process-compose) | Apache-2.0 | 2026-11-03 | Daemon lane + control plane | **ADOPT (design)** | Single binary, liveness/readiness probes, restart, interval/cron, TUI + token-auth REST — the closest product analogue to HiveMind's missing daemon lane |
| 3 | [`alerta/alerta`](https://github.com/alerta/alerta) | Apache-2.0 | 2026-10-17 | Report / dead-man | **ADOPT (design)** | Alert-as-state-record: lifecycle (`new/ack/closed/expired`), heartbeat + timeout, blackout; exactly the report/dedupe/mute model |
| 4 | [`postlund/pyatv`](https://github.com/postlund/pyatv) | MIT | 2026-08-14 | TV-remote | **ADOPT (design)** | The literal TV remote: fixed `RemoteControl` verbs + push updates + state; proves a remote daemon needs no planner |
| 5 | [`huginn/huginn`](https://github.com/huginn/huginn) | MIT | 2026-11-08 | Watchdog agent | **ADOPT (design)** | "Agents that monitor and act": event graph, schedules, dry-run, self-hosted; the historic watchdog-agent contract, still active |
| 6 | [`node-red/node-red`](https://github.com/node-red/node-red) | Apache-2.0 | 2026-11-06 | Switch / control | **ADAPT** | Deterministic flow runtime with persisted flows and `switch`/`inject` semantics — the switch model without an LLM |
| 7 | [`fleetdm/fleet`](https://github.com/fleetdm/fleet) | MIT (+ open-core `ee/`) | 2026-11-08 | Enrolment | **ADOPT (design)** | Enrolment → node identity → scheduled/distributed query → result: the remote-daemon identity loop |
| 8 | [`nsqio/nsq`](https://github.com/nsqio/nsq) | MIT | 2026-08-11 | Bounded queue | **ADOPT (design)** | `--mem-queue-size` + transparent disk overflow + `#ephemeral` drop: the clearest bounded-delivery contract |
| 9 | [`slawlor/ractor`](https://github.com/slawlor/ractor) | MIT | 2026-11-06 | Rust supervision | **ADOPT (lib candidate)** | Erlang-`gen_server`-style actors + supervision in Rust; the only credible way to get supervision primitives without a second scheduler |
| 10 | [`openai/codex`](https://github.com/openai/codex) | Apache-2.0 | 2026-11-08 | Agent host | **ADAPT** | The best current evidence for a local agent process: thread/turn state, sandboxed tools, approval modes (host shape; internals not read) — a host design, not a daemon |

**Runner-ups:** `micromdm/nanomdm` (minimal enrolment state machine), `riverqueue/river` (queue in
the same SQLite/Postgres), `temporalio/temporal` (durable turns), `tower-rs/tower` +
`boinkor-net/governor` (Rust admission control), `prymitive/karma` (alert grouping UI),
`AnalogJ/scrutiny` (one-job watcher product shape).

---

## 1. Decision frame and what "second pass" means

### 1.1 The decision this lane supports

| Field | Value |
|---|---|
| Decision | For the P1 "Remote/Watchdog role/cards" work, which **new** open-source projects should HiveMind study, and for each: adopt, adapt, or reject — with a licence-clean path and a concrete field-level donor map? |
| Owner | Operator (Riley), via the HiveMind Lead. |
| Deadline | Feeds P1 in `09-synthesis-and-goal.md` §7; not P0. |
| "Good enough" | 25+ projects that wave 2 did not catalogue, each with live-verified URL, licence, last activity, architecture, tier, fit, verdict, and risk; a top-10 shortlist; licence obligations; anti-patterns; 20 answered questions; and a donor map that a Rust implementer can follow without re-deciding the design. |
| Non-goal | Product edits, migrations, UI code, new dependencies added to `Cargo.toml`, benchmarks, or legal advice. |

### 1.2 What wave 2 covered, and the seven holes this pass fills

`18-open-source-agents.md` catalogued 49 repositories: 24 agent-side frameworks and 25 daemon-side
monitoring projects (Healthchecks, Uptime Kuma, Gatus, Alertmanager, osquery, Telegraf,
node_exporter, OTel Collector, Salt, Apprise, ntfy, Gotify, Diun, changedetection.io, Watchtower,
monit, Supervisor, Beszel, windows_exporter, WinSW, Vigil, watchexec, sysinfo, Netdata,
healthdog-rs). That is a good *brand* list. It leaves seven mechanical holes:

1. **The supervision lineage is missing.** `s6`, `runit`, `daemontools`, `immortal`, `dumb-init`
   and `process-compose` are the projects that actually implement "one job forever". Wave 2 named
   `Supervisor` and `healthdog-rs` but not the grammar: readiness, restart policy, one
   supervisor per service, no self-supervision.
2. **Enrolment has a concept but no implementation.** Wave 2 had osquery's `node_key` and Salt;
   it did not name the projects whose *entire product* is enrol → identify → schedule → report
   (Fleet, NanoMDM, Tailscale, Syncthing).
3. **Bounded delivery is one row deep.** Wave 2 had the OTel Collector's sending queue. The
   sharper contract is NSQ's `--mem-queue-size` + disk overflow + `#ephemeral` drop, and the
   durable-queue alternatives (River, Temporal, NATS JetStream, `tower`/`governor`/`flume`).
4. **"TV-remote" was a label, not a donor.** Wave 2 treated it as a property of a daemon. The
   literal donors — `pyatv` for Apple TV, `scrcpy` for Android devices, `node-red` for
   switch/flow semantics, `home-assistant` for entity/service calls, `openHAB` for rules — were
   not catalogued.
5. **Alert *lifecycle and dedupe* beyond Alertmanager is missing.** Alerta, Karma, Keep, Robusta
   and OneUptime each decompose "the same alarm keeps firing" differently; only Alertmanager was
   covered.
6. **Rust-native primitives are two libraries deep.** Wave 2 had `sysinfo` and `watchexec`. The
   supervision/admission layer needs `ractor`, `tower`, `governor`, `flume` and `sd-notify`.
7. **Agent-native *hosts* are missing.** Codex, DBOS, PydanticAI and mcp-agent are about running
   and resuming agent work, not about defining agent teams. That is the relevant donor class for
   "a daemon turn survives a crash", and it is absent from wave 2.

### 1.3 Live-repo gap check (READ, 2026-11-08)

The target shape is unchanged from `12-role-capability-model.md`; this is the short evidence that
the daemon half is still greenfield at HEAD `bd575ba`:

| Check | Command / path | Result |
|---|---|---|
| No daemon concept in the engine | `Select-String backend/crates/*/src/*.rs -Pattern 'daemon|watchdog'` | **0 matches** |
| Node kinds are four | `backend/crates/hiverouter-contract/src/event.rs:240` | `["hyperagent","superagent","worker","tool"]` — no `subagent`, no `daemon` |
| Persona tiers are four | `backend/crates/hiverouter-core/src/prompts.rs:370-379` | `Hyper, Super, Worker, Tool` |
| No daemon persona | `backend/crates/hiverouter-core/assets/prompts/personas.json` | 7 records: `planner`, `content-writer`, `superagent`, `subagent`, `data-analyst`, `code-reviewer`, `chatbot`; none is a daemon/watchdog/remote |
| No daemon tables | `backend/crates/hiverouter-core/src/db.rs` `CREATE TABLE` scan | 14 tables (`runs`, `tasks`, `agent_messages`, `agent_memory`, `agent_profiles`, …); no `daemons`, `daemon_reports`, `daemon_turns`, `daemon_audit` |
| Caps a daemon would collide with | `backend/crates/hiverouter-core/src/caps.rs:9-15` | depth 3, fan-out 8, 64 agents/run, 40 calls/run |
| A "watchdog" that exists is a UI asset gate only | `web/src/assets/manifest.ts:2-4` | FigCraft asset watchdog ledger — unrelated to runtime daemons |

**What that means for donor selection:** HiveMind does not need a *framework* to become a daemon
host. It needs (a) a durable daemon record, (b) a scheduler lane that is not the operator run,
(c) a report schema with dedupe and a bound, (d) a control table, (e) an enrolment identity, and
(f) a bounded queue for delivery. Every item in this catalog is scored against one of those six
artifacts, not against "is it a cool agent project".

---

## 2. Method and evidence bar

### 2.1 Freshness rule (declared before searching)

Sample time **2026-11-08, 03:30Z–04:15Z**. Relative to that instant:

- **CURRENT** — last push ≤ 90 days;
- **SLOWING** — 90–365 days;
- **STALE** — > 365 days, or archived/unmaintained banner.

`pushed_at` from the GitHub REST API is a **push** timestamp, not a commit timestamp. It is labelled
"last push" everywhere in this document; no row claims a commit date. Release feeds were not
sampled in this pass, so no release number is asserted.

### 2.2 Source hierarchy and claim labels

| Label | Source | Used for |
|---|---|---|
| **A** | GitHub REST `repos/{owner}/{repo}` (licence SPDX, `pushed_at`, archived) + repository `README.md` at `HEAD` read in this session | Licence, activity, architecture mechanism |
| **B** | GitHub REST metadata + repository description only | Licence, activity, one-line purpose |
| **D** | Official project documentation page read in this session (`s6`, `process-compose`, `NSQ`, Alerta docs index) | Mechanism claims (readiness fd, probes, mem-queue-size, heartbeat/lifecycle) |
| **L** | `LICENSE` file at `HEAD` read in this session | Open-core splits and custom licences |
| **READ** | HiveMind working tree at `bd575ba` | HiveMind-side claims |
| **INFERRED** | Reasoned from A/B/D plus the HiveMind code read | Fit arguments, not vendor facts |
| **UNVERIFIED** | Anything not fetched, built, or run | Listed in §14.3 |

No blog, forum, social post, or AI-generated summary is used. Every licence was read from the
GitHub API and, for the three `NOASSERTION` repositories, from the `LICENSE` file itself. Star
counts are a snapshot and are not used as a quality argument.

### 2.3 What was actually fetched

- **34 repositories** were queried through the GitHub API (35 calls; one 404 for a moved repo):
  name, SPDX licence expression, `pushed_at`, `archived`, stars, language, description.
- **12 READMEs** were read at `HEAD` for the projects whose architecture matters most (Alerta,
  process-compose, immortal, Huginn, Node-RED, pyatv, Fleet, NanoMDM, Codex, ractor, NSQ,
  PydanticAI).
- **3 `LICENSE` files** were read because the API returned `NOASSERTION` (Keep, OneUptime, Fleet).
- **4 official documentation pages** were read for mechanism claims: `skarnet.org/software/s6/
  overview.html`, `f1bonacc1.github.io/process-compose/`, `nsq.io/overview/design.html`, and the
  Alerta 9.1 docs index (which names the "Alert Lifecycle", "Alert timeouts, heartbeats and
  Housekeeping", and "Suppressing Alerts using Blackouts" pages).
- **Not done:** no repository was cloned, built, installed, or executed; no Windows service was
  registered; no release artefacts were downloaded; no code was searched inside repositories; no
  benchmark was run. Every architecture statement is from the repository's own description/README
  or the named docs page, never from reading its source.

### 2.4 Limits stated rather than hidden

- **No independent verifier.** This lane is a delegated subagent at depth 1 (`maxDepth: 1`,
  remaining levels 0), so it cannot spawn a researcher or a verifier. Every decision-critical
  external claim was re-read in the fetched text before being written; §14.3 lists what a verifier
  should re-fetch.
- **No legal advice.** §10 states obligations as engineering constraints, not as counsel.
- **Go/Ruby/Java/Python donors cannot be code donors** for a Rust product. For those, "ADOPT"
  always means *design*, and the row says so.
- **`pushed_at` can overstate activity** (a single README push is enough). Where the distinction
  matters, the row says "push" and the top-10 risk column says how confident the maintenance read
  is.

---

## 3. Top-10 shortlist

Ranking rule: (1) does it supply a HiveMind daemon artifact that does not exist; (2) is the licence
clean for design reuse; (3) is it current; (4) can a Rust implementer act on it without adding a
runtime. Stars and fame are not inputs.

| Rank | Donor | Licence | Last push | Class | Verdict | Integration cost | Risk |
|---|---|---|---|---|---|---|---|
| 1 | [`skarnet/s6`](https://github.com/skarnet/s6) | ISC | 2026-11-07 | Supervision | **ADOPT (design)** | S (≤1 day) | Low. Unix-only concepts; the Windows equivalent is a service wrapper, not s6 code. |
| 2 | [`F1bonacc1/process-compose`](https://github.com/F1bonacc1/process-compose) | Apache-2.0 | 2026-11-03 | Supervisor + control plane | **ADOPT (design)** | S–M (1–3 days) | Low licence. Go binary; the REST surface must never be exposed unauthenticated. |
| 3 | [`alerta/alerta`](https://github.com/alerta/alerta) | Apache-2.0 | 2026-10-17 | Report / dead-man | **ADOPT (design)** | S–M | Low. Python/Mongo/Postgres is irrelevant for a design port. |
| 4 | [`postlund/pyatv`](https://github.com/postlund/pyatv) | MIT | 2026-08-14 | TV-remote | **ADOPT (design)** | S | Low. Apple protocols; only the command/state shape transfers. |
| 5 | [`huginn/huginn`](https://github.com/huginn/huginn) | MIT | 2026-11-08 | Watchdog agent | **ADOPT (design)** | S–M | Low licence. Rails app; heavy to run, so design only. |
| 6 | [`node-red/node-red`](https://github.com/node-red/node-red) | Apache-2.0 | 2026-11-06 | Switch / flows | **ADAPT** | M (2–4 days) | Medium. A flow editor can become a second programming language if the operator is allowed arbitrary code. |
| 7 | [`fleetdm/fleet`](https://github.com/fleetdm/fleet) | MIT core (+`ee/`) | 2026-11-08 | Enrolment | **ADOPT (design)** | S–M | Low licence if `ee/` is avoided; large product surface. |
| 8 | [`nsqio/nsq`](https://github.com/nsqio/nsq) | MIT | 2026-08-11 | Bounded queue | **ADOPT (design)** | S | Low. Distributed broker is heavier than HiveMind needs; take the bound/drop semantics only. |
| 9 | [`slawlor/ractor`](https://github.com/slawlor/ractor) | MIT | 2026-11-06 | Rust supervision | **ADOPT (lib candidate)** | M–L | Medium: adding actors before the admission scheduler would create two concurrency models (same warning as `11` and `18`). |
| 10 | [`openai/codex`](https://github.com/openai/codex) | Apache-2.0 | 2026-11-08 | Agent host | **ADAPT** | S (study) | Medium: OpenAI cadence and a very large surface; only the host/approval/thread patterns transfer. |

**Why these ten, in one line each**

1. **s6** — the only donor that defines a daemon as *one service, one supervisor, one readiness
   signal*, with the supervision tree and restart semantics spelled out. HiveMind's daemon record
   is a re-expression of this in SQLite; no code is needed.
2. **process-compose** — the closest single-binary product to the operator's sentence: it
   supervises non-container processes, probes liveness/readiness, restarts, schedules interval and
   cron jobs, shows a TUI, and exposes a token-authenticated REST API. That is the daemon lane plus
   the "remote" in one artefact.
3. **Alerta** — a report is not a log line; it is an alert with a lifecycle, a heartbeat, a
   timeout, and a blackout. HiveMind's `daemon_reports` should copy this shape.
4. **pyatv** — the literal TV remote: a small fixed verb set (`up/down/left/right/select/menu/
   home`), push updates, and a state read. It shows the remote daemon is a *device adapter*, not an
   agent.
5. **Huginn** — the original "agents that monitor and act", as an event graph with schedules and
   a dry-run. It is the watchdog-agent vocabulary HiveMind's persona registry lacks.
6. **Node-RED** — a deterministic event-flow runtime whose flows persist and whose `switch`/
   `inject`/`change` nodes are the switch semantics the operator means by "TV-remote".
7. **Fleet** — an enrolment credential becomes a node identity, which then receives scheduled
   queries and returns results. That is the remote daemon's identity and report loop.
8. **NSQ** — the sharpest statement of a bounded queue: `--mem-queue-size`, transparent disk
   overflow, `#ephemeral` explicitly dropping. It gives HiveMind the vocabulary for a named
   `E_QUEUE_FULL` instead of silent loss.
9. **ractor** — Erlang-style actors and supervision in Rust, with a registry and supervision
   trees. It is the only library in this catalog that could implement supervision *inside* the
   existing runtime, and it is also the easiest way to accidentally build a second scheduler, so
   the test is explicit.
10. **Codex** — the most current example of a local agent host that exposes threads, turns,
   approvals and sandboxed tools (host shape; internals not read). HiveMind's daemon turn is a
   much smaller version of the same host problem.

---

## 4. Catalog A — one-job-forever supervision (6)

| # | Repository | Licence | Last push (UTC) | What it is (evidence) | Tier | Fit for HiveMind | Verdict | Risk |
|---|---|---|---|---|---|---|---|---|
| A1 | [`skarnet/s6`](https://github.com/skarnet/s6) | ISC | 2026-11-07 | Supervision suite: `s6-svscan`/`s6-supervise`; a daemon announces **readiness by writing a newline to a chosen fd**; supervision tree with dependency ordering; `./finish`; "one service → one daemon" (D) | Daemon supervisor | The daemon lifecycle grammar: one service per supervisor, explicit readiness, supervised restart, log as a service | **ADOPT (design)** | Low. Unix concepts; Windows needs a service wrapper, not s6 code |
| A2 | [`F1bonacc1/process-compose`](https://github.com/F1bonacc1/process-compose) | Apache-2.0 | 2026-11-03 | Go single binary; orchestrates **non-container** processes; **liveness and readiness health checks**; restart; **cron and interval scheduled processes**; TUI; **REST/OpenAPI with optional token auth**; namespaces; replicas; file-watch restart (D) | Supervisor + control plane | The closest product analogue to a HiveMind daemon lane: probe model, schedule model, operator TUI, control API, per-process config | **ADOPT (design)** | Low licence. Go; REST must never be exposed unauthenticated; feature breadth is larger than needed |
| A3 | [`immortal/immortal`](https://github.com/immortal/immortal) | BSD-3-Clause | 2026-11-06 | "*nix cross-platform (OS agnostic) supervisor"; a small CLI supervisor (B) | Minimal supervisor | A minimal run/stop/restart supervisor state machine to compare against HiveMind's daemon control verbs | **ADAPT** | Unix-only in practice; small community; no Windows story |
| A4 | [`Yelp/dumb-init`](https://github.com/Yelp/dumb-init) | MIT | 2026-11-02 | "A minimal init system for Linux containers" (B) | Init / PID 1 | Child-process hygiene for daemon turns: signal forwarding and reaping when a turn spawns a tool subprocess | **ADAPT** | Linux containers only; the lesson (never orphan children) is portable, the code is not |
| A5 | [`AnalogJ/scrutiny`](https://github.com/AnalogJ/scrutiny) | MIT | 2026-11-04 | SMART monitoring, historical trends, real-world failure thresholds (B) | One-job watcher | Product shape of a one-job daemon: collector cadence → threshold evaluation → notification, with history retained | **ADAPT** | Domain-specific; Go; not a general runtime |
| A6 | [`containrrr/shoutrrr`](https://github.com/containrrr/shoutrrr) | MIT | 2026-07-01 | Go notification library with URL-style service schemes (B) | Report delivery | Report fan-out as a pure function of a URL/DSN, so a daemon never learns a protocol | **ADAPT** | SLOWING (131 days); Go; no delivery guarantee |

**What this category proves.** There is no need to invent the daemon envelope. Every serious
supervisor in the lineage agrees on the same six facts: (1) a service is a *program plus a
policy*, not an agent; (2) readiness is *signalled*, not slept for; (3) a supervisor restarts its
child, never itself; (4) restart policy is rate-limited (intensity) or it becomes a fork bomb;
(5) logs are a separate service; (6) the supervisor's own state is the source of truth, not the
child's self-report. `12-role-capability-model.md` §7 already states HiveMind's version of these
facts (`Armed/Suspended/Degraded/Faulted/Retired`, heartbeat 30 s, report 300 s, restart 3/h, no
self-control). This catalog adds the *provenance*: s6 and process-compose implemented exactly
that, and HiveMind can cite them instead of re-deriving the contract.

**Windows caveat (important, and not a licence issue).** `s6`, `runit`, `dumb-init` and `immortal`
are Unix process tools. HiveMind is a portable Windows desktop app. The semantics port; the code
does not. Wave 2 already catalogued `WinSW` (MIT) and `windows_exporter`; `process-compose` is
cross-platform and is the better comparison point for a portable supervisor. A HiveMind daemon is
an in-process scheduler record executed by the Rust engine, so it needs neither s6's `fork/exec`
nor a Windows service per daemon — one Windows service (or the app itself) hosts the scheduler.

---

## 5. Catalog B — dead-man switches, health, alert dedupe, monitor-and-act agents (6)

| # | Repository | Licence | Last push (UTC) | What it is (evidence) | Tier | Fit for HiveMind | Verdict | Risk |
|---|---|---|---|---|---|---|---|---|
| B1 | [`alerta/alerta`](https://github.com/alerta/alerta) | Apache-2.0 | 2026-10-17 | Alert aggregation server; docs index names **Alert Lifecycle**, **"Alert timeouts, heartbeats and Housekeeping"**, **Blackouts**; Postgres or MongoDB (A+D) | Report / dead-man | Report as a *state record*: lifecycle, heartbeat + timeout, blackout/mute, severity; the exact shape `daemon_reports` needs | **ADOPT (design)** | Low. Python + Mongo/Postgres irrelevant to a design port |
| B2 | [`prymitive/karma`](https://github.com/prymitive/karma) | Apache-2.0 | 2026-11-03 | "Alert dashboard for Prometheus Alertmanager" (B) | Dedupe UI | Grouping/dedupe views, silence preview, and the operator-facing list of "what is actually on fire" | **ADAPT** | Tied to the Alertmanager API; UI-only |
| B3 | [`keephq/keep`](https://github.com/keephq/keep) | **MIT outside `ee/`** (L) | 2026-11-07 | "Open-source AIOps and alert management platform" (B+L) | Alert workflow | Alert ingestion and management as a pipeline; the non-AI parts are the useful layer | **ADAPT (non-AI parts)** | Open-core `ee/`; AI correlation in the critical path; Python |
| B4 | [`robusta-dev/robusta`](https://github.com/robusta-dev/robusta) | MIT | 2026-11-06 | "Better Prometheus alerts for Kubernetes — smart grouping, AI enrichment, automatic remediation" (B) | Alert enrichment | Grouping/inhibition and the *remediation action envelope* — an alert may propose an action, never execute it silently | **ADAPT (grouping only)** | Kubernetes-specific; AI enrichment is a paid/LLM path |
| B5 | [`OneUptime/oneuptime`](https://github.com/OneUptime/oneuptime) | **Apache-2.0 outside `ee/`** (L) | 2026-11-07 | Complete monitoring/observability platform (B+L) | Incident lifecycle | Operator-facing "current state" as a projection of monitored objects; verify incident/status surfaces before citing them | **ADAPT** | Open-core; large TypeScript monorepo; heavy |
| B6 | [`huginn/huginn`](https://github.com/huginn/huginn) | MIT | 2026-11-08 | "Agents that perform automated tasks… read the web, watch for events, and take actions"; agents create and consume **events** along a **directed graph**; self-hosted; 50k★ (A) | Watchdog agent | The historic monitor-and-act contract: an agent has a schedule, a check, an event, and an action; events propagate; a dry-run is a first-class mode | **ADOPT (design)** | Rails app; too heavy to run as a dependency; Ruby |

**What this category adds beyond wave 2.** Wave 2 adopted Alertmanager's group/dedupe/inhibit/
silence and Healthchecks' "no ping ⇒ alert". That answers *when to notify*. It does not answer
*what a report is*. Alerta's decomposition is the missing half:

- **identity** — a repeat of the same condition updates one record rather than appending a new one
  (inferred from the lifecycle model; the exact key fields were not re-read);
- **lifecycle** — `new → acknowledged → closed/expired`, with the transition history retained;
- **heartbeat** — a periodic "still alive" that is *not* a state change;
- **timeout** — if the heartbeat stops, the alert escalates without a new event;
- **blackout/mute** — suppression is a separate, auditable object, not a flag on the alert.

HiveMind's `daemon_reports` in `12` already has `(daemon_id, generation, check_id, check_hash,
observed_state)` as its dedupe key. Alerta confirms that shape and adds the piece `12` leaves
implicit: a **report status** (`ok/info/warn/critical`) with an acknowledged/closed operator
transition, and a **mute** object with an expiry. Keep and Robusta are deliberately downgraded:
their correlation/AI layer is the exact "LLM in the check loop" anti-pattern in §11, and both are
open-core or Kubernetes-bound. Huginn is the one *agent-shaped* donor: it shows the watchdog agent
as a small event machine with a schedule and a dry-run, not as a chat participant.

---

## 6. Catalog C — TV-remote / switch / local-first control (5)

| # | Repository | Licence | Last push (UTC) | What it is (evidence) | Tier | Fit for HiveMind | Verdict | Risk |
|---|---|---|---|---|---|---|---|---|
| C1 | [`postlund/pyatv`](https://github.com/postlund/pyatv) | MIT | 2026-08-14 | "A client library for Apple TV and AirPlay devices"; connect to a device, issue remote-control commands, receive push updates (A) | TV-remote | The literal remote: a small fixed verb set, a state read, push updates, and a per-device connection; the `job.kind=control` button table in `12` §7.10 | **ADOPT (design)** | Apple protocols; Python; only the command/state shape transfers |
| C2 | [`node-red/node-red`](https://github.com/node-red/node-red) | Apache-2.0 | 2026-11-06 | "Low-code programming for event-driven applications"; flows are persisted and edited in a browser; `inject`/`switch`/`change` nodes; admin API (A) | Switch / flows | Deterministic switch semantics and a persisted flow definition: the operator can see and edit the rule without an LLM | **ADAPT** | A `function` node is arbitrary code; Node runtime; flows can become a second language |
| C3 | [`home-assistant/core`](https://github.com/home-assistant/core) | Apache-2.0 | 2026-11-08 | Local-first home automation; entities, services, events, automations (B) | Local control plane | The entity/service split: a *state* is read, an *action* is a service call; local-first and privacy-first by default | **ADAPT (concepts)** | Enormous domain; Python; not a donor for the runtime |
| C4 | [`Genymobile/scrcpy`](https://github.com/Genymobile/scrcpy) | Apache-2.0 | 2026-11-06 | "Display and control your Android device" over adb (B) | Device remote | A typed control channel to a device with a small server component, a fixed command set, and a clean teardown | **ADAPT** | adb dependency; one device per process; not a long-lived service |
| C5 | [`openhab/openhab-core`](https://github.com/openhab/openhab-core) | **EPL-2.0** | 2026-11-07 | Core framework of openHAB (Java) (B) | Rules / switch | Rules-as-data and item state as a comparison for HiveMind's deterministic control table (model not re-read here) | **REJECT (code), ADAPT (concepts)** | EPL-2.0 weak copyleft (file/module level); JVM |

**What "TV-remote" actually means, mechanically.** Across all five projects, a remote is:

1. a **fixed command enum** (`up`, `down`, `select`, `home`, `start`, `stop`, `mute`, `report_now`),
   not free-form text;
2. a **state read** that is separate from the command (`is the target on?`);
3. an **optimistic acknowledgement** with a timeout and a reconcile (the command was accepted, not
   necessarily applied);
4. a **capability table** per device/target, so an unsupported button is refused by name;
5. **no planner** — the operator or the owning agent chooses the button; the daemon only transports
   it.

`12` §7.10 already lands on this (`status/start/stop/mute/report_now`, deterministic, no LLM for
those verbs). `pyatv` is the best evidence that this is the whole remote; `node-red` is the best
evidence that the *rule* around the button can be a persisted data structure; `home-assistant` is
the best evidence for splitting state from service call; `scrcpy` is the best evidence for the
transport side; `openHAB` is the one licence to avoid.

---

## 7. Catalog D — enrolment and node identity (4)

| # | Repository | Licence | Last push (UTC) | What it is (evidence) | Tier | Fit for HiveMind | Verdict | Risk |
|---|---|---|---|---|---|---|---|---|
| D1 | [`fleetdm/fleet`](https://github.com/fleetdm/fleet) | **MIT outside `docs/` and `ee/`** (L) | 2026-11-08 | Open device-management platform: MDM, patching, software deployment, API/GitOps/webhooks/YAML (A) | Enrolment + reporting | Enrolment credential → node identity → scheduled/distributed query → result; policy-as-YAML and webhook delivery are directly analogous to daemon config and reports | **ADOPT (design)** | Open-core `ee/`; large product; Go |
| D2 | [`micromdm/nanomdm`](https://github.com/micromdm/nanomdm) | MIT | 2026-11-05 | "Minimalist Apple MDM server and library"; enrolment, check-in, command/report (A) | Enrolment state machine | The smallest complete enrol → check-in → command → report loop in the catalog; a reference implementation to read, not to port | **ADOPT (design)** | Apple-specific protocol; small project (706★) |
| D3 | [`tailscale/tailscale`](https://github.com/tailscale/tailscale) | BSD-3-Clause | 2026-11-07 | WireGuard-based mesh with 2FA; `tailscaled` node daemon (B) | Node identity / transport | Node keys, key rotation, NAT traversal, and a daemon that joins a private network without exposing a port | **ADAPT** | Coordination-server dependency; Go; far larger than a local app needs |
| D4 | [`syncthing/syncthing`](https://github.com/syncthing/syncthing) | **MPL-2.0** | 2026-11-05 | Continuous file synchronisation; device IDs, discovery, TLS (B) | Device identity / trust | Device identity + discovery + trust-on-first-use, all local-first and self-hosted | **ADAPT** | MPL file-level copyleft; Go; solves sync, not agent identity |

**The enrolment contract this category establishes.** A remote daemon needs five things, and every
project here implements a version of them:

1. **A one-time secret** (enrolment token/secret) that is not the long-lived credential.
2. **A stable node identity** minted at enrolment (`node_key`, device ID, node key), stored on the
   node and recognised by the owner.
3. **An owner-signed/owner-approved record** so a node cannot enrol itself into another hive.
4. **A heartbeat** that distinguishes "enrolled but offline" from "never enrolled".
5. **A scoped report** — the node reports only the checks assigned to it, not everything it can see.

Wave 2 used osquery's `node_key` as the vocabulary but did not name an implementation that is small
enough to copy. `nanomdm` is that implementation; `fleet` is the product-scale version; `tailscale`
and `syncthing` are the identity/transport extremes. For HiveMind, the minimum is: operator creates
a daemon → engine mints a one-time enrolment token → daemon instance exchanges it for a
`daemon_id`-bound key → heartbeat + scoped reports. No MDM, no mesh VPN, no file sync is required;
the *state machine* is the donor.

---

## 8. Catalog E — bounded queues and durable turns (5)

| # | Repository | Licence | Last push (UTC) | What it is (evidence) | Tier | Fit for HiveMind | Verdict | Risk |
|---|---|---|---|---|---|---|---|---|
| E1 | [`nsqio/nsq`](https://github.com/nsqio/nsq) | MIT | 2026-08-11 | Realtime distributed messaging; topics/channels; **`--mem-queue-size` bounds in-memory messages, overflow is written to disk transparently**; `#ephemeral` channels **drop** after the bound; at-least-once with requeue/timeout (A+D) | Bounded queue | The exact vocabulary for a bounded report/outbox queue: an explicit memory bound, disk overflow, an explicit ephemeral/drop mode, and at-least-once for the rest | **ADOPT (design)** | Distributed broker is heavier than a local app; 89-day push is CURRENT but the project is mature/slow |
| E2 | [`temporalio/temporal`](https://github.com/temporalio/temporal) | MIT | 2026-11-08 | Durable execution service: workflows, activities, retries, timers, heartbeats, task queues (B) | Durable turns | A daemon turn as a durable workflow: idempotency key, retry policy, timer, heartbeat, resume after crash | **ADAPT (concept)** | Heavy server; adopting it would be a second scheduler next to `RunEngine` |
| E3 | [`riverqueue/river`](https://github.com/riverqueue/river) | **MPL-2.0** | 2026-11-07 | "The polyglot queue: fast, reliable background jobs on Postgres/SQLite"; unique jobs, retries (B) | Durable queue | A job queue *in the same SQLite database* as the daemon record: no new server, transactional enqueue, at-least-once | **ADAPT** | Go library; MPL file-level; SQLite backend maturity not verified here |
| E4 | [`nats-io/nats-server`](https://github.com/nats-io/nats-server) | Apache-2.0 | 2026-11-06 | High-performance pub/sub with JetStream persistence, limits and consumers (B) | Event/report bus | Per-subject stream limits (age/bytes/messages) and consumer acks for report delivery | **ADAPT** | Extra server; operational surface; overkill for one desktop |
| E5 | [`dbos-inc/dbos-transact-py`](https://github.com/dbos-inc/dbos-transact-py) | MIT | 2026-11-04 | "Database-backed durable Python workflows"; steps and recovery (B) | Durable turns | The smallest statement of "a turn is a row plus steps; replay from the last completed step" — portable to Rust/SQLite | **ADAPT (concept)** | Postgres dependency; Python |

**What this category settles.** `12` §7.5 requires "never more than one in-flight turn per daemon"
and a bounded report history, and `10-gap-meta.md` flags the missing collision/bounded-load
daemon. NSQ supplies the clearest contract for what happens at the bound: messages spill to disk
until the bound, and an explicitly *ephemeral* channel drops instead of blocking. HiveMind's
version should be a bounded `mpsc` (or a SQLite outbox with a hard row cap) plus a **named refusal**
(`E_QUEUE_FULL` / `E_REPORT_BACKLOG`) — never a silent drop, never unbounded growth. River and
DBOS are the two projects that show how to keep the queue **in the same database** as the state;
that matters because HiveMind already runs SQLite and the operator wants everything persisted.
Temporal and NATS are the "do not add a server" warnings: they solve the problem by adding a
second system to operate, which the local-desktop product cannot absorb.

---

## 9. Catalog F — Rust primitives and agent-native daemon hosts (8)

| # | Repository | Licence | Last push (UTC) | What it is (evidence) | Tier | Fit for HiveMind | Verdict | Risk |
|---|---|---|---|---|---|---|---|---|
| F1 | [`tower-rs/tower`](https://github.com/tower-rs/tower) | MIT | 2026-06-22 | Composable async `Service` middleware: concurrency limits, load shedding, timeouts, rate limits, buffering (B) | Rust admission | Compose the daemon lane from `ConcurrencyLimit` + `LoadShed` + `Timeout` instead of hand-rolling semaphores; the failure mode is a named `Overloaded` error | **ADOPT (lib)** | SLOWING (139 days); abstraction cost; tower is a toolkit, not a scheduler |
| F2 | [`boinkor-net/governor`](https://github.com/boinkor-net/governor) | MIT | 2026-08-24 | Rust rate-limiting library (GCRA), quotas, keyed limits (B) | Rust rate limits | Per-daemon and per-provider token buckets with burst control; a cheap way to stop a 5-minute daemon from becoming a 5-second one | **ADOPT (lib)** | Small project; rate limiting only |
| F3 | [`zesterer/flume`](https://github.com/zesterer/flume) | Apache-2.0 | 2026-10-15 | Safe MPMC channels, bounded and unbounded, async/select (B) | Rust queue | A bounded in-process report queue with a blocking/`try_send` refusal path | **ADOPT (lib, or std/tokio mpsc)** | Low; do not add a dependency where `tokio::sync::mpsc` already suffices |
| F4 | [`slawlor/ractor`](https://github.com/slawlor/ractor) | MIT | 2026-11-06 | Pure-Rust actor framework inspired by Erlang `gen_server`; supervision and registry (A) | Rust supervision | Supervision trees, named actors and message passing if the daemon lane needs them; a possible implementation substrate, not a product dependency yet | **ADOPT (lib candidate)** | Medium: a second concurrency model beside the serial engine (same caution as `18` §5.2.5) |
| F5 | [`lnicola/sd-notify`](https://github.com/lnicola/sd-notify) | Apache-2.0 | 2026-03-09 | Rust crate for systemd `sd_notify` readiness/watchdog messages (purpose inferred from crate name; repo description empty) | Rust watchdog | If HiveMind ever runs as a Linux service, readiness/watchdog must use the platform protocol, not a sleep | **ADAPT** | SLOWING (244 days); Linux only; 54★; tiny surface |
| F6 | [`openai/codex`](https://github.com/openai/codex) | Apache-2.0 | 2026-11-08 | Local coding agent in Rust; terminal/IDE/app surfaces (A) | Agent host | How a local agent process exposes threads/turns, tool sandboxing, approvals and state — the closest current model for a daemon turn's host | **ADAPT** | OpenAI product cadence; very large surface; not a daemon supervisor |
| F7 | [`lastmile-ai/mcp-agent`](https://github.com/lastmile-ai/mcp-agent) | Apache-2.0 | 2026-01-25 | "Build effective agents using Model Context Protocol and simple workflow patterns" (B) | Agent framework | MCP as the tool boundary and workflow patterns as data | **REJECT (SLOWING, 287 days)** | Stale; do not depend on it; read only if MCP workflow semantics are needed |
| F8 | [`pydantic/pydantic-ai`](https://github.com/pydantic/pydantic-ai) | MIT | 2026-11-08 | Typed agent framework: "every model, every interface", typed end to end (B) | Agent contracts | Typed tool/result schemas and the graph-of-steps idea; durability is an integration, not a core (not verified here) | **ADAPT (concept)** | Python; durability needs external infrastructure (not verified) |

**What this category rejects.** The wave-2 conclusion stands and is strengthened: the agent
frameworks are **not daemon hosts**. `mcp-agent` is nine months without a push; Codex is an agent
product, not a supervisor; PydanticAI is a typed agent framework, not a durable-execution service
(its durability story is outside this catalog's verification). What *is* usable is the
layer below them: `tower` for admission, `governor` for rate, `flume` for a bounded channel,
`ractor` for supervision if needed, and Codex's **host shape** (thread, turn, approval, sandbox)
as a design reference for `daemon_turn` execution.

---

## 10. Licence blocks

This section is an engineering constraint list, not legal advice. Every licence below was read
from the repository's GitHub API metadata and, where marked, from its `LICENSE` file.

### 10.1 Licence families present in this catalog

| Family | Repos in this catalog | What reuse obliges (plain terms) | Code-safe for HiveMind? |
|---|---|---|---|
| **MIT** | pyatv, Huginn, dumb-init, scrutiny, shoutrrr, nsq, temporal, tower, governor, ractor, dbos, pydantic-ai, nanomdm, Robusta, Fleet core, Keep core | Keep the copyright notice and permission text in copies/substantial portions; no warranty; no endorsement | Yes |
| **Apache-2.0** | process-compose, node-red, home-assistant, scrcpy, nats, alerta, karma, flume, sd-notify, codex, mcp-agent, OneUptime core | Keep the licence, keep any `NOTICE`, **state changes**, patent grant included, trademark not granted | Yes, with NOTICE hygiene |
| **BSD-3-Clause** | immortal, tailscale | Keep copyright + the three clauses; do not use names for endorsement | Yes |
| **ISC** | s6 | Keep copyright + permission notice; otherwise equivalent to MIT | Yes |
| **MPL-2.0** | syncthing, river | **File-level copyleft**: modifications to MPL-covered files must be available under MPL; larger works may combine; notices required | Yes, if files are not modified/vendored; keep MPL files separate |
| **EPL-2.0** | openhab-core | **Weak copyleft at module level**; source for modified EPL code; patent grant; static linking questions are why this is usually avoided | Not recommended for code reuse |
| **CC BY-SA 4.0** | Fleet `docs/` only | Attribution + share-alike for the documentation files | Docs only; do not copy into product text without attribution |
| **Open-core custom** | Keep (`ee/`), OneUptime (`ee/`), Fleet (`ee/`, `docs/`) | The `ee/` directory is under a separate commercial licence; the rest is MIT/Apache as stated | Only the non-`ee/` part; never vendor `ee/` |

### 10.2 The `NOASSERTION` trap

GitHub returned `NOASSERTION` for **Keep**, **OneUptime** and **Fleet**. That is not "no licence".
All three `LICENSE` files are **open-core**: the root licence applies to everything except an
`ee/` directory (and, for Fleet, `docs/` under CC BY-SA 4.0). Consequences:

- `license.spdx_id` cannot be used as the audit result; the `LICENSE` text is the audit result.
- Any code copied from these repos must record **which path** it came from and prove it is outside
  `ee/`.
- A dependency scanner that only reads the SPDX field will misclassify all three. Record the
  exception in the repository's licence inventory (`cargo-deny`/`cargo-about` config for Rust,
  plus a manual note for reference-only repos).

### 10.3 Rejected licence classes (carried forward and extended)

- **AGPL-3.0 / GPL-2.0 / GPL-3.0** — not present in this catalog (wave 2 rejected `monit`,
  `netdata`, `hyperbrowserai/HyperAgent`, `ntfy` server for this reason). No code from any
  copyleft project belongs in the product's binary.
- **CC BY-NC-SA 4.0** — wave 2 rejected `facebookresearch/HyperAgents`; the same rule applies to
  any research artefact: literature only.
- **"Sustainable Use" / source-available** — not catalogued here; check before use.

### 10.4 What HiveMind must do before any code reuse

1. Add the donor to a licence inventory with `repo`, `path`, `commit`, `SPDX`, `obligation`.
2. For MIT/BSD/ISC: include the notice in `THIRD-PARTY-NOTICES`.
3. For Apache-2.0: include the licence and `NOTICE`; mark files changed.
4. For MPL-2.0: keep MPL files in their own files; publish modifications to those files.
5. For EPL-2.0: prefer no code; if unavoidable, treat the module boundary as a legal boundary.
6. For open-core: prove the path is outside `ee/`; never copy from `docs/` under CC BY-SA into
   product text.
7. Rust dependencies: run `cargo deny check licenses` in CI and pin the exact crate version
   (operating contract §28, dependency discipline).

**Licence-clean path for this lane:** everything recommended as **ADOPT (design)** copies no code,
so no obligation is triggered beyond attribution of ideas in this document. The only rows that
recommend a *dependency* are `tower`, `governor`, `flume`, `ractor` (candidates) and `sd-notify`
(adapt); all five are permissive and would still need the inventory above.

---

## 11. Anti-patterns (with the donor that exposes each one)

| # | Anti-pattern | Why it is a defect in HiveMind | Where it is visible | Rule |
|---|---|---|---|---|
| X1 | **An LLM in the check loop** | A watchdog that calls a model to decide "is it down?" is neither cheap nor deterministic; a prompt-injected page can flip the verdict, and a provider outage becomes a monitoring outage | Keep ("AIOps"), Robusta ("AI enrichment"), any "AI SRE" pitch | The check and the control verbs are deterministic. A model may only *summarise an already-computed state*, after the state is persisted (`12` §7.9, DA-1…DA-10) |
| X2 | **Unbounded report queue** | A 5-minute daemon that produces a report per tick is 288 rows/day per daemon; at 50 daemons it is 14,400/day, and an outage catch-up can multiply it | OTel Collector (wave 2) had to add a sending queue; NSQ exists because unbounded memory is fatal | Hard cap + SQLite outbox + named `E_QUEUE_FULL`; one in-flight turn per daemon (`12` §7.5) |
| X3 | **Self-supervision / daemon-of-daemon cycles** | A watchdog inside the thing it watches deadlocks; a daemon that can restart itself is an autonomy escape | s6's "one service → one daemon"; Erlang supervisor lesson cited in `12` §7.8 | Target graph acyclic; target may not be owner/ancestor/self; kill/restart are operator requests, never in the daemon's grant |
| X4 | **Sleep-based readiness** | `sleep 5` is a guess; it fails under load and lies under fast start | s6 readiness fd; process-compose readiness probes | A daemon turn signals ready; the scheduler waits on the signal with a timeout, never on a fixed sleep |
| X5 | **Remote = generic shell over HTTP** | A "remote" endpoint that executes arbitrary commands is a remote-code-execution surface, not a TV remote | pyatv's fixed verbs; HA's service calls; scrcpy's typed control messages | Fixed `RemoteAction` enum + per-target capability table; no `exec` route; approval in modes 1–2 |
| X6 | **Vendoring from `ee/`** | The most useful part of an open-core product is often the paid part; copying it is a licence violation, not a shortcut | Keep/OneUptime/Fleet `LICENSE` | Record path + commit; only non-`ee/` code; `cargo-deny` plus manual inventory |
| X7 | **"AI correlation" as the alert path** | Correlation by LLM hides missed alerts behind a plausible summary and usually needs a paid API | Keep, Robusta | Deterministic grouping/inhibition (Alertmanager, wave 2) + Alerta-style lifecycle; AI only as an offline annotation |
| X8 | **A framework as the daemon host** | Importing an agent framework to run "every 5 minutes" adds a model loop, a scheduler, and a dependency with its own release cadence | mcp-agent (stale), PydanticAI (durability is an integration, not core), Codex (agent product) | The daemon is a record + a short turn executed by `RunEngine`; the framework is a design reference, not a runtime |
| X9 | **Thundering herd / cron at `:00`** | 50 daemons scheduled at the same minute spike the provider and the DB; the operator sees a stall and blames the engine | Telegraf (wave 2) has interval jitter for exactly this | Interval + jitter + one catch-up; no backfill; stagger by `daemon_id` hash |
| X10 | **Self-reported health** | A process that heartbeats while doing nothing is "alive" and useless; self-report can be forged by the same bug that broke the work | s6 (supervisor observes), process-compose (probes), `12` §7.6 | Health is computed by the scheduler from heartbeat age + last turn outcome + report freshness; a daemon cannot report itself healthy |
| X11 | **A "remote" daemon that is really a second agent** | If the remote can plan, it can do things the operator did not press; the card becomes a lie | pyatv/HA/scrcpy are all deterministic adapters | The remote has no planner and no tools beyond its button table; requesting a new button is a config change, not a model decision |
| X12 | **Silent drop under pressure** | Losing a report is worse than refusing it, because the operator cannot tell the difference from "all clear" | NSQ makes the drop mode explicit (`#ephemeral`); HiveMind's refusal discipline is named codes (`caps.rs:49-71`) | Every bound has a refusal code; persisted reports survive delivery failure; retry twice, then mark `delivery_failed`, never delete |

---

## 12. Twenty questions

**Q1. What did wave 2 miss that actually changes the build?**
Three mechanics, not brands: (a) the **supervision grammar** (s6: one service, one supervisor,
readiness signal, supervision tree); (b) **enrolment as a state machine** (Fleet/NanoMDM: secret →
node key → scheduled query → result); (c) **bounded delivery with an explicit drop mode** (NSQ:
`--mem-queue-size`, disk overflow, `#ephemeral`). Wave 2 named the *categories*; this pass names the
*implementations* and the fields to copy.

**Q2. Is there one daemon framework HiveMind should adopt?**
No. There are three layers, and no single project covers all three: **supervision** (s6,
process-compose), **report lifecycle/dedupe** (Alerta, Alertmanager), **delivery bounds** (NSQ,
River). Adopting any one as "the daemon framework" would either add a Go/Ruby runtime or a second
scheduler. HiveMind builds the thin Rust layer and imports the three contracts.

**Q3. Why not just run process-compose and register each daemon as a process?**
Because a HiveMind daemon is not an OS process: it has an owner, a grant, a budget, a card, a
report schema, and an approval policy. process-compose has no concept of an agent, a hive, or a
policy. It is the best **design donor for the lane and the control plane**, not a runtime HiveMind
can embed (it is a Go binary). The transferable parts are the probe model, restart policy,
interval/cron schedule, TUI vocabulary, and the token-authenticated REST shape.

**Q4. Is s6 relevant to a Windows Rust app at all?**
Yes, as **semantics**, not code. s6's contract — one service per supervisor, readiness notified on
an fd, a supervision tree, restart policy, log as a service — is platform-independent. HiveMind's
daemon is a SQLite record executed by the engine, so the Windows question is only "who hosts the
scheduler": the desktop app (a single process) or a Windows service (`WinSW`, wave 2). The
per-daemon `fork/exec` model is explicitly *not* imported.

**Q5. What is the minimal daemon record?**
`daemon_id` (stable forever), `generation`, `owner` (HyperAgent/SuperAgent), `job` (immutable:
kind `watch|control`, target, check id, cadence), `job_hash`, `state`
(`Armed|Suspended|Degraded|Faulted|Retired`), `switch` (operator-only), `restart_window`,
`budget_bucket`, `last_heartbeat_at`, `last_report_at`, `next_run_at`, `profile_version`,
`target_terminal_policy`. This is `12` §7.2 plus the enrolment key (`node_key`) and the job hash.
Everything else is a report or a turn.

**Q6. How should the schedule work?**
Interval seconds + jitter (deterministic from `daemon_id`), `next_run_at` persisted, one in-flight
turn per daemon, and **exactly one catch-up** after downtime (`12` §7.5). No backfill: a 40-minute
outage produces one "missed N windows" report, never N reports. A cron expression is acceptable as
a *config surface* only if it compiles to the same `next_run_at` model; process-compose supports
both interval and cron, which is why it is the donor for the config schema.

**Q7. How are reports deduplicated?**
Three keys, in order: (a) **identity** — `(daemon_id, generation, check_id)`; (b) **change** —
`check_hash` + `observed_state`; (c) **status** — `ok|info|warn|critical` plus an operator
`acknowledged|closed` transition. Identical consecutive reports update the existing row's
`last_seen_at` and increment `repeat_count` instead of inserting. Alerta's alert lifecycle is the
donor; Alertmanager's group/inhibit rules (wave 2) are the notification-side complement. Do not
dedupe by message text.

**Q8. What is the dead-man rule, exactly?**
A daemon has a heartbeat (scheduler-written, 30 s in `12`) and a report cadence (300 s or change).
If `now - last_heartbeat_at > 2 × heartbeat` the scheduler marks `Degraded`; if
`now - last_report_at > report_s + grace`, it marks `Faulted` and emits one fault report to the
owner. **A missed report is an event, not a turn**: the scheduler does not spawn a check to ask why
the check did not run. Alerta's "timeouts, heartbeats and housekeeping" is the model; Healthchecks
(wave 2) is the same idea from the opposite direction ("no ping arrived").

**Q9. How does a remote daemon enrol?**
Operator creates the daemon → engine mints a one-time enrolment token with an expiry and a target
scope → the remote process exchanges it for a `daemon_id`-bound key and a generation → the owner
approves the first heartbeat (or auto-approves if the token was operator-created) → only then are
checks assigned. Fleet's enrolment flow, NanoMDM's check-in, and Tailscale's node key are three
implementations of the same five-step contract (§7). HiveMind does **not** need MDM or a mesh VPN.

**Q10. What is a TV-remote daemon, mechanically?**
A `job.kind=control` daemon with a fixed button table (`status`, `start`, `stop`, `mute`,
`report_now`), a per-target capability list, a state read, an acknowledgement with timeout, and no
planner. `pyatv` is the literal evidence (fixed `RemoteControl` verbs + push updates); Home
Assistant supplies the state/service split; Node-RED supplies the persisted rule that decides
*when* a button is pressed. The operator presses, or a deterministic rule presses; the model does
not.

**Q11. Should a daemon ever use an LLM?**
Only to **summarise an already-persisted state**, never to decide whether a check passed, whether
a target is up, or whether to press a button. The operator's "no hard budget" makes an LLM loop
tempting; it is also the denial-of-wallet path. If summarisation is enabled, it is an explicit
per-daemon setting with its own budget bucket and a bounded output (`E_REPORT_TOO_LARGE` if it
exceeds the schema).

**Q12. How is the report queue bounded?**
Two bounds: in-memory (`tokio::sync::mpsc` capacity, e.g. 256) and persisted (SQLite outbox with a
per-daemon row cap, e.g. 1,000). `try_send` failure and outbox overflow both return
`E_QUEUE_FULL`/`E_REPORT_BACKLOG`; the turn is marked `degraded`, not silently dropped. This is
NSQ's `--mem-queue-size`/disk/ephemeral contract translated to a single process. Delivery failure
keeps the row and marks `delivery_failed` after two retries.

**Q13. Do we need River, Temporal, or NATS?**
No. River and DBOS are useful *concepts* (queue in the same database; turn as steps), but adding a
Go/Python service to a portable Windows desktop is the wrong trade. Temporal and NATS are
"second systems to operate". HiveMind already has SQLite; a `daemon_turns` table with an
idempotency key and a bounded outbox is the same guarantee at the product's scale. Revisit only if
the S4–S5 stages prove SQLite contention.

**Q14. Do we need a durable workflow engine for daemon turns?**
No, but the turn must be **crash-safe**: a turn row is written before the check starts, the check
result is written before the report, and the report is written before delivery. On boot, a turn in
`running` is marked `interrupted` and retried at most once under the restart budget. That is the
useful part of DBOS/Temporal without the engine.

**Q15. Is `ractor` a good fit?**
As a **candidate library**, yes: it is Rust, MIT, active, Erlang-inspired, and has supervision and
a registry. As an architecture, it is a risk: adding actors before the admission scheduler exists
creates two concurrency models (`18` §5.2.5 makes the same point about AgentScope). The test is
narrow — one actor type for `DaemonSupervisor` with one child per daemon, no actor ever executing a
model call, and no actor-owned queue. If that test cannot be met, use `tower` + a plain scheduler
loop and skip `ractor`.

**Q16. What can HiveMind learn from Codex?**
The **host shape**, not the agent: a local process with explicit threads/turns, tool calls that
pass through an approval mode, sandboxed execution, and state that can be inspected/resumed. That
maps to `daemon_turn` (turn id, state, tool calls, approval, result) and to the UI activity row.
Codex is not a supervisor: it has no immutable-job record, no owner-scoped reports, no daemon
lifecycle. Copy the host discipline; reject the product model.

**Q17. What is the health model?**
Four computed states — `healthy`, `degraded`, `faulted`, `unknown` — computed by the scheduler from:
heartbeat age, last turn outcome, last report freshness, restart window usage. A daemon cannot set
its own health. process-compose's liveness/readiness split is the probe vocabulary; s6's
readiness-not-self-report is the principle; `12` §7.6 is the product rule.

**Q18. How do we test a daemon subsystem?**
At minimum: (1) job immutability refused by name; (2) exactly one catch-up after a simulated
40-minute outage; (3) restart limit trips on the 4th restart in an hour; (4) kill is terminal and a
retired id never resurrects; (5) one in-flight turn under a slow check; (6) queue-full returns a
named refusal and persists nothing silently; (7) a muted daemon keeps checking but does not notify;
(8) a target-ancestor daemon is refused; (9) report dedupe updates `repeat_count`; (10) a daemon
cannot switch/kill/restart itself. These are the `12` RC tests plus the queue and mute tests from
this catalog.

**Q19. Which licences actually block reuse?**
EPL-2.0 (`openhab-core`) is the only code-level block in this catalog; MPL-2.0 (`syncthing`,
`river`) is usable if files are kept separate and modifications published. The `NOASSERTION`
open-core trio (Keep, OneUptime, Fleet) is usable **outside `ee/`** only, and the path must be
recorded. Everything else recommended here is MIT/Apache-2.0/BSD-3/ISC and design-only. See §10.

**Q20. What is the P1 implementation order?**
1. `daemons` + `daemon_turns` + `daemon_reports` tables and refusal codes (from `12` §7.12 and
   its refusal-code table).
2. Scheduler lane with one in-flight turn per daemon, interval+jitter, one catch-up.
3. Report schema + dedupe + bounded outbox (`E_QUEUE_FULL`), delivery retry twice.
4. Control enum + capability table + operator switch (no self-control).
5. Enrolment token → `node_key` → owner approval (for remote daemons only).
6. Card projection (`daemon` kind, switch, health dot, last/next report) and audit rows.
7. Only then: optional summarisation with its own budget bucket.

---

## 13. Concrete HiveMind daemon donor map

This is the implementable output: one row per field/behaviour, the donor that settles it, the Rust
artifact, the decision, and the test that proves it. Field names follow `12` §7.2/§7.12; nothing
here adds a tier or a second scheduler.

| # | Field / behaviour | Donor | HiveMind artifact | Decision | Acceptance test |
|---|---|---|---|---|---|
| 1 | Daemon identity, generation, retirement | s6 "one service → one daemon"; `12` §7.2/§7.4 | `daemons(daemon_id, generation, state, created_at, retired_at)`; `E_DAEMON_RETIRED` | ADOPT | Retired id refuses every operation; restart increments generation, same id |
| 2 | Immutable job | s6 service definition; A2A task immutability (`12`) | `job` JSON + `job_hash`; update returns `E_DAEMON_JOB_IMMUTABLE` | ADOPT | Changing cadence/target refused; new daemon id required |
| 3 | Enrolment (remote only) | Fleet enrolment flow; NanoMDM check-in; Tailscale node key | `daemon_enrollment(token_hash, expires_at, scope, used_at)`; `node_key` on the daemon | ADAPT | Expired/reused/foreign-scope token refused; first heartbeat owner-approved |
| 4 | Schedule | process-compose interval/cron; Telegraf jitter (wave 2) | `interval_s`, `jitter_s`, `next_run_at`; deterministic jitter from id | ADAPT | 50 daemons do not start in the same second |
| 5 | Readiness | s6 readiness fd; process-compose readiness probe | `turn_ready` event written by the turn; scheduler waits with timeout | ADOPT | Slow-start check does not burn the turn timeout; no fixed sleep in the scheduler |
| 6 | Heartbeat | Alerta heartbeat/timeout; Healthchecks (wave 2) | `last_heartbeat_at` written by the scheduler, never by the model | ADOPT | A hung turn still heartbeats; a dead scheduler does not |
| 7 | Health | process-compose liveness/readiness; `12` §7.6 | Computed `healthy/degraded/faulted/unknown`; stored on the record | ADOPT | Daemon cannot set its own health; 2 missed heartbeats → `degraded` |
| 8 | Restart intensity | immortal; process-compose; `12` §7.7 | Sliding window `max 3/h`; `E_DAEMON_RESTART_LIMIT`, state `faulted` | ADAPT | 4th restart in an hour refused; operator must clear |
| 9 | Report shape | Alerta alert model | `daemon_reports(report_id, daemon_id, generation, check_id, check_hash, status, observed_state, summary, repeat_count, first_seen_at, last_seen_at, ack_at, closed_at)`; `E_REPORT_TOO_LARGE` | ADOPT | Report over the schema bound is refused whole, not truncated |
| 10 | Report dedupe | Alerta lifecycle; Alertmanager grouping (wave 2) | Unique `(daemon_id, generation, check_id, check_hash, observed_state)`; identical repeats update `repeat_count` | ADOPT | Two identical ticks produce one row with `repeat_count=2` |
| 11 | Mute / blackout | Alerta blackout | `mute_until`, `muted_by`, `mute_reason`; muted = persist, do not notify | ADOPT | Muted daemon keeps checking; notifications suppressed; unmute resumes |
| 12 | Control buttons | pyatv `RemoteControl`; Home Assistant service calls | `RemoteAction {Status, Start, Stop, Mute, ReportNow}`; per-target capability table; `E_DAEMON_NO_CAPABILITY` | ADOPT | Unsupported button refused by name; no free-form command |
| 13 | Remote transport | scrcpy typed control channel; pyatv push updates | Request/ack envelope with timeout; optimistic UI; reconcile on next state read | ADAPT | Ack timeout shows `unknown`, not success |
| 14 | Report/outbox queue bound | NSQ `--mem-queue-size` + disk + `#ephemeral` | Bounded `mpsc` (256) + SQLite outbox (1,000/daemon); `E_QUEUE_FULL` / `E_REPORT_BACKLOG` | ADOPT (design) | Full queue refuses with a code; nothing silently dropped |
| 15 | Admission | tower `ConcurrencyLimit`/`LoadShed`/`Timeout`; `12` daemon lane (bounded permits) | Daemon lane: 1 in-flight per daemon, 2 global; paused during operator run | ADOPT (lib) | Slow check does not start a second turn; global cap holds |
| 16 | Rate limiting | governor GCRA | Per-daemon and per-provider quotas; burst ≤ 2 | ADOPT (lib) | A 5-second cadence cannot exceed the configured quota |
| 17 | Durable turn | River/DBOS turn-as-row; Temporal heartbeats (concept) | `daemon_turns(turn_id, daemon_id, generation, idempotency_key, state, started_at, ended_at)`; `running` → `interrupted` on boot | ADAPT | Kill mid-turn: no partial success; one retry at most |
| 18 | Supervision tree (optional) | ractor | One `DaemonSupervisor` actor only if the narrow test passes; no actor executes model calls | ADOPT (conditional) | No second scheduler; actor tree never owns the queue |
| 19 | Audit | Huginn event log; Alerta history | Append-only `daemon_audit(actor, action, daemon_id, before, after, at)` | ADAPT | Create/switch/kill/restart/mute all produce a row |
| 20 | Card / operator surface | process-compose TUI; `12` §12 | `daemon` card: switch, health dot, last report age, next report, activity rows (no new card per turn) | ADOPT | 50 daemons × 288 turns/day do not create 14,400 cards |
| 21 | Delivery | shoutrrr/Apprise (wave 2) | Channel DSN; retry twice; then `delivery_failed`; report row stays | ADAPT | Channel down loses no report row |
| 22 | Operator remote API | process-compose token-auth REST | `GET/POST /api/daemons/{id}/{switch,restart,kill,report-now}`; localhost + token + Host allowlist | ADAPT | Unauthenticated request refused; no generic exec route |

**How to read this map.** Rows 1–2 are the identity contract; 3 is the only remote-only row; 4–8
are the supervisor; 9–11 are the report model; 12–13 are the remote; 14–18 are reliability; 19–22
are operator surfaces. Every row is implementable in the existing Rust engine and SQLite schema; no
row requires a broker, a container, an external service, or an LLM. That is the point of the
second pass: the daemon tier is a *small, boring, well-specified subsystem*, and the donors that
prove it are boring too.

---

## 14. Claim register, conflicts, and gaps

### 14.1 Claim register

| # | Claim | Evidence | Class |
|---|---|---|---|
| 1 | This catalog adds 34 repositories not present in `18-open-source-agents.md` | Name-by-name comparison against `18` §4.1–4.2 (49 rows) | **VERIFIED (read)** |
| 2 | All 34 rows have licence + last-push metadata | GitHub REST `repos/{owner}/{repo}` queried 2026-11-08 | **VERIFIED (API)** |
| 3 | The backend has zero daemon/watchdog runtime references | `Select-String backend/crates/*/src/*.rs -Pattern 'daemon|watchdog'` → 0 | **VERIFIED (command)** |
| 4 | Node-kind vocabulary is four values, no daemon/subagent | `event.rs:240` | **VERIFIED (read)** |
| 5 | Persona registry has 7 records, no daemon | `personas.json` parse | **VERIFIED (read)** |
| 6 | Database has 14 tables, none daemon/report/turn/audit | `db.rs` `CREATE TABLE` scan | **VERIFIED (read)** |
| 7 | s6 requires a readiness notification on an fd and models "one service → one daemon" | `skarnet.org/software/s6/overview.html` | **VERIFIED (docs)** |
| 8 | process-compose has liveness/readiness probes, restart, interval/cron, TUI, token-auth REST | `f1bonacc1.github.io/process-compose/` | **VERIFIED (docs)** |
| 9 | NSQ bounds memory with `--mem-queue-size`, spills to disk, and `#ephemeral` drops | `nsq.io/overview/design.html` | **VERIFIED (docs)** |
| 10 | Alerta documents alert lifecycle, heartbeat/timeout/housekeeping, and blackouts | Alerta 9.1 docs index | **VERIFIED (docs index)** |
| 11 | Keep/OneUptime/Fleet are open-core (`ee/`) and GitHub reports `NOASSERTION` | `LICENSE` files at HEAD | **VERIFIED (read)** |
| 12 | `mcp-agent` last push 2026-01-25 (~287 days) → SLOWING | GitHub API | **VERIFIED (API)** |
| 13 | `tower` last push 2026-06-22 (~139 days) and `shoutrrr` 2026-07-01 (~131 days) → SLOWING | GitHub API | **VERIFIED (API)** |
| 14 | Codex is a Rust local agent with CLI/app/IDE surfaces | GitHub API language + README | **VERIFIED (API+README)** |
| 15 | The wave-2 `02` X13 conflict with the operator's daemon requirement is resolved as a service/lease, not a long-lived agent | `10-gap-meta.md` §X5; `12` §0/§7; this catalog's supervision model | **VERIFIED (read)** for the conflict; **DESIGN** for the resolution |
| 16 | Donor mechanics improve HiveMind's daemon correctness | Argument from docs + `12` contract | **INFERRED**, to be proven by the §13 tests |

### 14.2 Conflicts, and how they are resolved

1. **`02` X13 ("reject cross-run agent identity / long-lived agents") vs the operator's daemon
   requirement.** `10-gap-meta.md` records this as unresolved (X5). `12` resolves it by making the
   daemon a one-job *office with short turns*, not a long-lived agent mind. This catalog supports
   that resolution from the outside: s6/process-compose/immortal all supervise **processes**, not
   agents, and none of them keeps a conversation alive. The daemon record is durable; the turn is
   short; no model context survives between turns.
2. **Alerta identity model vs Alertmanager grouping.** They are not competitors. Alertmanager
   groups and inhibits *notifications*; Alerta stores an alert as a *state record*. HiveMind needs
   both: identity/lifecycle in `daemon_reports`, grouping/inhibition in the notifier. Adopt both,
   in different layers.
3. **Keep/Robusta "AI correlation" vs the operator's no-paid-API, deterministic requirement.**
   Reject the AI path; keep the provider ingestion (Keep) and grouping (Robusta) only. This is a
   conflict between a vendor feature and an operator constraint; the operator wins.
4. **process-compose's REST control plane vs HiveMind's local HTTP boundary.** process-compose
   offers optional token auth; HiveMind's current boundary lacks a Host allowlist
   (`09` §1 Q18). A daemon control API is a *new mutation surface*; it must ship with token + Host
   allowlist + localhost bind, or it becomes the worst vulnerability in the product.
5. **`18` says `11-donor-frameworks.md` does not exist.** It exists in the working tree now
   (134,737 bytes). This is corpus drift in the wave-2 file, not a fact about the current tree;
   this lane uses `18` only as the catalog to extend.
6. **"Adopt" a Go/Ruby/Java/Python donor.** In every such row, ADOPT is explicitly *design*
   adoption (no code, no licence obligation). The only dependency rows are the Rust crates in §9.

### 14.3 What is not verified (and would settle it)

- **No repository was built, installed, or run.** A verifier should clone `s6`,
  `process-compose` and `alerta` and exercise the §13 tests' mechanics (readiness, probe, report
  dedupe) before any design is frozen.
- **No Windows behaviour was tested.** process-compose cross-platform claims, WinSW hosting, and
  the desktop app's service story are all unverified on this machine.
- **`pushed_at` is a push date, not a commit date.** A verifier should read `commits/HEAD.atom`
  for the ten shortlisted repos and correct any SLOWING/CURRENT label that changes.
- **Alerta dedupe specifics** are inferred from the docs index and the alert model; the exact
  "Alert Format"/"API" pages returned 404 at the guessed URLs. A verifier should fetch
  `docs.alerta.io` via its navigation and quote the identity fields.
- **Codex internals** (thread/turn, sandbox, approvals) are inferred from the product/README; the
  internal docs path was not fetchable. Do not treat the host shape as a code-level fact.
- **MPL/EPL obligations** are stated as engineering constraints, not legal advice; counsel should
  confirm before any code reuse.
- **Star counts** are a snapshot and are not used as evidence of quality.
- **No independent verifier ran** (depth-1 subagent, no delegation); §14.1's external claims were
  re-read in the fetched text, but a second reader has not checked them.

---

## 15. Sources

All fetched **2026-11-08 (UTC)**. Repository URLs are the canonical `https://github.com/...`
pages; metadata came from `https://api.github.com/repos/{owner}/{repo}`.

**Primary documentation (mechanism claims)**

- s6 overview — <https://skarnet.org/software/s6/overview.html>
- process-compose docs — <https://f1bonacc1.github.io/process-compose/>
- NSQ design — <https://nsq.io/overview/design.html>
- Alerta 9.1 docs index — <https://docs.alerta.io/>

**READMEs read at HEAD**

- <https://github.com/alerta/alerta>
- <https://github.com/F1bonacc1/process-compose>
- <https://github.com/immortal/immortal>
- <https://github.com/huginn/huginn>
- <https://github.com/node-red/node-red>
- <https://github.com/postlund/pyatv>
- <https://github.com/fleetdm/fleet>
- <https://github.com/micromdm/nanomdm>
- <https://github.com/openai/codex>
- <https://github.com/slawlor/ractor>
- <https://github.com/nsqio/nsq>
- <https://github.com/pydantic/pydantic-ai>

**Licence files read at HEAD (open-core check)**

- <https://raw.githubusercontent.com/keephq/keep/HEAD/LICENSE>
- <https://raw.githubusercontent.com/OneUptime/oneuptime/HEAD/LICENSE>
- <https://raw.githubusercontent.com/fleetdm/fleet/HEAD/LICENSE>

**Every catalogued repository** (metadata verified; see §4–§9 for the licence/activity values)

- A: <https://github.com/skarnet/s6>, <https://github.com/F1bonacc1/process-compose>,
  <https://github.com/immortal/immortal>, <https://github.com/Yelp/dumb-init>,
  <https://github.com/AnalogJ/scrutiny>, <https://github.com/containrrr/shoutrrr>
- B: <https://github.com/alerta/alerta>, <https://github.com/prymitive/karma>,
  <https://github.com/keephq/keep>, <https://github.com/robusta-dev/robusta>,
  <https://github.com/OneUptime/oneuptime>, <https://github.com/huginn/huginn>
- C: <https://github.com/postlund/pyatv>, <https://github.com/node-red/node-red>,
  <https://github.com/home-assistant/core>, <https://github.com/Genymobile/scrcpy>,
  <https://github.com/openhab/openhab-core>
- D: <https://github.com/fleetdm/fleet>, <https://github.com/micromdm/nanomdm>,
  <https://github.com/tailscale/tailscale>, <https://github.com/syncthing/syncthing>
- E: <https://github.com/nsqio/nsq>, <https://github.com/temporalio/temporal>,
  <https://github.com/riverqueue/river>, <https://github.com/nats-io/nats-server>,
  <https://github.com/dbos-inc/dbos-transact-py>
- F: <https://github.com/tower-rs/tower>, <https://github.com/boinkor-net/governor>,
  <https://github.com/zesterer/flume>, <https://github.com/slawlor/ractor>,
  <https://github.com/lnicola/sd-notify>, <https://github.com/openai/codex>,
  <https://github.com/lastmile-ai/mcp-agent>, <https://github.com/pydantic/pydantic-ai>

**Internal sources (read in-workspace)**

- `docs/research/hmars/18-open-source-agents.md` — wave-2 catalog of 49 repos (the list this file
  extends; no overlap).
- `docs/research/hmars/12-role-capability-model.md` — daemon contract, refusal codes, tables,
  card schema; the target shape.
- `docs/research/hmars/10-gap-meta.md` — X5 conflict and the missing collision/bounded-load daemon.
- `docs/research/hmars/14-a2a-bus-roster.md` — remote/watchdog control plane and message bounds.
- `docs/research/hmars/09-synthesis-and-goal.md` — P1 work and the "not coded" daemon finding.
- `OPERATOR-DECISIONS.md` — one-job-forever, report-every-5-minutes, TV-remote, no paid APIs.
- Live code anchors: `event.rs:240`, `prompts.rs:370-379`, `caps.rs:9-15`, `db.rs` table scan,
  `assets/prompts/personas.json`.

---

## 16. Lane self-report

### Done

| Item | Evidence |
|---|---|
| 34 new candidate repositories catalogued, zero overlap with wave 2 | §4–§9; each row has URL, licence, last push, architecture, tier, fit, verdict, risk |
| Licence and last-push verified for all 34 | GitHub REST API, 2026-11-08; `NOASSERTION` resolved by reading 3 `LICENSE` files |
| Top-10 shortlist with rationale, cost, risk | §3 |
| Licence blocks and open-core trap | §10 |
| 12 anti-patterns | §11 |
| 20 questions answered | §12 |
| Field-level donor map (22 rows) | §13 |
| Live-repo gap check | §1.3 (commands + `event.rs:240`, `prompts.rs:370`, `db.rs`, `caps.rs`) |
| Claim register, conflicts, unverified list | §14 |
| No product file edited | This file only; no repo clone/build/test |

### Not done

| Item | Reason |
|---|---|
| Cloning, building, or running any donor | Research-only lane; no toolchain for Go/Ruby/Java/Python donors, and no product change is authorised |
| Reading release feeds / version numbers | Not needed for the design decision; asserting a version without the feed would be invented |
| Code-level reading of donor internals | Only README/docs were read; architecture claims are labelled A/B/D and §14.3 says so |
| Independent verification pass | Depth-1 subagent, `maxDepth: 1`, no delegation available |
| Windows service experiment | No daemon code exists to host yet; would be product work |
| Legal review | §10 is engineering guidance, not counsel |

### Unverified

| Claim / change | What would prove it |
|---|---|
| Any runtime behaviour of s6/process-compose/Alerta as a HiveMind analogue | Clone and run the §13 tests against a real daemon lane |
| process-compose REST auth on Windows | Start it locally, call the API with/without token, verify binding |
| Alerta's exact dedupe fields | Fetch the Alerta "Alert Format"/"Alert Lifecycle" pages through the docs navigation |
| Codex host internals | Read `codex-rs` source at a pinned commit |
| River SQLite backend maturity | Read its docs/changelog at a pinned release |
| Licence obligations for MPL/EPL | Counsel review before any code reuse |
| Maintenance labels from `pushed_at` | `commits/HEAD.atom` per shortlisted repo |

**Budgets (for the delegation record):** subagents used 0/7 (depth 1, `maxDepth: 1`, remaining
levels 0); background jobs 0; model-request concurrency not measured (no ceiling exposed to this
lane); 35 GitHub API metadata calls (34 successful; 1 404 for a moved repo), 12 README fetches,
3 licence fetches, 4 docs fetches — all listed in §15.

**Bottom line:** the daemon tier does not need an agent framework. It needs the supervision
grammar of `s6`, the probe/control shape of `process-compose`, the report lifecycle of `Alerta`,
the enrolment loop of `Fleet`/`NanoMDM`, the bounded-delivery contract of `NSQ`, and the Rust
admission primitives of `tower`/`governor`. The TV-remote is a fixed button table (`pyatv`,
`node-red`), not a planner. Everything else in this catalog is either a design reference or a
licence to avoid.
