# OSATURA — licences (DÉCIDÉ : Variante A, appliquée le 22 oct. 2026 ; voir LICENSE et LICENSES/)

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/LICENSING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/LICENSING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/LICENSING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/LICENSING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/LICENSING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/LICENSING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/LICENSING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/LICENSING.md)

**Statut :** la Variante A ci-dessous a été choisie et appliquée : MPL-2.0 pour les bibliothèques (core, capi, wasm, gpu), AGPL-3.0-or-later pour `osatura-cli` (CLI + studio) et `osatura-gui` (le studio dans une fenêtre native), CC-BY-4.0 pour la documentation et les ressources/modèles synthétiques, politique de marque dans TRADEMARK.md, DCO dans CONTRIBUTING.md, en-têtes SPDX dans tous les fichiers .rs/.js/.mjs/.sh et REUSE.toml pour le reste, offre de code source de l'AGPL section 13 (`/source`, `--source-url`), champs `license` de Cargo renseignés. Actions restant au propriétaire : relecture par un juriste, recherche d'antériorité de la marque (EUIPO/OSIM), vérification des brevets (docs/PATENT_NOTES.md), un CLA si une double licence est souhaitée, inclusion des avis sur les composants tiers dans les versions binaires.


*Ceci n'est pas un avis juridique. La licence est DÉCIDÉE (Variante A, voir LICENSE) ; les sections ci-dessous sont conservées comme justification et comme alternatives qui ont été envisagées. Faites confirmer le choix par un juriste (idéalement quelqu'un qui connaît le droit d'auteur roumain/européen) avant la première publication. Contributions : aucun droit de relicencier ou de doubler la licence des contributions tierces n'existe tant qu'un CLA n'est pas en place (CONTRIBUTING.md).*

