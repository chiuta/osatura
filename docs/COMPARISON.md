<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA compared with other 2D animation tools

*Written by the author of OSATURA, so read it as a biased source. Dated 24 Oct 2026. Rule used: a statement about another product appears only when it is general, well known and stable; anything the author could not check that day is marked **unverified** or left out. Prices and licence terms change: look at the vendor's page before deciding.*

## The short version

OSATURA is a pre-alpha engine with a command line and a browser editor, built by one person. The tools below are mature products with artists, documentation and years of production use behind them. **If you have a deadline, use one of them.** OSATURA is worth a look if one of its specific properties matters to you (right column of the table).

| | OSATURA | Spine (Esoteric Software) | Moho (Lost Marble) | DragonBones |
|---|---|---|---|---|
| Maturity | Alpha, one author, builds checked on three systems in CI, no known production use | Long-established commercial tool with runtimes for many engines | Long-established commercial tool | Reported as discontinued by a software directory (AlternativeTo, Oct 2026): check before relying on it |
| Interface | Browser editor and window shell; **has not been tested by working animators** | Desktop editor | Desktop editor | Desktop editor (unverified) |
| Licence | Libraries MPL-2.0, CLI and studio AGPL-3.0-or-later, docs CC-BY-4.0 | Proprietary, paid | Proprietary, paid | Unverified |
| Headless and scripted use | Yes: every operation is a CLI command or a script, with CI checks for animations (`osatura ci`) | Command-line export exists (vendor documentation) | Unverified | Unverified |
| Game-engine runtimes | **None.** Exports GIF, PNG sequence, sprite sheet, SVG, Lottie, HTML player, C library and Rust crate | Many official runtimes | Exports to several formats; engine plug-ins unverified | Unverified |
| Mesh deformation, weights | Weights on anchors, deformers, IK, physics | Mature, with a large feature set (meshes are not in the cheapest edition) | Mature | Unverified |
| Community, tutorials, support | None yet | Large | Large | Unverified |

## Where OSATURA is different

* **Everything is text.** The project is one JSON file that can be diffed, reviewed and generated; the script engine edits it all-or-nothing. That suits teams that already work in Git and CI.
* **No account, no server, no telemetry.** The studio runs on your machine; the web module runs in the page.
* **One engine for the editor, the command line, the browser (WebAssembly) and the C library.** On the demo frame tested, the browser output equals the native output byte for byte ([WASM.md](WASM.md)).
* **Free software.** You may read, change and redistribute it under the licences above.

## Where it is behind, plainly

* No engine runtimes: to use a rig in Unity, Godot or similar you export frames, SVG or Lottie, or write your own integration with the C library.
* No artist has used it for a real project. The interface is designed from the code outwards and may be awkward ([LIMITATIONS.md](LIMITATIONS.md)).
* Import is limited to a subset of SVG and to OpenRaster; there is no Photoshop import.
* Not yet published: no installer, no signed binaries, no macOS user report.

## How to decide

Try the two-minute path in [GETTING_STARTED.md](GETTING_STARTED.md) and the [SVG tutorial](TUTORIAL_SVG.md). If the result is not useful to you in an hour, that is a finding: please report it as an issue on [GitHub](https://github.com/chiuta/osatura/issues) (rules for contributions: [CONTRIBUTING.md](../CONTRIBUTING.md)). Corrections to this page, above all about other products, are welcome as issues.
