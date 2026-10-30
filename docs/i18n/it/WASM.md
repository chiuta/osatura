# OSATURA per WebAssembly

*ABI versione 1 (`arm_abi_version()`). Crate `osatura-wasm`, wrapper JavaScript `osatura-wasm/js/osatura.js`, pagina demo `osatura-wasm/index.html`, script di build `osatura-wasm/build-wasm.sh`. Destinatari: sviluppatori web che vogliono riprodurre progetti OSATURA in un browser o in node, e la prima persona che esegue il modulo in un browser. Documentazione CC-BY-4.0; esempi di codice MPL-2.0.*

> ## STATO: COMPILATO E TESTATO IN CI, ESEGUITO SOLO IN CHROMIUM HEADLESS
>
> **Al 24 ott 2026 il modulo viene compilato su GitHub Actions** (`wasm32-unknown-unknown`, build di debug in `ci.yml`, build di release in `release.yml`) e **il test del wrapper passa sul modulo reale con Node 20** (`node osatura-wasm/js/test.mjs demo.json`, compresa la parte con il modulo reale). La build di release è allegata alla pre-release in bozza `v0.1.0-alpha.3` come `osatura-web.tar.gz` (circa 0,9 MB compressi, con `osatura.js` e la pagina demo). **È stato caricato una sola volta in Chromium 141 headless su Linux senza GPU** (risultati in fondo a questa pagina, prodotti da `osatura-wasm/js/browser-check.js`); Firefox, Safari, telefoni e GPU reali non sono stati provati, quindi le affermazioni su di essi in questa pagina non sono comportamento osservato, e lo stesso ambiente di sviluppo non può ancora installare il target (il download della sua libreria standard è bloccato lì).
>
> Che cosa è stato testato, e fin dove arriva:
>
> | Livello | Test | Dimostra | Non dimostra |
> |---|---|---|---|
> | Le funzioni ABI Rust (`arm_*`) | `cargo test -p osatura-wasm`: 8 test unitari sull'**host**, che chiamano direttamente le funzioni esportate | il controllo degli argomenti, i codici di errore, il protocollo del buffer di output, le macchine a stati, le chiamate di layout / scorrimento / livello di dettaglio, il rifiuto di output troppo grandi prima dell'allocazione | il comportamento del modulo compilato |
> | Il modulo compilato | CI: `cargo build --target wasm32-unknown-unknown`, poi `node osatura-wasm/js/test.mjs demo.json` con il vero `.wasm` | che le dipendenze compilino per wasm32, che il modulo esporti ciò che il wrapper si aspetta e che caricamento, rendering e chiamate delle macchine a stati funzionino nel motore WebAssembly di Node | i browser, velocità e memoria della release, i panic (un trap), la pagina demo |
> | Il wrapper JavaScript | lo stesso script contro un **mock JavaScript** delle esportazioni (tre sezioni di mock: render, macchina a stati, layout) | le sequenze di chiamate del wrapper, il passaggio delle stringhe, la traduzione degli errori, la copia dei risultati | - |
> | La pagina demo | caricata in Chromium headless **senza** il file `.wasm` | i percorsi di import del modulo si risolvono e la pagina segnala "wasm not found" invece di bloccarsi | qualsiasi rendering in un browser |
> | Dipendenze | `cargo tree -p osatura-wasm`: nessun `rhai` / `smartstring` nell'albero | il motore di scripting non fa parte del modulo | - |
>
> **Rischi residui:** (1) *Risolto in CI:* la compilazione di `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder`, `i_overlay` e delle loro dipendenze per `wasm32-unknown-unknown`. (2) `std::time::Instant` / `SystemTime`, `process::id` e `thread::spawn` vanno in **panic** su `wasm32-unknown-unknown`: dalla wave 7 il core non li chiama più direttamente. Ogni uso passa per `osatura_core::clock` (un tipo di timestamp, `unix_seconds`, `process_id`; il costruttore dei proxy LOD ripiega sul thread chiamante quando non è possibile avviare un thread), che su questo target legge un orologio fornito dall'host (l'esportazione `arm_set_clock(mono_ms, unix_ms)`, alimentata dal wrapper JavaScript con `performance.now()` e `Date.now()` prima di `load`, `svg` e `rgba`; un host che non la chiama mai ottiene durate pari a 0, quindi i budget di tempo non scadono mai) e non va mai in panic. Un test (`osatura-core/tests/wasm_safety.rs`) fallisce quando compare un nuovo uso diretto fuori da `clock.rs`; il test Node in CI esercita i percorsi che raggiunge, non tutti. Rhai (che richiederebbe anche la sua feature `no_time`) non fa parte della build. (3) I moduli legati al file system e ai processi compilano ma falliscono a runtime: `audio.rs` (apertura di file), `soundtrack.rs` (ffprobe), `snapshots.rs`, `open_formats.rs` (directory di esportazione), `agent.rs` (salvataggio), e il caricamento di immagini e normal map con percorso relativo in `engine.rs`, `meshtrace.rs` e `postfx.rs`; restituiscono errori invece di funzionare in un browser, e nessuna feature di cargo li disattiva. (4) Le funzioni in virgola mobile (`sin`, `cos`, `powf`, ...) provengono da una libreria matematica diversa su wasm32, quindi i pixel renderizzati possono differire leggermente da quelli nativi (il passo 6 della checklist lo misura). (5) Su `wasm32-unknown-unknown` il comportamento predefinito è `panic = "abort"`, `catch_unwind` non può intercettare nulla e un panic di Rust manda in trap l'intera istanza (`WebAssembly.RuntimeError: unreachable`). Vedi la [checklist di verifica](#checklist-di-verifica-per-la-prima-persona-in-grado-di-eseguirlo-in-un-browser) alla fine.

