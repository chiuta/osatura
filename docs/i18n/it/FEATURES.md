# OSATURA — matrice delle funzionalità + checklist di riferimento e della concorrenza

> Diciture di versione: tutti i crate sono `0.1.0` (le prime build sono la pre-release in bozza `v0.1.0-alpha.3`); "v0.5 / v0.6 / v0.7" in questo repository sono *livelli di funzionalità* (vedi [CHANGELOG.md](CHANGELOG.md)), e la sezione 9 di `ROADMAP.md` è l'autorità su ciò che è stato realizzato.

Stato onesto di ciò che esiste. **✅ fatto e testato · 🟡 parziale · ❌ non implementato.**

> Nota sull'ambito: OSATURA è un motore implementato in modo indipendente (a partire da specifiche e articoli pubblici; le liste di funzionalità dei concorrenti sono state consultate per definire l'ambito; nessun codice sorgente di terzi è stato copiato; vedi CLEANROOM.md per la dichiarazione sulla ricerca), non una reimplementazione di alcun
> prodotto commerciale, e **non** raggiunge la parità con una suite commerciale di 25 anni
> (GUI matura, strumenti di disegno, input dal vivo, 3D, ecosistema). Le prime tabelle sono organizzate per
> capacità; la **seconda metà di questo file** è una checklist versione per versione delle
> funzionalità documentate pubblicamente del prodotto di riferimento (fonti elencate in fondo) con
> lo stato di OSATURA per ciascuna. Le "innovazioni" sono dichiarate solo rispetto alla tecnica generica,
> non come dimostrata assenza da qualsiasi altro strumento.

---

# ★ SCALA DI REALIZZAZIONE — dal semplice al complesso, con stato di copyright / IP (livello di funzionalità v0.7, 20-22 ott 2026)

Questa sezione è l'**unico posto in cui vedere cosa si può costruire, in quale ordine e se è sicuro**.
Sostituisce gli elenchi sparsi di "candidati n." più avanti; dettagli e ID si trovano in `ROADMAP.md`
(R-xxx / I-xx), il metodo clean-room in `CLEANROOM.md`, le licenze in `LICENSING.md`.

