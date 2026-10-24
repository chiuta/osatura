# OSATURA — feature matrix + reference and competitor checklists

> Version wording: all crates are `0.1.0` (the first builds are the draft pre-release `v0.1.0-alpha.3`); "v0.5 / v0.6 / v0.7" in this repository are *feature levels* (see [CHANGELOG.md](CHANGELOG.md)), and `ROADMAP.md` section 9 is the authority on what is built.

Honest status of what exists. **✅ done & tested · 🟡 partial · ❌ not implemented.**

> Scope note: OSATURA is an independently implemented engine (from public specifications and papers; competitor feature lists were consulted for scope; no third-party source code was copied; see CLEANROOM.md for the research disclosure), not a re-implementation of any
> commercial product, and it does **not** reach parity with a 25-year-old commercial suite
> (mature GUI, drawing tools, live input, 3D, ecosystem). The first tables are organised by
> capability; the **second half of this file** is a version-by-version checklist of the
> reference product's publicly documented features (sources listed at the end) with
> OSATURA's status for each. "Innovations" are claimed only relative to the generic technique,
> not as proven absent from any other tool.

---

# ★ BUILD LADDER — from simple to complex, with copyright / IP status (feature level v0.7, 20-22 Oct 2026)

This section is the **single place to see what can be built, in what order, and whether it is safe**.
It supersedes the scattered "candidate #" lists further down; details and IDs live in `ROADMAP.md`
(R-xxx / I-xx), the clean-room method in `CLEANROOM.md`, licences in `LICENSING.md`.

**Status:** ✅ built & tested · 🟡 partly built · ⬜ planned (not started)
**IP status (after the alternatives of ROADMAP §8):**
🟢 safe — open standard, textbook maths, or our own design ·
🟠 safe *if* you keep a condition (stated in the last column) ·
🔴 do not build.
*Engineering screen, not legal advice; a lawyer should confirm every 🟠 before a commercial release.*

## Level 1 — Foundations (hours–days each; no IP exposure at all)
| Feature | Status | IP | Safe basis / condition |
|---|---|---|---|
| Project JSON, validation, pure `evaluate(frame)` | ✅ | 🟢 | own schema |
| Bézier paths, fills, strokes, gradients, dashes, variable-width strokes | ✅ | 🟢 | textbook geometry |
| FK bones, limits, 2-bone/FABRIK IK, constraints | ✅ | 🟢 | textbook kinematics |
| Keyframes, easing, expressions, noise | ✅ | 🟢 | own design |
| Patch-op editor with exact undo/redo | ✅ | 🟢 | RFC 6902-style JSON pointers |
| SVG, PNG, GIF, HTML player, Lottie export | ✅ | 🟢 | public specs |
| Provenance log, `stats` / semantic `diff`, golden-frame regression (exact hash + tolerance), resource caps, mutation-fuzz test, file-format version gate (R-001…R-005) | ✅ | 🟢 | own design |
| Own standard role vocabulary + name-based tagger (`tag`) (R-006) | ✅ | 🟢 | own names, plain anatomy words |
| JSON Schema export (R-003 schema) | ⬜ | 🟢 | own schema |

## Level 2 — Authoring basics and open outputs (days)
| Feature | Status | IP | Safe basis / condition |
|---|---|---|---|
| APNG, sprite sheet PNG+JSON | ✅ | 🟢 | PNG/APNG specs |
| WebM VP9 / AV1 with alpha (via your ffmpeg) | ✅ | 🟢 | openly specified codecs; patent status not assessed |
| OCA export (Open Cel Animation) | ✅ | 🟢 | public spec; import ⬜ |
| OpenRaster (.ora) import | ✅ | 🟢 | open spec, zip-bomb limits |
| MP4 / H.264 | ✅ | 🟠 | only via the *user's own* ffmpeg; never bundle an encoder |
| Key shift/scale/reverse/copy/paste, batch ease, per-channel time offset, markers & regions, SMPTE timecode (R-010, R-011, R-044) | ✅ | 🟢 | generic editing |
| Revision snapshots with pruning and crash recovery (R-014); layer comps (R-015); timeline events with payloads (R-045); reparent preserving rest pose (part of R-013) | ✅ | 🟢 | generic editing |
| Align/distribute, reference layer, shy/solo/lock tags (R-012, R-013, R-016) | ✅ | 🟢 | generic editing |
| Onion skin, motion trails (overlays never exported) (R-020) | ✅ | 🟢 | own design |
| Post-effects chain: blur, drop shadow, glow, outline, posterize, pixelate, levels/gamma, fade (part of R-022) | ✅ | 🟢 | textbook image processing |
| Trim paths, per-layer filters, masks, depth-of-field (R-021, R-024, R-025, R-034) | ✅ (baked trim paths; layer effects limits: see ROADMAP §9) | 🟢 | standard image maths |
| SVG import (paths, shapes, groups, transforms, solid paints; gradients/text/clip reported as warnings) (R-032) | ✅ | 🟢 | W3C SVG 1.1 grammar |
| BVH import (one rotation axis → bone angles, bounded key reduction) (R-036) | ✅ | 🟢 | open text format |
| dotLottie container export (R-03A) | ✅ | 🟢 | public container spec (zip + manifest) |
| Animated-SVG export, image-sequence import, OpenEXR/16-bit output, glTF export (R-032, R-037…R-039) | 🟡 (glTF: frame-0 shapes plus bones and morph weights, see ROADMAP R-039) | 🟢 | public formats |
| Papagayo (Moho-switch) and Rhubarb mouth-cue import (R-03B) | ✅ | 🟢 | read their plain text output only |

## Level 3 — Rig and animation depth (weeks)
| Feature | Status | IP | Safe basis / condition |
|---|---|---|---|
| Springs (baked), lattices (FFD), spine deformer, switch layers, particles, liquid, booleans | ✅ | 🟢 | textbook |
| Audio → mouth-shape lip-sync, walk-cycle generator, mirror, retime, clip, crossfade | ✅ | 🟢 | own design |
| **Pin warp** (replaces cage / ARAP-style warp) | ✅ | 🟢 | Wendland 1995 RBF |
| **Heat-diffusion auto-skin weights** | ✅ | 🟢 | Laplace equation |
| **Performance recording** with bounded key reduction | ✅ | 🟢 | Douglas–Peucker |
| **Input mapping** (CSV/JSON signals → parameters) | ✅ | 🟢 | no tracker bundled |
| Soft/stretch IK (R-051), pose drivers: slider + 2-D joystick (R-050), angle-driven view swap (R-056), tag auto-rig (R-058) | ✅ | 🟢 | generic rigging; textbook maths, own design |
| Path constraints, foot-lock IK, parent-switch keys, skins, magnets (R-040…R-049, R-052…R-055, R-057) | 🟡 (built: R-040…R-043, R-044, R-047, R-048, R-052…R-055, R-057; see ROADMAP section 9 for the parts marked NOT done) | 🟢 | generic rigging |
| Idle generators (blink, breath, sway) and follow-through chains (R-047, R-048) | ✅ | 🟢 | seeded, deterministic |
| Wind, gravity, collisions, hair/cloth strips (R-070…R-072) | 🟡 (R-070, R-071 done; R-072 strips are bones only, cloth is a planar sheet) | 🟢 | mass-spring / PBD textbooks |

## Level 4 — Smart tools (weeks–months)
| Feature | Status | IP | Safe basis / condition |
|---|---|---|---|
| **Image → vector trace** (replaces ML auto-trace) | ✅ | 🟢 | marching squares + RDP |
| **Event-log state machine** + bake to channels (replaces runtime state-machine clones) | ✅ | 🟢 | own semantics — keep our names |
| **Effector-reach retargeting** | ✅ | 🟢 | own method |
| **Pose-match solver** (evolution strategy, joint-limit aware) | ✅ | 🟢 | Beyer & Schwefel |
| **WCAG 2.3.1 flash / red-flash check + camera-motion report + reduced-motion twin** | ✅ | 🟠 | public definitions only; say "engineering aid", never "certified" |
| **Rig linter**, **motion contracts**, **seamless-loop solver**, **principles coach** (I-02, I-04, I-05, I-14) | ✅ | 🟢 | own ideas |
| **Draw-the-overshoot spring fitting** (I-08) — fits frequency/damping with the engine's own integrator | ✅ | 🟢 | own idea |
| **Explain-this-pixel** for bones and layers (I-07) | ✅ | 🟢 | own idea |
| Before | after | difference review image (first step of I-06; the perceptual metric is `ci diff`, SSIM and CIEDE2000) | ✅ | 🟢 | own idea |
| Normal-map lighting, parallax, depth-of-field (R-064, R-065, R-024) | ✅ | 🟢 | textbook |
| Transcript-aligned lip-sync (R-081) | ✅ (own aligner, `speech.rs`) | 🟠 | use a permissively licensed aligner; check its model licence |
| Webcam face/body tracking feeding the input mapper (R-082) | 🟡 (own detector trained on synthetic faces only; real webcams and photographs untested) | 🟠 | user brings a permissive tracker; bundle no model weights unless licence allows |
| Full BVH mocap retargeting with auto-match by hierarchy (R-083) | ✅ | 🟢 | builds on the BVH import |
| PSD import (R-035) | ⬜ | 🟠 | read-only, published spec, no Adobe marks; or just use ORA |

