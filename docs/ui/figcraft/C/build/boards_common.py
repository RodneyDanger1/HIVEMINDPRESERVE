"""Shared model for the HiveMind v1 C boards (FigCraft-C; palette copied from FigCraft-B, unchanged).

Tokens are the 20 printed PDF swatches (evidence/pdf/pages.txt p2:20-79) plus the
recorded corrections from reports/ui/UI-REQUIREMENTS.md (meta -> ink/500 #75663F,
focus ring -> ink/700 #4E3A15). Nothing here is invented: every id comes from
`figcraft-cli commands --json` (597 rows, figcraft-cli 0.11.2). The palette is
copied verbatim from FigCraft-B; nothing is invented for C.
"""
import math

# --------------------------------------------------------------- printed tokens
C = {
    "canvas": "#F8F4EB", "paper": "#FFFFFF", "panel_alt": "#FBF8F1",
    "recess": "#F1E9CF", "hairline": "#E5E1D8",
    "honey100": "#FBF3DC", "honey200": "#F6E7BE", "honey300": "#F0C050",
    "honey400": "#E8B850", "honey500": "#D9A02F", "honey600": "#C88030",
    "honey700": "#8A5A1C", "honey800": "#604818",
    "ink900": "#282010", "ink800": "#3F3115", "ink700": "#4E3A15",
    "ink500": "#75663F",
    "amber": "#B7791F", "signal": "#B23A2E", "verify": "#3F7A55",
}

# PDF p2:80-92 printed contrast rows (reproduce; used in labels only)
CONTRAST = [("ink-900 / canvas", "14.67:1", "AAA"),
            ("ink-900 / honey-400", "8.74:1", "AAA"),
            ("white / ink-800", "12.64:1", "AAA"),
            ("amber / canvas", "3.32:1", "AA-large only"),
            ("error / canvas", "5.41:1", "AA"),
            ("verify / canvas", "4.64:1", "AA")]

# UI-WIRING-GUIDE R-U4 / UI-COMMANDS-v1 §4: one enum, >=13 members, each with a
# shape + text cue so colour is never the only channel.
STATES = [
    ("healthy", "verify", "circle"), ("running", "honey500", "circle"),
    ("queued", "ink500", "square"), ("failed", "signal", "triangle"),
    ("completed", "verify", "circle"), ("cancelled", "ink500", "square"),
    ("killed", "signal", "diamond"), ("planning", "honey700", "circle"),
    ("thinking", "honey700", "circle"), ("awaiting_approval", "amber", "diamond"),
    ("streaming", "honey600", "circle"), ("reconnecting", "amber", "diamond"),
    ("replay", "ink700", "square"), ("empty", "ink500", "square"),
    ("loading", "ink500", "circle"),
]
STATE_COLOR = dict((s[0], C[s[1]]) for s in STATES)
STATE_SHAPE = dict((s[0], s[2]) for s in STATES)

# UI-COMMANDS-v1 §2 frozen command names (labels on mutating controls)
COMMANDS = {
    "SUBMIT_PROMPT": "S03 composer", "SPAWN_SUB_HIVE": "S04", "REWIRE_TASK": "S05",
    "CANCEL_RUN": "S06", "KILL_WORKER": "S07", "TEMPLATE_INSERT": "S12",
    "HISTORY_REVISION_SET": "S10", "VIEWPORT_SET": "S20", "VIEWPORT_RECENTER": "S20",
    "DIFF_PROPOSE": "S25 (DECISION-GATED)",
}

# PDF-printed values used verbatim by the boards
HUD = ("4 SUPERAGENTS", "12 WORKERS", "98.2% GREEN", "+3.4%")
SCRUB = ("09:41:07", "routing state 12 of 38")
SUB_HIVE = ("Backend Architecture", "RUNNING", "SUPERAGENT",
            "decomposes intent, owns the sub-hive", "3 workers", "12.4k ctx")
