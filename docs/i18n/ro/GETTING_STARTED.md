# Primii pași cu OSATURA

Acest ghid te duce de la un checkout proaspăt până la un mic personaj animat pe care l-ai construit, ai făcut rigging, ai creat chei, l-ai făcut să ruleze în buclă, l-ai randat și l-ai exportat folosind exclusiv linia de comandă. Fiecare comandă de mai jos a fost rulată cu adevărat în timp ce s-a scris această pagină; rezultatele afișate sunt cele care au fost tipărite.

Cum a fost verificat: Linux, `rustc 1.97.0`, `ffmpeg 6.1.1` și build-ul **debug** al binarului (`cargo build -p osatura-cli`). Un build release rulează același cod mai rapid. Suita de teste trece și pe macOS și Windows în GitHub Actions; comenzile de pe această pagină au fost rulate manual doar pe Linux. Repository-ul nu declară nicio versiune minimă acceptată de Rust; 1.97.0 este pur și simplu compilatorul care a fost testat.

Licența documentației: CC-BY-4.0. Vezi și [RIGGING_GUIDE.md](RIGGING_GUIDE.md), [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md), [STUDIO.md](STUDIO.md), [FAQ.md](FAQ.md) și lista completă de comenzi din [CLI_REFERENCE.md](CLI_REFERENCE.md).

## 1. Ce îți trebuie

| Cerință | Necesar pentru | Note |
|---|---|---|
| Toolchain Rust (`rustc`, `cargo`) | compilarea OSATURA | instalează de la <https://rustup.rs>; canalul stable |
| Un browser web | studio (`osatura studio`) | doar Chromium a fost testat, vezi [STUDIO.md](STUDIO.md) |
| `ffmpeg` în `PATH` (opțional) | `.mp4`, `.webm`, mixarea sunetului în video/HTML, `sound mix`, `face-track` pe un fișier video | tot restul funcționează fără el |

Fără `ffmpeg`, comenzile care îl necesită se opresc cu un mesaj clar și nu scriu nimic:

```
$ PATH=/nonexistent osatura export buddy-final.json o.mp4
cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead
```

Nu există nicio cerință de GPU. Randarea opțională pe GPU (`render-gpu`) nu face parte din build-ul implicit.

