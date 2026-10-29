# OSATURA studio

Studio este un editor în browser servit chiar de binarul `osatura`: `osatura studio project.json`.
Are un canvas cu gizmo-uri de selecție și transformare, o listă de straturi, o listă de oase, un dope sheet și un grafic de curbe,
onion skin și traiectorii de mișcare, o paletă de comenzi cu scurtături reasignabile, sugestii de poziții, panouri de intrare live și
de față, butoane de export GIF/HTML/Lottie și o sesiune de editare partajată între mai mulți utilizatori. Nu are conturi, nu are TLS și
nu are bază de date: este un mic server HTTP din biblioteca standard, plasat în fața aceluiași editor pe care îl folosește linia de comandă.

Tot ce se află aici a fost verificat pe binarul real (`osatura 0.1.0`, build debug, Linux) și, pentru interfața cu utilizatorul,
în Chromium headless condus de Playwright. Capturile de ecran au fost făcute astfel (1400x900, proiectul demo încorporat).
Doar **Chromium** a fost testat; vezi secțiunea 11. Pe Windows și macOS, arhivele de lansare conțin și `osatura-studio`, același studio într-o fereastră nativă (compilat și autotestat în CI, încă nefolosit de o persoană; vezi [GETTING_STARTED.md](GETTING_STARTED.md) și [LIMITATIONS.md](LIMITATIONS.md) L-75), iar pe Windows un dublu clic pe `osatura.exe` pornește studioul și deschide browserul (L-76).

Licența acestui document: CC-BY-4.0. Pentru linia de comandă vezi [GETTING_STARTED.md](GETTING_STARTED.md) și
[CLI_REFERENCE.md](CLI_REFERENCE.md); pentru securitatea serverului vezi și [SECURITY.md](SECURITY.md).

![Studio cu proiectul demo: bara de instrumente sus, canvasul cu un strat de braț selectat și oasele lui desenate, panoul Layers în dreapta și timeline-ul dedesubt, cu romburi pentru chei.](../../img/studio-overview.png)

## 1. Pornirea studio

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

Deschide adresa într-un browser. Proiectul este validat la pornire; un fișier care nu trece de `check` este refuzat cu același
mesaj. Câmpurile necunoscute sunt semnalate cu avertisment (`warning: unknown field bogus (ignored)`).

| Flag | Implicit | Semnificație |
|---|---|---|
| `--port N` | 8080 | Port TCP. Un port ocupat dă `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)`; unul greșit dă `cannot listen on 127.0.0.1:99999: invalid port value`. |
| `--bind ADDR` | `127.0.0.1` | Interfața pe care se ascultă. Orice adresă care nu este loopback **necesită `--collab-token`**. |
| `--ui-config F` | `$OSATURA_UI_CONFIG`, altfel `$XDG_CONFIG_HOME/osatura/ui.json`, altfel pe Windows `%APPDATA%\OSATURA\ui.json` (înainte de alpha.3 nu exista niciuna: setările erau păstrate doar pe durata sesiunii), altfel `~/.config/osatura/ui.json` | Unde sunt păstrate layoutul, scurtăturile, comenzile recente și setările overlay-urilor. Este un fișier per utilizator, niciodată parte a proiectului. |

Aplicația cu fereastră și pornirea prin dublu clic creează primul proiect în `OSATURA_DOCUMENTS`, dacă această variabilă este setată, altfel în `Documents\OSATURA` (folosit de pachetul portabil).
| `--allow-host NAMES` | niciuna | Nume `Host` suplimentare de acceptat (separate prin virgulă), pentru accesul sub un nume DNS. |
| `--public-origin URL` | niciuna | Originea simplă (`https://studio.example.org`, fără cale) pe care o prezintă un reverse proxy TLS; `Host` și `Origin` ale ei sunt atunci acceptate. |
| `--source-url URL` | niciuna; în binarele oficiale de lansare, commit-ul din care au fost construite (`https://github.com/chiuta/osatura/tree/<commit>`) | Locul de unde poate fi descărcat codul sursă corespunzător build-ului *tău* (AGPL secțiunea 13); afișat pe `/source` și de `--version`. O valoare explicită câștigă întotdeauna. |
| `--collab-token T` | niciuna | Secret partajat, **cel puțin 16 caractere** (`openssl rand -hex 16`). Necesar pentru legări non-loopback. |
| `--collab-log LOG` | niciuna | Persistă sesiunea de editare partajată într-un fișier și o reia după o repornire. |
| `--collab-nosync` | oprit | Cu `--collab-log`: nu se face `fsync` după fiecare editare (mai rapid; o pană de curent poate pierde ultimele editări). Fără `--collab-log` este refuzat la pornire (`--collab-nosync needs --collab-log`). |

**Salvarea.** Editările stau în memorie. Fișierul de proiect de pe disc se schimbă doar când apeși **Save** (`Ctrl+S`), care scrie atomic
(fișier temporar, apoi redenumire, niciodată printr-un link simbolic). Acest lucru este valabil și în timpul unei sesiuni partajate (secțiunea 8).

## 2. Modelul de securitate (citește înainte de a te lega la o rețea)

Studio este construit pentru o persoană pe propria mașină. Ce îl protejează:

* **Listă de gazde permise.** Se răspunde doar cererilor al căror antet `Host` este unul dintre numele proprii ale serverului; orice altceva primește **421**.
  Cu `--allow-host` și `--public-origin` sunt acceptate mai multe nume.
* **Protecție cross-site.** O cerere cu un `Origin` străin sau cu `Sec-Fetch-Site: cross-site` primește **403**. Un POST al cărui corp nu este
  `application/json` primește **415** (POST-urile fără corp, precum `/api/undo`, nu au nevoie de tip).
