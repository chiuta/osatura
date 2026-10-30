# CI pentru animație (`osatura ci`)

Tratează o animație ca pe un cod testabil: un manifest listează verificările, un job CI le rulează, iar o
schimbare vizuală care eșuează vine cu un raport care arată ce s-a mișcat. Totul este local, offline și
determinist; metricile sunt cele din manuale (CIEDE2000, SSIM, MSE/PSNR, un hash DCT).

```
osatura ci init  <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run   [manifest] [--update] [--bless] [--prune] [--only a,b]
                  [--junit out.xml] [--html out.html] [--json out.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest] [--only a,b] [--prune]
osatura ci diff  a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--pixel-de D] [--aa N]
                  [--ignore x,y,w,h;x,y,w,h] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```

Coduri de ieșire: **0** toate verificările au trecut (sau doar info / actualizat / blessed), **1** cel puțin o verificare
a eșuat (sau `ci diff` a găsit o diferență), **2** ceva nu a putut fi evaluat (proiect lipsă,
manifest greșit, golden ilizibil sau alterat, opțiune necunoscută).

## Pornire rapidă

```
osatura ci init animation.json        # writes osatura-ci.json and goldens/ next to the project
osatura ci run --html report.html     # verify; exit 1 and a report on regressions
osatura ci bless                      # after reviewing the report: accept the new look
git add osatura-ci.json goldens       # commit the goldens with the change that caused them
```

`init` calibrează manifestul pe proiectul așa cum este acum: aproximativ șase cadre echidistante la o
scală proxy (latura cea mai lungă de cel mult 480 px), o verificare cu randerul float pe două dintre ele, determinism,
validate, un plafon al ratei de eșec la pose-fuzz (rata măsurată plus 2 puncte; rig-ul demo eșuează la majoritatea
eșantioanelor din întreaga gamă, deci plafonul lui este 100 % și verificarea nu protejează nimic până când îl cobori),
limite cu 50 % marjă, un buget de dimensiune GIF (+50 %), o verificare a cusăturii buclei (doar când proiectul
se buclează curat în prezent) și un buget informativ de viteză de randare (de 4 ori mediana măsurată, care
depinde de mașină și de build-urile debug față de release).

## Manifest (`osatura-ci.json`)

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

Căile sunt relative la manifest. Câmpurile și tipurile (kind) necunoscute sunt erori; toate problemele dintr-un manifest
sunt raportate împreună (numele trebuie să aibă 1-64 de caractere: litere, cifre, `-`, `_`, `.`; numele sunt
unice; fiecare verificare are nevoie de un proiect; intervalele sunt verificate).

| kind | ce verifică | eșuează când |
| --- | --- | --- |
| `render` | cadre (lista `frames`, sau `step`, implicit primul/mijlocul/ultimul), `scale`, `size` (containerele de layout se rezolvă din nou), `renderer`: `raster` (tiny-skia), `float` (rasterizator f32, cuantizat la 8 biți) sau `svg` (comparare exactă de text) | imaginea diferă de golden peste praguri, nu există golden sau dimensiunea s-a schimbat |
| `validate` | proiectul se compilează; constatările linter-ului la severitatea `lint` (`error` implicit, `warn`, `info`, `off`) | orice astfel de constatare sau o eroare de compilare |
| `fuzz` | fuzzer-ul de poze (`samples`, `seed`, `frame`) | rata de eșec peste `max_fail_rate` (implicit 0). Verificarea **blochează implicit** (`gate` nesetat înseamnă `true`); este doar raportată, cu starea `info` și fără a eșua vreodată, când `"gate": false` sau `max_fail_rate` este 1 sau mai mare (o limită care nu poate eșua nu este afișată niciodată ca reușită). Comanda `osatura fuzz` este o unealtă separată, doar de raportare, dacă nu i se dă `--gate` sau `--max-fail-rate` |
| `limits` | `max_layers`, `max_bones`, `max_channels`, `max_frames`, `max_canvas_pixels`, `max_file_bytes` | un număr depășește limita lui |
| `export-size` | octeții pentru `gif`, `apng`, `lottie`, `html` sau `svg-frames` (exportatori existenți) | mai mare decât `max_bytes` |
| `determinism` | fiecare cadru randat de același motor de două ori și de un motor proaspăt încărcat; SHA-256 al pixelilor (sau al textului SVG) trebuie să coincidă; `renderers` implicit raster, float, svg | orice digest diferă |
| `loop` | cusătura dintre ultimul și primul cadru față de schimbarea medie de la un cadru la altul (diferența medie absolută pe canal, în procente); `max_seam_ratio` implicit 2, `max_seam_mad` opțional | cusătura este mai mare decât se permite; cel mult 600 de cadre |
| `perf` | mediana ms pe cadru, `budget_ms` cu `tolerance` relativă (implicit 0.5) | doar când `enforce` este true; altfel starea `info` |

