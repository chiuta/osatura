# OSATURA-Studio

Das Studio ist ein Browser-Editor, den die Binärdatei `osatura` selbst bereitstellt: `osatura studio project.json`.
Es bietet eine Zeichenfläche mit Auswahl- und Transformations-Gizmos, eine Ebenenliste, eine Knochenliste, ein Dope Sheet und einen Kurvengraphen,
Onion Skin und Bewegungsspuren, eine Befehlspalette mit umbelegbaren Tastenkürzeln, Pose-Vorschläge, Panels für Live-Eingabe und
Gesicht, Export-Schaltflächen für GIF/HTML/Lottie und eine gemeinsame Mehrbenutzer-Bearbeitungssitzung. Es hat keine Konten, kein TLS und
keine Datenbank: Es ist ein kleiner HTTP-Server auf Basis der Standardbibliothek vor demselben Editor, den auch die Kommandozeile verwendet.

Alles hier wurde gegen die echte Binärdatei (`osatura 0.1.0`, Debug-Build, Linux) geprüft und, was die Benutzeroberfläche betrifft,
in Headless-Chromium, gesteuert von Playwright. Die Screenshots wurden auf diese Weise erstellt (1400x900, das eingebaute Demo-
Projekt). Getestet wurde nur **Chromium**; siehe Abschnitt 11. Unter Windows und macOS enthalten die Release-Archive außerdem `osatura-studio`, dasselbe Studio in einem nativen Fenster (in CI kompiliert und selbstgetestet, noch von keiner Person benutzt; siehe [GETTING_STARTED.md](GETTING_STARTED.md) und [LIMITATIONS.md](LIMITATIONS.md) L-75), und unter Windows startet ein Doppelklick auf `osatura.exe` das Studio und öffnet den Browser (L-76).

Lizenz dieses Dokuments: CC-BY-4.0. Für die Kommandozeile siehe [GETTING_STARTED.md](GETTING_STARTED.md) und
[CLI_REFERENCE.md](CLI_REFERENCE.md); zur Serversicherheit siehe auch [SECURITY.md](SECURITY.md).

![Das Studio mit dem Demoprojekt: oben die Werkzeugleiste, die Zeichenfläche mit ausgewählter Arm-Ebene und eingeblendeten Knochen, rechts das Ebenen-Panel und darunter die Timeline mit Key-Rauten.](../../img/studio-overview.png)

## 1. Das Studio starten

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

Öffne die Adresse in einem Browser. Das Projekt wird beim Start validiert; eine Datei, die `check` nicht besteht, wird mit derselben
Meldung abgelehnt. Vor unbekannten Feldern wird gewarnt (`warning: unknown field bogus (ignored)`).

| Flag | Standard | Bedeutung |
|---|---|---|
| `--port N` | 8080 | TCP-Port. Ein belegter Port ergibt `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)`; ein ungültiger `cannot listen on 127.0.0.1:99999: invalid port value`. |
| `--bind ADDR` | `127.0.0.1` | Schnittstelle, auf der gelauscht wird. Alles, was keine Loopback-Adresse ist, **erfordert `--collab-token`**. |
| `--ui-config F` | `$OSATURA_UI_CONFIG`, sonst `$XDG_CONFIG_HOME/osatura/ui.json`, sonst unter Windows `%APPDATA%\OSATURA\ui.json` (vor alpha.3 gab es keinen: Die Einstellungen wurden nur für die Sitzung gehalten), sonst `~/.config/osatura/ui.json` | Wo Layout, Tastenkürzel, zuletzt verwendete Befehle und Overlay-Einstellungen gespeichert werden. Es ist eine Datei pro Benutzer und nie Teil des Projekts. |

Die Fenster-App und der Doppelklick-Start legen das erste Projekt in `OSATURA_DOCUMENTS` an, wenn diese Variable gesetzt ist, sonst in `Documents\OSATURA` (vom portablen Paket verwendet).
| `--allow-host NAMES` | keine | Zusätzliche `Host`-Namen, die akzeptiert werden (kommagetrennt), für den Zugriff unter einem DNS-Namen. |
| `--public-origin URL` | keine | Der reine Origin (`https://studio.example.org`, kein Pfad), den ein TLS-Reverse-Proxy präsentiert; dessen `Host` und `Origin` werden dann akzeptiert. |
| `--source-url URL` | keine; in den offiziellen Release-Binärdateien der Commit, aus dem sie gebaut wurden (`https://github.com/chiuta/osatura/tree/<commit>`) | Wo der zugehörige Quellcode *deines* Builds heruntergeladen werden kann (AGPL Abschnitt 13); wird auf `/source` und von `--version` angezeigt. Ein ausdrücklich angegebener Wert hat immer Vorrang. |
| `--collab-token T` | keine | Gemeinsames Geheimnis, **mindestens 16 Zeichen** (`openssl rand -hex 16`). Für Nicht-Loopback-Bindungen erforderlich. |
| `--collab-log LOG` | keine | Die gemeinsame Bearbeitungssitzung in eine Datei persistieren und nach einem Neustart fortsetzen. |
| `--collab-nosync` | aus | Mit `--collab-log`: nach jeder Bearbeitung kein `fsync` ausführen (schneller; ein Stromausfall kann die letzten Bearbeitungen kosten). Ohne `--collab-log` wird es beim Start abgelehnt (`--collab-nosync needs --collab-log`). |

**Speichern.** Bearbeitungen liegen im Arbeitsspeicher. Die Projektdatei auf der Festplatte ändert sich nur, wenn du auf **Speichern** (`Ctrl+S`) klickst; das schreibt atomar
(temporäre Datei, dann Umbenennen, nie durch einen symbolischen Link). Das gilt auch während einer gemeinsamen Sitzung (Abschnitt 8).

## 2. Sicherheitsmodell (lies das, bevor du an ein Netzwerk bindest)

Das Studio ist für eine Person auf ihrem eigenen Rechner gebaut. Was es schützt:

* **Host-Allowlist.** Nur Anfragen, deren `Host`-Header einer der eigenen Namen des Servers ist, werden beantwortet; alles andere erhält **421**.
  Mit `--allow-host` und `--public-origin` werden weitere Namen akzeptiert.
* **Schutz vor Cross-Site-Zugriffen.** Eine Anfrage mit fremdem `Origin` oder mit `Sec-Fetch-Site: cross-site` erhält **403**. Ein POST, dessen Body nicht
  `application/json` ist, erhält **415** (POSTs ohne Body wie `/api/undo` brauchen keinen Typ).
