# Animáció-CI (`osatura ci`)

Kezeld az animációt tesztelhető kódként: egy manifest ellenőrzéseket sorol fel, egy CI-feladat lefuttatja őket, a sikertelen
vizuális változáshoz pedig jelentés tartozik, amely megmutatja, mi mozdult el. Minden helyi, offline és
determinisztikus; a metrikák tankönyviek (CIEDE2000, SSIM, MSE/PSNR, DCT-hash).

```
osatura ci init  <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run   [manifest] [--update] [--bless] [--prune] [--only a,b]
                  [--junit out.xml] [--html out.html] [--json out.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest] [--only a,b] [--prune]
osatura ci diff  a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--pixel-de D] [--aa N]
                  [--ignore x,y,w,h;x,y,w,h] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```

Kilépési kódok: **0** minden ellenőrzés sikeres (vagy csak info / frissített / elfogadott), **1** legalább egy ellenőrzés
sikertelen (vagy a `ci diff` különbséget talált), **2** valamit nem lehetett kiértékelni (hiányzó projekt,
hibás manifest, olvashatatlan vagy meghamisított golden, ismeretlen opció).

## Gyors kezdés

```
osatura ci init animation.json        # writes osatura-ci.json and goldens/ next to the project
osatura ci run --html report.html     # verify; exit 1 and a report on regressions
osatura ci bless                      # after reviewing the report: accept the new look
git add osatura-ci.json goldens       # commit the goldens with the change that caused them
```

Az `init` a projekt jelenlegi állapotára kalibrálja a manifestet: nagyjából hat egyenletesen elosztott képkocka proxy
skálán (a leghosszabb oldal legfeljebb 480 px), kettőn lebegőpontos renderer ellenőrzés, determinizmus,
validálás, egy pózfuzz-hibaarány-plafon (a mért arány plusz 2 pont; a demó rigje a teljes tartományú minták nagy részén elbukik, ezért a plafonja 100 %, és az ellenőrzés
addig semmit sem őriz, amíg le nem viszed), korlátok 50 %-os ráhagyással, egy GIF-méretkeret (+50 %), egy ismétlési-varrat ellenőrzés (csak ha a projekt
ma tisztán ismétlődik), és egy tájékoztató jellegű renderelési sebességkeret (a mért medián 4-szerese, amely
a géptől és a debug, illetve release build közti különbségtől függ).

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

Az útvonalak a manifesthez képest relatívak. Az ismeretlen mezők és típusok hibák; a manifest összes problémáját
együtt jelenti a rendszer (a nevek 1-64 karakteresek, betűkből, számjegyekből, `-`, `_`, `.` jelekből állnak; a nevek
egyediek; minden ellenőrzéshez kell projekt; a tartományok ellenőrzöttek).

| kind | mit ellenőriz | mikor sikertelen |
| --- | --- | --- |
| `render` | képkockák (`frames` lista, vagy `step`, alapértelmezés az első/középső/utolsó), `scale`, `size` (a layout container-ek újraszámolódnak), `renderer`: `raster` (tiny-skia), `float` (f32 rasztereződő, 8 bitre kvantálva) vagy `svg` (pontos szövegösszehasonlítás) | a kép a küszöbökön túl eltér a golden-től, nincs golden, vagy a méret megváltozott |
| `validate` | a projekt lefordul; lint-találatok a `lint` súlyossággal (`error` alapértelmezett, `warn`, `info`, `off`) | bármely ilyen találat vagy fordítási hiba |
| `fuzz` | a pózfuzzer (`samples`, `seed`, `frame`) | a hibaarány meghaladja a `max_fail_rate` értéket (alapértelmezés 0). Az ellenőrzés **alapértelmezés szerint kapuz** (a `gate` hiánya `true`-t jelent); csak jelentve van, `info` státusszal és sosem bukva, ha `"gate": false` vagy a `max_fail_rate` 1 vagy több (a soha el nem bukható korlátot sosem mutatja sikeresként). Az `osatura fuzz` parancs külön, csak jelentő eszköz, hacsak nem kapja a `--gate` vagy `--max-fail-rate` kapcsolót |
| `limits` | `max_layers`, `max_bones`, `max_channels`, `max_frames`, `max_canvas_pixels`, `max_file_bytes` | egy darabszám meghaladja a korlátját |
| `export-size` | a `gif`, `apng`, `lottie`, `html` vagy `svg-frames` bájtjai (meglévő exportálók) | nagyobb, mint a `max_bytes` |
| `determinism` | minden képkockát ugyanaz a motor kétszer, és egy frissen betöltött motor is renderel; a pixelek (vagy az SVG-szöveg) SHA-256 értékének egyeznie kell; a `renderers` alapértelmezése raster, float, svg | bármely digest eltér |
| `loop` | az utolsó és az első képkocka közötti varrat a képkockák közti átlagos változáshoz képest (átlagos abszolút csatornakülönbség százalékban); `max_seam_ratio` alapértelmezés 2, opcionális `max_seam_mad` | a varrat nagyobb a megengedettnél; legfeljebb 600 képkocka |
| `perf` | a képkockánkénti medián ms, `budget_ms` relatív `tolerance` értékkel (alapértelmezés 0,5) | csak ha az `enforce` igaz; egyébként `info` státusz |

