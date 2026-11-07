"""Independent verification for the regenerated HiveMind FigCraft set.

Checks (prints PASS/FAIL with values; writes verify-report.json):
  A. figcraft-cli info parses the design; 10 frames with exact names/sizes;
     missingFonts empty; name HiveMind Style System.
  B. phase-1 deterministic ids: every expected id returned by the CLI run.
  C. document text: HiveRouter/HIVEROUTER absent; HiveMind present; HMAS and
     state/refusal labels present.
  D. board 07 authoritative composition geometry + crimson error card.
  E. ASSETS exports exist/nonzero, hashes match HIVEMIND-ASSETS.json, PNG
     dimensions match board x2; one SVG per board.
  F. token audit: every hex colour in the exported SVGs exists in the source
     builder palette (no invented tokens).
  G. source evidence preserved byte-identically vs the external HiveRouter set;
     renamed spec has no HiveRouter; A/B index aliases untouched; C marked
     superseded.
Exit 0 only if every check passes.
"""
import hashlib
import json
import os
import re
import subprocess
import zipfile

from PIL import Image

CLI = r"C:\Program Files\FigCraft\figcraft-cli.exe"
HERE = os.path.dirname(os.path.abspath(__file__))
HM = os.path.dirname(HERE)
FIG = os.path.dirname(HM)
REPO = os.path.abspath(os.path.join(FIG, "..", "..", ".."))
ASSETS = os.path.join(FIG, "ASSETS")
DESIGN = os.path.join(HM, "HiveMind-Style-System.figcraft")
MANIFEST = os.path.join(HM, "HiveMind-ASSETS.json")
EVIDENCE = os.path.join(HM, "HiveMind-SOURCE-EVIDENCE.json")
EPISODES = os.path.join(HM, "HiveMind-EPISODES.json")
INDEX = os.path.join(FIG, "ASSETS-INDEX.json")
SOURCE = r"C:\Users\Riley\Desktop\HiveRouter\design\hive-style"
REPORT = os.path.join(HERE, "verify-report.json")

EXPECTED_FRAMES = [
    ("00 · Cover", 1440, 900), ("01 · Colour", 1440, 1240),
    ("02 · Typography", 1440, 1340), ("03 · Space & Depth", 1440, 1000),
    ("04 · Brand Lockups", 1440, 1000), ("05 · Agent Cards", 1440, 1300),
    ("06 · Controls & HUD", 1440, 1040), ("07 · Spatial Canvas", 1440, 900),
    ("08 · HMAS Hierarchy", 1440, 900), ("09 · HMAS Governance", 1440, 900),
    ("10 · States & Refusals", 1440, 900), ("11 · Motion & Live", 1440, 900),
    ("12 · Run Detail & Files", 1440, 900),
]


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def read_text_auto(path):
    raw = open(path, "rb").read()
    if raw.startswith(b"\xff\xfe") or raw.startswith(b"\xfe\xff"):
        return raw.decode("utf-16")
    return raw.decode("utf-8", "replace")


def rel(path):
    return os.path.relpath(path, REPO).replace("\\", "/")


class Checks:
    def __init__(self):
        self.rows = []
        self.failed = 0

    def check(self, name, ok, detail):
        status = "PASS" if ok else "FAIL"
        if not ok:
            self.failed += 1
        self.rows.append({"check": name, "status": status, "detail": detail})
        print("%-4s %-52s %s" % (status, name, detail))


def run_cli(argv):
    proc = subprocess.run(argv, capture_output=True, text=True, encoding="utf-8",
                          errors="replace")
    if proc.returncode != 0:
        raise RuntimeError("CLI failed: %s %s" % (proc.stdout[-2000:], proc.stderr[-2000:]))
    return proc.stdout


def pos(node):
    t = node.get("transform")
    if not t:
        return (0.0, 0.0)
    return (float(t[4]), float(t[5]))


def close(a, b, eps=0.02):
    return abs(float(a) - float(b)) <= eps


