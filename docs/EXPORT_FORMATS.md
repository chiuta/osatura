<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# OSATURA export and import formats

What every file format that OSATURA writes or reads keeps, approximates and drops, which command produces it, and how to
check the result. This text is licensed CC-BY-4.0. Documented version: **osatura 0.1.0**.

Every command line below was run against the built-in demo project (`osatura demo demo.json`) or an 8-frame copy of it
(`small.json`, the same project with `"frames": 8`). Sizes quoted are from that 8-frame, 640x480 project and are meant for
orientation only: they scale with frames, anchors and layers, not with anything fixed. Command details (every option,
every error) are in [CLI_REFERENCE.md](CLI_REFERENCE.md); the file format of the project itself is in
[PROJECT_FORMAT.md](PROJECT_FORMAT.md); codec and patent status is in [PATENT_NOTES.md](PATENT_NOTES.md) (not legal advice).

## Contents

1. [Which format for which job](#1-which-format-for-which-job)
2. [Output formats at a glance](#2-output-formats-at-a-glance)
3. [Still images: SVG, PNG, 16-bit PNG, OpenEXR](#3-still-images)
4. [Raster animation: PNG sprite sheet, GIF, APNG, WebM, MP4](#4-raster-animation)
5. [HTML player](#5-html-player)
6. [Lottie (.json) and dotLottie (.lottie)](#6-lottie-json-and-dotlottie-lottie)
7. [glTF binary (.glb)](#7-gltf-binary-glb)
8. [Open Cel Animation (OCA) folder](#8-open-cel-animation-oca-folder)
9. [Import: SVG, OpenRaster, Lottie, images, GIF/APNG](#9-import-svg-openraster-lottie-images-gifapng)
10. [Import: BVH motion capture, MIDI and other live input](#10-import-bvh-motion-capture-midi-and-other-live-input)
11. [Audio formats](#11-audio-formats)
12. [Feature support matrix](#12-feature-support-matrix)
13. [Making files smaller: export-fit, lod, size-report](#13-making-files-smaller)
14. [ffmpeg requirements](#14-ffmpeg-requirements)
15. [Patent and licence pointers](#15-patent-and-licence-pointers)
16. [Validating what you wrote](#16-validating-what-you-wrote)
17. [Differences from other documents](#17-differences-from-other-documents)

---

## 1. Which format for which job

| You want... | Use | Why |
|---|---|---|
| The exact look, shared as one file, plays in any browser, scrubbable | `.html` | Inline SVG frames drawn by OSATURA's own renderer; nothing is approximated. |
| A small vector animation for a web page or app with a Lottie player | `.json` or `.lottie` | Vector, resolution independent, small; but see the drop list (images, variable-width strokes, audio...). |
| A looping preview for chat or a README | `.gif` | Works everywhere; 256 colours per frame, 1-bit transparency, no sound. |
| A GIF replacement with full colour and alpha | `.apng` | Lossless 8-bit RGBA; larger than GIF. |
| A video with sound | `.mp4` (H.264 + AAC) or `.webm` (VP9/AV1 + Opus) | Needs `ffmpeg` installed by you. `.webm` keeps transparency. |
| Sprite sheet for a game engine | `.png` | One PNG plus a JSON atlas with the tile rectangles. |
| A skeleton and motion for a 3D tool | `.glb` | Bones, animation, flat shapes as triangle meshes. |
| Frames for compositing/painting tools | `export-oca` (Open Cel Animation) or `frames --png` | Cels as PNG, one folder. |
| Linear-light or high-bit-depth stills | `render-hdr` (`.exr` or 16-bit `.png`) | Float renderer, 16-bit or half/float OpenEXR. |
| One still frame | `render` (`.svg` or `.png`) | Vector or 8-bit raster. |

## 2. Output formats at a glance

| Format | Command | Extension | Needs ffmpeg | Alpha | Audio | Size for the 8-frame demo* |
|---|---|---|---|---|---|---|
| SVG (one frame) | `render` | `.svg` | no | yes | no | a few KB |
| SVG/PNG sequence | `frames` | folder | no | yes | no | one file per frame |
| PNG (one frame) | `render` | `.png` | no | yes (8-bit) | no | |
| 16-bit PNG | `render-hdr` | `.png` | no | yes | no | |
| OpenEXR (half/float) | `render-hdr` | `.exr` | no | yes | no | 466 KB half, 923 KB float (640x480) |
| PNG sprite sheet + JSON | `export` | `.png` (+ `.atlas.json`) | no | yes | no | |
| GIF | `export` | `.gif` | no | 1-bit | no | 83 KB |
| APNG | `export` | `.apng` | no | yes | no | 131 KB |
| WebM | `export` | `.webm` | yes | yes | yes (Opus) | 10.5 KB |
| MP4 | `export` | `.mp4` | yes | no | yes (AAC) | 8.9 KB |
| HTML player | `export` | `.html`, `.htm` | only for audio | yes | embedded mp3 | 66 KB |
| Lottie | `export` | `.json` | no | yes | no | 47 KB |
| dotLottie | `export` | `.lottie` | no | yes | no | 11.5 KB |
| glTF binary | `export` | `.glb` | no | yes | no | 115 KB |
| Open Cel Animation | `export-oca` | folder | no | yes | no | one PNG per cel |

\* GIF, APNG, WebM and MP4 at `--scale 0.5` (320x240); vector formats at full size. "Needs ffmpeg" means a working
`ffmpeg` on `PATH` (section 14). Output names are chosen by extension, case-insensitively; an unknown extension is refused and
nothing is written (`unknown export format '.xyz' (use .gif .html .json .mp4 .apng .webm .png .glb)`, and `.lottie` is also
accepted).

All writers refuse to overwrite the input project with the output, write through a temporary file and rename it, and refuse
symbolic links unless `--follow-symlinks` is given (CLI_REFERENCE section 1.4). The size guard for vector exporters
(Lottie, HTML, GLB) refuses an export of more than about 40 million values (frames x anchors, plus bones and layers per frame)
with `export would write about N million values ...`.

## 3. Still images

### SVG (`render ... out.svg`, `frames`)

```
$ osatura render demo.json 12 f12.svg --bones
wrote f12.svg
$ osatura frames demo.json fr            # fr/frame_0000.svg ... frame_0047.svg
```
The vector output of OSATURA's own renderer: paths, gradients, strokes with width profiles, dashes, blend modes, masks and layer
effects are written as SVG. `--bones` draws the skeleton overlay. For `.svg`, `--scale` leaves the document's viewBox unchanged. **Known drops/limits:** none are reported by the renderer; how a given SVG viewer treats blend modes and filters is up
to that viewer. `--proxy N` writes a simplified scrub proxy that drops fine strokes, gradients and blend modes (not for
deliverables). **Validation:** open in a browser, or `python3 -c "import xml.dom.minidom,sys; xml.dom.minidom.parse(sys.argv[1])" f12.svg`.

### PNG (`render ... out.png`, `frames --png`)

8-bit RGBA, sRGB, rendered by the engine's own rasteriser. `--scale N` in (0, 16] multiplies the pixel size. A project with
`"background": null` renders transparent where nothing is drawn. By default not colour managed: values blend in sRGB-encoded space.
Since wave 7 a project can opt in to linear-light compositing (`osatura color-space p.json out.json linear`, field `color_space`):
the raster, GIF, APNG, video and `render-hdr` outputs are then composited in f32 linear light and converted to sRGB once; SVG and
Lottie do not change (LIMITATIONS L-22).

### 16-bit PNG and OpenEXR (`render-hdr`)

```
$ osatura render-hdr small.json 3 f.exr                           # HALF, 640x480, 465,611 bytes
$ osatura render-hdr small.json 3 f2.exr linear --precision float # FLOAT, 923,211 bytes
$ osatura render-hdr small.json 3 f4.png                          # 16-bit/colour RGBA, sRGB-encoded
```
* **OpenEXR:** single-part, scanline, **uncompressed**, `A B G R` channels, linear light, premultiplied alpha. `HALF` by default,
  `FLOAT` with `--precision float` (or `--float32`). The word `linear` has no effect for EXR (always linear).
* **16-bit PNG:** sRGB-encoded unless the word `linear` is given.
* **Fidelity:** the float renderer carries more precision than the 8-bit one, but the artwork is ordinary sRGB colour, so the file
  holds display-range values at higher precision, not scene-referred HDR light. The project background is not drawn.
* **Known drops/limits:** one frame per file; no compression, layers or AOVs; no ICC profile; `--8bit` falls back to the 8-bit
  rasteriser as the source.
* **Validation:** `ffprobe f.exr` reports `exr 640x480 gbrapf32le`; any EXR viewer opens it.

## 4. Raster animation

All of these render every frame through the raster engine first. A project with `"background": null` renders with transparent
pixels wherever nothing is drawn; the demo has a background colour (`#f6f1e7`) and is therefore opaque. `--scale N` in (0, 16]
applies to `.gif .apng .webm .mp4 .png`; on other formats it is refused (`unknown option --scale for 'export .json'`).

### PNG sprite sheet (`export ... out.png`)

```
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.atlas.json
wrote out3.png                    # 4 columns x 2 rows of 160x120 tiles
```
One PNG with all frames as tiles (`--columns N`, default a near-square grid: ceil(sqrt(frames)) columns) and an atlas
`out3.atlas.json` (the output name with the extension replaced by `.atlas.json`) listing `{index, x, y, w, h}` per frame. Alpha is kept.
The atlas used to be `out3.json` and could silently overwrite a Lottie file of the same stem; the new name avoids that, and the atlas path goes through the same input/symlink guards.

### GIF (`export ... out.gif`)

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif                      # 8 frames, 83 KB
```
* One image per source frame, **looping forever**, a local palette of up to 256 colours per frame (quantised, no dithering).
* **Timing:** GIF delays are whole centiseconds, and viewers treat anything under 2 cs as 10 cs, so 50 fps is the real maximum.
  Up to 50 fps the delays are error-diffused (29.97 fps alternates 3 and 4 cs) so the total duration stays within 1 cs of
  `frames / fps`. Above 50 fps the sequence is **decimated to 50 fps**, the duration is kept and a warning is printed
  (`GIF cannot hold 60 fps ... use APNG, WebM or MP4 for the full rate`).
* **Transparency:** 1-bit (the transparent-colour flag is set); semi-transparent pixels are not preserved.
* **Known drops:** no audio (`note: GIF cannot carry audio; ...` when the project has sounds); no partial alpha; images wider
  or taller than 65535 pixels are refused (`image too large for GIF`).
* **`--report r.json`:** writes the list of things the format could not carry (here: the frame-rate decimation, if any).
* **Size tips:** a smaller `--scale`, fewer frames, or `export-fit --format gif` (section 13). GIF is the least efficient of the raster
  formats for photographic or gradient-heavy art.
* **Validation:** `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` prints the
  number of frames (8).

### APNG (`export ... out.apng`)

Animated PNG, 8-bit RGBA, lossless, one frame per project frame at the project fps (0.0417 s per frame at 24 fps). Full alpha is kept. No audio. Larger
than GIF for flat art (131 KB against 83 KB in the example) because there is no palette. Most browsers play APNG; some tools do not,
in which case use WebM or GIF. **Validation:** `ffprobe` with `-count_frames` (8 frames for the example).

### WebM (`export ... out.webm`) and MP4 (`export ... out.mp4`)

```
$ osatura export small.json out.mp4  --scale 0.5     # h264, yuv420p, 320x240, 8 frames
$ osatura export small.json out.webm --scale 0.5     # vp9, yuv420p + alpha_mode=1
$ osatura export small.json out.webm --codec av1     # needs an ffmpeg with libaom-av1
```
* **MP4:** H.264 (`libx264`), `yuv420p`, **no alpha**. AAC audio when the project has sound tracks.
* **WebM:** VP9 (`libvpx-vp9`, default) or AV1 (`libaom-av1`, `--codec av1`); the alpha channel is kept (`yuva420p`, container tag
  `alpha_mode=1`); Opus audio (`libopus`) when the project has sounds. Any other `--codec` value: `unknown --codec 'foo' (vp9 or av1)`.
* **Sound:** the project's sound tracks are mixed (start, gain, trim, fades; cut at the project length) and muxed.
* **Needs ffmpeg:** frames are rendered to PNG in a private temporary directory, handed to `ffmpeg`, and removed afterwards. If
  `ffmpeg` is missing: `cannot run ffmpeg (...); install it or export a GIF/HTML instead` (exit 1, nothing written). If it lacks
  the encoder: `ffmpeg failed`.
* **Known limits:** 8-bit only; the video size follows the project size and `--scale` (check the result with `ffprobe`).
  `dof-export` also writes `.mp4`/`.webm`, without audio.
* **Validation:** `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4`
  printed `h264,320,240,yuv420p,24/1,8`.
* **Patents:** see section 15; H.264 and AAC are covered by licensing pools, VP9/AV1 are not assessed.

## 5. HTML player

```
$ osatura export small.json out.html
wrote out.html                     # 66 KB for 8 frames
```
One self-contained HTML file: **every frame as inline SVG**, a play/pause button, a scrubber (range input) and a frame label,
driven by one inline `<script>`. It makes **no network request**: the only URL in the file is the SVG namespace. Opening it needs
nothing but a browser. The picture is exactly what the SVG renderer produces, so it is the most faithful of the "playable"
formats (masks, blend modes, effects, bones, IK, deformers, particles are all baked into the frames).

* **Audio:** if the project has sounds, the mixed audio is embedded as an mp3 data URI (needs `ffmpeg`; a warning above 3 MiB, the
  audio is dropped with a warning above 16 MiB).
* **Size:** grows linearly with frames (about 7.7 KB per frame here) and with the vector detail; the file is not compressed. Typical
  tips: serve it gzipped (the 66 KB example is about 9 KB gzipped), lower the frame count or the path detail with `lod`, and use
  `export-fit --format html`.
* **Limits:** `--report` is refused (`unknown option --report for 'export .html'`);
  the vector-export size guard applies; the player offers play/pause and a scrubber only.
* **Validation:** open it; `grep -c '<svg' out.html` counts the frames; `grep -o 'http[^"<]*' out.html` should list only the SVG namespace.

## 6. Lottie (.json) and dotLottie (.lottie)

```
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ osatura export small.json out.lottie       # a zip: manifest.json + a/animation.json
```

### How it is built

The file is Bodymovin (`"v": "5.7.0"`) with `fr` = project fps, `ip` = 0, `op` = frame count and `w`/`h` = the stage size. **Every
vector layer becomes one shape layer whose path geometry is baked per frame into world space (hold keyframes)**, so bones, IK,
skinning, deformers, springs, particles, the camera (including parallax) and parent transforms are all *in* the geometry. The result
plays like the original but it is not an editable rig: there are no bones, no channels, no expressions. A solid background layer is
written when the project has a background colour. Consequently the file grows with `frames x anchors`; it is much larger than a
hand-built Lottie with transform animation (47 KB for the 8-frame demo, 11.5 KB inside a `.lottie`).

### Exported

Solid and gradient fills and strokes (gradient stops with alpha; gradient end points follow the animated layer transform), dash
patterns, caps, joins and miter limit, stroke widths scaled by the layer scale, blend modes, layer opacity, alpha and inverted-alpha
track mattes for layer masks (a copy of the mask source acts as the matte), layer effects (blur, drop shadow, tint, fill) on
single-leaf layers, and **group opacity as a pre-composition** so overlapping children do not show through each other. Numbers are
written as their shortest decimal.

### Warned about (never dropped silently)

Every item below is printed on stderr as `warning: [code] layer: message` and written to `--report FILE` as
`{"format":"lottie","count":N,"warnings":[{"code","layer","message"}]}`:

| Code | Meaning |
|---|---|
| `image-layer` | Image layers are not exported (Lottie output holds vector layers only). |
| `variable-width-stroke` | Width profiles are not supported; the constant base width is used. |
| `stroke-anisotropic` | Non-uniform scale or skew on the layer; Lottie strokes are uniform, so the width is scaled by the geometric mean. |
| `gradient-anisotropic` | A gradient on such a layer is approximated (Lottie gradients are not sheared). |
| `blend-add` | Blend mode `add` has no Lottie equivalent; `screen` is used. |
| `group-blend` | A blend mode on a group is not exported. |
| `group-mask` | A mask on a group is not exported (masks on single layers are). |
| `group-effect` | Layer effects on a group (of several layers) are not exported. |
| `mask-source-not-vector` | The mask source is not a vector layer, so the mask is not exported. |
| `depth-of-field` | `camera_focus`/`camera_blur` channels are not applied; use `dof-export`. |
| `audio` | Lottie cannot carry audio; N sound track(s) are not included. |

### Dropped without a Lottie equivalent

Bones, IK, channels, state machines, data bindings, containers, scroll regions, markers/events, sounds, post-effects chains (`fx`),
normal maps, and the editable structure of the rig. Per-layer effect colours cannot be animated (the first value is used) and
sizes are in stage units (LIMITATIONS L-23).

### dotLottie (`.lottie`)

A [dotLottie 2.0](https://dotlottie.io/spec/2.0/) zip with `manifest.json` (`{"animations":[{"id":"animation"}],"generator":"OSATURA","version":"2"}`) and
`a/animation.json` (the same Lottie as above, deflated: 46,843 bytes became an 11,515-byte file). `--report` works the same
way. **Importing** a `.lottie` is not supported (`import-lottie` needs the JSON; unzip `a/animation.json` first). Up to v0.1.0-alpha.4 the container was dotLottie 1.0 (`animations/animation.json`, `autoplay`/`loop` in the manifest); the 2.0 layout plays in the current dotLottie players (checked with @lottiefiles/dotlottie-web).

### Validation and caveats

* `python3 -m json.tool out.json > /dev/null` for well-formedness; `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],len(d['layers']))"`.
* `unzip -l out.lottie` must show exactly the two entries.
* Round trip: `osatura import-lottie out.json back.json` followed by `osatura check back.json` verifies that OSATURA can read its own
  output (the round trip is lossy: the rig is gone).
* **No independent Lottie player was available** when this was written (LIMITATIONS L-28): the output was checked against the Lottie field
  layout and against OSATURA's own importer. Test in your target player (lottie-web, dotLottie player, Skottie, ...) and read the
  warnings; if exactness matters, use `.html` or video.
* **Size tips:** `export-fit --format lottie` searches four knobs (`decimals`, `path_eps`, `key_tol`, `frame_step`) against a byte budget;
  `lod` reduces anchors before export; `size-report` shows the baked Lottie size per layer (section 13).
* **Patents:** Lottie and dotLottie are open formats; nothing in PATENT_NOTES.md concerns them.

## 7. glTF binary (.glb)

```
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
```
glTF 2.0 binary (`glTF` magic, version 2, `"generator": "osatura"`), core features only. The project must have bones
(`the project has no bones to export` otherwise).

* **Exact:** the bone hierarchy and its animation (sampled at every frame, linear interpolation, shear dropped).
* **Fills** become flat triangle meshes (ear clipping). Shapes are rigid (a layer with a bone follows that bone node) or **skinned**
  (JOINTS_0/WEIGHTS_0, 4 influences per vertex) when every anchor carries bone weights.
* **Strokes** are triangulated into their own meshes (width, width profile, butt/round/square caps, miter/round/bevel joins, dashes);
  translucent strokes can look darker where triangles overlap.
* **Gradients** (linear and radial) become a small generated PNG used as a base-colour texture; pad (clamp) spread.
* **Masks** are baked by clipping triangles against the mask layer's fill, only for a vector mask source with opaque fills and no
  strokes, for the frame-0 mask shape only. **Image layers** become textured quads or deformation meshes with embedded PNG.
* **Path morphs** become morph targets with animated weights when the per-frame check reproduces the engine geometry; otherwise
  the path stays a static frame-0 shape and the reason is reported.
* **Not exported (reported as `[glb-limit]` warnings):** blend modes other than normal (drawn as normal), animated layer opacity (frozen at
  frame 0), non-bone layer transform animation, layer effects, deformers beyond their frame-0 shape, animated paths without a valid
  morph, animated mask shapes, sound.
* **Coordinates:** y is negated (glTF is y-up); shapes are layered with a small z offset (0.01 per shape) for draw order.
* **Validation:** the header is `glTF`, version 2, and the length field equals the file size (`python3` one-liner in section 16). Use a glTF
  validator or viewer of your choice for the full check; none is bundled and none was run for this document. The example above has 22 nodes,
  15 meshes, 1 skin, 1 animation and 7 embedded images.
* **Size tips:** fewer vector anchors (`lod`), no gradients (each is a texture), fewer frames (the animation is sampled per frame).
* **Patents:** open format; no pointer in PATENT_NOTES.md.

## 8. Open Cel Animation (OCA) folder

```
$ osatura export-oca small.json oca1                 # one layer "render" with all composited frames
$ osatura export-oca small.json oca2 --layers        # one OCA layer per top-level layer
wrote oca2/small.oca
```
Writes `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) plus PNG cels `layerN/fNNNN.png`. Without `--layers` there is one layer
`render` with the composited frames; with `--layers` every top-level layer is rendered alone on transparency (11 layers for the demo).
`--scale N` multiplies the cel size. **Known drops:** cels are rasters (no vectors, bones or channels survive); layer opacity and blend
are not written (every layer is a `paintlayer`, opacity 1); sounds are not exported; files inside an existing folder are overwritten.
**Import:** there is no `import-oca` command in the CLI. **Validation:** the `.oca` file is JSON (`python3 -m json.tool`); every cel listed
must exist as a PNG.

## 9. Import: SVG, OpenRaster, Lottie, images, GIF/APNG

Imports create a **new** project (they do not merge into an existing one) unless stated; use `script` or edit the JSON to combine.

### SVG (`import-svg`)

```
$ osatura import-svg art.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
wrote svg.json (3 paths)
```
Subset: `path` (all commands), `rect` (rounded corners ignored), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` with `transform`,
solid `fill`, `stroke`, `stroke-width`. Result: 24 fps, 24 frames, size from `viewBox` or `width`/`height` (else 512x512), **one vector
layer `svg`**. Skipped with a `warning:` line (printed on **stdout**): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`,
`<linearGradient>`/`<radialGradient>` (a gradient paint falls back to grey), `<image>`, `<use>`, `<foreignObject>`. Limits: 8 MB of SVG,
200,000 anchors. For photographs and rasters use `trace` / `autotrace` (CLI_REFERENCE section 9).

### OpenRaster (`import-ora`)

```
$ osatura import-ora art.ora ora.json --fps 12 --frames 24
```
One **image layer per raster layer** (embedded PNG), bottom-to-top order, hidden layers kept but invisible, groups flattened, per-layer
opacity, visibility and position, and the common `svg:` composite operations (multiply, screen, overlay, darken, lighten, color-dodge,
color-burn, hard-light, soft-light, difference, exclusion, plus mapped to add). A nested stack's opacity multiplies into its layers and its
blend mode is inherited, but overlapping layers inside a translucent stack are **not** flattened first. Duplicate names get `_1`, `_2`.
Limits: 2000 archive entries, 128 MiB per entry, 64 Mpixel layers. A file that is not a zip: `not a zip/ORA file: ...` (exit 1). There is no ORA
export. PSD import is not implemented (LIMITATIONS L-32).

### Lottie (`import-lottie`)

```
$ osatura import-lottie in.json out.json [fx.json]
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
```
A deliberate **subset**; every skipped feature is a `warning:` line.
* **Imported:** shape layers with `sh` paths, `rc`, `el`; solid and gradient fills (animated gradient end points/centre/radius become
  `grad_dx`/`grad_dy` channels); strokes; groups; layer transforms (static or keyframed position, scale, rotation, opacity; curved position
  paths are baked per frame); blend modes; a solid background layer; image layers with embedded PNG/JPEG data URIs; null layers as parents
  (their transforms are baked into children); layer masks (add/subtract/intersect with opacity, inversion, keyed shapes); luma mattes
  (approximated); precompositions; layer in/out ranges (as an opacity step track); keyframe easing (hold becomes step, bezier handles become
  cubic-bezier, none becomes linear).
* **Animated paths:** shape keys with the same topology (same anchor count and open/closed) become a base path plus one **morph target**
  per further distinct key (`k1`, `k2`, ...) with `path_morph` weight channels carrying the easing. A different topology uses the first shape
  and prints a warning.
* **Effects (`ef`):** Gaussian blur, drop shadow, tint and fill become **per-layer effects** with animated numbers as `fx_*` channels;
  static tint/fill/tritone/levels on shape layers are baked into colours. Effect colours cannot be animated (the first value is used); other
  effects are warnings.
* **Skipped with a warning:** text and other layer types, audio assets, animated anchor points (first value used), curved motion paths on
  properties other than position, masks on a track-matte source, parent chains too long to bake, more than 2000 layers, animation longer
  than 20,000 frames.
* `fx.json` is kept for compatibility: an empty array is written.
* **Not supported:** `.lottie` zip input. **Gotcha:** there is no overwrite guard against the input: `import-lottie in.json in.json` replaces the
  Lottie with the project.

### Images, GIF and APNG as frame-by-frame layers

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20      # folder of PNGs
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0                    # animated GIF/APNG
```
`import-seq` takes all PNGs of a folder in file-name order (all the same size); `import-anim` reads an animated GIF or APNG using the file's
own delays (a plain PNG gives one frame). Both add a switch layer with one embedded image per frame plus a step channel; they work on an
**existing** project. A long GIF makes the project large (every frame is an embedded PNG). `trace` and `autotrace` turn bitmaps into vector
paths (`trace` reads the alpha channel; `autotrace` quantises colours and fits Bezier curves).

## 10. Import: BVH motion capture, MIDI and other live input

### BVH (`import-bvh`, `mocap-info`, `mocap-match`, `mocap-retarget`)

```
$ osatura mocap-info walk.bvh
$ osatura import-bvh demo.json walk.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```
`import-bvh` maps chosen joint rotation channels onto bones as keyed `bone_angle` channels sampled at the project fps and frame count
(`--axis` selects `Zrotation`, `Xrotation` or `Yrotation`; `--tol` simplifies keys). `mocap-match` proposes a joint-to-bone matching with a
confidence table and `mocap-retarget` bakes appended channels with root motion, a view (`front|side|top|yaw=DEG`) and scale. The BVH
text subset only: no binary mocap formats, no FBX (by design). **Limits (LIMITATIONS L-45):** no real motion-capture file was tested; the 2-D
projection discards depth; twist about a bone axis, fingers and face are ignored; the rig keeps its bone lengths, so foot sliding is only
reduced.

### MIDI and other live input (`live-map`)

```
$ osatura live-map small.json song.mid mapping.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote out.json
```
A **Standard MIDI File** (`.mid`; MIDI 1.0 messages, running status, tempo meta events, ticks or SMPTE division) or an events JSON
(`{"events":[{"source","id","value","time"}]}`, value normalised 0..1, time in seconds) is turned into keyed channels, events and takes through a
mapping file. Sources: `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`. At most
3,000,000 events; times within 7 days. Nothing talks to a MIDI device from the CLI; the studio uses the browser's Web MIDI. MIDI is **import
only** (there is no MIDI export). Malformed files return an error, never a crash.

## 11. Audio formats

| Use | Formats | Notes |
|---|---|---|
| Analysis (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) | **WAV** only | 8/16/24/32-bit integer or 32-bit float PCM, any channel count mixed to mono, up to 256 MiB. Convert MP3/OGG first (`ffmpeg -i in.mp3 out.wav`). |
| Project sound tracks (`sound add`) | WAV, OGG, MP3, anything `ffprobe` accepts | Stored as a **reference** (relative path, start frame, gain 0..4, trim window, fades); nothing is embedded; the file must stay inside the project folder unless `--allow-external-assets`. |
| Mixed output (`sound mix`) | `.wav` (44.1 kHz PCM 16-bit), `.ogg` (Vorbis q4), `.mp3` (96 kb/s) | Mixed through ffmpeg for the **project length**; `.flac` and others: `unknown audio format '.flac' (use .wav, .ogg or .mp3)`. |
| Inside exports | `.mp4` AAC, `.webm` Opus, `.html` embedded mp3 | `.gif`/`.apng` stay silent with a note; Lottie warns `[audio]`; GLB and OCA drop sound. |
| Mouth cues (`mouth`) | Papagayo/Moho-switch text, Rhubarb TSV | Not audio: cue files. |

Encoders used: `libmp3lame` (mp3), `libvorbis` (ogg), `aac`, `libopus`. No per-track pan/EQ/ducking or looping (ROADMAP section 9).

## 12. Feature support matrix

Rows are things a project can contain, columns the output formats. **yes** = carried as in the project, **approx** = carried with a visible
difference, **baked** = present only as per-frame results, **no** = dropped. Cells for formats I could not check in an independent viewer
describe what the exporter writes.

| Feature | SVG / PNG | Sprite sheet / APNG | GIF | WebM / MP4 | HTML | Lottie / .lottie | GLB | OCA |
|---|---|---|---|---|---|---|---|---|
| Vector shapes, fills | yes | baked (raster) | baked (raster) | baked (raster) | yes (SVG) | yes | approx (triangles) | baked (raster) |
| Gradients | yes | baked | baked | baked | yes | yes (approx if layer skewed) | yes (texture) | baked |
| Strokes with width profile | yes | baked | baked | baked | yes | approx (constant width) | yes | baked |
| Dashes, caps, joins | yes | baked | baked | baked | yes | yes | yes (triangulated) | baked |
| Image (bitmap) layers | yes | baked | baked | baked | yes | no (`image-layer`) | yes | baked |
| Bones and animation | baked | baked | baked | baked | baked | baked | **yes (exact)** | baked |
| Skinning / deformers | baked | baked | baked | baked | baked | baked | skinned; deformers at frame 0 | baked |
| Camera and parallax | baked | baked | baked | baked | baked | baked (camera moves are in the geometry) | no (shapes come from frame 0) | baked |
| Blend modes | yes | baked | baked | baked | yes | approx (`add` becomes `screen`; none on groups) | no (normal) | baked |
| Layer masks | yes | baked | baked | baked | yes | single layers only | frame-0 shape, restricted | baked |
| Layer effects (blur, shadow...) | yes | baked | baked | baked | yes | single-leaf layers | no | baked |
| Animated opacity | baked | baked | baked | baked | baked | yes | no (frozen at frame 0) | baked |
| Path morphs | baked | baked | baked | baked | baked | baked | morph targets (when valid) | baked |
| Depth of field | via `dof-export` | via `dof-export` | via `dof-export` | via `dof-export` | no | no (warned) | no | no |
| Post-effects chain (`fx`) | PNG only | no | no | no | no | no | no | no |
| Transparency | yes | yes (alpha) | 1-bit | WebM yes, MP4 no | yes | yes | yes | yes |
| Audio | no | no | no | **yes** | yes (mp3) | no (warned) | no | no |
| Looping | n/a | sheet: n/a, APNG: yes | yes | player-dependent | player | player (dotLottie 2.0 keeps loop in the player or a state machine) | n/a | n/a |
| Editable rig/channels afterwards | no | no | no | no | no | no | bones only | no |
| State machines, bindings, containers, scroll | no | no | no | no | no | no | no | no |
| Resolution independent | yes (SVG) | no | no | no | yes | yes | yes | no |
| Needs ffmpeg | no | no | no | **yes** | audio only | no | no | no |

"Baked" means the effect of the feature is in the pixels or the per-frame geometry, but the feature itself cannot be edited afterwards.

## 13. Making files smaller

1. **See where the bytes are:** `osatura size-report project.json` lists the project file, the baked Lottie and the HTML page by layer,
   image and channel (`--json` for scripts).
2. **Fit a budget automatically:** `osatura export-fit project.json out.json --max-size 50KB --format lottie` searches the exporter knobs and
   keeps the best SSIM that fits. Formats: `lottie`, `html`, `gif`, `apng`, `webm`, `project` (a slimmed project JSON). `--measure gzip` budgets the
   gzip size, `--min-ssim X` sets a quality floor, `--max-seconds N` is a real wall-clock limit (best result so far is kept). Exit 1 with the
   smallest result when the budget cannot be met.
3. **Reduce the art first:** `osatura lod project.json --tolerance-px 1 --levels 3 --out-dir lv --verify` writes level-of-detail projects with a
   declared (and, with `--verify`, measured) error in output pixels; `lod-pick --screen-px WxH` picks the coarsest level for a given screen.
4. **Per format:**
   * Lottie/HTML/GLB grow with `frames x anchors`: fewer frames, fewer anchors, a lower fps, trimmed key ranges (`keys` tools).
   * GIF/APNG grow with pixels x frames: `--scale 0.5` is a 4x reduction; GIF suits flat art, APNG suits alpha, WebM/MP4 suit anything with motion.
   * `.lottie` is the same Lottie deflated (about 4x smaller in the example); serve `.html`/`.json` gzipped.
   * Images in a project are embedded as base64 PNG: shrink them before import.
5. Limits: a Lottie of a 14-layer rig cannot be made arbitrarily small (a 1 KB budget on the example reports "every knob is at its floor"); the
   search needs time on large projects.

## 14. ffmpeg requirements

OSATURA never bundles or links ffmpeg; it runs the `ffmpeg` (and `ffprobe`) found on `PATH` as an **external subprocess** of your own installation.

| Needed for | Encoder/decoder | Checked on |
|---|---|---|
| `export .mp4` | `libx264` (`yuv420p`), `aac` | ffmpeg 6.1.1 |
| `export .webm` (default) | `libvpx-vp9`, `libopus` | ffmpeg 6.1.1 |
| `export .webm --codec av1` | `libaom-av1`, `libopus` | ffmpeg 6.1.1 |
| `dof-export .mp4/.webm` | same as above, `--codec` | ffmpeg 6.1.1 |
| `sound mix .ogg` | `libvorbis` | ffmpeg 6.1.1 |
| `sound mix .mp3`, `.html` audio | `libmp3lame` | ffmpeg 6.1.1 |
| `face-track` from a video | decoders only | |
| `sound add` content check | `ffprobe` (without it, only existence is checked for non-WAV files) | |

Check yours with `ffmpeg -hide_banner -encoders | grep -E 'libx264|aac|libvpx-vp9|libaom-av1|libopus|libvorbis|libmp3lame'`. Everything else
(GIF, APNG, HTML without audio, Lottie, dotLottie, GLB, OCA, PNG, SVG, EXR) works without ffmpeg.
If ffmpeg is missing the command fails with `cannot run ffmpeg (...)` and writes nothing; a missing encoder gives `ffmpeg failed`.

## 15. Patent and licence pointers

* [PATENT_NOTES.md](PATENT_NOTES.md) records what has **not** been screened. For export, the relevant row is *Video and audio codecs through ffmpeg
  (`export` .mp4 / .webm, `soundtrack`)*: H.264 and AAC are covered by licensing pools; VP9 and AV1 are marketed as royalty-free but pool and individual
  claims have been announced for both; **status: not assessed, do not treat VP9/AV1 as cleared.** Because ffmpeg is the user's own installation, every
  codec choice is the user's responsibility, and OSATURA must not bundle ffmpeg.
* The deterministic raster, GIF, APNG, PNG, SVG, OpenEXR, OCA, ORA, glTF and Lottie writers are OSATURA's own implementations from public
  format descriptions (clean-room, see CLEANROOM.md). Their patent status is not assessed beyond that: PATENT_NOTES.md lists only the areas above
  (face-driven puppeteering, state machines and data binding, FABRIK/MVC/weights, stencil-then-cover filling, codecs).
* GIF: the LZW patents are long expired in all jurisdictions; this is general knowledge, not a clearance recorded in the repository.
* Licences of produced files: files you export are yours; the engine is MPL-2.0 and the CLI AGPL-3.0-or-later, which applies to the programs, not to the
  files they write. The studio offers its corresponding source at `/source` (AGPL section 13).
* This is not legal advice.

## 16. Validating what you wrote

| Format | Check | Expect |
|---|---|---|
| any project | `osatura check out.json` | `ok: ...` (also after `import-*`) |
| GIF | `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` | frame count (8) |
| APNG | same command on `out.apng` | frame count (8) |
| MP4 | `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4` | `h264,320,240,yuv420p,24/1,8` |
| WebM | same on `out.webm` with `:stream_tags=alpha_mode` | `vp9,...,yuv420p,...,1` (alpha_mode 1) |
| EXR | `ffprobe f.exr` | `exr 640x480 gbrapf32le` |
| Lottie | `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],d['w'],d['h'],len(d['layers']))"` | `5.7.0 24.0 8 640.0 480.0 12` |
| Lottie warnings | `osatura export p.json out.json --report rep.json` | `rep.json` lists every approximation |
| dotLottie | `unzip -l out.lottie` | `manifest.json` and `a/animation.json` only |
| Lottie round trip | `osatura import-lottie out.json back.json && osatura check back.json` | `ok:` |
| GLB | `python3 -c "import struct;b=open('out.glb','rb').read();print(b[:4],struct.unpack('<II',b[4:12]),len(b))"` | `b'glTF' (2, N)` with N equal to the length |
| HTML | `grep -c '<svg' out.html`; `grep -o 'http[^"<]*' out.html` | one `<svg` per frame; only the SVG namespace URL |
| SVG | open in a browser or parse as XML | well-formed |
| Sprite sheet | `python3 -m json.tool out.atlas.json` | `{index,x,y,w,h}` per frame |
| OCA | `python3 -m json.tool <name>.oca` | every referenced cel exists |
| Visual regression | `osatura ci diff a.png b.png --metric ssim` | exit 0 when equal; see [CI.md](CI.md) |

For visual checks of a lossy export against the original, `osatura export-fit` already reports SSIM and CIEDE2000 against the original render, and
`osatura compare` writes a before/after/difference image of two projects.

## 17. Differences from other documents

* `docs/LIMITATIONS.md` L-29 says Lottie import does not take image assets, effects, precomps or mattes; the current importer does import images
  with embedded data URIs, precompositions, per-layer effects (blur, drop shadow, tint, fill) and approximates luma mattes (section 9). The list of
  skipped features in section 9 is what the command really warns about.
* `docs/LIMITATIONS.md` L-32 lists OCA among the import formats; the CLI has an OCA **export** (`export-oca`) and no `import-oca` command.
* The usage text of `export` names only `.gif .html .json .mp4` plus "the open formats"; the full list of extensions is the one in section 2.
* Other mismatches between usage text and behaviour are collected at the end of [CLI_REFERENCE.md](CLI_REFERENCE.md).
