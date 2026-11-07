#!/usr/bin/env python3
"""
HiveMind Spatial Style System -- deterministic FigCraft builder.

Emits FigCraft command scripts (the same JSON format `figcraft-cli run` executes)
in two phases:

  phase 1 : file.new + 8 artboard frames + every node -> base.figcraft
  phase 2 : native paint/text/effect styles + colour variables + exports -> final

Why two phases: `run` is transactional. A rejected style parameter in phase 2
rolls back the whole script, so the (large, expensive) node build is committed
first and only additive tokens are attempted second.

Node ids are assigned sequentially by FigCraft (page = 1, then +1 per created
layer), so the builder predicts every id and writes `expected_ids.json`; the
verification step compares prediction against the ids the CLI actually returned.

Usage:
  python build_hive_style.py 1   -> writes commands_phase1.json, expected_ids.json
  python build_hive_style.py 2   -> writes commands_phase2.json
"""
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
EXPORTS = os.path.join(ROOT, "exports")

# --------------------------------------------------------------------------
# Design tokens  (provenance recorded per swatch in the kit document)
# --------------------------------------------------------------------------
CANVAS = "#F8F4EB"        # sampled: dominant cream of the reference canvas
PANEL = "#FFFFFF"
PANEL_ALT = "#FBF8F1"     # derived 50/50 canvas+paper
RECESS = "#F1E9CF"        # sampled: pale honey inset
HAIR = "#E5E1D8"          # existing app token --color-hairline
HAIR_SOFT = "#EFE7D2"     # derived honeycomb line
BORDER_GOLD = "#E5C27A"
H100 = "#FBF3DC"
H200 = "#F6E7BE"
H300 = "#F0C050"          # sampled: bright honey highlight
H400 = "#E8B850"          # sampled: dominant honey-gold  (PRIMARY)
H500 = "#D9A02F"
H600 = "#C88030"          # sampled: amber shadow
H700 = "#8A5A1C"
H800 = "#604818"          # sampled: deep bronze
INK950 = "#14100A"
INK900 = "#282010"        # sampled: darkest warm ink
INK800 = "#3F3115"        # sampled: chat header
INK700 = "#4E3A15"        # sampled: header gradient end
INK600 = "#6B5A33"
INK500 = "#75663F"        # DERIVED: darkened until meta text passes AA on canvas (3.84 -> 5.02)
AMBER = "#B7791F"         # existing app token --color-amber
ERR = "#B23A2E"
OK = "#3F7A55"

BRAND = "Playfair Display"
UI = "Inter"
MONO = "Consolas"

SH_E1 = [{"type": "dropShadow", "offset": [0, 1], "radius": 3, "spread": 0,
          "color": "#28201012", "showBehind": True},
         {"type": "dropShadow", "offset": [0, 8], "radius": 24, "spread": 0,
          "color": "#28201014", "showBehind": True}]
SH_E2 = [{"type": "dropShadow", "offset": [0, 2], "radius": 6, "spread": 0,
          "color": "#28201014", "showBehind": True},
         {"type": "dropShadow", "offset": [0, 20], "radius": 48, "spread": 0,
          "color": "#2820101F", "showBehind": True}]
SH_E3 = [{"type": "dropShadow", "offset": [0, 0], "radius": 80, "spread": -10,
          "color": "#8A5A1C3D", "showBehind": True},
         {"type": "dropShadow", "offset": [0, 24], "radius": 48, "spread": 0,
          "color": "#60481833", "showBehind": True}]

GOLD_GRAD = [{"type": "radialGradient",
              "stops": ["#FFF8E5", "#F0C050", "#C88030"]}]
GOLD_GRAD_DEEP = [{"type": "radialGradient",
                   "stops": ["#F6E7BE", "#D9A02F", "#8A5A1C"]}]
HEADER_GRAD = [{"type": "linearGradient",
                "stops": ["#3F3115", "#604818"], "from": [0, 0], "to": [1, 1]}]
HONEY_GRAD = [{"type": "linearGradient",
               "stops": ["#F0C050", "#C88030"], "from": [0, 0], "to": [1, 1]}]

# --------------------------------------------------------------------------
# Command buffer
# --------------------------------------------------------------------------
CMDS = []
NEXT_ID = [2]          # page is id 1
FRAME_IDS = {}
EXPECTED = []


def nid(label=""):
    i = NEXT_ID[0]
    NEXT_ID[0] += 1
    EXPECTED.append({"id": i, "label": label})
    return i


def add(command, **params):
    CMDS.append({"command": command, "params": params})


def frame(name, x, y, w, h, fill=CANVAS):
    i = nid("frame:" + name)
    FRAME_IDS[name] = i
    add("frame.create", x=x, y=y, width=w, height=h, name=name,
        props={"fill": fill})
    return i


def node(parent, kind, x, y, w, h, name=None, **props):
    i = nid(kind)
    p = {"type": kind, "x": x, "y": y, "width": w, "height": h, "parent": parent}
    if name:
        p["name"] = name
    if props:
        p["props"] = props
    add("node.create", **p)
    return i


def rect(parent, x, y, w, h, fill=None, fills=None, radius=0, stroke=None,
         sw=1, shadow=None, name=None, rotation=None, dashes=None,
         stroke_align=None, opacity=None, clip=None, effects=None):
    p = {}
    if fills is not None:
        p["fills"] = fills
    elif fill is not None:
        p["fill"] = fill
    if radius:
        p["cornerRadius"] = radius
    if stroke:
        p["stroke"] = stroke
        p["strokeWeight"] = sw
    if stroke_align:
        p["strokeAlign"] = stroke_align
    if dashes:
        p["dashes"] = dashes
    if shadow:
        p["effects"] = shadow
    if effects:
        p["effects"] = effects
    if rotation is not None:
        p["rotation"] = rotation
    if opacity is not None:
        p["opacity"] = opacity
    if clip is not None:
        p["clip"] = clip
    return node(parent, "rectangle", x, y, w, h, name, **p)


def ellipse(parent, x, y, w, h, fill=None, fills=None, stroke=None, sw=1,
            shadow=None, name=None, rotation=None, ring=False):
    p = {}
    if fills is not None:
        p["fills"] = fills
    elif fill is not None:
        p["fill"] = fill
    if ring:
        p["fill"] = "none"
    if stroke:
        p["stroke"] = stroke
        p["strokeWeight"] = sw
    if shadow:
        p["effects"] = shadow
    if rotation is not None:
        p["rotation"] = rotation
    return node(parent, "ellipse", x, y, w, h, name, **p)


def vector(parent, x, y, w, h, path, fill=None, stroke=None, sw=1,
           name=None, shadow=None, dashes=None, opacity=None,
           stroke_cap=None, stroke_join=None):
    p = {"path": path}
    if fill is not None:
        p["fill"] = fill
    if stroke:
        p["stroke"] = stroke
        p["strokeWeight"] = sw
    if dashes:
        p["dashes"] = dashes
    if shadow:
        p["effects"] = shadow
    if opacity is not None:
        p["opacity"] = opacity
    if stroke_cap:
        p["strokeCap"] = stroke_cap
    if stroke_join:
        p["strokeJoin"] = stroke_join
    return node(parent, "vector", x, y, w, h, name, **p)


def text(parent, s, x, y, size=15, font=UI, style="Regular", color=INK900,
         width=None, lh=None, ls=None, align=None, name=None,
         decoration=None, case=None):
    i = nid("text")
    props = {"fontSize": size, "fontFamily": font, "fontStyle": style,
             "fill": color}
    if lh:
        props["lineHeight"] = lh
    if ls:
        props["letterSpacing"] = ls
    if align:
        props["textAlign"] = align
    if decoration:
        props["textDecoration"] = decoration
    if case:
        props["textCase"] = case
    p = {"text": s, "x": x, "y": y, "parent": parent, "props": props}
    if width is not None:
        p["width"] = width
        props["textAutoResize"] = "height"
    if name:
        p["name"] = name
    add("text.create", **p)
    return i


def rule(parent, x, y, w, color=HAIR, weight=1):
    i = nid("rule")
    add("node.create", type="line", x=x, y=y, width=w, parent=parent,
        props={"stroke": color, "strokeWeight": weight})
    return i


def dot(parent, cx, cy, r, color):
    return ellipse(parent, cx - r, cy - r, r * 2, r * 2, fill=color)


# --------------------------------------------------------------------------
# Geometry helpers
# --------------------------------------------------------------------------
def hex_path(R):
    """Unit pointy-top hexagon path in a (sqrt3*R x 2R) box."""
    px = math.sqrt(3) * R
    pts = [(0, 0.5 * R), (px / 2, 0), (px, 0.5 * R),
           (px, 1.5 * R), (px / 2, 2 * R), (0, 1.5 * R)]
    return "M " + " L ".join(f"{a:.2f} {b:.2f}" for a, b in pts) + " Z", px, 2 * R


def honeycomb(parent, x, y, w, h, R=34, color=HAIR_SOFT, weight=1):
    """One vector node containing the whole honeycomb lattice (cheap, precise)."""
    px = math.sqrt(3) * R
    py = 1.5 * R
    cols = int(math.ceil(w / px)) + 2
    rows = int(math.ceil(h / py)) + 2
    centers = []
    for r in range(rows):
        for c in range(cols):
            cx = c * px + (px / 2 if r % 2 else 0)
            cy = r * py
            centers.append((cx, cy))
    minx = min(c[0] for c in centers) - px / 2
    miny = min(c[1] for c in centers) - R
    maxx = max(c[0] for c in centers) + px / 2
    maxy = max(c[1] for c in centers) + R
    parts = []
    for cx, cy in centers:
        ox, oy = cx - px / 2 - minx, cy - R - miny
        pts = [(0, 0.5 * R), (px / 2, 0), (px, 0.5 * R),
               (px, 1.5 * R), (px / 2, 2 * R), (0, 1.5 * R)]
        parts.append("M " + " L ".join(f"{ox + a:.1f} {oy + b:.1f}"
                                       for a, b in pts) + " Z")
    return vector(parent, x + minx, y + miny, maxx - minx, maxy - miny,
                  " ".join(parts), fill="none", stroke=color, sw=weight,
                  name="honeycomb")


def hexmark(parent, x, y, size, face=INK900, glyph=H400, stroke=None,
            wing=H200):
    """Hexagon brand mark with a bee glyph centred inside."""
    p, w, h = hex_path(size / 2)
    vector(parent, x, y, w, h, p, fill=face,
           stroke=stroke, sw=1.5, name="hex-mark")
    s = size * 0.50
    bee(parent, x + 0.105 * size, y + 0.375 * size, s, body=glyph,
        stripe=face, wing=wing, simple=True)


def bee(parent, x, y, s, body=INK900, stripe=H400, wing="#F6E7BE",
        simple=False, facing=1):
    """Flight-vector bee: abdomen + thorax + head, three bands, two angled wings.

    Geometry was selected by rendering six candidates at brand (96px) and UI
    (16-24px) sizes. `facing=-1` mirrors the glyph so the head points along -x,
    which lets bee_route() orient the bee to its direction of travel.
    Body spans x..x+1.26s, y-0.20s..y+0.68s.
    """
    BW = 1.26 * s

    def X(ox, w):
        return x + (BW - ox - w if facing < 0 else ox)

    rot = -32 if facing > 0 else 32
    rot2 = -48 if facing > 0 else 48
    # wings sit behind the body, angled up from the thorax
    ellipse(parent, X(s * 0.60, s * 0.52), y - s * 0.14, s * 0.52, s * 0.26,
            fill=wing, rotation=rot)
    ellipse(parent, X(s * 0.86, s * 0.46), y - s * 0.20, s * 0.46, s * 0.24,
            fill=wing, rotation=rot2)
    # abdomen / thorax / head
    ellipse(parent, X(0, s * 0.72), y + s * 0.26, s * 0.72, s * 0.44, fill=body)
    ellipse(parent, X(s * 0.58, s * 0.42), y + s * 0.22, s * 0.42, s * 0.42,
            fill=body)
    ellipse(parent, X(s * 0.96, s * 0.30), y + s * 0.26, s * 0.30, s * 0.30,
            fill=body)
    # abdominal bands
    rect(parent, X(s * 0.10, s * 0.08), y + s * 0.28, s * 0.08, s * 0.40,
         fill=stripe)
    rect(parent, X(s * 0.30, s * 0.08), y + s * 0.26, s * 0.08, s * 0.44,
         fill=stripe)
    rect(parent, X(s * 0.50, s * 0.06), y + s * 0.26, s * 0.06, s * 0.40,
         fill=stripe)


