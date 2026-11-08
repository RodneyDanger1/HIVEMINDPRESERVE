# Round 9 — merged findings register

Sources: `recon-backend.md`, `recon-ui.md`, `skill-unslop.md`, `skill-frontend.md`, `skill-modernize.md`.
This register is the merge; it does not replace the detailed reports. Status reflects the safe fixes applied this pass.

## P0 — authority/truth failures

| ID | Claim | Source | Status |
|---|---|---|---|
| R9-B01 | Memory is keyed by role string, not hive/branch/run; cross-run facts leak and survive rewire | `execution.rs:3029`; `store.rs:612-671`; `context.rs:100-159` | OPEN — design blocker |
| R9-B02 | `REWIRE_TASK` never updates `tasks.parent_task_id`; roster/A2A/fan-out/recovery keep stale parent | no such UPDATE; `ws.rs:650-736` | **FIXED** — row parent + row rev now updated after graph rewire; store + WS tests |
| F-UI-01 | Server-restart client adopts server `last_seq`; store never rehydrates while snapshot is ready | `client.ts:309,319`; `store.ts:1381-1391` | **FIXED** — reconnect boundary now forces a committed refresh even when snapshot was ready |
| FE-01 | Approval modal has no focus trap/initial focus/Escape/restore; destructive decision keyboard-unreachable | `ApprovalOverlay.tsx:221-250` | OPEN — needs DOM test tooling |
| R9-B09 | Keyless WS guard depends on a check suppressed when provider disabled | `ws.rs:743-753`; `checks.rs:143-146` | OPEN — add WS keyless test |

## P1 — remaining known areas

| ID | Claim | Source | Status |
|---|---|---|---|
| R9-B03 / MOD-05 | `SERVER_HELLO.last_seq` is server current seq, sent before replay; client cursor can advance ahead | `ws.rs:495-565`; `store.ts:1365` | OPEN |
| R9-B04 | Terminal `RUN_STATUS` precedes terminal snapshot; approval-timeout path may have no snapshot | `execution.rs:1293-1312`, `:2367-2396` | OPEN |
| R9-B05 / LOG-15 | History keeps oldest 1000 messages, drops newest; no direction in `truncated` | `store.rs:439-441`; `http.rs:782-786` | OPEN |
| R9-B06 | HyperAgent root never executes and never aggregates | `planner.rs:143-193`; `execution.rs:1331-1345` | OPEN |
| R9-B07 | `ask_parent` has no answer path | `execution.rs:2427-2497`; no reader | OPEN |
| R9-B08 | 40-provider-call cap is not enforced; `check_calls(1)` only | `execution.rs:1323,2781` | OPEN |
| FE-02 | Snapshot/resync error model is computed but never rendered; failed boot looks empty | `store.ts:1211-1248`; zero UI consumers | **PARTIAL** — HistoryPanel now renders `history.refreshRefusal` + Retry; bootstrap snapshot refusal still unrendered |
| FE-03 | Any snapshot refresh clears `replayGraph`, silently exits replay | `store.ts:1201-1210` | OPEN |
| FE-04 | Scrubber thumb can claim a refused revision | `Scrubber.tsx:21-25,57-59` | OPEN |
| FE-05 | Typed chat draft destroyed on tab switch (ChatCore unmount) | `ChatCore.tsx:94`; `App.tsx:50-102` | OPEN |
| FE-08 | `pendingRewire` has no origin/timeout/expiry; stale intent can set wrong parent | `store.ts:1981-1991` | OPEN |
| FE-10 | Canvas nodes memo omitted `interactiveGraph`; terminal cards stay draggable | `Canvas.tsx:206,221` | **FIXED** (dep added) |
| FE-11 / unslop B? | Tab bar hidden below `lg`, taking Swarm/Routing/History | `LibraryPanel.tsx:45` | OPEN — layout |
| FE-13 | N pending approvals render N modal scrims; covered dialogs focusable | `ApprovalOverlay.tsx:221-244` | OPEN |
| FE-16 | Foreign `REPLY_DELTA` could append chat / spin streaming | store run gate | **FIXED** (gate added + test) |
| F-UI-02 / MOD-03 | Hydration spread lets stale live tasks beat newer snapshot rev | `store.ts:1175-1194` | **FIXED** (rev-aware merge + test) |
| F-UI-09 / unslop B1/B2/B16 | Replay shows live counters in HUD delta, rail, canvas count/minimap | `HudBadge.tsx`, `LibraryPanel.tsx`, `Canvas.tsx` | **FIXED** (replay sources; HUD delta hidden) |
| F-UI-04 | N approval dialogs / focus (duplicate of FE-01) | | OPEN |
| F-UI-06 | Per-hive/branch memory missing; role/persona key mismatch | `execution.rs:3029`; `http.rs:985-1001` | OPEN — requirement |
| F-UI-07 | A2A roster now exists but has no web consumer; no answer/topic/mute | `http.rs:946-983`; no UI | PARTIAL — API only |
| F-UI-08 | `fetchNodeServerRev` can overwrite a newer rev, causing CAS loop | `store.ts:1323-1326,1976` | OPEN |
| MOD-01/02 | Node kind inferred from role; absent kind coerced to worker | `http.rs:743-751`; `store.ts:1504-1507` | PARTIAL — subagent intentionally mapped; needs contract decision |
| MOD-04 | Recovery timer not cancelled on success → duplicate reconnect/approval replay | `client.ts:377-413` | **FIXED** (timer cleared) |
| MOD-06 | Persona-id vs role-key capability/memory mismatch | `http.rs:985-1001`; `execution.rs:550-558` | OPEN |
| MOD-07 | `TEMPLATE_INSERT` bypasses spawn role policy | `ws.rs:240-291` | OPEN |

