# 02 - Card parity: HiveMind reference vs current

**Status:** analysis only. No product source was edited. This file is the only write.
**Scope:** `web/src/flow/{nodes.tsx,graph.ts,Canvas.tsx,BeeEdge.tsx}`,
`web/src/chat/ChatCore.tsx`, `web/src/state/agentState.ts`, plus the HUD/canvas
chrome that those components sit inside.
**Snapshot:** 2026-11-07 15:10-06:00 (America/Chicago). The working tree was being
edited while this analysis ran; the hashes below are the state analysed. If a hash
differs, re-read the file before applying a fix.

| File | Modified | SHA-256 (first 12) |
|---|---|---|
| `web/src/flow/nodes.tsx` | 14:47:04 | `B69DB7E9AF52` |
| `web/src/flow/graph.ts` | 15:01:19 | `2842E1C084B6` |
| `web/src/flow/Canvas.tsx` | 15:04:45 | `1B98B49BC30B` |
| `web/src/flow/BeeEdge.tsx` | 15:00:53 | `EFAFC8F82E64` |
| `web/src/chat/ChatCore.tsx` | 14:51:52 | `04A228E5E795` |
| `web/src/state/agentState.ts` | 11-06 11:07 | `4CFC0735A6E0` |

## 0. Reference precedence

1. `C:\Users\Riley\Desktop\HiveRouter\design\hive-style\build_hive_style.py`
   lines 815-902 are the executable source of truth for the five card types and the
   chat shell; lines 522-535 define the hive; lines 318-342 define the bee route.
   Values below are read from that file, not remembered.
2. The exported boards are the visual truth:
   `docs/ui/figcraft/ASSETS/hivemind-agent-cards.png` (1440x1300),
   `hivemind-spatial-canvas.png` (1440x900),
   `hivemind-controls-and-hud.png` (1440x1040).
   `HiveRouter\design\hive-style\exports\hiverouter-agent-cards.png` is byte-identical
   to `hivemind-agent-cards.png` (both SHA-256 `1bbc81a4...`), so the HiveMind and
   HiveRouter card boards are the same artwork.
3. `docs/ui/figcraft/HiveMind/README.md:30-32` pins the board sizes; lines 46-48 pin
   the board-07 composition (52px topbar, 72px rail, 240px library, canvas x=312,
   chat 420x500 at 620,196, HUD 460x44, minimap 220x150, SuperAgent 420x150,
   WorkerAgent 320x96, ToolAgent 420x240, crimson error card).
4. `HiveMind-ASSETS.json` surfaces S15-S19 declare the same card sizes. The
   `source-evidence/HiveRouter-Style-Kit.md` is secondary; where it disagrees with
   the builder, the builder wins (for example the kit says the SuperAgent header has
   "5px state dot", while `superagent_card()` actually draws a `RUNNING` pill and no
   dot).

Key reference numbers used throughout:

| Element | Reference value | Source |
|---|---|---|
| SuperAgent card | 420x150, r14, 1.5px `#E5C27A`, e1 | builder 846-861 |
| Worker card | 320x96, r12, 1px `#E5E1D8`, e1 | 864-873 |
| Tool card | 420x240, r14, 1px hairline, e1 | 876-890 |
| Error card | 320x110, r12, 1.5px `#B23A2E`, fill `#FFF7F5` | 893-902 |
| Chat shell | 420x500, r18, e2, fill `#F8F4EB` | 815-843 |
| Hive | 660x700, 5 radial-gold lobes, `#7A4E14` 1.6 outline | 522-535, 1219 |
| HUD | 460x44, r999, `#E5C27A` 1.2, e2 | 1228-1233 |
| Minimap | 220x150, r12, e2 | 1235-1243 |
| Type | Inter UI, Consolas mono; names 13/13.5/14.5; role tags 9-9.5 mono | 34-62, 846-900 |

## 1. Executive findings

1. **The card dimensions are now correct, but the card anatomy is not.** `graph.ts`
   now uses `LAYOUT` for all five sizes, and `designParity.test.ts` passes (verified:
   `npm.cmd test -- tests/designParity.test.ts`, 3 passed). The remaining gaps are
   border colour/width, header inset, state rendering, role-tag type, progress track
   and fill, and meta rows.
2. **There are two HyperAgents.** The fixed `ChatCore` overlay is one; the live graph
   also renders the server's depth-0 `hyperagent` task as a 420x500 `HyperAgentNode`
   behind it. The backend deliberately publishes that root
   (`execution.rs:1322-1335`; tier mapping `execution.rs:151-162`), so this is not a
   rare edge case. Hide the chat and a giant empty hex card with the label
   `Task <id>` is left on the canvas; it also becomes a 420x500 minimap blob and a
   focusable node.
3. **State colour is broken by a one-line vocabulary mismatch.** `visualFor()`
   returns `toneToken: state` for the four dot states, but `tokenColor()` only accepts
   `honey-*`, `ink-*`, `verify`, `signal`, `amber`, `hairline`. Healthy, running,
   failed and queued therefore all render `--color-ink-500` (grey-brown), not
   honey-400 / honey-600 / signal / hairline.
4. **The chat component is a chat log, not the reference chat core.** It has the
   correct 420x500 shell and header gradient, but no bee, no `x` close glyph, no user
   or assistant bubbles, no dark code block, and a different composer. Reference
   inner padding is 16; current shell padding is 12.
5. **Four user-facing claims are false in the current UI:** "Drop a repository" (no
   drop handler exists), "Unpin chat" (the shell never unpins or moves), the HUD's
   always-green dot when there are no metrics, and the error card's "Compile failed"
   fallback when no error code was reported. These are P0 honesty fixes, not polish.
6. **The edge is still a Bezier, not a bee route.** Reference: straight dashed
   `#E8B850` 1.5px, dash `[10,10]`, arrowhead, bee at 55%. Current: Bezier,
   `honey-600`, dash `[6,4]`, no arrowhead. The new broken-edge branch is correct in
   intent (`data.broken`, crimson `[6,6]`) but has no halted bee/warning glyph and
   misses the `error` kind.

## 2. Priority order

### P0 - structural or truthfulness blockers

| ID | Item | Why P0 |
|---|---|---|
| P0-1 | One HyperAgent entity: remove the graph-rendered root card, keep a layout anchor, hide it from minimap/focus; 5-lobe hive is the fixed core | Duplicate entity, ghost card when chat hidden, wrong minimap, misleading `Task <id>` label |
| P0-2 | Fix state dot colours and split "dot" vs "pill" renderers | Every healthy/running/failed/queued dot is grey today; SuperAgent pill contains a dot **and** text, violating "state: dot or pill, never both" |
| P0-3 | Remove/replace false chat empty-state copy and the fake pin control | "Drop a repository" has no handler; "Unpin chat" changes only a label |
| P0-4 | Make the HUD dot truthful | Green dot renders when `source='none'` and `greenPct=null` |
| P0-5 | Error card: stop inventing "Compile failed"; add the crimson paused-bee route | The UI asserts a diagnosis the server did not send |
| P0-6 | Canvas empty state when filters hide every node | Says "No agents yet" while `tasks` is non-empty; filter state is not empty state |

### P1 - reference anatomy

| ID | Item | Reference seam |
|---|---|---|
| P1-1 | SuperAgent: gold 1.5px border, 40px honey header, 30px title inset, 68x18 RUNNING pill, 9.5 role tag, 5px bar, `n workers - ctx` meta | `nodes.tsx:86-118` |
| P1-2 | Worker: dot only, 9px dot at x+16/y+22, name at x+30, 9px role tag at x+30/y+32, percent above the 3px bar, no ctx placeholder | `nodes.tsx:120-139` |
| P1-3 | Tool: 80x18 TOOL pill, 132px dark code box with three traffic dots, 10.5 mono output, 6px gold bar, honest output meta | `nodes.tsx:141-164` |
| P1-4 | Chat: bee + title + `x` header, canvas body with 10px bubbles, code box, 56px composer, 16px inner padding | `ChatCore.tsx:166-272` |
| P1-5 | Bee route: straight dashed honey-400 `[10,10]`, arrowhead, 18px bee at 55%, crimson `[6,6]` broken variant with warning | `BeeEdge.tsx:13-58` |
| P1-6 | Hive core: five stacked radial-gold lobes, `#7A4E14` outline, 660x700; honeycomb R=44 `#EFE7D2` | `Canvas.tsx:235-268` |
| P1-7 | Layout direction: SuperAgents and workers are underneath, not to the right (`rankdir: 'TB'`; top/bottom handles) | `graph.ts:40` |

### P2 - polish, tokens, and parity debt

