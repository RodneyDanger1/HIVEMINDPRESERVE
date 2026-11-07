"""Publish the regenerated HiveMind FigCraft set.

Reads the regenerated HiveMind-Style-System.figcraft + CLI info + exports, then:
  1. exports one SVG per artboard (12) into docs/ui/figcraft/ASSETS/
  2. copies the CLI-generated 1x/2x PNG exports into ASSETS as hivemind-*.png
  3. writes HiveMind/HiveMind-ASSETS.json (design file, frames, exports, surfaces)
  4. writes HiveMind/HiveMind-SOURCE-EVIDENCE.json (hashes of preserved evidence)
  5. writes HiveMind/HiveMind-EPISODES.json + episodes/ (phase command scripts)
  6. updates docs/ui/figcraft/ASSETS-INDEX.json for the HiveMind set only
     (A/B rows and hashes are never rewritten)
  7. writes HiveMind/README.md

No source/product code is touched; the external HiveRouter repo is read-only.
"""
import hashlib
import json
import os
import shutil
import subprocess
from datetime import datetime, timezone

from PIL import Image

CLI = r"C:\Program Files\FigCraft\figcraft-cli.exe"
HERE = os.path.dirname(os.path.abspath(__file__))
HM = os.path.dirname(HERE)                      # docs/ui/figcraft/HiveMind
FIG = os.path.dirname(HM)                       # docs/ui/figcraft
REPO = os.path.abspath(os.path.join(FIG, "..", "..", ".."))
ASSETS = os.path.join(FIG, "ASSETS")
DESIGN = os.path.join(HM, "HiveMind-Style-System.figcraft")
INDEX = os.path.join(FIG, "ASSETS-INDEX.json")
MANIFEST = os.path.join(HM, "HiveMind-ASSETS.json")
EVIDENCE = os.path.join(HM, "HiveMind-SOURCE-EVIDENCE.json")
EPISODES = os.path.join(HM, "episodes")
SOURCE = r"C:\Users\Riley\Desktop\HiveRouter\design\hive-style"

FRAMES = [
    ("00", "00 · Cover", "cover", 1440, 900),
    ("01", "01 · Colour", "colour", 1440, 1240),
    ("02", "02 · Typography", "typography", 1440, 1340),
    ("03", "03 · Space & Depth", "space-and-depth", 1440, 1000),
    ("04", "04 · Brand Lockups", "brand-lockups", 1440, 1000),
    ("05", "05 · Agent Cards", "agent-cards", 1440, 1300),
    ("06", "06 · Controls & HUD", "controls-and-hud", 1440, 1040),
    ("07", "07 · Spatial Canvas", "spatial-canvas", 1440, 900),
    ("08", "08 · HMAS Hierarchy", "hmas-hierarchy", 1440, 900),
    ("09", "09 · HMAS Governance", "hmas-governance", 1440, 900),
    ("10", "10 · States & Refusals", "states-and-refusals", 1440, 900),
    ("11", "11 · Motion & Live", "motion-and-live", 1440, 900),
]

