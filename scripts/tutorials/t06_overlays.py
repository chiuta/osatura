# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 06: overlays."""
ID = "06-overlays"
KIND = "studio"


def run(r, url, proj):
    r.open_studio(url)
    r.say("c01", 5.5, lambda: r.click(r.vis("Overlays")))

    def toggles():
        boxes = r.pg.locator("input[type=checkbox]:visible")
        n = boxes.count()
        for i in range(n - 1, max(n - 6, -1), -1):
            try:
                r.click(boxes.nth(i))
                r.wait(0.7)
            except Exception:
                pass
    r.say("c02", 8.0, toggles)
    r.say("c03", 5.0, lambda: r.wait(3))
