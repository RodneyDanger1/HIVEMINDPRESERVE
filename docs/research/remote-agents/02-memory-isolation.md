# 02 — Universal memory and remote isolation

**Status:** DESIGN (D), current-code facts V. Depends on the memory-bank implementation planned in
`22-memory-bank-implementation.md`; the committed tree currently stops at the v11 run/branch-scoped memory table
(`backend/crates/hiverouter-core/src/db.rs:249-274`), not the bank tree. Decisions D-R3/D-R7.

## 1. What "universal memory" means here

Two requirements must hold at once:

1. **One policy surface:** all memory kinds share bank identity, trust, provenance, retention, purge, audit and
   read-set rules — the operator's "connected Venn diagram", not four ad-hoc stores
   (`22` §2.5 rule 1: six bank kinds `global, project, run, hive, branch, agent_role`; `OPERATOR-DECISIONS.md:61-75`).
2. **No leakage across authority boundaries:** a tree-free remote cannot inherit a run/hive/branch bank, cannot read
   another remote's bank, and cannot turn device/memory text into instructions or authority.

This design adds **one bank kind, `remote`**, and uses the existing **`global`** kind as the **system/universal
bank**. It does not add a separate memory table. If the bank tree (renumbered **v14** in `04` §2; `20`'s plan called
it v13) has not landed, Phase R1 remote agents operate with **no learned memory** and read only the
operator-curated system facts/device registry; learned remote memory begins only after the bank migration
(gate G-R3 in `06`).

```
memory_banks(kind, scope_id)                      ← v14 per `04` §2 (22's v13 renumbered), extended here with kind='remote'
  global:default                → system/universal, operator-curated, read-only to every agent/remote
  remote:<remote_id>            → private to exactly that remote agent
  project/run/hive/branch/role  → task-tree banks; INVISIBLE to remotes (no edge, deny by construction)
```

`device` values (last state, room, alias) are **telemetry/registry**, not memory: they live in `devices`/
`device_events` and are read through the capability layer, not as bank entries. This prevents device state from
becoming "a fact" without provenance/TTL.

## 2. Actor × bank matrix (authoritative)

Legend: **R** read whole entries; **S** summaries/counts only; **W** write own bank; **P** promote one level (not
applicable to remotes in v1); **—** denied; **A** requires an operator-approved, TTL-bound grant.

| Actor | global/system | project/run/hive/branch/role | own remote bank | other remote bank | device registry |
|---|---|---|---|---|---|
| Operator (UI) | R + W | R + W | R + W | R + W | R + W |
| Task/agent in a run | R (via existing read-set) | per 22 matrix | **—** | **—** | **—** |
| Remote agent | **R (never W)** | **—** (never; no tree inheritance) | **R + W (quarantined)** | **—** | R (own devices only; state via capability API) |
| Remote supervisor | S (counts/health only) | — | — | — | S (reachability/state hash; not memory content) |
| Another task via explicit grant | — | — | **A** (one entry, TTL, same server, audited) | — | — |

Rules:

- **M-1** A remote's read set is exactly `{global:default, remote:<own_id>}`. There is no ancestor walk or "all
  banks" mode; the store API requires an explicit bank list (`22` I5 deny-by-default).
- **M-2** Tasks never read a remote bank and remotes never read a task bank. Cross-boundary reads require an
  explicit operator grant (`memory_grants`, point-to-point, one entry, TTL ≤ 24 h, same server), audited; no
  wildcard/bank-wide grant.
- **M-3** System/global entries are operator-authored or operator-approved; a remote (or any agent) can never write
  them. System content is small, stable, non-secret and prompt-cache friendly.
- **M-4** Remote writes are `observation|note|fact` entries with `trust ∈ {NULL, quarantined, unverified}`;
  `verified/authoritative` require operator promotion. A remote cannot promote; promotion is a separate
  operator-only API in the first release (D: reduce blast radius).
