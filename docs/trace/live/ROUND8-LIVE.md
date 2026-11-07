# Round 8 live proof — localhost server + embedded UI

Date: 2026-11-07 · HEAD: `f15ad6f` · binary: `target/release/hiverouter.exe` (1.2.0-dev)
Server: `http://127.0.0.1:8099`, data `C:\Users\Riley\Desktop\hivemind-live-data`, schema v10.

## Boot

- stdout: `hiverouter 1.2.0-dev listening on http://127.0.0.1:8099 (db ... schema v10, contract hiverouter.v1)`
- `GET /health` -> `status=degraded`, because `deepseek_api_key=false`; schema `10`.
- `GET /` -> 200, title `HiveMind — Spatial Execution Environment`.
- WebSocket reached `CONNECTED` in headless Chrome.

## Prompt / terminal truth

- Submitted `live banner proof` through the real composer in headless Chrome.
- The run was created and failed with `E_KEY_MISSING` (no provider key; no paid call).
- Before the fix the code existed only in state; after `ChatCore.tsx` the UI rendered:
  `RUN REFUSED: E_KEY_MISSING`.
- API run row confirmed `status=failed, error_code=E_KEY_MISSING`.

## API probes

`GET /api/projects`, `/api/agents/profiles`, `/api/agents/memory`, `/api/history/runs` all 200.
Live profile import (`live-writer`) persisted and then deleted successfully.

## Launcher

`HiveMind.cmd` -> `tools/start-hivemind.ps1`; detected the running server and reopened the UI.
When no server is healthy it starts the release exe, waits for health, then opens the browser.

## Unverified

- Paid model run (no key present).
- A click-through after the last rebuild of every profile/memory surface.
- Browser visual layout/accessibility audit (headless DOM only).
