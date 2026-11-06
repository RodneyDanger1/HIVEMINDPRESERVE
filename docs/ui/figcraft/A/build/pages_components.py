"""Page 02 Components — node cards, chat, controls, chrome, surface index."""
from gen_common import (Board, C, SURFACES, STATES)

W = 1440
H = 2580
CARD_Y = 266
CARD_H = 300
CARD_X = [92, 412, 732, 1052]
CARD_W = 296


def _card_panel(b, x, y, h, group, surface, name, fill=C["paper"], r=12,
                stroke=C["hairline"], sw=1.0):
    b.rect(x, y, CARD_W, h, fill=fill, r=r, stroke=stroke, sw=sw, group=group,
           surface=surface, name=name)


def super_card(b):
    x, y = CARD_X[0], CARD_Y
    g = "S15-superagent"
    _card_panel(b, x, y, CARD_H, g, "S15", "02/super/panel", r=14,
                stroke=C["honey500"], sw=1.5)
    b.rect(x, y, CARD_W, 72, fill=C["honey100"], r=[14, 14, 0, 0], group=g,
           name="02/super/header")
    b.name_text(x + 16, y + 14, "Backend Architecture", 14.5, group=g)
    b.rect(x + 200, y + 17, 80, 20, fill=C["honey300"], r=999, group=g,
           name="02/super/pill")
    b.dot(x + 208, y + 23, 7, "circle", C["honey800"], group=g,
          name="02/super/pill-dot")
    b.text(x + 220, y + 22, "RUNNING", 8, "Inter", "SemiBold", C["ink900"],
           width=54, line=11, tracking=8, group=g, name="02/super/pill-label")
    b.text(x + 16, y + 86, "SUPERAGENT", 9.5, "Consolas", "Regular", C["ink500"],
           width=200, line=13, tracking=12, group=g)
    b.text(x + 16, y + 104, "decomposes intent,", 12, "Inter", "Regular",
           C["ink700"], width=264, line=17, group=g)
    b.text(x + 16, y + 121, "owns the sub-hive", 12, "Inter", "Regular",
           C["ink700"], width=264, line=17, group=g)
    b.text(x + 16, y + 146, "3 workers  ·  12.4k ctx", 9.5, "Consolas", "Regular",
           C["ink500"], width=264, line=13, group=g)
    for i in range(3):
        b.hex_bee(x + 16 + i * 30, y + 168, 22, C["honey300"], C["ink900"],
                  C["ink900"], C["honey100"], stripe=C["honey300"],
                  group=g, name="02/super/worker-chip-%d" % (i + 1))
    b.rect(x + 16, y + 200, 264, 70, fill=C["honey100"], r=8, stroke=C["honey300"],
           sw=1, dash=[5, 4], group=g, name="02/super/containment")
    b.text(x + 26, y + 210, "SUB-HIVE CONTAINMENT", 8, "Consolas", "Regular",
           C["honey800"], width=230, line=11, tracking=8, group=g)
    b.text(x + 26, y + 224, "child workers clip inside; the card grows downward",
           8.5, "Inter", "Regular", C["ink700"], width=244, line=12, group=g)
    b.text(x + 26, y + 242, "radius 14 (off-scale exception)", 8, "Consolas",
           "Regular", C["ink500"], width=244, line=11, group=g)
    b.text(x, y + CARD_H + 8, "S15 · SUPERAGENT / SUB-HIVE · PDF p6:185-191", 9,
           "Consolas", "Regular", C["ink500"], width=CARD_W, line=13, group=g)


def worker_card(b):
    x, y = CARD_X[1], CARD_Y
    g = "S16-worker"
    _card_panel(b, x, y, CARD_H, g, "S16", "02/worker/panel", r=12)
    b.name_text(x + 16, y + 16, "tailwind-builder", 14.5, group=g)
    b.text(x + 16, y + 42, "WORKER  ·  react/vite", 9.5, "Consolas", "Regular",
           C["ink500"], width=200, line=13, tracking=12, group=g)
    b.text(x + 16, y + 56, "55%", 10, "Consolas", "Regular", C["ink700"],
           width=264, line=14, align="right", group=g)
    b.rect(x + 16, y + 74, 264, 5, fill=C["recess"], r=999, group=g,
           name="02/worker/track")
    b.rect(x + 16, y + 75, 145, 3, fill=C["honey600"], r=999, group=g,
           name="02/worker/bar")
    b.text(x + 16, y + 92, "5px track / 3px worker · progress is the only motion",
           8.5, "Consolas", "Regular", C["ink500"], width=264, line=12, group=g)
    b.dot(x + 16, y + 124, 8, "circle", C["honey500"], group=g,
          name="02/worker/state-dot")
    b.text(x + 30, y + 121, "running", 9.5, "Consolas", "Regular", C["ink700"],
           width=120, line=13, group=g)
    b.text(x + 16, y + 152, "meta  ·  0.55  ·  react/vite", 9.5, "Consolas",
           "Regular", C["ink500"], width=264, line=13, group=g)
    b.text(x + 16, y + 168, "no header fill — the progress bar is the only motion",
           9, "Inter", "Regular", C["ink700"], width=264, line=13, group=g)
    b.rect(x + 16, y + 192, 264, 54, fill=C["panel_alt"], r=8, stroke=C["hairline"],
           sw=1, group=g, name="02/worker/spec")
    b.text(x + 26, y + 184, "CARD ANATOMY / WORKER", 8, "Consolas", "Regular",
           C["ink500"], width=230, line=11, tracking=8, group=g)
    b.text(x + 26, y + 207, "name 14.5 · role mono 9.5 · meta mono 9.5 #75663F",
           8, "Consolas", "Regular", C["ink700"], width=244, line=11, group=g)
    b.text(x + 26, y + 220, "radius 12 · padding 16 · progress 5/3",
           8, "Consolas", "Regular", C["ink700"], width=244, line=11, group=g)
    b.text(x, y + CARD_H + 8, "S16 · WORKERAGENT · PDF p6:192-196,222", 9,
           "Consolas", "Regular", C["ink500"], width=CARD_W, line=13, group=g)


