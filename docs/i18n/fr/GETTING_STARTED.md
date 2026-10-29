# Prise en main d'OSATURA

Ce guide vous mène d'un dépôt fraîchement cloné à un petit personnage animé que vous aurez construit, rigué,
animé par clés, mis en boucle, rendu et exporté en n'utilisant que la ligne de commande. Chaque commande ci-dessous a été
exécutée pour de vrai pendant la rédaction de cette page ; les sorties affichées sont celles qui ont été imprimées.

Comment cela a été vérifié : Linux, `rustc 1.97.0`, `ffmpeg 6.1.1`, et le build **debug** du binaire
(`cargo build -p osatura-cli`). Un build release exécute le même code plus vite. La suite de tests passe aussi sous macOS et Windows dans GitHub Actions ; les commandes de cette page n'ont été exécutées à la main que sous Linux. Aucune version minimale
supportée de Rust n'est déclarée dans le dépôt ; 1.97.0 est simplement le compilateur qui a été testé.

Licence de la documentation : CC-BY-4.0. Voir aussi [RIGGING_GUIDE.md](RIGGING_GUIDE.md),
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md), [STUDIO.md](STUDIO.md), [FAQ.md](FAQ.md) et la liste complète
des commandes dans [CLI_REFERENCE.md](CLI_REFERENCE.md).

## 1. Ce dont vous avez besoin

| Prérequis | Nécessaire pour | Remarques |
|---|---|---|
| Chaîne d'outils Rust (`rustc`, `cargo`) | compiler OSATURA | à installer depuis <https://rustup.rs> ; canal stable |
| Un navigateur web | le studio (`osatura studio`) | seul Chromium a été testé, voir [STUDIO.md](STUDIO.md) |
| `ffmpeg` dans le `PATH` (facultatif) | `.mp4`, `.webm`, mixage du son dans la vidéo/le HTML, `sound mix`, `face-track` sur un fichier vidéo | tout le reste fonctionne sans lui |

Sans `ffmpeg`, les commandes qui en ont besoin s'arrêtent avec un message clair et n'écrivent rien :

```
$ PATH=/nonexistent osatura export buddy-final.json o.mp4
cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead
```

Aucun GPU n'est requis. Le moteur de rendu GPU facultatif (`render-gpu`) ne fait pas partie du build par défaut.

**Pas de Rust ? Utilisez une archive précompilée.** Quand le propriétaire a publié la préversion en brouillon `v0.1.0-alpha.3`, la [page des versions](https://github.com/chiuta/osatura/releases) propose `osatura-windows-x86_64.zip`, `osatura-macos-arm64.tar.gz`, `osatura-linux-x86_64.tar.gz` (glibc 2.35+ ; les versions suivantes ajoutent l'archive statique `osatura-linux-x86_64-musl.tar.gz` pour les distributions plus anciennes) et `SHA256SUMS.txt`. Vérifiez l'archive avec les sommes de contrôle (`sha256sum -c SHA256SUMS.txt --ignore-missing`, ou `Get-FileHash` sous Windows), décompressez-la et lancez `osatura` (`osatura.exe`) depuis le dossier décompressé ; sautez la compilation de la section 2 et lisez « Windows : double-clic sur `osatura.exe` » et « Le studio dans une fenêtre » vers la fin de cette page. Les programmes ne sont pas signés : le système vous avertira la première fois. Tout ce qui suit a été exécuté avec une compilation locale des mêmes sources.

## 2. Le compiler

À la racine du dépôt :

```
cargo build --release -p osatura-cli
```

Le binaire est `target/release/osatura`. Placez-le dans votre `PATH`, ou créez un alias :

```
export PATH="$PWD/target/release:$PATH"      # or: alias osatura=$PWD/target/release/osatura
osatura --version
```

`--version` affiche la version, la licence (AGPL-3.0-or-later pour la CLI et le studio, MPL-2.0 pour les
bibliothèques du moteur) et l'offre de code source. Lancer `osatura` sans argument affiche l'aide complète ;
`osatura <command> --help` affiche l'aide d'une seule commande.

Vous pouvez aussi l'exécuter sans l'installer : `cargo run --release -p osatura-cli -- demo demo.json`.

Compiler l'outil en ligne de commande avec `--no-default-features` supprime la commande `script` (le moteur
de script Rhai) ; ce tutoriel utilise `script`, conservez donc les fonctionnalités par défaut.

## 3. Premiers pas avec la démo intégrée

