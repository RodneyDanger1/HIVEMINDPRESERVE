import hashlib, json, datetime, os

ROOT = r"C:\Users\Riley\Desktop\hiverouter-v1"
WIRING = r"C:\Users\Riley\Desktop\hivemind-blueprint\blueprint\UI-WIRING-GUIDE.md"
OMD = os.path.join(ROOT, "docs", "contracts", "UI-COMMANDS-v1.md")
OUT = os.path.join(ROOT, "docs", "audit", "evidence", "controls-verifier-skeleton.json")


def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest().upper()


# (surface_id, guide_order, name, region, expected_render_file, expected_handler, expected_command,
#  expected_events, acceptance, owner_phase, provenance)
SURFACES = [
    ("S01", 1, "Infinite canvas + honeycomb background", "canvas",
     "web/src/flow/CanvasView.tsx (+ web/src/flow/HoneycombGrid.tsx)", "React Flow canvas; viewport state only",
     "none (client-only viewport state)", "NODE_* (render-only)",
     "e2e: pan/zoom bounds 0.1x-3.0x; Ctrl+0 recentres core hive", "Phase 7 (P3/P25); Phase 8 (MP-18)", "PDF:113-114,303-305"),
    ("S02", 2, "HyperAgent hive chat core (pinned)", "chat",
     "web/src/chat/HiveChatCore.tsx", "pinned chat widget; hide on close (widget only)",
     "CHAT_PIN", "ACK; messages persist",
     "pinned core stays centred through pan/zoom; messages persist", "Phase 10 (UI-ADD-16) + Phase 4 MP-09", "PDF:175-184,296-302"),
    ("S03", 3, "Composer / HyperAgent input", "chat",
     "web/src/chat/Composer.tsx", "submit handler -> typed command with correlation_id",
     "SUBMIT_PROMPT {text, project_id, correlation_id}", "ACK + RUN_CREATED; ERROR(E_PROMPT_EMPTY / E_KEY_MISSING)",
     "empty text -> E_PROMPT_EMPTY; valid -> run row + streamed reply", "Phase 10 (UI-ADD-16); Phase 4 WP 4.6; Phase 1 UI-ADD-14", "PDF:235-237,302"),
    ("S04", 4, "Button: Spawn sub-hive", "controls",
     "web/src/chat/RunControls.tsx", "button handler -> command; ACK-or-refuse before UI mutation",
     "SPAWN_SUB_HIVE {parent_task_id, role, correlation_id}", "ACK + NODE_SPAWN; ERROR(E_SPAWN_DEPTH / E_FANOUT_LIMIT)",
     "depth/fan-out over cap -> named refusal; ACK before render", "Phase 4 MP-08; Phase 2; Phase 1 UI-ADD-14", "PDF:238-240"),
    ("S05", 5, "Button: Re-route", "controls",
     "web/src/chat/RunControls.tsx", "button handler -> cycle check via graph authority",
     "REWIRE_TASK {task_id, new_parent_id, expected_rev, correlation_id}", "ACK + NODE_REWIRE; ERROR(E_CYCLE / E_RESYNC_REQUIRED / E_REWIRE_IN_FLIGHT)",
     "cycle -> E_CYCLE; streaming worker -> E_REWIRE_IN_FLIGHT refusal toast", "Phase 2 (P2.2/EXT-R3-01); Phase 10; Phase 1 UI-ADD-14", "PDF:239; source:404"),
    ("S06", 6, "Button: Cancel", "controls",
     "web/src/chat/RunControls.tsx", "button handler -> cancellation token",
     "CANCEL_RUN {run_id, correlation_id}", "ACK + RUN_STATUS(CANCELLED); ERROR(E_RUN_TERMINAL / E_RUN_NOT_FOUND)",
     "run -> Cancelled; partial state preserved; no further provider calls", "Phase 4 MP-09/EXT-R3-03; Phase 10; Phase 1 UI-ADD-14", "PDF:239"),
    ("S07", 7, "Button: Kill worker", "controls",
     "web/src/chat/RunControls.tsx", "button handler -> child terminate + reap",
     "KILL_WORKER {task_id, correlation_id}", "ACK + NODE_STATE(KILLED); ERROR(E_TASK_NOT_FOUND / E_TASK_TERMINAL / E_KILL_DENIED)",
     "child+grandchild gone; Cancelled/Killed state broadcast; ledger row", "Phase 6 MP-06/EXT-R1-03; Phase 10; Phase 1 UI-ADD-14", "PDF:239"),
    ("S08", 8, "Tabs: Workspace/Swarm/Routing/History", "navigation",
     "web/src/panels/TopTabs.tsx", "route/view state only (R-U11 one owner rule)",
     "none (view route state)", "none",
     "one owner rule; deep-link state", "Phase 10 UI-ADD-05", "PDF:241-242"),
    ("S09", 9, "HUD throughput badge", "navigation",
     "web/src/panels/SwarmHud.tsx", "derived aggregate query + WS push; no hardcode",
     "none (derived aggregate; server push)", "RUN_METRICS {superagents, workers, green_pct, delta}",
     "changing graph changes numbers; delta window labelled; empty -> em-dash", "Phase 10 MP-15", "PDF:243-245,303"),
    ("S10", 10, "Time-travel scrubber", "navigation",
     "web/src/panels/TimeTravelScrubber.tsx", "read/replay via history API",
     "HISTORY_REVISION_SET {run_id, revision}", "ACK + HISTORY_SNAPSHOT; ERROR(E_HISTORY_GAP / E_RUN_NOT_FOUND)",
     "scrubber replays a real revision (state 12 of 38 is a real count)", "Phase 9 MP-14; Phase 10", "PDF:246-247"),
    ("S11", 11, "Nav rail (72px fixed, 6 items)", "navigation",
     "web/src/panels/NavRail.tsx", "view route state; Settings reads key status",
     "none (view route state)", "none",
     "all six destinations exist; Settings shows key status/refusal", "Phase 10 MP-17", "PDF:248-254,276-281"),
    ("S12", 12, "Library panel (260px)", "navigation",
     "web/src/panels/LibraryPanel.tsx", "template manifest read -> insert command",
     "TEMPLATE_INSERT {template_id, project_id}", "ACK + NODE_SPAWN; ERROR(E_TEMPLATE_UNKNOWN / E_TEMPLATE_LICENSE_MISSING)",
     "templates resolve from manifest; licence row present", "Phase 10 MP-17 + EXT-R2-02/03", "PDF:255-266,282-290"),
    ("S13", 13, "Mini-map (260x170)", "canvas",
     "web/src/flow/MiniMapOverlay.tsx", "client-only minimap (single owner)",
     "none (client-only)", "none",
     "minimap reflects viewport; single minimap owner", "Phase 7 (P65; @xyflow MiniMap)", "PDF:267-268"),
    ("S14", 14, "Refusal toast", "controls",
     "web/src/panels/RefusalToast.tsx", "ERROR frame -> toast + log + ledger",
     "none (consumes ERROR)", "ERROR {code, message, how_to_fix, retryable}",
     "refusal shows code+how_to_fix; dismissible; no fake success", "Phase 1 MP-07; Phase 10 UI-ADD-04", "PDF:269-273"),
    ("S15", 15, "SuperAgent sub-hive card", "nodes",
     "web/src/flow/nodes/SuperAgentCard.tsx", "node render from graph + metrics push",
     "none (render + metrics push)", "NODE_STATE / RUN_METRICS",
     "card fields derive from state; 3 workers / 12.4k ctx not hardcoded", "Phase 7 WP 7.9; Phase 10 MP-16", "PDF:185-191"),
    ("S16", 16, "WorkerAgent card", "nodes",
     "web/src/flow/nodes/WorkerAgentCard.tsx", "node render from graph + metrics push",
     "none (render + metrics push)", "NODE_STATE",
     "progress 0-100 from real state; motion-only progress", "Phase 7; Phase 10 MP-16", "PDF:192-196,222"),
    ("S17", 17, "ToolAgent card", "nodes",
     "web/src/flow/nodes/ToolAgentCard.tsx", "tool result + score payload render",
     "none (render)", "tool result payload",
     "score shown only when measured, else em-dash/UNVERIFIED", "Phase 7; Phase 10 MP-16", "PDF:197-204"),
    ("S18", 18, "Broken-route error card", "nodes",
     "web/src/flow/nodes/BrokenRouteCard.tsx", "Error state from run/tool; retry counter from ledger",
     "none (render)", "ERROR / run status",
     "E-code + attempt N/M sourced from ledger; retry action bounded", "Phase 7; Phase 10 UI-ADD-04", "PDF:205-210"),
    ("S19", 19, "Agent chips / state dot (40px hex)", "controls",
     "web/src/flow/AgentChip.tsx", "enum state -> dot + label + shape cue",
     "none (render from enum)", "NODE_STATE",
     "4 dots map 1:1 into enum; non-colour cue present", "Phase 10 UI-ADD-7A", "PDF:211-217"),
    ("S20", 20, "Zoom controls + shortcut hint", "canvas",
     "web/src/flow/ZoomControls.tsx", "controls + keyboard handler (Ctrl+0 on Windows)",
     "VIEWPORT_SET {x,y,zoom} / VIEWPORT_RECENTER", "ERROR(E_ZOOM_RANGE) for out-of-range",
     "Ctrl+0 Windows; bounds 0.1/3.0; button disabled states", "Phase 8 MP-18", "PDF:304,183"),
    ("S21", 21, "Settings / key status", "navigation",
     "web/src/panels/SettingsPanel.tsx", "settings read; E_KEY_MISSING refusal surface",
     "none (read + refusal surface)", "ERROR(E_KEY_MISSING)",
     "no key -> named refusal from composer+settings; never fake run", "Phase 10 MP-17; Phase 5", "PDF:254; source MP-07"),
    ("S22", 22, "File/code hierarchy viewer", "navigation",
     "web/src/panels/FileTree.tsx", "file list API + open file read (workspace-jailed)",
     "none (read-only API)", "file list payload",
     "tree from workspace; traversal refused outside root", "Phase 10 MP-17; Phase 6 jail", "PDF:291-295"),
    ("S23", 23, "Brand lockups / bee glyph", "canvas",
     "web/src/tokens/brand.ts + docs/ui/figcraft/ASSETS/**", "static assets + token rows",
     "none (static assets)", "none",
     "asset set present; min-size rules enforced (56/32/16px)", "Phase 7 (design system); Phase 10 UI-ADD-05", "PDF:148-169"),
    ("S24", 24, "Elevation e1/e2/e3", "canvas",
     "web/src/tokens/elevation.ts", "CSS elevation tokens",
     "none (CSS tokens)", "none",
     "per-tier shadow spec decided or UNVERIFIED (all three print same value)", "Phase 7 UI-ADD-05", "PDF:137-146; UI-VISUAL-CRITIC"),
    ("S25", 25, "Code-diff review modal (decision-gated)", "surfaces",
     "web/src/panels/CodeDiffModal.tsx", "decision-gated: DO NOT IMPLEMENT until user approves spec",
     "DIFF_PROPOSE / DIFF_APPLY / DIFF_REJECT (NOT FROZEN)", "APPROVAL_REQUEST / APPROVAL_RESOLVED; ERROR(E_DECISION_REQUIRED)",
     "decision-gated; diff renders real Phase-6 patch; denial leaves bytes unchanged", "Phase 6 MP-05; Phase 10 UI-ADD-06", "U3 G-03; PROPOSED-ADDITION"),
    ("S26", 26, "First-run/empty/loading/offline/missing-key states", "surfaces",
     "web/src/panels/StatePack.tsx", "UI-ADD-04 state pack",
     "none (surface states)", "none",
     "each state has copy + action; no blank success", "Phase 10 UI-ADD-04", "U1 M-01..M-12"),
    ("S27", 27, "Streamed reply/cards", "chat",
     "web/src/chat/ReplyStream.tsx", "WS fan-out -> streaming chat/card render",
     "none (consumes events)", "REPLY_DELTA / REPLY_DONE",
     "stream renders progressively; terminal event clears thinking", "Phase 4 MP-09; Phase 2 MP-13; Phase 10", "CF-UI-06; source:1738"),
    ("S28", 28, "Project identity", "chat",
     "web/src/panels/ProjectIdentity.tsx", "project_id/session fields; string table",
     "none (state-derived)", "none",
     "identity from state, not hardcoded ('Project: HiveMind AI' / AC A. Chen)", "Phase 4 MP-09; Phase 10 UI-ADD-11", "PDF:256,275-282"),
]