* **Sicherheits-Header** in den Antworten: eine restriktive `Content-Security-Policy` (`default-src 'self'` und kein Framing), `X-Frame-Options: DENY`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`.
* **Token für alles außerhalb des Rechners.** Mit `--collab-token` braucht jede Anfrage das Token als Header `X-Collab-Token` oder im
  Cookie `osatura_token`, das von `POST /api/login` gesetzt wird (HttpOnly, SameSite=Strict). Das Token wird **in der URL nie akzeptiert**. Falsche Tokens werden pro Client-Adresse
  gebremst (ab dem fünften Fehlversuch in Folge erhält diese Adresse **429** für 2, 4, ... bis zu 30 Sekunden), aber das Back-off kann einen
  Angreifer nicht aufhalten, der es weiter versucht: Verwende ein langes Zufalls-Token.
* **Geheimnisse der Teilnehmer.** Das Beitreten zu einer gemeinsamen Sitzung liefert ein zufälliges 128-Bit-Geheimnis; jeder Aufruf, der als dieser Teilnehmer erfolgt, muss es als
  `X-Collab-Secret` senden. Die Nummer eines Teilnehmers zu kennen genügt nicht.

Ich habe jeden dieser Punkte mit `curl` gegen einen laufenden Server geprüft:

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

Startverweigerungen (Exit-Code 1, nichts wird gestartet):

```
refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)
--collab-token must be at least 16 characters (e.g. `openssl rand -hex 16`)
--source-url must be an http:// or https:// URL without spaces or quotes
--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org
```

**Ehrliche Grenzen.** Der Verkehr läuft über einfaches HTTP: Das Token und alles andere werden im Klartext übertragen, stelle das Studio also hinter einen TLS-Reverse-Proxy
(`--public-origin`), wenn es für jemand anderen erreichbar ist. Das `Set-Cookie` für das Token erhält das **`Secure`-Flag**, wenn die Anfrage unter einem `https://`-`--public-origin`-Namen eintrifft (oder mit einem https-`Origin` / `X-Forwarded-Proto: https`); einfaches HTTP auf Loopback bekommt es nie,
und hinter einem Proxy erscheint jeder Client mit der Adresse des Proxys, sodass das Back-off alle gleichzeitig bremst. `/source` ist ohne Token offen
(die AGPL verlangt das). Es gibt keine Benutzerkonten: Wer das Token hat, kann das Projekt ändern und den Speicherpfad beschreiben.

### Das AGPL-Quellcode-Angebot

Das Studio ist AGPL-3.0-or-later-Software: Wenn du es änderst und anderen die Nutzung über ein Netzwerk ermöglichst, musst du ihnen den Quellcode
deiner Version anbieten. `/source` ist dieses Angebot; setze `--source-url` auf den Ort, an dem der Quellcode deines Builds liegt. Ohne es (ein Build, den du selbst mit einem einfachen `cargo build` erstellt hast, hat keines) sagt die Seite das und nennt den Maintainer. Die von `.github/workflows/release.yml` gebauten Release-Binärdateien tragen die Adresse ihres eigenen Commits (`OSATURA_SOURCE_URL` beim Kompilieren; ein Fork, der mit seinem eigenen Workflow baut, verweist auf sich selbst).

## 3. Ein Rundgang durch die Oberfläche

### Werkzeugleiste

Rückgängig, Wiederholen, **Speichern**; die Kontrollkästchen `bones` (das Skelett zeichnen), `auto-key`, `grid` (Abstand in Einheiten der Zeichenfläche, 0 = aus), `snap` (an anderen Ebenen und der Zeichenfläche),
`whole groups` (ein Klick auf einen Teil einer Gruppe wählt die Gruppe der obersten Ebene), `points` (Anker, Handles, Cage-/Lattice-/Pin-Punkte und Mesh-Eckpunkte der
ausgewählten Ebenen anzeigen); eine Auswahl `box`, was ein Aufziehen eines Rahmens auswählt (`touches`, `contains`, `bounds touch`; sie beginnt bei `touches`); **Lock**; und die Schaltflächen **Overlays**, **Suggest**, **Layout**,
**Commands** (Palette) und **Keys** (Tastenkürzel-Editor). Eine zweite Zeile enthält deinen Namen und **Collab** sowie die Export-Schaltflächen **GIF**, **HTML player** und **Lottie**,
die herunterladen, was `osatura export` schreiben würde (der HTML-Player enthält den gemischten Ton, wenn es Ton gibt und `ffmpeg` das mp3 erzeugen kann). Die Schaltfläche **Simple view** blendet die Bearbeitungs-Checkboxen, Overlays, Suggest und die Schaltfläche der Befehlspalette aus, sodass nur Undo, Speichern, Panels und Exporte bleiben; die Wahl wird in diesem Browser (oder Fenster) gemerkt, und mit **Full view** kommt die volle Werkzeugleiste zurück.

### Panels und Layouts

Das Fenster ist ein Dock aus Panels, jedes mit einer stabilen ID: `stage` (die Zeichenfläche), `layers`, `selection`, `bones`, `live` (Live-Eingabe), `face` (Webcam-Gesicht), `json`
(das ganze Projekt als bearbeitbares JSON mit einer Schaltfläche *Apply JSON*) und `timeline`. Es gibt vier Layout-Presets:

```
animate  stage over timeline on the left, all panels as tabs on the right
rig      stage with a folded timeline; layers+bones and selection+json on the right (live and face hidden)
compact  stage over a tabbed timeline and panels
live     stage and timeline on the left; live, face, bones and selection on the right (layers and json hidden)
```

Wechsle Presets über das Menü **Layout** oder die Palette; ziehe Tabs, um ein Panel neu anzudocken oder schweben zu lassen; „reset layout“ stellt den Standard wieder her.
Das Layout wird pro Benutzer in der UI-Konfiguration gespeichert. `osatura ui layout presets|show NAME|validate FILE.json` gibt dieselben Daten aus und validiert sie
(ein Layout muss jedes Panel erreichbar halten; ein unvollständiges wird vom Server mit 400 abgelehnt).

### Ebenen-Panel

![Das Ebenen-Panel mit einer gesperrten Ebene (sun), einer ausgeblendeten Ebene (hills) sowie gemeinsam ausgewähltem Arm und Bein.](../../img/studio-layers.png)

Jede Zeile hat zwei Kontrollkästchen: **visible** und **lock** (*mit dem Projekt* gespeichert; gesperrte Ebenen werden von Klicks und Rahmen übersprungen und lehnen Bearbeitungen ab).
Der Screenshot zeigt `sun` gesperrt, `hills` ausgeblendet und zwei per Shift-Klick ausgewählte Ebenen.

### Selection-Panel

Nennt die Regeln auf der Seite selbst: *Ein Klick wählt das oberste Objekt (gesperrte und ausgeblendete werden übersprungen); Shift-Klick schaltet um; Ziehen auf leerem Raum
zeichnet einen Rahmen (mit gedrückter Alt-Taste ein freies Lasso, Shift fügt hinzu). Ziehe den Rahmen zum Verschieben, ein Quadrat zum Skalieren (Shift behält die Proportionen, Alt skaliert vom
Zentrum aus), den runden Knauf zum Drehen (Shift rastet auf 15 Grad ein). Halte beim Ziehen Ctrl gedrückt, um das Einrasten auszuschalten. Die Pfeiltasten verschieben um 1 Einheit (Shift: 10),
Entf löscht Ebenen, Esc bricht einen Zug ab oder hebt die Auswahl auf.*

### Timeline: Dope Sheet und Graph

![Das Dope Sheet mit einer Rahmenauswahl von drei Keys bei Frame 12 (die Keys sind ausgefüllt gezeichnet, die anderen hohl).](../../img/studio-dopesheet.png)