`osatura demo` écrit un projet d'exemple complet : un bras avec skinning, une jambe en IK, un calque de commutation de lip-sync,
un arrière-plan en parallaxe et un mouvement de caméra.

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
```

`check` compile le projet exactement comme le ferait le moteur de rendu : un projet qui passe `check` peut donc être
rendu. Dessinons maintenant une image fixe, avec les os en surimpression, et exportons une animation :

```
$ osatura render demo.json 12 frame12.png --bones
wrote frame12.png
$ osatura render demo.json 12 frame12.svg
wrote frame12.svg
$ osatura export demo.json demo.gif
wrote demo.gif
$ osatura export demo.json demo.html
wrote demo.html
$ osatura export demo.json demo.lottie
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
wrote demo.lottie
```

Ce que vous venez de faire :

* `render <project> <frame> <out.png|out.svg>` dessine une image. `--bones` superpose le squelette,
  `--scale N` effectue le rendu à N fois la taille du canevas (0 < N <= 16).
* `export` choisit le format d'après l'extension du fichier : `.gif`, `.html` (une page de lecteur autonome),
  `.json` (Lottie), `.lottie` (dotLottie), `.apng`, `.webm`, `.mp4`, `.glb`, `.png` (sprite sheet).
* Les lignes `warning:` sont la partie honnête : Lottie ne peut pas tout représenter de ce que sait faire OSATURA. Chaque
  limitation est affichée sur stderr, et `--report r.json` écrit la même liste au format JSON.
* Ouvrez `demo.html` dans un navigateur pour lire l'animation ; ni serveur ni réseau n'interviennent.

Pour explorer de façon interactive :

```
osatura studio demo.json --port 8080
```

et ouvrez <http://127.0.0.1:8080/>. Voir [STUDIO.md](STUDIO.md).

## 4. Un tutoriel de 15 minutes : un personnage qui salue

Vous allez construire « Buddy » à partir d'un projet vide. Le fichier de projet est du JSON, mais vous n'avez jamais à écrire
plus qu'un en-tête : la commande `script` exécute un petit script [Rhai](https://rhai.rs) en bac à sable à travers
le même éditeur validé que celui du studio. Si une requête d'un script est rejetée, *rien* n'est appliqué
(tout ou rien), de sorte qu'un mauvais script ne laisse jamais un fichier à moitié modifié.

Travaillez dans un répertoire vide. Chaque étape écrit un nouveau fichier numéroté, ce qui vous permet de toujours revenir en arrière.

### Étape 1 : un projet vide (1 minute)

```
cat > buddy.json <<'EOF'
{"format": 1, "name": "Buddy", "width": 320, "height": 240, "fps": 24, "frames": 24}
EOF
osatura check buddy.json
```
```
ok: 'Buddy' 320×240 @ 24 fps, 24 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Ces six clés sont le minimum. 24 images à 24 ips, c'est une seconde, ce qui sera notre boucle.

### Étape 2 : les formes (2 minutes)

Les calques sont dessinés de l'arrière vers l'avant, dans l'ordre où vous les ajoutez. Créez `shapes.rhai` :

```
add_gradient_rect("sky", 0, 0, 320, 240, "#bfe3f5", "#fdf3d8");
add_rect("ground", 0, 200, 320, 40, "#7fa66b");
add_rect("torso", 140, 120, 40, 70, "#e07a5f");
add_circle("head", 160, 95, 24, "#f2cc8f");
add_rect("upper_arm", 160, 134, 40, 12, "#3d405b");
add_rect("forearm", 200, 134, 40, 12, "#81b29a");
```
```
$ osatura script buddy.json 01-shapes.json shapes.rhai
6 request(s) applied; wrote 01-shapes.json
```

Les coordonnées sont des unités de canevas, avec l'origine en haut à gauche et l'axe y orienté vers le bas. Le bras est dessiné
à l'horizontale, l'épaule en (160, 140), car c'est la pose dans laquelle nous allons construire ses os.

### Étape 3 : os, hiérarchie et rattachement du dessin (3 minutes)

Un os a un nom, un parent (vide = racine), une longueur, une position de tête (`x`, `y`) dans l'espace de son parent
(pour une racine : dans le canevas) et un angle de repos en degrés relatif à son parent. L'angle 0 pointe le long de +x,
et les angles positifs tournent dans le sens horaire à l'écran ; 90 pointe donc vers le bas et -90 vers le haut.
Créez `bones.rhai` :

