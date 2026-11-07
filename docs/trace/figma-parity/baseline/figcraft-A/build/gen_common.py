"""Shared drawing model for the HiveMind v1 FigCraft design boards.

All geometry is authored in page units (px) inside one top-level frame per page.
Every command id comes from `figcraft-cli commands --json` (597 rows); every
colour/size used here is either a printed PDF token or a recorded correction from
reports/ui/UI-REQUIREMENTS.md. Nothing is invented.
"""
from collections import OrderedDict

# ---------------------------------------------------------------- printed tokens
COLOR = {
    "canvas": "#F8F4EB", "paper": "#FFFFFF", "panel_alt": "#FBF8F1",
    "recess": "#F1E9CF", "hairline": "#E5E1D8",
    "honey100": "#FBF3DC", "honey200": "#F6E7BE", "honey300": "#F0C050",
    "honey400": "#E8B850", "honey500": "#D9A02F", "honey600": "#C88030",
    "honey700": "#8A5A1C", "honey800": "#604818",
    "ink900": "#282010", "ink800": "#3F3115", "ink700": "#4E3A15",
    "ink500": "#75663F",
    "amber": "#B7791F", "signal": "#B23A2E", "verify": "#3F7A55",
}
C = COLOR

# 20 printed swatches, PDF p2:20-79. Token path maps 1:1 to a FigCraft colour variable.
TOKENS = [
    ("Canvas", "surface/canvas", C["canvas"]),
    ("Paper", "surface/panel", C["paper"]),
    ("Panel Alt", "surface/panel-alt", C["panel_alt"]),
    ("Recess", "surface/recessed", C["recess"]),
    ("Hairline", "border/hairline", C["hairline"]),
    ("Honey 100", "honey/100", C["honey100"]),
    ("Honey 200", "honey/200", C["honey200"]),
    ("Honey 300", "honey/300", C["honey300"]),
    ("Honey 400", "honey/400", C["honey400"]),
    ("Honey 500", "honey/500", C["honey500"]),
    ("Honey 600", "honey/600", C["honey600"]),
    ("Honey 700", "honey/700", C["honey700"]),
    ("Honey 800", "honey/800", C["honey800"]),
    ("Ink 900", "ink/900", C["ink900"]),
    ("Ink 800", "ink/800", C["ink800"]),
    ("Ink 700", "ink/700", C["ink700"]),
    ("Ink 500", "ink/500", C["ink500"]),
    ("Amber", "accent/amber", C["amber"]),
    ("Signal", "state/error", C["signal"]),
    ("Verify", "state/success", C["verify"]),
]

# PDF p2:80-92 measured rows.
CONTRAST_PRINTED = [
    ("ink-900/canvas", "14.67:1", "AAA"),
    ("ink-900/honey-400", "8.74:1", "AAA"),
    ("white/ink-800", "12.64:1", "AAA"),
    ("amber/canvas", "3.32:1", "AA-LG"),
    ("error/canvas", "5.41:1", "AA"),
    ("verify/canvas", "4.64:1", "AA"),
]

# PDF p3:111-124 type steps: id, size, line, tracking %, family, style, sample
TYPE_SCALE = [
    ("display/xxl", 96, 100, -3.0, "Playfair Display", "SemiBold", "Aa"),
    ("display/l", 56, 60, -2.0, "Playfair Display", "SemiBold", "HiveMind"),
    ("heading/h1", 40, 48, -1.5, "Playfair Display", "SemiBold", "Agent Components"),
    ("heading/h3", 24, 32, -0.5, "Inter", "SemiBold", "SuperAgent - Backend Architecture"),
    ("body/l", 17, 28, 0.0, "Inter", "Regular",
     "Drop a repository into the core hive and the HyperAgent maps its architecture."),
    ("label", 12, 16, 8.0, "Inter", "SemiBold", "ACTIVE AGENTS"),
    ("mono/code", 13, 22, 0.0, "Consolas", "Regular",
     "const route = hive.snap(worker, superAgentB);"),
    ("mono/meta", 11, 16, 8.0, "Consolas", "Regular",
     "NODE_SPAWN  ·  NODE_DRAG  ·  NODE_REWIRE"),
]

SPACING = [4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96]
RADII = [4, 8, 12, 16, 24, 48, 999]
ELEVATION = [("e1 / node", "cards resting on canvas"),
             ("e2 / float", "chat core, drawers, menus"),
             ("e3 / hive", "the hive itself, behind the chat")]

