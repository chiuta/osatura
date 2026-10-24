// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! The `osatura --help` text. Every line must appear verbatim in docs/CLI_REFERENCE.md (a test checks it).

pub(crate) const USAGE: &str = "\
osatura — 2D rigging & animation tools (independently implemented; see CLEANROOM.md)

GLOBAL OPTIONS (accepted anywhere on the line, also after a sub-command; `--key=value` works for every value option):
  --allow-external-assets     let a project name files by absolute path or outside its own directory (default: refused)
  --follow-symlinks           write to the target of a symbolic link given as an output file (default: refused)
  --max-input-bytes N         largest project file that is read (default 268435456); devices and pipes are never read
  <command> --help            the usage of one command;  <command> --version  the version

USAGE:
  osatura --version   print the version, licence (AGPL-3.0-or-later), source offer and support links
  osatura demo     <out.json>                         write the built-in rig demo
  osatura demo-person <out.json>                      write the walking-person demo (head, ears, hands with fingers, feet)
  osatura demo-fx  <out.json>                         write the liquid / boolean / particles demo
  osatura check    <project.json>                     validate a project
  osatura render   <project.json> <frame> <out.svg|out.png> [--bones] [--scale N] [--clamp] [--proxy N]
                    --proxy N: scrub proxy (N = 1..4): .png = bitmap at 1/2^N resolution, .svg = simplified vectors; both drop fine strokes, gradients, blends
  osatura frames   <project.json> <out_dir> [--png] [--bones] [--scale N]
  osatura export   <project.json> <out.gif> [--scale N] [--report r.json]   looping GIF; limits are warned on stderr and listed in --report
  osatura export   <project.json> <out.html|out.htm>   one self-contained HTML player (no options)
  osatura export   <project.json> <out.json|out.lottie> [--report r.json]   .json = Lottie, .lottie = dotLottie; limits are warned on stderr and listed in --report
  osatura export   <project.json> <out.mp4> [--scale N]   H.264, needs ffmpeg on PATH
  osatura lipsync  <project.json> <audio.wav> <switch_layer> <out.json> [--map 0,1,2,3,4]   default map follows the switch's child count; shapes rest,small,open,round,wide: 1 child all, 2 closed+open, 3 closed+open+round/wide, 4 closed+small+open+round/wide, 5+ in that order
  osatura walk     <project.json> <out.json> --thigh-l B --thigh-r B [--shin-l B --shin-r B
                    --arm-l B --arm-r B --hips B --period N --stride DEG --knee DEG --bob PX --swing DEG]
  osatura mirror   <project.json> <out.json>          mirror all keyed channels left<->right
  osatura studio   <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES] [--public-origin URL] [--source-url URL] [--collab-token T (16+ characters)] [--collab-secret-file F [--collab-revoke NAMES]] [--collab-rate N] [--collab-log LOG [--collab-nosync]]   browser editor with undo/redo (UI layout and shortcuts are kept in F, default ~/.config/osatura/ui.json); LOG persists the shared session and resumes it after a restart
  osatura collab-token <name> [--secret-file F] [--ttl 28800]   mint a per-participant token for a studio started with --collab-secret-file F (HMAC-SHA-256; bound to the name; default life 8 hours, at most 30 days; without --secret-file the secret comes from the environment variable OSATURA_COLLAB_SECRET)
  osatura collab-sim <project.json> [--clients N] [--ops M] [--seed S] [--undo-pct P] [--replays K] [--require-valid] [--raw] [--json]   N replicas edit concurrently with random delivery order and duplicates, then verify they all hold the same document (exit 1 if not). default edits are valid project edits (numbers, key values, visibility, own layers) so ok can be true; --raw uses arbitrary JSON edits instead (usually not a valid project); JSON ok also needs the merged document to compile; converged_ok is convergence alone; --require-valid makes a non-compiling result exit 1 too
  osatura fuzz     <project.json> [--samples N] [--seed S] [--strategy random|lhs|halton|sweep|corners|pca] [--frame F] [--report out.json] [--heatmap out.svg] [--minimize] [--gate] [--max-fail-rate R] [--props angle,scale,offx,offy] [--only BONES] [--margin DEG] [--db posedb.json] [--ignore KINDS] [--no-deformers]   find poses where the rig breaks (folds, stretch, spikes, self-intersection, IK unreachable, limits, NaN, off-canvas), stress heat-map, minimal reproducer; a report (exit 0) unless --gate (ceiling 0) or --max-fail-rate R (fraction 0..1) is given, then exit 1 above the failure rate
  osatura fuzz-grid <project.json> --x PARAM --y PARAM [--res N] out.svg|out.png|out.json   severity map of two parameters, e.g. bone:arm_up:angle
  osatura vcs      init <project.json> [--author A]   version control with semantic merge for a project file (see: osatura vcs)
  osatura vcs      commit [-m MSG | --message MSG] [--author A] [--repo DIR]
  osatura vcs      log [REV] [--limit N] [--json] [--repo DIR]
  osatura vcs      status [--json] [--repo DIR]
  osatura vcs      diff [A [B]] [--json] [--repo DIR]
  osatura vcs      show [REV] [--json] [--repo DIR]
  osatura vcs      branch [NAME [AT] | -d NAME] [--repo DIR]
  osatura vcs      tag [NAME [AT]] [--repo DIR]
  osatura vcs      checkout REV [--force] [--repo DIR]
  osatura vcs      merge REV [--no-ff] [-m MSG | --message MSG] [--author A] [--abort] [--repo DIR]
  osatura vcs      cherry-pick|revert REV [--author A] [--repo DIR]
  osatura vcs      resolve PATH [--ours] [--theirs] [--value V] [--repo DIR]   exactly one of the three
  osatura vcs      fsck|gc [--repo DIR]
  osatura ui       commands|shortcuts [--ui-config F] [--json]    studio command registry / effective shortcuts and conflicts
  osatura ui       layout presets|show <name>|validate <file.json>   dock layout presets (animate, rig, compact, live)
  osatura ui       search <query...> [--ui-config F]              fuzzy command search as the palette ranks it

