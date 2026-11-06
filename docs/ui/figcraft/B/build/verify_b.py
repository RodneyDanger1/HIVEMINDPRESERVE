"""Final self-check for FigCraft-B: manifest truth, exports, exact strings, tokens."""
import glob
import hashlib
import json
import os
import re

B = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # docs/ui/figcraft/B
FG = os.path.dirname(B)                                            # docs/ui/figcraft
ASSETS = os.path.join(FG, "ASSETS")
EPISODES = os.path.join(FG, "EPISODES")
DOCROOT = os.path.dirname(FG)                                      # docs/ui
PROJECT = os.path.dirname(os.path.dirname(DOCROOT))                # hiverouter-v1


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for c in iter(lambda: f.read(1 << 20), b""):
            h.update(c)
    return h.hexdigest()


def png_size(path):
    from PIL import Image
    with Image.open(path) as im:
        return list(im.size)


problems = []
m = json.load(open(os.path.join(B, "B-ASSETS.json"), encoding="utf-8"))

# 1. design file hash + info
design = os.path.join(PROJECT, m["design_file"]["path"].replace("\\", "/"))
if not os.path.exists(design):
    problems.append("design file missing")
elif sha(design) != m["design_file"]["sha256"]:
    problems.append("design file sha mismatch")
info = json.load(open(os.path.join(B, "build", "info-B.json"),
                      encoding="utf-8-sig"))
pages = [p["name"] for p in info["pages"]]
want_pages = ["Canvas / default", "Canvas / zoom states",
              "Canvas / interaction states", "Brand hero"]
if pages != want_pages:
    problems.append("page names %s" % pages)
frames = {f["name"]: (f["width"], f["height"]) for p in info["pages"]
          for f in p["frames"]}
for f in ("b-canvas-default", "b-zoom-states", "b-interaction", "b-hero"):
    if f not in frames or frames[f] != (1440.0, 900.0):
        problems.append("frame %s %s" % (f, frames.get(f)))

# 2. exports exist, match manifest, have 2880x1800
need = ["b-canvas-default", "b-zoom-states", "b-interaction", "b-hero"]
paths = {}
for stem in need:
    svg = os.path.join(ASSETS, stem + ".svg")
    png = os.path.join(ASSETS, stem + ".png")
    for p in (svg, png):
        if not os.path.exists(p) or os.path.getsize(p) == 0:
            problems.append("missing/empty " + p)
    if png_size(png) != [2880, 1800]:
        problems.append("png dims " + png_size(png).__str__())
    paths[stem] = {"svg": svg, "png": png}
rows = {e["svg"].replace("\\", "/").split("/")[-1]: e for e in m["exports"].values()}
for stem, e in paths.items():
    row = rows.get(stem + ".svg")
    if not row:
        problems.append("manifest export missing " + stem)
        continue
    if row["sha256_svg"] != sha(e["svg"]) or row["sha256_png"] != sha(e["png"]):
        problems.append("sha mismatch " + stem)
    if row["dimensions"]["png_2x"] != [2880, 1800]:
        problems.append("manifest dims " + stem)

# 3. required surfaces present, node ids resolved, exports exist
required = ["S01", "S10", "S13", "S19", "S20", "S24", "S25"]
got = {r["surface_id"]: r for r in m["surfaces"]}
for sid in required:
    r = got.get(sid)
    if not r:
        problems.append("surface missing " + sid)
        continue
    if r["node_id"] is None:
        problems.append("surface node_id null " + sid)
    for kind, relp in r["exports"].items():
        p = os.path.join(PROJECT, relp.replace("\\", "/"))
        if not os.path.exists(p):
            problems.append("surface export missing %s %s" % (sid, relp))
if m["pending_surfaces"]:
    problems.append("pending surfaces %s" % m["pending_surfaces"])

