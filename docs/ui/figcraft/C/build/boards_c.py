"""FigCraft-C board builders: C-shell, C-hmas, C-states (HiveMind v1).

Every id used is a real figcraft-cli 0.11.2 command (the Board helpers in
boards_common.py). Tokens are the 20 printed PDF swatches copied from
FigCraft-B; nothing is invented. User-facing copy is HiveMind (never the
legacy product name) and strings reused from the PDF/string table are noted in
C-ASSETS.json provenance rows.

Layouts: three 1440x900 boards.
  C-shell : 72px nav rail, 260px library, canvas + honeycomb lattice, centred
            HUD, 336x330 chat core, 260x170 minimap, control bar + scrubber.
  C-hmas  : hierarchy/topology, profiles/import, memory, mailbox, projects,
            usage, cross-branch message approval.
  C-states: named refusal, reconnect, empty/loading, kill confirmation, replay
            read-only, DECISION-GATED diff explanation.
"""
import math

from boards_common import (
    Board, C, C_SURFACES, CHAT, FILES, HUD, NAV, SCRUB, TABS, TEMPLATES, TIP,
    STATE_COLOR, honeycomb_path, window_path,
)

INK = C["ink900"]
BODY = C["ink700"]
META = C["ink500"]


# --------------------------------------------------------------------- helpers
def panel(b, x, y, w, h, fill, r=16, stroke=None, name="panel", surface=None, sw=1.0):
    b.rect(x, y, w, h, fill=fill, r=r, stroke=stroke, sw=sw, name=name,
           surface=surface)


def rule(b, x, y, w, color=None, name="rule"):
    b.rect(x, y, w, 1, fill=color or C["hairline"], name=name)


def chip(b, x, y, w, h, text, fill, color, size=8.5, stroke=None, name="chip"):
    b.rect(x, y, w, h, fill=fill, r=h / 2.0, stroke=stroke, sw=1.0,
           name=name + "/bg")
    b.text(x, y + (h - size - 2) / 2.0, text, size, "Inter", "SemiBold", color,
           width=w, align="center", line=size + 2, tracking=6,
           name=name + "/label")


def button(b, x, y, w, h, text, fill, color, stroke=None, size=9.5,
           name="btn", surface=None):
    b.rect(x, y, w, h, fill=fill, r=8, stroke=stroke, sw=1.0,
           name=name + "/bg", surface=surface)
    b.text(x, y + (h - size - 2) / 2.0, text, size, "Inter", "SemiBold", color,
           width=w, align="center", line=size + 2, tracking=5,
           name=name + "/label")


def brand(b, x, y, size=15.5, prefix="brand"):
    b.poly(x, y, 26, 26, C["honey300"], pointCount=6, stroke=C["honey500"],
           sw=1.0, name=prefix + "/logo-hex")
    b.bee(x + 13, y + 13, 13, -90, name=prefix + "/logo-bee")
    b.name_t(x + 34, y + 2, "HiveMind", size, name=prefix + "/wordmark")
    b.meta(x + 34, y + size + 7, "hierarchical agent mesh", size=8,
           name=prefix + "/wordmark-meta")


def state_row(b, x, y, state, label, meta="", name="row", label_w=150):
    b.state_dot(x, y, state, 9, name=name + "/dot")
    b.text(x + 15, y - 2, label, 9.5, "Inter", "SemiBold", INK,
           width=label_w, line=13, name=name + "/label")
    if meta:
        b.meta(x + 15 + label_w, y, meta, width=110, size=8, name=name + "/meta")


def honey(b, x0, y0, x1, y1, r=36.0, opacity=0.13, name="honeycomb"):
    b.vector(honeycomb_path(x0, y0, x1, y1, r), 0, 0, stroke=C["honey500"],
             sw=1.0, opacity=opacity, name=name)


def bee_node(b, cx, cy, r, label, state, prefix, surface=None):
    b.poly(cx - r, cy - r, 2 * r, 2 * r, C["honey200"], pointCount=6,
           stroke=C["honey500"], sw=1.0,
           name=prefix + "/hex", surface=surface)
    b.bee(cx, cy, r * 0.58, -90, name=prefix + "/bee")
    b.text(cx - 80, cy + r + 5, label, 9.0, "Inter", "SemiBold", INK,
           width=160, align="center", line=12, name=prefix + "/label")
    b.state_dot(cx + r * 0.55, cy - r * 0.95, state, 9, name=prefix + "/state")


def eyebrow(b, x, y, text, name="eyebrow", color=None):
    b.eyebrow(x, y, text, color=color or META, name=name)


