<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# OSATURA glossary

Domain terms used in the code and documentation, in alphabetical order. "See" points to the
module (in `osatura-core/src` unless stated) or document where the term is implemented or
specified. Documents: [PROJECT_FORMAT.md](PROJECT_FORMAT.md), [ARCHITECTURE.md](ARCHITECTURE.md).
Sources for techniques are listed in `CLEANROOM.md`.

**Aim constraint.** A constraint that rotates a bone so its +x axis points at another bone's head,
plus an offset. See `rig`; PROJECT_FORMAT section 4.3.

**Anchor.** A point on a Bezier path with a position and optional in and out handles; it may also
carry bone weights. See `path`; PROJECT_FORMAT section 6.

**Angle limits.** Per-bone minimum and maximum of the animated rotation, in degrees, enforced after
channels and after each IK chain. See `rig`, `engine`.

**API version.** The version of the stable Rust surface (`api` and `prelude`), currently 1.4.0,
checked by a snapshot test. Not the same as the file `format` or the C `ABI_VERSION`. See `api`.

**Asset.** A file referenced by a project (image, normal map, sound). Paths must be relative and stay
inside the project directory unless `--allow-external-assets` is given. See `assets`; PROJECT_FORMAT
section 18.

**Bake.** To compute values over time and store them as ordinary channels or keys, so that they no
longer need the computing tool. Used by physics, clips, magnet, footlock, state machines and more. See
ARCHITECTURE section 4.

**Bend.** The sign (+1 or -1) that picks which way a two-bone IK chain folds, such as elbow in or out.
See `rig`.

**Bezier curve.** A parametric cubic curve defined by two anchors and their handles. Flattened to line
segments (de Casteljau subdivision) for filling and measuring. See `path`.

**Binding.** A run-time rule that maps an external value (a game variable, a slider) through an
expression, a remap and smoothing onto a property or a state-machine input. See `binding`.

**Blend mode.** How a layer's colour combines with what is beneath it (multiply, screen, overlay and
so on). Formulas follow W3C Compositing and Blending Level 1. See `raster`, `fraster`.

**Blend tree.** A 1-D blend of several clips controlled by a number input, used inside a state
machine. See `machine`.

**Blob.** One circle of a liquid (metaball) layer. See `fx`.

**Bone.** A named segment of the skeleton with a head position, length, rest angle and optional
parent. Bones drive skinned geometry and attached layers. See `rig`; PROJECT_FORMAT section 4.

**Boolean layer.** A layer that combines vector paths by union, intersection, difference or xor. The
first path is the subject and the rest are united as the clip. See `fx`.

**Bounce (easing).** An easing curve of a ball falling and rebounding with restitution 0.6 and three
rebounds. An original derivation from free fall. See `anim`.

**Bounding box.** The smallest axis-aligned rectangle around geometry; used for alignment, picking and
the fit of normal maps. See `layout`, `pick`.

**Cage deformer.** A deformer in which content inside a closed polygon follows the polygon's vertices
using mean-value coordinates. See `cage`; also MVC.

**Catmull-Rom spline.** A cubic spline that passes through its control points. The `smooth` easing uses
a non-uniform version through neighbouring keys. See `anim`, `liquid` smoothing in `fx`.

**Channel.** One animated value: a target, a property, a source (keys, expression, noise or driven) and
a blend mode and weight. Channels run in list order. See PROJECT_FORMAT section 8.

**CIE76.** The original colour difference, the Euclidean distance in CIE L*a*b*. Cheap, less accurate
than CIEDE2000. See `percep`.

**CIEDE2000.** The CIE 2000 colour-difference formula (CIE TR 142-2001), implemented as printed in
Sharma, Wu and Dalal (2005) and checked against their published test pairs. Used by animation CI to
judge visible change. See `percep`, `ci`; `docs/CI.md`.

**Independent implementation (not a strict clean room).** The development rule that code is written from public standards and textbook methods,
with no copied code and no reproduction of other tools' formats or names. Each technique is logged in
`CLEANROOM.md`.

**Clip.** A reusable take of animation tracks that can be placed on the timeline with speed, loops,
weight, fades and a blend mode. See `clipstack`; PROJECT_FORMAT section 11.1.

**Cloth.** A grid deformer whose vertices move exactly by their own offsets. Also the position-based
dynamics simulation that bakes those offsets. See `cloth`.

