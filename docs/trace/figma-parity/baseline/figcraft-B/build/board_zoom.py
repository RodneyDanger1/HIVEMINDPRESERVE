"""Board 2 - "Canvas / zoom states": explicit 0.1x / 1.0x / 3.0x frames on one board."""
from boards_common import C, honeycomb_path, hex_subpath


def _panel(b, x, y, w, h, name):
    b.rect(x, y, w, h, fill=C["paper"], r=16, stroke=C["hairline"], sw=1,
           name=name, effect="e1")


def _mini_subhive(b, x, y, name):
    b.rect(x, y, 296, 132, fill=C["paper"], r=14, name=name + "-bg")
    b.rect(x, y, 296, 38, fill=C["honey100"], r=14, name=name + "-header")
    b.rect(x, y + 26, 296, 12, fill=C["honey100"], name=name + "-header2")
    b.name_t(x + 16, y + 10, "Backend Architecture", 14.5, name=name + "-name")
    b.pill(x + 200, y + 8, 84, 22, C["paper"], "RUNNING", C["ink800"], size=8.5,
           stroke=C["honey500"], sw=1, name=name + "-state")
    b.eyebrow(x + 16, y + 38, "SUPERAGENT", size=9, color=C["ink500"],
              name=name + "-role")
    b.body(x + 16, y + 54, "decomposes intent, owns the sub-hive", 264,
           size=10.5, line=15, name=name + "-body")
    b.meta(x + 16, y + 92, "3 workers  ·  12.4k ctx", size=9.5,
           name=name + "-meta")
    b.rect(x, y, 296, 132, fill=None, r=14, stroke=C["honey400"], sw=1.5,
           name=name + "-border")


def _mini_worker(b, x, y, name):
    b.rect(x, y, 220, 116, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           name=name + "-bg")
    b.name_t(x + 16, y + 14, "tailwind-builder", 14.5, name=name + "-name")
    b.text(x + 144, y + 16, "55%", 11, "Consolas", "Regular", C["ink700"],
           width=60, align="right", line=14, name=name + "-pct")
    b.eyebrow(x + 16, y + 34, "WORKER · react/vite", size=9.5,
              color=C["ink500"], name=name + "-role")
    b.rect(x + 16, y + 60, 188, 5, fill=C["recess"], r=999,
           name=name + "-track")
    b.rect(x + 16, y + 61, 103, 3, fill=C["honey500"], r=999,
           name=name + "-bar")
    b.meta(x + 16, y + 74, "progress 0-100 from real metric", size=8.5,
           name=name + "-metric")


