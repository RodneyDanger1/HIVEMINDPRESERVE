import io
import json
import os
import shutil

HERE = os.path.dirname(os.path.abspath(__file__))
# keep the pre-change manifest for a byte-preservation check of untouched boards
prev = os.path.join(HM := os.path.dirname(HERE), "HiveMind-ASSETS.json")
if os.path.isfile(prev):
    shutil.copyfile(prev, os.path.join(HERE, "prev-assets.json"))

path = os.path.join(HERE, "build_hive_style.py")
text = io.open(path, encoding="utf-8").read()

NEW = r'''
# --------------------------------------------------------------------------
# BOARD 05 refinement -- exact spatial-system card anatomy annotations
# --------------------------------------------------------------------------
def board_agents_refine(f):
    RX = 1080
    label = "9.5 MONO"
    # dimension tags beside each component (exact spatial-system sizes)
    for (x, y, w, s) in [(392, 246, 100, "420 \u00d7 500"),
                         (900, 246, 140, "420 \u00d7 150"),
                         (900, 502, 140, "320 \u00d7 96"),
                         (900, 700, 140, "420 \u00d7 240"),
                         (900, 1046, 140, "320 \u00d7 110")]:
        text(f, s, x, y, 9, MONO, "Regular", INK500, width=w,
             align="right")
    # exact anatomy panel (bottom-left free area)
    rect(f, 72, 980, 540, 280, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "EXACT ANATOMY / SPATIAL SYSTEM", 92, 1000, 10, MONO, "Regular",
         INK500, ls="12%")
    rows = [
        ("HYPERAGENT / CHAT CORE", "420 \u00d7 500 \u00b7 radius 18 \u00b7 header 44"),
        ("SUPERAGENT", "420 \u00d7 150 \u00b7 header 40 \u00b7 state pill 68 \u00d7 18"),
        ("WORKERAGENT", "320 \u00d7 96 \u00b7 dot + name + role \u00b7 progress 3px"),
        ("TOOLAGENT", "420 \u00d7 240 \u00b7 132px dark result area + 3 dots"),
        ("ERROR", "320 \u00d7 110 \u00b7 fill #FFF7F5 \u00b7 crimson 1.5px"),
        ("GRID", "8 / 16 / 24 padding \u00b7 radius 14 super / 12 worker"),
    ]
    for i, (name, spec) in enumerate(rows):
        y = 1028 + i * 34
        text(f, name, 92, y, 10.5, UI, "SemiBold", INK900)
        text(f, spec, 92, y + 14, 9, MONO, "Regular", INK500, width=500)
        if i < len(rows) - 1:
            rule(f, 92, y + 30, 500, HAIR, 1)
    text(f, "measured from the spatial system; no new tokens or behaviour",
         92, 1238, 9, MONO, "Regular", INK500)


# --------------------------------------------------------------------------
# BOARD 11 -- Motion & Live (design proposal: durations/easing PROPOSED)
# --------------------------------------------------------------------------
def _arrow(f, x, y, angle, color=H400, size=8):
    import math
    a = math.radians(angle)
    p = "M 0 0 L %.2f %.2f L %.2f %.2f Z" % (
        -size * math.cos(a) - size * 0.5 * math.sin(a),
        -size * math.sin(a) + size * 0.5 * math.cos(a),
        -size * math.cos(a) + size * 0.5 * math.sin(a),
        -size * math.sin(a) - size * 0.5 * math.cos(a))
    vector(f, x, y, size * 2, size * 2, p, fill=color)


def board_motion(f):
    W, H = 1440, 900
    board_header(f, "11", "MOTION & LIVE", "Motion & live",
                 "The same spatial shell and cards, with the proposed motion layer: "
                 "ambient honeycomb travel, the 5-lobe hive sheen, bee flow, running "
                 "pulse, spinner, rise/hover and the ink-700 focus ring. Durations "
                 "and easing are PROPOSED, not measured; no new tokens or behaviour.")
    close_space(f, W, H, "MOTION / 32PX ARGUMENT")

    # ---- left: spatial shell, live layers ---------------------------------
    rect(f, 72, 200, 600, 430, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "SPATIAL SHELL / LIVE LAYERS \u00b7 PROPOSED", 92, 220, 10, MONO,
         "Regular", INK500, ls="10%")
    sx, sy, sw = 84, 246, 576
    scale = sw / 1440.0
    shell_h = 900 * scale
    top_h = 52 * scale
    rail_w = 72 * scale
    lib_w = 240 * scale
    canvas_x = sx + 312 * scale
    canvas_w = 1128 * scale
    canvas_y = sy + top_h
    canvas_h = 848 * scale
    rect(f, sx, sy, sw, shell_h, fill=PANEL, radius=8, stroke=HAIR, sw=1)
    rect(f, sx, sy, sw, top_h, fill=PANEL, radius=[8, 8, 0, 0], stroke=HAIR,
         sw=1)
    rect(f, sx, sy + top_h, rail_w, 848 * scale, fill=PANEL, stroke=HAIR, sw=1)
    rect(f, sx + rail_w, sy + top_h, lib_w, 848 * scale, fill=PANEL_ALT,
         stroke=HAIR, sw=1)
    rect(f, canvas_x, canvas_y, canvas_w, canvas_h, fill=CANVAS)
    honeycomb(f, canvas_x, canvas_y, canvas_w, canvas_h, R=13, color=HAIR_SOFT,
              weight=1)
    # micro shell furniture: topbar brand, rail dots, library rows, HUD,
    # chat 400x480 app variant (scaled), minimap
    hexmark(f, sx + 6, sy + 5, 10)
    text(f, "HiveMind", sx + 20, sy + 5, 6, UI, "Bold", INK900)
    for i in range(5):
        dot(f, sx + rail_w / 2, sy + top_h + 22 + i * 26, 3, H400 if i == 0 else HAIR)
    for i in range(5):
        rect(f, sx + rail_w + 8, sy + top_h + 16 + i * 22, lib_w - 18, 4,
             fill=HAIR, radius=999)
    hud_w, hud_h = 460 * scale, 44 * scale
    rect(f, sx + 312 * scale + (1128 * scale - hud_w) / 2, sy + 10, hud_w, hud_h,
         fill=H100, radius=6, stroke=BORDER_GOLD, sw=1)
    text(f, "4 SUPERAGENTS \u00b7 12 WORKERS \u00b7 98.2% GREEN",
         sx + 312 * scale + (1128 * scale - hud_w) / 2 + 6,
         sy + 10 + hud_h / 2 - 2, 5.5, MONO, "Regular", H800)
    chat_w, chat_h = 400 * scale, 480 * scale
    chat_x = canvas_x + (canvas_w - chat_w) / 2
    chat_y = canvas_y + (canvas_h - chat_h) / 2
    rect(f, chat_x, chat_y, chat_w, chat_h, fill=PANEL, radius=6, stroke=HAIR,
         sw=1, shadow=SH_E2)
    rect(f, chat_x, chat_y, chat_w, 44 * scale, fill=INK900, radius=[6, 6, 0, 0])
    text(f, "Hive AI Assistant", chat_x + 10, chat_y + 7, 5.5, UI, "SemiBold",
         "#FFFFFF")
    for i in range(3):
        rect(f, chat_x + 8, chat_y + 26 + i * 18, chat_w - 16, 9, fill=H100,
             radius=4)
    rect(f, chat_x + 8, chat_y + chat_h - 24, chat_w - 16, 16, fill=PANEL,
         radius=4, stroke=HAIR, sw=1)
    mini_w, mini_h = 220 * scale, 150 * scale
    rect(f, canvas_x + canvas_w - mini_w - 8, canvas_y + canvas_h - mini_h - 8,
         mini_w, mini_h, fill=PANEL_ALT, radius=5, stroke=HAIR, sw=1)
    # running pulse + focus ring + rise/hover mini nodes on the canvas
    pulse_x, pulse_y = canvas_x + 34, canvas_y + 34
    for i, o in enumerate([14, 9, 4]):
        ellipse(f, pulse_x - o, pulse_y - o, o * 2, o * 2, fill="none",
                stroke=H400 if i else H600, sw=1)
    _arrow(f, chat_x + chat_w + 18, chat_y + chat_h * 0.25, 0, H400, 5)
    # bee flow route (honey-gold, dashed, arrowheaded)
    route = "M %d %d C %d %d %d %d %d %d" % (
        chat_x + chat_w, chat_y + chat_h * 0.62,
        chat_x + chat_w + 50, chat_y + chat_h * 0.62,
        chat_x + chat_w + 70, chat_y + chat_h * 0.30,
        canvas_x + canvas_w - 10, canvas_y + 26)
    vector(f, 0, 0, canvas_w, canvas_h, route, fill="none", stroke=AMBER,
           sw=1.2, dashes=[5, 4])
    _arrow(f, canvas_x + canvas_w - 10, canvas_y + 26, 0, AMBER, 5)
    bee(f, chat_x + chat_w + 44, chat_y + chat_h * 0.55, 10, facing=1)
    # ambient honeycomb glow sweep (three dashed arcs + arrows)
    for i in range(3):
        gx = canvas_x + 12 + i * 10
        gy = canvas_y + 60 + i * 28
        vector(f, 0, 0, canvas_w, canvas_h, "M %d %d C %d %d %d %d %d %d" % (
            gx, gy, gx + 60, gy - 30, gx + 130, gy - 24, gx + 200, gy - 44),
            fill="none", stroke=H400, sw=1.0, dashes=[4, 5])
        _arrow(f, gx + 202, gy - 44, 0, H400, 4)
    # 5-lobe hive sheen
    for i in range(5):
        import math
        a = math.radians(-90 + i * 72)
        ex = chat_x + chat_w / 2 + 14 * math.cos(a)
        ey = chat_y + chat_h / 2 + 14 * math.sin(a)
        ellipse(f, ex - 10, ey - 10, 20, 20, fill=H200, stroke="none")
    # node rise/hover + ink-700 focus ring on a mini superagent node
    nx, ny = canvas_x + canvas_w - 150, canvas_y + canvas_h - 86
    rect(f, nx, ny, 92, 42, fill=H100, radius=6, stroke=BORDER_GOLD, sw=1.2)
    text(f, "SuperAgent", nx + 6, ny + 6, 5.5, UI, "SemiBold", INK900)
    rect(f, nx + 6, ny + 18, 70, 3, fill=HAIR, radius=999)
    rect(f, nx + 6, ny + 18, 44, 3, fill=H400, radius=999)
    text(f, "RISE 240ms \u00b7 HOVER 120ms", nx + 6, ny + 26, 4.5, MONO,
         "Regular", INK500)
    vector(f, nx - 4, ny - 4, 100, 50,
           "M 0 0 L 100 0 L 100 50 L 0 50 Z", fill="none", stroke=INK700,
           sw=1.6, dashes=[5, 4])
    _arrow(f, nx + 78, ny + 33, -90, INK700, 4)
    text(f, "FOCUS RING \u00b7 ink-700 \u00b7 2px \u00b7 160ms PROPOSED",
         nx - 28, ny + 52, 6, MONO, "Regular", INK700)
    # callout strip inside the left panel
    text(f, "AMBIENT GLOW 18s \u00b7 SHEEN 6s \u00b7 BEE FLOW 2.4s \u00b7 PULSE 1.6s "
            "\u00b7 SPINNER 1.2s (all PROPOSED)",
         92, 602, 8, MONO, "Regular", INK500, width=560)

    # ---- reduced-motion fallback ------------------------------------------
    rect(f, 72, 650, 600, 190, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "REDUCED-MOTION FALLBACK \u00b7 PROPOSED", 92, 670, 10, MONO,
         "Regular", INK500, ls="10%")
    fallback = [
        ("AMBIENT / SHEEN / PULSE / SPINNER", "loops off; static single frame"),
        ("BEE FLOW", "static dashed route + arrowhead + progress bar"),
        ("RISE / HOVER", "no translate; \u2264 120ms opacity change only"),
        ("FOCUS RING", "preserved as ink-700 2px ring (not motion)"),
        ("CHAT WORKING-BEE", "static bee + \u201cWorking\u2026\u201d text, no spin"),
    ]
    for i, (name, spec) in enumerate(fallback):
        y = 698 + i * 26
        dot(f, 96, y + 4, 3, INK700 if i == 3 else H400)
        text(f, name, 108, y, 9.5, UI, "Medium", INK900)
        text(f, spec, 300, y + 1, 9, MONO, "Regular", INK500, width=350)

    # ---- middle: the current app chat variant 400x480 ---------------------
    text(f, "CHAT CORE / APP VARIANT \u00b7 400 \u00d7 480", 700, 206, 10, MONO,
         "Regular", INK500, ls="8%")
    chat_card(f, 700, 230, 400, 480)
    ellipse(f, 1038, 244, 22, 22, fill="none", stroke=H400, sw=2)
    bee(f, 1044, 252, 9)
    _arrow(f, 1050, 255, 30, H400, 4)
    labels = [
        ("WORKING-BEE SPINNER", "1.2s linear \u00b7 PROPOSED"),
        ("HIVE SHEEN", "6s ease-in-out \u00b7 5 lobes \u00b7 PROPOSED"),
        ("FOCUS RING", "160ms \u00b7 ink-700 \u00b7 not honey \u00b7 PROPOSED"),
    ]
    for i, (name, spec) in enumerate(labels):
        y = 728 + i * 22
        text(f, name, 700, y, 9, MONO, "SemiBold", INK700, width=180)
        text(f, spec, 880, y, 8.5, MONO, "Regular", INK500, width=240)
    text(f, "board 07 stays 420 \u00d7 500; this board shows the app's current "
            "slightly-smaller centred chat variant.", 700, 800, 9, MONO,
         "Regular", INK500, width=400, lh="135%")

    # ---- right: motion spec table (durations/easing PROPOSED) -------------
    rect(f, 1120, 200, 248, 620, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "MOTION SPEC", 1140, 220, 10, MONO, "Regular", INK500, ls="12%")
    text(f, "PROPOSED \u00b7 not measured", 1140, 238, 8.5, MONO, "Regular",
         AMBER)
    spec = [
        ("AMBIENT TRAVEL", "18s \u00b7 linear \u00b7 loop"),
        ("HIVE SHEEN", "6s \u00b7 ease-in-out \u00b7 5 lobes"),
        ("BEE FLOW", "2.4s \u00b7 cubic-bezier(.2,.8,.2,1)"),
        ("BEE ARROWHEADS", "static caps on every route"),
        ("RUNNING PULSE", "1.6s \u00b7 ease-out \u00b7 loop"),
        ("WORKING-BEE SPINNER", "1.2s \u00b7 linear \u00b7 loop"),
        ("NODE RISE", "240ms \u00b7 cubic-bezier(.2,.8,.2,1)"),
        ("HOVER LIFT", "120ms \u00b7 ease-out \u00b7 2px"),
        ("FOCUS RING", "160ms \u00b7 ease-out \u00b7 ink-700"),
        ("REDUCED MOTION", "\u2264 120ms opacity \u00b7 loops off"),
    ]
    for i, (name, value) in enumerate(spec):
        y = 264 + i * 52
        text(f, name, 1140, y, 8.5, MONO, "SemiBold", INK800, width=210)
        text(f, value, 1140, y + 15, 8, MONO, "Regular", INK500, width=210,
             lh="130%")
        if i < len(spec) - 1:
            rule(f, 1140, y + 40, 208, HAIR, 1)
    text(f, "Design proposal only \u00b7 no component behaviour is frozen here.",
         1140, 786, 8, MONO, "Regular", INK500, width=210, lh="130%")
'''