WORKER = ("tailwind-builder", "WORKER", "react/vite", "55%")
TOOL = ("Prisma Generator", "TOOL", "schema.prisma", "0.78")
BROKEN = ("Compile failed", "db-schema", "E0421", "retry 2/3",
          "Prisma client type mismatch on Run.sessionId")
CHAT = ("Hive AI Assistant",
        "Generate a Python script for sentiment analysis.",
        "Certainly! Here is the code...",
        ("import libraries,", "Load dataset,", "Preprocess data..."),
        "Type your prompt...")
NAV = ["Workspace", "Agents", "Data", "Models", "Routing", "Settings"]
TABS = ["Workspace", "Swarm", "Routing", "History"]
FILES = ["router.ts", "schema.prisma", "agent.spec.ts", "main.tsx", "beam.rs"]
TEMPLATES = ["Content Writer", "Data Analyst", "Code Reviewer", "Chatbot"]
TIP = ("Drag a worker onto another super-agent to re-route its context "
       "stream live.")

# Warm-ink elevation: every printed tier prints the same #282010 @7-13%
# (PDF p4:137-146; unreconciled - recorded as a single value here).
SHADOW_INK = {"r": 0.157, "g": 0.125, "b": 0.063, "a": 0.10}


def r2(v):
    return round(v, 2)


def hex_subpath(cx, cy, r, rotation=0.0):
    pts = []
    for i in range(6):
        a = math.radians(60 * i - 90 + rotation)
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    d = "M %.2f %.2f " % pts[0]
    for p in pts[1:]:
        d += "L %.2f %.2f " % p
    return d + "Z "


def honeycomb_path(x0, y0, x1, y1, r=38.0):
    """One SVG path, many hexagonal subpaths: the soft honeycomb lattice."""
    h = math.sqrt(3) * r
    parts, row, y = [], 0, y0 - 0.5 * r
    while y <= y1 + r:
        off = 0.0 if row % 2 == 0 else h / 2.0
        x = x0 - h + off
        while x <= x1 + h:
            parts.append(hex_subpath(x, y, r))
            x += h
        y += 1.5 * r
        row += 1
    return "".join(parts).strip()


def window_path(x0, y0, x1, y1, r=34.0):
    """Honeycomb clipped to a small region (interaction/hero fragments)."""
    return honeycomb_path(x0, y0, x1, y1, r)


def quad(p0, p1, p2, t):
    u = 1.0 - t
    return (u * u * p0[0] + 2 * u * t * p1[0] + t * t * p2[0],
            u * u * p0[1] + 2 * u * t * p1[1] + t * t * p2[1])


def quad_angle(p0, p1, p2, t):
    dx = 2 * (1 - t) * (p1[0] - p0[0]) + 2 * t * (p2[0] - p1[0])
    dy = 2 * (1 - t) * (p1[1] - p0[1]) + 2 * t * (p2[1] - p1[1])
    return math.degrees(math.atan2(dy, dx))


def hex_path_pts(cx, cy, r, rotation=0.0):
    out = []
    for i in range(6):
        a = math.radians(60 * i - 90 + rotation)
        out.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return out


