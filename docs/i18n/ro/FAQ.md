<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Întrebări frecvente

*Răspunsuri scurte, fiecare luat din ceva ce a fost rulat sau citit în arborele sursă și în celelalte documente. Documentație CC-BY-4.0. Acolo unde un răspuns depinde de o limită, o spune și trimite la [LIMITATIONS.md](LIMITATIONS.md). Șirurile de eroare din ultima secțiune au fost capturate dintr-un build real; sunt citate exact, astfel încât să poți căuta pe această pagină textul pe care îl vezi.*

Cuprins: [Instalare și configurare](#instalare-și-configurare) - [ffmpeg](#ffmpeg) - [Performanță](#performanță) - [Determinism](#determinism) - [Licențiere și rezultatele tale](#licențiere-și-rezultatele-tale) - [Siguranța fișierelor și căile resurselor](#siguranța-fișierelor-și-căile-resurselor) - [De ce rezultatul trebuie să difere de intrare](#de-ce-rezultatul-trebuie-să-difere-de-intrare) - [Rigging și animație](#rigging-și-animație) - [Studio](#studio) - [Față, vorbire și motion capture](#față-vorbire-și-motion-capture) - [Bug-uri, securitate și contribuții](#bug-uri-securitate-și-contribuții) - [Limitări](#limitări) - [Comparații cu alte instrumente](#comparații-cu-alte-instrumente) - [Tabel de depanare](#tabel-de-depanare)

---

## Instalare și configurare

**1. Cum instalez OSATURA?**
Arhivele precompilate (Windows x86-64, macOS arm64, Linux x86-64 și modulul WebAssembly) sunt pregătite ca pre-lansare ciornă `v0.1.0-alpha.3` pe <https://github.com/chiuta/osatura/releases>; devin descărcabile când proprietarul publică ciorna și nu sunt semnate cu cod (SmartScreen afișează „Unknown publisher”; macOS cere aprobare). Nu există installer: despachetează arhiva și rulează `osatura`. Sau îl compilezi din sursă cu un toolchain Rust stable recent: `cargo build -p osatura-cli` produce `target/debug/osatura`; adaugă `--release` pentru `target/release/osatura`. Vezi [DEVELOPMENT.md](DEVELOPMENT.md).

**2. Ce platforme funcționează?**
Linux x86-64 este locul unde proiectul a fost dezvoltat și exersat. Pe GitHub Actions, suita de teste și clippy trec și pe macOS (arm64) și Windows (x86-64), iar binarele de lansare pentru cele trei sunt construite acolo; dar nicio persoană nu a folosit încă executabilele pentru Windows sau macOS pe un desktop real, deci sunt „testate de CI”, nu „acceptate” ([LIMITATIONS.md](LIMITATIONS.md), secțiunea 1).

**3. Există un installer grafic sau o aplicație?**
Nu există installer. Linia de comandă este produsul. Studio este o pagină web locală servită de `osatura studio`; pe Windows, dublu clic pe `osatura.exe` îl pornește și deschide browserul. Arhivele pentru Windows și macOS conțin și `osatura-studio`, o mică fereastră nativă în jurul aceluiași studio (a fost doar compilată și autotestată în CI); vezi [STUDIO.md](STUDIO.md) și [GETTING_STARTED.md](GETTING_STARTED.md).

**4. Cum încep rapid?**
Urmează [GETTING_STARTED.md](GETTING_STARTED.md): construiește un personaj animat pornind de la un proiect gol, fiecare pas fiind rulat, și se termină cu un GIF.

**5. Pot compila fără motorul de scripting?**
Da: `cargo build -p osatura-cli --no-default-features` lasă deoparte Rhai, deci comanda `script` lipsește (vezi [DEVELOPMENT.md](DEVELOPMENT.md)). Funcționalitatea `scripting` este singurul loc în care intră un crate cu licență copyleft slabă (smartstring); vezi [THIRD_PARTY.md](THIRD_PARTY.md).

**6. Are nevoie de o conexiune la rețea sau de un cont?**
Nu. Programul nu are nevoie de cont și nici de rețea pentru a funcționa (studio nu are conturi). Implicit, studio ascultă doar pe propria ta mașină (vezi întrebarea 51).

**7. Unde își păstrează studio setările?**
În `$OSATURA_UI_CONFIG` dacă este setată, altfel în `$XDG_CONFIG_HOME/osatura/ui.json`, altfel pe Windows în `%APPDATA%\OSATURA\ui.json`, altfel în `~/.config/osatura/ui.json`. Proiectele tale nu sunt atinse niciodată de el.

---

## ffmpeg

**8. Am nevoie de ffmpeg?**
Doar pentru exportul MP4 și WebM, pentru `sound mix`, pentru audio mp3/ogg și pentru `face-track` pe un fișier video. Exportul GIF, APNG, HTML, secvențe PNG, SVG, glTF și JSON nu necesită nimic în plus. Lista completă se află în [CLI_REFERENCE.md](CLI_REFERENCE.md), secțiunea 1.6.

**9. Este ffmpeg inclus în pachet?**
Niciodată. OSATURA pornește `ffmpeg` din `PATH`-ul tău ca program separat și nu se execută nimic altceva. Ce licență și ce situație a brevetelor are build-ul tău de ffmpeg rămâne să verifici tu; vezi [PATENT_NOTES.md](PATENT_NOTES.md).

**10. Ce spune când lipsește ffmpeg?**
`cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` (pentru WebM, sugestia se termină cu `or export an APNG instead`). Nu se scrie nimic.

**11. Ce encodere trebuie să aibă ffmpeg-ul meu?**
`libx264` pentru mp4, `libvpx-vp9` sau `libaom-av1` pentru webm, `aac` și `libopus` pentru sunetul din acestea, `libvorbis` și `libmp3lame` pentru `sound mix`.

---

## Performanță

**12. De ce este atât de lent?**
Probabil rulezi un build debug. Cifrele din documente provin dintr-un build debug și sunt de 10 până la 30 de ori mai lente decât un build release ([CLI_REFERENCE.md](CLI_REFERENCE.md), secțiunea 1.7). Pe mașina de test cu 2 nuclee, un build debug a avut nevoie de 46 s pentru a scrie cele 48 de cadre ale proiectului demo ca PNG și de 57 s pentru a-l exporta ca GIF. Un build release proaspăt a făcut aceleași două lucrări în 1,5 s și 2,2 s. Compilează cu `cargo build --release` și folosește `target/release/osatura`.

**13. Folosește GPU-ul?**
Randatorul implicit este cel pe CPU. Există un crate `gpu` și o comandă `render-gpu` în spatele unui flag de funcționalitate, dar în timpul dezvoltării nu a fost disponibil niciun GPU real: pe un GPU software (llvmpipe), un cadru de 640x480 a durat între 253 și 295 ms, față de 13,8 ms pe calea CPU. Tratează randarea pe GPU ca netestată pe hardware ([LIMITATIONS.md](LIMITATIONS.md)).

**14. Cât costă un rig per cadru?**
Într-un build debug, exemplul `biped` a avut o mediană de 64,5 ms per cadru în verificarea de performanță `ci`. Un bake de cloth pentru un steag mic a durat 0,43 s. Măsoară-ți propriul proiect cu `osatura stats` și `osatura ci`; rig-urile diferă foarte mult.

**15. Cum pot face un proiect greu mai ușor?**
Folosește `lod` pentru a crea variante mai ieftine, `export-fit` pentru a găsi setările de export care se încadrează într-un buget de octeți și `proxy-bench` pentru a compara proxy-urile. Vezi [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) și [EXPORT_FORMATS.md](EXPORT_FORMATS.md). `export-fit` se poate opri mai devreme cu `--max-seconds` și raportează atunci că a rămas fără timp.

**16. Un script cu mii de apeluri eșuează. De ce?**
`osatura script` acceptă cel mult 5000 de cereri per rulare și este totul-sau-nimic: dacă o cerere este respinsă, nu se scrie nimic. Împarte o lucrare mare în mai multe scripturi care citesc rezultatul anterior.

---

## Determinism

**17. Este rezultatul reproductibil?**
Randarea este construită să fie: același proiect și același seed dau aceiași pixeli, iar fizica, particulele și zgomotul au seed. Acest lucru le permite lui `golden` și `ci` să compare randări. A fost verificat doar pe un singur tip de mașină (Linux x86-64); rezultate identice bit cu bit pe alte procesoare sau compilatoare nu sunt promise.

**18. De ce un bake de fizică dă un rezultat diferit după ce schimb un singur număr?**
Bake-ul este determinist pentru o intrare dată, dar este o simulare: orice schimbare de masă, frecvență de cadre, collider sau seed schimbă întreaga rulare. Fă bake cu aceleași setări și același seed pentru a compara. Bake-ul se îmbină implicit cu animația existentă (`merge=keep`: canalele existente sunt păstrate, iar comanda spune `kept existing animation on 'b2' (use merge=replace to overwrite)`); folosește `merge=replace` pentru a suprascrie.

**19. Se repetă zgomotul în buclă?**
Nu. Un canal `noise` nu se repetă de la sine, deci o buclă va arăta un salt la îmbinare. Folosește în schimb chei sau o expresie precum `8 * sin(t * 2 * pi)`; în expresii, `t` merge de la 0 la 1 pe durata clipului, nu în cadre ([ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)).

**20. Pot verifica dacă o animație rulează curat în buclă?**
Rulează `lint`. `make-loop` elimină complet deplasarea: într-un test, o minge care se rostogolea a devenit statică, iar `lint` a avertizat (L005). `loop-solve` închide îmbinarea pentru mișcarea ciclică, dar a lăsat un obiect în deplasare cu un salt vizibil (280 la 280 pe canalul său de deplasare). Pentru o deplasare continuă, folosește modul de extrapolare `loop_offset`.

---

## Licențiere și rezultatele tale

*Acesta este un rezumat în limbaj simplu al [LICENSING.md](LICENSING.md), nu un sfat juridic. O analiză a configurației de licențiere de către un avocat este încă în așteptare.*

**21. Ce licențe se aplică?**
MPL-2.0 pentru biblioteca de bază, C ABI, WASM și crate-urile GPU; AGPL-3.0-or-later pentru programul din linia de comandă și studio (`osatura-cli`); CC-BY-4.0 pentru documentație și pentru resursele și modelele de exemplu sintetice.

**22. Cui aparțin animațiile și randările pe care le fac?**
Ție. Declarația proiectului este că proiectele, animațiile și randările create cu OSATURA îți aparțin în totalitate; licențele se aplică software-ului, nu rezultatului lui. Exportul unui GIF sau rularea unei comenzi nu declanșează nicio obligație.

**23. Pot folosi rezultatul în scop comercial?**
Da, conform declarației de mai sus. Excepția este tot ce copiezi din resursele de exemplu: acestea sunt CC-BY-4.0 și necesită atribuire.

**24. Trebuie să îmi public proiectele dacă folosesc părțile AGPL?**
Nu. AGPL acoperă programul. Dacă rulezi un `osatura-cli` sau un studio modificat astfel încât alte persoane să îl folosească prin rețea, trebuie să le oferi sursa lui. Studio face asta printr-o pagină `/source` și un flag `--source-url` care indică copia ta.

**25. Pot lega biblioteca de bază într-o aplicație cu sursă închisă?**
Nucleul este MPL-2.0, care este copyleft la nivel de fișier: îl poți folosi din cod închis și trebuie să partajezi modificările aduse chiar fișierelor acoperite de MPL. CLI și studio sunt AGPL și sunt cu totul altă situație.

**26. Pot spune că instrumentul meu „funcționează cu OSATURA”?**
Da: „compatibil cu OSATURA” și „realizat cu OSATURA” sunt permise. Un build modificat nu trebuie distribuit sub același nume ([TRADEMARK.md](TRADEMARK.md)).

**27. Pot licenția dublu contribuția mea?**
Nu există un acord de licență pentru contribuitori. Contribuțiile sunt acceptate sub licența proiectului, cu semnătură DCO (`git commit -s`), deci nu există un proprietar care să le poată relicenția. Detalii în [CONTRIBUTING.md](CONTRIBUTING.md).

**28. Ce se întâmplă cu codul terților?**
Aproximativ 360 de crate-uri ale terților (numărate pentru toate platformele; aplicația cu fereastră adaugă toolkitul Windows / macOS), toate sub licențe permisive, cu excepția a șase crate-uri MPL-2.0: smartstring (MPL-2.0 sau ulterioară, doar cu funcționalitatea `scripting`) și cinci pe care doar aplicația cu fereastră le poate aduce. Listele se află în [THIRD_PARTY.md](THIRD_PARTY.md) și [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md).

**29. Există probleme legate de brevete?**
Vezi [PATENT_NOTES.md](PATENT_NOTES.md). Sunt note de inginerie, nu o opinie juridică.

---

## Siguranța fișierelor și căile resurselor

**30. Poate o comandă să îmi deterioreze proiectul?**
Scrierile sunt atomice: rezultatul merge într-un fișier temporar din același director, este sincronizat și redenumit peste țintă, deci o cădere lasă fișierul vechi. Comenzile de editare revalidează proiectul pe care urmează să îl scrie și refuză să scrie unul invalid (`output would be invalid: ...`). Comanda `script` nu aplică nimic dacă vreo cerere eșuează.

**31. Este sigur să deschid un proiect primit de la altcineva?**
Rezonabil, dar nu necondiționat. Un proiect este date, nu cod; singurul program extern pornit de OSATURA este ffmpeg, pentru formatele enumerate în întrebarea 8. Încărcătorul refuză dispozitive, pipe-uri și directoare, limitează dimensiunea intrării și refuză căile de resurse absolute sau cu `..`. Scripturile Rhai rulează într-un sandbox. Totuși, citește [SECURITY.md](SECURITY.md) înainte de a procesa fișiere în care nu ai încredere într-un serviciu.

**32. Unde caută un proiect sunetele și imaginile?**
Relativ la folderul fișierului de proiect. Căile absolute și orice cale cu `..` sunt refuzate, cu excepția cazului în care folosești `--allow-external-assets`, iar trebuie să-l transmiți din nou la fiecare comandă ulterioară care încarcă acel proiect. Exemplu din referință:
`sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)`.

**33. Cât de mare poate fi un proiect pe care îl citește?**
Până la 256 MiB implicit. Schimbă cu `--max-input-bytes N`. Fișierele mai mari eșuează cu `cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)` (cifrele sunt cele din exemplu).

**34. De ce refuză să scrie într-un link simbolic?**
Pentru ca un link plasat lângă rezultatul tău să nu poată redirecționa o scriere în altă parte. Mesajul este `refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself`.

**35. De ce nu pot citi din `<(...)` sau dintr-un pipe?**
Intrările trebuie să fie fișiere obișnuite: `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)`. Scrie mai întâi datele într-un fișier.

**36. Un câmp necunoscut dintr-un proiect îl strică?**
Nu: se afișează `warning: unknown field bogus (ignored)`, iar starea de ieșire este 0. Fișierele auxiliare pe care le scrii pentru o comandă (contracte, mapări, mașini) sunt mai stricte: o cheie necunoscută acolo este o eroare și nu se aplică nimic.

---

## De ce rezultatul trebuie să difere de intrare

**37. De ce `render in.png in.png` spune „refusing to overwrite”?**
Pentru comenzile care citesc un proiect și scriu o imagine, un video sau alt fișier care nu este proiect, numele rezultatului nu poate fi numele intrării. Mesajul este `refusing to overwrite the input project X with output of the same name`. Previne greșeala clasică de a exporta peste unica ta copie a proiectului.

**38. Atunci de ce pot `script` și `autorig` să scrie peste propria intrare?**
Protecția acoperă rezultatele care nu sunt proiecte. Comenzile care citesc un proiect și scriu un proiect (`script`, `autorig`, `loop-solve` și altele) pot indica același fișier pentru intrare și rezultat; scrierea este atomică, iar dacă comanda eșuează nu se schimbă nimic. Dacă vrei să păstrezi originalul, dă un nume nou sau folosește `osatura vcs` pentru a păstra istoricul ([CI.md](CI.md) și [CLI_REFERENCE.md](CLI_REFERENCE.md)).

**39. De ce refuză `render` valoarea `out.jpg`?**
`render` scrie doar `.png` sau `.svg`: `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)`. Pentru alte formate folosește `export`; lista se află în [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

---

## Rigging și animație

**40. Încotro indică zero grade pentru un os?**
Unghiul osului 90 indică în jos; unghiurile nu se măsoară de la axa x așa cum o face un manual de matematică. Canalele sunt diferențe față de poziția de repaus, iar `rest_offset` este în cadrul părintelui ([RIGGING_GUIDE.md](RIGGING_GUIDE.md)).

**41. Lanțul meu IK nu se întinde sau are o smucitură. Ce sunt `soft` și `stretch`?**
`soft` (0 până la 0,9) introduce lin lanțul în extensia completă, iar `stretch` (1 până la 4) îi permite să se lungească. Ambele sunt doar în JSON, nu opțiuni Rhai sau CLI. Cu `stretch` peste 1, zona soft este sfârșitul razei de acțiune întinse. Lanțurile de două oase sunt rezolvate analitic; cele de trei sau mai multe folosesc FABRIK.

**42. Constrângerile mele dau rezultate ciudate.**
Constrângerile rulează în ordinea listei, iar ordinea contează. Un aim cu ponderea 0,5 dă jumătate din unghi (un unghi de -74,7 devine -37,4). Reordonează în JSON dacă o constrângere ulterioară trebuie să vadă rezultatul uneia anterioare.

**43. Există un skinner automat pentru grafica vectorială?**
Nu în linia de comandă. `Project::auto_skin` există doar în bibliotecă. Pentru grafica vectorială faci rigging cu oase părinte ale straturilor, cu lattice-uri, spine-uri sau pini (doar JSON) ori cu mesh-uri trasate prin `mesh-trace`.

**44. `skin-diffuse` și `cloth` se plâng de mesh.**
Ambele au nevoie de un mesh, iar cloth are nevoie de un mesh **grilă**: `cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Un strat fără niciun mesh dă `layer 'flag' has no mesh`.

**45. `foot-lock` spune că are nevoie de o rădăcină.**
`foot lock needs a root controller bone (no parent)`. Osul controller rădăcină nu trebuie să aibă un os părinte.

**46. `scale` îmi refuză cheile.**
`scale: frame 27 is outside the project range 0..=24` înseamnă că o cheie rezultată ar cădea în afara proiectului; `keyedit scale` este refuzat când iese din interval, cu excepția cazului în care adaugi `--free`. O rulare de probă folosește `-` ca nume de ieșire.

**47. De ce a devenit proiectul meu mai lung după `mocap-retarget`?**
`mocap-retarget` extinde durata proiectului pentru a cuprinde mișcarea captată. `follow --chain` poate muta chei dincolo de ultimul cadru, lucru pe care `lint` îl raportează (L007).

**48. Cum animez un strat fără os?**
Editează canalele `layer_x` și `layer_y` ale stratului. În studio, activează auto-key; cu el dezactivat, o editare de strat rămâne statică, cu excepția cazului în care acea proprietate are deja un canal cu chei. Editările de oase creează întotdeauna chei. Vezi [STUDIO.md](STUDIO.md).

---

## Studio

**49. Cum deschid studio?**
`osatura studio project.json` și deschide adresa afișată, în mod normal `http://127.0.0.1:<port>`. Este o aplicație single-page servită de mini-serverul HTTP propriu al OSATURA (doar biblioteca standard); nu are nevoie de internet.

**50. Îmi modifică studio fișierul cât timp lucrez?**
Nu. Fișierul de pe disc se schimbă doar când apeși Save. Închiderea paginii fără salvare pierde editările (jurnalul de colaborare, dacă este activat, păstrează întregul proiect, dar nu istoricul de anulare).

**51. Se pot conecta alte persoane?**
Doar dacă te legi la o adresă non-loopback, iar atunci este necesar un token: `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)`. Tokenul trebuie să aibă cel puțin 16 caractere, de exemplu din `openssl rand -hex 16`. Nu există criptare: folosește un tunel SSH sau un reverse proxy cu TLS.

**52. Unde pun tokenul?**
În antetul `X-Collab-Token` sau în cookie-ul `osatura_token` (HttpOnly, SameSite=Strict și Secure când cererea sosește sub un `--public-origin` https). Nu este acceptat niciodată în URL. De la a cincea încercare eșuată, acea adresă de client primește HTTP 429 cu o întârziere crescătoare (2, 4 ... până la 30 s).

**53. Ce înseamnă erorile HTTP 403, 415 și 421?**
421: antetul Host nu este unul pe care studio îl așteaptă. 403: un `Origin` sau `Sec-Fetch-Site` străin. 415: un POST care nu este JSON. Acestea sunt protecții împotriva paginilor din browserul tău care apelează serverul local; vezi [STUDIO.md](STUDIO.md) și [SECURITY.md](SECURITY.md).

**54. Funcționează în orice browser?**
A fost testat doar în Chromium headless. Celelalte sunt netestate. Nu există o comandă de zoom pe canvas, iar un panou din studio pentru urmărirea feței cu webcam există, dar nu a fost disponibilă niciodată o cameră reală pentru testare ([LIMITATIONS.md](LIMITATIONS.md)).

---

## Față, vorbire și motion capture

**55. Urmărește OSATURA fețe?**
Există un mic detector încorporat, antrenat doar pe fețe sintetice. Pe 50 de scene de test sintetice (seed 9) a găsit fiecare față, cu o eroare medie a reperelor de 0,028 din dimensiunea feței, și a dat 2 alarme false în 50 de scene fără față. Pe desene în stil cartoon s-a descurcat slab: 0 din 6 fețe de roboți unghiulari au fost găsite, iar cazurile emoji rotund și oval de manga au avut erori de 0,164 și 0,166. Nu a fost rulat niciodată pe fotografii reale sau pe o webcam reală. Reține că [LIMITATIONS.md](LIMITATIONS.md) L-13 și textul panoului webcam din studio spun că nu este inclus niciun detector de fețe; acest lucru este depășit în ceea ce privește detectorul sintetic.

**56. Cum funcționează lip-sync?**
Indiciile de gură aleg între cinci forme (repaus, mică, deschisă, rotundă, lată). Un moment de indiciu cu punct zecimal este în secunde; un întreg simplu este un cadru. `--rhubarb` schimbă doar modul în care este clasificată litera E (rotundă în loc de lată). Audio mai lung decât proiectul este trunchiat.

**57. Pot anima pornind de la video?**
`face-track` citește video prin ffmpeg. `face-bake` scrie semnalele ca canale cu chei (rezumatul spune „+N channels (K keys in them), M trigger keys set”) și trunchiază la lungimea proiectului, cu un avertisment pe stderr care numără cadrele nescrise. Un semnal necunoscut dă `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.

---

## Bug-uri, securitate și contribuții

**58. Cum raportez un bug?**
Deschide un issue cu linia de comandă, rezultatul exact (și stderr), starea de ieșire, build-ul (`osatura --version`) și, dacă poți, un proiect minimal. Rulează mai întâi `osatura check project.json` și `osatura lint project.json`: multe probleme sunt raportate acolo. Vezi [CONTRIBUTING.md](CONTRIBUTING.md).

**59. Cum raportez o problemă de securitate?**
În privat, nu într-un issue public; vezi [SECURITY.md](SECURITY.md). Adresa din repository este `alexio@trom.tf`; este aceeași adresă ca a contactului maintainerului și nu a fost înlocuită de o căsuță a unei echipe de securitate.

**60. Există o listă de discuții, un chat sau un contract de suport?**
Nu. Documentele numesc un singur contact al maintainerului; prima lansare este încă o pre-lansare ciornă.

**61. Cum contribui?**
Semnează fiecare commit (`git commit -s`), păstrează testele trecute (`cargo test` și `osatura ci` pentru munca de animație) și citește [DEVELOPMENT.md](DEVELOPMENT.md). Documentația este CC-BY-4.0 și primește cu plăcere corecturi, mai ales acolo unde o pagină și programul nu concordă.

---

## Limitări

**62. Care este cea mai importantă limită?**
A fost construit și testat de un singur proiect într-un singur mediu. Tot ce nu este listat ca rulat este „netestat”. Citește [LIMITATIONS.md](LIMITATIONS.md); are id-uri pe care le poți cita în issue-uri.

**63. Ce formate de export necesită atenție?**
Video necesită ffmpeg. Lottie și glTF acoperă un subset din ce poate OSATURA să deseneze și să anime, deci verifică ce se pierde în [EXPORT_FORMATS.md](EXPORT_FORMATS.md). Eroarea `export` pentru o extensie necunoscută enumeră `.gif .html .json .mp4 .apng .webm .png .glb`, ceea ce omite `.lottie`; `.lottie` (dotLottie) este o țintă validă, vezi [EXPORT_FORMATS.md](EXPORT_FORMATS.md), secțiunea 6.

**64. Este utilizabil build-ul WASM?**
Se compilează pentru `wasm32-unknown-unknown` pe GitHub Actions, iar testul wrapper-ului trece față de modulul real sub Node 20, și este atașat la lansarea ciornă ca `osatura-web.tar.gz`. Nu a fost rulat într-un browser, iar viteza și memoria lui nu sunt măsurate ([WASM.md](WASM.md), [LIMITATIONS.md](LIMITATIONS.md)). Tratează-l ca parțial verificat.

**65. Importă fișierele de proiect ale altor instrumente?**
Importă SVG, OpenRaster, Lottie JSON, imagini, GIF/APNG, motion capture BVH și MIDI ([EXPORT_FORMATS.md](EXPORT_FORMATS.md), secțiunile 9 și 10). Nu importă PSD, fișiere zip `.lottie` sau foldere OCA. Importurile creează un proiect nou. Reține că `import-lottie in.json in.json` nu are protecție la suprascriere și își înlocuiește intrarea. Nu te aștepta ca un fișier importat să se potrivească cu originalul pixel cu pixel.

**66. Este JSON-ul proiectului stabil?**
Este „format 1”. Un fișier mai nou este refuzat: `project format 2 is newer than this build supports (1); update OSATURA`. Cum există doar o pre-lansare alpha, formatul 1 însuși se poate schimba încă înaintea unei lansări stabile; vezi [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

---

## Comparații cu alte instrumente

**67. Prin ce diferă de un editor desktop de animație bazat pe timeline?**
Acelea sunt programe interactive, orientate pe desen. OSATURA este orientată pe comandă: fiecare operație este o comandă sau un script, proiectul este JSON care se poate compara cu diff, iar randările sunt deterministe, deci pot fi testate în CI. Studio este o vedere web subțire peste același motor și este mult mai puțin șlefuit decât un editor matur.

**68. Prin ce diferă de un instrument de animație scheletică 2D pentru jocuri?**
Acele instrumente se concentrează pe mesh-uri, skin-uri și biblioteci runtime pentru motoare de joc specifice. OSATURA are și ea oase, IK, constrângeri și mesh-uri, dar o poveste runtime mai restrânsă: propriul player în HTML, un C ABI și un crate WASM necompilat. Alege în funcție de runtime-ul pe care trebuie să îl vizezi.

**69. Prin ce diferă de un compositor de motion graphics?**
Un compositor este construit în jurul straturilor cu efecte și expresii pe filmări video. OSATURA are efecte de strat și expresii, dar centrul ei este rigging-ul personajelor și grafica vectorială și nu face compoziție video, 3D sau etalonare a culorii filmărilor.

**70. Când este OSATURA alegerea greșită?**
Când ai nevoie astăzi de performanță în timp real pe GPU, de un flux de lucru șlefuit cu tabletă grafică, de animație desenată manual cadru cu cadru, de un produs susținut de un furnizor, cu lansare stabilă și suport, sau de comportament confirmat în afara Linux x86-64.

---

## Tabel de depanare

Fiecare mesaj de mai jos a fost produs de un build real și se termină cu starea de ieșire 1, dacă nu se precizează altfel. Formularea unui mesaj se poate schimba înainte de o lansare.

| Mesaj | Cauză | Remediere |
|---|---|---|
| `cannot read nope.json: No such file or directory (os error 2)` | Cale sau nume greșit | Verifică calea și directorul curent |
| `invalid JSON in broken.json: EOF while parsing a value at line 2 column 0` | Fișierul nu este JSON complet | Corectează sintaxa la linia indicată; restaurează dintr-o copie de rezervă sau cu `osatura vcs` |
| `badparent.json is not a valid project:` apoi `bone 'a' has parent 'ghost' which is missing or listed after it` | Un os părinte lipsește sau este listat după copilul său | Definește părinții înaintea copiilor; corectează numele |
| `invalid JSON in fmt2.json: project format 2 is newer than this build supports (1); update OSATURA` | Fișier dintr-un build mai nou | Folosește un build mai nou |
| `warning: unknown field bogus (ignored)` (exit 0) | O cheie pe care acest build nu o cunoaște | Elimină sau corectează cheia; verifică dacă nu este o greșeală de scriere |
| `refusing to overwrite the input project X with output of the same name` | Numele fișierului de ieșire este egal cu cel de intrare | Alege un alt nume de ieșire |
| `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)` | Extensie greșită pentru `render` | Folosește `.png`/`.svg` sau `export` pentru alte formate |
| `unknown option --bone for 'render' (did you mean --bones?); see: osatura (no arguments) for the usage` | Opțiune scrisă greșit | Folosește opțiunea sugerată |
| `unknown export format '.bmp' (use .gif .html .json .mp4 .apng .webm .png .glb)` | Extensie care nu se poate exporta | Alege un format listat (`.lottie` este de asemenea valid) |
| `--scale needs a number in (0, 16]` | Scară în afara intervalului | Folosește o valoare peste 0 și până la 16 |
| `bad frame number 'x'` | Un argument de cadru nu este număr întreg | Transmite un întreg |
| `cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` | ffmpeg nu este în `PATH` | Instalează ffmpeg sau exportă GIF, HTML ori APNG |
| `the project has no sound tracks (see: sound add)` | Comandă de sunet pe un proiect fără sunet | Adaugă o pistă cu `sound add` |
| `channel hand_ctrl.BoneOffsetY is locked` | Canalul este blocat | Deblochează-l sau editează alt canal |
| `request 1 (hand_ctrl.BoneOffsetY at 0) was rejected, nothing changed: channel hand_ctrl.BoneOffsetY is locked` | La fel, din `script` | Deblochează mai întâi; nu s-a aplicat nimic |
| `request 1 (channel head.BoneAngle) was rejected, nothing changed: ... expression error: unknown variable 'tt'` | Greșeală de scriere într-o expresie | Variabilele sunt frame, time, fps, frames, pi, t |
| `... unexpected end of expression` | Expresie neterminată | Închide parantezele, termină formula |
| `points need an even number of coordinates: 3..1000 points` | Număr impar de coordonate sau prea puține puncte | Dă perechi x,y: de la 3 la 1000 de puncte |
| `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)` | Substituție de proces sau pipe ca intrare | Salvează mai întâi într-un fișier obișnuit |
| `sound add: asset '/etc/passwd': absolute paths are not allowed (...)` | Resursă în afara folderului proiectului | Copiază-l lângă proiect sau folosește `--allow-external-assets` |
| `cannot read demo.json: N bytes exceed the input limit of M bytes (raise it with --max-input-bytes N)` | Fișier peste limita de intrare | Mărește `--max-input-bytes` dacă ai încredere în fișier |
| `refusing to write through a symbolic link; pass --follow-symlinks ...` | Ieșirea este un link simbolic | Indică ținta sau folosește `--follow-symlinks` |
| `cloth needs a grid mesh; the layer has a free-form (traced) mesh` | Cloth pe un mesh trasat | Construiește un mesh grilă pentru strat |
| `layer 'flag' has no mesh` | O comandă de mesh pe un strat simplu | Creează mai întâi un mesh |
| `no bone 'b9'` | Numele osului nu există | Verifică numele cu `stats` sau `explain` |
| `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)` | Nume de semnal greșit | Folosește un nume din listă |
| `foot lock needs a root controller bone (no parent)` | Rădăcina aleasă are un părinte | Folosește un controller fără părinte |
| `scale: frame 27 is outside the project range 0..=24` | Cheile scalate cad în afara proiectului | Lungește proiectul, scalează mai puțin sau folosește `--free` |
| `--extrapolate must be hold, linear, loop, ping_pong or loop_offset` | Mod greșit | Folosește unul dintre cele cinci |
| `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)` | Port ocupat | Alege alt `--port` sau oprește celălalt proces |
| `cannot listen on 127.0.0.1:99999: invalid port value` | Port peste 65535 | Folosește de la 1 la 65535 |
| `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (...)` | Legare publică fără token | Adaugă `--collab-token` sau rămâi pe loopback |
| `--collab-token must be at least 16 characters (e.g. \`openssl rand -hex 16\`)` | Token slab | Generează unul mai lung |
| `--source-url must be an http:// or https:// URL without spaces or quotes` | URL malformat | Folosește un URL simplu |
| `--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org` | Originea are o cale | Dă doar schemă, host și port |
| `output would be invalid: ...` | Editarea ar lăsa un proiect invalid | Citește motivele de după două puncte; nu s-a scris nimic |
| `budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes ...` | `export-fit` nu poate atinge bugetul | Mărește bugetul sau simplifică proiectul (cel mai mic fișier este totuși scris) |
| `time budget exhausted (60 s ...)` | `export-fit` a rămas fără timp | Mărește `--max-seconds` |
| `fuzz: failure rate 98.00% exceeds --max-fail-rate 50.00%` | Pozele au stricat rig-ul, iar ai cerut un prag (`--gate` sau `--max-fail-rate`) | Privește cazurile eșuate; mărește rata doar dacă este intenționat; `fuzz` simplu doar raportează (exit 0) |
| `no .osatura-vcs repository found from ... (run: osatura vcs init <project.json>)` | Încă fără istoric | Rulează `vcs init` |
| `frame 999 out of range` (API studio) | Cadru dincolo de proiect | Folosește un cadru de la 0 la ultimul cadru |
| `bad kind 'x' (vector, raster, png)` (API studio) | Tip greșit | Folosește un tip listat |
| `nothing selected` (API studio) | O comandă de selecție fără selecție | Selectează mai întâi straturi sau oase |

Codurile de ieșire pentru `osatura ci` sunt 0 reușit, 1 eșuat, 2 eroare.

---

*Documentație CC-BY-4.0. Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio). O poți partaja și adapta cu atribuire. Vezi și: [INDEX.md](INDEX.md), [GLOSSARY.md](GLOSSARY.md).*
