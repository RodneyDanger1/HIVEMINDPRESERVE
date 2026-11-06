"""Crop regions of an exported 2x PNG for visual QA."""
import sys
from PIL import Image

src = sys.argv[1]
out = sys.argv[2]
# args: x y w h in board units (1440-space); PNG is 2x
x, y, w, h = [int(v) * 2 for v in sys.argv[3:7]]
im = Image.open(src)
im.crop((x, y, x + w, y + h)).save(out)
print(out, im.crop((x, y, x + w, y + h)).size)
