<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA

**Un motor de rigging și animație vectorială 2D scris în Rust: oase, cinematică inversă (IK), skinning, deformatoare, un runtime cu mașini de stări, o trusă de instrumente în linia de comandă, un studio în browser și biblioteci integrabile, într-un singur workspace.** Versiunea engleză: [README.md](README.md). Documentația din `docs/` este în limba engleză.

![Un personaj văzut din lateral (cap, ureche, coadă, trunchi, brațe cu degete, picioare, labe) merge pe loc pe un sol care se derulează; ieșirea reală a comenzii osatura export pe proiectul demo încorporat](docs/img/walk.gif)

*Ce produce: omulețul care merge, inclus în aplicație, exportat cu `osatura demo-person` și `osatura export` (tot proiectul este un singur fișier JSON). Încearcă în două minute: [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).*

## Rezumat și filozofie

OSATURA îți permite să construiești un personaj sau o scenă din forme vectoriale și imagini, să le echipezi cu oase și constrângeri, să le animezi cu chei, clipuri și fizică și să le exporți ca GIF, video, Lottie, glTF, un player HTML autonom sau un runtime interactiv. Este alcătuit din piese mici și deterministe: același proiect și același seed dau aceiași pixeli, fiecare instrument citește și scrie un proiect JSON documentat, iar fiecare comandă funcționează dintr-un terminal, așa că animația poate fi scriptată, revizuită, comparată și testată ca un cod sursă.

Principiile proiectului:

* **Implementare independentă.** Scris pe baza specificațiilor publice, a articolelor științifice și a matematicii din manuale. Listele de funcții ale competitorilor au fost consultate doar pentru *domeniul* de acoperire; nu a fost copiat niciun cod sursă străin, iar proiectul nu citește și nu scrie niciun format proprietar. Nu este un proces „clean-room” strict (vezi [CLEANROOM.md](CLEANROOM.md), inclusiv declarația despre cercetare) și nu este afiliat cu, derivat din sau compatibil cu Moho ori cu alt produs comercial. Codul a fost scris în mare parte cu ajutorul unui model de inteligență artificială.
* **Stare descrisă cinstit.** Fiecare funcție este descrisă împreună cu ceea ce *nu* face ([docs/LIMITATIONS.md](docs/LIMITATIONS.md), [ROADMAP.md](ROADMAP.md) secțiunea 9).
* **Sigur în mod implicit.** Fără panici la date de intrare, verificări ale valorilor finite, limite de resurse, scrieri atomice, căi de resurse limitate la folderul proiectului ([docs/SECURITY.md](docs/SECURITY.md)).
* **Formate deschise mai întâi.** OpenRaster, OCA, SVG, Lottie, glTF, APNG, WebM, EXR; niciun format de fișier proprietar, intenționat.
* **Dependențe puține**, cu o politică de licențe verificată prin teste și prin `cargo deny`.

## Stare

*Nume: proiectul s-a numit ARMATURA până la 24 oct. 2026 și a fost redenumit OSATURA în urma unei căutări de mărci ([docs/LIMITATIONS.md](docs/LIMITATIONS.md) L-78). Depozitul și adresa site-ului au fost redenumite și ele (linkurile vechi către depozit redirecționează).*