```
add_bone("body", "", 70, 160, 190, -90);
add_bone("neck", "body", 20, 70, 0, 0);
add_bone("arm_up", "body", 40, 50, 0, 90);
add_bone("arm_low", "arm_up", 40, 40, 0, 0);
add_bone("hand_ctrl", "", 10, 210, 165, 0);
set_layer("torso", "bone", "body");
set_layer("head", "bone", "neck");
set_layer("upper_arm", "bone", "arm_up");
set_layer("forearm", "bone", "arm_low");
```
```
$ osatura script 01-shapes.json 02-bones.json bones.rhai
9 request(s) applied; wrote 02-bones.json
$ osatura render 02-bones.json 0 rig.png --bones --scale 2
wrote rig.png
```

Lisez les os ainsi : `body` est un os racine en (160, 190) qui pointe vers le haut (-90) et mesure 70 de long.
`neck` est son enfant ; sa tête se trouve à 70 unités le long de `body` (en haut). `arm_up` est un autre enfant de
`body`, à 50 unités le long de la colonne (l'épaule), tourné de +90 pour pointer sur le côté. `arm_low` est l'enfant
de `arm_up` : la hiérarchie signifie que faire tourner le haut du bras entraîne l'avant-bras avec lui. `hand_ctrl` est un
os racine distinct que nous utiliserons comme cible d'IK ; il n'a pas de dessin.

`set_layer(layer, "bone", bone)` fait suivre un os à un calque de façon *rigide* : il suit le mouvement de l'os
par rapport à sa pose de repos. C'est le moyen le plus simple de rigger un dessin animé composé de pièces séparées. (Pour
des formes souples, vous liez par skinning des ancres vectorielles à plusieurs os ; voir [RIGGING_GUIDE.md](RIGGING_GUIDE.md).)

### Étape 4 : cinématique inverse (1 minute)

Au lieu d'animer par clés les angles de l'épaule et du coude, laissez le solveur atteindre `hand_ctrl`. Créez `ik.rhai` :

```
add_ik("arm_ik", ["arm_up", "arm_low"], "hand_ctrl", 1);
```
```
$ osatura script 02-bones.json 03-ik.json ik.rhai
1 request(s) applied; wrote 03-ik.json
```

Le dernier argument est le sens de flexion : +1 ou -1 choisit de quel côté le coude se plie. Une chaîne de deux os utilise
un solveur analytique ; les chaînes de trois os ou plus utilisent FABRIK. Désormais, déplacer `hand_ctrl`
déplace tout le bras.

### Étape 5 : clés et interpolation (3 minutes)

`set_key(target, property, frame, value, ease)` crée des clés. Pour un os, `bone_angle`, `bone_scale`,
`bone_offset_x` et `bone_offset_y` sont des *écarts par rapport à la pose de repos* : une valeur de clé de 0 signifie donc
« repos ». Le nom d'interpolation s'applique au segment qui *commence* à cette clé. Créez `keys.rhai` :

```
set_key("hand_ctrl", "bone_offset_y", 0, 0, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 12, -75, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 24, 0, "ease_in_out");
set_key("neck", "bone_angle", 0, 0, "smooth");
set_key("neck", "bone_angle", 16, 10, "smooth");
set_key("neck", "bone_angle", 24, 0, "smooth");
```
```
$ osatura script 03-ik.json 04-keys.json keys.rhai
6 request(s) applied; wrote 04-keys.json
$ osatura timeline 04-keys.json
timeline: 24 frames at 24 fps, 2 channels
Bones
  hand_ctrl
    #0 bone_offset_y (keys @ 0 12 24)
  body
    neck
      #1 bone_angle (keys @ 0 16 24)
```

Les noms d'interpolation qui ne demandent aucun paramètre sont `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`,
`smooth` et `bounce`. `bezier:x1,y1,x2,y2` est accepté par `keyedit`, et `spring` et `back` prennent des
paramètres (voir [PROJECT_FORMAT.md](PROJECT_FORMAT.md)). La main et la tête utilisent des timings différents volontairement : l'action
qui se chevauche donne une impression de vie.

### Étape 6 : la mettre en boucle (1 minute)

