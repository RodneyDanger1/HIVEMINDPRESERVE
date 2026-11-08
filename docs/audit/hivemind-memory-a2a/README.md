# Memory / A2A / HyperAgent access — audit index

**Task:** `task-14` · **Mode:** read-only source audit · **Owner write scope:** `docs/audit/hivemind-memory-a2a/**`
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `7610b86` · **Audit date:** 2026-11-08
**Working-tree caveat:** `backend/crates/hiverouter-core/src/execution.rs` is modified (uncommitted root-HyperAgent final-turn diff, +84/−15). Findings that cite `execution.rs:1543-1588` apply to this working tree; re-read if the diff is reverted.

## Files

| File | Purpose |
|---|---|
| [`AUDIT.md`](AUDIT.md) | 12 findings (M-01…M-12) with severity, `file:line` evidence, current behavior, exact fix, and references. Includes positive controls and a surface-parity matrix. |
| [`TEST-PLAN.md`](TEST-PLAN.md) | Exact new/extended test files, test names, fixtures, assertions, commands, acceptance criteria. T-P0-01…T-P2-11. |
| [`FINDINGS.json`](FINDINGS.json) | Machine-readable findings for triage import. |

## Method

Read-only source inspection of `backend/crates/hiverouter-core/src/{execution,store,context,http,db}.rs`, `backend/crates/hiverouter-tools/src/{tool,cli,census}.rs`, `tools/hivemind-cli.mjs`, `web/src/panels/{AgentMemoryPanel,AgentMessagesPanel}.tsx`, `web/src/state/store.ts`, existing tests, and the relevant HMARS/requirements docs. No build, test, server, migration, browser, provider call, or git mutation was performed. No claim below is a runtime result.

## Verdict in one paragraph

The implemented v11 `agent_memory` feature adds a real scope column, but it is **not yet a safe branch memory**: writes and reads key on the **parent task** (siblings share a bank, ancestors are not inherited), the root cannot aggregate branch/hive banks, and model/peer text is injected under an **“authoritative”** header with only a four-marker secret filter. A2A is **write-only**: `ask_parent` has no answer/wake, inbox refresh stops after task start, terminal recipients accept messages, and the message budget is racy. Clear is a one-click/one-call **global** role delete with no scope, confirmation, or receipt. The web UI exposes only part of the API/CLI surface (no roster, no sender/recipient filters, no scoped clear) and several memory/roster reads are cacheable.

## Highest-priority fixes

1. **M-01** (Critical): single redaction/escaping trust boundary for A2A and memory; stop calling model text authoritative.
2. **M-02** (High): replace parent-id scope with own-bank + explicit ancestor/role read set (or the full bank tree).
3. **M-04** (High): add inbox delivery status, refresh-before-continuation, wake/answer/expiry.
4. **M-05** (High): scoped clear with confirmation, receipt/audit, and run/branch targeting.
5. **M-03** (High): root aggregate read set with explicit truncation, not silent `take(32)`.
6. **M-06** (High): separate role vs run/branch banks and migrate legacy scopes.
7. **M-07** (High): validate recipient state and make the message budget atomic.

## Non-claims

No runtime behavior, exploitability, performance, or test pass is claimed. Line references are to the baseline working tree. The only write performed by the audit seat is under this directory.