## Level 5 — Platform (months)
| Feature | Status | IP | Safe basis / condition |
|---|---|---|---|
| Browser studio with undo/redo | 🟡 | 🟢 | own UI; no copying of vendor layouts or names |
| WASM runtime, stable Rust API + C ABI, sandboxed scripting (R-090…R-093) | 🟡 (R-091, R-092, R-093 done; R-090 the wasm module builds and passes the wrapper's test under Node in CI, and was run only in headless Chromium) | 🟢 | own API |
| **Agent / MCP server** over patch ops: inspect, edit (atomic, undoable), lint, safety, render, save to its own file only (R-094) | ✅ | 🟢 | open JSON-RPC protocol |
| Full GUI: gizmos, timeline, graph editor (R-100…R-103) | 🟡 (R-101, R-102 done; R-100 and R-103 pages exist but were never run in a real browser by the author) | 🟢 | generic widgets; own look |
| Animation version control with semantic merge, CI perceptual diff, explain-this-pixel (I-01, I-06, I-07) | ✅ (I-01 `osatura vcs`, I-06 `osatura ci`, I-07 `explain`) | 🟢 | own ideas |
| Pose fuzzing with stress heat-maps, size-targeted export, deterministic level of detail with a declared error bound (I-03, I-10, I-11) | 🟡 (all three built; the I-11 bound does not cover effects, skinned paths, IK-driven values or state-machine poses) | 🟢 | own ideas, textbook maths |

## Level 6 — Frontier (research-grade)
| Feature | Status | IP | Safe basis / condition |
|---|---|---|---|
| GPU renderer with CPU parity tests (R-110) | 🟡 (crate `osatura-gpu` built and tested; hardware run limited, see ROADMAP) | 🟢 | wgpu (permissive) |
| Real-time multi-user editing over the op-log (R-111) | 🟡 (done with an optional persisted session log `--collab-log`; participants and undo history are not persisted; plain HTTP, shared token) | 🟢 | CRDT literature |
| Data-binding / layout runtime (R-091, R-095) | ✅ (own design; patent screening not done, docs/PATENT_NOTES.md) | 🟠 | patent-screen before shipping |
| ML-assisted tools (R-112) | 🟡 | 🟠 | classical methods only, no model weights: `autotrace` (k-means, Schneider Bezier fit, Zhang-Suen line mode) and learned-from-your-poses suggestions (PCA, kernels, optional small MLP in an own weight format); no deep learning |

## Never build (🔴)
- Reading or writing native **Live2D, Spine, Moho, Harmony, Rive, Cartoon Animator** project files (proprietary; EULA/reverse-engineering exposure) — use Lottie, OCA, ORA, APNG, SVG instead.
- **ARAP-style fast artwork-deformation solvers** as described in Adobe's patents — pin warp, FFD and skinning cover the use case.
- Copying any vendor's **UI layout, icons, names, sample content or file-format magic numbers/terminology**.
- Mimicking the **Harding** photosensitivity tool or claiming broadcast compliance.
- Bundling **H.264/H.265/AAC encoders**.

## Conditions that apply to every 🟢 row
Provenance log kept (`CLEANROOM.md`; independent implementation, not a strict clean-room); only permissive dependencies (regenerate with `scripts/licence-audit.sh`; currently 363 third-party crates over all platforms, all MIT / Apache-2.0 / BSD / Zlib / Unlicense-class except six MPL-2.0 crates (smartstring via the `scripting` feature, which is on by default, and five via the window app `osatura-gui`); see docs/THIRD_PARTY.md); no third-party artwork or audio shipped without its own licence file; the name **OSATURA** cleared in a trademark search before launch.


# Detailed capability tables
## Drawing & shapes
| Feature | Status | Notes |
|---|---|---|
| Cubic-Bézier paths, open/closed, handles | ✅ | |
| Fill: solid, linear & radial gradient | ✅ | gradients follow the layer transform |
| Stroke: width, caps, joins, dashes | ✅ | |
| Variable-width stroke (width profile) | ✅ | polygon outline; closed paths → two rings |
| Catmull-Rom smoothing, rect/circle/ellipse/polygon helpers | ✅ | |
| Polygon booleans (union/intersect/difference/xor) | ✅ | `boolean` layer; holes bridged into one polygon; result is polygonal (flattened) |
| Metaball / liquid layers | ✅ | marching squares, bones can drive blobs |
| Image layers (PNG), embedded or file | ✅ | |
| Bitmap/vector brushes, texture fills, text/fonts | ❌ | |
| Freehand drawing, point-editing tools, image trace | ❌ | no drawing GUI |

## Rigging
| Feature | Status | Notes |
|---|---|---|
| Bone hierarchy, FK, rest pose | ✅ | |
| Angle / scale limits | ✅ | |
| IK: analytic 2-bone (bend ±) | ✅ | |
| IK: N-bone chains (FABRIK) | ✅ | |
| FK↔IK mix (animatable) | ✅ | |
| Constraints: aim, copy-rotation, pin (weight animatable) | ✅ | |
| Linear-blend skinning of anchors + handles, auto-skin | ✅ | |
| Bone-attached layers, layer parenting / groups | ✅ | |
| Secondary motion: baked damped springs | ✅ | deterministic, random-access (baked at load) |
| Pose-driven channels ("driven keys") | 🟡 | curve maps any property to another; no dedicated smart-bone UI/actions |
| Rigid-body physics, cloth, soft-body | ❌ | |

## Deformation
| Feature | Status | Notes |
|---|---|---|
| Per-point offsets (animatable) | ✅ | addressed `layer::path::index` |
| Morph targets / blend shapes (additive weights) | ✅ | |
| Free-form lattice (Bernstein FFD) | ✅ | animatable control points |
| Spine / curve bend deformer | ✅ | optional soft radius |
| "Boil" wobble (hand-drawn jitter) | ✅ | coherent noise, deterministic |
| Image mesh warp with bone weights | ✅ | grid mesh; no mesh editor |

## Animation
| Feature | Status | Notes |
|---|---|---|
| Keyframes + easing (linear/step/ease/cubic-bezier) | ✅ | |
| Extrapolation: hold/linear/loop/ping-pong/loop-offset | ✅ | |
| Expression channels | ✅ | small safe expression language; can read other channels |
| Noise channels, add/multiply/replace blend, weights | ✅ | |
| Switch layers + exposure (frame-by-frame, mouth swaps) | ✅ | |
| Clip retime/offset, retarget, mirror L↔R, crossfade bake | ✅ | functions + CLI `mirror`; no NLA UI |
| Procedural walk cycle | ✅ | simple sinusoidal generator, not a pro walk tool |
| Audio lip-sync from WAV | 🟡 | loudness/brightness heuristic → 5 mouth shapes, **not** phoneme recognition |
| Deterministic particle emitter | ✅ | closed-form: any frame evaluated directly |
| Onion skin, motion trails | ✅ | CLI `onion` and `trail`, studio overlays; never exported |
| Audio playback / waveform scrubbing | 🟡 | `waveform` command and studio audio playback; no waveform scrubbing UI claim made |

## Camera & compositing
| Feature | Status | Notes |
|---|---|---|
| 2D camera pan/zoom/rotate (animatable) | ✅ | |
| Multiplane parallax per top-level layer | ✅ | |
| Opacity, blend modes, alpha masks (+invert) | ✅ | |
| True 3D layers / camera, stereo | ❌ | |

## Output & interchange
| Feature | Status | Notes |
|---|---|---|
| SVG frames | ✅ | |
| PNG frames (tiny-skia raster) | ✅ | |
| Animated GIF | ✅ | |
| MP4 | ✅ | requires `ffmpeg` on PATH |
| Self-contained HTML player | ✅ | all frames inline, scrubber |
| Lottie JSON | 🟡 | per-frame baked geometry; gradients → first colour; no masks/blend modes/images; **not validated in a real Lottie player** |
| Import of other tools' files, PSD, FBX, etc. | ❌ | (deliberately: no proprietary formats) |
| Scripting / plugin API | 🟡 | sandboxed Rhai `script` through validated requests, plus expressions; no plugin API |

## Editing
| Feature | Status | Notes |
|---|---|---|
| JSON-patch edit model with undo/redo | ✅ | every edit validated; invalid edits roll back |
| Keyframe set/delete helpers | ✅ | |
| Web studio (browser) | 🟡 | preview, scrubber, per-bone angle keying, JSON editor, undo/redo, save, exports. **No drawing, no point/bone editing on canvas, no graph editor.** |

## Things that are modest "improvements" over the textbook baseline
(not claimed to be absent elsewhere)
- Every project edit is a validated, invertible JSON patch → undo can never corrupt a project.
- All simulation (springs, particles) is random-access deterministic: scrub or render any frame in any order, get identical pixels.
- Any property — including deformer control points and morph weights — is addressable by one `layer::sub::id` path, so expressions and driven curves can wire anything to anything.
- Validation reports *all* problems in one pass with the offending names.
- One engine → SVG, raster, GIF, MP4, HTML, Lottie; the same `evaluate(frame)` feeds all.

## Tests
`cargo test` — 142 unit/integration tests (math, tracks, expressions, rig solvers, deformers,
rendering pixels, exporters, editor undo/redo, audio, particles, booleans, metaballs).

---

# Checklist against the reference product, version by version

Reference: Moho (formerly Anime Studio), 1999 → 14.5 "SHOWTIME" (12 Oct 2026). Feature lists come
from vendor release pages and trade press (see Sources). Gaps in the record: Anime Studio 6–8,
12.5 and 14.1 had no detailed public list that I could find, so they are covered only by the
general history. Status is OSATURA's: ✅ equivalent exists · 🟡 partial / different approach · ❌ missing.

## Early versions (1999 – Anime Studio 8, general history only)
| Feature | OSATURA |
|---|---|
| Particles | ✅ deterministic emitter layer |
| Layer masks | ✅ alpha masks + invert |
| Onion skins | ✅ (CLI `onion`, studio overlay) |
| Bone constraints | ✅ aim / copy-rotation / pin, angle & scale limits |
| Lua scripting | 🟡 (Rhai, not Lua; expression channels too) |
| Lip-sync | 🟡 amplitude heuristic from WAV |
| Physics | 🟡 springs + particle gravity; no rigid bodies |
| 3D creation / import | ❌ |
| Photoshop import | ❌ |
| Wacom support | ❌ (no drawing GUI) |
| Smart bones, IK | 🟡 driven channels / ✅ 2-bone + FABRIK |

## Anime Studio 9 (2013)
| Feature | OSATURA |
|---|---|
| 64-bit, GPU acceleration | 🟡 native 64-bit Rust; CPU rasteriser only (no GPU) |
| Smart Bones (joint-bend control dials) | 🟡 driven channels map any value to any other; no dial UI/actions |
| Editable motion graphs | ❌ (curves are data with cubic-bezier easing; no graph editor) |
| Revamped timeline | ❌ (studio has a scrubber + key marks only) |
| Wacom multi-touch | ❌ |

## Anime Studio 10 (2014)
| Feature | OSATURA |
|---|---|
| Independent (global) angle constraints | 🟡 copy-rotation to a control bone approximates it |
| Squash & stretch | 🟡 bone scale is animatable; no automatic squash/stretch |
| Smart Bone dials / setup | 🟡 see above |
| Bounce / elastic / stagger interpolation | 🟡 bounce, spring (elastic-like), back, cubic-bezier ✅; stagger ❌ |
| Paint bucket, blob brush, eraser, reduce points | ❌ |
| Preloaded characters / library content | ❌ |
| 3D object import | ❌ |

## Anime Studio 11 (2015)
| Feature | OSATURA |
|---|---|
| Frame-by-frame animation | ✅ switch layers + exposure channels |
| Layer referencing (linked duplicates) | ❌ |
| Animated shape ordering | ❌ (draw order is fixed per layer; switch layers can swap) |
| Editable motion curves | 🟡 curve data yes, no curve UI |
| PSD import (incl. live update) | ❌ |
| FBX / Unity export | ❌ |
| Audio import/record + auto lip-sync | 🟡 import WAV + heuristic lip-sync; no recording |
| Merge strokes, Color Points tool | ❌ |

## Moho 12 (2016) and 12.x
| Feature | OSATURA |
|---|---|
| Bézier handles | ✅ |
| Improved freehand tools | ❌ |
| Smart Warp (custom meshes) | 🟡 image grid mesh with bone weights; lattice / spine deformers for vectors; no mesh editor |
| Motion blur | ❌ |
| Pin bones | 🟡 pin constraint, not single-point pin bones |
| Import/export actions between similar rigs | 🟡 `clip`, `retarget`, `mirror` functions on channel sets; no action library UI |
| Animate multiple layers at once | ✅ any number of channels per timeline |
| Redesigned UI, bone anchoring, position auto-freeze | ❌ |

## Moho 13 (2019)
| Feature | OSATURA |
|---|---|
| Bitmap frame-by-frame + onion skin | 🟡 PNG image layers inside switch layers; no onion skin, no bitmap drawing |
| Bitmap drawing tools / custom brushes | ❌ |
| Storyboard & animatic tools | ❌ |
| Actions window | ❌ (data-level clips only) |
| 3D lights, materials, toon shading | ❌ |
| More output presets | 🟡 CLI export formats, no preset manager |
| Bundled characters | ❌ |

## Moho 13.5 (2021) and 13.5.x
| Feature | OSATURA |
|---|---|
| Vitruvian Bones (swap drawings *and* bone sets) | 🟡 switch layers swap drawings; no bone-set swapping |
| Quad meshes (perspective) | 🟡 grid mesh + lattice give free-form warps; no projective quad |
| Wind dynamics (strength, direction, turbulence) | ❌ (springs have no wind; noise channels can fake it) |
| Relative keyframing | ❌ |
| Lip-sync viseme variants chosen at random | ❌ (mapping can merge shapes, no random variants) |
| Export audio as WAV | ❌ |
| Deep-keyframe copy/paste, pivot adjust, timeline markers | ❌ (no GUI) |
| Floating licences | n/a |

## Moho 14 (2023)
| Feature | OSATURA |
|---|---|
| New graphics engine, real-time effects (blur, shadow, noise) | ❌ no blur/shadow/noise filters; blend modes ✅ |
| Liquid Shapes (merge/subtract/intersect/blend) | 🟡 metaball liquid ✅ + polygon booleans ✅; not the same shape model |
| Curvers / Compressible Curvers | 🟡 spine bend deformer is the closest analogue |
| Improved SVG import | ❌ (SVG **export** only) |
| Freehand improvements, new styles, shape merging | ❌ / ❌ / 🟡 (booleans) |
| Multi-Stroke Fill, Connect & Create | ❌ |
| Smart Line Boil, brush options | 🟡 "boil" wobble with adjustable rate ✅; brush options ❌ |
| Point colours, improved Add Point | ❌ |
| Dynamic Bones (+ gravity) | 🟡 damped-spring bones ✅; no gravity term |
| Delayed constraints | 🟡 springs give lag behind a target; no explicit delay constraint |
| New timeline options | ❌ |
| Frame-by-frame improvements | 🟡 |
| Scripting interface update (Lua 5.4) | ❌ |

## Moho 14.1 – 14.2 (2023-11, 2024-06)
| Feature | OSATURA |
|---|---|
| (14.1: no public detail found) | — |
| Animated curve-profile offset | ❌ (width profiles are static) |
| Smart boil for vector noise | 🟡 wobble ticks at a set rate; no "only while moving" mode |
| Reworked Transform Bone tool | ❌ (no canvas tools) |
| Pixelation display | ❌ |
| Colour Burn blend mode | ✅ |
| Lower minimum brush spacing, PSD clipped layers | ❌ |
| Frame-rate-based timeline numbering | ❌ |
| "Turn off masked layers" display option | ❌ |
| Better lip-sync at low frame rates | 🟡 |
| Multilingual names for bones/layers/markers | ✅ (UTF-8 names) |
| Exclude-Strokes mask anti-aliasing, coloured onion skins on images | ❌ |
| Smart Bones controlling bone dynamics | ❌ (spring settings are not animatable) |
| Lua scripts on startup; Windows console flag | ❌ / n/a |
| ProRes alpha export | ❌ (GIF/PNG/MP4 only) |

## Moho 14.3 (2024-12)
| Feature | OSATURA |
|---|---|
| Per-point brush opacity (pressure-driven) | ❌ |
| Colour drift by pen pressure | ❌ |
| Freehand/line-width tool controls | ❌ |

## Moho 14.4 "SIDE QUEST" (2025-11)
| Feature | OSATURA |
|---|---|
| Export to Unreal / Unity / Godot | ❌ |
| glTF export to Blender / 3D tools | 🟡 (`.glb`: frame-0 shapes, bones, morph weights) |
| 360° Smart Bones | ❌ |
| Create Style from Shape | ❌ |
| Particle options (replicate along curve, delay, ordered emission, distribution) | 🟡 start delay ✅, deterministic ordered emission ✅; replicate-along-curve ❌ |
| Layer Selector, Magnet falloff, PSD import, Freehand smoothing, Lua additions | ❌ |
| Easier cycle creation | ✅ loop extrapolation + walk-cycle generator |

## Moho 14.5 "SHOWTIME" (2026-10-12)
| Feature | OSATURA |
|---|---|
| Live Playback (webcam / gamepad / mic, virtual webcam, OBS) | ❌ |
| Face tracking → bones / Smart Bones / mesh | 🟡 (synthetic-trained detector, `face-bake`; not validated on real faces) |
| Live recording to timeline, multi-character control | ❌ |
| Video tracking (image sequences) | ❌ |
| Freehand algorithms, straight lines, trim tails | ❌ |
| Remade Paint Bucket (Vector / Trace / Recolor) | ❌ |
| Magnet reverse move, Select Points shape selection | ❌ |
| Custom brush categories | ❌ |
| Onion skin colour/opacity, Shift & Trace, magnetic onion skins | ❌ |
| Layers window filters / multi-select | ❌ |
| Performance: faster undo/redo, lighter files | 🟡 undo is a cheap JSON-patch inverse; files are plain JSON |
| MP4/H.264 export | ✅ via ffmpeg |
| PNG colour profiles, EXR import/export | ❌ |
| Batch exporter, `-exr` / `-pngprofile` CLI flags | 🟡 `frames` / `export` CLI, no batch queue |
| Frame rate with 3 decimals (e.g. 23.976) | ✅ |
| SVG export handles variable-width strokes & liquid shapes | ✅ (strokes become outlines, liquids become paths) |
| Liquid hide-edge, stroke-only liquid shapes, stroke exposure | ❌ |
| Linux via Wine/Proton | ✅ native Rust (Linux/macOS/Windows); prebuilt Windows x86-64, macOS arm64 and Linux x86-64 archives in the draft pre-release; tested by CI on all three, by a person only on Linux |

## Roll-up (reference product only; competitors follow below)
Of the reference product's documented features, OSATURA covers the **rigging, deformation,
procedural-animation, compositing-math and export** core (roughly: skeleton/IK/constraints,
mesh & lattice deformation, keyframes & expressions, masks/blends/gradients, particles,
metaballs/booleans, GIF/MP4/HTML/Lottie), but it has **no drawing tools, no live input, no 3D,
no PSD/FBX I/O** (at the time of the first survey; glTF export, EXR output, Rhai scripting, a graph/timeline studio, onion skin and blur effects have been added since, see `ROADMAP.md` section 9).
Those are the large, mostly GUI-bound blocks of the product and are the honest gap.

## Sources
- Moho 14.5 SHOWTIME announcement, moho.lostmarble.com (blog, 12 Oct 2026)
- Moho Pro feature page, mohoanimation.com/pages/features
- Moho 14.4 SIDE QUEST appcast, lostmarble.com/appcasts/Moho14.4Update.html
- Moho 14.3: Digital Production, 27 Dec 2024
- Moho 14.2 release post, moho.lostmarble.com; "Moho 14: What is new?" tutorials list
- Moho Pro 13.5 new features, moho.lostmarble.com; Lester Banks (Apr 2021); 13.5.3 notes
- Moho 13: Lester Banks (Jun 2019)
- Moho Pro 12: 80.lv; Digital Production (Nov 2016)
- Anime Studio 11: Architosh (May 2015); 10: Architosh (Mar 2014); 9: Renderosity
- Version dates: mohoscripting.com/moho_versions; general history: Wikipedia "Moho (software)"

---

# Addendum: Moho / Anime Studio history, versions 1 – 11 (filled in)

Source: the vendor-hosted version history PDF (lostmarble.com, "Anime Studio history", 2015) plus the
Anime Studio 7 and 8 press pages. It supersedes the "general history only" rows near the top of the
reference checklist. Not found even here: details of 8.x, 10.1.x, 11.1, 12.5, 14.1.

| Version | Documented features | OSATURA |
|---|---|---|
| Moho 1 (1999) | BeOS, Windows port, Flash output | ❌ Flash/SWF (obsolete) |
| 2.x | QuickTime I/O, renders to 4096², layer shading, switch layers, colour swatches in styles, flexible bone binding, improved Illustrator import | ✅ switch layers, 🟡 weight binding, ❌ Illustrator import, ✅ large raster scales (≤16×) |
| 3.x | Particles, image wrapping, layer masks, onion skins, virtual camera with depth, image warping, enhanced switch layers, tablet support, SMPTE timecode, hide offscreen artwork, reversible masks, line-thickness randomiser, safe zones, layer shear | ✅ particles, masks (+invert), camera, image mesh warp, layer skew; ❌ onion skin, tablet, timecode, safe zones, thickness randomiser |
| 4.x | Graphical animation curves, bone constraints, batch rendering, audio, animatable parameters, animation cycling, keyframe scaling, resizable projects, particle depth, randomised crowds, auto flexible binding, Linux release | ✅ constraints, cycles (loop extrapolation), animatable params, resizable projects, `clip` retime ≈ keyframe scaling, Linux; 🟡 crowds (particles), 🟡 audio (lip-sync only); ❌ curve editor UI |
| 5.x | New UI, SWF export, 3D and note layers, actions, styles, Lua scripting | ❌ 3D / note layers / Lua; 🟡 actions (clip functions); ✅ styles are per-path paint |
| Anime Studio 6 | Motion tracking, integrated lip-sync, scatter brush, Lua 5.1, auto shading, **morph targets**, sequencer, HD video | ✅ morph targets; 🟡 lip-sync; ❌ motion tracking, scatter brush, auto shading, sequencer |
| 6.1 / 6.2 | Facebook upload, transparent GIF import | ❌ / 🟡 GIF export ✅, import ❌ |
| Anime Studio 7 | Simulated physics (gravity, density, friction, springiness), 3D shape design, audio recording, content library, Poser scene import, **follow path tool**, gather media | 🟡 springs/particle gravity; ❌ rigid physics, 3D, recording, library, Poser, follow-path |
| Anime Studio 8 | Automatic image tracing, Character Wizard, Beginner's mode, real-time media connection, global rendering styles, advanced bone rigging | ❌ all (no tracing, wizard, live link, global styles) |
| Anime Studio 9 / 9.5 | Editable motion graphs, Smart Bones, **layer comps**, improved onion skins, 64-bit + GPU, enhanced keyframing, coloured timeline; 9.5: Smart Bone enhancements, depth shifting, nested layer controls, flexi-binding, switch-layer improvements | see AS 9 table; ❌ layer comps, depth shifting |
| Anime Studio 10 / 10.1 | New constraints, Smart Bone setup, bounce/elastic/stagger, separate render process, combined bone+point tools, animation preview, multi-document, shortcut editor, multi-shape selection, frame zero, point/bone hiding, multi-layer editing, **variable-width curves**, texture transparency, adjustable particle source, depth of field, fractional blur; 10.1: auto image trace, global styles, patch layers, PSD import, real-time media, Character Wizard | ✅ variable-width strokes, ✅ multi-layer animation; 🟡 depth of field ❌ blur ❌; ❌ patch layers, others as above |
| Anime Studio 11 | Frame-by-frame, layer referencing, animated shape ordering, **animated bone targets**, **animated bone parenting**, bone flipping, selection-layer grouping, normalised layer scale, batch export, new file format | ✅ FbF, ✅ animated IK target (target is just a bone), ❌ animated *re-parenting*, 🟡 bone flipping (`mirror`), ✅ batch via CLI, ✅ plain-JSON file format |

**New gaps surfaced here** (add to the roadmap): layer comps (saved layer-visibility states),
animated bone re-parenting, patch layers, depth shifting, follow-path layers, safe-zone overlays,
SMPTE timecode, auto shading, scatter brush, image auto-trace.

---

# Competitor features not covered by the checklist above

Products surveyed: Toon Boom Harmony 24, Spine 4.2, Live2D Cubism 5.x, Cartoon Animator 5.3, Rive,
Synfig Studio, OpenToonz, TVPaint Animation 12, Adobe Animate, Cavalry, Procreate Dreams, Duik (After
Effects), Blender Grease Pencil. Items below are the ones **not already listed** in the reference
checklist. Source quality varies: most rows come from vendor feature/release pages I fetched;
rows marked † are from general knowledge or a title/ToC only and were not confirmed on a page.
Status is OSATURA's: ✅ · 🟡 · ❌.

## Rigging, constraints, deformation
| Feature | Seen in | OSATURA |
|---|---|---|
| Weight tools: weld weights across meshes, copy/paste weights between vertices | Spine 4.2 | ❌ (auto-skin only, no painting) |
| Bone physics constraints (inertia, wind, gravity, mix) | Spine 4.2 | 🟡 damped springs (frequency/damping); no wind/gravity/inertia |
| Path constraint: bones follow a composite Bézier spline | Spine, Rive (Follow Path) | ❌ |
| Constraint mixes, duplication with bones, multi-edit, folders | Spine 4.2 | 🟡 IK mix & constraint weight animatable; no folders/multi-edit |
| Skins (swap attachment sets on one skeleton) | Spine, Rive solos | 🟡 switch layers |
| Multi-bone IK "pose tool" (drag a limb end) | Spine | ❌ (needs canvas GUI) |
| Bounding-box / hit-test attachments | Spine | ❌ (not needed for offline output) |
| Free-Form Deformation, mesh weights | Spine, Harmony | ✅ lattice + weighted image mesh |
| Deformer chain: Bone, Game Bone, Curve, Envelope, Free-Form, Mesh Warp | Harmony | 🟡 bone ✅ curve (spine) ✅ free-form ✅ mesh warp ✅; envelope/cage ❌; game-bone ❌ |
| Shape-Aware Deformation (points, bones, cages) | Harmony 24 | ❌ |
| Baking of all deformation types | Harmony 24 | 🟡 springs baked; geometry baked per frame in Lottie export |
| Master controller / transformation selector for deform groups | Harmony 24 | ❌ |
| Pegs and layer hierarchy | Harmony, Animate | ✅ groups, bone-attached layers, parenting |
| Auto-muscle, auto foot-roll, kinematic output † | Harmony | ❌ |
| Rotation / warp deformers + keyform parameters, glue | Live2D | 🟡 parameters ≈ driven channels, warp ≈ lattice; glue ❌ |
| Blend shapes driving rotation, opacity, draw order, multiply/screen colour | Live2D 5.0 | 🟡 morph weights drive geometry only |
| Auto-mesh with even density control for thin shapes | Live2D 5.0 | ❌ (regular grid only) |
| Warp deformer auto-resizes to contain child keyform | Live2D 5.3 | ❌ |
| Deform brush (square/line/expansion shapes) | Live2D 5.3 | ❌ |
| AI-assisted face deformer / facial motion generation | Live2D 5.0 | ❌ |
| Joysticks (2-D control mapped to animations) | Rive | 🟡 driven channel maps one input; no 2-input joystick |
| Constraints between arbitrary objects (translate/rotate/scale/distance) | Rive | 🟡 aim, copy-rotation, pin only |
| Bitmap rigging with automatic in-betweens | TVPaint 12 | 🟡 image mesh ✅; auto in-betweens ❌ |
| Plastic tool (mesh-based rigging) | OpenToonz | 🟡 image mesh + weights |
| Auto-rigging, controllers, spring dynamics | Duik † | 🟡 springs ✅; auto-rig ❌ |

## Animation, timeline, interactivity
| Feature | Seen in | OSATURA |
|---|---|---|
| Animation mixing: blend/crossfade concurrent animations at runtime | Spine | 🟡 Add/Multiply modes + weights; baked `crossfade`; no runtime transitions |
| Dopesheet + graph editor + runtime preview | Spine, Rive, OpenToonz | ❌ (data supports it; no UI) |
| State machine: inputs, listeners, transitions with conditions, blend states, layers | Rive | ❌ |
| Events, audio triggers, dynamic assets | Rive | ❌ |
| Data binding (design elements ↔ live data) | Rive | ❌ (expressions can read channels, not external data) |
| Embedded scripting (Luau, JavaScript, ToonzScript) | Rive, Cavalry, OpenToonz | ❌ |
| Responsive layouts / components / libraries | Rive | ❌ |
| Trim Paths, vector feathering, N-slicing | Rive | ❌ |
| Resolution-independent timing, vector tweening | Synfig | ✅ |
| Cross-layer parameter linking via expressions | Synfig | ✅ |
| Procedural node graph (generators, transformers, effectors), duplicators | Cavalry | ❌ (no node graph; no layer instancing) |
| Data-driven animation from JSON/CSV | Cavalry | ❌ |
| Performing: record live gestures as keyframes | Procreate Dreams | ❌ |
| Puppet-stage trigger hotkeys, motion pilot (cursor-driven walks) | Cartoon Animator 5.3 | ❌ |
| Camera switcher: saved cameras, hotkey cuts, camera-switch track | Cartoon Animator 5.3 | 🟡 one camera; cuts can be faked with step keys |
| Cinematic moves (dolly zoom, tracking a target) | Cartoon Animator 5.3 | 🟡 animatable camera; no target tracking |
| Facial mocap (webcam / iPhone), head-driven body, mic volume | Cartoon Animator, Live2D | ❌ (mic volume only via offline WAV) |
| Livestream mode (OBS/Twitch, Stream Deck mapping) | Cartoon Animator 5.3 | ❌ |
| Motion Sync lip-sync generating timeline keys | Live2D 5.0 | 🟡 heuristic WAV → exposure keys |
| Rhubarb lip-sync integration | OpenToonz | ❌ (heuristic only) |
| Root motion extraction (delta compensation) | Spine 4.2 | 🟡 loop-offset extrapolation |
| Looping-curve handle sync | Spine 4.2 | ✅ loop extrapolation (no handle UI) |
| Morphing with morph hints | Harmony | 🟡 morph targets + path lerp; no hints |
| Key-based 2D camera with timeline timing | TVPaint 12 | ✅ |

## Drawing, painting, bitmap tools
| Feature | Seen in | OSATURA |
|---|---|---|
| Brush: randomisation (stamp count, offset), dual papers, multitextured brushes | Harmony 24, TVPaint 12 | ❌ |
| Pencil opacity by pressure/tilt/speed/taper/fade | Harmony 24 | ❌ |
| Symmetry guides (vertical, horizontal, dual) | Harmony 24 | ❌ |
| Cutter tool (lasso, trim), contour editor, close-gap | Harmony | ❌ |
| Drawing guides, perspective/ruler guides † | Harmony | ❌ |
| Paperless animation: onion skin, flipping, light table | Harmony, Procreate Dreams, TVPaint | 🟡 (onion skin yes; no drawing tools or light table) |
| Cleanup of scanned drawings, scanning, camera capture, stop-motion | OpenToonz | ❌ |
| Selection brush, image library | TVPaint 12 | ❌ |
| Sharpening on transform | TVPaint 12 | n/a |
| Pen-pressure sensitivity | Synfig | ❌ |
| Path-based gradient shading | Synfig | ❌ |
| Text with variable/dynamic runtime content, any language | Rive, Procreate Dreams | ❌ |
| Custom fonts, titles, credits | Procreate Dreams | ❌ |
| Palettes and styles management | OpenToonz | ❌ |
| >50 layer types (fractals, filters, distortions) | Synfig | ❌ (≈7 layer kinds) |

## Effects and compositing
| Feature | Seen in | OSATURA |
|---|---|---|
| Node-view compositing network with named ports | Harmony 24 | ❌ (layer tree + masks/blends) |
| Mattes, groups under composite | Harmony | ✅ masks, isolated groups |
| CryptoMatte read/write | Harmony 24 | ❌ |
| Toon shader (outline, posterise) | Harmony 24 | ❌ |
| Unsharp mask, channel swap | Harmony 24 | ❌ |
| Blur family (radial, spin, directional), bokeh, fractal noise, tile | OpenToonz | ❌ |
| Particles FX | OpenToonz | ✅ emitter layer |
| 15 RGB + 5 alpha blend modes, per-part offscreen drawing | Live2D 5.3 | 🟡 13 blend modes; isolated-group rendering ✅ |
| Clipping (per-part clip IDs) | Live2D, Rive | ✅ alpha masks |
| HDR / HDRI imaging | Synfig | ❌ |
| Video import, ProRes, rotoscoping over footage | Procreate Dreams | ❌ |
| Audio engine (voice-over, music, SFX) | Procreate Dreams | ❌ |

## 3D, pipeline, interchange, runtimes
| Feature | Seen in | OSATURA |
|---|---|---|
| 3D scene, FBX bone posing, Vulkan scene graph, perspective-plane drawing | Harmony 24 | ❌ |
| External rendering via Maya / Blender | Harmony 24 | ❌ |
| Vector import: PDF / AI / SVG; multi-layer EXR; DPX alpha; OpenColorIO (ACES) | Harmony 24 | ❌ |
| PSD import without Photoshop | Spine 4.2, Rive | ❌ |
| Lottie / Figma paste import | Rive | ❌ (Lottie **export** only) |
| Texture packer / atlases | Spine | ❌ |
| JSON/binary skeleton export + 19 official / 40+ community runtimes | Spine | 🟡 own JSON; no runtimes |
| Open-source runtimes (Web, Apple, Android, React, Flutter, C++, Unity, Unreal) | Rive | ❌ |
| Pixels mode (snap sprites to screen pixels) | Spine 4.2 | ❌ |
| Export: WebM with transparency | Live2D 5.3 | ❌ |
| Export: SVG/PDF, PNG/EXR sequences, 16/32-bit depth, batch, ProRes | Cavalry | 🟡 SVG, PNG ✅; PDF/EXR/ProRes/high bit depth ❌ |
| Cloud rendering, share links, embeds | Rive | ❌ |
| Distributed rendering (Toonz Farm), version control | OpenToonz | ❌ |
| PDF / TVPaint JSON export | OpenToonz | ❌ |
| Real-time team collaboration, community remixing | Rive | ❌ |
| HTML5 Canvas / interactive web export | Animate † | 🟡 HTML player (pre-rendered, not interactive) |
| Native Linux/macOS/Windows | Synfig, OpenToonz | ✅ |
| Open-source licence | Synfig, OpenToonz | ✅ MIT/Apache-2.0 |
| Layer parenting view, symbols/library instances † | Animate | 🟡 parenting ✅; no instance library |
| Modular detachable UI, custom workspaces, dark/light theme | TVPaint 12, Live2D 5.0 | ❌ |

## Candidate roadmap (ordered by value ÷ effort, no GUI needed first)
1. Trim paths + path-following (bones/layers along a Bézier) — Rive, Spine
2. Onion skin + motion-trail overlays — nearly every 2D tool
3. SVG import (paths, gradients) — Moho 14, Harmony, Synfig
4. Wind / gravity terms for springs, bone dynamics driven by Smart-Bone-style inputs
5. Blur, drop shadow, glow, outline, posterise (layer filters)
6. Layer instancing / duplicators (clone N along a path, with per-copy offsets)
7. Interactive state machine in the HTML/JSON runtime (inputs, transitions, blend)
8. Envelope (cage) deformer, weight copy/weld/smooth tools, even-density auto-mesh
9. Camera cuts / saved cameras, target-tracking camera
10. glTF export of the rig; EXR/PDF/WebM-alpha export
11. Text layers (needs font loading) with path-following
12. Studio GUI: canvas point/bone editing, graph editor, onion skin

## Additional sources (competitors)
- Toon Boom Harmony 24 release notes and Premium Getting Started Guide (docs.toonboom.com)
- Spine 4.2 "The physics revolution" and "What is Spine?" (esotericsoftware.com)
- Live2D Cubism 5.0 and 5.3 "New Features" (docs.live2d.com)
- Cartoon Animator 5.3 feature highlights (discussions.reallusion.com)
- Rive features page (rive.app/features)
- Synfig Studio features (zoftwarehub.com); OpenToonz "What's New" docs
- TVPaint Animation 12 (tvpaint.com); Cavalry review (superrendersfarm.com)
- Adobe Animate "Layer parenting" (adobe.com/learn); Procreate Dreams (Cartoon Brew)
- Duik: Lester Banks tag page (title only †); Blender Grease Pencil: manual page not retrievable (not covered)

---

# Addendum — Competitor version histories (round 2, 2026-10-19)

Legend as above: ✅ done, 🟡 partial, ❌ not yet. Only features confirmed on vendor/project pages or press are listed. Pirate sites were not used; web.archive.org is not reachable from this environment, so some older releases remain undocumented (see "Known gaps").

## Toon Boom Harmony 20 (vendor release notes)
| Feature | OSATURA |
|---|---|
| Centerline editor: point deletion, smoothness option | ❌ |
| Textured pencil lines → textured brush strokes; pencil-texture eyedropper | ❌ |
| Curved / S-shaped Line tool; Auto-Fill brush; Repaint brush | ❌ |
| Alignment guides, snapping limited to selected layers | ❌ |
| Quick Ease / Ease In / Ease Out on selected keyframes | 🟡 easing per key ✅; batch ease commands ❌ |
| Scene markers (no-duration, ripple edit) | ❌ |
| Drawing substitution view (rename/filter) | 🟡 Switch layers ✅ |
| Surface shading nodes: ambient occlusion, cast shadow, surface map/normal | ❌ |
| **Weighted Deform node** | 🟡 weighted image mesh ✅ |
| Point Kinematic Output node (1/2/3-point sampling) | ❌ |
| Pre-render cache node | 🟡 spring bake cache ✅ |
| Colour fade / curves / levels / gamma nodes | ❌ |
| Planar-region particle shape using a vector drawing | 🟡 emitter ✅; custom shape ❌ |
| Master controllers: deformer-on-deformer, Function Wizard, Slider Wizard | 🟡 channels ✅; wizard UIs ❌ |
| Python scripting (PythonManager) | 🟡 JSON patch ops + CLI; no embedded scripting |
| Colour-space management, 16-bit rendering, scripted post-process movie | ❌ (ffmpeg post-process by hand) |
| Control Center: batch scene copy/import/run-script, render queue | ❌ |

## OpenToonz 1.7
| Feature | OSATURA |
|---|---|
| Straight-line mode for brush | ❌ |
| GIMP + MyPaint brush support | ❌ |
| Selection+Freehand fill mode | ❌ |
| Clipboard image paste | ❌ |
| Pre-production dashboard / storyboard | ❌ |
| Custom panels, translucent main window, rotate/flip viewer | ❌ |
| Xsheet scale + tag navigation | 🟡 timeline data ✅; no GUI |
| Export with camera-movement info | 🟡 camera in Frame ✅ |
| **OCA (Open Cel Animation) export** | ❌ |
| MOV and APNG export | 🟡 MP4/GIF ✅; APNG/MOV ❌ |
| Batch TZP→TLV conversion | ❌ |
| Floor Bump Iwa, Flow Fx; float + linear-colour rendering | ❌ |
| Haiku OS support | n/a |

## Synfig 1.4.2 (and 1.4.0 from round 1)
| Feature | OSATURA |
|---|---|
| Lottie export incl. advanced outline and blur layers, gamma | 🟡 Lottie export ✅ (shapes/transform); blur ❌ |
| Keyboard shortcut editor, single-key tool shortcuts | ❌ |
| Editable TimeLoop in workspace | 🟡 loop clips ✅ |
| Skeleton / Skeleton Deformation layer duplication with bone links | ✅ (bones are project data) |
| SVG importer handling style attributes | ❌ (no SVG import) |

## Items from round 1 (vendor pages, abbreviated)
- **Spine 4.2/4.3**: physics constraints, sliders, constraint mixing, pixel snapping — springs ✅ (baked), sliders/mix 🟡 (IK `mix` ✅), pixels mode ❌.
- **Live2D Cubism 4.0–5.3**: ArtPath, onion skin, WebM-alpha export — ArtPath ❌; onion skin and WebM-alpha ✅.
- **Harmony 21/22**: weighted deformers, line-boil textures — 🟡 / ✅ (wobble).
- **Cartoon Animator 5** — see competitor tables above.

## New roadmap candidates from this round
13. OCA (Open Cel Animation) and APNG export — OpenToonz 1.7
14. Batch ease / "quick ease" commands in the editor — Harmony 20
15. Colour-correction filters (fade, levels, curves, gamma) — Harmony 20
16. Surface shading (cast shadow / AO) for vector layers — Harmony 20
17. Embedded scripting (Python/Lua) over the patch-op API — Harmony 20
18. Pre-render cache node / cached evaluation — Harmony 20
19. Scene markers + ripple editing on the timeline — Harmony 20
20. Layer comps, animated bone re-parenting, patch layers, follow-path, SMPTE timecode, safe zones, auto-trace, scatter brush (gaps found in Moho research)

## Known gaps (not retrievable from accessible sources)
Anime Studio 8.x, 10.1.x, 11.1; Moho 12.5 and 14.1 details; Spine 3.0–4.1 (3.7 blog is a video page only); Harmony 17, 23, 25–27 (docs pages errored); Synfig 1.4.1; TVPaint 11; Cartoon Animator 4/5.1/5.2; Rive changelog.

## Sources (round 2)
- docs.toonboom.com — Harmony 20 release notes
- en.ubunlog.com — OpenToonz 1.7 article
- synfig.readthedocs.io — Synfig 1.4.2 release notes
- esotericsoftware.com — Spine 3.7 features-rundown (video page, no list)

---

# Addendum — YouTube sources (round 3, 2026-10-19)

WebSearch restricted to youtube.com returns video titles; WebFetch returns the **description/chapter list** of each video (no transcripts). Only features named there are listed; "new in version X" is as stated by the uploader (often the vendor).

## Moho 14.1 (video description)
| Feature | OSATURA |
|---|---|
| Show/hide handles in Curvature tool | ❌ (no GUI canvas) |
| Shaded and Halo shape effects visible in edit mode | 🟡 gradients ✅; halo/shade effects ❌ |
| Live Project Settings pixelation effect | ❌ |
| Preview Animation: higher resolution, hidden output border | 🟡 raster preview ✅ |
| Modeless dialogs for Lua scripts | ❌ (no scripting) |
| PSD "Linear Dodge (Add)" blend mode; updated blend modes | 🟡 check blend list; add/dodge ❌ |
| Unsaved-document indicator in title bar | ✅ studio tracks undo state |
| Fixes: smart-bone evaluation, sub-frame motion blur + bone dynamics, zero-length Bézier handles | n/a |

## Moho 12.5
Title/description pages gave no feature list (video outdated; free 12.5 download withdrawn). Related titles mention Smart Bones tutorials, Smart Warp 2.5D rigs and "Mynd Tools" install — Smart Bones 🟡 (channels/morphs), Smart Warp 🟡 (FFD lattice ✅).

## Spine
| Version | Feature | OSATURA |
|---|---|---|
| 3.7 | Audio support | ❌ (WAV analysed for lip-sync only) |
| 3.7 | Revamped exports | 🟡 |
| 3.7 | Type-to-search | ❌ |
| 3.7 | Stretchy / compressed / uniform IK | ❌ (IK is length-preserving) |
| 3.7 | Combining skins, duplicating skins, placeholders | 🟡 Switch layers |
| 3.7 | Vertex copy/paste | ❌ |
| 3.7 | Pixel grid rendering | ❌ |
| 3.7 | Additive animation blending | 🟡 additive morphs only |
| 3.7 | spine-cpp runtime; web player | ❌ / 🟡 HTML player |
| 3.8 | Mesh tracing | 🟡 auto grid mesh; no image tracing |
| 3.8 | Polygonal texture packing | ❌ |
| 3.8 | **Soft IK** | ❌ |
| 3.8 | Skin bones and constraints | ❌ |
| 3.8 | Mix-and-match project | ❌ |
| 3.8 | Folders, rulers, enhanced PSD export, welcome screen | ❌ |
| 3.8 | Smaller .json/.skel; binary loading in spine-ts | ❌ (JSON only) |
| 4.0 beta | **Graph / curve editor** | 🟡 per-key easing ✅; no editor |
| 4.1 beta | Sequence (image-sequence attachment) function | ❌ |

## Toon Boom Harmony 25 / 25.2
| Feature | OSATURA |
|---|---|
| Pencil Retouch tool | ❌ |
| Import Photoshop brushes | ❌ |
| **Breakdown Assistant for cut-out animation** | ❌ (walk-cycle generator is related) |
| Lock timeline ruler; action-template preferences | ❌ |
| Better image, 3D and USDZ file support | ❌ |
| Art Layer Manager (swap, clear, merge, duplicate art layers; 25.2+) | ❌ (layer duplicate via patch ops ✅) |
| Master Controller Grid Wizard | ❌ |

## Cartoon Animator 5 series
| Feature | OSATURA |
|---|---|
| 5.1 PSD and vector round-trip editing | ❌ |
| 5.1 Image-sequence and GIF import | ❌ |
| 5.2 **Motion Pilot** (puppeteer motion by dragging) | ❌ |
| 5.2 Motion Path animation / path editing | ❌ (roadmap #1) |
| Spring dynamics, FFD, vector animation (existing) | ✅ / ✅ / ✅ |

## New roadmap candidates from this round
21. Soft IK + stretchy IK modes — Spine 3.8/3.7
22. Image-sequence layer (frame list attachment) and GIF/APNG import — Spine 4.1, CA 5.1
23. Pixelation / pixel-snap output filter — Moho 14.1, Spine 4.2
24. Motion Pilot-style recording: capture live drag of a bone/handle into keys — CA 5.2
25. Breakdown assistant (auto in-betweens with arcs for cut-out rigs) — Harmony 25
26. Mesh tracing from alpha outline — Spine 3.8
27. Skin sets (swap groups of attachments together) — Spine 3.7/3.8
28. Audio track in project + playback sync (not just analysis) — Spine 3.7
29. Lua/Python scripting hooks — Moho, Harmony 20

## Sources (round 3, YouTube)
- Moho 14.1: A Complete Breakdown — youtube.com/watch?v=QT4AiV1wO1k; Moho 14.1 Update — watch?v=FCiBKBjDaNg
- Spine 3.7 features rundown — watch?v=Fi0ks2LeYSM; Spine 3.8 features rundown — watch?v=kU26B5465SQ; Spine v4-beta curve editor — watch?v=37CeFRzrjqc
- Harmony 25 changes — watch?v=kJ58C2DNIk8; Art Layer Manager 25.2+ — watch?v=AZc-8zHucOo
- Cartoon Animator 5.2 Motion Pilot — watch?v=Eu7vO2uGGN4; 5.1 sequence/GIF import — watch?v=O8EGwVwqgfs

---

# Addendum — Exhaustive gap search (round 4, 2026-10-19)

Sources: Synfig/Rive official release notes, YouTube titles/descriptions. Only named features are listed.

## Anime Studio 8 (YouTube "Pro Feature" series, titles + descriptions)
| Feature | OSATURA |
|---|---|
| Automatic image tracing (bitmap → editable vector) | ❌ (roadmap #20) |
| Character Wizard (guided character creation) | 🟡 demo rigs/walk generator; no wizard |
| Import Photoshop (PSD) files | ❌ |
| Real-Time Media Connection (live link to external media) | ❌ |
| Scale Points tool | 🟡 patch ops on points |
| 1080p HD project presets | ✅ any size |
Anime Studio 10.1 / 11.1: no accessible source found.

## Synfig 1.4.1 – 1.4.4
| Version | Feature | OSATURA |
|---|---|---|
| 1.4.1 | Sound layers mixed into exported video | ❌ (no audio track) |
| 1.4.1 | Duplicate-layer copies all selected layers | ✅ (batch ops) |
| 1.4.3 | Bone-link prompt: link transformation instead of origin | ✅ |
| 1.4.3 | Papagayo (.pgo) import; SVG import fixes; image-sequence framerate | ❌ / ❌ / ❌ |
| 1.4.3 | GIF render fixes, FFmpeg 3.x compat | ✅ GIF/MP4 |
| 1.4.3 | Autosave (down to 15 s) | ❌ (explicit save) |
| 1.4.4 | Last-frame export fix | ✅ covered by tests |

## Rive (official release notes, Sept–Oct 2026) — the most active competitor
| Feature | OSATURA |
|---|---|
| String Template converter (text from view-model values, number formatting) | ❌ |
| Dashed strokes (Dash Effect), imported from SVG | ❌ |
| Hold Q/W/E + drag to move/rotate/scale | ❌ (no GUI) |
| Brush shapes keyable in Animate mode | ❌ |
| Constraint keyframe copy/paste; converters keep formulas | 🟡 constraints ✅, copy ❌ |
| Relative data binding; view models; Blob properties | ❌ |
| Luau scripting (fetch, text shaping, GPU canvas, shaders/WGSL, instanced image meshes) | ❌ |
| Blend states / Mix-by-view-model timeline | 🟡 crossfade bake |
| Keyboard, focus, gamepad listeners; Text Input; accessibility semantics | ❌ |
| Grid and Stack layouts, virtualized lists, scroll constraints & scroll physics | ❌ |
| Set Parent (Alt+P) keeping position and draw order | 🟡 re-parent via patch ops; keep-world ❌ |
| Align to Key Object; root-bone offset; Scroll to selection | ❌ |
| Additive strength for fills/strokes; separate RGBA channels keyable/bindable | 🟡 additive morphs; RGBA keyed ✅ |
| Inner/outer strokes (early access) | ❌ |
| Auto Trace (better for pixel art) | ❌ |
| Flash `.fla`/XFL → Rive conversion | ❌ |
| Render presets: state machine → video | 🟡 clip → MP4/GIF |
| Free-plan `.riv` export; CLI (`rive push/publish/ls/docs`), VS Code extension, headless input simulation | 🟡 CLI ✅ (own commands) |
| AI Agent in editor | ❌ |
| Android preview; React Native / Apple / Android / Flutter runtimes | ❌ |
| Revision history & restore, offline sync, undo that keeps zoom | 🟡 undo/redo ✅ |

## TVPaint Animation 11 (YouTube descriptions)
Shortcut creation, removable under-drawings, line smoothing, drawing recording, KeyFramer pivot-point click-and-drag (11.7.1), Sprite Generator, Inlay Texture FX, guidelines, "end of update releases" video — all ❌ (raster paint tools; OSATURA is vector/rig).

## Toon Boom Harmony 17 (YouTube)
Free Form Deformer 🟡 (FFD lattice ✅), pumped-up Master Controllers 🟡, RGB Difference Keyer ❌, Motion Blur ❌ (roadmap #5).

## Cartoon Animator 4 / Spine 3.6 / Harmony 22–23
Videos found (CA 4: Rokoko mocap plugin, 3D-motion-to-2D update, normal-map & lighting in 4.5; Spine 3.4 paths; Spine 3.6 feature video; Harmony 22 drawing tools) but descriptions list no features → not claimed. Notable by title only: **motion-capture retargeting** (CA 4) ❌, **normal maps/lighting** (CA 4.5) ❌, **path constraints** (Spine 3.4) ❌.

## New roadmap candidates from this round
30. Dash effect + inner/outer strokes
31. Embedded scripting runtime (Lua/Luau) with GPU-free API
32. Data-binding / view models + State-machine inputs (keyboard/pointer/focus) for the HTML player
33. Scroll constraints / layout containers (grid, stack) for UI-style animation
34. Papagayo (.pgo) lip-sync import; Rhubarb-style import
35. Set Parent keeping world transform
36. Autosave + revision history in the studio
37. Motion Blur and RGB keyer filters
38. Path constraints (bone along Bézier) — Spine 3.4, merges with roadmap #1
39. Motion-capture retargeting to 2D rig (BVH/JSON) — CA 4
40. Normal-map lighting for image layers — CA 4.5
41. Sound track mixed into MP4 export — Synfig 1.4.1
42. .fla / XFL and PSD import

## Still not retrievable
Anime Studio 10.1 / 11.1, Moho 12.5 detailed notes, Spine 3.0–3.6 detail pages, Harmony 23, Cartoon Animator 4.x detailed notes, TVPaint 11 change-log text.

## Sources (round 4)
- synfig.readthedocs.io — release notes 1.4.1, 1.4.3, 1.4.4
- releasebot.io/updates/rive — Rive Editor 0.9–0.9.272, CLI 1.2–1.6
- YouTube: Anime Studio 8 Pro Feature series (q_HH2zOE-Yk, CUemDIlPH6Y, 408CLy652mU, l9eXLUO9Dus); TVPaint 11 demo (MoqNrCAUy5k); Harmony 17 (1CU_3RJBcl4); Cartoon Animator 4 playlist

---

# Addendum — Social networks and community channels (round 5, 2026-10-19)

**Access reality:** X/Twitter pages are blocked to fetching by robots.txt (only search-result snippets/titles are visible); Facebook likewise; Reddit searches with a domain filter were rejected by the search proxy and unfiltered searches returned no Reddit threads; the Lost Marble forum serves a JavaScript redirect shell with no readable content. Nothing was circumvented. What was readable:

## Moho official channels (moho.lostmarble.com news, X snippets)
| Item | Notes | OSATURA |
|---|---|---|
| Moho 13.5.3 (Apr 2022) | improved automatic lip-sync, better interface, less intrusive bones | ✅ lip-sync (WAV heuristic) |
| Moho 13.5.5 (X, May 2022) | **Relative Paths** for linked assets | ✅ asset dir is relative |
| Moho 14.3 "Mantis Shrimp" (Dec 2024) | brush-focused update | ❌ |
| Moho 14.4 "Side Quest" (Nov 2025) | export to game engines and 3D software | 🟡 Lottie/own JSON; glTF/FBX ❌ |
| Moho 14.5 "Showtime" (Oct 2026) | magnetic onion skins, Shift & Trace, Liquid Shapes options, EXR/PNG with colour profiles, live-playback mode for streaming/controlling characters in real time | see main matrix; live mode ❌ |
| Ambassador Program (Jun 2026); official stance: no AI tools planned (Mar 2025) | policy | n/a — OSATURA also has no AI features |
| Webinars (Víctor Paredes, etc.) | tutorials | n/a |

## Community comparisons (Spine forum, "Spine vs Moho 12")
- Users praise in Moho: built-in physics, layer masking, smart bones, vector editing.
- Users criticise: FFD considered weak; **no real runtime**, game export limited to FBX.
- Creature praised for integrated motion capture (FFD criticised).
- Spine 4.2 physics constraints get enthusiastic reactions on X (user posts).
OSATURA: physics ✅ (baked springs), masking 🟡 (clip via booleans), smart bones 🟡, vector editing ✅, runtime ❌, mocap ❌.

## Forum wish-list titles seen (content not readable)
"Concentrate on Timeline/Workflow improvements" (2024), "Road map for Moho", "Help Shape the Future of Anime Studio", "13.5.2 performance issue" — suggests timeline/workflow and performance are the most-requested areas; not a verified feature list.

## Roadmap additions
43. Real-time live-playback / streaming mode (WebSocket pose input → rendered frames) — Moho 14.5
44. Runtime library in a small portable language (WASM build of the evaluator) — most-cited Moho weakness
45. Layer masking as a first-class layer type (not only boolean ops)
46. Timeline/workflow ergonomics in the studio (graph editor, multi-select keys) — community wish-lists

## Sources (round 5)
- moho.lostmarble.com/blogs/news (17 posts, Dec 2021 – Oct 2026)
- X search snippets: @MohoAnimation posts (13.5.5 Relative Paths; Moho 14 pricing), user posts on Spine 4.2 physics
- Spine forum thread "8033 spine vs moho 12" (de.esotericsoftware.com)
- Lost Marble forum thread titles (content unreadable)

---

# Addendum — Wishlists and feature-request boards (round 6, 2026-10-19)

Only requests actually readable are listed. "OSATURA" column = whether the request is something the engine already covers.

## Moho (Lost Marble forum)
Readable: the forum index only. Sections "Feature Request Discussions" and "Unofficial Bug Reports and Feature Requests" exist; latest visible request title: **"Dock Windows – Custom Interface"** (18 Oct 2026). The long-running "Help Shape the Future of Anime Studio" thread (828 replies, 2011–2021) and other wish-list threads ("Road map for Moho", "My wish for Moho in 2024 – Concentrate on Timeline/Workflow improvements", "Request for top 10 Anime Studio Pro 6 features") are JavaScript-redirect shells → content not readable. Specific forum-to-forum comparisons (Spine forum): physics, masking, smart bones wanted in Spine; Moho criticised for weak FFD and no runtime.
| Request | OSATURA |
|---|---|
| Dockable windows / custom interface | ❌ (web studio fixed layout) |
| Timeline / workflow improvements | 🟡 |
| Real runtime for games | ❌ |
| Stronger FFD | ✅ Bernstein lattice + mesh |

## Cartoon Animator (Reallusion Feedback Tracker + wish-list thread)
| Request (votes where shown) | OSATURA |
|---|---|
| Default keyframe curve in preferences | ✅ per-key easing |
| Multi-channel keyframe editing | 🟡 patch batch ops |
| Sticky panel settings | ❌ |
| Colour-coded markers and channels (left red / right green) | ❌ (mirror L↔R ✅) |
| "Uber key" to show all animated channels | ❌ |
| SVG support (Affinity, Inkscape); paste vector paths | ❌ |
| Align tool/panel | ❌ |
| Depth of field / blur filter | ❌ |
| Drag-drop GIF/APNG/video import placed on timeline | ❌ |
| Spot/ambient lights with shadows | ❌ |
| Timeline auto-scroll | n/a |
| Better text: animatable kerning/leading | ❌ |
| Animatable SVG colour | ✅ |
| Global hide / "shy" channels | ❌ |
| Docked panels | ❌ |
| Multiple scenes per project | ❌ |
| **Auto Animate panel (procedural idle motion on bones)** | 🟡 coherent-noise wobble ✅ |
| Live drawing layer / dynamic editable props (18) | ❌ |
| Hitbox and physics (14) | 🟡 springs |
| Transparent MOV import (14) | ❌ |
| 360° panorama (5); AI integration ideas (6) | ❌ |

## Live2D Cubism Editor feature-request board (30 threads, 2023–2026)
| Request | OSATURA |
|---|---|
| Linux support | ✅ |
| Keyboard shortcuts in manual mesh edit | n/a |
| Blendshape limit down to −100% | ✅ morphs unbounded |
| Proportional resizing; gravity tool; mesh-to-mesh lock | ❌ / ❌ / ❌ |
| Glue for deformers + precise glue weights | 🟡 weights ✅ |
| Import physics without overwriting existing groups | ❌ |
| Lock parameters in animator; loop-edit vs repeat | ❌ |
| "Expand Warp based on Child" options; change warp centre of rotation | ❌ |
| "Set Vertex Flow"-like tool, loop-edge selection, variable-density stroke mesh-mapping | ❌ (auto-mesh uniform) |
| Pixel-art support | ❌ |
| Vertex opacity; parts filtering and cleaner display | ❌ |
| Move keyframes easily | ✅ patch ops |
| Purge cached atlas from memory | n/a |
| Physics window: output max %, bulk-edit pendulum ranges | 🟡 |
| RISC-V / embedded Cubism Core | ❌ |
| Optional drawables in CDI3 spec | n/a |

## Toon Boom Harmony
The user forum is closed and redirects to Discord (not searchable); the old "Feature Requests board" threads ("One can only dream: A Feature List", "Software Upgrade Suggestions Part 2", "Bitmap Layer Overhaul LONG OVERDUE", "Various Quality of Life suggestions") show only the closure notice. No requests recoverable.

## Synfig (GSoC ideas list 2019–2022, linked GitHub issue numbers)
| Request | OSATURA |
|---|---|
| Copy/paste waypoints (#2139) | ✅ keys via ops |
| Ctrl+Tab tab switching, window shortcuts, playback-area selection, properties search, rulers toggle, mouse-driven parameter change | ❌ (GUI) |
| Performance: faster render/export; cache layer parameters; incremental render-task tree | ✅ pure-function evaluate; no cache ❌ |
| Port GTK 4, Conan/vcpkg builds | n/a |

## Spine
Requests go to the forum and are moved to a public Trello board (not fetchable: robots.txt). From the forum: highest priority historically is editor stability and runtime updates. Kickstarter-era plan (meshes → FFD → skinning → IK pinning) is now fully shipped, so OSATURA's mesh/FFD/skinning are at parity with that old list; IK pinning ✅ (Pin constraint). Recent forum request titles: support for latest three.js / React Three Fiber; Spine 4.3 support in Defold.

## Rive
Public roadmap = "Feature Requests channel" with status labels (in progress / planned / launched); needs login to read. Nothing recoverable beyond release notes (round 4).

## OpenToonz, TVPaint, Pencil2D
GitHub (OpenToonz/Synfig) pages are blocked by robots.txt; no readable TVPaint board found; a Pencil2D "Feature requests to add" thread exists but was not opened (out of scope).

## Roadmap additions from wishlists
47. Auto-animate / procedural idle layer (breathing, blink, head sway with rarity & intensity) — CA wish #20; builds on existing wobble
48. Camera depth-of-field and blur filter — CA
49. Lights (spot/ambient) with cast shadows for image layers — CA
50. Align / distribute operations (editor op) — CA
51. Channel colour-coding and "shy" channel visibility in the studio — CA
52. Multiple scenes per project + scene cuts — CA
53. Variable-density auto-mesh; vertex-flow smoothing — Live2D
54. Warp-deformer centre-of-rotation parameter; proportional resize — Live2D
55. Physics import-merge without overwriting groups — Live2D
56. Parameter locking per track during loop editing — Live2D
57. Dockable/custom studio layout — Moho, CA
58. Parameter cache across frames (incremental evaluation) — Synfig

## Sources (round 6)
- Reallusion Feedback Tracker: "Wish list for new features for future updates"; Cartoon Animator 5 board (reallusion.com/FeedBackTracker)
- community.live2d.com — Cubism Editor feature requests (page 1 of 7)
- Synfig GSoC 2022 ideas list (synfig-docs-dev.readthedocs.io)
- Lost Marble forum index (lostmarble.net/forum); Spine forum threads 2250, 8033; Kickstarter thread 1501
- Rive FAQ: "Does Rive have a public roadmap?"

---

# Addendum — Open-source projects on git platforms (round 7, 2026-10-19)

**Access:** GitHub's search API is not available in this environment ("sessions are bound to their configured repositories"), so projects were found via web search and read from their public READMEs. GitLab and Codeberg APIs are blocked by the proxy; Codeberg/GitLab pages appeared only in search results (Friction was readable). Star counts could not be retrieved. Several projects are very new/early — features listed are as claimed in READMEs, not verified by running them.

## Similar open-source rigging / animation tools
| Project | Platform | Notable features | OSATURA comparison |
|---|---|---|---|
| **Rigby2D** (jaisurya-dev-art/rigby2d) | GitHub, MIT/Apache | Spine-style editor (PSD import, mesh + weight paint, linked meshes, IK/transform/path/physics/**slider** constraints, skins with skin-specific bones, events, bounding-box/clipping/point/path attachments, dopesheet + graph editor), Spine JSON 3.8/4.2/4.3 + .skel export, atlas, JS/Rust/Godot runtimes, **built-in MCP server** for AI assistants, Tauri desktop | OSATURA lacks editor GUI, atlas, runtimes, skins, events, slider/path constraints; has pure-function evaluator, liquid/boolean/emitter FX, lip-sync, walk generator |
| **SkelForm** (Retropaint/SkelForm, Rust) | GitHub | FABRIK IK, native + web + mobile wrapper | similar rig core ✅; no GUI ❌ |
| **Ankhimate editor** | GitHub, pre-alpha | Setup/Animate modes, slots + skins, FABRIK any length, path/physics constraints, mesh silhouette tracing from alpha, weight brush with blend modes + bone locking, **FFD timelines**, events with audio, markers, per-bone track offsets, **template-based export format editor**, Rust reference runtime | ❌ mostly GUI; ideas: export templates, bone track offsets |
| **rig2d-rs** (tanhieuno41) | GitHub, Rust | Spine-4.2 physics port, IK with **mix/softness/bend/compress/stretch/uniform**, auto-bind weights by distance falloff, weight smooth/normalise/limit-to-4, shelf atlas packer, onion-skin ghosting, Spine JSON import/export | ❌ soft/stretch IK, weight tools, atlas, Spine interchange |
| **Boner2D**, **Skelly**, **omarvision 2d-bones-IK** | GitHub | Unity / game-designer skeletal tools | basic |
| **Skeletal-Animator** (ostad-ai) | GitHub, Python | bones + FABRIK, rect/capsule/circle shapes, 4 easings, onion skin, keyframed camera clip window, **per-bone visibility ranges**, spritesheet PNG+JSON, **transparent MP4/WebM/MOV/GIF** | ❌ visibility ranges, transparent video/spritesheet export, onion skin |
| **Stretchy Studio** (MangoLion) | GitHub, MIT | PSD/PNG import, **AI auto-rigging** (DWPose pose detection), setup wizard, mesh-per-part vertex animation, vertex skinning for limbs, **auto eye clipping** (iris inside eye), **Blender-style shape keys** with influence sliders, audio tracks on timeline, **Spine 4.0 JSON export** | ✅ morphs ≈ shape keys; ❌ auto-rig, eye clipping, audio |
| **Iki** (zeikar/iki) | GitHub, MIT | Live2D-style web engine: parameter-driven quads, warp-mesh/grid deformers, stencil masks, spring/chain physics, PSD import, **auto-rigger from role-named layers** (blink, gaze, lip-sync, head turn, nod, tilt, brows, breathing, hair physics), open JSON schema with **StandardParameter IDs**, MCP server + Claude Code plugin | ✅ JSON schema, lip-sync; ❌ standard parameter IDs, auto-rigger, masks |
| **Umamo** (umamoorg/umamo) | GitHub, Java | Live2D-compatible editor: read/write CMO3/MOC3 (up to Cubism 5.4), PSD/KRA/CLIP import, command palette, key-binding presets, **diagnostic CLI** (dump/extract/convert/diff models, auto-mesh quality presets); animation not yet supported | ❌ model diff CLI, command palette |
| **Ayatsuri2D** | GitHub | `.aom` MessagePack model (superset of moc3), moc3 v3–6 export, C ABI, fuzzing in CI, panic-free runtime, p99.9 latency benchmarks (early, "not usable yet") | ❌ binary format, C ABI, fuzzing |
| **Ayagami** (AyagamiDev) | GitHub, Rust/wgpu | MOC3 up to SDK 5.0 incl. glue and blendshape params, partial recomputation, linear/sRGB blending, SDK 5.3 advanced blend/off-screen, Godot component | ❌ GPU renderer, MOC3 |
| **kasane-2d**, **img2live**, **claude-live-2d**, **minivt** | GitHub | Live2D-like modelling / single-image-to-puppet research | n/a |
| **Glaxnimate** (KDE) | GitHub mirror / KDE | vector animation; build deps show Potrace bitmap tracing, libav, Python scripting, libarchive; Lottie focus | ❌ tracing, scripting |
| **Friction** (friction/friction, Codeberg) | Codeberg, GPL-3 | vector + raster animation for web/video, motion graphics, SVG, Skia + FFmpeg + GLSL (based on enve) | ❌ SVG animation, shaders |
| **Tahoma2D** | GitHub | 2D + stop-motion, based on Toonz Studio Ghibli version | ❌ |
| **Pencil2D** | GitHub, GPL-2 | bitmap + vector hand-drawn, onion skin | ❌ |
| **q0idmation** (q0s0id) | GitHub | editor + player + **scripting language** | ❌ scripting |
| **svgMotion** (michaelsboost) | GitHub | vector animation in browser | ❌ |
| **kinetic** | GitHub | SVG keyframe timeline → CSS/GSAP export | ❌ CSS/GSAP export |
| **dotlottie-rs** (LottieFiles) | GitHub, Rust | **state machines** (guards/transitions/actions), theming/**slots** (colour/scalar/text/vector runtime overrides), multi-animation `.lottie` zip container, C API + WASM, ThorVG renderer | 🟡 Lottie export ✅; ❌ slots/state machines/dotLottie container |
| **rive-runtime** | GitHub | C++ runtime: GPU vector renderer (Metal, Vulkan, D3D11/12, GL/WebGL), wrapped by Apple/Android/Flutter/Unity/Unreal/web | ❌ |
| **spine-runtimes** | GitHub | runtimes for Unity, Godot, Unreal, libGDX, MonoGame, XNA, Flutter, Haxe, C/C++/C#/TS, Android, iOS, GLFW, SDL, SFML; branch-per-editor-version policy | ❌ |
| **Moho tools repo** (wanderjimenezRD/Moho-Pro-14.4-2D-Animation-Tools) | GitHub | listing only; description gives no features — not evaluated | — |

Pirated-software repos surfaced (e.g. "activate-free" Adobe Animate): not opened or used.

## Ideas worth adopting (new roadmap candidates)
59. **MCP server / agent API** over the patch-op editor so AI assistants can rig and animate (Rigby2D, Iki) — fits OSATURA's JSON-patch architecture
60. **Standard parameter IDs** (ParamAngleX, EyeOpen, MouthOpen…) so any host can drive a model (Iki)
61. **Auto-rigger from role-named layers**: blink, gaze, head turn/nod/tilt, brows, breathing, hair physics (Iki)
62. **Auto eye clipping** (iris masked by eye shape) (Stretchy Studio) — via Boolean layer
63. **Slider constraint** and **path constraint** (Rigby2D, Spine)
64. **Soft / stretchy / compress / uniform IK** modes (rig2d)
65. **Weight tools**: smooth, normalise, limit-to-N, copy, bone locking, brush blend modes (rig2d, Ankhimate)
66. **Mesh silhouette tracing from alpha** (Ankhimate)
67. **Per-bone visibility ranges** (Skeletal-Animator)
68. **Transparent-background video export (WebM/MOV with alpha) and spritesheet PNG+JSON** (Skeletal-Animator)
69. **Export template engine** (user-defined output formats) (Ankhimate)
70. **dotLottie `.lottie` container + runtime slots/theming + state machines** (dotlottie-rs)
71. **Model diff / dump / convert CLI** (`osatura diff a.json b.json`) (Umamo)
72. **Command palette + rebindable shortcuts** in the studio (Umamo)
73. **Fuzzed loader, panic-free runtime path, latency benchmarks** (Ayatsuri2D) — robustness hardening of `Engine::new`
74. **Spine JSON import/export** for interchange (Rigby2D, rig2d, Stretchy Studio)
75. **Shelf atlas packer** for image layers (rig2d, Rigby2D)
76. **Events with keyed values and audio cues** on the timeline (Rigby2D, Ankhimate)
77. **Per-bone track offsets / time-shift** (Ankhimate)

## Sources (round 7)
- github.com/jaisurya-dev-art/rigby2d; Retropaint/SkelForm; Ankhimate/editor; tanhieuno41/rig2d-rs (release v0.1.0)
- github.com/zeikar/iki; MangoLion/stretchystudio; umamoorg/umamo; Yoru-KenomoVT/ayatsuri_2d; AyagamiDev/ayagami
- github.com/ostad-ai/Skeletal-Animator; KDE/glaxnimate; tahoma2d/tahoma2d; pencil2d/pencil
- github.com/rive-app/rive-runtime; EsotericSoftware/spine-runtimes; LottieFiles/dotlottie-rs
- github.com/topics/animation-editor, /topics/vector-animation
- codeberg.org/friction/friction

---

# Addendum — Other public sources (round 8, 2026-10-19)

Sources: itch.io, Steam, AlternativeTo, Wikipedia, academic/patent search results, After Effects tool reviews. Claims are from store/project pages, not hands-on tests.

## Indie / niche rigging tools (itch.io, Steam)
| Tool | Notable features | OSATURA |
|---|---|---|
| **Inochi Creator / Inochi2D** (Kitsunebi Games, open source, beta) | open VTuber-oriented 2D puppet rigging + real-time runtime; own format (not Live2D/Spine compatible); GIF/image-sequence export; multilingual UI | 🟡 same niche (open rig format) |
| **Keyframe.it** (browser, free) | bones + IK + weighted meshes + spring motion; **game-symbol clips** (idle, land, anticipation, win); built-in image editor + PSD import; export GIF, sprite sheet, video, standalone HTML player, **Spine 3.8/4.2 bundles**; autosave in browser; **keyframe-mcp** AI-agent connector; optional built-in AI assistant | ✅ HTML player/GIF; ❌ sprite sheet, Spine bundle, MCP, in-browser editor |
| **RigBone 2D** (Godot) | bone posing, pose-as-frame, onion skin, crop + transparent background, PNG frame sequence (no runtime needed) | ❌ onion skin, PNG sequence with alpha crop |
| **SpineBones** | slots + image attachments, groups, IK, **audio preview on timeline**, background reference image, JSON / sprite sheet / WebM export | ❌ |
| **PySpine, Tangent (sprite animator)** | listed on itch.io; no feature text read | — |
| **Mocap2D / FBX2SPINE** (Steam) | FBX/BVH mocap import; **Auto Match** skeleton by names + hierarchy; **Smart Retargeting** (projection, good for side view); **Direct Retargeting** per axis; **automatic in-place conversion** for locomotion loops; **ground-contact IK** (foot lock); live preview; import/export FBX, BVH, Spine skeletons, Unity 2D Animation | 🟡 retarget by name ✅; ❌ mocap import, foot-lock IK, in-place conversion |
| **VTube Studio** (Steam) | webcam tracking (OpenSeeFace), **voice-driven mouth**, mouse/pen as tracking input, any input → any model parameter mapping, hotkeys for expressions/animations, pinned PNG "items" that follow the model; Live2D models only | 🟡 parameter channels ✅; ❌ live input mapping, hotkeys, pinned items |
| VBridger, Vitamins | Steam tools around VTubing; not evaluated | — |

## After Effects rigging ecosystem (School of Motion review)
| Feature | Source | OSATURA |
|---|---|---|
| **Joystick controller** (5 key poses: origin/right/left/top/bottom) interpolated in 2D | Joysticks 'n Sliders | ❌ — strong candidate |
| **Sliders** mixing unlimited poses | JnS, Duik | 🟡 morph channels |
| Structures (bones independent of artwork, reusable), autorig | Duik Bassel | ✅ |
| Controllers: slider, 2D slider, angle | Duik | ❌ |
| **Connector** (slider/joystick/rotation linking any property to another), master property control | Duik | 🟡 expressions? (expression engine ✅ in channels) |
| Mid-pose keys between pose states | Duik | ✅ |
| Animate mask paths | JnS | ✅ path morphs |

## Competing / alternative apps (AlternativeTo, Wikipedia list as of 2026)
Tahoma2D (OpenToonz fork, stop-motion), TupiTube, enve (discontinued; became Friction), Expressive Animator (interactive SVG), Cavalry, Jitter (web motion design), Blazy (Figma → animation, CSS/React export), Animation Desk, Pivot Animator, FlipaClip, Krita animation, Blender (Mar 2026), Clip Studio Paint (animation), Adobe Character Animator (webcam puppeteering; "what's new" page blocked by robots.txt), Adobe Animate (last release listed Oct 2023), CrazyTalk, DigiCel FlipBook, Apple Motion, RETAS. None add rigging features beyond what earlier rounds cover; new ideas: **Figma/design-tool → animation presets with CSS/React export** (Blazy) and **webcam puppeteering** (Character Animator, VTube Studio).

## Academic and patent literature (titles/abstracts only)
- Jacobson et al. 2011, **Bounded Biharmonic Weights** for real-time deformation — automatic smooth skinning weights from a cage/skeleton ✅ idea: improve `auto_skin` (currently distance-based).
- Adobe patents US9865073 / US10078910, **"Computer animation of artwork using fast 2-dimensional deformation solving"** (Character Animator's mesh warp). *Clean-room note:* OSATURA uses Bernstein FFD and weighted skinning, not that patented solver; any future ARAP-style solver should be reviewed with counsel. This is not legal advice.
- arXiv 2025–26 auto-rigging: **ASMR** (3D mesh skeleton + skin via 2D generative priors), UniRig, "How to Train Your Dragon" (diffusion rigging), HumanRig, SPRig, UniMate — all 3D/ML; relevant only as a future "auto-rig suggestion" feature.
- Stony Brook TVC 2017 **automatic skinning weight retargeting** — copy weights between meshes (✅ roadmap: weight copy).

## New roadmap candidates from this round
78. **Joystick controller**: 2D control point blending 5 key poses (and N-pose variants) → drives any channels — JnS / Duik / Live2D parameters
79. Mocap import (BVH) + auto-match skeleton + in-place loop conversion + foot-lock IK — Mocap2D
80. Live input mapping: any input (mouse, MIDI, audio level, webcam landmarks) → parameter, plus hotkeys for expressions — VTube Studio
81. Pinned items/props following a bone/anchor with its own offset — VTube Studio
82. Game-symbol clip presets (idle / land / anticipation / win) — Keyframe.it
83. Spritesheet (PNG + JSON) and transparent PNG-sequence export with crop — Keyframe.it, RigBone
84. Timeline audio preview with waveform — SpineBones
85. Reference-image layer (non-rendered background) — SpineBones
86. Bounded-biharmonic or heat-diffusion auto-skin weights (replace distance falloff) — BBW paper
87. Design-tool presets → CSS/React export — Blazy
88. Webcam face-tracking puppeteering via the studio (WebRTC + landmark model) — Character Animator, VTube Studio

## Sources (round 8)
- itch.io: kitsunebi-games/inochi-creator, keyframeit.itch.io, itch.io/blog/1586091 (RigBone 2.0), itch.io/t/6177139 (SpineBones)
- Steam: FBX2SPINE / Mocap2D (app 2467960), VTube Studio (app 1325860)
- schoolofmotion.com — Joysticks 'n Sliders vs Duik Bassel
- alternativeto.net (Moho, Expressive Animator alternatives); en.wikipedia.org/wiki/List_of_2D_animation_software
- Jacobson 2011 BBW (homes.cs.washington.edu/~jovan/papers/jacobson-2011-bbw.pdf); patents US9865073B2 / US10078910; arXiv 2503.13579, 2503.15586, 2504.12451; Stony Brook TVC 2017 weight retargeting

---

# Addendum — Adobe Character Animator, full feature set (round 9, 2026-10-19)

Sources: Wikipedia version table (incl. 26.0 of Jan 2026), Adobe product page, Adobe Help "Behaviors" page and release notes (Oct 2018, Oct 2020/Feb 2021, Oct/Dec 2021, 2023). Adobe's own "what's new" index returns 2023 and 2026 only; 24.x and 25.x are listed by Wikipedia as stability-only releases.

## Concept
Layered Photoshop/Illustrator art (also EPS/SVG since Oct 2018) becomes a **puppet** rigged automatically from **layer names/tags**; performance is captured live (webcam face/body, microphone, keyboard, mouse, MIDI) as **takes** on a timeline; outputs go to PNG sequences, WAV, Media Encoder (incl. **video with alpha**), Dynamic Link to After Effects / Premiere, and live Syphon / Mercury Transmit. Starter mode is free; Pro needs Creative Cloud.

## Behaviors (full list from Adobe Help)
| Behavior | What it does | OSATURA |
|---|---|---|
| **Face** | head, eyes, brows, nose, mouth from webcam; parallax and head-position strength | ❌ (no live tracking) — roadmap #88 |
| **Body** (Body Tracker, Sensei AI, 22.0) | arms, torso, legs from webcam | ❌ |
| **Lip Sync** | mouth visemes from voice; transcript-assisted accuracy (22.1); engine 3.4 better F, W-Oo, fewer L/M | 🟡 WAV → 5 visemes ✅; transcript alignment ❌ |
| **Nutcracker Jaw** | lower jaw from face/voice | ❌ |
| **Eye Gaze** | pupils move independently | ✅ aim constraints/channels |
| **Head & Body Turner** | swaps layer groups (front, quarter, side) as head/body rotate | 🟡 Switch layers ✅; no angle-driven auto swap |
| **Walk** | walk cycle, run, pose emphasis, shoulder/hip sway, **Pin Feet When Standing** (3.4/3.5) | ✅ walk generator; ❌ pin-feet |
| **Limb IK** (ex Arm IK) | bend direction + stretch for arms and legs | 🟡 2-bone IK ✅; stretch ❌ |
| **Leader/Follower** (22.5) | shapes how limbs bend/stretch | ❌ |
| **Dragger** | mouse/touch moves a region | ❌ (studio is not interactive on canvas) |
| **Physics / Dangle** | swing, collision, gravity, wind; squash & stretch, damping | 🟡 springs ✅; collision/wind ❌ |
| **Magnets** | handles snap together by proximity | ❌ |
| **Particles** | emits physics objects (snow, cannon fire) | ✅ emitter |
| **Motion Lines** | trailing lines behind fast objects | ❌ |
| **Wiggler** | wobble on vector art | ✅ wobble/boil |
| **Breath** | breathing motion | 🟡 manual channel; roadmap #61 |
| **Auto Blink** | automatic blinks / layer flicker | ❌ (roadmap #61) |
| **Motion Library** (23.0, 350+ motions, strength param 23.1) | apply full-body animations | 🟡 walk only; ❌ library |
| **Cycle Layers** | step through layers in a group | 🟡 Switch layers ✅; frame-cycling ❌ |
| **Layer Picker / Motion Trigger / Fader** | show a layer; switch by motion direction; fade on trigger | 🟡 |
| **Transform** | move/scale/rotate/opacity of handles | ✅ |
| **Handle Fixer** | lock parts of a mesh | ✅ pins |
| **Triggers** | key-press shows layers/gestures; **Controls panel** with sliders, angle dials, buttons, MIDI mapping | ❌ — roadmap #80 |
| **Collide, Dynamic, Cycle Pause, Auto-swap (22.5)** | tags/behaviors for physics and frame-like swaps | ❌ |

## Timeline, recording, and workflow features
| Feature | Version | OSATURA |
|---|---|---|
| Takes, replays (reusable triggerable recordings), smart replays, editable replays | 2.0 / 22.0 / 23.6 | 🟡 clips ✅; replays ❌ |
| Keyframes, time stretching of keyframes, scene cameras, triggerable audio | 3.0 / 3.2 | ✅ keys/camera; ❌ triggerable audio |
| Speech-Aware Animation (head/brow motion from voice, Sensei) | 3.4 | ❌ — roadmap #61 |
| Merge/sequence takes; coloured take rows; search filters (`kind:` `shared:` `armed:`); shy rows; collapse takes | 3.4 | ❌ |
| Snap to frames / objects; timeline snapping | 3.5 | ❌ |
| Work-area ops: ripple delete, insert time, loop, trim, export | 2.0 | 🟡 clip ops ✅ |
| Rigging issues pane; tag & trigger search | 3.0 | ✅ validation collects all errors |
| History bookmarks; automatic damaged-project recovery (4.2); new compact/corruption-resistant file format (4.0) | | 🟡 undo ✅; recovery ❌ |
| Puppet Maker (templates: hair, skin, accessories) | 22.1.1 | ❌ |
| Characterizer (turn a photo/painting into a puppet) | 2.0 | ❌ |
| Quick Export for social sharing | 22.5 | 🟡 GIF/MP4 ✅ |
| Scene snapshots, recording countdown, custom control button art | 1.5 | ❌ |
| Camera selection, multi-monitor, HiDPI | 2.0 | n/a |
| Live output (Syphon, Mercury Transmit), Twitch Character Trigger extension | 1.1 / 2.1 | ❌ — roadmap #43 |
| MIDI, scripts via V8 (JavaScript) | 1.1 / 2.0 | ❌ — roadmap #31 |
| Share/publish puppets, shareable puppet files | Beta 5 | 🟡 JSON portable |
| Dynamic Link to AE / Premiere | Beta 5 | ❌ |
| Weaknesses reported (review): steep learning curve, hardware (webcam/mic) dependence, long character-prep time, paid Pro, trigger setup complexity | | opportunity: OSATURA deterministic offline pipeline, CLI/JSON |

## What stands out for OSATURA (new roadmap candidates)
89. **Tag-based auto-rig from layer names** (head, eyes, brows, mouth visemes, arm/leg tags) — Character Animator core idea; extends roadmap #61
90. **Takes & replays**: record parameter performance as reusable, triggerable clips with blend easing
91. **Pin Feet When Standing** and ground-contact lock for walk generator
92. **Leader/Follower** limb stretch behaviors; **Magnets**; **Motion Lines**
93. **Motion Library** format (named motion clips with strength 0-100%) 
94. **Head & Body Turner**: auto-swap view groups from a yaw angle channel
95. **Transcript-assisted lip-sync** (forced alignment of text to WAV phonemes)
96. **Speech-aware head/brow motion** from audio energy and pauses
97. **Video-with-alpha export** (ProRes 4444 / WebM VP9 alpha) — also in Skeletal-Animator (roadmap #68)
98. **Triggerable audio** and audio-reactive parameters

## Sources (round 9)
- en.wikipedia.org/wiki/Adobe_Character_Animator (feature list and version table through 26.0, Jan 2026)
- adobe.com/products/character-animator.html
- helpx.adobe.com — Behaviors in Character Animator; Feature summaries Oct/Dec 2021, Oct 2020/Feb 2021; What's new (2023–2026)
- community.adobe.com — "Character Animator October 2018 detailed changes"
- manytools.com/review/adobe-character-animator (strengths/weaknesses)
