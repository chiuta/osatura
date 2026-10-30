# CI d'animation (`osatura ci`)

Traitez une animation comme du code testable : un manifeste liste des vérifications, un job de CI les exécute, et une
modification visuelle en échec s'accompagne d'un rapport qui montre ce qui a bougé. Tout est local, hors ligne et
déterministe ; les métriques sont des classiques (CIEDE2000, SSIM, MSE/PSNR, un hash DCT).

```
osatura ci init  <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run   [manifest] [--update] [--bless] [--prune] [--only a,b]
                  [--junit out.xml] [--html out.html] [--json out.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest] [--only a,b] [--prune]
osatura ci diff  a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--pixel-de D] [--aa N]
                  [--ignore x,y,w,h;x,y,w,h] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```

Codes de sortie : **0** toutes les vérifications ont réussi (ou seulement info / mis à jour / bénies), **1** au moins une vérification
a échoué (ou `ci diff` a trouvé une différence), **2** quelque chose n'a pas pu être évalué (projet manquant,
manifeste invalide, golden illisible ou altéré, option inconnue).

## Démarrage rapide

```
osatura ci init animation.json        # writes osatura-ci.json and goldens/ next to the project
osatura ci run --html report.html     # verify; exit 1 and a report on regressions
osatura ci bless                      # after reviewing the report: accept the new look
git add osatura-ci.json goldens       # commit the goldens with the change that caused them
```

`init` calibre le manifeste sur le projet tel qu'il est maintenant : environ six images régulièrement espacées à une échelle de
proxy (plus grand côté d'au plus 480 px), une vérification du moteur de rendu en virgule flottante sur deux d'entre elles, le déterminisme,
la validation, un plafond du taux d'échec du pose-fuzz (taux mesuré plus 2 points ; le rig de démo échoue sur la plupart des
échantillons pleine plage, son plafond est donc de 100 % et la vérification ne protège rien tant que vous ne l'abaissez pas),
des limites avec 50 % de marge, un budget de taille GIF (+50 %), une vérification de la jointure de boucle (uniquement lorsque le projet
boucle proprement aujourd'hui), et un budget informatif de vitesse de rendu (4 x la médiane mesurée, qui
dépend de la machine et des builds debug par opposition à release).

## Manifeste (`osatura-ci.json`)

```json
{
  "schema": "osatura-ci/1",
  "project": "animation.json",
  "goldens": "goldens",
  "defaults": { "max_de": 6.0 },
  "checks": [
    { "kind": "validate", "name": "validate" },
    { "kind": "render", "name": "frames", "frames": [0, 12, 24, 47], "scale": 0.5,
      "thresholds": { "min_ssim": 0.995 }, "ignore": [[500, 0, 140, 40]] },
    { "kind": "render", "name": "tablet", "step": 12, "size": "1024x768", "scale": 0.5 },
    { "kind": "render", "name": "svg-text", "frames": [0], "renderer": "svg" },
    { "kind": "determinism", "name": "determinism", "frames": [0, 47] },
    { "kind": "fuzz", "name": "pose-fuzz", "samples": 200, "max_fail_rate": 0.05 },
    { "kind": "limits", "name": "limits", "max_layers": 200, "max_file_bytes": 2000000 },
    { "kind": "export-size", "name": "gif", "format": "gif", "max_bytes": 800000, "scale": 0.5 },
    { "kind": "loop", "name": "loop-seam" },
    { "kind": "perf", "name": "speed", "budget_ms": 40, "tolerance": 0.5, "enforce": false }
  ]
}
```

Les chemins sont relatifs au manifeste. Les champs et les kinds inconnus sont des erreurs ; tous les problèmes d'un manifeste
sont signalés ensemble (les noms doivent comporter de 1 à 64 caractères parmi lettres, chiffres, `-`, `_`, `.` ; les noms sont
uniques ; chaque vérification nécessite un projet ; les plages sont vérifiées).

| kind | ce qu'il vérifie | échoue lorsque |
| --- | --- | --- |
| `render` | images (liste `frames`, ou `step`, par défaut première/milieu/dernière), `scale`, `size` (les conteneurs de mise en page sont résolus à nouveau), `renderer` : `raster` (tiny-skia), `float` (rastériseur f32, quantifié sur 8 bits) ou `svg` (comparaison exacte du texte) | l'image diffère du golden au-delà des seuils, aucun golden n'existe, ou la taille a changé |
| `validate` | le projet se compile ; constats du linter à la sévérité `lint` (`error` par défaut, `warn`, `info`, `off`) | tout constat de ce type ou une erreur de compilation |
| `fuzz` | le pose fuzzer (`samples`, `seed`, `frame`) | taux d'échec supérieur à `max_fail_rate` (0 par défaut). La vérification **bloque par défaut** (`gate` non défini signifie `true`) ; elle est seulement rapportée, avec le statut `info` et sans jamais échouer, lorsque `"gate": false` ou que `max_fail_rate` vaut 1 ou plus (une borne qui ne peut pas échouer n'est jamais présentée comme une réussite). La commande `osatura fuzz` est un outil distinct, uniquement de rapport, sauf si on lui donne `--gate` ou `--max-fail-rate` |
| `limits` | `max_layers`, `max_bones`, `max_channels`, `max_frames`, `max_canvas_pixels`, `max_file_bytes` | un nombre dépasse sa limite |
| `export-size` | octets de `gif`, `apng`, `lottie`, `html` ou `svg-frames` (exporteurs existants) | supérieur à `max_bytes` |
| `determinism` | chaque image rendue deux fois par le même moteur et par un moteur fraîchement chargé ; le SHA-256 des pixels (ou du texte SVG) doit concorder ; `renderers` par défaut raster, float, svg | un condensat diffère |
| `loop` | jointure entre la dernière et la première image par rapport à la variation moyenne d'une image à l'autre (différence absolue moyenne de canal en pourcentage) ; `max_seam_ratio` 2 par défaut, `max_seam_mad` facultatif | jointure plus grande que permis ; au plus 600 images |
| `perf` | médiane de ms par image, `budget_ms` avec `tolerance` relative (0,5 par défaut) | uniquement lorsque `enforce` est vrai ; sinon statut `info` |

