# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 07: Suggest (in-between poses)."""
ID = "07-suggest"
KIND = "studio"


def run(r, url, proj):
    r.open_studio(url)
    r.say("c01", 5.0, lambda: r.go(r.vis("Suggest")))

    def look():
        r.click(r.vis("Suggest"))
        r.wait(0.8)
        r.click(r.pg.get_by_text("Suggest", exact=True).locator("visible=true").last)
        r.wait(1.5)
    r.say("c02", 6.5, look)

    def apply():
        btn = r.pg.get_by_text("Apply", exact=True).locator("visible=true").last
        r.click(btn)
    r.say("c03", 6.5, apply)
    r.say("c04", 5.0, lambda: r.key("Control+z"))