def tool_card(b):
    x, y = CARD_X[2], CARD_Y
    g = "S17-tool"
    _card_panel(b, x, y, CARD_H, g, "S17", "02/tool/panel", r=12)
    b.name_text(x + 16, y + 16, "Prisma Generator", 14.5, group=g)
    b.text(x + 16, y + 42, "TOOL  ·  code", 9.5, "Consolas", "Regular", C["ink500"],
           width=200, line=13, tracking=12, group=g)
    b.rect(x + 16, y + 64, 264, 140, fill=C["ink900"], r=8, group=g,
           name="02/tool/syntax")
    code = ["model Session {", "  id   String @id", "  run  Run[]", "}"]
    for i, line in enumerate(code):
        b.text(x + 28, y + 76 + i * 20, line, 10.5, "Consolas", "Regular",
               C["honey100"] if i in (0, 3) else C["honey300"], width=240,
               line=16, group=g)
    b.rect(x + 16, y + 214, 206, 3, fill=C["honey700"], r=999, group=g,
           name="02/tool/exec-bar")
    b.text(x + 230, y + 204, "78%", 10, "Consolas", "Regular", C["ink700"],
           width=50, line=14, align="right", group=g)
    b.text(x + 16, y + 228, "schema.prisma  ·  0.78", 9.5, "Consolas", "Regular",
           C["ink500"], width=264, line=13, group=g)
    b.text(x + 16, y + 252, "dark syntax = ink-900 code surface (PDF p6:197-204)",
           8, "Consolas", "Regular", C["ink500"], width=264, line=11, group=g)
    b.text(x + 16, y + 266, "gold bar = honey-700 · score shown when measured",
           8, "Consolas", "Regular", C["ink500"], width=264, line=11, group=g)
    b.text(x, y + CARD_H + 8, "S17 · TOOLAGENT / NESTED CODE · PDF p6:197-204", 9,
           "Consolas", "Regular", C["ink500"], width=CARD_W, line=13, group=g)


def error_card(b):
    x, y = CARD_X[3], CARD_Y
    g = "S18-error"
    _card_panel(b, x, y, CARD_H, g, "S18", "02/error/panel", r=12,
                stroke=C["signal"], sw=1.5)
    b.text(x + 16, y + 14, "BROKEN ROUTE", 9.5, "Consolas", "Regular",
           C["signal"], width=200, line=13, tracking=12, group=g)
    b.text(x + 16, y + 34, "Compile failed", 16, "Inter", "SemiBold", C["ink900"],
           width=264, line=20, group=g)
    b.text(x + 16, y + 60, "db-schema  ·  E0421  ·  retry 2/3", 9.5, "Consolas",
           "Regular", C["signal"], width=264, line=13, group=g)
    b.text(x + 16, y + 82, "Prisma client type mismatch on Run.sessionId", 11.5,
           "Inter", "Regular", C["ink700"], width=264, line=17, group=g)
    # halted bee + warning glyph
    b.hex_bee(x + 16, y + 116, 30, C["signal"], C["paper"], C["paper"],
              C["paper"], stripe=C["signal"], group=g, name="02/error/bee")
    b.poly(x + 40, y + 112, 18, 16, C["amber"], pointCount=3, group=g,
           name="02/error/warn")
    b.text(x + 46, y + 118, "!", 10, "Inter", "SemiBold", C["ink900"], width=10,
           line=12, align="center", group=g)
    b.text(x + 66, y + 118, "halted bee + warning glyph", 8, "Consolas",
           "Regular", C["ink500"], width=200, line=11, group=g)
    b.line(x + 16, y + 168, 120, C["signal"], 1.5, dash=[6, 4], group=g,
           name="02/error/dashed-wire")
    b.text(x + 144, y + 162, "dashed crimson wire", 8, "Consolas", "Regular",
           C["ink500"], width=140, line=11, group=g)
    b.text(x + 16, y + 196, "no retry beyond 3/3; the run ledger owns the counter",
           8.5, "Inter", "Regular", C["ink700"], width=264, line=12, group=g)
    b.text(x + 16, y + 216, "retry action bounded · E-code from run ledger", 8,
           "Consolas", "Regular", C["ink500"], width=264, line=11, group=g)
    b.text(x + 16, y + 250, "radius 12 · crimson 1.5px border (state/error)", 8,
           "Consolas", "Regular", C["ink500"], width=264, line=11, group=g)
    b.text(x, y + CARD_H + 8, "S18 · BROKEN-ROUTE ERROR CARD · PDF p6:205-210", 9,
           "Consolas", "Regular", C["ink500"], width=CARD_W, line=13, group=g)