### Praguri de render

Se stabilesc în `defaults` (toate verificările de render) și se suprascriu pe verificare în `thresholds`.

| cheie | implicit | semnificație |
| --- | --- | --- |
| `metric` | `de2000` | `de2000` (CIEDE2000) sau `de76` (CIE76, euclidian în Lab) |
| `pixel_de` | 1.0 | un pixel este considerat diferit peste această diferență de culoare |
| `max_de` | 8.0 | cea mai mare diferență de culoare permisă printre pixelii numărați |
| `min_ssim` | 0.99 | cel mai mic SSIM mediu permis |
| `max_diff_fraction` | 0.001 | cea mai mare fracțiune permisă de pixeli numărați (0.1 %) |
| `aa_isolated` | 2 | toleranța la anti-aliasing: grupurile numărate de cel mult atâția pixeli 8-conectați, toți aflați pe o margine, sunt iertate (0 = dezactivat) |
| `aa_edge_de` | 0 (dezactivat) | pixelii de margine care diferă cu cel mult atât sunt iertați (mai permisiv decât `pixel_de`, doar pe margini) |
| `max_phash_distance` | neverificat | cea mai mare distanță Hamming a hash-urilor DCT pe 64 de biți |

Un cadru ai cărui pixeli sunt identici octet cu octet cu golden-ul trece fără nicio metrică. Altfel,
cadrul este comparat pixel cu pixel; fiecare pixel este compus peste alb și peste negru și se folosește diferența
mai mare, astfel încât o schimbare doar a alpha-ului să fie vizibilă. Un pixel este pixel de margine când doi vecini opuși
(limitați la bordură), în oricare dintre imagini, diferă cu mai mult de 40 pe vreun canal; valoarea proprie
a pixelului este ignorată, deci un pixel rătăcit într-o zonă plată este o diferență reală, iar marginea unei forme
nu este. Dreptunghiurile `ignore` sunt `[x, y, w, h]` în unități de scenă; sunt egalizate în ambele imagini înainte de
orice metrică, deci nu contează nici în SSIM, MSE sau hash.

Orientativ, pe proiectul demo, rasterizatorul CPU și rasterizatorul float (două căi de cod diferite
pentru aceeași scenă) diferă prin SSIM 0.9997, 0.83 % dintre pixeli peste diferența de culoare 1 (alți 386
iertați ca anti-aliasing), cea mai mare diferență de culoare 11.3 pe marginile formelor. O mutare de 40 de unități a stratului
soare dă SSIM 0.993, 6.4 % dintre pixeli, cea mai mare diferență de culoare 19.6 într-o singură regiune.

### Goldens: `--update` față de `bless`

Golden-urile se află în `goldens/` ca fișiere PNG denumite după primele 24 de cifre hexazecimale ale unui SHA-256 peste dimensiune și
pixeli (cadrele identice împart un fișier), plus `goldens/index.json` care mapează verificarea și cadrul la fișier.
Golden-urile SVG sunt fișiere `.svg` denumite în același fel. Un golden este verificat față de numele lui când
este citit, deci un fișier editat manual sau corupt este o eroare (cod de ieșire 2), nu o trecere tăcută.

* `ci run --update` rescrie fiecare golden al verificărilor de render selectate din randarea curentă și
  uită cadrele care nu mai sunt listate. Folosește-l când un manifest s-a schimbat sau după o schimbare
  globală intenționată.
* `ci bless` (sau `ci run --bless`) acceptă doar cadrele care eșuează în prezent sau nu au golden și
  le lasă neatinse pe cele care trec, astfel încât zgomotul de toleranță să nu schimbe fișierele fără rost. Raportul le marchează
  `blessed`.
* `--prune` șterge fișierele golden la care nu se referă nimic. Comite `goldens/` odată cu schimbarea care le-a cauzat.

### Rapoarte

