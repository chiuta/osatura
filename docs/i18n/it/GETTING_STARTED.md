# Primi passi con OSATURA

Questa guida ti accompagna da un checkout appena creato fino a un piccolo personaggio animato che hai
costruito, dotato di rig, animato con chiavi, messo in loop, renderizzato ed esportato usando solo la
riga di comando. Ogni comando qui sotto è stato eseguito davvero mentre scrivevamo questa pagina; gli
output mostrati sono quelli che sono stati stampati.

Come è stato verificato: Linux, `rustc 1.97.0`, `ffmpeg 6.1.1` e la build **debug** del binario
(`cargo build -p osatura-cli`). Una build release esegue lo stesso codice più velocemente. La suite di test passa anche su macOS e Windows in GitHub Actions; i comandi di questa pagina sono stati eseguiti a mano solo su Linux. Nel
repository non è dichiarata alcuna versione minima supportata di Rust; la 1.97.0 è semplicemente il
compilatore che è stato provato.

Licenza della documentazione: CC-BY-4.0. Vedi anche [RIGGING_GUIDE.md](RIGGING_GUIDE.md),
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md), [STUDIO.md](STUDIO.md), [FAQ.md](FAQ.md) e l'elenco completo
dei comandi in [CLI_REFERENCE.md](CLI_REFERENCE.md).

## 1. Cosa ti serve

| Requisito | Serve per | Note |
|---|---|---|
| Toolchain Rust (`rustc`, `cargo`) | compilare OSATURA | installala da <https://rustup.rs>; canale stable |
| Un browser web | lo studio (`osatura studio`) | è stato provato solo Chromium, vedi [STUDIO.md](STUDIO.md) |
| `ffmpeg` nel `PATH` (opzionale) | `.mp4`, `.webm`, mixaggio del suono in video/HTML, `sound mix`, `face-track` su un file video | tutto il resto funziona senza |

Senza `ffmpeg` i comandi che ne hanno bisogno si fermano con un messaggio chiaro e non scrivono nulla:

```
$ PATH=/nonexistent osatura export buddy-final.json o.mp4
cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead
```

Non è richiesta alcuna GPU. Il renderer GPU opzionale (`render-gpu`) non fa parte della build predefinita.

