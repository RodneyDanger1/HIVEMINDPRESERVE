# Operator decisions — second-wave research input

Date: 2026-11-07. These are the operator's answers to the 36 planning questions. They are
requirements/decision inputs, not proof. Research lanes must test them for risk and report
conflicts.

## Product and HMARS

- Target: polished **local desktop** HiveMind, one operator, Rust localhost server, portable
  Windows app, FigCraft visual quality.
- HMARS means **HiveMind Multi-Agent Runtime System**: HyperAgent → SuperAgents → Agents →
  Subagents, plus Remote/Watchdog daemon agents, with A2A, memory banks, and observable UI.
- Scale: design/test **50–150 active** first, staged to **300–600 total tasks**. Never claim a
  higher number than the stage proves.
- Priority: agent hierarchy + memory/A2A first; then DeepSeek completeness, scale, and UI polish.

## Autonomy, approvals, settings

- Autonomy must be **settings/modes 1–4**, default mode 1. The operator wants more autonomy but
  reliability/safeguards matter.
- Approval policy: all options must be settings; default is side effects only (write, shell,
  network, spend). No silent bypasses.
- Agent capabilities editable in settings, validated/audited, default enabled.
- Budget: **no hard default budget**; token caching is the goal. Budget can be switched on.
  A visible soft spend guard/emergency stop is acceptable; denial-of-wallet must still be
  designed out.
- Retention/visibility/blindness are human-only settings and must be auditable.

## Agents and templates

- Every agent, including tiny Watchdog/Remote agents, must have a UI card.
- Remote/Watchdog agents are deliberately simple switches/daemons: one job forever, report to
  their owner, low complexity (e.g. "watchdog this hive and report every 5 minutes").
- Default agents should be code-defined; built-in HiveMind templates can exist in a settings
  menu, clearly labeled. Custom agents/templates are created/edited through settings and chat;
  no uncontrolled third-party marketplace by default.
- HyperAgent/SuperAgent may choose or spawn an appropriate built-in persona when useful, subject
  to settings.
- Default hierarchy: HyperAgent → SuperAgents → Agents → Subagents; Remote agents may sit under
  HyperAgent or a SuperAgent.
- SuperAgent ownership is switchable: department/domain owner, branch lead, or single complex
  task. Research more ownership models.
- Chat is operator UI; whether it also hosts a conversational SuperAgent is deferred until
  research/donor analysis. Do not silently turn it into an agent before the decision.

## Tools and execution

- Workspace read/list, write/patch, shell, network fetch/search and provider files/search are all
  desired; computer/browser use is desired if collisions/sandboxing can be controlled.
- Design a HiveWide collision/usage enforcement daemon or equivalent: warn/reject duplicate tool
  use, watch CPU/RAM/tool load, log and prevent crashes. Sandboxing/sand pits are in scope.
- All agent A2A must support mute/blind/invisible settings; tree communication can be disabled.
- A2A topology: any-agent mesh is allowed by settings, but the framework default is the safest
  useful tree/sibling mode; private topic buses are optional.
- Cross-hive SuperAgent messaging is allowed by settings; the default should be the most
  auditable mode (HyperAgent/approved bridge).
- Bus: bounded event/notice bus, with optional pub/sub topics; avoid raw group chat.
- Messages crossing a hive or exposing sensitive data require approval; ordinary parent/sibling
  chatter does not.

## Memory

- Memory must support **global, per-run, per-hive, per-branch, and per-role** scopes; the
  operator describes a connected Venn-diagram cluster, not one flat store.
- SuperAgents get their own banks; children can tap the tree they spawned under; sharing across
  branches should be possible when useful.
- HyperAgent may read all banks **only if leakage/noise is prevented**; summaries/audited reads
  first, raw access only as an explicit setting.
- Storage: SQLite structured rows + summaries first; local vector index and external vector DB
  are future options; design all four paths if possible.
- Thought storage: summaries default; raw reasoning and none are settings.
- Retention: run lifetime + human-approved project persistence; all retention settings are
  human-only and audited.
- Memory writes must be provenance/trust aware; prompt-injected content must not become
  authoritative memory.

## A2A and UI

- A2A UI: dedicated topology/memory inspector by default; agent drawer, canvas overlays, and
  chat summaries are all settings.
- Main view: hierarchy + live execution + memory/A2A edges; all projections (canvas, chat,
  graph, dashboard) are settings, all enabled by default; make it beautiful and allow hiding.
- Agent cards: role, task, state, cost, tools, branch by default; metric/message density is a
  setting. All tiers get cards.
- Design source: extend the current FigCraft system; it can grow. Research option 4 but default
  to extension.

## DeepSeek/API decisions

- DeepSeek only. No Anthropic key. Anthropic-format docs are research only.
- Files: research/recommend; default yes only for vision/image use, not repo files.
- Search & Fetch: research/recommend; agents should be able to use them when needed, gated by
  approval/budget.
- Responses API: research/recommend; Chat Completions remains the likely canonical path.
- FIM/Prefix: research/recommend; not required for the product now.
- Vision: important later, text/code first.
- Token caching is a first-class goal: stable prefix units, measured cache hits, cross-agent
  reuse where correct; never claim "better than DeepSeek's harness" without measurement.

## Scale, failure, persistence

- Scheduler: per-hive fair admission queue default; all queuing/reject/spawn-all modes settings.
- Retry: bounded retry then branch failure, HyperAgent decides; operator escalation option.
- Persistence: everything — runs, events, messages, memory, approvals, artifacts; ephemeral
  runs only as an explicit setting.
- 300–600 total tasks requires hierarchical planning and real local infrastructure; not just
  provider permits or Rust speed.
