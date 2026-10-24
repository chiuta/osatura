# API Rust di OSATURA e politica di stabilità

*Versione della facciata 1.4.0 (`osatura_core::api::API_VERSION`). Destinatari: sviluppatori Rust che incorporano il motore e maintainer che modificano la facciata. Licenza della documentazione CC-BY-4.0; esempi di codice MPL-2.0.*

Questo è il riferimento per l'**API Rust supportata** di `osatura-core`: il modulo facciata `osatura_core::api` e il suo import in una riga `osatura_core::prelude`. Elenca ogni elemento pubblico, le regole che mantengono stabile l'API ed esempi testati. Ogni esempio di questo file viene compilato ed eseguito sui sorgenti correnti prima della pubblicazione del file (vedi [Come vengono verificati gli esempi](#come-vengono-verificati-gli-esempi)).

Indice: [Cosa è coperto](#cosa-è-coperto) - [Come ottenerlo](#come-ottenere-il-crate) - [Politica di stabilità](#politica-di-stabilità) - [Regole semver](#regole-semver) - [Feature flag](#feature-flag) - [Thread-safety](#thread-safety) - [Determinismo](#determinismo) - [Limiti](#limiti-e-intervalli-degli-argomenti) - [Errori](#errori) - [Riferimento: `Animation`](#riferimento-animation) - [Riferimento: `Player`](#riferimento-player) - [Riferimento: funzioni libere](#riferimento-funzioni-libere-strumenti-controllo-di-versione-ci) - [Riferimento: tipi](#riferimento-tipi) - [Esempi](#esempi) - [Altri linguaggi](#altri-linguaggi) - [Appendice: JSON macchina e JSON dei binding](#appendice-json-delle-macchine-e-json-dei-binding)

## Cosa è coperto

Solo `osatura_core::api` e `osatura_core::prelude` costituiscono l'API Rust pubblica supportata. La facciata è composta da

* `Animation`: un progetto validato più la sua cronologia di annullamento (caricamento, interrogazione, rendering, esportazione, modifica, strumenti);
* `Player`: una macchina a stati in esecuzione con layout, scorrimento e livello di dettaglio (1.1, 1.2);
* funzioni libere per il controllo di versione e la CI delle animazioni (1.2 lettura, 1.3 scrittura) e per la vettorizzazione di bitmap;
* semplici tipi valore (`AnimationInfo`, `RenderSettings`, `Image`, `LodLevel`, `SizeFit`, `MocapReport`, `FuzzSummary`, `VcsCommit`, `VcsConflict`, `CiOutcome`, `CiInit`, ...) e gli enum `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome`, `VcsChoice`;
* il tipo di errore `ApiError` (implementa `std::error::Error` e `Display`).

L'ABI C (`osatura-capi`, header `osatura.h`) e il modulo WebAssembly (`osatura-wasm`) sono scritti esclusivamente su questa facciata e hanno versioni proprie: [C_ABI.md](C_ABI.md), [WASM.md](WASM.md).

### Cosa NON è coperto

Ogni altro modulo di `osatura-core` (`engine`, `project`, `edit`, `svg`, `raster`, `physics`, `vcs`, `ci`, ...) e i re-export alla radice (`Engine`, `Project`, ...). Sono `pub` e `#[doc(hidden)]` solo perché lo strumento da riga di comando, lo studio, i test dell'ABI C e i test unitari li usano; possono cambiare o sparire in qualsiasi release, e anche i loro testi `Err(String)` non sono un contratto. Se ti serve qualcosa raggiungibile solo da lì, chiedi l'aggiunta di un elemento alla facciata (una release minor) invece di dipendere dagli interni.

Il formato JSON del progetto ha un proprio campo di versione `format` (attualmente 1); un progetto scritto da una build più recente viene rifiutato con `InvalidJson`. Vedi [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Come ottenere il crate

OSATURA non è pubblicato su un registry; dipendi da esso tramite percorso (o git) dal tuo workspace:

```toml
[dependencies]
osatura-core = { path = "../osatura/osatura-core", default-features = false }
```

`default-features = false` esclude il motore di scripting Rhai (vedi [Feature flag](#feature-flag)); la facciata non ne ha bisogno. Poi `use osatura_core::prelude::*;`. La versione del crate (`0.1.0`) e la versione della facciata (`API_VERSION`) sono indipendenti: la facciata segue semver, la versione del crate per ora non promette nulla. Non è dichiarata alcuna versione minima supportata di Rust; il workspace viene sviluppato e testato con Rust 1.97 (edition 2021).

## Politica di stabilità

1. **Il file della superficie.** `osatura-core/src/api_surface.txt` elenca ogni elemento pubblico di `src/api/` (tipi, campi, varianti, costanti e firme dei metodi, uno per riga). Il test unitario `api::tests::api_surface_snapshot` ricostruisce quell'elenco da `api/` e fallisce se differisce dal file versionato. Ogni modifica della superficie è quindi una modifica deliberata e rivista di entrambi i file.
2. **Il test del prelude.** `osatura-core/tests/prelude_surface.rs` viene compilato come crate *esterno*. Importa i nomi della facciata soltanto tramite `osatura_core::prelude` e chiama un sottoinsieme rappresentativo (caricamento, rendering, modifica, esportazione, `Player`, gli strumenti, i wrapper di controllo di versione e CI, gestione dell'input errato). Se un nome che usa smette di essere riesportato dal prelude, o una firma cambia in un modo che quelle chiamate rilevano, quel test smette di compilare. Verifica inoltre che `API_VERSION == "1.4.0"`, quindi anche un aumento di versione è una modifica consapevole in quel file. Non enumera ogni nome del prelude, perciò l'aggiunta di un nome all'elenco dei re-export resta un passaggio manuale.
3. **Limiti del controllo.** Lo snapshot è una scansione testuale delle firme su singola riga in `api/`, non un diff a livello di compilatore come `cargo-public-api`: una modifica semantica che conserva il testo della firma (per esempio un errore diverso per lo stesso input) non viene rilevata, e una firma su più righe passerebbe inosservata (per questo il file mantiene le firme su una riga). L'elenco dei re-export del prelude (`lib.rs`) è verificato dal test esterno, non dallo snapshot.
4. **Nascosto non significa supportato.** I moduli `#[doc(hidden)] pub` esistono per i consumatori interni al workspace. Restano fuori dalla promessa anche se per te compilano.
5. **Come modificare la facciata** (maintainer; anche in [DEVELOPMENT.md](DEVELOPMENT.md)):
   * modifica `api/` (firme su una riga, commento di documentazione, un doc-test per le nuove funzioni libere, test unitari);
   * aggiungi i nuovi nomi pubblici all'elenco dei re-export del `prelude` in `lib.rs` e a `tests/prelude_surface.rs`;
   * incrementa `API_VERSION` (e `API_VERSION_MAJOR` se la modifica è incompatibile) secondo le regole qui sotto, e la versione verificata in `prelude_surface.rs`;
   * rigenera deliberatamente il file della superficie: `OSATURA_BLESS=1 cargo test -p osatura-core api::` e rivedi il diff di `api_surface.txt`;
   * aggiorna questo file, e `osatura-capi` / `osatura-wasm` solo se il nuovo elemento deve attraversare quei confini (la loro versione ABI si muove separatamente, le aggiunte la lasciano invariata).

## Regole semver

`API_VERSION` è una versione semantica della sola facciata.

| Modifica | Release |
|---|---|
| Correzione di bug, nessuna modifica di firma, nessun nuovo contratto di comportamento | patch (1.4.x) |
| Nuova funzione, metodo, tipo, variante di enum, campo di struct, costante | minor (1.x.0) |
| Rimozione o ridenominazione di qualsiasi cosa; modifica di una firma, di un intervallo di argomenti documentato in modo da rifiutare input prima validi, o del significato di un risultato documentato | major (2.0.0) |

Conseguenze per chi chiama:

* **Gli enum e le struct pubblici della facciata sono `#[non_exhaustive]`.** Aggiungi un ramo jolly quando fai `match` su `ApiError`, `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome` o `VcsChoice`; una nuova variante è una release minor. Non puoi costruire le struct con un letterale fuori dal crate (e `Image`, `SizeFit` e le struct di report sono risultati che puoi solo leggere). Le impostazioni si costruiscono con i costruttori: `RenderSettings::new(scale).with_bones(true)`, `MocapSettings::new().with_smooth(2)`.
* **Le stringhe all'interno dei risultati sono dati, non contratto.** `VcsConflict::kind` oggi è uno tra `"BothChanged"`, `"DeleteEdit"`, `"KeyRetimed"`, `"OrderConflict"`, `"ArrayConflict"` e *può acquisire nuovi valori nelle release minor*. I report JSON (`FuzzSummary::json`, `CiOutcome::json`) portano i propri nomi di schema (`osatura-fuzz-report` v1, `osatura-ci/1`). I testi dei messaggi di errore sono destinati alle persone.
* **Stabilità del comportamento.** Il rendering e la valutazione sono deterministici (vedi sotto), ma l'output in pixel può cambiare in una release minor quando si corregge un bug del rasterizzatore o di un effetto; i golden registrati dalla CI delle animazioni ne sono un promemoria. Non calcolare l'hash dei pixel renderizzati tra versioni diverse di OSATURA o tra macchine diverse senza una tolleranza.

Cronologia delle versioni della facciata: 1.0 (caricamento, interrogazione, rendering, esportazione, modifica, annullamento) - 1.1 (`Player`, macchine a stati, data binding) - 1.2 (layout, scorrimento e livello di dettaglio su `Player`; `lod_levels`, `fit_to_size`, `retarget_bvh`, `fuzz`, `from_image`, `trace_image_svg`, `vcs_log`, `ci_run`) - 1.3 (operazioni di scrittura per controllo di versione e CI, `DirtyTree` / `MergeInProgress` / `Conflicts`, `CiOutcome::updated` / `blessed`) - 1.4 (creazione di livelli e ossi, `bake_physics`, esportazione APNG ed EXR, `vcs_branches` / `vcs_branch_delete` / `vcs_status` / `vcs_diff` / `vcs_show` / `vcs_gc` / `vcs_fsck`).

## Feature flag

`osatura-core` ha una sola feature Cargo:

| Feature | Predefinita | Effetto |
|---|---|---|
| `scripting` | attiva | Compila il modulo di scripting in sandbox basato su Rhai (`script.rs`), usato soltanto dal comando `script` da riga di comando. Include `rhai` e, tramite esso, `smartstring` (MPL-2.0+, l'unico crate di terze parti con copyleft debole nell'albero). |

La facciata **non** ha alcuna funzionalità dietro una feature: ogni elemento qui documentato esiste con `default-features = false`. `osatura-capi`, `osatura-wasm` e `osatura-gpu` dipendono dal core senza la feature, quindi non contengono crate MPL di terze parti; `osatura-cli --no-default-features` fa lo stesso e poi segnala il comando `script` come non disponibile. I back end di rendering non sono feature: la facciata usa sempre il rasterizzatore CPU; il renderer GPU opzionale è un crate separato (`osatura-gpu`) e una feature della CLI (`gpu`) e non è raggiungibile tramite questa API.

## Thread-safety

* `Animation`, `Player`, `ApiError` e `Image` sono `Send + Sync` (verificato in compilazione per questo documento). La facciata non scrive alcuno stato globale; legge soltanto due impostazioni di asset a livello di processo (la directory predefinita degli asset e l'interruttore degli asset esterni) che nulla nella facciata modifica.
* I metodi che prendono `&self` (`info`, `bone_world`, `render_*`, `export`, `lod_levels`, `fit_to_size`, `fuzz`, `player`, `to_json`, ...) possono essere eseguiti in concorrenza su più thread sulla stessa `Animation`; i metodi che prendono `&mut self` (`set_key`, `delete_key`, `undo`, `redo`, `retarget_bvh`, ogni input di `Player` e `advance`) richiedono accesso esclusivo, come di consueto in Rust. L'esempio `render_in_parallel` più sotto renderizza ogni fotogramma sul proprio thread e verifica che le immagini siano uguali a quelle a thread singolo.
* Non c'è alcun locking interno né alcun thread in background. Un `Player` lavora su una propria copia del progetto, quindi modificare l'`Animation` in seguito non influisce mai su un player in esecuzione.
* `Animation::from_json` risolve i file immagine relativi rispetto alla **directory corrente** del processo e rifiuta i percorsi assoluti e `..` (vedi [SECURITY.md](SECURITY.md)); incorpora le immagini come data URI, oppure imposta la directory di lavoro prima del caricamento, se un progetto fa riferimento a file.
* Le funzioni di controllo di versione e CI lavorano su file: non sono sicure contro un altro processo che modifichi lo stesso progetto nello stesso momento (la facciata controlla in anticipo la presenza di un albero sporco e di un merge in sospeso, ma non esiste alcun lock).

## Determinismo

Stesso progetto, stessi argomenti, stessa build di OSATURA e stessa macchina: stessi byte. Questo vale per `render_rgba` / `render_png` / `render_svg`, `bone_world`, ogni esportazione, `lod_levels` e `fuzz` (per un dato seed), `fit_to_size` e per un `Player` (gli stessi input e le stesse chiamate a `advance` danno la stessa posa; `advance` suddivide il tempo in pezzi di al massimo 1/60 s, quindi il modo in cui spezzi il tempo conta solo al di sotto di quella risoluzione). Le scelte casuali all'interno del motore (rumore, particelle, fuzzing, cluster di vettorizzazione) usano generatori con seed, mai l'orologio. Ciò che *non* è promesso: pixel di bordo anti-aliasati identici tra sistemi operativi, CPU, compilatori o versioni di `libm` diversi, tra target a 32 e 64 bit, o tra versioni di OSATURA. I golden esatti al byte sono stati verificati solo tra processi su una macchina Linux x86-64 (vedi [LIMITATIONS.md](LIMITATIONS.md)); usa tolleranze (la CI delle animazioni lo fa) per qualsiasi confronto tra macchine diverse.

## Limiti e intervalli degli argomenti

| Elemento | Regola |
|---|---|
| `MAX_JSON_BYTES` | 256 MiB; un testo più lungo dà `InvalidJson` |
| `MAX_RENDER_SIDE` | 16384 pixel di output per lato (`render_*` e `Player::render_*`) |
| Scala di render / GIF | finita e compresa in `0.05..=4`, altrimenti `InvalidArgument` |
| Posizioni dei fotogrammi | devono essere finite; i valori fuori dalla timeline vengono **limitati** a `0..=frames-1` (nessun errore). I fotogrammi frazionari vanno bene. |
| Chiavi (`set_key`, `delete_key`) | `frame` e `value` finiti; `prop` è un nome di proprietà in snake_case dal JSON del progetto (`"bone_angle"`, `"layer_opacity"`, ...) |
| Progetto | `fps` in `(0, 1000]`, `frames` in `1..=200000`, palco fino a 32768 unità per lato, al massimo 4096 ossi, numeri oltre +-1e7 rifiutati al caricamento; ulteriori limiti per singola funzionalità sono validati al caricamento e tornano come `InvalidProject` |
| `Player::advance(dt)` | `dt` finito, `0..=10` secondi |
| `Player::set_canvas_size` | entrambi i lati finiti, `> 0`, al massimo 32768 |
| `Player::set_screen_size` | larghezza e altezza finite e `> 0`; `max_error_px` finito e `>= 0` |
| `Animation::lod_levels` | `tolerance_px > 0`; `scale` in `(0, 64]`; `levels` in `1..=12` |
| `Animation::fit_to_size` | `max_bytes` in `1000..=1_000_000_000` |
| `Animation::fuzz` | `samples` in `1..=100000` |
| `trace_image_svg`, `Animation::from_image` | `colors` in `2..=64`; byte PNG o JPEG (si applicano limiti di dimensione) |
| `vcs_log` | `limit` in `1..=10000` |

## Errori

Ogni funzione fallibile restituisce `Result<_, ApiError>`. L'input errato è un valore di errore e **non provoca mai un panic**; un panic che avviene comunque all'interno del motore viene intercettato al confine della facciata e restituito come `ApiError::Internal` (segnalalo come bug). `ApiError` implementa `Display` e `std::error::Error`, quindi `?` funziona con `Box<dyn Error>` e tipi in stile `anyhow`.

| Variante | Significato | Origini tipiche |
|---|---|---|
| `InvalidJson(String)` | non è JSON, non è un progetto, struttura errata, troppo grande (> `MAX_JSON_BYTES`), o un `format` più recente di questa build; anche un valore JSON errato passato a `vcs_resolve` | `Animation::from_json`, `validate_json`, `vcs_resolve` |
| `InvalidProject(Vec<String>)` | analizzato, ma la validazione è fallita; un messaggio per ogni problema (osso sconosciuto, intervallo errato, ciclo, ...) | `from_json`, `from_image`, ricostruzioni del motore da `Player` |
| `InvalidArgument(String)` | un argomento del chiamante è stato rifiutato: numero non finito, valore fuori intervallo, proprietà sconosciuta, id sconosciuto di macchina / input / scroll, BVH o immagine inutilizzabile, nome di branch errato, niente da committare, nessun repository | quasi ogni funzione |
| `EditRejected(Vec<String>)` | una modifica è stata rifiutata (per esempio un canale o un livello bloccato, o un risultato non valido); il progetto resta invariato | `set_key`, `delete_key`, `retarget_bvh` |
| `Export(String)` | un encoder è fallito o il formato non è possibile per questo progetto | `export`, `render_png`, `fit_to_size` |
| `Internal(String)` | un panic intercettato | qualsiasi |
| `DirtyTree(String)` | (1.3) il file di progetto ha modifiche non committate che l'operazione sovrascriverebbe | `vcs_checkout` (salvo `force`), `vcs_merge`, `vcs_cherry_pick`, `vcs_revert` |
| `MergeInProgress(String)` | (1.3) un merge, cherry-pick o revert è incompleto; risolvi e fai commit, oppure `vcs_abort` | le stesse funzioni |
| `Conflicts(Vec<String>)` | (1.3) `vcs_commit` mentre restano conflitti; un puntatore per ogni conflitto | `vcs_commit` |

Un controllo di CI delle animazioni che fallisce **non** è un errore: `ci_run` restituisce `Ok(CiOutcome)` con `exit_code` 1 (un controllo è fallito) o 2 (un controllo non ha potuto essere valutato); solo un manifest inutilizzabile è `Err`. Anche i conflitti di merge sono un risultato (`VcsMergeOutcome::Conflicted`), non un errore.

## Riferimento: `Animation`

Un progetto caricato e validato più la sua cronologia di annullamento. Si crea con `from_json`, `demo` o `from_image`. Tutti i metodi che leggono il progetto prendono `&self`.

| Metodo | Restituisce | Note |
|---|---|---|
| `Animation::from_json(&str)` | `Result<Animation, ApiError>` | analisi + validazione. `InvalidJson` / `InvalidProject`. I campi JSON sconosciuti qui vengono ignorati in silenzio (il comando `check` della CLI li segnala). |
| `Animation::validate_json(&str)` | `Result<(), ApiError>` | stessi controlli, non conserva nulla |
| `Animation::demo()` | `Animation` | piccolo rig integrato (6 ossi, 48 fotogrammi a 24 fps, 640 x 480); comodo per i test e i primi passi |
| `Animation::from_image(&[u8], colors, name)` | `Result<Animation, ApiError>` | vettorizza byte PNG/JPEG in livelli vettoriali riempiti su un palco delle dimensioni dell'immagine |
| `to_json(&self)` | `String` | il progetto corrente come JSON formattato (round trip attraverso `from_json`) |
| `info(&self)` | `AnimationInfo` | `name`, `width`, `height` (unità del palco), `fps`, `frames` |
| `frame_count(&self)`, `fps(&self)` | `u32`, `f32` | |
| `bone_names(&self)` | `Vec<String>` | ordine del rig = ordine di `bone_world` |
| `layer_names(&self)` | `Vec<String>` | percorsi dei livelli `group/child`, in profondità |
| `bone_world(&self, frame)` | `Result<Vec<[f32; 6]>, ApiError>` | matrice mondo per osso `[a b c d tx ty]`; un punto locale all'osso `(x, y)` viene mappato in `(a*x + c*y + tx, b*x + d*y + ty)`. Fotogramma limitato all'intervallo. Valuta l'intero rig (IK, vincoli, molle, espressioni). |
| `machine_names(&self)` | `Vec<String>` | macchine a stati di runtime memorizzate nel progetto |
| `player(&self, machine)` | `Result<Player, ApiError>` | avvia la macchina indicata al tempo 0 su una copia del progetto |
| `render_rgba(&self, frame, &RenderSettings)` | `Result<Image, ApiError>` | RGBA8 con alfa non premoltiplicato, per righe, riga superiore per prima |
| `render_png(&self, frame, &RenderSettings)` | `Result<Vec<u8>, ApiError>` | un file PNG in memoria |
| `render_svg(&self, frame, show_bones: bool)` | `Result<String, ApiError>` | un documento SVG (l'output vettoriale non ha scala) |
| `export(&self, ExportFormat, scale)` | `Result<Vec<u8>, ApiError>` | `Gif` (in loop, l'intera timeline; `scale` entro `0.05..=4`), `Glb` (glTF 2.0 binario), `Lottie` (byte JSON), `Html` (una pagina player autonoma); `scale` è ignorata dagli altri tre. Che cosa conserva o scarta ogni formato: [EXPORT_FORMATS.md](EXPORT_FORMATS.md). |
| `set_key(&mut self, target, prop, frame, value, Interpolation)` | `Result<(), ApiError>` | imposta o aggiunge una chiave su `target.prop`; validata, annullabile, rispetta i blocchi di canale e di livello (`EditRejected`) |
| `delete_key(&mut self, target, prop, frame)` | `Result<bool, ApiError>` | `true` se una chiave è stata rimossa |
| `add_rect_layer(&mut self, name, x, y, w, h, fill, parent)`, `add_ellipse_layer(&mut self, name, cx, cy, rx, ry, fill, parent)` | `Result<(), ApiError>` | (1.4) aggiunge in coda un livello di forma riempita (al livello superiore, o nel gruppo di primo livello indicato da `parent`); nome univoco senza `/`, dimensioni finite e positive, un colore valido; annullabile |
| `add_group_layer(&mut self, name)` | `Result<(), ApiError>` | (1.4) aggiunge in coda un livello gruppo vuoto |
| `add_bone(&mut self, name, parent, length, x, y, angle)` | `Result<(), ApiError>` | (1.4) aggiunge in coda un osso (`parent` deve esistere, angolo in gradi); annullabile |
| `bake_physics(&mut self, chain, preset, overwrite)` | `Result<PhysicsBake, ApiError>` | (1.4) simula una catena di ossi (radice per prima) con `hair`, `tail`, `rope`, `antenna` o `cape` e memorizza i canali angolari come un unico passo di annullamento; con `overwrite = false` gli ossi con chiavi manuali vengono conservati ed elencati in `skipped` |
| `render_exr(&self, frame, &RenderSettings, float32)` | `Result<Vec<u8>, ApiError>` | (1.4) file OpenEXR (lineare, premoltiplicato; HALF o FLOAT) dal compositor in virgola mobile |
| `undo(&mut self)`, `redo(&mut self)` | `bool` | `false` quando non c'è nulla da annullare / ripetere |
| `lod_levels(&self, tolerance_px, scale, levels)` | `Result<Vec<LodLevel>, ApiError>` | il livello 0 è l'originale, il livello k >= 1 resta entro `tolerance_px * 2^(k-1)` pixel di output a `scale`; ogni livello porta il proprio limite dichiarato e un JSON di progetto. Deterministico; il limite copre la riproduzione della timeline (non le pose delle macchine a stati). Lento nelle build non ottimizzate (usa release). |
| `fit_to_size(&self, FitFormat, max_bytes)` | `Result<SizeFit, ApiError>` | la più piccola esportazione entro il budget, abbassando passo dopo passo le manopole di qualità; se impossibile, il risultato più piccolo con `met == false` e un `reason` |
| `retarget_bvh(&mut self, bvh_text, &MocapSettings)` | `Result<MocapReport, ApiError>` | retarget BVH come **un'unica modifica annullabile**; ossi associati per nome e topologia; i canali bloccati vengono saltati e segnalati |
| `fuzz(&self, samples, seed)` | `Result<FuzzSummary, ApiError>` | fuzzing delle pose; l'animazione viene solo letta; report completo come JSON |

`Interpolation` del segmento che parte da una chiave: `Linear`, `Step` (mantenimento), `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` (Catmull-Rom attraverso le chiavi vicine). Le curve più ricche del formato di progetto (Bezier cubica, bounce, back, spring) si creano tramite la CLI e lo studio, non tramite `set_key`.

## Riferimento: `Player`

Una macchina a stati in esecuzione (riproduzione interattiva). Creato da `Animation::player`; lavora su una copia del progetto; deterministico. Le macchine a stati e i data binding sono semplice JSON di progetto (`machines`, `bindings`), riassunti nell'[appendice](#appendice-json-delle-macchine-e-json-dei-binding).

| Metodo | Restituisce | Note |
|---|---|---|
| `set_bool(name, bool)`, `set_number(name, f64)`, `fire(name)` | `Result<(), ApiError>` | imposta un input per nome; il tipo di input sbagliato, o un nome sconosciuto, è `InvalidArgument`. Un trigger resta impostato finché non scatta una transizione che lo verifica (viene consumato) o fino alla fine del successivo `advance`; `hold: true` lo mantiene finché non viene consumato. |
| `set_value(path, f64)` | `Result<(), ApiError>` | pubblica un valore esterno per i data binding (`"player.health"`). Una regione di scorrimento il cui `input` è questo percorso prende il valore come proprio offset in pixel lungo il suo asse principale (lo scorrimento è allora a riposo). |
| `advance(dt)` | `Result<(), ApiError>` | fa avanzare macchina e regioni di scorrimento di `dt` secondi (`0..=10`) |
| `state(layer)` | `Option<String>` | stato corrente di un livello macchina; `None` per un livello sconosciuto |
| `events_json()` | `String` | eventi in sospeso `[{"time":..,"layer":..,"kind":"entered"|"exited","state":..}]`, non consumati |
| `take_events_json()`, `clear_events()` | `String`, `()` | consuma / scarta gli eventi in sospeso |
| `render_rgba(&RenderSettings)`, `render_svg(show_bones)`, `bone_world()` | come su `Animation` | la posa corrente; l'immagine ha le dimensioni del canvas impostate da `set_canvas_size` moltiplicate per la scala |
| `set_canvas_size(w, h)`, `canvas_size()` | `Result<(), ApiError>`, `(f32, f32)` | (1.2) layout responsivo: i contenitori vengono risolti di nuovo, i limiti di scorrimento seguono |
| `scroll_ids()` | `Vec<String>` | id delle regioni di scorrimento del progetto |
| `scroll_drag(id, x, y)` | `Result<(), ApiError>` | offset grezzo del puntatore in pixel (contenuto spostato in alto / a sinistra di tale quantità); oltre le estremità viene limitato o rimbalza elasticamente |
| `scroll_release(id, vx, vy)` | `Result<(), ApiError>` | velocità di rilascio in px/s; `advance` poi lancia, aggancia e fa rimbalzare indietro come una molla |
| `scroll_offset(id)`, `scroll_max(id)`, `scroll_settled(id)` | `Option<[f32; 2]>`, `Option<[f32; 2]>`, `Option<bool>` | `None` per un id sconosciuto |
| `set_screen_size(w_px, h_px, max_error_px)` | `Result<(), ApiError>` | (1.2) sceglie il più grossolano degli otto livelli di dettaglio precostruiti il cui limite dichiarato, alla scala che adatta il canvas a quello schermo, sia al massimo `max_error_px`. I livelli vengono costruiti una sola volta al primo uso (livello 1 limitato a 0.125 unità del palco, raddoppiando) e messi in cache; con `max_error_px` 0 si qualificano solo i livelli senza perdita. |
| `clear_screen_size()`, `lod_level()`, `lod_bound_px()` | `()`, `usize`, `f32` | torna all'originale; livello in uso (0 = originale); il suo limite dichiarato in pixel |

Note: il limite del livello di dettaglio copre la geometria e le chiavi della timeline come documentato in `lod.rs` (`ASSUMPTIONS`, `LIMITS`); le pose delle macchine a stati e i binding non vengono modellati, e i canali di posa che puntano a un livello rimosso da un livello di dettaglio vengono scartati. I proxy di scrubbing dello studio usano un meccanismo separato con una propria griglia di livelli. L'`input` di una regione di scorrimento passa dalla tabella dei valori del player, non dal runtime dei data binding, quindi espressioni, rimappatura e smoothing non si applicano.

## Riferimento: funzioni libere (strumenti, controllo di versione, CI)

| Funzione | Restituisce | Cosa fa |
|---|---|---|
| `trace_image_svg(&[u8], colors)` | `Result<String, ApiError>` | vettorizza una bitmap in un SVG di regioni riempite |
| `vcs_log(path, limit)` | `Result<Vec<VcsCommit>, ApiError>` | (1.2, sola lettura) cronologia dal più recente; `path` è il file di progetto o qualsiasi percorso all'interno del suo repository; una cronologia vuota dà un elenco vuoto |
| `ci_run(manifest)` | `Result<CiOutcome, ApiError>` | (1.2) CI delle animazioni in modalità verifica (nulla viene registrato di nuovo); i percorsi relativi nel manifest si risolvono accanto ad esso |
| `vcs_init(project_file, author)` | `Result<(), ApiError>` | (1.3) avvia una cronologia accanto al file sul branch `main`, ancora senza commit; fallisce se ne esiste una |
| `vcs_commit(path, message, author)` | `Result<String, ApiError>` | id del commit; conclude un merge quando non restano conflitti; `Conflicts` finché ne restano; niente da committare o nessun messaggio è `InvalidArgument` |
| `vcs_branch(path, name, at)`, `vcs_tag(path, name, at)` | `Result<String, ApiError>` | crea un branch / tag (predefinito: commit corrente); non vi si passa; i tag non si spostano mai; viene restituito l'id del commit |
| `vcs_checkout(path, target, force)` | `Result<String, ApiError>` | porta il file su un branch (vi rimane) o su una qualsiasi revisione (detached); `DirtyTree` salvo `force`; `MergeInProgress` |
| `vcs_merge(path, rev, no_ff, message, author)` | `Result<VcsMergeOutcome, ApiError>` | `UpToDate`, `FastForward(id)`, `Merged(id)` o `Conflicted(Vec<VcsConflict>)` |
| `vcs_cherry_pick(path, rev, author)`, `vcs_revert(path, rev, author)` | `Result<VcsMergeOutcome, ApiError>` | riapplica / annulla la modifica di un commit |
| `vcs_conflicts(path)` | `Result<Vec<VcsConflict>, ApiError>` | i conflitti ancora in sospeso (vuoto se non ce ne sono) |
| `vcs_resolve(path, conflict_path, &VcsChoice)` | `Result<usize, ApiError>` | risolve un conflitto con `Ours`, `Theirs` o `Value(json text)`; restituisce quanti ne restano |
| `vcs_abort(path)` | `Result<(), ApiError>` | scarta un merge / cherry-pick / revert incompleto e ripristina l'ultimo commit |
| `vcs_branches(path)` | `Result<Vec<VcsBranch>, ApiError>` | (1.4) i branch con il loro commit e se è quello attivo |
| `vcs_branch_delete(path, name)` | `Result<(), ApiError>` | (1.4) elimina un branch; il branch attivo e i nomi sconosciuti vengono rifiutati; gli oggetti restano fino a `vcs_gc` |
| `vcs_status(path)` | `Result<VcsStatus, ApiError>` | (1.4) branch, HEAD, modifiche semantiche non committate, merge incompleto, percorsi dei conflitti |
| `vcs_diff(path, a, b)` | `Result<Vec<VcsChange>, ApiError>` | (1.4) differenze semantiche tra due revisioni; `None` è il file di lavoro |
| `vcs_show(path, rev)` | `Result<VcsShow, ApiError>` | (1.4) un commit e le sue modifiche rispetto al primo genitore |
| `vcs_gc(path)` | `Result<VcsGc, ApiError>` | (1.4) elimina gli oggetti non raggiungibili; si interrompe senza eliminare se la cronologia non può essere letta |
| `vcs_fsck(path)` | `Result<Vec<String>, ApiError>` | (1.4) problemi di integrità, vuoto se tutto è sano |
| `ci_init(project, manifest, write_goldens)` | `Result<CiInit, ApiError>` | scrive un manifest calibrato predefinito (non sovrascrive mai un file esistente; nome predefinito `osatura-ci.json` accanto al progetto); facoltativamente registra subito i golden |
| `ci_run_mode(manifest, CiMode)` | `Result<CiOutcome, ApiError>` | `Verify`, `Update` (registra di nuovo tutti i golden dei controlli di render), `Bless` (accetta solo quelli falliti o mancanti) |
| `ci_bless(manifest)` | `Result<CiOutcome, ApiError>` | abbreviazione di `Bless` |

La cronologia è memorizzata accanto al file di progetto (`.osatura-vcs/`); i messaggi di commit e gli autori sono testo semplice. Un conflitto contiene il puntatore di identità `path`, un `kind`, una `description` e i valori `base` / `ours` / `theirs` come testo JSON (`None` = assente lì). Dopo un merge con conflitti il file di progetto contiene lo stato unito con gli elementi in conflitto al valore del branch corrente, e viene registrato uno stato in sospeso finché non risolvi ogni conflitto e fai `vcs_commit`, oppure `vcs_abort`. Gli asset binari a cui un progetto fa riferimento non sono versionati, solo le loro stringhe di percorso. Il merge semantico, i suoi tipi di conflitto e i suoi limiti sono descritti in [ROADMAP.md](ROADMAP.md) sezione 9 (voce I-01) e in [LIMITATIONS.md](LIMITATIONS.md).

Non presenti nella facciata: override della mappatura degli ossi per il mocap, regolazioni della vettorizzazione oltre al numero di colori, opzioni di fit oltre a formato e budget, rendering GPU, lo studio, la collaborazione, il server per agenti, i numerosi comandi di authoring e baking (fisica, foot lock, magneti, autorig, ...), l'eliminazione / elenco dei branch, diff, show, status, gc, fsck e opzioni di CI come prune, `--only` e gli artefatti. Usa la CLI per questi ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

## Riferimento: tipi

| Tipo | Campi / varianti |
|---|---|
| `AnimationInfo` | `name: String`, `width: f32`, `height: f32` (unità del palco), `fps: f32`, `frames: u32` |
| `RenderSettings` | `scale: f32` (0.05..=4), `show_bones: bool`, `transparent: bool`; `new(scale)`, `with_bones(bool)`, `with_transparent(bool)`; `Default` = scala 1, entrambi disattivati |
| `Image` | `width: u32`, `height: u32`, `pixels: Vec<u8>` (RGBA8, alfa non premoltiplicato) |
| `ExportFormat` | `Gif`, `Glb`, `Lottie`, `Html`, `Apng` (1.4) |
| `PhysicsBake` (1.4) | `added: usize`, `skipped: Vec<String>` |
| `VcsBranch` (1.4) | `name`, `commit`, `current: bool` |
| `VcsChange` (1.4) | `kind` (`"Added"`, `"Removed"`, `"Changed"`, `"Moved"`, `"Retimed"`), `path`, `description` |
| `VcsStatus` (1.4) | `branch`, `head`, `changes: Vec<VcsChange>`, `merge`, `conflicts: Vec<String>` |
| `VcsShow` (1.4) | `commit: VcsCommit`, `changes: Vec<VcsChange>` |
| `VcsGc` (1.4) | `kept`, `removed` |
| `Interpolation` | `Linear`, `Step`, `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` |
| `LodLevel` | `level`, `declared_px: f64`, `bound_stage: f64`, `path_anchors`, `keys`, `layers`, `json: String`; `bound_at(screen_scale) -> f64` |
| `FitFormat` | `Lottie`, `Html`, `Gif`, `Apng`, `Project` |
| `SizeFit` | `met: bool`, `size: usize`, `bytes: Vec<u8>`, `ssim_min: f64`, `reason: Option<String>` |
| `MocapSettings` | `in_place`, `mirror`, `smooth` (raggio della media mobile in fotogrammi); `new()`, `with_in_place`, `with_mirror`, `with_smooth`; `Default` |
| `MocapReport` | `matched: Vec<(String, String)>` (giunto BVH, osso del rig), `unmatched_joints`, `unmatched_bones`, `frames`, `bones_used`, `blocked` (`target.prop` bloccati), `notes` |
| `FuzzSummary` | `samples`, `failures`, `failure_rate: f32`, `max_severity: f32`, `json: String` |
| `VcsCommit` | `id`, `author`, `time: u64` (secondi Unix), `message` |
| `VcsConflict` | `path`, `kind`, `description`, `base`, `ours`, `theirs: Option<String>` |
| `VcsMergeOutcome` | `UpToDate`, `FastForward(String)`, `Merged(String)`, `Conflicted(Vec<VcsConflict>)` |
| `VcsChoice` | `Ours`, `Theirs`, `Value(String)` |
| `CiMode` | `Verify`, `Update`, `Bless` |
| `CiOutcome` | `total`, `passed`, `failed`, `errors`, `exit_code: i32` (0 buono, 1 fallito, 2 non valutabile), `json: String`, `updated`, `blessed` |
| `CiInit` | `manifest: String`, `checks: Vec<(String, String)>` (tipo, nome), `recorded: Option<CiOutcome>` |
| Costanti | `API_VERSION: &str`, `API_VERSION_MAJOR: u32`, `MAX_RENDER_SIDE: u32`, `MAX_JSON_BYTES: usize` |

## Esempi

Tutti questi esempi sono completi e sono stati eseguiti. Presuppongono `use osatura_core::prelude::*;`.

### Caricare, valutare, renderizzare, esportare

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Load a project, query it, render one frame and export the whole animation.
pub fn quick(project_path: &str) -> Result<(), Box<dyn Error>> {
    let json = std::fs::read_to_string(project_path)?;
    let mut anim = Animation::from_json(&json)?;           // parse + validate; typed error otherwise

    // Facts about the project.
    let info = anim.info();
    println!("{}: {}x{} @ {} fps, {} frames", info.name, info.width, info.height, info.fps, info.frames);

    // Evaluate: world matrices of every bone at frame 10 (six floats per bone: a b c d tx ty).
    let names = anim.bone_names();
    let world = anim.bone_world(10.0)?;
    let m = world[0];
    // A bone-local point (x, y) lands at (a*x + c*y + tx, b*x + d*y + ty).
    let (x, y) = (m[0] * 10.0 + m[2] * 0.0 + m[4], m[1] * 10.0 + m[3] * 0.0 + m[5]);
    println!("tip of {} (10 units along its x axis) is at ({x:.1}, {y:.1})", names[0]);

    // Edit through the undoable editor, then render.
    anim.set_key(&names[0], "bone_angle", 10.0, 25.0, Interpolation::EaseInOut)?;
    let png = anim.render_png(10.0, &RenderSettings::new(1.0).with_bones(true))?;
    std::fs::write("frame10.png", &png)?;
    let svg = anim.render_svg(10.0, false)?;
    std::fs::write("frame10.svg", svg)?;

    // Raw pixels: straight-alpha RGBA8, row-major, top row first.
    let img = anim.render_rgba(10.0, &RenderSettings::new(0.5).with_transparent(true))?;
    assert_eq!(img.pixels.len(), img.width as usize * img.height as usize * 4);

    // Whole-animation exports.
    let glb = anim.export(ExportFormat::Glb, 1.0)?;
    let gif = anim.export(ExportFormat::Gif, 0.5)?;        // `scale` only matters for GIF
    std::fs::write("anim.glb", &glb)?;
    std::fs::write("anim.gif", &gif)?;
    println!("{} bones, {} bytes of glb, {} bytes of gif", names.len(), glb.len(), gif.len());
    Ok(())
}
```

### Modifica, annullamento e gestione degli errori

```rust
use osatura_core::prelude::*;

/// Edits are validated, undoable and all-or-nothing.
pub fn edit_and_errors() {
    let mut anim = Animation::demo();                       // small built-in project
    let bone = anim.bone_names()[0].clone();
    let before = anim.to_json();

    anim.set_key(&bone, "bone_angle", 4.0, 30.0, Interpolation::Smooth).unwrap();
    assert!(anim.delete_key(&bone, "bone_angle", 4.0).unwrap());      // true: a key was removed
    assert!(!anim.delete_key(&bone, "bone_angle", 77.0).unwrap());    // false: nothing there
    assert!(anim.undo() && anim.undo());                    // each successful edit is one undo step
    assert_eq!(anim.to_json(), before);
    assert!(anim.redo());

    // Errors are values. Match on them with a wildcard arm: the enum is #[non_exhaustive].
    match anim.set_key(&bone, "no_such_prop", 0.0, 1.0, Interpolation::Linear) {
        Err(ApiError::InvalidArgument(why)) => println!("rejected argument: {why}"),
        Err(ApiError::EditRejected(problems)) => println!("edit refused: {problems:?}"),
        Err(other) => println!("other failure: {other}"),   // Display is implemented for every variant
        Ok(()) => unreachable!(),
    }
    // A rejected edit leaves the project exactly as it was.
    let snapshot = anim.to_json();
    assert!(anim.set_key("no_such_bone", "bone_angle", 1.0, 1.0, Interpolation::Linear).is_err());
    assert_eq!(anim.to_json(), snapshot);

    // Bad input never panics.
    assert!(matches!(Animation::from_json("{ nope"), Err(ApiError::InvalidJson(_))));
    assert!(anim.render_rgba(f32::NAN, &RenderSettings::default()).is_err());
    assert!(anim.render_rgba(0.0, &RenderSettings::new(50.0)).is_err()); // scale must be 0.05..=4
}
```

### Riproduzione interattiva con una macchina a stati

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Run a state machine of a project (machine.json has machine "hero", layer "body", bool input "excited").
pub fn play(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let machine = anim.machine_names()[0].clone();         // e.g. "hero"
    let mut player = anim.player(&machine)?;               // works on a copy of the project

    player.set_bool("excited", true)?;                     // also set_number, fire (trigger), set_value (binding path)
    for _ in 0..30 {
        player.advance(1.0 / 30.0)?;                       // deterministic; dt in seconds, 0..=10
    }
    println!("body is in state {:?}", player.state("body"));  // Some("wave")
    println!("{}", player.take_events_json());             // [{"time":..,"layer":"body","kind":"entered","state":..}, ..]

    let frame = player.render_rgba(&RenderSettings::new(0.5))?;
    println!("pose rendered at {}x{}", frame.width, frame.height);
    let svg = player.render_svg(false)?;
    assert!(svg.starts_with("<svg"));

    // Same inputs, same advance calls, same pose: a second player is bit-for-bit identical.
    let mut again = anim.player(&machine)?;
    again.set_bool("excited", true)?;
    for _ in 0..30 {
        again.advance(1.0 / 30.0)?;
    }
    assert_eq!(again.render_rgba(&RenderSettings::new(0.5))?, frame);

    // Wrong calls are typed errors, not panics.
    assert!(matches!(player.set_number("excited", 1.0), Err(ApiError::InvalidArgument(_))));
    assert!(matches!(anim.player("no such machine"), Err(ApiError::InvalidArgument(_))));
    Ok(())
}
```

### Layout, scorrimento e livello di dettaglio

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Layout, scrolling and level of detail on a Player (scroll.json has one scroll region "list").
pub fn layout_and_lod(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let mut player = anim.player(&anim.machine_names()[0])?;

    player.set_canvas_size(100.0, 120.0)?;                 // containers re-solve; renders get this size
    for id in player.scroll_ids() {
        let max = player.scroll_max(&id).unwrap();         // about [0, 80] here: 200 units of content in a 120 unit view
        player.scroll_drag(&id, 0.0, 50.0)?;               // raw pointer offset in px (clamped / rubber-banded)
        assert_eq!(player.scroll_offset(&id), Some([0.0, 50.0]));
        player.scroll_release(&id, 0.0, 900.0)?;           // fling at 900 px/s
        while player.scroll_settled(&id) == Some(false) {
            player.advance(1.0 / 60.0)?;                   // steps the machine and every scroll region
        }
        println!("{id}: max {max:?}, rests at {:?}", player.scroll_offset(&id).unwrap());
    }

    // Pick the coarsest level of detail whose declared error stays within 0.5 px at this screen size.
    player.set_screen_size(100.0, 120.0, 0.5)?;
    println!("level {} (declared bound {} px)", player.lod_level(), player.lod_bound_px());
    player.clear_screen_size();                            // back to the original
    assert_eq!(player.lod_level(), 0);

    // A scroll region whose `input` names a path follows set_value(path, px) instead of a drag.
    player.set_value("ui.scroll", 200.0)?;
    Ok(())
}
```

### Rendering su più thread

```rust
use osatura_core::prelude::*;

/// `Animation` and `Player` are `Send + Sync`: read-only calls (`&self`) may run on several threads.
pub fn render_in_parallel() -> Result<(), ApiError> {
    let anim = Animation::demo();
    let frames = anim.frame_count();
    let settings = RenderSettings::new(0.25);
    let (a, s) = (&anim, &settings);                        // shared references are Copy: move them into the threads
    let images: Vec<Result<Image, ApiError>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..frames).map(|f| scope.spawn(move || a.render_rgba(f as f32, s))).collect();
        jobs.into_iter().map(|j| j.join().expect("render thread")).collect()
    });
    // Mutating calls (`&mut self`: set_key, undo, retarget_bvh ...) need exclusive access.
    for (f, img) in images.into_iter().enumerate() {
        assert_eq!(img?, anim.render_rgba(f as f32, &settings)?);   // threads do not change the result
    }
    Ok(())
}
```

### Controllo di versione e CI delle animazioni

```rust
use osatura_core::prelude::*;

/// Version control of a project file and animation CI, from Rust.
pub fn history(dir: &std::path::Path) -> Result<(), ApiError> {
    let file = dir.join("hero.json");
    std::fs::write(&file, Animation::demo().to_json()).unwrap();
    let path = file.to_str().unwrap();

    // A tiny helper: re-key the first bone at frame 5 and save the file.
    let edit = |angle: f32| -> Result<(), ApiError> {
        let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
        let bone = a.bone_names()[0].clone();
        a.set_key(&bone, "bone_angle", 5.0, angle, Interpolation::Linear)?;
        std::fs::write(path, a.to_json()).unwrap();
        Ok(())
    };

    vcs_init(path, Some("ana"))?;                           // branch `main`, no commit yet
    vcs_commit(path, Some("base"), None)?;
    vcs_branch(path, "other", None)?;                       // created, not switched to
    edit(20.0)?;
    vcs_commit(path, Some("main edit"), None)?;
    vcs_checkout(path, "other", false)?;                    // DirtyTree if the file has uncommitted changes
    edit(40.0)?;                                            // the same key, another value
    vcs_commit(path, Some("other edit"), None)?;
    vcs_checkout(path, "main", false)?;

    match vcs_merge(path, "other", false, None, None)? {
        VcsMergeOutcome::Conflicted(conflicts) => {         // a normal result, not an error
            for c in &conflicts {
                println!("{} ({}): {}", c.path, c.kind, c.description);
                vcs_resolve(path, &c.path, &VcsChoice::Theirs)?;
            }
            vcs_commit(path, None, None)?;                  // the merge commit
        }
        other => println!("merged without conflicts: {other:?}"),
    }
    for c in vcs_log(path, 10)? {
        println!("{} {} {}", &c.id[..8], c.author, c.message);
    }

    // Animation CI: write a calibrated manifest, record goldens, then verify.
    let init = ci_init(path, None, true)?;                  // never overwrites an existing manifest
    let run = ci_run(&init.manifest)?;                      // verify mode: nothing is re-recorded
    println!("{} checks, {} passed, exit code {}", run.total, run.passed, run.exit_code);
    if run.exit_code == 1 {
        // A failing check is a result, not an Err. After reviewing the report, accept the new look:
        ci_bless(&init.manifest)?;
    }
    Ok(())
}
```

### Mocap, fuzzing, vettorizzazione, livello di dettaglio ed esportazione con dimensione obiettivo

```rust
use osatura_core::prelude::*;

const RIG: &str = r#"{"format":1,"name":"rig","width":400.0,"height":400.0,"fps":30.0,"frames":10,"background":null,
 "camera":{"position":null,"zoom":1.0,"rotation":0.0},
 "bones":[{"name":"hips","parent":null,"length":20.0,"rest_offset":[200.0,250.0],"rest_angle":-90.0},
          {"name":"spine","parent":"hips","length":40.0,"rest_offset":[20.0,0.0],"rest_angle":0.0},
          {"name":"head","parent":"spine","length":30.0,"rest_offset":[40.0,0.0],"rest_angle":0.0}],
 "ik":[],"constraints":[],"springs":[],"layers":[],"channels":[]}"#;

fn mini_bvh() -> String {
    let mut s = String::from("HIERARCHY\nROOT Hips\n{\n OFFSET 0 100 0\n CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation\n");
    s.push_str(" JOINT Spine\n {\n  OFFSET 0 12 0\n  CHANNELS 3 Zrotation Xrotation Yrotation\n  JOINT Head\n  {\n   OFFSET 0 12 0\n   CHANNELS 3 Zrotation Xrotation Yrotation\n   End Site\n   {\n    OFFSET 0 8 0\n   }\n  }\n }\n}\nMOTION\nFrames: 4\nFrame Time: 0.0333333\n");
    for f in 0..4 {
        s.push_str(&format!("0 100 0 0 0 0 {} 0 0 0 0 0\n", f * 10));
    }
    s
}

/// Motion capture, pose fuzzing, tracing, level of detail and size-targeted export.
pub fn tools() -> Result<(), ApiError> {
    // BVH retargeting: one undoable edit; bones are matched by name and topology.
    let mut rig = Animation::from_json(RIG)?;
    let report = rig.retarget_bvh(&mini_bvh(), &MocapSettings::new().with_smooth(1))?;
    println!("matched {:?}; {} frames on {} bones", report.matched, report.frames, report.bones_used);
    assert!(rig.undo());                                    // the whole retarget is one step

    // Pose fuzzing: where does the rig break?  (deterministic for a given seed)
    let demo = Animation::demo();
    let fuzz = demo.fuzz(50, 1)?;
    println!("{} of {} samples failed (max severity {:.2})", fuzz.failures, fuzz.samples, fuzz.max_severity);
    let _full_report: &str = &fuzz.json;                    // schema "osatura-fuzz-report" v1

    // Bitmap tracing: PNG/JPEG bytes -> SVG, or straight into an animation.
    let png = demo.render_png(0.0, &RenderSettings::new(0.25))?;
    let svg = trace_image_svg(&png, 6)?;
    let traced = Animation::from_image(&png, 6, "traced")?;
    println!("traced SVG: {} bytes; project has {} layers", svg.len(), traced.layer_names().len());

    // Level of detail with a declared error bound, one project JSON per level (level 0 = original).
    for l in demo.lod_levels(0.5, 1.0, 3)? {
        println!("L{}: bound {:.3} px, {} anchors, {} keys", l.level, l.declared_px, l.path_anchors, l.keys);
        let _reduced = Animation::from_json(&l.json)?;
    }

    // Smallest export within a byte budget.
    let fit = demo.fit_to_size(FitFormat::Lottie, 250_000)?;
    println!("met={} {} bytes, worst SSIM {:.3}", fit.met, fit.size, fit.ssim_min);
    Ok(())
}
```

### Come vengono verificati gli esempi

Il codice qui sopra è il codice che è stato eseguito: questo file è generato a partire dai sorgenti degli esempi, che sono stati compilati con `cargo` rispetto ad `osatura-core` ed eseguiti (i file di progetto usati sono l'output di `osatura demo`, `take-record`, `clip add` e `machine add`, più un progetto con un contenitore e una regione di scorrimento). I doc-test in `api/` (`cargo test --doc -p osatura-core`: 19 doc-test, 18 su funzioni della facciata in `api/` e l'esempio a livello di crate in `lib.rs`; circa 170 s in una build di debug perché ciascuno compila un programma) coprono i wrapper di controllo di versione e CI sul repository stesso.

## Altri linguaggi

La facciata è l'unica fonte di verità per gli altri binding. Copertura:

| Capacità | Facciata Rust | ABI C (`osatura.h`) | WebAssembly / JS |
|---|---|---|---|
| Caricamento, validazione, info, nomi degli ossi / matrici mondo | sì | `osatura_open`, `_bone_*`, `_frame_count`, ... | `arm_open`, `load()` (senza matrici degli ossi) |
| Rendering RGBA | sì | `osatura_render_rgba` | `arm_render_rgba`, `rgba()` |
| Rendering SVG / PNG / formati di esportazione | sì | no | SVG sì (`svg()`), PNG / esportazione no |
| Modifica delle chiavi, annulla / ripeti | sì | no | no |
| `Player`: input, advance, stato, eventi, rendering | sì | `osatura_player_*` | `arm_player_*`, `startPlayer()`, ... |
| Layout, scorrimento, livello di dettaglio | sì | `osatura_player_set_canvas`, `_scroll_*`, `_set_screen`, `_lod_level` | `arm_player_set_canvas / layout / scroll_* / set_screen` |
| Strumenti, controllo di versione, CI | sì | no | no |

La versione dell'ABI C resta 1 perché ogni aggiunta finora è stata additiva. Il binario wasm è compilato e testato sotto Node in CI ma non è stato eseguito in un browser; vedi [WASM.md](WASM.md).

## Appendice: JSON delle macchine e JSON dei binding

Brevi riepiloghi delle due sezioni di progetto eseguite dal `Player` (formato completo: [PROJECT_FORMAT.md](PROJECT_FORMAT.md)).

### JSON delle macchine (riepilogo)

```json
{"name":"hero",
 "inputs":[{"name":"moving","kind":"bool"},{"name":"speed","kind":"number"},{"name":"jump","kind":"trigger"}],
 "layers":[{"name":"body","mask":["arm_*"],"entry":"idle",
   "states":[{"name":"idle","source":{"kind":"clip","clip":"idle"},"blend_in":0.2},
             {"name":"loco","source":{"kind":"blend","input":"speed","children":[
                {"threshold":0,"source":{"kind":"clip","clip":"walk"}},{"threshold":1,"source":{"kind":"clip","clip":"run"}}]}},
             {"name":"jump","source":{"kind":"clip","clip":"jump"},"mode":"once"}],
   "transitions":[{"from":"idle","to":"loco","when":[{"input":"moving"}],"duration":0.25,"ease":{"type":"ease_in_out"}},
                  {"from":"*","to":"jump","when":[{"input":"jump"}],"duration":0.1},
                  {"from":"jump","to":"idle","exit_time":1.0}]}]}
```

Le condizioni sono `{"input","op":"eq|ne|lt|le|gt|ge","value"}` (valori predefiniti `eq`, `1`; devono valere tutte). `from: "*"` corrisponde a qualsiasi stato tranne la destinazione. Vince la prima transizione corrispondente in ordine di dichiarazione. `exit_time` è la frazione riprodotta (0..1) dello stato. Sorgenti: `clip` (una voce di `Project.clips`), `segment` (i fotogrammi `start`, `end` dei canali con chiavi in modalità replace del progetto), `region` (un marcatore con una fine), `blend` (1-D, due o più sorgenti foglia a soglie crescenti). Le chiavi sconosciute in una transizione o in una condizione sono un errore. CLI: `machine add|list|remove|validate`, `machine-sim <project> <script.json> <out_dir|out.json>` (script: `{"machine":"hero","duration":2,"events":[{"t":0.5,"set":{"moving":true},"trigger":["jump"],"value":{"player.health":40}}]}`).

### JSON dei binding (riepilogo)

```json
{"name":"hp","path":"player.health","expr":"value / 100","remap":{"in_min":0,"in_max":1,"out_min":-30,"out_max":30},
 "smooth":0.15,"target":{"kind":"prop","target":"arm_up","prop":"bone_angle"}}
```

Pipeline: il valore, poi `expr` (variabile `value`; le funzioni di espressione del motore, sorgente al massimo di 4 KB e annidamento al massimo di 64), poi `remap` (limitato per impostazione predefinita; `curve` ammorbidisce), poi lo smoothing (costante di tempo in secondi), poi la destinazione: `{"kind":"prop",...}` (qualsiasi proprietà di canale, incluse `layer_opacity` e `layer_switch`) oppure `{"kind":"input","machine","input"}` (un input di tipo trigger scatta sul fronte di salita). CLI: `bind add|list|remove`, `bind-eval`.

### Layout e normal map per livello

I contenitori di layout e le regioni di scorrimento sono campi di progetto `containers` / `scrolls` (omessi quando vuoti): CLI `container add|list|remove|solve`, `scroll add|list|remove|eval`, `layout-preview <project> <out.svg> --size WxH`, e `--size WxH` su `render`, `frames` ed `export` (i contenitori vengono risolti di nuovo per quella dimensione di canvas; il file resta invariato). `Layer.normal_map` (`data` PNG in base64 oppure `path`, `fit` stretch / tile / exact, `strength`, `green_down`) alimenta l'effetto di post-elaborazione `relight` per i pixel che quel livello, o un figlio senza una propria mappa, copre (CLI `layer-normal`, `fx ... --layer-normals`); non fa parte della facciata.
