# Animation-CI (`osatura ci`)

Behandle eine Animation wie testbaren Code: Ein Manifest listet Prüfungen auf, ein CI-Job führt sie aus, und eine fehlschlagende
visuelle Änderung kommt mit einem Bericht, der zeigt, was sich bewegt hat. Alles läuft lokal, offline und
deterministisch; die Metriken sind Standardmetriken (CIEDE2000, SSIM, MSE/PSNR, ein DCT-Hash).

```
osatura ci init  <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run   [manifest] [--update] [--bless] [--prune] [--only a,b]
                  [--junit out.xml] [--html out.html] [--json out.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest] [--only a,b] [--prune]
osatura ci diff  a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--pixel-de D] [--aa N]
                  [--ignore x,y,w,h;x,y,w,h] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```

Exit-Codes: **0** alle Prüfungen bestanden (oder nur info / aktualisiert / geblesst), **1** mindestens eine Prüfung
fehlgeschlagen (oder `ci diff` hat einen Unterschied gefunden), **2** etwas konnte nicht ausgewertet werden (fehlendes Projekt,
fehlerhaftes Manifest, unlesbares oder manipuliertes Golden, unbekannte Option).

## Schnellstart

```
osatura ci init animation.json        # writes osatura-ci.json and goldens/ next to the project
osatura ci run --html report.html     # verify; exit 1 and a report on regressions
osatura ci bless                      # after reviewing the report: accept the new look
git add osatura-ci.json goldens       # commit the goldens with the change that caused them
```

`init` kalibriert das Manifest auf das Projekt, wie es jetzt ist: etwa sechs gleichmäßig verteilte Frames bei einer
Proxy-Skalierung (längste Seite höchstens 480 px), eine Float-Renderer-Prüfung an zweien davon, Determinismus,
Validierung, eine Obergrenze für die Fehlerrate des Pose-Fuzzings (gemessene Rate plus 2 Punkte; das Demo-Rig scheitert bei den meisten
Stichproben über den vollen Bereich, daher ist seine Obergrenze 100 % und die Prüfung schützt nichts, bis du sie senkst),
Limits mit 50 % Spielraum, ein GIF-Größenbudget (+50 %), eine Loop-Naht-Prüfung (nur wenn das Projekt heute
sauber loopt) und ein informatives Renderzeit-Budget (das 4-Fache des gemessenen Medians, das
von der Maschine und davon abhängt, ob Debug- oder Release-Builds verwendet werden).

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

Pfade sind relativ zum Manifest. Unbekannte Felder und Arten sind Fehler; alle Probleme eines Manifests
werden gemeinsam gemeldet (Namen müssen 1-64 Zeichen aus Buchstaben, Ziffern, `-`, `_`, `.` sein; Namen sind
eindeutig; jede Prüfung braucht ein Projekt; Bereiche werden geprüft).

| kind | was geprüft wird | schlägt fehl, wenn |
| --- | --- | --- |
| `render` | Frames (`frames`-Liste oder `step`, Standard erster/mittlerer/letzter), `scale`, `size` (Layout-Container werden neu gelöst), `renderer`: `raster` (tiny-skia), `float` (f32-Rasterisierer, auf 8 Bit quantisiert) oder `svg` (exakter Textvergleich) | das Bild vom Golden über die Schwellenwerte hinaus abweicht, kein Golden existiert oder sich die Größe geändert hat |
| `validate` | das Projekt lässt sich kompilieren; Linter-Befunde mit `lint`-Schweregrad (`error` Standard, `warn`, `info`, `off`) | irgendein solcher Befund oder ein Kompilierfehler |
| `fuzz` | der Pose-Fuzzer (`samples`, `seed`, `frame`) | die Fehlerrate liegt über `max_fail_rate` (Standard 0). Die Prüfung **blockiert standardmäßig** (`gate` ungesetzt bedeutet `true`); sie wird nur berichtet, mit Status `info` und ohne je fehlzuschlagen, wenn `"gate": false` ist oder `max_fail_rate` 1 oder mehr beträgt (eine Schranke, die nicht scheitern kann, wird nie als bestanden angezeigt). Der Befehl `osatura fuzz` ist ein separates Werkzeug, das nur berichtet, sofern nicht `--gate` oder `--max-fail-rate` angegeben ist |
| `limits` | `max_layers`, `max_bones`, `max_channels`, `max_frames`, `max_canvas_pixels`, `max_file_bytes` | eine Anzahl überschreitet ihr Limit |
| `export-size` | Bytes von `gif`, `apng`, `lottie`, `html` oder `svg-frames` (vorhandene Exporter) | größer als `max_bytes` |
| `determinism` | jeder Frame wird von derselben Engine zweimal und von einer frisch geladenen Engine gerendert; SHA-256 der Pixel (oder des SVG-Texts) muss übereinstimmen; `renderers` standardmäßig raster, float, svg | irgendein Digest weicht ab |
| `loop` | Naht zwischen letztem und erstem Frame im Vergleich zur mittleren Änderung von Frame zu Frame (mittlere absolute Kanaldifferenz in Prozent); `max_seam_ratio` Standard 2, optional `max_seam_mad` | Naht größer als erlaubt; höchstens 600 Frames |
| `perf` | Median in ms pro Frame, `budget_ms` mit relativer `tolerance` (Standard 0.5) | nur wenn `enforce` true ist; andernfalls Status `info` |

