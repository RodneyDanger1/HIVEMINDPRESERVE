# HiveMind UI/UX + HMARS gap audit

Status date: 2026-11-08  
Audited commit: `d8afbd5` (UI pass; latest gate below)
Design sources: `00 · Cover.pdf` / `docs/ui/figcraft/ASSETS/hivemind-cover.png`, accepted FigCraft boards, `web/src/tokens/tokens.ts`, `web/src/styles/tokens.css`, `docs/ui/figcraft/ASSETS-INDEX.json`, `ledgers/ui-watchdog.json`  
Offline only: no server, browser, provider call, or live FigCraft generator was used.

This is not a completion claim. It is a current-tree inventory plus implementation pseudocode for what is still absent. The source of truth is the repository and the gate receipts, not the older plan prose.

---

## 0. First principles and claim boundaries

1. **Verified by tool** means a command was run and its output observed. The gate in §2 is verified this way.
2. **Read from a file** means the code or asset was opened in this session. File references below are read evidence.
3. **Believed from history** means it comes from the 80-round DSH session ledger; it is recorded separately and never promoted to verified fact without a new run.
4. **Visual fidelity is unverified.** The cover was inspected as an image; no browser screenshot, DOM geometry, colour sampling, pixel diff, or FigCraft regeneration occurred. The token module and accepted boards are the design authority.
5. **No FigCraft generator exists in this workspace.** `figcraft` / `figcraft-cli` is not on PATH; the repository has accepted SVG/PNG exports, a catalog, and a QA asset checker. We can design and implement surfaces in the FigCraft-derived token language, but we cannot honestly claim to have *generated new FigCraft boards*.
6. **Scale is intentionally excluded** from this document's implementation scope (50–150 / 300–600), except to mark the boundary.

---

## 1. What actually exists (not missing)

### 1.1 Backend runtime

| Area | Present now | Evidence |
|---|---|---|
| Frozen contract | `hiverouter.v1`, typed commands/events/refusals, golden schema and round-trip tests | `backend/crates/hiverouter-contract/` |
| HTTP | 36 routes: health, diagnose, session, history, agents, messages, memory, A2A pending, approvals, templates, workspace, active run, profiles, usage, detail, snapshot context, projects, provider | `http.rs::build_router` |
| WS | hello/replay/resync, command dispatch, fan-out, origin/token gate | `ws.rs` |
| Planner/DAG | plan insertion, parent chain, readiness, caps, rewire, replay | `planner.rs`, `graph.rs`, `execution.rs` |
| Tiers/classes | Hyper/Super/Worker/Subagent, class-may-spawn matrix, persisted class/rank/profile trigger, strict unknown-role refusal | `execution.rs`, `db.rs` v13/v17 |
| Root synthesis | hierarchical/flat multi-node synthesis turn, answer-only, rejects tool calls, run digest in run bank | `execution.rs` Phase 3; `provider_executor.rs::finalize_root` |
| A2A | ask_parent/send, durable rows, deadline, operator answer CAS, status, pending route, boot expiry, hello replay, continuation updates/cursor | `execution.rs`, `store.rs`, `http.rs`, `ws.rs`, `recovery.rs` |
| Memory | v14 banks/entries, dual-write, run/own/parent read-set, trust, digest, v15 journal, v16 snapshots, v18/20 hash chain + head, scoped clear, inactive prune, operator promotion | `db.rs`, `store.rs`, `execution.rs`, `http.rs` |
| Crash/recovery | snapshots, boot recovery, interrupted-run settlement, A2A expiry | `snapshot.rs`, `recovery.rs` |
| Provider | DeepSeek-only config, chat/stream/tools/models/balance, retry/refusal matrix, budget preflight | `config.rs`, `provider_executor.rs`, `hiverouter-provider/` |
| Tools | jailed read/write/list, approval ledger, shell allowlist, watcher, side effects | `hiverouter-tools/`, `tools_bridge.rs` |
| DB | schema v20, pre-migration verified backups, newer-schema refusal, `db.backup`/`db.restore` | `db.rs`, `db_admin.rs`, `cli.rs` |
| Tests | 552 backend tests / 43 suites / 2 ignored; 191 web tests; typecheck/build 0 | gate receipt §2 |

### 1.2 Frontend surfaces

Current navigation: Workspace (workspace/swarm/routing/history tabs), Agents, Data, Models, Routing, Settings.  
Current panels include: canvas, chat, scrubber, control bar, library, header, nav rail, agent hierarchy/cards, hierarchy roster, A2A inbox, messages, profiles, memory, memory banks, run detail, run transcript, history, data, models, routing, swarm, projects, settings, diagnostics, FigCraft gallery, approvals, toasts, state pack.

New in this pass:

- `A2AInboxPanel.tsx`: global durable open-question inbox, answer submit, refresh, in-flight guard.
- `DiagnosticsPanel.tsx`: `/diagnose` summary, raw payload, workspace paths, active-run projection.
- `RunTranscriptPanel.tsx`: renders hydrated historical messages in the History surface.
- `SettingsPanel.tsx`: five tabbed sections with icons, keyboard tablist, motion slider, palette, diagnostics, FigCraft.
- `ConfirmDialog.tsx`: token-styled destructive confirmation, focus restoration, Escape.
- `A2AInboxPanel`/`RunDetail`/`MemoryBanks` normalized to token radii and button language.
- Dynamic insertable template catalog from `/api/templates`.
- Chat status chip, live styling, streaming surface.
- Figure-eight ambient bees around the hive; sheen opacity; traveling edge bees with fallback.
- Design parity tests now cover the new operator surfaces.

