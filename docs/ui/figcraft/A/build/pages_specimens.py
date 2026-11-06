"""Page 03 Specimens — selected / hover / focus / disabled / empty / loading /
offline / error / awaiting-approval / streaming / reduced-motion + enum legend."""
from gen_common import Board, C, STATES

W = 1440
H = 1400
COL_X = [60, 508, 956]
ROW_Y = [236, 514, 792, 1070]
TW, TH = 424, 256


def tile(b, col, row, title, tag):
    x, y = COL_X[col], ROW_Y[row]
    g = "spec-%s" % title.lower().replace(" ", "-").replace("/", "-")
    b.rect(x, y, TW, TH, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g, name="03/%s/panel" % g)
    b.text(x + 16, y + 13, title.upper(), 11, "Inter", "SemiBold", C["ink900"],
           width=280, line=15, tracking=4, group=g)
    b.text(x + 408, y + 14, tag, 8, "Consolas", "Regular", C["ink500"],
           width=190, line=12, align="right", group=g)
    b.line(x + 16, y + 34, 392, C["hairline"], 1, group=g,
           name="03/%s/rule" % g)
    return x, y, g


def note(b, x, y, g, lines):
    for i, s in enumerate(lines):
        b.text(x + 16, y + 200 + i * 13, s, 8, "Consolas", "Regular", C["ink500"],
               width=392, line=11, group=g)


def tile_selected(b):
    x, y, g = tile(b, 0, 0, "Selected", "canvas selection")
    # mini super card
    b.rect(x + 44, y + 60, 200, 104, fill=C["paper"], r=12, stroke=C["honey500"],
           sw=1.5, group=g, name="03/sel/card")
    b.rect(x + 44, y + 60, 200, 30, fill=C["honey100"], r=[12, 12, 0, 0],
           group=g, name="03/sel/header")
    b.text(x + 56, y + 67, "Backend Architecture", 9.5, "Inter", "SemiBold",
           C["ink900"], width=170, line=13, group=g)
    b.text(x + 56, y + 100, "SUPERAGENT", 7.5, "Consolas", "Regular", C["ink500"],
           width=120, line=10, tracking=8, group=g)
    b.rect(x + 56, y + 118, 160, 4, fill=C["recess"], r=999, group=g,
           name="03/sel/track")
    b.rect(x + 56, y + 119, 110, 2, fill=C["honey600"], r=999, group=g,
           name="03/sel/bar")
    # selection outline + corner handles
    b.rect(x + 38, y + 54, 212, 116, fill=None, r=14, stroke=C["honey500"],
           sw=2, group=g, name="03/sel/outline")
    for hx, hy in ((x + 34, y + 50), (x + 242, y + 50), (x + 34, y + 162),
                   (x + 242, y + 162)):
        b.rect(hx, hy, 8, 8, fill=C["paper"], r=2, stroke=C["honey700"], sw=1.5,
               group=g, name="03/sel/handle")
    b.text(x + 268, y + 84, "selected", 9.5, "Consolas", "Regular", C["ink700"],
           width=140, line=13, group=g)
    b.text(x + 268, y + 100, "honey-500 outline", 8, "Consolas", "Regular",
           C["ink500"], width=140, line=11, group=g)
    b.text(x + 268, y + 112, "+ 8px handles", 8, "Consolas", "Regular",
           C["ink500"], width=140, line=11, group=g)
    note(b, x, y, g, ["selected = canvas selection layer, not an agent state;",
                      "keyboard path: tab to node, Space selects, arrows move"])


def tile_hover(b):
    x, y, g = tile(b, 1, 0, "Hover", "pointer · e1")
    b.rect(x + 44, y + 56, 200, 104, fill=C["paper"], r=12, stroke=C["honey500"],
           sw=1.5, group=g, name="03/hover/card")
    b.rect(x + 44, y + 56, 200, 26, fill=C["honey200"], r=[12, 12, 0, 0],
           group=g, name="03/hover/strip")
    b.text(x + 56, y + 62, "tailwind-builder", 9.5, "Inter", "SemiBold",
           C["ink900"], width=170, line=13, group=g)
    b.text(x + 56, y + 94, "WORKER · react/vite", 7.5, "Consolas", "Regular",
           C["ink500"], width=170, line=10, tracking=8, group=g)
    b.rect(x + 56, y + 112, 176, 5, fill=C["recess"], r=999, group=g,
           name="03/hover/track")
    b.rect(x + 56, y + 113, 96, 3, fill=C["honey600"], r=999, group=g,
           name="03/hover/bar")
    # cursor
    b.poly(x + 254, y + 122, 16, 20, C["ink900"], pointCount=3, rotation=200,
           group=g, name="03/hover/cursor")
    b.text(x + 266, y + 132, "hover", 8, "Consolas", "Regular", C["ink700"],
           width=80, line=11, group=g)
    note(b, x, y, g, ["hover raises the card to e1 and warms the header;",
                      "120ms ease · motion off under reduced-motion"])