Les deux canaux commencent et finissent à la même valeur (l'image 24 répète l'image 0), ce dont une boucle a besoin.
`loop-solve` vérifie la jonction et règle les canaux en boucle :

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
$ osatura lint buddy-final.json
0 finding(s), 0 error(s)
```

« seam 0.000 -> 0.000 » signifie que la dernière pose est déjà égale à la première, donc rien n'a eu à être corrigé. Un
projet de N images joue les images 0..N-1 en boucle ; l'image N est donc la même pose que l'image 0 et n'est jamais
affichée deux fois. Si vous créez une boucle de sorte que la *dernière image affichée* soit égale à la première, passez `--inclusive`
(l'ancienne convention). Si votre animation a un déplacement net, `make-loop` le supprime. Voir
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

`lint` exécute des contrôles statiques du rig et de l'animation (cibles d'IK inatteignables, canaux morts, os manquants...) ;
il se termine avec le code 1 lorsqu'il trouve des erreurs, ce qui permet de l'intégrer sans risque à un script ou à un job de CI.

### Étape 7 : la regarder, la rendre (2 minutes)

```
$ osatura stats buddy-final.json
```
affiche un résumé JSON (ici : 5 os, 1 chaîne d'IK, 6 calques, 2 canaux, 6 clés). Rendez des images fixes et une
séquence d'images complète :

```
$ osatura render buddy-final.json 12 frame12.png --scale 2
wrote frame12.png
$ osatura render buddy-final.json 12 frame12.svg
wrote frame12.svg
$ osatura frames buddy-final.json frames --png
wrote 24 frames to frames
$ ls frames | head -3
frame_0000.png
frame_0001.png
frame_0002.png
```

![Quatre images (0, 6, 12, 18) de l'animation de Buddy, une fois seules et une fois avec les os en surimpression. Le bras droit se lève dans un salut, coude vers le bas, tandis que la tête hoche légèrement.](../../img/tutorial-buddy.png)

Sans `--png`, `frames` écrit des fichiers SVG. Les rendus sont déterministes : le même projet et la même image donnent
les mêmes octets sur toute machine avec le même build, ce qui rend possibles les contrôles par images de référence de
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Étape 8 : exporter (2 minutes)

```
$ osatura export buddy-final.json buddy.gif
wrote buddy.gif
$ osatura export buddy-final.json buddy.html
wrote buddy.html
$ osatura export buddy-final.json buddy.json --report buddy-report.json
wrote buddy.json
$ cat buddy-report.json
{
  "count": 0,
  "format": "lottie",
  "warnings": []
}
$ osatura export buddy-final.json buddy.mp4        # needs ffmpeg
wrote buddy.mp4
$ osatura export buddy-final.json buddy.apng
wrote buddy.apng
```

Tailles de fichier de ce minuscule projet : GIF 145 Ko, JSON Lottie 21 Ko, lecteur HTML 50 Ko, APNG 75 Ko, MP4 6 Ko.
`buddy.json` est du Lottie ; sa liste d'avertissements est vide parce que Buddy n'utilise que des fonctionnalités que Lottie sait
exprimer. Le fichier HTML est un lecteur autonome (ouvrez-le dans n'importe quel navigateur).

Quelques règles à retenir :

* `export` et `render` **refusent d'écraser le projet d'entrée** (`refusing to overwrite the input
  project buddy-final.json with output of the same name`), et refusent d'écrire à travers un lien symbolique
  sauf si vous passez `--follow-symlinks`. Les commandes qui transforment un projet en un autre
  (`script`, `loop-solve`, `walk`, `autorig`...) n'ont *pas* cette vérification : elles écrivent de façon atomique, donc
  `osatura script p.json p.json x.rhai` modifie le fichier sur place. Utilisez des noms distincts tant que vous ne faites pas confiance à un script.
* Le GIF a 256 couleurs et une transparence sur 1 bit ; pour un alpha progressif, utilisez `.apng` ou `.webm`.
* Lancez `osatura export-fit` (voir [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) lorsque vous devez respecter un budget d'octets.

### Étape 9 (facultative) : le modifier dans le studio

```
osatura studio buddy-final.json --port 8080
```

Faites glisser les calques, sélectionnez une clé dans la dope sheet, appuyez sur `Space` pour lire. Rien n'est écrit dans votre fichier
tant que vous n'appuyez pas sur **Enregistrer** (`Ctrl+S`).

## 5. Comment les pièces s'assemblent

| Concept | Dans le fichier | Vous le créez avec |
|---|---|---|
| Calque (vecteur, image, groupe, commutation...) | `layers` | `add_rect`, `add_circle`, `add_polygon`, `add_line`, `import-svg`, `autotrace`... |
| Os | `bones` (parents avant enfants) | `add_bone`, `set_parent`, `reparent` |
| Chaîne d'IK / contrainte / ressort | `ik`, `constraints`, `springs` | `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin` ; ressorts via JSON |
| Canal (une propriété animée) | `channels` | `set_key`, `keys`, `keyedit`, expressions, bruit, bakes |
| Clip, take, machine à états, liaison | `clips`, `machines`, `bindings` | `take-record`, `clip`, `machine`, `bind` |

Tout est du JSON brut, ce qui explique que les diffs (`osatura diff a.json b.json`), la gestion de versions
(`osatura vcs`) et les scripts fonctionnent. Les champs inconnus sont signalés par des avertissements et ignorés :

```
$ osatura check unk.json
warning: unknown field bogus (ignored)
ok: 'X' 100×100 @ 24 fps, 10 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Un projet ne peut référencer des fichiers d'image et de son que **dans son propre répertoire**. Les chemins absolus et `..`
sont refusés, sauf si vous passez `--allow-external-assets` (voir [FAQ.md](FAQ.md)).