---

## 2. Verified gate

Command run at `3f5ec41a6146d3774568b5f9c5cb9173537cd40c`:

```text
cargo test --workspace --locked
  552 passed / 0 failed / 2 ignored across 43 suites, exit 0

npm run typecheck      exit 0
npm test               191 passed / 0 failed
npm run build          exit 0
node --check tools/hivemind-cli.mjs  exit 0
```

Receipt: `docs/evidence/gate-d8afbd54-offline.json`.

Graph check: `detect-changes` reported **high** risk, 16 changed symbols, 8 affected flows (`AppFrame`, `ChatCore`, `BeeEdge`, `SettingsPanel`, etc.). `impact HiveField` returned `UNKNOWN` (not indexed), so it is not evidence of safety. Full gate is the stronger signal here.

Stub sweep on changed files: only input `placeholder=` attributes matched the generic sweep; no TODO/FIXME/NotImplemented/mock production path. Those matches are legitimate DOM input placeholders, not unfinished implementations.

No browser, no `/api` server, no live provider, no FigCraft generation.

---

## 3. FigCraft / HiveMind design source audit

### 3.1 The visual contract read from the source

The cover image establishes:

- warm paper field (`canvas`/`panel`), not dark chrome;
- a five-lobe metallic honey hive with a soft specular sheen;
- warm honey-gold accents and thin honeycomb lines;
- ink typography, with an editorial serif display and restrained sans metadata;
- small bee glyphs as routing/identity markers, not generic icons;
- white cards with hairline borders and a single warm-ink shadow family;
- pill/10px/14px geometry, not a Bootstrap/Tailwind default look.

The repository token module encodes the 20 printed swatches, eight printed type steps, spacing, radii, elevation, focus corrections, and layout geometry. That is the only non-hex source. Components must not sample pixels from boards.

### 3.2 What "custom made in FigCraft" can and cannot mean here

```pseudo
function create_figcraft_surface(request):
    if figcraft_cli_available():
        board = figcraft.generate(board_schema, token_refs, interaction_spec)
        accepted = watchdog.accept(board)
        catalog.add(accepted)
        return accepted
    else:
        # truthful fallback in this workspace
        require request.uses_existing_tokens
        require request.does_not_claim_new_board
        implement_with(React + token_module + shared_primitives)
        register_in_design_parity_test
        mark_visual_fidelity = UNVERIFIED
```

There is no executable FigCraft generator in this workspace. New components were therefore built from the accepted token/geometry vocabulary. If the requirement is literally "produce new FigCraft boards," that requires the owner's FigCraft toolchain or accepted SVG exports added to the evidence tree; it cannot be faked from React.

### 3.3 Design rules applied to new surfaces

```pseudo
Card      := surface-panel + radius-card(14) + hairline + shadow-e1
Button    := radius-button(10) + type-label + honey hover + focus-visible ring
Input     := radius-sm(8) + hairline + panel/panel-alt
Section   := surface-panel + radius-md(12) + p-3
Meta      := type-mono-meta + text-meta (ink-500)
Primary   := type-body-l / type-mono-code + ink-900
Active tab:= honey-100 surface + honey-500 border + ink-900
Motion    := slow, decorative-only, reduced-motion aware, no layout animation
```

### 3.4 Residual visual inconsistencies still present

These are real, not fixed by this pass:

1. **Arbitrary text sizes** remain widely used (`text-[8px]`, `text-[8.5px]`, `text-[9px]`, `text-[10.5px]`, `text-[11.5px]`, `text-[12.5px]`, `text-[26px]`). The printed scale has eight steps; many surfaces use interstitial values. A full pass should map these to semantic steps or add named type utilities, then re-run design parity.
2. **Inline SVG icon set** is duplicated across `SettingsPanel`, `A2AInboxPanel`, `ConfirmDialog`, `Header`, `ControlBar` etc. There is no shared `Icon` primitive with one stroke weight/viewBox. Settings currently has a local icon switch; that should move to a shared design-system module.
3. **`LibraryPanel`** still uses arbitrary heights (`h-[30px]`), small text sizes (`text-[8px]`, `text-[9px]`, `text-[10.5px]`), and `text-[11px]`; its active-agent chips are custom rather than the shared card/token primitives.
4. **History aside** is 300px and now contains run detail, transcript, and revisions. On narrow desktops this stack is cramped; a proper layout should be a resizable split pane or a run-detail drawer, not three vertical sections in a fixed rail.
5. **Canvas motion** uses both CSS offset animation and SVG SMIL:
   - `AmbientBee` uses `<animateMotion>`;
   - ambient-off hides it with CSS `display:none` but does not actually stop the SMIL clock;
   - no `document.visibilitychange` pause for offscreen canvas.
   These are performance/consistency issues, not fabricated data.