**Constraint.** A rule applied after IK that limits a bone: aim, copy rotation, or pin. See `rig`.

**Container (layout).** A flexbox-style one-dimensional layout group with gap, padding, justification,
alignment and wrapping, which positions layers and other containers. See `containers`.

**Copy-rotation constraint.** A constraint that makes a bone match another bone's world rotation. See
`rig`.

**CRDT.** Conflict-free replicated data type: a structure that replicas can edit independently and
merge to the same result without coordination. OSATURA uses an operation-based CRDT. See `collab`;
ARCHITECTURE section 10. Related: LWW register, RGA, Lamport clock, vector clock, tombstone.

**Cubic Bezier easing.** A CSS-style easing curve defined by two control points between (0,0) and
(1,1). See `anim`.

**Deformer.** A shape modifier applied to a vector or image layer in rest space: lattice, spine, pin,
cage or cloth. See PROJECT_FORMAT section 7.1.

**Delaunay triangulation.** A triangulation of points in which no point lies inside any triangle's
circumcircle, which avoids thin triangles. Built with the Bowyer-Watson algorithm for image meshes.
See `meshtrace`.

**Depth of field (DoF).** Blur that increases with the distance of a layer's `parallax` from the camera
`focus`. Applied by the `dof` and `dof-export` commands only. See `dof`.

**Determinism.** The property that the same input yields the same output. Frame evaluation is pure;
rendering is byte-identical on one machine and build, not verified across platforms. See
ARCHITECTURE section 6.

**Dope sheet.** A timeline view that shows keys as marks per channel and edits them in time. See
`timeline`, `keyops`.

**Driven key.** A channel whose input is another property's value rather than time, for example elbow
angle to bulge size. See `drivers`; PROJECT_FORMAT section 8.1.

**Easing.** The shaping function applied to the segment starting at a key. See `anim`; PROJECT_FORMAT
section 8.4.

**Editor.** The object that owns a project and its undo history and applies ops. See `edit`;
ARCHITECTURE section 5.

**Emitter.** A layer that produces particles as a closed-form function of time. See `fx`.

**Engine.** The validated, name-resolved form of a project that evaluates frames. See `engine`.

**Envelope (collab).** The message that carries ops between replicas: replica id, sequence number,
Lamport time, dependencies and operations. See `collab`.

**Evaluation.** Computing the pose, geometry and camera for one frame from a project. A pure function
of the frame. See PROJECT_FORMAT section 16.

**FABRIK.** Forward And Backward Reaching Inverse Kinematics: an iterative IK solver that alternately
pulls the chain toward the target from the tip and then re-anchors it at the root. Used for chains of
more than two bones. See `rig`.

**FK.** Forward kinematics: computing world transforms of bones from their parents' transforms and
their own angles and offsets. See `rig`, `engine`.

**Format version.** The integer `format` field of a project, currently 1. Newer files are rejected.
See PROJECT_FORMAT section 19.

**Frame.** (1) One time step of the timeline, `0..frames-1`. (2) The `Frame` struct: the result of
evaluating one step (bones, geometry, camera, flags). See `engine`.

**Glue.** A link that makes vertices of one mesh follow vertices of another (snap, rigid or offset). See
`links`.

**Gradient handle.** A control point of a linear or radial gradient that can be animated with
`grad_dx` and `grad_dy`. See PROJECT_FORMAT section 8.2.

**Group layer.** A layer that contains other layers and passes its transform to them. See `project`.

**Handle.** A Bezier control point attached to an anchor. Absolute coordinates in the file. See `path`.

**HOG.** Histogram of oriented gradients, a local-shape feature (Dalal and Triggs 2005) used by the face
detector. See `facedet`.

**IK.** Inverse kinematics: computing bone angles so that the tip of a chain reaches a target. See `rig`.
Related: FABRIK, soft IK, stretch IK, bend.

**IK mix.** The blend between the FK pose (0) and the IK result (1) for a chain. Animated with `ik_mix`.
See PROJECT_FORMAT section 4.2.

**Image mesh.** A grid or free triangle mesh over an image layer, used for bone weights and deformers.
See PROJECT_FORMAT section 5.8.

**Keyframe (key).** A (frame, value, easing) triple on a track. The easing shapes the segment that begins
at the key. See `anim`.