MARK = '# --------------------------------------------------------------------------\n# Phase 1\n# --------------------------------------------------------------------------'
assert MARK in text
text = text.replace(MARK, NEW + '\n\n' + MARK)

old_call = '    board_states(FRAME_IDS["10 · States & Refusals"])\n'
new_call = old_call + (
    '\n    # preserve ids of boards 00-10: refine 05 and add 11 after they exist\n'
    '    board_agents_refine(FRAME_IDS["05 · Agent Cards"])\n'
    '    frame("11 · Motion & Live", 0, 5580, 1440, 900)\n'
    '    board_motion(FRAME_IDS["11 · Motion & Live"])\n')
assert old_call in text
text = text.replace(old_call, new_call)

assert '11 artboards' in text
text = text.replace('11 artboards', '12 artboards')

old_order = ('    order = ["00 · Cover", "01 · Colour", "02 · Typography", "03 · Space & Depth",\n'
             '             "04 · Brand Lockups", "05 · Agent Cards", "06 · Controls & HUD",\n'
             '             "07 · Spatial Canvas", "08 · HMAS Hierarchy",\n'
             '             "09 · HMAS Governance", "10 · States & Refusals"]')
new_order = ('    order = ["00 · Cover", "01 · Colour", "02 · Typography", "03 · Space & Depth",\n'
             '             "04 · Brand Lockups", "05 · Agent Cards", "06 · Controls & HUD",\n'
             '             "07 · Spatial Canvas", "08 · HMAS Hierarchy",\n'
             '             "09 · HMAS Governance", "10 · States & Refusals",\n'
             '             "11 · Motion & Live"]')
