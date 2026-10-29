# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 09: exporting from the studio."""
ID = "09-export"
KIND = "studio"


def run(r, url, proj):
    r.open_studio(url)
    r.say("c01", 5.0, lambda: r.go(r.vis("GIF")))
    for cid, name in (("c02", "GIF"), ("c03", "HTML player"), ("c04", "Lottie")):
        r.say(cid, 6.0, lambda n=name: (r.click(r.vis(n)), r.wait(2.5)))
