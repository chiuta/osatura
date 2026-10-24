<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Good first tasks

*Small, self-contained jobs for a first contribution. Read [CONTRIBUTING.md](../CONTRIBUTING.md) first (sign-off with `git commit -s`, tests, licence rules). Build and test steps: [DEVELOPMENT.md](DEVELOPMENT.md). Each task says where to look and how you know it is done. Several need no Rust at all.*

## Without writing code

1. **Run the tutorial on your system and report.** Follow [TUTORIAL_SVG.md](TUTORIAL_SVG.md) on macOS, Windows or Linux and open an issue with where you got stuck. Nobody who is not the author has done this yet. *Done when:* the page matches what you saw, or the differences are listed.
2. **Test the browser module in Firefox or Safari.** Run `osatura-wasm/js/browser-check.js` or the checklist in [WASM.md](WASM.md); only Chromium is verified. *Done when:* results are posted.
3. **Draw a second demo.** A short SVG character (see the drawing rules in the tutorial) released under CC-BY-4.0. *Done when:* `import-svg --split` and `check` accept it.
4. **Read a translation.** Seven languages (ro, fr, it, es, pt, de, hu) exist for the main documents on the `landing-page` branch, written by a machine and not reviewed by native speakers. Correct wording in `docs/i18n/<lang>/` there. *Done when:* `python3 scripts/i18n.py check` (also on that branch) still passes.

## Small code and docs jobs

5. **Gradients in `import-svg`.** Gradients currently fall back to grey; map a two-stop linear gradient to a project paint. Start at the SVG reader in `osatura-core` and its tests. *Done when:* a test imports a gradient and the render differs from grey.
6. **Rounded rectangles in `import-svg`.** `rx`/`ry` are ignored. *Done when:* a test checks the corner geometry.
7. **TypeScript declarations for the web wrapper** (`osatura-wasm/js/osatura.js`), listed under known rough edges in [WASM.md](WASM.md).
8. **A `--help` test per command.** The usage text is checked against [CLI_REFERENCE.md](CLI_REFERENCE.md) by one test; add examples that run the documented command lines.
9. **Studio keyboard help.** Add missing shortcuts to the in-app list and [STUDIO.md](STUDIO.md).

Pick one, say so in an issue so two people do not do the same job, and keep the change small. If a task description is wrong or out of date, fixing it is itself a good first contribution.
