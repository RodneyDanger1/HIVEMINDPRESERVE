#!/usr/bin/env python3
"""Publish + verify the proposed chat-dock/Help board (13).

Reads the deterministic command script, re-runs the installed FigCraft CLI to
export SVG/PNG from the saved design file, verifies the artwork and writes
ChatDock-ASSETS.json. It never marks the board as watchdog-audited or served.

Exit 0 only when every check passes. Prints one PASS/FAIL line per check.
"""
import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from PIL import Image

CLI = r"C:\Program Files\FigCraft\figcraft-cli.exe"
HERE = Path(__file__).resolve().parent
CHATDOCK = HERE.parent
REPO = HERE.parents[4]
DESIGN = CHATDOCK / "ChatDock-Help.figcraft"
COMMANDS = HERE / "commands.json"
EXPORTS = CHATDOCK / "exports"
SVG = EXPORTS / "chatdock-help.svg"
PNG = EXPORTS / "chatdock-help.png"
PNG2 = EXPORTS / "chatdock-help@2x.png"
MANIFEST = CHATDOCK / "ChatDock-ASSETS.json"

FRAME = "13 · Chat Dock & Help"
WIDTH, HEIGHT = 1440, 1800
SOURCE_FILES = [
    "web/src/chat/ChatCore.tsx",
    "web/src/chat/chatDock.ts",
    "web/src/panels/HelpPanel.tsx",
    "web/src/tokens/tokens.ts",
    "ledgers/visual-dock-8099/VISUALS-FINDINGS.md",
]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def rel(path: Path) -> str:
    return str(path.relative_to(REPO)).replace("\\", "/")


def run(argv):
    proc = subprocess.run(argv, capture_output=True, text=True, encoding="utf-8",
                          errors="replace")
    if proc.returncode != 0:
        raise RuntimeError("CLI failed (%s): %s %s" % (argv, proc.stdout[-2000:],
                                                       proc.stderr[-2000:]))
    return proc.stdout


def info():
    return json.loads(run([CLI, "info", str(DESIGN), "--json"]))


def png_size(path: Path):
    with Image.open(path) as image:
        return list(image.size)


