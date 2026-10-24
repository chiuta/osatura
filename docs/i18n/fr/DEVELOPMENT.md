<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Guide de développement

*Pour celles et ceux qui construisent, modifient, testent et publient OSATURA. Documentation CC-BY-4.0. Tout ce qui figure ici est tiré de l'arborescence des sources, de la section 9 de `ROADMAP.md`, de [CI.md](CI.md), de [CLEANROOM.md](CLEANROOM.md), de [CONTRIBUTING.md](CONTRIBUTING.md) et de [LICENSING.md](LICENSING.md) ; les commandes marquées « run » ont été exécutées pendant la rédaction de cette page (Linux x86-64, rustc 1.97.0, registre hors ligne).*

Sommaire : [Environnement](#1-environnement) - [Commandes de build et de test](#2-commandes-de-build-et-de-test) - [Les suites de tests](#3-les-suites-de-tests) - [Conventions de code](#4-conventions-de-code) - [Liste de contrôle pour un nouveau module ou une nouvelle commande](#5-liste-de-contrôle-pour-un-nouveau-module-ou-une-nouvelle-commande) - [Règles clean-room et DCO](#6-règles-clean-room-et-dco) - [Liste de contrôle de publication](#7-liste-de-contrôle-de-publication) - [Intégration continue](#8-intégration-continue) - [Débogage](#9-débogage) - [Profilage](#10-profilage)

## 1. Environnement

| Besoin | Pourquoi | Remarques |
|---|---|---|
| Rust (stable, workspace en édition 2021) | tout | développé avec rustc 1.97.0 ; `Cargo.lock` est versionné, les builds sont compatibles `--locked` |
| Un compilateur C (`cc`) | l'exemple/test de l'ABI C (facultatif) | voir [C_ABI.md](C_ABI.md) |
| `ffmpeg` / `ffprobe` dans le `PATH` | export vidéo, décodage audio de mp3/ogg, `dof-export` vers .mp4/.webm, `soundtrack` | **jamais fourni avec le projet ni nécessaire à la compilation** ; les commandes qui en ont besoin le signalent et échouent sans message de succès |
| Node.js >= 18 et `playwright-core` + Chromium | les quatre tests de fumée du studio ; le test du wrapper WASM | valeurs par défaut : `/opt/npm-tools/node_modules/playwright-core`, `/opt/pw-browsers/chromium-1194/chrome-linux/chrome` ; à remplacer avec `PLAYWRIGHT_CORE`, `CHROME` |
| `cargo-deny` | politique de licences/interdictions/sources | `scripts/licence-audit.sh` l'exécute s'il est installé (`--no-deny` l'ignore) |
| `python3` | `scripts/licence-audit.sh` génère la documentation avec un script en ligne | s'exécute avec `python3 -I` |
| Un adaptateur Vulkan ou GL | uniquement pour obtenir des mesures GPU réelles avec `osatura-gpu` | les tests se rabattent sur un adaptateur logiciel ; voir la section 2 |
| `rustup target wasm32-unknown-unknown`, `wasm-opt` | uniquement pour [WASM.md](WASM.md) | jamais installés dans l'environnement de développement ; la CI les installe |

Organisation du workspace (voir [ARCHITECTURE.md](ARCHITECTURE.md) pour la conception) :

| Crate | Licence | Rôle |
|---|---|---|
| `osatura-core` | MPL-2.0 | moteur, formats, outils, la façade `api` stable ([API.md](API.md)) ; features : `scripting` (par défaut, Rhai) |
| `osatura-cli` | AGPL-3.0-or-later | le binaire `osatura`, le serveur du studio, le serveur de collaboration ; features : `scripting` (par défaut), `gpu` |
| `osatura-capi` | MPL-2.0 | ABI C ([C_ABI.md](C_ABI.md)) |
| `osatura-wasm` | MPL-2.0 | ABI WebAssembly + wrapper JS ([WASM.md](WASM.md)) |
| `osatura-gpu` | MPL-2.0 | moteur de rendu wgpu facultatif ; **n'est pas** un membre par défaut |
| `osatura-gui` | AGPL-3.0-or-later | `osatura-studio`, le studio dans une fenêtre native (`tao` + `wry`, Windows et macOS uniquement ; une coquille vide ailleurs) ; **n'est pas** un membre par défaut, `publish = false` |

Fichiers au niveau du dépôt : `ROADMAP.md` (la section 9 est le journal daté des fonctionnalités et des revues), `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `REUSE.toml`, `deny.toml`, `LICENSES/`, `scripts/`.

## 2. Commandes de build et de test

```sh
cargo build                                  # default members: core, cli, capi, wasm (gpu is opt-in)
cargo test --workspace                       # run: 1338 tests + 19 doc-tests without gpu and gui (section 3)
cargo test -p osatura-core                  # one crate (the engine and the facade tests)
cargo test -p osatura-core api::            # a module or a name filter
cargo test -p osatura-cli --test e2e        # one integration test file
cargo test --doc -p osatura-core            # doc-tests only (the facade examples)
cargo test -p osatura-cli --test spdx_audit # SPDX header audit (0.01 s)
cargo clippy --workspace --all-targets -- -D warnings   # zero warnings (gated in CI since wave 8)
scripts/licence-audit.sh                     # regenerate THIRD_PARTY*, SPDX check, cargo deny
scripts/e2e.sh                               # the end-to-end pipeline in release mode (--debug for debug)
```

Features et parties facultatives :

```sh
cargo build -p osatura-cli --no-default-features         # no Rhai (no `script` command, no smartstring)
cargo build -p osatura-core --no-default-features        # what capi, wasm and gpu use
cargo build -p osatura-cli --features gpu                # adds `render-gpu` (wgpu)
cargo test  -p osatura-gpu                               # GPU tests (31: 18 unit + 13 parity); NOT in the workspace default
OSATURA_GPU_BACKEND=vulkan osatura render-gpu ...     # restrict the wgpu backend: vulkan or gl
```

Règles pratiques : utilisez des builds de débogage pour les suites de tests (le dépôt a été développé en économisant l'espace disque : ne copiez pas `target/`) ; n'utilisez `--release` que pour les questions de performance. L'instantané de la façade est validé avec `OSATURA_BLESS=1 cargo test -p osatura-core api::` (section 5). `OSATURA_UI_CONFIG` sélectionne le fichier de réglages de l'interface du studio dans les tests, `OSATURA_AUTHOR` le nom de l'auteur dans les commits de gestion de versions, `OSATURA_E2E_TIMINGS=<file>` fait écrire au test e2e ses durées par étape.

## 3. Les suites de tests

Les chiffres proviennent de la section 9 de `ROADMAP.md` et d'une exécution complète de `cargo test --workspace --offline` le 22 octobre 2026 pour cette page (profil de débogage, machine partagée avec d'autres tâches : les durées sont donc indicatives et **seront plus longues que sur une machine inactive**) :

| Suite | Tests | Durée (cette exécution) | Ce qu'elle couvre |
|---|---:|---:|---|
| `osatura-core` unitaires (`src/lib.rs`) | 1167 | 349 s | tous les modules ; inclut `api::tests::api_surface_snapshot` |
| `osatura-core/tests/prelude_surface.rs` | 6 | 0,7 s | la façade vue de l'extérieur de la crate (`#[non_exhaustive]`, correspondance des erreurs, un sous-ensemble représentatif d'appels) |
| `osatura-cli` unitaires (`src/main.rs`, studio, collab, http) | 80 | 42 s | analyse des options, USAGE face à l'analyseur, serveur HTTP, politique, journal de collaboration |
| `tests/e2e.rs` | 2 | 214 s | tout le pipeline via le vrai binaire (environ 19 s dans un build de débogage sur machine inactive et 40 s en release d'après la ROADMAP ; ici la machine était chargée) |
| `tests/fixes.rs`, `tests/wave4.rs`, `tests/robust.rs` | 7, 15, 10 | 19 s, 1,2 s, 1,2 s | régressions des vagues de revue et tests d'entrées hostiles |
| `tests/doc_mismatch.rs` | 20 | 1,8 s | régressions documentation/comportement : sorties atomiques, options strictes, matrice d'acceptation de `--json`/`--quiet`/`--allow-empty`, règles d'échelle de `golden` |
| `tests/source_offer.rs` | 3 | 0,2 s | offre de sources `/source` de l'AGPL section 13 |
| `tests/face_cli.rs` | 1 | 0,8 s | CLI de suivi de visage |
| `osatura-core/tests/wasm_safety.rs` | 2 | 0,1 s | aucun `Instant` / `SystemTime` / `process::id` / `thread::spawn` direct hors de `clock.rs` (analyse du source) |
| `osatura-cli/tests/wave7.rs` | 7 | 7 s | vague 7 : options par sous-commande, `follow` / `make-loop`, `color-space`, `collab-token`, fichier secret du studio, plage d'images de `dof` / `render-hdr` / `fx` |
| `tests/spdx_audit.rs` | 2 | 0,01 s | chaque `.rs/.js/.mjs/.sh` a une ligne SPDX égale à la licence de sa crate ; REUSE.toml couvre le reste |
| `osatura-capi/tests/abi.rs` | 7 | 1,1 s | l'ABI C côté Rust (test de fumée C : `osatura-capi/tests/smoke.c`, compilé à la main, voir [C_ABI.md](C_ABI.md)) |
| `osatura-wasm` unitaires | 8 | 1,6 s | les fonctions de l'ABI sur l'hôte |
| `osatura-gpu` unitaires + `tests/parity.rs` | 18 + 13 | 0,2 s + 6,5 s | parité GPU/CPU (adaptateur logiciel en l'absence de GPU) |
| Doc-tests (`osatura-core`) | 19 | 171 s (chacun compile un programme) | les exemples de la façade (`api/`, `lib.rs`) ; `osatura-capi` et `osatura-wasm` n'en ont aucun |

Total : 1338 tests dans les binaires de tests unitaires et d'intégration et 19 doc-tests, comptés le 24 octobre 2026 avec `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` (les 1368 de la phrase suivante incluaient aussi les tests du crate GPU ; recompté à partir d'une exécution complète de `cargo test --workspace` le 22 octobre 2026 après la vague 7, les durées par suite ci-dessus proviennent d'une exécution antérieure), plus 19 doc-tests ; l'ensemble de `cargo test --workspace --offline` a pris 836 s sur la machine partagée.

Suites hors de `cargo test` :

* **Tests de fumée du studio (Chromium).** Cinq scripts Node pilotent le vrai studio dans Chromium sans interface avec `playwright-core` : `studio_ui_smoke.js` (128 vérifications), `studio_collab_smoke.js` (60, deux pages sur une même machine), `studio_overlays_smoke.js` (89), `studio_suggest_smoke.js` (35) et `studio_demo_smoke.js` (12 : la démo guidée charge les 13 étapes, n'écrit jamais, restaure le projet octet pour octet et récupère un onglet fermé) ; les cinq ont réussi le 24 oct. 2026 (chiffres tels qu'affichés par les scripts). Ils ne sont **pas** exécutés par `cargo test` et ne font pas partie de la CI par défaut. Pour les lancer :

  ```sh
  cargo build -p osatura-cli
  node osatura-cli/tests/studio_ui_smoke.js        # same for collab / overlays / suggest / demo
  ```

  Environnement : `OSATURA_BIN` (par défaut `target/debug/osatura`), `PLAYWRIGHT_CORE`, `CHROME`, et pour le script des surcouches `SHOTS` (répertoire des captures d'écran). Les scripts nettoient leurs fichiers temporaires.
* **Test du wrapper WASM.** `node osatura-wasm/js/test.mjs` (simulation uniquement, sauf si vous passez un chemin de projet et avez compilé le module ; voir [WASM.md](WASM.md)).
* **Audit des licences.** `scripts/licence-audit.sh` : reconstruit `docs/THIRD_PARTY.md` et `docs/THIRD_PARTY_NOTICES.md` à partir de `cargo metadata --locked --all-features` (dépendances normales et de build, dépendances de développement exclues), vérifie la ligne SPDX de chaque fichier source par rapport à la `license` de sa crate, puis exécute `cargo deny check licenses bans sources` sauf avec `--no-deny`. `deny.toml` autorise les licences permissives, et la MPL-2.0 uniquement pour des crates nommées.
* **clippy.** `cargo clippy --workspace --all-targets -- -D warnings` se termine avec le code 0 sans avertissement (vague 8) et est bloquant en CI. Lorsqu'un lint est inadapté au cas, le code porte un `#[allow(clippy::...)]` ciblé avec un commentaire (boucles indexées sur des tableaux parallèles, `too_many_arguments` sur des signatures stables, `manual_clamp` là où le traitement de NaN diffère, noms inhérents `next` / `mul` qui font partie de l'API publique) ; n'ajoutez pas d'autorisations (allow) au niveau de la crate.
* **Câblage de la CI.** Le dépôt fournit `osatura ci` pour les vérifications d'*animation* ([CI.md](CI.md)) ; c'est autre chose que la CI du code. Le projet n'a aucune configuration de CI du code dans l'arborescence ; la section 8 donne une recette minimale.

Ajout de tests : placez les tests du moteur à côté du code (`#[cfg(test)]`), les tests de comportement du CLI dans `osatura-cli/tests/`, et chaque bogue corrigé reçoit un test de régression (les vagues de revue dans [CHANGELOG.md](CHANGELOG.md) listent ce qui a été corrigé et où). Les tests ne doivent pas laisser de fichiers temporaires (utilisez des gardes de destruction) et ne doivent dépendre ni de l'horloge ni du réseau. Les budgets de temps sont testés avec une horloge de test déterministe.

## 4. Conventions de code

* **Aucune panique sur une entrée.** Tout ce qui lit un projet, un fichier, un argument ou un octet reçu du réseau renvoie une erreur. L'indexation, le découpage, `unwrap` et le dépassement arithmétique sur des valeurs non fiables sont des bogues. La façade (`api/`) intercepte en outre les paniques et renvoie `ApiError::Internal` (uniquement là où la cible déroule la pile), mais c'est la dernière ligne de défense, pas un principe de conception.
* **Politique des valeurs finies.** Les nombres issus de fichiers et d'arguments sont vérifiés (NaN, infini) ainsi que leur ordre de grandeur (les plages de la façade figurent dans [API.md](API.md) : magnitudes jusqu'à 1e7, fps dans (0, 1000], images 1..=200000, scène jusqu'à 32768, 4096 os, côté de rendu jusqu'à 16384 px). Rejetez, ne tronquez pas, sauf si la fonction est documentée comme tronquant (les images sont ramenées dans la timeline).
* **Vérifiez les tailles avant d'allouer.** Les tampons de pixels, les tailles de maillage et les nombres d'échantillons sont calculés avec une arithmétique vérifiée et refusés avant `vec![..]`.
* **Écritures atomiques.** Écrivez les résultats avec `fsutil::write_atomic` (fichier temporaire dans le même répertoire, `fsync`, renommage ; refuse les liens symboliques sauf avec `--follow-symlinks` ; se rabat sur un écrasement sur place suivi d'un `fsync`, documenté comme non atomique, lorsque le répertoire n'est pas accessible en écriture). N'utilisez jamais directement `std::fs::write` sur le fichier d'un utilisateur. Les répertoires temporaires proviennent de `fsutil` (mode 0700, fichiers 0600). Les octets aléatoires proviennent de `fsutil::random_bytes` (`/dev/urandom`).
* **Les ressources passent par `assets::resolve`.** Les chemins relatifs d'images, de normal maps et de sons sont résolus par rapport au répertoire de ressources valable pour tout le processus (`assets::default_dir`, défini à partir du dossier du projet d'entrée) ; les chemins absolus et `..` sont refusés sauf avec `--allow-external-assets` ; les fichiers image sont limités à 64 Mo et l'audio à 256 Mo. Les commandes qui écrivent un projet dérivé valident par rapport au répertoire du projet *d'entrée* et réécrivent les chemins de ressources relatifs ou avertissent à leur sujet (`write_json`).
* **Aléa déterministe.** Tout comportement aléatoire (fuzzing, bruit, gigue, simulation) utilise le générateur à graine propre au moteur avec une option `--seed` et est reproductible au bit près. Pas de crate `rand`, pas d'heure système comme graine.
* **Println et tubes fermés.** Le CLI masque `println!`/`print!` par des macros qui se terminent silencieusement sur un tube fermé (`osatura ... | head` ne doit pas paniquer). Continuez à les utiliser dans `main.rs`.
* **Les erreurs disent quoi faire.** Les messages indiquent ce qui n'allait pas, quelle valeur, ainsi que la plage autorisée ou la correction (les tests de la façade vérifient ce style).
<!-- REUSE-IgnoreStart -->
* **En-têtes SPDX.** Chaque fichier `.rs`, `.js`, `.mjs` et `.sh` commence par `SPDX-License-Identifier:` égal à la licence de sa crate (MPL-2.0 pour core/capi/wasm/gpu/scripts, AGPL-3.0-or-later pour le CLI) et une ligne de copyright ; les documents Markdown portent le commentaire d'en-tête CC-BY-4.0, comme ce fichier. Les autres fichiers (JSON, images, textes de licence) sont couverts par `REUSE.toml` ; lorsque vous ajoutez un nouveau motif de chemin, ajoutez-le à ce fichier. Sinon `cargo test -p osatura-cli --test spdx_audit` échoue.
<!-- REUSE-IgnoreEnd -->
* **Dépendances.** Peu nombreuses, en Rust pur, sous licence permissive. Une nouvelle crate demande une vérification `deny.toml` (`scripts/licence-audit.sh`), une ligne dans `docs/THIRD_PARTY.md` (régénéré) et une justification dans l'entrée de la ROADMAP. `osatura-core` est compilée sans features par défaut par capi, wasm, gpu et le CLI : tout ce qui est lourd doit donc passer derrière une feature.
* **La façade d'abord.** Toute nouvelle capacité du moteur visible par l'utilisateur et que d'autres langages doivent pouvoir atteindre va dans `osatura-core/src/api/` avec une ligne d'instantané, un doc-test et un test unitaire ; les couches C et wasm n'enveloppent que la façade.

## 5. Liste de contrôle pour un nouveau module ou une nouvelle commande

1. **Module** dans `osatura-core/src/<name>.rs`, déclaré dans `lib.rs` ; en-tête SPDX ; tests unitaires (valides, limites, entrées hostiles, déterminisme) ; aucune panique.
2. **Commande CLI** dans `osatura-cli/src/main.rs` : une entrée `USAGE` (orthographe exacte de chaque option), la branche de dispatch et les tables d'options : toute option qui prend une valeur doit figurer dans `VALUE_OPTS` ; les drapeaux sans valeur dans l'entrée `CMD_OPTS` de la commande lorsque la table globale serait inexacte (par exemple `ci --prune` est un drapeau, `export-oca --layers` et `autotrace --png` prennent des valeurs, `keyedit --free` est un drapeau). Les tests unitaires USAGE contre analyseur échouent si le texte et l'analyseur divergent. Prenez en charge les drapeaux globaux (`--json`, `--quiet`, `--allow-empty`, `--allow-external-assets`, `--follow-symlinks`, `--max-input-bytes`), `<command> --help` et `--key=value`.
3. **Écriture des sorties** uniquement avec `fsutil::write_atomic` ou `write_json` ; les commandes qui ne produisent rien se terminent avec un code non nul sauf avec `--allow-empty`.
4. **Documentation** : la commande dans [CLI_REFERENCE.md](CLI_REFERENCE.md) si vous en ajoutez une, une mention dans un guide là où c'est pertinent, et l'entrée dans la section 9 de `ROADMAP.md` (ce qui a été ajouté, ce qui n'est *pas* fait : cette liste alimente [LIMITATIONS.md](LIMITATIONS.md)).
5. **Ligne de CLEANROOM.md** : pour chaque nouvel algorithme, consignez la source publique à partir de laquelle il a été implémenté (manuel, norme, article) et la dérivation indépendante ; pas de ligne, pas de fusion.
6. **Revalidation de la surface de la façade** si `api/` a changé : exécutez `OSATURA_BLESS=1 cargo test -p osatura-core api::`, examinez le diff de `osatura-core/src/api_surface.txt` (une ligne supprimée ou modifiée est un changement incompatible et exige une version majeure de la façade ; une ligne ajoutée est une version mineure), puis relancez les tests sans la variable. Mettez à jour [API.md](API.md), ainsi que les wrappers C/WASM et leurs en-têtes s'ils doivent l'exposer.
7. **Bout en bout** : ajoutez la commande au pipeline e2e si elle appartient au chemin de production ; ajoutez une vérification de fumée aux scripts du studio si elle a une interface.
8. **Exécution** : `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/licence-audit.sh` (lorsque les dépendances ou les en-têtes ont changé).

## 6. Règles clean-room et DCO

Le projet est une implémentation indépendante. En bref (le texte qui fait foi est [CLEANROOM.md](CLEANROOM.md) et [CONTRIBUTING.md](CONTRIBUTING.md)) :

* Implémentez à partir de normes publiques, de manuels et d'articles, et citez-les. Ne lisez pas, ne décompilez pas, ne copiez pas et ne faites pas de rétro-ingénierie du code, des fichiers ou des interfaces d'outils d'animation propriétaires, et ne lisez ni ne collez le code source d'outils concurrents pendant que vous écrivez une fonctionnalité. Ne collez pas de code issu de projets dont la licence est incompatible.
* Consignez chaque nouvelle méthode dans le tableau de `CLEANROOM.md` avec sa source publique.
* Les notes sur les brevets se trouvent dans [PATENT_NOTES.md](PATENT_NOTES.md) ; une fonctionnalité qui touche un domaine sensible en matière de brevets y est d'abord discutée. Ceci n'est pas un avis juridique.
* Les contributions sont acceptées sous la licence de la partie qu'elles modifient (entrant égal sortant) : MPL-2.0 pour les bibliothèques et les scripts, AGPL-3.0-or-later pour `osatura-cli`, CC-BY-4.0 pour la documentation et les ressources synthétiques.
* **Sign-off.** Chaque commit exige `Signed-off-by: Name <email>` (`git commit -s`), attestant le Developer Certificate of Origin 1.1 (texte dans CONTRIBUTING.md). Le sign-off atteste uniquement l'origine : il n'accorde **aucun** droit de relicenciement, et aucune licence commerciale ou double licence n'est possible pour le code contribué tant qu'il n'existe pas de CLA.
* Le nom et le logo sont couverts par [TRADEMARK.md](TRADEMARK.md).

## 7. Liste de contrôle de publication

Les premiers builds sont préparés comme la préversion en brouillon `v0.1.0-alpha.3` (toutes les crates sont toujours en 0.1.0 ; les libellés de version « v0.5/v0.6/v0.7 » dans README, FEATURES et ROADMAP sont des niveaux de fonctionnalités, pas des versions de crates ; voir [CHANGELOG.md](CHANGELOG.md)). Pour en produire une :

1. **Version.** Décidez de la version de chaque crate. La façade a sa propre version (`API_VERSION`, 1.4.0 aujourd'hui : majeure en cas de suppression ou de modification d'une ligne de surface, mineure en cas d'ajout) et `OSATURA_ABI_VERSION` / `arm_abi_version()` (tous deux à 1) ne changent qu'en cas de changement incompatible de l'ABI C ou wasm. Alignez les versions de `Cargo.toml`, la sortie de `osatura --version`, l'encadré d'état du README et le CHANGELOG.
2. **Contrôles au vert.** `cargo test --workspace`, `cargo test --doc -p osatura-core`, `cargo test -p osatura-cli --test spdx_audit`, `cargo clippy --workspace --all-targets -- -D warnings`, les quatre tests de fumée Chromium, `scripts/e2e.sh` (release), `cargo test -p osatura-gpu` si le code GPU a changé.
3. **Mentions des tiers.** Exécutez `scripts/licence-audit.sh` : il régénère `docs/THIRD_PARTY.md` (inventaire) et `docs/THIRD_PARTY_NOTICES.md` (textes de licence de tout ce qui est livré), vérifie les lignes SPDX et exécute `cargo deny`. Validez les fichiers régénérés. Une distribution binaire doit livrer le lot de mentions à côté du binaire.
4. **Fichiers de licence.** `LICENSE`, `LICENSES/` (textes de chaque licence utilisée) et `REUSE.toml` sont synchronisés avec [LICENSING.md](LICENSING.md) ; l'attribution CC-BY-4.0 de la documentation est intacte.
5. **Pas de ffmpeg.** Ne fournissez pas, ne liez pas et ne téléchargez pas ffmpeg dans les artefacts de publication. Il est détecté dans le `PATH` à l'exécution et la documentation indique que l'utilisateur l'installe lui-même.
6. **Offre de sources AGPL.** Le binaire de `osatura-cli` sert sa propre offre de sources à l'adresse `/source` (AGPL section 13) et affiche un avis avec `--version` ; `release.yml` définit `OSATURA_SOURCE_URL` à la compilation avec `<server>/<repository>/tree/<commit>` afin que les binaires de release nomment les sources exactes à partir desquelles ils ont été construits (le test de fumée du workflow vérifie sa présence dans `--version` ; un simple `cargo build` n'a aucune adresse intégrée) ; les exploitants de serveurs de studio modifiés définissent `--source-url` vers l'endroit où se trouvent leurs sources modifiées. Tests : `tests/source_offer.rs`. Pour une publication binaire, fournissez les sources correspondantes (tag, archive tar) à l'URL accessible aux utilisateurs.
7. **Documentation.** Mettez à jour l'encadré d'état du README, [LIMITATIONS.md](LIMITATIONS.md) et le CHANGELOG (déplacez *Unreleased* sous la nouvelle version avec une date).
8. **Construire le brouillon sur GitHub.** `.github/workflows/release.yml` construit les archives Windows, macOS et Linux ainsi que le module web à chaque push sur `main` ; un commit sur `main` dont le message contient `[release vX.Y.Z-label]` (ou un tag `v*` poussé) crée aussi une *préversion en brouillon* avec les archives et `SHA256SUMS.txt` (un ancien brouillon portant le même tag est remplacé). Relisez le brouillon, puis publiez-le à la main.
9. **Tag et publication** avec un tag signé ; joignez des sommes de contrôle. Ne publiez pas de crates sous des noms qui ne sont pas les vôtres ; les règles de TRADEMARK.md s'appliquent aux forks.

## 8. Intégration continue

Deux choses différentes partagent ce mot (un troisième workflow, `.github/workflows/release.yml`, construit les archives de release et le brouillon de version, voir la section 7) :

* **CI d'animation** (`osatura ci`) : vérifications des *projets* (lint, rendus de référence, performance, déterminisme). Décrite en détail dans [CI.md](CI.md) : manifeste `osatura-ci.json`, seuils, références (`--update` contre `bless`), rapports, exemples GitHub Actions et en shell simple (marqués « example, not run » dans ce document).
* **CI du code** de ce dépôt : `.github/workflows/ci.yml` comporte quatre sortes de jobs : `test` (matrice ubuntu / windows / macos : `cargo test --workspace --exclude osatura-gpu --locked` et clippy avec `-D warnings`), `wasm32` (compile `osatura-wasm` pour `wasm32-unknown-unknown`, copie le module à côté de `osatura.js` et exécute `node osatura-wasm/js/test.mjs` avec un projet de démonstration, y compris sa partie avec le vrai module), `gpu parity` (informatif ; ignoré s'il n'existe aucun adaptateur, s'exécute sur l'adaptateur DX12 logiciel de Windows) et `licences` (`cargo-deny check` et `reuse lint`). La toolchain est figée (1.97.0) pour que de nouveaux lints clippy ne cassent pas le build. Le job `test` compile aussi `osatura-gui` et exécute `osatura-studio --selftest` (un vrai auto-test sous Windows et macOS). **Première exécution entièrement verte : 2026-10-24** sur des runners hébergés par GitHub. En cas d'échec, les étapes affichent les tests en échec sous forme d'annotations de workflow (lisibles sans accès aux journaux). Windows ignore les tests réservés à POSIX (liens symboliques, permissions, `/dev/zero`, séparateurs de chemin POSIX).

  ```sh
  cargo test --workspace --locked
  cargo test --doc -p osatura-core --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  scripts/licence-audit.sh            # needs python3 and (optional) cargo-deny
  ```

  Mettez en cache `~/.cargo/registry` et `target/` entre les exécutions, laissez plusieurs minutes au test e2e dans les builds de débogage, et n'exécutez les tests de fumée Chromium que sur des runners qui disposent de Chromium et de `playwright-core`.

## 9. Débogage

* **Backtraces.** `RUST_BACKTRACE=1 osatura ...` pour toute panique (une panique sur une entrée est un bogue ; merci de la signaler avec le projet qui la déclenche).
* **Sortie lisible par machine.** `--json` sur les commandes qui le prennent en charge ; `--quiet` pour les scripts. Codes de sortie : 0 en cas de succès, 1 en cas d'échec (vérification échouée, budget non respecté, mauvaise utilisation ; run : une commande inconnue et un argument manquant se terminent tous deux par 1) ; `osatura ci` distingue 1 (une vérification a échoué) de 2 (erreur).
* **Inspecter un projet.** `osatura check`, `osatura lint` (code 1 en cas d'erreurs) et `osatura diff a.json b.json` donnent la structure et les problèmes sans rendu ; `osatura render project.json N out.png` rend une image, et `out.svg` produit une sortie vectorielle que vous pouvez ouvrir dans un navigateur (`--bones` montre le squelette).
* **Studio.** Lancez `osatura studio project.json --port 0` (affiche le port réel). Les outils de développement du navigateur montrent l'API JSON : les requêtes sont de simples `POST /api/...` avec `Content-Type: application/json`. Un 421 signifie un en-tête Host absent de `--allow-host` ; un 403 signifie qu'un contrôle d'Origin ou de `Sec-Fetch-Site` a échoué (utilisez `--public-origin` derrière un proxy) ; 415 concerne le type de contenu ; 429 est la temporisation liée au jeton.
* **Étapes de la démo.** La démo guidée du Studio (et la démo pas à pas du site du projet) sont générées à partir de `osatura demo-person` par `scripts/build-studio-tour.mjs` et `scripts/lib/person-stages.mjs` ; la construction échoue si la démo terminée ne correspond plus aux chiffres cités dans les textes (`EXPECT` dans la bibliothèque). Après avoir modifié `osatura-core/src/demo.rs`, relancez le script et reconstruisez la CLI.
* **Journal de collaboration.** Le journal d'une session partagée (`--collab-log`) est un texte, une ligne `<hash> <json>` par opération ; une chaîne rompue est signalée au chargement.
* **Un test n'échoue que parfois.** Cherchez du côté de l'horloge, des noms de répertoires temporaires et de l'ordre d'itération des `HashMap` ; le code du moteur est déterministe par construction, de sorte que les échecs intermittents relèvent presque toujours de l'hygiène des tests.
* **L'instantané de la façade échoue.** Vous avez modifié une signature publique : voir la section 5, point 6.
* **GPU.** `osatura render-gpu <project> <out.png> --adapter software|auto --bench N` (uniquement dans les builds avec `--features gpu`) compare avec le moteur de rendu CPU (un adaptateur logiciel indique que ses durées ne signifient rien) ; `OSATURA_GPU_BACKEND` sélectionne le backend.
* **Sanitizers pour l'ABI C.** Compilez `smoke.c` ou votre propre appelant avec `-fsanitize=address,undefined` contre la bibliothèque statique (fait pendant le travail de documentation, sans problème ; voir [C_ABI.md](C_ABI.md)).

## 10. Profilage

* **Durées par étape du pipeline de production :** `scripts/e2e.sh` affiche un tableau (`OSATURA_E2E_TIMINGS` nomme le fichier) ; les chiffres en release de la section 9 de `ROADMAP.md` en proviennent.
* **GPU contre CPU :** `osatura render-gpu <project> <out.png> --bench N`.
* **Budget de taille et de vitesse :** `osatura size-report`, `osatura export-fit --max-seconds` (le budget de temps est vérifié à l'intérieur des boucles de candidats ; un appel à ffmpeg ou à un quantificateur n'est pas interruptible).
* **Coût de rendu d'un projet :** `osatura ci` avec un seuil de performance ([CI.md](CI.md)) enregistre le temps par image sur votre machine ; les durées dépendent de la machine, comparez donc sur la même machine.
* **Outils généraux :** compilez avec `--release` et utilisez `perf record`/`perf report` ou `cargo flamegraph` (ne font pas partie du projet ; à installer séparément). Les builds de débogage sont environ un ordre de grandeur plus lents et ne disent rien de la vitesse réelle.