def tile_focus(b):
    x, y, g = tile(b, 2, 0, "Focus", "ink-700 ring")
    b.rect(x + 44, y + 64, 176, 44, fill=C["paper"], r=10, stroke=C["ink700"],
           sw=1.5, group=g, name="03/focus/button")
    b.text(x + 44, y + 77, "Spawn sub-hive", 12, "Inter", "SemiBold", C["ink900"],
           width=176, line=16, align="center", group=g)
    b.rect(x + 40, y + 60, 184, 52, fill=None, r=12, stroke=C["ink700"], sw=2,
           group=g, name="03/focus/ring")
    b.rect(x + 276, y + 68, 20, 20, fill=C["ink700"], r=4, group=g,
           name="03/focus/swatch")
    b.text(x + 304, y + 70, "#4E3A15 · 9.86:1", 9, "Consolas", "Regular",
           C["ink900"], width=110, line=13, group=g)
    b.text(x + 276, y + 94, "focus ring ≥3:1", 8, "Consolas", "Regular",
           C["ink700"], width=120, line=11, group=g)
    b.text(x + 276, y + 106, "replaces honey-300", 8, "Consolas", "Regular",
           C["ink500"], width=120, line=11, group=g)
    b.text(x + 276, y + 118, "(1.70 / 1.55 fails)", 8, "Consolas", "Regular",
           C["signal"], width=120, line=11, group=g)
    note(b, x, y, g, ["ring: 2px ink-700, 2px offset, on every interactive control;",
                      "no control's only boundary is hairline 1.19:1"])


def tile_disabled(b):
    x, y, g = tile(b, 0, 1, "Disabled", "pending · in-flight")
    b.rect(x + 44, y + 56, 176, 40, fill=C["recess"], r=10, stroke=C["hairline"],
           sw=1, group=g, name="03/dis/button")
    b.text(x + 44, y + 68, "Spawn sub-hive", 11.5, "Inter", "SemiBold",
           C["ink500"], width=176, line=15, align="center", group=g)
    b.rect(x + 44, y + 108, 280, 40, fill=C["canvas"], r=10, stroke=C["hairline"],
           sw=1, group=g, name="03/dis/composer")
    b.text(x + 56, y + 120, "Type your prompt…", 10, "Consolas", "Regular",
           C["ink500"], width=240, line=14, group=g)
    b.text(x + 44, y + 160, "disabled = pending/in-flight · never a fake success",
           8.5, "Consolas", "Regular", C["ink700"], width=360, line=12, group=g)
    b.text(x + 44, y + 174, "duplicate submit → one run · ACK or refusal", 8,
           "Consolas", "Regular", C["ink500"], width=360, line=11, group=g)
    note(b, x, y, g, ["disabled is not an agent state — it is a control state;",
                      "the same run keeps its correlation_id until ACK"])


def tile_empty(b):
    x, y, g = tile(b, 1, 1, "Empty", "S26 · empty canvas")
    # honeycomb canvas (S01 background, vector)
    b.rect(x + 200, y + 46, 200, 140, fill=C["canvas"], r=10, group=g,
           name="03/empty/canvas", surface="S01")
    hx0, hy0 = x + 212, y + 58
    for r in range(3):
        for c in range(3):
            hx = hx0 + c * 40 + (20 if r % 2 else 0)
            hy = hy0 + r * 38
            if hx + 30 > x + 392:
                continue
            b.poly(hx, hy, 30, 30, C["honey200"], pointCount=6, stroke=C["hairline"],
                   sw=1, group=g, name="03/empty/honeycomb")
    b.rect(x + 200, y + 46, 200, 140, fill=None, r=10, stroke=C["hairline"],
           sw=1, group=g, name="03/empty/frame", surface="S26")
    b.text(x + 44, y + 66, "No sub-hives yet", 13, "Inter", "SemiBold",
           C["ink900"], width=150, line=17, group=g)
    b.text(x + 44, y + 88, "Drop a repository into the core hive and the HyperAgent "
                           "maps its architecture before a single worker spawns.",
           8.5, "Inter", "Regular", C["ink700"], width=140, line=12, group=g)
    b.rect(x + 44, y + 146, 132, 32, fill=C["ink900"], r=10, group=g,
           name="03/empty/cta")
    b.text(x + 44, y + 154, "Spawn sub-hive", 9.5, "Inter", "SemiBold", C["paper"],
           width=132, line=13, align="center", group=g)
    b.text(x + 200, y + 190, "honeycomb grid is vector (SVG) · pan X/Y free",
           7.5, "Consolas", "Regular", C["ink500"], width=200, line=10, group=g)
    note(b, x, y, g, ["empty is a surface state, not an agent state;",
                      "empty canvas, empty library and no-history are distinct copy"])


