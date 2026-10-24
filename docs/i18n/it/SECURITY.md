<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Sicurezza

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/README.md) · [Română](https://github.com/chiuta/osatura/blob/main/README.ro.md) · [Français](https://github.com/chiuta/osatura/blob/main/README.fr.md) · [Italiano](https://github.com/chiuta/osatura/blob/main/README.it.md) · [Español](https://github.com/chiuta/osatura/blob/main/README.es.md) · [Português](https://github.com/chiuta/osatura/blob/main/README.pt.md) · [Deutsch](https://github.com/chiuta/osatura/blob/main/README.de.md) · [Magyar](https://github.com/chiuta/osatura/blob/main/README.hu.md)

*Modello di minaccia, protezioni integrate, lacune note, come segnalare un problema e come installare lo studio in sicurezza. Documentazione CC-BY-4.0. Ricavata dai sorgenti (`osatura-core/src/{api,assets,fsutil,expr,script}.rs`, `osatura-cli/src/{http,studio,collab_srv}.rs`), da `ROADMAP.md` sezione 9 (ondate di revisione da 1 a 4) e dai test indicati qui sotto. **OSATURA non ha mai avuto un audit di sicurezza indipendente.** Le ondate di revisione sono state svolte dal progetto stesso; considera questa pagina come una descrizione delle intenzioni e di ciò che i test coprono, non come una garanzia.*

Indice: [1 Modello di minaccia](#1-modello-di-minaccia) - [2 Server HTTP dello studio](#2-server-http-dello-studio) - [3 Autenticazione e collaborazione](#3-autenticazione-e-collaborazione) - [4 Risoluzione degli asset e limiti sull'input](#4-risoluzione-degli-asset-e-limiti-sullinput) - [5 Sandbox di espressioni e script](#5-sandbox-di-espressioni-e-script) - [6 File, collegamenti simbolici e dati temporanei](#6-file-collegamenti-simbolici-e-dati-temporanei) - [7 Programmi esterni (ffmpeg)](#7-programmi-esterni-ffmpeg) - [8 Librerie (C ABI, WASM, facciata)](#8-librerie-c-abi-wasm-facciata) - [9 Lacune note](#9-lacune-note) - [10 Segnalare una vulnerabilità](#10-segnalare-una-vulnerabilità) - [11 Checklist di hardening per chi installa](#11-checklist-di-hardening-per-chi-installa) - [12 Reverse proxy con TLS: nginx e Caddy](#12-reverse-proxy-con-tls-nginx-e-caddy)

## 1. Modello di minaccia

Ciò che OSATURA elabora: **file di progetto** (JSON), immagini, audio, file di importazione Lottie / OCA / in stile glTF, file di mappatura e, nello studio, richieste HTTP da un browser. Nel modello qui sotto i file di progetto sono *dati non fidati*: un progetto ricevuto da altri può essere ostile, e aprirlo non deve eseguire codice, leggere file fuori dalla sua cartella né esaurire le risorse della macchina.

| Attore | Può | Obiettivo ipotizzato | Difese principali |
|---|---|---|---|
| Autore di un progetto / file di importazione ostile | consegnarti un file | leggere i tuoi file, mandare in crash o bloccare lo strumento, eseguire codice, riempire il disco | parsing rigoroso, limiti di dimensione e profondità, rifiuto dei percorsi degli asset, nessun panic sull'input, sandbox di espressioni e script, output atomico |
| Pagina web che visiti mentre lo studio gira su localhost | far inviare al tuo browser richieste a `127.0.0.1` (CSRF, DNS rebinding) | modificare il tuo progetto, leggerlo, scrivere file | lista di Host consentiti, controlli di Origin / `Sec-Fetch-Site`, tipo di contenuto JSON, token con cookie `SameSite=Strict`, CSP |
| Altro utente sulla stessa macchina | connettersi alla porta, leggere `/tmp` | leggere o dirottare una sessione o file temporanei | bind su `127.0.0.1` per impostazione predefinita, directory temporanee private (0700), file di log 0600, token obbligatorio per fare il bind altrove |
| Qualcuno sulla tua rete, quando lo studio è in bind oltre localhost | raggiungere la porta | usare o indovinare il token, esaurire le connessioni | token obbligatorio (16+ caratteri), confronto a tempo costante, back-off, limiti su connessioni e dimensioni; **nessun TLS** |
| Un collaboratore con il token | modificare la sessione condivisa | danneggiare il progetto | cronologia di annullamento, log con catena di hash; ai collaboratori è affidato il progetto, non ci sono account né ruoli |
| Chiamante ostile delle librerie | passare argomenti errati alla facciata, alla ABI C o WASM | corruzione della memoria, panic oltre il confine | validazione degli argomenti, controlli di dimensione prima dell'allocazione, panic intercettati (dove il target esegue l'unwinding) |

Fuori ambito: la protezione da un amministratore locale malevolo, da una toolchain di build o da una dipendenza compromessa (vedi [THIRD_PARTY.md](THIRD_PARTY.md) per ciò che viene distribuito; `cargo deny` controlla licenze, ban e sorgenti, non le vulnerabilità), i canali laterali e la sicurezza di ffmpeg stesso.

## 2. Server HTTP dello studio

`osatura studio` serve l'editor con una propria piccola implementazione HTTP/1.1 (`osatura-cli/src/http.rs`, solo libreria standard, nessun framework web di terze parti). Il comportamento, con i test che lo fissano (test unitari di `osatura-cli`, `tests/robust.rs`, `tests/wave4.rs`, `tests/source_offer.rs` e gli smoke test Chromium):

**Limiti**

| Limite | Valore | Risultato se superato |
|---|---|---|
| Tempo per ricevere l'intestazione della richiesta | 10 s | 408 |
| Tempo per ricevere il corpo | 20 s | 408 |
| Tempo per scrivere una risposta | 10 s | connessione chiusa |
| Dimensione dell'intestazione della richiesta | 16 KB | 431 |
| Numero di header | 100 | 431 |
| Dimensione del corpo | 32 MB | 413 |
| Connessioni concorrenti | 128 | 503 |
| Corpi di richiesta chunked | non supportati | 411 |
| Header `Host` duplicato | rifiutato | 400 |
| Carattere di controllo (CR, NUL, DEL…) nel valore di un header | rifiutato (il tab è consentito) | 400 |

Ogni risposta porta `Connection: close` (nessuno stato keep-alive da esaurire), `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` e un `Content-Security-Policy`.

**Politica sulle richieste** (`Policy::check`, applicata a ogni richiesta):

1. **Lista di Host consentiti.** L'header `Host` deve essere l'indirizzo in bind (`127.0.0.1`, `localhost`, la porta reale) o un nome indicato con `--allow-host`. Altrimenti 421 Misdirected Request. È ciò che ferma il DNS rebinding: una pagina su `evil.example` che si risolve in `127.0.0.1` invia `Host: evil.example`.
2. **Origin e `Sec-Fetch-Site`.** Le richieste con un `Origin` di un altro sito, o con `Sec-Fetch-Site: cross-site` / `same-site`, sono rifiutate con 403. `Origin: https://<allowed host>` è accettato per gli host in `--allow-host`; dietro un reverse proxy che serve la pagina a un URL pubblico, passa `--public-origin URL` in modo che l'origine del proxy sia accettata. Il cross-origin resta rifiutato.
3. **Tipo di contenuto.** `POST` richiede `Content-Type: application/json` (altrimenti 415), così un semplice modulo HTML non può fare POST all'API.
4. **Token** (vedi sezione 3), quando configurato; obbligatorio quando `--bind` non è un indirizzo di loopback.

**Offerta dei sorgenti.** Lo studio serve `/source` (anche quando è richiesto un token) e `--source-url` ne imposta l'indirizzo (i binari di release ufficiali usano come predefinito il commit da cui sono stati compilati), come richiesto dalla sezione 13 dell'AGPL-3.0 (vedi [LICENSING.md](LICENSING.md)).

**Percorsi e metodi sconosciuti o malformati** ricevono una semplice risposta di errore; la pagina dell'editor è compilata nel binario (`studio.html`) e non viene letta dal file system, quindi non esiste una superficie di path traversal per i contenuti statici.

## 3. Autenticazione e collaborazione

L'opzione `--collab-token T` (o il token richiesto da un `--bind` non locale) abilita il controllo degli accessi; `--collab-log LOG` rende persistente una sessione condivisa.

* **Formato del token.** Almeno 16 caratteri, tra `A-Za-z0-9._~+/=-`. Qualsiasi cosa con CR, LF, `;` o altri caratteri è rifiutata all'avvio (iniezione in header e cookie).
* **Presentazione.** Header `X-Collab-Token: T`, oppure il cookie `arm_token` (`HttpOnly`, `SameSite=Strict`, più `Secure` quando la richiesta è arrivata con un nome `--public-origin` `https://`, un `Origin` https o `X-Forwarded-Proto: https`) impostato da `POST /api/login`. Un cookie errato conta come tentativo fallito come un header errato.
* **Confronto.** A tempo costante (entrambi i lati sono sottoposti a hash con SHA-256 e i digest confrontati), quindi il tempo di risposta non rivela corrispondenze di prefisso.
* **Back-off.** Per indirizzo IP del client, dal **5°** fallimento consecutivo: 2, 4, 8, 16, poi 30 secondi di risposte 429; vengono tracciati fino a 512 indirizzi client. Il token corretto è sempre accettato, anche durante un back-off (così un attaccante non può chiudere fuori il proprietario). Il back-off rallenta i tentativi alla cieca; non ferma un attacco distribuito. **Dietro un reverse proxy ogni client appare con l'indirizzo del proxy**, quindi condividono un unico contatore di back-off: un'ondata di richieste errate da un visitatore ritarda il contatore dei fallimenti per tutti, ma il token corretto continua a funzionare.
* **Segreti dei partecipanti.** Ogni collaboratore riceve un segreto casuale a 128 bit (`X-Collab-Secret`) che identifica il suo flusso; al massimo 32 partecipanti e 64 flussi di eventi.
* **Integrità del log.** Il log della sessione è un file di testo, una riga per operazione: `<hash> <json>`, concatenata con SHA-256 (formato versione 2). Modificare, cancellare o riordinare una riga spezza la catena, cosa che viene segnalata al caricamento del log. Ogni richiesta è sottoposta a `fdatasync` prima della risposta, il file è creato con modo 0600 e riscritto atomicamente. La catena rileva danni accidentali e manomissioni occasionali; **non è firmata**, quindi chi può scrivere il file può ricostruire una catena valida.
* **Token per partecipante (ondata 7, opzionale).** `--collab-secret-file F` sostituisce il bearer condiviso unico con token derivati da un segreto: `ap1.<name>.<expires>.<mac>` con `mac` = HMAC-SHA-256 (RFC 2104, SHA-256 proprio, vettori di test RFC 4231 nei test) calcolato sul nome e sulla scadenza. L'host li genera con `osatura collab-token NAME --secret-file F --ttl SECONDS` (predefinito 8 ore, al massimo 30 giorni) e ne consegna uno personale a ciascuna persona. Un token è legato a un solo nome (un partecipante che entra ottiene quel nome qualunque cosa invii la pagina, quindi nessuno può entrare spacciandosi per un altro), scade da solo, è verificato per ricalcolo (nessuna tabella sul server, nulla da rubarvi tranne il segreto stesso) e il suo cookie riceve un `Max-Age` pari alla vita residua. `--collab-revoke NAMES` rifiuta i token di quei nomi prima della scadenza; cambiare il segreto revoca tutto. Il file del segreto deve essere privato (modo 600 su Unix, altrimenti lo studio rifiuta di avviarsi), di almeno 16 caratteri, e non viene mai accettato come argomento da riga di comando. `--collab-token` e `--collab-secret-file` possono essere combinati; ciascuno soddisfa la regola per cui un bind non di loopback richiede una credenziale. La scadenza è controllata **per richiesta**: un flusso di eventi già aperto resta aperto fino alla sua fine, e la connessione successiva viene rifiutata. A un token scaduto o revocato si risponde `401` con il motivo, e non alimenta il back-off dei tentativi.
* **Limite di frequenza (ondata 7).** `--collab-rate N` consente N richieste al secondo per indirizzo client, con raffiche di 2N (token bucket); oltre si risponde `429` con `Retry-After`. Il valore predefinito è 100 al secondo quando è richiesta una credenziale e nessun limite altrimenti; 0 lo disattiva. Viene applicato dopo l'autenticazione e per **indirizzo** (la sorgente di una connessione), non per connessione TCP, perché un limite per connessione si aggira aprendo più connessioni; dietro un reverse proxy tutti i client condividono l'indirizzo del proxy, quindi limita anche sul proxy (sezione 12).
* **Avvisi sull'indirizzo di bind (ondata 7).** Un `--bind` non di loopback stampa un avviso all'avvio (HTTP semplice; per `0.0.0.0` / `::` che lo studio è in ascolto su ogni interfaccia; per un token condiviso che non ha nomi e non scade mai).
* **Nessun TLS, nessun account.** Il server parla HTTP semplice. Un token che viaggia su una rete è leggibile da chiunque si trovi sul percorso. Metti lo studio dietro un reverse proxy che termina il TLS o un tunnel SSH / VPN (sezione 11). Non ci sono account utente, ruoli o permessi per progetto: il token concede pieno accesso in modifica.

## 4. Risoluzione degli asset e limiti sull'input

* **Gli asset** (file immagine, normal map, suoni indicati da un progetto) si risolvono tramite `assets::resolve`: relativamente alla directory degli asset valida per tutto il processo (`assets::default_dir`, normalmente la cartella del progetto in ingresso). **I percorsi assoluti e qualsiasi componente `..` sono rifiutati** a meno che sia indicato il flag globale `--allow-external-assets`. I file immagine sono limitati a 64 MB, i file audio a 256 MB.
* **File di progetto.** `load_project` legge solo file regolari (niente dispositivi, niente pipe), fino a 256 MB (`--max-input-bytes N` per cambiarlo). `MAX_JSON_BYTES` della facciata è 256 MiB.
* **Progetti derivati.** I comandi che scrivono un progetto derivato validano rispetto alla directory del progetto *in ingresso*; quando l'output è altrove, i percorsi relativi degli asset dei file esistenti vengono riscritti oppure un avviso dice che l'output non li troverà (`--allow-external-assets` li rende assoluti).
* **Rendering.** La facciata rifiuta immagini più grandi di 16384 px per lato (`MAX_RENDER_SIDE`) e scale fuori da 0.05-4; i livelli C e wasm rifiutano output superiori a 256 MiB prima di allocare. Altri limiti della facciata: fps in (0, 1000], fotogrammi da 1 a 200000, stage fino a 32768, 4096 ossa, grandezze entro 1e7, `lod_levels` da 1 a 12, campioni di `fuzz` fino a 100000, colori di `trace` da 2 a 64 ([API.md](API.md)).
* **Nessun panic sull'input.** I test con input ostili si trovano in `osatura-cli/tests/robust.rs` e nei test delle ondate; test in stile fuzz coprono i parser. Un panic raggiungibile dall'input è trattato come una vulnerabilità.
* **I corpi dello studio** sono limitati a 32 MB; le modifiche passano per la stessa validazione dei file.

## 5. Sandbox di espressioni e script

* **Le espressioni** (binding di valori, vincoli, driver) sono analizzate da un parser interno: **sorgente fino a 4096 byte, profondità di annidamento 64**, aritmetica, confronto, le variabili fisse `frame`, `time`, `fps`, `frames`, `pi`, `t`, `get("layer","property")` per i driver e una tabella fissa di funzioni predefinite (`sin`, `abs`, `min`, ...); niente cicli, niente I/O e nessuna funzione definita dall'utente, quindi la valutazione termina sempre. I risultati non finiti sono rifiutati dalla politica dei valori finiti.
* **Gli script** (comando `script`, feature `scripting`, attiva per impostazione predefinita) girano in Rhai con questi limiti: **500.000 operazioni, 32 livelli di chiamata, profondità delle espressioni (64 al livello superiore, 32 nelle funzioni), stringhe di 10.000 caratteri, array di 10.000 elementi, mappe di 1.000 voci, 5.000 richieste all'API, sorgente dello script di 64 KB**, e `no_module` (nessun `import`, quindi nessun accesso ai file tramite moduli). L'API host esposta agli script è un insieme fisso di funzioni di modifica del progetto. Compila con `--no-default-features` per rimuovere del tutto il motore di scripting (rimuove anche la dipendenza `smartstring`).
* Uno script è comunque codice che hai scelto di eseguire su un progetto: non può leggere né scrivere file da sé, ma può rendere molto grande il progetto che modifica entro i limiti sopra indicati, e un ciclo infinito termina al limite di operazioni.

## 6. File, collegamenti simbolici e dati temporanei
* **Scritture atomiche.** Gli output sono scritti con `fsutil::write_atomic`: un file temporaneo nella stessa directory (nome `.<name>.tmp-<pid>-<random>`), `fsync`, `rename`. Un crash lascia il file vecchio o quello nuovo. **I collegamenti simbolici indicati come output sono rifiutati**, citando `--follow-symlinks` (ciò ora copre ogni output indirizzato dall'utente: report e golden di `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, file di pose, snapshot, `vcs`, il log di collaborazione e i file video scritti da ffmpeg, che sono controllati in anticipo), così un collegamento piazzato ad arte non può reindirizzare una scrittura. Se la directory non è scrivibile ma il file sì, il ripiego è una sovrascrittura sul posto con `fsync` che **non è atomica** (documentato).
* **Dati privati.** I log di collaborazione e gli altri file che solo il proprietario deve poter leggere sono creati con modo 0600 (`write_atomic_private`). Le directory temporanee sono create in modo esclusivo con modo 0700 (`private_temp_dir`), così un nome prevedibile in un `/tmp` condiviso non può essere creato in anticipo da un altro utente. I valori casuali (token per i nuovi partecipanti, nomi temporanei) provengono da `/dev/urandom` (`fsutil::random_bytes`); il generatore deterministico con seed del motore non viene mai usato per i segreti.
* **Impostazioni dell'interfaccia.** Il `ui.json` dello studio ignora le chiavi sconosciute con un avviso e viene copiato in `ui.json.bak` (poi `.bak.1`, ...) prima del primo salvataggio.
* **Windows / non Unix.** I modi di permesso sopra indicati valgono su Unix. Su Windows e macOS la suite di test passa in CI (i controlli solo POSIX vengono saltati su Windows), ma i controlli su symlink, permessi e `/dev/urandom` girano solo su Linux e macOS e nessuno ha rivisto a mano il comportamento di Windows.
* **Binari non firmati e app a finestra.** Gli eseguibili di release non sono firmati; verifica i download con `SHA256SUMS.txt`. `osatura-studio` avvia lo stesso server locale (solo `127.0.0.1`, porta libera casuale) e lo mostra nella web view di sistema (WebView2 / WKWebView); la sua web view è esposta come una scheda del browser su quell'indirizzo e non ha avuto una revisione di sicurezza separata.

## 7. Programmi esterni (ffmpeg)

ffmpeg e ffprobe sono eseguiti come processi separati **solo** per l'esportazione video, la decodifica mp3 / ogg, il mixaggio della colonna sonora e alcuni strumenti; non vengono mai inclusi ([LICENSING.md](LICENSING.md)). Ogni invocazione:

* passa gli argomenti come array (nessuna shell, nessuna concatenazione di stringhe in una riga di comando);
* usa `-protocol_whitelist file` (un uso interno di pipe aggiunge `pipe,fd`), così un file multimediale non può far aprire a ffmpeg `http://`, `concat:` o altri protocolli di rete / speciali;
* passa i nomi dei file tramite `fsutil::file_arg`, che antepone `./` ai nomi che iniziano con `-` o contengono `:` o `|` nel primo componente del percorso, così un nome di file non può essere letto come opzione o URL;
* usa `-loglevel error`.

ffmpeg stesso analizza formati multimediali complessi e ha avuto vulnerabilità proprie: tienilo aggiornato e non dargli in pasto contenuti multimediali da fonti non fidate su una macchina che custodisce segreti. I comandi con budget di tempo (`export-fit --max-seconds`) non possono interrompere una singola chiamata a ffmpeg in corso.

## 8. Librerie (C ABI, WASM, facciata)

* La **facciata** valida ogni argomento, restituisce `ApiError` e converte i panic in `ApiError::Internal` dove il target esegue l'unwinding. `Animation` e `Player` sono `Send + Sync` (verificato in compilazione).
* La **C ABI** ([C_ABI.md](C_ABI.md)) controlla puntatori nulli, lunghezze e UTF-8 e restituisce codici; il chiamante deve comunque passare puntatori validi e rispettare le regole sui buffer. La stringa dell'ultimo errore è per thread. La ABI è stata messa alla prova con AddressSanitizer e UndefinedBehaviorSanitizer con gli esempi documentati (pulita). Non è rafforzata contro un chiamante che viola i contratti documentati (doppia chiusura, uso dopo la chiusura, lunghezze errate).
* Il **modulo WASM** ([WASM.md](WASM.md)) viene compilato in CI e supera il test del wrapper sotto Node ed è stato eseguito una volta in Chromium headless (un input errato produce errori, nessuna perdita in 60 cicli di apertura/chiusura, stage troppo grandi rifiutati), ma **non è stato rivisto come componente di browser e non è stato eseguito in Firefox o Safari**, quindi le sue proprietà di sicurezza sono note solo dalla lettura del codice e da quel test. Per costruzione non ha accesso al file system né alla rete; un panic lì manda in trap l'istanza.
* Nessuno dei due livelli isola in sandbox il *progetto*: il rendering di un progetto ostile è limitato dai limiti sopra indicati, ma un progetto può comunque richiedere molto lavoro entro di essi (molti livelli, immagini grandi, timeline lunghe).

## 9. Lacune note

Elenco onesto; vedi [LIMITATIONS.md](LIMITATIONS.md) per l'elenco più ampio dei limiti funzionali.

1. **Nessun TLS e nessun account** nello studio. Dall'ondata 7 il token può essere per partecipante e con scadenza (sezione 3), cosa che comunque non è un sistema di account: non ci sono ruoli, né reimpostazione della password, né un TLS proprio. Non è stato aggiunto un TLS nativo perché in `Cargo.lock` non c'è alcun crate TLS compatibile per licenza (aggiungerne uno porterebbe una grande nuova dipendenza, e codice crittografico, in un progetto AGPL/MPL senza un audit); la sezione 12 mostra il modo supportato, un reverse proxy che termina il TLS.
2. **Il back-off è per indirizzo** ed è condiviso dietro un proxy; rallenta ma non impedisce i tentativi online alla cieca. Usa un token lungo e casuale (il minimo di 16 caratteri è una soglia minima, non una raccomandazione).
3. **Il log di collaborazione è concatenato, non firmato.**
4. **Non è una base di codice sottoposta ad audit.** Nessun penetration test di terze parti né campagna formale di fuzzing; i controlli sono i test e le ondate di revisione del progetto stesso.
5. **Le dipendenze sono controllate per le licenze, non per le vulnerabilità.** Nel repository non c'è alcun passaggio `cargo audit`; eseguilo tu prima del deployment.
6. **La directory degli asset vale per tutto il processo.** `merge` su più directory in ingresso usa l'ultima directory per gli asset; solo `write_json` riscrive i percorsi degli asset (gli altri scrittori no).
7. **Ripiego non atomico** quando la directory di output non è scrivibile.
8. **I controlli sui percorsi usano percorsi canonici.** `assets::resolve` canonicalizza i file esistenti e rifiuta un file la cui posizione reale è fuori dalla directory del progetto, quindi un collegamento simbolico dentro la cartella di un progetto che punta all'esterno è rifiutato (testato in `assets.rs`); i collegamenti che restano dentro la cartella funzionano. Una race tra il controllo e la lettura successiva (time-of-check to time-of-use) da parte di chi può modificare contemporaneamente la cartella del progetto non è esclusa.
9. **ffmpeg** è fuori dal nostro controllo (sezione 7).
10. **WASM** non è testato in un motore di browser (in CI gira solo sotto Node); il modello di minaccia del browser per una pagina che lo incorpora (CSP, limiti di memoria) non è stato esaminato.
11. **Denial of service per dimensione.** I limiti circoscrivono le singole richieste e i file, non il lavoro totale: un progetto valido ma molto pesante può occupare una CPU a lungo (il tetto alle connessioni sopra indicato limita la concorrenza, non il costo di una richiesta).

## 10. Segnalare una vulnerabilità

* **Non aprire una issue pubblica** per una vulnerabilità. Invia una segnalazione privata al maintainer: **`alexio@trom.tf`**. *(Segnaposto: l'indirizzo compare nell'avviso di `--version` del programma e nei documenti di licenza; il proprietario deve confermarlo come contatto per la sicurezza e può sostituirlo con un indirizzo dedicato o un canale di advisory privato prima del primo rilascio.)*
* Includi: versione (`osatura --version`) e piattaforma, il progetto o la richiesta che provoca il problema (un file minimo è la cosa migliore), cosa ti aspettavi e cosa è successo, e se vuoi essere citato.
* Cosa aspettarsi *(un'intenzione, non un contratto: questo è un progetto di una sola persona)*: una conferma di ricezione in tempi ragionevoli, una correzione sviluppata in privato, un test di regressione aggiunto alla suite, una nota in [CHANGELOG.md](CHANGELOG.md) e un riconoscimento se lo desideri. Lascia il tempo per una correzione prima di pubblicare i dettagli.
* Versioni supportate: solo l'ultima pre-release (`v0.1.0-alpha.3` una volta pubblicata) e l'albero attuale su `main` sono supportati.
* Le segnalazioni sull'esposizione di rete dello studio quando è deliberatamente in bind su un indirizzo pubblico senza reverse proxy sono prima di tutto problemi di documentazione: vedi sezione 11.

## 11. Checklist di hardening per chi installa

Per chiunque esegua `osatura studio` per più di una persona o fuori da localhost.

1. [ ] Preferisci **non** esporlo. Utente singolo: lascia il bind predefinito (`127.0.0.1`) e usa un tunnel SSH (`ssh -L 8080:127.0.0.1:8080 host`).
2. [ ] Se è esposto, **termina il TLS** in un reverse proxy (nginx, Caddy, ...) davanti ad esso; non esporre mai la porta HTTP semplice a Internet.
3. [ ] Usa un **token lungo e casuale** (per esempio 32+ caratteri da `openssl rand -base64 24`), passalo tramite un file protetto o l'ambiente del servizio anziché la cronologia della shell, e ruotalo quando qualcuno se ne va. Ricorda che dà pieno accesso in modifica.
4. [ ] Avvia con `--bind 127.0.0.1` (o l'interfaccia privata) più il proxy; imposta `--allow-host your.domain` e `--public-origin https://your.domain` in modo che i controlli su Host e Origin passino per il proxy e per nient'altro.
5. [ ] Preferisci **token per partecipante** (`--collab-secret-file`, sezione 3) con un `--ttl` breve a un unico token condiviso; tieni il file del segreto con modo 600 fuori dalla cartella servita.
5a. [ ] Fai in modo che il proxy applichi un **limite di frequenza** e, se possibile, limiti per IP o aggiunga un secondo livello di autenticazione (HTTP basic su TLS, VPN, proxy SSO), perché il back-off integrato vede solo l'indirizzo del proxy.
6. [ ] Esegui come **utente non privilegiato** in una directory dedicata contenente solo i progetti da servire; nessun segreto, nessun collegamento simbolico sensibile. Usa un file system in sola lettura altrove, un container o il sandboxing di systemd (`ProtectSystem`, `PrivateTmp`, `NoNewPrivileges`).
7. [ ] Mantieni **disattivati** `--allow-external-assets` e `--follow-symlinks`.
8. [ ] Imposta `--source-url` sul luogo da cui si possono ottenere i sorgenti (eventualmente modificati) (sezione 13 dell'AGPL).
9. [ ] Rendi persistente con `--collab-log` in una directory privata, fanne il backup e mantieni il modo 0600. Tratta il log come sensibile: contiene l'intera cronologia delle modifiche.
10. [ ] Limita le risorse a livello di sistema operativo (memoria, tempo CPU, file aperti, quota disco); i tetti integrati limitano le singole richieste, non il carico totale.
11. [ ] Mantieni aggiornati ffmpeg e la toolchain Rust usata per la build; esegui `cargo audit` (o un equivalente) su `Cargo.lock` per il tuo rilascio.
12. [ ] Non dare al server progetti da fonti non fidate a meno che giri nella sandbox sopra descritta. Per deployment senza scripting compila con `--no-default-features`.
13. [ ] Controlla nei log le risposte 421, 403, 429 e 503; i picchi indicano sondaggi o errori di configurazione.

## 12. Reverse proxy con TLS: nginx e Caddy

Lo studio parla HTTP semplice. Per qualunque uso oltre la tua macchina eseguilo su loopback e lascia che un proxy termini il TLS. Queste ricette sono state scritte a partire dalla documentazione pubblica dei proxy e dal comportamento dello studio descritto sopra; **non sono state eseguite nell'ambiente di sviluppo** (nessun nginx o Caddy, nessun certificato, nessuna rete): provale prima di farci affidamento.

Avvia lo studio su loopback e indica il suo host pubblico, in modo che i controlli su Host e Origin accettino il proxy e nient'altro:

```
osatura studio demo.json --port 8080 --bind 127.0.0.1 \
  --allow-host studio.example.org --public-origin https://studio.example.org \
  --collab-secret-file /etc/osatura/collab.secret --collab-log /var/lib/osatura/session.log
```

Con un `--public-origin` `https://` il cookie di login riceve l'attributo `Secure`. Il flusso di eventi (`/api/collab/events`) è Server-Sent Events: il proxy non deve metterlo in buffer e deve consentire letture lunghe.

**nginx** (certificati dalla tua CA o da `certbot`):

```
limit_req_zone $binary_remote_addr zone=osatura:10m rate=20r/s;

server {
    listen 80;
    server_name studio.example.org;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name studio.example.org;
    ssl_certificate     /etc/letsencrypt/live/studio.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/studio.example.org/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    add_header Strict-Transport-Security "max-age=31536000" always;
    client_max_body_size 64m;

    location / {
        limit_req zone=osatura burst=60 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
    }
    location /api/collab/events {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 1h;
    }
}
```

**Caddy** (ottiene e rinnova da solo il certificato; `Caddyfile`):

```
studio.example.org {
    encode zstd gzip
    header Strict-Transport-Security "max-age=31536000"
    request_body {
        max_size 64MB
    }
    reverse_proxy 127.0.0.1:8080 {
        flush_interval -1
        transport http {
            read_timeout 1h
        }
    }
}
```

`flush_interval -1` fa passare il flusso di eventi senza buffer. Caddy imposta `X-Forwarded-Proto` e inoltra l'`Host` originale. Per un limite alla frequenza delle richieste usa il plugin `caddy-ratelimit` oppure metti davanti il `limit_req` di nginx.

Controlli dopo l'avvio: `curl -I https://studio.example.org/` risponde `401` (pagina di login) e non serve mai lo studio senza un token; `curl -I http://127.0.0.1:8080/ -H 'Host: evil.example'` risponde `421`; il cookie impostato da `/api/login` porta `Secure; HttpOnly; SameSite=Strict`; il log della sessione dello studio stesso resta privato (sezione 6). Se il proxy gira su un'altra macchina, fai il bind dello studio sull'interfaccia privata raggiungibile dal proxy, proteggi la porta con un firewall e ricorda che lo studio stesso parla comunque HTTP semplice su quel collegamento.
