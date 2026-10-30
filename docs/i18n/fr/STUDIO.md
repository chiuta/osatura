# Le studio d'OSATURA

Le studio est un éditeur dans le navigateur servi par le binaire `osatura` lui-même : `osatura studio project.json`.
Il comprend un canevas avec des manipulateurs de sélection et de transformation, une liste de calques, une liste d'os, une dope sheet et un graphe de courbes,
l'onion skin et les trajectoires de mouvement, une palette de commandes avec des raccourcis reconfigurables, des suggestions de poses, des panneaux d'entrée en direct et de
visage, des boutons d'export GIF/HTML/Lottie, et une session d'édition partagée multi-utilisateur. Il n'a ni comptes, ni TLS, ni
base de données : c'est un petit serveur HTTP à bibliothèque standard placé devant le même éditeur que celui qu'utilise la ligne de commande.

Tout ce qui figure ici a été vérifié sur le vrai binaire (`osatura 0.1.0`, build de débogage, Linux) et, pour l'interface
utilisateur, dans Chromium sans interface piloté par Playwright. Les captures d'écran ont été réalisées ainsi (1400x900, le projet de
démonstration intégré). Seul **Chromium** a été testé ; voir la section 11. Sous Windows et macOS, les archives de release contiennent aussi `osatura-studio`, le même studio dans une fenêtre native (compilé et auto-testé en CI, pas encore utilisé par une personne ; voir [GETTING_STARTED.md](GETTING_STARTED.md) et [LIMITATIONS.md](LIMITATIONS.md) L-75), et sous Windows un double-clic sur `osatura.exe` démarre le studio et ouvre le navigateur (L-76).

Licence de ce document : CC-BY-4.0. Pour la ligne de commande, voir [GETTING_STARTED.md](GETTING_STARTED.md) et
[CLI_REFERENCE.md](CLI_REFERENCE.md) ; pour la sécurité du serveur, voir aussi [SECURITY.md](SECURITY.md).

![Le studio avec le projet de démonstration : barre d'outils en haut, le canevas avec un calque de bras sélectionné et ses os dessinés, le panneau Calques à droite, et la timeline en dessous avec des losanges de clés.](../../img/studio-overview.png)

## 1. Démarrer le studio

```
osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES]
                [--public-origin URL] [--source-url URL] [--collab-token T] [--collab-log LOG [--collab-nosync]]
```

```
$ osatura demo demo.json
$ osatura studio demo.json --port 8080
AGPL-3.0 source offer: http://127.0.0.1:8080/source
OSATURA studio on http://127.0.0.1:8080  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in /home/you/.config/osatura/ui.json      (your own home directory)
```

Ouvrez l'adresse dans un navigateur. Le projet est validé au démarrage ; un fichier qui ne passe pas `check` est refusé avec le
même message. Les champs inconnus donnent lieu à un avertissement (`warning: unknown field bogus (ignored)`).

| Option | Valeur par défaut | Signification |
|---|---|---|
| `--port N` | 8080 | Port TCP. Un port occupé donne `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)` ; un port invalide donne `cannot listen on 127.0.0.1:99999: invalid port value`. |
| `--bind ADDR` | `127.0.0.1` | Interface d'écoute. Toute adresse qui n'est pas une adresse de bouclage **exige `--collab-token`**. |
| `--ui-config F` | `$OSATURA_UI_CONFIG`, sinon `$XDG_CONFIG_HOME/osatura/ui.json`, sinon sous Windows `%APPDATA%\OSATURA\ui.json` (avant alpha.3, il n'y en avait pas : les réglages n'étaient conservés que pour la session), sinon `~/.config/osatura/ui.json` | Où sont conservés la disposition, les raccourcis, les commandes récentes et les réglages des surcouches. C'est un fichier propre à chaque utilisateur, qui ne fait jamais partie du projet. |

L'application à fenêtre et le démarrage par double-clic créent le premier projet dans `OSATURA_DOCUMENTS` lorsque cette variable est définie, sinon dans `Documents\OSATURA` (utilisé par le paquet portable).
| `--allow-host NAMES` | aucune | Noms `Host` supplémentaires à accepter (séparés par des virgules), pour un accès sous un nom DNS. |
| `--public-origin URL` | aucune | L'origine nue (`https://studio.example.org`, sans chemin) que présente un reverse proxy TLS ; ses `Host` et `Origin` sont alors acceptés. |
| `--source-url URL` | aucune ; dans les binaires de release officiels, le commit à partir duquel ils ont été construits (`https://github.com/chiuta/osatura/tree/<commit>`) | L'endroit où les sources correspondant à *votre* build peuvent être téléchargées (AGPL section 13) ; affiché sur `/source` et par `--version`. Une valeur explicite l'emporte toujours. |
| `--collab-token T` | aucune | Secret partagé, **d'au moins 16 caractères** (`openssl rand -hex 16`). Obligatoire pour les liaisons hors bouclage. |
| `--collab-log LOG` | aucune | Conserve la session d'édition partagée dans un fichier et la reprend après un redémarrage. |
| `--collab-nosync` | désactivé | Avec `--collab-log` : ne fait pas de `fsync` après chaque modification (plus rapide ; une coupure de courant peut faire perdre les dernières modifications). Sans `--collab-log`, l'option est refusée au démarrage (`--collab-nosync needs --collab-log`). |

**Enregistrement.** Les modifications vivent en mémoire. Le fichier de projet sur disque ne change que lorsque vous cliquez sur **Enregistrer** (`Ctrl+S`), ce qui écrit de façon atomique
(fichier temporaire puis renommage, jamais à travers un lien symbolique). Cela reste vrai pendant une session partagée (section 8).

## 2. Modèle de sécurité (à lire avant de se lier à un réseau)

Le studio est conçu pour une personne sur sa propre machine. Ce qui le protège :

* **Liste d'hôtes autorisés.** Seules les requêtes dont l'en-tête `Host` est l'un des noms propres au serveur reçoivent une réponse ; toute autre reçoit **421**.
  Avec `--allow-host` et `--public-origin`, davantage de noms sont acceptés.
* **Protection inter-sites.** Une requête avec un `Origin` étranger, ou `Sec-Fetch-Site: cross-site`, reçoit **403**. Un POST dont le corps n'est pas
  `application/json` reçoit **415** (les POST sans corps, tels que `/api/undo`, n'ont besoin d'aucun type).
* **En-têtes de sécurité** dans les réponses : une `Content-Security-Policy` restrictive (`default-src 'self'` et aucun cadre), `X-Frame-Options: DENY`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`.
* **Jeton pour tout ce qui sort de la machine.** Avec `--collab-token`, chaque requête a besoin du jeton, soit dans l'en-tête `X-Collab-Token`, soit dans le
  cookie `osatura_token` posé par `POST /api/login` (HttpOnly, SameSite=Strict). Le jeton n'est **jamais accepté dans l'URL**. Les jetons erronés sont ralentis
  par adresse cliente (à partir du cinquième échec consécutif, cette adresse reçoit **429** pendant 2, 4, ... jusqu'à 30 secondes), mais ce délai croissant ne peut pas arrêter un
  attaquant qui insiste : utilisez un jeton aléatoire long.
* **Secrets des participants.** Rejoindre une session partagée renvoie un secret aléatoire de 128 bits ; chaque appel fait en tant que ce participant doit l'envoyer sous la forme
  `X-Collab-Secret`. Connaître le numéro d'un participant ne suffit pas.

J'ai vérifié chacun de ces points avec `curl` contre un serveur en cours d'exécution :

```
bad Host (evil.example):               421
allowed host (--allow-host):           200
cross-origin POST (Origin: evil.example): 403
POST from the --public-origin:         200
Sec-Fetch-Site: cross-site:            403
POST with Content-Type: text/plain:    415
no token, GET / or /api/state:         401
wrong X-Collab-Token:                  401
right X-Collab-Token:                  200
token in the URL (?token=...):         401
/source without a token:               200
unknown route:                         404   (GET /favicon.ico answers with a small SVG icon, without a token)
```

Refus au démarrage (code de sortie 1, rien n'est lancé) :

```
refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)
--collab-token must be at least 16 characters (e.g. `openssl rand -hex 16`)
--source-url must be an http:// or https:// URL without spaces or quotes
--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org
```

**Limites assumées.** Le trafic est en HTTP simple : le jeton et tout le reste circulent en clair ; placez donc le studio derrière un reverse proxy TLS
(`--public-origin`) s'il est accessible à d'autres personnes. Le `Set-Cookie` du jeton reçoit l'**indicateur `Secure`** lorsque la requête arrive sous un nom `--public-origin` en `https://` (ou avec un `Origin` https / `X-Forwarded-Proto: https`) ; le HTTP simple sur l'adresse de bouclage n'en reçoit jamais,
et derrière un proxy tous les clients semblent avoir l'adresse du proxy, de sorte que le délai croissant freine tout le monde à la fois. `/source` est ouvert sans jeton
(l'AGPL l'exige). Il n'y a pas de comptes utilisateur : quiconque possède le jeton peut modifier le projet et écrire le chemin d'enregistrement.