# UI-WIRING-GUIDE §1: surface id -> (name, wiring, PDF lines)
SURFACES = OrderedDict([
    ("S01", ("Infinite canvas + honeycomb background", "viewport state only", "PDF:113-114,303-305")),
    ("S02", ("HyperAgent hive chat core (pinned)", "CHAT_PIN", "PDF:175-184,296-302")),
    ("S03", ("Composer / HyperAgent input", "SUBMIT_PROMPT", "PDF:235-237,302")),
    ("S04", ("Button: Spawn sub-hive", "SPAWN_SUB_HIVE", "PDF:238-240")),
    ("S05", ("Button: Re-route", "REWIRE_TASK", "PDF:239")),
    ("S06", ("Button: Cancel", "CANCEL_RUN", "PDF:239")),
    ("S07", ("Button: Kill worker", "KILL_WORKER", "PDF:239")),
    ("S08", ("Tabs: Workspace / Swarm / Routing / History", "view route state", "PDF:241-242")),
    ("S09", ("HUD throughput badge", "RUN_METRICS push", "PDF:243-245,303")),
    ("S10", ("Time-travel scrubber", "HISTORY_REVISION_SET", "PDF:246-247")),
    ("S11", ("Nav rail (72px, 6 items)", "view route state", "PDF:248-254,276-281")),
    ("S12", ("Library panel (260px)", "TEMPLATE_INSERT", "PDF:255-266,282-290")),
    ("S13", ("Mini-map (260x170)", "client-only", "PDF:267-268")),
    ("S14", ("Refusal toast", "ERROR frame", "PDF:269-273")),
    ("S15", ("SuperAgent sub-hive card", "node render + metrics", "PDF:185-191")),
    ("S16", ("WorkerAgent card", "metrics push", "PDF:192-196,222")),
    ("S17", ("ToolAgent card", "tool result + score", "PDF:197-204")),
    ("S18", ("Broken-route error card", "Error state from run/tool", "PDF:205-210")),
    ("S19", ("Agent chips / state dot", "enum state", "PDF:211-217")),
    ("S20", ("Zoom controls + shortcut hint", "VIEWPORT_SET", "PDF:304,183")),
    ("S21", ("Settings / key status", "settings read, E_KEY_MISSING", "PDF:254")),
    ("S22", ("File/code hierarchy viewer", "file list API", "PDF:291-295")),
    ("S23", ("Brand lockups / bee glyph", "static assets + tokens", "PDF:148-169")),
    ("S24", ("Elevation e1/e2/e3", "CSS elevation tokens", "PDF:137-146")),
    ("S25", ("Code-diff review modal (no artboard)", "DIFF_* decision-gated", "U3 G-03 PROPOSED")),
    ("S26", ("First-run / empty / loading / offline states", "UI-ADD-04 state pack", "PDF silent")),
    ("S27", ("Streamed reply / cards", "REPLY_DELTA / REPLY_DONE", "SOURCE src:1738")),
    ("S28", ("Project identity", "project_id / string table", "PDF:256,275-282")),
])

# 15-state vocabulary (UI-COMMANDS-v1 §4) with visual cue: colour token, shape, text cue.
STATES = [
    ("healthy", "verify", "circle"),
    ("running", "honey500", "circle"),
    ("queued", "ink500", "square"),
    ("failed", "signal", "triangle"),
    ("completed", "verify", "circle"),
    ("cancelled", "ink500", "square"),
    ("killed", "signal", "diamond"),
    ("planning", "honey700", "circle"),
    ("thinking", "honey700", "circle"),
    ("awaiting_approval", "amber", "diamond"),
    ("streaming", "honey600", "circle"),
    ("reconnecting", "amber", "diamond"),
    ("replay", "ink700", "square"),
    ("empty", "ink500", "square"),
    ("loading", "ink500", "circle"),
]
STATE_COLOR = dict((s[0], C[s[1]]) for s in STATES)
STATE_SHAPE = dict((s[0], s[2]) for s in STATES)