# ------------------------------------------------------------------ C-shell
def build_shell(frame_id):
    b = Board("00", "HiveMind / shell", frame_id, 1440, 900, "c-shell")

    # background + honeycomb lattice (S01)
    b.rect(0, 0, 1440, 900, fill=C["canvas"], name="shell/bg")
    b.vector(honeycomb_path(360, 44, 1500, 940, 38), 0, 0, stroke=C["honey500"],
             sw=1.0, opacity=0.14, name="s01/honeycomb-lattice", surface="S01")

    # 72px navigation rail (S11) + brand lockup (S23)
    panel(b, 0, 0, 72, 900, C["paper"], r=0, stroke=C["hairline"],
          name="s11/nav-rail", surface="S11")
    rule(b, 71, 0, 1, name="s11/rail-rule")
    b.name_t(12, 18, "HiveMind", 10.0, width=52, line=13, name="s23/wordmark")
    b.meta(12, 34, "v1 mesh", width=52, size=7.5, name="s23/wordmark-meta")
    b.poly(26, 54, 20, 20, C["honey300"], pointCount=6, stroke=C["honey500"],
           name="s23/logo-hex", surface="S23")
    b.bee(36, 64, 11, -90, name="s23/logo-bee")
    for i, item in enumerate(NAV[:-1]):
        y = 116 + i * 58
        active = i == 0
        if active:
            b.rect(10, y - 8, 52, 46, fill=C["honey100"], r=12,
                   name="s11/active-bg")
        b.poly(26, y, 20, 20, C["honey100"] if not active else C["honey300"],
               pointCount=6, stroke=C["honey500"], sw=1.0,
               name="s11/nav/%d/hex" % i)
        b.meta(10, y + 26, item.upper(), width=52, size=7.0, align="center",
               color=C["ink700"] if active else META,
               name="s11/nav/%d/label" % i)
    b.meta(10, 812, "SETTINGS", width=52, size=7.0, align="center",
           name="s11/settings/label")
    b.poly(26, 786, 20, 20, C["honey100"], pointCount=6, stroke=C["honey500"],
           name="s11/settings/hex")
    b.ellipse(20, 856, 32, 32, fill=C["ink900"], name="s11/account")
    b.meta(14, 868, "HM", width=44, size=8, align="center", color=C["paper"],
           name="s11/account-initials")

    # 260px left library panel (S12) with files (S22), templates, project (S28)
    panel(b, 72, 0, 260, 900, C["panel_alt"], r=0, stroke=C["hairline"],
          name="s12/library-panel", surface="S12")
    rule(b, 331, 0, 1, name="s12/library-rule")
    brand(b, 92, 18, size=15.0, prefix="libbrand")
    eyebrow(b, 92, 62, "PROJECT")
    b.name_t(92, 76, "HiveMind AI", 15.0, name="s28/project-name", surface="S28")
    b.meta(92, 96, "localhost · 127.0.0.1:8799 · v1.2.0-dev", width=220,
           size=8)
    # tabs
    for i, tab in enumerate(TABS):
        x = 92 + i * 56
        chip(b, x, 120, 52, 22, tab.upper()[:8],
             C["honey100"] if i == 0 else C["paper"],
             C["ink800"] if i == 0 else META, size=7.0,
             stroke=C["hairline"] if i else C["honey400"],
             name="s12/tab/%d" % i)
    # search
    b.rect(92, 160, 220, 30, fill=C["paper"], r=8, stroke=C["hairline"],
           name="s12/search")
    b.meta(104, 170, "Search files, templates", width=170, size=8)
    b.meta(286, 170, "/", width=16, size=8, align="center", name="s12/search-key")
    # files list (S22)
    eyebrow(b, 92, 208, "FILES · %d" % len(FILES))
    for i, name in enumerate(FILES):
        y = 226 + i * 25
        b.rect(92, y + 2, 10, 10, fill=C["honey100"], r=2,
               name="s22/file/%d/icon" % i, surface="S22" if i == 0 else None)
        b.text(108, y, name, 9.5, "Consolas", "Regular", INK, width=180,
               line=13, name="s22/file/%d/name" % i)
        if i == 0:
            b.meta(292, y, "ts", width=20, size=7.5, name="s22/file/%d/kw" % i)
    # templates
    eyebrow(b, 92, 372, "TEMPLATES · %d" % len(TEMPLATES))
    for i, name in enumerate(TEMPLATES):
        y = 390 + i * 24
        b.ellipse(92, y + 3, 6, 6, fill=C["honey500"], name="s12/tpl/%d/dot" % i)
        b.text(106, y, name, 9.0, "Inter", "Regular", BODY, width=200,
               line=12, name="s12/tpl/%d/name" % i)
    # tip + manifest + import
    panel(b, 92, 504, 220, 96, C["honey100"], r=12, stroke=C["honey300"],
          name="s12/tip")
    eyebrow(b, 108, 518, "TIP", color=C["honey700"], name="s12/tip/title")
    b.body(108, 536, TIP, 190, 9.0, C["ink800"], line=13, name="s12/tip/body")
    panel(b, 92, 614, 220, 86, C["paper"], r=12, stroke=C["hairline"],
          name="s12/manifest")
    eyebrow(b, 108, 628, "ASSET MANIFEST")
    b.meta(108, 648, "figcraft A/B/C · 20 tokens", width=190, size=8)
    b.meta(108, 664, "gate: PASS-with-notes", width=190, size=8,
           color=C["honey700"])
    b.meta(108, 682, "PDF p1-p9 · no invented tokens", width=190, size=8)
    button(b, 92, 720, 220, 34, "IMPORT PROJECT", C["honey300"], INK,
           name="s12/import-project")
    button(b, 92, 762, 220, 34, "OPEN WORKSPACE", C["paper"], C["ink700"],
           stroke=C["ink700"], name="s12/open-workspace")
    rule(b, 92, 818, 220, name="s12/rule-bottom")
    b.meta(92, 830, "HiveMind v1 · C shell", width=220, size=8)
    b.meta(92, 846, "rail 72 · library 260 · canvas honeycomb", width=220, size=8)
    b.meta(92, 862, "PDF p8:275-305 / p3:113-114", width=220, size=8)

    # centred HUD on the canvas (S09)
    panel(b, 696, 22, 380, 104, C["honey100"], r=16, stroke=C["honey400"],
          name="s09/hud", surface="S09")
    eyebrow(b, 716, 36, "SWARM THROUGHPUT")
    values = [("4", "SUPERAGENTS"), ("12", "WORKERS"), ("98.2%", "GREEN")]
    for i, (value, label) in enumerate(values):
        x = 716 + i * 112
        b.text(x, 54, value, 21, "Inter", "SemiBold", INK, width=104, line=24,
               name="s09/hud/%d/value" % i)
        b.meta(x, 80, label, width=104, size=7.5, name="s09/hud/%d/label" % i)
        if i:
            rule(b, x - 10, 54, 1, name="s09/hud/%d/rule" % i)
    chip(b, 1000, 46, 58, 20, "+3.4%", C["honey300"], INK, size=8.0,
         name="s09/hud/delta")
    b.meta(1000, 76, "source · live", width=70, size=7.5, name="s09/hud/source")

    # canvas chat core 336x330 (S02) + composer (S03)
    panel(b, 718, 286, 336, 330, C["paper"], r=18, stroke=C["hairline"],
          name="s02/chat-core", surface="S02", sw=1.0)
    b.bee(740, 306, 15, -90, name="chat/bee")
    b.name_t(760, 300, "Hive AI Assistant", 12.5, name="chat/title")
    b.meta(760, 318, "Pinned to hive centre", width=180, size=8, name="chat/pin")
    b.ellipse(1018, 302, 14, 14, fill=C["recess"], name="chat/close-bg")
    b.meta(1021, 305, "x", width=12, size=8, align="center", name="chat/close")
    b.rect(846, 340, 190, 40, fill=C["honey100"], r=12, name="chat/user-bubble")
    b.text(858, 350, CHAT[1], 9.0, "Inter", "Regular", C["ink800"], width=166,
           line=13, name="chat/user-text")
    eyebrow(b, 734, 396, "HIVE AI ASSISTANT")
    b.body(734, 414, CHAT[2], 300, 10.0, INK, line=14, name="chat/reply")
    for i, line in enumerate(CHAT[3]):
        b.rect(736, 442 + i * 15, 4, 4, fill=C["honey500"], r=2,
               name="chat/bullet/%d" % i)
        b.meta(746, 438 + i * 15, line, width=270, size=8.5,
               name="chat/bullet-text/%d" % i)
    rule(b, 734, 488, 304, name="chat/rule")
    b.meta(734, 498, "S02 · chat core 336x330 · PDF:175-184", width=300, size=8)
    b.rect(734, 520, 304, 42, fill=C["panel_alt"], r=10, stroke=C["hairline"],
           name="s03/composer", surface="S03")
    b.meta(746, 532, CHAT[4], width=240, size=9, name="composer/placeholder")
    button(b, 988, 525, 44, 32, "S03", C["honey300"], INK, name="composer/submit")
    b.meta(734, 574, "Enter submits · Shift+Enter newline · Ctrl+0 recentres",
           width=304, size=7.5, name="composer/hint")
    b.meta(734, 592, "PDF:235-237,296-302", width=304, size=7.5,
           name="composer/pdf")

    # 260x170 minimap (S13) bottom-right above the control bar
    panel(b, 1164, 630, 260, 170, C["panel_alt"], r=12, stroke=C["hairline"],
          name="s13/minimap", surface="S13")
    b.vector(window_path(1178, 642, 1412, 776, 22), 0, 0, stroke=C["honey500"],
             sw=1.0, opacity=0.20, name="s13/minimap-honey")
    for i, (mx, my) in enumerate([(1240, 690), (1300, 716), (1360, 682),
                                  (1320, 754)]):
        b.ellipse(mx - 5, my - 5, 10, 10,
                  fill=C["honey300"] if i % 2 == 0 else C["honey500"],
                  name="s13/minimap-node/%d" % i)
    b.rect(1222, 668, 120, 74, fill=None, stroke=C["honey500"], sw=1.5,
           name="s13/minimap-viewport")
    b.meta(1178, 770, "MINI-MAP 260x170", width=200, size=8)
    chip(b, 1332, 640, 78, 18, "client-only", C["paper"], META, size=7.0,
         stroke=C["hairline"], name="s13/minimap-chip")

    # bottom control bar (S04/S05/S06/S07) + zoom (S20)
    b.rect(332, 812, 1108, 48, fill=C["paper"], stroke=C["hairline"], sw=1.0,
           name="controls/bar")
    rule(b, 332, 812, 1108, name="controls/top-rule")
    b.state_dot(352, 830, "running", 9, name="controls/run-dot")
    b.meta(366, 826, "RUNNING", width=80, size=8.5, color=C["ink800"])
    b.meta(366, 840, "routing · 12 workers", width=120, size=7.5)
    button(b, 500, 824, 128, 30, "SPAWN SUB-HIVE", C["honey300"], INK,
           name="s04/spawn", surface="S04")
    button(b, 640, 824, 86, 30, "RE-ROUTE", C["paper"], C["ink700"],
           stroke=C["ink700"], name="s05/reroute", surface="S05")
    button(b, 738, 824, 76, 30, "CANCEL", C["paper"], C["signal"],
           stroke=C["signal"], name="s06/cancel", surface="S06")
    button(b, 826, 824, 104, 30, "KILL WORKER", C["signal"], C["paper"],
           name="s07/kill", surface="S07")
    b.meta(950, 830, "S04 / S05 / S06 / S07", width=180, size=7.5)
    button(b, 1140, 824, 30, 30, "-", C["paper"], C["ink700"],
           stroke=C["hairline"], name="s20/zoom-out")
    b.meta(1176, 832, "100%", width=44, size=8.5, align="center",
           name="s20/zoom-value")
    button(b, 1222, 824, 30, 30, "+", C["paper"], C["ink700"],
           stroke=C["hairline"], name="s20/zoom-in")
    button(b, 1266, 824, 96, 30, "CTRL+0", C["paper"], C["ink700"],
           stroke=C["hairline"], name="s20/recenter", surface="S20")
    b.meta(1378, 832, "VIEWPORT_SET", width=62, size=7.0, align="right",
           name="s20/wiring")

    # scrubber (S10)
    b.rect(332, 860, 1108, 40, fill=C["recess"], stroke=C["hairline"], sw=1.0,
           name="s10/scrubber", surface="S10")
    eyebrow(b, 352, 872, "TIME-TRAVEL", color=C["ink700"])
    b.text(452, 870, SCRUB[0], 12, "Consolas", "Regular", INK, width=96,
           line=15, name="s10/time")
    b.rect(600, 878, 520, 2, fill=C["ink500"], r=1, name="s10/track")
    for i in range(14):
        b.rect(600 + i * 40, 872, 1, 8, fill=C["hairline"],
               name="s10/tick/%d" % i)
    b.rect(600 + 12 * 40 - 4, 866, 8, 18, fill=C["honey500"], r=4,
           name="s10/thumb")
    chip(b, 1136, 868, 46, 18, "PLAY", C["paper"], INK, size=7.5,
         stroke=C["hairline"], name="s10/play")
    b.meta(1192, 872, SCRUB[1], width=200, size=8.5, name="s10/caption")
    b.meta(1256, 888, "HISTORY_REVISION_SET · PDF:246-247", width=160, size=7.0,
           align="right", name="s10/wiring")
    return b


