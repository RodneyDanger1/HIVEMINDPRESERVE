import io

path = 'build_hive_style.py'
text = io.open(path, encoding='utf-8').read()

START = '# --------------------------------------------------------------------------\n# BOARD 08'
END = '# --------------------------------------------------------------------------\n# Phase 1'
assert START in text and END in text
start = text.index(START)
end = text.index(END)
NEW = r'''# --------------------------------------------------------------------------
# BOARD 08 -- HMAS / hierarchy & topology
# --------------------------------------------------------------------------
def board_hmas_topology(f):
    W, H = 1440, 900
    board_header(f, "08", "HMAS / HIERARCHY", "Hierarchy & topology",
                 "The HyperAgent tree and the live swarm graph are the same "
                 "object: re-parent a worker only while it is not streaming; "
                 "every edge is a bee line, never decoration.")
    close_space(f, W, H, "HMAS / 32PX ARGUMENT")

    rect(f, 72, 220, 520, 600, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "HIERARCHY / WORKSPACE-JAILED", 96, 242, 10, MONO, "Regular",
         INK500, ls="12%")
    rect(f, 96, 270, 472, 64, fill=H100, radius=12, stroke=BORDER_GOLD, sw=1.2)
    hexmark(f, 110, 288, 32)
    text(f, "HyperAgent", 150, 284, 14, UI, "SemiBold", INK900)
    text(f, "root · 4 superagents · healthy", 150, 306, 9, MONO, "Regular",
         INK500, ls="4%")
    rows = [
        (120, 350, 448, "Backend Architecture", "SUPERAGENT · 3 workers · running", H400),
        (144, 390, 424, "tailwind-builder", "WORKER · react/vite · 55%", H600),
        (144, 430, 424, "db-schema", "WORKER · E0421 · retry 2/3", ERR),
        (120, 470, 448, "Frontend Architecture", "SUPERAGENT · 2 workers · running", H400),
        (144, 510, 424, "agent.spec.ts", "WORKER · completed · 0.78", H600),
        (144, 550, 424, "main.tsx", "WORKER · queued", HAIR),
    ]
    for x, y, w, name, meta, dotc in rows:
        rect(f, x, y, w, 34, fill=PANEL, radius=8, stroke=HAIR, sw=1)
        dot(f, x + 16, y + 17, 4, dotc)
        text(f, name, x + 28, y + 8, 11.5, UI, "Medium", INK900)
        text(f, meta, x + 166, y + 11, 8.5, MONO, "Regular", INK500, ls="4%")
    text(f, "nesting: HyperAgent > SuperAgent > WorkerAgent > ToolAgent",
         96, 602, 9.5, MONO, "Regular", INK500, ls="4%")
    rule(f, 96, 632, 472, HAIR, 1)
    text(f, "re-parent only while not streaming · kill/rewire are "
            "confirmation-first", 96, 648, 9.5, UI, "Regular", INK600,
         width=460, lh="145%")
    button(f, 96, 746, 216, 38, "Spawn sub-hive", "primary")
    button(f, 324, 746, 216, 38, "Kill worker", "danger")

    superagent_card(f, 620, 250, 420, 150)
    worker_card(f, 620, 430, 320, 96)
    tool_card(f, 620, 560, 420, 240)
    text(f, "S15 / S16 / S17 · authoritative card sizes (420x150, 320x96, "
            "420x240)", 620, 812, 9, MONO, "Regular", INK500, ls="4%")

    rect(f, 1080, 220, 288, 380, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "TOPOLOGY / LIVE EDGES", 1100, 242, 10, MONO, "Regular", INK500,
         ls="12%")
    honeycomb_window = honeycomb(f, 1092, 258, 264, 300, R=16, color=HAIR_SOFT,
                                 weight=1)
    topo = {"root": (1224, 310), "back": (1136, 400), "front": (1312, 400),
            "w1": (1116, 500), "w2": (1240, 500)}
    for key, (x0, y0), (x1, y1) in [
            ("root", topo["root"], topo["back"]),
            ("root2", topo["root"], topo["front"]),
            ("back", topo["back"], topo["w1"]),
            ("front", topo["front"], topo["w2"])]:
        bee_route(f, x0, y0, x1, y1, bee_at=0.6, bee_size=11)
    for key, (cx, cy), label in [
            ("root", topo["root"], "HyperAgent"), ("back", topo["back"], "Backend"),
            ("front", topo["front"], "Frontend"), ("w1", topo["w1"], "tailwind"),
            ("w2", topo["w2"], "agent.spec")]:
        dot(f, cx, cy, 7, H400 if key == "root" else H600)
        text(f, label, cx - 36, cy + 10, 8.5, MONO, "Regular", INK600,
             width=72, align="center")
    text(f, "bee faces the data direction", 1100, 566, 9, MONO, "Regular",
         INK500)

    rect(f, 1080, 620, 288, 200, fill=PANEL, radius=14, stroke=HAIR, sw=1,
         shadow=SH_E1)
    text(f, "STATES / SHAPE + TEXT CUE", 1100, 642, 10, MONO, "Regular",
         INK500, ls="12%")
    legend = [("healthy", H400, "circle"), ("running", H600, "circle"),
              ("queued", HAIR, "square"), ("failed", ERR, "triangle"),
              ("awaiting_approval", AMBER, "diamond")]
    for i, (name, color, shape) in enumerate(legend):
        y = 670 + i * 26
        if shape == "circle":
            ellipse(f, 1104, y, 8, 8, fill=color)
        elif shape == "square":
            rect(f, 1104, y, 8, 7, fill=color, radius=2)
        else:
            rect(f, 1104, y, 8, 8, fill=color, radius=2)
        text(f, name, 1122, y - 1, 9.5, UI, "Medium", INK700)
    text(f, "colour is never the only channel", 1100, 798, 9, MONO, "Regular",
         INK500)


# --------------------------------------------------------------------------
# BOARD 09 -- HMAS / governance (profiles, memory, mailbox, projects, usage,
#             cross-branch message approval)
# --------------------------------------------------------------------------
def board_hmas_governance(f):
    W, H = 1440, 900
    board_header(f, "09", "HMAS / GOVERNANCE", "Profiles, memory & mail",
                 "One operator surface for the mesh: importable profiles, a "
                 "clearable memory, the cross-branch mailbox, project identity, "
                 "usage counters and nonce-bound message approval.")
    close_space(f, W, H, "HMAS / 32PX ARGUMENT")

    def card(x, y, title, subtitle=""):
        rect(f, x, y, 420, 260, fill=PANEL, radius=14, stroke=HAIR, sw=1,
             shadow=SH_E1)
        text(f, title, x + 20, y + 20, 10, MONO, "Regular", INK500, ls="12%")
        if subtitle:
            text(f, subtitle, x + 20, y + 38, 9, MONO, "Regular", INK500)
        return y + 52

    # 01 profiles / import
    x, y = 72, 220
    cy = card(x, y, "PROFILES · IMPORT")
    rect(f, x + 20, cy, 380, 30, fill=H100, radius=8, stroke=BORDER_GOLD, sw=1)
    text(f, "Default profile · active · 2 gates", x + 32, cy + 8, 10.5, UI,
         "Medium", INK900)
    text(f, "Strict review · paused", x + 32, cy + 48, 11, UI, "Medium", INK700)
    text(f, "Import a folder or a .figcraft manifest; nothing executes at "
            "import time.", x + 20, cy + 74, 9, MONO, "Regular", INK500,
         width=380, lh="145%")
    button(f, x + 20, y + 204, 380, 36, "Import project", "secondary")

    # 02 memory
    x, y = 540, 220
    cy = card(x, y, "MEMORY")
    text(f, "Run summary · 12.4k ctx", x + 20, cy, 10.5, UI, "Regular", INK800)
    text(f, "Decision log · 3 rows", x + 20, cy + 22, 10.5, UI, "Regular", INK800)
    text(f, "Refusal ledger · 5 rows", x + 20, cy + 44, 10.5, UI, "Regular", INK800)
    text(f, "Clear is confirmation-first; the ledger is append-only.",
         x + 20, cy + 68, 9, MONO, "Regular", INK500, width=380, lh="145%")
    button(f, x + 20, y + 204, 380, 36, "Clear memory", "danger")

    # 03 mailbox
    x, y = 1008, 220
    cy = card(x, y, "MAILBOX · 2 UNREAD")
    dot(f, x + 24, cy + 8, 5, AMBER)
    text(f, "Backend → Frontend", x + 38, cy, 10.5, UI, "Medium", INK900)
    text(f, "schema.prisma v3 ready", x + 38, cy + 18, 8.5, MONO, "Regular", INK500)
    dot(f, x + 24, cy + 52, 5, H400)
    text(f, "Frontend → Backend", x + 38, cy + 44, 10.5, UI, "Medium", INK900)
    text(f, "API drift report", x + 38, cy + 62, 8.5, MONO, "Regular", INK500)
    text(f, "Mailbox is read-only; delivery is a branch event.", x + 20,
         cy + 92, 9, MONO, "Regular", INK500, width=380, lh="145%")

    # 04 projects
    x, y = 72, 540
    cy = card(x, y, "PROJECTS")
    rect(f, x + 20, cy, 380, 34, fill=H100, radius=8, stroke=BORDER_GOLD, sw=1)
    text(f, "HiveMind AI · active", x + 32, cy + 9, 11, UI, "Medium", INK900)
    text(f, "sandbox · idle", x + 32, cy + 50, 11, UI, "Regular", INK700)
    text(f, "project_id · localhost only · PDF:256,275-282", x + 20, cy + 76,
         9, MONO, "Regular", INK500, width=380)

    # 05 usage
    x, y = 540, 540
    cy = card(x, y, "USAGE")
    text(f, "4 SUPERAGENTS", x + 20, cy, 13, UI, "SemiBold", INK900)
    text(f, "12 WORKERS", x + 210, cy, 13, UI, "SemiBold", INK900)
    text(f, "98.2% GREEN", x + 20, cy + 26, 13, UI, "SemiBold", OK)
    text(f, "38 calls · 12.4k ctx · derived counters", x + 20, cy + 54, 9,
         MONO, "Regular", INK500, width=380)
    rect(f, x + 20, cy + 76, 380, 8, fill=HAIR, radius=999)
    rect(f, x + 20, cy + 76, 374, 8, fill=H400, radius=999)

    # 06 cross-branch message approval
    x, y = 1008, 540
    rect(f, x, y, 420, 260, fill=PANEL, radius=14, stroke=BORDER_GOLD, sw=1.5,
         shadow=SH_E1)
    text(f, "MESSAGE APPROVAL", x + 20, y + 20, 10, MONO, "Regular", INK500,
         ls="12%")
    dot(f, x + 386, y + 24, 6, AMBER)
    text(f, "Backend Architecture → Frontend Architecture", x + 20, y + 44,
         10.5, UI, "Medium", INK900, width=380, lh="140%")
    text(f, "schema.prisma v3 ready to merge", x + 20, y + 82, 9, MONO,
         "Regular", INK500)
    text(f, "args_sha256 · 9f86d081884c…", x + 20, y + 100, 9, MONO, "Regular",
         INK500)
    button(f, x + 20, y + 132, 176, 36, "Deny", "danger")
    button(f, x + 212, y + 132, 188, 36, "Approve", "primary")
    text(f, "APPROVAL_RESOLVE · nonce · single-use grant · TTL 120 s",
         x + 20, y + 188, 9, MONO, "Regular", INK500, width=380, lh="145%")
    text(f, "Approve executes no effect by itself; it grants the engine one "
            "consume.", x + 20, y + 214, 9, UI, "Regular", INK600, width=380,
         lh="145%")


# --------------------------------------------------------------------------
# BOARD 10 -- States & refusals
# --------------------------------------------------------------------------
def board_states(f):
    W, H = 1440, 900
    board_header(f, "10", "STATES & REFUSALS", "Named states, not silent failures",
                 "Every failure is a named refusal; every wait has a state; "
                 "nothing is a silent drop. Replay is read-only and the diff "
                 "surface stays DECISION-GATED until its commands are frozen.")
    close_space(f, W, H, "STATES / 32PX ARGUMENT")

    def state_card(x, y, label, title):
        rect(f, x, y, 420, 300, fill=PANEL, radius=14, stroke=HAIR, sw=1,
             shadow=SH_E1)
        text(f, label, x + 20, y + 16, 9.5, MONO, "Regular", INK500, ls="12%")
        text(f, title, x + 20, y + 34, 15, UI, "SemiBold", INK900)
        return y + 60

    # 01 named refusal (crimson error card, authoritative anatomy)
    x, y = 72, 220
    cy = state_card(x, y, "S14 / NAMED REFUSAL", "E_APPROVAL_TIMEOUT")
    error_card(f, x + 20, cy, 300, 110)
    text(f, "E_APPROVAL_TIMEOUT", x + 20, cy + 120, 11, MONO, "SemiBold", ERR)
    text(f, "The approval request timed out before a decision arrived.",
         x + 20, cy + 140, 10.5, UI, "Regular", INK700, width=380, lh="145%")
    text(f, "how_to_fix: Resolve the request when prompted; the tool call is "
            "refused, never faked.", x + 20, cy + 184, 9, MONO, "Regular",
         INK500, width=380, lh="145%")
    text(f, "ERROR frame → toast + log + ledger", x + 20, cy + 232, 9, MONO,
         "Regular", INK500, ls="4%")

    # 02 reconnect
    x = 540
    cy = state_card(x, y, "RECONNECT / SOCKET CLOSED", "Reconnecting")
    rect(f, x + 20, cy, 380, 60, fill=H100, radius=10)
    dot(f, x + 38, cy + 30, 5, AMBER)
    text(f, "Reconnecting to the local server; mutations are blocked until the "
            "socket is open.", x + 52, cy + 18, 10, UI, "Regular", INK800,
         width=330, lh="140%")
    bee_route(f, x + 30, cy + 100, x + 390, cy + 100, color=AMBER,
              dashes=[8, 8], bee_at=0.5, bee_size=14)
    text(f, "SERVER_HELLO · RESYNC_REQUIRED · retry with backoff", x + 20,
         cy + 132, 9, MONO, "Regular", INK500, ls="3%")
    text(f, "no mutation is queued while disconnected", x + 20, cy + 154, 9,
         MONO, "Regular", INK500)
    text(f, "S19 state dot stays amber; text cue is never colour-only.",
         x + 20, cy + 184, 9, UI, "Regular", INK600, width=380, lh="145%")

    # 03 empty / loading
    x = 1008
    cy = state_card(x, y, "S26 / EMPTY · LOADING", "Empty and loading")
    rect(f, x + 20, cy, 184, 150, fill=PANEL_ALT, radius=12, stroke=HAIR, sw=1)
    text(f, "Nothing here yet", x + 34, cy + 18, 11, UI, "SemiBold", INK900)
    text(f, "No agents yet — submit a prompt to create the first run.",
         x + 34, cy + 40, 8.5, UI, "Regular", INK600, width=156, lh="145%")
    rect(f, x + 216, cy, 184, 150, fill=PANEL_ALT, radius=12, stroke=HAIR, sw=1)
    ellipse(f, x + 232, cy + 18, 34, 34, fill="none", stroke=H400, sw=3)
    text(f, "Loading", x + 232, cy + 64, 11, UI, "SemiBold", INK900)
    rect(f, x + 232, cy + 92, 150, 9, fill=HAIR, radius=999)
    rect(f, x + 232, cy + 110, 118, 9, fill=HAIR, radius=999)
    text(f, "honest empty copy · skeleton bars · no fabricated rows",
         x + 20, cy + 168, 9, MONO, "Regular", INK500, width=380, lh="140%")

    # 04 kill confirmation
    x, y = 72, 540
    cy = state_card(x, y, "KILL CONFIRMATION", "Kill worker?")
    rect(f, x + 20, cy, 380, 190, fill=RECESS, radius=12)
    rect(f, x + 44, cy + 18, 332, 154, fill=PANEL, radius=14, stroke=ERR, sw=1.5)
    text(f, "Kill worker?", x + 64, cy + 32, 14, UI, "SemiBold", INK900)
    text(f, "tailwind-builder · WORKER · running", x + 64, cy + 54, 9, MONO,
         "Regular", INK500)
    text(f, "The worker cannot be killed from its current state.",
         x + 64, cy + 72, 9.5, UI, "Regular", INK800, width=292)
    text(f, "Only running workers can be killed; refresh the worker state.",
         x + 64, cy + 90, 8.5, MONO, "Regular", INK500, width=292, lh="140%")
    button(f, x + 64, cy + 112, 100, 30, "Cancel", "secondary")
    button(f, x + 176, cy + 112, 152, 30, "Kill worker", "danger")
    text(f, "KILL_WORKER · confirmation first · never silent", x + 20,
         cy + 210, 9, MONO, "Regular", INK500, ls="4%")

    # 05 replay read-only
    x = 540
    cy = state_card(x, y, "REPLAY / READ-ONLY", "Replay (read-only)")
    rect(f, x + 20, cy, 380, 36, fill=INK900, radius=10)
    text(f, "Replay (read-only) · revision held", x + 36, cy + 10, 10, MONO,
         "Regular", H300)
    rect(f, x + 20, cy + 48, 380, 112, fill=PANEL_ALT, radius=12, stroke=HAIR,
         sw=1)
    for i, gx in enumerate([x + 50, x + 190, x + 330]):
        rect(f, gx, cy + 82, 30, 24, fill=RECESS, radius=6, stroke=INK500, sw=1)
    bee_route(f, x + 90, cy + 94, x + 356, cy + 94, color=INK500,
              dashes=[6, 6], bee_at=0.55, bee_size=11)
    text(f, "HISTORY_REVISION_SET → HISTORY_SNAPSHOT · no mutations while in "
            "replay", x + 20, cy + 174, 9, MONO, "Regular", INK500, width=380,
         lh="140%")

    # 06 DECISION-GATED diff explanation
    x = 1008
    cy = state_card(x, y, "S25 / DECISION-GATED", "Code-diff review")
    rect(f, x + 20, cy, 380, 70, fill=PANEL_ALT, radius=10, stroke=HAIR, sw=1)
    text(f, "router.ts", x + 34, cy + 12, 9.5, MONO, "Regular", INK500)
    text(f, "- const route = legacy();", x + 34, cy + 30, 9, MONO, "Regular", ERR)
    text(f, "+ const route = mesh();", x + 34, cy + 48, 9, MONO, "Regular", OK)
    rect(f, x + 20, cy + 82, 380, 22, fill=H100, radius=999,
         stroke=BORDER_GOLD, sw=1)
    text(f, "DECISION-GATED · DIFF_* commands are NOT frozen",
         x + 34, cy + 89, 8.5, MONO, "SemiBold", H800, ls="2%")
    text(f, "Decision required: the code-diff surface is proposal-gated (no "
            "PDF artboard, no frozen DIFF command). It is not built in v1.",
         x + 20, cy + 116, 9, UI, "Regular", INK700, width=380, lh="145%")
    button(f, x + 20, cy + 166, 110, 26, "Propose", "secondary")
    button(f, x + 142, cy + 166, 110, 26, "Apply", "secondary")
    text(f, "U3 G-03 · resolve only after the contract freezes DIFF_*",
         x + 20, cy + 204, 8.5, MONO, "Regular", INK500, ls="2%")


'''
text = text[:start] + NEW + text[end:]

