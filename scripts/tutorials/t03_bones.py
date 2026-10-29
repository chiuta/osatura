# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 03: posing with bones: sliders, canvas gizmo, keys, undo, auto-key."""
ID = "03-bones"
KIND = "studio"


def run(r, url, proj):
    r.open_studio(url)
    r.say("c01", 5.0, lambda: r.click(r.vis("bones")))
    r.say("c02", 6.0, lambda: r.click(r.vis("Bones")))
    r.say("c03", 5.0, lambda: (r.pg.keyboard.press("Shift+."), r.wait(0.6)))

    def hips():
        # the hips slider is on the row of the text "hips" in the Bones panel; its thumb starts at the middle
        y = r.center(r.vis("hips"))[1]
        r.go((1132, y))
        r.pg.mouse.down()
        r.move(1150, y, 20)
        r.wait(0.5)
        r.move(1121, y, 25)
        r.pg.mouse.up()
    r.say("c04", 7.0, hips)
    r.say("c05", 5.5, lambda: r.go((553, 649)))

    def tip():
        r.click((528, 392))            # a point on the forearm: selects the bone
        r.drag((562, 408), (600, 350), 30)
    r.say("c06", 6.5, tip)

    def play():
        r.key("Space")
        r.wait(3.5)
        r.key("Space")
    r.say("c07", 6.0, play)
    r.say("c08", 5.5, lambda: [(r.key("Control+z"), r.wait(1.2)) for _ in range(3)])

    def autokey():
        r.click(r.vis("auto-key"))
        r.wait(1.5)
        r.click(r.vis("auto-key"))
    r.say("c09", 6.0, autokey)