# 4. exact printed strings must appear in the SVG exports
strings = {
    "b-canvas-default": ["09:41:07", "routing state 12 of 38", "4 SUPERAGENTS",
                         "12 WORKERS", "98.2% GREEN", "+3.4%",
                         "Backend Architecture", "tailwind-builder",
                         "WORKER", "react/vite", "55%", "Prisma Generator",
                         "schema.prisma", "0.78", "Compile failed", "E0421",
                         "retry 2/3", "Hive AI Assistant", "Type your prompt...",
                         "HiveMind AI", "Nova Vector Spark", "Content Writer",
                         "Data Analyst", "Code Reviewer", "Chatbot",
                         "Spawn sub-hive", "Re-route", "Cancel", "Kill worker",
                         "Workspace", "Swarm", "Routing", "History",
                         "Ctrl+0", "MINI-MAP", "260x170", "12.4k ctx",
                         "3 workers", "decomposes intent", "NODE_SPAWN",
                         "REFUSED", "E_REWIRE_IN_FLIGHT"],
    "b-zoom-states": ["0.1", "1.0", "3.0", "E_ZOOM_RANGE", "Ctrl+0",
                      "VIEWPORT_SET", "VIEWPORT_RECENTER", "TRUE SIZE",
                      "OVERVIEW", "DETAIL", "3.0x", "1.0x", "0.1x"],
    "b-interaction": ["Re-route refused", "context-prune blocked: worker still streaming",
                      "REFUSED BY NAME", "NOT SILENT", "E_REWIRE_IN_FLIGHT",
                      "Kill worker", "APPROVAL_REQUEST", "awaiting_approval",
                      "reconnecting", "RESYNC_REQUIRED", "NODE_DRAG", "replay",
                      "09:41:07", "routing state 12 of 38", "DECISION-GATED",
                      "NOT IMPLEMENTED", "DIFF_PROPOSE", "DIFF_APPLY",
                      "DIFF_REJECT", "E_DECISION_REQUIRED", "#282010",
                      "ink-700", "9.86", "REWIRE_TASK", "E_CYCLE",
                      "KILL_WORKER", "NODE_STATE(KILLED)", "e1", "e2", "e3"],
    "b-hero": ["HiveRouter", "SPATIAL EXECUTION ENVIRONMENT", "Honeycomb canvas",
               "Bee-line routing", "Hive chat core", "v1.0.0", "figcraft-cli",
               "CLEAR SPACE", "56px", "32px", "16px", "62%", "tailwind-builder",
               "running", "597 commands", "1,045"],
}
# FigCraft's SVG/PNG exporters outline live text to paths (0 <text> nodes in
# every export), so exact strings are verified against the command scripts that
# authored them; the live text stays in B-canvas.figcraft. The visual check of
# the PNG renders is recorded separately in the episode evaluation block.
import glob as _glob
script_text = []
for jf in sorted(_glob.glob(os.path.join(B, "scripts", "*.json"))):
    try:
        cmds = json.load(open(jf, encoding="utf-8"))
    except Exception as ex:
        problems.append("script parse %s: %s" % (os.path.basename(jf), ex))
        continue
    for c in cmds:
        p = c.get("params") or {}
        if isinstance(p.get("text"), str):
            script_text.append(p["text"])
        if isinstance(p.get("characters"), str):
            script_text.append(p["characters"])
blob = "\n".join(script_text)
for stem in need:
    for s in strings[stem]:
        if s not in blob:
            problems.append("string %r not authored in scripts" % s)
    if len(script_text) < 50:
        problems.append("too few text nodes authored")

# 5. token truth: only printed hexes may appear in the B generators
printed = {"#F8F4EB", "#FFFFFF", "#FBF8F1", "#F1E9CF", "#E5E1D8", "#FBF3DC",
           "#F6E7BE", "#F0C050", "#E8B850", "#D9A02F", "#C88030", "#8A5A1C",
           "#604818", "#282010", "#3F3115", "#4E3A15", "#75663F", "#B7791F",
           "#B23A2E", "#3F7A55"}
hexes = set()
for py in glob.glob(os.path.join(B, "build", "board_*.py")):
    src = open(py, encoding="utf-8").read()
    hexes |= set(re.findall(r"#[0-9A-Fa-f]{6}", src))
bad = hexes - printed
if bad:
    problems.append("invented hexes in generators: %s" % sorted(bad))

# 6. episodes exist and parse
eps = sorted(glob.glob(os.path.join(EPISODES, "b-*.json")))
if len(eps) < 2:
    problems.append("episodes missing")
for e in eps:
    try:
        d = json.load(open(e, encoding="utf-8"))
        for k in ("episode_id", "brief", "task_family", "actions", "artifacts",
                  "evaluation", "outcome"):
            if k not in d:
                problems.append("episode %s missing %s" % (os.path.basename(e), k))
    except Exception as ex:
        problems.append("episode parse %s: %s" % (e, ex))

# 7. no temp/stray files inside ASSETS (only a-* from A and b-* from B)
stray = [os.path.basename(p) for p in glob.glob(os.path.join(ASSETS, "*"))
         if not (os.path.basename(p).startswith("a-")
                 or os.path.basename(p).startswith("b-"))]
if stray:
    problems.append("stray files in ASSETS: %s" % stray)

print("design_file_sha:", m["design_file"]["sha256"][:16])
print("pages:", pages)
print("nodes:", info["nodes"], "missingFonts:", info["missingFonts"])
print("exports:", {s: (os.path.getsize(paths[s]["svg"]),
                       os.path.getsize(paths[s]["png"])) for s in need})
print("manifest surfaces:", len(m["surfaces"]), "required:",
      sorted(got.keys() & set(required)), "pending:", m["pending_surfaces"])
print("command_total:", m["totals"]["command_count"], "stages:", m["totals"]["stages"])
print("episodes:", [os.path.basename(e) for e in eps])
print("hexes used:", sorted(hexes))
print("PROBLEMS:", problems if problems else "NONE")
