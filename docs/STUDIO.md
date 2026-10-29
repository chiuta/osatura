# OSATURA studio

The studio is a browser editor served by the `osatura` binary itself: `osatura studio project.json`.
It has a canvas with selection and transform gizmos, a layer list, a bone list, a dope sheet and curve graph,
onion skin and motion trails, a command palette with rebindable shortcuts, pose suggestions, live-input and
face panels, GIF/HTML/Lottie export buttons, and a shared multi-user editing session. It has no accounts, no TLS and
no database: it is a small standard-library HTTP server in front of the same editor the command line uses.

Everything here was checked against the real binary (`osatura 0.1.0`, debug build, Linux) and, for the user
interface, in headless Chromium driven by Playwright. Screenshots were taken that way (1400x900, the built-in demo
project). Only **Chromium** was tested; see section 11. On Windows and macOS the release archives also contain `osatura-studio`, the same studio in a native window (compiled and self-tested in CI, not yet used by a person; see [GETTING_STARTED.md](GETTING_STARTED.md) and [LIMITATIONS.md](LIMITATIONS.md) L-75), and on Windows double-clicking `osatura.exe` starts the studio and opens the browser (L-76).

Licence of this document: CC-BY-4.0. For the command line see [GETTING_STARTED.md](GETTING_STARTED.md) and
[CLI_REFERENCE.md](CLI_REFERENCE.md); for server security see also [SECURITY.md](SECURITY.md).

![The studio with the demo project: toolbar on top, the canvas with an arm layer selected and its bones drawn, the Layers panel on the right, and the timeline below with key diamonds.](img/studio-overview.png)

## 1. Starting the studio

```
osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES]
                [--public-origin URL] [--source-url URL] [--collab-token T] [--collab-log LOG [--collab-nosync]]
```

```
$ osatura demo demo.json
$ osatura studio demo.json --port 8080
AGPL-3.0 source offer: http://127.0.0.1:8080/source
OSATURA studio on http://127.0.0.1:8080  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in /home/you/.config/osatura/ui.json      (your own home directory)
```

Open the address in a browser. The project is validated at start; a file that does not pass `check` is refused with the
same message. Unknown fields are warned about (`warning: unknown field bogus (ignored)`).

| Flag | Default | Meaning |
|---|---|---|
| `--port N` | 8080 | TCP port. An occupied port gives `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)`; a bad one `cannot listen on 127.0.0.1:99999: invalid port value`. |
| `--bind ADDR` | `127.0.0.1` | Interface to listen on. Anything that is not a loopback address **requires `--collab-token`**. |
| `--ui-config F` | `$OSATURA_UI_CONFIG`, else `$XDG_CONFIG_HOME/osatura/ui.json`, else on Windows `%APPDATA%\OSATURA\ui.json` (before alpha.3 there was none: settings were kept for the session only), else `~/.config/osatura/ui.json` | Where layout, shortcuts, recent commands and overlay settings are kept. It is a per-user file, never part of the project. |

The window application and the double-click start create the first project in `OSATURA_DOCUMENTS` when that variable is set, else in `Documents\OSATURA` (used by the portable package).
| `--allow-host NAMES` | none | Extra `Host` names to accept (comma separated), for access under a DNS name. |
| `--public-origin URL` | none | The bare origin (`https://studio.example.org`, no path) that a TLS reverse proxy presents; its `Host` and `Origin` are then accepted. |
| `--source-url URL` | none; in the official release binaries the commit they were built from (`https://github.com/chiuta/osatura/tree/<commit>`) | Where the corresponding source of *your* build can be downloaded (AGPL section 13); shown on `/source` and by `--version`. An explicit value always wins. |
| `--collab-token T` | none | Shared secret, **at least 16 characters** (`openssl rand -hex 16`). Required for non-loopback binds. |
| `--collab-log LOG` | none | Persist the shared editing session to a file and resume it after a restart. |
| `--collab-nosync` | off | With `--collab-log`: do not `fsync` after each edit (faster; a power cut can lose the last edits). Without `--collab-log` it is refused at start (`--collab-nosync needs --collab-log`). |

**Saving.** Edits live in memory. The project file on disk changes only when you press **Save** (`Ctrl+S`), which writes atomically
(temporary file then rename, never through a symbolic link). This is also true during a shared session (section 8).

## 2. Security model (read this before binding to a network)

The studio is built for a person on their own machine. What protects it:

* **Host allow-list.** Only requests whose `Host` header is one of the server's own names are answered; anything else gets **421**.
  With `--allow-host` and `--public-origin` more names are accepted.
* **Cross-site protection.** A request with a foreign `Origin`, or `Sec-Fetch-Site: cross-site`, gets **403**. A POST whose body is not
  `application/json` gets **415** (body-less POSTs such as `/api/undo` need no type).