6. **`motion-hive-sheen`** was made more visible (`opacity: 0.82`) but its visual result is unverified; pixel fidelity is still `UNVERIFIED`.
7. **Chat** now has a live chip and streaming outline, but the transcript still has no per-message copy/retry, no collapsible tool details, no scroll-lock affordance when the user scrolls up, and no empty-state illustration beyond the hive glyph.
8. **Confirmation dialog** has focus restoration and Escape, but not a full focus trap; Tab can leave the modal. `aria-modal` is present but the background is not `inert`.
9. **Settings tabs** are keyboard-arrow selectable, but there is no visual active-indicator transition beyond colour; no responsive compact tab strip under 480px.
10. **A2A inbox** is global, not filtered by selected run, and offers no search/status filter/history. It is a working operator queue, not a triage console.
11. **Palette chips** in Settings are display-only. They expose token order but are not clickable/copyable.
12. **No visual regression harness**: no axe, no Playwright, no screenshots, no pixel diff, no keyboard-only run. Design claims remain code-level.

---

## 4. Missing backend / HMARS work (pseudocode)

The following are absent or partial in the audited tree. Each block states the invariant, state, failure modes, and acceptance evidence.

### 4.1 Root HyperAgent orchestration loop

Current truth:

- `RunEngine` plans first, executes children, then calls `finalize_root` once.
- `execute_task_inner` calls `executor.finalize_root` when `root_final = true`.
- If a root reply contains tool calls, the task fails with `ProviderInvalidResponse`.
- Single-node flat plans do not necessarily take the root path (`has_aggregators || plan.nodes.len() > 1`).
- Root reads run bank/digest, not branch summaries.

Missing: a true root HyperAgent that can monitor, query, spawn, answer, re-plan, and terminate, with a policy that prevents late mutation after final answer.

```pseudo
type RootState = {
  run_id: RunId
  phase: "plan" | "monitor" | "synthesize" | "terminal"
  final_answer_committed: bool
  budget: { calls_left, children_left, tokens_left }
}

async function run_root_hyperagent(run):
  state = RootState(run.id, "plan")
  plan = await planner.plan(user_prompt)
  persist_plan(plan)
  dispatch_ready_children(plan)                 // bounded by scheduler
  while not all_children_terminal(plan):
     state.phase = "monitor"
     observe = read_graph + read_branch_summaries + read_a2a_status
     reply = await executor.root_tool_turn(observe, root_toolset)
     for call in reply.tool_calls:
        authorize_root_call(call, state, caps)    // named refusal, never clamp
        apply(call)                               // spawn/kill/query/send/answer
        persist_event_and_snapshot(call)
     if no calls and no active children:
        state.phase = "synthesize"
        final = await executor.root_synthesis(observe, run_digest)
        if final.tool_calls: refuse E_ROOT_TOOL_AFTER_FINAL
        commit_root_answer(final)
        state.final_answer_committed = true
        return
  return synthesize_from_results()               // if no monitor path implemented
```

Required invariants:

- Root may not mutate after `final_answer_committed`.
- Root tools are a named subset; unknown tool = named refusal.
- Every root spawn goes through the same class/depth/fan-out/ownership checks as a model child.
- Root monitor turns consume the same call/token budget as children.
- A flat/single-node policy is explicit: either always root, or flat refuses by name.

Acceptance tests missing: root tool call in monitor, late mutation refusal, root budget exhaustion, flat-plan policy, crash mid-root.

### 4.2 SuperAgent branch lead / owned sub-DAG

Current truth: SuperAgent is a role/tier; class-may-spawn allows agent/subagent; it can execute a task and spawn via tool. It does not own a persisted branch bank, branch plan, branch budget, or branch rollup.

```pseudo
async function lead_branch(super_task, parent_goal):
  require class(super_task) == "super"
  require owner_task_id == super_task.id and ownership_mode == "branch"
  hive_bank = ensure_bank(kind="hive", owner=super_task.id, parent=run_bank)
  branch_plan = await planner.plan_branch(
    goal = parent_goal,
    allowed = {agent, subagent},
    budget = branch_budget
  )
  persist_children(branch_plan)
  dispatch_children_bounded(branch_plan)
  results = await join_children(branch_plan)
  summary = bounded_summary(results)
  write_bank_entry(hive_bank, kind="summary", value=summary, trust="unverified")
  return summary
```

Missing: branch budget/ownership enforcement, branch summary/bank, branch-level kill/cancel, and card/panel.

### 4.3 TaskRecord identity and authority

Current truth: v13 trigger persists class/rank/profile; v17 persists ownership; `TaskRecord` has only role/depth/state/rev. Engine often re-derives class from role, and task ownership is read from the DB.

```pseudo
struct TaskRecord {
  id, run_id, parent_task_id, role, depth, spawn_seq, state, rev
  class: Class
  rank: u8
  profile_id: ProfileId
  ownership: OwnershipMode
  ownership_scope: ScopeId
  owner_task_id: Option<TaskId>
  persona_revision: Option<u32>
}

function authorize(task, action):
  identity = task.class ?? lookup_persisted_identity(task.id)
  if identity is null: return Refusal.E_PERSONA_UNKNOWN
  require policy(identity.class, action) == allow
  require ownership_allows(task.ownership, action)
```

Missing: fields, constructor/serde migration, authorization by record, list/tests.

### 4.4 A2A parent-model answer

Current truth: `ask_parent` writes a row, sets deadline, waits on an in-process waiter, and only the operator HTTP answer can wake it. Boot expiry prevents stale open rows. Parent model is not asked.