**Nu ai Rust? Folosește o arhivă precompilată.** După ce proprietarul publică pre-lansarea ciornă `v0.1.0-alpha.3`, [pagina Releases](https://github.com/chiuta/osatura/releases) oferă `osatura-windows-x86_64.zip`, `osatura-macos-arm64.tar.gz`, `osatura-linux-x86_64.tar.gz` (glibc 2.35+; versiunile următoare adaugă arhiva statică `osatura-linux-x86_64-musl.tar.gz` pentru distribuțiile mai vechi) și `SHA256SUMS.txt`. Verifică arhiva față de sume (`sha256sum -c SHA256SUMS.txt --ignore-missing`, sau `Get-FileHash` pe Windows), despachetează-o și rulează `osatura` (`osatura.exe`) din folderul despachetat; sari peste compilarea din secțiunea 2 și citește „Windows: dublu clic pe `osatura.exe`” și „Studioul ca fereastră” spre finalul acestei pagini. Programele nu sunt semnate cu cod, deci sistemul te va avertiza prima dată. Tot ce urmează a fost rulat cu un build local al aceleiași surse.

## 2. Compilează-l

Din rădăcina repository-ului:

```
cargo build --release -p osatura-cli
```

Binarul este `target/release/osatura`. Pune-l în `PATH` sau creează-i un alias:

```
export PATH="$PWD/target/release:$PATH"      # or: alias osatura=$PWD/target/release/osatura
osatura --version
```

`--version` afișează versiunea, licența (AGPL-3.0-or-later pentru CLI și studio, MPL-2.0 pentru bibliotecile motorului) și oferta de cod sursă. Rularea `osatura` fără argumente afișează utilizarea completă; `osatura <command> --help` afișează utilizarea unei singure comenzi.

Îl poți rula și fără instalare: `cargo run --release -p osatura-cli -- demo demo.json`.

Compilarea instrumentului din linia de comandă cu `--no-default-features` elimină comanda `script` (motorul de scripting Rhai); acest tutorial folosește `script`, așa că păstrează funcționalitățile implicite.

## 3. Primii pași cu demo-ul încorporat

`osatura demo` scrie un proiect-exemplu complet: un braț cu skin, un picior IK, un strat de tip switch pentru lip-sync, un fundal cu parallax și o mișcare de cameră.

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
```

`check` compilează proiectul exact cum ar face-o randatorul, deci un proiect care trece de `check` poate fi randat. Acum desenează o imagine statică, cu oasele suprapuse, și exportă o animație:

```
$ osatura render demo.json 12 frame12.png --bones
wrote frame12.png
$ osatura render demo.json 12 frame12.svg
wrote frame12.svg
$ osatura export demo.json demo.gif
wrote demo.gif
$ osatura export demo.json demo.html
wrote demo.html
$ osatura export demo.json demo.lottie
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
wrote demo.lottie
```

Ce tocmai ai făcut:

* `render <project> <frame> <out.png|out.svg>` desenează un singur cadru. `--bones` suprapune scheletul, `--scale N` randează la de N ori dimensiunea canvasului (0 < N <= 16).
* `export` alege formatul după extensia fișierului: `.gif`, `.html` (o pagină de player autonomă), `.json` (Lottie), `.lottie` (dotLottie), `.apng`, `.webm`, `.mp4`, `.glb`, `.png` (sprite sheet).
* Liniile `warning:` sunt partea sinceră: Lottie nu poate reprezenta tot ce poate OSATURA. Fiecare limitare este afișată pe stderr, iar `--report r.json` scrie aceeași listă ca JSON.
* Deschide `demo.html` într-un browser pentru a reda animația; nu este implicat niciun server și nicio rețea.

Pentru a explora interactiv:

```
osatura studio demo.json --port 8080
```

și deschide <http://127.0.0.1:8080/>. Vezi [STUDIO.md](STUDIO.md).

## 4. Un tutorial de 15 minute: un personaj care salută

Vei construi „Buddy” pornind de la un proiect gol. Fișierul de proiect este JSON, dar nu trebuie să scrii niciodată mai mult decât un antet: comanda `script` rulează un mic script [Rhai](https://rhai.rs) izolat (sandbox) prin același editor validat pe care îl folosește studio. Dacă o cerere dintr-un script este respinsă, *nu* se aplică nimic (totul sau nimic), deci un script defect nu lasă niciodată un fișier editat pe jumătate.

Lucrează într-un director gol. Fiecare pas scrie un fișier nou numerotat, astfel încât te poți întoarce oricând.

### Pasul 1: un proiect gol (1 minut)

```
cat > buddy.json <<'EOF'
{"format": 1, "name": "Buddy", "width": 320, "height": 240, "fps": 24, "frames": 24}
EOF
osatura check buddy.json
```
```
ok: 'Buddy' 320×240 @ 24 fps, 24 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Cele șase chei reprezintă minimul. 24 de cadre la 24 fps înseamnă o secundă, care va fi bucla noastră.

### Pasul 2: forme (2 minute)

Straturile sunt desenate din spate către față, în ordinea în care le adaugi. Creează `shapes.rhai`:

```
add_gradient_rect("sky", 0, 0, 320, 240, "#bfe3f5", "#fdf3d8");
add_rect("ground", 0, 200, 320, 40, "#7fa66b");
add_rect("torso", 140, 120, 40, 70, "#e07a5f");
add_circle("head", 160, 95, 24, "#f2cc8f");
add_rect("upper_arm", 160, 134, 40, 12, "#3d405b");
add_rect("forearm", 200, 134, 40, 12, "#81b29a");
```
```
$ osatura script buddy.json 01-shapes.json shapes.rhai
6 request(s) applied; wrote 01-shapes.json
```

Coordonatele sunt unități de canvas, cu originea în colțul din stânga sus și axa y îndreptată în jos. Brațul este desenat orizontal, cu umărul în (160, 140), pentru că aceasta este poziția în care îi vom construi oasele.

### Pasul 3: oase, relații părinte-copil și atașarea desenului (3 minute)

Un os are un nume, un părinte (gol = rădăcină), o lungime, o poziție a capului (`x`, `y`) în spațiul părintelui (pentru o rădăcină: în canvas) și un unghi de repaus în grade relativ la părinte. Unghiul 0 indică de-a lungul axei +x, iar unghiurile pozitive rotesc în sens orar pe ecran, deci 90 indică în jos și -90 indică în sus. Creează `bones.rhai`:

```
add_bone("body", "", 70, 160, 190, -90);
add_bone("neck", "body", 20, 70, 0, 0);
add_bone("arm_up", "body", 40, 50, 0, 90);
add_bone("arm_low", "arm_up", 40, 40, 0, 0);
add_bone("hand_ctrl", "", 10, 210, 165, 0);
set_layer("torso", "bone", "body");
set_layer("head", "bone", "neck");
set_layer("upper_arm", "bone", "arm_up");
set_layer("forearm", "bone", "arm_low");
```
```
$ osatura script 01-shapes.json 02-bones.json bones.rhai
9 request(s) applied; wrote 02-bones.json
$ osatura render 02-bones.json 0 rig.png --bones --scale 2
wrote rig.png
```

Citește oasele astfel: `body` este un os rădăcină în (160, 190), care indică în sus (-90) și are lungimea 70. `neck` este copilul lui; capul lui se află la 70 de unități de-a lungul lui `body` (în vârf). `arm_up` este un alt copil al lui `body`, la 50 de unități pe coloană (umărul), rotit cu +90 astfel încât să indice lateral. `arm_low` este copilul lui `arm_up`: relația părinte-copil înseamnă că rotirea brațului superior poartă după ea și antebrațul. `hand_ctrl` este un os rădăcină separat, pe care îl vom folosi ca țintă IK; nu are desen.

`set_layer(layer, "bone", bone)` face ca un strat să se miște *rigid* împreună cu un os: urmează mișcarea osului relativ la poziția lui de repaus. Aceasta este cea mai simplă metodă de a face rigging unui desen animat compus din piese separate. (Pentru forme flexibile aplici skin ancorelor vectoriale pe mai multe oase; vezi [RIGGING_GUIDE.md](RIGGING_GUIDE.md).)

### Pasul 4: cinematică inversă (1 minut)

În loc să creezi chei pentru unghiurile umărului și cotului, lasă solverul să ajungă la `hand_ctrl`. Creează `ik.rhai`:

```
add_ik("arm_ik", ["arm_up", "arm_low"], "hand_ctrl", 1);
```
```
$ osatura script 02-bones.json 03-ik.json ik.rhai
1 request(s) applied; wrote 03-ik.json
```

Ultimul argument este direcția de îndoire: +1 sau -1 alege în ce parte se pliază cotul. Un lanț de două oase folosește un solver analitic; lanțurile de trei sau mai multe oase folosesc FABRIK. De acum încolo, mutarea lui `hand_ctrl` mută întregul braț.

### Pasul 5: chei și easing (3 minute)

`set_key(target, property, frame, value, ease)` creează chei. Pentru un os, `bone_angle`, `bone_scale`, `bone_offset_x` și `bone_offset_y` sunt *diferențe față de poziția de repaus*, deci o valoare de cheie 0 înseamnă „repaus”. Numele easing-ului se aplică segmentului care *începe* la acea cheie. Creează `keys.rhai`:

```
set_key("hand_ctrl", "bone_offset_y", 0, 0, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 12, -75, "ease_in_out");
set_key("hand_ctrl", "bone_offset_y", 24, 0, "ease_in_out");
set_key("neck", "bone_angle", 0, 0, "smooth");
set_key("neck", "bone_angle", 16, 10, "smooth");
set_key("neck", "bone_angle", 24, 0, "smooth");
```
```
$ osatura script 03-ik.json 04-keys.json keys.rhai
6 request(s) applied; wrote 04-keys.json
$ osatura timeline 04-keys.json
timeline: 24 frames at 24 fps, 2 channels
Bones
  hand_ctrl
    #0 bone_offset_y (keys @ 0 12 24)
  body
    neck
      #1 bone_angle (keys @ 0 16 24)
```

Numele de easing care nu necesită parametri sunt `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`, `smooth` și `bounce`. `bezier:x1,y1,x2,y2` este acceptat de `keyedit`, iar `spring` și `back` primesc parametri (vezi [PROJECT_FORMAT.md](PROJECT_FORMAT.md)). Mâna și capul folosesc intenționat temporizări diferite: acțiunea suprapusă dă senzația de viață.

### Pasul 6: fă-l să ruleze în buclă (1 minut)

Ambele canale încep și se termină cu aceeași valoare (cadrul 24 repetă cadrul 0), de ceea ce are nevoie o buclă. `loop-solve` verifică îmbinarea și setează canalele în buclă:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
$ osatura lint buddy-final.json
0 finding(s), 0 error(s)
```

„seam 0.000 -> 0.000” înseamnă că ultima poză este deja egală cu prima, deci nu a trebuit corectat nimic. Un proiect de N cadre redă cadrele 0..N-1 în mod repetat, deci cadrul N este aceeași poză ca cadrul 0 și nu este afișat niciodată de două ori. Dacă creezi o buclă astfel încât *ultimul cadru afișat* să fie egal cu primul, folosește `--inclusive` (convenția mai veche). Dacă animația ta are deplasare netă, `make-loop` o elimină. Vezi [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

`lint` rulează verificări statice de rig și animație (ținte IK imposibil de atins, canale moarte, oase lipsă...); se încheie cu codul 1 când găsește erori, deci este sigur de pus într-un script sau într-un job de CI.

### Pasul 7: privește-l, randează-l (2 minute)

```
$ osatura stats buddy-final.json
```
afișează un rezumat JSON (aici: 5 bones, 1 ik chain, 6 layers, 2 channels, 6 keys). Randează imagini statice și o secvență completă de cadre:

```
$ osatura render buddy-final.json 12 frame12.png --scale 2
wrote frame12.png
$ osatura render buddy-final.json 12 frame12.svg
wrote frame12.svg
$ osatura frames buddy-final.json frames --png
wrote 24 frames to frames
$ ls frames | head -3
frame_0000.png
frame_0001.png
frame_0002.png
```

![Patru cadre (0, 6, 12, 18) din animația Buddy, o dată simple și o dată cu oasele suprapuse. Brațul drept se ridică într-un salut cu cotul în jos, în timp ce capul dă ușor din cap.](../../img/tutorial-buddy.png)

Fără `--png`, `frames` scrie fișiere SVG. Randările sunt deterministe: același proiect și același cadru dau aceiași octeți pe orice mașină cu același build, ceea ce face posibile verificările cu cadre de referință din [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Pasul 8: export (2 minute)

```
$ osatura export buddy-final.json buddy.gif
wrote buddy.gif
$ osatura export buddy-final.json buddy.html
wrote buddy.html
$ osatura export buddy-final.json buddy.json --report buddy-report.json
wrote buddy.json
$ cat buddy-report.json
{
  "count": 0,
  "format": "lottie",
  "warnings": []
}
$ osatura export buddy-final.json buddy.mp4        # needs ffmpeg
wrote buddy.mp4
$ osatura export buddy-final.json buddy.apng
wrote buddy.apng
```

Dimensiunile fișierelor acestui proiect minuscul: GIF 145 KB, Lottie JSON 21 KB, player HTML 50 KB, APNG 75 KB, MP4 6 KB. `buddy.json` este Lottie; are o listă goală de avertismente pentru că Buddy folosește doar funcționalități pe care Lottie le poate exprima. Fișierul HTML este un player autonom (deschide-l în orice browser).

Câteva reguli de reținut:

* `export` și `render` **refuză să suprascrie proiectul de intrare** (`refusing to overwrite the input
  project buddy-final.json with output of the same name`) și refuză să scrie printr-un link simbolic, cu excepția cazului în care folosești `--follow-symlinks`. Comenzile care transformă un proiect în altul (`script`, `loop-solve`, `walk`, `autorig`...) *nu* au această verificare: scriu atomic, deci `osatura script p.json p.json x.rhai` editează pe loc. Folosește nume distincte până când ai încredere într-un script.
* GIF are 256 de culori și transparență pe 1 bit; pentru alfa fin folosește `.apng` sau `.webm`.
* Rulează `osatura export-fit` (vezi [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) când trebuie să te încadrezi într-un buget de octeți.

### Pasul 9 (opțional): editează-l în studio

```
osatura studio buddy-final.json --port 8080
```

Trage straturile, selectează o cheie în dope sheet, apasă `Space` pentru redare. Nu se scrie nimic în fișierul tău până când nu apeși **Save** (`Ctrl+S`).

## 5. Cum se îmbină piesele

| Concept | În fișier | Îl creezi cu |
|---|---|---|
| Strat (vector, imagine, grup, switch...) | `layers` | `add_rect`, `add_circle`, `add_polygon`, `add_line`, `import-svg`, `autotrace`... |
| Os | `bones` (părinții înaintea copiilor) | `add_bone`, `set_parent`, `reparent` |
| Lanț IK / constrângere / arc | `ik`, `constraints`, `springs` | `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`; arcuri prin JSON |
| Canal (o proprietate animată) | `channels` | `set_key`, `keys`, `keyedit`, expresii, zgomot, bake-uri |
| Clip, take, mașină de stări, binding | `clips`, `machines`, `bindings` | `take-record`, `clip`, `machine`, `bind` |

Totul este JSON simplu, motiv pentru care diff-urile (`osatura diff a.json b.json`), controlul versiunilor (`osatura vcs`) și scripturile funcționează. Câmpurile necunoscute sunt raportate ca avertismente și ignorate:

```
$ osatura check unk.json
warning: unknown field bogus (ignored)
ok: 'X' 100×100 @ 24 fps, 10 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Un proiect poate face referire la fișiere de imagine și de sunet doar **din propriul director**. Căile absolute și `..` sunt refuzate, cu excepția cazului în care folosești `--allow-external-assets` (vezi [FAQ.md](FAQ.md)).

## 6. Încotro mai departe

* Fă rigging la ceva flexibil (skin-uri, greutăți, lattice, cage, cloth, fizică): [RIGGING_GUIDE.md](RIGGING_GUIDE.md).
* Cicluri de mers, stive de clipuri, lip-sync, mocap, mașini de stări, QA și bugete de dimensiune: [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).
* Editorul din browser, scurtăturile lui și API-ul HTTP: [STUDIO.md](STUDIO.md).
* Întrebări și mesaje de eroare: [FAQ.md](FAQ.md). Limite cunoscute: [LIMITATIONS.md](LIMITATIONS.md).
* Fiecare comandă și opțiune: [CLI_REFERENCE.md](CLI_REFERENCE.md); formatul fișierului: [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Windows: dublu clic pe `osatura.exe`

`osatura.exe` este un program în linia de comandă. Începând cu 0.1.0-alpha.2, deschiderea lui din Explorer (dublu clic) creează `Documents\OSATURA\first-rig.json` din demo-ul cu persoana care merge (`osatura demo-person`; un fișier existent nu este niciodată suprascris), pornește studioul pe `http://127.0.0.1:8080` și îl deschide în browser; fereastra de consolă rămâne deschisă cât timp rulează studioul, închide-o pentru a-l opri. Deschis dintr-un terminal fără argumente, afișează utilizarea ca înainte. În 0.1.0-alpha.1 un dublu clic afișa doar utilizarea, iar fereastra se închidea imediat.

Executabilele **nu sunt semnate cu cod**: Windows SmartScreen afișează „Unknown publisher” (alege *More info*, apoi *Run anyway*), iar macOS Gatekeeper îți cere să aprobi programul în *System Settings > Privacy & Security*. Verifică mai întâi descărcarea față de `SHA256SUMS.txt`. Dacă portul 8080 este ocupat, rulează `osatura studio first-rig.json --port 8090` dintr-un terminal.

## Studioul ca fereastră: `osatura-studio` (Windows și macOS)

Arhivele pentru Windows și macOS conțin un al doilea program, `osatura-studio` (`osatura-studio.exe`), o mică fereastră nativă în jurul studioului din browser: pornește motorul `osatura` din același folder ca `studio` pe un port local liber, afișează pagina într-o fereastră WebView2 (Windows 11 îl are încorporat; Windows mai vechi necesită runtime-ul Evergreen) sau WKWebView (macOS) și oprește serverul când fereastra este închisă. Nu apare nicio fereastră de consolă. Păstrează-l în același folder cu `osatura(.exe)`.

* **Exporturile** (GIF, player HTML, Lottie) deschid un dialog nativ „Save as” care pornește în folderul proiectului, iar un mesaj numește fișierul când este salvat. (alpha.2 le salva silențios în folderul Downloads; alpha.3 întreabă.)
* Fă dublu clic pe el pentru a lucra la `Documents/OSATURA/first-rig.json` (creat din demo-ul cu persoana care merge, `osatura demo-person`, dacă lipsește) sau pornește-l ca `osatura-studio my-project.json` (de exemplu prin „Open with”).
* `osatura-studio --selftest` pornește serverul, descarcă pagina și îl oprește fără fereastră; codul de ieșire este 0 când a mers (CI îl rulează pe Windows și macOS).
* **Limite.** Fereastra a fost folosită pe un desktop real Windows 11 (alpha.2: studioul și exporturile au funcționat); dialogul Save-as din alpha.3 este compilat și autotestat în CI, dar nimeni nu l-a încercat încă, iar fereastra nu a fost folosită de nimeni pe macOS. Celelalte funcții în stil browser care au nevoie de sistem (drag-and-drop din Explorer sau Finder, deschiderea fișierelor din selectoarele proprii studioului) sunt cele ale web view-ului sistemului și sunt netestate. Nu există fereastră pe Linux (ar necesita pachete de sistem GTK / WebKitGTK): folosește `osatura studio` și un browser. Programul nu este semnat cu cod (vezi mai sus).
