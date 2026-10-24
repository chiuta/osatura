<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Référence de la ligne de commande OSATURA

Référence du binaire `osatura` (crate `osatura-cli`, AGPL-3.0-or-later ; il lie le moteur `osatura-core` sous MPL-2.0).
Version documentée : **osatura 0.1.0**. Ce texte est placé sous licence CC-BY-4.0.

Chaque synopsis ci-dessous est copié du texte d'utilisation du programme lui-même (`osatura` sans argument, ou
`osatura <command> --help`). Chaque exemple a été exécuté sur le projet de démonstration intégré et la sortie affichée
est tronquée à partir de la sortie réelle. Lorsque le texte d'utilisation et le comportement réel divergent, ce document décrit le
comportement réel et signale la divergence dans [Écarts connus entre documentation et comportement](#divergences-connues-entre-la-documentation-et-le-comportement) à la fin.

Documents associés : [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (ce que chaque format d'export/import conserve et perd),
[CI.md](CI.md) (manifestes de CI d'animation), [API.md](API.md) (bibliothèque / ABI C), [PATENT_NOTES.md](PATENT_NOTES.md)
(codecs et statut des brevets, ne constitue pas un avis juridique).

## Sommaire

1. [Conventions communes à toutes les commandes](#1-conventions-communes-à-toutes-les-commandes)
2. [Prise en main : le projet de démonstration](#2-prise-en-main--le-projet-de-démonstration)
3. [Création et inspection de projets](#3-création-et-inspection-de-projets)
4. [Rendu et export](#4-rendu-et-export)
5. [Rigging, skins, IK et aides de mise en page](#5-rigging-skins-ik-et-aides-de-mise-en-page)
6. [Animation, clés, timeline et générateurs](#6-animation-clés-timeline-et-générateurs)
7. [Physique, tissu et déformateurs](#7-physique-tissu-et-déformateurs)
8. [Audio, synchronisation labiale et sons](#8-audio-synchronisation-labiale-et-sons)
9. [Import (SVG, OpenRaster, Lottie, BVH, GIF/APNG, séquences PNG, bitmaps)](#9-import)
10. [Entrée en direct, MIDI, suivi du visage et capture de mouvement](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement)
11. [Machines à états, liaisons de données, conteneurs et zones de défilement](#11-machines-à-états-liaisons-de-données-conteneurs-et-zones-de-défilement)
12. [Effets, profondeur de champ, onion skin, trajectoires et caméra](#12-effets-profondeur-de-champ-onion-skin-trajectoires-et-caméra)
13. [Niveau de détail, rapports de taille et export-fit](#13-niveau-de-détail-rapports-de-taille-et-export-fit)
14. [Assurance qualité : check, lint, ci, golden, fuzz, diff](#14-assurance-qualité)
15. [Gestion de versions (`vcs`)](#15-gestion-de-versions-vcs)
16. [Studio, collaboration, serveur d'agent MCP et registre de l'interface](#16-studio-collaboration-serveur-dagent-mcp-et-registre-de-linterface)
17. [Base de poses, suggestions et autres utilitaires](#17-base-de-poses-suggestions-et-autres-utilitaires)
18. [Écarts connus entre documentation et comportement](#divergences-connues-entre-la-documentation-et-le-comportement)
19. [Index alphabétique des commandes](#index-alphabétique-des-commandes)

---

## 1. Conventions communes à toutes les commandes

### 1.1 Forme de la ligne de commande

```
osatura <command> [sub-command] <positional arguments...> [options]
```

* Les options peuvent apparaître **n'importe où** sur la ligne, y compris après la sous-commande et après les arguments positionnels
  (`osatura render demo.json 0 o.png --scale 2` et `osatura --scale 2 render ...` sont équivalents).
* Les **options à valeur** prennent le mot suivant (`--scale 2`) ou la forme `--key=value` (`--scale=2`). La forme `=` fonctionne pour
  toutes les options à valeur. Une valeur qui contient elle-même `=` convient après la forme avec espace (`--remap a=b`).
* Les **drapeaux** (options booléennes comme `--bones`) ne prennent aucune valeur. `--bones=1` est refusé :
  `--bones is a flag and takes no value: write --bones (without =), got '--bones=1'`.
* **Options strictes.** Une option (drapeau ou option à valeur) que la commande ne documente pas est une erreur accompagnée d'une
  suggestion « did you mean » (code de sortie 1) au lieu d'être ignorée :
  ```
  $ osatura render demo.json 0 o.svg --bone
  unknown option --bone for 'render' (did you mean --bones?); see: osatura render --help
  ```
  Cette vérification s'applique à toute commande dotée d'une ligne d'utilisation, y compris `check`, `stats`, `lint`, `coach`, `contracts` et
  `machine list` (les anciennes versions ignoraient silencieusement les drapeaux superflus dans ces commandes). `--json`, `--quiet` et `--allow-empty` ne sont **pas**
  globales : elles sont refusées par toute commande dont la ligne d'utilisation ne les mentionne pas.
  **La vérification s'effectue par sous-commande (par forme de synopsis), et non par commande.** Une option citée pour une forme est refusée
  sur les formes voisines, et le message nomme la forme qui l'accepte :
  ```
  $ osatura ci diff a.png b.png --quiet
  unknown option --quiet for 'ci diff' (--quiet applies to: ci run); see: osatura ci --help
  ```
  Ainsi `--quiet` ne vaut que pour `ci run`, `--json` que pour `vcs log|status|diff|show`, `ui commands|shortcuts` et la
  forme `trail ... --target`, `--tol` que pour `golden check` (et `ci diff`), `--scale` que pour `golden write`, et
  chaque modification de `keyedit` n'accepte que les options écrites après son nom dans le synopsis. (`export` regroupe toujours ses deux
  lignes de synopsis, parce que le format est choisi par l'extension du fichier ; `--codec`, `--columns` et `--scale` qui ne
  s'appliquent pas à l'extension choisie sont signalés par `export` lui-même.)
* Certaines commandes plus anciennes prennent des **mots** `key=value` au lieu d'`--options` (`cloth`, `physics`, `cage`, `warp-pivot`,
  `soft-scale`, `mesh-lock`, `glue`, `audio-react`, `beats`, `import-seq`, `import-anim`, `layer-pivot`, `chan-meta`,
  `waveform`). Ils sont positionnels et s'écrivent exactement comme dans le synopsis (`gravity=900`, et non `--gravity 900`).
* Une **image** (frame) est un nombre (les fractions sont permises là où cela a un sens). Un projet avec `frames = N` a les images `0..N-1`.
  Les calques sont désignés par leur **chemin** (`group/child`), les os par leur **nom** ; la caméra a la cible vide `""`.
* Les **noms de propriétés** (utilisés par `--prop`, `slider`, `drive`, les fichiers de mapping, ...) sont les noms en snake_case :
  `bone_angle bone_scale bone_offset_x bone_offset_y layer_opacity layer_x layer_y layer_rotation layer_scale_x
  layer_scale_y layer_skew layer_switch layer_wobble camera_x camera_y camera_zoom camera_rotation camera_focus
  camera_blur ik_mix constraint_weight path_morph point_dx point_dy lattice_dx lattice_dy spine_dx spine_dy pin_dx
  pin_dy cage_dx cage_dy fx_blur fx_dx fx_dy fx_soft fx_amount grad_dx grad_dy`.  Certaines commandes affichent les mêmes propriétés en CamelCase (`BoneAngle`) dans leurs messages ; les valeurs d'options sont toujours en snake_case.

### 1.2 Options globales

| Option | Signification |
|---|---|
| `--allow-external-assets` | Permet à un projet de désigner des fichiers par chemin absolu ou en dehors de son propre répertoire (sons, images, normal maps...). Par défaut : refusé. |
| `--follow-symlinks` | Écrit dans la **cible** d'un lien symbolique donné comme fichier de sortie. Par défaut : refusé. |
| `--max-input-bytes N` | Taille maximale d'un fichier de projet lu. Par défaut `268435456` (256 Mio). Doit être un entier strictement positif. |
| `<command> --help` | Affiche les lignes d'utilisation d'une commande sur stdout (code de sortie 0). |
| `<command> --version`, `--version` | `osatura 0.1.0` ; sans commande, affiche aussi la ligne de licence et l'offre de sources AGPL. |
| `--json`, `--quiet`, `--allow-empty` | Options propres à certaines commandes, **refusées** (`unknown option`) par les commandes dont le synopsis ne les mentionne pas (`--json` : `pick`, `select-box`, `timeline`, `trail`, `size-report`, `collab-sim`, `vcs`, `ui`, `face-detect`, `pose-suggest` ; `--allow-empty` : `physics`, `live-map`, `face-bake` ; `--quiet` : `ci run`). Les scripts qui les passaient uniformément à toutes les commandes doivent cesser de le faire. |

Exemples (vérifiés) :

```
$ osatura --max-input-bytes 100 check demo.json
cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)
[exit 1]

$ osatura check /dev/zero
cannot read /dev/zero: not a regular file (devices, pipes and directories are refused)
[exit 1]

$ ln -s real_target.svg link.svg                       # dangling link
$ osatura render demo.json 0 link.svg
cannot write link.svg: link.svg: refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself
[exit 1]
```

### 1.3 Lecture des fichiers d'entrée

* Les fichiers d'entrée doivent être des **fichiers ordinaires** d'une taille maximale de `--max-input-bytes`. Les périphériques, tubes, répertoires et
  substitutions de processus (`<(...)`) sont refusés : écrivez d'abord les données dans un fichier.
* Les fichiers de projet sont du JSON. Un projet comportant des clés inconnues est **accepté** et les clés inconnues sont signalées par
  `warning: unknown field bogus_key (ignored)` via `check` (ainsi que `lint` et `studio`). Les fichiers annexes que vous rédigez pour une commande
  (contrats, mappings, machines, conteneurs, zones de défilement, manifestes ci, cartes drive ...) sont analysés **strictement** : une clé
  inconnue est une erreur et rien n'est appliqué (`unknown field(s): elastic.damping (typo? nothing was applied)`).
* Un projet est validé chaque fois qu'il est chargé pour un rendu ou un export (`<file> is not a valid project:` suivi d'un
  problème par ligne). Les commandes d'édition recompilent aussi le projet qu'elles s'apprêtent à écrire et refusent d'en écrire un qui ne serait plus valide.
* Les **chemins d'assets** à l'intérieur d'un projet (fichiers de sons, d'images, normal maps) sont résolus par rapport au **répertoire du fichier
  de projet**. Les chemins absolus et tout chemin contenant `..` sont refusés, sauf si `--allow-external-assets` est donné :
  ```
  $ osatura sound add s.json o.json x /etc/passwd
  sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)
  ```
  Le drapeau doit être redonné à chaque commande ultérieure qui charge ce projet (`check ext.json` échoue sans lui, réussit avec).

### 1.4 Écriture des fichiers de sortie

* **Écritures atomiques.** La plupart des commandes écrivent via un fichier temporaire situé dans le **même répertoire** (`.<name>.tmp-<pid>-<random>`), le `fsync`,
  recopient les permissions de l'ancien fichier et le `rename` par-dessus la cible, de sorte qu'un plantage ou un disque plein ne laisse jamais un fichier à moitié écrit.
  Si un fichier temporaire ne peut pas être créé (répertoire en lecture seule mais fichier inscriptible), le fichier est écrasé sur place, ce qui n'est **pas** atomique.
  Les FIFO et les fichiers spéciaux de type `/dev/stdout` sont écrits directement.
* Les **liens symboliques** comme sortie sont refusés, sauf si `--follow-symlinks` est donné (la cible du lien est alors écrite ; un lien pendant reste une erreur).
  Un répertoire donné comme fichier de sortie est une erreur (`is a directory`).
* **Jamais par-dessus l'entrée.** `render`, `render-hdr`, `dof`, `dof-export`, `export` (y compris le fichier annexe `.json` de sprite sheet d'un `.png`
  et `--report`), `layout-preview`, `render-gpu`, `export-fit`, `fuzz-grid`, `onion`, `trail` et `golden` refusent une sortie qui est le même fichier
  que le projet d'entrée : `refusing to overwrite the input project demo.json with output of the same name`.
* Les **commandes d'édition** (celles qui ont une paire `<project.json> <out.json>` : `reparent`, `keys`, `physics`, ...) peuvent écrire **sur place**
  (`osatura reparent p.json p.json hand none` fonctionne et remplace `p.json`). `import-lottie` refuse une sortie (ou `fx.json`) qui est
  le même fichier que son entrée ; les autres commandes d'import lisent un autre type de fichier que celui qu'elles écrivent.
* **Les sorties existantes sont écrasées silencieusement** par toutes les commandes. Il n'y a pas de `--force`.
* **Toute sortie adressée à l'utilisateur** (y compris les rapports et golden de `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, les fichiers `face-*`,
  les fichiers de poses, les snapshots, `vcs` et le journal de collaboration du studio) passe par le même écrivain atomique qui refuse les liens symboliques. Les
  sorties `.mp4`/`.webm` écrites par ffmpeg sont vérifiées pour détecter un lien symbolique avant le lancement de ffmpeg. Dossiers parents : `lod`, `export-fit`, `ci` et `face-*`
  créent les dossiers manquants ; `fuzz`, `fuzz-grid` et `export` ne le font pas (`No such file or directory`).

### 1.5 stdout, stderr et codes de sortie

* Les résultats (« wrote out.json », tableaux, JSON) vont sur **stdout**. Les erreurs et la plupart des avertissements vont sur **stderr**. `import-svg` affiche ses
  avertissements sur stdout ; `export` affiche les avertissements Lottie/GIF/GLB sur stderr.
* Le code de sortie **0** signifie le succès. Le code de sortie **1** signifie toute erreur *ou* une vérification échouée : un fichier illisible/invalide, de mauvais arguments,
  `lint` avec des erreurs, `diff` avec des différences, un contrat violé, un golden non conforme, un taux d'échec de fuzz supérieur à la limite,
  un budget d'`export-fit` impossible à respecter, `lod --verify` au-delà de la borne, une divergence de `collab-sim`, une vérification `ci` en échec.
  Le code de sortie **2** n'est utilisé que par `ci` (rien n'a pu être évalué : projet/manifeste manquant, golden altéré).
* Aucun argument, une commande inconnue, un nombre incorrect d'arguments positionnels, ou `osatura --help` affichent l'utilisation complète sur
  **stderr** et sortent avec le code **1**. `osatura <command> --help` affiche les lignes d'utilisation de cette commande sur stdout et sort avec le code 0.
  **Exception (Windows, depuis 0.1.0-alpha.2) :** lorsque `osatura.exe` est lancé par un double-clic depuis l'Explorateur (la fenêtre de console a été créée pour lui et aucun autre processus ne la partage) et n'a aucun argument, il n'affiche pas l'utilisation : il crée `Documents\OSATURA\first-rig.json` (à partir de `demo-person`) s'il manque, exécute `studio` sur ce fichier à `127.0.0.1:8080`, ouvre le navigateur et garde la console ouverte jusqu'à ce que vous la fermiez ou appuyiez sur Entrée après une erreur. Lancé depuis un terminal, il se comporte comme décrit plus haut. Le reste du programme s'exécute dans un thread à pile de 256 MiB (le thread principal de Windows n'a que 1 MiB).

### 1.6 Programmes externes

`ffmpeg` (trouvé dans le `PATH`) est requis pour : l'export `.mp4`/`.webm` et `dof-export` vers ces formats, `sound mix`,
l'audio dans `.html` (mp3), et `face-track` sur un fichier vidéo. Encodeurs nécessaires : `libx264` (mp4), `libvpx-vp9` / `libaom-av1` (webm),
`aac` (son mp4), `libopus` (son webm), `libvorbis`, `libmp3lame` (sound mix). Aucun code ni binaire ffmpeg n'est embarqué ; voir
[PATENT_NOTES.md](PATENT_NOTES.md). Rien d'autre n'est exécuté.

### 1.7 Remarque sur les performances

Toutes les durées de ce document proviennent d'un build **debug** et sont 10 à 30 fois plus lentes que celles d'un build release. Les commandes lourdes en images
(`safety`, `golden`, `ci`, `export-fit`, `proxy-bench`) ont été exécutées sur une copie réduite à 8 images de la démo (`small.json`, obtenue en réglant
`"frames": 8`).

---

## 2. Prise en main : le projet de démonstration

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura export demo.json demo.gif
wrote demo.gif
```

La démo comporte les os `arm_up arm_low hand thigh shin foot_ctrl`, une chaîne IK `leg`, et les calques de premier niveau
`sky sun hills ground shadow arm leg target blush mouth(switch: closed/open/smile) ink`. Tous les exemples de cette référence s'en servent,
sauf indication contraire.

---

## 3. Création et inspection de projets

### `demo`

**Synopsis :** `osatura demo <out.json>`

Écrit sous forme de fichier de projet la démo de rig intégrée (640x480, 24 fps, 48 images, 6 os, jambe IK, 14 calques).
**Sortie :** `wrote <out.json>`. **Erreurs :** argument manquant (usage, code de sortie 1) ; chemin non inscriptible. **Voir aussi :** `demo-fx`, `check`.
**Limites :** la démo est une ressource fixe ; elle sert de base à chaque exemple de ce document.

```
$ osatura demo demo.json
wrote demo.json
```

### `demo-person`

**Synopsis :** `osatura demo-person <out.json>`

Écrit sous forme de fichier de projet la démo de la personne qui marche (640x480, 24 fps, 32 images, 42 os, 45 calques) : une personne vue de profil avec tête, oreille et queue-de-cheval, cou, tronc, deux bras avec mains et cinq doigts chacun, et deux jambes avec pieds et orteils. Le cycle de marche dure deux pas et boucle sans raccord : le bassin oscille et s'incline, le tronc se penche et tourne en sens inverse, la tête reste à l'horizontale avec un petit hochement en retard, les bras balancent à l'opposé des jambes avec des coudes qui plient davantage lors du balancement vers l'avant, et les mains, les doigts, l'oreille et la queue-de-cheval suivent un peu plus tard. Chaque pied se pose sur le talon, roule à plat et repart sur la plante du pied ; le sol, la clôture et l'ombre défilent exactement à la vitesse du pied posé, si bien que les pieds ne glissent pas. C'est le projet que les lanceurs Windows (`osatura.exe`, `osatura-studio.exe`) créent sous le nom `first-rig.json` au premier démarrage ; `osatura demo` reste la petite base des exemples de ce document.
**Sortie :** `wrote <out.json>`. **Erreurs :** argument manquant (usage, code de sortie 1) ; chemin non inscriptible. **Voir aussi :** `demo`, `studio`.
**Limites :** les clés sont échantillonnées toutes les deux images, avec une clé supplémentaire avant l'image 0 et une après la dernière image, afin que la boucle soit fluide ; `lint` signale ces deux clés comme hors de la ligne de temps (information, pas une erreur). La figure est une vue de profil : il n'y a ni rotation ni profondeur.

```
$ osatura demo-person person.json && osatura check person.json
wrote person.json
ok: 'OSATURA walking person' 640×480 @ 24 fps, 32 frames, 42 bones, 0 ik, 45 layers, 48 channels
```

### `demo-fx`

**Synopsis :** `osatura demo-fx <out.json>`

Écrit la démo « liquide / booléen / particules » (640x480, 72 images, 3 os, 3 calques) qui met à l'épreuve les calques procéduraux.

```
$ osatura demo-fx fx.json && osatura check fx.json
wrote fx.json
ok: 'OSATURA fx demo' 640×480 @ 24 fps, 72 frames, 3 bones, 0 ik, 3 layers, 3 channels
```

### `check`

**Synopsis :** `osatura check <project.json>`

Charge et valide un projet. Affiche des lignes `warning:` (champs inconnus, problèmes de dégradés, problèmes de clés de switch, et avertissements sur les données d'éditeur du code lint L013 : zoom de caméra <= 0, opacité de calque hors de 0..1, références ou temps invalides dans les skins, comps, markers, events, shots, channel_meta, bone_vis, clips et lods) et une ligne de synthèse `ok:`. Seules les valeurs qui ne peuvent pas exister dans un fichier enregistré (NaN ou infini dans ces structures) sont des erreurs, de sorte que les anciens projets continuent de se charger.
Les pistes de son sont ouvertes aussi : un fichier audio illisible ou inutilisable est une **erreur**. **Sortie :** 0 valide ; 1 invalide.

```
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check unk.json                      # a project with two unknown keys
warning: unknown field bogus_key (ignored)
warning: unknown field bones[0].lenght (ignored)
ok: 'small' 640×480 @ 24 fps, 8 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check bad3.json                     # IK chain whose target bone does not exist
bad3.json is not a valid project:
  - ik 'leg': unknown target bone 'nope'
[exit 1]

$ osatura check bad.json                      # {"format":1,"name":"x"}
invalid JSON in bad.json: missing field `width`
[exit 1]
```

**Voir aussi :** `lint` (qualité du rig/de l'animation), `ci` (barrière de non-régression). **Piège :** `check` ne rend rien ; un projet peut réussir `check` et pourtant
échouer plus tard sur une image (les poses `NaN` sont interceptées par `render`/`export`, qui évaluent d'abord toutes les images).

### `stats`

**Synopsis :** `osatura stats <project.json>`

Affiche des compteurs en JSON : `name, size, fps, frames, seconds, bones, ik_chains, constraints, springs, layers, channels, keyed_channels, keys, markers`.
La sortie est toujours en JSON ; il n'y a pas d'option `--json` (elle est refusée comme toute option inconnue).

```
$ osatura stats demo.json
{ "name": "OSATURA demo", "size": [640.0, 480.0], "fps": 24.0, "frames": 48, "seconds": 2.0, "bones": 6,
  "ik_chains": 1, "constraints": 0, "springs": 0, "layers": 14, "channels": 14, "keyed_channels": 8, "keys": 37, "markers": 0 }
```
(la sortie réelle est mise en forme sur plusieurs lignes.) Notez que `layers` compte chaque nœud, y compris les enfants des groupes/switches, tandis que `check` affiche le même nombre.

### `diff`

**Synopsis :** `osatura diff <a.json> <b.json>`

Différence sémantique entre deux projets : `+` ajouté, `-` supprimé, `~` modifié. Insensible à l'ordre pour les listes adressées par nom.
**Sortie :** 0 identiques (`no differences`) ; **1** lorsqu'il y a des différences (le décompte `N difference(s)` va sur stderr), ce qui permet de conditionner un script.

```
$ osatura diff demo.json demo.json
no differences
$ osatura diff demo.json d2.json              # fps 24->30, arm_up longer, layer 'sun' deleted
~ fps: 24 → 30
~ bone 'arm_up' changed (length)
- layer 'sun' removed
3 difference(s)            (stderr)
[exit 1]
```
**Voir aussi :** `vcs diff` (historique), `compare` (différence de pixels), `ci diff` (différence d'images).

### `tag`

**Synopsis :** `osatura tag <project.json>`

Propose des noms de rôles standard (`arm.upper.l`, `hand.l`, `face.mouth` ...) pour les os et les calques à partir de leurs noms, avec un degré de confiance, et liste les rôles standard pour lesquels aucun os n'a été trouvé. En lecture seule ; utilisez les rôles comme guide pour `mocap-match` ou `autorig`.

```
$ osatura tag demo.json
bone  arm_up                   -> arm.upper.l    (45%)
bone  hand                     -> hand.l         (45%)
layer mouth                    -> face.mouth     (90%)
no bone found for: head, neck, torso, hips, face.eye.l, ...
```

### `explain`

**Synopsis :** `osatura explain <project.json> <bone-or-layer> <frame>`

« Pourquoi est-ce ici ? » - liste les contributions ordonnées (pose de repos, chaque canal avec son mode et son poids, contraintes, IK) qui produisent la transformation d'un os ou d'un calque à une image donnée, ainsi que l'angle résultant de la tête, de la pointe et du monde.

```
$ osatura explain demo.json hand 12
- bone 'hand' (length 45), parent 'arm_low'
- ancestry: hand ← arm_low ← arm_up
- rest pose: offset (100.00, 0.00), angle 0.00°
- channel #2 BoneAngle: keys → 20.000 (5 key(s), Hold outside range); mode replace, weight 1.00
- channel #3 BoneAngle: expression `6 * sin(time * 2 * pi * 2)`; mode add, weight 1.00
- result at frame 12: head (174.6, 126.4), tip (203.5, 92.0), world angle -50.0°
```
**Erreurs :** nom d'os/de calque inconnu ; image hors de l'animation.

### `pick`

**Synopsis :** `osatura pick <project.json> --frame N --x X --y Y [--tol PX] [--root] [--bones] [--points a,b] [--locked a,b] [--json]`

Ce qui se trouve sous un point du canevas, l'élément le plus haut en premier (test de sélection tel que le fait le studio).

| Option | Signification |
|---|---|
| `--frame N`, `--x X`, `--y Y` | Obligatoires : l'image et les coordonnées du canevas. |
| `--tol PX` | Tolérance de sélection en pixels du canevas (3 par défaut). |
| `--root` | Indique l'ancêtre de premier niveau (`mouth`) plutôt que le calque le plus interne (`mouth/closed`). |
| `--bones` | Prend aussi en compte les os. |
| `--points a,b` | Calques dont les points de contrôle (ancres/poignées/sommets de maillage) sont sélectionnables. |
| `--locked a,b` | Calques à ignorer (traités comme verrouillés). |
| `--json` | `{"hit":{...},"hits":[...]}`. |

```
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4
hit: layer ink (distance 0.00)
  under: layer mouth/closed (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4 --root --locked mouth
hit: layer ink (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 300 --y 240 --json
{"hit":{"distance":0.0,"target":{"kind":"layer","path":"sky"}},"hits":[{"distance":0.0,"target":{"kind":"layer","path":"sky"}}]}
```
**Piège :** le calque d'arrière-plan (`sky`) est une cible de sélection normale, il y a donc presque toujours au moins un résultat.

### `select-box`

**Synopsis :** `osatura select-box <project.json> --frame N (--rect x0,y0,x1,y1 | --lasso x,y;x,y;x,y) [--mode bbox|contain|intersect] [--root] [--bones] [--json]`

Sélection au rectangle ou au lasso. `--mode` détermine quand un calque est considéré comme à l'intérieur : `bbox` (sa boîte englobante touche la zone), `contain` (entièrement à l'intérieur), `intersect` (par défaut : sa géométrie dessinée coupe la zone). `--root` indique les ancêtres de premier niveau, `--bones` inclut les os.
Mettez une valeur de `--lasso` entre guillemets dans un shell (le `;` sépare les points).

```
$ osatura select-box demo.json --frame 0 --rect 200,95,220,115 --root
layer sky
layer arm
layer mouth
layer ink
$ osatura select-box demo.json --frame 0 --rect 0,0,640,480 --mode contain
layer sun
layer shadow
...
$ osatura select-box demo.json --frame 0 --lasso "100,100;300,100;300,300" --bones --json
{"targets":[{"kind":"bone","name":"arm_low"},{"kind":"bone","name":"hand"},{"kind":"layer","path":"sky"}, ...]}
```

### `timeline`

**Synopsis :** `osatura timeline <project.json> [--json]`

La vue dope sheet sous forme de texte : pistes par canal regroupées par os/calque, avec les images clés (`keys @ ...`), les canaux `expr` / `driven`, les verrous et les couleurs. Les numéros `#N` sont les indices de canaux utilisés par `keyedit --channels` et par les sélections `#N@F0..F1`.

```
$ osatura timeline demo.json
timeline: 48 frames at 24 fps, 14 channels
Bones
  arm_up
    #0 bone_angle (keys @ 0 12 24 36 48)
    arm_low
      #1 bone_angle (keys @ 0 12 24 36 48)
      hand
        #2 bone_angle (keys @ 0 12 24 36 48)
        #3 bone_angle (expr)
  foot_ctrl
    #4 bone_offset_x (keys @ 0 12 24 36 48)
...
```
`--json` fournit les groupes avec leurs identifiants, parents, enfants, pistes, `frames`, `locked`.

### `timecode`

**Synopsis :** `osatura timecode <frame|HH:MM:SS;FF> [--fps 24] [--drop] [--nodrop]`

Conversion de timecode SMPTE. Un nombre seul est une image et affiche un timecode ; un argument contenant `:` ou `;` est analysé et affiche le numéro d'image.
Les cadences 29.97, 59.94 et 119.88 fps utilisent par défaut le timecode drop-frame (`;` avant les images) ; `--nodrop` compte 30 (ou 60/120) à la place ; `--drop` force le drop-frame (erreur pour les autres cadences).
`--drop` et `--nodrop` s'excluent mutuellement.

```
$ osatura timecode 100
00:00:04:04
$ osatura timecode 100 --fps 29.97
00:00:03;10
$ osatura timecode 1800 --fps 29.97 --drop
00:01:00;02
$ osatura timecode 1800 --fps 29.97 --nodrop
00:01:00:00
$ osatura timecode 00:00:04:04
100
$ osatura timecode 00:00:04;04 --fps 24
drop-frame timecode is only defined for 29.97, 59.94 and 119.88 fps         [exit 1]
```
Ne lit pas de projet ; `--fps` vaut 24 par défaut (et non le fps d'un projet).

### `snapshot`

**Synopsis :**
```
osatura snapshot save <project.json> <dir> [--keep 20]
osatura snapshot list <project.json> <dir>
osatura snapshot restore <project.json> <dir> <out.json> [--number N]
osatura snapshot recover <project.json> <dir> <out.json>
```
`--keep` n'appartient qu'à `save` et `--number` qu'à `restore` ; toute autre combinaison est refusée (`unknown option --keep for 'snapshot list' (applies to: snapshot save)`), de même qu'un argument de fichier manquant ou en trop.

Un historique numéroté d'un projet dans un répertoire (indépendant de `vcs`). `save` enregistre le projet courant sous le nom `rev-NNNNNN-<hash>.json` et ne conserve que les `--keep` révisions les plus récentes (20 par défaut, les fichiers plus anciens sont supprimés) ; enregistrer un projet identique à la dernière révision affiche `unchanged since the last revision` et ne stocke rien. `list` affiche les révisions. `restore` écrit la révision `--number N` dans `<out.json>` ; `recover` écrit la révision la plus récente.

```
$ osatura snapshot save s.json snaps --keep 3
saved revision 1 (4922f48a86f24554)
$ osatura snapshot list s.json snaps
     1  4922f48a86f24554  snaps/rev-000001-4922f48a86f24554.json
$ osatura snapshot restore s.json snaps restored.json --number 1
wrote restored.json
$ osatura snapshot recover s.json snaps recovered.json
wrote recovered.json
```
`list` sur un répertoire inexistant n'affiche rien et sort avec le code 0. **Voir aussi :** `vcs` pour les branches et les fusions.

---

## 4. Rendu et export

### `render`

**Synopsis :** `osatura render <project.json> <frame> <out.svg|out.png> [--bones] [--scale N] [--clamp] [--proxy N]`

`--proxy N` produit un proxy de scrub (N = 1..4) : un `.png` est un bitmap à la résolution 1/2^N, un `.svg` est un vectoriel simplifié ; tous deux abandonnent les traits fins, les dégradés et les fusions.

Rend une image. Une `<frame>` égale ou supérieure à la longueur du projet est une erreur (`frame 99 is outside the animation ... valid frames are 0..N-1 ... pass --clamp`) ; `--clamp` rend à la place la dernière image et affiche un avertissement sur stderr. (Seul `render` effectue cette vérification ; `dof`, `render-hdr` et `fx` ne la font pas.) L'extension choisit le format (`.svg` vectoriel, `.png` raster RGBA 8 bits) ; toute autre extension est refusée **avant** que quoi que ce soit ne soit écrit.

| Option | Par défaut | Signification |
|---|---|---|
| `--bones` | désactivé | Dessine la superposition du squelette (têtes sous forme de points, os sous forme de lignes). |
| `--scale N` | 1 | Échelle de sortie, un nombre dans (0, 16]. Pour `.png`, elle multiplie la taille en pixels ; pour `.svg`, le document conserve son viewBox. |
| `--proxy N` | désactivé | Proxy de scrub, N = 1..4 : `.png` = bitmap à la résolution 1/2^N, `.svg` = vectoriel simplifié ; tous deux abandonnent les traits fins, les dégradés et les modes de fusion. Destiné au scrub interactif, pas aux livrables. |

```
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura render demo.json 0 /tmp/x.xyz
render writes .png or .svg; '/tmp/x.xyz' has another extension (nothing was written)       [exit 1]
$ osatura render demo.json 0 o.svg --scale 0
--scale needs a number in (0, 16]                                                         [exit 1]
$ osatura render demo.json abc o.svg
bad frame number 'abc'                                                                    [exit 1]
$ osatura render demo.json 2 px.png --proxy 2        # 640x480 project -> 160x120 PNG
wrote px.png
```
**Erreurs :** projet manquant/invalide, pose non finie à cette image, sortie identique au projet d'entrée. **Voir aussi :** `frames`, `render-hdr`, `onion`, `compare`, `fx`.
**Limites :** le rastériseur PNG est celui du moteur (8 bits, sRGB ; la fusion n'est pas gérée en couleur sauf si le projet l'active avec `color-space ... linear`). Les effets de calque, les masques et les modes de fusion sont dessinés par les deux.

### `frames`

**Synopsis :** `osatura frames <project.json> <out_dir> [--png] [--bones] [--scale N]`

Rend **chaque** image dans un répertoire (créé s'il n'existe pas) sous la forme `frame_0000.svg` ... (ou `.png` avec `--png`). Toutes les images sont d'abord vérifiées pour s'assurer que leurs poses sont finies.

```
$ osatura frames demo.json fr --png
wrote 48 frames to fr
$ ls fr | head -2
frame_0000.png
frame_0001.png
```
**Piège :** les fichiers existants portant les mêmes noms sont écrasés ; les fichiers sans rapport présents dans le répertoire sont laissés intacts. Servez-vous-en comme entrée pour `ffmpeg` ou `import-seq`.

### `export`

**Synopsis :**
```
osatura export <project.json> <out.gif> [--scale N] [--report r.json]
osatura export <project.json> <out.html|out.htm>
osatura export <project.json> <out.json|out.lottie> [--report r.json]
osatura export <project.json> <out.mp4> [--scale N]
osatura export <project.json> <out.apng> [--scale N]
osatura export <project.json> <out.webm> [--scale N] [--codec vp9|av1]
osatura export <project.json> <out.png> [--scale N] [--columns N]
osatura export <project.json> <out.glb> [--report r.json]
```
(une forme par format ; l'extension de sortie, sans distinction de casse, choisit la forme. Une option que la forme ne mentionne pas est refusée : `unknown option --scale for 'export .json' (applies to: export .gif, export .mp4, ...)`. Avec une extension inconnue ou absente, c'est l'erreur de format qui est signalée.)

L'extension de sortie sélectionne le format :

| Extension | Résultat | Nécessite ffmpeg | Options applicables |
|---|---|---|---|
| `.gif` | GIF en boucle à la cadence du projet (voir [EXPORT_FORMATS.md](EXPORT_FORMATS.md)) | non | `--scale`, `--report` |
| `.html`, `.htm` | un lecteur HTML autonome (images SVG intégrées, lecture/pause, curseur de défilement, audio intégré en option) | audio uniquement | - |
| `.json` | Lottie (Bodymovin) | non | `--report` |
| `.lottie` | conteneur dotLottie (zip : `manifest.json` + `animations/animation.json`) | non | `--report` |
| `.mp4` | H.264 yuv420p (+ AAC lorsque le projet contient des sons) | oui (`libx264`, `aac`) | `--scale` |
| `.apng` | PNG animé | non | `--scale` |
| `.webm` | VP9 (par défaut) ou AV1 avec alpha (+ son Opus) | oui (`libvpx-vp9` / `libaom-av1`, `libopus`) | `--scale`, `--codec vp9\|av1` |
| `.png` | sprite sheet **plus** métadonnées `<out>.atlas.json` (jamais `<out>.json`) | non | `--scale`, `--columns N` |
| `.glb` | glTF 2.0 binaire (os, animation, remplissages/traits skinnés, dégradés, masques, images, morph targets) | non | `--report` |

Options :

| Option | Signification |
|---|---|
| `--scale N` | Échelle du raster (0, 16] pour `.gif .mp4 .apng .webm .png`. Sur les autres formats, elle est refusée (`unknown option --scale for 'export .json'`) ; avant la vague 7, c'était un avertissement. |
| `--report FILE` | Écrit en JSON la liste des éléments que le format n'a pas pu transporter (`.gif .json .lottie .glb`) : `{"format","count","warnings":[{"code","layer","message"}]}`. Les mêmes avertissements sont toujours affichés sur stderr. Le fichier ne peut pas être le projet d'entrée. |
| `--codec vp9\|av1` | `.webm` uniquement (`vp9` par défaut) ; toute autre valeur : `unknown --codec 'foo' (vp9 or av1)`. |
| `--columns N` | Sprite sheet `.png` uniquement : nombre de vignettes par ligne (par défaut : une grille presque carrée, ceil(sqrt(frames)) : 3 colonnes pour 8 images). |

Exemples (tous vérifiés sur une copie à 8 images de la démo, `--scale 0.5`) :

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ cat rep.json
{ "count": 2, "format": "lottie", "warnings": [ {"code": "variable-width-stroke", "layer": "ink", ...}, {"code": "gradient-anisotropic", ...} ] }
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.json
wrote out3.png                          # 4 columns x 2 rows of 160x120 tiles; out3.json lists {index,x,y,w,h} per frame
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
$ osatura export small.json out.mp4 --scale 0.5       # 320x240 h264 yuv420p, 8 frames
wrote out.mp4
$ osatura export small.json out.webm --codec av1      # needs a libaom-av1 build of ffmpeg
```

Erreurs et refus :

```
$ osatura export small.json z.xyz
unknown export format '.xyz' (use .gif .html .htm .json .lottie .mp4 .apng .webm .png .glb)
$ osatura export small.json small.json
refusing to overwrite the input project small.json with output of the same name
$ osatura export ora.json ora.glb                       # a project of image layers and no bones
the project has no bones to export
$ PATH=/nonexistent osatura export small.json y.mp4
cannot run ffmpeg (...); install it or export a GIF/HTML instead                          [exit 1, nothing written]
```
Autres refus : `ffmpeg failed` (encodeur manquant), `export would write about N million values ...` (la garde de taille Lottie/HTML/GLB : la limite est de 40 millions de valeurs = images x (ancres + os + calques) ; raccourcissez l'animation ou simplifiez les chemins), poses non finies sur une image quelconque, et `image too large for GIF`.

**Sorties / effets de bord :** `.png` écrit aussi les métadonnées de la sprite sheet dans `<out-name>.atlas.json` (par ex. `out.png` -> `out.atlas.json`), de sorte qu'il ne peut plus remplacer un fichier Lottie `out.json` ; le fichier annexe est soumis aux mêmes gardes contre l'écrasement de l'entrée et contre les liens symboliques. `.mp4`/`.webm` rendent des images PNG dans un répertoire temporaire privé qui est supprimé ensuite.
**Son :** `.mp4` et `.webm` mixent les pistes de son du projet (début, gain, rognage, fondus ; coupées à la longueur du projet) ; `.html` intègre un mp3 mixé (avertissement au-delà de 3 Mio, abandonné au-delà de 16 Mio avec un avertissement) ; `.gif` et `.apng` affichent `note: GIF cannot carry audio; ...` et restent muets ; Lottie avertit par `[audio]`.
**Transparence :** un projet avec `"background": null` est rendu avec des pixels transparents partout où rien n'est dessiné. `.apng`, la sprite sheet `.png` et `.webm` (`yuva420p`, balise de conteneur `alpha_mode=1`) conservent le canal alpha ; `.gif` conserve une transparence sur 1 bit (vérifié : le drapeau de couleur transparente est positionné) ; `.mp4` n'a pas d'alpha. Un projet avec une couleur de fond (la démo a `#f6f1e7`) est opaque.
**Voir aussi :** `export-fit` (respecter un budget d'octets), `size-report`, `lod`, `dof-export`, `export-oca`, `render-hdr`. Détail complet des formats : [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

### `export-oca`

**Synopsis :** `osatura export-oca <project.json> <out_dir> [--layers]`

Écrit un dossier **Open Cel Animation** : `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) plus des cels PNG `layerN/fNNNN.png`. Sans `--layers`, il y a un seul calque `render` contenant les images composées ; avec `--layers`, chaque calque de **premier niveau** est rendu seul sur fond transparent dans son propre calque OCA (11 calques pour la démo). `--scale N` multiplie la taille des cels.

```
$ osatura export-oca small.json oca2 --layers
wrote oca2/small.oca
$ ls oca2
layer0 layer1 ... layer10 small.oca
```
**Limites :** les cels sont des rasters (ni vecteurs, ni os, ni canaux ne survivent) ; l'opacité et la fusion des calques ne sont pas écrites (chaque calque est un `paintlayer`, d'opacité 1) ; les sons ne sont pas exportés ; le dossier est créé si nécessaire et les fichiers qu'il contient sont écrasés.

### `render-hdr`

**Synopsis :** `osatura render-hdr <project.json> <frame> <out.exr|out.png> [linear] [--precision half|float] [--float32] [--8bit] [--scale N] [--clamp]`

Rendu en virgule flottante (32 bits) (une `<frame>` hors du projet est une erreur, `--clamp` prend l'image valide la plus proche, comme pour `render`) : `.exr` écrit un **OpenEXR** mono-partie, par lignes de balayage, non compressé (canaux HALF par défaut, FLOAT avec `--precision float` ou `--float32`), en lumière linéaire et prémultiplié ; `.png` écrit un **PNG RGBA 16 bits**, encodé en sRGB sauf si le mot littéral `linear` est donné comme cinquième argument. Le fond du projet n'est **pas** dessiné (l'image est transparente là où rien n'est dessiné). `--8bit` utilise l'ancien rastériseur 8 bits comme source (ne peut pas être combiné avec une sortie float32). `--scale N` s'applique.

```
$ osatura render-hdr small.json 3 f.exr
wrote f.exr (640x480)                       # 465,611 bytes: HALF
$ osatura render-hdr small.json 3 f2.exr linear --precision float
wrote f2.exr (640x480)                      # 923,211 bytes: FLOAT
$ osatura render-hdr small.json 3 f4.png
wrote f4.png (640x480)                      # PNG 16-bit/colour RGBA
$ osatura render-hdr small.json 3 f6.exr --precision bogus
unknown precision 'bogus' (use half or float)
$ osatura render-hdr small.json 3 f7.jpg
output must end in .exr or .png
```
`ffprobe` lit l'EXR comme `exr 640x480 gbrapf32le`. Pour `.exr`, le mot `linear` n'a aucun effet (l'EXR est toujours linéaire). **Limites :** pas de compression, une image par fichier, pas de calques/AOV ; l'art source est en couleur sRGB ordinaire, le fichier contient donc des valeurs de la plage d'affichage avec une précision plus élevée, et non une lumière HDR référencée à la scène. Pas de gestion de la couleur au-delà de la conversion sRGB<->linéaire.

### `render-gpu`

**Synopsis :** `osatura render-gpu <project.json> <out.png> [--frame N] [--adapter software|auto] [--scale S] [--bench N]`

Rendu GPU (wgpu) d'une image. Disponible uniquement dans les builds réalisés avec `--features gpu` ; le build par défaut répond :

```
$ osatura render-gpu small.json g.png
render-gpu needs a build with the gpu feature: cargo build -p osatura-cli --features gpu       [exit 1]
```
Dans un build GPU, les scènes non prises en charge (images, os) sont signalées par des erreurs, jamais dessinées de façon incorrecte. `--bench N` est une option de benchmark. Cette commande n'a pas pu être exécutée dans le build de test (sans la fonctionnalité gpu) ; ses options sont donc reprises du seul texte d'utilisation.

### `compare`

**Synopsis :** `osatura compare <a.json> <b.json> <frame> <out.png> [--gain 4 --scale N]`

Écrit un PNG : **avant | après | différence** côte à côte (le troisième panneau est une carte thermique, noir = identique, rouge -> jaune = changement plus important, multiplié par `--gain`, 4 par défaut). Les deux projets doivent être rendus à la même taille (`frame sizes differ (...); compare projects of the same size`).

```
$ osatura compare small.json d2.json 3 cmp.png --gain 4 --scale 0.25
wrote cmp.png
```
**Voir aussi :** `diff` (structure), `ci diff` (deux PNG, métriques perceptuelles), `golden`.

### `proxy-bench`

**Synopsis :** `osatura proxy-bench <project.json> [--frames N]`

Millisecondes par image, mono-thread, en pleine qualité et à chaque niveau de proxy de scrub (voir `render --proxy`).

```
$ osatura proxy-bench small.json --frames 2
2 frames of small (640x480), ms per frame, single thread:
  evaluate only           0.08
  svg (no raster)         0.48   vector proxy: level 0 0.32, level 2 0.24, level 4 0.24
  raster level 0       1360.87   (full quality)
  raster level 1        319.14   (4.3x faster than level 0)
  ...
  raster level 4          6.84   (199.0x faster than level 0)
```
(valeurs du build debug). 

---

## 5. Rigging, skins, IK et aides de mise en page

Toutes les commandes de cette section suivent le schéma `osatura <cmd> <project.json> <out.json> ...` : elles lisent un projet, le modifient, le **revalident**
et écrivent `<out.json>` (qui peut être identique à l'entrée). En cas d'erreur, rien n'est écrit.

### `autorig`

**Synopsis :** `osatura autorig <project.json> <out.json> [seed]`

Ajoute des automatismes standard déduits des **noms** des os et des calques : chaînes IK (mix 0 pour que le FK continue de fonctionner), une boucle de respiration (nécessite un os de torse) et le clignement (nécessite des calques d'yeux). Il affiche une ligne par décision : `+` ajouté, `-` ignoré avec la raison. Exécute aussi `eye-clip`.

```
$ osatura autorig small.json auto.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
- arm_r: upper/lower bone not found
- leg_l: already has an IK chain
- breathing: no torso bone
- blink: no eye layers
wrote auto.json
```
`seed` (positionnel facultatif) initialise le hasard du minutage des clignements. **Limites :** purement fondé sur les noms (`tag` montre comment les noms sont lus) ; ne crée jamais d'os.

### `slider`

**Synopsis :** `osatura slider <project.json> <out.json> <name> <poses>` où `poses` est `at:target.prop=val,target.prop=val;at:...`

Crée un **os contrôleur** `ctrl_<name>` dont l'`offset_x` pilote plusieurs propriétés entre des poses mémorisées (un slider de poses). Animez le `bone_offset_x` du contrôleur affiché.

```
$ osatura slider small.json sl.json pose "0:arm_up.bone_angle=0;24:arm_up.bone_angle=30"
controller bone: ctrl_pose (animate its offset_x)
wrote sl.json
```
`target.prop` utilise les noms de propriétés en snake_case (`arm_up.angle` échoue avec `unknown property 'angle'`). Le dernier point sépare la cible de la propriété, de sorte que des chemins de calque tels que `grp/child.layer_opacity` fonctionnent.

### `view-swap`

**Synopsis :** `osatura view-swap <project.json> <out.json> <switch_layer> <bone.prop> <from:child,from:child,...>`

Pilote un calque switch à partir d'une propriété d'os avec une fonction en escalier : les paires `from:child` sont triées selon `from`, et le switch affiche l'enfant de la dernière paire dont le `from` n'est pas supérieur à la valeur du pilote (`-40:0,0:1,40:2` = enfant 0 en dessous de 0, enfant 1 à partir de 0, enfant 2 à partir de 40). Servez-vous-en pour des vues de rotation (face/profil/dos) pilotées par un os de lacet. Les indices d'enfants doivent exister (`child index 5 is out of range (the layer has 3)`) ; le calque doit être un switch.

```
$ osatura view-swap small.json vs.json mouth foot_ctrl.bone_offset_x "-40:0,0:1,40:2"
wrote vs.json
```

### `align` et `distribute`

**Synopsis :**
`osatura align <project.json> <out.json> <mode> <layer,layer,...|layer> [stage]`  (mode : `left right top bottom center-x center-y`)
`osatura distribute <project.json> <out.json> <x|y> <gaps|centers> <layer,layer,...>`

`align` déplace des calques pour aligner les boîtes englobantes de leurs courbes aplaties (entre eux ou, pour un seul calque suivi du mot `stage`, sur le canevas). `distribute` espace trois calques ou plus de façon régulière le long d'un axe, en égalisant soit les **écarts** (gaps) entre les boîtes, soit les **centres**.

```
$ osatura align small.json al.json center-x sun,hills stage
wrote al.json
$ osatura align small.json al2.json top ground
select at least two layers (or one with --stage)                 [exit 1]
$ osatura distribute small.json di.json x gaps sun,ground,hills
wrote di.json
```
Ces commandes modifient les positions des calques (transformation statique), de sorte que les canaux `layer_x`/`layer_y` animés par clés les écrasent toujours. (Le message d'erreur mentionne `--stage` ; l'argument réel est le simple mot `stage`.)

### `trim`

**Synopsis :** `osatura trim <project.json> <out.json> <layer> <path_index> <start> <end> [offset]`

Fige un *trim path* : remplace le chemin numéro `path_index` d'un calque vectoriel par la seule partie comprise entre `start` et `end` (fractions 0..1 de sa longueur, toutes deux décalées de `offset` ; les chemins fermés bouclent). Le résultat est une **polyligne ouverte avec le même trait et sans remplissage** - destructif, conservez donc l'original. Pour un effet de tracé progressif, figez une copie rognée par image.

```
$ osatura trim small.json tr.json ink 0 0 0.5 0.25
wrote tr.json
```

### `reparent`

**Synopsis :** `osatura reparent <project.json> <out.json> <bone> <new_parent|none>`

Change le parent d'un os **sans le déplacer** : le décalage et l'angle de repos sont recalculés pour que la pose de repos reste inchangée (vérifié : seuls `parent`, `rest_angle` et `rest_offset` diffèrent ensuite). `none` en fait une racine. Un parent qui créerait un cycle est refusé.

```
$ osatura reparent small.json rp.json hand none
wrote rp.json
$ osatura diff small.json rp.json
~ bone 'hand' changed (parent, rest_angle, rest_offset)
```

### `parent-switch`

**Synopsis :** `osatura parent-switch <project.json> <out.json> <bone> <frame:parent|none,...>`

Fige un changement de parent animé (une main qui saisit un accessoire) : à chaque image de bascule, la pose monde de l'os relativement au nouveau parent est capturée et conservée (suivi rigide), ou figée dans l'espace monde pour `none`. L'os devient une racine et sa pose monde est écrite sous forme de clés ordinaires de décalage/angle pour chaque image ; ses anciens canaux restent dans le fichier mais sont écrasés par ceux qui ont été figés. **Limites :** refusé pour les os impliqués dans l'IK, les contraintes ou les os à ressort ; l'échelle n'est pas reportée.

```
$ osatura parent-switch demo.json ps.json hand 0:none,24:arm_up
wrote ps.json
```

### `skin`

**Synopsis (sous-commandes) :**
```
osatura skin <project.json> <out.json> add <group> <name> <layer,layer,...>
osatura skin <project.json> <out.json> rig <group> <name> <constraint,...|-> <ik,...|->
osatura skin <project.json> <out.json> bones <group> <name> <bone,...|->
osatura skin <project.json> <out.json> apply <group> <name>
osatura skin <project.json> <out.json> schedule <group> <frame:name,frame:name,...>
```

Un *jeu de skins* est identifié par une étiquette `<group>` (un jeu de « tenues » ou d'« expressions » ; dans l'exemple, elle se trouve être égale au nom d'un groupe de calques). Chaque skin nommé affiche un ensemble de calques, donnés par **chemin de calque** (par ex. `skins/box`) ; appliquer un skin affiche ses calques et masque les calques de tous les autres skins du même groupe.
`add` définit un skin ; `rig` liste les contraintes et chaînes IK qui ne sont actives que lorsque ce skin l'est ; `bones` liste les os qui n'existent que lorsqu'il est actif (réduits à l'échelle 0 sinon) ; `apply` rend un skin actif de façon statique ; `schedule` pose des clés pour le skin actif au fil du temps sous forme de clés en escalier sur l'**opacité des calques** (n'animez donc pas autrement l'opacité de ces calques). `-` signifie « aucun ».

```
$ osatura skin sc.json sk1.json add skins red skins/box
wrote sk1.json
$ osatura skin sk1.json sk2.json add skins green skins/box2
$ osatura skin sk2.json sk3.json apply skins green
$ osatura skin sk3.json sk4.json schedule skins 0:red,4:green
$ osatura skin sk4.json sk5.json bones skins red -
$ osatura skin sk4.json sk5.json rig skins red - leg
$ osatura skin sc.json x.json add skins red box
no layer 'box'                                                  [exit 1]     (use the full path skins/box)
```
**Limites (ROADMAP §9) :** un os qui n'existe que dans un skin n'est jamais retiré du rig évalué (les indices restent ; il est mis à l'échelle 0) ; un calque pondéré sur un os propre à un skin mais n'appartenant pas à ce skin disparaît avec lui, et cela n'est pas vérifié.

### `bone-vis`

**Synopsis :** `osatura bone-vis <project.json> <out.json> <bone> <from> <to>|clear`

Affiche l'os dans les **overlays** du studio uniquement sur une plage d'images (contrôle de l'encombrement). Ne modifie ni le rendu ni l'export. `clear` supprime la plage (`range removed`).

### `chan-meta`

**Synopsis :** `osatura chan-meta <project.json> <out.json> <target> <prop|*> [color=#rrggbb] [shy=on|off] [solo=on|off] [lock=on|off]`

Métadonnées de dope sheet d'un canal : couleur d'affichage, *shy* (masqué dans les listes), *solo* (prévisualiser uniquement ces canaux dans le studio) et **lock** (refuser les modifications : les canaux verrouillés sont ignorés par `keys`, refusés par `keyedit`, `audio-react` et `live-map`, et jamais touchés par `clip bake`). L'évaluation et l'export ignorent ces quatre réglages. `*` sélectionne toutes les propriétés de la cible.

```
$ osatura chan-meta small.json cm.json arm_up bone_angle 'color=#ff0000' shy=on lock=on
wrote cm.json
```

### `reference`

**Synopsis :** `osatura reference <project.json> <out.json> <layer> <on|off>`

Marque un calque comme **référence de tracé** : affiché dans le studio, jamais exporté.

### `parallax`

**Synopsis :** `osatura parallax <project.json> <out.json> <far_layer,...,near_layer> <far_value> <near_value>`

Attribue une profondeur multiplan (valeurs de `parallax` réparties entre `far_value` et `near_value`) aux calques de premier niveau listés, dans l'ordre donné (le premier = le plus éloigné). Avec un mouvement de caméra, les calques se décalent de quantités différentes ; `dof` floute selon `|parallax - focus|`.

```
$ osatura parallax small.json pa.json sky,hills,ground 0.2 1.0
wrote pa.json
```

### `eye-clip`

**Synopsis :** `osatura eye-clip <project.json> <out.json>`

Masque chaque calque d'iris/de pupille par le calque de forme d'œil du même côté (trouvé par son nom, voir `tag`) à l'aide d'un masque de calque ordinaire, de sorte que les pupilles ne quittent jamais l'œil lorsqu'elles bougent. Également exécuté par `autorig`. Lorsqu'aucun nom correspondant n'est trouvé, le fichier est tout de même écrit, inchangé.

### `comp`

**Synopsis :**
```
osatura comp capture|apply <project.json> <out.json> <name>
osatura comp list <project.json>
```

États de calques nommés (« comps » : visibilité, opacité et enfant sélectionné des calques switch, par chemin de calque ; **pas** les transformations). `capture` enregistre l'état courant sous `name`, `apply` le restaure, `list` affiche les noms (`list` ne prend pas de fichier de sortie et n'écrit rien ; un quatrième mot hérité, `comp list p.json p.json`, est toujours accepté et ignoré).

```
$ osatura comp capture small.json c1.json pose1
wrote c1.json
$ osatura comp list c1.json c1.json
pose1 (14 layers)
$ osatura comp apply c1.json c2.json pose1
wrote c2.json
```

### `retarget`

**Synopsis :** `osatura retarget <source.json> <dest.json> <out.json> --pairs srcChain=dstChain[,..]`

Retarget par *intention* : l'animation de l'**effecteur** IK (os cible) de chaque chaîne source est mise à l'échelle selon le rapport de la portée des deux membres (somme des longueurs d'os) et redirigée vers l'os cible de la chaîne de destination. Les canaux d'effecteur non animés par clés ne peuvent pas être mis à l'échelle ; ils sont transmis renommés et signalés par des lignes `note:`. `--pairs srcChain=dstChain[,...]` est obligatoire. Dans l'exemple, la source et la destination sont le même projet, ce qui ne sert que de test de fumée.

```
$ osatura retarget demo.json demo.json rt.json --pairs leg=leg
wrote rt.json (+14 channels)
```
**Voir aussi :** `mocap-retarget` pour les données BVH ; `walk`, `mirror`.

---

## 6. Animation, clés, timeline et générateurs

### `keys`

**Synopsis :**
```
osatura keys shift <project.json> <out.json> --by N [--from F --to F --target NAME --prop PROP]
osatura keys scale <project.json> <out.json> --factor X [--pivot F --from F --to F --target NAME --prop PROP]
osatura keys ease <project.json> <out.json> --ease linear|step|ease_in|ease_out|ease_in_out|smooth [--from F --to F --target NAME --prop PROP]
osatura keys reverse <project.json> <out.json> [--from F --to F --target NAME --prop PROP]
```
Chaque opération n'accepte que ses propres options (`keys shift --factor 2` est refusé) ; `--by`, `--factor` et `--ease` sont obligatoires pour leur opération (un `--by` ou `--factor` omis était auparavant sans effet, en silence).

Opérations de clés en masse sur **tous les canaux animés par clés** (filtrés par `--target` / `--prop` ; une plage `--from/--to` limite les clés touchées). Les canaux verrouillés sont ignorés avec `skipped locked channel ...`.

| Opération | Requiert | Effet |
|---|---|---|
| `shift` | `--by N` | Déplace les clés de N images. |
| `scale` | `--pivot F --factor X` | Met à l'échelle les temps des clés autour de l'image pivot. |
| `ease` | `--ease E` | Définit l'interpolation des clés (interpolations nommées ; utilisez `keyedit ease` pour `bezier:`). |
| `reverse` | - | Inverse l'ordre des clés dans la plage. |

```
$ osatura keys shift small.json k1.json --target arm_up --prop bone_angle --by 2
wrote k1.json (5 key(s) changed)
$ osatura keys scale small.json k2.json --pivot 0 --factor 0.5 --from 0 --to 48
wrote k2.json (37 key(s) changed)
$ osatura keys reverse small.json k4.json
wrote k4.json (37 key(s) changed)
```
Les clés déplacées en dehors de `0..frames-1` restent dans le fichier (lint signale `L007`) ; cette commande n'allonge pas la durée du projet. **Voir aussi :** `keyedit` (précis, fondé sur une sélection), `loop-solve`.

### `keyedit`

**Synopsis :** `osatura keyedit <project.json> <out.json|-> <sub-command> [options]`

Modifications de dope sheet sous la forme d'**un seul lot sûr pour l'annulation**. Avec `-` comme sortie, rien n'est écrit et l'opération prévue est affichée en JSON (simulation). Sous-commandes et options :

```
osatura keyedit <project.json> <out.json|-> move --sel SEL [--dt N] [--dv N] [--collision replace|merge|refuse] [--no-snap] [--free]
osatura keyedit <project.json> <out.json|-> scale --sel SEL --pivot F --factor X | dup --sel SEL --offset N | delete --sel SEL | ease --sel SEL --ease E | value --sel SEL --value V | smooth --sel SEL [--kind monotone|catmull_rom]
osatura keyedit <project.json> <out.json|-> copy --sel SEL --clip clip.json | paste --clip clip.json --at F [--remap from_target:prop=to_target:prop]
osatura keyedit <project.json> <out.json|-> add --target T --prop P --at F [--value V] [--ease E] | ripple --at F --by N --mode insert|delete [--scope 0,1] [--adjust-frames]
osatura keyedit <project.json> <out.json|-> offset --channels 0,1 --dt N | extrapolate --channels 0,1 --extrapolate loop | loop --channels 0,1 [--end F] | handle --sel SEL --side in|out --t F --v V [--mode free|aligned|auto] | lock --target T [--prop P] [--value on|off]
```

* **SEL** est constitué d'éléments `TARGET:PROP[@F0..F1]` joints par `;`, ou `#CHANNEL[@F0..F1]` (indice de canal fourni par `timeline`), ou `all`. Mettez-le entre guillemets (`;`, `@`, `#`).
* **E** (interpolation) vaut `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`, `smooth`, `bounce`, `bezier:x1,y1,x2,y2` ou un objet JSON.
* `--collision` décide de ce qui se passe lorsqu'une clé déplacée tombe sur une clé existante (`replace` par défaut, `merge`, `refuse`) ; `--no-snap` conserve les images fractionnaires ; `--free` lève la restriction aux images entières.
* Ligne de sortie : `<op>: N keys changed, wrote <out>`. `copy` avec une vraie sortie affiche `copy: 0 keys changed, project unchanged, clip saved`.
* Les opérations qui placeraient une clé en dehors de `0..=frames` sont refusées avec un message précis (`duplicate: frame 52 is outside the project range 0..=48`), sauf si `ripple --adjust-frames` allonge le projet.

```
$ osatura keyedit demo.json - move --sel "arm_up:bone_angle@12..36" --dt 2        # dry run: prints the batch JSON
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", ... } ] } }
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 2 --collision merge
move: 3 keys changed, wrote k1.json
$ osatura keyedit demo.json k5.json ease --sel "arm_up:bone_angle" --ease bezier:0.4,0,0.2,1
ease: 5 keys changed, wrote k5.json
$ osatura keyedit demo.json k7.json smooth --sel "arm_up:bone_angle" --kind monotone
smooth: 4 keys changed, wrote k7.json
$ osatura keyedit demo.json k8.json copy --sel "arm_up:bone_angle@0..12" --clip c2.json
copy: 0 keys changed, project unchanged, clip saved
$ osatura keyedit demo.json k8.json paste --clip c2.json --at 30 --remap arm_up:bone_angle=arm_low:bone_angle
paste: 2 keys changed, wrote k8.json
$ osatura keyedit demo.json k9.json add --target arm_up --prop bone_angle --at 6 --value 5 --ease linear
add: 1 keys changed, wrote k9.json
$ osatura keyedit demo.json k10.json ripple --at 12 --by 6 --mode insert --adjust-frames
ripple: 28 keys changed, wrote k10.json            # project now 54 frames
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 12 --collision refuse
a key already exists at frame 48                    [exit 1]
$ osatura keyedit demo.json k1.json move --sel "zzz:bone_angle" --dt 1
nothing matches 'zzz:bone_angle'                    [exit 1]
$ osatura keyedit demo.json k15.json lock --target arm_up --prop bone_angle --value on
lock: 1 keys changed, wrote k15.json
```
**Pièges :** `delete` avec `#0@12..12` supprime exactement une clé ; `ripple --mode delete --scope 0,1` limite le décalage aux canaux 0 et 1 ; `copy` et `paste` échangent un fichier de clip, et non le presse-papiers du système. Les canaux verrouillés refusent `move/scale/...` (utilisez d'abord `lock --value off`).

### `marker`

**Synopsis :** `osatura marker <project.json> <out.json> <name> <frame> [--end F]`

Ajoute un marqueur de timeline (un point nommé ou, avec `--end`, une plage). Un marqueur du même nom est remplacé. Les marqueurs sont triés par image.

```
$ osatura marker small.json m1.json hit 4 --end 6
wrote m1.json (marker 'hit' at 4 = 00:00:00:04)
```

### `event`

**Synopsis :** `osatura event <project.json> <out.json> <name> <frame> [--data JSON]`

Ajoute un événement nommé (un déclencheur d'exécution pour les lecteurs) à une image, avec une charge utile JSON facultative. JSON invalide : `--data is not valid JSON: ...`.

```
$ osatura event small.json ev1.json boom 3 --data '{"power":2}'
wrote ev1.json (1 events)
```

### `idle`

**Synopsis :**
```
osatura idle blink <project.json> <out.json> --target T --prop P [--interval S --jitter X --closed F --seed N]
osatura idle breath <project.json> <out.json> --target T --prop P [--amp A --period F]
osatura idle sway <project.json> <out.json> --target T --prop P [--amp A --freq HZ --seed N]
```
Chaque générateur n'accepte que ses propres options (`idle breath --seed 2` est refusé).

Génère un canal procédural de repos (idle) et l'**ajoute** (les canaux existants restent).

| Générateur | Options (valeurs par défaut) | Cible typique |
|---|---|---|
| `blink` | `--interval` nombre moyen de secondes entre deux clignements, `--jitter`, `--closed` images de fermeture, `--seed` | un `layer_switch` (œil ouvert/fermé) ou `layer_scale_y` |
| `breath` | `--amp` (0.03), `--period` en images (3 s à la cadence du projet) | `bone_scale` d'un torse |
| `sway` | `--amp` (2), `--freq` Hz (0.3), `--seed` (1) | `bone_angle` |

```
$ osatura idle blink small.json i1.json --target mouth --prop layer_switch --interval 1 --seed 3
wrote i1.json
$ osatura idle sway small.json i3.json --target arm_up --prop bone_angle --amp 3 --freq 0.5
wrote i3.json
```
`--target` et `--prop` sont obligatoires (`--target is required`).

### `follow` (deux formes)

**Follow-through (options) :** `osatura follow <project.json> <out.json> --target SRC --prop P --chain a,b,c [--delay F --decay X]`
Ajoute aux os de la chaîne des copies retardées et amorties d'un canal existant (`--target`/`--prop` désignent la source) : action qui se chevauche / follow-through (`--delay` images par maillon, 3 par défaut ; `--decay` par maillon, 0.7 par défaut). Une copie retardée est coupée à la dernière image du projet (aucune clé n'est écrite après l'image `frames`, de sorte que `lint` L007 ne se déclenche pas) ; les images situées entre la dernière clé conservée et la coupure sont échantillonnées pour que la courbe retardée soit exacte sur chaque image entière.

```
$ osatura follow small.json f1.json --target arm_up --prop bone_angle --chain arm_low,hand --delay 2 --decay 0.6
wrote f1.json (+2 channels)
```

**Suivi de chemin (positionnel) :** `osatura follow <project.json> <out.json> <layer|bone:NAME> <layer_with_path> <start_frame> <end_frame> [no-orient]`
Fige le déplacement d'un calque (ou de l'os racine `bone:NAME`) le long du premier chemin d'un autre calque entre deux images, en le faisant pivoter selon la tangente du chemin sauf si `no-orient` est donné.

```
$ osatura follow small.json fp.json target ink 0 7
wrote fp.json
$ osatura follow small.json fp2.json bone:hand ink 0 7 no-orient
path following needs a root bone (no parent)                      [exit 1]
```

### `take-record`, `take-apply`, `take-list`

**Synopsis :**
```
osatura take-record <project.json> <library.json> <name> <from> <to> [target_prefix,...]
osatura take-apply  <project.json> <out.json> <library.json> <name> <at> [from=to,...] [speed] [--blend FRAMES]
osatura take-list   <library.json>
```
Un **take** est une tranche d'animation réutilisable stockée dans un fichier de bibliothèque. `take-record` capture les images `from..to` des canaux dont la cible commence par l'un des préfixes (tous les canaux si omis) ; la bibliothèque est créée si elle n'existe pas et un take du même nom est remplacé. `take-apply` colle le take à l'image `at`, avec en option un **retargeting** (les paires `from=to` renomment les cibles) et un changement de `speed` ; les arguments sont vérifiés strictement : une entrée sans `=`, un nom de source pour lequel le take n'a pas de piste, ou une vitesse qui n'est pas un nombre positif est une erreur, et une bibliothèque illisible nomme le fichier (`cannot read the take library ...`) ; `--blend N` fait un fondu enchaîné avec l'animation existante sur N images aux deux extrémités.

```
$ osatura take-record demo.json lib.json wave 0 24 arm_
recorded 2 tracks into lib.json
$ osatura take-list lib.json
wave (24 frames, 2 tracks)
$ osatura take-apply demo.json ta.json lib.json wave 10 arm_up=arm_low 1.5 --blend 4
applied 2 tracks; wrote ta.json
```

### `clip`

**Synopsis :**
```
osatura clip add <project.json> <out.json> <library.json> <take> <clip> [--at F --speed S --loops N --mode replace|additive|multiply --weight W
                   --include a_*,b --exclude c --track-weight arm_*=1,leg_*=0.2 --rename a=b --fade-in F --fade-out F]
osatura clip list <project.json>
osatura clip remove <project.json> <out.json> <clip>
osatura clip set-weight <project.json> <out.json> <clip> [W] [--keys frame:w,frame:w] [--track PATTERN[:prop]]
osatura clip bake <project.json> <out.json>
```
Une **pile de clips** en couches (animation non linéaire) : les clips référencent des takes d'une bibliothèque, se jouent à `--at` avec `--speed`/`--loops`, et se mélangent en `replace`, `additive` ou `multiply` avec un poids (constant, animé par clés dans le temps, ou par motif de piste). `clip add/set-weight/remove` ne modifient que la pile ; **`clip bake` régénère les canaux appartenant à la pile** (les canaux faits à la main et les canaux verrouillés ne sont jamais touchés).

```
$ osatura clip add demo.json cl1.json lib.json wave w1 --at 6 --speed 1 --loops 2 --mode additive --weight 0.8 --fade-in 2 --include 'arm_*' --rename arm_up=hand
added clip 'w1' (1 clips); run clip bake to generate channels
$ osatura clip list cl1.json
1. w1 take 'wave' frames 6..30 x1 speed 1 Replace weight 1
0 baked channels
$ osatura clip set-weight cl1.json cl2.json w1 0.5
updated clip 'w1'; run clip bake to update channels
$ osatura clip set-weight cl1.json cl3.json w1 --keys 0:0,12:1
$ osatura clip bake cl2.json cl5.json
baked 1 clips into 4 channels (0 replaced); wrote cl5.json
$ osatura clip remove cl2.json cl6.json w1
removed clip 'w1'; run clip bake to update channels
```
(L'exemple de `clip list` montre le second `clip add` portant le même nom, qui a remplacé le premier.) **Limites (ROADMAP §9) :** les canaux figés sont échantillonnés à chaque image entière ; il n'existe pas de `clip refresh` pour récupérer à nouveau un take de bibliothèque modifié.

### `make-loop`

**Synopsis :** `osatura make-loop <project.json> <out.json> [targets|all] [props] [--inclusive]`

Fait terminer les canaux `replace` animés à la main là où ils ont commencé : chaque courbe est échantillonnée image par image, la droite reliant sa première à sa dernière valeur est soustraite (suppression de la dérive) et la première valeur est rajoutée. Appliquée au décalage X d'un os en déplacement, elle transforme une marche en marche sur place. La période de boucle est N = les `frames` du projet, de sorte que lire `0..N-1` en boucle ne montre aucune image de raccord dupliquée. `--inclusive` est la convention héritée où la dernière image est égale à la première. Sans argument, tous les canaux animés par clés sont traités. Affiche une ligne `removed drift` par canal modifié.

```
$ osatura make-loop demo.json ml2.json foot_ctrl bone_offset_x
foot_ctrl BoneOffsetX: removed drift 0.79
wrote ml2.json
$ osatura make-loop demo.json ml.json all bone_angle --inclusive
```
**Limites :** détendance linéaire classique, et non un re-timing du mouvement ; les canaux pilotés par expression ne sont pas touchés. **Le déplacement est supprimé volontairement** (un projet lu en `0..N-1` en boucle ne peut pas se déplacer indéfiniment, la boucle est donc sur place) : un canal dont toute la variation était du déplacement devient constant, et la commande affiche pour lui une ligne `note: ... is now constant` ; `lint` signale alors ce canal en L005 (information), ce qui est attendu. Nommez les cibles et les propriétés pour épargner le déplacement des autres canaux, ou utilisez `loop-solve` pour un mouvement déjà cyclique.

### `loop-solve`

**Synopsis :** `osatura loop-solve <project.json> <out.json> [--window 24] [--inclusive]`

Résout le **raccord** : fond la fin des canaux animés par clés dans leur début sur une fenêtre, de sorte que l'animation boucle sans à-coup. Affiche l'erreur de raccord avant et après.

```
$ osatura loop-solve demo.json ls.json --window 24
wrote ls.json: 8 channel(s) adjusted, seam 0.000 -> 0.000
```
`--window` est exprimé en images (24 par défaut). À comparer avec le contrat `loop` de `lint` (`contracts`).

### `walk`

**Synopsis :** `osatura walk <project.json> <out.json> --thigh-l B --thigh-r B [--shin-l B --shin-r B --arm-l B --arm-r B --hips B --period N --stride DEG --knee DEG --bob PX --swing DEG]`

Génère un cycle de marche en boucle sous forme de canaux animés par clés sur les os que vous nommez (au moins une option d'os est requise : `name at least one bone (--thigh-l, --thigh-r, ...)`). Valeurs par défaut : `--period 24` images, `--stride 28`, `--knee 40`, `--bob 4`, `--swing 22`. Les canaux utilisent l'extrapolation en boucle et sont **ajoutés**.

```
$ osatura walk small.json wk.json --thigh-l arm_up --thigh-r arm_low --period 8 --stride 20
wrote wk.json (+2 channels)
```

### `mirror`

**Synopsis :** `osatura mirror <project.json> <out.json>`

Reflète tous les canaux animés par clés par rapport à l'axe vertical : les cibles échangent leur côté (préfixes `L_`/`R_`, suffixes `_L`/`_R` et `_l`/`_r`, mots `left`/`right`), et les rotations et translations horizontales changent de signe. Les canaux réfléchis **remplacent** les canaux du projet (ils ne sont pas ajoutés à côté des originaux). La ligne de résultat est `wrote X (N of M channel target(s) swapped sides)` ; lorsqu'aucune cible n'a de nom gauche/droite, ou que le projet n'a aucun canal, un avertissement sur stderr le signale (seuls les signes sont inversés).

### `foot-lock`, `stretch`, `magnet`

```
osatura foot-lock <project.json> <out.json> <ctrl_bone> <ground_y> [tolerance] [min_frames]
osatura stretch   <project.json> <out.json> <follower_bone> <leader_bone> [min=1] [max=3]
osatura magnet    <project.json> <out.json> <ctrl_bone> <bone:NAME|x,y> <radius> <strength 0..1>
```

* **`foot-lock`** échantillonne la position monde du contrôleur à chaque image ; les séquences d'au moins `min_frames` images où il est au sol (l'axe y de la scène pointe vers le bas : le pied est posé lorsque `y >= ground_y - tolerance`) sont des *contacts*, et un canal de correction **additif** fixe le contrôleur là où chaque contact a commencé. L'animation d'origine reste intacte ; supprimez les canaux ajoutés pour annuler.
  ```
  $ osatura foot-lock demo.json fl.json foot_ctrl 385 2 2
  contact frames 23..47 pinned at (479.2, 384.1)
  wrote fl.json
  ```
* **`stretch`** fait viser à `follower_bone` la tête de `leader_bone` et le met à l'échelle (uniformément, entre `min` et `max`, valant 1 et 3 par défaut) pour que sa pointe l'atteigne. Figé sous la forme d'un canal d'angle additif plus un canal d'échelle multiplicatif ; les enfants du suiveur sont mis à l'échelle avec lui. Le meneur ne peut pas dépendre du suiveur (`the leader depends on the follower (it is one of its descendants)`).
  ```
  $ osatura stretch p.json st.json flw lead 1 2
  corrected 8 frames
  wrote st.json
  ```
* **`magnet`** fige l'attraction de proximité d'un contrôleur vers la tête d'un os (suivie image par image) ou vers un point fixe : atténuation douce, `strength` complet (0..1) sur la cible et nul à `radius`. Figé sous forme de canaux de décalage additifs.
  ```
  $ osatura magnet p.json mg.json lead 200,100 200 0.8
  pulled on 8 frames
  wrote mg.json
  ```
**Limites (ROADMAP §9) :** les trois produisent des résultats figés issus d'un échantillonnage image par image : relancez-les après avoir modifié l'animation source. `stretch` utilise une échelle uniforme. `foot-lock` et `magnet` attendent un os contrôleur racine.

### `shot`, `camera-seq`

```
osatura shot       <project.json> <out.json> <name> <x> <y> <zoom> [rotation]
osatura camera-seq <project.json> <out.json> <frame:shot[:cut],...>
```
`shot` enregistre un **plan de caméra** nommé (centre x, y, zoom, rotation facultative en degrés). `camera-seq` anime la caméra par clés à travers les plans enregistrés : à chaque `frame`, la caméra se déplace vers le plan (avec adoucissement) ou, avec le suffixe `:cut`, y saute. Écrit les canaux `camera_x/y/zoom/rotation`.

```
$ osatura shot small.json sh.json wide 320 240 1 15
wrote sh.json (1 shots)
$ osatura shot sh.json sh2.json close 400 200 2
wrote sh2.json (2 shots)
$ osatura camera-seq sh2.json cs.json 0:wide,4:close:cut
wrote cs.json
```

### `spring-fit`

**Synopsis :** `osatura spring-fit --start V --samples a,b,c,... [--fps 24]`

Ajuste la **fréquence (Hz) et le taux d'amortissement** d'un ressort à une courbe de réponse dessinée (échantillons par image d'un échelon de `--start` jusqu'à la dernière valeur). Affiche le couple ajusté et le JSON à coller dans les `springs` d'un projet. Ne lit pas de projet.

```
$ osatura spring-fit --start 0 --samples 0.5,0.9,1.1,1.0,0.98,1.0 --fps 24
frequency 4.68 Hz, damping 0.61 (rms error 0.028 of the step size)
add to the project: {"bone": "<bone>", "frequency": 4.68, "damping": 0.61}
```

### `script`

**Synopsis :** `osatura script <project.json> <out.json> <script.rhai>`

Exécute un **script Rhai isolé dans un bac à sable** qui modifie le projet par des requêtes validées (une requête rejetée laisse tout inchangé :
`request 5 (box.LayerOpacity at 0) was rejected, nothing changed: ...`). Fonctions intégrées : `fps() frames() bones() layers() set_key(target, prop, frame, value[, ease]) add_bone(name, parent (empty = root), length, x, y, angle) add_rect add_circle add_polygon|add_blob add_line add_gradient_rect add_ik add_aim add_copy_rotation add_pin add_group(name[, [layers]]) add_switch move_layer set_parent delete_layer delete_bone set_layer set_bone add_channel_expr add_noise add_marker add_event bone_info layer_info world print`.
Un script qui n'applique aucune requête affiche `warning: script: 0 requests were applied`. Les builds sans la fonctionnalité `scripting` répondent `this build has no scripting support`.

```
$ cat t.rhai
add_rect("box", 10.0, 10.0, 50.0, 50.0, "#ff0000");
add_rect("box2", 80.0, 10.0, 50.0, 50.0, "#00ff00");
add_group("skins", ["box", "box2"]);
set_key("skins/box", "layer_opacity", 0, 1.0);
print(fps());
$ osatura script small.json sc.json t.rhai
24.0
4 request(s) applied; wrote sc.json
```
Les calques à l'intérieur d'un groupe sont désignés par chemin (`skins/box`). `print` écrit sur stdout.

---

## 7. Physique, tissu et déformateurs

Toutes ces commandes **figent** une simulation ou un résultat géométrique dans des canaux ordinaires animés par clés : le projet reste déterministe et le résultat se lit partout (y compris dans Lottie/GLB lorsque c'est applicable). Relancez la commande après avoir modifié l'animation source.
Elles prennent `<project.json> <out.json>` et des mots `key=value` (sans `--`).

### `physics`

**Synopsis :**
```
osatura physics <project.json> <out.json> <bone,bone,...> [gravity=900] [wind=x,y] [gust=0.5] [damping=0.985]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [follow=bone] [--allow-empty]
    [drive=bone:gain;...] [vortex=bone|x,y:swirl[:radial[:radius]];...] [preset=hair|tail|rope|antenna|cape]
    [merge=keep|replace] [lateral=0..1]
```
Fige une **simulation de chaîne** de Verlet (cheveux, queue, corde, antenne, cape) sur les os listés (racine de la chaîne en premier) sous forme de clés `bone_angle`, une par image.

| Mot | Signification |
|---|---|
| `gravity=` | Accélération vers le bas (900 unités de scène/s^2 par défaut). |
| `wind=x,y`, `gust=`, | Vecteur de vent constant ; amplitude des rafales d'un vent modulé par du bruit. |
| `damping=` | Amortissement de la vitesse par pas (0.985 par défaut). |
| `circle=x,y,r[@bone];...`, `capsule=x1,y1,x2,y2,r[@bone];...` | Colliders, éventuellement attachés à un os (ils le suivent). Les capsules sont acceptées (vérifié avec `capsule=100,300,300,300,20` ; une ancienne note « capsule colliders NOT done » dans ROADMAP §9 est obsolète). |
| `follow=bone` | La racine de la chaîne est entraînée par le mouvement de cet os (inertie). |
| `drive=bone:gain;...` | Corps qui poussent la chaîne (leur vitesse multipliée par le gain). |
| `vortex=bone\|x,y:swirl[:radial[:radius]];...` | Champs tourbillonnaires autour d'un os ou d'un point. |
| `preset=` | Valeurs de départ pour `hair tail rope antenna cape` ; les mots écrits **après** `preset=` les remplacent. |
| `merge=keep\|replace` | `keep` (par défaut) laisse intacts les os qui ont déjà une animation ; `replace` les écrase. |
| `lateral=0..1` | Avec plusieurs chaînes (`bone,bone/bone,bone`), relie entre eux les brins voisins. |
| `--allow-empty` | Accepte une exécution qui n'a rien figé. |

```
$ osatura physics p.json ph.json tail1,tail2,tail3 gravity=500 preset=tail
baked 3 bones; wrote ph.json
$ osatura physics p.json ph4.json tail1,tail2/tail3 lateral=0.5            # two strands
baked 3 bones; wrote ph4.json
$ osatura physics p.json ph3.json tail1,tail2,tail3 follow=arm_up circle=300,300,40 wind=10,0 gust=0.5 lateral=0.3 drive=lead:0.5
baked 3 bones; wrote ph3.json
$ osatura physics demo.json ph.json arm_low,hand gravity=500 preset=tail   # these bones are keyed already
kept existing animation on 'arm_low' (use merge=replace to overwrite)
kept existing animation on 'hand' (use merge=replace to overwrite)
physics: nothing baked - 2 bone(s) already have animation and merge=keep (the default) leaves it alone; nothing was written. Use merge=replace to overwrite the existing animation, or --allow-empty to accept an unchanged file     [exit 1]
$ osatura physics demo.json ph.json hand preset=bogus
unknown preset 'bogus' (try hair, tail, rope, antenna, cape)                [exit 1]
```
(`p.json` est la démo plus trois os sans clés `tail1..tail3` créés avec un script Rhai - voir `script`.) **Piège :** un bake qui ne change rien est une erreur, sauf avec `--allow-empty`.
**Limites (ROADMAP §9) :** les colliders ignorent l'échelle des os (transformation rigide uniquement) ; les cercles statiques ne collisionnent qu'en des points (les cercles attachés aussi le long des liens) ; pas de collision capsule-capsule ni d'auto-collision.

### `cloth`

**Synopsis :**
```
osatura cloth <project.json> <out.json> <layer> [pin=top,left@bone,row:N,col:N,v:N,corners,none] [gravity=900]
    [wind=x,y] [gust=0.5] [gust-hz=0.5] [gust-len=px] [stiffness=1] [shear=0.05] [bend=0.2] [max-stretch=1.1]
    [damping=0.99] [cols=N rows=N] [rect=x,y,w,h] [name=cloth] [substeps=4] [iterations=10] [preroll=frames]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [from=F] [to=F]
```
Fige une **surface de tissu** déterministe sur le maillage d'un calque *image*, ou sur un calque *vectoriel* lorsque `rect=x,y,w,h` est donné (le calque vectoriel reçoit un maillage en grille sur ce rectangle), sous forme de canaux de déformation par sommet. `pin=` choisit les sommets épinglés (bords `top`, `left`..., `corners`, `row:N`, `col:N`, `v:N`, `@bone` attache les épingles à un os, `none`). `from`/`to` limitent la plage d'images figée, `preroll` exécute la simulation avant l'image `from` pour que le tissu démarre au repos.

```
$ osatura cloth small.json cloth.json sun rect=100,60,120,120 cols=6 rows=6 pin=top gravity=900 wind=40,0
baked cloth on 'sun' (84 channels); wrote cloth.json
$ osatura cloth small.json cloth.json sun
a vector layer needs rect=x,y,w,h                    [exit 1]
$ osatura cloth small.json cloth.json nosuch
no layer at 'nosuch'                                 [exit 1]
```
Les `84 channels` sont des clés `lattice_dx` / `lattice_dy` (cible `sun::cloth::<col>,<row>`) des 42 points non épinglés de la grille 7x7 (la rangée du haut est épinglée). Ce sont de vrais canaux dans le fichier, de sorte qu'un bake de tissu alourdit le projet (98 canaux au lieu de 14 ici) - utilisez `size-report` pour en voir le coût. **Limites (ROADMAP §9) :** le tissu est plan (pas de plis hors du plan, le cisaillement est donc souple par défaut) ; il refuse les maillages libres (maillages en grille uniquement) ; pas d'auto-collision ; figé à chaque image entière.

### `cage`

**Synopsis :** `osatura cage <project.json> <out.json> <layer> [rect=x,y,w,h|auto] [hull] [verts=N] [pad=P] [name=cage]`

Ajoute autour d'un calque un **déformateur de cage à coordonnées de valeur moyenne** : le contenu situé à l'intérieur de la cage suit les sommets de la cage, le contenu extérieur est inchangé. `auto` (par défaut en l'absence de `rect`) utilise les bornes avec marge, `verts=N` est le nombre de sommets par côté, `hull` utilise l'enveloppe convexe, `pad=` ajoute une marge à la boîte, `name=` nomme le déformateur (`cage` par défaut).

```
$ osatura cage small.json cg.json sun auto verts=3
added cage 'cage' (12 vertices) to 'sun'; wrote cg.json
```
**Limites (ROADMAP §9) :** pas d'ajustement automatique de la cage au-delà d'une boîte avec marge ou de l'enveloppe convexe (pas de tracé concave) ; il n'y a pas d'atténuation douce à l'extérieur de la cage (une marche visible apparaît là où le dessin traverse le bord de la cage) ; les maillages libres n'ont pas de décalages par sommet animés par clés (utilisez une cage).

### `cage-pose`

**Synopsis :** `osatura cage-pose <project.json> <out.json> <layer> <frame> [name=cage] [move=i:dx,dy;...] [all=dx,dy] [ease=linear]`

Pose des clés de décalage des sommets de la cage à une image (canaux `cage_dx` / `cage_dy`). `move=` décale des sommets individuels, `all=` décale tous les sommets. Les canaux verrouillés sont refusés.

```
$ osatura cage-pose cg.json cp.json sun 4 all=5,0
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
$ osatura cage-pose cg.json cp.json sun 4 move=0:5,5
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
```

### `warp-pivot` et `soft-scale`

**Synopsis :**
```
osatura warp-pivot <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=1,4-7] [region=x,y,w,h]
    [pivot=x,y|bone:NAME|centroid|center] [rotate=DEG] [scale=S|sx,sy] [move=dx,dy] [radius=R] [falloff=smooth] [ease=linear]
osatura soft-scale <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=..|region=..] scale=S|sx,sy
    [radius=R] [falloff=smooth] [pivot=..]
```
`warp-pivot` fait pivoter, met à l'échelle ou déplace une **sélection** de points de contrôle (déformateurs cage, lattice, cloth, pin, spine ; un maillage image en grille reçoit une grille de déformation par sommet) autour d'un pivot et pose une clé du résultat à l'image donnée. La sélection est `verts=` (indices et plages) et/ou `region=` ; une sélection vide est une erreur (`the selection is empty (give verts= and/or region=)`). `radius=` ajoute une **atténuation douce** (`linear smooth gauss sharp constant`). `soft-scale` est une mise à l'échelle proportionnelle (sélection douce) ; sans `verts`/`region`, tout le maillage est redimensionné autour du centre de sa boîte englobante par `scale=F`, `width=W`, `height=H` ou `size=WxH`, avec `aspect=keep|free`.

```
$ osatura warp-pivot cg.json wp.json sun 4 verts=0-3 rotate=15 pivot=centroid radius=30 falloff=gauss
warp-pivot on 'sun' keyed at frame 4; wrote wp.json
$ osatura soft-scale cg.json ss.json sun 4 scale=1.2 pivot=center
soft-scale on 'sun' keyed at frame 4; wrote ss.json
```

### `layer-pivot`

**Synopsis :** `osatura layer-pivot <project.json> <out.json> <layer> pivot=x,y|bone:NAME`

Déplace le pivot (ancre) d'un calque **sans changer l'image**, de sorte que les canaux de rotation et d'échelle tournent autour du nouveau point.

```
$ osatura layer-pivot small.json lp.json sun pivot=100,100
moved the pivot of 'sun'; wrote lp.json
```

### `mesh-lock`

**Synopsis :** `osatura mesh-lock add <project.json> <out.json> <name> host=LAYER[:path] follower=LAYER[:path] [verts=1,4-7] [strength=1]`
et `osatura mesh-lock list|remove <project.json> [<out.json> <name>]`

Fait suivre à chaque image aux sommets du suiveur la **surface du maillage hôte** (transfert de déplacement barycentrique), par ex. une chemise qui suit un corps. Les cycles sont refusés ; l'hôte et le suiveur doivent avoir le même `parallax` (`mesh lock 'lock1': host and follower must have the same parallax`).

```
$ osatura mesh-lock add two.json ml1.json lock1 host=ra follower=rb verts=0-1 strength=0.5
added mesh lock 'lock1'; wrote ml1.json
$ osatura mesh-lock list ml1.json
lock1: rb:0 follows ra:0 vertices [0, 1] strength 0.5
1 mesh lock(s)
$ osatura mesh-lock remove ml1.json ml2.json lock1
removed mesh lock 'lock1'; wrote ml2.json
```

### `glue`

**Synopsis :** `osatura glue add <project.json> <out.json> <name> a=LAYER[:path] b=LAYER[:path] pairs=i:j;i:j|auto=RADIUS [mode=rigid|snap|offset] [strength=1]`
et `osatura glue list|remove <project.json> [<out.json> <name>]`

Colle des sommets du calque `b` à des sommets du calque `a` (b suit a). `pairs=i:j;...` liste explicitement des paires de sommets, `auto=RADIUS` apparie chaque sommet de `b` situé à l'intérieur du rayon d'un sommet de `a`. Modes : `rigid` conserve le décalage de repos dans le repère de `a`, `snap` les fait coïncider, `offset` conserve un décalage constant.

```
$ osatura glue add two.json gl1.json g1 a=ra b=rb auto=5 mode=snap
added glue 'g1' (4 pair(s)); wrote gl1.json
$ osatura glue list gl1.json
g1: Snap strength 1 ra:0#0->rb:0#0 ra:0#1->rb:0#1 ra:0#2->rb:0#2 ra:0#3->rb:0#3
1 glue(s)
$ osatura glue add small.json x.json g1 a=sun b=hills auto=20 mode=snap
no vertex of 'hills' lies within 20 of a vertex of 'sun'          [exit 1]
```

### `mesh-trace`

**Synopsis :** `osatura mesh-trace <project.json> <out.json> <layer> [density=24] [tolerance=1.5] [smooth=N] [hotspot=x,y,r;...] [edge=D/4] [grading=0.5] [lambda=0.5] [taubin=1] [smooth-boundary=0] [threshold=128] [bones=R] [hot-size=E] [skin=1]`

Trace la **silhouette alpha d'un calque image** en un maillage triangulaire libre de densité variable (plus dense le long du contour, autour des points chauds et des os), le lisse, et le skinne sur les os. Affiche le nombre de sommets et de triangles ainsi que la qualité.

```
$ osatura mesh-trace o.json mt.json top density=12 tolerance=2 bones=0
wrote mt.json: 9589 vertices (1494 on the outline), 17682 triangles, area 1227677 px2
  quality min 0.484 mean 0.890, min angle 18.3 deg, mean edge 3.0 px at the outline / 13.7 px inside
```
(`o.json` est un import OpenRaster de 1280x960 ; une `density` plus basse donne moins de triangles, plus grands. L'exécution a pris 5 s dans un build debug.) **Limites :** nécessite un calque image (une silhouette alpha) ; un maillage aussi dense alourdit le fichier de projet - vérifiez avec `size-report`. Les maillages libres ne peuvent pas être simulés en tissu.

### `skin-diffuse`

**Synopsis :** `osatura skin-diffuse <project.json> <layer> <out.json> [--influences 4]`

Calcule des **poids de skin par diffusion de chaleur** pour un calque maillé à partir des os du projet, en conservant au plus `--influences` (4 par défaut) os par sommet. Notez l'ordre des arguments : **le calque avant la sortie**. Le calque doit être un calque image avec un maillage.

```
$ osatura skin-diffuse demo.json arm sd.json
layer 'arm' is not an image layer                          [exit 1]
$ osatura skin-diffuse o.json top sd.json
layer 'top' has no mesh                                    [exit 1]
$ osatura skin-diffuse mt.json top sd.json               # mt.json from mesh-trace
wrote sd.json
```

---

## 8. Audio, synchronisation labiale et sons

Les entrées audio des commandes d'analyse (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) doivent être au format **WAV** (PCM entier 8/16/24/32 bits ou flottant 32 bits, tout nombre de canaux mixé en mono, jusqu'à 256 Mio). Le MP3/OGG doit d'abord être converti (`ffmpeg -i in.mp3 in.wav`). Les **pistes** de son stockées dans un projet (`sound`) peuvent être dans tout format que ffmpeg sait lire.

### `lipsync`

**Synopsis :** `osatura lipsync <project.json> <audio.wav> <switch_layer> <out.json> [--map 0,1,2,3,4]`

Pilote un **calque switch** (formes de bouche) à partir de l'intensité sonore et de la brillance spectrale : une heuristique d'amplitude / de passages par zéro qui choisit l'une des cinq formes par image (`rest, small, open, round, wide`) et supprime le scintillement. **Ce n'est pas de la reconnaissance de phonèmes.** `--map` donne l'indice de l'enfant du switch pour chacune des cinq formes ; la valeur par défaut dépend du nombre d'enfants du switch : 1 enfant -> toutes ; 2 -> fermé + ouvert ; 3 -> fermé + ouvert + rond/large ; 4 -> fermé + petit + ouvert + rond/large ; 5+ -> dans l'ordre ci-dessus. Remplace un canal `layer_switch` animé par clés existant de ce calque. La longueur du projet n'est **pas** modifiée (un WAV de 2 s sur un projet de 8 images pose des clés sur 8 images).

```
$ osatura lipsync small.json speech.wav mouth lip.json
wrote lip.json (2.00s of audio → 8 frames)
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1,2,1,0
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1
--map needs exactly 5 numbers (rest,small,open,round,wide)          [exit 1]
$ osatura lipsync small.json speech.wav ink lip.json
layer 'ink' is not a switch layer                                   [exit 1]
$ osatura lipsync small.json notwav.mp3 mouth x.json
invalid WAV: Failed to read enough bytes.                           [exit 1]
```

### `lipsync-text`

**Synopsis :** `osatura lipsync-text <project.json> <audio.wav> <transcript.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--lang ro|en] [--words-out timing.json|timing.srt]`

Aligne une **transcription** sur l'audio (énergie + alignement du texte : les mots et les syllabes sont posés sur les zones de parole trouvées dans l'audio) et déduit les formes de bouche à partir des syllabes. `--lang` (`ro` par défaut) sélectionne les règles de voyelles/syllabes ; seuls `ro` et `en` existent. `--words-out` écrit les minutages des mots en JSON (mots avec syllabes) ou, pour un nom en `.srt`, sous forme de sous-titres.

```
$ osatura lipsync-text small.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (3 words, 2 exposure changes; energy + text alignment, not phoneme recognition)
$ head -c 140 words.json
{ "words": [ { "text": "hello", "start": 0.0, "end": 0.22, "syllables": [ {"text":"hel","start":0.0,"end":0.16}, {"text":"lo",...
$ osatura lipsync-text ... --words-out w.srt ; head -4 w.srt
1
00:00:00,000 --> 00:00:01,220
hello there world
```
**Limites (ROADMAP §9) :** pas de minutage par lettre (au niveau du phonème), pas de modèles de débit propres à un locuteur, pas d'autre langue que ro/en. Une tonalité synthétique est alignée comme de la parole - utilisez de vrais enregistrements.

### `speech-motion`

**Synopsis :** `osatura speech-motion <project.json> <audio.wav> <transcript.txt> <out.json> [--head bone] [--brow bone_or_layer] [--nod deg] [--fall deg] [--tilt deg] [--brow-px px] [--seed n] [--lang ro|en] [--words-out timing.json|timing.srt]`

Ajoute des hochements, chutes, inclinaisons de la tête et haussements de sourcils **additifs**, minutés sur les mots alignés. `--head` nomme l'os de la tête, `--brow` un os ou un calque pour les sourcils ; les options d'angle sont en degrés, `--brow-px` en pixels, `--seed` fixe la variation.

```
$ osatura speech-motion small.json speech.wav transcript.txt sm.json --head arm_up --lang en
wrote sm.json (1 additive channel(s) from 3 words)
```

### `mouth`

**Synopsis :** `osatura mouth <project.json> <cues.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--rhubarb]`

Importe des **repères** de bouche tout faits au lieu d'analyser l'audio : fichiers Papagayo/Moho-switch (`frame PHONEME` par ligne, première ligne facultative `MohoSwitch1`) ou TSV Rhubarb (`seconds<TAB>shape`, reconnu à la virgule décimale ; les secondes sont converties à la cadence du projet). Les lignes vides et les commentaires `#` sont ignorés. Les phonèmes sont associés aux cinq classes de formes (`rest/MBP/X/A` repos ; `etc/FV/L/B/G/H` petit ; `AI/D` ouvert ; `O/U/WQ/F` rond ; `E/C` large) ; les formes inconnues comptent comme `small`. `--rhubarb` traite le `E` de Rhubarb comme *rond* (le `E` de Papagayo est large). La valeur par défaut de `--map` est `0,1,2,3,4`.

```
$ printf '0 rest\n2 open\n4 round\n6 wide\n' > cues.txt
$ osatura mouth small.json cues.txt mouth mo.json --rhubarb
wrote mo.json (4 cues)
```

### `audio-react`

**Synopsis :** `osatura audio-react <project.json> <out.json> <audio.wav> <target> <prop> [gain=1] [offset=0] [floor=0.05] [attack=1] [release=4] [add=on]`

L'intensité sonore pilote une propriété : une clé par image suivant le niveau lissé (RMS relatif à l'image la plus forte). Sortie = (niveau filtré par `floor`) x `gain` + `offset` ; `attack`/`release` sont les temps de montée/descente en images ; `add=on` rend le canal additif au lieu de remplaçant. Un canal verrouillé est refusé. Les options sont des mots `key=value` ; une option inconnue donne `unknown option 'foo'`.

```
$ osatura audio-react small.json ar.json speech.wav ink layer_rotation gain=10
wrote ar.json
$ osatura audio-react small.json ar.json speech.wav arm_up bone_angle foo=1
unknown option 'foo'                                                [exit 1]
```

### `beats`

**Synopsis :** `osatura beats <project.json> <out.json> <audio.wav> [sensitivity=1.5] [gap=4] [as=markers|events] [name=beat]`

Détecte les sauts d'intensité au-dessus de la moyenne récente (`sensitivity` 1.2-3, plus élevée = moins de temps forts) espacés d'au moins `gap` images, et les enregistre comme **marqueurs** de timeline (par défaut) ou comme **événements** nommés `name`. Affiche les images trouvées.

```
$ osatura beats small.json bt.json speech.wav as=events name=hit sensitivity=1.2
1 beats at frames [1]; wrote bt.json
$ osatura beats small.json bt.json speech.wav
0 beats at frames []; wrote bt.json
```

### `waveform`

**Synopsis :** `osatura waveform <audio.wav> <out.json> [buckets=800]`

Pics min/max pour dessiner une forme d'onde : `{"sample_rate","duration","peaks":[[min,max],...]}`. Ne lit pas de projet.

```
$ osatura waveform speech.wav wf.json buckets=50
wrote wf.json (50 buckets)
```

### `sound`

**Synopsis :**
```
osatura sound list   <project.json>
osatura sound add    <project.json> <out.json> <name> <file.wav|ogg|mp3|...> [start_frame] [gain] [trim/fade options]
osatura sound edit   <project.json> <out.json> <name> [--start F] [--gain G] [trim/fade options]
osatura sound remove <project.json> <out.json> <name>
osatura sound mix    <project.json> <out.wav|out.ogg|out.mp3>
    trim/fade options (seconds, or frames with an f suffix such as 36f): --trim-in T --trim-out T --fade-in T --fade-out T
```
Les pistes de son sont des **références** (chemin du fichier relatif au projet, image de départ, gain 0..4, fenêtre de rognage dans le fichier, fondus). Rien n'est intégré au projet ; le fichier audio doit voyager avec lui, et doit se trouver dans le répertoire du projet, sauf si `--allow-external-assets` est donné. `edit` ne modifie que les champs indiqués (`0` supprime un rognage). `--trim-out 0` signifie « jusqu'à la fin ». `sound mix` rend uniquement l'audio mixé pour la **durée du projet** (via ffmpeg : wav = PCM 16 bits 44,1 kHz, ogg = Vorbis q4, mp3 = 96 kb/s).
L'audio mixé est aussi multiplexé dans `.mp4` (AAC) et `.webm` (Opus) et intégré dans `.html` ; `.gif`/`.apng` ne peuvent pas le porter.

```
$ osatura sound add small.json s1.json music speech.wav 2 0.8 --fade-in 0.5 --trim-out 36f
wrote s1.json
$ osatura sound list s1.json
music: speech.wav at frame 2 gain 0.8 trim 0s..1.5s fade-in 0.5s
$ osatura sound edit s1.json s2.json music --gain 0.5 --trim-in 0.1 --fade-out 4f
$ osatura sound list s2.json
music: speech.wav at frame 2 gain 0.5 trim 0.1s..1.5s fade-in 0.5s fade-out 0.16666667s
$ osatura sound mix s2.json mix.mp3
wrote mix.mp3 (0.33 s)
$ osatura sound mix s2.json mix.flac
unknown audio format '.flac' (use .wav, .ogg or .mp3)            [exit 1]
$ osatura sound add small.json x.json x nosuch.wav              # a missing file is refused, nothing written
```
`check` échoue lorsqu'un fichier de son référencé est manquant ou illisible. Sans ffprobe, seule l'existence du fichier (et non le fait qu'il s'agisse d'audio) est vérifiée pour les fichiers non WAV. **Limites (ROADMAP §9) :** pas de panoramique/EQ/ducking par piste, uniquement la courbe de fondu linéaire, pas de bouclage d'un clip, pas de re-timing.

---

## 9. Import

### `import-svg`

**Synopsis :** `osatura import-svg <file.svg> <out.json> [--split]`

Ligne d'usage : `osatura import-svg <file.svg> <out.json> [--split]  SVG subset -> one vector layer, or one layer per shape named by its id (warnings for the rest)`

Crée un nouveau projet (24 fps, 24 images ; dimensions tirées de `viewBox` ou de `width`/`height`, sinon 512x512) avec **un calque vectoriel `svg`** contenant les formes d'un sous-ensemble de SVG : `path` (toutes les commandes), `rect` (coins arrondis ignorés), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` avec `transform`, ainsi que `fill`/`stroke`/`stroke-width` unis. Tout le reste est ignoré avec une ligne `warning:` **sur stdout** (lorsque l'élément est hors de `<defs>` ; un `<filter>`, `<clipPath>`, `<mask>` ou `<pattern>` placé dans `<defs>` ainsi que les attributs `filter`, `clip-path` et `mask` sont ignorés sans avertissement) : `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`/`<radialGradient>`, `<image>`, `<use>`, `<foreignObject>`. Un remplissage en dégradé (`fill="url(#g)"`) se rabat sur du gris. Limites : 8 Mo de SVG, 200 000 ancres, un plafond sur le nombre d'éléments.

```
$ osatura import-svg s.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
warning: <filter> is not supported and was skipped
warning: <image> is not supported and was skipped
wrote svg.json (3 paths)
$ osatura check svg.json
ok: 's' 200×100 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 0 channels
```
Avec `--split`, chaque forme devient **son propre calque**, nommé d'après l'`id` de la forme, dans l'ordre du fichier. Une forme sans id est nommée d'après sa balise et sa position (`rect1`, `circle6`), un id répété reçoit `_1`, `_2`..., et les caractères qu'un nom de calque ne peut pas contenir (`/`, `::`, caractères de contrôle) deviennent `_` ; les noms sont tronqués à 100 caractères. C'est la forme à utiliser lorsque le dessin va être rigué partie par partie ; voir [TUTORIAL_SVG.md](TUTORIAL_SVG.md).

```
$ osatura import-svg pip.svg pip.json --split
wrote pip.json (16 paths, 16 layers)
```
Importer puis fusionner : aucune commande ne fusionne le calque dans un projet existant ; utilisez `script` ou modifiez le JSON.

### `import-ora`

**Synopsis :** `osatura import-ora <file.ora> <out.json> [--fps N] [--frames N]`

OpenRaster -> projet : un **calque image par calque raster** (PNG intégré), ordre de bas en haut conservé, calques masqués conservés mais invisibles, groupes aplatis, `opacity`, `visibility` et position (`x`,`y`) par calque, ainsi que les opérations de composition `svg:` usuelles (multiply, screen, overlay, darken, lighten, color-dodge/burn, hard/soft-light, difference, exclusion, plus -> add). Le projet compte `--frames` images (1 par défaut) à `--fps` (24 par défaut). L'opacité d'une pile imbriquée se multiplie dans ses calques et son mode de fusion est hérité, mais les calques qui se chevauchent à l'intérieur d'une pile translucide ne sont **pas** aplatis au préalable (ils se voient à travers l'un l'autre). Les noms de calques en double reçoivent des suffixes `_1`, `_2`.

```
$ osatura import-ora t.ora ora.json
wrote ora.json (2 layers)
$ osatura import-ora t.ora ora2.json --fps 12 --frames 24
$ osatura import-ora layer.png bad.json
not a zip/ORA file: invalid Zip archive: Could not find EOCD       [exit 1]
```
Limites : 2000 entrées d'archive, 128 Mio par entrée, calques de 64 Mpixels.

### `import-lottie`

**Synopsis :** `osatura import-lottie <in.json> <out.json> [fx.json]`

Importe un fichier Lottie (Bodymovin) - un **sous-ensemble** délibéré - sous forme de projet. Affiche `imported N layers; wrote <out>` et une ligne `warning:` par fonctionnalité ignorée.

* **Importé :** calques de formes avec tracés `sh`, `rc`, `el`, remplissages unis et en dégradé (les points d'extrémité, le centre et le rayon animés d'un dégradé deviennent des canaux `grad_dx/grad_dy`), contours, groupes, transformations de calque (position/échelle/rotation/opacité statiques ou en keyframes, trajectoires de position courbes précalculées image par image), modes de fusion, un calque de fond uni, calques image avec URI de données PNG/JPEG intégrées, calques nuls comme parents (les transformations des parents sont précalculées dans les enfants), masques de calque (add/subtract/intersect avec opacité, inversion, formes animées par clés), mattes de luminance (approximées), précompositions, plages d'entrée/sortie de calque (sous forme de piste d'opacité en escalier), interpolation des keyframes (hold -> step ; poignées de Bézier -> cubic-bezier ; sans poignées -> linéaire).
* **Tracés animés :** un tracé dont les clés de forme ont la même topologie (même nombre d'ancres, ouvert/fermé) devient un tracé de base plus une **cible de morphing** par clé distincte supplémentaire (`k1`, `k2`, ...), avec des canaux de poids `path_morph` qui portent l'interpolation. Topologie différente : la première forme est utilisée et un avertissement est affiché.
* **Effets (`ef`) :** le flou gaussien, l'ombre portée, la teinte et le remplissage deviennent des **effets par calque** (`effects` sur le calque, de n'importe quel type) avec des nombres animés sous forme de canaux `fx_*` ; la teinte, le remplissage, le tritone et les niveaux statiques sur les calques de formes sont précalculés dans les couleurs. Les couleurs d'effet ne peuvent pas être animées (la première valeur est utilisée) ; les autres effets, ainsi que les effets de couleur placés après un flou/une ombre sur des calques qui ne sont pas des calques de formes, donnent lieu à des avertissements. Seuls les premiers effets de chaque calque sont conservés.
* **Ignoré avec un avertissement :** texte et autres types de calques, ressources audio, points d'ancrage animés (première valeur), trajectoires de mouvement courbes sur des propriétés autres que la position (segments droits), masques sur une source de track matte, chaînes de parents trop longues pour être précalculées, plus de 2000 calques, animation de plus de 20 000 images.
* `fx.json` est conservé pour la compatibilité : un tableau vide y est écrit (le flou et l'ombre portée sont désormais des effets par calque à l'intérieur du projet).

```
$ osatura import-lottie out2.json li.json                   # a Lottie file made by `osatura export`
imported 11 layers; wrote li.json
$ osatura import-lottie in.json out.json                    # hand-made file: animated triangle with animated blur, plus a null and a text layer
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
$ osatura check out.json
ok: 't' 200×200 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 2 channels       # channels: blob::path::k1 path_morph, blob::fx0 fx_blur
$ osatura import-lottie out2.json li2.json fx.json
wrote 0 post effect(s) to fx.json
imported 11 layers; wrote li2.json
$ echo '{}' > bad.json; osatura import-lottie bad.json o.json
missing 'w'                                                        [exit 1]
```
**Piège :** `import-lottie in.json in.json` est refusé (`refusing to overwrite the input ...`), y compris pour `fx.json`. **Limites :** l'entrée `.lottie` (zip) n'est pas lue (`stream did not contain valid UTF-8`) - décompressez d'abord `animations/animation.json` ; le travail quadratique dans les morphings de formes n'est borné que par les plafonds de taille.

### `import-bvh`

**Synopsis :** `osatura import-bvh <project.json> <file.bvh> <out.json> --joint-map Joint=bone,.. [--axis Zrotation --tol 0.5]`

Associe les canaux de rotation de certaines articulations BVH à des os du rig sous forme de canaux `bone_angle` à clés, échantillonnés à la fréquence d'images et au nombre d'images du projet. `--joint-map` est obligatoire ; `--axis` (par défaut `Zrotation`) choisit le canal de rotation BVH lu (`Zrotation`, `Xrotation`, `Yrotation`) ; `--tol` (par défaut 0,5 degré) simplifie les clés. Pour l'association automatique, le mouvement de la racine, les vues et l'échelle, utilisez `mocap-retarget` (section 10).

```
$ osatura import-bvh demo.json m.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```

### `import-seq`

**Synopsis :** `osatura import-seq <project.json> <out.json> <layer_name> <folder> [hold=1] [loop=on|off] [start=0] [x=0] [y=0]`

Importe une **séquence PNG** (tous les PNG du dossier, dans l'ordre des noms de fichiers) sous forme de calque image par image : un calque de commutation nommé `<layer_name>` avec un calque image par image et un canal en escalier qui affiche chacune pendant `hold` images à partir de l'image `start`, à la position `(x, y)` ; `loop=on` la répète. Toutes les images doivent avoir la même taille (`img.png: size 640x480 differs from the first image (160x120)`).

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20
imported 3 images; wrote is.json
```

### `import-anim`

**Synopsis :** `osatura import-anim <project.json> <out.json> <layer_name> <file.gif|file.apng|file.png> [loop=on|off] [start=0] [x=0] [y=0]`

Importe un **GIF animé ou APNG** sous forme de calque image par image, en utilisant les délais d'image propres au fichier (convertis en images du projet). Un PNG simple donne une seule image.

```
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0
imported 4 frames as layer 'gifl'; wrote ia.json
$ osatura import-anim demo.json ia2.json apngl dof.apng
imported 3 frames as layer 'apngl'; wrote ia2.json
```
Les images deviennent des PNG intégrés : un long GIF produit un projet volumineux.

### `trace`

**Synopsis :** `osatura trace <image.png> <out.json> [--threshold 128 --tolerance 1.0 --min-area 12 --fill #000000]`

Conversion simple bitmap -> vectoriel pour les images avec **transparence** : elle vectorise le **canal alpha** du PNG (marching squares au seuil `--threshold`, 0..255, avec interpolation linéaire des bords), simplifie les contours (Ramer-Douglas-Peucker) à `--tolerance` pixels, écarte les régions inférieures à `--min-area`, détecte les trous par inclusion, et écrit un nouveau projet (nommé `traced`, canevas = taille de l'image) dont les contours sont des tracés remplis de la couleur `--fill`. Une image entièrement opaque donne donc un seul contour autour de toute l'image (comme dans l'exemple). Affiche `wrote <out> (N contours)`. Utilisez `autotrace` pour les images en couleur.

```
$ osatura trace img.png tr.json --threshold 128 --tolerance 1.0 --min-area 12 --fill '#112233'
wrote tr.json (1 contours)
```

### `autotrace`

**Synopsis :** `osatura autotrace <image.png|jpg> <out.json|out.svg> [--mode fill|line] [--colors N=8] [--min-area A=8] [--tol T=1] [--smooth N=3] [--corner DEG=55] [--denoise N=0] [--quant kmeans|mediancut] [--seed S] [--fit bezier|polygon|vw] [--zorder area|luminance] [--threshold T] [--invert] [--prune N=4] [--into project.json] [--name L] [--fit-stage] [--png preview.png]`

Vectorisation bitmap locale - **sans modèle, sans réseau** : quantification des couleurs (`--colors`, k-means ou median cut), extraction des contours, ajustement de courbes de Bézier cubiques de Schneider (`--fit` ; `polygon` et Visvalingam-Whyatt `vw` sont des alternatives), trous par la règle pair-impair ; `--mode line` amincit les traits en lignes médianes (Zhang-Suen, `--prune` supprime les petites ramifications). Le résultat est un nouveau projet, un nouveau `.svg` ou - avec `--into project.json` - une copie d'un projet existant avec un nouveau groupe de calques `--name` (`trace` par défaut) ; `--fit-stage` (avec `--into`) met le nouveau groupe à l'échelle de façon uniforme pour l'ajuster et le centrer sur la scène du projet, `--png` écrit un rendu d'aperçu. La commande affiche des décomptes et l'**erreur mesurée par rapport à la source** (erreur absolue moyenne, RMSE, PSNR, SSIM).

```
$ osatura autotrace img.png at.json --colors 4 --png prev.png
layer group 'trace' added
wrote at.json: 44 regions, 0 strokes, 44 paths, 324 anchors
fidelity vs source: MAE 13.38/255, RMSE 17.59, PSNR 23.2 dB, SSIM 0.960
$ osatura autotrace img.png at.svg --mode line --threshold 100
wrote at.svg: 0 regions, 11 strokes, 11 paths, 47 anchors
fidelity vs source: MAE 52.47/255, RMSE 58.73, PSNR 12.8 dB, SSIM 0.903
$ osatura autotrace img.png at2.json --into d.json --name traced
layer group 'traced' added
wrote at2.json: 51 regions, 0 strokes, 51 paths, 347 anchors
```
**Limites (ROADMAP §9) :** un vectoriseur classique - les photographies deviennent des aplats de couleur ; pas de vectorisation par apprentissage ni sémantique. Ici, `--png` est une option à valeur (nom du fichier d'aperçu), contrairement aux autres options `--png`.

### `autotrace-bench`

**Synopsis :** `osatura autotrace-bench`

Vectorise des formes, dégradés et traits synthétiques intégrés et affiche un tableau de fidélité (MAE, RMSE, PSNR, SSIM, nombre de régions/ancres, millisecondes). Ne prend aucun argument ; en lecture seule.

```
$ osatura autotrace-bench
image          mode  colors  tol     MAE    RMSE   PSNR   SSIM regions strokes  anchors      ms
shapes         fill       6    1    2.19   12.10   26.5  0.975      11       0       33    13.6
...
```

---

## 10. Entrée en direct, MIDI, suivi du visage et capture de mouvement

### `live-map`

**Synopsis :** `osatura live-map <project.json> <events.json|song.mid> <mapping.json> <out.json> [--takes lib.json] [--tol 0.01] [--allow-empty]`

Transforme une **performance enregistrée en direct** (MIDI, souris, manette, touches, signaux du visage) en canaux à clés, événements et takes ordinaires (non destructif, tout ou rien, verrous respectés). L'entrée est soit un fichier MIDI standard `.mid`, soit un JSON d'événements `{"events":[{"source","id","value","time"}]}` (`value` normalisée 0..1, `time` en secondes ; `source` = `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face` ; `id` = MIDI `"<channel 1-16>:<number>"`, bouton de souris `"0"`, manette `"<pad>:<index>"`, touche = code DOM comme `KeyA`).

Le fichier de mapping contient `params` (source/id -> `target`+`prop`, avec `in_min in_max out_min out_max deadzone invert curve smooth additive`) et `triggers` (franchissement de seuil -> action `jump_to_frame`, `play_take`, `set_key`, ...) ; les clés inconnues sont des erreurs. `--takes` est une bibliothèque de takes pour les actions `play_take` (doit exister), `--tol` simplifie les clés.

```
$ cat map.json
{"params":[{"source":"mouse_x","id":"0","target":"arm_up","prop":"bone_angle","out_min":-30,"out_max":30}],
 "triggers":[{"source":"key","id":"KeyA","action":{"type":"jump_to_frame","frame":3}}]}
$ osatura live-map small.json ev.json map.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 1 events (0 after the last frame skipped); wrote out.json
$ osatura live-map small.json song.mid mapm.json outm.json              # MIDI CC 1 on channel 1 -> arm_up
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote outm.json
$ osatura live-map small.json ev.json map.json out.json --takes lib.json    # lib.json missing
cannot read lib.json: No such file or directory (os error 2)             [exit 1]
$ osatura live-map small.json empty.json map.json out.json
note: map 0 (arm_up.BoneAngle): no matching events, nothing keyed
0 events -> +0 channels, ...; wrote out.json
```
**Piège :** si des événements ont été lus mais que **rien n'a été produit** (faute de frappe dans un id, mauvaise source), la commande échoue et n'écrit rien, sauf avec `--allow-empty` ; avec zéro événement en entrée, elle écrit simplement le projet inchangé. Les déclencheurs de saut sont enregistrés comme un événement nommé `jump` avec `data.to` (l'axe du temps n'est pas réécrit).
**Limites (ROADMAP §9) :** pas d'entrée en direct par webcam ni de niveau audio dans le studio via cette commande (`audio-react` fonctionne hors ligne) ; le studio fournit la partie interactive.

### `drive`

**Synopsis :** `osatura drive <project.json> <signals.csv> <maps.json> <out.json> [--tol 0.5]`

Performance -> clés à partir d'un **CSV** : la ligne d'en-tête nomme les signaux, chaque ligne suivante est une image (indice de ligne = image). `maps.json` est un tableau de `{input, target, prop, in_min, in_max, out_min, out_max, deadzone, invert, curve, smooth}` ; `--tol` (0,5 par défaut) simplifie les clés.

```
$ printf 'jaw,speed\n0,0\n0.5,0.2\n1,0.4\n0.5,0.6\n' > sig.csv
$ echo '[{"input":"jaw","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":40,"smooth":0.2}]' > maps.json
$ osatura drive demo.json sig.csv maps.json dr.json --tol 0.5
wrote dr.json (+1 channels)
```

### `face-synth`, `face-train`, `face-eval`, `face-detect`, `face-track`, `face-calibrate`, `face-bake`

Une chaîne pilotée par le visage, **entièrement locale et entraînée uniquement sur des visages synthétiques** (têtes stylisées/cartoon, 23 points de repère, disposition `osatura23`). Un petit modèle entraîné sur des données synthétiques est intégré au binaire ; `--model` le remplace. **Les photographies, les webcams et les vidéos réelles ne sont pas validées** (ROADMAP §9 ; voir aussi [PATENT_NOTES.md](PATENT_NOTES.md) : le puppeteering piloté par le visage n'a pas fait l'objet d'un examen de brevets).

| Command | Synopsis | What it does |
|---|---|---|
| `face-synth` | `osatura face-synth <out-dir> [--frames 90] [--size 160] [--seed 7] [--fps 30]` | Animation synthétique d'une tête : images PNG `000000.png...`, `truth.ndjson` (points de repère réels) et `params.json` (signaux réels) ; `--fps` est la fréquence d'images sur laquelle les signaux sont cadencés (écrite dans l'en-tête du NDJSON). |
| `face-train` | `osatura face-train --samples N --seed S --out model.bin [--mine-rounds 1] [--scene N] [--reg-win N] [--reg-cell N] [--det-win N] [--det-cell N] [--ridge-lambda X] [--stages N] [--refine-win N] [--stage-patch N] [--stage-cell N] [--stage-lambda X]` | Entraîne le détecteur propre au projet uniquement sur des visages synthétiques (aucun téléchargement). `--scene` est la taille des images d'entraînement (64..512) ; `--reg-win`, `--reg-cell`, `--det-win`, `--det-cell`, `--ridge-lambda`, `--stages`, `--refine-win`, `--stage-patch`, `--stage-cell` et `--stage-lambda` sont les réglages de taille du modèle et de régression (les valeurs par défaut sont celles avec lesquelles le modèle intégré a été entraîné). |
| `face-eval` | `osatura face-eval [--model m.bin] [--eval-n 200] [--seed 9] [--scene 128] [--report r.json] [--json]` | Précision synthétique sur données mises de côté, faux positifs et balayages de robustesse (JSON ; `--report` l'écrit, `--json` l'affiche même avec `--report` ; `--scene` est la taille des images de test). |
| `face-detect` | `osatura face-detect <image> [--model m.bin] [--json] [--overlay o.png] [--threshold T] [--max-faces N] [--min-side N] [--max-side N] [--max-dim N]` | Boîtes de visages et 23 points de repère ; `--overlay` les dessine sur un PNG ; `--min-side` / `--max-side` bornent la taille du visage en pixels et `--max-dim` réduit d'abord une grande image. |
| `face-track` | `osatura face-track <video-or-frame-dir> <out.ndjson> [--model m.bin] [--fps 30] [--max-dim 256]` | NDJSON de points de repère pour `face-bake`. Un dossier d'images est lu directement ; une vidéo passe par ffmpeg. |
| `face-calibrate` | `osatura face-calibrate <landmarks.ndjson> <out.json> [--calibrate-frames N] [--map mapfile.json] [--fps F] [--layout L]` | Calcule la calibration du visage neutre à partir des N premières images. |
| `face-bake` | `osatura face-bake <project.json> <landmarks.ndjson> <mapfile.json> <out.json> [--calibrate-frames N\|0] [--calib calib.json] [--fps F] [--layout L] [--tol T] [--allow-empty]` | Précalcule les signaux du visage (`mouth_open`, `eye_l_open`, `eye_r_open`, `brow_raise`, `yaw`, `pitch`, `roll`, ...) en canaux avec un fichier de mapping de style `live-map` dont la `source` est `face`. |

```
$ osatura face-synth syn --frames 6 --size 96 --seed 3
wrote 6 frames (96x96), truth.ndjson and params.json to syn
$ osatura face-train --samples 80 --seed 1 --out m.bin --scene 64        # 47 s in a debug build
wrote m.bin (649900 bytes) in 47.0s: 80 samples, seed 1, mined negatives 9
trained on synthetic faces only: expect stylised/cartoon faces to work, photographs and webcams are NOT validated
$ osatura face-train --samples 60 --seed 1 --out m.bin --scene 2
scene size must be 64..512                                                [exit 1]
$ osatura face-detect syn/000000.png --model m.bin
syn/000000.png: face 0: box centre (48.1, 47.4) side 69.0 score 4.49
$ osatura face-track syn trk.ndjson --model m.bin --fps 15 --max-dim 96
wrote trk.ndjson: 6 frames, face found in 6
$ osatura face-calibrate syn/truth.ndjson calib.json --calibrate-frames 3 --fps 15
6 frames, neutral from the first 3; wrote calib.json
$ echo '{"params":[{"source":"face","id":"mouth_open","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":60}],"triggers":[]}' > map.json
$ osatura face-bake small.json trk.ndjson map.json baked.json --calibrate-frames 3 --fps 15 --tol 0.5
6 landmark frames -> +1 channels (K keys in them), 0 trigger keys set, 0 events; wrote baked.json
$ osatura face-eval --eval-n 6 --model m.bin --report ev.json            # writes the JSON report
```
Ces commandes écrivent leurs fichiers de modèle/rapport avec l'écrivain atomique qui refuse les liens symboliques (les dossiers parents sont créés). `face-bake` et `live-map` indiquent `+N channels (K keys in them), M trigger keys set`, et avertissent sur stderr lorsque l'enregistrement est plus long que le projet (les images en trop ne sont pas précalculées ; la durée du projet reste inchangée).

### `mocap-info`, `mocap-match`, `mocap-retarget`

```
osatura mocap-info     <file.bvh>
osatura mocap-match    <project.json> <file.bvh> [--map map.json]
osatura mocap-retarget <project.json> <file.bvh> <out.json> [--view front|side|top|yaw=DEG] [--map map.json] [--scale auto|N] [--fps F]
    [--smooth N] [--start F] [--end F] [--rest-frame F] [--min-conf C] [--weight W] [--mirror] [--no-root] [--in-place] [--keep-length] [--footlock BONE --ground Y]
```
* **`mocap-info`** affiche la hiérarchie, les canaux, la fréquence d'images et l'étendue d'un fichier BVH.
  ```
  $ osatura mocap-info m.bvh
  4 joints (2 end sites), 15 channels per frame
  4 frames at 0.0417 s (24.000 fps), duration 0.125 s
  Hips  offset (0.000, 0.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
    LeftUpLeg  offset (5.000, 0.000, 0.000)  [Zrot Xrot Yrot]
  ...
  ```
* **`mocap-match`** affiche la **table d'association automatique articulation -> os** (emplacement, articulation BVH, os du rig, confiance, indice `via`, rapport de longueur, note), puis les articulations/os non associés et les avertissements. Les noms et la topologie sont tous deux utilisés. `--map` est un objet JSON `{"bvh joint": "rig bone" | null}` (`null` ou `""` exclut une articulation ; l'enveloppe `{"map": {...}}` est acceptée). Il se termine toujours avec le code 0 - lorsque rien ne correspond, il se contente d'avertir `no bones matched: supply a --map file`.
  ```
  $ osatura mocap-match demo.json m.bvh --map map2.json         # {"Hips":null,"LeftUpLeg":"thigh","LeftLeg":"shin"}
  slot         bvh joint                  rig bone                conf  via         len x  note
  shin.l       LeftLeg                    shin                    1.00  manual      12.00  manual
  thigh.l      LeftUpLeg                  thigh                   1.00  manual      12.00  manual
  unmatched BVH joints: Hips, RightUpLeg
  ...
  ```
* **`mocap-retarget`** précalcule le mouvement sous forme de canaux **ajoutés** : projette les rotations 3D sur la `--view` 2D (`front` par défaut, `side`, `top` ou `yaw=DEG`), met à l'échelle le mouvement de la racine (`--scale auto` ou un nombre), rééchantillonne à `--fps` (par défaut celui du projet), lisse (`--smooth` images), prend les images BVH `--start..--end`, utilise `--rest-frame` comme pose de repos, écarte les associations inférieures à `--min-conf`, mélange avec `--weight`, applique un miroir avec `--mirror`, supprime le mouvement de la racine avec `--no-root`, convertit un clip avec déplacement en clip sur place avec `--in-place`, et épingle un pied avec `--footlock BONE --ground Y`. Affiche la table, les notes et `N bones, F frames at R fps, root scale S, +C channels; wrote <out>`. Un clip plus long que le projet **allonge la durée du projet** ; une note après le résumé indique de quelle durée à quelle durée. `--keep-length` conserve plutôt la durée du projet et tronque les clés à la dernière image (une clé interpolée y est placée ; une note compte les clés non précalculées). Si rien n'atteint le seuil de confiance : `no bone matches to retarget (all below the confidence threshold?)` (code de sortie 1). Remarque : pour cette commande, `--scale` vaut `auto|N`, et non l'échelle de rendu habituelle.
  ```
  $ osatura mocap-retarget demo.json m.bvh mr.json --map map2.json --view side --smooth 1 --in-place --no-root
  ... (table)
  2 bones, 4 frames at 24 fps, root scale 12.0000, +2 channels; wrote mr.json
  ```
**Limites (ROADMAP §9) :** aucun véritable fichier de capture de mouvement n'a été testé (aucun n'est fourni : les exemples utilisent un BVH de 4 images écrit à la main) ; la projection 2D perd la profondeur (un os pointant vers la caméra conserve son dernier angle).

---

## 11. Machines à états, liaisons de données, conteneurs et zones de défilement

Ces commandes gèrent les parties **d'interaction à l'exécution** d'un projet : machines à états (`machines`), liaisons de données
(`bindings`), conteneurs de mise en page (`containers`) et zones de défilement (`scrolls`). Toutes stockent leurs données dans
le fichier de projet ; les formes `add` lisent un petit fichier JSON annexe, le valident et écrivent un nouveau projet.
Le modèle d'exécution lui-même est documenté dans [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

### `machine` (add, list, remove, validate)

**Synopsis :**
```
osatura machine add      <project.json> <out.json> <machine.json>
osatura machine list     <project.json>
osatura machine remove   <project.json> <out.json> <name>
osatura machine validate <project.json>
```

Une **machine** possède des `inputs` nommées (`bool`, `number` ou `trigger`) et un ou plusieurs `layers` ; chaque calque a un
état d'entrée `entry`, des `states` et des `transitions`. Un état joue une **source** : `{"kind":"segment","start":F,"end":F}` (images des
canaux à clés du projet), `{"kind":"clip","clip":NAME}`, `{"kind":"region","marker":NAME}` ou un blend tree 1D
`{"kind":"blend","input":NAME,"children":[{"threshold":T,"source":{...}}]}`. Champs d'un état : `name`, `source`,
`mode` (`loop` par défaut, `once`, `ping_pong`), `speed` (1), `blend_in` (secondes). Champs d'une transition : `from` (un état ou
`"*"`), `to`, `when` (une liste de `{"input":NAME,"op":eq|ne|lt|le|gt|ge,"value":V}`, toutes doivent être vraies ; `{"input":"go"}`
seul signifie « égal à 1 »), `exit_time` (0..1), `duration` (secondes de fondu enchaîné) et `ease`. Les clés inconnues dans une transition
sont une **erreur** (une faute de frappe comme `conditions` transformerait sinon une transition gardée en transition inconditionnelle).

```
$ cat mach.json
{"name":"m","inputs":[{"name":"go","kind":"bool"}],"layers":[{"name":"base","entry":"idle","states":[
 {"name":"idle","source":{"kind":"segment","start":0,"end":12}},
 {"name":"act","source":{"kind":"segment","start":12,"end":36}}],
 "transitions":[{"from":"idle","to":"act","when":[{"input":"go"}],"duration":0.1}]}]}
$ osatura machine add demo.json d1.json mach.json
added machine 'm' (1 machines); wrote d1.json
$ osatura machine list d1.json
m  inputs [go:bool]
  layer base (entry idle, mask []): 2 states, 1 transitions
$ osatura machine validate d1.json
1 machines, 0 bindings: ok
```

`machine remove` refuse de supprimer une machine qu'une liaison pilote encore
(`binding 'goBind' still drives machine 'm'`, code de sortie 1) : supprimez d'abord la liaison.
**Erreurs :** une machine invalide (état inconnu, noms en double, référence d'entrée incorrecte) est rejetée avec un message et
rien n'est écrit. **Voir aussi :** `machine-sim`, `bind`, `state-bake` (l'ancien format de machine basé sur les images).
**Limites :** les machines sont évaluées par le runtime et par `machine-sim` ; les exports n'en précalculent rien (Lottie, GIF
et les autres fichiers ne contiennent que des images, voir [EXPORT_FORMATS.md](EXPORT_FORMATS.md)).

### `machine-sim`

**Synopsis :** `osatura machine-sim <project.json> <script.json> <out_dir|out.json> [--fps N --svg-frames N]`

Rejoue des événements d'entrée horodatés sur une machine et consigne l'état de chaque tick. Le script est
`{"machine": NAME?, "duration": SECONDS?, "events": [{"t": SECONDS, "set": {"input": value}, "trigger": ["name"],
"value": {"external.path": number}}]}`. `machine` vaut par défaut la seule machine du projet, `duration` le dernier
événement plus une seconde ; `value` alimente les liaisons de données (voir `bind`). La valeur par défaut de `--fps` est celle du projet.
`--svg-frames N` produit en plus N images SVG réparties sur la durée de l'exécution.

```
$ cat sc.json
{"duration":1.5,"events":[{"t":0.5,"value":{"hud.go":1}}]}
$ osatura machine-sim d3.json sc.json sim.json --fps 24 --svg-frames 0
   0.000s  base     Entered idle
   0.500s  base     Exited idle
   0.500s  base     Entered act
37 ticks at 24 fps, 3 events, 0 svg frames; wrote sim.json
$ osatura machine-sim d3.json sc.json simdir --svg-frames 2     # a name that does not end in .json is a folder
...
37 ticks at 24 fps, 3 events, 2 svg frames; wrote simdir/sim.json
$ ls simdir
frame_00000.svg  frame_00036.svg  sim.json
```
`sim.json` contient `machine`, `fps`, `ticks[]` (`tick`, `time`, `states` sous forme de paires `[layer,state]`, `fading`) et le
journal des événements. **Voir aussi :** `bind`, `machine`.

### `state-bake`

**Synopsis :** `osatura state-bake <project.json> <machine.json> <events.json> <out.json> [--tol 0.01]`

Précalcule une machine à états **basée sur les images** (un format différent et plus ancien que `machine`) en canaux à clés ordinaires.
`machine.json` est `{"initial":NAME,"states":[{"name","length","looped","channels":[...]}],"transitions":[{"from":NAME
or "*","to","when":COND,"fade":FRAMES,"ease"}]}` où `COND` est l'un de `{"type":"event","name"}`,
`{"type":"above"|"below"|"equals","input","value"}`, `{"type":"finished"}`, `{"type":"after","frames"}`.
`events.json` est une liste de `{"frame":N,"name":NAME,"value":V}`. Le résultat est ajouté aux canaux du projet
(`--tol` est la tolérance de réduction des clés), le projet est compilé et écrit.

```
$ echo '[{"frame":10,"name":"go"}]' > ev.json
$ osatura state-bake demo.json sm.json ev.json baked.json --tol 0.01
wrote baked.json (+1 channels)
```
**Piège :** le canal précalculé est ajouté après les canaux existants ; les canaux s'exécutent dans l'ordre de la liste, supprimez donc ou coupez le
canal d'origine de la même cible si le résultat précalculé doit l'emporter. Un fichier de machine au format `machine`
(avec `layers`) est refusé : `invalid JSON in mach.json: missing field 'initial'`.

### `bind` (add, list, remove) et `bind-eval`

**Synopsis :**
```
osatura bind add    <project.json> <out.json> <name> <value.path> <target:prop|@machine:input>
                     [--expr E --remap a,b,c,d --no-clamp --smooth S --mode replace|add|multiply --weight W]
osatura bind list   <project.json>
osatura bind remove <project.json> <out.json> <name>
osatura bind-eval   <project.json> <values.json> [--data out.json]
```

Une **liaison de données** associe un chemin de valeur externe (par exemple `hud.size`) à une propriété du projet (`sun:layer_opacity`,
les noms de propriétés sont ceux en snake_case de la section 1) ou à une entrée de machine (`@m:go`). `--remap a,b,c,d` fait correspondre la
plage d'entrée `a..b` à `c..d` (bornée sauf avec `--no-clamp`) ; `--expr` est une expression portant sur la valeur ;
`--smooth S` applique un filtre passe-bas sur S secondes ; `--mode` et `--weight` contrôlent la façon dont le résultat se combine avec les canaux.

```
$ osatura bind add d1.json d2.json goBind hud.go @m:go
added binding 'goBind' (1 bindings); wrote d2.json
$ osatura bind add d2.json d3.json sz hud.size sun:layer_opacity --remap 0,100,0.2,1 --smooth 0.1
added binding 'sz' (2 bindings); wrote d3.json
$ osatura bind list d3.json
goBind  hud.go -> @m:go
sz  hud.size -> sun:LayerOpacity remap 0..100 -> 0.2..1 clamp smooth 0.1s
$ echo '{"hud.go":1,"hud.size":50}' > vals.json
$ osatura bind-eval d3.json vals.json --data bv.json
goBind = 1
sz = 0.6000000238418579
$ osatura bind add d2.json x.json sz hud.size sun:scale
unknown property 'scale'                                                  [exit 1]
$ osatura bind remove d3.json d7.json sz
removed binding 'sz'
```
`bind-eval` affiche la sortie de chaque liaison pour les valeurs données (`--data` les écrit aussi en JSON). `list` affiche les noms
de propriétés en CamelCase ; la ligne de commande attend toujours du snake_case. **Voir aussi :** `machine`, `machine-sim` (son tableau `value`).

### `container` (add, list, remove, solve)

**Synopsis :**
```
osatura container add    <project.json> <out.json> <container.json>
osatura container list   <project.json>
osatura container remove <project.json> <out.json> <id>
osatura container solve  <project.json> [--size WxH --data out.json]
```

Un **conteneur** est une boîte de mise en page de type flexbox : `id`, `members`, `axis` (`row`|`column`), `gap`, `cross_gap`, `padding`
(`[left,top,right,bottom]`), `justify` (`start|center|end|space_between|space_around`), `align`
(`start|center|end|stretch`), `wrap` (`none|wrap`) et `rect`, une règle pour son propre rectangle sur le canevas :
`{"kind":"fixed",...}`, `{"kind":"pin","left","right","top","bottom","width","height"}` (distances par rapport aux bords du canevas ; deux valeurs quelconques
par axe), `{"kind":"aspect",...}` ou `{"kind":"hug"}`. Un **membre** est exactement l'un de `{"layer":PATH}`,
`{"container":ID}` ou `{"spacer":true}`, avec `grow`, `shrink`, `width`, `height`, `min_w/max_w/min_h/max_h` et
un `align` de remplacement ; `"resize":true` permet au solveur de mettre le calque à l'échelle de son emplacement.

```
$ cat cont.json
{"id":"row1","axis":"row","gap":10,"padding":[10,10,10,10],
 "rect":{"kind":"pin","left":10,"right":10,"top":10,"height":100},
 "members":[{"layer":"sun"},{"layer":"blush","grow":1,"resize":true}]}
$ osatura container add d3.json d4.json cont.json
added container 'row1' (1 containers); wrote d4.json
$ osatura container list d4.json
row1: Row, 2 members, gap 10, justify Start, align Start, wrap None, rect {"kind":"pin","left":10.0,...}
$ osatura container solve d4.json --size 800x480 --data solved.json
canvas 800x480
container row1: x 10.00 y 10.00 w 780.00 h 100.00
layer sun: dx -430.000 dy 0.000 scale 1.0000 x 1.0000
layer blush: dx 280.432 dy -158.242 scale 33.8889 x 1.0000
```
Sans `--size`, la taille propre du projet est utilisée. `solve` affiche le rectangle de chaque conteneur et, pour chaque calque, le décalage et
l'échelle que la mise en page applique par-dessus l'animation. **Erreurs :** un calque inconnu, un conteneur imbriqué manquant ou un
cycle est rejeté par `add` (rien n'est écrit). **Voir aussi :** `scroll`, `layout-preview`.
**Limites :** la mise en page est appliquée par le runtime/l'aperçu ; les fichiers exportés contiennent la mise en page résolue pour la taille propre du projet.

### `scroll` (add, list, remove, eval)

**Synopsis :**
```
osatura scroll add    <project.json> <out.json> <scroll.json>
osatura scroll list   <project.json>
osatura scroll remove <project.json> <out.json> <id>
osatura scroll eval   <project.json> <id> [--offset X,Y --size WxH --velocity VX,VY --time S]
```

Une **zone de défilement** possède `id`, `content` (l'id d'un conteneur qui n'est pas membre d'un autre conteneur), `viewport`
(une règle de rectangle comme ci-dessus), `axis` (`x`, `y` ou les deux), `offset`, `elastic` (`{"limit","stiffness","damping_ratio"}` ; effet
élastique au lieu d'une butée stricte), `snap_x`/`snap_y` (décalages de repos), `fling` (secondes de vitesse de relâchement projetées pour trouver
le point d'arrivée, 0,15 par défaut), `clip_layer` (un calque rectangle utilisé comme masque pour le contenu) et `input` (un chemin de valeur externe qui
pilote le décalage).

```
$ cat scroll.json
{"id":"sc1","content":"row1","axis":"x","viewport":{"kind":"pin","left":0,"right":0,"top":0,"height":120},
 "elastic":{"limit":60,"stiffness":200,"damping_ratio":0.8},"snap_x":[0,100,200]}
$ osatura scroll add d4.json d5.json scroll.json
added scroll 'sc1' (1 scrolls); wrote d5.json
$ osatura scroll list d5.json
sc1: content row1, axis X, offset [0.0, 0.0], elastic true, snap x [0.0, 100.0, 200.0] y [], clip -
$ osatura scroll eval d5.json sc1 --offset 400,0
viewport [0.0, 0.0, 640.0, 120.0] content [-59.92364, 0.0, 640.0, 120.0] max [0.0, 0.0]
requested offset [400.0, 0.0] -> applied [59.92364, 0.0]
$ osatura scroll eval d5.json sc1 --offset 80,0 --velocity -500,0 --time 0.5
...
requested offset [80.0, 0.0] -> applied [44.184174, 0.0]
t 0.100s offset [22.794, 0.000]
t 0.300s offset [0.051, 0.000]
t 0.500s offset [-0.197, 0.000]
```
Sans `--velocity`, `eval` affiche le décalage borné (ou élastique) correspondant à celui demandé. Avec `--velocity`, elle
simule le relâchement (inertie, magnétisme, ressort) pendant `--time` secondes et affiche une ligne toutes les 0,1 s. `--size WxH` évalue pour une autre taille de canevas.
Dans l'exemple, le contenu est aussi large que la zone visible, donc tout ce qui dépasse 0 est un défilement excédentaire et fait apparaître l'effet élastique.

### `layout-preview`

**Synopsis :** `osatura layout-preview <project.json> <out.svg> --size WxH [--frame N --scroll id=X,Y]`

Rend une image avec la mise en page **recalculée pour une autre taille de canevas** et les contours des conteneurs dessinés, afin de
vérifier comment une création se réagence sur un téléphone ou un écran large. `--scroll id=X,Y` définit un décalage de défilement (à répéter pour plusieurs zones).

```
$ osatura layout-preview d5.json lp.svg --size 800x480 --scroll sc1=50,0
wrote lp.svg (800x480, 1 containers, 2 layers placed)
```

## 12. Effets, profondeur de champ, onion skin, trajectoires et caméra

### `fx`

**Synopsis :** `osatura fx <project.json> <frame> <chain.json> <out.png> [--scale N] [--layer-normals] [--clamp]`

Une `<frame>` hors du projet est une erreur ; `--clamp` rend l'image valide la plus proche.

Rend une image et applique par-dessus une **chaîne d'effets de post-traitement**. `chain.json` est une liste d'effets, chacun avec un `type` :
`blur {radius}`, `drop_shadow {dx,dy,blur,color,opacity}`, `glow {radius,strength,color?}`, `outline {width,color}`,
`posterize {levels}`, `pixelate {size}`, `levels {in_black,in_white,gamma,out_black?,out_white?}`, `fade {amount}` et
`relight {ambient,bevel,height,lights:[{x,y,z,color?,intensity?,radius?,cast_shadow?,spot?,falloff?}],ground_shadow?,
normal_map?,ambient_color?}`. Sans `normal_map`, le relight ombre un biseau fabriqué à partir du canal alpha flouté.

```
$ cat chain.json
[{"type":"drop_shadow","dx":6,"dy":6,"blur":3,"color":"#000000","opacity":0.5},{"type":"glow","radius":6,"strength":0.5}]
$ osatura fx demo.json 12 chain.json fx.png --scale 0.5
wrote fx.png (2 effect(s))
$ cat relight.json
[{"type":"relight","ambient":0.4,"bevel":6,"height":8,"lights":[{"x":300,"y":100,"z":120,"intensity":1.0}],"ground_shadow":0.3}]
$ osatura fx demo.json 12 relight.json lit.png --scale 0.5
wrote lit.png (1 effect(s))
```
`--layer-normals` ombre chaque calque avec sa propre normal map (voir `layer-normal`) au lieu d'une seule carte pour toute l'image.
Une lumière qui omet `z` est rejetée (`missing field 'z'`). **Limites :** la chaîne est un pipeline d'images fixes en PNG uniquement ; aucun de
ces effets n'atteint Lottie, GIF ou le lecteur HTML (l'export Lottie a son propre sous-ensemble d'effets, voir EXPORT_FORMATS.md).

### `color-space`

**Synopsis :** `osatura color-space <project.json> <out.json> srgb|linear`

Définit le `color_space` du projet, l'espace dans lequel le compositeur raster (PNG, GIF, APNG, vidéo, sprite sheet, `compare`, `ci`) et le compositeur flottant (`render-hdr`) effectuent leurs mélanges. `srgb` (la valeur par défaut, et celle avec laquelle chaque image de référence a été enregistrée) mélange les valeurs stockées encodées en sRGB ; `linear` convertit chaque couleur (remplissages, contours, arrêts de dégradé, fond, pixels d'image) en lumière linéaire, compose dans cet espace en f32, puis convertit une seule fois en sRGB à la fin. Un calque noir à 50 % sur du blanc donne alors 188/255 au lieu de 128/255, et les dégradés s'interpolent en lumière linéaire. Avec `srgb`, la clé n'est pas écrite, de sorte que les fichiers existants et les images de référence ne changent pas.

```
$ osatura color-space demo.json lin.json linear
wrote lin.json (color_space: linear)
$ osatura render lin.json 0 lin.png
wrote lin.png
$ osatura color-space lin.json x.json p3
unknown colour space 'p3' (srgb or linear)
```
**Limites :** le moteur de rendu 8 bits bascule vers le compositeur flottant pour les projets linéaires (rastériseur de couverture propre, donc les bords anticrénelés diffèrent légèrement de la sortie sRGB) ; les couleurs des effets de calque (ombre, lueur, teinte) sont utilisées telles qu'écrites, sans conversion ; `dof` et la chaîne de post-effets (`fx`) restent en sRGB ; SVG, Lottie et le moteur de rendu GPU ignorent ce réglage ; le rééchantillonnage des images s'effectue en sRGB avant la conversion. Les modes de fusion sont appliqués aux valeurs linéaires.

### `layer-normal`

**Synopsis :** `osatura layer-normal <project.json> <out.json> <layer> <normal.png|none> [--strength S] [--fit stretch|tile|exact] [--green-down]`

Associe (ou, avec `none`, retire) une normal map en espace tangent à un calque image pour `fx ... --layer-normals`.
`--fit` indique comment une carte d'une autre taille est ajustée (`stretch` par défaut, `tile`, `exact`) ; `--green-down` inverse le canal
vert (cartes de style DirectX) ; `--strength` met à l'échelle l'inclinaison (1 = telle que stockée, 0 = plat).

```
$ osatura layer-normal demo.json dn.json sun n.png --strength 1 --fit stretch
wrote dn.json
$ osatura fx dn.json 12 chain.json fxn.png --scale 0.5 --layer-normals
wrote fxn.png (2 effect(s))
$ osatura layer-normal dn.json dn2.json sun none
wrote dn2.json
```

### `onion` (deux formes)

**Synopsis :**
```
osatura onion <project.json> <frame> <out.png> [--before 2 --after 1 --step 1 --scale N]
osatura onion <project.json> <out.svg> --frame N [--before B --after A --step S --keyed --outline --all-parts
               --layers a,b --tint-before C --tint-after C --opacity X --falloff X --ghosts-only]
```
La première forme écrit un PNG avec des fantômes atténués des images précédentes et suivantes. La seconde écrit un SVG fixe : l'image
avec par-dessus des fantômes d'onion skin teintés. `--keyed` utilise les **images clés** les plus proches au lieu de pas fixes ;
`--outline` dessine les fantômes sous forme de contours ; `--all-parts` crée des fantômes de toutes les parties (par défaut : les parties en mouvement), `--layers` les limite,
`--ghosts-only` omet l'image elle-même.

```
$ osatura onion demo.json 12 on.png --before 2 --after 1 --step 2 --scale 0.5
wrote on.png
$ osatura onion demo.json on.svg --frame 12 --before 2 --after 2 --step 3 --outline --opacity 0.4
wrote on.svg (4 ghost(s) around frame 12)
$ osatura onion demo.json on2.svg --frame 12 --keyed --ghosts-only
wrote on2.svg (3 ghost(s) around frame 12)
```
La forme est choisie d'après le deuxième argument : un nombre indique la forme PNG, un nom se terminant par `.svg` la forme SVG.

### `trail` (deux formes)

**Synopsis :**
```
osatura trail <project.json> <bone> <out.svg> [--from F --to F]
osatura trail <project.json> [out.svg] --target layer:PATH|bone:NAME[:head|tip]|point:LAYER:KIND:A[:B] [--from F --to F --smooth N --json]
```
Dessine la **trajectoire de mouvement** de l'extrémité d'un os (première forme) ou de n'importe quelle cible (seconde forme : un calque, la base ou l'extrémité d'un os, ou un
point de déformateur), avec un repère par image et des marqueurs aux clés. `--smooth N` lisse la courbe ; `--json` affiche la
courbe échantillonnée en JSON (`out.svg` est alors facultatif).

```
$ osatura trail demo.json hand tr.svg --from 0 --to 24
wrote tr.svg (25 points)
$ osatura trail demo.json tr3.svg --target layer:sun --from 0 --to 24
wrote tr3.svg (25 ticks, 0 key marker(s))
$ osatura trail demo.json tr2.svg --target bone:hand:tip --from 0 --to 24 --smooth 2 --json
{ "curve": [ [212.8, 83.84], [213.77, 83.99], ...
```

### `dof` et `dof-export`

**Synopsis :**
```
osatura dof        <project.json> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24 --scale N --clamp]
osatura dof-export <project.json> <out.gif|out.apng|out.mp4|out.webm|out-folder> [--from F --to F --focus N --strength N --max-radius N --scale N --codec vp9|av1]
```
Une `<frame>` hors du projet est une erreur pour `dof` (ainsi que pour `render-hdr` et `fx`) ; `--clamp` rend à la place l'image valide la plus proche, comme pour `render`.

Profondeur de champ : chaque **calque de premier niveau** est flouté de `|parallax - focus|` fois `strength` (pixels par unité de
distance de parallaxe, plafonné à `--max-radius`). Sans `--focus` / `--strength`, les valeurs suivent les canaux à clés
`camera_focus` / `camera_blur` (mise au point par défaut 1.0). Les calques tirent leur profondeur de `parallax` (voir la section 5).
`dof-export` rend la plage `--from`..`--to` (numéros d'images entiers ; par défaut : de 0 à la fin) vers un GIF animé, un
PNG animé, une vidéo `.mp4`/`.webm` (nécessite `ffmpeg` ; `--codec` vaut `vp9` par défaut ; aucun audio n'est inclus) ou, pour un nom sans
extension, un dossier de PNG numérotés (`f0000.png`...). Toute autre extension est refusée (`dof-export writes .gif, .apng,
.mp4, .webm or a folder (no extension)`, rien n'est écrit).

```
$ osatura dof demo.json 12 dof.png --focus 1.0 --strength 12 --max-radius 24 --scale 0.5
wrote dof.png
$ osatura dof-export demo.json dof.gif --from 0 --to 6 --scale 0.25
wrote dof.gif
$ osatura dof-export demo.json dofdir --from 0 --to 2 --scale 0.25      # dofdir/f0000.png ... f0002.png
wrote dofdir
```
**Voir aussi :** `parallax`, `render --proxy`, `shot`/`camera-seq` pour les mouvements de caméra. **Limites :** le flou est une
approximation par calque, pas un modèle d'objectif (pas de formes de bokeh, pas de gestion des occlusions) ; il n'est pas exporté vers Lottie.

## 13. Niveau de détail, rapports de taille et export-fit

### `size-report`

**Synopsis :** `osatura size-report <project.json> [--tol T] [--json]`

Montre **où se trouvent les octets** : le fichier de projet, le Lottie précalculé et la page HTML, par calque, image et canal
(clés, et combien sont supprimables à la tolérance `--tol`, 0,05 par défaut).

```
$ osatura size-report small.json
project file: 17959 bytes (3511 gzip)
  by layer (JSON bytes; embedded image base64 in brackets):
         2983  arm
         2053  leg
  ...
  by channel (keys, removable at tol 0.05):
          420  foot_ctrl.BoneOffsetX  keys 5 removable 1
  ...
baked Lottie: 46843 bytes (11180 gzip), by layer:
         9181  arm
```
`--json` affiche les mêmes données en JSON (`html.defs_bytes_total`, `frame_bytes_max`, ...). Utilisez-la avant `export-fit` ou `lod`
pour savoir quel calque simplifier.

### `lod`

**Synopsis :** `osatura lod <project.json> --tolerance-px 0.5 [--scale S | --size WxH] [--levels 4] [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]`

Construit des **niveaux de détail** L1..Ln, chacun avec une *borne d'erreur déclarée en pixels de sortie* : simplification des tracés, réduction
des clés, effondrement des maillages et élimination. Le niveau k a pour tolérance `--tolerance-px x ratio^(k-1)`. `--scale` ou `--size` indique
la taille d'affichage de la sortie (1 px par unité par défaut). `--out-dir` (par défaut `.`) reçoit `<name>_L1.json` ... et
`--report` un rapport JSON. `--verify` rend `--frames N` images et **mesure** l'écart réel (code de sortie 1 s'il dépasse
la borne).

```
$ osatura lod small.json --tolerance-px 1 --levels 3 --out-dir lv --report rep.json --verify --frames 4
level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  path  mesh  keys  cull  measured_px  ratio
L0      0.000       74       0    37      14    17959       0.0000 ...                              -      -
L1      1.000       70       0    37      14    17651       0.3861 0.386 0.000 0.000 0.000       0.3342   1.2x
L2      2.000       69       0    37      14    17573       0.9277 ...                           0.8185   1.1x
L3      4.000       65       0    37      14    17251       1.5372 ...                           1.3367   1.1x
scale 1 px per unit; level k tolerance = 1 px x 2^(k-1); files lv/s_L1..L3.json
  L1: keys: 3 of 8 keyed channels have a documented lever, 0 reduced
  L1: removed 4 path anchor(s)
$ ls lv
s_L1.json  s_L2.json  s_L3.json
```
**Piège :** les gains sur une petite démo construite à la main sont minimes (environ 4 % à 4 px) ; les LOD sont rentables sur des créations vectorisées ou riches en maillages.
Le rapport et les fichiers de niveaux sont écrits avec une écriture simple (non atomique).

### `lod-pick`

**Synopsis :** `osatura lod-pick <project.json> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--out chosen.json]`

Choisit le **niveau le plus grossier** dont la borne déclarée reste dans `--max-px` pixels lorsque l'illustration est affichée sur un
écran de `--screen-px` (l'échelle est la taille de l'écran divisée par la taille du projet). `--out` écrit ce niveau.

```
$ osatura lod-pick small.json --screen-px 320x240 --out chosen.json
level  tol_px  anchors ...
screen 320x240 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.4638 px, 69 anchors, 0 mesh vertices, 37 keys)
wrote chosen.json
$ osatura lod-pick small.json --screen-px 64x48 --max-px 2 --out chosen2.json
screen 64x48 -> scale 0.1000 px/unit; budget 2 px -> level L4 (declared 0.5310 px, 62 anchors, 0 mesh vertices, 37 keys)
```

### `export-fit`

**Synopsis :** `osatura export-fit <project.json> <out> --max-size 500KB [--format lottie|html|gif|apng|webm|project] [--min-ssim X] [--max-seconds 60] [--measure raw|gzip] [--frames N] [--search ladder|descent|grid] [--report out.json] [--pareto out.csv] [--audit] [--no-baseline]`

Trouve l'**export de meilleure qualité qui tient dans un budget d'octets** : elle explore les paramètres de qualité de l'exporteur (Lottie : `decimals`,
`path_eps`, `key_tol`, `frame_step` ; les autres formats ont les leurs) et note chaque candidat par le SSIM par rapport au rendu
d'origine. `--max-size` accepte `500KB` (1000 octets), `1.5MB`, `64000`, `200KiB` (1024 octets). Le format provient de
`--format` ou de l'extension de sortie (`.json` pour Lottie, `.html`, `.gif`, `.apng`, `.webm`) ; `project` écrit un JSON de
projet allégé. `--measure gzip` applique le budget à la taille gzip (pour les serveurs qui compressent). `--min-ssim` est un plancher de qualité.
`--max-seconds` est un **véritable budget de temps réel** (60 par défaut ; `0` = illimité) : à l'expiration, le meilleur résultat obtenu jusque-là est conservé.
`--frames N` est le nombre d'images échantillonnées pour le SSIM (8 par défaut). `--search` choisit la stratégie (`descent` par défaut).
`--report` / `--pareto` écrivent l'historique de la recherche en JSON / CSV ; `--audit` affiche chaque candidat ; `--no-baseline` omet
la comparaison avec les réglages par défaut.

```
$ osatura export-fit small.json fit.json --max-size 30KB --format lottie --report fr.json
lottie -> fit.json: 25756 bytes raw, 5736 gzip (budget 30000 raw); SSIM mean 0.9699, worst 0.9308 at frame 5, max CIEDE2000 68.67; 5 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (...)
$ osatura export-fit small.json fit.html --max-size 60KB --format html --max-seconds 20 --frames 4
html -> fit.html: 59166 bytes raw, 8550 gzip (budget 60000 raw); SSIM mean 1.0000, worst 1.0000 at frame 5, max CIEDE2000 8.54; 1 candidates built
existing exporter at defaults: 66391 bytes raw, 8933 gzip
$ osatura export-fit small.json fit2.json --max-size 1KB --format lottie --max-seconds 10 --frames 3
lottie -> fit2.json: 14751 bytes raw, 2418 gzip (budget 1000 raw); ...
knobs at their floor: decimals, path_eps, key_tol, frame_step
budget not met: the smallest export found is 14751 bytes (raw), above the budget of 1000 bytes: ...          [exit 1]
$ osatura export-fit small.json fit.xyz --max-size 1KB
cannot tell the format from the output name; pass --format lottie|html|gif|apng|webm|project   [exit 1]
```
**Codes de sortie :** 0 lorsque le résultat tient dans le budget (même si le budget de temps a écourté la recherche) ; 1 lorsque le budget
ne peut pas être respecté (l'export le plus petit est tout de même écrit) ou en cas d'arguments incorrects. **Pièges :** elle travaille sur la sortie précalculée,
donc elle ne peut pas retirer plus que ce que les paramètres permettent (un Lottie de 1 Ko est impossible pour un rig de 14 calques) ; chaque candidat est un export
complet, si bien que les gros projets demandent un `--max-seconds` généreux (une build debug est bien plus lente qu'une build release). Les fichiers de
rapport sont écrits avec une écriture simple. **Voir aussi :** `size-report`, `lod`, `export` (section 4), EXPORT_FORMATS.md (conseils de taille).

## 14. Assurance qualité

Voir aussi, documentés plus haut : `check` (validation, section 3), `diff` (diff sémantique, section 3), `compare` (diff d'images de deux
rendus, section 4).

### `lint`

**Synopsis :** `osatura lint <project.json>`

Vérifications statiques du rig et de l'animation. Affiche une ligne par constat : `Severity CODE [target:property] message`. Code de sortie 1 si un
constat a une gravité Error ou supérieure ; Info et Warning donnent le code 0.
```
$ osatura lint small.json
Info L007 [:CameraX] key at frame 24 lies outside the 0..8 timeline
Info L007 [arm_low:BoneAngle] key at frame 12 lies outside the 0..8 timeline
```
`L007` est une clé située hors de la timeline (ici parce que `small.json` est la démo réduite à 8 images).

### `contracts`

**Synopsis :** `osatura contracts <project.json> <contracts.json>`

**Contrats de mouvement** : tests unitaires pour l'animation. `contracts.json` est une liste de contrats avec un `type` :
`loop_seamless {tol}`, `tip_range {bone,axis,min,max}`, `max_speed {bone,px_per_frame}` (d'autres types existent dans l'énumération
`Contract` du moteur). Les violations s'affichent sur stdout, leur nombre sur stderr, et le code de sortie est 1.
```
$ cat contracts.json
[{"type":"loop_seamless","tol":0.5},{"type":"tip_range","bone":"hand","axis":"y","min":0,"max":400},{"type":"max_speed","bone":"hand","px_per_frame":3}]
$ osatura contracts small.json contracts.json
contract 0 violated at frame 8: bone 'arm_up' ends 35.4 px away from where it started (tolerance 0.5)
...
contract 2 violated at frame 6: tip of 'hand' moves 3.3 px/frame (limit 3)
7 violation(s)                                                            [stderr, exit 1]
```

### `coach`

**Synopsis :** `osatura coach <project.json>`

Suggestions sur le timing et les principes d'animation (chevauchement, anticipation, interpolation). Simples conseils, code de sortie toujours 0.
```
$ osatura coach small.json
C003 [, arm_low, arm_up, foot_ctrl, hand, mouth, target] frame 0: 7 parts start moving on frame 0; offset them by a few frames (overlap) for a more natural feel
```

### `safety`

**Synopsis :** `osatura safety <project.json> [--scale N]`

Rapport sur les flashs, les flashs rouges et le mouvement de caméra (approximation de la WCAG 2.3.1 ; une aide d'ingénierie, pas une
certification de diffusion). Rend chaque image à `--scale` (utilisez une petite valeur pour les longs projets).
```
$ osatura safety small.json --scale 0.25
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.06 widths/s, rotation 0°/s, zoom ×1.00/s
```

### `reduced-motion`

**Synopsis :** `osatura reduced-motion <project.json> <out.json> [--factor 0.25]`

Écrit une copie plus calme : le mouvement de caméra et le wobble sont multipliés par `--factor` (0,25 par défaut).
```
$ osatura reduced-motion small.json calm.json --factor 0.25
wrote calm.json
```

### `golden`

**Synopsis :**
```
osatura golden write <project.json> <golden.json> [--scale N]
osatura golden check <project.json> <golden.json> [--tol T]
```
`write --scale N` accepte N dans (0, 1] : une valeur plus grande est **refusée** (`golden write: --scale 2 is above 1 ...`, rien n'est écrit ; elle était auparavant ramenée silencieusement à 1). `check` refait le rendu à l'échelle stockée dans le fichier golden et n'a pas d'option `--scale` : `golden check ... --scale N` est refusé comme option inconnue.


Un test de régression léger : `write` enregistre un hachage et une miniature 16x16 de chaque image dans `golden.json` ; `check`
refait le rendu et compare. **Exact par défaut** (`--tol 0`) : toute différence du hachage de pixels fait échouer la vérification. `--tol T` (T > 0) assouplit
la vérification : un hachage modifié passe tout de même si la distance entre miniatures est au plus T. Une valeur de `--tol` négative ou non finie est une erreur.
Pour des goldens basés sur des images avec une métrique perceptuelle, utilisez `ci`.
```
$ osatura golden write small.json g.json --scale 0.25
wrote g.json (8 frames)
$ osatura golden check small.json g.json
all 8 frames match exactly
$ osatura golden check small.json g.json --tol 0.01
all 8 frames match (within thumbnail tolerance 0.01)
```
Les images qui diffèrent sont listées (changement de hachage, distance entre miniatures, nombre de cellules de miniature (sur 256) qui diffèrent, plus grand écart de canal
sur 255), et le code de sortie est 1 :
```
$ osatura golden check changed.json g.json
frame 1 differs: pixel hash changed, thumbnail distance 0.003, 12 of 256 thumbnail cells differ, max channel difference 31/255
2 frame(s) differ (exact comparison; --tol T accepts small differences)       [stderr, exit 1]
```
Une image présente d'un seul côté est signalée par `one side only`. **Limites :** le fichier golden ne conserve que le hachage et la
miniature, donc les chiffres rapportés sont au niveau de la miniature, pas du pixel. Un changement invisible dans le rendu (par exemple
une couleur de fond entièrement recouverte par un calque opaque, comme le ciel de la démo) laisse légitimement le hachage inchangé.

### `ci` (init, run, bless, diff)

**Synopsis :**
```
osatura ci init <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run  [manifest] [--update] [--bless] [--prune] [--only a,b] [--junit f.xml] [--html f.html] [--json f.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest]
osatura ci diff a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--ignore x,y,w,h;...] [--aa N] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```
**CI d'animation** : images golden comparées avec une métrique perceptuelle (SSIM sur la luma, CIEDE2000), plus des vérifications structurelles et
des budgets. Le format du manifeste et le flux de travail sont décrits dans [CI.md](CI.md). `exit 0` = réussite, `1` = une vérification a échoué,
`2` = erreur (manifeste illisible, ...).
```
$ osatura ci init small.json --out ci.json
wrote ci.json with 8 checks:
  validate     validate
  render       frames
  render       frames-float
  determinism  determinism
  fuzz         pose-fuzz
  limits       limits
  export-size  gif-size
$ osatura ci run ci.json --junit j.xml --html r.html --json ci_r.json
[PASS   ] validate     validate           project compiles; 8 lint finding(s), none at 'error' or above
[PASS   ] render       frames             6 frame(s) match
[PASS   ] determinism  determinism        6 render(s) repeated byte-identically
[INFO   ] fuzz         pose-fuzz          98 of 100 pose samples fail (98.00%); not a gate: non-gating (gate = false)
[PASS   ] export-size  gif-size           gif export is 82881 bytes (budget 124322)
[INFO   ] perf         render-speed       median 429.67 ms/frame, budget 1633 ms (+50% tolerance = 2449.50 ms)
8 checks: 6 passed, 0 failed, 0 errors, 2 info, 0 updated, 0 blessed (14.7 s)
$ osatura ci diff a.png b.png --metric ssim --out heat.png
size        480x360
SSIM        0.90536   (luma)   channels min 0.90300   (R, G, B)
de2000      max 68.674   mean 1.9434
pixels      11124 differ (6.4375%), 31 forgiven as anti-aliasing, 0 ignored
regions     118
images differ: luma SSIM 0.90536 < 0.99; ...                                [stderr, exit 1]
$ osatura ci run nonexist.json
cannot read manifest nonexist.json: No such file or directory               [exit 2]
```
`init` écrit le manifeste et, sauf avec `--no-goldens`, les PNG golden dans le dossier `goldens/` à côté. `run
--update` réécrit les goldens manquants ou modifiés, `--bless` accepte les nouveaux rendus comme goldens, `--prune` supprime les
fichiers golden qu'aucune vérification ne référence plus, `--only a,b` exécute les vérifications nommées. `bless` équivaut à `run --bless`. `diff` compare deux PNG : `--tol` est le seuil
de la métrique, `--ignore` masque des rectangles (`x,y,w,h;...`), `--aa N` pardonne les différences d'anticrénelage, et `--out`,
`--side`, `--overlay`, `--flicker` écrivent une carte de chaleur, une image côte à côte, une superposition et un GIF à clignotement.
**Piège :** les rapports (`--junit`, `--html`, `--json`) et les fichiers golden sont écrits avec l'écrivain atomique qui refuse les liens symboliques (en créant les
dossiers parents) ; un chemin de rapport qui est un lien symbolique est refusé une fois les vérifications exécutées (les vérifications sont lentes, donc indiquez un vrai chemin). Une build debug est lente : les 8 vérifications ci-dessus ont pris 15 s sur un projet de 8 images.

### `fuzz`

**Synopsis :** `osatura fuzz <project.json> [--samples N] [--seed S] [--strategy random|lhs|halton|sweep|corners|pca] [--frame F] [--report out.json] [--heatmap out.svg] [--minimize] [--gate] [--max-fail-rate R] [--props angle,scale,offx,offy] [--only BONES] [--margin DEG] [--db posedb.json] [--ignore KINDS] [--no-deformers]`

**Fuzzing de poses** : échantillonne des poses dans les limites des os et signale où le rig casse (repliements, étirement, pointes,
auto-intersection, IK hors d'atteinte, violations de limites, NaN, hors canevas). `--strategy` choisit l'échantillonneur (`pca` échantillonne le long des axes PCA des poses clés du projet, ou de la base de poses
donnée avec `--db`), `--minimize` réduit une pose en échec à un reproducteur minimal, `--heatmap` écrit une carte de contraintes,
`--ignore` ignore certains types d'échec, `--only` restreint à certains os. Sans gate, `fuzz` produit un **rapport** : il affiche le résumé et se termine avec le code 0 (`(report only: exit 0; add --gate, ...)`). `--gate` échoue (code 1) dès qu'un échantillon échoue ; `--max-fail-rate R` (une **fraction** 0..1) active le gate à ce taux. (La vérification `fuzz` propre à `ci` a son propre réglage `gate` et n'est pas affectée.)
```
$ osatura fuzz small.json --samples 30 --seed 1 --strategy lhs --report fz.json --heatmap hm.svg --max-fail-rate 1.0
wrote fz.json
wrote hm.svg
fuzz 'small': 30 samples (lhs, seed 1, frame 0), 25 parameters, 30 failures (100.0%), max severity 54.62, baseline 0.00, 0.40 ms/sample
  ik_unreachable: 27
  off_canvas: 1
  stretch: 2
$ osatura fuzz small.json --samples 10 --max-fail-rate 0.5
fuzz: failure rate 100.00% exceeds --max-fail-rate 50.00%                   [stderr, exit 1]
```
La cible IK de la démo n'est volontairement atteignable que dans une petite plage, donc presque toute pose aléatoire est « hors d'atteinte » :
considérez le taux comme un indicateur de régression, et non comme un verdict réussite/échec sur ce rig. Le rapport et la carte de chaleur sont écrits de façon atomique et un chemin qui est un lien symbolique est refusé (sauf avec `--follow-symlinks`) ; le dossier parent doit exister.

### `fuzz-grid`

**Synopsis :** `osatura fuzz-grid <project.json> --x PARAM --y PARAM [--res N] out.svg|out.png|out.json`

Une carte de gravité de deux paramètres (`bone:NAME:PROP`, par exemple `bone:arm_up:angle`) sur une grille `N x N` (32 par défaut).
Le type de sortie suit l'extension.
```
$ osatura fuzz-grid small.json --x bone:arm_up:angle --y bone:arm_low:angle --res 8 grid.svg
wrote grid.svg (8x8 cells, 59.4% failing; x = bone:arm_up:angle, y = bone:arm_low:angle)
```
Même réserve d'écriture simple que pour `fuzz`.

## 15. Gestion de versions (`vcs`)

**Synopsis :**
```
osatura vcs init <project.json> [--author A]
osatura vcs commit [-m MSG | --message MSG] [--author A] [--repo DIR]
osatura vcs log [REV] [--limit N] [--json] [--repo DIR]
osatura vcs status [--json] [--repo DIR]
osatura vcs diff [A [B]] [--json] [--repo DIR]
osatura vcs show [REV] [--json] [--repo DIR]
osatura vcs branch [NAME [AT] | -d NAME] [--repo DIR]
osatura vcs tag [NAME [AT]] [--repo DIR]
osatura vcs checkout REV [--force] [--repo DIR]
osatura vcs merge REV [--no-ff] [-m MSG | --message MSG] [--author A] [--abort] [--repo DIR]
osatura vcs cherry-pick|revert REV [--author A] [--repo DIR]
osatura vcs resolve PATH [--ours] [--theirs] [--value V] [--repo DIR]
osatura vcs fsck|gc [--repo DIR]
```
Les options dépendent de la sous-commande : `--json` uniquement pour `log`, `status`, `diff` et `show` ; `--force` uniquement pour `checkout` ; `--ours`, `--theirs`, `--value` (exactement une) uniquement pour `resolve`.

Un petit historique adressé par contenu pour **un seul fichier de projet**, avec une **fusion sémantique** : les modifications sont comparées comme des
chemins de document (pointeurs JSON), et non comme des lignes de texte, si bien que deux personnes qui modifient des clés différentes n'entrent jamais en conflit. `init` crée
un dossier `.osatura-vcs/` à côté du projet ; la branche par défaut est `main`. Les révisions (`REV`) sont des noms de branches, des noms de
tags ou des préfixes d'identifiants de commit.

```
$ osatura vcs init p.json
initialised ./.osatura-vcs for p.json (no commits yet; run: osatura vcs commit -m "...")
$ osatura vcs commit -m first --author alexio
[5f80a63179] first
$ (edit p.json)
$ osatura vcs status
on branch main
HEAD 5f80a63179
uncommitted changes:
  project: background changed "#f6f1e7" -> "#ffffff"
$ osatura vcs commit -m "white bg"
[29c09d9368] white bg
$ osatura vcs branch alt          # branch alt at 29c09d9368
$ osatura vcs checkout alt        # checked out alt (29c09d9368)
$ osatura vcs log --limit 5
4e21f15940  unknown  30 fps
29c09d9368  unknown  white bg
5f80a63179  alexio  first
$ osatura vcs merge alt           # fast-forward or a merge commit; --no-ff forces a merge commit
merge committed as f08db3f77f
$ osatura vcs tag v1 ; osatura vcs tag
tag v1 at 4e21f15940
v1  4e21f15940
$ osatura vcs fsck
ok: 50 objects, history and refs are consistent
$ osatura vcs gc
gc: kept 50 objects, removed 0 unreachable
```

**Conflits.** Lorsque les deux côtés ont modifié la même valeur, `merge` s'arrête, conserve *notre* côté dans le fichier de projet, liste les
conflits et se termine avec le code 1 :
```
$ osatura vcs merge main
merge stopped with 1 conflict(s); the project file holds our side for each:
  CONFLICT /background
      project: background: ours "#222222" vs theirs "#333333" (base "#000000")
resolve each with: osatura vcs resolve <path> --ours|--theirs|--value <json>, then: osatura vcs commit   (or: osatura vcs merge --abort)
$ osatura vcs resolve background --theirs
no conflict at 'background' (see 'vcs status')                              [exit 1]   (the path needs its leading slash)
$ osatura vcs resolve /background --theirs
resolved /background; 0 conflict(s) left - run: osatura vcs commit
$ osatura vcs commit -m merged
[bd405f446a] merged
$ osatura vcs merge --abort       # alternatively: merge aborted; project restored to HEAD
```
Tant qu'une fusion, un cherry-pick ou un revert n'est pas résolu, `checkout`, `revert` et `commit` sont refusés avec
`a merge / cherry-pick / revert is in progress: resolve the conflicts and commit, or run 'vcs merge --abort'`.
Autres précisions : `diff` sans argument compare le fichier de travail avec HEAD, `diff A B` deux révisions (`--json` affiche une liste
de `{kind,path,description}`) ; `show` affiche un commit et sa liste de modifications ; `branch` sans argument liste les branches (`*` marque
la branche courante) ; `branch -d NAME` refuse de supprimer la branche extraite ; `checkout --force` abandonne les modifications
non validées ; `revert REV` et `cherry-pick REV` créent un nouveau commit (ou s'arrêtent sur des conflits comme `merge`) ; `--repo DIR`
sélectionne un dossier de dépôt autre que le dossier courant ; `--author` vaut `unknown` par défaut ; `log --json` affiche les objets
commit. **Limites :** un fichier par dépôt, en local uniquement (pas de dépôts distants, pas de push/pull), pas de suivi des renommages au-delà des chemins JSON ;
les ressources binaires référencées par le projet ne sont pas versionnées.

## 16. Studio, collaboration, serveur d'agent MCP et registre de l'interface

### `studio`

**Synopsis :** `osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES] [--public-origin URL] [--source-url URL] [--collab-token T (16+ characters)] [--collab-secret-file F [--collab-revoke NAMES]] [--collab-rate N] [--collab-log LOG [--collab-nosync]]`

Démarre l'**éditeur dans le navigateur** (annuler/rétablir, timeline, panneaux live et visage, VCS et opérations de type agent) sous forme de serveur web
local et reste bloqué jusqu'à Ctrl-C. Valeurs par défaut : `--port 8080`, `--bind 127.0.0.1`. Le projet est validé au démarrage ; les enregistrements vont
dans le même fichier (`POST /api/save`).
```
$ osatura studio demo.json --port 18931 --ui-config ui.json
AGPL-3.0 source offer: http://127.0.0.1:18931/source
OSATURA studio on http://127.0.0.1:18931  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in ui.json
```
Options : `--ui-config F` conserve la disposition et les raccourcis (par défaut `~/.config/osatura/ui.json`, sous Windows `%APPDATA%\OSATURA\ui.json`) ; `--source-url URL` définit le lien vers
le code source publié affiché à `/source` et par `--version` (AGPL-3.0 section 13 ; les binaires de release officiels prennent par défaut le commit à partir duquel ils ont été construits) ; `--allow-host NAMES` et `--public-origin URL` acceptent
les requêtes sous un nom public derrière un reverse proxy TLS ; `--collab-token T` (au moins 16 caractères, par exemple
`openssl rand -hex 16`) active la session multi-utilisateur et est **obligatoire pour tout `--bind` autre que loopback** ; `--collab-nosync` n'est valide qu'avec `--collab-log` (sinon c'est une erreur) ; `--collab-log LOG`
rend la session partagée persistante et la reprend après un redémarrage (`--collab-nosync` saute la synchronisation au démarrage).

**Options de durcissement** (vague 7 ; détails et modèle de menaces dans [SECURITY.md](SECURITY.md)) : `--collab-secret-file F` remplace
le jeton porteur unique partagé par des **jetons par participant** : F contient un secret aléatoire (première ligne, au moins 16 caractères, mode 600 sous
Unix, sinon il est refusé) et chaque personne reçoit un jeton via `osatura collab-token NAME` ; un jeton est lié à un seul nom
(la connexion utilise ce nom, personne ne peut se connecter sous le nom d'un autre), expire, et est vérifié en recalculant son HMAC-SHA-256, de sorte que le
serveur ne conserve aucune table. `--collab-revoke NAMES` (séparés par des virgules, nécessite `--collab-secret-file`) refuse les jetons de ces
noms avant leur expiration. `--collab-token` et `--collab-secret-file` peuvent être combinés (le jeton partagé joue alors le rôle de
clé d'administration) ; l'un ou l'autre satisfait la règle selon laquelle un bind non loopback exige un identifiant. `--collab-rate N` limite chaque adresse
cliente à N requêtes par seconde (rafales de 2N ; réponse `429` avec `Retry-After`) ; la valeur par défaut est 100 lorsqu'un identifiant est
requis et aucune limite sinon ; 0 la désactive. Un bind non loopback affiche des avertissements (HTTP en clair ; toutes les interfaces pour
`0.0.0.0` / `::` ; un jeton partagé n'a ni nom ni expiration).

**Règles de sécurité** (vérifiées) : le serveur ne répond qu'aux requêtes dont le `Host` est l'un de ses propres noms
(`curl -H 'Host: evil.example' .../api/state` donne 421) ; `Origin` et `Sec-Fetch-Site` provenant d'une autre page donnent 403 ; le corps d'un POST
doit être `application/json` (`curl -X POST -d '{}'` donne 415). Un jeton trop court est refusé
(`--collab-token must be at least 16 characters`), et `--bind 0.0.0.0` sans jeton est refusé
(`refusing to listen on 0.0.0.0:PORT: the studio has no accounts and no TLS, ...`). Le serveur n'a ni comptes ni TLS : placez
un proxy TLS devant pour tout usage autre que sur votre propre machine ; lorsque `--public-origin` est une URL `https://`, le cookie de connexion reçoit l'attribut `Secure` pour les requêtes qui arrivent sous ce nom (ou avec `Origin` https / `X-Forwarded-Proto: https`). La page sert un petit favicon SVG à `/favicon.ico`. L'API HTTP (par exemple `GET /api/state`, `GET /api/project`,
`POST /api/op`, `POST /api/save`, `GET /frame/N.svg`) existe pour la page elle-même et n'est pas une API publique stable.

### `collab-token`

**Synopsis :** `osatura collab-token <name> [--secret-file F] [--ttl 28800]`

Affiche un jeton de collaboration par participant pour un studio démarré avec `--collab-secret-file F` : `ap1.<name>.<expires>.<mac>`,
où `mac` est le HMAC-SHA-256 hexadécimal du nom et de l'échéance sous le secret. `<name>` fait de 1 à 32 caractères parmi lettres,
chiffres, `_` et `-`. `--ttl` est la durée de vie en secondes (28800 = 8 heures par défaut, au plus 2592000 = 30 jours). Le secret est lu dans
le fichier `F` (première ligne) ou, sans `--secret-file`, dans la variable d'environnement `OSATURA_COLLAB_SECRET` ; il n'est jamais
accepté en argument, où d'autres utilisateurs pourraient le voir. Le jeton va sur stdout, l'échéance sur stderr.

```
$ openssl rand -hex 32 > collab.secret && chmod 600 collab.secret
$ osatura studio demo.json --collab-secret-file collab.secret
$ osatura collab-token ana --secret-file collab.secret --ttl 3600
ap1.ana.1790000000.3f0c...
token for 'ana', valid until Unix time 1790000000 (in 3600 s); give it to that person only; ...
$ osatura collab-token ana --secret-file collab.secret --ttl 0
--ttl needs a whole number of seconds (1 to 2592000)
```
La personne colle le jeton sur la page de connexion du studio (il est alors conservé dans un cookie HttpOnly qui expire avec le jeton) ou l'envoie dans
`X-Collab-Token`. Un jeton expiré reçoit la réponse `401` avec l'échéance dans le message ; le nom du participant est tiré du
jeton. **Voir aussi :** `studio`.

### `collab-sim`

**Synopsis :** `osatura collab-sim <project.json> [--clients N] [--ops M] [--seed S] [--undo-pct P] [--replays K] [--require-valid] [--raw] [--json]`

Un banc d'essai pour le moteur de collaboration : N réplicas modifient le projet en parallèle avec un ordre de livraison aléatoire et des
doublons, puis il vérifie que tous détiennent le même document (code de sortie 1 sinon). Les modifications par défaut sont des modifications de projet
valides (nombres, valeurs de clés, visibilité, calques propres) ; `--raw` utilise des modifications JSON arbitraires (généralement pas un projet valide) ;
`--require-valid` fait aussi échouer (code 1) un résultat qui ne compile pas.
```
$ osatura collab-sim small.json --clients 3 --ops 20 --seed 2 --json
{"buffered":12,"clients":3,"clocks_identical":true,"converged":true,"converged_ok":true,"deliveries":34,"duplicates_dropped":0,"edits":20,"envelopes":17,...}
```
`converged_ok` désigne la seule convergence ; `ok` exige en plus que le document fusionné compile.

### `mcp`

**Synopsis :** `osatura mcp <project.json>`

Un **serveur d'agent** : JSON-RPC 2.0 (le Model Context Protocol) sur stdin/stdout, un message JSON par ligne. Il charge le
projet et expose les outils `get_project`, `stats`, `lint`, `coach`, `safety`, `apply_ops`, `set_key`, `undo`, `redo`,
`render_frame` et `save`. Rien n'est écrit tant que `save` n'est pas appelé. Stdout ne transporte que des messages du protocole.
```
$ printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lint","arguments":{}}}' | osatura mcp demo.json
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"protocolVersion":"2024-11-05","serverInfo":{"name":"osatura","version":"0.1.0"}}}
{"id":2,"jsonrpc":"2.0","result":{"tools":[{"description":"Return the project JSON, or the value at a JSON pointer ...","name":"get_project",...}]}}
{"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"[]","type":"text"}],"isError":false}}
```
Le serveur se termine (code 0) lorsque stdin se ferme.

### `ui`

**Synopsis :**
```
osatura ui commands|shortcuts [--ui-config F] [--json]
osatura ui layout presets|show <name>|validate <file.json>
osatura ui search <query...> [--ui-config F]
```
Le **registre des commandes** du studio : `commands` liste chaque commande avec sa catégorie, son titre et ses raccourcis clavier ;
`shortcuts` liste les raccourcis effectifs (valeurs par défaut plus celles de `--ui-config`) et les éventuels conflits ; `layout presets` liste les
préréglages de disposition des panneaux, `layout show NAME` en affiche un en JSON et `layout validate FILE` vérifie un fichier de disposition ;
`search` classe les commandes comme le fait la palette de commandes.
```
$ osatura ui commands
Edit
  edit.undo                    Undo                               global:Ctrl+Z
  edit.redo                    Redo                               global:Ctrl+Shift+Z  global:Ctrl+Y
$ osatura ui layout presets
animate  V0.62(H0.74([*stage] | [*layers selection bones live face json]) | [*timeline])  [valid]
rig      H0.66(V0.84([*stage] | [*timeline (folded)]) | V0.50([*layers bones] | [*selection json]))  hidden: live,face  [valid]
compact  V0.62([*stage] | [*timeline layers selection bones live face json])  [valid]
live     H0.58(V0.70([*stage] | [*timeline]) | [*live face bones selection])  hidden: layers,json  [valid]
$ osatura ui layout validate demo.json
demo.json: unknown field `format`, expected one of `version`, `root`, `floating`, `hidden` at line 2 column 10   [exit 1]
```

## 17. Base de poses, suggestions et autres utilitaires

Ces commandes construisent une **base de poses** (rotations des os aux images clés d'un projet et de chaque take d'une bibliothèque)
et utilisent sa structure apprise (axes PCA, petit réseau optionnel) pour suggérer des poses. C'est une aide statistique entraînée sur
*vos* poses, pas un modèle général. Leurs fichiers de modèle et de rapport sont écrits avec une écriture simple.

### `pose-db` (build, info, train-mlp)

**Synopsis :**
```
osatura pose-db build <out.json> [--project P.json] [--library takes.json] [--step N]
osatura pose-db info  <db.json>
osatura pose-db train-mlp <db.json> <out.json|out.bin> [--hidden 64,64] [--epochs N] [--lr R] [--seed S]
```
```
$ osatura pose-db build db.json --project demo.json
wrote db.json: 5 poses of 3 bones in 1 sequence(s)
$ osatura pose-db info db.json
5 poses, 3 bones: arm_up, arm_low, hand
  sequence 0: project (project, 5 poses)
PCA: 2 axes keep 99.2% of the variance (total 950.0 deg^2)
  axis 1: std 27.34 deg, cumulative 78.7%
  axis 2: std 13.95 deg, cumulative 99.2%
typical nearest-pose spacing 14.43 deg, typical step between consecutive poses 35.82 deg
$ osatura pose-db train-mlp db.json mlp.json --hidden 8 --epochs 20 --seed 1
wrote mlp.json: 6 -> [8] -> 3 (epochs 20, loss 1.0189 -> 0.4921)
trained on the library itself (no held-out data here); run pose-bench for held-out accuracy
```
`--project` ajoute les poses aux images clés du projet, `--library` les takes d'une bibliothèque de takes (`take-record`, section 6),
`--step N` échantillonne en plus toutes les N images. Le petit réseau (SGD simple) est facultatif ; utilisez-le avec `--model` ci-dessous.

### `pose-suggest` (between, complete, next)

**Synopsis :**
```
osatura pose-suggest between  <project.json> <db.json|-> --a F1 --b F2 [--t 0.5] [--snap 0.5] [--top 5] [--apply RANK --out out.json] [--json]
osatura pose-suggest complete <project.json> <db.json|-> --frame F [--fix bone=angle,...] [--reach bone=x,y,...] [--free b1,b2] [--prior 1] [--model m.json] [--apply RANK --out out.json]
osatura pose-suggest next     <project.json> <db.json|-> --frame F [--k 8]
```
`-` à la place d'une base fait apprendre à partir du projet lui-même. `between` propose une pose intermédiaire entre deux images qui reste
sur la variété apprise (candidats `linear`, `pca`, `manifold`) ; `complete` trouve la pose complète la plus plausible étant donnés des angles
fixés (`--fix`), des cibles d'extrémité (`--reach`) et des os libres ; `next` montre ce que la bibliothèque a fait après des poses semblables à celle de
l'image. `--apply RANK` (avec `--out`) écrit le candidat choisi sous forme de clés.
```
$ osatura pose-suggest between demo.json db.json --a 0 --b 24 --t 0.5 --top 3
between: 5 library poses, 2 PCA axes (99% of the variance)
 1. linear      score 1.00  plausibility 1.00  key at frame 12.0
      linear blend of the angles
      arm_up=0.0 arm_low=-7.5 hand=-10.0
 3. manifold    score 1.00  plausibility 1.00  key at frame 12.0
      PCA blend pulled toward the library
$ osatura pose-suggest complete demo.json db.json --frame 12 --fix arm_up=-40
 1. pca-ls      score 0.96  plausibility 1.00  constraint error 0.18  key at frame 12.0
      regularised least squares in PCA space (start 1)
      arm_up=-39.8 arm_low=24.9 hand=11.2
 2. library-nn  score 0.25  plausibility 1.00  constraint error 15.00  key at frame 12.0
(scores rank candidates against this library; they are heuristics, not probabilities)
$ osatura pose-suggest next demo.json db.json --frame 12 --k 3
 1. successor   score 0.60  plausibility 1.00  key at frame 24.0
      what followed pose 1 of 'project'
```

### `pose-bench`

**Synopsis :** `osatura pose-bench`

Affiche la précision sur données mises de côté des méthodes de suggestion sur une **bibliothèque synthétique de type marche** (24 cycles d'entraînement, 6
cycles mis de côté ; la complétion voit 3 os sur 10 ; elle entraîne aussi le petit réseau). Plus bas est meilleur. Elle ne lit aucun projet. Elle est
**lente dans une build debug** (des minutes) ; chiffres d'une build release :
```
completion  mean pose                                   20.96 deg RMS
completion  nearest library pose                         1.77 deg RMS
completion  PCA least squares                            0.51 deg RMS
completion  MLP                                          1.64 deg RMS
between     linear blend                                 1.83 deg RMS
next        Markov mean                                  0.72 deg RMS
```

### `pose-match`

**Synopsis :** `osatura pose-match <project.json> <params.json> <goals.json> <out.json> [--generations N --seed N]`

Recherche (un optimiseur évolutionnaire, 80 générations par défaut, graine 1, plage valide 1..100000) des valeurs d'un ensemble de
paramètres qui placent les extrémités d'os à des positions cibles dans l'espace de la scène. `params.json` est une liste de
`{"target","prop","min","max","start"}` ; `goals.json` est une liste de `{"bone","at":[x,y],"weight"}`. Le résultat est ajouté sous forme de
canaux ; la perte est la somme pondérée des carrés des erreurs d'extrémité en px².
```
$ echo '[{"target":"arm_up","prop":"bone_angle","min":-60,"max":40,"start":0},{"target":"arm_low","prop":"bone_angle","min":-60,"max":60,"start":0}]' > params.json
$ echo '[{"bone":"hand","at":[260,200],"weight":1.0}]' > goals.json
$ osatura pose-match demo.json params.json goals.json out.json --generations 30 --seed 3
wrote out.json (loss 3756.156 px²)
```
**Piège :** la perte de l'exemple est élevée parce que les plages ne permettent pas d'atteindre la cible : vérifiez la valeur, pas seulement le
code de sortie (un résultat est écrit même lorsque la correspondance est mauvaise).
## Divergences connues entre la documentation et le comportement

Lorsque le texte d'aide du programme (affiché par `osatura` sans argument) ou un document voisin dit
une chose et que le programme en fait une autre, les sections ci-dessus décrivent ce que le programme fait réellement. Voici les différences constatées
lors de la vérification de chaque exemple, classées par probabilité décroissante de poser problème :

1. **Corrigé - options inconnues.** Toute commande dotée d'une ligne d'aide rejette désormais les `--words` non documentés (avec une suggestion « vouliez-vous dire »),
   y compris `check`, `stats`, `lint`, `coach`, `contracts` et `machine list`. `--json`, `--quiet` et `--allow-empty` sont
   propres à chaque commande et refusés ailleurs (changement incompatible pour les scripts qui les passaient partout). Depuis la vague 5, la vérification se fait **par sous-commande** (`ci diff ... --quiet`, `trail <bone> ... --json`, `ui layout presets --json`, `golden write --tol` sont refusés) et couvre aussi les options à valeur. Tests : `osatura-cli/tests/doc_mismatch.rs` (matrice d'acceptation) et `options_are_validated_per_sub_command` dans `main.rs`.
2. **Corrigé - sorties en `fs::write` simple.** `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, fichiers de poses, snapshots,
   `vcs` et le journal de collaboration utilisent l'écrivain atomique qui refuse les liens symboliques ; les sorties ffmpeg sont vérifiées au préalable. Un test piloté par table
   couvre environ 24 commandes. Non couvert de bout en bout : `face-train`/`face-eval` (test unitaire sur leur écrivain seulement).
3. **Corrigé - `golden check`** est exact par défaut ; `--tol` l'assouplit ; il affiche le nombre de différences et le plus grand écart.
   L'affirmation selon laquelle « un changement de couleur de fond passe » était un artefact de la démo (le calque de ciel recouvre le fond), pas un bogue.
4. **Corrigé - code de sortie de `fuzz`.** Par défaut, rapport seul (code 0) ; `--gate` ou `--max-fail-rate` active le gate. Voir `fuzz`.
5. **Corrigé - `dof-export`** écrit des `.mp4` et `.webm` (via ffmpeg, avec `--codec`, sans audio) ; le texte d'aide les mentionne désormais. Un nom
   avec toute autre extension est refusé et rien n'est écrit.
6. **Corrigé - `export` `.png`** écrit `<name>.atlas.json`, jamais `<name>.json` ; l'erreur de format inconnu liste toutes les extensions.
7. **Valeurs par défaut que le texte d'aide n'indique pas :** `studio` écoute sur `127.0.0.1:8080` ; `fuzz-grid --res` vaut 32 ; `export-fit`
   échantillonne 8 images, cherche avec `descent` et s'arrête après 60 s (`--max-seconds 0` = illimité) ; `reduced-motion --factor`
   vaut 0,25 ; `size-report --tol` vaut 0,05 ; `pose-match` exécute 80 générations avec la graine 1 ; `lod --out-dir` est le dossier
   courant.
8. **`machine remove` peut refuser** (`binding 'goBind' still drives machine 'm'`) ; le texte d'aide ne mentionne pas la
   vérification de dépendance. `machine` et `state-bake` lisent deux formats de machine différents (`layers`/`inputs` contre
   `initial`/`states`) ; la ligne d'aide de `state-bake` n'a pas de description et n'indique aucun format.
9. **Corrigé - `stats`** n'accepte plus `--json` (la sortie est toujours en JSON ; la ligne d'aide le dit).
10. **`vcs` :** la branche par défaut est `main` (`checkout master` échoue avec `unknown revision 'master'`) ; `vcs resolve`
    prend le chemin du conflit *avec* sa barre oblique initiale, exactement comme `merge` l'affiche (`/background`), `background` seul donne
    `no conflict at 'background'` ; le texte d'aide écrit « see: osatura vcs » mais `osatura vcs` sans argument affiche une
    aide d'une ligne et se termine avec le code 1.
11. **Lignes d'aide sans description.** `state-bake`, `pose-match`, `pose-bench`, `follow`, `keys`, `event`, `marker`,
    `idle`, `mouth`, `beats`, `export`, `frames`, `onion` et d'autres ont une syntaxe mais aucune explication, ou une description qui
    se trouve sur une ligne de continuation ; leur comportement dans ce document provient de leur exécution.
12. **Nommage des propriétés.** Les messages et les sorties de `list` affichent les propriétés en CamelCase (`LayerOpacity`, `BoneAngle`),
    mais toutes les options, tous les fichiers de mapping et fichiers annexes attendent les noms en snake_case (`layer_opacity`, `bone_angle`) ; la forme CamelCase
    est rejetée (`bind add ... sun:LayerOpacity` échoue avec `unknown property 'LayerOpacity'`).
13. **`cloth` :** le texte d'aide parle de « per-vertex deformation » ; le résultat précalculé est stocké sous forme de canaux `lattice_dx` / `lattice_dy`
    sur les points de la grille du tissu (pour la grille 7x7 par défaut, les 42 points non épinglés), voir la section 7.
14. **`render-gpu`** n'existe que dans les builds faites avec `--features gpu` ; il n'a pas pu être exécuté dans la build de test, donc ses options
    sont reprises du texte d'aide et ne sont pas vérifiées ici.
15. **Lent dans les builds debug.** `pose-bench`, `face-train`, `ci run` et `export-fit` prennent des minutes dans une build non optimisée.
    Les durées de ce document proviennent du binaire debug et sont bien plus courtes dans une build release.
16. **Corrigé (vague 5) - `golden write --scale`** supérieur à 1 était auparavant ramené silencieusement à 1 ; il est désormais refusé (l'échelle est dans (0, 1]).
    `golden check` ignorait un `--scale` et utilisait celui qui est stocké ; il refuse désormais l'option.
17. **Corrigé (vague 5) - synopsis.** Chaque option du texte d'aide d'une commande figure dans son synopsis ici et inversement (test
    `cli_reference_synopses_match_usage`), et chaque ligne d'aide sans alternatives est répétée ici mot pour mot (test
    `cli_reference_repeats_usage_lines_verbatim`). `render-gpu --features gpu` et `layer-normal ... relight --layer-normals`
    dans les descriptions d'aide désignent une fonctionnalité de build et l'option d'une autre commande, et non des options de ces commandes.
18. **D'autres documents sont plus anciens que le programme sur deux points** (voir [EXPORT_FORMATS.md](EXPORT_FORMATS.md) section 17) :
    `docs/LIMITATIONS.md` L-29 indique que l'import Lottie ne prend en charge ni images, ni effets, ni précompositions, ni mattes (l'importeur les gère désormais,
    avec les avertissements listés à la section 9), et L-32 liste OCA parmi les formats d'import alors que la CLI n'a que
    `export-oca`.
## Index alphabétique des commandes

Chaque commande avec un résumé d'une ligne et la section qui la documente. Les sous-commandes (`machine add`, `vcs commit`, ...) se trouvent dans la section de leur commande parente.

| Command | Summary | Section |
|---|---|---|
| `--version` | affiche la version, la licence, l'offre de code source et les liens d'assistance | [1](#1-conventions-communes-à-toutes-les-commandes) |
| `align` | aligne des calques entre eux ou sur la scène | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `audio-react` | le volume sonore pilote une propriété | [8](#8-audio-synchronisation-labiale-et-sons) |
| `autorig` | IK, respiration et clignement configurés à partir des noms d'os et de calques | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `autotrace` | vectorisation bitmap locale (aplats de couleur, ajustement de Bézier) | [9](#9-import) |
| `autotrace-bench` | tableau de fidélité du vectoriseur sur des formes synthétiques | [9](#9-import) |
| `beats` | détecte les sauts de volume et enregistre des marqueurs ou des événements | [8](#8-audio-synchronisation-labiale-et-sons) |
| `bind` | liaisons de données : add, list, remove | [11](#11-machines-à-états-liaisons-de-données-conteneurs-et-zones-de-défilement) |
| `bind-eval` | évalue les liaisons pour des valeurs données | [11](#11-machines-à-états-liaisons-de-données-conteneurs-et-zones-de-défilement) |
| `bone-vis` | affiche un os dans les surimpressions seulement sur une plage d'images | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `cage` | ajoute un déformateur à cage par coordonnées de valeur moyenne | [7](#7-physique-tissu-et-déformateurs) |
| `cage-pose` | pose des clés de décalage des sommets de la cage à une image | [7](#7-physique-tissu-et-déformateurs) |
| `camera-seq` | anime la caméra à travers des plans enregistrés | [6](#6-animation-clés-timeline-et-générateurs) |
| `chan-meta` | couleur du canal, indicateurs shy, solo et verrou | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `check` | valide un projet (code de sortie 1 si invalide) | [3](#3-création-et-inspection-de-projets) |
| `ci` | CI d'animation : goldens, diff perceptuel, vérifications structurelles | [14](#14-assurance-qualité) |
| `clip` | superpose un take sur la pile de clips | [6](#6-animation-clés-timeline-et-générateurs) |
| `cloth` | précalcule une simulation de tissu déterministe | [7](#7-physique-tissu-et-déformateurs) |
| `coach` | suggestions sur le timing et les principes d'animation | [14](#14-assurance-qualité) |
| `collab-sim` | test de convergence de la collaboration | [16](#16-studio-collaboration-serveur-dagent-mcp-et-registre-de-linterface) |
| `collab-token` | émet un jeton de collaboration par participant (HMAC-SHA-256, expirant) | [16](#16-studio-collaboration-serveur-dagent-mcp-et-registre-de-linterface) |
| `color-space` | choisit une composition sRGB ou en lumière linéaire pour le projet | [12](#12-effets-profondeur-de-champ-onion-skin-trajectoires-et-caméra) |
| `comp` | états de calques nommés (visibilité, opacité, choix de commutateur) | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `compare` | image avant / après / différence de deux projets | [4](#4-rendu-et-export) |
| `container` | conteneurs de mise en page : add, list, remove, solve | [11](#11-machines-à-états-liaisons-de-données-conteneurs-et-zones-de-défilement) |
| `contracts` | contrats de mouvement (tests unitaires pour l'animation) | [14](#14-assurance-qualité) |
| `demo` | écrit le projet de rig de démonstration intégré | [3](#3-création-et-inspection-de-projets) |
| `demo-fx` | écrit la démo liquide / booléen / particules | [3](#3-création-et-inspection-de-projets) |
| `diff` | différences sémantiques entre deux projets | [3](#3-création-et-inspection-de-projets) |
| `distribute` | espace régulièrement trois calques ou plus | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `dof` | rendu avec profondeur de champ d'une image | [12](#12-effets-profondeur-de-champ-onion-skin-trajectoires-et-caméra) |
| `dof-export` | profondeur de champ animée (GIF/APNG/vidéo/dossier) | [12](#12-effets-profondeur-de-champ-onion-skin-trajectoires-et-caméra) |
| `drive` | signaux CSV -> clés via un fichier de mapping | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `event` | ajoute un événement d'exécution nommé (avec charge utile JSON facultative) | [6](#6-animation-clés-timeline-et-générateurs) |
| `explain` | pourquoi une propriété a sa valeur : contributions ordonnées | [3](#3-création-et-inspection-de-projets) |
| `export` | exporte en GIF, APNG, WebM/MP4, planche PNG, HTML, Lottie, .lottie, glTF/GLB, ... | [4](#4-rendu-et-export) |
| `export-fit` | export de meilleure qualité tenant dans un budget d'octets | [13](#13-niveau-de-détail-rapports-de-taille-et-export-fit) |
| `export-oca` | écrit un dossier Open Cel Animation | [4](#4-rendu-et-export) |
| `eye-clip` | masque les calques d'iris/pupille selon la forme de l'œil | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `face-bake` | points de repère -> clés | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `face-calibrate` | calibration du visage neutre | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `face-detect` | boîtes de visages et 23 points de repère dans une image | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `face-eval` | rapport de précision du détecteur sur données mises de côté | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `face-synth` | animation synthétique d'une tête | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `face-track` | NDJSON de points de repère à partir d'une vidéo ou d'images | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `face-train` | entraîne le détecteur de visages sur des visages synthétiques | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `follow` | générateurs d'action secondaire / de prolongement du mouvement | [6](#6-animation-clés-timeline-et-générateurs) |
| `foot-lock` | épingle un contrôleur pendant le contact avec le sol | [6](#6-animation-clés-timeline-et-générateurs) |
| `frames` | rend chaque image dans un dossier (SVG, ou PNG avec --png) | [4](#4-rendu-et-export) |
| `fuzz` | trouve les poses où le rig casse | [14](#14-assurance-qualité) |
| `fuzz-grid` | carte de gravité de deux paramètres | [14](#14-assurance-qualité) |
| `fx` | rend une image avec une chaîne d'effets de post-traitement | [12](#12-effets-profondeur-de-champ-onion-skin-trajectoires-et-caméra) |
| `glue` | colle des sommets de maillage ensemble | [7](#7-physique-tissu-et-déformateurs) |
| `golden` | écrit/vérifie des images de régression | [14](#14-assurance-qualité) |
| `idle` | génère un canal d'inactivité procédural | [6](#6-animation-clés-timeline-et-générateurs) |
| `import-anim` | GIF/APNG animé -> calque image par image | [9](#9-import) |
| `import-bvh` | capture de mouvement BVH -> canaux | [9](#9-import) |
| `import-lottie` | sous-ensemble Lottie -> projet | [9](#9-import) |
| `import-ora` | OpenRaster -> projet | [9](#9-import) |
| `import-seq` | séquence PNG -> calque image par image | [9](#9-import) |
| `import-svg` | sous-ensemble SVG -> calque vectoriel | [9](#9-import) |
| `keyedit` | modifications de dope sheet en un seul lot annulable en une fois | [6](#6-animation-clés-timeline-et-générateurs) |
| `keys` | opérations de clés en masse sur tous les canaux à clés | [6](#6-animation-clés-timeline-et-générateurs) |
| `layer-normal` | associe une normal map à un calque | [12](#12-effets-profondeur-de-champ-onion-skin-trajectoires-et-caméra) |
| `layer-pivot` | déplace le pivot d'un calque sans changer l'image | [7](#7-physique-tissu-et-déformateurs) |
| `layout-preview` | rendu avec mise en page recalculée pour une autre taille de canevas | [11](#11-machines-à-états-liaisons-de-données-conteneurs-et-zones-de-défilement) |
| `lint` | vérifications statiques du rig/de l'animation (code de sortie 1 en cas d'erreurs) | [14](#14-assurance-qualité) |
| `lipsync` | pilote un calque de commutation de bouche à partir du volume audio (sans phonèmes) | [8](#8-audio-synchronisation-labiale-et-sons) |
| `lipsync-text` | aligne une transcription sur l'audio et en déduit les formes de bouche | [8](#8-audio-synchronisation-labiale-et-sons) |
| `live-map` | entrée en direct enregistrée (MIDI, souris, manette, touches) -> clés, takes, événements | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `lod` | niveaux de détail avec bornes d'erreur en pixels déclarées | [13](#13-niveau-de-détail-rapports-de-taille-et-export-fit) |
| `lod-pick` | niveau le plus grossier qui tient dans un budget de pixels sur un écran | [13](#13-niveau-de-détail-rapports-de-taille-et-export-fit) |
| `loop-solve` | fait boucler les canaux à clés sans raccord | [6](#6-animation-clés-timeline-et-générateurs) |
| `machine` | machines à états d'exécution : add, list, remove, validate | [11](#11-machines-à-états-liaisons-de-données-conteneurs-et-zones-de-défilement) |
| `machine-sim` | rejoue des entrées horodatées : journal d'états plus images SVG | [11](#11-machines-à-états-liaisons-de-données-conteneurs-et-zones-de-défilement) |
| `magnet` | précalcule une attraction de proximité | [6](#6-animation-clés-timeline-et-générateurs) |
| `make-loop` | fait boucler un cycle sans raccord (éventuellement sur place) | [6](#6-animation-clés-timeline-et-générateurs) |
| `marker` | ajoute un marqueur ou une plage sur la timeline | [6](#6-animation-clés-timeline-et-générateurs) |
| `mcp` | serveur d'agent (JSON-RPC sur stdio) | [16](#16-studio-collaboration-serveur-dagent-mcp-et-registre-de-linterface) |
| `mesh-lock` | des suiveurs suivent la surface d'un maillage hôte | [7](#7-physique-tissu-et-déformateurs) |
| `mesh-trace` | vectorise la silhouette alpha d'une image en un maillage triangulaire avec skin | [7](#7-physique-tissu-et-déformateurs) |
| `mirror` | applique un miroir gauche<->droite à tous les canaux à clés | [6](#6-animation-clés-timeline-et-générateurs) |
| `mocap-info` | hiérarchie et étendue d'un fichier BVH | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `mocap-match` | association des articulations BVH aux os du rig | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `mocap-retarget` | précalcule la capture de mouvement en canaux ajoutés | [10](#10-entrée-en-direct-midi-suivi-du-visage-et-capture-de-mouvement) |
| `mouth` | importe des repères de bouche (Papagayo/Rhubarb) | [8](#8-audio-synchronisation-labiale-et-sons) |
| `onion` | onion skin en PNG ou SVG | [12](#12-effets-profondeur-de-champ-onion-skin-trajectoires-et-caméra) |
| `parallax` | profondeur multiplan pour les calques de premier niveau | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `parent-switch` | précalcule un changement de parent animé sans saut | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `physics` | précalcule une simulation de chaîne | [7](#7-physique-tissu-et-déformateurs) |
| `pick` | ce qui se trouve sous un point du canevas (le plus haut d'abord) | [3](#3-création-et-inspection-de-projets) |
| `pose-bench` | précision sur données mises de côté des méthodes de poses | [17](#17-base-de-poses-suggestions-et-autres-utilitaires) |
| `pose-db` | base de poses : build, info, train-mlp | [17](#17-base-de-poses-suggestions-et-autres-utilitaires) |
| `pose-match` | ajuste des paramètres pour que les extrémités d'os atteignent des cibles | [17](#17-base-de-poses-suggestions-et-autres-utilitaires) |
| `pose-suggest` | suggestions d'intercalaires, de complétion et de pose suivante | [17](#17-base-de-poses-suggestions-et-autres-utilitaires) |
| `proxy-bench` | millisecondes par image à chaque niveau de proxy | [4](#4-rendu-et-export) |
| `reduced-motion` | écrit une copie plus calme d'un projet | [14](#14-assurance-qualité) |
| `reference` | marque un calque comme référence de tracé (jamais exporté) | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `render` | rend une image en PNG ou SVG (proxys de défilement, os, échelle) | [4](#4-rendu-et-export) |
| `render-gpu` | rendu GPU (builds wgpu uniquement) | [4](#4-rendu-et-export) |
| `render-hdr` | rendu en virgule flottante vers OpenEXR ou PNG 16 bits | [4](#4-rendu-et-export) |
| `reparent` | change le parent d'un os en conservant la pose de repos | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `retarget` | copie l'animation entre rigs par paires de chaînes | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `safety` | rapport sur les flashs / flashs rouges / mouvement de caméra | [14](#14-assurance-qualité) |
| `script` | exécute un script Rhai en bac à sable qui modifie le projet | [6](#6-animation-clés-timeline-et-générateurs) |
| `scroll` | zones de défilement : add, list, remove, eval | [11](#11-machines-à-états-liaisons-de-données-conteneurs-et-zones-de-défilement) |
| `select-box` | sélection par rectangle | [3](#3-création-et-inspection-de-projets) |
| `shot` | enregistre un plan de caméra | [6](#6-animation-clés-timeline-et-générateurs) |
| `size-report` | où se trouvent les octets d'un projet/export | [13](#13-niveau-de-détail-rapports-de-taille-et-export-fit) |
| `skin` | skins nommés : ensembles de calques, os propres au skin, contraintes, chaînes IK | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `skin-diffuse` | poids de skin par diffusion de chaleur | [7](#7-physique-tissu-et-déformateurs) |
| `slider` | crée un os contrôleur qui mélange des poses enregistrées | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `snapshot` | historique numéroté des révisions d'un projet dans un dossier (save, list, restore) | [3](#3-création-et-inspection-de-projets) |
| `soft-scale` | mise à l'échelle proportionnelle d'une sélection | [7](#7-physique-tissu-et-déformateurs) |
| `sound` | pistes sonores du projet : list, add, edit, remove, mix | [8](#8-audio-synchronisation-labiale-et-sons) |
| `speech-motion` | hochements de tête et haussements de sourcils additifs cadencés sur les mots alignés | [8](#8-audio-synchronisation-labiale-et-sons) |
| `spring-fit` | ajuste la fréquence/l'amortissement d'un ressort sur une courbe | [6](#6-animation-clés-timeline-et-générateurs) |
| `state-bake` | précalcule une machine à états basée sur les images en clés | [11](#11-machines-à-états-liaisons-de-données-conteneurs-et-zones-de-défilement) |
| `stats` | décomptes d'os, de calques, de canaux, de clés (JSON) | [3](#3-création-et-inspection-de-projets) |
| `stretch` | précalcule un suiveur qui s'étire pour atteindre | [6](#6-animation-clés-timeline-et-générateurs) |
| `studio` | éditeur dans le navigateur (serveur web local) | [16](#16-studio-collaboration-serveur-dagent-mcp-et-registre-de-linterface) |
| `tag` | propose des noms de rôles standard pour les os et les calques | [3](#3-création-et-inspection-de-projets) |
| `take-apply` | applique un take de bibliothèque avec fondus enchaînés | [6](#6-animation-clés-timeline-et-générateurs) |
| `take-list` | liste les takes d'une bibliothèque | [6](#6-animation-clés-timeline-et-générateurs) |
| `take-record` | enregistre un take dans une bibliothèque | [6](#6-animation-clés-timeline-et-générateurs) |
| `timecode` | image <-> timecode SMPTE (gère le drop-frame) | [3](#3-création-et-inspection-de-projets) |
| `timeline` | pistes par canal avec clés, verrous et couleurs | [3](#3-création-et-inspection-de-projets) |
| `trace` | vectorise le canal alpha d'un PNG en un tracé vectoriel | [9](#9-import) |
| `trail` | trajectoire de mouvement d'un os, d'un calque ou d'un point | [12](#12-effets-profondeur-de-champ-onion-skin-trajectoires-et-caméra) |
| `trim` | précalcule un trim path dans un calque vectoriel | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `ui` | registre des commandes du studio, raccourcis, dispositions, recherche | [16](#16-studio-collaboration-serveur-dagent-mcp-et-registre-de-linterface) |
| `vcs` | gestion de versions avec fusion sémantique pour un fichier de projet | [15](#15-gestion-de-versions-vcs) |
| `view-swap` | pilote un calque de commutation à partir d'une propriété d'os (fonction en escalier) | [5](#5-rigging-skins-ik-et-aides-de-mise-en-page) |
| `walk` | génère un cycle de marche | [6](#6-animation-clés-timeline-et-générateurs) |
| `warp-pivot` | rotation/mise à l'échelle/déplacement d'une sélection de points de contrôle | [7](#7-physique-tissu-et-déformateurs) |
| `waveform` | pics min/max pour le dessin | [8](#8-audio-synchronisation-labiale-et-sons) |
