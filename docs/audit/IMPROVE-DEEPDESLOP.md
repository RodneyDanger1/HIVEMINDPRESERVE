# deep-deslop — HyperAgent governance path

Scope: `backend/crates/hiverouter-core/src/{execution,provider_executor,remote,store,ws,http}.rs`,
`backend/crates/hiverouter-tools/src/{tool,census}.rs`, and
`backend/crates/hiverouter-core/tests/hyperagent_governance.rs` (+ sibling test files).
Base: `HEAD 8f625bd`. Working tree reported clean (`git status --porcelain` empty) at audit time —
the described "uncommitted idempotency lane" was **not present in the tree**; see Unverified.

Method: symbol scan of every `fn` in the eight target files, counted repo-wide
(`\bsymbol\b` over all `.rs` under `backend/`). Declaration counts as 1 reference, so
`refs=1` ⇒ declaration only ⇒ dead; `refs=2` ⇒ declaration + one live call ⇒ live, not slop.
Then adjacent-line reading for comment/log/defensive patterns.

## 1. Proven deletion candidates

| # | Pattern | Exact file:line | Grep proof (refs) | Risk | Expected deletion |
|---|---|---|---|---|---|
| D1 | Dead public helper — superset query never called | `backend/crates/hiverouter-core/src/store.rs:1697` `pub fn agent_memory_for_scope_any_role` | `Select-String -Pattern 'agent_memory_for_scope_any_role'` over all `*.rs` in `backend/` ⇒ **1 hit, the definition itself**. 0 call sites in src, tests, benches, or bins. | **Low.** No caller, no trait impl, no `#[used]`/macro/reflection path. Doc comment claims it *is* "the branch/run bank recall path"; that claim is false today — recall goes through `agent_memory_entries_for_scope` (store.rs:2399, refs=2, live). Deleting removes a false doc claim too. | Function body + doc block, ~1691–1720 (≈30 lines). Keep the SQL if/when a role-free recall is actually wired. |
| D2 | Dead wrapper — pure forwarding shim | `backend/crates/hiverouter-core/src/ws.rs:1247` `pub async fn send_sequenced` | `Select-String 'send_sequenced'` over all `*.rs` ⇒ **1 hit, the definition**. Its own body is one line: `send_envelope(socket, &sequenced.envelope).await`. Doc comment says "used by tests and later phases" — no test uses it. | **Low.** Zero-arity-preserving shim; `send_envelope` is the live path (refs>2). Deleting changes no behavior, no sequencing semantics (it adds none — it forwards `.envelope` only, dropping `seq`). | Lines 1246–1252 (7 lines). Callers, if ever added, should call `send_envelope` directly. |

Both are `pub`, so `E_UNPROVEN` does not apply: the unused-symbol proof is a repo-wide grep,
and the crate is a library whose consumers are the sibling crates and bins — all inside the
same `backend/` grep root. If a consumer exists outside `backend/` (e.g. an out-of-tree
client), each is reclassified `E_UNPROVEN` and kept.

### 1.1 GitNexus cross-check (required by repo `AGENTS.md`)

Graph analysis was run, not substituted: `node .gitnexus/run.cjs impact <sym> --direction upstream --repo .`