```pseudo
async function ask_parent(child, parent, text):
  row = insert_message(
    kind="ask_parent", status="open",
    from=child.id, to=parent.id,
    deadline=now + 120s,
    text=bounded(text)
  )
  publish(AGENT_MESSAGE, row)
  ensure_parent_inbox_delivery(parent, row.id)   // durable, not only in-process
  return await wait_for_answer_or_deadline(row.id)

async function parent_answer(parent, message_id, text):
  require parent.class in {super, hyper}
  require parent is active and owns recipient relationship
  answer = validate_bounded(text)
  cas_open_to_answered(message_id, actor=parent.id, answer)
  publish(A2A_ANSWER, message_id)
  journal(answer)
```

Missing:

- parent tool/action to answer a child question;
- a persisted/durable waiter or child replay path;
- parent answer UI/action;
- answer-attempt/rate limit;
- separate answer table if the interim `agent_messages` columns are not accepted.

### 4.5 A2A attempt/rate/idempotency

```pseudo
function answer_attempt(run, message, actor, text):
  row = lock_or_tx(message)
  if row.status != "open": return E_TASK_TERMINAL
  if row.attempts >= MAX_ANSWERS: return E_ANSWER_LIMIT
  row.attempts += 1
  if not valid(text): journal_reject; return E_SCHEMA_INVALID
  if row.answer_hash == hash(text) and row.answer_actor == actor:
      return idempotent_success
  row = answered(actor, text, now)
  journal(trusted answer)
```

No cap/attempt column exists today. A UI can spam the route; CAS prevents duplicate `open` transitions but not operator retry loops.

### 4.6 Hive/ancestor memory banks and gated HyperAgent access

Current truth: banks exist for run + branch; read-set is own branch + immediate-parent branch + run bank; root reads a run digest and run bank; journal/snapshot/trust exist.

Missing: hive bank, ancestor summaries, root read of summaries, and a leak matrix.

```pseudo
function ensure_bank_tree(run, root, super_tasks):
  run_bank = ensure(kind="run", owner=root)
  for super in super_tasks:
      hive = ensure(kind="hive", owner=super, parent=run_bank)
      for child in children(super):
          ensure(kind="branch", owner=child, parent=hive)

function read_set(task):
  own = active_entries("branch:" + task.id)
  parent_branch = active_entries("branch:" + task.parent_task_id)
  ancestors = summary_entries_only(ancestor_hive_chain(task))
  run = run_constraints_and_digest(task.run_id)
  return bounded(run + ancestors + parent_branch + own, limit=16)

function promote_to_hive(branch_summary, target_hive, operator):
  require operator authorized
  if trust in {verified, authoritative}: write(hive, summary)
  else write(hive, summary, trust="unverified", quarantined=staged)
```

Acceptance tests missing:

- sibling raw entry canary is absent;
- ancestor raw observation is absent but summary is present;
- root sees summaries but not raw branch rows;
- quarantine blocks promotion/read;
- retracted entry is not read;
- read-set is bounded and deterministic.

### 4.7 Trust/quarantine lifecycle

Current truth: trust column and operator promotion; read filter excludes non-readable trust; migration notes become `note`; UI has promote.

Missing: explicit quarantine transition, reason, actor, promotion approval, demotion/retraction, and a gate that prevents a model-authored write from becoming authoritative without operator action.

```pseudo
state Trust = unverified | verified | authoritative | quarantined | retracted
transition:
  model_write -> unverified
  operator_promote(unverified, verified) -> journal
  operator_promote(verified, authoritative) -> journal
  operator_quarantine(any, quarantined, reason) -> journal
  operator_retract(any, retracted, reason) -> journal
invariant:
  read_set only sees active + (unverified|verified|authoritative) + migration note
  quarantine/retracted never readable
```

### 4.8 Retention, snapshots, CAS clear

Current truth: inactive prune (superseded/retracted) with dry-run; terminal-run clear; snapshots; hash chain/head.

Missing: age policy, snapshot pruning, journal compaction, CAS revision preview, clear receipt linked to journal/snapshots, and role/global clear policy.

```pseudo
function preview_clear(run, expected_rev):
  require run.status.terminal
  require run.rev == expected_rev
  return {flat_rows, bank_entries, snapshots, journal_rows, banks}

function clear_run(run, preview_token):
  tx {
    require preview_token.matches(current_rev)
    delete flat rows for run
    delete bank entries for run
    close banks
    optionally keep journal/snapshots as audit (default: keep)
    append journal("cleared", preview counts)
    bump run.rev
  }
  return receipt
```

Current clear has no `expected_rev`, no preview token, no snapshot/receipt details.

### 4.9 Context snapshots: actual prompt verification

Current truth: snapshot stores bank ids, entry ids, rendered char count, truncation flag, SHA-256 of rendered memory; UI shows metadata.

Missing: the actual **rendered prompt hash** (not only memory block), a provider request digest, and snapshot retention.

```pseudo
before_provider_call(task, request):
  memory_render = render_bank_facts(task)
  prompt_render = build_task_prompt(task, memory_render)
  snapshot = {
    task_id, run_id,
    bank_ids, entry_ids,
    memory_chars, memory_sha256,
    prompt_chars, prompt_sha256,
    provider_model, request_hash,
    truncated
  }
  store(snapshot)
  send(request)
```

