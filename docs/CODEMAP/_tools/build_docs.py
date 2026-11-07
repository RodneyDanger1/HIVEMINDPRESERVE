#!/usr/bin/env python3
"""Generate the CODEMAP markdown from the frozen census.

Reads docs/CODEMAP/_data/{inventory,summary}.json. Writes only docs/CODEMAP.
"""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
OUT = ROOT / "docs" / "CODEMAP"
DATA = OUT / "_data"
rows = json.loads((DATA / "inventory.json").read_text(encoding="utf-8"))
summary = json.loads((DATA / "summary.json").read_text(encoding="utf-8"))
tracked = [r for r in rows if r["tracked"]]
ignored = [r for r in rows if not r["tracked"]]

KEY = {
    "backend/crates/hiverouter-core/src/execution.rs": "Engine: run lifecycle, planner/task execution, tool loop, approval suspension/resume, side effects, snapshots, spawn queue, terminal reconciliation.",
    "backend/crates/hiverouter-core/src/http.rs": "Boot/HTTP/WS server composition: database, event bus, graph, tools bridge, engine, routes, static UI.",
    "backend/crates/hiverouter-core/src/ws.rs": "WebSocket command dispatch, event fan-out, approval replay/wake, template insert, rewire/context switch.",
    "backend/crates/hiverouter-core/src/tools_bridge.rs": "Approval gate projection, grant verification, preflight/commit side effects, ledgered resolve.",
    "backend/crates/hiverouter-core/src/provider_executor.rs": "DeepSeek adapter: planning/leaf/continuation calls, streaming deltas, tool specs, per-run budget.",
    "backend/crates/hiverouter-core/src/recovery.rs": "Boot crash recovery: interrupted runs, task settlement, graph repair, snapshot revisions.",
    "backend/crates/hiverouter-core/src/db.rs": "SQLite open/migrations/connection policy; schema v4.",
    "backend/crates/hiverouter-core/src/store.rs": "Run/task/message persistence and atomic task-state terminal guard.",
    "backend/crates/hiverouter-core/src/tool_loop.rs": "Tool-call planning/validation, caps, tool specs, bounded result text.",
    "backend/crates/hiverouter-core/src/approval_wait.rs": "In-process approval waiter map and one-shot wake semantics.",
    "backend/crates/hiverouter-contract/src/event.rs": "23-event wire contract incl. NODE topology fields and bounded redacted args preview.",
    "backend/crates/hiverouter-contract/src/command.rs": "17-command contract, validation, decision-gated diff commands.",
    "backend/crates/hiverouter-contract/src/refusal.rs": "Stable E_* refusal table with how_to_fix/retryable metadata.",
    "backend/crates/hiverouter-tools/src/approval.rs": "Approval rows, nonce/args digest, TTL, CAS resolve, ledger primitive tests.",
    "backend/crates/hiverouter-tools/src/side_effects.rs": "Granted file_write/shell_exec preflight, atomic write, allowlisted shell commit.",
    "backend/crates/hiverouter-tools/src/jail.rs": "Workspace jail path resolution, read/list, atomic write.",
    "backend/crates/hiverouter-tools/src/tool.rs": "Tool registry, tiers, allowlists, schemas, permission classes.",
    "backend/crates/hiverouter-tools/src/executor.rs": "Read-class executor for file_read/file_list/report_result/finish.",
    "web/src/state/store.ts": "Zustand store: run/task/message state, approvals, replay/resync, hydration, commands.",
    "web/src/api/contract.ts": "Typed client mirror of commands/events/refusals, decoders, limits.",
    "web/src/api/client.ts": "WebSocket/HTTP client, streaming, ACK/refusal handling, pending-approval pull.",
    "web/src/panels/ApprovalOverlay.tsx": "Approval dialog: evidence, expiry, approve/deny, retry/dismiss, stale banner.",
    "web/src/panels/DataPanel.tsx": "Workspace tree/preview, file/directory distinction, retry state.",
    "web/src/panels/Scrubber.tsx": "History revision scrubber and live-exit control.",
    "web/src/chat/ChatCore.tsx": "Composer, optimistic prompt, ACK/refusal rollback, chat transcript.",
    "e2e/playwright.config.ts": "Playwright projects, web server, mocked/live harness configuration.",
    "e2e/helpers/live-server.ts": "Live server harness, paid gates, allowed origins, binary selection.",
    "tools/qa/verify-harness.mjs": "Structural QA checks for contract/harness evidence pointers.",
    "tools/build-release.ps1": "Sealed release build, hashes, source/seal topology flags.",
    "tools/qa/run-gates.ps1": "Release/pre-release gate orchestration.",
}


def human(n: int) -> str:
    for unit in ("B", "KiB", "MiB", "GiB"):
        if n < 1024 or unit == "GiB":
            return f"{n:.2f} {unit}" if unit != "B" else f"{n} B"
        n /= 1024
    return str(n)


