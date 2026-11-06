# Live server/provider smoke evidence (no key material)

- Date: 2026-11-06 (America/Chicago)
- Binary: `target/debug/hiverouter.exe`, bind `127.0.0.1:8799`, fresh data directory `.live-smoke-data`
- Key: inherited only from the process environment; never printed/logged.
- Transport: HTTP `/health` → `ok`, `/api/session` token present; WebSocket `/ws` with allowlisted Origin; hello accepted.
- Command: `SUBMIT_PROMPT` (`Reply with exactly HIVE-LIVE-OK`, project `smoke`)
- Observed event sequence: `RUN_CREATED` → `ACK` → `NODE_SPAWN` ×2 → `NODE_STATE` → `REPLY_DELTA` → `REPLY_DONE` → `NODE_STATE` → `RUN_STATUS`
- Final run status: **completed**; error code: none
- Reply: `HIVE-LIVE-OK`
- Usage: prompt 89, completion 6, total 95 (cache hit 0, miss 89); reasoning tokens absent
- Provider call is real; no mock is used in the binary path. A terse prompt initially exposed an empty-content planner response; the planner prompt was hardened to wrap the user task in an explicit DAG instruction, after which the same terse prompt completed.
- Replay was subsequently verified through the release binary: `HISTORY_REVISION_SET` revision 0 returned a 2-node `HISTORY_SNAPSHOT`; the web canvas projects it read-only.
- Current limitation: the adapter returns the leaf reply as one delta after a non-streaming completion; true SSE chunk-by-chunk rendering is 1.1 and is not claimed.
- Final artifact hashes and sealed source commit are in `docs/RELEASE-EVIDENCE.md`.
