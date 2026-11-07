# 04 — Web UI

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