assert old_order in text
text = text.replace(old_order, new_order)

old_names = ('        # frame ids are deterministic: cover=2 .. hero=9\n'
             '        names = ["00 · Cover", "01 · Colour", "02 · Typography",\n'
             '                 "03 · Space & Depth", "04 · Brand Lockups", "05 · Agent Cards",\n'
             '                 "06 · Controls & HUD", "07 · Spatial Canvas",\n'
             '                 "08 · HMAS Hierarchy", "09 · HMAS Governance",\n'
             '                 "10 · States & Refusals"]\n'
             '        for i, n in enumerate(names):\n'
             '            FRAME_IDS[n] = 2 + i\n'
             '        phase2()')
new_names = ('        # frame ids are read back from phase 1: the motion frame is\n'
             '        # created after all existing board content to preserve prior ids.\n'
             '        expected = json.load(open(os.path.join(HERE, "expected_ids.json"),\n'
             '                                 encoding="utf-8"))\n'
             '        for row in expected:\n'
             '            if row["label"].startswith("frame:"):\n'
             '                FRAME_IDS[row["label"][6:]] = row["id"]\n'
             '        phase2()')
assert old_names in text, 'phase2 names block not found'
text = text.replace(old_names, new_names)

io.open(path, 'w', encoding='utf-8', newline='\n').write(text)
print('v2 builder patched: board 11 motion, board 05 anatomy refine, 12 artboards')
