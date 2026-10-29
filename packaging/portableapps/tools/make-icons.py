#!/usr/bin/env python3 -I
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Draws the package icon (an original design: a three-joint bone chain) into App/AppInfo.
Usage: python3 -I make-icons.py <App/AppInfo folder>. Needs Pillow."""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

def draw(size: int) -> Image.Image:
    s = size * 4
    im = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle((0, 0, s - 1, s - 1), radius=s // 5, fill=(20, 54, 66, 255))
    pts = [(0.24, 0.72), (0.46, 0.36), (0.76, 0.28)]
    px = [(x * s, y * s) for x, y in pts]
    w = max(2, int(s * 0.07))
    d.line(px, fill=(236, 240, 241, 255), width=w, joint="curve")
    for i, (x, y) in enumerate(px):
        r = s * (0.085 if i else 0.10)
        d.ellipse((x - r, y - r, x + r, y + r), fill=(242, 169, 0, 255) if i == 0 else (236, 240, 241, 255), outline=(20, 54, 66, 255), width=max(1, s // 64))
    return im.resize((size, size), Image.LANCZOS)

out = Path(sys.argv[1])
for n in (16, 32, 75, 128):
    draw(n).save(out / f"appicon_{n}.png")
draw(256).save(out / "appicon.ico", sizes=[(16, 16), (32, 32), (48, 48), (256, 256)])
