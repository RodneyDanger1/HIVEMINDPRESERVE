"""Page 01 Brand — one mark, four lockups, clear space, bee anatomy, min sizes."""
from gen_common import Board, C


def bee_side(b, bx, by, group="01-anatomy", ink=C["ink900"]):
    """Side-view bee construction, head at +x (flies along the data direction)."""
    # wings behind the body
    b.ellipse(bx + 14, by + 2, 48, 18, fill=C["honey100"], stroke=C["ink700"],
              sw=1.2, rotation=-30, group=group, name="01/bee/wing-1")
    b.ellipse(bx + 58, by - 2, 44, 16, fill=C["paper"], stroke=C["ink700"],
              sw=1.2, rotation=24, group=group, name="01/bee/wing-2")
    # abdomen - thorax - head, left to right
    b.ellipse(bx + 2, by + 28, 58, 38, fill=C["honey300"], stroke=C["ink700"],
              sw=1.4, group=group, name="01/bee/abdomen")
    b.ellipse(bx + 52, by + 29, 34, 33, fill=C["honey500"], stroke=C["ink700"],
              sw=1.4, group=group, name="01/bee/thorax")
    b.ellipse(bx + 84, by + 32, 25, 25, fill=C["ink800"], group=group,
              name="01/bee/head")
    # abdomen bands (ink on honey)
    for i in range(2):
        b.rect(bx + 16 + i * 13, by + 34, 6, 26, fill=ink, r=3, group=group,
               name="01/bee/band-%d" % (i + 1))
    # antennae
    b.line(bx + 98, by + 30, 17, ink, 2, rotation=-55, group=group,
           name="01/bee/antenna-1")
    b.line(bx + 102, by + 34, 15, ink, 2, rotation=-20, group=group,
           name="01/bee/antenna-2")
    # stinger points back along -x: the bee is a routing arrow, it flies +x
    b.poly(bx - 14, by + 40, 15, 11, ink, pointCount=3, rotation=90, group=group,
           name="01/bee/stinger")
    # construction labels + leaders
    b.text(bx + 14, by - 30, "wings", 9, "Consolas", "Regular", C["ink700"],
           width=60, line=12, group=group)
    b.line(bx + 34, by - 2, 20, C["ink500"], 1, rotation=42, group=group,
           name="01/bee/leader-wing")
    b.text(bx + 108, by + 2, "antennae", 9, "Consolas", "Regular", C["ink700"],
           width=70, line=12, group=group)
    b.line(bx + 104, by + 18, 16, C["ink500"], 1, rotation=115, group=group,
           name="01/bee/leader-antenna")
    b.text(bx + 116, by + 44, "head", 9, "Consolas", "Regular", C["ink700"],
           width=50, line=12, group=group)
    b.line(bx + 110, by + 46, 6, C["ink500"], 1, group=group,
           name="01/bee/leader-head")
    b.text(bx + 58, by + 78, "thorax", 9, "Consolas", "Regular", C["ink700"],
           width=60, line=12, group=group)
    b.line(bx + 68, by + 64, 8, C["ink500"], 1, rotation=-80, group=group,
           name="01/bee/leader-thorax")
    b.text(bx - 78, by + 16, "abdomen", 9, "Consolas", "Regular", C["ink700"],
           width=70, line=12, group=group)
    b.line(bx - 32, by + 30, 40, C["ink500"], 1, rotation=6, group=group,
           name="01/bee/leader-abdomen")
    b.text(bx - 82, by + 70, "stinger", 9, "Consolas", "Regular", C["ink700"],
           width=60, line=12, group=group)
    b.line(bx - 26, by + 54, 56, C["ink500"], 1, rotation=-14, group=group,
           name="01/bee/leader-stinger")
    # direction rule
    b.text(bx - 82, by + 102, "BEE FLIES +x  ·  SOURCE → TARGET (±1°)", 8.5,
           "Consolas", "Regular", C["ink700"], width=320, line=12,
           tracking=8, group=group)


