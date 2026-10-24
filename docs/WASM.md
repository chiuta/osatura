# OSATURA for WebAssembly

*ABI version 1 (`arm_abi_version()`). Crate `osatura-wasm`, JavaScript wrapper `osatura-wasm/js/osatura.js`, demo page `osatura-wasm/index.html`, build script `osatura-wasm/build-wasm.sh`. Audience: web developers who want to play OSATURA projects in a browser or in node, and the first person who runs the module in a browser. Documentation CC-BY-4.0; code samples MPL-2.0.*

> ## STATUS: BUILT AND TESTED IN CI, RUN ONLY IN HEADLESS CHROMIUM
>
> **As of 24 Oct 2026 the module is built on GitHub Actions** (`wasm32-unknown-unknown`, debug build in `ci.yml`, release build in `release.yml`) and **the wrapper's test passes against the real module under Node 20** (`node osatura-wasm/js/test.mjs demo.json`, including its real-module part). The release build is attached to the draft pre-release `v0.1.0-alpha.3` as `osatura-web.tar.gz` (about 0.9 MB compressed, with `osatura.js` and the demo page). **It was loaded once in headless Chromium 141 on Linux without a GPU** (results at the end of this page, produced by `osatura-wasm/js/browser-check.js`); Firefox, Safari, phones and real GPUs were not tried, so statements about them on this page are not observed behaviour, and the development environment itself still cannot install the target (the download of its standard library is blocked there).
>
> What was tested, and how far that goes:
>
> | Layer | Test | Proves | Does not prove |
> |---|---|---|---|
> | The Rust ABI functions (`arm_*`) | `cargo test -p osatura-wasm`: 8 unit tests on the **host**, calling the exported functions directly | argument checking, error codes, the output-buffer protocol, state machines, layout / scroll / level-of-detail calls, refusal of oversized output before allocation | behaviour of the compiled module |
> | The compiled module | CI: `cargo build --target wasm32-unknown-unknown`, then `node osatura-wasm/js/test.mjs demo.json` with the real `.wasm` | that the dependencies compile for wasm32, that the module exports what the wrapper expects, and that loading, rendering and the state-machine calls work under Node's WebAssembly engine | browsers, release speed and memory, panics (a trap), the demo page |
> | The JavaScript wrapper | the same script against a **JavaScript mock** of the exports (three mock sections: render, state machine, layout) | the wrapper's call sequences, string passing, error translation, copying of results | - |
> | The demo page | loaded in headless Chromium **without** the `.wasm` file | the module import paths resolve and the page reports "wasm not found" instead of crashing | any rendering in a browser |
> | Dependencies | `cargo tree -p osatura-wasm`: no `rhai` / `smartstring` in the tree | the scripting engine is not part of the module | - |
>
> **Remaining risks:** (1) *Resolved in CI:* the compile of `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder`, `i_overlay` and their dependencies for `wasm32-unknown-unknown`. (2) `std::time::Instant` / `SystemTime`, `process::id` and `thread::spawn` **panic** on `wasm32-unknown-unknown`: since wave 7 the core no longer calls them directly. Every use goes through `osatura_core::clock` (a stamp type, `unix_seconds`, `process_id`; the LOD proxy builder falls back to the calling thread when a thread cannot be spawned), which on this target reads a host-supplied clock (the export `arm_set_clock(mono_ms, unix_ms)`, fed by the JavaScript wrapper with `performance.now()` and `Date.now()` before `load`, `svg` and `rgba`; a host that never calls it gets durations of 0, so time budgets never expire) and never panics. A test (`osatura-core/tests/wasm_safety.rs`) fails when a new direct use appears outside `clock.rs`; the Node test in CI exercises the paths it reaches, not all of them. Rhai (which would also need its `no_time` feature) is not in the build. (3) File-system and process bound modules compile but fail at run time: `audio.rs` (file open), `soundtrack.rs` (ffprobe), `snapshots.rs`, `open_formats.rs` (export directories), `agent.rs` (save), and the loading of relative-path images and normal maps in `engine.rs`, `meshtrace.rs` and `postfx.rs`; they return errors rather than working in a browser, and no cargo feature gates them. (4) Floating-point functions (`sin`, `cos`, `powf`, ...) come from a different math library on wasm32, so rendered pixels can differ slightly from native ones (step 6 of the checklist measures it). (5) On `wasm32-unknown-unknown` the default is `panic = "abort"`, `catch_unwind` cannot intercept anything, and a Rust panic traps the whole instance (`WebAssembly.RuntimeError: unreachable`). See the [verification checklist](#verification-checklist-for-the-first-person-who-can-run-it-in-a-browser) at the end.

Contents: [What it is](#what-it-is) - [Building](#building) - [Memory model and the ABI](#memory-model-and-the-abi) - [Export reference](#export-reference) - [The JavaScript wrapper](#the-javascript-wrapper) - [The demo page](#the-demo-page) - [Differences from the C ABI](#differences-from-the-c-abi) - [Known rough edges](#known-rough-edges) - [Verification checklist](#verification-checklist-for-the-first-person-who-can-run-it-in-a-browser)

## What it is

`osatura-wasm` is a `cdylib` of **plain `extern "C"` functions** (no `wasm-bindgen`, no generated glue) over the stable Rust facade ([API.md](API.md)). Compiled for `wasm32-unknown-unknown` it gives a module that can open a project, render frames as SVG text or RGBA pixels and run state machines with layout, scrolling and level of detail. It cannot edit, export, version-control or run tools; it has no file system, network, audio or threads. `osatura.js` is a dependency-free ES module (browsers and node >= 18) that hides the pointer handling.

Licence: MPL-2.0 (crate and wrapper). The module depends on `osatura-core` with `default-features = false`: no scripting engine and no MPL-licensed third-party crate.

## Building

You do not have to build it: the draft release carries `osatura-web.tar.gz` (module, wrapper, demo page). To build it yourself you need a machine where the target can be installed (and, optionally, `wasm-opt` from Binaryen for size). CI builds it with `cargo build --release --target wasm32-unknown-unknown -p osatura-wasm --locked`.

```sh
rustup target add wasm32-unknown-unknown
./osatura-wasm/build-wasm.sh
```

The script (`osatura-wasm/build-wasm.sh`, a POSIX shell script) does exactly this, from the repository root:

1. `cargo build --target wasm32-unknown-unknown --release -p osatura-wasm`
2. copies `target/wasm32-unknown-unknown/release/osatura_wasm.wasm` to `osatura-wasm/osatura_wasm.wasm`;
3. if `wasm-opt` is on the `PATH`, rewrites that file with `wasm-opt -Oz`;
4. prints the size of the result and a hint to run the node test or serve the demo.

**The script itself is not what CI runs** (CI calls `cargo build` directly and copies the module). Its closing hint says `node osatura-wasm/js/test.mjs`; with a real `.wasm` present the test additionally needs a project path as its first argument (`node osatura-wasm/js/test.mjs project.json`), otherwise it stops with "pass a project JSON path as argv[2]". The release module in the draft is about 0.9 MB as a compressed archive; the raw and `wasm-opt -Oz` sizes are not recorded here.

Then serve the folder over HTTP (browsers do not load modules or `.wasm` from `file://`) and open the demo page:

```sh
cd osatura-wasm && python3 -m http.server 8000      # then open http://localhost:8000/index.html
```

If you use `WebAssembly.instantiateStreaming` (the wrapper does so when you pass a `Response`), the server must send `Content-Type: application/wasm`; the wrapper's URL path uses `fetch(...).arrayBuffer()` and has no such requirement. If your page sets a Content-Security-Policy, it must allow WebAssembly compilation (`'wasm-unsafe-eval'` in `script-src`).

An alternative target, `wasm32-wasip1`, builds with `-C panic=unwind` so that `catch_unwind` works (the source mentions it); it has not been tried and needs a WASI runtime or shim.

## Memory model and the ABI

The module owns one **linear memory** (exported as `memory`). The host never shares pointers with the engine except through these rules:

* **Input.** To pass bytes (the project JSON, or a name), the host calls `arm_alloc(len)`, which returns a pointer to `len` **zeroed** bytes (null when `len` is 0 or above 256 MiB). The host writes into `new Uint8Array(memory.buffer, ptr, len)`, calls the function with `(ptr, len)`, then releases the block with `arm_free(ptr, len)` using the **same** pointer and length. The engine copies what it needs; it keeps no pointer into your block.
* **Output.** Results are produced into a buffer **inside the handle**: after a call that returns a length `n >= 0`, `arm_out_ptr(h)` / `arm_out_len(h)` give the bytes (`n` equals the length), and for pictures `arm_out_width(h)` / `arm_out_height(h)` give the size (0 after SVG or text results). The pointer is valid **until the next call that produces output, or until `arm_close`**. The host must copy the bytes out immediately (the wrapper does: `.slice()`). Never keep a `Uint8Array` view across calls: besides being overwritten, any call may grow the module's memory, which detaches `memory.buffer` and invalidates every view of it. Re-read `memory.buffer` after every call.
* **Errors.** A function that fails returns `-1` (or a null handle) and stores the message; `arm_error_ptr()` / `arm_error_len()` give its UTF-8 bytes (not NUL-terminated). The slot is global to the module (it is single-threaded), is overwritten by the next failure and is **not cleared by successes**, so read it only right after a failure.
* **Strings in.** `(ptr, len)` UTF-8, `len` between 1 and 4096 (an empty or longer string is an error).
* **Handles.** `arm_open` returns an opaque pointer to a handle (null on error). One handle holds the animation, the optional player and the output buffer. Close it with `arm_close` exactly once; `null` is allowed.
* **Limits.** Project JSON 1 byte to 256 MiB; `arm_render_rgba` refuses before allocating if the picture would exceed 256 MiB (`output would be WxH pixels (N bytes), more than the limit of 268435456 bytes: lower the scale`); scale within 0.05 to 4; frames outside the timeline are clamped; lengths returned as `i32`. Rendering needs memory for the picture plus working buffers; the 32-bit address space (4 GiB, less in some browsers) is the real ceiling.
* **No threads, no panics promised.** The module is single-threaded. Panics are caught only on targets that unwind; on `wasm32-unknown-unknown` a panic traps the instance and the host must discard it and create a new one. The engine is written not to panic on user input and checks sizes before allocating, but this layer cannot promise to survive a bug.

## Export reference

37 exports plus `memory`. `ptr` and `len` are `i32` in the wasm signature (`usize` in Rust). `h` is a handle. Return `-1` = error unless stated.

| Export | Signature (Rust view) | Result |
|---|---|---|
| `arm_abi_version` | `() -> u32` | 1 |
| `arm_set_clock` | `(mono_ms: f64, unix_ms: f64)` | optional: gives the module the time (`performance.now()`, `Date.now()`); invalid values are ignored. Added after ABI version 1 was fixed; the wrapper calls it when present |
| `arm_alloc` | `(len) -> *mut u8` | zeroed block, null if `len` is 0 or > 256 MiB |
| `arm_free` | `(ptr, len)` | frees a block from `arm_alloc` (null / zero length ignored) |
| `arm_open` | `(json_ptr, json_len) -> *mut Handle` | handle or null; validates the project |
| `arm_close` | `(h)` | frees the handle |
| `arm_frame_count`, `arm_fps`, `arm_width`, `arm_height` | `(h) -> u32 / f32 / f32 / f32` | `width` / `height` are the stage size in stage units (not rounded); 0 and an error for a null handle |
| `arm_render_svg` | `(h, frame: f32, show_bones: i32) -> i32` | SVG text in the out buffer; its byte length |
| `arm_render_rgba` | `(h, frame: f32, scale: f32) -> i32` | RGBA8 straight alpha in the out buffer; byte length (`w*h*4`); size via `arm_out_width/height` |
| `arm_out_ptr`, `arm_out_len` | `(h) -> *const u8 / usize` | the out buffer (null / 0 for a null handle) |
| `arm_out_width`, `arm_out_height` | `(h) -> u32` | picture size of the last RGBA render (0 after text) |
| `arm_machine_count` | `(h) -> u32` | machines in the project |
| `arm_machine_name` | `(h, index) -> i32` | name into the out buffer; its length; -1 for a bad index |
| `arm_player_start` | `(h, name_ptr, name_len) -> i32` | start (or restart) the machine; 0 |
| `arm_player_stop` | `(h)` | drop the player |
| `arm_player_set_bool` | `(h, ptr, len, value: i32) -> i32` | 0 |
| `arm_player_set_number` | `(h, ptr, len, value: f64) -> i32` | 0 |
| `arm_player_fire` | `(h, ptr, len) -> i32` | 0 |
| `arm_player_set_value` | `(h, ptr, len, value: f64) -> i32` | 0; external value for bindings / scroll input |
| `arm_player_advance` | `(h, dt: f64) -> i32` | 0; `dt` in `0..=10` seconds |
| `arm_player_state` | `(h, layer_ptr, layer_len) -> i32` | state name into the out buffer; length; -1 unknown layer |
| `arm_player_events` | `(h) -> i32` | pending events as JSON into the out buffer, **consumed**; length |
| `arm_player_render_svg` | `(h, show_bones: i32) -> i32` | SVG of the current pose into the out buffer |
| `arm_player_render_rgba` | `(h, scale: f32) -> i32` | RGBA of the current pose; size via `arm_out_width/height` |
| `arm_player_set_canvas` | `(h, w: f32, h: f32) -> i32` | 0 |
| `arm_player_layout` | `(h) -> i32` | JSON `{"canvas":[w,h],"scrolls":[ids],"lod":level}` into the out buffer |
| `arm_player_scroll_drag` | `(h, id_ptr, id_len, x: f32, y: f32) -> i32` | 0 |
| `arm_player_scroll_release` | `(h, id_ptr, id_len, vx: f32, vy: f32) -> i32` | 0 |
| `arm_player_scroll_info` | `(h, id_ptr, id_len) -> i32` | JSON `{"offset":[x,y],"max":[x,y],"settled":bool}` into the out buffer |
| `arm_player_set_screen` | `(h, w_px: f32, h_px: f32, max_error_px: f32) -> i32` | 0; picks the level of detail |
| `arm_error_ptr`, `arm_error_len` | `() -> *const u8 / usize` | last error text |

All `arm_player_*` calls except `arm_player_start` need a started player ("no player started (call arm_player_start)" otherwise). The player calls are the facade's `Player` methods ([API.md](API.md)).

## The JavaScript wrapper

`osatura-wasm/js/osatura.js` exports `class Osatura` (also as the default export). Every method that changes something returns `this`, so calls chain. Errors are thrown as `Error` with the engine's message.

| Member | Description |
|---|---|
| `static async instantiate(source, imports = {})` | returns the raw exports object. `source`: an `ArrayBuffer` or typed array (bytes), a `Response` (uses `instantiateStreaming`; needs MIME `application/wasm`), or a URL string / `URL` (uses `fetch` + `arrayBuffer`). The module has no imports, so `imports` stays `{}`. |
| `static async create(source, project)` | `source` is either already-instantiated exports (an object with `arm_open`) or anything `instantiate` accepts; `project` is JSON text or a plain object (`JSON.stringify` is applied). Returns an `Osatura`. |
| `load(project)` | load or replace the project; throws on failure and then **keeps the previously loaded project usable**. Fills `frames`, `fps`, `width`, `height`. |
| `frames`, `fps`, `width`, `height` | facts from the last `load` (`width` / `height` in stage units, floats) |
| `svg(n, showBones = false)` | `string`: SVG document for frame `n` (fractional frames allowed, clamped) |
| `rgba(n, scale = 1)` | `{ width, height, data }`: `data` is a `Uint8ClampedArray` (straight-alpha RGBA, a copy you own) |
| `frame(n, { kind = 'image', scale = 1, bones = false } = {})` | `kind: 'svg'` gives `{ svg }`; otherwise `{ image: ImageData }` in a browser or `{ width, height, data }` where `ImageData` does not exist (node) |
| `machines()` | `string[]` machine names |
| `startPlayer(name)`, `stopPlayer()` | start (restart) / drop the player; inputs return to their defaults |
| `setBool(name, v)`, `setNumber(name, v)`, `fire(name)`, `setValue(path, v)` | inputs and external values; an empty name throws `empty string`, a name over 4096 bytes throws |
| `advance(dt)` | step the machine and scroll regions by `dt` seconds (`0..=10`) |
| `state(layer)` | `string` current state of a machine layer |
| `events()` | `{ time, layer, kind: 'entered' \| 'exited', state }[]`, **consumed** by the call |
| `setCanvas(w, h)` | responsive layout for a canvas size in stage units |
| `layout()` | `{ canvas: [w, h], scrolls: string[], lod: number }` |
| `scrollDrag(id, x, y)`, `scrollRelease(id, vx = 0, vy = 0)` | pointer drag / release velocity in px and px/s |
| `scrollInfo(id)` | `{ offset: [x, y], max: [x, y], settled: boolean }` |
| `setScreen(widthPx, heightPx, maxErrorPx = 0.5)` | choose the level of detail for a screen size |
| `playerSvg(showBones = false)`, `playerRgba(scale = 1)` | the player's current pose |
| `play(canvas, { scale = 1, loop = true, onFrame } = {})` | draw the timeline on a 2D canvas with `requestAnimationFrame` (falls back to `setTimeout` outside a browser); returns a stop function |
| `playMachine(canvas, { scale = 1, onTick } = {})` | advance the started machine by real elapsed time (at most 0.25 s per tick) and draw it; `onTick(dt)` runs before each advance, so set inputs there; returns a stop function |
| `stop()` | stop `play` / `playMachine` |
| `close()` | stop and free the handle; call when done |

The wrapper keeps `x.memory.buffer` lookups inside every call and copies results out with `.slice()`, so memory growth cannot leave it with a stale view. Each `Osatura` object owns one handle; create several objects (they may share one `exports`) to hold several projects. Do not share one object between concurrent async tasks that interleave calls and result reads: each call's result lives in the handle's out buffer until the next call.

### Example (call sequence)

This is the exact call sequence the node test exercises. **It was run against the JavaScript mock** (the stand-in in `test.mjs` with canned answers: machine `hero`, layer `body`, bool input `go`, scroll region `list`), not against a real module.

```js
import { Osatura } from './js/osatura.js';

// 1. Instantiate the module once (a URL in the browser; in node pass the bytes: readFileSync('osatura_wasm.wasm')).
const exports = await Osatura.instantiate('./osatura_wasm.wasm');

// 2. Load a project (JSON text or a plain object). Throws Error(<engine message>) when it is invalid.
const anim = await Osatura.create(exports, projectJsonText);
console.log(`${anim.width}x${anim.height} stage units, ${anim.frames} frames @ ${anim.fps} fps`);

// 3. Render. Frames may be fractional and are clamped to the timeline.
const svg = anim.svg(10, /* showBones */ false);        // string
const { width, height, data } = anim.rgba(10, 0.5);     // data: Uint8ClampedArray, straight-alpha RGBA, a copy you own

// 4. Interactive playback with a state machine.
console.log(anim.machines());                           // ['hero']
anim.startPlayer('hero').setBool('go', true).advance(0.1);
console.log(anim.state('body'), anim.events());         // 'b', [{time, layer, kind, state}, ...] (consumed)
const poseSvg = anim.playerSvg();
const pose = anim.playerRgba(0.5);

// 5. Layout and scrolling.
anim.setCanvas(100, 120);
anim.scrollDrag('list', 0, 50).scrollRelease('list', 0, 100);
console.log(anim.scrollInfo('list'));                   // { offset: [0, 50], max: [0, 80], settled: false }
anim.setScreen(100, 120, 4);                            // pick a level of detail within 4 px
console.log(anim.layout());                             // { canvas: [100, 120], scrolls: ['list'], lod: 2 }

// 6. Always release the handle (it frees the project inside the module's memory).
anim.close();
```

In a browser page:

```html
<canvas id="c"></canvas>
<script type="module">
  import { Osatura } from './js/osatura.js';
  const x = await Osatura.instantiate('./osatura_wasm.wasm');
  const text = await (await fetch('./project.json')).text();
  const anim = await Osatura.create(x, text);

  // Timeline playback: requestAnimationFrame loop, loops at the end, returns a stop function.
  const stop = anim.play(document.getElementById('c'), { scale: 1, loop: true });

  // Or run a state machine in real time; onTick(dt) runs before each advance, so set inputs there.
  // anim.startPlayer('hero');
  // const stopMachine = anim.playMachine(canvas, { onTick: () => anim.setBool('go', keyIsDown) });
</script>
```

## The demo page

`osatura-wasm/index.html` is a minimal page: it imports the wrapper, instantiates `./osatura_wasm.wasm`, and offers a file input (`.json` project) and a Stop button. Choosing a file creates an `Osatura`, shows `WIDTHxHEIGHT, N frames @ FPS fps` and starts `play(canvas)`; errors from the engine are shown in the text next to the buttons. If the module cannot be fetched or compiled it shows `wasm not found (not built yet): <reason>`. Tested only in that "not built" state, in headless Chromium: the page loads, the module import resolves, the message appears (with the server's 404 page as the "module" the reason reads `expected magic word 00 61 73 6d, found 3c 21 44 4f`, see below). Everything beyond that is unverified.

## Differences from the C ABI

| | C ABI ([C_ABI.md](C_ABI.md)) | WebAssembly |
|---|---|---|
| Strings in | NUL-terminated | `(ptr, len)` copied in with `arm_alloc` |
| Results out | caller-supplied buffers | the handle's out buffer (`arm_out_ptr/len`), valid until the next output call |
| Size query | `NULL` buffer returns the size | not needed: the call returns the length and the data is already in the out buffer |
| Error text | `osatura_last_error()` NUL string, per thread | `arm_error_ptr/len` bytes, global |
| Bone matrices | `osatura_bone_world` | not exported |
| SVG | not exported | `arm_render_svg`, `arm_player_render_svg` |
| Events | size-query pattern, kept if the buffer is too small | always consumed |
| Stage size | rounded integers | floats |

## Known rough edges

Found while reading the code; none of them has been seen failing:

* `Osatura.instantiate(url)` checks `response.ok` and fetch errors and reports the URL and HTTP status (mock-fetch tests in `osatura-wasm/js/test.mjs`; run under node, and in CI against the real module).
* `frame()` with `kind: 'svg'` returns `{ svg }`, not a string like `svg()`.
* The hint printed by `build-wasm.sh` now includes the `<project.json>` argument that `test.mjs` needs.
* `play()` draws by converting every frame to RGBA on the CPU and calling `putImageData`; there is no audio, no pausing, no seeking.
* The wrapper does not export TypeScript declarations.

## Verification checklist for the first person who can run it in a browser

Steps 1 and 5 were done by CI on 24 Oct 2026. Steps 3, 4, 6, 8, 9, 11 and 13 were done on the same day in **headless Chromium 141 (Linux, no GPU)** with `osatura-wasm/js/browser-check.js`; the numbers are in the table below. **Still open:** Firefox and Safari (steps 11 and 12 there), the CSP and content-type step 12, stack depth 10, and the player steps 7 and 14. Do these in order; stop at the first failure and record the exact message. Items marked (risk) correspond to the risks above. When you have results, update [LIMITATIONS.md](LIMITATIONS.md) (section 1), the R-090 line of ROADMAP.md section 9 and the status box of this page.

1. **Install the target and build.** `rustup target add wasm32-unknown-unknown && ./osatura-wasm/build-wasm.sh`. (risk) Expect to see whether `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder` and `i_overlay` compile. Record the compiler and `rustc --version`.
2. **No scripting engine.** `cargo tree --target wasm32-unknown-unknown -p osatura-wasm | grep -ci 'rhai\|smartstring'` must print 0.
3. **Look at the exports.** In node: `WebAssembly.Module.exports(new WebAssembly.Module(fs.readFileSync('osatura-wasm/osatura_wasm.wasm')))`. Expect `memory` and the 37 functions of the [export table](#export-reference), and **no imports** (`WebAssembly.Module.imports(...)` empty). If `memory` is missing or imports appear (for example `env.*` from a std feature), the wrapper must be adapted.
4. **Size.** Record the size before and after `wasm-opt -Oz`, and gzip / brotli sizes. Measured once in the Chromium run: see the results table at the end of this page.
5. **Run the node test with a real project.** `osatura demo demo.json && node osatura-wasm/js/test.mjs demo.json` must print `real wasm tests: ok` after the three mock sections.
6. **Compare with the native renderer.** For frames 0, 12 and 47 of the demo at scales 0.5 and 1, compare `Osatura.rgba` with `osatura render demo.json N out.png --scale S` decoded to RGBA. (risk) Expect small differences at anti-aliased edges (different math library); report the maximum and mean absolute difference and the fraction of pixels above 24/255. A large difference points at a real bug.
7. **Exercise the player.** Build the project of the C ABI example (demo + `take-record` + `clip add` + `machine add`, see [C_ABI.md](C_ABI.md)), drive `startPlayer`, `setBool`, `advance`, `state`, `events`, `playerRgba`; and the project with a container and a scroll region for `setCanvas`, `scrollDrag`, `scrollRelease`, `scrollInfo`, `setScreen` (the last builds level-of-detail levels: time it, it is slow unoptimised and has not been timed on wasm).
8. **Fail-safe paths.** Feed an invalid project, an empty string, a 5 MB random file, a project with a `source.file` image path and a project with a sound track. Every case must produce a thrown `Error` with a message, not a trap. (risk) A trap (`RuntimeError: unreachable`) on any of them is a bug to report; most likely candidates are `Instant` / `SystemTime` or file-system calls reached through an unexpected path.
9. **Large render.** `rgba(0, 4)` on the demo (2560 x 1920) and a stage of 12000 x 12000 at scale 1 (must be refused with the "more than the limit" message, not run out of memory). Watch memory growth and confirm the wrapper still works after the memory grew (views are re-read per call).
10. **Stack depth.** `wasm32-unknown-unknown` gives the main thread a small default stack (1 MiB in current toolchains). Open a project with deeply nested groups and a long expression chain (see `osatura-cli/tests/robust.rs` for hostile inputs) and confirm no stack overflow trap.
11. **Browser run.** Serve the folder, open `index.html` in Chromium and Firefox, choose a project, confirm playback, resize, close and reopen several projects (leak check: memory should not grow without bound; `close()` releases the handle).
12. **`instantiateStreaming` and CSP.** Serve with and without `Content-Type: application/wasm`, and with a CSP that omits `'wasm-unsafe-eval'`; document what happens.
13. **Performance.** Time `rgba(n, 1)` for the 640 x 480 demo (native release: about 14 ms per frame) and a heavy project; record frames per second of `play()` in the browser.
14. **Optional: panics.** Decide whether the trap-on-panic behaviour is acceptable for your use. If not, try the `wasm32-wasip1` + `-C panic=unwind` route mentioned in the source.

### Results of the Chromium run (24 Oct 2026)

| Step | Result |
|---|---|
| 4. Size | release module 2,802,534 bytes; 853,804 gzip; 615,267 brotli (not run through `wasm-opt`) |
| 6. Native comparison | demo frame 12, 640x480: **0 differing bytes** of 1,228,800 (mean and max absolute difference 0) |
| 8. Fail-safe | empty, junk, wrong-shape JSON, 5 MB random and "no frames" each throw an `Error` with a message; no trap |
| 9. Large render | scale 4 (2560x1920) works; a 12000x12000 stage is refused with the limit message and the module still works afterwards |
| 11. Leak | 60 open and close cycles: linear memory 54,591,488 bytes before and after |
| 13. Speed | `rgba` of the 640x480 demo: median 72.5 ms, p95 96.5 ms, max 125.4 ms (about 14 fps; native release about 14 ms) |

Re-run it with `node osatura-wasm/js/browser-check.js <unpacked osatura-web folder> <project.json>` (needs Playwright). The page raised no errors during the run.