### L'offre de code source AGPL

Le studio est un logiciel AGPL-3.0-or-later : si vous le modifiez et laissez d'autres personnes l'utiliser via un réseau, vous devez leur proposer les sources de votre
version. `/source` est cette offre ; définissez `--source-url` à l'endroit où se trouvent les sources de votre build. Sans cela (un build que vous avez réalisé vous-même avec un simple `cargo build` n'en a aucune), la page l'indique et nomme le mainteneur. Les binaires de release construits par `.github/workflows/release.yml` portent l'adresse de leur propre commit (`OSATURA_SOURCE_URL` à la compilation ; un fork qui construit avec son propre workflow pointe vers lui-même).

## 3. Visite de l'interface

### Barre d'outils

Annuler, Rétablir, **Enregistrer** ; les cases `bones` (dessiner le squelette), `auto-key`, `grid` (espacement en unités de canevas, 0 = désactivé), `snap` (sur les autres calques et le canevas),
`whole groups` (un clic sur une partie d'un groupe sélectionne le groupe de premier niveau), `points` (afficher les ancres, poignées, points de cage/lattice/pin et sommets de maillage des
calques sélectionnés) ; un sélecteur `box` pour ce que sélectionne un cadre de sélection (`touches`, `contains`, `bounds touch` ; il démarre à `touches`) ; **Lock** ; et les boutons **Overlays**, **Suggest**, **Layout**,
**Commands** (palette) et **Keys** (éditeur de raccourcis). Une seconde rangée contient votre nom et **Collab**, ainsi que les boutons d'export **GIF**, **HTML player** et **Lottie**,
qui téléchargent ce qu'écrirait `osatura export` (le lecteur HTML inclut le son mixé lorsqu'il y a du son et que `ffmpeg` peut produire le mp3). Le bouton **Simple view** masque les cases d'édition, Overlays, Suggest et le bouton de la palette de commandes, de sorte qu'il ne reste que l'annulation, l'enregistrement, les panneaux et les exports ; ce choix est mémorisé dans ce navigateur (ou cette fenêtre) et la barre d'outils complète revient avec **Full view**.

### Panneaux et dispositions

La fenêtre est un dock de panneaux, chacun avec un identifiant stable : `stage` (le canevas), `layers`, `selection`, `bones`, `live` (Live input), `face` (Webcam face), `json`
(le projet entier sous forme de JSON modifiable, avec un bouton *Apply JSON*) et `timeline`. Quatre préréglages de disposition existent :

```
animate  stage over timeline on the left, all panels as tabs on the right
rig      stage with a folded timeline; layers+bones and selection+json on the right (live and face hidden)
compact  stage over a tabbed timeline and panels
live     stage and timeline on the left; live, face, bones and selection on the right (layers and json hidden)
```

Changez de préréglage depuis le menu **Layout** ou la palette ; faites glisser les onglets pour réancrer ou détacher un panneau ; « reset layout » revient à la disposition par défaut.
La disposition est enregistrée dans la configuration de l'interface, par utilisateur. `osatura ui layout presets|show NAME|validate FILE.json` affiche et valide les mêmes données
(une disposition doit garder chaque panneau accessible ; une disposition incomplète est rejetée par le serveur avec 400).

### Panneau Calques

![Le panneau Calques avec un calque verrouillé (sun), un calque masqué (hills), et arm et leg sélectionnés ensemble.](../../img/studio-layers.png)

Chaque ligne a deux cases : **visible** et **lock** (enregistrée *avec le projet* ; les calques verrouillés sont ignorés par les clics et les cadres de sélection, et refusent les modifications).
La capture d'écran montre `sun` verrouillé, `hills` masqué et deux calques sélectionnés avec Maj-clic.

### Panneau Sélection

Énonce les règles dans la page elle-même : *Un clic choisit l'élément le plus haut (les éléments verrouillés et masqués sont ignorés) ; Maj-clic bascule ; faire glisser sur un espace vide
trace un cadre (maintenez Alt pour tracer un lasso à main levée, Maj ajoute). Faites glisser le cadre pour déplacer, un carré pour mettre à l'échelle (Maj conserve les proportions, Alt met à l'échelle depuis le
centre), le bouton rond pour pivoter (Maj aimante sur 15 degrés). Maintenez Ctrl pendant le glissement pour désactiver l'aimantation. Les touches fléchées déplacent de 1 unité (Maj : 10),
Suppr supprime les calques, Échap annule un glissement ou efface la sélection.*