Indice: [Che cos'è](#che-cosè) - [Compilazione](#compilazione) - [Modello di memoria e ABI](#modello-di-memoria-e-abi) - [Riferimento delle esportazioni](#riferimento-delle-esportazioni) - [Il wrapper JavaScript](#il-wrapper-javascript) - [La pagina demo](#la-pagina-demo) - [Differenze rispetto all'ABI C](#differenze-rispetto-allabi-c) - [Punti noti ancora ruvidi](#punti-noti-ancora-ruvidi) - [Checklist di verifica](#checklist-di-verifica-per-la-prima-persona-in-grado-di-eseguirlo-in-un-browser)

## Che cos'è

`osatura-wasm` è una `cdylib` di **semplici funzioni `extern "C"`** (senza `wasm-bindgen`, senza codice di collegamento generato) sopra la facciata Rust stabile ([API.md](API.md)). Compilata per `wasm32-unknown-unknown` fornisce un modulo che può aprire un progetto, renderizzare fotogrammi come testo SVG o pixel RGBA ed eseguire macchine a stati con layout, scorrimento e livello di dettaglio. Non può modificare, esportare, gestire il controllo di versione né eseguire strumenti; non ha file system, rete, audio né thread. `osatura.js` è un modulo ES senza dipendenze (browser e node >= 18) che nasconde la gestione dei puntatori.

Licenza: MPL-2.0 (crate e wrapper). Il modulo dipende da `osatura-core` con `default-features = false`: nessun motore di scripting e nessun crate di terze parti con licenza MPL.

## Compilazione

Non devi compilarlo: la release in bozza contiene `osatura-web.tar.gz` (modulo, wrapper, pagina demo). Per compilarlo tu stesso serve una macchina su cui il target possa essere installato (e, facoltativamente, `wasm-opt` di Binaryen per ridurre le dimensioni). La CI lo compila con `cargo build --release --target wasm32-unknown-unknown -p osatura-wasm --locked`.

```sh
rustup target add wasm32-unknown-unknown
./osatura-wasm/build-wasm.sh
```

Lo script (`osatura-wasm/build-wasm.sh`, uno script shell POSIX) fa esattamente questo, dalla radice del repository:

1. `cargo build --target wasm32-unknown-unknown --release -p osatura-wasm`
2. copia `target/wasm32-unknown-unknown/release/osatura_wasm.wasm` in `osatura-wasm/osatura_wasm.wasm`;
3. se `wasm-opt` è nel `PATH`, riscrive quel file con `wasm-opt -Oz`;
4. stampa la dimensione del risultato e un suggerimento per eseguire il test node o servire la demo.

**Lo script in sé non è ciò che esegue la CI** (la CI chiama direttamente `cargo build` e copia il modulo). Il suo suggerimento finale dice `node osatura-wasm/js/test.mjs`; con un vero `.wasm` presente il test richiede in più il percorso di un progetto come primo argomento (`node osatura-wasm/js/test.mjs project.json`), altrimenti si ferma con "pass a project JSON path as argv[2]". Il modulo di release nella bozza pesa circa 0,9 MB come archivio compresso; le dimensioni grezze e con `wasm-opt -Oz` non sono registrate qui.

Poi servi la cartella via HTTP (i browser non caricano moduli o `.wasm` da `file://`) e apri la pagina demo:

```sh
cd osatura-wasm && python3 -m http.server 8000      # then open http://localhost:8000/index.html
```

Se usi `WebAssembly.instantiateStreaming` (il wrapper lo fa quando gli passi una `Response`), il server deve inviare `Content-Type: application/wasm`; il percorso URL del wrapper usa `fetch(...).arrayBuffer()` e non ha questo requisito. Se la tua pagina imposta una Content-Security-Policy, questa deve consentire la compilazione di WebAssembly (`'wasm-unsafe-eval'` in `script-src`).

Un target alternativo, `wasm32-wasip1`, compila con `-C panic=unwind` in modo che `catch_unwind` funzioni (il sorgente lo menziona); non è stato provato e richiede un runtime WASI o uno shim.

## Modello di memoria e ABI

Il modulo possiede un'unica **memoria lineare** (esportata come `memory`). L'host non condivide mai puntatori con il motore se non secondo queste regole:

* **Input.** Per passare byte (il JSON del progetto, o un nome), l'host chiama `arm_alloc(len)`, che restituisce un puntatore a `len` byte **azzerati** (null quando `len` è 0 o supera 256 MiB). L'host scrive in `new Uint8Array(memory.buffer, ptr, len)`, chiama la funzione con `(ptr, len)`, poi rilascia il blocco con `arm_free(ptr, len)` usando **lo stesso** puntatore e la stessa lunghezza. Il motore copia ciò che gli serve; non conserva alcun puntatore al tuo blocco.
* **Output.** I risultati vengono prodotti in un buffer **all'interno dell'handle**: dopo una chiamata che restituisce una lunghezza `n >= 0`, `arm_out_ptr(h)` / `arm_out_len(h)` danno i byte (`n` è uguale alla lunghezza) e, per le immagini, `arm_out_width(h)` / `arm_out_height(h)` danno la dimensione (0 dopo risultati SVG o testuali). Il puntatore è valido **fino alla successiva chiamata che produce output, o fino a `arm_close`**. L'host deve copiare subito i byte (il wrapper lo fa: `.slice()`). Non conservare mai una vista `Uint8Array` tra una chiamata e l'altra: oltre a essere sovrascritta, qualsiasi chiamata può far crescere la memoria del modulo, il che stacca `memory.buffer` e invalida ogni sua vista. Rileggi `memory.buffer` dopo ogni chiamata.
* **Errori.** Una funzione che fallisce restituisce `-1` (o un handle null) e memorizza il messaggio; `arm_error_ptr()` / `arm_error_len()` ne danno i byte UTF-8 (non terminati da NUL). Lo slot è globale per il modulo (che è single-thread), viene sovrascritto dal fallimento successivo e **non viene cancellato dai successi**, quindi leggilo solo subito dopo un fallimento.
* **Stringhe in ingresso.** `(ptr, len)` UTF-8, `len` tra 1 e 4096 (una stringa vuota o più lunga è un errore).
* **Handle.** `arm_open` restituisce un puntatore opaco a un handle (null in caso di errore). Un handle contiene l'animazione, il player facoltativo e il buffer di output. Chiudilo con `arm_close` esattamente una volta; `null` è ammesso.
* **Limiti.** JSON del progetto da 1 byte a 256 MiB; `arm_render_rgba` rifiuta prima di allocare se l'immagine supererebbe 256 MiB (`output would be WxH pixels (N bytes), more than the limit of 268435456 bytes: lower the scale`); scala tra 0.05 e 4; i fotogrammi fuori dalla timeline vengono limitati all'intervallo; lunghezze restituite come `i32`. Il rendering richiede memoria per l'immagine più i buffer di lavoro; lo spazio di indirizzamento a 32 bit (4 GiB, meno in alcuni browser) è il vero tetto.
* **Nessun thread, nessuna promessa sui panic.** Il modulo è single-thread. I panic vengono intercettati solo sui target che eseguono l'unwind; su `wasm32-unknown-unknown` un panic manda in trap l'istanza e l'host deve scartarla e crearne una nuova. Il motore è scritto per non andare in panic sull'input dell'utente e controlla le dimensioni prima di allocare, ma questo livello non può promettere di sopravvivere a un bug.

## Riferimento delle esportazioni

37 esportazioni più `memory`. `ptr` e `len` sono `i32` nella firma wasm (`usize` in Rust). `h` è un handle. Il valore di ritorno `-1` = errore, salvo diversa indicazione.

| Esportazione | Firma (vista Rust) | Risultato |
|---|---|---|
| `arm_abi_version` | `() -> u32` | 1 |
| `arm_set_clock` | `(mono_ms: f64, unix_ms: f64)` | facoltativa: fornisce l'ora al modulo (`performance.now()`, `Date.now()`); i valori non validi vengono ignorati. Aggiunta dopo che la versione ABI 1 era stata fissata; il wrapper la chiama quando è presente |
| `arm_alloc` | `(len) -> *mut u8` | blocco azzerato, null se `len` è 0 o > 256 MiB |
| `arm_free` | `(ptr, len)` | libera un blocco di `arm_alloc` (null / lunghezza zero ignorati) |
| `arm_open` | `(json_ptr, json_len) -> *mut Handle` | handle o null; valida il progetto |
| `arm_close` | `(h)` | libera l'handle |
| `arm_frame_count`, `arm_fps`, `arm_width`, `arm_height` | `(h) -> u32 / f32 / f32 / f32` | `width` / `height` sono la dimensione del palco in unità del palco (non arrotondata); 0 e un errore per un handle null |
| `arm_render_svg` | `(h, frame: f32, show_bones: i32) -> i32` | testo SVG nel buffer di output; la sua lunghezza in byte |
| `arm_render_rgba` | `(h, frame: f32, scale: f32) -> i32` | RGBA8 con alfa non premoltiplicato nel buffer di output; lunghezza in byte (`w*h*4`); dimensione tramite `arm_out_width/height` |
| `arm_out_ptr`, `arm_out_len` | `(h) -> *const u8 / usize` | il buffer di output (null / 0 per un handle null) |
| `arm_out_width`, `arm_out_height` | `(h) -> u32` | dimensione dell'immagine dell'ultimo rendering RGBA (0 dopo un testo) |
| `arm_machine_count` | `(h) -> u32` | macchine nel progetto |
| `arm_machine_name` | `(h, index) -> i32` | nome nel buffer di output; la sua lunghezza; -1 per un indice errato |
| `arm_player_start` | `(h, name_ptr, name_len) -> i32` | avvia (o riavvia) la macchina; 0 |
| `arm_player_stop` | `(h)` | scarta il player |
| `arm_player_set_bool` | `(h, ptr, len, value: i32) -> i32` | 0 |
| `arm_player_set_number` | `(h, ptr, len, value: f64) -> i32` | 0 |
| `arm_player_fire` | `(h, ptr, len) -> i32` | 0 |
| `arm_player_set_value` | `(h, ptr, len, value: f64) -> i32` | 0; valore esterno per binding / input di scorrimento |
| `arm_player_advance` | `(h, dt: f64) -> i32` | 0; `dt` in `0..=10` secondi |
| `arm_player_state` | `(h, layer_ptr, layer_len) -> i32` | nome dello stato nel buffer di output; lunghezza; -1 livello sconosciuto |
| `arm_player_events` | `(h) -> i32` | eventi in sospeso come JSON nel buffer di output, **consumati**; lunghezza |
| `arm_player_render_svg` | `(h, show_bones: i32) -> i32` | SVG della posa corrente nel buffer di output |
| `arm_player_render_rgba` | `(h, scale: f32) -> i32` | RGBA della posa corrente; dimensione tramite `arm_out_width/height` |
| `arm_player_set_canvas` | `(h, w: f32, h: f32) -> i32` | 0 |
| `arm_player_layout` | `(h) -> i32` | JSON `{"canvas":[w,h],"scrolls":[ids],"lod":level}` nel buffer di output |
| `arm_player_scroll_drag` | `(h, id_ptr, id_len, x: f32, y: f32) -> i32` | 0 |
| `arm_player_scroll_release` | `(h, id_ptr, id_len, vx: f32, vy: f32) -> i32` | 0 |
| `arm_player_scroll_info` | `(h, id_ptr, id_len) -> i32` | JSON `{"offset":[x,y],"max":[x,y],"settled":bool}` nel buffer di output |
| `arm_player_set_screen` | `(h, w_px: f32, h_px: f32, max_error_px: f32) -> i32` | 0; sceglie il livello di dettaglio |
| `arm_error_ptr`, `arm_error_len` | `() -> *const u8 / usize` | testo dell'ultimo errore |

Tutte le chiamate `arm_player_*` tranne `arm_player_start` richiedono un player avviato (altrimenti "no player started (call arm_player_start)"). Le chiamate del player sono i metodi `Player` della facciata ([API.md](API.md)).

## Il wrapper JavaScript

`osatura-wasm/js/osatura.js` esporta `class Osatura` (anche come export predefinito). Ogni metodo che modifica qualcosa restituisce `this`, così le chiamate si concatenano. Gli errori vengono lanciati come `Error` con il messaggio del motore.

| Membro | Descrizione |
|---|---|
| `static async instantiate(source, imports = {})` | restituisce l'oggetto grezzo delle esportazioni. `source`: un `ArrayBuffer` o un array tipizzato (byte), una `Response` (usa `instantiateStreaming`; richiede il MIME `application/wasm`), oppure una stringa URL / `URL` (usa `fetch` + `arrayBuffer`). Il modulo non ha import, quindi `imports` resta `{}`. |
| `static async create(source, project)` | `source` è o un insieme di esportazioni già istanziato (un oggetto con `arm_open`) o qualsiasi cosa accetti `instantiate`; `project` è testo JSON o un oggetto semplice (viene applicato `JSON.stringify`). Restituisce un `Osatura`. |
| `load(project)` | carica o sostituisce il progetto; lancia un errore in caso di fallimento e poi **mantiene utilizzabile il progetto caricato in precedenza**. Riempie `frames`, `fps`, `width`, `height`. |
| `frames`, `fps`, `width`, `height` | dati dell'ultimo `load` (`width` / `height` in unità del palco, float) |
| `svg(n, showBones = false)` | `string`: documento SVG per il fotogramma `n` (fotogrammi frazionari ammessi, limitati all'intervallo) |
| `rgba(n, scale = 1)` | `{ width, height, data }`: `data` è un `Uint8ClampedArray` (RGBA con alfa non premoltiplicato, una copia di tua proprietà) |
| `frame(n, { kind = 'image', scale = 1, bones = false } = {})` | `kind: 'svg'` dà `{ svg }`; altrimenti `{ image: ImageData }` in un browser oppure `{ width, height, data }` dove `ImageData` non esiste (node) |
| `machines()` | `string[]` nomi delle macchine |
| `startPlayer(name)`, `stopPlayer()` | avvia (riavvia) / scarta il player; gli input tornano ai valori predefiniti |
| `setBool(name, v)`, `setNumber(name, v)`, `fire(name)`, `setValue(path, v)` | input e valori esterni; un nome vuoto lancia `empty string`, un nome di oltre 4096 byte lancia un errore |
| `advance(dt)` | fa avanzare la macchina e le regioni di scorrimento di `dt` secondi (`0..=10`) |
| `state(layer)` | `string` stato corrente di un livello macchina |
| `events()` | `{ time, layer, kind: 'entered' \| 'exited', state }[]`, **consumati** dalla chiamata |
| `setCanvas(w, h)` | layout responsivo per una dimensione di canvas in unità del palco |
| `layout()` | `{ canvas: [w, h], scrolls: string[], lod: number }` |
| `scrollDrag(id, x, y)`, `scrollRelease(id, vx = 0, vy = 0)` | trascinamento del puntatore / velocità di rilascio in px e px/s |
| `scrollInfo(id)` | `{ offset: [x, y], max: [x, y], settled: boolean }` |
| `setScreen(widthPx, heightPx, maxErrorPx = 0.5)` | sceglie il livello di dettaglio per una dimensione dello schermo |
| `playerSvg(showBones = false)`, `playerRgba(scale = 1)` | la posa corrente del player |
| `play(canvas, { scale = 1, loop = true, onFrame } = {})` | disegna la timeline su un canvas 2D con `requestAnimationFrame` (ripiega su `setTimeout` fuori da un browser); restituisce una funzione di stop |
| `playMachine(canvas, { scale = 1, onTick } = {})` | fa avanzare la macchina avviata del tempo reale trascorso (al massimo 0,25 s per tick) e la disegna; `onTick(dt)` viene eseguito prima di ogni avanzamento, quindi imposta lì gli input; restituisce una funzione di stop |
| `stop()` | ferma `play` / `playMachine` |
| `close()` | ferma e libera l'handle; chiamalo quando hai finito |

Il wrapper mantiene le ricerche di `x.memory.buffer` all'interno di ogni chiamata e copia i risultati con `.slice()`, quindi la crescita della memoria non può lasciarlo con una vista obsoleta. Ogni oggetto `Osatura` possiede un handle; crea più oggetti (possono condividere un unico `exports`) per tenere più progetti. Non condividere un oggetto tra attività asincrone concorrenti che alternano chiamate e letture dei risultati: il risultato di ogni chiamata resta nel buffer di output dell'handle fino alla chiamata successiva.

### Esempio (sequenza di chiamate)

Questa è l'esatta sequenza di chiamate che il test node esercita. **È stata eseguita contro il mock JavaScript** (il sostituto in `test.mjs` con risposte predefinite: macchina `hero`, livello `body`, input bool `go`, regione di scorrimento `list`), non contro un modulo reale.

```js
import { Osatura } from './js/osatura.js';

// 1. Instantiate the module once (a URL in the browser; in node pass the bytes: readFileSync('osatura_wasm.wasm')).
const exports = await Osatura.instantiate('./osatura_wasm.wasm');

// 2. Load a project (JSON text or a plain object). Throws Error(<engine message>) when it is invalid.
const anim = await Osatura.create(exports, projectJsonText);
console.log(`${anim.width}x${anim.height} stage units, ${anim.frames} frames @ ${anim.fps} fps`);

// 3. Render. Frames may be fractional and are clamped to the timeline.
const svg = anim.svg(10, /* showBones */ false);        // string
const { width, height, data } = anim.rgba(10, 0.5);     // data: Uint8ClampedArray, straight-alpha RGBA, a copy you own

// 4. Interactive playback with a state machine.
console.log(anim.machines());                           // ['hero']
anim.startPlayer('hero').setBool('go', true).advance(0.1);
console.log(anim.state('body'), anim.events());         // 'b', [{time, layer, kind, state}, ...] (consumed)
const poseSvg = anim.playerSvg();
const pose = anim.playerRgba(0.5);

// 5. Layout and scrolling.
anim.setCanvas(100, 120);
anim.scrollDrag('list', 0, 50).scrollRelease('list', 0, 100);
console.log(anim.scrollInfo('list'));                   // { offset: [0, 50], max: [0, 80], settled: false }
anim.setScreen(100, 120, 4);                            // pick a level of detail within 4 px
console.log(anim.layout());                             // { canvas: [100, 120], scrolls: ['list'], lod: 2 }

// 6. Always release the handle (it frees the project inside the module's memory).
anim.close();
```

In una pagina del browser:

```html
<canvas id="c"></canvas>
<script type="module">
  import { Osatura } from './js/osatura.js';
  const x = await Osatura.instantiate('./osatura_wasm.wasm');
  const text = await (await fetch('./project.json')).text();
  const anim = await Osatura.create(x, text);

  // Timeline playback: requestAnimationFrame loop, loops at the end, returns a stop function.
  const stop = anim.play(document.getElementById('c'), { scale: 1, loop: true });

  // Or run a state machine in real time; onTick(dt) runs before each advance, so set inputs there.
  // anim.startPlayer('hero');
  // const stopMachine = anim.playMachine(canvas, { onTick: () => anim.setBool('go', keyIsDown) });
</script>
```

## La pagina demo

`osatura-wasm/index.html` è una pagina minimale: importa il wrapper, istanzia `./osatura_wasm.wasm` e offre un campo file (progetto `.json`) e un pulsante Stop. Scegliendo un file crea un `Osatura`, mostra `WIDTHxHEIGHT, N frames @ FPS fps` e avvia `play(canvas)`; gli errori del motore vengono mostrati nel testo accanto ai pulsanti. Se il modulo non può essere recuperato o compilato mostra `wasm not found (not built yet): <reason>`. Testata solo in quello stato "non compilato", in Chromium headless: la pagina si carica, l'import del modulo viene risolto, il messaggio compare (con la pagina 404 del server come "modulo" il motivo riporta `expected magic word 00 61 73 6d, found 3c 21 44 4f`, vedi sotto). Tutto ciò che va oltre non è verificato.

## Differenze rispetto all'ABI C

| | ABI C ([C_ABI.md](C_ABI.md)) | WebAssembly |
|---|---|---|
| Stringhe in ingresso | terminate da NUL | `(ptr, len)` copiate con `arm_alloc` |
| Risultati in uscita | buffer forniti dal chiamante | il buffer di output dell'handle (`arm_out_ptr/len`), valido fino alla successiva chiamata con output |
| Richiesta di dimensione | un buffer `NULL` restituisce la dimensione | non necessaria: la chiamata restituisce la lunghezza e i dati sono già nel buffer di output |
| Testo dell'errore | stringa NUL `osatura_last_error()`, per thread | byte `arm_error_ptr/len`, globale |
| Matrici degli ossi | `osatura_bone_world` | non esportata |
| SVG | non esportato | `arm_render_svg`, `arm_player_render_svg` |
| Eventi | modello della richiesta di dimensione, conservati se il buffer è troppo piccolo | sempre consumati |
| Dimensione del palco | interi arrotondati | float |

## Punti noti ancora ruvidi

Trovati leggendo il codice; di nessuno è stato osservato il fallimento:

* `Osatura.instantiate(url)` controlla `response.ok` e gli errori di fetch e segnala l'URL e lo stato HTTP (test con fetch simulato in `osatura-wasm/js/test.mjs`; eseguiti sotto node, e in CI contro il modulo reale).
* `frame()` con `kind: 'svg'` restituisce `{ svg }`, non una stringa come `svg()`.
* Il suggerimento stampato da `build-wasm.sh` ora include l'argomento `<project.json>` di cui `test.mjs` ha bisogno.
* `play()` disegna convertendo ogni fotogramma in RGBA sulla CPU e chiamando `putImageData`; non c'è audio, né pausa, né ricerca.
* Il wrapper non esporta dichiarazioni TypeScript.

## Checklist di verifica per la prima persona in grado di eseguirlo in un browser

I passi 1 e 5 sono stati eseguiti dalla CI il 24 ott 2026. I passi 3, 4, 6, 8, 9, 11 e 13 sono stati eseguiti lo stesso giorno in **Chromium 141 headless (Linux, senza GPU)** con `osatura-wasm/js/browser-check.js`; i numeri sono nella tabella qui sotto. **Ancora aperti:** Firefox e Safari (lì i passi 11 e 12), il passo 12 su CSP e content-type, la profondità dello stack del passo 10 e i passi 7 e 14 del player. Esegui questi passi in ordine; fermati al primo fallimento e annota il messaggio esatto. Le voci contrassegnate (rischio) corrispondono ai rischi descritti sopra. Quando hai i risultati, aggiorna [LIMITATIONS.md](LIMITATIONS.md) (sezione 1), la riga R-090 della sezione 9 di ROADMAP.md e il riquadro di stato di questa pagina.

1. **Installa il target e compila.** `rustup target add wasm32-unknown-unknown && ./osatura-wasm/build-wasm.sh`. (rischio) Verifica se `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder` e `i_overlay` compilano. Annota il compilatore e `rustc --version`.
2. **Nessun motore di scripting.** `cargo tree --target wasm32-unknown-unknown -p osatura-wasm | grep -ci 'rhai\|smartstring'` deve stampare 0.
3. **Guarda le esportazioni.** In node: `WebAssembly.Module.exports(new WebAssembly.Module(fs.readFileSync('osatura-wasm/osatura_wasm.wasm')))`. Ti aspetti `memory` e le 37 funzioni della [tabella delle esportazioni](#riferimento-delle-esportazioni), e **nessun import** (`WebAssembly.Module.imports(...)` vuoto). Se `memory` manca o compaiono import (per esempio `env.*` da una feature di std), il wrapper va adattato.
4. **Dimensione.** Annota la dimensione prima e dopo `wasm-opt -Oz`, e le dimensioni gzip / brotli. Misurate una volta nell'esecuzione in Chromium: vedi la tabella dei risultati in fondo a questa pagina.
5. **Esegui il test node con un progetto reale.** `osatura demo demo.json && node osatura-wasm/js/test.mjs demo.json` deve stampare `real wasm tests: ok` dopo le tre sezioni di mock.
6. **Confronta con il renderer nativo.** Per i fotogrammi 0, 12 e 47 della demo alle scale 0.5 e 1, confronta `Osatura.rgba` con `osatura render demo.json N out.png --scale S` decodificato in RGBA. (rischio) Ti aspetti piccole differenze sui bordi anti-aliasati (libreria matematica diversa); riporta la differenza assoluta massima e media e la frazione di pixel sopra 24/255. Una differenza grande indica un bug reale.
7. **Metti alla prova il player.** Costruisci il progetto dell'esempio dell'ABI C (demo + `take-record` + `clip add` + `machine add`, vedi [C_ABI.md](C_ABI.md)), pilota `startPlayer`, `setBool`, `advance`, `state`, `events`, `playerRgba`; e il progetto con un contenitore e una regione di scorrimento per `setCanvas`, `scrollDrag`, `scrollRelease`, `scrollInfo`, `setScreen` (l'ultima costruisce i livelli di dettaglio: cronometrala, è lenta senza ottimizzazioni e non è stata cronometrata su wasm).
8. **Percorsi a prova di errore.** Fornisci un progetto non valido, una stringa vuota, un file casuale da 5 MB, un progetto con un percorso immagine `source.file` e un progetto con una traccia sonora. Ogni caso deve produrre un `Error` lanciato con un messaggio, non un trap. (rischio) Un trap (`RuntimeError: unreachable`) in uno qualsiasi di essi è un bug da segnalare; i candidati più probabili sono `Instant` / `SystemTime` o chiamate al file system raggiunte per un percorso inatteso.
9. **Rendering grande.** `rgba(0, 4)` sulla demo (2560 x 1920) e un palco di 12000 x 12000 a scala 1 (deve essere rifiutato con il messaggio "more than the limit", non esaurire la memoria). Osserva la crescita della memoria e conferma che il wrapper funzioni ancora dopo che la memoria è cresciuta (le viste vengono rilette a ogni chiamata).
10. **Profondità dello stack.** `wasm32-unknown-unknown` dà al thread principale uno stack predefinito piccolo (1 MiB nelle toolchain attuali). Apri un progetto con gruppi molto annidati e una lunga catena di espressioni (vedi `osatura-cli/tests/robust.rs` per gli input ostili) e conferma che non ci sia alcun trap per overflow dello stack.
11. **Esecuzione nel browser.** Servi la cartella, apri `index.html` in Chromium e Firefox, scegli un progetto, conferma la riproduzione, ridimensiona, chiudi e riapri più progetti (controllo delle perdite: la memoria non deve crescere senza limite; `close()` rilascia l'handle).
12. **`instantiateStreaming` e CSP.** Servi con e senza `Content-Type: application/wasm`, e con una CSP che omette `'wasm-unsafe-eval'`; documenta che cosa succede.
13. **Prestazioni.** Cronometra `rgba(n, 1)` per la demo 640 x 480 (nativo release: circa 14 ms per fotogramma) e per un progetto pesante; annota i fotogrammi al secondo di `play()` nel browser.
14. **Facoltativo: panic.** Decidi se il comportamento di trap in caso di panic è accettabile per il tuo uso. In caso contrario, prova il percorso `wasm32-wasip1` + `-C panic=unwind` menzionato nel sorgente.

### Risultati dell'esecuzione in Chromium (24 ott 2026)

| Passo | Risultato |
|---|---|
| 4. Dimensione | modulo release 2.802.534 byte; 853.804 gzip; 615.267 brotli (non passato da `wasm-opt`) |
| 6. Confronto con il nativo | fotogramma 12 della demo, 640x480: **0 byte diversi** su 1.228.800 (differenza assoluta media e massima 0) |
| 8. Sicurezza in caso di errore | JSON vuoto, spazzatura, di forma errata, 5 MB casuali e "nessun fotogramma" lanciano ciascuno un `Error` con un messaggio; nessun trap |
| 9. Render grande | la scala 4 (2560x1920) funziona; uno stage 12000x12000 viene rifiutato con il messaggio sul limite e il modulo continua a funzionare |
| 11. Perdite | 60 cicli di apertura e chiusura: memoria lineare 54.591.488 byte prima e dopo |
| 13. Velocità | `rgba` della demo 640x480: mediana 72,5 ms, p95 96,5 ms, massimo 125,4 ms (circa 14 fps; nativo release circa 14 ms) |

Rieseguilo con `node osatura-wasm/js/browser-check.js <cartella osatura-web decompressa> <project.json>` (richiede Playwright). La pagina non ha generato errori durante l'esecuzione.
