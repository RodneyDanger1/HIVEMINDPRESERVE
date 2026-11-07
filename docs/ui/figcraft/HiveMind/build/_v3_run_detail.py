import io
import json
import os
import shutil

HERE = os.path.dirname(os.path.abspath(__file__))
HM = os.path.dirname(HERE)
# baseline for the preservation check: current 12-board manifest
prev = os.path.join(HM, "HiveMind-ASSETS.json")
if os.path.isfile(prev):
    shutil.copyfile(prev, os.path.join(HERE, "prev-assets.json"))

path = os.path.join(HERE, "build_hive_style.py")
text = io.open(path, encoding="utf-8").read()

NEW = r'''
# --------------------------------------------------------------------------
# BOARD 12 -- Run Detail & Files (surfaces without a board; SAMPLE/READ only)
# --------------------------------------------------------------------------
def board_run_detail(f):
    W, H = 1440, 900
    board_header(f, "12", "RUN DETAIL & FILES", "Run detail & files",
                 "Files, tools, run detail, ledger, model/usage and settings.")
    close_space(f, W, H, "RUN DETAIL / 32PX ARGUMENT")

    def card(x, y, w, h, title, tag):
        rect(f, x, y, w, h, fill=PANEL, radius=14, stroke=HAIR, sw=1,
             shadow=SH_E1)
        text(f, title, x + 20, y + 18, 10, MONO, "Regular", INK500, ls="10%")
        tw = 16 + len(tag) * 6.2
        rect(f, x + w - 20 - tw, y + 12, tw, 18, fill=RECESS, radius=999)
        text(f, tag, x + w - 20 - tw + 8, y + 15, 7.5, MONO, "SemiBold",
             INK700, ls="6%")

    def pill(x, y, label, color=INK700, fill=RECESS, w=None):
        w = w or (16 + len(label) * 5.6)
        rect(f, x, y, w, 18, fill=fill, radius=999)
        text(f, label, x + 6, y + 4, 8, MONO, "SemiBold", color, ls="4%")

    # ---------------------------------------------------- DATA & FILES
    card(72, 200, 560, 300, "DATA & FILES", "READ-ONLY")
    text(f, "WORKSPACE / workspace-jailed", 92, 250, 9, MONO, "Regular",
         INK500, ls="4%")
    files = ["src/", "  components/", "    Canvas.tsx", "  lib/routes.ts",
             "prisma/schema.prisma"]
    for i, name in enumerate(files):
        y = 272 + i * 26
        if i == 2:
            rect(f, 88, y - 4, 224, 24, fill=H100, radius=6)
        dot(f, 96, y + 6, 3, H400 if i == 2 else HAIR)
        text(f, name, 108, y, 9, MONO, "Regular",
             INK800 if i == 2 else INK600, width=190)
        text(f, "READ" if i == 2 else "\u2014", 320, y, 8, MONO, "Regular",
             OK if i == 2 else INK500, width=44, align="right")
    rect(f, 360, 248, 252, 176, fill=INK800, radius=10)
    for i, c in enumerate([ERR, AMBER, OK]):
        dot(f, 376 + i * 12, 264, 3.4, c)
    text(f, "# FILE_READ \u00b7 read-only\n# no file content invented\n"
            "select a file from the tree\n\nthe preview is an honest\n"
            "empty state until FILE_READ\nreturns the selected path.",
         376, 286, 8.5, MONO, "Regular", "#F1E9CF", width=220, lh="150%")
    text(f, "FILE_TREE_LIST / FILE_READ \u00b7 never a write path", 92, 470, 8.5,
         MONO, "Regular", INK500, ls="3%")
    pill(472, 462, "SAMPLE", INK700, H100, 54)

    # ---------------------------------------------------- TOOL ACTIVITY
    card(72, 510, 560, 340, "TOOL ACTIVITY", "APPEND-ONLY")
    text(f, "TOOL", 92, 552, 8.5, MONO, "Regular", INK500, ls="6%")
    text(f, "OUTCOME", 300, 552, 8.5, MONO, "Regular", INK500, ls="6%")
    text(f, "HASH / LEDGER", 430, 552, 8.5, MONO, "Regular", INK500, ls="6%")
    rule(f, 92, 568, 520, HAIR, 1)
    rows = [
        ("\u2014", "\u2014", "\u2014", "AWAITING RUN"),
        ("\u2014", "\u2014", "\u2014", "AWAITING RUN"),
        ("\u2014", "\u2014", "\u2014", "AWAITING RUN"),
    ]
    for i, (tool, outcome, digest, state) in enumerate(rows):
        y = 584 + i * 30
        text(f, tool, 92, y, 9.5, MONO, "Regular", INK600, width=190)
        text(f, outcome, 300, y, 9.5, MONO, "Regular", INK600, width=120)
        text(f, digest, 430, y, 9.5, MONO, "Regular", INK600, width=180)
        pill(456, y - 5, "SAMPLE", INK500, RECESS, 52)
        rule(f, 92, y + 20, 520, HAIR, 1)
    text(f, "No tool calls yet \u2014 submit a prompt to create the first run.",
         92, 690, 10.5, UI, "Regular", INK700, width=500)
    text(f, "no fabricated calls \u00b7 every row is a real TOOL_ACTIVITY entry "
            "when a run exists", 92, 714, 8.5, MONO, "Regular", INK500,
         width=520, lh="135%")
    legend = [("SUCCEEDED", OK, PANEL, HAIR), ("FAILED", ERR, PANEL, HAIR),
              ("REFUSED", AMBER, PANEL, HAIR)]
    lx = 92
    for label, color, fill, stroke in legend:
        pill(lx, 784, label, color, fill, 16 + len(label) * 5.6)
        lx += 16 + len(label) * 5.6 + 12
    text(f, "outcome chips use the existing state palette; no new swatches",
         92, 814, 8, MONO, "Regular", INK500)

    # ---------------------------------------------------- RUN DETAIL
    card(660, 200, 360, 420, "RUN DETAIL", "SAMPLE")
    text(f, "run_id", 680, 248, 8.5, MONO, "Regular", INK500, ls="6%")
    text(f, "run_01H\u2026 (SAMPLE)", 680, 262, 11, MONO, "Regular", INK800)
    pill(920, 250, "RUNNING", INK700, H100, 82)
    meta = [("state", "running"), ("started", "\u2014"),
            ("updated", "\u2014"), ("revision", "\u2014")]
    for i, (key, value) in enumerate(meta):
        x = 680 + (i % 2) * 170
        y = 292 + (i // 2) * 38
        text(f, key, x, y, 8.5, MONO, "Regular", INK500, ls="4%")
        text(f, value, x, y + 13, 10.5, MONO, "Regular", INK800)
    rule(f, 680, 358, 320, HAIR, 1)
    text(f, "STAGES", 680, 372, 8.5, MONO, "Regular", INK500, ls="6%")
    stages = [("Planning", "completed"), ("Decompose", "completed"),
              ("Execute", "running"), ("Verify", "queued")]
    for i, (name, state) in enumerate(stages):
        y = 392 + i * 28
        dot(f, 686, y + 6, 4, OK if state == "completed" else
            (AMBER if state == "running" else HAIR))
        text(f, name, 700, y, 10, UI, "Medium", INK800)
        text(f, state, 860, y, 8.5, MONO, "Regular", INK500, width=80,
             align="right")
    text(f, "sample stage timeline; live stages come from the run frame",
         680, 512, 8.5, MONO, "Regular", INK500, width=320, lh="135%")
    rect(f, 680, 540, 320, 8, fill=HAIR, radius=999)
    rect(f, 680, 540, 132, 8, fill=H400, radius=999)
    text(f, "SAMPLE progress \u00b7 never fabricated for a live run", 680, 556,
         8.5, MONO, "Regular", INK500, width=320)
    text(f, "board 07 stays the authoritative spatial canvas; this is a "
            "run-detail surface.", 680, 584, 8.5, UI, "Regular", INK600,
         width=320, lh="135%")

    # ---------------------------------------------------- FAILURE / REFUSAL LEDGER
    card(660, 626, 360, 224, "FAILURE / REFUSAL LEDGER", "SAMPLE")
    ledger = [("E_APPROVAL_TIMEOUT", "retryable", "approval timed out"),
              ("E_KILL_DENIED", "\u2014", "worker cannot be killed"),
              ("E_RUN_TERMINAL", "\u2014", "run already terminal")]
    for i, (code, retry, msg) in enumerate(ledger):
        y = 672 + i * 44
        text(f, code, 680, y, 9.5, MONO, "SemiBold", ERR, width=200)
        text(f, msg, 680, y + 14, 8.5, MONO, "Regular", INK500, width=220)
        pill(880, y - 1, retry if retry != "\u2014" else "NOT RETRYABLE",
             INK700 if retry != "\u2014" else INK500, RECESS,
             82 if retry != "\u2014" else 110)
        rule(f, 680, y + 32, 320, HAIR, 1)
    text(f, "named codes only \u00b7 append-only \u00b7 no silent drops",
         680, 818, 8, MONO, "Regular", INK500, ls="2%")

    # ---------------------------------------------------- MODEL STATUS & USAGE
    card(1040, 200, 328, 320, "MODEL STATUS & USAGE", "SAMPLE")
    status = [("provider", "deepseek"), ("model", "deepseek-chat"),
              ("key status", "configured"), ("server health", "healthy")]
    for i, (key, value) in enumerate(status):
        y = 248 + i * 26
        text(f, key, 1060, y, 8.5, MONO, "Regular", INK500, width=100)
        text(f, value, 1160, y, 9.5, MONO, "Regular", INK800, width=180)
    rule(f, 1060, 358, 288, HAIR, 1)
    text(f, "USAGE / SIX COUNTERS", 1060, 372, 8.5, MONO, "Regular", INK500,
         ls="6%")
    counters = [("prompt", 0), ("completion", 1), ("total", 2),
                ("cache hit", 3), ("cache miss", 4), ("reasoning", 5)]
    for name, idx in counters:
        col = idx % 2
        row = idx // 2
        x = 1060 + col * 148
        y = 396 + row * 38
        text(f, name, x, y, 8.5, MONO, "Regular", INK500, ls="2%")
        text(f, "\u2014", x, y + 14, 13, UI, "SemiBold", INK800)
    text(f, "values arrive from RUN_METRICS; none are fabricated here.",
         1060, 514, 8, MONO, "Regular", INK500, width=288)

    # ---------------------------------------------------- SETTINGS
    card(1040, 530, 328, 320, "SETTINGS", "LOCAL")
    text(f, "Provider key", 1060, 578, 8.5, MONO, "Regular", INK500)
    dot(f, 1072, 600, 4, OK)
    text(f, "present \u00b7 configured", 1084, 594, 10, UI, "Medium", INK800)
    text(f, "missing path \u00b7 E_KEY_MISSING", 1060, 618, 8.5, MONO,
         "Regular", INK500)
    rule(f, 1060, 642, 288, HAIR, 1)
    text(f, "Reduced motion", 1060, 662, 10, UI, "Medium", INK900)
    rect(f, 1300, 660, 40, 20, fill=H400, radius=999)
    ellipse(f, 1324, 663, 14, 14, fill=PANEL)
    text(f, "PROPOSED \u00b7 stops ambient loops", 1060, 682, 8, MONO,
         "Regular", INK500)
    rule(f, 1060, 706, 288, HAIR, 1)
    text(f, "Code-diff control", 1060, 726, 10, UI, "Medium", INK900)
    pill(1268, 720, "DECISION-GATED", INK700, RECESS, 92)
    text(f, "no diff control; DIFF_* is not frozen", 1060, 746, 8, MONO,
         "Regular", INK500, width=288)
    rule(f, 1060, 770, 288, HAIR, 1)
    text(f, "Settings apply to the local session only; key material never "
            "lives in the design.", 1060, 790, 8.5, UI, "Regular", INK600,
         width=288, lh="135%")
    text(f, "AUTHORITATIVE SURFACES \u00b7 NO DATA INVENTED \u00b7 SAMPLE/READ "
            "LABELLED", 72, 860, 9, MONO, "Regular", INK500, ls="3%")
    text(f, "run detail & files \u00b7 1440 \u00d7 900 \u00b7 existing tokens only",
         1368, 860, 8.5, MONO, "Regular", INK500, width=320, align="right")
'''

MARK = '# --------------------------------------------------------------------------\n# Phase 1\n# --------------------------------------------------------------------------'
assert MARK in text
text = text.replace(MARK, NEW + '\n\n' + MARK)

old_tail = ('    frame("11 · Motion & Live", 0, 5580, 1440, 900)\n'
            '    board_motion(FRAME_IDS["11 · Motion & Live"])\n')
new_tail = old_tail + (
    '    frame("12 · Run Detail & Files", 0, 7460, 1440, 900)\n'
    '    board_run_detail(FRAME_IDS["12 · Run Detail & Files"])\n')
assert old_tail in text
text = text.replace(old_tail, new_tail)

assert '12 artboards' in text
text = text.replace('12 artboards', '13 artboards')

old_order = ('             "09 · HMAS Governance", "10 · States & Refusals",\n'
             '             "11 · Motion & Live"]')
new_order = ('             "09 · HMAS Governance", "10 · States & Refusals",\n'
             '             "11 · Motion & Live", "12 · Run Detail & Files"]')
assert old_order in text
text = text.replace(old_order, new_order)

io.open(path, 'w', encoding='utf-8', newline='\n').write(text)
print('v3 builder patched: board 12 Run Detail & Files, 13 artboards')
