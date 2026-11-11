# CAP pseudocode patch map and codebase impact

**Branch:** `capability-expansion` · **Base:** `dcc483f` · **Rule:** one HiveMind, HMAS remains
orchestrator, no second product/mode. These are integration contracts for the five code seats; they
are pseudocode, not claims that the code is complete.

## CAP-1 — schema, policy, sessions

```text
PermissionPolicy = classic | plan | standard | trusted
DEFAULT = classic                    # current approvals preserved
trusted = explicit opt-in, persisted per run/session

Run:
  add session_id: Option<String>
  add permission_policy: PermissionPolicy (default classic)

Migration v25:
  CREATE sessions(id, project_id, name, workspace_root, permission_policy, created_at, updated_at)
  CREATE session_events(...)
  CREATE todos(session_id, seq, text, state, created_at)
  ALTER runs ADD session_id TEXT NULL REFERENCES sessions(id)
  ALTER runs ADD permission_policy TEXT NOT NULL DEFAULT 'classic' CHECK(...)
  # down migration restores v24 shape

create_run(project, prompt, status):
  policy = session.policy or CLASSIC
  insert run with policy
  return Run

set_run_policy(run_id, policy):
  UPDATE runs SET permission_policy = ? WHERE id = ?
  # only explicit operator/API; no inference from prompt text

resolve_policy(run, task, tool):
  if run.policy == plan/read_only and tool.requires_approval: E_MODE_DENIED
  if run.policy == standard and tool.mutates_workspace: allow without interactive prompt
  if run.policy == trusted and tool.mutates_workspace/execute: allow (explicit opt-in)
  otherwise classic: current approval path
```

**Impact:** existing `Run` literals are only `create_run` and `row_to_run`; three run SELECTs gain
columns; all v24 rows load as `classic`/no session. Existing approval semantics remain default.
Risk: hardcoded schema 24 pins must become 25; no `runs.mode`; no removal of existing grants.

## CAP-2 — repo index/context

```text
index_workspace(conn, root):
  walk(root) where not ignored(.git,target,node_modules,binary,>1MiB)
  for file in deterministic order:
    chunk(file) -> repo_files/repo_chunks
    insert FTS5 row
  probe FTS5 first; if absent, fail E_REPO_INDEX_UNAVAILABLE

search(conn, query, scope, limit):
  jail_resolve(scope) or refuse
  FTS5 MATCH + glob/path filter
  bound hits; return paths/snippets with explicit truncation
```

**Impact:** additive v26; no existing file tool changed; requires rusqlite FTS5 feature if bundled
build lacks it. Search results are data, never instructions.

## CAP-3 — coding tools

```text
apply_patch(path, unified_diff, expected_sha?):
  resolve inside jail; refuse outside/binary/oversize
  if expected_sha and mismatch: E_PATCH_STALE
  parse all hunks; apply to temp; if any context mismatch: E_PATCH_CONTEXT (no write)
  journal before/after sha; atomic rename; return diff summary

fs_glob(pattern), fs_grep(regex), read_range(path,a,b), git_*(...), run_tests(...), run_command(...):
  validate args; jail; bounded output/time; approval policy; no shell interpolation
```

**Impact:** new ToolIds/grants; existing 12 remain; tier∩persona still applies. Every mutate/execute
goes through existing approval or explicit `trusted` policy. No second tool registry.

## CAP-4 — UI panes

```text
Workbench(props):
  FileTree(paths, selected)
  Editor(content, readonly)
  DiffView(before, after, onAccept/onReject)
  Terminal(lines, onInput)
  Timeline(steps, agentAttribution)
  TodoPanel(items)
  SessionRail(sessions)
  CostBar(usage)
  CommandPalette(actions)
  # rendered inside existing Workspace/Data/Agents, no new primary nav
```

**Impact:** no change to existing panels; dependency pins for CodeMirror/xterm/PTY required. All
actions call existing store/API; no local fake data. 1440/375 preserved.

## CAP-5 — checkpoints/jobs

```text
checkpoint_before(run, path):
  blob = content_addressed(read(path)); record file_op; return blob id

restore(run, seq):
  load journal; verify current sha; if drift: E_CHECKPOINT_DRIFT
  write each before blob atomically; refuse outside jail/symlink

job_start(program,args,cwd,policy):
  jail cwd; no interpreter hosts; bounded output ring; Job Object kill-tree
  cap 4 per session; cleanup on session end
```

**Impact:** additive v27/v28; no existing snapshot/history semantics changed. File operations are
only allowed after policy/approval; old graph snapshots remain history, not file rollback.

## Shared integration points (Lead-owned)

- `lib.rs` module declarations and exports.
- `http.rs`/`ws.rs` routes/commands for sessions, files, diff, terminal, jobs.
- `tool.rs`/`provider_executor.rs` tool registration and tier/persona grants.
- `execution.rs` policy enforcement and file-journal hooks.
- migrations/table ownership and test pins.
- `web/src/App.tsx` pane mounting; package dependency pins.

## Preservation checks

- Existing 12 tools/5 classes/35 grants remain; capability tools are added.
- Existing run/approval/ledger/jail/history paths untouched for `classic`.
- v24 database opens and migrates; down migration restores.
- Existing UI panels/nav/brand remain; new panes additive.
- No testing/build gate until all capability and residual coding is complete.
