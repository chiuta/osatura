# Getting started with OSATURA

This guide takes you from a fresh checkout to a small animated character that you built, rigged,
keyed, looped, rendered and exported using nothing but the command line. Every command below was
run for real while this page was written; the outputs shown are the outputs that were printed.

How it was verified: Linux, `rustc 1.97.0`, `ffmpeg 6.1.1`, and the **debug** build of the binary
(`cargo build -p osatura-cli`). A release build runs the same code faster. The test suite also passes on macOS and Windows in GitHub Actions; the commands on this page were run by hand only on Linux. No minimum supported Rust
version is declared in the repository; 1.97.0 is simply the compiler that was tested.

Documentation licence: CC-BY-4.0. See also [RIGGING_GUIDE.md](RIGGING_GUIDE.md),
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md), [STUDIO.md](STUDIO.md), [FAQ.md](FAQ.md) and the full
command list in [CLI_REFERENCE.md](CLI_REFERENCE.md).

## 1. What you need

| Requirement | Needed for | Notes |
|---|---|---|
| Rust toolchain (`rustc`, `cargo`) | building OSATURA | install from <https://rustup.rs>; stable channel |
| A web browser | the studio (`osatura studio`) | only Chromium was tested, see [STUDIO.md](STUDIO.md) |
| `ffmpeg` on `PATH` (optional) | `.mp4`, `.webm`, mixing sound into video/HTML, `sound mix`, `face-track` on a video file | everything else works without it |

Without `ffmpeg` the commands that need it stop with a clear message and write nothing:

```
$ PATH=/nonexistent osatura export buddy-final.json o.mp4
cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead
```

There is no GPU requirement. The optional GPU renderer (`render-gpu`) is not part of the default build.