| ID | Item |
|---|---|
| P2-1 | HUD exact 460px width, 10px semibold +6% type, 5px dot, delta outside the pill, derived badge outside |
| P2-2 | Minimap honeycomb field, e2 shadow, hidden React Flow attribution |
| P2-3 | Shadow tokens exact two-layer warm-ink values; add `border-gold`, `honeycomb-line`, `chat-user`, `error-surface`, code-dot tokens |
| P2-4 | Add `.type-role-tag` / `.type-card-meta` classes instead of reusing 11px/8% `type-mono-meta` |
| P2-5 | Progress width transition 200ms; reduced-motion respected |
| P2-6 | 150px snap radius, target ring/preview, collision displacement (spec §2) |
| P2-7 | Replay: all snapshot nodes render as `worker` with `task <id>`; label the limitation and/or carry `kind` if the snapshot ever adds it |
| P2-8 | Hardcoded "Hide finished"/"Focus branch"/"Select a node..." copy in `Canvas.tsx`; move to `STRINGS` |
| P2-9 | Visible React Flow handles (gold dots); reference has no connection handles |
| P2-10 | Bee glyph fidelity (three bands, two wings) in `BeeEdge` and chat header |

## 3. Shared anatomy and tokens

### Reference

`board_agents` prints the shared rules at `build_hive_style.py:974-983`:

- name - Inter SemiBold 14.5
- role tag - mono 9.5, +12% tracking
- state - "dot or pill, never both"
- progress - 5px track, 3px worker
- meta - mono 9.5, `#8A7A52` (the builder's actual `INK500` is `#75663F`)
- radius - 14 super / 12 worker
- padding - 16
- gap - 8 / 16 / 24 only

The visual rule at `:915-917`: colour is reserved for state; honey = healthy,
crimson = broken route. The card functions use different exact type steps per card:
SuperAgent name 14.5, Worker name 13, Tool title 13.5, Error title 13.

### Current

- `Frame` (`nodes.tsx:41-68`) always uses `border-[1.5px]` and `borderColor` hairline
  unless selected or `error`; radius is passed per caller (correct).
- `Progress` (`:70-84`) uses `bg-recessed` for the track and flat `bg-honey-500` for
  the fill at every size.
- All role tags and meta rows use `type-mono-meta`, which is 11px / +8% uppercase
  (`tokens.css:81-85`), not the reference 9-9.5px / +6-12%.
- `tokens.css:45-47` has only one-layer elevations:
  `e1 0 2 8 .09`, `e2 0 6 20 .12`, `e3 0 12 34 .13`. Reference:
  `e1 = 0 1 3 rgba(40,32,16,.07) + 0 8 24 rgba(40,32,16,.08)`;
  `e2 = 0 2 6 .08 + 0 20 48 .12`;
  `e3 = 0 0 80 -10 #8A5A1C3D + 0 24 48 #60481833`.
- Missing tokens: `#E5C27A` (border gold), `#EFE7D2` (honeycomb line), `#F3F1EC`
  (user bubble), `#FFF7F5` (error surface), `#E05B4A` / `#6FBF73` (traffic dots).
  `#E8B850` already exists as `--color-honey-400`; `#F1E9CF` already exists as
  `--color-recessed`; `#3F3115` is `--color-ink-800`.

### Gap

The skeleton is right; the type and fill semantics are not. A single `type-mono-meta`
cannot serve an 8.5px state pill, a 9px worker tag, a 9.5px meta row, and an 11px
toolbar label. The border width is also wrong for three of four cards: Worker and
Tool are 1px in the reference, Error is 1.5px crimson, SuperAgent is 1.5px gold.

### Fix seam

`web/src/styles/tokens.css` and `web/src/styles/index.css`:

```css
@theme {
  --color-border-gold: #e5c27a;
  --color-honeycomb-line: #efe7d2;
  --color-chat-user: #f3f1ec;
  --color-error-surface: #fff7f5;
  --color-code-red: #e05b4a;
  --color-code-green: #6fbf73;

  --shadow-e1: 0 1px 3px rgb(40 32 16 / 0.07), 0 8px 24px rgb(40 32 16 / 0.08);
  --shadow-e2: 0 2px 6px rgb(40 32 16 / 0.08), 0 20px 48px rgb(40 32 16 / 0.12);
  --shadow-e3: 0 0 80px -10px rgb(138 90 28 / 0.24), 0 24px 48px rgb(96 72 24 / 0.20);
}

/* Do not reuse type-mono-meta for card internals. */
.type-role-tag {
  font-family: var(--font-mono);
  font-size: 9.5px;
  line-height: 13px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}
.type-role-tag-sm {
  font-size: 9px;
  line-height: 12px;
  letter-spacing: 0.06em;
}
.type-card-meta {
  font-family: var(--font-mono);
  font-size: 9px;
  line-height: 12px;
  letter-spacing: 0;
}
.type-card-meta-lg {
  font-size: 9.5px;
  line-height: 13px;
}
```

`Frame` should become variant-driven rather than every caller guessing:

```tsx
type CardVariant = 'super' | 'worker' | 'tool' | 'error';

const FRAME: Record<CardVariant, { radius: number; border: string; fill: string }> = {
  super:  { radius: 14, border: 'border-[1.5px] border-[color:var(--color-border-gold)]', fill: 'bg-panel' },
  worker: { radius: 12, border: 'border border-hairline', fill: 'bg-panel' },
  tool:   { radius: 14, border: 'border border-hairline', fill: 'bg-panel' },
  error:  { radius: 12, border: 'border-[1.5px] border-signal', fill: 'bg-[var(--color-error-surface)]' },
};

function Frame({ data, variant, children }: {
  data: CanvasNodeData; variant: CardVariant; children: ReactNode;
}) {
  const f = FRAME[variant];
  return (
    <div
      className={`relative flex h-full w-full flex-col overflow-hidden text-left ${f.fill} ${f.border}`}
      style={{
        borderRadius: f.radius,
        boxShadow: data.selected ? 'var(--shadow-e2)' : 'var(--shadow-e1)',
        outline: data.selected ? '2px solid var(--color-ink-700)' : undefined,
        outlineOffset: data.selected ? 2 : undefined,
      }}
    >
      <Handle type="target" position={Position.Left} className="!h-2 !w-2 !border-0 !bg-honey-500 opacity-0" />
      {children}
      <Handle type="source" position={Position.Right} className="!h-2 !w-2 !border-0 !bg-honey-500 opacity-0" />
    </div>
  );
}
```

When P1-7 changes the layout to top-down, move the handles to `Position.Top` /
`Position.Bottom` in the same edit.

## 4. HyperAgent / hive core relation

### Reference

- The HyperAgent **is** the hive chat core. `chat_card()` (`build_hive_style.py:815-843`)
  is a 420x500 shell drawn inside `hive_core()` (`:522-535`), five stacked radial-gold
  lobes with a `#7A4E14` 1.6px outline and an e3 shadow.
- It is the only eternal node: panning and zooming never move it; `Cmd/Ctrl+0`
  recenters the viewport on it (`README.md:46-48`, board-05 caption).
- There is no `hyperagent_card()` in the component board. The five node types are
  HyperAgent core (chat), SuperAgent, WorkerAgent, ToolAgent, broken route.

### Current

Three layers attempt to represent it:

1. `Canvas.tsx:244-268` draws a static 680x720 SVG with **four** flat ellipses and
   no outline/shadow. The highlight ellipses are an extra fifth/sixth mark but not
   the reference lobe geometry.
2. `ChatCore.tsx:166-172` draws a fixed `absolute inset-0 flex items-center
   justify-center` 420x500 panel. This part is correctly viewport-pinned.
3. `graph.ts:86-98` still emits a React Flow node for every task, including the live
   root task whose `kind` the backend sets to `hyperagent`
   (`execution.rs:1322-1335`; `:151-162` maps `planner`/`hyper`/depth-0 to
   `HyperAgent`). `HyperAgentNode` (`nodes.tsx:185-207`) renders a 420x500 clipped
   hexagon with a small pill label. Because the root's server `NODE_SPAWN` carries no
   label (`execution.rs:765-778`), the store fallback is
   `Task <short-id>` (`store.ts:1424`). That string is what the hex card shows.

At the initial centred viewport the graph root sits exactly behind the chat shell
(both 420x500, both centred), so it is invisible. It becomes visible when:
- the operator hides the chat (`ChatCore.tsx:101-113` / `:195-202`);
- the operator pans/zooms (root moves with the world, chat does not);
- the minimap paints it as a 420x500 honey rectangle;
- keyboard focus lands on the 420x500 hidden node (`Canvas.tsx:311` sets
  `nodesFocusable`).

### Gap

Two entities for one concept; the graph one is a hex card that does not contain a
chat; the fixed one is a chat that is not a hive. The hive core is also a 4-lobe
approximation instead of the reference 5-lobe stack, at 680x720 instead of 660x700,
with no lobe outline. This is the largest visual/structural divergence in the
rebuild.

### Fix seam

Keep **one** HyperAgent: the fixed `ChatCore` inside a fixed `HiveCore`. Use the
server's root task only as a layout anchor, never as a rendered node.

`web/src/flow/graph.ts`:

```ts
export const HIVE_ANCHOR_ID = '__hive_core__';

const ROOT_KINDS: NodeKindLabel[] = ['hyperagent'];

// Tool cards count as workers in the reference metric ("3 workers" for
// tailwind-builder + db-schema + Prisma Generator).
function countChildren(
  tasks: Readonly<Record<string, TaskRecord>>,
  parentId: string,
): number {
  return Object.values(tasks).filter(
    (task) => task.parentId === parentId && (task.kind === 'worker' || task.kind === 'tool'),
  ).length;
}

export interface CanvasNodeData extends Record<string, unknown> {
  readonly task: TaskRecord;
  readonly selected: boolean;
  readonly onSelect: (taskId: string) => void;
  readonly workerCount: number | null;
}

export function buildNodes(
  tasks: Readonly<Record<string, TaskRecord>>,
  selectedTaskId: string | null,
  onSelect: (taskId: string) => void,
  interactive = true,
  allTasks: Readonly<Record<string, TaskRecord>> = tasks,
): CanvasNode[] {
  // Lay out the full graph so hiding/focusing does not re-flow every surviving
  // card; visible tasks keep their world position across filter toggles.
  const positions = layoutTasks(allTasks);
  const roots = Object.values(allTasks).filter((t) => ROOT_KINDS.includes(t.kind));
  const root = roots[0];

  const nodes = Object.values(tasks)
    .filter((task) => !ROOT_KINDS.includes(task.kind))
    .map((task) => {
      const size = nodeSize(task.kind);
      return {
        id: task.id,
        type: task.kind,
        position: positions[task.id] ?? { x: 0, y: 0 },
        style: { width: size.width, height: size.height },
        data: {
          task,
          selected: task.id === selectedTaskId,
          onSelect,
          workerCount: countChildren(allTasks, task.id),
        },
        ariaLabel: `${task.label} (${task.kind}, ${task.state})`,
        draggable: interactive,
        selectable: interactive,
      };
    });

  if (root !== undefined) {
    nodes.push({
      // invisible 1x1 anchor at the world origin; the fixed hive/chat covers it.
      id: HIVE_ANCHOR_ID,
      type: 'hiveAnchor',
      position: { x: -0.5, y: -0.5 },
      style: { width: 1, height: 1, opacity: 0, pointerEvents: 'none' },
      data: { task: root, selected: false, onSelect: () => {}, workerCount: null },
      ariaLabel: 'Hive core anchor',
      draggable: false,
      selectable: false,
    });
  }
  return nodes;
}

export function buildEdges(
  visibleTasks: Readonly<Record<string, TaskRecord>>,
  allTasks: Readonly<Record<string, TaskRecord>> = visibleTasks,
): Edge[] {
  const rootIds = new Set(
    Object.values(allTasks)
      .filter((task) => ROOT_KINDS.includes(task.kind))
      .map((task) => task.id),
  );
  return Object.values(visibleTasks)
    .filter(
      (task) =>
        task.parentId !== null &&
        (visibleTasks[task.parentId] !== undefined || rootIds.has(task.parentId)),
    )
    .map((task) => ({
      id: `edge-${task.parentId}-${task.id}`,
      source: rootIds.has(task.parentId ?? '') ? HIVE_ANCHOR_ID : (task.parentId ?? ''),
      target: task.id,
      type: 'bee',
      animated: false,
      data: {
        broken:
          task.kind === 'error' ||
          task.state === 'failed' ||
          task.state === 'killed' ||
          task.state === 'interrupted',
      },
    }));
}
```

`NODE_RENDERERS` adds a null renderer so the anchor exists for edge routing but paints
nothing; the minimap will no longer show a 420x500 root:

```tsx
export const NODE_RENDERERS = {
  hiveAnchor: () => <div aria-hidden="true" />,
  superagent: SuperAgentNode,
  worker: WorkerAgentNode,
  tool: ToolAgentNode,
  error: BrokenRouteNode,
  // hyperagent is intentionally absent: the fixed ChatCore is the only HyperAgent.
} as const;
```

Do **not** merely add `hidden: true`; React Flow still lays out a hidden node and its
edges can disappear. The explicit 1x1 anchor keeps first-level edges attached at the
origin while removing the card, the minimap blob and the tab stop.

`Canvas.tsx` currently calls `buildEdges(visibleTasks)` after the hide/focus filters
(`Canvas.tsx:170`). Pass the full `tasks` map as a second argument (or compute
`rootIds` from full `tasks`) so hiding a finished root does not silently drop every
first-level edge. Keep the visible-node filter only for rendering, not for deciding
whether the anchor exists. Do the same for `buildNodes(visibleTasks, ..., tasks)` so
`workerCount` counts children that the filters hid, not just the visible subset.

Also change `Canvas.tsx:336-343` to stop offering `hyperagent` as a rewire target
(the reference re-parents a worker onto another SuperAgent only), and change the
root-kind fallback in `buildNodes` if future snapshots ever carry `kind`.

`Canvas.tsx` hive core exact five-lobe replacement:

```tsx
// Reference: build_hive_style.py:522-535. Values are fractions of the 660x700 box.
const HIVE_LOBES = [
  { x: 0.16, y: 0.02, w: 0.68, h: 0.26, edge: '#FFF8E5' },
  { x: 0.06, y: 0.20, w: 0.88, h: 0.30, edge: '#F6E7BE' },
  { x: 0.00, y: 0.39, w: 1.00, h: 0.34, edge: '#F0C050' },
  { x: 0.06, y: 0.60, w: 0.88, h: 0.30, edge: '#D9A02F' },
  { x: 0.16, y: 0.78, w: 0.68, h: 0.22, edge: '#8A5A1C' },
] as const;

function HiveCore() {
  return (
    <svg
      aria-hidden="true"
      className="pointer-events-none absolute left-1/2 top-1/2 z-0 -translate-x-1/2 -translate-y-1/2"
      width="660" height="700" viewBox="0 0 660 700"
    >
      <defs>
        {HIVE_LOBES.map((lobe, i) => (
          <radialGradient key={i} id={`hive-lobe-${i}`} cx="50%" cy="42%" r="62%">
            <stop offset="0%" stopColor="#FFFDF4" />
            <stop offset="55%" stopColor={lobe.edge} />
            <stop offset="100%" stopColor="#8A5A1C" />
          </radialGradient>
        ))}
      </defs>
      {HIVE_LOBES.map((lobe, i) => (
        <ellipse
          key={i}
          cx={(lobe.x + lobe.w / 2) * 660}
          cy={(lobe.y + lobe.h / 2) * 700}
          rx={(lobe.w * 660) / 2}
          ry={(lobe.h * 700) / 2}
          fill={`url(#hive-lobe-${i})`}
          stroke="#7A4E14"
          strokeWidth="1.6"
          style={{
            // Reference e3: 0 0 80px -10 #8A5A1C3D + 0 24px 48px #60481833.
            // CSS drop-shadow has no spread, so 40px std-dev approximates 80px.
            filter:
              'drop-shadow(0 0 40px rgb(138 90 28 / 0.24)) drop-shadow(0 24px 24px rgb(96 72 24 / 0.20))',
          }}
        />
      ))}
    </svg>
  );
}
```

Add `HIVE_LOBES` and the honeycomb tile to `tokens.ts`/`tokens.css` rather than
leaving the hex literals in `Canvas.tsx` (G-UI-11).

## 5. SuperAgent card

### Reference

`build_hive_style.py:846-861`:

- rect 420x150, fill `#FFFFFF`, r14, 1.5px `#E5C27A`, e1.
- header rect 40 high, `#FBF3DC`, top radii 14.
- name Inter SemiBold 14.5 at x+30, y+11, `#282010`.
- state pill 68x18 at x+w-84, y+11, honey gradient `#F0C050 -> #C88030`, r999;
  text `RUNNING` mono SemiBold 8.5 `#3F3115`, +8% tracking. **No dot in the pill.**
