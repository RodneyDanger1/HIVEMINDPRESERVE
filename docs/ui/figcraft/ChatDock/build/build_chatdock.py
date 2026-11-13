#!/usr/bin/env python3
"""Build the proposed chat-dock/Help FigCraft board (13).

This is a *proposal* board for the current HiveMind UI, not watchdog-audited
design truth. It is generated with the installed FigCraft CLI's verified
command surface (`commands --json`, 2026-11-13, figcraft-cli 0.17.0):

  node.create {type, x, y, width, height, parent, name, props}
  text.create {text, x, y, parent, width, props}
  frame.create {x, y, width, height, name, props}
  file.saveAs {path}

The board is one 1440x1800 frame with two scenes:
  13A  the new full-width bottom chat dock (main column = 1128 at 1440)
  13B  the Help nav destination (not a dock tab)

Source of each geometry/value is cited in SPEC/CHATDOCK-TARGET-SPEC.md.
This script only writes commands.json; run it with the CLI to produce the
.figcraft design file, then run publish_chatdock.py to export + hash + manifest.
"""
import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
CHATDOCK = os.path.dirname(HERE)
OUT = os.path.join(HERE, "commands.json")

W, H = 1440, 1800
FRAME = 2  # file.new creates page 1; frame.create is the next layer

# ---- tokens: src/tokens/tokens.ts (20 printed swatches) + chrome tokens ----
CANVAS = "#F8F4EB"
PANEL = "#FFFFFF"
PANEL_ALT = "#FBF8F1"
RECESS = "#F1E9CF"
HAIR = "#E5E1D8"
H100 = "#FBF3DC"
H200 = "#F6E7BE"
H300 = "#F0C050"
H400 = "#E8B850"
H500 = "#D9A02F"
H600 = "#C88030"
H700 = "#8A5A1C"
H800 = "#604818"
INK950 = "#14100A"
INK900 = "#282010"
INK800 = "#3F3115"
INK700 = "#4E3A15"
INK600 = "#6B5A33"
INK500 = "#75663F"
AMBER = "#B7791F"
SIGNAL = "#B23A2E"
VERIFY = "#3F7A55"
BORDER_GOLD = "#E5C27A"
ERROR_SURFACE = "#FFF7F5"

BRAND = "Playfair Display"
UI = "Inter"
MONO = "Consolas"

SH_E1 = [
    {"type": "dropShadow", "offset": [0, 1], "radius": 3, "spread": 0,
     "color": "#28201012", "showBehind": True},
    {"type": "dropShadow", "offset": [0, 8], "radius": 24, "spread": 0,
     "color": "#28201014", "showBehind": True},
]
SH_E2 = [
    {"type": "dropShadow", "offset": [0, 2], "radius": 6, "spread": 0,
     "color": "#28201014", "showBehind": True},
    {"type": "dropShadow", "offset": [0, 20], "radius": 48, "spread": 0,
     "color": "#2820101F", "showBehind": True},
]
SH_E3 = [
    {"type": "dropShadow", "offset": [0, 0], "radius": 80, "spread": -10,
     "color": "#8A5A1C3D", "showBehind": True},
    {"type": "dropShadow", "offset": [0, 24], "radius": 48, "spread": 0,
     "color": "#60481833", "showBehind": True},
]
HEADER_GRAD = [
    {"type": "linearGradient", "stops": ["#3F3115", "#604818"],
     "from": [0, 0], "to": [1, 1]},
]
HONEY_GRAD = [
    {"type": "radialGradient", "stops": ["#FFF8E5", "#F0C050", "#C88030"]},
]

CMDS = []


def add(command, **params):
    CMDS.append({"command": command, "params": params})


def rect(x, y, w, h, fill=None, fills=None, radius=0, stroke=None, sw=1,
         shadow=None, opacity=None, name=None):
    props = {}
    if fills is not None:
        props["fills"] = fills
    elif fill is not None:
        props["fill"] = fill
    if radius:
        props["cornerRadius"] = radius
    if stroke:
        props["stroke"] = stroke
        props["strokeWeight"] = sw
    if shadow:
        props["effects"] = shadow
    if opacity is not None:
        props["opacity"] = opacity
    p = {"type": "rectangle", "x": x, "y": y, "width": w, "height": h,
         "parent": FRAME, "props": props}
    if name:
        p["name"] = name
    add("node.create", **p)