* **Antete de securitate** în răspunsuri: un `Content-Security-Policy` restrictiv (`default-src 'self'` și fără framing), `X-Frame-Options: DENY`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`.
* **Token pentru orice în afara mașinii.** Cu `--collab-token`, fiecare cerere are nevoie de token ca antet `X-Collab-Token` sau în cookie-ul
  `osatura_token` setat de `POST /api/login` (HttpOnly, SameSite=Strict). Tokenul **nu este acceptat niciodată în URL**. Tokenurile greșite sunt încetinite
  per adresă de client (de la a cincea eșuare consecutivă, acea adresă primește **429** timp de 2, 4, ... până la 30 de secunde), dar această amânare nu poate opri un
  atacator care continuă să încerce: folosește un token lung și aleatoriu.
* **Secretele participanților.** Intrarea într-o sesiune partajată returnează un secret aleatoriu de 128 de biți; fiecare apel făcut ca acel participant trebuie să îl trimită ca
  `X-Collab-Secret`. Nu este suficient să cunoști numărul unui participant.

Am verificat fiecare dintre acestea cu `curl` împotriva unui server în execuție:

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

Refuzuri la pornire (cod de ieșire 1, nu se pornește nimic):

```
refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)
--collab-token must be at least 16 characters (e.g. `openssl rand -hex 16`)
--source-url must be an http:// or https:// URL without spaces or quotes
--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org
```

**Limite sincere.** Traficul este HTTP simplu: tokenul și tot restul circulă în clar, așa că pune studio în spatele unui reverse proxy TLS
(`--public-origin`) dacă poate fi accesat de oricine altcineva. `Set-Cookie` pentru token primește **flag-ul `Secure`** când cererea sosește sub un nume `--public-origin` `https://` (sau cu un `Origin` https / `X-Forwarded-Proto: https`); HTTP simplu pe loopback nu îl primește niciodată,
iar în spatele unui proxy fiecare client pare să aibă adresa proxy-ului, deci amânarea îi încetinește pe toți deodată. `/source` este deschis fără token
(AGPL cere acest lucru). Nu există conturi de utilizator: oricine are tokenul poate schimba proiectul și poate scrie calea de salvare.

### Oferta de cod sursă AGPL

Studio este software AGPL-3.0-or-later: dacă îl modifici și lași alte persoane să îl folosească printr-o rețea, trebuie să le oferi sursa
versiunii tale. `/source` este acea ofertă; setează `--source-url` la locul unde se află sursa build-ului tău. Fără el (un build făcut de tine cu un simplu `cargo build` nu are niciuna), pagina spune acest lucru și numește maintainerul. Binarele de lansare construite de `.github/workflows/release.yml` poartă adresa propriului commit (`OSATURA_SOURCE_URL` la compilare; un fork care construiește cu propriul workflow trimite la el însuși).

## 3. Un tur al interfeței

### Bara de instrumente

Undo, Redo, **Save**; casete de bifat `bones` (desenează scheletul), `auto-key`, `grid` (spațiere în unități de canvas, 0 = oprit), `snap` (la celelalte straturi și la canvas),
`whole groups` (un clic pe o parte dintr-un grup selectează grupul de nivel superior), `points` (arată ancorele, mânerele, punctele de cage/lattice/pin și vârfurile de mesh ale
straturilor selectate); un selector `box` pentru ceea ce selectează o casetă trasă (`touches`, `contains`, `bounds touch`; pornește cu `touches`); **Lock**; și butoanele **Overlays**, **Suggest**, **Layout**,
**Commands** (paleta) și **Keys** (editorul de scurtături). Un al doilea rând conține numele tău și **Collab**, precum și butoanele de export **GIF**, **HTML player** și **Lottie**,
care descarcă ceea ce ar scrie `osatura export` (playerul HTML include sunetul mixat când există sunet și `ffmpeg` poate produce mp3-ul). Butonul **Simple view** ascunde casetele de editare, Overlays, Suggest și butonul paletei de comenzi, astfel încât rămân doar undo, save, panourile și exporturile; alegerea este reținută în acest browser (sau fereastră), iar bara completă revine cu **Full view**.

### Panouri și layouturi

Fereastra este un dock de panouri, fiecare cu un id stabil: `stage` (canvasul), `layers`, `selection`, `bones`, `live` (Live input), `face` (Webcam face), `json`
(întregul proiect ca JSON editabil, cu un buton *Apply JSON*) și `timeline`. Există patru presetări de layout:

```
animate  stage over timeline on the left, all panels as tabs on the right
rig      stage with a folded timeline; layers+bones and selection+json on the right (live and face hidden)
compact  stage over a tabbed timeline and panels
live     stage and timeline on the left; live, face, bones and selection on the right (layers and json hidden)
```

Schimbă presetările din meniul **Layout** sau din paletă; trage filele pentru a reancora sau a face plutitor un panou; „reset layout” revine la cel implicit.
Layoutul este salvat în configurația UI, per utilizator. `osatura ui layout presets|show NAME|validate FILE.json` afișează și validează aceleași date
(un layout trebuie să lase fiecare panou accesibil; unul incomplet este respins de server cu 400).

### Panoul Layers

![Panoul Layers cu un strat blocat (sun), un strat ascuns (hills) și brațul și piciorul selectate împreună.](../../img/studio-layers.png)

Fiecare rând are două casete de bifat: **visible** și **lock** (salvat *împreună cu proiectul*; straturile blocate sunt ignorate de clicuri și casete și refuză editările).
Captura de ecran arată `sun` blocat, `hills` ascuns și două straturi selectate cu Shift-clic.

### Panoul Selection

Enunță regulile chiar în pagină: *Clic alege elementul din față (cele blocate și ascunse sunt ignorate); Shift-clic comută; tragerea pe spațiu gol
desenează o casetă (ține apăsat Alt pentru un lasso liber, Shift adaugă). Trage caseta pentru a muta, un pătrat pentru a scala (Shift păstrează proporțiile, Alt scalează din
centru), butonul rotund pentru a roti (Shift face snap la 15 grade). Ține apăsat Ctrl în timpul tragerii pentru a opri snapping-ul. Tastele săgeată deplasează cu 1 unitate (Shift: 10),
Delete elimină straturile, Esc anulează o tragere sau golește selecția.*

### Timeline: dope sheet și grafic

![Dope sheet-ul cu o selecție în casetă a trei chei la cadrul 12 (cheile sunt desenate pline, celelalte goale).](../../img/studio-dopesheet.png)

![Vederea grafic de curbe: trei curbe bone_angle (arm_up, arm_low, hand) în culori diferite, cu trei chei selectate la cadrul 12 care își arată mânerele de tangentă; linia de stare indică „3 keys selected | frames 12”.](../../img/studio-graph.png)

