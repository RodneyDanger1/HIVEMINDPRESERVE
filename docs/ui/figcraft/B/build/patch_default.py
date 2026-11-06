"""Patch board_default.py layout fixes after the first render review."""
import io
import os

P = os.path.join(os.path.dirname(os.path.abspath(__file__)), "board_default.py")
s = io.open(P, encoding="utf-8").read()

REPL = []

# 1. tab note must not run under the HUD
REPL.append((
 'b.meta(360, 57, "S08 · tabs = view route state · CF-UI-04 tabs-vs-rail OPEN",\n           width=360, size=8, name="s08/tabs-note")',
 'b.meta(360, 57, "S08 · tabs = view route state · CF-UI-04 OPEN",\n           width=276, size=8, name="s08/tabs-note")'))

# 2. HUD shifted right so it clears the History tab
REPL.append((
 '''    b.rect(620, 20, 400, 40, fill=C["paper"], r=999, stroke=C["hairline"],
           sw=1, name="s09/hud", effect="e2")
    b.text(644, 31, HUD[0], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/superagents")
    b.text(748, 31, HUD[1], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/workers")
    b.dot(848, 34, 7, "circle", C["verify"], name="s09/hud/green-dot")
    b.text(860, 31, HUD[2], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/green")
    b.pill(952, 26, 52, 28, C["honey200"], HUD[3], C["ink800"], size=10,
           name="s09/hud/delta")
    b.meta(0, 66, "RUN_METRICS · derived aggregate, never hardcoded (R-U13)",
           width=1440, align="center", size=8, name="s09/hud/note")''',
 '''    b.rect(690, 20, 400, 40, fill=C["paper"], r=999, stroke=C["hairline"],
           sw=1, name="s09/hud", effect="e2")
    b.text(714, 31, HUD[0], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/superagents")
    b.text(818, 31, HUD[1], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/workers")
    b.dot(918, 34, 7, "circle", C["verify"], name="s09/hud/green-dot")
    b.text(930, 31, HUD[2], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/green")
    b.pill(1022, 26, 52, 28, C["honey200"], HUD[3], C["ink800"], size=10,
           name="s09/hud/delta")
    b.meta(690, 66, "RUN_METRICS · derived aggregate, never hardcoded (R-U13)",
           width=400, align="center", size=8, name="s09/hud/note")'''))

# 3. longer, bigger bee edges; crimson wire moved after the cards (z-order)
REPL.append((
 '''    # ------------------------------------------------------------ bee edges
    # core (parent) -> sub-hive (child): bee flies source->target
    b.bee_edge((742, 392), (668, 296), ctrl=(700, 336), bee_t=0.60,
               bee_size=18, name="edge/core-subhive", label="NODE_SPAWN")
    # sub-hive -> worker
    b.bee_edge((520, 342), (512, 436), ctrl=(530, 392), bee_t=0.58,
               bee_size=16, name="edge/subhive-worker")
    # worker -> tool: the tool card pops open beneath its parent (attached)
    # core -> broken route: dashed crimson wire, halted bee + warning glyph
    b.vector("M 1078 424 Q 1120 404 1160 388", 0, 0, stroke=C["signal"],
             sw=1.5, name="edge/core-broken/line")
    b.cmd[-1]["params"]["props"]["dashes"] = [6, 4]
    b.bee(1122, 402, 17, -22, name="edge/core-broken/bee", body=C["signal"],
          wing=C["honey200"], stripe=C["paper"])
    b.poly(1112, 382, 14, 14, C["signal"], pointCount=3,
           name="edge/core-broken/warn")
    b.text(1115, 386, "!", 9, "Inter", "SemiBold", C["paper"], width=8,
           align="center", line=12, name="edge/core-broken/warn-mark")
    b.meta(1090, 428, "E0421 · halted bee", width=90, align="center",
           color=C["signal"], size=8.5, name="edge/core-broken/note")''',
 '''    # ------------------------------------------------------------ bee edges
    # core (parent) -> sub-hive (child): bee flies source->target
    b.bee_edge((742, 430), (668, 300), ctrl=(696, 356), bee_t=0.55,
               bee_size=22, name="edge/core-subhive", label="NODE_SPAWN")
    # sub-hive -> worker
    b.bee_edge((520, 342), (514, 436), ctrl=(534, 392), bee_t=0.50,
               bee_size=20, name="edge/subhive-worker", label="NODE_SPAWN")
    # worker -> tool: the tool card pops open beneath its parent (attached)'''))

