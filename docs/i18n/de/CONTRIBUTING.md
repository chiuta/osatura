# Zu OSATURA beitragen

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/CONTRIBUTING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/CONTRIBUTING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/CONTRIBUTING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/CONTRIBUTING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/CONTRIBUTING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/CONTRIBUTING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/CONTRIBUTING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/CONTRIBUTING.md)

* **Verhalten.** Alle Beteiligten halten sich an den [Verhaltenskodex](../../../CODE_OF_CONDUCT.md); zum Zitieren von OSATURA siehe [CITATION.cff](../../../CITATION.cff).
* **Regel der unabhängigen Implementierung.** Steuere nur Code, Mathematik und Formate bei, die du rechtmäßig weitergeben darfst. Kopiere nichts aus proprietären Werkzeugen, Dateien oder Oberflächen, dekompiliere oder reverse-engineere sie nicht, und füge keinen Code aus Projekten ein, deren Lizenz nicht kompatibel ist (siehe CLEANROOM.md). Lies oder füge keinen Quellcode konkurrierender Werkzeuge ein, während du ein Feature schreibst. Nenne den öffentlichen Standard oder das Lehrbuch, nach dem du implementiert hast.
* **Lizenz der Beiträge (inbound = outbound).** Ein Beitrag wird unter der Lizenz des Teils angenommen, den er ändert: MPL-2.0 für `osatura-core`, `-capi`, `-wasm`, `-gpu` und `scripts/`; AGPL-3.0-or-later für `osatura-cli`; CC-BY-4.0 für Dokumentation und synthetische Assets. Jede Quelldatei trägt eine SPDX-Zeile, die der `license` ihres Crates entsprechen muss (geprüft von `osatura-cli/tests/spdx_audit.rs`).
* **Das Sign-off gewährt kein Recht zur Neulizenzierung.** Das untenstehende Sign-off bescheinigt nur die Herkunft. **Eine kommerzielle oder Dual-Lizenz (zum Beispiel eine proprietäre Lizenz für Einbettende) ist für beigetragenen Code derzeit NICHT möglich**: Sie würde eine gesonderte Contributor Licence Agreement oder eine ausdrückliche Outbound-Einräumung jedes Beitragenden erfordern, und keine von beiden liegt vor. Solange es keine gibt, darf niemand (auch nicht der Maintainer) behaupten, die Beiträge anderer neu lizenzieren zu können. Das Projekt bietet überhaupt keine kommerzielle oder Dual-Lizenz an ([CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md)).
* **Review.** Jeder Pull Request von jemandem ohne Schreibrechte wird vor dem Merge von einem Committer geprüft; codesignierte Release-Builds stammen nur aus geprüftem Code auf `main` ([CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md), Rollen im Team).
* **Sign-off (DCO).** Jeder Commit muss `Signed-off-by: Your Name <email>` tragen (`git commit -s`) und damit das unten wiedergegebene Developer Certificate of Origin 1.1 bescheinigen.
* Füge Tests hinzu; führe vor dem Öffnen eines Pull Requests `cargo test --workspace` und `scripts/licence-audit.sh` aus.

## Developer Certificate of Origin 1.1 (wörtlich)

*Unverändert wiedergegeben von https://developercertificate.org/ (Copyright (C) 2004, 2006 The Linux Foundation and its contributors); seine eigenen Bedingungen erlauben wörtliche Kopien. Dieser Text fällt nicht unter die CC-BY-4.0 des Rests dieser Datei.*

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