def bee_route(parent, x1, y1, x2, y2, color=H400, dashes=(10, 10),
              bee_at=0.5, bee_size=16, arrow=True):
    """Dashed honey-gold routing vector with a bee flying along it."""
    w = x2 - x1
    h = y2 - y1
    p = f"M 0 0 L {abs(w):.1f} {abs(h):.1f}" if (w >= 0 and h >= 0) else \
        f"M 0 {abs(h):.1f} L {abs(w):.1f} 0"
    x0 = min(x1, x2)
    y0 = min(y1, y2)
    vector(parent, x0, y0, abs(w) or 1, abs(h) or 1, p, fill="none",
           stroke=color, sw=1.5, dashes=list(dashes), name="bee-route")
    bx = x1 + w * bee_at
    by = y1 + h * bee_at
    # the bee must face its direction of travel (spec §2)
    bee(parent, bx - bee_size * 0.4, by - bee_size * 0.5, bee_size,
        facing=(-1 if w < 0 else 1))
    if arrow:
        ang = math.atan2(h, w)
        ax, ay = x2, y2
        tri = [(0, -4), (9, 0), (0, 4)]
        ca, sa = math.cos(ang), math.sin(ang)
        pts = [(ax + px * ca - py * sa, ay + px * sa + py * ca)
               for px, py in tri]
        d = "M " + " L ".join(f"{a:.1f} {b:.1f}" for a, b in pts) + " Z"
        vector(parent, ax - 4.5, ay - 4, 10, 8, d, fill=color)


def chevron(parent, x, y, s=8, color=INK500, direction="down"):
    if direction == "down":
        p = f"M 0 0 L {s/2:.1f} {s*0.6:.1f} L {s:.1f} 0"
    else:
        p = f"M 0 {s*0.6:.1f} L {s/2:.1f} 0 L {s:.1f} {s*0.6:.1f}"
    vector(parent, x, y, s, s * 0.7, p, fill="none", stroke=color, sw=1.5)


def check(parent, x, y, s=10, color=OK):
    vector(parent, x, y, s, s, f"M 0 {s*0.55:.1f} L {s*0.38:.1f} {s:.1f} L {s:.1f} 0",
           fill="none", stroke=color, sw=1.8)


def warn_icon(parent, x, y, s=12, color=ERR):
    vector(parent, x, y, s, s * 0.9,
           f"M {s/2:.1f} 0 L {s:.1f} {s*0.9:.1f} L 0 {s*0.9:.1f} Z",
           fill="none", stroke=color, sw=1.5)
    rect(parent, x + s / 2 - 0.7, y + s * 0.3, 1.4, s * 0.3, fill=color)
    dot(parent, x + s / 2, y + s * 0.74, 0.8, color)


def icon_search(parent, x, y, s=18, color=INK900):
    ellipse(parent, x, y, s * 0.72, s * 0.72, fill="none", stroke=color, sw=1.6)
    rule(parent, x + s * 0.58, y + s * 0.58, s * 0.42, color, 1.6)


def icon_bell(parent, x, y, s=18, color=INK900):
    vector(parent, x, y, s, s,
           f"M {s*0.16:.1f} {s*0.64:.1f} L {s*0.84:.1f} {s*0.64:.1f} "
           f"L {s*0.72:.1f} {s*0.48:.1f} L {s*0.72:.1f} {s*0.40:.1f} "
           f"A {s*0.22:.1f} {s*0.22:.1f} 0 0 0 {s*0.28:.1f} {s*0.40:.1f} "
           f"L {s*0.28:.1f} {s*0.48:.1f} Z", fill="none", stroke=color, sw=1.5)
    rule(parent, x + s * 0.42, y + s * 0.86, s * 0.16, color, 1.5)


def icon_layers(parent, x, y, s=18, color=INK900):
    vector(parent, x, y, s, s,
           f"M {s/2:.1f} 0 L {s:.1f} {s*0.3:.1f} L {s/2:.1f} {s*0.6:.1f} L 0 {s*0.3:.1f} Z "
           f"M 0 {s*0.55:.1f} L {s/2:.1f} {s*0.85:.1f} L {s:.1f} {s*0.55:.1f}",
           fill="none", stroke=color, sw=1.4)


def icon_hex(parent, x, y, s=18, color=INK900):
    p, w, h = hex_path(s / 2)
    vector(parent, x, y, w, h, p, fill="none", stroke=color, sw=1.5)


def icon_cube(parent, x, y, s=18, color=INK900):
    h2 = s * 0.5
    vector(parent, x, y + h2 * 0.32, s, s * 0.82,
           f"M {s/2:.1f} 0 L {s:.1f} {s*0.22:.1f} L {s:.1f} {s*0.66:.1f} "
           f"L {s/2:.1f} {s*0.88:.1f} L 0 {s*0.66:.1f} L 0 {s*0.22:.1f} Z "
           f"M {s/2:.1f} 0 L {s/2:.1f} {s*0.44:.1f} M 0 {s*0.22:.1f} L {s/2:.1f} {s*0.44:.1f} "
           f"L {s:.1f} {s*0.22:.1f}",
           fill="none", stroke=color, sw=1.4)


def icon_fork(parent, x, y, s=18, color=INK900):
    vector(parent, x, y, s, s,
           f"M {s*0.5:.1f} {s:.1f} L {s*0.5:.1f} {s*0.5:.1f} "
           f"M {s*0.5:.1f} {s*0.5:.1f} L {s*0.12:.1f} {s*0.12:.1f} "
           f"M {s*0.5:.1f} {s*0.5:.1f} L {s*0.88:.1f} {s*0.12:.1f}",
           fill="none", stroke=color, sw=1.5)
    dot(parent, x + s * 0.5, y + s, 2, color)
    dot(parent, x + s * 0.12, y + s * 0.12, 2, color)
    dot(parent, x + s * 0.88, y + s * 0.12, 2, color)


def icon_gear(parent, x, y, s=18, color=INK900):
    ellipse(parent, x + s * 0.15, y + s * 0.15, s * 0.7, s * 0.7,
            fill="none", stroke=color, sw=1.5)
    dot(parent, x + s * 0.44, y + s * 0.44, 2.2, color)
    for k in range(8):
        a = math.pi * k / 4
        cx = x + s / 2 + math.cos(a) * s * 0.44
        cy = y + s / 2 + math.sin(a) * s * 0.44
        rect(parent, cx - 1, cy - 1, 2, 2, fill=color)


def icon_send(parent, x, y, s=16, color=INK600):
    vector(parent, x, y, s, s,
           f"M {s*0.06:.1f} {s*0.06:.1f} L {s:.1f} {s*0.5:.1f} L {s*0.06:.1f} {s*0.94:.1f} "
           f"L {s*0.28:.1f} {s*0.5:.1f} Z", fill=color)


def icon_mic(parent, x, y, s=16, color=INK600):
    rect(parent, x + s * 0.32, y, s * 0.32, s * 0.55, fill="none", radius=3,
         stroke=color, sw=1.4)
    vector(parent, x + s * 0.18, y + s * 0.36, s * 0.64, s * 0.62,
           f"M 0 0 A {s*0.32:.1f} {s*0.32:.1f} 0 0 0 {s*0.64:.1f} 0",
           fill="none", stroke=color, sw=1.4)
    rule(parent, x + s * 0.5, y + s * 0.68, s * 0.22, color, 1.4)


def icon_clip(parent, x, y, s=16, color=INK600):
    vector(parent, x, y, s, s,
           f"M {s*0.58:.1f} {s*0.28:.1f} L {s*0.24:.1f} {s*0.62:.1f} "
           f"A {s*0.24:.1f} {s*0.24:.1f} 0 0 0 {s*0.58:.1f} {s*0.96:.1f} "
           f"L {s*0.92:.1f} {s*0.62:.1f} L {s*0.92:.1f} {s*0.40:.1f} "
           f"A {s*0.24:.1f} {s*0.24:.1f} 0 0 0 {s*0.44:.1f} {s*0.40:.1f} "
           f"L {s*0.44:.1f} {s*0.74:.1f}",
           fill="none", stroke=color, sw=1.3)


# --------------------------------------------------------------------------
# Shared board furniture
# --------------------------------------------------------------------------
def board_header(f, num, kicker, title, sub):
    text(f, f"{num} / {kicker}", 72, 46, 11, MONO, "Regular", INK500, ls="14%")
    text(f, title, 72, 66, 52, BRAND, "SemiBold", INK900, lh="104%")
    rule(f, 72, 152, 1296, HAIR, 1)
    text(f, sub, 72, 168, 15, UI, "Regular", INK600, width=780, lh="150%")


def close_space(f, w, h, label=""):
    """Dashed argument margin, used on component boards."""
    vector(f, 32, 32, w - 64, h - 64,
           f"M 0 0 L {w-64} 0 L {w-64} {h-64} L 0 {h-64} Z",
           fill="none", stroke="#D9CFB8", sw=1, dashes=[6, 6])
    if label:
        text(f, label, 36, 12, 9, MONO, "Regular", "#A99B7C", ls="12%")


# --------------------------------------------------------------------------
# BOARD 00 -- Cover
# --------------------------------------------------------------------------
def board_cover(f):
    W, H = 1440, 900
    honeycomb(f, -20, -20, W + 40, H + 40, R=40, color=HAIR_SOFT, weight=1)
    rect(f, 0, 0, W, H, fills=GOLD_GRAD_DEEP, opacity=0.05)

    hexmark(f, 72, 64, 52)
    text(f, "HiveMind", 136, 70, 26, UI, "Bold", INK900)
    text(f, "SPATIAL EXECUTION ENVIRONMENT", 137, 100, 9.5, MONO, "Regular",
         INK500, ls="18%")

    text(f, "DESIGN SYSTEM  /  V1.0  /  FIGCRAFT NATIVE", 72, 208, 11, MONO,
         "Regular", AMBER, ls="14%")
    text(f, "Route agents", 72, 236, 88, BRAND, "SemiBold", INK900, lh="98%")
    text(f, "like a hive.", 72, 322, 88, BRAND, "SemiBold", INK900, lh="98%")

    text(f,
         "A spatial execution environment where code routing becomes a living "
         "honeycomb topology: bee-line vectors, sub-hives and worker nodes you "
         "can drag, re-parent and re-route by hand.",
         72, 436, 16.5, UI, "Regular", INK600, width=520, lh="165%")

    chips = ["Honeycomb canvas", "Bee-line routing", "Hive chat core"]
    cx = 72
    for c in chips:
        wid = 20 + len(c) * 7.0
        rect(f, cx, 572, wid, 32, fill=PANEL, radius=999, stroke=BORDER_GOLD, sw=1)
        text(f, c, cx + 12, 580, 12, UI, "Medium", INK700)
        cx += wid + 12

    rule(f, 72, 800, 500, HAIR, 1)
    text(f, "v1.0.0  ·  built with figcraft-cli  ·  deterministic, zero GUI", 72, 816,
         10.5, MONO, "Regular", INK500, ls="4%")
    text(f, "12 artboards  ·  20 paint styles  ·  13 text styles  ·  3 depth styles  ·  21 colour variables",
         72, 840, 10.5, MONO, "Regular", INK500, ls="2%")

    # hive illustration
    hive_core(f, 720, 96, 640, 620)
    bee(f, 640, 300, 26)
    bee(f, 1268, 520, 22)
    bee_route(f, 1010, 250, 1300, 120, bee_at=0.45, bee_size=18)

    # floating micro-card to sell the spatial idea
    rect(f, 1112, 560, 250, 104, fill=PANEL, radius=14, shadow=SH_E2)
    dot(f, 1132, 588, 5, H400)
    text(f, "WorkerAgent", 1146, 578, 13, UI, "SemiBold", INK900)
    text(f, "tailwind-builder  ·  running", 1146, 598, 10, MONO, "Regular", INK500)
    rect(f, 1128, 622, 218, 5, fill=HAIR, radius=999)
    rect(f, 1128, 622, 138, 5, fills=HONEY_GRAD, radius=999)
    text(f, "62%", 1310, 632, 9.5, MONO, "Regular", INK500)


