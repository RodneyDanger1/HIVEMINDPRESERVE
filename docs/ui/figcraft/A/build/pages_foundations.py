"""Stage commands: file setup (pages, 20 colour variables) and Page 00 Foundations."""
from gen_common import (Board, C, TOKENS, CONTRAST_PRINTED, TYPE_SCALE, SPACING,
                        RADII, ELEVATION)

PAGES = [
    ("00 Foundations", 1440, 1200),
    ("01 Brand", 1440, 900),
    ("02 Components", 1440, 2580),
    ("03 Specimens", 1440, 1400),
]


def setup_a_commands():
    """Run 0: new file, four pages, 20 colour variables (PDF p2 token paths)."""
    cmds = [
        {"command": "file.new", "params": {"name": "HiveMind v1 Design System"}},
        {"command": "page.rename", "params": {"id": 1, "name": PAGES[0][0]}},
    ]
    for name, _w, _h in PAGES[1:]:
        cmds.append({"command": "page.add", "params": {"name": name}})
    cmds.append({"command": "variable.createCollection",
                 "params": {"name": "HiveMind v1"}})
    for display, path, hexv in TOKENS:
        cmds.append({"command": "variable.create",
                     "params": {"collection": "HiveMind v1", "name": path,
                                "type": "color", "value": hexv}})
    return cmds


def setup_b_commands(page_ids):
    """Run 1: one top-level frame per page + page backgrounds."""
    cmds = []
    for (name, w, h), pid in zip(PAGES, page_ids):
        cmds.append({"command": "page.select", "params": {"id": pid}})
        cmds.append({"command": "frame.create", "params": {
            "name": name, "x": 0, "y": 0, "width": w, "height": h}})
        # the new frame is the selection: paint it with the canvas token
        cmds.append({"command": "node.set", "params": {"fill": C["canvas"]}})
        cmds.append({"command": "page.setBackground",
                     "params": {"id": pid, "color": C["canvas"]}})
    return cmds