def lockup(b, x, y, kind):
    """Four lockups: primary, stacked, inverse, icon/mono."""
    if kind == "primary":
        b.rect(x, y, 420, 230, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
               group="01-lockups", surface="S23", name="01/lockup/primary/panel")
        b.text(x + 16, y + 14, "PRIMARY / ON PAPER", 9, "Consolas", "Regular",
               C["ink700"], width=260, line=12, tracking=8, group="01-lockups")
        b.text(x + 404, y + 14, "PDF p5:151-153", 8.5, "Consolas", "Regular",
               C["ink500"], width=120, line=12, align="right", group="01-lockups")
        b.hex_bee(x + 34, y + 68, 56, C["ink900"], C["honey300"], C["honey100"],
                  C["paper"], stripe=C["ink900"], thorax=C["honey500"],
                  group="01-lockups", name="01/lockup/primary/mark")
        b.text(x + 108, y + 66, "HiveRouter", 40, "Playfair Display", "SemiBold",
               C["ink900"], width=280, line=44, tracking=-1.5, group="01-lockups",
               name="01/lockup/primary/word")
        b.text(x + 110, y + 116, "SPATIAL EXECUTION ENVIRONMENT", 9, "Consolas",
               "Regular", C["ink700"], width=290, line=13, tracking=12,
               group="01-lockups")
        b.text(x + 16, y + 196, "Default lockup. Mark min 56px; wordmark min 20px.",
               8.5, "Consolas", "Regular", C["ink500"], width=388, line=12,
               group="01-lockups")
    elif kind == "stacked":
        b.rect(x, y, 420, 230, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
               group="01-lockups", name="01/lockup/stacked/panel")
        b.text(x + 16, y + 14, "STACKED / ON PAPER", 9, "Consolas", "Regular",
               C["ink700"], width=260, line=12, tracking=8, group="01-lockups")
        b.text(x + 404, y + 14, "PDF p5:154-156", 8.5, "Consolas", "Regular",
               C["ink500"], width=120, line=12, align="right", group="01-lockups")
        b.hex_bee(x + 178, y + 46, 64, C["ink900"], C["honey300"], C["honey100"],
                  C["paper"], stripe=C["ink900"], thorax=C["honey500"],
                  group="01-lockups", name="01/lockup/stacked/mark")
        b.text(x, y + 122, "HiveRouter", 34, "Playfair Display", "SemiBold",
               C["ink900"], width=420, line=38, tracking=-1, align="center",
               group="01-lockups", name="01/lockup/stacked/word")
        b.text(x, y + 162, "SPATIAL EXECUTION ENVIRONMENT", 8.5, "Consolas",
               "Regular", C["ink700"], width=420, line=12, tracking=12,
               align="center", group="01-lockups")
        b.text(x + 16, y + 196, "For narrow columns and splash panels.", 8.5,
               "Consolas", "Regular", C["ink500"], width=388, line=12,
               group="01-lockups")
    elif kind == "inverse":
        b.rect(x, y, 420, 230, fill=C["ink900"], r=12, group="01-lockups",
               name="01/lockup/inverse/panel")
        b.text(x + 16, y + 14, "INVERSE / ON INK", 9, "Consolas", "Regular",
               C["honey200"], width=260, line=12, tracking=8, group="01-lockups")
        b.text(x + 404, y + 14, "PDF p5:157-159", 8.5, "Consolas", "Regular",
               C["honey200"], width=120, line=12, align="right", group="01-lockups")
        b.hex_bee(x + 34, y + 68, 56, C["honey300"], C["ink900"], C["ink900"],
                  C["honey100"], stripe=C["honey300"], thorax=C["ink800"],
                  group="01-lockups", name="01/lockup/inverse/mark")
        b.text(x + 108, y + 66, "HiveRouter", 40, "Playfair Display", "SemiBold",
               C["paper"], width=280, line=44, tracking=-1.5, group="01-lockups",
               name="01/lockup/inverse/word")
        b.text(x + 110, y + 116, "SPATIAL EXECUTION ENVIRONMENT", 9, "Consolas",
               "Regular", C["honey200"], width=290, line=13, tracking=12,
               group="01-lockups")
        b.text(x + 16, y + 196, "On ink-900 only; never on honey-500 or darker honey.",
               8.5, "Consolas", "Regular", C["honey200"], width=388, line=12,
               group="01-lockups")
    else:  # icon / mono
        b.rect(x, y, 420, 230, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
               group="01-lockups", name="01/lockup/icon/panel")
        b.text(x + 16, y + 14, "ICON / APP MONO · ONE COLOUR", 9, "Consolas",
               "Regular", C["ink700"], width=300, line=12, tracking=8,
               group="01-lockups")
        b.text(x + 404, y + 14, "PDF p5:160-165", 8.5, "Consolas", "Regular",
               C["ink500"], width=120, line=12, align="right", group="01-lockups")
        b.hex_bee(x + 36, y + 60, 64, C["ink900"], C["paper"], C["paper"],
                  C["paper"], mono=True, group="01-lockups",
                  name="01/lockup/icon/mark")
        b.text(x + 118, y + 74, "HiveRouter", 20, "Playfair Display", "SemiBold",
               C["ink900"], width=200, line=24, tracking=-0.5, group="01-lockups",
               name="01/lockup/icon/word")
        b.text(x + 118, y + 100, "wordmark min 20px", 8.5, "Consolas", "Regular",
               C["ink500"], width=180, line=12, group="01-lockups")
        b.text(x + 16, y + 150, "One ink only: hexagon silhouette, bee knocked out.",
               8.5, "Consolas", "Regular", C["ink500"], width=388, line=12,
               group="01-lockups")
        b.text(x + 16, y + 166, "Mono 1-colour lockup = fallback for single-ink print.",
               8.5, "Consolas", "Regular", C["ink500"], width=388, line=12,
               group="01-lockups")
        b.text(x + 16, y + 196, "32px and 16px use the same silhouette; below 24px the inner bee is dropped.",
               8, "Consolas", "Regular", C["ink500"], width=388, line=12,
               group="01-lockups")