def ellipse(x, y, w, h, fill=None, fills=None, stroke=None, sw=1,
            shadow=None, opacity=None, name=None):
    props = {}
    if fills is not None:
        props["fills"] = fills
    elif fill is not None:
        props["fill"] = fill
    if stroke:
        props["stroke"] = stroke
        props["strokeWeight"] = sw
    if shadow:
        props["effects"] = shadow
    if opacity is not None:
        props["opacity"] = opacity
    p = {"type": "ellipse", "x": x, "y": y, "width": w, "height": h,
         "parent": FRAME, "props": props}
    if name:
        p["name"] = name
    add("node.create", **p)


def vector(x, y, w, h, path, fill=None, stroke=None, sw=1, dashes=None,
           opacity=None, name=None):
    props = {}
    if fill is not None:
        props["fill"] = fill
    if stroke:
        props["stroke"] = stroke
        props["strokeWeight"] = sw
    if dashes:
        props["dashes"] = list(dashes)
    if opacity is not None:
        props["opacity"] = opacity
    p = {"type": "vector", "x": x, "y": y, "width": w, "height": h,
         "parent": FRAME, "path": path, "props": props}
    if name:
        p["name"] = name
    add("node.create", **p)


def text(s, x, y, size=13, font=UI, style="Regular", color=INK900,
         width=None, lh=None, ls=None, align=None, name=None):
    props = {"fontSize": size, "fontFamily": font, "fontStyle": style,
             "fill": color}
    if lh:
        props["lineHeight"] = lh
    if ls:
        props["letterSpacing"] = ls
    if align:
        props["textAlign"] = align
    p = {"text": s, "x": x, "y": y, "parent": FRAME, "props": props}
    if width is not None:
        p["width"] = width
        props["textAutoResize"] = "height"
    if name:
        p["name"] = name
    add("text.create", **p)


def vrule(x, y, h, color=HAIR, w=1):
    rect(x, y, w, h, fill=color)


def hex_path(r):
    px = math.sqrt(3) * r
    pts = [(0, 0.5 * r), (px / 2, 0), (px, 0.5 * r),
           (px, 1.5 * r), (px / 2, 2 * r), (0, 1.5 * r)]
    return "M " + " L ".join(f"{a:.2f} {b:.2f}" for a, b in pts) + " Z", px, 2 * r


def hex_icon(cx, cy, r, stroke=INK700, fill="none", sw=1.6):
    path, w, h = hex_path(r)
    vector(cx - w / 2, cy - h / 2, w, h, path, fill=fill, stroke=stroke, sw=sw)


def hex_mark(x, y, size, face=INK900, glyph=H400):
    """Small brand hexagon with an inner honey hex."""
    rect(x, y, size, size, fill=face, radius=size * 0.24)
    hex_icon(x + size / 2, y + size / 2, size * 0.27, stroke=glyph, sw=2)


def dot(cx, cy, r, color):
    ellipse(cx - r, cy - r, r * 2, r * 2, fill=color)


def chevron(x, y, s=8, color=INK500):
    vector(x, y, s, s * 0.7, f"M 0 0 L {s / 2:.1f} {s * 0.6:.1f} L {s:.1f} 0",
           fill="none", stroke=color, sw=1.4)


def honeycomb(x, y, w, h, r=44, color="#EFE6CE", weight=1):
    px = math.sqrt(3) * r
    py = 1.5 * r
    cols = int(math.ceil(w / px)) + 2
    rows = int(math.ceil(h / py)) + 2
    centers = []
    for row in range(rows):
        for col in range(cols):
            cx = col * px + (px / 2 if row % 2 else 0)
            cy = row * py
            centers.append((cx, cy))
    minx = min(c[0] for c in centers) - px / 2
    miny = min(c[1] for c in centers) - r
    maxx = max(c[0] for c in centers) + px / 2
    maxy = max(c[1] for c in centers) + r
    parts = []
    for cx, cy in centers:
        ox, oy = cx - px / 2 - minx, cy - r - miny
        pts = [(0, 0.5 * r), (px / 2, 0), (px, 0.5 * r),
               (px, 1.5 * r), (px / 2, 2 * r), (0, 1.5 * r)]
        parts.append("M " + " L ".join(f"{ox + a:.1f} {oy + b:.1f}"
                                       for a, b in pts) + " Z")
    vector(x + minx, y + miny, maxx - minx, maxy - miny, " ".join(parts),
           fill="none", stroke=color, sw=weight, name="honeycomb")