def role(r: dict) -> str:
    p = r["path"].replace("\\", "/")
    ext = r["extension"]
    if p in KEY:
        return KEY[p]
    if ext == ".rs":
        return "Rust source module"
    if ext == ".tsx":
        return "React component"
    if ext in (".ts",):
        return "TypeScript source/test module"
    if ext == ".mjs":
        return "Node ESM script"
    if ext == ".py":
        return "Python tooling/analysis script"
    if ext == ".ps1":
        return "PowerShell build/QA script"
    if ext == ".md":
        return "Markdown documentation/evidence"
    if ext == ".json":
        return "JSON config/data/evidence"
    if ext == ".toml":
        return "TOML manifest/config"
    if ext in (".png",):
        return "Raster UI asset"
    if ext in (".svg",):
        return "Vector UI asset"
    if ext == ".figcraft":
        return "FigCraft design/build asset"
    if ext in (".db", ".db-wal", ".db-shm"):
        return "SQLite runtime data/WAL/SHM"
    if ext == ".exe":
        return "Compiled Windows executable"
    if ext == ".zip":
        return "Archive/artifact bundle"
    if ext == ".pyc":
        return "Compiled Python bytecode"
    if ext in (".out", ".err", ".log"):
        return "Generated command evidence/stream log"
    if ext == ".csv":
        return "CSV data/evidence table"
    if ext in (".txt",):
        return "Plain-text data/evidence"
    if ext == ".mmd":
        return "Mermaid diagram source"
    if ext == ".c":
        return "C fixture/tool source"
    if ext in (".html", ".css"):
        return "Web/static asset"
    if ext == ".sha256":
        return "Hash manifest"
    if ext == ".jsonl":
        return "JSONL event/ledger data"
    if ext == ".lock":
        return "Dependency lockfile"
    if ext == ".prompty":
        return "Prompt asset"
    if ext == ".gitignore":
        return "Git ignore rule"
    return "Repository file"


def catalog_entry(r: dict) -> str:
    rel = r["path"].replace("\\", "/")
    sha = r["sha256"][:16] + "…" if r["sha256"] else "—"
    deep = "DEEP" if r["path"] in KEY else "INDEXED"
    return (
        f"- [`{rel}`](../../{rel}) — {human(r['bytes'])} · sha256 `{sha}` · "
        f"{r['encoding'] or '—'} · {deep} — {role(r)}"
    )


def table_from_area() -> str:
    lines = ["| Area | Files | Tracked | Ignored | Bytes |", "|---|---:|---:|---:|---:|"]
    for name, v in summary["areas"].items():
        lines.append(f"| `{name}` | {v['files']:,} | {v['tracked']:,} | {v['ignored']:,} | {human(v['bytes'])} |")
    lines.append(
        f"| **Total** | **{summary['working_tree_files']:,}** | **{summary['tracked_files']:,}** | "
        f"**{summary['ignored_files']:,}** | **{human(summary['working_tree_bytes'])}** |"
    )
    return "\n".join(lines)


def table_ext(which: str, limit: int = 30) -> str:
    src = summary["extensions"]
    lines = ["| Extension | Files | Tracked | Ignored | Bytes |", "|---|---:|---:|---:|---:|"]
    items = sorted(src.items(), key=lambda kv: kv[1]["files"], reverse=True)
    if which == "tracked":
        items = sorted(
            [(k, v) for k, v in src.items() if v["tracked"] > 0],
            key=lambda kv: kv[1]["tracked"],
            reverse=True,
        )
    for ext, v in items[:limit]:
        lines.append(f"| `{ext}` | {v['files']:,} | {v['tracked']:,} | {v['ignored']:,} | {human(v['bytes'])} |")
    return "\n".join(lines)


def table_biggest(limit: int = 25) -> str:
    lines = ["| Path | Bytes | Tracked |", "|---|---:|---|"]
    for r in summary["biggest"][:limit]:
        lines.append(f"| `{r['path']}` | {r['bytes']:,} | {'yes' if r['tracked'] else 'no'} |")
    return "\n".join(lines)


# 00 — start here
(OUT / "00-START-HERE.md").write_text(
    f"""# HiveMind Codebase Encyclopedia

Frozen at commit [`{summary['head'][:12]}`](https://github.com/) — clean worktree, source version `1.2.0-dev`.

This is the map of **every path in the working tree**, not a marketing overview.

## The partition (exact)

| Universe | Count | Bytes |
|---|---:|---:|
| Git-tracked project files | **{summary['tracked_files']:,}** | **{human(summary['tracked_bytes'])}** |
| Ignored/generated/vendor files | **{summary['ignored_files']:,}** | **{human(summary['ignored_bytes'])}** |
| Working tree (excl. `.git`) | **{summary['working_tree_files']:,}** | **{human(summary['working_tree_bytes'])}** |
| `.git` internals | **226** | volatile, VCS state |
| **Total including `.git`** | **36,706** | — |

`{summary['tracked_files']:,} + {summary['ignored_files']:,} = {summary['working_tree_files']:,}`. That is the exact partition; no path is uncounted.

## Read in this order

1. [`01-SCOPE-AND-CENSUS.md`](01-SCOPE-AND-CENSUS.md) — exact counts, per-directory and extension tables, text/binary rule, CRLF/blob nuance.
2. [`02-ARCHITECTURE.md`](02-ARCHITECTURE.md) — process topology and the full request/run/tool/approval flow.
3. [`03-BACKEND.md`](03-BACKEND.md) — contract, provider, tools, core.
4. [`04-WEB.md`](04-WEB.md) — React UI, store, API, panels, replay/approval UX.
5. [`05-E2E-TOOLS.md`](05-E2E-TOOLS.md) — Playwright, QA gates, release/seal tooling.
6. [`06-DOCS-PLANS-AUDIT.md`](06-DOCS-PLANS-AUDIT.md) — docs/plans/audit/claims corpus and supersession.
7. [`07-RUNTIME-BINARIES.md`](07-RUNTIME-BINARIES.md) — runtime-test, dist, DB/WAL, binaries, assets.
8. [`08-FILE-CATALOG.md`](08-FILE-CATALOG.md) — every tracked file with hash/encoding/role; deep entries flagged.
9. [`09-RISKS-GAPS.md`](09-RISKS-GAPS.md) — verified gaps and unverified claims.
10. [`10-VERIFICATION.md`](10-VERIFICATION.md) — commands, acceptance gates, coverage proof.
11. [`11-AUDITOR-FINDINGS.md`](11-AUDITOR-FINDINGS.md) — the three independent audits and the corrections they forced.

Machine data:
- [`_data/inventory.csv`](_data/inventory.csv) — **{summary['working_tree_files']:,} rows**, every path (tracked and ignored), size, area, extension, tracked flag, and SHA-256 for tracked files.
- [`_data/summary.json`](_data/summary.json) — recomputable tables.
- [`_data/inventory.json`](_data/inventory.json) — same data as JSON.

## What is proven and what is not

**Proven:** the path partition, per-directory counts, per-extension counts, tracked SHA-256 values, encoding census, line-ending caveat, largest files, and Git state.

**Not claimed:** that every generated/vendor file was semantically read line by line. Tracked authored files are indexed and the architecture-critical ones are deeply read; generated files are fully enumerated and family-explained. See [`01-SCOPE-AND-CENSUS.md`](01-SCOPE-AND-CENSUS.md) for the exact depth labels.
""",
    encoding="utf-8",
)

