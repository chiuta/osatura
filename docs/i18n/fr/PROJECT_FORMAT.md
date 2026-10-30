<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Format de projet OSATURA

Voici la référence du fichier de projet OSATURA : un unique document JSON qui
contient la scène, le rig, les illustrations et l'animation. Elle est rédigée à partir des
structs serde de `osatura-core/src/project.rs` et des modules qu'elles référencent,
et chaque exemple des sections 13 à 15 a été vérifié avec le binaire `osatura`
(`osatura check`, `osatura render`, `osatura explain`).

Sommaire

1. [Conventions et types primitifs](#1-conventions-et-types-primitifs)
2. [Chargement, versionnage et champs inconnus](#2-chargement-versionnage-et-champs-inconnus)
3. [Champs de premier niveau](#3-champs-de-premier-niveau)
4. [Rig : bones, ik, constraints, springs](#4-rig--bones-ik-constraints-springs)
5. [Calques](#5-calques)
6. [Tracés, ancres, remplissage, contour](#6-tracés-ancres-remplissage-contour)
7. [Déformateurs, effets, normal map](#7-déformateurs-effets-normal-map)
8. [Canaux, pistes, easing](#8-canaux-pistes-easing)
9. [Expressions](#9-expressions)
10. [Caméra, plans, audio, métadonnées](#10-caméra-plans-audio-métadonnées)
11. [Clips, skins, liens, interaction, lods](#11-clips-skins-liens-interaction-lods)
12. [Règles de validation, limites et messages d'erreur](#12-règles-de-validation-limites-et-messages-derreur)
13. [Exemple 1 : balle qui rebondit](#13-exemple-1--balle-rebondissante)
14. [Exemple 2 : bras riggé avec IK](#14-exemple-2--bras-riggé-avec-ik)
15. [Exemple 3 : bouche en calque switch](#15-exemple-3--bouche-à-calque-switch)
16. [Ordre d'évaluation](#16-ordre-dévaluation)
17. [Ordre de rendu](#17-ordre-de-rendu)
18. [Ressources et chemins](#18-assets-et-chemins)
19. [Politique de versionnage](#19-politique-de-versionnage)
20. [Écarts connus entre le code et les commentaires](#20-écarts-connus-entre-le-code-et-les-commentaires)

## 1. Conventions et types primitifs

| Type | Forme JSON | Remarques |
| --- | --- | --- |
| number | nombre JSON | Stocké en `f32` sauf si le champ indique un entier. Tout flottant doit satisfaire `abs(x) <= 1e7` (section 12). |
| `Vec2` | `[x, y]` | Deux nombres. Les unités de la scène sont des pixels, +x vers la droite, +y vers le bas. |
| `Rgba` | chaîne | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, ou l'un de `black`, `white`, `red`, `green` (0,128,0), `blue`, `yellow`, `transparent`. La sortie est toujours en hexadécimal minuscule. |
| `Paint` | chaîne ou objet | Une chaîne de couleur est un remplissage uni. `{"from": [x,y], "to": [x,y], "stops": [...]}` est un dégradé linéaire. `{"center": [x,y], "radius": r, "stops": [...]}` est un dégradé radial. Le type est une enum non étiquetée (untagged). |
| `Stop` | `{"offset": 0..1, "color": Rgba}` | Les stops sont triés (de façon stable) au moment du dessin et les offsets sont bornés à 0..1 dans le rastériseur. Un dégradé dégénéré peint le dernier stop. |
| angle | nombre | Des degrés partout dans le fichier (angles d'os, rotation de calque, skew, rotation de caméra, `angle_limits`). |
| time | nombre | Des images dans les clés et les marqueurs, des secondes dans les sons, les machines et les émetteurs. Chaque champ précise lequel. |
| names | chaîne | Noms d'os, de calques, de tracés, de déformateurs, d'ik, de contraintes. Les sections 5 et 12 listent les restrictions de caractères. |

Une mauvaise couleur dans un contexte de remplissage produit le message serde
`data did not match any variant of untagged enum Paint`, car `Paint` est non étiqueté et serde
ne peut pas dire quelle branche était visée.

Les tableaux de champs ci-dessous utilisent cette notation : **Req** signifie que le champ n'a pas de valeur par défaut et doit être présent ;
sinon la valeur par défaut est indiquée. « Range » désigne ce que le chargeur ou le moteur impose. Une plage notée
« clamped » n'est pas une erreur : le moteur borne silencieusement la valeur.

## 2. Chargement, versionnage et champs inconnus

`Project::from_json(text)` est le point d'entrée unique utilisé par la CLI, l'API C, le module WASM
et le Studio. Il effectue, dans l'ordre :

1. Analyser le JSON avec serde. Serde ignore les champs qu'il ne connaît pas.
2. Exécuter le contrôle de magnitude : tout littéral flottant dont la valeur absolue dépasse `1e7` fait échouer le chargement
   avec `invalid JSON in <file>: value ... is outside the supported range of +-1e7 ...`. Les littéraux
   entiers sont détectés plus tard, par la revérification dans `Engine::new` (voir la section 12).
3. Appliquer le verrou de format : un `format` supérieur au `FORMAT_VERSION` du build (actuellement `1`) est
   rejeté avec `project format 2 is newer than this build supports (1); update OSATURA`.

`Engine::new` (ou `Engine::with_asset_dir`) valide ensuite le projet entier et renvoie tous les
problèmes trouvés sous forme de liste. La CLI les affiche ainsi :

```text
<file> is not a valid project:
  - first problem
  - second problem
```

Les textes d'erreur s'adressent aux humains. Ils ne constituent pas un contrat stable ; ne les comparez que dans les tests.

### Champs inconnus

Serde supprime les clés inconnues au chargement ; une faute de frappe comme `fil` au lieu de `fill` serait donc perdue silencieusement. Le module
`unknownfields` compare le JSON d'entrée au projet resérialisé et signale les clés qui
n'ont pas survécu :

```text
warning: unknown field layers[0].paths[0].fil (ignored)
```

`osatura check` affiche ces avertissements. Les valeurs égales à la valeur par défaut ne sont pas signalées (l'écriture
omet les valeurs par défaut, donc un `"visible": true` explicite ne pose pas de problème). Quelques structures sont strictes plutôt que
tolérantes et rejettent les clés inconnues par une erreur franche : `Transition` et `Condition` des machines, configuration CI,
configuration d'entrée en direct, surcouches et configuration des paramètres de visage ; les fichiers de configuration sont lus avec `parse_strict`.

### Aller-retour des métadonnées d'éditeur

Le projet est sérialisé avec `skip_serializing_if` pour les listes facultatives. Un fichier écrit à la main peut
être plus court que ce qu'`osatura` réécrit ; les deux sont équivalents.

## 3. Champs de premier niveau

| Nom JSON | Type | Défaut | Plage / règle | Signification |
| --- | --- | --- | --- | --- |
| `format` | entier | `FORMAT_VERSION` (1) | doit être `<=` à la version du build | Version du format, section 19. |
| `name` | chaîne | **Req** | quelconque | Nom d'affichage. Affiché par `check`. |
| `width` | nombre | **Req** | `> 0`, `<= 32768` | Largeur de la scène en pixels. |
| `height` | nombre | **Req** | `> 0`, `<= 32768` | Hauteur de la scène en pixels. |
| `fps` | nombre | **Req** | `(0, 1000]` | Images par seconde. |
| `frames` | entier (`u32`) | **Req** | `1..=200000` | Longueur de la timeline. Les indices d'images vont de `0..frames-1`. Un littéral flottant tel que `48.0` est rejeté : `invalid type: floating point `48.0`, expected u32`. |
| `background` | `Rgba` ou null | null (transparent) | | Couleur de la scène derrière tout le reste. |
| `color_space` | chaîne | `"srgb"` (non écrit) | `srgb`, `linear` | Espace dans lequel les compositeurs raster et flottant mélangent. `linear` : les couleurs sont converties en lumière linéaire, composées en f32 puis reconverties une seule fois (un noir à 50 % sur blanc donne 188/255, et non 128/255). Se règle avec `osatura color-space`. SVG, Lottie, le moteur de rendu GPU, `dof` et `fx` l'ignorent. |
| `camera` | objet | valeurs par défaut | section 10 | Caméra globale. |
| `bones` | tableau | `[]` | au plus 4096 | Squelette, section 4. |
| `ik` | tableau | `[]` | | Chaînes d'IK. |
| `constraints` | tableau | `[]` | | Aim, copy-rotation, pin. |
| `springs` | tableau | `[]` | | Mouvement secondaire, baké au chargement. |
| `layers` | tableau | `[]` | | Calques racines, dessinés dans l'ordre de la liste. |
| `channels` | tableau | `[]` | | Canaux d'animation, évalués dans l'ordre de la liste. |
| `markers`, `comps`, `events`, `bone_vis`, `channel_meta`, `shots` | tableaux | omis lorsque vides | non validés | Métadonnées de timeline et d'éditeur, section 10. |
| `sounds` | tableau | omis lorsque vide | validés | Pistes audio, section 10. |
| `skins`, `clips`, `clip_baked`, `lods` | tableaux | omis lorsque vides | non validés au chargement | Section 11. |
| `mesh_locks`, `glues` | tableaux | omis lorsque vides | validés | Liens de maillage entre calques, section 11. |
| `machines`, `bindings`, `containers`, `scrolls` | tableaux | omis lorsque vides | validés | Interaction et mise en page, section 11. |

Projet valide minimal :

```json
{"name": "empty", "width": 100, "height": 100, "fps": 24, "frames": 1}
```

Magnitudes : tout flottant du fichier doit satisfaire `abs(x) <= 1e7`. Cela s'applique aux coordonnées,
échelles, décalages et valeurs de clés sans distinction. La largeur et la hauteur sont en outre limitées à 32768, et la sortie
raster à 16384 par côté et 100 mégapixels (section 18).

## 4. Rig : bones, ik, constraints, springs

### 4.1 `bones[]`

Les os doivent être listés en commençant par les parents. Le parent d'un os doit apparaître plus tôt dans la liste.

| Nom JSON | Type | Défaut | Plage | Signification |
| --- | --- | --- | --- | --- |
| `name` | chaîne | **Req** | unique | Référencé par les poids, ik, canaux, `Layer.bone`. |
| `parent` | chaîne ou null | null | un os antérieur | Os parent. Null désigne une racine. |
| `length` | nombre | **Req** | fini, `>= 0` | Longueur de l'os en pixels, le long de l'axe +x de l'os. |
| `rest_offset` | `Vec2` | `[0,0]` | | Position de la tête dans le repère du parent (repère de la scène pour les racines). |
| `rest_angle` | nombre | 0 | | Rotation de repos en degrés relative au parent. |
| `angle_limits` | `[min, max]` ou null | null | `min <= max` | Borne le delta d'angle animé, en degrés. |
| `scale_limits` | `[min, max]` ou null | null | `0 < min <= max` | Borne l'échelle animée. |

Interactions : les canaux `bone_angle` s'ajoutent à `rest_angle` ; `bone_offset_x/y` s'ajoutent à `rest_offset` ;
`bone_scale` multiplie. `clamp_limits` s'exécute après tous les canaux et avant la FK, puis de nouveau après chaque
chaîne d'IK.

```json
{"name": "upper", "parent": null, "length": 90, "rest_offset": [60, 120],
 "rest_angle": 0, "angle_limits": [-120, 120]}
```

### 4.2 `ik[]`

| Nom JSON | Type | Défaut | Plage | Signification |
| --- | --- | --- | --- | --- |
| `name` | chaîne | **Req** | unique | Cible des canaux `ik_mix`. |
| `bones` | tableau de chaînes | **Req** | au moins 2 ; chacun enfant direct du précédent | Chaîne de la racine à l'extrémité. |
| `target` | chaîne | **Req** | un os, hors de la chaîne | Os dont la tête est visée par l'extrémité. |
| `bend` | nombre | 1 | +1 ou -1 | Sens dans lequel se plie une chaîne de deux os. |
| `mix` | nombre | 1 | borné à 0..1 | Mélange FK-vers-IK initial ; s'anime avec `ik_mix`. `<= 0` ignore la chaîne. |
| `iterations` | entier | 16 | | Passes FABRIK pour les chaînes de plus de deux os. C'est un plancher : le solveur s'arrête plus tôt quand l'extrémité est à moins de 0.01 px, et sinon continue jusqu'à 256 passes. |
| `soft` | nombre | 0 | `0..0.9` | Fraction de la portée sur laquelle l'extension s'amortit. Avec `stretch > 1`, la zone correspond à la dernière fraction de la portée étirée (`reach x stretch`). |
| `stretch` | nombre | 1 | `1..4` | Facteur d'étirement maximal ; au-dessus de 1, l'échelle de l'os racine croît pour atteindre la cible. |

Une chaîne de deux os utilise le solveur analytique de la loi des cosinus ; les chaînes plus longues utilisent FABRIK. Après la résolution,
les angles sont mélangés dans la pose selon `old + wrap_pi(new - old) * mix`.

```json
{"name": "reach", "bones": ["upper", "lower"], "target": "target", "bend": 1, "mix": 1}
```

### 4.3 `constraints[]`

Les types de contrainte utilisent une étiquette interne `"type"`, aplatie à côté de `name`, `bone` et `weight`.

| Nom JSON | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `name` | chaîne | **Req** | Cible des canaux `constraint_weight`. |
| `bone` | chaîne | **Req** | L'os contraint. |
| `type` | `aim`, `copy_rotation`, `pin` | **Req** | Genre. |
| `target` | chaîne | **Req** pour `aim`, `pin` | Os à viser ou sur lequel glisser. |
| `offset` | nombre | 0 | `aim` uniquement : degrés supplémentaires. |
| `source` | chaîne | **Req** pour `copy_rotation` | Os dont la rotation monde est copiée. |
| `weight` | nombre | 1 | Influence, bornée à 0..1. `<= 0` ignore la contrainte. |

Une contrainte ne peut pas se cibler elle-même. Les contraintes s'exécutent dans l'ordre de la liste après toutes les chaînes d'IK.

```json
{"name": "aim", "bone": "root", "type": "aim", "target": "look", "offset": 0, "weight": 0.5}
```

### 4.4 `springs[]`

Un ressort amorti qui fait traîner la rotation monde d'un os derrière sa rotation animée. Il est
simulé une fois au chargement puis baké, de sorte que n'importe quelle image peut être évaluée dans n'importe quel ordre.

| Nom JSON | Type | Défaut | Plage | Signification |
| --- | --- | --- | --- | --- |
| `bone` | chaîne | **Req** | un ressort par os | Os à piloter. |
| `frequency` | nombre | 3 | `(0, 30]` Hz | Raideur. |
| `damping` | nombre | 0.35 | `>= 0` | Rapport d'amortissement ; en dessous de 1 le ressort oscille, 1 est critique. |
| `warmup` | bool | true | | Précalcule une passe afin que les boucles démarrent en régime établi. |

Les angles bakés remplacent l'angle de l'os après les contraintes (étape d'évaluation 8).

## 5. Calques

Un calque est `{"name": ..., "type": ..., <kind fields>, <common fields>}`. `type` est l'étiquette de
`LayerKind` ; ses champs sont aplatis dans l'objet calque.

### 5.1 Champs communs

| Nom JSON | Type | Défaut | Plage | Signification |
| --- | --- | --- | --- | --- |
| `name` | chaîne | **Req** | non vide, sans `/`, sans `::` ; le chemin complet doit être unique | Nom du calque. Les enfants sont adressés sous la forme `parent/child`. |
| `type` | chaîne | **Req** | `vector`, `group`, `switch`, `liquid`, `boolean`, `emitter`, `image` | Genre. |
| `transform` | objet | identité | | `position` `[0,0]`, `rotation` 0, `scale` `[1,1]`, `skew` 0, `anchor` `[0,0]`. |
| `opacity` | nombre | 1 | bornée à 0..1 | Se multiplie dans la composition du calque. |
| `visible` | bool | true | | Les calques masqués sont ignorés au dessin mais peuvent toujours servir de source de masque. |
| `blend` | chaîne | `normal` | voir ci-dessous | Mode de fusion. |
| `mask` | `{layer, invert}` | null | pas d'auto-masque, pas de cycles | Utilise l'alpha d'un autre calque comme masque. |
| `bone` | chaîne | null | un os connu | Rattache à un os : la matrice de skin de l'os est appliquée entre la matrice de scène du parent et la transformation locale. |
| `parallax` | nombre | 1 | | Facteur de profondeur de caméra. Seule la valeur du calque de premier niveau compte. |
| `deformers` | tableau | omis lorsque vide | calques vector et image uniquement | Section 7. |
| `wobble` | objet | null | | `amp`, `scale` (0.03), `rate` (8), `seed` (0) : déplacement par bruit déterministe. Animez `amp` avec `layer_wobble`. |
| `reference` | bool | false | | Calque de référence : masqué sauf si l'éditeur affiche les références. |
| `locked` | bool | false | | Verrou d'éditeur, voir ci-dessous. |
| `normal_map` | objet | null | | Section 7. |
| `effects` | tableau | omis lorsque vide | au plus 16 | Section 7. |

Matrice locale : `T(position) * R(rotation) * Shear(skew) * S(scale) * T(-anchor)`.

Modes de fusion : `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`, `color_dodge`,
`color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `add` (plus-lighter de CSS).

`locked` ne bloque que les opérations d'éditeur : sélection sur le canevas, gizmos, déplacements au clavier, suppressions, modifications de la timeline et
modifications de canaux (via `assert_unlocked`), pour le calque et ses descendants. Le rendu et
l'évaluation l'ignorent, et les `Op` simples (section « Editor ops » de ARCHITECTURE.md) s'appliquent toujours.

Les champs communs sont écrits après les champs du genre en sortie, mais l'ordre est sans importance en entrée.

### 5.2 `vector`

| Champ | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `paths` | tableau de tracés | `[]` | Section 6. |

### 5.3 `group`

| Champ | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `layers` | tableau de calques | `[]` | Les enfants héritent de la transformation du groupe. |

### 5.4 `switch`

| Champ | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `layers` | tableau de calques | `[]` | Dessins candidats. |
| `selected` | entier | 0 | Indice initial de l'enfant. S'anime avec `layer_switch` (clés en escalier). |

Un seul enfant est affiché. La valeur est arrondie ; une valeur négative devient 0 ; les valeurs égales ou supérieures au nombre
d'enfants deviennent le dernier enfant. La validation ne rejette pas les clés hors plage. Voir la section 20 pour une
discordance de formulation du lint à ce sujet.

### 5.5 `liquid`

Liquide à métaballes. Le champ est la somme de `r^2 / d^2` sur les blobs ; le contour est l'iso-contour,
extrait par marching squares sur une grille plafonnée à 600 par 600 cellules et lissé par Catmull-Rom.

| Champ | Type | Défaut | Plage | Signification |
| --- | --- | --- | --- | --- |
| `blobs` | tableau de `{pos, radius, bone?}` | **Req** | au moins 1 ; `radius > 0` | Un blob. `bone` le rattache à un os. |
| `threshold` | nombre | 1 | `> 0` | Iso-niveau ; 1 fait d'un blob isolé un cercle de son rayon. |
| `cell` | nombre | 3 | `>= 0.25` | Cellule de la grille en unités de calque. |
| `fill` | `Paint` | null | | |
| `stroke` | objet stroke | null | | |

### 5.6 `boolean`

| Champ | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `op` | `union`, `intersect`, `difference`, `xor` | **Req** | Opération. |
| `paths` | tableau de tracés | **Req**, au moins 2 | `paths[0]` est le sujet ; le clip est l'union des autres. |

Le résultat reprend le remplissage et le contour du sujet ; la tolérance d'aplatissement est de 0.25. Les tracés résultants sont
nommés `<first path name><index>`.

### 5.7 `emitter`

`{"type": "emitter", "spec": {...}}`. Les particules sont une fonction explicite du temps, de sorte que n'importe quelle image
est évaluable. Les temps sont en secondes.

| Champ de `spec` | Défaut | Plage | Signification |
| --- | --- | --- | --- |
| `pos` | `[0,0]` | | Position de l'émetteur (repère du calque). |
| `bone` | null | | Rattache à un os. |
| `rate` | 20 | `(0, 2000]` | Particules par seconde. |
| `lifetime` | 1 | `> 0` | Secondes. |
| `lifetime_jitter` | 0 | 0..1 | |
| `speed` | 60 | | Pixels par seconde. |
| `speed_jitter` | 0 | 0..1 | |
| `direction` | -90 | | Degrés ; -90 est vers le haut. |
| `spread` | 30 | | Degrés de largeur du cône. |
| `gravity` | `[0,0]` | | Accélération, en pixels par seconde au carré. |
| `size_start`, `size_end` | 3, 3 | | Rayon au cours de la vie. |
| `color_start`, `color_end` | `#fff`, `#fff0` | | Couleur au cours de la vie. |
| `seed` | 0 | | Graine déterministe. |
| `start` | 0 | | Instant de début en secondes. |
| `duration` | null | | L'émission s'arrête après ce nombre de secondes ; null signifie indéfiniment. |
| `max_particles` | 2000 | | Plafond strict de particules vivantes. |

### 5.8 `image`

| Champ | Type | Défaut | Plage | Signification |
| --- | --- | --- | --- | --- |
| `source` | `{file?, png_base64?}` | **Req** | l'un des deux | `file` est un chemin relatif au dossier du projet (section 18) ; `png_base64` est autonome. Ni l'un ni l'autre : `image has neither 'png_base64' nor 'file'`. |
| `origin` | `Vec2` | `[0,0]` | | Coin supérieur gauche dans le repère du calque. |
| `size` | `Vec2` ou null | null (taille en pixels) | | Taille affichée. |
| `mesh` | objet ou null | null | | Maillage de déformation. |

`mesh` : `cols`, `rows` (maillage en grille, chacun au moins 1), `weights` (poids d'os par sommet ; vide
signifie statique), et éventuellement `free` = `{vertices, triangles, uvs}`. Un maillage libre exige au moins 3
sommets, au moins 1 triangle, au plus 200000 sommets et 400000 triangles, des coordonnées finies,
des indices de triangles valides, et des `uvs` soit vides, soit de même longueur que `vertices`. Avec des déformateurs mais
sans maillage, une grille de 16 par 16 est utilisée.

```json
{"name": "img2", "type": "image", "source": {"png_base64": "iVBOR..."},
 "origin": [100, 0], "size": [80, 80],
 "mesh": {"free": {"vertices": [[100,0],[180,0],[180,80],[100,80]],
                   "triangles": [[0,1,2],[0,2,3]],
                   "uvs": [[0,0],[8,0],[8,8],[0,8]]}, "weights": []}}
```

## 6. Tracés, ancres, remplissage, contour

### 6.1 Tracé (`paths[]`)

| Nom JSON | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `name` | chaîne | `""` | Un nom vide est remplacé par `path{index}` pour l'adressage. Ne doit pas contenir `::`. Unique au sein du calque. |
| `closed` | bool | false | |
| `anchors` | tableau | **Req** | Ancres de Bézier. |
| `fill` | `Paint` | null | |
| `stroke` | stroke | null | |
| `morphs` | tableau de `{name, anchors}` | omis lorsque vide | Formes de mélange additives (blend shapes). Le nombre d'ancres doit être égal à celui du tracé de base. Poids via `path_morph`. |

### 6.2 Ancre

| Nom JSON | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `pos` | `Vec2` | **Req** | Position. |
| `handle_in`, `handle_out` | `Vec2` | null | Points de contrôle dans le même repère que `pos` (absolus, non relatifs). Une poignée absente équivaut à `pos` (un coin). |
| `weights` | tableau de `{bone, weight}` | omis lorsque vide | Skinning. Chaque os doit exister et chaque poids est fini et `>= 0`. Les poids sont normalisés à l'évaluation ; vide signifie statique. Les poignées sont skinnées aussi. |

### 6.3 Contour (stroke)

| Nom JSON | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `paint` | `Paint` | **Req** | |
| `width` | nombre | 1 | |
| `cap` | `butt`, `round`, `square` | `round` | |
| `join` | `miter`, `round`, `bevel` | `round` | Une valeur inconnue donne ``unknown variant `foo`, expected one of `miter`, `round`, `bevel` ``. |
| `dash` | tableau | omis | Utilisé seulement si sa longueur est paire, au moins 2, et si chaque entrée est `> 0`. |
| `dash_offset` | nombre | 0 | |
| `width_profile` | tableau de `[t, factor]` | omis | Multiplicateur de largeur linéaire par morceaux le long de la longueur (`t` dans 0..1), constant au-delà des extrémités, limite de mitre 2. Un contour à largeur variable ignore `dash` et `join`. |

Exemple de remplissage en dégradé radial avec un contour (issu de l'exemple de la balle) :

```json
"fill": {"center": [-8, -10], "radius": 44,
         "stops": [{"offset": 0, "color": "#ffd9a0"}, {"offset": 1, "color": "#d6336c"}]},
"stroke": {"paint": "#3d2c2e", "width": 2}
```

Le linter (`osatura lint`) avertit des problèmes de dégradé (L011), de clés de switch (L012) et de données d'éditeur (L013) ; voir
`osatura-core/src/lint.rs`.

## 7. Déformateurs, effets, normal map

### 7.1 `deformers[]`

Autorisés uniquement sur les calques `vector` et `image`. Chacun a une étiquette `"type"` et un `name` (unique au sein
du calque, sans `::`). Ils s'exécutent dans l'ordre de la liste sur la géométrie en espace de repos, avant le skinning et avant
`wobble`. Les décalages sont statiques ; animez-les avec les canaux de sous-cible.

| `type` | Champs | Contraintes | S'anime avec (cible) |
| --- | --- | --- | --- |
| `lattice` | `cols`, `rows`, `rect` `[x,y,w,h]`, `offsets` | `cols, rows >= 1` ; `w, h > 0` ; `offsets` vide ou `(cols+1)*(rows+1)` ligne par ligne | `lattice_dx/dy` (`layer::name::c,r`) |
| `spine` | `rest` (points de la courbe), `offsets`, `radius` (bord doux facultatif) | au moins 2 points | `spine_dx/dy` (`layer::name::i`) |
| `pin` | `pins`, `offsets`, `radius` | 1 à 64 pins, `radius > 0`, pins non confondus | `pin_dx/dy` (`layer::name::i`) |
| `cage` | `rest` (polygone), `offsets` | 3 à 256 sommets, polygone simple, aire non nulle | `cage_dx/dy` (`layer::name::i`) |
| `cloth` | comme `lattice` | comme `lattice` | `lattice_dx/dy` (`layer::name::c,r`) |

Lattice mélange les décalages avec des poids de Bézier-Bernstein (lisse) ; cloth déplace chaque sommet de la grille
exactement (bilinéaire). Pin utilise une fonction de base radiale à support compact de Wendland. Cage utilise
les coordonnées de valeur moyenne de Floater : le contenu à l'intérieur du polygone le suit, le contenu à l'extérieur est laissé
tel quel.

```json
"deformers": [
  {"type": "lattice", "name": "lat", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []},
  {"type": "spine", "name": "sp", "rest": [[0,0],[50,0],[100,0]], "offsets": [], "radius": 30},
  {"type": "pin", "name": "pn", "pins": [[0,0],[100,100]], "offsets": [], "radius": 80},
  {"type": "cage", "name": "cg", "rest": [[0,0],[100,0],[100,100],[0,100]], "offsets": []},
  {"type": "cloth", "name": "cl", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []}
]
```

### 7.2 `effects[]`

Au plus 16 par calque. Chaque effet a un `"type"`, `enabled` (true par défaut) et les champs ci-dessous. La
cible de canal `fxN` indexe le tableau en comptant les effets désactivés.

| `type` | Champs | Plage |
| --- | --- | --- |
| `blur` | `radius` | 0..64 |
| `drop_shadow` | `dx`, `dy`, `blur`, `color`, `opacity` (1) | décalages au plus 1024 ; blur 0..64 ; opacity 0..1 |
| `tint` | `black`, `white`, `amount` (1) | amount 0..1 |
| `fill` | `color`, `opacity` (1) | opacity 0..1 |

Animez avec `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` sur la cible `layer::fxN`.

### 7.3 `normal_map`

`{"data": base64-PNG | "path": file, "fit": "stretch" | "tile" | "exact" (default stretch),
"strength": 0..8 (default 1), "green_down": bool (default false)}`. Exige `data` ou `path`. Elle alimente
le post-effet `relight` ; les rendus simples ne l'utilisent pas. En tant que `Layer.normal_map`, elle est ajustée à la boîte englobante en pixels de ce calque. `fit` `exact` exige une carte exactement à la taille de l'image et constitue une erreur dans le cas contraire.

## 8. Canaux, pistes, easing

### 8.1 Canal

```json
{"target": "ball", "prop": "layer_y", "source": "keys",
 "track": {"keys": [{"frame": 0, "value": 40}, {"frame": 24, "value": 170}]},
 "mode": "replace", "weight": 1}
```

| Nom JSON | Type | Défaut | Signification |
| --- | --- | --- | --- |
| `target` | chaîne | `""` | Nom, ou forme `layer::sub`. Ignoré pour les props de caméra. |
| `prop` | chaîne | **Req** | L'une des props en snake_case ci-dessous. |
| `source` | `keys`, `expr`, `noise`, `driven` | **Req** | Étiquette aplatie, avec les champs de cette source à côté. |
| `mode` | `replace`, `add`, `multiply` | `replace` | Façon dont la valeur se combine avec la valeur courante. |
| `weight` | nombre | 1 | Influence, doit être finie ; mélange vers le résultat. |

Combinaison : replace vaut `cur + (v - cur) * w` ; add vaut `cur + v * w` ; multiply vaut
`cur * (1 + (v - 1) * w)`. Les valeurs non finies sont ignorées. Les canaux s'exécutent dans l'ordre de la liste, de sorte qu'un canal
postérieur voit les résultats des précédents (via `get(...)` et `driven`).

Sources :

| `source` | Champs |
| --- | --- |
| `keys` | `track` : `{keys: [{frame, value, ease?}], extrapolate?}` |
| `expr` | `expr` : chaîne (section 9) |
| `noise` | `amp`, `freq` (cycles par seconde), `seed` (0), `octaves` (1). Sortie dans `[-amp, amp]` ; fondé sur un hachage, déterministe. |
| `driven` | `input` (cible), `input_prop`, `curve` : une piste dont les « images » sont des valeurs d'entrée (par exemple de l'angle du coude vers la quantité de renflement). |

### 8.2 Props

Il existe 39 props. La prop détermine ce que signifie `target`.

| Groupe | Props | `target` |
| --- | --- | --- |
| Os | `bone_angle`, `bone_scale`, `bone_offset_x`, `bone_offset_y` | nom d'os |
| Calque | `layer_opacity`, `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`, `layer_skew`, `layer_switch`, `layer_wobble` | chemin de calque (`parent/child`) |
| Caméra | `camera_x`, `camera_y`, `camera_zoom`, `camera_rotation`, `camera_focus`, `camera_blur` | ignoré |
| Rig | `ik_mix`, `constraint_weight` | nom d'ik ou de contrainte |
| Tracé | `path_morph`, `point_dx`, `point_dy`, `grad_dx`, `grad_dy` | `layer::path::morphName`, `layer::path::anchorIndex`, `layer::path::fill.from` / `fill.to` / `stroke.from` / `stroke.to` |
| Déformateur | `lattice_dx/dy`, `spine_dx/dy`, `pin_dx/dy`, `cage_dx/dy` | `layer::deformer::c,r` (lattice, cloth) ou `layer::deformer::i` |
| Effet | `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` | `layer::fxN` |

Sémantique des poignées de dégradé : pour les dégradés linéaires, `.from` et `.to` déplacent le point de départ et le point d'arrivée de
`(dx, dy)`. Pour les dégradés radiaux, `.from` déplace le centre et `.to` modifie le rayon de `dx`.
Seuls les tracés dont le remplissage ou le contour est un dégradé possèdent ces cibles. `layer_switch` exige un calque
`switch`. Une prop inconnue liste toutes les variantes valides dans l'erreur.

Bornes à l'écriture : opacité 0..1, zoom de caméra au moins 0.01, flou de caméra au moins 0, `ik_mix` et
`constraint_weight` 0..1.

### 8.3 Piste et clé

`Track` = `{"keys": [...], "extrapolate": "hold"}`. Une clé est `{"frame": number, "value": number,
"ease": {...}}`. Les clés doivent être finies et strictement croissantes en image :
`channel N ('target'): keys must be finite and strictly increasing in frame`. L'ease d'une clé
façonne le segment qui commence à cette clé.

Extrapolation (en dehors de la première et de la dernière clé) : `hold` (par défaut), `linear`, `loop`, `ping_pong`,
`loop_offset`.

### 8.4 Easing

`ease` est un objet avec une étiquette `type`. La valeur par défaut est `{"type": "linear"}`.

| `type` | Paramètres | Courbe |
| --- | --- | --- |
| `linear` | | `u` |
| `step` | | Maintient la valeur de la clé jusqu'à la clé suivante. |
| `ease_in` | | `u^2` |
| `ease_out` | | `1 - (1-u)^2` |
| `ease_in_out` | | Smoothstep. |
| `cubic_bezier` | `x1, y1, x2, y2` | Courbe de style CSS passant par (0,0) et (1,1). |
| `smooth` | | Hermite de Catmull-Rom non uniforme passant par les clés voisines. |
| `spring` | `bounces`, `decay` | `1 - e^(-decay*u) * cos(2*pi*bounces*u) * window`, window = `1 - smooth01((u-0.85)/0.15)` ; atterrit exactement sur la cible. |
| `back` | `overshoot` | Cubique avec une pente de départ `3 + overshoot` (bornée à 0 ou plus). Overshoot 1 culmine à 1.037 (u = 2/3) ; overshoot 2 culmine à 1.1317 (u = 0.5556). |
| `bounce` | | Restitution 0.6, trois rebonds, durée totale 3.352 unités de temps. Premier impact à u = 0.2983 ; sommets des rebonds 0.64, 0.8704, 0.953344 ; se termine à 1. |

`back` et `bounce` sont des dérivations originales (voir `CLEANROOM.md`).

## 9. Expressions

Un canal avec `"source": "expr"` évalue une formule par image. La source est limitée à 4096 octets
et à une profondeur d'imbrication de 64 ; les erreurs sont signalées à la validation avec une position, par exemple
`channel 2 ('upper'): expression error: unexpected end of expression`.

Variables : `frame`, `time` (= frame / fps), `fps`, `frames`, `pi`, `t` (= frame / frames).

Opérateurs (de la priorité la plus basse à la plus haute) : `?:`, `||`, `&&`, comparaisons, `+ -`, `* / %`, unaires `- !`,
`^` (associatif à droite). La division par zéro donne 0 ; `%` est euclidien (`rem_euclid`).

Fonctions : `sin cos tan asin acos atan atan2 abs sqrt pow min max clamp lerp smoothstep floor ceil
round fract sign mod exp ln noise` (1 ou 2 arguments) `fbm` (2 ou 3) `wiggle` (2 ou 3) `saw tri
square deg rad`, et `get("target", "prop")`. Un résultat de fonction non fini devient 0.

`get("target", "prop")` lit la valeur courante de n'importe quelle propriété, y compris les résultats des canaux
précédents. Les deux chaînes doivent se résoudre à la validation (`channel 2: get("q", "bone_angle"): unknown
target`). Un canal qui lit une valeur doit venir après le canal qui l'écrit.

Exemple issu du projet de la balle :

```json
{"target": "shadow", "prop": "layer_opacity", "source": "expr",
 "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"}
```

Les expressions de liaison (section 11) utilisent une forme restreinte : la variable est `value`, et les variables de temps
ainsi que `get()` sont interdites.

## 10. Caméra, plans, audio, métadonnées

### 10.1 `camera`

| Nom JSON | Type | Valeur par défaut | Plage | Signification |
| --- | --- | --- | --- | --- |
| `position` | `Vec2` ou null | null (centre de la scène) | | Point du monde situé au centre de l'écran. |
| `zoom` | nombre | 1 | borné à 0,01 au minimum | |
| `rotation` | nombre | 0 | degrés | |
| `focus` | nombre | 1 | | Profondeur de `parallax` du calque qui est nette (utilisée par `dof` et `dof-export`). |
| `blur` | nombre | 12 | fini, `>= 0` | Rayon de flou en pixels par unité de distance de parallaxe. |

Parallaxe : un calque avec `parallax = p` voit `zoom = 1 + (zoom - 1) * p`,
`pos = centre + (pos - centre) * p`, `rotation = -rotation * p`. Un `focus` ou un `blur` non fini
produit `camera focus must be finite and camera blur a finite number >= 0`.

### 10.2 Tableaux de métadonnées (non validés au chargement)

Ces tableaux transportent des données d'éditeur et d'outillage. Le moteur les accepte comme données ; les opérations qui les utilisent
les vérifient lors de leur exécution.

| Tableau | Champs d'un élément |
| --- | --- |
| `markers` | `name`, `frame`, `end` (facultatif ; une région lorsqu'il est défini) |
| `comps` | `name`, `layers` : table associant un chemin de calque à `{visible, opacity, selected?}` |
| `events` | `name`, `frame`, `data` (JSON quelconque) |
| `bone_vis` | `bone`, `from`, `to` (plage d'images dans laquelle l'os est affiché) |
| `channel_meta` | `target`, `prop`, `color`, `shy`, `solo`, `lock` (indicateurs d'éditeur ; `lock` bloque set-key) |
| `shots` | `name`, `x`, `y`, `zoom`, `rotation` (vues de caméra enregistrées) |

```json
"markers": [{"name": "intro", "frame": 0, "end": 12}, {"name": "hit", "frame": 20}],
"events": [{"name": "step", "frame": 12, "data": {"sound": "step.wav"}}],
"shots": [{"name": "wide", "x": 200, "y": 150, "zoom": 1, "rotation": 0}]
```

### 10.3 `sounds[]`

| Nom JSON | Type | Plage | Signification |
| --- | --- | --- | --- |
| `name` | chaîne | unique | |
| `file` | chaîne | un chemin d'asset (section 18) | WAV ou autre format audio pris en charge. |
| `start` | nombre | secondes | Placement sur la timeline. |
| `gain` | nombre | 0..4 | |
| `trim_in`, `trim_out` | nombre | `trim <= 86400` | Secondes retirées au début et à la fin du fichier. |
| `fade_in`, `fade_out` | nombre | tient dans le clip rogné | Secondes. |

`check` (et non `Engine::new`) signale un fichier manquant : `sound 's': cannot read sound file ./nofile.wav: ...`.

## 11. Clips, skins, liens, interaction, lods

### 11.1 `clips[]` et `clip_baked`

Un clip place une `take` réutilisable sur la timeline. Le bake des clips ajoute des canaux ordinaires (après ceux
créés à la main) et enregistre un instantané dans `clip_baked` ; un nouveau bake est donc idempotent et le travail manuel n'est
jamais supprimé.

| Champ du clip | Signification |
| --- | --- |
| `name`, `start`, `speed`, `loops` | Placement. |
| `take` | `{name, length, tracks: [{target, prop, keys}]}` |
| `mode` | `replace`, `additive`, `multiply` |
| `weight`, `weight_keys` | Influence ; piste animée facultative. |
| `include`, `exclude` | Listes de motifs sélectionnant des pistes. |
| `track_weights` | `[{pattern, prop, weight}]` |
| `fade_in`, `fade_out` | Images. |
| `rename` | `[[from, to]]` renommages de cibles. |

### 11.2 `skins[]`

`{name, group, layers, constraints, iks, bones}`. Les skins d'un même `group` sont des variantes exclusives.
L'échelle d'os propre à chaque skin est regroupée en un canal en escalier `bone_scale` de type multiply lors du bake. Les règles des skins
ne sont vérifiées que lorsque des opérations sur les skins s'exécutent.

### 11.3 `mesh_locks[]` et `glues[]`

Une paire de glue utilise `VertexRef` = `{layer, path (default 0), index}` ; un mesh lock désigne des chemins entiers avec `{layer, path}` et une liste facultative d'indices de sommets suiveurs (vide signifie tous les sommets).

```json
"mesh_locks": [{"name": "lock", "host": {"layer": "hostm", "path": 0},
                "follower": {"layer": "follm", "path": 0}, "vertices": [], "strength": 1}],
"glues": [{"name": "g", "mode": "rigid", "strength": 1,
           "pairs": [{"a": {"layer": "hostm", "path": 0, "index": 0},
                      "b": {"layer": "s1", "path": 0, "index": 0}}]}]
```

Le `mode` d'une glue est `snap`, `rigid` ou `offset`. Les deux sont validés par `links::resolve` et appliqués dans
l'espace de la scène dans l'ordre topologique (étape 12 de l'évaluation).

### 11.4 `machines[]` et `bindings[]` (à l'exécution uniquement)

Les machines à états et les liaisons de données sont pilotées par le Player à l'exécution. Pour l'export et le bake, elles
deviennent des canaux `replace` constants ajoutés au projet. Chaque machine requiert au moins un calque.

```json
"machines": [{"name": "m",
  "inputs": [{"name": "go", "kind": "bool", "default": 0},
             {"name": "fire", "kind": "trigger", "hold": false}],
  "layers": [{"name": "main", "entry": "idle", "mask": [],
    "states": [
      {"name": "idle", "source": {"kind": "region", "marker": "intro"},
       "mode": "loop", "speed": 1, "blend_in": 0.1},
      {"name": "wave", "source": {"kind": "clip", "clip": "wave"},
       "mode": "once", "speed": 1, "blend_in": 0.1}],
    "transitions": [
      {"from": "idle", "to": "wave", "when": [{"input": "go", "op": "eq", "value": 1}],
       "duration": 0.2, "ease": {"type": "ease_in_out"}},
      {"from": "wave", "to": "idle", "exit_time": 1}]}]}]
```

Le `kind` d'une entrée est `bool`, `number` ou `trigger`. Le `source.kind` d'un état est `clip`, `segment`, `region`
ou `blend` ; `mode` est `loop`, `once` ou `ping_pong`. `Transition` et `Condition` rejettent les clés
inconnues.

```json
"bindings": [
  {"name": "hp", "path": "player.health", "expr": "value/100",
   "remap": {"in_min": 0, "in_max": 1, "out_min": 0, "out_max": 1, "clamp": true,
             "curve": {"type": "linear"}},
   "smooth": 0.1,
   "target": {"kind": "prop", "target": "sw", "prop": "layer_opacity"},
   "mode": "replace", "weight": 1},
  {"name": "go", "path": "input.go",
   "target": {"kind": "input", "machine": "m", "input": "go"}}]
```

Le `weight` d'une liaison doit être compris entre 0 et 1 (`binding 'b': weight must be within 0..1`). Les cibles des machines et des liaisons
sont vérifiées par une compilation d'essai.

### 11.5 `containers[]` et `scrolls[]`

Mise en page 1D de type flexbox. Champs d'un conteneur : `id`, `members`, `axis` (`row`, `column`), `gap`,
`cross_gap`, `padding` (4 nombres), `justify`, `align`, `wrap`, `rect`. Un `rect` a un `kind` :
`fixed` (`x,y,w,h`), `pin`, `aspect` ou `hug`. Un membre comporte exactement un élément parmi `layer`, `container` ou
`spacer`, plus `grow`, `shrink`, `width`, `height`, `align`, `resize`. Limites : 100000 membres,
profondeur d'imbrication 64. Un scroll comporte `id`, `viewport`, `content`, `axis`, `offset`, `elastic`, `snap_x`,
`snap_y`, `fling`, `clip_layer`, `input`.

```json
"containers": [{"id": "row", "axis": "row", "gap": 4, "padding": [2,2,2,2],
  "justify": "start", "align": "start", "wrap": "none",
  "rect": {"kind": "fixed", "x": 0, "y": 0, "w": 200, "h": 60},
  "members": [{"layer": "c1"}, {"spacer": true, "grow": 1},
              {"layer": "c2", "grow": 0, "shrink": 1, "width": 30, "height": 30,
               "align": "center", "resize": true}]}],
"scrolls": [{"id": "sc", "viewport": {"kind": "fixed", "x": 0, "y": 100, "w": 100, "h": 80},
  "content": "list", "axis": "y", "offset": [0, 10],
  "elastic": {"limit": 40, "stiffness": 180, "damping_ratio": 0.9},
  "snap_x": [], "snap_y": [0, 40], "fling": 0.15, "clip_layer": "mask_rect", "input": "scroll.y"}]
```

Les décalages des conteneurs sont ajoutés à la position du calque et multipliés dans l'échelle (étape 7 de l'évaluation).
`apply_clip_masks` ajoute des références de masque au projet qui lui est fourni ; elle modifie donc la copie du moteur.

### 11.6 `lods[]`

Enregistrement d'une passe de niveau de détail écrite par `osatura lod` : `level`, `tolerance_px`, `scale`,
`declared_px`, `bound_stage`, `path_px`, `mesh_px`, `keys_px`, `cull_px`, `stats`
(`path_anchors`, `mesh_vertices`, `keys`, `layers`, `bytes`) et `notes`. C'est une documentation de ce qu'a fait
une passe ; le moteur de rendu ne le lit jamais.


## 12. Règles de validation, limites et messages d'erreur

`Engine::new` valide en une seule passe et renvoie tous les problèmes. Le tableau énumère ce qui est appliqué ;
les messages entre guillemets ont été produits par `osatura check` sur des fichiers volontairement cassés.

### 12.1 Limites globales

| Règle | Limite | Message |
| --- | --- | --- |
| fps | `(0, 1000]` | `fps must be within (0, 1000]` |
| frames | `1..=200000` | `frames must be within 1..=200000` |
| width, height | `> 0` | `width and height must be > 0` |
| width, height | `<= 32768` | `width and height must be at most 32768` |
| tout flottant | `abs <= 1e7` | `value 100000000.0 at /bones/0/length is outside the supported range of +-1e7 (coordinates, scales, offsets and key values must stay below it)` |
| bones | au plus 4096 | `at most 4096 bones` |
| format | `<= 1` | `project format 2 is newer than this build supports (1); update OSATURA` |
| taille du JSON | CLI `--max-input-bytes`, 256 MiB par défaut ; API `MAX_JSON_BYTES` 256 MiB | |

La limite de 1e7 est appliquée deux fois. Au chargement, un littéral flottant qui la dépasse fait échouer l'analyse et le message
est enveloppé sous la forme `invalid JSON in <file>: value ...`. Un littéral sans point décimal (un entier)
passe l'analyse et est intercepté par la revérification dans `Engine::new`, où il apparaît comme un élément de liste
ordinaire. La limite existe pour qu'une multiplication ultérieure ne puisse pas faire déborder un `f32` jusqu'à l'infini.

### 12.2 Rig

| Règle | Message |
| --- | --- |
| le parent d'un os doit exister plus tôt | `bone 'lower' has parent 'nope' which is missing or listed after it` |
| longueur d'os finie, `>= 0` | `bone 'lower' has invalid length` |
| `angle_limits` `min <= max` ; `scale_limits` `0 < min <= max` | |
| une ik nécessite 2 os | `ik 'reach': needs at least two bones` |
| chaîne ik liée par le parent | `ik 'reach': each bone must be a direct child of the previous one` |
| la cible de l'ik n'est pas dans la chaîne | |
| soft, stretch | `ik 'reach': soft must be within 0..0.9 and stretch within 1..4` |
| une contrainte ne peut pas se cibler elle-même | |
| spring | `spring on 'upper': frequency must be in (0, 30] Hz and damping >= 0` ; un par os |

### 12.3 Calques, chemins, géométrie

| Règle | Message |
| --- | --- |
| nom de calque non vide, sans `/`, sans `::` | `layer name 'a/b' must be non-empty and contain neither '/' nor '::'` |
| chemin en double | `duplicate layer path 'arm'` (et des lignes pour les cibles animables dupliquées) |
| les noms de chemins et de déformateurs ne contiennent pas `::` et sont uniques au sein du calque | |
| le nombre d'ancres d'un morph est égal à celui du chemin de base | |
| les poids de skin nomment des os connus ; poids fini, `>= 0` | |
| mesh libre | au moins 3 sommets, au moins 1 triangle, au plus 200000 / 400000, fini, indices valides, `uvs` vide ou de même nombre |
| mesh en grille | `cols`, `rows >= 1` |
| masque | pas de masque sur soi-même ; `mask cycle involving layer 'arm'` |
| source d'image | `layer 'im': image has neither 'png_base64' nor 'file'` |
| liquid | au moins 1 blob ; `threshold > 0` ; `cell >= 0.25` ; `radius > 0` du blob |
| boolean | au moins 2 chemins |
| emitter | `rate` dans `(0, 2000]` ; `lifetime > 0` ; jitters 0..1 |
| déformateurs | uniquement sur les calques vectoriels et d'image ; voir la section 7.1 pour chaque type |
| effets | au plus 16 ; flou et douceur 0..64 ; décalages au plus 1024 ; quantités 0..1 |
| normal map | `strength` 0..8 ; nécessite `data` ou `path` |
| caméra | `camera focus must be finite and camera blur a finite number >= 0` |

### 12.4 Canaux

| Règle | Message |
| --- | --- |
| la cible se résout | `channel 0: unknown target 'zzz' for BoneOffsetX` |
| `layer_switch` sur un calque switch | `channel 2: layer_switch needs a switch layer, 'arm' is not one` |
| poids fini | |
| clés finies et strictement croissantes | `channel 0 ('target'): keys must be finite and strictly increasing in frame` |
| l'expression s'analyse | `channel 2 ('upper'): expression error: unexpected end of expression` |
| les cibles et props de `get()` se résolvent | `channel 2: get("q", "bone_angle"): unknown target` |
| l'entrée pilotée se résout | |

Des noms de `prop` ou d'`ease` inconnus produisent des erreurs serde qui listent toutes les variantes valides, par exemple
``unknown variant `foo`, expected one of ...``. Des types entiers erronés produisent
``invalid type: floating point `48.0`, expected u32``. Un champ obligatoire manquant produit
``missing field `frames` ``.

### 12.5 Autres sections validées

- Sons : `gain` 0..4, `trim` au plus 86400 s, les fondus tiennent dans le clip rogné, noms uniques.
  L'existence du fichier est vérifiée par `check`, pas par `Engine::new`.
- Machines : au moins un calque (`machine 'm': needs at least one layer`) ; les transitions et
  les conditions rejettent les clés inconnues. Liaisons : `binding 'b': weight must be within 0..1`. Leurs cibles
  sont vérifiées en les compilant une fois.
- Conteneurs et scrolls : membres uniques, au plus 100000, profondeur d'imbrication 64, règles de rect.
- Mesh locks et glues : références résolues par `links::resolve`.
- Fichiers d'image et audio : section 18.

### 12.6 Données d'éditeur : avertissements, et erreurs uniquement pour les nombres non finis

`Engine::new` et `check` examinent `skins`, `comps`, `markers`, `events`, `shots`, `channel_meta`, `bone_vis`, `clips` et
`lods` (module `editordata`). Pour que tous les projets enregistrés auparavant continuent de se charger, **seules les valeurs qui ne peuvent pas
exister dans un fichier enregistré sont des erreurs** : NaN ou l'infini dans les markers, events, shots, `bone_vis`, l'opacité de comp, les lods et l'opacité de calque. Tout le reste est
un **avertissement** affiché par `check` (et signalé par `lint` sous L013) : zoom de caméra `<= 0` (le moteur le borne à 0,01), `opacity` de calque
hors de 0..1 (bornée), `ik_mix` et `constraint_weight` hors de 0..1 (bornés), valeurs clés de `layer_opacity` et
`camera_zoom` hors de leur plage, références inconnues à des calques/os/canaux, noms en double, instants hors de
l'animation, skins invalides (`skins::validate`), clips (`clipstack::validate`, et cibles de pistes de take qui ne se résolvent pas,
en tenant compte de `rename`), `channel_meta` et `lods`. Les fixtures de `out/` se chargent sans avertissement.

`osatura check` affiche les avertissements de lint de ce type ; le linter complet (`osatura lint`) comporte davantage de règles (à partir de L001).

## 13. Exemple 1 : balle rebondissante

Vérifié avec `osatura check` (`ok: 'bouncing ball' 320×240 @ 24 fps, 48 frames, 0 bones, 0 ik,
3 layers, 5 channels`) et `osatura render ball.json 7 out.png`. Ce qu'il montre :

- Un `layer_y` à clés avec une interpolation `bounce` sur le premier segment : la balle tombe de y=40 jusqu'au
  sol à y=170 et rebondit trois fois. Le premier impact a lieu vers l'image 7,2 (24 / 3,352).
  Le segment de 24 à 47 remonte avec `ease_out`.
- Une ombre qui suit la balle : `layer_x` et `layer_opacity` sont des expressions qui lisent
  la balle avec `get()`. Elles sont listées après les canaux propres à la balle, car les canaux s'exécutent dans l'ordre.
- Écrasement au contact : une expression renvoie 0,8 pour `layer_scale_y` lorsque la balle est sous y=160.
- Cercle à partir de quatre ancres de Bézier avec une longueur de poignée de 16,57 (0,5523 fois le rayon 30).

```json
{
  "format": 1,
  "name": "bouncing ball",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "floor",
      "type": "vector",
      "paths": [
        {
          "name": "line",
          "closed": false,
          "anchors": [{"pos": [0, 200]}, {"pos": [320, 200]}],
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "shadow",
      "type": "vector",
      "paths": [
        {
          "name": "blob",
          "closed": true,
          "anchors": [
            {"pos": [-26, 0], "handle_out": [-26, -5], "handle_in": [-26, 5]},
            {"pos": [0, -6], "handle_in": [-14, -6], "handle_out": [14, -6]},
            {"pos": [26, 0], "handle_in": [26, -5], "handle_out": [26, 5]},
            {"pos": [0, 6], "handle_in": [14, 6], "handle_out": [-14, 6]}
          ],
          "fill": "#00000055"
        }
      ],
      "transform": {"position": [160, 200]}
    },
    {
      "name": "ball",
      "type": "vector",
      "paths": [
        {
          "name": "disc",
          "closed": true,
          "anchors": [
            {"pos": [30, 0], "handle_in": [30, -16.57], "handle_out": [30, 16.57]},
            {"pos": [0, 30], "handle_in": [16.57, 30], "handle_out": [-16.57, 30]},
            {
              "pos": [-30, 0],
              "handle_in": [-30, 16.57],
              "handle_out": [-30, -16.57]
            },
            {
              "pos": [0, -30],
              "handle_in": [-16.57, -30],
              "handle_out": [16.57, -30]
            }
          ],
          "fill": {
            "center": [-8, -10],
            "radius": 44,
            "stops": [
              {"offset": 0, "color": "#ffd9a0"},
              {"offset": 1, "color": "#d6336c"}
            ]
          },
          "stroke": {"paint": "#3d2c2e", "width": 2}
        }
      ],
      "transform": {"position": [160, 40]}
    }
  ],
  "channels": [
    {
      "target": "ball",
      "prop": "layer_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 40, "ease": {"type": "bounce"}},
          {"frame": 24, "value": 170, "ease": {"type": "ease_out"}},
          {"frame": 47, "value": 40}
        ]
      }
    },
    {
      "target": "ball",
      "prop": "layer_x",
      "source": "keys",
      "track": {"keys": [{"frame": 0, "value": 60}, {"frame": 47, "value": 260}]}
    },
    {
      "target": "shadow",
      "prop": "layer_x",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_x\")"
    },
    {
      "target": "shadow",
      "prop": "layer_opacity",
      "source": "expr",
      "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"
    },
    {
      "target": "ball",
      "prop": "layer_scale_y",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_y\") > 160 ? 0.8 : 1",
      "mode": "replace"
    }
  ]
}
```

## 14. Exemple 2 : bras riggé avec IK

Vérifié avec `osatura check` (3 bones, 1 ik, 3 layers, 2 channels). `osatura explain arm.json
24` indique la pointe en (170, 190), égale à l'os `target`, et `osatura render arm.json 24
out.png --bones` dessine le squelette. Ce qu'il montre :

- `upper` est une racine avec une limite d'angle ; `lower` est son enfant (`rest_offset` `[90, 0]` égal à la
  longueur du parent) ; `target` est un os racine libre que la chaîne IK cherche à atteindre.
- `reach` est une chaîne IK à deux os ; le solveur analytique est donc utilisé.
- `sleeve` est skinné : chaque ancre liste des poids d'os. `hand` et `goal` sont attachés avec
  `bone`.
- Déplacer l'os `target` avec des canaux `bone_offset_x` et `bone_offset_y` fait atteindre sa cible au bras.

```json
{
  "format": 1,
  "name": "rigged arm",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "bones": [
    {
      "name": "upper",
      "parent": null,
      "length": 90,
      "rest_offset": [60, 120],
      "rest_angle": 0,
      "angle_limits": [-120, 120]
    },
    {
      "name": "lower",
      "parent": "upper",
      "length": 80,
      "rest_offset": [90, 0],
      "rest_angle": 0
    },
    {
      "name": "target",
      "parent": null,
      "length": 10,
      "rest_offset": [230, 120],
      "rest_angle": 0
    }
  ],
  "ik": [
    {
      "name": "reach",
      "bones": ["upper", "lower"],
      "target": "target",
      "bend": 1,
      "mix": 1
    }
  ],
  "layers": [
    {
      "name": "arm",
      "type": "vector",
      "paths": [
        {
          "name": "sleeve",
          "closed": true,
          "anchors": [
            {"pos": [60, 106], "weights": [{"bone": "upper", "weight": 1}]},
            {
              "pos": [150, 104],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [228, 110], "weights": [{"bone": "lower", "weight": 1}]},
            {"pos": [228, 130], "weights": [{"bone": "lower", "weight": 1}]},
            {
              "pos": [150, 136],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [60, 134], "weights": [{"bone": "upper", "weight": 1}]}
          ],
          "fill": "#ffb347",
          "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
        }
      ]
    },
    {
      "name": "hand",
      "type": "vector",
      "bone": "lower",
      "paths": [
        {
          "name": "palm",
          "closed": true,
          "anchors": [
            {"pos": [246, 120], "handle_in": [246, 111], "handle_out": [246, 129]},
            {"pos": [232, 134], "handle_in": [241, 134], "handle_out": [223, 134]},
            {"pos": [218, 120], "handle_in": [218, 129], "handle_out": [218, 111]},
            {"pos": [232, 106], "handle_in": [223, 106], "handle_out": [241, 106]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "goal",
      "type": "vector",
      "bone": "target",
      "paths": [
        {
          "name": "ring",
          "closed": false,
          "anchors": [{"pos": [222, 120]}, {"pos": [238, 120]}],
          "stroke": {"paint": "#d6336c", "width": 3, "dash": [4, 4]}
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "target",
      "prop": "bone_offset_x",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": -60, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    },
    {
      "target": "target",
      "prop": "bone_offset_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": 70, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    }
  ]
}
```

## 15. Exemple 3 : bouche à calque switch

Vérifié avec `osatura check` et en rendant les images 0, 5 et 9, qui montrent trois bouches
différentes. Les clés hors plage sont bornées : une valeur de clé de 7 affiche l'enfant 2 et -3 affiche l'enfant 0.

```json
{
  "format": 1,
  "name": "switch mouth",
  "width": 200,
  "height": 160,
  "fps": 12,
  "frames": 25,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "face",
      "type": "vector",
      "paths": [
        {
          "name": "head",
          "closed": true,
          "anchors": [
            {"pos": [180, 80], "handle_in": [180, 35], "handle_out": [180, 125]},
            {"pos": [100, 150], "handle_in": [145, 150], "handle_out": [55, 150]},
            {"pos": [20, 80], "handle_in": [20, 125], "handle_out": [20, 35]},
            {"pos": [100, 10], "handle_in": [55, 10], "handle_out": [145, 10]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "mouth",
      "type": "switch",
      "selected": 0,
      "layers": [
        {
          "name": "closed",
          "type": "vector",
          "paths": [
            {
              "name": "line",
              "closed": false,
              "anchors": [{"pos": [75, 110]}, {"pos": [125, 110]}],
              "stroke": {"paint": "#3d2c2e", "width": 4, "cap": "round"}
            }
          ]
        },
        {
          "name": "open",
          "type": "vector",
          "paths": [
            {
              "name": "oval",
              "closed": true,
              "anchors": [
                {"pos": [75, 106]},
                {"pos": [125, 106]},
                {
                  "pos": [112, 132],
                  "handle_in": [122, 132],
                  "handle_out": [102, 134]
                },
                {"pos": [88, 132], "handle_in": [98, 134], "handle_out": [78, 132]}
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
            }
          ]
        },
        {
          "name": "round",
          "type": "vector",
          "paths": [
            {
              "name": "o",
              "closed": true,
              "anchors": [
                {
                  "pos": [112, 118],
                  "handle_in": [112, 110],
                  "handle_out": [112, 126]
                },
                {
                  "pos": [100, 130],
                  "handle_in": [106, 130],
                  "handle_out": [94, 130]
                },
                {"pos": [88, 118], "handle_in": [88, 126], "handle_out": [88, 110]},
                {
                  "pos": [100, 106],
                  "handle_in": [94, 106],
                  "handle_out": [106, 106]
                }
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3}
            }
          ]
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "mouth",
      "prop": "layer_switch",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "step"}},
          {"frame": 4, "value": 1, "ease": {"type": "step"}},
          {"frame": 8, "value": 2, "ease": {"type": "step"}},
          {"frame": 12, "value": 1, "ease": {"type": "step"}},
          {"frame": 16, "value": 0, "ease": {"type": "step"}}
        ]
      }
    }
  ]
}
```

## 16. Ordre d'évaluation

`Engine::evaluate(frame)` est une fonction pure du numéro d'image et du projet. Les springs sont
simulés une seule fois à la construction du moteur ; rien d'autre ne conserve d'état entre les images. Les étapes :

1. **État initial** (`initial_state`) : transformations statiques des calques, opacité bornée à 0..1, `selected` des switch,
   amplitude de wobble, caméra, mix `ik`, poids des contraintes, et décalages initiaux des
   emplacements de paramètres (décalages d'os, décalages de déformateurs, poids de morph, etc.).
2. **Canaux** (`apply_channels`) dans l'ordre de la liste, chacun se combinant avec la valeur courante selon son
   `mode` et son `weight`. Les résultats non finis sont ignorés. Les écritures sont bornées comme indiqué à la section 8.2.
3. **Bornage des limites** (`clamp_limits`) : `angle_limits` et `scale_limits` par os.
4. **Cinématique directe** : matrices monde de tous les os, parents d'abord.
5. **Chaînes IK** dans l'ordre de la liste, en ignorant les chaînes avec `mix <= 0`. Après chaque chaîne, la pose est bornée
   et la FK recalculée, de sorte que la chaîne suivante voit le résultat.
6. **Contraintes** dans l'ordre de la liste, en ignorant `weight <= 0` : aim, copy_rotation, pin.
7. **Conteneurs de mise en page** : les décalages sont ajoutés à x et y du calque et multipliés dans l'échelle.
8. **Springs bakés** : les angles bakés remplacent l'angle de l'os, puis la FK est recalculée.
9. **Matrices de skin** : `skin_mat = world * inverse(rest world)` par os.
10. **Caméra** : une matrice de caméra par calque racine, utilisant le `parallax` du calque racine (section 10.1).
11. **Nœuds** en pré-ordre. La matrice de scène d'un nœud est `parentStage * boneAttach(skin_mat) * local`.
    La géométrie vectorielle est construite dans cet ordre :
    1. morphs (blend shapes additifs) ;
    2. décalages `point_dx/dy` par ancre ;
    3. décalages des poignées de dégradé (`grad_dx/dy`) ;
    4. déformateurs dans l'ordre de la liste, puis `wobble` ;
    5. skinning avec des poids normalisés (des poids vides signifient statique ; les poignées de dégradé sont
       aussi skinnées).
    La géométrie skinnée vit dans les coordonnées de repos de la scène, et la transformation du calque est appliquée après.
    Un switch affiche un enfant (arrondi, borné). La géométrie des liquid, boolean et emitter est
    synthétisée à chaque image. Une image obtient un mesh en grille ou libre ; avec des déformateurs mais sans mesh, elle obtient une
    grille de 16 par 16.
12. **Mesh locks et glues** dans l'espace de la scène, dans l'ordre topologique.
13. **Réparation des valeurs non finies** (`repair_nonfinite`) : un nœud dont la géométrie contient NaN ou l'infini
    est masqué et `Frame.nonfinite` est positionné. `evaluate_checked` et `check_all_frames_finite` transforment cela
    en erreur.

Conséquences sur lesquelles vous pouvez compter :

- Un canal ne voit que les canaux qui le précèdent dans la liste (via `get` et `driven`).
- L'IK prend le pas sur la FK pour les os de la chaîne, en proportion de `mix` (le mélange est
  `old + wrap_pi(new - old) * mix`). Avec `mix` à 0, la chaîne est de la FK pure.
- Les contraintes s'exécutent après l'IK ; une contrainte aim sur un os d'IK l'emporte donc.
- Les springs agissent en dernier sur l'angle de l'os, après l'IK et les contraintes.

## 17. Ordre de rendu

- Les calques racines sont dessinés dans l'ordre de la liste, les enfants dans l'ordre de la liste à l'intérieur de leur parent.
- Un nœud qui n'est pas `visible` est ignoré (il peut néanmoins être une source de masque).
- Un nœud est rendu dans un calque hors écran lorsqu'il a `opacity < 1`, des effets, un mode de fusion autre
  que `normal`, ou un masque. Étapes : contenu ; effets de calque dans l'ordre de la liste (sur un canevas agrandi) ; masque
  (le calque de masque est dessiné avec sa propre transformation ; son alpha, y compris son opacité, se multiplie ;
  `invert` l'inverse) ; composition avec l'opacité et le mode de fusion.
- La profondeur de champ n'est appliquée que par `dof` et `dof-export` : rayon = `abs(parallax - focus) *
  blur`, d'au plus 24 pixels par défaut.
- La surcouche des os (`--bones`) est dessinée en dernier.

## 18. Assets et chemins

Les projets peuvent référencer des fichiers : `image.source.file`, `normal_map.path`, `sounds[].file`.

- Les chemins doivent être relatifs, ne doivent pas contenir `..`, et doivent, après résolution des liens symboliques, rester à l'intérieur du
  répertoire du projet : `layer 'im': asset '/etc/passwd': absolute paths are not allowed (a project
  may only reference files inside its own directory; use --allow-external-assets to override)`, et
  `layer 'im': asset '../x.png': '..' is not allowed (...)`.
- `--allow-external-assets` lève cette restriction pour une seule commande. Seuls les fichiers réguliers
  sont ouverts.
- Plafonds de taille : images 64 Mo sur disque, audio 256 Mo. Images décodées : chaque côté d'au plus 16384 et d'au
  plus 64 mégapixels.
- Plafonds de sortie : raster d'au plus 16384 par côté et 100 mégapixels ; séquences d'au plus 2 GiB ; budget
  d'export images fois ancres d'au plus 40 millions.
- Lorsque la CLI écrit un projet, elle le revalide via `Engine::new` et réécrit les chemins d'assets
  relativement à l'emplacement de sortie.

Préférez `png_base64` pour un fichier autonome.

## 19. Politique de versionnage

- `format` est un entier, actuellement `1` (`FORMAT_VERSION`).
- Un fichier dont le `format` est supérieur à celui pris en charge par la build est refusé avec le message de la section 12.1 ;
  le moteur ne devine pas.
- Ajouter un champ facultatif avec une valeur par défaut n'est pas un changement de version. Les anciennes builds l'ignorent (et
  `check` les en avertit comme d'un champ inconnu).
- Supprimer ou réinterpréter un champ exige un changement de version, et ce changement exige une étape de migration dans
  `Project::from_json` qui met à niveau les anciens documents avant la validation.
- L'API de la bibliothèque Rust a sa propre version, `API_VERSION` 1.4.0, protégée par le test d'instantané
  `api_surface.txt` ; l'ABI C a `ABI_VERSION` 1. Voir `docs/API.md`.

## 20. Écarts connus entre le code et les commentaires

Ils ont été découverts lors de la rédaction de cette référence.

1. **Corrigé.** La règle des clés de switch (`lint::switch_key_warnings`, L012) décrit désormais ce que fait le moteur : une clé inférieure à 0 affiche l'enfant 0,
   une clé supérieure ou égale au nombre d'enfants affiche le dernier enfant (un switch vide n'affiche rien). Test : tests unitaires de `lint`, vérifiés
   contre le moteur.
2. Le commentaire de `project.rs` indique que les fichiers sans `format` sont de version 1, mais la valeur par défaut serde est
   `FORMAT_VERSION`. Les deux concordent aujourd'hui uniquement parce que la constante vaut 1. Lorsque la constante sera augmentée, un
   fichier sans `format` serait lu comme la version la plus récente.
3. **Corrigé.** `unknownfields` ne signale plus une structure vide écrite explicitement (`clips[].weight_keys: {"keys": []}`) ;
   `default_like` est récursive, et un test fige l'ensemble des structures facultatives que l'écrivain omet lorsqu'elles sont vides.
4. **Corrigé (sous forme d'avertissements).** Les champs d'éditeur sont validés ; voir la section 12.6.
5. **Corrigé dans la vague 7 (L-71).** La géométrie d'un calque skinné est dans l'espace de repos de la scène et la transformation du parent est appliquée après
   elle. `Layer.bone` s'attache désormais avec `world = bone_skin * parent_stage * local` : la matrice de skin (monde de l'os x inverse du repos) est
   un mouvement dans l'espace de la scène, elle est donc appliquée après toute la chaîne des parents. Au repos, la matrice de skin est l'identité, donc rien
   ne bouge ; lorsque l'os est **posé**, un groupe parent d'échelle 2 ne double plus le déplacement d'un calque attaché. Un calque
   sans os propre suit l'os de son ancêtre attaché à un os le plus proche ; l'os propre d'un calque remplace celui de l'ancêtre
   (les deux mouvements ne sont pas cumulés). Tests : `bone_attach_under_a_transformed_group_composes_bone_after_the_group`,
   `bone_attach_is_inherited_and_a_child_bone_replaces_the_parents`. Les projets qui reposaient sur l'ancien déplacement doublé (un os posé
   sous un groupe mis à l'échelle ou pivoté) s'affichent différemment ; les projets sans cette combinaison sont inchangés.
