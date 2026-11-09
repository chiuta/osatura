<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# OSATURA project format

This is the reference for the OSATURA project file: a single JSON document that
holds the stage, the rig, the artwork and the animation. It is written from the
serde structs in `osatura-core/src/project.rs` and the modules they reference,
and every example in sections 13 to 15 was checked with the `osatura` binary
(`osatura check`, `osatura render`, `osatura explain`).

Contents

1. [Conventions and primitive types](#1-conventions-and-primitive-types)
2. [Loading, versioning and unknown fields](#2-loading-versioning-and-unknown-fields)
3. [Top-level fields](#3-top-level-fields)
4. [Rig: bones, ik, constraints, springs](#4-rig-bones-ik-constraints-springs)
5. [Layers](#5-layers)
6. [Paths, anchors, paint, stroke](#6-paths-anchors-paint-stroke)
7. [Deformers, effects, normal map](#7-deformers-effects-normal-map)
8. [Channels, tracks, easing](#8-channels-tracks-easing)
9. [Expressions](#9-expressions)
10. [Camera, shots, audio, metadata](#10-camera-shots-audio-metadata)
11. [Clips, skins, links, interaction, lods](#11-clips-skins-links-interaction-lods)
12. [Validation rules, limits and error messages](#12-validation-rules-limits-and-error-messages)
13. [Example 1: bouncing ball](#13-example-1-bouncing-ball)
14. [Example 2: rigged arm with IK](#14-example-2-rigged-arm-with-ik)
15. [Example 3: switch-layer mouth](#15-example-3-switch-layer-mouth)
16. [Evaluation order](#16-evaluation-order)
17. [Rendering order](#17-rendering-order)
18. [Assets and paths](#18-assets-and-paths)
19. [Versioning policy](#19-versioning-policy)
20. [Known gaps between code and comments](#20-known-gaps-between-code-and-comments)

## 1. Conventions and primitive types

| Type | JSON form | Notes |
| --- | --- | --- |
| number | JSON number | Stored as `f32` unless the field says integer. Every float must satisfy `abs(x) <= 1e7` (section 12). |
| `Vec2` | `[x, y]` | Two numbers. Stage units are pixels, +x right, +y down. |
| `Rgba` | string | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, or one of `black`, `white`, `red`, `green` (0,128,0), `blue`, `yellow`, `transparent`. Output is always lowercase hex. |
| `Paint` | string or object | A colour string is a solid paint. `{"from": [x,y], "to": [x,y], "stops": [...]}` is a linear gradient. `{"center": [x,y], "radius": r, "stops": [...]}` is a radial gradient. The type is an untagged enum. |
| `Stop` | `{"offset": 0..1, "color": Rgba}` | Stops are sorted (stable) at draw time and offsets are clamped to 0..1 in the rasteriser. A degenerate gradient paints the last stop. |
| angle | number | Degrees everywhere in the file (bone angles, layer rotation, skew, camera rotation, `angle_limits`). |
| time | number | Frames in keys and markers, seconds in sounds, machines and emitters. Each field says which. |
| names | string | Bone, layer, path, deformer, ik, constraint names. Sections 5 and 12 list the character restrictions. |

A bad colour inside a paint context produces the serde message
`data did not match any variant of untagged enum Paint`, because `Paint` is untagged and serde
cannot say which branch was meant.

Field tables below use this notation: **Req** means the field has no default and must be present;
otherwise the default is shown. "Range" is what the loader or engine enforces. A range written
"clamped" is not an error: the engine silently clamps the value.

## 2. Loading, versioning and unknown fields

`Project::from_json(text)` is the single entry point used by the CLI, the C API, the WASM module
and the Studio. It does, in order:

1. Parse the JSON with serde. Serde ignores fields it does not know.
2. Run the magnitude check: any float literal whose absolute value is above `1e7` fails the load
   with `invalid JSON in <file>: value ... is outside the supported range of +-1e7 ...`. Integer
   literals are caught later, by the re-check inside `Engine::new` (see section 12).
3. Apply the format gate: `format` greater than the build's `FORMAT_VERSION` (currently `1`) is
   rejected with `project format 2 is newer than this build supports (1); update OSATURA`.

`Engine::new` (or `Engine::with_asset_dir`) then validates the whole project and returns every
problem it finds as a list. The CLI prints them as:

```text
<file> is not a valid project:
  - first problem
  - second problem
```

Error texts are for humans. They are not a stable contract; match on the text only in tests.

### Unknown fields

Serde drops unknown keys on load, so a typo such as `fil` for `fill` would be silently lost. The
`unknownfields` module compares the input JSON with the re-serialized project and reports keys that
did not survive:

```text
warning: unknown field layers[0].paths[0].fil (ignored)
```

`osatura check` prints these warnings. Values that equal the default are not reported (the writer
omits defaults, so an explicit `"visible": true` is fine). A few structures are strict instead of
lenient and reject unknown keys with a hard error: machine `Transition` and `Condition`, CI config,
live-input config, overlays and face-parameter config; config files are read with `parse_strict`.

### Editor metadata round trip

The project serializes with `skip_serializing_if` for optional lists. A file you write by hand may
be shorter than what `osatura` writes back; both are equivalent.

A JSON Schema (dialect 2020-12) of the file is printed by `osatura schema [out.json] [--strict]`; it describes the root, `camera`, `bones` and `ik` field by field and the other lists as arrays of objects. It allows unknown properties, like the loader, unless `--strict` is given.

## 3. Top-level fields

| JSON name | Type | Default | Range / rule | Meaning |
| --- | --- | --- | --- | --- |
| `format` | integer | `FORMAT_VERSION` (1) | must be `<=` the build's version | Format version, section 19. |
| `name` | string | **Req** | any | Display name. Printed by `check`. |
| `width` | number | **Req** | `> 0`, `<= 32768` | Stage width in pixels. |
| `height` | number | **Req** | `> 0`, `<= 32768` | Stage height in pixels. |
| `fps` | number | **Req** | `(0, 1000]` | Frames per second. |
| `frames` | integer (`u32`) | **Req** | `1..=200000` | Length of the timeline. Frame indices are `0..frames-1`. A float literal such as `48.0` is rejected: `invalid type: floating point `48.0`, expected u32`. |
| `background` | `Rgba` or null | null (transparent) | | Stage colour behind everything. |
| `color_space` | string | `"srgb"` (not written) | `srgb`, `linear` | Space in which the raster and float compositors blend. `linear`: colours are converted to linear light, composited in f32 and converted back once (a 50 % black over white is 188/255, not 128/255). Set with `osatura color-space`. SVG, Lottie, the GPU renderer, `dof` and `fx` ignore it. |
| `camera` | object | defaults | section 10 | Global camera. |
| `bones` | array | `[]` | at most 4096 | Skeleton, section 4. |
| `ik` | array | `[]` | | IK chains. |
| `constraints` | array | `[]` | | Aim, copy-rotation, pin. |
| `springs` | array | `[]` | | Secondary motion, baked at load. |
| `layers` | array | `[]` | | Root layers, drawn in list order. |
| `channels` | array | `[]` | | Animation channels, evaluated in list order. |
| `markers`, `comps`, `events`, `bone_vis`, `channel_meta`, `shots` | arrays | omitted when empty | not validated | Timeline and editor metadata, section 10. |
| `sounds` | array | omitted when empty | validated | Audio tracks, section 10. |
| `skins`, `clips`, `clip_baked`, `lods` | arrays | omitted when empty | not validated at load | Section 11. |
| `mesh_locks`, `glues` | arrays | omitted when empty | validated | Cross-layer mesh links, section 11. |
| `machines`, `bindings`, `containers`, `scrolls` | arrays | omitted when empty | validated | Interaction and layout, section 11. |

Minimal valid project:

```json
{"name": "empty", "width": 100, "height": 100, "fps": 24, "frames": 1}
```

Magnitudes: every float in the file must satisfy `abs(x) <= 1e7`. This applies to coordinates,
scales, offsets and key values alike. Width and height are further limited to 32768, and raster
output to 16384 per side and 100 megapixels (section 18).

## 4. Rig: bones, ik, constraints, springs

### 4.1 `bones[]`

Bones must be listed parents-first. A bone's parent must appear earlier in the list.

| JSON name | Type | Default | Range | Meaning |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | unique | Referenced by weights, ik, channels, `Layer.bone`. |
| `parent` | string or null | null | an earlier bone | Parent bone. Null means a root. |
| `length` | number | **Req** | finite, `>= 0` | Bone length in pixels, along the bone's +x axis. |
| `rest_offset` | `Vec2` | `[0,0]` | | Head position in the parent's space (stage space for roots). |
| `rest_angle` | number | 0 | | Rest rotation in degrees relative to the parent. |
| `angle_limits` | `[min, max]` or null | null | `min <= max` | Clamp on the animated angle delta, degrees. |
| `scale_limits` | `[min, max]` or null | null | `0 < min <= max` | Clamp on the animated scale. |

Interactions: `bone_angle` channels add to `rest_angle`; `bone_offset_x/y` add to `rest_offset`;
`bone_scale` multiplies. `clamp_limits` runs after all channels and before FK, and again after each
IK chain.

```json
{"name": "upper", "parent": null, "length": 90, "rest_offset": [60, 120],
 "rest_angle": 0, "angle_limits": [-120, 120]}
```

### 4.2 `ik[]`

| JSON name | Type | Default | Range | Meaning |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | unique | Target of `ik_mix` channels. |
| `bones` | array of strings | **Req** | at least 2; each a direct child of the previous | Chain from root to tip. |
| `target` | string | **Req** | a bone, not in the chain | Bone whose head the tip reaches for. |
| `bend` | number | 1 | +1 or -1 | Which way a two-bone chain folds. |
| `mix` | number | 1 | clamped 0..1 | Initial FK-to-IK blend; animate with `ik_mix`. `<= 0` skips the chain. |
| `iterations` | integer | 16 | | FABRIK sweeps for chains longer than two bones. This is a floor: the solver exits early when the tip is within 0.01 px and otherwise continues up to 256 sweeps. |
| `soft` | number | 0 | `0..0.9` | Fraction of the reach over which extension eases off. With `stretch > 1` the zone is the last fraction of the stretched reach (`reach x stretch`). |
| `stretch` | number | 1 | `1..4` | Maximum stretch factor; above 1 the root bone's scale grows to reach. |

A two-bone chain uses the analytic law-of-cosines solver; longer chains use FABRIK. After solving,
angles are blended into the pose as `old + wrap_pi(new - old) * mix`.

```json
{"name": "reach", "bones": ["upper", "lower"], "target": "target", "bend": 1, "mix": 1}
```

### 4.3 `constraints[]`

Constraint types use an internal `"type"` tag, flattened next to `name`, `bone` and `weight`.

| JSON name | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | **Req** | Target of `constraint_weight` channels. |
| `bone` | string | **Req** | The constrained bone. |
| `type` | `aim`, `copy_rotation`, `pin` | **Req** | Kind. |
| `target` | string | **Req** for `aim`, `pin` | Bone to aim at or slide onto. |
| `offset` | number | 0 | `aim` only: extra degrees. |
| `source` | string | **Req** for `copy_rotation` | Bone whose world rotation is copied. |
| `weight` | number | 1 | Influence, clamped 0..1. `<= 0` skips the constraint. |

A constraint may not target itself. Constraints run in list order after all IK chains.

```json
{"name": "aim", "bone": "root", "type": "aim", "target": "look", "offset": 0, "weight": 0.5}
```

### 4.4 `springs[]`

A damped spring that makes a bone's world rotation lag behind its animated rotation. It is
simulated once at load and baked, so any frame can be evaluated in any order.

| JSON name | Type | Default | Range | Meaning |
| --- | --- | --- | --- | --- |
| `bone` | string | **Req** | one spring per bone | Bone to drive. |
| `frequency` | number | 3 | `(0, 30]` Hz | Stiffness. |
| `damping` | number | 0.35 | `>= 0` | Damping ratio; below 1 rings, 1 is critical. |
| `warmup` | bool | true | | Pre-roll one pass so loops start in steady state. |

Baked angles replace the bone angle after constraints (evaluation step 8).

## 5. Layers

A layer is `{"name": ..., "type": ..., <kind fields>, <common fields>}`. `type` is the tag of
`LayerKind`; its fields are flattened into the layer object.

### 5.1 Common fields

| JSON name | Type | Default | Range | Meaning |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | non-empty, no `/`, no `::`; the full path must be unique | Layer name. Children are addressed as `parent/child`. |
| `type` | string | **Req** | `vector`, `group`, `selector`, `metaball`, `boolean`, `emitter`, `image` | Kind. |
| `transform` | object | identity | | `position` `[0,0]`, `rotation` 0, `scale` `[1,1]`, `skew` 0, `anchor` `[0,0]`. |
| `opacity` | number | 1 | clamped 0..1 | Multiplies into the layer composite. |
| `visible` | bool | true | | Hidden layers are skipped when drawing but can still be mask sources. |
| `blend` | string | `normal` | see below | Blend mode. |
| `mask` | `{layer, invert}` | null | no self-mask, no cycles | Use another layer's alpha as a mask. |
| `bone` | string | null | a known bone | Attach to a bone: the bone's skin matrix is applied between the parent stage matrix and the local transform. |
| `parallax` | number | 1 | | Camera depth factor. Only the top-level layer's value counts. |
| `deformers` | array | omitted when empty | vector and image layers only | Section 7. |
| `wobble` | object | null | | `amp`, `scale` (0.03), `rate` (8), `seed` (0): deterministic noise displacement. Animate `amp` with `layer_wobble`. |
| `reference` | bool | false | | Reference layer: hidden unless the editor shows references. |
| `locked` | bool | false | | Editor lock, see below. |
| `normal_map` | object | null | | Section 7. |
| `effects` | array | omitted when empty | at most 16 | Section 7. |

Local matrix: `T(position) * R(rotation) * Shear(skew) * S(scale) * T(-anchor)`.

Blend modes: `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`, `color_dodge`,
`color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `add` (CSS plus-lighter).

`locked` blocks editor operations only: canvas picking, gizmos, nudges, deletes, timeline edits and
channel edits (through `assert_unlocked`), for the layer and its descendants. Rendering and
evaluation ignore it, and plain `Op`s (section "Editor ops" in ARCHITECTURE.md) still apply.

The common fields are written after the kind fields in output but order is irrelevant on input.

### 5.2 `vector`

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `paths` | array of path | `[]` | Section 6. |

### 5.3 `group`

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `layers` | array of layers | `[]` | Children inherit the group transform. |

### 5.4 `selector`

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `layers` | array of layers | `[]` | Candidate drawings. |
| `selected` | integer | 0 | Initial child index. Animate with `layer_switch` (step keys). |

Exactly one child is shown. The value is rounded; negative becomes 0; values at or above the child
count become the last child. Validation does not reject out-of-range keys. See section 20 for a
lint wording mismatch about this.

### 5.5 `metaball`

Metaball liquid. The field is the sum of `r^2 / d^2` over the blobs; the outline is the iso-contour,
extracted by marching squares on a grid capped at 600 by 600 cells and smoothed with Catmull-Rom.

| Field | Type | Default | Range | Meaning |
| --- | --- | --- | --- | --- |
| `blobs` | array of `{pos, radius, bone?}` | **Req** | at least 1; `radius > 0` | A blob. `bone` attaches it to a bone. |
| `threshold` | number | 1 | `> 0` | Iso-level; 1 makes an isolated blob a circle of its radius. |
| `cell` | number | 3 | `>= 0.25` | Grid cell in layer units. |
| `fill` | `Paint` | null | | |
| `stroke` | stroke object | null | | |

### 5.6 `boolean`

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `op` | `union`, `intersect`, `difference`, `xor` | **Req** | Operation. |
| `paths` | array of path | **Req**, at least 2 | `paths[0]` is the subject; the clip is the union of the rest. |

The result takes the subject's fill and stroke; flattening tolerance is 0.25. Result paths are
named `<first path name><index>`.

### 5.7 `emitter`

`{"type": "emitter", "spec": {...}}`. Particles are a closed-form function of time, so any frame
is evaluable. Times are in seconds.

| `spec` field | Default | Range | Meaning |
| --- | --- | --- | --- |
| `pos` | `[0,0]` | | Emitter position (layer space). |
| `bone` | null | | Attach to a bone. |
| `rate` | 20 | `(0, 2000]` | Particles per second. |
| `lifetime` | 1 | `> 0` | Seconds. |
| `lifetime_jitter` | 0 | 0..1 | |
| `speed` | 60 | | Pixels per second. |
| `speed_jitter` | 0 | 0..1 | |
| `direction` | -90 | | Degrees; -90 is up. |
| `spread` | 30 | | Degrees of cone width. |
| `gravity` | `[0,0]` | | Acceleration, pixels per second squared. |
| `size_start`, `size_end` | 3, 3 | | Radius over life. |
| `color_start`, `color_end` | `#fff`, `#fff0` | | Colour over life. |
| `seed` | 0 | | Deterministic seed. |
| `start` | 0 | | Start time in seconds. |
| `duration` | null | | Emission stops after this many seconds; null is forever. |
| `max_particles` | 2000 | | Hard cap on live particles. |

### 5.8 `image`

| Field | Type | Default | Range | Meaning |
| --- | --- | --- | --- | --- |
| `source` | `{file?, png_base64?}` | **Req** | one of the two | `file` is a path relative to the project directory (section 18); `png_base64` is self-contained. Neither: `image has neither 'png_base64' nor 'file'`. |
| `origin` | `Vec2` | `[0,0]` | | Top-left corner in layer space. |
| `size` | `Vec2` or null | null (pixel size) | | Displayed size. |
| `mesh` | object or null | null | | Deformation mesh. |

`mesh`: `cols`, `rows` (grid mesh, each at least 1), `weights` (per-vertex bone weights; empty
means static), and optionally `free` = `{vertices, triangles, uvs}`. A free mesh needs at least 3
vertices, at least 1 triangle, at most 200000 vertices and 400000 triangles, finite coordinates,
valid triangle indices, and `uvs` either empty or the same length as `vertices`. With deformers but
no mesh, a 16 by 16 grid is used.

```json
{"name": "img2", "type": "image", "source": {"png_base64": "iVBOR..."},
 "origin": [100, 0], "size": [80, 80],
 "mesh": {"free": {"vertices": [[100,0],[180,0],[180,80],[100,80]],
                   "triangles": [[0,1,2],[0,2,3]],
                   "uvs": [[0,0],[8,0],[8,8],[0,8]]}, "weights": []}}
```

## 6. Paths, anchors, paint, stroke

### 6.1 Path (`paths[]`)

| JSON name | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | `""` | An empty name is replaced by `path{index}` for addressing. Must not contain `::`. Unique within the layer. |
| `closed` | bool | false | |
| `anchors` | array | **Req** | Bezier anchors. |
| `fill` | `Paint` | null | |
| `stroke` | stroke | null | |
| `morphs` | array of `{name, anchors}` | omitted when empty | Additive blend shapes. Anchor count must equal the base path. Weight via `path_morph`. |

### 6.2 Anchor

| JSON name | Type | Default | Meaning |
| --- | --- | --- | --- |
| `pos` | `Vec2` | **Req** | Position. |
| `handle_in`, `handle_out` | `Vec2` | null | Control points in the same space as `pos` (absolute, not relative). A missing handle equals `pos` (a corner). |
| `weights` | array of `{bone, weight}` | omitted when empty | Skinning. Each bone must exist and each weight is finite and `>= 0`. Weights are normalized at evaluation; empty means static. Handles are skinned too. |

### 6.3 Stroke

| JSON name | Type | Default | Meaning |
| --- | --- | --- | --- |
| `paint` | `Paint` | **Req** | |
| `width` | number | 1 | |
| `cap` | `butt`, `round`, `square` | `round` | |
| `join` | `miter`, `round`, `bevel` | `round` | An unknown value gives ``unknown variant `foo`, expected one of `miter`, `round`, `bevel` ``. |
| `dash` | array | omitted | Used only if its length is even, at least 2, and every entry `> 0`. |
| `dash_offset` | number | 0 | |
| `width_profile` | array of `[t, factor]` | omitted | Piecewise-linear width multiplier along the length (`t` in 0..1), constant beyond the ends, mitre limit 2. A variable-width stroke ignores `dash` and `join`. |

Example of a radial gradient fill with a stroke (from the ball example):

```json
"fill": {"center": [-8, -10], "radius": 44,
         "stops": [{"offset": 0, "color": "#ffd9a0"}, {"offset": 1, "color": "#d6336c"}]},
"stroke": {"paint": "#3d2c2e", "width": 2}
```

The linter (`osatura lint`) warns about gradient problems (L011), switch keys (L012) and editor-data problems (L013); see
`osatura-core/src/lint.rs`.

## 7. Deformers, effects, normal map

### 7.1 `deformers[]`

Allowed only on `vector` and `image` layers. Each has a `"type"` tag and a `name` (unique within
the layer, no `::`). They run in list order on rest-space geometry, before skinning, and before
`wobble`. Offsets are static; animate them with the sub-target channels.

| `type` | Fields | Constraints | Animate with (target) |
| --- | --- | --- | --- |
| `lattice` | `cols`, `rows`, `rect` `[x,y,w,h]`, `offsets` | `cols, rows >= 1`; `w, h > 0`; `offsets` empty or `(cols+1)*(rows+1)` row-major | `lattice_dx/dy` (`layer::name::c,r`) |
| `spine` | `rest` (curve points), `offsets`, `radius` (optional soft edge) | at least 2 points | `spine_dx/dy` (`layer::name::i`) |
| `pin` | `pins`, `offsets`, `radius` | 1 to 64 pins, `radius > 0`, pins not coincident | `pin_dx/dy` (`layer::name::i`) |
| `cage` | `rest` (polygon), `offsets` | 3 to 256 vertices, simple polygon, non-zero area | `cage_dx/dy` (`layer::name::i`) |
| `cloth` | like `lattice` | like `lattice` | `lattice_dx/dy` (`layer::name::c,r`) |

Lattice blends offsets with Bezier-Bernstein weights (smooth); cloth displaces each grid vertex
exactly (bilinear). Pin uses a Wendland compactly supported radial basis function. Cage uses
Floater mean-value coordinates: content inside the polygon follows it, content outside is left
alone.

```json
"deformers": [
  {"type": "lattice", "name": "lat", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []},
  {"type": "spine", "name": "sp", "rest": [[0,0],[50,0],[100,0]], "offsets": [], "radius": 30},
  {"type": "pin", "name": "pn", "pins": [[0,0],[100,100]], "offsets": [], "radius": 80},
  {"type": "cage", "name": "cg", "rest": [[0,0],[100,0],[100,100],[0,100]], "offsets": []},
  {"type": "cloth", "name": "cl", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []}
]
```

### 7.2 `effects[]`

At most 16 per layer. Each effect has `"type"`, `enabled` (default true) and the fields below. The
`fxN` channel target indexes the array including disabled effects.

| `type` | Fields | Range |
| --- | --- | --- |
| `blur` | `radius` | 0..64 |
| `drop_shadow` | `dx`, `dy`, `blur`, `color`, `opacity` (1) | offsets at most 1024; blur 0..64; opacity 0..1 |
| `tint` | `black`, `white`, `amount` (1) | amount 0..1 |
| `fill` | `color`, `opacity` (1) | opacity 0..1 |

Animate with `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` on target `layer::fxN`.

### 7.3 `normal_map`

`{"data": base64-PNG | "path": file, "fit": "stretch" | "tile" | "exact" (default stretch),
"strength": 0..8 (default 1), "green_down": bool (default false)}`. Needs `data` or `path`. It feeds
the `relight` post effect; plain renders do not use it. As `Layer.normal_map` it is fitted to the pixel bounding box of that layer. `fit` `exact` demands a map of exactly the frame size and is an error otherwise.

## 8. Channels, tracks, easing

### 8.1 Channel

```json
{"target": "ball", "prop": "layer_y", "source": "keys",
 "track": {"keys": [{"frame": 0, "value": 40}, {"frame": 24, "value": 170}]},
 "mode": "replace", "weight": 1}
```

| JSON name | Type | Default | Meaning |
| --- | --- | --- | --- |
| `target` | string | `""` | Name, or `layer::sub` form. Ignored for camera props. |
| `prop` | string | **Req** | One of the snake_case props below. |
| `source` | `keys`, `expr`, `noise`, `driven` | **Req** | Flattened tag, with the fields of that source next to it. |
| `mode` | `replace`, `add`, `multiply` | `replace` | How the value combines with the current one. |
| `weight` | number | 1 | Influence, must be finite; blending toward the result. |

Combination: replace is `cur + (v - cur) * w`; add is `cur + v * w`; multiply is
`cur * (1 + (v - 1) * w)`. Non-finite values are skipped. Channels run in list order, so a later
channel sees the results of earlier ones (through `get(...)` and `driven`).

Sources:

| `source` | Fields |
| --- | --- |
| `keys` | `track`: `{keys: [{frame, value, ease?}], extrapolate?}` |
| `expr` | `expr`: string (section 9) |
| `noise` | `amp`, `freq` (cycles per second), `seed` (0), `octaves` (1). Output in `[-amp, amp]`; hash-based, deterministic. |
| `driven` | `input` (target), `input_prop`, `curve`: a track whose "frames" are input values (for example elbow angle to bulge amount). |

### 8.2 Props

There are 39 props. The prop decides what `target` means.

| Group | Props | `target` |
| --- | --- | --- |
| Bone | `bone_angle`, `bone_scale`, `bone_offset_x`, `bone_offset_y` | bone name |
| Layer | `layer_opacity`, `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`, `layer_skew`, `layer_switch`, `layer_wobble` | layer path (`parent/child`) |
| Camera | `camera_x`, `camera_y`, `camera_zoom`, `camera_rotation`, `camera_focus`, `camera_blur` | ignored |
| Rig | `ik_mix`, `constraint_weight` | ik or constraint name |
| Path | `path_morph`, `point_dx`, `point_dy`, `grad_dx`, `grad_dy` | `layer::path::morphName`, `layer::path::anchorIndex`, `layer::path::fill.from` / `fill.to` / `stroke.from` / `stroke.to` |
| Deformer | `lattice_dx/dy`, `spine_dx/dy`, `pin_dx/dy`, `cage_dx/dy` | `layer::deformer::c,r` (lattice, cloth) or `layer::deformer::i` |
| Effect | `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` | `layer::fxN` |

Gradient handle semantics: for linear gradients `.from` and `.to` move the start and end point by
`(dx, dy)`. For radial gradients `.from` moves the centre and `.to` changes the radius by `dx`.
Only paths whose fill or stroke is a gradient have these targets. `layer_switch` requires a
`selector` layer. An unknown prop lists every valid variant in the error.

Clamps on write: opacity 0..1, camera zoom at least 0.01, camera blur at least 0, `ik_mix` and
`constraint_weight` 0..1.

### 8.3 Track and key

`Track` = `{"keys": [...], "extrapolate": "hold"}`. A key is `{"frame": number, "value": number,
"ease": {...}}`. Keys must be finite and strictly increasing in frame:
`channel N ('target'): keys must be finite and strictly increasing in frame`. The ease on a key
shapes the segment that starts at that key.

Extrapolation (outside the first and last key): `hold` (default), `linear`, `loop`, `ping_pong`,
`loop_offset`.

### 8.4 Easing

`ease` is an object with a `type` tag. Default is `{"type": "linear"}`.

| `type` | Parameters | Curve |
| --- | --- | --- |
| `linear` | | `u` |
| `step` | | Hold the key value until the next key. |
| `ease_in` | | `u^2` |
| `ease_out` | | `1 - (1-u)^2` |
| `ease_in_out` | | Smoothstep. |
| `cubic_bezier` | `x1, y1, x2, y2` | CSS-style curve through (0,0) and (1,1). |
| `smooth` | | Non-uniform Catmull-Rom Hermite through neighbouring keys. |
| `spring` | `bounces`, `decay` | `1 - e^(-decay*u) * cos(2*pi*bounces*u) * window`, window = `1 - smooth01((u-0.85)/0.15)`; lands exactly on the target. |
| `back` | `overshoot` | Cubic with start slope `3 + overshoot` (clamped at 0 or more). Overshoot 1 peaks at 1.037 (u = 2/3); overshoot 2 peaks at 1.1317 (u = 0.5556). |
| `bounce` | | Restitution 0.6, three rebounds, total duration 3.352 time units. First impact at u = 0.2983; rebound peaks 0.64, 0.8704, 0.953344; ends at 1. |

`back` and `bounce` are original derivations (see `CLEANROOM.md`).

## 9. Expressions

A channel with `"source": "expr"` evaluates a formula per frame. Source is limited to 4096 bytes
and nesting depth 64; errors are reported at validation time with a position, for example
`channel 2 ('upper'): expression error: unexpected end of expression`.

Variables: `frame`, `time` (= frame / fps), `fps`, `frames`, `pi`, `t` (= frame / frames).

Operators (low to high precedence): `?:`, `||`, `&&`, comparisons, `+ -`, `* / %`, unary `- !`,
`^` (right-associative). Division by zero gives 0; `%` is Euclidean (`rem_euclid`).

Functions: `sin cos tan asin acos atan atan2 abs sqrt pow min max clamp lerp smoothstep floor ceil
round fract sign mod exp ln noise` (1 or 2 arguments) `fbm` (2 or 3) `wiggle` (2 or 3) `saw tri
square deg rad`, and `get("target", "prop")`. A non-finite function result becomes 0.

`get("target", "prop")` reads the current value of any property, including results of earlier
channels. Both strings must resolve at validation time (`channel 2: get("q", "bone_angle"): unknown
target`). A channel that reads a value must come after the channel that writes it.

Example from the ball project:

```json
{"target": "shadow", "prop": "layer_opacity", "source": "expr",
 "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"}
```

Binding expressions (section 11) use a restricted form: the variable is `value`, and the time
variables and `get()` are forbidden.

## 10. Camera, shots, audio, metadata

### 10.1 `camera`

| JSON name | Type | Default | Range | Meaning |
| --- | --- | --- | --- | --- |
| `position` | `Vec2` or null | null (stage centre) | | World point at the screen centre. |
| `zoom` | number | 1 | clamped to at least 0.01 | |
| `rotation` | number | 0 | degrees | |
| `focus` | number | 1 | | The layer `parallax` depth that is sharp (used by `dof` and `dof-export`). |
| `blur` | number | 12 | finite, `>= 0` | Blur radius in pixels per unit of parallax distance. |

Parallax: a layer with `parallax = p` sees `zoom = 1 + (zoom - 1) * p`,
`pos = centre + (pos - centre) * p`, `rotation = -rotation * p`. A non-finite `focus` or `blur`
gives `camera focus must be finite and camera blur a finite number >= 0`.

### 10.2 Metadata arrays (not validated at load)

These carry editor and tooling data. The engine accepts them as data; operations that use them
check them when they run.

| Array | Item fields |
| --- | --- |
| `markers` | `name`, `frame`, `end` (optional; a region when set) |
| `comps` | `name`, `layers`: map of layer path to `{visible, opacity, selected?}` |
| `events` | `name`, `frame`, `data` (any JSON) |
| `bone_vis` | `bone`, `from`, `to` (frame range in which the bone is shown) |
| `channel_meta` | `target`, `prop`, `color`, `shy`, `solo`, `lock` (editor flags; `lock` blocks set-key) |
| `shots` | `name`, `x`, `y`, `zoom`, `rotation` (saved camera views) |

```json
"markers": [{"name": "intro", "frame": 0, "end": 12}, {"name": "hit", "frame": 20}],
"events": [{"name": "step", "frame": 12, "data": {"sound": "step.wav"}}],
"shots": [{"name": "wide", "x": 200, "y": 150, "zoom": 1, "rotation": 0}]
```

### 10.3 `sounds[]`

| JSON name | Type | Range | Meaning |
| --- | --- | --- | --- |
| `name` | string | unique | |
| `file` | string | an asset path (section 18) | WAV or other supported audio. |
| `start` | number | seconds | Placement on the timeline. |
| `gain` | number | 0..4 | |
| `trim_in`, `trim_out` | number | `trim <= 86400` | Seconds cut from the start and end of the file. |
| `fade_in`, `fade_out` | number | fits the trimmed clip | Seconds. |

`check` (not `Engine::new`) reports a missing file: `sound 's': cannot read sound file ./nofile.wav: ...`.

## 11. Clips, skins, links, interaction, lods

### 11.1 `clips[]` and `clip_baked`

A clip places a reusable `take` on the timeline. Baking clips appends ordinary channels (after the
hand-made ones) and stores a snapshot in `clip_baked`, so re-baking is idempotent and hand work is
never deleted.

| Clip field | Meaning |
| --- | --- |
| `name`, `start`, `speed`, `loops` | Placement. |
| `take` | `{name, length, tracks: [{target, prop, keys}]}` |
| `mode` | `replace`, `additive`, `multiply` |
| `weight`, `weight_keys` | Influence; optional animated track. |
| `include`, `exclude` | Pattern lists selecting tracks. |
| `track_weights` | `[{pattern, prop, weight}]` |
| `fade_in`, `fade_out` | Frames. |
| `rename` | `[[from, to]]` target renames. |

### 11.2 `skins[]`

`{name, group, layers, constraints, iks, bones}`. Skins in the same `group` are exclusive variants.
Per-skin bone scale is collapsed into a multiply `bone_scale` step channel when baked. Skin rules
are only checked when skin operations run.

### 11.3 `mesh_locks[]` and `glues[]`

A glue pair uses `VertexRef` = `{layer, path (default 0), index}`; a mesh lock names whole paths with `{layer, path}` and an optional list of follower vertex indices (empty means every vertex).

```json
"mesh_locks": [{"name": "lock", "host": {"layer": "hostm", "path": 0},
                "follower": {"layer": "follm", "path": 0}, "vertices": [], "strength": 1}],
"glues": [{"name": "g", "mode": "rigid", "strength": 1,
           "pairs": [{"a": {"layer": "hostm", "path": 0, "index": 0},
                      "b": {"layer": "s1", "path": 0, "index": 0}}]}]
```

Glue `mode` is `snap`, `rigid` or `offset`. Both are validated by `links::resolve` and applied in
stage space in topological order (evaluation step 12).

### 11.4 `machines[]` and `bindings[]` (runtime only)

State machines and data bindings are driven by the Player at run time. For export and baking they
become constant `replace` channels appended to the project. Each machine needs at least one layer.

```json
"machines": [{"name": "m",
  "inputs": [{"name": "go", "kind": "bool", "default": 0},
             {"name": "fire", "kind": "trigger", "hold": false}],
  "layers": [{"name": "main", "entry": "idle", "mask": [],
    "states": [
      {"name": "idle", "source": {"kind": "region", "marker": "intro"},
       "mode": "loop", "speed": 1, "blend_in": 0.1},
      {"name": "wave", "source": {"kind": "clip", "clip": "wave"},
       "mode": "once", "speed": 1, "blend_in": 0.1}],
    "transitions": [
      {"from": "idle", "to": "wave", "when": [{"input": "go", "op": "eq", "value": 1}],
       "duration": 0.2, "ease": {"type": "ease_in_out"}},
      {"from": "wave", "to": "idle", "exit_time": 1}]}]}]
```

Input `kind` is `bool`, `number` or `trigger`. A state `source.kind` is `clip`, `segment`, `region`
or `blend`; `mode` is `loop`, `once` or `ping_pong`. `Transition` and `Condition` reject unknown
keys.

```json
"bindings": [
  {"name": "hp", "path": "player.health", "expr": "value/100",
   "remap": {"in_min": 0, "in_max": 1, "out_min": 0, "out_max": 1, "clamp": true,
             "curve": {"type": "linear"}},
   "smooth": 0.1,
   "target": {"kind": "prop", "target": "sw", "prop": "layer_opacity"},
   "mode": "replace", "weight": 1},
  {"name": "go", "path": "input.go",
   "target": {"kind": "input", "machine": "m", "input": "go"}}]
```

Binding `weight` must be within 0..1 (`binding 'b': weight must be within 0..1`). Machine and binding
targets are checked by a trial compile.

### 11.5 `containers[]` and `scrolls[]`

Flexbox-style 1-D layout. Container fields: `id`, `members`, `axis` (`row`, `column`), `gap`,
`cross_gap`, `padding` (4 numbers), `justify`, `align`, `wrap`, `rect`. A `rect` has a `kind`:
`fixed` (`x,y,w,h`), `pin`, `aspect` or `hug`. A member has exactly one of `layer`, `container` or
`spacer`, plus `grow`, `shrink`, `width`, `height`, `align`, `resize` and the optional `baseline`.
`align` is `start`, `center`, `end`, `stretch` or `baseline`. In a row container the members of a line
that choose `baseline` (on the container or per member) are shifted so that their baselines coincide;
`baseline` is the distance from the top edge of the member's box to its baseline (a number >= 0, not
allowed on a spacer; without it the baseline is the bottom edge of the box). The group sits at the
start of the line, and in a wrapped container it also sets the height of its line. In a column
container `baseline` acts as `start`. `baseline` is omitted when not set, so a project that does not
use it is unchanged. Limits: 100000 members, nesting depth 64. A scroll has `id`, `viewport`, `content`, `axis`, `offset`, `elastic`, `snap_x`,
`snap_y`, `fling`, `clip_layer`, `input`.

```json
"containers": [{"id": "row", "axis": "row", "gap": 4, "padding": [2,2,2,2],
  "justify": "start", "align": "start", "wrap": "none",
  "rect": {"kind": "fixed", "x": 0, "y": 0, "w": 200, "h": 60},
  "members": [{"layer": "c1"}, {"spacer": true, "grow": 1},
              {"layer": "c2", "grow": 0, "shrink": 1, "width": 30, "height": 30,
               "align": "center", "resize": true}]}],
"scrolls": [{"id": "sc", "viewport": {"kind": "fixed", "x": 0, "y": 100, "w": 100, "h": 80},
  "content": "list", "axis": "y", "offset": [0, 10],
  "elastic": {"limit": 40, "stiffness": 180, "damping_ratio": 0.9},
  "snap_x": [], "snap_y": [0, 40], "fling": 0.15, "clip_layer": "mask_rect", "input": "scroll.y"}]
```

Container offsets are added to the layer position and multiplied into scale (evaluation step 7).
`apply_clip_masks` adds mask references to the project it is given, so it mutates the engine's copy.

### 11.6 `lods[]`

A record of a level-of-detail pass written by `osatura lod`: `level`, `tolerance_px`, `scale`,
`declared_px`, `bound_stage`, `path_px`, `mesh_px`, `keys_px`, `cull_px`, `stats`
(`path_anchors`, `mesh_vertices`, `keys`, `layers`, `bytes`) and `notes`. It is documentation of what
a pass did; the renderer never reads it.


## 12. Validation rules, limits and error messages

`Engine::new` validates in one pass and returns all problems. The table lists what is enforced;
messages in quotes were produced by `osatura check` on deliberately broken files.

### 12.1 Global limits

| Rule | Limit | Message |
| --- | --- | --- |
| fps | `(0, 1000]` | `fps must be within (0, 1000]` |
| frames | `1..=200000` | `frames must be within 1..=200000` |
| width, height | `> 0` | `width and height must be > 0` |
| width, height | `<= 32768` | `width and height must be at most 32768` |
| any float | `abs <= 1e7` | `value 100000000.0 at /bones/0/length is outside the supported range of +-1e7 (coordinates, scales, offsets and key values must stay below it)` |
| bones | at most 4096 | `at most 4096 bones` |
| format | `<= 1` | `project format 2 is newer than this build supports (1); update OSATURA` |
| JSON size | CLI `--max-input-bytes`, default 256 MiB; API `MAX_JSON_BYTES` 256 MiB | |

The 1e7 limit is applied twice. At load, a float literal beyond it fails the parse and the message
is wrapped as `invalid JSON in <file>: value ...`. A literal without a decimal point (an integer)
passes the parse and is caught by the re-check in `Engine::new`, where it appears as an ordinary
list item. The limit exists so that a later multiplication cannot overflow `f32` into infinity.

### 12.2 Rig

| Rule | Message |
| --- | --- |
| bone parent must exist earlier | `bone 'lower' has parent 'nope' which is missing or listed after it` |
| bone length finite, `>= 0` | `bone 'lower' has invalid length` |
| `angle_limits` `min <= max`; `scale_limits` `0 < min <= max` | |
| ik needs 2 bones | `ik 'reach': needs at least two bones` |
| ik chain linked by parent | `ik 'reach': each bone must be a direct child of the previous one` |
| ik target not in chain | |
| soft, stretch | `ik 'reach': soft must be within 0..0.9 and stretch within 1..4` |
| constraint cannot target itself | |
| spring | `spring on 'upper': frequency must be in (0, 30] Hz and damping >= 0`; one per bone |

### 12.3 Layers, paths, geometry

| Rule | Message |
| --- | --- |
| layer name non-empty, no `/`, no `::` | `layer name 'a/b' must be non-empty and contain neither '/' nor '::'` |
| duplicate path | `duplicate layer path 'arm'` (and lines for the duplicated animatable targets) |
| path and deformer names contain no `::` and are unique within the layer | |
| morph anchor count equals the base path | |
| skin weights name known bones; weight finite, `>= 0` | |
| free mesh | at least 3 vertices, at least 1 triangle, at most 200000 / 400000, finite, valid indices, `uvs` empty or same count |
| grid mesh | `cols`, `rows >= 1` |
| mask | no self mask; `mask cycle involving layer 'arm'` |
| image source | `layer 'im': image has neither 'png_base64' nor 'file'` |
| liquid | at least 1 blob; `threshold > 0`; `cell >= 0.25`; blob `radius > 0` |
| boolean | at least 2 paths |
| emitter | `rate` in `(0, 2000]`; `lifetime > 0`; jitters 0..1 |
| deformers | only on vector and image layers; see section 7.1 for each type |
| effects | at most 16; blur and softness 0..64; offsets at most 1024; amounts 0..1 |
| normal map | `strength` 0..8; needs `data` or `path` |
| camera | `camera focus must be finite and camera blur a finite number >= 0` |

### 12.4 Channels

| Rule | Message |
| --- | --- |
| target resolves | `channel 0: unknown target 'zzz' for BoneOffsetX` |
| `layer_switch` on a switch layer | `channel 2: layer_switch needs a switch layer, 'arm' is not one` |
| weight finite | |
| keys finite and strictly increasing | `channel 0 ('target'): keys must be finite and strictly increasing in frame` |
| expression parses | `channel 2 ('upper'): expression error: unexpected end of expression` |
| `get()` targets and props resolve | `channel 2: get("q", "bone_angle"): unknown target` |
| driven input resolves | |

Unknown `prop` or `ease` names produce serde errors that list every valid variant, for example
``unknown variant `foo`, expected one of ...``. Wrong integer types produce
``invalid type: floating point `48.0`, expected u32``. A missing required field produces
``missing field `frames` ``.

### 12.5 Other validated sections

- Sounds: `gain` 0..4, `trim` at most 86400 s, fades fit inside the trimmed clip, names unique.
  File existence is checked by `check`, not `Engine::new`.
- Machines: at least one layer (`machine 'm': needs at least one layer`); transitions and
  conditions reject unknown keys. Bindings: `binding 'b': weight must be within 0..1`. Their targets
  are checked by compiling them once.
- Containers and scrolls: members unique, at most 100000, nesting depth 64, rect rules.
- Mesh locks and glues: references resolved by `links::resolve`.
- Image and audio files: section 18.

### 12.6 Editor data: warnings, and errors only for non-finite numbers

`Engine::new` and `check` look at `skins`, `comps`, `markers`, `events`, `shots`, `channel_meta`, `bone_vis`, `clips` and
`lods` (module `editordata`). To keep every previously saved project loading, **only values that cannot exist in a saved file
are errors**: NaN or infinity in markers, events, shots, `bone_vis`, comp opacity, lods and layer opacity. Everything else is
a **warning** printed by `check` (and reported by `lint` as L013): camera zoom `<= 0` (the engine clamps it to 0.01), layer
`opacity` outside 0..1 (clamped), `ik_mix` and `constraint_weight` outside 0..1 (clamped), keyed `layer_opacity` and
`camera_zoom` values outside their range, unknown layer/bone/channel references, duplicate names, times outside the
animation, invalid skins (`skins::validate`), clips (`clipstack::validate`, and take-track targets that do not resolve,
honouring `rename`), `channel_meta` and `lods`. The `out/` fixtures load with no warnings.

`osatura check` prints the lint warnings of this kind; the full linter (`osatura lint`) has more rules (L001 onward).

## 13. Example 1: bouncing ball

Verified with `osatura check` (`ok: 'bouncing ball' 320×240 @ 24 fps, 48 frames, 0 bones, 0 ik,
3 layers, 5 channels`) and `osatura render ball.json 7 out.png`. What it shows:

- Keyed `layer_y` with `bounce` easing on the first segment: the ball falls from y=40 to the
  floor at y=170 and bounces three times. The first impact lands at about frame 7.2 (24 / 3.352).
  The segment from 24 to 47 rises back with `ease_out`.
- A shadow that follows the ball: `layer_x` and `layer_opacity` are expressions that read the
  ball with `get()`. They are listed after the ball's own channels, because channels run in order.
- Squash on contact: an expression returns 0.8 for `layer_scale_y` when the ball is below y=160.
- Circle from four Bezier anchors with handle length 16.57 (0.5523 times the radius 30).

```json
{
  "format": 1,
  "name": "bouncing ball",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "floor",
      "type": "vector",
      "paths": [
        {
          "name": "line",
          "closed": false,
          "anchors": [{"pos": [0, 200]}, {"pos": [320, 200]}],
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "shadow",
      "type": "vector",
      "paths": [
        {
          "name": "blob",
          "closed": true,
          "anchors": [
            {"pos": [-26, 0], "handle_out": [-26, -5], "handle_in": [-26, 5]},
            {"pos": [0, -6], "handle_in": [-14, -6], "handle_out": [14, -6]},
            {"pos": [26, 0], "handle_in": [26, -5], "handle_out": [26, 5]},
            {"pos": [0, 6], "handle_in": [14, 6], "handle_out": [-14, 6]}
          ],
          "fill": "#00000055"
        }
      ],
      "transform": {"position": [160, 200]}
    },
    {
      "name": "ball",
      "type": "vector",
      "paths": [
        {
          "name": "disc",
          "closed": true,
          "anchors": [
            {"pos": [30, 0], "handle_in": [30, -16.57], "handle_out": [30, 16.57]},
            {"pos": [0, 30], "handle_in": [16.57, 30], "handle_out": [-16.57, 30]},
            {
              "pos": [-30, 0],
              "handle_in": [-30, 16.57],
              "handle_out": [-30, -16.57]
            },
            {
              "pos": [0, -30],
              "handle_in": [-16.57, -30],
              "handle_out": [16.57, -30]
            }
          ],
          "fill": {
            "center": [-8, -10],
            "radius": 44,
            "stops": [
              {"offset": 0, "color": "#ffd9a0"},
              {"offset": 1, "color": "#d6336c"}
            ]
          },
          "stroke": {"paint": "#3d2c2e", "width": 2}
        }
      ],
      "transform": {"position": [160, 40]}
    }
  ],
  "channels": [
    {
      "target": "ball",
      "prop": "layer_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 40, "ease": {"type": "bounce"}},
          {"frame": 24, "value": 170, "ease": {"type": "ease_out"}},
          {"frame": 47, "value": 40}
        ]
      }
    },
    {
      "target": "ball",
      "prop": "layer_x",
      "source": "keys",
      "track": {"keys": [{"frame": 0, "value": 60}, {"frame": 47, "value": 260}]}
    },
    {
      "target": "shadow",
      "prop": "layer_x",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_x\")"
    },
    {
      "target": "shadow",
      "prop": "layer_opacity",
      "source": "expr",
      "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"
    },
    {
      "target": "ball",
      "prop": "layer_scale_y",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_y\") > 160 ? 0.8 : 1",
      "mode": "replace"
    }
  ]
}
```

## 14. Example 2: rigged arm with IK

Verified with `osatura check` (3 bones, 1 ik, 3 layers, 2 channels). `osatura explain arm.json
24` reports the tip at (170, 190), equal to the `target` bone, and `osatura render arm.json 24
out.png --bones` draws the skeleton. What it shows:

- `upper` is a root with an angle limit; `lower` is its child (`rest_offset` `[90, 0]` equals the
  parent's length); `target` is a free root bone that the IK chain reaches for.
- `reach` is a two-bone IK chain, so the analytic solver is used.
- `sleeve` is skinned: each anchor lists bone weights. `hand` and `goal` are attached with
  `bone`.
- Moving the `target` bone with `bone_offset_x` and `bone_offset_y` channels makes the arm reach.

```json
{
  "format": 1,
  "name": "rigged arm",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "bones": [
    {
      "name": "upper",
      "parent": null,
      "length": 90,
      "rest_offset": [60, 120],
      "rest_angle": 0,
      "angle_limits": [-120, 120]
    },
    {
      "name": "lower",
      "parent": "upper",
      "length": 80,
      "rest_offset": [90, 0],
      "rest_angle": 0
    },
    {
      "name": "target",
      "parent": null,
      "length": 10,
      "rest_offset": [230, 120],
      "rest_angle": 0
    }
  ],
  "ik": [
    {
      "name": "reach",
      "bones": ["upper", "lower"],
      "target": "target",
      "bend": 1,
      "mix": 1
    }
  ],
  "layers": [
    {
      "name": "arm",
      "type": "vector",
      "paths": [
        {
          "name": "sleeve",
          "closed": true,
          "anchors": [
            {"pos": [60, 106], "weights": [{"bone": "upper", "weight": 1}]},
            {
              "pos": [150, 104],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [228, 110], "weights": [{"bone": "lower", "weight": 1}]},
            {"pos": [228, 130], "weights": [{"bone": "lower", "weight": 1}]},
            {
              "pos": [150, 136],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [60, 134], "weights": [{"bone": "upper", "weight": 1}]}
          ],
          "fill": "#ffb347",
          "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
        }
      ]
    },
    {
      "name": "hand",
      "type": "vector",
      "bone": "lower",
      "paths": [
        {
          "name": "palm",
          "closed": true,
          "anchors": [
            {"pos": [246, 120], "handle_in": [246, 111], "handle_out": [246, 129]},
            {"pos": [232, 134], "handle_in": [241, 134], "handle_out": [223, 134]},
            {"pos": [218, 120], "handle_in": [218, 129], "handle_out": [218, 111]},
            {"pos": [232, 106], "handle_in": [223, 106], "handle_out": [241, 106]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "goal",
      "type": "vector",
      "bone": "target",
      "paths": [
        {
          "name": "ring",
          "closed": false,
          "anchors": [{"pos": [222, 120]}, {"pos": [238, 120]}],
          "stroke": {"paint": "#d6336c", "width": 3, "dash": [4, 4]}
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "target",
      "prop": "bone_offset_x",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": -60, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    },
    {
      "target": "target",
      "prop": "bone_offset_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": 70, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    }
  ]
}
```

## 15. Example 3: switch-layer mouth

Verified with `osatura check` and by rendering frames 0, 5 and 9, which show three different
mouths. Out-of-range keys clamp: a key value of 7 shows child 2 and -3 shows child 0.

```json
{
  "format": 1,
  "name": "switch mouth",
  "width": 200,
  "height": 160,
  "fps": 12,
  "frames": 25,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "face",
      "type": "vector",
      "paths": [
        {
          "name": "head",
          "closed": true,
          "anchors": [
            {"pos": [180, 80], "handle_in": [180, 35], "handle_out": [180, 125]},
            {"pos": [100, 150], "handle_in": [145, 150], "handle_out": [55, 150]},
            {"pos": [20, 80], "handle_in": [20, 125], "handle_out": [20, 35]},
            {"pos": [100, 10], "handle_in": [55, 10], "handle_out": [145, 10]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "mouth",
      "type": "selector",
      "selected": 0,
      "layers": [
        {
          "name": "closed",
          "type": "vector",
          "paths": [
            {
              "name": "line",
              "closed": false,
              "anchors": [{"pos": [75, 110]}, {"pos": [125, 110]}],
              "stroke": {"paint": "#3d2c2e", "width": 4, "cap": "round"}
            }
          ]
        },
        {
          "name": "open",
          "type": "vector",
          "paths": [
            {
              "name": "oval",
              "closed": true,
              "anchors": [
                {"pos": [75, 106]},
                {"pos": [125, 106]},
                {
                  "pos": [112, 132],
                  "handle_in": [122, 132],
                  "handle_out": [102, 134]
                },
                {"pos": [88, 132], "handle_in": [98, 134], "handle_out": [78, 132]}
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
            }
          ]
        },
        {
          "name": "round",
          "type": "vector",
          "paths": [
            {
              "name": "o",
              "closed": true,
              "anchors": [
                {
                  "pos": [112, 118],
                  "handle_in": [112, 110],
                  "handle_out": [112, 126]
                },
                {
                  "pos": [100, 130],
                  "handle_in": [106, 130],
                  "handle_out": [94, 130]
                },
                {"pos": [88, 118], "handle_in": [88, 126], "handle_out": [88, 110]},
                {
                  "pos": [100, 106],
                  "handle_in": [94, 106],
                  "handle_out": [106, 106]
                }
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3}
            }
          ]
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "mouth",
      "prop": "layer_switch",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "step"}},
          {"frame": 4, "value": 1, "ease": {"type": "step"}},
          {"frame": 8, "value": 2, "ease": {"type": "step"}},
          {"frame": 12, "value": 1, "ease": {"type": "step"}},
          {"frame": 16, "value": 0, "ease": {"type": "step"}}
        ]
      }
    }
  ]
}
```

## 16. Evaluation order

`Engine::evaluate(frame)` is a pure function of the frame number and the project. Springs are
simulated once when the engine is built; nothing else carries state between frames. The steps:

1. **Initial state** (`initial_state`): static layer transforms, opacity clamped 0..1, switch
   `selected`, wobble amplitude, camera, `ik` mix, constraint weights, and initial offsets of the
   parameter slots (bone offsets, deformer offsets, morph weights and so on).
2. **Channels** (`apply_channels`) in list order, each combining with the current value by its
   `mode` and `weight`. Non-finite results are skipped. Writes clamp as listed in section 8.2.
3. **Limit clamp** (`clamp_limits`): `angle_limits` and `scale_limits` per bone.
4. **Forward kinematics**: world matrices of all bones, parents first.
5. **IK chains** in list order, skipping chains with `mix <= 0`. After each chain the pose is clamped
   and FK recomputed, so the next chain sees the result.
6. **Constraints** in list order, skipping `weight <= 0`: aim, copy_rotation, pin.
7. **Layout containers**: offsets are added to layer x and y and multiplied into scale.
8. **Baked springs**: the baked angles replace the bone angle, then FK is recomputed.
9. **Skin matrices**: `skin_mat = world * inverse(rest world)` per bone.
10. **Camera**: one camera matrix per root layer, using the root layer's `parallax` (section 10.1).
11. **Nodes** in pre-order. A node's stage matrix is `parentStage * boneAttach(skin_mat) * local`.
    Vector geometry is built in this order:
    1. morphs (additive blend shapes);
    2. per-anchor `point_dx/dy` offsets;
    3. gradient handle shifts (`grad_dx/dy`);
    4. deformers in list order, then `wobble`;
    5. skinning with normalized weights (empty weights mean static; gradient handles are skinned
       too).
    Skinned geometry lives in stage rest coordinates, and the layer transform is applied after it.
    A switch shows one child (rounded, clamped). Liquid, boolean and emitter geometry is
    synthesized per frame. An image gets a grid or free mesh; with deformers but no mesh it gets a
    16 by 16 grid.
12. **Mesh locks and glues** in stage space, in topological order.
13. **Non-finite repair** (`repair_nonfinite`): a node whose geometry contains NaN or infinity is
    hidden and `Frame.nonfinite` is set. `evaluate_checked` and `check_all_frames_finite` turn that
    into an error.

Consequences you can rely on:

- A channel sees only channels before it in the list (through `get` and `driven`).
- IK overrides FK for the bones in the chain, in proportion to `mix` (the blend is
  `old + wrap_pi(new - old) * mix`). With `mix` 0 the chain is pure FK.
- Constraints run after IK, so an aim constraint on an IK bone wins.
- Springs act last on the bone angle, after IK and constraints.

## 17. Rendering order

- Root layers are drawn in list order, children in list order inside their parent.
- A node that is not `visible` is skipped (it can still be a mask source).
- A node is rendered to an offscreen layer when it has `opacity < 1`, effects, a blend mode other
  than `normal`, or a mask. Steps: content; layer effects in list order (on a padded canvas); mask
  (the mask layer is drawn with its own transform; its alpha, including its opacity, multiplies;
  `invert` flips it); composite with opacity and blend mode.
- Depth of field is applied only by `dof` and `dof-export`: radius = `abs(parallax - focus) *
  blur`, at most 24 pixels by default.
- The bone overlay (`--bones`) is drawn last.

## 18. Assets and paths

Projects may reference files: `image.source.file`, `normal_map.path`, `sounds[].file`.

- Paths must be relative, must not contain `..`, and after following symlinks must stay inside the
  project's directory: `layer 'im': asset '/etc/passwd': absolute paths are not allowed (a project
  may only reference files inside its own directory; use --allow-external-assets to override)`, and
  `layer 'im': asset '../x.png': '..' is not allowed (...)`.
- `--allow-external-assets` lifts that restriction for a single command. Only regular files are
  opened.
- Size caps: images 64 MB on disk, audio 256 MB. Decoded images: each side at most 16384 and at
  most 64 megapixels.
- Output caps: raster at most 16384 per side and 100 megapixels; sequences at most 2 GiB; export
  budget frames times anchors at most 40 million.
- When the CLI writes a project it re-validates through `Engine::new` and rewrites asset paths
  relative to the output location.

Prefer `png_base64` for a self-contained file.

## 19. Versioning policy

- `format` is an integer, currently `1` (`FORMAT_VERSION`).
- A file with a larger `format` than the build supports is refused with the message in section 12.1;
  the engine does not guess.
- Adding an optional field with a default is not a version change. Older builds ignore it (and
  `check` warns about it as an unknown field).
- Removing or reinterpreting a field requires a bump, and the bump requires a migration step in
  `Project::from_json` that upgrades older documents before validation.
- The Rust library API has its own version, `API_VERSION` 1.4.0, guarded by the snapshot test
  `api_surface.txt`; the C ABI has `ABI_VERSION` 1. See `docs/API.md`.

## 20. Known gaps between code and comments

These were found while writing this reference.

1. **Fixed.** The switch-key rule (`lint::switch_key_warnings`, L012) now says what the engine does: a key below 0 shows child 0,
   a key at or above the child count shows the last child (an empty switch shows nothing). Test: `lint` unit tests, verified
   against the engine.
2. The comment in `project.rs` says files without `format` are version 1, but the serde default is
   `FORMAT_VERSION`. They agree today only because the constant is 1. When the constant is raised, a
   file with no `format` would be read as the newest version.
3. **Fixed.** `unknownfields` no longer reports an explicitly written empty structure (`clips[].weight_keys: {"keys": []}`);
   `default_like` is recursive, and a test pins the set of optional structures the writer omits when empty.
4. **Fixed (as warnings).** The editor fields are validated; see section 12.6.
5. **Fixed in wave 7 (L-71).** A skinned layer's geometry is in stage rest space and the parent transform is applied after
   it. `Layer.bone` now attaches with `world = bone_skin * parent_stage * local`: the skin matrix (bone world x inverse rest) is
   a stage-space motion, so it is applied after the whole parent chain. At rest the skin matrix is the identity, so nothing
   moves; when the bone is **posed**, a parent group with scale 2 no longer doubles the travel of an attached layer. A layer
   without a bone of its own follows the bone of its nearest bone-attached ancestor; a layer's own bone replaces the ancestor's
   (the two motions are not stacked). Tests: `bone_attach_under_a_transformed_group_composes_bone_after_the_group`,
   `bone_attach_is_inherited_and_a_child_bone_replaces_the_parents`. Projects that relied on the old doubled travel (a posed
   bone under a scaled or rotated group) render differently; projects without that combination are unchanged.
