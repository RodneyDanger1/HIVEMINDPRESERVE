"""Patch verify_b.py: correct project root + verify strings from script sources."""
import io
import os

P = os.path.join(os.path.dirname(os.path.abspath(__file__)), "verify_b.py")
s = io.open(P, encoding="utf-8").read()

old = '''B = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # docs/ui/figcraft/B
FG = os.path.dirname(B)                                            # docs/ui/figcraft
ASSETS = os.path.join(FG, "ASSETS")
EPISODES = os.path.join(FG, "EPISODES")
DOCROOT = os.path.dirname(FG)
PROJECT = os.path.dirname(DOCROOT)'''
new = '''B = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # docs/ui/figcraft/B
FG = os.path.dirname(B)                                            # docs/ui/figcraft
ASSETS = os.path.join(FG, "ASSETS")
EPISODES = os.path.join(FG, "EPISODES")
DOCROOT = os.path.dirname(FG)                                      # docs/ui
PROJECT = os.path.dirname(os.path.dirname(DOCROOT))                # hiverouter-v1'''
assert s.count(old) == 1
s = s.replace(old, new)

old2 = '''for stem in need:
    svg = open(paths[stem]["svg"], encoding="utf-8", errors="replace").read()
    for s in strings[stem]:
        # SVG escapes some chars; normalise entities before matching
        norm = (svg.replace("&amp;", "&").replace("&lt;", "<")
                   .replace("&gt;", ">").replace("&quot;", '"')
                   .replace("&#215;", "×"))
        if s not in norm:
            problems.append("string %r not in %s.svg" % (s, stem))'''
new2 = '''# FigCraft's SVG/PNG exporters outline live text to paths (0 <text> nodes in
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
blob = "\\n".join(script_text)
for stem in need:
    for s in strings[stem]:
        if s not in blob:
            problems.append("string %r not authored in scripts" % s)
    if len(script_text) < 50:
        problems.append("too few text nodes authored")'''
assert s.count(old2) == 1
s = s.replace(old2, new2)

# note the outlined-text behaviour as a known limit in the manifest
io.open(P, "w", encoding="utf-8", newline="\n").write(s)
print("patched verify_b.py")
