# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Shared parts of the tutorial recorders: a studio started on a fresh copy of a project, a screen recorder with a drawn
mouse pointer and timed captions, and a simulated terminal that runs real commands. Needs Python Playwright (Chromium)."""
import base64
import contextlib
import json
import os
import re
import shlex
import socket
import subprocess
import tempfile
import time
import urllib.request
from pathlib import Path

from playwright.sync_api import sync_playwright

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent

CURSOR_JS = """
(() => {
  const c = document.createElement('div');
  c.style.cssText = 'position:fixed;z-index:2147483647;width:18px;height:18px;border-radius:50%;background:rgba(255,200,0,.85);border:2px solid #fff;pointer-events:none;left:-40px;top:-40px;transform:translate(-50%,-50%);box-shadow:0 0 6px #000';
  const add = () => document.body && document.body.appendChild(c);
  if (document.body) add(); else document.addEventListener('DOMContentLoaded', add);
  addEventListener('mousemove', e => { c.style.left = e.clientX + 'px'; c.style.top = e.clientY + 'px'; }, true);
  addEventListener('mousedown', () => { c.style.background = 'rgba(255,90,60,.95)'; }, true);
  addEventListener('mouseup', () => { c.style.background = 'rgba(255,200,0,.85)'; }, true);
})();
"""


def binary():
    p = os.environ.get("OSATURA_BIN") or str(ROOT / "target" / "release" / "osatura")
    if not Path(p).exists():
        raise SystemExit(f"{p} not found: build it (cargo build --release -p osatura-cli) or set OSATURA_BIN")
    return p


def free_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


@contextlib.contextmanager
def studio(work, make=("demo-person", "first-rig.json"), project=None, extra=()):
    """Starts `osatura studio` on a fresh project and a fresh UI config (the studio remembers its layout, so a
    reused config would make recordings differ). `make` = (command, file) to create the project in `work`."""
    work = Path(work)
    work.mkdir(parents=True, exist_ok=True)
    exe = binary()
    proj = work / (project or make[1])
    if project is None:
        subprocess.run([exe, make[0], str(proj)], check=True, stdout=subprocess.DEVNULL)
    port = free_port()
    log = open(work / "studio.log", "w")
    ui = work / "ui.json"
    if ui.exists():
        ui.unlink()
    proc = subprocess.Popen([exe, "studio", str(proj), "--port", str(port), "--ui-config", str(ui), *extra], cwd=work, stdout=log, stderr=log)
    url = f"http://127.0.0.1:{port}"
    try:
        for _ in range(100):
            try:
                urllib.request.urlopen(url, timeout=1).read(1)
                break
            except Exception:
                time.sleep(0.1)
        else:
            raise SystemExit("the studio did not start; see " + str(work / "studio.log"))
        yield url, proj
    finally:
        proc.terminate()
        try:
            proc.wait(5)
        except subprocess.TimeoutExpired:
            proc.kill()
        log.close()


class Recorder:
    """Drives one page and records the caption timing in `cues` (seconds from the start of the video)."""

    def __init__(self, page, t0):
        self.pg, self.t0, self.cues, self.pos = page, t0, [], [640, 360]

    def now(self):
        return time.monotonic() - self.t0

    def wait(self, secs):
        self.pg.wait_for_timeout(int(secs * 1000))

    def move(self, x, y, steps=28):
        self.pg.mouse.move(x, y, steps=steps)
        self.pos[:] = [x, y]

    def vis(self, text, exact=True):
        return self.pg.get_by_text(text, exact=exact).locator("visible=true").first

    def center(self, loc):
        bb = loc.bounding_box()
        if bb is None:
            raise RuntimeError(f"not visible: {loc}")
        return bb["x"] + bb["width"] / 2, bb["y"] + bb["height"] / 2

    def go(self, loc, steps=28):
        x, y = self.center(loc) if not isinstance(loc, tuple) else loc
        self.move(x, y, steps)
        self.wait(0.25)

    def click(self, loc, steps=28):
        self.go(loc, steps)
        self.pg.mouse.down()
        self.wait(0.09)
        self.pg.mouse.up()

    def drag(self, a, b, steps=30):
        self.go(a)
        self.pg.mouse.down()
        self.move(b[0], b[1], steps)
        self.pg.mouse.up()

    def key(self, k):
        self.pg.keyboard.press(k)

    def transport_play(self):
        """The Play/Pause button of the transport bar: the lowest visible one."""
        btns = self.pg.get_by_role("button", name=re.compile(r"^(Play|Pause|Stop)$")).all()
        vis = [(b.bounding_box(), b) for b in btns if b.is_visible()]
        return max(vis, key=lambda t: t[0]["y"])[1]

    def open_studio(self, url):
        self.pg.goto(url)
        self.pg.wait_for_selector("#tl")
        self.wait(1.5)
        self.move(640, 300, 5)

    def say(self, cid, secs, action=None):
        start = self.now()
        if action:
            action()
        rest = secs - (self.now() - start)
        if rest > 0:
            self.wait(rest)
        self.cues.append({"id": cid, "start": round(start + 0.2, 2), "end": round(self.now() - 0.2, 2)})


@contextlib.contextmanager
def recording(out, size=(1280, 720)):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as p:
        b = p.chromium.launch()
        ctx = b.new_context(viewport={"width": size[0], "height": size[1]}, record_video_dir=str(out / "v"), record_video_size={"width": size[0], "height": size[1]})
        ctx.add_init_script(CURSOR_JS)
        pg = ctx.new_page()
        rec = Recorder(pg, time.monotonic())
        try:
            yield rec
        finally:
            vid = pg.video
            ctx.close()
            b.close()
        Path(vid.path()).replace(out / "raw.webm")
    (out / "cues.json").write_text(json.dumps(rec.cues, indent=1))
    print(f"{out/'raw.webm'}: {rec.cues[-1]['end']:.1f} s, {len(rec.cues)} captions")


class Term(Recorder):
    """A simulated terminal page. Commands are really run (in `cwd`, with `osatura` on the PATH); their output is typed
    out line by line, so what the viewer reads is what the program printed."""

    def __init__(self, rec, cwd):
        super().__init__(rec.pg, rec.t0)
        self.cues = rec.cues
        self.cwd = Path(cwd)
        self.cwd.mkdir(parents=True, exist_ok=True)
        self.env = dict(os.environ, PATH=str(Path(binary()).parent) + os.pathsep + os.environ["PATH"], NO_COLOR="1")
        rec.pg.goto((HERE / "terminal.html").as_uri())

    def comment(self, text):
        self.pg.evaluate("t => out(t, 'd')", "# " + text)
        self.wait(0.3)

    def type(self, cmd, delay=0.045):
        self.pg.evaluate("prompt()")
        for ch in cmd:
            self.pg.evaluate("c => typeChar(c)", ch)
            self.wait(delay)
        self.wait(0.35)
        self.pg.evaluate("endCmd()")

    def run(self, cmd, hold=1.5, lines=12, width=96, shown=None, check=True):
        """Types `cmd`, runs it for real and prints the start of its output (`lines`, each cut at `width`)."""
        self.type(shown or cmd)
        r = subprocess.run(shlex.split(cmd), cwd=self.cwd, env=self.env, capture_output=True, text=True, timeout=300)
        if check and r.returncode != 0:
            raise RuntimeError(f"{cmd} failed ({r.returncode}): {r.stderr[:300]}")
        text = (r.stdout + r.stderr).rstrip("\n")
        ls = text.split("\n") if text else []
        more = len(ls) - lines
        for l in ls[:lines]:
            self.pg.evaluate("t => out(t)", l if len(l) <= width else l[: width - 1] + "…")
            self.wait(0.07)
        if more > 0:
            self.pg.evaluate("t => out(t, 'd')", f"… ({more} more lines)")
        self.wait(hold)
        return r

    def image(self, path, cap=""):
        data = "data:image/png;base64," + base64.b64encode(Path(path).read_bytes()).decode()
        self.pg.evaluate("a => showImg(a[0], a[1])", [data, cap])

    def hide_image(self):
        self.pg.evaluate("hideImg()")
