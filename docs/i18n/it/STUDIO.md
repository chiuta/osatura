# Lo studio di OSATURA

Lo studio è un editor nel browser servito dallo stesso binario `osatura`: `osatura studio project.json`.
Ha un canvas con gizmo di selezione e trasformazione, un elenco dei livelli, un elenco delle ossa, un dope sheet e un grafico delle curve,
onion skin e traiettorie di movimento, una palette dei comandi con scorciatoie riassegnabili, suggerimenti di pose, pannelli per l'input dal vivo e
per il volto, pulsanti di esportazione GIF/HTML/Lottie e una sessione di modifica condivisa tra più utenti. Non ha account, né TLS, né
database: è un piccolo server HTTP basato sulla sola libreria standard, posto davanti allo stesso editor che usa la riga di comando.

Tutto ciò che è scritto qui è stato verificato con il binario reale (`osatura 0.1.0`, build di debug, Linux) e, per l'interfaccia
utente, in Chromium headless pilotato da Playwright. Gli screenshot sono stati realizzati così (1400x900, il progetto demo
integrato). È stato provato solo **Chromium**; vedi la sezione 11. Su Windows e macOS gli archivi di release contengono anche `osatura-studio`, lo stesso studio in una finestra nativa (compilato e auto-testato in CI, non ancora usato da una persona; vedi [GETTING_STARTED.md](GETTING_STARTED.md) e [LIMITATIONS.md](LIMITATIONS.md) L-75), e su Windows un doppio clic su `osatura.exe` avvia lo studio e apre il browser (L-76).

Licenza di questo documento: CC-BY-4.0. Per la riga di comando vedi [GETTING_STARTED.md](GETTING_STARTED.md) e
[CLI_REFERENCE.md](CLI_REFERENCE.md); per la sicurezza del server vedi anche [SECURITY.md](SECURITY.md).

![Lo studio con il progetto demo: barra degli strumenti in alto, il canvas con un livello braccio selezionato e le sue ossa disegnate, il pannello Layers a destra e sotto la timeline con i rombi delle chiavi.](../../img/studio-overview.png)

## 1. Avviare lo studio

```
osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES]
                [--public-origin URL] [--source-url URL] [--collab-token T] [--collab-log LOG [--collab-nosync]]
```

```
$ osatura demo demo.json
$ osatura studio demo.json --port 8080
AGPL-3.0 source offer: http://127.0.0.1:8080/source
OSATURA studio on http://127.0.0.1:8080  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in /home/you/.config/osatura/ui.json      (your own home directory)
```

Apri l'indirizzo in un browser. Il progetto viene validato all'avvio; un file che non supera `check` viene rifiutato con lo
stesso messaggio. I campi sconosciuti vengono segnalati con un avviso (`warning: unknown field bogus (ignored)`).

