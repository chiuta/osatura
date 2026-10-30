<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Riferimento della riga di comando di OSATURA

Riferimento per il binario `osatura` (crate `osatura-cli`, AGPL-3.0-or-later; collega il motore `osatura-core`, MPL-2.0).
Versione documentata: **osatura 0.1.0**. Questo testo è rilasciato con licenza CC-BY-4.0.

Ogni sinossi qui sotto è copiata dal testo d'uso del programma stesso (`osatura` senza argomenti, oppure
`osatura <command> --help`). Ogni esempio è stato eseguito sul progetto demo integrato e l'output mostrato
è ridotto rispetto a quello reale. Dove il testo d'uso e il comportamento reale non coincidono, questo documento descrive il
comportamento reale ed elenca la discrepanza in [Discrepanze note tra documentazione e comportamento](#discrepanze-note-tra-documentazione-e-comportamento) alla fine.

Documenti correlati: [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (che cosa conserva e che cosa scarta ogni formato di esportazione/importazione),
[CI.md](CI.md) (manifest CI per le animazioni), [API.md](API.md) (libreria/ABI C), [PATENT_NOTES.md](PATENT_NOTES.md)
(stato di codec e brevetti, non è una consulenza legale).

## Indice

1. [Convenzioni comuni a tutti i comandi](#1-convenzioni-comuni-a-tutti-i-comandi)
2. [Primi passi: il progetto demo](#2-primi-passi-il-progetto-demo)
3. [Creazione e ispezione dei progetti](#3-creazione-e-ispezione-dei-progetti)
4. [Rendering ed esportazione](#4-rendering-ed-esportazione)
5. [Rigging, skin, IK e strumenti di layout](#5-rigging-skin-ik-e-strumenti-di-layout)
6. [Animazione, chiavi, timeline e generatori](#6-animazione-chiavi-timeline-e-generatori)
7. [Fisica, cloth e deformatori](#7-fisica-cloth-e-deformatori)
8. [Audio, lip-sync e suoni](#8-audio-lip-sync-e-suoni)
9. [Importazione (SVG, OpenRaster, Lottie, BVH, GIF/APNG, sequenze PNG, bitmap)](#9-import)
10. [Input live, MIDI, face tracking e motion capture](#10-input-dal-vivo-midi-face-tracking-e-motion-capture)
11. [Macchine a stati, data binding, contenitori e regioni di scorrimento](#11-state-machine-data-binding-container-e-regioni-di-scorrimento)
12. [Effetti, profondità di campo, onion skin, trail e camera](#12-effetti-profondità-di-campo-onion-skin-scie-e-camera)
13. [Livello di dettaglio, report sulle dimensioni ed export-fit](#13-livello-di-dettaglio-report-sulle-dimensioni-ed-export-fit)
14. [Controllo qualità: check, lint, ci, golden, fuzz, diff](#14-controllo-qualità)
15. [Controllo di versione (`vcs`)](#15-controllo-di-versione-vcs)
16. [Studio, collaborazione, server agente MCP e registro della UI](#16-studio-collaborazione-server-agente-mcp-e-registro-ui)
17. [Database di pose, suggerimenti e altri strumenti](#17-database-di-pose-suggerimenti-e-altri-strumenti-di-supporto)
18. [Discrepanze note tra documentazione e comportamento](#discrepanze-note-tra-documentazione-e-comportamento)
19. [Indice alfabetico dei comandi](#indice-alfabetico-dei-comandi)

---

## 1. Convenzioni comuni a tutti i comandi

### 1.1 Forma della riga di comando

```
osatura <command> [sub-command] <positional arguments...> [options]
```

* Le opzioni possono comparire **ovunque** nella riga, anche dopo il sotto-comando e dopo gli argomenti posizionali
  (`osatura render demo.json 0 o.png --scale 2` e `osatura --scale 2 render ...` sono equivalenti).
* Le **opzioni con valore** prendono la parola successiva (`--scale 2`) oppure la forma `--key=value` (`--scale=2`). La forma con `=` funziona per
  ogni opzione con valore. Un valore che contiene a sua volta `=` va bene con la forma separata da spazio (`--remap a=b`).
* I **flag** (opzioni booleane come `--bones`) non prendono alcun valore. `--bones=1` viene rifiutato:
  `--bones is a flag and takes no value: write --bones (without =), got '--bones=1'`.
* **Opzioni rigorose.** Un'opzione (flag od opzione con valore) che il comando non documenta è un errore con un suggerimento
  "did you mean" (uscita 1), invece di essere ignorata:
  ```
  $ osatura render demo.json 0 o.svg --bone
  unknown option --bone for 'render' (did you mean --bones?); see: osatura render --help
  ```
  Il controllo vale per ogni comando che ha una riga d'uso, inclusi `check`, `stats`, `lint`, `coach`, `contracts` e
  `machine list` (le versioni precedenti ignoravano in silenzio i flag in più). `--json`, `--quiet` e `--allow-empty` **non** sono
  globali: vengono rifiutati da ogni comando la cui riga d'uso non li elenca.
  **Il controllo è per sotto-comando (per forma di sinossi), non per comando.** Un'opzione elencata per una forma viene rifiutata
  nelle forme sorelle, e il messaggio indica la forma che la accetta:
  ```
  $ osatura ci diff a.png b.png --quiet
  unknown option --quiet for 'ci diff' (--quiet applies to: ci run); see: osatura ci --help
  ```
  Quindi `--quiet` vale solo per `ci run`, `--json` solo per `vcs log|status|diff|show`, `ui commands|shortcuts` e la
  forma `trail ... --target`, `--tol` solo per `golden check` (e `ci diff`), `--scale` solo per `golden write`, e
  ogni modifica di `keyedit` accetta soltanto le opzioni scritte dopo il suo nome nella sinossi. (`export` unisce ancora le sue due
  righe di sinossi, perché il formato è scelto dall'estensione del file; `--codec`, `--columns` e `--scale` che non
  si applicano all'estensione scelta vengono segnalati da `export` stesso.)
* Alcuni comandi più vecchi prendono **parole** `key=value` invece di `--options` (`cloth`, `physics`, `cage`, `warp-pivot`,
  `soft-scale`, `mesh-lock`, `glue`, `audio-react`, `beats`, `import-seq`, `import-anim`, `layer-pivot`, `chan-meta`,
  `waveform`). Sono posizionali e si scrivono esattamente come nella sinossi (`gravity=900`, non `--gravity 900`).
* Un **fotogramma** è un numero (frazioni ammesse dove ha senso). Un progetto con `frames = N` ha i fotogrammi `0..N-1`.
  I livelli sono indirizzati per **percorso** (`group/child`), le ossa per **nome**; la camera ha il target vuoto `""`.
* I **nomi delle proprietà** (usati da `--prop`, `slider`, `drive`, file di mappatura, ...) sono i nomi in snake_case:
  `bone_angle bone_scale bone_offset_x bone_offset_y layer_opacity layer_x layer_y layer_rotation layer_scale_x
  layer_scale_y layer_skew layer_switch layer_wobble camera_x camera_y camera_zoom camera_rotation camera_focus
  camera_blur ik_mix constraint_weight path_morph point_dx point_dy lattice_dx lattice_dy spine_dx spine_dy pin_dx
  pin_dy cage_dx cage_dy fx_blur fx_dx fx_dy fx_soft fx_amount grad_dx grad_dy`.  Alcuni comandi stampano le stesse proprietà in CamelCase (`BoneAngle`) nei loro messaggi; i valori delle opzioni sono sempre in snake_case.

### 1.2 Opzioni globali

| Opzione | Significato |
|---|---|
| `--allow-external-assets` | Permette a un progetto di indicare file con percorso assoluto o esterni alla propria directory (suoni, immagini, normal map...). Predefinito: rifiutato. |
| `--follow-symlinks` | Scrive nella **destinazione** di un collegamento simbolico indicato come file di output. Predefinito: rifiutato. |
| `--max-input-bytes N` | Il file di progetto più grande che viene letto. Predefinito `268435456` (256 MiB). Deve essere un intero positivo. |
| `<command> --help` | Stampa su stdout le righe d'uso di un comando (uscita 0). |
| `<command> --version`, `--version` | `osatura 0.1.0`; senza comando anche la riga della licenza e l'offerta del sorgente AGPL. |
| `--json`, `--quiet`, `--allow-empty` | Opzioni per singolo comando, **rifiutate** (`unknown option`) dai comandi la cui sinossi non le elenca (`--json`: `pick`, `select-box`, `timeline`, `trail`, `size-report`, `collab-sim`, `vcs`, `ui`, `face-detect`, `pose-suggest`; `--allow-empty`: `physics`, `live-map`, `face-bake`; `--quiet`: `ci run`). Gli script che le passavano in modo uniforme a ogni comando devono smettere di farlo. |

Esempi (verificati):

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

### 1.3 Lettura dei file di input

* I file di input devono essere **file regolari** non più grandi di `--max-input-bytes`. Dispositivi, pipe, directory e
  sostituzioni di processo (`<(...)`) vengono rifiutati: scrivi prima i dati in un file.
* I file di progetto sono JSON. Un progetto con chiavi sconosciute viene **accettato** e le chiavi sconosciute sono segnalate come
  `warning: unknown field bogus_key (ignored)` da `check` (e da `lint`, `studio`). I file ausiliari che scrivi per un comando
  (contratti, mappature, macchine, contenitori, regioni di scorrimento, manifest ci, mappe drive ...) vengono analizzati in modo **rigoroso**: una chiave
  sconosciuta è un errore e non viene applicato nulla (`unknown field(s): elastic.damping (typo? nothing was applied)`).
* Un progetto viene validato ogni volta che è caricato per il rendering o l'esportazione (`<file> is not a valid project:` seguito da
  un problema per riga). Anche i comandi di modifica ricompilano il progetto che stanno per scrivere e rifiutano di scriverne uno che non sia più valido.
* I **percorsi degli asset** all'interno di un progetto (file audio, file immagine, normal map) sono risolti rispetto alla **directory del file di
  progetto**. I percorsi assoluti e qualunque percorso contenente `..` vengono rifiutati, a meno che non sia indicato `--allow-external-assets`:
  ```
  $ osatura sound add s.json o.json x /etc/passwd
  sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)
  ```
  Il flag va indicato di nuovo a ogni comando successivo che carica quel progetto (`check ext.json` fallisce senza, riesce con).

### 1.4 Scrittura dei file di output

* **Scritture atomiche.** La maggior parte dei comandi scrive attraverso un file temporaneo nella **stessa directory** (`.<name>.tmp-<pid>-<random>`), ne esegue `fsync`,
  copia i permessi del vecchio file e lo `rename` sopra la destinazione, così un crash o un disco pieno non lasciano mai un file scritto a metà.
  Se non è possibile creare un file temporaneo (directory di sola lettura ma file scrivibile) il file viene sovrascritto sul posto, il che **non** è atomico.
  Le FIFO e i file speciali tipo `/dev/stdout` vengono scritti direttamente.
* I **collegamenti simbolici** come output vengono rifiutati, a meno che non sia indicato `--follow-symlinks` (allora viene scritta la destinazione del collegamento; un collegamento pendente resta comunque un errore).
  Una directory indicata come file di output è un errore (`is a directory`).
* **Mai sopra l'input.** `render`, `render-hdr`, `dof`, `dof-export`, `export` (anche il sidecar `.json` dello sprite sheet di un `.png`
  e `--report`), `layout-preview`, `render-gpu`, `export-fit`, `fuzz-grid`, `onion`, `trail` e `golden` rifiutano un output che sia lo stesso file
  del progetto di input: `refusing to overwrite the input project demo.json with output of the same name`.
* I **comandi di modifica** (quelli con una coppia `<project.json> <out.json>`: `reparent`, `keys`, `physics`, ...) possono scrivere **sul posto**
  (`osatura reparent p.json p.json hand none` funziona e sostituisce `p.json`). `import-lottie` rifiuta un output (o `fx.json`) che sia
  lo stesso file del suo input; gli altri comandi di importazione leggono un tipo di file diverso da quello che scrivono.
* **Gli output esistenti vengono sovrascritti in silenzio** da ogni comando. Non esiste `--force`.
* **Ogni output indirizzato dall'utente** (inclusi i report e i golden di `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, i file `face-*`,
  i file di pose, gli snapshot, `vcs` e il log di collaborazione dello studio) passa per lo stesso writer atomico che rifiuta i collegamenti simbolici. Gli output
  `.mp4`/`.webm` scritti da ffmpeg vengono controllati per i collegamenti simbolici prima dell'avvio di ffmpeg. Cartelle padre: `lod`, `export-fit`, `ci` e `face-*`
  creano le cartelle mancanti; `fuzz`, `fuzz-grid` e `export` no (`No such file or directory`).

### 1.5 stdout, stderr e codici di uscita

* I risultati ("wrote out.json", tabelle, JSON) vanno su **stdout**. Gli errori e la maggior parte degli avvisi vanno su **stderr**. `import-svg` stampa i suoi
  avvisi su stdout; `export` stampa gli avvisi di Lottie/GIF/GLB su stderr.
* Il codice di uscita **0** significa successo. Il codice di uscita **1** significa qualunque errore *oppure* un controllo non superato: un file illeggibile/non valido, argomenti errati,
  `lint` con errori, `diff` con differenze, un contratto violato, un golden non corrispondente, un tasso di fallimento del fuzz oltre il limite,
  un budget di `export-fit` che non è stato possibile rispettare, `lod --verify` oltre il limite, divergenza di `collab-sim`, un controllo `ci` fallito.
  Il codice di uscita **2** è usato solo da `ci` (non è stato possibile valutare nulla: progetto/manifest mancante, golden manomesso).
* Nessun argomento, un comando sconosciuto, un numero errato di argomenti posizionali o `osatura --help` stampano l'uso completo su
  **stderr** e terminano con **1**. `osatura <command> --help` stampa su stdout le righe d'uso di quel comando e termina con 0.
  **Eccezione (Windows, dalla 0.1.0-alpha.2):** quando `osatura.exe` viene avviato con un doppio clic da Esplora file (la finestra della console è stata creata per lui e nessun altro processo la condivide) e non ha argomenti, non stampa l'uso: crea `Documents\OSATURA\first-rig.json` (da `demo-person`) se manca, esegue `studio` su quel file su `127.0.0.1:8080`, apre il browser e tiene aperta la console finché non la chiudi o premi Invio dopo un errore. Avviato da un terminale, si comporta come descritto sopra. Il resto del programma gira su un thread con uno stack da 256 MiB (il thread principale di Windows ne ha solo 1 MiB).

### 1.6 Programmi esterni

`ffmpeg` (trovato nel `PATH`) è necessario per: l'esportazione `.mp4`/`.webm` e `dof-export` verso questi formati, `sound mix`,
l'audio in `.html` (mp3) e `face-track` su un file video. Encoder necessari: `libx264` (mp4), `libvpx-vp9` / `libaom-av1` (webm),
`aac` (suono mp4), `libopus` (suono webm), `libvorbis`, `libmp3lame` (sound mix). Non è incluso alcun codice né binario di ffmpeg; vedi
[PATENT_NOTES.md](PATENT_NOTES.md). Non viene eseguito nient'altro.

### 1.7 Nota sulle prestazioni

Tutti i tempi in questo documento provengono da una build **debug** e sono 10-30 volte più lenti di una build release. I comandi pesanti sui fotogrammi
(`safety`, `golden`, `ci`, `export-fit`, `proxy-bench`) sono stati eseguiti su una copia ridotta di 8 fotogrammi della demo (`small.json`, ottenuta impostando
`"frames": 8`).

---

## 2. Primi passi: il progetto demo

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

La demo ha le ossa `arm_up arm_low hand thigh shin foot_ctrl`, una catena IK `leg` e i livelli di primo livello
`sky sun hills ground shadow arm leg target blush mouth(switch: closed/open/smile) ink`. Tutti gli esempi di questo riferimento la usano
salvo diversa indicazione.

---

## 3. Creazione e ispezione dei progetti

### `demo`

**Sinossi:** `osatura demo <out.json>`

Scrive il rig demo integrato (640x480, 24 fps, 48 fotogrammi, 6 ossa, gamba IK, 14 livelli) come file di progetto.
**Output:** `wrote <out.json>`. **Errori:** argomento mancante (uso, uscita 1); percorso non scrivibile. **Correlati:** `demo-fx`, `check`.
**Limiti:** la demo è un asset fisso; è la base di ogni esempio di questo documento.

```
$ osatura demo demo.json
wrote demo.json
```

### `demo-person`

**Sinossi:** `osatura demo-person <out.json>`

Scrive la demo della persona che cammina (640x480, 24 fps, 32 fotogrammi, 42 ossa, 45 livelli) come file di progetto: una persona vista di lato con testa, orecchio e coda di cavallo, collo, tronco, due braccia con mani e cinque dita ciascuna, e due gambe con piedi e dita dei piedi. Il ciclo di camminata dura due passi e si ripete senza stacchi: il bacino oscilla e si inclina, il tronco si piega e ruota in controfase, la testa resta in piano con un piccolo cenno in ritardo, le braccia oscillano in senso opposto alle gambe con i gomiti che si piegano di più nell'oscillazione in avanti, e mani, dita, orecchio e coda di cavallo seguono un attimo dopo. Ogni piede atterra sul tallone, rotola fino ad appoggiarsi piatto e si stacca sull'avampiede; il terreno, la recinzione e l'ombra scorrono esattamente alla velocità del piede appoggiato, così i piedi non scivolano. È il progetto che i launcher per Windows (`osatura.exe`, `osatura-studio.exe`) creano come `first-rig.json` al primo avvio; `osatura demo` resta la piccola base degli esempi di questo documento.
**Output:** `wrote <out.json>`. **Errori:** argomento mancante (uso, uscita 1); percorso non scrivibile. **Correlati:** `demo`, `studio`.
**Limiti:** le chiavi sono campionate ogni due fotogrammi, con una chiave in più prima del fotogramma 0 e una dopo l'ultimo, così il loop è fluido; `lint` segnala quelle due chiavi come fuori dalla timeline (informazione, non errore). La figura è vista di lato, quindi non ci sono rotazioni né profondità.

```
$ osatura demo-person person.json && osatura check person.json
wrote person.json
ok: 'OSATURA walking person' 640×480 @ 24 fps, 32 frames, 42 bones, 0 ik, 45 layers, 48 channels
```

### `demo-fx`

**Sinossi:** `osatura demo-fx <out.json>`

Scrive la demo "liquid / boolean / particles" (640x480, 72 fotogrammi, 3 ossa, 3 livelli) che mette alla prova i livelli procedurali.

```
$ osatura demo-fx fx.json && osatura check fx.json
wrote fx.json
ok: 'OSATURA fx demo' 640×480 @ 24 fps, 72 frames, 3 bones, 0 ik, 3 layers, 3 channels
```

### `check`

**Sinossi:** `osatura check <project.json>`

Carica e valida un progetto. Stampa righe `warning:` (campi sconosciuti, problemi di gradiente, problemi delle chiavi di switch e gli avvisi sui dati dell'editor del codice lint L013: zoom della camera <= 0, opacità del livello fuori da 0..1, riferimenti o tempi errati in skin, comp, marker, eventi, shot, channel_meta, bone_vis, clip e lod) e una riga di riepilogo `ok:`. Sono errori solo i valori che non possono esistere in un file salvato (NaN o infinito in quelle strutture), così i progetti più vecchi continuano a caricarsi.
Vengono aperte anche le tracce audio: un file audio illeggibile/inutilizzabile è un **errore**. **Uscita:** 0 valido; 1 non valido.

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

**Correlati:** `lint` (qualità di rig/animazione), `ci` (gate di regressione). **Attenzione:** `check` non renderizza nulla; un progetto può superare `check` ed
essere comunque in errore più tardi su un fotogramma (le pose `NaN` vengono intercettate da `render`/`export`, che valutano prima ogni fotogramma).

### `stats`

**Sinossi:** `osatura stats <project.json>`

Stampa i conteggi come JSON: `name, size, fps, frames, seconds, bones, ik_chains, constraints, springs, layers, channels, keyed_channels, keys, markers`.
L'output è sempre JSON; non esiste l'opzione `--json` (viene rifiutata come qualunque opzione sconosciuta).

```
$ osatura stats demo.json
{ "name": "OSATURA demo", "size": [640.0, 480.0], "fps": 24.0, "frames": 48, "seconds": 2.0, "bones": 6,
  "ik_chains": 1, "constraints": 0, "springs": 0, "layers": 14, "channels": 14, "keyed_channels": 8, "keys": 37, "markers": 0 }
```
(l'output reale è formattato su più righe.) Nota che `layers` conta ogni nodo inclusi i figli di gruppi/switch, mentre `check` stampa lo stesso numero.

### `diff`

**Sinossi:** `osatura diff <a.json> <b.json>`

Differenza semantica tra due progetti: `+` aggiunto, `-` rimosso, `~` modificato. Insensibile all'ordine per le liste indirizzate per nome.
**Uscita:** 0 identici (`no differences`); **1** quando ci sono differenze (il conteggio `N difference(s)` va su stderr), così può fare da gate in uno script.

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
**Correlati:** `vcs diff` (cronologia), `compare` (differenza di pixel), `ci diff` (differenza di immagini).

### `tag`

**Sinossi:** `osatura tag <project.json>`

Propone nomi di ruolo standard (`arm.upper.l`, `hand.l`, `face.mouth` ...) per ossa e livelli a partire dai loro nomi, con un grado di confidenza, ed elenca i ruoli standard per cui non è stato trovato alcun osso. Sola lettura; usa i ruoli come guida per `mocap-match` o `autorig`.

```
$ osatura tag demo.json
bone  arm_up                   -> arm.upper.l    (45%)
bone  hand                     -> hand.l         (45%)
layer mouth                    -> face.mouth     (90%)
no bone found for: head, neck, torso, hips, face.eye.l, ...
```

### `explain`

**Sinossi:** `osatura explain <project.json> <bone-or-layer> <frame>`

"Perché è qui?" - elenca in ordine i contributi (posa di riposo, ogni canale con la sua modalità e il suo peso, vincoli, IK) che producono la trasformazione di un osso o livello a un dato fotogramma, e l'angolo head/tip/world risultante.

```
$ osatura explain demo.json hand 12
- bone 'hand' (length 45), parent 'arm_low'
- ancestry: hand ← arm_low ← arm_up
- rest pose: offset (100.00, 0.00), angle 0.00°
- channel #2 BoneAngle: keys → 20.000 (5 key(s), Hold outside range); mode replace, weight 1.00
- channel #3 BoneAngle: expression `6 * sin(time * 2 * pi * 2)`; mode add, weight 1.00
- result at frame 12: head (174.6, 126.4), tip (203.5, 92.0), world angle -50.0°
```
**Errori:** nome di osso/livello sconosciuto; fotogramma fuori dall'animazione.

### `pick`

**Sinossi:** `osatura pick <project.json> --frame N --x X --y Y [--tol PX] [--root] [--bones] [--points a,b] [--locked a,b] [--json]`

Che cosa c'è sotto un punto del canvas, dal più in alto (hit-testing come lo fa lo studio).

| Opzione | Significato |
|---|---|
| `--frame N`, `--x X`, `--y Y` | Obbligatori: fotogramma e coordinate del canvas. |
| `--tol PX` | Tolleranza di selezione in pixel del canvas (predefinito 3). |
| `--root` | Riporta l'antenato di primo livello (`mouth`) invece del livello più interno (`mouth/closed`). |
| `--bones` | Considera anche le ossa. |
| `--points a,b` | Livelli i cui punti di controllo (ancore/maniglie/vertici della mesh) sono selezionabili. |
| `--locked a,b` | Livelli da saltare (trattati come bloccati). |
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
**Attenzione:** il livello di sfondo (`sky`) è un normale bersaglio di selezione, quindi c'è quasi sempre almeno un risultato.

### `select-box`

**Sinossi:** `osatura select-box <project.json> --frame N (--rect x0,y0,x1,y1 | --lasso x,y;x,y;x,y) [--mode bbox|contain|intersect] [--root] [--bones] [--json]`

Selezione a riquadro/lazo. `--mode` decide quando un livello conta come interno: `bbox` (il suo riquadro di delimitazione tocca la regione), `contain` (completamente all'interno), `intersect` (predefinito: la sua geometria disegnata interseca la regione). `--root` riporta gli antenati di primo livello, `--bones` include le ossa.
Metti tra virgolette un valore di `--lasso` in una shell (il `;` separa i punti).

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

**Sinossi:** `osatura timeline <project.json> [--json]`

La vista dope sheet in forma di testo: tracce per canale raggruppate per osso/livello, con i fotogrammi chiave (`keys @ ...`), i canali `expr` / `driven`, i blocchi e i colori. I numeri `#N` sono gli indici dei canali usati da `keyedit --channels` e dalle selezioni `#N@F0..F1`.

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
`--json` fornisce i gruppi con id, genitori, figli, tracce, `frames`, `locked`.

### `timecode`

**Sinossi:** `osatura timecode <frame|HH:MM:SS;FF> [--fps 24] [--drop] [--nodrop]`

Conversione del timecode SMPTE. Un numero semplice è un fotogramma e stampa un timecode; un argomento contenente `:` o `;` viene analizzato e stampa il numero del fotogramma.
A 29.97, 59.94 e 119.88 fps si usa per impostazione predefinita il timecode drop-frame (`;` prima dei fotogrammi); `--nodrop` conta invece 30 (o 60/120); `--drop` forza il drop-frame (errore per le altre frequenze).
`--drop` e `--nodrop` si escludono a vicenda.

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
Non legge un progetto; `--fps` vale 24 per impostazione predefinita (non gli fps di un progetto).

### `snapshot`

**Sinossi:**
```
osatura snapshot save <project.json> <dir> [--keep 20]
osatura snapshot list <project.json> <dir>
osatura snapshot restore <project.json> <dir> <out.json> [--number N]
osatura snapshot recover <project.json> <dir> <out.json>
```
`--keep` appartiene solo a `save` e `--number` solo a `restore`; qualunque altra combinazione viene rifiutata (`unknown option --keep for 'snapshot list' (applies to: snapshot save)`), così come un argomento file mancante o in più.

Una cronologia numerata di un progetto in una directory (indipendente da `vcs`). `save` memorizza il progetto corrente come `rev-NNNNNN-<hash>.json` e conserva solo le `--keep` revisioni più recenti (predefinito 20, i file più vecchi vengono eliminati); salvare un progetto identico all'ultima revisione stampa `unchanged since the last revision` e non memorizza nulla. `list` stampa le revisioni. `restore` scrive la revisione `--number N` in `<out.json>`; `recover` scrive la revisione più recente.

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
`list` su una directory mancante non stampa nulla e termina con 0. **Correlati:** `vcs` per rami e merge.

---

## 4. Rendering ed esportazione

### `render`

**Sinossi:** `osatura render <project.json> <frame> <out.svg|out.png> [--bones] [--scale N] [--clamp] [--proxy N]`

`--proxy N` renderizza un proxy di scrubbing (N = 1..4): un `.png` è una bitmap a risoluzione 1/2^N, un `.svg` è composto da vettori semplificati; entrambi scartano tratti sottili, gradienti e fusioni.

Renderizza un fotogramma. Un `<frame>` uguale o superiore alla lunghezza del progetto è un errore (`frame 99 is outside the animation ... valid frames are 0..N-1 ... pass --clamp`); `--clamp` renderizza invece l'ultimo fotogramma e stampa un avviso su stderr. (Solo `render` ha questo controllo; `dof`, `render-hdr` e `fx` no.) L'estensione sceglie il formato (`.svg` vettoriale, `.png` raster RGBA a 8 bit); qualunque altra estensione viene rifiutata **prima** che venga scritto qualcosa.

| Opzione | Predefinito | Significato |
|---|---|---|
| `--bones` | off | Disegna la sovrapposizione dello scheletro (teste come punti, ossa come linee). |
| `--scale N` | 1 | Scala dell'output, un numero in (0, 16]. Per `.png` moltiplica la dimensione in pixel; per `.svg` il documento mantiene il proprio viewBox. |
| `--proxy N` | off | Proxy di scrubbing, N = 1..4: `.png` = bitmap a risoluzione 1/2^N, `.svg` = vettori semplificati; entrambi scartano tratti sottili, gradienti e modalità di fusione. Pensato per lo scrubbing interattivo, non per i prodotti finali. |

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
**Errori:** progetto mancante/non valido, posa non finita a quel fotogramma, output uguale al progetto di input. **Correlati:** `frames`, `render-hdr`, `onion`, `compare`, `fx`.
**Limiti:** il rasterizzatore PNG è quello del motore (8 bit, sRGB; la fusione non è gestita a colori, a meno che il progetto non lo richieda con `color-space ... linear`). Effetti dei livelli, maschere e modalità di fusione sono disegnati da entrambi.

### `frames`

**Sinossi:** `osatura frames <project.json> <out_dir> [--png] [--bones] [--scale N]`

Renderizza **ogni** fotogramma in una directory (creata se manca) come `frame_0000.svg` ... (oppure `.png` con `--png`). Prima si verifica che tutti i fotogrammi abbiano pose finite.

```
$ osatura frames demo.json fr --png
wrote 48 frames to fr
$ ls fr | head -2
frame_0000.png
frame_0001.png
```
**Attenzione:** i file esistenti con gli stessi nomi vengono sovrascritti; i file estranei nella directory restano intatti. Usalo come input per `ffmpeg` o `import-seq`.

### `export`

**Sinossi:**
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
(una forma per formato; l'estensione dell'output, senza distinzione tra maiuscole e minuscole, sceglie la forma. Un'opzione che la forma non elenca viene rifiutata: `unknown option --scale for 'export .json' (applies to: export .gif, export .mp4, ...)`. Con un'estensione sconosciuta o mancante viene segnalato invece l'errore sul formato.)

L'estensione dell'output seleziona il formato:

| Estensione | Risultato | Richiede ffmpeg | Opzioni applicabili |
|---|---|---|---|
| `.gif` | GIF in loop alla frequenza di fotogrammi del progetto (vedi [EXPORT_FORMATS.md](EXPORT_FORMATS.md)) | no | `--scale`, `--report` |
| `.html`, `.htm` | un unico player HTML autonomo (fotogrammi SVG inline, play/pausa, scrubber, audio incorporato opzionale) | solo per l'audio | - |
| `.json` | Lottie (Bodymovin) | no | `--report` |
| `.lottie` | contenitore dotLottie (zip: `manifest.json` + `a/animation.json`) | no | `--report` |
| `.mp4` | H.264 yuv420p (+ AAC quando il progetto ha suoni) | sì (`libx264`, `aac`) | `--scale` |
| `.apng` | PNG animato | no | `--scale` |
| `.webm` | VP9 (predefinito) o AV1 con alfa (+ suono Opus) | sì (`libvpx-vp9` / `libaom-av1`, `libopus`) | `--scale`, `--codec vp9\|av1` |
| `.png` | sprite sheet **più** i metadati `<out>.atlas.json` (mai `<out>.json`) | no | `--scale`, `--columns N` |
| `.glb` | glTF 2.0 binario (ossa, animazione, riempimenti/tratti con skin, gradienti, maschere, immagini, morph target) | no | `--report` |

Opzioni:

| Opzione | Significato |
|---|---|
| `--scale N` | Scala raster (0, 16] per `.gif .mp4 .apng .webm .png`. Per gli altri formati viene rifiutata (`unknown option --scale for 'export .json'`); prima della wave 7 era un avviso. |
| `--report FILE` | Scrive come JSON l'elenco delle cose che il formato non ha potuto trasportare (`.gif .json .lottie .glb`): `{"format","count","warnings":[{"code","layer","message"}]}`. Gli stessi avvisi vengono sempre stampati su stderr. Il file non può essere il progetto di input. |
| `--codec vp9\|av1` | Solo per `.webm` (predefinito `vp9`); qualunque altro valore: `unknown --codec 'foo' (vp9 or av1)`. |
| `--columns N` | Solo per lo sprite sheet `.png`: tessere per riga (predefinito: una griglia quasi quadrata, ceil(sqrt(frames)): 3 colonne per 8 fotogrammi). |

Esempi (tutti verificati su una copia di 8 fotogrammi della demo, `--scale 0.5`):

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

Errori e rifiuti:

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
Altri rifiuti: `ffmpeg failed` (encoder mancante), `export would write about N million values ...` (la protezione sulle dimensioni di Lottie/HTML/GLB: il limite è 40 milioni di valori = fotogrammi x (ancore + ossa + livelli); accorcia l'animazione o semplifica i tracciati), pose non finite su un qualsiasi fotogramma e `image too large for GIF`.

**Output / effetti collaterali:** `.png` scrive anche i metadati dello sprite sheet in `<out-name>.atlas.json` (per esempio `out.png` -> `out.atlas.json`), quindi non può più sostituire un file Lottie `out.json`; il sidecar è soggetto alle stesse protezioni su input/collegamenti simbolici. `.mp4`/`.webm` renderizzano i fotogrammi PNG in una directory temporanea privata che viene rimossa al termine.
**Suono:** `.mp4` e `.webm` mixano le tracce sonore del progetto (inizio, guadagno, taglio, dissolvenze; tagliate alla lunghezza del progetto); `.html` incorpora un mp3 mixato (avviso sopra 3 MiB, scartato sopra 16 MiB con un avviso); `.gif` e `.apng` stampano `note: GIF cannot carry audio; ...` e restano muti; Lottie avvisa con `[audio]`.
**Trasparenza:** un progetto con `"background": null` viene renderizzato con pixel trasparenti ovunque non sia disegnato nulla. `.apng`, lo sprite sheet `.png` e `.webm` (`yuva420p`, tag del contenitore `alpha_mode=1`) mantengono il canale alfa; `.gif` mantiene la trasparenza a 1 bit (verificato: il flag del colore trasparente è impostato); `.mp4` non ha alfa. Un progetto con un colore di sfondo (la demo ha `#f6f1e7`) è opaco.
**Correlati:** `export-fit` (rientrare in un budget di byte), `size-report`, `lod`, `dof-export`, `export-oca`, `render-hdr`. Dettagli completi sui formati: [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

### `export-oca`

**Sinossi:** `osatura export-oca <project.json> <out_dir> [--layers]`

Scrive una cartella **Open Cel Animation**: `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) più cel PNG `layerN/fNNNN.png`. Senza `--layers` c'è un solo livello `render` che contiene i fotogrammi composti; con `--layers` ogni livello **di primo livello** viene renderizzato da solo su trasparenza nel proprio livello OCA (11 livelli per la demo). `--scale N` moltiplica la dimensione delle cel.

```
$ osatura export-oca small.json oca2 --layers
wrote oca2/small.oca
$ ls oca2
layer0 layer1 ... layer10 small.oca
```
**Limiti:** le cel sono raster (non sopravvivono vettori, ossa o canali); opacità e fusione dei livelli non vengono scritte (ogni livello è `paintlayer`, opacità 1); i suoni non vengono esportati; la cartella viene creata se necessario e i file al suo interno vengono sovrascritti.

### `render-hdr`

**Sinossi:** `osatura render-hdr <project.json> <frame> <out.exr|out.png> [linear] [--precision half|float] [--float32] [--8bit] [--scale N] [--clamp]`

Render in virgola mobile (32 bit) (un `<frame>` fuori dal progetto è un errore, `--clamp` prende il fotogramma valido più vicino, come per `render`): `.exr` scrive un **OpenEXR** a parte singola, per scanline, non compresso (canali HALF per impostazione predefinita, FLOAT con `--precision float` o `--float32`), in luce lineare e premoltiplicato; `.png` scrive un **PNG RGBA a 16 bit**, codificato sRGB a meno che come quinto argomento non sia indicata la parola letterale `linear`. Lo sfondo del progetto **non** viene disegnato (l'immagine è trasparente dove non è disegnato nulla). `--8bit` usa come sorgente il rasterizzatore a 8 bit legacy (non può essere combinato con un output float32). `--scale N` si applica.

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
`ffprobe` legge l'EXR come `exr 640x480 gbrapf32le`. Per `.exr` la parola `linear` non ha effetto (EXR è sempre lineare). **Limiti:** nessuna compressione, un fotogramma per file, nessun livello/AOV; la grafica sorgente è in normale colore sRGB, quindi il file contiene valori nell'intervallo di visualizzazione a maggiore precisione, non luce HDR riferita alla scena. Nessuna gestione del colore oltre alla conversione sRGB<->lineare.

### `render-gpu`

**Sinossi:** `osatura render-gpu <project.json> <out.png> [--frame N] [--adapter software|auto] [--scale S] [--bench N]`

Render GPU (wgpu) di un fotogramma. Disponibile solo nelle build realizzate con `--features gpu`; la build predefinita risponde:

```
$ osatura render-gpu small.json g.png
render-gpu needs a build with the gpu feature: cargo build -p osatura-cli --features gpu       [exit 1]
```
In una build GPU le scene non supportate (immagini, ossa) vengono segnalate come errori, mai disegnate in modo errato. `--bench N` è un'opzione di benchmark. Questo comando non ha potuto essere eseguito nella build di test (senza la feature gpu), quindi i suoi flag derivano solo dal testo d'uso.

### `compare`

**Sinossi:** `osatura compare <a.json> <b.json> <frame> <out.png> [--gain 4 --scale N]`

Scrive un PNG: **prima | dopo | differenza** affiancati (il terzo pannello è una mappa di calore, nero = identico, rosso -> giallo = cambiamento maggiore, moltiplicato per `--gain`, predefinito 4). Entrambi i progetti devono essere renderizzati alla stessa dimensione (`frame sizes differ (...); compare projects of the same size`).

```
$ osatura compare small.json d2.json 3 cmp.png --gain 4 --scale 0.25
wrote cmp.png
```
**Correlati:** `diff` (struttura), `ci diff` (due PNG, metriche percettive), `golden`.

### `proxy-bench`

**Sinossi:** `osatura proxy-bench <project.json> [--frames N]`

Millisecondi per fotogramma, thread singolo, a qualità piena e a ogni livello di proxy di scrubbing (vedi `render --proxy`).

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
(valori di una build debug). 

---

## 5. Rigging, skin, IK e strumenti di layout

Tutti i comandi di questa sezione seguono lo schema `osatura <cmd> <project.json> <out.json> ...`: leggono un progetto, lo modificano, lo **rivalidano**
e scrivono `<out.json>` (che può coincidere con l'input). In caso di errore non viene scritto nulla.

### `autorig`

**Sinossi:** `osatura autorig <project.json> <out.json> [seed]`

Aggiunge automazioni standard dedotte dai **nomi** di ossa/livelli: catene IK (mix 0, così l'FK continua a funzionare), un loop di respirazione (richiede un osso torso) e l'ammiccamento (richiede livelli degli occhi). Stampa una riga per decisione: `+` aggiunto, `-` saltato con il motivo. Esegue anche `eye-clip`.

```
$ osatura autorig small.json auto.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
- arm_r: upper/lower bone not found
- leg_l: already has an IK chain
- breathing: no torso bone
- blink: no eye layers
wrote auto.json
```
`seed` (posizionale opzionale) inizializza la temporizzazione casuale dell'ammiccamento. **Limiti:** si basa solo sui nomi (`tag` mostra come vengono letti i nomi); non crea mai ossa.

### `slider`

**Sinossi:** `osatura slider <project.json> <out.json> <name> <poses>` dove `poses` è `at:target.prop=val,target.prop=val;at:...`

Crea un **osso controller** `ctrl_<name>` il cui `offset_x` pilota diverse proprietà tra pose memorizzate (uno slider di pose). Anima il `bone_offset_x` del controller stampato.

```
$ osatura slider small.json sl.json pose "0:arm_up.bone_angle=0;24:arm_up.bone_angle=30"
controller bone: ctrl_pose (animate its offset_x)
wrote sl.json
```
`target.prop` usa i nomi di proprietà in snake_case (`arm_up.angle` fallisce con `unknown property 'angle'`). L'ultimo punto separa il target dalla proprietà, quindi funzionano percorsi di livelli come `grp/child.layer_opacity`.

### `view-swap`

**Sinossi:** `osatura view-swap <project.json> <out.json> <switch_layer> <bone.prop> <from:child,from:child,...>`

Pilota un livello switch da una proprietà di un osso con una funzione a gradini: le coppie `from:child` sono ordinate per `from`, e lo switch mostra il figlio dell'ultima coppia il cui `from` non supera il valore del driver (`-40:0,0:1,40:2` = figlio 0 sotto 0, figlio 1 da 0, figlio 2 da 40). Usalo per le viste di rotazione (fronte/lato/retro) pilotate da un osso di imbardata. Gli indici dei figli devono esistere (`child index 5 is out of range (the layer has 3)`); il livello deve essere uno switch.

```
$ osatura view-swap small.json vs.json mouth foot_ctrl.bone_offset_x "-40:0,0:1,40:2"
wrote vs.json
```

### `align` e `distribute`

**Sinossi:**
`osatura align <project.json> <out.json> <mode> <layer,layer,...|layer> [stage]`  (mode: `left right top bottom center-x center-y`)
`osatura distribute <project.json> <out.json> <x|y> <gaps|centers> <layer,layer,...>`

`align` sposta i livelli in modo che i riquadri di delimitazione delle loro curve appiattite si allineino (tra loro oppure, per un singolo livello seguito dalla parola `stage`, al canvas). `distribute` spazia in modo uniforme tre o più livelli lungo un asse, rendendo uguali o gli **spazi** tra i riquadri o i **centri**.

```
$ osatura align small.json al.json center-x sun,hills stage
wrote al.json
$ osatura align small.json al2.json top ground
select at least two layers (or one with --stage)                 [exit 1]
$ osatura distribute small.json di.json x gaps sun,ground,hills
wrote di.json
```
Questi comandi modificano le posizioni dei livelli (trasformazione statica), quindi i canali `layer_x`/`layer_y` con chiavi li sovrascrivono comunque. (Il messaggio di errore cita `--stage`; l'argomento effettivo è la sola parola `stage`.)

### `trim`

**Sinossi:** `osatura trim <project.json> <out.json> <layer> <path_index> <start> <end> [offset]`

Esegue il bake di un *trim path*: sostituisce il tracciato numero `path_index` di un livello vettoriale con la sola parte compresa tra `start` ed `end` (frazioni 0..1 della sua lunghezza, entrambe spostate di `offset`; i tracciati chiusi si avvolgono). Il risultato è una **polilinea aperta con lo stesso tratto e senza riempimento**: è distruttivo, quindi conserva l'originale. Per un effetto di disegno progressivo, esegui il bake di una copia tagliata per fotogramma.

```
$ osatura trim small.json tr.json ink 0 0 0.5 0.25
wrote tr.json
```

### `reparent`

**Sinossi:** `osatura reparent <project.json> <out.json> <bone> <new_parent|none>`

Cambia il genitore di un osso **senza spostarlo**: offset e angolo di riposo vengono ricalcolati in modo che la posa di riposo resti invariata (verificato: dopo differiscono solo `parent`, `rest_angle`, `rest_offset`). `none` lo rende una radice. Un genitore che creerebbe un ciclo viene rifiutato.

```
$ osatura reparent small.json rp.json hand none
wrote rp.json
$ osatura diff small.json rp.json
~ bone 'hand' changed (parent, rest_angle, rest_offset)
```

### `parent-switch`

**Sinossi:** `osatura parent-switch <project.json> <out.json> <bone> <frame:parent|none,...>`

Esegue il bake di un cambio di genitore animato (una mano che afferra un oggetto): a ogni fotogramma di cambio viene catturata e mantenuta la posa world dell'osso rispetto al nuovo genitore (inseguimento rigido), oppure congelata nello spazio world per `none`. L'osso diventa una radice e la sua posa world viene scritta come semplici chiavi di offset/angolo per ogni fotogramma; i suoi vecchi canali restano nel file ma sono sovrascritti da quelli calcolati. **Limiti:** rifiutato per le ossa coinvolte in IK, vincoli o ossa con molle; la scala non viene trasportata.

```
$ osatura parent-switch demo.json ps.json hand 0:none,24:arm_up
wrote ps.json
```

### `skin`

**Sinossi (sottocomandi):**
```
osatura skin <project.json> <out.json> add <group> <name> <layer,layer,...>
osatura skin <project.json> <out.json> rig <group> <name> <constraint,...|-> <ik,...|->
osatura skin <project.json> <out.json> bones <group> <name> <bone,...|->
osatura skin <project.json> <out.json> apply <group> <name>
osatura skin <project.json> <out.json> schedule <group> <frame:name,frame:name,...>
```

Un *insieme di skin* è identificato da un'etichetta `<group>` (un insieme di "costumi" o di "espressioni"; nell'esempio coincide con il nome di un gruppo di livelli). Ogni skin con nome mostra un insieme di livelli, indicati per **percorso del livello** (es. `skins/box`); applicare una skin mostra i suoi livelli e nasconde i livelli di ogni altra skin dello stesso gruppo.
`add` definisce una skin; `rig` elenca vincoli e catene IK attivi solo mentre quella skin lo è; `bones` elenca le ossa che esistono solo mentre è attiva (altrimenti ridotte a scala 0); `apply` rende attiva una skin in modo statico; `schedule` imposta nel tempo la skin attiva come chiavi a gradino sull'**opacità del livello** (quindi non animare altrimenti l'opacità di quei livelli). `-` significa "nessuno".

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
**Limiti (ROADMAP §9):** un osso che esiste in una sola skin non viene mai rimosso dal rig valutato (gli indici restano; viene portato a scala 0); un livello pesato su un osso presente solo in una skin ma non appartenente a quella skin scompare con essa, e questo non viene verificato.

### `bone-vis`

**Sinossi:** `osatura bone-vis <project.json> <out.json> <bone> <from> <to>|clear`

Mostra l'osso solo negli **overlay** dello studio per un intervallo di fotogrammi (controllo del disordine visivo). Non cambia il rendering/l'esportazione. `clear` rimuove l'intervallo (`range removed`).

### `chan-meta`

**Sinossi:** `osatura chan-meta <project.json> <out.json> <target> <prop|*> [color=#rrggbb] [shy=on|off] [solo=on|off] [lock=on|off]`

Metadati dope sheet di un canale: colore di visualizzazione, *shy* (nascosto negli elenchi), *solo* (anteprima solo di questi canali nello studio) e **lock** (rifiuta le modifiche: i canali bloccati sono saltati da `keys`, rifiutati da `keyedit`, `audio-react` e `live-map`, e mai toccati da `clip bake`). Valutazione ed esportazione ignorano tutti e quattro. `*` seleziona ogni proprietà del target.

```
$ osatura chan-meta small.json cm.json arm_up bone_angle 'color=#ff0000' shy=on lock=on
wrote cm.json
```

### `reference`

**Sinossi:** `osatura reference <project.json> <out.json> <layer> <on|off>`

Contrassegna un livello come **riferimento di tracciamento**: mostrato nello studio, mai esportato.

### `parallax`

**Sinossi:** `osatura parallax <project.json> <out.json> <far_layer,...,near_layer> <far_value> <near_value>`

Assegna una profondità multipiano (valori `parallax` distribuiti tra `far_value` e `near_value`) ai livelli di primo livello elencati, nell'ordine indicato (il primo = il più lontano). Con il movimento della camera i livelli si spostano di quantità diverse; `dof` sfoca in base a `|parallax - focus|`.

```
$ osatura parallax small.json pa.json sky,hills,ground 0.2 1.0
wrote pa.json
```

### `eye-clip`

**Sinossi:** `osatura eye-clip <project.json> <out.json>`

Maschera ogni livello di iride/pupilla con il livello della forma dell'occhio dallo stesso lato (trovato per nome, vedi `tag`) usando una normale maschera di livello, così le pupille non escono mai dall'occhio mentre si muovono. Eseguito anche da `autorig`. Quando non si trovano nomi corrispondenti il file viene comunque scritto, invariato.

### `comp`

**Sinossi:**
```
osatura comp capture|apply <project.json> <out.json> <name>
osatura comp list <project.json>
```

Stati di livello con nome ("comp": visibilità, opacità e figlio selezionato dei livelli switch, per percorso di livello; **non** le trasformazioni). `capture` memorizza lo stato corrente come `name`, `apply` lo ripristina, `list` stampa i nomi (`list` non prende alcun file di output e non scrive nulla; una quarta parola legacy, `comp list p.json p.json`, è ancora accettata e ignorata).

```
$ osatura comp capture small.json c1.json pose1
wrote c1.json
$ osatura comp list c1.json c1.json
pose1 (14 layers)
$ osatura comp apply c1.json c2.json pose1
wrote c2.json
```

### `retarget`

**Sinossi:** `osatura retarget <source.json> <dest.json> <out.json> --pairs srcChain=dstChain[,..]`

Esegue il retarget per *intento*: l'animazione dell'**effettore** IK (osso target) di ogni catena sorgente viene scalata in base al rapporto tra le portate dei due arti (somma delle lunghezze delle ossa) e ripuntata sull'osso target della catena di destinazione. I canali dell'effettore senza chiavi non possono essere scalati; vengono passati rinominati e segnalati come righe `note:`. `--pairs srcChain=dstChain[,...]` è obbligatorio. Nell'esempio sorgente e destinazione sono lo stesso progetto, il che serve solo come smoke test.

```
$ osatura retarget demo.json demo.json rt.json --pairs leg=leg
wrote rt.json (+14 channels)
```
**Correlati:** `mocap-retarget` per i dati BVH; `walk`, `mirror`.

---

## 6. Animazione, chiavi, timeline e generatori

### `keys`

**Sinossi:**
```
osatura keys shift <project.json> <out.json> --by N [--from F --to F --target NAME --prop PROP]
osatura keys scale <project.json> <out.json> --factor X [--pivot F --from F --to F --target NAME --prop PROP]
osatura keys ease <project.json> <out.json> --ease linear|step|ease_in|ease_out|ease_in_out|smooth [--from F --to F --target NAME --prop PROP]
osatura keys reverse <project.json> <out.json> [--from F --to F --target NAME --prop PROP]
```
Ogni operazione accetta solo le proprie opzioni (`keys shift --factor 2` viene rifiutato); `--by`, `--factor` e `--ease` sono obbligatori per la rispettiva operazione (un `--by` o `--factor` omesso era in passato un'operazione nulla e silenziosa).

Operazioni in blocco sulle chiavi di **tutti i canali con chiavi** (filtrati da `--target` / `--prop`; un intervallo `--from/--to` limita le chiavi toccate). I canali bloccati vengono saltati con `skipped locked channel ...`.

| Operazione | Richiede | Effetto |
|---|---|---|
| `shift` | `--by N` | Sposta le chiavi di N fotogrammi. |
| `scale` | `--pivot F --factor X` | Scala i tempi delle chiavi rispetto al fotogramma pivot. |
| `ease` | `--ease E` | Imposta l'easing delle chiavi (easing con nome; usa `keyedit ease` per `bezier:`). |
| `reverse` | - | Inverte l'ordine delle chiavi all'interno dell'intervallo. |

```
$ osatura keys shift small.json k1.json --target arm_up --prop bone_angle --by 2
wrote k1.json (5 key(s) changed)
$ osatura keys scale small.json k2.json --pivot 0 --factor 0.5 --from 0 --to 48
wrote k2.json (37 key(s) changed)
$ osatura keys reverse small.json k4.json
wrote k4.json (37 key(s) changed)
```
Le chiavi spostate fuori da `0..frames-1` restano nel file (lint segnala `L007`); questo comando non estende la lunghezza del progetto. **Correlati:** `keyedit` (preciso, basato sulla selezione), `loop-solve`.

### `keyedit`

**Sinossi:** `osatura keyedit <project.json> <out.json|-> <sub-command> [options]`

Modifiche del dope sheet come **un unico batch sicuro per l'annullamento**. Con `-` come output non viene scritto nulla e l'operazione pianificata viene stampata come JSON (prova a vuoto). Sotto-comandi e opzioni:

```
osatura keyedit <project.json> <out.json|-> move --sel SEL [--dt N] [--dv N] [--collision replace|merge|refuse] [--no-snap] [--free]
osatura keyedit <project.json> <out.json|-> scale --sel SEL --pivot F --factor X | dup --sel SEL --offset N | delete --sel SEL | ease --sel SEL --ease E | value --sel SEL --value V | smooth --sel SEL [--kind monotone|catmull_rom]
osatura keyedit <project.json> <out.json|-> copy --sel SEL --clip clip.json | paste --clip clip.json --at F [--remap from_target:prop=to_target:prop]
osatura keyedit <project.json> <out.json|-> add --target T --prop P --at F [--value V] [--ease E] | ripple --at F --by N --mode insert|delete [--scope 0,1] [--adjust-frames]
osatura keyedit <project.json> <out.json|-> offset --channels 0,1 --dt N | extrapolate --channels 0,1 --extrapolate loop | loop --channels 0,1 [--end F] | handle --sel SEL --side in|out --t F --v V [--mode free|aligned|auto] | lock --target T [--prop P] [--value on|off]
```

* **SEL** è composto da elementi `TARGET:PROP[@F0..F1]` uniti da `;`, oppure `#CHANNEL[@F0..F1]` (indice del canale da `timeline`), oppure `all`. Mettilo tra virgolette (`;`, `@`, `#`).
* **E** (easing) è `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`, `smooth`, `bounce`, `bezier:x1,y1,x2,y2` oppure un oggetto JSON.
* `--collision` decide che cosa succede quando una chiave spostata cade su una chiave esistente (`replace` predefinito, `merge`, `refuse`); `--no-snap` mantiene i fotogrammi frazionari; `--free` rimuove la restrizione ai fotogrammi interi.
* Riga di output: `<op>: N keys changed, wrote <out>`. `copy` con un output reale stampa `copy: 0 keys changed, project unchanged, clip saved`.
* Le operazioni che porterebbero una chiave fuori da `0..=frames` vengono rifiutate con un messaggio preciso (`duplicate: frame 52 is outside the project range 0..=48`), a meno che `ripple --adjust-frames` non estenda il progetto.

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
**Attenzione:** `delete` con `#0@12..12` rimuove esattamente una chiave; `ripple --mode delete --scope 0,1` limita lo spostamento ai canali 0 e 1; `copy` e `paste` si scambiano un file di clip, non gli appunti di sistema. I canali bloccati rifiutano `move/scale/...` (usa prima `lock --value off`).

### `marker`

**Sinossi:** `osatura marker <project.json> <out.json> <name> <frame> [--end F]`

Aggiunge un marker alla timeline (un punto con nome oppure, con `--end`, un intervallo). Un marker con lo stesso nome viene sostituito. I marker sono ordinati per fotogramma.

```
$ osatura marker small.json m1.json hit 4 --end 6
wrote m1.json (marker 'hit' at 4 = 00:00:00:04)
```

### `event`

**Sinossi:** `osatura event <project.json> <out.json> <name> <frame> [--data JSON]`

Aggiunge un evento con nome (un trigger di runtime per i player) a un fotogramma, con payload JSON opzionale. JSON non valido: `--data is not valid JSON: ...`.

```
$ osatura event small.json ev1.json boom 3 --data '{"power":2}'
wrote ev1.json (1 events)
```

### `idle`

**Sinossi:**
```
osatura idle blink <project.json> <out.json> --target T --prop P [--interval S --jitter X --closed F --seed N]
osatura idle breath <project.json> <out.json> --target T --prop P [--amp A --period F]
osatura idle sway <project.json> <out.json> --target T --prop P [--amp A --freq HZ --seed N]
```
Ogni generatore accetta solo le proprie opzioni (`idle breath --seed 2` viene rifiutato).

Genera un canale idle procedurale e lo **accoda** (i canali esistenti restano).

| Generatore | Opzioni (valori predefiniti) | Target tipico |
|---|---|---|
| `blink` | `--interval` secondi medi tra gli ammiccamenti, `--jitter`, `--closed` fotogrammi a occhio chiuso, `--seed` | un `layer_switch` (occhio aperto/chiuso) o `layer_scale_y` |
| `breath` | `--amp` (0.03), `--period` fotogrammi (3 s agli fps del progetto) | `bone_scale` di un torso |
| `sway` | `--amp` (2), `--freq` Hz (0.3), `--seed` (1) | `bone_angle` |

```
$ osatura idle blink small.json i1.json --target mouth --prop layer_switch --interval 1 --seed 3
wrote i1.json
$ osatura idle sway small.json i3.json --target arm_up --prop bone_angle --amp 3 --freq 0.5
wrote i3.json
```
`--target` e `--prop` sono obbligatori (`--target is required`).

### `follow` (due forme)

**Follow-through (opzioni):** `osatura follow <project.json> <out.json> --target SRC --prop P --chain a,b,c [--delay F --decay X]`
Aggiunge copie ritardate e smorzate di un canale esistente (`--target`/`--prop` indicano la sorgente) alle ossa della catena: azione sovrapposta / follow-through (`--delay` fotogrammi per anello, predefinito 3; `--decay` per anello, predefinito 0.7). Una copia ritardata viene tagliata all'ultimo fotogramma del progetto (nessuna chiave viene scritta dopo il fotogramma `frames`, quindi `lint` L007 non scatta); i fotogrammi tra l'ultima chiave mantenuta e il taglio vengono campionati in modo che la curva ritardata sia esatta a ogni fotogramma intero.

```
$ osatura follow small.json f1.json --target arm_up --prop bone_angle --chain arm_low,hand --delay 2 --decay 0.6
wrote f1.json (+2 channels)
```

**Path following (posizionale):** `osatura follow <project.json> <out.json> <layer|bone:NAME> <layer_with_path> <start_frame> <end_frame> [no-orient]`
Esegue il bake di un livello (o dell'osso radice `bone:NAME`) che percorre il primo tracciato di un altro livello tra due fotogrammi, ruotando secondo la tangente del tracciato a meno che non sia indicato `no-orient`.

```
$ osatura follow small.json fp.json target ink 0 7
wrote fp.json
$ osatura follow small.json fp2.json bone:hand ink 0 7 no-orient
path following needs a root bone (no parent)                      [exit 1]
```

### `take-record`, `take-apply`, `take-list`

**Sinossi:**
```
osatura take-record <project.json> <library.json> <name> <from> <to> [target_prefix,...]
osatura take-apply  <project.json> <out.json> <library.json> <name> <at> [from=to,...] [speed] [--blend FRAMES]
osatura take-list   <library.json>
```
Una **take** è una porzione riutilizzabile di animazione memorizzata in un file di libreria. `take-record` cattura i fotogrammi `from..to` dei canali il cui target inizia con uno dei prefissi (tutti i canali se omessi); la libreria viene creata se manca e una take con lo stesso nome viene sostituita. `take-apply` incolla la take al fotogramma `at`, con l'opzione di **retargeting** (le coppie `from=to` rinominano i target) e di cambiare `speed`; gli argomenti sono controllati in modo rigoroso: una voce senza `=`, un nome sorgente per cui la take non ha traccia o una velocità che non sia un numero positivo è un errore, e una libreria illeggibile indica il file (`cannot read the take library ...`); `--blend N` fa una dissolvenza incrociata con l'animazione esistente per N fotogrammi a entrambe le estremità.

```
$ osatura take-record demo.json lib.json wave 0 24 arm_
recorded 2 tracks into lib.json
$ osatura take-list lib.json
wave (24 frames, 2 tracks)
$ osatura take-apply demo.json ta.json lib.json wave 10 arm_up=arm_low 1.5 --blend 4
applied 2 tracks; wrote ta.json
```

### `clip`

**Sinossi:**
```
osatura clip add <project.json> <out.json> <library.json> <take> <clip> [--at F --speed S --loops N --mode replace|additive|multiply --weight W
                   --include a_*,b --exclude c --track-weight arm_*=1,leg_*=0.2 --rename a=b --fade-in F --fade-out F]
osatura clip list <project.json>
osatura clip remove <project.json> <out.json> <clip>
osatura clip set-weight <project.json> <out.json> <clip> [W] [--keys frame:w,frame:w] [--track PATTERN[:prop]]
osatura clip bake <project.json> <out.json>
```
Una **pila di clip** a livelli (animazione non lineare): i clip fanno riferimento alle take di una libreria, vengono riprodotti a `--at` con `--speed`/`--loops` e si fondono in modalità `replace`, `additive` o `multiply` con un peso (costante, con chiavi nel tempo o per pattern di traccia). `clip add/set-weight/remove` modificano soltanto la pila; **`clip bake` rigenera i canali di proprietà della pila** (i canali creati a mano e quelli bloccati non vengono mai toccati).

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
(L'esempio di `clip list` mostra il secondo `clip add` con lo stesso nome, che ha sostituito il primo.) **Limiti (ROADMAP §9):** i canali calcolati sono campionati per fotogramma intero; non esiste un `clip refresh` per riprendere una take della libreria modificata.

### `make-loop`

**Sinossi:** `osatura make-loop <project.json> <out.json> [targets|all] [props] [--inclusive]`

Fa terminare i canali `replace` con chiavi create a mano dove sono iniziati: ogni curva viene campionata per fotogramma, la retta dal suo primo al suo ultimo valore viene sottratta (rimozione della deriva) e il primo valore viene riaggiunto. Applicato all'offset X di un osso in movimento, trasforma una camminata in una camminata sul posto. Il periodo del loop è N = i `frames` del progetto, quindi riprodurre ripetutamente `0..N-1` non mostra alcun fotogramma di giunzione duplicato. `--inclusive` è la convenzione legacy in cui l'ultimo fotogramma è uguale al primo. Senza argomenti viene elaborato ogni canale con chiavi. Stampa una riga `removed drift` per ogni canale modificato.

```
$ osatura make-loop demo.json ml2.json foot_ctrl bone_offset_x
foot_ctrl BoneOffsetX: removed drift 0.79
wrote ml2.json
$ osatura make-loop demo.json ml.json all bone_angle --inclusive
```
**Limiti:** è un classico detrending lineare, non una ri-temporizzazione del movimento; i canali pilotati da espressioni non vengono toccati. **Lo spostamento viene rimosso di proposito** (un progetto riprodotto ripetutamente da `0..N-1` non può spostarsi all'infinito, quindi il loop è sul posto): un canale la cui intera variazione era spostamento diventa costante e il comando stampa per esso una riga `note: ... is now constant`; `lint` segnala poi quel canale come L005 (informazione), il che è previsto. Indica target e proprietà per lasciare intatto lo spostamento degli altri canali, oppure usa `loop-solve` per un movimento già ciclico.

### `loop-solve`

**Sinossi:** `osatura loop-solve <project.json> <out.json> [--window 24] [--inclusive]`

Risolve la **giunzione**: fonde la fine dei canali con chiavi nel loro inizio su una finestra, così l'animazione fa il loop senza scatti. Stampa l'errore di giunzione prima e dopo.

```
$ osatura loop-solve demo.json ls.json --window 24
wrote ls.json: 8 channel(s) adjusted, seam 0.000 -> 0.000
```
`--window` è in fotogrammi (predefinito 24). Confrontalo con il contratto `loop` di `lint` (`contracts`).

### `walk`

**Sinossi:** `osatura walk <project.json> <out.json> --thigh-l B --thigh-r B [--shin-l B --shin-r B --arm-l B --arm-r B --hips B --period N --stride DEG --knee DEG --bob PX --swing DEG]`

Genera un ciclo di camminata in loop come canali con chiavi sulle ossa indicate (è richiesta almeno un'opzione di osso: `name at least one bone (--thigh-l, --thigh-r, ...)`). Valori predefiniti: `--period 24` fotogrammi, `--stride 28`, `--knee 40`, `--bob 4`, `--swing 22`. I canali usano l'estrapolazione in loop e vengono **accodati**.

```
$ osatura walk small.json wk.json --thigh-l arm_up --thigh-r arm_low --period 8 --stride 20
wrote wk.json (+2 channels)
```

### `mirror`

**Sinossi:** `osatura mirror <project.json> <out.json>`

Specchia tutti i canali con chiavi rispetto all'asse verticale: i target scambiano lato (prefissi `L_`/`R_`, suffissi `_L`/`_R` e `_l`/`_r`, le parole `left`/`right`), e le rotazioni e le traslazioni orizzontali cambiano segno. I canali specchiati **sostituiscono** i canali del progetto (non vengono aggiunti accanto agli originali). La riga di risultato è `wrote X (N of M channel target(s) swapped sides)`; quando nessun target ha un nome sinistro/destro, o il progetto non ha canali, un avviso su stderr lo segnala (vengono invertiti solo i segni).

### `foot-lock`, `stretch`, `magnet`

```
osatura foot-lock <project.json> <out.json> <ctrl_bone> <ground_y> [tolerance] [min_frames]
osatura stretch   <project.json> <out.json> <follower_bone> <leader_bone> [min=1] [max=3]
osatura magnet    <project.json> <out.json> <ctrl_bone> <bone:NAME|x,y> <radius> <strength 0..1>
```

* **`foot-lock`** campiona la posizione world del controller a ogni fotogramma; le sequenze di almeno `min_frames` fotogrammi in cui è a terra (la y dello stage punta verso il basso: il piede è a terra quando `y >= ground_y - tolerance`) sono *contatti*, e un canale di correzione **additivo** blocca il controller dove ogni contatto è iniziato. L'animazione originale resta intatta; elimina i canali aggiunti per annullare.
  ```
  $ osatura foot-lock demo.json fl.json foot_ctrl 385 2 2
  contact frames 23..47 pinned at (479.2, 384.1)
  wrote fl.json
  ```
* **`stretch`** fa puntare `follower_bone` verso la testa di `leader_bone` e lo scala (in modo uniforme, tra `min` e `max`, predefiniti 1 e 3) in modo che la sua punta la raggiunga. Calcolato come canale di angolo additivo più un canale di scala moltiplicativo; i figli del follower vengono scalati con lui. Il leader non può dipendere dal follower (`the leader depends on the follower (it is one of its descendants)`).
  ```
  $ osatura stretch p.json st.json flw lead 1 2
  corrected 8 frames
  wrote st.json
  ```
* **`magnet`** calcola con il bake un'attrazione di prossimità di un controller verso la testa di un osso (seguita a ogni fotogramma) o verso un punto fisso: decadimento morbido, `strength` pieno (0..1) sul target e zero a `radius`. Calcolato come canali di offset additivi.
  ```
  $ osatura magnet p.json mg.json lead 200,100 200 0.8
  pulled on 8 frames
  wrote mg.json
  ```
**Limiti (ROADMAP §9):** tutti e tre sono risultati calcolati con campionamento per fotogramma: rieseguili dopo aver modificato l'animazione sorgente. `stretch` usa una scala uniforme. `foot-lock` e `magnet` si aspettano un osso controller radice.

### `shot`, `camera-seq`

```
osatura shot       <project.json> <out.json> <name> <x> <y> <zoom> [rotation]
osatura camera-seq <project.json> <out.json> <frame:shot[:cut],...>
```
`shot` salva un'**inquadratura di camera** con nome (centro x, y, zoom, rotazione opzionale in gradi). `camera-seq` imposta le chiavi della camera attraverso le inquadrature salvate: a ogni `frame` la camera si sposta sull'inquadratura (con easing) oppure, con il suffisso `:cut`, vi salta. Scrive i canali `camera_x/y/zoom/rotation`.

```
$ osatura shot small.json sh.json wide 320 240 1 15
wrote sh.json (1 shots)
$ osatura shot sh.json sh2.json close 400 200 2
wrote sh2.json (2 shots)
$ osatura camera-seq sh2.json cs.json 0:wide,4:close:cut
wrote cs.json
```

### `spring-fit`

**Sinossi:** `osatura spring-fit --start V --samples a,b,c,... [--fps 24]`

Adatta **frequenza (Hz) e rapporto di smorzamento** di una molla a una curva di risposta disegnata (campioni per fotogramma di un gradino dal valore `--start` all'ultimo valore). Stampa la coppia adattata e il JSON da incollare negli `springs` di un progetto. Non legge un progetto.

```
$ osatura spring-fit --start 0 --samples 0.5,0.9,1.1,1.0,0.98,1.0 --fps 24
frequency 4.68 Hz, damping 0.61 (rms error 0.028 of the step size)
add to the project: {"bone": "<bone>", "frequency": 4.68, "damping": 0.61}
```

### `script`

**Sinossi:** `osatura script <project.json> <out.json> <script.rhai>`

Esegue uno **script Rhai in sandbox** che modifica il progetto tramite richieste validate (una richiesta rifiutata lascia tutto invariato:
`request 5 (box.LayerOpacity at 0) was rejected, nothing changed: ...`). Funzioni integrate: `fps() frames() bones() layers() set_key(target, prop, frame, value[, ease]) add_bone(name, parent (empty = root), length, x, y, angle) add_rect add_circle add_polygon|add_blob add_line add_gradient_rect add_ik add_aim add_copy_rotation add_pin add_group(name[, [layers]]) add_switch move_layer set_parent delete_layer delete_bone set_layer set_bone add_channel_expr add_noise add_marker add_event bone_info layer_info world print`.
Uno script che non applica alcuna richiesta stampa `warning: script: 0 requests were applied`. Le build senza la feature `scripting` rispondono `this build has no scripting support`.

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
I livelli all'interno di un gruppo sono indirizzati per percorso (`skins/box`). `print` va su stdout.

---

## 7. Fisica, cloth e deformatori

Tutti questi comandi eseguono il **bake** di una simulazione o di un risultato geometrico in normali canali con chiavi: il progetto resta deterministico e il risultato si riproduce ovunque (inclusi Lottie/GLB dove applicabile). Riesegui il comando dopo aver modificato l'animazione sorgente.
Prendono `<project.json> <out.json>` e parole `key=value` (senza `--`).

### `physics`

**Sinossi:**
```
osatura physics <project.json> <out.json> <bone,bone,...> [gravity=900] [wind=x,y] [gust=0.5] [damping=0.985]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [follow=bone] [--allow-empty]
    [drive=bone:gain;...] [vortex=bone|x,y:swirl[:radial[:radius]];...] [preset=hair|tail|rope|antenna|cape]
    [merge=keep|replace] [lateral=0..1]
```
Esegue il bake di una **simulazione di catena** Verlet (capelli, coda, corda, antenna, mantello) sulle ossa elencate (prima la radice della catena) come chiavi `bone_angle`, una per fotogramma.

| Parola | Significato |
|---|---|
| `gravity=` | Accelerazione verso il basso (predefinito 900 unità dello stage/s^2). |
| `wind=x,y`, `gust=`, | Vettore del vento costante; ampiezza delle raffiche di un vento modulato da rumore. |
| `damping=` | Smorzamento della velocità per passo (predefinito 0.985). |
| `circle=x,y,r[@bone];...`, `capsule=x1,y1,x2,y2,r[@bone];...` | Collider, opzionalmente collegati a un osso (lo seguono). Le capsule sono accettate (verificato con `capsule=100,300,300,300,20`; una vecchia nota "capsule colliders NOT done" nella ROADMAP §9 è superata). |
| `follow=bone` | La radice della catena viene trascinata dal movimento di questo osso (inerzia). |
| `drive=bone:gain;...` | Corpi che spingono la catena (la loro velocità moltiplicata per il guadagno). |
| `vortex=bone\|x,y:swirl[:radial[:radius]];...` | Campi di vortice attorno a un osso o a un punto. |
| `preset=` | Valori iniziali per `hair tail rope antenna cape`; le parole scritte **dopo** `preset=` lo sovrascrivono. |
| `merge=keep\|replace` | `keep` (predefinito) lascia intatte le ossa che hanno già un'animazione; `replace` le sovrascrive. |
| `lateral=0..1` | Con più catene (`bone,bone/bone,bone`) lega tra loro i fili vicini. |
| `--allow-empty` | Accetta un'esecuzione che non ha calcolato nulla. |

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
(`p.json` è la demo più tre ossa senza chiavi `tail1..tail3` create con uno script Rhai - vedi `script`.) **Attenzione:** un bake che non cambia nulla è un errore, a meno che non sia indicato `--allow-empty`.
**Limiti (ROADMAP §9):** i collider ignorano la scala delle ossa (solo trasformazione rigida); i cerchi statici collidono solo nei punti (quelli collegati anche lungo i segmenti); nessuna collisione capsula-capsula né autocollisione.

### `cloth`

**Sinossi:**
```
osatura cloth <project.json> <out.json> <layer> [pin=top,left@bone,row:N,col:N,v:N,corners,none] [gravity=900]
    [wind=x,y] [gust=0.5] [gust-hz=0.5] [gust-len=px] [stiffness=1] [shear=0.05] [bend=0.2] [max-stretch=1.1]
    [damping=0.99] [cols=N rows=N] [rect=x,y,w,h] [name=cloth] [substeps=4] [iterations=10] [preroll=frames]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [from=F] [to=F]
```
Esegue il bake di una **superficie di tessuto** deterministica sulla mesh di un livello *immagine*, oppure su un livello *vettoriale* quando è indicato `rect=x,y,w,h` (il livello vettoriale riceve una mesh a griglia su quel rettangolo), come canali di deformazione per vertice. `pin=` sceglie i vertici fissati (bordi `top`, `left`..., `corners`, `row:N`, `col:N`, `v:N`, `@bone` collega i pin a un osso, `none`). `from`/`to` limitano l'intervallo di fotogrammi calcolato, `preroll` esegue la simulazione prima del fotogramma `from` in modo che il tessuto parta da fermo.

```
$ osatura cloth small.json cloth.json sun rect=100,60,120,120 cols=6 rows=6 pin=top gravity=900 wind=40,0
baked cloth on 'sun' (84 channels); wrote cloth.json
$ osatura cloth small.json cloth.json sun
a vector layer needs rect=x,y,w,h                    [exit 1]
$ osatura cloth small.json cloth.json nosuch
no layer at 'nosuch'                                 [exit 1]
```
Gli `84 channels` sono chiavi `lattice_dx` / `lattice_dy` (target `sun::cloth::<col>,<row>`) dei 42 punti non fissati della griglia 7x7 (la riga superiore è fissata). Sono canali reali nel file, quindi il bake di un cloth ingrandisce il progetto (qui 98 canali invece di 14): usa `size-report` per vederne il costo. **Limiti (ROADMAP §9):** il tessuto è planare (nessuna piega fuori dal piano, quindi per impostazione predefinita lo shear è cedevole); rifiuta le mesh libere (solo mesh a griglia); nessuna autocollisione; calcolato per fotogramma intero.

### `cage`

**Sinossi:** `osatura cage <project.json> <out.json> <layer> [rect=x,y,w,h|auto] [hull] [verts=N] [pad=P] [name=cage]`

Aggiunge un **deformatore a gabbia con coordinate a valor medio** attorno a un livello: il contenuto dentro la gabbia segue i vertici della gabbia, il contenuto fuori resta invariato. `auto` (predefinito quando manca `rect`) usa i limiti con margine, `verts=N` è il numero di vertici per lato, `hull` usa l'inviluppo convesso, `pad=` aggiunge margine al riquadro, `name=` dà un nome al deformatore (predefinito `cage`).

```
$ osatura cage small.json cg.json sun auto verts=3
added cage 'cage' (12 vertices) to 'sun'; wrote cg.json
```
**Limiti (ROADMAP §9):** nessun adattamento automatico della gabbia oltre a un riquadro con margine o all'inviluppo convesso (nessun tracciamento concavo); non c'è un decadimento morbido fuori dalla gabbia (compare un gradino visibile dove la grafica attraversa il bordo della gabbia); le mesh libere non hanno offset con chiavi per vertice (usa una gabbia).

### `cage-pose`

**Sinossi:** `osatura cage-pose <project.json> <out.json> <layer> <frame> [name=cage] [move=i:dx,dy;...] [all=dx,dy] [ease=linear]`

Imposta chiavi per gli offset dei vertici della gabbia a un fotogramma (canali `cage_dx` / `cage_dy`). `move=` sposta singoli vertici, `all=` sposta ogni vertice. I canali bloccati vengono rifiutati.

```
$ osatura cage-pose cg.json cp.json sun 4 all=5,0
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
$ osatura cage-pose cg.json cp.json sun 4 move=0:5,5
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
```

### `warp-pivot` e `soft-scale`

**Sinossi:**
```
osatura warp-pivot <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=1,4-7] [region=x,y,w,h]
    [pivot=x,y|bone:NAME|centroid|center] [rotate=DEG] [scale=S|sx,sy] [move=dx,dy] [radius=R] [falloff=smooth] [ease=linear]
osatura soft-scale <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=..|region=..] scale=S|sx,sy
    [radius=R] [falloff=smooth] [pivot=..]
```
`warp-pivot` ruota/scala/sposta una **selezione** di punti di controllo (deformatori cage, lattice, cloth, pin, spine; una mesh immagine a griglia riceve una griglia di warp per vertice) attorno a un pivot e imposta il risultato come chiave al fotogramma. La selezione è `verts=` (indici e intervalli) e/o `region=`; una selezione vuota è un errore (`the selection is empty (give verts= and/or region=)`). `radius=` aggiunge un **decadimento morbido** (`linear smooth gauss sharp constant`). `soft-scale` è una scalatura proporzionale (selezione morbida); senza `verts`/`region` l'intera mesh viene ridimensionata attorno al centro del suo riquadro di delimitazione con `scale=F`, `width=W`, `height=H` o `size=WxH`, con `aspect=keep|free`.

```
$ osatura warp-pivot cg.json wp.json sun 4 verts=0-3 rotate=15 pivot=centroid radius=30 falloff=gauss
warp-pivot on 'sun' keyed at frame 4; wrote wp.json
$ osatura soft-scale cg.json ss.json sun 4 scale=1.2 pivot=center
soft-scale on 'sun' keyed at frame 4; wrote ss.json
```

### `layer-pivot`

**Sinossi:** `osatura layer-pivot <project.json> <out.json> <layer> pivot=x,y|bone:NAME`

Sposta il pivot (ancora) di un livello **senza cambiare l'immagine**, così i canali di rotazione e scala ruotano attorno al nuovo punto.

```
$ osatura layer-pivot small.json lp.json sun pivot=100,100
moved the pivot of 'sun'; wrote lp.json
```

### `mesh-lock`

**Sinossi:** `osatura mesh-lock add <project.json> <out.json> <name> host=LAYER[:path] follower=LAYER[:path] [verts=1,4-7] [strength=1]`
e `osatura mesh-lock list|remove <project.json> [<out.json> <name>]`

Fa seguire ai vertici del follower la **superficie della mesh host** a ogni fotogramma (trasferimento di spostamento baricentrico), per esempio una camicia che segue un corpo. I cicli vengono rifiutati; host e follower devono avere lo stesso `parallax` (`mesh lock 'lock1': host and follower must have the same parallax`).

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

**Sinossi:** `osatura glue add <project.json> <out.json> <name> a=LAYER[:path] b=LAYER[:path] pairs=i:j;i:j|auto=RADIUS [mode=rigid|snap|offset] [strength=1]`
e `osatura glue list|remove <project.json> [<out.json> <name>]`

Incolla i vertici del livello `b` ai vertici del livello `a` (b segue a). `pairs=i:j;...` elenca esplicitamente le coppie di vertici, `auto=RADIUS` accoppia ogni vertice di `b` che si trova entro il raggio da un vertice di `a`. Modalità: `rigid` mantiene l'offset di riposo nel sistema di riferimento di `a`, `snap` li fa coincidere, `offset` mantiene un offset costante.

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

**Sinossi:** `osatura mesh-trace <project.json> <out.json> <layer> [density=24] [tolerance=1.5] [smooth=N] [hotspot=x,y,r;...] [edge=D/4] [grading=0.5] [lambda=0.5] [taubin=1] [smooth-boundary=0] [threshold=128] [bones=R] [hot-size=E] [skin=1]`

Traccia la **silhouette alfa di un livello immagine** in una mesh triangolare libera a densità variabile (più fitta lungo il contorno, attorno agli hot-spot e alle ossa), la leviga e ne fa lo skinning sulle ossa. Stampa il numero di vertici/triangoli e la qualità.

```
$ osatura mesh-trace o.json mt.json top density=12 tolerance=2 bones=0
wrote mt.json: 9589 vertices (1494 on the outline), 17682 triangles, area 1227677 px2
  quality min 0.484 mean 0.890, min angle 18.3 deg, mean edge 3.0 px at the outline / 13.7 px inside
```
(`o.json` è un'importazione OpenRaster di 1280x960; un valore di `density` più basso produce triangoli più grandi e meno numerosi. L'esecuzione ha richiesto 5 s in una build debug.) **Limiti:** richiede un livello immagine (una silhouette alfa); una mesh così densa ingrandisce molto il file di progetto: controlla `size-report`. Le mesh libere non possono essere simulate come cloth.

### `skin-diffuse`

**Sinossi:** `osatura skin-diffuse <project.json> <layer> <out.json> [--influences 4]`

Calcola i **pesi di skin per diffusione del calore** per un livello con mesh a partire dalle ossa del progetto, mantenendo al massimo `--influences` (predefinito 4) ossa per vertice. Nota l'ordine degli argomenti: **il livello prima dell'output**. Il livello deve essere un livello immagine con una mesh.

```
$ osatura skin-diffuse demo.json arm sd.json
layer 'arm' is not an image layer                          [exit 1]
$ osatura skin-diffuse o.json top sd.json
layer 'top' has no mesh                                    [exit 1]
$ osatura skin-diffuse mt.json top sd.json               # mt.json from mesh-trace
wrote sd.json
```

---

## 8. Audio, lip-sync e suoni

Gli input audio dei comandi di analisi (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) devono essere **WAV** (PCM intero a 8/16/24/32 bit o in virgola mobile a 32 bit, qualunque numero di canali mixato in mono, fino a 256 MiB). MP3/OGG vanno prima convertiti (`ffmpeg -i in.mp3 in.wav`). Le **tracce** sonore memorizzate in un progetto (`sound`) possono avere qualunque formato che ffmpeg sappia leggere.

### `lipsync`

**Sinossi:** `osatura lipsync <project.json> <audio.wav> <switch_layer> <out.json> [--map 0,1,2,3,4]`

Pilota un **livello switch** (forme della bocca) in base a volume e luminosità spettrale: un'euristica su ampiezza / passaggi per lo zero che sceglie una di cinque forme per fotogramma (`rest, small, open, round, wide`) ed elimina lo sfarfallio. **Non è riconoscimento dei fonemi.** `--map` fornisce l'indice del figlio dello switch per ciascuna delle cinque forme; il valore predefinito dipende dal numero di figli dello switch: 1 figlio -> tutte; 2 -> chiusa + aperta; 3 -> chiusa + aperta + tonda/larga; 4 -> chiusa + piccola + aperta + tonda/larga; 5+ -> nell'ordine indicato sopra. Sostituisce un eventuale canale `layer_switch` con chiavi di quel livello. La lunghezza del progetto **non** viene cambiata (un WAV di 2 s su un progetto di 8 fotogrammi imposta chiavi su 8 fotogrammi).

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

**Sinossi:** `osatura lipsync-text <project.json> <audio.wav> <transcript.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--lang ro|en] [--words-out timing.json|timing.srt]`

Allinea una **trascrizione** all'audio (energia + allineamento del testo: parole e sillabe vengono distribuite sulle regioni di parlato trovate nell'audio) e ricava le forme della bocca dalle sillabe. `--lang` (predefinito `ro`) seleziona le regole per vocali/sillabe; esistono solo `ro` e `en`. `--words-out` scrive i tempi delle parole come JSON (parole con sillabe) oppure, per un nome `.srt`, come sottotitoli.

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
**Limiti (ROADMAP §9):** nessuna temporizzazione per lettera (a livello di fonema), nessun modello di velocità specifico del parlante, nessuna lingua oltre a ro/en. Un tono sintetico viene allineato come parlato: usa registrazioni reali.

### `speech-motion`

**Sinossi:** `osatura speech-motion <project.json> <audio.wav> <transcript.txt> <out.json> [--head bone] [--brow bone_or_layer] [--nod deg] [--fall deg] [--tilt deg] [--brow-px px] [--seed n] [--lang ro|en] [--words-out timing.json|timing.srt]`

Aggiunge cenni del capo, cadute, inclinazioni e sollevamenti delle sopracciglia **additivi**, temporizzati sulle parole allineate. `--head` indica l'osso della testa, `--brow` un osso o livello per le sopracciglia; le opzioni sugli angoli sono in gradi, `--brow-px` in pixel, `--seed` fissa la variazione.

```
$ osatura speech-motion small.json speech.wav transcript.txt sm.json --head arm_up --lang en
wrote sm.json (1 additive channel(s) from 3 words)
```

### `mouth`

**Sinossi:** `osatura mouth <project.json> <cues.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--rhubarb]`

Importa **cue** della bocca già pronte invece di analizzare l'audio: file Papagayo/Moho-switch (`frame PHONEME` per riga, prima riga opzionale `MohoSwitch1`) oppure TSV di Rhubarb (`seconds<TAB>shape`, riconosciuto dal punto decimale; i secondi vengono convertiti agli fps del progetto). Le righe vuote e i commenti `#` vengono ignorati. I fonemi sono mappati sulle cinque classi di forma (`rest/MBP/X/A` riposo; `etc/FV/L/B/G/H` piccola; `AI/D` aperta; `O/U/WQ/F` tonda; `E/C` larga); le forme sconosciute contano come `small`. `--rhubarb` tratta la `E` di Rhubarb come *tonda* (la `E` di Papagayo è larga). Il `--map` predefinito è `0,1,2,3,4`.

```
$ printf '0 rest\n2 open\n4 round\n6 wide\n' > cues.txt
$ osatura mouth small.json cues.txt mouth mo.json --rhubarb
wrote mo.json (4 cues)
```

### `audio-react`

**Sinossi:** `osatura audio-react <project.json> <out.json> <audio.wav> <target> <prop> [gain=1] [offset=0] [floor=0.05] [attack=1] [release=4] [add=on]`

Il volume pilota una proprietà: una chiave per fotogramma che segue il livello levigato (RMS relativo al fotogramma più forte). Output = (livello filtrato da `floor`) x `gain` + `offset`; `attack`/`release` sono i tempi di salita/discesa in fotogrammi; `add=on` rende il canale additivo invece di sostitutivo. Un canale bloccato viene rifiutato. Le opzioni sono parole `key=value`; una sconosciuta dà `unknown option 'foo'`.

```
$ osatura audio-react small.json ar.json speech.wav ink layer_rotation gain=10
wrote ar.json
$ osatura audio-react small.json ar.json speech.wav arm_up bone_angle foo=1
unknown option 'foo'                                                [exit 1]
```

### `beats`

**Sinossi:** `osatura beats <project.json> <out.json> <audio.wav> [sensitivity=1.5] [gap=4] [as=markers|events] [name=beat]`

Rileva i salti di volume sopra la media recente (`sensitivity` 1.2-3, più alta = meno battiti) distanziati di almeno `gap` fotogrammi, e li registra come **marker** della timeline (predefinito) oppure come **eventi** chiamati `name`. Stampa i fotogrammi trovati.

```
$ osatura beats small.json bt.json speech.wav as=events name=hit sensitivity=1.2
1 beats at frames [1]; wrote bt.json
$ osatura beats small.json bt.json speech.wav
0 beats at frames []; wrote bt.json
```

### `waveform`

**Sinossi:** `osatura waveform <audio.wav> <out.json> [buckets=800]`

Picchi min/max per disegnare una forma d'onda: `{"sample_rate","duration","peaks":[[min,max],...]}`. Non legge un progetto.

```
$ osatura waveform speech.wav wf.json buckets=50
wrote wf.json (50 buckets)
```

### `sound`

**Sinossi:**
```
osatura sound list   <project.json>
osatura sound add    <project.json> <out.json> <name> <file.wav|ogg|mp3|...> [start_frame] [gain] [trim/fade options]
osatura sound edit   <project.json> <out.json> <name> [--start F] [--gain G] [trim/fade options]
osatura sound remove <project.json> <out.json> <name>
osatura sound mix    <project.json> <out.wav|out.ogg|out.mp3>
    trim/fade options (seconds, or frames with an f suffix such as 36f): --trim-in T --trim-out T --fade-in T --fade-out T
```
Le tracce sonore sono **riferimenti** (percorso del file relativo al progetto, fotogramma iniziale, guadagno 0..4, finestra di taglio all'interno del file, dissolvenze). Nel progetto non viene incorporato nulla; il file audio deve viaggiare con esso e deve trovarsi dentro la directory del progetto, a meno che non sia indicato `--allow-external-assets`. `edit` cambia solo i campi indicati (`0` azzera un taglio). `--trim-out 0` significa "fino alla fine". `sound mix` renderizza solo l'audio mixato per la **lunghezza del progetto** (tramite ffmpeg: wav = PCM 44.1 kHz a 16 bit, ogg = Vorbis q4, mp3 = 96 kb/s).
L'audio mixato viene anche multiplexato in `.mp4` (AAC) e `.webm` (Opus) e incorporato in `.html`; `.gif`/`.apng` non possono trasportarlo.

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
`check` fallisce quando un file sonoro referenziato manca o non è leggibile. Senza ffprobe, per i file non WAV viene verificata solo l'esistenza (non che il file sia audio). **Limiti (ROADMAP §9):** nessun pan/EQ/ducking per traccia, solo la curva di dissolvenza lineare, nessun loop di un clip, nessuna ri-temporizzazione.

---

## 9. Import

### `import-svg`

**Sintassi:** `osatura import-svg <file.svg> <out.json> [--split]`

Riga d'uso: `osatura import-svg <file.svg> <out.json> [--split]  SVG subset -> one vector layer, or one layer per shape named by its id (warnings for the rest)`

Crea un nuovo progetto (24 fps, 24 fotogrammi; dimensioni dal `viewBox` o da `width`/`height`, altrimenti 512x512) con **un livello vettoriale `svg`** che contiene le forme di un sottoinsieme di SVG: `path` (tutti i comandi), `rect` (angoli arrotondati ignorati), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` con `transform`, e `fill`/`stroke`/`stroke-width` a tinta unita. Tutto il resto viene saltato con una riga `warning:` **su stdout** (quando l'elemento è fuori da `<defs>`; un `<filter>`, `<clipPath>`, `<mask>` o `<pattern>` dentro `<defs>` e gli attributi `filter`, `clip-path` e `mask` vengono ignorati senza avviso): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`/`<radialGradient>`, `<image>`, `<use>`, `<foreignObject>`. Un riempimento a gradiente (`fill="url(#g)"`) ripiega sul grigio. Limiti: 8 MB di SVG, 200.000 punti di ancoraggio, un tetto al numero di elementi.

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
Con `--split` ogni forma diventa **un proprio livello**, chiamato come l'`id` della forma, nell'ordine del file. Una forma senza id prende il nome dal suo tag e dalla sua posizione (`rect1`, `circle6`), un id ripetuto riceve `_1`, `_2`..., e i caratteri che un nome di livello non può contenere (`/`, `::`, caratteri di controllo) diventano `_`; i nomi vengono troncati a 100 caratteri. È la forma da usare quando il disegno verrà rigato parte per parte; vedi [TUTORIAL_SVG.md](../../TUTORIAL_SVG.md).

```
$ osatura import-svg pip.svg pip.json --split
wrote pip.json (16 paths, 16 layers)
```
Import e poi merge: non esiste alcun comando che unisca il livello a un progetto esistente; usa `script` oppure modifica il JSON.

### `import-ora`

**Sintassi:** `osatura import-ora <file.ora> <out.json> [--fps N] [--frames N]`

OpenRaster -> progetto: un **livello immagine per ogni livello raster** (PNG incorporato), con l'ordine dal basso verso l'alto conservato, i livelli nascosti mantenuti ma invisibili, i gruppi appiattiti, `opacity`, `visibility` e posizione (`x`,`y`) per livello e le comuni operazioni di composizione `svg:` (multiply, screen, overlay, darken, lighten, color-dodge/burn, hard/soft-light, difference, exclusion, plus -> add). Il progetto ha `--frames` fotogrammi (predefinito 1) a `--fps` (predefinito 24). L'opacità di una pila annidata si moltiplica in quella dei suoi livelli e la sua modalità di fusione viene ereditata, ma i livelli sovrapposti all'interno di una pila traslucida **non** vengono prima appiattiti (si vedono in trasparenza l'uno attraverso l'altro). I nomi di livello duplicati ricevono i suffissi `_1`, `_2`.

```
$ osatura import-ora t.ora ora.json
wrote ora.json (2 layers)
$ osatura import-ora t.ora ora2.json --fps 12 --frames 24
$ osatura import-ora layer.png bad.json
not a zip/ORA file: invalid Zip archive: Could not find EOCD       [exit 1]
```
Limiti: 2000 voci nell'archivio, 128 MiB per voce, livelli da 64 Mpixel.

### `import-lottie`

**Sintassi:** `osatura import-lottie <in.json> <out.json> [fx.json]`

Importa come progetto un file Lottie (Bodymovin) - un **sottoinsieme** deliberato. Stampa `imported N layers; wrote <out>` e una riga `warning:` per ogni funzionalità saltata.

* **Importato:** livelli shape con tracciati `sh`, `rc`, `el`, riempimenti a tinta unita e a gradiente (i punti finali/centro/raggio animati del gradiente diventano canali `grad_dx/grad_dy`), contorni, gruppi, trasformazioni di livello (posizione/scala/rotazione/opacità statiche o con chiavi; i percorsi curvi di posizione vengono baked per fotogramma), modalità di fusione, un livello di sfondo a tinta unita, livelli immagine con data URI PNG/JPEG incorporati, livelli null come genitori (le trasformazioni del genitore vengono baked nei figli), maschere di livello (add/subtract/intersect con opacità, inversione, forme con chiavi), luma matte (approssimati), precomposizioni, intervalli in/out del livello (come traccia a gradino dell'opacità), easing delle chiavi (hold -> step; maniglie bezier -> cubic-bezier; nessuna maniglia -> lineare).
* **Tracciati animati:** un tracciato le cui chiavi di forma hanno la stessa topologia (stesso numero di ancoraggi, aperto/chiuso) diventa un tracciato di base più un **morph target** per ogni ulteriore chiave distinta (`k1`, `k2`, ...) con canali di peso `path_morph` che trasportano l'easing. Topologia diversa: viene usata la prima forma e viene stampato un avviso.
* **Effetti (`ef`):** sfocatura gaussiana, ombra esterna, tinta e riempimento diventano **effetti per livello** (`effects` sul livello, di qualsiasi tipo) con i numeri animati come canali `fx_*`; tinta/riempimento/tritone/livelli statici sui livelli shape vengono baked nei colori. I colori degli effetti non possono essere animati (si usa il primo valore); gli altri effetti, e gli effetti colore successivi a una sfocatura/ombra sui livelli non shape, generano avvisi. Per ogni livello vengono mantenuti al massimo i primi effetti.
* **Saltato con un avviso:** testo e altri tipi di livello, risorse audio, punti di ancoraggio animati (primo valore), percorsi di movimento curvi su proprietà diverse dalla posizione (segmenti rettilinei), maschere su una sorgente track-matte, catene di genitori troppo lunghe da bakare, più di 2000 livelli, animazioni più lunghe di 20.000 fotogrammi.
* `fx.json` è mantenuto per compatibilità: viene scritto un array vuoto (sfocatura e ombra esterna ora sono effetti per livello all'interno del progetto).

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
**Attenzione:** `import-lottie in.json in.json` viene rifiutato (`refusing to overwrite the input ...`), anche per `fx.json`. **Limiti:** l'input `.lottie` (zip) non viene letto (`stream did not contain valid UTF-8`) - decomprimi prima `a/animation.json`; il lavoro quadratico nei morph di forma è limitato solo dai tetti di dimensione.

### `import-bvh`

**Sintassi:** `osatura import-bvh <project.json> <file.bvh> <out.json> --joint-map Joint=bone,.. [--axis Zrotation --tol 0.5]`

Mappa i canali di rotazione di giunti BVH scelti sugli ossi del rig come canali `bone_angle` con chiavi, campionati al fps e al numero di fotogrammi del progetto. `--joint-map` è obbligatorio; `--axis` (predefinito `Zrotation`) seleziona quale canale di rotazione BVH viene letto (`Zrotation`, `Xrotation`, `Yrotation`); `--tol` (predefinito 0,5 gradi) semplifica le chiavi. Per l'abbinamento automatico, il moto della radice, le viste e la scala usa `mocap-retarget` (sezione 10).

```
$ osatura import-bvh demo.json m.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```

### `import-seq`

**Sintassi:** `osatura import-seq <project.json> <out.json> <layer_name> <folder> [hold=1] [loop=on|off] [start=0] [x=0] [y=0]`

Importa una **sequenza PNG** (tutti i PNG della cartella in ordine di nome file) come livello fotogramma per fotogramma: un livello switch chiamato `<layer_name>` con un livello immagine per ogni fotogramma e un canale a gradino che espone ciascuno per `hold` fotogrammi a partire dal fotogramma `start`, nella posizione `(x, y)`; `loop=on` lo ripete. Tutte le immagini devono avere la stessa dimensione (`img.png: size 640x480 differs from the first image (160x120)`).

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20
imported 3 images; wrote is.json
```

### `import-anim`

**Sintassi:** `osatura import-anim <project.json> <out.json> <layer_name> <file.gif|file.apng|file.png> [loop=on|off] [start=0] [x=0] [y=0]`

Importa una **GIF animata o un APNG** come livello fotogramma per fotogramma, usando i ritardi dei fotogrammi del file stesso (convertiti in fotogrammi del progetto). Un PNG semplice dà un solo fotogramma.

```
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0
imported 4 frames as layer 'gifl'; wrote ia.json
$ osatura import-anim demo.json ia2.json apngl dof.apng
imported 3 frames as layer 'apngl'; wrote ia2.json
```
Le immagini dei fotogrammi diventano PNG incorporati: una GIF lunga rende il progetto grande.

### `trace`

**Sintassi:** `osatura trace <image.png> <out.json> [--threshold 128 --tolerance 1.0 --min-area 12 --fill #000000]`

Semplice conversione da bitmap a vettoriale per immagini con **trasparenza**: traccia il **canale alfa** del PNG (marching squares a `--threshold`, 0..255, con interpolazione lineare dei bordi), semplifica i contorni (Ramer-Douglas-Peucker) fino a `--tolerance` pixel, scarta le regioni sotto `--min-area`, rileva i fori per contenimento e scrive un nuovo progetto (di nome `traced`, canvas = dimensione dell'immagine) con i contorni come tracciati riempiti del colore `--fill`. Un'immagine completamente opaca produce quindi un solo contorno attorno all'intera immagine (come nell'esempio). Stampa `wrote <out> (N contours)`. Per le immagini a colori usa `autotrace`.

```
$ osatura trace img.png tr.json --threshold 128 --tolerance 1.0 --min-area 12 --fill '#112233'
wrote tr.json (1 contours)
```

### `autotrace`

**Sintassi:** `osatura autotrace <image.png|jpg> <out.json|out.svg> [--mode fill|line] [--colors N=8] [--min-area A=8] [--tol T=1] [--smooth N=3] [--corner DEG=55] [--denoise N=0] [--quant kmeans|mediancut] [--seed S] [--fit bezier|polygon|vw] [--zorder area|luminance] [--threshold T] [--invert] [--prune N=4] [--into project.json] [--name L] [--fit-stage] [--png preview.png]`

Tracciamento locale di bitmap - **nessun modello, nessuna rete**: quantizzazione dei colori (`--colors`, k-means o median cut), estrazione dei contorni, fitting di Bezier cubiche di Schneider (`--fit`; `polygon` e Visvalingam-Whyatt `vw` sono alternative), fori con regola pari-dispari; `--mode line` assottiglia i tratti fino alle linee centrali (Zhang-Suen, `--prune` rimuove le piccole sporgenze). L'output è un nuovo progetto, un nuovo `.svg` oppure - con `--into project.json` - una copia di un progetto esistente con un nuovo gruppo di livelli `--name` (predefinito `trace`); `--fit-stage` (con `--into`) scala il nuovo gruppo in modo uniforme per adattarlo e centrarlo sullo stage del progetto, `--png` scrive un rendering di anteprima. Stampa i conteggi e l'**errore misurato rispetto alla sorgente** (errore assoluto medio, RMSE, PSNR, SSIM).

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
**Limiti (ROADMAP §9):** un tracciatore classico - le fotografie diventano bande di colore; nessun tracciamento appreso o semantico. `--png` qui è un'opzione con valore (nome del file di anteprima), a differenza degli altri flag `--png`.

### `autotrace-bench`

**Sintassi:** `osatura autotrace-bench`

Traccia forme sintetiche, gradienti e tratti integrati e stampa una tabella di fedeltà (MAE, RMSE, PSNR, SSIM, numero di regioni/ancoraggi, millisecondi). Non accetta argomenti; sola lettura.

```
$ osatura autotrace-bench
image          mode  colors  tol     MAE    RMSE   PSNR   SSIM regions strokes  anchors      ms
shapes         fill       6    1    2.19   12.10   26.5  0.975      11       0       33    13.6
...
```

---

## 10. Input dal vivo, MIDI, face tracking e motion capture

### `live-map`

**Sintassi:** `osatura live-map <project.json> <events.json|song.mid> <mapping.json> <out.json> [--takes lib.json] [--tol 0.01] [--allow-empty]`

Trasforma una **performance dal vivo registrata** (MIDI, mouse, gamepad, tasti, segnali del volto) in normali canali con chiavi, eventi e take (non distruttivo, tutto o niente, blocchi rispettati). L'input è un file `.mid` Standard MIDI File oppure un JSON di eventi `{"events":[{"source","id","value","time"}]}` (`value` normalizzato 0..1, `time` in secondi; `source` = `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`; `id` = per MIDI `"<channel 1-16>:<number>"`, per il pulsante del mouse `"0"`, per il gamepad `"<pad>:<index>"`, per i tasti il codice DOM come `KeyA`).

Il file di mapping ha `params` (source/id -> `target`+`prop`, con `in_min in_max out_min out_max deadzone invert curve smooth additive`) e `triggers` (superamento di soglia -> azione `jump_to_frame`, `play_take`, `set_key`, ...); le chiavi sconosciute sono errori. `--takes` è una libreria di take per le azioni `play_take` (deve esistere), `--tol` semplifica le chiavi.

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
**Attenzione:** se gli eventi sono stati letti ma **non è stato prodotto nulla** (errore di battitura in un id, source sbagliata) il comando fallisce e non scrive nulla, a meno di `--allow-empty`; con zero eventi in input scrive semplicemente il progetto invariato. I trigger di salto vengono registrati come un evento chiamato `jump` con `data.to` (l'asse temporale non viene riscritto).
**Limiti (ROADMAP §9):** nessun input dal vivo da webcam/livello audio nello studio tramite questo comando (`audio-react` è offline); l'aspetto interattivo è fornito dallo studio.

### `drive`

**Sintassi:** `osatura drive <project.json> <signals.csv> <maps.json> <out.json> [--tol 0.5]`

Performance -> chiavi da un **CSV**: la riga di intestazione dà i nomi ai segnali, ogni riga successiva è un fotogramma (indice di riga = fotogramma). `maps.json` è un array di `{input, target, prop, in_min, in_max, out_min, out_max, deadzone, invert, curve, smooth}`; `--tol` (predefinito 0,5) semplifica le chiavi.

```
$ printf 'jaw,speed\n0,0\n0.5,0.2\n1,0.4\n0.5,0.6\n' > sig.csv
$ echo '[{"input":"jaw","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":40,"smooth":0.2}]' > maps.json
$ osatura drive demo.json sig.csv maps.json dr.json --tol 0.5
wrote dr.json (+1 channels)
```

### `face-synth`, `face-train`, `face-eval`, `face-detect`, `face-track`, `face-calibrate`, `face-bake`

Una pipeline guidata dal volto che è **interamente locale e addestrata solo su volti sintetici** (teste stilizzate/cartoon, 23 landmark, layout `osatura23`). Un piccolo modello addestrato su dati sintetici è incorporato nel binario; `--model` lo sostituisce. **Fotografie, webcam e video reali non sono validati** (ROADMAP §9; vedi anche [PATENT_NOTES.md](PATENT_NOTES.md): il puppeteering guidato dal volto non è stato sottoposto a verifica brevettuale).

| Comando | Sintassi | Cosa fa |
|---|---|---|
| `face-synth` | `osatura face-synth <out-dir> [--frames 90] [--size 160] [--seed 7] [--fps 30]` | Animazione sintetica di una testa: fotogrammi PNG `000000.png...`, `truth.ndjson` (landmark reali) e `params.json` (segnali reali); `--fps` è la frequenza di fotogrammi a cui sono temporizzati i segnali (scritta nell'intestazione NDJSON). |
| `face-train` | `osatura face-train --samples N --seed S --out model.bin [--mine-rounds 1] [--scene N] [--reg-win N] [--reg-cell N] [--det-win N] [--det-cell N] [--ridge-lambda X] [--stages N] [--refine-win N] [--stage-patch N] [--stage-cell N] [--stage-lambda X]` | Addestra il rilevatore proprio solo su volti sintetici (nessun download). `--scene` è la dimensione dell'immagine di addestramento (64..512); `--reg-win`, `--reg-cell`, `--det-win`, `--det-cell`, `--ridge-lambda`, `--stages`, `--refine-win`, `--stage-patch`, `--stage-cell` e `--stage-lambda` sono i parametri di regolazione della dimensione del modello e della regressione (i valori predefiniti sono quelli con cui è stato addestrato il modello incorporato). |
| `face-eval` | `osatura face-eval [--model m.bin] [--eval-n 200] [--seed 9] [--scene 128] [--report r.json] [--json]` | Accuratezza su dati sintetici esclusi dall'addestramento, falsi positivi e sweep di robustezza (JSON; `--report` lo scrive, `--json` lo stampa anche con `--report`; `--scene` è la dimensione dell'immagine di test). |
| `face-detect` | `osatura face-detect <image> [--model m.bin] [--json] [--overlay o.png] [--threshold T] [--max-faces N] [--min-side N] [--max-side N] [--max-dim N]` | Riquadri dei volti e 23 landmark; `--overlay` li disegna su un PNG; `--min-side` / `--max-side` limitano la dimensione del volto in pixel e `--max-dim` riduce prima un'immagine grande. |
| `face-track` | `osatura face-track <video-or-frame-dir> <out.ndjson> [--model m.bin] [--fps 30] [--max-dim 256]` | NDJSON dei landmark per `face-bake`. Una directory di immagini viene letta direttamente; un video passa per ffmpeg. |
| `face-calibrate` | `osatura face-calibrate <landmarks.ndjson> <out.json> [--calibrate-frames N] [--map mapfile.json] [--fps F] [--layout L]` | Calcola la calibrazione del volto neutro dai primi N fotogrammi. |
| `face-bake` | `osatura face-bake <project.json> <landmarks.ndjson> <mapfile.json> <out.json> [--calibrate-frames N\|0] [--calib calib.json] [--fps F] [--layout L] [--tol T] [--allow-empty]` | Esegue il bake dei segnali del volto (`mouth_open`, `eye_l_open`, `eye_r_open`, `brow_raise`, `yaw`, `pitch`, `roll`, ...) in canali con un file di mapping in stile `live-map` la cui `source` è `face`. |

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
Questi comandi scrivono i loro file di modello/report con lo scrittore atomico che rifiuta i symlink (le cartelle padre vengono create). `face-bake` e `live-map` riportano `+N channels (K keys in them), M trigger keys set` e avvisano su stderr quando la registrazione è più lunga del progetto (i fotogrammi in eccesso non vengono baked; la lunghezza del progetto resta invariata).

### `mocap-info`, `mocap-match`, `mocap-retarget`

```
osatura mocap-info     <file.bvh>
osatura mocap-match    <project.json> <file.bvh> [--map map.json]
osatura mocap-retarget <project.json> <file.bvh> <out.json> [--view front|side|top|yaw=DEG] [--map map.json] [--scale auto|N] [--fps F]
    [--smooth N] [--start F] [--end F] [--rest-frame F] [--min-conf C] [--weight W] [--mirror] [--no-root] [--in-place] [--keep-length] [--footlock BONE --ground Y]
```
* **`mocap-info`** stampa la gerarchia, i canali, la frequenza di fotogrammi e l'estensione di un file BVH.
  ```
  $ osatura mocap-info m.bvh
  4 joints (2 end sites), 15 channels per frame
  4 frames at 0.0417 s (24.000 fps), duration 0.125 s
  Hips  offset (0.000, 0.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
    LeftUpLeg  offset (5.000, 0.000, 0.000)  [Zrot Xrot Yrot]
  ...
  ```
* **`mocap-match`** stampa la **tabella di abbinamento automatico giunto -> osso** (slot, giunto BVH, osso del rig, confidenza, evidenza `via`, rapporto di lunghezza, nota), poi i giunti/ossi non abbinati e gli avvisi. Vengono usati sia i nomi sia la topologia. `--map` è un oggetto JSON `{"bvh joint": "rig bone" | null}` (`null` o `""` esclude un giunto; è accettato anche l'involucro `{"map": {...}}`). Termina sempre con codice 0 - quando nulla corrisponde avvisa soltanto `no bones matched: supply a --map file`.
  ```
  $ osatura mocap-match demo.json m.bvh --map map2.json         # {"Hips":null,"LeftUpLeg":"thigh","LeftLeg":"shin"}
  slot         bvh joint                  rig bone                conf  via         len x  note
  shin.l       LeftLeg                    shin                    1.00  manual      12.00  manual
  thigh.l      LeftUpLeg                  thigh                   1.00  manual      12.00  manual
  unmatched BVH joints: Hips, RightUpLeg
  ...
  ```
* **`mocap-retarget`** esegue il bake del movimento come canali **aggiunti**: proietta le rotazioni 3D sulla `--view` 2D (`front` predefinita, `side`, `top` o `yaw=DEG`), scala il moto della radice (`--scale auto` o un numero), ricampiona a `--fps` (predefinito quello del progetto), leviga (`--smooth` fotogrammi), prende i fotogrammi BVH `--start..--end`, usa `--rest-frame` come posa di riposo, scarta gli abbinamenti sotto `--min-conf`, fonde con `--weight`, specchia con `--mirror`, elimina il moto della radice con `--no-root`, converte una clip con spostamento in una sul posto con `--in-place` e blocca un piede con `--footlock BONE --ground Y`. Stampa la tabella, le note e `N bones, F frames at R fps, root scale S, +C channels; wrote <out>`. Una clip più lunga del progetto **estende la lunghezza del progetto**; una nota dopo il riepilogo indica da quale a quale lunghezza. `--keep-length` mantiene invece la lunghezza del progetto e tronca le chiavi all'ultimo fotogramma (lì viene posta una chiave interpolata; una nota conta le chiavi non baked). Se nulla supera la soglia di confidenza: `no bone matches to retarget (all below the confidence threshold?)` (codice 1). Nota: per questo comando `--scale` è `auto|N`, non la solita scala di rendering.
  ```
  $ osatura mocap-retarget demo.json m.bvh mr.json --map map2.json --view side --smooth 1 --in-place --no-root
  ... (table)
  2 bones, 4 frames at 24 fps, root scale 12.0000, +2 channels; wrote mr.json
  ```
**Limiti (ROADMAP §9):** non è stato testato alcun file di motion capture reale (nessuno è incluso: gli esempi usano un BVH di 4 fotogrammi scritto a mano); la proiezione 2D scarta la profondità (un osso che punta verso la camera mantiene l'ultimo angolo).

---

## 11. State machine, data binding, container e regioni di scorrimento

Questi comandi gestiscono le parti di **interazione a runtime** di un progetto: le state machine (`machines`), i data binding
(`bindings`), i container di layout (`containers`) e le regioni di scorrimento (`scrolls`). Tutti memorizzano i propri dati all'interno
del file di progetto; le forme `add` leggono un piccolo file JSON di appoggio, lo validano e scrivono un nuovo progetto.
Il modello di runtime in sé è documentato in [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

### `machine` (add, list, remove, validate)

**Sintassi:**
```
osatura machine add      <project.json> <out.json> <machine.json>
osatura machine list     <project.json>
osatura machine remove   <project.json> <out.json> <name>
osatura machine validate <project.json>
```

Una **machine** ha `inputs` con nome (`bool`, `number` o `trigger`) e uno o più `layers`; ogni livello ha uno stato `entry`,
`states` e `transitions`. Uno stato riproduce una **sorgente**: `{"kind":"segment","start":F,"end":F}` (fotogrammi dei
canali con chiavi del progetto), `{"kind":"clip","clip":NAME}`, `{"kind":"region","marker":NAME}` oppure un blend tree 1-D
`{"kind":"blend","input":NAME,"children":[{"threshold":T,"source":{...}}]}`. Campi dello stato: `name`, `source`,
`mode` (`loop` predefinito, `once`, `ping_pong`), `speed` (1), `blend_in` (secondi). Campi della transizione: `from` (uno stato o
`"*"`), `to`, `when` (un elenco di `{"input":NAME,"op":eq|ne|lt|le|gt|ge,"value":V}`, devono valere tutte; `{"input":"go"}`
da solo significa "è uguale a 1"), `exit_time` (0..1), `duration` (secondi di crossfade) ed `ease`. Le chiavi sconosciute in una transizione
sono un **errore** (un refuso come `conditions` trasformerebbe altrimenti una transizione condizionata in una incondizionata).

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

`machine remove` rifiuta di rimuovere una machine che è ancora pilotata da un binding
(`binding 'goBind' still drives machine 'm'`, codice 1): rimuovi prima il binding.
**Errori:** una machine non valida (stato sconosciuto, nomi duplicati, riferimento a un input errato) viene rifiutata con un messaggio e
non viene scritto nulla. **Correlati:** `machine-sim`, `bind`, `state-bake` (il formato di machine più vecchio, basato sui fotogrammi).
**Limiti:** le machine sono valutate dal runtime e da `machine-sim`; gli esportatori non eseguono il bake di nulla di tutto ciò (Lottie, GIF
e gli altri file contengono solo fotogrammi, vedi [EXPORT_FORMATS.md](EXPORT_FORMATS.md)).

### `machine-sim`

**Sintassi:** `osatura machine-sim <project.json> <script.json> <out_dir|out.json> [--fps N --svg-frames N]`

Riproduce eventi di input temporizzati su una machine e registra lo stato di ogni tick. Lo script è
`{"machine": NAME?, "duration": SECONDS?, "events": [{"t": SECONDS, "set": {"input": value}, "trigger": ["name"],
"value": {"external.path": number}}]}`. `machine` è per impostazione predefinita l'unica machine del progetto, `duration` l'ultimo
evento più un secondo; `value` alimenta i data binding (vedi `bind`). `--fps` predefinito è l'fps del progetto.
`--svg-frames N` renderizza inoltre N fotogrammi SVG distribuiti sull'esecuzione.

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
`sim.json` contiene `machine`, `fps`, `ticks[]` (`tick`, `time`, `states` come coppie `[layer,state]`, `fading`) e il
registro degli eventi. **Correlati:** `bind`, `machine`.

### `state-bake`

**Sintassi:** `osatura state-bake <project.json> <machine.json> <events.json> <out.json> [--tol 0.01]`

Esegue il bake di una state machine **basata sui fotogrammi** (un formato diverso e più vecchio di `machine`) in normali canali con chiavi.
`machine.json` è `{"initial":NAME,"states":[{"name","length","looped","channels":[...]}],"transitions":[{"from":NAME
or "*","to","when":COND,"fade":FRAMES,"ease"}]}` dove `COND` è uno tra `{"type":"event","name"}`,
`{"type":"above"|"below"|"equals","input","value"}`, `{"type":"finished"}`, `{"type":"after","frames"}`.
`events.json` è un elenco di `{"frame":N,"name":NAME,"value":V}`. Il risultato viene aggiunto in coda ai canali del progetto
(`--tol` è la tolleranza di riduzione delle chiavi), il progetto viene compilato e scritto.

```
$ echo '[{"frame":10,"name":"go"}]' > ev.json
$ osatura state-bake demo.json sm.json ev.json baked.json --tol 0.01
wrote baked.json (+1 channels)
```
**Attenzione:** il canale baked viene aggiunto dopo quelli esistenti; i canali vengono eseguiti nell'ordine dell'elenco, quindi rimuovi o silenzia il
canale originale per lo stesso target se vuoi che prevalga il bake. Un file di machine nel formato `machine`
(con `layers`) viene rifiutato: `invalid JSON in mach.json: missing field 'initial'`.

### `bind` (add, list, remove) e `bind-eval`

**Sintassi:**
```
osatura bind add    <project.json> <out.json> <name> <value.path> <target:prop|@machine:input>
                     [--expr E --remap a,b,c,d --no-clamp --smooth S --mode replace|add|multiply --weight W]
osatura bind list   <project.json>
osatura bind remove <project.json> <out.json> <name>
osatura bind-eval   <project.json> <values.json> [--data out.json]
```

Un **data binding** mappa un percorso di valore esterno (per esempio `hud.size`) su una proprietà del progetto (`sun:layer_opacity`,
i nomi delle proprietà sono quelli in snake_case della sezione 1) oppure su un input di machine (`@m:go`). `--remap a,b,c,d` mappa
l'intervallo di input `a..b` su `c..d` (con clamp, salvo `--no-clamp`); `--expr` è un'espressione sul valore;
`--smooth S` applica un filtro passa-basso su S secondi; `--mode` e `--weight` controllano come il risultato si combina con i canali.

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
`bind-eval` stampa l'output di ciascun binding per i valori dati (`--data` li scrive anche come JSON). `list` stampa i nomi delle
proprietà in CamelCase; la riga di comando accetta sempre snake_case. **Correlati:** `machine`, `machine-sim` (la sua mappa `value`).

### `container` (add, list, remove, solve)

**Sintassi:**
```
osatura container add    <project.json> <out.json> <container.json>
osatura container list   <project.json>
osatura container remove <project.json> <out.json> <id>
osatura container solve  <project.json> [--size WxH --data out.json]
```

Un **container** è un riquadro di layout in stile flexbox: `id`, `members`, `axis` (`row`|`column`), `gap`, `cross_gap`, `padding`
(`[left,top,right,bottom]`), `justify` (`start|center|end|space_between|space_around`), `align`
(`start|center|end|stretch`), `wrap` (`none|wrap`) e `rect`, una regola per il proprio rettangolo sul canvas:
`{"kind":"fixed",...}`, `{"kind":"pin","left","right","top","bottom","width","height"}` (distanze dai bordi del canvas; due qualsiasi
per ciascun asse), `{"kind":"aspect",...}` oppure `{"kind":"hug"}`. Un **membro** è esattamente uno tra `{"layer":PATH}`,
`{"container":ID}` o `{"spacer":true}`, con `grow`, `shrink`, `width`, `height`, `min_w/max_w/min_h/max_h` e
un override di `align`; `"resize":true` permette al solver di scalare il livello al proprio slot.

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
Senza `--size` si usa la dimensione del progetto stesso. `solve` stampa il rettangolo di ogni container e, per ogni livello, lo spostamento e
la scala che il layout applica sopra l'animazione. **Errori:** un livello sconosciuto, un container annidato mancante o un
ciclo vengono rifiutati da `add` (nulla viene scritto). **Correlati:** `scroll`, `layout-preview`.
**Limiti:** il layout viene applicato dal runtime/anteprima; i file esportati contengono il layout risolto per la dimensione del progetto stesso.

### `scroll` (add, list, remove, eval)

**Sintassi:**
```
osatura scroll add    <project.json> <out.json> <scroll.json>
osatura scroll list   <project.json>
osatura scroll remove <project.json> <out.json> <id>
osatura scroll eval   <project.json> <id> [--offset X,Y --size WxH --velocity VX,VY --time S]
```

Una **regione di scorrimento** ha `id`, `content` (l'id di un container che non è membro di un altro container), `viewport`
(una regola di rettangolo come sopra), `axis` (`x`, `y` o entrambi), `offset`, `elastic` (`{"limit","stiffness","damping_ratio"}`; effetto
elastico invece di un clamp rigido), `snap_x`/`snap_y` (offset di riposo), `fling` (secondi di velocità di rilascio proiettati per trovare
il punto di arrivo, predefinito 0.15), `clip_layer` (un livello rettangolo usato come maschera per il contenuto) e `input` (un percorso di valore esterno che
pilota l'offset).

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
Senza `--velocity`, `eval` mostra l'offset limitato (o con effetto elastico) per quello richiesto. Con `--velocity`,
simula il rilascio (inerzia, snap, molla) per `--time` secondi e stampa un valore ogni 0,1 s. `--size WxH` valuta per un'altra dimensione di canvas.
Nell'esempio il contenuto è largo quanto il viewport, quindi tutto ciò che supera 0 è overscroll e mostra l'effetto elastico.

### `layout-preview`

**Sintassi:** `osatura layout-preview <project.json> <out.svg> --size WxH [--frame N --scroll id=X,Y]`

Renderizza un fotogramma con il layout **risolto di nuovo per un'altra dimensione di canvas** e con i contorni dei container disegnati, così puoi
verificare come un design si riadatta su un telefono o su uno schermo largo. `--scroll id=X,Y` imposta un offset di scorrimento (ripeti per più regioni).

```
$ osatura layout-preview d5.json lp.svg --size 800x480 --scroll sc1=50,0
wrote lp.svg (800x480, 1 containers, 2 layers placed)
```

## 12. Effetti, profondità di campo, onion skin, scie e camera

### `fx`

**Sintassi:** `osatura fx <project.json> <frame> <chain.json> <out.png> [--scale N] [--layer-normals] [--clamp]`

Un `<frame>` fuori dal progetto è un errore; `--clamp` renderizza il fotogramma valido più vicino.

Renderizza un fotogramma e applica su di esso una **catena di post-effetti**. `chain.json` è un elenco di effetti, ciascuno con un `type`:
`blur {radius}`, `drop_shadow {dx,dy,blur,color,opacity}`, `glow {radius,strength,color?}`, `outline {width,color}`,
`posterize {levels}`, `pixelate {size}`, `levels {in_black,in_white,gamma,out_black?,out_white?}`, `fade {amount}` e
`relight {ambient,bevel,height,lights:[{x,y,z,color?,intensity?,radius?,cast_shadow?,spot?,falloff?}],ground_shadow?,
normal_map?,ambient_color?}`. Senza una `normal_map` il relight ombreggia un bisello ricavato dal canale alfa sfocato.

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
`--layer-normals` ombreggia ogni livello con la propria normal map (vedi `layer-normal`) invece di una sola mappa per l'intera immagine.
Una luce che omette `z` viene rifiutata (`missing field 'z'`). **Limiti:** la catena è una pipeline di immagini statiche solo PNG; nessuno di
questi effetti arriva a Lottie, GIF o al player HTML (l'esportatore Lottie ha un proprio sottoinsieme di effetti, vedi EXPORT_FORMATS.md).

### `color-space`

**Sintassi:** `osatura color-space <project.json> <out.json> srgb|linear`

Imposta il `color_space` del progetto, lo spazio in cui il compositore raster (PNG, GIF, APNG, video, sprite sheet, `compare`, `ci`) e il compositore float (`render-hdr`) eseguono la fusione. `srgb` (il valore predefinito, e quello con cui è stato registrato ogni fotogramma di riferimento) fonde i valori memorizzati codificati in sRGB; `linear` converte ogni colore (riempimenti, contorni, stop di gradiente, sfondo, pixel delle immagini) in luce lineare, compone lì in f32 e converte in sRGB una sola volta alla fine. Un livello nero al 50 % su bianco dà allora 188/255 invece di 128/255, e i gradienti interpolano in luce lineare. Con `srgb` la chiave non viene scritta, quindi i file esistenti e i fotogrammi di riferimento non cambiano.

```
$ osatura color-space demo.json lin.json linear
wrote lin.json (color_space: linear)
$ osatura render lin.json 0 lin.png
wrote lin.png
$ osatura color-space lin.json x.json p3
unknown colour space 'p3' (srgb or linear)
```
**Limiti:** il renderer a 8 bit passa al compositore float per i progetti lineari (rasterizzatore di copertura proprio, quindi i bordi con anti-aliasing differiscono leggermente dall'output sRGB); i colori degli effetti di livello (ombra, bagliore, tinta) sono usati come scritti, non convertiti; `dof` e la catena di post-effetti (`fx`) restano in sRGB; SVG, Lottie e il renderer GPU ignorano l'impostazione; il ricampionamento delle immagini avviene in sRGB prima della conversione. Le modalità di fusione sono applicate ai valori lineari.

### `layer-normal`

**Sintassi:** `osatura layer-normal <project.json> <out.json> <layer> <normal.png|none> [--strength S] [--fit stretch|tile|exact] [--green-down]`

Associa (o, con `none`, rimuove) una normal map in spazio tangente a un livello immagine per `fx ... --layer-normals`.
`--fit` indica come adattare una mappa di dimensione diversa (`stretch` predefinito, `tile`, `exact`); `--green-down` inverte il canale
verde (mappe in stile DirectX); `--strength` scala l'inclinazione (1 = come memorizzata, 0 = piatta).

```
$ osatura layer-normal demo.json dn.json sun n.png --strength 1 --fit stretch
wrote dn.json
$ osatura fx dn.json 12 chain.json fxn.png --scale 0.5 --layer-normals
wrote fxn.png (2 effect(s))
$ osatura layer-normal dn.json dn2.json sun none
wrote dn2.json
```

### `onion` (due forme)

**Sintassi:**
```
osatura onion <project.json> <frame> <out.png> [--before 2 --after 1 --step 1 --scale N]
osatura onion <project.json> <out.svg> --frame N [--before B --after A --step S --keyed --outline --all-parts
               --layers a,b --tint-before C --tint-after C --opacity X --falloff X --ghosts-only]
```
La prima forma scrive un PNG con fantasmi sbiaditi dei fotogrammi precedenti e successivi. La seconda scrive un'immagine statica SVG: il
fotogramma con sopra fantasmi onion skin colorati. `--keyed` usa i **fotogrammi chiave** più vicini invece di passi fissi;
`--outline` disegna i fantasmi come contorni; `--all-parts` crea fantasmi di ogni parte (predefinito: le parti in movimento), `--layers` li limita,
`--ghosts-only` omette il fotogramma stesso.

```
$ osatura onion demo.json 12 on.png --before 2 --after 1 --step 2 --scale 0.5
wrote on.png
$ osatura onion demo.json on.svg --frame 12 --before 2 --after 2 --step 3 --outline --opacity 0.4
wrote on.svg (4 ghost(s) around frame 12)
$ osatura onion demo.json on2.svg --frame 12 --keyed --ghosts-only
wrote on2.svg (3 ghost(s) around frame 12)
```
La forma è scelta dal secondo argomento: un numero indica la forma PNG, un nome che termina in `.svg` la forma SVG.

### `trail` (due forme)

**Sintassi:**
```
osatura trail <project.json> <bone> <out.svg> [--from F --to F]
osatura trail <project.json> [out.svg] --target layer:PATH|bone:NAME[:head|tip]|point:LAYER:KIND:A[:B] [--from F --to F --smooth N --json]
```
Disegna la **scia di movimento** della punta di un osso (prima forma) o di un target qualsiasi (seconda forma: un livello, la testa/punta di un osso o un
punto di deformatore), con un segno per fotogramma e indicatori in corrispondenza delle chiavi. `--smooth N` leviga la curva; `--json` stampa la
curva campionata come JSON (e in tal caso `out.svg` è facoltativo).

```
$ osatura trail demo.json hand tr.svg --from 0 --to 24
wrote tr.svg (25 points)
$ osatura trail demo.json tr3.svg --target layer:sun --from 0 --to 24
wrote tr3.svg (25 ticks, 0 key marker(s))
$ osatura trail demo.json tr2.svg --target bone:hand:tip --from 0 --to 24 --smooth 2 --json
{ "curve": [ [212.8, 83.84], [213.77, 83.99], ...
```

### `dof` e `dof-export`

**Sintassi:**
```
osatura dof        <project.json> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24 --scale N --clamp]
osatura dof-export <project.json> <out.gif|out.apng|out.mp4|out.webm|out-folder> [--from F --to F --focus N --strength N --max-radius N --scale N --codec vp9|av1]
```
Un `<frame>` fuori dal progetto è un errore per `dof` (e per `render-hdr` e `fx`); `--clamp` renderizza invece il fotogramma valido più vicino, come per `render`.

Profondità di campo: ogni **livello di primo livello** viene sfocato di `|parallax - focus|` volte `strength` (pixel per unità di
distanza di parallasse, limitato da `--max-radius`). Senza `--focus` / `--strength` i valori seguono i canali con chiavi
`camera_focus` / `camera_blur` (fuoco predefinito 1.0). I livelli ricavano la profondità da `parallax` (vedi sezione 5).
`dof-export` renderizza l'intervallo `--from`..`--to` (numeri di fotogramma interi; predefinito: da 0 alla fine) in una GIF animata, un
PNG animato, un video `.mp4`/`.webm` (richiede `ffmpeg`; `--codec` è `vp9` per impostazione predefinita; non viene incluso alcun audio) oppure, per un nome senza
estensione, in una cartella di PNG numerati (`f0000.png`...). Qualsiasi altra estensione viene rifiutata (`dof-export writes .gif, .apng,
.mp4, .webm or a folder (no extension)`, nulla viene scritto).

```
$ osatura dof demo.json 12 dof.png --focus 1.0 --strength 12 --max-radius 24 --scale 0.5
wrote dof.png
$ osatura dof-export demo.json dof.gif --from 0 --to 6 --scale 0.25
wrote dof.gif
$ osatura dof-export demo.json dofdir --from 0 --to 2 --scale 0.25      # dofdir/f0000.png ... f0002.png
wrote dofdir
```
**Correlati:** `parallax`, `render --proxy`, `shot`/`camera-seq` per i movimenti di camera. **Limiti:** la sfocatura è
un'approssimazione per livello, non un modello di lente (nessuna forma di bokeh, nessuna gestione dell'occlusione); non viene esportata in Lottie.

## 13. Livello di dettaglio, report sulle dimensioni ed export-fit

### `size-report`

**Sintassi:** `osatura size-report <project.json> [--tol T] [--json]`

Mostra **dove sono i byte**: il file di progetto, il Lottie baked e la pagina HTML, per livello, immagine e canale
(chiavi, e quante sono rimovibili alla tolleranza `--tol`, predefinita 0.05).

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
`--json` stampa gli stessi dati come JSON (`html.defs_bytes_total`, `frame_bytes_max`, ...). Usalo prima di `export-fit` o `lod`
per vedere quale livello semplificare.

### `lod`

**Sintassi:** `osatura lod <project.json> --tolerance-px 0.5 [--scale S | --size WxH] [--levels 4] [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]`

Costruisce **livelli di dettaglio** L1..Ln, ciascuno con un *limite di errore dichiarato in pixel di output*: semplificazione dei tracciati, riduzione
delle chiavi, collasso delle mesh e culling. Il livello k ha tolleranza `--tolerance-px x ratio^(k-1)`. `--scale` o `--size` indica
a che dimensione viene mostrato l'output (predefinito 1 px per unità). `--out-dir` (predefinito `.`) riceve `<name>_L1.json` ... e
`--report` un report JSON. `--verify` renderizza `--frames N` fotogrammi e **misura** la deviazione reale (codice 1 se supera
il limite).

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
**Attenzione:** i risparmi su una piccola demo costruita a mano sono minimi (circa il 4% a 4 px); i LOD rendono su artwork tracciati o con molte mesh.
Il report e i file dei livelli vengono scritti con una scrittura semplice (non atomica).

### `lod-pick`

**Sintassi:** `osatura lod-pick <project.json> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--out chosen.json]`

Sceglie il **livello più grossolano** il cui limite dichiarato resta entro `--max-px` pixel quando l'artwork è mostrato su uno
schermo di `--screen-px` (la scala è la dimensione dello schermo diviso la dimensione del progetto). `--out` scrive quel livello.

```
$ osatura lod-pick small.json --screen-px 320x240 --out chosen.json
level  tol_px  anchors ...
screen 320x240 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.4638 px, 69 anchors, 0 mesh vertices, 37 keys)
wrote chosen.json
$ osatura lod-pick small.json --screen-px 64x48 --max-px 2 --out chosen2.json
screen 64x48 -> scale 0.1000 px/unit; budget 2 px -> level L4 (declared 0.5310 px, 62 anchors, 0 mesh vertices, 37 keys)
```

### `update-check`

**Sintassi:** `osatura update-check [--proxy-preset tor|tor-browser|warp | --proxy-url URL] [--list-proxy-presets] [--json]`

Chiede a GitHub se esiste una versione di OSATURA più recente. Parte solo quando la digiti: nulla controlla in background, nulla viene scaricato o installato, e l'unica cosa inviata è una richiesta anonima per l'elenco pubblico delle versioni (fatta dal `curl` del sistema, solo HTTPS, 30 s, al massimo 2 MB). La risposta confronta la tua versione con la più recente pubblicata secondo le regole SemVer: `0.1.0-alpha.5` è più recente di `0.1.0-alpha.4`, e `0.1.0` è più recente di `0.1.0-rc.1`.

`--proxy-preset` invia la richiesta attraverso un proxy gratuito che gira sul tuo computer: `tor` (il servizio Tor su `127.0.0.1:9050`), `tor-browser` (un Tor Browser in esecuzione, `127.0.0.1:9150`) o `warp` (Cloudflare WARP in modalità proxy, `127.0.0.1:40000`). Tutti e tre usano `socks5h://`, quindi il nome `api.github.com` è risolto dal proxy e non dal tuo sistema. Tor è la scelta consigliata (rete di volontari, senza scopo di lucro, open source); WARP è gratuito ma è un servizio commerciale con client chiuso. Gli elenchi di proxy pubblici aperti non sono offerti di proposito: sono spesso compromessi o intercettano il traffico. `--proxy-url` accetta il tuo proxy `http://`, `https://`, `socks5://` o `socks5h://`; le due opzioni non si combinano. `--list-proxy-presets` stampa i preset ed esce. (`--proxy` è un'altra opzione: il livello di proxy di dettaglio dello studio.)

```
$ osatura update-check --proxy-preset tor
update-check: going through socks5h://127.0.0.1:9050
You are up to date: 0.1.0-alpha.4 (newest published release: v0.1.0-alpha.4).
$ osatura update-check --json
{"current":"0.1.0-alpha.4","latest":"v0.1.0-alpha.5","prerelease":true,"update_available":true,"url":"https://github.com/chiuta/osatura/releases/tag/v0.1.0-alpha.5"}
```
**Attenzione:** solo le build prodotte dal workflow di rilascio conoscono la propria versione; una build dai sorgenti (versione `0.1.0`) stampa l'ultima versione ma non può dire se è più recente, e `--json` ha allora `"current": null` e `"update_available": null`. Codice di uscita 1 con un motivo su una riga quando `curl` manca o GitHub non è raggiungibile (le uscite Tor a volte sono bloccate o limitate). L'indirizzo di download stampato è sempre un link `github.com/chiuta/osatura`.

### `export-fit`

**Sintassi:** `osatura export-fit <project.json> <out> --max-size 500KB [--format lottie|html|gif|apng|webm|project] [--min-ssim X] [--max-seconds 60] [--measure raw|gzip] [--frames N] [--search ladder|descent|grid] [--report out.json] [--pareto out.csv] [--audit] [--no-baseline]`

Trova l'**export di qualità migliore che rientra in un budget di byte**: cerca tra i parametri di qualità dell'esportatore (Lottie: `decimals`,
`path_eps`, `key_tol`, `frame_step`; gli altri formati hanno i propri) e valuta ogni candidato con SSIM rispetto al rendering
originale. `--max-size` accetta `500KB` (1000 byte), `1.5MB`, `64000`, `200KiB` (1024 byte). Il formato deriva da
`--format` o dall'estensione di output (`.json` è Lottie, `.html`, `.gif`, `.apng`, `.webm`); `project` scrive un JSON di progetto
alleggerito. `--measure gzip` calcola il budget sulla dimensione gzip (per i server che comprimono). `--min-ssim` è una soglia minima di qualità.
`--max-seconds` è un **budget di tempo reale** (predefinito 60; `0` = illimitato): alla scadenza viene mantenuto il miglior risultato ottenuto fino a quel momento.
`--frames N` indica quanti fotogrammi vengono campionati per l'SSIM (predefinito 8). `--search` sceglie la strategia (`descent` predefinita).
`--report` / `--pareto` scrivono il resoconto della ricerca come JSON / CSV; `--audit` stampa ogni candidato; `--no-baseline` salta
il confronto con le impostazioni predefinite.

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
**Codici di uscita:** 0 quando il risultato rientra nel budget (anche se il budget di tempo ha interrotto la ricerca); 1 quando il budget
non può essere rispettato (l'export più piccolo viene comunque scritto) o in caso di argomenti errati. **Attenzione:** lavora sull'output baked,
quindi non può rimuovere più di quanto consentano i parametri (un Lottie da 1 KB è impossibile per un rig di 14 livelli); ogni candidato è un
export completo, quindi i progetti grandi richiedono un `--max-seconds` generoso (una build di debug è molto più lenta di una build release). I file
di report sono scritti con una scrittura semplice. **Correlati:** `size-report`, `lod`, `export` (sezione 4), EXPORT_FORMATS.md (consigli sulle dimensioni).

## 14. Controllo qualità

Correlati, documentati in precedenza: `check` (validazione, sezione 3), `diff` (diff semantico, sezione 3), `compare` (diff di immagini di due
render, sezione 4).

### `lint`

**Sintassi:** `osatura lint <project.json>`

Controlli statici su rig e animazione. Stampa una riga per ogni rilievo: `Severity CODE [target:property] message`. Codice di uscita 1 se un
rilievo ha severità Error o superiore; Info e Warning terminano con 0.
```
$ osatura lint small.json
Info L007 [:CameraX] key at frame 24 lies outside the 0..8 timeline
Info L007 [arm_low:BoneAngle] key at frame 12 lies outside the 0..8 timeline
```
`L007` è una chiave fuori dalla timeline (qui perché `small.json` è la demo tagliata a 8 fotogrammi).

### `contracts`

**Sintassi:** `osatura contracts <project.json> <contracts.json>`

**Contratti di movimento**: unit test per l'animazione. `contracts.json` è un elenco di contratti con un `type`:
`loop_seamless {tol}`, `tip_range {bone,axis,min,max}`, `max_speed {bone,px_per_frame}` (esistono altri tipi nell'enum
`Contract` del motore). Le violazioni vengono stampate su stdout, il conteggio su stderr, e il codice di uscita è 1.
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

**Sintassi:** `osatura coach <project.json>`

Suggerimenti su tempi e principi di animazione (overlap, anticipazione, easing). Solo consigli, codice di uscita sempre 0.
```
$ osatura coach small.json
C003 [, arm_low, arm_up, foot_ctrl, hand, mouth, target] frame 0: 7 parts start moving on frame 0; offset them by a few frames (overlap) for a more natural feel
```

### `safety`

**Sintassi:** `osatura safety <project.json> [--scale N]`

Report su lampeggi, lampeggi rossi e movimento della camera (un'approssimazione di WCAG 2.3.1; uno strumento di supporto tecnico, non una
certificazione per la trasmissione). Renderizza ogni fotogramma a `--scale` (usa un valore piccolo per i progetti lunghi).
```
$ osatura safety small.json --scale 0.25
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.06 widths/s, rotation 0°/s, zoom ×1.00/s
```

### `reduced-motion`

**Sintassi:** `osatura reduced-motion <project.json> <out.json> [--factor 0.25]`

Scrive una copia più calma: il movimento della camera e l'oscillazione sono scalati di `--factor` (predefinito 0.25).
```
$ osatura reduced-motion small.json calm.json --factor 0.25
wrote calm.json
```

### `golden`

**Sintassi:**
```
osatura golden write <project.json> <golden.json> [--scale N]
osatura golden check <project.json> <golden.json> [--tol T]
```
`write --scale N` accetta N in (0, 1]: un valore maggiore viene **rifiutato** (`golden write: --scale 2 is above 1 ...`, nulla viene scritto; prima veniva limitato a 1 in silenzio). `check` renderizza di nuovo alla scala memorizzata nel file golden e non ha l'opzione `--scale`: `golden check ... --scale N` viene rifiutato come opzione sconosciuta.


Un test di regressione leggero: `write` registra un hash e una miniatura 16x16 di ogni fotogramma in `golden.json`; `check`
renderizza di nuovo e confronta. **Esatto per impostazione predefinita** (`--tol 0`): qualsiasi differenza nell'hash dei pixel è un fallimento. `--tol T` (T > 0) allenta
il controllo: un hash cambiato passa comunque se la distanza tra le miniature è al massimo T. Un `--tol` negativo o non finito è un errore.
Per i golden basati su immagini con una metrica percettiva, usa `ci`.
```
$ osatura golden write small.json g.json --scale 0.25
wrote g.json (8 frames)
$ osatura golden check small.json g.json
all 8 frames match exactly
$ osatura golden check small.json g.json --tol 0.01
all 8 frames match (within thumbnail tolerance 0.01)
```
I fotogrammi che differiscono sono elencati (cambio di hash, distanza tra miniature, numero delle 256 celle della miniatura che differiscono, massima
differenza di canale su 255), e il codice di uscita è 1:
```
$ osatura golden check changed.json g.json
frame 1 differs: pixel hash changed, thumbnail distance 0.003, 12 of 256 thumbnail cells differ, max channel difference 31/255
2 frame(s) differ (exact comparison; --tol T accepts small differences)       [stderr, exit 1]
```
Un fotogramma presente da una sola parte viene segnalato come `one side only`. **Limiti:** il file golden conserva solo l'hash e la
miniatura, quindi i numeri riportati sono a livello di miniatura, non di pixel. Una modifica invisibile nel render (per esempio
un colore di sfondo completamente coperto da un livello opaco, come il cielo della demo) lascia legittimamente invariato l'hash.

### `ci` (init, run, bless, diff)

**Sintassi:**
```
osatura ci init <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run  [manifest] [--update] [--bless] [--prune] [--only a,b] [--junit f.xml] [--html f.html] [--json f.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest]
osatura ci diff a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--ignore x,y,w,h;...] [--aa N] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```
**CI per l'animazione**: fotogrammi golden confrontati con una metrica percettiva (SSIM sulla luma, CIEDE2000), più controlli strutturali e
budget. Il formato del manifest e il flusso di lavoro sono descritti in [CI.md](CI.md). `exit 0` = superato, `1` = un controllo è fallito,
`2` = errore (manifest illeggibile, ...).
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
`init` scrive il manifest e, a meno di `--no-goldens`, i PNG golden nella cartella `goldens/` accanto ad esso. `run
--update` riscrive i golden mancanti o cambiati, `--bless` accetta i nuovi render come golden, `--prune` rimuove i
file golden che nessun controllo referenzia più, `--only a,b` esegue i controlli indicati. `bless` equivale a `run --bless`. `diff` confronta due PNG: `--tol` è la soglia
della metrica, `--ignore` maschera rettangoli (`x,y,w,h;...`), `--aa N` perdona le differenze di anti-aliasing, e `--out`,
`--side`, `--overlay`, `--flicker` scrivono una heat map, un'immagine affiancata, una sovrapposizione e una GIF a lampeggio.
**Attenzione:** i report (`--junit`, `--html`, `--json`) e i file golden sono scritti con lo scrittore atomico che rifiuta i symlink (creando
le cartelle padre); un percorso di report che è un symlink viene rifiutato dopo l'esecuzione dei controlli (i controlli sono lenti, quindi passa un percorso reale). Una build di debug è lenta: gli 8 controlli sopra hanno richiesto 15 s su un progetto di 8 fotogrammi.

### `fuzz`

**Sintassi:** `osatura fuzz <project.json> [--samples N] [--seed S] [--strategy random|lhs|halton|sweep|corners|pca] [--frame F] [--report out.json] [--heatmap out.svg] [--minimize] [--gate] [--max-fail-rate R] [--props angle,scale,offx,offy] [--only BONES] [--margin DEG] [--db posedb.json] [--ignore KINDS] [--no-deformers]`

**Pose fuzzing**: campiona pose entro i limiti degli ossi e riporta dove il rig si rompe (pieghe, stiramenti, spigoli,
auto-intersezioni, IK irraggiungibile, violazioni dei limiti, NaN, fuori dal canvas). `--strategy` sceglie il campionatore (`pca` campiona lungo gli assi PCA delle pose chiave del progetto stesso, o del database di pose
indicato con `--db`), `--minimize` riduce una posa che fallisce a un riproduttore minimo, `--heatmap` scrive una mappa di stress,
`--ignore` salta i tipi di errore, `--only` limita agli ossi. Senza gate, `fuzz` è un **report**: stampa il riepilogo e termina con 0 (`(report only: exit 0; add --gate, ...)`). `--gate` fallisce (codice 1) quando un qualsiasi campione fallisce; `--max-fail-rate R` (una **frazione** 0..1) implica il gating a quella soglia. (Il controllo `fuzz` di `ci` ha una propria impostazione `gate` e non ne è influenzato.)
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
Il target IK della demo è deliberatamente raggiungibile solo in un piccolo intervallo, quindi quasi ogni posa casuale è "irraggiungibile":
considera la percentuale come un numero di regressione, non come un esito superato/fallito per questo rig. Il report e la heatmap sono scritti in modo atomico e un percorso che è un symlink viene rifiutato (a meno di `--follow-symlinks`); la cartella padre deve esistere.

### `fuzz-grid`

**Sintassi:** `osatura fuzz-grid <project.json> --x PARAM --y PARAM [--res N] out.svg|out.png|out.json`

Una mappa di severità di due parametri (`bone:NAME:PROP`, per esempio `bone:arm_up:angle`) su una griglia `N x N` (predefinito 32).
Il tipo di output segue l'estensione.
```
$ osatura fuzz-grid small.json --x bone:arm_up:angle --y bone:arm_low:angle --res 8 grid.svg
wrote grid.svg (8x8 cells, 59.4% failing; x = bone:arm_up:angle, y = bone:arm_low:angle)
```
Stessa avvertenza di `fuzz` sulla scrittura semplice.

## 15. Controllo di versione (`vcs`)

**Sintassi:**
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
Le opzioni sono per sotto-comando: `--json` solo su `log`, `status`, `diff` e `show`; `--force` solo su `checkout`; `--ours`, `--theirs`, `--value` (esattamente una) solo su `resolve`.

Una piccola cronologia indirizzata per contenuto per **un solo file di progetto** con un **merge semantico**: le modifiche sono confrontate come
percorsi del documento (JSON pointer), non come righe di testo, quindi due persone che modificano chiavi diverse non vanno mai in conflitto. `init` crea
una cartella `.osatura-vcs/` accanto al progetto; il branch predefinito è `main`. Le revisioni (`REV`) sono nomi di branch, nomi di tag
o prefissi di id di commit.

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

**Conflitti.** Quando entrambi i lati hanno modificato lo stesso valore, `merge` si ferma, mantiene il lato *nostro* nel file di progetto, elenca i
conflitti e termina con 1:
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
Finché un merge, un cherry-pick o un revert non è risolto, `checkout`, `revert` e `commit` vengono rifiutati con
`a merge / cherry-pick / revert is in progress: resolve the conflicts and commit, or run 'vcs merge --abort'`.
Altri fatti: `diff` senza argomenti confronta il file di lavoro con HEAD, `diff A B` due revisioni (`--json` stampa un elenco
di `{kind,path,description}`); `show` stampa un commit e il suo elenco di modifiche; `branch` senza argomenti elenca i branch (`*` indica quello corrente); `branch -d NAME` rifiuta di eliminare il branch attivo; `checkout --force` scarta le modifiche non committate; `revert REV` e `cherry-pick REV` creano un nuovo commit (o si fermano con conflitti come `merge`); `--repo DIR`
seleziona una cartella di repository diversa da quella corrente; `--author` è `unknown` per impostazione predefinita; `log --json` stampa gli oggetti
commit. **Limiti:** un file per repository, solo locale (nessun remote, nessun push/pull), nessun tracciamento dei rename oltre ai percorsi JSON;
le risorse binarie referenziate dal progetto non sono versionate.

## 16. Studio, collaborazione, server agente MCP e registro UI

### `studio`

**Sintassi:** `osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES] [--public-origin URL] [--source-url URL] [--collab-token T (16+ characters)] [--collab-secret-file F [--collab-revoke NAMES] [--collab-host NAMES]] [--collab-rate N] [--collab-log LOG [--collab-nosync]]`

Avvia l'**editor nel browser** (annulla/ripeti, timeline, pannelli live e del volto, VCS e operazioni in stile agente) come server web
locale e resta in attesa fino a Ctrl-C. Valori predefiniti: `--port 8080`, `--bind 127.0.0.1`. Il progetto viene validato all'avvio; i salvataggi vanno
sullo stesso file (`POST /api/save`).
```
$ osatura studio demo.json --port 18931 --ui-config ui.json
AGPL-3.0 source offer: http://127.0.0.1:18931/source
OSATURA studio on http://127.0.0.1:18931  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in ui.json
```
Opzioni: `--ui-config F` conserva layout e scorciatoie (predefinito `~/.config/osatura/ui.json`, su Windows `%APPDATA%\OSATURA\ui.json`); `--source-url URL` imposta il
link al sorgente pubblicato mostrato su `/source` e da `--version` (AGPL-3.0 sezione 13; i binari di release ufficiali usano come predefinito il commit da cui sono stati compilati); `--allow-host NAMES` e `--public-origin URL` accettano
richieste sotto un nome pubblico dietro un reverse proxy TLS; `--collab-token T` (almeno 16 caratteri, per esempio
`openssl rand -hex 16`) abilita la sessione multiutente ed è **obbligatorio per qualsiasi `--bind` non loopback**; `--collab-nosync` è valido solo insieme a `--collab-log` (altrimenti è un errore); `--collab-log LOG`
rende persistente la sessione condivisa e la riprende dopo un riavvio (`--collab-nosync` salta la sincronizzazione all'avvio).

**Opzioni di hardening** (wave 7; dettagli e threat model in [SECURITY.md](SECURITY.md)): `--collab-secret-file F` sostituisce
l'unico bearer condiviso con **token per singolo partecipante**: F contiene un segreto casuale (prima riga, almeno 16 caratteri, modalità 600 su
Unix, altrimenti viene rifiutato) e ogni persona ottiene un token da `osatura collab-token NAME`; un token è legato a un solo nome
(l'accesso usa quel nome, nessuno può entrare come qualcun altro), scade, e viene verificato ricalcolandone l'HMAC-SHA-256, così il
server non conserva alcuna tabella. `--collab-revoke NAMES` (separati da virgola, richiede `--collab-secret-file`) rifiuta i token di quei
nomi prima che scadano (un server in esecuzione può anche revocare con `POST /api/collab/revoke {"names":["ana"]}`, vedi sotto). `--collab-host NAMES`
(separati da virgola, richiede `--collab-secret-file`) indica i partecipanti i cui token possono **salvare il progetto, compattare il log di sessione e revocare
i token**; tutti gli altri con un token valido possono comunque modificare, ma per quelle tre richieste ricevono `403`. Senza `--collab-host` ogni partecipante può
salvare e compattare (come prima) e nessuno oltre al `--collab-token` condiviso / a una sessione locale non autenticata può revocare via HTTP; lo studio
lo indica all'avvio. `--collab-token` e `--collab-secret-file` possono essere combinati (il token condiviso funge allora da
chiave di amministrazione); l'uno o l'altro soddisfa la regola per cui un bind non loopback richiede una credenziale. `--collab-rate N` limita ogni indirizzo
client a N richieste al secondo (raffiche di 2N; risposta `429` con `Retry-After`); il valore predefinito è 100 quando è richiesta una credenziale
e nessun limite altrimenti; 0 lo disattiva. Un bind non loopback stampa avvisi (HTTP in chiaro; tutte le interfacce per
`0.0.0.0` / `::`; un token condiviso non ha nomi né scadenza).

**Regole di sicurezza** (verificate): il server risponde solo alle richieste il cui `Host` è uno dei propri nomi
(`curl -H 'Host: evil.example' .../api/state` dà 421); `Origin` e `Sec-Fetch-Site` da un'altra pagina danno 403; il corpo di un POST
deve essere `application/json` (`curl -X POST -d '{}'` dà 415). Un token breve viene rifiutato
(`--collab-token must be at least 16 characters`), e `--bind 0.0.0.0` senza token viene rifiutato
(`refusing to listen on 0.0.0.0:PORT: the studio has no accounts and no TLS, ...`). Il server non ha account né TLS: metti
un proxy TLS davanti per qualsiasi uso diverso dalla tua macchina; quando `--public-origin` è un URL `https://`, il cookie di login riceve l'attributo `Secure` per le richieste che arrivano sotto quel nome (o con `Origin` https / `X-Forwarded-Proto: https`). La pagina serve una piccola favicon SVG su `/favicon.ico`. L'API HTTP (per esempio `GET /api/state`, `GET /api/project`,
`POST /api/op`, `POST /api/save`, `GET /frame/N.svg`) esiste per la pagina stessa e non è un'API pubblica stabile.

### `collab-token`

**Sintassi:** `osatura collab-token <name> [--secret-file F] [--ttl 28800]`

Stampa un token di collaborazione per singolo partecipante per uno studio avviato con `--collab-secret-file F`: `ap1.<name>.<expires>.<mac>`,
dove `mac` è l'HMAC-SHA-256 in esadecimale del nome e della scadenza sotto il segreto. `<name>` è di 1-32 caratteri tra lettere,
cifre, `_` e `-`. `--ttl` è la durata in secondi (predefinito 28800 = 8 ore, al massimo 2592000 = 30 giorni). Il segreto viene letto dal
file `F` (prima riga) oppure, senza `--secret-file`, dalla variabile d'ambiente `OSATURA_COLLAB_SECRET`; non è mai
accettato come argomento, dove altri utenti potrebbero vederlo. Il token va su stdout, la scadenza su stderr.

```
$ openssl rand -hex 32 > collab.secret && chmod 600 collab.secret
$ osatura studio demo.json --collab-secret-file collab.secret
$ osatura collab-token ana --secret-file collab.secret --ttl 3600
ap1.ana.1790000000.3f0c...
token for 'ana', valid until Unix time 1790000000 (in 3600 s); give it to that person only; ...
$ osatura collab-token ana --secret-file collab.secret --ttl 0
--ttl needs a whole number of seconds (1 to 2592000)
```
La persona incolla il token nella pagina di login dello studio (viene poi conservato in un cookie HttpOnly che scade insieme al token) oppure lo invia come
`X-Collab-Token`. Un token scaduto riceve risposta `401` con la scadenza nel messaggio; il nome del partecipante viene preso dal
token. **Correlati:** `studio`.

### `collab-sim`

**Sintassi:** `osatura collab-sim <project.json> [--clients N] [--ops M] [--seed S] [--undo-pct P] [--replays K] [--require-valid] [--raw] [--json]`

Un banco di prova per il motore di collaborazione: N repliche modificano il progetto in modo concorrente con ordine di consegna casuale e
duplicati, poi verifica che tutte contengano lo stesso documento (codice 1 in caso contrario). Le modifiche predefinite sono modifiche valide del progetto
(numeri, valori delle chiavi, visibilità, livelli propri); `--raw` usa modifiche JSON arbitrarie (di solito non un progetto valido);
`--require-valid` fa terminare con 1 anche un risultato che non compila.
```
$ osatura collab-sim small.json --clients 3 --ops 20 --seed 2 --json
{"buffered":12,"clients":3,"clocks_identical":true,"converged":true,"converged_ok":true,"deliveries":34,"duplicates_dropped":0,"edits":20,"envelopes":17,...}
```
`converged_ok` indica la sola convergenza; `ok` richiede anche che il documento unito compili.

### `mcp`

**Sintassi:** `osatura mcp <project.json>`

Un **server per agenti**: JSON-RPC 2.0 (il Model Context Protocol) su stdin/stdout, un messaggio JSON per riga. Carica il
progetto ed espone gli strumenti `get_project`, `stats`, `lint`, `coach`, `safety`, `apply_ops`, `set_key`, `undo`, `redo`,
`render_frame` e `save`. Nulla viene scritto finché non si chiama `save`. Stdout trasporta solo messaggi del protocollo.
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
Il server termina (codice 0) quando stdin si chiude.

### `ui`

**Sintassi:**
```
osatura ui commands|shortcuts [--ui-config F] [--json]
osatura ui layout presets|show <name>|validate <file.json>
osatura ui search <query...> [--ui-config F]
```
Il **registro dei comandi** dello studio: `commands` elenca ogni comando con categoria, titolo e associazioni di tasti;
`shortcuts` elenca le scorciatoie effettive (quelle predefinite più quelle in `--ui-config`) ed eventuali conflitti; `layout presets` elenca i
preset di layout dei dock, `layout show NAME` ne stampa uno come JSON e `layout validate FILE` controlla un file di layout;
`search` ordina i comandi come fa la palette dei comandi.
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

## 17. Database di pose, suggerimenti e altri strumenti di supporto

Questi comandi costruiscono un **database di pose** (rotazioni degli ossi ai fotogrammi chiave di un progetto e di ogni take di una libreria)
e usano la sua struttura appresa (assi PCA, piccola rete opzionale) per suggerire pose. È un aiuto statistico addestrato sulle
*tue* pose, non un modello generale. I loro file di modello e di report sono scritti con una scrittura semplice.

### `pose-db` (build, info, train-mlp)

**Sintassi:**
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
`--project` aggiunge le pose ai fotogrammi chiave del progetto, `--library` i take di una libreria di take (`take-record`, sezione 6),
`--step N` campiona in aggiunta ogni N fotogrammi. La piccola rete (SGD semplice) è facoltativa; usala con `--model` più sotto.

### `pose-suggest` (between, complete, next)

**Sintassi:**
```
osatura pose-suggest between  <project.json> <db.json|-> --a F1 --b F2 [--t 0.5] [--snap 0.5] [--top 5] [--apply RANK --out out.json] [--json]
osatura pose-suggest complete <project.json> <db.json|-> --frame F [--fix bone=angle,...] [--reach bone=x,y,...] [--free b1,b2] [--prior 1] [--model m.json] [--apply RANK --out out.json]
osatura pose-suggest next     <project.json> <db.json|-> --frame F [--k 8]
```
`-` al posto di un database apprende dal progetto stesso. `between` propone una posa intermedia tra due fotogrammi che resta
sulla varietà appresa (candidati `linear`, `pca`, `manifold`); `complete` trova la posa completa più plausibile dati angoli
fissi (`--fix`), bersagli delle punte (`--reach`) e ossi liberi; `next` mostra cosa ha fatto la libreria dopo pose come quella al
fotogramma indicato. `--apply RANK` (con `--out`) scrive il candidato scelto come chiavi.
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

**Sintassi:** `osatura pose-bench`

Stampa l'accuratezza su dati esclusi dall'addestramento dei metodi di suggerimento su una **libreria sintetica di tipo andatura** (24 cicli di addestramento, 6
cicli esclusi; il completamento vede 3 ossi su 10; addestra anche la piccola rete). Più basso è meglio. Non legge alcun progetto. È
**lento in una build di debug** (minuti); numeri da una build release:
```
completion  mean pose                                   20.96 deg RMS
completion  nearest library pose                         1.77 deg RMS
completion  PCA least squares                            0.51 deg RMS
completion  MLP                                          1.64 deg RMS
between     linear blend                                 1.83 deg RMS
next        Markov mean                                  0.72 deg RMS
```

### `pose-match`

**Sintassi:** `osatura pose-match <project.json> <params.json> <goals.json> <out.json> [--generations N --seed N]`

Cerca (un ottimizzatore evolutivo, predefiniti 80 generazioni, seed 1, intervallo valido 1..100000) i valori di un insieme di
parametri che portano le punte degli ossi nelle posizioni obiettivo nello spazio dello stage. `params.json` è un elenco di
`{"target","prop","min","max","start"}`; `goals.json` è un elenco di `{"bone","at":[x,y],"weight"}`. Il risultato viene aggiunto in coda come
canali; la perdita è la somma pesata degli errori quadratici delle punte in px².
```
$ echo '[{"target":"arm_up","prop":"bone_angle","min":-60,"max":40,"start":0},{"target":"arm_low","prop":"bone_angle","min":-60,"max":60,"start":0}]' > params.json
$ echo '[{"bone":"hand","at":[260,200],"weight":1.0}]' > goals.json
$ osatura pose-match demo.json params.json goals.json out.json --generations 30 --seed 3
wrote out.json (loss 3756.156 px²)
```
**Attenzione:** la perdita nell'esempio è grande perché gli intervalli non possono raggiungere l'obiettivo: controlla il numero, non solo il
codice di uscita (un risultato viene scritto anche quando l'abbinamento è scarso).
## Discrepanze note tra documentazione e comportamento

Dove il testo d'uso del programma (stampato da `osatura` senza argomenti) o un documento vicino dice una
cosa e il programma ne fa un'altra, le sezioni precedenti descrivono ciò che il programma fa davvero. Le differenze trovate
verificando ogni esempio, in ordine di quanto è probabile che causino problemi:

1. **Risolto - flag sconosciuti.** Ogni comando con una riga d'uso ora rifiuta i `--words` non documentati (con un suggerimento "did you mean"),
   inclusi `check`, `stats`, `lint`, `coach`, `contracts` e `machine list`. `--json`, `--quiet` e `--allow-empty` sono
   per singolo comando e rifiutati altrove (una modifica incompatibile per gli script che li passavano ovunque). Dalla wave 5 il controllo è **per sotto-comando** (`ci diff ... --quiet`, `trail <bone> ... --json`, `ui layout presets --json`, `golden write --tol` vengono rifiutati) e copre anche le opzioni con valore. Test: `osatura-cli/tests/doc_mismatch.rs` (matrice di accettazione) e `options_are_validated_per_sub_command` in `main.rs`.
2. **Risolto - output con `fs::write` semplice.** `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, file di pose, snapshot,
   `vcs` e il log di collaborazione usano lo scrittore atomico che rifiuta i symlink; gli output di ffmpeg sono controllati in anticipo. Un test guidato da tabella
   copre circa 24 comandi. Non coperti end to end: `face-train`/`face-eval` (unit test solo sul loro scrittore).
3. **Risolto - `golden check`** è esatto per impostazione predefinita; `--tol` lo allenta; stampa il numero di differenze e la differenza maggiore.
   L'affermazione secondo cui "una modifica del colore di sfondo passa" era un artefatto della demo (il livello del cielo copre lo sfondo), non un bug.
4. **Risolto - codice di uscita di `fuzz`.** Il valore predefinito è solo report (codice 0); `--gate` o `--max-fail-rate` attivano il gating. Vedi `fuzz`.
5. **Risolto - `dof-export`** scrive `.mp4` e `.webm` (tramite ffmpeg, con `--codec`, senza audio); il testo d'uso ora li elenca. Un nome
   con qualsiasi altra estensione viene rifiutato e nulla viene scritto.
6. **Risolto - `export` `.png`** scrive `<name>.atlas.json`, mai `<name>.json`; l'errore di formato sconosciuto elenca ogni estensione.
7. **Valori predefiniti che il testo d'uso non indica:** `studio` è in ascolto su `127.0.0.1:8080`; `fuzz-grid --res` è 32; `export-fit`
   campiona 8 fotogrammi, cerca con `descent` e si ferma dopo 60 s (`--max-seconds 0` = illimitato); `reduced-motion --factor`
   è 0.25; `size-report --tol` è 0.05; `pose-match` esegue 80 generazioni con seed 1; `lod --out-dir` è la cartella
   corrente.
8. **`machine remove` può rifiutare** (`binding 'goBind' still drives machine 'm'`); il testo d'uso non menziona il
   controllo delle dipendenze. `machine` e `state-bake` leggono due formati di machine diversi (`layers`/`inputs` contro
   `initial`/`states`); la riga d'uso di `state-bake` non ha descrizione e non indica il formato.
9. **Risolto - `stats`** non accetta più `--json` (l'output è sempre JSON; la riga d'uso lo dice).
10. **`vcs`:** il branch predefinito è `main` (`checkout master` fallisce con `unknown revision 'master'`); `vcs resolve`
    accetta il percorso del conflitto *con* la barra iniziale esattamente come lo stampa `merge` (`/background`), `background` da solo dà
    `no conflict at 'background'`; il testo d'uso scrive "see: osatura vcs" ma `osatura vcs` senza argomenti stampa un
    uso di una riga e termina con 1.
11. **Righe d'uso senza descrizione.** `state-bake`, `pose-match`, `pose-bench`, `follow`, `keys`, `event`, `marker`,
    `idle`, `mouth`, `beats`, `export`, `frames`, `onion` e altri hanno una sintassi ma nessuna spiegazione, oppure una descrizione che
    sta su una riga di continuazione; il loro comportamento in questo documento deriva dalla loro esecuzione.
12. **Nomi delle proprietà.** I messaggi e gli output di `list` stampano le proprietà in CamelCase (`LayerOpacity`, `BoneAngle`),
    ma ogni opzione, file di mapping e file di appoggio accetta i nomi in snake_case (`layer_opacity`, `bone_angle`); la forma CamelCase
    viene rifiutata (`bind add ... sun:LayerOpacity` fallisce con `unknown property 'LayerOpacity'`).
13. **`cloth`:** il testo d'uso parla di "per-vertex deformation"; il risultato baked è memorizzato come canali `lattice_dx` / `lattice_dy`
    sui punti del reticolo del cloth (per la griglia 7x7 predefinita, i 42 punti non bloccati), vedi sezione 7.
14. **`render-gpu`** esiste solo nelle build realizzate con `--features gpu`; non è stato possibile eseguirlo nella build di test, quindi le sue opzioni
    sono prese dal testo d'uso e non sono verificate qui.
15. **Lento nelle build di debug.** `pose-bench`, `face-train`, `ci run` ed `export-fit` richiedono minuti in una build non ottimizzata.
    I tempi di questo documento provengono dal binario di debug e sono molto più brevi in una build release.
16. **Risolto (wave 5) - `golden write --scale`** superiore a 1 veniva prima limitato a 1 in silenzio; ora viene rifiutato (la scala è in (0, 1]).
    `golden check` ignorava un `--scale` e usava quello memorizzato; ora rifiuta l'opzione.
17. **Risolto (wave 5) - sintassi.** Ogni opzione del testo d'uso di un comando è nella sua sintassi qui e viceversa (test
    `cli_reference_synopses_match_usage`), e ogni riga d'uso senza alternative è ripetuta qui parola per parola (test
    `cli_reference_repeats_usage_lines_verbatim`). `render-gpu --features gpu` e `layer-normal ... relight --layer-normals`
    nelle descrizioni d'uso indicano una funzionalità di build e l'opzione di un altro comando, non opzioni di quei comandi.
18. **Altri documenti sono più vecchi del programma in due punti** (vedi [EXPORT_FORMATS.md](EXPORT_FORMATS.md) sezione 17):
    `docs/LIMITATIONS.md` L-29 dice che l'import Lottie non accetta immagini, effetti, precomposizioni o matte (l'importatore ora li gestisce,
    con gli avvisi elencati nella sezione 9), e L-32 elenca OCA tra i formati di import mentre la CLI ha solo
    `export-oca`.
## Indice alfabetico dei comandi

Ogni comando con un riepilogo di una riga e la sezione che lo documenta. I sotto-comandi (`machine add`, `vcs commit`, ...) sono nella sezione del comando padre.

| Comando | Riepilogo | Sezione |
|---|---|---|
| `--version` | stampa la versione, la licenza, l'offerta del sorgente e i link di supporto | [1](#1-convenzioni-comuni-a-tutti-i-comandi) |
| `align` | allinea i livelli tra loro o allo stage | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `audio-react` | il volume sonoro pilota una proprietà | [8](#8-audio-lip-sync-e-suoni) |
| `autorig` | IK, respirazione e ammiccamento impostati a partire dai nomi di ossi e livelli | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `autotrace` | tracciamento locale di bitmap (bande di colore, fitting di Bezier) | [9](#9-import) |
| `autotrace-bench` | tabella di fedeltà del tracciatore su forme sintetiche | [9](#9-import) |
| `beats` | rileva i salti di volume e registra marker o eventi | [8](#8-audio-lip-sync-e-suoni) |
| `bind` | data binding: add, list, remove | [11](#11-state-machine-data-binding-container-e-regioni-di-scorrimento) |
| `bind-eval` | valuta i binding per valori dati | [11](#11-state-machine-data-binding-container-e-regioni-di-scorrimento) |
| `bone-vis` | mostra un osso solo negli overlay per un intervallo di fotogrammi | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `cage` | aggiunge un deformatore a gabbia con coordinate a valore medio | [7](#7-fisica-cloth-e-deformatori) |
| `cage-pose` | imposta chiavi sugli offset dei vertici della gabbia a un fotogramma | [7](#7-fisica-cloth-e-deformatori) |
| `camera-seq` | anima la camera attraverso inquadrature salvate | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `chan-meta` | colore del canale, flag shy, solo e blocco | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `check` | valida un progetto (codice 1 se non valido) | [3](#3-creazione-e-ispezione-dei-progetti) |
| `ci` | CI per l'animazione: golden, diff percettivo, controlli strutturali | [14](#14-controllo-qualità) |
| `clip` | sovrappone un take sulla pila di clip | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `cloth` | esegue il bake di una simulazione deterministica di tessuto | [7](#7-fisica-cloth-e-deformatori) |
| `coach` | suggerimenti su tempi e principi | [14](#14-controllo-qualità) |
| `collab-sim` | test di convergenza della collaborazione | [16](#16-studio-collaborazione-server-agente-mcp-e-registro-ui) |
| `collab-token` | genera un token di collaborazione per partecipante (HMAC-SHA-256, con scadenza) | [16](#16-studio-collaborazione-server-agente-mcp-e-registro-ui) |
| `color-space` | sceglie la composizione in sRGB o in luce lineare per il progetto | [12](#12-effetti-profondità-di-campo-onion-skin-scie-e-camera) |
| `comp` | stati di livello con nome (visibilità, opacità, scelta dello switch) | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `compare` | immagine prima / dopo / differenza di due progetti | [4](#4-rendering-ed-esportazione) |
| `container` | container di layout: add, list, remove, solve | [11](#11-state-machine-data-binding-container-e-regioni-di-scorrimento) |
| `contracts` | contratti di movimento (unit test per l'animazione) | [14](#14-controllo-qualità) |
| `demo` | scrive il progetto demo di rig integrato | [3](#3-creazione-e-ispezione-dei-progetti) |
| `demo-fx` | scrive la demo di liquido / booleane / particelle | [3](#3-creazione-e-ispezione-dei-progetti) |
| `diff` | differenze semantiche tra due progetti | [3](#3-creazione-e-ispezione-dei-progetti) |
| `distribute` | distribuisce uniformemente tre o più livelli | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `dof` | render con profondità di campo di un fotogramma | [12](#12-effetti-profondità-di-campo-onion-skin-scie-e-camera) |
| `dof-export` | profondità di campo animata (GIF/APNG/video/cartella) | [12](#12-effetti-profondità-di-campo-onion-skin-scie-e-camera) |
| `drive` | segnali CSV -> chiavi tramite file di mapping | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `event` | aggiunge un evento di runtime con nome (con payload JSON facoltativo) | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `explain` | perché una proprietà ha il suo valore: contributi ordinati | [3](#3-creazione-e-ispezione-dei-progetti) |
| `export` | esporta GIF, APNG, WebM/MP4, sheet PNG, HTML, Lottie, .lottie, glTF/GLB, ... | [4](#4-rendering-ed-esportazione) |
| `export-fit` | export di qualità migliore che rientra in un budget di byte | [13](#13-livello-di-dettaglio-report-sulle-dimensioni-ed-export-fit) |
| `export-oca` | scrive una cartella Open Cel Animation | [4](#4-rendering-ed-esportazione) |
| `eye-clip` | maschera i livelli di iride/pupilla con la forma dell'occhio | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `face-bake` | landmark -> chiavi | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `face-calibrate` | calibrazione del volto neutro | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `face-detect` | riquadri dei volti e 23 landmark in un'immagine | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `face-eval` | report di accuratezza del rilevatore su dati esclusi dall'addestramento | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `face-synth` | animazione sintetica di una testa | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `face-track` | NDJSON dei landmark da video o fotogrammi | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `face-train` | addestra il rilevatore di volti su volti sintetici | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `follow` | generatori di azione sovrapposta / follow-through | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `foot-lock` | blocca un controller durante il contatto con il suolo | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `frames` | renderizza ogni fotogramma in una cartella (SVG, o PNG con --png) | [4](#4-rendering-ed-esportazione) |
| `fuzz` | trova le pose in cui il rig si rompe | [14](#14-controllo-qualità) |
| `fuzz-grid` | mappa di severità di due parametri | [14](#14-controllo-qualità) |
| `fx` | renderizza un fotogramma con una catena di post-effetti | [12](#12-effetti-profondità-di-campo-onion-skin-scie-e-camera) |
| `glue` | incolla insieme i vertici di una mesh | [7](#7-fisica-cloth-e-deformatori) |
| `golden` | scrive/controlla i fotogrammi di regressione | [14](#14-controllo-qualità) |
| `idle` | genera un canale idle procedurale | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `import-anim` | GIF/APNG animata -> livello fotogramma per fotogramma | [9](#9-import) |
| `import-bvh` | motion capture BVH -> canali | [9](#9-import) |
| `import-lottie` | sottoinsieme Lottie -> progetto | [9](#9-import) |
| `import-ora` | OpenRaster -> progetto | [9](#9-import) |
| `import-seq` | sequenza PNG -> livello fotogramma per fotogramma | [9](#9-import) |
| `import-svg` | sottoinsieme SVG -> livello vettoriale | [9](#9-import) |
| `keyedit` | modifiche del dope sheet come un unico batch sicuro per l'annullamento | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `keys` | operazioni di massa sulle chiavi di tutti i canali con chiavi | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `layer-normal` | associa una normal map a un livello | [12](#12-effetti-profondità-di-campo-onion-skin-scie-e-camera) |
| `layer-pivot` | sposta il pivot di un livello senza cambiare l'immagine | [7](#7-fisica-cloth-e-deformatori) |
| `layout-preview` | renderizza con il layout risolto di nuovo per un'altra dimensione di canvas | [11](#11-state-machine-data-binding-container-e-regioni-di-scorrimento) |
| `lint` | controlli statici su rig/animazione (codice 1 in caso di errori) | [14](#14-controllo-qualità) |
| `lipsync` | pilota un livello switch della bocca dal volume dell'audio (non basato sui fonemi) | [8](#8-audio-lip-sync-e-suoni) |
| `lipsync-text` | allinea una trascrizione all'audio e ricava le forme della bocca | [8](#8-audio-lip-sync-e-suoni) |
| `live-map` | input dal vivo registrato (MIDI, mouse, gamepad, tasti) -> chiavi, take, eventi | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `lod` | livelli di dettaglio con limiti di errore in pixel dichiarati | [13](#13-livello-di-dettaglio-report-sulle-dimensioni-ed-export-fit) |
| `lod-pick` | livello più grossolano che rientra in un budget di pixel su uno schermo | [13](#13-livello-di-dettaglio-report-sulle-dimensioni-ed-export-fit) |
| `loop-solve` | rende i canali con chiavi ciclici senza giunture | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `machine` | state machine di runtime: add, list, remove, validate | [11](#11-state-machine-data-binding-container-e-regioni-di-scorrimento) |
| `machine-sim` | riproduce input temporizzati: log degli stati più fotogrammi SVG | [11](#11-state-machine-data-binding-container-e-regioni-di-scorrimento) |
| `magnet` | esegue il bake di un'attrazione di prossimità | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `make-loop` | rende un ciclo continuo senza giunture (anche sul posto) | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `marker` | aggiunge un marker o un intervallo alla timeline | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `mcp` | server per agenti (JSON-RPC su stdio) | [16](#16-studio-collaborazione-server-agente-mcp-e-registro-ui) |
| `mesh-lock` | i follower seguono la superficie di una mesh ospite | [7](#7-fisica-cloth-e-deformatori) |
| `mesh-trace` | traccia la sagoma alfa di un'immagine in una mesh triangolare con skin | [7](#7-fisica-cloth-e-deformatori) |
| `mirror` | specchia tutti i canali con chiavi sinistra<->destra | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `mocap-info` | gerarchia ed estensione di un file BVH | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `mocap-match` | abbinamento tra giunti BVH e ossi del rig | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `mocap-retarget` | esegue il bake del mocap come canali aggiunti | [10](#10-input-dal-vivo-midi-face-tracking-e-motion-capture) |
| `mouth` | importa i cue della bocca (Papagayo/Rhubarb) | [8](#8-audio-lip-sync-e-suoni) |
| `onion` | PNG o SVG onion skin | [12](#12-effetti-profondità-di-campo-onion-skin-scie-e-camera) |
| `parallax` | profondità multipiano per i livelli di primo livello | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `parent-switch` | esegue il bake di un cambio di genitore animato senza salti | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `physics` | esegue il bake di una simulazione di catena | [7](#7-fisica-cloth-e-deformatori) |
| `pick` | cosa c'è sotto un punto del canvas (prima quello in cima) | [3](#3-creazione-e-ispezione-dei-progetti) |
| `pose-bench` | accuratezza dei metodi di posa su dati esclusi dall'addestramento | [17](#17-database-di-pose-suggerimenti-e-altri-strumenti-di-supporto) |
| `pose-db` | database di pose: build, info, train-mlp | [17](#17-database-di-pose-suggerimenti-e-altri-strumenti-di-supporto) |
| `pose-match` | adatta i parametri affinché le punte degli ossi raggiungano gli obiettivi | [17](#17-database-di-pose-suggerimenti-e-altri-strumenti-di-supporto) |
| `pose-suggest` | suggerimenti di posa intermedia, di completamento e successiva | [17](#17-database-di-pose-suggerimenti-e-altri-strumenti-di-supporto) |
| `proxy-bench` | millisecondi per fotogramma a ogni livello di proxy | [4](#4-rendering-ed-esportazione) |
| `reduced-motion` | scrive una copia più calma di un progetto | [14](#14-controllo-qualità) |
| `reference` | segna un livello come riferimento per il tracciamento (mai esportato) | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `render` | renderizza un fotogramma in PNG o SVG (proxy di scrub, ossi, scala) | [4](#4-rendering-ed-esportazione) |
| `render-gpu` | render GPU (solo build wgpu) | [4](#4-rendering-ed-esportazione) |
| `render-hdr` | render float in OpenEXR o PNG a 16 bit | [4](#4-rendering-ed-esportazione) |
| `reparent` | cambia il genitore di un osso mantenendo la posa di riposo | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `retarget` | copia l'animazione tra rig per coppie di catene | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `safety` | report su lampeggi / lampeggi rossi / movimento della camera | [14](#14-controllo-qualità) |
| `script` | esegue uno script Rhai in sandbox che modifica il progetto | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `scroll` | regioni di scorrimento: add, list, remove, eval | [11](#11-state-machine-data-binding-container-e-regioni-di-scorrimento) |
| `select-box` | selezione a riquadro | [3](#3-creazione-e-ispezione-dei-progetti) |
| `shot` | salva un'inquadratura della camera | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `size-report` | dove sono i byte di un progetto/export | [13](#13-livello-di-dettaglio-report-sulle-dimensioni-ed-export-fit) |
| `skin` | skin con nome: insiemi di livelli, ossi solo-skin, vincoli, catene IK | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `skin-diffuse` | pesi di skin per diffusione del calore | [7](#7-fisica-cloth-e-deformatori) |
| `slider` | crea un osso controller che fonde pose memorizzate | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `snapshot` | cronologia di revisioni numerate di un progetto in una cartella (save, list, restore) | [3](#3-creazione-e-ispezione-dei-progetti) |
| `soft-scale` | scala proporzionale di una selezione | [7](#7-fisica-cloth-e-deformatori) |
| `sound` | tracce sonore del progetto: list, add, edit, remove, mix | [8](#8-audio-lip-sync-e-suoni) |
| `speech-motion` | cenni del capo e sopracciglia sollevate additivi temporizzati sulle parole allineate | [8](#8-audio-lip-sync-e-suoni) |
| `spring-fit` | adatta frequenza/smorzamento della molla a una curva | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `state-bake` | esegue il bake di una state machine basata sui fotogrammi in chiavi | [11](#11-state-machine-data-binding-container-e-regioni-di-scorrimento) |
| `stats` | conteggi di ossi, livelli, canali, chiavi (JSON) | [3](#3-creazione-e-ispezione-dei-progetti) |
| `stretch` | esegue il bake di un follower che si allunga per raggiungere | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `studio` | editor nel browser (server web locale) | [16](#16-studio-collaborazione-server-agente-mcp-e-registro-ui) |
| `tag` | propone nomi di ruolo standard per ossi e livelli | [3](#3-creazione-e-ispezione-dei-progetti) |
| `take-apply` | applica un take della libreria con cross-fade | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `take-list` | elenca i take di una libreria | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `take-record` | registra un take in una libreria | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `timecode` | fotogramma <-> timecode SMPTE (con gestione del drop-frame) | [3](#3-creazione-e-ispezione-dei-progetti) |
| `timeline` | tracce per canale con chiavi, blocchi e colori | [3](#3-creazione-e-ispezione-dei-progetti) |
| `trace` | traccia il canale alfa di un PNG in un tracciato vettoriale | [9](#9-import) |
| `trail` | scia di movimento di un osso, livello o punto | [12](#12-effetti-profondità-di-campo-onion-skin-scie-e-camera) |
| `trim` | esegue il bake di un trim path in un livello vettoriale | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `ui` | registro dei comandi dello studio, scorciatoie, layout, ricerca | [16](#16-studio-collaborazione-server-agente-mcp-e-registro-ui) |
| `update-check` | chiede a GitHub se esiste una versione più recente; proxy gratuito opzionale | [13](#13-livello-di-dettaglio-report-sulle-dimensioni-ed-export-fit) |
| `vcs` | controllo di versione con merge semantico per un file di progetto | [15](#15-controllo-di-versione-vcs) |
| `view-swap` | pilota un livello switch da una proprietà di un osso (funzione a gradino) | [5](#5-rigging-skin-ik-e-strumenti-di-layout) |
| `walk` | genera un ciclo di camminata | [6](#6-animazione-chiavi-timeline-e-generatori) |
| `warp-pivot` | ruota/scala/sposta una selezione di punti di controllo | [7](#7-fisica-cloth-e-deformatori) |
| `waveform` | picchi min/max per il disegno | [8](#8-audio-lip-sync-e-suoni) |
