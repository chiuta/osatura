# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 04: the dope sheet: selecting, moving, copying keys, range, zoom, delete."""
ID = "04-timeline"
KIND = "studio"

X0, DX = 255, 29.8          # x of frame 0 and pixels per frame in the default view
ROW = 649                   # the bone_angle row of hips
RULER = 557


def fx(f):
    return int(X0 + DX * f)


def run(r, url, proj):
    r.open_studio(url)
    r.say("c01", 6.0, lambda: r.go((fx(8), ROW)))

    def pick():
        r.click((fx(4), ROW))
        r.wait(0.5)
        r.pg.keyboard.down("Shift")
        r.click((fx(8), ROW))
        r.pg.keyboard.up("Shift")
    r.say("c02", 6.0, pick)

    def box():
        r.key("Escape")
        r.drag((fx(9) - 8, 641), (fx(14) + 8, 658), 30)
    r.say("c03", 5.5, box)
    r.say("c04", 6.0, lambda: [(r.key("Alt+ArrowRight"), r.wait(0.5)) for _ in range(2)] + [(r.key("Alt+ArrowLeft"), r.wait(0.5)) for _ in range(2)])

    def copy_paste():
        r.key("Control+c")
        r.wait(0.5)
        r.click((fx(20), RULER))
        r.wait(0.5)
        r.key("Control+v")
    r.say("c05", 7.0, copy_paste)

    def rng():
        r.click((fx(4), RULER))
        r.click(r.vis("Set in"))
        r.click((fx(12), RULER))
        r.click(r.vis("Set out"))
        r.click(r.vis("range"))
        r.click(r.transport_play())
        r.wait(2.5)
        r.click(r.transport_play())
    r.say("c06", 9.5, rng)

    def zoom():
        r.click(r.vis("+"))
        r.click(r.vis("+"))
        r.wait(0.8)
        r.click(r.vis("-"))
        r.wait(0.8)
        r.click(r.vis("Fit"))
    r.say("c07", 6.5, zoom)

    def delete():
        r.click(r.vis("range"))                      # range off again
        r.click((fx(6), ROW))
        r.wait(0.6)
        r.key("Delete")
        r.wait(1.3)
        r.key("Control+z")
    r.say("c08", 7.0, delete)
