import io
import os

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, 'build_hive_style.py')
text = io.open(SRC, encoding='utf-8').read()

# ---- provenance / output paths (the HiveMind copy, never the HiveRouter repo)
old_paths = 'EXPORTS = os.path.join(HERE, "exports")'
new_paths = ('ROOT = os.path.dirname(HERE)\n'
             'EXPORTS = os.path.join(ROOT, "exports")')
assert old_paths in text
text = text.replace(old_paths, new_paths)

old_final = 'final = os.path.join(HERE, "HiveRouter-Style-System.figcraft").replace("\\\\", "/")'
new_final = ('final = os.path.join(ROOT, "HiveMind-Style-System.figcraft").replace("\\\\", "/")')
assert old_final in text, 'final path not found'
text = text.replace(old_final, new_final)

# ---- user-facing rename (evidence copies keep the original names)
text = text.replace('HiveRouter', 'HiveMind')
text = text.replace('HIVEROUTER', 'HIVEMIND')
text = text.replace('hiverouter', 'hivemind')

# ---- cover copy: the set now has 10 artboards and a generic build claim
assert '8 artboards' in text
text = text.replace('8 artboards', '10 artboards')
old_cmdline = ('text(f, "v1.0.0  ·  built with figcraft-cli  ·  1,045 commands, zero GUI", 72, 816,\n'
               '         10.5, MONO, "Regular", INK500, ls="4%")')
new_cmdline = ('text(f, "v1.0.0  ·  built with figcraft-cli  ·  deterministic, zero GUI", 72, 816,\n'
               '         10.5, MONO, "Regular", INK500, ls="4%")')
assert old_cmdline in text, 'cover command line not found'
text = text.replace(old_cmdline, new_cmdline)