![Die Kurvengraph-Ansicht: drei bone_angle-Kurven (arm_up, arm_low, hand) in verschiedenen Farben mit drei ausgewählten Keys bei Frame 12, die ihre Tangenten-Handles zeigen; die Statuszeile lautet „3 keys selected | frames 12“.](../../img/studio-graph.png)

`Graph` (oder `Alt+V`) schaltet zwischen Dope Sheet und Kurvengraph um. Die Steuerelemente der Timeline: `Play`, `|<` (erstes Frame des Bereichs), `loop`, `range` mit
`Set in` / `Set out` (nur zwischen In und Out abspielen), `snap` (verschobene Keys auf ganze Frames runden), `on collision` (`replace`, `merge`, `refuse`, wenn verschobene Keys auf
vorhandene treffen), `Copy` / `Paste` / `Duplicate` / `Delete`, `interp` + `Apply` (das Easing der ausgewählten Keys setzen), `Auto-smooth` (überschwingungsfreie Tangenten in die
ausgewählten Keys backen), `ripple` N mit `Insert` / `Remove` (und `grow project`, um die Projektlänge damit zu ändern), Zoom `-` `+` `Fit` und `shy` (als shy markierte Kanäle anzeigen).

Maus: Klicke auf einen Key, um ihn auszuwählen, Shift-Klick fügt hinzu, ziehe einen Rahmen zum Auswählen, ziehe Keys, um sie zu verschieben; **Alt-Ziehen oder die mittlere Taste verschiebt die Ansicht**, **Ctrl+Mausrad zoomt**,
**Shift+Mausrad scrollt** vertikal; im Graphen **beschränkt Shift das Ziehen eines Keys auf eine Achse**. Die Snap-Toleranz für das Einrasten an der Zeichenfläche beträgt standardmäßig 6 Einheiten der Zeichenfläche.
Jede Key-Bearbeitung läuft über dieselben validierten Operationen wie `osatura keyedit` (siehe [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)), ist also ein Rückgängig-Schritt, und gesperrte Kanäle lehnen sie ab.

### Overlays: Onion Skin, Spuren, Proxys

![Onion Skin und Bewegungsspuren für den Arm bei Frame 14 eingeschaltet, bei geöffnetem Overlays-Panel.](../../img/studio-overlays.png)

`Overlays` (`Alt+7`) enthält Onion Skin (`Alt+N`: Frames davor/danach in roten/grünen Tönungen, fester Schritt oder Key-Frames, Umrissmodus, Abfall, nur ausgewählte Ebenen),
Bewegungsspuren (`Alt+M`: Pfade von Knochen, Ebenen-Pivot oder Kontrollpunkten mit Teilstrichen pro Frame) und **Scrub-Proxys** (`Alt+Q`: Bilder reduzierter Qualität, während du den
Abspielkopf ziehst, mit einem Bild-Cache und einer optionalen Kopie in einer Detailstufe). Das Panel meldet zum Beispiel `picture cache: 2 frames, 17 KiB, 0 hits / 7 misses, emptied 4x by edits`.
Die Einstellungen werden in der UI-Konfiguration gespeichert und überstehen einen Neustart. Overlays sind nie Teil von Exporten.

### Befehlspalette und Tastenkürzel

![Die mit Ctrl+K geöffnete und nach dem Wort „toggle“ gefilterte Befehlspalette, die passende Befehle mit ihren Tastenkürzel-Hinweisen auflistet.](../../img/studio-palette.png)

`Ctrl+K` (oder `Ctrl+Shift+P`) öffnet die Palette: Tippe zum Filtern (unscharf, passende Buchstaben hervorgehoben), die Pfeiltasten bewegen, Enter führt aus, Esc schließt. Solange nichts eingegeben ist, werden
zuletzt verwendete Befehle zuerst aufgelistet. Es gibt **84 Befehle**; `osatura ui commands` listet sie auf, und `osatura ui search onion` reiht sie genau so wie die Palette.

![Der mit Ctrl+Komma geöffnete Tastenkürzel-Editor, der Befehle nach Kategorie mit ihren Tastenkombinationen auflistet.](../../img/studio-shortcuts.png)

