# D-2 / D-3 WIRING EVIDENCE — tools commands over WS + live `/diagnose`

Date: 2026-11-06 (America/Chicago). Executor: B-Core. Verifier defects: **D-2** (core WS dispatch returned
`E_INTERNAL_ERROR` for `SETTINGS_STATUS`/`FILE_TREE_LIST`/`FILE_READ`/`APPROVAL_RESOLVE`) and **D-3** (`/diagnose` not
mounted). Both are fixed in `hiverouter-core` by consuming the existing `hiverouter-tools` handlers (no tools-crate
edits, no `web/` edits).

## 1. What changed

- `backend/crates/hiverouter-core/src/tools_bridge.rs` (new): single core→tools wiring for the four patched commands
  plus `/diagnose`. `ToolsBridge::new` opens the `hiverouter-tools` `WorkspaceJail` against the configured
  `--workspace-root` (default: process working directory); a missing/non-directory root leaves file access genuinely
  unavailable. `ToolsBridge::migrate` creates the tools-owned `approvals` table through the core database connection.
- `backend/crates/hiverouter-core/src/ws.rs`: dispatch arms now call the tools handlers and emit
  `ACK + SETTINGS_STATUS_RESULT` / `ACK + FILE_TREE_RESULT` / `ACK + FILE_CONTENT` / `ACK + APPROVAL_RESOLVED`, or the
  truthful named refusal on failure. No path returns `E_INTERNAL_ERROR` for these commands any more; the only
  `E_INTERNAL_ERROR` left is the defensive fallback for a command that should already be handled.
- `backend/crates/hiverouter-core/src/db.rs`: `Database::with_connection_refusal` lets a tools-layer business refusal
  (`E_APPROVAL_REPLAY`, `E_APPROVAL_TIMEOUT`, …) pass through unchanged instead of being flattened into a database error.
- `backend/crates/hiverouter-core/src/http.rs`: `GET /diagnose` mounted on the same server; payload is
  `hiverouter_tools::health::diagnostics(db_ok, key_present)` (derived census, no key material).
- `backend/crates/hiverouter-core/src/config.rs`: `--workspace-root` / `HIVEROUTER_WORKSPACE_ROOT` (default: working
  directory) and `Config::resolve_with_workspace` for fixtures.
- `backend/crates/hiverouter-contract/src/refusal.rs` + `schema.rs`: new truthful refusal
  `E_FILE_TREE_UNAVAILABLE` (`FileTreeUnavailable`), registered on the `FILE_TREE_LIST`/`FILE_READ` matrix rows;
  `docs/contracts/UI-COMMANDS-v1.md` §5 updated with the code.

## 2. Tests (observed)

`cargo test -p hiverouter-core -p hiverouter-tools` → `TEST_EXIT=0`:

```
running 34 tests   (lib)                                  test result: ok. 34 passed; 0 failed
chat_store 5 / context_switch 2 / graph_authority 7 / handshake 2 / http_boot 6
kill_restore 2 (+1 ignored helper) / prompt_assets 2 / run_lifecycle 6 / snapshot_history 4
ws_transport 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.49s
hiverouter-tools: running 77 tests                        test result: ok. 77 passed; 0 failed
hiverouter-tools audit_jsonl: running 1 test              test result: ok. 1 passed; 0 failed
```

New D-2 tests in `tests/ws_transport.rs` (all green):

| Command | Test | Asserted |
|---|---|---|
| `SETTINGS_STATUS` | `settings_status_command_returns_result_over_ws` | ACK + `SETTINGS_STATUS_RESULT` (`provider=deepseek`, `model=deepseek-flash`, boolean `key_present`); payload never contains a key |
| `FILE_TREE_LIST` | `file_tree_and_file_read_commands_use_the_jail` | ACK + `FILE_TREE_RESULT` listing the real workspace file |
| `FILE_READ` | same test | ACK + `FILE_CONTENT` (`text`, `bytes`, 64-char `sha256`); missing file → `E_PATH_NOT_FOUND`; escape → `E_PATH_OUTSIDE_JAIL` |
| unavailable workspace | `file_commands_refuse_by_name_when_workspace_is_unavailable` | `E_FILE_TREE_UNAVAILABLE` for both file commands, explicitly asserted `!= E_INTERNAL_ERROR` |
| `APPROVAL_RESOLVE` | `approval_resolve_command_acks_and_emits_resolved_event` | ACK + `APPROVAL_RESOLVED` (approval id, `tool=file_write`, `decision=approve`); replaying the nonce → `E_APPROVAL_REPLAY` |

