# OSATURA — licensing (DECIDED: Variant A, applied 22 Oct 2026; see COPYRIGHT and LICENSES/)

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/LICENSING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/LICENSING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/LICENSING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/LICENSING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/LICENSING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/LICENSING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/LICENSING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/LICENSING.md)

**Status:** Variant A below was chosen and applied: MPL-2.0 for the libraries (core, capi, wasm, gpu), AGPL-3.0-or-later for `osatura-cli` (CLI + studio) and `osatura-gui` (the studio in a native window), CC-BY-4.0 for docs and synthetic assets/models, trademark policy in TRADEMARK.md, DCO in CONTRIBUTING.md, SPDX headers in all .rs/.js/.mjs/.sh files and REUSE.toml for the rest, AGPL section 13 source offer (`/source`, `--source-url`), Cargo `license` fields set. Remaining owner actions: lawyer review, trademark search (EUIPO/OSIM), patent screening (docs/PATENT_NOTES.md), bundle third-party notices with binary releases. **No dual licensing (decided 25 Oct 2026):** OSATURA is not offered, and will not be offered while its binaries are signed with a SignPath Foundation certificate, under any commercial or proprietary licence; all components are under the OSI-approved licences above (CODE_SIGNING_POLICY.md).


*Not legal advice. The licence is DECIDED (Variant A, see COPYRIGHT); the sections below are kept as the rationale and the alternatives that were considered. Have a lawyer (ideally one who knows Romanian/EU copyright) confirm before the first public release. Contributions: no relicensing or dual-licensing right exists for third-party contributions until a CLA is in place (CONTRIBUTING.md).*

## 1. What the licence must do for you
Your income is **donations (Buy Me a Coffee)**, not licence fees. So the licence does not create the money — **goodwill, visibility and trust do**. It has three jobs:
1. **Maximise users** (more users → more supporters): it must be free to use, including in commercial work.
2. **Stop a company taking the code, closing it and selling it** (or hosting it as a paid service) without giving anything back.
3. **Keep the right to defend the name** and to sell support and services. (Selling licence exceptions, i.e. dual licensing, was considered and dropped on 25 Oct 2026: the free SignPath Foundation code signing requires a project without commercial dual licensing.)

