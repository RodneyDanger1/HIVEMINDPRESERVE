"""Board 1 - "Canvas / default" 1440x900: full spatial canvas composition.

Every string/number is a PDF-printed value (evidence/pdf/pages.txt) or a frozen
UI-COMMANDS-v1 id. Derived counts are labelled derived; no value is invented.
"""
from boards_common import (C, HUD, SCRUB, SUB_HIVE, WORKER, TOOL, BROKEN, CHAT,
                           NAV, TABS, FILES, TEMPLATES, TIP, SHADOW_INK,
                           honeycomb_path, hex_subpath)


def build_default(frame):
    from boards_common import Board
    b = Board("00", "Canvas / default", frame, 1440, 900, "b-canvas-default")
    S = "S01"

    # ---------------------------------------------------------------- canvas
    b.rect(0, 0, 1440, 900, fill=C["canvas"], name="default/bg")
    b.vector(honeycomb_path(340, 88, 1440, 900, r=38), 0, 0,
             stroke=C["honey500"], sw=1.0, opacity=0.15,
             name="s01/honeycomb-lattice", surface=S)

    # -------------------------------------------------------------- nav rail
    b.rect(0, 0, 72, 900, fill=C["paper"], name="s11/nav-rail")
    b.rect(71, 0, 1, 900, fill=C["hairline"], name="s11/rail-hairline")
    # logo: brand bee in a hex chip
    b.poly(16, 18, 40, 40, C["honey300"], pointCount=6, name="s23/logo-hex")
    b.bee(36, 38, 20, 0, name="s23/logo-bee")
    b.meta(0, 64, "HIVE", width=72, align="center", size=8,
           name="s11/logo-word")
    items = [(116, True), (190, False), (264, False), (338, False),
             (412, False), (486, False)]
    for i, (y, active) in enumerate(items):
        if active:
            b.rect(8, y - 6, 56, 46, fill=C["honey100"], r=12,
                   name="s11/nav/%s/active" % NAV[i].lower())
        b.poly(25, y, 22, 22,
               C["honey400"] if active else C["recess"], pointCount=6,
               stroke=None if active else C["hairline"], sw=1,
               name="s11/nav/%s/icon" % NAV[i].lower())
        b.text(0, y + 26, NAV[i], 8, "Consolas", "Regular",
               C["ink900"] if active else C["ink500"], width=72, align="center",
               line=11, tracking=4, name="s11/nav/%s/label" % NAV[i].lower())
    # account (PDF p8:275 "AC  A. Chen")
    b.ellipse(18, 812, 36, 36, fill=C["ink900"], name="s28/avatar")
    b.text(18, 823, "AC", 11, "Inter", "SemiBold", C["paper"], width=36,
           align="center", line=14, name="s28/avatar-label")
    b.meta(0, 856, "A. Chen", width=72, align="center", size=8.5,
           name="s28/account-name")

    # --------------------------------------------------------- library panel
    b.rect(72, 0, 260, 900, fill=C["panel_alt"], name="s12/library-panel")
    b.rect(331, 0, 1, 900, fill=C["hairline"], name="s12/library-hairline")
    b.eyebrow(92, 20, "PROJECT", name="s28/project-eyebrow")
    b.name_t(92, 34, "HiveMind AI", 17, name="s28/project-name")
    b.meta(92, 58, "PROPOSED · v1 · project_id from state", size=8.5,
           name="s28/project-note")
    b.rect(92, 80, 220, 1, fill=C["hairline"], name="lib/rule-1")

    b.label(92, 94, "ACTIVE AGENTS", name="s12/active-label")
    b.poly(92, 120, 24, 24, C["honey300"], pointCount=6,
           name="s19/agent-chip-hex")
    b.bee(104, 132, 13, 0, name="s19/agent-chip-bee")
    b.text(124, 118, "Nova Vector Spark", 12.5, "Inter", "SemiBold",
           C["ink900"], width=150, line=16, name="s19/agent-chip-name")
    b.meta(124, 136, "Spark · healthy", size=8.5, name="s19/agent-chip-meta")
    b.dot(296, 124, 8, "circle", C["verify"], name="s19/agent-chip-dot")
    b.rect(92, 158, 220, 1, fill=C["hairline"], name="lib/rule-2")

    b.label(92, 172, "TEMPLATES", name="s12/templates-label")
    for i, t in enumerate(TEMPLATES):
        y = 192 + i * 28
        b.rect(92, y + 3, 15, 15, fill=C["recess"], r=4,
               name="s12/template/%d/icon" % i)
        b.text(116, y + 2, t, 12, "Inter", "Regular", C["ink800"], width=160,
               line=16, name="s12/template/%d/name" % i)
    b.meta(92, 306, "TEMPLATE_INSERT · licence-checked manifest", size=8.5,
           name="s12/template-note")
    b.rect(92, 326, 220, 1, fill=C["hairline"], name="lib/rule-3")

    b.label(92, 340, "TIP", name="s12/tip-label")
    b.body(92, 358, TIP, 220, size=11, line=15, name="s12/tip-body")
    b.rect(92, 416, 220, 1, fill=C["hairline"], name="lib/rule-4")

    b.eyebrow(92, 428, "STATE VOCABULARY", name="s19/state-label")
    b.meta(92, 444, "4 printed dots · shape + text cue · one >=13 enum",
           width=220, size=8.5, name="s19/state-note")
    chips = [("healthy", 458, 92), ("running", 458, 196),
             ("failed", 480, 92), ("queued", 480, 196)]
    for state, y, x in chips:
        b.state_dot(x, y, state, 8, name="s19/state/%s" % state,
                    surface="S19" if state == "healthy" else None)
        b.text(x + 13, y - 1, state, 9.5, "Consolas", "Regular", C["ink700"],
               width=88, line=13, tracking=2, name="s19/state/%s/label" % state)
    b.meta(92, 500, "colour is never the only channel (R-U4)", size=8,
           name="s19/state-rule")
    b.rect(92, 518, 220, 1, fill=C["hairline"], name="lib/rule-5")

    b.eyebrow(92, 532, "PROJECT FILES", name="s22/files-label")
    b.meta(92, 548, "workspace-jailed · S22", size=8.5, name="s22/files-note")
    for i, f in enumerate(FILES):
        y = 566 + i * 21
        b.rect(92, y + 3, 12, 12, fill=C["recess"], r=3,
               name="s22/file/%d/icon" % i)
        col = C["honey700"] if f == "schema.prisma" else C["ink700"]
        b.text(112, y, f, 10.5, "Consolas", "Regular", col, width=160,
               line=14, name="s22/file/%d/name" % i)
    b.rect(92, 682, 220, 1, fill=C["hairline"], name="lib/rule-6")

    b.eyebrow(92, 696, "SETTINGS", name="s21/settings-label")
    b.text(92, 714, "provider · deepseek", 9.5, "Consolas", "Regular",
           C["ink700"], width=200, line=13, name="s21/provider")
    b.text(92, 730, "key status · present", 9.5, "Consolas", "Regular",
           C["verify"], width=200, line=13, name="s21/key-status")
    b.text(92, 746, "no key -> E_KEY_MISSING (never fake a run)", 8.5,
           "Consolas", "Regular", C["signal"], width=210, line=12,
           name="s21/key-refusal")
    b.meta(92, 852, "BRIEF-B · canvas surfaces", width=220, size=8)
    b.meta(92, 868, "PDF p8:275-305 / p6-p7", width=220, size=8)

    # ------------------------------------------------------------ top chrome
    # tabs (S08): Workspace active
    b.rect(360, 20, 300, 32, fill=C["paper"], r=999, stroke=C["hairline"],
           sw=1, name="s08/tabs")
    b.rect(364, 24, 84, 24, fill=C["honey100"], r=999, name="s08/tab-active-bg")
    tab_x = [364, 456, 532, 608]
    tab_w = [84, 66, 66, 66]
    for i, t in enumerate(TABS):
        b.text(tab_x[i], 29, t, 11, "Inter", "SemiBold",
               C["ink900"] if i == 0 else C["ink500"], width=tab_w[i],
               align="center", line=15, name="s08/tab/%d" % i)
    b.rect(372, 44, 68, 2, fill=C["honey300"], r=0, name="s08/tab-active-rule")
    b.meta(360, 57, "S08 · tabs = view route state · CF-UI-04 OPEN",
           width=276, size=8, name="s08/tabs-note")

    # HUD (S09)
    b.rect(690, 20, 440, 40, fill=C["paper"], r=999, stroke=C["hairline"],
           sw=1, name="s09/hud", effect="e2")
    b.text(714, 31, HUD[0], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/superagents")
    b.text(826, 31, "·", 11, "Inter", "SemiBold", C["ink500"], width=12,
           align="center", line=15, name="s09/hud/sep1")
    b.text(842, 31, HUD[1], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/workers")
    b.text(924, 31, "·", 11, "Inter", "SemiBold", C["ink500"], width=12,
           align="center", line=15, name="s09/hud/sep2")
    b.dot(944, 34, 7, "circle", C["verify"], name="s09/hud/green-dot")
    b.text(956, 31, HUD[2], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/green")
    b.pill(1058, 26, 52, 28, C["honey200"], HUD[3], C["ink800"], size=10,
           name="s09/hud/delta")
    b.meta(690, 66, "RUN_METRICS · derived aggregate, never hardcoded (R-U13)",
           width=440, align="center", size=8, name="s09/hud/note")

    b.eyebrow(1160, 20, "S01 · INFINITE CANVAS", width=260, align="right",
              name="s01/right-eyebrow")
    b.meta(1160, 38, "honeycomb lattice + pan/zoom 0.1x-3.0x",
           width=260, align="right", size=8.5, name="s01/right-note")

    # ------------------------------------------------- hive containment (e3)
    b.vector(hex_subpath(910, 456, 262), 0, 0, fill=C["honey100"],
             stroke=C["honey300"], sw=1.5, name="e3/hive-containment",
             effect="e3")
    b.vector(hex_subpath(910, 456, 206), 0, 0, fill=None,
             stroke=C["honey300"], sw=1.0, name="e3/hive-containment-inner")

    # ------------------------------------------------------------ bee edges
    # core (parent) -> sub-hive (child): bee flies source->target
    b.bee_edge((742, 430), (668, 300), ctrl=(696, 356), bee_t=0.55,
               bee_size=22, name="edge/core-subhive", label="NODE_SPAWN")
    # sub-hive -> worker
    b.bee_edge((520, 342), (514, 436), ctrl=(534, 392), bee_t=0.50,
               bee_size=20, name="edge/subhive-worker", label="NODE_SPAWN")
    # worker -> tool: the tool card pops open beneath its parent (attached)

    # ------------------------------------------------------------- sub-hive
    b.rect(372, 210, 296, 132, fill=C["paper"], r=14,
           name="s15/subhive-card-bg", effect="e1")
    b.rect(372, 210, 296, 38, fill=C["honey100"], r=14,
           name="s15/subhive-header")
    b.rect(372, 236, 296, 12, fill=C["honey100"],
           name="s15/subhive-header-square")
    b.name_t(388, 220, SUB_HIVE[0], 14.5, name="s15/subhive-name",
             surface="S15")
    b.pill(572, 218, 84, 22, C["paper"], SUB_HIVE[1], C["ink800"], size=8.5,
           stroke=C["honey500"], sw=1, name="s15/subhive-state")
    b.dot(582, 226, 7, "circle", C["honey500"], name="s15/subhive-state-dot")
    b.eyebrow(388, 246, SUB_HIVE[2], size=9.5, color=C["ink500"],
              name="s15/subhive-role")
    b.body(388, 262, SUB_HIVE[3], 264, size=10.5, line=15,
           name="s15/subhive-body")
    b.meta(388, 300, "3 workers  ·  12.4k ctx", size=9.5, color=C["ink500"],
           name="s15/subhive-meta")
    b.rect(372, 210, 296, 132, fill=None, r=14, stroke=C["honey400"], sw=1.5,
           name="s15/subhive-border")
    b.meta(372, 350, "S15 · light honey header + honey-gold 1.5px border · radius 14",
           width=296, size=8, name="s15/subhive-note")

    # --------------------------------------------------------------- worker
    b.rect(404, 436, 220, 116, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, name="s16/worker-card", effect="e1")
    b.name_t(420, 450, WORKER[0], 14.5, name="s16/worker-name", surface="S16")
    b.text(548, 452, WORKER[3], 11, "Consolas", "Regular", C["ink700"],
           width=60, align="right", line=14, name="s16/worker-pct")
    b.eyebrow(420, 470, "%s · %s" % (WORKER[1], WORKER[2]), size=9.5,
              color=C["ink500"], name="s16/worker-role")
    b.rect(420, 496, 188, 5, fill=C["recess"], r=999, name="s16/worker-track")
    b.rect(420, 497, 103, 3, fill=C["honey500"], r=999,
           name="s16/worker-bar")
    b.meta(420, 510, "progress 0-100 from real metric", size=8.5,
           name="s16/worker-metric")


    # ----------------------------------------------------------- tool agent
    b.rect(404, 560, 260, 176, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, name="s17/tool-card", effect="e1")
    b.name_t(420, 574, TOOL[0], 13, name="s17/tool-name", surface="S17")
    b.pill(584, 574, 64, 20, C["recess"], TOOL[1], C["ink700"], size=8.5,
           name="s17/tool-role")
    b.rect(420, 602, 228, 96, fill=C["ink900"], r=8, name="s17/syntax-box")
    b.code_t(432, 610, "model Session {", width=210, size=10,
           name="s17/code-1")
    b.code_t(432, 628, "  id  String @id", width=210, size=10,
           name="s17/code-2")
    b.code_t(432, 646, "  run Run[]", width=210, size=10, name="s17/code-3")
    b.code_t(432, 664, "}", width=210, size=10, name="s17/code-4")
    b.rect(420, 694, 178, 3, fill=C["honey700"], r=0,
           name="s17/execution-bar")
    b.meta(420, 704, "schema.prisma  ·  0.78", size=9.5, color=C["ink500"],
           name="s17/tool-meta")
    b.meta(420, 720, "score only when measured; else —", width=228,
           size=8, name="s17/tool-note")

    # --------------------------------------------------------- broken route
    b.rect(1160, 330, 250, 150, fill=C["paper"], r=12, stroke=C["signal"],
           sw=1.5, name="s18/broken-card", effect="e1")
    b.poly(1176, 344, 16, 16, C["signal"], pointCount=3,
           name="s18/broken-warn")
    b.text(1178, 348, "!", 10, "Inter", "SemiBold", C["paper"], width=12,
           align="center", line=13, name="s18/broken-warn-mark")
    b.name_t(1200, 344, BROKEN[0], 14, color=C["signal"],
             name="s18/broken-name")
    b.eyebrow(1176, 368, "BROKEN ROUTE · ERROR STATE", size=8.5,
              color=C["signal"], name="s18/broken-role")
    b.meta(1176, 386, "db-schema  ·  E0421  ·  retry 2/3", size=9.5,
           name="s18/broken-meta")
    b.body(1176, 404, BROKEN[4], 218, size=10.5, line=15,
           name="s18/broken-body")
    b.meta(1176, 446, "how_to_fix: regenerate Prisma client", size=8.5,
           color=C["signal"], name="s18/broken-fix")
    b.meta(1160, 488, "S18 · attempt N/M sourced from the run ledger",
           width=250, size=8, name="s18/broken-note")
    # core -> broken route: dashed crimson wire, halted bee + warning glyph
    b.vector("M 1078 462 Q 1124 438 1160 406", 0, 0, stroke=C["signal"],
             sw=1.5, name="edge/core-broken/line")
    b.cmd[-1]["params"]["props"]["dashes"] = [6, 4]
    b.bee(1124, 434, 19, -32, name="edge/core-broken/bee", body=C["signal"],
          wing=C["honey200"], stripe=C["paper"])
    b.poly(1114, 408, 15, 15, C["signal"], pointCount=3,
           name="edge/core-broken/warn")
    b.text(1117, 412, "!", 9.5, "Inter", "SemiBold", C["paper"], width=9,
           align="center", line=12, name="edge/core-broken/warn-mark")
    b.meta(1080, 452, "halted bee", width=76, align="center",
           color=C["signal"], size=8, name="edge/core-broken/note")
    b.meta(1080, 464, "E0421", width=76, align="center",
           color=C["signal"], size=8, name="edge/core-broken/note2")

    # ------------------------------------------------- hive chat core (S02)
    b.rect(742, 286, 336, 330, fill=C["paper"], r=24, stroke=C["hairline"],
           sw=1, name="s02/chat-core", effect="e2", surface="S02")
    b.name_t(762, 298, CHAT[0], 14.5, name="s02/chat-title")
    b.eyebrow(762, 318, "HYPERAGENT", size=9, color=C["ink500"],
              name="s02/chat-role")
    b.text(1046, 296, "×", 14, "Inter", "SemiBold", C["ink500"], width=20,
           align="center", line=18, name="s02/chat-close")
    b.rect(742, 344, 336, 1, fill=C["hairline"], name="s02/chat-rule")
    b.rect(846, 360, 212, 46, fill=C["honey100"], r=12,
           name="s02/msg-user-bg")
    b.body(858, 368, CHAT[1], 188, size=11, line=15, color=C["ink800"],
           name="s02/msg-user")
    b.body(762, 416, CHAT[2], 296, size=11, line=15, color=C["ink800"],
           name="s02/msg-reply", surface="S27")
    b.rect(762, 436, 296, 86, fill=C["recess"], r=8,
           name="s02/code-block")
    for i, ln in enumerate(CHAT[3]):
        b.text(772, 444 + i * 16, ln, 10, "Consolas", "Regular",
               C["ink900"], width=272, line=14, name="s02/reply-code/%d" % i)
    b.rect(762, 536, 296, 42, fill=C["honey100"], r=10, stroke=C["honey300"],
           sw=2, name="s03/composer", surface="S03")
    b.text(774, 548, CHAT[4], 11.5, "Inter", "Regular", C["ink500"],
           width=240, line=15, name="s03/composer-placeholder")
    b.poly(1024, 544, 26, 26, C["ink900"], pointCount=6,
           name="s03/send-hex")
    b.vector("M 1032 557 L 1042 557 M 1038 553 L 1042 557 L 1038 561", 0, 0,
             stroke=C["paper"], sw=1.4, name="s03/send-arrow")
    b.meta(742, 584, "SUBMIT_PROMPT · empty text -> E_PROMPT_EMPTY", size=8,
           width=336, name="s03/composer-note")
    b.meta(742, 598, "composer focus: PDF 2px honey-300 ring 1.70:1 FAILS 3:1",
           size=8, color=C["signal"], width=336, name="s03/focus-note")
    b.meta(742, 612, "-> replacement ring ink-700 #4E3A15 9.86:1 (R-U3)",
           size=8, width=336, name="s03/focus-fix")
    b.meta(742, 630, "PINNED CORE · never pans/zooms · Ctrl+0 recentres",
           size=8.5, width=336, name="s02/pinned-note")

    # -------------------------------------------------- buttons bar (S04-S07)
    buttons = [("Spawn sub-hive", 360, 152, "primary"),
               ("Re-route", 524, 104, "secondary"),
               ("Cancel", 640, 88, "ghost"),
               ("Kill worker", 740, 118, "danger")]
    for label, x, w, kind in buttons:
        if kind == "primary":
            b.rect(x, 744, w, 40, fill=C["ink900"], r=10,
                   name="s04/button-bg")
            col = C["paper"]
        elif kind == "secondary":
            b.rect(x, 744, w, 40, fill=C["paper"], r=10, stroke=C["ink700"],
                   sw=1.5, name="s05/button-bg")
            col = C["ink900"]
        elif kind == "ghost":
            b.rect(x, 744, w, 40, fill=C["canvas"], r=10, stroke=C["hairline"],
                   sw=1, name="s06/button-bg")
            col = C["ink700"]
        else:
            b.rect(x, 744, w, 40, fill=C["paper"], r=10, stroke=C["signal"],
                   sw=1.5, name="s07/button-bg")
            col = C["signal"]
        b.text(x, 755, label, 12.5, "Inter", "SemiBold", col, width=w,
               align="center", line=16, name="button/%s" % kind)
    b.meta(360, 790, "S04 SPAWN_SUB_HIVE · S05 REWIRE_TASK · S06 CANCEL_RUN · "
                     "S07 KILL_WORKER  (frozen ids, UI-COMMANDS-v1 §2)",
           width=520, size=8, name="buttons/wiring")

    # ------------------------------------------------- Ctrl+0 + zoom (S20)
    b.text(360, 808, "Ctrl+0 recentres the core hive", 11, "Inter", "SemiBold",
           C["ink700"], width=330, line=15, name="s20/ctrl0")
    b.meta(360, 826, "zoom clamp 0.1x-3.0x · PDF board prints Cmd+0 · R-U7 "
                     "Windows override", width=330, size=8,
           name="s20/ctrl0-note")
    b.rect(1160, 668, 132, 28, fill=C["paper"], r=8, stroke=C["hairline"],
           sw=1, name="s20/zoom-control", surface="S20")
    b.text(1168, 672, "-", 14, "Inter", "SemiBold", C["ink700"], width=16,
           align="center", line=18, name="s20/zoom-out")
    b.text(1188, 675, "100%", 10.5, "Consolas", "Regular", C["ink900"],
           width=60, align="center", line=13, name="s20/zoom-value")
    b.text(1256, 672, "+", 14, "Inter", "SemiBold", C["ink700"], width=16,
           align="center", line=18, name="s20/zoom-in")
    b.meta(1160, 700, "VIEWPORT_SET · E_ZOOM_RANGE outside 0.1..3.0", width=260,
           size=8, name="s20/zoom-note")
    b.meta(1160, 640, "R-U6 · bee = routing arrow · source->target",
           width=260, size=8, name="s01/bee-rule")

    # ------------------------------------------------------ scrubber (S10)
    b.rect(700, 800, 380, 68, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, name="s10/scrubber", effect="e2", surface="S10")
    b.ellipse(714, 810, 24, 24, fill=C["honey300"], name="s10/play")
    b.vector("M 722 816 L 733 822 L 722 828 Z", 0, 0, fill=C["ink900"],
             name="s10/play-icon")
    b.text(748, 804, SCRUB[0], 11.5, "Consolas", "Regular", C["ink900"],
           width=110, line=14, name="s10/time")
    b.meta(900, 806, "· " + SCRUB[1], width=168, align="right", size=9,
           name="s10/state-label")
    b.rect(748, 838, 312, 2, fill=C["hairline"], name="s10/track")
    b.rect(748, 838, 98, 2, fill=C["honey500"], name="s10/progress")
    ticks = []
    for i in range(38):
        x = 748 + i * (312 / 37.0)
        ticks.append("M %.1f 832 L %.1f 840" % (x, x))
    b.vector(" ".join(ticks), 0, 0, stroke=C["ink500"], sw=0.75, opacity=0.55,
             name="s10/ticks")
    b.ellipse(840, 832, 12, 12, fill=C["paper"], stroke=C["honey500"], sw=2,
             name="s10/knob")
    b.meta(748, 850, "HISTORY_REVISION_SET · replay renders a real revision",
           width=312, size=8, name="s10/command-note")

    # ------------------------------------------------------- minimap (S13)
    b.rect(1164, 714, 260, 170, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, name="s13/minimap", effect="e1", surface="S13")
    b.eyebrow(1176, 724, "MINI-MAP · 260x170", size=8.5,
              name="s13/minimap-label")
    # miniature graph: 1440x900 -> 236x130 at 0.14, origin (1176,742)
    ox, oy, sc = 1176, 742, 0.14
    b.rect(ox + 372 * sc, oy + 210 * sc, 296 * sc, 132 * sc,
           fill=C["honey200"], r=2, name="s13/mini/subhive")
    b.rect(ox + 404 * sc, oy + 436 * sc, 220 * sc, 116 * sc,
           fill=C["recess"], r=2, name="s13/mini/worker")
    b.rect(ox + 404 * sc, oy + 560 * sc, 260 * sc, 170 * sc,
           fill=C["recess"], r=2, name="s13/mini/tool")
    b.rect(ox + 742 * sc, oy + 286 * sc, 336 * sc, 330 * sc,
           fill=C["honey300"], r=3, name="s13/mini/core")
    b.rect(ox + 1160 * sc, oy + 330 * sc, 250 * sc, 150 * sc,
           fill=None, r=2, stroke=C["signal"], sw=1,
           name="s13/mini/broken")
    b.rect(1196, 760, 150, 96, fill=None, stroke=C["ink700"], sw=1.5,
           name="s13/viewport-rect")
    b.meta(1176, 856, "viewport rect · client-only", size=8,
           name="s13/minimap-note")

    # --------------------------------------------------------- board footer
    b.meta(360, 872, "CANVAS / DEFAULT · 1440x900 · PDF p8:275-305 · "
                     "S01 S02 S03 S04 S05 S06 S07 S08 S09 S10 S11 S12 S13 "
                     "S15 S16 S17 S18 S19 S20", width=640, size=8,
           name="board/footer")
    return b
