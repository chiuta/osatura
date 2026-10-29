# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 05: the graph view: interpolation, Apply, Auto-smooth."""
ID = "05-graph"
KIND = "studio"


def run(r, url, proj):
    r.open_studio(url)

    def pick():
        r.drag((247, 641), (670, 658), 30)
        r.wait(0.4)
        r.click(r.vis("Graph"))
    r.say("c01", 6.0, pick)
    r.say("c02", 5.5, lambda: r.go((520, 600)))

    def choose():
        sel = r.pg.locator("select:visible").filter(has_text="linear").first
        r.go(sel)
        sel.select_option(index=2)
    r.say("c03", 5.5, choose)
    r.say("c04", 6.0, lambda: r.click(r.vis("Apply")))
    r.say("c05", 6.0, lambda: r.click(r.vis("Auto-smooth")))

    def play():
        r.click(r.transport_play())
        r.wait(3)
        r.click(r.transport_play())
        r.key("Control+z")
    r.say("c06", 7.5, play)