It must also fit your **trade-free / TROM** convictions: the software should not be crippled, ad-funded or tied to data harvesting. (Licences that forbid commercial use, e.g. CC-NC or Polyform-NC, are *not* open-source and are not the same thing as "trade-free"; check TROM's own wording before adopting any of them.)

## 2. Findings that constrain the choice
- **Dependencies are no obstacle.** Audit of all third-party crates (363 in the 24 Oct 2026 regeneration by `scripts/licence-audit.sh`, counted over all platforms together after the window app was added; it was 160 on 22 Oct; the number changes, see docs/THIRD_PARTY.md): all are MIT, Apache-2.0, BSD, Zlib, ISC, Unlicense, CC0, Unicode or duals of those, except six weak-copyleft MPL-2.0 crates with file-level copyleft: smartstring (MPL-2.0+, reached only through the `scripting` feature, which is ON by default in `osatura-core` and `osatura-cli`) and `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors` and `option-ext` (reached only through the window app `osatura-gui`); a build of the libraries with `--no-default-features` has no MPL third-party crate. They can be combined with AGPL, MPL or a proprietary licence; the MPL crate needs its source/notice preserved.
- **Copyright ownership.** Much of the code was written with Claude. Copyright generally attaches to *human* creative contribution (US Copyright Office guidance; EU/Romanian law likewise require a human author's own intellectual creation). Practical consequences: (a) you can and should license the project as its human author/compiler, but (b) don't promise exclusivity of every line, (c) keep your own design notes, selections and edits in the repo as evidence of authorship, (d) a DCO only certifies origin; to relicense contributions later (e.g. dual licensing) a CLA or explicit outbound grant is needed and is NOT in place.
- **No licence can force donations.** Never gate code behind a donation; a fork would simply remove the gate. Sell *convenience and extras*, not permission.

## 3. Recommended package — "Open core library, protected app" (all variants below are ready to apply)

### ★ Variant A (recommended): MPL-2.0 library + AGPL-3.0 apps + trademark
| Part | Licence | Why |
|---|---|---|
| `osatura-core` (engine, evaluator, formats, exporters) and any future runtime/WASM/C-ABI | **MPL-2.0** | File-level copyleft: anyone may embed it in games/apps (closed or open), but changes to *our files* must be shared. Includes an explicit **patent grant** and termination on patent suit. Maximises adoption, which is what donations need. |
| `osatura-cli`, `osatura-gui`, studio (browser/window) | **AGPL-3.0-or-later** | Stops closed forks and "OSATURA-as-a-paid-web-service" clones; users' own *projects and renders* stay entirely theirs. |
| Name "OSATURA", logo | **Trademark policy** (separate file) | Forks must rename; protects supporters from imitations. Check availability (EUIPO/OSIM/WIPO) before launch. |
| Sample rigs, art, audio you create | **CC BY 4.0** (or CC BY-SA) | Free to reuse with credit; third-party art keeps its own licence. |
| Documentation | **CC BY 4.0** | |
<!-- REUSE-IgnoreStart -->
Cargo: `license = "MPL-2.0"` (core), `license = "AGPL-3.0-or-later"` (cli). SPDX header per file: `// SPDX-License-Identifier: MPL-2.0`.
<!-- REUSE-IgnoreEnd -->
**Dual-licensing:** not offered (decision of 25 Oct 2026, see Status). Even if reconsidered it would be possible only for code the owner alone wrote, or after a CLA/outbound grant from every contributor (CONTRIBUTING.md), and it would end the SignPath Foundation code signing.

### Variant B: AGPL-3.0-or-later everywhere (+ commercial licence on request)
Strongest anti-hijack position and closest to trade-free ethos. Cost: companies and game engines will avoid embedding the runtime, so fewer users and fewer indirect supporters. Choose if protection matters more than reach.

### Variant C: Apache-2.0 everywhere (not chosen; the workspace applies Variant A: MPL-2.0 libraries and `license = "AGPL-3.0-or-later"` for `osatura-cli`)
Maximum reach and zero friction; patent grant included. Cost: anyone — including a large company — may close and sell it with no obligation. Donations then depend purely on goodwill. Choose only if you want to be a public utility.

### Variant D: Source-available (BUSL-1.1 or Polyform Shield/NC with delayed open date)
Prevents commercial competition for N years. Cost: **not open source**; conflicts with the clean-room/open-format story, with contributions from the community and with trade-free values; many users and distributions will refuse it. **Not recommended.**

| | Reach | Anti-hijack | Patent grant | Dual-licence income | Fits trade-free |
|---|---|---|---|---|---|
| **A MPL core + AGPL apps** | high | good | yes | yes | yes |
| B AGPL all | medium | strongest | yes (v3) | yes | yes |
| C Apache all | highest | none | yes | no | partly |
| D source-available | low | strong | n/a | yes | no |

## 4. How to actually earn from donations (all compatible with Variant A)
1. **Buy Me a Coffee page** (`buymeacoffee.com/echo.of.the.strings`) linked from the README, the CLI `--version` text, the studio footer and the HTML player credit line (a *removable* credit — never mandatory).
2. **Memberships, not paywalls:** monthly tiers for early builds, voting on the next roadmap item, name in `SUPPORTERS.md`, private chat. Everyone gets the same code; members get *time and voice*.
3. **Paid extras that are not the code:** asset packs (rigs, backgrounds, walk-cycle libraries) under a licence that allows use in films/games but not resale of the pack itself, video courses, workshops.
4. **Commissions and support:** rigging services, custom exporters, integration consulting. (No commercial licence for embedders: the MPL-2.0 libraries already allow embedding in closed products.)
5. **Sponsor tiers for organisations** (logo on the README) and grant applications to open-source/creative funds (NLnet, Sovereign Tech-type programmes, culture funds) — they pay for exactly this kind of open infrastructure.
6. **Transparency page:** publish income and what it funded; it measurably raises recurring support.
7. Keep **Patreon** (`patreon.com/c/alexio_tf`) and BMC both visible, but give each a distinct purpose (BMC one-off thanks; Patreon recurring) so supporters aren't confused.

## 5. Checklist to apply Variant A (≈ 1 hour of work once chosen)
- [x] `LICENSES/MPL-2.0.txt`, `LICENSES/AGPL-3.0-or-later.txt`, `LICENSES/CC-BY-4.0.txt`, `LICENSES/LicenseRef-third-party.txt` (REUSE layout) plus the summary `COPYRIGHT`; `REUSE.toml` covers non-source files.
- [x] `license =` set in all six `Cargo.toml` files; SPDX headers in all .rs/.js/.mjs/.sh files, checked by `osatura-cli/tests/spdx_audit.rs`.
- [x] `docs/THIRD_PARTY.md` and `docs/THIRD_PARTY_NOTICES.md`, both regenerated by `scripts/licence-audit.sh`.
- [x] `CONTRIBUTING.md` with the DCO 1.1 text. No CLA: no dual licence is offered.
- [x] `TRADEMARK.md` (permitted: "compatible with OSATURA", unmodified redistribution; not permitted: modified builds under the same name).
- [x] `.github/FUNDING.yml` (`buy_me_a_coffee: echo.of.the.strings`) so the platform shows a Sponsor button.
- [~] Trademark search: the first name ARMATURA collided with a mark in classes 9/42 (EUIPO, 24 Oct 2026), so the project was renamed OSATURA; the EUIPO search for "osatura" found no conflict (one unrelated expired hit in class 10). The package names `osatura` on crates.io, npm and PyPI are free (checked 24 Oct 2026), but a close mark, OSATRA (Open-Xchange AG, US reg. 7459236, classes 9/38/42), exists. OSIM / TMview / WIPO / domain searches and a lawyer's opinion are still open, see [docs/LIMITATIONS.md](docs/LIMITATIONS.md) L-78.