**L*a*b*.** CIE 1976 colour space designed so that distances approximate perceived differences. See
`percep`.

**Lamport clock.** A logical counter that orders events in a distributed system without a shared
clock (Lamport 1978). See `collab`.

**Lattice.** A deformer with a grid of control points over a rectangle whose offsets bend the content
through Bezier-Bernstein blending. See PROJECT_FORMAT section 7.1.

**Layer.** A node in the scene tree: vector, group, switch, liquid, boolean, emitter or image. See
PROJECT_FORMAT section 5.

**Layer effect.** A per-layer filter (blur, drop shadow, tint, fill) rendered on an offscreen canvas.
See `layerfx`.

**Level of detail (LOD).** A simplified version of a project produced under a declared error bound in
pixels, for fast preview or small output. See `lod`, `osatura lod`.

**Linter.** The advisory checker (`osatura lint`) that reports animation-quality problems with codes
such as L011 and L012. Separate from validation. See `lint`.

**Liquid layer.** A layer of metaballs whose outline is the iso-contour of a sum-of-influence field. See
`fx`.

**LWW register.** Last-writer-wins register: a CRDT cell where the write with the highest logical
timestamp survives. Used for object keys. See `collab`.

**Magnet.** A baking tool that pulls positions toward a point with smoothstep falloff. See `magnet`.

**Marching squares.** An algorithm that extracts contour lines from a sampled scalar field on a grid.
Used for liquid outlines and bitmap tracing. See `fx`, `trace`, `meshtrace`.

**Marker.** A named frame or frame region on the timeline. State machines can use a marker region as
a source. See `keyops`; PROJECT_FORMAT section 10.2.

**Mask.** A layer whose alpha limits another layer's visibility; may be inverted. A hidden layer can
still be a mask. See `raster`.

**Merge (CRDT).** Combining two replicas' states so that both converge to the same document. See
`collab`.

**Mesh lock.** A link that makes a follower mesh track a host mesh with a strength 0..1. See `links`.

**Metaball.** A blob whose field influence blends with neighbours so that nearby blobs fuse. See liquid
layer.

**Morph (blend shape).** An alternative anchor set for a path, applied additively with a weight. See
PROJECT_FORMAT section 6.1.

**MVC.** Mean-value coordinates (Floater 2003): generalized barycentric coordinates for a point inside
a polygon, used by the cage deformer. See `cage`.

**Non-destructive.** Leaving existing work intact. Bakers append channels instead of rewriting keys, and
the editor records inverse ops. See ARCHITECTURE sections 4 and 5.

**Normal map.** An image that stores surface direction as RGB, used by the `relight` effect and
per-layer lighting. See `postfx`; PROJECT_FORMAT section 7.3.

**Op.** An edit expressed as `set`, `insert`, `remove` or `batch` on an RFC 6901 JSON pointer; it has an
inverse for undo. See `edit`.

**OT.** Operational transformation: an alternative to CRDTs that rewrites concurrent operations against
each other, usually with a central server. OSATURA does not use it; it uses a CRDT with a central
sequencer. See ARCHITECTURE section 10.

**Parallax.** A per-layer factor that scales camera movement, so distant layers move less. Also the
depth value used for depth of field. See PROJECT_FORMAT section 10.1.

**Particle.** A short-lived point emitted by an emitter; its position is a closed-form function of
time. See `fx`.

**Path (target).** The slash-separated name of a nested layer, such as `parent/child`; sub-targets use
`::` as in `layer::path::morph`. See PROJECT_FORMAT section 5.

**Path (vector).** A sequence of anchors, open or closed, with optional fill, stroke and morphs. See
`path`.

**Pin constraint.** A constraint that slides a bone's head to another bone's head. See `rig`.

**Pin warp.** A deformer in which dragged pins move the artwork smoothly within a radius, using a
compactly supported radial basis function (Wendland 1995). See `pinwarp`.

**Player.** The run-time wrapper in the facade that adds state machines and bindings to an animation.
See `api`.

**Preorder.** The visiting order of the layer tree: a node, then its children. The evaluation uses it so
parents are computed first.

**Proxy.** A reduced-detail picture used while scrubbing, built from a level of detail. See `scrub`, `lod`.

**QEM.** Quadric error metric (Garland and Heckbert): a measure of the geometric error caused by
collapsing a vertex, used for mesh simplification. See `lod`.