# 01 — scope and census
(OUT / "01-SCOPE-AND-CENSUS.md").write_text(
    f"""# 01 — Scope and Census

## Freeze

- Commit: `{summary['head']}`
- `git status --porcelain`: `{summary['status_porcelain'] or '(clean)'}`
- Source version: `1.2.0-dev` (`web/package.json`, four crate manifests)
- Census generated from a filesystem walk excluding `.git` and excluding the new `docs/CODEMAP/` output, so the numbers describe the repository **before** this encyclopedia was added.
- `.git` is counted separately (226 files) and explained as VCS state, not product source.

## Exact partition

| Layer | Files | Bytes |
|---|---:|---:|
| Tracked | {summary['tracked_files']:,} | {human(summary['tracked_bytes'])} |
| Ignored | {summary['ignored_files']:,} | {human(summary['ignored_bytes'])} |
| Working tree | {summary['working_tree_files']:,} | {human(summary['working_tree_bytes'])} |

## Per-directory census

{table_from_area()}

Notable: `backend/` is 4,754 total because `backend/target/` is a second Cargo target directory (4,633 ignored build files) in addition to the root `target/` (17,244). `tools/` is 9,475 because it contains the ignored `tools/w64devkit/` compiler toolchain (9,452 files), not 9,475 authored tools.

## Extension census — tracked

{table_ext('tracked')}

## Extension census — all files

{table_ext('all')}

## Largest files

{table_biggest()}

## Text vs binary rule

There are two honest rules and they disagree:

- **Git content rule:** 687 tracked files contain no NUL in the first 8,000 bytes → Git treats them as text; 118 as binary.
- **Format rule used by the catalog:** 662 “text-like” tracked files (`.rs/.ts/.tsx/.js/.mjs/.py/.ps1/.json/.toml/.md/.css/.html/.svg/.mmd/.prompty/.txt/.csv/.sha256/.log/.jsonl/.lock/.c/.gitignore`) and 143 “binary/asset” files.

The difference is **33 files**: four NUL-containing `.txt/.json` files are misclassified as text, and 29 plain-text `.out/.err` files are misclassified as binary by the format rule. The catalog uses the format rule for grouping but records the actual encoding per file; `.out/.err` evidence is treated as text evidence. Lockfiles are generated, not authored prose.

## Encoding and line endings

Tracked encoding census:

| Encoding | Files | Bytes |
|---|---:|---:|
| UTF-8 | 655 | 8,751,138 |
| NUL-containing | 96 | 80,718,456 |
| UTF-8 BOM | 32 | 236,091 |
| UTF-16LE BOM | 22 | 65,626 |

`core.autocrlf=true`; there is no `.gitattributes`. Therefore:

- Working-tree bytes and Git blob bytes differ for text files (CRLF vs LF); the tracked-side SHA-256 in this catalog hashes **working-tree bytes**.
- Do not compare it directly with GitHub blob IDs. For commit-stable identity, use `git hash-object`/`git ls-tree`; for content identity on this checkout, use the recorded SHA-256.
- 87 tracked files are CRLF, 24 mixed, 551 LF, 12 no trailing newline (auditor recount); the catalog records encoding, not a normalized line count.
- “177,861 lines” from the first pass = PowerShell non-empty lines; physical newlines ≈189,444. This encyclopedia does not use a single line-count claim.

## Ignored partition (35,675)

| Family | Files | Bytes | Why |
|---|---:|---:|---|
| Root `target/` | 17,244 | 15.78 GB | Cargo build output |
| `backend/target/` | 4,633 | 2.35 GB | second Cargo target dir |
| `tools/w64devkit/` | 9,452 | 716.7 MB | vendored MinGW toolchain |
| `web/node_modules/` | 4,067 | 116.5 MB | npm deps |
| `e2e/node_modules/` | 194 | 18.2 MB | npm deps |
| `web/dist/` | 3 | 0.7 MB | Vite build embedded by rust-embed |
| `dist/` ignored | 15 | ~38.7 MB | release archives/manifests |
| `docs/` ignored | 54 | ~7.4 MB | evidence logs + FigCraft sources/assets |
| `e2e` reports | 4 | ~1.2 MB | Playwright results/report |
| backend logs | 9 | small | command logs |
| **Sum** | **35,675** | **19.97 GB** | exact |

## Depth labels

| Label | Meaning |
|---|---|
| `DEEP` | Architecture-critical file read and explained in the area docs |
| `INDEXED` | Every path is in the catalog with identity/encoding/role; semantics covered by its area doc |
| `STRUCTURAL` | Binary/asset: format, hash, size, provenance, consumer; not line-by-line |
| `FAMILY` | Generated/vendor file: exact path + family row; not semantically re-explained |

No file is omitted from `_data/inventory.csv`.
""",
    encoding="utf-8",
)

