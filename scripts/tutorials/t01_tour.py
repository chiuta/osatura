# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 01: a tour of the studio."""
ID = "01-tour"
KIND = "studio"


def run(r, url, proj):
    r.open_studio(url)
    r.say("c01", 5.5)
    r.say("c02", 7.5, lambda: r.click(r.transport_play()))

    def scrub():
        r.click(r.transport_play())  # pause
        bb = r.pg.locator("#tl").bounding_box()
        y = bb["y"] + bb["height"] / 2
        r.move(bb["x"] + 8, y)
        r.pg.mouse.down()
        r.move(bb["x"] + bb["width"] * 0.55, y, 40)
        r.pg.mouse.up()
    r.say("c03", 7.0, scrub)

    def layers():
        r.click(r.vis("Layers"))
        r.click(r.pg.get_by_text("arm_far").locator("visible=true").first)
    r.say("c04", 5.5, layers)
    r.say("c05", 6.0, lambda: r.click(r.vis("bones")))
    r.say("c06", 5.5, lambda: r.click(r.vis("Bones")))

    def key_click():
        # the timeline is drawn on a canvas, so the key (bone_angle row, frame 6) is addressed by position
        r.move(434, 649)
        r.pg.mouse.down(); r.wait(0.09); r.pg.mouse.up()
    r.say("c07", 6.5, key_click)
    r.say("c08", 6.0, lambda: r.click(r.vis("auto-key")))
    r.say("c09", 6.5, lambda: r.click(r.pg.locator("#palbtn")))

    def exports():
        r.key("Escape")
        r.wait(0.3)
        for sel in ("GIF", "HTML player", "Lottie"):
            r.go(r.vis(sel))
            r.wait(0.7)
    r.say("c10", 6.5, exports)

    def demo():
        r.click(r.pg.locator("#demobtn"))
        r.wait(0.6)
        r.click(r.pg.locator("#dm-start"))
    r.say("c11", 7.0, demo)

    def restore():
        for _ in range(2):
            r.click(r.pg.locator("#dm-next"))
            r.wait(0.9)
        r.click(r.pg.locator("#dm-restore"))
    r.say("c12", 7.0, restore)
    r.say("c13", 5.5, lambda: r.move(640, 360, 20))
