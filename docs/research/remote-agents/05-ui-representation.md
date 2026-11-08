# 05 — UI representation: tree-free remotes, devices, daemons, supervisors

**Status:** DESIGN (D); current UI facts from `24-ui-implementation.md`, `15-ui-ux-goal.md`, FigCraft boards.
Decisions D-R6, R-5, X-R9.

## 1. Placement: a separate lane, not a canvas node

The canvas/tree is the **task graph**. A remote is not a task; drawing it on the canvas would lie about depth,
edges, spawn caps and run success. The UI therefore adds one **Remote** surface beside the existing agents view:

| Surface | Contents | Where |
|---|---|---|
| **Remote** destination (new nav item or Agents sub-tab, decision U-R6) | Remote cards, device cards, grants, supervisor/watchdog panel, audit | New `web/src/panels/RemotePanel.tsx` + route; no canvas change |
| **Agents** destination (existing) | Task roster (`/api/agents/runs/{run_id}/roster`), profiles, task memory/mailbox | unchanged; remote entries are **not** mixed into task roster |
| **Run detail** | Tool/device actions requested by a run appear as *evidence* rows with a link to the Remote lane; no device control on the run screen | run detail |
| **Swarm/canvas** | no remote/device nodes; a single summary chip ("4 remotes · 2 degraded") may link to Remote | canvas chip only |
| **Settings → Remote** | enrolment, adapter/endpoint ref, grants editor, TLS warnings, revocation | settings |

Rationale: `21` §3.8 makes the roster the agent-card invariant (every agent gets a card), not the canvas; the same
rule applies to remotes/devices, but in their own roster. The current Agents panel already hosts task cards,
profiles, memory and mailbox; adding remote cards there would mix three different lifecycles and confuse the
existing `agentState` vocabulary (16 states). Do not overload it.

## 2. Cards

### 2.1 Remote card (tree-free principal)

| Field | Source | Honest rules |
|---|---|---|
| Name + owner chip (`operator`, `hive:…`, `office:…`) | `remote_agents.display_name`, owner fields | owner is a link/filter, not a tree path |
| Transport/adaptor chip | `transport_kind` + `adapter_kind` | never show a credential/endpoint secret |
| State chip | `RemoteState` (7 values) | distinct colour/glyph from task states; `unknown` ≠ `offline` |
| `last_seen` / lease age | `last_seen_at`, `lease_expires_at` | relative + absolute (`last seen 42 s ago`); stale if lease expired |
| Capability summary | grant count / declared capability count | "3 grants · 4 devices"; empty means "not granted", never "ready" |
| Device count + worst state | derived from devices | "2 lights · 1 camera; 1 unavailable" |
| Watchdog/health | `REMOTE_REPORT`/supervisor | show last report severity, not a fake heartbeat |
| Actions | Enable/disable, revoke, edit grants, open audit | mutations routed through WS commands (`04` §5); disabled if `revoked/retired` |

Card states: `enrolled` (grey, "not connected"), `online` (verify green + last seen), `degraded` (amber + failing
check), `offline` (grey, "offline since T"), `quarantined` (red, "actions refused"), `revoked` (red, terminal),
`retired` (hidden by default). No green border on `unknown`.

### 2.2 Device card

| Field | Source | Rule |
|---|---|---|
| Class glyph + name + room | registry | class determines shape/icon, not colour alone |
| State | `state_json` normalized by class | show `unknown` when no fresh read; `on/off`, `open/closed/opening`, `locked` |
| Freshness | `observed_at` + class window | "confirmed 3 s ago" / "stale (2 min)" / "unknown"; never show a stale value as current |
| Connection | supervisor/device availability | `available|unavailable|stale|unknown` separate from the state itself |
| Controls | capability grants + policy | only when enabled; disabled control has a reason + refresh |
| Last action | latest `device_actions` | result + code + time; `unknown` explicit |
| Camera | snapshot button | image with capture time; "no live stream"; approval/consent required |
| Garage/lock | open/close controls | **never** a bare toggle; hold-to-confirm/typed phrase + pre-state display |

### 2.3 Daemon/watchdog card (record, not task)

Adopt `21` §3.8.3/§4.4: name, job kind/target, owner, enabled switch (CAS `switch_rev`), cadence, health
(`unknown|healthy|degraded|faulted`), last heartbeat/check/report, consecutive failures, restart attempts in window,
reports list (severity/transition/dedupe/suppressed), `report_now`, `retire` (kill). **No** task controls (spawn/kill
worker), no provider/model budget, no memory bank. A watchdog that watches a remote/device links to that card.

### 2.4 Supervisor panel (system, not an agent)

A compact "Remote supervisor" section in the Remote lane: lane state (idle/running/backoff), `next_due`, probe
backlog, quarantined remotes, last reconciliation time, and a **Restart lane** operator action (refused if busy).
It is not selectable, not a card in the roster, and has no "talk to supervisor" control.

## 3. Control UX

1. **Light switch:** a real control only when `online + grant + fresh state`. Click → if approval policy requires,
   `DEVICE_APPROVAL_REQUEST` card (device/room, current state, action, consequence "turns the study light on", TTL,
   deny-first, Escape no-decision). Approve → button shows pending; on `DEVICE_ACTION(confirmed)` move the switch and
   show "confirmed at T". On `unknown/timeout`, **do not** snap the switch to a guess; show "unknown outcome" with
   Refresh and audit link. Auto mode (opt-in, R1) shows a distinct "auto" badge, still audited.
2. **Camera:** snapshot only; modal/drawer with capture time, remote/device, purpose and consent note; no
   continuous stream; close discards unless operator saves (explicit artifact action). A privacy setting at remote
   level can disable snapshots entirely.