`Graph` (sau `Alt+V`) comută între dope sheet și graficul de curbe. Controalele timeline-ului: `Play`, `|<` (primul cadru al intervalului), `loop`, `range` cu
`Set in` / `Set out` (redă doar între in și out), `snap` (rotunjește cheile mutate la cadre întregi), `on collision` (`replace`, `merge`, `refuse` când cheile mutate ajung peste
cele existente), `Copy` / `Paste` / `Duplicate` / `Delete`, `interp` + `Apply` (setează easing-ul cheilor selectate), `Auto-smooth` (face bake de tangente fără overshoot în
cheile selectate), `ripple` N cu `Insert` / `Remove` (și `grow project` pentru a schimba odată cu ele durata proiectului), zoom `-` `+` `Fit` și `shy` (arată canalele marcate shy).

Mouse: clic pe o cheie pentru a selecta, Shift-clic pentru a adăuga, trage o casetă pentru a selecta, trage cheile pentru a le muta; **Alt-tragere sau butonul din mijloc deplasează vederea**, **Ctrl+rotiță face zoom**,
**Shift+rotiță derulează** vertical; în grafic **Shift constrânge tragerea cheii la o singură axă**. Toleranța de snap pentru snapping-ul pe canvas este implicit de 6 unități de canvas.
Fiecare editare de cheie trece prin aceleași operații validate ca `osatura keyedit` (vezi [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)), deci este un singur pas de anulare, iar canalele blocate o refuză.

### Overlay-uri: onion skin, traiectorii, proxy-uri

![Onion skin și traiectoriile de mișcare activate pentru braț la cadrul 14, cu panoul Overlays deschis.](../../img/studio-overlays.png)

`Overlays` (`Alt+7`) conține onion skin (`Alt+N`: cadre înainte/după în nuanțe de roșu/verde, pas fix sau cadre-cheie, mod contur, atenuare, doar straturile selectate),
traiectorii de mișcare (`Alt+M`: trasee ale osului, ale pivotului stratului sau ale punctelor de control, cu marcaje pe cadru) și **proxy-uri de scrub** (`Alt+Q`: imagini de calitate redusă în timp ce tragi
capul de redare, cu un cache de imagini și o copie opțională cu nivel de detaliu). Panoul raportează, de exemplu, `picture cache: 2 frames, 17 KiB, 0 hits / 7 misses, emptied 4x by edits`.
Setările sunt stocate în configurația UI și supraviețuiesc unei reporniri. Overlay-urile nu fac niciodată parte din exporturi.

### Paleta de comenzi și scurtăturile

![Paleta de comenzi deschisă cu Ctrl+K și filtrată după cuvântul „toggle”, cu lista comenzilor potrivite și sugestiile lor de scurtături.](../../img/studio-palette.png)

`Ctrl+K` (sau `Ctrl+Shift+P`) deschide paleta: scrie pentru a filtra (fuzzy, literele potrivite sunt evidențiate), tastele săgeată se mută, Enter rulează, Esc închide. Când nu ai scris nimic,
listează mai întâi comenzile recente. Există **84 de comenzi**; `osatura ui commands` le listează, iar `osatura ui search onion` le ordonează exact cum o face paleta.

![Editorul de scurtături de tastatură deschis cu Ctrl+virgulă, cu comenzile listate pe categorii și combinațiile lor de taste.](../../img/studio-shortcuts.png)