Current snapshot hashes the memory texture, not the whole provider prompt.

### 4.10 Provider context/capability preflight

Current truth: DeepSeek-only config, budget preflight (`RunBudget::preflight`), model/balance metadata, refusal matrix.

Missing: context-window/model-support preflight before HTTP; a user-visible capability matrix; secure key onboarding; live verification.

```pseudo
function preflight(request, model):
  facts = model_facts(model)                     // only documented facts
  if facts.context_window is null:
      return Refusal.E_MODEL_CAPABILITY_UNKNOWN  // or explicit unsupported
  estimate = estimate_tokens(request)
  if estimate.prompt + request.max_tokens > facts.context_window:
      return Refusal.E_CONTEXT_LIMIT
  if request.tools not supported: return E_TOOL_UNSUPPORTED
  if model not in allowed: return E_MODEL_UNKNOWN
  return ok

function set_key(new_key):
  require local operator UI + confirmation
  write to OS credential store, never source/log
  verify with a minimal account call
  clear the in-form value after success
```

No such endpoint/UI exists.

### 4.11 Read/session authentication and authority

Current truth: mutation middleware requires a bearer/x-hivemind-token; host allowlist; loopback bind; `/api/session` issues the WS token; some provider routes. GET routes are generally unauthenticated.

Missing: a coherent session principal, auth on sensitive reads, session expiry, role/autonomy settings, and a secure onboarding boundary.

```pseudo
request -> host_guard -> session_auth:
  if path in {health, diagnose?}: allow local unauthenticated with redaction
  else require Authorization or x-hivemind-token
  principal = verify_session(token)
  require principal.role allows operation
  audit(principal, method, path)
```

Also: change `/api/session` so a random loopback page cannot mint a usable token; use a pre-shared/OS-scoped bootstrap or same-origin secret.

### 4.12 Crash resume and durable event journal

Current truth: snapshots + boot recovery mark active runs interrupted; task settlement; A2A open questions expire; no resume.

Missing: durable event journal, deterministic resume, operator resume/abort, daemon watchdog.

```pseudo
append_event(run, event):
  seq = next
  write(db, event, hash_chain)
  publish(event)

recover_on_boot():
  for run in active:
      mark interrupted
      settle tasks
      expire open A2A
      reconcile snapshots
  for run in resumable:
      if operator_resume_allowed(run):
          replay plan + last snapshot + committed events
          restart scheduler with idempotency keys
      else:
          leave interrupted with explicit UI
```

Current no durable event store/resume. A page cannot resume an interrupted run.

### 4.13 Ownership/cards/authority

Current truth: ownership projection (run/branch/none), one durable run owner index, roster UI.

Missing: branch owner uniqueness, exclusive ownership enforcement for tools/banks, an "office" model, card display of owner/branch, and an operator ownership switch.

```pseudo
function claim_branch(super_task):
  require class(super) and run active
  unique_index(run_id, ownership_scope) where ownership_mode="branch"
  upsert(ownership=super.id, scope="hive:"+super.id)
  ensure_hive_bank(super.id)

function authorize_tool(task, tool):
  require ownership_allows(task.ownership, tool.scope)
  require bank_owner(task.bank) in {task.owner_task_id, task.owner_task_id.ancestors}
```

### 4.14 Remote/daemon runtime

Absent. No principal/device/ACL/grant table, no daemon records, no watchdog, no remote lane UI. Any claim is invalid.

```pseudo
principal = device_principal(device_id, capability_scope)
grant = issue_grant(principal, task_or_bank, expiry, nonce)
require grant.covers(resource) && nonce_unused
daemon = {id, lane, job, heartbeat_at, last_error}
watchdog: if heartbeat stale -> mark unknown, do not fabricate success
```

### 4.15 Release and artifact integrity

Current truth: release preflight accepts `1.2.0-dev`; dry-run blocks at `E_RELEASE_TAG_MISSING`; no sealed artifact.

Missing: owner-created annotated tag, sealed build, artifact hash manifest, runtime smoke, upgrade/rollback drill.

```pseudo
tag = owner.create_annotated_tag(version)
seal = build_release(source_commit=tag.target, clean=true)
manifest = hashes(binaries, assets, docs)
verify(manifest)
publish(zip, sha256)
```

### 4.16 DB/security hardening residuals

- schema lacks FKs for some multi-row relationships;
- multi-row writes are not all transactional;
- tool shell allowlist is not a containment boundary;
- approval preview truncation can hide the tail while args hash binds full payload;
- read routes expose memory/roster/projects to any local caller;
- durable event journal absent;
- no CI/e2e wrapper that refuses skipped security specs.

---

## 5. Missing UI/UX and panels (pseudocode)

### 5.1 Surface inventory: what is wired now

| Surface | Wired? | Notes |
|---|---|---|
| Workspace canvas/chat | yes | Canvas, HiveField, ChatCore, ControlBar, Scrubber |
| Swarm | yes | hierarchy, capacity from `/api/runtime`, branch list |
| Routing | yes | read-only edge table from graph |
| History | yes | revisions, replay, run detail, transcript |
| Agents | yes | hierarchy cards, activity, profiles, roster, memory, banks, messages, A2A inbox |
| Data | yes | file tree/read, jailed |
| Models | yes | provider models/balance/usage |
| Settings | yes | tabs: overview/provider/appearance/diagnostics/FigCraft |
| Approvals | yes | overlay + toasts; no history |
| A2A inbox | yes | global pending + answer |
| Diagnostics | yes | `/diagnose` + workspace + active run |
| FigCraft | yes | static accepted gallery |