# 02 — architecture
(OUT / "02-ARCHITECTURE.md").write_text(
    """# 02 — Architecture

## Process and planes

HiveMind is a local-first single-binary Rust server plus a React client.

```
web/src (React 19 + Vite)
  └─ WsApiClient  ── WebSocket /ws + HTTP /api/* ──┐
                                                    ▼
backend/crates/hiverouter-core/src/main.rs
  └─ http.rs boot: Config → Database → EventBus → GraphAuthority → ToolsBridge
      → RunEngine → Axum router (static UI + API + WS)
          ├─ planner (provider_executor.rs → DeepSeek)
          ├─ task execution + tool loop (execution.rs)
          ├─ approval gate (tools/approval.rs + approval_wait.rs + tools_bridge.rs)
          ├─ side effects (tools/side_effects.rs, jail.rs, shell.rs)
          ├─ SQLite store (db.rs, store.rs) + snapshots
          └─ WS command dispatch (ws.rs) + event fan-out (bus.rs)
```

## The run path

1. Client sends `SUBMIT_PROMPT`; `ws.rs` dispatches to `RunEngine::submit_prompt`.
2. Engine creates a run row (`queued`), user message, active-run slot, and cancellation token.
3. `dispatch_inner` sets `planning`, calls the provider planner, validates the DAG, inserts root + plan tasks, publishes `NODE_SPAWN` frames, transitions `planning → running`.
4. Leaf tasks execute in topological order. Each task calls the provider; streamed deltas become `REPLY_DELTA`.
5. A tool call enters `tool_loop`: plan/validate → budget → tier/persona authorization → read-only executor or approval suspension.
6. Read-only tools execute through `WorkspaceJail`; results become `TOOL_RESULT` and are fed back via `continue_with_tools`.
7. Side-effecting tools request approval (`APPROVAL_REQUEST`), suspend the task as `awaiting_approval`, and wait on `ApprovalWaiters`.
8. `APPROVAL_RESOLVE` consumes the nonce, commits the decision, appends a ledger row, wakes the waiter. Approve → preflight → grant CAS → one effect; deny → denied result; timeout/cancel → terminal path with no effect.
9. Each tool turn persists a `tool_turns` row and increments run counters. Caps: turns/calls/waits, provider `RunBudget`, spawn depth/fan-out/agents.
10. Completion reconciles root/non-leaf tasks, writes a terminal snapshot, publishes one `RUN_STATUS`, then `REPLY_DONE`.

## Invariants

- No fake success: every failure is a typed `E_*` refusal with message/how_to_fix/retryable.
- Single-use approval: nonce + `args_sha256` + `decision IS NULL` CAS; unique pending index.
- At-most-once side effect: preflight before grant; grant consumed exactly once; no retry after commit failure.
- Terminal-state guard: a killed/failed/completed task cannot be overwritten by a late completion.
- One terminal `RUN_STATUS` owner; terminal owner guard in dispatch.
- Unknown is not zero: usage/bytes/limits stay absent when unmeasured.
- Replay is bounded and idempotent; approval replay is server-push after hello, no auto-approve.

## Key source anchors

- `backend/crates/hiverouter-core/src/main.rs` — binary entry.
- `http.rs` — boot/constructors, static UI, routes, recovery catch-up.
- `ws.rs` — command handler, event fan-out, hello replay, template insert, rewire.
- `execution.rs` — engine, suspension, side effects, spawn queue, snapshots, terminal reconciliation.
- `tools_bridge.rs` — approval event projection/grant/commit/ledger.
- `provider_executor.rs` — DeepSeek client, streaming, budget, tool specs.
- `db.rs` / `store.rs` — schema, migrations, run/task/message/tool-turn persistence.
- `web/src/state/store.ts` — client state machine and hydration/replay.
- `web/src/api/client.ts` / `contract.ts` — transport and typed contract.
- `e2e/playwright.config.ts` + `helpers/live-server.ts` — mocked/live test harness.
""",
    encoding="utf-8",
)

