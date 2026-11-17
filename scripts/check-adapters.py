#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Check the engine adapters against what they depend on, without Unity or Godot installed:
  * every `osatura_*` function the Unity bindings import exists in osatura-capi/osatura.h, with the same number of arguments;
  * every key the Godot script reads from the atlas exists in a real atlas written by the osatura binary.
Usage: scripts/check-adapters.py [path-to-osatura-binary]   (default target/debug/osatura; the atlas check is skipped if absent)."""
import json, os, re, subprocess, sys, tempfile

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
problems = []

# ---- Unity: C# DllImport vs the header ----
hdr = open(os.path.join(root, "osatura-capi/osatura.h"), encoding="utf-8").read()
hdr = re.sub(r"/\*.*?\*/", "", hdr, flags=re.S)
protos = {}
for m in re.finditer(r"([\w\s\*]+?)\b(osatura_\w+)\s*\(([^)]*)\)\s*;", hdr):
    args = m.group(3).strip()
    n = 0 if args in ("", "void") else args.count(",") + 1
    protos[m.group(2)] = n
cs = open(os.path.join(root, "adapters/unity/Runtime/Osatura.cs"), encoding="utf-8").read()
seen = 0
for m in re.finditer(r"\[DllImport\(Lib\)\]\s*public static extern\s+[\w\.]+\s+(osatura_\w+)\s*\(([^)]*)\)", cs):
    seen += 1
    name, args = m.group(1), m.group(2).strip()
    n = 0 if args == "" else args.count(",") + 1
    if name not in protos:
        problems.append(f"Unity binding {name} is not in osatura.h")
    elif protos[name] != n:
        problems.append(f"Unity binding {name} takes {n} arguments, osatura.h declares {protos[name]}")
if seen < 10:
    problems.append(f"only {seen} DllImport lines found; the pattern in this script no longer matches Osatura.cs")
abi = re.search(r"#define OSATURA_ABI_VERSION (\d+)", hdr + open(os.path.join(root, "osatura-capi/osatura.h"), encoding="utf-8").read())
exp = re.search(r"ExpectedAbi = (\d+)", cs)
if not abi or not exp or abi.group(1) != exp.group(1):
    problems.append(f"ABI version: header {abi and abi.group(1)} vs Unity adapter {exp and exp.group(1)}")

# ---- Godot: atlas keys vs a real atlas ----
gd = open(os.path.join(root, "adapters/godot/addons/osatura/osatura_sheet.gd"), encoding="utf-8").read()
top = set(re.findall(r'parsed\.get\("(\w+)"', gd))
per = set(re.findall(r'f\["(\w+)"\]', gd))
bin_ = sys.argv[1] if len(sys.argv) > 1 else os.path.join(root, "target/debug/osatura")
if os.path.exists(bin_):
    with tempfile.TemporaryDirectory() as d:
        proj, png = os.path.join(d, "p.json"), os.path.join(d, "s.png")
        subprocess.run([bin_, "demo", proj], check=True, capture_output=True)
        doc = json.load(open(proj)); doc["frames"] = 4; json.dump(doc, open(proj, "w"))
        subprocess.run([bin_, "export", proj, png, "--columns", "2", "--scale", "0.2"], check=True, capture_output=True)
        atlas = json.load(open(os.path.join(d, "s.atlas.json")))
        for k in sorted(top):
            if k not in atlas:
                problems.append(f"Godot script reads atlas key '{k}' that the CLI does not write")
        for k in sorted(per):
            if k not in atlas["frames"][0]:
                problems.append(f"Godot script reads frame key '{k}' that the CLI does not write")
        if len(atlas["frames"]) != 4:
            problems.append(f"expected 4 frames in the atlas, got {len(atlas['frames'])}")
else:
    print(f"note: {bin_} not found, atlas check skipped")
if len(top) < 2 or len(per) < 4:
    problems.append("the Godot script's atlas reads were not found (pattern out of date?)")

# ---- the call sequence the Unity adapter uses, replayed through the real shared library (ctypes) ----
import ctypes, glob
libs = glob.glob(os.path.join(root, "target/*/libosatura_capi.so")) + glob.glob(os.path.join(root, "target/*/libosatura_capi.dylib")) + glob.glob(os.path.join(root, "target/*/osatura_capi.dll"))
if libs and os.path.exists(bin_):
    L = ctypes.CDLL(libs[0])
    L.osatura_open.restype = ctypes.c_void_p
    L.osatura_open.argtypes = [ctypes.c_char_p, ctypes.c_size_t]
    L.osatura_close.argtypes = [ctypes.c_void_p]
    L.osatura_abi_version.restype = ctypes.c_uint32
    L.osatura_frame_count.restype = ctypes.c_uint32
    L.osatura_frame_count.argtypes = [ctypes.c_void_p]
    L.osatura_render_rgba.argtypes = [ctypes.c_void_p, ctypes.c_float, ctypes.c_float, ctypes.c_void_p, ctypes.c_size_t, ctypes.POINTER(ctypes.c_uint32), ctypes.POINTER(ctypes.c_uint32)]
    L.osatura_last_error.restype = ctypes.c_char_p
    if exp and L.osatura_abi_version() != int(exp.group(1)):
        problems.append(f"library ABI {L.osatura_abi_version()} vs adapter {exp.group(1)}")
    with tempfile.TemporaryDirectory() as d:
        proj = os.path.join(d, "p.json")
        subprocess.run([bin_, "demo", proj], check=True, capture_output=True)
        js = open(proj, "rb").read()
        bad = L.osatura_open(b"{not json", 9)
        if bad:
            problems.append("a broken project was accepted by osatura_open")
        elif not L.osatura_last_error():
            problems.append("osatura_last_error is empty after a failed open")
        h = L.osatura_open(js, len(js))
        if not h:
            problems.append("osatura_open failed on the demo project")
        else:
            w, hh = ctypes.c_uint32(), ctypes.c_uint32()
            r = L.osatura_render_rgba(h, 0.0, 0.5, None, 0, ctypes.byref(w), ctypes.byref(hh))
            if r != -2 or w.value == 0 or hh.value == 0:
                problems.append(f"size query: expected -2 with a size, got {r} and {w.value}x{hh.value}")
            n = w.value * hh.value * 4
            a, b = (ctypes.c_ubyte * n)(), (ctypes.c_ubyte * n)()
            ra = L.osatura_render_rgba(h, 3.0, 0.5, a, n, ctypes.byref(w), ctypes.byref(hh))
            rb = L.osatura_render_rgba(h, 3.0, 0.5, b, n, ctypes.byref(w), ctypes.byref(hh))
            if ra != 0 or rb != 0:
                problems.append(f"render returned {ra}, {rb}")
            elif bytes(a) != bytes(b):
                problems.append("two renders of the same frame differ")
            elif not any(bytes(a)[3::4]):
                problems.append("the rendered frame is completely transparent")
            if L.osatura_render_rgba(h, float("nan"), 1.0, a, n, ctypes.byref(w), ctypes.byref(hh)) >= 0:
                problems.append("a NaN frame was accepted")
            small = (ctypes.c_ubyte * 8)()
            if L.osatura_render_rgba(h, 0.0, 0.5, small, 8, ctypes.byref(w), ctypes.byref(hh)) != -2:
                problems.append("a too-small buffer did not return -2")
            L.osatura_close(h)
            print(f"C library replay ok: {w.value}x{hh.value} frame, deterministic, size query and error paths behave")
else:
    print("note: no built osatura-capi library, C replay skipped")

if problems:
    print("\n".join("PROBLEM: " + p for p in problems))
    sys.exit(1)
print(f"adapters ok: {seen} Unity imports match osatura.h; Godot reads {sorted(top)} and {sorted(per)} from the atlas")
