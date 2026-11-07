import io

path = 'build_hive_style.py'
text = io.open(path, encoding='utf-8').read()

old_sub = '''                 "The same spatial shell and cards, with the proposed motion layer: "
                 "ambient honeycomb travel, the 5-lobe hive sheen, bee flow, running "
                 "pulse, spinner, rise/hover and the ink-700 focus ring. Durations "
                 "and easing are PROPOSED, not measured; no new tokens or behaviour.")'''
new_sub = '''                 "Same shell and cards + the proposed motion layer "
                 "(durations/easing PROPOSED).")'''
if old_sub in text:
    text = text.replace(old_sub, new_sub)
elif new_sub not in text:
    raise SystemExit('motion sub not found')

# focus callout is written with \u00b7 escapes in the builder source
old_focus = 'FOCUS RING \\u00b7 ink-700 \\u00b7 2px \\u00b7 160ms PROPOSED'
new_focus = 'FOCUS RING \\u00b7 ink-700 2px'
if old_focus in text:
    text = text.replace(old_focus, new_focus)
elif new_focus not in text:
    raise SystemExit('focus label not found')

io.open(path, 'w', encoding='utf-8', newline='\n').write(text)
print('motion header sub shortened; focus callout shortened')
