"""Patch 2: z-order of the beehive containment vs bee edges, label collisions."""
import io
import os

P = os.path.join(os.path.dirname(os.path.abspath(__file__)), "board_default.py")
s = io.open(P, encoding="utf-8").read()


def rep(old, new):
    global s
    if s.count(old) != 1:
        raise SystemExit("pattern count %d for %r" % (s.count(old), old[:80]))
    s = s.replace(old, new)


# 1. beehive containment moves BEFORE the bee edges so the bees sit on the hive
rep('''    # ------------------------------------------------------------ bee edges''',
    '''    # ------------------------------------------------- hive containment (e3)
    b.vector(hex_subpath(910, 456, 262), 0, 0, fill=C["honey100"],
             stroke=C["honey300"], sw=1.5, name="e3/hive-containment",
             effect="e3")
    b.vector(hex_subpath(910, 456, 206), 0, 0, fill=None,
             stroke=C["honey300"], sw=1.0, name="e3/hive-containment-inner")

    # ------------------------------------------------------------ bee edges''')

rep('''    # ------------------------------------------------- hive chat core (S02)
    b.vector(hex_subpath(910, 456, 262), 0, 0, fill=C["honey100"],
             stroke=C["honey300"], sw=1.5, name="e3/hive-containment",
             effect="e3")
    b.vector(hex_subpath(910, 456, 206), 0, 0, fill=None,
             stroke=C["honey300"], sw=1.0, name="e3/hive-containment-inner")
    b.rect(742, 286, 336, 330''',
    '''    # ------------------------------------------------- hive chat core (S02)
    b.rect(742, 286, 336, 330''')

# 2. crimson wire label clear of the broken card
rep('''    b.meta(1096, 446, "halted bee · E0421", width=110, align="center",
           color=C["signal"], size=8.5, name="edge/core-broken/note")''',
    '''    b.meta(1066, 458, "halted bee · E0421", width=110, align="center",
           color=C["signal"], size=8.5, name="edge/core-broken/note")''')

# 3. tool card: taller card, meta/note no longer overlap
rep('''    b.rect(404, 560, 260, 170, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, name="s17/tool-card", effect="e1")''',
    '''    b.rect(404, 560, 260, 176, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, name="s17/tool-card", effect="e1")''')
rep('''    b.meta(420, 708, "schema.prisma  ·  0.78", size=9.5, color=C["ink500"],
           name="s17/tool-meta")''',
    '''    b.meta(420, 704, "schema.prisma  ·  0.78", size=9.5, color=C["ink500"],
           name="s17/tool-meta")''')
rep('''    b.meta(420, 720, "score shown only when measured; else —", width=228,
           size=8, name="s17/tool-note")''',
    '''    b.meta(420, 720, "score only when measured; else —", width=228,
           size=8, name="s17/tool-note")''')

# 4. shorten the rule note so it does not wrap
rep('''    b.meta(1160, 640, "R-U6 · the bee is a routing arrow: it flies source->target",
           width=260, size=8, name="s01/bee-rule")''',
    '''    b.meta(1160, 640, "R-U6 · bee = routing arrow · source->target",
           width=260, size=8, name="s01/bee-rule")''')

io.open(P, "w", encoding="utf-8", newline="\n").write(s)
print("patched board_default.py (patch 2)")