def bee(cx, cy, size=14, facing=1):
    body = f"M 0 {size * 0.28:.1f} L {size * 0.72:.1f} {size * 0.28:.1f} " \
           f"L {size * 0.62:.1f} {size * 0.62:.1f} L {size * 0.1:.1f} {size * 0.62:.1f} Z"
    vector(cx - size / 2, cy - size * 0.2, size, size * 0.7, body,
           fill=INK800)
    ellipse(cx - size * 0.42, cy - size * 0.28, size * 0.34, size * 0.22,
            fill=H200, opacity=0.9)
    ellipse(cx + size * 0.08, cy - size * 0.28, size * 0.34, size * 0.22,
            fill=H200, opacity=0.9)
    rect(cx - size * 0.14, cy + size * 0.06, size * 0.5, 1.6, fill=H400)
    if facing < 0:
        rect(cx - size * 0.04, cy + size * 0.02, size * 0.2, 1.2, fill=INK950)


def bee_route(x1, y1, x2, y2, color=H500, bee_at=0.55, size=13):
    w, h = x2 - x1, y2 - y1
    p = (f"M 0 0 L {abs(w):.1f} {abs(h):.1f}" if (w >= 0 and h >= 0)
         else f"M 0 {abs(h):.1f} L {abs(w):.1f} 0")
    vector(min(x1, x2), min(y1, y2), abs(w) or 1, abs(h) or 1, p,
           fill="none", stroke=color, sw=1.4, dashes=[8, 8],
           name="bee-route")
    bee(x1 + w * bee_at, y1 + h * bee_at, size=size,
        facing=(-1 if w < 0 else 1))


def pill(x, y, w, h, label, size=9, color=INK800, fill=PANEL,
         stroke=HAIR, ls="8%"):
    rect(x, y, w, h, fill=fill, radius=999, stroke=stroke, sw=1)
    text(label, x + 10, y + (h - size) / 2 - 1, size, MONO, "Medium", color,
         ls=ls)


def feature_card(x, y, w, h, label, body):
    rect(x, y, w, h, fill=PANEL_ALT, radius=10, stroke=HAIR, sw=1)
    text(label, x + 12, y + 7, 8.5, MONO, "SemiBold", INK800, ls="8%")
    text(body, x + 12, y + 20, 10, UI, "Regular", INK500, width=w - 24, lh=13)


def tutorial_card(x, y, w, h, title, steps):
    rect(x, y, w, h, fill=PANEL, radius=14, stroke=HAIR, sw=1, shadow=SH_E1)
    text(title, x + 14, y + 13, 18, BRAND, "SemiBold", INK900)
    yy = y + 42
    for step in steps:
        dot(x + 18, yy + 5, 2, H500)
        text(step, x + 28, yy, 10, UI, "Regular", INK600, width=w - 42, lh=13)
        yy += 25


def header(y, title_hint):
    rect(0, y, W, 52, fill=PANEL)
    vrule(0, y + 52, 1, HAIR, W)
    hex_mark(16, y + 11, 30)
    text("HiveMind", 54, y + 15, 17, UI, "Bold", INK900)
    pill(1024, y + 12, 116, 28, "HIDE CHAT")
    rect(1150, y + 10, 32, 32, fill=PANEL, radius=999, stroke=HAIR)
    text("?", 1161, y + 16, 13, UI, "SemiBold", INK900)
    rect(1194, y + 12, 104, 28, fill=PANEL, radius=999, stroke=HAIR)
    dot(1208, y + 26, 3, VERIFY)
    text("CONNECTED", 1218, y + 21, 8.5, MONO, "Medium", INK800, ls="6%")
    rect(1312, y + 8, 32, 32, fill=H200, radius=999)
    text("HM", 1322, y + 18, 9.5, UI, "SemiBold", INK900)
    text("OPERATOR", 1350, y + 20, 8.5, MONO, "Regular", INK500, ls="6%")
    text(title_hint, 900, y + 21, 8, MONO, "Regular", "#A0916F", ls="10%")