### Seuils de rendu

Définis dans `defaults` (toutes les vérifications de rendu) et surchargés par vérification dans `thresholds`.

| clé | valeur par défaut | signification |
| --- | --- | --- |
| `metric` | `de2000` | `de2000` (CIEDE2000) ou `de76` (CIE76, euclidien dans Lab) |
| `pixel_de` | 1.0 | un pixel est compté comme différent au-delà de cette différence de couleur |
| `max_de` | 8.0 | plus grande différence de couleur autorisée parmi les pixels comptés |
| `min_ssim` | 0.99 | plus petit SSIM moyen autorisé |
| `max_diff_fraction` | 0.001 | plus grande fraction autorisée de pixels comptés (0,1 %) |
| `aa_isolated` | 2 | tolérance d'anticrénelage : les groupes comptés d'au plus ce nombre de pixels 8-connexes, tous situés sur un bord, sont pardonnés (0 = désactivé) |
| `aa_edge_de` | 0 (désactivé) | les pixels de bord qui diffèrent d'au plus cette valeur sont pardonnés (plus souple que `pixel_de`, sur les bords uniquement) |
| `max_phash_distance` | non vérifié | plus grande distance de Hamming des hashs DCT de 64 bits |

Une image dont les pixels sont identiques octet pour octet au golden réussit sans aucune métrique. Sinon, l'image
est comparée pixel par pixel ; chaque pixel est composé sur du blanc et sur du noir, et la plus grande
différence est retenue, de sorte qu'un simple changement d'alpha est visible. Un pixel est un pixel de bord lorsque deux voisins opposés
(bornés à la bordure) dans l'une ou l'autre image diffèrent de plus de 40 dans un canal ; la valeur propre
du pixel est ignorée, de sorte qu'un pixel isolé dans une zone uniforme est une vraie différence et que le pourtour d'une forme
n'en est pas une. Les rectangles `ignore` sont `[x, y, w, h]` en unités de scène ; ils sont égalisés dans les deux images avant
toute métrique, et ne comptent donc pas non plus dans le SSIM, la MSE ou le hash.

