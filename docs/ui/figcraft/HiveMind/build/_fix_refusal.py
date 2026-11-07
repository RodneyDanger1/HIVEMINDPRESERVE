import io

path = 'build_hive_style.py'
text = io.open(path, encoding='utf-8').read()
lines = text.splitlines()
start = next(i for i, line in enumerate(lines) if '# 01 named refusal' in line)
end = None
for i in range(start, len(lines)):
    if 'ERROR frame' in lines[i]:
        end = i
        break
assert end is not None, 'ERROR frame line not found'
# include the closing continuation line (contains ls="4%")
while end + 1 < len(lines) and 'ls="4%"' not in lines[end]:
    end += 1
assert 'ls="4%"' in lines[end], 'end anchor not found'

new_block = '''    # 01 named refusal (crimson refusal card, S18 error-card anatomy)
    x, y = 72, 220
    cy = state_card(x, y, "S14 / NAMED REFUSAL", "E_APPROVAL_TIMEOUT")
    rect(f, x + 20, cy, 380, 138, fill=PANEL, radius=14, stroke=ERR, sw=1.5,
         shadow=SH_E1)
    text(f, "E_APPROVAL_TIMEOUT", x + 36, cy + 14, 11.5, MONO, "SemiBold", ERR)
    text(f, "The approval request timed out before a decision arrived.",
         x + 36, cy + 38, 9.5, UI, "Regular", INK700, width=344, lh="145%")
    text(f, "how_to_fix: Resolve the request when prompted; the tool call is "
            "refused, never faked.", x + 36, cy + 82, 8.5, MONO, "Regular",
         INK500, width=344, lh="145%")
    text(f, "S18 crimson error anatomy · ERROR frame → toast + log + ledger",
         x + 20, cy + 160, 9, MONO, "Regular", INK500, ls="3%")'''.splitlines()

out = lines[:start] + new_block + lines[end + 1:]
io.open(path, 'w', encoding='utf-8', newline='\n').write('\n'.join(out) + '\n')
print('states refusal card fixed: replaced lines', start + 1, '-', end + 1)
