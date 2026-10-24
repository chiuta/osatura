# OSATURA Rust API and stability policy

*Facade version 1.4.0 (`osatura_core::api::API_VERSION`). Audience: Rust developers who embed the engine, and maintainers who change the facade. Documentation licence CC-BY-4.0; code samples MPL-2.0.*

This is the reference for the **supported Rust API** of `osatura-core`: the facade module `osatura_core::api` and its one-line import `osatura_core::prelude`. It lists every public item, the rules that keep the API stable, and tested examples. Every example in this file is compiled and run against the current sources before the file is published (see [How the examples are checked](#how-the-examples-are-checked)).

Contents: [What is covered](#what-is-covered) - [Getting it](#getting-the-crate) - [Stability policy](#stability-policy) - [Semver rules](#semver-rules) - [Feature flags](#feature-flags) - [Thread-safety](#thread-safety) - [Determinism](#determinism) - [Limits](#limits-and-argument-ranges) - [Errors](#errors) - [Reference: `Animation`](#reference-animation) - [Reference: `Player`](#reference-player) - [Reference: free functions](#reference-free-functions-tools-version-control-ci) - [Reference: types](#reference-types) - [Examples](#examples) - [Other languages](#other-languages) - [Appendix: machine and binding JSON](#appendix-machine-json-and-binding-json)

## What is covered

Only `osatura_core::api` and `osatura_core::prelude` are the supported public Rust API. The facade consists of

* `Animation`: a validated project plus its undo history (load, query, render, export, edit, tools);
* `Player`: a running state machine with layout, scrolling and level of detail (1.1, 1.2);
* free functions for version control and animation CI (1.2 read, 1.3 write) and bitmap tracing;
* plain value types (`AnimationInfo`, `RenderSettings`, `Image`, `LodLevel`, `SizeFit`, `MocapReport`, `FuzzSummary`, `VcsCommit`, `VcsConflict`, `CiOutcome`, `CiInit`, ...) and the enums `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome`, `VcsChoice`;
* the error type `ApiError` (implements `std::error::Error` and `Display`).

The C ABI (`osatura-capi`, header `osatura.h`) and the WebAssembly module (`osatura-wasm`) are written against this facade only and have their own versions: [C_ABI.md](C_ABI.md), [WASM.md](WASM.md).

### What is NOT covered

Every other module of `osatura-core` (`engine`, `project`, `edit`, `svg`, `raster`, `physics`, `vcs`, `ci`, ...) and the root re-exports (`Engine`, `Project`, ...). They are `pub` and `#[doc(hidden)]` only because the command-line tool, the studio, the C ABI tests and the unit tests use them; they can change or vanish in any release, and their `Err(String)` texts are not a contract either. If you need something that is only reachable there, ask for a facade addition (a minor release) instead of depending on internals.

The project JSON format has its own `format` version field (currently 1); a project written by a newer build is refused with `InvalidJson`. See [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Getting the crate

OSATURA is not published to a registry; depend on it by path (or git) from your workspace:

```toml
[dependencies]
osatura-core = { path = "../osatura/osatura-core", default-features = false }
```

`default-features = false` drops the Rhai scripting engine (see [Feature flags](#feature-flags)); the facade does not need it. Then `use osatura_core::prelude::*;`. The crate version (`0.1.0`) and the facade version (`API_VERSION`) are independent: the facade follows semver, the crate version does not yet promise anything. No minimum supported Rust version is declared; the workspace is developed and tested with Rust 1.97 (edition 2021).

## Stability policy

1. **The surface file.** `osatura-core/src/api_surface.txt` lists every public item of `src/api/` (types, fields, variants, constants and method signatures, one per line). The unit test `api::tests::api_surface_snapshot` rebuilds that list from `api/` and fails when it differs from the checked-in file. Every change of the surface is therefore a deliberate, reviewed edit of both files.
2. **The prelude test.** `osatura-core/tests/prelude_surface.rs` is compiled as an *external* crate. It imports facade names through `osatura_core::prelude` only and calls a representative subset (load, render, edit, export, `Player`, the tools, the version-control and CI wrappers, bad-input handling). If a name it uses stops being re-exported from the prelude, or a signature changes in a way those calls notice, that test stops compiling. It also asserts `API_VERSION == "1.4.0"`, so a version bump is a conscious edit there as well. It does not enumerate every prelude name, so adding a name to the re-export list remains a manual step.
3. **Limits of the check.** The snapshot is a text scan of single-line signatures in `api/`, not a compiler-level diff such as `cargo-public-api`: a semantic change that keeps the signature text (for example a different error for the same input) is not caught, and a multi-line signature would be missed (the file keeps signatures on one line for that reason). The prelude re-export list (`lib.rs`) is checked by the external test, not by the snapshot.
4. **Hidden does not mean supported.** `#[doc(hidden)] pub` modules exist for in-workspace consumers. They are outside the promise even though they compile for you.
5. **How to change the facade** (maintainers; also in [DEVELOPMENT.md](DEVELOPMENT.md)):
   * edit `api/` (one-line signatures, doc comment, a doc-test for new free functions, unit tests);
   * add new public names to the `prelude` re-export list in `lib.rs`, and to `tests/prelude_surface.rs`;
   * bump `API_VERSION` (and `API_VERSION_MAJOR` if breaking) according to the rules below, and the version asserted in `prelude_surface.rs`;
   * regenerate the surface file deliberately: `OSATURA_BLESS=1 cargo test -p osatura-core api::` and review the diff of `api_surface.txt`;
   * update this file, and `osatura-capi` / `osatura-wasm` only if the new item should cross those boundaries (their ABI version moves separately, additions keep it).

## Semver rules

`API_VERSION` is a semantic version of the facade only.

| Change | Release |
|---|---|
| Bug fix, no signature change, no new behaviour contract | patch (1.4.x) |
| New function, method, type, enum variant, struct field, constant | minor (1.x.0) |
| Removing or renaming anything; changing a signature, a documented argument range in a way that rejects previously valid input, or the meaning of a documented result | major (2.0.0) |

Consequences for callers:

* **Public enums and structs of the facade are `#[non_exhaustive]`.** Add a wildcard arm when you `match` on `ApiError`, `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome` or `VcsChoice`; a new variant is a minor release. You cannot construct the structs with a literal outside the crate (and `Image`, `SizeFit` and the report structs are results you only read). Settings are built with constructors: `RenderSettings::new(scale).with_bones(true)`, `MocapSettings::new().with_smooth(2)`.
* **Strings inside results are data, not contract.** `VcsConflict::kind` is one of `"BothChanged"`, `"DeleteEdit"`, `"KeyRetimed"`, `"OrderConflict"`, `"ArrayConflict"` today and *may gain values in minor releases*. The JSON reports (`FuzzSummary::json`, `CiOutcome::json`) carry their own schema names (`osatura-fuzz-report` v1, `osatura-ci/1`). Error message texts are for people.
* **Behavioural stability.** Rendering and evaluation are deterministic (see below), but pixel output may change in a minor release when a bug in the rasteriser or an effect is fixed; goldens recorded by animation CI are a reminder of that. Do not hash rendered pixels across OSATURA versions or across machines without a tolerance.

Version history of the facade: 1.0 (load, query, render, export, edit, undo) - 1.1 (`Player`, state machines, data bindings) - 1.2 (layout, scrolling and level of detail on `Player`; `lod_levels`, `fit_to_size`, `retarget_bvh`, `fuzz`, `from_image`, `trace_image_svg`, `vcs_log`, `ci_run`) - 1.3 (version-control and CI write operations, `DirtyTree` / `MergeInProgress` / `Conflicts`, `CiOutcome::updated` / `blessed`) - 1.4 (layer and bone creation, `bake_physics`, APNG and EXR export, `vcs_branches` / `vcs_branch_delete` / `vcs_status` / `vcs_diff` / `vcs_show` / `vcs_gc` / `vcs_fsck`).

## Feature flags

`osatura-core` has one Cargo feature:

| Feature | Default | Effect |
|---|---|---|
| `scripting` | on | Compiles the Rhai-based sandboxed scripting module (`script.rs`), used only by the command-line `script` command. Pulls `rhai` and through it `smartstring` (MPL-2.0+, the only weak-copyleft third-party crate in the tree). |

The facade has **no** functionality behind a feature: every item documented here exists with `default-features = false`. `osatura-capi`, `osatura-wasm` and `osatura-gpu` depend on the core without the feature, so they contain no MPL third-party crate; `osatura-cli --no-default-features` does the same and then reports the `script` command as unavailable. Rendering back ends are not features: the CPU rasteriser is always used by the facade; the optional GPU renderer is a separate crate (`osatura-gpu`) and a CLI feature (`gpu`) and is not reachable through this API.

## Thread-safety

* `Animation`, `Player`, `ApiError` and `Image` are `Send + Sync` (checked at compile time for this document). The facade writes no global state; it only reads two process-wide asset settings (the default asset directory and the external-assets switch) that nothing in the facade changes.
* Methods that take `&self` (`info`, `bone_world`, `render_*`, `export`, `lod_levels`, `fit_to_size`, `fuzz`, `player`, `to_json`, ...) can run concurrently on several threads over the same `Animation`; methods that take `&mut self` (`set_key`, `delete_key`, `undo`, `redo`, `retarget_bvh`, every `Player` input and `advance`) need exclusive access, as usual in Rust. The `render_in_parallel` example below renders every frame on its own thread and checks that the pictures equal the single-threaded ones.
* There is no internal locking and no background thread. A `Player` works on its own copy of the project, so editing the `Animation` later never affects a running player.
* `Animation::from_json` resolves relative image files against the process's **current directory** and refuses absolute paths and `..` (see [SECURITY.md](SECURITY.md)); embed images as data URIs, or set the working directory before loading, if a project references files.
* The version-control and CI functions work on files: they are not safe against another process editing the same project at the same moment (the facade pre-checks for a dirty tree and a pending merge, but there is no lock).

## Determinism

Same project, same arguments, same OSATURA build and machine: same bytes. This holds for `render_rgba` / `render_png` / `render_svg`, `bone_world`, every export, `lod_levels` and `fuzz` (for a given seed), `fit_to_size` and for a `Player` (the same inputs and the same `advance` calls give the same pose; `advance` splits time into pieces of at most 1/60 s, so how you chunk time matters only below that resolution). Random choices inside the engine (noise, particles, fuzzing, tracing clusters) use seeded generators, never the clock. What is *not* promised: identical anti-aliased edge pixels between different operating systems, CPUs, compilers or `libm` versions, between 32- and 64-bit targets, or between OSATURA versions. Byte-exact goldens were verified only across processes on one Linux x86-64 machine (see [LIMITATIONS.md](LIMITATIONS.md)); use tolerances (animation CI does) for anything cross-machine.

## Limits and argument ranges

| Item | Rule |
|---|---|
| `MAX_JSON_BYTES` | 256 MiB; a longer text is `InvalidJson` |
| `MAX_RENDER_SIDE` | 16384 output pixels per side (`render_*` and `Player::render_*`) |
| Render / GIF scale | finite and within `0.05..=4`, else `InvalidArgument` |
| Frame positions | must be finite; values outside the timeline are **clamped** to `0..=frames-1` (no error). Fractional frames are fine. |
| Keys (`set_key`, `delete_key`) | `frame` and `value` finite; `prop` is a snake_case property name from the project JSON (`"bone_angle"`, `"layer_opacity"`, ...) |
| Project | `fps` in `(0, 1000]`, `frames` in `1..=200000`, stage up to 32768 units per side, at most 4096 bones, numbers beyond +-1e7 refused at load; further per-feature limits are validated at load and come back as `InvalidProject` |
| `Player::advance(dt)` | `dt` finite, `0..=10` seconds |
| `Player::set_canvas_size` | both sides finite, `> 0`, at most 32768 |
| `Player::set_screen_size` | width and height finite and `> 0`; `max_error_px` finite and `>= 0` |
| `Animation::lod_levels` | `tolerance_px > 0`; `scale` in `(0, 64]`; `levels` in `1..=12` |
| `Animation::fit_to_size` | `max_bytes` in `1000..=1_000_000_000` |
| `Animation::fuzz` | `samples` in `1..=100000` |
| `trace_image_svg`, `Animation::from_image` | `colors` in `2..=64`; PNG or JPEG bytes (size caps apply) |
| `vcs_log` | `limit` in `1..=10000` |

## Errors

Every fallible function returns `Result<_, ApiError>`. Bad input is an error value and **never panics**; a panic that does happen inside the engine is caught at the facade boundary and returned as `ApiError::Internal` (please report it as a bug). `ApiError` implements `Display` and `std::error::Error`, so `?` works with `Box<dyn Error>` and `anyhow`-style types.

| Variant | Meaning | Typical sources |
|---|---|---|
| `InvalidJson(String)` | not JSON, not a project, wrong shape, too large (> `MAX_JSON_BYTES`), or a `format` newer than this build; also a bad JSON value passed to `vcs_resolve` | `Animation::from_json`, `validate_json`, `vcs_resolve` |
| `InvalidProject(Vec<String>)` | parsed, but validation failed; one message per problem (unknown bone, bad range, cycle, ...) | `from_json`, `from_image`, `Player` engine rebuilds |
| `InvalidArgument(String)` | a caller argument was rejected: non-finite number, out-of-range value, unknown property, unknown machine / input / scroll id, unusable BVH or image, bad branch name, nothing to commit, no repository | almost every function |
| `EditRejected(Vec<String>)` | an edit was refused (for example a locked channel or layer, or an invalid result); the project is unchanged | `set_key`, `delete_key`, `retarget_bvh` |
| `Export(String)` | an encoder failed or the format is not possible for this project | `export`, `render_png`, `fit_to_size` |
| `Internal(String)` | a caught panic | any |
| `DirtyTree(String)` | (1.3) the project file has uncommitted changes that the operation would overwrite | `vcs_checkout` (unless `force`), `vcs_merge`, `vcs_cherry_pick`, `vcs_revert` |
| `MergeInProgress(String)` | (1.3) a merge, cherry-pick or revert is unfinished; resolve and commit, or `vcs_abort` | same functions |
| `Conflicts(Vec<String>)` | (1.3) `vcs_commit` while conflicts remain; one pointer per conflict | `vcs_commit` |

A failing animation-CI check is **not** an error: `ci_run` returns `Ok(CiOutcome)` with `exit_code` 1 (a check failed) or 2 (a check could not be evaluated); only an unusable manifest is `Err`. Merge conflicts are likewise a result (`VcsMergeOutcome::Conflicted`), not an error.

## Reference: `Animation`

A loaded, validated project plus its undo history. Create it with `from_json`, `demo` or `from_image`. All methods that read the project take `&self`.

| Method | Returns | Notes |
|---|---|---|
| `Animation::from_json(&str)` | `Result<Animation, ApiError>` | parse + validate. `InvalidJson` / `InvalidProject`. Unknown JSON fields are ignored silently here (the CLI `check` command reports them). |
| `Animation::validate_json(&str)` | `Result<(), ApiError>` | same checks, keeps nothing |
| `Animation::demo()` | `Animation` | small built-in rig (6 bones, 48 frames at 24 fps, 640 x 480); handy for tests and first steps |
| `Animation::from_image(&[u8], colors, name)` | `Result<Animation, ApiError>` | traces PNG/JPEG bytes into filled vector layers on a stage of the image size |
| `to_json(&self)` | `String` | the current project as pretty-printed JSON (round-trips through `from_json`) |
| `info(&self)` | `AnimationInfo` | `name`, `width`, `height` (stage units), `fps`, `frames` |
| `frame_count(&self)`, `fps(&self)` | `u32`, `f32` | |
| `bone_names(&self)` | `Vec<String>` | rig order = order of `bone_world` |
| `layer_names(&self)` | `Vec<String>` | layer paths `group/child`, depth first |
| `bone_world(&self, frame)` | `Result<Vec<[f32; 6]>, ApiError>` | world matrix per bone `[a b c d tx ty]`; a bone-local point `(x, y)` maps to `(a*x + c*y + tx, b*x + d*y + ty)`. Frame clamped. Evaluates the whole rig (IK, constraints, springs, expressions). |
| `machine_names(&self)` | `Vec<String>` | runtime state machines stored in the project |
| `player(&self, machine)` | `Result<Player, ApiError>` | starts the named machine at time 0 on a copy of the project |
| `render_rgba(&self, frame, &RenderSettings)` | `Result<Image, ApiError>` | straight-alpha RGBA8, row-major, top row first |
| `render_png(&self, frame, &RenderSettings)` | `Result<Vec<u8>, ApiError>` | a PNG file in memory |
| `render_svg(&self, frame, show_bones: bool)` | `Result<String, ApiError>` | an SVG document (vector output has no scale) |
| `export(&self, ExportFormat, scale)` | `Result<Vec<u8>, ApiError>` | `Gif` (looping, whole timeline; `scale` within `0.05..=4`), `Glb` (binary glTF 2.0), `Lottie` (JSON bytes), `Html` (one self-contained player page); `scale` is ignored by the other three. What each format keeps or drops: [EXPORT_FORMATS.md](EXPORT_FORMATS.md). |
| `set_key(&mut self, target, prop, frame, value, Interpolation)` | `Result<(), ApiError>` | set or add one keyframe on `target.prop`; validated, undoable, respects channel and layer locks (`EditRejected`) |
| `delete_key(&mut self, target, prop, frame)` | `Result<bool, ApiError>` | `true` if a key was removed |
| `add_rect_layer(&mut self, name, x, y, w, h, fill, parent)`, `add_ellipse_layer(&mut self, name, cx, cy, rx, ry, fill, parent)` | `Result<(), ApiError>` | (1.4) append a filled shape layer (to the top level, or into the top-level group named by `parent`); unique name without `/`, finite positive sizes, a valid colour; undoable |
| `add_group_layer(&mut self, name)` | `Result<(), ApiError>` | (1.4) append an empty group layer |
| `add_bone(&mut self, name, parent, length, x, y, angle)` | `Result<(), ApiError>` | (1.4) append a bone (`parent` must exist, angle in degrees); undoable |
| `bake_physics(&mut self, chain, preset, overwrite)` | `Result<PhysicsBake, ApiError>` | (1.4) simulate a bone chain (root first) with `hair`, `tail`, `rope`, `antenna` or `cape` and store angle channels as one undo step; with `overwrite = false` hand-keyed bones are kept and listed in `skipped` |
| `render_exr(&self, frame, &RenderSettings, float32)` | `Result<Vec<u8>, ApiError>` | (1.4) OpenEXR file (linear, premultiplied; HALF or FLOAT) from the float compositor |
| `undo(&mut self)`, `redo(&mut self)` | `bool` | `false` when there is nothing to undo / redo |
| `lod_levels(&self, tolerance_px, scale, levels)` | `Result<Vec<LodLevel>, ApiError>` | level 0 is the original, level k >= 1 stays within `tolerance_px * 2^(k-1)` output pixels at `scale`; each level carries its declared bound and a project JSON. Deterministic; the bound covers timeline playback (not state-machine poses). Slow in unoptimised builds (use release). |
| `fit_to_size(&self, FitFormat, max_bytes)` | `Result<SizeFit, ApiError>` | smallest export within the budget by lowering quality knobs step by step; if impossible, the smallest result with `met == false` and a `reason` |
| `retarget_bvh(&mut self, bvh_text, &MocapSettings)` | `Result<MocapReport, ApiError>` | BVH retarget as **one undoable edit**; bones matched by name and topology; locked channels skipped and reported |
| `fuzz(&self, samples, seed)` | `Result<FuzzSummary, ApiError>` | pose fuzzing; the animation is only read; full report as JSON |

`Interpolation` of the segment that starts at a key: `Linear`, `Step` (hold), `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` (Catmull-Rom through neighbouring keys). The richer curves of the project format (cubic Bezier, bounce, back, spring) are authored through the CLI and the studio, not through `set_key`.

## Reference: `Player`

A running state machine (interactive playback). Created by `Animation::player`; works on a copy of the project; deterministic. State machines and data bindings are plain project JSON (`machines`, `bindings`), summarised in the [appendix](#appendix-machine-json-and-binding-json).

| Method | Returns | Notes |
|---|---|---|
| `set_bool(name, bool)`, `set_number(name, f64)`, `fire(name)` | `Result<(), ApiError>` | set an input by name; the wrong kind of input, or an unknown name, is `InvalidArgument`. A trigger stays set until a transition that tests it fires (it is consumed) or until the end of the next `advance`; `hold: true` keeps it until consumed. |
| `set_value(path, f64)` | `Result<(), ApiError>` | publish an external value for the data bindings (`"player.health"`). A scroll region whose `input` is this path takes the value as its offset in pixels along its main axis (the scroll is then at rest). |
| `advance(dt)` | `Result<(), ApiError>` | advance machine and scroll regions by `dt` seconds (`0..=10`) |
| `state(layer)` | `Option<String>` | current state of a machine layer; `None` for an unknown layer |
| `events_json()` | `String` | pending events `[{"time":..,"layer":..,"kind":"entered"|"exited","state":..}]`, not consumed |
| `take_events_json()`, `clear_events()` | `String`, `()` | consume / discard the pending events |
| `render_rgba(&RenderSettings)`, `render_svg(show_bones)`, `bone_world()` | as on `Animation` | the current pose; the picture has the canvas size set by `set_canvas_size` times the scale |
| `set_canvas_size(w, h)`, `canvas_size()` | `Result<(), ApiError>`, `(f32, f32)` | (1.2) responsive layout: containers re-solve, scroll limits follow |
| `scroll_ids()` | `Vec<String>` | ids of the project's scroll regions |
| `scroll_drag(id, x, y)` | `Result<(), ApiError>` | raw pointer offset in pixels (content moved up / left by that much); past the ends it is clamped or rubber-banded |
| `scroll_release(id, vx, vy)` | `Result<(), ApiError>` | release velocity in px/s; `advance` then flings, snaps and springs back |
| `scroll_offset(id)`, `scroll_max(id)`, `scroll_settled(id)` | `Option<[f32; 2]>`, `Option<[f32; 2]>`, `Option<bool>` | `None` for an unknown id |
| `set_screen_size(w_px, h_px, max_error_px)` | `Result<(), ApiError>` | (1.2) pick the coarsest of eight prebuilt levels of detail whose declared bound, at the scale that fits the canvas into that screen, is at most `max_error_px`. Levels are built once on first use (level 1 bounded by 0.125 stage units, doubling) and cached; with `max_error_px` 0 only lossless levels qualify. |
| `clear_screen_size()`, `lod_level()`, `lod_bound_px()` | `()`, `usize`, `f32` | back to the original; level in use (0 = original); its declared bound in pixels |

Notes: the level-of-detail bound covers geometry and timeline keys as documented in `lod.rs` (`ASSUMPTIONS`, `LIMITS`); state-machine poses and bindings are not modelled by it, and pose channels that target a layer a level removed are dropped. The studio's scrub proxies use a separate mechanism with their own level grid. A scroll region's `input` goes through the player's value table, not through the data-binding runtime, so expressions, remapping and smoothing do not apply to it.

## Reference: free functions (tools, version control, CI)

| Function | Returns | Does |
|---|---|---|
| `trace_image_svg(&[u8], colors)` | `Result<String, ApiError>` | trace a bitmap into an SVG of filled regions |
| `vcs_log(path, limit)` | `Result<Vec<VcsCommit>, ApiError>` | (1.2, read-only) history newest first; `path` is the project file or any path inside its repository; an empty history gives an empty list |
| `ci_run(manifest)` | `Result<CiOutcome, ApiError>` | (1.2) animation CI in verify mode (nothing re-recorded); relative paths in the manifest resolve next to it |
| `vcs_init(project_file, author)` | `Result<(), ApiError>` | (1.3) start a history next to the file on branch `main`, no commit yet; fails if one exists |
| `vcs_commit(path, message, author)` | `Result<String, ApiError>` | commit id; finishes a merge once no conflict is left; `Conflicts` while some remain; nothing to commit or no message is `InvalidArgument` |
| `vcs_branch(path, name, at)`, `vcs_tag(path, name, at)` | `Result<String, ApiError>` | create a branch / tag (default: current commit); not switched to; tags never move; commit id returned |
| `vcs_checkout(path, target, force)` | `Result<String, ApiError>` | switch the file to a branch (stays on it) or any revision (detached); `DirtyTree` unless `force`; `MergeInProgress` |
| `vcs_merge(path, rev, no_ff, message, author)` | `Result<VcsMergeOutcome, ApiError>` | `UpToDate`, `FastForward(id)`, `Merged(id)` or `Conflicted(Vec<VcsConflict>)` |
| `vcs_cherry_pick(path, rev, author)`, `vcs_revert(path, rev, author)` | `Result<VcsMergeOutcome, ApiError>` | replay / undo one commit's change |
| `vcs_conflicts(path)` | `Result<Vec<VcsConflict>, ApiError>` | the conflicts still pending (empty when none) |
| `vcs_resolve(path, conflict_path, &VcsChoice)` | `Result<usize, ApiError>` | settle one conflict with `Ours`, `Theirs` or `Value(json text)`; returns how many are left |
| `vcs_abort(path)` | `Result<(), ApiError>` | drop an unfinished merge / cherry-pick / revert and restore the last commit |
| `vcs_branches(path)` | `Result<Vec<VcsBranch>, ApiError>` | (1.4) the branches with their commit and whether checked out |
| `vcs_branch_delete(path, name)` | `Result<(), ApiError>` | (1.4) delete a branch; the checked-out branch and unknown names are refused; objects stay until `vcs_gc` |
| `vcs_status(path)` | `Result<VcsStatus, ApiError>` | (1.4) branch, HEAD, uncommitted semantic changes, unfinished merge, conflict paths |
| `vcs_diff(path, a, b)` | `Result<Vec<VcsChange>, ApiError>` | (1.4) semantic differences between two revisions; `None` is the working file |
| `vcs_show(path, rev)` | `Result<VcsShow, ApiError>` | (1.4) one commit and its changes against its first parent |
| `vcs_gc(path)` | `Result<VcsGc, ApiError>` | (1.4) delete unreachable objects; aborts without deleting if the history cannot be read |
| `vcs_fsck(path)` | `Result<Vec<String>, ApiError>` | (1.4) integrity problems, empty when healthy |
| `ci_init(project, manifest, write_goldens)` | `Result<CiInit, ApiError>` | write a default calibrated manifest (never overwrites an existing file; default name `osatura-ci.json` next to the project); optionally record the goldens now |
| `ci_run_mode(manifest, CiMode)` | `Result<CiOutcome, ApiError>` | `Verify`, `Update` (re-record all goldens of the render checks), `Bless` (accept only failing or missing ones) |
| `ci_bless(manifest)` | `Result<CiOutcome, ApiError>` | shorthand for `Bless` |

History is stored next to the project file (`.osatura-vcs/`); commit messages and authors are plain text. A conflict holds the identity pointer `path`, a `kind`, a `description`, and the `base` / `ours` / `theirs` values as JSON text (`None` = absent there). After a conflicted merge the project file contains the merged state with the conflicting items at the current branch's value, and a pending state is recorded until you resolve every conflict and `vcs_commit`, or `vcs_abort`. The binary assets a project references are not versioned, only their path strings. The semantic merge, its conflict kinds and its limits are described in [ROADMAP.md](../ROADMAP.md) section 9 (item I-01) and [LIMITATIONS.md](LIMITATIONS.md).

Not in the facade: mocap bone-mapping overrides, trace tuning beyond the colour count, fit options beyond format and budget, GPU rendering, the studio, collaboration, the agent server, the many authoring and baking commands (physics, foot lock, magnets, autorig, ...), branch deletion / listing, diff, show, status, gc, fsck, and CI options such as prune, `--only` and artifacts. Use the CLI for those ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

## Reference: types

| Type | Fields / variants |
|---|---|
| `AnimationInfo` | `name: String`, `width: f32`, `height: f32` (stage units), `fps: f32`, `frames: u32` |
| `RenderSettings` | `scale: f32` (0.05..=4), `show_bones: bool`, `transparent: bool`; `new(scale)`, `with_bones(bool)`, `with_transparent(bool)`; `Default` = scale 1, both off |
| `Image` | `width: u32`, `height: u32`, `pixels: Vec<u8>` (RGBA8, straight alpha) |
| `ExportFormat` | `Gif`, `Glb`, `Lottie`, `Html`, `Apng` (1.4) |
| `PhysicsBake` (1.4) | `added: usize`, `skipped: Vec<String>` |
| `VcsBranch` (1.4) | `name`, `commit`, `current: bool` |
| `VcsChange` (1.4) | `kind` (`"Added"`, `"Removed"`, `"Changed"`, `"Moved"`, `"Retimed"`), `path`, `description` |
| `VcsStatus` (1.4) | `branch`, `head`, `changes: Vec<VcsChange>`, `merge`, `conflicts: Vec<String>` |
| `VcsShow` (1.4) | `commit: VcsCommit`, `changes: Vec<VcsChange>` |
| `VcsGc` (1.4) | `kept`, `removed` |
| `Interpolation` | `Linear`, `Step`, `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` |
| `LodLevel` | `level`, `declared_px: f64`, `bound_stage: f64`, `path_anchors`, `keys`, `layers`, `json: String`; `bound_at(screen_scale) -> f64` |
| `FitFormat` | `Lottie`, `Html`, `Gif`, `Apng`, `Project` |
| `SizeFit` | `met: bool`, `size: usize`, `bytes: Vec<u8>`, `ssim_min: f64`, `reason: Option<String>` |
| `MocapSettings` | `in_place`, `mirror`, `smooth` (moving-average radius in frames); `new()`, `with_in_place`, `with_mirror`, `with_smooth`; `Default` |
| `MocapReport` | `matched: Vec<(String, String)>` (BVH joint, rig bone), `unmatched_joints`, `unmatched_bones`, `frames`, `bones_used`, `blocked` (locked `target.prop`), `notes` |
| `FuzzSummary` | `samples`, `failures`, `failure_rate: f32`, `max_severity: f32`, `json: String` |
| `VcsCommit` | `id`, `author`, `time: u64` (Unix seconds), `message` |
| `VcsConflict` | `path`, `kind`, `description`, `base`, `ours`, `theirs: Option<String>` |
| `VcsMergeOutcome` | `UpToDate`, `FastForward(String)`, `Merged(String)`, `Conflicted(Vec<VcsConflict>)` |
| `VcsChoice` | `Ours`, `Theirs`, `Value(String)` |
| `CiMode` | `Verify`, `Update`, `Bless` |
| `CiOutcome` | `total`, `passed`, `failed`, `errors`, `exit_code: i32` (0 good, 1 failed, 2 not evaluable), `json: String`, `updated`, `blessed` |
| `CiInit` | `manifest: String`, `checks: Vec<(String, String)>` (kind, name), `recorded: Option<CiOutcome>` |
| Constants | `API_VERSION: &str`, `API_VERSION_MAJOR: u32`, `MAX_RENDER_SIDE: u32`, `MAX_JSON_BYTES: usize` |

## Examples

All of these are complete and were run. They assume `use osatura_core::prelude::*;`.

### Load, evaluate, render, export

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Load a project, query it, render one frame and export the whole animation.
pub fn quick(project_path: &str) -> Result<(), Box<dyn Error>> {
    let json = std::fs::read_to_string(project_path)?;
    let mut anim = Animation::from_json(&json)?;           // parse + validate; typed error otherwise

    // Facts about the project.
    let info = anim.info();
    println!("{}: {}x{} @ {} fps, {} frames", info.name, info.width, info.height, info.fps, info.frames);

    // Evaluate: world matrices of every bone at frame 10 (six floats per bone: a b c d tx ty).
    let names = anim.bone_names();
    let world = anim.bone_world(10.0)?;
    let m = world[0];
    // A bone-local point (x, y) lands at (a*x + c*y + tx, b*x + d*y + ty).
    let (x, y) = (m[0] * 10.0 + m[2] * 0.0 + m[4], m[1] * 10.0 + m[3] * 0.0 + m[5]);
    println!("tip of {} (10 units along its x axis) is at ({x:.1}, {y:.1})", names[0]);

    // Edit through the undoable editor, then render.
    anim.set_key(&names[0], "bone_angle", 10.0, 25.0, Interpolation::EaseInOut)?;
    let png = anim.render_png(10.0, &RenderSettings::new(1.0).with_bones(true))?;
    std::fs::write("frame10.png", &png)?;
    let svg = anim.render_svg(10.0, false)?;
    std::fs::write("frame10.svg", svg)?;

    // Raw pixels: straight-alpha RGBA8, row-major, top row first.
    let img = anim.render_rgba(10.0, &RenderSettings::new(0.5).with_transparent(true))?;
    assert_eq!(img.pixels.len(), img.width as usize * img.height as usize * 4);

    // Whole-animation exports.
    let glb = anim.export(ExportFormat::Glb, 1.0)?;
    let gif = anim.export(ExportFormat::Gif, 0.5)?;        // `scale` only matters for GIF
    std::fs::write("anim.glb", &glb)?;
    std::fs::write("anim.gif", &gif)?;
    println!("{} bones, {} bytes of glb, {} bytes of gif", names.len(), glb.len(), gif.len());
    Ok(())
}
```

### Editing, undo and error handling

```rust
use osatura_core::prelude::*;

/// Edits are validated, undoable and all-or-nothing.
pub fn edit_and_errors() {
    let mut anim = Animation::demo();                       // small built-in project
    let bone = anim.bone_names()[0].clone();
    let before = anim.to_json();

    anim.set_key(&bone, "bone_angle", 4.0, 30.0, Interpolation::Smooth).unwrap();
    assert!(anim.delete_key(&bone, "bone_angle", 4.0).unwrap());      // true: a key was removed
    assert!(!anim.delete_key(&bone, "bone_angle", 77.0).unwrap());    // false: nothing there
    assert!(anim.undo() && anim.undo());                    // each successful edit is one undo step
    assert_eq!(anim.to_json(), before);
    assert!(anim.redo());

    // Errors are values. Match on them with a wildcard arm: the enum is #[non_exhaustive].
    match anim.set_key(&bone, "no_such_prop", 0.0, 1.0, Interpolation::Linear) {
        Err(ApiError::InvalidArgument(why)) => println!("rejected argument: {why}"),
        Err(ApiError::EditRejected(problems)) => println!("edit refused: {problems:?}"),
        Err(other) => println!("other failure: {other}"),   // Display is implemented for every variant
        Ok(()) => unreachable!(),
    }
    // A rejected edit leaves the project exactly as it was.
    let snapshot = anim.to_json();
    assert!(anim.set_key("no_such_bone", "bone_angle", 1.0, 1.0, Interpolation::Linear).is_err());
    assert_eq!(anim.to_json(), snapshot);

    // Bad input never panics.
    assert!(matches!(Animation::from_json("{ nope"), Err(ApiError::InvalidJson(_))));
    assert!(anim.render_rgba(f32::NAN, &RenderSettings::default()).is_err());
    assert!(anim.render_rgba(0.0, &RenderSettings::new(50.0)).is_err()); // scale must be 0.05..=4
}
```

### Interactive playback with a state machine

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Run a state machine of a project (machine.json has machine "hero", layer "body", bool input "excited").
pub fn play(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let machine = anim.machine_names()[0].clone();         // e.g. "hero"
    let mut player = anim.player(&machine)?;               // works on a copy of the project

    player.set_bool("excited", true)?;                     // also set_number, fire (trigger), set_value (binding path)
    for _ in 0..30 {
        player.advance(1.0 / 30.0)?;                       // deterministic; dt in seconds, 0..=10
    }
    println!("body is in state {:?}", player.state("body"));  // Some("wave")
    println!("{}", player.take_events_json());             // [{"time":..,"layer":"body","kind":"entered","state":..}, ..]

    let frame = player.render_rgba(&RenderSettings::new(0.5))?;
    println!("pose rendered at {}x{}", frame.width, frame.height);
    let svg = player.render_svg(false)?;
    assert!(svg.starts_with("<svg"));

    // Same inputs, same advance calls, same pose: a second player is bit-for-bit identical.
    let mut again = anim.player(&machine)?;
    again.set_bool("excited", true)?;
    for _ in 0..30 {
        again.advance(1.0 / 30.0)?;
    }
    assert_eq!(again.render_rgba(&RenderSettings::new(0.5))?, frame);

    // Wrong calls are typed errors, not panics.
    assert!(matches!(player.set_number("excited", 1.0), Err(ApiError::InvalidArgument(_))));
    assert!(matches!(anim.player("no such machine"), Err(ApiError::InvalidArgument(_))));
    Ok(())
}
```

### Layout, scrolling and level of detail

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Layout, scrolling and level of detail on a Player (scroll.json has one scroll region "list").
pub fn layout_and_lod(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let mut player = anim.player(&anim.machine_names()[0])?;

    player.set_canvas_size(100.0, 120.0)?;                 // containers re-solve; renders get this size
    for id in player.scroll_ids() {
        let max = player.scroll_max(&id).unwrap();         // about [0, 80] here: 200 units of content in a 120 unit view
        player.scroll_drag(&id, 0.0, 50.0)?;               // raw pointer offset in px (clamped / rubber-banded)
        assert_eq!(player.scroll_offset(&id), Some([0.0, 50.0]));
        player.scroll_release(&id, 0.0, 900.0)?;           // fling at 900 px/s
        while player.scroll_settled(&id) == Some(false) {
            player.advance(1.0 / 60.0)?;                   // steps the machine and every scroll region
        }
        println!("{id}: max {max:?}, rests at {:?}", player.scroll_offset(&id).unwrap());
    }

    // Pick the coarsest level of detail whose declared error stays within 0.5 px at this screen size.
    player.set_screen_size(100.0, 120.0, 0.5)?;
    println!("level {} (declared bound {} px)", player.lod_level(), player.lod_bound_px());
    player.clear_screen_size();                            // back to the original
    assert_eq!(player.lod_level(), 0);

    // A scroll region whose `input` names a path follows set_value(path, px) instead of a drag.
    player.set_value("ui.scroll", 200.0)?;
    Ok(())
}
```

### Rendering on several threads

```rust
use osatura_core::prelude::*;

/// `Animation` and `Player` are `Send + Sync`: read-only calls (`&self`) may run on several threads.
pub fn render_in_parallel() -> Result<(), ApiError> {
    let anim = Animation::demo();
    let frames = anim.frame_count();
    let settings = RenderSettings::new(0.25);
    let (a, s) = (&anim, &settings);                        // shared references are Copy: move them into the threads
    let images: Vec<Result<Image, ApiError>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..frames).map(|f| scope.spawn(move || a.render_rgba(f as f32, s))).collect();
        jobs.into_iter().map(|j| j.join().expect("render thread")).collect()
    });
    // Mutating calls (`&mut self`: set_key, undo, retarget_bvh ...) need exclusive access.
    for (f, img) in images.into_iter().enumerate() {
        assert_eq!(img?, anim.render_rgba(f as f32, &settings)?);   // threads do not change the result
    }
    Ok(())
}
```

### Version control and animation CI

```rust
use osatura_core::prelude::*;

/// Version control of a project file and animation CI, from Rust.
pub fn history(dir: &std::path::Path) -> Result<(), ApiError> {
    let file = dir.join("hero.json");
    std::fs::write(&file, Animation::demo().to_json()).unwrap();
    let path = file.to_str().unwrap();

    // A tiny helper: re-key the first bone at frame 5 and save the file.
    let edit = |angle: f32| -> Result<(), ApiError> {
        let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
        let bone = a.bone_names()[0].clone();
        a.set_key(&bone, "bone_angle", 5.0, angle, Interpolation::Linear)?;
        std::fs::write(path, a.to_json()).unwrap();
        Ok(())
    };

    vcs_init(path, Some("ana"))?;                           // branch `main`, no commit yet
    vcs_commit(path, Some("base"), None)?;
    vcs_branch(path, "other", None)?;                       // created, not switched to
    edit(20.0)?;
    vcs_commit(path, Some("main edit"), None)?;
    vcs_checkout(path, "other", false)?;                    // DirtyTree if the file has uncommitted changes
    edit(40.0)?;                                            // the same key, another value
    vcs_commit(path, Some("other edit"), None)?;
    vcs_checkout(path, "main", false)?;

    match vcs_merge(path, "other", false, None, None)? {
        VcsMergeOutcome::Conflicted(conflicts) => {         // a normal result, not an error
            for c in &conflicts {
                println!("{} ({}): {}", c.path, c.kind, c.description);
                vcs_resolve(path, &c.path, &VcsChoice::Theirs)?;
            }
            vcs_commit(path, None, None)?;                  // the merge commit
        }
        other => println!("merged without conflicts: {other:?}"),
    }
    for c in vcs_log(path, 10)? {
        println!("{} {} {}", &c.id[..8], c.author, c.message);
    }

    // Animation CI: write a calibrated manifest, record goldens, then verify.
    let init = ci_init(path, None, true)?;                  // never overwrites an existing manifest
    let run = ci_run(&init.manifest)?;                      // verify mode: nothing is re-recorded
    println!("{} checks, {} passed, exit code {}", run.total, run.passed, run.exit_code);
    if run.exit_code == 1 {
        // A failing check is a result, not an Err. After reviewing the report, accept the new look:
        ci_bless(&init.manifest)?;
    }
    Ok(())
}
```

### Mocap, fuzzing, tracing, level of detail and size-targeted export

```rust
use osatura_core::prelude::*;

const RIG: &str = r#"{"format":1,"name":"rig","width":400.0,"height":400.0,"fps":30.0,"frames":10,"background":null,
 "camera":{"position":null,"zoom":1.0,"rotation":0.0},
 "bones":[{"name":"hips","parent":null,"length":20.0,"rest_offset":[200.0,250.0],"rest_angle":-90.0},
          {"name":"spine","parent":"hips","length":40.0,"rest_offset":[20.0,0.0],"rest_angle":0.0},
          {"name":"head","parent":"spine","length":30.0,"rest_offset":[40.0,0.0],"rest_angle":0.0}],
 "ik":[],"constraints":[],"springs":[],"layers":[],"channels":[]}"#;

fn mini_bvh() -> String {
    let mut s = String::from("HIERARCHY\nROOT Hips\n{\n OFFSET 0 100 0\n CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation\n");
    s.push_str(" JOINT Spine\n {\n  OFFSET 0 12 0\n  CHANNELS 3 Zrotation Xrotation Yrotation\n  JOINT Head\n  {\n   OFFSET 0 12 0\n   CHANNELS 3 Zrotation Xrotation Yrotation\n   End Site\n   {\n    OFFSET 0 8 0\n   }\n  }\n }\n}\nMOTION\nFrames: 4\nFrame Time: 0.0333333\n");
    for f in 0..4 {
        s.push_str(&format!("0 100 0 0 0 0 {} 0 0 0 0 0\n", f * 10));
    }
    s
}

/// Motion capture, pose fuzzing, tracing, level of detail and size-targeted export.
pub fn tools() -> Result<(), ApiError> {
    // BVH retargeting: one undoable edit; bones are matched by name and topology.
    let mut rig = Animation::from_json(RIG)?;
    let report = rig.retarget_bvh(&mini_bvh(), &MocapSettings::new().with_smooth(1))?;
    println!("matched {:?}; {} frames on {} bones", report.matched, report.frames, report.bones_used);
    assert!(rig.undo());                                    // the whole retarget is one step

    // Pose fuzzing: where does the rig break?  (deterministic for a given seed)
    let demo = Animation::demo();
    let fuzz = demo.fuzz(50, 1)?;
    println!("{} of {} samples failed (max severity {:.2})", fuzz.failures, fuzz.samples, fuzz.max_severity);
    let _full_report: &str = &fuzz.json;                    // schema "osatura-fuzz-report" v1

    // Bitmap tracing: PNG/JPEG bytes -> SVG, or straight into an animation.
    let png = demo.render_png(0.0, &RenderSettings::new(0.25))?;
    let svg = trace_image_svg(&png, 6)?;
    let traced = Animation::from_image(&png, 6, "traced")?;
    println!("traced SVG: {} bytes; project has {} layers", svg.len(), traced.layer_names().len());

    // Level of detail with a declared error bound, one project JSON per level (level 0 = original).
    for l in demo.lod_levels(0.5, 1.0, 3)? {
        println!("L{}: bound {:.3} px, {} anchors, {} keys", l.level, l.declared_px, l.path_anchors, l.keys);
        let _reduced = Animation::from_json(&l.json)?;
    }

    // Smallest export within a byte budget.
    let fit = demo.fit_to_size(FitFormat::Lottie, 250_000)?;
    println!("met={} {} bytes, worst SSIM {:.3}", fit.met, fit.size, fit.ssim_min);
    Ok(())
}
```

### How the examples are checked

The code above is the code that ran: this file is generated from the sample sources, which were compiled with `cargo` against `osatura-core` and executed (the project files used are the output of `osatura demo`, `take-record`, `clip add` and `machine add`, plus a project with a container and a scroll region). The doc-tests in `api/` (`cargo test --doc -p osatura-core`: 19 doc-tests, 18 on facade functions in `api/` and the crate-level example in `lib.rs`; about 170 s in a debug build because each one compiles a program) cover the version-control and CI wrappers in the repository itself.

## Other languages

The facade is the single source of truth for the other bindings. Coverage:

| Capability | Rust facade | C ABI (`osatura.h`) | WebAssembly / JS |
|---|---|---|---|
| Load, validate, info, bone names / world matrices | yes | `osatura_open`, `_bone_*`, `_frame_count`, ... | `arm_open`, `load()` (no bone matrices) |
| Render RGBA | yes | `osatura_render_rgba` | `arm_render_rgba`, `rgba()` |
| Render SVG / PNG / export formats | yes | no | SVG yes (`svg()`), PNG / export no |
| Edit keys, undo / redo | yes | no | no |
| `Player`: inputs, advance, state, events, render | yes | `osatura_player_*` | `arm_player_*`, `startPlayer()`, ... |
| Layout, scroll, level of detail | yes | `osatura_player_set_canvas`, `_scroll_*`, `_set_screen`, `_lod_level` | `arm_player_set_canvas / layout / scroll_* / set_screen` |
| Tools, version control, CI | yes | no | no |

The C ABI version stays 1 because every addition so far was additive. The wasm binary is built and tested under Node in CI but has not been run in a browser; see [WASM.md](WASM.md).

## Appendix: machine JSON and binding JSON

Short summaries of the two project sections the `Player` runs (full format: [PROJECT_FORMAT.md](PROJECT_FORMAT.md)).

### Machine JSON (summary)

```json
{"name":"hero",
 "inputs":[{"name":"moving","kind":"bool"},{"name":"speed","kind":"number"},{"name":"jump","kind":"trigger"}],
 "layers":[{"name":"body","mask":["arm_*"],"entry":"idle",
   "states":[{"name":"idle","source":{"kind":"clip","clip":"idle"},"blend_in":0.2},
             {"name":"loco","source":{"kind":"blend","input":"speed","children":[
                {"threshold":0,"source":{"kind":"clip","clip":"walk"}},{"threshold":1,"source":{"kind":"clip","clip":"run"}}]}},
             {"name":"jump","source":{"kind":"clip","clip":"jump"},"mode":"once"}],
   "transitions":[{"from":"idle","to":"loco","when":[{"input":"moving"}],"duration":0.25,"ease":{"type":"ease_in_out"}},
                  {"from":"*","to":"jump","when":[{"input":"jump"}],"duration":0.1},
                  {"from":"jump","to":"idle","exit_time":1.0}]}]}
```

Conditions are `{"input","op":"eq|ne|lt|le|gt|ge","value"}` (defaults `eq`, `1`; all must hold). `from: "*"` matches any state except the target. The first matching transition in declaration order wins. `exit_time` is the played fraction (0..1) of the state. Sources: `clip` (a `Project.clips` entry), `segment` (`start`, `end` frames of the project's keyed replace-mode channels), `region` (a marker with an end), `blend` (1-D, two or more leaf sources at increasing thresholds). Unknown keys in a transition or condition are an error. CLI: `machine add|list|remove|validate`, `machine-sim <project> <script.json> <out_dir|out.json>` (script: `{"machine":"hero","duration":2,"events":[{"t":0.5,"set":{"moving":true},"trigger":["jump"],"value":{"player.health":40}}]}`).

### Binding JSON (summary)

```json
{"name":"hp","path":"player.health","expr":"value / 100","remap":{"in_min":0,"in_max":1,"out_min":-30,"out_max":30},
 "smooth":0.15,"target":{"kind":"prop","target":"arm_up","prop":"bone_angle"}}
```

Pipeline: value, then `expr` (variable `value`; the engine's expression functions, source at most 4 KB and nesting at most 64), then `remap` (clamped by default; `curve` eases), then smoothing (time constant in seconds), then the target: `{"kind":"prop",...}` (any channel property, including `layer_opacity` and `layer_switch`) or `{"kind":"input","machine","input"}` (a trigger input fires on the rising edge). CLI: `bind add|list|remove`, `bind-eval`.

### Layout and per-layer normal maps

Layout containers and scroll regions are project fields `containers` / `scrolls` (omitted when empty): CLI `container add|list|remove|solve`, `scroll add|list|remove|eval`, `layout-preview <project> <out.svg> --size WxH`, and `--size WxH` on `render`, `frames` and `export` (containers re-solve for that canvas size; the file is unchanged). `Layer.normal_map` (`data` base64 PNG or `path`, `fit` stretch / tile / exact, `strength`, `green_down`) feeds the `relight` post effect for the pixels that layer, or a child without its own map, covers (CLI `layer-normal`, `fx ... --layer-normals`); it is not part of the facade.