class Board:
    """One page = one top-level frame. Records commands and element metadata."""

    def __init__(self, code, title, frame_id, frame_w, frame_h):
        self.code = code
        self.title = title
        self.frame = frame_id
        self.w = frame_w
        self.h = frame_h
        self.cmd = []            # figcraft commands, creation order
        self.el = []             # element records, same order as create commands
        self.groups = OrderedDict()  # group name -> [element indices]
        self.surfaces = OrderedDict()  # surface id -> element index (first)
        self.paint_styles = []   # (style name, element index)
        self.text_styles = []    # (style name, element index)
        self.effect_styles = []  # (style name, element index)

    # ------------------------------------------------------------ primitives
    def _add(self, command, params, name, group=None, surface=None):
        p = dict(params)
        p["parent"] = self.frame
        p["name"] = name
        self.cmd.append({"command": command, "params": p})
        idx = len(self.el)
        self.el.append({"idx": idx, "name": name, "group": group,
                        "surface": surface, "id": None})
        if group:
            self.groups.setdefault(group, []).append(idx)
        if surface and surface not in self.surfaces:
            self.surfaces[surface] = idx
        return idx

    def rect(self, x, y, w, h, fill=None, r=0, stroke=None, sw=1.0,
             dash=None, name="rect", group=None, surface=None, opacity=None,
             rotation=None):
        props = {}
        if fill is not None:
            props["fill"] = fill
        elif stroke:
            props["fill"] = "none"
        if r:
            props["cornerRadius"] = r
        if stroke:
            props["stroke"] = stroke
            props["strokeWeight"] = sw
            props["strokeAlign"] = "inside"
        if dash:
            props["dashes"] = dash
        if opacity is not None:
            props["opacity"] = opacity
        if rotation is not None:
            props["rotation"] = rotation
        return self._add("shape.rectangle",
                         {"x": x, "y": y, "width": w, "height": h, "props": props},
                         name, group, surface)

    def ellipse(self, x, y, w, h, fill=None, stroke=None, sw=1.0, name="ellipse",
                group=None, surface=None, rotation=None):
        props = {}
        if fill is not None:
            props["fill"] = fill
        if stroke:
            props["stroke"] = stroke
            props["strokeWeight"] = sw
        if rotation is not None:
            props["rotation"] = rotation
        return self._add("shape.ellipse",
                         {"x": x, "y": y, "width": w, "height": h, "props": props},
                         name, group, surface)

    def poly(self, x, y, w, h, fill, pointCount=6, stroke=None, sw=1.0,
             name="poly", group=None, surface=None, rotation=None, r=0):
        props = {"pointCount": pointCount, "fill": fill}
        if stroke:
            props["stroke"] = stroke
            props["strokeWeight"] = sw
        if rotation is not None:
            props["rotation"] = rotation
        if r:
            props["cornerRadius"] = r
        return self._add("shape.polygon",
                         {"x": x, "y": y, "width": w, "height": h, "props": props},
                         name, group, surface)

    def line(self, x, y, length, color, sw=1.0, dash=None, rotation=None,
             name="line", group=None, surface=None):
        props = {"stroke": color, "strokeWeight": sw}
        if dash:
            props["dashes"] = dash
        if rotation is not None:
            props["rotation"] = rotation
        return self._add("shape.line",
                         {"x": x, "y": y, "width": length, "props": props},
                         name, group, surface)

    def vector(self, path, x, y, w=None, h=None, fill=None, stroke=None, sw=1.0,
               name="vector", group=None, surface=None):
        params = {"path": path, "x": x, "y": y}
        if w is not None and h is not None:
            params["width"] = w
            params["height"] = h
        props = {}
        if fill is not None:
            props["fill"] = fill
        if stroke:
            props["stroke"] = stroke
            props["strokeWeight"] = sw
        params["props"] = props
        return self._add("vector.create", params, name, group, surface)

    def text(self, x, y, s, size, family, style, color, width=None,
             line=None, tracking=None, align="left", name="text", group=None,
             surface=None, text_case=None):
        props = {"fontSize": size, "fontFamily": family, "fontStyle": style,
                 "fill": color}
        if line is not None:
            props["lineHeight"] = line
        if tracking:
            props["letterSpacing"] = "%s%%" % tracking
        if align != "left":
            props["textAlign"] = align
        if text_case:
            props["textCase"] = text_case
        params = {"text": s, "x": x, "y": y, "props": props}
        if width:
            params["width"] = width
        return self._add("text.create", params, name, group, surface)

    # --------------------------------------------------------- text shortcuts
    def eyebrow(self, x, y, s, color=None, name="eyebrow", group=None, width=None,
                align="left"):
        return self.text(x, y, s, 11, "Consolas", "Regular", color or C["ink700"],
                         width=width, line=16, tracking=8, name=name, group=group,
                         align=align)

    def section(self, x, y, w, s, note=None):
        """Uppercase mono label + right note + hairline rule."""
        note_w = 300 if w >= 900 else 150
        self.text(x, y, s.upper(), 11, "Consolas", "Regular", C["ink700"],
                  width=w - (note_w + 14 if note else 0), line=16, tracking=8,
                  name="sec/" + s[:24])
        if note:
            self.text(x + w - note_w, y, note, 10, "Consolas", "Regular", C["ink500"],
                      width=note_w, line=16, tracking=4, align="right",
                      name="secnote/" + s[:18])
        self.line(x, y + 22, w, C["hairline"], 1, name="secrule/" + s[:24])
        return y + 24

    def caption(self, x, y, s, width=None, color=None, align="left", name="cap"):
        return self.text(x, y, s, 9.5, "Consolas", "Regular", color or C["ink500"],
                         width=width, line=14, tracking=2, align=align, name=name)

    def body_text(self, x, y, s, width, size=13, color=None, line=None, group=None,
                  name="body"):
        return self.text(x, y, s, size, "Inter", "Regular", color or C["ink700"],
                         width=width, line=line or round(size * 1.55), group=group,
                         name=name)

    def name_text(self, x, y, s, size=14.5, color=None, width=None, group=None,
                  name="name"):
        return self.text(x, y, s, size, "Inter", "SemiBold", color or C["ink900"],
                         width=width, line=size + 4, group=group, name=name)

    # --------------------------------------------------------- compound atoms
    def pill(self, x, y, w, h, fill, s, color, size=9, stroke=None, sw=1.0,
             group=None, name="pill", align="center"):
        self.rect(x, y, w, h, fill, r=999, stroke=stroke, sw=sw, group=group,
                  name=name + "/bg")
        self.text(x, y + (h - size - 2) / 2.0, s, size, "Inter", "SemiBold", color,
                  width=w, align=align, line=size + 2, tracking=4, group=group,
                  name=name + "/label")
        return x + w

    def dot(self, x, y, size, shape, color, group=None, name="dot"):
        if shape == "circle":
            return self.ellipse(x, y, size, size, fill=color, group=group, name=name)
        if shape == "square":
            return self.rect(x, y + size * 0.1, size, size * 0.8, fill=color, r=2,
                             group=group, name=name)
        n = 3 if shape == "triangle" else 4
        rot = 0 if n == 3 else 45
        return self.poly(x, y, size, size, color, pointCount=n, rotation=rot,
                         group=group, name=name)

    def state_dot(self, x, y, state, size=9, group=None, name="state"):
        self.dot(x, y, size, STATE_SHAPE.get(state, "circle"),
                 STATE_COLOR.get(state, C["ink500"]), group=group,
                 name=name + "/" + state)
        return x + size

    def hex_bee(self, x, y, s, hex_fill, bee_body, bee_head, wing, stripe=None,
                mono=False, group=None, name="mark", bee=True, thorax=None,
                surface=None):
        """Hexagon mark (pointy-top) with an inner bee flying right (+x)."""
        self.poly(x, y, s, s, hex_fill, pointCount=6, group=group,
                  name=name + "/hex", surface=surface)
        if not bee or s < 20:
            return
        u = s / 56.0
        cx, cy = x + s * 0.5, y + s * 0.5
        # wings behind the body, angled out from the thorax
        self.ellipse(cx - 10 * u, cy - 15 * u, 14 * u, 6.5 * u, fill=wing,
                     rotation=-24, group=group, name=name + "/wing-l")
        self.ellipse(cx + 1 * u, cy - 16 * u, 13 * u, 6.5 * u, fill=wing,
                     rotation=20, group=group, name=name + "/wing-r")
        # stinger trails the body (-x): the bee is a routing arrow flying +x
        self.poly(cx - 22 * u, cy - 2.8 * u, 8 * u, 5.5 * u, bee_head,
                  pointCount=3, rotation=90, group=group, name=name + "/stinger")
        # abdomen, bands, thorax, head
        self.ellipse(cx - 17 * u, cy - 6.5 * u, 19 * u, 13 * u, fill=bee_body,
                     group=group, name=name + "/abdomen")
        if stripe and not mono:
            self.rect(cx - 14.5 * u, cy - 6 * u, 2.0 * u, 12 * u, fill=stripe,
                      group=group, name=name + "/stripe-1")
            self.rect(cx - 11.0 * u, cy - 6 * u, 2.0 * u, 12 * u, fill=stripe,
                      group=group, name=name + "/stripe-2")
        self.ellipse(cx - 2.5 * u, cy - 5 * u, 10 * u, 10 * u,
                     fill=thorax or bee_head, group=group, name=name + "/thorax")
        self.ellipse(cx + 5.5 * u, cy - 4 * u, 8.5 * u, 8.5 * u, fill=bee_head,
                     group=group, name=name + "/head")
        self.line(cx + 8 * u, cy - 5.5 * u, 6.5 * u, bee_head, 1.3 * u,
                  rotation=-45, group=group, name=name + "/antenna-1")
        self.line(cx + 9 * u, cy - 3 * u, 5 * u, bee_head, 1.3 * u, rotation=-12,
                  group=group, name=name + "/antenna-2")

    def button(self, x, y, w, h, label, variant, state, group=None, name="btn"):
        """Four buttons x five interaction states, tokens only."""
        h = max(h, 40)
        r = 10  # recorded off-scale exception (PDF p7:240)
        fill = C["honey300"]
        text = C["ink900"]
        stroke = None
        sw = 1.0
        underline = None
        if variant == "primary":
            fill, text = C["ink900"], C["paper"]
            if state == "hover":
                fill = C["ink800"]
            elif state == "pressed":
                underline = C["honey300"]
            elif state == "disabled":
                fill, text, stroke, sw = C["recess"], C["ink500"], C["hairline"], 1.0
            elif state == "refused":
                fill, text, stroke, sw = C["paper"], C["signal"], C["signal"], 1.5
        elif variant == "secondary":
            fill, text, stroke, sw = C["paper"], C["ink900"], C["ink700"], 1.5
            if state == "hover":
                fill = C["honey100"]
            elif state == "pressed":
                fill = C["honey200"]
                stroke = C["ink900"]
                underline = C["honey700"]
            elif state == "disabled":
                fill, text, stroke, sw = C["paper"], C["ink500"], C["hairline"], 1.0
            elif state == "refused":
                text, stroke, sw = C["signal"], C["signal"], 1.5
        elif variant == "ghost":
            fill, text, stroke, sw = C["canvas"], C["ink700"], C["hairline"], 1.0
            if state == "hover":
                fill = C["panel_alt"]
                stroke, sw = C["ink500"], 1.0
            elif state == "pressed":
                fill = C["recess"]
                stroke, sw = C["ink500"], 1.0
                underline = C["ink700"]
            elif state == "disabled":
                fill, text, stroke = C["canvas"], C["ink500"], C["hairline"]
            elif state == "refused":
                fill, text, stroke, sw = C["canvas"], C["signal"], C["signal"], 1.5
        else:  # danger
            fill, text, stroke, sw = C["paper"], C["signal"], C["signal"], 1.5
            if state == "hover":
                fill, text = C["signal"], C["paper"]
            elif state == "pressed":
                fill, text = C["signal"], C["paper"]
                underline = C["ink900"]
            elif state == "disabled":
                fill, text, stroke, sw = C["recess"], C["ink500"], C["hairline"], 1.0
            elif state == "refused":
                fill, text, stroke, sw = C["paper"], C["signal"], C["signal"], 1.5
        self.rect(x, y, w, h, fill, r=r, stroke=stroke, sw=sw, group=group,
                  name=name + "/" + state + "/bg")
        if underline:
            self.rect(x + r, y + h - 3, w - 2 * r, 1.5, underline,
                      group=group, name=name + "/" + state + "/underline")
        self.text(x, y + (h - 17) / 2.0, label, 13, "Inter", "SemiBold", text,
                  width=w, align="center", line=17, group=group,
                  name=name + "/" + state + "/label")
        return x + w

    def swatch_row(self, x, y, w, token, group=None, surface=None):
        """One printed swatch row: 30px chip, name, hex, token path."""
        display, path, hexv = token
        chip = self.rect(x, y + 6, 30, 30, fill=hexv, r=6, stroke=C["hairline"],
                         sw=1.0, group=group, name="swatch/%s/chip" % path)
        self.paint_styles.append((path, chip))
        self.text(x + 40, y + 1, display, 12, "Inter", "SemiBold", C["ink900"],
                  width=w - 150, line=16, group=group, name="swatch/%s/name" % path)
        self.text(x + 40, y + 20, hexv, 10, "Consolas", "Regular", C["ink700"],
                  width=82, line=13, group=group, name="swatch/%s/hex" % path)
        self.text(x + w - 122, y + 20, path, 9, "Consolas", "Regular", C["ink500"],
                  width=122, line=13, align="right", group=group,
                  name="swatch/%s/path" % path)
        return y
