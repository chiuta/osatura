#!/usr/bin/env python3 -I
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Records tutorial 01 (a tour of the studio) from a running `osatura studio` and writes the caption timing.

    osatura demo-person first-rig.json && osatura studio first-rig.json --port 8765 &
    python3 -I record.py http://127.0.0.1:8765 OUTDIR

Writes OUTDIR/raw.webm (1280x720, no audio, with a drawn mouse pointer) and OUTDIR/cues.json
([{"id": "c01", "start": seconds, "end": seconds}, ...]). build.py turns them into subtitles and .mp4 files.
Needs Python Playwright with Chromium."""
import json
import re
import sys
import time
from pathlib import Path
from playwright.sync_api import sync_playwright

CURSOR_JS = """
(() => {
  const c = document.createElement('div');
  c.style.cssText = 'position:fixed;z-index:2147483647;width:18px;height:18px;border-radius:50%;background:rgba(255,200,0,.85);border:2px solid #fff;pointer-events:none;left:-40px;top:-40px;transform:translate(-50%,-50%);box-shadow:0 0 6px #000';
  document.addEventListener('DOMContentLoaded', () => document.body.appendChild(c));
  addEventListener('mousemove', e => { c.style.left = e.clientX + 'px'; c.style.top = e.clientY + 'px'; }, true);
  addEventListener('mousedown', () => { c.style.background = 'rgba(255,90,60,.95)'; }, true);
  addEventListener('mouseup', () => { c.style.background = 'rgba(255,200,0,.85)'; }, true);
})();
"""


def main(url, out):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    cues = []
    with sync_playwright() as p:
        b = p.chromium.launch()
        ctx = b.new_context(viewport={"width": 1280, "height": 720}, record_video_dir=str(out / "v"), record_video_size={"width": 1280, "height": 720})
        ctx.add_init_script(CURSOR_JS)
        pg = ctx.new_page()
        t0 = time.monotonic()
        pos = [640, 360]

        def now():
            return time.monotonic() - t0

        def move(x, y, steps=28):
            pg.mouse.move(x, y, steps=steps)
            pos[0], pos[1] = x, y

        def center(loc):
            bb = loc.bounding_box()
            return bb["x"] + bb["width"] / 2, bb["y"] + bb["height"] / 2

        def go(loc):
            x, y = center(loc)
            move(x, y)
            pg.wait_for_timeout(250)

        def click(loc):
            go(loc)
            pg.mouse.down()
            pg.wait_for_timeout(90)
            pg.mouse.up()

        def bottom_play():
            # the transport bar's Play button is the lowest visible button called "Play"
            btns = pg.get_by_role("button", name=re.compile(r"^(Play|Pause|Stop)$")).all()
            vis = [(b.bounding_box(), b) for b in btns if b.is_visible()]
            return max(vis, key=lambda t: t[0]["y"])[1]

        def say(cid, secs, action=None):
            start = now()
            if action:
                action()
            rest = secs - (now() - start)
            if rest > 0:
                pg.wait_for_timeout(int(rest * 1000))
            cues.append({"id": cid, "start": round(start + 0.2, 2), "end": round(now() - 0.2, 2)})

        pg.goto(url)
        pg.wait_for_selector("#tl")
        pg.wait_for_timeout(1500)
        move(640, 300, 5)

        say("c01", 5.5)
        say("c02", 7.5, lambda: click(bottom_play()))
        def scrub():
            click(bottom_play())  # pause
            sl = pg.locator("#tl")
            bb = sl.bounding_box()
            y = bb["y"] + bb["height"] / 2
            move(bb["x"] + 8, y)
            pg.mouse.down()
            move(bb["x"] + bb["width"] * 0.55, y, 40)
            pg.mouse.up()
        say("c03", 7.0, scrub)
        def layers():
            click(pg.get_by_text("Layers", exact=True).locator("visible=true").first)
            click(pg.get_by_text("arm_far").locator("visible=true").first)
        say("c04", 5.5, layers)
        def bones_on():
            click(pg.get_by_text("bones", exact=True).locator("visible=true").first)
        say("c05", 6.0, bones_on)
        say("c06", 5.5, lambda: click(pg.get_by_text("Bones", exact=True).locator("visible=true").first))
        def key_click():
            # the timeline is drawn on a canvas, so the key (bone_angle row, frame 6) is addressed by position
            move(434, 649)
            pg.mouse.down(); pg.wait_for_timeout(90); pg.mouse.up()
        say("c07", 6.5, key_click)
        def autokey():
            click(pg.get_by_text("auto-key", exact=True).locator("visible=true").first)
        say("c08", 6.0, autokey)
        def palette():
            click(pg.locator("#palbtn"))
        say("c09", 6.5, palette)
        def exports():
            pg.keyboard.press("Escape")
            pg.wait_for_timeout(300)
            for sel in ("GIF", "HTML player", "Lottie"):
                go(pg.get_by_text(sel, exact=True).locator("visible=true").first)
                pg.wait_for_timeout(700)
        say("c10", 6.5, exports)
        def demo():
            click(pg.locator("#demobtn"))
            pg.wait_for_timeout(600)
            click(pg.locator("#dm-start"))
        say("c11", 7.0, demo)
        def restore():
            for _ in range(2):
                click(pg.locator("#dm-next"))
                pg.wait_for_timeout(900)
            click(pg.locator("#dm-restore"))
        say("c12", 7.0, restore)
        say("c13", 5.5, lambda: move(640, 360, 20))
        vid = pg.video
        ctx.close()
        b.close()
        Path(vid.path()).replace(out / "raw.webm")
    (out / "cues.json").write_text(json.dumps(cues, indent=1))
    print(f"{out/'raw.webm'}: {cues[-1]['end']:.1f} s, {len(cues)} captions")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