def build_brand(frame_id):
    b = Board("01", "Brand", frame_id, 1440, 900)
    b.rect(0, 0, 1440, 900, fill=C["canvas"], name="01/board-bg")

    # header ---------------------------------------------------------------
    b.eyebrow(48, 44, "04 / BRAND  ·  ONE MARK / FOUR LOCKUPS / CLEAR SPACE 0.25× MARK HEIGHT  ·  "
                      "BEE MIN 56 · 32 · 16  ·  PDF p5:148-169", group="01-header")
    b.text(48, 64, "One mark. Four lockups.", 56, "Playfair Display", "SemiBold",
           C["ink900"], width=1000, line=60, tracking=-2, name="01/title",
           group="01-header")
    b.body_text(48, 140,
                "The bee always flies toward the route direction — it is a routing arrow, "
                "not decoration. One clear-space rule governs every lockup.", 860,
                size=17, line=28, group="01-header")
    b.eyebrow(1092, 44, "HIVEROUTER  ·  BRAND  /  V1.0", color=C["ink700"],
              width=300, align="right", group="01-header")
    b.eyebrow(1092, 62, "FIGCRAFT NATIVE  ·  1440 × 900", color=C["ink500"],
              width=300, align="right", group="01-header")

    # four lockups ----------------------------------------------------------
    b.section(48, 196, 860, "lockups · primary / stacked / inverse / icon",
              note="PDF p5:151-166")
    lockup(b, 48, 220, "primary")
    lockup(b, 488, 220, "stacked")
    lockup(b, 48, 470, "inverse")
    lockup(b, 488, 470, "icon")

    # clear space -----------------------------------------------------------
    b.section(932, 196, 460, "clear space · 0.25 × mark height", note="PDF p5:160")
    b.rect(932, 220, 460, 220, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group="01-clearspace", surface="S23", name="01/clearspace/panel")
    # guide box 96 = 64 mark + 16 clear space each side
    gx, gy = 932 + 40, 220 + 54
    b.rect(gx, gy, 96, 96, fill=None, r=4, stroke=C["ink500"], sw=1, dash=[4, 4],
           group="01-clearspace", name="01/clearspace/guide")
    b.hex_bee(gx + 16, gy + 16, 64, C["ink900"], C["honey300"], C["honey100"],
              C["paper"], stripe=C["ink900"], thorax=C["honey500"],
              group="01-clearspace", name="01/clearspace/mark")
    b.text(gx, gy + 104, "h = 64  ·  clear space 0.25h = 16  ·  box 1.5h = 96", 8,
           "Consolas", "Regular", C["ink500"], width=200, line=12,
           group="01-clearspace")
    b.text(gx - 6, gy + 62, "16", 8, "Consolas", "Regular", C["signal"], width=30,
           line=12, align="right", group="01-clearspace")
    b.text(gx - 6, gy - 16, "16", 8, "Consolas", "Regular", C["signal"], width=30,
           line=12, align="right", group="01-clearspace")
    b.text(gx + 170, gy - 10, "Clear space = 0.25 × mark height on every side.",
           10.5, "Inter", "SemiBold", C["ink900"], width=250, line=14,
           group="01-clearspace")
    b.text(gx + 170, gy + 22, "No type, rule, edge or other mark may enter the clear "
                              "space. The guide box is 1.5h square around a 1h mark.",
           9, "Inter", "Regular", C["ink700"], width=250, line=14,
           group="01-clearspace")
    b.text(gx + 170, gy + 74, "MEASURE / mm-free", 8.5, "Consolas", "Regular",
           C["ink700"], width=250, line=12, tracking=8, group="01-clearspace")
    b.text(gx + 170, gy + 90, "0.25 × mark height, all four sides", 8.5,
           "Consolas", "Regular", C["ink500"], width=250, line=12,
           group="01-clearspace")

    # bee anatomy -----------------------------------------------------------
    b.section(932, 456, 460, "bee glyph · construction", note="PDF p5:162-169")
    b.rect(932, 480, 460, 250, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group="01-anatomy", name="01/anatomy/panel")
    bee_side(b, 1060, 516)
    b.text(948, 700, "The bee is the routing arrow: head leads, stinger trails, so the "
                     "glyph reads source → target.", 8.5, "Consolas", "Regular",
           C["ink500"], width=430, line=12, group="01-anatomy")

    # minimum sizes ---------------------------------------------------------
    b.section(48, 736, 1344, "minimum sizes · bee 56 / 32 / 16  ·  wordmark min 20",
              note="PDF p5:160-169")
    b.rect(48, 760, 1344, 100, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group="01-minsizes", name="01/minsizes/panel")
    b.hex_bee(72, 772, 56, C["ink900"], C["honey300"], C["honey100"], C["paper"],
              stripe=C["ink900"], thorax=C["honey500"], group="01-minsizes",
              name="01/minsizes/56")
    b.text(66, 836, "56 · app icon / hero", 8.5, "Consolas", "Regular", C["ink700"],
           width=150, line=12, group="01-minsizes")
    b.hex_bee(232, 784, 32, C["ink900"], C["honey300"], C["honey100"], C["paper"],
              stripe=C["ink900"], thorax=C["honey500"], group="01-minsizes",
              name="01/minsizes/32")
    b.text(212, 836, "32 · node chip", 8.5, "Consolas", "Regular", C["ink700"],
           width=120, line=12, group="01-minsizes")
    b.hex_bee(344, 792, 16, C["ink900"], None, None, None, bee=False,
              group="01-minsizes", name="01/minsizes/16")
    b.text(316, 836, "16 · favicon — inner bee dropped below 24px", 8.5, "Consolas",
           "Regular", C["ink700"], width=220, line=12, group="01-minsizes")
    b.text(556, 772, "Never below 16px.", 9, "Consolas", "Regular", C["ink700"],
           width=180, line=13, group="01-minsizes")
    b.text(556, 788, "Below 24px drop the inner bee and keep the hexagon silhouette only.",
           8.5, "Inter", "Regular", C["ink700"], width=180, line=12,
           group="01-minsizes")
    b.text(556, 828, "24–55px: inner bee may drop.", 8.5, "Consolas", "Regular",
           C["ink500"], width=180, line=12, group="01-minsizes")
    b.text(760, 776, "HiveRouter", 20, "Playfair Display", "SemiBold", C["ink900"],
           width=200, line=24, tracking=-0.5, group="01-minsizes",
           name="01/minsizes/wordmark")
    b.text(760, 804, "wordmark min 20px", 8.5, "Consolas", "Regular", C["ink500"],
           width=160, line=12, group="01-minsizes")
    b.text(760, 824, "≥56px: full construction.", 8.5, "Consolas", "Regular",
           C["ink500"], width=160, line=12, group="01-minsizes")
    b.text(932, 772, "MONO 1-COLOUR LOCKUP IS THE FALLBACK", 8.5, "Consolas",
           "Regular", C["ink700"], width=380, line=12, tracking=8,
           group="01-minsizes")
    b.text(932, 788, "for single-ink print and embroidery. The mark may rotate with the "
                     "edge angle (±1°); never mirror the bee.", 9, "Inter", "Regular",
           C["ink700"], width=440, line=14, group="01-minsizes")
    b.text(932, 828, "Clear space scales with h — 0.25 × h on every side.", 8.5,
           "Consolas", "Regular", C["ink500"], width=440, line=12,
           group="01-minsizes")

    # footer ----------------------------------------------------------------
    b.line(48, 872, 1344, C["hairline"], 1, name="01/footer-rule")
    b.text(48, 878, "HIVEROUTER  ·  BRAND  /  V1.0  ·  01 BRAND  ·  FIGCRAFT NATIVE",
           9, "Consolas", "Regular", C["ink500"], width=900, line=13, tracking=8,
           name="01/footer")
    b.text(1192, 878, "PAGE 2 / 4  ·  PDF p5", 9, "Consolas", "Regular",
           C["ink500"], width=200, line=13, align="right", name="01/footer-no")
    return b
