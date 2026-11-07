# FigCraft command surface — what an AI agent can actually drive

Measured, not assumed: `figcraft-cli.exe commands --json` returned **597 commands**
(164,981 bytes) from the installed `C:\Program Files\FigCraft\figcraft-cli.exe`.

## 1. Drive modes (the whole surface)

| Mode | Invocation | Semantics | Used in this job |
|---|---|---|---|
| Script run | `figcraft-cli run <script.json\|.jsonl> [--in f] [--out f]` | Executes an ordered array of `{"command": id, "params": {…}}` **transactionally**; prints one JSON result line per step; **the whole script rolls back on the first rejected parameter** | ✅ primary driver (1,036 commands across 2 phases) |
| MCP server | `figcraft-cli mcp [--attach PORT]` | Exposes the same command set to an MCP client. `--attach` requires the GUI to have been launched with `figcraft --control PORT` | ❌ not available — the open GUI instance was launched without `--control` |
| One-shot | `info`, `export`, `render`, `convert`, `perf` | Read/inspect/rasterise a file without a session | ✅ verification (`info --json`, renders) |
| Distribution | `plugin publish <folder> --server ws://…` | Ships a plugin folder to a figcraft-server | ❌ out of scope |
| Confinement | `--automation-read-root` / `--automation-write-root` | Denies reads/writes outside the named roots; no write root = no writes | ❌ not needed (full-access policy) |

**Honest limitation:** the user's open FigCraft window could not be attached to, because the
running process (`figcraft.exe "…CODEBASE-MASTER.mmd"`) was **not** started with `--control PORT`,
and `mcp --attach` refuses to confine against it. The deterministic CLI path drove the same
engine headlessly instead; the produced `.figcraft` opens in that GUI.

## 2. Command distribution (61 namespaces, 597 commands)

| Namespace | n | Namespace | n | Namespace | n |
|---|---:|---|---:|---|---:|
| text | 57 | image | 5 | libraries | 2 |
| view | 48 | doc | 5 | window | 2 |
| vector | 42 | guide | 5 | app | 2 |
| tool | 29 | layout | 4 | plugins | 2 |
| object | 28 | effect | 4 | distribute | 2 |
| file | 22 | devResource | 4 | measure | 2 |
| component | 21 | codeConnect | 4 | template | 1 |
| select | 21 | shapeBuilder | 4 | paint | 1 |
| collab | 21 | dev | 4 | data | 1 |
| proto | 21 | help | 4 | layers | 1 |
| variable | 18 | grid | 3 | section | 1 |
| library | 18 | devMode | 3 | command | 1 |
| edit | 16 | widget | 3 | | |
| comment | 14 | export | 3 | | |
| pref | 13 | frame | 3 | | |
| plugin | 11 | shape | 6 | | |
| arrange | 10 | stroke | 6 | | |
| style | 10 | selection | 6 | | |
| board | 9 | align | 6 | | |
| transform | 9 | branch | 7 | | |
| page | 8 | instance | 8 | | |
| version | 8 | inspect | 5 | | |
| annotation | 8 | node | 5 | | |
| | | bool | 5 | | |
| | | fill | 5 | | |

## 3. Verified capabilities relevant to an agent-driven design system

- **Native design-system objects:** `variable.createCollection` / `variable.create` (`VariableID:n`),
  `style.create` for `paint | text | effect` (`S:n`), `variable.list` / `style.list`. These ids
  live in a **different id space** from layer ids, so they cannot be pre-computed the way layer
  ids can.
- **Deterministic layer ids:** page = 1, then **+1 per created layer** in creation order. This let
  the builder predict all 960 node ids and diff prediction against the CLI's returned ids.
- **Paints:** `fills` accepts `{"type":"linearGradient|radialGradient|angularGradient|diamondGradient","stops":[…]}`;
  `from`/`to` are **tuples** `[x,y]`, not objects. Singular `fill` accepts a colour only.
- **Effects:** `dropShadow` takes `offset` as a **tuple** `[x,y]` plus `radius`, `spread`, `color`,
  `blend`, `showBehind`, `visible`.
- **Geometry:** `node.create type="vector"` takes SVG `path` data; a whole honeycomb lattice is one
  node (one path with many subpaths) — cheap and exact.
- **Transactional safety:** a single bad parameter aborts the entire script and rolls back the
  document. This is why the build was split into phase 1 (nodes) and phase 2 (tokens + exports).

## 4. What the analysis cost, and what it bought

- 597 commands analysed by namespace; ~40 command schemas read in full before use.
- Two rejected syntax guesses were resolved empirically (`fills` vs `fill`; tuple `offset`).
- The build itself: **1,036 commands**, 0 failures, two phases, ~25 s total, producing 8 artboards,
  36 styles, 21 colour variables and 17 exported files.