`Ctrl+,` (oder **Keys**) öffnet den Tastenkürzel-Editor: Klicke auf eine Kombination und drücke die neue Tastenkombination. Ein Konflikt nennt den anderen Befehl und ändert nichts, bis du
„Take it over“ bestätigst. Vom Browser reservierte Kombinationen werden abgelehnt: `Ctrl+W`, `Ctrl+T`, `Ctrl+N`, `Ctrl+Q`, `Ctrl+R`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+Shift+W`, `Ctrl+Shift+N`, `F5`, `F11`, `F12`,
`Alt+F4`. Das Umbelegen wird in der UI-Konfiguration gespeichert und übersteht ein Neuladen. Der Server lehnt eine Zuordnung mit Konflikten, einem unbekannten Befehl, einem unvollständigen Layout oder einem Body, der kein Objekt ist, mit 400 ab.

### Suggest

![Das Suggest-Panel mit drei gerankten Kandidaten für Zwischenposen (linear, pca, manifold) zwischen den Frames 12 und 24, jeweils mit Score und einer Schaltfläche Apply.](../../img/studio-suggest.png)

`Suggest` (`Alt+8`) schlägt Posen vor, die aus den eigenen Key-Posen des Projekts gelernt wurden: eine Zwischenpose zweier Frames, die Vervollständigung einer teilweise festgelegten Pose oder „was üblicherweise
als Nächstes kommt“. Beim Demoprojekt reihte es zwischen den Frames 12 und 24 `linear` mit 1,00, `pca` mit 1,00 und `manifold` mit 0,99, aufgebaut aus 5 Posen auf 2 Achsen (99 % der Varianz).
`Apply` schreibt die Pose als Keys, ein Rückgängig-Schritt. Dieselbe Engine ist `osatura pose-suggest` und `/api/suggest`. Es sind klassische Methoden (lineare Mischung, PCA, eine Kernel-
Mannigfaltigkeit, ein optionales kleines Netz, das du selbst trainierst); bei einer Handvoll Posen stimmen die drei überein.

### Panels für Live-Eingabe und Webcam-Gesicht

* **Live input** (`Alt+4`): Maus über dem Bild, Tasten, Gamepad und Web MIDI werden über eine Tabelle auf Eigenschaften abgebildet; `Record` bakt eine Performance als einen Rückgängig-
  Schritt in Keys. Web MIDI braucht einen Browser, der es unterstützt (Chromium tut das).
* **Webcam face** (`Alt+5`): Landmarks steuern dieselbe Abbildungstabelle (Quelle `face`, IDs wie `mouth_open`, `eye_r_open`, `head_yaw`). Die Schaltflächen sind
  *Start camera*, *Synthetic face*, *Synthetic face (detector)*, *Calibrate (hold neutral)* und *Face example mapping*. Der Kamerapfad braucht eine Landmark-Provider-Funktion,
  die du bereitstellst (`window.osaturaLandmarkProvider`, siehe den Kommentar im Seitenquelltext); das Video verlässt die Seite nie, und nur Landmarks gehen an den lokalen Server.
  Die beiden synthetischen Modi funktionieren sofort. *Synthetic face (detector)* rendert den synthetischen Kopf auf dem Server und findet ihn mit dem eingebauten Detektor wieder,
  der **nur mit synthetischen Gesichtern trainiert wurde** (stilisierte Köpfe, keine Fotos); siehe den Abschnitt zum Gesicht in [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md).

### Geführte Demo

Die Schaltfläche **Demo** (Befehl `demo.panel`) öffnet eine geführte Demo in 13 Schritten, die die laufende Person aus `osatura demo-person` aufbaut: Kulisse, Körper, Kopf und Gesicht, Arme mit fünf Fingern pro Hand, das Skelett, die gehenden Beine, die schwingenden Arme, Rumpf und Kopf, ein Blinzeln und ein Lächeln, der bewegte Boden, eine Prüfung und der Export. Jeder Schritt lädt diese Stufe des Projekts mit einer einzigen Ersetzen-alles-Operation in den Editor, sodass du sie abspielen, scrubben, Knochen und Ebenen auswählen und die Timeline lesen kannst. Die Stufen sind Ausschnitte des fertigen Projekts; die im Panel gezeigten Befehle und ihre Ausgabe wurden mit dem echten Programm ausgeführt.

* **Nichts wird auf die Festplatte geschrieben.** Speichern (und Ctrl+S) ist deaktiviert, solange die Demo geladen ist. Der einzige Schreibweg des Studios, `POST /api/save`, wird von der Demo nie aufgerufen.
* **Dein Projekt kommt zurück.** *Restore my project* stellt das zuvor geöffnete Projekt wieder her (eine einzige Ersetzen-alles-Operation, ein Undo-Schritt). Wenn du den Tab schließt oder neu lädst, während die Demo geladen ist, bemerkt das nächste Laden der Seite, dass der Server noch die Demo hält, lässt Speichern ausgeschaltet und bietet *Restore my project* erneut an; das Original wird dafür im lokalen Speicher des Browsers aufbewahrt.
* Die Demo weigert sich, innerhalb einer Zusammenarbeitssitzung zu starten, weil sie das ganze Projekt ersetzt.
* Erzeuge die Stufen mit `node scripts/build-studio-tour.mjs --bin target/release/osatura` neu und baue dann die CLI neu (die Seite ist in das Programm eingebettet).

## 4. Tastenkürzel (Standard)

Dies ist die Ausgabe von `osatura ui shortcuts` für eine frische Konfiguration: **72 Belegungen, 84 Befehle, 0 Konflikte**. Tastenkürzel gehören zu einem **Kontext**:
`global` (überall), `canvas` (Fokus auf dem Bild) und `timeline` (Fokus auf der Timeline). Textfelder behalten ihre eigenen Tasten.

### Global

| Tastenkürzel | Befehl | Tastenkürzel | Befehl |
|---|---|---|---|
| `Space` | Wiedergabe / Pause (`playback.toggle`) | `Alt+B` | Knochen umschalten |
| `Home` / `End` | erstes / letztes Frame | `Alt+G` | ganze Gruppen umschalten |
| `,` / `.` | vorheriges / nächstes Frame | `Alt+K` | Auto-Key umschalten |
| `Shift+,` / `Shift+.` | 10 Frames zurück / vor | `Alt+M` | Bewegungsspuren umschalten |
| `[` / `]` | Bereich In / Out setzen | `Alt+N` | Onion Skin umschalten |
| `Alt+R` | Wiedergabebereich umschalten | `Alt+O` | Loop-Wiedergabe umschalten |
| `Alt+1` | Ebenen-Panel umschalten | `Alt+P` | Kontrollpunkte umschalten |
| `Alt+2` | Selection-Panel umschalten | `Alt+Q` | Scrub-Proxys umschalten |
| `Alt+3` | Knochen-Panel umschalten | `Alt+S` | Einrasten an der Zeichenfläche umschalten |
| `Alt+4` | Live-Input-Panel umschalten | `Alt+T` | Timeline-Panel umschalten |
| `Alt+5` | Webcam-Gesicht-Panel umschalten | `Alt+V` | Dope Sheet / Graph umschalten |
| `Alt+6` | JSON-Panel umschalten | `Alt+Shift+S` | Einrasten in der Timeline umschalten |
| `Alt+7` | Overlays-Panel | `Alt+=` / `Alt+-` | Timeline vergrößern / verkleinern |
| `Alt+8` | Suggest-Panel | `Ctrl+S` | Speichern |
| `Ctrl+K` / `Ctrl+Shift+P` | Befehlspalette | `Ctrl+,` | Tastenkürzel |
| `Ctrl+Z` | Rückgängig | `Ctrl+Y` / `Ctrl+Shift+Z` | Wiederholen |

### Canvas

| Tastenkürzel | Befehl |
|---|---|
| `Arrow keys` | die Auswahl um 1 Einheit verschieben (`Shift`: 10 Einheiten) |
| `Delete` / `Backspace` | die Auswahl löschen |
| `Escape` | die Auswahl aufheben |
| `Ctrl+A` | alles auswählen |
| `Alt+L` | die ausgewählten Ebenen sperren / entsperren |
| `I` | einen Key am Abspielkopf hinzufügen (`anim.add_key`) |

### Timeline

| Tastenkürzel | Befehl |
|---|---|
| `ArrowLeft` / `ArrowRight` | vorheriges / nächstes Frame (`Shift`: 10) |
| `-` / `=` | verkleinern / vergrößern |
| `F` | auf Fenstergröße zoomen |
| `Delete` / `Backspace` | ausgewählte Keys löschen |
| `Escape` | Keys abwählen |
| `Ctrl+A` | alle Keys auswählen |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | Keys kopieren / einfügen / duplizieren |
| `Alt+ArrowLeft` / `Alt+ArrowRight` | ausgewählte Keys ein Frame früher / später verschieben |
| `Alt+ArrowUp` / `Alt+ArrowDown` | die ausgewählten Keys anheben / absenken (ihren Wert) |
| `I` | einen Key am Abspielkopf hinzufügen |

Modifikatoren auf der Zeichenfläche: **Shift** beschränkt (Achsensperre beim Verschieben, 15-Grad-Schritte beim Drehen, proportional beim Skalieren, eine Knochenspitze dreht sich dann nur,
und Shift-Klick schaltet die Auswahl um, während ein Rahmen ihr hinzufügt); **Alt** skaliert vom Zentrum aus (und Alt-Ziehen auf leerem Raum zeichnet ein Lasso); **Ctrl** (Cmd unter macOS)
schaltet das Einrasten aus.

## 5. Auswahl und Gizmos

Ein Klick wählt das oberste Objekt unter dem Zeiger: eine Ebene, einen Knochen (seine Kapsel, etwas breiter als die gezeichnete Linie) oder, bei eingeschaltetem `points`, einen Kontrollpunkt. Gesperrte und ausgeblendete
Ebenen werden übersprungen, ein Klick geht also hindurch auf das, was darunter liegt. Ein Rahmen wählt nach dem Modus in der Auswahl `box`: `touches` (alles, was der Rahmen überlappt), `contains` (nur, was vollständig darin liegt) oder `bounds touch` (die Begrenzungsrahmen überlappen); Alt-Ziehen zeichnet stattdessen ein Lasso.

Bei einer einzelnen Ebene folgt der Transformationsrahmen den eigenen Achsen der Ebene: 8 Handles skalieren (Kanten eine Achse, Ecken beide; das Ziehen über die gegenüberliegende Kante spiegelt mit negativer Skalierung),
der runde Knauf dreht um das Zentrum, und das Ziehen im Inneren verschiebt. Bei einem Knochen verschiebt der Kopfkreis den Knochen, und die Spitze dreht ihn (und streckt ihn, sofern Shift nicht gehalten wird).
Bei Kontrollpunkten (Ankern, Bézier-Handles, Deformer-Punkten, Mesh-Eckpunkten) verschiebt dasselbe Gizmo die ausgewählten Punkte. Das Einrasten umfasst ein Raster (standardmäßig aus), die Kanten und Zentren anderer Ebenen
sowie die Ränder der Zeichenfläche, innerhalb von 6 Einheiten der Zeichenfläche. **Ein Zug ist ein Rückgängig-Schritt** (eine Batch-Operation).

## 6. Auto-Key: was genau mit Keys versehen wird

Ob eine Bearbeitung auf der Zeichenfläche einen *statischen* Wert ändert oder *Keys* erzeugt, wird so entschieden. Ich habe es mit der HTTP-API am Demoprojekt geprüft (der `mode` jeder Bearbeitung wird gemeldet):

| Situation | Ergebnis |
|---|---|
| Ebenenbearbeitung, `auto-key` aus, die Eigenschaft hat keinen Kanal mit Keys | **statische** Änderung der Transformation der Ebene (es wird kein Kanal erzeugt) |
| Ebenenbearbeitung, `auto-key` ein | die bearbeiteten Eigenschaften erhalten am aktuellen Frame **Keys** (neue Kanäle `layer_x` und `layer_y` wurden erzeugt: 14 auf 16 Kanäle) |
| Eigenschaft einer Ebene oder eines Punkts, die **bereits einen Kanal mit Keys hat**, `auto-key` aus | erhält am aktuellen Frame trotzdem **Keys** (eine Bearbeitung bei Frame 9 fügte den Kanälen ab Frame 5 einen zweiten Key hinzu) |
| **Knochen**-Bearbeitung | erhält immer **Keys**, unabhängig von der Einstellung `auto-key` (Knochen-Offset-Kanäle wurden erzeugt: 16 auf 18) |

Eine animierte Eigenschaft kann also nie stillschweigend durch einen statischen Wert überschrieben werden, und eine Knochenpose geht nie „verloren“, weil man den Schalter vergessen hat. Schalte `auto-key` ein (`Alt+K`),
wenn du eine Eigenschaft animieren willst, die noch statisch ist. Die Taste `I` (`anim.add_key`, „Add key at playhead“) fügt am aktuellen Frame einen Key hinzu.

## 7. Rückgängig und Wiederholen

Rückgängig und Wiederholen (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) arbeiten mit ganzen Operationen: Ein Gizmo-Zug, ein Verschieben, eine Key-Bearbeitung und ein Apply sind je ein Schritt. Ohne gemeinsame Sitzung sind sie
linear. **In einer gemeinsamen Sitzung gilt Rückgängig pro Benutzer**: Es macht nur deine eigene letzte Bearbeitung rückgängig und lässt die Bearbeitungen anderer bestehen (Abschnitt 8). Der Rückgängig-Verlauf wird nicht mit dem
Projekt gespeichert und geht verloren, wenn der Server stoppt.

## 8. Gemeinsame Bearbeitung (Zusammenarbeit)

### Walkthrough: zwei Personen, ein Projekt

![Zwei Browser-Benutzer in einer Sitzung: Die Werkzeugleiste zeigt zwei Teilnehmer-Chips, Ana (du) bei Frame 14 und Bo bei Frame 0, und Bos Auswahlrahmen und Cursor sind auf der Zeichenfläche um die Bein-Ebene gezeichnet.](../../img/studio-collab.png)

1. Starte das Studio. Für Personen an anderen Rechnern musst du ein Token hinzufügen und an eine Netzwerkadresse binden:
   `osatura studio demo.json --bind 0.0.0.0 --port 8080 --collab-token $(openssl rand -hex 16) --collab-log session.log`,
   am besten hinter einem TLS-Proxy (Abschnitt 2). Auf einem einzelnen Rechner ist kein Token nötig.
2. Gib im ersten Browser neben **Collab** deinen Namen ein („Ana“) und klicke auf **Collab**. Die Schaltfläche wird zu **Leave**, und für dich erscheint ein Chip. Gib im zweiten Browser
   „Bo“ ein und tritt bei. Jeder Chip zeigt das aktuelle Frame des Teilnehmers (`Ana (you) @14`, `Bo @0`).
3. Wähle Dinge aus. Die andere Person sieht deinen Auswahlrahmen und Cursor in deiner Farbe mit deinem Namen darauf (Bo wählte `leg` aus und bewegte den Zeiger; Ana sieht einen violetten Rahmen
   mit dem Namen „Bo“ und einen Punkt neben `leg` in der Ebenenliste).
4. Bearbeitet gleichzeitig. Im Test: Ana verschob die Arm-Ebene um 10 Einheiten (`Shift+ArrowRight`); Bo sah `[10,0]` in `/api/project` erscheinen. Bo blendete dann die Ebene `sun` aus, was Ana
   sofort sah.
5. Ana drückte `Ctrl+Z`. **Nur Anas Verschiebung wurde rückgängig gemacht** (der Arm kehrte zu `[0,0]` zurück); Bos ausgeblendete Sonne blieb ausgeblendet.
6. Bearbeitungen, die gemacht wurden, während jemand von einer älteren Ansicht aus arbeitete, werden zusammengeführt, nicht abgelehnt: Ich ließ die projekteigene Simulation laufen (`osatura collab-sim`, 6 Replikate, 600 zufällige gleichzeitige
   Bearbeitungen, 12 andere Zustellreihenfolgen), und alle Replikate endeten identisch, und das zusammengeführte Projekt ließ sich kompilieren.
7. Klicke in einem der beiden Browser auf **Speichern** (oder `Ctrl+S`), um den gemeinsamen Zustand in die Projektdatei zu schreiben; vorher wird nichts in die Datei geschrieben.

Getestet wurde nur Chromium, mit zwei Seiten auf einem Rechner. Ich habe weder unterschiedliche Rechner noch langsame Verbindungen noch mehr als zwei echte Browser getestet.

### Persistenz: `--collab-log`

```
$ osatura studio demo.json --collab-log session.log
...
$ # (an edit is made in a session, the server is stopped and started again)
collaboration log session.log: resumed at index 1 (1 envelopes after the checkpoint)
```

Das Log ist eine Zeilendatei (Modus `0600`, nur für den Eigentümer), die einen per Hash verketteten Checkpoint **des gesamten Projekts** (etwa 178 KB beim Demoprojekt) sowie eine Zeile pro akzeptierter Bearbeitung enthält. Anhänge werden
pro Anfrage per `fsync` gesichert, sofern nicht `--collab-nosync` gilt. Nach einem Neustart wird der Sitzungszustand fortgesetzt (ich änderte in einer Sitzung den Projektnamen; nach dem Neustart hatte `/api/project` immer noch den
neuen Namen), während die Datei auf der Festplatte bis zum Speichern unverändert blieb (`OSATURA demo`). **Teilnehmer und Rückgängig-Verlauf werden nicht persistiert**: Seiten treten mit neuen IDs erneut bei. Ein beschädigtes Log oder ein Log
eines anderen Projekts hält den Server mit einem Fehler an, statt überschrieben zu werden. Ein Absturz während eines Anhängens kostet höchstens eine abgeschnittene letzte Zeile.

**Das Log enthält dein gesamtes Projekt im Klartext; schütze es wie das Projekt selbst.**

## 9. HTTP-API-Referenz

Die Seite des Studios verwendet diese Routen, und du kannst sie ebenfalls nutzen (Skripte, Tests). Alle Bodys und Antworten sind JSON, sofern nichts anderes angegeben ist; Fehler haben die Form `{"ok": false, "errors": ["..."]}` mit dem Status 400 (oder 401, 403,
404, 415, 421, 429, 503 wie in Abschnitt 2 beschrieben). Erfolgsantworten enthalten `"ok": true`, wo es vermerkt ist. `state` bezeichnet unten das Dokument, das `GET /api/state` zurückgibt:
`{bones:[{name,rest_angle}], channels:[...], fps, frames, width, height, can_undo, can_redo, rev, ok}`; `rev` steigt bei jeder Änderung. Jede Route unterliegt den Prüfungen aus Abschnitt 2.

### Lesen

| Route | Parameter | Antwort |
|---|---|---|
| `GET /` | | die Studio-Seite (HTML) |
| `GET /source` | | die Seite mit dem AGPL-Quellcode-Angebot; ohne Token offen |
| `GET /api/state` | | `state` |
| `GET /api/project` | | das gesamte Projekt-JSON |
| `GET /api/layers` | | `{ok, layers:[{path, depth, kind, visible, locked, locked_eff, reference, ptr}]}` (`locked_eff` berücksichtigt auch gesperrte Gruppen darüber) |
| `GET /frame/N.svg` | `bones=1` zeichnet das Skelett | ein Frame als SVG (N darf gebrochen sein); Header `X-Cache`, `X-Render-Ms` |
| `GET /api/frame` | `frame`, `proxy` 0-4 (0 = volle Qualität, 1-4 Scrub-Proxys), `kind` = `vector` (Standard) \| `raster` \| `png`, `bones=1`, `lod=1&screen=WxH&lodpx=P` (aus einer detailreduzierten Kopie zeichnen, innerhalb von P Pixeln, 0-16) | das Bild; Header `X-Cache` (hit/miss), `X-Render-Ms`, `X-Lod-Level`, `X-Lod-Bound-Px`, `X-Proxy-Level`. Ein Frame außerhalb des Bereichs: `{"errors":["frame 999 out of range"]}`; ein ungültiger kind: `bad kind 'x' (vector, raster, png)` |
| `GET /api/cache` | | `{ok, revision, entries, bytes, hits, misses, invalidations, lod:{builds, engines, invalidations, last_build_ms, levels, unavailable}}` |
| `GET /api/overlays` | `frame`, `sel` (durch Kommas getrennte Ebenennamen) | `{ok, frames, settings, ...}` sowie Ghosts und SVG für den Frame, wenn einer angegeben ist |
| `GET /api/sounds` | | `{ok, total, sounds:[{name, start, gain, duration, peaks, error?}]}` Wellenform-Peaks nach Trim, Fades und Gain |
| `GET /api/vcs/log` | `limit` (max. 1000), `rev` | `{ok, branch, commits:[{hash, author, time, message, parents}]}`; `{"ok":false,"errors":["no .osatura-vcs repository found ..."]}` ohne Repository (siehe `osatura vcs`) |
| `GET /api/vcs/diff` | `a` (Standard `HEAD`), `b` | `{ok, changes}` |
| `GET /api/timeline` | | `{ok, fps, frames, groups, rows, tracks}` das Dope-Sheet-Modell |
| `GET /api/graph` | `ch` (durch Kommas getrennte Kanalindizes), `f0`, `f1`, `n` (Samples, 2-2000, Standard 200) | `{ok, channels, f0, f1, range, skipped}` Kurven-Samples und Tangenten-Handles |
| `GET /api/live` | | `{ok, mapping:{params,triggers}, recorded, recording}` |
| `GET /api/ui` | | `{ok, layout, presets, preset_names, default_layout, shortcuts, commands, conflicts, reserved, overlays, recents, panels, persisted, path, warning, ...}` |
| `GET /api/ui/search` | `q`, `limit` (1-200, Standard 50) | `{hits:[{id, title, category, description, bindings, score, positions}]}` das Ranking der Palette |
| `GET /export.gif`, `/export.html`, `/export.html.nosound`, `/export.lottie.json` | | die Datei, als Download (`animation.gif`, `player.html`, `animation.json`) |

### Bearbeiten

| Route | Body | Antwort |
|---|---|---|
| `POST /api/op` | `{"op":"set","path":"/fps","value":30}` — `op` ist `set`, `insert` (Array-Pfad, `.../-` hängt an), `remove` oder `batch` (`{"op":"batch","ops":[...]}`, ein Undo-Schritt). Pfade sind JSON Pointers in das Projekt. | `state`. Ein ungültiges Ergebnis wird mit `{"errors":["invalid project: invalid value: integer `-3`, expected u32"]}` abgelehnt, und nichts ändert sich |
| `POST /api/undo`, `POST /api/redo` | keiner | `state` (pro Benutzer in einer gemeinsamen Sitzung) |
| `POST /api/key` | `{"target","prop","frame","value","ease"?}` (`ease` wie `{"type":"smooth"}`; Standard linear) | `state` |
| `POST /api/delete_key` | `{"target","prop","frame"}` | `state` |
| `POST /api/save` | keiner | `{"ok":true,"saved":"<path>"}`; schreibt atomar. Nur Host, wenn das Studio mit `--collab-host` läuft (sonst **403**) |
| `POST /api/suggest` | `{"kind":"between"\|"complete"\|"next","a","b","t","frame","constraints","free","top","snap","prior","apply":RANK, "library"?, "model"?}` | `{ok, suggestions}`; mit `apply` zusätzlich `state` der Bearbeitung. Eine Anfrage ohne `kind` ergibt `bad request: missing field `kind`` |

### Auswahl und Gizmos (zustandslos: die Seite hält die Auswahl)

Ein **Target** ist `{"kind":"layer","path":"arm"}`, `{"kind":"bone","name":"hand"}` oder `{"kind":"point","layer":"...","point":...}`.

| Route | Body | Antwort |
|---|---|---|
| `POST /api/pick` | `{"frame","x","y","options"?}` (`options`: `tolerance`, `bone_radius`, `point_radius`, `layers`, `bones`, `point_layers`, `locked`, `locked_bones`) | `{ok, hit, hits:[{distance, target}]}` der oberste zuerst |
| `POST /api/marquee` | `{"frame","rect":[x0,y0,x1,y1]` oder `"lasso":[[x,y],...]` (3 bis 5000 Punkte)`,"mode":"bbox"\|"contain"\|"intersect","options"?}` | `{ok, targets}` |
| `POST /api/select` | `{"frame","selection":[...],"targets":[...],"mode":"replace"\|"add"\|"toggle"\|"subtract"}` | `{ok, selection, bounds, gizmo:{corners, handles}, points}` |
| `POST /api/gizmo` | `{"frame","selection","handle","start":[x,y],"delta":[dx,dy],"modifiers":{"constrain","center","no_snap"},"snap":{"grid","objects","canvas","tolerance"},"auto_key","locked","pivot","commit":bool}` — `handle` ist `{"type":"move"}`, `rotate`, `{"type":"scale","at":"nw"}`, `bone_head` oder `bone_tip` | ohne `commit`: `{ok, drag:{changed, edits, guides, matrix, summary, box_after, outline}}` (eine Vorschau); mit `commit:true`: `state` plus `drag` und `selection`, ein Undo-Schritt |
| `POST /api/nudge` | `{"frame","selection","delta":[dx,dy],"auto_key","locked"}` | `state` + `drag` (Bearbeitungen mit `mode` `static` oder `key`) + `selection` |
| `POST /api/delete` | `{"frame","selection"}` | `state` + `selection`; `{"errors":["nothing selected"]}`, wenn leer |

### Timeline

| Route | Body | Antwort |
|---|---|---|
| `POST /api/timeline/select` | `{"selection":[{ch,frame}...],"mode":"replace"\|"add"\|"toggle"\|"subtract","all":true` oder `"channel":N` oder `"group":N` oder `"box":{f0,f1,row0,row1}`, `"include_locked"}` | `{ok, selection:[{ch,frame}...]}` |
| `POST /api/timeline/op` | `{"op":{...},"preview":bool}` — `op` ist eines von `move_keys`, `scale_keys`, `copy`, `paste`, `duplicate`, `delete`, `set_ease`, `set_value`, `add_key`, `ripple`, `offset`, `extrapolate`, `make_loop`, `handle`, `smooth`, `flags` (ein unbekannter Name listet diese auf) | mit `preview:true`: `{ok, preview:true, plan}`, ohne etwas zu ändern; andernfalls `state` + `plan:{changed, notes, op, clip}` |

Zum Beispiel liefert `{"preview":true,"op":{"op":"delete","keys":[{"ch":0,"frame":12}]}}` den Plan (`changed: 1`) und lässt das Projekt unberührt.

### Overlays, Live, Gesicht, UI

| Route | Body | Antwort |
|---|---|---|
| `POST /api/overlays` | `{"settings":{"onion":{"enabled":true,"before":3}},"frame","selection":[names],"reset":bool}` | `{ok, frames, ghosts, settings, svg}`; die Einstellungen werden in der UI-Konfiguration gespeichert |
| `POST /api/trails` | `{"targets":[{"kind":"bone","name":"hand"}],"from","to"` oder `"frame","before","after"`, `"smooth","svg":bool,"hit":{x,y,radius}}` | `{ok, from, to, trails:[{curve,frames,keys,...}], svg?, hit?, path_hit?}` |
| `POST /api/live/mapping` | `{"mapping":{params,triggers},"library"?}` (gleiches Format wie `live-map`) | Live-Status |
| `POST /api/live/push` | `{"events":[{source,id,value,time}],"midi":[raw...]}` | Live-Status mit der Wirkung der Events |
| `POST /api/live/record` | `{"on":true}` startet; `{"on":false,"tol":0.01}` stoppt und **backt die Performance in das Projekt** als ein Undo-Schritt | Status / `state` + `live`-Zusammenfassung |
| `POST /api/face/config` | `{"config"?,"calibration"?}` | `{ok}` |
| `POST /api/face/calibrate` | `{"on":true}` / `{"on":false}` | `{ok, frames, calibration}` |
| `POST /api/face/push` | `{"frames":[...landmarks...],"layout"?,"size"?}` (höchstens 10000 Frames pro Push) | Live-Status |
| `POST /api/face/detect` | `{"image":"<base64 PNG/JPEG or data: URI>","track":bool,"push":bool,"t"}` | `{ok, found, layout:"osatura23", size, points, box, note:"detector trained on synthetic faces only"}` |
| `POST /api/face/synth` | `{"t","size":32-512,"seed"}` | `{ok, image:"data:image/png;base64,...", layout, size, points}` ein Frame des synthetischen Kopfes mit seinen exakten Landmarks |
| `POST /api/ui` | eines von `{"layout":{...}\|null,"shortcuts":{...}\|null,"recents":[...],"overlays":{...},"reset":"layout"\|"shortcuts"\|"overlays"\|"all"}` | `GET /api/ui`-Dokument; unbekannte Felder (`unknown field 'bogus'`), Konflikte und unvollständige Layouts werden abgelehnt, und nichts ändert sich |
| `POST /api/login` | `{"token":"..."}` | `{ok:true}` und `Set-Cookie: osatura_token=...; Path=/; HttpOnly; SameSite=Strict`; falsch: 401 (429 ab dem 5. Fehlversuch); ohne konfiguriertes Token `{"ok":true,"token_required":false}` |

### Zusammenarbeit

Aufrufe als Teilnehmer senden `X-Collab-Replica: <id>` und `X-Collab-Secret: <secret>` (beide von join zurückgegeben). Eine Replica-ID mit falschem oder fehlendem Secret erhält **403**.

| Route | Body / Query | Antwort |
|---|---|---|
| `POST /api/collab/join` | `{"name":"Ana","mode":"thin"\|"replica"}` | `{ok, replica, secret, colour, name, index, mode, clock, users, presence, doc}` (`snapshot` im Replica-Modus). Der erste Beitritt startet die Sitzung mit dem Dokument des Editors |
| `POST /api/collab/leave` | `{"replica"}` | `{ok, left}` |
| `GET /api/collab/state` | `replica`, `doc=1`, `snapshot=1` | `state` + `collab_active` (+ users, presence) |
| `GET /api/collab/log` | `since=N` | `{ok, index, entries:[{index, by, envelope}]}` |
| `POST /api/collab/ops` | `{"replica","ops":[...]` oder `"op":{...}`, `"base"?}` oder `{"replica","envelopes":[...]}` | `state` + `applied`, `index`. Unbekannter Teilnehmer: `unknown participant N: join first`; leere Liste: `no ops`; ungültige Projektbearbeitungen werden abgelehnt |
| `POST /api/collab/undo`, `.../redo` | `{"replica"}` | `state` + `collab_can_undo`, `collab_can_redo` (nur deine eigenen Bearbeitungen) |
| `POST /api/collab/presence` | `{"replica","n","frame","cursor":[x,y],"selection":[names up to 64]}` | `{ok}` |
| `POST /api/collab/compact` | `{}` | Kompaktierungs-Zusammenfassung (`ok`, `logged`). Nur Host, wenn das Studio mit `--collab-host` läuft (sonst **403**) |
| `POST /api/collab/revoke` | `{"names":["ana","bob"]}` (1 bis 64 Namen) | `{ok, revoked, revoked_total}`: Die Tokens dieser Namen werden ab sofort abgelehnt, und Teilnehmende, die unter diesen Namen beigetreten sind, werden entfernt. Erfordert Tokens pro Teilnehmer (`--collab-secret-file`) und ein Host-Token (`--collab-host`) oder das gemeinsame `--collab-token`; sonst **403**. Ein bereits geöffneter Event-Stream endet, wenn sein Client die Verbindung trennt |
| `GET /api/collab/events` | `since=N` oder `Last-Event-ID` | ein **Server-Sent-Events**-Stream: Wiedergabe ab `since`, dann Benutzer, Presence und jedes spätere Event; `resync`, wenn der Client zu weit zurückliegt; höchstens 64 Streams (`503 too many event streams`) |

In einer aktiven Sitzung werden auch die gewöhnlichen Bearbeitungsrouten (`/api/op`, `/api/undo`, `/api/redo` und die eigenen Bearbeitungen des Studios über Nudge, Key oder Gizmo) für die anderen Teilnehmer aufgezeichnet.

## 10. Fehlerbehebung

| Symptom | Ursache und Lösung |
|---|---|
| Der Browser zeigt 421 oder „misdirected request“ | Du hast das Studio unter einem Namen geöffnet, den es nicht kennt (ein DNS-Name, eine LAN-Adresse). Füge ihn mit `--allow-host name` hinzu oder öffne `http://127.0.0.1:PORT/`. |
| 403 bei jeder Anfrage hinter einem Proxy | Der Proxy verändert `Origin`/`Host`. Gib den öffentlichen Origin an: `--public-origin https://studio.example.org`. |
| 401 | `--collab-token` ist gesetzt: Öffne die Seite und gib das Token einmal ein, oder sende `X-Collab-Token`. Ein Token in der URL wird nicht akzeptiert. |
| 429 | Fünf falsche Tokens in Folge von deiner Adresse; warte bis zu 30 s. Hinter einem Proxy teilen sich alle Benutzer eine Adresse. |
| `refusing to listen on 0.0.0.0:...` | Bind auf eine Nicht-Loopback-Adresse ohne Token. Füge `--collab-token` hinzu (16 oder mehr Zeichen). |
| `cannot listen on 127.0.0.1:8080: Address already in use` | Ein anderer Prozess belegt den Port; wähle `--port`. Meist ist ein veraltetes Studio eines früheren Laufs die Ursache. |
| Die Seite lädt, aber das Bild ist leer | Sieh im Terminal nach; ein Projekt, das `check` nicht besteht, wird beim Start abgelehnt. Nach der Korrektur neu laden. |
| Eine Browser-Konsolenmeldung `Failed to load resource ... 404` | Ältere Builds hatten kein `/favicon.ico`; es wird jetzt ausgeliefert. Ein verbleibender 404 stammt von etwas anderem. |
| Der Text des Face-Panels | Das Panel weist jetzt darauf hin, dass der auf synthetischen Daten trainierte Detektor enthalten ist, nicht an Fotos oder Webcams validiert wurde und für eine echte Webcam ein eigener Landmark-Provider nötig ist. |
| Das Layout ist kaputt oder Panels fehlen | Menü `Layout`, Layout zurücksetzen, oder die beim Start angezeigte `ui.json` löschen. Eine Konfigurationsdatei, die nicht vollständig gelesen werden kann, wird nach `ui.json.bak` kopiert (dann `.bak.1`, ...), bevor Standardwerte verwendet werden, und `GET /api/ui` hat dazu ein Feld `warning`. |
| Meine Bearbeitungen sind nach einem Neustart verschwunden | Nichts wird geschrieben, bevor du **Speichern** wählst. Mit `--collab-log` überlebt die *Sitzung* einen Neustart, aber die Datei auf der Festplatte wird weiterhin nur durch Speichern geschrieben. |
| `Ctrl+W`, `Ctrl+T` und Ähnliches lassen sich nicht belegen | Von Browsern reserviert (Liste in Abschnitt 3). |
| Strg+Z macht die Änderung eines anderen in einer Sitzung nicht rückgängig | Gewollt: Undo gilt pro Benutzer. |
| Exporte verlangen `ffmpeg` | Nur für Ton in HTML (mp3) und für mp4/webm auf der Kommandozeile; GIF, Lottie und HTML ohne Ton brauchen nichts. |

