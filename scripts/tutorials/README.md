<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Video tutorials (generated)

The tutorials are recorded from the real studio by a script, so they can be remade after every interface change.

| File | Role |
|---|---|
| `record.py` | drives `osatura studio` in headless Chromium (Playwright), records 1280x720 video with a drawn mouse pointer and writes the caption timing (`cues.json`) |
| `captions.json` | the captions of tutorial 01 in eight languages: en, ro, fr, it, es, pt, de, hu |
| `build.py` | writes `.srt` and `.vtt` for each language and the videos: one `tutorial-01.mp4` with eight switchable subtitle tracks, and `tutorial-01.<lang>.mp4` with the subtitles burned in |

```
osatura demo-person first-rig.json
osatura studio first-rig.json --port 8765 --ui-config fresh-ui.json &     # a new, empty --ui-config: the studio remembers its layout
python3 -I scripts/tutorials/record.py http://127.0.0.1:8765 rec
python3 -I scripts/tutorials/build.py rec out
```

Tutorial 01 is a tour of the studio (play, frames, layers, bones, keys, commands, exports, the guided demo). There is no
voice: the videos carry subtitles only. The translations were written by the assistant and **have not been reviewed by
native speakers**; corrections to `captions.json` are welcome. The video files are not stored in the repository.
