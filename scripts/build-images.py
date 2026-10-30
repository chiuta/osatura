#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""WebP copies of docs/img/*.png for the site gallery: site/img/<name>-{320,800,1400}.webp (needs Pillow)."""
import json, pathlib
from PIL import Image
ROOT = pathlib.Path(__file__).resolve().parent.parent
out = ROOT / "site" / "img"; out.mkdir(parents=True, exist_ok=True)
sizes = {}
for p in sorted((ROOT / "docs" / "img").glob("*.png")):
    im = Image.open(p).convert("RGB"); w, h = im.size; sizes[p.name] = [w, h]
    for tw in (320, 800, 1400):
        if tw > w and tw != 320: continue
        r = im if tw >= w else im.resize((tw, round(h * tw / w)), Image.LANCZOS)
        r.save(out / f"{p.stem}-{tw}.webp", "WEBP", quality=82 if tw > 320 else 75, method=6)
(out / "sizes.json").write_text(json.dumps(sizes, separators=(",", ":")) + "\n")
print(f"{len(sizes)} images -> {out}")