`Ctrl+,` (sau **Keys**) deschide editorul de scurtături: dă clic pe o combinație, apasă noua combinație de taste. Un conflict numește cealaltă comandă și nu schimbă nimic până când
confirmi „Take it over”. Combinațiile rezervate de browser sunt refuzate: `Ctrl+W`, `Ctrl+T`, `Ctrl+N`, `Ctrl+Q`, `Ctrl+R`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+Shift+W`, `Ctrl+Shift+N`, `F5`, `F11`, `F12`,
`Alt+F4`. Reasignarea este stocată în configurația UI și supraviețuiește unei reîncărcări. Serverul respinge cu 400 o mapare cu conflicte, o comandă necunoscută, un layout incomplet sau un corp care nu este obiect.

### Suggest

![Panoul Suggest cu trei poziții intermediare candidate, ordonate (linear, pca, manifold), între cadrele 12 și 24, fiecare cu un scor și un buton Apply.](../../img/studio-suggest.png)

`Suggest` (`Alt+8`) propune poziții învățate din pozițiile-cheie ale proiectului însuși: o poziție intermediară între două cadre, o completare a unei poziții fixate parțial sau „ce urmează de obicei”. Pe proiectul demo, între cadrele 12 și 24 a ordonat `linear` 1,00, `pca` 1,00 și `manifold` 0,99, construite din 5 poziții pe 2 axe (99 % din varianță).
`Apply` scrie poziția ca chei, un singur pas de anulare. Același motor este `osatura pose-suggest` și `/api/suggest`. Acestea sunt metode clasice (amestec liniar, PCA, un manifold cu kernel, o rețea mică opțională pe care o antrenezi singur); cu un număr mic de poziții, cele trei concordă.

### Panourile Live input și Webcam face

* **Live input** (`Alt+4`): mouse-ul peste imagine, tastele, gamepad-ul și Web MIDI sunt mapate la proprietăți printr-un tabel; `Record` face bake unei performanțe în chei, ca un singur pas de
  anulare. Web MIDI are nevoie de un browser care îl suportă (Chromium îl suportă).
* **Webcam face** (`Alt+5`): reperele conduc același tabel de mapare (sursa `face`, id-uri precum `mouth_open`, `eye_r_open`, `head_yaw`). Butoanele sunt
  *Start camera*, *Synthetic face*, *Synthetic face (detector)*, *Calibrate (hold neutral)* și *Face example mapping*. Calea camerei are nevoie de o funcție furnizoare de repere
  pe care o oferi tu (`window.osaturaLandmarkProvider`, vezi comentariul din sursa paginii); videoul nu părăsește niciodată pagina și doar reperele ajung la serverul local.
  Cele două moduri sintetice funcționează din start. *Synthetic face (detector)* randează capul sintetic pe server și îl regăsește cu detectorul încorporat,
  care a fost **antrenat doar pe fețe sintetice** (capete stilizate, nu fotografii); vezi secțiunea despre față din [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Demo ghidat

Butonul **Demo** (comanda `demo.panel`) deschide un demo ghidat în 13 pași care construiește omul care merge din `osatura demo-person`: decor, corp, cap și față, brațe cu cinci degete la fiecare mână, scheletul, picioarele care merg, brațele care se balansează, trunchi și cap, un clipit și un zâmbet, solul în mișcare, o verificare și exportul. Fiecare pas încarcă în editor etapa respectivă a proiectului printr-o singură operație de înlocuire totală, așa că o poți reda, parcurge, poți selecta oase și straturi și poți citi timeline-ul. Etapele sunt tăieturi din proiectul finalizat; comenzile și ieșirea lor afișate în panou au fost rulate cu binarul real.

* **Nimic nu se scrie pe disc.** Save (și Ctrl+S) este dezactivat cât timp demo-ul este încărcat. Singura cale de scriere a studioului, `POST /api/save`, nu este apelată niciodată de demo.
* **Proiectul tău revine.** *Restore my project* readuce proiectul care era deschis înainte (o singură operație de înlocuire totală, un pas de Undo). Dacă închizi fila sau reîncarci pagina cât timp demo-ul este încărcat, următoarea încărcare a paginii observă că serverul încă ține demo-ul, păstrează Save oprit și oferă din nou *Restore my project*; originalul este păstrat pentru asta în stocarea locală a browserului.
* Demo-ul refuză să pornească într-o sesiune de colaborare, deoarece înlocuiește întregul proiect.
* Regenerează etapele cu `node scripts/build-studio-tour.mjs --bin target/release/osatura`, apoi reconstruiește CLI-ul (pagina este încorporată în binar).

## 4. Scurtături de tastatură (implicite)

Aceasta este ieșirea comenzii `osatura ui shortcuts` pentru o configurație nouă: **72 de asocieri, 84 de comenzi, 0 conflicte**. Scurtăturile aparțin unui **context**:
`global` (peste tot), `canvas` (focus pe imagine) și `timeline` (focus pe timeline). Câmpurile de text își păstrează propriile taste.

### Global

| Scurtătură | Comandă | Scurtătură | Comandă |
|---|---|---|---|
| `Space` | redare / pauză (`playback.toggle`) | `Alt+B` | comută oasele |
| `Home` / `End` | primul / ultimul cadru | `Alt+G` | comută grupurile întregi |
| `,` / `.` | cadrul anterior / următor | `Alt+K` | comută auto-key |
| `Shift+,` / `Shift+.` | înapoi / înainte 10 cadre | `Alt+M` | comută traiectoriile de mișcare |
| `[` / `]` | setează începutul / sfârșitul intervalului | `Alt+N` | comută onion skin |
| `Alt+R` | comută intervalul de redare | `Alt+O` | comută redarea în buclă |
| `Alt+1` | comută panoul Layers | `Alt+P` | comută punctele de control |
| `Alt+2` | comută panoul Selection | `Alt+Q` | comută proxy-urile de scrub |
| `Alt+3` | comută panoul Bones | `Alt+S` | comută snap-ul pe canvas |
| `Alt+4` | comută panoul Live input | `Alt+T` | comută panoul Timeline |
| `Alt+5` | comută panoul Webcam face | `Alt+V` | comută dope sheet / grafic |
| `Alt+6` | comută panoul JSON | `Alt+Shift+S` | comută snap-ul pe timeline |
| `Alt+7` | panoul Overlays | `Alt+=` / `Alt+-` | zoom timeline mărire / micșorare |
| `Alt+8` | panoul Suggest | `Ctrl+S` | salvare |
| `Ctrl+K` / `Ctrl+Shift+P` | paleta de comenzi | `Ctrl+,` | scurtături de tastatură |
| `Ctrl+Z` | anulare | `Ctrl+Y` / `Ctrl+Shift+Z` | refacere |

### Canvas

| Scurtătură | Comandă |
|---|---|
| `Arrow keys` | deplasează selecția cu 1 unitate (`Shift`: 10 unități) |
| `Delete` / `Backspace` | șterge selecția |
| `Escape` | golește selecția |
| `Ctrl+A` | selectează tot |
| `Alt+L` | blochează / deblochează straturile selectate |
| `I` | adaugă o cheie la capul de redare (`anim.add_key`) |

### Timeline

| Scurtătură | Comandă |
|---|---|
| `ArrowLeft` / `ArrowRight` | cadrul anterior / următor (`Shift`: 10) |
| `-` / `=` | zoom micșorare / mărire |
| `F` | zoom pentru încadrare |
| `Delete` / `Backspace` | șterge cheile selectate |
| `Escape` | deselectează cheile |
| `Ctrl+A` | selectează toate cheile |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | copiază / lipește / duplică cheile |
| `Alt+ArrowLeft` / `Alt+ArrowRight` | mută cheile selectate cu un cadru mai devreme / mai târziu |
| `Alt+ArrowUp` / `Alt+ArrowDown` | ridică / coboară cheile selectate (valoarea lor) |
| `I` | adaugă o cheie la capul de redare |

Modificatori pe canvas: **Shift** constrânge (blocare pe axă la mutare, pași de 15 grade la rotire, proporțional la scalare, vârful unui os doar se rotește atunci,
iar Shift-clic comută selecția în timp ce o casetă adaugă la ea); **Alt** scalează din centru (iar Alt-tragere pe spațiu gol desenează un lasso); **Ctrl** (Cmd pe macOS)
oprește snapping-ul.

## 5. Selecția și gizmo-urile

Un clic alege elementul din față de sub cursor: un strat, un os (capsula lui, puțin mai lată decât linia desenată) sau, cu `points` activat, un punct de control. Straturile blocate și ascunse
sunt ignorate, astfel că un clic trece prin ele spre ce se află dedesubt. O casetă selectează după modul din selectorul `box`: `touches` (tot ce suprapune caseta), `contains` (doar ce se află complet înăuntru) sau `bounds touch` (casetele de încadrare se suprapun); Alt-tragere desenează în schimb un lasso.

Pentru un singur strat, caseta de transformare urmează axele proprii ale stratului: 8 mânere scalează (laturile pe o axă, colțurile pe ambele; tragerea peste latura opusă răstoarnă cu o scală negativă),
butonul rotund rotește în jurul centrului, iar tragerea dinăuntru mută. Pentru un os, cercul capului mută osul, iar vârful îl rotește (și îl întinde, cu excepția cazului în care ții apăsat Shift).
Pentru punctele de control (ancore, mânere bezier, puncte de deformator, vârfuri de mesh), același gizmo mută punctele selectate. Snapping-ul acoperă o grilă (oprită implicit), marginile și centrele celorlalte straturi
și marginile canvasului, într-o rază de 6 unități de canvas. **O tragere este un pas de anulare** (o operație batch).

## 6. Auto-key: exact ce primește chei

Dacă o editare pe canvas schimbă o valoare *statică* sau creează *chei* se hotărăște astfel. Am verificat cu API-ul HTTP pe proiectul demo (se raportează `mode` al fiecărei editări):

| Situație | Rezultat |
|---|---|
| Editare de strat, `auto-key` oprit, proprietatea nu are canal cu chei | schimbare **statică** a transformării stratului (nu se creează niciun canal) |
| Editare de strat, `auto-key` pornit | proprietățile editate primesc **chei** la cadrul curent (au fost create canale noi `layer_x` și `layer_y`: de la 14 la 16 canale) |
| Proprietate de strat sau de punct care **are deja un canal cu chei**, `auto-key` oprit | primește în continuare **chei** la cadrul curent (o editare la cadrul 9 a adăugat o a doua cheie canalelor de la cadrul 5) |
| Editare de **os** | întotdeauna **cu chei**, indiferent de setarea `auto-key` (au fost create canale de offset de os: de la 16 la 18) |

Astfel, o proprietate animată nu poate fi niciodată suprascrisă pe tăcute de o valoare statică, iar o poziție de os nu se „pierde” niciodată din cauză că ai uitat comutatorul. Pornește `auto-key` (`Alt+K`)
când vrei să animezi o proprietate care este încă statică. Tasta `I` (`anim.add_key`, „Add key at playhead”) adaugă o cheie la cadrul curent.

## 7. Anulare și refacere

Anularea și refacerea (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) lucrează pe operații întregi: o tragere de gizmo, o deplasare cu săgeți, o editare de cheie și un Apply sunt fiecare un singur pas. Fără o sesiune partajată sunt
liniare. **Într-o sesiune partajată, anularea este per utilizator**: anulează doar ultima ta editare și lasă pe loc editările celorlalți (secțiunea 8). Istoricul de anulare nu este salvat împreună cu
proiectul și se pierde când serverul se oprește.

## 8. Editare partajată (colaborare)

### Parcurs: două persoane, un proiect

![Doi utilizatori de browser într-o sesiune: bara de instrumente arată două chip-uri de participanți, Ana (tu) la cadrul 14 și Bo la cadrul 0, iar caseta de selecție și cursorul lui Bo sunt desenate pe canvas în jurul stratului piciorului.](../../img/studio-collab.png)

1. Pornește studio. Pentru persoanele de pe alte mașini trebuie să adaugi un token și să te legi la o adresă de rețea:
   `osatura studio demo.json --bind 0.0.0.0 --port 8080 --collab-token $(openssl rand -hex 16) --collab-log session.log`,
   de preferință în spatele unui proxy TLS (secțiunea 2). Pe o singură mașină nu este nevoie de token.
2. În primul browser, scrie-ți numele lângă **Collab** („Ana”) și apasă **Collab**. Butonul devine **Leave** și apare un chip pentru tine. În al doilea browser
   scrie „Bo” și intră. Fiecare chip arată cadrul curent al participantului (`Ana (you) @14`, `Bo @0`).
3. Selectează lucruri. Cealaltă persoană vede caseta ta de selecție și cursorul în culoarea ta, cu numele tău pe ele (Bo a selectat `leg` și a mișcat cursorul; Ana vede o casetă violetă
   numită „Bo” și un punct lângă `leg` în lista de straturi).
4. Editați în același timp. În test: Ana a deplasat stratul brațului cu 10 unități (`Shift+ArrowRight`); Bo a văzut `[10,0]` apărând în `/api/project`. Apoi Bo a ascuns stratul `sun`, lucru pe care Ana l-a văzut
   imediat.
5. Ana a apăsat `Ctrl+Z`. **Doar deplasarea făcută de Ana a fost anulată** (brațul a revenit la `[0,0]`); soarele ascuns de Bo a rămas ascuns.
6. Editările făcute cât timp cineva lucra dintr-o vedere mai veche sunt îmbinate, nu respinse: am rulat simularea proiectului însuși (`osatura collab-sim`, 6 replici, 600 de editări concurente
   aleatorii, 12 alte ordini de livrare) și toate replicile s-au încheiat identice, iar proiectul îmbinat s-a compilat.
7. Apasă **Save** (sau `Ctrl+S`) în oricare browser pentru a scrie starea partajată în fișierul de proiect; nu se scrie nimic în fișier înainte de aceasta.

Doar Chromium a fost testat, cu două pagini pe o singură mașină. Nu am testat mașini diferite, conexiuni lente sau mai mult de două browsere reale.

### Persistență: `--collab-log`

```
$ osatura studio demo.json --collab-log session.log
...
$ # (an edit is made in a session, the server is stopped and started again)
collaboration log session.log: resumed at index 1 (1 envelopes after the checkpoint)
```

Jurnalul este un fișier pe linii (mod `0600`, doar pentru proprietar) care conține un checkpoint cu lanț de hash-uri al **întregului proiect** (aproximativ 178 KB pentru demo), plus o linie pentru fiecare editare acceptată. Adăugările sunt
`fsync`-ate la fiecare cerere, cu excepția cazului în care se folosește `--collab-nosync`. După o repornire, starea sesiunii este reluată (am schimbat numele proiectului într-o sesiune; după repornire `/api/project` avea în continuare
noul nume), în timp ce fișierul de pe disc a rămas neschimbat (`OSATURA demo`) până la Save. **Participanții și istoricul de anulare nu sunt persistați**: paginile intră din nou cu id-uri noi. Un jurnal deteriorat sau un jurnal
al altui proiect oprește serverul cu o eroare în loc să fie suprascris. O cădere în timpul unei adăugări pierde cel mult o ultimă linie întreruptă.

**Jurnalul conține întregul tău proiect în clar; protejează-l la fel ca proiectul însuși.**

## 9. Referință API HTTP

Pagina studioului folosește aceste rute și le poți folosi și tu (scripturi, teste). Toate corpurile și răspunsurile sunt JSON, dacă nu se precizează altfel; erorile au forma `{"ok": false, "errors": ["..."]}` cu statusul 400 (sau 401, 403,
404, 415, 421, 429, 503, conform descrierii din secțiunea 2). Răspunsurile de succes conțin `"ok": true` acolo unde este indicat. `state` de mai jos înseamnă documentul returnat de `GET /api/state`:
`{bones:[{name,rest_angle}], channels:[...], fps, frames, width, height, can_undo, can_redo, rev, ok}`; `rev` crește la fiecare modificare. Fiecare rută este supusă verificărilor din secțiunea 2.

### Citire

| Rută | Parametri | Răspuns |
|---|---|---|
| `GET /` | | pagina studioului (HTML) |
| `GET /source` | | pagina cu oferta de cod sursă AGPL; se deschide fără token |
| `GET /api/state` | | `state` |
| `GET /api/project` | | întregul JSON al proiectului |
| `GET /api/layers` | | `{ok, layers:[{path, depth, kind, visible, locked, locked_eff, reference, ptr}]}` (`locked_eff` ia în calcul și grupurile blocate de deasupra) |
| `GET /frame/N.svg` | `bones=1` desenează scheletul | un cadru ca SVG (N poate fi fracționar); anteturi `X-Cache`, `X-Render-Ms` |
| `GET /api/frame` | `frame`, `proxy` 0-4 (0 = calitate maximă, 1-4 proxy-uri pentru derulare), `kind` = `vector` (implicit) \| `raster` \| `png`, `bones=1`, `lod=1&screen=WxH&lodpx=P` (desenează dintr-o copie cu detaliu redus, în limita a P pixeli, 0-16) | imaginea; anteturi `X-Cache` (hit/miss), `X-Render-Ms`, `X-Lod-Level`, `X-Lod-Bound-Px`, `X-Proxy-Level`. Un cadru în afara intervalului: `{"errors":["frame 999 out of range"]}`; un kind greșit: `bad kind 'x' (vector, raster, png)` |
| `GET /api/cache` | | `{ok, revision, entries, bytes, hits, misses, invalidations, lod:{builds, engines, invalidations, last_build_ms, levels, unavailable}}` |
| `GET /api/overlays` | `frame`, `sel` (nume de straturi separate prin virgulă) | `{ok, frames, settings, ...}` plus fantomele și SVG-ul cadrului, când se indică un cadru |
| `GET /api/sounds` | | `{ok, total, sounds:[{name, start, gain, duration, peaks, error?}]}` vârfurile formei de undă după tăiere, fade-uri și gain |
| `GET /api/vcs/log` | `limit` (max 1000), `rev` | `{ok, branch, commits:[{hash, author, time, message, parents}]}`; `{"ok":false,"errors":["no .osatura-vcs repository found ..."]}` fără un depozit (vezi `osatura vcs`) |
| `GET /api/vcs/diff` | `a` (implicit `HEAD`), `b` | `{ok, changes}` |
| `GET /api/timeline` | | `{ok, fps, frames, groups, rows, tracks}` modelul dope sheet |
| `GET /api/graph` | `ch` (indecși de canale separați prin virgulă), `f0`, `f1`, `n` (eșantioane, 2-2000, implicit 200) | `{ok, channels, f0, f1, range, skipped}` eșantioanele curbelor și mânerele tangentelor |
| `GET /api/live` | | `{ok, mapping:{params,triggers}, recorded, recording}` |
| `GET /api/ui` | | `{ok, layout, presets, preset_names, default_layout, shortcuts, commands, conflicts, reserved, overlays, recents, panels, persisted, path, warning, ...}` |
| `GET /api/ui/search` | `q`, `limit` (1-200, implicit 50) | `{hits:[{id, title, category, description, bindings, score, positions}]}` clasamentul din paleta de comenzi |
| `GET /export.gif`, `/export.html`, `/export.html.nosound`, `/export.lottie.json` | | fișierul, ca descărcare (`animation.gif`, `player.html`, `animation.json`) |

### Editare

| Rută | Corp | Răspuns |
|---|---|---|
| `POST /api/op` | `{"op":"set","path":"/fps","value":30}` — `op` este `set`, `insert` (cale de tip array, `.../-` adaugă la sfârșit), `remove` sau `batch` (`{"op":"batch","ops":[...]}`, un singur pas de anulare). Căile sunt JSON Pointers în proiect. | `state`. Un rezultat invalid este refuzat cu `{"errors":["invalid project: invalid value: integer `-3`, expected u32"]}` și nu se schimbă nimic |
| `POST /api/undo`, `POST /api/redo` | niciunul | `state` (per utilizator într-o sesiune partajată) |
| `POST /api/key` | `{"target","prop","frame","value","ease"?}` (`ease` de forma `{"type":"smooth"}`; implicit liniar) | `state` |
| `POST /api/delete_key` | `{"target","prop","frame"}` | `state` |
| `POST /api/save` | niciunul | `{"ok":true,"saved":"<path>"}`; scrie atomic. Doar gazda, când studioul rulează cu `--collab-host` (altfel **403**) |
| `POST /api/suggest` | `{"kind":"between"\|"complete"\|"next","a","b","t","frame","constraints","free","top","snap","prior","apply":RANK, "library"?, "model"?}` | `{ok, suggestions}`; cu `apply` și `state` al editării. O cerere fără `kind` dă `bad request: missing field `kind`` |

### Selecție și gizmo-uri (fără stare: pagina păstrează selecția)

O **țintă** (target) este `{"kind":"layer","path":"arm"}`, `{"kind":"bone","name":"hand"}` sau `{"kind":"point","layer":"...","point":...}`.

| Rută | Corp | Răspuns |
|---|---|---|
| `POST /api/pick` | `{"frame","x","y","options"?}` (`options`: `tolerance`, `bone_radius`, `point_radius`, `layers`, `bones`, `point_layers`, `locked`, `locked_bones`) | `{ok, hit, hits:[{distance, target}]}` cel mai de sus primul |
| `POST /api/marquee` | `{"frame","rect":[x0,y0,x1,y1]` sau `"lasso":[[x,y],...]` (între 3 și 5000 de puncte)`,"mode":"bbox"\|"contain"\|"intersect","options"?}` | `{ok, targets}` |
| `POST /api/select` | `{"frame","selection":[...],"targets":[...],"mode":"replace"\|"add"\|"toggle"\|"subtract"}` | `{ok, selection, bounds, gizmo:{corners, handles}, points}` |
| `POST /api/gizmo` | `{"frame","selection","handle","start":[x,y],"delta":[dx,dy],"modifiers":{"constrain","center","no_snap"},"snap":{"grid","objects","canvas","tolerance"},"auto_key","locked","pivot","commit":bool}` — `handle` este `{"type":"move"}`, `rotate`, `{"type":"scale","at":"nw"}`, `bone_head` sau `bone_tip` | fără `commit`: `{ok, drag:{changed, edits, guides, matrix, summary, box_after, outline}}` (o previzualizare); cu `commit:true`: `state` plus `drag` și `selection`, un singur pas de anulare |
| `POST /api/nudge` | `{"frame","selection","delta":[dx,dy],"auto_key","locked"}` | `state` + `drag` (editări cu `mode` de tip `static` sau `key`) + `selection` |
| `POST /api/delete` | `{"frame","selection"}` | `state` + `selection`; `{"errors":["nothing selected"]}` când selecția este goală |

### Timeline

| Rută | Corp | Răspuns |
|---|---|---|
| `POST /api/timeline/select` | `{"selection":[{ch,frame}...],"mode":"replace"\|"add"\|"toggle"\|"subtract","all":true` sau `"channel":N` sau `"group":N` sau `"box":{f0,f1,row0,row1}`, `"include_locked"}` | `{ok, selection:[{ch,frame}...]}` |
| `POST /api/timeline/op` | `{"op":{...},"preview":bool}` — `op` este una dintre `move_keys`, `scale_keys`, `copy`, `paste`, `duplicate`, `delete`, `set_ease`, `set_value`, `add_key`, `ripple`, `offset`, `extrapolate`, `make_loop`, `handle`, `smooth`, `flags` (un nume necunoscut le listează) | cu `preview:true`: `{ok, preview:true, plan}` fără a schimba nimic; altfel `state` + `plan:{changed, notes, op, clip}` |

De exemplu, `{"preview":true,"op":{"op":"delete","keys":[{"ch":0,"frame":12}]}}` returnează planul (`changed: 1`) și lasă proiectul neatins.

### Suprapuneri, live, față, UI

| Rută | Corp | Răspuns |
|---|---|---|
| `POST /api/overlays` | `{"settings":{"onion":{"enabled":true,"before":3}},"frame","selection":[names],"reset":bool}` | `{ok, frames, ghosts, settings, svg}`; setările se salvează în configurația UI |
| `POST /api/trails` | `{"targets":[{"kind":"bone","name":"hand"}],"from","to"` sau `"frame","before","after"`, `"smooth","svg":bool,"hit":{x,y,radius}}` | `{ok, from, to, trails:[{curve,frames,keys,...}], svg?, hit?, path_hit?}` |
| `POST /api/live/mapping` | `{"mapping":{params,triggers},"library"?}` (același format ca `live-map`) | starea live |
| `POST /api/live/push` | `{"events":[{source,id,value,time}],"midi":[raw...]}` | starea live cu efectul evenimentelor |
| `POST /api/live/record` | `{"on":true}` pornește; `{"on":false,"tol":0.01}` oprește și **înglobează performanța în proiect** ca un singur pas de anulare | stare / `state` + rezumatul `live` |
| `POST /api/face/config` | `{"config"?,"calibration"?}` | `{ok}` |
| `POST /api/face/calibrate` | `{"on":true}` / `{"on":false}` | `{ok, frames, calibration}` |
| `POST /api/face/push` | `{"frames":[...landmarks...],"layout"?,"size"?}` (cel mult 10000 de cadre per push) | starea live |
| `POST /api/face/detect` | `{"image":"<base64 PNG/JPEG or data: URI>","track":bool,"push":bool,"t"}` | `{ok, found, layout:"osatura23", size, points, box, note:"detector trained on synthetic faces only"}` |
| `POST /api/face/synth` | `{"t","size":32-512,"seed"}` | `{ok, image:"data:image/png;base64,...", layout, size, points}` un cadru al capului sintetic cu reperele lui exacte |
| `POST /api/ui` | oricare dintre `{"layout":{...}\|null,"shortcuts":{...}\|null,"recents":[...],"overlays":{...},"reset":"layout"\|"shortcuts"\|"overlays"\|"all"}` | documentul `GET /api/ui`; câmpurile necunoscute (`unknown field 'bogus'`), conflictele și aranjamentele incomplete sunt refuzate și nu se schimbă nimic |
| `POST /api/login` | `{"token":"..."}` | `{ok:true}` și `Set-Cookie: osatura_token=...; Path=/; HttpOnly; SameSite=Strict`; token greșit: 401 (429 de la a 5-a eșuare); fără un token configurat `{"ok":true,"token_required":false}` |

### Colaborare

Apelurile făcute ca participant trimit `X-Collab-Replica: <id>` și `X-Collab-Secret: <secret>` (ambele returnate de join). Un id de replică cu secretul greșit sau lipsă primește **403**.

| Rută | Corp / query | Răspuns |
|---|---|---|
| `POST /api/collab/join` | `{"name":"Ana","mode":"thin"\|"replica"}` | `{ok, replica, secret, colour, name, index, mode, clock, users, presence, doc}` (`snapshot` în modul replica). Primul join pornește sesiunea din documentul editorului |
| `POST /api/collab/leave` | `{"replica"}` | `{ok, left}` |
| `GET /api/collab/state` | `replica`, `doc=1`, `snapshot=1` | `state` + `collab_active` (+ users, presence) |
| `GET /api/collab/log` | `since=N` | `{ok, index, entries:[{index, by, envelope}]}` |
| `POST /api/collab/ops` | `{"replica","ops":[...]` sau `"op":{...}`, `"base"?}` sau `{"replica","envelopes":[...]}` | `state` + `applied`, `index`. Participant necunoscut: `unknown participant N: join first`; listă goală: `no ops`; editările invalide ale proiectului sunt refuzate |
| `POST /api/collab/undo`, `.../redo` | `{"replica"}` | `state` + `collab_can_undo`, `collab_can_redo` (doar propriile tale editări) |
| `POST /api/collab/presence` | `{"replica","n","frame","cursor":[x,y],"selection":[names up to 64]}` | `{ok}` |
| `POST /api/collab/compact` | `{}` | rezumatul compactării (`ok`, `logged`). Doar gazda, când studioul rulează cu `--collab-host` (altfel **403**) |
| `POST /api/collab/revoke` | `{"names":["ana","bob"]}` (1 până la 64 de nume) | `{ok, revoked, revoked_total}`: token-urile acelor nume sunt refuzate de acum încolo, iar participanții care s-au alăturat sub ele sunt eliminați. Necesită token-uri per participant (`--collab-secret-file`) și un token de gazdă (`--collab-host`), sau token-ul partajat `--collab-token`; altfel **403**. Un flux de evenimente deja deschis se încheie când clientul se deconectează |
| `GET /api/collab/events` | `since=N` sau `Last-Event-ID` | un flux **server-sent-events**: redă de la `since`, apoi utilizatorii, prezența și fiecare eveniment ulterior; `resync` dacă clientul este prea în urmă; cel mult 64 de fluxuri (`503 too many event streams`) |

Într-o sesiune activă, rutele obișnuite de editare (`/api/op`, `/api/undo`, `/api/redo`, precum și editările proprii ale studioului prin nudge, key sau gizmo) sunt înregistrate și pentru ceilalți participanți.

## 10. Depanare

| Simptom | Cauză și remediu |
|---|---|
| Browserul afișează 421 sau „misdirected request" | Ai deschis studioul sub un nume pe care nu îl cunoaște (un nume DNS, o adresă din LAN). Adaugă-l cu `--allow-host name` sau deschide `http://127.0.0.1:PORT/`. |
| 403 la fiecare cerere în spatele unui proxy | Proxy-ul modifică `Origin`/`Host`. Indică originea publică: `--public-origin https://studio.example.org`. |
| 401 | `--collab-token` este setat: deschide pagina și introdu tokenul o singură dată, sau trimite `X-Collab-Token`. Un token în URL nu este acceptat. |
| 429 | Cinci tokenuri greșite la rând de la adresa ta; așteaptă până la 30 s. În spatele unui proxy toți utilizatorii împart aceeași adresă. |
| `refusing to listen on 0.0.0.0:...` | Bind non-loopback fără token. Adaugă `--collab-token` (16 sau mai multe caractere). |
| `cannot listen on 127.0.0.1:8080: Address already in use` | Alt proces deține portul; alege un alt `--port`. Cauza obișnuită este un studio rămas dintr-o rulare anterioară. |
| Pagina se încarcă, dar imaginea este goală | Verifică terminalul; un proiect care nu trece de `check` este refuzat la pornire. Reîncarcă după remediere. |
| Un mesaj în consola browserului `Failed to load resource ... 404` | Versiunile mai vechi nu aveau `/favicon.ico`; acum este servit. Un 404 rămas provine de la altceva. |
| Textul panoului Face | Panoul spune acum că detectorul antrenat pe date sintetice este inclus, că nu este validat pe fotografii sau webcam-uri și că o webcam reală necesită propriul tău furnizor de repere. |
| Aranjamentul este stricat sau lipsesc panouri | Meniul `Layout`, resetează aranjamentul, sau șterge `ui.json` afișat la pornire. Un fișier de configurare care nu poate fi citit complet este copiat în `ui.json.bak` (apoi `.bak.1`, ...) înainte de a se folosi valorile implicite, iar `GET /api/ui` are un câmp `warning` despre aceasta. |
| Editările mele au dispărut după o repornire | Nu se scrie nimic până la **Save**. Cu `--collab-log`, *sesiunea* supraviețuiește unei reporniri, dar fișierul de pe disc este scris tot doar de Save. |
| `Ctrl+W`, `Ctrl+T` și altele asemenea nu pot fi asociate | Sunt rezervate de browsere (lista în secțiunea 3). |
| Ctrl+Z nu anulează modificarea altcuiva într-o sesiune | Intenționat: anularea este per utilizator. |
| Exporturile cer `ffmpeg` | Doar pentru sunetul din HTML (mp3) și pentru mp4/webm din linia de comandă; GIF, Lottie și HTML fără sunet nu necesită nimic. |