# ------------------------------------------------------------------- C-hmas
def _card(b, x, y, w, h, sid, title, pfx, surface=None):
    panel(b, x, y, w, h, C["paper"], r=14, stroke=C["hairline"],
          name=pfx + "/card", surface=surface)
    eyebrow(b, x + 16, y + 12, title, name=pfx + "/title")
    return y + 18


def build_hmas(frame_id):
    b = Board("01", "HiveMind / HMAS", frame_id, 1440, 900, "c-hmas")
    b.rect(0, 0, 1440, 900, fill=C["canvas"], name="hmas/bg")
    b.vector(honeycomb_path(380, 70, 1500, 940, 36), 0, 0, stroke=C["honey500"],
             sw=1.0, opacity=0.11, name="hmas/background-lattice")

    # header
    b.rect(0, 0, 1440, 64, fill=C["paper"], stroke=C["hairline"], sw=1.0,
           name="hmas/header")
    brand(b, 24, 19, size=15.5, prefix="hmas-brand")
    for i, tab in enumerate(TABS):
        chip(b, 440 + i * 68, 22, 62, 22, tab.upper(),
             C["honey100"] if i == 0 else C["paper"],
             C["ink800"] if i == 0 else META, size=7.5,
             stroke=C["hairline"], name="hmas/tab/%d" % i)
    chip(b, 1188, 22, 118, 22, "MESH · TOPOLOGY", C["recess"], INK, size=7.5,
         name="hmas/mode")
    b.ellipse(1330, 16, 32, 32, fill=C["ink900"], name="hmas/account")
    b.meta(1325, 28, "HM", width=42, size=8, align="center", color=C["paper"],
           name="hmas/account-initials")

    # hierarchy panel (S22)
    panel(b, 24, 80, 340, 620, C["paper"], r=16, stroke=C["hairline"],
          name="s22/hierarchy-tree", surface="S22")
    eyebrow(b, 44, 98, "HIERARCHY")
    b.meta(44, 116, "workspace-jailed tree · role · kind · state", width=280,
           size=8)
    tree = [
        (44, "root", "HyperAgent", "healthy", "root · 4 superagents"),
        (66, "super", "Backend Architecture", "running", "3 workers · 12.4k ctx"),
        (90, "worker", "tailwind-builder", "running", "55%"),
        (90, "worker", "db-schema", "failed", "E0421 · retry 2/3"),
        (66, "super", "Frontend Architecture", "running", "2 workers · 8.1k ctx"),
        (90, "worker", "agent.spec.ts", "completed", "0.78"),
        (90, "worker", "main.tsx", "queued", "—"),
    ]
    for i, (x, kind, label, state, meta) in enumerate(tree):
        y = 146 + i * 42
        if i < len(tree) - 1:
            b.rect(x + 4, y + 12, 1, 30, fill=C["hairline"], name="tree-rule/%d" % i)
        glyph = "hex" if kind == "root" else ("hex" if kind == "super" else "square")
        if glyph == "hex":
            b.poly(x, y, 14, 14, C["honey200"], pointCount=6,
                   stroke=C["honey500"], name="tree/%d/glyph" % i)
        else:
            b.rect(x, y + 2, 12, 12, fill=C["recess"], r=2,
                   name="tree/%d/glyph" % i)
        state_row(b, x + 22, y + 2, state, label, meta,
                  name="tree/%d" % i, label_w=160)
    rule(b, 44, 456, 300, name="tree/rule")
    eyebrow(b, 44, 470, "SUPERAGENT CARD · S15")
    panel(b, 44, 488, 300, 76, C["honey100"], r=12, stroke=C["honey300"],
          name="s15/superagent-card", surface="S15")
    b.name_t(60, 498, "Backend Architecture", 11.5)
    b.meta(60, 514, "SUPERAGENT · decomposes intent, owns the sub-hive", width=270,
           size=7.5)
    b.meta(60, 530, "3 workers · 12.4k ctx · RUNNING", width=270, size=7.5,
           color=C["honey700"])
    rule(b, 44, 576, 300, name="tree/rule2")
    eyebrow(b, 44, 590, "WORKER CARD · S16")
    panel(b, 44, 608, 300, 76, C["paper"], r=12, stroke=C["hairline"],
          name="s16/worker-card", surface="S16")
    b.name_t(60, 618, "tailwind-builder", 11.5)
    b.meta(60, 634, "WORKER · react/vite · 55% · PDF:192-196", width=270, size=7.5)
    b.rect(60, 650, 240, 6, fill=C["recess"], r=3, name="worker/progress-bg")
    b.rect(60, 650, 132, 6, fill=C["honey500"], r=3, name="worker/progress")
    b.meta(60, 664, "state chips · S19", width=270, size=7.5)
    b.meta(44, 852, "HiveMind · C hmas · hierarchy 340px", width=300, size=8)
    b.meta(44, 868, "PDF:291-295 / p7:211-217", width=300, size=8)

    # topology canvas S01? keep background; nodes S15/S16/S19
    panel(b, 384, 80, 724, 620, C["panel_alt"], r=16, stroke=C["hairline"],
          name="hmas/topology-panel")
    b.vector(window_path(400, 96, 1092, 676, 30), 0, 0, stroke=C["honey500"],
             sw=1.0, opacity=0.18, name="hmas/topology-honey")
    eyebrow(b, 404, 98, "MESH / HIERARCHICAL TOPOLOGY")
    b.meta(404, 116, "HyperAgent → SuperAgents → WorkerAgents · bee edges",
           width=420, size=8)
    root = (746, 340)
    back = (548, 210)
    front = (944, 210)
    workers = {
        "tailwind-builder": (474, 452),
        "db-schema": (592, 512),
        "agent.spec.ts": (900, 452),
        "main.tsx": (1018, 512),
    }
    b.bee_edge(root, back, ctrl=(640, 250), label="decompose", name="edge/root-back")
    b.bee_edge(root, front, ctrl=(850, 250), label="decompose", name="edge/root-front")
    b.bee_edge(back, workers["tailwind-builder"], ctrl=(470, 330), label="stream",
               name="edge/back-tailwind")
    b.bee_edge(back, workers["db-schema"], ctrl=(530, 360), label="E0421",
               name="edge/back-db")
    b.bee_edge(front, workers["agent.spec.ts"], ctrl=(945, 330), label="stream",
               name="edge/front-spec")
    b.bee_edge(front, workers["main.tsx"], ctrl=(1010, 360), label="stream",
               name="edge/front-main")
    bee_node(b, root[0], root[1], 54, "HyperAgent", "healthy", "top/root")
    bee_node(b, back[0], back[1], 44, "Backend Architecture", "running",
             "top/back", surface="S15")
    bee_node(b, front[0], front[1], 44, "Frontend Architecture", "running",
             "top/front")
    bee_node(b, workers["tailwind-builder"][0], workers["tailwind-builder"][1], 34,
             "tailwind-builder", "running", "top/tailwind", surface="S16")
    bee_node(b, workers["db-schema"][0], workers["db-schema"][1], 34,
             "db-schema", "failed", "top/db")
    bee_node(b, workers["agent.spec.ts"][0], workers["agent.spec.ts"][1], 34,
             "agent.spec.ts", "completed", "top/spec")
    bee_node(b, workers["main.tsx"][0], workers["main.tsx"][1], 34,
             "main.tsx", "queued", "top/main")
    # state legend (S19)
    panel(b, 404, 588, 684, 96, C["paper"], r=12, stroke=C["hairline"],
          name="s19/state-chips", surface="S19")
    eyebrow(b, 420, 600, "STATE CHIPS · SHAPE + TEXT CUE (R-U4)")
    legend = ["healthy", "running", "queued", "awaiting_approval", "failed"]
    for i, state in enumerate(legend):
        x = 420 + i * 128
        b.state_dot(x, 630, state, 9, name="s19/%d/dot" % i)
        b.meta(x + 15, 626, state, width=104, size=8.5,
               name="s19/%d/label" % i)
    b.meta(420, 656, "roles: HyperAgent · SuperAgent · WorkerAgent · ToolAgent",
           width=640, size=8)
    b.meta(420, 672, "zoom controls + Ctrl+0 belong to the canvas (S20, C-shell)",
           width=640, size=8)

    # right column: profiles/import, memory, mailbox, projects, usage, approval
    x0, w0 = 1128, 288
    _card(b, x0, 80, w0, 104, "C-HMAS-PROFILES", "PROFILES · IMPORT", "profiles",
          surface="C-HMAS-PROFILES")
    state_row(b, x0 + 16, 112, "healthy", "Default profile", "active",
              name="profiles/default", label_w=150)
    state_row(b, x0 + 16, 128, "queued", "Strict review", "2 gates",
              name="profiles/strict", label_w=150)
    button(b, x0 + 16, 146, 256, 26, "IMPORT PROJECT", C["honey300"], INK,
           name="profiles/import")

    _card(b, x0, 192, w0, 104, "C-HMAS-MEMORY", "MEMORY", "memory",
          surface="C-HMAS-MEMORY")
    b.meta(x0 + 16, 214, "Run summary · 12.4k ctx", width=256, size=8.5,
           color=INK)
    b.meta(x0 + 16, 230, "Decision log · 3 rows", width=256, size=8.5,
           color=INK)
    b.meta(x0 + 16, 246, "Refusal ledger · 5 rows", width=256, size=8.5,
           color=INK)
    button(b, x0 + 16, 262, 256, 26, "CLEAR MEMORY", C["paper"], C["signal"],
           stroke=C["signal"], name="memory/clear")

    _card(b, x0, 304, w0, 104, "C-HMAS-MAILBOX", "MAILBOX · 2 UNREAD", "mailbox",
          surface="C-HMAS-MAILBOX")
    b.state_dot(x0 + 16, 332, "awaiting_approval", 8, name="mailbox/unread")
    b.meta(x0 + 32, 324, "Backend → Frontend", width=240, size=8.5, color=INK)
    b.meta(x0 + 32, 340, "schema.prisma v3 ready", width=240, size=8)
    b.meta(x0 + 32, 362, "Frontend → Backend · API drift report", width=240,
           size=8)
    b.meta(x0 + 32, 378, "2 unread · mailbox is read-only", width=240, size=7.5)

    _card(b, x0, 416, w0, 104, "C-HMAS-PROJECTS", "PROJECTS", "projects",
          surface="C-HMAS-PROJECTS")
    b.rect(x0 + 16, 438, 256, 26, fill=C["honey100"], r=8,
           name="projects/active-bg")
    b.meta(x0 + 26, 446, "HiveMind AI · active", width=200, size=8.5, color=INK)
    b.meta(x0 + 26, 476, "sandbox · idle", width=200, size=8.5)
    b.meta(x0 + 16, 498, "project_id · localhost only · PDF:256,275-282",
           width=256, size=7.5)

    _card(b, x0, 528, w0, 104, "C-HMAS-USAGE", "USAGE", "usage",
          surface="C-HMAS-USAGE")
    b.text(x0 + 16, 552, HUD[0], 12.5, "Inter", "SemiBold", INK, width=120,
           line=15, name="usage/supers")
    b.text(x0 + 148, 552, HUD[1], 12.5, "Inter", "SemiBold", INK, width=120,
           line=15, name="usage/workers")
    b.text(x0 + 16, 574, HUD[2], 12.5, "Inter", "SemiBold", C["verify"],
           width=120, line=15, name="usage/green")
    b.meta(x0 + 16, 594, "38 calls · 12.4k ctx · derived counters", width=256,
           size=8)
    b.rect(x0 + 16, 612, 256, 6, fill=C["recess"], r=3, name="usage/bar-bg")
    b.rect(x0 + 16, 612, 250, 6, fill=C["verify"], r=3, name="usage/bar")

    _card(b, x0, 640, w0, 104, "C-HMAS-APPROVAL", "MESSAGE APPROVAL", "approval",
          surface="C-HMAS-APPROVAL")
    chip(b, x0 + 170, 648, 100, 18, "APPROVAL", C["amber"], C["paper"], size=7.0,
         name="approval/pill")
    b.meta(x0 + 16, 674, "Backend Architecture → Frontend Architecture",
           width=256, size=7.5, color=INK)
    b.meta(x0 + 16, 690, "schema.prisma v3 ready · args_sha256 9f86d081",
           width=256, size=8)
    button(b, x0 + 16, 708, 120, 26, "DENY", C["paper"], C["signal"],
           stroke=C["signal"], name="approval/deny")
    button(b, x0 + 144, 708, 128, 26, "APPROVE", C["honey300"], INK,
           name="approval/approve", surface="C-HMAS-APPROVAL")
    b.meta(x0 + 16, 736, "APPROVAL_RESOLVE · nonce · single-use grant",
           width=256, size=7.0)

    # bottom status bar
    b.rect(0, 860, 1440, 40, fill=C["recess"], stroke=C["hairline"], sw=1.0,
           name="hmas/status-bar")
    b.meta(24, 872, "HiveMind mesh · 4 superagents · 12 workers · 38 calls",
           width=520, size=8.5)
    chip(b, 1050, 868, 120, 20, "awaiting_approval 1", C["honey100"], C["ink800"],
         size=7.5, stroke=C["honey300"], name="hmas/status-approval")
    chip(b, 1180, 868, 140, 20, "E_APPROVAL_TIMEOUT retryable", C["panel_alt"],
         C["ink700"], size=7.0, stroke=C["hairline"], name="hmas/status-refusal")
    return b