def anatomy_band(b):
    y = 612
    # left: card anatomy (8px grid)
    b.section(48, y, 656, "card anatomy · 8px grid", note="PDF p6:218-226")
    b.rect(48, y + 24, 656, 250, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, group="02-anatomy", name="02/anatomy/panel")
    rows = [
        ("name", "Inter SemiBold 14.5", "card title, one line"),
        ("role tag", "mono 9.5 · +12% tracking", "SUPERAGENT / WORKER / TOOL"),
        ("state", "dot or pill — never both", "single state enum · label always"),
        ("progress", "5px track · 3px worker bar", "worker cards only"),
        ("meta", "mono 9.5 · #75663F", "corrected from #8A7A52 (4.21/3.84 fails AA)"),
        ("radius", "14 super · 12 worker", "both off the declared scale (exceptions)"),
        ("padding", "16", "all sides, all node types"),
        ("gap", "8 / 16 / 24 only", "no other spacing values inside a card"),
    ]
    for i, (k, v, note) in enumerate(rows):
        ry = y + 36 + i * 28
        b.text(64, ry, k.upper(), 8.5, "Consolas", "Regular", C["ink500"],
               width=120, line=12, tracking=8, group="02-anatomy")
        b.text(184, ry, v, 11, "Inter", "SemiBold", C["ink900"], width=250,
               line=15, group="02-anatomy")
        b.text(444, ry, note, 8.5, "Consolas", "Regular", C["ink500"],
               width=248, line=12, group="02-anatomy")
        if i < len(rows) - 1:
            b.line(64, ry + 22, 624, C["hairline"], 1, group="02-anatomy",
                   name="02/anatomy/rule-%d" % i)
    b.text(64, y + 262, "Per-type matrix required: the chat core, ToolAgent and the error card "
                        "lack ≥2 of the four shared fields (UI-R-12 / K-14).", 8,
           "Consolas", "Regular", C["ink500"], width=624, line=11,
           group="02-anatomy")

    # right: agent chips + state dots
    b.section(736, y, 656, "agent chips · state dot", note="PDF p6:211-217")
    b.rect(736, y + 24, 656, 250, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, group="S19-chips", surface="S19", name="02/chips/panel")
    names = [("Nova", C["honey300"]), ("Vector", C["honey400"]),
             ("Spark", C["honey500"])]
    for i, (nm, col) in enumerate(names):
        cx = 768 + i * 96
        b.hex_bee(cx, y + 42, 40, col, C["ink900"], C["ink900"], C["paper"],
                  stripe=col, group="S19-chips", surface="S19" if i == 0 else None,
                  name="02/chips/%s" % nm)
        b.text(cx - 8, y + 88, nm, 9.5, "Consolas", "Regular", C["ink700"],
               width=56, line=13, align="center", group="S19-chips")
    b.text(768, y + 112, "chip = 40px hex · bee = state colour", 8.5, "Consolas",
           "Regular", C["ink500"], width=320, line=12, group="S19-chips")
    b.line(768, y + 128, 592, C["hairline"], 1, group="S19-chips",
           name="02/chips/rule")
    b.text(768, y + 136, "STATE DOT + TEXT CUE — NEVER COLOUR ALONE", 8.5,
           "Consolas", "Regular", C["ink700"], width=400, line=12, tracking=8,
           group="S19-chips")
    dots = [("healthy", "circle"), ("running", "circle"),
            ("failed", "triangle"), ("queued", "square")]
    for i, (st, shp) in enumerate(dots):
        cx = 768 + i * 150
        b.dot(cx, y + 160, 10, shp, C[{"healthy": "verify", "running": "honey500",
                                       "failed": "signal", "queued": "ink500"}[st]],
              group="S19-chips", name="02/chips/dot-%s" % st)
        b.text(cx + 16, y + 157, st, 9.5, "Consolas", "Regular", C["ink700"],
               width=120, line=13, group="S19-chips")
        b.text(cx, y + 178, {"healthy": "dot › verify", "running": "dot › honey-500",
                             "failed": "triangle › signal",
                             "queued": "square › ink-500"}[st],
               8, "Consolas", "Regular", C["ink500"], width=140, line=11,
               group="S19-chips")
    b.text(768, y + 204, "Four printed dots map 1:1 into the ≥13-state enum; every other "
                         "state adds a shape + text cue. Dot or pill — never both.",
           8.5, "Inter", "Regular", C["ink700"], width=592, line=12,
           group="S19-chips")
    b.text(768, y + 236, "honey = healthy, crimson = broken route: colour is state, "
                         "not decoration (CF-UI-05 recorded).", 8, "Consolas",
           "Regular", C["ink500"], width=592, line=11, group="S19-chips")


def chat_core(b):
    x, y, w, h = 48, 932, 420, 400
    g = "S02-chat"
    b.rect(x, y, w, h, fill=C["paper"], r=16, stroke=C["hairline"], sw=1, group=g,
           surface="S02", name="02/chat/panel")
    b.rect(x, y, w, 52, fill=C["honey100"], r=[16, 16, 0, 0], group=g,
           name="02/chat/topbar")
    b.hex_bee(x + 14, y + 12, 28, C["honey300"], C["ink900"], C["ink900"],
              C["honey100"], stripe=C["honey300"], group=g,
              name="02/chat/icon")
    b.name_text(x + 52, y + 16, "Hive AI Assistant", 13, group=g)
    b.text(x + 344, y + 14, "×", 18, "Inter", "Regular", C["ink700"], width=28,
           line=24, align="center", group=g, name="02/chat/close")
    b.text(x + 14, y + 66, "CHAT CORE · PINNED", 8, "Consolas", "Regular",
           C["ink500"], width=200, line=11, tracking=8, group=g)
    # user bubble
    b.rect(x + 176, y + 84, 228, 44, fill=C["honey200"], r=12, group=g,
           name="02/chat/user-bubble")
    b.text(x + 188, y + 92, "Generate a Python script for sentiment analysis.",
           10.5, "Inter", "Regular", C["ink900"], width=208, line=14, group=g)
    # assistant message
    b.text(x + 14, y + 140, "Certainly! Here is the code...", 10.5, "Inter",
           "Regular", C["ink700"], width=250, line=15, group=g)
    b.rect(x + 14, y + 160, 252, 92, fill=C["recess"], r=8, group=g,
           name="02/chat/code")
    for i, line in enumerate(["import libraries,", "Load dataset,",
                              "Preprocess data..."]):
        b.text(x + 24, y + 168 + i * 18, line, 9.5, "Consolas", "Regular",
               C["ink800"], width=232, line=14, group=g)
    b.text(x + 14, y + 262, "messages persist across pan/zoom · closing hides the "
                            "widget only", 8, "Consolas", "Regular", C["ink500"],
           width=390, line=11, group=g)
    # input surface + composer row
    b.rect(x + 14, y + 290, 300, 44, fill=C["honey100"], r=10, group=g,
           name="02/chat/input")
    b.text(x + 26, y + 303, "Type your prompt...", 11, "Consolas", "Regular",
           C["ink500"], width=250, line=15, group=g)
    b.rect(x + 322, y + 290, 84, 44, fill=C["honey500"], r=10, group=g,
           name="02/chat/send")
    b.text(x + 322, y + 302, "Send", 11, "Inter", "SemiBold", C["ink900"],
           width=84, line=15, align="center", group=g)
    b.text(x + 14, y + 350, "off-white #FBF3DC input surface · beehive containment · "
                            "the only eternal node", 8, "Consolas", "Regular",
           C["ink500"], width=390, line=11, group=g)
    b.text(x + 14, y + 366, "pan/zoom never move it · Ctrl+0 recentres the viewport on it",
           8, "Consolas", "Regular", C["ink500"], width=390, line=11, group=g)
    b.text(x, y + h + 8, "S02 · HYPERAGENT / HIVE CHAT CORE · PDF p6:175-184,296-302",
           9, "Consolas", "Regular", C["ink500"], width=w, line=13, group=g)


