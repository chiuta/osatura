<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA

**Un motore di rigging e animazione vettoriale 2D in Rust: ossa, IK, skinning, deformatori, un runtime a macchina a stati, una toolbox da riga di comando, uno studio nel browser e librerie incorporabili, in un unico workspace.** Versione in rumeno: [README.ro.md](README.ro.md).

![Un personaggio di profilo (testa, orecchio, coda di cavallo, tronco, braccia con dita, gambe, piedi) cammina sul posto su un terreno che scorre; output reale di osatura export sul progetto dimostrativo incluso](docs/img/walk.gif)

*Cosa produce: la persona che cammina incorporata, esportata con `osatura demo-person` e `osatura export` (l'intero progetto è un unico file JSON). Provalo in due minuti: [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).*

## Sintesi e filosofia

OSATURA permette di costruire un personaggio o una scena a partire da forme vettoriali e immagini, di dotarlo di rig con ossa e vincoli, di animarlo con chiavi, clip e fisica e di esportarlo come GIF, video, Lottie, glTF, un player HTML autonomo o un runtime interattivo. È composto da piccoli elementi deterministici: lo stesso progetto e lo stesso seed producono gli stessi pixel, ogni strumento legge e scrive un progetto JSON documentato e ogni comando funziona da terminale, quindi l'animazione può essere scritta in script, rivista, confrontata con diff e testata come il codice sorgente.

Principi che danno forma al progetto:

* **Implementazione indipendente.** Scritta a partire da specifiche pubbliche, articoli scientifici e matematica da manuale. Gli elenchi di funzionalità dei concorrenti sono stati consultati per definire l'*ambito*; nessun codice sorgente di terzi è stato copiato e il progetto non legge né scrive alcun formato proprietario. Non è un processo clean-room rigoroso (vedi [CLEANROOM.md](docs/i18n/it/CLEANROOM.md), compresa la sua dichiarazione sulla ricerca) e non è affiliato a Moho né ad altri prodotti commerciali, non ne deriva e non è compatibile con essi. Il codice è stato scritto in gran parte con un modello di IA.
* **Stato dichiarato con onestà.** Ogni funzionalità è descritta insieme a ciò che *non* fa ([docs/LIMITATIONS.md](docs/i18n/it/LIMITATIONS.md), [ROADMAP.md](docs/i18n/it/ROADMAP.md) sezione 9).
* **Sicuro per impostazione predefinita.** Nessun panic sugli input, controlli sui valori finiti, limiti alle risorse, scritture atomiche, percorsi degli asset confinati nella cartella del progetto ([docs/SECURITY.md](docs/i18n/it/SECURITY.md)).
* **Prima i formati aperti.** OpenRaster, OCA, SVG, Lottie, glTF, APNG, WebM, EXR; nessun formato di file proprietario, per scelta di progetto.
* **Poche dipendenze**, con una politica sulle licenze verificata da test e da `cargo deny`.

## Stato

*Nome: il progetto si chiamava ARMATURA fino al 24 ott 2026 ed è stato rinominato OSATURA dopo una ricerca di marchio ([docs/LIMITATIONS.md](docs/i18n/it/LIMITATIONS.md) L-78). Sono stati rinominati anche il repository e l'indirizzo del sito (i vecchi link al repository reindirizzano).*

> **Alpha. Prime build preparate come pre-release in bozza: `v0.1.0-alpha.3`.** Gli eseguibili per Windows x86-64, macOS arm64 e Linux x86-64 e il modulo WebAssembly sono compilati da GitHub Actions e allegati a una pre-release *in bozza* su <https://github.com/chiuta/osatura/releases> (una bozza è visibile solo al proprietario finché non viene pubblicata a mano). Tutti i crate sono ancora alla versione `0.1.0`; "v0.5 / v0.6 / v0.7" sono *livelli di funzionalità* nella cronologia, non numeri di release ([CHANGELOG.md](docs/i18n/it/CHANGELOG.md)).
>
> | Verificato | Non verificato |
> |---|---|
> | **In locale (Linux x86-64, 24 ott 2026):** 1338 test di `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` più 19 doc-test superati (il crate GPU richiede un adattatore e il crate della finestra un desktop; prima il conteggio che includeva il crate GPU era 1368). **Su GitHub Actions (24 ott 2026, toolchain 1.97.0):** la suite di test e `clippy -D warnings` passano su Linux, macOS (arm64) e Windows; `cargo deny` e `reuse lint` passano; il modulo WebAssembly viene compilato per `wasm32-unknown-unknown` e il test del wrapper passa sul modulo reale con Node 20; i binari di release superano uno smoke test (`--version`, `demo`, `check`, `render`) e `osatura-studio --selftest` viene eseguito su Windows e macOS. Quattro smoke test dello studio in Chromium headless superati; la CLI, la facciata Rust e la C ABI (anche con ASan/UBSan) sono state eseguite con gli esempi della documentazione | **Nessuna persona ha usato gli eseguibili su un vero desktop Windows o macOS**: il launcher a doppio clic di Windows e l'app a finestra `osatura-studio` (WebView2 / WKWebView) sono stati solo compilati e auto-testati in CI; i binari **non sono firmati** (SmartScreen "Editore sconosciuto", Gatekeeper di macOS); il binario Linux è stato compilato solo su un runner Ubuntu; **il modulo WebAssembly non è mai stato eseguito in un browser** e velocità e memoria non sono state misurate; lo studio non è mai stato usato in produzione da una persona né in un browser diverso da Chromium; non è mai stata usata una vera GPU (solo un adattatore software), né una webcam, una fotografia o un file di motion capture reali; nessun audit di sicurezza, revisione legale o ricerca di brevetti |
>
> Leggi [docs/LIMITATIONS.md](docs/i18n/it/LIMITATIONS.md) (78 limiti numerati, con gravità e soluzione alternativa) prima di fare affidamento su qualsiasi cosa.

## Download

Gli archivi precompilati, una volta che il proprietario avrà pubblicato la release in bozza, si trovano nella [pagina Releases](https://github.com/chiuta/osatura/releases) del progetto:

| Archivio | Contenuto |
|---|---|
| `osatura-windows-x86_64.zip` | `osatura.exe` (riga di comando + server dello studio), `osatura-studio.exe` (app a finestra), libreria C e header, licenze, avvisi di terze parti |
| `osatura-macos-arm64.tar.gz` | `osatura`, `osatura-studio` (solo Apple silicon), libreria C e header, licenze, avvisi |
| `osatura-linux-x86_64.tar.gz` | `osatura`, libreria C e header, licenze, avvisi (nessuna app a finestra: usa `osatura studio` e un browser) |
| `osatura-web.tar.gz` | `osatura_wasm.wasm`, `osatura.js`, `index.html` demo (non verificato nei browser, vedi [docs/WASM.md](docs/i18n/it/WASM.md)) |
| `SHA256SUMS.txt` | verifica ogni download con questo file |

* **Windows:** fai doppio clic su `osatura.exe` per creare `Documents\OSATURA\first-rig.json` e aprire lo studio nel browser, oppure doppio clic su `osatura-studio.exe` per una finestra tutta sua. Entrambi i programmi non sono firmati: in SmartScreen scegli *Ulteriori informazioni* e poi *Esegui comunque*. Vedi [docs/GETTING_STARTED.md](docs/i18n/it/GETTING_STARTED.md).
* **macOS:** la prima volta approva il programma in *Impostazioni di Sistema > Privacy e sicurezza*.
* **Sorgenti:** ogni archivio indica il commit da cui è stato compilato; le parti AGPL sono servite con la relativa offerta dei sorgenti (`osatura --version`, `/source`).

## Funzionalità per area

Dettagli e opzioni esatte: [docs/CLI_REFERENCE.md](docs/i18n/it/CLI_REFERENCE.md); ciò che è parziale o manca: [docs/LIMITATIONS.md](docs/i18n/it/LIMITATIONS.md).

* **Modello di disegno.** Tracciati vettoriali, riempimenti, contorni (tratteggi, larghezza variabile), gradienti lineari e radiali, maschere, modalità di fusione, effetti per livello (sfocatura, ombra, tinta, riempimento), livelli immagine con mesh warp, metaball liquide, operazioni booleane tra poligoni, particelle deterministiche, camera multipiano con inquadrature salvate, profondità di campo.
* **Rigging.** Ossa con cinematica diretta; IK a due ossa, soft, stretch e FABRIK; limiti, vincoli, molle; skinning con strumenti per i pesi; deformatori lattice, spine, morph, wobble e cage; mesh tracing dall'alfa; pivot e glue del warp; driver (slider, joystick, scambio di vista); set di skin; auto-rig dai nomi dei livelli; clipping degli occhi; contenitori di layout e regioni di scorrimento.
* **Animazione.** Chiavi con easing Bézier/step/spring/bounce, espressioni e canali pilotati, livelli switch, take e uno stack di clip a livelli, conversione in loop e in-place, foot lock, magneti, inseguimento di tracciato, generatori idle e overlap, fisica di catene/capelli/stoffa, lip-sync da audio o da trascrizione, movimento della testa legato al parlato, retargeting BVH, input dal vivo (MIDI, mouse, gamepad, landmark da webcam forniti da te), tracce audio con trim, dissolvenza e missaggio.
* **Runtime.** Macchine a stati con blend tree monodimensionali, data binding, fisica dello scorrimento, livello di dettaglio con limite d'errore, una sandbox di scripting Rhai, un server agente (MCP) basato su operazioni di patch.
* **Output.** SVG, PNG, sequenze PNG e sprite sheet, GIF, APNG, MP4/WebM (tramite ffmpeg), player HTML autonomo, Lottie e dotLottie, glTF 2.0 binario, OpenEXR/PNG a 16 bit, OCA; esportazione con dimensione obiettivo (`export-fit`). Importazione: SVG, OpenRaster, Lottie, BVH, sequenze PNG/GIF/APNG.
* **Strumenti per la qualità.** `check`, `lint`, `diff` semantico, pose fuzzing con heat map, CI dell'animazione con diff percettivo e fotogrammi di riferimento ([docs/CI.md](docs/i18n/it/CI.md)), controllo di versione con merge semantico (`vcs`), controlli di sicurezza per lampeggi e movimento.
* **Studio.** Editor nel browser servito dalla CLI: gizmo sul canvas, timeline, dope sheet ed editor di curve, overlay, palette dei comandi e scorciatoie, annulla/ripeti, modifica collaborativa in tempo reale con token opzionale e log persistente ([docs/STUDIO.md](docs/i18n/it/STUDIO.md)).
* **Librerie.** Facciata Rust stabile ([docs/API.md](docs/i18n/it/API.md)), C ABI ([docs/C_ABI.md](docs/i18n/it/C_ABI.md)), modulo WebAssembly e wrapper JavaScript (compilati e testati con Node in CI, non ancora eseguiti in un browser; [docs/WASM.md](docs/i18n/it/WASM.md)), renderer wgpu opzionale.

## Avvio rapido

Requisiti: una toolchain Rust stabile recente (oppure un archivio precompilato dalla sezione [Download](#download): salta la riga `cargo build` e usa l'`osatura` estratto). `ffmpeg` serve solo per l'esportazione MP4/WebM e per l'audio mp3/ogg; non viene mai incluso. Ogni comando qui sotto è stato eseguito per questo README (build di debug, Linux). Le build di release (`cargo build --release`) sono molto più veloci per il rendering e si usano allo stesso modo con `target/release`.

```sh
cargo build -p osatura-cli                       # builds target/debug/osatura
export PATH="$PWD/target/debug:$PATH"             # so that `osatura` is found

osatura --version                                # version, licence, source offer, support links
osatura demo demo.json                           # write the built-in rig demo (640x480, 48 frames)
osatura check demo.json                          # validate: ok: 'OSATURA demo' 640x480 @ 24 fps, 48 frames, 6 bones...
osatura lint demo.json                           # static rig/animation checks: 0 finding(s), 0 error(s)
osatura render demo.json 12 frame12.png --bones  # one frame, with the skeleton overlay
osatura render demo.json 12 frame12.svg          # the same frame as vector output
osatura frames demo.json frames_out --png        # every frame as a PNG
osatura export demo.json demo.gif                # also .html (player), .json (Lottie), .lottie, .mp4 (ffmpeg)
osatura demo-fx fx.json                          # second demo: liquid, booleans, particles
osatura lipsync demo.json voice.wav mouth out.json   # mouth shapes from a WAV file (switch layer "mouth")
osatura studio demo.json --port 8080             # browser editor at http://127.0.0.1:8080
osatura ci init demo.json && osatura ci run     # animation CI: goldens + checks (about 40 s in a debug build)
```

`voice.wav` è un qualsiasi file WAV mono o stereo che fornisci tu. Lo studio ascolta solo su `127.0.0.1` e rifiuta le richieste per altri nomi host; per condividerlo, leggi prima [docs/SECURITY.md](docs/i18n/it/SECURITY.md).

Usare le librerie:

```sh
cargo build -p osatura-capi                      # target/debug/libosatura_capi.so and .a
gcc -std=c99 -o smoke osatura-capi/tests/smoke.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./smoke demo.json    # abi 1, 48 frames, 320x240, rc=0, checksum=...
```

La facciata Rust è `osatura_core::prelude` ([docs/API.md](docs/i18n/it/API.md)); altri esempi in C e Python si trovano in [docs/C_ABI.md](docs/i18n/it/C_ABI.md).

Test (circa 14 minuti in una build di debug su una macchina condivisa; vedi [docs/DEVELOPMENT.md](docs/i18n/it/DEVELOPMENT.md)):

```sh
cargo test --workspace
cargo test --doc -p osatura-core
cargo test -p osatura-cli --test spdx_audit
```

## Struttura del repository

| Percorso | Contenuto | Licenza |
|---|---|---|
| `osatura-core/` | motore, formati, strumenti, facciata stabile `api` | MPL-2.0 |
| `osatura-cli/` | il binario `osatura`, studio e server di collaborazione, test end-to-end e smoke test | AGPL-3.0-or-later |
| `osatura-gui/` | `osatura-studio`, lo studio in una finestra nativa (Windows e macOS; uno stub su Linux) | AGPL-3.0-or-later |
| `osatura-capi/` | C ABI (`osatura.h`, libreria condivisa e statica) | MPL-2.0 |
| `osatura-wasm/` | ABI WebAssembly, wrapper JavaScript, pagina demo (compilata in CI) | MPL-2.0 |
| `osatura-gpu/` | renderer wgpu opzionale (non è un membro predefinito) | MPL-2.0 |
| `docs/` | documentazione (CC-BY-4.0) e file di terze parti generati | CC-BY-4.0 |
| `.github/workflows/` | CI (`ci.yml`) e build di release (`release.yml`) | MPL-2.0 |
| `scripts/` | `e2e.sh`, `licence-audit.sh` | MPL-2.0 |
| `LICENSES/`, `REUSE.toml`, `deny.toml` | testi delle licenze, metadati di licenza, politica sulle dipendenze | - |
| `ROADMAP.md`, `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `TRADEMARK.md`, `CONTRIBUTING.md`, `CHANGELOG.md` | documenti di progetto | CC-BY-4.0 |

## Mappa della documentazione

Parti da [docs/INDEX.md](docs/i18n/it/INDEX.md) (tutti i documenti, i destinatari, i percorsi di lettura).

| Pagina | Per |
|---|---|
| [docs/GETTING_STARTED.md](docs/i18n/it/GETTING_STARTED.md) | primo progetto, rendering, esportazione |
| [docs/CLI_REFERENCE.md](docs/i18n/it/CLI_REFERENCE.md) | ogni comando e opzione |
| [docs/STUDIO.md](docs/i18n/it/STUDIO.md) | l'editor nel browser |
| [docs/RIGGING_GUIDE.md](docs/i18n/it/RIGGING_GUIDE.md) | costruire rig |
| [docs/ANIMATION_GUIDE.md](docs/i18n/it/ANIMATION_GUIDE.md) | animare, clip, fisica, audio |
| [docs/PROJECT_FORMAT.md](docs/i18n/it/PROJECT_FORMAT.md) | il formato di progetto JSON |
| [docs/EXPORT_FORMATS.md](docs/i18n/it/EXPORT_FORMATS.md) | cosa trasporta e cosa perde ciascun formato |
| [docs/ARCHITECTURE.md](docs/i18n/it/ARCHITECTURE.md) | progettazione e moduli |
| [docs/GLOSSARY.md](docs/i18n/it/GLOSSARY.md) | termini |
| [docs/FAQ.md](docs/i18n/it/FAQ.md) | domande frequenti |
| [docs/API.md](docs/i18n/it/API.md) | facciata Rust 1.4 |
| [docs/C_ABI.md](docs/i18n/it/C_ABI.md) | interfaccia C |
| [docs/WASM.md](docs/i18n/it/WASM.md) | WebAssembly (compilato in CI, non eseguito in un browser) |
| [docs/CI.md](docs/i18n/it/CI.md) | CI dell'animazione |
| [docs/DEVELOPMENT.md](docs/i18n/it/DEVELOPMENT.md) | build, test, convenzioni, release |
| [docs/SECURITY.md](docs/i18n/it/SECURITY.md) | modello delle minacce e hardening |
| [docs/LIMITATIONS.md](docs/i18n/it/LIMITATIONS.md) | limiti noti |
| [docs/PATENT_NOTES.md](docs/i18n/it/PATENT_NOTES.md), [docs/THIRD_PARTY.md](docs/i18n/it/THIRD_PARTY.md), [docs/THIRD_PARTY_NOTICES.md](docs/THIRD_PARTY_NOTICES.md) | brevetti, licenze delle dipendenze |
| [CHANGELOG.md](docs/i18n/it/CHANGELOG.md), [ROADMAP.md](docs/i18n/it/ROADMAP.md), [FEATURES.md](docs/i18n/it/FEATURES.md) | cronologia, piano e stato dettagliato, matrice delle funzionalità |
| [CLEANROOM.md](docs/i18n/it/CLEANROOM.md), [LICENSING.md](docs/i18n/it/LICENSING.md), [TRADEMARK.md](docs/i18n/it/TRADEMARK.md), [CONTRIBUTING.md](docs/i18n/it/CONTRIBUTING.md) | provenienza, licenze, politica sul nome, regole per i contributi |

## Riepilogo delle licenze

* **MPL-2.0** per le librerie `osatura-core`, `osatura-capi`, `osatura-wasm`, `osatura-gpu` e per `scripts/`: puoi collegarle al tuo programma, aperto o chiuso; le modifiche a quei file devono essere pubblicate sotto MPL.
* **AGPL-3.0-or-later** per `osatura-cli` (la riga di comando e lo studio) e `osatura-gui` (la finestra che lo contiene). Se lo modifichi e permetti ad altri di interagire con esso tramite una rete, la sezione 13 ti impone di offrire loro il tuo codice sorgente: `osatura studio --source-url URL` pubblica dove si trova; `/source` e `osatura --version` riportano l'offerta.
* **CC-BY-4.0** per la documentazione e gli asset sintetici (compreso il modello sintetico di rilevamento dei volti).
* **I tuoi progetti e i tuoi render restano tuoi.**
* Il nome e il logo non sono coperti da queste licenze ([TRADEMARK.md](docs/i18n/it/TRADEMARK.md)). Lo stato dei brevetti non è stato valutato ([docs/PATENT_NOTES.md](docs/i18n/it/PATENT_NOTES.md)). Una licenza commerciale o duale per il codice contribuito non è possibile senza un contributor licence agreement, e non ne esiste alcuno.
* Crate di terze parti: 363 nell'inventario del 24 ott 2026 (tutte le piattaforme e i target insieme, inclusi i crate della sola app a finestra per Windows / macOS / Linux; un dato binario ne contiene un sottoinsieme), tutti permissivi tranne sei crate MPL-2.0 a copyleft debole: `smartstring` dietro la funzionalità opzionale `scripting`, e cinque (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`) che solo l'app a finestra `osatura-gui` può includere ([docs/THIRD_PARTY.md](docs/i18n/it/THIRD_PARTY.md)). ffmpeg non viene mai incluso.

Vedi [LICENSE](LICENSE), [LICENSES/](LICENSES/) e [LICENSING.md](docs/i18n/it/LICENSING.md). Nulla di tutto ciò è consulenza legale; la revisione di un avvocato è ancora in sospeso.

## Contribuire

I contributi sono benvenuti, con la licenza della parte che modificano. Leggi [CONTRIBUTING.md](docs/i18n/it/CONTRIBUTING.md) (ogni commit richiede una riga `Signed-off-by`, DCO 1.1), [CLEANROOM.md](docs/i18n/it/CLEANROOM.md) (implementa solo da fonti pubbliche; non copiare da codice proprietario o con licenza incompatibile) e [docs/DEVELOPMENT.md](docs/i18n/it/DEVELOPMENT.md) (convenzioni, checklist per un nuovo modulo, test). Buoni primi contributi: una correzione tratta da [docs/LIMITATIONS.md](docs/i18n/it/LIMITATIONS.md), un'esecuzione reale della checklist WebAssembly, un test reale su desktop degli eseguibili Windows / macOS, un benchmark su GPU reale.

Problemi di sicurezza: non aprire una issue pubblica; segui [docs/SECURITY.md](docs/i18n/it/SECURITY.md) sezione 10.

## Supporto

OSATURA è sviluppato da Alexandru-Ionuț Chiuță (Alexio). Se ti è utile, puoi sostenere il lavoro:

* Buy Me a Coffee: <https://buymeacoffee.com/echo.of.the.strings>
* Patreon: <https://patreon.com/c/alexio_tf>

Contatto del maintainer (stampato anche da `osatura --version`): alexio@trom.tf.

## Ringraziamenti

* Gli autori degli articoli pubblici e dei manuali che gli algoritmi seguono, citati per modulo in [CLEANROOM.md](docs/i18n/it/CLEANROOM.md) (matematica di Bézier e spline, cinematica inversa, linear-blend skinning, mean-value coordinates, curve fitting e altro).
* Le specifiche aperte che OSATURA legge e scrive: SVG, OpenRaster, OCA, Lottie/dotLottie, glTF, APNG, WebM, OpenEXR, BVH, PNG, GIF.
* I crate Rust su cui si basa, elencati con le relative licenze in [docs/THIRD_PARTY.md](docs/i18n/it/THIRD_PARTY.md), in particolare `tiny-skia`, `serde`, `png`, `zip`, `rhai` e `wgpu`.
* Tutti coloro che sostengono il progetto tramite i link qui sopra.