old_frames = ('    frame("08 · HMAS", 0, 4440, 1440, 1100)\n'
              '    frame("09 · States & Refusals", 1520, 4440, 1440, 1100)\n')
new_frames = ('    frame("08 · HMAS Hierarchy", 0, 4440, 1440, 900)\n'
              '    frame("09 · HMAS Governance", 1520, 4440, 1440, 900)\n'
              '    frame("10 · States & Refusals", 3040, 4440, 1440, 900)\n')
assert old_frames in text
text = text.replace(old_frames, new_frames)

old_calls = ('    board_hmas(FRAME_IDS["08 · HMAS"])\n'
             '    board_states(FRAME_IDS["09 · States & Refusals"])\n')
new_calls = ('    board_hmas_topology(FRAME_IDS["08 · HMAS Hierarchy"])\n'
             '    board_hmas_governance(FRAME_IDS["09 · HMAS Governance"])\n'
             '    board_states(FRAME_IDS["10 · States & Refusals"])\n')
assert old_calls in text
text = text.replace(old_calls, new_calls)

old_order = ('    order = ["00 · Cover", "01 · Colour", "02 · Typography", "03 · Space & Depth",\n'
             '             "04 · Brand Lockups", "05 · Agent Cards", "06 · Controls & HUD",\n'
             '             "07 · Spatial Canvas", "08 · HMAS", "09 · States & Refusals"]')