# ----------------------------------------------------------------- C-states
def _state_card(b, x, y, w, h, pfx, title, surface=None, pill=None, pill_fill=None,
                pill_color=None):
    panel(b, x, y, w, h, C["paper"], r=16, stroke=C["hairline"],
          name=pfx + "/card", surface=surface)
    eyebrow(b, x + 24, y + 18, title, name=pfx + "/title")
    if pill:
        chip(b, x + w - 24 - 190, y + 12, 190, 22, pill, pill_fill or C["honey100"],
             pill_color or C["ink800"], size=8.0,
             stroke=C["honey300"] if pill_fill is None else None,
             name=pfx + "/pill")
    return y + 16


def build_states(frame_id):
    b = Board("02", "HiveMind / states", frame_id, 1440, 900, "c-states")
    b.rect(0, 0, 1440, 900, fill=C["canvas"], name="states/bg")
    b.vector(honeycomb_path(0, 0, 1500, 960, 42), 0, 0, stroke=C["honey500"],
             sw=1.0, opacity=0.07, name="states/background-lattice")
    # header
    b.rect(0, 0, 1440, 64, fill=C["paper"], stroke=C["hairline"], sw=1.0,
           name="states/header")
    brand(b, 24, 18, size=15.0, prefix="states-brand")
    b.name_t(420, 21, "Named states", 17.0, name="states/title")
    b.meta(420, 42, "One vocabulary · named refusal + truth states", width=300,
           size=8.5)
    chip(b, 1200, 20, 210, 22, "16-state enum · contract", C["recess"], INK,
         size=8.0, name="states/enum-chip")
    y0, h0, gap = 96, 236, 20
    xs = (48, 744)
    w0 = 648

    # 1 — named refusal (S14)
    x, y = xs[0], y0
    _state_card(b, x, y, w0, h0, "refusal", "NAMED REFUSAL · E_APPROVAL_TIMEOUT",
                surface="S14", pill="ERROR · retryable", pill_fill=C["signal"],
                pill_color=C["paper"])
    b.rect(x + 24, y + 52, 270, 38, fill=C["recess"], r=8,
           name="refusal/code-bg")
    b.code_t(x + 36, y + 62, "E_APPROVAL_TIMEOUT", 12, color=INK,
             name="refusal/code")
    b.meta(x + 312, y + 62, "CODE", width=200, size=8)
    b.meta(x + 24, y + 108, "MESSAGE", width=560, size=8, color=C["signal"])
    b.body(x + 24, y + 124,
           "The approval request timed out before a decision arrived.", 596,
           10.5, INK, line=14, name="refusal/message")
    b.meta(x + 24, y + 156, "HOW_TO_FIX", width=560, size=8)
    b.body(x + 24, y + 172,
           "Resolve the request when prompted; the tool call is refused, never faked.",
           596, 10.0, C["ink700"], line=14, name="refusal/how-to-fix")
    b.meta(x + 24, y + 206, "ERROR frame → toast + log + ledger · PDF:269-273",
           width=596, size=7.5)

    # 2 — reconnect (C-STATE-RECONNECT)
    x = xs[1]
    _state_card(b, x, y, w0, h0, "reconnect", "RECONNECT · SOCKET CLOSED",
                surface="C-STATE-RECONNECT", pill="reconnecting",
                pill_fill=C["amber"], pill_color=C["paper"])
    for i, (cx, cy) in enumerate([(x + 110, y + 110), (x + 330, y + 110)]):
        b.poly(cx - 26, cy - 26, 52, 52, C["honey200"], pointCount=6,
               stroke=C["honey500"], name="reconnect/node/%d" % i)
        b.bee(cx, cy, 16, -90, name="reconnect/bee/%d" % i)
    for i in range(7):
        b.rect(x + 148 + i * 22, y + 108, 10, 3, fill=C["amber"],
               name="reconnect/dash/%d" % i)
    b.text(x + 380, y + 96, "0.5s", 12, "Consolas", "Regular", C["amber"],
           width=60, line=15, name="reconnect/countdown")
    b.body(x + 24, y + 160,
           "Reconnecting to the local server; mutations are blocked until the socket is open.",
           596, 10.5, INK, line=14, name="reconnect/message")
    chip(b, x + 24, y + 198, 174, 22, "mutations blocked", C["honey100"],
         C["ink800"], size=8.0, stroke=C["honey300"], name="reconnect/blocked")
    b.meta(x + 210, y + 202, "SERVER_HELLO · RESYNC_REQUIRED", width=380,
           size=7.5)

    # 3 — empty / loading (S26)
    x = xs[0]
    y = y0 + h0 + gap
    _state_card(b, x, y, w0, h0, "empty", "EMPTY / LOADING · S26",
                surface="S26", pill="empty · loading", pill_fill=C["recess"],
                pill_color=INK)
    panel(b, x + 24, y + 52, 286, 150, C["panel_alt"], r=12,
          stroke=C["hairline"], name="empty/state")
    b.meta(x + 44, y + 72, "EMPTY", width=240, size=8)
    b.body(x + 44, y + 92, "Nothing here yet", 240, 11.0, INK,
           name="empty/title")
    b.body(x + 44, y + 112,
           "No agents yet — submit a prompt to create the first run.", 240,
           9.0, C["ink700"], line=13, name="empty/body")
    panel(b, x + 330, y + 52, 294, 150, C["panel_alt"], r=12,
          stroke=C["hairline"], name="loading/state")
    b.ellipse(x + 348, y + 70, 30, 30, fill=None, stroke=C["honey500"], sw=3.0,
              name="loading/spinner")
    b.meta(x + 388, y + 78, "LOADING", width=200, size=8)
    b.rect(x + 348, y + 118, 250, 10, fill=C["recess"], r=5,
           name="loading/skeleton-1")
    b.rect(x + 348, y + 138, 190, 10, fill=C["recess"], r=5,
           name="loading/skeleton-2")
    b.rect(x + 348, y + 158, 220, 10, fill=C["recess"], r=5,
           name="loading/skeleton-3")
    b.meta(x + 24, y + 214, "S26 · skeleton bars + honest empty copy", width=596,
           size=7.5)

    # 4 — kill confirmation (C-STATE-KILL)
    x = xs[1]
    _state_card(b, x, y, w0, h0, "kill", "KILL CONFIRMATION", surface="C-STATE-KILL",
                pill="destructive", pill_fill=C["signal"], pill_color=C["paper"])
    b.rect(x + 24, y + 52, 596, 150, fill=C["recess"], r=12, name="kill/backdrop")
    panel(b, x + 66, y + 68, 512, 118, C["paper"], r=14, stroke=C["signal"],
          sw=1.5, name="kill/modal")
    b.name_t(x + 86, y + 82, "Kill worker?", 15.0)
    b.meta(x + 86, y + 104, "tailwind-builder · WORKER · running", width=470,
           size=8.5)
    b.body(x + 86, y + 122,
           "The worker cannot be killed from its current state.", 460, 9.5,
           C["ink800"], name="kill/message")
    b.meta(x + 86, y + 140,
           "Only running workers can be killed; refresh the worker state.",
           width=460, size=8.0)
    button(b, x + 86, y + 156, 110, 26, "CANCEL", C["paper"], C["ink700"],
           stroke=C["ink700"], size=8.5, name="kill/cancel")
    button(b, x + 210, y + 156, 140, 26, "KILL WORKER", C["signal"], C["paper"],
           size=8.5, name="kill/confirm")
    b.meta(x + 24, y + 214, "KILL_WORKER · confirmation first · never silent",
           width=596, size=7.5)

    # 5 — replay read-only (C-STATE-REPLAY)
    x = xs[0]
    y = y0 + 2 * (h0 + gap)
    _state_card(b, x, y, w0, h0, "replay", "REPLAY · READ-ONLY",
                surface="C-STATE-REPLAY", pill="Replay (read-only)",
                pill_fill=C["recess"], pill_color=C["ink700"])
    b.rect(x + 24, y + 52, 596, 32, fill=C["recess"], r=8, name="replay/banner")
    b.meta(x + 38, y + 62, "Replay (read-only) · banner + disabled controls",
           width=560, size=8.5, color=C["ink800"])
    b.rect(x + 24, y + 98, 596, 110, fill=C["panel_alt"], r=12,
           stroke=C["hairline"], name="replay/graph")
    for i, (cx, cy) in enumerate([(x + 110, y + 152), (x + 260, y + 128),
                                  (x + 260, y + 176), (x + 410, y + 152)]):
        b.rect(cx - 13, cy - 13, 26, 26, fill=C["recess"], r=6,
               name="replay/node/%d" % i)
        b.state_dot(cx + 8, cy - 18, "replay", 8, name="replay/node-state/%d" % i)
    for i, (x1, y1, x2, y2) in enumerate([(x + 123, y + 152, x + 247, y + 129),
                                          (x + 123, y + 152, x + 247, y + 175),
                                          (x + 273, y + 129, x + 397, y + 151),
                                          (x + 273, y + 175, x + 397, y + 153)]):
        b.line(x1, y1, math.hypot(x2 - x1, y2 - y1), C["ink500"], sw=1.0,
               rotation=math.degrees(math.atan2(y2 - y1, x2 - x1)),
               name="replay/edge/%d" % i)
    b.meta(x + 24, y + 214,
           "HISTORY_REVISION_SET → HISTORY_SNAPSHOT · no mutations · UI-COMMANDS §3",
           width=596, size=7.5)

    # 6 — DECISION-GATED diff (S25)
    x = xs[1]
    _state_card(b, x, y, w0, h0, "diff", "DIFF REVIEW · DECISION-GATED",
                surface="S25", pill="DECISION-GATED", pill_fill=C["amber"],
                pill_color=C["paper"])
    b.body(x + 24, y + 54,
           "Decision required: the code-diff surface is proposal-gated (no PDF "
           "artboard, no frozen DIFF command). It is not built in v1.",
           596, 10.0, INK, line=14, name="diff/explanation")
    b.rect(x + 24, y + 110, 596, 92, fill=C["panel_alt"], r=12,
           stroke=C["hairline"], name="diff/preview")
    b.meta(x + 38, y + 124, "router.ts", width=120, size=8.5)
    b.meta(x + 38, y + 142, "- const route = legacy();", width=240, size=8.5,
           color=C["signal"])
    b.meta(x + 38, y + 158, "+ const route = mesh();", width=240, size=8.5,
           color=C["verify"])
    b.meta(x + 300, y + 124, "files · 1 modified · +2 / -2", width=300, size=8)
    chip(b, x + 24, y + 210, 210, 22, "DIFF_* NOT FROZEN", C["recess"],
         C["ink800"], size=7.5, stroke=C["signal"], name="diff/not-frozen")
    b.meta(x + 250, y + 214, "DIFF_PROPOSE / APPLY / REJECT · U3 G-03",
           width=360, size=7.5)

    # footer
    b.rect(0, 860, 1440, 40, fill=C["recess"], stroke=C["hairline"], sw=1.0,
           name="states/footer")
    b.meta(24, 872, "HiveMind v1 · C states · 20 printed tokens · 1440x900",
           width=620, size=8.5)
    b.meta(1050, 872, "A/B unchanged · C new", width=366, size=8.0,
           align="right")
    return b


BUILDERS = {"00": build_shell, "01": build_hmas, "02": build_states}
