# Erste Schritte mit OSATURA

Diese Anleitung führt dich von einem frischen Checkout zu einer kleinen animierten Figur, die du
ausschließlich über die Kommandozeile gebaut, geriggt, mit Keys versehen, geloopt, gerendert und
exportiert hast. Jeder Befehl unten wurde beim Schreiben dieser Seite wirklich ausgeführt; die
gezeigten Ausgaben sind die Ausgaben, die tatsächlich ausgegeben wurden.

So wurde es verifiziert: Linux, `rustc 1.97.0`, `ffmpeg 6.1.1` und der **Debug**-Build der Binärdatei
(`cargo build -p osatura-cli`). Ein Release-Build führt denselben Code schneller aus. Die Testsuite besteht außerdem unter macOS und Windows in GitHub Actions; die Befehle auf dieser Seite wurden nur unter Linux von Hand ausgeführt. Im Repository ist
keine minimal unterstützte Rust-Version festgelegt; 1.97.0 ist einfach der Compiler, der getestet wurde.

Dokumentationslizenz: CC-BY-4.0. Siehe auch [RIGGING_GUIDE.md](RIGGING_GUIDE.md),
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md), [STUDIO.md](STUDIO.md), [FAQ.md](FAQ.md) und die vollständige
Befehlsliste in [CLI_REFERENCE.md](CLI_REFERENCE.md).

## 1. Was du brauchst

| Voraussetzung | Benötigt für | Hinweise |
|---|---|---|
| Rust-Toolchain (`rustc`, `cargo`) | OSATURA bauen | Installation über <https://rustup.rs>; Stable-Kanal |
| Ein Webbrowser | das Studio (`osatura studio`) | nur Chromium wurde getestet, siehe [STUDIO.md](STUDIO.md) |
| `ffmpeg` im `PATH` (optional) | `.mp4`, `.webm`, Ton in Video/HTML mischen, `sound mix`, `face-track` auf einer Videodatei | alles andere funktioniert ohne |

Ohne `ffmpeg` brechen die Befehle, die es brauchen, mit einer klaren Meldung ab und schreiben nichts:

```
$ PATH=/nonexistent osatura export buddy-final.json o.mp4
cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead
```

Eine GPU wird nicht vorausgesetzt. Der optionale GPU-Renderer (`render-gpu`) ist nicht Teil des Standard-Builds.