À titre indicatif, sur le projet de démo, le rastériseur CPU et le rastériseur flottant (deux chemins de code différents pour la même scène) diffèrent d'un SSIM de 0,9997, de 0,83 % de pixels au-dessus d'une différence de couleur de 1 (386 de plus
pardonnés comme anticrénelage), avec une plus grande différence de couleur de 11,3 sur les bords des formes. Un déplacement de 40 unités du calque
soleil donne un SSIM de 0,993, 6,4 % de pixels, une plus grande différence de couleur de 19,6 dans une région.

### Goldens : `--update` ou `bless`

Les goldens résident dans `goldens/` sous forme de fichiers PNG nommés d'après les 24 premiers chiffres hexadécimaux d'un SHA-256 calculé sur la taille et les
pixels (les images identiques partagent un fichier), plus `goldens/index.json` qui associe vérification et image à un fichier.
Les goldens SVG sont des fichiers `.svg` nommés de la même façon. Un golden est vérifié par rapport à son nom lors de sa
lecture ; un fichier modifié à la main ou corrompu est donc une erreur (code de sortie 2), et non un succès silencieux.

* `ci run --update` réécrit chaque golden des vérifications de rendu sélectionnées à partir du rendu courant et
  oublie les images qui ne sont plus listées. Utilisez-le lorsqu'un manifeste a changé ou après un changement
  global voulu.
* `ci bless` (ou `ci run --bless`) n'accepte que les images qui échouent actuellement ou n'ont pas de golden et
  laisse intacts ceux qui réussissent, de sorte que le bruit de tolérance ne fasse pas varier les fichiers. Le rapport les marque
  `blessed`.
* `--prune` supprime les fichiers golden auxquels plus rien ne fait référence. Commitez `goldens/` avec la modification qui l'a causé.

### Rapports

`--json` est le résultat complet (schéma `osatura-ci-report/1` ; métriques, régions, seuils utilisés).
`--junit` est un XML JUnit avec un cas de test par vérification. `--html` est une page autonome (sans ressource
externe) : résumé, tableau des vérifications, et pour chaque image en échec le golden, le rendu courant et une
carte des différences sous forme d'URI de données PNG intégrées, un bouton d'échange pour alterner entre golden et courant, et
la liste des régions qui diffèrent. `--artifacts dir` écrit aussi `dir/<check>/fNNNNN-{golden,actual,diff}.png`
pour les images en échec. La carte des différences montre le golden estompé, les différences comptées dans un dégradé du jaune au
rouge sombre selon la différence de couleur, l'anticrénelage pardonné en bleu clair, les zones ignorées hachurées en gris
et les cadres de région en magenta. `ci diff` peut aussi écrire une bande côte à côte, une superposition rouge/cyan et un
GIF de scintillement à deux images ; utilisez `--flicker` sur une paire golden/actual pour un GIF avant/après.

## GitHub Actions (exemple, non exécuté)

Documentation uniquement : ce workflow a été écrit pour correspondre à la CLI mais n'a **pas** été exécuté sur un vrai service
de CI. Adaptez l'installation de Rust et les chemins à votre dépôt.

```yaml
name: animation-ci
on:
  pull_request:
  push:
    branches: [main]
jobs:
  animation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        run: rustup toolchain install stable --profile minimal
      - name: Build osatura
        run: cargo build --release -p osatura-cli
      - name: Animation checks
        run: |
          ./target/release/osatura ci run animation/osatura-ci.json \
            --junit ci-out/junit.xml --html ci-out/report.html --json ci-out/report.json \
            --artifacts ci-out/frames
      - name: Upload report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: animation-ci-report
          path: ci-out/
      - name: Publish test results
        if: always()
        uses: mikepenz/action-junit-report@v4   # any JUnit consumer works
        with:
          report_paths: ci-out/junit.xml
```

