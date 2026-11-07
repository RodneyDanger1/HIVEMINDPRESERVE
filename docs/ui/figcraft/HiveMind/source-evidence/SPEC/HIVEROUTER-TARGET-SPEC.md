# HiveRouter — Target Specification (verbatim source brief)

**Provenance.** This file transcribes the user's paste, supplied in-session 2026-11-05.
It is the single source of truth every seat must read before producing anything.
Conversational filler in the original paste ("I can provide the structural codebase
boilerplates…", "Try without personalization", chat timestamps) is **omitted and marked
here**; no requirement, prompt, constraint, or technical detail is omitted.

---

## §1 Framework overview

HiveRouter is a spatial execution environment where code routing is represented as a
dynamic, symmetrical honeycomb topology. Instead of abstract logs or rigid node graphs,
the user orchestrates agents visually on an infinite canvas, styled with a high-fidelity
Figma aesthetic blended with clean, structural bee and hive design elements.

```
+-------------------------------------------------------------------------+
| [Left Sidebar] | (Infinite Canvas Grid - Soft Honeycomb Symmetrical Vector Lines)
|                |                                                        |
|   [UI Tabs]    |                       +--------------+                 |
|   [Icons]      |                       |  HyperAgent  |                 |
|   [Templates]  |                       |  (Main Hive  |                 |
|                |                       |   Chat UI)   |                 |
|                |                       +-------+------+                 |
|                |                               |                        |
|                |             - - 🐝 - - - - - -+ - - - - - 🐝 - -       |
|                |            |                                    |      |
|                |    +-------v-------+                    +-------v----+ |
|                |    |  SuperAgent   |                    | SuperAgent | |
|                |    |  (Sub-Hive)   |                    | (Sub-Hive) | |
|                |    +-------+-------+                    +------------+ |
|                |            |                                           |
|                |    - - 🐝 -+ - - - -                                   |
|                |   |                 |                                  |
|                | +-v-----------+  +-v-----------+                       |
|                | | WorkerAgent |  | ToolAgent   |                       |
|                | +-------------+  +-------------+                       |
+-------------------------------------------------------------------------+
```

## §2 Core architectural layout & system mechanics

### The infinite spatial canvas
- **Background layer:** infinite, zoomable (0.1x to 3.0x), scrollable (pan X/Y) canvas.
  The background is **not** a traditional dot or square grid; it is a high-precision,
  soft honeycomb yellow vector line symmetry pattern.
- **The symmetrical core:** the coordinate origin (0,0) anchors the primary Hive Chat UI.
  No matter how far a user pans or zooms, a single keyboard shortcut (**Cmd/Ctrl + 0**)
  smoothly centers the viewport back onto the core Hive Chat UI.
- **The left sidebar:** a fixed, high-contrast structural element (Figma off-white /
  light-tan) spanning the full viewport height. Contains slim UI tabs, tool icons, a
  prompt-based agent template library, and a file/code hierarchy viewer.

### Visual hierarchies & aesthetics
- **HyperAgent core (Hive Chat UI):** a sleek, perfectly symmetrical beehive-shaped
  containment box. Surfaces rich beehive yellows, honey gold accents, and a stark white,
  clean minimalist text input box mimicking premium conversational interfaces.
- **Agent control cards:** rectangular canvas elements with light off-white bodies and
  honey-gold borders. They float above the honeycomb grid, casting subtle modern
  drop-shadows to signify depth.
- **The bee line routing vector:** wiring paths follow the honeycomb grid lines.
  Connections are animated via a tiny, perfectly symmetrical bee vector trail
  (`- - 🐝 - -`). The bee **faces the vector direction** of the data stream, flying
  away from the core hive toward its delegated worker nodes.

### Smart wire snapping & collision avoidance
- **Radius-gated snapping:** wires only latch onto a SuperAgent if the dragged
  WorkerAgent card enters its active connection radius (e.g. 150px).
- **Path prioritization:** in a crowded cluster where multiple SuperAgent paths overlap,
  the engine calculates the closest node *along the current structural vector path*
  rather than snapping to the nearest visual card.
- **Visual displacement preview:** when a card hovers over an occupied slot, adjacent
  agents subtly shift away along honeycomb lines, previewing the drop zone before release.

## §3 Execution & routing paradigm

```
[User Prompt / Code File]
           │
           ▼
   [HyperAgent Core] ───► Analyzes structural intent, breaks down dependencies
           │
           ├───► [SuperAgent A (e.g., Backend Architecture Engine)]
           │            │
           │            ├───► [WorkerAgent: DB Schema Compiler]
           │            └───► [ToolAgent: Prisma Generator]
           │
           └───► [SuperAgent B (e.g., UI/UX & Frontend Synthesis)]
                        │
                        └───► [WorkerAgent: Tailwind Component Builder]
```

1. **Ingestion:** user drops a code repository or types a complex functional prompt into
   the central Hive Chat UI.
2. **Structural decomposition:** the HyperAgent reads the code, maps its architecture,
   and outputs a visual deployment plan.
3. **Visual spawning:** as the HyperAgent allocates tasks, SuperAgents physically burst
   out from the core hive box on the canvas, instantly mapping their execution lines.
4. **Subagent branching:** SuperAgents spawn targeted WorkerAgents or Tool-Specific Agents
   underneath them to write code, test scripts, or refactor modules concurrently.
5. **Manual overrides:** the developer can interrupt at any time by dragging a WorkerAgent
   away from SuperAgent A and dropping it onto SuperAgent B. The active bee line instantly
   re-routes its trajectory, changing system prompts and code context streams in real time.

## §4 UI/UX generation prompts (the design source set, 1-10)

**Prompt 1 — Canvas and core Hive Chat UI layout.** Modern UI design system for an AI
development tool, infinite canvas workspace style, light off-white and light tan background
covered in a precise, soft honeycomb yellow symmetrical vector line grid. Centered in the
middle of the screen is a perfectly symmetrical beehive-shaped chat UI box styled with deep
beehive yellow, honey gold gradients, and a sharp, clean white input text area similar to
ChatGPT/Grok interfaces. On the far left, a fixed, narrow, full-height rectangular sidebar
in clean off-white containing minimalist UI tabs, sharp monochrome development icons, and
agent template options. Modern flat aesthetic, vector lines, highly scannable look, clean
tech layout.

**Prompt 2 — Agent node cards and interactive bee routing lines.** Figma-inspired UI
component design showing interactive floating canvas nodes and network connections. Soft
yellow honeycomb grid. Two type-styles of rectangular card nodes: a larger 'SuperAgent' card
with a light honey-gold header and a smaller 'WorkerAgent' code-generation card. Clear vector
paths aligned to the honeycomb shapes. On the path lines, a small, clean, symmetrical vector
bee with trailing dashed lines flows dynamically toward the target worker card, showing data
routing direction. Crisp typography, smooth subtle drop shadows, development workspace UI.

**Prompt 3 — Left sidebar UI & template library drawer.** Minimalist UI component design for
a full-height development tool sidebar, off-white and light-tan clean flat aesthetic. On the
left edge a narrow vertical strip of sharp monochrome tabs and utility icons. Expanding from
it a highly structural 'Agent Library' template browser drawer with clean card-style listings
of pre-configured agent templates (e.g. 'API Drone', 'Code Refactor Bee', 'SQL Swarm') with
small crisp sans-serif text tags and precise status indicators. Highly scannable, sharp
typography, tight padding, mirroring Figma or modern developer tools.

**Prompt 4 — Tool-specific agents & nested 'sub-hive' code cards.** Modern development
workspace UI featuring a granular, tool-specific AI agent card. Soft honeycomb yellow
symmetrical vector line grid. A smaller 'ToolAgent'/'Subagent' rectangular card pops open
right beneath its parent node, revealing a crisp code generation layout: a dark
syntax-highlighted code editor box (clean JavaScript/Python blocks), a compact white text
area for inline modifications, and a bright honey-gold execution progress bar. Modern flat
aesthetic, sharp typography, compact card design.

**Prompt 5 — Interactive proximity snapping & collision feedback.** Figma-inspired
representation of an interactive drag-and-drop action on an infinite canvas with a soft
honeycomb grid. A user drags a smaller 'WorkerAgent' card across a crowded cluster. As the
card passes within a proximity radius of an existing parent 'SuperAgent', a glowing
honey-gold target circle and translucent vector guidelines appear, indicating the active
wire-snapping threshold. Adjacent worker cards subtly slide away along honeycomb grid lines
to preview the drop zone. Flat clean aesthetic, subtle modern depth shadows.

**Prompt 6 — Global execution overlay & canvas navigation mini-map.** UI design for a
development tool's heads-up display overlaying an infinite canvas with a soft yellow
honeycomb pattern. Bottom-right: floating rectangular off-white navigation mini-map showing
a scaled-down vector silhouette of the entire active agent swarm. Top center: a sleek
honey-gold systemic alert badge indicating global code routing metrics and execution
throughput. Modern flat aesthetic, crisp sans-serif typography, scannable panel.

**Prompt 7 — System error, loop detection, & broken routing paths.** Modern developer UI
showing an agent error state on an infinite canvas with soft yellow honeycomb grid. A
rectangular 'WorkerAgent' card highlighted with a striking, sharp crimson-red border and
subtle alert tint; text blocks indicate a compilation failure. The connecting vector path
line changes from smooth honey-gold to a broken, dashed crimson-red line, and the small bee
vector is paused with an active warning icon indicator. Flat modern aesthetic, clean
scannable layouts, bold minimalist alert design.

**Prompt 8 — Split-screen code diff & iteration review modal.** Figma-inspired overlay window
on an infinite canvas. A clean, sharp off-white modal pops up over the honeycomb background
with a side-by-side code diff viewer. Left pane: original code routed to the agent. Right
pane: newly generated, optimized code block. Insertions highlighted in soft muted green,
deletions in soft red. Minimalist tabs at top switch between 'Source', 'Tests', and
'Execution Logs Log'. Highly structured text, compact layout, sharp typography.

**Prompt 9 — Macro-swarm view (high-level canvas zoom-out).** Developer dashboard showing a
macro-level view of a complex multi-agent network zoomed to 0.25x. Canvas filled with
hundreds of small geometric off-white rectangular agent nodes across a vast symmetrical
yellow honeycomb matrix. Dense webs of honey-gold connection paths form intricate structural
patterns, like a living digital hive circuit. Major 'SuperAgent' clusters glow slightly
brighter to signify high throughput. Clean tech layout, organized data visualization, flat design.

**Prompt 10 — Interactive time-travel debugging & history slider.** Modern development tool
timeline bar floating at bottom center of an infinite canvas. A sleek horizontal off-white
panel with a honey-gold scrubbable timeline slider with precise timestamp markers. User drags
the slider backward to inspect previous routing states; inactive agent lines on the background
honeycomb turn translucent gray while active historical paths light up. Minimalist playback
controls (Play, Pause, Step-Back) with clean monochrome icons. High scannability, modern flat
aesthetic.

## §5 Technical transition rules (three requirements)

**Requirement 1 — Decouple state from the viewport (spatial canvas engine).**
Core router state must be tracked strictly as a pure directional graph database structure
(Nodes and Edges), completely independent of coordinates. Implement a canvas rendering layer
(WebGL, HTML5 Canvas, or high-performance libraries such as pixi.js or React Flow) where each
Agent ID maps to an [X, Y] spatial coordinate. Moving a card updates coordinate state;
snapping onto a new cluster updates parent-child edges in the backend state engine.

**Requirement 2 — Token-efficient context isolation.**
Do not feed the entire application state into every agent; the HyperAgent retains the global
system blueprint. When an agent is dragged from SuperAgent A to SuperAgent B, trigger a
context-switch function: clear memory tokens inherited from SuperAgent A, inject the system
state and architectural variables of SuperAgent B, and re-verify code compatibility before
the next generation cycle.

**Requirement 3 — Asynchronous event streams for real-time wiring.**
Establish an Event Emitter layer or WebSocket pipeline handling three core canvas states:
`NODE_SPAWN`, `NODE_DRAG`, `NODE_REWIRE`. When an agent runs a multi-minute compilation, its
card must stream status logs asynchronously. The user must retain the ability to pan, zoom,
spin up alternative SuperAgents, or pre-configure downstream routing pipelines elsewhere on
the canvas while execution occurs in the background.

## §6 What the user is asking for in the current turn (nine delegation seats)

The user's own framing, condensed but not softened:

1. **Two watchdogs** that watch the app / the UI it builds and record errors, or points where
   it strays from (a) the UI brand schema entered, (b) the original UI image style,
   (c) the design rules.
2. **One mapper** for where HiveRouter's UI will go — screen/route placement.
3. **Three mappers** for codebase / architectural / script / logic changes, done **100%
   truthfully**, mapping what the pasted spec says must be updated about HiveRouter and *why*,
   analysed in parallel against the target the user wants HiveRouter to become:
   *a HyperAgent running a chat UI box in the hive centre / the entire application, which is
   also an AI agent harness that spawns and runs SuperAgents, who spawn and run Agents, who
   spawn and/or run their own subagents, and so forth.*
4. **Three analysts** deciding whether the Agents/framework would be better coded in **Rust**
   than Python or other languages — Rust being lightweight and fast, allowing many
   API-powered AI agents at once — and whether it could work with the target above powered by
   **one DeepSeek key** for everything, or **multiple keys** of the same model distributed to
   HyperAgent and the SuperAgents it spawns to delegate API usage efficiently, or whether Rust
   would not need that at all; and whether this is truly possible to reverse-engineer or
   refactor into Rust after designing it, or whether it should all be rewritten in Rust now,
   and what other languages/tooling it would have to use and why.

**Hard constraints on every seat** (from the user's standing rules and the repo AGENTS.md):
- Truthfulness over plausibility. No invented APIs, benchmarks, versions, prices, or limits.
- Every claim carries evidence: `path:line`, command + observed output, or a primary-source URL
  with retrieval date. Otherwise it is labelled `UNVERIFIED`.
- No product source edits, no installs, no scope drift into implementing the change.
- The repo stack lock is authoritative for *current* state: TypeScript, Node 26.7, Fastify 5,
  React 19, Vite 8, Tailwind 4, `node:sqlite`, direct DeepSeek Chat Completions. Any claim
  that the backend is currently Python is false unless proven from the repo.