### Render-küszöbök

A `defaults` részben állíthatók (minden render ellenőrzésre), és ellenőrzésenként felülbírálhatók a `thresholds` részben.

| kulcs | alapérték | jelentés |
| --- | --- | --- |
| `metric` | `de2000` | `de2000` (CIEDE2000) vagy `de76` (CIE76, euklideszi Lab-ban) |
| `pixel_de` | 1.0 | egy pixel különbözőnek számít e színkülönbség felett |
| `max_de` | 8.0 | a megszámolt pixelek között megengedett legnagyobb színkülönbség |
| `min_ssim` | 0.99 | a megengedett legkisebb átlagos SSIM |
| `max_diff_fraction` | 0.001 | a megszámolt pixelek megengedett legnagyobb hányada (0,1 %) |
| `aa_isolated` | 2 | élsimítási tolerancia: a megszámolt, legfeljebb ennyi 8-szomszédos pixelből álló csoportok, amelyek mind élen fekszenek, megbocsátottak (0 = ki) |
| `aa_edge_de` | 0 (ki) | az ennél legfeljebb ennyivel eltérő élpixelek megbocsátottak (a `pixel_de` értéknél lazább, csak éleken) |
| `max_phash_distance` | nincs ellenőrizve | a 64 bites DCT-hashek legnagyobb Hamming-távolsága |

Az a képkocka, amelynek pixelei bájtra azonosak a golden-nel, bármilyen metrika nélkül megfelel. Egyébként a
képkockát pixelenként hasonlítja össze a rendszer; minden pixelt fehér és fekete fölé is kompozitál, és a nagyobb
különbséget használja, így az önmagában az alfa változása is látható. Egy pixel élpixel, ha két ellentétes
szomszédja (a határon korlátozva) bármelyik képen több mint 40-nel eltér valamelyik csatornában; a pixel
saját értékét figyelmen kívül hagyja, így egy elszigetelt pixel lapos területen valódi különbség, egy alakzat pereme pedig
nem. Az `ignore` téglalapok `[x, y, w, h]` alakúak színpadegységben; mindkét képen kiegyenlítődnek
bármely metrika előtt, így az SSIM-ben, az MSE-ben és a hashben sem számítanak.

Tájékoztatásul: a demóprojekten a CPU-rasztereződő és a lebegőpontos rasztereződő (két különböző kódút ugyanarra a jelenetre) SSIM 0,9997-tel
tér el, a pixelek 0,83 %-a az 1-es színkülönbség felett (további 386-ot élsimításként megbocsátott), a legnagyobb
színkülönbség 11,3 az alakzatok élein. A nap réteg 40 egységnyi mozgatása SSIM 0,993-at ad, a pixelek 6,4 %-át, a legnagyobb színkülönbség 19,6 egy régióban.

### Golden-ek: `--update` és `bless`

A golden-ek a `goldens/` mappában PNG-fájlként élnek, amelyeket a méretre és a pixelekre számított SHA-256 első 24 hexa számjegye nevez el
(az azonos képkockák egy fájlon osztoznak), plusz a `goldens/index.json`, amely az ellenőrzést és a képkockát fájlhoz rendeli.
Az SVG golden-ek ugyanígy elnevezett `.svg` fájlok. A golden-t olvasáskor a nevével veti össze a rendszer,
így a kézzel szerkesztett vagy sérült fájl hiba (2-es kilépési kód), nem csendes siker.

* A `ci run --update` a kijelölt render ellenőrzések minden golden-jét újraírja a jelenlegi renderből, és
  elfelejti a már nem szereplő képkockákat. Használd, ha a manifest megváltozott, vagy egy szándékos globális változás után.
* A `ci bless` (vagy `ci run --bless`) csak azokat a képkockákat fogadja el, amelyek jelenleg sikertelenek vagy nincs golden-jük, és
  érintetlenül hagyja a megfelelőket, így a tolerancia-zaj nem forgatja a fájlokat. A jelentés `blessed` jelöléssel látja el őket.
* A `--prune` törli azokat a golden-fájlokat, amelyekre semmi sem hivatkozik. A `goldens/` mappát azzal a változtatással együtt commitold, amely előidézte.

### Jelentések

