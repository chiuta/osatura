<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA

**Un moteur de rigging et d'animation vectoriels 2D en Rust : os, IK, skinning, déformateurs, un runtime de machines à états, une boîte à outils en ligne de commande, un studio dans le navigateur et des bibliothèques intégrables, réunis dans un seul workspace.** Version roumaine : [README.ro.md](README.ro.md).

![Un personnage de profil (tête, oreille, queue de cheval, tronc, bras avec doigts, jambes, pieds) marche sur place sur un sol qui défile ; sortie réelle de osatura export sur le projet de démonstration intégré](docs/img/walk.gif)

*Ce que ça produit : la personne qui marche intégrée, exportée avec `osatura demo-person` et `osatura export` (tout le projet tient dans un fichier JSON). À essayer en deux minutes : [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).*

## Résumé et philosophie

OSATURA vous permet de construire un personnage ou une scène à partir de formes vectorielles et d'images, de les rigger avec des os et des contraintes, de les animer avec des clés, des clips et de la physique, puis de les exporter en GIF, en vidéo, en Lottie, en glTF, en lecteur HTML autonome ou en runtime interactif. Il est composé de petites briques déterministes : le même projet et la même seed donnent les mêmes pixels, chaque outil lit et écrit un projet JSON documenté, et chaque commande fonctionne depuis un terminal ; l'animation peut ainsi être scriptée, relue, comparée (diff) et testée comme du code source.

Principes qui structurent le projet :

* **Implémentation indépendante.** Écrit à partir de spécifications publiques, d'articles scientifiques et de mathématiques de manuel. Les listes de fonctionnalités de la concurrence ont été consultées pour le *périmètre* ; aucun code source tiers n'a été copié, et le projet ne lit ni n'écrit aucun format propriétaire. Ce n'est pas un processus clean-room strict (voir [CLEANROOM.md](docs/i18n/fr/CLEANROOM.md), y compris sa divulgation des recherches) et il n'est ni affilié à Moho ni à aucun autre produit commercial, ni dérivé de ceux-ci, ni compatible avec eux. Le code a été en grande partie écrit avec un modèle d'IA.
* **Un état des lieux honnête.** Chaque fonctionnalité est décrite avec ce qu'elle ne fait *pas* ([docs/LIMITATIONS.md](docs/i18n/fr/LIMITATIONS.md), [ROADMAP.md](docs/i18n/fr/ROADMAP.md) section 9).
* **Sûr par défaut.** Aucune panique sur les entrées, contrôles des valeurs finies, plafonds de ressources, écritures atomiques, chemins des assets confinés au dossier du projet ([docs/SECURITY.md](docs/i18n/fr/SECURITY.md)).
* **Formats ouverts d'abord.** OpenRaster, OCA, SVG, Lottie, glTF, APNG, WebM, EXR ; aucun format de fichier propriétaire, par conception.
* **Faible empreinte de dépendances**, avec une politique de licences vérifiée par des tests et par `cargo deny`.

## État d'avancement

*Nom : le projet s'appelait ARMATURA jusqu'au 24 oct. 2026 et a été renommé OSATURA après une recherche de marque ([docs/LIMITATIONS.md](docs/i18n/fr/LIMITATIONS.md) L-78). Le dépôt et l'adresse du site ont également été renommés (les anciens liens du dépôt redirigent).*