- role tag `SUPERAGENT` mono 9.5 `#75663F`, +12% tracking, at x+16, y+50.
- description Inter 11.5 `#6B5A33` at x+16, y+68:
  "decomposes intent, owns the sub-hive".
- progress at x+16, y+h-46: track 388x5 `#E5E1D8`, fill honey gradient.
- meta mono 9.5 `#75663F` at x+16, y+h-32: `3 workers  -  12.4k ctx`.

### Current

`nodes.tsx:86-118`:

- `Frame radius={14}` but the frame border is hairline 1.5 (selected ink 700).
- header `h-10 bg-honey-100 px-4` (title at 16, not 30).
- title 14.5/18.5 semibold (size correct).
- pill `bg-honey-300 px-2 py-0.5` with `StateGlyph size={8}` **and** the state label
  inside; `type-mono-meta` is 11px; no fixed 68x18; no gradient.
- body `px-4 py-2`; role tag 11px; description 11.5 (correct); progress 5px but
  track `recessed` and fill flat `honey-500`.
- meta row left is the literal string `workers` with no count; right is
  `formatPercent(progress, 0)`. Reference has `n workers - ctx`, no percent text.

### Gap

| Property | Reference | Current | Delta |
|---|---|---|---|
| Border | 1.5px gold | 1.5px hairline | wrong colour |
| Header inset | 30px | 16px | 14px |
| State | 68x18 pill, no dot, 8.5 | auto pill + dot + 11px text | both cues, wrong type |
| Role tag | 9.5 mono +12% | 11 mono +8% | too large, too tight |
| Track | `#E5E1D8` | `recessed` | wrong |
| Fill | honey gradient | flat honey-500 | wrong |
| Meta | `3 workers - 12.4k ctx` | `workers` + `55%` | missing count, wrong right side |

