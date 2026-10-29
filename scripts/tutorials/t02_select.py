# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 02: selecting and moving things, undo, hiding and locking layers."""
ID = "02-select"
KIND = "studio"


def run(r, url, proj):
    r.open_studio(url)
    r.say("c01", 5.0, lambda: r.key("Alt+t"))
    sun, hills = (676, 270), (240, 530)

    def shift_click(xy):
        r.go(xy)
        r.pg.keyboard.down("Shift")
        r.pg.mouse.down(); r.wait(0.09); r.pg.mouse.up()
        r.pg.keyboard.up("Shift")

    def row(name, col):
        # the visible box is the first checkbox of a layer row, the lock box the second; rows are found by their text
        y = r.center(r.pg.get_by_text(name, exact=True).locator("visible=true").first)[1]
        return (968 if col == 0 else 990, y)

    r.say("c02", 5.5, lambda: r.click(sun))
    r.say("c03", 5.0, lambda: shift_click(hills))
    r.say("c04", 6.0, lambda: r.click(r.vis("Selection")))

    def move_sun():
        r.key("Escape")
        r.click(sun)
        r.drag(sun, (520, 215), 40)
    r.say("c05", 6.5, move_sun)
    r.say("c06", 6.0, lambda: [(r.key("ArrowRight"), r.wait(0.25)) for _ in range(4)] + [(r.key("Shift+ArrowDown"), r.wait(0.4)) for _ in range(3)])
    r.say("c07", 6.5, lambda: (r.key("Control+z"), r.wait(1.0), r.key("Control+z"), r.wait(1.0), r.key("Control+y"), r.wait(1.0)))

    def hide_show():
        r.click(r.vis("Layers"))
        r.wait(0.6)
        r.click(row("hills", 0))
        r.wait(1.2)
        r.click(row("hills", 0))
    r.say("c08", 6.5, hide_show)

    def lock_sun():
        r.key("Escape")
        r.click(row("sun", 1))
        r.wait(0.6)
        r.click((520, 215))   # where the sun is now: the click passes through the locked layer
    r.say("c09", 6.5, lock_sun)
    r.say("c10", 4.0, lambda: r.key("Escape"))