> **Alpha. Premiers builds préparés comme préversion en brouillon : `v0.1.0-alpha.3`.** Les exécutables pour Windows x86-64, macOS arm64 et Linux x86-64 ainsi que le module WebAssembly sont construits par GitHub Actions et joints à une préversion *en brouillon* sur <https://github.com/chiuta/osatura/releases> (un brouillon n'est visible que de son propriétaire tant qu'il n'a pas été publié à la main). Tous les crates sont toujours en version `0.1.0` ; « v0.5 / v0.6 / v0.7 » sont des *niveaux de fonctionnalités* dans l'historique, pas des numéros de version ([CHANGELOG.md](docs/i18n/fr/CHANGELOG.md)).
>
> | Vérifié | Non vérifié |
> |---|---|
> | **En local (Linux x86-64, 24 oct. 2026) :** 1338 tests de `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` plus 19 doc-tests passent (le crate GPU a besoin d'un adaptateur et le crate de fenêtre d'un bureau ; auparavant, le total incluant le crate GPU était de 1368). **Sur GitHub Actions (24 oct. 2026, chaîne d'outils 1.97.0) :** la suite de tests et `clippy -D warnings` passent sous Linux, macOS (arm64) et Windows ; `cargo deny` et `reuse lint` passent ; le module WebAssembly se compile pour `wasm32-unknown-unknown` et le test du wrapper passe sur le vrai module sous Node 20 ; les binaires de release passent un test de fumée (`--version`, `demo`, `check`, `render`) et `osatura-studio --selftest` s'exécute sous Windows et macOS. Quatre tests de fumée du studio avec Chromium sans interface passent ; la CLI, la façade Rust et l'ABI C (y compris sous ASan/UBSan) ont été exécutées avec les exemples de la documentation | **Personne n'a utilisé les exécutables sur un vrai bureau Windows ou macOS** : le lanceur par double-clic de Windows et l'application à fenêtre `osatura-studio` (WebView2 / WKWebView) n'ont été que compilés et auto-testés en CI ; les binaires **ne sont pas signés** (SmartScreen « Éditeur inconnu », Gatekeeper de macOS) ; le binaire Linux n'a été construit que sur un runner Ubuntu ; **le module WebAssembly n'a jamais été exécuté dans un navigateur** et sa vitesse comme sa mémoire n'ont pas été mesurées ; le studio n'a jamais été utilisé par une personne en production, ni dans un autre navigateur que Chromium ; aucun vrai GPU (seulement un adaptateur logiciel), aucune webcam, photographie ou capture de mouvement réelle n'a jamais été utilisée ; aucun audit de sécurité, aucune revue juridique, aucune recherche de brevets |
>
> Lisez [docs/LIMITATIONS.md](docs/i18n/fr/LIMITATIONS.md) (78 limites numérotées, avec gravité et contournement) avant de dépendre de quoi que ce soit.

## Téléchargement

Les archives précompilées, une fois que le propriétaire a publié la version en brouillon, se trouvent sur la [page des versions](https://github.com/chiuta/osatura/releases) du projet :

| Archive | Contenu |
|---|---|
| `osatura-windows-x86_64.zip` | `osatura.exe` (ligne de commande + serveur du studio), `osatura-studio.exe` (application à fenêtre), bibliothèque C et en-tête, licences, mentions des composants tiers |
| `osatura-macos-arm64.tar.gz` | `osatura`, `osatura-studio` (Apple Silicon uniquement), bibliothèque C et en-tête, licences, mentions |
| `osatura-linux-x86_64.tar.gz` | `osatura`, bibliothèque C et en-tête, licences, mentions (pas d'application à fenêtre : utilisez `osatura studio` et un navigateur) |
| `osatura-web.tar.gz` | `osatura_wasm.wasm`, `osatura.js`, page de démonstration `index.html` (non vérifiée dans des navigateurs, voir [docs/WASM.md](docs/i18n/fr/WASM.md)) |
| `SHA256SUMS.txt` | vérifiez chaque téléchargement avec ce fichier |

* **Windows :** double-cliquez sur `osatura.exe` pour créer `Documents\OSATURA\first-rig.json` et ouvrir le studio dans votre navigateur, ou double-cliquez sur `osatura-studio.exe` pour obtenir une fenêtre à part. Les deux programmes ne sont pas signés : dans SmartScreen, choisissez *Informations complémentaires* puis *Exécuter quand même*. Voir [docs/GETTING_STARTED.md](docs/i18n/fr/GETTING_STARTED.md).
* **macOS :** approuvez le programme dans *Réglages Système > Confidentialité et sécurité* la première fois.
* **Source :** chaque archive indique le commit dont elle provient ; les parties AGPL sont servies avec leur offre de code source (`osatura --version`, `/source`).

## Fonctionnalités par domaine

Détails et options exactes : [docs/CLI_REFERENCE.md](docs/i18n/fr/CLI_REFERENCE.md) ; ce qui est partiel ou manquant : [docs/LIMITATIONS.md](docs/i18n/fr/LIMITATIONS.md).

* **Modèle de dessin.** Tracés vectoriels, remplissages, contours (tirets, largeur variable), dégradés linéaires et radiaux, masques, modes de fusion, effets par calque (flou, ombre, teinte, remplissage), calques d'image avec déformation par maillage, metaballs liquides, opérations booléennes sur polygones, particules déterministes, caméra multiplan avec plans enregistrés, profondeur de champ.
* **Rigging.** Os avec cinématique directe ; IK à deux os, souple, étirable et FABRIK ; limites, contraintes, ressorts ; skinning avec outils de poids ; déformateurs lattice, spine, morph, wobble et cage ; tracé de maillage à partir de l'alpha ; pivots de déformation et glue ; drivers (curseurs, joysticks, permutation de vue) ; jeux de skins ; auto-rig à partir des noms de calques ; écrêtage des yeux ; conteneurs de mise en page et zones de défilement.
* **Animation.** Clés avec interpolation de Bézier/pas/ressort/rebond, expressions et canaux pilotés, calques de commutation, takes et pile de clips en couches, conversion en boucle et sur place, verrouillage des pieds, aimants, suivi de trajectoire, générateurs d'idle et de chevauchement, physique de chaîne/cheveux/tissu, lip-sync à partir d'un fichier audio ou d'une transcription, mouvements de tête synchronisés à la parole, retargeting BVH, entrées en direct (MIDI, souris, manette, points de repère de webcam que vous fournissez), pistes audio avec rognage, fondu et mixage.
* **Runtime.** Machines à états avec blend trees 1D, liaison de données, physique de défilement, niveau de détail avec borne d'erreur, un bac à sable de script Rhai, un serveur d'agent (MCP) fondé sur des opérations de patch.
* **Sortie.** SVG, PNG, séquences PNG et sprite sheets, GIF, APNG, MP4/WebM (via ffmpeg), lecteur HTML autonome, Lottie et dotLottie, glTF 2.0 binaire, OpenEXR/PNG 16 bits, OCA ; export ciblé sur une taille (`export-fit`). Import : SVG, OpenRaster, Lottie, BVH, séquences PNG/GIF/APNG.
* **Outils de qualité.** `check`, `lint`, `diff` sémantique, fuzzing de poses avec cartes de chaleur, CI d'animation avec diff perceptuel et images de référence ([docs/CI.md](docs/i18n/fr/CI.md)), gestion de versions avec merge sémantique (`vcs`), contrôles de sécurité pour les flashs et les mouvements.
* **Studio.** Éditeur dans le navigateur servi par la CLI : manipulateurs sur le canevas, timeline, dope sheet et éditeur de courbes, surcouches, palette de commandes et raccourcis, annuler/rétablir, édition multi-utilisateur en temps réel avec jeton optionnel et journal persistant ([docs/STUDIO.md](docs/i18n/fr/STUDIO.md)).
* **Bibliothèques.** Façade Rust stable ([docs/API.md](docs/i18n/fr/API.md)), ABI C ([docs/C_ABI.md](docs/i18n/fr/C_ABI.md)), module WebAssembly et wrapper JavaScript (compilés et testés sous Node en CI, pas encore exécutés dans un navigateur ; [docs/WASM.md](docs/i18n/fr/WASM.md)), moteur de rendu wgpu optionnel.

## Démarrage rapide

Prérequis : une chaîne d'outils Rust stable récente (ou une archive précompilée de la section [Téléchargement](#téléchargement) : sautez la ligne `cargo build` et utilisez le `osatura` décompressé). `ffmpeg` n'est nécessaire que pour l'export MP4/WebM et pour l'audio mp3/ogg ; il n'est jamais fourni avec le projet. Chaque commande ci-dessous a été exécutée pour ce README (build de débogage, Linux). Les builds de release (`cargo build --release`) sont bien plus rapides pour le rendu et s'utilisent de la même façon avec `target/release`.

```sh
cargo build -p osatura-cli                       # builds target/debug/osatura
export PATH="$PWD/target/debug:$PATH"             # so that `osatura` is found

osatura --version                                # version, licence, source offer, support links
osatura demo demo.json                           # write the built-in rig demo (640x480, 48 frames)
osatura check demo.json                          # validate: ok: 'OSATURA demo' 640x480 @ 24 fps, 48 frames, 6 bones...
osatura lint demo.json                           # static rig/animation checks: 0 finding(s), 0 error(s)
osatura render demo.json 12 frame12.png --bones  # one frame, with the skeleton overlay
osatura render demo.json 12 frame12.svg          # the same frame as vector output
osatura frames demo.json frames_out --png        # every frame as a PNG
osatura export demo.json demo.gif                # also .html (player), .json (Lottie), .lottie, .mp4 (ffmpeg)
osatura demo-fx fx.json                          # second demo: liquid, booleans, particles
osatura lipsync demo.json voice.wav mouth out.json   # mouth shapes from a WAV file (switch layer "mouth")
osatura studio demo.json --port 8080             # browser editor at http://127.0.0.1:8080
osatura ci init demo.json && osatura ci run     # animation CI: goldens + checks (about 40 s in a debug build)
```

`voice.wav` est un fichier WAV mono ou stéréo quelconque que vous fournissez. Le studio n'écoute que sur `127.0.0.1` et refuse les requêtes destinées à d'autres noms d'hôte ; pour le partager, lisez d'abord [docs/SECURITY.md](docs/i18n/fr/SECURITY.md).

Utiliser les bibliothèques :

```sh
cargo build -p osatura-capi                      # target/debug/libosatura_capi.so and .a
gcc -std=c99 -o smoke osatura-capi/tests/smoke.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./smoke demo.json    # abi 1, 48 frames, 320x240, rc=0, checksum=...
```

La façade Rust est `osatura_core::prelude` ([docs/API.md](docs/i18n/fr/API.md)) ; d'autres exemples en C et en Python figurent dans [docs/C_ABI.md](docs/i18n/fr/C_ABI.md).

Tests (environ 14 minutes en build de débogage sur une machine partagée ; voir [docs/DEVELOPMENT.md](docs/i18n/fr/DEVELOPMENT.md)) :

```sh
cargo test --workspace
cargo test --doc -p osatura-core
cargo test -p osatura-cli --test spdx_audit
```

## Structure du dépôt

| Chemin | Contenu | Licence |
|---|---|---|
| `osatura-core/` | moteur, formats, outils, façade stable `api` | MPL-2.0 |
| `osatura-cli/` | le binaire `osatura`, le studio et le serveur de collaboration, tests de bout en bout et tests de fumée | AGPL-3.0-or-later |
| `osatura-gui/` | `osatura-studio`, le studio dans une fenêtre native (Windows et macOS ; une coquille vide sous Linux) | AGPL-3.0-or-later |
| `osatura-capi/` | ABI C (`osatura.h`, bibliothèque partagée et statique) | MPL-2.0 |
| `osatura-wasm/` | ABI WebAssembly, wrapper JavaScript, page de démonstration (compilé en CI) | MPL-2.0 |
| `osatura-gpu/` | moteur de rendu wgpu optionnel (pas un membre par défaut) | MPL-2.0 |
| `docs/` | documentation (CC-BY-4.0) et fichiers tiers générés | CC-BY-4.0 |
| `.github/workflows/` | CI (`ci.yml`) et builds de release (`release.yml`) | MPL-2.0 |
| `scripts/` | `e2e.sh`, `licence-audit.sh` | MPL-2.0 |
| `LICENSES/`, `REUSE.toml`, `deny.toml` | textes des licences, métadonnées de licence, politique de dépendances | - |
| `ROADMAP.md`, `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `TRADEMARK.md`, `CONTRIBUTING.md`, `CHANGELOG.md` | documents du projet | CC-BY-4.0 |

## Carte de la documentation

Commencez par [docs/INDEX.md](docs/i18n/fr/INDEX.md) (tous les documents, publics visés, parcours de lecture).

| Page | Pour |
|---|---|
| [docs/GETTING_STARTED.md](docs/i18n/fr/GETTING_STARTED.md) | premier projet, rendu, export |
| [docs/CLI_REFERENCE.md](docs/i18n/fr/CLI_REFERENCE.md) | chaque commande et chaque option |
| [docs/STUDIO.md](docs/i18n/fr/STUDIO.md) | l'éditeur dans le navigateur |
| [docs/RIGGING_GUIDE.md](docs/i18n/fr/RIGGING_GUIDE.md) | construire des rigs |
| [docs/ANIMATION_GUIDE.md](docs/i18n/fr/ANIMATION_GUIDE.md) | animation, clips, physique, audio |
| [docs/PROJECT_FORMAT.md](docs/i18n/fr/PROJECT_FORMAT.md) | le format de projet JSON |
| [docs/EXPORT_FORMATS.md](docs/i18n/fr/EXPORT_FORMATS.md) | ce que chaque format conserve et ce qu'il perd |
| [docs/ARCHITECTURE.md](docs/i18n/fr/ARCHITECTURE.md) | conception et modules |
| [docs/GLOSSARY.md](docs/i18n/fr/GLOSSARY.md) | termes |
| [docs/FAQ.md](docs/i18n/fr/FAQ.md) | questions fréquentes |
| [docs/API.md](docs/i18n/fr/API.md) | façade Rust 1.4 |
| [docs/C_ABI.md](docs/i18n/fr/C_ABI.md) | interface C |
| [docs/WASM.md](docs/i18n/fr/WASM.md) | WebAssembly (compilé en CI, pas exécuté dans un navigateur) |
| [docs/CI.md](docs/i18n/fr/CI.md) | CI d'animation |
| [docs/DEVELOPMENT.md](docs/i18n/fr/DEVELOPMENT.md) | build, tests, conventions, release |
| [docs/SECURITY.md](docs/i18n/fr/SECURITY.md) | modèle de menaces et durcissement |
| [docs/LIMITATIONS.md](docs/i18n/fr/LIMITATIONS.md) | limites connues |
| [docs/PATENT_NOTES.md](docs/i18n/fr/PATENT_NOTES.md), [docs/THIRD_PARTY.md](docs/i18n/fr/THIRD_PARTY.md), [docs/THIRD_PARTY_NOTICES.md](docs/THIRD_PARTY_NOTICES.md) | brevets, licences des dépendances |
| [CHANGELOG.md](docs/i18n/fr/CHANGELOG.md), [ROADMAP.md](docs/i18n/fr/ROADMAP.md), [FEATURES.md](docs/i18n/fr/FEATURES.md) | historique, plan et état détaillé, matrice des fonctionnalités |
| [CLEANROOM.md](docs/i18n/fr/CLEANROOM.md), [LICENSING.md](docs/i18n/fr/LICENSING.md), [TRADEMARK.md](docs/i18n/fr/TRADEMARK.md), [CONTRIBUTING.md](docs/i18n/fr/CONTRIBUTING.md) | provenance, licences, politique du nom, règles de contribution |

## Résumé des licences

* **MPL-2.0** pour les bibliothèques `osatura-core`, `osatura-capi`, `osatura-wasm`, `osatura-gpu` et pour `scripts/` : vous pouvez les lier à votre propre programme, ouvert ou fermé ; les modifications apportées à ces fichiers doivent être publiées sous MPL.
* **AGPL-3.0-or-later** pour `osatura-cli` (la ligne de commande et le studio) et `osatura-gui` (la fenêtre qui l'entoure). Si vous le modifiez et laissez d'autres personnes interagir avec lui via un réseau, la section 13 vous oblige à leur proposer votre code source : `osatura studio --source-url URL` publie l'endroit où il se trouve ; `/source` et `osatura --version` portent cette offre.
* **CC-BY-4.0** pour la documentation et les assets synthétiques (y compris le modèle synthétique de détection de visage).
* **Vos projets et vos rendus restent les vôtres.**
* Le nom et le logo ne sont pas couverts par ces licences ([TRADEMARK.md](docs/i18n/fr/TRADEMARK.md)). Le statut vis-à-vis des brevets n'a pas été évalué ([docs/PATENT_NOTES.md](docs/i18n/fr/PATENT_NOTES.md)). Une licence commerciale ou double pour le code contribué n'est pas possible sans accord de licence de contributeur, et il n'en existe aucun.
* Crates tiers : 363 dans l'inventaire du 24 oct. 2026 (toutes plateformes et cibles confondues, ce qui inclut les crates propres à Windows / macOS / Linux de l'application à fenêtre ; un binaire donné en contient un sous-ensemble), tous permissifs sauf six crates MPL-2.0 à copyleft faible : `smartstring` derrière la fonctionnalité optionnelle `scripting`, et cinq (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`) que seule l'application à fenêtre `osatura-gui` peut faire intervenir ([docs/THIRD_PARTY.md](docs/i18n/fr/THIRD_PARTY.md)). ffmpeg n'est jamais fourni avec le projet.

Voir [LICENSE](LICENSE), [LICENSES/](LICENSES/) et [LICENSING.md](docs/i18n/fr/LICENSING.md). Rien de ceci ne constitue un avis juridique ; une revue par un juriste reste à faire.

## Contribuer

Les contributions sont les bienvenues, sous la licence de la partie qu'elles modifient. Lisez [CONTRIBUTING.md](docs/i18n/fr/CONTRIBUTING.md) (chaque commit nécessite une ligne `Signed-off-by`, DCO 1.1), [CLEANROOM.md](docs/i18n/fr/CLEANROOM.md) (implémenter uniquement à partir de sources publiques ; ne pas copier de code propriétaire ou sous licence incompatible) et [docs/DEVELOPMENT.md](docs/i18n/fr/DEVELOPMENT.md) (conventions, liste de contrôle pour un nouveau module, tests). Bonnes premières contributions : un correctif tiré de [docs/LIMITATIONS.md](docs/i18n/fr/LIMITATIONS.md), une exécution réelle de la liste de contrôle WebAssembly, un test sur un vrai bureau des exécutables Windows / macOS, un benchmark sur un vrai GPU.

Problèmes de sécurité : n'ouvrez pas de ticket public ; suivez la section 10 de [docs/SECURITY.md](docs/i18n/fr/SECURITY.md).

## Soutien

OSATURA est développé par Alexandru-Ionuț Chiuță (Alexio). Si le projet vous est utile, vous pouvez soutenir ce travail :

* Buy Me a Coffee : <https://buymeacoffee.com/echo.of.the.strings>
* Patreon : <https://patreon.com/c/alexio_tf>

Contact du mainteneur (également affiché par `osatura --version`) : alexio@trom.tf.

## Remerciements

* Les auteurs des articles et manuels publics que suivent les algorithmes, cités module par module dans [CLEANROOM.md](docs/i18n/fr/CLEANROOM.md) (mathématiques des courbes de Bézier et des splines, cinématique inverse, skinning par mélange linéaire, coordonnées à valeur moyenne, ajustement de courbes et autres).
* Les spécifications ouvertes qu'OSATURA lit et écrit : SVG, OpenRaster, OCA, Lottie/dotLottie, glTF, APNG, WebM, OpenEXR, BVH, PNG, GIF.
* Les crates Rust sur lesquels il repose, listés avec leurs licences dans [docs/THIRD_PARTY.md](docs/i18n/fr/THIRD_PARTY.md), notamment `tiny-skia`, `serde`, `png`, `zip`, `rhai` et `wgpu`.
* Toutes les personnes qui soutiennent le projet via les liens ci-dessus.