* **Security headers** on responses: a restrictive `Content-Security-Policy` (`default-src 'self'` and no framing), `X-Frame-Options: DENY`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`.
* **Token for anything off-machine.** With `--collab-token`, every request needs the token as the `X-Collab-Token` header, or in the
  `osatura_token` cookie set by `POST /api/login` (HttpOnly, SameSite=Strict). The token is **never accepted in the URL**. Wrong tokens are slowed
  down per client address (from the fifth failure in a row that address gets **429** for 2, 4, ... up to 30 seconds), but the back-off cannot stop an
  attacker who keeps trying: use a long random token.
* **Participant secrets.** Joining a shared session returns a random 128-bit secret; every call made as that participant must send it as
  `X-Collab-Secret`. Knowing a participant's number is not enough.

I verified each of those with `curl` against a running server:

```
bad Host (evil.example):               421
allowed host (--allow-host):           200
cross-origin POST (Origin: evil.example): 403
POST from the --public-origin:         200
Sec-Fetch-Site: cross-site:            403
POST with Content-Type: text/plain:    415
no token, GET / or /api/state:         401
wrong X-Collab-Token:                  401
right X-Collab-Token:                  200
token in the URL (?token=...):         401
/source without a token:               200
unknown route:                         404   (GET /favicon.ico answers with a small SVG icon, without a token)
```

Startup refusals (exit code 1, nothing is started):

```
refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)
--collab-token must be at least 16 characters (e.g. `openssl rand -hex 16`)
--source-url must be an http:// or https:// URL without spaces or quotes
--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org
```

**Honest limits.** Traffic is plain HTTP: the token and everything else travel in clear text, so put the studio behind a TLS reverse proxy
(`--public-origin`) if it is reachable by anyone else. The `Set-Cookie` for the token gets the **`Secure` flag** when the request arrives under an `https://` `--public-origin` name (or with an https `Origin` / `X-Forwarded-Proto: https`); plain HTTP on loopback never does,
and behind a proxy every client appears to have the proxy's address, so the back-off throttles everybody at once. `/source` is open without a token
(the AGPL asks for that). There are no user accounts: whoever has the token can change the project and write the save path.

### The AGPL source offer

The studio is AGPL-3.0-or-later software: if you modify it and let other people use it over a network, you must offer them your
version's source. `/source` is that offer; set `--source-url` to where your build's source lives. Without it (a build you made yourself with a plain `cargo build` has none) the page says so and names the maintainer. The release binaries built by `.github/workflows/release.yml` carry the address of their own commit (`OSATURA_SOURCE_URL` at compile time; a fork building with its own workflow points to itself).

## 3. A tour of the interface

### Toolbar

Undo, Redo, **Save**; checkboxes `bones` (draw the skeleton), `auto-key`, `grid` (spacing in canvas units, 0 = off), `snap` (to other layers and the canvas),
`whole groups` (a click on part of a group selects the top-level group), `points` (show anchors, handles, cage/lattice/pin points and mesh vertices of
selected layers); a `box` selector for what a drag-box selects (`touches`, `contains`, `bounds touch`; it starts at `touches`); **Lock**; and the buttons **Overlays**, **Suggest**, **Layout**,
**Commands** (palette) and **Keys** (shortcut editor). A second row has your name and **Collab**, and the export buttons **GIF**, **HTML player** and **Lottie**,
which download what `osatura export` would write (the HTML player includes the mixed sound when there is sound and `ffmpeg` can make the mp3). The **Simple view** button hides the editing checkboxes, Overlays, Suggest and the command palette button so that only undo, save, panels and exports remain; the choice is remembered in this browser (or window) and the full toolbar comes back with **Full view**.

### Panels and layouts

The window is a dock of panels, each with a stable id: `stage` (the canvas), `layers`, `selection`, `bones`, `live` (Live input), `face` (Webcam face), `json`
(the whole project as editable JSON with an *Apply JSON* button) and `timeline`. Four layout presets exist:

```
animate  stage over timeline on the left, all panels as tabs on the right
rig      stage with a folded timeline; layers+bones and selection+json on the right (live and face hidden)
compact  stage over a tabbed timeline and panels
live     stage and timeline on the left; live, face, bones and selection on the right (layers and json hidden)
```

Switch presets from the **Layout** menu or the palette; drag tabs to re-dock or float a panel; "reset layout" returns to the default.
The layout is saved in the UI config, per user. `osatura ui layout presets|show NAME|validate FILE.json` prints and validates the same data
(a layout must keep every panel reachable; an incomplete one is rejected by the server with 400).

### Layers panel

![The Layers panel with a locked layer (sun), a hidden layer (hills), and arm and leg selected together.](img/studio-layers.png)

Each row has two checkboxes: **visible**, and **lock** (saved *with the project*; locked layers are skipped by clicks and boxes, and refuse edits).
The screenshot shows `sun` locked, `hills` hidden and two layers selected with Shift-click.

### Selection panel

States the rules in the page itself: *Click picks the topmost thing (locked and hidden ones are skipped); Shift-click toggles; drag on empty space
draws a box (hold Alt to draw a free-form lasso, Shift adds). Drag the box to move, a square to scale (Shift keeps proportions, Alt scales from the
centre), the round knob to rotate (Shift snaps to 15 degrees). Hold Ctrl while dragging to switch snapping off. Arrow keys nudge 1 unit (Shift: 10),
Delete removes layers, Esc cancels a drag or clears the selection.*

### Timeline: dope sheet and graph

![The dope sheet with a box selection of three keys at frame 12 (the keys are drawn solid, the others hollow).](img/studio-dopesheet.png)

