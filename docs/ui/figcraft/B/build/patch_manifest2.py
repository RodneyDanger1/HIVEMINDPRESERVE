"""Patch make_manifest: project-relative design path + outlined-text note."""
import io
import os

P = os.path.join(os.path.dirname(os.path.abspath(__file__)), "make_manifest.py")
s = io.open(P, encoding="utf-8").read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep('''        "design_file": {
            "path": state.get("file"),''',
    '''        "design_file": {
            "path": rel(os.path.join(os.path.dirname(B_ROOT),
                                     os.path.basename(state.get("file") or ""))),''')

rep('''            "S25 DIFF commands are NOT FROZEN (UI-COMMANDS-v1 §2): board shows the modal labelled DECISION-GATED, not implemented.",
        ],''',
    '''            "S25 DIFF commands are NOT FROZEN (UI-COMMANDS-v1 §2): board shows the modal labelled DECISION-GATED, not implemented.",
            "SVG/PNG exports outline live text to paths (FigCraft exporter behaviour, 0 <text> nodes); the live text remains editable in B-canvas.figcraft and the exact strings are the `text` params in B/scripts/*.json.",
            "Bee direction is baked into path geometry (rotated in Python), so the bee always points source->target; SVG has no arrow marker to inspect separately.",
        ],''')

io.open(P, "w", encoding="utf-8", newline="\n").write(s)
print("patched make_manifest.py")