OPEN FORMATS (open specifications; no H.264/H.265 unless you ask for .mp4; codec patent status is not assessed, see docs/PATENT_NOTES.md)
  osatura export   <project.json> <out.apng> [--scale N]   animated PNG
  osatura export   <project.json> <out.webm> [--scale N] [--codec vp9|av1]   VP9 with alpha (--codec av1 for AV1) via ffmpeg
  osatura export   <project.json> <out.png> [--scale N] [--columns N]   sprite sheet + <out>.atlas.json (layout; never <out>.json)
  osatura export   <project.json> <out.glb> [--report r.json]   glTF 2.0 (bones, animation, skinned fills/strokes, gradients, masks, images, morph targets)
  osatura export-oca <project.json> <out_dir> [--layers]   Open Cel Animation folder
  osatura import-ora <file.ora> <out.json> [--fps N] [--frames N]   OpenRaster -> project

SAFETY
  osatura safety   <project.json> [--scale N]         flash / red-flash / camera-motion report (WCAG 2.3.1 approximation)
  osatura reduced-motion <project.json> <out.json> [--factor 0.25]   calmer copy (camera + wobble scaled)

INSPECT / QUALITY
  osatura stats    <project.json>                     counts of bones, layers, channels, keys (always printed as JSON)
  osatura diff     <a.json> <b.json>                  semantic differences (+ added, - removed, ~ changed)
  osatura golden   write <project.json> <golden.json> [--scale N]   record hash + thumbnail of every frame; N in (0, 1] (larger values are refused)
  osatura golden   check <project.json> <golden.json> [--tol T]   regression frames: exact by default (any pixel change fails, exit 1); --tol T accepts a mean thumbnail difference up to T (0..1); the scale stored in the golden is used (a scale option is refused here)
  osatura export-fit <project.json> <out> --max-size 500KB [--format lottie|html|gif|apng|webm|project] [--min-ssim X] [--max-seconds 60] [--measure raw|gzip] [--frames N] [--search ladder|descent|grid] [--report out.json] [--pareto out.csv] [--audit] [--no-baseline]   best-quality export that fits a byte budget (--max-seconds is a real wall-clock budget; SSIM against the original render; exit 1 and the smallest export if the budget cannot be met)
  osatura size-report <project.json> [--tol T] [--json]   which layers, images and channels contribute bytes (project file, baked Lottie, HTML page)
  osatura lod      <project.json> --tolerance-px 0.5 [--scale S | --size WxH] [--levels 4] [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]   level of detail L1..Ln with a declared error bound in output pixels (path simplification, key reduction, mesh collapse, culling); --verify measures the real deviation (exit 1 if above the bound)
  osatura lod-pick <project.json> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--out chosen.json]   coarsest level whose declared bound stays within the pixel budget on that screen
  osatura ci       init <project.json> [--out osatura-ci.json] [--no-goldens] | run [manifest] [--update] [--bless] [--prune] [--only a,b] [--junit f.xml] [--html f.html] [--json f.json] [--artifacts dir] [--quiet] | bless [manifest] | diff a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--ignore x,y,w,h;...] [--aa N] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]   animation CI: golden frames with perceptual diff (SSIM, CIEDE2000), structural checks, budgets; exit 0 pass, 1 fail, 2 error
  osatura lint     <project.json>                     static rig / animation checks (exit 1 on errors)
  osatura contracts <project.json> <contracts.json>   motion contracts (unit tests for animation)
  osatura loop-solve <project.json> <out.json> [--window 24] [--inclusive]   make keyed channels loop seamlessly (period N frames, no duplicated seam frame; --inclusive = last frame equals first, legacy)
  osatura coach    <project.json>                     timing / principles suggestions
  osatura timecode <frame|HH:MM:SS;FF> [--fps 24] [--drop] [--nodrop]   SMPTE timecode (29.97/59.94 = drop-frame with ';' by default; --nodrop counts 30; a timecode argument prints its frame)
  osatura mcp      <project.json>                     agent server (JSON-RPC over stdin/stdout; tools to inspect, edit, lint, render, save)

