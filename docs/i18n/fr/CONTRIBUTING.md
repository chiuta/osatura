# Contribuer à OSATURA

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/CONTRIBUTING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/CONTRIBUTING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/CONTRIBUTING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/CONTRIBUTING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/CONTRIBUTING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/CONTRIBUTING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/CONTRIBUTING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/CONTRIBUTING.md)

* **Conduite.** Toute personne qui participe respecte le [Code de conduite](../../../CODE_OF_CONDUCT.md) ; pour citer OSATURA, voir [CITATION.cff](../../../CITATION.cff).
* **Règle d'implémentation indépendante.** Ne contribuez que du code, des mathématiques et des formats que vous avez légalement le droit de partager. Ne copiez pas, ne décompilez pas et ne faites pas de rétro-ingénierie d'outils, de fichiers ou d'interfaces propriétaires, et ne collez pas de code issu de projets dont la licence n'est pas compatible (voir CLEANROOM.md). Ne lisez ni ne collez le code source d'outils concurrents pendant que vous écrivez une fonctionnalité. Citez la norme publique ou le manuel à partir desquels vous avez implémenté.
* **Licence des contributions (entrant = sortant).** Une contribution est acceptée sous la licence de la partie qu'elle modifie : MPL-2.0 pour `osatura-core`, `-capi`, `-wasm`, `-gpu` et `scripts/` ; AGPL-3.0-or-later pour `osatura-cli` ; CC-BY-4.0 pour la documentation et les ressources synthétiques. Chaque fichier source porte une ligne SPDX qui doit être égale à la `license` de sa crate (vérifié par `osatura-cli/tests/spdx_audit.rs`).
* **Le sign-off n'accorde aucun droit de relicenciement.** Le sign-off ci-dessous atteste uniquement l'origine. **Une licence commerciale ou double (par exemple une licence propriétaire pour les intégrateurs) n'est actuellement PAS possible pour le code contribué** : elle exigerait un Contributor Licence Agreement distinct ou une concession sortante explicite de chaque contributeur, et aucun n'est en place. Tant qu'il n'en existe pas, personne (y compris le mainteneur) ne peut prétendre être en mesure de relicencier les contributions d'autrui. Le projet n'offre aucune licence commerciale ou double ([CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md)).
* **Relecture.** Toute pull request d'une personne sans droit d'écriture est relue par un committer avant d'être fusionnée ; les builds de release signés ne proviennent que de code relu sur `main` ([CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md), rôles de l'équipe).
* **Sign-off (DCO).** Chaque commit doit porter `Signed-off-by: Your Name <email>` (`git commit -s`), attestant le Developer Certificate of Origin 1.1 reproduit ci-dessous.
* Ajoutez des tests ; exécutez `cargo test --workspace` et `scripts/licence-audit.sh` avant d'ouvrir une pull request.

## Developer Certificate of Origin 1.1 (verbatim)

*Reproduit sans modification depuis https://developercertificate.org/ (Copyright (C) 2004, 2006 The Linux Foundation and its contributors) ; ses propres conditions autorisent les copies verbatim. Ce texte n'est pas couvert par la CC-BY-4.0 du reste de ce fichier.*

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
