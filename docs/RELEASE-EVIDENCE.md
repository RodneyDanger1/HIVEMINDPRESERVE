# HiveMind 1.0.0 — release evidence

Date: 2026-11-06 (America/Chicago). Product commit: release commit (recorded at tag time). No key material appears in this file.

## Artifacts

| Artifact | SHA-256 | Bytes |
|---|---|---:|
| `hiverouter.exe` | `485dac243709a5e5bcf387b04d09b3c2e920b472a15528938c9aa083f6d4538c` | 13,253,120 |
| `hiverouter-cli.exe` | `49f46e2be488dd675280f3dae7e8d455ac5eeec2aa46d7fd44f1c2557c386623` | 5,373,952 |
| `dist/HiveMind-1.0.0-win64.zip` | `c675568f1205f4093348db9cf5d42cce7244d7e3d4965cd4c1e8609a555693d6` | 8,444,039 |

The zip contains both binaries, `README.md`, `RUNBOOK.md`, `RELEASE.md`, and `CHECKSUMS.txt`; an extraction smoke listed all six files and verified the archive opened.

## Release-binary live smoke (not the debug binary)

Started `target/release/hiverouter.exe` on `127.0.0.1:8800` with a fresh data directory and the key only in the process environment.

- `GET /health` → `ok`, version `1.0.0`.
- `GET /` → 200, embedded UI HTML present (`id="root"`), 774 bytes.
- `SETTINGS_STATUS` → `{"provider":"deepseek","model":"deepseek-flash","key_present":true,"thinking":"enabled","reasoning_effort":"high"}`.
- `SUBMIT_PROMPT` `Reply with exactly HIVE-1.0-FINAL` → `RUN_CREATED` → nodes/reply events → `REPLY_DONE` usage `{prompt:91, completion:8, total:99}` → `RUN_STATUS completed`; reply `HIVE-1.0-FINAL`.
- `HISTORY_REVISION_SET` revision 0 → `HISTORY_SNAPSHOT` with **2 graph nodes** (real snapshot; not fabricated).
- The key never appeared in the server log, settings payload, or evidence.
- Child-process isolation: shell/tool spawns remove `DEEPSEEK_API_KEY` before execution (`shell.rs`, `cli_support.rs`), pinned by the new `child_process_does_not_inherit_the_provider_key` test.

## Test evidence at integration baseline

- Rust workspace: all suites green after provider wiring, including the child-environment leak test; the only ignored test is `provider_live` (requires the key and is run explicitly).
- Web: 46/46 unit tests, typecheck 0 errors, Vite production build green.
- Live provider adapter: planner node + `HIVE-PROVIDER-OK`, usage 99 tokens (`docs/evidence/live-provider-smoke.md` has the server-level run).
- Key scan over the non-vendor tree: 0 occurrences of a `sk-...` credential; tracked-tree child spawns remove the key.

## Known limits (also in `docs/RELEASE.md`)

- Leaf reply is emitted as one delta after a non-streaming completion; true SSE chunk rendering is 1.1.
- Autonomous model tool-calling, full trace ladder, pixel fidelity and Tauri are 1.1.
- Normal tests never call the provider; live tests are explicit and budget-capped.
- The supplied key was exposed in chat; rotate after release if the transcript is shared.

## Final tag gate

Tag only after the independent audit and quiescent-tree check. Moving the previous private `v1.0.0` tag requires owner confirmation and records the superseded commit.