| Symbol | Graph result | Reading |
|---|---|---|
| `send_sequenced` | `impactedCount: 0`, `risk: "UNKNOWN"` | Graph finds **zone** callers but returns UNKNOWN, not LOW. Per `AGENTS.md` an empty caller set is *not* evidence of unused. Text search (D2's grep, 1 hit) is the confirming evidence `AGENTS.md` requires for an UNKNOWN verdict. Verdict unchanged: **dead**, confidence from grep, not from the graph. |
| `agent_memory_for_scope_any_role` | `error: Target not found`, `risk: "UNKNOWN"` | The index does not contain the symbol at all (stale for `store.rs`, or unindexed). The graph therefore neither confirms nor refutes D1 — `UNKNOWN` is unresolved, and D1's standing rests **only** on the grep. Treat D1 as proof-by-text-search; re-run `analyze --index-only` before deleting if graph-level certainty is required. |

No `impact`-HIGH/CRITICAL warnings were triggered, because neither symbol resolves to a caller set.
`detect-changes` was not run: no graph-changing edit was made (one new doc file only).

## 2. No deletion proven (kept deliberately)

| Symbol / site | File:line | Why it stays |
|---|---|---|
| `checkpoint_run_usage` | execution.rs:1011 | Live (refs=2, called at execution.rs:1008 after every measured reply). It is the *audit durability* write — "a crash must not hide tokens already spent". Deleting or merging it would drop spend on crash. **Protected: audit row.** |
| `persist_run_usage` | execution.rs:1027 | Live (refs=2), drains the map on terminal. Companion of D1's sibling above; distinct lifetime (remove vs. clone). Not a duplicate — see proposal P2. |
| `reconcile_terminal_tasks` | execution.rs:1635 | Live (refs=2). Per-task re-read under `task_state_lock` + `is_terminal_task_state` CAS is a **real** race guard against a late completion resurrecting a terminal task — exactly the case store.rs:2965 pins (`killed_task_stays_killed_after_a_late_completion`). Not defensive slop. |
| `with_snapshot_store`, `with_clock`, `with_persona_prompts` | execution.rs:737, 871; provider_executor.rs:149 | Live (refs=2). Test-seam constructors, exercised by the suites. Removing them breaks tests, not slop. |
| `host_guard`, `mutation_auth`, `host_allowed`, `loopback_host` | http.rs:687, 715, 777, 792 | Live (refs=2 each). **Real external-input guards** (Host header / DNS-rebind / CSRF-adjacent mutation auth). Explicitly out of scope for deletion. |
| `store.rs` terminal/rewire/CAS tests (`terminal_task_cannot_be_resurrected`, `task_parent_row_follows_a_rewire`, `a2a_answer_is_cas_and_duplicate_is_refused`, `journal_verification_detects_truncation` …) | store.rs:2908–3640 (21 fns, refs=1) | `refs=1` is expected for `#[test]` fns (invoked by the harness, not by name). Not dead. |
| All `#[test]` fns in execution.rs:4828+, provider_executor.rs:973+, remote.rs:1567+, ws.rs:1301+, http.rs:3009+, tool.rs:572+, census.rs:221+ | as listed | Same: harness-invoked. The scan flagged them only because the counter is name-based. |
| `publish_pong` (ws.rs:1242), `send_hello_and_replay` (ws.rs:593), `wait_for_hello` (ws.rs:567), `publish_pending_approval_replays` (ws.rs:677), `publish_pending_a2a_replays` (ws.rs:712) | ws.rs | refs=2 each ⇒ live. Handshake/replay ordering is load-bearing for reconnect correctness. |
| `contract_count` (tool.rs:556), `grant_from_row` (remote.rs:220), `find_by_request_id` (remote.rs:422), `record_admission` (remote.rs:1403) | as listed | refs=2 ⇒ live; `find_by_request_id` is the idempotency lookup — protect. |
| `caller_bank_for_class` (store.rs:261), `ensure_bank_for_inserted_task` (store.rs:2690), `memory_journal_chain` (store.rs:2020) | store.rs | refs=2 ⇒ live. Transaction-boundary / bank-affinity helpers. **Protected.** |

**Stub sweep** (`TODO|FIXME|XXX|HACK|placeholder|NotImplemented|dummy|unreachable!()` over the
eight target files): **zero hits.** Nothing to classify.

## 3. Comment / log / defensive slop

No deletions proven at the comment level. The candidates I read state *why*, not *what*:
- execution.rs:1006–1007 ("A crash or operator kill must not hide tokens already spent") — rationale, keep.
- store.rs:1691–1696 — rationale is sound, but it is attached to dead code (D1); deleting the
  body removes a **misleading** comment, which is the only comment-level win proven here.
- The paired `tracing::error!` at execution.rs:1019–1022 and 1035–1038 are not duplicates: they
  fire on different lifetimes (checkpoint vs. terminal drain) and carry distinct messages. Not log slop.
- `let _ = self.graph.set_state(..)` at execution.rs:1653/1658 discards a `Result` on a projection
  write that is re-derived from the store on boot; it is intentional best-effort, not a swallow.
  Flagged as **suspected** only — no deletion proposed.

## 4. Where the survivors point: 5 reliability proposals

Pseudocode only; no code was edited.

### P1 — Admit-or-replay on one idempotency key (before→after)

The idempotency lane is described as uncommitted and `find_by_request_id` (remote.rs:422) is the
lookup seam. Today's shape risks "check, then insert" being split by another writer.

```
BEFORE                                   AFTER
req -> find_by_request_id(k)             req -> BEGIN IMMEDIATE
       if none: BEGIN; insert; COMMIT           row = find_by_request_id(k)   -- inside txn
       return new result                          if row: COMMIT; return replay(row)  -- same result
                                                  insert; COMMIT
                                                  return new result
                                         -- one txn, one winner; loser replays, never 500s
```
Guard: the `UNIQUE` index on the key is the real arbiter; the pre-check is an optimization, not the
guard. Expected effect: duplicate submits converge to the first result instead of interleaving.

### P2 — One monotonic usage writer (before→after)

`checkpoint_run_usage` (execution.rs:1011) clones and writes; `persist_run_usage` (execution.rs:1027)
removes and writes. Two writers, two lifetimes, same row.

```
BEFORE                                    AFTER
checkpoint: lock; get().cloned(); write    checkpoint(terminal: bool):
persist:    lock; remove();      write       lock; let u = if terminal { remove } else { get.cloned };
race: last writer wins -> a stale clone          if u { write_with_max(run_id, u) }   -- SQL MAX, monotonic
        can shrink persisted totals
```
Guard: `record_run_usage` takes `MAX(existing, new)` per column, so a stale checkpoint cannot roll back
spend. Audit row preserved; no second writer added.

### P3 — Reconcile as one CAS sweep, not a scan-under-lock (before→after)

```
BEFORE                                        AFTER
tasks = tasks_for_run(); sort;                ids = tasks_for_run(); sort;
for t: lock; fresh=get(t);                    for t:
        if terminal: maybe fix projection          store.update_task_state_if_not_terminal(t.id, terminal)?
        else: update_state(t, terminal)             -- single UPDATE ... WHERE state NOT IN (terminal)
                                                    if changed: graph.set_state; publish_node
```
Guard: the `WHERE state NOT IN (terminal)` predicate *is* the resurrect protection that
store.rs:2965 pins. Effect: one lock acquisition per task becomes none held across I/O; a late
completion can no longer interleave between `get` and `update`.

### P4 — Order-preserving replay fence (before→after)

`send_sequenced` (D2) drops `seq` — it forwards only `.envelope`. Replay after reconnect can therefore
interleave with live events.

```
BEFORE                                      AFTER
replay: for e in replay { send(e.envelope) }  replayed_through = replay.last.seq
live:   send(bus.event)  -- unbounded          replay: send in seq order, then set gate
                                                   live_ok = seq > replayed_through  (drop/skip stale)
```
Guard: the fence is the sequence number, already present on `SequencedEvent`; no new state machine.
(Deleting D2 is safe *because* the fence belongs on the bus subscriber, not on a forwarding shim.)

### P5 — Terminal transitions carry an audit row on the failure path (before→after)

`reconcile_terminal_tasks` maps `_ => AgentState::Failed` (execution.rs:1640) — a non-terminal
`outcome` silently becomes `Failed` for every child.

```
BEFORE                                    AFTER
match outcome { C/C/I => same,            let terminal = match outcome {
                _ => Failed }                  Completed|Cancelled|Interrupted => outcome,
                                               other => { audit(run.id, "reconcile.forced_failed", other);
                                                          AgentState::Failed }
```
Guard: adds an audit row, deletes nothing; refusal-free. Effect: an unexplained mass-`Failed` becomes
diagnosable from the ledger rather than inferred from task states.

## 5. Report shape

| Done | Evidence |
|---|---|
| Identified 2 proven-dead helpers | repo-wide grep returns 1 hit (the definition) for each: `agent_memory_for_scope_any_role`, `send_sequenced` |
| Enumerated protected guards/tx-boundaries/audit rows | table §2, 20+ symbols, each with refs count and reason |
| Stub sweep | `Select-String` over 8 target files: **zero hits** |
| 5 proposals | §4, pseudocode, before→after, each names the invariant it protects |

| Not done | Reason |
|---|---|
| Applied any deletion | Task is read-only, document-only ("no code edits") |
| Audited non-target files (`db.rs`, `run_state.rs`, `recovery.rs`, `daemon.rs`) | Out of the requested scope; the scan regex covered only the 8 named files |
| Full per-function body read of `remote.rs` / `http.rs` | Timebox (2m30s); those files were scanned symbol-wide, not read line-by-line, so comment/log slop there is **not** exhaustively assessed |

| Unverified | What would prove it |
|---|---|
| "Uncommitted idempotency lane" | `git status --porcelain` at `HEAD 8f625bd` returned empty — no such changes were visible. Confirm the branch/stash, or the lane is already committed |
| P1's current pre-check/insert split | Body of `remote.rs` request-admission path beyond the symbol scan was not read; the "before" is a hypothesis matching the reported lane, not a read fact |
| D1/D2 unused outside `backend/` | A grep of any out-of-tree consumer, if one exists |
| D1 graph status | `status --repo .` did not return within 75s (moved to background, then killed). Index freshness for `store.rs` is unconfirmed — re-run `node .gitnexus/run.cjs analyze --index-only` and `impact agent_memory_for_scope_any_role` before deleting |
