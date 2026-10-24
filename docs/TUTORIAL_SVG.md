<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Tutorial: from an Inkscape drawing to a walking character

*For an artist who draws in a vector editor and has never written a line of code. About ten minutes. Every command below was run on 24 Oct 2026 with the release build of `osatura`, and the output shown is the real output. This page is English only; seven languages exist for the main documents on the `landing-page` branch, but not for this one.*

You will take a drawing, split it into parts, give the parts a skeleton, make them walk and export a GIF. The files used are in the repository: [`docs/examples/pip.svg`](examples/pip.svg) (the drawing) and [`docs/examples/pip-rig.rhai`](examples/pip-rig.rhai) (the skeleton and the walk, as a script).

![Eight frames of the walk of Pip, a simple character made of rounded parts: the legs swing against the arms, the knees bend and the head nods a little.](img/tutorial-svg-walk.png)

## 1. Draw it so that it can be taken apart

OSATURA reads **SVG** (the format Inkscape, Affinity Designer, Figma and Illustrator can export). Three habits make the import painless:

1. **One shape per body part, with an id.** In Inkscape: *Edit > XML Editor* (or *Object Properties*, Ctrl+Shift+O) and set the **ID** of each shape: `head`, `torso`, `arm_up`, `arm_low`, `leg_up`, `leg_low`, `foot`... The id becomes the name of the layer.
2. **Draw back to front.** The order in the file is the drawing order: the far arm and the far leg first, the near arm last.
3. **Split what must bend.** A shape is rigid: it turns around a bone but does not bend. An arm that bends at the elbow is therefore two shapes (`arm_up`, `arm_low`), a leg is `leg_up`, `leg_low`, `foot`.

What the importer understands: paths (every command), rectangles, circles, ellipses, lines, polylines, polygons, nested groups with transforms, solid fill and stroke colours. What it **skips with a warning**: gradients (drawn grey), text, images, `<use>` and **rounded corners of rectangles**; filters, masks and clipping are **ignored silently** (draw capsules as paths, as `pip.svg` does, or convert objects with *Path > Object to Path* in Inkscape before saving).

## 2. Import it, one layer per shape

```
$ osatura import-svg pip.svg pip.json --split
wrote pip.json (16 paths, 16 layers)
$ osatura check pip.json
ok: 'pip' 400×400 @ 24 fps, 24 frames, 0 bones, 0 ik, 16 layers, 0 channels
```

Without `--split` the whole drawing arrives as one layer called `svg`, which cannot be rigged part by part. With `--split` every shape is a layer named after its id (a repeated id gets `_1`, `_2`...). Look at it: `osatura render pip.json 0 pip.png --scale 2`.

## 3. Give it a skeleton and a walk

`pip-rig.rhai` is a short script (read it: it is about forty lines, with comments). It does three things: it adds **bones** (a body, a head, and for each side an arm with two bones and a leg with three), it **attaches** each layer to its bone (`set_layer("leg_up", "bone", "thigh")`), and it writes the **keys** of a one-second walk: the legs swing, the knees bend, the feet stay flat, the arms swing against the legs, the body bobs and the head nods.

```
$ osatura script pip.json pip-rig.json pip-rig.rhai
86 request(s) applied; wrote pip-rig.json
$ osatura check pip-rig.json
ok: 'pip' 400×400 @ 24 fps, 24 frames, 12 bones, 0 ik, 16 layers, 12 channels
$ osatura lint pip-rig.json
0 finding(s), 0 error(s)
```

The script is applied all or nothing: if one line is wrong, nothing changes and the message names the line.

## 4. Look at it, change it, export it

```
$ osatura studio pip-rig.json
```

opens the studio in your browser with your rig (on Windows a double-click on `osatura.exe` opens the walking-person demo instead; start the command above from a terminal for your own file). Press **Play**, drag a bone, press **Simple view** if the window looks crowded, click **GIF**: a *Save as* dialog appears at once and the page says "Rendering GIF..." until the file is written. From a terminal:

```
$ osatura export pip-rig.json pip.gif
wrote pip.gif
$ osatura export pip-rig.json pip.html
wrote pip.html
```

The GIF is 147 KiB and the self-contained HTML player 158 KiB.

## 5. What is missing, honestly

* **No bending inside a shape.** Pip is made of rigid parts, so joints show as overlaps. For a smooth elbow or knee the shape must be skinned to two bones (`osatura autorig`, the studio's weight tools, or the way [`demo-person`](CLI_REFERENCE.md#demo-person) is built). That is the next tutorial, and it is not written yet.
* **No gradients, text or images in the import** (see section 1). Re-draw gradients as flat bands, or paint them later in the project.
* **The script is the only place where bones are placed from the command line.** In the studio you can add and move bones with the mouse, but this tutorial was not tested with a person who had never used the program: if a step is unclear, that is a bug in this page, and report it as an issue on [GitHub](https://github.com/chiuta/osatura/issues) (rules for contributions: [CONTRIBUTING.md](../CONTRIBUTING.md)).

Next: [GETTING_STARTED.md](GETTING_STARTED.md) (Buddy, built from an empty file), [RIGGING_GUIDE.md](RIGGING_GUIDE.md), [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).
