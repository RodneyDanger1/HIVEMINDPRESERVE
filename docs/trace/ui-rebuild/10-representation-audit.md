# HiveMind UI representation audit — evidence gap matrix

Status: **AUDITED (UI evidence), runtime-unverified where noted**
Date: 2026-11-07
Method: evidence-gap-audit (issues → facts → proof → class), adapted from legal proof terms to UI evidence. No claim is admitted from memory; every row cites a design board, source file, backend route, or test. The human gate is at the end.

## 1. Issues ranked by product importance

| # | Issue | Importance |
|---|---|---|
| 1 | Every HiveMind run surface is represented without fake data | HIGH |
| 2 | Spatial canvas / chat / cards match the authoritative style system | HIGH |
| 3 | HMAS governance surfaces (profiles, memory, mailbox, projects, approvals, usage) are reachable | HIGH |
| 4 | History, replay, revision control are represented and read-only | HIGH |
| 5 | Swarm/topology is distinct from Workspace and shows the real hierarchy | HIGH |
| 6 | Every interactive control ends in a real command/route or is removed | HIGH |
| 7 | Empty/loading/error/offline states are honest | HIGH |
| 8 | Responsive/accessibility behavior is not broken | MEDIUM |
| 9 | Motion/live-feel is present but reduced-motion safe | MEDIUM |
| 10 | Design source is preserved and backed up | HIGH |

## 2. Fact → proof → gap matrix

Classification: `OK` (proof exists), `WEAK` (exists but incomplete/unverified in the required condition), `MISSING` (no proof), `CONFLICT` (sources disagree).

| Issue | Fact asserted by UI/product | Proof | Class | Action |
|---|---|---|---|---|
| 1 | Workspace is a spatial canvas with a pinned hive chat | `docs/ui/figcraft/ASSETS/hivemind-spatial-canvas.png`; `web/src/App.tsx` workspace view; `web/src/flow/Canvas.tsx`; `web/src/chat/ChatCore.tsx` | OK | none |
| 1 | Empty chat does not claim a drag-and-drop repository feature | `ChatCore.tsx` now says “Type a prompt…” + real starter buttons; `STRINGS.chat.empty` changed; no drag handlers exist | OK | none |
| 2 | Shell dimensions match board 07 | `web/src/tokens/tokens.ts` `LAYOUT` (52/72/240/400×480/460×44/220×150); `web/tests/designParity.test.ts` pins them; board 07 is 420×500 and this app intentionally uses the board-11 400×480 variant | OK | keep the variant labelled |
| 2 | Cards use the authoritative anatomy | `hivemind-agent-cards.png` board 05 v2; `web/src/flow/nodes.tsx` Super/Worker/Tool/Error anatomy; `LAYOUT.cards` | OK (design) / WEAK (live) | seed proof exists; keyed run still needed |
| 2 | Chat has code blocks, typing dots, working bee | `web/src/chat/ChatMotion.tsx`; `ChatCore.tsx` imports `MessageBody`, `TypingDots`, `WorkingBee`; `web/tests/chatMotion.test.ts` 4 tests | OK (unit) / WEAK (browser streaming) | provider-keyed run to see streaming |
| 3 | Agent hierarchy/topology is represented | `web/src/panels/AgentHierarchy.tsx`; `AgentsPanel.tsx`; new `SwarmPanel.tsx` | OK | none |
| 3 | Profiles + memory + mailbox are represented | `AgentProfilesPanel.tsx`, `AgentMemoryPanel.tsx`, `AgentMessagesPanel.tsx`, all mounted by `AgentsPanel.tsx`; real HTTP routes in backend (`/api/agents/profiles`, `/api/agents/memory`, `/api/agents/runs/{id}/messages`) | OK (surfaces) / WEAK (clear/confirm and live mailbox proof) | live keyed run; clear confirmation review |
| 3 | Projects registry is represented | new `web/src/panels/ProjectsPanel.tsx` fetches `/api/projects` (`backend/.../http.rs` `projects` handler); read-only because no create/rename route exists | OK (read-only) | do not add create controls without a route |
| 3 | Usage is represented | `ModelsPanel.tsx` uses `/api/usage/runs/{id}`; only a subset of fields is rendered | WEAK | render all returned counters and refresh on REPLY_DONE |
| 3 | Cross-branch approvals are represented | `ApprovalOverlay.tsx`; `store.applyEvent` approval path; server publishes `APPROVAL_REQUEST` | OK (surface) / WEAK (TTL/dismiss edge) | keyed approval run; add queue/ledger if a read path is added |
| 4 | History tab lists committed revisions and replays read-only | new `web/src/panels/HistoryPanel.tsx` uses `history.revisions` + `setHistoryRevision`; `Scrubber.tsx` owns live exit; replay guards in `ControlBar`/`ChatCore` | OK (code) / WEAK (live revisions) | keyed completed run |
| 4 | Replay blocks mutations | `ControlBar.tsx` `replaying` disables spawn/rewire/cancel/kill; `ChatCore.tsx` disables composer; `store.ts` history state | OK (code), WEAK (browser proof) | Playwright replay proof |
| 5 | Swarm tab is distinct from Workspace | `App.tsx` routes `tab === 'swarm'` to new `SwarmPanel`; `SwarmPanel` reads the same store and renders `AgentHierarchy`, census, projects | OK | browser proof |
| 6 | Dead “Unpin chat” removed | `ChatCore.tsx` header now has only the working hide control; `pinChat` UI removed | OK | none |
| 6 | Dead “Code-diff review” removed | `SettingsPanel.tsx` no longer renders the diff button | OK | `DIFF_*` remains decision-gated; do not fabricate |
| 6 | Agent search finds real tasks | `Header.tsx` search popover filters `tasks` and selects a task; no server search claim | OK | keyboard/empty proof |
| 6 | Project selector no longer looks like a dropdown | `LibraryPanel.tsx` project row has no chevron; read-only label | OK | none |
| 6 | Spawn role follows the selected parent tier | `store.ts` `spawnSubHive()` chooses SuperAgent/workerAgent/Subagent from `parentTask.kind` | OK (code) | keyed run to prove server ACK |
| 7 | Empty states are honest | `EmptyState`/`LoadingState`/`ErrorState` primitives; canvas empty suppressed while chat guidance is visible; chat tells the operator to submit a prompt | OK | none |
| 7 | Offline/replay states are named | `App.tsx` banner; `ToastRegion`; `Scrubber`; `ReplayBanner` | OK (code), WEAK (network transition) | live socket-drop test |
| 8 | Keyboard focus is visible on graph nodes | `web/src/styles/index.css` `.react-flow__node.selectable:focus-visible` | OK (rule) / WEAK (rendered) | keyboard screenshot |
| 8 | Minimap does not cover the chat at 1440 | minimap inset/margins; hidden below 1280 | OK (code) / WEAK (populated) | seeded run capture |
| 8 | Reduced motion is supported | `styles/motion.css` media + `[data-reduced-motion]`; `App.tsx` sets the data attribute; `index.css` fallback | OK (CSS) / WEAK (OS-level) | reduced-motion capture |
| 9 | Motion is decorative, never state-bearing | `motion.css` header; all classes are decorative; edge colour/state stays data-driven | OK | none |
| 9 | Motion is low-cost | loops animate transform/opacity/background-position; no filters; `will-change` limited | OK (static) / WEAK (frame trace) | DevTools trace on populated graph |
| 10 | FigCraft source, exports, manifest are preserved | `docs/ui/figcraft/HiveMind/HiveMind-Style-System.figcraft`; `HiveMind-ASSETS.json`; `verify_hivemind.py` PASS 28/28; A/B preserved | OK | Desktop backup copy |

