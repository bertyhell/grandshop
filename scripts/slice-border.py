"""Cuts a picture of a whole frame into the eight pieces of a border style in assets/borders.

The corners are boxes of `corner_width` × `corner_height` (fractions of the frame) at the four
corners, and each edge is the whole side between two corners. The result is scaled so the
corners are `--corner-size` pixels wide.

Usage: python3 -I scripts/slice-border.py frame.png assets/borders/<name> 0.14 0.17 [--corner-size 300]
    [--backing white --backing-gap 40]

Line art (outlines on transparency) lets the photo show through every shape. `--backing` fills
the drawing with a colour, closing gaps of up to `--backing-gap` pixels between strokes.

Render an SVG first with transparency kept, for example:
    magick -background none -density 300 frame.svg frame.png
"""
import argparse
from pathlib import Path

from PIL import Image, ImageFilter

parser = argparse.ArgumentParser()
parser.add_argument("frame", type=Path)
parser.add_argument("out_dir", type=Path)
parser.add_argument("corner_width", type=float, help="fraction of the frame width")
parser.add_argument("corner_height", type=float, help="fraction of the frame height")
parser.add_argument("--corner-size", type=int, default=300, help="width of the corners in pixels")
parser.add_argument("--backing", help="colour to fill line art with, e.g. white")
parser.add_argument("--backing-gap", type=int, default=40, help="widest gap between strokes to fill")
args = parser.parse_args()

frame = Image.open(args.frame).convert("RGBA")
frame = frame.crop(frame.getbbox())
scale = args.corner_size / (frame.width * args.corner_width)
frame = frame.resize((round(frame.width * scale), round(frame.height * scale)), Image.LANCZOS)
width, height = frame.size

if args.backing:
    # Closing (grow, then shrink) the shape of the drawing fills the space between its strokes
    size = args.backing_gap | 1
    shape = frame.getchannel("A").point(lambda a: 255 if a > 20 else 0)
    shape = shape.filter(ImageFilter.MaxFilter(size)).filter(ImageFilter.MinFilter(size))
    backed = Image.new("RGBA", frame.size, (0, 0, 0, 0))
    backed.paste(Image.new("RGBA", frame.size, args.backing), mask=shape)
    backed.alpha_composite(frame)
    frame = backed

cw, ch = round(width * args.corner_width), round(height * args.corner_height)

pieces = {
    "corner-top-left": (0, 0, cw, ch),
    "corner-top-right": (width - cw, 0, width, ch),
    "corner-bottom-left": (0, height - ch, cw, height),
    "corner-bottom-right": (width - cw, height - ch, width, height),
    "edge-top": (cw, 0, width - cw, ch),
    "edge-bottom": (cw, height - ch, width - cw, height),
    "edge-left": (0, ch, cw, height - ch),
    "edge-right": (width - cw, ch, width, height - ch),
}
args.out_dir.mkdir(parents=True, exist_ok=True)
for name, box in pieces.items():
    frame.crop(box).save(args.out_dir / f"{name}.png", optimize=True)
print(f"{args.out_dir}: frame {width}×{height}, corners {cw}×{ch}")