**Non hai Rust? Usa un archivio precompilato.** Quando il proprietario avrà pubblicato la pre-release in bozza `v0.1.0-alpha.3`, la [pagina Releases](https://github.com/chiuta/osatura/releases) offre `osatura-windows-x86_64.zip`, `osatura-macos-arm64.tar.gz`, `osatura-linux-x86_64.tar.gz` (glibc 2.35+; le release successive aggiungono l'archivio statico `osatura-linux-x86_64-musl.tar.gz` per le distribuzioni più vecchie) e `SHA256SUMS.txt`. Verifica l'archivio con le somme di controllo (`sha256sum -c SHA256SUMS.txt --ignore-missing`, oppure `Get-FileHash` su Windows), decomprimilo ed esegui `osatura` (`osatura.exe`) dalla cartella estratta; salta la compilazione della sezione 2 e leggi "Windows: doppio clic su `osatura.exe`" e "Lo studio come finestra" verso la fine di questa pagina. I programmi non sono firmati, quindi il sistema ti avviserà la prima volta. Tutto ciò che segue è stato eseguito con una build locale degli stessi sorgenti.

## 2. Compilarlo

Dalla radice del repository:

```
cargo build --release -p osatura-cli
```

Il binario è `target/release/osatura`. Mettilo nel tuo `PATH` oppure crea un alias:

```
export PATH="$PWD/target/release:$PATH"      # or: alias osatura=$PWD/target/release/osatura
osatura --version
```

`--version` stampa la versione, la licenza (AGPL-3.0-or-later per la CLI e lo studio, MPL-2.0 per le
librerie del motore) e l'offerta del sorgente. Eseguire `osatura` senza argomenti stampa l'uso completo;
`osatura <command> --help` stampa l'uso di un singolo comando.

Puoi anche eseguirlo senza installarlo: `cargo run --release -p osatura-cli -- demo demo.json`.

Compilare lo strumento da riga di comando con `--no-default-features` elimina il comando `script` (il
motore di scripting Rhai); questo tutorial usa `script`, quindi mantieni le funzionalità predefinite.

## 3. Primi passi con la demo integrata

`osatura demo` scrive un progetto di esempio completo: un braccio con skinning, una gamba con IK, un
livello switch per il lip-sync, uno sfondo con parallasse e un movimento di camera.

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
```

`check` compila il progetto esattamente come farebbe il renderer, quindi un progetto che supera `check`
può essere renderizzato. Ora disegna un fermo immagine, con le ossa sovrapposte, ed esporta un'animazione:

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

Che cosa hai appena fatto:

* `render <project> <frame> <out.png|out.svg>` disegna un singolo fotogramma. `--bones` sovrappone lo
  scheletro, `--scale N` renderizza a N volte la dimensione del canvas (0 < N <= 16).
* `export` sceglie il formato dall'estensione del file: `.gif`, `.html` (una pagina con player
  autonomo), `.json` (Lottie), `.lottie` (dotLottie), `.apng`, `.webm`, `.mp4`, `.glb`, `.png` (sprite sheet).
* Le righe `warning:` sono la parte onesta: Lottie non può rappresentare tutto ciò che sa fare OSATURA.
  Ogni limitazione viene stampata su stderr e `--report r.json` scrive lo stesso elenco come JSON.
* Apri `demo.html` in un browser per riprodurre l'animazione; non sono coinvolti né un server né la rete.

Per esplorare in modo interattivo:

```
osatura studio demo.json --port 8080
```

e apri <http://127.0.0.1:8080/>. Vedi [STUDIO.md](STUDIO.md).

## 4. Un tutorial di 15 minuti: un personaggio che saluta

Costruirai "Buddy" partendo da un progetto vuoto. Il file di progetto è JSON, ma non devi mai scrivere
più di un'intestazione: il comando `script` esegue un piccolo script [Rhai](https://rhai.rs) in sandbox
attraverso lo stesso editor validato che usa lo studio. Se una richiesta di uno script viene rifiutata,
*non viene applicato nulla* (tutto o niente), quindi uno script sbagliato non lascia mai un file
modificato a metà.

Lavora in una cartella vuota. Ogni passo scrive un nuovo file numerato, così puoi sempre tornare indietro.

### Passo 1: un progetto vuoto (1 minuto)

```
cat > buddy.json <<'EOF'
{"format": 1, "name": "Buddy", "width": 320, "height": 240, "fps": 24, "frames": 24}
EOF
osatura check buddy.json
```
```
ok: 'Buddy' 320×240 @ 24 fps, 24 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Queste sei chiavi sono il minimo. 24 fotogrammi a 24 fps fanno un secondo, che sarà il nostro loop.

### Passo 2: le forme (2 minuti)

I livelli vengono disegnati dal fondo al primo piano, nell'ordine in cui li aggiungi. Crea `shapes.rhai`:

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

Le coordinate sono unità del canvas, con l'origine in alto a sinistra e la y rivolta verso il basso. Il
braccio è disegnato in orizzontale, con la spalla in (160, 140), perché è la posa in cui ne costruiremo le ossa.

### Passo 3: ossa, parentela e collegamento della grafica (3 minuti)

Un osso ha un nome, un genitore (vuoto = radice), una lunghezza, una posizione della testa (`x`, `y`) nello
spazio del genitore (per una radice: nel canvas) e un angolo di riposo in gradi relativo al genitore.
L'angolo 0 punta lungo +x e gli angoli positivi ruotano in senso orario sullo schermo, quindi 90 punta
in basso e -90 punta in alto. Crea `bones.rhai`:

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

Leggi le ossa così: `body` è un osso radice in (160, 190) che punta verso l'alto (-90) ed è lungo 70.
`neck` è il suo figlio; la sua testa si trova a 70 unità lungo `body` (in cima). `arm_up` è un altro figlio
di `body`, a 50 unità di altezza lungo la colonna (la spalla), ruotato di +90 in modo che punti di lato.
`arm_low` è il figlio di `arm_up`: la parentela significa che ruotare la parte alta del braccio trascina con sé l'avambraccio.
`hand_ctrl` è un osso radice separato che useremo come bersaglio dell'IK; non ha grafica.

`set_layer(layer, "bone", bone)` fa sì che un livello viaggi su un osso in modo *rigido*: segue il movimento
dell'osso rispetto alla sua posa di riposo. È il modo più semplice per dotare di rig un cartone animato
fatto di pezzi separati. (Per le forme flessibili si applica lo skinning di ancore vettoriali a più ossa;
vedi [RIGGING_GUIDE.md](RIGGING_GUIDE.md).)

### Passo 4: cinematica inversa (1 minuto)

Invece di animare con chiavi gli angoli di spalla e gomito, lascia che sia il solver a raggiungere
`hand_ctrl`. Crea `ik.rhai`:

```
add_ik("arm_ik", ["arm_up", "arm_low"], "hand_ctrl", 1);
```
```
$ osatura script 02-bones.json 03-ik.json ik.rhai
1 request(s) applied; wrote 03-ik.json
```

L'ultimo argomento è la direzione di piegamento: +1 o -1 sceglie da che parte si piega il gomito. Una
catena di due ossa usa un solver analitico; le catene di tre o più ossa usano FABRIK. D'ora in poi,
muovere `hand_ctrl` muove l'intero braccio.

### Passo 5: chiavi ed easing (3 minuti)

`set_key(target, property, frame, value, ease)` crea le chiavi. Per un osso, `bone_angle`, `bone_scale`,
`bone_offset_x` e `bone_offset_y` sono *delta rispetto alla posa di riposo*, quindi un valore di chiave
pari a 0 significa "riposo". Il nome dell'easing si applica al segmento che *inizia* in quella chiave.
Crea `keys.rhai`:

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

I nomi di easing che non richiedono parametri sono `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`,
`smooth` e `bounce`. `bezier:x1,y1,x2,y2` è accettato da `keyedit`, mentre `spring` e `back` prendono
parametri (vedi [PROJECT_FORMAT.md](PROJECT_FORMAT.md)). La mano e la testa usano tempi diversi di proposito: l'azione
sovrapposta dà l'impressione di vita.

### Passo 6: farlo andare in loop (1 minuto)

Entrambi i canali iniziano e finiscono con lo stesso valore (il fotogramma 24 ripete il fotogramma 0), ed è
ciò che serve a un loop. `loop-solve` controlla la giunzione e imposta i canali in loop:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
$ osatura lint buddy-final.json
0 finding(s), 0 error(s)
```

"seam 0.000 -> 0.000" significa che l'ultima posa è già uguale alla prima, quindi non c'era nulla da
correggere. Un progetto di N fotogrammi riproduce ripetutamente i fotogrammi 0..N-1, quindi il fotogramma N
è la stessa posa del fotogramma 0 e non viene mai mostrato due volte. Se autori un loop in modo che l'*ultimo
fotogramma mostrato* sia uguale al primo, passa `--inclusive` (la convenzione precedente). Se la tua
animazione ha uno spostamento netto, `make-loop` lo rimuove. Vedi
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

`lint` esegue controlli statici su rig e animazione (bersagli IK irraggiungibili, canali inutilizzati, ossa mancanti...);
esce con codice 1 quando trova errori, quindi si può inserire senza problemi in uno script o in un job di CI.

### Passo 7: guardarlo e renderizzarlo (2 minuti)

```
$ osatura stats buddy-final.json
```
stampa un riepilogo JSON (qui: 5 ossa, 1 catena IK, 6 livelli, 2 canali, 6 chiavi). Renderizza dei fermo
immagine e un'intera sequenza di fotogrammi:

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

![Quattro fotogrammi (0, 6, 12, 18) dell'animazione di Buddy, una volta senza e una volta con le ossa sovrapposte. Il braccio destro si solleva in un saluto con il gomito in basso mentre la testa annuisce leggermente.](../../img/tutorial-buddy.png)

Senza `--png`, `frames` scrive file SVG. I render sono deterministici: lo stesso progetto e lo stesso
fotogramma producono gli stessi byte su qualsiasi macchina con la stessa build, ed è ciò che rende possibili
i controlli sui fotogrammi di riferimento descritti in [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Passo 8: esportare (2 minuti)

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

Dimensioni dei file di questo piccolo progetto: GIF 145 KB, JSON Lottie 21 KB, player HTML 50 KB, APNG 75 KB, MP4 6 KB.
`buddy.json` è Lottie; ha un elenco di avvisi vuoto perché Buddy usa solo funzionalità che Lottie sa
esprimere. Il file HTML è un player autonomo (aprilo in qualsiasi browser).

Alcune regole da ricordare:

* `export` e `render` **rifiutano di sovrascrivere il progetto di input** (`refusing to overwrite the input
  project buddy-final.json with output of the same name`) e rifiutano di scrivere attraverso un collegamento
  simbolico a meno che tu non passi `--follow-symlinks`. I comandi che trasformano un progetto in un altro
  (`script`, `loop-solve`, `walk`, `autorig`...) *non* hanno questo controllo: scrivono in modo atomico, quindi
  `osatura script p.json p.json x.rhai` modifica sul posto. Usa nomi distinti finché non ti fidi di uno script.
* La GIF ha 256 colori e trasparenza a 1 bit; per un alfa morbido usa `.apng` o `.webm`.
* Esegui `osatura export-fit` (vedi [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) quando devi rispettare un budget di byte.

### Passo 9 (opzionale): modificarlo nello studio

```
osatura studio buddy-final.json --port 8080
```

Trascina i livelli, seleziona una chiave nel dope sheet, premi `Space` per riprodurre. Nulla viene scritto
nel tuo file finché non premi **Save** (`Ctrl+S`).

## 5. Come si incastrano i pezzi

| Concetto | Nel file | Lo crei con |
|---|---|---|
| Livello (vettoriale, immagine, gruppo, switch...) | `layers` | `add_rect`, `add_circle`, `add_polygon`, `add_line`, `import-svg`, `autotrace`... |
| Osso | `bones` (i genitori prima dei figli) | `add_bone`, `set_parent`, `reparent` |
| Catena IK / vincolo / molla | `ik`, `constraints`, `springs` | `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`; le molle tramite JSON |
| Canale (una proprietà animata) | `channels` | `set_key`, `keys`, `keyedit`, espressioni, rumore, bake |
| Clip, take, macchina a stati, binding | `clips`, `machines`, `bindings` | `take-record`, `clip`, `machine`, `bind` |

Tutto è JSON semplice, ed è per questo che i diff (`osatura diff a.json b.json`), il controllo di versione
(`osatura vcs`) e gli script funzionano. I campi sconosciuti vengono segnalati come avvisi e ignorati:

```
$ osatura check unk.json
warning: unknown field bogus (ignored)
ok: 'X' 100×100 @ 24 fps, 10 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Un progetto può fare riferimento a file di immagini e suoni solo **all'interno della propria cartella**. I
percorsi assoluti e `..` vengono rifiutati a meno che tu non passi `--allow-external-assets` (vedi [FAQ.md](FAQ.md)).

## 6. Dove andare dopo

* Dotare di rig qualcosa di flessibile (skin, pesi, lattice, cage, stoffa, fisica): [RIGGING_GUIDE.md](RIGGING_GUIDE.md).
* Cicli di camminata, stack di clip, lip-sync, mocap, macchine a stati, QA e budget di dimensione:
  [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).
* L'editor nel browser, le sue scorciatoie e l'API HTTP: [STUDIO.md](STUDIO.md).
* Domande e messaggi di errore: [FAQ.md](FAQ.md). Limiti noti: [LIMITATIONS.md](LIMITATIONS.md).
* Ogni comando e opzione: [CLI_REFERENCE.md](CLI_REFERENCE.md); il formato del file: [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Windows: doppio clic su `osatura.exe`

`osatura.exe` è un programma da riga di comando. Dalla 0.1.0-alpha.2, aprirlo da Esplora file (doppio clic) crea `Documents\OSATURA\first-rig.json` dalla demo della persona che cammina (`osatura demo-person`; un file esistente non viene mai sovrascritto), avvia lo studio su `http://127.0.0.1:8080` e lo apre nel browser; la finestra della console resta aperta finché lo studio è in esecuzione, chiudila per fermarlo. Aperto da un terminale senza argomenti stampa l'uso come prima. Nella 0.1.0-alpha.1 un doppio clic stampava solo l'uso e la finestra si chiudeva subito.

Gli eseguibili **non sono firmati**: Windows SmartScreen mostra "Editore sconosciuto" (scegli *Ulteriori informazioni* e poi *Esegui comunque*) e Gatekeeper di macOS ti chiede di approvare il programma in *Impostazioni di Sistema > Privacy e sicurezza*. Prima verifica il download con `SHA256SUMS.txt`. Se la porta 8080 è occupata, esegui `osatura studio first-rig.json --port 8090` da un terminale.

## Lo studio come finestra: `osatura-studio` (Windows e macOS)

Gli archivi Windows e macOS contengono un secondo programma, `osatura-studio` (`osatura-studio.exe`), una piccola finestra nativa attorno allo studio nel browser: avvia il motore `osatura` della stessa cartella come `studio` su una porta locale libera, mostra la pagina in una finestra WebView2 (Windows 11 lo include; il Windows più vecchio richiede il runtime Evergreen) o WKWebView (macOS) e ferma il server alla chiusura della finestra. Non compare alcuna finestra di console. Tienilo nella stessa cartella di `osatura(.exe)`.

* **Esportazioni** (GIF, player HTML, Lottie) aprono una finestra nativa "Salva con nome" che parte dalla cartella del progetto, e un messaggio indica il file una volta salvato. (alpha.2 le salvava in silenzio nella cartella Download; alpha.3 chiede dove.)
* Fai doppio clic per lavorare su `Documents/OSATURA/first-rig.json` (creato dalla demo della persona che cammina, `osatura demo-person`, se manca), oppure avvialo come `osatura-studio my-project.json` (per esempio con "Apri con").
* `osatura-studio --selftest` avvia il server, scarica la pagina e lo ferma senza finestra; il codice di uscita è 0 se ha funzionato (la CI lo esegue su Windows e macOS).
* **Limiti.** La finestra è stata usata su un vero desktop Windows 11 (alpha.2: studio ed esportazioni funzionavano); la finestra "Salva con nome" di alpha.3 è compilata e auto-testata in CI ma nessuno l'ha ancora provata, e nessuno ha usato la finestra su macOS. Le altre funzioni da browser che richiedono il sistema (trascinamento da Esplora file o Finder, apertura di file dai selettori propri dello studio) sono quelle della web view di sistema e non sono testate. Non esiste una finestra per Linux (richiederebbe i pacchetti di sistema GTK / WebKitGTK): usa `osatura studio` e un browser. Il programma non è firmato (vedi sopra).