def build_zoom(frame):
    from boards_common import Board
    b = Board("01", "Canvas / zoom states", frame, 1440, 900, "b-zoom-states")
    b.rect(0, 0, 1440, 900, fill=C["canvas"], name="zoom/bg")
    b.vector(honeycomb_path(0, 60, 1440, 900, r=44), 0, 0, stroke=C["honey500"],
             sw=1.0, opacity=0.10, name="zoom/lattice-bg")

    # ------------------------------------------------------------------ header
    b.eyebrow(48, 30, "S20 · CANVAS / ZOOM STATES · EXPLICIT 0.1x / 1.0x / 3.0x "
                      "· CLAMP 0.1-3.0 · PDF:304,183", name="zoom/eyebrow")
    b.text(48, 48, "Zoom states", 40, "Playfair Display", "SemiBold",
           C["ink900"], width=700, line=48, tracking=-1.5, name="zoom/title")
    b.body(48, 104, "The same composition at the three clamped viewport values. "
                    "@xyflow/react defaults (0.5 / 2.0) are overridden; anything "
                    "outside 0.1-3.0 is refused with E_ZOOM_RANGE.", 820,
           size=13, line=19, name="zoom/intro")
    b.eyebrow(1160, 30, "3 FRAMES · PDF p8:303-305", width=232,
              align="right", name="zoom/right-eyebrow")
    b.meta(1160, 48, "1440 x 900 · S20 · VIEWPORT_SET", width=232,
           align="right", size=8.5, name="zoom/right-note")

    # ------------------------------------------------------- frame 1: 0.1x
    x = 48
    b.text(x, 128, "0.1×", 34, "Playfair Display", "SemiBold", C["ink900"],
           width=120, line=40, name="zoom/p1/value")
    b.meta(x + 150, 140, "OVERVIEW · LOD 1 · labels only, no card interiors",
           width=262, size=8.5, name="zoom/p1/caption")
    _panel(b, x, 180, 416, 556, "zoom/p1/panel")
    b.vector(honeycomb_path(x + 8, 190, x + 408, 640, r=22), 0, 0,
             stroke=C["honey500"], sw=0.8, opacity=0.22, name="zoom/p1/lattice")
    # LOD 1: nodes collapse to shapes + labels
    b.rect(x + 36, 236, 120, 36, fill=C["honey100"], r=6, stroke=C["honey400"],
           sw=1, name="zoom/p1/subhive")
    b.text(x + 36, 280, "Backend Architecture", 9, "Inter", "SemiBold",
           C["ink900"], width=190, line=12, name="zoom/p1/subhive-label")
    b.vector(hex_subpath(x + 220, 300, 34), 0, 0, fill=C["honey300"],
             stroke=C["honey500"], sw=1, name="zoom/p1/core")
    b.text(x + 188, 340, "core hive", 9, "Inter", "SemiBold", C["ink800"],
           width=90, align="center", line=12, name="zoom/p1/core-label")
    b.rect(x + 56, 400, 80, 28, fill=C["paper"], r=6, stroke=C["hairline"],
           sw=1, name="zoom/p1/worker")
    b.text(x + 56, 434, "tailwind-builder", 9, "Inter", "SemiBold",
           C["ink800"], width=150, line=12, name="zoom/p1/worker-label")
    b.rect(x + 56, 470, 70, 24, fill=C["ink900"], r=5, name="zoom/p1/tool")
    b.text(x + 56, 500, "schema.prisma", 9, "Consolas", "Regular",
           C["ink800"], width=150, line=12, name="zoom/p1/tool-label")
    b.rect(x + 260, 430, 100, 40, fill=C["paper"], r=6, stroke=C["signal"],
           sw=1.2, name="zoom/p1/broken")
    b.text(x + 260, 446, "Compile failed", 9, "Inter", "SemiBold",
           C["signal"], width=120, line=12, name="zoom/p1/broken-label")
    b.vector("M %d 336 Q %d 350 %d 418" % (x + 200, x + 210, x + 120), 0, 0,
             stroke=C["honey600"], sw=1, name="zoom/p1/edge1")
    b.vector("M %d 418 Q %d 400 %d 392" % (x + 136, x + 150, x + 150), 0, 0,
             stroke=C["honey600"], sw=1, name="zoom/p1/edge2")
    b.vector("M %d 332 Q %d 360 %d 430" % (x + 252, x + 270, x + 300), 0, 0,
             stroke=C["signal"], sw=1, name="zoom/p1/edge3")
    b.cmd[-1]["params"]["props"]["dashes"] = [4, 3]
    b.pill(x + 16, 194, 82, 26, C["ink900"], "0.1x", C["paper"], size=10,
           name="zoom/p1/badge")
    b.meta(x + 24, 660, "LOD 1 / 3 · node cards collapse to labels; no meta, no "
                        "token detail", width=376, size=8.5,
           name="zoom/p1/note")
    b.meta(x + 24, 678, "viewport 10%: 144px of the 1440 board", width=376,
           size=8, name="zoom/p1/note2")
    b.meta(x + 24, 696, "still pans; Ctrl+0 returns to the core hive", width=376,
           size=8, name="zoom/p1/note3")

    # ------------------------------------------------------- frame 2: 1.0x
    x = 512
    b.text(x, 128, "1.0×", 34, "Playfair Display", "SemiBold", C["ink900"],
           width=120, line=40, name="zoom/p2/value")
    b.meta(x + 150, 140, "TRUE SIZE · full cards, no token/meta magnification",
           width=262, size=8.5, name="zoom/p2/caption")
    _panel(b, x, 180, 416, 556, "zoom/p2/panel")
    b.vector(honeycomb_path(x + 8, 190, x + 408, 700, r=30), 0, 0,
             stroke=C["honey500"], sw=0.9, opacity=0.16, name="zoom/p2/lattice")
    _mini_subhive(b, x + 60, 220, "zoom/p2/subhive")
    b.bee_edge((x + 208, 352), (x + 208, 420), ctrl=(x + 228, 386), bee_t=0.5,
               bee_size=18, name="zoom/p2/edge")
    _mini_worker(b, x + 80, 420, "zoom/p2/worker")
    b.pill(x + 16, 194, 82, 26, C["ink900"], "1.0x", C["paper"], size=10,
           name="zoom/p2/badge")
    b.meta(x + 24, 634, "100% · card anatomy readable: name 14.5 · role mono 9.5",
           width=376, size=8.5, name="zoom/p2/note")
    b.meta(x + 24, 650, "progress 5px track / 3px worker · radius 14 / 12 · padding 16",
           width=376, size=8, name="zoom/p2/note2")
    b.meta(x + 24, 666, "gap 8/16/24 only · bee edge source->target (R-U6)",
           width=376, size=8, name="zoom/p2/note3")

    # ------------------------------------------------------- frame 3: 3.0x
    x = 976
    b.text(x, 128, "3.0×", 34, "Playfair Display", "SemiBold", C["ink900"],
           width=120, line=40, name="zoom/p3/value")
    b.meta(x + 150, 140, "DETAIL · 300% crop: tokens + meta readable",
           width=262, size=8.5, name="zoom/p3/caption")
    _panel(b, x, 180, 416, 556, "zoom/p3/panel")
    b.vector(honeycomb_path(x + 8, 190, x + 408, 640, r=26), 0, 0,
             stroke=C["honey500"], sw=0.8, opacity=0.14, name="zoom/p3/lattice")
    # 3.0x crop of the WorkerAgent card (lower half: role, progress, meta)
    b.rect(x + 8, 220, 400, 356, fill=C["paper"], r=8, stroke=C["hairline"],
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
           C["ink900"], width=368, line=48, name="zoom/p3/name")
    b.pill(x + 16, 194, 82, 26, C["ink900"], "3.0x", C["paper"], size=10,
           name="zoom/p3/badge")
    b.meta(x + 24, 588, "crop rule: lower half of the WorkerAgent card at 300%",
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
           width=376, size=8, name="zoom/p3/note5")

    # ------------------------------------------------------- bottom strip
    b.rect(48, 756, 1344, 1, fill=C["hairline"], name="zoom/footer-rule")
    b.rect(48, 772, 200, 32, fill=C["paper"], r=8, stroke=C["hairline"], sw=1,
           name="s20/zoom-control", surface="S20")
    b.text(60, 778, "-", 16, "Inter", "SemiBold", C["ink700"], width=20,
           align="center", line=20, name="zoom/ctrl/minus")
    b.text(88, 782, "100%", 11, "Consolas", "Regular", C["ink900"], width=64,
           align="center", line=14, name="zoom/ctrl/value")
    b.text(160, 778, "+", 16, "Inter", "SemiBold", C["ink700"], width=20,
           align="center", line=20, name="zoom/ctrl/plus")
    b.meta(48, 810, "VIEWPORT_SET · zoom outside 0.1..3.0 -> E_ZOOM_RANGE",
           width=260, size=8.5, name="zoom/ctrl/command")
    b.meta(300, 764, "zoom clamp 0.1x-3.0x · @xyflow/react defaults 0.5 / 2.0 "
                     "OVERRIDDEN (R-U7)", width=560, size=10,
           name="zoom/clamp")
    b.meta(300, 786, "frames carry explicit values 0.1 / 1.0 / 3.0 - no implicit "
                     "scale is drawn", width=560, size=9, name="zoom/values")
    b.text(48, 830, "Ctrl+0 recentres the core hive", 12, "Inter", "SemiBold",
           C["ink700"], width=320, line=16, name="zoom/ctrl0")
    b.meta(48, 850, "VIEWPORT_RECENTER · PDF board prints Cmd+0; the Windows "
                    "build binds Ctrl+0 (R-U7)", width=420, size=8,
           name="zoom/ctrl0-note")
    b.meta(1160, 766, "3.0x frame is a real vector crop, not a bitmap",
           width=232, align="right", size=8, name="zoom/vector-note")
    b.meta(1160, 782, "S20 · Zoom controls + shortcut hint", width=232,
           align="right", size=8.5, name="zoom/surface-note")
    b.meta(48, 878, "CANVAS / ZOOM STATES · 1440x900 · frames labelled 0.1x / "
                    "1.0x / 3.0x · PDF p8:303-305", width=900, size=8,
           name="zoom/footer")
    return b