SURFACES = [
    # (surface_id, name, board slug, provenance)
    ("S01", "Infinite canvas + honeycomb background", "spatial-canvas", "HiveRouter-Style-Kit §canvas"),
    ("S02", "HyperAgent hive chat core (pinned)", "spatial-canvas", "HiveRouter-Target-Spec §2"),
    ("S03", "Composer / HyperAgent input", "controls-and-hud", "HiveRouter-Target-Spec §2"),
    ("S04", "Button: Spawn sub-hive", "controls-and-hud", "HiveRouter-Target-Spec §2"),
    ("S05", "Button: Re-route", "controls-and-hud", "HiveRouter-Target-Spec §2"),
    ("S06", "Button: Cancel", "controls-and-hud", "HiveRouter-Target-Spec §2"),
    ("S07", "Button: Kill worker", "controls-and-hud", "HiveRouter-Target-Spec §2"),
    ("S08", "Tabs: Workspace / Swarm / Routing / History", "controls-and-hud", "HiveRouter-Target-Spec §2"),
    ("S09", "HUD throughput badge (460x44)", "spatial-canvas", "HiveRouter-Style-Kit §hud"),
    ("S10", "Time-travel scrubber", "controls-and-hud", "HiveRouter-Style-Kit §scrubber"),
    ("S11", "Nav rail (72px, 6 items)", "spatial-canvas", "HiveRouter-Target-Spec §2"),
    ("S12", "Library panel (240px authoritative)", "spatial-canvas", "board 07 composition"),
    ("S13", "Mini-map (220x150 authoritative)", "spatial-canvas", "board 07 composition"),
    ("S14", "Refusal card / named refusal", "states-and-refusals", "HiveRouter-Style-Kit §error"),
    ("S15", "SuperAgent card (420x150)", "agent-cards", "board 07 composition"),
    ("S16", "WorkerAgent card (320x96)", "agent-cards", "board 07 composition"),
    ("S17", "ToolAgent card (420x240)", "agent-cards", "board 07 composition"),
    ("S18", "Broken-route error card (crimson)", "agent-cards", "HiveRouter-Style-Kit §error"),
    ("S19", "Agent chips / state dot", "agent-cards", "HiveRouter-Style-Kit §chips"),
    ("S20", "Zoom controls + shortcut hint", "spatial-canvas", "board 07 composition"),
    ("S22", "File/code hierarchy viewer", "hmas-hierarchy", "HiveRouter-Target-Spec §2"),
    ("S23", "Brand lockups / bee glyph", "brand-lockups", "HiveRouter-Style-Kit §brand"),
    ("S24", "Elevation e1/e2/e3", "agent-cards", "HiveRouter-Style-Kit §depth"),
    ("S25", "Code-diff review (DECISION-GATED)", "states-and-refusals", "HiveRouter-Target-Spec §3 / DECISION-GATED"),
    ("S26", "First-run / empty / loading states", "states-and-refusals", "HiveRouter-Target-Spec §3"),
    ("S27", "Streamed reply / chat cards", "spatial-canvas", "HiveRouter-Target-Spec §2"),
    ("S28", "Project identity", "spatial-canvas", "board 07 library panel"),
    ("HMAS-HIERARCHY", "Hierarchy / workspace-jailed tree", "hmas-hierarchy", "PROPOSED-ADDITION (HMAS hierarchy board)"),
    ("HMAS-TOPOLOGY", "Topology / live bee edges", "hmas-hierarchy", "PROPOSED-ADDITION (HMAS hierarchy board)"),
    ("HMAS-PROFILES", "Profiles panel", "hmas-governance", "PROPOSED-ADDITION (HMAS governance board)"),
    ("HMAS-IMPORT", "Import project", "hmas-governance", "PROPOSED-ADDITION (HMAS governance board)"),
    ("HMAS-MEMORY", "Memory list + clear", "hmas-governance", "PROPOSED-ADDITION (HMAS governance board)"),
    ("HMAS-MAILBOX", "Mailbox", "hmas-governance", "PROPOSED-ADDITION (HMAS governance board)"),
    ("HMAS-PROJECTS", "Projects", "hmas-governance", "PROPOSED-ADDITION (HMAS governance board)"),
    ("HMAS-USAGE", "Usage counters", "hmas-governance", "PROPOSED-ADDITION (HMAS governance board)"),
    ("HMAS-MESSAGE-APPROVAL", "Cross-branch message approval", "hmas-governance", "PROPOSED-ADDITION (APPROVAL_RESOLVE nonce)"),
    ("STATE-RECONNECT", "Reconnect state", "states-and-refusals", "PROPOSED-ADDITION (truth state)"),
    ("STATE-KILL-CONFIRM", "Kill confirmation", "states-and-refusals", "PROPOSED-ADDITION (truth state)"),
    ("STATE-REPLAY", "Replay read-only", "states-and-refusals", "HiveRouter-Target-Spec §3"),
    ("STATE-DIFF-GATED", "DIFF surface explanation", "states-and-refusals", "DECISION-GATED / no frozen DIFF command"),
    ("CHAT-VARIANT-400x480", "Chat core app variant 400x480", "motion-and-live", "PROPOSED-ADDITION (current app variant; board 07 stays 420x500)"),
    ("MOTION-AMBIENT", "Honeycomb ambient glow sweep (18s linear, PROPOSED)", "motion-and-live", "PROPOSED-ADDITION (motion proposal)"),
    ("MOTION-SHEEN", "5-lobe hive sheen (6s ease-in-out, PROPOSED)", "motion-and-live", "PROPOSED-ADDITION (motion proposal)"),
    ("MOTION-BEE-FLOW", "Bee flow with arrowheads (2.4s cubic-bezier, PROPOSED)", "motion-and-live", "PROPOSED-ADDITION (motion proposal)"),
    ("MOTION-PULSE", "Running state pulse (1.6s, PROPOSED)", "motion-and-live", "PROPOSED-ADDITION (motion proposal)"),
    ("MOTION-SPINNER", "Working-bee spinner (1.2s linear, PROPOSED)", "motion-and-live", "PROPOSED-ADDITION (motion proposal)"),
    ("MOTION-RISE-HOVER", "Node rise/hover (240ms / 120ms, PROPOSED)", "motion-and-live", "PROPOSED-ADDITION (motion proposal)"),
    ("MOTION-FOCUS-RING", "Focus ring ink-700 2px (160ms, PROPOSED)", "motion-and-live", "PROPOSED-ADDITION (motion proposal)"),
    ("MOTION-REDUCED", "Reduced-motion fallback (PROPOSED)", "motion-and-live", "PROPOSED-ADDITION (motion proposal)"),
]


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def rel(path):
    return os.path.relpath(path, REPO).replace("\\", "/")


