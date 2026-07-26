#!/usr/bin/env python3
"""Render the macOS app-icon masters: chartr's art on Apple's icon grid.

macOS does NOT mask app icons — whatever shape the .icns carries is the shape the
Dock, Finder and the ⌘-Tab switcher draw. So the shape has to be baked in, and it
has to be Apple's shape, or the tile reads as the wrong size and the wrong
silhouette beside every neighbour.

The grid constants below are not taste. They were measured off six shipping system
icons (Calculator, Mail, Music, Notes, Preview, Reminders) by fitting this very
construction to their alpha channels; the fit lands on Apple's published template
values to within 0.19px RMS at 256px, and the corner smoothing minimises there
unanimously across all six. The same 824/1024 art box holds at every entry size —
Apple's own 16px entry measures 12.25/16 = 0.766 against the grid's 0.805, a
sub-pixel optical tightening not worth chasing.

One PNG per size band, because the artwork is redrawn per band rather than scaled:
at 16 the cursor is bigger and bleeds off the plate, at 32 it is contained, and 48
is the full-detail drawing that every larger entry comes from. Downscaling one
drawing to 16px instead is what made the old tile mush at list-view sizes.

Run from the repo root; needs rsvg-convert (brew install librsvg) and Pillow. The
output is committed, so `make bundle` needs neither — it only downscales.
"""
import math
import subprocess
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

# Apple's macOS app-icon grid. Every length here is in 1024-canvas units and is
# scaled to whatever canvas is being rendered.
GRID = 1024
ART = 824              # the rounded square the artwork fills, centred
RADIUS_RATIO = 0.225   # 185.4 / 824
SMOOTHING = 0.7        # continuous-corner term; 0 would be a plain circular arc

# The template's baked shadow, fitted to the same six icons. macOS composites the
# icon flat, so a tile with no shadow sits visibly flatter than its neighbours.
# This fit is shallow and degenerate — opacity and blur trade off against each
# other, and anything in this neighbourhood lands within 0.02 alpha of Apple's
# ramp, which is well under what an eye resolves. These are a near-minimum, not a
# unique answer; don't chase the last digit. At 16px the blur works out under a
# quarter-pixel and the shadow all but vanishes, which is what Apple's does too.
SHADOW_ALPHA = 0.26
SHADOW_BLUR = 16.0
SHADOW_DY = 6.0

MASK_SS = 4  # extra supersampling for the mask alone, for a clean corner

# master -> (output name, canvas). The canvas is the largest .icns entry the band
# feeds, so nothing downstream is ever upscaled.
BANDS = [
    ("logo-app-16.svg", "icon-mac-16.png", 16),
    ("logo-app-32.svg", "icon-mac-32.png", 32),
    ("logo-app.svg", "icon-mac-1024.png", 1024),
]


def squircle_path(side, radius, smoothing, x=0.0, y=0.0):
    """SVG path for a rounded square with Apple-style continuous corners.

    Each corner is a circular arc of measure 90*(1-s) flanked by two cubic
    Beziers that carry curvature continuously out to the straight edge, per
    Figma's corner-smoothing construction.
    """
    r, s = radius, smoothing
    p = (1 + s) * r
    arc_measure = 90 * (1 - s)
    arc = math.sin(math.radians(arc_measure / 2)) * r * math.sqrt(2)
    angle_alpha = (90 - arc_measure) / 2
    angle_beta = 45 * s
    c = r * math.tan(math.radians(angle_alpha / 2)) * math.cos(math.radians(angle_beta))
    d = c * math.tan(math.radians(angle_beta))
    b = (p - arc - c - d) / 3
    a = 2 * b

    def n(v):
        return f"{v:.4f}"

    return " ".join([
        f"M {n(x + side - p)} {n(y)}",
        f"c {n(a)} 0 {n(a + b)} 0 {n(a + b + c)} {n(d)}",
        f"a {n(r)} {n(r)} 0 0 1 {n(arc)} {n(arc)}",
        f"c {n(d)} {n(c)} {n(d)} {n(b + c)} {n(d)} {n(a + b + c)}",
        f"L {n(x + side)} {n(y + side - p)}",
        f"c 0 {n(a)} 0 {n(a + b)} {n(-d)} {n(a + b + c)}",
        f"a {n(r)} {n(r)} 0 0 1 {n(-arc)} {n(arc)}",
        f"c {n(-c)} {n(d)} {n(-(b + c))} {n(d)} {n(-(a + b + c))} {n(d)}",
        f"L {n(x + p)} {n(y + side)}",
        f"c {n(-a)} 0 {n(-(a + b))} 0 {n(-(a + b + c))} {n(-d)}",
        f"a {n(r)} {n(r)} 0 0 1 {n(-arc)} {n(-arc)}",
        f"c {n(-d)} {n(-c)} {n(-d)} {n(-(b + c))} {n(-d)} {n(-(a + b + c))}",
        f"L {n(x)} {n(y + p)}",
        f"c 0 {n(-a)} 0 {n(-(a + b))} {n(d)} {n(-(a + b + c))}",
        f"a {n(r)} {n(r)} 0 0 1 {n(arc)} {n(-arc)}",
        f"c {n(c)} {n(-d)} {n(b + c)} {n(-d)} {n(a + b + c)} {n(-d)}",
        "Z",
    ])


