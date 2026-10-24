# Contribuția la OSATURA

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/CONTRIBUTING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/CONTRIBUTING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/CONTRIBUTING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/CONTRIBUTING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/CONTRIBUTING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/CONTRIBUTING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/CONTRIBUTING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/CONTRIBUTING.md)

* **Regula implementării independente.** Contribuie doar cu cod, matematică și formate pe care ai voie să le împarți legal. Nu copia din unelte, fișiere sau interfețe proprietare, nu le decompila și nu face inginerie inversă pe ele, și nu lipi cod din proiecte a căror licență nu este compatibilă (vezi CLEANROOM.md). Nu citi și nu lipi cod sursă al uneltelor concurente în timp ce scrii o funcționalitate. Citează standardul public sau manualul după care ai implementat.
* **Licența contribuțiilor (inbound = outbound).** O contribuție este acceptată sub licența părții pe care o modifică: MPL-2.0 pentru `osatura-core`, `-capi`, `-wasm`, `-gpu` și `scripts/`; AGPL-3.0-or-later pentru `osatura-cli`; CC-BY-4.0 pentru documentație și resurse sintetice. Fiecare fișier sursă poartă o linie SPDX care trebuie să fie egală cu `license` din crate-ul său (verificat de `osatura-cli/tests/spdx_audit.rs`).
* **Sign-off-ul nu acordă niciun drept de relicențiere.** Sign-off-ul de mai jos certifică doar proveniența. **O licență comercială sau duală (de exemplu o licență proprietară pentru cei care integrează) NU este posibilă în prezent pentru codul contribuit**: ar necesita un Contributor Licence Agreement separat sau o cesiune outbound explicită din partea fiecărui contributor, iar niciuna nu există. Până când va exista una, nimeni (inclusiv mentenanța) nu poate pretinde că are dreptul de a relicenția contribuțiile altora.
* **Sign-off (DCO).** Fiecare commit trebuie să poarte `Signed-off-by: Your Name <email>` (`git commit -s`), certificând Developer Certificate of Origin 1.1, reprodus mai jos.
* Adaugă teste; rulează `cargo test --workspace` și `scripts/licence-audit.sh` înainte de a deschide un pull request.

## Developer Certificate of Origin 1.1 (verbatim)

*Reprodus nemodificat de la https://developercertificate.org/ (Copyright (C) 2004, 2006 The Linux Foundation and its contributors); propriii termeni permit copii verbatim. Acest text nu este acoperit de CC-BY-4.0 al restului acestui fișier.*

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