def hive_core(f, x, y, w, h):
    """Stacked beehive containment: 5 gold lobes + glow."""
    lobes = [
        (0.16, 0.02, 0.68, 0.26, "#FFF8E5"),
        (0.06, 0.20, 0.88, 0.30, "#F6E7BE"),
        (0.00, 0.39, 1.00, 0.34, "#F0C050"),
        (0.06, 0.60, 0.88, 0.30, "#D9A02F"),
        (0.16, 0.78, 0.68, 0.22, "#8A5A1C"),
    ]
    for i, (lx, ly, lw, lh, edge) in enumerate(lobes):
        ellipse(f, x + w * lx, y + h * ly, w * lw, h * lh,
                fills=[{"type": "radialGradient",
                        "stops": ["#FFFDF4", edge, "#8A5A1C"]}],
                stroke="#7A4E14", sw=1.6, shadow=SH_E3)


# --------------------------------------------------------------------------
# BOARD 01 -- Colour
# --------------------------------------------------------------------------
COLORS = [
    ("Canvas", CANVAS, "surface/canvas", "reference canvas"),
    ("Paper", PANEL, "surface/panel", "cards, inputs, chat body"),
    ("Panel Alt", PANEL_ALT, "surface/panel-alt", "sidebar panel"),
    ("Recess", RECESS, "surface/recessed", "inset / quote / code gutter"),
    ("Hairline", HAIR, "border/hairline", "app token, kept"),
    ("Honey 100", H100, "honey/100", "assistant bubble, soft wash"),
    ("Honey 200", H200, "honey/200", "hover wash, bee wings"),
    ("Honey 300", H300, "honey/300", "highlight, focus glow"),
    ("Honey 400", H400, "honey/400", "PRIMARY accent"),
    ("Honey 500", H500, "honey/500", "pressed / border strong"),
    ("Honey 600", H600, "honey/600", "shadow ramp, gradient end"),
    ("Honey 700", H700, "honey/700", "deep outline"),
    ("Honey 800", H800, "honey/800", "bronze, header gradient end"),
    ("Ink 900", INK900, "ink/900", "primary text, code surface"),
    ("Ink 800", INK800, "ink/800", "chat header"),
    ("Ink 700", INK700, "ink/700", "header gradient"),
    ("Ink 500", INK500, "ink/500", "meta, captions"),
    ("Amber", AMBER, "accent/amber", "app token, links"),
    ("Signal", ERR, "state/error", "failure, broken route"),
    ("Verify", OK, "state/success", "passed gate"),
]


def lum(h):
    c = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    c = [x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4 for x in c]
    return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]


def ratio(a, b):
    la, lb = sorted([lum(a), lum(b)], reverse=True)
    return (la + 0.05) / (lb + 0.05)


def board_color(f):
    board_header(f, "01", "FOUNDATIONS", "Colour",
                 "Warm ink on honey paper. Every value is either sampled from the "
                 "supplied canvas reference or inherited from the shipped app "
                 "tokens, or derived and recorded as derived. Token names map 1:1 to FigCraft "
                 "colour variables.")
    x0, y0 = 72, 230
    cw, ch, gap = 240, 118, 20
    for i, (name, hexv, token, role) in enumerate(COLORS):
        col, row = i % 5, i // 5
        x = x0 + col * (cw + gap)
        y = y0 + row * 200
        rect(f, x, y, cw, ch, fill=hexv, radius=12, stroke=HAIR, sw=1)
        if hexv in (PANEL, PANEL_ALT, CANVAS):
            rect(f, x, y, cw, ch, fill="none", radius=12, stroke=HAIR, sw=1)
        text(f, name, x, y + ch + 10, 13, UI, "SemiBold", INK900)
        text(f, hexv.upper(), x, y + ch + 30, 11, MONO, "Regular", INK700)
        text(f, token, x, y + ch + 48, 10, MONO, "Regular", INK500)

    # ramps
    ry = 1064
    text(f, "RAMP / HONEY", 72, ry - 22, 9.5, MONO, "Regular", INK500, ls="14%")
    ramps = [H100, H200, H300, H400, H500, H600, H700, H800]
    for i, c in enumerate(ramps):
        rect(f, 72 + i * 52, ry, 52, 44, fill=c)
    text(f, "RAMP / INK", 560, ry - 22, 9.5, MONO, "Regular", INK500, ls="14%")
    for i, c in enumerate([INK500, INK600, INK700, INK800, INK900]):
        rect(f, 560 + i * 52, ry, 52, 44, fill=c)

    pairs = [(INK900, CANVAS), (INK900, H400), ("#FFFFFF", INK800),
             (AMBER, CANVAS), (ERR, CANVAS), (OK, CANVAS)]
    labels = ["ink-900 / canvas", "ink-900 / honey-400", "white / ink-800",
              "amber / canvas", "error / canvas", "verify / canvas"]
    text(f, "WCAG 2.1 CONTRAST / MEASURED", 900, ry - 22, 9.5, MONO, "Regular",
         INK500, ls="14%")
    for i, ((a, b), lab) in enumerate(zip(pairs, labels)):
        r = ratio(a, b)
        col = i % 2
        row = i // 2
        x = 900 + col * 240
        y = ry + row * 26
        rect(f, x, y + 3, 10, 10, fill=a, radius=2, stroke=HAIR, sw=1)
        text(f, f"{lab}", x + 18, y, 9.5, MONO, "Regular", INK600)
        verdict = "AAA" if r >= 7 else ("AA" if r >= 4.5 else "AA-LARGE")
        text(f, f"{r:.2f}:1  {verdict}", x + 18, y + 12, 9.5, MONO, "SemiBold",
             OK if r >= 4.5 else ERR)


# --------------------------------------------------------------------------
# BOARD 02 -- Type
# --------------------------------------------------------------------------
TYPE_ROWS = [
    ("display/xxl", "HiveMind", 96, BRAND, "SemiBold", "96 / 100 / -3%", INK900),
    ("display/l", "Spatial routing", 56, BRAND, "SemiBold", "56 / 60 / -2%", INK900),
    ("heading/h1", "Agent Components", 40, UI, "SemiBold", "40 / 48 / -1.5%", INK900),
    ("heading/h3", "SuperAgent - Backend Architecture", 24, UI, "SemiBold",
     "24 / 32 / -0.5%", INK900),
    ("body/l", "Drop a repository into the core hive and the HyperAgent maps its "
               "architecture before a single worker spawns.", 17, UI, "Regular",
     "17 / 28 / 0", INK700),
    ("label", "ACTIVE AGENTS", 12, UI, "SemiBold", "12 / 16 / +8%", INK600),
    ("mono/code", "const route = hive.snap(worker, superAgentB);", 13, MONO,
     "Regular", "13 / 22 / 0", INK800),
    ("mono/meta", "NODE_SPAWN  ·  NODE_DRAG  ·  NODE_REWIRE", 11, MONO,
     "Regular", "11 / 16 / +8%", INK500),
]


def board_type(f):
    board_header(f, "02", "FOUNDATIONS", "Typography",
                 "Two working families plus one mono. Declared app fonts were "
                 "audited against what this machine actually resolves -- see the "
                 "availability strip at the bottom.")
    fams = [
        ("Playfair Display", BRAND, "SemiBold", "DISPLAY",
         "Brand statements, board titles, cover"),
        ("Inter", UI, "SemiBold", "UI / TEXT",
         "Every control, card, table and body string"),
        ("Consolas", MONO, "Regular", "MONO / CODE",
         "Code, tokens, hex values, event names"),
    ]
    for i, (name, fam, st, role, use) in enumerate(fams):
        x = 72 + i * 440
        rect(f, x, 230, 400, 300, fill=PANEL, radius=16, stroke=HAIR, sw=1,
             shadow=SH_E1)
        text(f, "Aa", x + 24, 252, 120, fam, st, INK900, lh="100%")
        text(f, name, x + 24, 404, 16, UI, "SemiBold", INK900)
        text(f, f"{role}  ·  {st}", x + 24, 430, 10, MONO, "Regular", INK500,
             ls="10%")
        text(f, use, x + 24, 456, 12, UI, "Regular", INK600, width=340)
        text(f, fam.lower().replace(" ", "-"), x + 24, 490, 9.5, MONO,
             "Regular", INK500)

    y = 566
    for token, sample, size, fam, st, meta, color in TYPE_ROWS:
        parts = meta.split(" / ")
        lh_pct = int(round(float(parts[1]) / size * 100))
        text(f, token, 72, y + 4, 10, MONO, "Regular", INK500)
        text(f, sample, 300, y, size, fam, st, color,
             width=760, lh=f"{lh_pct}%", ls=parts[-1])
        text(f, meta, 1130, y + 4, 10, MONO, "Regular", INK500)
        row_h = max(float(parts[1]) + 26, 60)
        rule(f, 72, y + row_h - 12, 1296, HAIR, 1)
        y += row_h

    # availability strip
    sy = 1200
    rect(f, 72, sy, 1296, 74, fill=RECESS, radius=12)
    text(f, "FONT AVAILABILITY AUDIT", 92, sy + 12, 9.5, MONO, "SemiBold",
         INK700, ls="14%")
    text(f, "RESOLVED HERE: Playfair Display  ·  Inter  ·  Consolas", 92, sy + 32,
         11, MONO, "Regular", INK700)
    text(f, "NOT INSTALLED HERE: JetBrains Mono.  The app self-hosts Instrument Serif"
            " (@fontsource, 8 woff files in web/dist) so the product resolves it, but this"
            " FigCraft machine cannot -- Playfair stands in on the boards.",
         92, sy + 50, 10.5, MONO, "Regular", ERR, width=1240, lh="145%")


# --------------------------------------------------------------------------
# BOARD 03 -- Space & depth
# --------------------------------------------------------------------------
SPACE = [4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96]
RADII = [4, 8, 12, 16, 24, 48]


