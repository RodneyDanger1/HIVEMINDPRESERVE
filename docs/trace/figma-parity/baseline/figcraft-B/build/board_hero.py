"""Board 4 - "Brand hero" (optional): cover-like statement + one canvas fragment."""
from boards_common import C, honeycomb_path, hex_subpath


def _lockup_card(b, x, y, w, h, name, dark=False):
    b.rect(x, y, w, h, fill=C["ink900"] if dark else C["paper"], r=12,
           stroke=None if dark else C["hairline"], sw=1, name=name,
           effect="e1")


def build_hero(frame):
    from boards_common import Board
    b = Board("03", "Brand hero", frame, 1440, 900, "b-hero")
    b.rect(0, 0, 1440, 900, fill=C["canvas"], name="hero/bg")
    b.vector(honeycomb_path(620, 60, 1440, 900, r=40), 0, 0,
             stroke=C["honey500"], sw=1.0, opacity=0.16,
             name="hero/lattice")

    # ------------------------------------------------------------- left column
    b.eyebrow(72, 62, "HIVEROUTER · DESIGN SYSTEM / V1.0 / FIGCRAFT NATIVE",
              name="hero/eyebrow")
    # primary brand lockup
    b.poly(72, 96, 56, 56, C["honey300"], pointCount=6, name="s23/logo-hex")
    b.bee(100, 124, 30, 0, name="s23/logo-bee", body=C["ink900"],
          wing=C["paper"], stripe=C["honey700"])
    b.text(144, 96, "HiveRouter", 56, "Playfair Display", "SemiBold",
           C["ink900"], width=470, line=60, tracking=-2, name="hero/wordmark")
    b.text(148, 164, "SPATIAL EXECUTION ENVIRONMENT", 11, "Consolas", "Regular",
           C["ink700"], width=470, line=15, tracking=12,
           name="hero/descriptor")
    b.rect(72, 200, 540, 1, fill=C["hairline"], name="hero/rule")

    b.text(72, 218, "Routing becomes a living honeycomb.", 40,
           "Playfair Display", "SemiBold", C["ink900"], width=540, line=48,
           tracking=-1.5, name="hero/statement")
    b.body(72, 320, "A spatial execution environment where code routing becomes "
                    "a living honeycomb topology: bee-line vectors, sub-hives "
                    "and worker nodes you can drag, re-parent and re-route by "
                    "hand.", 520, size=17, line=28, name="hero/tagline")
    chips = [("Honeycomb canvas", 72, 190), ("Bee-line routing", 272, 168),
             ("Hive chat core", 450, 160)]
    for label, cx, cw in chips:
        b.rect(cx, 442, cw, 32, fill=C["honey100"], r=999,
               stroke=C["honey400"], sw=1, name="hero/chip/%s" % label[:6])
        b.text(cx, 450, label, 11.5, "Inter", "SemiBold", C["ink800"],
               width=cw, align="center", line=15, name="hero/chip/%s/label"
               % label[:6])

    # ---------------------------------------------------------- brand lockups
    _lockup_card(b, 72, 500, 250, 132, "hero/lockup-primary")
    b.poly(88, 516, 40, 40, C["honey300"], pointCount=6,
           name="hero/lockup-primary/hex")
    b.bee(108, 536, 22, 0, name="hero/lockup-primary/bee", body=C["ink900"],
          wing=C["paper"], stripe=C["honey700"])
    b.text(136, 516, "HiveRouter", 20, "Playfair Display", "SemiBold",
           C["ink900"], width=170, line=24, tracking=-1,
           name="hero/lockup-primary/word")
    b.text(138, 540, "SPATIAL EXECUTION ENVIRONMENT", 7, "Consolas",
           "Regular", C["ink500"], width=170, line=10, tracking=8,
           name="hero/lockup-primary/mono")
    b.meta(88, 600, "PRIMARY / ON PAPER · wordmark min 20px", width=220,
           size=8, name="hero/lockup-primary/label")

    _lockup_card(b, 338, 500, 250, 132, "hero/lockup-inverse", dark=True)
    b.poly(354, 516, 40, 40, C["honey400"], pointCount=6,
           name="hero/lockup-inverse/hex")
    b.bee(374, 536, 22, 0, name="hero/lockup-inverse/bee", body=C["ink900"],
          wing=C["honey200"], stripe=C["honey700"])
    b.text(402, 516, "HiveRouter", 20, "Playfair Display", "SemiBold",
           C["paper"], width=170, line=24, tracking=-1,
           name="hero/lockup-inverse/word")
    b.text(404, 540, "SPATIAL EXECUTION ENVIRONMENT", 7, "Consolas",
           "Regular", C["honey200"], width=170, line=10, tracking=8,
           name="hero/lockup-inverse/mono")
    b.meta(354, 600, "INVERSE / ON INK · white on ink-800 12.64:1", width=220,
           size=8, color=C["honey200"],
           name="hero/lockup-inverse/label")

    # clear space + minimum sizes
    _lockup_card(b, 72, 648, 250, 124, "hero/clearspace")
    b.rect(102, 662, 48, 48, fill=None, r=2, stroke=C["ink500"], sw=1,
           dash=[3, 3], name="hero/clearspace/box")
    b.poly(110, 670, 32, 32, C["honey400"], pointCount=6,
           name="hero/clearspace/hex")
    b.meta(160, 664, "clear space", width=150, size=8,
           name="hero/clearspace/label")
    b.meta(160, 678, "= 0.25 x mark height", width=150, size=8,
           name="hero/clearspace/label2")
    b.meta(88, 726, "CLEAR SPACE RULE · never below 16px", width=220, size=8,
           name="hero/clearspace/note")
    b.meta(88, 742, "below 24px drop the inner bee", width=220, size=8,
           name="hero/clearspace/note2")

    _lockup_card(b, 338, 648, 250, 124, "hero/minsizes")
    b.poly(356, 694, 56, 56, C["honey300"], pointCount=6,
           name="hero/minsizes/56")
    b.bee(384, 722, 30, 0, name="hero/minsizes/56/bee", body=C["ink900"],
          wing=C["paper"], stripe=C["honey700"])
    b.meta(356, 758, "56px", width=56, align="center", size=8,
           name="hero/minsizes/56/label")
    b.poly(424, 702, 32, 32, C["honey400"], pointCount=6,
           name="hero/minsizes/32")
    b.bee(440, 718, 17, 0, name="hero/minsizes/32/bee", body=C["ink900"],
          wing=C["paper"], stripe=C["honey700"])
    b.meta(424, 758, "32px", width=32, align="center", size=8,
           name="hero/minsizes/32/label")
    b.poly(468, 710, 16, 16, C["honey500"], pointCount=6,
           name="hero/minsizes/16")
    b.meta(460, 758, "16px", width=32, align="center", size=8,
           name="hero/minsizes/16/label")
    b.meta(500, 694, "mono 1-colour", width=74, size=7.5,
           name="hero/minsizes/note")
    b.meta(500, 706, "fallback", width=74, size=7.5,
           name="hero/minsizes/note2")

    # ---------------------------------------------------------------- footer
    b.text(72, 800, "v1.0.0  ·  built with figcraft-cli  ·  zero GUI", 12.5,
           "Inter", "SemiBold", C["ink700"], width=540, line=17,
           name="hero/version")
    b.meta(72, 820, "figcraft-cli 0.11.2 · 597 commands discovered · PDF cover "
                    "prints 1,045 (UNRECONCILED - not restated as fact)", width=540,
           size=8.5, name="hero/evidence")
    b.meta(72, 838, "bee = routing arrow, flies source->target; never decoration "
                    "(R-U6 / PDF:149-150,227-229)", width=540, size=8,
           name="hero/bee-rule")
    b.meta(72, 854, "PDF p1:3-14 · cover statement + worker chip · 20 swatches / "
                    "8 type steps printed", width=540, size=8,
           name="hero/pdf-note")

    # ------------------------------------------------------- hero canvas fragment
    b.rect(664, 84, 704, 732, fill=C["paper"], r=16, stroke=C["hairline"],
           sw=1, name="hero/canvas-panel", effect="e2")
    b.vector(honeycomb_path(676, 96, 1356, 700, r=34), 0, 0,
             stroke=C["honey500"], sw=1.0, opacity=0.18,
             name="hero/canvas-lattice")
    b.eyebrow(688, 100, "HERO CANVAS FRAGMENT · CANVAS / DEFAULT @ 1.0x",
              size=8.5, name="hero/panel-label")
    b.meta(1268, 100, "1440 x 900 · 1.0x", width=84, align="right", size=8,
           name="hero/panel-scale")
    b.vector(hex_subpath(1016, 330, 250), 0, 0, fill=C["honey100"],
             stroke=C["honey300"], sw=1.5, name="hero/hive-hex", effect="e3")

    # chat core (pinned)
    b.rect(756, 190, 336, 300, fill=C["paper"], r=24, stroke=C["hairline"],
           sw=1, name="hero/chat-core", surface="S02", effect="e2")
    b.name_t(776, 202, "Hive AI Assistant", 14.5, name="hero/chat-title")
    b.eyebrow(776, 222, "HYPERAGENT", size=9, color=C["ink500"],
              name="hero/chat-role")
    b.text(1060, 200, "×", 14, "Inter", "SemiBold", C["ink500"], width=18,
           align="center", line=18, name="hero/chat-close")
    b.rect(756, 240, 336, 1, fill=C["hairline"], name="hero/chat-rule")
    b.rect(852, 256, 212, 42, fill=C["honey100"], r=12,
           name="hero/msg-user-bg")
    b.body(864, 264, "Generate a Python script for sentiment analysis.", 188,
           size=11, line=15, color=C["ink800"], name="hero/msg-user")
    b.body(776, 310, "Certainly! Here is the code...", 296, size=11, line=15,
           color=C["ink800"], name="hero/msg-reply")
    b.rect(776, 330, 296, 66, fill=C["recess"], r=8, name="hero/code-block")
    for i, ln in enumerate(("import libraries,", "Load dataset,",
                            "Preprocess data...")):
        b.text(786, 338 + i * 16, ln, 10, "Consolas", "Regular", C["ink900"],
               width=272, line=14, name="hero/code/%d" % i)
    b.rect(776, 412, 296, 40, fill=C["honey100"], r=10, stroke=C["honey300"],
           sw=2, name="hero/composer")
    b.text(788, 423, "Type your prompt...", 11.5, "Inter", "Regular",
           C["ink500"], width=240, line=15, name="hero/composer-ph")
    b.poly(1038, 419, 26, 26, C["ink900"], pointCount=6,
           name="hero/send-hex")
    b.vector("M 1046 432 L 1056 432 M 1052 428 L 1056 432 L 1052 436", 0, 0,
             stroke=C["paper"], sw=1.4, name="hero/send-arrow")

    # sub-hive + worker + bee edge (PDF cover values)
    b.rect(700, 556, 296, 132, fill=C["paper"], r=14,
           name="hero/subhive-bg", effect="e1")
    b.rect(700, 556, 296, 38, fill=C["honey100"], r=14,
           name="hero/subhive-header")
    b.rect(700, 582, 296, 12, fill=C["honey100"],
           name="hero/subhive-header2")
    b.name_t(716, 566, "Backend Architecture", 14.5,
             name="hero/subhive-name")
    b.pill(900, 564, 84, 22, C["paper"], "RUNNING", C["ink800"], size=8.5,
           stroke=C["honey500"], sw=1, name="hero/subhive-state")
    b.eyebrow(716, 592, "SUPERAGENT", size=9, color=C["ink500"],
              name="hero/subhive-role")
    b.body(716, 608, "decomposes intent, owns the sub-hive", 264, size=10.5,
           line=15, name="hero/subhive-body")
    b.meta(716, 646, "3 workers  ·  12.4k ctx", size=9.5,
           name="hero/subhive-meta")
    b.rect(700, 556, 296, 132, fill=None, r=14, stroke=C["honey400"], sw=1.5,
           name="hero/subhive-border")

    b.rect(1020, 556, 220, 116, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, name="hero/worker-card", effect="e1")
    b.name_t(1036, 570, "tailwind-builder", 14.5, name="hero/worker-name")
    b.text(1168, 572, "62%", 11, "Consolas", "Regular", C["ink700"], width=56,
           align="right", line=14, name="hero/worker-pct")
    b.eyebrow(1036, 590, "WORKER · react/vite", size=9.5, color=C["ink500"],
              name="hero/worker-role")
    b.rect(1036, 616, 188, 5, fill=C["recess"], r=999,
           name="hero/worker-track")
    b.rect(1036, 617, 117, 3, fill=C["honey500"], r=999,
           name="hero/worker-bar")
    b.meta(1036, 630, "running · 62% (PDF cover p1:12-14)", size=8,
           name="hero/worker-meta")

    b.bee_edge((924, 490), (866, 546), ctrl=(900, 528), bee_t=0.55,
               bee_size=20, name="hero/edge/core-subhive")
    b.meta(946, 508, "NODE_SPAWN", size=8, name="hero/edge/label")

    # canvas annotation strip
    b.meta(688, 766, "Honeycomb canvas · bee-line routing · hive chat core · "
                     "pinned core at the centre", width=440, size=8.5,
           name="hero/panel-note")
    b.meta(688, 782, "canvas fragment is drawn at 1.0x from the same vector "
                     "primitives as b-canvas-default", width=440, size=8,
           name="hero/panel-note2")
    b.meta(1150, 766, "S01 · S02 · S09 · S15 · S16", width=200, align="right",
           size=8, name="hero/panel-surfaces")
    return b