def png_size(path):
    with Image.open(path) as im:
        return list(im.size)


def run_cli(argv):
    proc = subprocess.run(argv, capture_output=True, text=True, encoding="utf-8",
                          errors="replace")
    if proc.returncode != 0:
        raise RuntimeError("CLI failed (%s): %s %s" % (argv, proc.stdout[-2000:],
                                                       proc.stderr[-2000:]))
    return proc.stdout


def info():
    return json.loads(run_cli([CLI, "info", DESIGN, "--json"]))


def export_svgs():
    os.makedirs(ASSETS, exist_ok=True)
    exported = []
    for code, frame_name, slug, w, h in FRAMES:
        out = os.path.join(ASSETS, "hivemind-%s.svg" % slug)
        run_cli([CLI, "export", DESIGN, "-o", out, "--format", "svg",
                 "--node", frame_name])
        if not (os.path.isfile(out) and os.path.getsize(out) > 0):
            raise RuntimeError("empty SVG export: %s" % out)
        exported.append(out)
    return exported


def copy_pngs():
    os.makedirs(ASSETS, exist_ok=True)
    copied = []
    for code, frame_name, slug, w, h in FRAMES:
        for suffix in ("", "@2x"):
            src = os.path.join(HM, "exports", "hivemind-%s%s.png" % (slug, suffix))
            if not os.path.isfile(src):
                raise RuntimeError("missing generated export: %s" % src)
            dst = os.path.join(ASSETS, "hivemind-%s%s.png" % (slug, suffix))
            shutil.copyfile(src, dst)
            copied.append(dst)
    return copied


def source_evidence():
    pairs = []
    root = os.path.join(HM, "source-evidence")
    for name in ("build_hive_style.py", "HiveRouter-Style-System.figcraft",
                 "base.figcraft", "HiveRouter-Style-Kit.md",
                 "FIGCRAFT-COMMAND-ANALYSIS.md",
                 r"SPEC\HIVEROUTER-TARGET-SPEC.md"):
        src = os.path.join(root, name)
        external = os.path.join(SOURCE, name)
        pairs.append({"path": rel(src), "sha256": sha256(src),
                      "source_path": "HiveRouter/design/hive-style/" + name.replace("\\", "/"),
                      "source_sha256": sha256(external),
                      "preserved": sha256(src) == sha256(external)})
    export_dir = os.path.join(root, "exports")
    for name in sorted(os.listdir(export_dir)):
        src = os.path.join(export_dir, name)
        external = os.path.join(SOURCE, "exports", name)
        pairs.append({"path": rel(src), "sha256": sha256(src),
                      "source_path": "HiveRouter/design/hive-style/exports/" + name,
                      "source_sha256": sha256(external),
                      "preserved": sha256(src) == sha256(external)})
    return pairs


