<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Engine adapters

Two small adapters that put OSATURA animations into a game engine. Both are **new and have not been run inside the engine**
by their author (neither Unity nor Godot is available in the build environment). What is checked, by
`scripts/check-adapters.py` (run in CI): every C function the Unity bindings import exists in `osatura-capi/osatura.h` with the same
number of arguments and the same ABI version; the call sequence the Unity adapter uses (open, size query, render, close, error paths) is
replayed through the real shared library and gives deterministic frames; every key the Godot script reads exists in a sprite-sheet
atlas written by the real `osatura` binary. Treat the rest as a starting point and report what breaks.

| Engine | Folder | How it plays | Needs |
|--------|--------|--------------|-------|
| Unity 2022.3+ | `unity/` | Renders frames on the CPU through the C library into a `Texture2D` on a `SpriteRenderer` | `osatura_capi` shared library in `Assets/Plugins/<platform>/` and the project JSON as a `TextAsset` |
| Godot 4 | `godot/addons/osatura/` | Plays a sprite sheet (a `Sprite2D` with a region) | `osatura export project.json hero.png --columns 8` gives `hero.png` and `hero.atlas.json`; no native code |

## Unity

1. Build the library: `cargo build --release -p osatura-capi` and copy `libosatura_capi.so` (Linux), `osatura_capi.dll` (Windows) or
   `libosatura_capi.dylib` (macOS) into `Assets/Plugins/`.
2. Copy `adapters/unity` into `Packages/tf.trom.osatura` (or reference it as a local package).
3. Drop the project `.json` into the project as a TextAsset, add **OSATURA / Osatura Player** to a GameObject and assign it.

Rendering is on the CPU every frame, so it suits a few characters, cut-scenes and UI. For many instances export a sprite sheet and
use it the normal way. Only the open, render and state-machine entry points are bound; the editing calls in `osatura.h` are not.

## Godot

Copy `adapters/godot/addons/osatura` into your project's `addons/`, enable the plugin, add an **OsaturaSheet** node, set `sheet` to the PNG and
`atlas_path` to the `.atlas.json`. It plays at the project's own frame rate; `loop`, `speed`, `seek()` and the `finished` signal are there.

## Licence

MPL-2.0, like the library they call. The README is CC-BY-4.0.
