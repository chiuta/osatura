#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Measure OSATURA on this machine: project size, open time, CPU render time per frame (through the C library, one thread) and the
size of the exports, for the built-in demo projects. Prints a Markdown table; --out FILE also writes it.

    cargo build --release -p osatura-cli -p osatura-capi && python3 scripts/benchmark.py --out bench.md

It measures OSATURA only. A comparison with another runtime needs the SAME rig in that runtime, on the same machine, which cannot be
shipped here (see docs/BENCHMARKS.md, "Comparing with another tool"). Numbers depend on the CPU, so quote them with the machine line."""
import ctypes, glob, json, os, platform, statistics, subprocess, sys, tempfile, time

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
rel = os.path.join(root, "target", "release")
BIN = os.path.join(rel, "osatura")
LIB = (glob.glob(os.path.join(rel, "libosatura_capi.so")) + glob.glob(os.path.join(rel, "libosatura_capi.dylib")) + glob.glob(os.path.join(rel, "osatura_capi.dll")) or [None])[0]
if not os.path.exists(BIN) or not LIB:
    sys.exit("build first: cargo build --release -p osatura-cli -p osatura-capi")
L = ctypes.CDLL(LIB)
L.osatura_open.restype = ctypes.c_void_p
L.osatura_open.argtypes = [ctypes.c_char_p, ctypes.c_size_t]
L.osatura_close.argtypes = [ctypes.c_void_p]
L.osatura_frame_count.restype = ctypes.c_uint32
L.osatura_frame_count.argtypes = [ctypes.c_void_p]
L.osatura_render_rgba.argtypes = [ctypes.c_void_p, ctypes.c_float, ctypes.c_float, ctypes.c_void_p, ctypes.c_size_t, ctypes.POINTER(ctypes.c_uint32), ctypes.POINTER(ctypes.c_uint32)]
L.osatura_version.restype = ctypes.c_char_p


def cpu():
    try:
        for line in open("/proc/cpuinfo"):
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return platform.processor() or platform.machine()


def ms(f, n):
    t = []
    for _ in range(n):
        a = time.perf_counter()
        f()
        t.append((time.perf_counter() - a) * 1000)
    t.sort()
    return statistics.median(t), t[max(0, int(len(t) * 0.95) - 1)]


rows = []
with tempfile.TemporaryDirectory() as d:
    for name, cmd in [("demo", "demo"), ("person", "demo-person"), ("fx", "demo-fx")]:
        p = os.path.join(d, name + ".json")
        subprocess.run([BIN, cmd, p], check=True, capture_output=True)
        doc = json.load(open(p))
        doc["frames"] = min(doc.get("frames", 24), 24)
        json.dump(doc, open(p, "w"))
        js = open(p, "rb").read()
        open_med, _ = ms(lambda: L.osatura_close(L.osatura_open(js, len(js))), 20)
        h = L.osatura_open(js, len(js))
        n = L.osatura_frame_count(h)
        w, hh = ctypes.c_uint32(), ctypes.c_uint32()
        for scale in (0.5, 1.0):
            L.osatura_render_rgba(h, 0.0, scale, None, 0, ctypes.byref(w), ctypes.byref(hh))
            buf = (ctypes.c_ubyte * (w.value * hh.value * 4))()
            fr = [0]

            def one():
                L.osatura_render_rgba(h, float(fr[0] % n), scale, buf, len(buf), ctypes.byref(w), ctypes.byref(hh))
                fr[0] += 1
            one()
            med, p95 = ms(one, 60)
            rows.append((name, f"{w.value}x{hh.value}", n, len(js), open_med, med, p95, 1000 / med))
        L.osatura_close(h)
        sizes = {}
        for ext, extra in [("lottie.json", []), ("png", ["--columns", "6", "--scale", "0.25"]), ("apng", ["--scale", "0.5"])]:
            out = os.path.join(d, name + "." + ext)
            r = subprocess.run([BIN, "export", p, out] + extra, capture_output=True)
            sizes[ext] = os.path.getsize(out) if r.returncode == 0 and os.path.exists(out) else None
        rows[-1] = rows[-1] + (sizes,)
        rows[-2] = rows[-2] + (sizes,)

ver = L.osatura_version().decode()
lines = [
    f"Machine: {cpu()}, {os.cpu_count()} logical cores, {platform.system()} {platform.release()}; osatura {ver}, release build, one render thread, median of 60 frames after a warm-up frame.",
    "",
    "| project | render size | frames | JSON bytes | open ms | render ms (median) | render ms (p95) | frames/s |",
    "|---|---|---|---|---|---|---|---|",
]
for r in rows:
    lines.append(f"| {r[0]} | {r[1]} | {r[2]} | {r[3]} | {r[4]:.2f} | {r[5]:.2f} | {r[6]:.2f} | {r[7]:.0f} |")
lines += ["", "Export sizes (24 frames; Lottie holds the vector layers only):", "", "| project | Lottie .json | sprite sheet .png (6 columns, scale 0.25) | APNG (scale 0.5) |", "|---|---|---|---|"]
seen = set()
for r in rows:
    if r[0] in seen:
        continue
    seen.add(r[0])
    s = r[8]
    f = lambda v: "-" if v is None else f"{v}"
    lines.append(f"| {r[0]} | {f(s['lottie.json'])} | {f(s['png'])} | {f(s['apng'])} |")
text = "\n".join(lines)
print(text)
if "--out" in sys.argv:
    open(sys.argv[sys.argv.index("--out") + 1], "w", encoding="utf-8").write(text + "\n")