def write_episodes():
    os.makedirs(EPISODES, exist_ok=True)
    mapping = [("01_phase1_nodes.json", "commands_phase1.json"),
               ("02_phase2_tokens_exports.json", "commands_phase2.json")]
    rows = []
    for episode, source in mapping:
        src = os.path.join(HERE, source)
        dst = os.path.join(EPISODES, episode)
        shutil.copyfile(src, dst)
        data = json.load(open(src, encoding="utf-8"))
        rows.append({
            "episode": episode,
            "path": rel(dst),
            "sha256": sha256(dst),
            "commands": len(data),
            "command_ids": sorted({command["command"] for command in data}),
        })
    ids = sorted({command for row in rows for command in row["command_ids"]})
    episodes = {
        "schema": "hivemind.figcraft-episodes.v1",
        "producer": "FigCraft-HiveMind",
        "generated_at": datetime_iso(),
        "source_builder": "docs/ui/figcraft/HiveMind/build/build_hive_style.py",
        "command_ids_used": ids,
        "command_id_count": len(ids),
        "episodes": rows,
        "rejected_syntax_guesses": [],
        "failures": [],
        "notes": [
            "Phase 1 creates 11 artboards + all nodes; phase 2 creates native "
            "paint/text/effect styles, 21 colour variables and the 11 board exports.",
            "Builder is the authoritative HiveRouter style-system builder copied "
            "into this repo; patch = rename to HiveMind + three 1440x900 boards "
            "(08 HMAS hierarchy, 09 HMAS governance, 10 states & refusals).",
        ],
    }
    path = os.path.join(HM, "HiveMind-EPISODES.json")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(episodes, f, ensure_ascii=False, indent=1)
        f.write("\n")
    return episodes


def build_assets(info_json, evidence, episodes, phase1_commands, phase2_commands):
    design_sha = sha256(DESIGN)
    exports = {}
    for code, frame_name, slug, w, h in FRAMES:
        svg = os.path.join(ASSETS, "hivemind-%s.svg" % slug)
        png = os.path.join(ASSETS, "hivemind-%s.png" % slug)
        png2 = os.path.join(ASSETS, "hivemind-%s@2x.png" % slug)
        exports[slug] = {
            "frame": frame_name,
            "svg": rel(svg), "png": rel(png), "png_2x": rel(png2),
            "sha256_svg": sha256(svg), "sha256_png": sha256(png),
            "sha256_png_2x": sha256(png2),
            "svg_bytes": os.path.getsize(svg), "png_bytes": os.path.getsize(png),
            "png_2x_bytes": os.path.getsize(png2),
            "dimensions": {"board": [w, h], "svg": "vector",
                           "png": png_size(png), "png_2x": png_size(png2)},
        }
    board_by_slug = {}
    for code, frame_name, slug, w, h in FRAMES:
        board_by_slug[slug] = {
            "code": code, "frame": frame_name, "slug": slug,
            "width": w, "height": h,
            "export": "hivemind-%s.png" % slug,
        }
    surfaces = []
    for sid, name, slug, provenance in SURFACES:
        export = exports[slug]
        surfaces.append({
            "surface_id": sid,
            "owner": "figcraft-hivemind",
            "name": name,
            "board": board_by_slug[slug]["frame"],
            "export_svg": export["svg"],
            "export_png": export["png"],
            "export_png_2x": export["png_2x"],
            "sha256_svg": export["sha256_svg"],
            "sha256_png": export["sha256_png"],
            "dimensions": export["dimensions"],
            "provenance": provenance,
        })
    return {
        "schema": "hivemind.figcraft-assets.v1",
        "producer": "FigCraft-HiveMind",
        "generated_at": datetime_iso(),
        "source": {
            "authoritative_set": "HiveRouter/design/hive-style (external; read-only)",
            "preserved_evidence": "docs/ui/figcraft/HiveMind/source-evidence/",
            "renamed": "user-facing HiveRouter/HIVEROUTER -> HiveMind/HIVEMIND",
            "a_b_unchanged": True,
            "motion_spec": "board 11 durations/easing are PROPOSED, not measured",
            "card_anatomy": "board 05 refined to exact spatial-system card anatomy",
            "chat_variants": "board 07 stays 420x500; board 11 shows app variant 400x480",
        },
        "tool": {
            "name": "figcraft-cli",
            "commands": 597,
            "info": {"name": info_json.get("name"),
                     "nodes": info_json.get("nodes"),
                     "styles": info_json.get("styles"),
                     "variables": info_json.get("variables"),
                     "missingFonts": info_json.get("missingFonts")},
        },
        "design_file": {
            "path": rel(DESIGN), "sha256": design_sha,
            "bytes": os.path.getsize(DESIGN),
            "frames": len(FRAMES),
            "nodes": info_json.get("nodes"),
            "pages": info_json.get("pages"),
        },
        "boards": [board_by_slug[slug] for _, _, slug, _, _ in FRAMES],
        "exports": exports,
        "surfaces": surfaces,
        "episodes": {
            "manifest": rel(os.path.join(HM, "HiveMind-EPISODES.json")),
            "items": episodes["episodes"],
            "command_id_count": episodes["command_id_count"],
        },
        "source_evidence": evidence,
        "build": {
            "phase1_commands": phase1_commands,
            "phase2_commands": phase2_commands,
            "run_logs": ["docs/ui/figcraft/HiveMind/build/run_phase1.log",
                         "docs/ui/figcraft/HiveMind/build/run_phase2.log"],
            "commands": [
                "python build_hive_style.py 1",
                "figcraft-cli run commands_phase1.json --out base.figcraft",
                "python build_hive_style.py 2",
                "figcraft-cli run commands_phase2.json --in base.figcraft",
                "python publish_hivemind.py",
            ],
        },
    }


