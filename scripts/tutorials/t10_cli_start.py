# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 10: first steps on the command line."""
ID = "10-cli-start"
KIND = "terminal"


def run(rec, work):
    from common import Term
    t = Term(rec, work)
    t.say("c01", 5.0, lambda: t.comment("osatura: everything from the command line"))
    t.say("c02", 6.0, lambda: t.run("osatura demo-person first-rig.json", lines=4))
    t.say("c03", 5.5, lambda: t.run("osatura check first-rig.json", lines=4))
    t.say("c04", 7.0, lambda: t.run("osatura timeline first-rig.json", lines=12, hold=2.5))

    def still():
        t.run("osatura render first-rig.json 12 still.png --scale 1", lines=3)
        t.image(t.cwd / "still.png")
        t.wait(2.5)
    t.say("c05", 8.0, still)