**Raster.** A pixel image, or the module that renders a frame to pixels with tiny-skia. See `raster`.

**RDP.** Ramer-Douglas-Peucker: recursive line simplification that keeps points farther than a tolerance
from the chord. See `trace`, `sizefit`.

**Reference layer.** A layer marked `reference`; hidden unless the editor shows references. See
PROJECT_FORMAT section 5.1.

**Remap.** An affine mapping of a range, with an optional curve and clamp, applied to a binding value.
See `binding`.

**Replica (collab).** One participant's copy of the document in the CRDT. See `collab`.

**Rest pose.** The bone transforms with no animation applied; skin matrices are `world * inverse(rest
world)`. See `rig`.

**RGA.** Replicated growable array (Roh et al. 2011): a sequence CRDT where each element has a unique
id, and deletes leave tombstones. Used for JSON arrays. See `collab`.

**Rig.** The skeleton and its helpers: bones, IK chains, constraints and springs. See PROJECT_FORMAT
section 4.

**Scrub.** Dragging the playhead; the studio caches frames by revision and uses proxies. See `scrub`.

**Skew.** A shear angle in a layer's transform. See PROJECT_FORMAT section 5.1.

**Skin (animation).** A named variant of layers, bones, constraints or IK that can be switched on in a
group, such as costumes. See `skins`.

**Skinning.** Deforming geometry by blending the transforms of bones, weighted per anchor or vertex.
OSATURA normalizes the weights and treats empty weights as static. See `engine`, `weights`.

**Soft IK.** An IK option that eases extension near full reach to avoid snapping straight. Range 0 to
0.9. See `rig`.

**SPDX.** Software Package Data Exchange: a standard way to state a file's licence, as in
<!-- REUSE-IgnoreStart -->
`SPDX-License-Identifier: MPL-2.0`. Every source file carries one. See `LICENSING.md`.
<!-- REUSE-IgnoreEnd -->

**Spine deformer.** A deformer that bends geometry along a curve by projecting anchors onto a rest
curve and re-attaching them to the deformed curve. See PROJECT_FORMAT section 7.1.

**Spring (bone).** A damped second-order spring that makes a bone's rotation lag and overshoot; baked at
load so any frame can be evaluated. See PROJECT_FORMAT section 4.4.

**SSIM.** Structural similarity index (Wang et al. 2004): an image-quality measure from local means,
variances and covariance. OSATURA uses a block version. See `percep`.

**Stage.** The coordinate space of the output image: pixels, +x right, +y down.

**State machine.** A graph of animation states with transitions driven by inputs. There is a baked,
event-log version and a run-time version. See `statemachine`, `machine`.

**Step easing.** An easing that holds the key's value until the next key. Used for switch layers. See
`anim`.

**Stretch IK.** An IK option that scales the chain up to a limit (1..4) to reach a distant target. See
`rig`.

**Stroke.** The outline of a path: paint, width, cap, join, dash and an optional width profile. See
PROJECT_FORMAT section 6.3.

**Switch layer.** A layer that shows exactly one of its children, chosen by an index channel. Used for
frame-by-frame animation and mouth shapes. See PROJECT_FORMAT section 5.4.

**Take.** A named set of tracks that forms a clip's content. See `clipstack`.

**Tombstone.** A marker left in a CRDT array in place of a deleted element so concurrent operations can
still be placed. Compaction removes tombstones every replica has seen. See `collab`.

**Track.** The list of keys of one channel, with an extrapolation mode. See `anim`.

**Transform.** Position, rotation, scale, skew and anchor of a layer; the local matrix is
`T(position) R Shear S T(-anchor)`. See PROJECT_FORMAT section 5.1.

**Vector clock.** A per-replica counter vector that tells whether one event happened before another or
whether they are concurrent (Fidge, Mattern). See `collab`.

**Weights (bone).** The influence of each bone on an anchor or vertex. They can be painted or computed
by diffusion. See `weights`.

**Wobble.** A deterministic noise displacement added to a layer's geometry, with amplitude, scale, rate
and seed. See PROJECT_FORMAT section 5.1.

**Zhang-Suen thinning.** A parallel algorithm that reduces a binary shape to a one-pixel skeleton. Used
by autotrace for centre-line tracing. See `autotrace`.