### Fix seam

`nodes.tsx:86-118`, plus `CanvasNodeData.workerCount` and the `countChildren` helper
from P0-1. Use the shared `Frame variant="super"` and `Progress bar="super"`.

```tsx
export function SuperAgentNode({ data }: HiveNodeProps) {
  const { task } = data;
  return (
    <Frame data={data} variant="super">
      <div className="flex h-10 shrink-0 items-center gap-2 rounded-t-[14px] bg-honey-100 pl-[30px] pr-4">
        <button
          type="button"
          className="min-w-0 flex-1 truncate text-left text-[14.5px] font-semibold leading-[18px] text-ink-900"
          onClick={() => data.onSelect(task.id)}
          aria-label={`${task.label} - SuperAgent, ${visualFor(task.state).label}`}
          aria-pressed={data.selected}
        >
          {task.label}
        </button>
        <StatePill state={task.state} />
      </div>

      <button
        type="button"
        className="flex min-h-0 flex-1 flex-col px-4 pb-4 pt-[10px] text-left"
        onClick={() => data.onSelect(task.id)}
      >
        <span className="type-role-tag text-ink-500">SUPERAGENT</span>
        <span className="mt-[5px] text-[11.5px] leading-[16px] text-ink-600">
          decomposes intent, owns the sub-hive
        </span>
        <div className="mt-auto">
          <Progress value={task.progress} bar="super" />
          <div className="type-card-meta-lg mt-[6px] flex items-center gap-2 text-ink-500">
            <span>{data.workerCount ?? 0} workers</span>
            <span aria-hidden="true">-</span>
            <span>ctx {STRINGS.node.noMetrics}</span>
          </div>
        </div>
      </button>
    </Frame>
  );
}
```

`StatePill` and `Progress` are defined in sections 6 and 7. If the server later
reports context tokens, add `ctxTokens` to `TaskRecord` and replace the `-`; do not
compute it from unrelated metrics.

## 6. State pill / dot rules

### Reference

- SuperAgent: **pill only**. 68x18, gradient, mono 8.5 SemiBold `#3F3115`, +8%
  tracking, `RUNNING` text. No dot (`superagent_card()` lines 852-854).
- Worker: **dot only**. 9px diameter (r4.5) at x+16/y+22; healthy `#E8B850`
  (honey-400), running `#C88030` (honey-600), failed `#B23A2E` (signal),
  queued `#E5E1D8` (hairline). No visible state text on the card
  (`worker_card()` line 866; board 05 states list lines 966-972).
- Tool: no state glyph; the role pill says `TOOL`.
- Error: warning triangle, not a state dot/pill.
- Board 10 adds shape + text for non-dot states; colour is never the only channel.

### Current

- `DOT_STATES` (`agentState.ts:30`) is the correct four.
- `visualFor()` (`:76-84`) returns `toneToken: state` for those four. `tokenColor()`
  (`nodes.tsx:15-19`) does not know state names, so `healthy`, `running`, `failed`,
  `queued` all fall through to `var(--color-ink-500)`. This is a verified one-line
  defect: every worker dot in the current UI is grey-brown.
- `StateGlyph` (`nodes.tsx:21-39`) always renders a visible text label next to the
  glyph. In the SuperAgent pill this makes dot + text; on the Worker it adds a
  visible `RUNNING` label that the reference does not have.
- The dot is an SVG with `width=size` and `viewBox=0 0 12 12`, circle r=4; at
  `size=9` the rendered dot is about 6px, not 9px.

### Gap

State colours are wrong, dot size is wrong, and the "dot or pill, never both" rule is
violated in both places. Non-dot states should keep the shape cue but not acquire a
permanent 11px label that breaks the card layout.

### Fix seam

`state/agentState.ts:76-84`:

```ts
const DOT_TONE: Readonly<Record<DotState, string>> = {
  healthy: 'honey-400',
  running: 'honey-600',
  failed: 'signal',
  queued: 'hairline',
};

export function visualFor(state: AgentState): StateVisual {
  const label = STRINGS.agentStates[state];
  if (isDotState(state)) {
    return { state, label, dot: state, shape: 'dot', toneToken: DOT_TONE[state] };
  }
  const visual = NON_DOT_VISUAL[state as Exclude<AgentState, DotState>];
  return { state, label, dot: null, shape: visual.shape, toneToken: visual.toneToken };
}
```

`nodes.tsx`: split the renderers; never use the labelled `StateGlyph` inside a card.

```tsx
export function StateDot({ state, size = 9 }: { state: AgentState; size?: number }) {
  const visual = visualFor(state);
  const color = `var(--color-${visual.toneToken})`;
  return (
    <span
      role="img"
      aria-label={visual.label}
      title={visual.label}
      className="inline-flex shrink-0 items-center justify-center"
      style={{ width: size, height: size, color }}
    >
      <svg width={size} height={size} viewBox="0 0 12 12" aria-hidden="true">
        {visual.dot ? (
          <circle cx="6" cy="6" r="4.5" fill="currentColor" />
        ) : (
          <StateShapeGlyph shape={visual.shape} />
        )}
      </svg>
    </span>
  );
}

export function StatePill({ state }: { state: AgentState }) {
  const visual = visualFor(state);
  return (
    <span
      className="flex h-[18px] min-w-[68px] shrink-0 items-center justify-center whitespace-nowrap rounded-full px-[8px] font-mono text-[8.5px] font-semibold uppercase leading-none tracking-[0.08em] text-ink-800"
      style={
        state === 'failed' || state === 'killed' || state === 'interrupted'
          ? { background: 'var(--color-error-surface)', color: 'var(--color-signal)' }
          : state === 'queued' || state === 'cancelled'
            ? { background: 'var(--color-recessed)', color: 'var(--color-ink-800)' }
            : { background: 'linear-gradient(135deg, var(--color-honey-300), var(--color-honey-600))' }
      }
      title={visual.label}
    >
      {visual.label}
    </span>
  );
}
```

`StateShapeGlyph` is the existing non-dot branch of `StateGlyph` (diamond, square,
triangle, ring, hex relative to `visual.shape`). Keep `StateGlyph` for the panels
that want a visible label; card internals use `StateDot`/`StatePill`.

## 7. Progress track / bar

### Reference

- SuperAgent: track 5px `#E5E1D8`; fill honey gradient `#F0C050 -> #C88030`.
- Worker: track 3px `#E5E1D8`; fill **solid** `#E8B850` (not a gradient).
- Tool: track 6px `#E5E1D8`; fill honey gradient.
- Width transition only, 200ms; never a spinner on a card (`Style Kit` §5).

### Current

