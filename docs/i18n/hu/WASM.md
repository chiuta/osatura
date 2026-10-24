# OSATURA WebAssemblyhez

*ABI 1-es verzió (`arm_abi_version()`). Crate: `osatura-wasm`, JavaScript-burkoló: `osatura-wasm/js/osatura.js`, demóoldal: `osatura-wasm/index.html`, build-szkript: `osatura-wasm/build-wasm.sh`. Célközönség: webfejlesztők, akik OSATURA-projekteket szeretnének lejátszani böngészőben vagy node-ban, és az első ember, aki böngészőben is lefuttatja a modult. Dokumentáció: CC-BY-4.0; kódminták: MPL-2.0.*

> ## ÁLLAPOT: A CI-BEN LEFORDÍTVA ÉS TESZTELVE, CSAK HEADLESS CHROMIUMBAN FUTTATTÁK
>
> **2026. okt. 24-től a modul a GitHub Actionsön épül** (`wasm32-unknown-unknown`, debug build a `ci.yml`-ben, release build a `release.yml`-ben), és **a burkoló tesztje sikeres a valódi modul ellen Node 20 alatt** (`node osatura-wasm/js/test.mjs demo.json`, a valódi-modul részét is beleértve). A release build `osatura-web.tar.gz` néven csatolva van a `v0.1.0-alpha.3` előzetes kiadás-vázlathoz (tömörítve kb. 0,9 MB, az `osatura.js` fájllal és a demóoldallal). **Egyszer betöltötték headless Chromium 141-ben Linuxon, GPU nélkül** (az eredmények az oldal végén, az `osatura-wasm/js/browser-check.js` állította elő őket); Firefoxot, Safarit, telefonokat és valódi GPU-kat nem próbáltak, ezért az ezekről szóló állítások ezen az oldalon nem megfigyelt viselkedés, és maga a fejlesztői környezet továbbra sem tudja telepíteni a célt (a szabványkönyvtárának letöltése ott blokkolt).
>
> Ami tesztelve volt, és ez meddig terjed:
>
> | Réteg | Teszt | Mit bizonyít | Mit nem bizonyít |
> |---|---|---|---|
> | A Rust ABI-függvények (`arm_*`) | `cargo test -p osatura-wasm`: 8 egységteszt a **gazdagépen**, az exportált függvényeket közvetlenül hívva | az argumentumok ellenőrzése, a hibakódok, a kimeneti puffer protokollja, az állapotgépek, az elrendezés / görgetés / részletességi szint hívásai, a túlméretes kimenet elutasítása a foglalás előtt | a lefordított modul viselkedését |
> | A lefordított modul | CI: `cargo build --target wasm32-unknown-unknown`, majd `node osatura-wasm/js/test.mjs demo.json` a valódi `.wasm` fájllal | hogy a függőségek lefordulnak wasm32-re, hogy a modul azt exportálja, amit a burkoló vár, és hogy a betöltés, a renderelés és az állapotgép-hívások működnek a Node WebAssembly-motorja alatt | a böngészőket, a release sebességét és memóriahasználatát, a pánikokat (csapda), a demóoldalt |
> | A JavaScript-burkoló | ugyanez a szkript az exportok egy **JavaScript-mockjával** szemben (három mock-szakasz: renderelés, állapotgép, elrendezés) | a burkoló hívássorozatai, a sztringátadás, a hibák fordítása, az eredmények másolása | - |
> | A demóoldal | betöltve headless Chromiumban a `.wasm` fájl **nélkül** | a modulimport-útvonalak feloldódnak, és az oldal összeomlás helyett azt jelzi: „wasm not found” | bármilyen renderelést böngészőben |
> | Függőségek | `cargo tree -p osatura-wasm`: nincs `rhai` / `smartstring` a fában | a szkriptmotor nem része a modulnak | - |
>
> **Fennmaradó kockázatok:** (1) *A CI-ben megoldva:* a `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder`, `i_overlay` és függőségeik fordítása `wasm32-unknown-unknown`-ra. (2) A `std::time::Instant` / `SystemTime`, a `process::id` és a `thread::spawn` a `wasm32-unknown-unknown` célon **pánikol**: a 7. hullám óta a core már nem hívja őket közvetlenül. Minden használat az `osatura_core::clock`-on megy keresztül (egy időbélyeg-típus, `unix_seconds`, `process_id`; a LOD-proxy építő a hívó szálra esik vissza, ha nem lehet szálat indítani), amely ezen a célon egy gazda által adott órát olvas (az `arm_set_clock(mono_ms, unix_ms)` export, amelyet a JavaScript-burkoló a `performance.now()` és a `Date.now()` értékével táplál a `load`, `svg` és `rgba` előtt; az a gazda, amely sosem hívja, 0 időtartamokat kap, így az időkeretek sosem járnak le), és sosem pánikol. Egy teszt (`osatura-core/tests/wasm_safety.rs`) elbukik, ha a `clock.rs`-en kívül új közvetlen használat jelenik meg; a CI-beli Node teszt azokat az utakat járja be, amelyeket elér, nem az összeset. A Rhai (amelynek szintén kellene a `no_time` feature-je) nincs a buildben. (3) A fájlrendszerhez és folyamatokhoz kötött modulok lefordulnak, de futásidőben elbuknak: `audio.rs` (fájlmegnyitás), `soundtrack.rs` (ffprobe), `snapshots.rs`, `open_formats.rs` (exportkönyvtárak), `agent.rs` (mentés), valamint a relatív útvonalú képek és normálletérképek betöltése az `engine.rs`, a `meshtrace.rs` és a `postfx.rs` fájlokban; működés helyett hibát adnak vissza böngészőben, és egyetlen cargo feature sem kapuzza őket. (4) A lebegőpontos függvények (`sin`, `cos`, `powf`, ...) wasm32-n más matematikai könyvtárból jönnek, így a renderelt pixelek kissé eltérhetnek a natívaktól (az ellenőrzési lista 6. lépése ezt méri). (5) A `wasm32-unknown-unknown` célon az alapértelmezés `panic = "abort"`, a `catch_unwind` semmit sem tud elfogni, és egy Rust-pánik az egész példányt csapdába ejti (`WebAssembly.RuntimeError: unreachable`). Lásd a végén az [ellenőrzési listát](#ellenőrzési-lista-az-első-embernek-aki-böngészőben-is-le-tudja-futtatni).

Tartalom: [Mi ez](#mi-ez) - [Építés](#építés) - [Memóriamodell és az ABI](#memóriamodell-és-az-abi) - [Exportreferencia](#exportreferencia) - [A JavaScript-burkoló](#a-javascript-burkoló) - [A demóoldal](#a-demóoldal) - [Eltérések a C ABI-tól](#eltérések-a-c-abi-tól) - [Ismert érdes élek](#ismert-érdes-élek) - [Ellenőrzési lista](#ellenőrzési-lista-az-első-embernek-aki-böngészőben-is-le-tudja-futtatni)

## Mi ez

Az `osatura-wasm` egy `cdylib`, amely **egyszerű `extern "C"` függvényekből** áll (nincs `wasm-bindgen`, nincs generált ragasztókód) a stabil Rust facade ([API.md](API.md)) fölött. `wasm32-unknown-unknown`-ra fordítva egy olyan modult ad, amely meg tud nyitni egy projektet, képkockákat tud renderelni SVG-szövegként vagy RGBA-pixelekként, és állapotgépeket tud futtatni elrendezéssel, görgetéssel és részletességi szinttel. Szerkeszteni, exportálni, verziókezelni vagy eszközöket futtatni nem tud; nincs fájlrendszere, hálózata, hangja és szálai. Az `osatura.js` egy függőségmentes ES-modul (böngészők és node >= 18), amely elrejti a mutatókezelést.

Licenc: MPL-2.0 (crate és burkoló). A modul az `osatura-core`-tól `default-features = false` mellett függ: nincs szkriptmotor és nincs MPL-licencű harmadik féltől származó crate.

## Építés

Nem kell magadnak megépítened: a vázlatkiadás tartalmazza az `osatura-web.tar.gz` fájlt (modul, burkoló, demóoldal). Saját építéshez olyan gép kell, amelyen a cél telepíthető (és opcionálisan a Binaryen `wasm-opt` eszköze a mérethez). A CI a `cargo build --release --target wasm32-unknown-unknown -p osatura-wasm --locked` paranccsal építi.

```sh
rustup target add wasm32-unknown-unknown
./osatura-wasm/build-wasm.sh
```

A szkript (`osatura-wasm/build-wasm.sh`, POSIX shell-szkript) pontosan ezt teszi, a repository gyökeréből:

1. `cargo build --target wasm32-unknown-unknown --release -p osatura-wasm`
2. átmásolja a `target/wasm32-unknown-unknown/release/osatura_wasm.wasm` fájlt az `osatura-wasm/osatura_wasm.wasm` helyre;
3. ha a `wasm-opt` a `PATH`-on van, átírja azt a fájlt `wasm-opt -Oz`-vel;
4. kiírja az eredmény méretét és egy tippet a node-teszt futtatásához vagy a demó kiszolgálásához.

**Magát a szkriptet nem a CI futtatja** (a CI közvetlenül a `cargo build` parancsot hívja, és átmásolja a modult). A záró tippje a `node osatura-wasm/js/test.mjs` parancsot mondja; valódi `.wasm` jelenlétében a tesztnek ezen felül első argumentumként egy projektútvonal is kell (`node osatura-wasm/js/test.mjs project.json`), különben „pass a project JSON path as argv[2]” üzenettel leáll. A vázlatban a release modul tömörített archívumként kb. 0,9 MB; a nyers és a `wasm-opt -Oz` utáni méretet itt nem rögzítettük.

Ezután szolgáld ki a mappát HTTP-n (a böngészők nem töltenek be modulokat vagy `.wasm` fájlt `file://`-ról), és nyisd meg a demóoldalt:

```sh
cd osatura-wasm && python3 -m http.server 8000      # then open http://localhost:8000/index.html
```

Ha a `WebAssembly.instantiateStreaming` függvényt használod (a burkoló ezt teszi, amikor `Response`-t adsz át), a kiszolgálónak `Content-Type: application/wasm` fejlécet kell küldenie; a burkoló URL-útvonala `fetch(...).arrayBuffer()`-t használ, és erre nincs szükség. Ha az oldalad Content-Security-Policy-t állít be, annak engednie kell a WebAssembly fordítását (`'wasm-unsafe-eval'` a `script-src`-ben).

Egy alternatív cél, a `wasm32-wasip1`, `-C panic=unwind` kapcsolóval épül, hogy a `catch_unwind` működjön (a forrás megemlíti); nem próbálták ki, és WASI futtatókörnyezetet vagy shimet igényel.

## Memóriamodell és az ABI

A modul egyetlen **lineáris memóriát** birtokol (`memory` néven exportálva). A gazda sosem oszt meg mutatókat a motorral, csak ezeken a szabályokon keresztül:

* **Bemenet.** Bájtok átadásához (a projekt-JSON vagy egy név) a gazda meghívja az `arm_alloc(len)` függvényt, amely egy `len` **nullázott** bájtra mutató mutatót ad vissza (null, ha a `len` 0 vagy 256 MiB fölötti). A gazda beleír a `new Uint8Array(memory.buffer, ptr, len)` nézetbe, meghívja a függvényt `(ptr, len)` paraméterekkel, majd felszabadítja a blokkot az `arm_free(ptr, len)` függvénnyel **ugyanazzal** a mutatóval és hosszal. A motor lemásolja, amire szüksége van; nem tart meg mutatót a blokkodba.
* **Kimenet.** Az eredmények a handle-ön **belüli** pufferbe készülnek: egy `n >= 0` hosszt visszaadó hívás után az `arm_out_ptr(h)` / `arm_out_len(h)` megadja a bájtokat (`n` egyenlő a hosszal), képeknél pedig az `arm_out_width(h)` / `arm_out_height(h)` a méretet (SVG- vagy szöveges eredmény után 0). A mutató **a következő kimenetet előállító hívásig vagy az `arm_close`-ig** érvényes. A gazdának azonnal ki kell másolnia a bájtokat (a burkoló ezt teszi: `.slice()`). Soha ne tarts meg `Uint8Array` nézetet hívások között: amellett, hogy felülíródik, bármely hívás megnövelheti a modul memóriáját, ami leválasztja a `memory.buffer` objektumot, és érvénytelenít minden nézetet rá. Olvasd újra a `memory.buffer` értékét minden hívás után.
* **Hibák.** Egy elbukó függvény `-1`-et (vagy null handle-t) ad, és eltárolja az üzenetet; az `arm_error_ptr()` / `arm_error_len()` megadja annak UTF-8 bájtjait (nem NUL-terminált). A hely globális a modulra (egyszálas), a következő hiba felülírja, és a sikerek **nem törlik**, ezért csak közvetlenül egy hiba után olvasd.
* **Bemeneti sztringek.** `(ptr, len)` UTF-8, a `len` 1 és 4096 között (az üres vagy hosszabb sztring hiba).
* **Handle-ök.** Az `arm_open` egy átlátszatlan mutatót ad egy handle-re (hiba esetén null). Egy handle tartalmazza az animációt, az opcionális playert és a kimeneti puffert. Pontosan egyszer zárd le az `arm_close`-zal; a `null` megengedett.
* **Korlátok.** Projekt-JSON 1 bájttól 256 MiB-ig; az `arm_render_rgba` a foglalás előtt elutasít, ha a kép meghaladná a 256 MiB-ot (`output would be WxH pixels (N bytes), more than the limit of 268435456 bytes: lower the scale`); a skála 0,05 és 4 között; az idővonalon kívüli képkockákat szorítja; a hosszak `i32`-ként jönnek vissza. A renderelés a képen kívül munkapuffereket is igényel; a 32 bites címtér (4 GiB, egyes böngészőkben kevesebb) a valódi plafon.
* **Nincsenek szálak, pánikra nincs ígéret.** A modul egyszálas. A pánikokat csak azokon a célokon fogjuk el, amelyek visszatekernek (unwind); a `wasm32-unknown-unknown` célon egy pánik csapdába ejti a példányt, és a gazdának el kell dobnia, majd újat létrehoznia. A motor úgy készült, hogy ne pánikoljon felhasználói bemeneten, és foglalás előtt ellenőrzi a méreteket, de ez a réteg nem ígérheti, hogy túléli egy hiba hatását.

## Exportreferencia

37 export plusz a `memory`. A `ptr` és a `len` a wasm-szignatúrában `i32` (Rustban `usize`). A `h` egy handle. A `-1` visszatérés = hiba, hacsak másként nem jelezzük.

| Export | Szignatúra (Rust-nézet) | Eredmény |
|---|---|---|
| `arm_abi_version` | `() -> u32` | 1 |
| `arm_set_clock` | `(mono_ms: f64, unix_ms: f64)` | opcionális: megadja a modulnak az időt (`performance.now()`, `Date.now()`); az érvénytelen értékeket figyelmen kívül hagyja. Az 1-es ABI-verzió rögzítése után adták hozzá; a burkoló hívja, ha jelen van |
| `arm_alloc` | `(len) -> *mut u8` | nullázott blokk, null, ha a `len` 0 vagy > 256 MiB |
| `arm_free` | `(ptr, len)` | felszabadít egy `arm_alloc`-ból származó blokkot (a null / nulla hossz figyelmen kívül marad) |
| `arm_open` | `(json_ptr, json_len) -> *mut Handle` | handle vagy null; validálja a projektet |
| `arm_close` | `(h)` | felszabadítja a handle-t |
| `arm_frame_count`, `arm_fps`, `arm_width`, `arm_height` | `(h) -> u32 / f32 / f32 / f32` | a `width` / `height` a színpad mérete színpad-egységekben (nem kerekített); null handle-re 0 és hiba |
| `arm_render_svg` | `(h, frame: f32, show_bones: i32) -> i32` | SVG-szöveg a kimeneti pufferben; a bájthossza |
| `arm_render_rgba` | `(h, frame: f32, scale: f32) -> i32` | RGBA8 egyenes alfával a kimeneti pufferben; bájthossz (`w*h*4`); a méret az `arm_out_width/height` függvényekkel |
| `arm_out_ptr`, `arm_out_len` | `(h) -> *const u8 / usize` | a kimeneti puffer (null / 0 null handle-re) |
| `arm_out_width`, `arm_out_height` | `(h) -> u32` | az utolsó RGBA-renderelés képmérete (szöveg után 0) |
| `arm_machine_count` | `(h) -> u32` | a projekt gépei |
| `arm_machine_name` | `(h, index) -> i32` | név a kimeneti pufferbe; a hossza; -1 rossz indexre |
| `arm_player_start` | `(h, name_ptr, name_len) -> i32` | a gép indítása (vagy újraindítása); 0 |
| `arm_player_stop` | `(h)` | a player eldobása |
| `arm_player_set_bool` | `(h, ptr, len, value: i32) -> i32` | 0 |
| `arm_player_set_number` | `(h, ptr, len, value: f64) -> i32` | 0 |
| `arm_player_fire` | `(h, ptr, len) -> i32` | 0 |
| `arm_player_set_value` | `(h, ptr, len, value: f64) -> i32` | 0; külső érték kötésekhez / scroll-bemenethez |
| `arm_player_advance` | `(h, dt: f64) -> i32` | 0; a `dt` `0..=10` másodperc |
| `arm_player_state` | `(h, layer_ptr, layer_len) -> i32` | állapotnév a kimeneti pufferbe; a hossz; -1 ismeretlen rétegre |
| `arm_player_events` | `(h) -> i32` | a függő események JSON-ként a kimeneti pufferbe, **elfogyasztva**; a hossz |
| `arm_player_render_svg` | `(h, show_bones: i32) -> i32` | az aktuális póz SVG-je a kimeneti pufferbe |
| `arm_player_render_rgba` | `(h, scale: f32) -> i32` | az aktuális póz RGBA-ja; a méret az `arm_out_width/height` függvényekkel |
| `arm_player_set_canvas` | `(h, w: f32, h: f32) -> i32` | 0 |
| `arm_player_layout` | `(h) -> i32` | JSON `{"canvas":[w,h],"scrolls":[ids],"lod":level}` a kimeneti pufferbe |
| `arm_player_scroll_drag` | `(h, id_ptr, id_len, x: f32, y: f32) -> i32` | 0 |
| `arm_player_scroll_release` | `(h, id_ptr, id_len, vx: f32, vy: f32) -> i32` | 0 |
| `arm_player_scroll_info` | `(h, id_ptr, id_len) -> i32` | JSON `{"offset":[x,y],"max":[x,y],"settled":bool}` a kimeneti pufferbe |
| `arm_player_set_screen` | `(h, w_px: f32, h_px: f32, max_error_px: f32) -> i32` | 0; kiválasztja a részletességi szintet |
| `arm_error_ptr`, `arm_error_len` | `() -> *const u8 / usize` | az utolsó hiba szövege |

Az `arm_player_start` kivételével minden `arm_player_*` hívásnak elindított player kell („no player started (call arm_player_start)” különben). A player hívásai a facade `Player` metódusai ([API.md](API.md)).

## A JavaScript-burkoló

Az `osatura-wasm/js/osatura.js` exportálja a `class Osatura` deklarációt (alapértelmezett exportként is). Minden, valamit módosító metódus `this`-t ad vissza, így a hívások láncolhatók. A hibák `Error`-ként dobódnak a motor üzenetével.

| Tag | Leírás |
|---|---|
| `static async instantiate(source, imports = {})` | a nyers exportobjektumot adja vissza. `source`: egy `ArrayBuffer` vagy típusos tömb (bájtok), egy `Response` (az `instantiateStreaming`-et használja; `application/wasm` MIME kell), vagy egy URL-sztring / `URL` (`fetch` + `arrayBuffer`). A modulnak nincsenek importjai, így az `imports` `{}` marad. |
| `static async create(source, project)` | a `source` vagy már példányosított exportok (egy `arm_open`-nel rendelkező objektum), vagy bármi, amit az `instantiate` elfogad; a `project` JSON-szöveg vagy egyszerű objektum (`JSON.stringify` alkalmazódik rá). `Osatura`-t ad vissza. |
| `load(project)` | projekt betöltése vagy cseréje; hiba esetén dob, és ekkor **az előzőleg betöltött projekt használható marad**. Kitölti a `frames`, `fps`, `width`, `height` tagokat. |
| `frames`, `fps`, `width`, `height` | az utolsó `load` adatai (a `width` / `height` színpad-egységekben, lebegőpontosan) |
| `svg(n, showBones = false)` | `string`: az `n`. képkocka SVG-dokumentuma (törtkockák engedettek, szorítva) |
| `rgba(n, scale = 1)` | `{ width, height, data }`: a `data` egy `Uint8ClampedArray` (egyenes alfás RGBA, a tiéd lévő másolat) |
| `frame(n, { kind = 'image', scale = 1, bones = false } = {})` | a `kind: 'svg'` `{ svg }`-t ad; egyébként `{ image: ImageData }` böngészőben, vagy `{ width, height, data }`, ahol nincs `ImageData` (node) |
| `machines()` | `string[]` a gépek nevei |
| `startPlayer(name)`, `stopPlayer()` | a player indítása (újraindítása) / eldobása; a bemenetek visszaállnak az alapértékükre |
| `setBool(name, v)`, `setNumber(name, v)`, `fire(name)`, `setValue(path, v)` | bemenetek és külső értékek; az üres név `empty string` hibát dob, a 4096 bájtnál hosszabb név is dob |
| `advance(dt)` | a gép és a scroll-régiók léptetése `dt` másodperccel (`0..=10`) |
| `state(layer)` | `string`: egy gépréteg aktuális állapota |
| `events()` | `{ time, layer, kind: 'entered' \| 'exited', state }[]`, a hívás **elfogyasztja** |
| `setCanvas(w, h)` | reszponzív elrendezés egy színpad-egységekben megadott vászonmérethez |
| `layout()` | `{ canvas: [w, h], scrolls: string[], lod: number }` |
| `scrollDrag(id, x, y)`, `scrollRelease(id, vx = 0, vy = 0)` | mutatóhúzás / elengedési sebesség px-ben és px/s-ban |
| `scrollInfo(id)` | `{ offset: [x, y], max: [x, y], settled: boolean }` |
| `setScreen(widthPx, heightPx, maxErrorPx = 0.5)` | a részletességi szint kiválasztása egy képernyőmérethez |
| `playerSvg(showBones = false)`, `playerRgba(scale = 1)` | a player aktuális pózja |
| `play(canvas, { scale = 1, loop = true, onFrame } = {})` | az idővonal kirajzolása egy 2D vászonra `requestAnimationFrame`-mel (böngészőn kívül `setTimeout`-ra esik vissza); egy leállító függvényt ad vissza |
| `playMachine(canvas, { scale = 1, onTick } = {})` | az elindított gép léptetése a valós eltelt idővel (ütemenként legfeljebb 0,25 s) és kirajzolása; az `onTick(dt)` minden léptetés előtt fut, így itt állítsd a bemeneteket; egy leállító függvényt ad vissza |
| `stop()` | a `play` / `playMachine` leállítása |
| `close()` | leállítás és a handle felszabadítása; ha végeztél, hívd |

A burkoló minden hívásban az `x.memory.buffer` lekérdezést végzi, és az eredményeket `.slice()`-szal másolja ki, így a memórianövekedés nem hagyhat nála elavult nézetet. Minden `Osatura` objektum egy handle-t birtokol; több projekt tartásához hozz létre több objektumot (megoszthatnak egyetlen `exports`-ot). Ne ossz meg egy objektumot olyan párhuzamos aszinkron feladatok között, amelyek összefésülik a hívásokat és az eredményolvasásokat: minden hívás eredménye a handle kimeneti pufferében él a következő hívásig.

### Példa (hívássorozat)

Ez pontosan az a hívássorozat, amelyet a node-teszt gyakorol. **A JavaScript-mockkal szemben futott** (a `test.mjs`-ben lévő helyettesítő előre gyártott válaszokkal: `hero` gép, `body` réteg, `go` bool bemenet, `list` scroll-régió), nem valódi modullal.

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

Egy böngészőoldalon:

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

## A demóoldal

Az `osatura-wasm/index.html` egy minimális oldal: importálja a burkolót, példányosítja a `./osatura_wasm.wasm` fájlt, és egy fájlbemenetet (`.json` projekt) meg egy Stop gombot kínál. Egy fájl kiválasztása létrehoz egy `Osatura` objektumot, kiírja: `WIDTHxHEIGHT, N frames @ FPS fps`, és elindítja a `play(canvas)` függvényt; a motor hibái a gombok melletti szövegben jelennek meg. Ha a modul nem tölthető le vagy nem fordítható, a `wasm not found (not built yet): <reason>` üzenetet mutatja. Csak ebben a „nincs megépítve” állapotban tesztelték, headless Chromiumban: az oldal betöltődik, a modulimport feloldódik, az üzenet megjelenik (a kiszolgáló 404-es oldalával mint „modullal” az ok így szól: `expected magic word 00 61 73 6d, found 3c 21 44 4f`, lásd lent). Minden ezen túli nem ellenőrzött.

## Eltérések a C ABI-tól

| | C ABI ([C_ABI.md](C_ABI.md)) | WebAssembly |
|---|---|---|
| Bemeneti sztringek | NUL-terminált | `(ptr, len)`, az `arm_alloc`-kal bemásolva |
| Kimeneti eredmények | a hívó által adott pufferek | a handle kimeneti puffere (`arm_out_ptr/len`), a következő kimeneti hívásig érvényes |
| Méretlekérdezés | a `NULL` puffer a méretet adja vissza | nem szükséges: a hívás a hosszt adja vissza, és az adat már a kimeneti pufferben van |
| Hibaszöveg | `osatura_last_error()` NUL-sztring, szálonként | `arm_error_ptr/len` bájtok, globális |
| Csontmátrixok | `osatura_bone_world` | nem exportált |
| SVG | nem exportált | `arm_render_svg`, `arm_player_render_svg` |
| Események | méretlekérdezéses minta, megmaradnak, ha a puffer túl kicsi | mindig elfogyasztódnak |
| Színpadméret | kerekített egészek | lebegőpontos számok |

## Ismert érdes élek

A kód olvasása közben találtuk; egyiket sem láttuk elbukni:

* Az `Osatura.instantiate(url)` ellenőrzi a `response.ok` értékét és a fetch-hibákat, és jelenti az URL-t és a HTTP-állapotot (mock-fetch tesztek az `osatura-wasm/js/test.mjs` fájlban; node alatt futnak, a CI-ben pedig a valódi modul ellen is).
* A `frame()` `kind: 'svg'` esetén `{ svg }`-t ad vissza, nem sztringet, mint az `svg()`.
* A `build-wasm.sh` által kiírt tipp most már tartalmazza a `<project.json>` argumentumot, amelyre a `test.mjs`-nek szüksége van.
* A `play()` úgy rajzol, hogy minden képkockát a CPU-n RGBA-vá alakít és meghívja a `putImageData` függvényt; nincs hang, nincs szüneteltetés, nincs keresés (seek).
* A burkoló nem exportál TypeScript-deklarációkat.

## Ellenőrzési lista az első embernek, aki böngészőben is le tudja futtatni

Az 1. és az 5. lépést a CI elvégezte 2026. okt. 24-én. A 3., 4., 6., 8., 9., 11. és 13. lépést ugyanazon a napon **headless Chromium 141-ben (Linux, GPU nélkül)** végeztük el az `osatura-wasm/js/browser-check.js` fájllal; a számok az alábbi táblázatban vannak. **Még nyitott:** Firefox és Safari (ott a 11. és 12. lépés), a CSP és a content-type 12. lépés, a 10. veremmélység, valamint a lejátszós 7. és 14. lépés. Ezeket sorrendben végezd el; az első hibánál állj meg, és jegyezd fel a pontos üzenetet. A (kockázat) jelölésű elemek a fenti kockázatoknak felelnek meg. Ha megvannak az eredmények, frissítsd a [LIMITATIONS.md](LIMITATIONS.md) fájlt (1. szakasz), a ROADMAP.md 9. szakaszának R-090 sorát és ennek az oldalnak az állapotdobozát.

1. **Telepítsd a célt és építs.** `rustup target add wasm32-unknown-unknown && ./osatura-wasm/build-wasm.sh`. (kockázat) Meglátod, lefordul-e a `zip`, a `tiny-skia`, a `png`, a `gif`, a `jpeg-decoder` és az `i_overlay`. Jegyezd fel a fordítót és a `rustc --version` kimenetét.
2. **Nincs szkriptmotor.** A `cargo tree --target wasm32-unknown-unknown -p osatura-wasm | grep -ci 'rhai\|smartstring'` parancsnak 0-t kell kiírnia.
3. **Nézd meg az exportokat.** Node-ban: `WebAssembly.Module.exports(new WebAssembly.Module(fs.readFileSync('osatura-wasm/osatura_wasm.wasm')))`. Várható a `memory` és az [exporttábla](#exportreferencia) 37 függvénye, és **nincs import** (a `WebAssembly.Module.imports(...)` üres). Ha a `memory` hiányzik, vagy importok jelennek meg (például `env.*` egy std feature-ből), a burkolót igazítani kell.
4. **Méret.** Jegyezd fel a méretet a `wasm-opt -Oz` előtt és után, valamint a gzip / brotli méreteket. Egyszer megmérték a Chromiumos futásban: lásd az oldal végén lévő eredménytáblát.
5. **Futtasd a node-tesztet valódi projekttel.** Az `osatura demo demo.json && node osatura-wasm/js/test.mjs demo.json` parancsnak a három mock-szakasz után `real wasm tests: ok` üzenetet kell kiírnia.
6. **Hasonlítsd össze a natív renderelővel.** A demó 0., 12. és 47. képkockájára, 0,5-ös és 1-es skálán hasonlítsd össze az `Osatura.rgba` eredményét az `osatura render demo.json N out.png --scale S` RGBA-ra dekódolt kimenetével. (kockázat) Az anti-aliasing éleknél kis eltérések várhatók (más matematikai könyvtár); jelentsd a maximális és az átlagos abszolút eltérést, valamint a 24/255 fölötti pixelek hányadát. Nagy eltérés valódi hibára utal.
7. **Gyakoroltasd a playert.** Építsd fel a C ABI példa projektjét (demo + `take-record` + `clip add` + `machine add`, lásd [C_ABI.md](C_ABI.md)), vezéreld a `startPlayer`, `setBool`, `advance`, `state`, `events`, `playerRgba` függvényekkel; valamint a konténert és scroll-régiót tartalmazó projektet a `setCanvas`, `scrollDrag`, `scrollRelease`, `scrollInfo`, `setScreen` függvényekkel (az utolsó részletességi szinteket épít: mérd az idejét, optimalizálatlanul lassú, és wasm-on nincs lemérve).
8. **Hibabiztos útvonalak.** Adj be egy érvénytelen projektet, egy üres sztringet, egy 5 MB-os véletlen fájlt, egy `source.file` képútvonalat tartalmazó projektet és egy hangsávot tartalmazó projektet. Minden esetnek üzenettel ellátott dobott `Error`-t kell eredményeznie, nem csapdát. (kockázat) Bármelyiknél csapda (`RuntimeError: unreachable`) jelentendő hiba; a legvalószínűbb jelöltek az `Instant` / `SystemTime` vagy a váratlan úton elért fájlrendszer-hívások.
9. **Nagy renderelés.** `rgba(0, 4)` a demón (2560 x 1920), és egy 12000 x 12000-es színpad 1-es skálán (el kell utasítani a „more than the limit” üzenettel, nem elfogyó memóriával). Figyeld a memórianövekedést, és erősítsd meg, hogy a burkoló a memória megnövekedése után is működik (a nézeteket hívásonként újraolvassa).
10. **Veremmélység.** A `wasm32-unknown-unknown` a fő szálnak kicsi alapértelmezett vermet ad (1 MiB a mai toolchainekben). Nyiss meg egy mélyen egymásba ágyazott csoportokat és hosszú kifejezéslánc tartalmazó projektet (a rosszindulatú bemenetekhez lásd az `osatura-cli/tests/robust.rs` fájlt), és erősítsd meg, hogy nincs veremtúlcsordulási csapda.
11. **Böngészős futtatás.** Szolgáld ki a mappát, nyisd meg az `index.html` fájlt Chromiumban és Firefoxban, válassz projektet, erősítsd meg a lejátszást, az átméretezést, zárj le és nyiss meg újra több projektet (szivárgásteszt: a memória nem nőhet korlátlanul; a `close()` felszabadítja a handle-t).
12. **`instantiateStreaming` és CSP.** Szolgálj ki `Content-Type: application/wasm` fejléccel és nélküle, valamint olyan CSP-vel, amely kihagyja a `'wasm-unsafe-eval'` értéket; dokumentáld, mi történik.
13. **Teljesítmény.** Mérd az `rgba(n, 1)` idejét a 640 x 480-as demóra (natív release: kb. 14 ms képkockánként) és egy nehéz projektre; jegyezd fel a `play()` képkocka/másodperc értékét a böngészőben.
14. **Opcionális: pánikok.** Döntsd el, elfogadható-e a pánik esetén csapdába esés viselkedése a felhasználási esetedre. Ha nem, próbáld ki a forrásban említett `wasm32-wasip1` + `-C panic=unwind` útvonalat.

### A Chromium-futtatás eredményei (2026. okt. 24.)

| Lépés | Eredmény |
|---|---|
| 4. Méret | release modul 2,802,534 bájt; 853,804 gzip; 615,267 brotli (nem futott át a `wasm-opt`-on) |
| 6. Natív összevetés | a 12. demókocka, 640x480: **0 eltérő bájt** az 1,228,800-ból (az abszolút eltérés átlaga és maximuma 0) |
| 8. Hibabiztosság | az üres, a szemét, a rossz alakú JSON, az 5 MB véletlen adat és a „no frames” mind `Error`-t dob üzenettel; nincs csapda |
| 9. Nagy render | a 4-es skála (2560x1920) működik; a 12000x12000-es stage-et elutasítja a korlátüzenettel, és a modul utána is működik |
| 11. Szivárgás | 60 nyitás-zárás ciklus: a lineáris memória előtte és utána is 54,591,488 bájt |
| 13. Sebesség | a 640x480-as demó `rgba` hívása: medián 72,5 ms, p95 96,5 ms, maximum 125,4 ms (kb. 14 fps; natív release kb. 14 ms) |

Futtasd újra így: `node osatura-wasm/js/browser-check.js <unpacked osatura-web folder> <project.json>` (Playwright kell hozzá). Az oldal a futás alatt nem dobott hibát.