- **M-5** Device/memory text is **data, not instructions**: rendered under a data fence; the prompt says the block
  is untrusted data and cannot grant approvals/tools. This is the OWASP LLM01 boundary, not a filter that "cleans"
  content (`SOURCES.md` [OWASP-LLM01]).
- **M-6** No secrets: write validation rejects secret-shaped keys/values; read rendering redacts; connector
  credentials never enter memory.
- **M-7** Retention/purge: `remote` banks default `unset` (visible, not "forever"); operator may set expire/forever
  with audit; revocation purges credentials/leases immediately and schedules content purge with a receipt
  (content-free journal per `22` I10).
- **M-8** Budgets: prompt render ≤ 1,200 chars stable (global) + ≤ 2,800 chars dynamic (own remote), max 12 entries;
  no task-tree dynamic block for remotes. Write rate/quotas configurable; `E_MEMORY_RATE_LIMIT`/`E_MEMORY_QUOTA`.
- **M-9** Prompt/cache: remote prompt = `system(L1 identity)` → user `[system/global memory, remote facts/state,
  request]`; stable system/global segment first (cacheable), volatile device state last. Memory is never in the
  system message (22 C19/OWASP); no cache claim without measured `prefix_fp` (`25`).
- **M-10** Audit: every read that renders a block records `memory_context_snapshots` (bank ids, `as_of_rev`,
  `prefix_fp`); every write/purge records a journal row; no silent failure (`22` I8/I11).

## 3. Schema (bank-tree extension)

The bank/entry tables themselves are `22`'s bank tables (v14 after the `04` §2 renumbering). This design adds a
minimal additive migration after the bank tree
(version assigned by the Lead; see `04` §2):

```sql
-- Extend the check constraint by table rebuild (SQLite ALTER cannot widen CHECK):
-- memory_banks.kind ∈ ('global','project','run','hive','branch','agent_role','remote')
-- One global bank per installation; one remote bank per remote_id; no parent_bank_id for either.
ALTER TABLE memory_banks ADD COLUMN remote_id TEXT;          -- NULL unless kind='remote'
CREATE UNIQUE INDEX ux_memory_remote_bank ON memory_banks(remote_id) WHERE remote_id IS NOT NULL;

CREATE TABLE memory_grants (
  grant_id       TEXT PRIMARY KEY NOT NULL,
  from_bank_id   TEXT NOT NULL REFERENCES memory_banks(id),
  to_remote_id   TEXT NOT NULL REFERENCES remote_agents(remote_id),
  entry_id       TEXT,                          -- NULL = summaries only (not v1 for remotes)
  capability     TEXT NOT NULL CHECK (capability IN ('read_entry','read_summary')),
  created_by     TEXT NOT NULL,                 -- operator only
  created_at     TEXT NOT NULL,
  expires_at     TEXT NOT NULL,
  revoked_at     TEXT,
  CHECK (expires_at > created_at)
);
CREATE UNIQUE INDEX ux_memory_grant_one_entry ON memory_grants(to_remote_id, entry_id)
WHERE revoked_at IS NULL AND entry_id IS NOT NULL;
```

- `kind='remote'` rows have `scope_id = remote_id`, `parent_bank_id = NULL`, `rev` as the inheritance clock; no
  ancestor contains a remote bank, so no path exists from a task to it.
- A remote's own banking is created at enrolment in the same transaction as `remote_agents` (no reads before the
  bank exists).
- Existing v11 `agent_memory` remains the legacy run/branch-scoped store until the bank migration backfills it;
  remote banks are **not** backfilled from v11 (no remote existed).
- Down migration: drop `memory_grants`, rebuild `memory_banks` without `remote` rows; exercised in tests
  (`migration_remote_memory_down_preserves_task_banks`).

## 4. API surface (read/organizer only in Delta R1)

