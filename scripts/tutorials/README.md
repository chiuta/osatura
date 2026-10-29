<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Video tutorials (generated)

The tutorials are recorded by scripts from the real program (the studio in headless Chromium, or real commands typed
into a simulated terminal), so they can be remade after every interface change. Subtitles in eight languages:
en, ro, fr, it, es, pt, de, hu. There is no voice. The video files are not stored in the repository.

| ID | Tutorial |
|---|---|
| 01-tour | tour of the studio |
| 02-select | selecting and moving layers |
| 03-bones | bones and posing |
| 04-timeline | the dope sheet: keys, copy, range, zoom |
| 05-graph | the graph view: interpolation, auto-smooth |
| 06-overlays | onion skin, motion trails, scrub proxy |
| 07-suggest | in-between pose suggestions |
| 08-layout | layouts, shortcuts, command palette, simple view |
| 09-export | GIF, HTML player, Lottie |
| 10-cli-start | command line: first steps |
| 11-cli-buddy | build a character from commands (docs/GETTING_STARTED.md) |
| 12-cli-export | render and export from the command line |

| File | Role |
|---|---|
| `make.py` | records and builds: `python3 -I make.py OUTDIR [ID-prefix ...]` (needs `target/release/osatura`, Python Playwright with Chromium, ffmpeg with libass) |
| `common.py`, `terminal.html` | recorder with a drawn mouse pointer, the terminal simulation, the studio launcher |
| `tNN_*.py` | one scenario per tutorial (`ID`, `KIND`, `run()`) |
| `captions/NN-*.json` | the captions, each in eight languages |
| `build.py` | writes `.srt`/`.vtt` per language, `tutorial-<ID>.mp4` with eight switchable subtitle tracks and `tutorial-<ID>.<lang>.mp4` with burned-in subtitles |

The translations were written by the assistant and **have not been reviewed by native speakers**; corrections to
`captions/*.json` are welcome.