`nodes.tsx:70-84`: one component, track `bg-recessed` (#F1E9CF), fill flat
`bg-honey-500` for every card. Heights are passed correctly (super 5, worker 3,
tool 6). `value === null` renders no fill, which is honest, but the surrounding meta
sometimes still implies a metric.

### Gap

Track colour, fill treatment and worker-specific fill are all wrong. No transition.

### Fix seam

Replace `Progress` with a variant-aware component:

```tsx
function Progress({ value, bar }: { value: number | null; bar: 'super' | 'worker' | 'tool' }) {
  const height = bar === 'worker' ? 3 : bar === 'tool' ? 6 : 5;
  const fill =
    bar === 'worker'
      ? 'bg-honey-400'
      : 'bg-[linear-gradient(135deg,var(--color-honey-300),var(--color-honey-600))]';
  const clamped = value === null ? null : Math.max(0, Math.min(100, value));
  return (
    <div
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={clamped ?? undefined}
      className="w-full overflow-hidden rounded-full bg-hairline"
      style={{ height }}
    >
      {clamped === null ? null : (
        <div
          className={`h-full rounded-full transition-[width] duration-200 ${fill}`}
          style={{ width: `${clamped}%` }}
        />
      )}
    </div>
  );
}
```

## 8. WorkerAgent card

### Reference

`build_hive_style.py:864-873`:

- 320x96, paper, r12, 1px hairline, e1. No header fill.
- dot 9px at x+16/y+22, honey-400.
- name Inter Medium 13 at x+30, y+13, ink 900.
- role tag `WORKER  -  react/vite` mono 9, `#75663F`, +6% at x+30, y+32.
- progress track 288x3 at x+16, y+h-18 (y+78), fill honey-400 solid.
- percent `55%` mono 9 at x+w-40, y+h-40 (x+280, y+56).

### Current

`nodes.tsx:120-139`:

- `Frame radius={12}`, border 1.5 hairline (should be 1px).
- body `px-4 py-3`; row `StateGlyph size={9}` + label + name 13 + percent 11 on the
  same line.
- role tag `type-mono-meta` 11px, text only `WORKER` (no role suffix).
- progress 3px at bottom, but flat honey-500 and recessed track.
- `formatCtx(null)` renders `-` under the bar; reference has no ctx line.

### Gap

| Property | Reference | Current |
|---|---|---|
| Border | 1px hairline | 1.5px |
| Dot | 9px, state colour | ~6px, grey + visible text |
| Name | x+30 | after dot + gap |
| Role | 9px +6% | 11px +8% |
| Percent | above bar, right | in header row |
| Fill | solid honey-400 | flat honey-500 |
| Footer | none | `ctx -` |

The role suffix `react/vite` is not in the frozen `NODE_*` payload
(`contract.ts:220-244` has no role field), so **do not invent it**. `WORKER` alone is
the honest tag; add the suffix only when the server starts reporting a role/subtype.

### Fix seam

`nodes.tsx:120-139`:

```tsx
export function WorkerAgentNode({ data }: HiveNodeProps) {
  const { task } = data;
  return (
    <Frame data={data} variant="worker">
      <button
        type="button"
        className="relative flex h-full w-full flex-col px-4 pb-[18px] pt-[13px] text-left"
        onClick={() => data.onSelect(task.id)}
      >
        <span className="absolute left-3 top-[17.5px]">
          <StateDot state={task.state} size={9} />
        </span>
        <span className="truncate pl-[30px] text-[13px] font-medium leading-[18px] text-ink-900">
          {task.label}
        </span>
        <span className="type-role-tag-sm mt-[5px] pl-[30px] text-ink-500">WORKER</span>
        {task.progress === null ? null : (
          <span className="type-card-meta absolute bottom-[40px] right-4 text-ink-500">
            {formatPercent(task.progress, 0)}
          </span>
        )}
        <div className="mt-auto">
          <Progress value={task.progress} bar="worker" />
        </div>
      </button>
    </Frame>
  );
}
```

Remove the `formatCtx(null)` line. If context tokens become available, add them to
`TaskRecord` and to the worker meta; do not render a permanent placeholder.

## 9. ToolAgent card

### Reference

`build_hive_style.py:876-890`:

- 420x240, paper, r14, 1px hairline, e1.
- title Inter SemiBold 13.5 at x+16, y+14.
- `TOOL` pill 80x18 at x+w-96, y+14, fill `#F1E9CF`, r999; text mono SemiBold 8.5
  `#4E3A15`, +10%.
- dark box x+16, y+44, w-32, h-108 (132 high), `#3F3115`, r8.
- three dots r3.4 at x+30+i*12, y+13: `#E05B4A`, `#E8B850`, `#6FBF73`.
- code mono 10.5 `#F1E9CF` at x+30, y+28, width w-60, lh150%.
- progress 388x6 at x+16, y+h-44 (y+196), honey gradient.
- meta mono 9 at x+16, y+h-30 (y+210): `schema.prisma  -  0.78`.

### Current

`nodes.tsx:141-164`:

- Frame radius 14, border 1.5 hairline (should be 1px).
- title 13.5 semibold (correct); `TOOL` pill uses `type-mono-meta` 11px, auto width,
  `bg-recessed` (colour right, dimensions/type wrong).
- dark box `mt-2 flex min-h-0 flex-1 items-center rounded-[8px] bg-ink-800 px-3 py-2`
  with only `tool output not measured`; no dots, no 132px fixed height, no code.
- progress 6px, flat honey-500/recessed.
- meta `score` + `-`; no output/identity line.

### Gap

The code box is the signature of the ToolAgent and it is currently an empty dark
rectangle. The tool has no output field in `TaskRecord`, so the honest state is "not
reported", not fabricated code. The three traffic dots are static chrome and may be
rendered even when empty; the code text may not.

### Fix seam

`nodes.tsx:141-164`:

```tsx
export function ToolAgentNode({ data }: HiveNodeProps) {
  const { task } = data;
  return (
    <Frame data={data} variant="tool">
      <div className="relative h-full px-4 pb-[18px] pt-[14px]">
        <div className="flex items-center justify-between gap-2">
          <span className="truncate text-[13.5px] font-semibold leading-[18px] text-ink-900">
            {task.label}
          </span>
          <span className="flex h-[18px] w-20 shrink-0 items-center justify-center rounded-full bg-recessed font-mono text-[8.5px] font-semibold uppercase leading-none tracking-[0.10em] text-ink-700">
            TOOL
          </span>
        </div>

        <button
          type="button"
          className="absolute left-4 right-4 top-11 h-[132px] overflow-hidden rounded-[8px] bg-ink-800 px-[14px] text-left"
          onClick={() => data.onSelect(task.id)}
        >
          <span className="flex gap-[5px] pt-[13px]" aria-hidden="true">
            <i className="h-[7px] w-[7px] rounded-full bg-[var(--color-code-red)]" />
            <i className="h-[7px] w-[7px] rounded-full bg-honey-400" />
            <i className="h-[7px] w-[7px] rounded-full bg-[var(--color-code-green)]" />
          </span>
          <span className="mt-[11px] block font-mono text-[10.5px] leading-[15.75px] text-recessed">
            {STRINGS.node.noToolOutput}
          </span>
        </button>

        <div className="absolute bottom-11 left-4 right-4">
          <Progress value={task.progress} bar="tool" />
        </div>
        <div className="type-card-meta absolute bottom-[18px] left-4 right-4 flex justify-between text-ink-500">
          <span>output {STRINGS.node.noMetrics}</span>
          <span>score {STRINGS.node.noMetrics}</span>
        </div>
      </div>
    </Frame>
  );
}
```

If a tool-output field is added later, render real lines inside the same box; do not
replace the empty state with sample code.

## 10. Broken route / error card

### Reference

`build_hive_style.py:893-902`:

- 320x110, fill `#FFF7F5`, r12, 1.5px `#B23A2E`.
- warning icon 12px at x+16, y+16.
- title Inter SemiBold 13 `#B23A2E` at x+36, y+14.
- meta mono 9 `#6B5A33` at x+36, y+34: `db-schema  -  E0421  -  retry 2/3`.
- body Inter 10.5 `#4E3A15` at x+16, y+58, width w-32, lh145%.
- below the card: dashed crimson route `[6,6]`, bee at 50%, 13px, warning glyph,
  arrowhead (the board is a specimen; in the live app the incoming/outgoing edge is
  the route).

### Current

`nodes.tsx:166-183`:

- `Frame error radius={12}`: border 1.5 signal (correct), fill `bg-panel` (white,
  wrong), no warning icon (a text `▲`), title is `task.label`, then
  `task.errorCode ?? "Compile failed"`, then static `broken route - error state`,
  then `attempt` + `-`.
- There is no error message field in `TaskRecord`, so the current card cannot show
  the reference body line without a server field.
- The edge is now crimson dashed via `data.broken` (`graph.ts:110-112`,
  `BeeEdge.tsx:27-53`), but it has no halted bee/warning glyph, and `graph.ts`
  misses `kind === 'error'`.

### Gap

The most important gap is truthfulness: `"Compile failed"` is a diagnosis, not a
fallback. If the server sent an error code, show it; if not, say so. The fill and
icon are visual; the message area must stay honest.

### Fix seam

`nodes.tsx:166-183`:

```tsx
export function BrokenRouteNode({ data }: HiveNodeProps) {
  const { task } = data;
  return (
    <Frame data={data} variant="error">
      <button
        type="button"
        className="flex h-full w-full flex-col px-4 pb-3 pt-[14px] text-left"
        onClick={() => data.onSelect(task.id)}
      >
        <div className="flex min-w-0 items-center gap-2 text-signal">
          <WarningGlyph size={12} />
          <span className="truncate text-[13px] font-semibold leading-[18px]">{task.label}</span>
        </div>
        <span className="type-card-meta mt-[5px] text-ink-600">
          ERROR - {task.errorCode ?? 'code not reported'} - retry {STRINGS.node.noMetrics}
        </span>
        <span className="mt-2 text-[10.5px] leading-[15px] text-ink-700">
          error message not reported
        </span>
        <span className="type-card-meta mt-auto text-ink-600">
          attempt {STRINGS.node.noMetrics}
        </span>
      </button>
    </Frame>
  );
}
```

Add `STRINGS.node.errorMessageMissing = 'error message not reported'` and
`STRINGS.node.errorCodeMissing = 'code not reported'`. If the server later carries a
message, render it here; do not turn a missing code into "Compile failed".

`graph.ts:110-112` must include the error kind:

```ts
data: {
  broken:
    task.kind === 'error' ||
    task.state === 'failed' ||
    task.state === 'killed' ||
    task.state === 'interrupted',
},
```

`BeeEdge.tsx` broken branch: keep `stroke-dasharray: 6 6`, but replace the moving bee
with a halted bee plus a 6px warning marker at the bee's position (reference prompt 7:
"the small bee vector is paused with an active warning icon indicator"). No animation
on the broken route.