RIG / PROJECT ORGANISATION
  osatura tag      <project.json>                     propose standard roles (arm.upper.l, face.mouth…) for bones and layers
  osatura autorig <project.json> <out.json> [seed]   IK (mix 0), breathing, blinking from bone/layer names
  osatura slider <project.json> <out.json> <name> <poses>   poses: at:target.prop=val,target.prop=val;at:...
  osatura view-swap <project.json> <out.json> <switch_layer> <bone.prop> <from:child,from:child,...>
  osatura align <project.json> <out.json> <mode> <layer,layer,...|layer> [stage]   mode: left right top bottom center-x center-y
  osatura distribute <project.json> <out.json> <x|y> <gaps|centers> <layer,layer,...>
  osatura trim <project.json> <out.json> <layer> <path_index> <start> <end> [offset]   bakes the trimmed part
  osatura shot <project.json> <out.json> <name> <x> <y> <zoom> [rotation]   save a camera shot
  osatura camera-seq <project.json> <out.json> <frame:shot[:cut],...>   keys the camera through saved shots
  osatura sound list <project.json> | add <project.json> <out.json> <name> <file.wav|ogg|mp3|...> [start_frame] [gain] [trim/fade options] | edit <project.json> <out.json> <name> [--start F] [--gain G] [trim/fade options] | remove <project.json> <out.json> <name>
                    trim/fade options (seconds, or frames with an f suffix such as 36f): --trim-in T --trim-out T --fade-in T --fade-out T (trim points are inside the audio file; edit: 0 clears)
  osatura sound mix <project.json> <out.wav|out.ogg|out.mp3>   only the mixed audio (start, gain, trim, fades; project length), through ffmpeg
                    audio is mixed into .mp4 (AAC) and .webm (Opus) and embedded in .html (needs ffmpeg); .gif and .apng cannot carry audio and stay silent
  osatura make-loop <project.json> <out.json> [targets|all] [props] [--inclusive]   (removes travel = in-place; period N frames so playing 0..N-1 repeatedly has no duplicate seam frame; --inclusive = frame N-1 equals frame 0, legacy)
  osatura follow <project.json> <out.json> <layer|bone:NAME> <layer_with_path> <start_frame> <end_frame> [no-orient]
  osatura take-record <project.json> <library.json> <name> <from> <to> [target_prefix,...]   library is created if missing
  osatura take-apply <project.json> <out.json> <library.json> <name> <at> [from=to,...] [speed] [--blend FRAMES]   blend cross-fades with existing animation at both ends
  osatura take-list <library.json>
  osatura clip add <project.json> <out.json> <library.json> <take> <clip> [--at F --speed S --loops N --mode replace|additive|multiply --weight W --include a_*,b --exclude c --track-weight arm_*=1,leg_*=0.2 --rename a=b --fade-in F --fade-out F]   layer a take on the timeline
  osatura clip list <project.json>                    the clip stack (later clips layer over earlier ones)
  osatura clip remove <project.json> <out.json> <clip>
  osatura clip set-weight <project.json> <out.json> <clip> [W] [--keys frame:w,frame:w] [--track PATTERN[:prop]]   constant, keyed, or per-track weight
  osatura clip bake <project.json> <out.json>         regenerate the stack-owned channels (hand-made and locked channels are never touched)
  osatura machine add <project.json> <out.json> <machine.json>   add a runtime state machine (inputs, states, transitions, blend trees, layers); validated
  osatura machine list <project.json>  |  machine remove <project.json> <out.json> <name>  |  machine validate <project.json>
  osatura machine-sim <project.json> <script.json> <out_dir|out.json> [--fps N --svg-frames N]   replay timed input events: state log (sim.json) + SVG frames
  osatura container add <project.json> <out.json> <container.json>   add a layout container (row/column, gap, padding, justify, align, wrap, grow/shrink, nested, canvas-relative rect); validated
  osatura container list <project.json>  |  container remove <project.json> <out.json> <id>  |  container solve <project.json> [--size WxH --data out.json]   print solved rectangles and layer offsets
  osatura scroll add <project.json> <out.json> <scroll.json>  |  scroll list <project.json>  |  scroll remove <project.json> <out.json> <id>   scroll regions (viewport, content container, clamp, elastic, snap, clip layer)
  osatura scroll eval <project.json> <id> [--offset X,Y --size WxH --velocity VX,VY --time S]   clamped/elastic offset; with --velocity simulate the release (momentum, snap, spring)
  osatura layout-preview <project.json> <out.svg> --size WxH [--frame N --scroll id=X,Y]   render with layout re-solved for another canvas size, container outlines drawn
  osatura bind add <project.json> <out.json> <name> <value.path> <target:prop|@machine:input> [--expr E --remap a,b,c,d --no-clamp --smooth S --mode replace|add|multiply --weight W]   data binding
  osatura bind list <project.json>  |  bind remove <project.json> <out.json> <name>  |  bind-eval <project.json> <values.json> [--data out.json]   evaluate bindings for given values
  osatura foot-lock <project.json> <out.json> <ctrl_bone> <ground_y> [tolerance] [min_frames]   pins a root controller during ground contact
  osatura stretch <project.json> <out.json> <follower_bone> <leader_bone> [min=1] [max=3]   follower aims at the leader's head and stretches to reach it (baked, uniform scale)
  osatura magnet <project.json> <out.json> <ctrl_bone> <bone:NAME|x,y> <radius> <strength 0..1>   bakes a proximity pull
  osatura cage <project.json> <out.json> <layer> [rect=x,y,w,h|auto] [hull] [verts=N] [pad=P] [name=cage]   adds a mean-value-coordinate cage deformer around a layer (auto = padded bounds, verts = segments per side; hull = convex hull); content inside follows the cage, outside is unchanged
  osatura cage-pose <project.json> <out.json> <layer> <frame> [name=cage] [move=i:dx,dy;...] [all=dx,dy] [ease=linear]   keys cage vertex offsets at a frame (cage_dx/cage_dy channels; locked channels are refused)
  osatura warp-pivot <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=1,4-7] [region=x,y,w,h] [pivot=x,y|bone:NAME|centroid|center] [rotate=DEG] [scale=S|sx,sy] [move=dx,dy] [radius=R] [falloff=smooth] [ease=linear]   rotates/scales/moves a selection of control points (cage, lattice, cloth, pin, spine; a grid image mesh gets a per-vertex warp grid) about a pivot and keys the result at the frame; radius adds a soft falloff (linear, smooth, gauss, sharp, constant)
  osatura soft-scale <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=..|region=..] scale=S|sx,sy [radius=R] [falloff=smooth] [pivot=..]   proportional (soft-selection) scaling of the selection; without verts/region the whole mesh is resized about its bounding-box centre: scale=F | width=W | height=H | size=WxH, aspect=keep|free
  osatura layer-pivot <project.json> <out.json> <layer> pivot=x,y|bone:NAME   moves the layer pivot (anchor) without changing the picture, so rotation/scale channels turn about it
  osatura mesh-lock add <project.json> <out.json> <name> host=LAYER[:path] follower=LAYER[:path] [verts=1,4-7] [strength=1]   follower vertices follow the host mesh surface (barycentric displacement transfer) every frame; cycles are refused
  osatura mesh-lock list|remove <project.json> [<out.json> <name>]   lists / removes mesh locks
  osatura glue add <project.json> <out.json> <name> a=LAYER[:path] b=LAYER[:path] pairs=i:j;i:j|auto=RADIUS [mode=rigid|snap|offset] [strength=1]   glues b vertices to a vertices (b follows a); rigid keeps the rest offset in the frame of a, snap makes them coincide, offset keeps a constant offset
  osatura glue list|remove <project.json> [<out.json> <name>]   lists / removes glues
  osatura cloth <project.json> <out.json> <layer> [pin=top,left@bone,row:N,col:N,v:N,corners,none] [gravity=900] [wind=x,y] [gust=0.5] [gust-hz=0.5] [gust-len=px] [stiffness=1] [shear=0.05] [bend=0.2] [max-stretch=1.1] [damping=0.99] [cols=N rows=N] [rect=x,y,w,h] [name=cloth] [substeps=4] [iterations=10] [preroll=frames] [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [from=F] [to=F]   bakes a deterministic cloth surface onto an image layer mesh (or a vector layer with rect=) as per-vertex deform channels
  osatura physics <project.json> <out.json> <bone,bone,...> [gravity=900] [wind=x,y] [gust=0.5] [damping=0.985] [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [follow=bone] [--allow-empty] [drive=bone:gain;...] [vortex=bone|x,y:swirl[:radial[:radius]];...] [preset=hair|tail|rope|antenna|cape] [merge=keep|replace] [lateral=0..1]   bakes a chain simulation; options after preset= override it. Several chains: bone,bone/bone,bone with lateral= ties neighbouring strands
  osatura eye-clip <project.json> <out.json>   masks iris/pupil layers by the eye shape on the same side
  osatura parallax <project.json> <out.json> <far_layer,...,near_layer> <far_value> <near_value>   multiplane depth for top-level layers
  osatura chan-meta <project.json> <out.json> <target> <prop|*> [color=#rrggbb] [shy=on|off] [solo=on|off] [lock=on|off]
  osatura reference <project.json> <out.json> <layer> <on|off>   tracing layer: shown in the studio, never exported
  osatura import-seq <project.json> <out.json> <layer_name> <folder> [hold=1] [loop=on|off] [start=0] [x=0] [y=0]   PNG sequence -> frame-by-frame layer
  osatura import-anim <project.json> <out.json> <layer_name> <file.gif|file.apng|file.png> [loop=on|off] [start=0] [x=0] [y=0]   animated GIF/APNG -> frame-by-frame layer with the file's delays
  osatura dof <project.json> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24 --scale N --clamp]   depth of field: each top-level layer is blurred by |parallax - focus|  (focus/strength follow camera_focus/camera_blur channels unless --focus/--strength is given)
  osatura dof-export <project.json> <out.gif|out.apng|out.mp4|out.webm|out-folder> [--from F --to F --focus N --strength N --max-radius N --scale N --codec vp9|av1]   animated depth of field over a frame range (.mp4/.webm through ffmpeg, without audio; --codec applies to .webm); focus/strength follow the camera_focus/camera_blur keys unless pinned; a folder gets a PNG sequence
  osatura render-gpu <project.json> <out.png> [--frame N] [--adapter software|auto] [--scale S] [--bench N]   GPU render (wgpu); only in builds with --features gpu; unsupported scenes (images, bones) are reported, never drawn wrongly
  osatura render-hdr <project.json> <frame> <out.exr|out.png> [linear] [--precision half|float] [--float32] [--8bit] [--scale N] [--clamp]   f32 render: EXR (HALF or FLOAT, linear, premultiplied) or 16-bit PNG (no dither); --8bit uses the legacy 8-bit source
  osatura import-lottie <in.json> <out.json> [fx.json]   subset import; unsupported parts are listed as warnings; blur / drop shadow / tint / fill become per-layer effects inside the project (fx.json is kept for compatibility and is now an empty chain)
  osatura script <project.json> <out.json> <script.rhai>   sandboxed Rhai: fps() frames() bones() layers() set_key(target, prop, frame, value[, ease]) add_bone(name, parent (empty = root), length, x, y, angle) add_rect(name, x, y, w, h, fill) add_circle(name, x, y, r, fill) add_polygon|add_blob(name, [x,y,...], fill) add_line(name, [x,y,...], colour, width, closed) add_gradient_rect(name, x, y, w, h, top, bottom) add_ik(name, [bones], target, bend) add_aim(name, bone, target[, offset, weight]) add_copy_rotation(name, bone, source, weight) add_pin(name, bone, target, weight) add_group(name[, [layers]]) add_switch(name, [layers]) move_layer(path, group_or_empty) set_parent(bone, parent_or_empty) delete_layer(path) delete_bone(name) set_layer(path, prop, value) set_bone(name, prop, value) add_channel_expr(target, prop, expr[, mode]) add_noise(target, prop, amp, freq, seed[, mode]) add_marker(name, frame[, end]) add_event(name, frame[, data]) bone_info(name) layer_info(path) world(bone, frame) print(x)
  osatura audio-react <project.json> <out.json> <audio.wav> <target> <prop> [gain=1] [offset=0] [floor=0.05] [attack=1] [release=4] [add=on]   loudness drives a property
  osatura live-map <project.json> <events.json|song.mid> <mapping.json> <out.json> [--takes lib.json] [--tol 0.01] [--allow-empty]   recorded live input (MIDI, mouse, gamepad, keys) -> keys, takes, events
  osatura beats <project.json> <out.json> <audio.wav> [sensitivity=1.5] [gap=4] [as=markers|events] [name=beat]
  osatura waveform <audio.wav> <out.json> [buckets=800]   min/max peaks for drawing
  osatura skin <project.json> <out.json> add <group> <name> <layer,layer,...>
  osatura skin <project.json> <out.json> rig <group> <name> <constraint,...|-> <ik,...|->   constraints / IK chains active only while that skin is
  osatura skin <project.json> <out.json> bones <group> <name> <bone,...|->   bones that exist only while that skin is active (collapsed to scale 0 otherwise)
  osatura skin <project.json> <out.json> apply <group> <name>
  osatura skin <project.json> <out.json> schedule <group> <frame:name,frame:name,...>   step keys on layer opacity
  osatura parent-switch <project.json> <out.json> <bone> <frame:parent|none,...>   bakes animated re-parenting without a jump (no IK/constraint/spring bones)
  osatura bone-vis <project.json> <out.json> <bone> <from> <to>|clear   show the bone in overlays only for a frame range
  osatura pick <project.json> --frame N --x X --y Y [--tol PX] [--root] [--bones] [--points a,b] [--locked a,b] [--json]   what is under a canvas point (topmost first)
  osatura select-box <project.json> --frame N (--rect x0,y0,x1,y1 | --lasso x,y;x,y;x,y) [--mode bbox|contain|intersect] [--root] [--bones] [--json]   marquee selection
  osatura timeline <project.json> [--json]   tracks per channel grouped by bone/layer, with keys, locks and colours
  osatura keyedit <project.json> <out.json|-> move --sel SEL [--dt N] [--dv N] [--collision replace|merge|refuse] [--no-snap] [--free]   dope-sheet edits as one undo-safe batch (- = dry run, prints the plan)
  osatura keyedit <project.json> <out.json|-> scale --sel SEL --pivot F --factor X | dup --sel SEL --offset N | delete --sel SEL | ease --sel SEL --ease E | value --sel SEL --value V | smooth --sel SEL [--kind monotone|catmull_rom]
  osatura keyedit <project.json> <out.json|-> copy --sel SEL --clip clip.json | paste --clip clip.json --at F [--remap from_target:prop=to_target:prop]
  osatura keyedit <project.json> <out.json|-> add --target T --prop P --at F [--value V] [--ease E] | ripple --at F --by N --mode insert|delete [--scope 0,1] [--adjust-frames]
  osatura keyedit <project.json> <out.json|-> offset --channels 0,1 --dt N | extrapolate --channels 0,1 --extrapolate loop | loop --channels 0,1 [--end F] | handle --sel SEL --side in|out --t F --v V [--mode free|aligned|auto] | lock --target T [--prop P] [--value on|off]
  (SEL = TARGET:PROP[@F0..F1] items joined by ; or #CHANNEL[@F0..F1] or all; E = linear, step, ease_in, ease_out, ease_in_out, smooth, bounce, bezier:x1,y1,x2,y2 or JSON)
  osatura reparent <project.json> <out.json> <bone> <new_parent|none>   keeps the rest pose
  osatura comp     capture|apply <project.json> <out.json> <name>  named layer states: store the current state as name, or restore it
  osatura comp     list <project.json>                print the names (nothing is written)
  osatura event    <project.json> <out.json> <name> <frame> [--data JSON]
  osatura snapshot save <project.json> <dir> [--keep 20]   numbered history of a project in a directory; only the newest --keep revisions stay
  osatura snapshot list <project.json> <dir>   print the revisions
  osatura snapshot restore <project.json> <dir> <out.json> [--number N]   write revision N (default: the newest)
  osatura snapshot recover <project.json> <dir> <out.json>   write the newest revision that can still be read
  osatura explain  <project.json> <bone-or-layer> <frame>      why is it here? ordered contributions
  osatura spring-fit --start V --samples a,b,c,... [--fps 24]    fit spring frequency/damping to a drawn curve
  osatura compare  <a.json> <b.json> <frame> <out.png> [--gain 4 --scale N]   before | after | difference

EFFECTS / OVERLAYS
  osatura fx       <project.json> <frame> <chain.json> <out.png> [--scale N] [--layer-normals] [--clamp]   render + post-effects chain (relight: normal_map, spot lights); --layer-normals shades each layer with its own normal map
  osatura layer-normal <project.json> <out.json> <layer> <normal.png|none> [--strength S] [--fit stretch|tile|exact] [--green-down]   attach (or remove) a normal map on a layer for relight --layer-normals
                    (blur, drop_shadow, glow, outline, posterize, pixelate, levels, fade)
  osatura color-space <project.json> <out.json> srgb|linear   set how the raster / GIF / video / render-hdr compositors blend: srgb (default, as stored) or linear (linear light)
  osatura onion    <project.json> <frame> <out.png> [--before 2 --after 1 --step 1 --scale N]
  osatura onion    <project.json> <out.svg> --frame N [--before B --after A --step S --keyed --outline --all-parts --layers a,b --tint-before C --tint-after C --opacity X --falloff X --ghosts-only]
                    SVG still: the frame with tinted onion-skin ghosts over it (--keyed = nearest key frames instead of fixed steps)
  osatura trail    <project.json> <bone> <out.svg> [--from F --to F]   motion trail of a bone tip
  osatura trail    <project.json> [out.svg] --target layer:PATH|bone:NAME[:head|tip]|point:LAYER:KIND:A[:B] [--from F --to F --smooth N --json]
                    world-to-screen path of a layer pivot, bone or control point with a tick per frame and key markers
  osatura proxy-bench <project.json> [--frames N]   milliseconds per frame, full quality against each proxy level

EDITING / GENERATORS / IMPORT
  osatura keys     shift <project.json> <out.json> --by N [--from F --to F --target NAME --prop PROP]   move keys by N frames
  osatura keys     scale <project.json> <out.json> --factor X [--pivot F --from F --to F --target NAME --prop PROP]   scale key times about a pivot frame
  osatura keys     ease <project.json> <out.json> --ease linear|step|ease_in|ease_out|ease_in_out|smooth [--from F --to F --target NAME --prop PROP]   set the easing of the keys
  osatura keys     reverse <project.json> <out.json> [--from F --to F --target NAME --prop PROP]   reverse the key order inside the range
  osatura marker   <project.json> <out.json> <name> <frame> [--end F]
  osatura idle     blink <project.json> <out.json> --target T --prop P [--interval S --jitter X --closed F --seed N]   random blinks
  osatura idle     breath <project.json> <out.json> --target T --prop P [--amp A --period F]   slow scale breathing
  osatura idle     sway <project.json> <out.json> --target T --prop P [--amp A --freq HZ --seed N]   gentle noise sway
  osatura follow   <project.json> <out.json> --target SRC --prop P --chain a,b,c [--delay F --decay X]
  osatura import-svg <file.svg> <out.json> [--split]  SVG subset -> one vector layer, or one layer per shape named by its id (warnings for the rest)
  osatura mocap-info <file.bvh>   hierarchy, channels, frame rate and extent of a BVH file
  osatura mocap-match <project.json> <file.bvh> [--map map.json]   automatic BVH joint to rig bone matching with confidence table
  osatura mocap-retarget <project.json> <file.bvh> <out.json> [--view front|side|top|yaw=DEG] [--map map.json] [--scale auto|N] [--fps F] [--smooth N] [--start F] [--end F] [--rest-frame F] [--min-conf C] [--weight W] [--mirror] [--no-root] [--in-place] [--keep-length] [--footlock BONE --ground Y]   bake mocap as appended channels; a clip longer than the project extends the project length (a message says so), --keep-length cuts the clip at the project length instead
  osatura import-bvh <project.json> <file.bvh> <out.json> --joint-map Joint=bone,.. [--axis Zrotation --tol 0.5]
  osatura face-calibrate <landmarks.ndjson> <out.json> [--calibrate-frames N] [--map mapfile.json] [--fps F] [--layout L]
  osatura face-train --samples N --seed S --out model.bin [--mine-rounds 1] [--scene N] [--reg-win N] [--reg-cell N] [--det-win N] [--det-cell N] [--ridge-lambda X] [--stages N] [--refine-win N] [--stage-patch N] [--stage-cell N] [--stage-lambda X]   train the own face detector on synthetic faces only (no downloads); stylised/cartoon faces, NOT photographs
  osatura face-eval [--model m.bin] [--eval-n 200] [--seed 9] [--scene 128] [--report r.json] [--json]   held-out synthetic accuracy, false positives, robustness sweeps (JSON)
  osatura face-detect <image> [--model m.bin] [--json] [--overlay o.png] [--threshold T] [--max-faces N] [--min-side N] [--max-side N] [--max-dim N]   face boxes and 23 landmarks (osatura23)
  osatura face-track <video-or-frame-dir> <out.ndjson> [--model m.bin] [--fps 30] [--max-dim 256]   landmarks NDJSON for face-bake (video through ffmpeg)
  osatura face-synth <out-dir> [--frames 90] [--size 160] [--seed 7] [--fps 30]   synthetic head animation: PNG frames, truth.ndjson, params.json
  osatura face-bake <project.json> <landmarks.ndjson> <mapfile.json> <out.json> [--calibrate-frames N|0] [--calib calib.json] [--fps F] [--layout L] [--tol T] [--allow-empty]
  osatura lipsync-text <project.json> <audio.wav> <transcript.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--lang ro|en] [--words-out timing.json|timing.srt]
  osatura speech-motion <project.json> <audio.wav> <transcript.txt> <out.json> [--head bone] [--brow bone_or_layer] [--nod deg] [--fall deg] [--tilt deg] [--brow-px px] [--seed n] [--lang ro|en] [--words-out timing.json|timing.srt]
  osatura mouth    <project.json> <cues.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--rhubarb]

GEOMETRY / RIG / ANIMATION HELPERS
  osatura trace    <image.png> <out.json> [--threshold 128 --tolerance 1.0 --min-area 12 --fill #000000]
  osatura pose-db  build <out.json> [--project P.json] [--library takes.json] [--step N]   pose database (bone-rotation vectors at the key frames of the project and of every library take)
  osatura pose-db  info <db.json>                     bones, sequences, PCA axes and explained variance, library spacing
  osatura pose-db  train-mlp <db.json> <out.json|out.bin> [--hidden 64,64] [--epochs N] [--lr R] [--seed S]   train the optional small pose-completion network (plain SGD) and write the weight file
  osatura pose-suggest between <project.json> <db.json|-> --a F1 --b F2 [--t 0.5] [--snap 0.5] [--top 5] [--apply RANK --out out.json] [--json]   in-between pose of two frames that stays on the learned pose manifold ('-' learns from the project itself)
  osatura pose-suggest complete <project.json> <db.json|-> --frame F [--fix bone=angle,...] [--reach bone=x,y,...] [--free b1,b2] [--prior 1] [--model m.json] [--apply RANK --out out.json]   most plausible full pose given fixed angles and/or tip targets
  osatura pose-suggest next <project.json> <db.json|-> --frame F [--k 8]   what the library did after poses like the one at F
  osatura pose-bench   held-out accuracy of the pose suggestion methods on a synthetic library (also trains the small network)
  osatura autotrace <image.png|jpg> <out.json|out.svg> [--mode fill|line] [--colors N=8] [--min-area A=8] [--tol T=1] [--smooth N=3] [--corner DEG=55] [--denoise N=0] [--quant kmeans|mediancut] [--seed S] [--fit bezier|polygon|vw] [--zorder area|luminance] [--threshold T] [--invert] [--prune N=4] [--into project.json] [--name L] [--fit-stage] [--png preview.png]   local bitmap tracing (no model, no network): colour quantisation, contours, Schneider cubic Bezier fitting, holes by even-odd; line mode thins to centre lines with Zhang-Suen; prints counts and the measured error against the source (mean abs error, SSIM)
  osatura autotrace-bench   trace synthetic shapes / gradient / strokes and print the measured fidelity table
  osatura mesh-trace <project.json> <out.json> <layer> [density=24] [tolerance=1.5] [smooth=N] [hotspot=x,y,r;...] [edge=D/4] [grading=0.5] [lambda=0.5] [taubin=1] [smooth-boundary=0] [threshold=128] [bones=R] [hot-size=E] [skin=1]   traces an image layer's alpha silhouette into a variable-density free triangle mesh (denser at the outline, hot-spots and bones), smooths it and skins it to the bones; prints vertex/triangle counts and quality
  osatura skin-diffuse <project.json> <layer> <out.json> [--influences 4]   heat-diffusion skin weights
  osatura drive    <project.json> <signals.csv> <maps.json> <out.json> [--tol 0.5]   performance -> keys
  osatura state-bake <project.json> <machine.json> <events.json> <out.json> [--tol 0.01]
  osatura pose-match <project.json> <params.json> <goals.json> <out.json> [--generations N --seed N]
  osatura retarget <source.json> <dest.json> <out.json> --pairs srcChain=dstChain[,..]
";
