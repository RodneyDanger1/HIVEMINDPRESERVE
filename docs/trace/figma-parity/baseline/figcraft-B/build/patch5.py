"""Patch 5: HUD spacing (printed separators must not collide)."""
import io
import os

P = os.path.join(os.path.dirname(os.path.abspath(__file__)), "board_default.py")
s = io.open(P, encoding="utf-8").read()

old = '''    b.rect(690, 20, 400, 40, fill=C["paper"], r=999, stroke=C["hairline"],
           sw=1, name="s09/hud", effect="e2")
    b.text(714, 31, HUD[0], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/superagents")
    b.text(802, 31, "·", 11, "Inter", "SemiBold", C["ink500"], width=12,
           align="center", line=15, name="s09/hud/sep1")
    b.text(818, 31, HUD[1], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/workers")
    b.text(894, 31, "·", 11, "Inter", "SemiBold", C["ink500"], width=12,
           align="center", line=15, name="s09/hud/sep2")
    b.dot(918, 34, 7, "circle", C["verify"], name="s09/hud/green-dot")
    b.text(930, 31, HUD[2], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/green")
    b.pill(1022, 26, 52, 28, C["honey200"], HUD[3], C["ink800"], size=10,
           name="s09/hud/delta")
    b.meta(690, 66, "RUN_METRICS · derived aggregate, never hardcoded (R-U13)",
           width=400, align="center", size=8, name="s09/hud/note")'''

new = '''    b.rect(690, 20, 440, 40, fill=C["paper"], r=999, stroke=C["hairline"],
           sw=1, name="s09/hud", effect="e2")
    b.text(714, 31, HUD[0], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/superagents")
    b.text(826, 31, "·", 11, "Inter", "SemiBold", C["ink500"], width=12,
           align="center", line=15, name="s09/hud/sep1")
    b.text(842, 31, HUD[1], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/workers")
    b.text(924, 31, "·", 11, "Inter", "SemiBold", C["ink500"], width=12,
           align="center", line=15, name="s09/hud/sep2")
    b.dot(944, 34, 7, "circle", C["verify"], name="s09/hud/green-dot")
    b.text(956, 31, HUD[2], 11, "Inter", "SemiBold", C["ink900"], line=15,
           name="s09/hud/green")
    b.pill(1058, 26, 52, 28, C["honey200"], HUD[3], C["ink800"], size=10,
           name="s09/hud/delta")
    b.meta(690, 66, "RUN_METRICS · derived aggregate, never hardcoded (R-U13)",
           width=440, align="center", size=8, name="s09/hud/note")'''

assert s.count(old) == 1
s = s.replace(old, new)
io.open(P, "w", encoding="utf-8", newline="\n").write(s)
print("patched board_default.py (patch 5)")
