# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 08: layout, keys, command palette, simple view."""
ID = "08-layout"
KIND = "studio"


def run(r, url, proj):
    r.open_studio(url)
    r.say("c01", 6.0, lambda: (r.click(r.vis("Layout")), r.wait(1.5)))
    r.key("Escape")
    r.say("c02", 6.0, lambda: (r.click(r.vis("Keys")), r.wait(2)))
    r.key("Escape")
    r.say("c03", 7.5, lambda: (r.click(r.vis("Commands")), r.wait(1), r.pg.keyboard.type("play", delay=180)))
    r.key("Escape")
    r.say("c04", 6.5, lambda: (r.click(r.vis("Simple view")), r.wait(2), r.click(r.vis("Simple view"))))
