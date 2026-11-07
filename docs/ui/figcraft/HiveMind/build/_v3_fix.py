import io

path = 'build_hive_style.py'
text = io.open(path, encoding='utf-8').read()

pairs = [
    # right footer: x was the right edge while the box extends right
    ('run detail & files \\u00b7 1440 \\u00d7 900 \\u00b7 existing tokens only",\n'
     '         1368, 860, 8.5, MONO, "Regular", INK500, width=320, align="right")',
     'run detail & files \\u00b7 1440 \\u00d7 900 \\u00b7 existing tokens only",\n'
     '         1048, 856, 8.5, MONO, "Regular", INK500, width=320, align="right")'),
    # READ / dash column must stay inside the tree, not under the preview
    ('320, y, 8, MONO, "Regular",', '300, y, 8, MONO, "Regular",'),
    # RUNNING pill inside the run card
    ('pill(920, 250, "RUNNING"', 'pill(918, 250, "RUNNING"'),
    # compact usage counter rows and lift the note
    ('        y = 396 + row * 38', '        y = 392 + row * 34'),
    ('"values arrive from RUN_METRICS; none are fabricated here.",\n'
     '         1060, 514', '"values arrive from RUN_METRICS; none are fabricated here.",\n'
     '         1060, 500'),
    # DECISION-GATED pill inside the settings card
    ('pill(1268, 720, "DECISION-GATED"', 'pill(1248, 720, "DECISION-GATED"'),
    # footer note clear of the dashed argument margin
    ('"LABELLED", 72, 860', '"LABELLED", 72, 856'),
]
for old, new in pairs:
    if old in text:
        text = text.replace(old, new)
    elif new not in text:
        raise SystemExit('anchor not found: %r' % old[:60])

io.open(path, 'w', encoding='utf-8', newline='\n').write(text)
print('board 12 layout polish applied')
