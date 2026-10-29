# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 12: render and export from the command line."""
ID = "12-cli-export"
KIND = "terminal"


def run(rec, work):
    import t11_cli_buddy as b
    from common import Term
    import subprocess
    t = Term(rec, work)
    (t.cwd / "buddy.json").write_text(b.HEADER)
    for src, dst, name, text in (("buddy.json", "01.json", "s.rhai", b.SHAPES), ("01.json", "02.json", "b.rhai", b.BONES),
                                 ("02.json", "03.json", "i.rhai", b.IK), ("03.json", "04.json", "k.rhai", b.KEYS)):
        (t.cwd / name).write_text(text)
        subprocess.run(["osatura", "script", src, dst, name], cwd=t.cwd, env=t.env, check=True, capture_output=True)
    subprocess.run(["osatura", "loop-solve", "04.json", "buddy-final.json"], cwd=t.cwd, env=t.env, check=True, capture_output=True)
    t.say("c01", 4.5, lambda: (t.comment("buddy-final.json is ready"), t.run("osatura stats buddy-final.json", lines=8, hold=1.5)))

    def still():
        t.run("osatura render buddy-final.json 12 frame12.png --scale 2", lines=2, hold=0.4)
        t.image(t.cwd / "frame12.png")
        t.wait(2.5)
        t.hide_image()
    t.say("c02", 8.0, still)
    t.say("c03", 7.0, lambda: t.run("osatura frames buddy-final.json frames --png", lines=2))

    def export():
        for out in ("buddy.gif", "buddy.html", "buddy.json"):
            t.run(f"osatura export buddy-final.json {out}" if out != "buddy.json" else "osatura export buddy-final.json buddy-lottie.json", lines=2, hold=0.6)
        t.run("ls -l buddy.gif buddy.html buddy-lottie.json", lines=4, hold=2.0)
    t.say("c04", 12.0, export)
    t.say("c05", 7.0, lambda: t.run("osatura lint buddy-final.json", lines=3, hold=2.0))