### 5.2 Root / HyperAgent console

Missing: an operator-visible root console that explains what the HyperAgent is doing now and exposes safe actions.

```pseudo
component RootConsole(run):
  state = query("/api/runs/{run}/root")
  render:
    header = "HyperAgent · {root_state.phase}"
    budget = calls_left / children_left / tokens_left
    active_children = tree(root).filter(not terminal)
    inbox = pending A2A addressed to root
    actions = [pause? cancel, force synthesize?]
  on action:
    dispatch typed command; disable until ack; never optimistic terminal state
```

Required server additions: root state projection, root tool event stream, root pause/synthesize policy. No fake buttons.

### 5.3 SuperAgent branch dashboard

```pseudo
component BranchDashboard(super_task):
  branch = query("/api/runs/{run}/branches/{super}")
  render:
    owner/scope + hive bank id
    children cards (class, state, depth, budget)
    branch summary/trust
    kill/retarget actions gated by policy
  empty/error states: "no branch projection" / named refusal
```

Missing server projection: branch rollups, hive bank, child counts, branch budget.

### 5.4 Ownership cards and office chips

```pseudo
component AgentCard(task):
  render:
    class badge (hyper/super/agent/subagent)
    rank/profile
    ownership mode/scope
    parent task link
    bank/trust indicators
  on select: selectTask(task.id)
```

Current Agents grid uses store labels only; the roster list shows ownership but cards do not. Add a **card projection** endpoint or extend roster data into the store. Do not infer class from a label.

### 5.5 A2A triage console

Current inbox is a flat list. Missing:

```pseudo
component A2ATriage():
  filters = {run_id, kind, status=open, from_task, text}
  list = GET /api/a2a/pending + filters
  selected = detail with timeline (created, deadline, answered, actor)
  answer = POST .../answer token-gated
  states:
    loading -> skeleton
    error -> named refusal, no rows
    empty -> "no open questions"
    inflight -> disable one row, optimistic answer NOT shown as accepted
  keyboard: j/k select, enter focus answer, escape clear
```

Current allows answer but no filter/history/selection/keyboard.

### 5.6 Run failure ledger and artifact/diff review

Missing entirely:

```pseudo
component RunFailureLedger(run):
  rows = failed/killed/interrupted tasks + refusal code + turn + last tool
  each row -> open task transcript / provider error
  actions: retry (if policy), export support bundle

component ArtifactDiff(run, base_rev, head_rev):
  files = GET /api/runs/{run}/artifacts/diff?base=&head=
  render additions/deletions with trust labels
  no data -> empty state
  approval gated for apply/rollback
```

Current run detail has state counts/usage/banks only; no artifacts/diffs.

### 5.7 Provider onboarding/settings

Current settings is read-only and shows status. Required:

```pseudo
component ProviderSettings:
  key state: absent | present-unprobed | verified
  model selector populated from documented facts
  context/budget readouts
  key entry form:
    password input, never repopulate from server
    submit -> POST /api/provider/key (not implemented)
    success -> clear field, refresh status
    refusal -> named code, never echo key
  "test connection" -> minimal call with explicit cost/usage
```

No key-write endpoint, no context preflight display, no model selector. Do not fake a key input that only writes local state.

### 5.8 Crash-resume panel

```pseudo
component RecoveryPanel:
  runs = GET /api/history/runs?status=interrupted
  if none -> no interrupted runs
  selected -> last snapshot, last completed tasks, open A2A
  actions: resume (requires durable journal) / acknowledge
  disabled with explanation while resume is unimplemented
```

Current no resume; an "acknowledge" action could be real if server supports status transition.

### 5.9 Approval history/ledger

```pseudo
component ApprovalHistory:
  rows = GET /api/approvals/history
  filters: decision, tool, run, task
  details: hash, preview, actor, timestamp, decision
  no route -> empty/error, never fabricated
```

Current only live overlay; backend ledger file exists but no history route.

### 5.10 Memory hive map / bank tree

```pseudo
component MemoryHiveMap(run):
  banks = GET /api/memory/banks?run_id
  build tree:
    run -> hive(super) -> branch(task)
  nodes show kind, owner, entries, trust counts, closed
  select bank -> entries + journal + snapshot links
  controls: promote, prune, clear (confirmation)
  no hive/ancestor data -> honest "not materialized"
```

Current list+entries only. There is no hive bank to draw.

### 5.11 Snapshot/journal viewer

Current snapshot metadata is in RunDetail; journal it integrated in bank panel. Missing:

```pseudo
component SnapshotViewer(task):
  GET /api/memory/context/{task}
  render: banks, entry ids, chars, truncation, hashes, created
  button: copy digest; link to journal verify
  if none -> "no snapshot"

component JournalViewer(run):
  GET /api/memory/journal + /verify
  filters/actions; copy chain head; export JSON
```

### 5.12 Template catalog detail