def composer(b):
    x, y, w, h = 492, 932, 440, 180
    g = "S03-composer"
    b.rect(x, y, w, h, fill=C["paper"], r=12, stroke=C["hairline"], sw=1, group=g,
           surface="S03", name="02/composer/panel")
    b.text(x + 16, y + 12, "COMPOSER · HYPERAGENT INPUT", 8.5, "Consolas",
           "Regular", C["ink700"], width=300, line=12, tracking=8, group=g)
    b.rect(x + 16, y + 32, 320, 44, fill=C["panel_alt"], r=10, stroke=C["ink700"],
           sw=2.0, group=g, name="02/composer/focus-ring")
    b.rect(x + 18, y + 34, 316, 40, fill=C["honey100"], r=9, group=g,
           name="02/composer/input")
    b.text(x + 30, y + 47, "Type your prompt...", 11, "Consolas", "Regular",
           C["ink500"], width=290, line=15, group=g)
    b.rect(x + 344, y + 32, 80, 44, fill=C["honey500"], r=10, group=g,
           name="02/composer/submit")
    b.text(x + 344, y + 45, "Send →", 11, "Inter", "SemiBold", C["ink900"],
           width=80, line=15, align="center", group=g)
    b.text(x + 16, y + 84, "focus: ink-700 #4E3A15 (9.86:1) — printed honey-300 ring "
                           "failed 3:1", 8.5,
           "Consolas", "Regular", C["signal"], width=420, line=12, group=g)
    b.text(x + 16, y + 100, "placeholder “Type your prompt…” · Enter submits · "
                            "Shift+Enter newline", 8.5, "Consolas", "Regular",
           C["ink700"], width=408, line=12, group=g)
    b.text(x + 16, y + 114, "empty → E_PROMPT_EMPTY · disabled in-flight · duplicate "
                            "submit = one run", 8.5, "Consolas", "Regular",
           C["ink700"], width=408, line=12, group=g)
    b.text(x + 16, y + 132, "SUBMIT_PROMPT {text, project_id, correlation_id} · ACK or "
                            "named refusal before mutation", 8, "Consolas", "Regular",
           C["ink500"], width=408, line=11, group=g)
    b.text(x, y + h + 8, "S03 · COMPOSER · PDF p7:235-237,302 · UI-R-04 / UI-R-29",
           9, "Consolas", "Regular", C["ink500"], width=w, line=13, group=g)


def toast(b):
    x, y, w, h = 492, 1132, 440, 210
    g = "S14-toast"
    b.rect(x, y, w, h, fill=C["paper"], r=12, stroke=C["hairline"], sw=1, group=g,
           surface="S14", name="02/toast/panel")
    b.rect(x, y, 5, h, fill=C["signal"], r=[12, 0, 0, 12], group=g,
           name="02/toast/bar")
    b.text(x + 20, y + 14, "R E F U S A L", 8.5, "Consolas", "Regular",
           C["signal"], width=200, line=12, tracking=8, group=g)
    b.text(x + 20, y + 30, "Re-route refused", 15, "Inter", "SemiBold",
           C["signal"], width=380, line=20, group=g)
    b.text(x + 20, y + 56, "context-prune blocked: worker still streaming "
                           "(NODE_REWIRE pending)", 10.5, "Inter", "Regular",
           C["ink700"], width=390, line=15, group=g)
    b.text(x + 20, y + 88, "E_REWIRE_IN_FLIGHT  ·  retryable  ·  how_to_fix: wait for "
                           "stream end or cancel the run", 9, "Consolas", "Regular",
           C["signal"], width=390, line=13, group=g)
    b.rect(x + 20, y + 124, 392, 22, fill=C["honey100"], r=6, group=g,
           name="02/toast/footer")
    b.text(x + 28, y + 130, "REFUSED BY NAME  ·  NOT SILENT", 8.5, "Consolas",
           "Regular", C["ink700"], width=300, line=13, tracking=8, group=g)
    b.text(x + 20, y + 156, "code + cause + how_to_fix + retryable · aria-live=assertive · "
                            "fired before any UI mutation · logged as a row",
           8.5, "Inter", "Regular", C["ink700"], width=390, line=12, group=g)
    b.text(x + 20, y + 184, "Fixtures 429, budget, rewire-while-streaming → three "
                            "coded refusals", 8, "Consolas",
           "Regular", C["ink500"], width=390, line=11, group=g)
    b.text(x, y + h + 8, "S14 · TOAST / REFUSAL · PDF p7:269-273 · MP-07 / B12",
           9, "Consolas", "Regular", C["ink500"], width=w, line=13, group=g)


