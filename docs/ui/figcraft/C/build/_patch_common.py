import io

path = 'docs/ui/figcraft/C/build/boards_common.py'
text = io.open(path, encoding='utf-8').read()
text = text.replace(
    '"""Shared model for the HiveMind v1 canvas boards (FigCraft-B).',
    '"""Shared model for the HiveMind v1 C boards (FigCraft-C; palette copied from FigCraft-B, unchanged).',
)
text = text.replace(
    'every id comes from\n`figcraft-cli commands --json` (597 rows, figcraft-cli 0.11.2).',
    'every id comes from\n`figcraft-cli commands --json` (597 rows, figcraft-cli 0.11.2). The palette is\ncopied verbatim from FigCraft-B; nothing is invented for C.',
)
text += '''

# ------------------------------------------------------------------ surfaces C
# C surface rows: frozen S ids where the C board re-renders an existing surface,
# plus C-specific ids for the HMAS/state panels introduced by this set.
C_SURFACES = {
    "S01": ("Infinite canvas + honeycomb lattice", "viewport state only", "PDF:113-114,303-305"),
    "S02": ("HyperAgent hive chat core (pinned)", "CHAT_PIN", "PDF:175-184,296-302"),
    "S03": ("Composer / HyperAgent input", "SUBMIT_PROMPT", "PDF:235-237,302"),
    "S09": ("HUD throughput badge", "RUN_METRICS push; derived aggregate", "PDF:243-245,303"),
    "S10": ("Time-travel scrubber", "HISTORY_REVISION_SET", "PDF:246-247"),
    "S11": ("Nav rail 72px, 6 items", "view route state", "PDF:248-254,276-281"),
    "S12": ("Library panel 260px", "TEMPLATE_INSERT; manifest", "PDF:255-266,282-290"),
    "S13": ("Mini-map 260x170", "client-only", "PDF:267-268"),
    "S14": ("Refusal toast / named refusal card", "ERROR frame -> toast + log + ledger", "PDF:269-273"),
    "S15": ("SuperAgent sub-hive card", "node render + metrics push", "PDF:185-191"),
    "S16": ("WorkerAgent card", "metrics push; progress", "PDF:192-196,222"),
    "S19": ("Agent chips / state dot", "enum state -> dot + shape/text cue", "PDF:211-217"),
    "S20": ("Zoom controls + Ctrl+0 hint", "VIEWPORT_SET / VIEWPORT_RECENTER", "PDF:303-304,183"),
    "S22": ("File/code hierarchy viewer", "workspace-jailed file list", "PDF:291-295"),
    "S23": ("Brand lockups / bee glyph", "static tokens", "PDF:148-169"),
    "S25": ("Code-diff review modal (decision-gated)", "DIFF_* NOT FROZEN - do not implement", "U3 G-03; UI-COMMANDS-v1 section 2"),
    "S26": ("First-run / empty / loading states", "empty + loading truth states", "PDF silent"),
    "S28": ("Project identity", "project_id / string table", "PDF:256,275-282"),
    "C-HMAS-PROFILES": ("Profiles + import panel", "profile state + import action", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-MEMORY": ("Memory list + clear", "clear requires confirmation", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-MAILBOX": ("Mailbox", "cross-branch messages", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-PROJECTS": ("Projects", "project identity rows", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-USAGE": ("Usage", "derived counters", "PROPOSED-ADDITION (task-6)"),
    "C-HMAS-APPROVAL": ("Cross-branch message approval", "APPROVAL_RESOLVE with nonce", "PROPOSED-ADDITION (task-6)"),
    "C-STATE-RECONNECT": ("Reconnect state", "SERVER_HELLO / RESYNC_REQUIRED", "PROPOSED-ADDITION (task-6)"),
    "C-STATE-KILL": ("Kill confirmation", "KILL_WORKER confirmation", "PROPOSED-ADDITION (task-6)"),
    "C-STATE-REPLAY": ("Replay read-only", "HISTORY_REVISION_SET / HISTORY_SNAPSHOT", "UI-COMMANDS-v1 section 3"),
}
'''
io.open(path, 'w', encoding='utf-8', newline='\n').write(text)
print('patched', path, len(text))