def nav_rail(y, h, active_index):
    rect(0, y, 72, h, fill=PANEL)
    vrule(72, y, h, HAIR, 1)
    items = ["Workspace", "Agents", "Data", "Models", "Routing", "Remote",
             "Settings", "Help"]
    step = 80 if h < 760 else 92
    for i, label in enumerate(items):
        yy = y + 22 + i * step
        active = i == active_index
        if active:
            rect(12, yy - 4, 48, 48, fill=INK900, radius=11)
        if label == "Help":
            rect(26, yy + 7, 20, 20, fill="none", radius=999,
                 stroke=H400 if active else INK700, sw=1.8)
            text("?", 31, yy + 10, 12, UI, "SemiBold",
                 H400 if active else INK700)
        elif label == "Agents":
            dot(36, yy + 13, 3, H400 if active else INK700)
            dot(30, yy + 20, 3, H400 if active else INK700)
            dot(42, yy + 20, 3, H400 if active else INK700)
        elif label == "Data":
            rect(27, yy + 10, 18, 4, fill=H400 if active else INK700, radius=2)
            rect(27, yy + 17, 18, 4, fill=H400 if active else INK700, radius=2)
            rect(27, yy + 24, 18, 4, fill=H400 if active else INK700, radius=2)
        elif label == "Models":
            hex_icon(36, yy + 18, 9, stroke=H400 if active else INK700, sw=1.7)
        elif label == "Routing":
            vector(27, yy + 10, 18, 16,
                   "M 0 8 L 8 8 L 12 2 L 18 2 M 8 8 L 12 14 L 18 14",
                   fill="none", stroke=H400 if active else INK700, sw=1.6)
        elif label == "Remote":
            vector(27, yy + 13, 18, 12,
                   "M 2 12 A 12 12 0 0 1 16 12 M 5 9 A 8 8 0 0 1 13 9",
                   fill="none", stroke=H400 if active else INK700, sw=1.6)
            dot(36, yy + 23, 2.5, H400 if active else INK700)
        elif label == "Settings":
            ellipse(28, yy + 10, 16, 16, fill="none", stroke=H400 if active else INK700, sw=1.6)
            dot(36, yy + 18, 3, H400 if active else INK700)
        else:
            hex_icon(36, yy + 18, 9, stroke=H400 if active else INK700, sw=1.6)
        text(label, -4, yy + 50, 8, UI, "Medium" if active else "Regular",
             INK900 if active else INK500, width=80, align="center")


