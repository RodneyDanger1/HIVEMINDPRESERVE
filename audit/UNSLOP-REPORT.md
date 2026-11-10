# HiveMind UI — Unslop polish pass

## Five-line direction

- **Product:** A local multi-agent execution workspace where one HyperAgent governs SuperAgents, Agents, and temporary Subagents.
- **Character:** Warm, precise, architectural.
- **World:** An apiary rendered as an engineering control room: honey, ink, paper, hex cells.
- **Moments:** The live graph edges, the HyperAgent beacon, and provider status are the only richly colored moments.
- **Signature:** Honeycomb cell fields and bee-line connections that carry the eye between agents.

## Scope mode

**Polish mode.** The FigCraft identity is intentional and preserved. No new brand, no
information-architecture changes, no feature removal. The hard ceiling was a modest
10–20% craft pass: spacing/type discipline and a cleaner paid-product finish.

## Changes per pass

- **Spacing (U006):** all findings removed. 6px dots -> 8px (`h-2 w-2`), 18px
  controls -> 20px (`h-5 w-5`), 30px -> 32px (`pl-8`), 5px -> 4px
  (`h-1 w-1`), 150px -> 152px, and one 6px CSS offset -> 8px.
- **Type (U003):** header wordmark weight 700 -> 600 (`font-semibold`).
- **All other passes:** no changes. The existing FigCraft token system,
  neutral ramp, icon set, section order, and rich live moments remain.

Files touched:
`web/src/flow/nodes.tsx`, `web/src/chat/ChatMotion.tsx`,
`web/src/panels/HudBadge.tsx`, `web/src/panels/LibraryPanel.tsx`,
`web/src/panels/RunDetailPanel.tsx`, `web/src/panels/DaemonPanel.tsx`,
`web/src/styles/index.css`, `web/src/panels/Header.tsx`.

## Audit before / after

Command:

`node C:\Users\Riley\.dsh\skills\deepseek-unslop\assets\scripts\audit.mjs web/src --json`

| | Before | After |
|---|---|---|
| Exit code | 1 | 1 |
| Error findings | 71 | 47 |
| Warning findings | 3 | 3 |
| U006 spacing | 23 | **0** |
| U003 weight | 1 | **0** |
| U001 exclamation | 25 | 25 |
| U005 infinite animation | 20 | 20 |
| U010 uppercase label | 3 | 3 |
| U011 hype vocabulary | 2 | 2 |

Artifacts: `audit/audit-before.json`, `audit/audit-after.json`.

### Remaining findings are deliberate/accepted

- **U001 (25):** Tailwind `!important` utilities and mock/test command literals
  (`Certainly!`, `!approval`, `!refuse:`) — not production interface copy.
- **U005 (20):** the FigCraft motion language (bee bob, live pulse, ambient
  hex drift). It is intentional and guarded by the existing ambient-motion
  preference; removing it would flatten the product identity and exceed the
  requested 10–20% change budget.
- **U010 (3):** tracked uppercase mono labels are part of the FigCraft system,
  not an accidental template habit.
- **U011 (2):** `unlock` is the domain action for a lock device, not hype.

## Render evidence

- Baseline: `audit/baseline.light.png` — 1440×900, light, captured before edits
  with the Unslop snapshot script.
- Final audit render: `audit/final.light.png` — 1440×900, light, same script/flags.
- Connected product screenshot: `docs/evidence/live/ui-polished-final.png` —
  same 1440×900 view after the app connects, showing the `CONNECTED` state and
  the idle HyperAgent workspace.

`read_image` inspection: the composition is unchanged — honeycomb field left,
HyperAgent beacon centered, chat on the right, quiet neutral chrome. The
4px-grid corrections read as tighter control rows and dots without moving any
feature. Contrast, focus color, and the one-accent rule are preserved.

## Template-resilience answer

If the wordmark were swapped, the honeycomb field, bee-line edges, and
HyperAgent beacon still make this HiveMind-specific rather than a generic
dashboard; the polish does not change that.

## Verification

- Backend: 617 passed, 0 failed, 2 ignored (49 suites).
- Web: typecheck 0; 225 tests passed; production build 0.
- No feature or section order was removed.