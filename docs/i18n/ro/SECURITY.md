<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Securitate

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/docs/SECURITY.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/SECURITY.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/SECURITY.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/SECURITY.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/SECURITY.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/SECURITY.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/SECURITY.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/SECURITY.md)

*Modelul de amenințări, protecțiile încorporate, lacunele cunoscute, cum raportezi o problemă și cum implementezi studioul în siguranță. Documentație CC-BY-4.0. Derivată din sursă (`osatura-core/src/{api,assets,fsutil,expr,script}.rs`, `osatura-cli/src/{http,studio,collab_srv}.rs`), din `ROADMAP.md` secțiunea 9 (valurile de revizie 1–4) și din testele numite mai jos. **OSATURA nu a avut niciun audit de securitate independent.** Valurile de revizie au fost făcute chiar de proiect; tratează această pagină ca o descriere a intenției și a ceea ce acoperă testele, nu ca o garanție.*

Cuprins: [1 Modelul de amenințări](#1-modelul-de-amenințări) - [2 Serverul HTTP al studioului](#2-serverul-http-al-studioului) - [3 Autentificare și colaborare](#3-autentificare-și-colaborare) - [4 Rezolvarea resurselor și limitele de intrare](#4-rezolvarea-resurselor-și-limitele-de-intrare) - [5 Sandbox-ul pentru expresii și scripturi](#5-sandbox-ul-pentru-expresii-și-scripturi) - [6 Fișiere, legături simbolice și date temporare](#6-fișiere-legături-simbolice-și-date-temporare) - [7 Programe externe (ffmpeg)](#7-programe-externe-ffmpeg) - [8 Biblioteci (C ABI, WASM, fațadă)](#8-biblioteci-c-abi-wasm-fațadă) - [9 Lacune cunoscute](#9-lacune-cunoscute) - [10 Raportarea unei vulnerabilități](#10-raportarea-unei-vulnerabilități) - [11 Listă de consolidare pentru cel care implementează](#11-listă-de-consolidare-pentru-cel-care-implementează) - [12 Reverse proxy cu TLS: nginx și Caddy](#12-reverse-proxy-cu-tls-nginx-și-caddy)

## 1. Modelul de amenințări

Ce procesează OSATURA: **fișiere de proiect** (JSON), imagini, audio, fișiere de import Lottie / OCA / de tip glTF, fișiere de mapare și, în studio, cereri HTTP de la un browser. Fișierele de proiect sunt *date nesigure* în modelul de mai jos: un proiect primit de la altcineva poate fi ostil, iar deschiderea lui nu trebuie să execute cod, să citească fișiere din afara folderului său sau să epuizeze mașina.

| Actor | Poate | Scop presupus | Apărări principale |
|---|---|---|---|
| Autorul unui proiect / fișier de import ostil | să-ți dea un fișier | să-ți citească fișierele, să blocheze sau să agațe unealta, să ruleze cod, să umple discul | parsare strictă, limite de dimensiune și adâncime, refuzul căilor de resurse, fără panici la intrare, sandbox pentru expresii și scripturi, ieșire atomică |
| O pagină web pe care o vizitezi cât timp studioul rulează pe localhost | să facă browserul tău să trimită cereri către `127.0.0.1` (CSRF, DNS rebinding) | să-ți editeze proiectul, să-l citească, să scrie fișiere | listă de gazde permise pentru Host, verificări Origin / `Sec-Fetch-Site`, tip de conținut JSON, token cu cookie `SameSite=Strict`, CSP |
| Alt utilizator de pe aceeași mașină | să se conecteze la port, să citească `/tmp` | să citească sau să deturneze o sesiune ori fișiere temporare | se leagă implicit la `127.0.0.1`, directoare temporare private (0700), fișiere de jurnal 0600, token obligatoriu pentru a lega în altă parte |
| Cineva din rețeaua ta, când studioul este legat dincolo de localhost | să ajungă la port | să folosească sau să ghicească tokenul, să epuizeze conexiunile | token obligatoriu (16+ caractere), comparație în timp constant, back-off, plafoane de conexiuni și dimensiuni; **fără TLS** |
| Un colaborator cu token | să editeze sesiunea partajată | să strice proiectul | istoricul de anulare, jurnalul înlănțuit prin hash; colaboratorilor li se acordă încredere pentru proiect, nu există conturi sau roluri |
| Apelant ostil al bibliotecilor | să dea argumente greșite fațadei, ABI-ului C sau WASM | corupere de memorie, panici peste graniță | validarea argumentelor, verificări de dimensiune înainte de alocare, panici prinse (unde ținta face unwinding) |

Singura cerere de ieșire pe care o poate face OSATURA este `osatura update-check`, și doar când o rulezi (un singur GET HTTPS anonim pentru lista publică de versiuni, prin `curl` din sistem: doar HTTPS, TLS 1.2+, 30 s, 2 MB, linkul afișat este verificat să fie pe `github.com/chiuta/osatura`). Poate trece prin Tor, Tor Browser, Cloudflare WARP sau propriul tău proxy; presetările sunt puncte locale `socks5h://`, deci DNS nu este rezolvat în afara proxy-ului. Testele sunt în `update_cmd.rs`.

În afara domeniului: protecția împotriva unui administrator local rău intenționat, a unui toolchain de build sau a unei dependențe compromise (vezi [THIRD_PARTY.md](THIRD_PARTY.md) pentru ce se livrează; `cargo deny` supraveghează licențele, interdicțiile și sursele, nu vulnerabilitățile), canalele laterale și securitatea ffmpeg însuși.

## 2. Serverul HTTP al studioului

`osatura studio` servește editorul dintr-o implementare HTTP/1.1 proprie, mică (`osatura-cli/src/http.rs`, doar biblioteca standard, fără framework web terț). Comportament, cu testele care îl fixează (testele unitare `osatura-cli`, `tests/robust.rs`, `tests/wave4.rs`, `tests/source_offer.rs` și testele smoke Chromium):

**Limite**

| Limită | Valoare | Rezultat la depășire |
|---|---|---|
| Timpul de primire a antetului cererii | 10 s | 408 |
| Timpul de primire a corpului | 20 s | 408 |
| Timpul de scriere a unui răspuns | 10 s | conexiune întreruptă |
| Dimensiunea antetului cererii | 16 KB | 431 |
| Numărul de antete | 100 | 431 |
| Dimensiunea corpului | 32 MB | 413 |
| Conexiuni simultane | 128 | 503 |
| Corpuri de cerere chunked | nesuportate | 411 |
| Antet `Host` duplicat | refuzat | 400 |
| Caracter de control (CR, NUL, DEL…) într-o valoare de antet | refuzat (tab-ul este permis) | 400 |

Fiecare răspuns poartă `Connection: close` (nicio stare keep-alive de epuizat), `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` și un `Content-Security-Policy`.

**Politica cererilor** (`Policy::check`, aplicată fiecărei cereri):

1. **Lista de gazde permise pentru Host.** Antetul `Host` trebuie să fie adresa legată (`127.0.0.1`, `localhost`, portul real) sau un nume dat cu `--allow-host`. Altfel, 421 Misdirected Request. Aceasta oprește DNS rebinding: o pagină de pe `evil.example` care se rezolvă la `127.0.0.1` trimite `Host: evil.example`.
2. **Origin și `Sec-Fetch-Site`.** Cererile care poartă un `Origin` de pe alt site, sau `Sec-Fetch-Site: cross-site` / `same-site`, sunt refuzate cu 403. `Origin: https://<allowed host>` este acceptat pentru gazdele din `--allow-host`; în spatele unui reverse proxy care servește pagina la un URL public, dă `--public-origin URL` astfel încât origin-ul proxy-ului să fie acceptat. Cross-origin rămâne refuzat.
3. **Tipul de conținut.** `POST` cere `Content-Type: application/json` (altfel 415), deci un formular HTML simplu nu poate posta către API.
4. **Token** (vezi secțiunea 3), când este configurat; obligatoriu când `--bind` nu este o adresă loopback.

**Oferta sursei.** Studioul servește `/source` (inclusiv când se cere token), iar `--source-url` stabilește adresa (binarele oficiale de lansare folosesc implicit commit-ul din care au fost construite), așa cum cere AGPL-3.0 secțiunea 13 (vezi [LICENSING.md](LICENSING.md)).

**Căile și metodele necunoscute sau malformate** primesc un răspuns de eroare simplu; pagina editorului este compilată în binar (`studio.html`) și nu este citită din sistemul de fișiere, deci nu există o suprafață de path traversal pentru conținutul static.

## 3. Autentificare și colaborare

Opțiunea `--collab-token T` (sau tokenul cerut de un `--bind` non-local) activează controlul accesului; `--collab-log LOG` păstrează persistent o sesiune partajată.

* **Formatul tokenului.** Cel puțin 16 caractere, din `A-Za-z0-9._~+/=-`. Orice conține CR, LF, `;` sau alte caractere este refuzat la pornire (injecție de antete și de cookie).
* **Prezentare.** Antetul `X-Collab-Token: T`, sau cookie-ul `osatura_token` (`HttpOnly`, `SameSite=Strict`, plus `Secure` când cererea a venit sub un nume `--public-origin` de tip `https://`, un `Origin` https sau `X-Forwarded-Proto: https`) setat de `POST /api/login`. Un cookie greșit se numără ca o încercare eșuată, ca un antet greșit.
* **Comparație.** În timp constant (ambele părți sunt hash-uite cu SHA-256 și se compară rezumatele), astfel încât timpul de răspuns nu dezvăluie potriviri de prefix.
* **Back-off.** Per adresă IP de client, începând cu al **5-lea** eșec consecutiv: 2, 4, 8, 16, apoi 30 de secunde de răspunsuri 429; se urmăresc până la 512 de adrese de client. Tokenul corect este mereu acceptat, chiar și în timpul unui back-off (astfel încât un atacator să nu-l poată bloca pe proprietar). Back-off-ul încetinește ghicitul; nu oprește un atac distribuit. **În spatele unui reverse proxy fiecare client apare cu adresa proxy-ului**, deci împart un singur contor de back-off: un val de cereri greșite de la un vizitator întârzie contorul de eșecuri pentru toți, dar tokenul corect tot funcționează.
* **Secrete ale participanților.** Fiecare colaborator primește un secret aleatoriu de 128 de biți (`X-Collab-Secret`) care îi identifică fluxul; cel mult 32 de participanți și 64 de fluxuri de evenimente.
* **Integritatea jurnalului.** Jurnalul sesiunii este un fișier text, o linie pe operație: `<hash> <json>`, înlănțuit cu SHA-256 (format versiunea 2). Editarea, ștergerea sau reordonarea unei linii rupe lanțul, lucru raportat la încărcarea jurnalului. Fiecare cerere este `fdatasync`-ată înainte de răspuns, fișierul este creat cu modul 0600 și rescris atomic. Lanțul detectează deteriorarea accidentală și manipularea ocazională; **nu este semnat**, deci cine poate scrie fișierul poate reconstrui un lanț valid.
* **Tokenuri per participant (valul 7, opțional).** `--collab-secret-file F` înlocuiește singurul bearer partajat cu tokenuri derivate dintr-un secret: `ap1.<name>.<expires>.<mac>`, unde `mac` = HMAC-SHA-256 (RFC 2104, SHA-256 propriu, vectorii de test RFC 4231 în teste) peste nume și termen. Gazda le emite cu `osatura collab-token NAME --secret-file F --ttl SECONDS` (implicit 8 ore, cel mult 30 de zile) și dă fiecărei persoane tokenul ei. Un token este legat de un singur nume (un participant care se alătură primește acel nume indiferent ce trimite pagina, deci nimeni nu se poate alătura drept altcineva), expiră singur, este verificat prin recalculare (nu există tabel pe server, nimic de furat de acolo în afară de secretul însuși), iar cookie-ul său primește un `Max-Age` egal cu durata de viață rămasă. `--collab-revoke NAMES` refuză tokenurile acelor nume înainte de expirare; schimbarea secretului revocă tot. Fișierul secret trebuie să fie privat (mod 600 pe Unix, altfel studioul refuză să pornească), de cel puțin 16 caractere, și nu este acceptat niciodată ca argument de linie de comandă. `--collab-token` și `--collab-secret-file` pot fi combinate; oricare satisface regula că o legare non-loopback are nevoie de o credențială. Expirarea este verificată **per cerere**: un flux de evenimente deja deschis rămâne deschis până se încheie, iar următoarea conexiune este refuzată. Un token expirat sau revocat primește răspuns `401` cu motivul și nu alimentează back-off-ul de ghicire.
* **Limită de rată (valul 7).** `--collab-rate N` permite N cereri pe secundă per adresă de client, cu rafale de 2N (token bucket); mai mult primește răspuns `429` cu `Retry-After`. Implicit este 100 pe secundă când se cere o credențială și fără limită altfel; 0 o dezactivează. Se aplică după autentificare și per **adresă** (sursa unei conexiuni), nu per conexiune TCP, deoarece o limită per conexiune este ocolită prin deschiderea mai multor conexiuni; în spatele unui reverse proxy toți clienții împart adresa proxy-ului, deci limitează și la proxy (secțiunea 12).
* **Avertismente despre adresa de legare (valul 7).** Un `--bind` non-loopback afișează un avertisment la pornire (HTTP simplu; pentru `0.0.0.0` / `::` că studioul ascultă pe fiecare interfață; pentru un token partajat că nu are nume și nu expiră niciodată).
* **Scrierea token-ului.** Termenul de expirare dintr-un token per participant are o singură formă scrisă (cifre zecimale, fără semn, fără zerouri la început, fără `+`): orice altă scriere a aceluiași număr este refuzată ca malformată, iar semnătura este judecată înaintea termenului, astfel încât un token falsificat nu află nimic din răspuns.
* **Gazde (opțional).** `--collab-host NAMES` (necesită `--collab-secret-file`) numește participanții care pot **salva** (`/api/save`), **compacta** jurnalul sesiunii (`/api/collab/compact`) și **revoca** (`/api/collab/revoke`, un `POST` care adaugă nume la lista celor revocate, cel mult 1000). Toți ceilalți, cu un token valid, pot în continuare să editeze și să urmărească, iar la aceste trei cereri primesc `403`. Fără `--collab-host` nu se schimbă nimic: orice deținător al unei credențiale valide poate face toate acestea, ca înainte. Token-ul partajat `--collab-token` nu este legat de un nume și contează ca gazdă.
* **Fluxul de evenimente.** Fluxul de evenimente cere o credențială (un token), dar nu un secret de participant; secretul identifică fluxul propriu al unui participant, nu este o a doua parolă.
* **Conexiuni per adresă.** O adresă de client poate ține cel mult 16 conexiuni simultan (în plus față de plafonul global), astfel încât o singură mașină să nu poată consuma toate locurile. Adresele loopback nu sunt numărate implicit, deoarece un reverse proxy de pe aceeași gazdă ajunge la studio de la `127.0.0.1` pentru fiecare vizitator; limitează per vizitator la proxy (secțiunea 12).
* **Fără TLS, fără conturi.** Serverul vorbește HTTP simplu. Un token care circulă printr-o rețea poate fi citit de oricine se află pe cale. Pune studioul în spatele unui reverse proxy care termină TLS sau al unui tunel SSH / VPN (secțiunea 11). Nu există conturi de utilizator sau permisiuni per proiect; singurul rol este lista opțională de gazde de mai sus, iar orice alt token valid acordă acces de editare.

## 4. Rezolvarea resurselor și limitele de intrare

* **Resursele** (fișiere de imagine, normal map-uri, sunete numite de un proiect) se rezolvă prin `assets::resolve`: relativ la directorul de resurse la nivel de proces (`assets::default_dir`, în mod normal folderul proiectului de intrare). **Căile absolute și orice componentă `..` sunt refuzate**, cu excepția cazului în care se dă flag-ul global `--allow-external-assets`. Fișierele de imagine sunt limitate la 64 MB, cele audio la 256 MB.
* **Fișiere de proiect.** `load_project` citește doar fișiere obișnuite (fără dispozitive, fără pipe-uri), până la 256 MB (`--max-input-bytes N` pentru a schimba). `MAX_JSON_BYTES` al fațadei este 256 MiB.
* **Proiecte derivate.** Comenzile care scriu un proiect derivat validează față de directorul proiectului de *intrare*; când ieșirea este în altă parte, căile relative ale resurselor existente sunt rescrise sau un avertisment spune că ieșirea nu le va găsi (`--allow-external-assets` le face absolute).
* **Randare.** Fațada refuză imaginile mai mari de 16384 px pe latură (`MAX_RENDER_SIDE`) și scări în afara intervalului 0.05–4; straturile C și wasm refuză ieșirea peste 256 MiB înainte de alocare. Alte limite ale fațadei: fps în (0, 1000], cadre de la 1 la 200000, scenă până la 32768, 4096 de oase, magnitudini în limita 1e7, `lod_levels` de la 1 la 12, eșantioane `fuzz` până la 100000, culori `trace` de la 2 la 64 ([API.md](API.md)).
* **Fără panici la intrare.** Testele cu intrări ostile se află în `osatura-cli/tests/robust.rs` și în testele valurilor; testele de tip fuzz acoperă parserele. O panică accesibilă din intrare este tratată ca o vulnerabilitate.
* **Corpurile studioului** sunt limitate la 32 MB; editările trec prin aceeași validare ca fișierele.

## 5. Sandbox-ul pentru expresii și scripturi

* **Expresiile** (legări de valori, constrângeri, drivere) sunt parsate de un parser propriu: **sursă de până la 4096 de octeți, adâncime de imbricare 64**, aritmetică, comparații, variabilele fixe `frame`, `time`, `fps`, `frames`, `pi`, `t`, `get("layer","property")` pentru drivere și un tabel fix de funcții încorporate (`sin`, `abs`, `min`, ...); fără bucle, fără I/O și fără funcții definite de utilizator, deci evaluarea se termină întotdeauna. Rezultatele non-finite sunt respinse de politica valorilor finite.
* **Scripturile** (comanda `script`, feature `scripting`, activ implicit) rulează în Rhai cu aceste limite: **500.000 de operații, 32 de niveluri de apel, adâncimea expresiilor (64 la nivel superior, 32 în funcții), șiruri de 10.000 de caractere, tablouri de 10.000 de elemente, hărți de 1.000 de intrări, 5.000 de cereri API, sursă de script de 64 KB** și `no_module` (fără `import`, deci fără acces la fișiere prin module). API-ul gazdă expus scripturilor este un set fix de funcții de editare a proiectului. Construiește cu `--no-default-features` pentru a elimina complet motorul de scripting (elimină și dependența `smartstring`).
* Un script rămâne cod pe care ai ales să îl rulezi pe un proiect: nu poate citi sau scrie singur fișiere, dar poate face proiectul pe care îl editează foarte mare în limitele de mai sus, iar o buclă infinită se încheie la limita de operații.

## 6. Fișiere, legături simbolice și date temporare

* **Scrieri atomice.** Ieșirile sunt scrise cu `fsutil::write_atomic`: un fișier temporar în același director (nume `.<name>.tmp-<pid>-<random>`), `fsync`, `rename`. O cădere lasă fie fișierul vechi, fie pe cel nou. **Legăturile simbolice date ca ieșire sunt refuzate**, indicând `--follow-symlinks` (aceasta acoperă acum fiecare ieșire adresată de utilizator: rapoartele și goldens din `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, fișierele de poze, snapshot-urile, `vcs`, jurnalul de colaborare și fișierele video scrise de ffmpeg, care sunt pre-verificate), deci o legătură plantată nu poate redirecționa o scriere. Dacă directorul nu poate fi scris, dar fișierul poate, soluția de rezervă este o suprascriere pe loc cu `fsync`, care **nu este atomică** (documentat).
* **Date private.** Jurnalele collab și alte fișiere pe care trebuie să le citească doar proprietarul sunt create cu modul 0600 (`write_atomic_private`). Directoarele temporare sunt create exclusiv cu modul 0700 (`private_temp_dir`), astfel încât un nume previzibil într-un `/tmp` partajat nu poate fi creat în prealabil de alt utilizator. Valorile aleatorii (tokenuri pentru participanți noi, nume temporare) vin din `/dev/urandom` (`fsutil::random_bytes`); generatorul determinist cu seed al motorului nu este folosit niciodată pentru secrete.
* **Setările UI.** `ui.json` al studioului ignoră cheile necunoscute cu un avertisment și este copiat în `ui.json.bak` (apoi `.bak.1`, ...) înainte de prima salvare.
* **Windows / non-Unix.** Modurile de permisiuni de mai sus se aplică pe Unix. Pe Windows și macOS suita de teste trece în CI (verificările doar-POSIX sunt omise pe Windows), dar verificările pentru symlink-uri, permisiuni și `/dev/urandom` rulează doar pe Linux și macOS, iar comportamentul pe Windows nu a fost revizuit manual de nimeni.
* **Binare nesemnate și aplicația cu fereastră.** Executabilele de lansare nu sunt semnate cu cod; verifică descărcările față de `SHA256SUMS.txt`. `osatura-studio` pornește același server local (doar `127.0.0.1`, port liber aleatoriu) și îl afișează în web view-ul sistemului (WebView2 / WKWebView); web view-ul său este la fel de expus ca o filă de browser pe acea adresă și nu a avut o revizuire de securitate separată.

## 7. Programe externe (ffmpeg)

ffmpeg și ffprobe sunt rulate ca procese separate **doar** pentru exportul video, decodarea mp3 / ogg, mixajul coloanei sonore și câteva unelte; nu sunt incluse niciodată ([LICENSING.md](LICENSING.md)). Fiecare invocare:

* transmite argumentele ca tablou (fără shell, fără concatenare de șiruri într-o linie de comandă);
* folosește `-protocol_whitelist file` (o utilizare internă de pipe adaugă `pipe,fd`), astfel încât un fișier media să nu poată face ffmpeg să deschidă `http://`, `concat:` sau alte protocoale de rețea / speciale;
* transmite numele de fișiere prin `fsutil::file_arg`, care prefixează `./` numelor ce încep cu `-` sau conțin `:` ori `|` în prima componentă a căii, astfel încât un nume de fișier să nu poată fi citit ca opțiune sau URL;
* pentru fișierele de sunet `.wav`, `.mp3` și `.ogg` forțează demuxerul cu `-f` (`soundtrack::forced_demuxer`), astfel încât un fișier care nu este ceea ce spune numele lui este refuzat, nu deschis ca un alt container;
* folosește `-loglevel error`.

ffmpeg însuși parsează formate media complexe și a avut propriile vulnerabilități: ține-l actualizat și nu-i da media din surse în care nu ai încredere pe o mașină care deține secrete. Comenzile cu buget de timp (`export-fit --max-seconds`) nu pot întrerupe un singur apel ffmpeg în curs.

## 8. Biblioteci (C ABI, WASM, fațadă)

* **Fațada** validează fiecare argument, returnează `ApiError` și convertește panicile în `ApiError::Internal` acolo unde ținta face unwinding. `Animation` și `Player` sunt `Send + Sync` (verificat la compilare).
* **C ABI** ([C_ABI.md](C_ABI.md)) verifică pointerii nuli, lungimile și UTF-8 și returnează coduri; apelantul trebuie totuși să dea pointeri valizi și să respecte regulile de buffer. Șirul ultimei erori este per thread. ABI-ul a fost exersat sub AddressSanitizer și UndefinedBehaviorSanitizer cu exemplele documentate (curat). Nu este întărit împotriva unui apelant care încalcă contractele documentate (double close, use after close, lungimi greșite).
* **Modulul WASM** ([WASM.md](WASM.md)) se construiește în CI și trece testul wrapper-ului sub Node și a fost rulat o dată în Chromium headless (intrarea greșită dă erori, fără scurgeri în 60 de cicluri deschidere/închidere, scenele supradimensionate sunt refuzate), dar **nu a fost revizuit ca o componentă de browser și nu a fost rulat în Firefox sau Safari**, deci proprietățile lui de securitate sunt cunoscute doar din citirea codului și din acel test. Nu are acces la sistemul de fișiere sau la rețea prin construcție; o panică acolo oprește instanța printr-un trap.
* Niciun strat nu izolează *proiectul* în sandbox: randarea unui proiect ostil este mărginită de limitele de mai sus, dar un proiect poate totuși cere multă muncă în interiorul lor (multe straturi, imagini mari, timeline-uri lungi).

## 9. Lacune cunoscute

Listă sinceră; vezi [LIMITATIONS.md](LIMITATIONS.md) pentru lista mai largă de limite funcționale.

1. **Fără TLS și fără conturi** în studio. Începând cu valul 7 tokenul poate fi per participant și cu expirare (secțiunea 3), ceea ce tot nu este un sistem de conturi: nu există roluri, nu există resetare de parolă și nu există TLS propriu. Nu s-a adăugat TLS nativ deoarece nu există în `Cargo.lock` niciun crate TLS cu licență compatibilă (adăugarea unuia ar aduce o dependență nouă mare și cod criptografic într-un proiect AGPL/MPL fără audit); secțiunea 12 arată calea suportată, un reverse proxy care termină TLS.
2. **Back-off-ul este per adresă** și partajat în spatele unui proxy; încetinește, dar nu previne ghicitul online. Folosește un token lung, aleatoriu (minimul de 16 caractere este un prag, nu o recomandare).
3. **Jurnalul de colaborare este înlănțuit, nu semnat.**
4. **Nu este o bază de cod auditată.** Fără test de penetrare din partea unui terț sau campanie formală de fuzzing; verificările sunt propriile teste și valuri de revizie ale proiectului.
5. **Dependențe.** CI rulează `cargo deny` pentru advisories (vulnerabilități, crate-uri retrase, dependențe directe fără mentenanță), licențe, crate-uri interzise (fără stivă de client TLS sau HTTP) și surse. Notificările de mentenanță pentru dependențele tranzitive sunt revizuite manual (vezi `deny.toml`). Verificarea cunoaște doar advisories publicate până acum; rulează-o din nou înainte de implementare.
6. **Directorul de resurse este la nivel de proces.** `merge` peste mai multe directoare de intrare folosește ultimul director pentru resurse; doar `write_json` rescrie căile resurselor (ceilalți scriitori nu).
7. **Soluția de rezervă neatomică** când directorul de ieșire nu poate fi scris.
8. **Verificările de cale folosesc căi canonice.** `assets::resolve` canonicalizează fișierele existente și refuză un fișier a cărui locație reală este în afara directorului proiectului, deci o legătură simbolică din folderul unui proiect care indică în afara lui este refuzată (testat în `assets.rs`); legăturile care rămân în folder funcționează. O cursă între verificare și citirea ulterioară (time-of-check to time-of-use), făcută de cineva care poate modifica simultan folderul proiectului, nu este exclusă.
9. **ffmpeg** este în afara controlului nostru (secțiunea 7).
10. **WASM** este netestat într-un motor de browser (rulează doar sub Node în CI); modelul de amenințări din browser al unei pagini care îl încorporează (CSP, limite de memorie) nu a fost examinat.
11. **Denial of service prin dimensiune.** Limitele mărginesc cererile și fișierele individuale, nu munca totală: un proiect valid, dar foarte greu, poate ocupa un CPU mult timp (plafonul de conexiuni de mai sus limitează concurența, nu costul unei cereri).

## 10. Raportarea unei vulnerabilități

* **Nu deschide un issue public** pentru o vulnerabilitate. Trimite un raport privat către mentenanță: **`alexio@trom.tf`** (adresa din notificarea `--version` a programului și din documentele de licență), sau folosește raportarea privată de vulnerabilități a GitHub, din fila *Security* a depozitului, când este activată.
* Include: versiunea (`osatura --version`) și platforma, proiectul sau cererea care declanșează problema (un fișier minimal este cel mai bun), ce te așteptai să se întâmple și ce s-a întâmplat, și dacă dorești să fii menționat.
* La ce să te aștepți *(intenție, nu contract: acesta este un proiect al unei singure persoane)*: o confirmare într-un timp rezonabil, o remediere dezvoltată în privat, un test de regresie adăugat în suită, o notă în [CHANGELOG.md](CHANGELOG.md) și menționare dacă dorești. Te rugăm să lași timp pentru o remediere înainte de a publica detalii.
* Versiuni suportate: este suportată doar cea mai nouă pre-lansare (în prezent `v0.1.0-alpha.4`) și arborele curent de pe `main`.
* Rapoartele despre expunerea în rețea a studioului când este legat în mod deliberat la o adresă publică fără reverse proxy sunt mai întâi probleme de documentație: vezi secțiunea 11.

## 11. Listă de consolidare pentru cel care implementează

Pentru oricine rulează `osatura studio` pentru mai mult de o persoană sau în afara localhost.

1. [ ] Preferă să **nu** îl expui. Un singur utilizator: lasă legarea implicită (`127.0.0.1`) și folosește un tunel SSH (`ssh -L 8080:127.0.0.1:8080 host`).
2. [ ] Dacă este expus, **termină TLS** într-un reverse proxy (nginx, Caddy, ...) în fața lui; nu expune niciodată portul HTTP simplu pe Internet.
3. [ ] Folosește un **token lung, aleatoriu** (de exemplu 32+ de caractere din `openssl rand -base64 24`), dă-l printr-un fișier protejat sau prin mediul serviciului, nu prin istoricul shell-ului, și rotește-l când pleacă oameni. Ține minte că este acces complet de editare.
4. [ ] Pornește cu `--bind 127.0.0.1` (sau interfața privată) plus proxy-ul; stabilește `--allow-host your.domain` și `--public-origin https://your.domain` astfel încât verificările Host și Origin să treacă pentru proxy și pentru nimic altceva.
5. [ ] Preferă **tokenuri per participant** (`--collab-secret-file`, secțiunea 3) cu un `--ttl` scurt în locul unui singur token partajat; ține fișierul secret cu modul 600, în afara folderului servit, și numește cu `--collab-host` persoanele care pot salva, compacta și revoca.
6. [ ] Pune proxy-ul să aplice **limitarea ratei** și, dacă se poate, să restricționeze după IP sau să adauge un al doilea strat de autentificare (HTTP basic peste TLS, VPN, proxy SSO), deoarece back-off-ul încorporat vede doar adresa proxy-ului.
7. [ ] Rulează ca **utilizator fără privilegii** într-un director dedicat care conține doar proiectele de servit; fără secrete, fără legături simbolice sensibile. Folosește un sistem de fișiere doar-citire în rest, un container sau sandboxing systemd (`ProtectSystem`, `PrivateTmp`, `NoNewPrivileges`).
8. [ ] Ține `--allow-external-assets` și `--follow-symlinks` **dezactivate**.
9. [ ] Stabilește `--source-url` la adresa de unde poate fi obținută sursa (eventual modificată) (AGPL secțiunea 13).
10. [ ] Păstrează persistent cu `--collab-log` într-un director privat, fă-i copie de rezervă și păstrează modul 0600. Tratează jurnalul ca sensibil: conține întregul istoric al editărilor.
11. [ ] Limitează resursele la nivelul sistemului de operare (memorie, timp CPU, fișiere deschise, cotă de disc); plafoanele încorporate limitează cererile individuale, nu încărcarea totală.
12. [ ] Ține la zi ffmpeg și toolchain-ul Rust folosit la build; rulează `cargo deny check` (sau `cargo audit`) pe `Cargo.lock` pentru lansarea ta.
13. [ ] Nu da serverului proiecte din surse nesigure decât dacă rulează în sandbox-ul de mai sus. Pentru implementări fără scripting construiește cu `--no-default-features`.
14. [ ] Urmărește jurnalele pentru răspunsuri 421, 403, 429 și 503; vârfurile înseamnă sondare sau configurare greșită.

## 12. Reverse proxy cu TLS: nginx și Caddy

Studioul vorbește HTTP simplu. Pentru orice dincolo de propria ta mașină rulează-l pe loopback și lasă un proxy să termine TLS. Aceste rețete au fost scrise pe baza documentației publice a proxy-urilor și a comportamentului studioului descris mai sus; **nu au fost rulate în mediul de redactare** (fără nginx sau Caddy, fără certificate, fără rețea): testează-le înainte de a te baza pe ele.

Pornește studioul pe loopback și numește-i gazda publică, astfel încât verificările Host și Origin să accepte proxy-ul și nimic altceva:

```
osatura studio demo.json --port 8080 --bind 127.0.0.1 \
  --allow-host studio.example.org --public-origin https://studio.example.org \
  --collab-secret-file /etc/osatura/collab.secret --collab-log /var/lib/osatura/session.log
```

Cu un `--public-origin` de tip `https://`, cookie-ul de autentificare primește atributul `Secure`. Fluxul de evenimente (`/api/collab/events`) este Server-Sent Events: proxy-ul nu trebuie să îl pună în buffer și trebuie să permită citiri lungi.

**nginx** (certificate de la CA-ul tău sau `certbot`):

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

**Caddy** (obține și reînnoiește singur certificatul; `Caddyfile`):

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

`flush_interval -1` trimite fluxul de evenimente fără buffer. Caddy setează `X-Forwarded-Proto` și transmite `Host`-ul original. Pentru o limită a ratei cererilor folosește pluginul `caddy-ratelimit` sau pune în față `limit_req` din nginx.

Verificări după pornire: `curl -I https://studio.example.org/` răspunde `401` (pagina de autentificare) și nu servește niciodată studioul fără token; `curl -I http://127.0.0.1:8080/ -H 'Host: evil.example'` răspunde `421`; cookie-ul setat de `/api/login` poartă `Secure; HttpOnly; SameSite=Strict`; jurnalul propriu al studioului pentru sesiune rămâne privat (secțiunea 6). Dacă proxy-ul rulează pe altă mașină, leagă studioul la interfața privată la care ajunge proxy-ul, pune un firewall pe port și ține minte că studioul însuși vorbește tot HTTP simplu pe acea legătură.