## 11. Bee edge

### Reference

`bee_route()` (`build_hive_style.py:318-342`) and board 07 (`:1222-1226`):

- straight vector between endpoints, 1.5px, honey-400 `#E8B850`, dash `[10,10]`.
- arrowhead triangle 9x8 at the target.
- bee at `bee_at` 0.45-0.6, 13-18px, facing the direction of travel.
- broken route: crimson `#B23A2E`, dash `[6,6]`, bee paused with warning glyph.
- the bee is a routing arrow, never decoration (`:990-993`).

### Current

`BeeEdge.tsx:13-58`:

- `getBezierPath` (curved), not a straight/honeycomb vector.
- normal stroke `--color-honey-600` (`#C88030`), not honey-400; dash `[6,4]`, not
  `[10,10]`; no arrowhead.
- bee is a 14px simplified ellipse, angle computed from source->target endpoints,
  which is not the tangent of the Bezier at the midpoint.
- broken branch exists (`data.broken`): signal stroke, `[6,6]`, signal bee; no
  warning glyph, no halt distinction.

### Gap

The route reads as a generic React Flow curve, not the honeycomb data vector. The
reference is straight segments with a clear arrowhead and a larger bee. The current
angle bug is masked only because the card layout is LR; with a vertical layout it
would be more visible.

### Fix seam

`BeeEdge.tsx`:

