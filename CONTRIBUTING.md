# Contributing to OSATURA

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/CONTRIBUTING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/CONTRIBUTING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/CONTRIBUTING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/CONTRIBUTING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/CONTRIBUTING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/CONTRIBUTING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/CONTRIBUTING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/CONTRIBUTING.md)

* **Conduct.** Everyone taking part follows the [Code of Conduct](CODE_OF_CONDUCT.md); to cite OSATURA see [CITATION.cff](CITATION.cff).
* **Independent-implementation rule.** Contribute only code, maths and formats you may legally share. Do not copy from, decompile, or reverse-engineer proprietary tools, files or UIs, and do not paste code from projects whose licence is not compatible (see CLEANROOM.md). Do not read or paste source code of competing tools while writing a feature. Cite the public standard or textbook you implemented from.
* **Licence of contributions (inbound = outbound).** A contribution is accepted under the licence of the part it modifies: MPL-2.0 for `osatura-core`, `-capi`, `-wasm`, `-gpu` and `scripts/`; AGPL-3.0-or-later for `osatura-cli`; CC-BY-4.0 for documentation and synthetic assets. Every source file carries an SPDX line that must equal its crate's `license` (checked by `osatura-cli/tests/spdx_audit.rs`).
* **No relicensing right is granted by the sign-off.** The sign-off below certifies origin only. **A commercial or dual licence (for example a proprietary licence for embedders) is NOT currently possible for contributed code**: it would need a separate Contributor Licence Agreement or an explicit outbound grant from each contributor, and none is in place. Until one exists, nobody (including the maintainer) may claim to be able to relicense other people's contributions. The project offers no commercial or dual licence at all ([CODE_SIGNING_POLICY.md](CODE_SIGNING_POLICY.md)).
* **Review.** Every pull request from someone without write access is reviewed by a committer before it is merged; release builds that are code-signed come only from reviewed code on `main` ([CODE_SIGNING_POLICY.md](CODE_SIGNING_POLICY.md), team roles).
* **Sign-off (DCO).** Every commit must carry `Signed-off-by: Your Name <email>` (`git commit -s`), certifying the Developer Certificate of Origin 1.1 reproduced below.
* Add tests; run `cargo test --workspace` and `scripts/licence-audit.sh` before opening a pull request.

## Developer Certificate of Origin 1.1 (verbatim)

*Reproduced unchanged from https://developercertificate.org/ (Copyright (C) 2004, 2006 The Linux Foundation and its contributors); its own terms permit verbatim copies. This text is not covered by the CC-BY-4.0 of the rest of this file.*

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```