## 11. Barrierefreiheit und Browser-Unterstützung

**Browser.** Die Oberfläche wurde nur in **Chromium** getestet (dem mitgelieferten Build von Playwright, headless betrieben, unter Linux). Firefox und Safari wurden nicht ausprobiert und funktionieren vielleicht, vielleicht auch nicht.
Web MIDI erfordert einen Browser, der es implementiert (Chromium tut das); die Kamera benötigt `getUserMedia` und einen sicheren Kontext (HTTPS oder `localhost`). Die Seite nutzt Server-Sent Events für die
gemeinsame Sitzung. Für das Ziehen werden Pointer-Events verwendet; Touchscreens wurden nicht getestet.

**Barrierefreiheit, ehrlich gesagt.** Ich habe das Studio weder mit einem Screenreader noch in einer reinen Tastatursitzung geprüft. Vorhanden sind: ein `lang`-Attribut und ein Viewport-Meta-Tag, beschriftete Checkboxen, ein
Toast mit `role="status"`, einige ARIA-Attribute, **ein vollständiger Satz an Tastenkürzeln und eine Befehlspalette**, die jeden Befehl erreichen, umbelegbare Tastenkürzel sowie die Kommandozeile `osatura`, die
alles kann, was der Editor kann, ohne Zeigegerät. Es fehlt: Die Canvas ist ein Bild, und das meiste Bearbeiten auf der Canvas erfolgt per Zeigegerät (Verschieben mit den Pfeiltasten gibt es), es gibt kein helles Theme und keine
Behandlung von `prefers-reduced-motion` oder `prefers-color-scheme`, die Fokusumrandungen sind die Browser-Standards und ich habe sie nicht geprüft, und ich habe den Farbkontrast nicht gemessen. Wenn du auf assistive Technik angewiesen bist,
nutze bevorzugt die CLI und das JSON-Panel und melde dem Projekt, was nicht funktioniert.

## 12. Bekannte Ecken und Kanten

* Seit der ersten Beschreibung behoben: der Text des Webcam-Face-Panels, der 404 bei `/favicon.ico`, `--collab-nosync` ohne `--collab-log` und das fehlende `Secure`-Cookie-Flag hinter einem https-`--public-origin`.
* `osatura render p.json 99 o.png` bei einem Projekt mit 24 Frames ist jetzt ebenfalls ein Fehler (mit `--clamp` wird der letzte Frame gerendert); das `/api/frame` des Studios lehnt einen Frame außerhalb des Bereichs ab (`frame 999 out of range`) und akzeptiert jetzt nur noch die Frames `0..frames-1`.
* Änderungen an Layout und Tastenkürzeln werden pro Benutzer in der UI-Konfiguration gespeichert, nicht im Projekt, und wandern daher nicht mit der Datei mit.
