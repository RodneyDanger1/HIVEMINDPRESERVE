# EVIDENCE — b-core seat (read-only audit)

Checkpoint: HEAD `3988622` (verified via `git rev-parse --short HEAD`). Writes: only `audit/parts/b-core/*` in the
repo. No product, test, docs, ledger or git writes; **no rebuilds, no test runs, no installs, no paid calls**.

## C-01 Assignment + manifest cross-check (54 files)

```
$ Get-Content audit/inventory/ASSIGNMENTS.json | ConvertFrom-Json -> seat 'b-core'
seat count: 54          DEEP (cross-referenced with audit/inventory/DEEP-LIST.md): 21
$ per-file: Get-FileHash -Algorithm SHA256 + manifest CSV compare
TOTAL=54 DEEP=21 MISMATCH=0          # every assigned file hash equals FILE-MANIFEST.csv
```

Manifest rows also gave bytes/language/depth used in FILES.md metadata; extension/LOC computed locally. The 21 deep
files are exactly the contract source set (7) + core source set (14) named in DEEP-LIST.md for this seat.

## C-02 Structure / dependency evidence

```
$ Select-String -Path backend/crates/hiverouter-core/src/*.rs -Pattern 'fn build_router|AppState::|RunEngine::new|
  Database::open|EventBus::new|GraphAuthority::new|SnapshotStore::new|ToolsBridge::|recover_on_boot|HandshakeLog::new|
  ContextStore::new|TemplateRegistry::|publish_for_run|process::abort'
-> anchors used in FILES.md/DEPENDENCIES.md, e.g. http.rs:224 build_router, http.rs:411 serve, main.rs:54/79/81/117,
   execution.rs:208 ContextStore::new, ws.rs:400-429 result events, recovery.rs:42 recover_on_boot.
$ Select-String -Path backend/crates/hiverouter-core/tests/*.rs -Pattern '^async fn |^fn '
-> 63 test/harness functions listed (used for caller matrix; e.g. ws_transport.rs test 13 functions, run_lifecycle 6,
   kill_restore 3 incl. ignored helper, snapshot_history 4).
```

Deep files were additionally read earlier in their implementation/closeout work (command.rs/envelope.rs/event.rs/
refusal.rs/schema.rs/state.rs/bus.rs/context.rs/db.rs/execution.rs/graph.rs/handshake.rs/http.rs/main.rs/prompts.rs/
recovery.rs/snapshot.rs/static_ui.rs/tools_bridge.rs/ws.rs) and re-confirmed by hash + grep this audit. The 1,633-line
`schemas/contract-v1.json` was **not** line-audited (generated; drift-gated by `tests/golden_schemas.rs:49`) — marked
UNVERIFIED semantically in FILES.md.

## C-03 Repo state during the audit (read-only checks)

```
$ git rev-parse --short HEAD        -> 3988622
$ git status --porcelain            -> 1 entry at start
$ git status --porcelain (later)     -> ?? audit/          # the audit directory itself (untracked by design)
```
The only untracked path observed after audit writes is `audit/`; no tracked product file was modified by this seat.

## C-04 Behavioural claims cross-reference (not re-executed this audit)

Per the read-only mandate, no test binary was run and no server was started **in this audit**. Behavioural statements in
FILES.md cite the closeout evidence already recorded in-repo (READ, not re-VERIFIED here):

| Claim | In-repo evidence | Audit class |
|---|---|---|
| contract suite 32 passed | `docs/plans/PHASE-1-EVIDENCE.md` §4; `docs/claims/CLAIM-REGISTER.csv` C-320 | READ |
| core suite 83 passed / 1 ignored | `docs/plans/D2-D3-WIRING-EVIDENCE.md` §2 (TEST_EXIT=0) | READ |
| WS transport 13 tests incl. D-2 commands | D2-D3 evidence §2; `ws_transport.rs` test names | READ |
| kill-restore with real `abort()` | `docs/plans/PHASE-9-EVIDENCE.md` §2; `kill_restore.rs:22-48` | READ |
| live `/diagnose` + WS command probe | D2-D3 evidence §3 (quoted curl/ClientWebSocket output) | READ |
| release build exit 0 + hash | `PHASE-1-EVIDENCE.md` §10a; C-334 | READ |

## C-05 Files not read / limits

- `backend/crates/hiverouter-contract/schemas/contract-v1.json` — generated artefact, structure audited through its
  producer and drift test; not line-by-line (UNVERIFIED detail).
- No binary files are in this partition; nothing was skipped as unreadable.
- Deep-file paragraph analyses are based on source reading + grep anchors, not on new execution; any runtime-only
  property (concurrency stress, live provider streaming) is marked UNVERIFIED in FILES.md/FINDINGS.md.

## C-06 Process notes / deviations

- One shell command used a relative path after changing `$root` without `Set-Location`; it created the empty directory
  `C:\Users\Riley\Desktop\audit\parts\b-core` **outside the repository** before failing. **No file was written there**
  (zero bytes of content); the repo writes all target `C:\Users\Riley\Desktop\hiverouter-v1\audit\parts\b-core\`.
  Disclosed rather than silently deleted (read-only/no-destructive mandate).
- Total seat outputs: FILES.md (54 sections, 498 lines, 21 deep analyses), DEPENDENCIES.md, QUALITY.csv (54 rows),
  FINDINGS.md (F-01…F-11), EVIDENCE.md (this file).