![The curve graph view: three bone_angle curves (arm_up, arm_low, hand) in different colours with three selected keys at frame 12 showing their tangent handles; the status line reads "3 keys selected | frames 12".](img/studio-graph.png)

`Graph` (or `Alt+V`) toggles between the dope sheet and the curve graph. The timeline's controls: `Play`, `|<` (first frame of the range), `loop`, `range` with
`Set in` / `Set out` (play only between in and out), `snap` (round moved keys to whole frames), `on collision` (`replace`, `merge`, `refuse` when moved keys land on
existing ones), `Copy` / `Paste` / `Duplicate` / `Delete`, `interp` + `Apply` (set the ease of the selected keys), `Auto-smooth` (bake overshoot-free tangents into
selected keys), `ripple` N with `Insert` / `Remove` (and `grow project` to change the project length with them), zoom `-` `+` `Fit`, and `shy` (show channels marked shy).

Mouse: click a key to select, Shift-click to add, drag a box to select, drag keys to move them; **Alt-drag or the middle button pans**, **Ctrl+wheel zooms**,
**Shift+wheel scrolls** vertically; in the graph **Shift constrains the key drag to one axis**. Snap tolerance for canvas snapping is 6 canvas units by default.
Every key edit goes through the same validated operations as `osatura keyedit` (see [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)), so it is one undo step and locked channels refuse it.

### Overlays: onion skin, trails, proxies

![Onion skin and motion trails turned on for the arm at frame 14, with the Overlays panel open.](img/studio-overlays.png)

`Overlays` (`Alt+7`) holds onion skin (`Alt+N`: frames before/after in red/green tints, fixed step or key frames, outline mode, falloff, selected layers only),
motion trails (`Alt+M`: bone, layer pivot or control-point paths with tick marks per frame) and **scrub proxies** (`Alt+Q`: reduced-quality pictures while you drag the
playhead, with a picture cache and an optional level-of-detail copy). The panel reports, for example, `picture cache: 2 frames, 17 KiB, 0 hits / 7 misses, emptied 4x by edits`.
Settings are stored in the UI config and survive a restart. Overlays are never part of exports.

### Command palette and shortcuts

![The command palette opened with Ctrl+K and filtered by the word "toggle", listing matching commands with their shortcut hints.](img/studio-palette.png)

`Ctrl+K` (or `Ctrl+Shift+P`) opens the palette: type to filter (fuzzy, matched letters highlighted), arrow keys move, Enter runs, Esc closes. With nothing typed it
lists recent commands first. There are **84 commands**; `osatura ui commands` lists them and `osatura ui search onion` ranks them exactly as the palette does.

![The keyboard shortcut editor opened with Ctrl+comma, listing commands by category with their key chords.](img/studio-shortcuts.png)

