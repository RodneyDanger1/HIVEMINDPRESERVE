"""Patch 3: zoom / interaction / hero layout fixes after render review."""
import io
import os

HERE = os.path.dirname(os.path.abspath(__file__))


def patch(name, pairs):
    p = os.path.join(HERE, name)
    s = io.open(p, encoding="utf-8").read()
    for old, new in pairs:
        n = s.count(old)
        if n != 1:
            raise SystemExit("%s: pattern count %d for %r" % (name, n, old[:90]))
        s = s.replace(old, new)
    io.open(p, "w", encoding="utf-8", newline="\n").write(s)
    print("patched", name, len(pairs), "edits")


# ------------------------------------------------------------------ zoom board
patch("board_zoom.py", [
    # captions must clear the big zoom values
    ('b.meta(x + 112, 140, "OVERVIEW · LOD 1 · labels only, no card interiors",\n           width=300, size=9, name="zoom/p1/caption")',
     'b.meta(x + 150, 140, "OVERVIEW · LOD 1 · labels only, no card interiors",\n           width=262, size=8.5, name="zoom/p1/caption")'),
    ('b.meta(x + 112, 140, "TRUE SIZE · full cards, no token/meta magnification",\n           width=300, size=9, name="zoom/p2/caption")',
     'b.meta(x + 150, 140, "TRUE SIZE · full cards, no token/meta magnification",\n           width=262, size=8.5, name="zoom/p2/caption")'),
    ('b.meta(x + 112, 140, "DETAIL · 300% crop: tokens + meta become readable",\n           width=300, size=9, name="zoom/p3/caption")',
     'b.meta(x + 150, 140, "DETAIL · 300% crop: tokens + meta readable",\n           width=262, size=8.5, name="zoom/p3/caption")'),
    # 1.0x notes: three single lines
    ('''    b.meta(x + 24, 640, "100% · card anatomy readable: name 14.5 / role mono 9.5 "
                        "/ progress 5px track 3px worker", width=376, size=8.5,
           name="zoom/p2/note")
    b.meta(x + 24, 658, "radius 14 super / 12 worker · padding 16 · gap 8/16/24",
           width=376, size=8, name="zoom/p2/note2")
    b.meta(x + 24, 676, "bee edge: source->target, bee = routing arrow (R-U6)",
           width=376, size=8, name="zoom/p2/note3")''',
     '''    b.meta(x + 24, 634, "100% · card anatomy readable: name 14.5 · role mono 9.5",
           width=376, size=8.5, name="zoom/p2/note")
    b.meta(x + 24, 650, "progress 5px track / 3px worker · radius 14 / 12 · padding 16",
           width=376, size=8, name="zoom/p2/note2")
    b.meta(x + 24, 666, "gap 8/16/24 only · bee edge source->target (R-U6)",
           width=376, size=8, name="zoom/p2/note3")'''),
    # 3.0x detail: crop from the lower half of the WorkerAgent card (all 3x sizes)
    ('''    # worker card corner at 3x: radius 12 -> 36, padding 16 -> 48
    b.rect(x + 24, 220, 368, 300, fill=C["paper"], r=36, stroke=C["hairline"],
           sw=1, name="zoom/p3/card-3x")
    b.name_t(x + 72, 268, "tailwind-builder", 43.5, name="zoom/p3/name")
    b.text(x + 292, 272, "55%", 33, "Consolas", "Regular", C["ink700"],
           width=80, align="right", line=38, name="zoom/p3/pct")
    b.text(x + 72, 328, "WORKER · react/vite", 28.5, "Consolas", "Regular",
           C["ink500"], width=300, line=34, tracking=8, name="zoom/p3/role")
    b.rect(x + 72, 394, 272, 15, fill=C["recess"], r=999,
           name="zoom/p3/track")
    b.rect(x + 72, 397, 149, 9, fill=C["honey500"], r=999,
           name="zoom/p3/bar")
    b.text(x + 72, 424, "progress 0-100 from real metric", 25.5, "Consolas",
           "Regular", C["ink500"], width=272, line=30, tracking=2,
           name="zoom/p3/metric")
    b.pill(x + 16, 194, 82, 26, C["ink900"], "3.0x", C["paper"], size=10,
           name="zoom/p3/badge")
    b.meta(x + 24, 546, "radius 12 -> 36 px @3x · padding 16 -> 48 px",
           width=376, size=9, name="zoom/p3/callout1")
    b.meta(x + 24, 564, "meta mono 9.5 -> 28.5 px · track 5 -> 15 px · "
                        "worker bar 3 -> 9 px", width=376, size=9,
           name="zoom/p3/callout2")
    b.rect(x + 24, 590, 368, 1, fill=C["hairline"], name="zoom/p3/rule")
    b.meta(x + 24, 600, "at 3.0x the LOD is maximum: names, roles, state, "
                        "progress and meta are all rendered", width=376,
           size=8.5, name="zoom/p3/note")
    b.meta(x + 24, 618, "panning at 3.0x still keeps the core hive pinned "
                        "(S02)", width=376, size=8, name="zoom/p3/note2")
    b.meta(x + 24, 636, "Ctrl+0 animates back to 1.0x on the core hive", width=376,
           size=8, name="zoom/p3/note3")''',
     '''    # 3.0x crop of the WorkerAgent card (lower half: role, progress, meta)
    b.rect(x + 16, 220, 384, 356, fill=C["paper"], r=8, stroke=C["hairline"],
           sw=1, name="zoom/p3/crop-3x")
    b.rect(x + 16, 220, 384, 1, fill=C["honey400"], name="zoom/p3/crop-rule")
    b.text(x + 48, 240, "WORKER · react/vite", 28.5, "Consolas", "Regular",
           C["ink500"], width=300, line=34, tracking=8, name="zoom/p3/role")
    b.text(x + 292, 240, "55%", 33, "Consolas", "Regular", C["ink700"],
           width=84, align="right", line=38, name="zoom/p3/pct")
    b.rect(x + 48, 322, 288, 15, fill=C["recess"], r=999,
           name="zoom/p3/track")
    b.rect(x + 48, 325, 149, 9, fill=C["honey500"], r=999,
           name="zoom/p3/bar")
    b.text(x + 48, 358, "progress 0-100 from real metric", 25.5, "Consolas",
           "Regular", C["ink500"], width=288, line=30, tracking=2,
           name="zoom/p3/metric")
    b.rect(x + 48, 430, 288, 1.5, fill=C["hairline"], name="zoom/p3/rule")
    b.text(x + 48, 448, "tailwind-builder", 43.5, "Inter", "SemiBold",
           C["ink900"], width=330, line=48, name="zoom/p3/name")
    b.pill(x + 16, 194, 82, 26, C["ink900"], "3.0x", C["paper"], size=10,
           name="zoom/p3/badge")
    b.meta(x + 24, 600, "crop rule: this is the lower half of the WorkerAgent "
                        "card at 300%", width=376, size=8.5,
           name="zoom/p3/note")
    b.meta(x + 24, 616, "role mono 9.5 -> 28.5 px · track 5 -> 15 px · worker "
                        "bar 3 -> 9 px · name 14.5 -> 43.5 px", width=376, size=8,
           name="zoom/p3/note2")
    b.meta(x + 24, 632, "radius 12 -> 36 px · padding 16 -> 48 px (PDF p6:218-226)",
           width=376, size=8, name="zoom/p3/note3")
    b.meta(x + 24, 648, "panning at 3.0x keeps the core hive pinned (S02)",
           width=376, size=8, name="zoom/p3/note4")
    b.meta(x + 24, 664, "Ctrl+0 animates back to 1.0x on the core hive", width=376,
           size=8, name="zoom/p3/note5")'''),
    # right column: two clean notes instead of three overlapping ones
    ('''    b.meta(1160, 772, "third frame is a real 3.0x crop, not a scaled-up bitmap "
                      "(vector text)", width=232, align="right", size=8,
           name="zoom/vector-note")
    b.meta(1160, 796, "PDF p8:303-305 · 0.1x - 3.0x · Ctrl+0 recentres the core "
                      "hive", width=232, align="right", size=8,
           name="zoom/pdf-note")
    b.meta(1160, 820, "S20 · Zoom controls + shortcut hint", width=232,
           align="right", size=8.5, name="zoom/surface-note")''',
     '''    b.meta(1160, 766, "third frame is a real 3.0x vector crop, not a scaled "
                      "bitmap", width=232, align="right", size=8,
           name="zoom/vector-note")
    b.meta(1160, 782, "S20 · Zoom controls + shortcut hint", width=232,
           align="right", size=8.5, name="zoom/surface-note")'''),
])