def main() -> int:
    checks = []

    def check(name, ok, detail):
        checks.append({"check": name, "status": "PASS" if ok else "FAIL",
                       "detail": detail})
        print("%-4s %-52s %s" % ("PASS" if ok else "FAIL", name, detail))
        return ok

    if not DESIGN.is_file():
        check("design file exists", False, rel(DESIGN))
        return finish(checks)
    if not COMMANDS.is_file():
        check("command script exists", False, rel(COMMANDS))
        return finish(checks)
    commands = json.loads(COMMANDS.read_text(encoding="utf-8"))

    document = info()
    frames = [frame for page in document.get("pages", [])
              for frame in page.get("frames", [])]
    check("one frame named %r" % FRAME,
          [frame.get("name") for frame in frames] == [FRAME],
          str([frame.get("name") for frame in frames]))
    frame = frames[0] if frames else {}
    check("frame is %dx%d" % (WIDTH, HEIGHT),
          frame.get("width") == WIDTH and frame.get("height") == HEIGHT,
          "%sx%s" % (frame.get("width"), frame.get("height")))
    check("no missing fonts", document.get("missingFonts") == [],
          str(document.get("missingFonts")))
    check("command script is non-empty", len(commands) > 0, "%d commands" % len(commands))
    command_ids = sorted({entry.get("command") for entry in commands})
    check("only verified CLI commands", set(command_ids) <= {
        "file.new", "frame.create", "node.create", "text.create", "file.saveAs"},
        ",".join(command_ids))

    EXPORTS.mkdir(parents=True, exist_ok=True)
    run([CLI, "export", str(DESIGN), "-o", str(SVG), "--format", "svg",
         "--node", FRAME])
    run([CLI, "export", str(DESIGN), "-o", str(PNG), "--format", "png",
         "--node", FRAME, "--scale", "1"])
    run([CLI, "export", str(DESIGN), "-o", str(PNG2), "--format", "png",
         "--node", FRAME, "--scale", "2"])
    check("SVG exported non-empty", SVG.is_file() and SVG.stat().st_size > 0,
          "%d bytes" % (SVG.stat().st_size if SVG.is_file() else 0))
    check("PNG exported non-empty", PNG.is_file() and PNG.stat().st_size > 0,
          "%d bytes" % (PNG.stat().st_size if PNG.is_file() else 0))
    check("@2x PNG exported non-empty", PNG2.is_file() and PNG2.stat().st_size > 0,
          "%d bytes" % (PNG2.stat().st_size if PNG2.is_file() else 0))
    check("PNG dimensions 1440x1800", png_size(PNG) == [WIDTH, HEIGHT],
          str(png_size(PNG)))
    check("@2x PNG dimensions 2880x3600", png_size(PNG2) == [WIDTH * 2, HEIGHT * 2],
          str(png_size(PNG2)))
    svg_head = SVG.read_text(encoding="utf-8", errors="replace")[:400]
    check("SVG declares 1440x1800", 'width="1440"' in svg_head and 'height="1800"' in svg_head,
          svg_head.split(">", 1)[0][:180])
    svg_text = SVG.read_text(encoding="utf-8", errors="replace")
    check("no legacy HiveRouter brand in SVG", "HiveRouter" not in svg_text,
          "HiveRouter occurrences=%d" % svg_text.count("HiveRouter"))

    board = {
        "id": "chatdock-help",
        "owner": "figcraft-hivemind",
        "title": FRAME,
        "width": WIDTH,
        "height": HEIGHT,
        "source_svg": rel(SVG),
        "source_png": rel(PNG),
        "source_png_2x": rel(PNG2),
        "sha256_svg": sha(SVG),
        "sha256_png": sha(PNG),
        "sha256_png_2x": sha(PNG2),
        "bytes": {"svg": SVG.stat().st_size, "png": PNG.stat().st_size,
                  "png_2x": PNG2.stat().st_size},
        "dimensions": {"board": [WIDTH, HEIGHT], "svg": "vector",
                       "png": png_size(PNG), "png_2x": png_size(PNG2)},
        "surfaces": [
            {"surfaceId": "NEW-CHAT-DOCK",
             "name": "Full-width bottom chat dock (1128 px at 1440)"},
            {"surfaceId": "NEW-HELP",
             "name": "Help nav destination: tutorials + feature explanations"},
        ],
    }
    manifest = {
        "schema": "hivemind.figcraft-proposal-assets.v1",
        "producer": "FigCraft-ChatDock",
        "generated_at": datetime.now(timezone.utc).astimezone().isoformat(
            timespec="seconds"),
        "status": "PROPOSED",
        "watchdog_audited": False,
        "served": False,
        "reason_not_served": (
            "The served FigCraft catalog is the accepted 21-board set covered by "
            "web/tests/figcraftAssets.test.ts; this proposal must be accepted and "
            "that test updated before it can be served. It is real FigCraft "
            "evidence, but it is not watchdog-audited design truth."
        ),
        "design_file": {
            "path": rel(DESIGN),
            "sha256": sha(DESIGN),
            "bytes": DESIGN.stat().st_size,
            "frame": frame.get("name"),
            "width": frame.get("width"),
            "height": frame.get("height"),
            "nodes": document.get("nodes"),
            "missing_fonts": document.get("missingFonts"),
        },
        "tool": {
            "name": "figcraft-cli",
            "version": run([CLI, "--version"]).strip(),
            "command_schema_source": "figcraft-cli commands --json (2026-11-13)",
            "verified_commands": command_ids,
        },
        "commands": {
            "path": rel(COMMANDS),
            "sha256": sha(COMMANDS),
            "count": len(commands),
        },
        "board": board,
        "source_refs": [
            {"path": "web/src/chat/chatDock.ts",
             "claim": "DEFAULT_CHAT_HEIGHT=560, MIN=420, MAX=760, tray=240"},
            {"path": "web/src/chat/ChatCore.tsx",
             "claim": "w-full dock, 64x12 grip, CONTROLS / STATE & SETTINGS / CLEAR / HIDE footer"},
            {"path": "web/src/panels/HelpPanel.tsx",
             "claim": "Help tutorials, feature explanations and buttons"},
            {"path": "web/src/tokens/tokens.ts",
             "claim": "20 printed swatches; LAYOUT header 52, rail 72, library 240"},
            {"path": "ledgers/visual-dock-8099/VISUALS-FINDINGS.md",
             "claim": "dock/resize defects observed in the running UI"},
            {"path": "commit e83a444",
             "claim": "new UI: full-width dock, help tab"},
        ],
        "source_files": [
            {"path": path, "sha256": sha(REPO / path)}
            for path in SOURCE_FILES if (REPO / path).is_file()
        ],
        "build": {
            "steps": [
                "python docs/ui/figcraft/ChatDock/build/build_chatdock.py",
                "figcraft-cli run docs/ui/figcraft/ChatDock/build/commands.json",
                "python docs/ui/figcraft/ChatDock/build/publish_chatdock.py",
            ],
            "cli": CLI,
        },
        "notes": [
            "One 1440x1800 proposal frame with two scenes: the new full-width chat dock and the Help nav destination.",
            "Values are read from the current UI source and tokens; no runtime screenshot is embedded and no pixel fidelity is claimed.",
            "The board is labelled PROPOSED on its face and is not in ASSETS-INDEX.json / the served catalog.",
        ],
        "checks": checks,
    }
    MANIFEST.write_text(json.dumps(manifest, ensure_ascii=False, indent=1) + "\n",
                        encoding="utf-8", newline="\n")
    check("manifest written", MANIFEST.is_file(), rel(MANIFEST))
    return finish(checks)


def finish(checks) -> int:
    failed = [entry for entry in checks if entry["status"] == "FAIL"]
    print("status: %s (%d checks, %d failed)" %
          ("PASS" if not failed else "FAIL", len(checks), len(failed)))
    return 0 if not failed else 1


if __name__ == "__main__":
    sys.exit(main())
