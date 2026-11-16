<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Politique de signature du code

Signature de code gratuite fournie par [SignPath.io](https://about.signpath.io/), certificat de la [SignPath Foundation](https://signpath.org/).

*État (25 oct. 2026) : le projet a demandé, ou s’apprête à demander, un certificat gratuit de la SignPath Foundation. Tant qu’une version n’indique pas le contraire, les fichiers Windows ne sont **pas** signés Authenticode ; chaque fichier de version porte déjà une signature Sigstore et une attestation de provenance de build GitHub (voir [Vérification](#vérification)).*

## Ce qui est signé

Seuls sont signés les fichiers construits à partir de ce dépôt ([github.com/chiuta/osatura](https://github.com/chiuta/osatura)) par son workflow de publication ([`.github/workflows/release.yml`](../../../.github/workflows/release.yml)) sur des runners hébergés par GitHub. Rien de ce qui est construit sur un ordinateur personnel n’est jamais signé, et aucun binaire tiers n’est signé.

| Fichier (dans `osatura-windows-x86_64.zip`) | Ce que c’est | Licence |
|---|---|---|
| `osatura.exe` | outil en ligne de commande et studio dans le navigateur | AGPL-3.0-or-later |
| `osatura-studio.exe` | le studio dans une fenêtre native (WebView2) | AGPL-3.0-or-later |
| `lib/osatura_capi.dll` | bibliothèque C | MPL-2.0 |

Chaque fichier signé porte le nom de produit `OSATURA` et la version de la publication dans ses propriétés (intégrés par `build.rs` dans chaque crate et vérifiés par le workflow avant toute signature). La configuration d’artefact SignPath est conservée dans [`.signpath/artifact-configuration.xml`](../../../.signpath/artifact-configuration.xml).

## Rôles de l’équipe

Le projet n’a actuellement qu’un seul mainteneur, qui détient les trois rôles :

| Rôle | Qui |
|---|---|
| Committers et relecteurs | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |
| Approbateurs | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |

* Les modifications de toute personne sans droit d’écriture arrivent sous forme de pull requests et sont relues par un committer avant d’être fusionnées ([CONTRIBUTING.md](CONTRIBUTING.md)).
* Une demande de signature n’est créée que par le workflow de publication pour un tag de version (`vMAJOR.MINOR.PATCH[-pre]`) dont le commit se trouve sur `main`, et chaque demande est approuvée manuellement dans SignPath par un approbateur.
* Toute personne ayant l’un de ces rôles est tenue d’utiliser l’authentification multifacteur pour GitHub et SignPath (condition de la SignPath Foundation ; le mainteneur la confirme pour chaque compte lors de l’octroi du certificat).
* Toute modification de ces rôles est consignée dans ce fichier.

## Politique de confidentialité

Ce programme ne transférera aucune information vers d’autres systèmes en réseau, sauf demande explicite de l’utilisateur ou de la personne qui l’installe ou l’exploite. Détails : [PRIVACY.md](PRIVACY.md).

## Modifications de votre système et désinstallation

OSATURA n’a pas d’installateur, n’écrit rien dans le registre et n’installe aucun service, pilote ni entrée de démarrage. Les quelques fichiers qu’il peut créer, et la manière de tout supprimer, sont listés dans [UNINSTALL.md](../../UNINSTALL.md).

## Licences

Tout le code est open source sous des licences approuvées par l’OSI : MPL-2.0 (bibliothèques) et AGPL-3.0-or-later (outil en ligne de commande, studio et application à fenêtre) ; la documentation est sous CC-BY-4.0 ([COPYRIGHT](../../../COPYRIGHT), [LICENSING.md](LICENSING.md)). Les fichiers signés ne contiennent aucun code propriétaire, et OSATURA n’est proposé sous aucune licence commerciale ou propriétaire (pas de double licence).

## Vérification

* **Authenticode** (une fois disponible) : clic droit sur un fichier dans l’Explorateur Windows > Propriétés > Signatures numériques ; le signataire est SignPath Foundation.
* **Origine de chaque fichier de publication** : `gh attestation verify <file> -R chiuta/osatura`, ou `cosign verify-blob` avec le lot `.sigstore.json` ([VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md)).

## Signaler un problème

Si vous pensez qu’un fichier signé pour OSATURA est malveillant, n’a pas été construit à partir de ce dépôt, ou que cette politique n’a pas été respectée, écrivez à <alexio@trom.tf> ou ouvrez un ticket ; les signalements de sécurité suivent [SECURITY.md](SECURITY.md). Les problèmes concernant le certificat lui-même peuvent aussi être signalés à la [SignPath Foundation](https://signpath.org/).