| Flag | Predefinito | Significato |
|---|---|---|
| `--port N` | 8080 | Porta TCP. Una porta occupata dà `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)`; una non valida `cannot listen on 127.0.0.1:99999: invalid port value`. |
| `--bind ADDR` | `127.0.0.1` | Interfaccia su cui ascoltare. Qualsiasi indirizzo che non sia di loopback **richiede `--collab-token`**. |
| `--ui-config F` | `$OSATURA_UI_CONFIG`, altrimenti `$XDG_CONFIG_HOME/osatura/ui.json`, altrimenti su Windows `%APPDATA%\OSATURA\ui.json` (prima di alpha.3 non ce n'era uno: le impostazioni venivano conservate solo per la sessione), altrimenti `~/.config/osatura/ui.json` | Dove vengono conservati layout, scorciatoie, comandi recenti e impostazioni degli overlay. È un file per utente, mai parte del progetto. |

L'applicazione a finestra e l'avvio con doppio clic creano il primo progetto in `OSATURA_DOCUMENTS` se la variabile è impostata, altrimenti in `Documents\OSATURA` (usato dal pacchetto portatile).
| `--allow-host NAMES` | nessuno | Nomi `Host` aggiuntivi da accettare (separati da virgola), per l'accesso tramite un nome DNS. |
| `--public-origin URL` | nessuno | L'origin semplice (`https://studio.example.org`, senza percorso) presentato da un reverse proxy TLS; il suo `Host` e `Origin` vengono quindi accettati. |
| `--source-url URL` | nessuno; nei binari di release ufficiali il commit da cui sono stati compilati (`https://github.com/chiuta/osatura/tree/<commit>`) | Dove si può scaricare il sorgente corrispondente alla *tua* build (AGPL sezione 13); mostrato su `/source` e da `--version`. Un valore esplicito ha sempre la precedenza. |
| `--collab-token T` | nessuno | Segreto condiviso, **di almeno 16 caratteri** (`openssl rand -hex 16`). Obbligatorio per i bind non loopback. |
| `--collab-log LOG` | nessuno | Rende persistente la sessione di modifica condivisa su un file e la riprende dopo un riavvio. |
| `--collab-nosync` | off | Con `--collab-log`: non esegue `fsync` dopo ogni modifica (più veloce; un'interruzione di corrente può far perdere le ultime modifiche). Senza `--collab-log` viene rifiutato all'avvio (`--collab-nosync needs --collab-log`). |

**Salvataggio.** Le modifiche vivono in memoria. Il file di progetto su disco cambia solo quando premi **Save** (`Ctrl+S`), che scrive in modo atomico
(file temporaneo e poi rinomina, mai attraverso un collegamento simbolico). Questo vale anche durante una sessione condivisa (sezione 8).

## 2. Modello di sicurezza (leggi prima di fare il bind su una rete)

Lo studio è pensato per una persona sulla propria macchina. Ciò che lo protegge:

* **Lista di host consentiti.** Si risponde solo alle richieste il cui header `Host` è uno dei nomi propri del server; tutto il resto riceve **421**.
  Con `--allow-host` e `--public-origin` vengono accettati altri nomi.
* **Protezione cross-site.** Una richiesta con un `Origin` estraneo, o con `Sec-Fetch-Site: cross-site`, riceve **403**. Un POST il cui corpo non è
  `application/json` riceve **415** (i POST senza corpo come `/api/undo` non richiedono un tipo).
* **Header di sicurezza** nelle risposte: una `Content-Security-Policy` restrittiva (`default-src 'self'` e nessun framing), `X-Frame-Options: DENY`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`.
* **Token per tutto ciò che è fuori dalla macchina.** Con `--collab-token`, ogni richiesta richiede il token come header `X-Collab-Token`, oppure nel
  cookie `osatura_token` impostato da `POST /api/login` (HttpOnly, SameSite=Strict). Il token **non viene mai accettato nell'URL**. I token errati vengono rallentati
  per indirizzo client (dal quinto fallimento di fila quell'indirizzo riceve **429** per 2, 4, ... fino a 30 secondi), ma il back-off non può fermare un
  attaccante che continua a provare: usa un token casuale lungo.
* **Segreti dei partecipanti.** Entrare in una sessione condivisa restituisce un segreto casuale a 128 bit; ogni chiamata fatta come quel partecipante deve inviarlo come
  `X-Collab-Secret`. Conoscere il numero di un partecipante non basta.

Ho verificato ciascuna di queste protezioni con `curl` su un server in esecuzione:

```
bad Host (evil.example):               421
allowed host (--allow-host):           200
cross-origin POST (Origin: evil.example): 403
POST from the --public-origin:         200
Sec-Fetch-Site: cross-site:            403
POST with Content-Type: text/plain:    415
no token, GET / or /api/state:         401
wrong X-Collab-Token:                  401
right X-Collab-Token:                  200
token in the URL (?token=...):         401
/source without a token:               200
unknown route:                         404   (GET /favicon.ico answers with a small SVG icon, without a token)
```

Rifiuti all'avvio (codice di uscita 1, non viene avviato nulla):

```
refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)
--collab-token must be at least 16 characters (e.g. `openssl rand -hex 16`)
--source-url must be an http:// or https:// URL without spaces or quotes
--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org
```

**Limiti dichiarati con onestà.** Il traffico è HTTP in chiaro: il token e tutto il resto viaggiano in chiaro, quindi metti lo studio dietro un reverse proxy TLS
(`--public-origin`) se è raggiungibile da chiunque altro. Il `Set-Cookie` del token riceve il **flag `Secure`** quando la richiesta arriva con un nome `--public-origin` `https://` (o con un `Origin` https / `X-Forwarded-Proto: https`); l'HTTP semplice su loopback non lo riceve mai,
e dietro un proxy ogni client appare con l'indirizzo del proxy, quindi il back-off rallenta tutti contemporaneamente. `/source` è aperto senza token
(lo richiede l'AGPL). Non ci sono account utente: chi ha il token può modificare il progetto e scrivere nel percorso di salvataggio.

### L'offerta del sorgente AGPL

Lo studio è software AGPL-3.0-or-later: se lo modifichi e permetti ad altre persone di usarlo tramite una rete, devi offrire loro il sorgente della
tua versione. `/source` è quell'offerta; imposta `--source-url` su dove si trova il sorgente della tua build. Senza di esso (una build che hai compilato tu con un semplice `cargo build` non ne ha) la pagina lo dice e indica il maintainer. I binari di release compilati da `.github/workflows/release.yml` riportano l'indirizzo del proprio commit (`OSATURA_SOURCE_URL` in fase di compilazione; un fork che compila con il proprio workflow punta a se stesso).

## 3. Un giro dell'interfaccia

### Barra degli strumenti

Annulla, Ripeti, **Save**; caselle `bones` (disegna lo scheletro), `auto-key`, `grid` (spaziatura in unità del canvas, 0 = disattivata), `snap` (agli altri livelli e al canvas),
`whole groups` (un clic su una parte di un gruppo seleziona il gruppo di primo livello), `points` (mostra ancore, maniglie, punti di cage/lattice/pin e vertici delle mesh dei
livelli selezionati); un selettore `box` per ciò che seleziona un riquadro trascinato (`touches`, `contains`, `bounds touch`; parte da `touches`); **Lock**; e i pulsanti **Overlays**, **Suggest**, **Layout**,
**Commands** (palette) e **Keys** (editor delle scorciatoie). Una seconda riga contiene il tuo nome e **Collab**, e i pulsanti di esportazione **GIF**, **HTML player** e **Lottie**,
che scaricano ciò che scriverebbe `osatura export` (il player HTML include il suono mixato quando c'è audio e `ffmpeg` può produrre l'mp3). Il pulsante **Simple view** nasconde le caselle di modifica, Overlays, Suggest e il pulsante della palette dei comandi, così restano solo annulla, salva, pannelli ed esportazioni; la scelta viene ricordata in questo browser (o finestra) e la barra completa torna con **Full view**.

### Pannelli e layout

La finestra è un dock di pannelli, ciascuno con un id stabile: `stage` (il canvas), `layers`, `selection`, `bones`, `live` (Live input), `face` (Webcam face), `json`
(l'intero progetto come JSON modificabile con un pulsante *Apply JSON*) e `timeline`. Esistono quattro preset di layout:

```
animate  stage over timeline on the left, all panels as tabs on the right
rig      stage with a folded timeline; layers+bones and selection+json on the right (live and face hidden)
compact  stage over a tabbed timeline and panels
live     stage and timeline on the left; live, face, bones and selection on the right (layers and json hidden)
```

Cambia preset dal menu **Layout** o dalla palette; trascina le schede per riagganciare o rendere fluttuante un pannello; "reset layout" ripristina quello predefinito.
Il layout viene salvato nella configurazione dell'interfaccia, per utente. `osatura ui layout presets|show NAME|validate FILE.json` stampa e valida gli stessi dati
(un layout deve mantenere raggiungibile ogni pannello; uno incompleto viene rifiutato dal server con 400).

### Pannello Layers

![Il pannello Layers con un livello bloccato (sun), un livello nascosto (hills) e arm e leg selezionati insieme.](../../img/studio-layers.png)

Ogni riga ha due caselle: **visible** e **lock** (salvata *con il progetto*; i livelli bloccati vengono saltati da clic e riquadri e rifiutano le modifiche).
Lo screenshot mostra `sun` bloccato, `hills` nascosto e due livelli selezionati con Shift-clic.

### Pannello Selection

Riporta le regole nella pagina stessa: *Il clic seleziona l'elemento più in alto (quelli bloccati e nascosti vengono saltati); Shift-clic attiva o disattiva; trascinare su uno spazio vuoto
disegna un riquadro (tieni premuto Alt per disegnare un lazo a mano libera, Shift aggiunge). Trascina il riquadro per spostare, un quadrato per scalare (Shift mantiene le proporzioni, Alt scala dal
centro), la manopola rotonda per ruotare (Shift aggancia a 15 gradi). Tieni premuto Ctrl durante il trascinamento per disattivare l'aggancio. I tasti freccia spostano di 1 unità (Shift: 10),
Delete rimuove i livelli, Esc annulla un trascinamento o azzera la selezione.*

### Timeline: dope sheet e grafico

![Il dope sheet con una selezione a riquadro di tre chiavi al fotogramma 12 (le chiavi sono disegnate piene, le altre vuote).](../../img/studio-dopesheet.png)

![La vista grafico delle curve: tre curve bone_angle (arm_up, arm_low, hand) in colori diversi con tre chiavi selezionate al fotogramma 12 che mostrano le loro maniglie tangenti; la riga di stato dice "3 keys selected | frames 12".](../../img/studio-graph.png)

`Graph` (o `Alt+V`) alterna tra il dope sheet e il grafico delle curve. I controlli della timeline: `Play`, `|<` (primo fotogramma dell'intervallo), `loop`, `range` con
`Set in` / `Set out` (riproduce solo tra in e out), `snap` (arrotonda ai fotogrammi interi le chiavi spostate), `on collision` (`replace`, `merge`, `refuse` quando le chiavi spostate finiscono su
chiavi esistenti), `Copy` / `Paste` / `Duplicate` / `Delete`, `interp` + `Apply` (imposta l'easing delle chiavi selezionate), `Auto-smooth` (esegue il bake di tangenti senza overshoot nelle
chiavi selezionate), `ripple` N con `Insert` / `Remove` (e `grow project` per cambiare con essi la lunghezza del progetto), zoom `-` `+` `Fit`, e `shy` (mostra i canali contrassegnati come shy).

Mouse: clic su una chiave per selezionarla, Shift-clic per aggiungere, trascina un riquadro per selezionare, trascina le chiavi per spostarle; **Alt-trascinamento o il pulsante centrale fanno scorrere la vista (pan)**, **Ctrl+rotellina esegue lo zoom**,
**Shift+rotellina scorre** in verticale; nel grafico **Shift vincola il trascinamento della chiave a un solo asse**. La tolleranza di snap per l'aggancio al canvas è di 6 unità del canvas per impostazione predefinita.
Ogni modifica alle chiavi passa attraverso le stesse operazioni validate di `osatura keyedit` (vedi [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)), quindi è un solo passo di annulla e i canali bloccati la rifiutano.

### Overlay: onion skin, traiettorie, proxy

![Onion skin e traiettorie di movimento attivati per il braccio al fotogramma 14, con il pannello Overlays aperto.](../../img/studio-overlays.png)

`Overlays` (`Alt+7`) contiene l'onion skin (`Alt+N`: fotogrammi prima/dopo con tinte rossa/verde, passo fisso o fotogrammi chiave, modalità contorno, decadimento, solo livelli selezionati),
le traiettorie di movimento (`Alt+M`: percorsi di ossa, pivot di livello o punti di controllo con segni di graduazione per fotogramma) e i **proxy di scrub** (`Alt+Q`: immagini a qualità ridotta mentre trascini la
testina di riproduzione, con una cache di immagini e una copia opzionale a livello di dettaglio). Il pannello riporta, per esempio, `picture cache: 2 frames, 17 KiB, 0 hits / 7 misses, emptied 4x by edits`.
Le impostazioni sono memorizzate nella configurazione dell'interfaccia e sopravvivono a un riavvio. Gli overlay non fanno mai parte delle esportazioni.

### Palette dei comandi e scorciatoie

![La palette dei comandi aperta con Ctrl+K e filtrata con la parola "toggle", che elenca i comandi corrispondenti con i suggerimenti delle loro scorciatoie.](../../img/studio-palette.png)

`Ctrl+K` (o `Ctrl+Shift+P`) apre la palette: digita per filtrare (fuzzy, lettere corrispondenti evidenziate), i tasti freccia si spostano, Invio esegue, Esc chiude. Senza digitare nulla
elenca prima i comandi recenti. Ci sono **84 comandi**; `osatura ui commands` li elenca e `osatura ui search onion` li ordina esattamente come fa la palette.

![L'editor delle scorciatoie da tastiera aperto con Ctrl+virgola, che elenca i comandi per categoria con le loro combinazioni di tasti.](../../img/studio-shortcuts.png)

`Ctrl+,` (o **Keys**) apre l'editor delle scorciatoie: clicca una combinazione, premi la nuova combinazione di tasti. Un conflitto indica l'altro comando e non cambia nulla finché non
confermi "Take it over". Le combinazioni riservate dal browser vengono rifiutate: `Ctrl+W`, `Ctrl+T`, `Ctrl+N`, `Ctrl+Q`, `Ctrl+R`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+Shift+W`, `Ctrl+Shift+N`, `F5`, `F11`, `F12`,
`Alt+F4`. La riassegnazione è memorizzata nella configurazione dell'interfaccia e sopravvive a un ricaricamento. Il server rifiuta con 400 una mappa con conflitti, un comando sconosciuto, un layout incompleto o un corpo che non è un oggetto.

### Suggest

![Il pannello Suggest che mostra tre pose intermedie candidate ordinate (linear, pca, manifold) tra i fotogrammi 12 e 24, ciascuna con un punteggio e un pulsante Apply.](../../img/studio-suggest.png)

`Suggest` (`Alt+8`) propone pose apprese dalle pose chiave del progetto stesso: un'intercalazione tra due fotogrammi, il completamento di una posa parzialmente fissata, oppure "ciò che di solito
viene dopo". Sul progetto demo, tra i fotogrammi 12 e 24 ha ordinato `linear` 1,00, `pca` 1,00 e `manifold` 0,99, costruiti da 5 pose su 2 assi (99 % della varianza).
`Apply` scrive la posa come chiavi, in un solo passo di annulla. Lo stesso motore è `osatura pose-suggest` e `/api/suggest`. Sono metodi classici (fusione lineare, PCA, un manifold
a kernel, una piccola rete opzionale che addestri tu); con una manciata di pose i tre concordano.

### Pannelli Live input e Webcam face

* **Live input** (`Alt+4`): mouse sopra l'immagine, tasti, gamepad e Web MIDI sono associati alle proprietà tramite una tabella; `Record` esegue il bake di una performance in chiavi come un solo passo di
  annulla. Web MIDI richiede un browser che lo supporti (Chromium lo fa).
* **Webcam face** (`Alt+5`): i landmark pilotano la stessa tabella di mappatura (sorgente `face`, id come `mouth_open`, `eye_r_open`, `head_yaw`). I pulsanti sono
  *Start camera*, *Synthetic face*, *Synthetic face (detector)*, *Calibrate (hold neutral)* e *Face example mapping*. Il percorso della fotocamera richiede una funzione fornitrice di landmark
  che fornisci tu (`window.osaturaLandmarkProvider`, vedi il commento nel sorgente della pagina); il video non lascia mai la pagina e al server locale arrivano solo i landmark.
  Le due modalità sintetiche funzionano subito. *Synthetic face (detector)* renderizza la testa sintetica sul server e la ritrova con il rilevatore integrato,
  che è stato **addestrato solo su volti sintetici** (teste stilizzate, non fotografie); vedi la sezione sul volto di [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Demo guidata

Il pulsante **Demo** (comando `demo.panel`) apre una demo guidata in 13 passi che costruisce la persona che cammina di `osatura demo-person`: scenografia, corpo, testa e volto, braccia con cinque dita per mano, lo scheletro, le gambe che camminano, le braccia che oscillano, tronco e testa, un ammiccamento e un sorriso, il terreno in movimento, un controllo e l'esportazione. Ogni passo carica quella fase del progetto nell'editor con una sola operazione di sostituzione completa, così puoi riprodurla, scorrerla, selezionare ossa e livelli e leggere la timeline. Le fasi sono tagli del progetto finito; i comandi e il loro output mostrati nel pannello sono stati eseguiti con il binario reale.

* **Nulla viene scritto su disco.** Salva (e Ctrl+S) è disattivato mentre la demo è caricata. L'unico percorso di scrittura dello studio, `POST /api/save`, non viene mai chiamato dalla demo.
* **Il tuo progetto ritorna.** *Restore my project* rimette il progetto che era aperto prima (una sola operazione di sostituzione completa, un solo passo di Annulla). Se chiudi la scheda o ricarichi mentre la demo è caricata, al caricamento successivo la pagina nota che il server contiene ancora la demo, mantiene Salva disattivato e offre di nuovo *Restore my project*; l'originale viene conservato per questo nella memoria locale del browser.
* La demo si rifiuta di partire all'interno di una sessione di collaborazione, perché sostituisce l'intero progetto.
* Rigenera le fasi con `node scripts/build-studio-tour.mjs --bin target/release/osatura`, poi ricompila la CLI (la pagina è incorporata nel binario).

## 4. Scorciatoie da tastiera (predefinite)

Questo è l'output di `osatura ui shortcuts` per una configurazione nuova: **72 associazioni, 84 comandi, 0 conflitti**. Le scorciatoie appartengono a un **contesto**:
`global` (ovunque), `canvas` (focus sull'immagine) e `timeline` (focus sulla timeline). I campi di testo mantengono i propri tasti.

### Globali

| Scorciatoia | Comando | Scorciatoia | Comando |
|---|---|---|---|
| `Space` | riproduci / pausa (`playback.toggle`) | `Alt+B` | attiva/disattiva le ossa |
| `Home` / `End` | primo / ultimo fotogramma | `Alt+G` | attiva/disattiva i gruppi interi |
| `,` / `.` | fotogramma precedente / successivo | `Alt+K` | attiva/disattiva l'auto-key |
| `Shift+,` / `Shift+.` | indietro / avanti di 10 fotogrammi | `Alt+M` | attiva/disattiva le traiettorie di movimento |
| `[` / `]` | imposta l'inizio / la fine dell'intervallo | `Alt+N` | attiva/disattiva l'onion skin |
| `Alt+R` | attiva/disattiva l'intervallo di riproduzione | `Alt+O` | attiva/disattiva la riproduzione in loop |
| `Alt+1` | mostra/nascondi il pannello Layers | `Alt+P` | attiva/disattiva i punti di controllo |
| `Alt+2` | mostra/nascondi il pannello Selection | `Alt+Q` | attiva/disattiva i proxy di scrub |
| `Alt+3` | mostra/nascondi il pannello Bones | `Alt+S` | attiva/disattiva lo snap del canvas |
| `Alt+4` | mostra/nascondi il pannello Live input | `Alt+T` | mostra/nascondi il pannello Timeline |
| `Alt+5` | mostra/nascondi il pannello Webcam face | `Alt+V` | alterna dope sheet / grafico |
| `Alt+6` | mostra/nascondi il pannello JSON | `Alt+Shift+S` | attiva/disattiva lo snap della timeline |
| `Alt+7` | pannello Overlays | `Alt+=` / `Alt+-` | zoom avanti / indietro della timeline |
| `Alt+8` | pannello Suggest | `Ctrl+S` | salva |
| `Ctrl+K` / `Ctrl+Shift+P` | palette dei comandi | `Ctrl+,` | scorciatoie da tastiera |
| `Ctrl+Z` | annulla | `Ctrl+Y` / `Ctrl+Shift+Z` | ripeti |

### Canvas

| Scorciatoia | Comando |
|---|---|
| `Arrow keys` | sposta la selezione di 1 unità (`Shift`: 10 unità) |
| `Delete` / `Backspace` | elimina la selezione |
| `Escape` | azzera la selezione |
| `Ctrl+A` | seleziona tutto |
| `Alt+L` | blocca / sblocca i livelli selezionati |
| `I` | aggiunge una chiave alla testina di riproduzione (`anim.add_key`) |

### Timeline

| Scorciatoia | Comando |
|---|---|
| `ArrowLeft` / `ArrowRight` | fotogramma precedente / successivo (`Shift`: 10) |
| `-` / `=` | zoom indietro / avanti |
| `F` | zoom per adattare |
| `Delete` / `Backspace` | elimina le chiavi selezionate |
| `Escape` | deseleziona le chiavi |
| `Ctrl+A` | seleziona tutte le chiavi |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | copia / incolla / duplica le chiavi |
| `Alt+ArrowLeft` / `Alt+ArrowRight` | sposta le chiavi selezionate di un fotogramma prima / dopo |
| `Alt+ArrowUp` / `Alt+ArrowDown` | alza / abbassa le chiavi selezionate (il loro valore) |
| `I` | aggiunge una chiave alla testina di riproduzione |

Modificatori sul canvas: **Shift** vincola (blocco dell'asse durante lo spostamento, passi di 15 gradi durante la rotazione, proporzionale durante la scala, la punta di un osso allora ruota soltanto,
e Shift-clic attiva o disattiva la selezione mentre un riquadro la amplia); **Alt** scala dal centro (e Alt-trascinamento su uno spazio vuoto disegna un lazo); **Ctrl** (Cmd su macOS)
disattiva l'aggancio.

## 5. Selezione e gizmo

Un clic seleziona l'elemento più in alto sotto il puntatore: un livello, un osso (la sua capsula, un po' più larga della linea disegnata) oppure, con `points` attivo, un punto di controllo. I livelli bloccati e nascosti
vengono saltati, quindi un clic passa attraverso verso ciò che sta sotto. Un riquadro seleziona secondo la modalità del selettore `box`: `touches` (tutto ciò che il riquadro sovrappone), `contains` (solo ciò che sta completamente all'interno) o `bounds touch` (i bounding box si sovrappongono); Alt-trascinamento disegna invece un lazo.

Per un solo livello il riquadro di trasformazione segue gli assi propri del livello: 8 maniglie scalano (i bordi un asse, gli angoli entrambi; trascinando oltre il bordo opposto si ribalta con una scala negativa),
la manopola rotonda ruota attorno al centro e trascinare all'interno sposta. Per un osso, il cerchio della testa sposta l'osso e la punta lo ruota (e lo allunga a meno che non si tenga premuto Shift).
Per i punti di controllo (ancore, maniglie di Bézier, punti dei deformatori, vertici delle mesh) lo stesso gizmo sposta i punti selezionati. L'aggancio copre una griglia (disattivata per impostazione predefinita), i bordi e i centri
degli altri livelli e i bordi del canvas, entro 6 unità del canvas. **Un trascinamento è un passo di annulla** (un'operazione batch).

## 6. Auto-key: esattamente cosa riceve le chiavi

Se una modifica sul canvas cambia un valore *statico* o crea *chiavi* lo si decide così. L'ho verificato con l'API HTTP sul progetto demo (viene riportato il `mode` di ogni modifica):

| Situazione | Risultato |
|---|---|
| Modifica di un livello, `auto-key` disattivato, la proprietà non ha un canale con chiavi | cambiamento **statico** della trasformazione del livello (non viene creato alcun canale) |
| Modifica di un livello, `auto-key` attivo | le proprietà modificate ricevono **chiavi** al fotogramma corrente (sono stati creati nuovi canali `layer_x` e `layer_y`: da 14 a 16 canali) |
| Proprietà di un livello o di un punto che **ha già un canale con chiavi**, `auto-key` disattivato | riceve comunque una **chiave** al fotogramma corrente (una modifica al fotogramma 9 ha aggiunto una seconda chiave ai canali dal fotogramma 5) |
| Modifica di un **osso** | sempre **con chiavi**, qualunque sia l'impostazione di `auto-key` (sono stati creati canali di scostamento dell'osso: da 16 a 18) |

Quindi una proprietà animata non può mai essere sovrascritta in silenzio da un valore statico, e la posa di un osso non viene mai "persa" per aver dimenticato l'interruttore. Attiva `auto-key` (`Alt+K`)
quando vuoi animare una proprietà ancora statica. Il tasto `I` (`anim.add_key`, "Add key at playhead") aggiunge una chiave al fotogramma corrente.

## 7. Annulla e ripeti

Annulla e ripeti (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) agiscono su operazioni intere: un trascinamento di gizmo, uno spostamento, una modifica di chiavi e un Apply sono ciascuno un passo. Senza sessione condivisa
sono lineari. **In una sessione condivisa l'annulla è per utente**: annulla solo la tua ultima modifica e lascia al loro posto le modifiche degli altri (sezione 8). La cronologia dell'annulla non viene salvata con il
progetto e va persa quando il server si ferma.

## 8. Modifica condivisa (collaborazione)

### Procedura guidata: due persone, un progetto

![Due utenti del browser in una sessione: la barra degli strumenti mostra due chip di partecipanti, Ana (tu) al fotogramma 14 e Bo al fotogramma 0, e il riquadro di selezione e il cursore di Bo sono disegnati sul canvas attorno al livello leg.](../../img/studio-collab.png)

1. Avvia lo studio. Per le persone su altre macchine devi aggiungere un token ed eseguire il bind a un indirizzo di rete:
   `osatura studio demo.json --bind 0.0.0.0 --port 8080 --collab-token $(openssl rand -hex 16) --collab-log session.log`,
   preferibilmente dietro un proxy TLS (sezione 2). Su una sola macchina non serve alcun token.
2. Nel primo browser, scrivi il tuo nome accanto a **Collab** ("Ana") e premi **Collab**. Il pulsante diventa **Leave** e compare un chip per te. Nel secondo browser
   scrivi "Bo" e partecipa. Ogni chip mostra il fotogramma corrente del partecipante (`Ana (you) @14`, `Bo @0`).
3. Seleziona delle cose. L'altra persona vede il tuo riquadro di selezione e il tuo cursore nel tuo colore con il tuo nome sopra (Bo ha selezionato `leg` e ha mosso il puntatore; Ana vede un riquadro viola
   con il nome "Bo" e un punto accanto a `leg` nell'elenco dei livelli).
4. Modifica nello stesso momento. Nel test: Ana ha spostato il livello braccio di 10 unità (`Shift+ArrowRight`); Bo ha visto comparire `[10,0]` in `/api/project`. Bo ha poi nascosto il livello `sun`, cosa che Ana ha vista
   subito.
5. Ana ha premuto `Ctrl+Z`. **È stato annullato solo lo spostamento di Ana** (il braccio è tornato a `[0,0]`); il sole nascosto da Bo è rimasto nascosto.
6. Le modifiche fatte mentre qualcuno lavorava da una vista più vecchia vengono fuse, non rifiutate: ho eseguito la simulazione del progetto stesso (`osatura collab-sim`, 6 repliche, 600 modifiche
   concorrenti casuali, 12 altri ordini di consegna) e tutte le repliche sono finite identiche e il progetto fuso si è compilato.
7. Premi **Save** (o `Ctrl+S`) in uno dei due browser per scrivere lo stato condiviso nel file di progetto; prima di allora non viene scritto nulla nel file.

È stato provato solo Chromium, con due pagine su una sola macchina. Non ho provato macchine diverse, collegamenti lenti o più di due browser reali.

### Persistenza: `--collab-log`

```
$ osatura studio demo.json --collab-log session.log
...
$ # (an edit is made in a session, the server is stopped and started again)
collaboration log session.log: resumed at index 1 (1 envelopes after the checkpoint)
```

Il log è un file a righe (modalità `0600`, solo per il proprietario) che contiene un checkpoint con catena di hash dell'**intero progetto** (circa 178 KB per la demo) più una riga per ogni modifica accettata. Le aggiunte vengono
sottoposte a `fsync` a ogni richiesta, a meno di `--collab-nosync`. Dopo un riavvio lo stato della sessione viene ripreso (ho cambiato il nome del progetto in una sessione; dopo il riavvio `/api/project` aveva ancora il
nuovo nome) mentre il file su disco è rimasto invariato (`OSATURA demo`) fino a Save. **I partecipanti e la cronologia dell'annulla non sono persistenti**: le pagine si uniscono di nuovo con nuovi id. Un log danneggiato o un log
di un altro progetto ferma il server con un errore anziché essere sovrascritto. Un crash durante un'aggiunta fa perdere al massimo un'ultima riga troncata.

**Il log contiene l'intero progetto in chiaro; proteggilo come il progetto stesso.**

## 9. Riferimento dell'API HTTP

La pagina dello studio usa queste route e puoi usarle anche tu (script, test). Tutti i corpi e le risposte sono JSON, salvo indicazione contraria; gli errori sono `{"ok": false, "errors": ["..."]}` con stato 400 (oppure 401, 403,
404, 415, 421, 429, 503 come descritto nella sezione 2). Le risposte di successo contengono `"ok": true` dove indicato. Qui sotto `state` indica il documento restituito da `GET /api/state`:
`{bones:[{name,rest_angle}], channels:[...], fps, frames, width, height, can_undo, can_redo, rev, ok}`; `rev` aumenta a ogni modifica. Ogni route è soggetta ai controlli della sezione 2.

### Lettura

| Route | Parametri | Risposta |
|---|---|---|
| `GET /` | | la pagina dello studio (HTML) |
| `GET /source` | | la pagina con l'offerta del sorgente AGPL; aperta senza token |
| `GET /api/state` | | `state` |
| `GET /api/project` | | l'intero JSON del progetto |
| `GET /api/layers` | | `{ok, layers:[{path, depth, kind, visible, locked, locked_eff, reference, ptr}]}` (`locked_eff` conta anche i gruppi bloccati più in alto) |
| `GET /frame/N.svg` | `bones=1` disegna lo scheletro | un fotogramma come SVG (N può essere frazionario); header `X-Cache`, `X-Render-Ms` |
| `GET /api/frame` | `frame`, `proxy` 0-4 (0 = qualità piena, 1-4 proxy di scrubbing), `kind` = `vector` (predefinito) \| `raster` \| `png`, `bones=1`, `lod=1&screen=WxH&lodpx=P` (disegna da una copia a dettaglio ridotto entro P pixel, 0-16) | l'immagine; header `X-Cache` (hit/miss), `X-Render-Ms`, `X-Lod-Level`, `X-Lod-Bound-Px`, `X-Proxy-Level`. Un fotogramma fuori intervallo: `{"errors":["frame 999 out of range"]}`; un tipo errato: `bad kind 'x' (vector, raster, png)` |
| `GET /api/cache` | | `{ok, revision, entries, bytes, hits, misses, invalidations, lod:{builds, engines, invalidations, last_build_ms, levels, unavailable}}` |
| `GET /api/overlays` | `frame`, `sel` (nomi dei livelli separati da virgole) | `{ok, frames, settings, ...}` più i ghost e l'SVG del fotogramma quando ne viene indicato uno |
| `GET /api/sounds` | | `{ok, total, sounds:[{name, start, gain, duration, peaks, error?}]}` picchi della forma d'onda dopo taglio, dissolvenze e guadagno |
| `GET /api/vcs/log` | `limit` (max 1000), `rev` | `{ok, branch, commits:[{hash, author, time, message, parents}]}`; `{"ok":false,"errors":["no .osatura-vcs repository found ..."]}` senza un repository (vedi `osatura vcs`) |
| `GET /api/vcs/diff` | `a` (predefinito `HEAD`), `b` | `{ok, changes}` |
| `GET /api/timeline` | | `{ok, fps, frames, groups, rows, tracks}` il modello del dope sheet |
| `GET /api/graph` | `ch` (indici dei canali separati da virgole), `f0`, `f1`, `n` (campioni, 2-2000, predefinito 200) | `{ok, channels, f0, f1, range, skipped}` campioni delle curve e maniglie delle tangenti |
| `GET /api/live` | | `{ok, mapping:{params,triggers}, recorded, recording}` |
| `GET /api/ui` | | `{ok, layout, presets, preset_names, default_layout, shortcuts, commands, conflicts, reserved, overlays, recents, panels, persisted, path, warning, ...}` |
| `GET /api/ui/search` | `q`, `limit` (1-200, predefinito 50) | `{hits:[{id, title, category, description, bindings, score, positions}]}` la classifica della palette |
| `GET /export.gif`, `/export.html`, `/export.html.nosound`, `/export.lottie.json` | | il file, come download (`animation.gif`, `player.html`, `animation.json`) |

### Modifica

| Route | Corpo | Risposta |
|---|---|---|
| `POST /api/op` | `{"op":"set","path":"/fps","value":30}` — `op` è `set`, `insert` (percorso di array, `.../-` accoda), `remove` oppure `batch` (`{"op":"batch","ops":[...]}`, un solo passo di annullamento). I percorsi sono JSON Pointer nel progetto. | `state`. Un risultato non valido viene rifiutato con `{"errors":["invalid project: invalid value: integer `-3`, expected u32"]}` e non cambia nulla |
| `POST /api/undo`, `POST /api/redo` | nessuno | `state` (per utente in una sessione condivisa) |
| `POST /api/key` | `{"target","prop","frame","value","ease"?}` (`ease` come `{"type":"smooth"}`; predefinito lineare) | `state` |
| `POST /api/delete_key` | `{"target","prop","frame"}` | `state` |
| `POST /api/save` | nessuno | `{"ok":true,"saved":"<path>"}`; scrive in modo atomico. Solo host, quando lo studio gira con `--collab-host` (altrimenti **403**) |
| `POST /api/suggest` | `{"kind":"between"\|"complete"\|"next","a","b","t","frame","constraints","free","top","snap","prior","apply":RANK, "library"?, "model"?}` | `{ok, suggestions}`; con `apply` anche `state` della modifica. Una richiesta senza `kind` restituisce `bad request: missing field `kind`` |

### Selezione e gizmo (senza stato: la pagina conserva la selezione)

Un **target** è `{"kind":"layer","path":"arm"}`, `{"kind":"bone","name":"hand"}` oppure `{"kind":"point","layer":"...","point":...}`.

| Route | Corpo | Risposta |
|---|---|---|
| `POST /api/pick` | `{"frame","x","y","options"?}` (`options`: `tolerance`, `bone_radius`, `point_radius`, `layers`, `bones`, `point_layers`, `locked`, `locked_bones`) | `{ok, hit, hits:[{distance, target}]}` dal più in alto |
| `POST /api/marquee` | `{"frame","rect":[x0,y0,x1,y1]` oppure `"lasso":[[x,y],...]` (da 3 a 5000 punti)`,"mode":"bbox"\|"contain"\|"intersect","options"?}` | `{ok, targets}` |
| `POST /api/select` | `{"frame","selection":[...],"targets":[...],"mode":"replace"\|"add"\|"toggle"\|"subtract"}` | `{ok, selection, bounds, gizmo:{corners, handles}, points}` |
| `POST /api/gizmo` | `{"frame","selection","handle","start":[x,y],"delta":[dx,dy],"modifiers":{"constrain","center","no_snap"},"snap":{"grid","objects","canvas","tolerance"},"auto_key","locked","pivot","commit":bool}` — `handle` è `{"type":"move"}`, `rotate`, `{"type":"scale","at":"nw"}`, `bone_head` o `bone_tip` | senza `commit`: `{ok, drag:{changed, edits, guides, matrix, summary, box_after, outline}}` (un'anteprima); con `commit:true`: `state` più `drag` e `selection`, un solo passo di annullamento |
| `POST /api/nudge` | `{"frame","selection","delta":[dx,dy],"auto_key","locked"}` | `state` + `drag` (modifiche con `mode` pari a `static` o `key`) + `selection` |
| `POST /api/delete` | `{"frame","selection"}` | `state` + `selection`; `{"errors":["nothing selected"]}` quando è vuota |

### Timeline

| Route | Corpo | Risposta |
|---|---|---|
| `POST /api/timeline/select` | `{"selection":[{ch,frame}...],"mode":"replace"\|"add"\|"toggle"\|"subtract","all":true` oppure `"channel":N` oppure `"group":N` oppure `"box":{f0,f1,row0,row1}`, `"include_locked"}` | `{ok, selection:[{ch,frame}...]}` |
| `POST /api/timeline/op` | `{"op":{...},"preview":bool}` — `op` è uno tra `move_keys`, `scale_keys`, `copy`, `paste`, `duplicate`, `delete`, `set_ease`, `set_value`, `add_key`, `ripple`, `offset`, `extrapolate`, `make_loop`, `handle`, `smooth`, `flags` (un nome sconosciuto elenca questi) | con `preview:true`: `{ok, preview:true, plan}` senza modificare nulla; altrimenti `state` + `plan:{changed, notes, op, clip}` |

Per esempio `{"preview":true,"op":{"op":"delete","keys":[{"ch":0,"frame":12}]}}` restituisce il piano (`changed: 1`) e lascia il progetto intatto.

### Overlay, live, volto, UI

| Route | Corpo | Risposta |
|---|---|---|
| `POST /api/overlays` | `{"settings":{"onion":{"enabled":true,"before":3}},"frame","selection":[names],"reset":bool}` | `{ok, frames, ghosts, settings, svg}`; le impostazioni vengono salvate nella configurazione della UI |
| `POST /api/trails` | `{"targets":[{"kind":"bone","name":"hand"}],"from","to"` oppure `"frame","before","after"`, `"smooth","svg":bool,"hit":{x,y,radius}}` | `{ok, from, to, trails:[{curve,frames,keys,...}], svg?, hit?, path_hit?}` |
| `POST /api/live/mapping` | `{"mapping":{params,triggers},"library"?}` (stesso formato di `live-map`) | stato live |
| `POST /api/live/push` | `{"events":[{source,id,value,time}],"midi":[raw...]}` | stato live con l'effetto degli eventi |
| `POST /api/live/record` | `{"on":true}` avvia; `{"on":false,"tol":0.01}` ferma e **registra la performance nel progetto** (bake) come un solo passo di annullamento | stato / `state` + riepilogo `live` |
| `POST /api/face/config` | `{"config"?,"calibration"?}` | `{ok}` |
| `POST /api/face/calibrate` | `{"on":true}` / `{"on":false}` | `{ok, frames, calibration}` |
| `POST /api/face/push` | `{"frames":[...landmarks...],"layout"?,"size"?}` (al massimo 10000 fotogrammi per push) | stato live |
| `POST /api/face/detect` | `{"image":"<base64 PNG/JPEG or data: URI>","track":bool,"push":bool,"t"}` | `{ok, found, layout:"osatura23", size, points, box, note:"detector trained on synthetic faces only"}` |
| `POST /api/face/synth` | `{"t","size":32-512,"seed"}` | `{ok, image:"data:image/png;base64,...", layout, size, points}` un fotogramma della testa sintetica con i suoi landmark esatti |
| `POST /api/ui` | uno qualsiasi tra `{"layout":{...}\|null,"shortcuts":{...}\|null,"recents":[...],"overlays":{...},"reset":"layout"\|"shortcuts"\|"overlays"\|"all"}` | documento di `GET /api/ui`; i campi sconosciuti (`unknown field 'bogus'`), i conflitti e i layout incompleti vengono rifiutati e non cambia nulla |
| `POST /api/login` | `{"token":"..."}` | `{ok:true}` e `Set-Cookie: osatura_token=...; Path=/; HttpOnly; SameSite=Strict`; token errato: 401 (429 dal 5° fallimento); senza un token configurato `{"ok":true,"token_required":false}` |

### Collaborazione

Le chiamate effettuate come partecipante inviano `X-Collab-Replica: <id>` e `X-Collab-Secret: <secret>` (entrambi restituiti da join). Un id di replica con il secret errato o mancante riceve **403**.

| Route | Corpo / query | Risposta |
|---|---|---|
| `POST /api/collab/join` | `{"name":"Ana","mode":"thin"\|"replica"}` | `{ok, replica, secret, colour, name, index, mode, clock, users, presence, doc}` (`snapshot` in modalità replica). Il primo join avvia la sessione dal documento dell'editor |
| `POST /api/collab/leave` | `{"replica"}` | `{ok, left}` |
| `GET /api/collab/state` | `replica`, `doc=1`, `snapshot=1` | `state` + `collab_active` (+ users, presence) |
| `GET /api/collab/log` | `since=N` | `{ok, index, entries:[{index, by, envelope}]}` |
| `POST /api/collab/ops` | `{"replica","ops":[...]` oppure `"op":{...}`, `"base"?}` oppure `{"replica","envelopes":[...]}` | `state` + `applied`, `index`. Partecipante sconosciuto: `unknown participant N: join first`; lista vuota: `no ops`; le modifiche non valide al progetto vengono rifiutate |
| `POST /api/collab/undo`, `.../redo` | `{"replica"}` | `state` + `collab_can_undo`, `collab_can_redo` (solo le tue modifiche) |
| `POST /api/collab/presence` | `{"replica","n","frame","cursor":[x,y],"selection":[names up to 64]}` | `{ok}` |
| `POST /api/collab/compact` | `{}` | riepilogo della compattazione (`ok`, `logged`). Solo host, quando lo studio gira con `--collab-host` (altrimenti **403**) |
| `POST /api/collab/revoke` | `{"names":["ana","bob"]}` (da 1 a 64 nomi) | `{ok, revoked, revoked_total}`: da quel momento i token di quei nomi vengono rifiutati e i partecipanti entrati con quei nomi vengono rimossi. Richiede token per partecipante (`--collab-secret-file`) e un token host (`--collab-host`), oppure il `--collab-token` condiviso; altrimenti **403**. Un flusso di eventi già aperto termina quando il suo client si disconnette |
| `GET /api/collab/events` | `since=N` oppure `Last-Event-ID` | uno stream **server-sent-events**: riproduzione da `since`, poi utenti, presenza e ogni evento successivo; `resync` se il client è troppo indietro; al massimo 64 stream (`503 too many event streams`) |

In una sessione attiva anche le normali route di modifica (`/api/op`, `/api/undo`, `/api/redo` e le modifiche dello studio stesso tramite nudge, key o gizmo) vengono registrate per gli altri partecipanti.

## 10. Risoluzione dei problemi

| Sintomo | Causa e soluzione |
|---|---|
| Il browser mostra 421 o "misdirected request" | Hai aperto lo studio con un nome che non conosce (un nome DNS, un indirizzo LAN). Aggiungilo con `--allow-host name` oppure apri `http://127.0.0.1:PORT/`. |
| 403 a ogni richiesta dietro un proxy | Il proxy modifica `Origin`/`Host`. Indica l'origine pubblica: `--public-origin https://studio.example.org`. |
| 401 | `--collab-token` è impostato: apri la pagina e inserisci il token una volta, oppure invia `X-Collab-Token`. Un token nell'URL non viene accettato. |
| 429 | Cinque token errati di fila dal tuo indirizzo; attendi fino a 30 s. Dietro un proxy tutti gli utenti condividono un solo indirizzo. |
| `refusing to listen on 0.0.0.0:...` | Bind non loopback senza token. Aggiungi `--collab-token` (16 o più caratteri). |
| `cannot listen on 127.0.0.1:8080: Address already in use` | Un altro processo occupa la porta; scegli un'altra `--port`. La causa abituale è uno studio rimasto attivo da un'esecuzione precedente. |
| La pagina si carica ma l'immagine è vuota | Controlla il terminale; un progetto che non supera `check` viene rifiutato all'avvio. Ricarica dopo la correzione. |
| Un messaggio nella console del browser `Failed to load resource ... 404` | Le build precedenti non avevano `/favicon.ico`; ora viene servito. Un eventuale 404 residuo ha un'altra origine. |
| Il testo del pannello Face | Il pannello ora dichiara che il detector addestrato su dati sintetici è incluso, che non è validato su foto o webcam e che una vera webcam richiede un tuo provider di landmark. |
| Il layout è rotto o mancano dei pannelli | Menu `Layout`, ripristina il layout, oppure elimina l'`ui.json` mostrato all'avvio. Un file di configurazione che non può essere letto per intero viene copiato in `ui.json.bak` (poi `.bak.1`, ...) prima di usare i valori predefiniti, e `GET /api/ui` ha un campo `warning` al riguardo. |
| Le mie modifiche sono sparite dopo un riavvio | Non viene scritto nulla finché non usi **Save**. Con `--collab-log` la *sessione* sopravvive a un riavvio, ma il file su disco viene comunque scritto solo da Save. |
| `Ctrl+W`, `Ctrl+T` e simili non possono essere associati | Sono riservati dai browser (elenco nella sezione 3). |
| Ctrl+Z non annulla la modifica di qualcun altro in una sessione | È voluto: l'annullamento è per utente. |
| Le esportazioni richiedono `ffmpeg` | Solo per il suono nell'HTML (mp3) e per mp4/webm da riga di comando; GIF, Lottie e HTML senza suono non richiedono nulla. |

## 11. Accessibilità e supporto dei browser

**Browser.** L'interfaccia è stata testata solo in **Chromium** (la build inclusa in Playwright, pilotata in modalità headless, su Linux). Firefox e Safari non sono stati provati e potrebbero funzionare o no.
Il Web MIDI richiede un browser che lo implementi (Chromium lo fa); la fotocamera richiede `getUserMedia` e un contesto sicuro (HTTPS o `localhost`). La pagina usa i server-sent events per la
sessione condivisa. Per il trascinamento si usano i pointer event; gli schermi touch non sono stati testati.

**Accessibilità, onestamente.** Non ho sottoposto lo studio a un audit con uno screen reader né con una sessione solo da tastiera. Ciò che esiste: un attributo `lang` e il meta tag viewport, checkbox con etichetta, un
toast con `role="status"`, alcuni attributi ARIA, **un set completo di scorciatoie da tastiera e una palette dei comandi** che raggiungono ogni comando, scorciatoie riassegnabili e la riga di comando `osatura`, che fa
tutto ciò che fa l'editor senza un puntatore. Ciò che manca: il canvas è un'immagine e la maggior parte della modifica sul canvas avviene con il puntatore (lo spostamento con i tasti freccia esiste), non c'è un tema chiaro e
nessuna gestione di `prefers-reduced-motion` o `prefers-color-scheme`, i contorni del focus sono quelli predefiniti del browser e non li ho verificati, e non ho misurato il contrasto dei colori. Se dipendi da una tecnologia assistiva,
preferisci la CLI e il pannello JSON, e segnala al progetto che cosa non funziona.

## 12. Limiti noti

* Corretti dopo la prima stesura: il testo del pannello Webcam face, il 404 di `/favicon.ico`, `--collab-nosync` senza `--collab-log` e il flag `Secure` mancante del cookie dietro un `--public-origin` https.
* Anche `osatura render p.json 99 o.png` su un progetto di 24 fotogrammi ora è un errore (aggiungi `--clamp` per renderizzare l'ultimo fotogramma); `/api/frame` dello studio rifiuta un fotogramma fuori intervallo (`frame 999 out of range`) e ora accetta solo i fotogrammi `0..frames-1`.
* Le modifiche a layout e scorciatoie sono memorizzate per utente nella configurazione della UI, non nel progetto, quindi non viaggiano con il file.