**Kein Rust? Nimm ein fertiges Archiv.** Sobald der Eigentümer die Entwurfs-Vorabversion `v0.1.0-alpha.3` veröffentlicht hat, bietet die [Releases-Seite](https://github.com/chiuta/osatura/releases) `osatura-windows-x86_64.zip`, `osatura-macos-arm64.tar.gz`, `osatura-linux-x86_64.tar.gz` (glibc 2.35+; spätere Releases enthalten zusätzlich das statische `osatura-linux-x86_64-musl.tar.gz` für ältere Distributionen) und `SHA256SUMS.txt` an. Prüfe das Archiv gegen die Summen (`sha256sum -c SHA256SUMS.txt --ignore-missing`, unter Windows `Get-FileHash`), entpacke es und starte `osatura` (`osatura.exe`) aus dem entpackten Ordner; überspringe den Build in Abschnitt 2 und lies „Windows: Doppelklick auf `osatura.exe`“ und „Das Studio als Fenster“ am Ende dieser Seite. Die Programme sind nicht codesigniert, das System warnt dich also beim ersten Mal. Alles Folgende wurde mit einem lokalen Build desselben Quellcodes ausgeführt.

## 2. Bauen

Im Stammverzeichnis des Repositorys:

```
cargo build --release -p osatura-cli
```

Die Binärdatei ist `target/release/osatura`. Lege sie in deinen `PATH` oder richte einen Alias ein:

```
export PATH="$PWD/target/release:$PATH"      # or: alias osatura=$PWD/target/release/osatura
osatura --version
```

`--version` gibt die Version, die Lizenz (AGPL-3.0-or-later für die CLI und das Studio, MPL-2.0 für die
Engine-Bibliotheken) und das Quellcode-Angebot aus. Ein Aufruf von `osatura` ohne Argumente gibt die
vollständige Verwendung aus; `osatura <command> --help` gibt die Verwendung eines einzelnen Befehls aus.

Du kannst es auch ohne Installation ausführen: `cargo run --release -p osatura-cli -- demo demo.json`.

Wenn du das Kommandozeilenwerkzeug mit `--no-default-features` baust, entfällt der Befehl `script` (die
Rhai-Scripting-Engine); dieses Tutorial verwendet `script`, lass die Standard-Features also eingeschaltet.

## 3. Erste Schritte mit der eingebauten Demo

`osatura demo` schreibt ein vollständiges Beispielprojekt: einen per Skinning verformten Arm, ein IK-Bein, eine
Lip-Sync-Switch-Ebene, einen Parallax-Hintergrund und eine Kamerabewegung.

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
```

`check` kompiliert das Projekt genau so, wie es der Renderer tun würde; ein Projekt, das `check` besteht,
lässt sich also rendern. Zeichne nun ein Standbild mit eingeblendeten Knochen und exportiere eine Animation:

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

Was du gerade gemacht hast:

* `render <project> <frame> <out.png|out.svg>` zeichnet ein einzelnes Frame. `--bones` blendet das Skelett ein,
  `--scale N` rendert in der N-fachen Größe der Zeichenfläche (0 < N <= 16).
* `export` wählt das Format anhand der Dateiendung: `.gif`, `.html` (eine eigenständige Player-Seite),
  `.json` (Lottie), `.lottie` (dotLottie), `.apng`, `.webm`, `.mp4`, `.glb`, `.png` (Sprite-Sheet).
* Die `warning:`-Zeilen sind der ehrliche Teil: Lottie kann nicht alles darstellen, was OSATURA kann. Jede
  Einschränkung wird auf stderr ausgegeben, und `--report r.json` schreibt dieselbe Liste als JSON.
* Öffne `demo.html` in einem Browser, um die Animation abzuspielen; dabei sind weder ein Server noch ein Netzwerk beteiligt.

Um interaktiv zu erkunden:

```
osatura studio demo.json --port 8080
```

und öffne <http://127.0.0.1:8080/>. Siehe [STUDIO.md](STUDIO.md).

## 4. Ein 15-Minuten-Tutorial: eine winkende Figur

Du baust „Buddy“ aus einem leeren Projekt. Die Projektdatei ist JSON, aber du musst nie mehr als einen
Header schreiben: Der Befehl `script` führt ein kleines [Rhai](https://rhai.rs)-Skript in einer Sandbox über
denselben validierten Editor aus, den auch das Studio verwendet. Wird eine Anfrage in einem Skript abgelehnt, wird
*nichts* angewendet (alles oder nichts), sodass ein fehlerhaftes Skript nie eine halb bearbeitete Datei hinterlässt.

Arbeite in einem leeren Verzeichnis. Jeder Schritt schreibt eine neue nummerierte Datei, sodass du immer zurückgehen kannst.

### Schritt 1: ein leeres Projekt (1 Minute)

```
cat > buddy.json <<'EOF'
{"format": 1, "name": "Buddy", "width": 320, "height": 240, "fps": 24, "frames": 24}
EOF
osatura check buddy.json
```
```
ok: 'Buddy' 320×240 @ 24 fps, 24 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Diese sechs Schlüssel sind das Minimum. 24 Frames bei 24 fps sind eine Sekunde, und das wird unser Loop.

### Schritt 2: Formen (2 Minuten)

Ebenen werden von hinten nach vorn gezeichnet, in der Reihenfolge, in der du sie hinzufügst. Erstelle `shapes.rhai`:

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

Koordinaten sind Einheiten der Zeichenfläche mit dem Ursprung oben links und der y-Achse nach unten. Der Arm
wird waagerecht gezeichnet, mit der Schulter bei (160, 140), weil wir in dieser Pose seine Knochen aufbauen.

### Schritt 3: Knochen, Parenting und Grafik zuweisen (3 Minuten)

Ein Knochen hat einen Namen, einen Parent (leer = Root), eine Länge, eine Kopfposition (`x`, `y`) im Raum seines Parents
(bei einem Root: in der Zeichenfläche) und einen Ruhewinkel in Grad relativ zu seinem Parent. Der Winkel 0 zeigt entlang +x,
und positive Winkel drehen auf dem Bildschirm im Uhrzeigersinn; 90 zeigt also nach unten und -90 nach oben.
Erstelle `bones.rhai`:

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

Lies die Knochen so: `body` ist ein Root-Knochen bei (160, 190), der nach oben zeigt (-90) und 70 lang ist.
`neck` ist sein Kind; sein Kopf sitzt 70 Einheiten entlang `body` (oben). `arm_up` ist ein weiteres Kind von
`body`, 50 Einheiten oben an der Wirbelsäule (die Schulter), um +90 gedreht, sodass er zur Seite zeigt. `arm_low` ist das Kind
von `arm_up`: Parenting bedeutet, dass das Drehen des Oberarms den Unterarm mitnimmt. `hand_ctrl` ist ein
eigener Root-Knochen, den wir als IK-Ziel verwenden; er hat keine Grafik.

`set_layer(layer, "bone", bone)` lässt eine Ebene *starr* auf einem Knochen mitfahren: Sie folgt der Bewegung des Knochens
relativ zu seiner Ruhepose. Das ist der einfachste Weg, einen Cartoon aus einzelnen Teilen zu riggen. (Für
biegsame Formen bindest du Vektor-Anker per Skinning an mehrere Knochen; siehe [RIGGING_GUIDE.md](RIGGING_GUIDE.md).)

### Schritt 4: inverse Kinematik (1 Minute)

Statt die Winkel von Schulter und Ellbogen mit Keys zu versehen, lass den Solver nach `hand_ctrl` greifen. Erstelle `ik.rhai`:

```
add_ik("arm_ik", ["arm_up", "arm_low"], "hand_ctrl", 1);
```
```
$ osatura script 02-bones.json 03-ik.json ik.rhai
1 request(s) applied; wrote 03-ik.json
```

Das letzte Argument ist die Beugerichtung: +1 oder -1 legt fest, in welche Richtung der Ellbogen knickt. Eine Zwei-Knochen-Kette nutzt
einen analytischen Solver; Ketten mit drei oder mehr Knochen nutzen FABRIK. Von nun an bewegt `hand_ctrl`
den ganzen Arm mit.

### Schritt 5: Keys und Easing (3 Minuten)

`set_key(target, property, frame, value, ease)` erzeugt Keys. Bei einem Knochen sind `bone_angle`, `bone_scale`,
`bone_offset_x` und `bone_offset_y` *Abweichungen von der Ruhepose*, ein Key-Wert von 0 bedeutet also
„Ruhe“. Der Easing-Name gilt für das Segment, das an diesem Key *beginnt*. Erstelle `keys.rhai`:

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

Die Easing-Namen, die keine Parameter brauchen, sind `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`,
`smooth` und `bounce`. `bezier:x1,y1,x2,y2` wird von `keyedit` akzeptiert, und `spring` und `back` nehmen
Parameter entgegen (siehe [PROJECT_FORMAT.md](PROJECT_FORMAT.md)). Hand und Kopf verwenden absichtlich unterschiedliche Timings: Überlappende Bewegung
wirkt lebendig.

### Schritt 6: einen Loop daraus machen (1 Minute)

Beide Kanäle beginnen und enden mit demselben Wert (Frame 24 wiederholt Frame 0), was ein Loop braucht.
`loop-solve` prüft die Naht und stellt die Kanäle auf Loop:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
$ osatura lint buddy-final.json
0 finding(s), 0 error(s)
```

„seam 0.000 -> 0.000“ bedeutet, dass die letzte Pose bereits der ersten entspricht, es musste also nichts korrigiert werden. Ein
Projekt mit N Frames spielt die Frames 0..N-1 wiederholt ab, sodass Frame N dieselbe Pose wie Frame 0 ist und nie doppelt
gezeigt wird. Wenn du einen Loop so anlegst, dass das *zuletzt gezeigte* Frame dem ersten entspricht, übergib `--inclusive`
(die ältere Konvention). Wenn deine Animation eine Nettobewegung hat, entfernt `make-loop` sie. Siehe
[ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

`lint` führt statische Rig- und Animationsprüfungen aus (unerreichbare IK-Ziele, tote Kanäle, fehlende Knochen ...);
es beendet sich mit 1, wenn es Fehler findet, und lässt sich daher gefahrlos in ein Skript oder einen CI-Job einbauen.

### Schritt 7: ansehen und rendern (2 Minuten)

```
$ osatura stats buddy-final.json
```
gibt eine JSON-Zusammenfassung aus (hier: 5 bones, 1 ik chain, 6 layers, 2 channels, 6 keys). Rendere Standbilder und eine
ganze Frame-Sequenz:

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

![Vier Frames (0, 6, 12, 18) der Buddy-Animation, einmal normal und einmal mit eingeblendeten Knochen. Der rechte Arm hebt sich in einer Winkbewegung mit nach unten zeigendem Ellbogen, während der Kopf leicht nickt.](../../img/tutorial-buddy.png)

Ohne `--png` schreibt `frames` SVG-Dateien. Renderings sind deterministisch: Dasselbe Projekt und dasselbe Frame ergeben
auf jedem Rechner mit demselben Build dieselben Bytes, und genau das macht die Golden-Frame-Prüfungen
aus [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) möglich.

### Schritt 8: Export (2 Minuten)

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

Dateigrößen dieses winzigen Projekts: GIF 145 KB, Lottie-JSON 21 KB, HTML-Player 50 KB, APNG 75 KB, MP4 6 KB.
`buddy.json` ist Lottie; die Warnliste ist leer, weil Buddy nur Funktionen nutzt, die Lottie
ausdrücken kann. Die HTML-Datei ist ein eigenständiger Player (öffne sie in einem beliebigen Browser).

Ein paar Regeln, die man sich merken sollte:

* `export` und `render` **weigern sich, das Eingabeprojekt zu überschreiben** (`refusing to overwrite the input
  project buddy-final.json with output of the same name`) und weigern sich, durch einen symbolischen Link zu schreiben,
  es sei denn, du übergibst `--follow-symlinks`. Befehle, die ein Projekt in ein anderes umwandeln
  (`script`, `loop-solve`, `walk`, `autorig` ...), haben diese Prüfung *nicht*: Sie schreiben atomar, sodass
  `osatura script p.json p.json x.rhai` an Ort und Stelle bearbeitet. Verwende unterschiedliche Namen, bis du einem Skript vertraust.
* GIF hat 256 Farben und 1-Bit-Transparenz; für weiche Alpha-Übergänge nimm `.apng` oder `.webm`.
* Führe `osatura export-fit` aus (siehe [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)), wenn du ein Byte-Budget einhalten musst.

### Schritt 9 (optional): im Studio bearbeiten

```
osatura studio buddy-final.json --port 8080
```

Ziehe die Ebenen, wähle einen Key im Dope Sheet und drücke `Space` zum Abspielen. In deine Datei wird erst etwas geschrieben,
wenn du auf **Speichern** (`Ctrl+S`) klickst.

## 5. Wie die Teile zusammenpassen

| Konzept | In der Datei | Erzeugt mit |
|---|---|---|
| Ebene (Vektor, Bild, Gruppe, Switch ...) | `layers` | `add_rect`, `add_circle`, `add_polygon`, `add_line`, `import-svg`, `autotrace` ... |
| Knochen | `bones` (Parents vor Kindern) | `add_bone`, `set_parent`, `reparent` |
| IK-Kette / Constraint / Feder | `ik`, `constraints`, `springs` | `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`; Federn per JSON |
| Kanal (eine animierte Eigenschaft) | `channels` | `set_key`, `keys`, `keyedit`, Ausdrücke, Noise, Bakes |
| Clip, Take, Zustandsautomat, Binding | `clips`, `machines`, `bindings` | `take-record`, `clip`, `machine`, `bind` |

Alles ist reines JSON, weshalb Diffs (`osatura diff a.json b.json`), Versionskontrolle
(`osatura vcs`) und Skripte funktionieren. Unbekannte Felder werden als Warnungen gemeldet und ignoriert:

```
$ osatura check unk.json
warning: unknown field bogus (ignored)
ok: 'X' 100×100 @ 24 fps, 10 frames, 0 bones, 0 ik, 0 layers, 0 channels
```

Ein Projekt darf nur auf Bild- und Tondateien **innerhalb seines eigenen Verzeichnisses** verweisen. Absolute Pfade und `..`
werden abgelehnt, es sei denn, du übergibst `--allow-external-assets` (siehe [FAQ.md](FAQ.md)).

## 6. Wie es weitergeht

* Rigge etwas Biegsames (Skins, Gewichte, Lattice, Cage, Stoff, Physik): [RIGGING_GUIDE.md](RIGGING_GUIDE.md).
* Walk-Cycles, Clip-Stapel, Lip-Sync, Mocap, Zustandsautomaten, QA und Größenbudgets:
  [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).
* Der Browser-Editor, seine Tastenkürzel und die HTTP-API: [STUDIO.md](STUDIO.md).
* Fragen und Fehlermeldungen: [FAQ.md](FAQ.md). Bekannte Einschränkungen: [LIMITATIONS.md](LIMITATIONS.md).
* Jeder Befehl und jedes Flag: [CLI_REFERENCE.md](CLI_REFERENCE.md); das Dateiformat: [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Windows: Doppelklick auf `osatura.exe`

`osatura.exe` ist ein Kommandozeilenprogramm. Seit 0.1.0-alpha.2 legt es beim Öffnen aus dem Explorer (Doppelklick) `Documents\OSATURA\first-rig.json` aus der Demo „laufende Person“ (`osatura demo-person`; eine vorhandene Datei wird nie überschrieben) an, startet das Studio auf `http://127.0.0.1:8080` und öffnet es in deinem Browser; das Konsolenfenster bleibt offen, solange das Studio läuft, schließe es zum Beenden. Aus einem Terminal ohne Argumente geöffnet, gibt es wie bisher die Usage aus. In 0.1.0-alpha.1 gab ein Doppelklick nur die Usage aus, und das Fenster schloss sich sofort wieder.

Die Programme sind **nicht codesigniert**: Windows SmartScreen zeigt „Unbekannter Herausgeber“ (wähle *Weitere Informationen* und dann *Trotzdem ausführen*), und macOS Gatekeeper verlangt, dass du das Programm unter *Systemeinstellungen > Datenschutz & Sicherheit* bestätigst. Prüfe den Download zuerst gegen `SHA256SUMS.txt`. Ist Port 8080 belegt, führe `osatura studio first-rig.json --port 8090` in einem Terminal aus.

## Das Studio als Fenster: `osatura-studio` (Windows und macOS)

Die Archive für Windows und macOS enthalten ein zweites Programm, `osatura-studio` (`osatura-studio.exe`), ein kleines natives Fenster um das Browser-Studio: Es startet die `osatura`-Engine aus demselben Ordner als `studio` auf einem freien lokalen Port, zeigt die Seite in einem Fenster mit WebView2 (Windows 11 hat es eingebaut; ältere Windows-Versionen brauchen die Evergreen-Laufzeitumgebung) oder WKWebView (macOS) und beendet den Server, wenn das Fenster geschlossen wird. Es erscheint kein Konsolenfenster. Lege es im selben Ordner wie `osatura(.exe)` ab.

* **Exporte** (GIF, HTML-Player, Lottie) öffnen einen nativen „Speichern unter“-Dialog, der im Ordner des Projekts beginnt, und eine Meldung nennt die Datei, sobald sie gespeichert ist. (alpha.2 speicherte sie stillschweigend in deinen Downloads-Ordner; alpha.3 fragt nach.)
* Doppelklicke darauf, um an `Documents/OSATURA/first-rig.json` zu arbeiten (wird aus der Demo „laufende Person“, `osatura demo-person`, angelegt, falls sie fehlt), oder starte es als `osatura-studio my-project.json` (zum Beispiel über „Öffnen mit“).
* `osatura-studio --selftest` startet den Server, ruft die Seite ab und beendet ihn ohne Fenster; der Exit-Code ist 0, wenn das geklappt hat (CI führt es unter Windows und macOS aus).
* **Einschränkungen.** Das Fenster wurde auf einem echten Windows-11-Desktop benutzt (alpha.2: Studio und Exporte funktionierten); der Speichern-unter-Dialog von alpha.3 wurde in CI kompiliert und selbstgetestet, aber noch niemand hat ihn ausprobiert, und auf macOS hat noch niemand das Fenster benutzt. Weitere browser-typische Funktionen, die das System brauchen (Drag-and-drop aus dem Explorer oder Finder, das Öffnen von Dateien über die Dateiauswahldialoge des Studios), sind die der System-Webview und ungetestet. Es gibt kein Linux-Fenster (es bräuchte die Systempakete GTK / WebKitGTK): Nutze `osatura studio` und einen Browser. Das Programm ist nicht codesigniert (siehe oben).