def datetime_iso():
    return datetime.now(timezone.utc).astimezone().isoformat(timespec="seconds")


def update_index(manifest):
    with open(INDEX, encoding="utf-8") as f:
        index = json.load(f)
    index["source_manifests"]["HiveMind"] = {
        "path": rel(MANIFEST),
        "sha256": sha256(MANIFEST).upper(),
    }
    index["design_files"]["HiveMind"] = {
        "path": rel(DESIGN),
        "sha256": sha256(DESIGN).upper(),
    }
    keep = [row for row in index["assets"] if row.get("owner") != "figcraft-hivemind"]
    rows = []
    for surface in manifest["surfaces"]:
        rows.append({
            "surface_id": surface["surface_id"],
            "owner": "figcraft-hivemind",
            "name": surface["name"],
            "page": surface["board"],
            "node_id": None,
            "export_svg": surface["export_svg"],
            "export_png": surface["export_png"],
            "export_png_2x": surface["export_png_2x"],
            "sha256_svg": surface["sha256_svg"],
            "sha256_png": surface["sha256_png"],
            "dimensions": surface["dimensions"],
            "provenance_pdf_lines": surface["provenance"],
            "set": "HiveMind",
        })
    index["assets"] = keep + rows
    index["counts"]["assets"] = len(index["assets"])
    index["counts"]["surfaces"] = len({row["surface_id"] for row in index["assets"]})
    index["generated_at"] = datetime_iso()
    index["generated_by"] = (
        "Lead acceptance (canonical alias of A/B manifests; no pixels copied) + "
        "FigCraft-HiveMind set appended; A/B aliases retained byte-identical"
    )
    with open(INDEX, "w", encoding="utf-8", newline="\n") as f:
        json.dump(index, f, ensure_ascii=False, indent=2)
        f.write("\n")
    return index


