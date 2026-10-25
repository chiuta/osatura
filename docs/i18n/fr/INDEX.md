<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Index de la documentation d'OSATURA

*Commencez ici pour trouver la bonne page. Tous les documents sont en Markdown anglais sous CC-BY-4.0, à l'exception des fichiers tiers générés (voir leurs en-têtes).*

OSATURA est un moteur de rigging et d'animation vectoriels 2D en Rust. C'est un projet alpha (les premiers builds sont préparés comme la préversion en brouillon `v0.1.0-alpha.3` ; les crates sont toujours en 0.1.0) ; lisez [LIMITATIONS.md](LIMITATIONS.md) avant de dépendre d'une fonctionnalité.

Sommaire : [Tous les documents](#tous-les-documents) - [Parcours de lecture par public](#parcours-de-lecture-par-public) - [Documents à la racine du dépôt](#documents-à-la-racine-du-dépôt) - [Où trouver la réponse à chaque question](#où-trouver-la-réponse-à-chaque-question) - [Conventions](#conventions-utilisées-dans-les-documents)

## Tous les documents

| Document | Ce que c'est | Public |
|---|---|---|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Installation, premier projet, premiers rendu et export, première session dans le studio | tout le monde, première lecture |
| [CLI_REFERENCE.md](CLI_REFERENCE.md) | Chaque commande `osatura`, option, code de sortie et exemple | utilisateurs, auteurs de scripts, auteurs de CI |
| [STUDIO.md](STUDIO.md) | L'éditeur dans le navigateur : panneaux, raccourcis, collaboration, aperçus de scrub | animateurs, riggers |
| [RIGGING_GUIDE.md](RIGGING_GUIDE.md) | Construire un rig : os, IK, skinning, déformateurs, drivers, machines à états, mise en page | riggers, artistes techniques |
| [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) | Clés, interpolation, clips et takes, physique, lip-sync, audio, choix d'export | animateurs |
| [TUTORIAL_SVG.md](TUTORIAL_SVG.md) | D'un dessin vectoriel à un GIF de marche : import, rig par script, export (les commandes et les sorties sont réelles) | artistes |
| [COMPARISON.md](COMPARISON.md) | Une comparaison honnête et datée avec d'autres outils d'animation 2D | évaluateurs |
| [VERIFY_DOWNLOADS.md](VERIFY_DOWNLOADS.md) | Vérifier les archives de release avec les sommes SHA-256 | tous ceux qui installent |
| [GOOD_FIRST_TASKS.md](GOOD_FIRST_TASKS.md) | Petites tâches pour une première contribution | nouveaux contributeurs |
| [PROJECT_FORMAT.md](PROJECT_FORMAT.md) | Le format de projet JSON, champ par champ | créateurs d'outils, riggers qui modifient le JSON |
| [EXPORT_FORMATS.md](EXPORT_FORMATS.md) | Ce que chaque format d'export et d'import conserve et ce qu'il perd | créateurs de pipelines, intégrateurs |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crates, modules, flux de données, conception de l'évaluation et du rendu | contributeurs, relecteurs |
| [GLOSSARY.md](GLOSSARY.md) | Termes utilisés dans le moteur, la documentation et la CLI | tout le monde |
| [FAQ.md](FAQ.md) | Réponses courtes aux questions fréquentes | tout le monde |
| [API.md](API.md) | La façade Rust stable (version 1.4.0) : appels, erreurs, limites, exemples | intégrateurs Rust |
| [C_ABI.md](C_ABI.md) | L'interface C (`osatura-capi`) : fonctions, règles de mémoire, exemples en C et en Python | intégrateurs C, C++, Python, Go, C# |
| [WASM.md](WASM.md) | Module WebAssembly et wrapper JavaScript. **État : compilé et testé sous Node en CI ; exécuté dans Chromium sans interface, pas encore dans Firefox ni Safari** ; comprend la liste de contrôle pour la première exécution dans un navigateur | développeurs web |
| [CI.md](CI.md) | CI d'animation (`osatura ci`) : manifeste, seuils, images de référence, rapports, exemples de pipelines | artistes techniques, ingénieurs de build |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Compilation, tests, conventions, liste de contrôle pour un nouveau module, liste de contrôle de release, débogage | contributeurs, mainteneurs |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | Étapes à la charge du propriétaire avant et après la publication du brouillon de version | propriétaire |
| [SECURITY.md](SECURITY.md) | Modèle de menaces, protections intégrées, lacunes connues, signalement, liste de contrôle de durcissement | relecteurs de sécurité, déployeurs |
| [LIMITATIONS.md](LIMITATIONS.md) | Liste numérotée des limites connues avec gravité, contournement et identifiant de feuille de route | toute personne qui évalue OSATURA |
| [PATENT_NOTES.md](PATENT_NOTES.md) | Note sur le statut vis-à-vis des brevets (pas un avis juridique) | licenciés, juristes |
| [THIRD_PARTY.md](THIRD_PARTY.md) | Inventaire généré des crates tiers et de leurs licences | licenciés, packageurs |
| [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) | Textes de licence générés des dépendances, à livrer avec les binaires | packageurs |

## Parcours de lecture par public

Chaque parcours est un ordre, pas une obligation. Arrêtez-vous quand vous avez ce qu'il vous faut.

**Nouvel utilisateur (qui veut le voir fonctionner).** [../README.md](../../../README.fr.md) (démarrage rapide) -> [GETTING_STARTED.md](GETTING_STARTED.md) -> [STUDIO.md](STUDIO.md) -> [FAQ.md](FAQ.md) -> [LIMITATIONS.md](LIMITATIONS.md).

**Rigger.** GETTING_STARTED -> [RIGGING_GUIDE.md](RIGGING_GUIDE.md) -> [PROJECT_FORMAT.md](PROJECT_FORMAT.md) (pour les champs que vous modifiez à la main) -> [GLOSSARY.md](GLOSSARY.md) -> [CLI_REFERENCE.md](CLI_REFERENCE.md) (les commandes `lint`, `check`, `fuzz`, `autorig`, `machine`) -> [CI.md](CI.md) pour protéger le rig.

**Animateur.** GETTING_STARTED -> STUDIO -> [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) -> [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (choisir le format de sortie) -> CLI_REFERENCE (`export`, `export-fit`).

**Développeur de pipelines ou d'outils (scripts, autres formats).** CLI_REFERENCE -> PROJECT_FORMAT -> EXPORT_FORMATS -> [CI.md](CI.md) -> [SECURITY.md](SECURITY.md) sections 4 à 7 (règles sur les assets et les fichiers).

**Intégrateur (qui embarque le moteur).** Vue d'ensemble d'[ARCHITECTURE.md](ARCHITECTURE.md) -> choisir la couche : Rust ([API.md](API.md)), C et autres langages ([C_ABI.md](C_ABI.md)), navigateur ([WASM.md](WASM.md), lisez d'abord l'encadré d'état) -> [LIMITATIONS.md](LIMITATIONS.md) sections 1 et 8 -> [SECURITY.md](SECURITY.md) section 8.

**Contributeur.** [../CONTRIBUTING.md](CONTRIBUTING.md) (DCO) -> [DEVELOPMENT.md](DEVELOPMENT.md) -> ARCHITECTURE -> [../CLEANROOM.md](CLEANROOM.md) -> [../ROADMAP.md](ROADMAP.md) section 9 (ce qui existe et ce qui n'existe pas) -> LIMITATIONS (choisissez quelque chose à corriger).

**Relecteur de sécurité ou déployeur.** [SECURITY.md](SECURITY.md) -> [DEVELOPMENT.md](DEVELOPMENT.md) section 3 (ce qui est testé) -> LIMITATIONS sections 2 et 11 -> `osatura-cli/src/http.rs`, `studio.rs`, `collab_srv.rs`, `osatura-core/src/assets.rs`, `fsutil.rs` -> limites de [CI.md](CI.md).

**Licencié, packageur ou juriste.** [../LICENSING.md](LICENSING.md) -> [../COPYRIGHT](../../../COPYRIGHT) et [../LICENSES/](../../../LICENSES/) -> [THIRD_PARTY.md](THIRD_PARTY.md) -> [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) -> [PATENT_NOTES.md](PATENT_NOTES.md) -> [../TRADEMARK.md](TRADEMARK.md) -> [../CLEANROOM.md](CLEANROOM.md) -> liste de contrôle de release dans [DEVELOPMENT.md](DEVELOPMENT.md) section 7.

**Mainteneur qui prépare une release.** DEVELOPMENT section 7 -> [../CHANGELOG.md](CHANGELOG.md) -> LIMITATIONS (mise à jour) -> encadré d'état du README.

## Documents à la racine du dépôt

| Fichier | Rôle |
|---|---|
| [../README.md](../../../README.fr.md), [../README.ro.md](../../../README.fr.md) | Vue d'ensemble, état d'avancement, démarrage rapide, résumé des licences (anglais et roumain) |
| [../CHANGELOG.md](CHANGELOG.md) | Historique reconstitué ; format Keep-a-Changelog |
| [../ROADMAP.md](ROADMAP.md) | Plan ordonné ; **la section 9 est le journal daté de ce que fait et ne fait pas chaque élément** et fait autorité lorsque les documents divergent |
| [../FEATURES.md](FEATURES.md) | Matrice des fonctionnalités par rapport à d'autres outils (certaines parties sont périmées, voir LIMITATIONS) |
| [../CLEANROOM.md](CLEANROOM.md) | Journal de provenance et règles de l'implémentation indépendante |
| [../CONTRIBUTING.md](CONTRIBUTING.md) | Règles de contribution, licence entrante = licence sortante, DCO 1.1 |
| [../LICENSING.md](LICENSING.md), [../COPYRIGHT](../../../COPYRIGHT), [../LICENSES/](../../../LICENSES/) | Décision de licence (variante A), textes |
| [../TRADEMARK.md](TRADEMARK.md) | Politique du nom et du logo |
| `../REUSE.toml`, `../deny.toml` | Métadonnées de licence pour les fichiers non sources ; politique de dépendances |

## Où trouver la réponse à chaque question

| Question | Page |
|---|---|
| Comment l'installer et réaliser ma première animation ? | GETTING_STARTED |
| Que fait la commande X ? | CLI_REFERENCE |
| Que signifie ce champ du JSON ? | PROJECT_FORMAT, GLOSSARY |
| Quel format d'export conserve les masques / dégradés / l'audio ? | EXPORT_FORMATS |
| Puis-je l'utiliser depuis Python / C / le navigateur ? | C_ABI / WASM (état !) |
| Est-il sûr d'exposer le studio sur un réseau ? | SECURITY section 11 |
| Pourquoi X ne fonctionne-t-il pas ? Est-ce connu ? | LIMITATIONS, FAQ |
| Comment est-il compilé et testé ? | DEVELOPMENT |
| Puis-je l'utiliser commercialement ? Quelles sont les licences ? | README (licence), LICENSING, THIRD_PARTY |
| Qu'est-ce qui a changé ? | CHANGELOG, ROADMAP section 9 |

## Conventions utilisées dans les documents

* **« Vérifié »** signifie qu'une commande ou un programme a été exécuté pendant la rédaction des documents, dans un environnement Linux x86-64, sans GPU, sans webcam et avec seulement Chromium sans interface. **« Non testé »** signifie exactement cela. Les affirmations sur WebAssembly dans un vrai moteur sont toujours non vérifiées.
* Les exemples de code d'API.md et de C_ABI.md sont les programmes qui ont été compilés et exécutés ; ceux de WASM.md n'ont été exécutés que contre la maquette JavaScript.
* La section 9 de ROADMAP fait autorité sur ce que fait et ne fait pas une fonctionnalité. Si un document diverge de celle-ci ou du code, le code l'emporte ; signalez la différence.
* Les chemins sont relatifs à la racine du dépôt, sauf s'ils commencent par `docs/`.
* Les nombres (nombre de tests, limites) proviennent des sources ou d'une exécution du 22 oct. 2026 et peuvent être périmés après la prochaine modification ; les sources font foi.