### Render-Schwellenwerte

Gesetzt in `defaults` (alle Render-Prüfungen) und pro Prüfung in `thresholds` überschrieben.

| Schlüssel | Standard | Bedeutung |
| --- | --- | --- |
| `metric` | `de2000` | `de2000` (CIEDE2000) oder `de76` (CIE76, euklidisch in Lab) |
| `pixel_de` | 1.0 | ein Pixel zählt oberhalb dieser Farbdifferenz als abweichend |
| `max_de` | 8.0 | größte erlaubte Farbdifferenz unter den gezählten Pixeln |
| `min_ssim` | 0.99 | kleinster erlaubter mittlerer SSIM |
| `max_diff_fraction` | 0.001 | größter erlaubter Anteil gezählter Pixel (0.1 %) |
| `aa_isolated` | 2 | Toleranz für Anti-Aliasing: gezählte Gruppen aus höchstens so vielen 8-zusammenhängenden Pixeln, die alle auf einer Kante liegen, werden verziehen (0 = aus) |
| `aa_edge_de` | 0 (aus) | Kantenpixel, die sich um höchstens diesen Wert unterscheiden, werden verziehen (lockerer als `pixel_de`, nur an Kanten) |
| `max_phash_distance` | nicht geprüft | größte Hamming-Distanz der 64-Bit-DCT-Hashes |

Ein Frame, dessen Pixel byteidentisch mit dem Golden sind, besteht ohne jede Metrik. Andernfalls wird der
Frame Pixel für Pixel verglichen; jedes Pixel wird über Weiß und über Schwarz zusammengesetzt, und die größere
Differenz wird verwendet, sodass auch eine reine Alpha-Änderung sichtbar ist. Ein Pixel ist ein Kantenpixel, wenn zwei gegenüberliegende
Nachbarn (am Rand begrenzt) in einem der beiden Bilder sich in irgendeinem Kanal um mehr als 40 unterscheiden; der eigene
Wert des Pixels wird ignoriert, sodass ein vereinzeltes Pixel in einer flachen Fläche ein echter Unterschied ist und der Rand
einer Form nicht. `ignore`-Rechtecke sind `[x, y, w, h]` in Bühneneinheiten; sie werden in beiden Bildern vor
jeder Metrik angeglichen, zählen also auch nicht in SSIM, MSE oder im Hash.

Zur Orientierung: Beim Demoprojekt unterscheiden sich der CPU-Rasterisierer und der Float-Rasterisierer (zwei verschiedene Codepfade
für dieselbe Szene) um SSIM 0.9997, 0.83 % der Pixel liegen über der Farbdifferenz 1 (386 weitere
als Anti-Aliasing verziehen), die größte Farbdifferenz beträgt 11.3 an Formkanten. Eine Verschiebung der Sonnen-
Ebene um 40 Einheiten ergibt SSIM 0.993, 6.4 % der Pixel, größte Farbdifferenz 19.6 in einer Region.

### Goldens: `--update` gegenüber `bless`

Goldens liegen in `goldens/` als PNG-Dateien, benannt nach den ersten 24 Hexziffern eines SHA-256 über Größe und
Pixel (identische Frames teilen sich eine Datei), plus `goldens/index.json`, das Prüfung und Frame auf die Datei abbildet.
SVG-Goldens sind `.svg`-Dateien, die auf dieselbe Weise benannt sind. Ein Golden wird beim Lesen gegen seinen Namen geprüft,
sodass eine von Hand bearbeitete oder beschädigte Datei ein Fehler (Exit 2) ist, kein stilles Bestehen.

* `ci run --update` schreibt jedes Golden der ausgewählten Render-Prüfungen aus dem aktuellen Render neu und
  vergisst Frames, die nicht mehr aufgelistet sind. Verwende es, wenn sich ein Manifest geändert hat oder nach einer gewollten
  globalen Änderung.
* `ci bless` (oder `ci run --bless`) akzeptiert nur Frames, die aktuell fehlschlagen oder kein Golden haben, und
  lässt bestandene unberührt, sodass Toleranzrauschen die Dateien nicht aufwirbelt. Der Bericht markiert sie als
  `blessed`.
* `--prune` löscht Golden-Dateien, auf die nichts verweist. Committe `goldens/` zusammen mit der Änderung, die sie verursacht hat.

### Berichte

