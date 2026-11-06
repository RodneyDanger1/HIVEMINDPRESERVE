# tools/qa — static QA harness checks (no app, no installs needed)

These scripts run **now**, before the app exists, and never fabricate execution evidence.

| Script | What it proves | Run |
|---|---|---|
| `verify-harness.mjs` | 28-surface matrix integrity, frozen command/event/refusal vocabulary cross-check against `docs/contracts/UI-COMMANDS-v1.md`, 11-hop names, pre-evidence statuses only, trace seeds valid and PENDING | `node tools/qa/verify-harness.mjs` |
| `build-trace-seeds.mjs` | Regenerates `docs/trace/controls.json` / `traces.json` / `events.json` from the matrix (all PENDING; empty runs) | `node tools/qa/build-trace-seeds.mjs` |
| `figcraft-assets.mjs` | Exact FigCraft-A export inventory (file/size/sha256) + manifest/A-design presence (read-only) | `node tools/qa/figcraft-assets.mjs` |
| `check-env.ps1` | Host/toolchain/pin/browser/disk/project-state evidence (read-only) | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/check-env.ps1` |

Exit codes: `verify-harness.mjs` → 0 pass, 1 check failure, 2 parse/file error.
No script writes outside `docs/trace/` (build-trace-seeds only) and none touches `web/`, `backend/`, FigCraft assets, or the blueprint.
