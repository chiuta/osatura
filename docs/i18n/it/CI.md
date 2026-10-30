# CI delle animazioni (`osatura ci`)

Tratta un'animazione come codice testabile: un manifest elenca i controlli, un job di CI li esegue, e una modifica
visiva che fallisce arriva con un report che mostra che cosa si è mosso. Tutto è locale, offline e
deterministico; le metriche sono quelle classiche (CIEDE2000, SSIM, MSE/PSNR, un hash DCT).

```
osatura ci init  <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run   [manifest] [--update] [--bless] [--prune] [--only a,b]
                  [--junit out.xml] [--html out.html] [--json out.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest] [--only a,b] [--prune]
osatura ci diff  a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--pixel-de D] [--aa N]
                  [--ignore x,y,w,h;x,y,w,h] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```

Codici di uscita: **0** ogni controllo è stato superato (o c'è solo info / aggiornato / blessed), **1** almeno un controllo
è fallito (o `ci diff` ha trovato una differenza), **2** qualcosa non ha potuto essere valutato (progetto mancante,
manifest non valido, golden illeggibile o manomesso, opzione sconosciuta).

## Avvio rapido

```
osatura ci init animation.json        # writes osatura-ci.json and goldens/ next to the project
osatura ci run --html report.html     # verify; exit 1 and a report on regressions
osatura ci bless                      # after reviewing the report: accept the new look
git add osatura-ci.json goldens       # commit the goldens with the change that caused them
```

`init` calibra il manifest sul progetto così com'è ora: circa sei fotogrammi equidistanti a una
scala proxy (lato più lungo al massimo 480 px), un controllo con il renderer float su due di essi, determinismo,
validate, un tetto sul tasso di fallimento del pose-fuzz (tasso misurato più 2 punti; il rig demo fallisce nella maggior parte
dei campioni a intervallo pieno, quindi il suo tetto è 100 % e il controllo non protegge nulla finché non lo abbassi),
limiti con il 50 % di margine, un budget di dimensione per la GIF (+50 %), un controllo della giunzione del loop (solo quando il progetto
oggi va in loop in modo pulito), e un budget informativo di velocità di rendering (4 x la mediana misurata, che
dipende dalla macchina e dalle build debug rispetto a quelle release).

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

I percorsi sono relativi al manifest. I campi e i tipi sconosciuti sono errori; tutti i problemi di un manifest
vengono riportati insieme (i nomi devono essere di 1-64 caratteri tra lettere, cifre, `-`, `_`, `.`; i nomi sono
univoci; ogni controllo ha bisogno di un progetto; gli intervalli sono controllati).

| kind | che cosa controlla | fallisce quando |
| --- | --- | --- |
| `render` | fotogrammi (elenco `frames`, oppure `step`, per impostazione predefinita primo/centrale/ultimo), `scale`, `size` (i container di layout vengono risolti di nuovo), `renderer`: `raster` (tiny-skia), `float` (rasterizzatore f32, quantizzato a 8 bit) o `svg` (confronto esatto del testo) | l'immagine differisce dal golden oltre le soglie, non esiste alcun golden, o la dimensione è cambiata |
| `validate` | il progetto compila; risultati del linter alla severità `lint` (`error` predefinito, `warn`, `info`, `off`) | qualsiasi risultato di questo tipo o un errore di compilazione |
| `fuzz` | il pose fuzzer (`samples`, `seed`, `frame`) | tasso di fallimento superiore a `max_fail_rate` (predefinito 0). Il controllo **blocca per impostazione predefinita** (`gate` non impostato significa `true`); viene solo riportato, con stato `info` e senza mai fallire, quando `"gate": false` o `max_fail_rate` è 1 o più (un limite che non può fallire non viene mai mostrato come superato). Il comando `osatura fuzz` è uno strumento separato di sola segnalazione, a meno che non gli si passi `--gate` o `--max-fail-rate` |
| `limits` | `max_layers`, `max_bones`, `max_channels`, `max_frames`, `max_canvas_pixels`, `max_file_bytes` | un conteggio supera il proprio limite |
| `export-size` | byte di `gif`, `apng`, `lottie`, `html` o `svg-frames` (esportatori esistenti) | maggiore di `max_bytes` |
| `determinism` | ogni fotogramma renderizzato due volte dallo stesso motore e da un motore appena caricato; lo SHA-256 dei pixel (o del testo SVG) deve coincidere; `renderers` predefiniti raster, float, svg | qualsiasi digest differisce |
| `loop` | giunzione tra l'ultimo e il primo fotogramma rispetto alla variazione media tra fotogrammi consecutivi (differenza assoluta media dei canali in percentuale); `max_seam_ratio` predefinito 2, `max_seam_mad` facoltativo | giunzione maggiore del consentito; al massimo 600 fotogrammi |
| `perf` | ms mediani per fotogramma, `budget_ms` con `tolerance` relativa (predefinita 0.5) | solo quando `enforce` è true; altrimenti stato `info` |

### Soglie di rendering

Si impostano in `defaults` (tutti i controlli render) e si sovrascrivono per controllo in `thresholds`.

| chiave | predefinito | significato |
| --- | --- | --- |
| `metric` | `de2000` | `de2000` (CIEDE2000) o `de76` (CIE76, euclidea in Lab) |
| `pixel_de` | 1.0 | un pixel conta come diverso oltre questa differenza di colore |
| `max_de` | 8.0 | la massima differenza di colore consentita tra i pixel contati |
| `min_ssim` | 0.99 | il più piccolo SSIM medio consentito |
| `max_diff_fraction` | 0.001 | la massima frazione consentita di pixel contati (0.1 %) |
| `aa_isolated` | 2 | tolleranza dell'anti-aliasing: i gruppi contati di al massimo questo numero di pixel connessi a 8, tutti su un bordo, vengono perdonati (0 = disattivato) |
| `aa_edge_de` | 0 (disattivato) | i pixel di bordo che differiscono al massimo di questo valore vengono perdonati (più permissivo di `pixel_de`, solo sui bordi) |
| `max_phash_distance` | non controllato | la massima distanza di Hamming degli hash DCT a 64 bit |

Un fotogramma i cui pixel sono identici byte per byte al golden passa senza alcuna metrica. Altrimenti il
fotogramma viene confrontato pixel per pixel; ogni pixel viene composto su bianco e su nero e si usa la
differenza maggiore, così una variazione del solo alpha è visibile. Un pixel è un pixel di bordo quando due vicini
opposti (limitati al bordo dell'immagine) in una delle due immagini differiscono per più di 40 in qualche canale; il valore
del pixel stesso viene ignorato, così un pixel isolato in un'area piatta è una differenza reale e il bordo di una forma
non lo è. I rettangoli `ignore` sono `[x, y, w, h]` in unità di scena; vengono resi uguali in entrambe le immagini prima di
qualsiasi metrica, quindi non contano nemmeno in SSIM, MSE o nell'hash.

Come orientamento, sul progetto demo il rasterizzatore CPU e il rasterizzatore float (due percorsi di codice
diversi per la stessa scena) differiscono con SSIM 0.9997, 0.83 % dei pixel oltre la differenza di colore 1 (altri 386
perdonati come anti-aliasing), massima differenza di colore 11.3 sui bordi delle forme. Uno spostamento di 40 unità del livello
sun dà SSIM 0.993, 6.4 % dei pixel, massima differenza di colore 19.6 in una regione.

### Golden: `--update` contro `bless`

I golden stanno in `goldens/` come file PNG chiamati con le prime 24 cifre esadecimali di uno SHA-256 su dimensione e
pixel (i fotogrammi identici condividono un file), più `goldens/index.json` che mappa controllo e fotogramma al file.
I golden SVG sono file `.svg` con lo stesso schema di nomi. Un golden viene verificato rispetto al suo nome quando
viene letto, quindi un file modificato a mano o corrotto è un errore (exit 2), non un superamento silenzioso.

* `ci run --update` riscrive ogni golden dei controlli render selezionati a partire dal rendering corrente e
  dimentica i fotogrammi che non sono più elencati. Usalo quando un manifest è cambiato o dopo una modifica
  globale voluta.
* `ci bless` (o `ci run --bless`) accetta solo i fotogrammi che attualmente falliscono o non hanno un golden e
  lascia intatti quelli che passano, così il rumore di tolleranza non fa cambiare di continuo i file. Il report li marca
  `blessed`.
* `--prune` elimina i file golden a cui nulla fa riferimento. Fai il commit di `goldens/` insieme alla modifica che lo ha causato.

### Report

`--json` è il risultato completo (schema `osatura-ci-report/1`; metriche, regioni, soglie usate).
`--junit` è un XML JUnit con un test case per controllo. `--html` è una pagina autonoma (nessuna risorsa
esterna): riepilogo, tabella dei controlli, e per ogni fotogramma fallito il golden, il rendering corrente e una
mappa delle differenze come data URI PNG incorporati, un pulsante di scambio per alternare tra golden e corrente, e
l'elenco delle regioni diverse. `--artifacts dir` scrive inoltre `dir/<check>/fNNNNN-{golden,actual,diff}.png`
per i fotogrammi falliti. La mappa delle differenze mostra il golden sbiadito, le differenze contate in una rampa dal giallo al
rosso scuro in base alla differenza di colore, l'anti-aliasing perdonato in azzurro chiaro, le aree ignorate tratteggiate in grigio
e i riquadri delle regioni in magenta. `ci diff` può anche scrivere una striscia affiancata, una sovrapposizione rosso/ciano e una
GIF di flicker a due fotogrammi; usa `--flicker` su una coppia golden/actual per una GIF prima/dopo.

## GitHub Actions (esempio, non eseguito)

Solo documentazione: questo workflow è stato scritto per corrispondere alla CLI ma **non** è stato eseguito su un vero servizio
di CI. Adatta l'installazione di Rust e i percorsi al tuo repository.

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

Il codice di uscita 1 (regressione visiva o un budget) e 2 (impossibile valutare) fanno entrambi fallire il job; il report
viene caricato in ogni caso. Per accettare una modifica voluta, esegui `osatura ci bless` in locale, controlla il
diff di `goldens/` e fanne il commit.

## Script shell generico (esempio, non eseguito)

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

## Determinismo e altre macchine

Che cosa è stato verificato (una macchina Linux x86-64, il rasterizzatore CPU `tiny-skia` e il rasterizzatore f32):

* renderizzare lo stesso fotogramma due volte nello stesso processo, e da un motore appena caricato, dà byte
  identici (il controllo `determinism`; raster, float e testo SVG);
* i golden scritti da un processo sono stati riscontrati byte per byte da processi successivi (`init` poi `run`,
  `bless` poi `run`), per entrambi i renderer;
* controlli indipendenti delle metriche: tutte le 24 coppie CIEDE2000 di Sharma, Wu e Dalal (2005) usate nei
  unit test sono riprodotte a 5e-5; sui fotogrammi demo SSIM (una volta corretto per la regione di bordo che
  scikit-image ritaglia), il massimo valore CIEDE2000 e il numero di pixel diversi concordano
  con scikit-image 0.26 (0.99330 contro 0.99330 dopo la correzione del bordo; 19.648 contro 19.648; 11088
  contro 11088 pixel), e MSE con numpy.

Che cosa **non** è stato verificato: altri sistemi operativi, CPU, compilatori o versioni di Rust. I renderer
usano la virgola mobile (trigonometria, appiattimento delle curve, accumulo della copertura), quindi una diversa libm,
contrazione FMA o percorso SIMD può spostare di un livello o due un pixel di bordo con anti-aliasing. Gli hash esatti
servono quindi solo su una build dello strumento su un tipo di macchina; tra macchine diverse affidati alle
tolleranze (`pixel_de`, `aa_isolated`, `max_diff_fraction`) e conserva i golden prodotti dalla
stessa immagine di CI che li verifica. Il renderer GPU non fa parte di questo harness.

## Limiti

* Nel workspace non c'è alcun rasterizzatore SVG, quindi un controllo render non può rasterizzare l'output SVG e
  confrontarne i pixel; il renderer `svg` confronta esattamente il testo SVG (qualsiasi riordino degli attributi è un
  fallimento).
* Le sovrapposizioni degli ossi non sono renderizzate da nessun controllo CI; l'audio non viene confrontato.
* L'unione delle regioni e la regola dell'anti-aliasing sono euristiche (test dei bordi con passo fisso di 40, limite
  alla dimensione dei gruppi); uno spostamento sub-pixel di un bordo lungo forma un gruppo lungo e sottile e viene segnalato, non perdonato.
* Le soglie percettive sono valori predefiniti, non calibrati sul giudizio degli animatori.
* Il controllo perf misura il tempo reale su una macchina condivisa; mantienilo informativo a meno che il
  runner non sia dedicato.
* `ci init` renderizza, fa fuzzing ed esporta una volta per calibrare, il che è lento nelle build debug.
* Nessuna integrazione con le pull request (nessun commento di bot, nessun caricamento automatico di GIF prima/dopo): i pezzi
  (`--html`, `--artifacts`, `ci diff --flicker`) ci sono, il collegamento a un servizio di hosting no.
