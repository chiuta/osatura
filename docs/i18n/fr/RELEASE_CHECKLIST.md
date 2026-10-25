<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Liste de contrôle de publication (côté propriétaire)

*Étapes que seul le propriétaire peut réaliser, dans l'ordre. État au 24 oct. 2026 : le dépôt <https://github.com/chiuta/osatura> existe, GitHub Actions est au vert (tests sous Linux, macOS et Windows, clippy, wasm32, cargo-deny, REUSE) et la préversion en brouillon `v0.1.0-alpha.3` a été construite (archives Windows, macOS et Linux, module web et `SHA256SUMS.txt`) ; **rien n'est encore publié**. Les étapes ci-dessous que la CI a déjà accomplies sont cochées ; les autres demandent une personne, un juriste ou un compte. Ceci n'est pas un avis juridique. Les éléments marqués **[bloquant]** doivent être traités avant toute annonce publique.*

## 1. Avant de publier quoi que ce soit

- [ ] **[bloquant]** Confirmez ou remplacez le contact de sécurité (`alexio@trom.tf` est un espace réservé dans `docs/SECURITY.md`, `--version` et les documents de licence).
- [ ] **[bloquant]** Recherchez le nom « OSATURA » dans les registres de marques qui vous concernent (EUIPO, USPTO, OSIM) ainsi que dans les boutiques d'applications / registres de paquets. Voir `TRADEMARK.md`. En cas de conflit, changez de nom avant la première publication (le nom apparaît dans le binaire, les crates, la documentation).
- [ ] **[bloquant]** Demandez à un juriste de relire `LICENSING.md`, la répartition MPL-2.0 / AGPL-3.0, `docs/THIRD_PARTY.md` et `docs/PATENT_NOTES.md`. Décidez si vous voulez une double licence ; si oui, un CLA est nécessaire *avant* d'accepter des contributions extérieures (aujourd'hui : DCO uniquement).
- [ ] Vérifiez les conditions de la référence d'ajustement de courbes de Graphics Gems (citée de mémoire dans `CLEANROOM.md`) ; ne conservez la formulation d'implémentation indépendante que si elle est exacte.
- [x] `reuse lint` et `cargo deny check` passent en CI (le job `licences`) ; exécutez-les aussi une fois sur votre machine.
- [ ] Vérifiez qu'aucune donnée personnelle, aucun chemin local ni aucun jeton ne figure dans l'arborescence (`grep -r "/home/" .`, `grep -ri token`), et que le modèle fourni et les ressources de démonstration sont tous auto-générés.

## 2. Dépôt

- [x] Le dépôt GitHub existe et l'arborescence a été poussée sans `target/` ni `out/` (`.gitignore`).
- [ ] Conservez `COPYRIGHT`, `LICENSES/`, `REUSE.toml`, `.github/FUNDING.yml` (Buy Me a Coffee `echo.of.the.strings`, Patreon `alexio_tf`).
- [ ] Activez le signalement privé des vulnérabilités (onglet Security) et faites-y pointer `docs/SECURITY.md`.
- [ ] Passez l'URL du dépôt avec `--source-url` lors de l'exécution du studio (offre de code source AGPL, route `/source`) et testez l'offre depuis une installation neuve.

## 3. Première exécution de la CI (faite le 2026-10-24)

- [x] `.github/workflows/ci.yml` est au vert : tests Linux, Windows, macOS et `clippy -D warnings` ; la tâche de compilation wasm32 ; la tâche cargo-deny et REUSE.
- [x] Les échecs de la première exécution sous Windows ont été corrigés (taille de pile, tests réservés à POSIX, accès au journal de collaboration) et consignés dans `docs/LIMITATIONS.md` (L-04).
- [x] La tâche wasm32 réussit (compilation et test Node avec le vrai module). **Reste ouvert :** exécutez la liste de vérification de `docs/WASM.md` dans un vrai navigateur et mettez à jour L-01 / L-02.

## 4. Tests humains

- [ ] Donnez le studio à 3 à 5 animateurs pendant une semaine avec une courte tâche (rigger un personnage, réaliser un cycle de marche, exporter un GIF). Notez où ils bloquent ; ouvrez une issue pour chaque cas.
- [ ] Testez sur de vrais navigateurs (Chrome, Firefox, Safari), un appareil tactile et une session au clavier uniquement.
- [ ] Sur un vrai **Windows 11** et un vrai **Mac** : décompressez l'archive, double-cliquez sur `osatura.exe` (Windows) et sur `osatura-studio`, rigguez et exportez un projet, et signalez ce qui échoue (L-74 à L-76).
- [ ] Essayez le détecteur de visages / la mocap / les fonctions vocales avec des données réelles ; indiquez honnêtement qu'elles sont limitées (L-12, L-13).

## 5. Sécurité du déploiement (si vous hébergez le studio ou la collaboration)

- [ ] Placez-le derrière un reverse proxy qui termine le TLS (recette : `docs/SECURITY.md` section 12) ; n'exposez jamais le port HTTP simple à Internet.
- [ ] Utilisez des jetons par participant (`collab-token`), définissez une expiration, liez à localhost derrière le proxy.
- [ ] Commandez une revue de sécurité indépendante avant de proposer la collaboration hébergée à des inconnus.

## 6. Artefacts de publication

- [x] Les binaires de release sont construits sur des runners hébergés par GitHub (`release.yml`) et `SHA256SUMS.txt` est généré dans le brouillon. **Reste ouvert :** relisez le brouillon `v0.1.0-alpha.3`, cochez « pre-release », publiez-le et étiquetez le commit (le tag est créé avec la version).
- [ ] Décidez de la signature du code (certificat Authenticode pour Windows, Apple Developer ID et notarisation) ; d'ici là, les téléchargements sont signalés comme non signés (L-74).
- [ ] Joignez le code source correspondant pour les parties AGPL (l'étiquette elle-même suffit ; gardez-le disponible aussi longtemps que vous distribuez des binaires).
- [x] `docs/THIRD_PARTY_NOTICES.md` est régénéré (24 oct. 2026) et chaque archive le contient ; régénérez-le encore avec `scripts/licence-audit.sh` avant chaque version ultérieure.
- [ ] Crates.io / npm / registres de paquets : vérifiez que les noms sont libres et que les champs de licence correspondent à `LICENSING.md`.

## 7. Annonce et dons

- [ ] Premier écran du README : ce qui fonctionne, ce qui ne fonctionne pas (lien vers `docs/LIMITATIONS.md`) ; ne prétendez pas être « de classe Moho » sans nuance.
- [ ] Montrez les captures d'écran de `docs/img/` et un court GIF réalisé avec le projet de démonstration.
- [ ] Ajoutez les liens de dons (Buy Me a Coffee, Patreon) au README et à la boîte À propos du studio ; ne promettez pas de fonctionnalités en échange de dons à moins de pouvoir tenir la promesse.
- [ ] Publiez là où se trouvent les animateurs (communautés de rigging 2D / d'animation indépendante) ; demandez des rapports de bogues, pas des compliments.
- [ ] Réservez du temps pour répondre rapidement aux premières issues ; les deux premières semaines décident si les contributeurs restent.