class Board:
    """One page = one top-level 1440x900 frame. Every element is a creation
    command carrying its parent frame id; the real node id comes back from the
    run results (never predicted)."""

    def __init__(self, code, title, frame_id, w=1440, h=900, stem=""):
        self.code = code
        self.title = title
        self.frame = frame_id
        self.w = w
        self.h = h
        self.stem = stem
        self.cmd = []
        self.el = []
        self.surfaces = {}
        self.effects = []          # (element idx, tier e1|e2|e3)

    # ---------------------------------------------------------------- plumbing
    def _add(self, command, params, name, surface=None, effect=None,
             no_parent=False):
        p = dict(params)
        if not no_parent:
            p["parent"] = self.frame
        p["name"] = name
        self.cmd.append({"command": command, "params": p})
        idx = len(self.el)
        self.el.append({"idx": idx, "name": name, "id": None, "command": command})
        if surface:
            self.surfaces.setdefault(surface, name)
        if effect:
            self.effects.append((idx, effect))
        return idx

    def apply_ids(self, results):
        if len(results) != len(self.el):
            raise RuntimeError("%s: %d results for %d elements"
                               % (self.code, len(results), len(self.el)))
        for el, res in zip(self.el, results):
            rid = (res.get("result") or {}).get("id")
            if rid is None:
                raise RuntimeError("%s: no id returned for %s"
                                   % (self.code, el["name"]))
            el["id"] = rid
        return self.el

    # -------------------------------------------------------------- primitives
    def rect(self, x, y, w, h, fill=None, r=0, stroke=None, sw=1.0, dash=None,
             name="rect", opacity=None, effect=None, surface=None, rotation=None,
             align="inside"):
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
            props["strokeAlign"] = align
        if dash:
            props["dashes"] = dash
        if opacity is not None:
            props["opacity"] = opacity
        if rotation is not None:
            props["rotation"] = rotation
        return self._add("shape.rectangle",
                         {"x": x, "y": y, "width": w, "height": h, "props": props},
                         name, surface, effect)

    def ellipse(self, x, y, w, h, fill=None, stroke=None, sw=1.0, name="ellipse",
                effect=None, opacity=None, surface=None):
        props = {}
        if fill is not None:
            props["fill"] = fill
        if stroke:
            props["stroke"] = stroke
            props["strokeWeight"] = sw
        if opacity is not None:
            props["opacity"] = opacity
        return self._add("shape.ellipse",
                         {"x": x, "y": y, "width": w, "height": h, "props": props},
                         name, surface, effect)

    def poly(self, x, y, w, h, fill, pointCount=6, stroke=None, sw=1.0,
             rotation=None, name="poly", effect=None, surface=None, r=0):
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
                         name, surface, effect)

    def line(self, x, y, length, color, sw=1.0, dash=None, rotation=None,
             name="line", surface=None):
        props = {"stroke": color, "strokeWeight": sw}
        if dash:
            props["dashes"] = dash
        if rotation is not None:
            props["rotation"] = rotation
        return self._add("shape.line",
                         {"x": x, "y": y, "width": length, "props": props},
                         name, surface)

    def vector(self, path, x, y, fill=None, stroke=None, sw=1.0, w=None, h=None,
               name="vector", surface=None, opacity=None, effect=None,
               rotation=None):
        params = {"path": path, "x": x, "y": y}
        if w is not None and h is not None:
            params["width"] = w
            params["height"] = h
        props = {}
        if fill is not None:
            props["fill"] = fill
        elif stroke:
            props["fill"] = "none"
        if stroke:
            props["stroke"] = stroke
            props["strokeWeight"] = sw
        if opacity is not None:
            props["opacity"] = opacity
        if rotation is not None:
            props["rotation"] = rotation
        params["props"] = props
        return self._add("vector.create", params, name, surface, effect)

    def text(self, x, y, s, size, family, style, color, width=None, line=None,
             tracking=None, align=None, name="text", surface=None, case=None,
             rotation=None):
        props = {"fontSize": size, "fontFamily": family, "fontStyle": style,
                 "fill": color}
        if line is not None:
            props["lineHeight"] = line
        if tracking:
            props["letterSpacing"] = "%s%%" % tracking
        if align:
            props["textAlign"] = align
        if case:
            props["textCase"] = case
        if rotation is not None:
            props["rotation"] = rotation
        params = {"text": s, "x": x, "y": y, "props": props}
        if width:
            params["width"] = width
        return self._add("text.create", params, name, surface)

    # ------------------------------------------------------------- text styles
    def eyebrow(self, x, y, s, color=None, width=None, align=None, name="eyebrow",
                surface=None, size=10.5):
        return self.text(x, y, s, size, "Consolas", "Regular",
                         color or C["ink700"], width=width, line=15,
                         tracking=8, align=align, name=name, surface=surface)

    def meta(self, x, y, s, width=None, color=None, align=None, size=9.5,
             name="meta", surface=None):
        return self.text(x, y, s, size, "Consolas", "Regular",
                         color or C["ink500"], width=width, line=13,
                         tracking=2, align=align, name=name, surface=surface)

    def label(self, x, y, s, width=None, color=None, align=None, name="label",
              surface=None):
        return self.text(x, y, s, 12, "Inter", "SemiBold",
                         color or C["ink700"], width=width, line=16,
                         tracking=8, align=align, name=name, surface=surface)

    def name_t(self, x, y, s, size=14.5, color=None, width=None, line=None,
               name="name", surface=None):
        return self.text(x, y, s, size, "Inter", "SemiBold",
                         color or C["ink900"], width=width,
                         line=line or size + 4, name=name, surface=surface)

    def body(self, x, y, s, width, size=11.0, color=None, line=None,
             name="body", surface=None):
        return self.text(x, y, s, size, "Inter", "Regular",
                         color or C["ink700"], width=width,
                         line=line or round(size * 1.45), name=name,
                         surface=surface)

    def code_t(self, x, y, s, width=None, size=10.5, color=None, name="code",
             surface=None, line=None):
        return self.text(x, y, s, size, "Consolas", "Regular",
                         color or C["paper"], width=width,
                         line=line or round(size * 1.5), name=name,
                         surface=surface)

    # ------------------------------------------------------------- compound bits
    def pill(self, x, y, w, h, fill, s, color, size=9, stroke=None, sw=1.0,
             name="pill", surface=None, tracking=6, r=999):
        self.rect(x, y, w, h, fill, r=r, stroke=stroke, sw=sw,
                  name=name + "/bg", surface=surface)
        self.text(x, y + (h - size - 2) / 2.0, s, size, "Inter", "SemiBold",
                  color, width=w, align="center", line=size + 2,
                  tracking=tracking, name=name + "/label")
        return x + w

    def dot(self, x, y, size, shape, color, name="dot", surface=None):
        if shape == "circle":
            return self.ellipse(x, y, size, size, fill=color, name=name,
                                surface=surface)
        if shape == "square":
            return self.rect(x, y + size * 0.1, size, size * 0.8, fill=color,
                             r=2, name=name, surface=surface)
        n = 3 if shape == "triangle" else 4
        rot = 0 if n == 3 else 45
        return self.poly(x, y, size, size, color, pointCount=n, rotation=rot,
                         name=name, surface=surface)

    def state_dot(self, x, y, state, size=9, name="state", surface=None):
        self.dot(x, y, size, STATE_SHAPE.get(state, "circle"),
                 STATE_COLOR.get(state, C["ink500"]), name=name + "/" + state,
                 surface=surface)
        return x + size

    # ------------------------------------------------------------------- bees
    def bee(self, cx, cy, s, angle_deg, name="bee", body=None, wing=None,
            stripe=None):
        """Routing arrow: bee flies along +travel direction (angle_deg).

        Geometry is rotated in Python so each node keeps an unrotated box.
        """
        a = math.radians(angle_deg)
        ca, sa = math.cos(a), math.sin(a)

        def T(x, y):
            return (cx + x * ca - y * sa, cy + x * sa + y * ca)

        def P(p):
            return "%.2f %.2f " % p

        # body + head + stinger (one filled path, three subpaths)
        body_pts = [T(0.18 * s, 0), T(0.06 * s, -0.21 * s), T(-0.24 * s, -0.21 * s),
                    T(-0.40 * s, 0), T(-0.24 * s, 0.21 * s), T(0.06 * s, 0.21 * s)]
        d = "M " + P(body_pts[0]) + "".join("L " + P(p) for p in body_pts[1:]) + "Z "
        # head: octagon at the nose
        hc = T(0.26 * s, 0.0)
        hp = []
        for i in range(8):
            ang = math.radians(45 * i)
            hp.append((hc[0] + 0.13 * s * math.cos(ang),
                       hc[1] + 0.13 * s * math.sin(ang)))
        d += "M " + P(hp[0]) + "".join("L " + P(p) for p in hp[1:]) + "Z "
        # stinger trails the body (-travel direction)
        st = [T(-0.38 * s, -0.07 * s), T(-0.62 * s, 0.0), T(-0.38 * s, 0.07 * s)]
        d += "M " + P(st[0]) + "".join("L " + P(p) for p in st[1:]) + "Z"
        self.vector(d, 0, 0, fill=body or C["honey600"], name=name + "/body")

        # wings: two leaves rising from the thorax
        w1 = [T(-0.10 * s, -0.16 * s), T(-0.20 * s, -0.58 * s),
              T(0.04 * s, -0.52 * s), T(0.10 * s, -0.24 * s)]
        w2 = [T(-0.02 * s, -0.16 * s), T(-0.02 * s, -0.66 * s),
              T(0.20 * s, -0.52 * s), T(0.16 * s, -0.22 * s)]
        wd = ("M " + P(w1[0]) + "C " + P(w1[1]) + P(w1[2]) + P(w1[3]) + "Z ")
        wd += ("M " + P(w2[0]) + "C " + P(w2[1]) + P(w2[2]) + P(w2[3]) + "Z")
        self.vector(wd, 0, 0, fill=wing or C["honey200"], name=name + "/wings")

        # abdomen bands (two short strokes across the body)
        s1a, s1b = T(-0.10 * s, -0.19 * s), T(-0.10 * s, 0.19 * s)
        s2a, s2b = T(-0.22 * s, -0.17 * s), T(-0.22 * s, 0.17 * s)
        sd = "M " + P(s1a) + "L " + P(s1b) + " M " + P(s2a) + "L " + P(s2b)
        self.vector(sd, 0, 0, stroke=stripe or C["ink900"], sw=max(1.0, 0.055 * s),
                    name=name + "/bands")

    def bee_edge(self, p0, p1, ctrl=None, color=None, dash=None, sw=1.4,
                 bee_t=0.78, bee_size=18.0, name="edge", label=None,
                 label_at=0.5):
        """Bee-line edge: quadratic vector + bee at bee_t flying source->target."""
        p0 = tuple(p0)
        p1 = tuple(p1)
        ctrl = ctrl or ((p0[0] + p1[0]) / 2.0, (p0[1] + p1[1]) / 2.0)
        d = "M %.2f %.2f Q %.2f %.2f %.2f %.2f" % (
            p0[0], p0[1], ctrl[0], ctrl[1], p1[0], p1[1])
        self.vector(d, 0, 0, stroke=color or C["honey600"], sw=sw,
                    name=name + "/line", opacity=None)
        if dash:
            self.cmd[-1]["params"]["props"]["dashes"] = dash
        bx, by = quad(p0, ctrl, p1, bee_t)
        ang = quad_angle(p0, ctrl, p1, bee_t)
        self.bee(bx, by, bee_size, ang, name=name + "/bee")
        if label:
            lx, ly = quad(p0, ctrl, p1, label_at)
            self.meta(lx - 34, ly - 18, label, width=90, align="center",
                      color=color or C["ink500"], name=name + "/label")