# 03 — backend
(OUT / "03-BACKEND.md").write_text(
    """# 03 — Backend

## Workspace

`backend/Cargo.toml` is a workspace of four crates:

| Crate | Role |
|---|---|
| `hiverouter-contract` | Frozen wire types: 17 command names, 23 event names, refusal table, schemas, state/usage/timestamp. No I/O. |
| `hiverouter-provider` | DeepSeek HTTP/SSE client, DTOs, streaming parser, per-run budget, tool-call assembler. |
| `hiverouter-tools` | Tool registry/tiers/schemas, workspace jail, read executor, approval gate, side-effect executor, shell policy, census. |
| `hiverouter-core` | Boot/server/WS/engine/planner/task loop/recovery/history/store/bus. |

Dependency direction: `core → tools/provider/contract`; `tools → contract`; `provider → contract`; `contract` depends on no workspace crate. There are 112 tracked backend files: 102 Rust (14 contract, 16 provider, 24 tools, 48 core) plus 5 TOML, 3 JSON, 1 prompt, 1 lock. The earlier 13/16/23/46 table omitted four tracked `.rs` files (`contract/tests/common/mod.rs`, `core/tests/support/mod.rs`, `core/build.rs`, `tools/benches/engine_benches.rs`); they are in the file catalog.

## Contract crate (`backend/crates/hiverouter-contract`)

- `event.rs` — all event payloads, including `TOOL_CALL`/`TOOL_RESULT`, additive NODE `rev/kind/parent_task_id`, `APPROVAL_REQUEST.args_preview`, bounded redaction builder, 512 UTF-16 cap.
- `command.rs` — client commands, validation, `APPROVAL_RESOLVE`, rewire `expected_rev`, template insert, decision-gated diff commands, client pull note.
- `refusal.rs` — stable `E_*` codes, retryability, how_to_fix.
- `schema.rs` — generated payload schemas and envelope.
- `state.rs`, `usage.rs`, `timestamp.rs`, `run_id.rs`-style ids — shared primitives; unknown stays absent.
- Tests: golden schema, schema roundtrip, UI command matrix, refusal matrix, common fixture.

## Provider crate (`backend/crates/hiverouter-provider`)

- `client.rs` — DeepSeek HTTP/SSE client, auth, retries/budget preflight, cancellation.
- `dto.rs` — request/response/usage/tool-call DTOs.
- `stream.rs` / `budget.rs` — SSE parser and per-run `RunBudget` (`CALL_CAP=40`), `E_CALL_LIMIT`.
- `tool_assembler.rs` — streamed tool-call fragments assembled deterministically.
- Tests cover malformed SSE, budget refusal, usage completeness, streaming order.

## Tools crate (`backend/crates/hiverouter-tools`)

- `tool.rs` — 7 tool ids, 4 tiers, 4 permission classes, allowlists, JSON schemas.
- `executor.rs` — read-class execution only (`file_read`, `file_list`, `report_result`, `finish`).
- `jail.rs` — path normalization/escape prevention, 1 MiB read cap, atomic write.
- `approval.rs` — approval table/indexes, nonce + args digest, TTL, CAS resolve/expire/cancel/label/count, ledger.
- `side_effects.rs` — preflight → grant → one commit; file_write atomic/replace; shell_exec allowlist/argv/NUL/cwd bounds, Job Object.
- `shell.rs` — Windows process containment and allowlisted program policy.
- `census.rs` — derived tool/refusal/tier counts + executor-lane completeness check.
- `health.rs`, `commands.rs`, `watcher.rs` — boot checks, file tree/read commands, filesystem watcher.
- Tests: executor safety, jail escape, approval races, side-effect exactly-once, shell refusal.

## Core crate (`backend/crates/hiverouter-core`)

- `main.rs` / `http.rs` — boot, config, DB migrations, bus, graph, tools bridge, engine injection, Axum routes, static UI, recovery catch-up.
- `execution.rs` — planner dispatch, task DAG, provider streaming, tool loop, approval suspension/resume/denial/timeout/cancel, side-effect call, snapshots, spawn queue, terminal reconciliation, caps.
- `tools_bridge.rs` — approval request projection, `verify_tool_grant`, `execute_approved`, ledgered resolve, pending replay.
- `approval_wait.rs` — one-shot waiter map, duplicate-arm refusal.
- `provider_executor.rs` — planning/leaf/continuation requests, tool specs by task tier, reasoning passback.
- `ws.rs` — WS session, token/origin checks, command dispatch, event fan-out, hello replay, approval wake, template insert, rewire/context.
- `db.rs` — SQLite open/migrations v4, connection policy; `store.rs` — run/task/task-state guard/messages/tool-turns/receipts.
- `recovery.rs` — boot crash recovery: interrupted runs, task settlement, graph repair, snapshots.
- `history.rs`/`snapshot.rs` — revisions/graph snapshots; `bus.rs` — sequenced event bus/replay; `graph.rs` — DAG authority/rev; `run_state.rs` — state machine; `caps.rs` — depth/fan-out/agent/call limits; `prompts.rs` — planner prompt assets; `context.rs` — scoped persona context; `checks.rs`/`config.rs` — boot checks/config.
- Tests: run lifecycle, suspension, tool-loop matrix, approval ledger, WS transport, recovery gates, kill/restore, graph, snapshot, HTTP boot, context switch.

## Refusal and cap model

- Commands validate before dispatch; malformed payloads get typed schema errors.
- Tools fail closed: unknown tier/args never reach jail/side effects.
- Caps are hard, named refusals: `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`, `E_AGENT_LIMIT`, `E_CALL_LIMIT`, `E_BUDGET_EXCEEDED`, `E_TOOL_NOT_ALLOWED`.
- Crash recovery is forward-only; no fabricated historical success.

## Known backend gaps (verified or reported)

- Chat messages are persisted but no HTTP route exposes them; web hydration reports `messages-unavailable`.
- Only `planner` prompt/DAG asset exists; four Library templates still refuse after real lookup.
- No Persona registry loader; scoped `allowed_tool_ids` is the available carrier.
- `TaskRecord` does not carry the authoritative tier, so NODE `kind` is intentionally omitted.
- Tiny spawn-vs-terminal-drain race documented in engine comments.
""",
    encoding="utf-8",
)

# 04 — web
(OUT / "04-WEB.md").write_text(
    """# 04 — Web UI

## Structure

`web/src` has 45 tracked files; `web/tests` has 21. Vite + React 19 + TypeScript, Zustand vanilla store, no router dependency beyond app state.

| Path | Role |
|---|---|
| `main.tsx` / `App.tsx` | Mount, shell, nav/tab rendering, panels |
| `state/store.ts` | Single Zustand store: run/tasks/messages, approvals, history/replay, snapshot hydration, resync, commands, toasts |
| `state/approval.ts` | Pure approval classifier/display/availability/replay contract |
| `state/agentState.ts` | Node-state vocabulary/hierarchy helpers |
| `state/metrics.ts` | HUD derivation |
| `state/strings.ts` | All UI copy; refusal table mapping |
| `api/client.ts` | `WsApiClient`: WebSocket/HTTP, streaming, ACK/ERROR, pending-approval pull |
| `api/contract.ts` | Typed event/command/refusal mirror + decoders |
| `api/index.ts` | Dev-aware API factory; production WS default |
| `flow/Canvas.tsx`, `flow/MiniMap.tsx` | Graph canvas/edges/minimap |
| `panels/ControlBar.tsx` | Prompt composer controls, rewire target selection |
| `panels/ApprovalOverlay.tsx` | Approval dialog, expiry, evidence, approve/deny/retry/dismiss, stale banner |
| `panels/DataPanel.tsx` | Workspace tree, file/dir distinction, retry |
| `panels/Scrubber.tsx` | History revision scrubber + live exit |
| `chat/ChatCore.tsx` | Composer, optimistic message, ACK/refusal rollback |
| `panels/*` | Agents, Settings, History, Routing, ToolActivity, etc. |

## Store state machine

- `submitPrompt` validates, appends optimistic user message, sends `SUBMIT_PROMPT`, waits for ACK; refusal path is surfaced as named toast.
- NODE events update task kind/rev/parent. Missing metadata is honest `unknown`, never guessed.
- `REWIRE_TASK` requires a server node rev; if unknown, one bounded snapshot read, otherwise local `E_RESYNC_REQUIRED` and no command (no CAS loop).
- Boot: snapshot refresh `/api/history/runs` → `/revisions` → `/state?revision=latest`, then socket open; CANCEL_RUN is available from the run row.
- `RESYNC_REQUIRED`: records gap, sets `reconnecting`, starts resnapshot; `SERVER_HELLO` cannot overwrite an unresolved gap to `open`.
- History: `latest=-1` means none; revision 0 is valid; `exitHistoryReplay()` clears replayGraph.
- Chat hydration: messages route does not exist, so `snapshot.incomplete` names it; no fabricated transcript.
- Approval: nonce + resolve state machine; ACK settle timer; retry/dismiss; stale banner; no optimistic removal.

## UI capabilities and gaps

Working: prompt/run/task graph, agent cards, tool activity, approval card/retry, file tree/preview, settings, history scrubber, resync gap, replay exit, responsive shell, accessibility labels.

Known gaps: no message-history fetch route; directory expansion disabled honestly (root-only `FILE_TREE_LIST`); four Library template buttons point at missing assets; child-agent spawn UI has no per-child cancel; persona/tier labels incomplete.
""",
    encoding="utf-8",
)

