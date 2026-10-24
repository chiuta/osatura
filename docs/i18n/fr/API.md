# API Rust d'OSATURA et politique de stabilité

*Version de la façade 1.4.0 (`osatura_core::api::API_VERSION`). Public : développeurs Rust qui intègrent le moteur, et mainteneurs qui modifient la façade. Licence de la documentation CC-BY-4.0 ; exemples de code MPL-2.0.*

Voici la référence de l'**API Rust prise en charge** de `osatura-core` : le module façade `osatura_core::api` et son import en une ligne `osatura_core::prelude`. Elle liste chaque élément public, les règles qui garantissent la stabilité de l'API, et des exemples testés. Chaque exemple de ce fichier est compilé et exécuté sur les sources actuelles avant la publication du fichier (voir [Comment les exemples sont vérifiés](#comment-les-exemples-sont-vérifiés)).

Sommaire : [Ce qui est couvert](#ce-qui-est-couvert) - [Obtenir la crate](#obtenir-la-crate) - [Politique de stabilité](#politique-de-stabilité) - [Règles semver](#règles-semver) - [Feature flags](#feature-flags) - [Sécurité des threads](#sécurité-des-threads) - [Déterminisme](#déterminisme) - [Limites](#limites-et-plages-darguments) - [Erreurs](#erreurs) - [Référence : `Animation`](#référence--animation) - [Référence : `Player`](#référence--player) - [Référence : fonctions libres](#référence--fonctions-libres-outils-contrôle-de-version-ci) - [Référence : types](#référence--types) - [Exemples](#exemples) - [Autres langages](#autres-langages) - [Annexe : JSON machine et JSON de bindings](#annexe--json-de-machine-et-json-de-binding)

## Ce qui est couvert

Seuls `osatura_core::api` et `osatura_core::prelude` constituent l'API Rust publique prise en charge. La façade se compose de :

* `Animation` : un projet validé et son historique d'annulation (chargement, interrogation, rendu, export, édition, outils) ;
* `Player` : une machine à états en cours d'exécution, avec mise en page, défilement et niveau de détail (1.1, 1.2) ;
* des fonctions libres pour le contrôle de version et la CI d'animation (1.2 lecture, 1.3 écriture) et pour le tracé de bitmaps ;
* des types valeur simples (`AnimationInfo`, `RenderSettings`, `Image`, `LodLevel`, `SizeFit`, `MocapReport`, `FuzzSummary`, `VcsCommit`, `VcsConflict`, `CiOutcome`, `CiInit`, ...) et les énumérations `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome`, `VcsChoice` ;
* le type d'erreur `ApiError` (qui implémente `std::error::Error` et `Display`).

L'ABI C (`osatura-capi`, en-tête `osatura.h`) et le module WebAssembly (`osatura-wasm`) sont écrits uniquement contre cette façade et ont leurs propres versions : [C_ABI.md](C_ABI.md), [WASM.md](WASM.md).

### Ce qui n'est PAS couvert

Tous les autres modules de `osatura-core` (`engine`, `project`, `edit`, `svg`, `raster`, `physics`, `vcs`, `ci`, ...) et les réexportations à la racine (`Engine`, `Project`, ...). Ils sont `pub` et `#[doc(hidden)]` uniquement parce que l'outil en ligne de commande, le studio, les tests de l'ABI C et les tests unitaires les utilisent ; ils peuvent changer ou disparaître dans n'importe quelle version, et leurs textes `Err(String)` ne constituent pas non plus un contrat. Si vous avez besoin d'un élément accessible uniquement par ces modules, demandez un ajout à la façade (une version mineure) plutôt que de dépendre des internes.

Le format JSON du projet possède son propre champ de version `format` (actuellement 1) ; un projet écrit par une version plus récente est refusé avec `InvalidJson`. Voir [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Obtenir la crate

OSATURA n'est pas publié sur un registre ; dépendez-en par chemin (ou par git) depuis votre workspace :

```toml
[dependencies]
osatura-core = { path = "../osatura/osatura-core", default-features = false }
```

`default-features = false` supprime le moteur de script Rhai (voir [Feature flags](#feature-flags)) ; la façade n'en a pas besoin. Ensuite, `use osatura_core::prelude::*;`. La version de la crate (`0.1.0`) et la version de la façade (`API_VERSION`) sont indépendantes : la façade suit semver, la version de la crate ne promet encore rien. Aucune version minimale de Rust prise en charge n'est déclarée ; le workspace est développé et testé avec Rust 1.97 (édition 2021).

## Politique de stabilité

1. **Le fichier de surface.** `osatura-core/src/api_surface.txt` liste chaque élément public de `src/api/` (types, champs, variantes, constantes et signatures de méthodes, un par ligne). Le test unitaire `api::tests::api_surface_snapshot` reconstruit cette liste à partir de `api/` et échoue lorsqu'elle diffère du fichier versionné. Tout changement de la surface est donc une modification délibérée et relue des deux fichiers.
2. **Le test du prelude.** `osatura-core/tests/prelude_surface.rs` est compilé comme une crate *externe*. Il importe les noms de la façade uniquement via `osatura_core::prelude` et appelle un sous-ensemble représentatif (chargement, rendu, édition, export, `Player`, les outils, les wrappers de contrôle de version et de CI, gestion des entrées invalides). Si un nom qu'il utilise cesse d'être réexporté depuis le prelude, ou si une signature change d'une manière que ces appels détectent, ce test cesse de compiler. Il vérifie aussi `API_VERSION == "1.4.0"`, de sorte qu'un changement de version y est également une modification consciente. Il n'énumère pas chaque nom du prelude : ajouter un nom à la liste de réexportation reste donc une étape manuelle.
3. **Limites de la vérification.** L'instantané est une analyse de texte des signatures sur une seule ligne dans `api/`, et non une comparaison au niveau du compilateur comme `cargo-public-api` : un changement sémantique qui conserve le texte de la signature (par exemple une erreur différente pour la même entrée) n'est pas détecté, et une signature sur plusieurs lignes passerait inaperçue (le fichier garde les signatures sur une seule ligne pour cette raison). La liste de réexportation du prelude (`lib.rs`) est vérifiée par le test externe, pas par l'instantané.
4. **Caché ne veut pas dire pris en charge.** Les modules `#[doc(hidden)] pub` existent pour les consommateurs du workspace. Ils sont hors de la promesse même s'ils compilent chez vous.
5. **Comment modifier la façade** (mainteneurs ; voir aussi [DEVELOPMENT.md](DEVELOPMENT.md)) :
   * modifiez `api/` (signatures sur une ligne, commentaire de documentation, un doc-test pour les nouvelles fonctions libres, tests unitaires) ;
   * ajoutez les nouveaux noms publics à la liste de réexportation du `prelude` dans `lib.rs`, et à `tests/prelude_surface.rs` ;
   * incrémentez `API_VERSION` (et `API_VERSION_MAJOR` en cas de rupture) selon les règles ci-dessous, ainsi que la version vérifiée dans `prelude_surface.rs` ;
   * régénérez délibérément le fichier de surface : `OSATURA_BLESS=1 cargo test -p osatura-core api::` et relisez le diff de `api_surface.txt` ;
   * mettez à jour ce fichier, et `osatura-capi` / `osatura-wasm` uniquement si le nouvel élément doit franchir ces frontières (leur version d'ABI évolue séparément, les ajouts la conservent).

## Règles semver

`API_VERSION` est une version sémantique de la façade uniquement.

| Changement | Version |
|---|---|
| Correction de bug, sans changement de signature ni nouveau contrat de comportement | patch (1.4.x) |
| Nouvelle fonction, méthode, type, variante d'énumération, champ de struct, constante | mineure (1.x.0) |
| Suppression ou renommage de quoi que ce soit ; changement d'une signature, d'une plage d'arguments documentée de façon à rejeter une entrée auparavant valide, ou de la signification d'un résultat documenté | majeure (2.0.0) |

Conséquences pour les appelants :

* **Les énumérations et structs publics de la façade sont `#[non_exhaustive]`.** Ajoutez une branche générique lorsque vous faites un `match` sur `ApiError`, `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome` ou `VcsChoice` ; une nouvelle variante est une version mineure. Vous ne pouvez pas construire les structs par un littéral en dehors de la crate (et `Image`, `SizeFit` et les structs de rapport sont des résultats que vous ne faites que lire). Les réglages se construisent avec des constructeurs : `RenderSettings::new(scale).with_bones(true)`, `MocapSettings::new().with_smooth(2)`.
* **Les chaînes à l'intérieur des résultats sont des données, pas un contrat.** `VcsConflict::kind` vaut aujourd'hui l'une des valeurs `"BothChanged"`, `"DeleteEdit"`, `"KeyRetimed"`, `"OrderConflict"`, `"ArrayConflict"` et *peut gagner des valeurs dans les versions mineures*. Les rapports JSON (`FuzzSummary::json`, `CiOutcome::json`) portent leurs propres noms de schéma (`osatura-fuzz-report` v1, `osatura-ci/1`). Les textes des messages d'erreur sont destinés aux humains.
* **Stabilité du comportement.** Le rendu et l'évaluation sont déterministes (voir plus bas), mais la sortie en pixels peut changer dans une version mineure lorsqu'un bug du rastériseur ou d'un effet est corrigé ; les images de référence enregistrées par la CI d'animation en sont un rappel. Ne calculez pas de hachage des pixels rendus entre versions d'OSATURA ou entre machines sans tolérance.

Historique des versions de la façade : 1.0 (chargement, interrogation, rendu, export, édition, annulation) - 1.1 (`Player`, machines à états, data bindings) - 1.2 (mise en page, défilement et niveau de détail sur `Player` ; `lod_levels`, `fit_to_size`, `retarget_bvh`, `fuzz`, `from_image`, `trace_image_svg`, `vcs_log`, `ci_run`) - 1.3 (opérations d'écriture de contrôle de version et de CI, `DirtyTree` / `MergeInProgress` / `Conflicts`, `CiOutcome::updated` / `blessed`) - 1.4 (création de calques et d'os, `bake_physics`, export APNG et EXR, `vcs_branches` / `vcs_branch_delete` / `vcs_status` / `vcs_diff` / `vcs_show` / `vcs_gc` / `vcs_fsck`).

## Feature flags

`osatura-core` possède une seule feature Cargo :

| Feature | Par défaut | Effet |
|---|---|---|
| `scripting` | activée | Compile le module de script en bac à sable basé sur Rhai (`script.rs`), utilisé uniquement par la commande `script` de la ligne de commande. Tire `rhai` et, à travers lui, `smartstring` (MPL-2.0+, la seule crate tierce à copyleft faible de l'arbre). |

La façade n'a **aucune** fonctionnalité derrière une feature : chaque élément documenté ici existe avec `default-features = false`. `osatura-capi`, `osatura-wasm` et `osatura-gpu` dépendent de core sans la feature, donc ils ne contiennent aucune crate tierce MPL ; `osatura-cli --no-default-features` fait de même et signale alors la commande `script` comme indisponible. Les back-ends de rendu ne sont pas des features : le rastériseur CPU est toujours utilisé par la façade ; le moteur de rendu GPU facultatif est une crate distincte (`osatura-gpu`) et une feature de la CLI (`gpu`), et n'est pas accessible via cette API.

## Sécurité des threads

* `Animation`, `Player`, `ApiError` et `Image` sont `Send + Sync` (vérifié à la compilation pour ce document). La façade n'écrit aucun état global ; elle lit seulement deux réglages de ressources à l'échelle du processus (le répertoire de ressources par défaut et l'interrupteur des ressources externes) que rien dans la façade ne modifie.
* Les méthodes qui prennent `&self` (`info`, `bone_world`, `render_*`, `export`, `lod_levels`, `fit_to_size`, `fuzz`, `player`, `to_json`, ...) peuvent s'exécuter simultanément sur plusieurs threads sur la même `Animation` ; les méthodes qui prennent `&mut self` (`set_key`, `delete_key`, `undo`, `redo`, `retarget_bvh`, chaque entrée de `Player` et `advance`) exigent un accès exclusif, comme d'habitude en Rust. L'exemple `render_in_parallel` ci-dessous rend chaque image sur son propre thread et vérifie que les images sont identiques à celles du rendu mono-thread.
* Il n'y a ni verrouillage interne ni thread en arrière-plan. Un `Player` travaille sur sa propre copie du projet, de sorte que modifier l'`Animation` par la suite n'affecte jamais un player en cours d'exécution.
* `Animation::from_json` résout les fichiers image relatifs par rapport au **répertoire courant** du processus et refuse les chemins absolus et `..` (voir [SECURITY.md](SECURITY.md)) ; intégrez les images sous forme d'URI de données, ou définissez le répertoire de travail avant le chargement, si un projet référence des fichiers.
* Les fonctions de contrôle de version et de CI travaillent sur des fichiers : elles ne sont pas sûres si un autre processus modifie le même projet au même moment (la façade vérifie au préalable l'absence d'arbre modifié et de fusion en attente, mais il n'y a pas de verrou).

## Déterminisme

Même projet, mêmes arguments, même build d'OSATURA et même machine : mêmes octets. Cela vaut pour `render_rgba` / `render_png` / `render_svg`, `bone_world`, chaque export, `lod_levels` et `fuzz` (pour une graine donnée), `fit_to_size` et pour un `Player` (mêmes entrées et mêmes appels à `advance` donnent la même pose ; `advance` découpe le temps en morceaux d'au plus 1/60 s, donc la façon dont vous découpez le temps n'importe qu'en dessous de cette résolution). Les choix aléatoires à l'intérieur du moteur (bruit, particules, fuzzing, clusters de tracé) utilisent des générateurs à graine, jamais l'horloge. Ce qui n'est *pas* promis : des pixels de bord anticrénelés identiques entre systèmes d'exploitation, CPU, compilateurs ou versions de `libm` différents, entre cibles 32 et 64 bits, ou entre versions d'OSATURA. Les images de référence exactes à l'octet près n'ont été vérifiées qu'entre processus sur une seule machine Linux x86-64 (voir [LIMITATIONS.md](LIMITATIONS.md)) ; utilisez des tolérances (la CI d'animation le fait) pour tout ce qui traverse des machines.

## Limites et plages d'arguments

| Élément | Règle |
|---|---|
| `MAX_JSON_BYTES` | 256 Mio ; un texte plus long donne `InvalidJson` |
| `MAX_RENDER_SIDE` | 16384 pixels de sortie par côté (`render_*` et `Player::render_*`) |
| Échelle de rendu / GIF | finie et comprise dans `0.05..=4`, sinon `InvalidArgument` |
| Positions d'image | doivent être finies ; les valeurs hors de la timeline sont **bornées** à `0..=frames-1` (pas d'erreur). Les images fractionnaires sont acceptées. |
| Clés (`set_key`, `delete_key`) | `frame` et `value` finis ; `prop` est un nom de propriété en snake_case issu du JSON du projet (`"bone_angle"`, `"layer_opacity"`, ...) |
| Projet | `fps` dans `(0, 1000]`, `frames` dans `1..=200000`, scène jusqu'à 32768 unités par côté, au plus 4096 os, nombres au-delà de +-1e7 refusés au chargement ; d'autres limites propres à chaque fonctionnalité sont validées au chargement et reviennent sous forme de `InvalidProject` |
| `Player::advance(dt)` | `dt` fini, `0..=10` secondes |
| `Player::set_canvas_size` | les deux côtés finis, `> 0`, au plus 32768 |
| `Player::set_screen_size` | largeur et hauteur finies et `> 0` ; `max_error_px` fini et `>= 0` |
| `Animation::lod_levels` | `tolerance_px > 0` ; `scale` dans `(0, 64]` ; `levels` dans `1..=12` |
| `Animation::fit_to_size` | `max_bytes` dans `1000..=1_000_000_000` |
| `Animation::fuzz` | `samples` dans `1..=100000` |
| `trace_image_svg`, `Animation::from_image` | `colors` dans `2..=64` ; octets PNG ou JPEG (des plafonds de taille s'appliquent) |
| `vcs_log` | `limit` dans `1..=10000` |

## Erreurs

Toute fonction faillible renvoie `Result<_, ApiError>`. Une entrée invalide est une valeur d'erreur et **ne provoque jamais de panique** ; une panique qui survient malgré tout dans le moteur est interceptée à la frontière de la façade et renvoyée sous forme de `ApiError::Internal` (merci de la signaler comme un bug). `ApiError` implémente `Display` et `std::error::Error`, donc `?` fonctionne avec `Box<dyn Error>` et les types de style `anyhow`.

| Variante | Signification | Sources typiques |
|---|---|---|
| `InvalidJson(String)` | pas du JSON, pas un projet, mauvaise forme, trop volumineux (> `MAX_JSON_BYTES`), ou un `format` plus récent que ce build ; également une valeur JSON invalide passée à `vcs_resolve` | `Animation::from_json`, `validate_json`, `vcs_resolve` |
| `InvalidProject(Vec<String>)` | analysé, mais la validation a échoué ; un message par problème (os inconnu, plage invalide, cycle, ...) | `from_json`, `from_image`, reconstructions du moteur dans `Player` |
| `InvalidArgument(String)` | un argument de l'appelant a été rejeté : nombre non fini, valeur hors plage, propriété inconnue, identifiant inconnu de machine / d'entrée / de défilement, BVH ou image inutilisable, mauvais nom de branche, rien à valider, aucun dépôt | presque toutes les fonctions |
| `EditRejected(Vec<String>)` | une édition a été refusée (par exemple canal ou calque verrouillé, ou résultat invalide) ; le projet est inchangé | `set_key`, `delete_key`, `retarget_bvh` |
| `Export(String)` | un encodeur a échoué ou le format est impossible pour ce projet | `export`, `render_png`, `fit_to_size` |
| `Internal(String)` | une panique interceptée | toutes |
| `DirtyTree(String)` | (1.3) le fichier de projet contient des modifications non validées que l'opération écraserait | `vcs_checkout` (sauf `force`), `vcs_merge`, `vcs_cherry_pick`, `vcs_revert` |
| `MergeInProgress(String)` | (1.3) une fusion, un cherry-pick ou un revert est inachevé ; résolvez et validez, ou `vcs_abort` | mêmes fonctions |
| `Conflicts(Vec<String>)` | (1.3) `vcs_commit` alors que des conflits subsistent ; un pointeur par conflit | `vcs_commit` |

Une vérification échouée de la CI d'animation n'est **pas** une erreur : `ci_run` renvoie `Ok(CiOutcome)` avec `exit_code` valant 1 (une vérification a échoué) ou 2 (une vérification n'a pas pu être évaluée) ; seul un manifeste inutilisable donne `Err`. De même, les conflits de fusion sont un résultat (`VcsMergeOutcome::Conflicted`), pas une erreur.

## Référence : `Animation`

Un projet chargé et validé, avec son historique d'annulation. Créez-la avec `from_json`, `demo` ou `from_image`. Toutes les méthodes qui lisent le projet prennent `&self`.

| Méthode | Renvoie | Remarques |
|---|---|---|
| `Animation::from_json(&str)` | `Result<Animation, ApiError>` | analyse + validation. `InvalidJson` / `InvalidProject`. Les champs JSON inconnus sont ignorés silencieusement ici (la commande CLI `check` les signale). |
| `Animation::validate_json(&str)` | `Result<(), ApiError>` | mêmes vérifications, ne conserve rien |
| `Animation::demo()` | `Animation` | petit rig intégré (6 os, 48 images à 24 fps, 640 x 480) ; pratique pour les tests et les premiers pas |
| `Animation::from_image(&[u8], colors, name)` | `Result<Animation, ApiError>` | trace des octets PNG/JPEG en calques vectoriels remplis sur une scène de la taille de l'image |
| `to_json(&self)` | `String` | le projet courant en JSON indenté (aller-retour possible via `from_json`) |
| `info(&self)` | `AnimationInfo` | `name`, `width`, `height` (unités de scène), `fps`, `frames` |
| `frame_count(&self)`, `fps(&self)` | `u32`, `f32` | |
| `bone_names(&self)` | `Vec<String>` | ordre du rig = ordre de `bone_world` |
| `layer_names(&self)` | `Vec<String>` | chemins de calques `group/child`, en profondeur d'abord |
| `bone_world(&self, frame)` | `Result<Vec<[f32; 6]>, ApiError>` | matrice monde par os `[a b c d tx ty]` ; un point local à l'os `(x, y)` est envoyé sur `(a*x + c*y + tx, b*x + d*y + ty)`. Image bornée. Évalue tout le rig (IK, contraintes, ressorts, expressions). |
| `machine_names(&self)` | `Vec<String>` | machines à états runtime stockées dans le projet |
| `player(&self, machine)` | `Result<Player, ApiError>` | démarre la machine nommée au temps 0 sur une copie du projet |
| `render_rgba(&self, frame, &RenderSettings)` | `Result<Image, ApiError>` | RGBA8 à alpha droit, ligne par ligne, rangée du haut en premier |
| `render_png(&self, frame, &RenderSettings)` | `Result<Vec<u8>, ApiError>` | un fichier PNG en mémoire |
| `render_svg(&self, frame, show_bones: bool)` | `Result<String, ApiError>` | un document SVG (la sortie vectorielle n'a pas d'échelle) |
| `export(&self, ExportFormat, scale)` | `Result<Vec<u8>, ApiError>` | `Gif` (en boucle, toute la timeline ; `scale` dans `0.05..=4`), `Glb` (glTF 2.0 binaire), `Lottie` (octets JSON), `Html` (une page de lecteur autonome) ; `scale` est ignoré par les trois autres. Ce que chaque format conserve ou abandonne : [EXPORT_FORMATS.md](EXPORT_FORMATS.md). |
| `set_key(&mut self, target, prop, frame, value, Interpolation)` | `Result<(), ApiError>` | définit ou ajoute une clé sur `target.prop` ; validé, annulable, respecte les verrous de canal et de calque (`EditRejected`) |
| `delete_key(&mut self, target, prop, frame)` | `Result<bool, ApiError>` | `true` si une clé a été supprimée |
| `add_rect_layer(&mut self, name, x, y, w, h, fill, parent)`, `add_ellipse_layer(&mut self, name, cx, cy, rx, ry, fill, parent)` | `Result<(), ApiError>` | (1.4) ajoute un calque de forme remplie (au niveau supérieur, ou dans le groupe de niveau supérieur nommé par `parent`) ; nom unique sans `/`, tailles finies et positives, couleur valide ; annulable |
| `add_group_layer(&mut self, name)` | `Result<(), ApiError>` | (1.4) ajoute un calque de groupe vide |
| `add_bone(&mut self, name, parent, length, x, y, angle)` | `Result<(), ApiError>` | (1.4) ajoute un os (`parent` doit exister, angle en degrés) ; annulable |
| `bake_physics(&mut self, chain, preset, overwrite)` | `Result<PhysicsBake, ApiError>` | (1.4) simule une chaîne d'os (racine en premier) avec `hair`, `tail`, `rope`, `antenna` ou `cape` et stocke les canaux d'angle en une seule étape d'annulation ; avec `overwrite = false`, les os animés à la main sont conservés et listés dans `skipped` |
| `render_exr(&self, frame, &RenderSettings, float32)` | `Result<Vec<u8>, ApiError>` | (1.4) fichier OpenEXR (linéaire, prémultiplié ; HALF ou FLOAT) issu du compositeur en virgule flottante |
| `undo(&mut self)`, `redo(&mut self)` | `bool` | `false` quand il n'y a rien à annuler / rétablir |
| `lod_levels(&self, tolerance_px, scale, levels)` | `Result<Vec<LodLevel>, ApiError>` | le niveau 0 est l'original, le niveau k >= 1 reste dans `tolerance_px * 2^(k-1)` pixels de sortie à `scale` ; chaque niveau porte sa borne déclarée et un JSON de projet. Déterministe ; la borne couvre la lecture de la timeline (pas les poses de machine à états). Lent dans les builds non optimisés (utilisez release). |
| `fit_to_size(&self, FitFormat, max_bytes)` | `Result<SizeFit, ApiError>` | le plus petit export respectant le budget en abaissant pas à pas les paramètres de qualité ; si c'est impossible, le plus petit résultat avec `met == false` et un `reason` |
| `retarget_bvh(&mut self, bvh_text, &MocapSettings)` | `Result<MocapReport, ApiError>` | retarget BVH en **une seule édition annulable** ; os appariés par nom et topologie ; canaux verrouillés ignorés et signalés |
| `fuzz(&self, samples, seed)` | `Result<FuzzSummary, ApiError>` | fuzzing de poses ; l'animation est seulement lue ; rapport complet en JSON |

`Interpolation` du segment qui commence à une clé : `Linear`, `Step` (maintien), `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` (Catmull-Rom à travers les clés voisines). Les courbes plus riches du format de projet (Bézier cubique, bounce, back, spring) sont créées via la CLI et le studio, pas via `set_key`.

## Référence : `Player`

Une machine à états en cours d'exécution (lecture interactive). Créé par `Animation::player` ; travaille sur une copie du projet ; déterministe. Les machines à états et les data bindings sont du simple JSON de projet (`machines`, `bindings`), résumé dans l'[annexe](#annexe--json-de-machine-et-json-de-binding).

| Méthode | Renvoie | Remarques |
|---|---|---|
| `set_bool(name, bool)`, `set_number(name, f64)`, `fire(name)` | `Result<(), ApiError>` | définit une entrée par son nom ; un mauvais type d'entrée, ou un nom inconnu, donne `InvalidArgument`. Un déclencheur reste actif jusqu'à ce qu'une transition qui le teste soit franchie (il est consommé) ou jusqu'à la fin du prochain `advance` ; `hold: true` le conserve jusqu'à sa consommation. |
| `set_value(path, f64)` | `Result<(), ApiError>` | publie une valeur externe pour les data bindings (`"player.health"`). Une région de défilement dont l'`input` est ce chemin prend la valeur comme décalage en pixels le long de son axe principal (le défilement est alors au repos). |
| `advance(dt)` | `Result<(), ApiError>` | fait avancer la machine et les régions de défilement de `dt` secondes (`0..=10`) |
| `state(layer)` | `Option<String>` | état courant d'un calque de machine ; `None` pour un calque inconnu |
| `events_json()` | `String` | événements en attente `[{"time":..,"layer":..,"kind":"entered"|"exited","state":..}]`, non consommés |
| `take_events_json()`, `clear_events()` | `String`, `()` | consomme / efface les événements en attente |
| `render_rgba(&RenderSettings)`, `render_svg(show_bones)`, `bone_world()` | comme sur `Animation` | la pose courante ; l'image a la taille de canevas définie par `set_canvas_size` multipliée par l'échelle |
| `set_canvas_size(w, h)`, `canvas_size()` | `Result<(), ApiError>`, `(f32, f32)` | (1.2) mise en page adaptative : les conteneurs sont recalculés, les limites de défilement suivent |
| `scroll_ids()` | `Vec<String>` | identifiants des régions de défilement du projet |
| `scroll_drag(id, x, y)` | `Result<(), ApiError>` | décalage brut du pointeur en pixels (contenu déplacé vers le haut / la gauche d'autant) ; au-delà des extrémités, il est borné ou élastique |
| `scroll_release(id, vx, vy)` | `Result<(), ApiError>` | vitesse de relâchement en px/s ; `advance` lance alors l'inertie, aimante et ramène en ressort |
| `scroll_offset(id)`, `scroll_max(id)`, `scroll_settled(id)` | `Option<[f32; 2]>`, `Option<[f32; 2]>`, `Option<bool>` | `None` pour un identifiant inconnu |
| `set_screen_size(w_px, h_px, max_error_px)` | `Result<(), ApiError>` | (1.2) choisit le plus grossier de huit niveaux de détail préconstruits dont la borne déclarée, à l'échelle qui ajuste le canevas à cet écran, est au plus `max_error_px`. Les niveaux sont construits une fois à la première utilisation (niveau 1 borné à 0,125 unité de scène, puis doublement) et mis en cache ; avec `max_error_px` à 0, seuls les niveaux sans perte conviennent. |
| `clear_screen_size()`, `lod_level()`, `lod_bound_px()` | `()`, `usize`, `f32` | retour à l'original ; niveau utilisé (0 = original) ; sa borne déclarée en pixels |

Remarques : la borne du niveau de détail couvre la géométrie et les clés de la timeline telles que documentées dans `lod.rs` (`ASSUMPTIONS`, `LIMITS`) ; les poses de machine à états et les bindings n'y sont pas modélisés, et les canaux de pose qui ciblent un calque supprimé par un niveau sont abandonnés. Les proxys de défilement du studio utilisent un mécanisme distinct avec leur propre grille de niveaux. L'`input` d'une région de défilement passe par la table de valeurs du player, et non par le runtime de data binding, si bien que les expressions, le remappage et le lissage ne s'y appliquent pas.

## Référence : fonctions libres (outils, contrôle de version, CI)

| Fonction | Renvoie | Rôle |
|---|---|---|
| `trace_image_svg(&[u8], colors)` | `Result<String, ApiError>` | trace un bitmap en un SVG de régions remplies |
| `vcs_log(path, limit)` | `Result<Vec<VcsCommit>, ApiError>` | (1.2, lecture seule) historique, le plus récent en premier ; `path` est le fichier de projet ou n'importe quel chemin à l'intérieur de son dépôt ; un historique vide donne une liste vide |
| `ci_run(manifest)` | `Result<CiOutcome, ApiError>` | (1.2) CI d'animation en mode verify (rien n'est réenregistré) ; les chemins relatifs du manifeste se résolvent à côté de lui |
| `vcs_init(project_file, author)` | `Result<(), ApiError>` | (1.3) démarre un historique à côté du fichier sur la branche `main`, sans commit pour l'instant ; échoue s'il en existe un |
| `vcs_commit(path, message, author)` | `Result<String, ApiError>` | identifiant du commit ; termine une fusion une fois qu'aucun conflit ne subsiste ; `Conflicts` tant qu'il en reste ; rien à valider ou pas de message donne `InvalidArgument` |
| `vcs_branch(path, name, at)`, `vcs_tag(path, name, at)` | `Result<String, ApiError>` | crée une branche / un tag (par défaut : commit courant) ; sans basculer dessus ; les tags ne bougent jamais ; identifiant du commit renvoyé |
| `vcs_checkout(path, target, force)` | `Result<String, ApiError>` | bascule le fichier sur une branche (on y reste) ou sur n'importe quelle révision (détaché) ; `DirtyTree` sauf `force` ; `MergeInProgress` |
| `vcs_merge(path, rev, no_ff, message, author)` | `Result<VcsMergeOutcome, ApiError>` | `UpToDate`, `FastForward(id)`, `Merged(id)` ou `Conflicted(Vec<VcsConflict>)` |
| `vcs_cherry_pick(path, rev, author)`, `vcs_revert(path, rev, author)` | `Result<VcsMergeOutcome, ApiError>` | rejoue / annule la modification d'un commit |
| `vcs_conflicts(path)` | `Result<Vec<VcsConflict>, ApiError>` | les conflits encore en attente (vide s'il n'y en a pas) |
| `vcs_resolve(path, conflict_path, &VcsChoice)` | `Result<usize, ApiError>` | règle un conflit avec `Ours`, `Theirs` ou `Value(json text)` ; renvoie le nombre de conflits restants |
| `vcs_abort(path)` | `Result<(), ApiError>` | abandonne une fusion / un cherry-pick / un revert inachevé et restaure le dernier commit |
| `vcs_branches(path)` | `Result<Vec<VcsBranch>, ApiError>` | (1.4) les branches avec leur commit et leur état d'extraction |
| `vcs_branch_delete(path, name)` | `Result<(), ApiError>` | (1.4) supprime une branche ; la branche extraite et les noms inconnus sont refusés ; les objets restent jusqu'à `vcs_gc` |
| `vcs_status(path)` | `Result<VcsStatus, ApiError>` | (1.4) branche, HEAD, modifications sémantiques non validées, fusion inachevée, chemins des conflits |
| `vcs_diff(path, a, b)` | `Result<Vec<VcsChange>, ApiError>` | (1.4) différences sémantiques entre deux révisions ; `None` désigne le fichier de travail |
| `vcs_show(path, rev)` | `Result<VcsShow, ApiError>` | (1.4) un commit et ses modifications par rapport à son premier parent |
| `vcs_gc(path)` | `Result<VcsGc, ApiError>` | (1.4) supprime les objets inaccessibles ; s'interrompt sans rien supprimer si l'historique ne peut pas être lu |
| `vcs_fsck(path)` | `Result<Vec<String>, ApiError>` | (1.4) problèmes d'intégrité, vide si tout est sain |
| `ci_init(project, manifest, write_goldens)` | `Result<CiInit, ApiError>` | écrit un manifeste calibré par défaut (n'écrase jamais un fichier existant ; nom par défaut `osatura-ci.json` à côté du projet) ; enregistre éventuellement les images de référence tout de suite |
| `ci_run_mode(manifest, CiMode)` | `Result<CiOutcome, ApiError>` | `Verify`, `Update` (réenregistre toutes les images de référence des vérifications de rendu), `Bless` (accepte uniquement celles qui échouent ou manquent) |
| `ci_bless(manifest)` | `Result<CiOutcome, ApiError>` | raccourci pour `Bless` |

L'historique est stocké à côté du fichier de projet (`.osatura-vcs/`) ; les messages de commit et les auteurs sont du texte brut. Un conflit contient le pointeur d'identité `path`, un `kind`, une `description`, et les valeurs `base` / `ours` / `theirs` sous forme de texte JSON (`None` = absent à cet endroit). Après une fusion conflictuelle, le fichier de projet contient l'état fusionné avec les éléments en conflit à la valeur de la branche courante, et un état en attente est enregistré jusqu'à ce que vous résolviez chaque conflit puis fassiez `vcs_commit`, ou `vcs_abort`. Les ressources binaires qu'un projet référence ne sont pas versionnées, seulement leurs chaînes de chemin. La fusion sémantique, ses types de conflit et ses limites sont décrits dans [ROADMAP.md](ROADMAP.md) section 9 (élément I-01) et dans [LIMITATIONS.md](LIMITATIONS.md).

Hors de la façade : les surcharges de mappage d'os du mocap, le réglage fin du tracé au-delà du nombre de couleurs, les options de fit au-delà du format et du budget, le rendu GPU, le studio, la collaboration, le serveur d'agent, les nombreuses commandes de création et de baking (physique, foot lock, aimants, autorig, ...), la suppression / liste des branches, diff, show, status, gc, fsck, et les options de CI comme prune, `--only` et les artefacts. Utilisez la CLI pour cela ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

## Référence : types

| Type | Champs / variantes |
|---|---|
| `AnimationInfo` | `name: String`, `width: f32`, `height: f32` (unités de scène), `fps: f32`, `frames: u32` |
| `RenderSettings` | `scale: f32` (0.05..=4), `show_bones: bool`, `transparent: bool` ; `new(scale)`, `with_bones(bool)`, `with_transparent(bool)` ; `Default` = échelle 1, les deux désactivés |
| `Image` | `width: u32`, `height: u32`, `pixels: Vec<u8>` (RGBA8, alpha droit) |
| `ExportFormat` | `Gif`, `Glb`, `Lottie`, `Html`, `Apng` (1.4) |
| `PhysicsBake` (1.4) | `added: usize`, `skipped: Vec<String>` |
| `VcsBranch` (1.4) | `name`, `commit`, `current: bool` |
| `VcsChange` (1.4) | `kind` (`"Added"`, `"Removed"`, `"Changed"`, `"Moved"`, `"Retimed"`), `path`, `description` |
| `VcsStatus` (1.4) | `branch`, `head`, `changes: Vec<VcsChange>`, `merge`, `conflicts: Vec<String>` |
| `VcsShow` (1.4) | `commit: VcsCommit`, `changes: Vec<VcsChange>` |
| `VcsGc` (1.4) | `kept`, `removed` |
| `Interpolation` | `Linear`, `Step`, `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` |
| `LodLevel` | `level`, `declared_px: f64`, `bound_stage: f64`, `path_anchors`, `keys`, `layers`, `json: String` ; `bound_at(screen_scale) -> f64` |
| `FitFormat` | `Lottie`, `Html`, `Gif`, `Apng`, `Project` |
| `SizeFit` | `met: bool`, `size: usize`, `bytes: Vec<u8>`, `ssim_min: f64`, `reason: Option<String>` |
| `MocapSettings` | `in_place`, `mirror`, `smooth` (rayon de moyenne mobile en images) ; `new()`, `with_in_place`, `with_mirror`, `with_smooth` ; `Default` |
| `MocapReport` | `matched: Vec<(String, String)>` (articulation BVH, os du rig), `unmatched_joints`, `unmatched_bones`, `frames`, `bones_used`, `blocked` (`target.prop` verrouillés), `notes` |
| `FuzzSummary` | `samples`, `failures`, `failure_rate: f32`, `max_severity: f32`, `json: String` |
| `VcsCommit` | `id`, `author`, `time: u64` (secondes Unix), `message` |
| `VcsConflict` | `path`, `kind`, `description`, `base`, `ours`, `theirs: Option<String>` |
| `VcsMergeOutcome` | `UpToDate`, `FastForward(String)`, `Merged(String)`, `Conflicted(Vec<VcsConflict>)` |
| `VcsChoice` | `Ours`, `Theirs`, `Value(String)` |
| `CiMode` | `Verify`, `Update`, `Bless` |
| `CiOutcome` | `total`, `passed`, `failed`, `errors`, `exit_code: i32` (0 correct, 1 échec, 2 non évaluable), `json: String`, `updated`, `blessed` |
| `CiInit` | `manifest: String`, `checks: Vec<(String, String)>` (type, nom), `recorded: Option<CiOutcome>` |
| Constantes | `API_VERSION: &str`, `API_VERSION_MAJOR: u32`, `MAX_RENDER_SIDE: u32`, `MAX_JSON_BYTES: usize` |

## Exemples

Tous sont complets et ont été exécutés. Ils supposent `use osatura_core::prelude::*;`.

### Charger, évaluer, rendre, exporter

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Load a project, query it, render one frame and export the whole animation.
pub fn quick(project_path: &str) -> Result<(), Box<dyn Error>> {
    let json = std::fs::read_to_string(project_path)?;
    let mut anim = Animation::from_json(&json)?;           // parse + validate; typed error otherwise

    // Facts about the project.
    let info = anim.info();
    println!("{}: {}x{} @ {} fps, {} frames", info.name, info.width, info.height, info.fps, info.frames);

    // Evaluate: world matrices of every bone at frame 10 (six floats per bone: a b c d tx ty).
    let names = anim.bone_names();
    let world = anim.bone_world(10.0)?;
    let m = world[0];
    // A bone-local point (x, y) lands at (a*x + c*y + tx, b*x + d*y + ty).
    let (x, y) = (m[0] * 10.0 + m[2] * 0.0 + m[4], m[1] * 10.0 + m[3] * 0.0 + m[5]);
    println!("tip of {} (10 units along its x axis) is at ({x:.1}, {y:.1})", names[0]);

    // Edit through the undoable editor, then render.
    anim.set_key(&names[0], "bone_angle", 10.0, 25.0, Interpolation::EaseInOut)?;
    let png = anim.render_png(10.0, &RenderSettings::new(1.0).with_bones(true))?;
    std::fs::write("frame10.png", &png)?;
    let svg = anim.render_svg(10.0, false)?;
    std::fs::write("frame10.svg", svg)?;

    // Raw pixels: straight-alpha RGBA8, row-major, top row first.
    let img = anim.render_rgba(10.0, &RenderSettings::new(0.5).with_transparent(true))?;
    assert_eq!(img.pixels.len(), img.width as usize * img.height as usize * 4);

    // Whole-animation exports.
    let glb = anim.export(ExportFormat::Glb, 1.0)?;
    let gif = anim.export(ExportFormat::Gif, 0.5)?;        // `scale` only matters for GIF
    std::fs::write("anim.glb", &glb)?;
    std::fs::write("anim.gif", &gif)?;
    println!("{} bones, {} bytes of glb, {} bytes of gif", names.len(), glb.len(), gif.len());
    Ok(())
}
```

### Édition, annulation et gestion des erreurs

```rust
use osatura_core::prelude::*;

/// Edits are validated, undoable and all-or-nothing.
pub fn edit_and_errors() {
    let mut anim = Animation::demo();                       // small built-in project
    let bone = anim.bone_names()[0].clone();
    let before = anim.to_json();

    anim.set_key(&bone, "bone_angle", 4.0, 30.0, Interpolation::Smooth).unwrap();
    assert!(anim.delete_key(&bone, "bone_angle", 4.0).unwrap());      // true: a key was removed
    assert!(!anim.delete_key(&bone, "bone_angle", 77.0).unwrap());    // false: nothing there
    assert!(anim.undo() && anim.undo());                    // each successful edit is one undo step
    assert_eq!(anim.to_json(), before);
    assert!(anim.redo());

    // Errors are values. Match on them with a wildcard arm: the enum is #[non_exhaustive].
    match anim.set_key(&bone, "no_such_prop", 0.0, 1.0, Interpolation::Linear) {
        Err(ApiError::InvalidArgument(why)) => println!("rejected argument: {why}"),
        Err(ApiError::EditRejected(problems)) => println!("edit refused: {problems:?}"),
        Err(other) => println!("other failure: {other}"),   // Display is implemented for every variant
        Ok(()) => unreachable!(),
    }
    // A rejected edit leaves the project exactly as it was.
    let snapshot = anim.to_json();
    assert!(anim.set_key("no_such_bone", "bone_angle", 1.0, 1.0, Interpolation::Linear).is_err());
    assert_eq!(anim.to_json(), snapshot);

    // Bad input never panics.
    assert!(matches!(Animation::from_json("{ nope"), Err(ApiError::InvalidJson(_))));
    assert!(anim.render_rgba(f32::NAN, &RenderSettings::default()).is_err());
    assert!(anim.render_rgba(0.0, &RenderSettings::new(50.0)).is_err()); // scale must be 0.05..=4
}
```

### Lecture interactive avec une machine à états

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Run a state machine of a project (machine.json has machine "hero", layer "body", bool input "excited").
pub fn play(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let machine = anim.machine_names()[0].clone();         // e.g. "hero"
    let mut player = anim.player(&machine)?;               // works on a copy of the project

    player.set_bool("excited", true)?;                     // also set_number, fire (trigger), set_value (binding path)
    for _ in 0..30 {
        player.advance(1.0 / 30.0)?;                       // deterministic; dt in seconds, 0..=10
    }
    println!("body is in state {:?}", player.state("body"));  // Some("wave")
    println!("{}", player.take_events_json());             // [{"time":..,"layer":"body","kind":"entered","state":..}, ..]

    let frame = player.render_rgba(&RenderSettings::new(0.5))?;
    println!("pose rendered at {}x{}", frame.width, frame.height);
    let svg = player.render_svg(false)?;
    assert!(svg.starts_with("<svg"));

    // Same inputs, same advance calls, same pose: a second player is bit-for-bit identical.
    let mut again = anim.player(&machine)?;
    again.set_bool("excited", true)?;
    for _ in 0..30 {
        again.advance(1.0 / 30.0)?;
    }
    assert_eq!(again.render_rgba(&RenderSettings::new(0.5))?, frame);

    // Wrong calls are typed errors, not panics.
    assert!(matches!(player.set_number("excited", 1.0), Err(ApiError::InvalidArgument(_))));
    assert!(matches!(anim.player("no such machine"), Err(ApiError::InvalidArgument(_))));
    Ok(())
}
```

### Mise en page, défilement et niveau de détail

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Layout, scrolling and level of detail on a Player (scroll.json has one scroll region "list").
pub fn layout_and_lod(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let mut player = anim.player(&anim.machine_names()[0])?;

    player.set_canvas_size(100.0, 120.0)?;                 // containers re-solve; renders get this size
    for id in player.scroll_ids() {
        let max = player.scroll_max(&id).unwrap();         // about [0, 80] here: 200 units of content in a 120 unit view
        player.scroll_drag(&id, 0.0, 50.0)?;               // raw pointer offset in px (clamped / rubber-banded)
        assert_eq!(player.scroll_offset(&id), Some([0.0, 50.0]));
        player.scroll_release(&id, 0.0, 900.0)?;           // fling at 900 px/s
        while player.scroll_settled(&id) == Some(false) {
            player.advance(1.0 / 60.0)?;                   // steps the machine and every scroll region
        }
        println!("{id}: max {max:?}, rests at {:?}", player.scroll_offset(&id).unwrap());
    }

    // Pick the coarsest level of detail whose declared error stays within 0.5 px at this screen size.
    player.set_screen_size(100.0, 120.0, 0.5)?;
    println!("level {} (declared bound {} px)", player.lod_level(), player.lod_bound_px());
    player.clear_screen_size();                            // back to the original
    assert_eq!(player.lod_level(), 0);

    // A scroll region whose `input` names a path follows set_value(path, px) instead of a drag.
    player.set_value("ui.scroll", 200.0)?;
    Ok(())
}
```

### Rendu sur plusieurs threads

```rust
use osatura_core::prelude::*;

/// `Animation` and `Player` are `Send + Sync`: read-only calls (`&self`) may run on several threads.
pub fn render_in_parallel() -> Result<(), ApiError> {
    let anim = Animation::demo();
    let frames = anim.frame_count();
    let settings = RenderSettings::new(0.25);
    let (a, s) = (&anim, &settings);                        // shared references are Copy: move them into the threads
    let images: Vec<Result<Image, ApiError>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..frames).map(|f| scope.spawn(move || a.render_rgba(f as f32, s))).collect();
        jobs.into_iter().map(|j| j.join().expect("render thread")).collect()
    });
    // Mutating calls (`&mut self`: set_key, undo, retarget_bvh ...) need exclusive access.
    for (f, img) in images.into_iter().enumerate() {
        assert_eq!(img?, anim.render_rgba(f as f32, &settings)?);   // threads do not change the result
    }
    Ok(())
}
```

### Contrôle de version et CI d'animation

```rust
use osatura_core::prelude::*;

/// Version control of a project file and animation CI, from Rust.
pub fn history(dir: &std::path::Path) -> Result<(), ApiError> {
    let file = dir.join("hero.json");
    std::fs::write(&file, Animation::demo().to_json()).unwrap();
    let path = file.to_str().unwrap();

    // A tiny helper: re-key the first bone at frame 5 and save the file.
    let edit = |angle: f32| -> Result<(), ApiError> {
        let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
        let bone = a.bone_names()[0].clone();
        a.set_key(&bone, "bone_angle", 5.0, angle, Interpolation::Linear)?;
        std::fs::write(path, a.to_json()).unwrap();
        Ok(())
    };

    vcs_init(path, Some("ana"))?;                           // branch `main`, no commit yet
    vcs_commit(path, Some("base"), None)?;
    vcs_branch(path, "other", None)?;                       // created, not switched to
    edit(20.0)?;
    vcs_commit(path, Some("main edit"), None)?;
    vcs_checkout(path, "other", false)?;                    // DirtyTree if the file has uncommitted changes
    edit(40.0)?;                                            // the same key, another value
    vcs_commit(path, Some("other edit"), None)?;
    vcs_checkout(path, "main", false)?;

    match vcs_merge(path, "other", false, None, None)? {
        VcsMergeOutcome::Conflicted(conflicts) => {         // a normal result, not an error
            for c in &conflicts {
                println!("{} ({}): {}", c.path, c.kind, c.description);
                vcs_resolve(path, &c.path, &VcsChoice::Theirs)?;
            }
            vcs_commit(path, None, None)?;                  // the merge commit
        }
        other => println!("merged without conflicts: {other:?}"),
    }
    for c in vcs_log(path, 10)? {
        println!("{} {} {}", &c.id[..8], c.author, c.message);
    }

    // Animation CI: write a calibrated manifest, record goldens, then verify.
    let init = ci_init(path, None, true)?;                  // never overwrites an existing manifest
    let run = ci_run(&init.manifest)?;                      // verify mode: nothing is re-recorded
    println!("{} checks, {} passed, exit code {}", run.total, run.passed, run.exit_code);
    if run.exit_code == 1 {
        // A failing check is a result, not an Err. After reviewing the report, accept the new look:
        ci_bless(&init.manifest)?;
    }
    Ok(())
}
```

### Mocap, fuzzing, tracé, niveau de détail et export à taille cible

```rust
use osatura_core::prelude::*;

const RIG: &str = r#"{"format":1,"name":"rig","width":400.0,"height":400.0,"fps":30.0,"frames":10,"background":null,
 "camera":{"position":null,"zoom":1.0,"rotation":0.0},
 "bones":[{"name":"hips","parent":null,"length":20.0,"rest_offset":[200.0,250.0],"rest_angle":-90.0},
          {"name":"spine","parent":"hips","length":40.0,"rest_offset":[20.0,0.0],"rest_angle":0.0},
          {"name":"head","parent":"spine","length":30.0,"rest_offset":[40.0,0.0],"rest_angle":0.0}],
 "ik":[],"constraints":[],"springs":[],"layers":[],"channels":[]}"#;

fn mini_bvh() -> String {
    let mut s = String::from("HIERARCHY\nROOT Hips\n{\n OFFSET 0 100 0\n CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation\n");
    s.push_str(" JOINT Spine\n {\n  OFFSET 0 12 0\n  CHANNELS 3 Zrotation Xrotation Yrotation\n  JOINT Head\n  {\n   OFFSET 0 12 0\n   CHANNELS 3 Zrotation Xrotation Yrotation\n   End Site\n   {\n    OFFSET 0 8 0\n   }\n  }\n }\n}\nMOTION\nFrames: 4\nFrame Time: 0.0333333\n");
    for f in 0..4 {
        s.push_str(&format!("0 100 0 0 0 0 {} 0 0 0 0 0\n", f * 10));
    }
    s
}

/// Motion capture, pose fuzzing, tracing, level of detail and size-targeted export.
pub fn tools() -> Result<(), ApiError> {
    // BVH retargeting: one undoable edit; bones are matched by name and topology.
    let mut rig = Animation::from_json(RIG)?;
    let report = rig.retarget_bvh(&mini_bvh(), &MocapSettings::new().with_smooth(1))?;
    println!("matched {:?}; {} frames on {} bones", report.matched, report.frames, report.bones_used);
    assert!(rig.undo());                                    // the whole retarget is one step

    // Pose fuzzing: where does the rig break?  (deterministic for a given seed)
    let demo = Animation::demo();
    let fuzz = demo.fuzz(50, 1)?;
    println!("{} of {} samples failed (max severity {:.2})", fuzz.failures, fuzz.samples, fuzz.max_severity);
    let _full_report: &str = &fuzz.json;                    // schema "osatura-fuzz-report" v1

    // Bitmap tracing: PNG/JPEG bytes -> SVG, or straight into an animation.
    let png = demo.render_png(0.0, &RenderSettings::new(0.25))?;
    let svg = trace_image_svg(&png, 6)?;
    let traced = Animation::from_image(&png, 6, "traced")?;
    println!("traced SVG: {} bytes; project has {} layers", svg.len(), traced.layer_names().len());

    // Level of detail with a declared error bound, one project JSON per level (level 0 = original).
    for l in demo.lod_levels(0.5, 1.0, 3)? {
        println!("L{}: bound {:.3} px, {} anchors, {} keys", l.level, l.declared_px, l.path_anchors, l.keys);
        let _reduced = Animation::from_json(&l.json)?;
    }

    // Smallest export within a byte budget.
    let fit = demo.fit_to_size(FitFormat::Lottie, 250_000)?;
    println!("met={} {} bytes, worst SSIM {:.3}", fit.met, fit.size, fit.ssim_min);
    Ok(())
}
```

### Comment les exemples sont vérifiés

Le code ci-dessus est le code qui a été exécuté : ce fichier est généré à partir des sources d'exemple, qui ont été compilées avec `cargo` contre `osatura-core` puis exécutées (les fichiers de projet utilisés sont la sortie de `osatura demo`, `take-record`, `clip add` et `machine add`, plus un projet avec un conteneur et une région de défilement). Les doc-tests de `api/` (`cargo test --doc -p osatura-core` : 19 doc-tests, dont 18 sur des fonctions de la façade dans `api/` et l'exemple de niveau crate dans `lib.rs` ; environ 170 s dans un build de debug, car chacun compile un programme) couvrent les wrappers de contrôle de version et de CI dans le dépôt lui-même.

## Autres langages

La façade est la source de vérité unique pour les autres bindings. Couverture :

| Capacité | Façade Rust | ABI C (`osatura.h`) | WebAssembly / JS |
|---|---|---|---|
| Charger, valider, infos, noms d'os / matrices monde | oui | `osatura_open`, `_bone_*`, `_frame_count`, ... | `arm_open`, `load()` (pas de matrices d'os) |
| Rendu RGBA | oui | `osatura_render_rgba` | `arm_render_rgba`, `rgba()` |
| Rendu SVG / PNG / formats d'export | oui | non | SVG oui (`svg()`), PNG / export non |
| Édition des clés, annuler / rétablir | oui | non | non |
| `Player` : entrées, advance, état, événements, rendu | oui | `osatura_player_*` | `arm_player_*`, `startPlayer()`, ... |
| Mise en page, défilement, niveau de détail | oui | `osatura_player_set_canvas`, `_scroll_*`, `_set_screen`, `_lod_level` | `arm_player_set_canvas / layout / scroll_* / set_screen` |
| Outils, contrôle de version, CI | oui | non | non |

La version de l'ABI C reste 1 parce que tous les ajouts jusqu'ici étaient additifs. Le binaire wasm est compilé et testé sous Node en CI mais n'a pas été exécuté dans un navigateur ; voir [WASM.md](WASM.md).

## Annexe : JSON de machine et JSON de binding

Brefs résumés des deux sections du projet que le `Player` exécute (format complet : [PROJECT_FORMAT.md](PROJECT_FORMAT.md)).

### JSON de machine (résumé)

```json
{"name":"hero",
 "inputs":[{"name":"moving","kind":"bool"},{"name":"speed","kind":"number"},{"name":"jump","kind":"trigger"}],
 "layers":[{"name":"body","mask":["arm_*"],"entry":"idle",
   "states":[{"name":"idle","source":{"kind":"clip","clip":"idle"},"blend_in":0.2},
             {"name":"loco","source":{"kind":"blend","input":"speed","children":[
                {"threshold":0,"source":{"kind":"clip","clip":"walk"}},{"threshold":1,"source":{"kind":"clip","clip":"run"}}]}},
             {"name":"jump","source":{"kind":"clip","clip":"jump"},"mode":"once"}],
   "transitions":[{"from":"idle","to":"loco","when":[{"input":"moving"}],"duration":0.25,"ease":{"type":"ease_in_out"}},
                  {"from":"*","to":"jump","when":[{"input":"jump"}],"duration":0.1},
                  {"from":"jump","to":"idle","exit_time":1.0}]}]}
```

Les conditions sont `{"input","op":"eq|ne|lt|le|gt|ge","value"}` (valeurs par défaut `eq`, `1` ; toutes doivent être vraies). `from: "*"` correspond à n'importe quel état sauf la cible. La première transition correspondante dans l'ordre de déclaration l'emporte. `exit_time` est la fraction jouée (0..1) de l'état. Sources : `clip` (une entrée de `Project.clips`), `segment` (images `start`, `end` des canaux en mode remplacement du projet), `region` (un marqueur avec une fin), `blend` (1-D, deux sources feuilles ou plus à des seuils croissants). Les clés inconnues dans une transition ou une condition constituent une erreur. CLI : `machine add|list|remove|validate`, `machine-sim <project> <script.json> <out_dir|out.json>` (script : `{"machine":"hero","duration":2,"events":[{"t":0.5,"set":{"moving":true},"trigger":["jump"],"value":{"player.health":40}}]}`).

### JSON de binding (résumé)

```json
{"name":"hp","path":"player.health","expr":"value / 100","remap":{"in_min":0,"in_max":1,"out_min":-30,"out_max":30},
 "smooth":0.15,"target":{"kind":"prop","target":"arm_up","prop":"bone_angle"}}
```

Pipeline : valeur, puis `expr` (variable `value` ; les fonctions d'expression du moteur, source d'au plus 4 Ko et imbrication d'au plus 64), puis `remap` (borné par défaut ; `curve` adoucit), puis lissage (constante de temps en secondes), puis la cible : `{"kind":"prop",...}` (toute propriété de canal, y compris `layer_opacity` et `layer_switch`) ou `{"kind":"input","machine","input"}` (une entrée de type trigger se déclenche sur le front montant). CLI : `bind add|list|remove`, `bind-eval`.

### Mise en page et normal maps par calque

Les conteneurs de mise en page et les régions de défilement sont des champs de projet `containers` / `scrolls` (omis lorsqu'ils sont vides) : CLI `container add|list|remove|solve`, `scroll add|list|remove|eval`, `layout-preview <project> <out.svg> --size WxH`, et `--size WxH` sur `render`, `frames` et `export` (les conteneurs sont recalculés pour cette taille de canevas ; le fichier reste inchangé). `Layer.normal_map` (`data` PNG en base64 ou `path`, `fit` stretch / tile / exact, `strength`, `green_down`) alimente l'effet de post-traitement `relight` pour les pixels que ce calque, ou un enfant sans carte propre, recouvre (CLI `layer-normal`, `fx ... --layer-normals`) ; il ne fait pas partie de la façade.