# 4. crimson broken-route wire drawn after the cards so it stays visible
REPL.append((
 '''    b.meta(1160, 488, "S18 · attempt N/M sourced from the run ledger",
           width=250, size=8, name="s18/broken-note")''',
 '''    b.meta(1160, 488, "S18 · attempt N/M sourced from the run ledger",
           width=250, size=8, name="s18/broken-note")
    # core -> broken route: dashed crimson wire, halted bee + warning glyph
    b.vector("M 1078 462 Q 1124 438 1160 406", 0, 0, stroke=C["signal"],
             sw=1.5, name="edge/core-broken/line")
    b.cmd[-1]["params"]["props"]["dashes"] = [6, 4]
    b.bee(1124, 434, 19, -32, name="edge/core-broken/bee", body=C["signal"],
          wing=C["honey200"], stripe=C["paper"])
    b.poly(1114, 408, 15, 15, C["signal"], pointCount=3,
           name="edge/core-broken/warn")
    b.text(1117, 412, "!", 9.5, "Inter", "SemiBold", C["paper"], width=9,
           align="center", line=12, name="edge/core-broken/warn-mark")
    b.meta(1096, 446, "halted bee · E0421", width=110, align="center",
           color=C["signal"], size=8.5, name="edge/core-broken/note")'''))

# 5. worker percentage must not collide with the name
REPL.append((
 '''    b.text(536, 452, WORKER[3], 11, "Consolas", "Regular", C["ink700"],
           width=72, align="right", line=14, name="s16/worker-pct")''',
 '''    b.text(548, 452, WORKER[3], 11, "Consolas", "Regular", C["ink700"],
           width=60, align="right", line=14, name="s16/worker-pct")'''))

# 6. drop notes that collide with the button bar; keep the rule inside cards
REPL.append((
 '''    b.meta(404, 558, "S16 · progress is the only motion on the card",
           width=220, size=8, name="s16/worker-note")''', ''))
REPL.append((
 '''    b.meta(404, 738, "S17 · score shown only when measured", width=260,
           size=8, name="s17/tool-note")''',
 '''    b.meta(420, 720, "score shown only when measured; else —", width=228,
           size=8, name="s17/tool-note")'''))

# 7. agent state dot must not sit under the name text
REPL.append(("    b.dot(206, 124, 8, \"circle\", C[\"verify\"], name=\"s19/agent-chip-dot\")",
             "    b.dot(296, 124, 8, \"circle\", C[\"verify\"], name=\"s19/agent-chip-dot\")"))

# 8. PDF prints the multiplication sign
REPL.append(('    b.text(1046, 296, "x", 14, "Inter", "SemiBold", C["ink500"], width=20,',
             '    b.text(1046, 296, "×", 14, "Inter", "SemiBold", C["ink500"], width=20,'))

# 9. R-U6 rule note in the free right margin
REPL.append((
 '''    b.meta(1160, 700, "VIEWPORT_SET · E_ZOOM_RANGE outside 0.1..3.0", width=260,
           size=8, name="s20/zoom-note")''',
 '''    b.meta(1160, 700, "VIEWPORT_SET · E_ZOOM_RANGE outside 0.1..3.0", width=260,
           size=8, name="s20/zoom-note")
    b.meta(1160, 640, "R-U6 · the bee is a routing arrow: it flies source->target",
           width=260, size=8, name="s01/bee-rule")'''))

for old, new in REPL:
    n = s.count(old)
    if n != 1:
        raise SystemExit("pattern count %d for %r" % (n, old[:70]))
    s = s.replace(old, new)

io.open(P, "w", encoding="utf-8", newline="\n").write(s)
print("patched board_default.py:", len(REPL), "edits")