def library(y, h, compact=False):
    rect(72, y, 240, h, fill=PANEL_ALT)
    vrule(312, y, h, HAIR, 1)
    rect(84, y + 16, 216, 32, fill=PANEL, radius=8, stroke=HAIR, sw=1)
    text("Project: HiveMind AI", 96, y + 24, 10.5, UI, "Medium", INK800)
    chevron(276, y + 29, 7)
    rect(84, y + 60, 104, 24, fill=H300, radius=6)
    text("WORKSPACE", 94, y + 66, 8.5, MONO, "SemiBold", INK900, ls="6%")
    rect(194, y + 60, 70, 24, fill=PANEL, radius=6, stroke=HAIR, sw=1)
    text("SWARM", 204, y + 66, 8.5, MONO, "Regular", INK500, ls="6%")
    rect(84, y + 90, 80, 24, fill=PANEL, radius=6, stroke=HAIR, sw=1)
    text("ROUTING", 94, y + 96, 8.5, MONO, "Regular", INK500, ls="6%")
    rect(170, y + 90, 80, 24, fill=PANEL, radius=6, stroke=HAIR, sw=1)
    text("HISTORY", 180, y + 96, 8.5, MONO, "Regular", INK500, ls="6%")

    text("AGENTS IN RUN", 84, y + 130, 9.5, UI, "SemiBold", INK900, ls="8%")
    text("3 ACTIVE · 1 QUEUED", 84, y + 144, 8, MONO, "Regular", INK500, ls="6%")
    agents = [("SUPER", "#69a5678a", H300), ("WORKER", "#0da1931a", H500),
              ("WORKER", "#e41b9538", H600)]
    for i, (kind, ident, color) in enumerate(agents):
        yy = y + 160 + i * 30
        rect(84, yy, 216, 24, fill=PANEL, radius=6, stroke=HAIR, sw=1)
        hex_icon(96, yy + 12, 6, stroke=INK800, sw=1.4)
        text(f"{kind} {ident}", 108, yy + 7, 8.5, MONO, "Regular", INK800)
        dot(286, yy + 12, 2.5, color)

    text("TEMPLATES · 8", 84, y + 270, 9.5, UI, "SemiBold", INK900, ls="8%")
    templates = ["agent", "chatbot", "code-reviewer", "content-writer",
                 "data-analyst", "planner", "subagent", "superagent"]
    for i, name in enumerate(templates):
        yy = y + 286 + i * 28
        rect(84, yy, 216, 22, fill=PANEL, radius=6, stroke=HAIR, sw=1)
        hex_icon(95, yy + 11, 5, stroke=INK600, sw=1.3)
        text(name, 106, yy + 6, 9, UI, "Regular", INK800)
        if name in ("agent", "planner", "subagent", "superagent"):
            text("CATALOG", 242, yy + 6, 7, MONO, "Regular", INK500, ls="6%")

    tip_y = y + (560 if compact else 590)
    rect(84, tip_y, 216, 66, fill=H100, radius=8)
    text("TIP", 96, tip_y + 10, 8, MONO, "SemiBold", H700, ls="12%")
    text("Drag the dock grip up or down; Help lives in the nav rail.",
         96, tip_y + 24, 9.5, UI, "Regular", INK600, width=192, lh=12)
    text("STATE & SETTINGS", 84, y + h - 26, 8, MONO, "Regular", INK500, ls="8%")
    chevron(286, y + h - 25, 7)


