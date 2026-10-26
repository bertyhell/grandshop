"""Extract the FontAwesome solid glyphs the UI uses into standalone SVG files.

Usage: python3 -I scripts/extract-icons.py <fa-solid-900.svg> <output-dir>
"""
import re
import sys
from pathlib import Path

ICONS = [
    "cog",
    "search-plus",
    "search-minus",
    "compress",
    "undo",
    "redo",
    "arrows-alt-h",
    "arrows-alt-v",
]

font = Path(sys.argv[1]).read_text()
out_dir = Path(sys.argv[2])
out_dir.mkdir(parents=True, exist_ok=True)

ascent = int(re.search(r'ascent="(-?\d+)"', font).group(1))
units = int(re.search(r'units-per-em="(\d+)"', font).group(1))
default_adv = int(re.search(r'<font[^>]*horiz-adv-x="(\d+)"', font).group(1))

for name in ICONS:
    glyph = re.search(r'<glyph glyph-name="%s"(.*?)/>' % re.escape(name), font, re.S)
    if not glyph:
        sys.exit("glyph not found: " + name)
    attrs = glyph.group(1)
    adv = re.search(r'horiz-adv-x="(\d+)"', attrs)
    width = int(adv.group(1)) if adv else default_adv
    path = " ".join(re.search(r'd="([^"]+)"', attrs, re.S).group(1).split())
    # SVG fonts use a y-up coordinate system with the baseline at 0
    (out_dir / (name + ".svg")).write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 %d %d">'
        '<path fill="#000" transform="translate(0 %d) scale(1 -1)" d="%s"/></svg>\n'
        % (width, units, ascent, path)
    )