def board_space(f):
    board_header(f, "03", "FOUNDATIONS", "Space & depth",
                 "A 4px base, six radii and three elevation tiers. Shadows are "
                 "warm-ink (never neutral grey) so depth stays inside the honey "
                 "palette.")

    text(f, "SPACING / 4PX BASE", 72, 208, 10, MONO, "Regular", INK500, ls="14%")
    base = 540
    for i, v in enumerate(SPACE):
        x = 72 + i * 92
        h = v * 2.6
        rect(f, x, base - h, 54, h, fills=HONEY_GRAD, radius=3)
        text(f, str(v), x, base + 10, 10.5, MONO, "Regular", INK700)
    rule(f, 72, base, 1220, HAIR, 1)

    text(f, "RADIUS", 72, 620, 10, MONO, "Regular", INK500, ls="14%")
    for i, r in enumerate(RADII):
        x = 72 + i * 130
        rect(f, x, 648, 100, 100, fill=PANEL, radius=r, stroke=H500, sw=1.4)
        text(f, str(r), x, 758, 10.5, MONO, "Regular", INK700)
    text(f, "999", 72 + 6 * 130, 648, 10.5, MONO, "Regular", INK500)

    text(f, "ELEVATION", 940, 620, 10, MONO, "Regular", INK500, ls="14%")
    elev = [("e1 / node", SH_E1, "cards resting on canvas"),
            ("e2 / float", SH_E2, "chat core, drawers, menus"),
            ("e3 / hive", SH_E3, "the hive itself, behind the chat")]
    for i, (name, sh, use) in enumerate(elev):
        y = 648 + i * 118
        rect(f, 940, y, 240, 88, fill=PANEL, radius=14, shadow=sh)
        text(f, name, 1196, y + 16, 11, MONO, "SemiBold", INK800)
        text(f, use, 1196, y + 36, 10, UI, "Regular", INK500, width=170, lh="140%")
        text(f, "warm ink  #282010 @ 7-13%", 1196, y + 74, 9, MONO, "Regular", INK500)


# --------------------------------------------------------------------------
# BOARD 04 -- Brand lockups
# --------------------------------------------------------------------------
def wordmark(f, x, y, size=44, color=INK900, tag=True, tagcolor=INK500):
    text(f, "HiveMind", x, y, size, UI, "Bold", color)
    if tag:
        text(f, "SPATIAL EXECUTION ENVIRONMENT", x + 2, y + size + 8,
             size * 0.19, MONO, "Regular", tagcolor, ls="18%")


def board_brand(f):
    board_header(f, "04", "BRAND", "Lockups",
                 "One mark, four lockups, one clear-space rule. The bee always "
                 "flies toward the route direction -- it is a routing arrow, not "
                 "decoration.")

    # primary
    text(f, "PRIMARY / ON PAPER", 72, 232, 10, MONO, "Regular", INK500, ls="14%")
    hexmark(f, 72, 260, 76)
    wordmark(f, 168, 274, 44)

    # icon
    text(f, "ICON / APP", 560, 232, 10, MONO, "Regular", INK500, ls="14%")
    hexmark(f, 560, 260, 128)
    rect(f, 722, 260, 128, 128, fill=CANVAS, radius=28, stroke=HAIR, sw=1)
    hexmark(f, 742, 280, 88)

    # mono
    text(f, "MONO / ONE COLOUR", 900, 232, 10, MONO, "Regular", INK500, ls="14%")
    hexmark(f, 900, 260, 76, face=INK900, glyph="#FFFFFF", wing="#FFFFFF")
    wordmark(f, 996, 274, 44, color=INK900, tagcolor=INK700)

    # inverse
    text(f, "INVERSE / ON INK", 72, 468, 10, MONO, "Regular", INK500, ls="14%")
    rect(f, 72, 496, 780, 232, fill=INK900, radius=20)
    honeycomb(f, 84, 508, 756, 208, R=34, color="#FFFFFF14", weight=1)
    hexmark(f, 112, 568, 76, face=H400, glyph=INK900, stroke=H300)
    wordmark(f, 208, 582, 44, color="#FFFFFF", tagcolor=H300)

    # clear space
    text(f, "CLEAR SPACE = 0.25 x MARK HEIGHT", 72, 764, 10, MONO, "Regular",
         INK500, ls="14%")
    rect(f, 72, 792, 320, 180, fill=PANEL, radius=16, stroke=HAIR, sw=1)
    vector(f, 88, 808, 288, 148,
           "M 0 0 L 288 0 L 288 148 L 0 148 Z", fill="none",
           stroke=BORDER_GOLD, sw=1, dashes=[5, 5])
    hexmark(f, 110, 830, 68)
    wordmark(f, 196, 840, 26, tag=False)

    # construction
    text(f, "BEE GLYPH / CONSTRUCTION", 460, 764, 10, MONO, "Regular", INK500,
         ls="14%")
    rect(f, 460, 792, 320, 180, fill=PANEL, radius=16, stroke=HAIR, sw=1)
    honeycomb(f, 470, 802, 300, 160, R=22, color="#E8D9B4", weight=1)
    bee(f, 528, 856, 88)

    # minimum sizes
    text(f, "MINIMUM SIZES", 840, 764, 10, MONO, "Regular", INK500, ls="14%")
    rect(f, 840, 792, 528, 180, fill=PANEL, radius=16, stroke=HAIR, sw=1)
    hexmark(f, 864, 816, 56)
    text(f, "56px", 864, 884, 10, MONO, "Regular", INK500)
    hexmark(f, 1000, 828, 32)
    text(f, "32px", 1000, 884, 10, MONO, "Regular", INK500)
    _ph, _pw, _phh = hex_path(8.0)
    vector(f, 1108, 836, _pw, _phh, _ph, fill=INK900)
    text(f, "16px", 1108, 884, 10, MONO, "Regular", INK500)
    text(f, "Never below 16px. Below 24px drop the inner bee\nand keep the hexagon "
            "silhouette only.", 864, 908, 10.5, UI, "Regular", INK600, width=300,
         lh="150%")
    text(f, "Wordmark min 20px. Mono 1-colour lockup is the\nfallback for "
            "single-ink print and embroidery.", 1196, 908, 10.5, UI, "Regular",
         INK600, width=170, lh="150%")


WORK_MONO = "#FFFFFF"


# --------------------------------------------------------------------------
# Agent components
# --------------------------------------------------------------------------
def chat_card(f, x, y, w, h):
    """HyperAgent core -- the hive chat UI."""
    rect(f, x, y, w, h, fill=CANVAS, radius=18, shadow=SH_E2)
    # header
    rect(f, x, y, w, 44, fills=HEADER_GRAD, radius=[18, 18, 0, 0])
    bee(f, x + 14, y + 13, 18, body=H300, stripe=H800, wing="#FFFFFF")
    text(f, "Hive AI Assistant", x + 44, y + 13, 14, UI, "SemiBold", "#FFFFFF")
    text(f, "\u00d7", x + w - 30, y + 8, 18, UI, "Regular", "#FFFFFFB0")
    # user bubble
    rect(f, x + 16, y + 60, w - 32, 54, fill="#F3F1EC", radius=10)
    text(f, "Generate a Python script for sentiment analysis.", x + 28, y + 72,
         13.5, UI, "Regular", INK800, width=w - 60, lh="150%")
    # assistant bubble
    rect(f, x + 16, y + 124, w - 32, h - 224, fill=H100, radius=10)
    text(f, "Certainly! Here is the code...", x + 28, y + 136, 13.5, UI,
         "Regular", H800)
    cy = y + 168
    ch2 = h - 290
    rect(f, x + 28, cy, w - 56, ch2, fill=INK800, radius=8)
    for i, c in enumerate(["#E05B4A", "#E8B850", "#6FBF73"]):
        dot(f, x + 44 + i * 14, cy + 15, 4, c)
    text(f, "import libraries,\nLoad dataset,\nPreprocess data...", x + 44, cy + 34,
         13, MONO, "Regular", "#F1E9CF", width=w - 96, lh="150%")
    # composer
    rect(f, x + 16, y + h - 76, w - 32, 56, fill=PANEL, radius=12, stroke=HAIR, sw=1)
    text(f, "Type your prompt...", x + 32, y + h - 56, 13.5, UI, "Regular", INK500)
    icon_send(f, x + w - 102, y + h - 58, 15)
    icon_mic(f, x + w - 72, y + h - 58, 15)
    icon_clip(f, x + w - 44, y + h - 58, 15)


def superagent_card(f, x, y, w, h, name="Backend Architecture",
                    sub="SUPERAGENT", progress=0.62, meta="3 workers  ·  12.4k ctx"):
    rect(f, x, y, w, h, fill=PANEL, radius=14, stroke=BORDER_GOLD, sw=1.5,
         shadow=SH_E1)
    rect(f, x, y, w, 40, fill=H100, radius=[14, 14, 0, 0])
    text(f, name, x + 30, y + 11, 14.5, UI, "SemiBold", INK900)
    rect(f, x + w - 84, y + 11, 68, 18, fills=HONEY_GRAD, radius=999)
    text(f, "RUNNING", x + w - 76, y + 14, 8.5, MONO, "SemiBold", "#3F3115",
         ls="8%")
    text(f, sub, x + 16, y + 50, 9.5, MONO, "Regular", INK500, ls="12%")
    text(f, "decomposes intent, owns the sub-hive", x + 16, y + 68, 11.5, UI,
         "Regular", INK600)
    rect(f, x + 16, y + h - 46, w - 32, 5, fill=HAIR, radius=999)
    rect(f, x + 16, y + h - 46, (w - 32) * progress, 5, fills=HONEY_GRAD,
         radius=999)
    text(f, meta, x + 16, y + h - 32, 9.5, MONO, "Regular", INK500)


def worker_card(f, x, y, w, h, name="tailwind-builder", progress=0.55):
    rect(f, x, y, w, h, fill=PANEL, radius=12, stroke=HAIR, sw=1, shadow=SH_E1)
    dot(f, x + 16, y + 22, 4.5, H400)
    text(f, name, x + 30, y + 13, 13, UI, "Medium", INK900)
    text(f, "WORKER  ·  react/vite", x + 30, y + 32, 9, MONO, "Regular", INK500,
         ls="6%")
    rect(f, x + 16, y + h - 18, w - 32, 3, fill=HAIR, radius=999)
    rect(f, x + 16, y + h - 18, (w - 32) * progress, 3, fill=H400, radius=999)
    text(f, f"{int(progress*100)}%", x + w - 40, y + h - 40, 9, MONO, "Regular",
         INK500)


def tool_card(f, x, y, w, h):
    rect(f, x, y, w, h, fill=PANEL, radius=14, stroke=HAIR, sw=1, shadow=SH_E1)
    text(f, "Prisma Generator", x + 16, y + 14, 13.5, UI, "SemiBold", INK900)
    rect(f, x + w - 96, y + 14, 80, 18, fill=RECESS, radius=999)
    text(f, "TOOL", x + w - 82, y + 17, 8.5, MONO, "SemiBold", INK700, ls="10%")
    cy = y + 44
    ch = h - 108
    rect(f, x + 16, cy, w - 32, ch, fill=INK800, radius=8)
    for i, c in enumerate(["#E05B4A", "#E8B850", "#6FBF73"]):
        dot(f, x + 30 + i * 12, cy + 13, 3.4, c)
    text(f, "model Session {\n  id  String @id\n  run Run[]\n}", x + 30, cy + 28,
         10.5, MONO, "Regular", "#F1E9CF", width=w - 60, lh="150%")
    rect(f, x + 16, y + h - 44, w - 32, 6, fill=HAIR, radius=999)
    rect(f, x + 16, y + h - 44, (w - 32) * 0.78, 6, fills=HONEY_GRAD, radius=999)
    text(f, "schema.prisma  ·  0.78", x + 16, y + h - 30, 9, MONO, "Regular", INK500)


def error_card(f, x, y, w, h):
    rect(f, x, y, w, h, fill="#FFF7F5", radius=12, stroke=ERR, sw=1.5)
    warn_icon(f, x + 16, y + 16, 12)
    text(f, "Compile failed", x + 36, y + 14, 13, UI, "SemiBold", ERR)
    text(f, "db-schema  ·  E0421  ·  retry 2/3", x + 36, y + 34, 9, MONO,
         "Regular", INK600)
    text(f, "Prisma client type mismatch on Run.sessionId", x + 16, y + 58, 10.5,
         UI, "Regular", INK700, width=w - 32, lh="145%")
    bee_route(f, x + 16, y + h + 18, x + w - 16, y + h + 18, color=ERR,
              dashes=[6, 6], bee_at=0.5, bee_size=13)