def tile_loading(b):
    x, y, g = tile(b, 2, 1, "Loading", "thinking · first token")
    for i, wdt in enumerate((260, 200, 300)):
        b.rect(x + 44, y + 56 + i * 26, wdt, 14, fill=C["recess"], r=4,
               group=g, name="03/load/skeleton")
    b.ellipse(x + 44, y + 146, 14, 14, fill="none", stroke=C["honey500"], sw=2,
              group=g, name="03/load/spinner")
    b.text(x + 66, y + 144, "thinking…", 9.5, "Consolas", "Regular", C["ink700"],
           width=120, line=13, group=g)
    b.text(x + 44, y + 172, "skeleton is static under reduced motion; state still "
                            "updates", 8, "Consolas", "Regular", C["ink500"],
           width=360, line=11, group=g)
    note(b, x, y, g, ["no spinner without a timeout path: 30s → E_PROVIDER_TIMEOUT;",
                      "streaming ends the loading state on the first REPLY_DELTA"])


def tile_offline(b):
    x, y, g = tile(b, 0, 2, "Offline / Reconnecting", "E_WS_DISCONNECTED")
    b.rect(x + 44, y + 56, 340, 44, fill=C["recess"], r=10, group=g,
           name="03/offline/banner")
    b.dot(x + 58, y + 72, 10, "diamond", C["amber"], group=g,
          name="03/offline/dot")
    b.text(x + 76, y + 62, "Reconnecting… socket dropped", 10.5, "Inter",
           "SemiBold", C["ink900"], width=280, line=14, group=g)
    b.text(x + 76, y + 80, "retrying with backoff", 8, "Consolas", "Regular",
           C["ink500"], width=220, line=11, group=g)
    b.text(x + 44, y + 112, "hello{last_seq} → snapshot-or-diff", 9, "Consolas",
           "Regular", C["ink700"], width=360, line=13, group=g)
    b.text(x + 44, y + 128, "stale mutation → E_RESYNC_REQUIRED", 9, "Consolas",
           "Regular", C["signal"], width=360, line=13, group=g)
    note(b, x, y, g, ["reconnecting is an agent state; it resolves to healthy only",
                      "after the resnapshot — never optimistically"])


def tile_error(b):
    x, y, g = tile(b, 1, 2, "Error", "E0421 · retry 2/3")
    b.rect(x + 44, y + 54, 250, 128, fill=C["paper"], r=12, stroke=C["signal"],
           sw=1.5, group=g, name="03/error/card")
    b.text(x + 58, y + 62, "BROKEN ROUTE", 7.5, "Consolas", "Regular",
           C["signal"], width=150, line=10, tracking=8, group=g)
    b.text(x + 58, y + 76, "Compile failed", 12.5, "Inter", "SemiBold",
           C["ink900"], width=220, line=16, group=g)
    b.text(x + 58, y + 96, "db-schema · E0421 · retry 2/3", 8.5, "Consolas",
           "Regular", C["signal"], width=220, line=12, group=g)
    b.text(x + 58, y + 112, "Prisma client type mismatch", 8.5, "Inter", "Regular",
           C["ink700"], width=220, line=12, group=g)
    b.line(x + 58, y + 140, 120, C["signal"], 1.5, dash=[6, 4], group=g,
           name="03/error/wire")
    b.text(x + 186, y + 132, "dashed crimson wire", 7.5, "Consolas", "Regular",
           C["ink500"], width=140, line=10, group=g)
    b.text(x + 312, y + 76, "E-code + attempt", 8, "Consolas", "Regular",
           C["ink700"], width=100, line=11, group=g)
    b.text(x + 312, y + 88, "from run ledger", 8, "Consolas", "Regular",
           C["ink500"], width=100, line=11, group=g)
    note(b, x, y, g, ["error card = one enum state + a coded refusal;",
                      "retry is bounded 3/3 — no retry past the ledger counter"])