def working_scale(canvas):
    """Smallest integer supersample making both the art box and the inset whole.

    The art box is a non-integer fraction of small canvases (824/1024 of 16px is
    12.875), and rasterising art at a fractional size or pasting it at a
    fractional offset is how a 16px tile turns to mush. Composite on a canvas
    where both land on integers, then box down by a whole factor.
    """
    ss = 1
    while True:
        work = canvas * ss
        art = ART * work / GRID
        if art.is_integer() and ((work - art) / 2).is_integer() and work >= 256:
            return ss, work, int(art), int((work - art) / 2)
        ss *= 2


def rasterize(svg: str, size: int, tmp: Path) -> Image.Image:
    tmp.write_text(svg)
    out = tmp.with_suffix(".png")
    subprocess.run(
        ["rsvg-convert", "-w", str(size), "-h", str(size), str(tmp), "-o", str(out)],
        check=True,
    )
    img = Image.open(out).convert("RGBA")
    img.load()
    return img


def build(master: Path, dest: Path, canvas: int) -> None:
    ss, work, art_px, inset = working_scale(canvas)
    k = work / GRID  # 1024-units -> working-canvas units
    tmp = dest.parent / ".mac-app-icon.tmp.svg"

    # The mask: Apple's rounded square, supersampled again then boxed down so the
    # corner carries more gradations than rsvg gives at 1x.
    path = squircle_path(art_px * MASK_SS, art_px * MASK_SS * RADIUS_RATIO, SMOOTHING,
                         x=inset * MASK_SS, y=inset * MASK_SS)
    side = work * MASK_SS
    mask = rasterize(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{side}" height="{side}" '
        f'viewBox="0 0 {side} {side}"><path d="{path}" fill="#fff"/></svg>',
        side, tmp).getchannel("A").resize((work, work), Image.LANCZOS)

    # The artwork, rendered natively at the size it will occupy — never upscaled.
    art = rasterize(master.read_text(), art_px, tmp)

    plate = Image.new("RGBA", (work, work), (0, 0, 0, 0))
    plate.paste(art, (inset, inset))
    plate.putalpha(Image.fromarray(
        (_arr(plate.getchannel("A")) * _arr(mask) / 255).astype("uint8")))

    # The shadow is the mask, blurred and pushed down, under the plate.
    blurred = mask.filter(ImageFilter.GaussianBlur(SHADOW_BLUR * k))
    shifted = Image.new("L", (work, work), 0)
    shifted.paste(Image.fromarray((_arr(blurred) * SHADOW_ALPHA).astype("uint8")),
                  (0, round(SHADOW_DY * k)))
    shadow = Image.new("RGBA", (work, work), (0, 0, 0, 0))
    shadow.putalpha(shifted)

    out = Image.alpha_composite(shadow, plate)
    if ss > 1:
        out = out.resize((canvas, canvas), Image.LANCZOS)
    # optimize=True because this file is go:embed-ed into the shipped binary.
    out.save(dest, optimize=True)
    tmp.unlink(missing_ok=True)
    tmp.with_suffix(".png").unlink(missing_ok=True)
    print(f"  {dest.name:20} {canvas:>4}px canvas, {ART * canvas // GRID}px art "
          f"(composited at {work}px, {master.name})")


def _arr(img):
    return np.asarray(img).astype("float64")


if __name__ == "__main__":
    root = Path(__file__).resolve().parent.parent
    src = root / "docs/assets/v4"
    dests = [root / "web/public", root / "docs/assets/v4"]
    if len(sys.argv) > 1:  # explicit override: <master> <dest> <canvas>
        build(Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3]))
        sys.exit()
    for master, name, canvas in BANDS:
        print(f"{master}:")
        build(src / master, dests[0] / name, canvas)
        for extra in dests[1:]:
            (extra / name).write_bytes((dests[0] / name).read_bytes())