# 05 — e2e/tools
(OUT / "05-E2E-TOOLS.md").write_text(
    """# 05 — E2E and Tooling

## Playwright

- `e2e/playwright.config.ts` — projects (chrome/msedge), web server, base URL, trace/screenshot policy.
- `e2e/run-guarded.mjs` — grouped gate runner; live/paid specs skip by name unless gates set.
- `e2e/helpers/live-server.ts` — starts the real binary, temp workspace, token/origin options, `paidGate('prompt'|'tool-turn')`, frames/DOM/evidence capture.
- `e2e/helpers/replay-harness.ts`, `ws-probe.ts`, `mock-*` — mocked transport, frame capture, deterministic replay.
- Specs: tool-loop, approval-flow, control-matrix, gui-gates/residual, a11y, trace-hops, ws-contract, replay-state, live-app/streaming/replay/security/resync/tool-turn/trace-capture.
- Skips are explicit named skips for live/paid/axe, not silent passes.

## QA/release scripts (`tools/`)

- `tools/qa/run-gates.ps1` — orchestrated pre-release/release gates; pre-release exits 3, sealed manifest acceptance via checker.
- `tools/build-release.ps1` — source-only/sealed build, manifest v2, hashes, tag topology, `-RecordSeal`.
- `tools/qa/check-seal-topology.mjs` — source-C/seal-D allowlist, tag/diff/artifact verification.
- `tools/qa/verify-harness.mjs` — 28 structural checks incl. evidence-pointer resolution.
- `tools/qa/apply-live-trace.mjs` — capture validation (`E_CAPTURE_EMPTY`, evidence pointers, stale capture).
- `tools/qa/check-docs-truth.mjs` — docs version/claims checks.
- `tools/qa/*tests*` — node:test suites for seal/trace tools.
- `tools/build_assets_index.py`, `figcraft-assets.mjs`, `build-trace-seeds.mjs` — asset indexes and trace seeds.
- `tools/bin/` — tracked built `hiverouter.exe` / `hiverouter-cli.exe`; `tools/cc-test/` C link fixtures.
- `tools/w64devkit/` is ignored vendored MinGW (9,452 files); build scripts require it on PATH on this host.

## Test counts

Backend 475 passed / 0 failed / 2 ignored; web 179/179; guarded E2E 106 passed / 0 failed / 166 named skips; docs 19/19; harness 28/28. See `10-VERIFICATION.md`.
""",
    encoding="utf-8",
)

# 06 — docs/plans/audit
(OUT / "06-DOCS-PLANS-AUDIT.md").write_text(
    """# 06 — Documentation, Plans, and Audit Corpus

This repository carries its own history. Treat these as **claims**, not authority; source code wins on conflict.

| Area | Tracked files | What it is |
|---|---:|---|
| `docs/` | 260 | Product/architecture reports, contracts, trace registry, evidence, UI/Figma-craft assets, diagrams |
| `plans/` | 114 | Foresight runs, seat reports, wave plans, verification notes, compiled ledgers |
| `audit/` | 93 | Prior audit inventories, parts, diagrams, tools; includes stale file manifests |

Important subareas:
- `docs/contracts/` — UI command/event contract docs; counts must match 17/23.
- `docs/trace/` — 319-hop control registry (1 WIRED / 49 UNVERIFIED / 5 MISSING / 264 PENDING) plus live captures; raw capture is empty.
- `docs/evidence/` — local command logs; many ignored, so a fresh clone lacks them.
- `docs/ui/` — FigCraft design sources/assets; 18 ignored canonical deliverables + tracked build `.figcraft` ZIPs.
- `docs/plans/` and `plans/` — design history; some docs describe earlier 1.1/1.2 states and are superseded by later commits.
- `audit/inventory/FILE-MANIFEST*.json` — prior census at a different HEAD (reported 507 tracked vs 805 now); do not use as current oracle.

## Truth rules for this corpus

1. Sealed 1.1.0 artifacts are authoritative only for 1.1.0.
2. Current source is 1.2.0-dev; no 1.2 tag/artifact/seal exists.
3. A plan/report is not proof; it is a claim with a date/commit.
4. Old counts (507 files) are historical, not current.
5. The current oracle is `_data/inventory.csv` frozen at `e021b12`.
""",
    encoding="utf-8",
)