def hex_chip(f, x, y, size, label, glyph=H400, face=INK900):
    p, w, h = hex_path(size / 2)
    vector(f, x, y, w, h, p, fill=face, stroke=BORDER_GOLD, sw=1.2)
    bee(f, x + size * 0.24, y + size * 0.32, size * 0.5, body=glyph,
        stripe=face, wing=H200, simple=True)
    text(f, label, x - 6, y + size + 8, 9.5, UI, "Medium", INK700)


def board_agents(f):
    board_header(f, "05", "COMPONENTS", "Agent cards",
                 "Five node types share one anatomy: name, role tag, live state and "
                 "progress. Colour is reserved for state -- honey = healthy, crimson "
                 "= broken route -- so a swarm can be read from across the canvas.")
    close_space(f, 1440, 1300, "COMPONENT BOARD / 32PX ARGUMENT")

    # ---- left column: HyperAgent core -------------------------------------
    text(f, "HYPERAGENT / HIVE CHAT CORE", 72, 246, 10, MONO, "Regular", INK500,
         ls="12%")
    hive_core(f, 40, 272, 540, 620)
    chat_card(f, 72, 308, 420, 500)
    text(f, "The only eternal node. Panning and zooming never move it; Cmd+0 "
            "smoothly recentres the viewport on it from anywhere on the hive.",
         72, 910, 10.5, UI, "Regular", INK600, width=440, lh="155%")

    # ---- right column: the four working node types ------------------------
    text(f, "SUPERAGENT / SUB-HIVE", 620, 246, 10, MONO, "Regular", INK500,
         ls="12%")
    superagent_card(f, 620, 270, 420, 150)
    text(f, "Light honey header, honey-gold 1.5px border, spawns its own workers "
            "underneath itself.", 620, 434, 10.5, UI, "Regular", INK600,
         width=430, lh="155%")

    text(f, "WORKERAGENT", 620, 502, 10, MONO, "Regular", INK500, ls="12%")
    worker_card(f, 620, 526, 320, 96)
    text(f, "No header fill. The progress bar is the only motion on the card.",
         620, 636, 10.5, UI, "Regular", INK600, width=430, lh="155%")

    text(f, "TOOLAGENT / NESTED CODE", 620, 700, 10, MONO, "Regular", INK500,
         ls="12%")
    tool_card(f, 620, 724, 420, 240)
    text(f, "Pops open beneath its parent with a dark syntax box and a gold "
            "execution bar.", 620, 978, 10.5, UI, "Regular", INK600,
         width=430, lh="155%")

    text(f, "BROKEN ROUTE / ERROR STATE", 620, 1046, 10, MONO, "Regular",
         INK500, ls="12%")
    error_card(f, 620, 1070, 320, 110)
    text(f, "Crimson border and a dashed crimson wire; the bee halts and carries "
            "a warning glyph.", 962, 1070, 10.5, UI, "Regular", INK600,
         width=400, lh="155%")

    # ---- far-right column: chips, states, anatomy -------------------------
    RX = 1080
    text(f, "AGENT CHIPS / STATE DOT", RX, 246, 10, MONO, "Regular", INK500,
         ls="12%")
    hex_chip(f, RX, 272, 40, "Nova")
    hex_chip(f, RX + 56, 272, 40, "Vector", glyph=H200)
    hex_chip(f, RX + 112, 272, 40, "Spark", glyph="#FFFFFF")
    text(f, "chip = 40px hex, bee = state colour", RX, 344, 10, MONO,
         "Regular", INK500)

    states = [("healthy", H400, OK), ("running", H600, INK600),
              ("failed", ERR, ERR), ("queued", HAIR, INK500)]
    for i, (nm, c, tc) in enumerate(states):
        y = 388 + i * 30
        dot(f, RX + 4, y + 6, 4.5, c)
        text(f, nm, RX + 18, y, 10.5, UI, "Medium", tc)
        text(f, f"dot/{nm}", RX + 82, y, 9.5, MONO, "Regular", INK500)

    text(f, "CARD ANATOMY / 8PX GRID", RX, 526, 10, MONO, "Regular", INK500,
         ls="12%")
    anatomy = ["name  ·  Inter SemiBold 14.5",
               "role tag  ·  mono 9.5, +12% tracking",
               "state  ·  dot or pill, never both",
               "progress  ·  5px track, 3px worker",
               "meta  ·  mono 9.5  #8A7A52",
               "radius  ·  14 super / 12 worker",
               "padding  ·  16",
               "gap  ·  8 / 16 / 24 only"]
    for i, a in enumerate(anatomy):
        y = 552 + i * 26
        check(f, RX, y + 2, 9)
        text(f, a, RX + 16, y, 10.5, UI, "Regular", INK700)

    bee_route(f, RX, 800, RX + 260, 800, bee_at=0.55, bee_size=18)
    text(f, "bee faces + flies along the data direction", RX, 818, 9.5, MONO,
         "Regular", INK500)
    text(f, "the bee is a routing arrow,\nnever decoration", RX, 848, 10,
         UI, "Regular", INK600, width=200, lh="150%")


# --------------------------------------------------------------------------
# BOARD 06 -- Controls & HUD
# --------------------------------------------------------------------------
def button(f, x, y, w, h, label, kind="primary"):
    if kind == "primary":
        rect(f, x, y, w, h, fills=HONEY_GRAD, radius=10, shadow=SH_E1)
        text(f, label, x, y + h / 2 - 8, 13, UI, "SemiBold", "#3F3115", width=w,
             align="center")
    elif kind == "secondary":
        rect(f, x, y, w, h, fill=PANEL, radius=10, stroke=BORDER_GOLD, sw=1.5)
        text(f, label, x, y + h / 2 - 8, 13, UI, "SemiBold", INK800, width=w,
             align="center")
    elif kind == "ghost":
        text(f, label, x, y + h / 2 - 8, 13, UI, "Medium", AMBER, width=w,
             align="center")
    elif kind == "danger":
        rect(f, x, y, w, h, fill="#FFF7F5", radius=10, stroke=ERR, sw=1.5)
        text(f, label, x, y + h / 2 - 8, 13, UI, "SemiBold", ERR, width=w,
             align="center")