def tabs_hud_zoom(b):
    # tabs
    x, y, w = 956, 932, 436
    g = "S08-tabs"
    b.rect(x, y, w, 100, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g, surface="S08", name="02/tabs/panel")
    tabs = ["Workspace", "Swarm", "Routing", "History"]
    tx = x + 16
    for i, t in enumerate(tabs):
        tw = 12 + len(t) * 7.2
        b.text(tx, y + 20, t, 11.5, "Inter", "SemiBold",
               C["ink900"] if i == 0 else C["ink500"], width=tw + 10, line=16,
               group=g)
        if i == 0:
            b.rect(tx, y + 40, tw, 2, fill=C["honey700"], r=1, group=g,
                   name="02/tabs/active-underline")
        tx += tw + 22
    b.text(x + 16, y + 56, "4 tabs · active = ink-900 + honey-700 underline · "
                            "rail = sections, tabs = canvas modes", 8.5, "Consolas",
           "Regular", C["ink500"], width=404, line=12, group=g)
    b.text(x + 16, y + 74, "one navigation owner rule required (R-U11 / K-08): each "
                            "destination owned exactly once", 8.5, "Consolas",
           "Regular", C["ink500"], width=404, line=12, group=g)

    # HUD
    y2 = 1052
    g2 = "S09-hud"
    b.rect(x, y2, w, 100, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g2, surface="S09", name="02/hud/panel")
    b.text(x + 16, y2 + 12, "HUD · THROUGHPUT BADGE", 8.5, "Consolas", "Regular",
           C["ink700"], width=300, line=12, tracking=8, group=g2)
    b.rect(x + 16, y2 + 30, 404, 34, fill=C["ink900"], r=999, group=g2,
           name="02/hud/badge")
    b.dot(x + 30, y2 + 43, 8, "circle", C["verify"], group=g2,
          name="02/hud/dot")
    b.text(x + 46, y2 + 40, "4 SUPERAGENTS  ·  12 WORKERS  ·  98.2% GREEN", 10,
           "Consolas", "Regular", C["honey100"], width=300, line=14, group=g2)
    b.rect(x + 340, y2 + 37, 68, 20, fill=C["verify"], r=999, group=g2,
           name="02/hud/delta")
    b.text(x + 340, y2 + 41, "+3.4%", 8.5, "Consolas", "Regular", C["paper"],
           width=68, line=12, align="center", group=g2)
    b.text(x + 16, y2 + 72, "derived aggregate · changing graph changes numbers · "
                             "empty swarm renders “—” not 0%", 8, "Consolas",
           "Regular", C["ink500"], width=404, line=11, group=g2)

    # zoom + shortcut
    y3 = 1172
    g3 = "S20-zoom"
    b.rect(x, y3, w, 100, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g3, surface="S20", name="02/zoom/panel")
    b.text(x + 16, y3 + 12, "ZOOM 0.1× — 3.0×  ·  CTRL+0 RECENTRES", 8.5,
           "Consolas", "Regular", C["ink700"], width=360, line=12, tracking=8,
           group=g3)
    b.rect(x + 16, y3 + 30, 30, 30, fill=C["canvas"], r=8, stroke=C["hairline"],
           sw=1, group=g3, name="02/zoom/minus")
    b.text(x + 16, y3 + 36, "−", 14, "Inter", "SemiBold", C["ink700"], width=30,
           line=18, align="center", group=g3)
    b.text(x + 58, y3 + 36, "0.1× — 3.0×", 12, "Consolas", "Regular", C["ink900"],
           width=180, line=16, group=g3)
    b.rect(x + 250, y3 + 30, 30, 30, fill=C["canvas"], r=8, stroke=C["hairline"],
           sw=1, group=g3, name="02/zoom/plus")
    b.text(x + 250, y3 + 36, "+", 14, "Inter", "SemiBold", C["ink700"], width=30,
           line=18, align="center", group=g3)
    b.text(x + 16, y3 + 68, "explicit minZoom 0.1 / maxZoom 3.0 — library defaults "
                             "0.5/2 silently clip (K-02)", 8.5, "Consolas",
           "Regular", C["ink500"], width=404, line=12, group=g3)
    b.text(x + 16, y3 + 82, "Cmd+0 on the board → Ctrl+0 on Windows (K-01); ignored "
                             "while a text field has focus", 8, "Consolas",
           "Regular", C["ink500"], width=404, line=11, group=g3)

    # state vocabulary pointer
    b.rect(x, 1292, w, 40, fill=C["honey100"], r=8, group="02-statepointer",
           name="02/statepointer")
    b.text(x + 12, 1302, "15-STATE VOCABULARY — DRAWN ON 03 SPECIMENS  ·  "
                         "UI-COMMANDS-v1 §4", 8.5, "Consolas", "Regular",
           C["honey800"], width=410, line=12, tracking=4, group="02-statepointer")


def buttons_matrix(b):
    x, y, w, h = 48, 1396, 860, 330
    b.section(48, 1370, 860, "buttons · spawn / re-route / cancel / kill",
              note="PDF p7:238-240")
    b.rect(x, y, w, h, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           name="02/buttons/panel")
    states = ["default", "hover", "pressed", "disabled", "refused"]
    col_x = [x + 150 + i * 140 for i in range(5)]
    for i, st in enumerate(states):
        b.text(col_x[i], y + 14, st.upper(), 8, "Consolas", "Regular",
               C["ink500"], width=128, line=11, align="center", tracking=8)
    rows = [
        ("Spawn sub-hive", "primary", "S04-spawn", "S04", "SPAWN_SUB_HIVE",
         "E_SPAWN_DEPTH"),
        ("Re-route", "secondary", "S05-reroute", "S05", "REWIRE_TASK", "E_CYCLE"),
        ("Cancel", "ghost", "S06-cancel", "S06", "CANCEL_RUN", "E_RUN_TERMINAL"),
        ("Kill worker", "danger", "S07-kill", "S07", "KILL_WORKER", "E_KILL_DENIED"),
    ]
    for r, (label, variant, g, surf, cmd, refusal) in enumerate(rows):
        ry = y + 36 + r * 68
        b.surfaces[surf] = len(b.el)
        b.name_text(x + 16, ry + 8, label, 12,
                    name="02/row/%s/label" % surf)
        b.text(x + 16, ry + 26, cmd, 8, "Consolas", "Regular", C["ink500"],
               width=130, line=11)
        b.text(x + 16, ry + 40, "min 40px · radius 10", 8, "Consolas", "Regular",
               C["ink500"], width=130, line=11)
        for i, st in enumerate(states):
            b.button(col_x[i], ry + 2, 128, 40, label, variant, st,
                     name="02/btn/%s/%s" % (cmd.lower(), st))
        b.text(col_x[4], ry + 47, refusal, 7.5, "Consolas", "Regular",
               C["signal"], width=128, line=10, align="center",
               name="02/btn/%s/refusal-code" % cmd.lower())
    b.text(x + 16, y + 308, "40px min touch target · radius 10 is an off-scale "
                            "exception · label Inter SemiBold 13 · ACK or named refusal "
                            "before UI mutation · REFUSED column prints the refusal code "
                            "by name · DIFF_PROPOSE/APPLY/REJECT DECISION-GATED (S25).",
           8.5, "Consolas", "Regular", C["ink500"], width=820, line=12)