## P2 / design gaps

- Remote/Watchdog agents: no route, command, event kind, test, or UI.
- Ownership switching: display-only; no authority/command.
- Scheduler/scale: sequential dispatch; 40-call cap unenforced; no 50–150 active admission.
- Provider metadata: `/models` data parsed then discarded; hardcoded max (MOD-19).
- History revision authority split four ways; same-filename overwrite hazard (MOD-12).
- Corrupt snapshots silently listed (`history.rs:79-80`); `store.messages` rewrites corrupt rows (MOD-15).
- Minimap blank/honeycomb patterns; null progress looks like 0%; composer focus ring; amber contrast; `bee-bob` missing CSS; approval countdown `aria-live` at 1 Hz.
- CSS/DOM: no jsdom/testing-library in `web`, so approval focus work has no test harness today.
- `web/src/styles/index.css` is the real stylesheet; `web/src/index.css` does not exist (unslop correction).
- Port 8080 is held by an unrelated `posterpal` Vite server; do not read it as HiveMind.

## Dedupe map

| Finding family | Canonical ID |
|---|---|
| HUD delta in replay | unslop B1 / F-UI-09 |
| Rail live tasks in replay | unslop B2 / F-UI-09 |
| Canvas counts/minimap in replay | unslop B16 / F-UI-09 |
| Approval focus/modals | FE-01 / F-UI-04 / unslop B5 |
| Hydration stale merge | F-UI-02 / MOD-03 |
| Reconnect cascade | MOD-04 |
| Memory authority | R9-B01 / F-UI-06 / MOD-06 |
| Rewire row divergence | R9-B02 / F-UI-08 |

## Applied this pass

1. HUD replay delta suppressed.
2. Library rail uses replay tasks.
3. Canvas count/minimap use replay tasks.
4. Canvas memo depends on `interactiveGraph`.
5. Rev-aware hydration merge + unit test.
6. Foreign `REPLY_DELTA`/`REPLY_DONE` ignored + test.
7. Reconnect timer cleared on success.
8. Roster formatting repaired; unknown run returns 404 + test.
9. `REWIRE_TASK` now persists the new parent into the task row and pins row rev to graph rev + tests.
10. Reconnect boundary forces committed refresh even when the pre-restart snapshot was ready.
11. HistoryPanel renders `history.refreshRefusal` with a Retry action.

## Verification

- Web: `189 passed` (was 187), typecheck clean.
- Backend: `snapshot_history` 4/4; full `hiverouter-core` suite passed.
- App stopped; provider key absent; no provider call.
- Visual checks remain UNVERIFIED.
