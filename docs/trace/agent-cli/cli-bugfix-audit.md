# Agent CLI bug-fix audit (offline)

Skills applied: `/forge`, `/integration-logic`, `/logic-gatekeeper`,
`/evidence-check`, `/evidence-gap-audit`, `/agent-native-design`,
`/agent-experience`. No server/browser/provider was started; runtime claims are
UNVERIFIED by explicit instruction.

## Claims and evidence

| Claim | Evidence | Status |
|---|---|---|
| CLI has 45 commands | static registry keys in `tools/hivemind-cli.mjs` | VERIFIED (source) |
| CLI syntax parses | `node --check tools/hivemind-cli.mjs` exit 0 | VERIFIED |
| Start dry-run has no side effects | source order: resolve -> plan -> return before health/mkdir/state | VERIFIED (source) |
| HTTP includes Origin for provider routes | `httpJson` sets `origin: baseUrl.origin` | VERIFIED (source) |
| WS sends Origin/Host/Accept | `MiniWebSocket._connect` | VERIFIED (source); live handshake UNVERIFIED |
| Start/stop identity checks use database path | `startServer`/`stopServer` | VERIFIED (source); live UNVERIFIED |
| Stop auto-escalates and reports graceful:false | `stopServer` | VERIFIED (source); live UNVERIFIED |
| Exit table aligns with Rust CLI 0/2/3/4 | `EXIT` constants + docs | VERIFIED (source) |
| `/health` exposes pid/boot_id/data_dir/workspace_root | `HealthResponse` + additive assertions | VERIFIED (source); not run |
| CLI checks boot identity on start/stop | `startServer`/`stopServer` | VERIFIED (source); live UNVERIFIED |
| Full runtime command verification | not run | UNVERIFIED |

## Gap matrix

| Finding | Gap class | Action |
|---|---|---|
| `start --dry-run` previously probed health/wrote state | CONFLICT (fixed) | dry-run now returns before health/mkdir/state |
| HTTP provider calls had no Origin header | MISSING (fixed) | all `httpJson` calls send Origin |
| `start --force` could bypass instance conflict | WEAK (fixed) | conflict now unconditional |
| `taskkill` without `/F` does nothing on Windows console children | MISSING (fixed by design) | graceful attempt then one automatic force escalation; `graceful:false` |
| stop required `--yes` even for own instance | WEAK (fixed) | own stop is explicit; `--force` on external requires `--yes` |
| external state with no pid returned a successful no-op stop | CONFLICT (fixed) | explicit `E_NOT_OWNED` |
| boot-id check was skipped for legacy state without db_path | WEAK (fixed) | identity checks run when either db_path or boot_id exists |
| human help went to stdout | WEAK (fixed) | help is stderr; JSON-only stdout |
| new approvals/templates/workspace/active-run reads return honest empties | OK | routes implemented; no nonce consumption |
| exe search missed repo-root `target/release` | MISSING (fixed) | search order widened |
| CLI envelope lacked top-level `schema` | MISSING (fixed) | `schema: hivemind.cli.v1` on success/failure |
| `--workspace-root` absent from server help | MISSING (fixed) | help text and `.cmd` launch path corrected |
| `/health` lacked process identity | MISSING (fixed source) | added additive `pid`, `boot_id`, `data_dir`, `workspace_root`; CLI records/checks boot id |
| PID-reuse/process-image identity | MISSING | boot id now narrows this; creation-time image check remains open |
| OS-level data-dir lock | MISSING | state conflict only |
| DPAPI provider-key startup | MISSING | deliberately not implemented; key stays off |
| Live command verification | UNVERIFIED | requires a user-authorized test session |

## Pseudocode refactor record

### start dry-run
```
old: resolve -> health probe (may write state) -> port -> exe -> mkdir -> if dry run return
new: resolve base/exe/workspace/data/args -> if dry run return plan
     -> health/schema/db identity -> port -> mkdir -> spawn -> wait(pid)
```

### WS integration
```
old: new WebSocket(url)             # no Origin control
new: MiniWebSocket.connect(wsUrl)
     GET/Upgrade with Host, Origin, Sec-WebSocket-Key
     validate 101 + Accept
     frame parser: text/continuation/ping/close
```

### stop
```
old: require --yes always; taskkill (no /F); timeout -> error
new: own state -> term -> wait -> if alive force once -> report graceful/escalated
     external -> require --force --yes; otherwise E_NOT_OWNED
```

### output contract
```
success: {schema, ok, command, data, meta}
failure: {schema, ok:false, command, error{code,message,how_to_fix,retryable}, meta}
exit: 0 | 2 | 3 | 4
```

## Not done

- No live command/test run (user instruction).
- No `--format` render tests.
- No PID creation-time image check.
- No lock file; stale-state conflict only.
- No DPAPI key injection path.
- No `hiverouter-cli` merge into one Rust binary.