```tsx
import { BaseEdge, EdgeLabelRenderer, getStraightPath } from '@xyflow/react';

export function BeeEdge(props: EdgeProps) {
  const [path, labelX, labelY] = getStraightPath({
    sourceX: props.sourceX,
    sourceY: props.sourceY,
    targetX: props.targetX,
    targetY: props.targetY,
  });
  const broken = (props.data as { broken?: boolean } | undefined)?.broken === true;
  const color = broken ? 'var(--color-signal)' : 'var(--color-honey-400)';
  const markerId = `bee-arrow-${props.id.replace(/[^a-zA-Z0-9_-]/g, '-')}`;
  const angle = (Math.atan2(props.targetY - props.sourceY, props.targetX - props.sourceX) * 180) / Math.PI;

  return (
    <>
      <defs>
        <marker id={markerId} viewBox="0 0 10 8" refX="9" refY="4"
          markerWidth="9" markerHeight="8" orient="auto">
          <path d="M0 0 L10 4 L0 8 Z" fill={color} />
        </marker>
      </defs>
      <BaseEdge
        id={props.id}
        path={path}
        markerEnd={`url(#${markerId})`}
        style={{ stroke: color, strokeWidth: 1.5, strokeDasharray: broken ? '6 6' : '10 10' }}
      />
      <EdgeLabelRenderer>
        <div
          aria-hidden="true"
          style={{
            position: 'absolute',
            transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px) rotate(${angle}deg)`,
            pointerEvents: 'none',
            color: broken ? 'var(--color-signal)' : 'var(--color-ink-800)',
          }}
        >
          <BeeGlyph size={18} />
          {broken ? <WarningBadge /> : null}
        </div>
      </EdgeLabelRenderer>
    </>
  );
}
```

Use `bee_at = 0.55` by interpolating the label point along the straight segment if
React Flow's `labelX/labelY` is not exactly 0.55; or keep the midpoint and document
the 5% difference. Extract `BeeGlyph` from the current SVG and add the three body
bands/two wings from `bee()` lines 282-315; use `body`/`stripe`/`wing` props so the
chat header can reuse it.

## 12. Chat core

### Reference

`chat_card()` (`build_hive_style.py:815-843`):

- shell 420x500, fill `#F8F4EB` (canvas), r18, shadow e2, **no border**.
- header 44px, gradient `135deg #3F3115 -> #604818`, top radii 18.
  - bee 18px at x+14, y+13 (body H300, stripe H800, wing white).
  - title `Hive AI Assistant` Inter SemiBold 14 at x+44, y+13, white.
  - close `x` 18px at x+w-30, y+8, `#FFFFFFB0`.
- user bubble x+16, y+60, w-32, 54 high, `#F3F1EC`, r10; text 13.5 Inter
  Regular ink-800 at x+28, y+72, lh150%.
- assistant bubble x+16, y+124, w-32, h-224 high, `#FBF3DC`, r10; text 13.5
  `#604818` at x+28, y+136.
- code box x+28, y+168, w-56, h-290, `#3F3115`, r8; three dots at x+44+i*14,
  y+15; code 13 mono `#F1E9CF` at x+44, y+34, lh150%.
- composer x+16, y+h-76, w-32, 56 high, white, r12, 1px hairline; placeholder 13.5
  `#75663F` at x+32, y+h-56; send/mic/clip 15px icons at x+w-102, -72, -44,
  y+h-58.

### Current

`ChatCore.tsx:166-272`:

- shell: `surface-panel flex h-[500px] w-[min(420px,92%)] flex-col rounded-[18px] p-3`
  + `boxShadow: e3`. Correct size/radius; wrong fill (white), wrong border (hairline),
  wrong elevation (e3), wrong inner padding (12 vs 16).
- header: 44px, correct gradient, title 14 semibold; but no bee, no `x`; instead a
  subtitle line (`Pinned to hive centre - Connected`) and text buttons
  `UNPIN CHAT` / `CLOSE` built from `type-mono-meta` 11px. The two-line left block is
  not in the reference; the text buttons consume header width.
- body: `bg-panel-alt px-2 py-2`, no bubbles. Each message is a visible
  `YOU` / `HIVE AI ASSISTANT` mono label plus 12.5px plain text. Reference has 13.5px
  bubbles and no visible role label. No code block.
- empty state: `STRINGS.chat.empty` at `strings.ts:83` says "Drop a repository into
  the core hive..." but there is **no** `onDrop`/`dragover`/`DataTransfer` handler in
  `web/src` (grep returned zero). This is a false affordance.
- composer: a `type-mono-code` textarea (min 40px) beside a separate honey
  `SUBMIT PROMPT` button; no 56px white composer, no send/mic/clip icons. Hint line
  11px mono.
- refusal: a dark `bg-ink-900` card inside the chat (lines 206-216). Board 10's named
  refusal is a paper card with a crimson code box; board 06's toast is dark. Pick the
  surface: an in-chat run refusal should be the paper refusal card, not a toast.

### Gap

| Part | Reference | Current |
|---|---|---|
| Shell | canvas fill, no border, e2 | white, hairline border, e3 |
| Padding | 16 | 12 |
| Header | bee + title + `x` | title + status + two text buttons |
| Body | 10px bubbles, 13.5 text, code box | plain log, 12.5 text, no bubbles/code |
| Composer | 56px white, 3 icons | 40px textarea + button |
| Empty | sample conversation | false drop-a-repo instruction |
| Pin | none | fake toggle |

### Fix seam

`ChatCore.tsx:166-272` + `strings.ts:83`.

```tsx
<section
  aria-label={STRINGS.chat.aria}
  className="pointer-events-auto flex h-[500px] w-[min(420px,92%)] flex-col overflow-hidden rounded-[18px] bg-canvas shadow-[var(--shadow-e2)]"
>
  <header className="relative flex h-11 shrink-0 items-center rounded-t-[18px] bg-[linear-gradient(135deg,var(--color-ink-800),var(--color-honey-800))] pl-3.5 pr-2 text-white">
    <BeeGlyph size={18} body="var(--color-honey-300)" stripe="var(--color-honey-800)" wing="#FFFFFF" />
    <span className="ml-3 min-w-0 flex-1 truncate text-[14px] font-semibold leading-[18px]">
      {STRINGS.chat.title}
    </span>
    <button
      type="button"
      className="flex h-8 w-8 items-center justify-center text-[18px] leading-none text-white/70 hover:bg-white/10"
      onClick={() => setChatVisible(false)}
      aria-label={STRINGS.chat.closeAria}
    >
      x
    </button>
  </header>

  <div
    ref={listRef}
    className="min-h-0 flex-1 space-y-2 overflow-y-auto px-4 py-4"
    aria-live="polite"
    aria-label={STRINGS.chat.aria}
  >
    {messages.length === 0 ? (
      <p className="text-[13.5px] leading-[20px] text-ink-500">
        No messages yet - submit a prompt to create the first run.
      </p>
    ) : (
      messages.map((message) => <ChatBubble key={message.id} message={message} />)
    )}
    {streamingRunId ? (
      <p className="type-card-meta text-ink-500" role="status">{STRINGS.agentStates.streaming}</p>
    ) : null}
  </div>

  <form className="shrink-0 px-4 pb-4" onSubmit={onSubmit}>
    <div className="flex h-14 items-center gap-1 rounded-[12px] border border-hairline bg-panel px-3">
      <textarea
        id="composer"
        className="h-10 min-w-0 flex-1 resize-none bg-transparent py-2 text-[13.5px] leading-[20px] text-ink-900 placeholder:text-ink-500"
        placeholder={STRINGS.composer.placeholder}
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter' && !event.shiftKey) {
            event.preventDefault();
            void submit();
          }
        }}
        rows={2}
        disabled={connection !== 'open'}
      />
      <button
        type="submit"
        className="flex h-8 w-8 shrink-0 items-center justify-center rounded-[8px] text-ink-700 hover:bg-honey-100 disabled:opacity-40"
        disabled={connection !== 'open' || submitting || draft.trim().length === 0}
        aria-label={STRINGS.composer.submit}
      >
        <SendGlyph size={15} />
      </button>
      <button type="button" disabled aria-label="Voice input not available in this build" title="Not available in this build" className="h-8 w-8 opacity-40">
        <MicGlyph size={15} />
      </button>
      <button type="button" disabled aria-label="Attach file not available in this build" title="Not available in this build" className="h-8 w-8 opacity-40">
        <ClipGlyph size={15} />
      </button>
    </div>
    <p className="type-card-meta mt-1 text-ink-500">
      {connection === 'open' ? STRINGS.connection.open : STRINGS.connection[connection]}
      {' - '}
      {chat.pinned ? STRINGS.chat.pinned : STRINGS.chat.pin}
      {' - '}
      {STRINGS.composer.hint}
    </p>
  </form>
</section>
```

`ChatBubble`:

```tsx
function ChatBubble({ message }: { message: ChatMessage }) {
  const user = message.role === 'user';
  return (
    <article
      className={
        user
          ? 'rounded-[10px] bg-[var(--color-chat-user)] px-3 py-3'
          : 'rounded-[10px] bg-honey-100 px-3 py-3'
      }
    >
      <span className="sr-only">{user ? STRINGS.chat.userLabel : STRINGS.chat.assistantName}</span>
      <div className={`whitespace-pre-wrap text-[13.5px] leading-[20px] ${user ? 'text-ink-800' : 'text-honey-800'}`}>
        {message.text}
      </div>
    </article>
  );
}
```

For the reference code box, parse fenced blocks from the assistant text and render
each fence inside the assistant bubble as a `bg-ink-800` r8 block with the three dots
and 13px mono `text-recessed`. Do not add a code block when no fence exists; the
reference's sample code is content, not chrome.

The `x` above is the ASCII fallback in this doc; use `\u00d7` (the reference `x`
glyph) in JSX. The `Pin` control is removed from the header because it is not in the
reference and is currently fake (see section 13). If pinning must remain a user
action, use a 16px pin icon with `aria-pressed` next to `x` and **make the unpinned
state actually float/undock**; otherwise leave it as static "Pinned to hive centre"
copy in the footer.

## 13. HUD

### Reference

`build_hive_style.py:1228-1233` (board 07) and `:1056-1064` (board 06):

- 460x44 pill, white, 1.2px `#E5C27A`, e2.
- 5px `#3F7A55` dot at x+24, y+22.
- mono SemiBold 10 `#3F3115`, +6% tracking:
  `HYPERAGENT  -  4 SUPERAGENTS  -  12 WORKERS  -  98.2% GREEN`.
- delta `+3.4%` outside the pill, 9.5 mono verify.

### Current

`HudBadge.tsx:14-32`:

- wrapper `absolute left-1/2 top-3`; pill `h-[44px]` auto width, `border-honey-400`
  (not gold), e2, px-4, type-mono-meta 11 (not 10 semibold).
- dot is always `bg-verify` (6px), including `source='none'`, `greenPct=null`.
- suffix `GREEN` always renders, so empty state reads `- GREEN`; `DerivedBadge`
  (`DERIVED` / `NO METRICS`) is inside the pill and changes its width.
- delta, when present, is inside the pill instead of outside.

### Gap

The HUD is the one global "is the swarm healthy?" signal. A permanently green dot is
the most misleading single pixel in the UI. Width/type/tracking are also visibly off
against the 460x44 reference.

### Fix seam

`HudBadge.tsx:14-32`:

```tsx
const tone =
  hud.source === 'none' || hud.greenPct === null
    ? 'var(--color-hairline)'
    : hud.greenPct === 100
      ? 'var(--color-verify)'
      : hud.greenPct > 0
        ? 'var(--color-honey-400)'
        : 'var(--color-signal)';

<div className="pointer-events-none absolute left-1/2 top-3 z-20 -translate-x-1/2">
  <div
    className="flex h-[44px] w-[460px] items-center gap-2 rounded-full border-[1.2px] border-[color:var(--color-border-gold)] bg-panel px-4 font-mono text-[10px] font-semibold uppercase tracking-[0.06em] text-ink-800 shadow-[var(--shadow-e2)]"
    role="status"
    aria-label={STRINGS.hud.aria}
  >
    <span className="h-[5px] w-[5px] shrink-0 rounded-full" style={{ background: tone }} aria-hidden="true" />
    <span>{STRINGS.node.hyperAgent}</span>
    <span aria-hidden="true">-</span>
    <span>{formatCount(hud.superagents)} {STRINGS.hud.superagents}</span>
    <span aria-hidden="true">-</span>
    <span>{formatCount(hud.workers)} {STRINGS.hud.workers}</span>
    <span aria-hidden="true">-</span>
    <span>
      {hud.greenPct === null
        ? STRINGS.hud.sourceNone
        : `${formatPercent(hud.greenPct)} ${STRINGS.hud.green}`}
    </span>
  </div>
  {hud.delta === null ? null : (
    <span className="mt-1 block pl-1 font-mono text-[9.5px] text-verify">{formatDelta(hud.delta)}</span>
  )}
  <span className="sr-only">{/* DerivedBadge text for source transparency */}{hud.source}</span>
</div>
```

If the derived/server badge must stay visible, put it below the pill (not inside the
460px measurement), e.g. `DERIVED` at 9px `text-ink-500`. The reference has no badge;
transparency can live in the accessible description or a tooltip, but it should not
change the 460px pill.

## 14. Canvas, hive field, minimap

### Reference

Board 07 (`build_hive_style.py:1146-1249`):

- canvas begins at x=312, honeycomb R=44, line `#EFE6CE`/`#EFE7D2`, 1px.
- hive core 660x700 at 508,96; chat 420x500 at 620,196.
- HUD 460x44 at 540,68; minimap 220x150 at 1120,700.
- footer metadata `0.1x - 3.0x  -  Cmd+0 recentres the core hive  -  drag a worker
  to re-route` at 328,866, mono 9.5, `#A0916F`, +4%.

### Current

`Canvas.tsx`:

- honeycomb pattern 48x42, one stretched hex, stroke `currentColor` =
  `--color-honey-200`, opacity .9 (lines 235-243). Reference R=44 and
  `#EFE7D2`.
- hive 680x720, four ellipses, no stroke, no e3 (lines 244-268).
- new filter pills top-left (`Hide finished`, `Focus branch`, count) at lines
  270-302; not in reference, and the copy is hardcoded.
- minimap 220x150 with correct dimensions and border radius but a flat white field
  (no honeycomb), no e2 shadow; React Flow attribution text remains.
- zoom controls vertical bottom-left (lines 396-439); reference has footer metadata
  and no button stack (buttons are still useful, so this is P2).
- empty message at lines 441-445 uses `nodes.length === 0`; when filters hide all
  tasks, it says "No agents yet" although tasks exist.

### Gap

The field is too dense/dark, the hive is too large and has the wrong lobe count, and
the new filter chrome competes with the reference composition. The minimap is a
blank white rectangle.

### Fix seam

1. Replace the pattern with R=44 and the tokenised honeycomb line. Exactly:

```tsx
<pattern id="honeycomb-field" width="76.21" height="132" patternUnits="userSpaceOnUse">
  <path
    d="M38.105 0 L76.21 22 L76.21 66 L38.105 88 L0 66 L0 22 Z
       M76.21 66 L114.315 88 L114.315 132 L76.21 154 L38.105 132 L38.105 88 Z"
    fill="none"
    stroke="var(--color-honeycomb-line)"
    strokeWidth="1"
  />
</pattern>
```

The second path is clipped by the tile and completed by repetition; verify at 1x
before accepting. Keep `opacity: 1`; the current `.hive-canvas-field` (`index.css:121-127`)
sets color honey-200 and opacity .9, which must change to the new token.

2. Replace the four-ellipse SVG with `HiveCore` from section 4.

3. Empty-state honesty:

```tsx
const total = Object.keys(tasks).length;
const filteredEmpty = total > 0 && Object.keys(visibleTasks).length === 0;
...
{total === 0 ? (
  <p className="type-card-meta text-meta">{STRINGS.panels.agentsEmpty}</p>
) : filteredEmpty ? (
  <p className="type-card-meta text-meta">
    All {total} agents are hidden by the current filters.
  </p>
) : null}
```

Add the new string to `STRINGS.panels`. `nodes.length` is not the right emptiness test
because it includes only `visibleTasks`.

4. Minimap: add a honeycomb overlay (a wrapper with an absolutely positioned SVG
pattern and a transparent MiniMap background), set `boxShadow: var(--shadow-e2)`,
and add `proOptions={{ hideAttribution: true }}` to `<ReactFlow>`. Keep the 220x150
dimensions and r12.

5. Move the filter pills out of the reference canvas corner or restyle them as ghost
chips; move their copy into `STRINGS`. This is P2, not a blocker.

## 15. Faked data / false affordances

| # | Surface | What the UI claims | Evidence | Honest replacement |
|---|---|---|---|---|
| 1 | Chat empty state | "Drop a repository into the core hive..." | `strings.ts:83`; no `onDrop`/`dragover`/`DataTransfer` anywhere in `web/src` | "No messages yet - submit a prompt to create the first run." |
| 2 | Chat pin/unpin | Unpinning changes the chat's position/behaviour | `ChatCore.tsx:167-172` is always `absolute inset-0 flex items-center justify-center`; `pinChat` only sets `chat.pinned` and sends `CHAT_PIN` (`store.ts:2051-2056`) | Remove the toggle and show static "Pinned to hive centre"; or implement a real floating unpinned mode |
| 3 | HUD | Green dot means all green | `HudBadge.tsx:20` always `bg-verify`, including `source='none'` and `greenPct=null` | Tone from `hud.source`/`greenPct`; hairline when none; `NO METRICS` instead of `- GREEN` |
| 4 | Error card | "Compile failed" when no code was sent | `nodes.tsx:174` `errorCode ?? "Compile failed"`; `TaskRecord` has no message field | "code not reported"; body "error message not reported" |
| 5 | Canvas empty | "No agents yet" while tasks exist | `Canvas.tsx:441-445` uses `nodes.length === 0`, but `nodes` is built from `visibleTasks` (filters at `:113-144`) | Distinguish `tasks.length === 0` from filtered-empty; offer "Show finished" |
| 6 | HyperAgent card | A 420x500 "HyperAgent" node exists in the graph | `graph.ts:86-98` includes every task; backend root `kind='hyperagent'` (`execution.rs:1322-1335`); server root `NODE_SPAWN` has no label, so store falls back to `Task <id>` (`store.ts:1424`) | Hide the root node; the fixed `ChatCore` is the only HyperAgent (P0-1) |
| 7 | SuperAgent meta | A "workers" metric is shown | `nodes.tsx:110-113` prints the literal `workers` with no count | Derive child worker/tool count from `tasks`; show `0 workers - ctx -` until ctx exists |
| 8 | Tool meta | A `score` exists | `nodes.tsx:156-159` prints `score -`; no score field in `TaskRecord` | `output -` / `score -` with the explicit "not reported" wording, or omit the row |
| 9 | Worker footer | Context tokens exist | `nodes.tsx:134` calls `formatCtx(null)` unconditionally | Remove the line until `ctxTokens` is a real field |
| 10 | In-chat run error | Every run error is shown in chat | `ChatCore.tsx:79-80` sets `runRefusal = null` when `clientRefusal(code)` is `null` for an unknown code, so the in-chat card disappears | Show the raw code in the toast/alert even when copy is unknown; do not silently omit |

Items 1-6 are the P0 honesty set. Items 7-10 are incomplete/adjacent and should be
fixed with the same pass, not by inventing fields.

## 16. Implementation order

1. **P0-2 state vocabulary** (`agentState.ts` + `nodes.tsx`): one-line tone map,
   `StateDot`, `StatePill`. This unblocks every card's readability.
2. **P0-1 HyperAgent de-duplication** (`graph.ts` + `Canvas.tsx` + `nodes.tsx`):
   anchor, filter, minimap, focusability. Then `HiveCore` five lobes.
3. **P0-3/P0-4/P0-5/P0-6 honesty** (`ChatCore.tsx`, `strings.ts`, `HudBadge.tsx`,
   `nodes.tsx`, `Canvas.tsx`): false copy/controls removed, error fallback corrected,
   filtered empty state.
4. **P1-1/P1-2/P1-3 card anatomy** (`nodes.tsx`): shared `Frame` variants,
   `Progress`, exact type classes, worker count.
5. **P1-4 chat anatomy** (`ChatCore.tsx`): header, bubbles, composer, code fence.
6. **P1-5/P1-7 edges/layout** (`BeeEdge.tsx`, `graph.ts`): straight route, arrowhead,
   broken warning, TB layout + handles.
7. **P1-6 field/hive** (`Canvas.tsx`, `index.css`): pattern and lobes.
8. **P2** tokens, HUD exact width/type, minimap field/attribution, filters, transitions,
   snap preview.

Recommended verification after each P1 step:

- Screenshot at 1440x900 with the four working cards + chat; measure bounding boxes
  (420x150, 320x96, 420x240, 320x110, 420x500, 460x44, 220x150).
- Assert computed state dot colours: `rgb(232, 184, 80)`, `rgb(200, 128, 48)`,
  `rgb(178, 58, 46)`, `rgb(229, 225, 216)`.
- Assert chat header 44, pill 18, composer 56, bubble radius 10.
- Assert edge path has no `C` command (straight `L` only) and `strokeDasharray` is
  `10 10` normal / `6 6` broken.
- Run `npm.cmd test -- tests/designParity.test.ts` (currently 3 passed) and the full
  `npm.cmd test` after the source edits.

## 17. Evidence index

Reference:

- `C:\Users\Riley\Desktop\HiveRouter\design\hive-style\build_hive_style.py:815-902`
  (chat/super/worker/tool/error), `:522-535` (hive), `:318-342` (bee route),
  `:974-983` (anatomy), `:1146-1249` (board 07), `:1056-1064` (HUD).
- `C:\Users\Riley\Desktop\HiveRouter\design\hive-style\exports\hiverouter-agent-cards.png`
  (SHA-256 `1bbc81a4...`, byte-identical to the HiveMind export).
- `docs/ui/figcraft/ASSETS/hivemind-agent-cards.png` (1440x1300),
  `hivemind-spatial-canvas.png` (1440x900),
  `hivemind-controls-and-hud.png` (1440x1040).
- `docs/ui/figcraft/HiveMind/README.md:30-32`, `:46-48`.
- `docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json` surfaces S15-S19.
- `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md` §2 (core, snapping,
  bee direction), §3 (spawn hierarchy).

Current:

- `web/src/flow/nodes.tsx:15-39` (tokenColor/StateGlyph),
  `:41-68` (Frame), `:70-84` (Progress), `:86-118` (Super),
  `:120-139` (Worker), `:141-164` (Tool), `:166-183` (Error),
  `:185-207` (Hyper).
- `web/src/state/agentState.ts:30`, `:58-84`.
- `web/src/flow/graph.ts:26-32`, `:38-56`, `:62-74`, `:79-99`, `:101-114`,
  `:141-149`.
- `web/src/flow/Canvas.tsx:113-144` (filters), `:146-168` (nodes/centre),
  `:235-268` (field/hive), `:270-302` (filter pills), `:303-385` (ReactFlow/minimap),
  `:396-445` (zoom/empty).
- `web/src/flow/BeeEdge.tsx:13-58`.
- `web/src/chat/ChatCore.tsx:166-272`; `web/src/state/strings.ts:83`, `:275-294`.
- `web/src/panels/HudBadge.tsx:14-32`; `web/src/state/metrics.ts:37-47`.
- `web/src/styles/tokens.css:8-47`; `web/src/styles/index.css:81-85`, `:121-127`.
- `web/src/state/store.ts:1424` (label fallback), `:2051-2056` (pin).
- `backend/crates/hiverouter-core/src/execution.rs:151-162` (tier mapping),
  `:1322-1335` (root task), `:765-778` (NODE_SPAWN fields).
- `web/src/api/contract.ts:220-244` (no role/ctx fields).

Verification performed for this document:

- Static read of every file above at the snapshot hashes.
- `npm.cmd test -- tests/designParity.test.ts` -> 3 passed (the LAYOUT seam is
  currently fixed; the remaining gaps are visual/anatomical).
- Grep for `onDrop|dragover|DataTransfer|drop a repository` in `web/src` -> zero
  matches (false drop affordance confirmed).
- No live browser render or screenshot of the current build was taken in this
  analysis; pixel-level parity is therefore **UNVERIFIED** until the P1 changes land
  and are compared against the boards.

## 18. Definition of done for the rebuild

A card is only done when all of the following are observed, not inferred:

1. Rendered at 1440x900 with a live run: five card types plus the fixed chat/hive.
2. Bounding boxes match the reference within 1px.
3. State dot/pill computed colours match the four reference values; no card shows both
   a dot and a pill for state.
4. Role tags use the per-card type steps (9 / 9.5 / 8.5), not the global 11px meta.
5. Progress tracks are hairline, fills are gradient (super/tool) or solid honey-400
   (worker), heights 5/3/6.
6. Chat header 44, bubbles r10 at 13.5/20, composer 56, shell e2 with no border.
7. No false affordance from section 15 remains; empty states say only what the data
   supports.
8. The broken route is a straight dashed crimson vector with a halted bee/warning,
   and the error card no longer invents a diagnosis.