`--json` este rezultatul complet (schema `osatura-ci-report/1`; metrici, regiuni, pragurile folosite).
`--junit` este XML JUnit cu un caz de test pe verificare. `--html` este o singură pagină autonomă (fără resurse
externe): rezumat, tabel de verificări și, pentru fiecare cadru care eșuează, golden-ul, randarea curentă și o
hartă a diferențelor ca PNG data URI inline, un buton de comutare pentru a alterna între golden și curent, și
lista regiunilor diferite. `--artifacts dir` scrie de asemenea `dir/<check>/fNNNNN-{golden,actual,diff}.png`
pentru cadrele care eșuează. Harta diferențelor arată golden-ul estompat, diferențele numărate pe o scară de la galben la
roșu închis după diferența de culoare, anti-aliasing-ul iertat în albastru deschis, zonele ignorate hașurate în gri
și casetele regiunilor în magenta. `ci diff` poate scrie și o bandă alăturată, o suprapunere roșu/cian și un
GIF flicker cu două cadre; folosește `--flicker` pe o pereche golden/actual pentru un GIF înainte/după.

## GitHub Actions (exemplu, nerulat)

Doar documentație: acest workflow a fost scris pentru a corespunde CLI-ului, dar **nu** a fost rulat pe un serviciu CI
real. Adaptează instalarea Rust și căile la depozitul tău.

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

Codurile de ieșire 1 (regresie vizuală sau un buget) și 2 (nu s-a putut evalua) fac ambele jobul să eșueze; raportul
este încărcat în ambele cazuri. Pentru a accepta o schimbare intenționată, rulează `osatura ci bless` local, revizuiește
diff-ul din `goldens/` și comite-l.

## Script shell generic (exemplu, nerulat)

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

## Determinism și alte mașini

Ce a fost verificat (o mașină Linux x86-64, rasterizatorul CPU `tiny-skia` și rasterizatorul f32):

* randarea aceluiași cadru de două ori în același proces și dintr-un motor proaspăt încărcat dă octeți
  identici (verificarea `determinism`; raster, float și text SVG);
* golden-urile scrise de un proces au fost potrivite octet cu octet de procesele ulterioare (`init` apoi `run`,
  `bless` apoi `run`), pentru ambele rendere;
* verificări independente ale metricilor: toate cele 24 de perechi CIEDE2000 ale lui Sharma, Wu și Dalal (2005) folosite în
  testele unitare se reproduc la 5e-5; pe cadrele demo, SSIM (după corectarea regiunii de bordură pe care
  scikit-image o taie), cea mai mare valoare CIEDE2000 și numărul de pixeli diferiți coincid cu
  scikit-image 0.26 (0.99330 față de 0.99330 după corecția de bordură; 19.648 față de 19.648; 11088
  față de 11088 pixeli), iar MSE cu numpy.

Ce **nu** a fost verificat: alte sisteme de operare, procesoare, compilatoare sau versiuni de Rust. Randerele
folosesc virgulă mobilă (trigonometrie, aplatizarea curbelor, acumularea acoperirii), deci o altă libm,
contracție FMA sau cale SIMD poate muta un pixel de margine anti-aliased cu un nivel sau două. Hash-urile exacte
ajută așadar doar pe un singur build al uneltei pe un singur tip de mașină; între mașini bazează-te pe
toleranțe (`pixel_de`, `aa_isolated`, `max_diff_fraction`) și păstrează golden-urile produse de aceeași
imagine CI care le verifică. Randerul GPU nu face parte din acest harness.

## Limite

* Nu există niciun rasterizator SVG în workspace, deci o verificare de render nu poate rasteriza ieșirea SVG și
  compara pixeli; randerul `svg` compară exact textul SVG (orice reordonare a atributelor este un
  eșec).
* Suprapunerile de oase nu sunt randate de nicio verificare CI; audio nu este comparat.
* Îmbinarea regiunilor și regula anti-aliasing sunt euristici (test de margine cu un pas fix de 40, limită
  de dimensiune a grupului); o deplasare sub-pixel a unei margini lungi formează un grup lung și subțire și este raportată, nu iertată.
* Pragurile perceptuale sunt valori implicite, nu sunt calibrate față de judecata animatorilor.
* Verificarea perf măsoară timpul real pe o mașină partajată; păstreaz-o informativă, cu excepția cazului în care
  runner-ul este dedicat.
* `ci init` randează, face fuzzing și exportă o dată pentru calibrare, ceea ce este lent în build-urile debug.
* Nicio integrare cu pull request-urile (fără comentariu de bot, fără încărcare automată de GIF înainte/după): piesele
  (`--html`, `--artifacts`, `ci diff --flicker`) există, legătura cu un serviciu de hosting nu.