# ------------------------------------------------------------ interaction board
patch("board_interaction.py", [
    ('b.eyebrow(1160, 26, "1440 x 900 · 9 STATE PANELS + DIFF MODAL", width=232,\n              align="right", name="ix/right-eyebrow")',
     'b.eyebrow(1160, 26, "1440 x 900 · 9 PANELS + DIFF", width=232,\n              align="right", size=8, name="ix/right-eyebrow")'),
    # selected cell notes: one line each
    ('''    b.meta(x + 16, y + 164, "ring ink-700 #4E3A15 = 9.86:1 >= 3:1 · PDF honey-300 "
                            "1.70:1 FAILED 3:1 (R-U3 correction)", width=384,
           size=8, name="s02/focus-note")
    b.meta(x + 16, y + 178, "selected node keeps hover/pan affordances; focus is "
                            "never colour-only", width=384, size=8,
           name="s02/selected-note")''',
     '''    b.meta(x + 16, y + 164, "ring ink-700 #4E3A15 = 9.86:1 >= 3:1 · PDF honey-300 "
                            "1.70 FAILS (R-U3)", width=384, size=8,
           name="s02/focus-note")
    b.meta(x + 16, y + 178, "selected keeps hover/pan; focus is never colour-only",
           width=384, size=8, name="s02/selected-note")'''),
    # drag ghost must read as a card, not a white void
    ('    b.rect(x + 62, y + 48, 176, 78, fill=C["paper"], r=12, stroke=C["honey400"],\n           sw=1.5, opacity=0.45, name="s16/drag-ghost")',
     '    b.rect(x + 62, y + 48, 176, 78, fill=C["honey100"], r=12, stroke=C["honey400"],\n           sw=1.5, opacity=0.62, name="s16/drag-ghost")'),
    # replay badge below the scrubber (was covering the revision label)
    ('''    b.pill(x + 258, y + 38, 142, 24, C["ink700"], "replay · read-only", C["paper"],
           size=9, name="s10/replay-badge")''',
     '''    b.pill(x + 16, y + 118, 142, 22, C["ink700"], "replay · read-only",
           C["paper"], size=9, name="s10/replay-badge")
    b.meta(x + 170, y + 122, "read-only view state · no mutation while scrubbing",
           width=230, size=8, name="s10/replay-badge-note")'''),
    # kill body: one line
    ('b.body(x + 52, y + 90, "tailwind-builder terminates with its children; a "\n                           "ledger row is written", 312, size=9.5, line=13,\n           name="s07/kill-body")',
     'b.body(x + 52, y + 90, "tailwind-builder + children terminate; ledger row written",\n           312, size=9.5, line=13, name="s07/kill-body")'),
    # refusal notes: one line each
    ('b.meta(x + 16, y + 164, "toast + log + run ledger all carry code / "\n                            "message / how_to_fix (R-U9)", width=384, size=8,\n           name="s14/toast/note")',
     'b.meta(x + 16, y + 164, "toast + log + ledger carry code / message / how_to_fix "\n                            "(R-U9)", width=384, size=8,\n           name="s14/toast/note")'),
    ('b.meta(x + 16, y + 178, "worker still streaming is the blocker, not the "\n                            "target parent", width=384, size=8,\n           name="s14/toast/note2")',
     'b.meta(x + 16, y + 178, "the blocker is the streaming worker, not the target "\n                            "parent", width=384, size=8,\n           name="s14/toast/note2")'),
    # legend: long state names in the first column so nothing clips at the edge
    ('''    order = ["healthy", "running", "queued", "failed", "completed",
             "cancelled", "killed", "planning", "thinking", "awaiting_approval",
             "streaming", "reconnecting", "replay", "empty", "loading"]''',
     '''    order = ["healthy", "running", "queued", "failed", "completed",
             "cancelled", "killed", "planning", "thinking", "streaming",
             "awaiting_approval", "reconnecting", "replay", "empty", "loading"]'''),
])

