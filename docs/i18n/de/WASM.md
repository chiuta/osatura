# OSATURA für WebAssembly

*ABI-Version 1 (`arm_abi_version()`). Crate `osatura-wasm`, JavaScript-Wrapper `osatura-wasm/js/osatura.js`, Demoseite `osatura-wasm/index.html`, Build-Skript `osatura-wasm/build-wasm.sh`. Zielgruppe: Webentwickler, die OSATURA-Projekte in einem Browser oder in Node abspielen wollen, und die erste Person, die das Modul in einem Browser ausführt. Dokumentation CC-BY-4.0; Codebeispiele MPL-2.0.*

> ## STATUS: IN CI GEBAUT UND GETESTET, NUR IN HEADLESS-CHROMIUM AUSGEFÜHRT
>
> **Seit dem 24. Okt. 2026 wird das Modul auf GitHub Actions gebaut** (`wasm32-unknown-unknown`, Debug-Build in `ci.yml`, Release-Build in `release.yml`), und **der Test des Wrappers besteht gegen das echte Modul unter Node 20** (`node osatura-wasm/js/test.mjs demo.json`, einschließlich seines Teils mit dem echten Modul). Der Release-Build hängt als `osatura-web.tar.gz` (etwa 0,9 MB komprimiert, mit `osatura.js` und der Demoseite) an der Entwurfs-Vorabversion `v0.1.0-alpha.3`. **Es wurde einmal in Headless-Chromium 141 unter Linux ohne GPU geladen** (Ergebnisse am Ende dieser Seite, erzeugt von `osatura-wasm/js/browser-check.js`); Firefox, Safari, Telefone und echte GPUs wurden nicht ausprobiert, daher sind Aussagen dazu auf dieser Seite kein beobachtetes Verhalten, und die Entwicklungsumgebung selbst kann das Target weiterhin nicht installieren (der Download seiner Standardbibliothek ist dort blockiert).
>
> Was getestet wurde, und wie weit das reicht:
>
> | Schicht | Test | Belegt | Belegt nicht |
> |---|---|---|---|
> | Die Rust-ABI-Funktionen (`arm_*`) | `cargo test -p osatura-wasm`: 8 Unit-Tests auf dem **Host**, die die exportierten Funktionen direkt aufrufen | Argumentprüfung, Fehlercodes, das Ausgabepuffer-Protokoll, Zustandsautomaten, Aufrufe für Layout / Scrollen / Detailstufen, Ablehnung zu großer Ausgaben vor der Allokation | das Verhalten des kompilierten Moduls |
> | Das kompilierte Modul | CI: `cargo build --target wasm32-unknown-unknown`, dann `node osatura-wasm/js/test.mjs demo.json` mit der echten `.wasm` | dass die Abhängigkeiten für wasm32 kompilieren, dass das Modul exportiert, was der Wrapper erwartet, und dass Laden, Rendern und die Aufrufe des Zustandsautomaten in der WebAssembly-Engine von Node funktionieren | Browser, Geschwindigkeit und Speicher des Release-Builds, Panics (ein Trap), die Demoseite |
> | Der JavaScript-Wrapper | dasselbe Skript gegen einen **JavaScript-Mock** der Exporte (drei Mock-Abschnitte: Rendern, Zustandsautomat, Layout) | die Aufrufabfolgen des Wrappers, Stringübergabe, Fehlerübersetzung, Kopieren der Ergebnisse | - |
> | Die Demoseite | in Headless-Chromium **ohne** die `.wasm`-Datei geladen | die Modulimportpfade lösen sich auf und die Seite meldet „wasm not found“, statt abzustürzen | irgendein Rendering in einem Browser |
> | Abhängigkeiten | `cargo tree -p osatura-wasm`: kein `rhai` / `smartstring` im Baum | die Scripting-Engine ist nicht Teil des Moduls | - |
>
> **Verbleibende Risiken:** (1) *In CI gelöst:* die Kompilierung von `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder`, `i_overlay` und deren Abhängigkeiten für `wasm32-unknown-unknown`. (2) `std::time::Instant` / `SystemTime`, `process::id` und `thread::spawn` **geraten** auf `wasm32-unknown-unknown` **in Panik**: Seit Welle 7 ruft der Core sie nicht mehr direkt auf. Jede Verwendung läuft über `osatura_core::clock` (ein Stamp-Typ, `unix_seconds`, `process_id`; der Builder für LOD-Proxys fällt auf den aufrufenden Thread zurück, wenn kein Thread gestartet werden kann), der auf diesem Target eine vom Host gelieferte Uhr liest (der Export `arm_set_clock(mono_ms, unix_ms)`, vom JavaScript-Wrapper vor `load`, `svg` und `rgba` mit `performance.now()` und `Date.now()` gespeist; ein Host, der ihn nie aufruft, erhält Dauern von 0, sodass Zeitbudgets nie ablaufen) und nie in Panik gerät. Ein Test (`osatura-core/tests/wasm_safety.rs`) schlägt fehl, wenn außerhalb von `clock.rs` eine neue direkte Verwendung auftaucht; der Node-Test in CI übt die Pfade aus, die er erreicht, nicht alle. Rhai (das zusätzlich sein Feature `no_time` bräuchte) ist nicht im Build. (3) Module, die an Dateisystem und Prozesse gebunden sind, kompilieren, schlagen aber zur Laufzeit fehl: `audio.rs` (Datei öffnen), `soundtrack.rs` (ffprobe), `snapshots.rs`, `open_formats.rs` (Exportverzeichnisse), `agent.rs` (Speichern) sowie das Laden von Bildern und Normal Maps mit relativen Pfaden in `engine.rs`, `meshtrace.rs` und `postfx.rs`; sie liefern Fehler, statt in einem Browser zu funktionieren, und kein Cargo-Feature schaltet sie ab. (4) Gleitkommafunktionen (`sin`, `cos`, `powf`, ...) stammen auf wasm32 aus einer anderen Mathematikbibliothek, sodass gerenderte Pixel leicht von nativen abweichen können (Schritt 6 der Checkliste misst das). (5) Auf `wasm32-unknown-unknown` ist `panic = "abort"` der Standard, `catch_unwind` kann nichts abfangen, und eine Rust-Panik lässt die ganze Instanz abstürzen (`WebAssembly.RuntimeError: unreachable`). Siehe die [Verifikations-Checkliste](#verifikations-checkliste-für-die-erste-person-die-es-in-einem-browser-ausführt) am Ende.

Inhalt: [Was es ist](#was-es-ist) - [Bauen](#bauen) - [Speichermodell und die ABI](#speichermodell-und-die-abi) - [Export-Referenz](#export-referenz) - [Der JavaScript-Wrapper](#der-javascript-wrapper) - [Die Demoseite](#die-demoseite) - [Unterschiede zur C-ABI](#unterschiede-zur-c-abi) - [Bekannte Ecken und Kanten](#bekannte-ecken-und-kanten) - [Verifikations-Checkliste](#verifikations-checkliste-für-die-erste-person-die-es-in-einem-browser-ausführt)

## Was es ist

`osatura-wasm` ist eine `cdylib` aus **einfachen `extern "C"`-Funktionen** (kein `wasm-bindgen`, kein generierter Glue-Code) über der stabilen Rust-Fassade ([API.md](API.md)). Für `wasm32-unknown-unknown` kompiliert, ergibt sie ein Modul, das ein Projekt öffnen, Frames als SVG-Text oder RGBA-Pixel rendern und Zustandsautomaten mit Layout, Scrollen und Detailstufen ausführen kann. Es kann nicht bearbeiten, exportieren, Versionskontrolle betreiben oder Werkzeuge ausführen; es hat kein Dateisystem, kein Netzwerk, kein Audio und keine Threads. `osatura.js` ist ein abhängigkeitsfreies ES-Modul (Browser und Node >= 18), das die Zeigerverwaltung verbirgt.

Lizenz: MPL-2.0 (Crate und Wrapper). Das Modul hängt mit `default-features = false` von `osatura-core` ab: keine Scripting-Engine und kein MPL-lizenziertes Third-Party-Crate.

## Bauen

Du musst es nicht selbst bauen: Die Entwurfsversion enthält `osatura-web.tar.gz` (Modul, Wrapper, Demoseite). Zum eigenen Bauen brauchst du eine Maschine, auf der das Target installiert werden kann (und optional `wasm-opt` aus Binaryen für die Größe). CI baut es mit `cargo build --release --target wasm32-unknown-unknown -p osatura-wasm --locked`.

```sh
rustup target add wasm32-unknown-unknown
./osatura-wasm/build-wasm.sh
```

Das Skript (`osatura-wasm/build-wasm.sh`, ein POSIX-Shell-Skript) tut genau dies, vom Wurzelverzeichnis des Repositorys aus:

1. `cargo build --target wasm32-unknown-unknown --release -p osatura-wasm`
2. kopiert `target/wasm32-unknown-unknown/release/osatura_wasm.wasm` nach `osatura-wasm/osatura_wasm.wasm`;
3. wenn `wasm-opt` im `PATH` liegt, schreibt es diese Datei mit `wasm-opt -Oz` neu;
4. gibt die Größe des Ergebnisses und einen Hinweis aus, den Node-Test auszuführen oder die Demo auszuliefern.

**Das Skript selbst führt CI nicht aus** (CI ruft `cargo build` direkt auf und kopiert das Modul). Sein abschließender Hinweis lautet `node osatura-wasm/js/test.mjs`; mit einer echten `.wasm` braucht der Test zusätzlich einen Projektpfad als erstes Argument (`node osatura-wasm/js/test.mjs project.json`), andernfalls bricht er mit „pass a project JSON path as argv[2]“ ab. Das Release-Modul im Entwurf ist als komprimiertes Archiv etwa 0,9 MB groß; die rohe Größe und die Größe nach `wasm-opt -Oz` sind hier nicht festgehalten.

Liefere dann den Ordner über HTTP aus (Browser laden Module oder `.wasm` nicht von `file://`) und öffne die Demoseite:

```sh
cd osatura-wasm && python3 -m http.server 8000      # then open http://localhost:8000/index.html
```

Wenn du `WebAssembly.instantiateStreaming` verwendest (der Wrapper tut das, wenn du eine `Response` übergibst), muss der Server `Content-Type: application/wasm` senden; der URL-Pfad des Wrappers verwendet `fetch(...).arrayBuffer()` und hat diese Anforderung nicht. Wenn deine Seite eine Content-Security-Policy setzt, muss sie die WebAssembly-Kompilierung erlauben (`'wasm-unsafe-eval'` in `script-src`).

Ein alternatives Target, `wasm32-wasip1`, baut mit `-C panic=unwind`, sodass `catch_unwind` funktioniert (der Quelltext erwähnt es); es wurde nicht ausprobiert und braucht eine WASI-Laufzeitumgebung oder einen Shim.

## Speichermodell und die ABI

Das Modul besitzt einen **linearen Speicher** (als `memory` exportiert). Der Host teilt Zeiger mit der Engine ausschließlich nach diesen Regeln:

* **Eingabe.** Um Bytes zu übergeben (das Projekt-JSON oder einen Namen), ruft der Host `arm_alloc(len)` auf, das einen Zeiger auf `len` **mit Nullen gefüllte** Bytes zurückgibt (null, wenn `len` 0 oder größer als 256 MiB ist). Der Host schreibt in `new Uint8Array(memory.buffer, ptr, len)`, ruft die Funktion mit `(ptr, len)` auf und gibt den Block dann mit `arm_free(ptr, len)` frei, mit **demselben** Zeiger und derselben Länge. Die Engine kopiert, was sie braucht; sie behält keinen Zeiger in deinen Block.
* **Ausgabe.** Ergebnisse werden in einen Puffer **innerhalb des Handles** geschrieben: Nach einem Aufruf, der eine Länge `n >= 0` zurückgibt, liefern `arm_out_ptr(h)` / `arm_out_len(h)` die Bytes (`n` entspricht der Länge), und bei Bildern liefern `arm_out_width(h)` / `arm_out_height(h)` die Größe (0 nach SVG- oder Textergebnissen). Der Zeiger ist gültig **bis zum nächsten Aufruf, der Ausgabe erzeugt, oder bis `arm_close`**. Der Host muss die Bytes sofort herauskopieren (der Wrapper tut das: `.slice()`). Behalte nie eine `Uint8Array`-View über Aufrufe hinweg: Sie wird nicht nur überschrieben, sondern jeder Aufruf kann den Speicher des Moduls vergrößern, wodurch `memory.buffer` abgekoppelt wird und jede View darauf ungültig wird. Lies `memory.buffer` nach jedem Aufruf neu.
* **Fehler.** Eine Funktion, die fehlschlägt, gibt `-1` (oder ein Null-Handle) zurück und speichert die Meldung; `arm_error_ptr()` / `arm_error_len()` liefern deren UTF-8-Bytes (nicht NUL-terminiert). Der Slot ist für das Modul global (es ist single-threaded), wird vom nächsten Fehler überschrieben und wird **durch Erfolge nicht gelöscht**; lies ihn daher nur unmittelbar nach einem Fehler.
* **Strings hinein.** `(ptr, len)` UTF-8, `len` zwischen 1 und 4096 (ein leerer oder längerer String ist ein Fehler).
* **Handles.** `arm_open` gibt einen opaken Zeiger auf ein Handle zurück (null bei einem Fehler). Ein Handle enthält die Animation, den optionalen Player und den Ausgabepuffer. Schließe es genau einmal mit `arm_close`; `null` ist erlaubt.
* **Limits.** Projekt-JSON 1 Byte bis 256 MiB; `arm_render_rgba` lehnt vor der Allokation ab, wenn das Bild 256 MiB überschreiten würde (`output would be WxH pixels (N bytes), more than the limit of 268435456 bytes: lower the scale`); Skalierung zwischen 0.05 und 4; Frames außerhalb der Timeline werden begrenzt; Längen werden als `i32` zurückgegeben. Das Rendern braucht Speicher für das Bild plus Arbeitspuffer; der 32-Bit-Adressraum (4 GiB, in manchen Browsern weniger) ist die eigentliche Obergrenze.
* **Keine Threads, keine versprochenen Panics.** Das Modul ist single-threaded. Panics werden nur auf Targets abgefangen, die unwinden; auf `wasm32-unknown-unknown` lässt eine Panik die Instanz abstürzen, und der Host muss sie verwerfen und eine neue erstellen. Die Engine ist so geschrieben, dass sie bei Benutzereingaben nicht in Panik gerät, und prüft Größen vor dem Allozieren, aber diese Schicht kann nicht versprechen, einen Bug zu überleben.

## Export-Referenz

37 Exporte plus `memory`. `ptr` und `len` sind in der wasm-Signatur `i32` (in Rust `usize`). `h` ist ein Handle. Rückgabe `-1` = Fehler, sofern nicht anders angegeben.

| Export | Signatur (Rust-Sicht) | Ergebnis |
|---|---|---|
| `arm_abi_version` | `() -> u32` | 1 |
| `arm_set_clock` | `(mono_ms: f64, unix_ms: f64)` | optional: gibt dem Modul die Zeit (`performance.now()`, `Date.now()`); ungültige Werte werden ignoriert. Hinzugefügt, nachdem die ABI-Version 1 festgelegt war; der Wrapper ruft sie auf, wenn vorhanden |
| `arm_alloc` | `(len) -> *mut u8` | mit Nullen gefüllter Block, null, wenn `len` 0 oder > 256 MiB ist |
| `arm_free` | `(ptr, len)` | gibt einen Block aus `arm_alloc` frei (null / Länge null wird ignoriert) |
| `arm_open` | `(json_ptr, json_len) -> *mut Handle` | Handle oder null; validiert das Projekt |
| `arm_close` | `(h)` | gibt das Handle frei |
| `arm_frame_count`, `arm_fps`, `arm_width`, `arm_height` | `(h) -> u32 / f32 / f32 / f32` | `width` / `height` sind die Bühnengröße in Bühneneinheiten (nicht gerundet); 0 und ein Fehler bei einem Null-Handle |
| `arm_render_svg` | `(h, frame: f32, show_bones: i32) -> i32` | SVG-Text im Ausgabepuffer; dessen Bytelänge |
| `arm_render_rgba` | `(h, frame: f32, scale: f32) -> i32` | RGBA8 mit Straight-Alpha im Ausgabepuffer; Bytelänge (`w*h*4`); Größe über `arm_out_width/height` |
| `arm_out_ptr`, `arm_out_len` | `(h) -> *const u8 / usize` | der Ausgabepuffer (null / 0 bei einem Null-Handle) |
| `arm_out_width`, `arm_out_height` | `(h) -> u32` | Bildgröße des letzten RGBA-Renderings (0 nach Text) |
| `arm_machine_count` | `(h) -> u32` | Automaten im Projekt |
| `arm_machine_name` | `(h, index) -> i32` | Name in den Ausgabepuffer; dessen Länge; -1 bei ungültigem Index |
| `arm_player_start` | `(h, name_ptr, name_len) -> i32` | startet (oder startet neu) den Automaten; 0 |
| `arm_player_stop` | `(h)` | verwirft den Player |
| `arm_player_set_bool` | `(h, ptr, len, value: i32) -> i32` | 0 |
| `arm_player_set_number` | `(h, ptr, len, value: f64) -> i32` | 0 |
| `arm_player_fire` | `(h, ptr, len) -> i32` | 0 |
| `arm_player_set_value` | `(h, ptr, len, value: f64) -> i32` | 0; externer Wert für Bindungen / Scroll-Eingabe |
| `arm_player_advance` | `(h, dt: f64) -> i32` | 0; `dt` in `0..=10` Sekunden |
| `arm_player_state` | `(h, layer_ptr, layer_len) -> i32` | Zustandsname in den Ausgabepuffer; Länge; -1 bei unbekannter Ebene |
| `arm_player_events` | `(h) -> i32` | anstehende Ereignisse als JSON in den Ausgabepuffer, **verbraucht**; Länge |
| `arm_player_render_svg` | `(h, show_bones: i32) -> i32` | SVG der aktuellen Pose in den Ausgabepuffer |
| `arm_player_render_rgba` | `(h, scale: f32) -> i32` | RGBA der aktuellen Pose; Größe über `arm_out_width/height` |
| `arm_player_set_canvas` | `(h, w: f32, h: f32) -> i32` | 0 |
| `arm_player_layout` | `(h) -> i32` | JSON `{"canvas":[w,h],"scrolls":[ids],"lod":level}` in den Ausgabepuffer |
| `arm_player_scroll_drag` | `(h, id_ptr, id_len, x: f32, y: f32) -> i32` | 0 |
| `arm_player_scroll_release` | `(h, id_ptr, id_len, vx: f32, vy: f32) -> i32` | 0 |
| `arm_player_scroll_info` | `(h, id_ptr, id_len) -> i32` | JSON `{"offset":[x,y],"max":[x,y],"settled":bool}` in den Ausgabepuffer |
| `arm_player_set_screen` | `(h, w_px: f32, h_px: f32, max_error_px: f32) -> i32` | 0; wählt die Detailstufe |
| `arm_error_ptr`, `arm_error_len` | `() -> *const u8 / usize` | letzter Fehlertext |

Alle `arm_player_*`-Aufrufe außer `arm_player_start` brauchen einen gestarteten Player (andernfalls „no player started (call arm_player_start)“). Die Player-Aufrufe sind die `Player`-Methoden der Fassade ([API.md](API.md)).

## Der JavaScript-Wrapper

`osatura-wasm/js/osatura.js` exportiert `class Osatura` (auch als Default-Export). Jede Methode, die etwas ändert, gibt `this` zurück, sodass sich Aufrufe verketten lassen. Fehler werden als `Error` mit der Meldung der Engine geworfen.

| Member | Beschreibung |
|---|---|
| `static async instantiate(source, imports = {})` | gibt das rohe Exports-Objekt zurück. `source`: ein `ArrayBuffer` oder Typed Array (Bytes), eine `Response` (verwendet `instantiateStreaming`; braucht den MIME-Typ `application/wasm`) oder ein URL-String / `URL` (verwendet `fetch` + `arrayBuffer`). Das Modul hat keine Imports, daher bleibt `imports` `{}`. |
| `static async create(source, project)` | `source` ist entweder bereits instanziierte Exporte (ein Objekt mit `arm_open`) oder alles, was `instantiate` akzeptiert; `project` ist JSON-Text oder ein einfaches Objekt (`JSON.stringify` wird angewendet). Gibt eine `Osatura` zurück. |
| `load(project)` | lädt das Projekt oder ersetzt es; wirft bei einem Fehler und **lässt dann das zuvor geladene Projekt weiter benutzbar**. Füllt `frames`, `fps`, `width`, `height`. |
| `frames`, `fps`, `width`, `height` | Fakten aus dem letzten `load` (`width` / `height` in Bühneneinheiten, Floats) |
| `svg(n, showBones = false)` | `string`: SVG-Dokument für Frame `n` (gebrochene Frames erlaubt, begrenzt) |
| `rgba(n, scale = 1)` | `{ width, height, data }`: `data` ist ein `Uint8ClampedArray` (RGBA mit Straight-Alpha, eine Kopie, die dir gehört) |
| `frame(n, { kind = 'image', scale = 1, bones = false } = {})` | `kind: 'svg'` ergibt `{ svg }`; andernfalls `{ image: ImageData }` in einem Browser oder `{ width, height, data }`, wo `ImageData` nicht existiert (Node) |
| `machines()` | `string[]` Automatennamen |
| `startPlayer(name)`, `stopPlayer()` | startet (startet neu) / verwirft den Player; Eingaben kehren zu ihren Standardwerten zurück |
| `setBool(name, v)`, `setNumber(name, v)`, `fire(name)`, `setValue(path, v)` | Eingaben und externe Werte; ein leerer Name wirft `empty string`, ein Name über 4096 Bytes wirft ebenfalls |
| `advance(dt)` | schaltet Automat und Scroll-Regionen um `dt` Sekunden weiter (`0..=10`) |
| `state(layer)` | `string` aktueller Zustand einer Automatenebene |
| `events()` | `{ time, layer, kind: 'entered' \| 'exited', state }[]`, vom Aufruf **verbraucht** |
| `setCanvas(w, h)` | responsives Layout für eine Canvas-Größe in Bühneneinheiten |
| `layout()` | `{ canvas: [w, h], scrolls: string[], lod: number }` |
| `scrollDrag(id, x, y)`, `scrollRelease(id, vx = 0, vy = 0)` | Zeiger-Drag / Loslass-Geschwindigkeit in px und px/s |
| `scrollInfo(id)` | `{ offset: [x, y], max: [x, y], settled: boolean }` |
| `setScreen(widthPx, heightPx, maxErrorPx = 0.5)` | wählt die Detailstufe für eine Bildschirmgröße |
| `playerSvg(showBones = false)`, `playerRgba(scale = 1)` | die aktuelle Pose des Players |
| `play(canvas, { scale = 1, loop = true, onFrame } = {})` | zeichnet die Timeline mit `requestAnimationFrame` auf ein 2D-Canvas (fällt außerhalb eines Browsers auf `setTimeout` zurück); gibt eine Stopp-Funktion zurück |
| `playMachine(canvas, { scale = 1, onTick } = {})` | schaltet den gestarteten Automaten um die real verstrichene Zeit weiter (höchstens 0.25 s pro Tick) und zeichnet ihn; `onTick(dt)` läuft vor jedem Weiterschalten, setze Eingaben also dort; gibt eine Stopp-Funktion zurück |
| `stop()` | stoppt `play` / `playMachine` |
| `close()` | stoppt und gibt das Handle frei; rufe es auf, wenn du fertig bist |

Der Wrapper führt die Abfragen von `x.memory.buffer` innerhalb jedes Aufrufs aus und kopiert Ergebnisse mit `.slice()` heraus, sodass Speicherwachstum ihm keine veraltete View hinterlassen kann. Jedes `Osatura`-Objekt besitzt ein Handle; erstelle mehrere Objekte (sie dürfen sich ein `exports` teilen), um mehrere Projekte zu halten. Teile ein Objekt nicht zwischen nebenläufigen asynchronen Aufgaben, die Aufrufe und Ergebnislesungen verschränken: Das Ergebnis jedes Aufrufs liegt bis zum nächsten Aufruf im Ausgabepuffer des Handles.

### Beispiel (Aufrufabfolge)

Dies ist genau die Aufrufabfolge, die der Node-Test ausführt. **Sie wurde gegen den JavaScript-Mock ausgeführt** (den Stellvertreter in `test.mjs` mit vorgefertigten Antworten: Automat `hero`, Ebene `body`, Bool-Eingabe `go`, Scroll-Region `list`), nicht gegen ein echtes Modul.

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

Auf einer Browserseite:

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

## Die Demoseite

`osatura-wasm/index.html` ist eine minimale Seite: Sie importiert den Wrapper, instanziiert `./osatura_wasm.wasm` und bietet ein Dateieingabefeld (`.json`-Projekt) und einen Stopp-Button. Die Auswahl einer Datei erstellt eine `Osatura`, zeigt `WIDTHxHEIGHT, N frames @ FPS fps` an und startet `play(canvas)`; Fehler der Engine werden in dem Text neben den Buttons angezeigt. Wenn das Modul nicht abgerufen oder kompiliert werden kann, zeigt sie `wasm not found (not built yet): <reason>`. Nur in diesem Zustand „nicht gebaut“ getestet, in Headless-Chromium: Die Seite lädt, der Modulimport löst sich auf, die Meldung erscheint (mit der 404-Seite des Servers als „Modul“ lautet der Grund `expected magic word 00 61 73 6d, found 3c 21 44 4f`, siehe unten). Alles darüber hinaus ist unverifiziert.

## Unterschiede zur C-ABI

| | C-ABI ([C_ABI.md](C_ABI.md)) | WebAssembly |
|---|---|---|
| Strings hinein | NUL-terminiert | `(ptr, len)`, mit `arm_alloc` hineinkopiert |
| Ergebnisse hinaus | vom Aufrufer bereitgestellte Puffer | der Ausgabepuffer des Handles (`arm_out_ptr/len`), gültig bis zum nächsten Aufruf mit Ausgabe |
| Größenabfrage | ein `NULL`-Puffer gibt die Größe zurück | nicht nötig: Der Aufruf gibt die Länge zurück und die Daten liegen schon im Ausgabepuffer |
| Fehlertext | `osatura_last_error()` NUL-String, pro Thread | `arm_error_ptr/len` Bytes, global |
| Knochenmatrizen | `osatura_bone_world` | nicht exportiert |
| SVG | nicht exportiert | `arm_render_svg`, `arm_player_render_svg` |
| Ereignisse | Größenabfrage-Muster, bleiben erhalten, wenn der Puffer zu klein ist | immer verbraucht |
| Bühnengröße | gerundete Ganzzahlen | Floats |

## Bekannte Ecken und Kanten

Beim Lesen des Codes gefunden; keine davon wurde je als fehlschlagend beobachtet:

* `Osatura.instantiate(url)` prüft `response.ok` und Fetch-Fehler und meldet die URL und den HTTP-Status (Mock-Fetch-Tests in `osatura-wasm/js/test.mjs`; unter Node ausgeführt, und in CI gegen das echte Modul).
* `frame()` mit `kind: 'svg'` gibt `{ svg }` zurück, nicht wie `svg()` einen String.
* Der von `build-wasm.sh` ausgegebene Hinweis enthält jetzt das Argument `<project.json>`, das `test.mjs` braucht.
* `play()` zeichnet, indem es jeden Frame auf der CPU in RGBA umwandelt und `putImageData` aufruft; es gibt kein Audio, kein Pausieren, kein Springen.
* Der Wrapper exportiert keine TypeScript-Deklarationen.

## Verifikations-Checkliste für die erste Person, die es in einem Browser ausführt

Die Schritte 1 und 5 hat CI am 24. Okt. 2026 erledigt. Die Schritte 3, 4, 6, 8, 9, 11 und 13 wurden am selben Tag in **Headless-Chromium 141 (Linux, keine GPU)** mit `osatura-wasm/js/browser-check.js` erledigt; die Zahlen stehen in der Tabelle unten. **Noch offen:** Firefox und Safari (dort die Schritte 11 und 12), der CSP- und Content-Type-Schritt 12, Stacktiefe 10 und die Player-Schritte 7 und 14. Arbeite sie der Reihe nach ab; halte beim ersten Fehlschlag an und notiere die genaue Meldung. Mit (risk) markierte Punkte entsprechen den oben genannten Risiken. Wenn du Ergebnisse hast, aktualisiere [LIMITATIONS.md](LIMITATIONS.md) (Abschnitt 1), die Zeile R-090 in ROADMAP.md Abschnitt 9 und die Statusbox dieser Seite.

1. **Target installieren und bauen.** `rustup target add wasm32-unknown-unknown && ./osatura-wasm/build-wasm.sh`. (risk) Du wirst sehen, ob `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder` und `i_overlay` kompilieren. Notiere den Compiler und `rustc --version`.
2. **Keine Scripting-Engine.** `cargo tree --target wasm32-unknown-unknown -p osatura-wasm | grep -ci 'rhai\|smartstring'` muss 0 ausgeben.
3. **Die Exporte ansehen.** In Node: `WebAssembly.Module.exports(new WebAssembly.Module(fs.readFileSync('osatura-wasm/osatura_wasm.wasm')))`. Erwartet werden `memory` und die 37 Funktionen der [Export-Tabelle](#export-referenz) und **keine Imports** (`WebAssembly.Module.imports(...)` leer). Fehlt `memory` oder tauchen Imports auf (zum Beispiel `env.*` aus einem Std-Feature), muss der Wrapper angepasst werden.
4. **Größe.** Notiere die Größe vor und nach `wasm-opt -Oz` sowie die Größen mit gzip / brotli. Einmal im Chromium-Lauf gemessen: siehe die Ergebnistabelle am Ende dieser Seite.
5. **Den Node-Test mit einem echten Projekt ausführen.** `osatura demo demo.json && node osatura-wasm/js/test.mjs demo.json` muss nach den drei Mock-Abschnitten `real wasm tests: ok` ausgeben.
6. **Mit dem nativen Renderer vergleichen.** Vergleiche für die Frames 0, 12 und 47 der Demo bei den Skalierungen 0.5 und 1 `Osatura.rgba` mit `osatura render demo.json N out.png --scale S`, nach RGBA dekodiert. (risk) Erwarte kleine Unterschiede an anti-aliasten Kanten (andere Mathematikbibliothek); berichte die maximale und die mittlere absolute Abweichung und den Anteil der Pixel über 24/255. Eine große Abweichung deutet auf einen echten Fehler hin.
7. **Den Player ausprobieren.** Baue das Projekt des C-ABI-Beispiels (Demo + `take-record` + `clip add` + `machine add`, siehe [C_ABI.md](C_ABI.md)), steuere `startPlayer`, `setBool`, `advance`, `state`, `events`, `playerRgba`; und das Projekt mit einem Container und einer Scroll-Region für `setCanvas`, `scrollDrag`, `scrollRelease`, `scrollInfo`, `setScreen` (Letzteres baut Detailstufen: miss die Zeit, es ist unoptimiert langsam und wurde auf wasm nicht gemessen).
8. **Pfade für sicheres Scheitern.** Gib ein ungültiges Projekt, einen leeren String, eine 5 MB große Zufallsdatei, ein Projekt mit einem Bildpfad in `source.file` und ein Projekt mit einer Tonspur ein. Jeder Fall muss einen geworfenen `Error` mit einer Meldung erzeugen, keinen Trap. (risk) Ein Trap (`RuntimeError: unreachable`) bei einem davon ist ein zu meldender Bug; die wahrscheinlichsten Kandidaten sind `Instant` / `SystemTime` oder Dateisystemaufrufe, die über einen unerwarteten Pfad erreicht werden.
9. **Großes Rendering.** `rgba(0, 4)` auf der Demo (2560 x 1920) und eine Bühne von 12000 x 12000 bei Skalierung 1 (muss mit der Meldung „more than the limit“ abgelehnt werden, nicht den Speicher erschöpfen). Beobachte das Speicherwachstum und bestätige, dass der Wrapper nach dem Wachstum des Speichers noch funktioniert (Views werden pro Aufruf neu gelesen).
10. **Stacktiefe.** `wasm32-unknown-unknown` gibt dem Haupt-Thread einen kleinen Standard-Stack (1 MiB in aktuellen Toolchains). Öffne ein Projekt mit tief verschachtelten Gruppen und einer langen Ausdruckskette (feindselige Eingaben siehe `osatura-cli/tests/robust.rs`) und bestätige, dass kein Stack-Overflow-Trap auftritt.
11. **Lauf im Browser.** Liefere den Ordner aus, öffne `index.html` in Chromium und Firefox, wähle ein Projekt, bestätige die Wiedergabe, ändere die Größe, schließe und öffne mehrere Projekte erneut (Leak-Prüfung: Der Speicher sollte nicht unbegrenzt wachsen; `close()` gibt das Handle frei).
12. **`instantiateStreaming` und CSP.** Liefere mit und ohne `Content-Type: application/wasm` aus und mit einer CSP, die `'wasm-unsafe-eval'` auslässt; dokumentiere, was passiert.
13. **Performance.** Miss `rgba(n, 1)` für die Demo mit 640 x 480 (nativ im Release: etwa 14 ms pro Frame) und ein aufwendiges Projekt; notiere die Frames pro Sekunde von `play()` im Browser.
14. **Optional: Panics.** Entscheide, ob das Verhalten „Trap bei Panik“ für deinen Anwendungsfall akzeptabel ist. Falls nicht, probiere den im Quelltext erwähnten Weg über `wasm32-wasip1` + `-C panic=unwind`.

### Ergebnisse des Chromium-Laufs (24. Okt. 2026)

| Schritt | Ergebnis |
|---|---|
| 4. Größe | Release-Modul 2,802,534 Bytes; 853,804 gzip; 615,267 brotli (nicht durch `wasm-opt` geschickt) |
| 6. Vergleich mit nativ | Demo-Frame 12, 640x480: **0 abweichende Bytes** von 1,228,800 (mittlere und maximale absolute Abweichung 0) |
| 8. Sicheres Scheitern | leer, Müll, JSON mit falscher Form, 5 MB Zufallsdaten und „no frames“ lösen jeweils einen `Error` mit Meldung aus; kein Trap |
| 9. Großes Rendering | Skalierung 4 (2560x1920) funktioniert; eine Bühne von 12000x12000 wird mit der Limit-Meldung abgelehnt, und das Modul funktioniert danach weiter |
| 11. Leck | 60 Öffnen-/Schließen-Zyklen: linearer Speicher 54,591,488 Bytes davor und danach |
| 13. Geschwindigkeit | `rgba` der Demo mit 640x480: Median 72.5 ms, p95 96.5 ms, Maximum 125.4 ms (etwa 14 fps; nativ im Release etwa 14 ms) |

Führe ihn erneut aus mit `node osatura-wasm/js/browser-check.js <entpackter osatura-web-Ordner> <project.json>` (braucht Playwright). Die Seite löste während des Laufs keine Fehler aus.