NEW_BOARDS = r'''
# --------------------------------------------------------------------------
# BOARD 08 -- HMAS (hierarchy, profiles/import, memory, mailbox, projects,
#                  usage, cross-branch message approval)
# --------------------------------------------------------------------------
def board_hmas(f):
    W, H = 1440, 1100
    board_header(f, "08", "HMAS", "Hierarchy & mail",
                 "One mesh, one mailbox: the HyperAgent tree, importable profiles, "
                 "memory, projects, usage and cross-branch message approval share "
                 "the same honey-ink language as the canvas.")
    close_space(f, W, H, "HMAS / 32PX ARGUMENT")

    # ---- hierarchy / workspace-jailed tree
    rect(f, 72, 220, 520, 620, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "HIERARCHY / WORKSPACE-JAILED", 96, 242, 10, MONO, "Regular",
         INK500, ls="12%")
    rect(f, 96, 270, 472, 64, fill=H100, radius=12, stroke=BORDER_GOLD, sw=1.2)
    hexmark(f, 110, 288, 32)
    text(f, "HyperAgent", 150, 284, 14, UI, "SemiBold", INK900)
    text(f, "root · 4 superagents · healthy", 150, 306, 9, MONO, "Regular",
         INK500, ls="4%")
    rows = [
        (120, 350, 448, "Backend Architecture", "SUPERAGENT · 3 workers · 12.4k ctx", H400),
        (144, 416, 424, "tailwind-builder", "WORKER · react/vite · 55%", H600),
        (144, 456, 424, "db-schema", "WORKER · E0421 · retry 2/3", ERR),
        (120, 500, 448, "Frontend Architecture", "SUPERAGENT · 2 workers · 8.1k ctx", H400),
        (144, 566, 424, "agent.spec.ts", "WORKER · 0.78", H600),
        (144, 606, 424, "main.tsx", "WORKER · queued", HAIR),
    ]
    for x, y, w, name, meta, dotc in rows:
        rect(f, x, y, w, 34, fill=PANEL, radius=8, stroke=HAIR, sw=1)
        dot(f, x + 16, y + 17, 4, dotc)
        text(f, name, x + 28, y + 8, 11.5, UI, "Medium", INK900)
        text(f, meta, x + 166, y + 11, 8.5, MONO, "Regular", INK500, ls="4%")
    text(f, "nesting: HyperAgent > SuperAgent > WorkerAgent > ToolAgent",
         96, 664, 9.5, MONO, "Regular", INK500, ls="4%")
    rule(f, 96, 700, 472, HAIR, 1)
    text(f, "re-parent a worker only while it is not streaming;", 96, 716, 10,
         UI, "Regular", INK600)
    text(f, "kill and rewire stay confirmation-first.", 96, 734, 10, UI,
         "Regular", INK600)
    button(f, 96, 786, 216, 38, "Spawn sub-hive", "primary")
    button(f, 324, 786, 216, 38, "Kill worker", "danger")

    # ---- profiles / import
    rect(f, 620, 220, 420, 170, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "PROFILES · IMPORT", 644, 242, 10, MONO, "Regular", INK500,
         ls="12%")
    text(f, "Default profile", 644, 272, 12.5, UI, "Medium", INK900)
    text(f, "active · 2 gates", 644, 292, 9, MONO, "Regular", INK500)
    text(f, "Strict review", 644, 318, 12.5, UI, "Medium", INK700)
    text(f, "paused", 644, 338, 9, MONO, "Regular", INK500)
    button(f, 852, 266, 168, 36, "Import project", "secondary")
    text(f, "Import accepts a folder or a .figcraft manifest; nothing executes "
            "at import time.", 644, 360, 9, MONO, "Regular", INK500, width=372,
         lh="140%")

    superagent_card(f, 620, 410, 420, 150)
    worker_card(f, 620, 576, 320, 96)
    tool_card(f, 620, 688, 420, 240)

    # ---- right column: memory, mailbox, projects, usage, message approval
    RX, RW = 1080, 288
    rect(f, RX, 220, RW, 160, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "MEMORY", RX + 20, 240, 10, MONO, "Regular", INK500, ls="12%")
    text(f, "Run summary · 12.4k ctx", RX + 20, 268, 10, UI, "Regular", INK800)
    text(f, "Decision log · 3 rows", RX + 20, 288, 10, UI, "Regular", INK800)
    text(f, "Refusal ledger · 5 rows", RX + 20, 308, 10, UI, "Regular", INK800)
    button(f, RX + 20, 334, 248, 32, "Clear memory", "danger")

    rect(f, RX, 400, RW, 160, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "MAILBOX · 2 UNREAD", RX + 20, 420, 10, MONO, "Regular", INK500,
         ls="12%")
    dot(f, RX + 22, 451, 5, AMBER)
    text(f, "Backend → Frontend", RX + 36, 444, 11, UI, "Medium", INK900)
    text(f, "schema.prisma v3 ready", RX + 36, 462, 8.5, MONO, "Regular", INK500)
    dot(f, RX + 22, 499, 5, H400)
    text(f, "Frontend → Backend", RX + 36, 492, 11, UI, "Medium", INK900)
    text(f, "API drift report", RX + 36, 510, 8.5, MONO, "Regular", INK500)
    text(f, "mailbox is read-only · delivery is a branch event", RX + 20, 534,
         8.5, MONO, "Regular", INK500, width=248, lh="140%")

    rect(f, RX, 576, RW, 140, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "PROJECTS", RX + 20, 596, 10, MONO, "Regular", INK500, ls="12%")
    rect(f, RX + 20, 620, 248, 30, fill=H100, radius=8, stroke=BORDER_GOLD, sw=1)
    text(f, "HiveMind AI · active", RX + 32, 628, 11, UI, "Medium", INK900)
    text(f, "sandbox · idle", RX + 32, 662, 11, UI, "Regular", INK700)
    text(f, "project_id · localhost only", RX + 20, 686, 8.5, MONO, "Regular",
         INK500)

    rect(f, RX, 732, RW, 128, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "USAGE", RX + 20, 752, 10, MONO, "Regular", INK500, ls="12%")
    text(f, "4 SUPERAGENTS", RX + 20, 778, 12, UI, "SemiBold", INK900)
    text(f, "12 WORKERS", RX + 150, 778, 12, UI, "SemiBold", INK900)
    text(f, "98.2% GREEN", RX + 20, 800, 12, UI, "SemiBold", OK)
    text(f, "38 calls · 12.4k ctx · derived counters", RX + 20, 826, 8.5,
         MONO, "Regular", INK500)

    rect(f, RX, 876, RW, 180, fill=PANEL, radius=14, stroke=BORDER_GOLD,
         sw=1.5, shadow=SH_E1)
    text(f, "MESSAGE APPROVAL", RX + 20, 896, 10, MONO, "Regular", INK500,
         ls="12%")
    dot(f, RX + 250, 899, 5, AMBER)
    text(f, "Backend Architecture → Frontend Architecture", RX + 20, 922,
         10.5, UI, "Medium", INK900)
    text(f, "schema.prisma v3 ready to merge", RX + 20, 942, 9, MONO,
         "Regular", INK500)
    text(f, "args_sha256 · 9f86d081884c…", RX + 20, 962, 8.5, MONO, "Regular",
         INK500)
    button(f, RX + 20, 992, 112, 34, "Deny", "danger")
    button(f, RX + 148, 992, 120, 34, "Approve", "primary")
    text(f, "APPROVAL_RESOLVE · nonce · single-use", RX + 20, 1038, 8, MONO,
         "Regular", INK500)

    text(f, "HMAS · hierarchy, memory, mailbox, projects, usage, message approval",
         72, 1068, 9, MONO, "Regular", "#A99B7C", ls="6%")


# --------------------------------------------------------------------------
# BOARD 09 -- States & refusals
# --------------------------------------------------------------------------
def board_states(f):
    W, H = 1440, 1100
    board_header(f, "09", "STATES & REFUSALS", "Named states, not silent failures",
                 "Every failure is a named refusal; every wait has a state; "
                 "nothing is a silent drop. Replay is read-only and the diff "
                 "surface stays DECISION-GATED until its commands are frozen.")
    close_space(f, W, H, "STATES / 32PX ARGUMENT")

    def state_card(x, y, w, h, label, title):
        rect(f, x, y, w, h, fill=PANEL, radius=14, stroke=HAIR, sw=1,
             shadow=SH_E1)
        text(f, label, x + 20, y + 16, 9.5, MONO, "Regular", INK500, ls="12%")
        text(f, title, x + 20, y + 34, 15, UI, "SemiBold", INK900)

    # 01 named refusal (crimson error card, authoritative anatomy)
    x, y = 72, 220
    state_card(x, y, 624, 250, "S14 / NAMED REFUSAL", "E_APPROVAL_TIMEOUT")
    error_card(f, x + 20, y + 58, 300, 110)
    text(f, "E_APPROVAL_TIMEOUT", x + 340, y + 60, 11.5, MONO, "SemiBold", ERR)
    text(f, "The approval request timed out before a decision arrived.",
         x + 340, y + 82, 10.5, UI, "Regular", INK700, width=260, lh="145%")
    text(f, "how_to_fix: Resolve the request when prompted; the tool call is "
            "refused, never faked.", x + 340, y + 128, 9, MONO, "Regular",
         INK500, width=260, lh="145%")
    text(f, "ERROR frame → toast + log + ledger", x + 20, y + 196, 9, MONO,
         "Regular", INK500, ls="6%")

    # 02 reconnect
    x = 744
    state_card(x, y, 624, 250, "RECONNECT / SOCKET CLOSED", "Reconnecting")
    rect(f, x + 20, y + 58, 584, 54, fill=H100, radius=10)
    dot(f, x + 40, y + 85, 5, AMBER)
    text(f, "Reconnecting to the local server; mutations are blocked until the "
            "socket is open.", x + 54, y + 74, 10.5, UI, "Regular", INK800,
         width=540, lh="140%")
    bee_route(f, x + 40, y + 148, x + 570, y + 148, color=AMBER,
              dashes=[8, 8], bee_at=0.5, bee_size=14)
    text(f, "SERVER_HELLO · RESYNC_REQUIRED · retry with backoff", x + 20,
         y + 176, 9, MONO, "Regular", INK500, ls="4%")
    text(f, "no mutation is queued while disconnected", x + 20, y + 196, 9,
         MONO, "Regular", INK500)

    # 03 empty / loading
    y = 490
    x = 72
    state_card(x, y, 624, 250, "S26 / EMPTY · LOADING", "Empty and loading")
    rect(f, x + 20, y + 58, 286, 154, fill=PANEL_ALT, radius=12, stroke=HAIR,
         sw=1)
    text(f, "Nothing here yet", x + 36, y + 82, 12.5, UI, "SemiBold", INK900)
    text(f, "No agents yet — submit a prompt to create the first run.",
         x + 36, y + 106, 9.5, UI, "Regular", INK600, width=250, lh="145%")
    rect(f, x + 326, y + 58, 278, 154, fill=PANEL_ALT, radius=12, stroke=HAIR,
         sw=1)
    ellipse(f, x + 344, y + 80, 34, 34, fill=None, stroke=H400, sw=3)
    text(f, "Loading", x + 388, y + 88, 12.5, UI, "SemiBold", INK900)
    rect(f, x + 344, y + 132, 240, 10, fill=HAIR, radius=999)
    rect(f, x + 344, y + 152, 188, 10, fill=HAIR, radius=999)
    rect(f, x + 344, y + 172, 214, 10, fill=HAIR, radius=999)
    text(f, "honest empty copy · skeleton bars · no fabricated rows", x + 20,
         y + 226, 9, MONO, "Regular", INK500, ls="4%")

    # 04 kill confirmation
    x = 744
    state_card(x, y, 624, 250, "KILL CONFIRMATION", "Kill worker?")
    rect(f, x + 20, y + 58, 584, 154, fill=RECESS, radius=12)
    rect(f, x + 72, y + 72, 480, 126, fill=PANEL, radius=14, stroke=ERR, sw=1.5)
    text(f, "Kill worker?", x + 96, y + 86, 15, UI, "SemiBold", INK900)
    text(f, "tailwind-builder · WORKER · running", x + 96, y + 108, 9,
         MONO, "Regular", INK500)
    text(f, "The worker cannot be killed from its current state.",
         x + 96, y + 126, 9.5, UI, "Regular", INK800, width=430)
    text(f, "Only running workers can be killed; refresh the worker state.",
         x + 96, y + 144, 8.5, MONO, "Regular", INK500, width=430)
    button(f, x + 96, y + 162, 104, 30, "Cancel", "secondary")
    button(f, x + 212, y + 162, 140, 30, "Kill worker", "danger")
    text(f, "KILL_WORKER · confirmation first · never silent", x + 20,
         y + 226, 9, MONO, "Regular", INK500, ls="4%")

    # 05 replay read-only
    y = 760
    x = 72
    state_card(x, y, 624, 250, "REPLAY / READ-ONLY", "Replay (read-only)")
    rect(f, x + 20, y + 58, 584, 36, fill=INK900, radius=10)
    text(f, "Replay (read-only) · banner stays while the revision is held",
         x + 36, y + 68, 10, MONO, "Regular", H300)
    rect(f, x + 20, y + 108, 584, 90, fill=PANEL_ALT, radius=12, stroke=HAIR,
         sw=1)
    for i, gx in enumerate([x + 80, x + 250, x + 420]):
        rect(f, gx, y + 136, 34, 26, fill=RECESS, radius=6, stroke=INK500, sw=1)
        dot(f, gx + 8, y + 130, 4, INK700)
    bee_route(f, x + 120, y + 149, x + 570, y + 149, color=INK500,
              dashes=[6, 6], bee_at=0.55, bee_size=12)
    text(f, "HISTORY_REVISION_SET → HISTORY_SNAPSHOT · no mutations while in "
            "replay", x + 20, y + 214, 9, MONO, "Regular", INK500, ls="4%")

    # 06 DECISION-GATED diff explanation
    x = 744
    state_card(x, y, 624, 250, "S25 / DECISION-GATED", "Code-diff review")
    rect(f, x + 20, y + 58, 584, 84, fill=PANEL_ALT, radius=10, stroke=HAIR,
         sw=1)
    text(f, "router.ts", x + 36, y + 72, 9.5, MONO, "Regular", INK500)
    text(f, "- const route = legacy();", x + 36, y + 90, 9.5, MONO, "Regular",
         ERR)
    text(f, "+ const route = mesh();", x + 36, y + 108, 9.5, MONO, "Regular",
         OK)
    text(f, "files · 1 modified · +2 / -2", x + 330, y + 72, 9, MONO,
         "Regular", INK500)
    rect(f, x + 20, y + 154, 584, 22, fill=H100, radius=999,
         stroke=BORDER_GOLD, sw=1)
    text(f, "DECISION-GATED · DIFF_* commands are NOT frozen in this contract",
         x + 36, y + 160, 9, MONO, "SemiBold", H800, ls="3%")
    text(f, "Decision required: the code-diff surface is proposal-gated (no PDF "
            "artboard, no frozen DIFF command). It is not built in v1.",
         x + 20, y + 188, 9.5, UI, "Regular", INK700, width=584, lh="145%")
    button(f, x + 20, y + 216, 120, 26, "Propose", "secondary")
    button(f, x + 156, y + 216, 120, 26, "Apply", "secondary")

    text(f, "HMIND · named refusals, reconnect, empty/loading, kill, replay, "
            "decision-gated diff", 72, 1068, 9, MONO, "Regular", "#A99B7C",
         ls="4%")
'''

