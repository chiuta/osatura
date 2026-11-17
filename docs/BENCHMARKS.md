<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Benchmarks

What OSATURA costs on one machine, measured by `scripts/benchmark.py`, and how to compare it fairly with another tool. These are
measurements of OSATURA alone. **No other runtime was measured**: a comparison needs the same rig in the other tool, which
cannot be shipped here (Spine, Rive and Moho runtimes are used under their own licences, with the user's own assets).

## Measured (17 November 2026)

Machine: Intel(R) Xeon(R) Processor @ 2.10GHz, 2 logical cores, Linux 6.18.44-fc-v553; osatura 0.1.0, release build, one render thread, median of 60 frames after a warm-up frame.

| project | render size | frames | JSON bytes | open ms | render ms (median) | render ms (p95) | frames/s |
|---|---|---|---|---|---|---|---|
| demo | 320x240 | 24 | 20057 | 1.08 | 4.84 | 5.23 | 207 |
| demo | 640x480 | 24 | 20057 | 1.08 | 15.15 | 15.89 | 66 |
| person | 320x240 | 24 | 124929 | 5.35 | 5.41 | 5.58 | 185 |
| person | 640x480 | 24 | 124929 | 5.35 | 14.03 | 14.51 | 71 |
| fx | 320x240 | 24 | 3591 | 0.14 | 1.27 | 1.88 | 789 |
| fx | 640x480 | 24 | 3591 | 0.14 | 2.87 | 3.63 | 348 |

Export sizes (24 frames; Lottie holds the vector layers only):

| project | Lottie .json | sprite sheet .png (6 columns, scale 0.25) | APNG (scale 0.5) |
|---|---|---|---|
| demo | 141795 | 148763 | 395645 |
| person | 657749 | 143089 | 456470 |
| fx | 722405 | 62398 | 246448 |

How to read it. The renderer is the CPU one (the default); the GPU renderer is not part of these numbers (docs/LIMITATIONS.md, L-05). The
machine is a shared two-core virtual machine, so absolute times will be better on a desktop and the **ratios** between rows are
the useful part. "Render" is one frame through the C library at the stated size, single-threaded, including the transfer into a caller buffer.
"Open" parses and compiles the JSON. The demo projects are small and made by the author; they say nothing about a production character
with hundreds of layers and meshes.

## Comparing with another tool

To compare OSATURA with, say, the Spine runtime, run these on **one machine**, one after the other, with the same character:

1. **Same rig.** Build one character with the same bone count and the same number of mesh vertices in both tools. Say how many bones, how many meshes, how many vertices and how many animated channels. A rig that is simpler in one tool is not a comparison.
2. **File size.** The project file as the runtime loads it (OSATURA: the `.json`; the other tool: its export, binary and JSON if both exist), and the same compressed with gzip.
3. **Load time.** Median of 20 loads from memory, after one warm-up.
4. **Frame cost.** Median and 95th percentile of 60 frames of the same animation, for the part the runtime does (pose, skinning, mesh update). State whether drawing is included. OSATURA's number above includes rasterisation on the CPU; a game runtime usually hands vertices to the GPU, so compare "pose and skin" with "pose and skin", not with a full CPU render.
5. **Memory.** Peak resident memory of the process with the character loaded.
6. **Say what is missing.** Features the tool you are measuring does that the other does not (or the reverse) belong in the same table.

Report the machine, the versions and the commands. A result without them cannot be checked and should not be quoted.

## Reproduce

```
cargo build --release -p osatura-cli -p osatura-capi
python3 scripts/benchmark.py --out bench.md
```

The script needs Python 3 only. On Windows and macOS the library name differs and the script looks for it.
