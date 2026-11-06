# GATE-CY-3 — Cycle 3: Recon → Plan → Lead (product files, step 4)

Date 2026-11-06 11:1x–11:3x CST · HEAD `f4c780d` · Seat S2.

## RECON
Read each assigned product file fully, one at a time (metadata + sha256 + line count in READLOG rows 9–20):
`command.rs` (533→569 displayed lines), `event.rs` (489→521), `state.rs` (147→157), `refusal.rs` (314→336),
`schema.rs` (360→381), `envelope.rs` (175→191), `core/src/ws.rs` (538→572), `core/src/http.rs` (427→448),
`tools/src/settings.rs` (56→64), `tools/src/health.rs` (416→452, three segments), `core/tests/ws_transport.rs`
(440→483), `web/src/api/contract.ts` (825→875, two segments).
Recon facts (current source): frozen `CommandName` = 14 names + 3 decision-gated `DIFF_*` (17 total) —
`command.rs:26-61,65-83`; `EventName` = **21** (`event.rs:18-61`); `AgentState` = **16** incl. `Interrupted`
(`state.rs:18-51`); `Refusal` = **65** (`refusal.rs:73-282`; counted 8+39+4+6+5+2+1); envelope carries
schema/type/name/id/correlation_id/run_id/task_id/seq/ts/payload; `Event::ERROR.code` is a `String`, not the
`Refusal` enum. `ws.rs` gates: loopback peer → Origin allowlist → token query before upgrade, hello ≤10 s,
replay/resync; dispatch ACK-after-accept for rewire/submit/spawn/cancel/kill/settings/files/approval/history.
`http.rs` wires `ProviderExecutorAdapter::from_env` when provider enabled and falls back to honest
`UnconfiguredExecutor` (`E_KEY_MISSING`) otherwise; `/health` exposes status/version/schema/uptime/checks/
database(path+schema_version)/transport(last_seq, graph_nodes, allowed_origins count); `/api/session` returns
the boot token to loopback/allowlisted Origin; `/diagnose` is tools-owned. `settings.rs` returns
`SettingsStatusResult{provider,model,key_present,thinking,reasoning_effort}` using `ApiKey::present_in_env()`
only. `ws_transport.rs` tests: origin/token refusal, two-client fan-out, hello replay+gap resync, invalid-JSON
error path with usable connection, rewire ACK + cycle refusal with graph unchanged, 1,000 rewires, cancel-run
refusal, settings truthfulness (asserts no `sk-`/`DEEPSEEK_API_KEY`), jailed file tree/read/missing/escape,
unavailable workspace refusal, approval replay refusal. `contract.ts` lists 14 frozen commands, 21 events,
**45** refusal codes, tolerant decoders for SETTINGS_STATUS_RESULT (bool key only) and HISTORY_SNAPSHOT.

## PLAN
- Verify the three headline parity claims by counting source variants (14/21/65/16) — done in CY-3.
- Audit key material across every assigned wire/log/UI/DB path with class greps
  (`DEEPSEEK_API_KEY|sk-|Bearer|api_key|key_present|expose`) and by reading producers; classify each path.
- Verify SETTINGS_STATUS truthfulness end-to-end (tools payload → ws merge → contract.ts decode → test assertion)
  and HISTORY_SNAPSHOT truthfulness (payload shape → ws emit → contract.ts decode).
- Record remaining defects with `path:line`, severity, and owner; do not fix (read-only).

## LEAD
- All 12 product files read (full file, one at a time, READLOG rows 9–20) — PASS; two contract.ts/health.rs
  reads were segmented for size but cover the whole file.
- Parity counts verified from source (14 frozen / 21 / 65 / 16) — PASS.
- Key-material audit: no producer path in the slice reads/echoes the key; only `key_present` booleans,
  `ApiKey::present_in_env`, and the deliberate session token (auth material, not the API key). Tests assert
  absence. — PASS with findings below (nonce/state/refusal-table parity, token semantics, hardcoded
  NODE_REWIRE state; no key-leak defect found in the slice).
- **Cycle 3: PASS** (no failed cycle; findings are recorded defects, not blockers).