`--json` ist das vollständige Ergebnis (Schema `osatura-ci-report/1`; Metriken, Regionen, verwendete Schwellenwerte).
`--junit` ist JUnit-XML mit einem Testfall pro Prüfung. `--html` ist eine eigenständige Seite (keine externen
Ressourcen): Zusammenfassung, Prüfungstabelle und für jeden fehlgeschlagenen Frame das Golden, der aktuelle Render und eine
Differenzkarte als eingebettete PNG-Data-URIs, eine Umschalttaste zum Flackern zwischen Golden und aktuellem Stand sowie
die Liste der abweichenden Regionen. `--artifacts dir` schreibt außerdem `dir/<check>/fNNNNN-{golden,actual,diff}.png`
für fehlgeschlagene Frames. Die Differenzkarte zeigt das Golden verblasst, gezählte Unterschiede in einer Rampe von Gelb bis
Dunkelrot nach Farbdifferenz, verziehenes Anti-Aliasing in Hellblau, ignorierte Bereiche grau schraffiert
und Regionsrahmen in Magenta. `ci diff` kann außerdem einen Nebeneinander-Streifen, ein Rot/Cyan-Overlay und ein
Zwei-Frame-Flicker-GIF schreiben; verwende `--flicker` bei einem Golden/Aktuell-Paar für ein Vorher/Nachher-GIF.

## GitHub Actions (Beispiel, nicht ausgeführt)

Nur Dokumentation: Dieser Workflow wurde passend zur CLI geschrieben, aber **nicht** auf einem echten CI-
Dienst ausgeführt. Passe die Rust-Installation und die Pfade an dein Repository an.

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

Exit-Code 1 (visuelle Regression oder ein Budget) und 2 (konnte nicht ausgewertet werden) lassen beide den Job fehlschlagen; der Bericht
wird in jedem Fall hochgeladen. Um eine gewollte Änderung zu akzeptieren, führe lokal `osatura ci bless` aus, prüfe den
Diff von `goldens/` und committe ihn.

## Generisches Shell-Skript (Beispiel, nicht ausgeführt)

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

## Determinismus und andere Maschinen

Was verifiziert wurde (eine Linux-x86-64-Maschine, der CPU-Rasterisierer `tiny-skia` und der f32-Rasterisierer):

* Das zweimalige Rendern desselben Frames in einem Prozess und aus einer frisch geladenen Engine ergibt identische
  Bytes (die `determinism`-Prüfung; Raster, Float und SVG-Text);
* Von einem Prozess geschriebene Goldens wurden von späteren Prozessen Byte für Byte getroffen (`init` dann `run`,
  `bless` dann `run`), für beide Renderer;
* unabhängige Prüfungen der Metriken: Alle 24 CIEDE2000-Paare von Sharma, Wu und Dalal (2005), die in den
  Unit-Tests verwendet werden, werden auf 5e-5 genau reproduziert; bei den Demo-Frames stimmen SSIM (nach Korrektur um den Randbereich, den
  scikit-image abschneidet), der größte CIEDE2000-Wert und die Anzahl abweichender Pixel mit
  scikit-image 0.26 überein (0.99330 gegenüber 0.99330 nach der Randkorrektur; 19.648 gegenüber 19.648; 11088
  gegenüber 11088 Pixeln), und MSE mit numpy.

Was **nicht** verifiziert wurde: andere Betriebssysteme, CPUs, Compiler oder Rust-Versionen. Die Renderer
verwenden Gleitkomma (Trigonometrie, Kurvenglättung, Abdeckungsakkumulation), sodass eine andere libm,
FMA-Kontraktion oder ein anderer SIMD-Pfad ein Anti-Aliasing-Kantenpixel um ein, zwei Stufen verschieben kann. Exakte Hashes
helfen daher nur bei einem Build des Werkzeugs auf einer Art von Maschine; über Maschinen hinweg verlasse dich auf die
Toleranzen (`pixel_de`, `aa_isolated`, `max_diff_fraction`) und bewahre die Goldens auf, die vom selben
CI-Image erzeugt wurden, das sie auch verifiziert. Der GPU-Renderer ist nicht Teil dieses Harness.

## Grenzen

* Im Workspace gibt es keinen SVG-Rasterisierer, daher kann eine Render-Prüfung die SVG-Ausgabe nicht rastern und
  Pixel vergleichen; der `svg`-Renderer vergleicht den SVG-Text exakt (jede Umordnung von Attributen ist ein
  Fehler).
* Bone-Overlays werden von keiner CI-Prüfung gerendert; Audio wird nicht verglichen.
* Regionszusammenführung und die Anti-Aliasing-Regel sind Heuristiken (Kantentest mit festem Schritt von 40, Begrenzung der
  Gruppengröße); eine Subpixel-Verschiebung einer langen Kante bildet eine lange dünne Gruppe und wird gemeldet, nicht verziehen.
* Die Wahrnehmungsschwellen sind Standardwerte, nicht am Urteil von Animatoren kalibriert.
* Die perf-Prüfung misst die Wanduhrzeit auf einer geteilten Maschine; belasse sie informativ, sofern der
  Runner nicht dediziert ist.
* `ci init` rendert, fuzzt und exportiert zur Kalibrierung einmal, was in Debug-Builds langsam ist.
* Keine Pull-Request-Integration (kein Bot-Kommentar, kein automatischer Upload eines Vorher/Nachher-GIFs): Die Bausteine
  (`--html`, `--artifacts`, `ci diff --flicker`) sind vorhanden, der Kleber zu einem Hosting-Dienst nicht.