def settings_and_states(b):
    x, y, w = 936, 1396, 456
    b.section(936, 1370, 456, "settings · key status", note="PDF p7:254")
    g = "S21-settings"
    b.rect(x, y, w, 168, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g, surface="S21", name="02/settings/panel")
    b.text(x + 16, y + 12, "SETTINGS / PROVIDER + KEY", 8.5, "Consolas", "Regular",
           C["ink700"], width=300, line=12, tracking=8, group=g)
    rows = [
        ("provider", "deepseek", C["ink900"]),
        ("model", "deepseek-chat", C["ink900"]),
        ("api key", "present", C["verify"]),
    ]
    for i, (k, v, col) in enumerate(rows):
        ry = y + 34 + i * 24
        b.text(x + 16, ry, k.upper(), 8.5, "Consolas", "Regular", C["ink500"],
               width=110, line=12, tracking=8, group=g)
        b.text(x + 130, ry, v, 11, "Consolas", "Regular", col, width=200, line=14,
               group=g)
        if k == "api key":
            b.dot(x + 118, ry + 2, 8, "circle", C["verify"], group=g,
                  name="02/settings/key-dot")
    b.line(x + 16, y + 110, 424, C["hairline"], 1, group=g, name="02/settings/rule")
    b.text(x + 16, y + 118, "MISSING-KEY REFUSAL VARIANT", 8, "Consolas", "Regular",
           C["signal"], width=300, line=11, tracking=8, group=g)
    b.text(x + 16, y + 132, "key absent → E_KEY_MISSING from composer AND settings; "
                            "never fake a run.", 8.5, "Inter", "Regular", C["ink700"],
           width=424, line=12, group=g)
    b.text(x + 16, y + 148, "no key → named refusal; the run row is never created.",
           8, "Consolas", "Regular", C["ink500"], width=424, line=11, group=g)
    b.text(x, y + 176, "S21 · SETTINGS / KEY STATUS · PDF p7:254", 9, "Consolas",
           "Regular", C["ink500"], width=w, line=13, group=g)

    # state vocabulary (compact, pointer to specimens)
    y2 = 1606
    b.section(936, 1580, 456, "state enum · 15 members", note="UI-COMMANDS-v1 §4")
    b.rect(x, y2, w, 120, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group="02-stateenum", name="02/stateenum/panel")
    for i, (st, tok, shp) in enumerate(STATES):
        col = i % 4
        row = i // 4
        cx = x + 16 + col * 110
        cy = y2 + 14 + row * 22
        b.dot(cx, cy + 1, 8, shp, C[tok], group="02-stateenum",
              name="02/stateenum/%s" % st)
        b.text(cx + 13, cy, st, 7.8, "Consolas", "Regular", C["ink700"], width=96,
               line=11, group="02-stateenum")
    b.text(x + 16, y2 + 106, "dot + shape + text label — no colour-only state; every "
                             "member has a token", 8, "Consolas", "Regular",
           C["ink500"], width=424, line=11, group="02-stateenum")


def nav_rail(b):
    x, y, w, h = 48, 1796, 72, 420
    g = "S11-rail"
    b.rect(x, y, w, h, fill=C["paper"], r=16, stroke=C["hairline"], sw=1,
           group=g, surface="S11", name="02/rail/panel")
    items = ["Workspace", "Agents", "Data", "Models", "Routing", "Settings"]
    for i, it in enumerate(items):
        iy = y + 14 + i * 66
        active = i == 0
        if active:
            b.rect(x + 8, iy, 56, 58, fill=C["honey200"], r=12, group=g,
                   name="02/rail/active")
        b.rect(x + 26, iy + 10, 20, 20, fill=None, r=5,
               stroke=C["ink900"] if active else C["ink500"], sw=1.5, group=g,
               name="02/rail/icon-%d" % (i + 1))
        b.rect(x + 31, iy + 15, 10, 10, fill=C["honey500"] if active else None,
               r=2, stroke=None if active else C["ink500"], sw=1, group=g,
               name="02/rail/icon-dot-%d" % (i + 1))
        b.text(x + 4, iy + 38, it, 8.2, "Inter", "SemiBold" if active else "Regular",
               C["ink900"] if active else C["ink500"], width=64, line=11,
               align="center", group=g)
    b.text(x, y + h + 8, "S11 · NAV RAIL", 9, "Consolas",
           "Regular", C["ink500"], width=90, line=13, group=g)
    b.text(x, y + h + 22, "72px fixed · never scrolls", 8, "Consolas", "Regular",
           C["ink500"], width=92, line=11, group=g)


def library_panel(b):
    x, y, w, h = 144, 1796, 260, 420
    g = "S12-library"
    b.rect(x, y, w, h, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g, surface="S12", name="02/library/panel")
    # project identity (S28)
    g28 = "S28-project"
    b.text(x + 14, y + 14, "Project: HiveMind AI", 12.5, "Inter", "SemiBold",
           C["ink900"], width=230, line=16, group=g, surface="S28",
           name="02/library/project-name")
    b.text(x + 14, y + 32, "AC A. Chen", 9, "Consolas", "Regular", C["ink500"],
           width=150, line=12, group=g)
    b.text(x + 14, y + 48, "project_id from state · never hardcoded", 8, "Consolas",
           "Regular", C["ink500"], width=230, line=11, group=g)
    b.line(x + 14, y + 64, 232, C["hairline"], 1, group=g, name="02/library/rule1")
    b.text(x + 14, y + 72, "ACTIVE AGENTS", 8.5, "Consolas", "Regular", C["ink700"],
           width=200, line=12, tracking=12, group=g)
    agents = [("Nova", C["honey300"]), ("Vector", C["honey400"]),
              ("Spark", C["honey500"])]
    for i, (nm, col) in enumerate(agents):
        ay = y + 92 + i * 30
        b.hex_bee(x + 14, ay, 22, col, C["ink900"], C["ink900"], C["paper"],
                  stripe=col, group=g, name="02/library/agent-%s" % nm)
        b.text(x + 44, ay + 2, nm, 10, "Consolas", "Regular", C["ink700"],
               width=120, line=13, group=g)
        b.text(x + 44, ay + 14, "running", 8, "Consolas", "Regular", C["ink500"],
               width=100, line=11, group=g)
    b.line(x + 14, y + 192, 232, C["hairline"], 1, group=g,
           name="02/library/rule2")
    b.text(x + 14, y + 200, "TEMPLATES", 8.5, "Consolas", "Regular", C["ink700"],
           width=160, line=12, tracking=12, group=g)
    templates = ["Content Writer", "Data Analyst", "Code Reviewer", "Chatbot"]
    for i, t in enumerate(templates):
        ty = y + 220 + i * 22
        b.text(x + 14, ty, "+", 10, "Consolas", "Regular", C["honey700"], width=14,
               line=12, group=g)
        b.text(x + 30, ty, t, 9.5, "Inter", "Regular", C["ink700"], width=180,
               line=12, group=g)
        b.text(x + 200, ty, "MIT", 7.5, "Consolas", "Regular", C["ink500"],
               width=40, line=11, align="right", group=g)
    b.rect(x + 14, y + 316, 232, 88, fill=C["honey100"], r=8, group=g,
           name="02/library/tip")
    b.text(x + 22, y + 322, "TIP", 8, "Consolas", "Regular", C["honey800"],
           width=40, line=11, tracking=8, group=g)
    b.text(x + 22, y + 336, "Drag a worker onto another super-agent to re-route its "
                            "context stream live.", 9, "Inter", "Regular",
           C["ink700"], width=216, line=13, group=g)
    b.text(x + 14, y + 412, "TEMPLATE_INSERT · licence row required", 7.5,
           "Consolas", "Regular", C["ink500"], width=230, line=10, group=g)
    b.text(x, y + h + 8, "S12 · LIBRARY PANEL 260PX · PDF p7:255-266", 9,
           "Consolas", "Regular", C["ink500"], width=w, line=13, group=g)


