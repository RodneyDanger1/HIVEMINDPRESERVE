"""Patch 4: final text-exactness and layout fixes (render review 2)."""
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


# ----------------------------------------------------------------- default board
patch("board_default.py", [
    # HUD carries the printed separators so the whole printed string is present
    ('''    b.text(818, 31, HUD[1], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/workers")
    b.dot(918, 34, 7, "circle", C["verify"], name="s09/hud/green-dot")
    b.text(930, 31, HUD[2], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/green")''',
     '''    b.text(802, 31, "·", 11, "Inter", "SemiBold", C["ink500"], width=12,
           align="center", line=15, name="s09/hud/sep1")
    b.text(818, 31, HUD[1], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/workers")
    b.text(894, 31, "·", 11, "Inter", "SemiBold", C["ink500"], width=12,
           align="center", line=15, name="s09/hud/sep2")
    b.dot(918, 34, 7, "circle", C["verify"], name="s09/hud/green-dot")
    b.text(930, 31, HUD[2], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/green")'''),
    # scrubber carries the printed separator between time and revision
    ('    b.meta(900, 806, SCRUB[1], width=168, align="right", size=9,\n           name="s10/state-label")',
     '    b.meta(900, 806, "· " + SCRUB[1], width=168, align="right", size=9,\n           name="s10/state-label")'),
    # halted-bee label must fit the 82px gap beside the chat core
    ('''    b.meta(1066, 458, "halted bee · E0421", width=110, align="center",
           color=C["signal"], size=8.5, name="edge/core-broken/note")''',
     '''    b.meta(1080, 452, "halted bee", width=76, align="center",
           color=C["signal"], size=8, name="edge/core-broken/note")
    b.meta(1080, 464, "E0421", width=76, align="center",
           color=C["signal"], size=8, name="edge/core-broken/note2")'''),
])

# ------------------------------------------------------------------- zoom board
patch("board_zoom.py", [
    ('    b.eyebrow(1160, 30, "1440 x 900 · 3 FRAMES · PDF p8:303-305", width=232,\n              align="right", name="zoom/right-eyebrow")',
     '    b.eyebrow(1160, 30, "3 FRAMES · PDF p8:303-305", width=232,\n              align="right", name="zoom/right-eyebrow")'),
    ('    b.meta(1160, 48, "S20 · VIEWPORT_SET / VIEWPORT_RECENTER", width=232,\n           align="right", size=8.5, name="zoom/right-note")',
     '    b.meta(1160, 48, "1440 x 900 · S20 · VIEWPORT_SET", width=232,\n           align="right", size=8.5, name="zoom/right-note")'),
    # 3.0x crop: rebuild so every 3x size lands on one line
    ('''    b.rect(x + 16, 220, 384, 356, fill=C["paper"], r=8, stroke=C["hairline"],
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
           C["ink900"], width=330, line=48, name="zoom/p3/name")''',
     '''    b.rect(x + 8, 220, 400, 356, fill=C["paper"], r=8, stroke=C["hairline"],
           sw=1, name="zoom/p3/crop-3x")
    b.rect(x + 8, 220, 400, 1, fill=C["honey400"], name="zoom/p3/crop-rule")
    b.text(x + 40, 238, "WORKER · react/vite", 28.5, "Consolas", "Regular",
           C["ink500"], width=368, line=34, tracking=8, name="zoom/p3/role")
    b.text(x + 40, 288, "55%", 33, "Consolas", "Regular", C["ink700"],
           width=140, line=38, name="zoom/p3/pct")
    b.rect(x + 40, 344, 320, 15, fill=C["recess"], r=999,
           name="zoom/p3/track")
    b.rect(x + 40, 347, 176, 9, fill=C["honey500"], r=999,
           name="zoom/p3/bar")
    b.text(x + 40, 382, "progress 0-100 from real metric", 25.5, "Consolas",
           "Regular", C["ink500"], width=320, line=30, tracking=2,
           name="zoom/p3/metric")
    b.rect(x + 40, 466, 320, 1.5, fill=C["hairline"], name="zoom/p3/rule")
    b.text(x + 40, 482, "tailwind-builder", 43.5, "Inter", "SemiBold",
           C["ink900"], width=368, line=48, name="zoom/p3/name")'''),
    ('''    b.meta(x + 24, 600, "crop rule: this is the lower half of the WorkerAgent "
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
           size=8, name="zoom/p3/note5")''',
     '''    b.meta(x + 24, 588, "crop rule: lower half of the WorkerAgent card at 300%",
           width=376, size=8.5, name="zoom/p3/note")
    b.meta(x + 24, 604, "role 9.5 -> 28.5 · pct 11 -> 33 · track 5 -> 15 · bar 3 -> 9",
           width=376, size=8, name="zoom/p3/note2")
    b.meta(x + 24, 620, "metric mono 9.5 -> 28.5 px at 3.0x", width=376, size=8,
           name="zoom/p3/note2b")
    b.meta(x + 24, 636, "name 14.5 -> 43.5 · radius 12 -> 36 · padding 16 -> 48",
           width=376, size=8, name="zoom/p3/note3")
    b.meta(x + 24, 652, "all sizes are exactly 3x the 1.0x card (PDF p6:218-226)",
           width=376, size=8, name="zoom/p3/note4")
    b.meta(x + 24, 668, "panning at 3.0x keeps the core hive pinned (S02)",
           width=376, size=8, name="zoom/p3/note5")'''),
    ('    b.meta(1160, 766, "third frame is a real 3.0x vector crop, not a scaled "\n                      "bitmap", width=232, align="right", size=8,\n           name="zoom/vector-note")',
     '    b.meta(1160, 766, "3.0x frame is a real vector crop, not a bitmap",\n           width=232, align="right", size=8, name="zoom/vector-note")'),
])

# -------------------------------------------------------------- interaction board
patch("board_interaction.py", [
    ('    b.body(48, 98, "Every mutating control shows ACK or a named refusal before UI "\n                   "state changes. Focus is visible at >=3:1, refusals are never "\n                   "silent, and the code-diff surface stays DECISION-GATED.", 820,\n           size=12.5, line=18, name="ix/intro")',
     '    b.body(48, 96, "Every mutating control shows ACK or a named refusal before "\n                   "UI state changes · focus >=3:1 · refusals never silent · "\n                   "diff DECISION-GATED", 900, size=12.5, line=17,\n           name="ix/intro")'),
    # legend: 4 columns x 4 rows so no label touches its neighbour
    ('''    for i, st in enumerate(order):
        row, col = divmod(i, 5)
        cx = 944 + col * 90
        cy = 770 + row * 24''',
     '''    for i, st in enumerate(order):
        row, col = divmod(i, 4)
        cx = 944 + col * 108
        cy = 768 + row * 21'''),
    ('    b.meta(944, 848, "colour is never the only channel; unknown metric -> NULL / "',
     '    b.meta(944, 852, "colour is never the only channel; unknown metric -> NULL / "'),
])
print("done")
