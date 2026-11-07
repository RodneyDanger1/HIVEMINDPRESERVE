# HiveMind control-wiring matrix (weaver)

Every interactive control is classified `WIRED-AND-WORKS`, `WIRED-BUT-DEAD`, `MISSING-UI`, or `LYING`.
A control is only `WIRED-AND-WORKS` when its click path reaches a store action and a real command/route.
Runtime evidence is limited to empty/unkeyed states; rows that need a provider run are `WEAK` below.

| Control | Screen | Hop 1 (component) | Hop 2 (store) | Hop 3 (transport/route) | Class | Evidence / note |
|---|---|---|---|---|---|---|
| Search agents | Header | `Header.tsx` popover filters `tasks` | `selectTask` | local selection only | WIRED-AND-WORKS | no network claim |
| Workspace tab | Library | `LibraryPanel` tablist | `setTab('workspace')` | view state | WIRED-AND-WORKS | distinct canvas/chat view |
| Swarm tab | Library | `LibraryPanel` | `setTab('swarm')` | view state | WIRED-AND-WORKS (new) | new `SwarmPanel.tsx`; real hierarchy/census |
| Routing tab | Library | `LibraryPanel` | `setTab('routing')` | view state | WIRED-AND-WORKS | `RoutingPanel.tsx` |
| History tab | Library | `LibraryPanel` | `setTab('history')` | view state | WIRED-AND-WORKS (new) | `HistoryPanel.tsx` lists committed revisions; Canvas renders replay |
| Rail nav | NavRail | `setNav` | nav state | view state | WIRED-AND-WORKS | roving focus |
| Template insert | Library | `insertTemplate` | `insertTemplate` | `TEMPLATE_INSERT` WS | WIRED-AND-WORKS (code) / WEAK live | server ACK not exercised in this audit |
| Agent select | Library / canvas | `selectTask` | selection | local | WIRED-AND-WORKS | |
| Hide finished | Canvas | local filter | visible task projection | local | WIRED-AND-WORKS | derives from real states |
| Focus branch | Canvas | local filter | selected branch projection | local | WIRED-AND-WORKS | fixed; no global union |
| Graph only | Canvas | `setChatVisible(false)` | chat view + fit viewport | local | WIRED-AND-WORKS (new) | lets cards be viewed without the fixed chat overlay |
| Zoom in/out | Canvas | React Flow API | viewport | local | WIRED-AND-WORKS | |
| Recenter | Canvas | `setCenter(0,0)` | viewport | local | WIRED-AND-WORKS | Ctrl+0 too, text-field guarded |
| Fit swarm | Canvas | `fitView` | viewport | local | WIRED-AND-WORKS | |
| Chat submit | Chat | `validatePromptText` → `submitPrompt` | optimistic row + rollback | `SUBMIT_PROMPT` WS | WIRED-AND-WORKS (code) / WEAK provider | refusal `E_KEY_MISSING` is real and rendered |
| Prompt starters | Chat | `setDraft` + focus | none | local | WIRED-AND-WORKS (new) | never auto-submits |
| Hide chat / show chat | Chat/Canvas | `setChatVisible` | chat view | local | WIRED-AND-WORKS | both entry points exist |
| Unpin chat | — | removed | — | — | removed | no dead control remains |
| Spawn sub-hive | ControlBar | `spawnSubHive` | parent/role derivation | `SPAWN_SUB_HIVE` WS | WIRED-AND-WORKS (code) / WEAK live | role now follows parent tier |
| Re-route target select | ControlBar | local target | parent options from tasks | — | WIRED-AND-WORKS (no-op until button) | labelled |
| Re-route | ControlBar | `rewireSelected` | `rewireBlocker`/rev | `REWIRE_TASK` WS | WIRED-AND-WORKS (code) / WEAK live | server CAS |
| Cancel run | ControlBar | `cancelRun` | pending guard | `CANCEL_RUN` WS | WIRED-AND-WORKS (code) / WEAK live | |
| Kill worker | ControlBar | `killWorker` | selected worker guard | `KILL_WORKER` WS | WIRED-AND-WORKS (code) / WEAK live | |
| Revision select | History/Scrubber | `setHistoryRevision` | history state | `GET /api/history/runs/{id}/state?revision=` | WIRED-AND-WORKS (code) / WEAK live | read-only replay |
| Return live | History/Scrubber | `exitHistoryReplay` | replayGraph=null | local | WIRED-AND-WORKS | explicit exit |
| Approval approve/deny | ApprovalOverlay | `resolveApproval` | nonce/decision | `APPROVAL_RESOLVE` WS | WIRED-AND-WORKS (code) / WEAK live | TTL/dismiss edge documented |
| Settings probe | Settings | `probeSettings` | pending | `SETTINGS_STATUS` WS | WIRED-AND-WORKS / WEAK result | loading is honest |
| Health probe | Settings | `probeHealth` | health state | `GET /health` | WIRED-AND-WORKS | real DB/transport fields |
| Data tree | Data | `DataPanel` file list | read-only | `GET /api/files` | WIRED-AND-WORKS | |
| Model usage | Models | `ModelsPanel` fetch | read-only | `GET /api/usage/runs/{id}` | WIRED-AND-WORKS (code) / WEAK refresh | only subset rendered |
| Profiles/memory/mailbox | Agents | panels fetch/read | read-only | `/api/agents/*` | WIRED-AND-WORKS | live mailbox versioned event |
| Projects | Swarm | `ProjectsPanel` | read-only | `GET /api/projects` | WIRED-AND-WORKS (new) / WEAK live | no create/rename route |
| Toast dismiss | Toast | `dismiss` | refusal list | local | WIRED-AND-WORKS | |
| Notification bell | — | absent | — | — | MISSING-UI (constraint) | no notification domain; not a dead control |
| Project selector | Library | static row | — | — | WIRED-AND-WORKS (read-only) | chevron removed |
| Code-diff review | — | removed | — | — | removed | decision-gated contract |
| Reduced-motion override | Settings | `setReducedMotionOverride` | document attribute | local | WIRED-AND-WORKS | CSS media + attribute |

## Dead-control deltas

| Control | Before | Now |
|---|---|---|
| Unpin chat | label-only toggle | removed; only working hide/show remains |
| Code-diff review | opened nothing | removed |
| Project chevron | inert dropdown affordance | removed; row is visibly read-only |
| Swarm tab | identical to Workspace | dedicated `SwarmPanel` |
| History tab | canvas only | revision list + read-only replay + live exit |
| Focus branch | no-op with hide-finished | real branch filter |
| Error card fallback | invented “Compile failed” | named error code or “Error state” |
| HUD dot | always green | colour derives from metric source |
| Chat drop claim | “Drop a repository” with no handler | prompt copy + real starters |

## Unproven controls (WEAK)

These require a provider-keyed run or a live socket transition; they are not claimed green:

- Spawn / re-route / cancel / kill against a running server.
- Chat streaming, tool activity, approval approve/deny, reconnect/resync.
- Revision list with more than zero revisions and replay of a populated graph.
- Populated minimap, node drag/snap/rewire, populated HUD counters.

Evidence needed: one `DEEPSEEK_API_KEY` run, Playwright request/response capture, store transition capture, screenshot per state.