`Ctrl+,` (or **Keys**) opens the shortcut editor: click a chord, press the new key combination. A conflict names the other command and changes nothing until you
confirm "Take it over". Browser-reserved chords are refused: `Ctrl+W`, `Ctrl+T`, `Ctrl+N`, `Ctrl+Q`, `Ctrl+R`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+Shift+W`, `Ctrl+Shift+N`, `F5`, `F11`, `F12`,
`Alt+F4`. Rebinding is stored in the UI config, and survives a reload. The server rejects a map with conflicts, an unknown command, an incomplete layout or a non-object body with 400.

### Suggest

![The Suggest panel showing three ranked in-between pose candidates (linear, pca, manifold) between frames 12 and 24, each with a score and an Apply button.](img/studio-suggest.png)

`Suggest` (`Alt+8`) proposes poses learned from the project's own key poses: an in-between of two frames, a completion of a partially fixed pose, or "what usually
comes next". On the demo project, between frames 12 and 24 it ranked `linear` 1.00, `pca` 1.00 and `manifold` 0.99, built from 5 poses on 2 axes (99 % of the variance).
`Apply` writes the pose as keys, one undo step. The same engine is `osatura pose-suggest` and `/api/suggest`. These are classical methods (linear blend, PCA, a kernel
manifold, an optional small network you train yourself); with a handful of poses the three agree.

### Live input and Webcam face panels

* **Live input** (`Alt+4`): mouse over the picture, keys, gamepad and Web MIDI are mapped to properties by a table; `Record` bakes a performance into keys as one undo
  step. Web MIDI needs a browser that supports it (Chromium does).
* **Webcam face** (`Alt+5`): landmarks drive the same mapping table (source `face`, ids such as `mouth_open`, `eye_r_open`, `head_yaw`). The buttons are
  *Start camera*, *Synthetic face*, *Synthetic face (detector)*, *Calibrate (hold neutral)* and *Face example mapping*. The camera path needs a landmark provider function
  that you supply (`window.osaturaLandmarkProvider`, see the comment in the page source); video never leaves the page and only landmarks go to the local server.
  The two synthetic modes work out of the box. *Synthetic face (detector)* renders the synthetic head on the server and finds it again with the built-in detector,
  which was **trained on synthetic faces only** (stylised heads, not photographs); see the face section of [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Guided demo

The **Demo** button (command `demo.panel`) opens a 13-step guided demo that builds the walking person of `osatura demo-person`: scenery, body, head and face, arms with five fingers per hand, the skeleton, the legs walking, the arms swinging, trunk and head, a blink and a smile, the moving ground, a check, and the export. Each step loads that stage of the project into the editor with one replace-all operation, so you can play it, scrub it, select bones and layers and read the timeline. The stages are cuts of the finished project; the commands and their output shown in the panel were run with the real binary.

* **Nothing is written to disk.** Save (and Ctrl+S) is disabled while the demo is loaded. The only write path of the studio, `POST /api/save`, is never called by the demo.
* **Your project comes back.** *Restore my project* puts back the project that was open before (a single replace-all operation, one Undo step). If you close the tab or reload while the demo is loaded, the next page load notices that the server still holds the demo, keeps Save off and offers *Restore my project* again; the original is kept in the browser's local storage for that.
* The demo refuses to start inside a collaboration session, because it replaces the whole project.
* Regenerate the stages with `node scripts/build-studio-tour.mjs --bin target/release/osatura`, then rebuild the CLI (the page is embedded in the binary).

## 4. Keyboard shortcuts (defaults)

This is the output of `osatura ui shortcuts` for a fresh configuration: **72 bindings, 84 commands, 0 conflicts**. Shortcuts belong to a **context**:
`global` (everywhere), `canvas` (focus on the picture) and `timeline` (focus on the timeline). Text fields keep their own keys.

### Global

| Shortcut | Command | Shortcut | Command |
|---|---|---|---|
| `Space` | play / pause (`playback.toggle`) | `Alt+B` | toggle bones |
| `Home` / `End` | first / last frame | `Alt+G` | toggle whole groups |
| `,` / `.` | previous / next frame | `Alt+K` | toggle auto-key |
| `Shift+,` / `Shift+.` | back / forward 10 frames | `Alt+M` | toggle motion trails |
| `[` / `]` | set range in / out | `Alt+N` | toggle onion skin |
| `Alt+R` | toggle play range | `Alt+O` | toggle loop playback |
| `Alt+1` | toggle Layers panel | `Alt+P` | toggle control points |
| `Alt+2` | toggle Selection panel | `Alt+Q` | toggle scrub proxies |
| `Alt+3` | toggle Bones panel | `Alt+S` | toggle canvas snap |
| `Alt+4` | toggle Live input panel | `Alt+T` | toggle Timeline panel |
| `Alt+5` | toggle Webcam face panel | `Alt+V` | toggle dope sheet / graph |
| `Alt+6` | toggle JSON panel | `Alt+Shift+S` | toggle timeline snap |
| `Alt+7` | Overlays panel | `Alt+=` / `Alt+-` | timeline zoom in / out |
| `Alt+8` | Suggest panel | `Ctrl+S` | save |
| `Ctrl+K` / `Ctrl+Shift+P` | command palette | `Ctrl+,` | keyboard shortcuts |
| `Ctrl+Z` | undo | `Ctrl+Y` / `Ctrl+Shift+Z` | redo |

### Canvas

| Shortcut | Command |
|---|---|
| `Arrow keys` | nudge the selection 1 unit (`Shift`: 10 units) |
| `Delete` / `Backspace` | delete the selection |
| `Escape` | clear the selection |
| `Ctrl+A` | select all |
| `Alt+L` | lock / unlock the selected layers |
| `I` | add a key at the playhead (`anim.add_key`) |

### Timeline

| Shortcut | Command |
|---|---|
| `ArrowLeft` / `ArrowRight` | previous / next frame (`Shift`: 10) |
| `-` / `=` | zoom out / in |
| `F` | zoom to fit |
| `Delete` / `Backspace` | delete selected keys |
| `Escape` | deselect keys |
| `Ctrl+A` | select all keys |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | copy / paste / duplicate keys |
| `Alt+ArrowLeft` / `Alt+ArrowRight` | move selected keys one frame earlier / later |
| `Alt+ArrowUp` / `Alt+ArrowDown` | raise / lower the selected keys (their value) |
| `I` | add a key at the playhead |

Modifiers on the canvas: **Shift** constrains (axis lock when moving, 15-degree steps when rotating, proportional when scaling, a bone tip then only rotates,
and Shift-click toggles selection while a box adds to it); **Alt** scales from the centre (and Alt-drag on empty space draws a lasso); **Ctrl** (Cmd on macOS)
switches snapping off.

## 5. Selection and gizmos

A click picks the topmost thing under the pointer: a layer, a bone (its capsule, a little wider than the drawn line) or, with `points` on, a control point. Locked and hidden
layers are skipped, so a click passes through to what lies below. A box selects by the mode in the `box` selector: `touches` (anything the box overlaps), `contains` (only what lies completely inside) or `bounds touch` (the bounding boxes overlap); Alt-drag draws a lasso instead.

For one layer the transform box follows the layer's own axes: 8 handles scale (edges one axis, corners both; dragging across the opposite edge flips with a negative scale),
the round knob rotates around the centre, and dragging inside moves. For a bone, the head circle moves the bone and the tip rotates it (and stretches it unless Shift is held).
For control points (anchors, bezier handles, deformer points, mesh vertices) the same gizmo moves the selected points. Snapping covers a grid (off by default), other layers' edges
and centres and the canvas edges, within 6 canvas units. **One drag is one undo step** (one batch operation).

## 6. Auto-key: exactly what gets keyed

Whether a canvas edit changes a *static* value or makes *keys* is decided like this. I checked it with the HTTP API on the demo project (the `mode` of each edit is reported):

| Situation | Result |
|---|---|
| Layer edit, `auto-key` off, the property has no keyed channel | **static** change of the layer's transform (no channel is created) |
| Layer edit, `auto-key` on | the edited properties are **keyed** at the current frame (new `layer_x` and `layer_y` channels were created: 14 to 16 channels) |
| Layer or point property that **already has a keyed channel**, `auto-key` off | still **keyed** at the current frame (an edit at frame 9 added a second key to the channels from frame 5) |
| **Bone** edit | always **keyed**, whatever the `auto-key` setting (bone offset channels were created: 16 to 18) |

So an animated property can never be silently overwritten by a static value, and a bone pose is never "lost" by forgetting the toggle. Turn `auto-key` on (`Alt+K`)
when you want to animate a property that is still static. The `I` key (`anim.add_key`, "Add key at playhead") adds a key at the current frame.

## 7. Undo and redo

Undo and redo (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) work on whole operations: a gizmo drag, a nudge, a key edit and an Apply are each one step. With no shared session they
are linear. **In a shared session undo is per user**: it reverses only your own last edit and leaves other people's edits in place (section 8). Undo history is not saved with the
project and is lost when the server stops.

## 8. Shared editing (collaboration)

### Walkthrough: two people, one project

![Two browser users in one session: the toolbar shows two participant chips, Ana (you) at frame 14 and Bo at frame 0, and Bo's selection box and cursor are drawn on the canvas around the leg layer.](img/studio-collab.png)

1. Start the studio. For people on other machines you must add a token and bind to a network address:
   `osatura studio demo.json --bind 0.0.0.0 --port 8080 --collab-token $(openssl rand -hex 16) --collab-log session.log`,
   preferably behind a TLS proxy (section 2). On one machine no token is needed.
2. In the first browser, type your name next to **Collab** ("Ana") and press **Collab**. The button becomes **Leave** and a chip appears for you. In the second browser
   type "Bo" and join. Each chip shows the participant's current frame (`Ana (you) @14`, `Bo @0`).
3. Select things. The other person sees your selection box and cursor in your colour with your name on them (Bo selected `leg` and moved the pointer; Ana sees a purple box
   named "Bo" and a dot next to `leg` in the layer list).
4. Edit at the same time. In the test: Ana nudged the arm layer 10 units (`Shift+ArrowRight`); Bo saw `[10,0]` appear in `/api/project`. Bo then hid the `sun` layer, which Ana saw
   immediately.
5. Ana pressed `Ctrl+Z`. **Only Ana's nudge was undone** (the arm returned to `[0,0]`); Bo's hidden sun stayed hidden.
6. Edits made while someone was working from an older view are merged, not rejected: I ran the project's own simulation (`osatura collab-sim`, 6 replicas, 600 random concurrent
   edits, 12 other delivery orders) and all replicas ended identical and the merged project compiled.
7. Press **Save** (or `Ctrl+S`) in either browser to write the shared state to the project file; nothing is written to the file before that.

Only Chromium was tested, with two pages on one machine. I did not test different machines, slow links or more than two real browsers.

### Persistence: `--collab-log`

```
$ osatura studio demo.json --collab-log session.log
...
$ # (an edit is made in a session, the server is stopped and started again)
collaboration log session.log: resumed at index 1 (1 envelopes after the checkpoint)
```

The log is a line file (mode `0600`, owner only) that holds a hash-chained checkpoint of **the whole project** (about 178 KB for the demo) plus one line per accepted edit. Appends are
`fsync`ed per request unless `--collab-nosync`. After a restart the session state is resumed (I changed the project name in a session; after the restart `/api/project` still had the
new name) while the file on disk was unchanged (`OSATURA demo`) until Save. **Participants and undo history are not persisted**: pages join again with new ids. A damaged log or a log
of another project stops the server with an error instead of being overwritten. A crash during an append loses at most one cut-off last line.

**The log contains your whole project in clear text; protect it like the project itself.**

## 9. HTTP API reference

The studio's page uses these routes and you may too (scripts, tests). All bodies and responses are JSON unless stated; errors are `{"ok": false, "errors": ["..."]}` with status 400 (or 401, 403,
404, 415, 421, 429, 503 as described in section 2). Success responses carry `"ok": true` where noted. `state` below means the document returned by `GET /api/state`:
`{bones:[{name,rest_angle}], channels:[...], fps, frames, width, height, can_undo, can_redo, rev, ok}`; `rev` increases on every change. Every route is subject to the checks of section 2.

### Reading

| Route | Parameters | Response |
|---|---|---|
| `GET /` | | the studio page (HTML) |
| `GET /source` | | the AGPL source-offer page; open without a token |
| `GET /api/state` | | `state` |
| `GET /api/project` | | the whole project JSON |
| `GET /api/layers` | | `{ok, layers:[{path, depth, kind, visible, locked, locked_eff, reference, ptr}]}` (`locked_eff` also counts locked groups above) |
| `GET /frame/N.svg` | `bones=1` draws the skeleton | one frame as SVG (N may be fractional); headers `X-Cache`, `X-Render-Ms` |
| `GET /api/frame` | `frame`, `proxy` 0-4 (0 = full quality, 1-4 scrub proxies), `kind` = `vector` (default) \| `raster` \| `png`, `bones=1`, `lod=1&screen=WxH&lodpx=P` (draw from a reduced-detail copy within P pixels, 0-16) | the picture; headers `X-Cache` (hit/miss), `X-Render-Ms`, `X-Lod-Level`, `X-Lod-Bound-Px`, `X-Proxy-Level`. An out-of-range frame: `{"errors":["frame 999 out of range"]}`; a bad kind: `bad kind 'x' (vector, raster, png)` |
| `GET /api/cache` | | `{ok, revision, entries, bytes, hits, misses, invalidations, lod:{builds, engines, invalidations, last_build_ms, levels, unavailable}}` |
| `GET /api/overlays` | `frame`, `sel` (comma-separated layer names) | `{ok, frames, settings, ...}` plus ghosts and SVG for the frame when one is given |
| `GET /api/sounds` | | `{ok, total, sounds:[{name, start, gain, duration, peaks, error?}]}` waveform peaks after trim, fades and gain |
| `GET /api/vcs/log` | `limit` (max 1000), `rev` | `{ok, branch, commits:[{hash, author, time, message, parents}]}`; `{"ok":false,"errors":["no .osatura-vcs repository found ..."]}` without a repository (see `osatura vcs`) |
| `GET /api/vcs/diff` | `a` (default `HEAD`), `b` | `{ok, changes}` |
| `GET /api/timeline` | | `{ok, fps, frames, groups, rows, tracks}` the dope-sheet model |
| `GET /api/graph` | `ch` (comma-separated channel indexes), `f0`, `f1`, `n` (samples, 2-2000, default 200) | `{ok, channels, f0, f1, range, skipped}` curve samples and tangent handles |
| `GET /api/live` | | `{ok, mapping:{params,triggers}, recorded, recording}` |
| `GET /api/ui` | | `{ok, layout, presets, preset_names, default_layout, shortcuts, commands, conflicts, reserved, overlays, recents, panels, persisted, path, warning, ...}` |
| `GET /api/ui/search` | `q`, `limit` (1-200, default 50) | `{hits:[{id, title, category, description, bindings, score, positions}]}` the palette ranking |
| `GET /export.gif`, `/export.html`, `/export.html.nosound`, `/export.lottie.json` | | the file, as a download (`animation.gif`, `player.html`, `animation.json`) |

### Editing

| Route | Body | Response |
|---|---|---|
| `POST /api/op` | `{"op":"set","path":"/fps","value":30}` — `op` is `set`, `insert` (array path, `.../-` appends), `remove` or `batch` (`{"op":"batch","ops":[...]}`, one undo step). Paths are JSON Pointers into the project. | `state`. An invalid result is refused with `{"errors":["invalid project: invalid value: integer `-3`, expected u32"]}` and nothing changes |
| `POST /api/undo`, `POST /api/redo` | none | `state` (per user in a shared session) |
| `POST /api/key` | `{"target","prop","frame","value","ease"?}` (`ease` like `{"type":"smooth"}`; default linear) | `state` |
| `POST /api/delete_key` | `{"target","prop","frame"}` | `state` |
| `POST /api/save` | none | `{"ok":true,"saved":"<path>"}`; writes atomically. Host only when the studio runs with `--collab-host` (otherwise **403**) |
| `POST /api/suggest` | `{"kind":"between"\|"complete"\|"next","a","b","t","frame","constraints","free","top","snap","prior","apply":RANK, "library"?, "model"?}` | `{ok, suggestions}`; with `apply` also `state` of the edit. A request without `kind` gives `bad request: missing field `kind`` |

### Selection and gizmos (stateless: the page holds the selection)

A **target** is `{"kind":"layer","path":"arm"}`, `{"kind":"bone","name":"hand"}` or `{"kind":"point","layer":"...","point":...}`.

| Route | Body | Response |
|---|---|---|
| `POST /api/pick` | `{"frame","x","y","options"?}` (`options`: `tolerance`, `bone_radius`, `point_radius`, `layers`, `bones`, `point_layers`, `locked`, `locked_bones`) | `{ok, hit, hits:[{distance, target}]}` topmost first |
| `POST /api/marquee` | `{"frame","rect":[x0,y0,x1,y1]` or `"lasso":[[x,y],...]` (3 to 5000 points)`,"mode":"bbox"\|"contain"\|"intersect","options"?}` | `{ok, targets}` |
| `POST /api/select` | `{"frame","selection":[...],"targets":[...],"mode":"replace"\|"add"\|"toggle"\|"subtract"}` | `{ok, selection, bounds, gizmo:{corners, handles}, points}` |
| `POST /api/gizmo` | `{"frame","selection","handle","start":[x,y],"delta":[dx,dy],"modifiers":{"constrain","center","no_snap"},"snap":{"grid","objects","canvas","tolerance"},"auto_key","locked","pivot","commit":bool}` — `handle` is `{"type":"move"}`, `rotate`, `{"type":"scale","at":"nw"}`, `bone_head` or `bone_tip` | without `commit`: `{ok, drag:{changed, edits, guides, matrix, summary, box_after, outline}}` (a preview); with `commit:true`: `state` plus `drag` and `selection`, one undo step |
| `POST /api/nudge` | `{"frame","selection","delta":[dx,dy],"auto_key","locked"}` | `state` + `drag` (edits with `mode` of `static` or `key`) + `selection` |
| `POST /api/delete` | `{"frame","selection"}` | `state` + `selection`; `{"errors":["nothing selected"]}` when empty |

### Timeline

| Route | Body | Response |
|---|---|---|
| `POST /api/timeline/select` | `{"selection":[{ch,frame}...],"mode":"replace"\|"add"\|"toggle"\|"subtract","all":true` or `"channel":N` or `"group":N` or `"box":{f0,f1,row0,row1}`, `"include_locked"}` | `{ok, selection:[{ch,frame}...]}` |
| `POST /api/timeline/op` | `{"op":{...},"preview":bool}` — `op` is one of `move_keys`, `scale_keys`, `copy`, `paste`, `duplicate`, `delete`, `set_ease`, `set_value`, `add_key`, `ripple`, `offset`, `extrapolate`, `make_loop`, `handle`, `smooth`, `flags` (an unknown name lists these) | with `preview:true`: `{ok, preview:true, plan}` without changing anything; otherwise `state` + `plan:{changed, notes, op, clip}` |

For instance `{"preview":true,"op":{"op":"delete","keys":[{"ch":0,"frame":12}]}}` returns the plan (`changed: 1`) and leaves the project alone.

### Overlays, live, face, UI

| Route | Body | Response |
|---|---|---|
| `POST /api/overlays` | `{"settings":{"onion":{"enabled":true,"before":3}},"frame","selection":[names],"reset":bool}` | `{ok, frames, ghosts, settings, svg}`; settings are saved in the UI config |
| `POST /api/trails` | `{"targets":[{"kind":"bone","name":"hand"}],"from","to"` or `"frame","before","after"`, `"smooth","svg":bool,"hit":{x,y,radius}}` | `{ok, from, to, trails:[{curve,frames,keys,...}], svg?, hit?, path_hit?}` |
| `POST /api/live/mapping` | `{"mapping":{params,triggers},"library"?}` (same format as `live-map`) | live status |
| `POST /api/live/push` | `{"events":[{source,id,value,time}],"midi":[raw...]}` | live status with the events' effect |
| `POST /api/live/record` | `{"on":true}` starts; `{"on":false,"tol":0.01}` stops and **bakes the performance into the project** as one undo step | status / `state` + `live` summary |
| `POST /api/face/config` | `{"config"?,"calibration"?}` | `{ok}` |
| `POST /api/face/calibrate` | `{"on":true}` / `{"on":false}` | `{ok, frames, calibration}` |
| `POST /api/face/push` | `{"frames":[...landmarks...],"layout"?,"size"?}` (at most 10000 frames per push) | live status |
| `POST /api/face/detect` | `{"image":"<base64 PNG/JPEG or data: URI>","track":bool,"push":bool,"t"}` | `{ok, found, layout:"osatura23", size, points, box, note:"detector trained on synthetic faces only"}` |
| `POST /api/face/synth` | `{"t","size":32-512,"seed"}` | `{ok, image:"data:image/png;base64,...", layout, size, points}` one frame of the synthetic head with its exact landmarks |
| `POST /api/ui` | any of `{"layout":{...}\|null,"shortcuts":{...}\|null,"recents":[...],"overlays":{...},"reset":"layout"\|"shortcuts"\|"overlays"\|"all"}` | `GET /api/ui` document; unknown fields (`unknown field 'bogus'`), conflicts and incomplete layouts are refused and nothing changes |
| `POST /api/login` | `{"token":"..."}` | `{ok:true}` and `Set-Cookie: osatura_token=...; Path=/; HttpOnly; SameSite=Strict`; wrong: 401 (429 from the 5th failure); without a configured token `{"ok":true,"token_required":false}` |

### Collaboration

Calls made as a participant send `X-Collab-Replica: <id>` and `X-Collab-Secret: <secret>` (both returned by join). A replica id with the wrong or missing secret gets **403**.

| Route | Body / query | Response |
|---|---|---|
| `POST /api/collab/join` | `{"name":"Ana","mode":"thin"\|"replica"}` | `{ok, replica, secret, colour, name, index, mode, clock, users, presence, doc}` (`snapshot` in replica mode). The first join starts the session from the editor's document |
| `POST /api/collab/leave` | `{"replica"}` | `{ok, left}` |
| `GET /api/collab/state` | `replica`, `doc=1`, `snapshot=1` | `state` + `collab_active` (+ users, presence) |
| `GET /api/collab/log` | `since=N` | `{ok, index, entries:[{index, by, envelope}]}` |
| `POST /api/collab/ops` | `{"replica","ops":[...]` or `"op":{...}`, `"base"?}` or `{"replica","envelopes":[...]}` | `state` + `applied`, `index`. Unknown participant: `unknown participant N: join first`; empty list: `no ops`; invalid project edits are refused |
| `POST /api/collab/undo`, `.../redo` | `{"replica"}` | `state` + `collab_can_undo`, `collab_can_redo` (your own edits only) |
| `POST /api/collab/presence` | `{"replica","n","frame","cursor":[x,y],"selection":[names up to 64]}` | `{ok}` |
| `POST /api/collab/compact` | `{}` | compaction summary (`ok`, `logged`). Host only when the studio runs with `--collab-host` (otherwise **403**) |
| `POST /api/collab/revoke` | `{"names":["ana","bob"]}` (1 to 64 names) | `{ok, revoked, revoked_total}`: the tokens of those names are refused from now on and participants who joined under them are removed. Needs per-participant tokens (`--collab-secret-file`) and a host token (`--collab-host`), or the shared `--collab-token`; otherwise **403**. An event stream that is already open ends when its client disconnects |
| `GET /api/collab/events` | `since=N` or `Last-Event-ID` | a **server-sent-events** stream: replay from `since`, then users, presence and every later event; `resync` if the client is too far behind; at most 64 streams (`503 too many event streams`) |

In an active session the ordinary editing routes (`/api/op`, `/api/undo`, `/api/redo`, and the studio's own edits through nudge, key or gizmo) are recorded for the other participants too.

## 10. Troubleshooting

| Symptom | Cause and fix |
|---|---|
| The browser shows 421 or "misdirected request" | You opened the studio under a name it does not know (a DNS name, a LAN address). Add it with `--allow-host name` or open `http://127.0.0.1:PORT/`. |
| 403 on every request behind a proxy | The proxy changes `Origin`/`Host`. Give the public origin: `--public-origin https://studio.example.org`. |
| 401 | `--collab-token` is set: open the page and enter the token once, or send `X-Collab-Token`. A token in the URL is not accepted. |
| 429 | Five wrong tokens in a row from your address; wait up to 30 s. Behind a proxy all users share one address. |
| `refusing to listen on 0.0.0.0:...` | Non-loopback bind without a token. Add `--collab-token` (16 or more characters). |
| `cannot listen on 127.0.0.1:8080: Address already in use` | Another process owns the port; choose `--port`. A stale studio from an earlier run is the usual cause. |
| The page loads but the picture is blank | Check the terminal; a project that fails `check` is refused at start. Reload after fixing. |
| A browser console message `Failed to load resource ... 404` | Older builds had no `/favicon.ico`; it is served now. A remaining 404 comes from something else. |
| The Face panel text | The panel now says the synthetic-trained detector is included, is not validated on photos or webcams, and a real webcam needs your own landmark provider. |
| Layout is broken or panels are missing | `Layout` menu, reset layout, or delete the `ui.json` shown at startup. A config file that cannot be read completely is copied to `ui.json.bak` (then `.bak.1`, ...) before defaults are used, and `GET /api/ui` has a `warning` field about it. |
| My edits vanished after a restart | Nothing is written until **Save**. With `--collab-log` the *session* survives a restart, but the file on disk is still only written by Save. |
| `Ctrl+W`, `Ctrl+T` and friends cannot be bound | Reserved by browsers (list in section 3). |
| Ctrl+Z does not undo someone's change in a session | By design: undo is per user. |
| Exports ask for `ffmpeg` | Only for sound in HTML (mp3) and for mp4/webm on the command line; GIF, Lottie and HTML without sound need nothing. |