def tile_approval(b):
    x, y, g = tile(b, 2, 2, "Awaiting approval", "S25 · diff modal")
    b.rect(x + 44, y + 52, 336, 136, fill=C["paper"], r=12, stroke=C["ink700"],
           sw=1.5, group=g, name="03/appr/modal", surface="S25")
    b.text(x + 58, y + 60, "CODE-DIFF REVIEW · APPROVAL_REQUIRED", 7.5,
           "Consolas", "Regular", C["ink700"], width=300, line=10, tracking=8,
           group=g)
    b.rect(x + 58, y + 78, 150, 76, fill=C["panel_alt"], r=6, group=g,
           name="03/appr/left")
    b.rect(x + 214, y + 78, 150, 76, fill=C["panel_alt"], r=6, group=g,
           name="03/appr/right")
    for i, (ln, col) in enumerate([("- export const beam = 1;", C["signal"]),
                                   ("+ export const beam = 2;", C["verify"]),
                                   ("  // unchanged", C["ink500"])]):
        b.text(x + 66, y + 86 + i * 13, ln, 7.2, "Consolas", "Regular", col,
               width=140, line=10, group=g)
        b.text(x + 222, y + 86 + i * 13, ln, 7.2, "Consolas", "Regular", col,
               width=140, line=10, group=g)
    b.text(x + 58, y + 128, "DIFF_PROPOSE / DIFF_APPLY / DIFF_REJECT", 7.5,
           "Consolas", "Regular", C["ink500"], width=290, line=10, group=g)
    b.rect(x + 58, y + 160, 72, 22, fill=C["verify"], r=6, group=g,
           name="03/appr/approve")
    b.text(x + 58, y + 164, "approve", 7.5, "Inter", "SemiBold", C["paper"],
           width=72, line=11, align="center", group=g)
    b.rect(x + 138, y + 160, 72, 22, fill=C["canvas"], r=6, stroke=C["ink700"],
           sw=1.5, group=g, name="03/appr/deny")
    b.text(x + 138, y + 164, "deny", 7.5, "Inter", "SemiBold", C["ink700"],
           width=72, line=11, align="center", group=g)
    note(b, x, y, g, ["decision-gated: no diff surface ships without approval (K-12);",
                      "denial leaves bytes unchanged; the approval row is written"])


def tile_streaming(b):
    x, y, g = tile(b, 0, 3, "Streaming", "S27 · REPLY_DELTA")
    b.rect(x + 44, y + 52, 330, 116, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, group=g, name="03/stream/panel", surface="S27")
    b.rect(x + 56, y + 64, 240, 44, fill=C["honey100"], r=12, group=g,
           name="03/stream/bubble")
    b.text(x + 68, y + 72, "Certainly! Here is the code…", 9, "Inter", "Regular",
           C["ink900"], width=220, line=13, group=g)
    b.text(x + 68, y + 90, "import libraries, Load dataset,", 8, "Consolas",
           "Regular", C["ink700"], width=220, line=11, group=g)
    b.rect(x + 68, y + 116, 232, 40, fill=C["recess"], r=6, group=g,
           name="03/stream/code")
    b.text(x + 76, y + 120, ">> streaming token 1,284…", 7.5, "Consolas",
           "Regular", C["ink700"], width=220, line=10, group=g)
    b.rect(x + 76, y + 136, 2, 12, fill=C["ink900"], group=g,
           name="03/stream/cursor")
    b.text(x + 56, y + 160, "REPLY_DELTA appends; REPLY_DONE clears the cursor", 7.5,
           "Consolas", "Regular", C["ink500"], width=310, line=10, group=g)
    note(b, x, y, g, ["streaming is an agent state; terminal event clears thinking;",
                      "socket kill mid-stream → resnapshot, no duplicated text"])