# 07 — runtime/binaries
(OUT / "07-RUNTIME-BINARIES.md").write_text(
    """# 07 — Runtime, Artifacts, Binaries, and Assets

## Tracked binaries/assets

| Family | Count | Tracked bytes | Treatment |
|---|---:|---:|---|
| `.exe` | 10 | 58.1 MB | PE executable; producer/build commit, hash, version, consumer; three `hiverouter.exe` copies are byte-identical |
| `.zip` | 3 | 17.1 MB | Release archives; member manifest + hashes; sealed 1.0/1.1 only |
| `.db` / `.db-wal` / `.db-shm` | 13 each | 2.05 MB | SQLite runtime/fixture residue; schema from `db.rs`, not payload dumps; redaction/reset semantics |
| `.figcraft` | 18 | 1.79 MB | ZIP design build artifacts (`document.json` + `thumbnail.png`); ignored canonical sources are separate |
| `.png` | 20 | 1.49 MB | UI/evidence images; dimensions/IHDR, producer, consumer |
| `.pyc` | 6 | 0.12 MB | CPython 3.14 bytecode under `__pycache__` despite ignore; tracked generated residue |
| `.svg` | 11 | 0.58 MB | Vector assets; XML parse, dimensions, source |
| `.out` / `.err` | 47 | 42 KB | Command evidence streams; plain text despite "binary" grouping; 22 `.err` are empty |
| `.log` | 1 | 1.6 KB | tracked exception (`docs/evidence/live-1.1-paid.log`) |

## Ignored runtime/dist

- Root `target/` 17,244 files/15.78 GB; `backend/target/` 4,633/2.35 GB — Cargo debug/release outputs, `.rlib/.rmeta/.o/.d`, test binaries.
- `web/node_modules/` 4,067, `e2e/node_modules/` 194 — dependencies.
- `tools/w64devkit/` 9,452 — vendored MinGW toolchain (gcc/dlltool); required to compile on this host.
- `web/dist/` 3 files — Vite bundle embedded into the server binary (index/CSS/JS); build input, not arbitrary generated output.
- `dist/` ignored 15 — release archives, hidden manifests, unpacked dirs; 1.0/1.1 only.
- `docs/` ignored 54 — 29 evidence logs, 7 audit logs, 18 FigCraft canonical sources/assets.
- `e2e` generated 4 — test-results/playwright-report.
- `runtime-test/` 82 tracked — SQLite/WAL fixtures, packaged binaries, workspace fixtures; privacy/redaction risk because DB/WAL can contain prompts/approvals.

## Provenance and risk

- Tracked binaries are 86% of tracked bytes (≈77 MiB of 85.6 MiB).
- Three identical `hiverouter.exe` copies and 13 identical 4 KB DBs are redundancy; hashes group them.
- `.pyc` tracked despite `__pycache__/` ignore; source files exist, but staleness should be checked.
- No first-party LICENSE/NOTICE; dependency licenses are inside node_modules/w64devkit, not redistributed in a clean clone.
- No `.gitattributes`; CRLF/LF identity caveat applies to text artifacts.
""",
    encoding="utf-8",
)

# 08 — catalog
catalog = ["# 08 — File Catalog (805 tracked files)", "", "Every tracked file with working-tree SHA-256, size, encoding, depth label and role. `DEEP` files are explained in the area docs; `INDEXED` entries are covered by their area doc. Ignored files (35,675) are in `_data/inventory.csv` and `01-SCOPE-AND-CENSUS.md`.", ""]
by_area: dict[str, list[dict]] = {}
for r in tracked:
    by_area.setdefault(r["area"], []).append(r)
for area in sorted(by_area):
    catalog.append(f"## {area}")
    catalog.append("")
    for r in sorted(by_area[area], key=lambda x: x["path"].lower()):
        catalog.append(catalog_entry(r))
    catalog.append("")
(OUT / "08-FILE-CATALOG.md").write_text("\n".join(catalog), encoding="utf-8")

# 09 — risks
(OUT / "09-RISKS-GAPS.md").write_text(
    """# 09 — Verified Risks and Gaps

## Product/runtime gaps

1. **No chat-history endpoint.** `chat_messages` is persisted but no HTTP route exposes it; web hydration reports `messages-unavailable` and does not invent messages.
2. **Four Library templates missing assets.** Real lookup/insertion exists, but only `planner` has prompt/DAG assets; content-writer/data-analyst/code-reviewer/chatbot still refuse.
3. **Nested file listing absent.** `FILE_TREE_LIST` has no path payload; directories are disabled honestly, not falsely read.
4. **Persona loader absent.** Engine enforces task tier + scoped allowed_tool_ids; no Persona registry is wired.
5. **NODE `kind` absent.** Contract supports it; TaskRecord lacks authoritative tier, so producers omit rather than guess.
6. **Worker spawn policy.** `spawn_agent` is in Hyper/Super allowlists; worker-role tasks are correctly refused.
7. **Spawn-vs-terminal race.** A spawn accepted between queue-drain and terminal status can leave a queued task settled without execution; documented in code.
8. **Memory/research/learning loops absent.** No product code or tables.

## Proof/release gaps

- No 1.2 live paid proof, no live tool-turn/approval/security/resync run.
- No 1.2 tag, artifact, manifest, or seal; only sealed 1.1.
- Trace: 1 WIRED / 49 UNVERIFIED / 5 MISSING / 264 PENDING; raw capture empty.
- Axe packages absent; G-UI-3/G-UI-10/axe exclusions open.
- Key rotation/runtime-test purge are operator actions.
- `docs/CODEMAP` is documentation, not product verification.

## Data/security risks

- Tracked SQLite/WAL/SHM fixtures can contain prompts/approvals; redaction/reset policy needed.
- Three identical EXEs and 13 identical DBs inflate repo; provenance/hashes recorded.
- 11 fixture-token hits are fake test values; no real secrets found in pattern scan. `.env*` absent.
- No first-party LICENSE/NOTICE; dependency/license census incomplete.
- No `.gitattributes`; CRLF/LF makes hashes/citations unstable across checkouts.

## Documentation risks

- `audit/inventory/FILE-MANIFEST*.json` is stale (507 tracked at an older HEAD); current is 805 at `e021b12`.
- Many plans/reports describe intermediate states; only source + frozen inventory are current.
- Earlier “404/441 backend” and “5 failed web” claims are superseded by the committed final 475/0, 179/179, 106/0 receipts.
""",
    encoding="utf-8",
)