## 6. Pour aller plus loin

* Rigger quelque chose de souple (skins, poids, lattice, cage, tissu, physique) : [RIGGING_GUIDE.md](RIGGING_GUIDE.md).
* Cycles de marche, piles de clips, lip-sync, mocap, machines à états, assurance qualité et budgets de taille :
  [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).
* L'éditeur dans le navigateur, ses raccourcis et l'API HTTP : [STUDIO.md](STUDIO.md).
* Questions et messages d'erreur : [FAQ.md](FAQ.md). Limites connues : [LIMITATIONS.md](LIMITATIONS.md).
* Chaque commande et chaque option : [CLI_REFERENCE.md](CLI_REFERENCE.md) ; le format de fichier : [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Windows : double-clic sur `osatura.exe`

`osatura.exe` est un programme en ligne de commande. Depuis 0.1.0-alpha.2, l'ouvrir depuis l'Explorateur (double-clic) crée `Documents\OSATURA\first-rig.json` à partir de la démo de la personne qui marche (`osatura demo-person` ; un fichier existant n'est jamais écrasé), démarre le studio sur `http://127.0.0.1:8080` et l'ouvre dans votre navigateur ; la fenêtre de console reste ouverte tant que le studio tourne, fermez-la pour l'arrêter. Ouvert depuis un terminal sans argument, il affiche l'utilisation comme avant. Dans 0.1.0-alpha.1, un double-clic affichait seulement l'utilisation et la fenêtre se fermait aussitôt.

Les exécutables **ne sont pas signés** : Windows SmartScreen affiche « Éditeur inconnu » (choisissez *Informations complémentaires* puis *Exécuter quand même*) et Gatekeeper de macOS vous demande d'approuver le programme dans *Réglages Système > Confidentialité et sécurité*. Vérifiez d'abord le téléchargement avec `SHA256SUMS.txt`. Si le port 8080 est occupé, lancez `osatura studio first-rig.json --port 8090` depuis un terminal.

## Le studio dans une fenêtre : `osatura-studio` (Windows et macOS)

Les archives Windows et macOS contiennent un second programme, `osatura-studio` (`osatura-studio.exe`), une petite fenêtre native autour du studio du navigateur : il démarre le moteur `osatura` du même dossier en tant que `studio` sur un port local libre, affiche la page dans une fenêtre WebView2 (intégré à Windows 11 ; un Windows plus ancien a besoin du runtime Evergreen) ou WKWebView (macOS), et arrête le serveur à la fermeture de la fenêtre. Aucune fenêtre de console n'apparaît. Gardez-le dans le même dossier que `osatura(.exe)`.

* **Exports** (GIF, lecteur HTML, Lottie) : une boîte de dialogue native « Enregistrer sous » s'ouvre, démarrant dans le dossier du projet, et un message indique le fichier une fois enregistré. (alpha.2 les enregistrait silencieusement dans votre dossier Téléchargements ; alpha.3 vous demande.)
* Double-cliquez dessus pour travailler sur `Documents/OSATURA/first-rig.json` (créé à partir de la démo de la personne qui marche, `osatura demo-person`, s'il manque), ou lancez-le avec `osatura-studio my-project.json` (par exemple via « Ouvrir avec »).
* `osatura-studio --selftest` démarre le serveur, récupère la page et l'arrête sans fenêtre ; le code de sortie est 0 si cela a fonctionné (la CI l'exécute sous Windows et macOS).
* **Limites.** La fenêtre a été utilisée sur un vrai bureau Windows 11 (alpha.2 : le studio et les exports ont fonctionné) ; la boîte de dialogue « Enregistrer sous » d'alpha.3 est compilée et auto-testée en CI mais personne ne l'a encore essayée, et personne n'a utilisé la fenêtre sous macOS. Les autres fonctions de type navigateur qui dépendent du système (glisser-déposer depuis l'Explorateur ou le Finder, ouverture de fichiers depuis les sélecteurs propres au studio) sont celles de la vue web du système et ne sont pas testées. Il n'y a pas de fenêtre Linux (elle aurait besoin des paquets système GTK / WebKitGTK) : utilisez `osatura studio` et un navigateur. Le programme n'est pas signé (voir plus haut).
