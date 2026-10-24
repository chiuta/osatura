<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# OSATURA command-line reference

Reference for the `osatura` binary (crate `osatura-cli`, AGPL-3.0-or-later; it links the MPL-2.0 engine
`osatura-core`). Documented version: **osatura 0.1.0**. This text is licensed CC-BY-4.0.

Every synopsis below is copied from the program's own usage text (`osatura` with no arguments, or
`osatura <command> --help`). Every example was executed against the built-in demo project and the output shown
is trimmed from the real output. Where the usage text and the real behaviour disagree, this document describes the
real behaviour and lists the disagreement in [Known doc/behaviour mismatches](#known-docbehaviour-mismatches) at the end.

Related documents: [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (what every export/import format keeps and drops),
[CI.md](CI.md) (animation CI manifests), [API.md](API.md) (library/C ABI), [PATENT_NOTES.md](PATENT_NOTES.md)
(codec and patent status, not legal advice).

## Contents

1. [Conventions shared by all commands](#1-conventions-shared-by-all-commands)
2. [Getting started: the demo project](#2-getting-started-the-demo-project)
3. [Project creation and inspection](#3-project-creation-and-inspection)
4. [Rendering and export](#4-rendering-and-export)
5. [Rigging, skins, IK and layout helpers](#5-rigging-skins-ik-and-layout-helpers)
6. [Animation, keys, timeline and generators](#6-animation-keys-timeline-and-generators)
7. [Physics, cloth and deformers](#7-physics-cloth-and-deformers)
8. [Audio, lip-sync and sounds](#8-audio-lip-sync-and-sounds)
9. [Import (SVG, OpenRaster, Lottie, BVH, GIF/APNG, PNG sequences, bitmaps)](#9-import)
10. [Live input, MIDI, face tracking and motion capture](#10-live-input-midi-face-tracking-and-motion-capture)
11. [State machines, data bindings, containers and scroll regions](#11-state-machines-data-bindings-containers-and-scroll-regions)
12. [Effects, depth of field, onion skin, trails and camera](#12-effects-depth-of-field-onion-skin-trails-and-camera)
13. [Level of detail, size reports and export-fit](#13-level-of-detail-size-reports-and-export-fit)
14. [Quality assurance: check, lint, ci, golden, fuzz, diff](#14-quality-assurance)
15. [Version control (`vcs`)](#15-version-control-vcs)
16. [Studio, collaboration, MCP agent server and UI registry](#16-studio-collaboration-mcp-agent-server-and-ui-registry)
17. [Pose database, suggestions and other helpers](#17-pose-database-suggestions-and-other-helpers)
18. [Known doc/behaviour mismatches](#known-docbehaviour-mismatches)
19. [Alphabetical command index](#alphabetical-command-index)

---

## 1. Conventions shared by all commands

### 1.1 Command line shape

```
osatura <command> [sub-command] <positional arguments...> [options]
```

* Options may appear **anywhere** on the line, also after the sub-command and after positional arguments
  (`osatura render demo.json 0 o.png --scale 2` and `osatura --scale 2 render ...` are the same).
* **Value options** take the next word (`--scale 2`) or the `--key=value` form (`--scale=2`). The `=` form works for
  every value option. A value that itself contains `=` is fine after the space form (`--remap a=b`).
* **Flags** (boolean options such as `--bones`) take no value. `--bones=1` is refused:
  `--bones is a flag and takes no value: write --bones (without =), got '--bones=1'`.
* **Strict options.** An option (flag or value option) that the command does not document is an error with a
  "did you mean" hint (exit 1) instead of being ignored:
  ```
  $ osatura render demo.json 0 o.svg --bone
  unknown option --bone for 'render' (did you mean --bones?); see: osatura render --help
  ```
  The check applies to every command that has a usage line, including `check`, `stats`, `lint`, `coach`, `contracts` and
  `machine list` (older versions silently ignored extra flags there). `--json`, `--quiet` and `--allow-empty` are **not**
  global: they are refused by every command whose usage line does not list them.
  **The check is per sub-command (per synopsis form), not per command.** An option listed for one sub-form is refused on
  its siblings, and the message names the sub-form that accepts it:
  ```
  $ osatura ci diff a.png b.png --quiet
  unknown option --quiet for 'ci diff' (--quiet applies to: ci run); see: osatura ci --help
  ```
  So `--quiet` is for `ci run` only, `--json` only for `vcs log|status|diff|show`, `ui commands|shortcuts` and the
  `trail ... --target` form, `--tol` only for `golden check` (and `ci diff`), `--scale` only for `golden write`, and
  each `keyedit` edit accepts just the options written after its name in the synopsis. (`export` still pools its two
  synopsis lines, because the format is chosen by the file extension; `--codec`, `--columns` and `--scale` that do not
  apply to the chosen extension are reported by `export` itself.)
* Some older commands take `key=value` **words** instead of `--options` (`cloth`, `physics`, `cage`, `warp-pivot`,
  `soft-scale`, `mesh-lock`, `glue`, `audio-react`, `beats`, `import-seq`, `import-anim`, `layer-pivot`, `chan-meta`,
  `waveform`). These are positional and are written exactly as shown in the synopsis (`gravity=900`, not `--gravity 900`).
* A **frame** is a number (fractions allowed where it makes sense). A project with `frames = N` has frames `0..N-1`.
  Layers are addressed by **path** (`group/child`), bones by **name**; the camera has the empty target `""`.
* **Property names** (used by `--prop`, `slider`, `drive`, mapping files, ...) are the snake_case names:
  `bone_angle bone_scale bone_offset_x bone_offset_y layer_opacity layer_x layer_y layer_rotation layer_scale_x
  layer_scale_y layer_skew layer_switch layer_wobble camera_x camera_y camera_zoom camera_rotation camera_focus
  camera_blur ik_mix constraint_weight path_morph point_dx point_dy lattice_dx lattice_dy spine_dx spine_dy pin_dx
  pin_dy cage_dx cage_dy fx_blur fx_dx fx_dy fx_soft fx_amount grad_dx grad_dy`.  Some commands print the same properties in CamelCase (`BoneAngle`) in their messages; the option values are always snake_case.

### 1.2 Global options

| Option | Meaning |
|---|---|
| `--allow-external-assets` | Let a project name files by absolute path or outside its own directory (sounds, images, normal maps...). Default: refused. |
| `--follow-symlinks` | Write to the **target** of a symbolic link given as an output file. Default: refused. |
| `--max-input-bytes N` | Largest project file that is read. Default `268435456` (256 MiB). Must be a positive whole number. |
| `<command> --help` | Print the usage lines of one command to stdout (exit 0). |
| `<command> --version`, `--version` | `osatura 0.1.0`; with no command also the licence line and AGPL source offer. |
| `--json`, `--quiet`, `--allow-empty` | Per-command options, **refused** (`unknown option`) by commands whose synopsis does not list them (`--json`: `pick`, `select-box`, `timeline`, `trail`, `size-report`, `collab-sim`, `vcs`, `ui`, `face-detect`, `pose-suggest`; `--allow-empty`: `physics`, `live-map`, `face-bake`; `--quiet`: `ci run`). Scripts that passed them uniformly to every command must stop doing so. |

Examples (verified):

```
$ osatura --max-input-bytes 100 check demo.json
cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)
[exit 1]

$ osatura check /dev/zero
cannot read /dev/zero: not a regular file (devices, pipes and directories are refused)
[exit 1]

$ ln -s real_target.svg link.svg                       # dangling link
$ osatura render demo.json 0 link.svg
cannot write link.svg: link.svg: refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself
[exit 1]
```

### 1.3 Reading input files

* Input files must be **regular files** no larger than `--max-input-bytes`. Devices, pipes, directories and
  process substitutions (`<(...)`) are refused: write the data to a file first.
* Project files are JSON. A project with unknown keys is **accepted** and the unknown keys are reported as
  `warning: unknown field bogus_key (ignored)` by `check` (and `lint`, `studio`). Side files that you author for a command
  (contracts, mappings, machines, containers, scroll regions, ci manifests, drive maps ...) are parsed **strictly**: an unknown
  key is an error and nothing is applied (`unknown field(s): elastic.damping (typo? nothing was applied)`).
* A project is validated whenever it is loaded for rendering or export (`<file> is not a valid project:` followed by
  one problem per line). Editing commands also re-compile the project they are about to write and refuse to write one that no longer validates.
* **Asset paths** inside a project (sound files, image files, normal maps) are resolved relative to the **project file's
  directory**. Absolute paths and any path containing `..` are refused unless `--allow-external-assets` is given:
  ```
  $ osatura sound add s.json o.json x /etc/passwd
  sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)
  ```
  The flag must be given again on every later command that loads that project (`check ext.json` fails without it, succeeds with it).

### 1.4 Writing output files

* **Atomic writes.** Most commands write through a temporary file in the **same directory** (`.<name>.tmp-<pid>-<random>`), `fsync` it,
  copy the old file's permissions and `rename` it over the target, so a crash or a full disk never leaves a half-written file.
  If a temporary file cannot be created (read-only directory but writable file) the file is overwritten in place, which is **not** atomic.
  FIFOs and `/dev/stdout`-like special files are written directly.
* **Symbolic links** as output are refused unless `--follow-symlinks` is given (then the link's target is written; a dangling link is still an error).
  A directory given as an output file is an error (`is a directory`).
* **Never over the input.** `render`, `render-hdr`, `dof`, `dof-export`, `export` (also the `.json` sprite-sheet sidecar of a `.png`
  and `--report`), `layout-preview`, `render-gpu`, `export-fit`, `fuzz-grid`, `onion`, `trail` and `golden` refuse an output that is the same file
  as the input project: `refusing to overwrite the input project demo.json with output of the same name`.
* **Editing commands** (those with a `<project.json> <out.json>` pair: `reparent`, `keys`, `physics`, ...) are allowed to write **in place**
  (`osatura reparent p.json p.json hand none` works and replaces `p.json`). `import-lottie` refuses an output (or `fx.json`) that is
  the same file as its input; the other import commands read a different kind of file than they write.
* **Existing outputs are overwritten silently** by every command. There is no `--force`.
* **Every user-addressed output** (including `ci run` reports and goldens, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, the `face-*` files,
  pose files, snapshots, `vcs` and the studio collaboration log) goes through the same atomic, symlink-refusing writer. The
  `.mp4`/`.webm` outputs written by ffmpeg are checked for a symlink before ffmpeg starts. Parent folders: `lod`, `export-fit`, `ci` and `face-*`
  create missing folders; `fuzz`, `fuzz-grid` and `export` do not (`No such file or directory`).

### 1.5 stdout, stderr and exit codes

* Results ("wrote out.json", tables, JSON) go to **stdout**. Errors and most warnings go to **stderr**. `import-svg` prints its
  warnings on stdout; `export` prints Lottie/GIF/GLB warnings on stderr.
* Exit code **0** means success. Exit code **1** means any error *or* a failed check: an unreadable/invalid file, bad arguments,
  `lint` with errors, `diff` with differences, a violated contract, a mismatching golden, a fuzz failure rate above the limit,
  an `export-fit` budget that could not be met, `lod --verify` over the bound, `collab-sim` divergence, a failing `ci` check.
  Exit code **2** is used by `ci` only (nothing could be evaluated: missing project/manifest, tampered golden).
* No arguments, an unknown command, the wrong number of positional arguments, or `osatura --help` print the full usage on
  **stderr** and exit **1**. `osatura <command> --help` prints that command's usage lines on stdout and exits 0.
  **Exception (Windows, since 0.1.0-alpha.2):** when `osatura.exe` is started by a double-click from Explorer (the console window was created for it and no other process shares it) and has no arguments, it does not print the usage: it creates `Documents\OSATURA\first-rig.json` (from `demo-person`) if missing, runs `studio` on that file at `127.0.0.1:8080`, opens the browser and keeps the console open until you close it or press Enter after an error. Started from a terminal, it behaves as described above. The rest of the program runs on a thread with a 256 MiB stack (a Windows main thread has only 1 MiB).

### 1.6 External programs

`ffmpeg` (found on `PATH`) is required for: `.mp4`/`.webm` export and `dof-export` to those formats, `sound mix`,
audio in `.html` (mp3), and `face-track` on a video file. Needed encoders: `libx264` (mp4), `libvpx-vp9` / `libaom-av1` (webm),
`aac` (mp4 sound), `libopus` (webm sound), `libvorbis`, `libmp3lame` (sound mix). No ffmpeg code or binary is bundled; see
[PATENT_NOTES.md](PATENT_NOTES.md). Nothing else is executed.

### 1.7 Performance note

All timings in this document come from a **debug** build and are 10-30x slower than a release build. Frame-heavy commands
(`safety`, `golden`, `ci`, `export-fit`, `proxy-bench`) were run on a cut-down 8-frame copy of the demo (`small.json`, made by setting
`"frames": 8`).

---

## 2. Getting started: the demo project

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura export demo.json demo.gif
wrote demo.gif
```

The demo has bones `arm_up arm_low hand thigh shin foot_ctrl`, one IK chain `leg`, and top-level layers
`sky sun hills ground shadow arm leg target blush mouth(switch: closed/open/smile) ink`. All examples in this reference use it
unless they say otherwise.

---

## 3. Project creation and inspection

### `demo`

**Synopsis:** `osatura demo <out.json>`

Writes the built-in rig demo (640x480, 24 fps, 48 frames, 6 bones, IK leg, 14 layers) as a project file.
**Output:** `wrote <out.json>`. **Errors:** missing argument (usage, exit 1); unwritable path. **Related:** `demo-fx`, `check`.
**Limits:** the demo is a fixed asset; it is the base of every example in this document.

```
$ osatura demo demo.json
wrote demo.json
```

### `demo-person`

**Synopsis:** `osatura demo-person <out.json>`

Writes the walking-person demo (640x480, 24 fps, 32 frames, 42 bones, 45 layers) as a project file: a person seen from the side with head, ear and ponytail, neck, trunk, two arms with hands and five fingers each, and two legs with feet and toes. The walk cycle is two steps long and loops without a seam: the pelvis bobs and tilts, the trunk leans and counter-rotates, the head is kept level with a small lagging nod, the arms swing against the legs with the elbows bending more on the forward swing, and hands, fingers, ear and ponytail follow a little later. Each foot lands on the heel, rolls flat and leaves on the ball of the foot; the ground, the fence and the shadow scroll at exactly the speed of the planted foot, so the feet do not slide. This is the project that the Windows launchers (`osatura.exe`, `osatura-studio.exe`) create as `first-rig.json` on the first start; `osatura demo` stays the small base of the examples in this document.
**Output:** `wrote <out.json>`. **Errors:** missing argument (usage, exit 1); unwritable path. **Related:** `demo`, `studio`.
**Limits:** the keys are sampled every two frames with one extra key before frame 0 and one after the last frame, so the loop is smooth; `lint` reports those two keys as outside the timeline (information, not an error). The figure is a side view, so there is no turning or depth.

```
$ osatura demo-person person.json && osatura check person.json
wrote person.json
ok: 'OSATURA walking person' 640×480 @ 24 fps, 32 frames, 42 bones, 0 ik, 45 layers, 48 channels
```

### `demo-fx`

**Synopsis:** `osatura demo-fx <out.json>`

Writes the "liquid / boolean / particles" demo (640x480, 72 frames, 3 bones, 3 layers) that exercises procedural layers.

```
$ osatura demo-fx fx.json && osatura check fx.json
wrote fx.json
ok: 'OSATURA fx demo' 640×480 @ 24 fps, 72 frames, 3 bones, 0 ik, 3 layers, 3 channels
```

### `check`

**Synopsis:** `osatura check <project.json>`

Loads and validates a project. Prints `warning:` lines (unknown fields, gradient problems, switch-key problems, and the editor-data warnings of lint code L013: camera zoom <= 0, layer opacity outside 0..1, bad references or times in skins, comps, markers, events, shots, channel_meta, bone_vis, clips and lods) and one `ok:` summary line. Only values that cannot exist in a saved file (NaN or infinity in those structures) are errors, so older projects keep loading.
Sound tracks are opened too: an unreadable/unusable audio file is an **error**. **Exit:** 0 valid; 1 invalid.

```
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check unk.json                      # a project with two unknown keys
warning: unknown field bogus_key (ignored)
warning: unknown field bones[0].lenght (ignored)
ok: 'small' 640×480 @ 24 fps, 8 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check bad3.json                     # IK chain whose target bone does not exist
bad3.json is not a valid project:
  - ik 'leg': unknown target bone 'nope'
[exit 1]

$ osatura check bad.json                      # {"format":1,"name":"x"}
invalid JSON in bad.json: missing field `width`
[exit 1]
```

**Related:** `lint` (rig/animation quality), `ci` (regression gate). **Gotcha:** `check` does not render anything; a project can pass `check` and still
fail later on a frame (`NaN` poses are caught by `render`/`export`, which evaluate every frame first).

### `stats`

**Synopsis:** `osatura stats <project.json>`

Prints counts as JSON: `name, size, fps, frames, seconds, bones, ik_chains, constraints, springs, layers, channels, keyed_channels, keys, markers`.
The output is always JSON; there is no `--json` option (it is refused like any unknown option).

```
$ osatura stats demo.json
{ "name": "OSATURA demo", "size": [640.0, 480.0], "fps": 24.0, "frames": 48, "seconds": 2.0, "bones": 6,
  "ik_chains": 1, "constraints": 0, "springs": 0, "layers": 14, "channels": 14, "keyed_channels": 8, "keys": 37, "markers": 0 }
```
(real output is pretty-printed over several lines.) Note that `layers` counts every node including children of groups/switches, while `check` prints the same number.

### `diff`

**Synopsis:** `osatura diff <a.json> <b.json>`

Semantic difference between two projects: `+` added, `-` removed, `~` changed. Order-insensitive for lists addressed by name.
**Exit:** 0 identical (`no differences`); **1** when there are differences (the count `N difference(s)` goes to stderr), so it can gate a script.

```
$ osatura diff demo.json demo.json
no differences
$ osatura diff demo.json d2.json              # fps 24->30, arm_up longer, layer 'sun' deleted
~ fps: 24 → 30
~ bone 'arm_up' changed (length)
- layer 'sun' removed
3 difference(s)            (stderr)
[exit 1]
```
**Related:** `vcs diff` (history), `compare` (pixel difference), `ci diff` (image difference).

### `tag`

**Synopsis:** `osatura tag <project.json>`

Proposes standard role names (`arm.upper.l`, `hand.l`, `face.mouth` ...) for bones and layers from their names, with a confidence, and lists the standard roles for which no bone was found. Read-only; use the roles as a guide for `mocap-match` or `autorig`.

```
$ osatura tag demo.json
bone  arm_up                   -> arm.upper.l    (45%)
bone  hand                     -> hand.l         (45%)
layer mouth                    -> face.mouth     (90%)
no bone found for: head, neck, torso, hips, face.eye.l, ...
```

### `explain`

**Synopsis:** `osatura explain <project.json> <bone-or-layer> <frame>`

"Why is it here?" - lists the ordered contributions (rest pose, each channel with its mode and weight, constraints, IK) that produce the transform of one bone or layer at one frame, and the resulting head/tip/world angle.

```
$ osatura explain demo.json hand 12
- bone 'hand' (length 45), parent 'arm_low'
- ancestry: hand ← arm_low ← arm_up
- rest pose: offset (100.00, 0.00), angle 0.00°
- channel #2 BoneAngle: keys → 20.000 (5 key(s), Hold outside range); mode replace, weight 1.00
- channel #3 BoneAngle: expression `6 * sin(time * 2 * pi * 2)`; mode add, weight 1.00
- result at frame 12: head (174.6, 126.4), tip (203.5, 92.0), world angle -50.0°
```
**Errors:** unknown bone/layer name; frame outside the animation.

### `pick`

**Synopsis:** `osatura pick <project.json> --frame N --x X --y Y [--tol PX] [--root] [--bones] [--points a,b] [--locked a,b] [--json]`

What is under a canvas point, topmost first (hit-testing as the studio does it).

| Option | Meaning |
|---|---|
| `--frame N`, `--x X`, `--y Y` | Required: frame and canvas coordinates. |
| `--tol PX` | Pick tolerance in canvas pixels (default 3). |
| `--root` | Report the top-level ancestor (`mouth`) instead of the innermost layer (`mouth/closed`). |
| `--bones` | Also consider bones. |
| `--points a,b` | Layers whose control points (anchors/handles/mesh vertices) are pickable. |
| `--locked a,b` | Layers to skip (treated as locked). |
| `--json` | `{"hit":{...},"hits":[...]}`. |

```
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4
hit: layer ink (distance 0.00)
  under: layer mouth/closed (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4 --root --locked mouth
hit: layer ink (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 300 --y 240 --json
{"hit":{"distance":0.0,"target":{"kind":"layer","path":"sky"}},"hits":[{"distance":0.0,"target":{"kind":"layer","path":"sky"}}]}
```
**Gotcha:** the background layer (`sky`) is a normal hit target, so there is almost always at least one hit.

### `select-box`

**Synopsis:** `osatura select-box <project.json> --frame N (--rect x0,y0,x1,y1 | --lasso x,y;x,y;x,y) [--mode bbox|contain|intersect] [--root] [--bones] [--json]`

Marquee/lasso selection. `--mode` decides when a layer counts as inside: `bbox` (its bounding box touches the region), `contain` (fully inside), `intersect` (default: its drawn geometry intersects the region). `--root` reports top-level ancestors, `--bones` includes bones.
Quote a `--lasso` value in a shell (the `;` separates points).

```
$ osatura select-box demo.json --frame 0 --rect 200,95,220,115 --root
layer sky
layer arm
layer mouth
layer ink
$ osatura select-box demo.json --frame 0 --rect 0,0,640,480 --mode contain
layer sun
layer shadow
...
$ osatura select-box demo.json --frame 0 --lasso "100,100;300,100;300,300" --bones --json
{"targets":[{"kind":"bone","name":"arm_low"},{"kind":"bone","name":"hand"},{"kind":"layer","path":"sky"}, ...]}
```

### `timeline`

**Synopsis:** `osatura timeline <project.json> [--json]`

The dope-sheet view as text: tracks per channel grouped by bone/layer, with key frames (`keys @ ...`), `expr` / `driven` channels, locks and colours. The numbers `#N` are the channel indexes used by `keyedit --channels` and `#N@F0..F1` selections.

```
$ osatura timeline demo.json
timeline: 48 frames at 24 fps, 14 channels
Bones
  arm_up
    #0 bone_angle (keys @ 0 12 24 36 48)
    arm_low
      #1 bone_angle (keys @ 0 12 24 36 48)
      hand
        #2 bone_angle (keys @ 0 12 24 36 48)
        #3 bone_angle (expr)
  foot_ctrl
    #4 bone_offset_x (keys @ 0 12 24 36 48)
...
```
`--json` gives groups with ids, parents, children, tracks, `frames`, `locked`.

### `timecode`

**Synopsis:** `osatura timecode <frame|HH:MM:SS;FF> [--fps 24] [--drop] [--nodrop]`

SMPTE timecode conversion. A bare number is a frame and prints a timecode; an argument containing `:` or `;` is parsed and prints the frame number.
29.97, 59.94 and 119.88 fps use drop-frame timecode (`;` before the frames) by default; `--nodrop` counts 30 (or 60/120) instead; `--drop` forces drop-frame (error for other rates).
`--drop` and `--nodrop` exclude each other.

```
$ osatura timecode 100
00:00:04:04
$ osatura timecode 100 --fps 29.97
00:00:03;10
$ osatura timecode 1800 --fps 29.97 --drop
00:01:00;02
$ osatura timecode 1800 --fps 29.97 --nodrop
00:01:00:00
$ osatura timecode 00:00:04:04
100
$ osatura timecode 00:00:04;04 --fps 24
drop-frame timecode is only defined for 29.97, 59.94 and 119.88 fps         [exit 1]
```
Does not read a project; `--fps` defaults to 24 (not to a project's fps).

### `snapshot`

**Synopsis:**
```
osatura snapshot save <project.json> <dir> [--keep 20]
osatura snapshot list <project.json> <dir>
osatura snapshot restore <project.json> <dir> <out.json> [--number N]
osatura snapshot recover <project.json> <dir> <out.json>
```
`--keep` belongs to `save` only and `--number` to `restore` only; any other combination is refused (`unknown option --keep for 'snapshot list' (applies to: snapshot save)`), as is a missing or extra file argument.

A numbered history of a project in a directory (independent of `vcs`). `save` stores the current project as `rev-NNNNNN-<hash>.json` and keeps only the newest `--keep` revisions (default 20, older files are deleted); saving a project identical to the latest revision prints `unchanged since the last revision` and stores nothing. `list` prints revisions. `restore` writes revision `--number N` to `<out.json>`; `recover` writes the newest revision.

```
$ osatura snapshot save s.json snaps --keep 3
saved revision 1 (4922f48a86f24554)
$ osatura snapshot list s.json snaps
     1  4922f48a86f24554  snaps/rev-000001-4922f48a86f24554.json
$ osatura snapshot restore s.json snaps restored.json --number 1
wrote restored.json
$ osatura snapshot recover s.json snaps recovered.json
wrote recovered.json
```
`list` on a missing directory prints nothing and exits 0. **Related:** `vcs` for branches and merges.

---

## 4. Rendering and export

### `render`

**Synopsis:** `osatura render <project.json> <frame> <out.svg|out.png> [--bones] [--scale N] [--clamp] [--proxy N]`

`--proxy N` renders a scrub proxy (N = 1..4): a `.png` is a bitmap at 1/2^N resolution, an `.svg` is simplified vectors; both drop fine strokes, gradients and blends.

Renders one frame. A `<frame>` at or beyond the project length is an error (`frame 99 is outside the animation ... valid frames are 0..N-1 ... pass --clamp`); `--clamp` renders the last frame instead and prints a warning on stderr. (Only `render` has this check; `dof`, `render-hdr` and `fx` do not.) The extension chooses the format (`.svg` vector, `.png` 8-bit RGBA raster); any other extension is refused **before** anything is written.

| Option | Default | Meaning |
|---|---|---|
| `--bones` | off | Draw the skeleton overlay (heads as dots, bones as lines). |
| `--scale N` | 1 | Output scale, a number in (0, 16]. For `.png` it multiplies the pixel size; for `.svg` the document keeps its viewBox. |
| `--proxy N` | off | Scrub proxy, N = 1..4: `.png` = bitmap at 1/2^N resolution, `.svg` = simplified vectors; both drop fine strokes, gradients and blend modes. Meant for interactive scrubbing, not for deliverables. |

```
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura render demo.json 0 /tmp/x.xyz
render writes .png or .svg; '/tmp/x.xyz' has another extension (nothing was written)       [exit 1]
$ osatura render demo.json 0 o.svg --scale 0
--scale needs a number in (0, 16]                                                         [exit 1]
$ osatura render demo.json abc o.svg
bad frame number 'abc'                                                                    [exit 1]
$ osatura render demo.json 2 px.png --proxy 2        # 640x480 project -> 160x120 PNG
wrote px.png
```
**Errors:** missing/invalid project, non-finite pose at that frame, output equals input project. **Related:** `frames`, `render-hdr`, `onion`, `compare`, `fx`.
**Limits:** the PNG rasteriser is the engine's own (8-bit, sRGB; blending is not colour managed unless the project opts in with `color-space ... linear`). Layer effects, masks and blend modes are drawn by both.

### `frames`

**Synopsis:** `osatura frames <project.json> <out_dir> [--png] [--bones] [--scale N]`

Renders **every** frame into a directory (created if missing) as `frame_0000.svg` ... (or `.png` with `--png`). All frames are checked for finite poses first.

```
$ osatura frames demo.json fr --png
wrote 48 frames to fr
$ ls fr | head -2
frame_0000.png
frame_0001.png
```
**Gotcha:** existing files with the same names are overwritten; unrelated files in the directory are left alone. Use this as input for `ffmpeg` or `import-seq`.

### `export`

**Synopsis:**
```
osatura export <project.json> <out.gif> [--scale N] [--report r.json]
osatura export <project.json> <out.html|out.htm>
osatura export <project.json> <out.json|out.lottie> [--report r.json]
osatura export <project.json> <out.mp4> [--scale N]
osatura export <project.json> <out.apng> [--scale N]
osatura export <project.json> <out.webm> [--scale N] [--codec vp9|av1]
osatura export <project.json> <out.png> [--scale N] [--columns N]
osatura export <project.json> <out.glb> [--report r.json]
```
(one form per format; the output extension, case-insensitive, picks the form. An option that the form does not list is refused: `unknown option --scale for 'export .json' (applies to: export .gif, export .mp4, ...)`. With an unknown or missing extension the format error is reported instead.)

The output extension selects the format:

| Extension | Result | Needs ffmpeg | Options that apply |
|---|---|---|---|
| `.gif` | looping GIF at the project frame rate (see [EXPORT_FORMATS.md](EXPORT_FORMATS.md)) | no | `--scale`, `--report` |
| `.html`, `.htm` | one self-contained HTML player (inline SVG frames, play/pause, scrubber, optional embedded audio) | audio only | - |
| `.json` | Lottie (Bodymovin) | no | `--report` |
| `.lottie` | dotLottie container (zip: `manifest.json` + `animations/animation.json`) | no | `--report` |
| `.mp4` | H.264 yuv420p (+ AAC when the project has sounds) | yes (`libx264`, `aac`) | `--scale` |
| `.apng` | animated PNG | no | `--scale` |
| `.webm` | VP9 (default) or AV1 with alpha (+ Opus sound) | yes (`libvpx-vp9` / `libaom-av1`, `libopus`) | `--scale`, `--codec vp9\|av1` |
| `.png` | sprite sheet **plus** `<out>.atlas.json` metadata (never `<out>.json`) | no | `--scale`, `--columns N` |
| `.glb` | glTF 2.0 binary (bones, animation, skinned fills/strokes, gradients, masks, images, morph targets) | no | `--report` |

Options:

| Option | Meaning |
|---|---|
| `--scale N` | Raster scale (0, 16] for `.gif .mp4 .apng .webm .png`. On other formats it is refused (`unknown option --scale for 'export .json'`); before wave 7 it was a warning. |
| `--report FILE` | Write the list of things the format could not carry as JSON (`.gif .json .lottie .glb`): `{"format","count","warnings":[{"code","layer","message"}]}`. The same warnings are always printed on stderr. The file may not be the input project. |
| `--codec vp9\|av1` | `.webm` only (default `vp9`); any other value: `unknown --codec 'foo' (vp9 or av1)`. |
| `--columns N` | `.png` sprite sheet only: tiles per row (default: a near-square grid, ceil(sqrt(frames)): 3 columns for 8 frames). |

Examples (all verified on an 8-frame copy of the demo, `--scale 0.5`):

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ cat rep.json
{ "count": 2, "format": "lottie", "warnings": [ {"code": "variable-width-stroke", "layer": "ink", ...}, {"code": "gradient-anisotropic", ...} ] }
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.json
wrote out3.png                          # 4 columns x 2 rows of 160x120 tiles; out3.json lists {index,x,y,w,h} per frame
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
$ osatura export small.json out.mp4 --scale 0.5       # 320x240 h264 yuv420p, 8 frames
wrote out.mp4
$ osatura export small.json out.webm --codec av1      # needs a libaom-av1 build of ffmpeg
```

Errors and refusals:

```
$ osatura export small.json z.xyz
unknown export format '.xyz' (use .gif .html .htm .json .lottie .mp4 .apng .webm .png .glb)
$ osatura export small.json small.json
refusing to overwrite the input project small.json with output of the same name
$ osatura export ora.json ora.glb                       # a project of image layers and no bones
the project has no bones to export
$ PATH=/nonexistent osatura export small.json y.mp4
cannot run ffmpeg (...); install it or export a GIF/HTML instead                          [exit 1, nothing written]
```
Other refusals: `ffmpeg failed` (missing encoder), `export would write about N million values ...` (the Lottie/HTML/GLB size guard: the limit is 40 million values = frames x (anchors + bones + layers); shorten the animation or simplify the paths), non-finite poses on any frame, and `image too large for GIF`.

**Outputs / side effects:** `.png` also writes the sprite-sheet metadata to `<out-name>.atlas.json` (e.g. `out.png` -> `out.atlas.json`), so it can no longer replace a Lottie file `out.json`; the sidecar is subject to the same input/symlink guards. `.mp4`/`.webm` render PNG frames into a private temporary directory that is removed afterwards.
**Sound:** `.mp4` and `.webm` mix the project's sound tracks (start, gain, trim, fades; cut at the project length); `.html` embeds a mixed mp3 (warn above 3 MiB, dropped above 16 MiB with a warning); `.gif` and `.apng` print `note: GIF cannot carry audio; ...` and stay silent; Lottie warns `[audio]`.
**Transparency:** a project with `"background": null` renders with transparent pixels wherever nothing is drawn. `.apng`, the `.png` sprite sheet and `.webm` (`yuva420p`, container tag `alpha_mode=1`) keep the alpha channel; `.gif` keeps 1-bit transparency (verified: the transparent-colour flag is set); `.mp4` has no alpha. A project with a background colour (the demo has `#f6f1e7`) is opaque.
**Related:** `export-fit` (fit a byte budget), `size-report`, `lod`, `dof-export`, `export-oca`, `render-hdr`. Full format detail: [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

### `export-oca`

**Synopsis:** `osatura export-oca <project.json> <out_dir> [--layers]`

Writes an **Open Cel Animation** folder: `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) plus PNG cels `layerN/fNNNN.png`. Without `--layers` there is one layer `render` holding the composited frames; with `--layers` every **top-level** layer is rendered alone on transparency into its own OCA layer (11 layers for the demo). `--scale N` multiplies the cel size.

```
$ osatura export-oca small.json oca2 --layers
wrote oca2/small.oca
$ ls oca2
layer0 layer1 ... layer10 small.oca
```
**Limits:** cels are rasters (no vectors, bones or channels survive); layer opacity/blend are not written (every layer is `paintlayer`, opacity 1); sounds are not exported; the folder is created if needed and files inside are overwritten.

### `render-hdr`

**Synopsis:** `osatura render-hdr <project.json> <frame> <out.exr|out.png> [linear] [--precision half|float] [--float32] [--8bit] [--scale N] [--clamp]`

Float (32-bit) render (a `<frame>` outside the project is an error, `--clamp` takes the nearest valid frame, as for `render`): `.exr` writes a single-part, scanline, uncompressed **OpenEXR** (HALF channels by default, FLOAT with `--precision float` or `--float32`), linear-light and premultiplied; `.png` writes a **16-bit RGBA PNG**, sRGB-encoded unless the literal word `linear` is given as the fifth argument. The project background is **not** drawn (the image is transparent where nothing is drawn). `--8bit` uses the legacy 8-bit rasteriser as the source (cannot be combined with float32 output). `--scale N` applies.

```
$ osatura render-hdr small.json 3 f.exr
wrote f.exr (640x480)                       # 465,611 bytes: HALF
$ osatura render-hdr small.json 3 f2.exr linear --precision float
wrote f2.exr (640x480)                      # 923,211 bytes: FLOAT
$ osatura render-hdr small.json 3 f4.png
wrote f4.png (640x480)                      # PNG 16-bit/colour RGBA
$ osatura render-hdr small.json 3 f6.exr --precision bogus
unknown precision 'bogus' (use half or float)
$ osatura render-hdr small.json 3 f7.jpg
output must end in .exr or .png
```
`ffprobe` reads the EXR as `exr 640x480 gbrapf32le`. For `.exr`, the word `linear` has no effect (EXR is always linear). **Limits:** no compression, one frame per file, no layers/AOVs; the source art is ordinary sRGB colour, so the file holds display-range values at higher precision, not scene-referred HDR light. Not colour managed beyond the sRGB<->linear conversion.

### `render-gpu`

**Synopsis:** `osatura render-gpu <project.json> <out.png> [--frame N] [--adapter software|auto] [--scale S] [--bench N]`

GPU (wgpu) render of one frame. Only available in builds made with `--features gpu`; the default build answers:

```
$ osatura render-gpu small.json g.png
render-gpu needs a build with the gpu feature: cargo build -p osatura-cli --features gpu       [exit 1]
```
In a GPU build, unsupported scenes (images, bones) are reported as errors, never drawn wrongly. `--bench N` is a benchmark option. This command could not be run in the test build (no GPU feature), so its flags are taken from the usage text only.

### `compare`

**Synopsis:** `osatura compare <a.json> <b.json> <frame> <out.png> [--gain 4 --scale N]`

Writes one PNG: **before | after | difference** side by side (the third panel is a heat map, black = identical, red -> yellow = larger change, multiplied by `--gain`, default 4). Both projects must render at the same size (`frame sizes differ (...); compare projects of the same size`).

```
$ osatura compare small.json d2.json 3 cmp.png --gain 4 --scale 0.25
wrote cmp.png
```
**Related:** `diff` (structure), `ci diff` (two PNGs, perceptual metrics), `golden`.

### `proxy-bench`

**Synopsis:** `osatura proxy-bench <project.json> [--frames N]`

Milliseconds per frame, single thread, at full quality and at each scrub proxy level (see `render --proxy`).

```
$ osatura proxy-bench small.json --frames 2
2 frames of small (640x480), ms per frame, single thread:
  evaluate only           0.08
  svg (no raster)         0.48   vector proxy: level 0 0.32, level 2 0.24, level 4 0.24
  raster level 0       1360.87   (full quality)
  raster level 1        319.14   (4.3x faster than level 0)
  ...
  raster level 4          6.84   (199.0x faster than level 0)
```
(debug-build numbers). 

---

## 5. Rigging, skins, IK and layout helpers

All commands in this section follow the pattern `osatura <cmd> <project.json> <out.json> ...`: they read a project, change it, **re-validate it**
and write `<out.json>` (which may equal the input). On error nothing is written.

### `autorig`

**Synopsis:** `osatura autorig <project.json> <out.json> [seed]`

Adds standard automation guessed from bone/layer **names**: IK chains (mix 0 so FK keeps working), a breathing loop (needs a torso bone) and blinking (needs eye layers). It prints one line per decision: `+` added, `-` skipped with the reason. Also runs `eye-clip`.

```
$ osatura autorig small.json auto.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
- arm_r: upper/lower bone not found
- leg_l: already has an IK chain
- breathing: no torso bone
- blink: no eye layers
wrote auto.json
```
`seed` (optional positional) seeds the random blink timing. **Limits:** purely name based (`tag` shows how names are read); it never creates bones.

### `slider`

**Synopsis:** `osatura slider <project.json> <out.json> <name> <poses>` where `poses` is `at:target.prop=val,target.prop=val;at:...`

Creates a **controller bone** `ctrl_<name>` whose `offset_x` drives several properties between stored poses (a pose slider). Animate the printed controller's `bone_offset_x`.

```
$ osatura slider small.json sl.json pose "0:arm_up.bone_angle=0;24:arm_up.bone_angle=30"
controller bone: ctrl_pose (animate its offset_x)
wrote sl.json
```
`target.prop` uses the snake_case property names (`arm_up.angle` fails with `unknown property 'angle'`). The last dot splits target from property, so layer paths such as `grp/child.layer_opacity` work.

### `view-swap`

**Synopsis:** `osatura view-swap <project.json> <out.json> <switch_layer> <bone.prop> <from:child,from:child,...>`

Drives a switch layer from a bone property with a step function: the pairs `from:child` are sorted by `from`, and the switch shows the child of the last pair whose `from` is not above the driver value (`-40:0,0:1,40:2` = child 0 below 0, child 1 from 0, child 2 from 40). Use it for turn-around views (front/side/back) driven by a yaw bone. Child indexes must exist (`child index 5 is out of range (the layer has 3)`); the layer must be a switch.

```
$ osatura view-swap small.json vs.json mouth foot_ctrl.bone_offset_x "-40:0,0:1,40:2"
wrote vs.json
```

### `align` and `distribute`

**Synopsis:**
`osatura align <project.json> <out.json> <mode> <layer,layer,...|layer> [stage]`  (mode: `left right top bottom center-x center-y`)
`osatura distribute <project.json> <out.json> <x|y> <gaps|centers> <layer,layer,...>`

`align` moves layers so the bounding boxes of their flattened curves line up (to each other, or, for a single layer followed by the word `stage`, to the canvas). `distribute` spaces three or more layers evenly along an axis, equalising either the **gaps** between boxes or the **centers**.

```
$ osatura align small.json al.json center-x sun,hills stage
wrote al.json
$ osatura align small.json al2.json top ground
select at least two layers (or one with --stage)                 [exit 1]
$ osatura distribute small.json di.json x gaps sun,ground,hills
wrote di.json
```
These edit layer positions (static transform), so keyed `layer_x`/`layer_y` channels still override them. (The error message mentions `--stage`; the actual argument is the bare word `stage`.)

### `trim`

**Synopsis:** `osatura trim <project.json> <out.json> <layer> <path_index> <start> <end> [offset]`

Bakes a *trim path*: replaces path number `path_index` of a vector layer with only the part between `start` and `end` (fractions 0..1 of its length, both shifted by `offset`; closed paths wrap). The result is an **open polyline with the same stroke and no fill** - destructive, so keep the original. For a draw-on effect, bake one trimmed copy per frame.

```
$ osatura trim small.json tr.json ink 0 0 0.5 0.25
wrote tr.json
```

### `reparent`

**Synopsis:** `osatura reparent <project.json> <out.json> <bone> <new_parent|none>`

Changes a bone's parent **without moving it**: the rest offset/angle are recomputed so the rest pose is unchanged (verified: only `parent`, `rest_angle`, `rest_offset` differ afterwards). `none` makes it a root. A parent that would create a cycle is refused.

```
$ osatura reparent small.json rp.json hand none
wrote rp.json
$ osatura diff small.json rp.json
~ bone 'hand' changed (parent, rest_angle, rest_offset)
```

### `parent-switch`

**Synopsis:** `osatura parent-switch <project.json> <out.json> <bone> <frame:parent|none,...>`

Bakes animated re-parenting (a hand grabbing a prop): at each switch frame the bone's world pose relative to the new parent is captured and kept (rigid follow), or frozen in world space for `none`. The bone becomes a root and its world pose is written as plain offset/angle keys for every frame; its old channels stay in the file but are overridden by the baked ones. **Limits:** refused for bones involved in IK, constraints or spring bones; scale is not carried.

```
$ osatura parent-switch demo.json ps.json hand 0:none,24:arm_up
wrote ps.json
```

### `skin`

**Synopsis (subcommands):**
```
osatura skin <project.json> <out.json> add <group> <name> <layer,layer,...>
osatura skin <project.json> <out.json> rig <group> <name> <constraint,...|-> <ik,...|->
osatura skin <project.json> <out.json> bones <group> <name> <bone,...|->
osatura skin <project.json> <out.json> apply <group> <name>
osatura skin <project.json> <out.json> schedule <group> <frame:name,frame:name,...>
```

A *skin set* is identified by a `<group>` label (an "outfit" or "expression" set; in the example it happens to equal the name of a layer group). Each named skin shows a set of layers, given by **layer path** (e.g. `skins/box`); applying a skin shows its layers and hides the layers of every other skin of the same group.
`add` defines a skin; `rig` lists constraints and IK chains that are active only while that skin is; `bones` lists bones that exist only while it is active (collapsed to scale 0 otherwise); `apply` makes one skin active statically; `schedule` keys the active skin over time as step keys on **layer opacity** (so do not animate the opacity of those layers otherwise). `-` means "none".

```
$ osatura skin sc.json sk1.json add skins red skins/box
wrote sk1.json
$ osatura skin sk1.json sk2.json add skins green skins/box2
$ osatura skin sk2.json sk3.json apply skins green
$ osatura skin sk3.json sk4.json schedule skins 0:red,4:green
$ osatura skin sk4.json sk5.json bones skins red -
$ osatura skin sk4.json sk5.json rig skins red - leg
$ osatura skin sc.json x.json add skins red box
no layer 'box'                                                  [exit 1]     (use the full path skins/box)
```
**Limits (ROADMAP §9):** a bone that exists only in one skin is never removed from the evaluated rig (indices stay; it is scaled to 0); a layer weighted to a skin-only bone but not in that skin vanishes with it, and this is not checked.

### `bone-vis`

**Synopsis:** `osatura bone-vis <project.json> <out.json> <bone> <from> <to>|clear`

Shows the bone in studio **overlays** only for a frame range (clutter control). Does not change rendering/export. `clear` removes the range (`range removed`).

### `chan-meta`

**Synopsis:** `osatura chan-meta <project.json> <out.json> <target> <prop|*> [color=#rrggbb] [shy=on|off] [solo=on|off] [lock=on|off]`

Dope-sheet metadata of a channel: display colour, *shy* (hidden in lists), *solo* (preview only these channels in the studio) and **lock** (refuse edits: locked channels are skipped by `keys`, refused by `keyedit`, `audio-react` and `live-map`, and never touched by `clip bake`). Evaluation and export ignore all four. `*` selects every property of the target.

```
$ osatura chan-meta small.json cm.json arm_up bone_angle 'color=#ff0000' shy=on lock=on
wrote cm.json
```

### `reference`

**Synopsis:** `osatura reference <project.json> <out.json> <layer> <on|off>`

Marks a layer as a **tracing reference**: shown in the studio, never exported.

### `parallax`

**Synopsis:** `osatura parallax <project.json> <out.json> <far_layer,...,near_layer> <far_value> <near_value>`

Assigns multiplane depth (`parallax` values spread between `far_value` and `near_value`) to the listed top-level layers, in the order given (first = farthest). With camera movement the layers shift by different amounts; `dof` blurs by `|parallax - focus|`.

```
$ osatura parallax small.json pa.json sky,hills,ground 0.2 1.0
wrote pa.json
```

### `eye-clip`

**Synopsis:** `osatura eye-clip <project.json> <out.json>`

Masks each iris/pupil layer by the eye-shape layer on the same side (found by name, see `tag`) using an ordinary layer mask, so pupils never leave the eye while they move. Also run by `autorig`. When no matching names are found the file is still written, unchanged.

### `comp`

**Synopsis:**
```
osatura comp capture|apply <project.json> <out.json> <name>
osatura comp list <project.json>
```

Named layer states ("comps": visibility, opacity and the selected child of switch layers, per layer path; **not** transforms). `capture` stores the current state as `name`, `apply` restores it, `list` prints names (`list` takes no output file and writes nothing; a legacy fourth word, `comp list p.json p.json`, is still accepted and ignored).

```
$ osatura comp capture small.json c1.json pose1
wrote c1.json
$ osatura comp list c1.json c1.json
pose1 (14 layers)
$ osatura comp apply c1.json c2.json pose1
wrote c2.json
```

### `retarget`

**Synopsis:** `osatura retarget <source.json> <dest.json> <out.json> --pairs srcChain=dstChain[,..]`

Retargets by *intent*: the animation of the IK **effector** (target bone) of each source chain is scaled by the ratio of the two limbs' reach (sum of bone lengths) and re-pointed at the destination chain's target bone. Non-keyed effector channels cannot be scaled; they are passed through renamed and reported as `note:` lines. `--pairs srcChain=dstChain[,...]` is required. In the example the source and destination are the same project, which is only useful as a smoke test.

```
$ osatura retarget demo.json demo.json rt.json --pairs leg=leg
wrote rt.json (+14 channels)
```
**Related:** `mocap-retarget` for BVH data; `walk`, `mirror`.

---

## 6. Animation, keys, timeline and generators

### `keys`

**Synopsis:**
```
osatura keys shift <project.json> <out.json> --by N [--from F --to F --target NAME --prop PROP]
osatura keys scale <project.json> <out.json> --factor X [--pivot F --from F --to F --target NAME --prop PROP]
osatura keys ease <project.json> <out.json> --ease linear|step|ease_in|ease_out|ease_in_out|smooth [--from F --to F --target NAME --prop PROP]
osatura keys reverse <project.json> <out.json> [--from F --to F --target NAME --prop PROP]
```
Each operation accepts only its own options (`keys shift --factor 2` is refused); `--by`, `--factor` and `--ease` are required for their operation (an omitted `--by` or `--factor` used to be a silent no-op).

Bulk key operations over **all keyed channels** (filtered by `--target` / `--prop`; a `--from/--to` range limits which keys are touched). Locked channels are skipped with `skipped locked channel ...`.

| Operation | Needs | Effect |
|---|---|---|
| `shift` | `--by N` | Move keys by N frames. |
| `scale` | `--pivot F --factor X` | Scale key times about the pivot frame. |
| `ease` | `--ease E` | Set the easing of the keys (named easings; use `keyedit ease` for `bezier:`). |
| `reverse` | - | Reverse the key order inside the range. |

```
$ osatura keys shift small.json k1.json --target arm_up --prop bone_angle --by 2
wrote k1.json (5 key(s) changed)
$ osatura keys scale small.json k2.json --pivot 0 --factor 0.5 --from 0 --to 48
wrote k2.json (37 key(s) changed)
$ osatura keys reverse small.json k4.json
wrote k4.json (37 key(s) changed)
```
Keys moved outside `0..frames-1` stay in the file (lint reports `L007`); this command does not extend the project length. **Related:** `keyedit` (precise, selection based), `loop-solve`.

### `keyedit`

**Synopsis:** `osatura keyedit <project.json> <out.json|-> <sub-command> [options]`

Dope-sheet edits as **one undo-safe batch**. With `-` as output nothing is written and the planned operation is printed as JSON (dry run). Sub-commands and options:

```
osatura keyedit <project.json> <out.json|-> move --sel SEL [--dt N] [--dv N] [--collision replace|merge|refuse] [--no-snap] [--free]
osatura keyedit <project.json> <out.json|-> scale --sel SEL --pivot F --factor X | dup --sel SEL --offset N | delete --sel SEL | ease --sel SEL --ease E | value --sel SEL --value V | smooth --sel SEL [--kind monotone|catmull_rom]
osatura keyedit <project.json> <out.json|-> copy --sel SEL --clip clip.json | paste --clip clip.json --at F [--remap from_target:prop=to_target:prop]
osatura keyedit <project.json> <out.json|-> add --target T --prop P --at F [--value V] [--ease E] | ripple --at F --by N --mode insert|delete [--scope 0,1] [--adjust-frames]
osatura keyedit <project.json> <out.json|-> offset --channels 0,1 --dt N | extrapolate --channels 0,1 --extrapolate loop | loop --channels 0,1 [--end F] | handle --sel SEL --side in|out --t F --v V [--mode free|aligned|auto] | lock --target T [--prop P] [--value on|off]
```

* **SEL** is `TARGET:PROP[@F0..F1]` items joined by `;`, or `#CHANNEL[@F0..F1]` (channel index from `timeline`), or `all`. Quote it (`;`, `@`, `#`).
* **E** (easing) is `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`, `smooth`, `bounce`, `bezier:x1,y1,x2,y2` or a JSON object.
* `--collision` decides what happens when a moved key lands on an existing key (`replace` default, `merge`, `refuse`); `--no-snap` keeps fractional frames; `--free` lifts the whole-frame restriction.
* Output line: `<op>: N keys changed, wrote <out>`. `copy` with a real output prints `copy: 0 keys changed, project unchanged, clip saved`.
* Operations that would put a key outside `0..=frames` are refused with a precise message (`duplicate: frame 52 is outside the project range 0..=48`) unless `ripple --adjust-frames` extends the project.

```
$ osatura keyedit demo.json - move --sel "arm_up:bone_angle@12..36" --dt 2        # dry run: prints the batch JSON
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", ... } ] } }
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 2 --collision merge
move: 3 keys changed, wrote k1.json
$ osatura keyedit demo.json k5.json ease --sel "arm_up:bone_angle" --ease bezier:0.4,0,0.2,1
ease: 5 keys changed, wrote k5.json
$ osatura keyedit demo.json k7.json smooth --sel "arm_up:bone_angle" --kind monotone
smooth: 4 keys changed, wrote k7.json
$ osatura keyedit demo.json k8.json copy --sel "arm_up:bone_angle@0..12" --clip c2.json
copy: 0 keys changed, project unchanged, clip saved
$ osatura keyedit demo.json k8.json paste --clip c2.json --at 30 --remap arm_up:bone_angle=arm_low:bone_angle
paste: 2 keys changed, wrote k8.json
$ osatura keyedit demo.json k9.json add --target arm_up --prop bone_angle --at 6 --value 5 --ease linear
add: 1 keys changed, wrote k9.json
$ osatura keyedit demo.json k10.json ripple --at 12 --by 6 --mode insert --adjust-frames
ripple: 28 keys changed, wrote k10.json            # project now 54 frames
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 12 --collision refuse
a key already exists at frame 48                    [exit 1]
$ osatura keyedit demo.json k1.json move --sel "zzz:bone_angle" --dt 1
nothing matches 'zzz:bone_angle'                    [exit 1]
$ osatura keyedit demo.json k15.json lock --target arm_up --prop bone_angle --value on
lock: 1 keys changed, wrote k15.json
```
**Gotchas:** `delete` with `#0@12..12` removes exactly one key; `ripple --mode delete --scope 0,1` limits the shift to channels 0 and 1; `copy` and `paste` exchange a clip file, not the system clipboard. Locked channels refuse `move/scale/...` (use `lock --value off` first).

### `marker`

**Synopsis:** `osatura marker <project.json> <out.json> <name> <frame> [--end F]`

Adds a timeline marker (a named point or, with `--end`, a range). A marker with the same name is replaced. Markers are sorted by frame.

```
$ osatura marker small.json m1.json hit 4 --end 6
wrote m1.json (marker 'hit' at 4 = 00:00:00:04)
```

### `event`

**Synopsis:** `osatura event <project.json> <out.json> <name> <frame> [--data JSON]`

Adds a named event (a runtime trigger for players) at a frame, with optional JSON payload. Invalid JSON: `--data is not valid JSON: ...`.

```
$ osatura event small.json ev1.json boom 3 --data '{"power":2}'
wrote ev1.json (1 events)
```

### `idle`

**Synopsis:**
```
osatura idle blink <project.json> <out.json> --target T --prop P [--interval S --jitter X --closed F --seed N]
osatura idle breath <project.json> <out.json> --target T --prop P [--amp A --period F]
osatura idle sway <project.json> <out.json> --target T --prop P [--amp A --freq HZ --seed N]
```
Each generator accepts only its own options (`idle breath --seed 2` is refused).

Generates a procedural idle channel and **appends** it (existing channels stay).

| Generator | Options (defaults) | Typical target |
|---|---|---|
| `blink` | `--interval` mean seconds between blinks, `--jitter`, `--closed` frames closed, `--seed` | a `layer_switch` (eye open/closed) or `layer_scale_y` |
| `breath` | `--amp` (0.03), `--period` frames (3 s at the project fps) | `bone_scale` of a torso |
| `sway` | `--amp` (2), `--freq` Hz (0.3), `--seed` (1) | `bone_angle` |

```
$ osatura idle blink small.json i1.json --target mouth --prop layer_switch --interval 1 --seed 3
wrote i1.json
$ osatura idle sway small.json i3.json --target arm_up --prop bone_angle --amp 3 --freq 0.5
wrote i3.json
```
`--target` and `--prop` are required (`--target is required`).

### `follow` (two forms)

**Follow-through (options):** `osatura follow <project.json> <out.json> --target SRC --prop P --chain a,b,c [--delay F --decay X]`
Adds delayed, decaying copies of an existing channel (`--target`/`--prop` name the source) to the bones of the chain: overlapping action / follow-through (`--delay` frames per link, default 3; `--decay` per link, default 0.7). A delayed copy is cut at the project's last frame (no key is written after frame `frames`, so `lint` L007 does not fire); the frames between the last kept key and the cut are sampled so the delayed curve is exact on every whole frame.

```
$ osatura follow small.json f1.json --target arm_up --prop bone_angle --chain arm_low,hand --delay 2 --decay 0.6
wrote f1.json (+2 channels)
```

**Path following (positional):** `osatura follow <project.json> <out.json> <layer|bone:NAME> <layer_with_path> <start_frame> <end_frame> [no-orient]`
Bakes a layer (or the root bone `bone:NAME`) travelling along the first path of another layer between two frames, rotating to the path tangent unless `no-orient` is given.

```
$ osatura follow small.json fp.json target ink 0 7
wrote fp.json
$ osatura follow small.json fp2.json bone:hand ink 0 7 no-orient
path following needs a root bone (no parent)                      [exit 1]
```

### `take-record`, `take-apply`, `take-list`

**Synopsis:**
```
osatura take-record <project.json> <library.json> <name> <from> <to> [target_prefix,...]
osatura take-apply  <project.json> <out.json> <library.json> <name> <at> [from=to,...] [speed] [--blend FRAMES]
osatura take-list   <library.json>
```
A **take** is a reusable slice of animation stored in a library file. `take-record` captures frames `from..to` of the channels whose target starts with one of the prefixes (all channels when omitted); the library is created if missing and a take of the same name is replaced. `take-apply` pastes the take at frame `at`, optionally **retargeting** (`from=to` pairs rename targets) and changing `speed`; the arguments are checked strictly: an entry without `=`, a source name the take has no track for, or a speed that is not a positive number is an error, and an unreadable library names the file (`cannot read the take library ...`); `--blend N` cross-fades with existing animation for N frames at both ends.

```
$ osatura take-record demo.json lib.json wave 0 24 arm_
recorded 2 tracks into lib.json
$ osatura take-list lib.json
wave (24 frames, 2 tracks)
$ osatura take-apply demo.json ta.json lib.json wave 10 arm_up=arm_low 1.5 --blend 4
applied 2 tracks; wrote ta.json
```

### `clip`

**Synopsis:**
```
osatura clip add <project.json> <out.json> <library.json> <take> <clip> [--at F --speed S --loops N --mode replace|additive|multiply --weight W
                   --include a_*,b --exclude c --track-weight arm_*=1,leg_*=0.2 --rename a=b --fade-in F --fade-out F]
osatura clip list <project.json>
osatura clip remove <project.json> <out.json> <clip>
osatura clip set-weight <project.json> <out.json> <clip> [W] [--keys frame:w,frame:w] [--track PATTERN[:prop]]
osatura clip bake <project.json> <out.json>
```
A layered **clip stack** (non-linear animation): clips reference takes of a library, play at `--at` with `--speed`/`--loops`, and blend `replace`, `additive` or `multiply` with a weight (constant, keyed over time, or per track pattern). `clip add/set-weight/remove` only edit the stack; **`clip bake` regenerates the stack-owned channels** (hand-made and locked channels are never touched).

```
$ osatura clip add demo.json cl1.json lib.json wave w1 --at 6 --speed 1 --loops 2 --mode additive --weight 0.8 --fade-in 2 --include 'arm_*' --rename arm_up=hand
added clip 'w1' (1 clips); run clip bake to generate channels
$ osatura clip list cl1.json
1. w1 take 'wave' frames 6..30 x1 speed 1 Replace weight 1
0 baked channels
$ osatura clip set-weight cl1.json cl2.json w1 0.5
updated clip 'w1'; run clip bake to update channels
$ osatura clip set-weight cl1.json cl3.json w1 --keys 0:0,12:1
$ osatura clip bake cl2.json cl5.json
baked 1 clips into 4 channels (0 replaced); wrote cl5.json
$ osatura clip remove cl2.json cl6.json w1
removed clip 'w1'; run clip bake to update channels
```
(The `clip list` example shows the second `clip add` of the same name, which replaced the first.) **Limits (ROADMAP §9):** baked channels are sampled per integer frame; there is no `clip refresh` to re-pull an edited library take.

### `make-loop`

**Synopsis:** `osatura make-loop <project.json> <out.json> [targets|all] [props] [--inclusive]`

Makes hand-keyed `replace` channels end where they began: each curve is sampled per frame, the straight line from its first to its last value is subtracted (drift removal) and the first value is added back. Applied to a travelling bone's X offset this turns a walk into an in-place walk. The loop period is N = the project's `frames`, so playing `0..N-1` repeatedly shows no duplicated seam frame. `--inclusive` is the legacy convention where the last frame equals the first. Without arguments every keyed channel is processed. Prints one `removed drift` line per changed channel.

```
$ osatura make-loop demo.json ml2.json foot_ctrl bone_offset_x
foot_ctrl BoneOffsetX: removed drift 0.79
wrote ml2.json
$ osatura make-loop demo.json ml.json all bone_angle --inclusive
```
**Limits:** textbook linear detrending, not a re-timing of the motion; expression-driven channels are not touched. **Travel is removed on purpose** (a project that plays `0..N-1` repeatedly cannot travel for ever, so the loop is in place): a channel whose whole change was travel becomes constant, and the command prints a `note: ... is now constant` line for it; `lint` then reports that channel as L005 (information), which is expected. Name the targets and props to leave other channels' travel alone, or use `loop-solve` for motion that is already cyclic.

### `loop-solve`

**Synopsis:** `osatura loop-solve <project.json> <out.json> [--window 24] [--inclusive]`

Solves the **seam**: blends the end of keyed channels into their start over a window so the animation loops without a pop. Prints the seam error before and after.

```
$ osatura loop-solve demo.json ls.json --window 24
wrote ls.json: 8 channel(s) adjusted, seam 0.000 -> 0.000
```
`--window` is in frames (default 24). Compare with `lint`'s `loop` contract (`contracts`).

### `walk`

**Synopsis:** `osatura walk <project.json> <out.json> --thigh-l B --thigh-r B [--shin-l B --shin-r B --arm-l B --arm-r B --hips B --period N --stride DEG --knee DEG --bob PX --swing DEG]`

Generates a looping walk cycle as keyed channels on the bones you name (at least one bone option is required: `name at least one bone (--thigh-l, --thigh-r, ...)`). Defaults: `--period 24` frames, `--stride 28`, `--knee 40`, `--bob 4`, `--swing 22`. The channels use loop extrapolation and are **appended**.

```
$ osatura walk small.json wk.json --thigh-l arm_up --thigh-r arm_low --period 8 --stride 20
wrote wk.json (+2 channels)
```

### `mirror`

**Synopsis:** `osatura mirror <project.json> <out.json>`

Mirrors all keyed channels across the vertical axis: targets swap sides (`L_`/`R_` prefixes, `_L`/`_R` and `_l`/`_r` suffixes, the words `left`/`right`), and rotations and horizontal translations change sign. The mirrored channels **replace** the project's channels (they are not added next to the originals). The result line is `wrote X (N of M channel target(s) swapped sides)`; when no target has a left/right name, or the project has no channels, a warning on stderr says so (only signs are flipped).

### `foot-lock`, `stretch`, `magnet`

```
osatura foot-lock <project.json> <out.json> <ctrl_bone> <ground_y> [tolerance] [min_frames]
osatura stretch   <project.json> <out.json> <follower_bone> <leader_bone> [min=1] [max=3]
osatura magnet    <project.json> <out.json> <ctrl_bone> <bone:NAME|x,y> <radius> <strength 0..1>
```

* **`foot-lock`** samples the controller's world position every frame; runs of at least `min_frames` frames where it is on the ground (stage y points down: the foot is down when `y >= ground_y - tolerance`) are *contacts*, and an **additive** correction channel pins the controller to where each contact began. The original animation stays untouched; delete the added channels to undo.
  ```
  $ osatura foot-lock demo.json fl.json foot_ctrl 385 2 2
  contact frames 23..47 pinned at (479.2, 384.1)
  wrote fl.json
  ```
* **`stretch`** makes `follower_bone` aim at the head of `leader_bone` and scale (uniformly, between `min` and `max`, defaults 1 and 3) so its tip reaches it. Baked as an additive angle channel plus a multiplying scale channel; the follower's children are scaled with it. The leader may not depend on the follower (`the leader depends on the follower (it is one of its descendants)`).
  ```
  $ osatura stretch p.json st.json flw lead 1 2
  corrected 8 frames
  wrote st.json
  ```
* **`magnet`** bakes a proximity pull of a controller toward a bone head (followed per frame) or a fixed point: smooth falloff, full `strength` (0..1) at the target and zero at `radius`. Baked as additive offset channels.
  ```
  $ osatura magnet p.json mg.json lead 200,100 200 0.8
  pulled on 8 frames
  wrote mg.json
  ```
**Limits (ROADMAP §9):** all three are baked results from per-frame sampling: re-run after editing the source animation. `stretch` uses a uniform scale. `foot-lock` and `magnet` expect a root controller bone.

### `shot`, `camera-seq`

```
osatura shot       <project.json> <out.json> <name> <x> <y> <zoom> [rotation]
osatura camera-seq <project.json> <out.json> <frame:shot[:cut],...>
```
`shot` saves a named **camera shot** (centre x, y, zoom, optional rotation in degrees). `camera-seq` keys the camera through saved shots: at each `frame` the camera moves to the shot (eased) or, with the `:cut` suffix, jumps to it. Writes `camera_x/y/zoom/rotation` channels.

```
$ osatura shot small.json sh.json wide 320 240 1 15
wrote sh.json (1 shots)
$ osatura shot sh.json sh2.json close 400 200 2
wrote sh2.json (2 shots)
$ osatura camera-seq sh2.json cs.json 0:wide,4:close:cut
wrote cs.json
```

### `spring-fit`

**Synopsis:** `osatura spring-fit --start V --samples a,b,c,... [--fps 24]`

Fits spring **frequency (Hz) and damping ratio** to a drawn response curve (samples per frame of a step from `--start` to the last value). Prints the fitted pair and the JSON to paste into a project's `springs`. Does not read a project.

```
$ osatura spring-fit --start 0 --samples 0.5,0.9,1.1,1.0,0.98,1.0 --fps 24
frequency 4.68 Hz, damping 0.61 (rms error 0.028 of the step size)
add to the project: {"bone": "<bone>", "frequency": 4.68, "damping": 0.61}
```

### `script`

**Synopsis:** `osatura script <project.json> <out.json> <script.rhai>`

Runs a **sandboxed Rhai script** that edits the project through validated requests (a rejected request leaves everything unchanged:
`request 5 (box.LayerOpacity at 0) was rejected, nothing changed: ...`). Built-ins: `fps() frames() bones() layers() set_key(target, prop, frame, value[, ease]) add_bone(name, parent (empty = root), length, x, y, angle) add_rect add_circle add_polygon|add_blob add_line add_gradient_rect add_ik add_aim add_copy_rotation add_pin add_group(name[, [layers]]) add_switch move_layer set_parent delete_layer delete_bone set_layer set_bone add_channel_expr add_noise add_marker add_event bone_info layer_info world print`.
A script that applies no request prints `warning: script: 0 requests were applied`. Builds without the `scripting` feature answer `this build has no scripting support`.

```
$ cat t.rhai
add_rect("box", 10.0, 10.0, 50.0, 50.0, "#ff0000");
add_rect("box2", 80.0, 10.0, 50.0, 50.0, "#00ff00");
add_group("skins", ["box", "box2"]);
set_key("skins/box", "layer_opacity", 0, 1.0);
print(fps());
$ osatura script small.json sc.json t.rhai
24.0
4 request(s) applied; wrote sc.json
```
Layers inside a group are addressed by path (`skins/box`). `print` goes to stdout.

---

## 7. Physics, cloth and deformers

All of these **bake** a simulation or a geometric result into ordinary keyed channels: the project stays deterministic and the result plays back anywhere (including Lottie/GLB where applicable). Re-run the command after changing the source animation.
They take `<project.json> <out.json>` and `key=value` words (no `--`).

### `physics`

**Synopsis:**
```
osatura physics <project.json> <out.json> <bone,bone,...> [gravity=900] [wind=x,y] [gust=0.5] [damping=0.985]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [follow=bone] [--allow-empty]
    [drive=bone:gain;...] [vortex=bone|x,y:swirl[:radial[:radius]];...] [preset=hair|tail|rope|antenna|cape]
    [merge=keep|replace] [lateral=0..1]
```
Bakes a Verlet **chain simulation** (hair, tail, rope, antenna, cape) onto the bones listed (root of the chain first) as `bone_angle` keys, one per frame.

| Word | Meaning |
|---|---|
| `gravity=` | Downward acceleration (default 900 stage units/s^2). |
| `wind=x,y`, `gust=`, | Constant wind vector; gust amplitude of a noise-modulated wind. |
| `damping=` | Velocity damping per step (default 0.985). |
| `circle=x,y,r[@bone];...`, `capsule=x1,y1,x2,y2,r[@bone];...` | Colliders, optionally attached to a bone (they follow it). Capsules are accepted (verified with `capsule=100,300,300,300,20`; an older "capsule colliders NOT done" note in ROADMAP §9 is out of date). |
| `follow=bone` | The chain root is carried along with this bone's motion (inertia). |
| `drive=bone:gain;...` | Bodies that push the chain (their velocity times gain). |
| `vortex=bone\|x,y:swirl[:radial[:radius]];...` | Swirl fields around a bone or point. |
| `preset=` | Starting values for `hair tail rope antenna cape`; words written **after** `preset=` override it. |
| `merge=keep\|replace` | `keep` (default) leaves bones that already have animation untouched; `replace` overwrites them. |
| `lateral=0..1` | With several chains (`bone,bone/bone,bone`) ties neighbouring strands together. |
| `--allow-empty` | Accept a run that baked nothing. |

```
$ osatura physics p.json ph.json tail1,tail2,tail3 gravity=500 preset=tail
baked 3 bones; wrote ph.json
$ osatura physics p.json ph4.json tail1,tail2/tail3 lateral=0.5            # two strands
baked 3 bones; wrote ph4.json
$ osatura physics p.json ph3.json tail1,tail2,tail3 follow=arm_up circle=300,300,40 wind=10,0 gust=0.5 lateral=0.3 drive=lead:0.5
baked 3 bones; wrote ph3.json
$ osatura physics demo.json ph.json arm_low,hand gravity=500 preset=tail   # these bones are keyed already
kept existing animation on 'arm_low' (use merge=replace to overwrite)
kept existing animation on 'hand' (use merge=replace to overwrite)
physics: nothing baked - 2 bone(s) already have animation and merge=keep (the default) leaves it alone; nothing was written. Use merge=replace to overwrite the existing animation, or --allow-empty to accept an unchanged file     [exit 1]
$ osatura physics demo.json ph.json hand preset=bogus
unknown preset 'bogus' (try hair, tail, rope, antenna, cape)                [exit 1]
```
(`p.json` is the demo plus three unkeyed bones `tail1..tail3` made with a Rhai script - see `script`.) **Gotcha:** a bake that changes nothing is an error unless `--allow-empty`.
**Limits (ROADMAP §9):** colliders ignore bone scale (rigid transform only); static circles collide at points only (attached circles also along links); no capsule-capsule or self collision.

### `cloth`

**Synopsis:**
```
osatura cloth <project.json> <out.json> <layer> [pin=top,left@bone,row:N,col:N,v:N,corners,none] [gravity=900]
    [wind=x,y] [gust=0.5] [gust-hz=0.5] [gust-len=px] [stiffness=1] [shear=0.05] [bend=0.2] [max-stretch=1.1]
    [damping=0.99] [cols=N rows=N] [rect=x,y,w,h] [name=cloth] [substeps=4] [iterations=10] [preroll=frames]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [from=F] [to=F]
```
Bakes a deterministic **cloth surface** onto the mesh of an *image* layer, or onto a *vector* layer when `rect=x,y,w,h` is given (the vector layer gets a grid mesh over that rectangle), as per-vertex deform channels. `pin=` chooses pinned vertices (edges `top`, `left`..., `corners`, `row:N`, `col:N`, `v:N`, `@bone` attaches the pins to a bone, `none`). `from`/`to` limit the baked frame range, `preroll` runs the simulation before frame `from` so the cloth starts at rest.

```
$ osatura cloth small.json cloth.json sun rect=100,60,120,120 cols=6 rows=6 pin=top gravity=900 wind=40,0
baked cloth on 'sun' (84 channels); wrote cloth.json
$ osatura cloth small.json cloth.json sun
a vector layer needs rect=x,y,w,h                    [exit 1]
$ osatura cloth small.json cloth.json nosuch
no layer at 'nosuch'                                 [exit 1]
```
The `84 channels` are `lattice_dx` / `lattice_dy` keys (target `sun::cloth::<col>,<row>`) of the 42 unpinned points of the 7x7 grid (the top row is pinned). They are real channels in the file, so a cloth bake makes the project larger (98 channels instead of 14 here) - use `size-report` to see the cost. **Limits (ROADMAP §9):** the cloth is planar (no out-of-plane folds, so shear is compliant by default); it refuses free-form meshes (grid meshes only); no self-collision; baked per integer frame.

### `cage`

**Synopsis:** `osatura cage <project.json> <out.json> <layer> [rect=x,y,w,h|auto] [hull] [verts=N] [pad=P] [name=cage]`

Adds a **mean-value-coordinate cage deformer** around a layer: content inside the cage follows the cage vertices, content outside is unchanged. `auto` (default when no `rect`) uses the padded bounds, `verts=N` is the number of vertices per side, `hull` uses the convex hull, `pad=` pads the box, `name=` names the deformer (default `cage`).

```
$ osatura cage small.json cg.json sun auto verts=3
added cage 'cage' (12 vertices) to 'sun'; wrote cg.json
```
**Limits (ROADMAP §9):** no automatic cage fitting beyond a padded box or convex hull (no concave tracing); there is no smooth falloff outside the cage (a visible step appears where artwork crosses the cage border); free-form meshes have no per-vertex keyed offsets (use a cage).

### `cage-pose`

**Synopsis:** `osatura cage-pose <project.json> <out.json> <layer> <frame> [name=cage] [move=i:dx,dy;...] [all=dx,dy] [ease=linear]`

Keys cage vertex offsets at a frame (`cage_dx` / `cage_dy` channels). `move=` offsets individual vertices, `all=` offsets every vertex. Locked channels are refused.

```
$ osatura cage-pose cg.json cp.json sun 4 all=5,0
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
$ osatura cage-pose cg.json cp.json sun 4 move=0:5,5
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
```

### `warp-pivot` and `soft-scale`

**Synopsis:**
```
osatura warp-pivot <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=1,4-7] [region=x,y,w,h]
    [pivot=x,y|bone:NAME|centroid|center] [rotate=DEG] [scale=S|sx,sy] [move=dx,dy] [radius=R] [falloff=smooth] [ease=linear]
osatura soft-scale <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=..|region=..] scale=S|sx,sy
    [radius=R] [falloff=smooth] [pivot=..]
```
`warp-pivot` rotates/scales/moves a **selection** of control points (cage, lattice, cloth, pin, spine deformers; a grid image mesh gets a per-vertex warp grid) about a pivot and keys the result at the frame. The selection is `verts=` (indices and ranges) and/or `region=`; an empty selection is an error (`the selection is empty (give verts= and/or region=)`). `radius=` adds a **soft falloff** (`linear smooth gauss sharp constant`). `soft-scale` is proportional (soft-selection) scaling; without `verts`/`region` the whole mesh is resized about its bounding-box centre by `scale=F`, `width=W`, `height=H` or `size=WxH`, with `aspect=keep|free`.

```
$ osatura warp-pivot cg.json wp.json sun 4 verts=0-3 rotate=15 pivot=centroid radius=30 falloff=gauss
warp-pivot on 'sun' keyed at frame 4; wrote wp.json
$ osatura soft-scale cg.json ss.json sun 4 scale=1.2 pivot=center
soft-scale on 'sun' keyed at frame 4; wrote ss.json
```

### `layer-pivot`

**Synopsis:** `osatura layer-pivot <project.json> <out.json> <layer> pivot=x,y|bone:NAME`

Moves a layer's pivot (anchor) **without changing the picture**, so rotation and scale channels turn about the new point.

```
$ osatura layer-pivot small.json lp.json sun pivot=100,100
moved the pivot of 'sun'; wrote lp.json
```

### `mesh-lock`

**Synopsis:** `osatura mesh-lock add <project.json> <out.json> <name> host=LAYER[:path] follower=LAYER[:path] [verts=1,4-7] [strength=1]`
and `osatura mesh-lock list|remove <project.json> [<out.json> <name>]`

Makes the follower's vertices follow the **surface of the host mesh** every frame (barycentric displacement transfer), e.g. a shirt following a body. Cycles are refused; host and follower must have the same `parallax` (`mesh lock 'lock1': host and follower must have the same parallax`).

```
$ osatura mesh-lock add two.json ml1.json lock1 host=ra follower=rb verts=0-1 strength=0.5
added mesh lock 'lock1'; wrote ml1.json
$ osatura mesh-lock list ml1.json
lock1: rb:0 follows ra:0 vertices [0, 1] strength 0.5
1 mesh lock(s)
$ osatura mesh-lock remove ml1.json ml2.json lock1
removed mesh lock 'lock1'; wrote ml2.json
```

### `glue`

**Synopsis:** `osatura glue add <project.json> <out.json> <name> a=LAYER[:path] b=LAYER[:path] pairs=i:j;i:j|auto=RADIUS [mode=rigid|snap|offset] [strength=1]`
and `osatura glue list|remove <project.json> [<out.json> <name>]`

Glues vertices of layer `b` to vertices of layer `a` (b follows a). `pairs=i:j;...` lists vertex pairs explicitly, `auto=RADIUS` pairs every `b` vertex that lies within the radius of an `a` vertex. Modes: `rigid` keeps the rest offset in the frame of `a`, `snap` makes them coincide, `offset` keeps a constant offset.

```
$ osatura glue add two.json gl1.json g1 a=ra b=rb auto=5 mode=snap
added glue 'g1' (4 pair(s)); wrote gl1.json
$ osatura glue list gl1.json
g1: Snap strength 1 ra:0#0->rb:0#0 ra:0#1->rb:0#1 ra:0#2->rb:0#2 ra:0#3->rb:0#3
1 glue(s)
$ osatura glue add small.json x.json g1 a=sun b=hills auto=20 mode=snap
no vertex of 'hills' lies within 20 of a vertex of 'sun'          [exit 1]
```

### `mesh-trace`

**Synopsis:** `osatura mesh-trace <project.json> <out.json> <layer> [density=24] [tolerance=1.5] [smooth=N] [hotspot=x,y,r;...] [edge=D/4] [grading=0.5] [lambda=0.5] [taubin=1] [smooth-boundary=0] [threshold=128] [bones=R] [hot-size=E] [skin=1]`

Traces an **image layer's alpha silhouette** into a variable-density free triangle mesh (denser along the outline, around hot-spots and bones), smooths it, and skins it to the bones. Prints vertex/triangle counts and quality.

```
$ osatura mesh-trace o.json mt.json top density=12 tolerance=2 bones=0
wrote mt.json: 9589 vertices (1494 on the outline), 17682 triangles, area 1227677 px2
  quality min 0.484 mean 0.890, min angle 18.3 deg, mean edge 3.0 px at the outline / 13.7 px inside
```
(`o.json` is a 1280x960 OpenRaster import; lower `density` makes fewer, larger triangles. The run took 5 s in a debug build.) **Limits:** needs an image layer (an alpha silhouette); a mesh this dense makes the project file large - check `size-report`. Free meshes cannot be cloth-simulated.

### `skin-diffuse`

**Synopsis:** `osatura skin-diffuse <project.json> <layer> <out.json> [--influences 4]`

Computes **heat-diffusion skin weights** for a meshed layer from the project's bones, keeping at most `--influences` (default 4) bones per vertex. Note the argument order: **layer before output**. The layer must be an image layer with a mesh.

```
$ osatura skin-diffuse demo.json arm sd.json
layer 'arm' is not an image layer                          [exit 1]
$ osatura skin-diffuse o.json top sd.json
layer 'top' has no mesh                                    [exit 1]
$ osatura skin-diffuse mt.json top sd.json               # mt.json from mesh-trace
wrote sd.json
```

---

## 8. Audio, lip-sync and sounds

Audio inputs for the analysis commands (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) must be **WAV** (8/16/24/32-bit integer or 32-bit float PCM, any channel count mixed down to mono, up to 256 MiB). MP3/OGG need converting first (`ffmpeg -i in.mp3 in.wav`). Sound **tracks** stored in a project (`sound`) may be any format ffmpeg can read.

### `lipsync`

**Synopsis:** `osatura lipsync <project.json> <audio.wav> <switch_layer> <out.json> [--map 0,1,2,3,4]`

Drives a **switch layer** (mouth shapes) from loudness and spectral brightness: an amplitude / zero-crossing heuristic that picks one of five shapes per frame (`rest, small, open, round, wide`) and removes flicker. **It is not phoneme recognition.** `--map` gives the switch child index for each of the five shapes; the default follows the switch's child count: 1 child -> all; 2 -> closed + open; 3 -> closed + open + round/wide; 4 -> closed + small + open + round/wide; 5+ -> in the order above. Replaces an existing keyed `layer_switch` channel of that layer. The project length is **not** changed (a 2 s WAV on an 8-frame project keys 8 frames).

```
$ osatura lipsync small.json speech.wav mouth lip.json
wrote lip.json (2.00s of audio → 8 frames)
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1,2,1,0
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1
--map needs exactly 5 numbers (rest,small,open,round,wide)          [exit 1]
$ osatura lipsync small.json speech.wav ink lip.json
layer 'ink' is not a switch layer                                   [exit 1]
$ osatura lipsync small.json notwav.mp3 mouth x.json
invalid WAV: Failed to read enough bytes.                           [exit 1]
```

### `lipsync-text`

**Synopsis:** `osatura lipsync-text <project.json> <audio.wav> <transcript.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--lang ro|en] [--words-out timing.json|timing.srt]`

Aligns a **transcript** to the audio (energy + text alignment: words and syllables are laid over the speech regions found in the audio) and derives mouth shapes from the syllables. `--lang` (default `ro`) selects the vowel/syllable rules; only `ro` and `en` exist. `--words-out` writes the word timings as JSON (words with syllables) or, for a `.srt` name, as subtitles.

```
$ osatura lipsync-text small.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (3 words, 2 exposure changes; energy + text alignment, not phoneme recognition)
$ head -c 140 words.json
{ "words": [ { "text": "hello", "start": 0.0, "end": 0.22, "syllables": [ {"text":"hel","start":0.0,"end":0.16}, {"text":"lo",...
$ osatura lipsync-text ... --words-out w.srt ; head -4 w.srt
1
00:00:00,000 --> 00:00:01,220
hello there world
```
**Limits (ROADMAP §9):** no per-letter (phoneme-level) timing, no speaker-specific rate models, no language beyond ro/en. A synthetic tone is aligned as speech - use real recordings.

### `speech-motion`

**Synopsis:** `osatura speech-motion <project.json> <audio.wav> <transcript.txt> <out.json> [--head bone] [--brow bone_or_layer] [--nod deg] [--fall deg] [--tilt deg] [--brow-px px] [--seed n] [--lang ro|en] [--words-out timing.json|timing.srt]`

Adds **additive** head nods, falls, tilts and brow raises timed on the aligned words. `--head` names the head bone, `--brow` a bone or layer for eyebrows; the angle options are in degrees, `--brow-px` in pixels, `--seed` fixes the variation.

```
$ osatura speech-motion small.json speech.wav transcript.txt sm.json --head arm_up --lang en
wrote sm.json (1 additive channel(s) from 3 words)
```

### `mouth`

**Synopsis:** `osatura mouth <project.json> <cues.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--rhubarb]`

Imports ready-made mouth **cues** instead of analysing audio: Papagayo/Moho-switch files (`frame PHONEME` per line, optional first line `MohoSwitch1`) or Rhubarb TSV (`seconds<TAB>shape`, recognised by the decimal point; seconds are converted at the project fps). Blank lines and `#` comments are ignored. Phonemes map to the five shape classes (`rest/MBP/X/A` rest; `etc/FV/L/B/G/H` small; `AI/D` open; `O/U/WQ/F` round; `E/C` wide); unknown shapes count as `small`. `--rhubarb` treats Rhubarb's `E` as *round* (Papagayo's `E` is wide). Default `--map` is `0,1,2,3,4`.

```
$ printf '0 rest\n2 open\n4 round\n6 wide\n' > cues.txt
$ osatura mouth small.json cues.txt mouth mo.json --rhubarb
wrote mo.json (4 cues)
```

### `audio-react`

**Synopsis:** `osatura audio-react <project.json> <out.json> <audio.wav> <target> <prop> [gain=1] [offset=0] [floor=0.05] [attack=1] [release=4] [add=on]`

Loudness drives a property: one key per frame following the smoothed level (RMS relative to the loudest frame). Output = (level gated by `floor`) x `gain` + `offset`; `attack`/`release` are the rise/fall times in frames; `add=on` makes the channel additive instead of replacing. A locked channel is refused. Options are `key=value` words; an unknown one is `unknown option 'foo'`.

```
$ osatura audio-react small.json ar.json speech.wav ink layer_rotation gain=10
wrote ar.json
$ osatura audio-react small.json ar.json speech.wav arm_up bone_angle foo=1
unknown option 'foo'                                                [exit 1]
```

### `beats`

**Synopsis:** `osatura beats <project.json> <out.json> <audio.wav> [sensitivity=1.5] [gap=4] [as=markers|events] [name=beat]`

Detects loudness jumps above the recent average (`sensitivity` 1.2-3, higher = fewer beats) at least `gap` frames apart, and records them as timeline **markers** (default) or **events** named `name`. Prints the frames found.

```
$ osatura beats small.json bt.json speech.wav as=events name=hit sensitivity=1.2
1 beats at frames [1]; wrote bt.json
$ osatura beats small.json bt.json speech.wav
0 beats at frames []; wrote bt.json
```

### `waveform`

**Synopsis:** `osatura waveform <audio.wav> <out.json> [buckets=800]`

Min/max peaks for drawing a waveform: `{"sample_rate","duration","peaks":[[min,max],...]}`. Does not read a project.

```
$ osatura waveform speech.wav wf.json buckets=50
wrote wf.json (50 buckets)
```

### `sound`

**Synopsis:**
```
osatura sound list   <project.json>
osatura sound add    <project.json> <out.json> <name> <file.wav|ogg|mp3|...> [start_frame] [gain] [trim/fade options]
osatura sound edit   <project.json> <out.json> <name> [--start F] [--gain G] [trim/fade options]
osatura sound remove <project.json> <out.json> <name>
osatura sound mix    <project.json> <out.wav|out.ogg|out.mp3>
    trim/fade options (seconds, or frames with an f suffix such as 36f): --trim-in T --trim-out T --fade-in T --fade-out T
```
Sound tracks are **references** (file path relative to the project, start frame, gain 0..4, trim window inside the file, fades). Nothing is embedded in the project; the audio file must travel with it, and must lie inside the project directory unless `--allow-external-assets` is given. `edit` changes the given fields only (`0` clears a trim). `--trim-out 0` means "to the end". `sound mix` renders only the mixed audio for the **project length** (through ffmpeg: wav = 44.1 kHz PCM 16-bit, ogg = Vorbis q4, mp3 = 96 kb/s).
Mixed audio is also muxed into `.mp4` (AAC), `.webm` (Opus) and embedded in `.html`; `.gif`/`.apng` cannot carry it.

```
$ osatura sound add small.json s1.json music speech.wav 2 0.8 --fade-in 0.5 --trim-out 36f
wrote s1.json
$ osatura sound list s1.json
music: speech.wav at frame 2 gain 0.8 trim 0s..1.5s fade-in 0.5s
$ osatura sound edit s1.json s2.json music --gain 0.5 --trim-in 0.1 --fade-out 4f
$ osatura sound list s2.json
music: speech.wav at frame 2 gain 0.5 trim 0.1s..1.5s fade-in 0.5s fade-out 0.16666667s
$ osatura sound mix s2.json mix.mp3
wrote mix.mp3 (0.33 s)
$ osatura sound mix s2.json mix.flac
unknown audio format '.flac' (use .wav, .ogg or .mp3)            [exit 1]
$ osatura sound add small.json x.json x nosuch.wav              # a missing file is refused, nothing written
```
`check` fails when a referenced sound file is missing or unreadable. Without ffprobe, only existence (not that the file is audio) is checked for non-WAV files. **Limits (ROADMAP §9):** no per-track pan/EQ/ducking, only the linear fade curve, no looping of a clip, no re-timing.

---

## 9. Import

### `import-svg`

**Synopsis:** `osatura import-svg <file.svg> <out.json> [--split]`

Usage line: `osatura import-svg <file.svg> <out.json> [--split]  SVG subset -> one vector layer, or one layer per shape named by its id (warnings for the rest)`

Creates a new project (24 fps, 24 frames; size from the `viewBox` or `width`/`height`, else 512x512) with **one vector layer `svg`** holding the shapes of an SVG subset: `path` (all commands), `rect` (rounded corners ignored), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` with `transform`, and solid `fill`/`stroke`/`stroke-width`. Everything else is skipped with a `warning:` line **on stdout** (when the element is outside `<defs>`; a `<filter>`, `<clipPath>`, `<mask>` or `<pattern>` inside `<defs>` and the `filter`, `clip-path` and `mask` attributes are ignored without a warning): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`/`<radialGradient>`, `<image>`, `<use>`, `<foreignObject>`. A gradient paint (`fill="url(#g)"`) falls back to grey. Limits: 8 MB of SVG, 200,000 anchors, a cap on elements.

```
$ osatura import-svg s.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
warning: <filter> is not supported and was skipped
warning: <image> is not supported and was skipped
wrote svg.json (3 paths)
$ osatura check svg.json
ok: 's' 200×100 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 0 channels
```
With `--split` every shape becomes **its own layer**, named after the shape's `id` in file order. A shape without an id is named after its tag and position (`rect1`, `circle6`), a repeated id gets `_1`, `_2`..., and characters a layer name may not contain (`/`, `::`, control characters) become `_`; names are cut to 100 characters. That is the form to use when the drawing is going to be rigged part by part; see [TUTORIAL_SVG.md](TUTORIAL_SVG.md).

```
$ osatura import-svg pip.svg pip.json --split
wrote pip.json (16 paths, 16 layers)
```
Import-then-merge: there is no command that merges the layer into an existing project; use `script` or edit the JSON.

### `import-ora`

**Synopsis:** `osatura import-ora <file.ora> <out.json> [--fps N] [--frames N]`

OpenRaster -> project: one **image layer per raster layer** (embedded PNG), bottom-to-top order preserved, hidden layers kept but invisible, groups flattened, per-layer `opacity`, `visibility`, position (`x`,`y`) and the common `svg:` composite operations (multiply, screen, overlay, darken, lighten, color-dodge/burn, hard/soft-light, difference, exclusion, plus -> add). The project has `--frames` frames (default 1) at `--fps` (default 24). A nested stack's opacity multiplies into its layers and its blend mode is inherited, but overlapping layers inside a translucent stack are **not** flattened first (they show through each other). Duplicate layer names get `_1`, `_2` suffixes.

```
$ osatura import-ora t.ora ora.json
wrote ora.json (2 layers)
$ osatura import-ora t.ora ora2.json --fps 12 --frames 24
$ osatura import-ora layer.png bad.json
not a zip/ORA file: invalid Zip archive: Could not find EOCD       [exit 1]
```
Limits: 2000 archive entries, 128 MiB per entry, 64 Mpixel layers.

### `import-lottie`

**Synopsis:** `osatura import-lottie <in.json> <out.json> [fx.json]`

Imports a Lottie (Bodymovin) file - a deliberate **subset** - as a project. Prints `imported N layers; wrote <out>` and one `warning:` line per skipped feature.

* **Imported:** shape layers with `sh` paths, `rc`, `el`, solid and gradient fills (animated gradient end points/centre/radius become `grad_dx/grad_dy` channels), strokes, groups, layer transforms (static or keyframed position/scale/rotation/opacity, curved position paths baked per frame), blend modes, a solid background layer, image layers with embedded PNG/JPEG data URIs, null layers as parents (parent transforms are baked into children), layer masks (add/subtract/intersect with opacity, inversion, keyed shapes), luma mattes (approximated), precompositions, layer in/out ranges (as an opacity step track), keyframe easing (hold -> step; bezier handles -> cubic-bezier; no handles -> linear).
* **Animated paths:** a path whose shape keys have the same topology (same anchor count, open/closed) becomes a base path plus one **morph target** per further distinct key (`k1`, `k2`, ...) with `path_morph` weight channels carrying the easing. Different topology: the first shape is used and a warning is printed.
* **Effects (`ef`):** Gaussian blur, drop shadow, tint and fill become **per-layer effects** (`effects` on the layer, any layer kind) with animated numbers as `fx_*` channels; static tint/fill/tritone/levels on shape layers are baked into colours. Effect colours cannot be animated (first value is used); other effects, and colour effects after a blur/shadow on non-shape layers, are warnings. At most the first few effects per layer are kept.
* **Skipped with a warning:** text and other layer types, audio assets, animated anchor points (first value), curved motion paths on properties other than position (straight segments), masks on a track-matte source, parent chains too long to bake, more than 2000 layers, animation longer than 20,000 frames.
* `fx.json` is kept for compatibility: an empty array is written (blur and drop shadow are now per-layer effects inside the project).

```
$ osatura import-lottie out2.json li.json                   # a Lottie file made by `osatura export`
imported 11 layers; wrote li.json
$ osatura import-lottie in.json out.json                    # hand-made file: animated triangle with animated blur, plus a null and a text layer
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
$ osatura check out.json
ok: 't' 200×200 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 2 channels       # channels: blob::path::k1 path_morph, blob::fx0 fx_blur
$ osatura import-lottie out2.json li2.json fx.json
wrote 0 post effect(s) to fx.json
imported 11 layers; wrote li2.json
$ echo '{}' > bad.json; osatura import-lottie bad.json o.json
missing 'w'                                                        [exit 1]
```
**Gotcha:** `import-lottie in.json in.json` is refused (`refusing to overwrite the input ...`), also for `fx.json`. **Limits:** `.lottie` (zip) input is not read (`stream did not contain valid UTF-8`) - unzip `animations/animation.json` first; quadratic work in shape morphs is bounded only by the size caps.

### `import-bvh`

**Synopsis:** `osatura import-bvh <project.json> <file.bvh> <out.json> --joint-map Joint=bone,.. [--axis Zrotation --tol 0.5]`

Maps chosen BVH joint rotation channels onto rig bones as keyed `bone_angle` channels sampled at the project fps and frame count. `--joint-map` is required; `--axis` (default `Zrotation`) selects which BVH rotation channel (`Zrotation`, `Xrotation`, `Yrotation`) is read; `--tol` (default 0.5 degrees) simplifies keys. For automatic matching, root motion, views and scale use `mocap-retarget` (section 10).

```
$ osatura import-bvh demo.json m.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```

### `import-seq`

**Synopsis:** `osatura import-seq <project.json> <out.json> <layer_name> <folder> [hold=1] [loop=on|off] [start=0] [x=0] [y=0]`

Imports a **PNG sequence** (all PNGs of the folder in file-name order) as a frame-by-frame layer: a switch layer named `<layer_name>` with one image layer per frame and a step channel exposing each for `hold` frames from frame `start`, at position `(x, y)`; `loop=on` repeats it. All images must have the same size (`img.png: size 640x480 differs from the first image (160x120)`).

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20
imported 3 images; wrote is.json
```

### `import-anim`

**Synopsis:** `osatura import-anim <project.json> <out.json> <layer_name> <file.gif|file.apng|file.png> [loop=on|off] [start=0] [x=0] [y=0]`

Imports an **animated GIF or APNG** as a frame-by-frame layer using the file's own frame delays (converted to project frames). A plain PNG gives one frame.

```
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0
imported 4 frames as layer 'gifl'; wrote ia.json
$ osatura import-anim demo.json ia2.json apngl dof.apng
imported 3 frames as layer 'apngl'; wrote ia2.json
```
Frame images become embedded PNGs: a long GIF makes the project large.

### `trace`

**Synopsis:** `osatura trace <image.png> <out.json> [--threshold 128 --tolerance 1.0 --min-area 12 --fill #000000]`

Simple bitmap -> vector for images with **transparency**: it traces the PNG's **alpha channel** (marching squares at `--threshold`, 0..255, with linear edge interpolation), simplifies the contours (Ramer-Douglas-Peucker) to `--tolerance` pixels, drops regions under `--min-area`, detects holes by containment, and writes a new project (named `traced`, canvas = image size) with the contours as filled paths of colour `--fill`. A fully opaque image therefore yields one contour around the whole picture (as in the example). Prints `wrote <out> (N contours)`. Use `autotrace` for colour images.

```
$ osatura trace img.png tr.json --threshold 128 --tolerance 1.0 --min-area 12 --fill '#112233'
wrote tr.json (1 contours)
```

### `autotrace`

**Synopsis:** `osatura autotrace <image.png|jpg> <out.json|out.svg> [--mode fill|line] [--colors N=8] [--min-area A=8] [--tol T=1] [--smooth N=3] [--corner DEG=55] [--denoise N=0] [--quant kmeans|mediancut] [--seed S] [--fit bezier|polygon|vw] [--zorder area|luminance] [--threshold T] [--invert] [--prune N=4] [--into project.json] [--name L] [--fit-stage] [--png preview.png]`

Local bitmap tracing - **no model, no network**: colour quantisation (`--colors`, k-means or median cut), contour extraction, Schneider cubic Bezier fitting (`--fit`; `polygon` and Visvalingam-Whyatt `vw` are alternatives), holes by even-odd; `--mode line` thins strokes to centre lines (Zhang-Suen, `--prune` removes short spurs). The output is a new project, a new `.svg`, or - with `--into project.json` - a copy of an existing project with a new layer group `--name` (default `trace`); `--fit-stage` (with `--into`) scales the new group uniformly to fit and centre it on the project's stage, `--png` writes a preview rendering. It prints counts and the **measured error against the source** (mean absolute error, RMSE, PSNR, SSIM).

```
$ osatura autotrace img.png at.json --colors 4 --png prev.png
layer group 'trace' added
wrote at.json: 44 regions, 0 strokes, 44 paths, 324 anchors
fidelity vs source: MAE 13.38/255, RMSE 17.59, PSNR 23.2 dB, SSIM 0.960
$ osatura autotrace img.png at.svg --mode line --threshold 100
wrote at.svg: 0 regions, 11 strokes, 11 paths, 47 anchors
fidelity vs source: MAE 52.47/255, RMSE 58.73, PSNR 12.8 dB, SSIM 0.903
$ osatura autotrace img.png at2.json --into d.json --name traced
layer group 'traced' added
wrote at2.json: 51 regions, 0 strokes, 51 paths, 347 anchors
```
**Limits (ROADMAP §9):** a classical tracer - photographs become colour bands; no learned or semantic tracing. `--png` here is a value option (preview file name), unlike other `--png` flags.

### `autotrace-bench`

**Synopsis:** `osatura autotrace-bench`

Traces built-in synthetic shapes, gradients and strokes and prints a fidelity table (MAE, RMSE, PSNR, SSIM, region/anchor counts, milliseconds). Takes no arguments; read-only.

```
$ osatura autotrace-bench
image          mode  colors  tol     MAE    RMSE   PSNR   SSIM regions strokes  anchors      ms
shapes         fill       6    1    2.19   12.10   26.5  0.975      11       0       33    13.6
...
```

---

## 10. Live input, MIDI, face tracking and motion capture

### `live-map`

**Synopsis:** `osatura live-map <project.json> <events.json|song.mid> <mapping.json> <out.json> [--takes lib.json] [--tol 0.01] [--allow-empty]`

Turns a **recorded live performance** (MIDI, mouse, gamepad, keys, face signals) into ordinary keyed channels, events and takes (non-destructive, all-or-nothing, locks respected). The input is either a `.mid` Standard MIDI File or an events JSON `{"events":[{"source","id","value","time"}]}` (`value` normalised 0..1, `time` seconds; `source` = `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`; `id` = MIDI `"<channel 1-16>:<number>"`, mouse button `"0"`, gamepad `"<pad>:<index>"`, key = DOM code such as `KeyA`).

The mapping file has `params` (source/id -> `target`+`prop`, with `in_min in_max out_min out_max deadzone invert curve smooth additive`) and `triggers` (threshold crossing -> action `jump_to_frame`, `play_take`, `set_key`, ...); unknown keys are errors. `--takes` is a take library for `play_take` actions (must exist), `--tol` simplifies keys.

```
$ cat map.json
{"params":[{"source":"mouse_x","id":"0","target":"arm_up","prop":"bone_angle","out_min":-30,"out_max":30}],
 "triggers":[{"source":"key","id":"KeyA","action":{"type":"jump_to_frame","frame":3}}]}
$ osatura live-map small.json ev.json map.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 1 events (0 after the last frame skipped); wrote out.json
$ osatura live-map small.json song.mid mapm.json outm.json              # MIDI CC 1 on channel 1 -> arm_up
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote outm.json
$ osatura live-map small.json ev.json map.json out.json --takes lib.json    # lib.json missing
cannot read lib.json: No such file or directory (os error 2)             [exit 1]
$ osatura live-map small.json empty.json map.json out.json
note: map 0 (arm_up.BoneAngle): no matching events, nothing keyed
0 events -> +0 channels, ...; wrote out.json
```
**Gotcha:** if events were read but **nothing was produced** (typo in an id, wrong source) the command fails and writes nothing unless `--allow-empty`; with zero input events it just writes the unchanged project. Jump triggers are recorded as an event named `jump` with `data.to` (the time axis is not rewritten).
**Limits (ROADMAP §9):** no webcam/audio-level live input in the studio via this command (`audio-react` is offline); the studio provides the interactive side.

### `drive`

**Synopsis:** `osatura drive <project.json> <signals.csv> <maps.json> <out.json> [--tol 0.5]`

Performance -> keys from a **CSV**: the header row names the signals, each following row is one frame (row index = frame). `maps.json` is an array of `{input, target, prop, in_min, in_max, out_min, out_max, deadzone, invert, curve, smooth}`; `--tol` (default 0.5) simplifies the keys.

```
$ printf 'jaw,speed\n0,0\n0.5,0.2\n1,0.4\n0.5,0.6\n' > sig.csv
$ echo '[{"input":"jaw","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":40,"smooth":0.2}]' > maps.json
$ osatura drive demo.json sig.csv maps.json dr.json --tol 0.5
wrote dr.json (+1 channels)
```

### `face-synth`, `face-train`, `face-eval`, `face-detect`, `face-track`, `face-calibrate`, `face-bake`

A face-driven pipeline that is **entirely local and trained only on synthetic faces** (stylised/cartoon heads, 23 landmarks, layout `osatura23`). A small synthetic-trained model is built into the binary; `--model` overrides it. **Photographs, webcams and real video are not validated** (ROADMAP §9; see also [PATENT_NOTES.md](PATENT_NOTES.md): face-driven puppeteering has not been patent-screened).

| Command | Synopsis | What it does |
|---|---|---|
| `face-synth` | `osatura face-synth <out-dir> [--frames 90] [--size 160] [--seed 7] [--fps 30]` | Synthetic head animation: PNG frames `000000.png...`, `truth.ndjson` (true landmarks) and `params.json` (true signals); `--fps` is the frame rate the signals are timed at (written to the NDJSON header). |
| `face-train` | `osatura face-train --samples N --seed S --out model.bin [--mine-rounds 1] [--scene N] [--reg-win N] [--reg-cell N] [--det-win N] [--det-cell N] [--ridge-lambda X] [--stages N] [--refine-win N] [--stage-patch N] [--stage-cell N] [--stage-lambda X]` | Trains the own detector on synthetic faces only (no downloads). `--scene` is the training image size (64..512); `--reg-win`, `--reg-cell`, `--det-win`, `--det-cell`, `--ridge-lambda`, `--stages`, `--refine-win`, `--stage-patch`, `--stage-cell` and `--stage-lambda` are the model-size and regression tuning knobs (the defaults are the ones the built-in model was trained with). |
| `face-eval` | `osatura face-eval [--model m.bin] [--eval-n 200] [--seed 9] [--scene 128] [--report r.json] [--json]` | Held-out synthetic accuracy, false positives and robustness sweeps (JSON; `--report` writes it, `--json` prints it even with `--report`; `--scene` is the test image size). |
| `face-detect` | `osatura face-detect <image> [--model m.bin] [--json] [--overlay o.png] [--threshold T] [--max-faces N] [--min-side N] [--max-side N] [--max-dim N]` | Face boxes and 23 landmarks; `--overlay` draws them on a PNG; `--min-side` / `--max-side` bound the face size in pixels and `--max-dim` scales a large image down first. |
| `face-track` | `osatura face-track <video-or-frame-dir> <out.ndjson> [--model m.bin] [--fps 30] [--max-dim 256]` | Landmarks NDJSON for `face-bake`. A directory of images is read directly; a video goes through ffmpeg. |
| `face-calibrate` | `osatura face-calibrate <landmarks.ndjson> <out.json> [--calibrate-frames N] [--map mapfile.json] [--fps F] [--layout L]` | Computes the neutral-face calibration from the first N frames. |
| `face-bake` | `osatura face-bake <project.json> <landmarks.ndjson> <mapfile.json> <out.json> [--calibrate-frames N\|0] [--calib calib.json] [--fps F] [--layout L] [--tol T] [--allow-empty]` | Bakes face signals (`mouth_open`, `eye_l_open`, `eye_r_open`, `brow_raise`, `yaw`, `pitch`, `roll`, ...) to channels with a `live-map` style mapping file whose `source` is `face`. |

```
$ osatura face-synth syn --frames 6 --size 96 --seed 3
wrote 6 frames (96x96), truth.ndjson and params.json to syn
$ osatura face-train --samples 80 --seed 1 --out m.bin --scene 64        # 47 s in a debug build
wrote m.bin (649900 bytes) in 47.0s: 80 samples, seed 1, mined negatives 9
trained on synthetic faces only: expect stylised/cartoon faces to work, photographs and webcams are NOT validated
$ osatura face-train --samples 60 --seed 1 --out m.bin --scene 2
scene size must be 64..512                                                [exit 1]
$ osatura face-detect syn/000000.png --model m.bin
syn/000000.png: face 0: box centre (48.1, 47.4) side 69.0 score 4.49
$ osatura face-track syn trk.ndjson --model m.bin --fps 15 --max-dim 96
wrote trk.ndjson: 6 frames, face found in 6
$ osatura face-calibrate syn/truth.ndjson calib.json --calibrate-frames 3 --fps 15
6 frames, neutral from the first 3; wrote calib.json
$ echo '{"params":[{"source":"face","id":"mouth_open","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":60}],"triggers":[]}' > map.json
$ osatura face-bake small.json trk.ndjson map.json baked.json --calibrate-frames 3 --fps 15 --tol 0.5
6 landmark frames -> +1 channels (K keys in them), 0 trigger keys set, 0 events; wrote baked.json
$ osatura face-eval --eval-n 6 --model m.bin --report ev.json            # writes the JSON report
```
These commands write their model/report files with the atomic, symlink-refusing writer (parent folders are created). `face-bake` and `live-map` report `+N channels (K keys in them), M trigger keys set`, and warn on stderr when the recording is longer than the project (the extra frames are not baked; the project length is unchanged).

### `mocap-info`, `mocap-match`, `mocap-retarget`

```
osatura mocap-info     <file.bvh>
osatura mocap-match    <project.json> <file.bvh> [--map map.json]
osatura mocap-retarget <project.json> <file.bvh> <out.json> [--view front|side|top|yaw=DEG] [--map map.json] [--scale auto|N] [--fps F]
    [--smooth N] [--start F] [--end F] [--rest-frame F] [--min-conf C] [--weight W] [--mirror] [--no-root] [--in-place] [--keep-length] [--footlock BONE --ground Y]
```
* **`mocap-info`** prints the hierarchy, channels, frame rate and extent of a BVH file.
  ```
  $ osatura mocap-info m.bvh
  4 joints (2 end sites), 15 channels per frame
  4 frames at 0.0417 s (24.000 fps), duration 0.125 s
  Hips  offset (0.000, 0.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
    LeftUpLeg  offset (5.000, 0.000, 0.000)  [Zrot Xrot Yrot]
  ...
  ```
* **`mocap-match`** prints the **automatic joint -> bone matching table** (slot, BVH joint, rig bone, confidence, evidence `via`, length ratio, note), then the unmatched joints/bones and warnings. Names and topology are both used. `--map` is a JSON object `{"bvh joint": "rig bone" | null}` (`null` or `""` excludes a joint; `{"map": {...}}` wrapping is accepted). It always exits 0 - when nothing matched it only warns `no bones matched: supply a --map file`.
  ```
  $ osatura mocap-match demo.json m.bvh --map map2.json         # {"Hips":null,"LeftUpLeg":"thigh","LeftLeg":"shin"}
  slot         bvh joint                  rig bone                conf  via         len x  note
  shin.l       LeftLeg                    shin                    1.00  manual      12.00  manual
  thigh.l      LeftUpLeg                  thigh                   1.00  manual      12.00  manual
  unmatched BVH joints: Hips, RightUpLeg
  ...
  ```
* **`mocap-retarget`** bakes the motion as **appended** channels: projects the 3-D rotations to the 2-D `--view` (`front` default, `side`, `top`, or `yaw=DEG`), scales root motion (`--scale auto` or a number), resamples to `--fps` (default the project's), smooths (`--smooth` frames), takes `--start..--end` BVH frames, uses `--rest-frame` as the rest pose, drops matches below `--min-conf`, blends with `--weight`, mirrors with `--mirror`, drops root motion with `--no-root`, converts a travelling clip to in-place with `--in-place`, and pins a foot with `--footlock BONE --ground Y`. Prints the table, notes and `N bones, F frames at R fps, root scale S, +C channels; wrote <out>`. A clip longer than the project **extends the project length**; a note after the summary says from/to which length. `--keep-length` keeps the project length instead and truncates the keys at the last frame (an interpolated key is placed there; a note counts the keys not baked). With nothing above the confidence threshold: `no bone matches to retarget (all below the confidence threshold?)` (exit 1). Note: for this command `--scale` is `auto|N`, not the usual render scale.
  ```
  $ osatura mocap-retarget demo.json m.bvh mr.json --map map2.json --view side --smooth 1 --in-place --no-root
  ... (table)
  2 bones, 4 frames at 24 fps, root scale 12.0000, +2 channels; wrote mr.json
  ```
**Limits (ROADMAP §9):** no real motion-capture file was tested (none is bundled: the examples use a hand-written 4-frame BVH); the 2-D projection discards depth (a bone pointing at the camera holds its last angle).

---

## 11. State machines, data bindings, containers and scroll regions

These commands manage the **runtime-interaction** parts of a project: state machines (`machines`), data bindings
(`bindings`), layout containers (`containers`) and scroll regions (`scrolls`). All of them store their data inside
the project file; the `add` forms read a small side JSON file, validate it and write a new project.
The runtime model itself is documented in [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

### `machine` (add, list, remove, validate)

**Synopsis:**
```
osatura machine add      <project.json> <out.json> <machine.json>
osatura machine list     <project.json>
osatura machine remove   <project.json> <out.json> <name>
osatura machine validate <project.json>
```

A **machine** has named `inputs` (`bool`, `number` or `trigger`) and one or more `layers`; each layer has an `entry`
state, `states` and `transitions`. A state plays a **source**: `{"kind":"segment","start":F,"end":F}` (frames of the
project's keyed channels), `{"kind":"clip","clip":NAME}`, `{"kind":"region","marker":NAME}` or a 1-D blend tree
`{"kind":"blend","input":NAME,"children":[{"threshold":T,"source":{...}}]}`. State fields: `name`, `source`,
`mode` (`loop` default, `once`, `ping_pong`), `speed` (1), `blend_in` (seconds). Transition fields: `from` (a state or
`"*"`), `to`, `when` (a list of `{"input":NAME,"op":eq|ne|lt|le|gt|ge,"value":V}`, all must hold; `{"input":"go"}`
alone means "equals 1"), `exit_time` (0..1), `duration` (crossfade seconds) and `ease`. Unknown keys in a transition
are an **error** (a typo such as `conditions` would otherwise turn a guarded transition into an unconditional one).

```
$ cat mach.json
{"name":"m","inputs":[{"name":"go","kind":"bool"}],"layers":[{"name":"base","entry":"idle","states":[
 {"name":"idle","source":{"kind":"segment","start":0,"end":12}},
 {"name":"act","source":{"kind":"segment","start":12,"end":36}}],
 "transitions":[{"from":"idle","to":"act","when":[{"input":"go"}],"duration":0.1}]}]}
$ osatura machine add demo.json d1.json mach.json
added machine 'm' (1 machines); wrote d1.json
$ osatura machine list d1.json
m  inputs [go:bool]
  layer base (entry idle, mask []): 2 states, 1 transitions
$ osatura machine validate d1.json
1 machines, 0 bindings: ok
```

`machine remove` refuses to remove a machine that a binding still drives
(`binding 'goBind' still drives machine 'm'`, exit 1): remove the binding first.
**Errors:** an invalid machine (unknown state, duplicate names, bad input reference) is rejected with a message and
nothing is written. **Related:** `machine-sim`, `bind`, `state-bake` (the older frame-based machine format).
**Limits:** machines are evaluated by the runtime and by `machine-sim`; the exporters bake none of this (Lottie, GIF
and the other files contain frames only, see [EXPORT_FORMATS.md](EXPORT_FORMATS.md)).

### `machine-sim`

**Synopsis:** `osatura machine-sim <project.json> <script.json> <out_dir|out.json> [--fps N --svg-frames N]`

Replays timed input events against a machine and logs the state of every tick. The script is
`{"machine": NAME?, "duration": SECONDS?, "events": [{"t": SECONDS, "set": {"input": value}, "trigger": ["name"],
"value": {"external.path": number}}]}`. `machine` defaults to the project's only machine, `duration` to the last
event plus one second; `value` feeds data bindings (see `bind`). Default `--fps` is the project fps.
`--svg-frames N` additionally renders N SVG frames spread over the run.

```
$ cat sc.json
{"duration":1.5,"events":[{"t":0.5,"value":{"hud.go":1}}]}
$ osatura machine-sim d3.json sc.json sim.json --fps 24 --svg-frames 0
   0.000s  base     Entered idle
   0.500s  base     Exited idle
   0.500s  base     Entered act
37 ticks at 24 fps, 3 events, 0 svg frames; wrote sim.json
$ osatura machine-sim d3.json sc.json simdir --svg-frames 2     # a name that does not end in .json is a folder
...
37 ticks at 24 fps, 3 events, 2 svg frames; wrote simdir/sim.json
$ ls simdir
frame_00000.svg  frame_00036.svg  sim.json
```
`sim.json` holds `machine`, `fps`, `ticks[]` (`tick`, `time`, `states` as `[layer,state]` pairs, `fading`) and the
event log. **Related:** `bind`, `machine`.

### `state-bake`

**Synopsis:** `osatura state-bake <project.json> <machine.json> <events.json> <out.json> [--tol 0.01]`

Bakes a **frame-based** state machine (a different, older format than `machine`) into ordinary keyed channels.
`machine.json` is `{"initial":NAME,"states":[{"name","length","looped","channels":[...]}],"transitions":[{"from":NAME
or "*","to","when":COND,"fade":FRAMES,"ease"}]}` where `COND` is one of `{"type":"event","name"}`,
`{"type":"above"|"below"|"equals","input","value"}`, `{"type":"finished"}`, `{"type":"after","frames"}`.
`events.json` is a list of `{"frame":N,"name":NAME,"value":V}`. The result is appended to the project's channels
(`--tol` is the key-reduction tolerance), the project is compiled and written.

```
$ echo '[{"frame":10,"name":"go"}]' > ev.json
$ osatura state-bake demo.json sm.json ev.json baked.json --tol 0.01
wrote baked.json (+1 channels)
```
**Gotcha:** the baked channel is appended after the existing ones; channels run in list order, so remove or mute the
original channel for the same target if the bake should win. A machine file in the `machine` format
(with `layers`) is refused: `invalid JSON in mach.json: missing field 'initial'`.

### `bind` (add, list, remove) and `bind-eval`

**Synopsis:**
```
osatura bind add    <project.json> <out.json> <name> <value.path> <target:prop|@machine:input>
                     [--expr E --remap a,b,c,d --no-clamp --smooth S --mode replace|add|multiply --weight W]
osatura bind list   <project.json>
osatura bind remove <project.json> <out.json> <name>
osatura bind-eval   <project.json> <values.json> [--data out.json]
```

A **data binding** maps an external value path (for example `hud.size`) to a project property (`sun:layer_opacity`,
property names are the snake_case ones of section 1) or to a machine input (`@m:go`). `--remap a,b,c,d` maps the
input range `a..b` onto `c..d` (clamped unless `--no-clamp`); `--expr` is an expression over the value;
`--smooth S` low-passes over S seconds; `--mode` and `--weight` control how the result combines with the channels.

```
$ osatura bind add d1.json d2.json goBind hud.go @m:go
added binding 'goBind' (1 bindings); wrote d2.json
$ osatura bind add d2.json d3.json sz hud.size sun:layer_opacity --remap 0,100,0.2,1 --smooth 0.1
added binding 'sz' (2 bindings); wrote d3.json
$ osatura bind list d3.json
goBind  hud.go -> @m:go
sz  hud.size -> sun:LayerOpacity remap 0..100 -> 0.2..1 clamp smooth 0.1s
$ echo '{"hud.go":1,"hud.size":50}' > vals.json
$ osatura bind-eval d3.json vals.json --data bv.json
goBind = 1
sz = 0.6000000238418579
$ osatura bind add d2.json x.json sz hud.size sun:scale
unknown property 'scale'                                                  [exit 1]
$ osatura bind remove d3.json d7.json sz
removed binding 'sz'
```
`bind-eval` prints each binding's output for the given values (`--data` also writes them as JSON). `list` prints property
names in CamelCase; the command line always takes snake_case. **Related:** `machine`, `machine-sim` (its `value` map).

### `container` (add, list, remove, solve)

**Synopsis:**
```
osatura container add    <project.json> <out.json> <container.json>
osatura container list   <project.json>
osatura container remove <project.json> <out.json> <id>
osatura container solve  <project.json> [--size WxH --data out.json]
```

A **container** is a flexbox-like layout box: `id`, `members`, `axis` (`row`|`column`), `gap`, `cross_gap`, `padding`
(`[left,top,right,bottom]`), `justify` (`start|center|end|space_between|space_around`), `align`
(`start|center|end|stretch`), `wrap` (`none|wrap`) and `rect`, a rule for its own rectangle on the canvas:
`{"kind":"fixed",...}`, `{"kind":"pin","left","right","top","bottom","width","height"}` (distances from the canvas edges; any two
of each axis), `{"kind":"aspect",...}` or `{"kind":"hug"}`. A **member** is exactly one of `{"layer":PATH}`,
`{"container":ID}` or `{"spacer":true}`, with `grow`, `shrink`, `width`, `height`, `min_w/max_w/min_h/max_h` and
an `align` override; `"resize":true` lets the solver scale the layer to its slot.

```
$ cat cont.json
{"id":"row1","axis":"row","gap":10,"padding":[10,10,10,10],
 "rect":{"kind":"pin","left":10,"right":10,"top":10,"height":100},
 "members":[{"layer":"sun"},{"layer":"blush","grow":1,"resize":true}]}
$ osatura container add d3.json d4.json cont.json
added container 'row1' (1 containers); wrote d4.json
$ osatura container list d4.json
row1: Row, 2 members, gap 10, justify Start, align Start, wrap None, rect {"kind":"pin","left":10.0,...}
$ osatura container solve d4.json --size 800x480 --data solved.json
canvas 800x480
container row1: x 10.00 y 10.00 w 780.00 h 100.00
layer sun: dx -430.000 dy 0.000 scale 1.0000 x 1.0000
layer blush: dx 280.432 dy -158.242 scale 33.8889 x 1.0000
```
Without `--size` the project's own size is used. `solve` prints each container rectangle and, per layer, the offset and
scale the layout applies on top of the animation. **Errors:** an unknown layer, a missing nested container or a
cycle is rejected by `add` (nothing written). **Related:** `scroll`, `layout-preview`.
**Limits:** the layout is applied by the runtime/preview; exported files hold the layout solved for the project's own size.

### `scroll` (add, list, remove, eval)

**Synopsis:**
```
osatura scroll add    <project.json> <out.json> <scroll.json>
osatura scroll list   <project.json>
osatura scroll remove <project.json> <out.json> <id>
osatura scroll eval   <project.json> <id> [--offset X,Y --size WxH --velocity VX,VY --time S]
```

A **scroll region** has `id`, `content` (the id of a container that is not a member of another container), `viewport`
(a rectangle rule as above), `axis` (`x`, `y` or both), `offset`, `elastic` (`{"limit","stiffness","damping_ratio"}`; rubber
band instead of a hard clamp), `snap_x`/`snap_y` (resting offsets), `fling` (seconds of release velocity projected to find
the landing point, default 0.15), `clip_layer` (a rectangle layer used as a mask for the content) and `input` (an external value path that
drives the offset).

```
$ cat scroll.json
{"id":"sc1","content":"row1","axis":"x","viewport":{"kind":"pin","left":0,"right":0,"top":0,"height":120},
 "elastic":{"limit":60,"stiffness":200,"damping_ratio":0.8},"snap_x":[0,100,200]}
$ osatura scroll add d4.json d5.json scroll.json
added scroll 'sc1' (1 scrolls); wrote d5.json
$ osatura scroll list d5.json
sc1: content row1, axis X, offset [0.0, 0.0], elastic true, snap x [0.0, 100.0, 200.0] y [], clip -
$ osatura scroll eval d5.json sc1 --offset 400,0
viewport [0.0, 0.0, 640.0, 120.0] content [-59.92364, 0.0, 640.0, 120.0] max [0.0, 0.0]
requested offset [400.0, 0.0] -> applied [59.92364, 0.0]
$ osatura scroll eval d5.json sc1 --offset 80,0 --velocity -500,0 --time 0.5
...
requested offset [80.0, 0.0] -> applied [44.184174, 0.0]
t 0.100s offset [22.794, 0.000]
t 0.300s offset [0.051, 0.000]
t 0.500s offset [-0.197, 0.000]
```
Without `--velocity`, `eval` shows the clamped (or rubber-banded) offset for the requested one. With `--velocity`, it
simulates the release (momentum, snap, spring) for `--time` seconds and prints every 0.1 s. `--size WxH` evaluates for another canvas size.
In the example the content is as wide as the viewport, so everything past 0 is overscroll and shows the rubber band.

### `layout-preview`

**Synopsis:** `osatura layout-preview <project.json> <out.svg> --size WxH [--frame N --scroll id=X,Y]`

Renders one frame with the layout **re-solved for another canvas size** and the container outlines drawn, so you can
check how a design reflows on a phone or a wide screen. `--scroll id=X,Y` sets a scroll offset (repeat for several regions).

```
$ osatura layout-preview d5.json lp.svg --size 800x480 --scroll sc1=50,0
wrote lp.svg (800x480, 1 containers, 2 layers placed)
```

## 12. Effects, depth of field, onion skin, trails and camera

### `fx`

**Synopsis:** `osatura fx <project.json> <frame> <chain.json> <out.png> [--scale N] [--layer-normals] [--clamp]`

A `<frame>` outside the project is an error; `--clamp` renders the nearest valid frame.

Renders a frame and runs a **post-effects chain** over it. `chain.json` is a list of effects, each with a `type`:
`blur {radius}`, `drop_shadow {dx,dy,blur,color,opacity}`, `glow {radius,strength,color?}`, `outline {width,color}`,
`posterize {levels}`, `pixelate {size}`, `levels {in_black,in_white,gamma,out_black?,out_white?}`, `fade {amount}` and
`relight {ambient,bevel,height,lights:[{x,y,z,color?,intensity?,radius?,cast_shadow?,spot?,falloff?}],ground_shadow?,
normal_map?,ambient_color?}`. Without a `normal_map` the relight shades a bevel made from the blurred alpha channel.

```
$ cat chain.json
[{"type":"drop_shadow","dx":6,"dy":6,"blur":3,"color":"#000000","opacity":0.5},{"type":"glow","radius":6,"strength":0.5}]
$ osatura fx demo.json 12 chain.json fx.png --scale 0.5
wrote fx.png (2 effect(s))
$ cat relight.json
[{"type":"relight","ambient":0.4,"bevel":6,"height":8,"lights":[{"x":300,"y":100,"z":120,"intensity":1.0}],"ground_shadow":0.3}]
$ osatura fx demo.json 12 relight.json lit.png --scale 0.5
wrote lit.png (1 effect(s))
```
`--layer-normals` shades every layer with its own normal map (see `layer-normal`) instead of one map for the whole picture.
A light that omits `z` is rejected (`missing field 'z'`). **Limits:** the chain is a PNG-only still pipeline; none of
these effects reach Lottie, GIF or the HTML player (the Lottie exporter has its own effect subset, see EXPORT_FORMATS.md).

### `color-space`

**Synopsis:** `osatura color-space <project.json> <out.json> srgb|linear`

Sets the project's `color_space`, the space in which the raster compositor (PNG, GIF, APNG, video, sprite sheet, `compare`, `ci`) and the float compositor (`render-hdr`) blend. `srgb` (the default, and what every golden was recorded with) blends the stored sRGB-encoded values; `linear` converts every colour (fills, strokes, gradient stops, background, image pixels) to linear light, composites there in f32, and converts to sRGB once at the end. A 50 % black layer over white then gives 188/255 instead of 128/255, and gradients interpolate in linear light. With `srgb` the key is not written, so existing files and goldens do not change.

```
$ osatura color-space demo.json lin.json linear
wrote lin.json (color_space: linear)
$ osatura render lin.json 0 lin.png
wrote lin.png
$ osatura color-space lin.json x.json p3
unknown colour space 'p3' (srgb or linear)
```
**Limits:** the 8-bit renderer switches to the float compositor for linear projects (own coverage rasteriser, so anti-aliased edges differ slightly from the sRGB output); layer-effect colours (shadow, glow, tint) are used as written, not converted; `dof` and the post-effect chain (`fx`) stay sRGB; SVG, Lottie and the GPU renderer ignore the setting; image resampling happens in sRGB before the conversion. Blend modes are applied to the linear values.

### `layer-normal`

**Synopsis:** `osatura layer-normal <project.json> <out.json> <layer> <normal.png|none> [--strength S] [--fit stretch|tile|exact] [--green-down]`

Attaches (or, with `none`, removes) a tangent-space normal map on an image layer for `fx ... --layer-normals`.
`--fit` says how a map of another size is fitted (`stretch` default, `tile`, `exact`); `--green-down` flips the green
channel (DirectX-style maps); `--strength` scales the tilt (1 = as stored, 0 = flat).

```
$ osatura layer-normal demo.json dn.json sun n.png --strength 1 --fit stretch
wrote dn.json
$ osatura fx dn.json 12 chain.json fxn.png --scale 0.5 --layer-normals
wrote fxn.png (2 effect(s))
$ osatura layer-normal dn.json dn2.json sun none
wrote dn2.json
```

### `onion` (two forms)

**Synopsis:**
```
osatura onion <project.json> <frame> <out.png> [--before 2 --after 1 --step 1 --scale N]
osatura onion <project.json> <out.svg> --frame N [--before B --after A --step S --keyed --outline --all-parts
               --layers a,b --tint-before C --tint-after C --opacity X --falloff X --ghosts-only]
```
The first form writes a PNG with faded ghosts of the frames before and after. The second writes an SVG still: the
frame with tinted onion-skin ghosts over it. `--keyed` uses the nearest **key frames** instead of fixed steps;
`--outline` draws ghosts as outlines; `--all-parts` ghosts every part (default: moving parts), `--layers` limits them,
`--ghosts-only` omits the frame itself.

```
$ osatura onion demo.json 12 on.png --before 2 --after 1 --step 2 --scale 0.5
wrote on.png
$ osatura onion demo.json on.svg --frame 12 --before 2 --after 2 --step 3 --outline --opacity 0.4
wrote on.svg (4 ghost(s) around frame 12)
$ osatura onion demo.json on2.svg --frame 12 --keyed --ghosts-only
wrote on2.svg (3 ghost(s) around frame 12)
```
The form is chosen by the second argument: a number means the PNG form, a name ending in `.svg` the SVG form.

### `trail` (two forms)

**Synopsis:**
```
osatura trail <project.json> <bone> <out.svg> [--from F --to F]
osatura trail <project.json> [out.svg] --target layer:PATH|bone:NAME[:head|tip]|point:LAYER:KIND:A[:B] [--from F --to F --smooth N --json]
```
Draws the **motion trail** of a bone tip (first form) or of any target (second form: a layer, a bone head/tip or a
deformer point), with a tick per frame and markers at keys. `--smooth N` smooths the curve; `--json` prints the sampled
curve as JSON (and then `out.svg` is optional).

```
$ osatura trail demo.json hand tr.svg --from 0 --to 24
wrote tr.svg (25 points)
$ osatura trail demo.json tr3.svg --target layer:sun --from 0 --to 24
wrote tr3.svg (25 ticks, 0 key marker(s))
$ osatura trail demo.json tr2.svg --target bone:hand:tip --from 0 --to 24 --smooth 2 --json
{ "curve": [ [212.8, 83.84], [213.77, 83.99], ...
```

### `dof` and `dof-export`

**Synopsis:**
```
osatura dof        <project.json> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24 --scale N --clamp]
osatura dof-export <project.json> <out.gif|out.apng|out.mp4|out.webm|out-folder> [--from F --to F --focus N --strength N --max-radius N --scale N --codec vp9|av1]
```
A `<frame>` outside the project is an error for `dof` (and for `render-hdr` and `fx`); `--clamp` renders the nearest valid frame instead, as for `render`.

Depth of field: each **top-level layer** is blurred by `|parallax - focus|` times `strength` (pixels per unit of
parallax distance, capped at `--max-radius`). Without `--focus` / `--strength` the values follow the keyed
`camera_focus` / `camera_blur` channels (default focus 1.0). Layers get their depth from `parallax` (see section 5).
`dof-export` renders the range `--from`..`--to` (whole frame numbers; default: from 0 to the end) to an animated GIF, an
animated PNG, an `.mp4`/`.webm` video (needs `ffmpeg`; `--codec` defaults to `vp9`; no audio is included) or, for a name with
no extension, a folder of numbered PNGs (`f0000.png`...). Any other extension is refused (`dof-export writes .gif, .apng,
.mp4, .webm or a folder (no extension)`, nothing written).

```
$ osatura dof demo.json 12 dof.png --focus 1.0 --strength 12 --max-radius 24 --scale 0.5
wrote dof.png
$ osatura dof-export demo.json dof.gif --from 0 --to 6 --scale 0.25
wrote dof.gif
$ osatura dof-export demo.json dofdir --from 0 --to 2 --scale 0.25      # dofdir/f0000.png ... f0002.png
wrote dofdir
```
**Related:** `parallax`, `render --proxy`, `shot`/`camera-seq` for camera moves. **Limits:** the blur is a
per-layer approximation, not a lens model (no bokeh shapes, no occlusion handling); it is not exported to Lottie.

## 13. Level of detail, size reports and export-fit

### `size-report`

**Synopsis:** `osatura size-report <project.json> [--tol T] [--json]`

Shows **where the bytes are**: the project file, the baked Lottie and the HTML page, by layer, image and channel
(keys, and how many are removable at tolerance `--tol`, default 0.05).

```
$ osatura size-report small.json
project file: 17959 bytes (3511 gzip)
  by layer (JSON bytes; embedded image base64 in brackets):
         2983  arm
         2053  leg
  ...
  by channel (keys, removable at tol 0.05):
          420  foot_ctrl.BoneOffsetX  keys 5 removable 1
  ...
baked Lottie: 46843 bytes (11180 gzip), by layer:
         9181  arm
```
`--json` prints the same data as JSON (`html.defs_bytes_total`, `frame_bytes_max`, ...). Use it before `export-fit` or `lod`
to see which layer to simplify.

### `lod`

**Synopsis:** `osatura lod <project.json> --tolerance-px 0.5 [--scale S | --size WxH] [--levels 4] [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]`

Builds **levels of detail** L1..Ln, each with a *declared error bound in output pixels*: path simplification, key
reduction, mesh collapse and culling. Level k has tolerance `--tolerance-px x ratio^(k-1)`. `--scale` or `--size` says
how big the output is shown (default 1 px per unit). `--out-dir` (default `.`) receives `<name>_L1.json` ... and
`--report` a JSON report. `--verify` renders `--frames N` frames and **measures** the real deviation (exit 1 if above
the bound).

```
$ osatura lod small.json --tolerance-px 1 --levels 3 --out-dir lv --report rep.json --verify --frames 4
level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  path  mesh  keys  cull  measured_px  ratio
L0      0.000       74       0    37      14    17959       0.0000 ...                              -      -
L1      1.000       70       0    37      14    17651       0.3861 0.386 0.000 0.000 0.000       0.3342   1.2x
L2      2.000       69       0    37      14    17573       0.9277 ...                           0.8185   1.1x
L3      4.000       65       0    37      14    17251       1.5372 ...                           1.3367   1.1x
scale 1 px per unit; level k tolerance = 1 px x 2^(k-1); files lv/s_L1..L3.json
  L1: keys: 3 of 8 keyed channels have a documented lever, 0 reduced
  L1: removed 4 path anchor(s)
$ ls lv
s_L1.json  s_L2.json  s_L3.json
```
**Gotcha:** the savings on a small, hand-built demo are tiny (about 4% at 4 px); LODs pay off on traced or mesh-heavy art.
The report and level files are written with a plain (non-atomic) write.

### `lod-pick`

**Synopsis:** `osatura lod-pick <project.json> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--out chosen.json]`

Chooses the **coarsest level** whose declared bound stays within `--max-px` pixels when the artwork is shown on a
screen of `--screen-px` (the scale is screen size over project size). `--out` writes that level.

```
$ osatura lod-pick small.json --screen-px 320x240 --out chosen.json
level  tol_px  anchors ...
screen 320x240 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.4638 px, 69 anchors, 0 mesh vertices, 37 keys)
wrote chosen.json
$ osatura lod-pick small.json --screen-px 64x48 --max-px 2 --out chosen2.json
screen 64x48 -> scale 0.1000 px/unit; budget 2 px -> level L4 (declared 0.5310 px, 62 anchors, 0 mesh vertices, 37 keys)
```

### `export-fit`

**Synopsis:** `osatura export-fit <project.json> <out> --max-size 500KB [--format lottie|html|gif|apng|webm|project] [--min-ssim X] [--max-seconds 60] [--measure raw|gzip] [--frames N] [--search ladder|descent|grid] [--report out.json] [--pareto out.csv] [--audit] [--no-baseline]`

Finds the **best-quality export that fits a byte budget**: it searches the exporter's quality knobs (Lottie: `decimals`,
`path_eps`, `key_tol`, `frame_step`; other formats have their own) and scores each candidate with SSIM against the
original render. `--max-size` takes `500KB` (1000 bytes), `1.5MB`, `64000`, `200KiB` (1024 bytes). The format comes from
`--format` or the output extension (`.json` is Lottie, `.html`, `.gif`, `.apng`, `.webm`); `project` writes a slimmed
project JSON. `--measure gzip` budgets the gzip size (for servers that compress). `--min-ssim` is a quality floor.
`--max-seconds` is a **real wall-clock budget** (default 60; `0` = unlimited): on expiry the best result so far is kept.
`--frames N` is how many frames are sampled for the SSIM (default 8). `--search` picks the strategy (`descent` default).
`--report` / `--pareto` write the search record as JSON / CSV; `--audit` prints every candidate; `--no-baseline` skips
the default-settings comparison.

```
$ osatura export-fit small.json fit.json --max-size 30KB --format lottie --report fr.json
lottie -> fit.json: 25756 bytes raw, 5736 gzip (budget 30000 raw); SSIM mean 0.9699, worst 0.9308 at frame 5, max CIEDE2000 68.67; 5 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (...)
$ osatura export-fit small.json fit.html --max-size 60KB --format html --max-seconds 20 --frames 4
html -> fit.html: 59166 bytes raw, 8550 gzip (budget 60000 raw); SSIM mean 1.0000, worst 1.0000 at frame 5, max CIEDE2000 8.54; 1 candidates built
existing exporter at defaults: 66391 bytes raw, 8933 gzip
$ osatura export-fit small.json fit2.json --max-size 1KB --format lottie --max-seconds 10 --frames 3
lottie -> fit2.json: 14751 bytes raw, 2418 gzip (budget 1000 raw); ...
knobs at their floor: decimals, path_eps, key_tol, frame_step
budget not met: the smallest export found is 14751 bytes (raw), above the budget of 1000 bytes: ...          [exit 1]
$ osatura export-fit small.json fit.xyz --max-size 1KB
cannot tell the format from the output name; pass --format lottie|html|gif|apng|webm|project   [exit 1]
```
**Exit codes:** 0 when the result fits the budget (even if the time budget cut the search short); 1 when the budget
cannot be met (the smallest export is still written) or on bad arguments. **Gotchas:** it works on the baked output,
so it cannot remove more than the knobs allow (a 1 KB Lottie is impossible for a 14-layer rig); every candidate is a full
export, so large projects need a generous `--max-seconds` (a debug build is far slower than a release build). The report
files are written with a plain write. **Related:** `size-report`, `lod`, `export` (section 4), EXPORT_FORMATS.md (size tips).

## 14. Quality assurance

Related, documented earlier: `check` (validation, section 3), `diff` (semantic diff, section 3), `compare` (image diff of two
renders, section 4).

### `lint`

**Synopsis:** `osatura lint <project.json>`

Static rig and animation checks. Prints one line per finding: `Severity CODE [target:property] message`. Exit 1 if any
finding has severity Error or above; Info and Warning exit 0.
```
$ osatura lint small.json
Info L007 [:CameraX] key at frame 24 lies outside the 0..8 timeline
Info L007 [arm_low:BoneAngle] key at frame 12 lies outside the 0..8 timeline
```
`L007` is a key outside the timeline (here because `small.json` is the demo cut to 8 frames).

### `contracts`

**Synopsis:** `osatura contracts <project.json> <contracts.json>`

**Motion contracts**: unit tests for animation. `contracts.json` is a list of contracts with a `type`:
`loop_seamless {tol}`, `tip_range {bone,axis,min,max}`, `max_speed {bone,px_per_frame}` (further types exist in the
engine's `Contract` enum). Violations print on stdout, the count on stderr, and the exit code is 1.
```
$ cat contracts.json
[{"type":"loop_seamless","tol":0.5},{"type":"tip_range","bone":"hand","axis":"y","min":0,"max":400},{"type":"max_speed","bone":"hand","px_per_frame":3}]
$ osatura contracts small.json contracts.json
contract 0 violated at frame 8: bone 'arm_up' ends 35.4 px away from where it started (tolerance 0.5)
...
contract 2 violated at frame 6: tip of 'hand' moves 3.3 px/frame (limit 3)
7 violation(s)                                                            [stderr, exit 1]
```

### `coach`

**Synopsis:** `osatura coach <project.json>`

Timing and animation-principle suggestions (overlap, anticipation, easing). Advice only, always exit 0.
```
$ osatura coach small.json
C003 [, arm_low, arm_up, foot_ctrl, hand, mouth, target] frame 0: 7 parts start moving on frame 0; offset them by a few frames (overlap) for a more natural feel
```

### `safety`

**Synopsis:** `osatura safety <project.json> [--scale N]`

Flash, red-flash and camera-motion report (an approximation of WCAG 2.3.1; an engineering aid, not a broadcast
certification). Renders every frame at `--scale` (use a small value for long projects).
```
$ osatura safety small.json --scale 0.25
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.06 widths/s, rotation 0°/s, zoom ×1.00/s
```

### `reduced-motion`

**Synopsis:** `osatura reduced-motion <project.json> <out.json> [--factor 0.25]`

Writes a calmer copy: camera motion and wobble are scaled by `--factor` (default 0.25).
```
$ osatura reduced-motion small.json calm.json --factor 0.25
wrote calm.json
```

### `golden`

**Synopsis:**
```
osatura golden write <project.json> <golden.json> [--scale N]
osatura golden check <project.json> <golden.json> [--tol T]
```
`write --scale N` takes N in (0, 1]: a larger value is **refused** (`golden write: --scale 2 is above 1 ...`, nothing is written; it used to be clamped to 1 silently). `check` re-renders at the scale stored in the golden file and has no `--scale` option: `golden check ... --scale N` is refused as an unknown option.


A lightweight regression test: `write` records a hash and a 16x16 thumbnail of every frame into `golden.json`; `check`
re-renders and compares. **Exact by default** (`--tol 0`): any difference of the pixel hash fails. `--tol T` (T > 0) loosens
the check: a changed hash still passes when the thumbnail distance is at most T. A negative or non-finite `--tol` is an error.
For image-based goldens with a perceptual metric, use `ci`.
```
$ osatura golden write small.json g.json --scale 0.25
wrote g.json (8 frames)
$ osatura golden check small.json g.json
all 8 frames match exactly
$ osatura golden check small.json g.json --tol 0.01
all 8 frames match (within thumbnail tolerance 0.01)
```
Differing frames are listed (hash change, thumbnail distance, number of the 256 thumbnail cells that differ, largest channel
difference out of 255), and the exit code is 1:
```
$ osatura golden check changed.json g.json
frame 1 differs: pixel hash changed, thumbnail distance 0.003, 12 of 256 thumbnail cells differ, max channel difference 31/255
2 frame(s) differ (exact comparison; --tol T accepts small differences)       [stderr, exit 1]
```
A frame present on only one side is reported as `one side only`. **Limits:** the golden file keeps only the hash and the
thumbnail, so the reported numbers are thumbnail-level, not pixel-level. A change that is invisible in the render (for example
a background colour completely covered by an opaque layer, as the demo's sky does) legitimately leaves the hash unchanged.

### `ci` (init, run, bless, diff)

**Synopsis:**
```
osatura ci init <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run  [manifest] [--update] [--bless] [--prune] [--only a,b] [--junit f.xml] [--html f.html] [--json f.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest]
osatura ci diff a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--ignore x,y,w,h;...] [--aa N] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```
**Animation CI**: golden frames compared with a perceptual metric (luma SSIM, CIEDE2000), plus structural checks and
budgets. The manifest format and the workflow are described in [CI.md](CI.md). `exit 0` = pass, `1` = a check failed,
`2` = error (unreadable manifest, ...).
```
$ osatura ci init small.json --out ci.json
wrote ci.json with 8 checks:
  validate     validate
  render       frames
  render       frames-float
  determinism  determinism
  fuzz         pose-fuzz
  limits       limits
  export-size  gif-size
$ osatura ci run ci.json --junit j.xml --html r.html --json ci_r.json
[PASS   ] validate     validate           project compiles; 8 lint finding(s), none at 'error' or above
[PASS   ] render       frames             6 frame(s) match
[PASS   ] determinism  determinism        6 render(s) repeated byte-identically
[INFO   ] fuzz         pose-fuzz          98 of 100 pose samples fail (98.00%); not a gate: non-gating (gate = false)
[PASS   ] export-size  gif-size           gif export is 82881 bytes (budget 124322)
[INFO   ] perf         render-speed       median 429.67 ms/frame, budget 1633 ms (+50% tolerance = 2449.50 ms)
8 checks: 6 passed, 0 failed, 0 errors, 2 info, 0 updated, 0 blessed (14.7 s)
$ osatura ci diff a.png b.png --metric ssim --out heat.png
size        480x360
SSIM        0.90536   (luma)   channels min 0.90300   (R, G, B)
de2000      max 68.674   mean 1.9434
pixels      11124 differ (6.4375%), 31 forgiven as anti-aliasing, 0 ignored
regions     118
images differ: luma SSIM 0.90536 < 0.99; ...                                [stderr, exit 1]
$ osatura ci run nonexist.json
cannot read manifest nonexist.json: No such file or directory               [exit 2]
```
`init` writes the manifest and, unless `--no-goldens`, the golden PNGs into the `goldens/` folder next to it. `run
--update` rewrites goldens that are missing or changed, `--bless` accepts the new renders as goldens, `--prune` removes
golden files that no check references any more, `--only a,b` runs the named checks. `bless` is `run --bless`. `diff` compares two PNGs: `--tol` is the metric
threshold, `--ignore` masks rectangles (`x,y,w,h;...`), `--aa N` forgives anti-aliasing differences, and `--out`,
`--side`, `--overlay`, `--flicker` write a heat map, a side-by-side image, an overlay and a flicker GIF.
**Gotcha:** the reports (`--junit`, `--html`, `--json`) and golden files are written with the atomic, symlink-refusing writer (creating
parent folders); a symlinked report path is refused after the checks ran (the checks are slow, so pass a real path). A debug build is slow: the 8 checks above took 15 s on an 8-frame project.

### `fuzz`

**Synopsis:** `osatura fuzz <project.json> [--samples N] [--seed S] [--strategy random|lhs|halton|sweep|corners|pca] [--frame F] [--report out.json] [--heatmap out.svg] [--minimize] [--gate] [--max-fail-rate R] [--props angle,scale,offx,offy] [--only BONES] [--margin DEG] [--db posedb.json] [--ignore KINDS] [--no-deformers]`

**Pose fuzzing**: samples poses within the bone limits and reports where the rig breaks (folds, stretch, spikes,
self-intersection, IK unreachable, limit violations, NaN, off-canvas). `--strategy` picks the sampler (`pca` samples along the PCA axes of the project's own key poses, or of the pose database
given with `--db`), `--minimize` shrinks a failing pose to a minimal reproducer, `--heatmap` writes a stress map,
`--ignore` skips failure kinds, `--only` restricts to bones. Without a gate, `fuzz` is a **report**: it prints the summary and exits 0 (`(report only: exit 0; add --gate, ...)`). `--gate` fails (exit 1) when any sample fails; `--max-fail-rate R` (a **fraction** 0..1) implies gating at that rate. (`ci`'s own `fuzz` check has its own `gate` setting and is unaffected.)
```
$ osatura fuzz small.json --samples 30 --seed 1 --strategy lhs --report fz.json --heatmap hm.svg --max-fail-rate 1.0
wrote fz.json
wrote hm.svg
fuzz 'small': 30 samples (lhs, seed 1, frame 0), 25 parameters, 30 failures (100.0%), max severity 54.62, baseline 0.00, 0.40 ms/sample
  ik_unreachable: 27
  off_canvas: 1
  stretch: 2
$ osatura fuzz small.json --samples 10 --max-fail-rate 0.5
fuzz: failure rate 100.00% exceeds --max-fail-rate 50.00%                   [stderr, exit 1]
```
The demo's IK target is deliberately reachable only in a small range, so almost every random pose is "unreachable":
treat the rate as a regression number, not as a pass/fail on this rig. The report and heatmap are written atomically and a symlinked path is refused (unless `--follow-symlinks`); the parent folder must exist.

### `fuzz-grid`

**Synopsis:** `osatura fuzz-grid <project.json> --x PARAM --y PARAM [--res N] out.svg|out.png|out.json`

A severity map of two parameters (`bone:NAME:PROP`, for example `bone:arm_up:angle`) on an `N x N` grid (default 32).
The output type follows the extension.
```
$ osatura fuzz-grid small.json --x bone:arm_up:angle --y bone:arm_low:angle --res 8 grid.svg
wrote grid.svg (8x8 cells, 59.4% failing; x = bone:arm_up:angle, y = bone:arm_low:angle)
```
Same plain-write caveat as `fuzz`.

## 15. Version control (`vcs`)

**Synopsis:**
```
osatura vcs init <project.json> [--author A]
osatura vcs commit [-m MSG | --message MSG] [--author A] [--repo DIR]
osatura vcs log [REV] [--limit N] [--json] [--repo DIR]
osatura vcs status [--json] [--repo DIR]
osatura vcs diff [A [B]] [--json] [--repo DIR]
osatura vcs show [REV] [--json] [--repo DIR]
osatura vcs branch [NAME [AT] | -d NAME] [--repo DIR]
osatura vcs tag [NAME [AT]] [--repo DIR]
osatura vcs checkout REV [--force] [--repo DIR]
osatura vcs merge REV [--no-ff] [-m MSG | --message MSG] [--author A] [--abort] [--repo DIR]
osatura vcs cherry-pick|revert REV [--author A] [--repo DIR]
osatura vcs resolve PATH [--ours] [--theirs] [--value V] [--repo DIR]
osatura vcs fsck|gc [--repo DIR]
```
Options are per sub-command: `--json` only on `log`, `status`, `diff` and `show`; `--force` only on `checkout`; `--ours`, `--theirs`, `--value` (exactly one) only on `resolve`.

A small content-addressed history for **one project file** with a **semantic merge**: changes are compared as
document paths (JSON pointers), not as text lines, so two people editing different keys never conflict. `init` creates
a `.osatura-vcs/` folder next to the project; the default branch is `main`. Revisions (`REV`) are branch names, tag
names or commit-id prefixes.

```
$ osatura vcs init p.json
initialised ./.osatura-vcs for p.json (no commits yet; run: osatura vcs commit -m "...")
$ osatura vcs commit -m first --author alexio
[5f80a63179] first
$ (edit p.json)
$ osatura vcs status
on branch main
HEAD 5f80a63179
uncommitted changes:
  project: background changed "#f6f1e7" -> "#ffffff"
$ osatura vcs commit -m "white bg"
[29c09d9368] white bg
$ osatura vcs branch alt          # branch alt at 29c09d9368
$ osatura vcs checkout alt        # checked out alt (29c09d9368)
$ osatura vcs log --limit 5
4e21f15940  unknown  30 fps
29c09d9368  unknown  white bg
5f80a63179  alexio  first
$ osatura vcs merge alt           # fast-forward or a merge commit; --no-ff forces a merge commit
merge committed as f08db3f77f
$ osatura vcs tag v1 ; osatura vcs tag
tag v1 at 4e21f15940
v1  4e21f15940
$ osatura vcs fsck
ok: 50 objects, history and refs are consistent
$ osatura vcs gc
gc: kept 50 objects, removed 0 unreachable
```

**Conflicts.** When both sides changed the same value, `merge` stops, keeps *our* side in the project file, lists the
conflicts and exits 1:
```
$ osatura vcs merge main
merge stopped with 1 conflict(s); the project file holds our side for each:
  CONFLICT /background
      project: background: ours "#222222" vs theirs "#333333" (base "#000000")
resolve each with: osatura vcs resolve <path> --ours|--theirs|--value <json>, then: osatura vcs commit   (or: osatura vcs merge --abort)
$ osatura vcs resolve background --theirs
no conflict at 'background' (see 'vcs status')                              [exit 1]   (the path needs its leading slash)
$ osatura vcs resolve /background --theirs
resolved /background; 0 conflict(s) left - run: osatura vcs commit
$ osatura vcs commit -m merged
[bd405f446a] merged
$ osatura vcs merge --abort       # alternatively: merge aborted; project restored to HEAD
```
While a merge, cherry-pick or revert is unresolved, `checkout`, `revert` and `commit` are refused with
`a merge / cherry-pick / revert is in progress: resolve the conflicts and commit, or run 'vcs merge --abort'`.
Other facts: `diff` with no argument compares the working file with HEAD, `diff A B` two revisions (`--json` prints a list
of `{kind,path,description}`); `show` prints a commit and its change list; `branch` with no argument lists branches (`*` marks
the current one); `branch -d NAME` refuses to delete the checked-out branch; `checkout --force` discards uncommitted
changes; `revert REV` and `cherry-pick REV` create a new commit (or stop with conflicts like `merge`); `--repo DIR`
selects a repository folder other than the current one; `--author` defaults to `unknown`; `log --json` prints the commit
objects. **Limits:** one file per repository, local only (no remotes, no push/pull), no rename tracking beyond JSON paths;
binary assets referenced by the project are not versioned.

## 16. Studio, collaboration, MCP agent server and UI registry

### `studio`

**Synopsis:** `osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES] [--public-origin URL] [--source-url URL] [--collab-token T (16+ characters)] [--collab-secret-file F [--collab-revoke NAMES]] [--collab-rate N] [--collab-log LOG [--collab-nosync]]`

Starts the **browser editor** (undo/redo, timeline, live and face panels, VCS and agent-style operations) as a local web
server and blocks until Ctrl-C. Defaults: `--port 8080`, `--bind 127.0.0.1`. The project is validated at start; saves go
to the same file (`POST /api/save`).
```
$ osatura studio demo.json --port 18931 --ui-config ui.json
AGPL-3.0 source offer: http://127.0.0.1:18931/source
OSATURA studio on http://127.0.0.1:18931  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in ui.json
```
Options: `--ui-config F` keeps layout and shortcuts (default `~/.config/osatura/ui.json`, on Windows `%APPDATA%\OSATURA\ui.json`); `--source-url URL` sets the
published-source link shown at `/source` and by `--version` (AGPL-3.0 section 13; the official release binaries default to the commit they were built from); `--allow-host NAMES` and `--public-origin URL` accept
requests under a public name behind a TLS reverse proxy; `--collab-token T` (at least 16 characters, for example
`openssl rand -hex 16`) enables the multi-user session and is **required for any non-loopback `--bind`**; `--collab-nosync` is only valid together with `--collab-log` (otherwise an error); `--collab-log LOG`
persists the shared session and resumes it after a restart (`--collab-nosync` skips the sync at start).

**Hardening options** (wave 7; details and the threat model in [SECURITY.md](SECURITY.md)): `--collab-secret-file F` replaces
the one shared bearer by **per-participant tokens**: F holds a random secret (first line, at least 16 characters, mode 600 on
Unix, otherwise it is refused) and each person gets a token from `osatura collab-token NAME`; a token is bound to one name
(joining uses that name, nobody can join as somebody else), expires, and is checked by recomputing its HMAC-SHA-256, so the
server keeps no table. `--collab-revoke NAMES` (comma separated, needs `--collab-secret-file`) refuses the tokens of those
names before they expire. `--collab-token` and `--collab-secret-file` can be combined (the shared token then acts as an
admin key); either one satisfies the rule that a non-loopback bind needs a credential. `--collab-rate N` limits every client
address to N requests per second (bursts of 2N; answer `429` with `Retry-After`); the default is 100 when a credential is
required and no limit otherwise; 0 turns it off. A non-loopback bind prints warnings (plain HTTP; every interface for
`0.0.0.0` / `::`; a shared token has no names and no expiry).

**Safety rules** (verified): the server answers only requests whose `Host` is one of its own names
(`curl -H 'Host: evil.example' .../api/state` gives 421); `Origin` and `Sec-Fetch-Site` from another page give 403; a POST body
must be `application/json` (`curl -X POST -d '{}'` gives 415). A short token is refused
(`--collab-token must be at least 16 characters`), and `--bind 0.0.0.0` without a token is refused
(`refusing to listen on 0.0.0.0:PORT: the studio has no accounts and no TLS, ...`). The server has no accounts and no TLS: put
a TLS proxy in front for anything but your own machine; when `--public-origin` is an `https://` URL, the login cookie gets the `Secure` attribute for requests that arrive under that name (or with `Origin` https / `X-Forwarded-Proto: https`). The page serves a small SVG favicon at `/favicon.ico`. The HTTP API (for example `GET /api/state`, `GET /api/project`,
`POST /api/op`, `POST /api/save`, `GET /frame/N.svg`) exists for the page itself and is not a stable public API.

### `collab-token`

**Synopsis:** `osatura collab-token <name> [--secret-file F] [--ttl 28800]`

Prints a per-participant collaboration token for a studio started with `--collab-secret-file F`: `ap1.<name>.<expires>.<mac>`,
where `mac` is the hex HMAC-SHA-256 of the name and the deadline under the secret. `<name>` is 1 to 32 characters of letters,
digits, `_` and `-`. `--ttl` is the life in seconds (default 28800 = 8 hours, at most 2592000 = 30 days). The secret is read from
the file `F` (first line) or, without `--secret-file`, from the environment variable `OSATURA_COLLAB_SECRET`; it is never
accepted as an argument, where other users can see it. The token goes to stdout, the deadline to stderr.

```
$ openssl rand -hex 32 > collab.secret && chmod 600 collab.secret
$ osatura studio demo.json --collab-secret-file collab.secret
$ osatura collab-token ana --secret-file collab.secret --ttl 3600
ap1.ana.1790000000.3f0c...
token for 'ana', valid until Unix time 1790000000 (in 3600 s); give it to that person only; ...
$ osatura collab-token ana --secret-file collab.secret --ttl 0
--ttl needs a whole number of seconds (1 to 2592000)
```
The person pastes the token on the studio's login page (it is then kept in an HttpOnly cookie that expires with the token) or sends it as
`X-Collab-Token`. An expired token is answered `401` with the deadline in the message; the participant's name is taken from the
token. **Related:** `studio`.

### `collab-sim`

**Synopsis:** `osatura collab-sim <project.json> [--clients N] [--ops M] [--seed S] [--undo-pct P] [--replays K] [--require-valid] [--raw] [--json]`

A test harness for the collaboration engine: N replicas edit the project concurrently with random delivery order and
duplicates, then it verifies that all of them hold the same document (exit 1 if not). Default edits are valid project
edits (numbers, key values, visibility, own layers); `--raw` uses arbitrary JSON edits (usually not a valid project);
`--require-valid` makes a non-compiling result exit 1 too.
```
$ osatura collab-sim small.json --clients 3 --ops 20 --seed 2 --json
{"buffered":12,"clients":3,"clocks_identical":true,"converged":true,"converged_ok":true,"deliveries":34,"duplicates_dropped":0,"edits":20,"envelopes":17,...}
```
`converged_ok` is convergence alone; `ok` also needs the merged document to compile.

### `mcp`

**Synopsis:** `osatura mcp <project.json>`

An **agent server**: JSON-RPC 2.0 (the Model Context Protocol) over stdin/stdout, one JSON message per line. It loads the
project and exposes the tools `get_project`, `stats`, `lint`, `coach`, `safety`, `apply_ops`, `set_key`, `undo`, `redo`,
`render_frame` and `save`. Nothing is written until `save` is called. Stdout carries only protocol messages.
```
$ printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lint","arguments":{}}}' | osatura mcp demo.json
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"protocolVersion":"2024-11-05","serverInfo":{"name":"osatura","version":"0.1.0"}}}
{"id":2,"jsonrpc":"2.0","result":{"tools":[{"description":"Return the project JSON, or the value at a JSON pointer ...","name":"get_project",...}]}}
{"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"[]","type":"text"}],"isError":false}}
```
The server exits (code 0) when stdin closes.

### `ui`

**Synopsis:**
```
osatura ui commands|shortcuts [--ui-config F] [--json]
osatura ui layout presets|show <name>|validate <file.json>
osatura ui search <query...> [--ui-config F]
```
The studio's **command registry**: `commands` lists every command with its category, title and key bindings;
`shortcuts` lists the effective shortcuts (defaults plus those in `--ui-config`) and any conflicts; `layout presets` lists the
dock-layout presets, `layout show NAME` prints one as JSON and `layout validate FILE` checks a layout file;
`search` ranks commands as the command palette does.
```
$ osatura ui commands
Edit
  edit.undo                    Undo                               global:Ctrl+Z
  edit.redo                    Redo                               global:Ctrl+Shift+Z  global:Ctrl+Y
$ osatura ui layout presets
animate  V0.62(H0.74([*stage] | [*layers selection bones live face json]) | [*timeline])  [valid]
rig      H0.66(V0.84([*stage] | [*timeline (folded)]) | V0.50([*layers bones] | [*selection json]))  hidden: live,face  [valid]
compact  V0.62([*stage] | [*timeline layers selection bones live face json])  [valid]
live     H0.58(V0.70([*stage] | [*timeline]) | [*live face bones selection])  hidden: layers,json  [valid]
$ osatura ui layout validate demo.json
demo.json: unknown field `format`, expected one of `version`, `root`, `floating`, `hidden` at line 2 column 10   [exit 1]
```

## 17. Pose database, suggestions and other helpers

These commands build a **pose database** (bone rotations at the key frames of a project and of every take of a library)
and use its learned structure (PCA axes, optional small network) to suggest poses. It is a statistical aid trained on
*your* poses, not a general model. Their model and report files are written with a plain write.

### `pose-db` (build, info, train-mlp)

**Synopsis:**
```
osatura pose-db build <out.json> [--project P.json] [--library takes.json] [--step N]
osatura pose-db info  <db.json>
osatura pose-db train-mlp <db.json> <out.json|out.bin> [--hidden 64,64] [--epochs N] [--lr R] [--seed S]
```
```
$ osatura pose-db build db.json --project demo.json
wrote db.json: 5 poses of 3 bones in 1 sequence(s)
$ osatura pose-db info db.json
5 poses, 3 bones: arm_up, arm_low, hand
  sequence 0: project (project, 5 poses)
PCA: 2 axes keep 99.2% of the variance (total 950.0 deg^2)
  axis 1: std 27.34 deg, cumulative 78.7%
  axis 2: std 13.95 deg, cumulative 99.2%
typical nearest-pose spacing 14.43 deg, typical step between consecutive poses 35.82 deg
$ osatura pose-db train-mlp db.json mlp.json --hidden 8 --epochs 20 --seed 1
wrote mlp.json: 6 -> [8] -> 3 (epochs 20, loss 1.0189 -> 0.4921)
trained on the library itself (no held-out data here); run pose-bench for held-out accuracy
```
`--project` adds the poses at the project's key frames, `--library` the takes of a take library (`take-record`, section 6),
`--step N` samples every N frames in addition. The small network (plain SGD) is optional; use it with `--model` below.

### `pose-suggest` (between, complete, next)

**Synopsis:**
```
osatura pose-suggest between  <project.json> <db.json|-> --a F1 --b F2 [--t 0.5] [--snap 0.5] [--top 5] [--apply RANK --out out.json] [--json]
osatura pose-suggest complete <project.json> <db.json|-> --frame F [--fix bone=angle,...] [--reach bone=x,y,...] [--free b1,b2] [--prior 1] [--model m.json] [--apply RANK --out out.json]
osatura pose-suggest next     <project.json> <db.json|-> --frame F [--k 8]
```
`-` instead of a database learns from the project itself. `between` proposes an in-between pose of two frames that stays
on the learned manifold (`linear`, `pca`, `manifold` candidates); `complete` finds the most plausible full pose given fixed
angles (`--fix`), tip targets (`--reach`) and free bones; `next` shows what the library did after poses like the one at the
frame. `--apply RANK` (with `--out`) writes the chosen candidate as keys.
```
$ osatura pose-suggest between demo.json db.json --a 0 --b 24 --t 0.5 --top 3
between: 5 library poses, 2 PCA axes (99% of the variance)
 1. linear      score 1.00  plausibility 1.00  key at frame 12.0
      linear blend of the angles
      arm_up=0.0 arm_low=-7.5 hand=-10.0
 3. manifold    score 1.00  plausibility 1.00  key at frame 12.0
      PCA blend pulled toward the library
$ osatura pose-suggest complete demo.json db.json --frame 12 --fix arm_up=-40
 1. pca-ls      score 0.96  plausibility 1.00  constraint error 0.18  key at frame 12.0
      regularised least squares in PCA space (start 1)
      arm_up=-39.8 arm_low=24.9 hand=11.2
 2. library-nn  score 0.25  plausibility 1.00  constraint error 15.00  key at frame 12.0
(scores rank candidates against this library; they are heuristics, not probabilities)
$ osatura pose-suggest next demo.json db.json --frame 12 --k 3
 1. successor   score 0.60  plausibility 1.00  key at frame 24.0
      what followed pose 1 of 'project'
```

### `pose-bench`

**Synopsis:** `osatura pose-bench`

Prints the held-out accuracy of the suggestion methods on a **synthetic gait-like library** (24 training cycles, 6
held-out ones; completion sees 3 of 10 bones; it also trains the small network). Lower is better. It reads no project. It
is **slow in a debug build** (minutes); numbers from a release build:
```
completion  mean pose                                   20.96 deg RMS
completion  nearest library pose                         1.77 deg RMS
completion  PCA least squares                            0.51 deg RMS
completion  MLP                                          1.64 deg RMS
between     linear blend                                 1.83 deg RMS
next        Markov mean                                  0.72 deg RMS
```

### `pose-match`

**Synopsis:** `osatura pose-match <project.json> <params.json> <goals.json> <out.json> [--generations N --seed N]`

Searches (an evolutionary optimiser, default 80 generations, seed 1, valid range 1..100000) for values of a set of
parameters that put bone tips at goal positions in stage space. `params.json` is a list of
`{"target","prop","min","max","start"}`; `goals.json` is a list of `{"bone","at":[x,y],"weight"}`. The result is appended as
channels; the loss is the weighted sum of squared tip errors in px².
```
$ echo '[{"target":"arm_up","prop":"bone_angle","min":-60,"max":40,"start":0},{"target":"arm_low","prop":"bone_angle","min":-60,"max":60,"start":0}]' > params.json
$ echo '[{"bone":"hand","at":[260,200],"weight":1.0}]' > goals.json
$ osatura pose-match demo.json params.json goals.json out.json --generations 30 --seed 3
wrote out.json (loss 3756.156 px²)
```
**Gotcha:** the loss in the example is large because the ranges cannot reach the goal: check the number, not only the
exit code (a result is written even when the match is poor).
## Known doc/behaviour mismatches

Where the program's own usage text (printed by `osatura` with no arguments) or a neighbouring document says one
thing and the program does another, the sections above describe what the program really does. The differences found
while verifying every example, in the order of how likely they are to bite:

1. **Fixed - unknown flags.** Every command with a usage line now rejects undocumented `--words` (with a "did you mean" hint),
   including `check`, `stats`, `lint`, `coach`, `contracts` and `machine list`. `--json`, `--quiet` and `--allow-empty` are
   per-command and refused elsewhere (a breaking change for scripts that passed them everywhere). Since wave 5 the check is **per sub-command** (`ci diff ... --quiet`, `trail <bone> ... --json`, `ui layout presets --json`, `golden write --tol` are refused) and also covers value options. Tests: `osatura-cli/tests/doc_mismatch.rs` (acceptance matrix) and `options_are_validated_per_sub_command` in `main.rs`.
2. **Fixed - plain `fs::write` outputs.** `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, pose files, snapshots,
   `vcs` and the collaboration log use the atomic, symlink-refusing writer; ffmpeg outputs are pre-checked. A table-driven test
   covers about 24 commands. Not covered end to end: `face-train`/`face-eval` (unit test on their writer only).
3. **Fixed - `golden check`** is exact by default; `--tol` loosens it; it prints the differing count and the largest difference.
   The "background colour change passes" claim was a demo artefact (the sky layer covers the background), not a bug.
4. **Fixed - `fuzz` exit code.** Default is report-only (exit 0); `--gate` or `--max-fail-rate` gates. See `fuzz`.
5. **Fixed - `dof-export`** writes `.mp4` and `.webm` (through ffmpeg, with `--codec`, no audio); the usage text now lists them. A name
   with any other extension is refused and nothing is written.
6. **Fixed - `export` `.png`** writes `<name>.atlas.json`, never `<name>.json`; the unknown-format error lists every extension.
7. **Defaults the usage text does not state:** `studio` listens on `127.0.0.1:8080`; `fuzz-grid --res` is 32; `export-fit`
   samples 8 frames, searches with `descent` and stops after 60 s (`--max-seconds 0` = unlimited); `reduced-motion --factor`
   is 0.25; `size-report --tol` is 0.05; `pose-match` runs 80 generations with seed 1; `lod --out-dir` is the current
   folder.
8. **`machine remove` can refuse** (`binding 'goBind' still drives machine 'm'`); the usage text does not mention the
   dependency check. `machine` and `state-bake` read two different machine formats (`layers`/`inputs` versus
   `initial`/`states`); the usage line of `state-bake` has no description and gives no format.
9. **Fixed - `stats`** no longer accepts `--json` (output is always JSON; the usage line says so).
10. **`vcs`:** the default branch is `main` (`checkout master` fails with `unknown revision 'master'`); `vcs resolve`
    takes the conflict path *with* its leading slash exactly as `merge` prints it (`/background`), `background` alone gives
    `no conflict at 'background'`; the usage text writes "see: osatura vcs" but `osatura vcs` without arguments prints a
    one-line usage and exits 1.
11. **Usage lines without a description.** `state-bake`, `pose-match`, `pose-bench`, `follow`, `keys`, `event`, `marker`,
    `idle`, `mouth`, `beats`, `export`, `frames`, `onion` and others have a synopsis but no explanation, or a description that
    sits on a continuation line; their behaviour in this document comes from running them.
12. **Naming of properties.** Messages and `list` outputs print properties in CamelCase (`LayerOpacity`, `BoneAngle`),
    but every option, mapping file and side file takes the snake_case names (`layer_opacity`, `bone_angle`); the CamelCase form
    is rejected (`bind add ... sun:LayerOpacity` fails with `unknown property 'LayerOpacity'`).
13. **`cloth`:** the usage text speaks of "per-vertex deformation"; the baked result is stored as `lattice_dx` / `lattice_dy`
    channels on the cloth's lattice points (for the default 7x7 grid, the 42 unpinned points), see section 7.
14. **`render-gpu`** exists only in builds made with `--features gpu`; it could not be run in the test build, so its options
    are taken from the usage text and are not verified here.
15. **Slow in debug builds.** `pose-bench`, `face-train`, `ci run` and `export-fit` take minutes in an unoptimised build.
    This document's timings come from the debug binary and are much shorter in a release build.
16. **Fixed (wave 5) - `golden write --scale`** above 1 used to be clamped to 1 silently; it is now refused (the scale is in (0, 1]).
    `golden check` ignored a `--scale` and used the stored one; it now refuses the option.
17. **Fixed (wave 5) - synopses.** Every option of a command's usage text is in its synopsis here and vice versa (test
    `cli_reference_synopses_match_usage`), and every usage line without alternatives is repeated here word for word (test
    `cli_reference_repeats_usage_lines_verbatim`). `render-gpu --features gpu` and `layer-normal ... relight --layer-normals`
    in the usage descriptions name a build feature and another command's option, not options of those commands.
18. **Other documents are older than the program in two places** (see [EXPORT_FORMATS.md](EXPORT_FORMATS.md) section 17):
    `docs/LIMITATIONS.md` L-29 says Lottie import takes no images, effects, precompositions or mattes (the importer now handles
    them, with the warnings listed in section 9), and L-32 lists OCA among the import formats while the CLI only has
    `export-oca`.
## Alphabetical command index

Every command with a one-line summary and the section that documents it. Sub-commands (`machine add`, `vcs commit`, ...) are in the section of their parent command.

| Command | Summary | Section |
|---|---|---|
| `--version` | print the version, licence, source offer and support links | [1](#1-conventions-shared-by-all-commands) |
| `align` | align layers to each other or to the stage | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `audio-react` | loudness drives a property | [8](#8-audio-lip-sync-and-sounds) |
| `autorig` | IK, breathing and blinking set up from bone and layer names | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `autotrace` | local bitmap tracing (colour bands, Bezier fitting) | [9](#9-import) |
| `autotrace-bench` | fidelity table of the tracer on synthetic shapes | [9](#9-import) |
| `beats` | detect loudness jumps and record markers or events | [8](#8-audio-lip-sync-and-sounds) |
| `bind` | data bindings: add, list, remove | [11](#11-state-machines-data-bindings-containers-and-scroll-regions) |
| `bind-eval` | evaluate bindings for given values | [11](#11-state-machines-data-bindings-containers-and-scroll-regions) |
| `bone-vis` | show a bone in overlays only for a frame range | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `cage` | add a mean-value-coordinate cage deformer | [7](#7-physics-cloth-and-deformers) |
| `cage-pose` | key cage vertex offsets at a frame | [7](#7-physics-cloth-and-deformers) |
| `camera-seq` | key the camera through saved shots | [6](#6-animation-keys-timeline-and-generators) |
| `chan-meta` | channel colour, shy, solo and lock flags | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `check` | validate a project (exit 1 if invalid) | [3](#3-project-creation-and-inspection) |
| `ci` | animation CI: goldens, perceptual diff, structural checks | [14](#14-quality-assurance) |
| `clip` | layer a take on the clip stack | [6](#6-animation-keys-timeline-and-generators) |
| `cloth` | bake a deterministic cloth simulation | [7](#7-physics-cloth-and-deformers) |
| `coach` | timing/principles suggestions | [14](#14-quality-assurance) |
| `collab-sim` | collaboration convergence test | [16](#16-studio-collaboration-mcp-agent-server-and-ui-registry) |
| `collab-token` | mint a per-participant collaboration token (HMAC-SHA-256, expires) | [16](#16-studio-collaboration-mcp-agent-server-and-ui-registry) |
| `color-space` | choose sRGB or linear-light compositing for the project | [12](#12-effects-depth-of-field-onion-skin-trails-and-camera) |
| `comp` | named layer states (visibility, opacity, switch choice) | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `compare` | before / after / difference image of two projects | [4](#4-rendering-and-export) |
| `container` | layout containers: add, list, remove, solve | [11](#11-state-machines-data-bindings-containers-and-scroll-regions) |
| `contracts` | motion contracts (unit tests for animation) | [14](#14-quality-assurance) |
| `demo` | write the built-in rig demo project | [3](#3-project-creation-and-inspection) |
| `demo-fx` | write the liquid / boolean / particles demo | [3](#3-project-creation-and-inspection) |
| `diff` | semantic differences between two projects | [3](#3-project-creation-and-inspection) |
| `distribute` | space three or more layers evenly | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `dof` | depth-of-field render of one frame | [12](#12-effects-depth-of-field-onion-skin-trails-and-camera) |
| `dof-export` | animated depth of field (GIF/APNG/video/folder) | [12](#12-effects-depth-of-field-onion-skin-trails-and-camera) |
| `drive` | CSV signals -> keys via mapping file | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `event` | add a named runtime event (with optional JSON payload) | [6](#6-animation-keys-timeline-and-generators) |
| `explain` | why a property has its value: ordered contributions | [3](#3-project-creation-and-inspection) |
| `export` | export GIF, APNG, WebM/MP4, PNG sheet, HTML, Lottie, .lottie, glTF/GLB, ... | [4](#4-rendering-and-export) |
| `export-fit` | best-quality export that fits a byte budget | [13](#13-level-of-detail-size-reports-and-export-fit) |
| `export-oca` | write an Open Cel Animation folder | [4](#4-rendering-and-export) |
| `eye-clip` | mask iris/pupil layers by the eye shape | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `face-bake` | landmarks -> keys | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `face-calibrate` | neutral-face calibration | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `face-detect` | face boxes and 23 landmarks in an image | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `face-eval` | held-out accuracy report of the detector | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `face-synth` | synthetic head animation | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `face-track` | landmarks NDJSON from video or frames | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `face-train` | train the face detector on synthetic faces | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `follow` | overlapping action / follow-through generators | [6](#6-animation-keys-timeline-and-generators) |
| `foot-lock` | pin a controller during ground contact | [6](#6-animation-keys-timeline-and-generators) |
| `frames` | render every frame into a folder (SVG, or PNG with --png) | [4](#4-rendering-and-export) |
| `fuzz` | find poses where the rig breaks | [14](#14-quality-assurance) |
| `fuzz-grid` | severity map of two parameters | [14](#14-quality-assurance) |
| `fx` | render a frame with a post-effects chain | [12](#12-effects-depth-of-field-onion-skin-trails-and-camera) |
| `glue` | glue mesh vertices together | [7](#7-physics-cloth-and-deformers) |
| `golden` | write/check regression frames | [14](#14-quality-assurance) |
| `idle` | generate a procedural idle channel | [6](#6-animation-keys-timeline-and-generators) |
| `import-anim` | animated GIF/APNG -> frame-by-frame layer | [9](#9-import) |
| `import-bvh` | BVH motion capture -> channels | [9](#9-import) |
| `import-lottie` | Lottie subset -> project | [9](#9-import) |
| `import-ora` | OpenRaster -> project | [9](#9-import) |
| `import-seq` | PNG sequence -> frame-by-frame layer | [9](#9-import) |
| `import-svg` | SVG subset -> vector layer | [9](#9-import) |
| `keyedit` | dope-sheet edits as one undo-safe batch | [6](#6-animation-keys-timeline-and-generators) |
| `keys` | bulk key operations over all keyed channels | [6](#6-animation-keys-timeline-and-generators) |
| `layer-normal` | attach a normal map to a layer | [12](#12-effects-depth-of-field-onion-skin-trails-and-camera) |
| `layer-pivot` | move a layer's pivot without changing the picture | [7](#7-physics-cloth-and-deformers) |
| `layout-preview` | render with layout re-solved for another canvas size | [11](#11-state-machines-data-bindings-containers-and-scroll-regions) |
| `lint` | static rig/animation checks (exit 1 on errors) | [14](#14-quality-assurance) |
| `lipsync` | drive a mouth switch layer from audio loudness (not phoneme-based) | [8](#8-audio-lip-sync-and-sounds) |
| `lipsync-text` | align a transcript to the audio and derive mouth shapes | [8](#8-audio-lip-sync-and-sounds) |
| `live-map` | recorded live input (MIDI, mouse, gamepad, keys) -> keys, takes, events | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `lod` | levels of detail with declared pixel error bounds | [13](#13-level-of-detail-size-reports-and-export-fit) |
| `lod-pick` | coarsest level that fits a pixel budget on a screen | [13](#13-level-of-detail-size-reports-and-export-fit) |
| `loop-solve` | make keyed channels loop seamlessly | [6](#6-animation-keys-timeline-and-generators) |
| `machine` | runtime state machines: add, list, remove, validate | [11](#11-state-machines-data-bindings-containers-and-scroll-regions) |
| `machine-sim` | replay timed inputs: state log plus SVG frames | [11](#11-state-machines-data-bindings-containers-and-scroll-regions) |
| `magnet` | bake a proximity pull | [6](#6-animation-keys-timeline-and-generators) |
| `make-loop` | make a cycle loop seamlessly (optionally in place) | [6](#6-animation-keys-timeline-and-generators) |
| `marker` | add a timeline marker or range | [6](#6-animation-keys-timeline-and-generators) |
| `mcp` | agent server (JSON-RPC over stdio) | [16](#16-studio-collaboration-mcp-agent-server-and-ui-registry) |
| `mesh-lock` | followers follow a host mesh surface | [7](#7-physics-cloth-and-deformers) |
| `mesh-trace` | trace an image alpha silhouette into a skinned triangle mesh | [7](#7-physics-cloth-and-deformers) |
| `mirror` | mirror all keyed channels left<->right | [6](#6-animation-keys-timeline-and-generators) |
| `mocap-info` | hierarchy and extent of a BVH file | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `mocap-match` | BVH joint to rig bone matching | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `mocap-retarget` | bake mocap as appended channels | [10](#10-live-input-midi-face-tracking-and-motion-capture) |
| `mouth` | import mouth cues (Papagayo/Rhubarb) | [8](#8-audio-lip-sync-and-sounds) |
| `onion` | onion-skin PNG or SVG | [12](#12-effects-depth-of-field-onion-skin-trails-and-camera) |
| `parallax` | multiplane depth for top-level layers | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `parent-switch` | bake animated re-parenting without a jump | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `physics` | bake a chain simulation | [7](#7-physics-cloth-and-deformers) |
| `pick` | what is under a canvas point (topmost first) | [3](#3-project-creation-and-inspection) |
| `pose-bench` | held-out accuracy of the pose methods | [17](#17-pose-database-suggestions-and-other-helpers) |
| `pose-db` | pose database: build, info, train-mlp | [17](#17-pose-database-suggestions-and-other-helpers) |
| `pose-match` | fit parameters so bone tips reach goals | [17](#17-pose-database-suggestions-and-other-helpers) |
| `pose-suggest` | in-between, completion and next-pose suggestions | [17](#17-pose-database-suggestions-and-other-helpers) |
| `proxy-bench` | milliseconds per frame at each proxy level | [4](#4-rendering-and-export) |
| `reduced-motion` | write a calmer copy of a project | [14](#14-quality-assurance) |
| `reference` | mark a layer as a tracing reference (never exported) | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `render` | render one frame to PNG or SVG (scrub proxies, bones, scale) | [4](#4-rendering-and-export) |
| `render-gpu` | GPU render (wgpu builds only) | [4](#4-rendering-and-export) |
| `render-hdr` | float render to OpenEXR or 16-bit PNG | [4](#4-rendering-and-export) |
| `reparent` | change a bone's parent keeping the rest pose | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `retarget` | copy animation between rigs by chain pairs | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `safety` | flash / red-flash / camera-motion report | [14](#14-quality-assurance) |
| `script` | run a sandboxed Rhai script that edits the project | [6](#6-animation-keys-timeline-and-generators) |
| `scroll` | scroll regions: add, list, remove, eval | [11](#11-state-machines-data-bindings-containers-and-scroll-regions) |
| `select-box` | marquee selection | [3](#3-project-creation-and-inspection) |
| `shot` | save a camera shot | [6](#6-animation-keys-timeline-and-generators) |
| `size-report` | where the bytes of a project/export are | [13](#13-level-of-detail-size-reports-and-export-fit) |
| `skin` | named skins: layer sets, skin-only bones, constraints, IK chains | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `skin-diffuse` | heat-diffusion skin weights | [7](#7-physics-cloth-and-deformers) |
| `slider` | create a controller bone that blends stored poses | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `snapshot` | numbered revision history of a project in a folder (save, list, restore) | [3](#3-project-creation-and-inspection) |
| `soft-scale` | proportional scaling of a selection | [7](#7-physics-cloth-and-deformers) |
| `sound` | project sound tracks: list, add, edit, remove, mix | [8](#8-audio-lip-sync-and-sounds) |
| `speech-motion` | additive nods and brow raises timed on aligned words | [8](#8-audio-lip-sync-and-sounds) |
| `spring-fit` | fit spring frequency/damping to a curve | [6](#6-animation-keys-timeline-and-generators) |
| `state-bake` | bake a frame-based state machine into keys | [11](#11-state-machines-data-bindings-containers-and-scroll-regions) |
| `stats` | counts of bones, layers, channels, keys (JSON) | [3](#3-project-creation-and-inspection) |
| `stretch` | bake a stretch-to-reach follower | [6](#6-animation-keys-timeline-and-generators) |
| `studio` | browser editor (local web server) | [16](#16-studio-collaboration-mcp-agent-server-and-ui-registry) |
| `tag` | propose standard role names for bones and layers | [3](#3-project-creation-and-inspection) |
| `take-apply` | apply a library take with cross-fades | [6](#6-animation-keys-timeline-and-generators) |
| `take-list` | list takes in a library | [6](#6-animation-keys-timeline-and-generators) |
| `take-record` | record a take into a library | [6](#6-animation-keys-timeline-and-generators) |
| `timecode` | frame <-> SMPTE timecode (drop-frame aware) | [3](#3-project-creation-and-inspection) |
| `timeline` | tracks per channel with keys, locks and colours | [3](#3-project-creation-and-inspection) |
| `trace` | trace a PNG alpha channel into a vector path | [9](#9-import) |
| `trail` | motion trail of a bone, layer or point | [12](#12-effects-depth-of-field-onion-skin-trails-and-camera) |
| `trim` | bake a trim path into a vector layer | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `ui` | studio command registry, shortcuts, layouts, search | [16](#16-studio-collaboration-mcp-agent-server-and-ui-registry) |
| `vcs` | version control with semantic merge for one project file | [15](#15-version-control-vcs) |
| `view-swap` | drive a switch layer from a bone property (step function) | [5](#5-rigging-skins-ik-and-layout-helpers) |
| `walk` | generate a walk cycle | [6](#6-animation-keys-timeline-and-generators) |
| `warp-pivot` | rotate/scale/move a selection of control points | [7](#7-physics-cloth-and-deformers) |
| `waveform` | min/max peaks for drawing | [8](#8-audio-lip-sync-and-sounds) |