def write_readme():
    lines = [
        "# HiveMind FigCraft design set (authoritative spatial style system)",
        "",
        "Source: `HiveRouter/design/hive-style` (external repo, read-only; copied evidence in",
        "`source-evidence/`). User-facing HiveRouter/HIVEROUTER text and file names are renamed",
        "to HiveMind; A/B under `docs/ui/figcraft/A|B` are unchanged.",
        "",
        "## Build (deterministic FigCraft CLI, two transactional phases)",
        "```",
        "cd docs/ui/figcraft/HiveMind/build",
        "python build_hive_style.py 1",
        "figcraft-cli run commands_phase1.json --out base.figcraft",
        "python build_hive_style.py 2",
        "figcraft-cli run commands_phase2.json --in base.figcraft",
        "python publish_hivemind.py",
        "python verify_hivemind.py",
        "```",
        "`build_hive_style.py` is the source builder patched for HiveMind (rename",
        "+ HMAS/state boards 08-10 + motion board 11 + exact-anatomy refinement of",
        "board 05). `HiveMind-Style-System.figcraft` is the regenerated design file;",
        "exports live in `exports/` (generated) and the canonical copies in",
        "`docs/ui/figcraft/ASSETS/hivemind-*.{png,svg}`.",
        "",
        "## Artboards",
        "| # | frame | board size | export |",
        "|---|---|---|---|",
    ]
    for code, frame, slug, w, h in FRAMES:
        lines.append("| %s | %s | %dx%d | `hivemind-%s.png` |" % (code, frame, w, h, slug))
    lines += [
        "",
        "## Provenance / claims",
        "- Source style kit + target spec + original exports are preserved byte-identically",
        "  under `source-evidence/` (see `HiveMind-SOURCE-EVIDENCE.json`).",
        "- Episode command scripts and their hashes are in `HiveMind-EPISODES.json`",
        "  and `episodes/` (phase 1 nodes, phase 2 styles/variables/exports).",
        "- Token palette is the source's documented palette (no invented swatches); the",
        "  regenerated artboards reuse it.",
        "- HMAS and state/refusal boards are PROPOSED-ADDITION surfaces on top of the",
        "  authoritative style system; board 07 keeps the authoritative composition",
        "  (52px topbar, 72px rail, 240px library, canvas x=312, chat 420x500 at 620,196,",
        "  HUD 460x44, minimap 220x150, SuperAgent 420x150, WorkerAgent 320x96,",
        "  ToolAgent 420x240, crimson error card).",
        "- Board 05 refines the exact card anatomy annotations (Super 420x150 header 40 +",
        "  68x18 pill; Worker 320x96; Tool 420x240 with 132px dark result area; Error 320x110).",
        "- Board 11 motion/live is a design proposal: durations/easing are PROPOSED, not",
        "  measured; no component behaviour is frozen. It shows the current app chat variant",
        "  400x480 while board 07 remains the authoritative 420x500.",
    ]
    with open(os.path.join(HM, "README.md"), "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")


def main():
    if not os.path.isfile(DESIGN):
        raise SystemExit("missing %s; run the two FigCraft phases first" % DESIGN)
    info_json = info()
    # remove only known stale exports from the earlier HMAS board split; never
    # delete unrelated ASSETS files (e.g. shared contact sheets)
    stale = {"hivemind-hmas.svg", "hivemind-hmas.png", "hivemind-hmas@2x.png"}
    if os.path.isdir(ASSETS):
        for name in stale & set(os.listdir(ASSETS)):
            os.remove(os.path.join(ASSETS, name))
    svgs = export_svgs()
    pngs = copy_pngs()
    evidence = source_evidence()
    episodes = write_episodes()
    phase1 = json.load(open(os.path.join(HERE, "commands_phase1.json"),
                            encoding="utf-8"))
    phase2 = json.load(open(os.path.join(HERE, "commands_phase2.json"),
                            encoding="utf-8"))
    manifest = build_assets(info_json, evidence, episodes, len(phase1), len(phase2))
    manifest_path = MANIFEST
    with open(manifest_path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(manifest, f, ensure_ascii=False, indent=1)
        f.write("\n")
    with open(EVIDENCE, "w", encoding="utf-8", newline="\n") as f:
        json.dump({"schema": "hivemind.figcraft-source-evidence.v1",
                   "source": "HiveRouter/design/hive-style (external; read-only)",
                   "generated_at": datetime_iso(), "files": evidence}, f,
                  ensure_ascii=False, indent=1)
        f.write("\n")
    index = update_index(manifest)
    write_readme()
    print("svg exports:", len(svgs))
    print("png copies:", len(pngs))
    print("episodes:", len(episodes["episodes"]),
          "commands:", episodes["command_id_count"], "ids")
    print("manifest:", rel(manifest_path), sha256(manifest_path))
    print("index assets:", index["counts"]["assets"],
          "surfaces:", index["counts"]["surfaces"])
    print("evidence preserved:", sum(1 for e in evidence if e["preserved"]),
          "/", len(evidence))
    print("name:", info_json.get("name"), "nodes:", info_json.get("nodes"),
          "missingFonts:", info_json.get("missingFonts"))


if __name__ == "__main__":
    main()
