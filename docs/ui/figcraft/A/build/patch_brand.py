import io
p = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\pages_brand.py"
s = io.open(p, encoding="utf-8").read()

old = '''    b.section(48, 736, 1344, "minimum sizes · bee 56 / 32 / 16  ·  wordmark min 20",
              note="PDF p5:160-169")
    b.rect(48, 760, 1344, 96, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group="01-minsizes", name="01/minsizes/panel")
    b.hex_bee(76, 782, 56, C["ink900"], C["honey300"], C["honey500"], C["paper"],
              stripe=C["ink900"], group="01-minsizes", name="01/minsizes/56")
    b.text(76, 770 - 12 + 12, "", 8, "Consolas", "Regular", C["ink500"], width=1,
           line=1, group="01-minsizes")
    b.text(72, 842, "56 · app icon / hero", 8.5, "Consolas", "Regular", C["ink700"],
           width=140, line=12, group="01-minsizes")
    b.hex_bee(236, 794, 32, C["ink900"], C["honey300"], C["honey500"], C["paper"],
              stripe=C["ink900"], group="01-minsizes", name="01/minsizes/32")
    b.text(216, 842, "32 · node chip", 8.5, "Consolas", "Regular", C["ink700"],
           width=120, line=12, group="01-minsizes")
    b.hex_bee(352, 802, 16, C["ink900"], None, None, None, bee=False,
              group="01-minsizes", name="01/minsizes/16")
    b.text(316, 842, "16 · favicon — inner bee dropped below 24px", 8.5, "Consolas",
           "Regular", C["ink700"], width=200, line=12, group="01-minsizes")
    b.text(548, 786, "HiveRouter", 20, "Playfair Display", "SemiBold", C["ink900"],
           width=200, line=24, tracking=-0.5, group="01-minsizes",
           name="01/minsizes/wordmark")
    b.text(548, 812, "wordmark min 20px", 8.5, "Consolas", "Regular", C["ink500"],
           width=160, line=12, group="01-minsizes")
    b.text(320, 786, "Never below 16px. Below 24px drop the inner bee and keep the "
                     "hexagon silhouette only.", 8.5, "Consolas", "Regular",
           C["ink500"], width=210, line=12, group="01-minsizes")
    b.text(760, 782, "MONO 1-COLOUR LOCKUP IS THE FALLBACK", 8.5, "Consolas",
           "Regular", C["ink700"], width=280, line=12, tracking=8,
           group="01-minsizes")
    b.text(760, 798, "for single-ink print and embroidery. The mark may rotate with "
                     "the edge angle (±1°); never mirror the bee.",
           9, "Inter", "Regular", C["ink700"], width=380, line=14,
           group="01-minsizes")
    b.text(760, 834, "24–55px: inner bee may drop. ≥56px: full construction. "
                     "Clear space scales with h.", 8.5, "Consolas", "Regular",
           C["ink500"], width=420, line=12, group="01-minsizes")
'''
new = '''    b.section(48, 736, 1344, "minimum sizes · bee 56 / 32 / 16  ·  wordmark min 20",
              note="PDF p5:160-169")
    b.rect(48, 760, 1344, 100, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group="01-minsizes", name="01/minsizes/panel")
    b.hex_bee(72, 772, 56, C["ink900"], C["honey300"], C["honey500"], C["paper"],
              stripe=C["ink900"], group="01-minsizes", name="01/minsizes/56")
    b.text(66, 836, "56 · app icon / hero", 8.5, "Consolas", "Regular", C["ink700"],
           width=150, line=12, group="01-minsizes")
    b.hex_bee(232, 784, 32, C["ink900"], C["honey300"], C["honey500"], C["paper"],
              stripe=C["ink900"], group="01-minsizes", name="01/minsizes/32")
    b.text(212, 836, "32 · node chip", 8.5, "Consolas", "Regular", C["ink700"],
           width=120, line=12, group="01-minsizes")
    b.hex_bee(344, 792, 16, C["ink900"], None, None, None, bee=False,
              group="01-minsizes", name="01/minsizes/16")
    b.text(316, 836, "16 · favicon — inner bee dropped below 24px", 8.5, "Consolas",
           "Regular", C["ink700"], width=220, line=12, group="01-minsizes")
    b.text(556, 772, "Never below 16px.", 9, "Consolas", "Regular", C["ink700"],
           width=180, line=13, group="01-minsizes")
    b.text(556, 788, "Below 24px drop the inner bee and keep the hexagon silhouette only.",
           8.5, "Inter", "Regular", C["ink700"], width=180, line=12,
           group="01-minsizes")
    b.text(556, 828, "24–55px: inner bee may drop.", 8.5, "Consolas", "Regular",
           C["ink500"], width=180, line=12, group="01-minsizes")
    b.text(760, 776, "HiveRouter", 20, "Playfair Display", "SemiBold", C["ink900"],
           width=200, line=24, tracking=-0.5, group="01-minsizes",
           name="01/minsizes/wordmark")
    b.text(760, 804, "wordmark min 20px", 8.5, "Consolas", "Regular", C["ink500"],
           width=160, line=12, group="01-minsizes")
    b.text(760, 824, "≥56px: full construction.", 8.5, "Consolas", "Regular",
           C["ink500"], width=160, line=12, group="01-minsizes")
    b.text(932, 772, "MONO 1-COLOUR LOCKUP IS THE FALLBACK", 8.5, "Consolas",
           "Regular", C["ink700"], width=380, line=12, tracking=8,
           group="01-minsizes")
    b.text(932, 788, "for single-ink print and embroidery. The mark may rotate with the "
                     "edge angle (±1°); never mirror the bee.", 9, "Inter", "Regular",
           C["ink700"], width=440, line=14, group="01-minsizes")
    b.text(932, 828, "Clear space scales with h — 0.25 × h on every side.", 8.5,
           "Consolas", "Regular", C["ink500"], width=440, line=12,
           group="01-minsizes")
'''
assert old in s
s = s.replace(old, new)
io.open(p, "w", encoding="utf-8", newline="\n").write(s)
print("patched brand")