# ------------------------------------------------------------------ surfaces B
SURFACES_B = {
    "S01": ("Infinite canvas + honeycomb lattice", "viewport state only",
            "PDF:113-114,303-305"),
    "S10": ("Time-travel scrubber", "HISTORY_REVISION_SET", "PDF:246-247"),
    "S13": ("Mini-map 260x170", "client-only", "PDF:267-268"),
    "S19": ("Agent chips / state dot", "enum state -> dot + label",
            "PDF:211-217"),
    "S20": ("Zoom controls + Ctrl+0 hint", "VIEWPORT_SET / VIEWPORT_RECENTER",
            "PDF:303-304,183"),
    "S24": ("Elevation e1/e2/e3", "CSS elevation tokens (unreconciled)",
            "PDF:137-146"),
    "S25": ("Code-diff review modal (decision-gated)", "DIFF_* NOT FROZEN",
            "U3 G-03; UI-COMMANDS-v1 §2"),
}


# ------------------------------------------------------------------ surfaces C
# C surface rows: frozen S ids where the C board re-renders an existing surface,
# plus C-specific ids for the HMAS/state panels introduced by this set.
C_SURFACES = {
    "S01": ("Infinite canvas + honeycomb lattice", "viewport state only", "PDF:113-114,303-305"),
    "S02": ("HyperAgent hive chat core (pinned)", "CHAT_PIN", "PDF:175-184,296-302"),
    "S03": ("Composer / HyperAgent input", "SUBMIT_PROMPT", "PDF:235-237,302"),
    "S09": ("HUD throughput badge", "RUN_METRICS push; derived aggregate", "PDF:243-245,303"),
    "S10": ("Time-travel scrubber", "HISTORY_REVISION_SET", "PDF:246-247"),
    "S11": ("Nav rail 72px, 6 items", "view route state", "PDF:248-254,276-281"),
    "S12": ("Library panel 260px", "TEMPLATE_INSERT; manifest", "PDF:255-266,282-290"),
    "S13": ("Mini-map 260x170", "client-only", "PDF:267-268"),
    "S14": ("Refusal toast / named refusal card", "ERROR frame -> toast + log + ledger", "PDF:269-273"),
    "S15": ("SuperAgent sub-hive card", "node render + metrics push", "PDF:185-191"),
    "S16": ("WorkerAgent card", "metrics push; progress", "PDF:192-196,222"),
    "S19": ("Agent chips / state dot", "enum state -> dot + shape/text cue", "PDF:211-217"),
    "S20": ("Zoom controls + Ctrl+0 hint", "VIEWPORT_SET / VIEWPORT_RECENTER", "PDF:303-304,183"),
    "S22": ("File/code hierarchy viewer", "workspace-jailed file list", "PDF:291-295"),
    "S23": ("Brand lockups / bee glyph", "static tokens", "PDF:148-169"),
    "S25": ("Code-diff review modal (decision-gated)", "DIFF_* NOT FROZEN - do not implement", "U3 G-03; UI-COMMANDS-v1 section 2"),
    "S26": ("First-run / empty / loading states", "empty + loading truth states", "PDF silent"),
    "S28": ("Project identity", "project_id / string table", "PDF:256,275-282"),
    "C-HMAS-PROFILES": ("Profiles + import panel", "profile state + import action", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-MEMORY": ("Memory list + clear", "clear requires confirmation", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-MAILBOX": ("Mailbox", "cross-branch messages", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-PROJECTS": ("Projects", "project identity rows", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-USAGE": ("Usage", "derived counters", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-APPROVAL": ("Cross-branch message approval", "APPROVAL_RESOLVE with nonce", "PROPOSED-ADDITION (task-6)"),
    "C-STATE-RECONNECT": ("Reconnect state", "SERVER_HELLO / RESYNC_REQUIRED", "PROPOSED-ADDITION (task-6)"),
    "C-STATE-KILL": ("Kill confirmation", "KILL_WORKER confirmation", "PROPOSED-ADDITION (task-6)"),
    "C-STATE-REPLAY": ("Replay read-only", "HISTORY_REVISION_SET / HISTORY_SNAPSHOT", "UI-COMMANDS-v1 section 3"),
}