Les codes de sortie 1 (régression visuelle ou budget) et 2 (évaluation impossible) font tous deux échouer le job ; le rapport
est téléversé dans les deux cas. Pour accepter une modification voulue, exécutez `osatura ci bless` en local, examinez le
diff de `goldens/`, et commitez-le.

## Script shell générique (exemple, non exécuté)

```sh
#!/bin/sh
# ci.sh - run animation checks; works in any CI that can run a shell and keep files.
set -u
MANIFEST="${1:-animation/osatura-ci.json}"
OUT="${OUT:-ci-out}"
mkdir -p "$OUT"

osatura ci run "$MANIFEST" \
  --junit "$OUT/junit.xml" --html "$OUT/report.html" --json "$OUT/report.json" \
  --artifacts "$OUT/frames"
code=$?

case $code in
  0) echo "animation checks passed" ;;
  1) echo "animation regression: open $OUT/report.html" ;;
  *) echo "animation CI could not run (exit $code); see the log above" ;;
esac
exit $code
```

## Déterminisme et autres machines

Ce qui a été vérifié (une machine Linux x86-64, le rastériseur CPU `tiny-skia` et le rastériseur f32) :

* le rendu de la même image deux fois dans un processus, puis depuis un moteur fraîchement chargé, donne des octets
  identiques (la vérification `determinism` ; raster, float et texte SVG) ;
* les goldens écrits par un processus ont été retrouvés octet pour octet par des processus ultérieurs (`init` puis `run`,
  `bless` puis `run`), pour les deux moteurs de rendu ;
* des contrôles indépendants des métriques : les 24 paires CIEDE2000 de Sharma, Wu et Dalal (2005) utilisées dans les
  tests unitaires sont reproduites à 5e-5 près ; sur les images de la démo, le SSIM (une fois corrigé pour la région de bordure que
  scikit-image rogne), la plus grande valeur CIEDE2000 et le nombre de pixels différents concordent avec
  scikit-image 0.26 (0,99330 contre 0,99330 après la correction de bordure ; 19,648 contre 19,648 ; 11088
  contre 11088 pixels), et la MSE avec numpy.

Ce qui n'a **pas** été vérifié : d'autres systèmes d'exploitation, CPU, compilateurs ou versions de Rust. Les moteurs de rendu
utilisent la virgule flottante (trigonométrie, aplatissement des courbes, accumulation de la couverture), de sorte qu'une autre libm,
une contraction FMA ou un chemin SIMD différent peut déplacer d'un niveau ou deux un pixel de bord anticrénelé. Les hashs exacts
n'aident donc que sur une seule build de l'outil sur un seul type de machine ; d'une machine à l'autre, fiez-vous aux
tolérances (`pixel_de`, `aa_isolated`, `max_diff_fraction`) et conservez les goldens produits par la même
image de CI que celle qui les vérifie. Le moteur de rendu GPU ne fait pas partie de ce harnais.

## Limites

* Il n'y a pas de rastériseur SVG dans le workspace ; une vérification de rendu ne peut donc pas rastériser la sortie SVG et
  comparer les pixels ; le moteur de rendu `svg` compare exactement le texte SVG (tout réordonnancement d'attributs est un
  échec).
* Les surcouches d'os ne sont rendues par aucune vérification de CI ; l'audio n'est pas comparé.
* La fusion des régions et la règle d'anticrénelage sont des heuristiques (test de bord avec un pas fixe de 40, limite de taille
  de groupe) ; un décalage sub-pixel d'un long bord forme un long groupe étroit qui est signalé, et non pardonné.
* Les seuils perceptuels sont des valeurs par défaut, non calibrées sur le jugement d'animateurs.
* La vérification perf mesure le temps réel sur une machine partagée ; gardez-la informative sauf si le
  runner est dédié.
* `ci init` effectue un rendu, un fuzz et un export une fois pour calibrer, ce qui est lent dans les builds debug.
* Pas d'intégration aux pull requests (pas de commentaire de bot, pas de téléversement automatique d'un GIF avant/après) : les éléments
  (`--html`, `--artifacts`, `ci diff --flicker`) sont là, la colle vers un service d'hébergement ne l'est pas.
