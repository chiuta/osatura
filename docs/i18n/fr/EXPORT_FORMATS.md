<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Formats d'export et d'import d'OSATURA

Ce que chaque format de fichier qu'OSATURA écrit ou lit conserve, approxime et abandonne, la commande qui le produit, et comment
vérifier le résultat. Ce texte est placé sous licence CC-BY-4.0. Version documentée : **osatura 0.1.0**.

Chaque ligne de commande ci-dessous a été exécutée sur le projet de démonstration intégré (`osatura demo demo.json`) ou sur une copie de 8 images
(`small.json`, le même projet avec `"frames": 8`). Les tailles citées proviennent de ce projet de 8 images en 640x480 et ne servent qu'à
s'orienter : elles évoluent avec le nombre d'images, d'ancres et de calques, et non avec une quantité fixe. Le détail des commandes (chaque option,
chaque erreur) se trouve dans [CLI_REFERENCE.md](CLI_REFERENCE.md) ; le format de fichier du projet lui-même est décrit dans
[PROJECT_FORMAT.md](PROJECT_FORMAT.md) ; l'état des codecs et des brevets figure dans [PATENT_NOTES.md](PATENT_NOTES.md) (ce n'est pas un avis juridique).

## Sommaire

1. [Quel format pour quelle tâche](#1-quel-format-pour-quelle-tâche)
2. [Aperçu des formats de sortie](#2-aperçu-des-formats-de-sortie)
3. [Images fixes : SVG, PNG, PNG 16 bits, OpenEXR](#3-images-fixes)
4. [Animation raster : planche de sprites PNG, GIF, APNG, WebM, MP4](#4-animation-raster)
5. [Lecteur HTML](#5-lecteur-html)
6. [Lottie (.json) et dotLottie (.lottie)](#6-lottie-json-et-dotlottie-lottie)
7. [glTF binaire (.glb)](#7-gltf-binaire-glb)
8. [Dossier Open Cel Animation (OCA)](#8-dossier-open-cel-animation-oca)
9. [Import : SVG, OpenRaster, Lottie, images, GIF/APNG](#9-import--svg-openraster-lottie-images-gifapng)
10. [Import : capture de mouvement BVH, MIDI et autres entrées en direct](#10-import--capture-de-mouvement-bvh-midi-et-autres-entrées-en-direct)
11. [Formats audio](#11-formats-audio)
12. [Matrice de prise en charge des fonctionnalités](#12-matrice-de-prise-en-charge-des-fonctionnalités)
13. [Réduire la taille des fichiers : export-fit, lod, size-report](#13-réduire-la-taille-des-fichiers)
14. [Prérequis ffmpeg](#14-prérequis-ffmpeg)
15. [Repères sur les brevets et les licences](#15-repères-sur-les-brevets-et-les-licences)
16. [Valider ce que vous avez écrit](#16-valider-ce-que-vous-avez-écrit)
17. [Différences avec les autres documents](#17-différences-avec-les-autres-documents)

---

## 1. Quel format pour quelle tâche

| Vous voulez... | Utilisez | Pourquoi |
|---|---|---|
| Le rendu exact, partagé dans un seul fichier, lisible dans tout navigateur, avec défilement temporel | `.html` | Images SVG intégrées dessinées par le moteur de rendu propre d'OSATURA ; rien n'est approximé. |
| Une petite animation vectorielle pour une page web ou une application dotée d'un lecteur Lottie | `.json` ou `.lottie` | Vectoriel, indépendant de la résolution, léger ; mais voir la liste des éléments abandonnés (images, traits à largeur variable, audio...). |
| Un aperçu en boucle pour une messagerie ou un README | `.gif` | Fonctionne partout ; 256 couleurs par image, transparence sur 1 bit, pas de son. |
| Un remplaçant du GIF avec toutes les couleurs et l'alpha | `.apng` | RGBA 8 bits sans perte ; plus volumineux que le GIF. |
| Une vidéo avec du son | `.mp4` (H.264 + AAC) ou `.webm` (VP9/AV1 + Opus) | Nécessite `ffmpeg`, à installer vous-même. `.webm` conserve la transparence. |
| Une planche de sprites pour un moteur de jeu | `.png` | Un PNG plus un atlas JSON avec les rectangles des tuiles. |
| Un squelette et un mouvement pour un outil 3D | `.glb` | Os, animation, formes plates sous forme de meshes triangulaires. |
| Des images pour des outils de compositing ou de peinture | `export-oca` (Open Cel Animation) ou `frames --png` | Cels en PNG, dans un seul dossier. |
| Des images fixes en lumière linéaire ou à grande profondeur de bits | `render-hdr` (`.exr` ou `.png` 16 bits) | Moteur de rendu en virgule flottante, OpenEXR 16 bits, demi-précision ou flottant. |
| Une seule image fixe | `render` (`.svg` ou `.png`) | Vectoriel ou raster 8 bits. |

## 2. Aperçu des formats de sortie

| Format | Commande | Extension | Nécessite ffmpeg | Alpha | Audio | Taille pour la démo de 8 images* |
|---|---|---|---|---|---|---|
| SVG (une image) | `render` | `.svg` | non | oui | non | quelques Ko |
| Séquence SVG/PNG | `frames` | dossier | non | oui | non | un fichier par image |
| PNG (une image) | `render` | `.png` | non | oui (8 bits) | non | |
| PNG 16 bits | `render-hdr` | `.png` | non | oui | non | |
| OpenEXR (demi/flottant) | `render-hdr` | `.exr` | non | oui | non | 466 Ko en demi, 923 Ko en flottant (640x480) |
| Planche de sprites PNG + JSON | `export` | `.png` (+ `.atlas.json`) | non | oui | non | |
| GIF | `export` | `.gif` | non | 1 bit | non | 83 Ko |
| APNG | `export` | `.apng` | non | oui | non | 131 Ko |
| WebM | `export` | `.webm` | oui | oui | oui (Opus) | 10,5 Ko |
| MP4 | `export` | `.mp4` | oui | non | oui (AAC) | 8,9 Ko |
| Lecteur HTML | `export` | `.html`, `.htm` | uniquement pour l'audio | oui | mp3 intégré | 66 Ko |
| Lottie | `export` | `.json` | non | oui | non | 47 Ko |
| dotLottie | `export` | `.lottie` | non | oui | non | 11,5 Ko |
| glTF binaire | `export` | `.glb` | non | oui | non | 115 Ko |
| Open Cel Animation | `export-oca` | dossier | non | oui | non | un PNG par cel |

\* GIF, APNG, WebM et MP4 avec `--scale 0.5` (320x240) ; formats vectoriels en taille réelle. « Nécessite ffmpeg » signifie un
`ffmpeg` fonctionnel dans le `PATH` (section 14). Les noms de sortie sont choisis d'après l'extension, sans distinction de casse ; une extension inconnue est refusée et
rien n'est écrit (`unknown export format '.xyz' (use .gif .html .json .mp4 .apng .webm .png .glb)`, et `.lottie` est également
accepté).

Tous les écrivains refusent d'écraser le projet d'entrée avec la sortie, écrivent via un fichier temporaire qu'ils renomment ensuite, et refusent
les liens symboliques sauf si `--follow-symlinks` est fourni (CLI_REFERENCE section 1.4). Le garde-fou de taille des exporteurs vectoriels
(Lottie, HTML, GLB) refuse un export de plus d'environ 40 millions de valeurs (images x ancres, plus os et calques par image)
avec `export would write about N million values ...`.

## 3. Images fixes

### SVG (`render ... out.svg`, `frames`)

```
$ osatura render demo.json 12 f12.svg --bones
wrote f12.svg
$ osatura frames demo.json fr            # fr/frame_0000.svg ... frame_0047.svg
```
La sortie vectorielle du moteur de rendu propre d'OSATURA : les chemins, dégradés, traits avec profils de largeur, tirets, modes de fusion, masques et effets de calque
sont écrits en SVG. `--bones` dessine la surcouche du squelette. Pour `.svg`, `--scale` laisse la viewBox du document inchangée. **Éléments abandonnés/limites connus :** le moteur de rendu n'en signale aucun ; la façon dont un visualiseur SVG donné traite les modes de fusion et les filtres dépend
de ce visualiseur. `--proxy N` écrit un proxy de défilement simplifié qui abandonne les traits fins, les dégradés et les modes de fusion (pas destiné aux
livrables). **Validation :** ouvrez-le dans un navigateur, ou `python3 -c "import xml.dom.minidom,sys; xml.dom.minidom.parse(sys.argv[1])" f12.svg`.

### PNG (`render ... out.png`, `frames --png`)

RGBA 8 bits, sRGB, rendu par le rastériseur propre du moteur. `--scale N` dans (0, 16] multiplie la taille en pixels. Un projet avec
`"background": null` est rendu transparent là où rien n'est dessiné. Par défaut, sans gestion des couleurs : les valeurs se mélangent dans l'espace encodé en sRGB.
Depuis la vague 7, un projet peut opter pour une composition en lumière linéaire (`osatura color-space p.json out.json linear`, champ `color_space`) :
les sorties raster, GIF, APNG, vidéo et `render-hdr` sont alors composées en lumière linéaire f32 puis converties une seule fois en sRGB ; SVG et
Lottie ne changent pas (LIMITATIONS L-22).

### PNG 16 bits et OpenEXR (`render-hdr`)

```
$ osatura render-hdr small.json 3 f.exr                           # HALF, 640x480, 465,611 bytes
$ osatura render-hdr small.json 3 f2.exr linear --precision float # FLOAT, 923,211 bytes
$ osatura render-hdr small.json 3 f4.png                          # 16-bit/colour RGBA, sRGB-encoded
```
* **OpenEXR :** une seule partie, par lignes de balayage, **non compressé**, canaux `A B G R`, lumière linéaire, alpha prémultiplié. `HALF` par défaut,
  `FLOAT` avec `--precision float` (ou `--float32`). Le mot `linear` n'a aucun effet pour l'EXR (toujours linéaire).
* **PNG 16 bits :** encodé en sRGB sauf si le mot `linear` est donné.
* **Fidélité :** le moteur de rendu en virgule flottante offre plus de précision que celui en 8 bits, mais l'illustration est en couleur sRGB ordinaire ; le fichier
  contient donc des valeurs de la plage d'affichage à plus haute précision, et non une lumière HDR relative à la scène. L'arrière-plan du projet n'est pas dessiné.
* **Éléments abandonnés/limites connus :** une image par fichier ; pas de compression, de calques ni d'AOV ; pas de profil ICC ; `--8bit` se rabat sur le
  rastériseur 8 bits comme source.
* **Validation :** `ffprobe f.exr` indique `exr 640x480 gbrapf32le` ; tout visualiseur EXR l'ouvre.

## 4. Animation raster

Tous ces formats font d'abord passer chaque image par le moteur raster. Un projet avec `"background": null` est rendu avec des pixels transparents
partout où rien n'est dessiné ; la démo a une couleur d'arrière-plan (`#f6f1e7`) et est donc opaque. `--scale N` dans (0, 16]
s'applique à `.gif .apng .webm .mp4 .png` ; sur les autres formats, il est refusé (`unknown option --scale for 'export .json'`).

### Planche de sprites PNG (`export ... out.png`)

```
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.atlas.json
wrote out3.png                    # 4 columns x 2 rows of 160x120 tiles
```
Un PNG avec toutes les images sous forme de tuiles (`--columns N`, par défaut une grille presque carrée : ceil(sqrt(frames)) colonnes) et un atlas
`out3.atlas.json` (le nom de sortie dont l'extension est remplacée par `.atlas.json`) listant `{index, x, y, w, h}` par image. L'alpha est conservé.
L'atlas s'appelait auparavant `out3.json` et pouvait écraser silencieusement un fichier Lottie de même nom de base ; le nouveau nom évite cela, et le chemin de l'atlas passe par les mêmes garde-fous d'entrée et de liens symboliques.

### GIF (`export ... out.gif`)

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif                      # 8 frames, 83 KB
```
* Une image par image source, **en boucle infinie**, une palette locale d'au plus 256 couleurs par image (quantifiée, sans tramage).
* **Durées :** les délais GIF sont des centièmes de seconde entiers, et les lecteurs traitent tout ce qui est inférieur à 2 cs comme 10 cs ; 50 fps est donc le véritable maximum.
  Jusqu'à 50 fps, les délais sont distribués par diffusion d'erreur (29,97 fps alterne 3 et 4 cs) de sorte que la durée totale reste à 1 cs de
  `frames / fps`. Au-delà de 50 fps, la séquence est **décimée à 50 fps**, la durée est conservée et un avertissement est affiché
  (`GIF cannot hold 60 fps ... use APNG, WebM or MP4 for the full rate`).
* **Transparence :** 1 bit (l'indicateur de couleur transparente est positionné) ; les pixels semi-transparents ne sont pas conservés.
* **Éléments abandonnés :** pas d'audio (`note: GIF cannot carry audio; ...` lorsque le projet comporte des sons) ; pas d'alpha partiel ; les images plus larges
  ou plus hautes que 65535 pixels sont refusées (`image too large for GIF`).
* **`--report r.json` :** écrit la liste des éléments que le format n'a pas pu transporter (ici : la décimation de la fréquence d'images, le cas échéant).
* **Conseils de taille :** un `--scale` plus petit, moins d'images, ou `export-fit --format gif` (section 13). Le GIF est le moins efficace des formats raster
  pour les illustrations photographiques ou riches en dégradés.
* **Validation :** `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` affiche le
  nombre d'images (8).

### APNG (`export ... out.apng`)

PNG animé, RGBA 8 bits, sans perte, une image par image du projet à la fréquence du projet (0,0417 s par image à 24 fps). L'alpha complet est conservé. Pas d'audio. Plus volumineux
que le GIF pour des illustrations plates (131 Ko contre 83 Ko dans l'exemple) car il n'y a pas de palette. La plupart des navigateurs lisent l'APNG ; certains outils non,
auquel cas utilisez WebM ou GIF. **Validation :** `ffprobe` avec `-count_frames` (8 images pour l'exemple).

### WebM (`export ... out.webm`) et MP4 (`export ... out.mp4`)

```
$ osatura export small.json out.mp4  --scale 0.5     # h264, yuv420p, 320x240, 8 frames
$ osatura export small.json out.webm --scale 0.5     # vp9, yuv420p + alpha_mode=1
$ osatura export small.json out.webm --codec av1     # needs an ffmpeg with libaom-av1
```
* **MP4 :** H.264 (`libx264`), `yuv420p`, **sans alpha**. Audio AAC lorsque le projet comporte des pistes sonores.
* **WebM :** VP9 (`libvpx-vp9`, par défaut) ou AV1 (`libaom-av1`, `--codec av1`) ; le canal alpha est conservé (`yuva420p`, balise de conteneur
  `alpha_mode=1`) ; audio Opus (`libopus`) lorsque le projet comporte des sons. Toute autre valeur de `--codec` : `unknown --codec 'foo' (vp9 or av1)`.
* **Son :** les pistes sonores du projet sont mixées (début, gain, rognage, fondus ; coupées à la longueur du projet) puis multiplexées.
* **Nécessite ffmpeg :** les images sont rendues en PNG dans un répertoire temporaire privé, confiées à `ffmpeg`, puis supprimées. Si
  `ffmpeg` est absent : `cannot run ffmpeg (...); install it or export a GIF/HTML instead` (code de sortie 1, rien n'est écrit). S'il ne dispose pas
  de l'encodeur : `ffmpeg failed`.
* **Limites connues :** 8 bits uniquement ; la taille de la vidéo suit celle du projet et `--scale` (vérifiez le résultat avec `ffprobe`).
  `dof-export` écrit aussi `.mp4`/`.webm`, sans audio.
* **Validation :** `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4`
  a affiché `h264,320,240,yuv420p,24/1,8`.
* **Brevets :** voir la section 15 ; H.264 et AAC sont couverts par des pools de licences, VP9/AV1 n'ont pas été évalués.

## 5. Lecteur HTML

```
$ osatura export small.json out.html
wrote out.html                     # 66 KB for 8 frames
```
Un fichier HTML autonome : **chaque image en SVG intégré**, un bouton lecture/pause, un curseur de défilement (champ range) et une étiquette d'image,
piloté par un seul `<script>` intégré. Il ne fait **aucune requête réseau** : la seule URL du fichier est l'espace de noms SVG. Pour l'ouvrir, il ne faut
rien d'autre qu'un navigateur. L'image est exactement ce que produit le moteur de rendu SVG ; c'est donc le plus fidèle des formats
« lisibles » (masques, modes de fusion, effets, os, IK, déformateurs, particules sont tous intégrés aux images).

* **Audio :** si le projet comporte des sons, l'audio mixé est intégré sous forme d'URI de données mp3 (nécessite `ffmpeg` ; avertissement au-delà de 3 MiB, l'audio est
  abandonné avec un avertissement au-delà de 16 MiB).
* **Taille :** croît linéairement avec le nombre d'images (environ 7,7 Ko par image ici) et avec le détail vectoriel ; le fichier n'est pas compressé. Conseils
  habituels : le servir compressé en gzip (l'exemple de 66 Ko pèse environ 9 Ko en gzip), réduire le nombre d'images ou le détail des chemins avec `lod`, et utiliser
  `export-fit --format html`.
* **Limites :** `--report` est refusé (`unknown option --report for 'export .html'`) ;
  le garde-fou de taille de l'export vectoriel s'applique ; le lecteur n'offre que lecture/pause et un curseur de défilement.
* **Validation :** ouvrez-le ; `grep -c '<svg' out.html` compte les images ; `grep -o 'http[^"<]*' out.html` ne doit lister que l'espace de noms SVG.

## 6. Lottie (.json) et dotLottie (.lottie)

```
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ osatura export small.json out.lottie       # a zip: manifest.json + a/animation.json
```

### Comment il est construit

Le fichier est du Bodymovin (`"v": "5.7.0"`) avec `fr` = fréquence d'images du projet, `ip` = 0, `op` = nombre d'images et `w`/`h` = la taille de la scène. **Chaque
calque vectoriel devient un calque de forme dont la géométrie de chemin est précalculée à chaque image dans l'espace monde (images clés de maintien)** ; les os, l'IK,
le skinning, les déformateurs, les ressorts, les particules, la caméra (parallaxe comprise) et les transformations des parents sont donc tous *dans* la géométrie. Le résultat
se lit comme l'original, mais ce n'est pas un rig modifiable : il n'y a ni os, ni canaux, ni expressions. Un calque d'arrière-plan uni est
écrit lorsque le projet a une couleur d'arrière-plan. Par conséquent, le fichier croît avec `frames x anchors` ; il est bien plus volumineux qu'un
Lottie construit à la main avec une animation de transformation (47 Ko pour la démo de 8 images, 11,5 Ko à l'intérieur d'un `.lottie`).

### Exporté

Remplissages et traits unis et en dégradé (arrêts de dégradé avec alpha ; les points d'extrémité du dégradé suivent la transformation animée du calque), motifs de
tirets, terminaisons, jointures et limite de mitre, largeurs de trait mises à l'échelle du calque, modes de fusion, opacité du calque, track mattes
alpha et alpha inversé pour les masques de calque (une copie de la source du masque sert de matte), effets de calque (flou, ombre portée, teinte, remplissage) sur
les calques à feuille unique, et **opacité de groupe sous forme de précomposition** afin que les enfants qui se chevauchent ne transparaissent pas les uns à travers les autres. Les nombres sont
écrits sous leur forme décimale la plus courte.

### Signalé par un avertissement (jamais abandonné silencieusement)

Chaque élément ci-dessous est affiché sur stderr sous la forme `warning: [code] layer: message` et écrit dans `--report FILE` sous la forme
`{"format":"lottie","count":N,"warnings":[{"code","layer","message"}]}` :

| Code | Signification |
|---|---|
| `image-layer` | Les calques d'image ne sont pas exportés (la sortie Lottie ne contient que des calques vectoriels). |
| `variable-width-stroke` | Les profils de largeur ne sont pas pris en charge ; la largeur de base constante est utilisée. |
| `stroke-anisotropic` | Échelle non uniforme ou cisaillement sur le calque ; les traits Lottie sont uniformes, la largeur est donc mise à l'échelle par la moyenne géométrique. |
| `gradient-anisotropic` | Un dégradé sur un tel calque est approximé (les dégradés Lottie ne sont pas cisaillés). |
| `blend-add` | Le mode de fusion `add` n'a pas d'équivalent Lottie ; `screen` est utilisé. |
| `group-blend` | Un mode de fusion sur un groupe n'est pas exporté. |
| `group-mask` | Un masque sur un groupe n'est pas exporté (les masques sur des calques simples le sont). |
| `group-effect` | Les effets de calque sur un groupe (de plusieurs calques) ne sont pas exportés. |
| `mask-source-not-vector` | La source du masque n'est pas un calque vectoriel, le masque n'est donc pas exporté. |
| `depth-of-field` | Les canaux `camera_focus`/`camera_blur` ne sont pas appliqués ; utilisez `dof-export`. |
| `audio` | Lottie ne peut pas transporter d'audio ; N piste(s) sonore(s) ne sont pas incluses. |

### Abandonné faute d'équivalent Lottie

Os, IK, canaux, machines à états, liaisons de données, conteneurs, régions de défilement, markers/events, sons, chaînes de post-effets (`fx`),
normal maps, et la structure modifiable du rig. Les couleurs d'effet par calque ne peuvent pas être animées (la première valeur est utilisée) et
les tailles sont en unités de scène (LIMITATIONS L-23).

### dotLottie (`.lottie`)

Un zip [dotLottie 2.0](https://dotlottie.io/spec/2.0/) contenant `manifest.json` (`{"animations":[{"id":"animation"}],"generator":"OSATURA","version":"2"}`) et
`a/animation.json` (le même Lottie que ci-dessus, compressé par deflate : 46 843 octets sont devenus un fichier de 11 515 octets). `--report` fonctionne de la même
manière. **Importer** un `.lottie` n'est pas pris en charge (`import-lottie` a besoin du JSON ; décompressez d'abord `a/animation.json`). Jusqu'à v0.1.0-alpha.4, le conteneur était du dotLottie 1.0 (`animations/animation.json`, `autoplay`/`loop` dans le manifeste) ; la disposition 2.0 est lue par les lecteurs dotLottie actuels (vérifié avec @lottiefiles/dotlottie-web).

### Validation et réserves

* `python3 -m json.tool out.json > /dev/null` pour la bonne formation ; `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],len(d['layers']))"`.
* `unzip -l out.lottie` doit montrer exactement les deux entrées.
* Aller-retour : `osatura import-lottie out.json back.json` suivi de `osatura check back.json` vérifie qu'OSATURA peut relire sa propre
  sortie (l'aller-retour est avec perte : le rig a disparu).
* **Aucun lecteur Lottie indépendant n'était disponible** lors de la rédaction (LIMITATIONS L-28) : la sortie a été vérifiée par rapport à la disposition des champs Lottie
  et à l'importeur propre d'OSATURA. Testez dans votre lecteur cible (lottie-web, lecteur dotLottie, Skottie, ...) et lisez les
  avertissements ; si l'exactitude compte, utilisez `.html` ou la vidéo.
* **Conseils de taille :** `export-fit --format lottie` explore quatre paramètres (`decimals`, `path_eps`, `key_tol`, `frame_step`) par rapport à un budget d'octets ;
  `lod` réduit les ancres avant l'export ; `size-report` montre la taille du Lottie précalculé par calque (section 13).
* **Brevets :** Lottie et dotLottie sont des formats ouverts ; rien dans PATENT_NOTES.md ne les concerne.

## 7. glTF binaire (.glb)

```
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
```
glTF 2.0 binaire (magic `glTF`, version 2, `"generator": "osatura"`), fonctionnalités de base uniquement. Le projet doit comporter des os
(`the project has no bones to export` sinon).

* **Exact :** la hiérarchie des os et son animation (échantillonnée à chaque image, interpolation linéaire, cisaillement abandonné).
* **Les remplissages** deviennent des meshes triangulaires plats (découpage d'oreilles). Les formes sont rigides (un calque avec un os suit ce nœud d'os) ou **skinnées**
  (JOINTS_0/WEIGHTS_0, 4 influences par sommet) lorsque chaque ancre porte des poids d'os.
* **Les traits** sont triangulés en meshes distincts (largeur, profil de largeur, terminaisons plate/ronde/carrée, jointures en onglet/arrondie/biseautée, tirets) ;
  les traits translucides peuvent paraître plus sombres là où les triangles se chevauchent.
* **Les dégradés** (linéaires et radiaux) deviennent un petit PNG généré servant de texture de couleur de base ; propagation en pad (clamp).
* **Les masques** sont précalculés en découpant les triangles par le remplissage du calque de masque, uniquement pour une source de masque vectorielle avec des remplissages opaques et sans
  traits, et uniquement pour la forme du masque à l'image 0. **Les calques d'image** deviennent des quads texturés ou des meshes de déformation avec PNG intégré.
* **Les morphs de chemin** deviennent des morph targets avec des poids animés lorsque la vérification image par image reproduit la géométrie du moteur ; sinon
  le chemin reste une forme statique de l'image 0 et la raison est signalée.
* **Non exportés (signalés par des avertissements `[glb-limit]`) :** modes de fusion autres que normal (dessinés comme normal), opacité de calque animée (figée à
  l'image 0), animation de transformation de calque hors os, effets de calque, déformateurs au-delà de leur forme à l'image 0, chemins animés sans morph
  valide, formes de masque animées, son.
* **Coordonnées :** y est inversé (glTF a l'axe y vers le haut) ; les formes sont étagées avec un petit décalage en z (0,01 par forme) pour l'ordre de dessin.
* **Validation :** l'en-tête est `glTF`, version 2, et le champ de longueur est égal à la taille du fichier (commande `python3` d'une ligne à la section 16). Utilisez un validateur
  ou un visualiseur glTF de votre choix pour la vérification complète ; aucun n'est fourni et aucun n'a été exécuté pour ce document. L'exemple ci-dessus comporte 22 nœuds,
  15 meshes, 1 skin, 1 animation et 7 images intégrées.
* **Conseils de taille :** moins d'ancres vectorielles (`lod`), pas de dégradés (chacun est une texture), moins d'images (l'animation est échantillonnée à chaque image).
* **Brevets :** format ouvert ; aucune mention dans PATENT_NOTES.md.

## 8. Dossier Open Cel Animation (OCA)

```
$ osatura export-oca small.json oca1                 # one layer "render" with all composited frames
$ osatura export-oca small.json oca2 --layers        # one OCA layer per top-level layer
wrote oca2/small.oca
```
Écrit `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) ainsi que des cels PNG `layerN/fNNNN.png`. Sans `--layers`, il y a un seul calque
`render` avec les images composées ; avec `--layers`, chaque calque de premier niveau est rendu seul sur fond transparent (11 calques pour la démo).
`--scale N` multiplie la taille des cels. **Éléments abandonnés connus :** les cels sont des rasters (ni vecteurs, ni os, ni canaux ne survivent) ; l'opacité et la fusion des calques
ne sont pas écrites (chaque calque est un `paintlayer`, d'opacité 1) ; les sons ne sont pas exportés ; les fichiers d'un dossier existant sont écrasés.
**Import :** il n'existe pas de commande `import-oca` dans la CLI. **Validation :** le fichier `.oca` est du JSON (`python3 -m json.tool`) ; chaque cel listé
doit exister sous forme de PNG.

## 9. Import : SVG, OpenRaster, Lottie, images, GIF/APNG

Les imports créent un **nouveau** projet (ils ne fusionnent pas dans un projet existant) sauf indication contraire ; utilisez `script` ou modifiez le JSON pour les combiner.

### SVG (`import-svg`)

```
$ osatura import-svg art.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
wrote svg.json (3 paths)
```
Sous-ensemble : `path` (toutes les commandes), `rect` (coins arrondis ignorés), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` avec `transform`,
`fill` uni, `stroke`, `stroke-width`. Résultat : 24 fps, 24 images, taille tirée de `viewBox` ou de `width`/`height` (sinon 512x512), **un calque
vectoriel `svg`**. Ignorés avec une ligne `warning:` (affichée sur **stdout**) : `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`,
`<linearGradient>`/`<radialGradient>` (une peinture en dégradé retombe sur du gris), `<image>`, `<use>`, `<foreignObject>`. Limites : 8 Mo de SVG,
200 000 ancres. Pour les photographies et les rasters, utilisez `trace` / `autotrace` (CLI_REFERENCE section 9).

### OpenRaster (`import-ora`)

```
$ osatura import-ora art.ora ora.json --fps 12 --frames 24
```
Un **calque d'image par calque raster** (PNG intégré), ordre de bas en haut, calques masqués conservés mais invisibles, groupes aplatis, opacité,
visibilité et position par calque, et les opérations de composition `svg:` courantes (multiply, screen, overlay, darken, lighten, color-dodge,
color-burn, hard-light, soft-light, difference, exclusion, plus converti en add). L'opacité d'une pile imbriquée se multiplie dans ses calques et son mode de
fusion est hérité, mais les calques qui se chevauchent à l'intérieur d'une pile translucide ne sont **pas** aplatis au préalable. Les noms en double reçoivent `_1`, `_2`.
Limites : 2000 entrées d'archive, 128 MiB par entrée, calques de 64 Mpixels. Un fichier qui n'est pas un zip : `not a zip/ORA file: ...` (code de sortie 1). Il n'y a pas d'export
ORA. L'import PSD n'est pas implémenté (LIMITATIONS L-32).

### Lottie (`import-lottie`)

```
$ osatura import-lottie in.json out.json [fx.json]
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
```
Un **sous-ensemble** volontaire ; chaque fonctionnalité ignorée donne une ligne `warning:`.
* **Importé :** calques de forme avec chemins `sh`, `rc`, `el` ; remplissages unis et en dégradé (les points d'extrémité/centre/rayon de dégradé animés deviennent des
  canaux `grad_dx`/`grad_dy`) ; traits ; groupes ; transformations de calque (position, échelle, rotation, opacité statiques ou à images clés ; les chemins de position
  courbes sont précalculés à chaque image) ; modes de fusion ; un calque d'arrière-plan uni ; calques d'image avec URI de données PNG/JPEG intégrées ; calques nuls comme parents
  (leurs transformations sont précalculées dans les enfants) ; masques de calque (add/subtract/intersect avec opacité, inversion, formes animées) ; mattes de luminance
  (approximés) ; précompositions ; plages d'entrée/sortie des calques (sous forme de piste d'opacité en escalier) ; interpolation des images clés (hold devient step, les poignées de Bézier deviennent
  cubic-bezier, aucune devient linear).
* **Chemins animés :** les clés de forme de même topologie (même nombre d'ancres et même état ouvert/fermé) deviennent un chemin de base plus un **morph target**
  par clé distincte supplémentaire (`k1`, `k2`, ...) avec des canaux de poids `path_morph` portant l'interpolation. Une topologie différente utilise la première forme
  et affiche un avertissement.
* **Effets (`ef`) :** le flou gaussien, l'ombre portée, la teinte et le remplissage deviennent des **effets par calque** avec des nombres animés sous forme de canaux `fx_*` ;
  la teinte/le remplissage/le tritone/les niveaux statiques sur des calques de forme sont précalculés dans les couleurs. Les couleurs d'effet ne peuvent pas être animées (la première valeur est utilisée) ; les autres
  effets donnent des avertissements.
* **Ignorés avec un avertissement :** texte et autres types de calques, assets audio, points d'ancrage animés (première valeur utilisée), chemins de mouvement courbes sur
  des propriétés autres que la position, masques sur une source de track matte, chaînes de parents trop longues à précalculer, plus de 2000 calques, animation de plus
  de 20 000 images.
* `fx.json` est conservé pour la compatibilité : un tableau vide est écrit.
* **Non pris en charge :** entrée zip `.lottie`. **Piège :** il n'y a pas de garde-fou d'écrasement contre l'entrée : `import-lottie in.json in.json` remplace le
  Lottie par le projet.

### Images, GIF et APNG comme calques image par image

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20      # folder of PNGs
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0                    # animated GIF/APNG
```
`import-seq` prend tous les PNG d'un dossier dans l'ordre des noms de fichiers (tous de même taille) ; `import-anim` lit un GIF ou un APNG animé en utilisant les
délais propres au fichier (un PNG simple donne une image). Les deux ajoutent un calque switch avec une image intégrée par image plus un canal en escalier ; ils travaillent sur un projet
**existant**. Un long GIF alourdit le projet (chaque image est un PNG intégré). `trace` et `autotrace` transforment des bitmaps en chemins
vectoriels (`trace` lit le canal alpha ; `autotrace` quantifie les couleurs et ajuste des courbes de Bézier).

## 10. Import : capture de mouvement BVH, MIDI et autres entrées en direct

### BVH (`import-bvh`, `mocap-info`, `mocap-match`, `mocap-retarget`)

```
$ osatura mocap-info walk.bvh
$ osatura import-bvh demo.json walk.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```
`import-bvh` associe des canaux de rotation d'articulations choisis à des os sous forme de canaux `bone_angle` à clés, échantillonnés à la fréquence et au nombre d'images du projet
(`--axis` sélectionne `Zrotation`, `Xrotation` ou `Yrotation` ; `--tol` simplifie les clés). `mocap-match` propose une correspondance articulation-os avec un tableau de
confiance et `mocap-retarget` précalcule des canaux ajoutés avec mouvement de la racine, une vue (`front|side|top|yaw=DEG`) et une échelle. Le sous-ensemble texte
BVH uniquement : pas de formats de capture de mouvement binaires, pas de FBX (par conception). **Limites (LIMITATIONS L-45) :** aucun fichier de capture de mouvement réel n'a été testé ; la projection
2D perd la profondeur ; la torsion autour d'un axe d'os, les doigts et le visage sont ignorés ; le rig conserve les longueurs de ses os, le glissement des pieds n'est donc que
réduit.

### MIDI et autres entrées en direct (`live-map`)

```
$ osatura live-map small.json song.mid mapping.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote out.json
```
Un **fichier MIDI standard** (`.mid` ; messages MIDI 1.0, running status, méta-événements de tempo, division en ticks ou SMPTE) ou un JSON d'événements
(`{"events":[{"source","id","value","time"}]}`, valeur normalisée 0..1, temps en secondes) est transformé en canaux à clés, événements et takes au moyen d'un
fichier de correspondance. Sources : `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`. Au plus
3 000 000 d'événements ; temps dans une limite de 7 jours. Rien ne dialogue avec un périphérique MIDI depuis la CLI ; le studio utilise le Web MIDI du navigateur. Le MIDI est en **import
uniquement** (il n'y a pas d'export MIDI). Les fichiers mal formés renvoient une erreur, jamais un plantage.

## 11. Formats audio

| Usage | Formats | Remarques |
|---|---|---|
| Analyse (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) | **WAV** uniquement | PCM entier 8/16/24/32 bits ou flottant 32 bits, tout nombre de canaux mixé en mono, jusqu'à 256 MiB. Convertissez d'abord le MP3/OGG (`ffmpeg -i in.mp3 out.wav`). |
| Pistes sonores du projet (`sound add`) | WAV, OGG, MP3, tout ce que `ffprobe` accepte | Stockées comme une **référence** (chemin relatif, image de départ, gain 0..4, fenêtre de rognage, fondus) ; rien n'est intégré ; le fichier doit rester dans le dossier du projet sauf avec `--allow-external-assets`. |
| Sortie mixée (`sound mix`) | `.wav` (PCM 44,1 kHz 16 bits), `.ogg` (Vorbis q4), `.mp3` (96 kb/s) | Mixée via ffmpeg pour la **durée du projet** ; `.flac` et autres : `unknown audio format '.flac' (use .wav, .ogg or .mp3)`. |
| Dans les exports | `.mp4` AAC, `.webm` Opus, `.html` mp3 intégré | `.gif`/`.apng` restent muets avec une note ; Lottie avertit `[audio]` ; GLB et OCA abandonnent le son. |
| Repères de bouche (`mouth`) | Texte Papagayo/Moho-switch, TSV Rhubarb | Pas de l'audio : des fichiers de repères. |

Encodeurs utilisés : `libmp3lame` (mp3), `libvorbis` (ogg), `aac`, `libopus`. Pas de panoramique/égaliseur/ducking par piste ni de bouclage (ROADMAP section 9).

## 12. Matrice de prise en charge des fonctionnalités

Les lignes sont ce qu'un projet peut contenir, les colonnes les formats de sortie. **yes** = transporté tel que dans le projet, **approx** = transporté avec une
différence visible, **baked** = présent uniquement sous forme de résultats image par image, **no** = abandonné. Les cellules des formats que je n'ai pas pu vérifier dans un visualiseur indépendant
décrivent ce que l'exporteur écrit.

| Fonctionnalité | SVG / PNG | Planche de sprites / APNG | GIF | WebM / MP4 | HTML | Lottie / .lottie | GLB | OCA |
|---|---|---|---|---|---|---|---|---|
| Formes vectorielles, remplissages | yes | baked (raster) | baked (raster) | baked (raster) | yes (SVG) | yes | approx (triangles) | baked (raster) |
| Dégradés | yes | baked | baked | baked | yes | yes (approx si le calque est cisaillé) | yes (texture) | baked |
| Traits avec profil de largeur | yes | baked | baked | baked | yes | approx (largeur constante) | yes | baked |
| Tirets, terminaisons, jointures | yes | baked | baked | baked | yes | yes | yes (triangulé) | baked |
| Calques d'image (bitmap) | yes | baked | baked | baked | yes | no (`image-layer`) | yes | baked |
| Os et animation | baked | baked | baked | baked | baked | baked | **yes (exact)** | baked |
| Skinning / déformateurs | baked | baked | baked | baked | baked | baked | skinné ; déformateurs à l'image 0 | baked |
| Caméra et parallaxe | baked | baked | baked | baked | baked | baked (les mouvements de caméra sont dans la géométrie) | no (les formes viennent de l'image 0) | baked |
| Modes de fusion | yes | baked | baked | baked | yes | approx (`add` devient `screen` ; aucun sur les groupes) | no (normal) | baked |
| Masques de calque | yes | baked | baked | baked | yes | calques simples uniquement | forme de l'image 0, restreinte | baked |
| Effets de calque (flou, ombre...) | yes | baked | baked | baked | yes | calques à feuille unique | no | baked |
| Opacité animée | baked | baked | baked | baked | baked | yes | no (figée à l'image 0) | baked |
| Morphs de chemin | baked | baked | baked | baked | baked | baked | morph targets (si valides) | baked |
| Profondeur de champ | via `dof-export` | via `dof-export` | via `dof-export` | via `dof-export` | no | no (avertissement) | no | no |
| Chaîne de post-effets (`fx`) | PNG uniquement | no | no | no | no | no | no | no |
| Transparence | yes | yes (alpha) | 1 bit | WebM yes, MP4 no | yes | yes | yes | yes |
| Audio | no | no | no | **yes** | yes (mp3) | no (avertissement) | no | no |
| Bouclage | n/a | planche : n/a, APNG : yes | yes | selon le lecteur | lecteur | lecteur (dotLottie 2.0 laisse la boucle au lecteur ou à une machine à états) | n/a | n/a |
| Rig/canaux modifiables ensuite | no | no | no | no | no | no | os uniquement | no |
| Machines à états, liaisons, conteneurs, scroll | no | no | no | no | no | no | no | no |
| Indépendant de la résolution | yes (SVG) | no | no | no | yes | yes | yes | no |
| Nécessite ffmpeg | no | no | no | **yes** | audio uniquement | no | no | no |

« Baked » signifie que l'effet de la fonctionnalité est dans les pixels ou dans la géométrie image par image, mais que la fonctionnalité elle-même ne peut plus être modifiée ensuite.

## 13. Réduire la taille des fichiers

1. **Voir où sont les octets :** `osatura size-report project.json` liste le fichier de projet, le Lottie précalculé et la page HTML par calque,
   image et canal (`--json` pour les scripts).
2. **Respecter un budget automatiquement :** `osatura export-fit project.json out.json --max-size 50KB --format lottie` explore les paramètres de l'exporteur et
   conserve le meilleur SSIM qui tient dans le budget. Formats : `lottie`, `html`, `gif`, `apng`, `webm`, `project` (un JSON de projet allégé). `--measure gzip` applique le budget à la
   taille gzip, `--min-ssim X` fixe un seuil de qualité, `--max-seconds N` est une vraie limite de temps réel (le meilleur résultat obtenu jusque-là est conservé). Code de sortie 1 avec le
   plus petit résultat lorsque le budget ne peut pas être tenu.
3. **Réduire d'abord l'illustration :** `osatura lod project.json --tolerance-px 1 --levels 3 --out-dir lv --verify` écrit des projets de niveau de détail avec une erreur
   déclarée (et, avec `--verify`, mesurée) en pixels de sortie ; `lod-pick --screen-px WxH` choisit le niveau le plus grossier pour un écran donné.
4. **Par format :**
   * Lottie/HTML/GLB croissent avec `frames x anchors` : moins d'images, moins d'ancres, une fréquence d'images plus basse, des plages de clés rognées (outils `keys`).
   * GIF/APNG croissent avec pixels x images : `--scale 0.5` est une réduction d'un facteur 4 ; le GIF convient aux illustrations plates, l'APNG à l'alpha, WebM/MP4 à tout ce qui comporte du mouvement.
   * `.lottie` est le même Lottie compressé par deflate (environ 4 fois plus petit dans l'exemple) ; servez `.html`/`.json` compressés en gzip.
   * Les images d'un projet sont intégrées en PNG base64 : réduisez-les avant l'import.
5. Limites : un Lottie d'un rig de 14 calques ne peut pas être rendu arbitrairement petit (un budget de 1 Ko sur l'exemple indique « every knob is at its floor ») ; la
   recherche prend du temps sur les gros projets.

## 14. Prérequis ffmpeg

OSATURA n'embarque ni ne lie jamais ffmpeg ; il exécute le `ffmpeg` (et `ffprobe`) trouvé dans le `PATH` comme un **sous-processus externe** de votre propre installation.

| Nécessaire pour | Encodeur/décodeur | Vérifié sur |
|---|---|---|
| `export .mp4` | `libx264` (`yuv420p`), `aac` | ffmpeg 6.1.1 |
| `export .webm` (par défaut) | `libvpx-vp9`, `libopus` | ffmpeg 6.1.1 |
| `export .webm --codec av1` | `libaom-av1`, `libopus` | ffmpeg 6.1.1 |
| `dof-export .mp4/.webm` | identique à ci-dessus, `--codec` | ffmpeg 6.1.1 |
| `sound mix .ogg` | `libvorbis` | ffmpeg 6.1.1 |
| `sound mix .mp3`, audio `.html` | `libmp3lame` | ffmpeg 6.1.1 |
| `face-track` à partir d'une vidéo | décodeurs uniquement | |
| contrôle du contenu de `sound add` | `ffprobe` (sans lui, seule l'existence est vérifiée pour les fichiers non WAV) | |

Vérifiez le vôtre avec `ffmpeg -hide_banner -encoders | grep -E 'libx264|aac|libvpx-vp9|libaom-av1|libopus|libvorbis|libmp3lame'`. Tout le reste
(GIF, APNG, HTML sans audio, Lottie, dotLottie, GLB, OCA, PNG, SVG, EXR) fonctionne sans ffmpeg.
Si ffmpeg est absent, la commande échoue avec `cannot run ffmpeg (...)` et n'écrit rien ; un encodeur manquant donne `ffmpeg failed`.

## 15. Repères sur les brevets et les licences

* [PATENT_NOTES.md](PATENT_NOTES.md) consigne ce qui n'a **pas** été examiné. Pour l'export, la ligne pertinente est *Codecs vidéo et audio via ffmpeg
  (`export` .mp4 / .webm, `soundtrack`)* : H.264 et AAC sont couverts par des pools de licences ; VP9 et AV1 sont présentés comme libres de redevances, mais des revendications de pool et individuelles
  ont été annoncées pour les deux ; **statut : non évalué, ne considérez pas VP9/AV1 comme validés.** Comme ffmpeg est l'installation propre de l'utilisateur, chaque
  choix de codec relève de la responsabilité de l'utilisateur, et OSATURA ne doit pas embarquer ffmpeg.
* Les écrivains déterministes raster, GIF, APNG, PNG, SVG, OpenEXR, OCA, ORA, glTF et Lottie sont des implémentations propres d'OSATURA, réalisées à partir de descriptions
  publiques des formats (clean-room, voir CLEANROOM.md). Leur statut en matière de brevets n'est pas évalué au-delà : PATENT_NOTES.md ne liste que les domaines ci-dessus
  (marionnettage piloté par le visage, machines à états et liaison de données, FABRIK/MVC/poids, remplissage stencil-then-cover, codecs).
* GIF : les brevets LZW ont expiré depuis longtemps dans toutes les juridictions ; c'est une connaissance générale, et non une validation consignée dans le dépôt.
* Licences des fichiers produits : les fichiers que vous exportez vous appartiennent ; le moteur est sous MPL-2.0 et la CLI sous AGPL-3.0-or-later, ce qui s'applique aux programmes, pas aux
  fichiers qu'ils écrivent. Le studio propose son code source correspondant à `/source` (AGPL section 13).
* Ceci n'est pas un avis juridique.

## 16. Valider ce que vous avez écrit

| Format | Vérification | Résultat attendu |
|---|---|---|
| tout projet | `osatura check out.json` | `ok: ...` (également après `import-*`) |
| GIF | `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` | nombre d'images (8) |
| APNG | même commande sur `out.apng` | nombre d'images (8) |
| MP4 | `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4` | `h264,320,240,yuv420p,24/1,8` |
| WebM | idem sur `out.webm` avec `:stream_tags=alpha_mode` | `vp9,...,yuv420p,...,1` (alpha_mode 1) |
| EXR | `ffprobe f.exr` | `exr 640x480 gbrapf32le` |
| Lottie | `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],d['w'],d['h'],len(d['layers']))"` | `5.7.0 24.0 8 640.0 480.0 12` |
| Avertissements Lottie | `osatura export p.json out.json --report rep.json` | `rep.json` liste chaque approximation |
| dotLottie | `unzip -l out.lottie` | `manifest.json` et `a/animation.json` uniquement |
| Aller-retour Lottie | `osatura import-lottie out.json back.json && osatura check back.json` | `ok:` |
| GLB | `python3 -c "import struct;b=open('out.glb','rb').read();print(b[:4],struct.unpack('<II',b[4:12]),len(b))"` | `b'glTF' (2, N)` avec N égal à la longueur |
| HTML | `grep -c '<svg' out.html` ; `grep -o 'http[^"<]*' out.html` | un `<svg` par image ; uniquement l'URL de l'espace de noms SVG |
| SVG | ouvrir dans un navigateur ou analyser comme XML | bien formé |
| Planche de sprites | `python3 -m json.tool out.atlas.json` | `{index,x,y,w,h}` par image |
| OCA | `python3 -m json.tool <name>.oca` | chaque cel référencé existe |
| Régression visuelle | `osatura ci diff a.png b.png --metric ssim` | code de sortie 0 si égaux ; voir [CI.md](CI.md) |

Pour les contrôles visuels d'un export avec perte par rapport à l'original, `osatura export-fit` indique déjà le SSIM et le CIEDE2000 par rapport au rendu original, et
`osatura compare` écrit une image avant/après/différence de deux projets.

## 17. Différences avec les autres documents

* `docs/LIMITATIONS.md` L-29 indique que l'import Lottie ne prend pas en charge les assets d'image, les effets, les précompositions ni les mattes ; l'importeur actuel importe bien les images
  avec URI de données intégrées, les précompositions, les effets par calque (flou, ombre portée, teinte, remplissage) et approxime les mattes de luminance (section 9). La liste des
  fonctionnalités ignorées de la section 9 est ce dont la commande avertit réellement.
* `docs/LIMITATIONS.md` L-32 liste OCA parmi les formats d'import ; la CLI a un **export** OCA (`export-oca`) et aucune commande `import-oca`.
* Le texte d'aide de `export` ne nomme que `.gif .html .json .mp4` plus « les formats ouverts » ; la liste complète des extensions est celle de la section 2.
* Les autres écarts entre le texte d'aide et le comportement sont rassemblés à la fin de [CLI_REFERENCE.md](CLI_REFERENCE.md).
