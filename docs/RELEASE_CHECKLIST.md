<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Release checklist (owner side)

*Steps that only the owner can do, in order. Status on 24 Oct 2026: the repository <https://github.com/chiuta/osatura> exists, GitHub Actions is green (tests on Linux, macOS and Windows, clippy, wasm32, cargo-deny, REUSE) and the draft pre-release `v0.1.0-alpha.3` has been built (Windows, macOS and Linux archives, the web module and `SHA256SUMS.txt`); **nothing is published yet**. Steps below that CI already did are ticked; the rest need a person, a lawyer or an account. Not legal advice. Items marked **[blocker]** should be done before a public announcement.*

## 1. Before publishing anything

- [ ] **[blocker]** Confirm or replace the security contact (`alexio@trom.tf` is a placeholder in `docs/SECURITY.md`, `--version` and the licence documents).
- [ ] **[blocker]** Search the name "OSATURA" in the trademark registers you care about (EUIPO, USPTO, OSIM) and in app stores / package registries. See `TRADEMARK.md`. If it clashes, rename before the first release (the name appears in the binary, crates, docs).
- [ ] **[blocker]** Ask a lawyer to review `LICENSING.md`, the MPL-2.0 / AGPL-3.0 split, `docs/THIRD_PARTY.md` and `docs/PATENT_NOTES.md`. Decide whether you want dual licensing; if yes, a CLA is needed *before* accepting outside contributions (today: DCO only).
- [ ] Check the terms of the Graphics Gems curve-fitting reference (quoted from memory in `CLEANROOM.md`); keep the independent-implementation wording only if it is accurate.
- [x] `reuse lint` and `cargo deny check` pass in CI (the `licences` job); run them once on your machine too.
- [ ] Check that no personal data, local paths or tokens are in the tree (`grep -r "/home/" .`, `grep -ri token`), and that the bundled model and demo assets are all self-generated.

## 2. Repository

- [x] The GitHub repository exists and the tree was pushed without `target/` and `out/` (`.gitignore`).
- [ ] Keep `LICENSE`, `LICENSES/`, `REUSE.toml`, `.github/FUNDING.yml` (Buy Me a Coffee `echo.of.the.strings`, Patreon `alexio_tf`).
- [ ] Enable private vulnerability reporting (Security tab) and point `docs/SECURITY.md` to it.
- [ ] Pass the repository URL with `--source-url` when running the studio (AGPL source offer, `/source` route) and test the offer from a fresh install.

## 3. First CI run (done 2026-10-24)

- [x] `.github/workflows/ci.yml` is green: Linux, Windows, macOS tests and `clippy -D warnings`; the wasm32 build job; cargo-deny and REUSE job.
- [x] The first-run failures on Windows were fixed (stack size, POSIX-only tests, collaboration log access) and noted in `docs/LIMITATIONS.md` (L-04).
- [x] The wasm32 job passes (build and Node real-module test). **Open:** run the verification list in `docs/WASM.md` in a real browser and update L-01 / L-02.

## 4. Human testing

- [ ] Give the studio to 3 to 5 animators for a week with a short task (rig a character, make a walk cycle, export a GIF). Record where they stall; file each as an issue.
- [ ] Test on real browsers (Chrome, Firefox, Safari), a touch device and a keyboard-only session.
- [ ] On a real **Windows 11** and a real **Mac**: unpack the archive, double-click `osatura.exe` (Windows) and `osatura-studio`, rig and export one project, and report what fails (L-74 to L-76).
- [ ] Try the face detector / mocap / voice features with real data; report honestly that they are limited (L-12, L-13).

## 5. Deployment safety (if you host the studio or collaboration)

- [ ] Put it behind a TLS-terminating reverse proxy (recipe: `docs/SECURITY.md` section 12); never expose the plain HTTP port to the internet.
- [ ] Use per-participant tokens (`collab-token`), set expiry, bind to localhost behind the proxy.
- [ ] Commission an independent security review before offering hosted collaboration to strangers.

## 6. Release artefacts

- [x] The release binaries are built on GitHub-hosted runners (`release.yml`) and `SHA256SUMS.txt` is generated in the draft. **Open:** review the draft `v0.1.0-alpha.3`, tick "pre-release", publish it, and tag the commit (the tag is created with the release).
- [x] Provenance signing: Sigstore keyless signatures and GitHub build-provenance attestations for every release file (job `sign` in `release.yml`, verified in CI).
- [x] SignPath Foundation prerequisites in the repository: CODE_SIGNING_POLICY.md (linked from the README, the site and every release), PRIVACY.md, docs/UNINSTALL.md, no dual licensing (LICENSING.md), Windows file metadata (`build.rs`, checked in release.yml), artifact configuration in `.signpath/`, job `windows-authenticode`.
- [ ] Owner: enable two-factor authentication on GitHub (required by SignPath for every team member).
- [ ] Owner: publish a release (SignPath requires the project to be released in the form that will be signed), then apply at https://signpath.org/apply.
- [ ] After acceptance: in SignPath create project `osatura`, signing policy `release-signing`, artifact configuration `windows-zip` (paste `.signpath/artifact-configuration.xml`), link the GitHub repository as trusted build system; in GitHub add the repository variable `SIGNPATH_ORGANIZATION_ID` and the secret `SIGNPATH_API_TOKEN`. The next release is then signed after manual approval.
- [ ] macOS: Apple Developer ID and notarisation (not planned yet); until then Gatekeeper warns (L-74).
- [ ] Attach the corresponding source for the AGPL parts (the tag itself is enough; keep it available as long as you distribute binaries).
- [x] `docs/THIRD_PARTY_NOTICES.md` is regenerated (24 Oct 2026) and each archive carries it; regenerate again with `scripts/licence-audit.sh` before every later release.
- [ ] Crates.io / npm / package registries: check the names are free and the licence fields match `LICENSING.md`.

## 7. Announcement and donations

- [ ] README first screen: what works, what does not (link `docs/LIMITATIONS.md`); do not claim "Moho-class" without qualification.
- [ ] Show the screenshots in `docs/img/` and a short GIF made with the demo project.
- [ ] Add the donation links (Buy Me a Coffee, Patreon) to the README and the studio's About box; do not promise features in return for donations unless you can keep the promise.
- [ ] Post where animators are (communities for 2D rigging / indie animation); ask for bug reports, not praise.
- [ ] Reserve time to answer the first issues quickly; the first two weeks decide whether contributors stay.