A `--json` a teljes eredmény (séma: `osatura-ci-report/1`; metrikák, régiók, használt küszöbök).
A `--junit` JUnit XML, ellenőrzésenként egy teszteset. A `--html` egyetlen önálló oldal (külső
erőforrások nélkül): összegzés, ellenőrzéstábla, és minden sikertelen képkockához a golden, az aktuális render és egy
különbségtérkép beágyazott PNG data URI-kként, egy csere gomb a golden és az aktuális közti villogtatáshoz, és
az eltérő régiók listája. Az `--artifacts dir` a sikertelen képkockákhoz a `dir/<check>/fNNNNN-{golden,actual,diff}.png`
fájlokat is kiírja. A különbségtérkép halványítva mutatja a golden-t, a megszámolt különbségeket sárgától sötétvörösig tartó skálán
a színkülönbség szerint, a megbocsátott élsimítást világoskékkel, a figyelmen kívül hagyott területeket szürke vonalkázással,
a régiók dobozait pedig magentával. A `ci diff` egymás melletti sávot, piros/ciánkék átfedést és
kétképkockás villogtató GIF-et is tud írni; golden/aktuális párosnál a `--flicker` használatával előtte/utána GIF készül.

## GitHub Actions (példa, nem futtatva)

Csak dokumentáció: ez a workflow a CLI-hez igazodva íródott, de **nem** futott valódi CI-
szolgáltatáson. Igazítsd a Rust-telepítést és az útvonalakat a saját tárolódhoz.

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

Az 1-es (vizuális regresszió vagy keret) és a 2-es (nem lehetett kiértékelni) kilépési kód egyaránt elbuktatja a feladatot; a jelentés
mindkét esetben feltöltődik. Egy szándékos változtatás elfogadásához futtasd helyben az `osatura ci bless` parancsot, nézd át a
`goldens/` különbségét, és commitold.

## Általános shell-szkript (példa, nem futtatva)

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

## Determinizmus és más gépek

Ami ellenőrizve lett (egy Linux x86-64 gép, a `tiny-skia` CPU-rasztereződő és az f32 rasztereződő):

* ugyanannak a képkockának a kétszeri renderelése egy folyamatban, és egy frissen betöltött motorból, azonos
  bájtokat ad (a `determinism` ellenőrzés; raster, float és SVG-szöveg);
* az egyik folyamat által írt golden-eket a későbbi folyamatok bájtról bájtra egyezőnek találták (`init`, majd `run`,
  `bless`, majd `run`), mindkét rendererrel;
* a metrikák független ellenőrzései: Sharma, Wu és Dalal (2005) mind a 24 CIEDE2000-párja, amelyeket az
  egységtesztek használnak, 5e-5 pontossággal reprodukálódik; a demó képkockákon az SSIM (a scikit-image által levágott
  határterületre korrigálva), a legnagyobb CIEDE2000-érték és az eltérő pixelek száma egyezik
  a scikit-image 0.26 értékeivel (0,99330 a 0,99330 ellenében a határkorrekció után; 19,648 a 19,648 ellenében; 11088
  a 11088 pixel ellenében), az MSE pedig a numpy értékeivel.

Ami **nem** lett ellenőrizve: más operációs rendszerek, CPU-k, fordítók vagy Rust-verziók. A rendererek
lebegőpontos számítást használnak (trigonometria, görbelaposítás, lefedettség-felhalmozás), így egy másik libm,
FMA-összevonás vagy SIMD-útvonal egy-két szinttel elmozdíthat egy élsimított élpixelt. A pontos hashek
ezért csak az eszköz egy buildjén, egyfajta gépen segítenek; gépek között a tűrésekre (`pixel_de`, `aa_isolated`, `max_diff_fraction`) támaszkodj, és a golden-eket
ugyanaz a CI-image állítsa elő, amelyik ellenőrzi őket. A GPU-renderer nem része ennek a keretrendszernek.

## Korlátok

* A workspace-ben nincs SVG-rasztereződő, ezért a render ellenőrzés nem tudja rasztereszteni az SVG kimenetet és
  pixeleket összehasonlítani; az `svg` renderer pontosan az SVG-szöveget hasonlítja össze (az attribútumok bármely átrendezése
  hibának számít).
* A csontrétegeket egyetlen CI-ellenőrzés sem rendereli; a hangot nem hasonlítja össze.
* A régiók összevonása és az élsimítási szabály heurisztika (élteszt 40-es rögzített lépéssel, csoportméret-
  korlát); egy hosszú él szubpixeles eltolódása hosszú, vékony csoportot alkot, és jelentve van, nem megbocsátva.
* Az észlelési küszöbök alapértelmezések, nem animátorok ítéletéhez kalibráltak.
* A perf ellenőrzés falióra-időt mér megosztott gépen; tartsd tájékoztató jellegűnek, hacsak a
  futtató nem dedikált.
* Az `ci init` egyszer renderel, fuzzol és exportál a kalibráláshoz, ami debug buildekben lassú.
* Nincs pull request integráció (nincs bot-megjegyzés, nincs automatikus előtte/utána GIF-feltöltés): az építőelemek
  (`--html`, `--artifacts`, `ci diff --flicker`) megvannak, a tárhelyszolgáltatáshoz való ragasztó nincs.