new_order = ('    order = ["00 · Cover", "01 · Colour", "02 · Typography", "03 · Space & Depth",\n'
             '             "04 · Brand Lockups", "05 · Agent Cards", "06 · Controls & HUD",\n'
             '             "07 · Spatial Canvas", "08 · HMAS Hierarchy",\n'
             '             "09 · HMAS Governance", "10 · States & Refusals"]')
assert old_order in text
text = text.replace(old_order, new_order)

old_names = ('        names = ["00 · Cover", "01 · Colour", "02 · Typography",\n'
             '                 "03 · Space & Depth", "04 · Brand Lockups", "05 · Agent Cards",\n'
             '                 "06 · Controls & HUD", "07 · Spatial Canvas",\n'
             '                 "08 · HMAS", "09 · States & Refusals"]')
new_names = ('        names = ["00 · Cover", "01 · Colour", "02 · Typography",\n'
             '                 "03 · Space & Depth", "04 · Brand Lockups", "05 · Agent Cards",\n'
             '                 "06 · Controls & HUD", "07 · Spatial Canvas",\n'
             '                 "08 · HMAS Hierarchy", "09 · HMAS Governance",\n'
             '                 "10 · States & Refusals"]')
assert old_names in text
text = text.replace(old_names, new_names)

text = text.replace('10 artboards', '11 artboards')
io.open(path, 'w', encoding='utf-8', newline='\n').write(text)
print('reboarded: 11 artboards, 3 new 1440x900 boards')