### Timeline : dope sheet et graphe

![La dope sheet avec une sélection au cadre de trois clés à l'image 12 (les clés sont dessinées pleines, les autres creuses).](../../img/studio-dopesheet.png)

![La vue en graphe de courbes : trois courbes bone_angle (arm_up, arm_low, hand) de couleurs différentes, avec trois clés sélectionnées à l'image 12 montrant leurs poignées de tangente ; la ligne d'état indique « 3 keys selected | frames 12 ».](../../img/studio-graph.png)

`Graph` (ou `Alt+V`) bascule entre la dope sheet et le graphe de courbes. Les commandes de la timeline : `Play`, `|<` (première image de la plage), `loop`, `range` avec
`Set in` / `Set out` (lire uniquement entre l'entrée et la sortie), `snap` (arrondir les clés déplacées à des images entières), `on collision` (`replace`, `merge`, `refuse` lorsque des clés déplacées tombent sur
des clés existantes), `Copy` / `Paste` / `Duplicate` / `Delete`, `interp` + `Apply` (définir l'interpolation des clés sélectionnées), `Auto-smooth` (intégrer par bake des tangentes sans dépassement dans les
clés sélectionnées), `ripple` N avec `Insert` / `Remove` (et `grow project` pour modifier avec eux la durée du projet), zoom `-` `+` `Fit`, et `shy` (afficher les canaux marqués shy).

Souris : cliquez sur une clé pour la sélectionner, Maj-clic pour en ajouter, faites glisser un cadre pour sélectionner, faites glisser des clés pour les déplacer ; **Alt-glisser ou le bouton du milieu fait un panoramique**, **Ctrl+molette zoome**,
**Maj+molette fait défiler** verticalement ; dans le graphe, **Maj contraint le glissement de clé à un seul axe**. La tolérance d'aimantation du canevas est par défaut de 6 unités de canevas.
Chaque modification de clé passe par les mêmes opérations validées que `osatura keyedit` (voir [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) ; c'est donc une seule étape d'annulation, et les canaux verrouillés la refusent.

### Surcouches : onion skin, trajectoires, proxies

![L'onion skin et les trajectoires de mouvement activés pour le bras à l'image 14, avec le panneau Overlays ouvert.](../../img/studio-overlays.png)

`Overlays` (`Alt+7`) regroupe l'onion skin (`Alt+N` : images avant/après en teintes rouge/verte, pas fixe ou images clés, mode contour, atténuation, calques sélectionnés uniquement),
les trajectoires de mouvement (`Alt+M` : chemins d'os, de pivot de calque ou de point de contrôle avec des repères par image) et les **proxies de scrub** (`Alt+Q` : images de qualité réduite pendant que vous faites glisser la
tête de lecture, avec un cache d'images et une copie facultative à niveau de détail réduit). Le panneau indique, par exemple, `picture cache: 2 frames, 17 KiB, 0 hits / 7 misses, emptied 4x by edits`.
Les réglages sont stockés dans la configuration de l'interface et survivent à un redémarrage. Les surcouches ne font jamais partie des exports.

### Palette de commandes et raccourcis

![La palette de commandes ouverte avec Ctrl+K et filtrée par le mot « toggle », listant les commandes correspondantes avec leurs indications de raccourci.](../../img/studio-palette.png)

`Ctrl+K` (ou `Ctrl+Shift+P`) ouvre la palette : tapez pour filtrer (approximatif, lettres correspondantes surlignées), les touches fléchées déplacent, Entrée exécute, Échap ferme. Sans saisie, elle
liste d'abord les commandes récentes. Il y a **84 commandes** ; `osatura ui commands` les liste et `osatura ui search onion` les classe exactement comme le fait la palette.

![L'éditeur de raccourcis clavier ouvert avec Ctrl+virgule, listant les commandes par catégorie avec leurs combinaisons de touches.](../../img/studio-shortcuts.png)

`Ctrl+,` (ou **Keys**) ouvre l'éditeur de raccourcis : cliquez sur une combinaison, appuyez sur la nouvelle combinaison de touches. Un conflit nomme l'autre commande et ne change rien tant que vous n'avez pas
confirmé « Take it over ». Les combinaisons réservées au navigateur sont refusées : `Ctrl+W`, `Ctrl+T`, `Ctrl+N`, `Ctrl+Q`, `Ctrl+R`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+Shift+W`, `Ctrl+Shift+N`, `F5`, `F11`, `F12`,
`Alt+F4`. Les reliaisons sont stockées dans la configuration de l'interface et survivent à un rechargement. Le serveur rejette avec 400 une table comportant des conflits, une commande inconnue, une disposition incomplète ou un corps qui n'est pas un objet.

### Suggest

![Le panneau Suggest montrant trois candidats de pose intermédiaire classés (linear, pca, manifold) entre les images 12 et 24, chacun avec un score et un bouton Apply.](../../img/studio-suggest.png)

`Suggest` (`Alt+8`) propose des poses apprises à partir des propres poses clés du projet : une pose intermédiaire entre deux images, la complétion d'une pose partiellement fixée, ou « ce qui vient
généralement ensuite ». Sur le projet de démonstration, entre les images 12 et 24, il a classé `linear` 1,00, `pca` 1,00 et `manifold` 0,99, construits à partir de 5 poses sur 2 axes (99 % de la variance).
`Apply` écrit la pose sous forme de clés, en une seule étape d'annulation. Le même moteur est `osatura pose-suggest` et `/api/suggest`. Ce sont des méthodes classiques (mélange linéaire, ACP, une variété à
noyau, un petit réseau facultatif que vous entraînez vous-même) ; avec une poignée de poses, les trois s'accordent.

### Panneaux Live input et Webcam face

* **Live input** (`Alt+4`) : la souris au-dessus de l'image, les touches, la manette et Web MIDI sont reliés à des propriétés par une table ; `Record` fait le bake d'une performance en clés en une seule étape
  d'annulation. Web MIDI nécessite un navigateur qui le prend en charge (Chromium le fait).
* **Webcam face** (`Alt+5`) : les points de repère pilotent la même table de correspondances (source `face`, identifiants tels que `mouth_open`, `eye_r_open`, `head_yaw`). Les boutons sont
  *Start camera*, *Synthetic face*, *Synthetic face (detector)*, *Calibrate (hold neutral)* et *Face example mapping*. Le chemin par caméra nécessite une fonction fournisseuse de points de repère
  que vous fournissez (`window.osaturaLandmarkProvider`, voir le commentaire dans le source de la page) ; la vidéo ne quitte jamais la page et seuls les points de repère vont au serveur local.
  Les deux modes synthétiques fonctionnent d'emblée. *Synthetic face (detector)* rend la tête synthétique sur le serveur et la retrouve avec le détecteur intégré,
  qui a été **entraîné uniquement sur des visages synthétiques** (têtes stylisées, pas des photographies) ; voir la section sur le visage de [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Démo guidée

Le bouton **Demo** (commande `demo.panel`) ouvre une démo guidée en 13 étapes qui construit la personne qui marche de `osatura demo-person` : décor, corps, tête et visage, bras avec cinq doigts par main, squelette, jambes qui marchent, bras qui se balancent, tronc et tête, un clignement et un sourire, sol mobile, une vérification, et l'export. Chaque étape charge ce stade du projet dans l'éditeur par une seule opération de remplacement global ; vous pouvez donc le lire, le parcourir, sélectionner des os et des calques et lire la timeline. Les stades sont des extraits du projet terminé ; les commandes et leur sortie affichées dans le panneau ont été exécutées avec le vrai binaire.

* **Rien n'est écrit sur le disque.** Save (et Ctrl+S) est désactivé tant que la démo est chargée. Le seul chemin d'écriture du studio, `POST /api/save`, n'est jamais appelé par la démo.
* **Votre projet revient.** *Restore my project* remet en place le projet qui était ouvert auparavant (une seule opération de remplacement global, une seule étape d'annulation). Si vous fermez l'onglet ou rechargez la page pendant que la démo est chargée, le chargement suivant remarque que le serveur détient encore la démo, laisse Save désactivé et propose de nouveau *Restore my project* ; l'original est conservé pour cela dans le stockage local du navigateur.
* La démo refuse de démarrer dans une session de collaboration, car elle remplace tout le projet.
* Régénérez les stades avec `node scripts/build-studio-tour.mjs --bin target/release/osatura`, puis reconstruisez la CLI (la page est intégrée au binaire).

## 4. Raccourcis clavier (par défaut)

Voici la sortie de `osatura ui shortcuts` pour une configuration neuve : **72 raccourcis, 84 commandes, 0 conflit**. Les raccourcis appartiennent à un **contexte** :
`global` (partout), `canvas` (focus sur l'image) et `timeline` (focus sur la timeline). Les champs de texte gardent leurs propres touches.

### Global

| Raccourci | Commande | Raccourci | Commande |
|---|---|---|---|
| `Space` | lecture / pause (`playback.toggle`) | `Alt+B` | afficher/masquer les os |
| `Home` / `End` | première / dernière image | `Alt+G` | activer/désactiver les groupes entiers |
| `,` / `.` | image précédente / suivante | `Alt+K` | activer/désactiver l'auto-key |
| `Shift+,` / `Shift+.` | reculer / avancer de 10 images | `Alt+M` | afficher/masquer les trajectoires de mouvement |
| `[` / `]` | définir l'entrée / la sortie de la plage | `Alt+N` | activer/désactiver l'onion skin |
| `Alt+R` | activer/désactiver la plage de lecture | `Alt+O` | activer/désactiver la lecture en boucle |
| `Alt+1` | afficher/masquer le panneau Calques | `Alt+P` | afficher/masquer les points de contrôle |
| `Alt+2` | afficher/masquer le panneau Sélection | `Alt+Q` | activer/désactiver les proxies de scrub |
| `Alt+3` | afficher/masquer le panneau Os | `Alt+S` | activer/désactiver l'aimantation du canevas |
| `Alt+4` | afficher/masquer le panneau Live input | `Alt+T` | afficher/masquer le panneau Timeline |
| `Alt+5` | afficher/masquer le panneau Webcam face | `Alt+V` | basculer dope sheet / graphe |
| `Alt+6` | afficher/masquer le panneau JSON | `Alt+Shift+S` | activer/désactiver l'aimantation de la timeline |
| `Alt+7` | panneau Overlays | `Alt+=` / `Alt+-` | zoom avant / arrière de la timeline |
| `Alt+8` | panneau Suggest | `Ctrl+S` | enregistrer |
| `Ctrl+K` / `Ctrl+Shift+P` | palette de commandes | `Ctrl+,` | raccourcis clavier |
| `Ctrl+Z` | annuler | `Ctrl+Y` / `Ctrl+Shift+Z` | rétablir |

### Canvas

| Raccourci | Commande |
|---|---|
| `Arrow keys` | déplacer la sélection de 1 unité (`Shift` : 10 unités) |
| `Delete` / `Backspace` | supprimer la sélection |
| `Escape` | effacer la sélection |
| `Ctrl+A` | tout sélectionner |
| `Alt+L` | verrouiller / déverrouiller les calques sélectionnés |
| `I` | ajouter une clé à la tête de lecture (`anim.add_key`) |

### Timeline

| Raccourci | Commande |
|---|---|
| `ArrowLeft` / `ArrowRight` | image précédente / suivante (`Shift` : 10) |
| `-` / `=` | zoom arrière / avant |
| `F` | zoom pour tout afficher |
| `Delete` / `Backspace` | supprimer les clés sélectionnées |
| `Escape` | désélectionner les clés |
| `Ctrl+A` | sélectionner toutes les clés |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | copier / coller / dupliquer les clés |
| `Alt+ArrowLeft` / `Alt+ArrowRight` | déplacer les clés sélectionnées d'une image plus tôt / plus tard |
| `Alt+ArrowUp` / `Alt+ArrowDown` | augmenter / diminuer les clés sélectionnées (leur valeur) |
| `I` | ajouter une clé à la tête de lecture |

Modificateurs sur le canevas : **Shift** contraint (verrouillage d'axe lors d'un déplacement, pas de 15 degrés lors d'une rotation, proportionnel lors d'une mise à l'échelle, l'extrémité d'un os ne fait alors que pivoter,
et Maj-clic bascule la sélection tandis qu'un cadre y ajoute) ; **Alt** met à l'échelle depuis le centre (et Alt-glisser sur un espace vide trace un lasso) ; **Ctrl** (Cmd sur macOS)
désactive l'aimantation.

## 5. Sélection et manipulateurs

Un clic choisit l'élément le plus haut sous le pointeur : un calque, un os (sa capsule, un peu plus large que la ligne dessinée) ou, avec `points` activé, un point de contrôle. Les calques verrouillés et masqués
sont ignorés, de sorte qu'un clic traverse vers ce qui se trouve dessous. Un cadre sélectionne selon le mode du sélecteur `box` : `touches` (tout ce que le cadre recouvre), `contains` (seulement ce qui se trouve entièrement à l'intérieur) ou `bounds touch` (les boîtes englobantes se recouvrent) ; Alt-glisser trace un lasso à la place.

Pour un seul calque, la boîte de transformation suit les axes propres du calque : 8 poignées mettent à l'échelle (les bords sur un axe, les coins sur les deux ; faire glisser au-delà du bord opposé retourne avec une échelle négative),
le bouton rond pivote autour du centre, et glisser à l'intérieur déplace. Pour un os, le cercle de la tête déplace l'os et l'extrémité le fait pivoter (et l'étire sauf si Maj est maintenu).
Pour les points de contrôle (ancres, poignées de Bézier, points de déformateur, sommets de maillage), le même manipulateur déplace les points sélectionnés. L'aimantation couvre une grille (désactivée par défaut), les bords et centres des autres calques
et les bords du canevas, dans un rayon de 6 unités de canevas. **Un glissement est une étape d'annulation** (une opération par lot).

## 6. Auto-key : exactement ce qui reçoit des clés

Le fait qu'une modification sur le canevas change une valeur *statique* ou crée des *clés* se décide ainsi. Je l'ai vérifié avec l'API HTTP sur le projet de démonstration (le `mode` de chaque modification est rapporté) :

| Situation | Résultat |
|---|---|
| Modification d'un calque, `auto-key` désactivé, la propriété n'a pas de canal à clés | changement **statique** de la transformation du calque (aucun canal n'est créé) |
| Modification d'un calque, `auto-key` activé | les propriétés modifiées reçoivent des **clés** à l'image courante (de nouveaux canaux `layer_x` et `layer_y` ont été créés : de 14 à 16 canaux) |
| Propriété de calque ou de point qui **a déjà un canal à clés**, `auto-key` désactivé | reçoit tout de même une **clé** à l'image courante (une modification à l'image 9 a ajouté une seconde clé aux canaux de l'image 5) |
| Modification d'un **os** | toujours **avec clés**, quel que soit le réglage `auto-key` (des canaux de décalage d'os ont été créés : de 16 à 18) |

Ainsi, une propriété animée ne peut jamais être écrasée silencieusement par une valeur statique, et une pose d'os n'est jamais « perdue » parce que vous avez oublié la bascule. Activez `auto-key` (`Alt+K`)
quand vous voulez animer une propriété encore statique. La touche `I` (`anim.add_key`, « Add key at playhead ») ajoute une clé à l'image courante.

## 7. Annuler et rétablir

Annuler et rétablir (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) agissent sur des opérations entières : un glissement de manipulateur, un déplacement par touches fléchées, une modification de clé et un Apply sont chacun une étape. Sans session partagée, elles
sont linéaires. **Dans une session partagée, l'annulation est propre à chaque utilisateur** : elle n'annule que votre propre dernière modification et laisse en place celles des autres (section 8). L'historique d'annulation n'est pas enregistré avec le
projet et est perdu à l'arrêt du serveur.

## 8. Édition partagée (collaboration)

### Pas à pas : deux personnes, un projet

![Deux utilisateurs de navigateur dans une session : la barre d'outils montre deux pastilles de participants, Ana (vous) à l'image 14 et Bo à l'image 0, et le cadre de sélection et le curseur de Bo sont dessinés sur le canevas autour du calque leg.](../../img/studio-collab.png)

1. Démarrez le studio. Pour des personnes sur d'autres machines, vous devez ajouter un jeton et vous lier à une adresse réseau :
   `osatura studio demo.json --bind 0.0.0.0 --port 8080 --collab-token $(openssl rand -hex 16) --collab-log session.log`,
   de préférence derrière un proxy TLS (section 2). Sur une seule machine, aucun jeton n'est nécessaire.
2. Dans le premier navigateur, tapez votre nom à côté de **Collab** (« Ana ») et cliquez sur **Collab**. Le bouton devient **Leave** et une pastille apparaît pour vous. Dans le second navigateur,
   tapez « Bo » et rejoignez. Chaque pastille indique l'image courante du participant (`Ana (you) @14`, `Bo @0`).
3. Sélectionnez des éléments. L'autre personne voit votre cadre de sélection et votre curseur dans votre couleur, avec votre nom dessus (Bo a sélectionné `leg` et déplacé le pointeur ; Ana voit un cadre violet
   nommé « Bo » et un point à côté de `leg` dans la liste des calques).
4. Modifiez en même temps. Dans le test : Ana a déplacé le calque du bras de 10 unités (`Shift+ArrowRight`) ; Bo a vu `[10,0]` apparaître dans `/api/project`. Bo a ensuite masqué le calque `sun`, ce qu'Ana a vu
   immédiatement.
5. Ana a appuyé sur `Ctrl+Z`. **Seul le déplacement d'Ana a été annulé** (le bras est revenu à `[0,0]`) ; le soleil masqué par Bo est resté masqué.
6. Les modifications faites alors que quelqu'un travaillait à partir d'une vue plus ancienne sont fusionnées, pas rejetées : j'ai exécuté la simulation propre au projet (`osatura collab-sim`, 6 répliques, 600 modifications
   concurrentes aléatoires, 12 autres ordres de livraison) et toutes les répliques se sont retrouvées identiques et le projet fusionné a compilé.
7. Cliquez sur **Enregistrer** (ou `Ctrl+S`) dans l'un ou l'autre navigateur pour écrire l'état partagé dans le fichier de projet ; rien n'est écrit dans le fichier avant cela.

Seul Chromium a été testé, avec deux pages sur une seule machine. Je n'ai pas testé de machines différentes, de liaisons lentes ni plus de deux vrais navigateurs.

### Persistance : `--collab-log`

```
$ osatura studio demo.json --collab-log session.log
...
$ # (an edit is made in a session, the server is stopped and started again)
collaboration log session.log: resumed at index 1 (1 envelopes after the checkpoint)
```

Le journal est un fichier de lignes (mode `0600`, réservé au propriétaire) qui contient un point de contrôle chaîné par hachage de **l'ensemble du projet** (environ 178 Ko pour la démo) plus une ligne par modification acceptée. Les ajouts sont
synchronisés par `fsync` à chaque requête, sauf avec `--collab-nosync`. Après un redémarrage, l'état de la session est repris (j'ai changé le nom du projet dans une session ; après le redémarrage, `/api/project` avait toujours le
nouveau nom) tandis que le fichier sur disque était inchangé (`OSATURA demo`) jusqu'à l'enregistrement. **Les participants et l'historique d'annulation ne sont pas conservés** : les pages rejoignent de nouveau avec de nouveaux identifiants. Un journal endommagé ou un journal
d'un autre projet arrête le serveur avec une erreur au lieu d'être écrasé. Un plantage pendant un ajout fait perdre au plus une dernière ligne tronquée.

**Le journal contient l'intégralité de votre projet en clair ; protégez-le comme le projet lui-même.**

## 9. Référence de l'API HTTP

La page du studio utilise ces routes, et vous pouvez les utiliser aussi (scripts, tests). Tous les corps de requête et les réponses sont en JSON sauf indication contraire ; les erreurs sont `{"ok": false, "errors": ["..."]}` avec le statut 400 (ou 401, 403,
404, 415, 421, 429, 503 comme décrit à la section 2). Les réponses de succès portent `"ok": true` là où c'est indiqué. `state` ci-dessous désigne le document renvoyé par `GET /api/state` :
`{bones:[{name,rest_angle}], channels:[...], fps, frames, width, height, can_undo, can_redo, rev, ok}` ; `rev` augmente à chaque modification. Chaque route est soumise aux vérifications de la section 2.

### Lecture

| Route | Paramètres | Réponse |
|---|---|---|
| `GET /` | | la page du studio (HTML) |
| `GET /source` | | la page de l'offre de sources AGPL ; accessible sans jeton |
| `GET /api/state` | | `state` |
| `GET /api/project` | | le JSON complet du projet |
| `GET /api/layers` | | `{ok, layers:[{path, depth, kind, visible, locked, locked_eff, reference, ptr}]}` (`locked_eff` tient aussi compte des groupes verrouillés situés au-dessus) |
| `GET /frame/N.svg` | `bones=1` dessine le squelette | une image en SVG (N peut être fractionnaire) ; en-têtes `X-Cache`, `X-Render-Ms` |
| `GET /api/frame` | `frame`, `proxy` 0-4 (0 = pleine qualité, 1-4 = proxys de scrub), `kind` = `vector` (par défaut) \| `raster` \| `png`, `bones=1`, `lod=1&screen=WxH&lodpx=P` (dessiner à partir d'une copie à détail réduit à P pixels près, 0-16) | l'image ; en-têtes `X-Cache` (hit/miss), `X-Render-Ms`, `X-Lod-Level`, `X-Lod-Bound-Px`, `X-Proxy-Level`. Une image hors plage : `{"errors":["frame 999 out of range"]}` ; un type invalide : `bad kind 'x' (vector, raster, png)` |
| `GET /api/cache` | | `{ok, revision, entries, bytes, hits, misses, invalidations, lod:{builds, engines, invalidations, last_build_ms, levels, unavailable}}` |
| `GET /api/overlays` | `frame`, `sel` (noms de calques séparés par des virgules) | `{ok, frames, settings, ...}` plus les fantômes et le SVG de l'image lorsqu'une image est donnée |
| `GET /api/sounds` | | `{ok, total, sounds:[{name, start, gain, duration, peaks, error?}]}` pics de la forme d'onde après rognage, fondus et gain |
| `GET /api/vcs/log` | `limit` (max 1000), `rev` | `{ok, branch, commits:[{hash, author, time, message, parents}]}` ; `{"ok":false,"errors":["no .osatura-vcs repository found ..."]}` sans dépôt (voir `osatura vcs`) |
| `GET /api/vcs/diff` | `a` (par défaut `HEAD`), `b` | `{ok, changes}` |
| `GET /api/timeline` | | `{ok, fps, frames, groups, rows, tracks}` le modèle de la dope sheet |
| `GET /api/graph` | `ch` (indices de canaux séparés par des virgules), `f0`, `f1`, `n` (échantillons, 2-2000, 200 par défaut) | `{ok, channels, f0, f1, range, skipped}` échantillons de courbe et poignées de tangente |
| `GET /api/live` | | `{ok, mapping:{params,triggers}, recorded, recording}` |
| `GET /api/ui` | | `{ok, layout, presets, preset_names, default_layout, shortcuts, commands, conflicts, reserved, overlays, recents, panels, persisted, path, warning, ...}` |
| `GET /api/ui/search` | `q`, `limit` (1-200, 50 par défaut) | `{hits:[{id, title, category, description, bindings, score, positions}]}` le classement de la palette |
| `GET /export.gif`, `/export.html`, `/export.html.nosound`, `/export.lottie.json` | | le fichier, en téléchargement (`animation.gif`, `player.html`, `animation.json`) |

### Édition

| Route | Corps | Réponse |
|---|---|---|
| `POST /api/op` | `{"op":"set","path":"/fps","value":30}` — `op` vaut `set`, `insert` (chemin de tableau, `.../-` ajoute à la fin), `remove` ou `batch` (`{"op":"batch","ops":[...]}`, une seule étape d'annulation). Les chemins sont des JSON Pointers dans le projet. | `state`. Un résultat invalide est refusé avec `{"errors":["invalid project: invalid value: integer `-3`, expected u32"]}` et rien ne change |
| `POST /api/undo`, `POST /api/redo` | aucun | `state` (par utilisateur dans une session partagée) |
| `POST /api/key` | `{"target","prop","frame","value","ease"?}` (`ease` du type `{"type":"smooth"}` ; linéaire par défaut) | `state` |
| `POST /api/delete_key` | `{"target","prop","frame"}` | `state` |
| `POST /api/save` | aucun | `{"ok":true,"saved":"<path>"}` ; écrit de façon atomique. Réservé à l'hôte lorsque le studio tourne avec `--collab-host` (sinon **403**) |
| `POST /api/suggest` | `{"kind":"between"\|"complete"\|"next","a","b","t","frame","constraints","free","top","snap","prior","apply":RANK, "library"?, "model"?}` | `{ok, suggestions}` ; avec `apply`, aussi le `state` de la modification. Une requête sans `kind` donne `bad request: missing field `kind`` |

### Sélection et gizmos (sans état : c'est la page qui conserve la sélection)

Une **cible** est `{"kind":"layer","path":"arm"}`, `{"kind":"bone","name":"hand"}` ou `{"kind":"point","layer":"...","point":...}`.

| Route | Corps | Réponse |
|---|---|---|
| `POST /api/pick` | `{"frame","x","y","options"?}` (`options` : `tolerance`, `bone_radius`, `point_radius`, `layers`, `bones`, `point_layers`, `locked`, `locked_bones`) | `{ok, hit, hits:[{distance, target}]}` l'élément le plus haut en premier |
| `POST /api/marquee` | `{"frame","rect":[x0,y0,x1,y1]` ou `"lasso":[[x,y],...]` (3 à 5000 points)`,"mode":"bbox"\|"contain"\|"intersect","options"?}` | `{ok, targets}` |
| `POST /api/select` | `{"frame","selection":[...],"targets":[...],"mode":"replace"\|"add"\|"toggle"\|"subtract"}` | `{ok, selection, bounds, gizmo:{corners, handles}, points}` |
| `POST /api/gizmo` | `{"frame","selection","handle","start":[x,y],"delta":[dx,dy],"modifiers":{"constrain","center","no_snap"},"snap":{"grid","objects","canvas","tolerance"},"auto_key","locked","pivot","commit":bool}` — `handle` vaut `{"type":"move"}`, `rotate`, `{"type":"scale","at":"nw"}`, `bone_head` ou `bone_tip` | sans `commit` : `{ok, drag:{changed, edits, guides, matrix, summary, box_after, outline}}` (un aperçu) ; avec `commit:true` : `state` plus `drag` et `selection`, une seule étape d'annulation |
| `POST /api/nudge` | `{"frame","selection","delta":[dx,dy],"auto_key","locked"}` | `state` + `drag` (modifications avec `mode` valant `static` ou `key`) + `selection` |
| `POST /api/delete` | `{"frame","selection"}` | `state` + `selection` ; `{"errors":["nothing selected"]}` si la sélection est vide |

### Timeline

| Route | Corps | Réponse |
|---|---|---|
| `POST /api/timeline/select` | `{"selection":[{ch,frame}...],"mode":"replace"\|"add"\|"toggle"\|"subtract","all":true` ou `"channel":N` ou `"group":N` ou `"box":{f0,f1,row0,row1}`, `"include_locked"}` | `{ok, selection:[{ch,frame}...]}` |
| `POST /api/timeline/op` | `{"op":{...},"preview":bool}` — `op` est l'une de `move_keys`, `scale_keys`, `copy`, `paste`, `duplicate`, `delete`, `set_ease`, `set_value`, `add_key`, `ripple`, `offset`, `extrapolate`, `make_loop`, `handle`, `smooth`, `flags` (un nom inconnu renvoie cette liste) | avec `preview:true` : `{ok, preview:true, plan}` sans rien modifier ; sinon `state` + `plan:{changed, notes, op, clip}` |

Par exemple, `{"preview":true,"op":{"op":"delete","keys":[{"ch":0,"frame":12}]}}` renvoie le plan (`changed: 1`) et laisse le projet intact.

### Overlays, live, visage, interface

| Route | Corps | Réponse |
|---|---|---|
| `POST /api/overlays` | `{"settings":{"onion":{"enabled":true,"before":3}},"frame","selection":[names],"reset":bool}` | `{ok, frames, ghosts, settings, svg}` ; les réglages sont enregistrés dans la configuration de l'interface |
| `POST /api/trails` | `{"targets":[{"kind":"bone","name":"hand"}],"from","to"` ou `"frame","before","after"`, `"smooth","svg":bool,"hit":{x,y,radius}}` | `{ok, from, to, trails:[{curve,frames,keys,...}], svg?, hit?, path_hit?}` |
| `POST /api/live/mapping` | `{"mapping":{params,triggers},"library"?}` (même format que `live-map`) | état du live |
| `POST /api/live/push` | `{"events":[{source,id,value,time}],"midi":[raw...]}` | état du live avec l'effet des événements |
| `POST /api/live/record` | `{"on":true}` démarre ; `{"on":false,"tol":0.01}` arrête et **intègre la performance au projet** en une seule étape d'annulation | statut / `state` + résumé `live` |
| `POST /api/face/config` | `{"config"?,"calibration"?}` | `{ok}` |
| `POST /api/face/calibrate` | `{"on":true}` / `{"on":false}` | `{ok, frames, calibration}` |
| `POST /api/face/push` | `{"frames":[...landmarks...],"layout"?,"size"?}` (au plus 10000 images par envoi) | état du live |
| `POST /api/face/detect` | `{"image":"<base64 PNG/JPEG or data: URI>","track":bool,"push":bool,"t"}` | `{ok, found, layout:"osatura23", size, points, box, note:"detector trained on synthetic faces only"}` |
| `POST /api/face/synth` | `{"t","size":32-512,"seed"}` | `{ok, image:"data:image/png;base64,...", layout, size, points}` une image de la tête synthétique avec ses points de repère exacts |
| `POST /api/ui` | n'importe lesquels de `{"layout":{...}\|null,"shortcuts":{...}\|null,"recents":[...],"overlays":{...},"reset":"layout"\|"shortcuts"\|"overlays"\|"all"}` | le document de `GET /api/ui` ; les champs inconnus (`unknown field 'bogus'`), les conflits et les dispositions incomplètes sont refusés et rien ne change |
| `POST /api/login` | `{"token":"..."}` | `{ok:true}` et `Set-Cookie: osatura_token=...; Path=/; HttpOnly; SameSite=Strict` ; jeton erroné : 401 (429 dès le 5e échec) ; sans jeton configuré : `{"ok":true,"token_required":false}` |

### Collaboration

Les appels effectués en tant que participant envoient `X-Collab-Replica: <id>` et `X-Collab-Secret: <secret>` (tous deux renvoyés par join). Un identifiant de réplique avec un secret erroné ou absent reçoit **403**.

| Route | Corps / requête | Réponse |
|---|---|---|
| `POST /api/collab/join` | `{"name":"Ana","mode":"thin"\|"replica"}` | `{ok, replica, secret, colour, name, index, mode, clock, users, presence, doc}` (`snapshot` en mode replica). Le premier join démarre la session à partir du document de l'éditeur |
| `POST /api/collab/leave` | `{"replica"}` | `{ok, left}` |
| `GET /api/collab/state` | `replica`, `doc=1`, `snapshot=1` | `state` + `collab_active` (+ users, presence) |
| `GET /api/collab/log` | `since=N` | `{ok, index, entries:[{index, by, envelope}]}` |
| `POST /api/collab/ops` | `{"replica","ops":[...]` ou `"op":{...}`, `"base"?}` ou `{"replica","envelopes":[...]}` | `state` + `applied`, `index`. Participant inconnu : `unknown participant N: join first` ; liste vide : `no ops` ; les modifications de projet invalides sont refusées |
| `POST /api/collab/undo`, `.../redo` | `{"replica"}` | `state` + `collab_can_undo`, `collab_can_redo` (vos propres modifications uniquement) |
| `POST /api/collab/presence` | `{"replica","n","frame","cursor":[x,y],"selection":[names up to 64]}` | `{ok}` |
| `POST /api/collab/compact` | `{}` | résumé du compactage (`ok`, `logged`). Réservé à l'hôte lorsque le studio tourne avec `--collab-host` (sinon **403**) |
| `POST /api/collab/revoke` | `{"names":["ana","bob"]}` (1 à 64 noms) | `{ok, revoked, revoked_total}` : les jetons de ces noms sont refusés à partir de maintenant et les participants qui ont rejoint sous ces noms sont retirés. Nécessite des jetons par participant (`--collab-secret-file`) et un jeton d'hôte (`--collab-host`), ou le `--collab-token` partagé ; sinon **403**. Un flux d'événements déjà ouvert se termine lorsque son client se déconnecte |
| `GET /api/collab/events` | `since=N` ou `Last-Event-ID` | un flux **server-sent events** : relecture à partir de `since`, puis utilisateurs, présence et chaque événement ultérieur ; `resync` si le client a trop de retard ; au plus 64 flux (`503 too many event streams`) |

Dans une session active, les routes d'édition ordinaires (`/api/op`, `/api/undo`, `/api/redo`, ainsi que les propres modifications du studio via nudge, key ou gizmo) sont également enregistrées pour les autres participants.

## 10. Dépannage

| Symptôme | Cause et solution |
|---|---|
| Le navigateur affiche 421 ou « misdirected request » | Vous avez ouvert le studio sous un nom qu'il ne connaît pas (un nom DNS, une adresse du réseau local). Ajoutez-le avec `--allow-host name` ou ouvrez `http://127.0.0.1:PORT/`. |
| 403 à chaque requête derrière un proxy | Le proxy modifie `Origin`/`Host`. Indiquez l'origine publique : `--public-origin https://studio.example.org`. |
| 401 | `--collab-token` est défini : ouvrez la page et saisissez le jeton une fois, ou envoyez `X-Collab-Token`. Un jeton dans l'URL n'est pas accepté. |
| 429 | Cinq jetons erronés d'affilée depuis votre adresse ; attendez jusqu'à 30 s. Derrière un proxy, tous les utilisateurs partagent une même adresse. |
| `refusing to listen on 0.0.0.0:...` | Écoute sur une interface autre que loopback sans jeton. Ajoutez `--collab-token` (16 caractères ou plus). |
| `cannot listen on 127.0.0.1:8080: Address already in use` | Un autre processus occupe le port ; choisissez `--port`. Un studio résiduel d'une exécution précédente en est la cause habituelle. |
| La page se charge mais l'image est vide | Consultez le terminal ; un projet qui échoue à `check` est refusé au démarrage. Rechargez après correction. |
| Un message de la console du navigateur `Failed to load resource ... 404` | Les anciennes versions n'avaient pas de `/favicon.ico` ; il est maintenant servi. Un 404 qui subsiste provient d'autre chose. |
| Le texte du panneau Face | Le panneau indique désormais que le détecteur entraîné sur des données synthétiques est inclus, qu'il n'est pas validé sur des photos ni des webcams, et qu'une vraie webcam nécessite votre propre fournisseur de points de repère. |
| La disposition est cassée ou des panneaux manquent | Menu `Layout`, réinitialiser la disposition, ou supprimez le `ui.json` indiqué au démarrage. Un fichier de configuration qui ne peut pas être lu entièrement est copié vers `ui.json.bak` (puis `.bak.1`, ...) avant l'utilisation des valeurs par défaut, et `GET /api/ui` comporte un champ `warning` à ce sujet. |
| Mes modifications ont disparu après un redémarrage | Rien n'est écrit tant que vous n'avez pas fait **Save**. Avec `--collab-log`, la *session* survit à un redémarrage, mais le fichier sur disque n'est toujours écrit que par Save. |
| `Ctrl+W`, `Ctrl+T` et consorts ne peuvent pas être associés | Réservés par les navigateurs (liste à la section 3). |
| Ctrl+Z n'annule pas la modification de quelqu'un d'autre dans une session | C'est voulu : l'annulation est propre à chaque utilisateur. |
| Les exports demandent `ffmpeg` | Uniquement pour le son dans le HTML (mp3) et pour mp4/webm en ligne de commande ; GIF, Lottie et HTML sans son n'ont besoin de rien. |

## 11. Accessibilité et compatibilité des navigateurs

**Navigateurs.** L'interface n'a été testée que dans **Chromium** (la version fournie avec Playwright, pilotée en mode headless, sous Linux). Firefox et Safari n'ont pas été essayés et peuvent fonctionner ou non.
Web MIDI nécessite un navigateur qui l'implémente (Chromium le fait) ; la caméra nécessite `getUserMedia` et un contexte sécurisé (HTTPS ou `localhost`). La page utilise des server-sent events pour la
session partagée. Les événements de pointeur servent au glisser-déposer ; les écrans tactiles n'ont pas été testés.

**Accessibilité, en toute honnêteté.** Je n'ai pas audité le studio avec un lecteur d'écran ni lors d'une session uniquement au clavier. Ce qui existe : un attribut `lang` et une balise meta viewport, des cases à cocher étiquetées, une
notification `role="status"`, quelques attributs ARIA, **un ensemble complet de raccourcis clavier et une palette de commandes** qui atteignent chaque commande, des raccourcis reconfigurables, et la ligne de commande `osatura`, qui fait
tout ce que fait l'éditeur sans pointeur. Ce qui manque : le canevas est une image et la plupart des modifications sur le canevas se font au pointeur (le déplacement par touches fléchées existe), il n'y a ni thème clair ni
prise en charge de `prefers-reduced-motion` ou de `prefers-color-scheme`, les contours de focus sont ceux par défaut du navigateur et je ne les ai pas vérifiés, et je n'ai pas mesuré les contrastes de couleur. Si vous dépendez de technologies d'assistance,
préférez la CLI et le panneau JSON, et signalez au projet ce qui dysfonctionne.

## 12. Défauts connus

* Corrigés depuis la première rédaction : le texte du panneau Webcam face, le 404 de `/favicon.ico`, `--collab-nosync` sans `--collab-log`, et l'absence du drapeau de cookie `Secure` derrière une `--public-origin` en https.
* `osatura render p.json 99 o.png` sur un projet de 24 images est désormais aussi une erreur (ajoutez `--clamp` pour rendre la dernière image) ; `/api/frame` du studio refuse une image hors plage (`frame 999 out of range`) et n'accepte désormais que les images `0..frames-1`.
* Les modifications de disposition et de raccourcis sont stockées par utilisateur dans la configuration de l'interface, et non dans le projet ; elles ne voyagent donc pas avec le fichier.