Current library fetches insertable templates and shows name; missing inputs/license/source/fetched-at and details.

```pseudo
component TemplateCatalog:
  GET /api/templates
  filter insertable
  detail: name, inputs, spdx, source_url, fetched_at
  insert only if active run and policy allows
  error: no buttons, no stale hardcoded fallback
```

### 5.13 Settings sub-tabs (implemented) + remaining features

Implemented: Overview, Provider, Appearance (motion slider + palette), Diagnostics, FigCraft.  
Still missing/for later:

```pseudo
SettingsTabs:
  General
  Provider
  Appearance (motion, palette, reduced motion)
  Diagnostics
  FigCraft
  Shortcuts (future): keymap table, reset
  Labs (future): feature flags gated by environment
  About (future): version/commit/build, licenses, support bundle
```

No fake "Labs" toggles until server flags exist.

### 5.14 Chat alive/UX

Implemented: working bee, typing dots, streaming outline, live status chip, honeycomb chat background, enter/stop/send buttons, refusal panel, tool activity, reduced motion.

Still missing:

```pseudo
component ChatCore:
  live chip: connected/idle/active/offline (now)
  scroll management: if user scrolled up, show "new output" affordance instead of forcing scroll
  message actions: copy, retry, collapse tool payload
  transcript limits: virtualization for >200 messages
  composer: draft persistence, character/token estimate, attachment policy
  stop: confirm or immediate? current immediate
  accessibility: role=log on message list, live region throttled
```

No fake token count; show only server-reported usage.

### 5.15 Mobile/responsive

```pseudo
breakpoint < 1024:
  hide library rail (already hidden)
  collapse history aside into drawer
  chat becomes bottom sheet/full-screen overlay
  settings tabs become horizontally scrollable
  card grids min 1 column
  focus returns to opener on close
```

Not implemented/tested.

### 5.16 Accessibility

```pseudo
checks:
  tablist has aria-selected/tabIndex (settings yes, library yes)
  modal focus trap + inert background (confirm partial)
  live regions: chat, A2A answer receipt, run detail loading
  forms labelled; inputs have aria-label/visible label
  contrast: token corrections
  keyboard: every action reachable; no drag-only action
  reduced motion: data attr + media query
```

No axe/e2e evidence.

### 5.17 "Random UI/UX features saved for later" (real candidates)

These are ideas, not implemented; no dead controls should be shipped.

1. **Command palette** (`Ctrl/Cmd+K`): run, task, panel, memory bank actions; server-fed; gated.
2. **Shortcut cheatsheet** in Settings → Shortcuts.
3. **Run story/timeline** with replay playhead and event narrative.
4. **Memory hive map** (tree above).
5. **Branch budget meters** with sparkline from usage events.
6. **Support bundle exporter** (diagnose + logs + schema, secrets redacted).
7. **Approval ledger history**.
8. **Notification center** for A2A/approvals/refusals.
9. **Theme/contrast/density** controls with real CSS variables; only if tokens support them.
10. **Agent profile editor** with JSON schema validation and preview.
11. **Artifact/diff review**.
12. **Run compare** (revision vs revision graph diff).
13. **Ownership office chips** and transfer action.
14. **Provider key onboarding** (secure store).
15. **Release/About panel** with build/commit/evidence.
16. **FigCraft board-to-surface map** (board → panel links).
17. **Calm-mode schedule** (auto motion off at night).
18. **Chat message search** within run.
19. **Pinned A2A question** / triage queue.
20. **Keyboard-only spatial movement** for canvas cards.

---

## 6. FigCraft implementation rules and residual design debt

### 6.1 Rules for every new panel

```pseudo
render_surface(spec):
  require spec.uses(TokenModule)
  require spec.card.radius in {4,8,12,14,16,24,999} with named token
  require spec.button.radius == radius-button
  require typography from type scale or declare a design decision
  require hover/focus/disabled/loading/error/empty states
  require aria label + keyboard path
  require reduced-motion fallback
  require no hex outside token module
  add designParity test for the new file
  if FigCraft board required:
      mark VISUAL_FIDELITY=UNVERIFIED until owner/asset acceptance
```

### 6.2 Known component mismatches to fix next

1. Move local settings icon switch to shared `Icon` primitive with 14px line icons and `currentColor`.
2. Normalize remaining arbitrary text sizes into named utilities:
   - `--text-card-meta`, `--text-hud`, `--text-chip`, `--text-readout` as tokens or documented exceptions;
   - one audit pass across `LibraryPanel`, `SwarmPanel`, `Header`, `ChatCore`, `AgentMessagesPanel`.
3. Replace `HistoryPanel` fixed 300px stack with a split pane/drawer.
4. Add focus trap/inert to `ConfirmDialog`.
5. Add `aria-live` throttling and `role="log"` to chat transcript.
6. Add `scroll-pause` behavior in chat.
7. Add a real **settings shortcut tab** only after commands exist.
8. Add a **motion audit**: ambient-off must stop/wrap SMIL, not just hide it.
9. Add a **visual regression harness** (Playwright screenshots at 1440×900 and 375×812) and axe; this is the missing visual evidence.
10. Add a **FigCraft board regeneration decision**: either install the owner toolchain or freeze the accepted board set and stop claiming regeneration.

### 6.3 Bee choreography: current implementation truth