## 11. Accessibility and browser support

**Browsers.** The interface was tested only in **Chromium** (Playwright's bundled build, driven headless, on Linux). Firefox and Safari were not tried and may work or may not.
Web MIDI needs a browser that implements it (Chromium does); the camera needs `getUserMedia` and a secure context (HTTPS or `localhost`). The page uses server-sent events for the
shared session. Pointer events are used for dragging; touch screens were not tested.

**Accessibility, honestly.** I did not audit the studio with a screen reader or a keyboard-only session. What exists: a `lang` attribute and viewport meta tag, labelled checkboxes, a
`role="status"` toast, a few ARIA attributes, **a complete set of keyboard shortcuts and a command palette** that reach every command, rebindable shortcuts, and the `osatura` command line, which does
everything the editor does without a pointer. What is missing: the canvas is a picture and most canvas editing is by pointer (arrow-key nudging exists), there is no light theme and no
`prefers-reduced-motion` or `prefers-color-scheme` handling, focus outlines are the browser defaults and I did not check them, and I did not measure colour contrast. If you depend on assistive technology,
prefer the CLI and the JSON panel, and tell the project what breaks.

## 12. Known rough edges

* Fixed since the first write-up: the Webcam face panel text, the `/favicon.ico` 404, `--collab-nosync` without `--collab-log`, and the missing `Secure` cookie flag behind an https `--public-origin`.
* `osatura render p.json 99 o.png` on a 24-frame project is now an error too (add `--clamp` to render the last frame); the studio's `/api/frame` refuses an out-of-range frame (`frame 999 out of range`) and now accepts frames `0..frames-1` only.
* Layout and shortcut changes are stored per user in the UI config, not in the project, so they do not travel with the file.