**No Rust? Use a prebuilt archive.** When the owner has published the draft pre-release `v0.1.0-alpha.3`, the [Releases page](https://github.com/chiuta/osatura/releases) offers `osatura-windows-x86_64.zip`, `osatura-macos-arm64.tar.gz`, `osatura-linux-x86_64.tar.gz` (glibc 2.35+; later releases add the static `osatura-linux-x86_64-musl.tar.gz` for older distributions) and `SHA256SUMS.txt`. Check the archive against the sums (`sha256sum -c SHA256SUMS.txt --ignore-missing`, or `Get-FileHash` on Windows), unpack it and run `osatura` (`osatura.exe`) from the unpacked folder; skip the build in section 2 and read "Windows: double-clicking `osatura.exe`" and "The studio as a window" near the end of this page. The programs are not code-signed, so the system will warn you the first time. Everything below was run with a local build of the same source.

## 2. Build it

From the root of the repository:

```
cargo build --release -p osatura-cli
```

The binary is `target/release/osatura`. Put it on your `PATH`, or alias it:

```
export PATH="$PWD/target/release:$PATH"      # or: alias osatura=$PWD/target/release/osatura
osatura --version
```

`--version` prints the version, the licence (AGPL-3.0-or-later for the CLI and studio, MPL-2.0 for the
engine libraries) and the source offer. Running `osatura` with no arguments prints the complete usage;
`osatura <command> --help` prints the usage of one command.

You can also run it without installing: `cargo run --release -p osatura-cli -- demo demo.json`.

Building the command-line tool with `--no-default-features` drops the `script` command (the Rhai
scripting engine); this tutorial uses `script`, so keep the default features.

## 3. First steps with the built-in demo

`osatura demo` writes a complete example project: a skinned arm, an IK leg, a lip-sync switch layer,
a parallax background and a camera move.

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
```

`check` compiles the project exactly as the renderer would, so a project that passes `check` can be
rendered. Now draw a still, with the bones overlaid, and export an animation:

```
$ osatura render demo.json 12 frame12.png --bones
wrote frame12.png
$ osatura render demo.json 12 frame12.svg
wrote frame12.svg
$ osatura export demo.json demo.gif
wrote demo.gif
$ osatura export demo.json demo.html
wrote demo.html
$ osatura export demo.json demo.lottie
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
wrote demo.lottie
```

What you just did:

* `render <project> <frame> <out.png|out.svg>` draws one frame. `--bones` overlays the skeleton,
  `--scale N` renders at N times the canvas size (0 < N <= 16).
* `export` chooses the format from the file extension: `.gif`, `.html` (a self-contained player page),
  `.json` (Lottie), `.lottie` (dotLottie), `.apng`, `.webm`, `.mp4`, `.glb`, `.png` (sprite sheet).
* The `warning:` lines are the honest part: Lottie cannot represent everything OSATURA can. Every
  limitation is printed on stderr, and `--report r.json` writes the same list as JSON.
* Open `demo.html` in a browser to play the animation; no server and no network are involved.

To explore interactively:

```
osatura studio demo.json --port 8080
```

and open <http://127.0.0.1:8080/>. See [STUDIO.md](STUDIO.md).

## 4. A 15-minute tutorial: a waving character

You will build "Buddy" from an empty project. The project file is JSON, but you never have to write
more than a header: the `script` command runs a small sandboxed [Rhai](https://rhai.rs) script through
the same validated editor the studio uses. If a request in a script is rejected, *nothing* is applied
(all-or-nothing), so a bad script never leaves a half-edited file.

Work in an empty directory. Each step writes a new numbered file, so you can always go back.

### Step 1: an empty project (1 minute)

```
cat > buddy.json <<'EOF'
{"format": 1, "name": "Buddy", "width": 320, "height": 240, "fps": 24, "frames": 24}
EOF
osatura check buddy.json
```
```
ok: 'Buddy' 320×240 @ 24 fps, 24 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Those six keys are the minimum. 24 frames at 24 fps is one second, which will be our loop.

### Step 2: shapes (2 minutes)

Layers are drawn back to front, in the order you add them. Create `shapes.rhai`:

```
add_gradient_rect("sky", 0, 0, 320, 240, "#bfe3f5", "#fdf3d8");
add_rect("ground", 0, 200, 320, 40, "#7fa66b");
add_rect("torso", 140, 120, 40, 70, "#e07a5f");
add_circle("head", 160, 95, 24, "#f2cc8f");
add_rect("upper_arm", 160, 134, 40, 12, "#3d405b");
add_rect("forearm", 200, 134, 40, 12, "#81b29a");
```
```
$ osatura script buddy.json 01-shapes.json shapes.rhai
6 request(s) applied; wrote 01-shapes.json
```

Coordinates are canvas units with the origin at the top left and y pointing down. The arm is drawn
horizontally, shoulder at (160, 140), because that is the pose in which we will build its bones.

### Step 3: bones, parenting and attaching art (3 minutes)

A bone has a name, a parent (empty = root), a length, a head position (`x`, `y`) in its parent's space
(for a root: in the canvas) and a rest angle in degrees relative to its parent. Angle 0 points along +x,
and positive angles turn clockwise on screen, so 90 points down and -90 points up.
Create `bones.rhai`:

```
add_bone("body", "", 70, 160, 190, -90);
add_bone("neck", "body", 20, 70, 0, 0);
add_bone("arm_up", "body", 40, 50, 0, 90);
add_bone("arm_low", "arm_up", 40, 40, 0, 0);
add_bone("hand_ctrl", "", 10, 210, 165, 0);
set_layer("torso", "bone", "body");
set_layer("head", "bone", "neck");
set_layer("upper_arm", "bone", "arm_up");
set_layer("forearm", "bone", "arm_low");
```
```
$ osatura script 01-shapes.json 02-bones.json bones.rhai
9 request(s) applied; wrote 02-bones.json
$ osatura render 02-bones.json 0 rig.png --bones --scale 2
wrote rig.png
```

Read the bones like this: `body` is a root bone at (160, 190) that points up (-90) and is 70 long.
`neck` is its child; its head sits 70 units along `body` (at the top). `arm_up` is another child of
`body`, 50 units up the spine (the shoulder), turned +90 so it points sideways. `arm_low` is the child
of `arm_up`: parenting means rotating the upper arm carries the forearm with it. `hand_ctrl` is a
separate root bone that we will use as an IK target; it has no art.

`set_layer(layer, "bone", bone)` makes a layer ride a bone *rigidly*: it follows the bone's motion
relative to its rest pose. That is the simplest way to rig a cartoon made of separate pieces. (For
bendy shapes you skin vector anchors to several bones; see [RIGGING_GUIDE.md](RIGGING_GUIDE.md).)

### Step 4: inverse kinematics (1 minute)

Instead of keying the shoulder and elbow angles, let the solver reach for `hand_ctrl`. Create `ik.rhai`:

```
add_ik("arm_ik", ["arm_up", "arm_low"], "hand_ctrl", 1);
```
```
$ osatura script 02-bones.json 03-ik.json ik.rhai
1 request(s) applied; wrote 03-ik.json
```

The last argument is the bend direction: +1 or -1 chooses which way the elbow folds. A two-bone chain uses
an analytic solver; chains of three or more bones use FABRIK. From now on, moving `hand_ctrl`
moves the whole arm.

### Step 5: keys and easing (3 minutes)

`set_key(target, property, frame, value, ease)` creates keys. For a bone, `bone_angle`, `bone_scale`,
`bone_offset_x` and `bone_offset_y` are *deltas from the rest pose*, so a key value of 0 means
"rest". The easing name applies to the segment that *starts* at that key. Create `keys.rhai`:

```
set_key("hand_ctrl", "bone_offset_y", 0, 0, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 12, -75, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 24, 0, "ease_in_out");
set_key("neck", "bone_angle", 0, 0, "smooth");
set_key("neck", "bone_angle", 16, 10, "smooth");
set_key("neck", "bone_angle", 24, 0, "smooth");
```
```
$ osatura script 03-ik.json 04-keys.json keys.rhai
6 request(s) applied; wrote 04-keys.json
$ osatura timeline 04-keys.json
timeline: 24 frames at 24 fps, 2 channels
Bones
  hand_ctrl
    #0 bone_offset_y (keys @ 0 12 24)
  body
    neck
      #1 bone_angle (keys @ 0 16 24)
```

The easing names that need no parameters are `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`,
`smooth` and `bounce`. `bezier:x1,y1,x2,y2` is accepted by `keyedit`, and `spring` and `back` take
parameters (see [PROJECT_FORMAT.md](PROJECT_FORMAT.md)). The hand and the head use different timings on purpose: overlapping action
reads as life.

### Step 6: make it loop (1 minute)

Both channels start and end at the same value (frame 24 repeats frame 0), which is what a loop needs.
`loop-solve` checks the seam and sets the channels to loop:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
$ osatura lint buddy-final.json
0 finding(s), 0 error(s)
```

"seam 0.000 -> 0.000" means the last pose already equals the first, so nothing had to be corrected. A
project of N frames plays frames 0..N-1 repeatedly, so frame N is the same pose as frame 0 and is never
shown twice. If you author a loop so that the *last shown* frame equals the first, pass `--inclusive`
(the older convention). If your animation has net travel, `make-loop` removes it. See
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

`lint` runs static rig and animation checks (unreachable IK targets, dead channels, missing bones...);
it exits 1 when it finds errors, so it is safe to put in a script or CI job.

### Step 7: look at it, render it (2 minutes)

```
$ osatura stats buddy-final.json
```
prints a JSON summary (here: 5 bones, 1 ik chain, 6 layers, 2 channels, 6 keys). Render stills and a
whole frame sequence:

```
$ osatura render buddy-final.json 12 frame12.png --scale 2
wrote frame12.png
$ osatura render buddy-final.json 12 frame12.svg
wrote frame12.svg
$ osatura frames buddy-final.json frames --png
wrote 24 frames to frames
$ ls frames | head -3
frame_0000.png
frame_0001.png
frame_0002.png
```

![Four frames (0, 6, 12, 18) of the Buddy animation, once plain and once with the bones overlaid. The right arm lifts in an elbow-down wave while the head nods slightly.](img/tutorial-buddy.png)

Without `--png`, `frames` writes SVG files. Renders are deterministic: the same project and frame give
the same bytes on any machine with the same build, which is what makes the golden-frame checks of
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) possible.

### Step 8: export (2 minutes)

```
$ osatura export buddy-final.json buddy.gif
wrote buddy.gif
$ osatura export buddy-final.json buddy.html
wrote buddy.html
$ osatura export buddy-final.json buddy.json --report buddy-report.json
wrote buddy.json
$ cat buddy-report.json
{
  "count": 0,
  "format": "lottie",
  "warnings": []
}
$ osatura export buddy-final.json buddy.mp4        # needs ffmpeg
wrote buddy.mp4
$ osatura export buddy-final.json buddy.apng
wrote buddy.apng
```

File sizes of this tiny project: GIF 145 KB, Lottie JSON 21 KB, HTML player 50 KB, APNG 75 KB, MP4 6 KB.
`buddy.json` is Lottie; it has an empty warnings list because Buddy only uses features that Lottie can
express. The HTML file is a standalone player (open it in any browser).

A few rules worth remembering:

* `export` and `render` **refuse to overwrite the input project** (`refusing to overwrite the input
  project buddy-final.json with output of the same name`), and refuse to write through a symbolic link
  unless you pass `--follow-symlinks`. Commands that transform one project into another
  (`script`, `loop-solve`, `walk`, `autorig`...) do *not* have that check: they write atomically, so
  `osatura script p.json p.json x.rhai` edits in place. Use distinct names until you trust a script.
* GIF has 256 colours and 1-bit transparency; for smooth alpha use `.apng` or `.webm`.
* Run `osatura export-fit` (see [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) when you need to hit a byte budget.

### Step 9 (optional): edit it in the studio

```
osatura studio buddy-final.json --port 8080
```

Drag the layers, select a key in the dope sheet, press `Space` to play. Nothing is written to your file
until you press **Save** (`Ctrl+S`).

## 5. How the pieces fit

| Concept | In the file | You make it with |
|---|---|---|
| Layer (vector, image, group, switch...) | `layers` | `add_rect`, `add_circle`, `add_polygon`, `add_line`, `import-svg`, `autotrace`... |
| Bone | `bones` (parents before children) | `add_bone`, `set_parent`, `reparent` |
| IK chain / constraint / spring | `ik`, `constraints`, `springs` | `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`; springs by JSON |
| Channel (an animated property) | `channels` | `set_key`, `keys`, `keyedit`, expressions, noise, bakes |
| Clip, take, state machine, binding | `clips`, `machines`, `bindings` | `take-record`, `clip`, `machine`, `bind` |

Everything is plain JSON, which is why diffs (`osatura diff a.json b.json`), version control
(`osatura vcs`) and scripts work. Unknown fields are reported as warnings and ignored:

```
$ osatura check unk.json
warning: unknown field bogus (ignored)
ok: 'X' 100×100 @ 24 fps, 10 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

A project may only reference image and sound files **inside its own directory**. Absolute paths and `..`
are refused unless you pass `--allow-external-assets` (see [FAQ.md](FAQ.md)).

## 6. Where to go next

* Rig something bendy (skins, weights, lattice, cage, cloth, physics): [RIGGING_GUIDE.md](RIGGING_GUIDE.md).
* Walk cycles, clip stacks, lip-sync, mocap, state machines, QA and size budgets:
  [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).
* The browser editor, its shortcuts and the HTTP API: [STUDIO.md](STUDIO.md).
* Questions and error messages: [FAQ.md](FAQ.md). Known limits: [LIMITATIONS.md](LIMITATIONS.md).
* Every command and flag: [CLI_REFERENCE.md](CLI_REFERENCE.md); the file format: [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Windows: double-clicking `osatura.exe`

`osatura.exe` is a command-line program. Since 0.1.0-alpha.2, opening it from Explorer (double-click) creates `Documents\OSATURA\first-rig.json` from the walking-person demo (`osatura demo-person`; an existing file is never overwritten), starts the studio on `http://127.0.0.1:8080` and opens it in your browser; the console window stays open while the studio runs, close it to stop. Opened from a terminal without arguments it prints the usage as before. In 0.1.0-alpha.1 a double-click only printed the usage and the window closed at once.

The executables are **not code-signed**: Windows SmartScreen shows "Unknown publisher" (choose *More info* then *Run anyway*) and macOS Gatekeeper asks you to approve the program in *System Settings > Privacy & Security*. Check the download against `SHA256SUMS.txt` first. If port 8080 is busy, run `osatura studio first-rig.json --port 8090` from a terminal.

## The studio as a window: `osatura-studio` (Windows and macOS)

The Windows and macOS archives contain a second program, `osatura-studio` (`osatura-studio.exe`), a small native window around the browser studio: it starts the `osatura` engine from the same folder as `studio` on a free local port, shows the page in a WebView2 (Windows 11 has it built in; older Windows needs the Evergreen runtime) or WKWebView (macOS) window, and stops the server when the window is closed. No console window appears. Keep it in the same folder as `osatura(.exe)`.

* **Exports** (GIF, HTML player, Lottie) open a native "Save as" dialog that starts in the project's folder, and a message names the file when it is saved. (alpha.2 saved them silently into your Downloads folder; alpha.3 asks.)
* Double-click it to work on `Documents/OSATURA/first-rig.json` (created from the walking-person demo, `osatura demo-person`, if missing), or start it as `osatura-studio my-project.json` (for example through "Open with").
* `osatura-studio --selftest` starts the server, fetches the page and stops it without a window; the exit code is 0 when that worked (CI runs it on Windows and macOS).
* **Limits.** The window was used on a real Windows 11 desktop (alpha.2: studio and exports worked); the Save-as dialog of alpha.3 is compiled and self-tested in CI but nobody has tried it yet, and nobody has used the window on macOS. Other browser-style features that need the system (drag-and-drop from Explorer or Finder, opening files from the studio's own pickers) are those of the system web view and are untested. There is no Linux window (it would need GTK / WebKitGTK system packages): use `osatura studio` and a browser. The program is not code-signed (see above).
