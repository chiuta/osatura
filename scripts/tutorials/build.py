#!/usr/bin/env python3 -I
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Turns a recording (raw.webm + cues.json) and captions/<ID>.json into subtitles and videos.

    python3 -I build.py ID RECDIR OUTDIR

OUTDIR gets: tutorial-<ID>.<lang>.srt and .vtt for en ro fr it es pt de hu; tutorial-<ID>.mp4 (one video with the eight
subtitle tracks, switchable in a player, none shown by default); tutorial-<ID>.<lang>.mp4 (subtitles burned in).
Needs ffmpeg with libass."""
import json
import subprocess
import sys
from pathlib import Path

LANGS = {"en": "English", "ro": "Română", "fr": "Français", "it": "Italiano", "es": "Español", "pt": "Português", "de": "Deutsch", "hu": "Magyar"}
ISO3 = {"en": "eng", "ro": "ron", "fr": "fra", "it": "ita", "es": "spa", "pt": "por", "de": "deu", "hu": "hun"}
HERE = Path(__file__).resolve().parent
STYLE = "FontName=DejaVu Sans,FontSize=13,Bold=1,PrimaryColour=&H00FFFFFF,OutlineColour=&H00000000,BackColour=&H80000000,BorderStyle=3,Outline=1,Shadow=0,MarginV=22"


def ts(sec, comma):
    ms = int(round(sec * 1000))
    h, ms = divmod(ms, 3600000)
    m, ms = divmod(ms, 60000)
    s, ms = divmod(ms, 1000)
    return f"{h:02}:{m:02}:{s:02}{',' if comma else '.'}{ms:03}"


def check_captions(caps, cues, tid):
    for c in cues:
        if c["id"] not in caps:
            sys.exit(f"{tid}: caption {c['id']} is missing in captions/{tid}.json")
        miss = [l for l in LANGS if not caps[c["id"]].get(l, "").strip()]
        if miss:
            sys.exit(f"{tid}: caption {c['id']} has no text for {miss}")
        for l in LANGS:
            if max(len(x) for x in caps[c["id"]][l].split("\n")) > 62 or caps[c["id"]][l].count("\n") > 1:
                print(f"warning: {tid} {c['id']} {l}: line too long or more than two lines", file=sys.stderr)


def main(tid, rec, out):
    rec, out = Path(rec), Path(out)
    out.mkdir(parents=True, exist_ok=True)
    caps = json.loads((HERE / "captions" / f"{tid}.json").read_text(encoding="utf-8"))
    cues = json.loads((rec / "cues.json").read_text())
    check_captions(caps, cues, tid)
    name = f"tutorial-{tid}"
    for lang in LANGS:
        srt, vtt = [], ["WEBVTT", ""]
        for i, c in enumerate(cues, 1):
            text = caps[c["id"]][lang]
            srt.append(f"{i}\n{ts(c['start'], True)} --> {ts(c['end'], True)}\n{text}\n")
            vtt.append(f"{ts(c['start'], False)} --> {ts(c['end'], False)}\n{text}\n")
        (out / f"{name}.{lang}.srt").write_text("\n".join(srt), encoding="utf-8")
        (out / f"{name}.{lang}.vtt").write_text("\n".join(vtt), encoding="utf-8")
    base = out / f"{name}.base.mp4"
    ff = ["ffmpeg", "-y", "-loglevel", "error"]
    subprocess.run(ff + ["-i", str(rec / "raw.webm"), "-c:v", "libx264", "-preset", "veryfast", "-crf", "23", "-pix_fmt", "yuv420p", "-r", "30", "-movflags", "+faststart", str(base)], check=True)
    cmd = ff + ["-i", str(base)]
    for lang in LANGS:
        cmd += ["-i", str(out / f"{name}.{lang}.srt")]
    cmd += ["-map", "0:v"]
    for i in range(len(LANGS)):
        cmd += ["-map", str(i + 1)]
    cmd += ["-c:v", "copy", "-c:s", "mov_text"]
    for i, lang in enumerate(LANGS):
        cmd += [f"-metadata:s:s:{i}", f"language={ISO3[lang]}", f"-metadata:s:s:{i}", f"title={LANGS[lang]}", f"-disposition:s:{i}", "0"]
    cmd += ["-movflags", "+faststart", str(out / f"{name}.mp4")]
    subprocess.run(cmd, check=True)
    for lang in LANGS:
        srt = f"{name}.{lang}.srt"
        subprocess.run(ff + ["-i", str(base), "-vf", f"subtitles={srt}:force_style='{STYLE}'", "-c:v", "libx264", "-preset", "veryfast", "-crf", "23", "-pix_fmt", "yuv420p", "-movflags", "+faststart", str(out / f"{name}.{lang}.mp4")], check=True, cwd=out)
    base.unlink()
    print(f"{tid}: {len(cues)} captions, nine videos in {out}")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    main(*sys.argv[1:])