assert len(SURFACES) == 28
assert [s[0] for s in SURFACES] == [f"S{i:02d}" for i in range(1, 29)]
assert sorted(s[1] for s in SURFACES) == list(range(1, 29))

regions = {}
for sid, order, name, region, rfile, handler, cmd, events, acc, owner, prov in SURFACES:
    control = {
        "surface_id": sid,
        "guide_order": order,
        "name": name,
        "render_file": None,
        "expected_render_file": rfile,
        "render_file_status": "PLANNED (frontend not implemented at baseline; web/ contains 0 files)",
        "expected_handler": handler,
        "expected_command": cmd,
        "expected_events": events,
        "expected_acceptance": acc,
        "owner_phase": owner,
        "provenance": prov,
        "verdict": "UNVERIFIED",
        "evidence": "none â€” Phase A baseline; surface not rendered yet",
        "blocked_by": "Phase 7/8/10 frontend not built (web/ empty at baseline)",
        "trace_ladder": {
            "render": "UNVERIFIED",
            "handler": "UNVERIFIED",
            "client_call": "UNVERIFIED",
            "request": "UNVERIFIED",
            "route": "UNVERIFIED",
            "service": "UNVERIFIED",
            "store": "UNVERIFIED",
            "provider": "UNVERIFIED",
            "response": "UNVERIFIED",
            "render_back": "UNVERIFIED",
            "re_entry": "UNVERIFIED",
        },
    }
    regions.setdefault(region, []).append(control)