| Method/route | Purpose | Rules |
|---|---|---|
| `GET /api/memory/remote/{remote_id}` | page entries for a remote bank (`limit<=100`, `before`, `kind`, `trust`) | operator-only reads; remote reads its own through the adapter, not the browser UI |
| `GET /api/memory/system` | page global/system entries | operator only; no credentials |
| `POST /api/memory/system` (WS `MEMORY_SYSTEM_WRITE` in a later delta) | operator writes a system fact | audited; size/kind/trust bounded |
| `POST /api/memory/grants` / `DELETE …/{grant_id}` | create/revoke a point-to-point grant | operator only; TTL; audit |
| `GET /api/memory/audit?remote_id=` | journal/read-set audit | read-only; no content of purged entries |

Remote-side reads happen through the connector protocol (`MEMORY_READ`/`MEMORY_WRITE` verbs in `04`), not through
HTTP routes exposed to the network. The connector enforces `{global, own}` and returns a refusal, never an empty
success, on a denied scope (`E_MEMORY_SCOPE`/`E_MEMORY_WRITE_DENIED`; `22` I11).

## 5. Refusals and tests

Refusal codes: `E_MEMORY_SCOPE_DENIED` (remote requested another bank), `E_MEMORY_WRITE_DENIED` (global/foreign/
authoritative write), `E_MEMORY_QUOTA`, `E_MEMORY_RATE_LIMIT`, `E_MEMORY_GRANT_INVALID`,
`E_MEMORY_GRANT_EXPIRED`, `E_MEMORY_GRANT_REVOKED`, `E_MEMORY_SECRET` (write contained secret-shaped material),
`E_MEMORY_UNAVAILABLE` (read error: no empty-success). Existing `E_MEMORY_SCOPE`/`E_MEMORY_WRITE_DENIED` names from
`20` §7 are reused where already planned; final naming is the contract owner's call (`04` §6).

Acceptance tests (no paid calls, no device):

1. `remote_reads_only_global_and_own` — a mocked remote requesting `run/hive/branch/role` gets
   `E_MEMORY_SCOPE_DENIED`, zero rows. 
2. `task_cannot_read_remote_bank` — a task's store read with the remote bank id returns denied, not empty.
3. `remote_cannot_write_global` — write attempt refused; global row unchanged; audit row.
4. `remote_write_is_quarantined` — new entry has trust `quarantined`/NULL and cannot be rendered as authoritative.
5. `grant_is_point_to_point_ttl_capped` — grant covers exactly one entry/remote, expires, and is audited; a second
   live grant for the same entry is refused (`E_MEMORY_GRANT_INVALID`).
6. `memory_failure_is_not_absence` — DB read error surfaces `E_MEMORY_UNAVAILABLE`; no empty block.
7. `memory_text_is_data_fence` — a fixture containing "approve shell_exec" appears inside the untrusted block and
   changes no grants/approvals; prompt hash test.
8. `secret_shaped_write_refused` — `sk-…`/`Bearer …` keys/values refused `E_MEMORY_SECRET`.
9. `purge_keeps_receipt_not_content` — operator purge removes rows, keeps content hash/receipt, audit row.
10. `system_memory_is_operator_only_and_stable` — remote/task writes refused; system prompt segment hash stable for
    cache; no task-tree content.

## 6. Relationship to task memory (do not blur)

- Tasks keep the v11 run/scope read set (`store.rs:720-900`) and, once v14 lands, the `22` bank read set; remote
  banks are **not** in either set.
- A run's HyperAgent may read device *reports* via the Remote lane/audit (as data), but not the remote's learned
  memory unless an operator grant says so. The "HyperAgent reads all banks" operator preference is outside v1 and
  would need its own summary/audit design (`04` §12; `22` UQ-15) — not implied by this design.
- A remote's own memory never affects task prompts; only the remote's local/remote model sees it. This is the key
  isolation that keeps a compromised remote from poisoning the run tree.
