#!/usr/bin/env python3 -I
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Records and builds the tutorials.

    python3 -I make.py OUTDIR [ID ...]        (no ID: all of them; an ID is a prefix such as 03 or 03-keys)

Each tNN_*.py module defines ID, KIND ("studio" or "terminal") and run(). A studio tutorial gets a freshly started
`osatura studio`; a terminal tutorial gets an empty working folder. Output: OUTDIR/<ID>/tutorial-<ID>*.{mp4,srt,vtt}.
Needs target/release/osatura (or OSATURA_BIN), Python Playwright with Chromium, and ffmpeg with libass."""
import importlib
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import build  # noqa: E402
import common  # noqa: E402


def modules():
    here = Path(__file__).resolve().parent
    return [importlib.import_module(p.stem) for p in sorted(here.glob("t[0-9][0-9]_*.py"))]


def make(mod, out):
    out = Path(out) / mod.ID
    with tempfile.TemporaryDirectory(prefix="osatura-tut-") as work:
        if mod.KIND == "studio":
            size = getattr(mod, "SIZE", (1280, 720))
            with common.studio(work, make=getattr(mod, "MAKE", ("demo-person", "first-rig.json"))) as (url, proj):
                with common.recording(out / "rec", size) as r:
                    mod.run(r, url, proj)
        else:
            with common.recording(out / "rec") as r:
                mod.run(r, Path(work))
    build.main(mod.ID, out / "rec", out)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    want = sys.argv[2:]
    for m in modules():
        if not want or any(m.ID.startswith(w) for w in want):
            make(m, sys.argv[1])