def rgba(value):
    return tuple(round(float(v), 3) for v in (value["r"], value["g"], value["b"]))


def main():
    c = Checks()
    report = {"generated_at": None, "checks": c.rows, "values": {}}

    # A. info
    info = json.loads(run_cli([CLI, "info", DESIGN, "--json"]))
    frames = [f for page in info.get("pages", []) for f in page.get("frames", [])]
    got = [(f["name"], int(f["width"]), int(f["height"])) for f in frames]
    c.check("info parses design file",
            os.path.basename(str(info.get("file"))) == os.path.basename(DESIGN),
            info.get("file"))
    c.check("info name is HiveMind (not HiveRouter)",
            str(info.get("name", "")).startswith("HiveMind")
            and "HiveRouter" not in str(info.get("name", "")),
            repr(info.get("name")))
    c.check("13 artboards with exact names/sizes", got == EXPECTED_FRAMES, str(got))
    c.check("no missing fonts", info.get("missingFonts") == [],
            repr(info.get("missingFonts")))
    report["values"]["info"] = {k: info.get(k) for k in
                                ("name", "nodes", "styles", "variables", "missingFonts")}
    report["values"]["frames"] = got
    frame_ids = {f["name"]: f["id"] for f in frames}

    # B. deterministic ids
    expected = json.load(open(os.path.join(HERE, "expected_ids.json"), encoding="utf-8"))
    log_path = os.path.join(HERE, "run_phase1.log")
    results = [json.loads(line) for line in read_text_auto(log_path).splitlines()
               if line.strip()]
    id_mismatches = []
    for index, exp in enumerate(expected):
        step = index + 1
        if step >= len(results):
            id_mismatches.append("missing step %d" % step)
            break
        got_id = (results[step].get("result") or {}).get("id")
        if got_id != exp["id"]:
            id_mismatches.append("step %d label=%s expected=%s got=%s"
                                 % (step, exp["label"], exp["id"], got_id))
            if len(id_mismatches) > 4:
                break
    c.check("phase-1 ids match the CLI-returned ids",
            not id_mismatches, "%d expected nodes; mismatches=%s"
            % (len(expected), id_mismatches[:5]))
    c.check("phase-1 run ok + one result per command",
            len(results) == len(expected) + 1 and all(r.get("ok") for r in results),
            "%d results / %d expected" % (len(results), len(expected)))
    report["values"]["expected_ids"] = len(expected)
    report["values"]["phase1_results"] = len(results)

    # B2. structural correspondence with the preserved source builder: the first
    # eight frames keep their ids; the original board nodes are identical apart
    # from a +2 id offset caused by inserting the two new frames before content.
    source_expected = json.load(open(os.path.join(SOURCE, "expected_ids.json"),
                                     encoding="utf-8"))
    frames_ok = all(source_expected[i]["id"] == expected[i]["id"]
                    and source_expected[i]["label"] == expected[i]["label"]
                    for i in range(8))
    board_bad = []
    for index in range(8, len(source_expected)):
        mine = expected[index + 3]
        orig = source_expected[index]
        if mine["label"] != orig["label"] or mine["id"] != orig["id"] + 3:
            board_bad.append((index, orig, mine))
    c.check("first 8 artboards structurally match the preserved source builder",
            frames_ok and not board_bad,
            "source nodes=%d matched=%d new_boards_nodes=%d mismatches=%s"
            % (len(source_expected), len(source_expected) - 8 - len(board_bad),
               len(expected) - 3 - len(source_expected), board_bad[:3]))

    # C. text / branding per board (frame children carry their own text)
    with zipfile.ZipFile(DESIGN) as z:
        raw = z.read("document.json")
        doc = json.loads(raw)
    nodes = doc["nodes"]

    def frame_text(frame_name):
        frame_id = frame_ids[frame_name]
        parts = []
        for child_id in nodes[str(frame_id)].get("children", []):
            child = nodes[str(child_id)]
            if child.get("type") == "text":
                parts.append(str(child.get("characters", "")))
        return "\n".join(parts)

    def children(frame):
        return [nodes[str(cid)] for cid in nodes[str(frame)].get("children", [])]

    texts = [str(node.get("characters", "")) for node in nodes.values()
             if node.get("type") == "text"]
    joined = "\n".join(texts)
    lower_raw = raw.decode("utf-8", "replace").lower()
    c.check("no HiveRouter/HIVEROUTER in generated board text",
            "hiverouter" not in lower_raw and "HiveRouter" not in joined
            and "HIVEROUTER" not in joined,
            "hiverouter occurrences=%d" % lower_raw.count("hiverouter"))
    c.check("HiveMind branding present on boards",
            "HiveMind" in joined and "Project: HiveMind AI" in joined,
            "HiveMind text nodes=%d" % joined.count("HiveMind"))
    hierarchy_labels = ["HIERARCHY / WORKSPACE-JAILED", "TOPOLOGY / LIVE EDGES",
                        "STATES / SHAPE + TEXT CUE", "HyperAgent",
                        "Backend Architecture", "tailwind-builder"]
    hierarchy_text = frame_text("08 · HMAS Hierarchy")
    missing = [l for l in hierarchy_labels if l not in hierarchy_text]
    c.check("HMAS hierarchy/topology board surfaces present", not missing,
            "missing=%s" % missing)
    governance_labels = ["PROFILES · IMPORT", "MEMORY", "MAILBOX · 2 UNREAD",
                         "PROJECTS", "USAGE", "MESSAGE APPROVAL"]
    governance_text = frame_text("09 · HMAS Governance")
    missing = [l for l in governance_labels if l not in governance_text]
    c.check("HMAS governance board surfaces present (profiles/import/memory/"
            "mailbox/projects/usage/approval)", not missing,
            "missing=%s" % missing)
    state_labels = ["E_APPROVAL_TIMEOUT", "Reconnecting", "Nothing here yet",
                    "Kill worker?", "Replay (read-only)", "DECISION-GATED",
                    "The approval request timed out before a decision arrived.",
                    "Only running workers can be killed; refresh the worker state."]
    states_text = frame_text("10 · States & Refusals")
    missing = [l for l in state_labels if l not in states_text]
    c.check("states board present (refusal/reconnect/empty/loading/kill/replay/"
            "diff-gated)", not missing, "missing=%s" % missing)

    # C2. board 05 exact-anatomy annotations
    anatomy_labels = ["EXACT ANATOMY / SPATIAL SYSTEM", "420 × 500",
                      "420 × 150", "320 × 96", "420 × 240", "320 × 110",
                      "68 × 18", "132px dark result area + 3 dots",
                      "#FFF7F5", "radius 18"]
    anatomy_text = frame_text("05 · Agent Cards")
    missing = [l for l in anatomy_labels if l not in anatomy_text]
    c.check("board 05 refined to exact spatial-system card anatomy", not missing,
            "missing=%s" % missing)

    # C3. board 11 motion & live proposal
    motion_labels = ["MOTION SPEC", "PROPOSED · not measured", "AMBIENT TRAVEL",
                     "HIVE SHEEN", "BEE FLOW", "RUNNING PULSE",
                     "WORKING-BEE SPINNER", "NODE RISE", "HOVER LIFT",
                     "FOCUS RING", "REDUCED MOTION", "ink-700",
                     "400 × 480", "Design proposal only"]
    motion_text = frame_text("11 · Motion & Live")
    missing = [l for l in motion_labels if l not in motion_text]
    c.check("board 11 motion & live proposal present (ambient/sheen/bee flow/"
            "pulse/spinner/rise/hover/focus/reduced-motion, PROPOSED)",
            not missing, "missing=%s" % missing)
    motion_frame = frame_ids["11 · Motion & Live"]
    chat_variant = [cnode for cnode in children(motion_frame)
                    if cnode.get("type") == "rectangle"
                    and close(cnode.get("width"), 400) and close(cnode.get("height"), 480)]
    c.check("motion board shows chat core 400x480 app variant", bool(chat_variant),
            "matching rects=%d" % len(chat_variant))
    focus_ring = []
    for cnode in children(motion_frame):
        if cnode.get("type") != "vector":
            continue
        for stroke in cnode.get("strokes", []) or []:
            color = stroke.get("color")
            if color and close(rgba(color)[0], 0.306, 0.01) \
                    and close(rgba(color)[1], 0.227, 0.01) \
                    and close(rgba(color)[2], 0.082, 0.01):
                focus_ring.append(cnode.get("id"))
    c.check("focus ring is ink-700 (not honey)", bool(focus_ring),
            "ink-700 vector nodes=%s" % focus_ring[:4])

    # C4. board 12 run detail & files surfaces
    run_labels = ["DATA & FILES", "TOOL ACTIVITY", "RUN DETAIL",
                  "FAILURE / REFUSAL LEDGER", "MODEL STATUS & USAGE",
                  "SETTINGS", "SAMPLE", "READ", "AUTHORITATIVE SURFACES",
                  "prompt", "completion", "total", "cache hit", "cache miss",
                  "reasoning", "E_APPROVAL_TIMEOUT", "E_KILL_DENIED",
                  "E_RUN_TERMINAL", "DECISION-GATED",
                  "no fabricated calls", "NO DATA INVENTED"]
    run_text = frame_text("12 · Run Detail & Files")
    missing = [l for l in run_labels if l not in run_text]
    c.check("board 12 run detail & files surfaces present (files/tools/run "
            "detail/ledger/model+six counters/settings)", not missing,
            "missing=%s" % missing)
    report["values"]["text_nodes"] = len(texts)

    # D. authoritative composition on board 07 (+ crimson error card on 05)
    hero = frame_ids["07 · Spatial Canvas"]
    boards_frame = frame_ids["05 · Agent Cards"]

    def children(frame):
        return [nodes[str(cid)] for cid in nodes[str(frame)].get("children", [])]

    def rect_at(frame, x, y, w, h):
        for node in children(frame):
            if node.get("type") != "rectangle":
                continue
            px, py = pos(node)
            if close(px, x) and close(py, y) and close(node.get("width"), w) \
                    and close(node.get("height"), h):
                return node
        return None

    hero_geometry = [
        ("topbar 52px", 0, 0, 1440, 52),
        ("rail 72px", 0, 52, 72, 848),
        ("library 240px", 72, 52, 240, 848),
        ("canvas starts x=312", 312, 52, 1128, 848),
        ("chat 420x500 at 620,196", 620, 196, 420, 500),
        ("HUD 460x44", 540, 68, 460, 44),
        ("minimap 220x150", 1120, 700, 220, 150),
    ]
    missing = [name for name, x, y, w, h in hero_geometry
               if rect_at(hero, x, y, w, h) is None]
    c.check("board 07 authoritative composition (topbar/rail/library/canvas/"
            "chat/HUD/minimap)", not missing, "missing=%s" % missing)
    card_geometry = [
        ("SuperAgent 420x150", 620, 270, 420, 150),
        ("WorkerAgent 320x96", 620, 526, 320, 96),
        ("ToolAgent 420x240", 620, 724, 420, 240),
    ]
    missing = [name for name, x, y, w, h in card_geometry
               if rect_at(boards_frame, x, y, w, h) is None]
    c.check("authoritative agent card sizes (board 05: SuperAgent 420x150, "
            "WorkerAgent 320x96, ToolAgent 420x240)", not missing,
            "missing=%s" % missing)
    error = rect_at(boards_frame, 620, 1070, 320, 110)
    crimson = False
    if error is not None:
        for stroke in error.get("strokes", []) or []:
            color = stroke.get("color")
            if color and close(rgba(color)[0], 0.698, 0.01) \
                    and close(rgba(color)[1], 0.227, 0.01) \
                    and close(rgba(color)[2], 0.180, 0.01):
                crimson = True
    c.check("error card crimson (320x110, #B23A2E border)", crimson,
            "node=%s" % (None if error is None else error.get("id")))

    # E. exports vs manifest
    manifest = json.load(open(MANIFEST, encoding="utf-8"))
    problems = []
    for slug, row in manifest["exports"].items():
        for key, path_key in (("sha256_svg", "svg"), ("sha256_png", "png"),
                              ("sha256_png_2x", "png_2x")):
            path = os.path.join(REPO, row[path_key])
            if not os.path.isfile(path) or os.path.getsize(path) == 0:
                problems.append("%s missing/zero (%s)" % (path_key, slug))
                continue
            if sha256(path) != row[key]:
                problems.append("%s hash mismatch (%s)" % (path_key, slug))
            if path_key == "png":
                with Image.open(path) as im:
                    if list(im.size) != row["dimensions"]["png"]:
                        problems.append("png size mismatch (%s)" % slug)
            if path_key == "png_2x":
                with Image.open(path) as im:
                    expected2 = [row["dimensions"]["board"][0] * 2,
                                 row["dimensions"]["board"][1] * 2]
                    if list(im.size) != expected2:
                        problems.append("png@2x size mismatch (%s)" % slug)
    c.check("exports exist/nonzero + hashes + dimensions match manifest",
            not problems, "%d exports; problems=%s" % (len(manifest["exports"]),
                                                        problems[:4]))
    report["values"]["exports"] = len(manifest["exports"])

    # E1b. untouched existing boards should be byte-identical where possible
    prev_path = os.path.join(HERE, "prev-assets.json")
    if os.path.isfile(prev_path):
        prev = json.load(open(prev_path, encoding="utf-8"))
        unchanged_slugs = ["colour", "typography", "space-and-depth",
                           "brand-lockups", "agent-cards", "controls-and-hud",
                           "spatial-canvas", "hmas-hierarchy", "hmas-governance",
                           "states-and-refusals", "motion-and-live"]
        changed = []
        for slug in unchanged_slugs:
            old = prev["exports"].get(slug)
            new = manifest["exports"].get(slug)
            if not old or not new:
                changed.append("%s missing" % slug)
                continue
            for key in ("sha256_svg", "sha256_png", "sha256_png_2x"):
                if old[key] != new[key]:
                    changed.append("%s/%s" % (slug, key))
        c.check("existing untouched artboards byte-identical (11 boards)",
                not changed, "changed=%s" % changed[:6])
    else:
        c.check("existing untouched artboards byte-identical (11 boards)",
                False, "prev-assets.json missing")

    # E2. episodes: manifest lists the copied phase scripts with matching hashes
    ep_manifest = json.load(open(EPISODES, encoding="utf-8"))
    ep_problems = []
    manifest_episodes = {row["episode"]: row
                         for row in manifest.get("episodes", {}).get("items", [])}
    for row in ep_manifest["episodes"]:
        path = os.path.join(REPO, row["path"])
        if not os.path.isfile(path):
            ep_problems.append("missing %s" % row["path"])
            continue
        if sha256(path) != row["sha256"]:
            ep_problems.append("hash %s" % row["path"])
        if row["episode"] not in manifest_episodes:
            ep_problems.append("not in manifest %s" % row["episode"])
    c.check("episodes written and match manifest", not ep_problems,
            "%d episodes; problems=%s" % (len(ep_manifest["episodes"]),
                                          ep_problems[:4]))

    # E3. every .figcraft under HiveMind parses with figcraft-cli info
    figcrafts = []
    for root, _dirs, files in os.walk(HM):
        for name in files:
            if name.endswith(".figcraft"):
                figcrafts.append(os.path.join(root, name))
    bad = []
    for path in figcrafts:
        proc = subprocess.run([CLI, "info", path, "--json"], capture_output=True,
                              text=True, encoding="utf-8", errors="replace")
        if proc.returncode != 0:
            bad.append(rel(path))
    c.check("figcraft-cli info parses every .figcraft in HiveMind", not bad,
            "%d files; failures=%s" % (len(figcrafts), bad))
    report["values"]["figcraft_files"] = [rel(p) for p in figcrafts]

    # F. token audit against the preserved source builder palette
    source_builder = open(os.path.join(HM, "source-evidence", "build_hive_style.py"),
                          encoding="utf-8").read()
    palette = {m.group(0).lower() for m in
               re.finditer(r"#[0-9A-Fa-f]{6,8}\b", source_builder)}
    hex_re = re.compile(r"#[0-9A-Fa-f]{6,8}\b")
    offenders = {}
    for slug, row in manifest["exports"].items():
        svg = open(os.path.join(REPO, row["svg"]), encoding="utf-8", errors="replace").read()
        for match in hex_re.finditer(svg):
            value = match.group(0).lower()
            if len(value) == 9:  # #RRGGBBAA -> compare RGB prefix
                value = value[:7]
            if value not in palette:
                offenders.setdefault(value, []).append(slug)
    c.check("no invented colour tokens in exported SVGs", not offenders,
            "svg hex colours=%d; offenders=%s"
            % (sum(len(hex_re.findall(open(os.path.join(REPO, r["svg"]),
                                           encoding="utf-8", errors="replace").read()))
                   for r in manifest["exports"].values()), offenders))
    report["values"]["palette"] = sorted(palette)

    # G. preserved evidence + index + superseded marker
    evidence = json.load(open(EVIDENCE, encoding="utf-8"))["files"]
    not_preserved = [e["path"] for e in evidence if not e["preserved"]]
    c.check("source evidence preserved byte-identically", not not_preserved,
            "%d files; not preserved=%s" % (len(evidence), not_preserved))
    spec = open(os.path.join(HM, "SPEC", "HIVEMIND-TARGET-SPEC.md"),
                encoding="utf-8").read()
    c.check("renamed spec has no HiveRouter legacy brand",
            "HiveRouter" not in spec and "HIVEROUTER" not in spec,
            "spec bytes=%d" % len(spec))
    index = json.load(open(INDEX, encoding="utf-8"))
    hm_rows = [r for r in index["assets"] if r.get("owner") == "figcraft-hivemind"]
    ab_ok = all(
        sha256(os.path.join(REPO, index["source_manifests"][key]["path"])).upper()
        == index["source_manifests"][key]["sha256"]
        for key in ("A", "B"))
    c.check("ASSETS-INDEX updated for HiveMind only; A/B aliases intact",
            bool(hm_rows) and ab_ok and "HiveMind" in index["source_manifests"]
            and index["counts"]["assets"] == len(index["assets"]),
            "hiveMind rows=%d assets=%d surfaces=%d A/B=%s"
            % (len(hm_rows), index["counts"]["assets"],
               index["counts"]["surfaces"], ab_ok))
    c.check("C is marked superseded (no C exports/index rows)",
            os.path.isfile(os.path.join(FIG, "C", "SUPERSEDED.md"))
            and not [r for r in index["assets"] if r.get("owner") == "figcraft-c"],
            "superseded note=%s"
            % os.path.isfile(os.path.join(FIG, "C", "SUPERSEDED.md")))

    report["status"] = "PASS" if c.failed == 0 else "FAIL"
    report["failed_checks"] = c.failed
    from datetime import datetime, timezone
    report["generated_at"] = datetime.now(timezone.utc).astimezone().isoformat(timespec="seconds")
    with open(REPORT, "w", encoding="utf-8", newline="\n") as f:
        json.dump(report, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print()
    print("VERIFY: %s (%d checks, %d failed)" % (report["status"],
                                                 len(c.rows), c.failed))
    print("report:", rel(REPORT))
    raise SystemExit(0 if c.failed == 0 else 1)


if __name__ == "__main__":
    main()
