<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Domande frequenti

*Risposte brevi, ciascuna ricavata da qualcosa che è stato eseguito o letto nell'albero dei sorgenti e negli altri documenti. Documentazione CC-BY-4.0. Dove una risposta dipende da un limite, lo dice e rimanda a [LIMITATIONS.md](LIMITATIONS.md). Le stringhe di errore dell'ultima sezione sono state catturate da una build reale; sono citate esattamente, così puoi cercare in questa pagina il testo che vedi.*

Indice: [Installazione e configurazione](#installazione-e-configurazione) - [ffmpeg](#ffmpeg) - [Prestazioni](#prestazioni) - [Determinismo](#determinismo) - [Licenze e il tuo output](#licenze-e-il-tuo-output) - [Sicurezza dei file e percorsi degli asset](#sicurezza-dei-file-e-percorsi-degli-asset) - [Perché l'output deve essere diverso dall'input](#perché-loutput-deve-essere-diverso-dallinput) - [Rigging e animazione](#rigging-e-animazione) - [Studio](#studio) - [Volto, parlato e motion capture](#volto-parlato-e-motion-capture) - [Bug, sicurezza e contributi](#bug-sicurezza-e-contributi) - [Limitazioni](#limitazioni) - [Confronti con altri strumenti](#confronti-con-altri-strumenti) - [Tabella di risoluzione dei problemi](#tabella-di-risoluzione-dei-problemi)

---

## Installazione e configurazione

**1. Come installo OSATURA?**
Archivi precompilati (Windows x86-64, macOS arm64, Linux x86-64 e il modulo WebAssembly) sono preparati come pre-release in bozza `v0.1.0-alpha.3` su <https://github.com/chiuta/osatura/releases>; diventano scaricabili quando il proprietario pubblica la bozza e non sono firmati (SmartScreen mostra "Editore sconosciuto"; macOS chiede l'approvazione). Non c'è un installer: decomprimi l'archivio ed esegui `osatura`. Oppure lo compili dai sorgenti con una toolchain Rust stabile recente: `cargo build -p osatura-cli` produce `target/debug/osatura`; aggiungi `--release` per `target/release/osatura`. Vedi [DEVELOPMENT.md](DEVELOPMENT.md).

**2. Su quali piattaforme funziona?**
Linux x86-64 è dove il progetto è stato sviluppato ed esercitato. Su GitHub Actions la suite di test e clippy passano anche su macOS (arm64) e Windows (x86-64), e lì vengono compilati i binari di release per questi tre; ma nessuna persona ha ancora usato gli eseguibili Windows o macOS su un vero desktop, quindi sono "testati dalla CI", non "supportati" ([LIMITATIONS.md](LIMITATIONS.md), sezione 1).

**3. Esiste un installer grafico o un'app?**
Non c'è un installer. La riga di comando è il prodotto. Lo studio è una pagina web locale servita da `osatura studio`; su Windows, un doppio clic su `osatura.exe` lo avvia e apre il browser. Gli archivi Windows e macOS contengono anche `osatura-studio`, una piccola finestra nativa attorno allo stesso studio (è stata solo compilata e auto-testata in CI); vedi [STUDIO.md](STUDIO.md) e [GETTING_STARTED.md](GETTING_STARTED.md).

**4. Come comincio in fretta?**
Segui [GETTING_STARTED.md](GETTING_STARTED.md): costruisce un personaggio animato partendo da un progetto vuoto, con ogni passo eseguito, e termina con una GIF.

**5. Posso compilare senza il motore di scripting?**
Sì: `cargo build -p osatura-cli --no-default-features` lascia fuori Rhai, quindi il comando `script` è assente (vedi [DEVELOPMENT.md](DEVELOPMENT.md)). La funzionalità `scripting` è l'unico punto in cui entra un crate con una licenza a copyleft debole (smartstring); vedi [THIRD_PARTY.md](THIRD_PARTY.md).

**6. Serve una connessione di rete o un account?**
No. Il programma non richiede alcun account né la rete per funzionare (lo studio non ha account). Per impostazione predefinita lo studio ascolta solo sulla tua macchina (vedi la domanda 51).

**7. Dove conserva le impostazioni lo studio?**
In `$OSATURA_UI_CONFIG` se impostata, altrimenti in `$XDG_CONFIG_HOME/osatura/ui.json`, altrimenti su Windows in `%APPDATA%\OSATURA\ui.json`, altrimenti in `~/.config/osatura/ui.json`. I tuoi progetti non vengono mai toccati.

---

## ffmpeg

**8. Mi serve ffmpeg?**
Solo per l'esportazione MP4 e WebM, per `sound mix`, per l'audio mp3/ogg e per `face-track` su un file video. L'esportazione in GIF, APNG, HTML, sequenze PNG, SVG, glTF e JSON non richiede altro. L'elenco completo è in [CLI_REFERENCE.md](CLI_REFERENCE.md), sezione 1.6.

**9. ffmpeg è incluso?**
Mai. OSATURA avvia `ffmpeg` dal tuo `PATH` come programma separato e non viene eseguito nient'altro. Qualunque sia la situazione di licenza e di brevetti della tua build di ffmpeg, spetta a te verificarla; vedi [PATENT_NOTES.md](PATENT_NOTES.md).

**10. Cosa dice quando ffmpeg manca?**
`cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` (per WebM il suggerimento termina con `or export an APNG instead`). Non viene scritto nulla.

**11. Quali encoder deve avere il mio ffmpeg?**
`libx264` per mp4, `libvpx-vp9` o `libaom-av1` per webm, `aac` e `libopus` per il suono in questi formati, `libvorbis` e `libmp3lame` per `sound mix`.

---

## Prestazioni

**12. Perché è così lento?**
Probabilmente stai usando una build di debug. I numeri nei documenti provengono da una build di debug e sono da 10 a 30 volte più lenti di una build di release ([CLI_REFERENCE.md](CLI_REFERENCE.md), sezione 1.7). Sulla macchina di test a 2 core una build di debug ha impiegato 46 s per scrivere in PNG i 48 fotogrammi del progetto demo e 57 s per esportarlo come GIF. Una build di release appena compilata ha svolto gli stessi due lavori in 1,5 s e 2,2 s. Compila con `cargo build --release` e usa `target/release/osatura`.

**13. Usa la GPU?**
Il renderer predefinito è la CPU. Esistono un crate `gpu` e un comando `render-gpu` dietro un flag di funzionalità, ma durante lo sviluppo non era disponibile alcuna GPU reale: su una GPU software (llvmpipe) un fotogramma 640x480 ha richiesto da 253 a 295 ms contro i 13,8 ms del percorso su CPU. Considera il rendering su GPU come non testato su hardware ([LIMITATIONS.md](LIMITATIONS.md)).

**14. Quanto costa un rig per fotogramma?**
In una build di debug l'esempio `biped` ha avuto una mediana di 64,5 ms per fotogramma nel controllo di prestazioni di `ci`. Il bake della stoffa di una piccola bandiera ha richiesto 0,43 s. Misura il tuo progetto con `osatura stats` e `osatura ci`; i rig variano molto.

**15. Come alleggerisco un progetto pesante?**
Usa `lod` per creare varianti più economiche, `export-fit` per trovare le impostazioni di esportazione che rispettano un budget di byte e `proxy-bench` per confrontare i proxy. Vedi [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) e [EXPORT_FORMATS.md](EXPORT_FORMATS.md). `export-fit` può fermarsi prima con `--max-seconds` e in tal caso riporta di aver esaurito il tempo.

**16. Uno script con migliaia di chiamate fallisce. Perché?**
`osatura script` accetta al massimo 5000 richieste per esecuzione ed è tutto o niente: se una richiesta viene rifiutata non viene scritto nulla. Dividi un lavoro grande in più script che leggono l'output precedente.

---

## Determinismo

**17. L'output è riproducibile?**
Il rendering è progettato per esserlo: lo stesso progetto e lo stesso seed producono gli stessi pixel, e fisica, particelle e rumore usano un seed. È ciò che permette a `golden` e `ci` di confrontare i render. È stato verificato su un solo tipo di macchina (Linux x86-64); un output identico bit per bit su altre CPU o altri compilatori non è garantito.

**18. Perché un bake della fisica dà un risultato diverso dopo che cambio un solo numero?**
Il bake è deterministico per un dato input, ma è una simulazione: qualsiasi modifica a massa, frame rate, collider o seed cambia l'intera esecuzione. Esegui il bake con le stesse impostazioni e lo stesso seed per confrontare. Per impostazione predefinita il bake si fonde con l'animazione esistente (`merge=keep`: i canali esistenti vengono mantenuti e il comando dice `kept existing animation on 'b2' (use merge=replace to overwrite)`); usa `merge=replace` per sovrascrivere.

**19. Il rumore va in loop?**
No. Un canale `noise` non si ripete da solo, quindi un loop mostrerà un salto nella giunzione. Usa invece chiavi o un'espressione come `8 * sin(t * 2 * pi)`; nelle espressioni `t` va da 0 a 1 lungo la clip, non in fotogrammi ([ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)).

**20. Posso verificare che un'animazione vada in loop in modo pulito?**
Esegui `lint`. `make-loop` rimuove del tutto lo spostamento: in un test una palla che rotolava è diventata statica e `lint` ha avvisato (L005). `loop-solve` chiude la giunzione per il movimento ciclico, ma ha lasciato un oggetto in movimento con un salto visibile (da 280 a 280 sul suo canale di spostamento). Per uno spostamento che prosegue, usa la modalità di estrapolazione `loop_offset`.

---

## Licenze e il tuo output

*Questo è un riepilogo in linguaggio semplice di [LICENSING.md](LICENSING.md), non una consulenza legale. La revisione di un avvocato sull'impostazione delle licenze è ancora in sospeso.*

**21. Quali licenze si applicano?**
MPL-2.0 per la libreria core, la C ABI e i crate WASM e GPU; AGPL-3.0-or-later per il programma da riga di comando e lo studio (`osatura-cli`); CC-BY-4.0 per la documentazione e per gli asset e i modelli di esempio sintetici.

**22. Di chi sono le animazioni e i render che creo?**
Tuoi. La dichiarazione del progetto è che i progetti, le animazioni e i render che crei con OSATURA ti appartengono interamente; le licenze si applicano al software, non al suo output. Esportare una GIF o eseguire un comando non comporta alcun obbligo.

**23. Posso usare l'output a scopo commerciale?**
Sì, in base alla dichiarazione di cui sopra. L'eccezione è tutto ciò che copi dagli asset di esempio: quelli sono CC-BY-4.0 e richiedono l'attribuzione.

**24. Devo pubblicare i miei progetti se uso le parti AGPL?**
No. L'AGPL copre il programma. Se esegui un `osatura-cli` o uno studio modificati in modo che altre persone li usino tramite una rete, devi offrire loro il sorgente. Lo studio lo fa con una pagina `/source` e un flag `--source-url` che punta alla tua copia.

**25. Posso collegare la libreria core a un'applicazione closed-source?**
Il core è MPL-2.0, un copyleft a livello di file: puoi usarlo da codice chiuso e devi condividere le modifiche ai file coperti da MPL. La CLI e lo studio sono AGPL e sono un'altra questione.

**26. Posso dire che il mio strumento "funziona con OSATURA"?**
Sì: "compatibile con OSATURA" e "realizzato con OSATURA" sono consentiti. Una build modificata non deve essere distribuita con lo stesso nome ([TRADEMARK.md](TRADEMARK.md)).

**27. Posso rilasciare il mio contributo con doppia licenza?**
Non esiste un contributor licence agreement. I contributi sono accettati con la licenza del progetto e con un sign-off DCO (`git commit -s`), quindi non esiste un titolare che possa rilicenziarli. Dettagli in [CONTRIBUTING.md](CONTRIBUTING.md).

**28. E il codice di terze parti?**
Circa 360 crate di terze parti (contati su tutte le piattaforme; l'app a finestra aggiunge il toolkit per Windows / macOS), tutti con licenze permissive tranne sei MPL-2.0: smartstring (MPL-2.0 o successiva, solo con la funzionalità `scripting`) e cinque che solo l'app a finestra può includere. Gli elenchi sono in [THIRD_PARTY.md](THIRD_PARTY.md) e [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md).

**29. Ci sono problemi di brevetti?**
Vedi [PATENT_NOTES.md](PATENT_NOTES.md). Sono note di ingegneria, non un parere legale.

---

## Sicurezza dei file e percorsi degli asset

**30. Un comando può danneggiare il mio progetto?**
Le scritture sono atomiche: il risultato va in un file temporaneo nella stessa cartella, viene sincronizzato e rinominato sopra la destinazione, quindi un crash lascia il vecchio file. I comandi di modifica rivalidano il progetto che stanno per scrivere e rifiutano di scriverne uno non valido (`output would be invalid: ...`). Il comando `script` non applica nulla se una qualsiasi richiesta fallisce.

**31. È sicuro aprire un progetto di qualcun altro?**
Ragionevolmente, ma non in modo incondizionato. Un progetto è dati, non codice; l'unico programma esterno che OSATURA avvia è ffmpeg, per i formati elencati nella domanda 8. Il loader rifiuta dispositivi, pipe e directory, limita la dimensione dell'input e rifiuta i percorsi di asset assoluti o con `..`. Gli script Rhai girano in una sandbox. Leggi comunque [SECURITY.md](SECURITY.md) prima di elaborare file non fidati in un servizio.

**32. Dove cerca suoni e immagini un progetto?**
Relativamente alla cartella del file di progetto. I percorsi assoluti e qualsiasi percorso con `..` vengono rifiutati a meno che tu non passi `--allow-external-assets`, e devi passarlo di nuovo a ogni comando successivo che carica quel progetto. Esempio dal riferimento:
`sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)`.

**33. Che dimensione di progetto legge?**
Fino a 256 MiB per impostazione predefinita. Cambialo con `--max-input-bytes N`. I file più grandi falliscono con `cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)` (i numeri sono quelli dell'esempio).

**34. Perché rifiuta di scrivere su un collegamento simbolico?**
Perché un collegamento piazzato accanto al tuo output non possa reindirizzare una scrittura altrove. Il messaggio è `refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself`.

**35. Perché non posso leggere da `<(...)` o da una pipe?**
Gli input devono essere file regolari: `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)`. Scrivi prima i dati in un file.

**36. Un campo sconosciuto in un progetto lo rompe?**
No: viene stampato `warning: unknown field bogus (ignored)` e lo stato di uscita è 0. I file ausiliari che scrivi per un comando (contratti, mappature, macchine) sono più rigorosi: una chiave sconosciuta lì è un errore e non viene applicato nulla.

---

## Perché l'output deve essere diverso dall'input

**37. Perché `render in.png in.png` dice "refusing to overwrite"?**
Per i comandi che leggono un progetto e scrivono un'immagine, un video o un altro file che non è un progetto, il nome dell'output non può coincidere con quello dell'input. Il messaggio è `refusing to overwrite the input project X with output of the same name`. Evita il classico errore di esportare sopra l'unica copia del progetto.

**38. Allora perché `script` e `autorig` possono scrivere sopra il proprio input?**
La protezione riguarda gli output che non sono progetti. I comandi che leggono un progetto e scrivono un progetto (`script`, `autorig`, `loop-solve` e altri) possono indicare lo stesso file per input e output; la scrittura è atomica e, se il comando fallisce, non cambia nulla. Se vuoi conservare l'originale, dai un nuovo nome oppure usa `osatura vcs` per mantenere la cronologia ([CI.md](CI.md) e [CLI_REFERENCE.md](CLI_REFERENCE.md)).

**39. Perché `render` rifiuta `out.jpg`?**
`render` scrive solo `.png` o `.svg`: `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)`. Per gli altri formati usa `export`; l'elenco è in [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

---

## Rigging e animazione

**40. In che direzione è lo zero gradi per un osso?**
L'angolo 90 dell'osso punta verso il basso; gli angoli non sono misurati dall'asse x nel modo in cui lo fa un libro di matematica. I canali sono delta rispetto alla posa di riposo e `rest_offset` è nel sistema di riferimento del genitore ([RIGGING_GUIDE.md](RIGGING_GUIDE.md)).

**41. La mia catena IK non si estende o ha uno scatto. Cosa sono `soft` e `stretch`?**
`soft` (da 0 a 0,9) addolcisce l'arrivo della catena alla piena estensione e `stretch` (da 1 a 4) le permette di allungarsi. Entrambi sono solo JSON, non opzioni di Rhai o della CLI. Con `stretch` superiore a 1, la zona soft è la fine della portata allungata. Le catene di due ossa sono risolte in modo analitico; quelle di tre o più usano FABRIK.

**42. I miei vincoli danno risultati strani.**
I vincoli vengono eseguiti nell'ordine dell'elenco, e l'ordine conta. Un aim con peso 0,5 dà metà dell'angolo (un angolo di -74,7 diventa -37,4). Riordina nel JSON se un vincolo successivo deve vedere il risultato di uno precedente.

**43. Esiste uno skinner automatico per la grafica vettoriale?**
Non da riga di comando. `Project::auto_skin` esiste solo nella libreria. Per la grafica vettoriale si fa il rig con ossa legate ai livelli, con lattice, spine o pin (solo JSON), oppure con mesh tracciate tramite `mesh-trace`.

**44. `skin-diffuse` e `cloth` si lamentano della mesh.**
Entrambi richiedono una mesh, e cloth richiede una mesh **a griglia**: `cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Un livello senza alcuna mesh dà `layer 'flag' has no mesh`.

**45. `foot-lock` dice che serve una radice.**
`foot lock needs a root controller bone (no parent)`. L'osso controller radice non deve avere un osso genitore.

**46. `scale` rifiuta le mie chiavi.**
`scale: frame 27 is outside the project range 0..=24` significa che una chiave risultante cadrebbe fuori dal progetto; `keyedit scale` viene rifiutato quando è fuori intervallo a meno che tu non aggiunga `--free`. Una prova a vuoto usa `-` come nome di output.

**47. Perché il mio progetto è diventato più lungo dopo `mocap-retarget`?**
`mocap-retarget` estende la lunghezza del progetto per adattarla al movimento catturato. `follow --chain` può spostare le chiavi oltre l'ultimo fotogramma, cosa che `lint` segnala (L007).

**48. Come animo un livello senza un osso?**
Modifica i canali `layer_x` e `layer_y` del livello. Nello studio attiva l'auto-key; con l'auto-key disattivato una modifica a un livello resta statica a meno che quella proprietà abbia già un canale con chiavi. Le modifiche alle ossa creano sempre chiavi. Vedi [STUDIO.md](STUDIO.md).

---

## Studio

**49. Come apro lo studio?**
`osatura studio project.json` e apri l'indirizzo che stampa, normalmente `http://127.0.0.1:<port>`. È un'app a pagina singola servita dal piccolo server HTTP di OSATURA (solo libreria standard); non richiede internet.

**50. Lo studio modifica il mio file mentre lavoro?**
No. Il file su disco cambia solo quando premi Save. Chiudere la pagina senza salvare fa perdere le modifiche (il log di collaborazione, se abilitato, conserva l'intero progetto, ma non la cronologia di annulla).

**51. Altre persone possono connettersi?**
Solo se colleghi un indirizzo non loopback, e in quel caso serve un token: `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)`. Il token deve avere almeno 16 caratteri, per esempio da `openssl rand -hex 16`. Non c'è crittografia: usa un tunnel SSH o un reverse proxy con TLS.

**52. Dove metto il token?**
Nell'header `X-Collab-Token` o nel cookie `osatura_token` (HttpOnly, SameSite=Strict e Secure quando la richiesta arriva con un `--public-origin` https). Non viene mai accettato nell'URL. Dal quinto tentativo fallito, quell'indirizzo client riceve HTTP 429 con un ritardo crescente (2, 4 ... fino a 30 s).

**53. Che cosa significano gli errori HTTP 403, 415 e 421?**
421: l'header Host non è uno di quelli che lo studio si aspetta. 403: un `Origin` o `Sec-Fetch-Site` estraneo. 415: un POST che non è JSON. Sono protezioni contro le pagine nel tuo browser che chiamano il server locale; vedi [STUDIO.md](STUDIO.md) e [SECURITY.md](SECURITY.md).

**54. Funziona in qualsiasi browser?**
È stato provato solo in Chromium headless. Gli altri non sono testati. Non esiste un comando di zoom del canvas, e nello studio esiste un pannello per il tracciamento del volto da webcam, ma non era mai disponibile una vera fotocamera per provarlo ([LIMITATIONS.md](LIMITATIONS.md)).

---

## Volto, parlato e motion capture

**55. OSATURA traccia i volti?**
C'è un piccolo rilevatore integrato, addestrato solo su volti sintetici. Su 50 scene di test sintetiche (seed 9) ha trovato ogni volto, con un errore medio dei landmark di 0,028 della dimensione del volto, e ha generato 2 falsi allarmi in 50 scene senza volto. Sui disegni in stile cartone animato è andato male: 0 volti su 6 di robot squadrati sono stati trovati, e i casi dell'emoji rotonda e dell'ovale manga hanno avuto errori di 0,164 e 0,166. Non è mai stato eseguito su fotografie reali o su una vera webcam. Nota che [LIMITATIONS.md](LIMITATIONS.md) L-13 e il testo del pannello webcam dello studio dicono che non è incluso alcun rilevatore di volti; ciò è superato rispetto a questo rilevatore sintetico.

**56. Come funziona il lip-sync?**
I cue della bocca scelgono tra cinque forme (riposo, piccola, aperta, rotonda, larga). Un tempo di cue con il punto decimale è in secondi; un intero semplice è un fotogramma. `--rhubarb` cambia solo il modo in cui viene classificata la lettera E (rotonda invece di larga). L'audio più lungo del progetto viene troncato.

**57. Posso animare a partire da un video?**
`face-track` legge il video tramite ffmpeg. `face-bake` scrive i segnali come canali con chiavi (il riepilogo dice "+N channels (K keys in them), M trigger keys set") e tronca alla lunghezza del progetto, con un avviso su stderr che conta i fotogrammi di cui non è stato fatto il bake. Un segnale sconosciuto dà `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.

---

## Bug, sicurezza e contributi

**58. Come segnalo un bug?**
Apri una issue con la riga di comando, l'output esatto (anche stderr), lo stato di uscita, la build (`osatura --version`) e, se puoi, un progetto minimo. Esegui prima `osatura check project.json` e `osatura lint project.json`: molti problemi vengono segnalati lì. Vedi [CONTRIBUTING.md](CONTRIBUTING.md).

**59. Come segnalo un problema di sicurezza?**
In privato, non in una issue pubblica; vedi [SECURITY.md](SECURITY.md). L'indirizzo nel repository è `alexio@trom.tf`; è lo stesso indirizzo del contatto del maintainer e non è stato sostituito da una casella di un team di sicurezza.

**60. Esistono una mailing list, una chat o un contratto di assistenza?**
No. I documenti indicano un solo contatto del maintainer; la prima release è ancora una pre-release in bozza.

**61. Come contribuisco?**
Firma ogni commit (`git commit -s`), mantieni i test superati (`cargo test`, e `osatura ci` per il lavoro sull'animazione) e leggi [DEVELOPMENT.md](DEVELOPMENT.md). La documentazione è CC-BY-4.0 e accoglie volentieri correzioni, soprattutto dove una pagina e il programma non concordano.

---

## Limitazioni

**62. Qual è il singolo limite più importante?**
È stato costruito e testato da un solo progetto in un solo ambiente. Tutto ciò che non è elencato come eseguito è "non testato". Leggi [LIMITATIONS.md](LIMITATIONS.md); contiene id che puoi citare nelle issue.

**63. Quali formati di esportazione richiedono attenzione?**
Il video richiede ffmpeg. Lottie e glTF coprono un sottoinsieme di ciò che OSATURA sa disegnare e animare, quindi controlla cosa si perde in [EXPORT_FORMATS.md](EXPORT_FORMATS.md). L'errore di `export` per un'estensione sconosciuta elenca `.gif .html .json .mp4 .apng .webm .png .glb`, che omette `.lottie`; `.lottie` (dotLottie) è una destinazione valida, vedi [EXPORT_FORMATS.md](EXPORT_FORMATS.md), sezione 6.

**64. La build WASM è utilizzabile?**
Viene compilata per `wasm32-unknown-unknown` su GitHub Actions e il test del wrapper passa sul modulo reale con Node 20; è allegata alla release in bozza come `osatura-web.tar.gz`. Non è stata eseguita in un browser e velocità e memoria non sono state misurate ([WASM.md](WASM.md), [LIMITATIONS.md](LIMITATIONS.md)). Consideralo parzialmente verificato.

**65. Importa i file di progetto di altri strumenti?**
Importa SVG, OpenRaster, JSON Lottie, immagini, GIF/APNG, motion capture BVH e MIDI ([EXPORT_FORMATS.md](EXPORT_FORMATS.md), sezioni 9 e 10). Non importa PSD, file zip `.lottie` o cartelle OCA. Le importazioni creano un nuovo progetto. Nota che `import-lottie in.json in.json` non ha alcuna protezione contro la sovrascrittura e sostituisce il proprio input. Non aspettarti che un file importato corrisponda all'originale pixel per pixel.

**66. Il JSON di progetto è stabile?**
È "format 1". Un file più recente viene rifiutato: `project format 2 is newer than this build supports (1); update OSATURA`. Poiché esiste solo una pre-release alpha, lo stesso format 1 potrebbe ancora cambiare prima di una release stabile; vedi [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

---

## Confronti con altri strumenti

**67. In cosa si differenzia da un editor di animazione desktop basato sulla timeline?**
Quelli sono programmi interattivi, che mettono il disegno al primo posto. OSATURA mette al primo posto i comandi: ogni operazione è un comando o uno script, il progetto è JSON confrontabile con diff e i render sono deterministici, quindi possono essere testati in CI. Lo studio è una sottile vista web sullo stesso motore ed è molto meno rifinito di un editor maturo.

**68. In cosa si differenzia da uno strumento di animazione scheletrica 2D per i giochi?**
Quegli strumenti si concentrano su mesh, skin e librerie runtime per specifici motori di gioco. OSATURA ha anch'essa ossa, IK, vincoli e mesh, ma una storia runtime più ridotta: un proprio player in HTML, una C ABI e un crate WASM mai compilato. Scegli in base al runtime che devi raggiungere.

**69. In cosa si differenzia da un compositor di motion graphics?**
Un compositor è costruito attorno a livelli con effetti ed espressioni su filmati. OSATURA ha effetti di livello ed espressioni, ma il suo centro è il rigging di personaggi e la grafica vettoriale, e non fa compositing video, 3D né color grading di filmati.

**70. Quando OSATURA è la scelta sbagliata?**
Quando ti servono oggi prestazioni in tempo reale su GPU, un flusso di lavoro rifinito con tavoletta grafica, animazione fotogramma per fotogramma disegnata a mano, un prodotto con supporto del fornitore, con una release stabile e assistenza, o un comportamento confermato al di fuori di Linux x86-64.

---

## Tabella di risoluzione dei problemi

Ogni messaggio qui sotto è stato prodotto da una build reale e termina con stato di uscita 1, salvo diversa indicazione. La formulazione di un messaggio può cambiare prima di una release.

| Messaggio | Causa | Soluzione |
|---|---|---|
| `cannot read nope.json: No such file or directory (os error 2)` | Percorso o nome errato | Controlla il percorso e la cartella corrente |
| `invalid JSON in broken.json: EOF while parsing a value at line 2 column 0` | Il file non è un JSON completo | Correggi la sintassi alla riga indicata; ripristina da un backup o da `osatura vcs` |
| `badparent.json is not a valid project:` then `bone 'a' has parent 'ghost' which is missing or listed after it` | Un osso genitore manca o è elencato dopo il proprio figlio | Definisci i genitori prima dei figli; correggi il nome |
| `invalid JSON in fmt2.json: project format 2 is newer than this build supports (1); update OSATURA` | File proveniente da una build più recente | Usa una build più recente |
| `warning: unknown field bogus (ignored)` (exit 0) | Una chiave che questa build non conosce | Rimuovi o correggi la chiave; controlla se c'è un refuso |
| `refusing to overwrite the input project X with output of the same name` | Il nome del file di output coincide con l'input | Scegli un nome di output diverso |
| `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)` | Estensione sbagliata per `render` | Usa `.png`/`.svg`, oppure `export` per gli altri formati |
| `unknown option --bone for 'render' (did you mean --bones?); see: osatura (no arguments) for the usage` | Opzione scritta male | Usa l'opzione suggerita |
| `unknown export format '.bmp' (use .gif .html .json .mp4 .apng .webm .png .glb)` | Estensione non esportabile | Scegli un formato elencato (è valido anche `.lottie`) |
| `--scale needs a number in (0, 16]` | Scala fuori intervallo | Usa un valore maggiore di 0 e fino a 16 |
| `bad frame number 'x'` | Un argomento di fotogramma non è un numero intero | Passa un intero |
| `cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` | ffmpeg non è nel `PATH` | Installa ffmpeg, oppure esporta in GIF, HTML o APNG |
| `the project has no sound tracks (see: sound add)` | Comando audio su un progetto muto | Aggiungi una traccia con `sound add` |
| `channel hand_ctrl.BoneOffsetY is locked` | Il canale è bloccato | Sbloccalo, oppure modifica un altro canale |
| `request 1 (hand_ctrl.BoneOffsetY at 0) was rejected, nothing changed: channel hand_ctrl.BoneOffsetY is locked` | Lo stesso, da `script` | Sblocca prima; non è stato applicato nulla |
| `request 1 (channel head.BoneAngle) was rejected, nothing changed: ... expression error: unknown variable 'tt'` | Refuso in un'espressione | Le variabili sono frame, time, fps, frames, pi, t |
| `... unexpected end of expression` | Espressione incompleta | Chiudi le parentesi, completa la formula |
| `points need an even number of coordinates: 3..1000 points` | Numero di coordinate dispari o troppo pochi punti | Fornisci coppie x,y: da 3 a 1000 punti |
| `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)` | Process substitution o pipe come input | Salva prima in un file regolare |
| `sound add: asset '/etc/passwd': absolute paths are not allowed (...)` | Asset fuori dalla cartella del progetto | Copialo accanto al progetto, oppure passa `--allow-external-assets` |
| `cannot read demo.json: N bytes exceed the input limit of M bytes (raise it with --max-input-bytes N)` | File oltre il limite di input | Alza `--max-input-bytes` se ti fidi del file |
| `refusing to write through a symbolic link; pass --follow-symlinks ...` | L'output è un collegamento simbolico | Indica il file di destinazione, oppure passa `--follow-symlinks` |
| `cloth needs a grid mesh; the layer has a free-form (traced) mesh` | Cloth su una mesh tracciata | Costruisci una mesh a griglia per il livello |
| `layer 'flag' has no mesh` | Un comando per mesh su un livello semplice | Crea prima una mesh |
| `no bone 'b9'` | Il nome dell'osso non esiste | Controlla i nomi con `stats` o `explain` |
| `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)` | Nome del segnale errato | Usa un nome dell'elenco |
| `foot lock needs a root controller bone (no parent)` | La radice scelta ha un genitore | Usa un controller senza genitore |
| `scale: frame 27 is outside the project range 0..=24` | Le chiavi scalate cadono fuori dal progetto | Allunga il progetto, scala meno, oppure usa `--free` |
| `--extrapolate must be hold, linear, loop, ping_pong or loop_offset` | Modalità errata | Usane una delle cinque |
| `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)` | Porta occupata | Scegli un'altra `--port` oppure ferma l'altro processo |
| `cannot listen on 127.0.0.1:99999: invalid port value` | Porta oltre 65535 | Usa da 1 a 65535 |
| `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (...)` | Binding pubblico senza token | Aggiungi `--collab-token`, oppure resta sul loopback |
| `--collab-token must be at least 16 characters (e.g. \`openssl rand -hex 16\`)` | Token debole | Genera un token più lungo |
| `--source-url must be an http:// or https:// URL without spaces or quotes` | URL malformato | Usa un URL semplice |
| `--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org` | L'origin ha un percorso | Indica solo schema, host e porta |
| `output would be invalid: ...` | La modifica lascerebbe un progetto non valido | Leggi i motivi dopo i due punti; non è stato scritto nulla |
| `budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes ...` | `export-fit` non riesce a raggiungere il budget | Alza il budget o semplifica il progetto (il file più piccolo viene comunque scritto) |
| `time budget exhausted (60 s ...)` | `export-fit` ha esaurito il tempo | Alza `--max-seconds` |
| `fuzz: failure rate 98.00% exceeds --max-fail-rate 50.00%` | Le pose hanno rotto il rig e hai chiesto una soglia di blocco (`--gate` o `--max-fail-rate`) | Guarda i casi che falliscono; alza la soglia solo se voluto; `fuzz` semplice si limita a riportare (exit 0) |
| `no .osatura-vcs repository found from ... (run: osatura vcs init <project.json>)` | Ancora nessuna cronologia | Esegui `vcs init` |
| `frame 999 out of range` (studio API) | Fotogramma oltre il progetto | Usa un fotogramma da 0 all'ultimo |
| `bad kind 'x' (vector, raster, png)` (studio API) | Tipo errato | Usa un tipo elencato |
| `nothing selected` (studio API) | Un comando di selezione senza selezione | Seleziona prima livelli od ossa |

I codici di uscita di `osatura ci` sono 0 superato, 1 fallito, 2 errore.

---

*Documentazione CC-BY-4.0. Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio). Puoi condividerla e adattarla citando l'autore. Vedi anche: [INDEX.md](INDEX.md), [GLOSSARY.md](GLOSSARY.md).*