def minimap(b):
    x, y, w, h = 428, 1796, 260, 170
    g = "S13-minimap"
    b.rect(x, y, w, h, fill=C["recess"], r=12, group=g, surface="S13",
           name="02/minimap/panel")
    nodes = [(x + 60, y + 40), (x + 110, y + 84), (x + 168, y + 46),
             (x + 196, y + 104), (x + 96, y + 122)]
    b.rect(x + 78, y + 58, 1, 48, fill=C["honey600"], group=g,
           name="02/minimap/edge-a")
    b.rect(x + 78, y + 58, 120, 1, fill=C["honey600"], group=g,
           name="02/minimap/edge-b")
    b.rect(x + 128, y + 100, 1, 40, fill=C["honey600"], group=g,
           name="02/minimap/edge-c")
    for i, (nx, ny) in enumerate(nodes):
        b.rect(nx, ny, 16, 16, fill=C["honey300"] if i < 3 else C["paper"], r=3,
               stroke=C["honey700"], sw=1, group=g,
               name="02/minimap/node-%d" % (i + 1))
    b.rect(x + 90, y + 60, 120, 84, fill=None, r=4, stroke=C["honey500"], sw=1.5,
           group=g, name="02/minimap/viewport")
    b.text(x + 8, y + 148, "MINIMAP 260 × 170 · viewport rect ≤1 frame", 8,
           "Consolas", "Regular", C["ink700"], width=244, line=11, group=g)
    b.text(x, y + h + 8, "S13 · MINI-MAP · PDF p7:267-268", 9, "Consolas",
           "Regular", C["ink500"], width=w, line=13, group=g)


def scrubber(b):
    x, y, w, h = 712, 1796, 680, 120
    g = "S10-scrubber"
    b.rect(x, y, w, h, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g, surface="S10", name="02/scrubber/panel")
    b.text(x + 16, y + 12, "TIME-TRAVEL SCRUBBER", 8.5, "Consolas", "Regular",
           C["ink700"], width=300, line=12, tracking=8, group=g)
    b.text(x + 16, y + 30, "09:41:07  ·  routing state 12 of 38", 12, "Consolas",
           "Regular", C["ink900"], width=400, line=16, group=g)
    b.rect(x + 16, y + 62, 600, 4, fill=C["recess"], r=999, group=g,
           name="02/scrubber/track")
    for i in range(38):
        if i % 6 == 0:
            b.rect(x + 16 + i * 15.7, y + 56, 1, 12, fill=C["honey500"], group=g,
                   name="02/scrubber/tick-%d" % i)
    px = x + 16 + 11 * 15.7
    b.rect(px - 1, y + 52, 3, 22, fill=C["honey700"], r=1, group=g,
           name="02/scrubber/playhead")
    b.dot(px - 3, y + 51, 9, "circle", C["honey700"], group=g,
          name="02/scrubber/playhead-dot")
    b.text(x + 16, y + 82, "REPLAY · READ-ONLY — scrubbing never mutates the live "
                            "graph (UI-R-36)", 8.5, "Consolas", "Regular",
           C["ink700"], width=440, line=12, group=g)
    b.rect(x + 512, y + 78, 152, 24, fill=C["honey100"], r=6, group=g,
           name="02/scrubber/replay-chip")
    b.text(x + 520, y + 83, "HISTORY_REVISION_SET", 8, "Consolas", "Regular",
           C["honey800"], width=140, line=12, align="center", group=g)
    b.text(x, y + h + 8, "S10 · TIME-TRAVEL SCRUBBER · PDF p7:246-247 · MP-14",
           9, "Consolas", "Regular", C["ink500"], width=w, line=13, group=g)


def file_tree(b):
    x, y, w, h = 712, 1936, 320, 280
    g = "S22-tree"
    b.rect(x, y, w, h, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g, surface="S22", name="02/tree/panel")
    b.text(x + 14, y + 12, "PROJECT FILES · WORKSPACE-JAILED", 8.5, "Consolas",
           "Regular", C["ink700"], width=280, line=12, tracking=8, group=g)
    files = [("router.ts", 0, False), ("schema.prisma", 1, True),
             ("agent.spec.ts", 1, False), ("main.tsx", 0, False),
             ("beam.rs", 1, False)]
    for i, (name, depth, active) in enumerate(files):
        fy = y + 36 + i * 28
        if active:
            b.rect(x + 10, fy - 2, 300, 24, fill=C["honey100"], r=6, group=g,
                   name="02/tree/active")
        b.text(x + 16 + depth * 18, fy, "▸" if depth == 0 else "·", 9, "Consolas",
               "Regular", C["ink500"], width=12, line=13, group=g)
        b.text(x + 30 + depth * 18, fy, name, 10.5, "Consolas", "Regular",
               C["ink900"] if active else C["ink700"], width=250, line=14, group=g)
    b.text(x + 14, y + 186, "file list API · open file read jailed to the workspace; "
                            "traversal refused outside root", 8.5, "Consolas",
           "Regular", C["ink500"], width=290, line=12, group=g)
    b.text(x + 14, y + 216, "5 files shown are the board's printed tree (PDF p8:291-295)",
           8, "Consolas", "Regular", C["ink500"], width=290, line=11, group=g)
    b.text(x + 14, y + 240, "click = open in the code pane · no write from the tree",
           8, "Consolas", "Regular", C["ink500"], width=290, line=11, group=g)
    b.text(x, y + h + 8, "S22 · FILE / CODE HIERARCHY · PDF p8:291-295", 9,
           "Consolas", "Regular", C["ink500"], width=w, line=13, group=g)