# 10 — verification
(OUT / "10-VERIFICATION.md").write_text(
    """# 10 — Verification

## Frozen census commands

```powershell
git rev-parse HEAD
git -c core.quotePath=false ls-files -z    # 805
Get-ChildItem -Recurse -File -Force | Measure-Object   # includes .git; exclude it for 36,480
python docs/CODEMAP/_tools/build_codemap.py            # rebuilds _data
```

## Acceptance checks

| Check | Expected | Result |
|---|---|---|
| Tracked | 805 | 805 |
| Working tree excl. `.git` | 36,480 | 36,480 |
| Ignored | 35,675 | 35,675 |
| `.git` files | 226 | 226 |
| Total incl. `.git` | 36,706 | 36,706 |
| Partition | 805 + 35,675 = 36,480 | pass |
| Tracked bytes | 89,771,311 | pass |
| Ignored bytes | 19,971,803,372 | pass |
| Working-tree bytes | 20,061,574,683 | pass |
| Catalog rows | 805 | 805 |
| Inventory rows | 36,480 | 36,480 |
| Catalog hashes | all tracked rows have SHA-256 | pass |
| Encoding census | 655 UTF-8 / 96 NUL / 32 UTF-8-BOM / 22 UTF-16LE | pass |
| Text/binary reconciliation | 33 format-rule outliers named | pass |
| `.git` scope | separate, not claimed as product source | pass |

## Committed product gates (source: `docs/RELEASE-EVIDENCE.md`)

- `cargo check --workspace --tests --locked` exit 0.
- `cargo test --workspace --locked`: **475 passed / 0 failed / 2 ignored**.
- web `tsc --noEmit` exit 0; vitest **179/179**.
- guarded E2E **106 passed / 0 failed / 166 named skips**.
- docs truth **19/19**; QA harness **28/28**.
- No 1.2 release build/tag/seal/live proof.

## What this document does not verify

- Runtime behavior of the current binary (no live server run for this map).
- The content/semantics of every generated/vendor file (family-level only).
- Visual fidelity of design assets (needs human review).
- Hardlink groups beyond duplicate-hash detection.
""",
    encoding="utf-8",
)

# 11 — auditor findings
(OUT / "11-AUDITOR-FINDINGS.md").write_text(
    """# 11 — Independent Auditor Findings and Corrections

Three read-only auditors were run before this document: evidence-check, evidence-gap-audit, research (research was stopped before final report; its partial findings are marked).

| Claim from draft plan | Auditor result | Correction applied |
|---|---|---|
| 805 tracked | VERIFIED | kept |
| 36,480 working tree | VERIFIED | kept |
| 36,706 incl. `.git` | VERIFIED | kept |
| 662 text / 177,861 lines | Numerically reproducible, but “text” was an undisclosed extension rule; lines were non-empty only | Catalog records encoding; no single line-count claim; 33 outliers named |
| 143 binary / 80.76 MB | Numerically reproducible, but 29 are plain-text `.out/.err` | Treated as evidence text, not structural binaries |
| Area table root 7 | CONTRADICTED: actual root 4 + `.cargo` 1; total 805 not 807 | corrected table |
| Backend 13/16/23/46 `.rs` | CONTRADICTED: raw 14/16/24/48; omitted 4 `.rs` | catalog includes all 102 Rust files |
| Ignored components 17,244/9,475/4,141 | VERIFIED total but components mixed tracked/total; 4,909 unnamed | full 35,675 family table with sum |
| Tier-3 “all files” | CONFLICT: 35,675 generated files cannot all get prose | depth labels: DEEP/INDEXED/STRUCTURAL/FAMILY; every path in inventory |
| No manifest/oracle | MISSING | `_data/inventory.csv` + summary + rebuild script |
| Encoding/CRLF/dedupe | MISSING | encoding field + core.autocrlf caveat + duplicate groups |
| `.git` internals | MISSING | separate scope, 226 files counted |
| Secrets/license/release | MISSING | risk section + no-first-party-license note |
| Research auditor | Stopped before final report | findings not claimed; gap remains documented |

## Auditor-verified numbers used

- 0 untracked, clean at `e021b12`, 49 commits, 3 tags.
- 5,571 directories, 942 empty.
- 149 paths with spaces; 0 reparse points/symlinks.
- 40 non-UTF-8 tracked files (22 UTF-16LE + 18 ZIP `.figcraft`), 31 UTF-8 BOM, 41 NUL-containing text; 551 LF / 87 CRLF / 24 mixed / 12 no-newline.
- 433 empty files (29 tracked); 8 duplicate hash groups / 22 redundant files / 23.07 MB.
- 18 ignored design deliverables = 22,009,234 B; 18 tracked build `.figcraft`; 6 tracked `.pyc`.
- 13 DB + 13 WAL + 13 SHM; 3 identical EXEs; 13 identical 4 KB DBs.
- 7,236 extensionless files (auditor total incl. different scope); this inventory records 7,051 extensionless in the frozen working tree.
- 99 dependency license files; no first-party LICENSE/NOTICE.
- 11 fixture-token hits, 0 real secrets; `.env*` absent.
- Prior audit FILE-MANIFEST 507 tracked at older HEAD vs current 805.
""",
    encoding="utf-8",
)

print("wrote CODEMAP docs")