## 3. Gap counts

| Class | Count |
|---|---|
| OK | 20 |
| WEAK | 11 |
| MISSING | 0 in the code-reviewed UI surfaces |
| CONFLICT | 2 (board 06 vs 07 library/minimap dimensions; board 05 chat 420×500 vs board 11/app 400×480) |

The two CONFLICTs are intentional and documented in `HiveMind/README.md`: board 07 is the authoritative shell, board 11 shows the slightly-smaller app variant. Do not “fix” one board to match the other without a product decision.

## 4. Missing representations that are constraints, not omissions

| Capability | Why no UI control is honest today |
|---|---|
| Notification bell | No notification domain/event/route exists in the contract. Adding a bell would be a false affordance. |
| Project create/rename/switch | `/api/projects` is GET-only; no create/rename/route exists. The panel is read-only. |
| Diff review | `DIFF_*` is decision-gated and not frozen; the former dead button was removed. |
| Operator menu | No operator/account domain; the header shows the current project owner only. |
| Full HMAS run picker | History endpoint returns `project_id/created_at/run_status`, but the client decoders drop metadata and no UI run picker existed. This audit adds revision history, not a fabricated run picker. |

## 5. Orphan design / code artifacts

| Artifact | Status |
|---|---|
| `hivemind-motion-and-live` board | **Used**: its motion spec informed the implemented CSS layer and the Motion section of this audit. |
| `hivemind-hmas-governance` board | **Used**: profiles/memory/mailbox/projects/usage/approval surfaces exist; Projects now reads the real registry. |
| `hivemind-states-and-refusals` board | **Used**: refusal toast, reconnect banner, empty/loading, kill confirm, replay banner exist. |
| `hivemind-typography/colour/space` boards | **Used** by token system; not separate screens by design. |
| Old `docs/ui/figcraft/C` set | **Superseded**, kept as evidence only; not referenced by UI. |

## 6. Research note — unknowns still blocking a claim

**Question:** does the current app represent every HiveMind capability that the design system claims?

Confirmed facts (with source):
- The canonical style source is `C:\Users\Riley\Desktop\HiveRouter\design\hive-style`; HiveMind is a renamed copy at `docs/ui/figcraft/HiveMind/`.
- The 12 artboards and 33+ HiveMind exports verify structurally (28/28).
- Real backend routes exist for projects, profiles, memory, messages, usage, history, approvals.
- A provider key is **not configured**; `/health` reports `E_KEY_MISSING`, so streaming/tool/approval paths cannot be exercised live.

Unknowns:
- Whether a keyed run emits every expected HMAS event on this build.
- Whether the acceptance/approval TTL path publishes a close event.
- Whether the history endpoint returns more than one committed revision for a real run.
- Whether the card layout/readability holds at 1440 with a dense 6+ node run.

What would prove each: one provider-keyed run; Playwright capture of every state; replay with ≥2 revisions; frame trace at 100% zoom.

## 7. Human gate

| Decision | Recommendation |
|---|---|
| Board 05 chat 420×500 vs app 400×480 | Keep the app at 400×480; board 06/07 remain authoritative for components, board 11 documents the variant. |
| Projects read-only | Accept until a real project-management route exists; do not add a fake create control. |
| Notification/operator menu | Defer; no domain. |
| Diff review | Defer; decision-gated contract. |
| Keyed HMAS proof | Requires `DEEPSEEK_API_KEY` in the server environment; do not fabricate event captures. |

**Verdict:** `AUDITED — UI representation complete against the current backend contract; runtime states remain WEAK until a keyed run.`