# ------------------------------------------------------------------ hero board
patch("board_hero.py", [
    # sub-hive + worker drop lower, freeing a readable bee edge from the core
    ('    b.rect(700, 520, 296, 132, fill=C["paper"], r=14,\n           name="hero/subhive-bg", effect="e1")',
     '    b.rect(700, 556, 296, 132, fill=C["paper"], r=14,\n           name="hero/subhive-bg", effect="e1")'),
    ('    b.rect(700, 520, 296, 38, fill=C["honey100"], r=14,\n           name="hero/subhive-header")',
     '    b.rect(700, 556, 296, 38, fill=C["honey100"], r=14,\n           name="hero/subhive-header")'),
    ('    b.rect(700, 546, 296, 12, fill=C["honey100"],\n           name="hero/subhive-header2")',
     '    b.rect(700, 582, 296, 12, fill=C["honey100"],\n           name="hero/subhive-header2")'),
    ('    b.name_t(716, 530, "Backend Architecture", 14.5,\n             name="hero/subhive-name")',
     '    b.name_t(716, 566, "Backend Architecture", 14.5,\n             name="hero/subhive-name")'),
    ('    b.pill(900, 528, 84, 22, C["paper"], "RUNNING", C["ink800"], size=8.5,\n           stroke=C["honey500"], sw=1, name="hero/subhive-state")',
     '    b.pill(900, 564, 84, 22, C["paper"], "RUNNING", C["ink800"], size=8.5,\n           stroke=C["honey500"], sw=1, name="hero/subhive-state")'),
    ('    b.eyebrow(716, 556, "SUPERAGENT", size=9, color=C["ink500"],\n              name="hero/subhive-role")',
     '    b.eyebrow(716, 592, "SUPERAGENT", size=9, color=C["ink500"],\n              name="hero/subhive-role")'),
    ('    b.body(716, 572, "decomposes intent, owns the sub-hive", 264, size=10.5,\n           line=15, name="hero/subhive-body")',
     '    b.body(716, 608, "decomposes intent, owns the sub-hive", 264, size=10.5,\n           line=15, name="hero/subhive-body")'),
    ('    b.meta(716, 610, "3 workers  ·  12.4k ctx", size=9.5,\n           name="hero/subhive-meta")',
     '    b.meta(716, 646, "3 workers  ·  12.4k ctx", size=9.5,\n           name="hero/subhive-meta")'),
    ('    b.rect(700, 520, 296, 132, fill=None, r=14, stroke=C["honey400"], sw=1.5,\n           name="hero/subhive-border")',
     '    b.rect(700, 556, 296, 132, fill=None, r=14, stroke=C["honey400"], sw=1.5,\n           name="hero/subhive-border")'),
    ('    b.rect(1020, 520, 220, 116, fill=C["paper"], r=12, stroke=C["hairline"],\n           sw=1, name="hero/worker-card", effect="e1")',
     '    b.rect(1020, 556, 220, 116, fill=C["paper"], r=12, stroke=C["hairline"],\n           sw=1, name="hero/worker-card", effect="e1")'),
    ('    b.name_t(1036, 534, "tailwind-builder", 14.5, name="hero/worker-name")',
     '    b.name_t(1036, 570, "tailwind-builder", 14.5, name="hero/worker-name")'),
    ('    b.text(1168, 536, "62%", 11, "Consolas", "Regular", C["ink700"], width=56,\n           align="right", line=14, name="hero/worker-pct")',
     '    b.text(1168, 572, "62%", 11, "Consolas", "Regular", C["ink700"], width=56,\n           align="right", line=14, name="hero/worker-pct")'),
    ('    b.eyebrow(1036, 554, "WORKER · react/vite", size=9.5, color=C["ink500"],\n              name="hero/worker-role")',
     '    b.eyebrow(1036, 590, "WORKER · react/vite", size=9.5, color=C["ink500"],\n              name="hero/worker-role")'),
    ('    b.rect(1036, 580, 188, 5, fill=C["recess"], r=999,\n           name="hero/worker-track")',
     '    b.rect(1036, 616, 188, 5, fill=C["recess"], r=999,\n           name="hero/worker-track")'),
    ('    b.rect(1036, 581, 117, 3, fill=C["honey500"], r=999,\n           name="hero/worker-bar")',
     '    b.rect(1036, 617, 117, 3, fill=C["honey500"], r=999,\n           name="hero/worker-bar")'),
    ('    b.meta(1036, 594, "running · 62% (PDF cover p1:12-14)", size=8,\n           name="hero/worker-meta")',
     '    b.meta(1036, 630, "running · 62% (PDF cover p1:12-14)", size=8,\n           name="hero/worker-meta")'),
    ('''    b.bee_edge((988, 600), (1020, 590), ctrl=(1006, 602), bee_t=0.5,
               bee_size=20, name="hero/edge/worker", label="NODE_SPAWN")
    b.bee_edge((990, 560), (962, 540), ctrl=(980, 546), bee_t=0.5,
               bee_size=16, name="hero/edge/core")''',
     '''    b.bee_edge((924, 490), (866, 546), ctrl=(900, 528), bee_t=0.55,
               bee_size=20, name="hero/edge/core-subhive")
    b.meta(946, 508, "NODE_SPAWN", size=8, name="hero/edge/label")'''),
])
print("done")