# ------------------------------------------------------------------ page 00
def build_foundations(frame_id):
    b = Board("00", "Foundations", frame_id, 1440, 1200)
    b.rect(0, 0, 1440, 1200, fill=C["canvas"], name="00/board-bg")

    # header ---------------------------------------------------------------
    b.eyebrow(48, 44, "01 / FOUNDATIONS  ·  20 SWATCHES  ·  8 TYPE STEPS  ·  7 RADII  ·  "
                      "3 ELEVATION TIERS  ·  PDF p2:20-92, p3:111-124, p4:130-146",
              group="00-header")
    b.text(48, 64, "Warm ink on honey paper.", 56, "Playfair Display", "SemiBold",
           C["ink900"], width=1100, line=60, tracking=-2, name="00/title",
           group="00-header")
    b.body_text(48, 140,
                "Every value below is either sampled from the supplied canvas reference or "
                "inherited from the shipped app tokens; derived values are recorded as derived. "
                "Token names map 1:1 to FigCraft colour variables.", 900, size=17, line=28,
                group="00-header")
    b.eyebrow(1092, 44, "HIVEROUTER  ·  DESIGN SYSTEM  /  V1.0", color=C["ink700"],
              width=300, align="right", group="00-header")
    b.eyebrow(1092, 62, "FIGCRAFT NATIVE  ·  1440 × 1200", color=C["ink500"],
              width=300, align="right", group="00-header")

    # ---- band A: palette (left) -------------------------------------------
    b.section(48, 220, 620, "palette · 20 printed swatches", note="PDF p2:20-79")
    for i, tok in enumerate(TOKENS):
        col, row = i // 10, i % 10
        x = 48 + col * 312
        y = 244 + row * 44
        b.swatch_row(x, y, 292, tok, group="00-palette",
                     surface=tok[1] if tok[1] == "surface/canvas" else None)

    # ---- band A: type scale (right, 2 sub-columns) ------------------------
    b.section(700, 220, 692, "type scale · 8 printed steps", note="PDF p3:111-124")
    TYPE_LAYOUT = [
        # (column, id, size, sample, width, line, tracking)
        (0, "display/xxl", 96, "Aa", None, 100, -3.0),
        (0, "heading/h1", 40, "Agent", None, 48, -1.5),
        (0, "body/l", 17, "Drop a repository into the core hive.", 330, 28, 0.0),
        (0, "mono/code", 13, "const route = hive.snap(worker, a);", None, 22, 0.0),
        (1, "display/l", 56, "HiveMind", None, 60, -2.0),
        (1, "heading/h3", 24, "SuperAgent - Backend", None, 32, -0.5),
        (1, "label", 12, "ACTIVE AGENTS", None, 16, 8.0),
        (1, "mono/meta", 11, "NODE_SPAWN  ·  NODE_DRAG  ·  NODE_REWIRE", None, 16, 8.0),
    ]
    spec_of = dict((t[0], t) for t in TYPE_SCALE)
    col_y = {0: 244, 1: 244}
    for col, tid, size, sample, width, line, track in TYPE_LAYOUT:
        x = 700 + col * 346
        y = col_y[col]
        fam, sty = spec_of[tid][4], spec_of[tid][5]
        b.text(x, y, sample, size, fam, sty, C["ink900"], width=width, line=line,
               tracking=track if track else None, name="00/type/%s" % tid,
               group="00-type")
        idx = len(b.el) - 1
        b.text_styles.append((tid, idx))
        b.text(x, y + line + 3, "%s %s/%s/%+g%%" % (tid, size, line, track), 9,
               "Consolas", "Regular", C["ink500"], width=330, line=12, align="right",
               group="00-type")
        col_y[col] += line + 21

    b.text(1046, 478,
           "Specs read: token  SIZE / LINE / TRACKING.",
           8.5, "Consolas", "Regular", C["ink500"], width=346, line=12, group="00-type")
    b.text(1046, 492,
           "Tracking is a percentage of the em; larger sizes track tighter (PDF p3:111-124).",
           8.5, "Consolas", "Regular", C["ink500"], width=346, line=12, group="00-type")

    # font families strip under the type scale ------------------------------
    b.section(700, 566, 692, "font families · 3 resolved", note="PDF p3:97-127")
    fams = [
        ("Playfair Display", "SemiBold", "DISPLAY · brand statements, board titles"),
        ("Inter", "SemiBold / Regular", "UI / TEXT · controls, cards, tables, body"),
        ("Consolas", "Regular", "MONO / CODE · code, tokens, hex, events"),
    ]
    for i, (fam, sty, role) in enumerate(fams):
        fx = 700 + i * 232
        b.rect(fx, 592, 224, 52, fill=C["paper"], r=8, stroke=C["hairline"], sw=1,
               group="00-fonts")
        b.text(fx + 10, 598, "Aa", 20, fam, sty.split(" / ")[0], C["ink900"],
               width=36, line=24, group="00-fonts")
        b.text(fx + 52, 596, fam, 11.5, "Inter", "SemiBold", C["ink900"],
               width=140, line=15, group="00-fonts")
        b.text(fx + 52, 612, sty, 9, "Consolas", "Regular", C["ink500"],
               width=164, line=12, group="00-fonts")
        b.text(fx + 10, 630, role, 8, "Consolas", "Regular", C["ink500"],
               width=204, line=11, group="00-fonts")
    b.text(700, 660,
           "RESOLVED HERE: Playfair Display · Inter · Consolas.   NOT INSTALLED: JetBrains Mono.",
           8.5, "Consolas", "Regular", C["ink700"], width=692, line=12, group="00-fonts")
    b.text(700, 674,
           "The product self-hosts Instrument Serif (@fontsource); this board machine cannot resolve it — Playfair stands in (PDF p3:127).",
           8.5, "Consolas", "Regular", C["ink500"], width=692, line=12, group="00-fonts")

    # ---- band B: contrast / spacing / radii / elevation -------------------
    b.section(48, 720, 330, "contrast · WCAG 2.1", note="PDF p2:80-92")
    for i, (pair, ratio, level) in enumerate(CONTRAST_PRINTED):
        col, row = i // 3, i % 3
        x = 48 + col * 166
        y = 748 + row * 24
        b.text(x, y, pair, 8.5, "Consolas", "Regular", C["ink700"], width=96,
               line=12, group="00-contrast")
        b.text(x + 98, y, ratio + " " + level, 8.5, "Consolas", "Regular",
               C["ink900"], width=66, line=12, align="right", group="00-contrast")
    b.text(48, 828, "CORRECTED FOR AA · UI-R-03/04/05", 8.5, "Consolas", "Regular",
           C["signal"], width=330, line=12, tracking=8, group="00-corrections")
    corr = [
        "meta #8A7A52 4.21/3.84 → ink-500 #75663F 5.13",
        "ring honey-300 1.70/1.55 → ink-700 #4E3A15 9.86",
        "amber #B7791F 3.32 → large text only (≥24px)",
    ]
    for i, s in enumerate(corr):
        b.text(48, 844 + i * 15, s, 8, "Consolas", "Regular", C["ink700"],
               width=330, line=12, group="00-corrections")

    # spacing
    b.section(394, 720, 330, "spacing · 4px base", note="PDF p4:132-133")
    for i, v in enumerate(SPACING):
        bx = 394 + i * 29.5
        bh = 6 + (v / 96.0) * 50
        b.rect(bx, 858 - bh, 20, bh, fill=C["honey500"] if v in (16, 24) else C["honey300"],
               r=2, group="00-spacing")
        b.text(bx, 864, str(v), 8.5, "Consolas", "Regular", C["ink700"], width=26,
               line=12, align="center", group="00-spacing")
    b.text(394, 880, "12 stops: 4 8 12 16 20 24 32 40 48 64 80 96", 8.5,
           "Consolas", "Regular", C["ink500"], width=330, line=13, group="00-spacing")
    b.rect(394, 898, 330, 62, fill=C["paper"], r=8, stroke=C["hairline"], sw=1,
           group="00-subscale")
    b.text(402, 904, "SUB-4PX EXCEPTIONS · UI-R-09", 8.5, "Consolas", "Regular",
           C["signal"], width=300, line=12, tracking=8, group="00-subscale")
    b.text(402, 920, "border 1.5px · worker bar 3px · progress 5px", 8.5,
           "Consolas", "Regular", C["ink700"], width=314, line=12, group="00-subscale")
    b.text(402, 934, "focus ring 2px · meta type 9.5px", 8.5, "Consolas", "Regular",
           C["ink700"], width=314, line=12, group="00-subscale")

    # radii
    b.section(740, 720, 300, "radius · 7 stops", note="PDF p4:135")
    rad_layout = [(4, 34, 34), (8, 34, 34), (12, 34, 34), (16, 34, 34),
                  (24, 40, 40), (48, 40, 40), (999, 58, 26)]
    rx = 740
    for v, wdt, hgt in rad_layout:
        b.rect(rx, 750 + (40 - hgt), wdt, hgt, fill=C["honey100"],
               r=(hgt if v == 999 else v), stroke=C["honey500"], sw=1,
               group="00-radii")
        b.text(rx, 794, str(v), 8.5, "Consolas", "Regular", C["ink700"],
               width=wdt, line=12, align="center", group="00-radii")
        rx += wdt + 8
    b.text(740, 806, "10 · button (off scale)", 8.5, "Consolas", "Regular",
           C["signal"], width=200, line=12, group="00-radii")
    b.text(740, 820, "14 · super card (off scale)", 8.5, "Consolas", "Regular",
           C["signal"], width=200, line=12, group="00-radii")
    b.text(740, 838, "Printed: seven stops (p4:136). Prose: “six radii” (p4:135). "
                     "Components add 10 and 14 (p6:224 / p7:240) — R-U14 / K-07.",
           7.5, "Consolas", "Regular", C["ink500"], width=300, line=11,
           group="00-radii")

    # elevation (S24)
    b.section(1056, 720, 336, "elevation · e1 e2 e3", note="PDF p4:137-146")
    for i, (name, desc) in enumerate(ELEVATION):
        ey = 748 + i * 62
        b.rect(1078, ey + 4, 292, 50, fill=C["paper"], r=10, stroke=C["hairline"],
               sw=1, group="00-elevation", surface="S24" if i == 0 else None,
               name="00/elev/%s/card" % name)
        idx = len(b.el) - 1
        b.effect_styles.append(("e%d/%s" % (i + 1, name.split(" / ")[1]), idx))
        b.name_text(1088, ey + 10, name, 11.5, group="00-elevation")
        b.text(1088, ey + 26, desc, 8.5, "Consolas", "Regular", C["ink500"],
               width=200, line=12, group="00-elevation")
        b.text(1264, ey + 26, "#282010 @ 7-13%", 8.5, "Consolas", "Regular",
               C["ink700"], width=96, line=12, align="right", group="00-elevation")
    b.rect(1056, 938, 336, 24, fill=C["honey100"], r=6, group="00-elevation")
    b.text(1064, 942, "One printed value for all tiers — do not differentiate (UI-R-10 / K-13).",
           7.5, "Consolas", "Regular", C["ink700"], width=322, line=11,
           group="00-elevation")

    # ---- band C: printed totals do not reconcile --------------------------
    b.section(48, 976, 1344, "printed totals do not reconcile · printed values only",
              note="R-U2 / K-11 — declared counts stay UNVERIFIED")
    stats = [
        ("20 / 21", "SWATCHES / COLOUR VARIABLES",
         "20 printed swatches (p2:20-79); cover declares 21 colour variables (p1:11)."),
        ("8 / 13", "TYPE STYLES",
         "8 printed steps (p3:111-124); cover declares 13 text styles (p1:11)."),
        ("7 / six", "RADIUS STOPS",
         "7 printed stops (p4:136); prose says six (p4:135)."),
        ("3 / 1", "ELEVATION TIERS / VALUES",
         "3 tiers print one identical shadow value (p4:137-146)."),
    ]
    for i, (big, label, note) in enumerate(stats):
        sx = 48 + i * 342
        b.rect(sx, 1002, 318, 96, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
               group="00-totals")
        b.text(sx + 14, 1010, big, 34, "Playfair Display", "SemiBold", C["ink900"],
               width=280, line=40, tracking=-1, group="00-totals")
        b.text(sx + 14, 1054, label, 8.5, "Consolas", "Regular", C["ink700"],
               width=290, line=12, tracking=8, group="00-totals")
        b.text(sx + 14, 1070, note, 8, "Consolas", "Regular", C["ink500"],
               width=290, line=11, group="00-totals")
    b.text(48, 1104,
           "Rule R-U2: printed totals (21 swatches / 13 text styles / six radii) do not reconcile with the extract (20 / 8 / 7). "
           "Only printed values enter the token module; every unreconciled count stays UNVERIFIED.",
           9, "Consolas", "Regular", C["ink700"], width=1344, line=14, group="00-totals")

    # footer ----------------------------------------------------------------
    b.line(48, 1150, 1344, C["hairline"], 1, name="00/footer-rule")
    b.text(48, 1158, "HIVEROUTER  ·  DESIGN SYSTEM / V1.0  ·  00 FOUNDATIONS  ·  FIGCRAFT NATIVE",
           9, "Consolas", "Regular", C["ink500"], width=900, line=13, tracking=8,
           name="00/footer")
    b.text(1192, 1158, "PAGE 1 / 4  ·  PDF p2-p4", 9, "Consolas", "Regular",
           C["ink500"], width=200, line=13, align="right", name="00/footer-no")
    return b