tabs = []
region_names = {
    "canvas": "Canvas & viewport (S01,S13,S20,S23,S24)",
    "chat": "Chat core, composer, streaming, identity (S02,S03,S27,S28)",
    "controls": "Mutating run controls, refusal toast, state chips (S04-S07,S14,S19)",
    "navigation": "Tabs, HUD, scrubber, rail, library, settings, files (S08-S12,S21,S22)",
    "nodes": "Node cards (S15-S18)",
    "surfaces": "State pack + decision-gated diff modal (S25,S26)",
}
for rid in ["canvas", "chat", "controls", "navigation", "nodes", "surfaces"]:
    ctrls = sorted(regions[rid], key=lambda c: c["guide_order"])
    tabs.append({
        "id": rid,
        "name": region_names[rid],
        "control_count": len(ctrls),
        "controls": ctrls,
    })

doc = {
    "register": "docs/trace/controls.json",
    "kind": "skeleton inventory (Phase A baseline)",
    "seat": "independent verifier",
    "generated_at": datetime.datetime.now().astimezone().isoformat(),
    "baseline_commit": "d1638f2",
    "sources": {
        "ui_wiring_guide": {"path": WIRING, "sha256": sha(WIRING), "surfaces": "S01-S28"},
        "ui_commands_v1": {"path": OMD, "sha256": sha(OMD)},
        "ui_requirements": {"path": r"C:\Users\Riley\Desktop\hivemind-blueprint\reports\ui\UI-REQUIREMENTS.md"},
        "contract": "docs/contracts/UI-COMMANDS-v1.md (frozen Phase 1)",
    },
    "counts": {
        "surfaces_expected": 28,
        "surfaces_inventoried": len(SURFACES),
        "controls_total": len(SURFACES),
        "wired_and_works": 0,
        "wired_but_dead": 0,
        "missing": 0,
        "refused_by_name": 0,
        "unverified": 28,
        "rendered_files_present": 0,
    },
    "tabs": tabs,
    "verdict_definitions": {
        "WIRED-AND-WORKS": "render->handler->client->route->service->store->response->render-back proven with path:line or captured request/DB evidence",
        "WIRED-BUT-DEAD": "control renders but a hop is dead/no-op/fake success",
        "MISSING": "surface not rendered at all",
        "REFUSED-BY-NAME": "surface intentionally refuses with a named code (e.g. decision-gated S25)",
        "UNVERIFIED": "not yet exercised; no evidence",
    },
    "frontend_state_at_baseline": {
        "web_dir": os.path.join(ROOT, "web"),
        "files_present": 0,
        "directories": ["src", "tests"],
        "note": "UI surfaces are planned only; expected_render_file paths follow EXECUTION-PROMPT Â§3 layout (web/src/{tokens,flow,chat,panels,state,api,a11y}) and are NOT authoritative file names until the frontend worker commits them.",
    },
    "gaps": [
        {"id": "GAP-UI-01", "detail": "All 28 surfaces are UNVERIFIED: no React app files exist at baseline (web/ has 0 files).", "owner": "Frontend worker (Phase 7/8/10)", "closed_when": "render file exists and trace ladder re-run"},
        {"id": "GAP-UI-02", "detail": "No e2e test harness / package.json in web/ yet; expected_probe commands are planned, not executable.", "owner": "Frontend worker", "closed_when": "npm scripts + Playwright config land"},
        {"id": "GAP-UI-03", "detail": "S25 DIFF_PROPOSE/APPLY/REJECT is NOT FROZEN (decision-gated) â€” must not be implemented or claimed until the user approves the code-diff spec.", "owner": "Lead/user decision", "closed_when": "user decision recorded in CHANGE-LEDGER.md"},
        {"id": "GAP-UI-04", "detail": "Visual fidelity to FigCraft PDF is UNVERIFIED/needs_human (no board raster; PDF text-layer only).", "owner": "UI watchdog / human", "closed_when": "human visual acceptance or board raster provided"},
        {"id": "GAP-UI-05", "detail": "Surface-count reconciliation (28 = 24 PDF surfaces + 4 additions; 26 = 22 controls + 4 buttons) must be re-proven against the implemented app.", "owner": "Verifier (Phase B)", "closed_when": "inventory from live render matches 28/24+4"},
    ],
    "probeCorrections": [],
    "expected_probe": {
        "per_control": "read expected_render_file:line (when present, render+handler hops); e2e: npm run e2e -- --grep <surface_id> (planned)",
        "process": "inventory reading order -> static code hops -> live request capture -> negative paths -> verdict",
    },
}

os.makedirs(os.path.dirname(OUT), exist_ok=True)
json.dump(doc, open(OUT, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print("WROTE", OUT, os.path.getsize(OUT), "bytes")
print("surfaces:", len(SURFACES))
for t in tabs:
    print(t["id"], t["control_count"])
print("total:", sum(t["control_count"] for t in tabs))