def surface_index(b):
    y = 2262
    b.section(48, y, 1344, "surface index · S01-S28 wiring",
              note="UI-WIRING-GUIDE §1 · UI-COMMANDS-v1")
    short = {
        "S01": ("Canvas + honeycomb bg", "viewport state"),
        "S02": ("HyperAgent chat core", "CHAT_PIN"),
        "S03": ("Composer", "SUBMIT_PROMPT"),
        "S04": ("Spawn sub-hive", "SPAWN_SUB_HIVE"),
        "S05": ("Re-route", "REWIRE_TASK"),
        "S06": ("Cancel", "CANCEL_RUN"),
        "S07": ("Kill worker", "KILL_WORKER"),
        "S08": ("Tabs ×4", "view route state"),
        "S09": ("HUD throughput", "RUN_METRICS push"),
        "S10": ("Time-travel scrubber", "HISTORY_REVISION_SET"),
        "S11": ("Nav rail 72px", "view route state"),
        "S12": ("Library panel 260px", "TEMPLATE_INSERT"),
        "S13": ("Mini-map 260×170", "client-only"),
        "S14": ("Refusal toast", "ERROR frame"),
        "S15": ("SuperAgent card", "node render"),
        "S16": ("WorkerAgent card", "metrics push"),
        "S17": ("ToolAgent card", "tool result"),
        "S18": ("Broken-route card", "Error state"),
        "S19": ("Agent chips + dots", "enum state"),
        "S20": ("Zoom + shortcut", "VIEWPORT_SET"),
        "S21": ("Settings / key", "settings read"),
        "S22": ("File tree", "file list API"),
        "S23": ("Brand lockups", "static assets"),
        "S24": ("Elevation tiers", "CSS tokens"),
        "S25": ("Diff review (no board)", "DIFF_* gated"),
        "S26": ("Empty/loading/offline", "UI-ADD-04"),
        "S27": ("Streamed reply", "REPLY_DELTA"),
        "S28": ("Project identity", "string table"),
    }
    cw, ch, gap = 184, 44, 8
    for i, sid in enumerate(SURFACES.keys()):
        col, row = i % 7, i // 7
        cx = 48 + col * (cw + gap)
        cy = y + 26 + row * (ch + gap)
        b.rect(cx, cy, cw, ch, fill=C["paper"], r=8, stroke=C["hairline"], sw=1,
               group="02-surface-index", name="02/index/%s" % sid)
        b.text(cx + 8, cy + 5, sid, 8.5, "Consolas", "Regular", C["ink700"],
               width=40, line=11, group="02-surface-index")
        nm, wiring = short[sid]
        b.text(cx + 8, cy + 15, nm, 8.8, "Inter", "SemiBold", C["ink900"],
               width=168, line=12, group="02-surface-index")
        b.text(cx + 8, cy + 29, wiring, 7.5, "Consolas", "Regular", C["ink500"],
               width=168, line=10, group="02-surface-index")
    b.text(48, y + 26 + 4 * (ch + gap) + 4,
           "26 control rows = 22 surfaces + 4 folded labels (K-16); every mutating "
           "surface shows ACK or a named refusal before UI state changes.",
           8.5, "Consolas", "Regular", C["ink500"], width=1344, line=12,
           group="02-surface-index")


def build_components(frame_id):
    b = Board("02", "Components", frame_id, W, H)
    b.rect(0, 0, W, H, fill=C["canvas"], name="02/board-bg")

    # header ---------------------------------------------------------------
    b.eyebrow(48, 44, "05 / COMPONENTS  ·  4 NODE TYPES + 1 ERROR STATE  ·  "
                      "4 BUTTONS × 5 STATES  ·  PDF p6:170-230, p7:232-273",
              group="02-header")
    b.text(48, 64, "Four node types. One shared anatomy.", 56, "Playfair Display",
           "SemiBold", C["ink900"], width=1100, line=60, tracking=-2,
           name="02/title", group="02-header")
    b.body_text(48, 132,
                "Colour is reserved for state — honey = healthy, crimson = broken route — "
                "so a swarm can be read from across the canvas. Vocabulary: 4 node types + "
                "1 error state (CF-UI-02).", 960, size=17, line=28, group="02-header")
    b.eyebrow(1092, 44, "HIVEROUTER  ·  COMPONENTS  /  V1.0", color=C["ink700"],
              width=300, align="right", group="02-header")
    b.eyebrow(1092, 62, "FIGCRAFT NATIVE  ·  1440 × 2580", color=C["ink500"],
              width=300, align="right", group="02-header")

    # band 1: node types ----------------------------------------------------
    b.section(48, 240, 1344, "node types · 4 + 1 error state", note="PDF p6:170-230")
    super_card(b)
    worker_card(b)
    tool_card(b)
    error_card(b)

    # band 2: anatomy + chips ----------------------------------------------
    anatomy_band(b)

    # band 3: chat / composer / toast / tabs+hud+zoom -----------------------
    b.section(48, 906, 420, "hyperagent · hive chat core", note="PDF p6:175-184")
    chat_core(b)
    b.section(492, 906, 440, "composer + refusal toast",
              note="PDF p7:235-237, 269-273")
    composer(b)
    toast(b)
    b.section(956, 906, 436, "tabs · hud · zoom", note="PDF p7:241-245,304")
    tabs_hud_zoom(b)

    # band 4: buttons + settings -------------------------------------------
    buttons_matrix(b)
    settings_and_states(b)

    # band 5: chrome --------------------------------------------------------
    b.section(48, 1770, 1344, "chrome · rail / library / minimap / tree / scrubber",
              note="PDF p7:232-268, p8:291-295")
    nav_rail(b)
    library_panel(b)
    minimap(b)
    scrubber(b)
    file_tree(b)

    # band 6: surface index -------------------------------------------------
    surface_index(b)

    # footer ----------------------------------------------------------------
    b.line(48, 2530, 1344, C["hairline"], 1, name="02/footer-rule")
    b.text(48, 2538, "HIVEROUTER  ·  COMPONENTS  /  V1.0  ·  02 COMPONENTS  ·  "
                     "FIGCRAFT NATIVE", 9, "Consolas", "Regular", C["ink500"],
           width=900, line=13, tracking=8, name="02/footer")
    b.text(1192, 2538, "PAGE 3 / 4  ·  PDF p6-p8", 9, "Consolas", "Regular",
           C["ink500"], width=200, line=13, align="right", name="02/footer-no")
    return b