> **Alpha. Primele build-uri sunt pregătite ca pre-lansare ciornă: `v0.1.0-alpha.3`.** Executabilele pentru Windows x86-64, macOS arm64 și Linux x86-64 și modulul WebAssembly sunt construite de GitHub Actions și atașate la o pre-lansare *ciornă* pe <https://github.com/chiuta/osatura/releases> (o ciornă este vizibilă doar proprietarului până când este publicată manual). Toate crate-urile au în continuare versiunea `0.1.0`; „v0.5 / v0.6 / v0.7” sunt *niveluri de funcționalitate* în istoric, nu numere de lansare ([CHANGELOG.md](CHANGELOG.md)).
>
> | Verificat | Neverificat |
> |---|---|
> | **Local (Linux x86-64, 24 oct. 2026):** 1338 de teste din `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` plus 19 doc-teste trec (crate-ul GPU are nevoie de un adaptor, iar crate-ul ferestrei de un desktop; înainte de aceasta, numărul incluzând crate-ul GPU era 1368). **Pe GitHub Actions (24 oct. 2026, toolchain 1.97.0):** suita de teste și `clippy -D warnings` trec pe Linux, macOS (arm64) și Windows; `cargo deny` și `reuse lint` trec; modulul WebAssembly se construiește pentru `wasm32-unknown-unknown`, iar testul wrapper-ului trece față de modulul real sub Node 20; binarele de lansare trec un test de fum (`--version`, `demo`, `check`, `render`), iar `osatura-studio --selftest` rulează pe Windows și macOS. Patru teste de fum ale studioului în Chromium fără interfață trec; CLI-ul, fațada Rust și ABI-ul C (inclusiv sub ASan/UBSan) au fost rulate cu exemplele din documentație | **Nicio persoană nu a folosit executabilele pe un desktop real Windows sau macOS**: lansatorul Windows prin dublu clic și aplicația cu fereastră `osatura-studio` (WebView2 / WKWebView) au fost doar compilate și autotestate în CI; binarele **nu sunt semnate cu cod** (SmartScreen „Unknown publisher”, macOS Gatekeeper); binarul Linux a fost construit doar pe un runner Ubuntu; **modulul WebAssembly nu a fost niciodată rulat într-un browser**, iar viteza și memoria lui nu sunt măsurate; studioul nu a fost folosit de o persoană în producție și nici într-un alt browser decât Chromium; nu s-a folosit niciodată un GPU real (doar un adaptor software), o cameră web, o fotografie sau un fișier de captură de mișcare; fără audit de securitate, revizuire juridică sau căutare de brevete |
>
> Citește [docs/LIMITATIONS.md](docs/LIMITATIONS.md) (78 de limite numerotate, cu gravitate și soluție ocolitoare) înainte să te bazezi pe ceva.

## Descărcare