MARK = '# --------------------------------------------------------------------------\n# Phase 1\n# --------------------------------------------------------------------------'
assert MARK in text
text = text.replace(MARK, NEW_BOARDS + '\n\n' + MARK)

# ---- phase 1: add the two new artboards after the eight originals
old_frames_tail = ('    frame("07 · Spatial Canvas", 1520, 2900, 1440, 900)\n')
new_frames_tail = (old_frames_tail +
                   '    frame("08 · HMAS", 0, 4440, 1440, 1100)\n'
                   '    frame("09 · States & Refusals", 1520, 4440, 1440, 1100)\n')
assert old_frames_tail in text
text = text.replace(old_frames_tail, new_frames_tail)

old_hero_call = '    board_hero(FRAME_IDS["07 · Spatial Canvas"])\n'
new_hero_call = (old_hero_call +
                 '    board_hmas(FRAME_IDS["08 · HMAS"])\n'
                 '    board_states(FRAME_IDS["09 · States & Refusals"])\n')
assert old_hero_call in text
text = text.replace(old_hero_call, new_hero_call)

# ---- phase 2: export all ten boards (and keep the brand-lockup SVG export)
old_order = ('    order = ["00 · Cover", "01 · Colour", "02 · Typography", "03 · Space & Depth",\n'
             '             "04 · Brand Lockups", "05 · Agent Cards", "06 · Controls & HUD",\n'
             '             "07 · Spatial Canvas"]')
new_order = ('    order = ["00 · Cover", "01 · Colour", "02 · Typography", "03 · Space & Depth",\n'
             '             "04 · Brand Lockups", "05 · Agent Cards", "06 · Controls & HUD",\n'
             '             "07 · Spatial Canvas", "08 · HMAS", "09 · States & Refusals"]')
assert old_order in text
text = text.replace(old_order, new_order)

old_names = ('        names = ["00 · Cover", "01 · Colour", "02 · Typography",\n'
             '                 "03 · Space & Depth", "04 · Brand Lockups", "05 · Agent Cards",\n'
             '                 "06 · Controls & HUD", "07 · Spatial Canvas"]')
new_names = ('        names = ["00 · Cover", "01 · Colour", "02 · Typography",\n'
             '                 "03 · Space & Depth", "04 · Brand Lockups", "05 · Agent Cards",\n'
             '                 "06 · Controls & HUD", "07 · Spatial Canvas",\n'
             '                 "08 · HMAS", "09 · States & Refusals"]')
assert old_names in text, 'phase2 names block not found'
text = text.replace(old_names, new_names)

io.open(SRC, 'w', encoding='utf-8', newline='\n').write(text)
print('patched', SRC, len(text), 'chars')
