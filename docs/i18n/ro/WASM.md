# OSATURA pentru WebAssembly

*Versiunea ABI 1 (`arm_abi_version()`). Crate `osatura-wasm`, wrapper JavaScript `osatura-wasm/js/osatura.js`, pagină demo `osatura-wasm/index.html`, script de build `osatura-wasm/build-wasm.sh`. Public țintă: dezvoltatorii web care vor să redea proiecte OSATURA într-un browser sau în node și prima persoană care rulează modulul într-un browser. Documentație CC-BY-4.0; exemple de cod MPL-2.0.*

> ## STARE: CONSTRUIT ȘI TESTAT ÎN CI, RULAT DOAR ÎN CHROMIUM FĂRĂ INTERFAȚĂ
>
> **La 24 oct. 2026 modulul este construit pe GitHub Actions** (`wasm32-unknown-unknown`, build debug în `ci.yml`, build release în `release.yml`) și **testul wrapper-ului trece față de modulul real sub Node 20** (`node osatura-wasm/js/test.mjs demo.json`, inclusiv partea sa cu modulul real). Build-ul release este atașat la pre-lansarea ciornă `v0.1.0-alpha.3` ca `osatura-web.tar.gz` (aproximativ 0,9 MB comprimat, cu `osatura.js` și pagina demo). **A fost încărcat o singură dată în Chromium 141 fără interfață, pe Linux, fără GPU** (rezultatele sunt la sfârșitul acestei pagini, produse de `osatura-wasm/js/browser-check.js`); Firefox, Safari, telefoanele și GPU-urile reale nu au fost încercate, deci afirmațiile despre ele din această pagină nu sunt comportament observat, iar mediul de dezvoltare în sine tot nu poate instala ținta (descărcarea bibliotecii sale standard este blocată acolo).
>
> Ce a fost testat și până unde ajunge acest lucru:
>
> | Strat | Test | Dovedește | Nu dovedește |
> |---|---|---|---|
> | Funcțiile ABI Rust (`arm_*`) | `cargo test -p osatura-wasm`: 8 teste unitare pe **gazdă**, apelând direct funcțiile exportate | verificarea argumentelor, codurile de eroare, protocolul bufferului de ieșire, mașinile de stări, apelurile de layout / derulare / nivel de detaliu, refuzul ieșirilor supradimensionate înainte de alocare | comportamentul modulului compilat |
> | Modulul compilat | CI: `cargo build --target wasm32-unknown-unknown`, apoi `node osatura-wasm/js/test.mjs demo.json` cu `.wasm` real | că dependențele compilează pentru wasm32, că modulul exportă ce așteaptă wrapperul și că încărcarea, randarea și apelurile mașinii de stări funcționează sub motorul WebAssembly din Node | browserele, viteza și memoria în release, panic-urile (un trap), pagina demo |
> | Wrapperul JavaScript | același script față de un **mock JavaScript** al exporturilor (trei secțiuni de mock: randare, mașină de stări, layout) | secvențele de apeluri ale wrapperului, transmiterea șirurilor, traducerea erorilor, copierea rezultatelor | - |
> | Pagina demo | încărcată în Chromium headless **fără** fișierul `.wasm` | căile de import ale modulului se rezolvă, iar pagina raportează „wasm not found” în loc să se blocheze | orice randare într-un browser |
> | Dependențe | `cargo tree -p osatura-wasm`: fără `rhai` / `smartstring` în arbore | motorul de scripting nu face parte din modul | - |
>
> **Riscuri rămase:** (1) *Rezolvat în CI:* compilarea `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder`, `i_overlay` și a dependențelor lor pentru `wasm32-unknown-unknown`. (2) `std::time::Instant` / `SystemTime`, `process::id` și `thread::spawn` dau **panic** pe `wasm32-unknown-unknown`: de la valul 7 core nu le mai apelează direct. Fiecare utilizare trece prin `osatura_core::clock` (un tip de marcaj temporal, `unix_seconds`, `process_id`; constructorul de proxy-uri LOD revine la thread-ul apelant când nu se poate crea un thread), care pe această țintă citește un ceas furnizat de gazdă (exportul `arm_set_clock(mono_ms, unix_ms)`, alimentat de wrapperul JavaScript cu `performance.now()` și `Date.now()` înainte de `load`, `svg` și `rgba`; o gazdă care nu îl apelează niciodată obține durate de 0, deci bugetele de timp nu expiră niciodată) și nu dă niciodată panic. Un test (`osatura-core/tests/wasm_safety.rs`) eșuează când apare o utilizare directă nouă în afara `clock.rs`; testul Node din CI exersează căile pe care le atinge, nu pe toate. Rhai (care ar avea nevoie și de feature-ul său `no_time`) nu face parte din build. (3) Modulele legate de sistemul de fișiere și de procese compilează, dar eșuează la rulare: `audio.rs` (deschidere de fișier), `soundtrack.rs` (ffprobe), `snapshots.rs`, `open_formats.rs` (directoare de export), `agent.rs` (salvare) și încărcarea imaginilor și a hărților normale cu cale relativă în `engine.rs`, `meshtrace.rs` și `postfx.rs`; ele returnează erori în loc să funcționeze într-un browser și niciun feature cargo nu le blochează. (4) Funcțiile în virgulă mobilă (`sin`, `cos`, `powf`, ...) provin dintr-o bibliotecă matematică diferită pe wasm32, deci pixelii randați pot diferi ușor de cei nativi (pasul 6 al listei de verificare măsoară diferența). (5) Pe `wasm32-unknown-unknown` implicit este `panic = "abort"`, `catch_unwind` nu poate intercepta nimic, iar un panic Rust blochează (trap) întreaga instanță (`WebAssembly.RuntimeError: unreachable`). Vezi [lista de verificare](#lista-de-verificare-pentru-prima-persoană-care-îl-poate-rula-într-un-browser) de la sfârșit.

Cuprins: [Ce este](#ce-este) - [Construirea](#construirea) - [Modelul de memorie și ABI-ul](#modelul-de-memorie-și-abi-ul) - [Referința exporturilor](#referința-exporturilor) - [Wrapperul JavaScript](#wrapperul-javascript) - [Pagina demo](#pagina-demo) - [Diferențe față de ABI-ul C](#diferențe-față-de-abi-ul-c) - [Aspecte încă neșlefuite](#aspecte-încă-neșlefuite) - [Lista de verificare](#lista-de-verificare-pentru-prima-persoană-care-îl-poate-rula-într-un-browser)

## Ce este

`osatura-wasm` este un `cdylib` de **funcții `extern "C"` simple** (fără `wasm-bindgen`, fără cod de legătură generat) peste fațada Rust stabilă ([API.md](API.md)). Compilat pentru `wasm32-unknown-unknown`, dă un modul care poate deschide un proiect, randa cadre ca text SVG sau pixeli RGBA și rula mașini de stări cu layout, derulare și nivel de detaliu. Nu poate edita, exporta, face control al versiunilor sau rula unelte; nu are sistem de fișiere, rețea, audio sau thread-uri. `osatura.js` este un modul ES fără dependențe (browsere și node >= 18) care ascunde manipularea pointerilor.

Licență: MPL-2.0 (crate și wrapper). Modulul depinde de `osatura-core` cu `default-features = false`: fără motor de scripting și fără niciun crate terț cu licență MPL.

## Construirea

Nu trebuie să îl construiești: ciorna de lansare conține `osatura-web.tar.gz` (modul, wrapper, pagină demo). Ca să îl construiești singur ai nevoie de o mașină pe care ținta poate fi instalată (și, opțional, de `wasm-opt` din Binaryen pentru reducerea dimensiunii). CI îl construiește cu `cargo build --release --target wasm32-unknown-unknown -p osatura-wasm --locked`.

```sh
rustup target add wasm32-unknown-unknown
./osatura-wasm/build-wasm.sh
```

Scriptul (`osatura-wasm/build-wasm.sh`, un script shell POSIX) face exact următoarele, din rădăcina repository-ului:

1. `cargo build --target wasm32-unknown-unknown --release -p osatura-wasm`
2. copiază `target/wasm32-unknown-unknown/release/osatura_wasm.wasm` în `osatura-wasm/osatura_wasm.wasm`;
3. dacă `wasm-opt` se află în `PATH`, rescrie acel fișier cu `wasm-opt -Oz`;
4. afișează dimensiunea rezultatului și un indiciu pentru a rula testul node sau a servi demo-ul.

**Scriptul în sine nu este ceea ce rulează CI** (CI apelează direct `cargo build` și copiază modulul). Indiciul său de final spune `node osatura-wasm/js/test.mjs`; cu un `.wasm` real prezent, testul are nevoie în plus de o cale de proiect ca prim argument (`node osatura-wasm/js/test.mjs project.json`), altfel se oprește cu „pass a project JSON path as argv[2]”. Modulul release din ciornă are aproximativ 0,9 MB ca arhivă comprimată; dimensiunile brute și cele după `wasm-opt -Oz` nu sunt consemnate aici.

Apoi servește folderul prin HTTP (browserele nu încarcă module sau `.wasm` de la `file://`) și deschide pagina demo:

```sh
cd osatura-wasm && python3 -m http.server 8000      # then open http://localhost:8000/index.html
```

Dacă folosești `WebAssembly.instantiateStreaming` (wrapperul o face când îi transmiți un `Response`), serverul trebuie să trimită `Content-Type: application/wasm`; calea prin URL a wrapperului folosește `fetch(...).arrayBuffer()` și nu are o astfel de cerință. Dacă pagina ta setează un Content-Security-Policy, acesta trebuie să permită compilarea WebAssembly (`'wasm-unsafe-eval'` în `script-src`).

O țintă alternativă, `wasm32-wasip1`, se construiește cu `-C panic=unwind`, astfel încât `catch_unwind` să funcționeze (sursa o menționează); nu a fost încercată și are nevoie de un runtime sau de un shim WASI.

## Modelul de memorie și ABI-ul

Modulul deține o **memorie liniară** (exportată ca `memory`). Gazda nu partajează niciodată pointeri cu motorul decât prin aceste reguli:

* **Intrare.** Pentru a transmite octeți (JSON-ul proiectului sau un nume), gazda apelează `arm_alloc(len)`, care returnează un pointer către `len` octeți **zeroizați** (nul când `len` este 0 sau peste 256 MiB). Gazda scrie în `new Uint8Array(memory.buffer, ptr, len)`, apelează funcția cu `(ptr, len)`, apoi eliberează blocul cu `arm_free(ptr, len)`, folosind **același** pointer și aceeași lungime. Motorul copiază ce îi trebuie; nu păstrează niciun pointer în blocul tău.
* **Ieșire.** Rezultatele sunt produse într-un buffer **din interiorul handle-ului**: după un apel care returnează o lungime `n >= 0`, `arm_out_ptr(h)` / `arm_out_len(h)` dau octeții (`n` este egal cu lungimea), iar pentru imagini `arm_out_width(h)` / `arm_out_height(h)` dau dimensiunea (0 după rezultate SVG sau text). Pointerul este valid **până la următorul apel care produce ieșire sau până la `arm_close`**. Gazda trebuie să copieze imediat octeții (wrapperul o face: `.slice()`). Nu păstra niciodată un view `Uint8Array` între apeluri: pe lângă faptul că poate fi suprascris, orice apel poate mări memoria modulului, ceea ce detașează `memory.buffer` și invalidează toate view-urile lui. Recitește `memory.buffer` după fiecare apel.
* **Erori.** O funcție care eșuează returnează `-1` (sau un handle nul) și stochează mesajul; `arm_error_ptr()` / `arm_error_len()` dau octeții lui UTF-8 (fără terminator NUL). Slotul este global pentru modul (acesta rulează pe un singur thread), este suprascris de următorul eșec și **nu este șters de succese**, deci citește-l doar imediat după un eșec.
* **Șiruri de intrare.** `(ptr, len)` UTF-8, `len` între 1 și 4096 (un șir gol sau mai lung este o eroare).
* **Handle-uri.** `arm_open` returnează un pointer opac către un handle (nul la eroare). Un handle conține animația, player-ul opțional și bufferul de ieșire. Închide-l cu `arm_close` exact o dată; `null` este permis.
* **Limite.** JSON de proiect între 1 octet și 256 MiB; `arm_render_rgba` refuză înainte de alocare dacă imaginea ar depăși 256 MiB (`output would be WxH pixels (N bytes), more than the limit of 268435456 bytes: lower the scale`); scală între 0.05 și 4; cadrele din afara timeline-ului sunt limitate; lungimile se returnează ca `i32`. Randarea are nevoie de memorie pentru imagine plus buffere de lucru; spațiul de adrese pe 32 de biți (4 GiB, mai puțin în unele browsere) este plafonul real.
* **Fără thread-uri, fără promisiuni privind panic-urile.** Modulul rulează pe un singur thread. Panic-urile sunt prinse doar pe țintele care fac unwind; pe `wasm32-unknown-unknown` un panic blochează instanța, iar gazda trebuie să o arunce și să creeze una nouă. Motorul este scris să nu dea panic la intrări ale utilizatorului și verifică dimensiunile înainte de alocare, dar acest strat nu poate promite că supraviețuiește unui bug.

## Referința exporturilor

37 de exporturi plus `memory`. `ptr` și `len` sunt `i32` în semnătura wasm (`usize` în Rust). `h` este un handle. Returnarea `-1` = eroare, dacă nu se precizează altfel.

| Export | Semnătură (vedere Rust) | Rezultat |
|---|---|---|
| `arm_abi_version` | `() -> u32` | 1 |
| `arm_set_clock` | `(mono_ms: f64, unix_ms: f64)` | opțional: dă modulului timpul (`performance.now()`, `Date.now()`); valorile invalide sunt ignorate. Adăugat după fixarea versiunii ABI 1; wrapperul îl apelează când este prezent |
| `arm_alloc` | `(len) -> *mut u8` | bloc zeroizat, nul dacă `len` este 0 sau > 256 MiB |
| `arm_free` | `(ptr, len)` | eliberează un bloc de la `arm_alloc` (nul / lungime zero ignorate) |
| `arm_open` | `(json_ptr, json_len) -> *mut Handle` | handle sau nul; validează proiectul |
| `arm_close` | `(h)` | eliberează handle-ul |
| `arm_frame_count`, `arm_fps`, `arm_width`, `arm_height` | `(h) -> u32 / f32 / f32 / f32` | `width` / `height` sunt dimensiunea scenei în unități de scenă (nerotunjită); 0 și o eroare pentru un handle nul |
| `arm_render_svg` | `(h, frame: f32, show_bones: i32) -> i32` | text SVG în bufferul de ieșire; lungimea lui în octeți |
| `arm_render_rgba` | `(h, frame: f32, scale: f32) -> i32` | RGBA8 cu alfa neprefixat în bufferul de ieșire; lungimea în octeți (`w*h*4`); dimensiunea prin `arm_out_width/height` |
| `arm_out_ptr`, `arm_out_len` | `(h) -> *const u8 / usize` | bufferul de ieșire (nul / 0 pentru un handle nul) |
| `arm_out_width`, `arm_out_height` | `(h) -> u32` | dimensiunea imaginii ultimei randări RGBA (0 după text) |
| `arm_machine_count` | `(h) -> u32` | mașinile din proiect |
| `arm_machine_name` | `(h, index) -> i32` | numele în bufferul de ieșire; lungimea lui; -1 pentru un index greșit |
| `arm_player_start` | `(h, name_ptr, name_len) -> i32` | pornește (sau repornește) mașina; 0 |
| `arm_player_stop` | `(h)` | abandonează player-ul |
| `arm_player_set_bool` | `(h, ptr, len, value: i32) -> i32` | 0 |
| `arm_player_set_number` | `(h, ptr, len, value: f64) -> i32` | 0 |
| `arm_player_fire` | `(h, ptr, len) -> i32` | 0 |
| `arm_player_set_value` | `(h, ptr, len, value: f64) -> i32` | 0; valoare externă pentru bindings / intrarea de derulare |
| `arm_player_advance` | `(h, dt: f64) -> i32` | 0; `dt` în `0..=10` secunde |
| `arm_player_state` | `(h, layer_ptr, layer_len) -> i32` | numele stării în bufferul de ieșire; lungimea; -1 strat necunoscut |
| `arm_player_events` | `(h) -> i32` | evenimentele în așteptare ca JSON în bufferul de ieșire, **consumate**; lungimea |
| `arm_player_render_svg` | `(h, show_bones: i32) -> i32` | SVG al poziției curente în bufferul de ieșire |
| `arm_player_render_rgba` | `(h, scale: f32) -> i32` | RGBA al poziției curente; dimensiunea prin `arm_out_width/height` |
| `arm_player_set_canvas` | `(h, w: f32, h: f32) -> i32` | 0 |
| `arm_player_layout` | `(h) -> i32` | JSON `{"canvas":[w,h],"scrolls":[ids],"lod":level}` în bufferul de ieșire |
| `arm_player_scroll_drag` | `(h, id_ptr, id_len, x: f32, y: f32) -> i32` | 0 |
| `arm_player_scroll_release` | `(h, id_ptr, id_len, vx: f32, vy: f32) -> i32` | 0 |
| `arm_player_scroll_info` | `(h, id_ptr, id_len) -> i32` | JSON `{"offset":[x,y],"max":[x,y],"settled":bool}` în bufferul de ieșire |
| `arm_player_set_screen` | `(h, w_px: f32, h_px: f32, max_error_px: f32) -> i32` | 0; alege nivelul de detaliu |
| `arm_error_ptr`, `arm_error_len` | `() -> *const u8 / usize` | textul ultimei erori |

Toate apelurile `arm_player_*`, cu excepția `arm_player_start`, au nevoie de un player pornit (altfel „no player started (call arm_player_start)”). Apelurile player sunt metodele `Player` ale fațadei ([API.md](API.md)).

## Wrapperul JavaScript

`osatura-wasm/js/osatura.js` exportă `class Osatura` (și ca export implicit). Fiecare metodă care schimbă ceva returnează `this`, deci apelurile se pot înlănțui. Erorile sunt aruncate ca `Error` cu mesajul motorului.

| Membru | Descriere |
|---|---|
| `static async instantiate(source, imports = {})` | returnează obiectul brut de exporturi. `source`: un `ArrayBuffer` sau un tablou tipizat (octeți), un `Response` (folosește `instantiateStreaming`; necesită MIME `application/wasm`) sau un șir URL / `URL` (folosește `fetch` + `arrayBuffer`). Modulul nu are importuri, deci `imports` rămâne `{}`. |
| `static async create(source, project)` | `source` este fie exporturi deja instanțiate (un obiect cu `arm_open`), fie orice acceptă `instantiate`; `project` este text JSON sau un obiect simplu (se aplică `JSON.stringify`). Returnează un `Osatura`. |
| `load(project)` | încarcă sau înlocuiește proiectul; aruncă la eșec și atunci **păstrează utilizabil proiectul încărcat anterior**. Completează `frames`, `fps`, `width`, `height`. |
| `frames`, `fps`, `width`, `height` | date de la ultimul `load` (`width` / `height` în unități de scenă, float-uri) |
| `svg(n, showBones = false)` | `string`: document SVG pentru cadrul `n` (cadre fracționare permise, limitate) |
| `rgba(n, scale = 1)` | `{ width, height, data }`: `data` este un `Uint8ClampedArray` (RGBA cu alfa neprefixat, o copie a ta) |
| `frame(n, { kind = 'image', scale = 1, bones = false } = {})` | `kind: 'svg'` dă `{ svg }`; altfel `{ image: ImageData }` într-un browser sau `{ width, height, data }` acolo unde `ImageData` nu există (node) |
| `machines()` | `string[]` numele mașinilor |
| `startPlayer(name)`, `stopPlayer()` | pornește (repornește) / abandonează player-ul; intrările revin la valorile implicite |
| `setBool(name, v)`, `setNumber(name, v)`, `fire(name)`, `setValue(path, v)` | intrări și valori externe; un nume gol aruncă `empty string`, un nume de peste 4096 de octeți aruncă |
| `advance(dt)` | avansează mașina și regiunile de derulare cu `dt` secunde (`0..=10`) |
| `state(layer)` | `string` starea curentă a unui strat-mașină |
| `events()` | `{ time, layer, kind: 'entered' \| 'exited', state }[]`, **consumate** de apel |
| `setCanvas(w, h)` | layout adaptiv pentru o dimensiune de canvas în unități de scenă |
| `layout()` | `{ canvas: [w, h], scrolls: string[], lod: number }` |
| `scrollDrag(id, x, y)`, `scrollRelease(id, vx = 0, vy = 0)` | tragere cu pointerul / viteză la eliberare în px și px/s |
| `scrollInfo(id)` | `{ offset: [x, y], max: [x, y], settled: boolean }` |
| `setScreen(widthPx, heightPx, maxErrorPx = 0.5)` | alege nivelul de detaliu pentru o dimensiune de ecran |
| `playerSvg(showBones = false)`, `playerRgba(scale = 1)` | poziția curentă a player-ului |
| `play(canvas, { scale = 1, loop = true, onFrame } = {})` | desenează timeline-ul pe un canvas 2D cu `requestAnimationFrame` (revine la `setTimeout` în afara unui browser); returnează o funcție de oprire |
| `playMachine(canvas, { scale = 1, onTick } = {})` | avansează mașina pornită cu timpul real scurs (cel mult 0.25 s pe tick) și o desenează; `onTick(dt)` rulează înainte de fiecare avans, deci setează intrările acolo; returnează o funcție de oprire |
| `stop()` | oprește `play` / `playMachine` |
| `close()` | oprește și eliberează handle-ul; apelează-l când ai terminat |

Wrapperul face căutările `x.memory.buffer` în interiorul fiecărui apel și copiază rezultatele cu `.slice()`, deci creșterea memoriei nu poate lăsa un view învechit. Fiecare obiect `Osatura` deține un handle; creează mai multe obiecte (pot partaja același `exports`) pentru a ține mai multe proiecte. Nu partaja un obiect între task-uri async concurente care intercalează apelurile și citirile rezultatelor: rezultatul fiecărui apel stă în bufferul de ieșire al handle-ului până la următorul apel.

### Exemplu (secvența de apeluri)

Aceasta este exact secvența de apeluri pe care o exersează testul node. **A fost rulată față de mock-ul JavaScript** (înlocuitorul din `test.mjs` cu răspunsuri predefinite: mașina `hero`, stratul `body`, intrarea bool `go`, regiunea de derulare `list`), nu față de un modul real.

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

Într-o pagină de browser:

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

## Pagina demo

`osatura-wasm/index.html` este o pagină minimală: importă wrapperul, instanțiază `./osatura_wasm.wasm` și oferă un câmp de fișier (proiect `.json`) și un buton Stop. Alegerea unui fișier creează un `Osatura`, afișează `WIDTHxHEIGHT, N frames @ FPS fps` și pornește `play(canvas)`; erorile motorului sunt afișate în textul de lângă butoane. Dacă modulul nu poate fi descărcat sau compilat, afișează `wasm not found (not built yet): <reason>`. Testată doar în această stare „nefinalizat”, în Chromium headless: pagina se încarcă, importul modulului se rezolvă, apare mesajul (cu pagina 404 a serverului în rolul de „modul”, motivul este `expected magic word 00 61 73 6d, found 3c 21 44 4f`, vezi mai jos). Tot ce depășește acest punct este neverificat.

## Diferențe față de ABI-ul C

| | ABI C ([C_ABI.md](C_ABI.md)) | WebAssembly |
|---|---|---|
| Șiruri de intrare | terminate cu NUL | `(ptr, len)` copiate cu `arm_alloc` |
| Rezultate de ieșire | buffere oferite de apelant | bufferul de ieșire al handle-ului (`arm_out_ptr/len`), valid până la următorul apel cu ieșire |
| Interogarea dimensiunii | un buffer `NULL` returnează dimensiunea | nu este necesară: apelul returnează lungimea, iar datele sunt deja în bufferul de ieșire |
| Textul erorii | șirul NUL `osatura_last_error()`, per thread | octeți `arm_error_ptr/len`, global |
| Matrici de oase | `osatura_bone_world` | neexportat |
| SVG | neexportat | `arm_render_svg`, `arm_player_render_svg` |
| Evenimente | tiparul de interogare a dimensiunii, păstrate dacă bufferul e prea mic | întotdeauna consumate |
| Dimensiunea scenei | întregi rotunjiți | float-uri |

## Aspecte încă neșlefuite

Găsite la citirea codului; niciunul nu a fost văzut eșuând:

* `Osatura.instantiate(url)` verifică `response.ok` și erorile fetch și raportează URL-ul și starea HTTP (teste cu fetch simulat în `osatura-wasm/js/test.mjs`; rulate sub node și, în CI, față de modulul real).
* `frame()` cu `kind: 'svg'` returnează `{ svg }`, nu un șir ca `svg()`.
* Indiciul afișat de `build-wasm.sh` include acum argumentul `<project.json>` de care are nevoie `test.mjs`.
* `play()` desenează convertind fiecare cadru în RGBA pe CPU și apelând `putImageData`; nu există audio, pauză sau căutare (seeking).
* Wrapperul nu exportă declarații TypeScript.

## Lista de verificare pentru prima persoană care îl poate rula într-un browser

Pașii 1 și 5 au fost făcuți de CI pe 24 oct. 2026. Pașii 3, 4, 6, 8, 9, 11 și 13 au fost făcuți în aceeași zi în **Chromium 141 headless (Linux, fără GPU)** cu `osatura-wasm/js/browser-check.js`; cifrele sunt în tabelul de mai jos. **Încă deschise:** Firefox și Safari (pașii 11 și 12 acolo), pasul 12 cu CSP și content-type, adâncimea stivei 10 și pașii 7 și 14 cu playerul. Parcurge-le în ordine; oprește-te la primul eșec și notează mesajul exact. Elementele marcate (risc) corespund riscurilor de mai sus. Când ai rezultate, actualizează [LIMITATIONS.md](LIMITATIONS.md) (secțiunea 1), linia R-090 din secțiunea 9 a ROADMAP.md și caseta de stare a acestei pagini.

1. **Instalează ținta și construiește.** `rustup target add wasm32-unknown-unknown && ./osatura-wasm/build-wasm.sh`. (risc) Așteaptă-te să vezi dacă `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder` și `i_overlay` compilează. Notează compilatorul și `rustc --version`.
2. **Fără motor de scripting.** `cargo tree --target wasm32-unknown-unknown -p osatura-wasm | grep -ci 'rhai\|smartstring'` trebuie să afișeze 0.
3. **Privește exporturile.** În node: `WebAssembly.Module.exports(new WebAssembly.Module(fs.readFileSync('osatura-wasm/osatura_wasm.wasm')))`. Așteaptă-te la `memory` și la cele 37 de funcții din [tabelul exporturilor](#referința-exporturilor), și la **niciun import** (`WebAssembly.Module.imports(...)` gol). Dacă `memory` lipsește sau apar importuri (de exemplu `env.*` dintr-un feature std), wrapperul trebuie adaptat.
4. **Dimensiune.** Notează dimensiunea înainte și după `wasm-opt -Oz`, precum și dimensiunile gzip / brotli. Măsurat o singură dată în rularea Chromium: vezi tabelul de rezultate de la sfârșitul acestei pagini.
5. **Rulează testul node cu un proiect real.** `osatura demo demo.json && node osatura-wasm/js/test.mjs demo.json` trebuie să afișeze `real wasm tests: ok` după cele trei secțiuni de mock.
6. **Compară cu randerul nativ.** Pentru cadrele 0, 12 și 47 ale demo-ului, la scalele 0.5 și 1, compară `Osatura.rgba` cu `osatura render demo.json N out.png --scale S` decodat în RGBA. (risc) Așteaptă-te la diferențe mici la marginile anti-aliasate (bibliotecă matematică diferită); raportează diferența absolută maximă și medie și fracțiunea de pixeli peste 24/255. O diferență mare indică un bug real.
7. **Exersează player-ul.** Construiește proiectul din exemplul ABI-ului C (demo + `take-record` + `clip add` + `machine add`, vezi [C_ABI.md](C_ABI.md)), comandă `startPlayer`, `setBool`, `advance`, `state`, `events`, `playerRgba`; și proiectul cu un container și o regiune de derulare pentru `setCanvas`, `scrollDrag`, `scrollRelease`, `scrollInfo`, `setScreen` (ultimul construiește niveluri de detaliu: măsoară-i timpul, este lent neoptimizat și nu a fost cronometrat pe wasm).
8. **Căi fail-safe.** Oferă un proiect invalid, un șir gol, un fișier aleatoriu de 5 MB, un proiect cu o cale de imagine `source.file` și un proiect cu o coloană sonoră. Fiecare caz trebuie să producă un `Error` aruncat cu un mesaj, nu un trap. (risc) Un trap (`RuntimeError: unreachable`) în oricare dintre ele este un bug de raportat; cei mai probabili candidați sunt `Instant` / `SystemTime` sau apeluri către sistemul de fișiere ajunse pe o cale neașteptată.
9. **Randare mare.** `rgba(0, 4)` pe demo (2560 x 1920) și o scenă de 12000 x 12000 la scala 1 (trebuie refuzată cu mesajul „more than the limit”, nu să rămână fără memorie). Urmărește creșterea memoriei și confirmă că wrapperul încă funcționează după ce memoria a crescut (view-urile sunt recitite la fiecare apel).
10. **Adâncimea stivei.** `wasm32-unknown-unknown` oferă thread-ului principal o stivă implicită mică (1 MiB în toolchain-urile actuale). Deschide un proiect cu grupuri adânc imbricate și un lanț lung de expresii (vezi `osatura-cli/tests/robust.rs` pentru intrări ostile) și confirmă că nu apare un trap de depășire a stivei.
11. **Rulare în browser.** Servește folderul, deschide `index.html` în Chromium și Firefox, alege un proiect, confirmă redarea, redimensionarea, închide și redeschide mai multe proiecte (verificare de scurgeri: memoria nu trebuie să crească fără limită; `close()` eliberează handle-ul).
12. **`instantiateStreaming` și CSP.** Servește cu și fără `Content-Type: application/wasm`, și cu un CSP care omite `'wasm-unsafe-eval'`; documentează ce se întâmplă.
13. **Performanță.** Măsoară timpul `rgba(n, 1)` pentru demo-ul 640 x 480 (nativ release: aproximativ 14 ms pe cadru) și pentru un proiect greu; notează cadrele pe secundă ale lui `play()` în browser.
14. **Opțional: panic-uri.** Decide dacă comportamentul de trap la panic este acceptabil pentru utilizarea ta. Dacă nu, încearcă ruta `wasm32-wasip1` + `-C panic=unwind` menționată în sursă.

### Rezultatele rulării în Chromium (24 oct. 2026)

| Pas | Rezultat |
|---|---|
| 4. Dimensiune | modul release 2.802.534 octeți; 853.804 gzip; 615.267 brotli (netrecut prin `wasm-opt`) |
| 6. Comparație cu nativul | cadrul 12 al demo-ului, 640x480: **0 octeți diferiți** din 1.228.800 (diferența absolută medie și maximă 0) |
| 8. Siguranță la erori | JSON gol, nefolositor, cu formă greșită, 5 MB aleatori și „fără cadre” aruncă fiecare un `Error` cu mesaj; fără trap |
| 9. Randare mare | scara 4 (2560x1920) funcționează; o scenă de 12000x12000 este refuzată cu mesajul de limită, iar modulul funcționează în continuare după aceea |
| 11. Scurgeri | 60 de cicluri de deschidere și închidere: memoria liniară 54.591.488 octeți înainte și după |
| 13. Viteză | `rgba` pentru demo-ul 640x480: mediană 72,5 ms, p95 96,5 ms, maxim 125,4 ms (aproximativ 14 fps; nativ release aproximativ 14 ms) |

Rulează-l din nou cu `node osatura-wasm/js/browser-check.js <unpacked osatura-web folder> <project.json>` (are nevoie de Playwright). Pagina nu a ridicat nicio eroare în timpul rulării.