New D-3 test in `tests/http_boot.rs`: `diagnose_route_returns_tools_json` — `GET /diagnose` 200 with
`schema == "hiverouter.diagnose.v1"`, derived census/tool fields present, no key material in the body.

## 3. Live release probe

Release build: `cargo build --release --locked` → `Finished release profile ... in 2m 52s`, `RELEASE_EXIT=0`;
`target/release/hiverouter.exe` = **9,329,152 bytes**, sha256
`DE676AFB12B8F79620FBC6D98C6C6E493883931FA79BC33676F28BA47BFD93C3`;
`target/release/hiverouter-cli.exe` = 5,364,224 bytes, sha256
`51E1ABA6DC7349BFB8904014D7007015E13642EA9710F04D380525DCCBD9B9AD`.

Started: `hiverouter.exe --bind 127.0.0.1:8816 --data-dir %TEMP%\hm-d2d3\data --workspace-root %TEMP%\hm-d2d3\ws`
(a workspace containing `note.txt`), then probed with curl, the shipped CLI, and a real
`System.Net.WebSockets.ClientWebSocket` (Origin header set, boot token from `/api/session`):

```
LIVE_HTTP GET /            -> 200
LIVE_HTTP GET /health      -> 200
LIVE_HTTP GET /diagnose    -> 200
LIVE_HTTP GET /api/session -> 200
/diagnose body: {"schema":"hiverouter.diagnose.v1","version":"0.1.0","db_ok":true,
                 "provider_key_present":false,... "refusals":{"count":65,... "E_FILE_TREE_UNAVAILABLE" ...},
                 "tools":{"count":7,...},"capability_census":{...}}
CLI diagnose -> {"schema":"hiverouter.cli.v1","ok":true,"command":"diagnose",
                 "data":{"url":"http://127.0.0.1:8816/diagnose","response":{"schema":"hiverouter.diagnose.v1",...}}}
CLI_DIAGNOSE_EXIT=0

WS SETTINGS_STATUS   -> ACK (correlation matched) + SETTINGS_STATUS_RESULT: provider=deepseek model=deepseek-flash key_present=False
WS FILE_TREE_LIST    -> ACK + FILE_TREE_RESULT: root=...\hm-d2d3\ws entries=note.txt
WS FILE_READ         -> ACK + FILE_CONTENT: text="hello live jail" bytes=15 sha_len=64
WS FILE_READ missing -> ERROR E_PATH_NOT_FOUND
WS APPROVAL_RESOLVE  -> ACK + APPROVAL_RESOLVED: tool=file_write decision=approve id_match=True
WS APPROVAL replay   -> ERROR E_APPROVAL_REPLAY
server stdout: hiverouter 0.1.0 listening on http://127.0.0.1:8816 (db ...\hm-d2d3\data\hiverouter.db, schema v3, contract hiverouter.v1)
```

No WS command returned `E_INTERNAL_ERROR`; the only unnamed failures were the expected named refusals.


## 4. Done / Not done / Unverified

| Done | Evidence |
|---|---|
| D-2 fixed: all four patched commands work over the real WS transport with ACK + result/refusal | §2 (13 ws_transport tests) |
| D-3 fixed: live `/diagnose` route returns tools JSON | §2 + §3 |
| Truthful named refusals instead of `E_INTERNAL_ERROR` | `E_FILE_TREE_UNAVAILABLE`, `E_APPROVAL_REPLAY`, `E_PATH_*` asserted in tests |
| core + tools suites green; release build + sha; live probes | §3 |

| Not done (reason) | |
|---|---|
| browser session against the live server | QA/frontend e2e owns the browser click-through; the WS/HTTP paths are verified here |

| Unverified | What would prove it |
|---|---|
| live `APPROVAL_RESOLVE` against a real tool-call approval created by a provider run | needs a live provider key (Phase 5 live call); the wire path itself is covered by the WS test |