- `HiveField` now renders 3 `AmbientBee` glyphs at scale 0.1/0.15/0.2 (≈2/3/4px body) on a figure-eight `animateMotion` path around the 620×660 hive.
- They are decorative, blur `0.18px`, low opacity, reduced-motion removed, ambient-off hidden by CSS.
- Main hive sheen opacity increased to 0.82; the existing gradient/soft-light layer remains.
- `BeeEdge` now travels source→target with `offset-path` + `motion-bee-ride` back-and-forth, with a static midpoint bee fallback when `offset-path` is unsupported or reduced motion is on.
- `ChatCore` now has a live status chip and streaming outline; the working bee and typing dots remain.

Unverified: actual browser rendering, whether `offset-path` aligns in React Flow coordinates, whether SMIL bees respect ambient-off CPU-wise, and whether the visual density is tasteful at 1440×900. These need screenshots/e2e.

---

## 7. DeepSeek Harness session history: what it proves and does not

The 80-round DSH ledger is development history, not application data. It records the work sequence, commits, and gate receipts. It is not a source of runtime truth for a new tree; the repository is.

```pseudo
truth_priority:
  1. current command output / gate receipt
  2. current file content
  3. durable session history (this record)
  4. memory/belief
  never: assume history is current
```

History summary relevant to this UI pass:

- P0 truth freeze and evidence discipline established early.
- A2A answer/status/wake, memory bank schema/read-set/journal/snapshot/trust/clear/retention, ownership, strict roles, and run detail were added incrementally.
- Rounds 77/79 audited the tree and explicitly refused completion.
- `d519831` was the last clean docs-only head before this pass; the current pass starts from that clean state and adds the UI/settings/inbox/diagnostics/bees/chat changes.
- The 80-round cap is a session budget, not an objective-complete signal.

What history does **not** prove: live provider behavior, browser rendering, remote/daemon, scale, release artifact, or HMARS completion. Those remain unverified/missing.

---

## 8. Priority order to finish honestly

```pseudo
P0 (truth):
  keep every gate green at one SHA
  browser/e2e + axe + visual snapshots
  remove remaining arbitrary typography/radii
  focus trap/inert modal
P1 (operator essentials):
  A2A triage filters/history
  run failure ledger
  memory hive map/snapshot viewer
  provider onboarding only if secure store exists
P2 (HMARS semantics):
  Root HyperAgent monitor/tool loop
  SuperAgent branch plan/bank/budget
  parent-model A2A answer + durable waiter
  TaskRecord identity authority
P3 (durability/security):
  read/session auth
  context preflight
  durable event journal/resume
  FK/transaction hardening
P4 (product edges):
  remote/daemon (design-only current)
  release tag/sealed artifact
  scale (explicitly excluded from this pass)
```

---

## 9. Acceptance pseudocode for the UI work delivered here

```pseudo
verify_ui_pass():
  run typecheck -> exit 0
  run unit tests -> 191 pass
  run build -> exit 0
  run designParity -> 5 pass
  run noHardcoded -> all pass
  check new surfaces:
    A2A inbox: fetch pending, answer CAS, error/empty/inflight
    Settings: tablist keyboard, motion slider + persisted preference, palette
    Chat: live chip, streaming outline, send/stop/enter
    Hive: ambient bees, sheen, edge bees; reduced-motion fallback
    Confirm: focus confirm, escape/cancel, restore focus
  read cover image + tokens + FigCraft manifest
  report:
    done with command/file evidence
    not done: root console, branch dashboard, failure ledger, auth, provider setup, resume, remote, release, scale
    unverified: browser layout, pixel fidelity, SMIL/offset-path behavior, axe, live provider
```

---

## 10. Final truth

The UI is now materially more complete and more consistently HiveMind/FigCraft-shaped: settings are tabbed, the A2A queue is usable, diagnostics and transcripts are visible, destructive actions use a token-styled dialog, chat feels alive, and the hive has ambient/sheen/edge-bee choreography plus the randomized honeycomb ripple. The run insight and shortcut cards add real data/controls rather than decorative placeholders.

The HMARS objective is still not complete. The largest missing pieces are root orchestration tools, SuperAgent branch ownership, parent-model A2A answers, hive/ancestor banks, read/session authority, durable resume, remote/daemon, and a sealed release. Scale remains intentionally out of scope.

No new FigCraft boards were generated; the workspace has no FigCraft generator. The implementation uses the accepted token system and boards, and pixel fidelity remains unverified.

### 1.3 Additional cards/features added in the ripple pass

1. Honeycomb gold ripple: a soft vertical column travels left-to-right across the full honeycomb grid, behind the hive; each cycle randomizes its duration from 12–30 seconds.
2. Run insight state-distribution card: real task-state proportions, legend, and accessible label.
3. Keyboard shortcut card: the actual implemented chords (canvas, chat, modal, tabs), no fake controls.
4. Settings motion control: persisted ambient level.
5. Settings palette: all 20 printed tokens.
6. Settings tab icons: shared local line-icon set.
7. A2A inbox card.
8. Diagnostics card.
9. Historical transcript card.
10. Token-styled destructive confirmation card.
11. Chat live status/streaming surface.
12. Figure-eight ambient bees.
13. Traveling edge bees.
14. Stronger hive sheen.
15. Template catalog detail list (insertable IDs from server).