## 1. Ce que la licence doit faire pour vous
Vos revenus proviennent de **dons (Buy Me a Coffee)**, et non de redevances de licence. La licence ne crée donc pas l'argent : **la bonne volonté, la visibilité et la confiance le font**. Elle a trois missions :
1. **Maximiser le nombre d'utilisateurs** (plus d'utilisateurs → plus de soutiens) : elle doit être libre d'utilisation, y compris dans un cadre commercial.
2. **Empêcher une entreprise de prendre le code, de le fermer et de le vendre** (ou de l'héberger comme service payant) sans rien rendre en retour.
3. **Préserver votre droit de vendre des exceptions** (licence commerciale, support) et de défendre le nom.

Elle doit aussi s'accorder avec vos convictions **trade-free / TROM** : le logiciel ne doit pas être bridé, financé par la publicité ni lié à la collecte de données. (Les licences qui interdisent l'usage commercial, p. ex. CC-NC ou Polyform-NC, ne sont *pas* open source et ne sont pas la même chose que « trade-free » ; vérifiez la formulation propre à TROM avant d'en adopter une.)

## 2. Constats qui contraignent le choix
- **Les dépendances ne sont pas un obstacle.** Audit de toutes les crates tierces (363 lors de la régénération du 24 oct. 2026 par `scripts/licence-audit.sh`, comptées toutes plateformes confondues après l'ajout de l'application à fenêtre ; il y en avait 160 le 22 oct. ; ce nombre change, voir docs/THIRD_PARTY.md) : toutes sont sous MIT, Apache-2.0, BSD, Zlib, ISC, Unlicense, CC0, Unicode ou des doubles licences de celles-ci, à l'exception de six crates MPL-2.0 à copyleft faible, au niveau du fichier : smartstring (MPL-2.0+, atteinte uniquement via la fonctionnalité `scripting`, ACTIVÉE par défaut dans `osatura-core` et `osatura-cli`) et `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors` et `option-ext` (atteintes uniquement via l'application à fenêtre `osatura-gui`) ; une compilation des bibliothèques avec `--no-default-features` ne contient aucune crate tierce MPL. Elles peuvent être combinées avec l'AGPL, la MPL ou une licence propriétaire ; la crate MPL exige que sa source et son avis soient conservés.
- **Titularité du droit d'auteur.** Une grande partie du code a été écrite avec Claude. Le droit d'auteur se rattache en général à la contribution créative *humaine* (recommandations du US Copyright Office ; le droit européen/roumain exige de même la création intellectuelle propre d'un auteur humain). Conséquences pratiques : (a) vous pouvez et devez placer le projet sous licence en tant qu'auteur/compilateur humain, mais (b) ne promettez pas l'exclusivité de chaque ligne, (c) conservez dans le dépôt vos propres notes de conception, sélections et modifications comme preuve de paternité, (d) un DCO ne certifie que l'origine ; pour relicencier plus tard des contributions (p. ex. double licence), un CLA ou une concession sortante explicite est nécessaire et n'est PAS en place.
- **Aucune licence ne peut forcer les dons.** Ne conditionnez jamais le code à un don ; un fork supprimerait simplement cette condition. Vendez de la *commodité et des extras*, pas une autorisation.

## 3. Ensemble recommandé — « Bibliothèque à cœur ouvert, application protégée » (toutes les variantes ci-dessous sont prêtes à être appliquées)

### ★ Variante A (recommandée) : bibliothèque MPL-2.0 + applications AGPL-3.0 + marque
| Élément | Licence | Pourquoi |
|---|---|---|
| `osatura-core` (moteur, évaluateur, formats, exporteurs) et tout futur runtime/WASM/C-ABI | **MPL-2.0** | Copyleft au niveau du fichier : chacun peut l'intégrer à des jeux/applications (fermés ou ouverts), mais les modifications de *nos fichiers* doivent être partagées. Comprend une **concession de brevet** explicite et une résiliation en cas de poursuite pour brevet. Maximise l'adoption, dont les dons ont besoin. |
| `osatura-cli`, `osatura-gui`, studio (navigateur/fenêtre) | **AGPL-3.0-or-later** | Empêche les forks fermés et les clones « OSATURA comme service web payant » ; les *projets et rendus* des utilisateurs restent entièrement les leurs. |
| Nom « OSATURA », logo | **Politique de marque** (fichier séparé) | Les forks doivent changer de nom ; protège les soutiens contre les imitations. Vérifiez la disponibilité (EUIPO/OSIM/OMPI) avant le lancement. |
| Rigs d'exemple, illustrations, audio que vous créez | **CC BY 4.0** (ou CC BY-SA) | Réutilisation libre avec attribution ; les illustrations tierces conservent leur propre licence. |
| Documentation | **CC BY 4.0** | |
<!-- REUSE-IgnoreStart -->
Cargo: `license = "MPL-2.0"` (core), `license = "AGPL-3.0-or-later"` (cli). SPDX header per file: `// SPDX-License-Identifier: MPL-2.0`.
<!-- REUSE-IgnoreEnd -->
**Double licence :** possible uniquement pour le code écrit par le seul propriétaire, ou après un CLA/une concession sortante de chaque contributeur. PAS possible aujourd'hui pour le code contribué (CONTRIBUTING.md).

### Variante B : AGPL-3.0-or-later partout (+ licence commerciale sur demande)
Position la plus forte contre le détournement et la plus proche de l'éthique trade-free. Coût : les entreprises et les moteurs de jeu éviteront d'intégrer le runtime, d'où moins d'utilisateurs et moins de soutiens indirects. À choisir si la protection compte plus que la portée.

### Variante C : Apache-2.0 partout (non retenue ; l'espace de travail applique la Variante A : bibliothèques MPL-2.0 et `license = "AGPL-3.0-or-later"` pour `osatura-cli`)
Portée maximale et aucune friction ; concession de brevet incluse. Coût : n'importe qui — y compris une grande entreprise — peut le fermer et le vendre sans aucune obligation. Les dons dépendent alors uniquement de la bonne volonté. À choisir seulement si vous voulez être un service d'utilité publique.

### Variante D : code source disponible (BUSL-1.1 ou Polyform Shield/NC avec date d'ouverture différée)
Empêche la concurrence commerciale pendant N années. Coût : **pas open source** ; en conflit avec le récit de la salle blanche/des formats ouverts, avec les contributions de la communauté et avec les valeurs trade-free ; de nombreux utilisateurs et distributions le refuseront. **Non recommandée.**

| | Portée | Anti-détournement | Concession de brevet | Revenu de double licence | Compatible trade-free |
|---|---|---|---|---|---|
| **A cœur MPL + applications AGPL** | élevée | bon | oui | oui | oui |
| B tout AGPL | moyenne | le plus fort | oui (v3) | oui | oui |
| C tout Apache | la plus élevée | aucun | oui | non | en partie |
| D code source disponible | faible | fort | s.o. | oui | non |

## 4. Comment gagner réellement de l'argent avec les dons (tout est compatible avec la Variante A)
1. **Page Buy Me a Coffee** (`buymeacoffee.com/echo.of.the.strings`) liée depuis le README, le texte de `--version` de la CLI, le pied de page du studio et la ligne de crédit du lecteur HTML (un crédit *amovible* — jamais obligatoire).
2. **Des adhésions, pas des paywalls :** paliers mensuels pour les versions anticipées, vote sur le prochain point de la feuille de route, nom dans `SUPPORTERS.md`, discussion privée. Tout le monde reçoit le même code ; les membres reçoivent du *temps et une voix*.
3. **Des extras payants qui ne sont pas le code :** binaires/installeurs signés prêts à l'emploi, packs de ressources (rigs, arrière-plans, bibliothèques de cycles de marche) sous une licence qui autorise l'usage dans des films/jeux mais pas la revente du pack lui-même, cours vidéo, ateliers.
4. **Commandes et support :** services de rigging, exporteurs sur mesure, conseil en intégration ; la double licence commerciale pour ceux qui l'intègrent.
5. **Paliers de parrainage pour les organisations** (logo dans le README) et candidatures à des fonds open source/créatifs (NLnet, programmes de type Sovereign Tech, fonds culturels) — ils financent précisément ce type d'infrastructure ouverte.
6. **Page de transparence :** publiez les revenus et ce qu'ils ont financé ; cela augmente de façon mesurable le soutien récurrent.
7. Gardez **Patreon** (`patreon.com/c/alexio_tf`) et BMC tous deux visibles, mais donnez à chacun un rôle distinct (BMC pour un remerciement ponctuel ; Patreon pour le récurrent) afin que les soutiens ne soient pas désorientés.

## 5. Liste de contrôle pour appliquer la Variante A (≈ 1 heure de travail une fois le choix fait)
- [x] `LICENSES/MPL-2.0.txt`, `LICENSES/AGPL-3.0-or-later.txt`, `LICENSES/CC-BY-4.0.txt`, `LICENSES/LicenseRef-third-party.txt` (organisation REUSE) plus le résumé `LICENSE` ; `REUSE.toml` couvre les fichiers non source.
- [x] `license =` renseigné dans les six fichiers `Cargo.toml` ; en-têtes SPDX dans tous les fichiers .rs/.js/.mjs/.sh, vérifiés par `osatura-cli/tests/spdx_audit.rs`.
- [x] `docs/THIRD_PARTY.md` et `docs/THIRD_PARTY_NOTICES.md`, tous deux régénérés par `scripts/licence-audit.sh`.
- [x] `CONTRIBUTING.md` avec le texte du DCO 1.1. Un CLA reste nécessaire avant toute double licence.
- [x] `TRADEMARK.md` (autorisé : « compatible avec OSATURA », redistribution non modifiée ; non autorisé : versions modifiées sous le même nom).
- [x] `.github/FUNDING.yml` (`buy_me_a_coffee: echo.of.the.strings`) pour que la plateforme affiche un bouton Sponsor.
- [~] Recherche de marque : le premier nom ARMATURA entrait en collision avec une marque des classes 9/42 (EUIPO, 24 oct. 2026), le projet a donc été renommé OSATURA ; la recherche EUIPO pour « osatura » n'a révélé aucun conflit (un résultat sans rapport, radié, en classe 10). Les noms de paquets `osatura` sur crates.io, npm et PyPI sont libres (vérifié le 24 oct. 2026), mais une marque proche, OSATRA (Open-Xchange AG, enregistrement américain 7459236, classes 9/38/42), existe. Les recherches OSIM / TMview / OMPI / noms de domaine et l'avis d'un juriste restent ouverts, voir [docs/LIMITATIONS.md](LIMITATIONS.md) L-78.