def board_controls(f):
    board_header(f, "06", "COMPONENTS", "Controls & HUD",
                 "The chrome that stays out of the way: one composer, one primary "
                 "action, a rail that never scrolls, and a HUD that reports swarm "
                 "throughput without stealing the canvas.")
    close_space(f, 1440, 1040, "CONTROLS / 32PX ARGUMENT")

    # composer
    text(f, "COMPOSER / HYPERAGENT INPUT", 72, 228, 10, MONO, "Regular", INK500,
         ls="12%")
    rect(f, 72, 248, 560, 64, fill=PANEL, radius=12, stroke=HAIR, sw=1, shadow=SH_E1)
    text(f, "Type your prompt...", 92, 270, 14, UI, "Regular", INK500)
    icon_send(f, 566, 272, 16)
    icon_mic(f, 596, 272, 16)
    icon_clip(f, 624, 272, 16)
    text(f, "focus: 2px honey-300 ring + hairline", 72, 320, 9.5, MONO, "Regular",
         INK500)

    # buttons
    text(f, "BUTTONS", 72, 352, 10, MONO, "Regular", INK500, ls="12%")
    button(f, 72, 372, 168, 44, "Spawn sub-hive", "primary")
    button(f, 256, 372, 168, 44, "Re-route", "secondary")
    button(f, 440, 372, 130, 44, "Cancel", "ghost")
    button(f, 580, 372, 130, 44, "Kill worker", "danger")
    text(f, "40px min touch target  ·  radius 10  ·  label Inter SemiBold 13",
         72, 426, 9.5, MONO, "Regular", INK500)

    # chips / tabs
    text(f, "TABS", 72, 466, 10, MONO, "Regular", INK500, ls="12%")
    tabs = ["Workspace", "Swarm", "Routing", "History"]
    tx = 72
    for i, t in enumerate(tabs):
        w = 24 + len(t) * 8.4
        if i == 1:
            rect(f, tx, 486, w, 34, fill=PANEL, radius=8, stroke=BORDER_GOLD, sw=1.2)
        text(f, t, tx, 494, 12, UI, "Medium" if i == 1 else "Regular",
             INK900 if i == 1 else INK500, width=w, align="center")
        tx += w + 18

    # HUD badge
    text(f, "HUD / THROUGHPUT BADGE", 72, 552, 10, MONO, "Regular", INK500,
         ls="12%")
    rect(f, 72, 572, 420, 52, fill=PANEL, radius=999, stroke=BORDER_GOLD, sw=1.2,
         shadow=SH_E1)
    dot(f, 96, 598, 5, OK)
    text(f, "4 SUPERAGENTS  ·  12 WORKERS  ·  98.2% GREEN", 112, 590, 10.5, MONO,
         "SemiBold", INK800, ls="6%")
    text(f, "+3.4%", 74, 636, 9.5, MONO, "Regular", OK)

    # timeline
    text(f, "TIME-TRAVEL SCRUBBER", 72, 676, 10, MONO, "Regular", INK500, ls="12%")
    rect(f, 72, 696, 600, 66, fill=PANEL, radius=12, stroke=HAIR, sw=1, shadow=SH_E1)
    rect(f, 96, 728, 552, 4, fill=HAIR, radius=999)
    rect(f, 96, 728, 320, 4, fills=HONEY_GRAD, radius=999)
    for i in range(7):
        x = 96 + i * 92
        dot(f, x, 730, 3, INK500 if i > 3 else H600)
    dot(f, 416, 730, 9, PANEL)
    ellipse(f, 408.5, 722.5, 15, 15, fill=PANEL, stroke=H500, sw=2)
    text(f, "09:41:07  ·  routing state 12 of 38", 96, 706, 9.5, MONO, "Regular",
         INK500)

    # sidebar rail
    text(f, "NAV RAIL / 72PX FIXED", 720, 228, 10, MONO, "Regular", INK500,
         ls="12%")
    rect(f, 720, 248, 72, 560, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    rail = [("Workspace", icon_hex), ("Agents", icon_bell), ("Data", icon_layers),
            ("Models", icon_cube), ("Routing", icon_fork), ("Settings", icon_gear)]
    for i, (nm, ic) in enumerate(rail):
        y = 264 + i * 88
        if i == 0:
            rect(f, 730, y - 6, 52, 52, fill=INK900, radius=12)
            ic(f, 745, y + 6, 22, H400)
        else:
            ic(f, 745, y + 6, 22, INK700)
        text(f, nm, 724, y + 54, 8.5, UI, "Medium" if i == 0 else "Regular",
             INK900 if i == 0 else INK500, width=64, align="center")

    # panel
    text(f, "LIBRARY PANEL / 260PX", 880, 228, 10, MONO, "Regular", INK500,
         ls="12%")
    rect(f, 820, 248, 260, 560, fill=PANEL_ALT, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    rect(f, 836, 264, 228, 34, fill=PANEL, radius=8, stroke=HAIR, sw=1)
    text(f, "Project: HiveMind AI", 848, 273, 11.5, UI, "Medium", INK800)
    chevron(f, 1046, 278, 9)
    text(f, "ACTIVE AGENTS", 836, 316, 9, UI, "SemiBold", INK700, ls="8%")
    hex_chip(f, 838, 334, 38, "Nova")
    hex_chip(f, 894, 334, 38, "Vector", glyph=H200)
    hex_chip(f, 950, 334, 38, "Spark", glyph="#FFFFFF")
    text(f, "TEMPLATES", 836, 416, 9, UI, "SemiBold", INK700, ls="8%")
    tpl = ["Content Writer", "Data Analyst", "Code Reviewer", "Chatbot"]
    for i, t in enumerate(tpl):
        y = 434 + i * 38
        rect(f, 836, y, 228, 30, fill=PANEL, radius=8, stroke=HAIR, sw=1)
        icon_layers(f, 848, y + 8, 13, INK600)
        text(f, t, 870, y + 8, 11.5, UI, "Regular", INK800)
    rect(f, 836, 598, 228, 96, fill=H100, radius=10)
    text(f, "TIP", 848, 608, 8.5, MONO, "SemiBold", H800, ls="12%")
    text(f, "Drag a worker onto another super-agent to re-route its context stream "
            "live.", 848, 624, 10.5, UI, "Regular", H800, width=204, lh="145%")

    # minimap
    text(f, "MINI-MAP / 260X170", 1120, 228, 10, MONO, "Regular", INK500, ls="12%")
    rect(f, 1120, 248, 260, 170, fill=PANEL, radius=12, stroke=HAIR, sw=1,
         shadow=SH_E2)
    honeycomb(f, 1128, 256, 244, 154, R=14, color="#EFE7D2", weight=1)
    for gx, gy, gw, gh in [(1146, 268, 34, 20), (1206, 264, 44, 24),
                           (1156, 316, 40, 22), (1216, 320, 36, 20),
                           (1180, 364, 54, 26)]:
        rect(f, gx, gy, gw, gh, fill=H100, radius=4, stroke=H600, sw=1)
    rect(f, 1178, 306, 26, 18, fills=HONEY_GRAD, radius=4, stroke=H800, sw=1)
    rect(f, 1200, 296, 70, 52, fill="none", radius=6, stroke=INK900, sw=1.6)
    text(f, "viewport", 1128, 426, 9, MONO, "Regular", INK500)

    # toast
    text(f, "TOAST / REFUSAL", 1120, 458, 10, MONO, "Regular", INK500, ls="12%")
    rect(f, 1120, 478, 260, 92, fill=INK900, radius=12, shadow=SH_E2)
    text(f, "Re-route refused", 1136, 490, 12.5, UI, "SemiBold", "#FFFFFF")
    text(f, "context-prune blocked: worker still\nstreaming (NODE_REWIRE pending)",
         1136, 512, 10, MONO, "Regular", H300, width=228, lh="145%")
    text(f, "REFUSED BY NAME  ·  NOT SILENT", 1136, 548, 8.5, MONO, "Regular",
         "#FFFFFF70", ls="10%")


# --------------------------------------------------------------------------
# BOARD 07 -- Spatial canvas hero
# --------------------------------------------------------------------------
def board_hero(f):
    W, H = 1440, 900
    # window chrome
    rect(f, 0, 0, W, 52, fill=PANEL)
    rule(f, 0, 52, W, HAIR, 1)
    hexmark(f, 16, 12, 28)
    text(f, "HiveMind", 52, 15, 17, UI, "Bold", INK900)
    icon_search(f, 1214, 17, 18)
    icon_bell(f, 1256, 17, 18)
    dot(f, 1276, 16, 4, ERR)
    ellipse(f, 1300, 10, 32, 32, fill=H200)
    text(f, "AC", 1308, 17, 11, UI, "SemiBold", INK800)
    text(f, "A. Chen", 1344, 16, 12.5, UI, "Medium", INK800)
    chevron(f, 1404, 22, 8, INK500)

    # rail
    rect(f, 0, 52, 72, H - 52, fill=PANEL)
    rule(f, 72, 52, 1, HAIR, 1)
    rail = [("Workspace", icon_hex), ("Agents", icon_bell), ("Data", icon_layers),
            ("Models", icon_cube), ("Routing", icon_fork), ("Settings", icon_gear)]
    for i, (nm, ic) in enumerate(rail):
        y = 72 + i * 92
        if i == 0:
            rect(f, 12, y - 4, 48, 48, fill=INK900, radius=11)
            ic(f, 25, y + 7, 22, H400)
        else:
            ic(f, 25, y + 7, 22, INK700)
        text(f, nm, -4, y + 50, 8, UI, "Medium" if i == 0 else "Regular",
             INK900 if i == 0 else INK500, width=80, align="center")

    # panel
    rect(f, 72, 52, 240, H - 52, fill=PANEL_ALT)
    rule(f, 312, 52, 1, HAIR, 1)
    rect(f, 84, 68, 216, 32, fill=PANEL, radius=8, stroke=HAIR, sw=1)
    text(f, "Project: HiveMind AI", 96, 76, 11.5, UI, "Medium", INK800)
    chevron(f, 274, 81, 8)
    text(f, "Active Agents", 84, 116, 11.5, UI, "SemiBold", INK900)
    hex_chip(f, 86, 138, 40, "Nova")
    hex_chip(f, 144, 138, 40, "Vector", glyph=H200)
    hex_chip(f, 202, 138, 40, "Spark", glyph="#FFFFFF")
    hex_chip(f, 86, 210, 40, "Spark", glyph="#FFFFFF")
    text(f, "Templates", 84, 284, 11.5, UI, "SemiBold", INK900)
    tpl = ["Content Writer", "Data Analyst", "Code Reviewer", "Chatbot"]
    for i, t in enumerate(tpl):
        y = 306 + i * 32
        rect(f, 84, y, 216, 26, fill=PANEL, radius=7, stroke=HAIR, sw=1)
        icon_layers(f, 94, y + 7, 12, INK600)
        text(f, t, 112, y + 6, 11, UI, "Regular", INK800)

    # canvas
    rect(f, 312, 52, W - 312, H - 52, fill=CANVAS)
    honeycomb(f, 312, 52, W - 312, H - 52, R=44, color="#EFE6CE", weight=1)

    # ghost topology lines
    for (x1, y1, x2, y2) in [(860, 400, 1120, 300), (900, 520, 1180, 620),
                             (700, 300, 620, 200), (760, 640, 560, 700),
                             (1000, 250, 1180, 130)]:
        vector(f, min(x1, x2), min(y1, y2), abs(x2 - x1) or 1, abs(y2 - y1) or 1,
               f"M 0 0 L {abs(x2-x1)} {abs(y2-y1)}", fill="none",
               stroke="#DCD2B8", sw=1.2)

    # ghost cards
    ghosts = [(392, 160, 96, 44, "router.ts"), (1104, 128, 120, 52, "schema.prisma"),
              (1180, 560, 116, 46, "agent.spec.ts"), (356, 600, 104, 40, "main.tsx"),
              (1288, 330, 96, 44, "beam.rs")]
    for gx, gy, gw, gh, lab in ghosts:
        rect(f, gx, gy, gw, gh, fill=PANEL, radius=8, opacity=0.82, shadow=SH_E1)
        rect(f, gx + 10, gy + 10, 34, 4, fill=HAIR, radius=999)
        rect(f, gx + 10, gy + 19, gw - 30, 3, fill="#EDE9E2", radius=999)
        rect(f, gx + 10, gy + 26, gw - 44, 3, fill="#EDE9E2", radius=999)
        text(f, lab, gx, gy + gh + 5, 8.5, MONO, "Regular", "#A99B7C")

    # hive + chat
    hive_core(f, 508, 96, 660, 700)
    chat_card(f, 620, 196, 420, 500)

    # routing bees
    bee_route(f, 1046, 300, 1096, 168, bee_at=0.55, bee_size=15)
    bee_route(f, 1046, 470, 1160, 560, bee_at=0.6, bee_size=15)
    bee_route(f, 618, 470, 520, 620, bee_at=0.5, bee_size=14)
    bee_route(f, 620, 300, 470, 208, bee_at=0.45, bee_size=13)

    # HUD badge
    rect(f, 540, 68, 460, 44, fill=PANEL, radius=999, stroke=BORDER_GOLD, sw=1.2,
         shadow=SH_E2)
    dot(f, 564, 90, 5, OK)
    text(f, "HYPERAGENT  ·  4 SUPERAGENTS  ·  12 WORKERS  ·  98.2% GREEN", 580, 83,
         10, MONO, "SemiBold", INK800, ls="6%")

    # minimap
    rect(f, 1120, 700, 220, 150, fill=PANEL, radius=12, stroke=HAIR, sw=1,
         shadow=SH_E2)
    honeycomb(f, 1126, 706, 208, 138, R=12, color="#EFE7D2", weight=1)
    for gx, gy, gw, gh in [(1140, 716, 30, 18), (1184, 712, 38, 20),
                           (1148, 754, 34, 18), (1192, 756, 30, 16)]:
        rect(f, gx, gy, gw, gh, fill=H100, radius=3, stroke=H600, sw=0.8)
    rect(f, 1176, 740, 24, 16, fills=HONEY_GRAD, radius=3, stroke=H800, sw=0.8)
    rect(f, 1150, 730, 100, 60, fill="none", radius=6, stroke=INK900, sw=1.4)

    # canvas footer
    text(f, "0.1x - 3.0x   ·   Cmd+0 recentres the core hive   ·   drag a worker to re-route",
         328, 866, 9.5, MONO, "Regular", "#A0916F", ls="4%")
    text(f, "HIVEMIND / SPATIAL CANVAS", 328, 70, 9.5, MONO, "Regular",
         "#A0916F", ls="14%")



# --------------------------------------------------------------------------
# BOARD 08 -- HMAS / hierarchy & topology
# --------------------------------------------------------------------------
def board_hmas_topology(f):
    W, H = 1440, 900
    board_header(f, "08", "HMAS / HIERARCHY", "Hierarchy & topology",
                 "The HyperAgent tree and the live swarm graph are the same "
                 "object: re-parent a worker only while it is not streaming; "
                 "every edge is a bee line, never decoration.")
    close_space(f, W, H, "HMAS / 32PX ARGUMENT")

    rect(f, 72, 220, 520, 600, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "HIERARCHY / WORKSPACE-JAILED", 96, 242, 10, MONO, "Regular",
         INK500, ls="12%")
    rect(f, 96, 270, 472, 64, fill=H100, radius=12, stroke=BORDER_GOLD, sw=1.2)
    hexmark(f, 110, 288, 32)
    text(f, "HyperAgent", 150, 284, 14, UI, "SemiBold", INK900)
    text(f, "root · 4 superagents · healthy", 150, 306, 9, MONO, "Regular",
         INK500, ls="4%")
    rows = [
        (120, 350, 448, "Backend Architecture", "SUPERAGENT · 3 workers · running", H400),
        (144, 390, 424, "tailwind-builder", "WORKER · react/vite · 55%", H600),
        (144, 430, 424, "db-schema", "WORKER · E0421 · retry 2/3", ERR),
        (120, 470, 448, "Frontend Architecture", "SUPERAGENT · 2 workers · running", H400),
        (144, 510, 424, "agent.spec.ts", "WORKER · completed · 0.78", H600),
        (144, 550, 424, "main.tsx", "WORKER · queued", HAIR),
    ]
    for x, y, w, name, meta, dotc in rows:
        rect(f, x, y, w, 34, fill=PANEL, radius=8, stroke=HAIR, sw=1)
        dot(f, x + 16, y + 17, 4, dotc)
        text(f, name, x + 28, y + 8, 11.5, UI, "Medium", INK900)
        text(f, meta, x + 166, y + 11, 8.5, MONO, "Regular", INK500, ls="4%")
    text(f, "nesting: HyperAgent > SuperAgent > WorkerAgent > ToolAgent",
         96, 602, 9.5, MONO, "Regular", INK500, ls="4%")
    rule(f, 96, 632, 472, HAIR, 1)
    text(f, "re-parent only while not streaming · kill/rewire are "
            "confirmation-first", 96, 648, 9.5, UI, "Regular", INK600,
         width=460, lh="145%")
    button(f, 96, 746, 216, 38, "Spawn sub-hive", "primary")
    button(f, 324, 746, 216, 38, "Kill worker", "danger")

    superagent_card(f, 620, 250, 420, 150)
    worker_card(f, 620, 430, 320, 96)
    tool_card(f, 620, 560, 420, 240)
    text(f, "S15 / S16 / S17 · authoritative card sizes (420x150, 320x96, "
            "420x240)", 620, 812, 9, MONO, "Regular", INK500, ls="4%")

    rect(f, 1080, 220, 288, 380, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "TOPOLOGY / LIVE EDGES", 1100, 242, 10, MONO, "Regular", INK500,
         ls="12%")
    honeycomb_window = honeycomb(f, 1092, 258, 264, 300, R=16, color=HAIR_SOFT,
                                 weight=1)
    topo = {"root": (1224, 310), "back": (1136, 400), "front": (1312, 400),
            "w1": (1116, 500), "w2": (1240, 500)}
    for key, (x0, y0), (x1, y1) in [
            ("root", topo["root"], topo["back"]),
            ("root2", topo["root"], topo["front"]),
            ("back", topo["back"], topo["w1"]),
            ("front", topo["front"], topo["w2"])]:
        bee_route(f, x0, y0, x1, y1, bee_at=0.6, bee_size=11)
    for key, (cx, cy), label in [
            ("root", topo["root"], "HyperAgent"), ("back", topo["back"], "Backend"),
            ("front", topo["front"], "Frontend"), ("w1", topo["w1"], "tailwind"),
            ("w2", topo["w2"], "agent.spec")]:
        dot(f, cx, cy, 7, H400 if key == "root" else H600)
        text(f, label, cx - 36, cy + 10, 8.5, MONO, "Regular", INK600,
             width=72, align="center")
    text(f, "bee faces the data direction", 1100, 566, 9, MONO, "Regular",
         INK500)

    rect(f, 1080, 620, 288, 200, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "STATES / SHAPE + TEXT CUE", 1100, 642, 10, MONO, "Regular",
         INK500, ls="12%")
    legend = [("healthy", H400, "circle"), ("running", H600, "circle"),
              ("queued", HAIR, "square"), ("failed", ERR, "triangle"),
              ("awaiting_approval", AMBER, "diamond")]
    for i, (name, color, shape) in enumerate(legend):
        y = 670 + i * 26
        if shape == "circle":
            ellipse(f, 1104, y, 8, 8, fill=color)
        elif shape == "square":
            rect(f, 1104, y, 8, 7, fill=color, radius=2)
        else:
            rect(f, 1104, y, 8, 8, fill=color, radius=2)
        text(f, name, 1122, y - 1, 9.5, UI, "Medium", INK700)
    text(f, "colour is never the only channel", 1100, 798, 9, MONO, "Regular",
         INK500)


# --------------------------------------------------------------------------
# BOARD 09 -- HMAS / governance (profiles, memory, mailbox, projects, usage,
#             cross-branch message approval)
# --------------------------------------------------------------------------
def board_hmas_governance(f):
    W, H = 1440, 900
    board_header(f, "09", "HMAS / GOVERNANCE", "Profiles, memory & mail",
                 "One operator surface for the mesh: importable profiles, a "
                 "clearable memory, the cross-branch mailbox, project identity, "
                 "usage counters and nonce-bound message approval.")
    close_space(f, W, H, "HMAS / 32PX ARGUMENT")

    def card(x, y, title, subtitle=""):
        rect(f, x, y, 420, 260, fill=PANEL, radius=14, stroke=HAIR, sw=1,
             shadow=SH_E1)
        text(f, title, x + 20, y + 20, 10, MONO, "Regular", INK500, ls="12%")
        if subtitle:
            text(f, subtitle, x + 20, y + 38, 9, MONO, "Regular", INK500)
        return y + 52

    # 01 profiles / import
    x, y = 72, 220
    cy = card(x, y, "PROFILES · IMPORT")
    rect(f, x + 20, cy, 380, 30, fill=H100, radius=8, stroke=BORDER_GOLD, sw=1)
    text(f, "Default profile · active · 2 gates", x + 32, cy + 8, 10.5, UI,
         "Medium", INK900)
    text(f, "Strict review · paused", x + 32, cy + 48, 11, UI, "Medium", INK700)
    text(f, "Import a folder or a .figcraft manifest; nothing executes at "
            "import time.", x + 20, cy + 74, 9, MONO, "Regular", INK500,
         width=380, lh="145%")
    button(f, x + 20, y + 204, 380, 36, "Import project", "secondary")

    # 02 memory
    x, y = 540, 220
    cy = card(x, y, "MEMORY")
    text(f, "Run summary · 12.4k ctx", x + 20, cy, 10.5, UI, "Regular", INK800)
    text(f, "Decision log · 3 rows", x + 20, cy + 22, 10.5, UI, "Regular", INK800)
    text(f, "Refusal ledger · 5 rows", x + 20, cy + 44, 10.5, UI, "Regular", INK800)
    text(f, "Clear is confirmation-first; the ledger is append-only.",
         x + 20, cy + 68, 9, MONO, "Regular", INK500, width=380, lh="145%")
    button(f, x + 20, y + 204, 380, 36, "Clear memory", "danger")

    # 03 mailbox
    x, y = 1008, 220
    cy = card(x, y, "MAILBOX · 2 UNREAD")
    dot(f, x + 24, cy + 8, 5, AMBER)
    text(f, "Backend → Frontend", x + 38, cy, 10.5, UI, "Medium", INK900)
    text(f, "schema.prisma v3 ready", x + 38, cy + 18, 8.5, MONO, "Regular", INK500)
    dot(f, x + 24, cy + 52, 5, H400)
    text(f, "Frontend → Backend", x + 38, cy + 44, 10.5, UI, "Medium", INK900)
    text(f, "API drift report", x + 38, cy + 62, 8.5, MONO, "Regular", INK500)
    text(f, "Mailbox is read-only; delivery is a branch event.", x + 20,
         cy + 92, 9, MONO, "Regular", INK500, width=380, lh="145%")

    # 04 projects
    x, y = 72, 540
    cy = card(x, y, "PROJECTS")
    rect(f, x + 20, cy, 380, 34, fill=H100, radius=8, stroke=BORDER_GOLD, sw=1)
    text(f, "HiveMind AI · active", x + 32, cy + 9, 11, UI, "Medium", INK900)
    text(f, "sandbox · idle", x + 32, cy + 50, 11, UI, "Regular", INK700)
    text(f, "project_id · localhost only · PDF:256,275-282", x + 20, cy + 76,
         9, MONO, "Regular", INK500, width=380)

    # 05 usage
    x, y = 540, 540
    cy = card(x, y, "USAGE")
    text(f, "4 SUPERAGENTS", x + 20, cy, 13, UI, "SemiBold", INK900)
    text(f, "12 WORKERS", x + 210, cy, 13, UI, "SemiBold", INK900)
    text(f, "98.2% GREEN", x + 20, cy + 26, 13, UI, "SemiBold", OK)
    text(f, "38 calls · 12.4k ctx · derived counters", x + 20, cy + 54, 9,
         MONO, "Regular", INK500, width=380)
    rect(f, x + 20, cy + 76, 380, 8, fill=HAIR, radius=999)
    rect(f, x + 20, cy + 76, 374, 8, fill=H400, radius=999)

    # 06 cross-branch message approval
    x, y = 1008, 540
    rect(f, x, y, 420, 260, fill=PANEL, radius=14, stroke=BORDER_GOLD, sw=1.5,
         shadow=SH_E1)
    text(f, "MESSAGE APPROVAL", x + 20, y + 20, 10, MONO, "Regular", INK500,
         ls="12%")
    dot(f, x + 386, y + 24, 6, AMBER)
    text(f, "Backend Architecture → Frontend Architecture", x + 20, y + 44,
         10.5, UI, "Medium", INK900, width=380, lh="140%")
    text(f, "schema.prisma v3 ready to merge", x + 20, y + 82, 9, MONO,
         "Regular", INK500)
    text(f, "args_sha256 · 9f86d081884c…", x + 20, y + 100, 9, MONO, "Regular",
         INK500)
    button(f, x + 20, y + 132, 176, 36, "Deny", "danger")
    button(f, x + 212, y + 132, 188, 36, "Approve", "primary")
    text(f, "APPROVAL_RESOLVE · nonce · single-use grant · TTL 120 s",
         x + 20, y + 188, 9, MONO, "Regular", INK500, width=380, lh="145%")
    text(f, "Approve executes no effect by itself; it grants the engine one "
            "consume.", x + 20, y + 214, 9, UI, "Regular", INK600, width=380,
         lh="145%")


# --------------------------------------------------------------------------
# BOARD 10 -- States & refusals
# --------------------------------------------------------------------------
def board_states(f):
    W, H = 1440, 900
    board_header(f, "10", "STATES & REFUSALS", "Named states, not silent failures",
                 "Every failure is a named refusal; every wait has a state; "
                 "nothing is a silent drop. Replay is read-only and the diff "
                 "surface stays DECISION-GATED until its commands are frozen.")
    close_space(f, W, H, "STATES / 32PX ARGUMENT")

    def state_card(x, y, label, title):
        rect(f, x, y, 420, 300, fill=PANEL, radius=14, stroke=HAIR, sw=1,
             shadow=SH_E1)
        text(f, label, x + 20, y + 16, 9.5, MONO, "Regular", INK500, ls="12%")
        text(f, title, x + 20, y + 34, 15, UI, "SemiBold", INK900)
        return y + 60

    # 01 named refusal (crimson refusal card, S18 error-card anatomy)
    x, y = 72, 220
    cy = state_card(x, y, "S14 / NAMED REFUSAL", "E_APPROVAL_TIMEOUT")
    rect(f, x + 20, cy, 380, 138, fill=PANEL, radius=14, stroke=ERR, sw=1.5,
         shadow=SH_E1)
    text(f, "E_APPROVAL_TIMEOUT", x + 36, cy + 14, 11.5, MONO, "SemiBold", ERR)
    text(f, "The approval request timed out before a decision arrived.",
         x + 36, cy + 38, 9.5, UI, "Regular", INK700, width=344, lh="145%")
    text(f, "how_to_fix: Resolve the request when prompted; the tool call is "
            "refused, never faked.", x + 36, cy + 82, 8.5, MONO, "Regular",
         INK500, width=344, lh="145%")
    text(f, "S18 crimson error anatomy · ERROR frame → toast + log + ledger",
         x + 20, cy + 160, 9, MONO, "Regular", INK500, ls="3%")

    # 02 reconnect
    x = 540
    cy = state_card(x, y, "RECONNECT / SOCKET CLOSED", "Reconnecting")
    rect(f, x + 20, cy, 380, 60, fill=H100, radius=10)
    dot(f, x + 38, cy + 30, 5, AMBER)
    text(f, "Reconnecting to the local server; mutations are blocked until the "
            "socket is open.", x + 52, cy + 18, 10, UI, "Regular", INK800,
         width=330, lh="140%")
    bee_route(f, x + 30, cy + 100, x + 390, cy + 100, color=AMBER,
              dashes=[8, 8], bee_at=0.5, bee_size=14)
    text(f, "SERVER_HELLO · RESYNC_REQUIRED · retry with backoff", x + 20,
         cy + 132, 9, MONO, "Regular", INK500, ls="3%")
    text(f, "no mutation is queued while disconnected", x + 20, cy + 154, 9,
         MONO, "Regular", INK500)
    text(f, "S19 state dot stays amber; text cue is never colour-only.",
         x + 20, cy + 184, 9, UI, "Regular", INK600, width=380, lh="145%")

    # 03 empty / loading
    x = 1008
    cy = state_card(x, y, "S26 / EMPTY · LOADING", "Empty and loading")
    rect(f, x + 20, cy, 184, 150, fill=PANEL_ALT, radius=12, stroke=HAIR, sw=1)
    text(f, "Nothing here yet", x + 34, cy + 18, 11, UI, "SemiBold", INK900)
    text(f, "No agents yet — submit a prompt to create the first run.",
         x + 34, cy + 40, 8.5, UI, "Regular", INK600, width=156, lh="145%")
    rect(f, x + 216, cy, 184, 150, fill=PANEL_ALT, radius=12, stroke=HAIR, sw=1)
    ellipse(f, x + 232, cy + 18, 34, 34, fill="none", stroke=H400, sw=3)
    text(f, "Loading", x + 232, cy + 64, 11, UI, "SemiBold", INK900)
    rect(f, x + 232, cy + 92, 150, 9, fill=HAIR, radius=999)
    rect(f, x + 232, cy + 110, 118, 9, fill=HAIR, radius=999)
    text(f, "honest empty copy · skeleton bars · no fabricated rows",
         x + 20, cy + 168, 9, MONO, "Regular", INK500, width=380, lh="140%")

    # 04 kill confirmation
    x, y = 72, 540
    cy = state_card(x, y, "KILL CONFIRMATION", "Kill worker?")
    rect(f, x + 20, cy, 380, 190, fill=RECESS, radius=12)
    rect(f, x + 44, cy + 18, 332, 154, fill=PANEL, radius=14, stroke=ERR, sw=1.5)
    text(f, "Kill worker?", x + 64, cy + 32, 14, UI, "SemiBold", INK900)
    text(f, "tailwind-builder · WORKER · running", x + 64, cy + 54, 9, MONO,
         "Regular", INK500)
    text(f, "The worker cannot be killed from its current state.",
         x + 64, cy + 72, 9.5, UI, "Regular", INK800, width=292)
    text(f, "Only running workers can be killed; refresh the worker state.",
         x + 64, cy + 90, 8.5, MONO, "Regular", INK500, width=292, lh="140%")
    button(f, x + 64, cy + 112, 100, 30, "Cancel", "secondary")
    button(f, x + 176, cy + 112, 152, 30, "Kill worker", "danger")
    text(f, "KILL_WORKER · confirmation first · never silent", x + 20,
         cy + 210, 9, MONO, "Regular", INK500, ls="4%")

    # 05 replay read-only
    x = 540
    cy = state_card(x, y, "REPLAY / READ-ONLY", "Replay (read-only)")
    rect(f, x + 20, cy, 380, 36, fill=INK900, radius=10)
    text(f, "Replay (read-only) · revision held", x + 36, cy + 10, 10, MONO,
         "Regular", H300)
    rect(f, x + 20, cy + 48, 380, 112, fill=PANEL_ALT, radius=12, stroke=HAIR,
         sw=1)
    for i, gx in enumerate([x + 50, x + 190, x + 330]):
        rect(f, gx, cy + 82, 30, 24, fill=RECESS, radius=6, stroke=INK500, sw=1)
    bee_route(f, x + 90, cy + 94, x + 356, cy + 94, color=INK500,
              dashes=[6, 6], bee_at=0.55, bee_size=11)
    text(f, "HISTORY_REVISION_SET → HISTORY_SNAPSHOT · no mutations while in "
            "replay", x + 20, cy + 174, 9, MONO, "Regular", INK500, width=380,
         lh="140%")

    # 06 DECISION-GATED diff explanation
    x = 1008
    cy = state_card(x, y, "S25 / DECISION-GATED", "Code-diff review")
    rect(f, x + 20, cy, 380, 70, fill=PANEL_ALT, radius=10, stroke=HAIR, sw=1)
    text(f, "router.ts", x + 34, cy + 12, 9.5, MONO, "Regular", INK500)
    text(f, "- const route = legacy();", x + 34, cy + 30, 9, MONO, "Regular", ERR)
    text(f, "+ const route = mesh();", x + 34, cy + 48, 9, MONO, "Regular", OK)
    rect(f, x + 20, cy + 82, 380, 22, fill=H100, radius=999,
         stroke=BORDER_GOLD, sw=1)
    text(f, "DECISION-GATED · DIFF_* commands are NOT frozen",
         x + 34, cy + 89, 8.5, MONO, "SemiBold", H800, ls="2%")
    text(f, "Decision required: the code-diff surface is proposal-gated (no "
            "PDF artboard, no frozen DIFF command). It is not built in v1.",
         x + 20, cy + 116, 9, UI, "Regular", INK700, width=380, lh="145%")
    button(f, x + 20, cy + 166, 110, 26, "Propose", "secondary")
    button(f, x + 142, cy + 166, 110, 26, "Apply", "secondary")
    text(f, "U3 G-03 · resolve only after the contract freezes DIFF_*",
         x + 20, cy + 204, 8.5, MONO, "Regular", INK500, ls="2%")



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
                 "Same shell and cards + the proposed motion layer "
                 "(durations/easing PROPOSED).")
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
    text(f, "FOCUS RING \u00b7 ink-700 2px",
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


# --------------------------------------------------------------------------
# Phase 1
# --------------------------------------------------------------------------
def phase1():
    add("file.new", name="HiveMind Style System")
    frame("00 · Cover", 0, 0, 1440, 900)
    frame("01 · Colour", 1520, 0, 1440, 1240)
    frame("02 · Typography", 3040, 0, 1440, 1340)
    frame("03 · Space & Depth", 0, 1440, 1440, 1000)
    frame("04 · Brand Lockups", 1520, 1440, 1440, 1000)
    frame("05 · Agent Cards", 3040, 1440, 1440, 1300)
    frame("06 · Controls & HUD", 0, 2900, 1440, 1040)
    frame("07 · Spatial Canvas", 1520, 2900, 1440, 900)
    frame("08 · HMAS Hierarchy", 0, 4440, 1440, 900)
    frame("09 · HMAS Governance", 1520, 4440, 1440, 900)
    frame("10 · States & Refusals", 3040, 4440, 1440, 900)

    board_cover(FRAME_IDS["00 · Cover"])
    board_color(FRAME_IDS["01 · Colour"])
    board_type(FRAME_IDS["02 · Typography"])
    board_space(FRAME_IDS["03 · Space & Depth"])
    board_brand(FRAME_IDS["04 · Brand Lockups"])
    board_agents(FRAME_IDS["05 · Agent Cards"])
    board_controls(FRAME_IDS["06 · Controls & HUD"])
    board_hero(FRAME_IDS["07 · Spatial Canvas"])
    board_hmas_topology(FRAME_IDS["08 · HMAS Hierarchy"])
    board_hmas_governance(FRAME_IDS["09 · HMAS Governance"])
    board_states(FRAME_IDS["10 · States & Refusals"])

    # preserve ids of boards 00-10: refine 05 and add 11 after they exist
    board_agents_refine(FRAME_IDS["05 · Agent Cards"])
    frame("11 · Motion & Live", 0, 5580, 1440, 900)
    board_motion(FRAME_IDS["11 · Motion & Live"])

    with open(os.path.join(HERE, "commands_phase1.json"), "w", encoding="utf-8") as fh:
        json.dump(CMDS, fh, indent=0)
    with open(os.path.join(HERE, "expected_ids.json"), "w", encoding="utf-8") as fh:
        json.dump(EXPECTED, fh, indent=0)
    print(f"phase1: {len(CMDS)} commands, {NEXT_ID[0]-2} nodes, "
          f"next id would be {NEXT_ID[0]}")


# --------------------------------------------------------------------------
# Phase 2 -- native tokens + exports
# --------------------------------------------------------------------------
def phase2():
    CMDS.clear()
    # colour variables
    add("variable.createCollection", name="Hive / Colour")
    for name, hexv, token, _role in COLORS:
        add("variable.create", collection="Hive / Colour", name=token,
            type="color", value=hexv)
    for name, hexv, token in [("GRID", HAIR_SOFT, "canvas/honeycomb-line")]:
        add("variable.create", collection="Hive / Colour", name=token,
            type="color", value=hexv)

    # paint styles
    groups = {
        "Hive/Surface": [(n, h) for n, h, t, _ in COLORS if t.startswith(("surface", "border"))],
        "Hive/Honey": [(n, h) for n, h, t, _ in COLORS if t.startswith("honey")],
        "Hive/Ink": [(n, h) for n, h, t, _ in COLORS if t.startswith("ink")],
        "Hive/State": [(n, h) for n, h, t, _ in COLORS if t.startswith(("state", "accent"))],
    }
    for group, items in groups.items():
        for n, h in items:
            add("style.create", type="paint", name=f"{group}/{n}", value=[h])

    # text styles
    for token, fam, st, size, lh, ls in [
        ("Brand/Display XXL", BRAND, "SemiBold", 96, "104%", "-3%"),
        ("Brand/Display L", BRAND, "SemiBold", 56, "107%", "-2%"),
        ("UI/Heading H1", UI, "SemiBold", 40, "120%", "-1.5%"),
        ("UI/Heading H3", UI, "SemiBold", 24, "133%", "-0.5%"),
        ("UI/Heading H4", UI, "SemiBold", 20, "140%", "0%"),
        ("UI/Body L", UI, "Regular", 17, "165%", "0%"),
        ("UI/Body M", UI, "Regular", 15, "160%", "0%"),
        ("UI/Body S", UI, "Regular", 13, "154%", "0%"),
        ("UI/Label", UI, "SemiBold", 12, "133%", "8%"),
        ("UI/Label Micro", UI, "SemiBold", 9.5, "133%", "12%"),
        ("Mono/Code", MONO, "Regular", 13, "169%", "0%"),
        ("Mono/Code S", MONO, "Regular", 10.5, "150%", "0%"),
        ("Mono/Meta", MONO, "Regular", 11, "145%", "8%"),
    ]:
        add("style.create", type="text", name=token,
            value={"fontFamily": fam, "fontStyle": st, "fontSize": size,
                   "lineHeight": lh, "letterSpacing": ls, "textAlign": "left"})

    # effect styles
    for nm, sh in [("Depth/e1-node", SH_E1), ("Depth/e2-float", SH_E2),
                   ("Depth/e3-hive", SH_E3)]:
        add("style.create", type="effect", name=nm, value=sh)

    # exports
    os.makedirs(EXPORTS, exist_ok=True)
    order = ["00 · Cover", "01 · Colour", "02 · Typography", "03 · Space & Depth",
             "04 · Brand Lockups", "05 · Agent Cards", "06 · Controls & HUD",
             "07 · Spatial Canvas", "08 · HMAS Hierarchy",
             "09 · HMAS Governance", "10 · States & Refusals",
             "11 · Motion & Live"]
    for name in order:
        slug = name.split("· ")[1].lower().replace(" ", "-").replace("&", "and")
        p = os.path.join(EXPORTS, f"hivemind-{slug}.png").replace("\\", "/")
        add("file.export", ids=[FRAME_IDS[name]], format="png", path=p, scale=1)
        p2 = os.path.join(EXPORTS, f"hivemind-{slug}@2x.png").replace("\\", "/")
        add("file.export", ids=[FRAME_IDS[name]], format="png", path=p2, scale=2)

    svg = os.path.join(EXPORTS, "hivemind-brand-lockups.svg").replace("\\", "/")
    add("file.export", ids=[FRAME_IDS["04 · Brand Lockups"]], format="svg", path=svg)

    final = os.path.join(ROOT, "HiveMind-Style-System.figcraft").replace("\\", "/")
    add("file.saveAs", path=final)

    with open(os.path.join(HERE, "commands_phase2.json"), "w", encoding="utf-8") as fh:
        json.dump(CMDS, fh, indent=0)
    print(f"phase2: {len(CMDS)} commands, {len(FRAME_IDS)} frames exported")


if __name__ == "__main__":
    which = sys.argv[1] if len(sys.argv) > 1 else "1"
    if which == "1":
        phase1()
    elif which == "2":
        # frame ids are read back from phase 1: the motion frame is
        # created after all existing board content to preserve prior ids.
        expected = json.load(open(os.path.join(HERE, "expected_ids.json"),
                                 encoding="utf-8"))
        for row in expected:
            if row["label"].startswith("frame:"):
                FRAME_IDS[row["label"][6:]] = row["id"]
        phase2()
    else:
        raise SystemExit("usage: build_hive_style.py [1|2]")