## 11. Accesibilitate și suport pentru browsere

**Browsere.** Interfața a fost testată doar în **Chromium** (versiunea inclusă în Playwright, rulată headless, pe Linux). Firefox și Safari nu au fost încercate și pot funcționa sau nu.
Web MIDI necesită un browser care îl implementează (Chromium îl are); camera necesită `getUserMedia` și un context sigur (HTTPS sau `localhost`). Pagina folosește server-sent events pentru
sesiunea partajată. Pentru tragere se folosesc evenimente de pointer; ecranele tactile nu au fost testate.

**Accesibilitatea, pe față.** Nu am auditat studioul cu un cititor de ecran și nici cu o sesiune doar de la tastatură. Ce există: un atribut `lang` și eticheta meta viewport, casete de bifare etichetate, un
toast cu `role="status"`, câteva atribute ARIA, **un set complet de scurtături de tastatură și o paletă de comenzi** care ajung la orice comandă, scurtături reasociabile, precum și linia de comandă `osatura`, care face
tot ce face editorul, fără pointer. Ce lipsește: canvasul este o imagine și cea mai mare parte a editării pe canvas se face cu pointerul (există deplasare cu tastele săgeți), nu există temă luminoasă și nu există
tratare pentru `prefers-reduced-motion` sau `prefers-color-scheme`, conturele de focus sunt cele implicite ale browserului și nu le-am verificat, iar contrastul culorilor nu l-am măsurat. Dacă depinzi de tehnologii asistive,
preferă CLI-ul și panoul JSON și spune proiectului ce nu funcționează.

## 12. Neajunsuri cunoscute

* Remediate de la prima redactare: textul panoului Webcam face, eroarea 404 la `/favicon.ico`, `--collab-nosync` fără `--collab-log` și lipsa indicatorului de cookie `Secure` în spatele unui `--public-origin` https.
* `osatura render p.json 99 o.png` pe un proiect de 24 de cadre este acum de asemenea o eroare (adaugă `--clamp` pentru a randa ultimul cadru); `/api/frame` din studio refuză un cadru în afara intervalului (`frame 999 out of range`) și acceptă acum doar cadrele `0..frames-1`.
* Modificările de aranjament și de scurtături se păstrează per utilizator în configurația UI, nu în proiect, deci nu însoțesc fișierul.