def scene_a():
    header(96, "SCENE 13A / CHAT DOCK")
    nav_rail(148, 848, 0)
    library(148, 848)
    rect(312, 148, 1128, 848, fill=CANVAS)
    honeycomb(312, 148, 1128, 848, r=44, color="#EFE6CE", weight=1)

    # canvas chrome
    pill(340, 164, 96, 24, "HIDE FINISHED")
    pill(444, 164, 100, 24, "FOCUS BRANCH")
    pill(552, 164, 44, 24, "2/4")
    pill(604, 164, 84, 24, "GRAPH ONLY")
    text("HIVEMIND / WORKSPACE", 1200, 168, 8, MONO, "Regular", "#A0916F", ls="12%")

    # HUD
    rect(700, 164, 460, 44, fill=PANEL, radius=999, stroke=BORDER_GOLD, sw=1.2,
         shadow=SH_E2)
    dot(724, 186, 4.5, VERIFY)
    text("HYPERAGENT · 3 SUPERAGENTS · 4 WORKERS · LIVE", 740, 180, 9.5,
         MONO, "SemiBold", INK800, ls="5%")

    # hive rings (partly occluded by the real full-width dock, as in the app)
    ellipse(560, 228, 520, 210, fills=HONEY_GRAD, opacity=0.9, shadow=SH_E3)
    ellipse(604, 320, 432, 186, fills=HONEY_GRAD, opacity=0.92, shadow=SH_E3)
    ellipse(640, 404, 360, 150, fills=HONEY_GRAD, opacity=0.95, shadow=SH_E3)

    # live cards above the dock
    rect(360, 258, 292, 108, fill=PANEL, radius=12, stroke=HAIR, sw=1,
         shadow=SH_E1)
    rect(360, 258, 292, 30, fill=INK800, radius=12)
    text("SUPERAGENT · backend", 374, 266, 9, MONO, "SemiBold", "#FBF3DC",
         ls="6%")
    text("Maps the repository and splits work", 374, 300, 11, UI, "Regular",
         INK800)
    text("3 workers · 2 queued · routing healthy", 374, 318, 9, MONO,
         "Regular", INK500)
    dot(632, 274, 3.5, VERIFY)

    rect(1040, 236, 220, 76, fill=PANEL, radius=10, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text("WORKER · tailwind-builder", 1052, 246, 8.5, MONO, "SemiBold",
         INK800, ls="5%")
    text("editing 3 files · 41%", 1052, 264, 10, UI, "Regular", INK500)
    rect(1052, 288, 196, 5, fill=HAIR, radius=999)
    rect(1052, 288, 80, 5, fill=H400, radius=999)

    bee_route(652, 312, 780, 300, bee_at=0.55, size=12)
    bee_route(1040, 274, 900, 310, bee_at=0.45, size=12)

    # ---- the dock: main column width = 1128 at 1440 ----
    rect(312, 428, 1128, 560, fill=PANEL, radius=18, stroke=HAIR, sw=1,
         shadow=SH_E3, name="chat-dock")
    rect(844, 438, 64, 10, fill=HAIR, radius=999, name="dock-grip")
    text("grip 64x12, topmost hit target", 918, 436, 8, MONO, "Regular",
         H700, ls="4%")

    rect(313, 452, 1126, 50, fills=HEADER_GRAD, radius=14, name="dock-header")
    hex_icon(342, 477, 10, stroke=H400, sw=1.8)
    text("Hive AI Assistant", 364, 466, 15, BRAND, "SemiBold", "#FFFFFF")
    dot(1330, 477, 4, H300)
    text("IDLE", 1340, 471, 8.5, MONO, "SemiBold", H200, ls="10%")
    text("×", 1402, 463, 17, UI, "Regular", "#FBF3DC")

    # transcript
    ellipse(340, 522, 24, 24, fill=H200)
    text("You · 09:41", 372, 518, 8.5, MONO, "SemiBold", INK500, ls="4%")
    text("ILLUSTRATIVE COPY · NOT RUN DATA", 1090, 518, 7.5, MONO, "Regular",
         H700, ls="6%")
    rect(372, 534, 880, 44, fill=PANEL_ALT, radius=10, stroke=HAIR, sw=1)
    text("What changed in the new chat dock and Help?", 386, 546, 11.5, UI,
         "Regular", INK800, width=840)

    hex_mark(340, 592, 24)
    text("Hive AI Assistant · just now", 372, 588, 8.5, MONO, "SemiBold",
         INK500, ls="4%")
    text("The dock now spans the main column: 1128 px at 1440. It keeps a 420 px minimum, 560 px default and 760 px maximum; the optional tray adds 240 px. Help is a nav destination, not a dock tab.",
         372, 606, 11.5, UI, "Regular", INK800, width=880, lh=17)

    rect(372, 690, 880, 58, fill=ERROR_SURFACE, radius=10, stroke=SIGNAL,
         sw=1)
    text("REFUSED BY NAME · NOT SILENT", 386, 698, 8, MONO, "SemiBold",
         SIGNAL, ls="8%")
    text("E_CALL_LIMIT - the run reached its bounded provider/tool budget. Start a smaller run; do not retry blindly.",
         386, 714, 10.5, UI, "Regular", INK800, width=850, lh=14)
    text("REFUSAL", 1180, 698, 8, MONO, "SemiBold", SIGNAL, ls="10%")

    # session + composer
    rect(330, 762, 1076, 30, fill=RECESS, radius=6)
    text("SESSION & PERMISSIONS · CLASSIC", 342, 770, 8.5, MONO, "SemiBold",
         INK800, ls="6%")
    chevron(1390, 775, 7)

    rect(330, 800, 976, 64, fill=PANEL, radius=10, stroke=HAIR, sw=1)
    text("Type your prompt...", 346, 824, 13, UI, "Regular", INK500)
    rect(1184, 814, 36, 36, fill=PANEL, radius=999, stroke=HAIR, sw=1)
    vector(1196, 826, 12, 12, "M 2 12 L 12 2 M 5 2 L 12 2 L 12 9",
           fill="none", stroke=INK700, sw=1.4)
    rect(1226, 814, 36, 36, fill=PANEL, radius=999, stroke=HAIR, sw=1)
    rect(1237, 827, 14, 12, fill="none", radius=2, stroke=INK700, sw=1.4)
    rect(1268, 814, 36, 36, fill=H300, radius=999)
    vector(1281, 825, 12, 12, "M 0 6 L 12 0 L 8 12 L 5 7 Z", fill=INK900)

    text("ENTER SUBMITS · SHIFT+ENTER NEWLINE · CTRL+0 RECENTRES", 330, 872,
         8, MONO, "Regular", "#A0916F", ls="6%")
    rect(330, 892, 1078, 1, fill=HAIR)
    text("CONTROLS", 330, 906, 8.5, MONO, "SemiBold", INK800, ls="8%")
    chevron(392, 911, 6)
    text("STATE & SETTINGS", 430, 906, 8.5, MONO, "SemiBold", INK800, ls="8%")
    chevron(548, 911, 6)
    text("CLEAR", 1270, 906, 8.5, MONO, "Regular", INK700, ls="8%")
    text("HIDE", 1360, 906, 8.5, MONO, "Regular", INK700, ls="8%")
    text("tray +240 when open", 1000, 906, 8, MONO, "Regular", H700, ls="4%")
    rect(330, 934, 1078, 42, fill=PANEL_ALT, radius=6, stroke=HAIR, sw=1)
    text("DOCK GEOMETRY", 344, 943, 8, MONO, "SemiBold", H700, ls="10%")
    text("min 420 · default 560 · max 760 · bottom edge fixed during resize · 12 px viewport gap",
         344, 958, 9, MONO, "Regular", INK600, ls="2%")


def scene_b():
    y0 = 1024
    header(y0, "SCENE 13B / HELP")
    nav_rail(1076, 724, 7)
    library(1076, 724, compact=True)
    rect(312, 1076, 1128, 724, fill=CANVAS)

    # Help header
    rect(386, 1100, 64, 64, fill=H100, radius=18, stroke=BORDER_GOLD, sw=1)
    hex_icon(418, 1132, 19, stroke=INK800, sw=1.6)
    text("HELP & TUTORIALS", 466, 1100, 8.5, MONO, "SemiBold", H700, ls="14%")
    text("What do you want to do?", 466, 1116, 30, BRAND, "SemiBold", INK900)
    text("HiveMind is a local multi-agent workspace. The HyperAgent plans; SuperAgents split the work; Workers execute; Subagents help read-only. Every risky action is approved or refused by name, and every run is recorded locally.",
         466, 1158, 12, UI, "Regular", INK500, width=880, lh=17)

    tutorial_card(386, 1216, 300, 142, "First run", [
        "Choose a policy in SESSION & PERMISSIONS.",
        "Type a small task in the composer; press Enter.",
        "Watch the graph: HYPER plans, SUPER splits, WORKER executes.",
        "Approve or deny side effects inline.",
    ])
    tutorial_card(706, 1216, 300, 142, "The dock", [
        "Drag the top grip to resize the chat.",
        "CONTROLS opens spawn / re-route / cancel / kill.",
        "STATE & SETTINGS shows provider, model and key.",
        "HIDE removes it; the header Chat button brings it back.",
    ])
    tutorial_card(1026, 1216, 300, 142, "Safety", [
        "classic asks before side effects; plan refuses writes.",
        "standard allows workspace writes; trusted removes prompts.",
        "Never use trusted on a workspace you do not trust.",
        "Every refusal has a name; the run ledger keeps it.",
    ])

    features = [
        ("Workspace", "The live agent graph and the resizable chat dock."),
        ("Agents", "Roster, jobs, todos, memory, lessons, and proposals."),
        ("Data", "Runs, revisions, evidence, and snapshots."),
        ("Models", "Model catalogue, usage, cache counters, and balance."),
        ("Routing", "Which model or strategy handles a task."),
        ("Remote", "Remote principals and daemons are local records today."),
        ("Settings", "Provider profile, key status, diagnostics, and FigCraft."),
        ("FigCraft", "Design boards and the gallery; the live UI uses tokens."),
        ("Screen access", "Agents cannot see your desktop, browser, or screen."),
        ("Workspace", "The agent sees only the configured workspace root."),
    ]
    for i, (label, body) in enumerate(features):
        col, row = i % 2, i // 2
        feature_card(386 + col * 440, 1376 + row * 40, 420, 36, label, body)

    rect(386, 1584, 130, 36, fill=H300, radius=6, stroke=H500, sw=1)
    text("Go to Workspace", 400, 1594, 10.5, UI, "SemiBold", INK900)
    rect(528, 1584, 184, 36, fill=PANEL, radius=6, stroke=HAIR, sw=1)
    text("Provider & key settings", 542, 1594, 10.5, UI, "Regular", INK800)
    rect(724, 1584, 164, 36, fill=PANEL, radius=6, stroke=HAIR, sw=1)
    text("Agent roster & jobs", 738, 1594, 10.5, UI, "Regular", INK800)
    text("HiveMind · press ? in the header for the quick canvas guide.", 386,
         1644, 8.5, MONO, "Regular", INK500, ls="4%")

    # honest anatomy notes inside the scene
    rect(312, 1670, 1128, 118, fill=PANEL_ALT)
    vrule(312, 1670, 1, HAIR, 118)
    text("ANATOMY NOTES", 386, 1684, 8.5, MONO, "SemiBold", H700, ls="12%")
    text("Help is reached from the 72 px nav rail (Help item), not from the chat dock.",
         386, 1702, 10, UI, "Regular", INK600, width=980)
    text("This page is static copy: three tutorials, ten feature explanations and three working buttons. No dead controls.",
         386, 1720, 10, UI, "Regular", INK600, width=980)
    text("Source: web/src/panels/HelpPanel.tsx at commit e83a444. This board is a design proposal; it does not claim runtime behaviour.",
         386, 1738, 10, UI, "Regular", INK600, width=980)
    text("Source note: HelpPanel.tsx declares two features named 'Workspace' (lines 37 and 46); this board mirrors that label and does not fix the defect.",
         386, 1756, 9, UI, "Regular", INK500, width=980)
    text("PROPOSED · NOT WATCHDOG-AUDITED", 1140, 1684, 8, MONO, "SemiBold",
         SIGNAL, ls="8%")


def title_strip():
    rect(0, 0, W, 96, fill=PANEL_ALT)
    rect(0, 96, W, 1, fill=HAIR)
    hex_mark(32, 28, 38)
    text("HiveMind · FigCraft 13", 88, 24, 21, BRAND, "SemiBold", INK900)
    text("CHAT DOCK + HELP · NEW UI DESIGN PROPOSAL", 88, 54, 9.5, MONO,
         "SemiBold", INK500, ls="12%")
    text("dock 1128x560 (min 420 / max 760, tray +240) · rail 72 · library 240 · header 52 · illustrative values, no runtime data",
         88, 72, 8.5, MONO, "Regular", INK500, ls="2%")
    rect(1030, 26, 378, 40, fill=ERROR_SURFACE, radius=8, stroke=SIGNAL, sw=1)
    text("PROPOSED · NOT WATCHDOG-AUDITED · NOT SERVED", 1044, 39, 8.5, MONO,
         "SemiBold", SIGNAL, ls="6%")
    text("evidence: web/src/chat/ChatCore.tsx · chatDock.ts · web/src/panels/HelpPanel.tsx",
         1030, 74, 8, MONO, "Regular", INK500, ls="2%")


def divider():
    rect(0, 996, W, 28, fill=RECESS)
    text("02 · HELP — NAV DESTINATION, NOT A DOCK TAB", 24, 1002, 9, MONO,
         "SemiBold", INK800, ls="8%")
    text("source: web/src/panels/HelpPanel.tsx · no dead controls", 1040, 1002,
         8, MONO, "Regular", INK500, ls="4%")


def footer():
    rect(0, 1788, W, 12, fill=INK900)
    text("PROPOSED BOARD · generated by docs/ui/figcraft/ChatDock/build/build_chatdock.py · figcraft-cli 0.17.0 · no runtime data",
         16, 1789, 7.5, MONO, "Regular", "#FBF3DC", ls="2%")


def main():
    add("file.new", name="HiveMind Chat Dock & Help (proposed)")
    add("frame.create", x=0, y=0, width=W, height=H,
        name="13 · Chat Dock & Help", props={"fill": CANVAS})
    title_strip()
    scene_a()
    divider()
    scene_b()
    footer()
    add("file.saveAs", path=os.path.join(
        CHATDOCK, "ChatDock-Help.figcraft").replace("\\", "/"))
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="\n") as handle:
        json.dump(CMDS, handle, ensure_ascii=False, indent=1)
        handle.write("\n")
    print("wrote %s (%d commands)" % (OUT, len(CMDS)))


if __name__ == "__main__":
    main()
