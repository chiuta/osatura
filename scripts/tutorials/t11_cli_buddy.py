# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Tutorial 11: building Buddy from commands (docs/GETTING_STARTED.md, steps 1-6)."""
ID = "11-cli-buddy"
KIND = "terminal"

SHAPES = """add_gradient_rect("sky", 0, 0, 320, 240, "#bfe3f5", "#fdf3d8");
add_rect("ground", 0, 200, 320, 40, "#7fa66b");
add_rect("torso", 140, 120, 40, 70, "#e07a5f");
add_circle("head", 160, 95, 24, "#f2cc8f");
add_rect("upper_arm", 160, 134, 40, 12, "#3d405b");
add_rect("forearm", 200, 134, 40, 12, "#81b29a");
"""
BONES = """add_bone("body", "", 70, 160, 190, -90);
add_bone("neck", "body", 20, 70, 0, 0);
add_bone("arm_up", "body", 40, 50, 0, 90);
add_bone("arm_low", "arm_up", 40, 40, 0, 0);
add_bone("hand_ctrl", "", 10, 210, 165, 0);
set_layer("torso", "bone", "body");
set_layer("head", "bone", "neck");
set_layer("upper_arm", "bone", "arm_up");
set_layer("forearm", "bone", "arm_low");
"""
IK = 'add_ik("arm_ik", ["arm_up", "arm_low"], "hand_ctrl", 1);\n'
KEYS = """set_key("hand_ctrl", "bone_offset_y", 0, 0, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 12, -75, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 24, 0, "ease_in_out");
set_key("neck", "bone_angle", 0, 0, "smooth");
set_key("neck", "bone_angle", 16, 10, "smooth");
set_key("neck", "bone_angle", 24, 0, "smooth");
"""
HEADER = '{"format": 1, "name": "Buddy", "width": 320, "height": 240, "fps": 24, "frames": 24}\n'


def run(rec, work):
    from common import Term
    t = Term(rec, work)
    w = lambda n, s: (t.cwd / n).write_text(s)
    t.say("c01", 4.5, lambda: t.comment("Buddy, built only with commands"))

    def empty():
        w("buddy.json", HEADER)
        t.run("cat buddy.json", lines=3, hold=0.8)
        t.run("osatura check buddy.json", lines=3)
    t.say("c02", 8.0, empty)

    def shapes():
        w("shapes.rhai", SHAPES)
        t.run("cat shapes.rhai", lines=8, hold=1.5)
        t.run("osatura script buddy.json 01-shapes.json shapes.rhai", lines=3)
    t.say("c03", 9.0, shapes)

    def bones():
        w("bones.rhai", BONES)
        t.run("cat bones.rhai", lines=10, hold=1.5)
        t.run("osatura script 01-shapes.json 02-bones.json bones.rhai", lines=3)
    t.say("c04", 10.0, bones)

    def still():
        t.run("osatura render 02-bones.json 0 rig.png --bones --scale 2", lines=3, hold=0.5)
        t.image(t.cwd / "rig.png")
        t.wait(2.5)
        t.hide_image()
    t.say("c05", 8.0, still)

    def ik():
        w("ik.rhai", IK)
        t.run("cat ik.rhai", lines=3, hold=1.0)
        t.run("osatura script 02-bones.json 03-ik.json ik.rhai", lines=3)
    t.say("c06", 7.5, ik)

    def keys():
        w("keys.rhai", KEYS)
        t.run("cat keys.rhai", lines=8, hold=1.5)
        t.run("osatura script 03-ik.json 04-keys.json keys.rhai", lines=3)
        t.run("osatura timeline 04-keys.json", lines=10, hold=1.5)
    t.say("c07", 10.0, keys)

    def loop():
        t.run("osatura loop-solve 04-keys.json buddy-final.json", lines=3)
        t.run("osatura lint buddy-final.json", lines=3, hold=2.5)
    t.say("c08", 8.0, loop)