**Stato:** ✅ realizzato e testato · 🟡 realizzato in parte · ⬜ pianificato (non iniziato)
**Stato IP (dopo le alternative di ROADMAP §8):**
🟢 sicuro — standard aperto, matematica da manuale o progetto nostro ·
🟠 sicuro *se* si rispetta una condizione (indicata nell'ultima colonna) ·
🔴 da non realizzare.
*Verifica tecnica, non consulenza legale; un avvocato dovrebbe confermare ogni 🟠 prima di un rilascio commerciale.*

## Livello 1 — Fondamenta (da ore a giorni ciascuna; nessuna esposizione IP)
| Funzionalità | Stato | IP | Base sicura / condizione |
|---|---|---|---|
| JSON di progetto, validazione, `evaluate(frame)` pura | ✅ | 🟢 | schema nostro |
| Percorsi di Bézier, riempimenti, tratti, gradienti, tratteggi, tratti a larghezza variabile | ✅ | 🟢 | geometria da manuale |
| Ossa FK, limiti, IK a 2 ossa/FABRIK, vincoli | ✅ | 🟢 | cinematica da manuale |
| Keyframe, easing, espressioni, noise | ✅ | 🟢 | progetto nostro |
| Editor a patch-op con undo/redo esatto | ✅ | 🟢 | puntatori JSON in stile RFC 6902 |
| Export SVG, PNG, GIF, player HTML, Lottie | ✅ | 🟢 | specifiche pubbliche |
| Log di provenienza, `stats` / `diff` semantico, regressione con fotogrammi di riferimento (hash esatto + tolleranza), limiti di risorse, test di mutation-fuzz, controllo della versione del formato file (R-001…R-005) | ✅ | 🟢 | progetto nostro |
| Vocabolario standard proprio dei ruoli + tagger basato sui nomi (`tag`) (R-006) | ✅ | 🟢 | nomi nostri, semplici parole di anatomia |
| Export JSON Schema (schema R-003) | ⬜ | 🟢 | schema nostro |

## Livello 2 — Basi dell'authoring e output aperti (giorni)
| Funzionalità | Stato | IP | Base sicura / condizione |
|---|---|---|---|
| APNG, sprite sheet PNG+JSON | ✅ | 🟢 | specifiche PNG/APNG |
| WebM VP9 / AV1 con alpha (tramite il tuo ffmpeg) | ✅ | 🟢 | codec con specifica aperta; stato dei brevetti non valutato |
| Export OCA (Open Cel Animation) | ✅ | 🟢 | specifica pubblica; import ⬜ |
| Import OpenRaster (.ora) | ✅ | 🟢 | specifica aperta, limiti anti zip-bomb |
| MP4 / H.264 | ✅ | 🟠 | solo tramite l'ffmpeg *dell'utente*; non includere mai un encoder |
| Shift/scala/inversione/copia/incolla delle chiavi, easing in batch, offset temporale per canale, marker e regioni, timecode SMPTE (R-010, R-011, R-044) | ✅ | 🟢 | editing generico |
| Snapshot di revisione con potatura e ripristino dopo crash (R-014); layer comp (R-015); eventi di timeline con payload (R-045); cambio di parent con conservazione della posa di riposo (parte di R-013) | ✅ | 🟢 | editing generico |
| Allinea/distribuisci, livello di riferimento, tag shy/solo/lock (R-012, R-013, R-016) | ✅ | 🟢 | editing generico |
| Onion skin, motion trail (overlay mai esportati) (R-020) | ✅ | 🟢 | progetto nostro |
| Catena di post-effetti: blur, drop shadow, glow, outline, posterize, pixelate, levels/gamma, fade (parte di R-022) | ✅ | 🟢 | elaborazione delle immagini da manuale |
| Trim path, filtri per livello, maschere, profondità di campo (R-021, R-024, R-025, R-034) | ✅ (trim path pre-calcolati; limiti degli effetti di livello: vedi ROADMAP §9) | 🟢 | matematica standard delle immagini |
| Import SVG (percorsi, forme, gruppi, trasformazioni, riempimenti pieni; gradienti/testo/clip segnalati come avvisi) (R-032) | ✅ | 🟢 | grammatica W3C SVG 1.1 |
| Import BVH (un asse di rotazione → angoli delle ossa, riduzione limitata delle chiavi) (R-036) | ✅ | 🟢 | formato testuale aperto |
| Export contenitore dotLottie (R-03A) | ✅ | 🟢 | specifica pubblica del contenitore (zip + manifest) |
| Export SVG animato, import di sequenze di immagini, output OpenEXR/16 bit, export glTF (R-032, R-037…R-039) | 🟡 (glTF: forme del frame 0 più ossa e pesi di morph, vedi ROADMAP R-039) | 🟢 | formati pubblici |
| Import di Papagayo (Moho-switch) e dei mouth cue di Rhubarb (R-03B) | ✅ | 🟢 | si legge solo il loro output in testo semplice |

## Livello 3 — Profondità di rig e animazione (settimane)
| Funzionalità | Stato | IP | Base sicura / condizione |
|---|---|---|---|
| Molle (pre-calcolate), lattice (FFD), deformatore spine, livelli switch, particelle, liquidi, booleane | ✅ | 🟢 | da manuale |
| Lip-sync da audio → forme della bocca, generatore di walk cycle, specchio, retime, clip, crossfade | ✅ | 🟢 | progetto nostro |
| **Pin warp** (sostituisce il warp in stile cage / ARAP) | ✅ | 🟢 | RBF di Wendland 1995 |
| **Pesi di auto-skin per diffusione del calore** | ✅ | 🟢 | equazione di Laplace |
| **Registrazione della performance** con riduzione limitata delle chiavi | ✅ | 🟢 | Douglas–Peucker |
| **Input mapping** (segnali CSV/JSON → parametri) | ✅ | 🟢 | nessun tracker incluso |
| IK soft/stretch (R-051), driver di posa: slider + joystick 2D (R-050), cambio di vista guidato dall'angolo (R-056), auto-rig da tag (R-058) | ✅ | 🟢 | rigging generico; matematica da manuale, progetto nostro |
| Vincoli di percorso, IK con blocco del piede, chiavi di cambio parent, skin, magneti (R-040…R-049, R-052…R-055, R-057) | 🟡 (realizzati: R-040…R-043, R-044, R-047, R-048, R-052…R-055, R-057; vedi ROADMAP sezione 9 per le parti segnate NOT done) | 🟢 | rigging generico |
| Generatori di idle (ammiccamento, respiro, oscillazione) e catene di follow-through (R-047, R-048) | ✅ | 🟢 | con seed, deterministici |
| Vento, gravità, collisioni, strisce per capelli/stoffa (R-070…R-072) | 🟡 (R-070, R-071 fatti; le strisce R-072 sono solo ossa, la stoffa è un foglio planare) | 🟢 | manuali di mass-spring / PBD |

## Livello 4 — Strumenti intelligenti (da settimane a mesi)
| Funzionalità | Stato | IP | Base sicura / condizione |
|---|---|---|---|
| **Traccia immagine → vettore** (sostituisce l'auto-trace basato su ML) | ✅ | 🟢 | marching squares + RDP |
| **State machine a log di eventi** + bake in canali (sostituisce i cloni di state machine di runtime) | ✅ | 🟢 | semantica nostra — mantenere i nostri nomi |
| **Retargeting a raggiungimento dell'effettore** | ✅ | 🟢 | metodo nostro |
| **Solver di pose-match** (strategia evolutiva, attento ai limiti dei giunti) | ✅ | 🟢 | Beyer & Schwefel |
| **Controllo flash WCAG 2.3.1 / flash rosso + rapporto sul movimento della camera + gemello con movimento ridotto** | ✅ | 🟠 | solo definizioni pubbliche; dire "ausilio tecnico", mai "certificato" |
| **Rig linter**, **contratti di movimento**, **solver di loop senza giunture**, **coach dei principi** (I-02, I-04, I-05, I-14) | ✅ | 🟢 | idee nostre |
| **Fitting della molla "disegna l'overshoot"** (I-08) — adatta frequenza/smorzamento con l'integratore del motore stesso | ✅ | 🟢 | idea nostra |
| **Explain-this-pixel** per ossa e livelli (I-07) | ✅ | 🟢 | idea nostra |
| Prima | dopo | immagine di revisione delle differenze (primo passo di I-06; la metrica percettiva è `ci diff`, SSIM e CIEDE2000) | ✅ | 🟢 | idea nostra |
| Illuminazione con normal map, parallasse, profondità di campo (R-064, R-065, R-024) | ✅ | 🟢 | da manuale |
| Lip-sync allineato alla trascrizione (R-081) | ✅ (allineatore proprio, `speech.rs`) | 🟠 | usare un allineatore con licenza permissiva; verificare la licenza del suo modello |
| Tracking di volto/corpo da webcam che alimenta l'input mapper (R-082) | 🟡 (rilevatore proprio addestrato solo su volti sintetici; webcam reali e fotografie non testate) | 🟠 | l'utente porta un tracker permissivo; non includere pesi di modelli salvo che la licenza lo consenta |
| Retargeting completo di mocap BVH con auto-match per gerarchia (R-083) | ✅ | 🟢 | si basa sull'import BVH |
| Import PSD (R-035) | ⬜ | 🟠 | sola lettura, specifica pubblicata, nessun marchio Adobe; oppure usare ORA |

## Livello 5 — Piattaforma (mesi)
| Funzionalità | Stato | IP | Base sicura / condizione |
|---|---|---|---|
| Studio nel browser con undo/redo | 🟡 | 🟢 | UI nostra; nessuna copia di layout o nomi di fornitori |
| Runtime WASM, API Rust stabile + C ABI, scripting in sandbox (R-090…R-093) | 🟡 (R-091, R-092, R-093 fatti; R-090 il modulo wasm viene compilato e supera in CI il test del wrapper sotto Node, ed è stato eseguito solo in Chromium headless) | 🟢 | API nostra |
| **Server Agent / MCP** su patch op: ispezione, modifica (atomica, annullabile), lint, sicurezza, render, salvataggio solo nel proprio file (R-094) | ✅ | 🟢 | protocollo JSON-RPC aperto |
| GUI completa: gizmo, timeline, graph editor (R-100…R-103) | 🟡 (R-101, R-102 fatti; le pagine R-100 e R-103 esistono ma l'autore non le ha mai eseguite in un vero browser) | 🟢 | widget generici; aspetto nostro |
| Controllo di versione dell'animazione con merge semantico, diff percettivo per CI, explain-this-pixel (I-01, I-06, I-07) | ✅ (I-01 `osatura vcs`, I-06 `osatura ci`, I-07 `explain`) | 🟢 | idee nostre |
| Pose fuzzing con heat-map di stress, export con dimensione obiettivo, livello di dettaglio deterministico con limite d'errore dichiarato (I-03, I-10, I-11) | 🟡 (tutti e tre realizzati; il limite di I-11 non copre effetti, percorsi con skinning, valori guidati da IK o pose da state machine) | 🟢 | idee nostre, matematica da manuale |

## Livello 6 — Frontiera (livello ricerca)
| Funzionalità | Stato | IP | Base sicura / condizione |
|---|---|---|---|
| Renderer GPU con test di parità con la CPU (R-110) | 🟡 (crate `osatura-gpu` realizzato e testato; esecuzione su hardware limitata, vedi ROADMAP) | 🟢 | wgpu (permissiva) |
| Editing multiutente in tempo reale sull'op-log (R-111) | 🟡 (fatto con un log di sessione persistente opzionale `--collab-log`; partecipanti e cronologia undo non sono persistenti; HTTP semplice, token condiviso) | 🟢 | letteratura sui CRDT |
| Runtime di data-binding / layout (R-091, R-095) | ✅ (progetto nostro; verifica dei brevetti non eseguita, docs/PATENT_NOTES.md) | 🟠 | verificare i brevetti prima della distribuzione |
| Strumenti assistiti da ML (R-112) | 🟡 | 🟠 | solo metodi classici, nessun peso di modello: `autotrace` (k-means, fit di Bézier di Schneider, modalità linea Zhang-Suen) e suggerimenti appresi dalle tue pose (PCA, kernel, piccola MLP opzionale in un formato di pesi proprio); nessun deep learning |

## Da non realizzare mai (🔴)
- Leggere o scrivere file di progetto nativi di **Live2D, Spine, Moho, Harmony, Rive, Cartoon Animator** (proprietari; esposizione a EULA/reverse engineering) — usare invece Lottie, OCA, ORA, APNG, SVG.
- **Solver veloci di deformazione di artwork in stile ARAP** come descritti nei brevetti Adobe — pin warp, FFD e skinning coprono il caso d'uso.
- Copiare di qualsiasi fornitore **layout di UI, icone, nomi, contenuti di esempio o numeri magici/terminologia dei formati file**.
- Imitare lo strumento di fotosensibilità **Harding** o dichiarare la conformità broadcast.
- Includere encoder **H.264/H.265/AAC**.

## Condizioni valide per ogni riga 🟢
Log di provenienza mantenuto (`CLEANROOM.md`; implementazione indipendente, non un clean-room rigoroso); solo dipendenze permissive (rigenerare con `scripts/licence-audit.sh`; attualmente 363 crate di terzi su tutte le piattaforme, tutti di classe MIT / Apache-2.0 / BSD / Zlib / Unlicense tranne sei crate MPL-2.0 (smartstring tramite la feature `scripting`, attiva per impostazione predefinita, e cinque tramite l'app a finestra `osatura-gui`); vedi docs/THIRD_PARTY.md); nessuna grafica o audio di terzi distribuiti senza il proprio file di licenza; il nome **OSATURA** verificato con una ricerca di marchio prima del lancio.


# Tabelle dettagliate delle capacità
## Disegno e forme
| Funzionalità | Stato | Note |
|---|---|---|
| Percorsi di Bézier cubici, aperti/chiusi, maniglie | ✅ | |
| Riempimento: pieno, gradiente lineare e radiale | ✅ | i gradienti seguono la trasformazione del livello |
| Tratto: larghezza, terminazioni, giunzioni, tratteggi | ✅ | |
| Tratto a larghezza variabile (profilo di larghezza) | ✅ | contorno poligonale; percorsi chiusi → due anelli |
| Smussatura Catmull-Rom, helper per rettangolo/cerchio/ellisse/poligono | ✅ | |
| Booleane tra poligoni (unione/intersezione/differenza/xor) | ✅ | livello `boolean`; i buchi sono collegati in un unico poligono; il risultato è poligonale (appiattito) |
| Livelli metaball / liquidi | ✅ | marching squares, le ossa possono guidare i blob |
| Livelli immagine (PNG), incorporati o su file | ✅ | |
| Pennelli bitmap/vettoriali, riempimenti con texture, testo/font | ❌ | |
| Disegno a mano libera, strumenti di modifica dei punti, traccia immagine | ❌ | nessuna GUI di disegno |

## Rigging
| Funzionalità | Stato | Note |
|---|---|---|
| Gerarchia di ossa, FK, posa di riposo | ✅ | |
| Limiti di angolo / scala | ✅ | |
| IK: analitica a 2 ossa (piegamento ±) | ✅ | |
| IK: catene di N ossa (FABRIK) | ✅ | |
| Mix FK↔IK (animabile) | ✅ | |
| Vincoli: aim, copy-rotation, pin (peso animabile) | ✅ | |
| Skinning a blend lineare di ancore + maniglie, auto-skin | ✅ | |
| Livelli collegati alle ossa, parenting / gruppi di livelli | ✅ | |
| Movimento secondario: molle smorzate pre-calcolate | ✅ | deterministico, ad accesso casuale (pre-calcolato al caricamento) |
| Canali guidati dalla posa ("driven keys") | 🟡 | una curva mappa qualsiasi proprietà su un'altra; nessuna UI/azioni dedicate per smart bone |
| Fisica di corpi rigidi, stoffa, soft-body | ❌ | |

## Deformazione
| Funzionalità | Stato | Note |
|---|---|---|
| Offset per punto (animabili) | ✅ | indirizzati come `layer::path::index` |
| Morph target / blend shape (pesi additivi) | ✅ | |
| Lattice a forma libera (FFD di Bernstein) | ✅ | punti di controllo animabili |
| Deformatore di piegatura spine / curva | ✅ | raggio morbido opzionale |
| Oscillazione "boil" (jitter disegnato a mano) | ✅ | noise coerente, deterministico |
| Warp di mesh d'immagine con pesi delle ossa | ✅ | mesh a griglia; nessun editor di mesh |

## Animazione
| Funzionalità | Stato | Note |
|---|---|---|
| Keyframe + easing (lineare/step/ease/cubic-bezier) | ✅ | |
| Estrapolazione: hold/lineare/loop/ping-pong/loop-offset | ✅ | |
| Canali di espressione | ✅ | piccolo linguaggio di espressioni sicuro; può leggere altri canali |
| Canali di noise, blend add/multiply/replace, pesi | ✅ | |
| Livelli switch + exposure (fotogramma per fotogramma, cambi di bocca) | ✅ | |
| Retime/offset di clip, retarget, specchio S↔D, bake del crossfade | ✅ | funzioni + CLI `mirror`; nessuna UI NLA |
| Walk cycle procedurale | ✅ | semplice generatore sinusoidale, non uno strumento professionale per la camminata |
| Lip-sync audio da WAV | 🟡 | euristica di volume/luminosità → 5 forme della bocca, **non** riconoscimento dei fonemi |
| Emettitore di particelle deterministico | ✅ | in forma chiusa: qualsiasi fotogramma è valutato direttamente |
| Onion skin, motion trail | ✅ | CLI `onion` e `trail`, overlay dello studio; mai esportati |
| Riproduzione audio / scrubbing della forma d'onda | 🟡 | comando `waveform` e riproduzione audio nello studio; nessuna dichiarazione su una UI di scrubbing della forma d'onda |

## Camera e compositing
| Funzionalità | Stato | Note |
|---|---|---|
| Camera 2D pan/zoom/rotazione (animabile) | ✅ | |
| Parallasse multipiano per livello di primo livello | ✅ | |
| Opacità, modalità di fusione, maschere alpha (+inversione) | ✅ | |
| Veri livelli / camera 3D, stereo | ❌ | |

## Output e interscambio
| Funzionalità | Stato | Note |
|---|---|---|
| Fotogrammi SVG | ✅ | |
| Fotogrammi PNG (raster tiny-skia) | ✅ | |
| GIF animata | ✅ | |
| MP4 | ✅ | richiede `ffmpeg` nel PATH |
| Player HTML autonomo | ✅ | tutti i fotogrammi inline, scrubber |
| Lottie JSON | 🟡 | geometria pre-calcolata per fotogramma; gradienti → primo colore; niente maschere/modalità di fusione/immagini; **non validato in un vero player Lottie** |
| Import di file di altri strumenti, PSD, FBX, ecc. | ❌ | (deliberatamente: nessun formato proprietario) |
| API di scripting / plugin | 🟡 | `script` Rhai in sandbox tramite richieste validate, più espressioni; nessuna API di plugin |

## Editing
| Funzionalità | Stato | Note |
|---|---|---|
| Modello di modifica JSON-patch con undo/redo | ✅ | ogni modifica è validata; le modifiche non valide vengono annullate |
| Helper per impostare/eliminare keyframe | ✅ | |
| Studio web (browser) | 🟡 | anteprima, scrubber, keying dell'angolo per osso, editor JSON, undo/redo, salvataggio, export. **Nessun disegno, nessuna modifica di punti/ossa sul canvas, nessun graph editor.** |

## Cose che sono modesti "miglioramenti" rispetto alla base da manuale
(non si dichiara che manchino altrove)
- Ogni modifica al progetto è una patch JSON validata e invertibile → l'undo non può mai corrompere un progetto.
- Tutta la simulazione (molle, particelle) è deterministica ad accesso casuale: puoi fare scrubbing o renderizzare qualsiasi fotogramma in qualsiasi ordine e ottenere pixel identici.
- Qualsiasi proprietà — inclusi i punti di controllo dei deformatori e i pesi di morph — è indirizzabile con un unico percorso `layer::sub::id`, così espressioni e curve guidate possono collegare qualsiasi cosa a qualsiasi cosa.
- La validazione segnala *tutti* i problemi in un solo passaggio con i nomi coinvolti.
- Un solo motore → SVG, raster, GIF, MP4, HTML, Lottie; la stessa `evaluate(frame)` alimenta tutti.

## Test
`cargo test` — 142 test unitari/di integrazione (matematica, track, espressioni, solver di rig, deformatori,
pixel di rendering, esportatori, undo/redo dell'editor, audio, particelle, booleane, metaball).

---

# Checklist rispetto al prodotto di riferimento, versione per versione

Riferimento: Moho (già Anime Studio), 1999 → 14.5 "SHOWTIME" (12 ott 2026). Le liste di funzionalità
provengono dalle pagine di rilascio del fornitore e dalla stampa di settore (vedi Fonti). Lacune nella documentazione: Anime Studio 6–8,
12.5 e 14.1 non avevano una lista pubblica dettagliata che io sia riuscito a trovare, quindi sono coperti solo dalla
storia generale. Lo stato è quello di OSATURA: ✅ esiste un equivalente · 🟡 parziale / approccio diverso · ❌ mancante.

## Prime versioni (1999 – Anime Studio 8, solo storia generale)
| Funzionalità | OSATURA |
|---|---|
| Particelle | ✅ livello emettitore deterministico |
| Maschere di livello | ✅ maschere alpha + inversione |
| Onion skin | ✅ (CLI `onion`, overlay dello studio) |
| Vincoli delle ossa | ✅ aim / copy-rotation / pin, limiti di angolo e scala |
| Scripting Lua | 🟡 (Rhai, non Lua; anche canali di espressione) |
| Lip-sync | 🟡 euristica di ampiezza da WAV |
| Fisica | 🟡 molle + gravità delle particelle; nessun corpo rigido |
| Creazione / import 3D | ❌ |
| Import Photoshop | ❌ |
| Supporto Wacom | ❌ (nessuna GUI di disegno) |
| Smart bone, IK | 🟡 canali guidati / ✅ 2 ossa + FABRIK |

## Anime Studio 9 (2013)
| Funzionalità | OSATURA |
|---|---|
| 64 bit, accelerazione GPU | 🟡 Rust nativo a 64 bit; solo rasterizzatore CPU (nessuna GPU) |
| Smart Bone (quadranti di controllo della flessione dei giunti) | 🟡 i canali guidati mappano qualsiasi valore su qualsiasi altro; nessuna UI/azioni a quadrante |
| Grafici di movimento modificabili | ❌ (le curve sono dati con easing cubic-bezier; nessun graph editor) |
| Timeline rinnovata | ❌ (lo studio ha solo uno scrubber + segni delle chiavi) |
| Multi-touch Wacom | ❌ |

## Anime Studio 10 (2014)
| Funzionalità | OSATURA |
|---|---|
| Vincoli di angolo indipendenti (globali) | 🟡 la copy-rotation su un osso di controllo li approssima |
| Squash & stretch | 🟡 la scala dell'osso è animabile; nessun squash/stretch automatico |
| Quadranti / setup degli Smart Bone | 🟡 vedi sopra |
| Interpolazione bounce / elastic / stagger | 🟡 bounce, spring (simile a elastic), back, cubic-bezier ✅; stagger ❌ |
| Secchiello, pennello blob, gomma, riduzione dei punti | ❌ |
| Personaggi precaricati / contenuti di libreria | ❌ |
| Import di oggetti 3D | ❌ |

## Anime Studio 11 (2015)
| Funzionalità | OSATURA |
|---|---|
| Animazione fotogramma per fotogramma | ✅ livelli switch + canali di exposure |
| Riferimento tra livelli (duplicati collegati) | ❌ |
| Ordinamento animato delle forme | ❌ (l'ordine di disegno è fisso per livello; i livelli switch possono scambiare) |
| Curve di movimento modificabili | 🟡 dati delle curve sì, nessuna UI per le curve |
| Import PSD (incluso aggiornamento live) | ❌ |
| Export FBX / Unity | ❌ |
| Import/registrazione audio + lip-sync automatico | 🟡 import WAV + lip-sync euristico; nessuna registrazione |
| Unione dei tratti, strumento Color Points | ❌ |

## Moho 12 (2016) e 12.x
| Funzionalità | OSATURA |
|---|---|
| Maniglie di Bézier | ✅ |
| Strumenti a mano libera migliorati | ❌ |
| Smart Warp (mesh personalizzate) | 🟡 mesh d'immagine a griglia con pesi delle ossa; deformatori lattice / spine per i vettori; nessun editor di mesh |
| Motion blur | ❌ |
| Pin bone | 🟡 vincolo pin, non pin bone a punto singolo |
| Azioni di import/export tra rig simili | 🟡 funzioni `clip`, `retarget`, `mirror` su insiemi di canali; nessuna UI di libreria di azioni |
| Animare più livelli contemporaneamente | ✅ qualsiasi numero di canali per timeline |
| UI ridisegnata, ancoraggio delle ossa, auto-freeze della posizione | ❌ |

## Moho 13 (2019)
| Funzionalità | OSATURA |
|---|---|
| Bitmap fotogramma per fotogramma + onion skin | 🟡 livelli immagine PNG dentro livelli switch; nessun onion skin, nessun disegno bitmap |
| Strumenti di disegno bitmap / pennelli personalizzati | ❌ |
| Strumenti di storyboard e animatic | ❌ |
| Finestra Actions | ❌ (solo clip a livello di dati) |
| Luci 3D, materiali, toon shading | ❌ |
| Più preset di output | 🟡 formati di export da CLI, nessun gestore di preset |
| Personaggi inclusi | ❌ |

## Moho 13.5 (2021) e 13.5.x
| Funzionalità | OSATURA |
|---|---|
| Vitruvian Bones (scambio di disegni *e* di insiemi di ossa) | 🟡 i livelli switch scambiano i disegni; nessuno scambio di insiemi di ossa |
| Quad mesh (prospettiva) | 🟡 mesh a griglia + lattice danno warp a forma libera; nessun quad proiettivo |
| Dinamica del vento (forza, direzione, turbolenza) | ❌ (le molle non hanno vento; i canali di noise possono simularlo) |
| Keyframing relativo | ❌ |
| Varianti di visemi del lip-sync scelte a caso | ❌ (il mapping può unire forme, nessuna variante casuale) |
| Export dell'audio come WAV | ❌ |
| Copia/incolla di deep keyframe, regolazione del pivot, marker di timeline | ❌ (nessuna GUI) |
| Licenze floating | n/a |

## Moho 14 (2023)
| Funzionalità | OSATURA |
|---|---|
| Nuovo motore grafico, effetti in tempo reale (blur, ombra, noise) | ❌ nessun filtro blur/ombra/noise; modalità di fusione ✅ |
| Liquid Shapes (unione/sottrazione/intersezione/fusione) | 🟡 liquido metaball ✅ + booleane tra poligoni ✅; non è lo stesso modello di forma |
| Curvers / Compressible Curvers | 🟡 il deformatore di piegatura spine è l'analogo più vicino |
| Import SVG migliorato | ❌ (solo **export** SVG) |
| Miglioramenti a mano libera, nuovi stili, unione di forme | ❌ / ❌ / 🟡 (booleane) |
| Multi-Stroke Fill, Connect & Create | ❌ |
| Smart Line Boil, opzioni del pennello | 🟡 oscillazione "boil" con frequenza regolabile ✅; opzioni del pennello ❌ |
| Colori dei punti, Add Point migliorato | ❌ |
| Dynamic Bones (+ gravità) | 🟡 ossa a molla smorzata ✅; nessun termine di gravità |
| Vincoli ritardati | 🟡 le molle producono ritardo rispetto a un bersaglio; nessun vincolo di ritardo esplicito |
| Nuove opzioni di timeline | ❌ |
| Miglioramenti fotogramma per fotogramma | 🟡 |
| Aggiornamento dell'interfaccia di scripting (Lua 5.4) | ❌ |

## Moho 14.1 – 14.2 (2023-11, 2024-06)
| Funzionalità | OSATURA |
|---|---|
| (14.1: nessun dettaglio pubblico trovato) | — |
| Offset animato del profilo della curva | ❌ (i profili di larghezza sono statici) |
| Smart boil per il noise vettoriale | 🟡 l'oscillazione scatta a una frequenza impostata; nessuna modalità "solo durante il movimento" |
| Strumento Transform Bone rielaborato | ❌ (nessuno strumento su canvas) |
| Visualizzazione pixelata | ❌ |
| Modalità di fusione Colour Burn | ✅ |
| Spaziatura minima del pennello ridotta, livelli ritagliati PSD | ❌ |
| Numerazione della timeline basata sul frame rate | ❌ |
| Opzione di visualizzazione "Turn off masked layers" | ❌ |
| Lip-sync migliore a frame rate bassi | 🟡 |
| Nomi multilingue per ossa/livelli/marker | ✅ (nomi UTF-8) |
| Anti-aliasing della maschera Exclude-Strokes, onion skin colorati sulle immagini | ❌ |
| Smart Bone che controllano la dinamica delle ossa | ❌ (le impostazioni delle molle non sono animabili) |
| Script Lua all'avvio; flag della console di Windows | ❌ / n/a |
| Export ProRes alpha | ❌ (solo GIF/PNG/MP4) |

## Moho 14.3 (2024-12)
| Funzionalità | OSATURA |
|---|---|
| Opacità del pennello per punto (guidata dalla pressione) | ❌ |
| Deriva del colore in base alla pressione della penna | ❌ |
| Controlli dello strumento a mano libera / larghezza della linea | ❌ |

## Moho 14.4 "SIDE QUEST" (2025-11)
| Funzionalità | OSATURA |
|---|---|
| Export verso Unreal / Unity / Godot | ❌ |
| Export glTF verso Blender / strumenti 3D | 🟡 (`.glb`: forme del frame 0, ossa, pesi di morph) |
| Smart Bone a 360° | ❌ |
| Create Style from Shape | ❌ |
| Opzioni delle particelle (replica lungo la curva, ritardo, emissione ordinata, distribuzione) | 🟡 ritardo di avvio ✅, emissione ordinata deterministica ✅; replica lungo la curva ❌ |
| Layer Selector, decadimento del Magnet, import PSD, smussatura a mano libera, aggiunte Lua | ❌ |
| Creazione più semplice dei cicli | ✅ estrapolazione in loop + generatore di walk cycle |

## Moho 14.5 "SHOWTIME" (2026-10-12)
| Funzionalità | OSATURA |
|---|---|
| Live Playback (webcam / gamepad / microfono, webcam virtuale, OBS) | ❌ |
| Face tracking → ossa / Smart Bone / mesh | 🟡 (rilevatore addestrato su dati sintetici, `face-bake`; non validato su volti reali) |
| Registrazione live sulla timeline, controllo di più personaggi | ❌ |
| Video tracking (sequenze di immagini) | ❌ |
| Algoritmi a mano libera, linee rette, taglio delle code | ❌ |
| Secchiello rifatto (Vector / Trace / Recolor) | ❌ |
| Movimento inverso del Magnet, selezione dei punti per forma | ❌ |
| Categorie di pennelli personalizzate | ❌ |
| Colore/opacità dell'onion skin, Shift & Trace, onion skin magnetici | ❌ |
| Filtri / selezione multipla nella finestra Layers | ❌ |
| Prestazioni: undo/redo più veloce, file più leggeri | 🟡 l'undo è un'inversa economica della patch JSON; i file sono JSON semplice |
| Export MP4/H.264 | ✅ tramite ffmpeg |
| Profili colore PNG, import/export EXR | ❌ |
| Batch exporter, flag CLI `-exr` / `-pngprofile` | 🟡 CLI `frames` / `export`, nessuna coda batch |
| Frame rate con 3 decimali (es. 23.976) | ✅ |
| L'export SVG gestisce tratti a larghezza variabile e forme liquide | ✅ (i tratti diventano contorni, i liquidi diventano percorsi) |
| Liquid hide-edge, forme liquide solo tratto, exposure del tratto | ❌ |
| Linux tramite Wine/Proton | ✅ Rust nativo (Linux/macOS/Windows); archivi precompilati per Windows x86-64, macOS arm64 e Linux x86-64 nella pre-release in bozza; testato dalla CI su tutti e tre, da una persona solo su Linux |

## Riepilogo (solo prodotto di riferimento; i concorrenti seguono sotto)
Tra le funzionalità documentate del prodotto di riferimento, OSATURA copre il nucleo di **rigging, deformazione,
animazione procedurale, matematica del compositing ed export** (all'incirca: scheletro/IK/vincoli,
deformazione di mesh e lattice, keyframe ed espressioni, maschere/fusioni/gradienti, particelle,
metaball/booleane, GIF/MP4/HTML/Lottie), ma **non ha strumenti di disegno, input dal vivo, 3D,
I/O PSD/FBX** (al momento della prima analisi; da allora sono stati aggiunti export glTF, output EXR, scripting Rhai, uno studio con graph editor/timeline, onion skin ed effetti di blur, vedi `ROADMAP.md` sezione 9).
Sono i blocchi grandi, per lo più legati alla GUI, del prodotto e rappresentano la lacuna onesta.

## Fonti
- Annuncio di Moho 14.5 SHOWTIME, moho.lostmarble.com (blog, 12 ott 2026)
- Pagina delle funzionalità di Moho Pro, mohoanimation.com/pages/features
- Appcast di Moho 14.4 SIDE QUEST, lostmarble.com/appcasts/Moho14.4Update.html
- Moho 14.3: Digital Production, 27 dic 2024
- Post di rilascio di Moho 14.2, moho.lostmarble.com; elenco dei tutorial "Moho 14: What is new?"
- Novità di Moho Pro 13.5, moho.lostmarble.com; Lester Banks (apr 2021); note di 13.5.3
- Moho 13: Lester Banks (giu 2019)
- Moho Pro 12: 80.lv; Digital Production (nov 2016)
- Anime Studio 11: Architosh (mag 2015); 10: Architosh (mar 2014); 9: Renderosity
- Date delle versioni: mohoscripting.com/moho_versions; storia generale: Wikipedia "Moho (software)"

---

# Addendum: storia di Moho / Anime Studio, versioni 1 – 11 (completata)

Fonte: il PDF della cronologia delle versioni ospitato dal fornitore (lostmarble.com, "Anime Studio history", 2015) più le
pagine stampa di Anime Studio 7 e 8. Sostituisce le righe "solo storia generale" in cima alla
checklist di riferimento. Non trovati nemmeno qui: dettagli di 8.x, 10.1.x, 11.1, 12.5, 14.1.

| Versione | Funzionalità documentate | OSATURA |
|---|---|---|
| Moho 1 (1999) | BeOS, port per Windows, output Flash | ❌ Flash/SWF (obsoleto) |
| 2.x | I/O QuickTime, render fino a 4096², shading dei livelli, livelli switch, campioni di colore negli stili, binding flessibile delle ossa, import Illustrator migliorato | ✅ livelli switch, 🟡 binding dei pesi, ❌ import Illustrator, ✅ grandi scale raster (≤16×) |
| 3.x | Particelle, image wrapping, maschere di livello, onion skin, camera virtuale con profondità, image warping, livelli switch avanzati, supporto tavoletta, timecode SMPTE, nascondere l'artwork fuori schermo, maschere reversibili, randomizzatore dello spessore delle linee, safe zone, shear dei livelli | ✅ particelle, maschere (+inversione), camera, warp di mesh d'immagine, skew dei livelli; ❌ onion skin, tavoletta, timecode, safe zone, randomizzatore dello spessore |
| 4.x | Curve di animazione grafiche, vincoli delle ossa, rendering in batch, audio, parametri animabili, cicli di animazione, scala delle chiavi, progetti ridimensionabili, profondità delle particelle, folle randomizzate, flexible binding automatico, versione Linux | ✅ vincoli, cicli (estrapolazione in loop), parametri animabili, progetti ridimensionabili, retime `clip` ≈ scala delle chiavi, Linux; 🟡 folle (particelle), 🟡 audio (solo lip-sync); ❌ UI dell'editor di curve |
| 5.x | Nuova UI, export SWF, livelli 3D e di note, azioni, stili, scripting Lua | ❌ livelli 3D / di note / Lua; 🟡 azioni (funzioni clip); ✅ gli stili sono colore per percorso |
| Anime Studio 6 | Motion tracking, lip-sync integrato, scatter brush, Lua 5.1, auto shading, **morph target**, sequencer, video HD | ✅ morph target; 🟡 lip-sync; ❌ motion tracking, scatter brush, auto shading, sequencer |
| 6.1 / 6.2 | Upload su Facebook, import di GIF trasparenti | ❌ / 🟡 export GIF ✅, import ❌ |
| Anime Studio 7 | Fisica simulata (gravità, densità, attrito, elasticità), progettazione di forme 3D, registrazione audio, libreria di contenuti, import di scene Poser, **strumento follow path**, gather media | 🟡 molle/gravità delle particelle; ❌ fisica rigida, 3D, registrazione, libreria, Poser, follow-path |
| Anime Studio 8 | Tracciamento automatico delle immagini, Character Wizard, modalità Beginner, connessione media in tempo reale, stili di rendering globali, rigging avanzato delle ossa | ❌ tutto (nessun tracing, wizard, collegamento live, stili globali) |
| Anime Studio 9 / 9.5 | Grafici di movimento modificabili, Smart Bone, **layer comp**, onion skin migliorati, 64 bit + GPU, keyframing avanzato, timeline colorata; 9.5: miglioramenti agli Smart Bone, depth shifting, controlli annidati dei livelli, flexi-binding, miglioramenti ai livelli switch | vedi tabella AS 9; ❌ layer comp, depth shifting |
| Anime Studio 10 / 10.1 | Nuovi vincoli, setup degli Smart Bone, bounce/elastic/stagger, processo di render separato, strumenti combinati ossa+punti, anteprima dell'animazione, multi-documento, editor delle scorciatoie, selezione di più forme, frame zero, nascondere punti/ossa, editing multi-livello, **curve a larghezza variabile**, trasparenza delle texture, sorgente di particelle regolabile, profondità di campo, blur frazionario; 10.1: auto image trace, stili globali, patch layer, import PSD, media in tempo reale, Character Wizard | ✅ tratti a larghezza variabile, ✅ animazione multi-livello; 🟡 profondità di campo ❌ blur ❌; ❌ patch layer, altri come sopra |
| Anime Studio 11 | Fotogramma per fotogramma, riferimento tra livelli, ordinamento animato delle forme, **target delle ossa animati**, **parenting delle ossa animato**, bone flipping, raggruppamento dei livelli di selezione, scala dei livelli normalizzata, export in batch, nuovo formato di file | ✅ FbF, ✅ target IK animato (il target è semplicemente un osso), ❌ *re-parenting* animato, 🟡 bone flipping (`mirror`), ✅ batch tramite CLI, ✅ formato di file JSON semplice |

**Nuove lacune emerse qui** (da aggiungere alla roadmap): layer comp (stati di visibilità dei livelli salvati),
re-parenting animato delle ossa, patch layer, depth shifting, livelli follow-path, overlay di safe zone,
timecode SMPTE, auto shading, scatter brush, auto-trace delle immagini.

---

# Funzionalità della concorrenza non coperte dalla checklist precedente

Prodotti analizzati: Toon Boom Harmony 24, Spine 4.2, Live2D Cubism 5.x, Cartoon Animator 5.3, Rive,
Synfig Studio, OpenToonz, TVPaint Animation 12, Adobe Animate, Cavalry, Procreate Dreams, Duik (After
Effects), Blender Grease Pencil. Le voci seguenti sono quelle **non già elencate** nella checklist
di riferimento. La qualità delle fonti varia: la maggior parte delle righe proviene da pagine di funzionalità/rilascio dei fornitori che ho scaricato;
le righe segnate † provengono da conoscenza generale o solo da un titolo/indice e non sono state confermate su una pagina.
Lo stato è quello di OSATURA: ✅ · 🟡 · ❌.

## Rigging, vincoli, deformazione
| Funzionalità | Vista in | OSATURA |
|---|---|---|
| Strumenti per i pesi: saldatura dei pesi tra mesh, copia/incolla dei pesi tra vertici | Spine 4.2 | ❌ (solo auto-skin, nessuna pittura) |
| Vincoli di fisica delle ossa (inerzia, vento, gravità, mix) | Spine 4.2 | 🟡 molle smorzate (frequenza/smorzamento); nessun vento/gravità/inerzia |
| Vincolo di percorso: le ossa seguono una spline di Bézier composta | Spine, Rive (Follow Path) | ❌ |
| Mix di vincoli, duplicazione con le ossa, multi-edit, cartelle | Spine 4.2 | 🟡 mix IK e peso dei vincoli animabili; nessuna cartella/multi-edit |
| Skin (scambio di insiemi di attachment su uno scheletro) | Spine, Rive solos | 🟡 livelli switch |
| "Pose tool" IK multi-osso (trascinare l'estremità di un arto) | Spine | ❌ (richiede una GUI con canvas) |
| Attachment bounding-box / hit-test | Spine | ❌ (non necessari per l'output offline) |
| Free-Form Deformation, pesi di mesh | Spine, Harmony | ✅ lattice + mesh d'immagine con pesi |
| Catena di deformatori: Bone, Game Bone, Curve, Envelope, Free-Form, Mesh Warp | Harmony | 🟡 bone ✅ curve (spine) ✅ free-form ✅ mesh warp ✅; envelope/cage ❌; game-bone ❌ |
| Shape-Aware Deformation (punti, ossa, cage) | Harmony 24 | ❌ |
| Bake di tutti i tipi di deformazione | Harmony 24 | 🟡 molle pre-calcolate; geometria pre-calcolata per fotogramma nell'export Lottie |
| Master controller / selettore di trasformazione per gruppi di deformazione | Harmony 24 | ❌ |
| Peg e gerarchia dei livelli | Harmony, Animate | ✅ gruppi, livelli collegati alle ossa, parenting |
| Auto-muscle, auto foot-roll, output cinematico † | Harmony | ❌ |
| Deformatori di rotazione / warp + parametri keyform, glue | Live2D | 🟡 parametri ≈ canali guidati, warp ≈ lattice; glue ❌ |
| Blend shape che guidano rotazione, opacità, ordine di disegno, colore multiply/screen | Live2D 5.0 | 🟡 i pesi di morph guidano solo la geometria |
| Auto-mesh con controllo uniforme della densità per forme sottili | Live2D 5.0 | ❌ (solo griglia regolare) |
| Il warp deformer si ridimensiona automaticamente per contenere il keyform figlio | Live2D 5.3 | ❌ |
| Pennello di deformazione (forme quadrata/linea/espansione) | Live2D 5.3 | ❌ |
| Deformatore del volto / generazione del movimento facciale assistiti da IA | Live2D 5.0 | ❌ |
| Joystick (controllo 2D mappato su animazioni) | Rive | 🟡 il canale guidato mappa un solo input; nessun joystick a 2 input |
| Vincoli tra oggetti arbitrari (traslazione/rotazione/scala/distanza) | Rive | 🟡 solo aim, copy-rotation, pin |
| Rigging bitmap con interpolazioni intermedie automatiche | TVPaint 12 | 🟡 mesh d'immagine ✅; interpolazioni automatiche ❌ |
| Strumento Plastic (rigging basato su mesh) | OpenToonz | 🟡 mesh d'immagine + pesi |
| Auto-rigging, controller, dinamica a molla | Duik † | 🟡 molle ✅; auto-rig ❌ |

## Animazione, timeline, interattività
| Funzionalità | Vista in | OSATURA |
|---|---|---|
| Animation mixing: blend/crossfade di animazioni concorrenti a runtime | Spine | 🟡 modalità Add/Multiply + pesi; `crossfade` pre-calcolato; nessuna transizione a runtime |
| Dopesheet + graph editor + anteprima a runtime | Spine, Rive, OpenToonz | ❌ (i dati lo supportano; nessuna UI) |
| State machine: input, listener, transizioni con condizioni, blend state, livelli | Rive | ❌ |
| Eventi, trigger audio, asset dinamici | Rive | ❌ |
| Data binding (elementi di design ↔ dati live) | Rive | ❌ (le espressioni possono leggere canali, non dati esterni) |
| Scripting incorporato (Luau, JavaScript, ToonzScript) | Rive, Cavalry, OpenToonz | ❌ |
| Layout responsive / componenti / librerie | Rive | ❌ |
| Trim Paths, feathering vettoriale, N-slicing | Rive | ❌ |
| Temporizzazione indipendente dalla risoluzione, tweening vettoriale | Synfig | ✅ |
| Collegamento di parametri tra livelli tramite espressioni | Synfig | ✅ |
| Grafo di nodi procedurale (generatori, trasformatori, effettori), duplicatori | Cavalry | ❌ (nessun grafo di nodi; nessuna istanza di livelli) |
| Animazione guidata dai dati da JSON/CSV | Cavalry | ❌ |
| Performing: registrare gesti live come keyframe | Procreate Dreams | ❌ |
| Hotkey di trigger per puppet-stage, motion pilot (camminate guidate dal cursore) | Cartoon Animator 5.3 | ❌ |
| Camera switcher: camere salvate, stacchi con hotkey, track di cambio camera | Cartoon Animator 5.3 | 🟡 una sola camera; gli stacchi si possono simulare con chiavi step |
| Movimenti cinematografici (dolly zoom, tracking di un bersaglio) | Cartoon Animator 5.3 | 🟡 camera animabile; nessun tracking del bersaglio |
| Mocap facciale (webcam / iPhone), corpo guidato dalla testa, volume del microfono | Cartoon Animator, Live2D | ❌ (volume del microfono solo tramite WAV offline) |
| Modalità livestream (OBS/Twitch, mapping Stream Deck) | Cartoon Animator 5.3 | ❌ |
| Lip-sync Motion Sync che genera chiavi di timeline | Live2D 5.0 | 🟡 WAV euristico → chiavi di exposure |
| Integrazione del lip-sync Rhubarb | OpenToonz | ❌ (solo euristico) |
| Estrazione del root motion (compensazione delta) | Spine 4.2 | 🟡 estrapolazione loop-offset |
| Sincronizzazione delle maniglie delle curve in loop | Spine 4.2 | ✅ estrapolazione in loop (nessuna UI per le maniglie) |
| Morphing con morph hint | Harmony | 🟡 morph target + lerp dei percorsi; nessun hint |
| Camera 2D basata su chiavi con temporizzazione sulla timeline | TVPaint 12 | ✅ |

## Disegno, pittura, strumenti bitmap
| Funzionalità | Vista in | OSATURA |
|---|---|---|
| Pennello: randomizzazione (numero di stampi, offset), doppia carta, pennelli multitexture | Harmony 24, TVPaint 12 | ❌ |
| Opacità della matita in base a pressione/inclinazione/velocità/rastremazione/dissolvenza | Harmony 24 | ❌ |
| Guide di simmetria (verticale, orizzontale, doppia) | Harmony 24 | ❌ |
| Strumento Cutter (lazo, trim), editor di contorni, close-gap | Harmony | ❌ |
| Guide di disegno, guide di prospettiva/righello † | Harmony | ❌ |
| Animazione senza carta: onion skin, flipping, tavolo luminoso | Harmony, Procreate Dreams, TVPaint | 🟡 (onion skin sì; nessuno strumento di disegno né tavolo luminoso) |
| Pulizia di disegni scansionati, scansione, acquisizione da camera, stop-motion | OpenToonz | ❌ |
| Pennello di selezione, libreria di immagini | TVPaint 12 | ❌ |
| Nitidezza durante la trasformazione | TVPaint 12 | n/a |
| Sensibilità alla pressione della penna | Synfig | ❌ |
| Ombreggiatura a gradiente basata su percorso | Synfig | ❌ |
| Testo con contenuto variabile/dinamico a runtime, in qualsiasi lingua | Rive, Procreate Dreams | ❌ |
| Font personalizzati, titoli, titoli di coda | Procreate Dreams | ❌ |
| Gestione di palette e stili | OpenToonz | ❌ |
| Oltre 50 tipi di livello (frattali, filtri, distorsioni) | Synfig | ❌ (≈7 tipi di livello) |

## Effetti e compositing
| Funzionalità | Vista in | OSATURA |
|---|---|---|
| Rete di compositing a vista nodi con porte nominate | Harmony 24 | ❌ (albero dei livelli + maschere/fusioni) |
| Matte, gruppi sotto composite | Harmony | ✅ maschere, gruppi isolati |
| Lettura/scrittura CryptoMatte | Harmony 24 | ❌ |
| Toon shader (outline, posterise) | Harmony 24 | ❌ |
| Unsharp mask, scambio di canali | Harmony 24 | ❌ |
| Famiglia di blur (radiale, spin, direzionale), bokeh, noise frattale, tile | OpenToonz | ❌ |
| Particles FX | OpenToonz | ✅ livello emettitore |
| 15 modalità di fusione RGB + 5 alpha, disegno offscreen per parte | Live2D 5.3 | 🟡 13 modalità di fusione; rendering di gruppi isolati ✅ |
| Clipping (ID di clip per parte) | Live2D, Rive | ✅ maschere alpha |
| Imaging HDR / HDRI | Synfig | ❌ |
| Import video, ProRes, rotoscoping su filmato | Procreate Dreams | ❌ |
| Motore audio (voice-over, musica, SFX) | Procreate Dreams | ❌ |

## 3D, pipeline, interscambio, runtime
| Funzionalità | Vista in | OSATURA |
|---|---|---|
| Scena 3D, posing di ossa FBX, scene graph Vulkan, disegno su piani prospettici | Harmony 24 | ❌ |
| Rendering esterno tramite Maya / Blender | Harmony 24 | ❌ |
| Import vettoriale: PDF / AI / SVG; EXR multilivello; DPX alpha; OpenColorIO (ACES) | Harmony 24 | ❌ |
| Import PSD senza Photoshop | Spine 4.2, Rive | ❌ |
| Import Lottie / incolla da Figma | Rive | ❌ (solo **export** Lottie) |
| Texture packer / atlanti | Spine | ❌ |
| Export dello scheletro JSON/binario + 19 runtime ufficiali / oltre 40 della community | Spine | 🟡 JSON proprio; nessun runtime |
| Runtime open source (Web, Apple, Android, React, Flutter, C++, Unity, Unreal) | Rive | ❌ |
| Modalità Pixels (allineare gli sprite ai pixel dello schermo) | Spine 4.2 | ❌ |
| Export: WebM con trasparenza | Live2D 5.3 | ❌ |
| Export: SVG/PDF, sequenze PNG/EXR, profondità 16/32 bit, batch, ProRes | Cavalry | 🟡 SVG, PNG ✅; PDF/EXR/ProRes/alta profondità di bit ❌ |
| Rendering in cloud, link di condivisione, embed | Rive | ❌ |
| Rendering distribuito (Toonz Farm), controllo di versione | OpenToonz | ❌ |
| Export PDF / TVPaint JSON | OpenToonz | ❌ |
| Collaborazione di team in tempo reale, remix della community | Rive | ❌ |
| Export HTML5 Canvas / web interattivo | Animate † | 🟡 player HTML (pre-renderizzato, non interattivo) |
| Linux/macOS/Windows nativi | Synfig, OpenToonz | ✅ |
| Licenza open source | Synfig, OpenToonz | ✅ MIT/Apache-2.0 |
| Vista del parenting dei livelli, istanze di simboli/libreria † | Animate | 🟡 parenting ✅; nessuna libreria di istanze |
| UI modulare staccabile, workspace personalizzati, tema scuro/chiaro | TVPaint 12, Live2D 5.0 | ❌ |

## Roadmap candidata (ordinata per valore ÷ sforzo, prima senza bisogno di GUI)
1. Trim path + path-following (ossa/livelli lungo una Bézier) — Rive, Spine
2. Overlay di onion skin + motion trail — quasi ogni strumento 2D
3. Import SVG (percorsi, gradienti) — Moho 14, Harmony, Synfig
4. Termini di vento / gravità per le molle, dinamica delle ossa guidata da input in stile Smart Bone
5. Blur, drop shadow, glow, outline, posterise (filtri di livello)
6. Istanze di livelli / duplicatori (clonare N lungo un percorso, con offset per copia)
7. State machine interattiva nel runtime HTML/JSON (input, transizioni, blend)
8. Deformatore envelope (cage), strumenti per copia/saldatura/smussatura dei pesi, auto-mesh a densità uniforme
9. Stacchi di camera / camere salvate, camera con tracking del bersaglio
10. Export glTF del rig; export EXR/PDF/WebM-alpha
11. Livelli di testo (richiede caricamento dei font) con path-following
12. GUI dello studio: modifica di punti/ossa su canvas, graph editor, onion skin

## Fonti aggiuntive (concorrenti)
- Note di rilascio di Toon Boom Harmony 24 e Premium Getting Started Guide (docs.toonboom.com)
- Spine 4.2 "The physics revolution" e "What is Spine?" (esotericsoftware.com)
- Live2D Cubism 5.0 e 5.3 "New Features" (docs.live2d.com)
- Cartoon Animator 5.3 feature highlights (discussions.reallusion.com)
- Pagina delle funzionalità di Rive (rive.app/features)
- Funzionalità di Synfig Studio (zoftwarehub.com); documentazione "What's New" di OpenToonz
- TVPaint Animation 12 (tvpaint.com); recensione di Cavalry (superrendersfarm.com)
- Adobe Animate "Layer parenting" (adobe.com/learn); Procreate Dreams (Cartoon Brew)
- Duik: pagina dei tag di Lester Banks (solo titolo †); Blender Grease Pencil: pagina del manuale non recuperabile (non coperto)

---

# Addendum — Cronologie delle versioni dei concorrenti (secondo giro, 2026-10-19)

Legenda come sopra: ✅ fatto, 🟡 parziale, ❌ non ancora. Sono elencate solo le funzionalità confermate su pagine di fornitori/progetti o sulla stampa. Non sono stati usati siti pirata; web.archive.org non è raggiungibile da questo ambiente, quindi alcune versioni più vecchie restano non documentate (vedi "Lacune note").

## Toon Boom Harmony 20 (note di rilascio del fornitore)
| Funzionalità | OSATURA |
|---|---|
| Centerline editor: eliminazione di punti, opzione di morbidezza | ❌ |
| Linee a matita texturizzate → tratti di pennello texturizzati; contagocce per la texture della matita | ❌ |
| Strumento Line curvo / a S; pennello Auto-Fill; pennello Repaint | ❌ |
| Guide di allineamento, snapping limitato ai livelli selezionati | ❌ |
| Quick Ease / Ease In / Ease Out sui keyframe selezionati | 🟡 easing per chiave ✅; comandi di ease in batch ❌ |
| Marker di scena (senza durata, ripple edit) | ❌ |
| Vista di sostituzione dei disegni (rinomina/filtro) | 🟡 livelli switch ✅ |
| Nodi di surface shading: ambient occlusion, cast shadow, surface map/normal | ❌ |
| **Nodo Weighted Deform** | 🟡 mesh d'immagine con pesi ✅ |
| Nodo Point Kinematic Output (campionamento a 1/2/3 punti) | ❌ |
| Nodo di cache di pre-render | 🟡 cache di bake delle molle ✅ |
| Nodi di colour fade / curves / levels / gamma | ❌ |
| Forma di particelle a regione planare usando un disegno vettoriale | 🟡 emettitore ✅; forma personalizzata ❌ |
| Master controller: deformer-su-deformer, Function Wizard, Slider Wizard | 🟡 canali ✅; UI wizard ❌ |
| Scripting Python (PythonManager) | 🟡 patch op JSON + CLI; nessuno scripting incorporato |
| Gestione dello spazio colore, rendering a 16 bit, filmato di post-process con script | ❌ (post-process con ffmpeg a mano) |
| Control Center: copia/import di scene in batch/esecuzione di script, coda di render | ❌ |

## OpenToonz 1.7
| Funzionalità | OSATURA |
|---|---|
| Modalità linea retta per il pennello | ❌ |
| Supporto pennelli GIMP + MyPaint | ❌ |
| Modalità di riempimento Selection+Freehand | ❌ |
| Incolla immagine dagli appunti | ❌ |
| Dashboard di pre-produzione / storyboard | ❌ |
| Pannelli personalizzati, finestra principale traslucida, rotazione/capovolgimento del viewer | ❌ |
| Scala Xsheet + navigazione per tag | 🟡 dati della timeline ✅; nessuna GUI |
| Export con informazioni sul movimento della camera | 🟡 camera in Frame ✅ |
| **Export OCA (Open Cel Animation)** | ❌ |
| Export MOV e APNG | 🟡 MP4/GIF ✅; APNG/MOV ❌ |
| Conversione batch TZP→TLV | ❌ |
| Floor Bump Iwa, Flow Fx; rendering float + colore lineare | ❌ |
| Supporto per Haiku OS | n/a |

## Synfig 1.4.2 (e 1.4.0 dal primo giro)
| Funzionalità | OSATURA |
|---|---|
| Export Lottie con livelli outline avanzati e blur, gamma | 🟡 export Lottie ✅ (forme/trasformazioni); blur ❌ |
| Editor delle scorciatoie da tastiera, scorciatoie di strumento a tasto singolo | ❌ |
| TimeLoop modificabile nel workspace | 🟡 clip in loop ✅ |
| Duplicazione dei livelli Skeleton / Skeleton Deformation con collegamenti alle ossa | ✅ (le ossa sono dati di progetto) |
| Importer SVG che gestisce gli attributi di stile | ❌ (nessun import SVG) |

## Voci dal primo giro (pagine dei fornitori, abbreviate)
- **Spine 4.2/4.3**: vincoli di fisica, slider, mix di vincoli, pixel snapping — molle ✅ (pre-calcolate), slider/mix 🟡 (`mix` IK ✅), modalità pixels ❌.
- **Live2D Cubism 4.0–5.3**: ArtPath, onion skin, export WebM-alpha — ArtPath ❌; onion skin e WebM-alpha ✅.
- **Harmony 21/22**: deformatori con pesi, texture di line-boil — 🟡 / ✅ (oscillazione).
- **Cartoon Animator 5** — vedi le tabelle dei concorrenti sopra.

## Nuovi candidati per la roadmap da questo giro
13. Export OCA (Open Cel Animation) e APNG — OpenToonz 1.7
14. Comandi di ease in batch / "quick ease" nell'editor — Harmony 20
15. Filtri di correzione colore (fade, levels, curves, gamma) — Harmony 20
16. Surface shading (cast shadow / AO) per livelli vettoriali — Harmony 20
17. Scripting incorporato (Python/Lua) sopra l'API patch-op — Harmony 20
18. Nodo di cache di pre-render / valutazione in cache — Harmony 20
19. Marker di scena + ripple editing sulla timeline — Harmony 20
20. Layer comp, re-parenting animato delle ossa, patch layer, follow-path, timecode SMPTE, safe zone, auto-trace, scatter brush (lacune trovate nella ricerca su Moho)

## Lacune note (non recuperabili da fonti accessibili)
Anime Studio 8.x, 10.1.x, 11.1; dettagli di Moho 12.5 e 14.1; Spine 3.0–4.1 (il blog 3.7 è solo una pagina video); Harmony 17, 23, 25–27 (pagine della documentazione in errore); Synfig 1.4.1; TVPaint 11; Cartoon Animator 4/5.1/5.2; changelog di Rive.

## Fonti (secondo giro)
- docs.toonboom.com — note di rilascio di Harmony 20
- en.ubunlog.com — articolo su OpenToonz 1.7
- synfig.readthedocs.io — note di rilascio di Synfig 1.4.2
- esotericsoftware.com — rassegna delle funzionalità di Spine 3.7 (pagina video, nessun elenco)

---

# Addendum — Fonti YouTube (terzo giro, 2026-10-19)

WebSearch limitato a youtube.com restituisce titoli di video; WebFetch restituisce la **descrizione/elenco dei capitoli** di ciascun video (nessuna trascrizione). Sono elencate solo le funzionalità lì nominate; "novità della versione X" è come dichiarato dall'uploader (spesso il fornitore).

## Moho 14.1 (descrizione del video)
| Funzionalità | OSATURA |
|---|---|
| Mostrare/nascondere le maniglie nello strumento Curvature | ❌ (nessuna GUI con canvas) |
| Effetti di forma Shaded e Halo visibili in modalità di modifica | 🟡 gradienti ✅; effetti halo/shade ❌ |
| Effetto di pixelazione nelle Live Project Settings | ❌ |
| Preview Animation: risoluzione più alta, bordo di output nascosto | 🟡 anteprima raster ✅ |
| Finestre di dialogo non modali per gli script Lua | ❌ (nessuno scripting) |
| Modalità di fusione PSD "Linear Dodge (Add)"; modalità di fusione aggiornate | 🟡 verificare l'elenco delle modalità di fusione; add/dodge ❌ |
| Indicatore di documento non salvato nella barra del titolo | ✅ lo studio tiene traccia dello stato di undo |
| Correzioni: valutazione degli smart bone, motion blur sub-frame + dinamica delle ossa, maniglie di Bézier di lunghezza zero | n/a |

## Moho 12.5
Le pagine di titolo/descrizione non davano alcun elenco di funzionalità (video obsoleto; download gratuito di 12.5 ritirato). I titoli correlati citano tutorial sugli Smart Bone, rig 2.5D con Smart Warp e l'installazione di "Mynd Tools" — Smart Bone 🟡 (canali/morph), Smart Warp 🟡 (lattice FFD ✅).

## Spine
| Versione | Funzionalità | OSATURA |
|---|---|---|
| 3.7 | Supporto audio | ❌ (WAV analizzato solo per il lip-sync) |
| 3.7 | Export rinnovati | 🟡 |
| 3.7 | Digita per cercare | ❌ |
| 3.7 | IK stretchy / compressa / uniforme | ❌ (l'IK conserva le lunghezze) |
| 3.7 | Combinare skin, duplicare skin, placeholder | 🟡 livelli switch |
| 3.7 | Copia/incolla dei vertici | ❌ |
| 3.7 | Rendering a griglia di pixel | ❌ |
| 3.7 | Blending additivo delle animazioni | 🟡 solo morph additivi |
| 3.7 | Runtime spine-cpp; web player | ❌ / 🟡 player HTML |
| 3.8 | Mesh tracing | 🟡 mesh a griglia automatica; nessun tracing dell'immagine |
| 3.8 | Texture packing poligonale | ❌ |
| 3.8 | **Soft IK** | ❌ |
| 3.8 | Ossa e vincoli delle skin | ❌ |
| 3.8 | Progetto mix-and-match | ❌ |
| 3.8 | Cartelle, righelli, export PSD migliorato, schermata di benvenuto | ❌ |
| 3.8 | .json/.skel più piccoli; caricamento binario in spine-ts | ❌ (solo JSON) |
| 4.0 beta | **Graph / curve editor** | 🟡 easing per chiave ✅; nessun editor |
| 4.1 beta | Funzione Sequence (attachment a sequenza di immagini) | ❌ |

## Toon Boom Harmony 25 / 25.2
| Funzionalità | OSATURA |
|---|---|
| Strumento Pencil Retouch | ❌ |
| Import dei pennelli Photoshop | ❌ |
| **Breakdown Assistant per l'animazione cut-out** | ❌ (il generatore di walk cycle è affine) |
| Blocco del righello della timeline; preferenze dei template di azione | ❌ |
| Migliore supporto per file immagine, 3D e USDZ | ❌ |
| Art Layer Manager (scambia, svuota, unisci, duplica livelli artistici; 25.2+) | ❌ (duplicazione dei livelli tramite patch op ✅) |
| Master Controller Grid Wizard | ❌ |

## Serie Cartoon Animator 5
| Funzionalità | OSATURA |
|---|---|
| 5.1 Modifica round-trip di PSD e vettoriale | ❌ |
| 5.1 Import di sequenze di immagini e GIF | ❌ |
| 5.2 **Motion Pilot** (guidare il movimento del puppet trascinando) | ❌ |
| 5.2 Animazione su Motion Path / modifica del percorso | ❌ (roadmap n. 1) |
| Dinamica a molla, FFD, animazione vettoriale (esistenti) | ✅ / ✅ / ✅ |

## Nuovi candidati per la roadmap da questo giro
21. Modalità Soft IK + stretchy IK — Spine 3.8/3.7
22. Livello a sequenza di immagini (attachment con elenco di fotogrammi) e import GIF/APNG — Spine 4.1, CA 5.1
23. Filtro di output di pixelazione / pixel-snap — Moho 14.1, Spine 4.2
24. Registrazione in stile Motion Pilot: catturare il trascinamento live di un osso/maniglia in chiavi — CA 5.2
25. Breakdown assistant (interpolazioni automatiche con archi per rig cut-out) — Harmony 25
26. Mesh tracing dal contorno alpha — Spine 3.8
27. Insiemi di skin (scambio di gruppi di attachment insieme) — Spine 3.7/3.8
28. Traccia audio nel progetto + sincronizzazione della riproduzione (non solo analisi) — Spine 3.7
29. Hook di scripting Lua/Python — Moho, Harmony 20

## Fonti (terzo giro, YouTube)
- Moho 14.1: A Complete Breakdown — youtube.com/watch?v=QT4AiV1wO1k; Moho 14.1 Update — watch?v=FCiBKBjDaNg
- Rassegna delle funzionalità di Spine 3.7 — watch?v=Fi0ks2LeYSM; rassegna delle funzionalità di Spine 3.8 — watch?v=kU26B5465SQ; curve editor di Spine v4-beta — watch?v=37CeFRzrjqc
- Cambiamenti di Harmony 25 — watch?v=kJ58C2DNIk8; Art Layer Manager 25.2+ — watch?v=AZc-8zHucOo
- Cartoon Animator 5.2 Motion Pilot — watch?v=Eu7vO2uGGN4; import di sequenze/GIF di 5.1 — watch?v=O8EGwVwqgfs

---

# Addendum — Ricerca esaustiva delle lacune (quarto giro, 2026-10-19)

Fonti: note di rilascio ufficiali di Synfig/Rive, titoli/descrizioni di YouTube. Sono elencate solo le funzionalità nominate.

## Anime Studio 8 (serie YouTube "Pro Feature", titoli + descrizioni)
| Funzionalità | OSATURA |
|---|---|
| Tracciamento automatico delle immagini (bitmap → vettore modificabile) | ❌ (roadmap n. 20) |
| Character Wizard (creazione guidata di personaggi) | 🟡 rig demo/generatore di camminata; nessun wizard |
| Import di file Photoshop (PSD) | ❌ |
| Real-Time Media Connection (collegamento live a media esterni) | ❌ |
| Strumento Scale Points | 🟡 patch op sui punti |
| Preset di progetto HD 1080p | ✅ qualsiasi dimensione |
Anime Studio 10.1 / 11.1: nessuna fonte accessibile trovata.

## Synfig 1.4.1 – 1.4.4
| Versione | Funzionalità | OSATURA |
|---|---|---|
| 1.4.1 | Livelli audio mixati nel video esportato | ❌ (nessuna traccia audio) |
| 1.4.1 | Duplicate-layer copia tutti i livelli selezionati | ✅ (op in batch) |
| 1.4.3 | Richiesta di bone-link: collegare la trasformazione invece dell'origine | ✅ |
| 1.4.3 | Import Papagayo (.pgo); correzioni all'import SVG; frame rate delle sequenze di immagini | ❌ / ❌ / ❌ |
| 1.4.3 | Correzioni al render GIF, compatibilità FFmpeg 3.x | ✅ GIF/MP4 |
| 1.4.3 | Salvataggio automatico (fino a 15 s) | ❌ (salvataggio esplicito) |
| 1.4.4 | Correzione dell'export dell'ultimo fotogramma | ✅ coperto dai test |

## Rive (note di rilascio ufficiali, set–ott 2026) — il concorrente più attivo
| Funzionalità | OSATURA |
|---|---|
| Convertitore String Template (testo da valori di view-model, formattazione dei numeri) | ❌ |
| Tratti tratteggiati (Dash Effect), importati da SVG | ❌ |
| Tenere premuto Q/W/E + trascinare per spostare/ruotare/scalare | ❌ (nessuna GUI) |
| Forme di pennello animabili in modalità Animate | ❌ |
| Copia/incolla di keyframe di vincoli; i convertitori mantengono le formule | 🟡 vincoli ✅, copia ❌ |
| Data binding relativo; view model; proprietà Blob | ❌ |
| Scripting Luau (fetch, text shaping, canvas GPU, shader/WGSL, mesh d'immagine istanziate) | ❌ |
| Blend state / Mix-by-view-model sulla timeline | 🟡 bake del crossfade |
| Listener di tastiera, focus, gamepad; Text Input; semantica di accessibilità | ❌ |
| Layout Grid e Stack, liste virtualizzate, vincoli di scroll e fisica dello scroll | ❌ |
| Set Parent (Alt+P) che mantiene posizione e ordine di disegno | 🟡 re-parent tramite patch op; keep-world ❌ |
| Align to Key Object; offset dell'osso radice; Scroll to selection | ❌ |
| Intensità additiva per riempimenti/tratti; canali RGBA separati animabili/collegabili | 🟡 morph additivi; RGBA con chiavi ✅ |
| Tratti interni/esterni (accesso anticipato) | ❌ |
| Auto Trace (migliore per la pixel art) | ❌ |
| Conversione Flash `.fla`/XFL → Rive | ❌ |
| Preset di render: state machine → video | 🟡 clip → MP4/GIF |
| Export `.riv` nel piano gratuito; CLI (`rive push/publish/ls/docs`), estensione VS Code, simulazione di input headless | 🟡 CLI ✅ (comandi propri) |
| Agente IA nell'editor | ❌ |
| Anteprima Android; runtime React Native / Apple / Android / Flutter | ❌ |
| Cronologia delle revisioni e ripristino, sincronizzazione offline, undo che mantiene lo zoom | 🟡 undo/redo ✅ |

## TVPaint Animation 11 (descrizioni YouTube)
Creazione di scorciatoie, sotto-disegni rimovibili, smussatura delle linee, registrazione del disegno, clic e trascinamento del punto pivot di KeyFramer (11.7.1), Sprite Generator, Inlay Texture FX, guide, video "end of update releases" — tutto ❌ (strumenti di pittura raster; OSATURA è vettoriale/rig).

## Toon Boom Harmony 17 (YouTube)
Free Form Deformer 🟡 (lattice FFD ✅), Master Controller potenziati 🟡, RGB Difference Keyer ❌, Motion Blur ❌ (roadmap n. 5).

## Cartoon Animator 4 / Spine 3.6 / Harmony 22–23
Video trovati (CA 4: plugin mocap Rokoko, aggiornamento da movimento 3D a 2D, normal map e illuminazione in 4.5; percorsi di Spine 3.4; video sulle funzionalità di Spine 3.6; strumenti di disegno di Harmony 22) ma le descrizioni non elencano funzionalità → non dichiarate. Degni di nota solo dal titolo: **retargeting di motion capture** (CA 4) ❌, **normal map/illuminazione** (CA 4.5) ❌, **vincoli di percorso** (Spine 3.4) ❌.

## Nuovi candidati per la roadmap da questo giro
30. Dash effect + tratti interni/esterni
31. Runtime di scripting incorporato (Lua/Luau) con API senza GPU
32. Data binding / view model + input della state machine (tastiera/puntatore/focus) per il player HTML
33. Vincoli di scroll / contenitori di layout (grid, stack) per animazioni in stile UI
34. Import del lip-sync Papagayo (.pgo); import in stile Rhubarb
35. Set Parent che mantiene la trasformazione world
36. Salvataggio automatico + cronologia delle revisioni nello studio
37. Filtri Motion Blur e RGB keyer
38. Vincoli di percorso (osso lungo una Bézier) — Spine 3.4, si fonde con la roadmap n. 1
39. Retargeting di motion capture su rig 2D (BVH/JSON) — CA 4
40. Illuminazione con normal map per livelli immagine — CA 4.5
41. Traccia audio mixata nell'export MP4 — Synfig 1.4.1
42. Import .fla / XFL e PSD

## Ancora non recuperabili
Anime Studio 10.1 / 11.1, note dettagliate di Moho 12.5, pagine di dettaglio di Spine 3.0–3.6, Harmony 23, note dettagliate di Cartoon Animator 4.x, testo del changelog di TVPaint 11.

## Fonti (quarto giro)
- synfig.readthedocs.io — note di rilascio 1.4.1, 1.4.3, 1.4.4
- releasebot.io/updates/rive — Rive Editor 0.9–0.9.272, CLI 1.2–1.6
- YouTube: serie Anime Studio 8 Pro Feature (q_HH2zOE-Yk, CUemDIlPH6Y, 408CLy652mU, l9eXLUO9Dus); demo di TVPaint 11 (MoqNrCAUy5k); Harmony 17 (1CU_3RJBcl4); playlist di Cartoon Animator 4

---

# Addendum — Social network e canali della community (quinto giro, 2026-10-19)

**Realtà dell'accesso:** le pagine di X/Twitter sono bloccate al fetching da robots.txt (sono visibili solo snippet/titoli dei risultati di ricerca); lo stesso vale per Facebook; le ricerche su Reddit con filtro di dominio sono state rifiutate dal proxy di ricerca e quelle senza filtro non hanno restituito thread di Reddit; il forum di Lost Marble serve un guscio di reindirizzamento JavaScript senza contenuto leggibile. Nulla è stato aggirato. Ciò che era leggibile:

## Canali ufficiali di Moho (news su moho.lostmarble.com, snippet di X)
| Voce | Note | OSATURA |
|---|---|---|
| Moho 13.5.3 (apr 2022) | lip-sync automatico migliorato, interfaccia migliore, ossa meno invadenti | ✅ lip-sync (euristica su WAV) |
| Moho 13.5.5 (X, mag 2022) | **Relative Paths** per gli asset collegati | ✅ la directory degli asset è relativa |
| Moho 14.3 "Mantis Shrimp" (dic 2024) | aggiornamento incentrato sui pennelli | ❌ |
| Moho 14.4 "Side Quest" (nov 2025) | export verso motori di gioco e software 3D | 🟡 Lottie/JSON proprio; glTF/FBX ❌ |
| Moho 14.5 "Showtime" (ott 2026) | onion skin magnetici, Shift & Trace, opzioni di Liquid Shapes, EXR/PNG con profili colore, modalità live-playback per trasmettere/controllare personaggi in tempo reale | vedi matrice principale; modalità live ❌ |
| Ambassador Program (giu 2026); posizione ufficiale: nessuno strumento IA previsto (mar 2025) | policy | n/a — nemmeno OSATURA ha funzionalità IA |
| Webinar (Víctor Paredes, ecc.) | tutorial | n/a |

## Confronti della community (forum di Spine, "Spine vs Moho 12")
- Gli utenti apprezzano in Moho: fisica integrata, mascheratura dei livelli, smart bone, editing vettoriale.
- Gli utenti criticano: FFD ritenuto debole; **nessun vero runtime**, export per i giochi limitato a FBX.
- Creature elogiato per la motion capture integrata (FFD criticato).
- I vincoli di fisica di Spine 4.2 ricevono reazioni entusiastiche su X (post di utenti).
OSATURA: fisica ✅ (molle pre-calcolate), mascheratura 🟡 (clip tramite booleane), smart bone 🟡, editing vettoriale ✅, runtime ❌, mocap ❌.

## Titoli di wish-list del forum visti (contenuto non leggibile)
"Concentrate on Timeline/Workflow improvements" (2024), "Road map for Moho", "Help Shape the Future of Anime Studio", "13.5.2 performance issue" — suggerisce che timeline/workflow e prestazioni siano le aree più richieste; non è un elenco di funzionalità verificato.

## Aggiunte alla roadmap
43. Modalità live-playback / streaming in tempo reale (input di pose via WebSocket → fotogrammi renderizzati) — Moho 14.5
44. Libreria di runtime in un piccolo linguaggio portabile (build WASM dell'evaluator) — la debolezza più citata di Moho
45. Mascheratura dei livelli come tipo di livello di prima classe (non solo operazioni booleane)
46. Ergonomia di timeline/workflow nello studio (graph editor, selezione multipla delle chiavi) — wish-list della community

## Fonti (quinto giro)
- moho.lostmarble.com/blogs/news (17 post, dic 2021 – ott 2026)
- Snippet di ricerca su X: post di @MohoAnimation (Relative Paths di 13.5.5; prezzi di Moho 14), post di utenti sulla fisica di Spine 4.2
- Thread del forum di Spine "8033 spine vs moho 12" (de.esotericsoftware.com)
- Titoli dei thread del forum di Lost Marble (contenuto non leggibile)

---

# Addendum — Wishlist e bacheche di richieste di funzionalità (sesto giro, 2026-10-19)

Sono elencate solo le richieste effettivamente leggibili. La colonna "OSATURA" indica se la richiesta è qualcosa che il motore già copre.

## Moho (forum di Lost Marble)
Leggibile: solo l'indice del forum. Esistono le sezioni "Feature Request Discussions" e "Unofficial Bug Reports and Feature Requests"; ultimo titolo di richiesta visibile: **"Dock Windows – Custom Interface"** (18 ott 2026). Il thread di lunga durata "Help Shape the Future of Anime Studio" (828 risposte, 2011–2021) e altri thread di wish-list ("Road map for Moho", "My wish for Moho in 2024 – Concentrate on Timeline/Workflow improvements", "Request for top 10 Anime Studio Pro 6 features") sono gusci di reindirizzamento JavaScript → contenuto non leggibile. Confronti specifici da forum a forum (forum di Spine): fisica, mascheratura, smart bone desiderati in Spine; Moho criticato per FFD debole e assenza di runtime.
| Richiesta | OSATURA |
|---|---|
| Finestre ancorabili / interfaccia personalizzata | ❌ (layout fisso dello studio web) |
| Miglioramenti a timeline / workflow | 🟡 |
| Vero runtime per i giochi | ❌ |
| FFD più potente | ✅ lattice di Bernstein + mesh |

## Cartoon Animator (Reallusion Feedback Tracker + thread di wish-list)
| Richiesta (voti dove indicati) | OSATURA |
|---|---|
| Curva di keyframe predefinita nelle preferenze | ✅ easing per chiave |
| Modifica di keyframe multicanale | 🟡 op in batch con patch |
| Impostazioni dei pannelli persistenti | ❌ |
| Marker e canali codificati a colori (sinistra rosso / destra verde) | ❌ (specchio S↔D ✅) |
| "Uber key" per mostrare tutti i canali animati | ❌ |
| Supporto SVG (Affinity, Inkscape); incolla di percorsi vettoriali | ❌ |
| Strumento/pannello di allineamento | ❌ |
| Profondità di campo / filtro blur | ❌ |
| Import di GIF/APNG/video con drag-drop posizionato sulla timeline | ❌ |
| Luci spot/ambient con ombre | ❌ |
| Scorrimento automatico della timeline | n/a |
| Testo migliore: kerning/leading animabili | ❌ |
| Colore SVG animabile | ✅ |
| Canali nascosti globalmente / "shy" | ❌ |
| Pannelli ancorati | ❌ |
| Più scene per progetto | ❌ |
| **Pannello Auto Animate (movimento idle procedurale sulle ossa)** | 🟡 oscillazione con noise coerente ✅ |
| Livello di disegno live / proprietà dinamiche modificabili (18) | ❌ |
| Hitbox e fisica (14) | 🟡 molle |
| Import di MOV trasparenti (14) | ❌ |
| Panorama a 360° (5); idee di integrazione IA (6) | ❌ |

## Bacheca delle richieste di funzionalità di Live2D Cubism Editor (30 thread, 2023–2026)
| Richiesta | OSATURA |
|---|---|
| Supporto Linux | ✅ |
| Scorciatoie da tastiera nella modifica manuale della mesh | n/a |
| Limite dei blendshape fino a −100% | ✅ morph illimitati |
| Ridimensionamento proporzionale; strumento gravità; blocco mesh-su-mesh | ❌ / ❌ / ❌ |
| Glue per i deformatori + pesi di glue precisi | 🟡 pesi ✅ |
| Import della fisica senza sovrascrivere i gruppi esistenti | ❌ |
| Blocco dei parametri nell'animator; loop-edit vs repeat | ❌ |
| Opzioni "Expand Warp based on Child"; cambio del centro di rotazione del warp | ❌ |
| Strumento tipo "Set Vertex Flow", selezione dei bordi in loop, mappatura mesh con tratto a densità variabile | ❌ (auto-mesh uniforme) |
| Supporto per la pixel art | ❌ |
| Opacità dei vertici; filtro delle parti e visualizzazione più pulita | ❌ |
| Spostare facilmente i keyframe | ✅ patch op |
| Svuotare dalla memoria l'atlas in cache | n/a |
| Finestra Physics: massimo % di output, modifica in blocco degli intervalli del pendolo | 🟡 |
| Cubism Core per RISC-V / embedded | ❌ |
| Drawable opzionali nella specifica CDI3 | n/a |

## Toon Boom Harmony
Il forum degli utenti è chiuso e reindirizza a Discord (non ricercabile); i vecchi thread della "Feature Requests board" ("One can only dream: A Feature List", "Software Upgrade Suggestions Part 2", "Bitmap Layer Overhaul LONG OVERDUE", "Various Quality of Life suggestions") mostrano solo l'avviso di chiusura. Nessuna richiesta recuperabile.

## Synfig (elenco di idee GSoC 2019–2022, numeri delle issue GitHub collegate)
| Richiesta | OSATURA |
|---|---|
| Copia/incolla dei waypoint (#2139) | ✅ chiavi tramite op |
| Cambio scheda con Ctrl+Tab, scorciatoie di finestra, selezione dell'area di riproduzione, ricerca nelle proprietà, attivazione dei righelli, modifica dei parametri col mouse | ❌ (GUI) |
| Prestazioni: render/export più veloci; cache dei parametri dei livelli; albero incrementale dei render-task | ✅ evaluate come funzione pura; nessuna cache ❌ |
| Port a GTK 4, build con Conan/vcpkg | n/a |

## Spine
Le richieste vanno sul forum e sono spostate su una bacheca Trello pubblica (non scaricabile: robots.txt). Dal forum: la priorità storicamente più alta è la stabilità dell'editor e gli aggiornamenti dei runtime. Il piano dell'era Kickstarter (mesh → FFD → skinning → IK pinning) è ormai interamente realizzato, quindi mesh/FFD/skinning di OSATURA sono alla pari con quel vecchio elenco; IK pinning ✅ (vincolo Pin). Titoli recenti di richieste del forum: supporto per le ultime three.js / React Three Fiber; supporto di Spine 4.3 in Defold.

## Rive
La roadmap pubblica = "canale Feature Requests" con etichette di stato (in corso / pianificato / rilasciato); per leggerla serve il login. Nulla di recuperabile oltre alle note di rilascio (quarto giro).

## OpenToonz, TVPaint, Pencil2D
Le pagine GitHub (OpenToonz/Synfig) sono bloccate da robots.txt; nessuna bacheca TVPaint leggibile trovata; esiste un thread "Feature requests to add" di Pencil2D ma non è stato aperto (fuori ambito).

## Aggiunte alla roadmap dalle wishlist
47. Auto-animate / livello idle procedurale (respiro, ammiccamento, oscillazione della testa con rarità e intensità) — desiderio CA n. 20; si basa sull'oscillazione esistente
48. Profondità di campo della camera e filtro blur — CA
49. Luci (spot/ambient) con cast shadow per livelli immagine — CA
50. Operazioni di allineamento / distribuzione (op dell'editor) — CA
51. Codifica a colori dei canali e visibilità "shy" dei canali nello studio — CA
52. Più scene per progetto + stacchi di scena — CA
53. Auto-mesh a densità variabile; smussatura del vertex-flow — Live2D
54. Parametro del centro di rotazione del warp deformer; ridimensionamento proporzionale — Live2D
55. Merge all'import della fisica senza sovrascrivere i gruppi — Live2D
56. Blocco dei parametri per track durante la modifica dei loop — Live2D
57. Layout dello studio ancorabile/personalizzato — Moho, CA
58. Cache dei parametri tra i fotogrammi (valutazione incrementale) — Synfig

## Fonti (sesto giro)
- Reallusion Feedback Tracker: "Wish list for new features for future updates"; bacheca di Cartoon Animator 5 (reallusion.com/FeedBackTracker)
- community.live2d.com — richieste di funzionalità per Cubism Editor (pagina 1 di 7)
- Elenco di idee GSoC 2022 di Synfig (synfig-docs-dev.readthedocs.io)
- Indice del forum di Lost Marble (lostmarble.net/forum); thread del forum di Spine 2250, 8033; thread Kickstarter 1501
- FAQ di Rive: "Does Rive have a public roadmap?"

---

# Addendum — Progetti open source su piattaforme git (settimo giro, 2026-10-19)

**Accesso:** l'API di ricerca di GitHub non è disponibile in questo ambiente ("sessions are bound to their configured repositories"), quindi i progetti sono stati trovati tramite ricerca web e letti dai loro README pubblici. Le API di GitLab e Codeberg sono bloccate dal proxy; le pagine di Codeberg/GitLab sono comparse solo nei risultati di ricerca (Friction era leggibile). Non è stato possibile recuperare il numero di stelle. Diversi progetti sono molto nuovi/in fase iniziale — le funzionalità elencate sono quelle dichiarate nei README, non verificate eseguendoli.

## Strumenti open source simili di rigging / animazione
| Progetto | Piattaforma | Funzionalità notevoli | Confronto con OSATURA |
|---|---|---|---|
| **Rigby2D** (jaisurya-dev-art/rigby2d) | GitHub, MIT/Apache | Editor in stile Spine (import PSD, mesh + weight paint, mesh collegate, vincoli IK/transform/path/physics/**slider**, skin con ossa specifiche della skin, eventi, attachment bounding-box/clipping/point/path, dopesheet + graph editor), export Spine JSON 3.8/4.2/4.3 + .skel, atlas, runtime JS/Rust/Godot, **server MCP integrato** per assistenti IA, desktop Tauri | OSATURA non ha GUI editor, atlas, runtime, skin, eventi, vincoli slider/path; ha un evaluator a funzione pura, FX liquid/boolean/emitter, lip-sync, generatore di camminata |
| **SkelForm** (Retropaint/SkelForm, Rust) | GitHub | IK FABRIK, wrapper nativo + web + mobile | nucleo di rig simile ✅; nessuna GUI ❌ |
| **Ankhimate editor** | GitHub, pre-alpha | Modalità Setup/Animate, slot + skin, FABRIK di qualsiasi lunghezza, vincoli path/physics, tracing della silhouette della mesh dall'alpha, pennello dei pesi con modalità di fusione + blocco delle ossa, **timeline FFD**, eventi con audio, marker, offset di track per osso, **editor di formati di export basato su template**, runtime di riferimento in Rust | ❌ per lo più GUI; idee: template di export, offset di track per osso |
| **rig2d-rs** (tanhieuno41) | GitHub, Rust | Port della fisica di Spine 4.2, IK con **mix/softness/bend/compress/stretch/uniform**, auto-bind dei pesi con decadimento per distanza, smooth/normalise/limit-to-4 dei pesi, packer di atlas a scaffali, ghosting da onion skin, import/export Spine JSON | ❌ IK soft/stretch, strumenti per i pesi, atlas, interscambio con Spine |
| **Boner2D**, **Skelly**, **omarvision 2d-bones-IK** | GitHub | Strumenti scheletrici per Unity / game designer | di base |
| **Skeletal-Animator** (ostad-ai) | GitHub, Python | ossa + FABRIK, forme rettangolo/capsula/cerchio, 4 easing, onion skin, finestra di clip della camera con keyframe, **intervalli di visibilità per osso**, spritesheet PNG+JSON, **MP4/WebM/MOV/GIF trasparenti** | ❌ intervalli di visibilità, export di video/spritesheet trasparenti, onion skin |
| **Stretchy Studio** (MangoLion) | GitHub, MIT | Import PSD/PNG, **auto-rigging IA** (rilevamento di pose DWPose), setup wizard, animazione dei vertici per mesh-per-parte, skinning dei vertici per gli arti, **auto eye clipping** (iride dentro l'occhio), **shape key in stile Blender** con slider di influenza, tracce audio sulla timeline, **export Spine 4.0 JSON** | ✅ morph ≈ shape key; ❌ auto-rig, eye clipping, audio |
| **Iki** (zeikar/iki) | GitHub, MIT | Motore web in stile Live2D: quad guidati da parametri, deformatori warp-mesh/griglia, maschere stencil, fisica a molla/catena, import PSD, **auto-rigger da livelli con nomi di ruolo** (ammiccamento, sguardo, rotazione della testa, cenno, inclinazione, sopracciglia, respiro, fisica dei capelli), schema JSON aperto con **ID StandardParameter**, server MCP + plugin per Claude Code | ✅ schema JSON, lip-sync; ❌ ID di parametro standard, auto-rigger, maschere |
| **Umamo** (umamoorg/umamo) | GitHub, Java | Editor compatibile con Live2D: lettura/scrittura CMO3/MOC3 (fino a Cubism 5.4), import PSD/KRA/CLIP, command palette, preset di key-binding, **CLI diagnostica** (dump/extract/convert/diff di modelli, preset di qualità dell'auto-mesh); animazione non ancora supportata | ❌ CLI di diff dei modelli, command palette |
| **Ayatsuri2D** | GitHub | Modello `.aom` MessagePack (superinsieme di moc3), export moc3 v3–6, C ABI, fuzzing in CI, runtime senza panic, benchmark di latenza p99.9 (iniziale, "non ancora utilizzabile") | ❌ formato binario, C ABI, fuzzing |
| **Ayagami** (AyagamiDev) | GitHub, Rust/wgpu | MOC3 fino a SDK 5.0 inclusi glue e parametri blendshape, ricalcolo parziale, blending lineare/sRGB, blend avanzato/off-screen di SDK 5.3, componente Godot | ❌ renderer GPU, MOC3 |
| **kasane-2d**, **img2live**, **claude-live-2d**, **minivt** | GitHub | Modellazione in stile Live2D / ricerca da singola immagine a puppet | n/a |
| **Glaxnimate** (KDE) | mirror GitHub / KDE | animazione vettoriale; le dipendenze di build mostrano tracing bitmap Potrace, libav, scripting Python, libarchive; focus su Lottie | ❌ tracing, scripting |
| **Friction** (friction/friction, Codeberg) | Codeberg, GPL-3 | animazione vettoriale + raster per web/video, motion graphics, SVG, Skia + FFmpeg + GLSL (basato su enve) | ❌ animazione SVG, shader |
| **Tahoma2D** | GitHub | 2D + stop-motion, basato sulla versione Studio Ghibli di Toonz | ❌ |
| **Pencil2D** | GitHub, GPL-2 | disegno a mano bitmap + vettoriale, onion skin | ❌ |
| **q0idmation** (q0s0id) | GitHub | editor + player + **linguaggio di scripting** | ❌ scripting |
| **svgMotion** (michaelsboost) | GitHub | animazione vettoriale nel browser | ❌ |
| **kinetic** | GitHub | timeline di keyframe SVG → export CSS/GSAP | ❌ export CSS/GSAP |
| **dotlottie-rs** (LottieFiles) | GitHub, Rust | **state machine** (guard/transizioni/azioni), theming/**slot** (override a runtime di colore/scalare/testo/vettore), contenitore zip `.lottie` multi-animazione, C API + WASM, renderer ThorVG | 🟡 export Lottie ✅; ❌ slot/state machine/contenitore dotLottie |
| **rive-runtime** | GitHub | Runtime C++: renderer vettoriale GPU (Metal, Vulkan, D3D11/12, GL/WebGL), incapsulato da Apple/Android/Flutter/Unity/Unreal/web | ❌ |
| **spine-runtimes** | GitHub | runtime per Unity, Godot, Unreal, libGDX, MonoGame, XNA, Flutter, Haxe, C/C++/C#/TS, Android, iOS, GLFW, SDL, SFML; politica di un branch per ogni versione dell'editor | ❌ |
| **Repo di strumenti Moho** (wanderjimenezRD/Moho-Pro-14.4-2D-Animation-Tools) | GitHub | solo elenco; la descrizione non indica funzionalità — non valutato | — |

Sono comparsi repository di software piratato (es. Adobe Animate "activate-free"): non aperti né usati.

## Idee da adottare (nuovi candidati per la roadmap)
59. **Server MCP / API per agenti** sopra l'editor patch-op, così gli assistenti IA possono fare rigging e animare (Rigby2D, Iki) — si adatta all'architettura JSON-patch di OSATURA
60. **ID di parametro standard** (ParamAngleX, EyeOpen, MouthOpen…) così che qualsiasi host possa guidare un modello (Iki)
61. **Auto-rigger da livelli con nomi di ruolo**: ammiccamento, sguardo, rotazione/cenno/inclinazione della testa, sopracciglia, respiro, fisica dei capelli (Iki)
62. **Auto eye clipping** (iride mascherata dalla forma dell'occhio) (Stretchy Studio) — tramite livello Boolean
63. **Vincolo slider** e **vincolo di percorso** (Rigby2D, Spine)
64. Modalità IK **soft / stretchy / compress / uniform** (rig2d)
65. **Strumenti per i pesi**: smooth, normalise, limit-to-N, copia, blocco delle ossa, modalità di fusione del pennello (rig2d, Ankhimate)
66. **Tracing della silhouette della mesh dall'alpha** (Ankhimate)
67. **Intervalli di visibilità per osso** (Skeletal-Animator)
68. **Export video con sfondo trasparente (WebM/MOV con alpha) e spritesheet PNG+JSON** (Skeletal-Animator)
69. **Motore di template di export** (formati di output definiti dall'utente) (Ankhimate)
70. **Contenitore dotLottie `.lottie` + slot/theming a runtime + state machine** (dotlottie-rs)
71. **CLI di diff / dump / convert dei modelli** (`osatura diff a.json b.json`) (Umamo)
72. **Command palette + scorciatoie riassegnabili** nello studio (Umamo)
73. **Loader sottoposto a fuzzing, percorso di runtime senza panic, benchmark di latenza** (Ayatsuri2D) — rafforzamento della robustezza di `Engine::new`
74. **Import/export Spine JSON** per l'interscambio (Rigby2D, rig2d, Stretchy Studio)
75. **Packer di atlas a scaffali** per i livelli immagine (rig2d, Rigby2D)
76. **Eventi con valori con chiave e cue audio** sulla timeline (Rigby2D, Ankhimate)
77. **Offset / time-shift di track per osso** (Ankhimate)

## Fonti (settimo giro)
- github.com/jaisurya-dev-art/rigby2d; Retropaint/SkelForm; Ankhimate/editor; tanhieuno41/rig2d-rs (release v0.1.0)
- github.com/zeikar/iki; MangoLion/stretchystudio; umamoorg/umamo; Yoru-KenomoVT/ayatsuri_2d; AyagamiDev/ayagami
- github.com/ostad-ai/Skeletal-Animator; KDE/glaxnimate; tahoma2d/tahoma2d; pencil2d/pencil
- github.com/rive-app/rive-runtime; EsotericSoftware/spine-runtimes; LottieFiles/dotlottie-rs
- github.com/topics/animation-editor, /topics/vector-animation
- codeberg.org/friction/friction

---

# Addendum — Altre fonti pubbliche (ottavo giro, 2026-10-19)

Fonti: itch.io, Steam, AlternativeTo, Wikipedia, risultati di ricerca accademici/di brevetti, recensioni di strumenti per After Effects. Le affermazioni provengono da pagine di negozi/progetti, non da prove pratiche.

## Strumenti indie / di nicchia per il rigging (itch.io, Steam)
| Strumento | Funzionalità notevoli | OSATURA |
|---|---|---|
| **Inochi Creator / Inochi2D** (Kitsunebi Games, open source, beta) | rigging aperto di puppet 2D orientato ai VTuber + runtime in tempo reale; formato proprio (non compatibile con Live2D/Spine); export GIF/sequenza di immagini; UI multilingue | 🟡 stessa nicchia (formato di rig aperto) |
| **Keyframe.it** (browser, gratuito) | ossa + IK + mesh con pesi + movimento a molla; **clip di simboli di gioco** (idle, land, anticipation, win); editor di immagini integrato + import PSD; export GIF, sprite sheet, video, player HTML autonomo, **bundle Spine 3.8/4.2**; salvataggio automatico nel browser; connettore per agenti IA **keyframe-mcp**; assistente IA integrato opzionale | ✅ player HTML/GIF; ❌ sprite sheet, bundle Spine, MCP, editor nel browser |
| **RigBone 2D** (Godot) | posing delle ossa, posa-come-fotogramma, onion skin, ritaglio + sfondo trasparente, sequenza di fotogrammi PNG (nessun runtime necessario) | ❌ onion skin, sequenza PNG con ritaglio alpha |
| **SpineBones** | slot + attachment di immagini, gruppi, IK, **anteprima audio sulla timeline**, immagine di riferimento di sfondo, export JSON / sprite sheet / WebM | ❌ |
| **PySpine, Tangent (sprite animator)** | elencati su itch.io; nessun testo di funzionalità letto | — |
| **Mocap2D / FBX2SPINE** (Steam) | import mocap FBX/BVH; **Auto Match** dello scheletro per nomi + gerarchia; **Smart Retargeting** (proiezione, adatto alla vista laterale); **Direct Retargeting** per asse; **conversione in-place automatica** per i loop di locomozione; **IK di contatto con il suolo** (foot lock); anteprima live; import/export di FBX, BVH, scheletri Spine, Unity 2D Animation | 🟡 retarget per nome ✅; ❌ import mocap, IK foot-lock, conversione in-place |
| **VTube Studio** (Steam) | tracking da webcam (OpenSeeFace), **bocca guidata dalla voce**, mouse/penna come input di tracking, mapping di qualsiasi input su qualsiasi parametro del modello, hotkey per espressioni/animazioni, "item" PNG fissati che seguono il modello; solo modelli Live2D | 🟡 canali di parametro ✅; ❌ input mapping live, hotkey, item fissati |
| VBridger, Vitamins | Strumenti Steam attorno al VTubing; non valutati | — |

## Ecosistema di rigging per After Effects (recensione di School of Motion)
| Funzionalità | Fonte | OSATURA |
|---|---|---|
| **Controller joystick** (5 pose chiave: origine/destra/sinistra/alto/basso) interpolate in 2D | Joysticks 'n Sliders | ❌ — candidato forte |
| **Slider** che mescolano un numero illimitato di pose | JnS, Duik | 🟡 canali morph |
| Strutture (ossa indipendenti dall'artwork, riutilizzabili), autorig | Duik Bassel | ✅ |
| Controller: slider, slider 2D, angolo | Duik | ❌ |
| **Connector** (slider/joystick/rotazione che collega qualsiasi proprietà a un'altra), controllo delle proprietà master | Duik | 🟡 espressioni? (motore di espressioni ✅ nei canali) |
| Chiavi di pose intermedie tra stati di posa | Duik | ✅ |
| Animare i percorsi delle maschere | JnS | ✅ morph dei percorsi |

## App concorrenti / alternative (AlternativeTo, elenco di Wikipedia al 2026)
Tahoma2D (fork di OpenToonz, stop-motion), TupiTube, enve (dismesso; diventato Friction), Expressive Animator (SVG interattivo), Cavalry, Jitter (motion design sul web), Blazy (Figma → animazione, export CSS/React), Animation Desk, Pivot Animator, FlipaClip, animazione in Krita, Blender (mar 2026), Clip Studio Paint (animazione), Adobe Character Animator (puppeteering da webcam; pagina "what's new" bloccata da robots.txt), Adobe Animate (ultima versione elencata: ott 2023), CrazyTalk, DigiCel FlipBook, Apple Motion, RETAS. Nessuno aggiunge funzionalità di rigging oltre a quanto coperto dai giri precedenti; nuove idee: **preset da strumento di design/Figma → animazione con export CSS/React** (Blazy) e **puppeteering da webcam** (Character Animator, VTube Studio).

## Letteratura accademica e brevettuale (solo titoli/abstract)
- Jacobson et al. 2011, **Bounded Biharmonic Weights** per la deformazione in tempo reale — pesi di skinning morbidi automatici da una cage/scheletro ✅ idea: migliorare `auto_skin` (attualmente basato sulla distanza).
- Brevetti Adobe US9865073 / US10078910, **"Computer animation of artwork using fast 2-dimensional deformation solving"** (il mesh warp di Character Animator). *Nota clean-room:* OSATURA usa FFD di Bernstein e skinning con pesi, non quel solver brevettato; qualsiasi futuro solver in stile ARAP dovrebbe essere rivisto con un legale. Questa non è consulenza legale.
- Auto-rigging su arXiv 2025–26: **ASMR** (scheletro e skin di mesh 3D tramite prior generativi 2D), UniRig, "How to Train Your Dragon" (rigging a diffusione), HumanRig, SPRig, UniMate — tutti 3D/ML; rilevanti solo come futura funzionalità di "suggerimento di auto-rig".
- Stony Brook TVC 2017 **retargeting automatico dei pesi di skinning** — copia dei pesi tra mesh (✅ roadmap: copia dei pesi).

## Nuovi candidati per la roadmap da questo giro
78. **Controller joystick**: punto di controllo 2D che mescola 5 pose chiave (e varianti a N pose) → guida qualsiasi canale — JnS / Duik / parametri Live2D
79. Import mocap (BVH) + auto-match dello scheletro + conversione in-place dei loop + IK foot-lock — Mocap2D
80. Input mapping live: qualsiasi input (mouse, MIDI, livello audio, landmark della webcam) → parametro, più hotkey per le espressioni — VTube Studio
81. Item/prop fissati che seguono un osso/ancora con un proprio offset — VTube Studio
82. Preset di clip di simboli di gioco (idle / land / anticipation / win) — Keyframe.it
83. Export di spritesheet (PNG + JSON) e di sequenza PNG trasparente con ritaglio — Keyframe.it, RigBone
84. Anteprima audio sulla timeline con forma d'onda — SpineBones
85. Livello con immagine di riferimento (sfondo non renderizzato) — SpineBones
86. Pesi di auto-skin con biharmonic limitati o diffusione del calore (sostituiscono il decadimento per distanza) — articolo BBW
87. Preset da strumenti di design → export CSS/React — Blazy
88. Puppeteering con face tracking da webcam tramite lo studio (WebRTC + modello di landmark) — Character Animator, VTube Studio

## Fonti (ottavo giro)
- itch.io: kitsunebi-games/inochi-creator, keyframeit.itch.io, itch.io/blog/1586091 (RigBone 2.0), itch.io/t/6177139 (SpineBones)
- Steam: FBX2SPINE / Mocap2D (app 2467960), VTube Studio (app 1325860)
- schoolofmotion.com — Joysticks 'n Sliders vs Duik Bassel
- alternativeto.net (alternative a Moho, Expressive Animator); en.wikipedia.org/wiki/List_of_2D_animation_software
- Jacobson 2011 BBW (homes.cs.washington.edu/~jovan/papers/jacobson-2011-bbw.pdf); brevetti US9865073B2 / US10078910; arXiv 2503.13579, 2503.15586, 2504.12451; Stony Brook TVC 2017 weight retargeting

---

# Addendum — Adobe Character Animator, set completo di funzionalità (nono giro, 2026-10-19)

Fonti: tabella delle versioni di Wikipedia (inclusa la 26.0 di gen 2026), pagina di prodotto Adobe, pagina "Behaviors" di Adobe Help e note di rilascio (ott 2018, ott 2020/feb 2021, ott/dic 2021, 2023). L'indice "what's new" di Adobe restituisce solo il 2023 e il 2026; 24.x e 25.x sono elencate da Wikipedia come versioni di sola stabilità.

## Concetto
L'artwork a livelli di Photoshop/Illustrator (anche EPS/SVG dall'ott 2018) diventa un **puppet** dotato di rig automatico a partire da **nomi/tag dei livelli**; la performance viene catturata dal vivo (volto/corpo da webcam, microfono, tastiera, mouse, MIDI) come **take** su una timeline; gli output vanno a sequenze PNG, WAV, Media Encoder (incluso **video con alpha**), Dynamic Link verso After Effects / Premiere e Syphon / Mercury Transmit dal vivo. La modalità Starter è gratuita; Pro richiede Creative Cloud.

## Behavior (elenco completo da Adobe Help)
| Behavior | Cosa fa | OSATURA |
|---|---|---|
| **Face** | testa, occhi, sopracciglia, naso, bocca da webcam; parallasse e intensità della posizione della testa | ❌ (nessun tracking live) — roadmap n. 88 |
| **Body** (Body Tracker, Sensei AI, 22.0) | braccia, busto, gambe da webcam | ❌ |
| **Lip Sync** | visemi della bocca dalla voce; accuratezza assistita dalla trascrizione (22.1); motore 3.4 migliore su F, W-Oo, meno L/M | 🟡 WAV → 5 visemi ✅; allineamento alla trascrizione ❌ |
| **Nutcracker Jaw** | mandibola inferiore da volto/voce | ❌ |
| **Eye Gaze** | le pupille si muovono indipendentemente | ✅ vincoli aim/canali |
| **Head & Body Turner** | scambia gruppi di livelli (frontale, tre quarti, laterale) mentre testa/corpo ruotano | 🟡 livelli switch ✅; nessuno scambio automatico guidato dall'angolo |
| **Walk** | walk cycle, corsa, enfasi della posa, oscillazione di spalle/fianchi, **Pin Feet When Standing** (3.4/3.5) | ✅ generatore di camminata; ❌ pin-feet |
| **Limb IK** (ex Arm IK) | direzione di piegamento + stretch per braccia e gambe | 🟡 IK a 2 ossa ✅; stretch ❌ |
| **Leader/Follower** (22.5) | modella come gli arti si piegano/allungano | ❌ |
| **Dragger** | mouse/tocco sposta una regione | ❌ (lo studio non è interattivo sul canvas) |
| **Physics / Dangle** | oscillazione, collisione, gravità, vento; squash & stretch, smorzamento | 🟡 molle ✅; collisione/vento ❌ |
| **Magnets** | le maniglie si agganciano per prossimità | ❌ |
| **Particles** | emette oggetti fisici (neve, colpi di cannone) | ✅ emettitore |
| **Motion Lines** | linee di scia dietro gli oggetti veloci | ❌ |
| **Wiggler** | oscillazione sull'arte vettoriale | ✅ wobble/boil |
| **Breath** | movimento di respirazione | 🟡 canale manuale; roadmap n. 61 |
| **Auto Blink** | ammiccamenti automatici / sfarfallio dei livelli | ❌ (roadmap n. 61) |
| **Motion Library** (23.0, oltre 350 movimenti, parametro di intensità 23.1) | applica animazioni a corpo intero | 🟡 solo camminata; ❌ libreria |
| **Cycle Layers** | scorre i livelli in un gruppo | 🟡 livelli switch ✅; ciclo dei fotogrammi ❌ |
| **Layer Picker / Motion Trigger / Fader** | mostra un livello; cambia in base alla direzione del movimento; dissolvenza al trigger | 🟡 |
| **Transform** | spostamento/scala/rotazione/opacità delle maniglie | ✅ |
| **Handle Fixer** | blocca parti di una mesh | ✅ pin |
| **Triggers** | la pressione di un tasto mostra livelli/gesti; **pannello Controls** con slider, quadranti angolari, pulsanti, mapping MIDI | ❌ — roadmap n. 80 |
| **Collide, Dynamic, Cycle Pause, Auto-swap (22.5)** | tag/behavior per fisica e scambi simili a fotogrammi | ❌ |

## Funzionalità di timeline, registrazione e workflow
| Funzionalità | Versione | OSATURA |
|---|---|---|
| Take, replay (registrazioni riutilizzabili e attivabili), smart replay, replay modificabili | 2.0 / 22.0 / 23.6 | 🟡 clip ✅; replay ❌ |
| Keyframe, time stretching dei keyframe, camere di scena, audio attivabile | 3.0 / 3.2 | ✅ chiavi/camera; ❌ audio attivabile |
| Speech-Aware Animation (movimento di testa/sopracciglia dalla voce, Sensei) | 3.4 | ❌ — roadmap n. 61 |
| Unire/sequenziare i take; righe di take colorate; filtri di ricerca (`kind:` `shared:` `armed:`); righe shy; comprimere i take | 3.4 | ❌ |
| Snap ai fotogrammi / agli oggetti; snapping della timeline | 3.5 | ❌ |
| Operazioni sull'area di lavoro: ripple delete, inserimento di tempo, loop, trim, export | 2.0 | 🟡 operazioni sulle clip ✅ |
| Pannello dei problemi di rigging; ricerca di tag e trigger | 3.0 | ✅ la validazione raccoglie tutti gli errori |
| Segnalibri della cronologia; ripristino automatico di progetti danneggiati (4.2); nuovo formato di file compatto e resistente alla corruzione (4.0) | | 🟡 undo ✅; ripristino ❌ |
| Puppet Maker (template: capelli, pelle, accessori) | 22.1.1 | ❌ |
| Characterizer (trasforma una foto/un dipinto in un puppet) | 2.0 | ❌ |
| Quick Export per la condivisione sui social | 22.5 | 🟡 GIF/MP4 ✅ |
| Snapshot di scena, conto alla rovescia di registrazione, grafica personalizzata dei pulsanti di controllo | 1.5 | ❌ |
| Selezione della camera, multi-monitor, HiDPI | 2.0 | n/a |
| Output live (Syphon, Mercury Transmit), estensione Twitch Character Trigger | 1.1 / 2.1 | ❌ — roadmap n. 43 |
| MIDI, script tramite V8 (JavaScript) | 1.1 / 2.0 | ❌ — roadmap n. 31 |
| Condivisione/pubblicazione di puppet, file di puppet condivisibili | Beta 5 | 🟡 JSON portabile |
| Dynamic Link con AE / Premiere | Beta 5 | ❌ |
| Punti deboli segnalati (recensione): curva di apprendimento ripida, dipendenza dall'hardware (webcam/microfono), lungo tempo di preparazione del personaggio, Pro a pagamento, complessità della configurazione dei trigger | | opportunità: pipeline offline deterministica, CLI/JSON di OSATURA |

## Cosa spicca per OSATURA (nuovi candidati per la roadmap)
89. **Auto-rig basato su tag dai nomi dei livelli** (testa, occhi, sopracciglia, visemi della bocca, tag di braccia/gambe) — idea centrale di Character Animator; estende la roadmap n. 61
90. **Take e replay**: registrare la performance dei parametri come clip riutilizzabili e attivabili con easing di blend
91. **Pin Feet When Standing** e blocco del contatto con il suolo per il generatore di camminata
92. Behavior di stretch degli arti **Leader/Follower**; **Magnets**; **Motion Lines**
93. Formato di **Motion Library** (clip di movimento con nome e intensità 0-100%)
94. **Head & Body Turner**: scambio automatico dei gruppi di vista da un canale di angolo yaw
95. **Lip-sync assistito dalla trascrizione** (forced alignment del testo ai fonemi del WAV)
96. **Movimento di testa/sopracciglia consapevole del parlato** dall'energia e dalle pause dell'audio
97. **Export di video con alpha** (ProRes 4444 / WebM VP9 alpha) — presente anche in Skeletal-Animator (roadmap n. 68)
98. **Audio attivabile** e parametri reattivi all'audio

## Fonti (nono giro)
- en.wikipedia.org/wiki/Adobe_Character_Animator (elenco delle funzionalità e tabella delle versioni fino alla 26.0, gen 2026)
- adobe.com/products/character-animator.html
- helpx.adobe.com — Behaviors in Character Animator; riepiloghi delle funzionalità ott/dic 2021, ott 2020/feb 2021; What's new (2023–2026)
- community.adobe.com — "Character Animator October 2018 detailed changes"
- manytools.com/review/adobe-character-animator (punti di forza/debolezza)
