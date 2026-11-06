"""Board 3 - "Canvas / interaction states": 9 state panels + decision-gated diff."""
from boards_common import C, STATES, STATE_COLOR, STATE_SHAPE, honeycomb_path, hex_subpath


def _cell(b, x, y, w, h, eyebrow, title, name):
    b.rect(x, y, w, h, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           name=name, effect="e1")
    b.eyebrow(x + 16, y + 10, eyebrow, size=8, name=name + "/eyebrow")
    b.name_t(x + 16, y + 22, title, 12.5, name=name + "/title")


def build_interaction(frame):
    from boards_common import Board
    b = Board("02", "Canvas / interaction states", frame, 1440, 900,
              "b-interaction")
    b.rect(0, 0, 1440, 900, fill=C["canvas"], name="ix/bg")
    b.vector(honeycomb_path(0, 60, 1440, 900, r=46), 0, 0, stroke=C["honey500"],
             sw=1.0, opacity=0.10, name="ix/lattice-bg")

    # ------------------------------------------------------------------ header
    b.eyebrow(48, 26, "S14 · S24 · S25 · S26 · INTERACTION STATES · "
                      "UI-COMMANDS-v1 §2 / R-U3 / R-U9", name="ix/eyebrow")
    b.text(48, 42, "Interaction states", 40, "Playfair Display", "SemiBold",
           C["ink900"], width=760, line=48, tracking=-1.5, name="ix/title")
    b.body(48, 96, "Every mutating control shows ACK or a named refusal before "
                   "UI state changes · focus >=3:1 · refusals never silent · "
                   "diff DECISION-GATED", 900, size=12.5, line=17,
           name="ix/intro")
    b.eyebrow(1160, 26, "1440 x 900 · 9 PANELS + DIFF", width=232,
              align="right", size=8, name="ix/right-eyebrow")
    b.meta(1160, 44, "PDF:237,269-273 · UI-ADD-04", width=232, align="right",
           size=8.5, name="ix/right-note")

    # ================================================== row 1, cell 1: selected
    x, y = 48, 128
    _cell(b, x, y, 416, 192, "S02 · SELECTED NODE", "focus ring ink-700",
          "s02/selected-cell")
    b.rect(x + 17, y + 53, 206, 102, fill=None, r=14, stroke=C["ink700"], sw=2,
           name="s02/focus-ring")
    b.rect(x + 20, y + 56, 200, 96, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, name="s02/selected-card")
    b.name_t(x + 36, y + 68, "tailwind-builder", 13.5, name="s02/selected-name")
    b.eyebrow(x + 36, y + 88, "WORKER · react/vite", size=9,
              color=C["ink500"], name="s02/selected-role")
    b.text(x + 148, y + 68, "55%", 10.5, "Consolas", "Regular", C["ink700"],
           width=52, align="right", line=13, name="s02/selected-pct")
    b.rect(x + 36, y + 112, 168, 5, fill=C["recess"], r=999,
           name="s02/selected-track")
    b.rect(x + 36, y + 113, 92, 3, fill=C["honey500"], r=999,
           name="s02/selected-bar")
    b.meta(x + 16, y + 164, "ring ink-700 #4E3A15 = 9.86:1 >= 3:1 · PDF honey-300 "
                            "1.70 FAILS (R-U3)", width=384, size=8,
           name="s02/focus-note")
    b.meta(x + 16, y + 178, "selected keeps hover/pan; focus is never colour-only",
           width=384, size=8, name="s02/selected-note")

    # ============================================== row 1, cell 2: drag ghost
    x = 480
    _cell(b, x, y, 416, 192, "NODE_DRAG · DRAGGING GHOST",
          "drag preserves identity", "s16/drag-cell")
    b.rect(x + 22, y + 56, 176, 78, fill=None, r=12, stroke=C["ink500"], sw=1.5,
           dash=[4, 4], name="s16/drag-origin")
    b.rect(x + 62, y + 48, 176, 78, fill=C["honey100"], r=12, stroke=C["honey400"],
           sw=1.5, opacity=0.62, name="s16/drag-ghost")
    b.rect(x + 66, y + 52, 168, 70, fill=None, r=10, stroke=C["hairline"], sw=1,
           opacity=0.5, name="s16/drag-ghost-inner")
    b.text(x + 22, y + 140, "origin (dashed) -> ghost 45% follows cursor",
           9.5, "Consolas", "Regular", C["ink700"], width=384, line=13,
           name="s16/drag-caption")
    b.meta(x + 16, y + 164, "NODE_DRAG event · drop snaps to the 8px grid; no "
                            "optimistic mutation", width=384, size=8,
           name="s16/drag-note")
    b.meta(x + 16, y + 178, "ghost is client state only - re-parent is a "
                            "REWIRE_TASK, not a drag side-effect", width=384,
           size=8, name="s16/drag-note2")

    # ============================================ row 1, cell 3: rewire preview
    x = 912
    _cell(b, x, y, 416, 192, "S05 · EDGE RE-ROUTE PREVIEW",
          "REWIRE_TASK · cycle-checked", "s05/rewire-cell")
    b.rect(x + 20, y + 54, 110, 30, fill=C["honey100"], r=8,
           stroke=C["honey400"], sw=1, name="s05/parent-a")
    b.text(x + 20, y + 62, "SuperAgent A", 9.5, "Inter", "SemiBold",
           C["ink800"], width=110, align="center", line=12,
           name="s05/parent-a/label")
    b.rect(x + 272, y + 54, 110, 30, fill=C["honey100"], r=8,
           stroke=C["honey400"], sw=1, name="s05/parent-b")
    b.text(x + 272, y + 62, "SuperAgent B", 9.5, "Inter", "SemiBold",
           C["ink800"], width=110, align="center", line=12,
           name="s05/parent-b/label")
    b.rect(x + 20, y + 118, 110, 30, fill=C["paper"], r=8,
           stroke=C["hairline"], sw=1, name="s05/child")
    b.text(x + 20, y + 126, "tailwind-builder", 9.5, "Inter", "SemiBold",
           C["ink800"], width=110, align="center", line=12,
           name="s05/child/label")
    b.vector("M %d %d L %d %d" % (x + 75, y + 84, x + 75, y + 118), 0, 0,
             stroke=C["honey600"], sw=1.3, name="s05/edge-current")
    b.vector("M %d %d Q %d %d %d %d" % (x + 130, y + 130, x + 232, y + 140,
                                        x + 272, y + 92), 0, 0,
             stroke=C["amber"], sw=1.6, name="s05/edge-preview")
    b.cmd[-1]["params"]["props"]["dashes"] = [6, 4]
    b.bee(258 + x, y + 116, 15, -26, name="s05/edge-preview/bee",
          body=C["amber"])
    b.meta(x + 16, y + 164, "dashed amber = preview only; commit on ACK + "
                            "expected_rev match", width=384, size=8,
           name="s05/rewire-note")
    b.meta(x + 16, y + 178, "cycle -> E_CYCLE · stale rev -> E_RESYNC_REQUIRED "
                            "· streaming -> E_REWIRE_IN_FLIGHT", width=384,
           size=8, name="s05/rewire-note2")

    # ============================================ row 2, cell 1: reconnect
    x, y = 48, 332
    _cell(b, x, y, 416, 192, "S26 · RECONNECT BANNER",
          "reconnecting until resnapshot", "s26/reconnect-cell")
    b.rect(x + 16, y + 50, 384, 48, fill=C["honey200"], r=10,
           name="s26/reconnect-banner")
    b.poly(x + 32, y + 64, 14, 14, C["amber"], pointCount=4, rotation=45,
           name="s26/reconnect-diamond")
    b.text(x + 56, y + 58, "Reconnecting…", 12.5, "Inter", "SemiBold",
           C["ink900"], width=200, line=16, name="s26/reconnect-title")
    b.meta(x + 56, y + 78, "WS dropped · hello{last_seq} -> RESYNC_REQUIRED",
           width=330, size=8.5, name="s26/reconnect-sub")
    b.meta(x + 16, y + 164, "state = reconnecting · healthy only after the "
                            "snapshot resync (G-UI-10)", width=384, size=8,
           name="s26/reconnect-note")
    b.meta(x + 16, y + 178, "no optimistic success while the socket is down; "
                            "stale mutation -> E_RESYNC_REQUIRED", width=384,
           size=8, name="s26/reconnect-note2")

    # ======================================= row 2, cell 2: awaiting approval
    x = 480
    _cell(b, x, y, 416, 192, "S26 · AWAITING APPROVAL",
          "approval gate blocks the run", "s26/approval-cell")
    b.rect(x + 36, y + 44, 344, 108, fill=C["paper"], r=12,
           stroke=C["amber"], sw=1.5, name="s26/approval-card")
    b.name_t(x + 52, y + 56, "Tool approval required", 12.5,
             name="s26/approval-title")
    b.meta(x + 52, y + 74, "APPROVAL_REQUEST · apply_patch · approval_id",
           width=312, size=8.5, name="s26/approval-meta")
    b.body(x + 52, y + 90, "awaiting_approval blocks approve/deny until the "
                           "request is resolved", 312, size=9.5, line=13,
           name="s26/approval-body")
    b.rect(x + 52, y + 118, 96, 26, fill=C["ink900"], r=8,
           name="s26/approval-approve")
    b.text(x + 52, y + 124, "Approve", 10.5, "Inter", "SemiBold", C["paper"],
           width=96, align="center", line=14, name="s26/approval-approve/label")
    b.rect(x + 156, y + 118, 88, 26, fill=C["paper"], r=8,
           stroke=C["signal"], sw=1.5, name="s26/approval-deny")
    b.text(x + 156, y + 124, "Deny", 10.5, "Inter", "SemiBold", C["signal"],
           width=88, align="center", line=14, name="s26/approval-deny/label")
    b.meta(x + 16, y + 164, "APPROVAL_RESOLVED resumes; timeout -> "
                            "E_APPROVAL_TIMEOUT", width=384, size=8,
           name="s26/approval-note")
    b.meta(x + 16, y + 178, "no tool executes until the gate resolves", width=384,
           size=8, name="s26/approval-note2")

    # ================================================= row 2, cell 3: refusal
    x = 912
    _cell(b, x, y, 416, 192, "S14 · REFUSAL TOAST (BY NAME)",
          "refused, not silent", "s14/refusal-cell")
    b.rect(x + 16, y + 44, 384, 116, fill=C["ink900"], r=12,
           name="s14/toast", surface="S14", effect="e2")
    b.text(x + 32, y + 54, "Re-route refused", 13, "Inter", "SemiBold",
           C["paper"], width=340, line=17, name="s14/toast/title")
    b.text(x + 32, y + 76, "context-prune blocked: worker still streaming "
                           "(NODE_REWIRE pending)", 10, "Inter", "Regular",
           C["honey100"], width=352, line=14, name="s14/toast/reason")
    b.rect(x + 32, y + 110, 352, 1, fill=C["ink700"], name="s14/toast/rule")
    b.text(x + 32, y + 118, "REFUSED BY NAME  ·  NOT SILENT", 9.5, "Consolas",
           "Regular", C["honey300"], width=210, line=13, tracking=8,
           name="s14/toast/motto")
    b.rect(x + 246, y + 116, 138, 20, fill=C["signal"], r=999,
           name="s14/toast/code-bg")
    b.text(x + 246, y + 120, "E_REWIRE_IN_FLIGHT", 8.5, "Consolas", "Regular",
           C["paper"], width=138, align="center", line=12,
           name="s14/toast/code")
    b.text(x + 32, y + 140, "how_to_fix: wait for the stream to end, then re-route",
           8.5, "Consolas", "Regular", C["honey100"], width=352, line=12,
           name="s14/toast/fix")
    b.meta(x + 16, y + 164, "toast + log + ledger carry code / message / how_to_fix "
                            "(R-U9)", width=384, size=8,
           name="s14/toast/note")
    b.meta(x + 16, y + 178, "the blocker is the streaming worker, not the target "
                            "parent", width=384, size=8,
           name="s14/toast/note2")

    # ================================================ row 3, cell 1: kill
    x, y = 48, 536
    _cell(b, x, y, 416, 192, "S07 · KILL WORKER CONFIRM",
          "destructive action confirms", "s07/kill-cell")
    b.rect(x + 36, y + 44, 344, 108, fill=C["paper"], r=12,
           stroke=C["hairline"], sw=1, name="s07/kill-modal", effect="e2")
    b.name_t(x + 52, y + 56, "Kill worker?", 13, name="s07/kill-title")
    b.meta(x + 52, y + 74, "KILL_WORKER · task_id · correlation_id", width=312,
           size=8.5, name="s07/kill-command")
    b.body(x + 52, y + 90, "tailwind-builder + children terminate; ledger row written",
           312, size=9.5, line=13, name="s07/kill-body")
    b.rect(x + 52, y + 118, 88, 26, fill=C["canvas"], r=8,
           stroke=C["hairline"], sw=1, name="s07/kill-cancel")
    b.text(x + 52, y + 124, "Cancel", 10.5, "Inter", "SemiBold", C["ink700"],
           width=88, align="center", line=14, name="s07/kill-cancel/label")
    b.rect(x + 148, y + 118, 110, 26, fill=C["signal"], r=8,
           name="s07/kill-confirm")
    b.text(x + 148, y + 124, "Kill worker", 10.5, "Inter", "SemiBold",
           C["paper"], width=110, align="center", line=14,
           name="s07/kill-confirm/label")
    b.meta(x + 16, y + 164, "one ACK then NODE_STATE(KILLED) · child + "
                            "grandchild reaped (MP-06)", width=384, size=8,
           name="s07/kill-note")
    b.meta(x + 16, y + 178, "E_KILL_DENIED / E_TASK_TERMINAL are named "
                            "refusals, never silent drops", width=384, size=8,
           name="s07/kill-note2")

    # ============================================= row 3, cell 2: replay
    x = 480
    _cell(b, x, y, 416, 192, "S10 · TIME-TRAVEL REPLAY",
          "replay is a read-only view", "s10/replay-cell")
    b.rect(x + 16, y + 50, 384, 62, fill=C["paper"], r=12,
           stroke=C["hairline"], sw=1, name="s10/replay-scrubber")
    b.ellipse(x + 28, y + 60, 22, 22, fill=C["honey300"],
              name="s10/replay-play")
    b.vector("M %d %d L %d %d L %d %d Z" % (x + 35, y + 66, x + 45, y + 71,
                                           x + 35, y + 76), 0, 0,
             fill=C["ink900"], name="s10/replay-play-icon")
    b.text(x + 60, y + 56, "09:41:07", 11, "Consolas", "Regular", C["ink900"],
           width=100, line=14, name="s10/replay-time")
    b.meta(x + 236, y + 58, "routing state 12 of 38", width=156, align="right",
           size=8.5, name="s10/replay-state")
    b.rect(x + 60, y + 86, 324, 2, fill=C["hairline"],
           name="s10/replay-track")
    b.rect(x + 60, y + 86, 102, 2, fill=C["honey500"],
           name="s10/replay-progress")
    b.ellipse(x + 156, y + 80, 12, 12, fill=C["paper"], stroke=C["honey500"],
             sw=2, name="s10/replay-knob")
    b.pill(x + 16, y + 118, 142, 22, C["ink700"], "replay · read-only",
           C["paper"], size=9, name="s10/replay-badge")
    b.meta(x + 170, y + 122, "read-only view state · no mutation while scrubbing",
           width=230, size=8, name="s10/replay-badge-note")
    b.meta(x + 16, y + 164, "HISTORY_REVISION_SET -> HISTORY_SNAPSHOT · renders "
                            "a real revision (MP-14)", width=384, size=8,
           name="s10/replay-note")
    b.meta(x + 16, y + 178, "replay is a view state: no mutation is allowed "
                            "while scrubbing", width=384, size=8,
           name="s10/replay-note2")

    # ============================================ row 3, cell 3: elevation S24
    x = 912
    _cell(b, x, y, 416, 192, "S24 · ELEVATION e1 / e2 / e3",
          "one printed shadow value", "s24/elevation-cell")
    tiers = [("e1 / node", "cards rest", x + 20, "e1"),
             ("e2 / float", "chat, menus", x + 142, "e2"),
             ("e3 / hive", "behind chat", x + 264, "e3")]
    for i, (label, sub, cx, tier) in enumerate(tiers):
        b.rect(cx, y + 54, 110, 58, fill=C["paper"], r=12,
               stroke=C["hairline"], sw=1,
               name="s24/elevation-panel" if i == 0 else "s24/elev-%d" % i,
               surface="S24" if i == 0 else None,
               effect=tier)
        b.text(cx + 10, y + 62, label, 9.5, "Inter", "SemiBold", C["ink800"],
               width=90, line=13, name="s24/elev-%d/label" % i)
        b.meta(cx + 10, y + 80, sub, width=90, size=8,
               name="s24/elev-%d/sub" % i)
    b.meta(x + 16, y + 124, "all three tiers print #282010 @ 7-13% - identical "
                            "(PDF:137-146)", width=384, size=8,
           name="s24/elevation-note")
    b.meta(x + 16, y + 138, "tiers are UNRECONCILED: do not differentiate "
                            "without a decision", width=384, size=8,
           name="s24/elevation-note2")
    b.meta(x + 16, y + 164, "e1 cards resting on canvas · e2 chat core / drawers "
                            "/ menus · e3 the hive", width=384, size=8,
           name="s24/elevation-legend")
    b.meta(x + 16, y + 178, "labels are PDF copy; single value kept so no tier "
                            "invents a shadow", width=384, size=8,
           name="s24/elevation-legend2")

    # =================================================== bottom: S25 (gated)
    b.rect(48, 740, 864, 132, fill=C["paper"], r=12, stroke=C["hairline"],
           sw=1, name="s25/diff-modal", surface="S25", effect="e2")
    b.rect(48, 740, 864, 26, fill=C["ink900"], r=12, name="s25/diff-banner")
    b.rect(48, 752, 864, 14, fill=C["ink900"], name="s25/diff-banner2")
    b.meta(64, 747, "S25 · CODE-DIFF REVIEW · DECISION-GATED · DIFF_PROPOSE / "
                    "DIFF_APPLY / DIFF_REJECT NOT FROZEN · NOT IMPLEMENTED",
           width=832, size=8.5, color=C["honey200"], name="s25/diff-banner-text")
    b.rect(64, 778, 392, 58, fill=C["recess"], r=6, name="s25/diff-before")
    b.meta(72, 782, "BEFORE · current bytes untouched", width=376, size=8,
           name="s25/diff-before-label")
    b.text(72, 796, "func route(worker, superAgentB)\n  return hive.snap(worker)",
           8.5, "Consolas", "Regular", C["ink900"], width=376, line=12,
           name="s25/diff-before-code")
    b.rect(472, 778, 392, 58, fill=C["honey100"], r=6, name="s25/diff-after")
    b.meta(480, 782, "AFTER · PROPOSED (not applied)", width=376, size=8,
           name="s25/diff-after-label")
    b.text(480, 796, "func route(worker, subHive)\n  return hive.route(worker)",
           8.5, "Consolas", "Regular", C["ink900"], width=376, line=12,
           name="s25/diff-after-code")
    b.rect(64, 844, 100, 20, fill=C["ink900"], r=6, name="s25/approve-bg")
    b.text(64, 848, "Approve", 9, "Inter", "SemiBold", C["paper"], width=100,
           align="center", line=12, name="s25/approve-label")
    b.rect(172, 844, 80, 20, fill=C["paper"], r=6, stroke=C["signal"], sw=1.5,
           name="s25/deny-bg")
    b.text(172, 848, "Deny", 9, "Inter", "SemiBold", C["signal"], width=80,
           align="center", line=12, name="s25/deny-label")
    b.meta(264, 848, "E_DECISION_REQUIRED · denial leaves bytes unchanged · "
                     "frozen contract wins over the mock-up", width=640,
           size=8.5, color=C["ink500"], name="s25/diff-note")

    # ============================================= bottom: state legend (S19/S26)
    b.rect(928, 740, 464, 132, fill=C["panel_alt"], r=12,
           stroke=C["hairline"], sw=1, name="s19/state-legend", effect="e1")
    b.eyebrow(944, 748, "ONE >=13-STATE ENUM · DOT + LABEL + SHAPE CUE",
              size=8, name="s19/legend-label")
    order = ["healthy", "running", "queued", "failed", "completed",
             "cancelled", "killed", "planning", "thinking", "streaming",
             "awaiting_approval", "reconnecting", "replay", "empty", "loading"]
    for i, st in enumerate(order):
        row, col = divmod(i, 4)
        cx = 944 + col * 108
        cy = 768 + row * 21
        b.dot(cx, cy, 7, STATE_SHAPE[st], STATE_COLOR[st],
              name="s19/legend/%s" % st)
        b.text(cx + 11, cy - 1, st, 8, "Consolas", "Regular", C["ink700"],
               width=80, line=11, tracking=2, name="s19/legend/%s/label" % st)
    b.meta(944, 852, "colour is never the only channel; unknown metric -> NULL / "
                     "\"—\", never 0", width=432, size=8,
           name="s19/legend-note")

    b.meta(48, 884, "CANVAS / INTERACTION STATES · 1440x900 · UI-ADD-04 · "
                    "UI-COMMANDS-v1 §2 · PDF:237,269-273", width=900, size=8,
           name="ix/footer")
    return b
