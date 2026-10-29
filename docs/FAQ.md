<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Frequently asked questions

*Short answers, each taken from something that was run or read in the source tree and the other documents. Documentation CC-BY-4.0. Where an answer depends on a limit, it says so and points to [LIMITATIONS.md](LIMITATIONS.md). Error strings in the last section were captured from a real build; they are quoted exactly, so you can search this page for the text you see.*

Contents: [Installation and setup](#installation-and-setup) - [ffmpeg](#ffmpeg) - [Performance](#performance) - [Determinism](#determinism) - [Licensing and your output](#licensing-and-your-output) - [File safety and asset paths](#file-safety-and-asset-paths) - [Why the output must differ from the input](#why-the-output-must-differ-from-the-input) - [Rigging and animation](#rigging-and-animation) - [Studio](#studio) - [Face, speech and motion capture](#face-speech-and-motion-capture) - [Bugs, security and contributing](#bugs-security-and-contributing) - [Limitations](#limitations) - [Comparisons with other tools](#comparisons-with-other-tools) - [Troubleshooting table](#troubleshooting-table)

---

## Installation and setup

**1. How do I install OSATURA?**
Prebuilt archives (Windows x86-64, macOS arm64, Linux x86-64, and the WebAssembly module) are prepared as the draft pre-release `v0.1.0-alpha.3` on <https://github.com/chiuta/osatura/releases>; they become downloadable when the owner publishes the draft, and they are not code-signed (SmartScreen shows "Unknown publisher"; macOS asks for approval). There is no installer: unpack the archive and run `osatura`. Or build it from source with a recent stable Rust toolchain: `cargo build -p osatura-cli` gives `target/debug/osatura`; add `--release` for `target/release/osatura`. See [DEVELOPMENT.md](DEVELOPMENT.md).

**2. Which platforms work?**
Linux x86-64 is where the project was developed and exercised. On GitHub Actions the test suite and clippy also pass on macOS (arm64) and Windows (x86-64), and release binaries for those three are built there; but no person has used the Windows or macOS executables on a real desktop yet, so they are "tested by CI", not "supported" ([LIMITATIONS.md](LIMITATIONS.md), section 1).

**3. Is there a graphical installer or an app?**
There is no installer. The command line is the product. The studio is a local web page served by `osatura studio`; on Windows, double-clicking `osatura.exe` starts it and opens your browser. The Windows and macOS archives also contain `osatura-studio`, a small native window around the same studio (it has only been compiled and self-tested in CI); see [STUDIO.md](STUDIO.md) and [GETTING_STARTED.md](GETTING_STARTED.md).

**4. How do I get started quickly?**
Follow [GETTING_STARTED.md](GETTING_STARTED.md): it builds one animated character from an empty project, every step run, and ends with a GIF.

**5. Can I build without the scripting engine?**
Yes: `cargo build -p osatura-cli --no-default-features` leaves out Rhai, so the `script` command is absent (see [DEVELOPMENT.md](DEVELOPMENT.md)). The `scripting` feature is the only place a crate with a weak-copyleft licence (smartstring) comes in; see [THIRD_PARTY.md](THIRD_PARTY.md).

**6. Does it need a network connection or an account?**
No. The program needs no account and no network to work (the studio has no accounts). By default the studio listens on your own machine only (see question 51).

**7. Where does the studio keep its settings?**
In `$OSATURA_UI_CONFIG` if set, else `$XDG_CONFIG_HOME/osatura/ui.json`, else on Windows `%APPDATA%\OSATURA\ui.json`, else `~/.config/osatura/ui.json`. Your projects are never touched by it.

---

## ffmpeg

**8. Do I need ffmpeg?**
Only for MP4 and WebM export, for `sound mix`, for mp3/ogg audio and for `face-track` on a video file. GIF, APNG, HTML, PNG sequences, SVG, glTF and JSON export need nothing else. The full list is in [CLI_REFERENCE.md](CLI_REFERENCE.md), section 1.6.

**9. Is ffmpeg bundled?**
Never. OSATURA starts `ffmpeg` from your `PATH` as a separate program and nothing else is executed. Whatever licence and patent situation your ffmpeg build has is yours to check; see [PATENT_NOTES.md](PATENT_NOTES.md).

**10. What does it say when ffmpeg is missing?**
`cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` (for WebM the hint ends `or export an APNG instead`). Nothing is written.

**11. Which encoders must my ffmpeg have?**
`libx264` for mp4, `libvpx-vp9` or `libaom-av1` for webm, `aac` and `libopus` for sound in those, `libvorbis` and `libmp3lame` for `sound mix`.

---

## Performance

**12. Why is it so slow?**
You are probably running a debug build. The numbers in the documents come from a debug build and are 10 to 30 times slower than a release build ([CLI_REFERENCE.md](CLI_REFERENCE.md), section 1.7). On the 2-core test machine a debug build needed 46 s to write the 48 frames of the demo project as PNG and 57 s to export it as a GIF. A fresh release build did the same two jobs in 1.5 s and 2.2 s. Build with `cargo build --release` and use `target/release/osatura`.

**13. Does it use the GPU?**
The default renderer is the CPU. A `gpu` crate and a `render-gpu` command exist behind a feature flag, but no real GPU was available while building: on a software GPU (llvmpipe) a 640x480 frame took 253 to 295 ms against 13.8 ms on the CPU path. Treat GPU rendering as untested on hardware ([LIMITATIONS.md](LIMITATIONS.md)).

**14. How much does a rig cost per frame?**
In a debug build the `biped` example had a median of 64.5 ms per frame in the `ci` perf check. A cloth bake of a small flag took 0.43 s. Measure your own project with `osatura stats` and `osatura ci`; rigs differ a lot.

**15. How can I make a heavy project lighter?**
Use `lod` to make cheaper variants, `export-fit` to find the export settings that meet a byte budget, and `proxy-bench` to compare proxies. See [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) and [EXPORT_FORMATS.md](EXPORT_FORMATS.md). `export-fit` can stop early with `--max-seconds` and then reports it ran out of time.

**16. A script with thousands of calls fails. Why?**
`osatura script` accepts at most 5000 requests per run and is all-or-nothing: if any request is rejected nothing is written. Split a big job into several scripts that read the previous output.

---

## Determinism

**17. Is the output reproducible?**
Rendering is built to be: the same project and the same seed give the same pixels, and physics, particles and noise are seeded. This is what lets `golden` and `ci` compare renders. It has been checked on one machine type only (Linux x86-64); bit-identical output on other CPUs or compilers is not promised.

**18. Why does a physics bake give a different result after I change one number?**
The bake is deterministic for a given input, but it is a simulation: any change in mass, frame rate, collider or seed changes the whole run. Bake with the same settings and seed to compare. Baking merges with existing animation by default (`merge=keep`: existing channels are kept and the command says `kept existing animation on 'b2' (use merge=replace to overwrite)`); use `merge=replace` to overwrite.

**19. Does noise loop?**
No. A `noise` channel does not repeat on its own, so a loop will show a jump at the seam. Use keys or an expression such as `8 * sin(t * 2 * pi)` instead; in expressions `t` runs 0 to 1 over the clip, not in frames ([ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)).

**20. Can I check that an animation loops cleanly?**
Run `lint`. `make-loop` removes travel entirely: in a test a rolling ball became static and `lint` warned (L005). `loop-solve` closes the seam for cyclic motion but left a travelling object with a visible jump (280 to 280 on its travel channel). For continuing travel, use the extrapolate mode `loop_offset`.

---

## Licensing and your output

*This is a plain-language summary of [LICENSING.md](../LICENSING.md), not legal advice. A lawyer's review of the licensing set-up is still outstanding.*

**21. What licences apply?**
MPL-2.0 for the core library, C ABI, WASM and GPU crates; AGPL-3.0-or-later for the command-line program and the studio (`osatura-cli`); CC-BY-4.0 for the documentation and for the synthetic example assets and models.

**22. Who owns the animations and renders I make?**
You do. The project statement is that projects, animations and renders you create with OSATURA belong entirely to you; the licences apply to the software, not to its output. Exporting a GIF or running a command triggers no obligation.

**23. Can I use the output commercially?**
Yes, under the statement above. The exception is anything you copy from the example assets: those are CC-BY-4.0 and need attribution.

**24. Do I have to publish my projects if I use the AGPL parts?**
No. The AGPL covers the program. If you run a modified `osatura-cli` or studio so that other people use it over a network, you must offer them its source. The studio does this with a `/source` page and a `--source-url` flag that points to your copy.

**25. Can I link the core library into a closed-source application?**
The core is MPL-2.0, which is file-level copyleft: you may use it from closed code and must share changes to the MPL-covered files themselves. The CLI and studio are AGPL and are a different matter.

**26. May I say my tool "works with OSATURA"?**
Yes: "compatible with OSATURA" and "made with OSATURA" are allowed. A modified build must not be distributed under the same name ([TRADEMARK.md](../TRADEMARK.md)).

**27. Can I dual-license my contribution?**
There is no contributor licence agreement. Contributions are accepted under the project licence with a DCO sign-off (`git commit -s`), so there is no owner who could relicense them. Details in [CONTRIBUTING.md](../CONTRIBUTING.md).

**28. What about the third-party code?**
About 360 third-party crates (counted over all platforms; the window app adds the Windows / macOS toolkit), all under permissive licences except six MPL-2.0 ones: smartstring (MPL-2.0 or later, only with the `scripting` feature) and five that only the window app can pull in. Lists are in [THIRD_PARTY.md](THIRD_PARTY.md) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

**29. Are there patent concerns?**
See [PATENT_NOTES.md](PATENT_NOTES.md). It is engineering notes, not a legal opinion.

---

## File safety and asset paths

**30. Can a command damage my project?**
Writes are atomic: the result goes into a temporary file in the same directory, is synced, and is renamed over the target, so a crash leaves the old file. Editing commands re-validate the project they are about to write and refuse to write an invalid one (`output would be invalid: ...`). The `script` command applies nothing if any request fails.

**31. Is it safe to open a project from someone else?**
Reasonably, but not unconditionally. A project is data, not code; the one external program OSATURA starts is ffmpeg, for the formats listed in question 8. The loader refuses devices, pipes and directories, caps the input size, and refuses absolute or `..` asset paths. Rhai scripts run in a sandbox. Still read [SECURITY.md](SECURITY.md) before processing untrusted files in a service.

**32. Where does a project look for sounds and images?**
Relative to the folder of the project file. Absolute paths and any path with `..` are refused unless you pass `--allow-external-assets`, and you must pass it again on every later command that loads that project. Example from the reference:
`sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)`.

**33. How big a project will it read?**
Up to 256 MiB by default. Change it with `--max-input-bytes N`. Larger files fail with `cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)` (the numbers are the example's).

**34. Why does it refuse to write to a symbolic link?**
So a link planted next to your output cannot redirect a write elsewhere. The message is `refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself`.

**35. Why can I not read from `<(...)` or a pipe?**
Inputs must be regular files: `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)`. Write the data to a file first.

**36. Does an unknown field in a project break it?**
No: `warning: unknown field bogus (ignored)` is printed and the exit status is 0. Side files you write for a command (contracts, mappings, machines) are stricter: an unknown key there is an error and nothing is applied.

---

## Why the output must differ from the input

**37. Why does `render in.png in.png` say "refusing to overwrite"?**
For commands that read a project and write a picture, video or other non-project file, the output name may not be the input name. The message is `refusing to overwrite the input project X with output of the same name`. It prevents the classic slip of exporting over your only copy of the project.

**38. Then why can `script` and `autorig` write over their own input?**
The guard covers non-project outputs. Commands that read a project and write a project (`script`, `autorig`, `loop-solve` and others) may name the same file for input and output; the write is atomic, and if the command fails nothing changes. If you want the original kept, give a new name, or use `osatura vcs` to keep history ([CI.md](CI.md) and [CLI_REFERENCE.md](CLI_REFERENCE.md)).

**39. Why does `render` refuse `out.jpg`?**
`render` writes `.png` or `.svg` only: `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)`. For other formats use `export`; the list is in [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

---

## Rigging and animation

**40. Which way is zero degrees for a bone?**
Bone angle 90 points down; angles are not measured from the x axis in the way a maths textbook does. Channels are deltas from the rest pose, and `rest_offset` is in the parent's frame ([RIGGING_GUIDE.md](RIGGING_GUIDE.md)).

**41. My IK chain will not stretch or has a snap. What are `soft` and `stretch`?**
`soft` (0 to 0.9) eases the chain into full extension and `stretch` (1 to 4) lets it lengthen. Both are JSON-only, not Rhai or CLI options. With `stretch` above 1, the soft zone is the end of the stretched reach. Two-bone chains are solved analytically; three or more use FABRIK.

**42. My constraints give odd results.**
Constraints run in list order, and order matters. An aim with weight 0.5 gives half the angle (an angle of -74.7 becomes -37.4). Reorder in the JSON if a later constraint should see an earlier one's result.

**43. Is there an automatic skinner for vector art?**
Not on the command line. `Project::auto_skin` exists in the library only. For vector art you rig with bones parented to layers, with lattices, spines or pins (JSON-only), or with traced meshes via `mesh-trace`.

**44. `skin-diffuse` and `cloth` complain about the mesh.**
Both need a mesh, and cloth needs a **grid** mesh: `cloth needs a grid mesh; the layer has a free-form (traced) mesh`. A layer without any mesh gives `layer 'flag' has no mesh`.

**45. `foot-lock` says it needs a root.**
`foot lock needs a root controller bone (no parent)`. The root controller must have no parent bone.

**46. `scale` refuses my keyframes.**
`scale: frame 27 is outside the project range 0..=24` means a result key would fall outside the project; `keyedit scale` is refused when out of range unless you add `--free`. A dry run uses `-` as the output name.

**47. Why did my project get longer after `mocap-retarget`?**
`mocap-retarget` extends the project length to fit the captured motion. `follow --chain` can shift keys beyond the last frame, which `lint` reports (L007).

**48. How do I animate a layer without a bone?**
Edit the layer's `layer_x` and `layer_y` channels. In the studio, turn on auto-key; with it off a layer edit stays static unless that property already has a keyed channel. Bone edits always key. See [STUDIO.md](STUDIO.md).

---

## Studio

**49. How do I open the studio?**
`osatura studio project.json` and open the address it prints, normally `http://127.0.0.1:<port>`. It is a single-page app served by OSATURA's own small HTTP server (standard library only); it needs no internet.

**50. Does the studio change my file while I work?**
No. The file on disk changes only when you press Save. Closing the page without saving loses the edits (the collaboration log, if enabled, keeps the whole project, but not undo history).

**51. Can other people connect?**
Only if you bind a non-loopback address, and then a token is required: `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)`. The token must be at least 16 characters, for example from `openssl rand -hex 16`. There is no encryption: use an SSH tunnel or a reverse proxy with TLS.

**52. Where do I put the token?**
In the `X-Collab-Token` header or the `osatura_token` cookie (HttpOnly, SameSite=Strict, and Secure when the request arrives under an https `--public-origin`). It is never accepted in the URL. From the fifth failed attempt, that client address gets HTTP 429 with a growing delay (2, 4 ... up to 30 s).

**53. What do the HTTP errors 403, 415 and 421 mean?**
421: the Host header is not one the studio expects. 403: a foreign `Origin` or `Sec-Fetch-Site`. 415: a POST that is not JSON. These are protections against pages in your browser calling the local server; see [STUDIO.md](STUDIO.md) and [SECURITY.md](SECURITY.md).

**54. Does it work in any browser?**
It was tested only in headless Chromium. Others are untested. There is no canvas zoom command, and an in-studio panel for webcam face tracking exists but a real camera was never available for testing ([LIMITATIONS.md](LIMITATIONS.md)).

---

## Face, speech and motion capture

**55. Does OSATURA track faces?**
There is a small built-in detector, trained on synthetic faces only. On 50 synthetic test scenes (seed 9) it found every face, with a mean landmark error of 0.028 of the face size, and it raised 2 false alarms in 50 scenes without a face. On cartoon-style drawings it did poorly: 0 of 6 boxy-robot faces were found, and the round-emoji and manga-oval cases had errors of 0.164 and 0.166. It has never been run on real photographs or a real webcam. Note that [LIMITATIONS.md](LIMITATIONS.md) L-13 and the studio's webcam panel text say no face detector is bundled; that is out of date with respect to this synthetic one.

**56. How does lip-sync work?**
Mouth cues choose between five shapes (rest, small, open, round, wide). A cue time with a decimal point is seconds; a bare integer is a frame. `--rhubarb` only changes how the letter E is classified (round instead of wide). Audio longer than the project is truncated.

**57. Can I animate from video?**
`face-track` reads video through ffmpeg. `face-bake` writes the signals as keyed channels (the summary reads "+N channels (K keys in them), M trigger keys set") and truncates to the project length, with a stderr warning counting the frames not baked. An unknown signal gives `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.

---

## Bugs, security and contributing

**58. How do I report a bug?**
Open an issue with the command line, the exact output (stderr too), the exit status, the build (`osatura --version`) and, if you can, a minimal project. Run `osatura check project.json` and `osatura lint project.json` first: many problems are reported there. See [CONTRIBUTING.md](../CONTRIBUTING.md).

**59. How do I report a security problem?**
Privately, not in a public issue; see [SECURITY.md](SECURITY.md). The address in the repository is `alexio@trom.tf`; that is the same address as the maintainer contact and has not been replaced by a security-team mailbox.

**60. Is there a mailing list, chat or support contract?**
No. The documents name one maintainer contact; the first release is still a draft pre-release.

**61. How do I contribute?**
Sign off each commit (`git commit -s`), keep the tests passing (`cargo test`, and `osatura ci` for animation work) and read [DEVELOPMENT.md](DEVELOPMENT.md). Documentation is CC-BY-4.0 and welcomes corrections, especially where a page and the program disagree.

---

## Limitations

**62. What is the single most important limit?**
It has been built and tested by one project in one environment. Anything not listed as run is "not tested". Read [LIMITATIONS.md](LIMITATIONS.md); it has ids you can quote in issues.

**63. Which export formats need care?**
Video needs ffmpeg. Lottie and glTF cover a subset of what OSATURA can draw and animate, so check what is lost in [EXPORT_FORMATS.md](EXPORT_FORMATS.md). The `export` error for an unknown extension lists `.gif .html .json .mp4 .apng .webm .png .glb`, which omits `.lottie`; `.lottie` (dotLottie) is a valid target, see [EXPORT_FORMATS.md](EXPORT_FORMATS.md), section 6.

**64. Is the WASM build usable?**
It builds for `wasm32-unknown-unknown` on GitHub Actions and the wrapper's test passes against the real module under Node 20, and it is attached to the draft release as `osatura-web.tar.gz`. It has not been run in a browser, and its speed and memory are unmeasured ([WASM.md](WASM.md), [LIMITATIONS.md](LIMITATIONS.md)). Treat it as partly verified.

**65. Does it import other tools' project files?**
It imports SVG, OpenRaster, Lottie JSON, images, GIF/APNG, BVH motion capture and MIDI ([EXPORT_FORMATS.md](EXPORT_FORMATS.md), sections 9 and 10). It does not import PSD, `.lottie` zip files or OCA folders. Imports create a new project. Note that `import-lottie in.json in.json` has no overwrite guard and replaces its input. Do not expect an imported file to match the original pixel for pixel.

**66. Is the project JSON stable?**
It is "format 1". A newer file is refused: `project format 2 is newer than this build supports (1); update OSATURA`. Since only an alpha pre-release exists, format 1 itself may still change before a stable release; see [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

---

## Comparisons with other tools

**67. How does it differ from a timeline-based desktop animation editor?**
Those are interactive, drawing-first programs. OSATURA is command-first: every operation is a command or a script, the project is diffable JSON, and renders are deterministic so they can be tested in CI. The studio is a thin web view over the same engine, and it is far less polished than a mature editor.

**68. How does it differ from a 2D skeletal animation tool for games?**
Those tools focus on meshes, skins and runtime libraries for specific game engines. OSATURA has bones, IK, constraints and meshes too, but a smaller runtime story: its own player in HTML, a C ABI and a WASM crate that is unbuilt. Choose by the runtime you must target.

**69. How does it differ from a motion-graphics compositor?**
A compositor is built around layers with effects and expressions on video footage. OSATURA has layer effects and expressions, but its centre is character rigging and vector art, and it does not do video compositing, 3D or colour grading of footage.

**70. When is OSATURA the wrong choice?**
When you need real-time performance on a GPU today, a polished drawing tablet workflow, frame-by-frame hand-drawn animation, a vendor-supported product with a stable release and support, or confirmed behaviour outside Linux x86-64.

---

## Troubleshooting table

Every message below was produced by a real build and ends with exit status 1 unless noted. The wording of a message may change before a release.

| Message | Cause | Fix |
|---|---|---|
| `cannot read nope.json: No such file or directory (os error 2)` | Wrong path or name | Check the path and the current directory |
| `invalid JSON in broken.json: EOF while parsing a value at line 2 column 0` | The file is not complete JSON | Fix the syntax at the line named; restore from backup or `osatura vcs` |
| `badparent.json is not a valid project:` then `bone 'a' has parent 'ghost' which is missing or listed after it` | A parent bone is missing or listed after its child | Define parents before children; fix the name |
| `invalid JSON in fmt2.json: project format 2 is newer than this build supports (1); update OSATURA` | File from a newer build | Use a newer build |
| `warning: unknown field bogus (ignored)` (exit 0) | A key this build does not know | Remove or fix the key; check for a typo |
| `refusing to overwrite the input project X with output of the same name` | Output file name equals the input | Choose a different output name |
| `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)` | Wrong extension for `render` | Use `.png`/`.svg`, or `export` for other formats |
| `unknown option --bone for 'render' (did you mean --bones?); see: osatura (no arguments) for the usage` | Misspelt option | Use the suggested option |
| `unknown export format '.bmp' (use .gif .html .json .mp4 .apng .webm .png .glb)` | Extension not exportable | Pick a listed format (`.lottie` is also valid) |
| `--scale needs a number in (0, 16]` | Scale out of range | Use a value above 0 and up to 16 |
| `bad frame number 'x'` | A frame argument is not a whole number | Pass an integer |
| `cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` | ffmpeg not on `PATH` | Install ffmpeg, or export GIF, HTML or APNG |
| `the project has no sound tracks (see: sound add)` | Sound command on a silent project | Add a track with `sound add` |
| `channel hand_ctrl.BoneOffsetY is locked` | The channel is locked | Unlock it, or edit another channel |
| `request 1 (hand_ctrl.BoneOffsetY at 0) was rejected, nothing changed: channel hand_ctrl.BoneOffsetY is locked` | Same, from `script` | Unlock first; nothing was applied |
| `request 1 (channel head.BoneAngle) was rejected, nothing changed: ... expression error: unknown variable 'tt'` | Typo in an expression | Variables are frame, time, fps, frames, pi, t |
| `... unexpected end of expression` | Unfinished expression | Close brackets, finish the formula |
| `points need an even number of coordinates: 3..1000 points` | Odd coordinate count or too few points | Give x,y pairs: 3 to 1000 points |
| `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)` | Process substitution or pipe as input | Save to a regular file first |
| `sound add: asset '/etc/passwd': absolute paths are not allowed (...)` | Asset outside the project folder | Copy it next to the project, or pass `--allow-external-assets` |
| `cannot read demo.json: N bytes exceed the input limit of M bytes (raise it with --max-input-bytes N)` | File above the input limit | Raise `--max-input-bytes` if you trust the file |
| `refusing to write through a symbolic link; pass --follow-symlinks ...` | Output is a symbolic link | Name the target, or pass `--follow-symlinks` |
| `cloth needs a grid mesh; the layer has a free-form (traced) mesh` | Cloth on a traced mesh | Build a grid mesh for the layer |
| `layer 'flag' has no mesh` | A mesh command on a plain layer | Create a mesh first |
| `no bone 'b9'` | Bone name does not exist | Check names with `stats` or `explain` |
| `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)` | Wrong signal name | Use a name from the list |
| `foot lock needs a root controller bone (no parent)` | The chosen root has a parent | Use a parentless controller |
| `scale: frame 27 is outside the project range 0..=24` | Scaled keys fall outside the project | Lengthen the project, scale less, or use `--free` |
| `--extrapolate must be hold, linear, loop, ping_pong or loop_offset` | Bad mode | Use one of the five |
| `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)` | Port taken | Pick another `--port` or stop the other process |
| `cannot listen on 127.0.0.1:99999: invalid port value` | Port above 65535 | Use 1 to 65535 |
| `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (...)` | Public bind without a token | Add `--collab-token`, or stay on loopback |
| `--collab-token must be at least 16 characters (e.g. \`openssl rand -hex 16\`)` | Weak token | Generate a longer one |
| `--source-url must be an http:// or https:// URL without spaces or quotes` | Malformed URL | Use a plain URL |
| `--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org` | Origin has a path | Give only scheme, host and port |
| `output would be invalid: ...` | The edit would leave an invalid project | Read the reasons after the colon; nothing was written |
| `budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes ...` | `export-fit` cannot reach the budget | Raise the budget or simplify the project (the smallest file is still written) |
| `time budget exhausted (60 s ...)` | `export-fit` ran out of time | Raise `--max-seconds` |
| `fuzz: failure rate 98.00% exceeds --max-fail-rate 50.00%` | Poses broke the rig, and you asked for a gate (`--gate` or `--max-fail-rate`) | Look at the failing cases; raise the rate only if intended; plain `fuzz` only reports (exit 0) |
| `no .osatura-vcs repository found from ... (run: osatura vcs init <project.json>)` | No history yet | Run `vcs init` |
| `frame 999 out of range` (studio API) | Frame beyond the project | Use a frame within 0 to the last frame |
| `bad kind 'x' (vector, raster, png)` (studio API) | Wrong kind | Use a listed kind |
| `nothing selected` (studio API) | A selection command with no selection | Select layers or bones first |

Exit codes for `osatura ci` are 0 pass, 1 fail, 2 error.

---

*Documentation CC-BY-4.0. Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio). You may share and adapt it with attribution. See also: [INDEX.md](INDEX.md), [GLOSSARY.md](GLOSSARY.md).*