Arhivele precompilate, după ce proprietarul publică lansarea ciornă, se găsesc pe [pagina Releases](https://github.com/chiuta/osatura/releases) a proiectului:

| Arhivă | Conținut |
|---|---|
| `osatura-windows-x86_64.zip` | `osatura.exe` (linia de comandă + serverul studioului), `osatura-studio.exe` (aplicația cu fereastră), biblioteca C și header-ul, licențe, notificări despre terți |
| `osatura-macos-arm64.tar.gz` | `osatura`, `osatura-studio` (doar Apple silicon), biblioteca C și header-ul, licențe, notificări |
| `osatura-linux-x86_64.tar.gz` | `osatura`, biblioteca C și header-ul, licențe, notificări (fără aplicație cu fereastră: folosește `osatura studio` și un browser) |
| `osatura-web.tar.gz` | `osatura_wasm.wasm`, `osatura.js`, demo `index.html` (neverificat în browsere, vezi [docs/WASM.md](docs/WASM.md)) |
| `SHA256SUMS.txt` | verifică fiecare descărcare față de el |

* **Windows:** fă dublu clic pe `osatura.exe` pentru a crea `Documents\OSATURA\first-rig.json` și a deschide studioul în browser, sau dublu clic pe `osatura-studio.exe` pentru o fereastră proprie. Ambele programe sunt nesemnate: alege *More info*, apoi *Run anyway* în SmartScreen. Vezi [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).
* **macOS:** aprobă programul în *System Settings > Privacy & Security* prima dată.
* **Sursa:** fiecare arhivă indică commit-ul din care a fost construită; părțile AGPL sunt servite cu oferta lor de sursă (`osatura --version`, `/source`).

## Funcții pe domenii

Detalii și opțiunile exacte: [docs/CLI_REFERENCE.md](docs/CLI_REFERENCE.md); ce este parțial sau lipsește: [docs/LIMITATIONS.md](docs/LIMITATIONS.md).

* **Modelul de desen.** Trasee vectoriale, umpleri, contururi (linii întrerupte, lățime variabilă), gradienți liniari și radiali, măști, moduri de amestec, efecte pe strat (estompare, umbră, nuanțare, umplere), straturi de imagine cu deformare pe plasă, metaballs lichide, operații booleene pe poligoane, particule deterministe, cameră multiplan cu cadre salvate, profunzime de câmp.
* **Rigging.** Oase cu cinematică directă; IK cu două oase, soft, stretch și FABRIK; limite, constrângeri, arcuri; skinning cu instrumente pentru ponderi; deformatoare lattice, spine, morph, wobble și cage; trasarea plasei din canalul alfa; pivoți de warp și lipire (glue); drivere (slidere, joystickuri, schimbarea vederii); seturi de piele (skin sets); auto-rig după numele straturilor; decuparea ochilor; containere de aranjare și regiuni de derulare.
* **Animație.** Chei cu easing Bézier/pas/arc/bounce, expresii și canale conduse, straturi comutator, preluări (takes) și o stivă de clipuri pe straturi, conversie în buclă și pe loc, blocarea piciorului, magneți, urmărirea unei căi, generatoare de repaus și de suprapunere, fizică de lanț/păr/pânză, sincronizare labială din audio sau din transcriere, mișcare a capului conștientă de vorbire, retargetare BVH, intrare live (MIDI, mouse, gamepad, repere faciale furnizate de tine), piste audio cu tăiere, fade și mixaj.
* **Runtime.** Mașini de stări cu arbori de amestec 1-D, legare de date, fizică de derulare, nivel de detaliu cu eroare declarată, un sandbox de scripting Rhai, un server pentru agenți (MCP) peste operații de tip patch.
* **Ieșire.** SVG, PNG, secvențe PNG și sprite sheet, GIF, APNG, MP4/WebM (prin ffmpeg), player HTML autonom, Lottie și dotLottie, glTF 2.0 binar, OpenEXR/PNG pe 16 biți, OCA; export cu țintă de dimensiune (`export-fit`). Import: SVG, OpenRaster, Lottie, BVH, secvențe PNG/GIF/APNG.
* **Instrumente de calitate.** `check`, `lint`, `diff` semantic, fuzzing de poze cu hărți termice, CI de animație cu diferență perceptuală și cadre de referință ([docs/CI.md](docs/CI.md)), control de versiuni cu îmbinare semantică (`vcs`), verificări de siguranță pentru străluciri și mișcare.
* **Studio.** Editor în browser servit de CLI: gizmo-uri pe canvas, cronologie, dope sheet și editor de grafice, suprapuneri, paletă de comenzi și scurtături, undo/redo, editare multiutilizator în timp real cu token opțional și jurnal persistent ([docs/STUDIO.md](docs/STUDIO.md)).
* **Biblioteci.** Fațadă Rust stabilă ([docs/API.md](docs/API.md)), ABI C ([docs/C_ABI.md](docs/C_ABI.md)), modul WebAssembly și învelitoare JavaScript (construit și testat sub Node în CI, încă nerulat într-un browser; [docs/WASM.md](docs/WASM.md)), randor wgpu opțional.

## Pornire rapidă

Cerințe: un lanț de unelte Rust stabil, recent (sau o arhivă precompilată din secțiunea [Descărcare](#descărcare): sari peste linia `cargo build` și folosește `osatura` despachetat). `ffmpeg` este necesar doar pentru exportul MP4/WebM și pentru audio mp3/ogg; nu este niciodată inclus în pachet. Fiecare comandă de mai jos a fost rulată pentru acest README (build de depanare, Linux). Build-urile de lansare (`cargo build --release`) sunt mult mai rapide la randare și se folosesc la fel, cu `target/release`.

```sh
cargo build -p osatura-cli                       # construiește target/debug/osatura
export PATH="$PWD/target/debug:$PATH"             # ca `osatura` să fie găsit

osatura --version                                # versiune, licență, oferta sursei, linkuri de sprijin
osatura demo demo.json                           # scrie demo-ul de rig încorporat (640x480, 48 de cadre)
osatura check demo.json                          # validează: ok: 'OSATURA demo' 640x480 @ 24 fps, 48 frames, 6 bones...
osatura lint demo.json                           # verificări statice de rig/animație: 0 finding(s), 0 error(s)
osatura render demo.json 12 frame12.png --bones  # un cadru, cu suprapunerea scheletului
osatura render demo.json 12 frame12.svg          # același cadru ca ieșire vectorială
osatura frames demo.json frames_out --png        # toate cadrele ca PNG
osatura export demo.json demo.gif                # și .html (player), .json (Lottie), .lottie, .mp4 (ffmpeg)
osatura demo-fx fx.json                          # al doilea demo: lichide, booleene, particule
osatura lipsync demo.json voice.wav mouth out.json   # forme ale gurii dintr-un fișier WAV (stratul comutator „mouth”)
osatura studio demo.json --port 8080             # editor în browser la http://127.0.0.1:8080
osatura ci init demo.json && osatura ci run     # CI de animație: cadre de referință + verificări (circa 40 s într-un build de depanare)
```

`voice.wav` este orice fișier WAV (mono sau stereo) pe care îl furnizezi tu. Studioul ascultă doar pe `127.0.0.1` și refuză cererile pentru alte nume de gazdă; înainte să îl expui altora, citește [docs/SECURITY.md](docs/SECURITY.md).

Folosirea bibliotecilor:

```sh
cargo build -p osatura-capi                      # target/debug/libosatura_capi.so și .a
gcc -std=c99 -o smoke osatura-capi/tests/smoke.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./smoke demo.json    # abi 1, 48 frames, 320x240, rc=0, checksum=...
```

Fațada Rust este `osatura_core::prelude` ([docs/API.md](docs/API.md)); mai multe exemple în C și Python sunt în [docs/C_ABI.md](docs/C_ABI.md).

Teste (circa 14 minute într-un build de depanare pe o mașină partajată; vezi [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)):

```sh
cargo test --workspace
cargo test --doc -p osatura-core
cargo test -p osatura-cli --test spdx_audit
```

## Structura depozitului

| Cale | Conținut | Licență |
|---|---|---|
| `osatura-core/` | motorul, formatele, instrumentele, fațada stabilă `api` | MPL-2.0 |
| `osatura-cli/` | binarul `osatura`, serverul studioului și al colaborării, teste end-to-end și de fum | AGPL-3.0-or-later |
| `osatura-gui/` | `osatura-studio`, studioul într-o fereastră nativă (Windows și macOS; stub pe Linux) | AGPL-3.0-or-later |
| `osatura-capi/` | ABI C (`osatura.h`, bibliotecă dinamică și statică) | MPL-2.0 |
| `osatura-wasm/` | ABI WebAssembly, învelitoare JavaScript, pagină demo (construit în CI) | MPL-2.0 |
| `osatura-gpu/` | randor wgpu opțional (nu este membru implicit al workspace-ului) | MPL-2.0 |
| `docs/` | documentație (CC-BY-4.0) și fișiere generate despre terți | CC-BY-4.0 |
| `.github/workflows/` | CI (`ci.yml`) și build-uri de lansare (`release.yml`) | MPL-2.0 |
| `scripts/` | `e2e.sh`, `licence-audit.sh` | MPL-2.0 |
| `LICENSES/`, `REUSE.toml`, `deny.toml` | texte de licență, metadate de licență, politica dependențelor | - |
| `ROADMAP.md`, `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `TRADEMARK.md`, `CONTRIBUTING.md`, `CHANGELOG.md` | documentele proiectului | CC-BY-4.0 |

## Harta documentației

Începe de la [docs/INDEX.md](docs/INDEX.md) (toate documentele, publicul țintă, trasee de lectură).

| Pagină | Pentru |
|---|---|
| [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md) | primul proiect, randare, export |
| [docs/CLI_REFERENCE.md](docs/CLI_REFERENCE.md) | fiecare comandă și opțiune |
| [docs/STUDIO.md](docs/STUDIO.md) | editorul din browser |
| [docs/RIGGING_GUIDE.md](docs/RIGGING_GUIDE.md) | construirea rig-urilor |
| [docs/ANIMATION_GUIDE.md](docs/ANIMATION_GUIDE.md) | animație, clipuri, fizică, audio |
| [docs/PROJECT_FORMAT.md](docs/PROJECT_FORMAT.md) | formatul JSON al proiectului |
| [docs/EXPORT_FORMATS.md](docs/EXPORT_FORMATS.md) | ce păstrează și ce pierde fiecare format |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | proiectare și module |
| [docs/GLOSSARY.md](docs/GLOSSARY.md) | termeni |
| [docs/FAQ.md](docs/FAQ.md) | întrebări frecvente |
| [docs/API.md](docs/API.md) | fațada Rust 1.4 |
| [docs/C_ABI.md](docs/C_ABI.md) | interfața C |
| [docs/WASM.md](docs/WASM.md) | WebAssembly (construit în CI, nerulat într-un browser) |
| [docs/CI.md](docs/CI.md) | CI de animație |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | construire, teste, convenții, lansare |
| [docs/SECURITY.md](docs/SECURITY.md) | model de amenințări și întărire |
| [docs/LIMITATIONS.md](docs/LIMITATIONS.md) | limite cunoscute |
| [docs/PATENT_NOTES.md](docs/PATENT_NOTES.md), [docs/THIRD_PARTY.md](docs/THIRD_PARTY.md), [docs/THIRD_PARTY_NOTICES.md](docs/THIRD_PARTY_NOTICES.md) | brevete, licențele dependențelor |
| [CHANGELOG.md](CHANGELOG.md), [ROADMAP.md](ROADMAP.md), [FEATURES.md](FEATURES.md) | istoric, plan și stare detaliată, matricea de funcții |
| [CLEANROOM.md](CLEANROOM.md), [LICENSING.md](LICENSING.md), [TRADEMARK.md](TRADEMARK.md), [CONTRIBUTING.md](CONTRIBUTING.md) | proveniență, licențe, politica numelui, reguli de contribuție |

## Rezumatul licențelor

* **MPL-2.0** pentru bibliotecile `osatura-core`, `osatura-capi`, `osatura-wasm`, `osatura-gpu` și pentru `scripts/`: le poți lega în propriul program, deschis sau închis; modificările aduse acelor fișiere trebuie publicate sub MPL.
* **AGPL-3.0-or-later** pentru `osatura-cli` (linia de comandă și studioul) și `osatura-gui` (fereastra din jurul lui). Dacă îl modifici și lași alte persoane să interacționeze cu el prin rețea, secțiunea 13 te obligă să le oferi sursa ta: `osatura studio --source-url URL` publică locul unde se află; `/source` și `osatura --version` conțin oferta.
* **CC-BY-4.0** pentru documentație și resurse sintetice (inclusiv modelul sintetic de detecție a fețelor).
* **Proiectele și randările tale rămân ale tale.**
* Numele și sigla nu sunt acoperite de aceste licențe ([TRADEMARK.md](TRADEMARK.md)). Situația brevetelor nu a fost evaluată ([docs/PATENT_NOTES.md](docs/PATENT_NOTES.md)). O licență comercială sau duală pentru codul contribuit nu este posibilă fără un acord de licență al contribuitorilor (CLA), iar un astfel de acord nu există.
* Crate-uri terțe: 363 în inventarul din 24 oct. 2026 (toate platformele și țintele împreună, ceea ce include crate-urile doar-Windows / macOS / Linux ale aplicației cu fereastră; un binar dat conține un subset), toate permisive, cu excepția a șase crate-uri MPL-2.0 cu copyleft slab: `smartstring`, în spatele funcției opționale `scripting`, și cinci (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`) pe care le poate aduce doar aplicația cu fereastră `osatura-gui` ([docs/THIRD_PARTY.md](docs/THIRD_PARTY.md)). ffmpeg nu este niciodată inclus în pachet.

Vezi [LICENSE](LICENSE), [LICENSES/](LICENSES/) și [LICENSING.md](LICENSING.md). Nimic din toate acestea nu este consultanță juridică; revizuirea de către un avocat este încă de făcut.

## Contribuții

Contribuțiile sunt binevenite, sub licența părții pe care o modifică. Citește [CONTRIBUTING.md](CONTRIBUTING.md) (fiecare commit are nevoie de o linie `Signed-off-by`, DCO 1.1), [CLEANROOM.md](CLEANROOM.md) (implementează doar din surse publice; nu copia cod proprietar sau cu licență incompatibilă) și [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) (convenții, listă de verificare pentru un modul nou, teste). Contribuții bune pentru început: o corectură dintre cele din [docs/LIMITATIONS.md](docs/LIMITATIONS.md), o rulare reală a listei de verificare WebAssembly, un test pe desktop real al executabilelor pentru Windows / macOS, un benchmark pe un GPU real.

Probleme de securitate: nu deschide un issue public; urmează [docs/SECURITY.md](docs/SECURITY.md) secțiunea 10.

## Sprijin

OSATURA este dezvoltat de Alexandru-Ionuț Chiuță (Alexio). Dacă îți este de folos, poți sprijini munca:

* Buy Me a Coffee: <https://buymeacoffee.com/echo.of.the.strings>
* Patreon: <https://patreon.com/c/alexio_tf>

Contactul responsabilului (afișat și de `osatura --version`): alexio@trom.tf.

## Mulțumiri

* Autorilor articolelor științifice și ai manualelor publice pe care le urmează algoritmii, numite pe module în [CLEANROOM.md](CLEANROOM.md) (matematica curbelor Bézier și a spline-urilor, cinematica inversă, skinning liniar, coordonate de valoare medie, potrivirea curbelor și altele).
* Specificațiilor deschise pe care OSATURA le citește și le scrie: SVG, OpenRaster, OCA, Lottie/dotLottie, glTF, APNG, WebM, OpenEXR, BVH, PNG, GIF.
* Crate-urilor Rust pe care se sprijină, listate cu licențele lor în [docs/THIRD_PARTY.md](docs/THIRD_PARTY.md), în special `tiny-skia`, `serde`, `png`, `zip`, `rhai` și `wgpu`.
* Tuturor celor care sprijină proiectul prin linkurile de mai sus.