def tile_motion(b):
    x, y, g = tile(b, 1, 3, "Reduced motion", "prefers-reduced-motion")
    b.rect(x + 44, y + 56, 44, 22, fill=C["recess"], r=999, group=g,
           name="03/motion/track")
    b.rect(x + 46, y + 58, 18, 18, fill=C["ink700"], r=999, group=g,
           name="03/motion/knob")
    b.text(x + 100, y + 60, "Motion: off", 10, "Inter", "SemiBold", C["ink900"],
           width=120, line=14, group=g)
    b.text(x + 100, y + 76, "persists across reload", 8, "Consolas", "Regular",
           C["ink500"], width=160, line=11, group=g)
    b.hex_bee(x + 44, y + 100, 40, C["ink900"], C["honey300"], C["honey100"],
              C["paper"], stripe=C["ink900"], thorax=C["honey500"], group=g,
              name="03/motion/bee")
    b.text(x + 96, y + 110, "bee static", 8.5, "Consolas", "Regular", C["ink700"],
           width=100, line=12, group=g)
    b.rect(x + 200, y + 106, 180, 5, fill=C["recess"], r=999, group=g,
           name="03/motion/track2")
    b.rect(x + 200, y + 107, 99, 3, fill=C["honey600"], r=999, group=g,
           name="03/motion/bar")
    b.text(x + 200, y + 118, "progress static at 55%", 8, "Consolas", "Regular",
           C["ink500"], width=180, line=11, group=g)
    b.text(x + 44, y + 152, "state still updates — only animation stops", 8.5,
           "Consolas", "Regular", C["ink700"], width=360, line=12, group=g)
    note(b, x, y, g, ["global toggle + prefers-reduced-motion + pause when hidden;",
                      "bee / wire / progress are the only motion and all switch off"])


def tile_legend(b):
    x, y, g = tile(b, 2, 3, "State enum legend", "15 members")
    for i, (st, tok, shp) in enumerate(STATES):
        col = i % 3
        row = i // 3
        cx = x + 44 + col * 128
        cy = y + 52 + row * 26
        b.dot(cx, cy + 1, 9, shp, C[tok], group=g, name="03/legend/%s" % st)
        b.text(cx + 15, cy, st, 8.2, "Consolas", "Regular", C["ink700"],
               width=112, line=11, group=g)
    b.text(x + 44, y + 190, "dot + shape + text label — no colour-only state",
           8, "Consolas", "Regular", C["ink500"], width=380, line=11, group=g)
    note(b, x, y, g, ["one enum, ≥13 members required; 15 shipped, superset allowed;",
                      "the four printed dots map 1:1; the rest add shape + text"])


def build_specimens(frame_id):
    b = Board("03", "Specimens", frame_id, W, H)
    b.rect(0, 0, W, H, fill=C["canvas"], name="03/board-bg")

    b.eyebrow(48, 44, "06 / SPECIMENS  ·  11 INTERACTION + GLOBAL STATES  ·  S25-S27  ·  "
                      "PROPOSED-ADDITION (THE PDF SHIPS ZERO STATE ARTBOARDS)",
              group="03-header")
    b.text(48, 64, "Every state, drawn.", 56, "Playfair Display", "SemiBold",
           C["ink900"], width=900, line=60, tracking=-2, name="03/title",
           group="03-header")
    b.body_text(48, 132,
                "One enum drives every visual state. Focus = ink-700 #4E3A15 (9.86:1); the "
                "printed honey-300 ring (1.70) is replaced. Amber = large text only; reduced "
                "motion stops animation, state still updates.", 1120, size=17, line=28,
                group="03-header")
    b.eyebrow(1092, 44, "HIVEROUTER  ·  SPECIMENS  /  V1.0", color=C["ink700"],
              width=300, align="right", group="03-header")
    b.eyebrow(1092, 62, "FIGCRAFT NATIVE  ·  1440 × 1400", color=C["ink500"],
              width=300, align="right", group="03-header")

    b.section(48, 206, 1344, "interaction + global states · 11 tiles + enum legend",
              note="UI-R-44 / UI-R-45 / R-U4 · UI-COMMANDS-v1 §4")
    tile_selected(b)
    tile_hover(b)
    tile_focus(b)
    tile_disabled(b)
    tile_empty(b)
    tile_loading(b)
    tile_offline(b)
    tile_error(b)
    tile_approval(b)
    tile_streaming(b)
    tile_motion(b)
    tile_legend(b)

    b.line(48, 1355, 1344, C["hairline"], 1, name="03/footer-rule")
    b.text(48, 1362, "HIVEROUTER  ·  SPECIMENS  /  V1.0  ·  03 SPECIMENS  ·  "
                     "FIGCRAFT NATIVE", 9, "Consolas", "Regular", C["ink500"],
           width=900, line=13, tracking=8, name="03/footer")
    b.text(1192, 1362, "PAGE 4 / 4  ·  PROPOSED-ADDITION", 9, "Consolas",
           "Regular", C["ink500"], width=200, line=13, align="right",
           name="03/footer-no")
    return b