3. **Garage/lock:** two-step control: first a request card showing current state and risk; then an explicit
   hold-to-confirm or typed phrase (`OPEN GARAGE`) + approve. If state is stale/unknown, the control is disabled and
   "Refresh state" is the only action. While an action is in flight, controls lock (single-flight). After
   `confirmed`, show pre→post and timestamp; after timeout/unknown, show "check the door; do not retry blindly".
4. **Grants editor:** matrix of devices × capabilities × actions with risk badges and constraints; default deny;
   saving requires an explicit review + audit note; revoke is one click + confirmation; no bulk "grant all".
5. **Audit drawer:** filter by remote/device/outcome; fields: time, actor (operator/system/remote), action,
   request id, idempotency key, outcome/code; no secrets. Copy request id for support.
6. **Revocation:** confirm; shows consequences (leases/actions/grants invalidated) and that a physical action may
   already have happened; revokes immediately, retains audit/card as `revoked`.

## 4. Honest-state rules (hard requirements)

| Rule | Test |
|---|---|
| A last-known state is never rendered as current; it carries `observed_at` and a staleness marker | `device_card_never_fakes_fresh_state` |
| Controls are disabled when `unknown/stale/offline/revoked/quarantined`; disabled reason is visible/accessible | `controls_disabled_with_reason` |
| No optimistic state change for any action; UI updates only on a server event/result | `no_optimistic_device_state` |
| `unknown` outcome is a first-class UI state, not mapped to `succeeded` | `unknown_is_not_success` |
| Remote/device/daemons never appear as task cards, never get agent dot semantics, never enter canvas/tree | `remote_not_in_task_roster` |
| A capability-less remote shows "no capabilities granted", not "ready" | `empty_capabilities_not_ready` |
| A credential value never appears in DOM/aria/title/copy buffer | `credentials_not_in_dom` |

## 5. Visual tokens and FigCraft extension

- Reuse the existing token system (`tokens.ts`, `tokens.css`, `index.css`) and card anatomy from board 05
  (`docs/ui/figcraft/ASSETS/hivemind-agent-cards.png`; card registry in `24`): honey header for remote, paper body,
  hairline separators, mono meta ≥ 9.5 px, state = dot + glyph + text (never colour alone).
- Device cards use the board-09 governance/paper style (`hivemind-hmas-governance.png`); risk class is text + icon
  (LOW / CRITICAL), not only crimson.
- A **new FigCraft board (proposed 13 · Remote & devices)** is justified: remote card, device card (light switch on,
  camera snapshot, garage locked/unknown), approval card, supervisor strip, audit drawer. Extend the existing style
  system; no new palette. Do not claim parity before a measured pass; board work is a Phase R2 deliverable.
- Accessibility: card buttons are actions with accessible names including device/name/action; confirmation dialogs
  are `alertdialog` with focus trap, Escape no-decision, focus return; status regions `aria-live=polite`; no
  colour-only risk/state; keyboard-only garage phrase input allowed; reduced-motion respected.

## 6. "Not available on this build" rule

Until v12+ lands, the Remote destination must render an honest empty/disabled state sourced from a capability
probe (route 404/status capability): "Remote control plane is not implemented in this build; see task-12 design."
No fake cards, no disabled-but-present switches, no placeholder devices. The existing `ProjectsPanel` read-only
pattern is the precedent (do not present dead controls).

## 7. UI acceptance tests (vitest/DOM; no device)

1. `remoteCard_states_render` — all 7 remote states render correct label/glyph/disabled actions.
2. `deviceCard_freshness` — fresh/stale/unknown render distinct text; stale value labelled with timestamp.
3. `lightSwitch_requires_grant_and_fresh` — disabled with reason when no grant/stale; enabled only when both.
4. `deviceApprovalCard_denyFirst_escapeNoDecision` — focus order, Escape, TTL, no optimistic close.
5. `garageNeedsConfirmationPhrase` — approve without phrase ⇒ `E_DEVICE_CONFIRM_REQUIRED` displayed; controls locked
   while in flight.
6. `unknownOutcome_notSuccess` — action result `unknown` renders "unknown outcome", not a moved switch.
7. `remoteNotInTaskRoster` — task roster/canvas has no remote/device; Remote lane has them.
8. `credentials_not_in_dom` — fixture endpoint/credential refs render no secret-looking strings.
9. `supervisorPanel_noAgentActions` — lane state, restart; no spawn/kill/chat.
10. `daemonCard_switchCAS` — stale `switch_rev` refusal visible; report list paginated; no model/budget controls.
11. `emptyCapabilities_notReady` — remote with no grants shows "not granted", not green.
12. `remoteUnavailableHonest` — no v12 route ⇒ "not implemented on this build"; no placeholder devices.
13. `a11y_alertdialog_focus_trap` — focus trap/return/Escape, accessible names.
14. `reduced_motion_no_actuation_animation` — no animation implies device state changed.
15. `hostile_device_text_escaped` — device name containing HTML/script renders as text.

## 8. Open UI decisions

- **U-R6** Remote as a seventh nav destination vs an Agents sub-tab; recommendation: sub-tab first (less nav churn),
  promote to destination when devices > 1 remote.
- **U-R7** Camera snapshot storage: memory-only vs explicit saved artifact; default memory-only.
- **U-R8** Garage confirmation phrase vs hold-to-confirm; recommendation both available, phrase default for
  accessibility/keyboard.
- **U-R9** Auto light-switch policy exposure: per-device allowlist in Settings with a visible "auto" badge; default
  off until approved by the operator.
