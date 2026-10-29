<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Formati di esportazione e importazione di OSATURA

Che cosa conserva, approssima e scarta ogni formato di file che OSATURA scrive o legge, quale comando lo produce e come
verificare il risultato. Questo testo è rilasciato con licenza CC-BY-4.0. Versione documentata: **osatura 0.1.0**.

Ogni riga di comando qui sotto è stata eseguita sul progetto demo integrato (`osatura demo demo.json`) o su una sua copia di 8 fotogrammi
(`small.json`, lo stesso progetto con `"frames": 8`). Le dimensioni citate si riferiscono a quel progetto di 8 fotogrammi, 640x480, e servono solo
come orientamento: scalano con i fotogrammi, le ancore e i livelli, non con qualcosa di fisso. I dettagli dei comandi (ogni opzione,
ogni errore) sono in [CLI_REFERENCE.md](CLI_REFERENCE.md); il formato di file del progetto stesso è in
[PROJECT_FORMAT.md](PROJECT_FORMAT.md); lo stato di codec e brevetti è in [PATENT_NOTES.md](PATENT_NOTES.md) (non è una consulenza legale).

## Indice

1. [Quale formato per quale lavoro](#1-quale-formato-per-quale-lavoro)
2. [Formati di output in sintesi](#2-formati-di-output-in-sintesi)
3. [Immagini statiche: SVG, PNG, PNG a 16 bit, OpenEXR](#3-immagini-statiche)
4. [Animazione raster: sprite sheet PNG, GIF, APNG, WebM, MP4](#4-animazione-raster)
5. [Player HTML](#5-player-html)
6. [Lottie (.json) e dotLottie (.lottie)](#6-lottie-json-e-dotlottie-lottie)
7. [glTF binario (.glb)](#7-gltf-binario-glb)
8. [Cartella Open Cel Animation (OCA)](#8-cartella-open-cel-animation-oca)
9. [Importazione: SVG, OpenRaster, Lottie, immagini, GIF/APNG](#9-importazione-svg-openraster-lottie-immagini-gifapng)
10. [Importazione: motion capture BVH, MIDI e altri input in tempo reale](#10-importazione-motion-capture-bvh-midi-e-altri-input-in-tempo-reale)
11. [Formati audio](#11-formati-audio)
12. [Matrice di supporto delle funzionalità](#12-matrice-di-supporto-delle-funzionalità)
13. [Come ridurre i file: export-fit, lod, size-report](#13-come-ridurre-i-file)
14. [Requisiti di ffmpeg](#14-requisiti-di-ffmpeg)
15. [Riferimenti a brevetti e licenze](#15-riferimenti-a-brevetti-e-licenze)
16. [Convalidare ciò che hai scritto](#16-convalidare-ciò-che-hai-scritto)
17. [Differenze rispetto ad altri documenti](#17-differenze-rispetto-ad-altri-documenti)

---

## 1. Quale formato per quale lavoro

| Vuoi... | Usa | Perché |
|---|---|---|
| L'aspetto esatto, condiviso come un unico file, riproducibile in qualsiasi browser, scorrevole con lo scrubber | `.html` | Fotogrammi SVG inline disegnati dal renderer di OSATURA; nulla è approssimato. |
| Una piccola animazione vettoriale per una pagina web o un'app con un player Lottie | `.json` o `.lottie` | Vettoriale, indipendente dalla risoluzione, piccola; ma vedi l'elenco di ciò che viene scartato (immagini, tratti a larghezza variabile, audio...). |
| Un'anteprima in loop per una chat o un README | `.gif` | Funziona ovunque; 256 colori per fotogramma, trasparenza a 1 bit, senza suono. |
| Un sostituto della GIF con colore pieno e alpha | `.apng` | RGBA a 8 bit senza perdita; più grande della GIF. |
| Un video con audio | `.mp4` (H.264 + AAC) o `.webm` (VP9/AV1 + Opus) | Richiede `ffmpeg` installato da te. `.webm` mantiene la trasparenza. |
| Sprite sheet per un motore di gioco | `.png` | Un PNG più un atlante JSON con i rettangoli dei tile. |
| Uno scheletro e un movimento per uno strumento 3D | `.glb` | Ossi, animazione, forme piatte come mesh di triangoli. |
| Fotogrammi per strumenti di compositing/pittura | `export-oca` (Open Cel Animation) o `frames --png` | Cel come PNG, in una cartella. |
| Immagini statiche in luce lineare o ad alta profondità di bit | `render-hdr` (`.exr` o `.png` a 16 bit) | Renderer in virgola mobile, OpenEXR a 16 bit o half/float. |
| Un singolo fotogramma statico | `render` (`.svg` o `.png`) | Vettoriale o raster a 8 bit. |

## 2. Formati di output in sintesi

| Formato | Comando | Estensione | Richiede ffmpeg | Alpha | Audio | Dimensione per la demo di 8 fotogrammi* |
|---|---|---|---|---|---|---|
| SVG (un fotogramma) | `render` | `.svg` | no | sì | no | pochi KB |
| Sequenza SVG/PNG | `frames` | cartella | no | sì | no | un file per fotogramma |
| PNG (un fotogramma) | `render` | `.png` | no | sì (8 bit) | no | |
| PNG a 16 bit | `render-hdr` | `.png` | no | sì | no | |
| OpenEXR (half/float) | `render-hdr` | `.exr` | no | sì | no | 466 KB half, 923 KB float (640x480) |
| Sprite sheet PNG + JSON | `export` | `.png` (+ `.atlas.json`) | no | sì | no | |
| GIF | `export` | `.gif` | no | 1 bit | no | 83 KB |
| APNG | `export` | `.apng` | no | sì | no | 131 KB |
| WebM | `export` | `.webm` | sì | sì | sì (Opus) | 10.5 KB |
| MP4 | `export` | `.mp4` | sì | no | sì (AAC) | 8.9 KB |
| Player HTML | `export` | `.html`, `.htm` | solo per l'audio | sì | mp3 incorporato | 66 KB |
| Lottie | `export` | `.json` | no | sì | no | 47 KB |
| dotLottie | `export` | `.lottie` | no | sì | no | 11.5 KB |
| glTF binario | `export` | `.glb` | no | sì | no | 115 KB |
| Open Cel Animation | `export-oca` | cartella | no | sì | no | un PNG per cel |

\* GIF, APNG, WebM e MP4 con `--scale 0.5` (320x240); formati vettoriali a dimensione piena. "Richiede ffmpeg" significa un
`ffmpeg` funzionante nel `PATH` (sezione 14). I nomi di output sono scelti in base all'estensione, senza distinzione tra maiuscole e minuscole; un'estensione sconosciuta viene rifiutata e
non viene scritto nulla (`unknown export format '.xyz' (use .gif .html .json .mp4 .apng .webm .png .glb)`, e anche `.lottie` è
accettata).

Tutti i writer rifiutano di sovrascrivere il progetto di input con l'output, scrivono tramite un file temporaneo che poi rinominano, e rifiutano
i link simbolici a meno che non venga indicato `--follow-symlinks` (CLI_REFERENCE sezione 1.4). La protezione sulle dimensioni per gli esportatori vettoriali
(Lottie, HTML, GLB) rifiuta un'esportazione di oltre circa 40 milioni di valori (fotogrammi x ancore, più ossi e livelli per fotogramma)
con `export would write about N million values ...`.

## 3. Immagini statiche

### SVG (`render ... out.svg`, `frames`)

```
$ osatura render demo.json 12 f12.svg --bones
wrote f12.svg
$ osatura frames demo.json fr            # fr/frame_0000.svg ... frame_0047.svg
```
L'output vettoriale del renderer di OSATURA: percorsi, gradienti, tratti con profili di larghezza, tratteggi, modalità di fusione, maschere ed effetti
dei livelli vengono scritti come SVG. `--bones` disegna la sovrapposizione dello scheletro. Per `.svg`, `--scale` lascia invariato il viewBox del documento. **Scarti/limiti noti:** il renderer non ne segnala nessuno; il modo in cui un dato visualizzatore SVG tratta le modalità di fusione e i filtri dipende
da quel visualizzatore. `--proxy N` scrive un proxy di scrubbing semplificato che scarta tratti sottili, gradienti e modalità di fusione (non per
le consegne finali). **Convalida:** apri il file in un browser, oppure `python3 -c "import xml.dom.minidom,sys; xml.dom.minidom.parse(sys.argv[1])" f12.svg`.

### PNG (`render ... out.png`, `frames --png`)

RGBA a 8 bit, sRGB, renderizzato dal rasterizzatore interno del motore. `--scale N` in (0, 16] moltiplica la dimensione in pixel. Un progetto con
`"background": null` viene renderizzato trasparente dove non è disegnato nulla. Per impostazione predefinita senza gestione del colore: i valori si fondono nello spazio codificato sRGB.
Dalla wave 7 un progetto può scegliere la composizione in luce lineare (`osatura color-space p.json out.json linear`, campo `color_space`):
gli output raster, GIF, APNG, video e `render-hdr` vengono allora composti in luce lineare f32 e convertiti in sRGB una sola volta; SVG e
Lottie non cambiano (LIMITATIONS L-22).

### PNG a 16 bit e OpenEXR (`render-hdr`)

```
$ osatura render-hdr small.json 3 f.exr                           # HALF, 640x480, 465,611 bytes
$ osatura render-hdr small.json 3 f2.exr linear --precision float # FLOAT, 923,211 bytes
$ osatura render-hdr small.json 3 f4.png                          # 16-bit/colour RGBA, sRGB-encoded
```
* **OpenEXR:** a parte singola, per scanline, **non compresso**, canali `A B G R`, luce lineare, alpha premoltiplicato. `HALF` per impostazione predefinita,
  `FLOAT` con `--precision float` (o `--float32`). La parola `linear` non ha effetto per EXR (sempre lineare).
* **PNG a 16 bit:** codificato sRGB a meno che non sia indicata la parola `linear`.
* **Fedeltà:** il renderer in virgola mobile ha più precisione di quello a 8 bit, ma la grafica è un normale colore sRGB, quindi il file
  contiene valori nell'intervallo di visualizzazione con maggiore precisione, non luce HDR riferita alla scena. Lo sfondo del progetto non viene disegnato.
* **Scarti/limiti noti:** un fotogramma per file; nessuna compressione, livelli o AOV; nessun profilo ICC; `--8bit` ripiega sul rasterizzatore a 8 bit
  come sorgente.
* **Convalida:** `ffprobe f.exr` riporta `exr 640x480 gbrapf32le`; qualsiasi visualizzatore EXR lo apre.

## 4. Animazione raster

Tutti questi formati renderizzano prima ogni fotogramma con il motore raster. Un progetto con `"background": null` viene renderizzato con pixel trasparenti
ovunque non sia disegnato nulla; la demo ha un colore di sfondo (`#f6f1e7`) ed è quindi opaca. `--scale N` in (0, 16]
si applica a `.gif .apng .webm .mp4 .png`; sugli altri formati viene rifiutato (`unknown option --scale for 'export .json'`).

### Sprite sheet PNG (`export ... out.png`)

```
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.atlas.json
wrote out3.png                    # 4 columns x 2 rows of 160x120 tiles
```
Un PNG con tutti i fotogrammi come tile (`--columns N`, per impostazione predefinita una griglia quasi quadrata: ceil(sqrt(frames)) colonne) e un atlante
`out3.atlas.json` (il nome di output con l'estensione sostituita da `.atlas.json`) che elenca `{index, x, y, w, h}` per ogni fotogramma. L'alpha viene mantenuto.
L'atlante si chiamava `out3.json` e poteva sovrascrivere in silenzio un file Lottie con lo stesso nome base; il nuovo nome lo evita, e il percorso dell'atlante passa per le stesse protezioni su input/symlink.

### GIF (`export ... out.gif`)

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif                      # 8 frames, 83 KB
```
* Un'immagine per fotogramma sorgente, **in loop infinito**, una palette locale fino a 256 colori per fotogramma (quantizzata, senza dithering).
* **Temporizzazione:** i ritardi GIF sono centesimi di secondo interi, e i visualizzatori trattano qualsiasi valore inferiore a 2 cs come 10 cs, quindi 50 fps è il vero massimo.
  Fino a 50 fps i ritardi sono distribuiti con diffusione dell'errore (29.97 fps alterna 3 e 4 cs) così la durata totale resta entro 1 cs da
  `frames / fps`. Oltre i 50 fps la sequenza viene **decimata a 50 fps**, la durata viene mantenuta e viene stampato un avviso
  (`GIF cannot hold 60 fps ... use APNG, WebM or MP4 for the full rate`).
* **Trasparenza:** 1 bit (il flag del colore trasparente è impostato); i pixel semitrasparenti non vengono preservati.
* **Scarti noti:** nessun audio (`note: GIF cannot carry audio; ...` quando il progetto ha dei suoni); nessun alpha parziale; le immagini più larghe
  o più alte di 65535 pixel vengono rifiutate (`image too large for GIF`).
* **`--report r.json`:** scrive l'elenco di ciò che il formato non ha potuto trasportare (qui: la decimazione del frame rate, se presente).
* **Suggerimenti sulle dimensioni:** un `--scale` più piccolo, meno fotogrammi, oppure `export-fit --format gif` (sezione 13). La GIF è il meno efficiente tra i formati
  raster per grafica fotografica o ricca di gradienti.
* **Convalida:** `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` stampa il
  numero di fotogrammi (8).

### APNG (`export ... out.apng`)

PNG animato, RGBA a 8 bit, senza perdita, un fotogramma per fotogramma del progetto al fps del progetto (0.0417 s per fotogramma a 24 fps). L'alpha pieno viene mantenuto. Nessun audio. Più grande
della GIF per grafica piatta (131 KB contro 83 KB nell'esempio) perché non c'è palette. La maggior parte dei browser riproduce APNG; alcuni strumenti no,
nel qual caso usa WebM o GIF. **Convalida:** `ffprobe` con `-count_frames` (8 fotogrammi per l'esempio).

### WebM (`export ... out.webm`) e MP4 (`export ... out.mp4`)

```
$ osatura export small.json out.mp4  --scale 0.5     # h264, yuv420p, 320x240, 8 frames
$ osatura export small.json out.webm --scale 0.5     # vp9, yuv420p + alpha_mode=1
$ osatura export small.json out.webm --codec av1     # needs an ffmpeg with libaom-av1
```
* **MP4:** H.264 (`libx264`), `yuv420p`, **senza alpha**. Audio AAC quando il progetto ha tracce sonore.
* **WebM:** VP9 (`libvpx-vp9`, predefinito) o AV1 (`libaom-av1`, `--codec av1`); il canale alpha viene mantenuto (`yuva420p`, tag del container
  `alpha_mode=1`); audio Opus (`libopus`) quando il progetto ha dei suoni. Qualsiasi altro valore di `--codec`: `unknown --codec 'foo' (vp9 or av1)`.
* **Suono:** le tracce sonore del progetto vengono mixate (start, gain, trim, dissolvenze; tagliate alla lunghezza del progetto) e multiplexate.
* **Richiede ffmpeg:** i fotogrammi vengono renderizzati in PNG in una directory temporanea privata, passati a `ffmpeg` e poi rimossi. Se
  `ffmpeg` manca: `cannot run ffmpeg (...); install it or export a GIF/HTML instead` (exit 1, nulla viene scritto). Se manca
  l'encoder: `ffmpeg failed`.
* **Limiti noti:** solo 8 bit; la dimensione del video segue quella del progetto e `--scale` (controlla il risultato con `ffprobe`).
  `dof-export` scrive anche `.mp4`/`.webm`, senza audio.
* **Convalida:** `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4`
  ha stampato `h264,320,240,yuv420p,24/1,8`.
* **Brevetti:** vedi la sezione 15; H.264 e AAC sono coperti da pool di licenze, VP9/AV1 non sono stati valutati.

## 5. Player HTML

```
$ osatura export small.json out.html
wrote out.html                     # 66 KB for 8 frames
```
Un file HTML autonomo: **ogni fotogramma come SVG inline**, un pulsante play/pausa, uno scrubber (input range) e un'etichetta del fotogramma,
pilotati da un unico `<script>` inline. Non fa **nessuna richiesta di rete**: l'unico URL nel file è il namespace SVG. Per aprirlo non serve
altro che un browser. L'immagine è esattamente quella prodotta dal renderer SVG, quindi è il più fedele tra i formati
"riproducibili" (maschere, modalità di fusione, effetti, ossi, IK, deformatori, particelle sono tutti incorporati nei fotogrammi).

* **Audio:** se il progetto ha dei suoni, l'audio mixato è incorporato come data URI mp3 (richiede `ffmpeg`; avviso sopra i 3 MiB, l'audio
  viene scartato con un avviso sopra i 16 MiB).
* **Dimensione:** cresce linearmente con i fotogrammi (circa 7.7 KB per fotogramma qui) e con il dettaglio vettoriale; il file non è compresso. Suggerimenti
  tipici: servilo compresso con gzip (l'esempio da 66 KB diventa circa 9 KB con gzip), riduci il numero di fotogrammi o il dettaglio dei percorsi con `lod`, e usa
  `export-fit --format html`.
* **Limiti:** `--report` viene rifiutato (`unknown option --report for 'export .html'`);
  si applica la protezione sulle dimensioni dell'esportazione vettoriale; il player offre solo play/pausa e uno scrubber.
* **Convalida:** aprilo; `grep -c '<svg' out.html` conta i fotogrammi; `grep -o 'http[^"<]*' out.html` dovrebbe elencare solo il namespace SVG.

## 6. Lottie (.json) e dotLottie (.lottie)

```
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ osatura export small.json out.lottie       # a zip: manifest.json + a/animation.json
```

### Come è costruito

Il file è Bodymovin (`"v": "5.7.0"`) con `fr` = fps del progetto, `ip` = 0, `op` = numero di fotogrammi e `w`/`h` = dimensione della scena. **Ogni
livello vettoriale diventa un livello shape la cui geometria dei percorsi viene baked per ogni fotogramma nello spazio mondo (keyframe hold)**, quindi ossi, IK,
skinning, deformatori, molle, particelle, la camera (inclusa la parallasse) e le trasformazioni dei genitori sono tutti *dentro* la geometria. Il risultato
si riproduce come l'originale ma non è un rig modificabile: non ci sono ossi, canali, espressioni. Viene scritto un livello di sfondo pieno
quando il progetto ha un colore di sfondo. Di conseguenza il file cresce con `frames x anchors`; è molto più grande di un
Lottie costruito a mano con animazione delle trasformazioni (47 KB per la demo di 8 fotogrammi, 11.5 KB dentro un `.lottie`).

### Esportato

Riempimenti e tratti pieni e a gradiente (stop di gradiente con alpha; i punti finali del gradiente seguono la trasformazione animata del livello), pattern
di tratteggio, terminazioni, giunzioni e limite di miter, larghezze del tratto scalate con la scala del livello, modalità di fusione, opacità del livello, track matte
alpha e alpha invertito per le maschere dei livelli (una copia della sorgente della maschera agisce da matte), effetti dei livelli (blur, ombra, tinta, riempimento) sui
livelli a foglia singola, e **l'opacità dei gruppi come pre-composizione** così i figli sovrapposti non si vedono in trasparenza l'uno attraverso l'altro. I numeri vengono
scritti nella loro forma decimale più breve.

### Segnalato con avvisi (mai scartato in silenzio)

Ogni voce qui sotto viene stampata su stderr come `warning: [code] layer: message` e scritta in `--report FILE` come
`{"format":"lottie","count":N,"warnings":[{"code","layer","message"}]}`:

| Codice | Significato |
|---|---|
| `image-layer` | I livelli immagine non vengono esportati (l'output Lottie contiene solo livelli vettoriali). |
| `variable-width-stroke` | I profili di larghezza non sono supportati; viene usata la larghezza base costante. |
| `stroke-anisotropic` | Scala non uniforme o skew sul livello; i tratti Lottie sono uniformi, quindi la larghezza viene scalata con la media geometrica. |
| `gradient-anisotropic` | Un gradiente su un livello di questo tipo è approssimato (i gradienti Lottie non vengono deformati). |
| `blend-add` | La modalità di fusione `add` non ha equivalente in Lottie; viene usata `screen`. |
| `group-blend` | Una modalità di fusione su un gruppo non viene esportata. |
| `group-mask` | Una maschera su un gruppo non viene esportata (quelle sui singoli livelli sì). |
| `group-effect` | Gli effetti dei livelli su un gruppo (di più livelli) non vengono esportati. |
| `mask-source-not-vector` | La sorgente della maschera non è un livello vettoriale, quindi la maschera non viene esportata. |
| `depth-of-field` | I canali `camera_focus`/`camera_blur` non vengono applicati; usa `dof-export`. |
| `audio` | Lottie non può trasportare audio; N tracce sonore non sono incluse. |

### Scartato senza equivalente in Lottie

Ossi, IK, canali, macchine a stati, data binding, container, regioni di scorrimento, marker/eventi, suoni, catene di post-effetti (`fx`),
normal map e la struttura modificabile del rig. I colori degli effetti per livello non possono essere animati (viene usato il primo valore) e
le dimensioni sono in unità di scena (LIMITATIONS L-23).

### dotLottie (`.lottie`)

Uno zip [dotLottie 2.0](https://dotlottie.io/spec/2.0/) con `manifest.json` (`{"animations":[{"id":"animation"}],"generator":"OSATURA","version":"2"}`) e
`a/animation.json` (lo stesso Lottie di sopra, compresso con deflate: 46,843 byte sono diventati un file di 11,515 byte). `--report` funziona allo stesso
modo. **Importare** un `.lottie` non è supportato (`import-lottie` richiede il JSON; decomprimi prima `a/animation.json`). Fino a v0.1.0-alpha.4 il contenitore era dotLottie 1.0 (`animations/animation.json`, `autoplay`/`loop` nel manifest); il layout 2.0 funziona nei player dotLottie attuali (verificato con @lottiefiles/dotlottie-web).

### Convalida e avvertenze

* `python3 -m json.tool out.json > /dev/null` per la correttezza formale; `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],len(d['layers']))"`.
* `unzip -l out.lottie` deve mostrare esattamente le due voci.
* Round trip: `osatura import-lottie out.json back.json` seguito da `osatura check back.json` verifica che OSATURA riesca a leggere il proprio
  output (il round trip comporta perdite: il rig non c'è più).
* **Non era disponibile alcun player Lottie indipendente** quando questo testo è stato scritto (LIMITATIONS L-28): l'output è stato controllato rispetto alla struttura dei campi di Lottie
  e all'importatore di OSATURA stesso. Prova nel player di destinazione (lottie-web, dotLottie player, Skottie, ...) e leggi gli
  avvisi; se la precisione è importante, usa `.html` o il video.
* **Suggerimenti sulle dimensioni:** `export-fit --format lottie` cerca quattro parametri (`decimals`, `path_eps`, `key_tol`, `frame_step`) rispetto a un budget di byte;
  `lod` riduce le ancore prima dell'esportazione; `size-report` mostra la dimensione del Lottie baked per livello (sezione 13).
* **Brevetti:** Lottie e dotLottie sono formati aperti; nulla in PATENT_NOTES.md li riguarda.

## 7. glTF binario (.glb)

```
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
```
glTF 2.0 binario (magic `glTF`, versione 2, `"generator": "osatura"`), solo funzionalità di base. Il progetto deve avere degli ossi
(altrimenti `the project has no bones to export`).

* **Esatto:** la gerarchia degli ossi e la loro animazione (campionata a ogni fotogramma, interpolazione lineare, shear scartato).
* I **riempimenti** diventano mesh piatte di triangoli (ear clipping). Le forme sono rigide (un livello con un osso segue quel nodo osso) oppure **con skin**
  (JOINTS_0/WEIGHTS_0, 4 influenze per vertice) quando ogni ancora porta pesi degli ossi.
* I **tratti** sono triangolati in mesh proprie (larghezza, profilo di larghezza, terminazioni butt/round/square, giunzioni miter/round/bevel, tratteggi);
  i tratti traslucidi possono apparire più scuri dove i triangoli si sovrappongono.
* I **gradienti** (lineari e radiali) diventano un piccolo PNG generato usato come texture del colore base; diffusione pad (clamp).
* Le **maschere** sono baked ritagliando i triangoli contro il riempimento del livello maschera, solo per una sorgente di maschera vettoriale con riempimenti opachi e senza
  tratti, e solo per la forma della maschera al fotogramma 0. I **livelli immagine** diventano quad con texture o mesh di deformazione con PNG incorporato.
* I **path morph** diventano morph target con pesi animati quando il controllo per fotogramma riproduce la geometria del motore; altrimenti
  il percorso resta una forma statica del fotogramma 0 e il motivo viene segnalato.
* **Non esportato (segnalato con avvisi `[glb-limit]`):** modalità di fusione diverse da normal (disegnate come normal), opacità animata del livello (congelata al
  fotogramma 0), animazione della trasformazione di livelli non legati a ossi, effetti dei livelli, deformatori oltre la loro forma al fotogramma 0, percorsi animati senza un morph
  valido, forme di maschera animate, suono.
* **Coordinate:** y è negata (glTF ha y verso l'alto); le forme sono stratificate con un piccolo offset z (0.01 per forma) per l'ordine di disegno.
* **Convalida:** l'intestazione è `glTF`, versione 2, e il campo lunghezza è uguale alla dimensione del file (one-liner `python3` nella sezione 16). Usa un validatore
  o un visualizzatore glTF a tua scelta per il controllo completo; nessuno è incluso e nessuno è stato eseguito per questo documento. L'esempio sopra ha 22 nodi,
  15 mesh, 1 skin, 1 animazione e 7 immagini incorporate.
* **Suggerimenti sulle dimensioni:** meno ancore vettoriali (`lod`), niente gradienti (ognuno è una texture), meno fotogrammi (l'animazione è campionata per fotogramma).
* **Brevetti:** formato aperto; nessun riferimento in PATENT_NOTES.md.

## 8. Cartella Open Cel Animation (OCA)

```
$ osatura export-oca small.json oca1                 # one layer "render" with all composited frames
$ osatura export-oca small.json oca2 --layers        # one OCA layer per top-level layer
wrote oca2/small.oca
```
Scrive `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) più cel PNG `layerN/fNNNN.png`. Senza `--layers` c'è un solo livello
`render` con i fotogrammi composti; con `--layers` ogni livello di primo livello viene renderizzato da solo su trasparenza (11 livelli per la demo).
`--scale N` moltiplica la dimensione delle cel. **Scarti noti:** le cel sono raster (nessun vettore, osso o canale sopravvive); l'opacità e la fusione dei livelli
non vengono scritte (ogni livello è un `paintlayer`, opacità 1); i suoni non vengono esportati; i file all'interno di una cartella esistente vengono sovrascritti.
**Importazione:** non esiste alcun comando `import-oca` nella CLI. **Convalida:** il file `.oca` è JSON (`python3 -m json.tool`); ogni cel elencata
deve esistere come PNG.

## 9. Importazione: SVG, OpenRaster, Lottie, immagini, GIF/APNG

Le importazioni creano un **nuovo** progetto (non si fondono in uno esistente) salvo indicazione contraria; usa `script` o modifica il JSON per combinarli.

### SVG (`import-svg`)

```
$ osatura import-svg art.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
wrote svg.json (3 paths)
```
Sottoinsieme: `path` (tutti i comandi), `rect` (angoli arrotondati ignorati), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` con `transform`,
`fill` pieno, `stroke`, `stroke-width`. Risultato: 24 fps, 24 fotogrammi, dimensione da `viewBox` o `width`/`height` (altrimenti 512x512), **un solo livello
vettoriale `svg`**. Saltati con una riga `warning:` (stampata su **stdout**): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`,
`<linearGradient>`/`<radialGradient>` (un paint a gradiente ripiega sul grigio), `<image>`, `<use>`, `<foreignObject>`. Limiti: 8 MB di SVG,
200,000 ancore. Per fotografie e raster usa `trace` / `autotrace` (CLI_REFERENCE sezione 9).

### OpenRaster (`import-ora`)

```
$ osatura import-ora art.ora ora.json --fps 12 --frames 24
```
Un **livello immagine per ogni livello raster** (PNG incorporato), ordine dal basso verso l'alto, livelli nascosti mantenuti ma invisibili, gruppi appiattiti, opacità,
visibilità e posizione per livello, e le comuni operazioni di composizione `svg:` (multiply, screen, overlay, darken, lighten, color-dodge,
color-burn, hard-light, soft-light, difference, exclusion, plus mappato su add). L'opacità di una pila annidata si moltiplica nei suoi livelli e la sua
modalità di fusione viene ereditata, ma i livelli sovrapposti all'interno di una pila traslucida **non** vengono prima appiattiti. I nomi duplicati ricevono `_1`, `_2`.
Limiti: 2000 voci di archivio, 128 MiB per voce, livelli da 64 Mpixel. Un file che non è uno zip: `not a zip/ORA file: ...` (exit 1). Non esiste esportazione
ORA. L'importazione PSD non è implementata (LIMITATIONS L-32).

### Lottie (`import-lottie`)

```
$ osatura import-lottie in.json out.json [fx.json]
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
```
Un **sottoinsieme** deliberato; ogni funzionalità saltata è una riga `warning:`.
* **Importato:** livelli shape con percorsi `sh`, `rc`, `el`; riempimenti pieni e a gradiente (i punti finali/centro/raggio animati del gradiente diventano
  canali `grad_dx`/`grad_dy`); tratti; gruppi; trasformazioni dei livelli (posizione, scala, rotazione, opacità statiche o con keyframe; i percorsi
  di posizione curvi sono baked per fotogramma); modalità di fusione; un livello di sfondo pieno; livelli immagine con data URI PNG/JPEG incorporati; livelli null come genitori
  (le loro trasformazioni sono baked nei figli); maschere dei livelli (add/subtract/intersect con opacità, inversione, forme con chiavi); luma matte
  (approssimati); precomposizioni; intervalli in/out dei livelli (come traccia a gradini dell'opacità); easing dei keyframe (hold diventa step, le maniglie bezier diventano
  cubic-bezier, nessuno diventa lineare).
* **Percorsi animati:** le chiavi di forma con la stessa topologia (stesso numero di ancore e aperto/chiuso) diventano un percorso base più un **morph target**
  per ogni ulteriore chiave distinta (`k1`, `k2`, ...) con canali di peso `path_morph` che portano l'easing. Una topologia diversa usa la prima forma
  e stampa un avviso.
* **Effetti (`ef`):** sfocatura gaussiana, ombra, tinta e riempimento diventano **effetti per livello** con numeri animati come canali `fx_*`;
  tinta/riempimento/tritone/livelli statici sui livelli shape sono baked nei colori. I colori degli effetti non possono essere animati (viene usato il primo valore); gli altri
  effetti producono avvisi.
* **Saltato con un avviso:** testo e altri tipi di livello, asset audio, punti di ancoraggio animati (viene usato il primo valore), percorsi di movimento curvi su
  proprietà diverse dalla posizione, maschere su una sorgente di track matte, catene di genitori troppo lunghe da bakare, più di 2000 livelli, animazione più lunga
  di 20,000 fotogrammi.
* `fx.json` è mantenuto per compatibilità: viene scritto un array vuoto.
* **Non supportato:** input zip `.lottie`. **Attenzione:** non c'è alcuna protezione contro la sovrascrittura dell'input: `import-lottie in.json in.json` sostituisce il
  Lottie con il progetto.

### Immagini, GIF e APNG come livelli fotogramma per fotogramma

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20      # folder of PNGs
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0                    # animated GIF/APNG
```
`import-seq` prende tutti i PNG di una cartella in ordine di nome file (tutti della stessa dimensione); `import-anim` legge una GIF o APNG animata usando i
ritardi propri del file (un PNG semplice dà un fotogramma). Entrambi aggiungono un livello switch con un'immagine incorporata per fotogramma più un canale a gradini; operano su un
progetto **esistente**. Una GIF lunga rende grande il progetto (ogni fotogramma è un PNG incorporato). `trace` e `autotrace` trasformano le bitmap in percorsi
vettoriali (`trace` legge il canale alpha; `autotrace` quantizza i colori e adatta curve di Bezier).

## 10. Importazione: motion capture BVH, MIDI e altri input in tempo reale

### BVH (`import-bvh`, `mocap-info`, `mocap-match`, `mocap-retarget`)

```
$ osatura mocap-info walk.bvh
$ osatura import-bvh demo.json walk.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```
`import-bvh` mappa i canali di rotazione di giunti scelti sugli ossi come canali `bone_angle` con chiavi, campionati al fps e al numero di fotogrammi del progetto
(`--axis` seleziona `Zrotation`, `Xrotation` o `Yrotation`; `--tol` semplifica le chiavi). `mocap-match` propone un abbinamento giunto-osso con una
tabella di confidenza e `mocap-retarget` esegue il baking dei canali aggiunti con root motion, una vista (`front|side|top|yaw=DEG`) e una scala. Il BVH è
solo un sottoinsieme testuale: nessun formato mocap binario, nessun FBX (per scelta). **Limiti (LIMITATIONS L-45):** nessun vero file di motion capture è stato testato; la proiezione
2D scarta la profondità; la torsione attorno all'asse di un osso, le dita e il volto sono ignorati; il rig mantiene le lunghezze dei suoi ossi, quindi lo scivolamento dei piedi viene solo
ridotto.

### MIDI e altri input in tempo reale (`live-map`)

```
$ osatura live-map small.json song.mid mapping.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote out.json
```
Un **Standard MIDI File** (`.mid`; messaggi MIDI 1.0, running status, meta-eventi di tempo, divisione in tick o SMPTE) oppure un JSON di eventi
(`{"events":[{"source","id","value","time"}]}`, valore normalizzato 0..1, tempo in secondi) viene trasformato in canali con chiavi, eventi e take tramite un
file di mappatura. Sorgenti: `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`. Al massimo
3,000,000 eventi; tempi entro 7 giorni. Nulla dalla CLI comunica con un dispositivo MIDI; lo studio usa il Web MIDI del browser. Il MIDI è **solo
importazione** (non esiste esportazione MIDI). I file malformati restituiscono un errore, mai un crash.

## 11. Formati audio

| Uso | Formati | Note |
|---|---|---|
| Analisi (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) | solo **WAV** | PCM intero a 8/16/24/32 bit o float a 32 bit, qualsiasi numero di canali mixato in mono, fino a 256 MiB. Converti prima MP3/OGG (`ffmpeg -i in.mp3 out.wav`). |
| Tracce sonore del progetto (`sound add`) | WAV, OGG, MP3, qualsiasi formato accettato da `ffprobe` | Memorizzate come **riferimento** (percorso relativo, fotogramma di inizio, gain 0..4, finestra di trim, dissolvenze); nulla è incorporato; il file deve restare all'interno della cartella del progetto a meno di `--allow-external-assets`. |
| Output mixato (`sound mix`) | `.wav` (PCM 44.1 kHz a 16 bit), `.ogg` (Vorbis q4), `.mp3` (96 kb/s) | Mixato tramite ffmpeg per la **lunghezza del progetto**; `.flac` e altri: `unknown audio format '.flac' (use .wav, .ogg or .mp3)`. |
| Dentro le esportazioni | `.mp4` AAC, `.webm` Opus, `.html` mp3 incorporato | `.gif`/`.apng` restano muti con una nota; Lottie segnala `[audio]`; GLB e OCA scartano il suono. |
| Indicazioni per la bocca (`mouth`) | testo Papagayo/Moho-switch, TSV di Rhubarb | Non è audio: sono file di cue. |

Encoder usati: `libmp3lame` (mp3), `libvorbis` (ogg), `aac`, `libopus`. Nessun pan/EQ/ducking per traccia né looping (ROADMAP sezione 9).

## 12. Matrice di supporto delle funzionalità

Le righe sono le cose che un progetto può contenere, le colonne i formati di output. **sì** = conservato come nel progetto, **approx** = conservato con una
differenza visibile, **baked** = presente solo come risultato per fotogramma, **no** = scartato. Le celle dei formati che non ho potuto controllare in un visualizzatore indipendente
descrivono ciò che l'esportatore scrive.

| Funzionalità | SVG / PNG | Sprite sheet / APNG | GIF | WebM / MP4 | HTML | Lottie / .lottie | GLB | OCA |
|---|---|---|---|---|---|---|---|---|
| Forme vettoriali, riempimenti | sì | baked (raster) | baked (raster) | baked (raster) | sì (SVG) | sì | approx (triangoli) | baked (raster) |
| Gradienti | sì | baked | baked | baked | sì | sì (approx se il livello ha skew) | sì (texture) | baked |
| Tratti con profilo di larghezza | sì | baked | baked | baked | sì | approx (larghezza costante) | sì | baked |
| Tratteggi, terminazioni, giunzioni | sì | baked | baked | baked | sì | sì | sì (triangolati) | baked |
| Livelli immagine (bitmap) | sì | baked | baked | baked | sì | no (`image-layer`) | sì | baked |
| Ossi e animazione | baked | baked | baked | baked | baked | baked | **sì (esatto)** | baked |
| Skinning / deformatori | baked | baked | baked | baked | baked | baked | con skin; deformatori al fotogramma 0 | baked |
| Camera e parallasse | baked | baked | baked | baked | baked | baked (i movimenti di camera sono nella geometria) | no (le forme provengono dal fotogramma 0) | baked |
| Modalità di fusione | sì | baked | baked | baked | sì | approx (`add` diventa `screen`; nessuna sui gruppi) | no (normal) | baked |
| Maschere dei livelli | sì | baked | baked | baked | sì | solo singoli livelli | forma del fotogramma 0, con restrizioni | baked |
| Effetti dei livelli (blur, ombra...) | sì | baked | baked | baked | sì | livelli a foglia singola | no | baked |
| Opacità animata | baked | baked | baked | baked | baked | sì | no (congelata al fotogramma 0) | baked |
| Path morph | baked | baked | baked | baked | baked | baked | morph target (quando validi) | baked |
| Profondità di campo | tramite `dof-export` | tramite `dof-export` | tramite `dof-export` | tramite `dof-export` | no | no (con avviso) | no | no |
| Catena di post-effetti (`fx`) | solo PNG | no | no | no | no | no | no | no |
| Trasparenza | sì | sì (alpha) | 1 bit | WebM sì, MP4 no | sì | sì | sì | sì |
| Audio | no | no | no | **sì** | sì (mp3) | no (con avviso) | no | no |
| Looping | n/d | sheet: n/d, APNG: sì | sì | dipende dal player | player | player (dotLottie 2.0 lascia il loop al player o a una macchina a stati) | n/d | n/d |
| Rig/canali modificabili in seguito | no | no | no | no | no | no | solo ossi | no |
| Macchine a stati, binding, container, scroll | no | no | no | no | no | no | no | no |
| Indipendente dalla risoluzione | sì (SVG) | no | no | no | sì | sì | sì | no |
| Richiede ffmpeg | no | no | no | **sì** | solo audio | no | no | no |

"Baked" significa che l'effetto della funzionalità è nei pixel o nella geometria per fotogramma, ma la funzionalità stessa non può essere modificata in seguito.

## 13. Come ridurre i file

1. **Vedi dove sono i byte:** `osatura size-report project.json` elenca il file di progetto, il Lottie baked e la pagina HTML per livello,
   immagine e canale (`--json` per gli script).
2. **Rientra automaticamente in un budget:** `osatura export-fit project.json out.json --max-size 50KB --format lottie` cerca tra i parametri dell'esportatore e
   mantiene il miglior SSIM che rientra nel budget. Formati: `lottie`, `html`, `gif`, `apng`, `webm`, `project` (un JSON di progetto alleggerito). `--measure gzip` imposta il budget sulla
   dimensione gzip, `--min-ssim X` imposta una soglia minima di qualità, `--max-seconds N` è un vero limite di tempo reale (viene mantenuto il miglior risultato ottenuto fino a quel momento). Exit 1 con il
   risultato più piccolo quando il budget non può essere rispettato.
3. **Riduci prima la grafica:** `osatura lod project.json --tolerance-px 1 --levels 3 --out-dir lv --verify` scrive progetti a livelli di dettaglio con un errore
   dichiarato (e, con `--verify`, misurato) in pixel di output; `lod-pick --screen-px WxH` sceglie il livello più grossolano per un dato schermo.
4. **Per formato:**
   * Lottie/HTML/GLB crescono con `frames x anchors`: meno fotogrammi, meno ancore, un fps più basso, intervalli di chiavi tagliati (strumenti `keys`).
   * GIF/APNG crescono con pixel x fotogrammi: `--scale 0.5` è una riduzione di 4 volte; la GIF si adatta alla grafica piatta, APNG all'alpha, WebM/MP4 a qualsiasi cosa con movimento.
   * `.lottie` è lo stesso Lottie compresso con deflate (circa 4 volte più piccolo nell'esempio); servi `.html`/`.json` compressi con gzip.
   * Le immagini in un progetto sono incorporate come PNG base64: riducile prima dell'importazione.
5. Limiti: un Lottie di un rig a 14 livelli non può essere reso piccolo a piacere (un budget di 1 KB sull'esempio riporta "every knob is at its floor"); la
   ricerca richiede tempo sui progetti grandi.

## 14. Requisiti di ffmpeg

OSATURA non include né collega mai ffmpeg; esegue `ffmpeg` (e `ffprobe`) trovato nel `PATH` come **sottoprocesso esterno** della tua installazione.

| Necessario per | Encoder/decoder | Verificato su |
|---|---|---|
| `export .mp4` | `libx264` (`yuv420p`), `aac` | ffmpeg 6.1.1 |
| `export .webm` (predefinito) | `libvpx-vp9`, `libopus` | ffmpeg 6.1.1 |
| `export .webm --codec av1` | `libaom-av1`, `libopus` | ffmpeg 6.1.1 |
| `dof-export .mp4/.webm` | come sopra, `--codec` | ffmpeg 6.1.1 |
| `sound mix .ogg` | `libvorbis` | ffmpeg 6.1.1 |
| `sound mix .mp3`, audio `.html` | `libmp3lame` | ffmpeg 6.1.1 |
| `face-track` da un video | solo decoder | |
| controllo del contenuto di `sound add` | `ffprobe` (senza di esso, per i file non WAV viene controllata solo l'esistenza) | |

Controlla il tuo con `ffmpeg -hide_banner -encoders | grep -E 'libx264|aac|libvpx-vp9|libaom-av1|libopus|libvorbis|libmp3lame'`. Tutto il resto
(GIF, APNG, HTML senza audio, Lottie, dotLottie, GLB, OCA, PNG, SVG, EXR) funziona senza ffmpeg.
Se ffmpeg manca il comando fallisce con `cannot run ffmpeg (...)` e non scrive nulla; un encoder mancante dà `ffmpeg failed`.

## 15. Riferimenti a brevetti e licenze

* [PATENT_NOTES.md](PATENT_NOTES.md) registra ciò che **non** è stato vagliato. Per l'esportazione, la riga rilevante è *Video and audio codecs through ffmpeg
  (`export` .mp4 / .webm, `soundtrack`)*: H.264 e AAC sono coperti da pool di licenze; VP9 e AV1 sono presentati come privi di royalty, ma per entrambi sono state annunciate rivendicazioni di pool e individuali;
  **stato: non valutato, non trattare VP9/AV1 come approvati.** Poiché ffmpeg è l'installazione dell'utente, ogni
  scelta di codec è responsabilità dell'utente, e OSATURA non deve includere ffmpeg.
* I writer deterministici di raster, GIF, APNG, PNG, SVG, OpenEXR, OCA, ORA, glTF e Lottie sono implementazioni proprie di OSATURA a partire da descrizioni
  pubbliche dei formati (clean-room, vedi CLEANROOM.md). Il loro stato dei brevetti non è valutato oltre a questo: PATENT_NOTES.md elenca solo le aree sopra citate
  (puppeteering guidato dal volto, macchine a stati e data binding, FABRIK/MVC/pesi, riempimento stencil-then-cover, codec).
* GIF: i brevetti LZW sono scaduti da tempo in tutte le giurisdizioni; è una conoscenza generale, non un'autorizzazione registrata nel repository.
* Licenze dei file prodotti: i file che esporti sono tuoi; il motore è MPL-2.0 e la CLI AGPL-3.0-or-later, il che si applica ai programmi, non ai
  file che scrivono. Lo studio offre il proprio codice sorgente corrispondente a `/source` (AGPL sezione 13).
* Questa non è una consulenza legale.

## 16. Convalidare ciò che hai scritto

| Formato | Controllo | Atteso |
|---|---|---|
| qualsiasi progetto | `osatura check out.json` | `ok: ...` (anche dopo `import-*`) |
| GIF | `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` | numero di fotogrammi (8) |
| APNG | lo stesso comando su `out.apng` | numero di fotogrammi (8) |
| MP4 | `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4` | `h264,320,240,yuv420p,24/1,8` |
| WebM | lo stesso su `out.webm` con `:stream_tags=alpha_mode` | `vp9,...,yuv420p,...,1` (alpha_mode 1) |
| EXR | `ffprobe f.exr` | `exr 640x480 gbrapf32le` |
| Lottie | `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],d['w'],d['h'],len(d['layers']))"` | `5.7.0 24.0 8 640.0 480.0 12` |
| Avvisi Lottie | `osatura export p.json out.json --report rep.json` | `rep.json` elenca ogni approssimazione |
| dotLottie | `unzip -l out.lottie` | solo `manifest.json` e `a/animation.json` |
| Round trip Lottie | `osatura import-lottie out.json back.json && osatura check back.json` | `ok:` |
| GLB | `python3 -c "import struct;b=open('out.glb','rb').read();print(b[:4],struct.unpack('<II',b[4:12]),len(b))"` | `b'glTF' (2, N)` con N uguale alla lunghezza |
| HTML | `grep -c '<svg' out.html`; `grep -o 'http[^"<]*' out.html` | un `<svg` per fotogramma; solo l'URL del namespace SVG |
| SVG | apri in un browser o analizza come XML | ben formato |
| Sprite sheet | `python3 -m json.tool out.atlas.json` | `{index,x,y,w,h}` per fotogramma |
| OCA | `python3 -m json.tool <name>.oca` | ogni cel referenziata esiste |
| Regressione visiva | `osatura ci diff a.png b.png --metric ssim` | exit 0 se uguali; vedi [CI.md](CI.md) |

Per i controlli visivi di un'esportazione con perdita rispetto all'originale, `osatura export-fit` riporta già SSIM e CIEDE2000 rispetto al render originale, e
`osatura compare` scrive un'immagine prima/dopo/differenza di due progetti.

## 17. Differenze rispetto ad altri documenti

* `docs/LIMITATIONS.md` L-29 dice che l'importazione Lottie non accetta asset immagine, effetti, precomp o matte; l'importatore attuale importa invece le immagini
  con data URI incorporati, le precomposizioni, gli effetti per livello (blur, ombra, tinta, riempimento) e approssima i luma matte (sezione 9). L'elenco delle
  funzionalità saltate nella sezione 9 è ciò di cui il comando avvisa realmente.
* `docs/LIMITATIONS.md` L-32 elenca OCA tra i formati di importazione; la CLI ha un'**esportazione** OCA (`export-oca`) e nessun comando `import-oca`.
* Il testo d'uso di `export` nomina solo `.gif .html .json .mp4` più "i formati aperti"; l'elenco completo delle estensioni è quello della sezione 2.
* Altre discrepanze tra il testo d'uso e il comportamento sono raccolte alla fine di [CLI_REFERENCE.md](CLI_REFERENCE.md).
