<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Häufig gestellte Fragen

*Kurze Antworten, jede aus etwas entnommen, das ausgeführt oder im Quellbaum und in den anderen Dokumenten nachgelesen wurde. Dokumentation unter CC-BY-4.0. Wo eine Antwort von einer Einschränkung abhängt, steht das dabei, mit einem Verweis auf [LIMITATIONS.md](LIMITATIONS.md). Die Fehlermeldungen im letzten Abschnitt stammen aus einem echten Build; sie sind wörtlich zitiert, sodass du diese Seite nach dem Text durchsuchen kannst, den du siehst.*

Inhalt: [Installation und Einrichtung](#installation-und-einrichtung) - [ffmpeg](#ffmpeg) - [Performance](#performance) - [Determinismus](#determinismus) - [Lizenzierung und deine Ausgabe](#lizenzierung-und-deine-ausgabe) - [Dateisicherheit und Asset-Pfade](#dateisicherheit-und-asset-pfade) - [Warum die Ausgabe sich von der Eingabe unterscheiden muss](#warum-die-ausgabe-sich-von-der-eingabe-unterscheiden-muss) - [Rigging und Animation](#rigging-und-animation) - [Studio](#studio) - [Gesicht, Sprache und Motion Capture](#gesicht-sprache-und-motion-capture) - [Bugs, Sicherheit und Mitwirken](#bugs-sicherheit-und-mitwirken) - [Einschränkungen](#einschränkungen) - [Vergleiche mit anderen Werkzeugen](#vergleiche-mit-anderen-werkzeugen) - [Tabelle zur Fehlersuche](#tabelle-zur-fehlersuche)

---

## Installation und Einrichtung

**1. Wie installiere ich OSATURA?**
Fertige Archive (Windows x86-64, macOS arm64, Linux x86-64 und das WebAssembly-Modul) sind als Entwurfs-Vorabversion `v0.1.0-alpha.3` auf <https://github.com/chiuta/osatura/releases> vorbereitet; sie lassen sich herunterladen, sobald der Eigentümer den Entwurf veröffentlicht, und sie sind nicht codesigniert (SmartScreen zeigt „Unbekannter Herausgeber“; macOS verlangt eine Bestätigung). Es gibt keinen Installer: Entpacke das Archiv und starte `osatura`. Oder baue es aus dem Quellcode mit einer aktuellen stabilen Rust-Toolchain: `cargo build -p osatura-cli` liefert `target/debug/osatura`; mit `--release` entsteht `target/release/osatura`. Siehe [DEVELOPMENT.md](DEVELOPMENT.md).

**2. Welche Plattformen funktionieren?**
Linux x86-64 ist die Plattform, auf der das Projekt entwickelt und erprobt wurde. Auf GitHub Actions bestehen die Testsuite und clippy auch unter macOS (arm64) und Windows (x86-64), und dort werden Release-Binärdateien für alle drei gebaut; aber noch keine Person hat die Windows- oder macOS-Programme auf einem echten Desktop benutzt, sie sind also „von CI getestet“, nicht „unterstützt“ ([LIMITATIONS.md](LIMITATIONS.md), Abschnitt 1).

**3. Gibt es einen grafischen Installer oder eine App?**
Es gibt keinen Installer. Die Kommandozeile ist das Produkt. Das Studio ist eine lokale Webseite, die `osatura studio` bereitstellt; unter Windows startet ein Doppelklick auf `osatura.exe` es und öffnet deinen Browser. Die Archive für Windows und macOS enthalten außerdem `osatura-studio`, ein kleines natives Fenster um dasselbe Studio (es wurde in CI nur kompiliert und selbstgetestet); siehe [STUDIO.md](STUDIO.md) und [GETTING_STARTED.md](GETTING_STARTED.md).

**4. Wie komme ich schnell los?**
Folge [GETTING_STARTED.md](GETTING_STARTED.md): Dort entsteht aus einem leeren Projekt eine animierte Figur, jeder Schritt wurde ausgeführt, und am Ende steht ein GIF.

**5. Kann ich ohne die Scripting-Engine bauen?**
Ja: `cargo build -p osatura-cli --no-default-features` lässt Rhai weg, sodass der Befehl `script` fehlt (siehe [DEVELOPMENT.md](DEVELOPMENT.md)). Das Feature `scripting` ist die einzige Stelle, an der ein Crate mit schwachem Copyleft (smartstring) hinzukommt; siehe [THIRD_PARTY.md](THIRD_PARTY.md).

**6. Braucht es eine Netzwerkverbindung oder ein Konto?**
Nein. Das Programm braucht zum Arbeiten weder ein Konto noch ein Netzwerk (das Studio kennt keine Konten). Standardmäßig lauscht das Studio nur auf deinem eigenen Rechner (siehe Frage 51).

**7. Wo speichert das Studio seine Einstellungen?**
In `$OSATURA_UI_CONFIG`, falls gesetzt, sonst in `$XDG_CONFIG_HOME/osatura/ui.json`, sonst unter Windows in `%APPDATA%\OSATURA\ui.json`, sonst in `~/.config/osatura/ui.json`. Deine Projekte werden davon nie angefasst.

---

## ffmpeg

**8. Brauche ich ffmpeg?**
Nur für den MP4- und WebM-Export, für `sound mix`, für mp3/ogg-Audio und für `face-track` auf einer Videodatei. Der Export als GIF, APNG, HTML, PNG-Sequenz, SVG, glTF und JSON braucht nichts weiter. Die vollständige Liste steht in [CLI_REFERENCE.md](CLI_REFERENCE.md), Abschnitt 1.6.

**9. Wird ffmpeg mitgeliefert?**
Nie. OSATURA startet `ffmpeg` aus deinem `PATH` als eigenständiges Programm, und sonst wird nichts ausgeführt. Welche Lizenz- und Patentlage dein ffmpeg-Build hat, musst du selbst prüfen; siehe [PATENT_NOTES.md](PATENT_NOTES.md).

**10. Was wird gemeldet, wenn ffmpeg fehlt?**
`cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` (bei WebM endet der Hinweis mit `or export an APNG instead`). Es wird nichts geschrieben.

**11. Welche Encoder muss mein ffmpeg haben?**
`libx264` für mp4, `libvpx-vp9` oder `libaom-av1` für webm, `aac` und `libopus` für Ton darin, `libvorbis` und `libmp3lame` für `sound mix`.

---

## Performance

**12. Warum ist es so langsam?**
Vermutlich läufst du mit einem Debug-Build. Die Zahlen in den Dokumenten stammen aus einem Debug-Build und sind 10- bis 30-mal langsamer als bei einem Release-Build ([CLI_REFERENCE.md](CLI_REFERENCE.md), Abschnitt 1.7). Auf dem Testrechner mit 2 Kernen brauchte ein Debug-Build 46 s, um die 48 Frames des Demoprojekts als PNG zu schreiben, und 57 s, um es als GIF zu exportieren. Ein frischer Release-Build erledigte dieselben beiden Aufgaben in 1,5 s und 2,2 s. Baue mit `cargo build --release` und verwende `target/release/osatura`.

**13. Nutzt es die GPU?**
Der Standard-Renderer läuft auf der CPU. Ein `gpu`-Crate und ein Befehl `render-gpu` existieren hinter einem Feature-Flag, aber beim Bauen stand keine echte GPU zur Verfügung: Auf einer Software-GPU (llvmpipe) brauchte ein 640x480-Frame 253 bis 295 ms gegenüber 13,8 ms auf dem CPU-Pfad. Betrachte GPU-Rendering auf Hardware als ungetestet ([LIMITATIONS.md](LIMITATIONS.md)).

**14. Was kostet ein Rig pro Frame?**
In einem Debug-Build hatte das Beispiel `biped` im Performance-Check von `ci` einen Median von 64,5 ms pro Frame. Ein Stoff-Bake einer kleinen Fahne dauerte 0,43 s. Miss dein eigenes Projekt mit `osatura stats` und `osatura ci`; Rigs unterscheiden sich stark.

**15. Wie mache ich ein schweres Projekt leichter?**
Nutze `lod`, um günstigere Varianten zu erzeugen, `export-fit`, um die Exporteinstellungen zu finden, die ein Byte-Budget einhalten, und `proxy-bench`, um Proxys zu vergleichen. Siehe [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) und [EXPORT_FORMATS.md](EXPORT_FORMATS.md). `export-fit` kann mit `--max-seconds` vorzeitig abbrechen und meldet dann, dass die Zeit ausgegangen ist.

**16. Ein Skript mit Tausenden von Aufrufen schlägt fehl. Warum?**
`osatura script` akzeptiert höchstens 5000 Anfragen pro Lauf und arbeitet nach dem Prinzip alles oder nichts: Wird eine Anfrage abgelehnt, wird nichts geschrieben. Teile eine große Aufgabe in mehrere Skripte auf, die die vorherige Ausgabe lesen.

---

## Determinismus

**17. Ist die Ausgabe reproduzierbar?**
Das Rendering ist darauf ausgelegt: Dasselbe Projekt und derselbe Seed ergeben dieselben Pixel, und Physik, Partikel und Noise sind mit einem Seed initialisiert. Dadurch können `golden` und `ci` Renderings vergleichen. Geprüft wurde nur auf einem Rechnertyp (Linux x86-64); bitgleiche Ausgabe auf anderen CPUs oder Compilern wird nicht versprochen.

**18. Warum liefert ein Physik-Bake ein anderes Ergebnis, wenn ich eine Zahl ändere?**
Der Bake ist für eine gegebene Eingabe deterministisch, aber er ist eine Simulation: Jede Änderung an Masse, Framerate, Collider oder Seed ändert den gesamten Lauf. Bake mit denselben Einstellungen und demselben Seed, um zu vergleichen. Das Baken wird standardmäßig mit vorhandener Animation zusammengeführt (`merge=keep`: vorhandene Kanäle bleiben erhalten, und der Befehl meldet `kept existing animation on 'b2' (use merge=replace to overwrite)`); mit `merge=replace` überschreibst du sie.

**19. Läuft Noise als Loop?**
Nein. Ein `noise`-Kanal wiederholt sich nicht von selbst, ein Loop zeigt also an der Naht einen Sprung. Verwende stattdessen Keys oder einen Ausdruck wie `8 * sin(t * 2 * pi)`; in Ausdrücken läuft `t` über den Clip von 0 bis 1, nicht in Frames ([ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)).

**20. Kann ich prüfen, ob eine Animation sauber loopt?**
Führe `lint` aus. `make-loop` entfernt die Fortbewegung vollständig: In einem Test wurde ein rollender Ball statisch, und `lint` warnte (L005). `loop-solve` schließt die Naht bei zyklischer Bewegung, ließ aber bei einem sich fortbewegenden Objekt einen sichtbaren Sprung (280 zu 280 auf seinem Bewegungskanal). Verwende für fortlaufende Bewegung den Extrapolationsmodus `loop_offset`.

---

## Lizenzierung und deine Ausgabe

*Dies ist eine allgemeinverständliche Zusammenfassung von [LICENSING.md](LICENSING.md), keine Rechtsberatung. Eine anwaltliche Prüfung der Lizenzierung steht noch aus.*

**21. Welche Lizenzen gelten?**
MPL-2.0 für die Kernbibliothek, die C-ABI sowie die WASM- und GPU-Crates; AGPL-3.0-or-later für das Kommandozeilenprogramm und das Studio (`osatura-cli`); CC-BY-4.0 für die Dokumentation und für die synthetischen Beispiel-Assets und -Modelle.

**22. Wem gehören die Animationen und Renderings, die ich erstelle?**
Dir. Die Projektaussage lautet, dass Projekte, Animationen und Renderings, die du mit OSATURA erstellst, vollständig dir gehören; die Lizenzen gelten für die Software, nicht für ihre Ausgabe. Das Exportieren eines GIFs oder das Ausführen eines Befehls löst keine Pflicht aus.

**23. Darf ich die Ausgabe kommerziell nutzen?**
Ja, gemäß der obigen Aussage. Die Ausnahme ist alles, was du aus den Beispiel-Assets kopierst: Diese stehen unter CC-BY-4.0 und verlangen eine Namensnennung.

**24. Muss ich meine Projekte veröffentlichen, wenn ich die AGPL-Teile nutze?**
Nein. Die AGPL betrifft das Programm. Wenn du ein verändertes `osatura-cli` oder Studio so betreibst, dass andere Personen es über ein Netzwerk nutzen, musst du ihnen dessen Quellcode anbieten. Das Studio erledigt das mit einer Seite `/source` und einem Flag `--source-url`, das auf deine Kopie zeigt.

**25. Kann ich die Kernbibliothek in eine Closed-Source-Anwendung einbinden?**
Der Kern steht unter MPL-2.0, einem Copyleft auf Dateiebene: Du darfst ihn aus geschlossenem Code verwenden und musst Änderungen an den von der MPL erfassten Dateien selbst weitergeben. CLI und Studio stehen unter der AGPL und sind eine andere Sache.

**26. Darf ich sagen, mein Werkzeug „funktioniert mit OSATURA“?**
Ja: „kompatibel mit OSATURA“ und „erstellt mit OSATURA“ sind erlaubt. Ein veränderter Build darf nicht unter demselben Namen verbreitet werden ([TRADEMARK.md](TRADEMARK.md)).

**27. Kann ich meinen Beitrag doppelt lizenzieren?**
Es gibt keine Contributor-Lizenzvereinbarung. Beiträge werden unter der Projektlizenz mit einem DCO-Sign-off (`git commit -s`) angenommen, es gibt also keinen Eigentümer, der sie neu lizenzieren könnte. Details in [CONTRIBUTING.md](CONTRIBUTING.md).

**28. Was ist mit dem Code von Drittanbietern?**
Etwa 360 Crates von Drittanbietern (über alle Plattformen gezählt; die Fenster-App fügt das Toolkit für Windows / macOS hinzu), alle unter permissiven Lizenzen außer sechs unter MPL-2.0: smartstring (MPL-2.0 oder später, nur mit dem Feature `scripting`) und fünf, die nur die Fenster-App einbinden kann. Die Listen stehen in [THIRD_PARTY.md](THIRD_PARTY.md) und [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md).

**29. Gibt es Patentbedenken?**
Siehe [PATENT_NOTES.md](PATENT_NOTES.md). Das sind technische Notizen, kein Rechtsgutachten.

---

## Dateisicherheit und Asset-Pfade

**30. Kann ein Befehl mein Projekt beschädigen?**
Schreibvorgänge sind atomar: Das Ergebnis geht in eine temporäre Datei im selben Verzeichnis, wird synchronisiert und über das Ziel umbenannt, sodass bei einem Absturz die alte Datei erhalten bleibt. Bearbeitende Befehle validieren das Projekt, das sie schreiben wollen, erneut und weigern sich, ein ungültiges zu schreiben (`output would be invalid: ...`). Der Befehl `script` wendet nichts an, wenn eine Anfrage fehlschlägt.

**31. Ist es sicher, ein Projekt von jemand anderem zu öffnen?**
Einigermaßen, aber nicht bedingungslos. Ein Projekt sind Daten, kein Code; das einzige externe Programm, das OSATURA startet, ist ffmpeg, für die in Frage 8 genannten Formate. Der Loader weist Geräte, Pipes und Verzeichnisse zurück, begrenzt die Eingabegröße und lehnt absolute Asset-Pfade und Pfade mit `..` ab. Rhai-Skripte laufen in einer Sandbox. Lies trotzdem [SECURITY.md](SECURITY.md), bevor du nicht vertrauenswürdige Dateien in einem Dienst verarbeitest.

**32. Wo sucht ein Projekt nach Tönen und Bildern?**
Relativ zum Ordner der Projektdatei. Absolute Pfade und jeder Pfad mit `..` werden abgelehnt, es sei denn, du übergibst `--allow-external-assets`; das musst du bei jedem späteren Befehl, der dieses Projekt lädt, erneut tun. Beispiel aus der Referenz:
`sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)`.

**33. Wie groß darf ein Projekt sein, das gelesen wird?**
Standardmäßig bis zu 256 MiB. Ändere das mit `--max-input-bytes N`. Größere Dateien scheitern mit `cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)` (die Zahlen stammen aus dem Beispiel).

**34. Warum weigert es sich, in einen symbolischen Link zu schreiben?**
Damit ein neben deiner Ausgabe platzierter Link einen Schreibvorgang nicht woandershin umleiten kann. Die Meldung lautet `refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself`.

**35. Warum kann ich nicht von `<(...)` oder einer Pipe lesen?**
Eingaben müssen reguläre Dateien sein: `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)`. Schreibe die Daten zuerst in eine Datei.

**36. Bricht ein unbekanntes Feld in einem Projekt es kaputt?**
Nein: Es wird `warning: unknown field bogus (ignored)` ausgegeben, und der Exit-Status ist 0. Nebendateien, die du für einen Befehl schreibst (Contracts, Mappings, Maschinen), sind strenger: Ein unbekannter Schlüssel darin ist ein Fehler, und es wird nichts angewendet.

---

## Warum die Ausgabe sich von der Eingabe unterscheiden muss

**37. Warum sagt `render in.png in.png` „refusing to overwrite“?**
Bei Befehlen, die ein Projekt lesen und ein Bild, Video oder eine andere Nicht-Projekt-Datei schreiben, darf der Ausgabename nicht dem Eingabenamen entsprechen. Die Meldung lautet `refusing to overwrite the input project X with output of the same name`. Das verhindert den klassischen Ausrutscher, über die einzige Kopie des Projekts zu exportieren.

**38. Warum können `script` und `autorig` dann ihre eigene Eingabe überschreiben?**
Die Sperre gilt für Ausgaben, die keine Projekte sind. Befehle, die ein Projekt lesen und ein Projekt schreiben (`script`, `autorig`, `loop-solve` und andere), dürfen dieselbe Datei für Eingabe und Ausgabe nennen; der Schreibvorgang ist atomar, und wenn der Befehl fehlschlägt, ändert sich nichts. Wenn du das Original behalten willst, gib einen neuen Namen an oder nutze `osatura vcs`, um die Historie zu bewahren ([CI.md](CI.md) und [CLI_REFERENCE.md](CLI_REFERENCE.md)).

**39. Warum lehnt `render` `out.jpg` ab?**
`render` schreibt nur `.png` oder `.svg`: `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)`. Verwende für andere Formate `export`; die Liste steht in [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

---

## Rigging und Animation

**40. In welche Richtung zeigt null Grad bei einem Knochen?**
Der Knochenwinkel 90 zeigt nach unten; Winkel werden nicht so von der x-Achse aus gemessen, wie es ein Mathematiklehrbuch tut. Kanäle sind Abweichungen von der Ruhepose, und `rest_offset` liegt im Raum des Parents ([RIGGING_GUIDE.md](RIGGING_GUIDE.md)).

**41. Meine IK-Kette streckt sich nicht oder hat einen Knick. Was sind `soft` und `stretch`?**
`soft` (0 bis 0,9) führt die Kette sanft in die volle Streckung, und `stretch` (1 bis 4) lässt sie sich verlängern. Beide gibt es nur in JSON, nicht als Rhai- oder CLI-Optionen. Bei `stretch` über 1 ist die Soft-Zone das Ende der gestreckten Reichweite. Zwei-Knochen-Ketten werden analytisch gelöst; bei drei oder mehr kommt FABRIK zum Einsatz.

**42. Meine Constraints liefern merkwürdige Ergebnisse.**
Constraints laufen in der Reihenfolge der Liste, und die Reihenfolge ist wichtig. Ein Aim mit dem Gewicht 0,5 ergibt den halben Winkel (aus einem Winkel von -74,7 wird -37,4). Ändere die Reihenfolge im JSON, wenn ein späteres Constraint das Ergebnis eines früheren sehen soll.

**43. Gibt es einen automatischen Skinner für Vektorgrafik?**
Nicht auf der Kommandozeile. `Project::auto_skin` gibt es nur in der Bibliothek. Vektorgrafik riggst du mit Knochen, die Ebenen als Parent haben, mit Lattices, Splines oder Pins (nur JSON) oder mit getracten Meshes über `mesh-trace`.

**44. `skin-diffuse` und `cloth` beanstanden das Mesh.**
Beide brauchen ein Mesh, und Cloth braucht ein **Grid**-Mesh: `cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Eine Ebene ganz ohne Mesh ergibt `layer 'flag' has no mesh`.

**45. `foot-lock` sagt, es braucht einen Root.**
`foot lock needs a root controller bone (no parent)`. Der Root-Controller darf keinen Parent-Knochen haben.

**46. `scale` lehnt meine Keyframes ab.**
`scale: frame 27 is outside the project range 0..=24` bedeutet, dass ein resultierender Key außerhalb des Projekts läge; `keyedit scale` wird außerhalb des Bereichs abgelehnt, es sei denn, du fügst `--free` hinzu. Ein Probelauf verwendet `-` als Ausgabenamen.

**47. Warum ist mein Projekt nach `mocap-retarget` länger geworden?**
`mocap-retarget` verlängert das Projekt, damit es zur aufgenommenen Bewegung passt. `follow --chain` kann Keys über das letzte Frame hinaus verschieben, was `lint` meldet (L007).

**48. Wie animiere ich eine Ebene ohne Knochen?**
Bearbeite die Kanäle `layer_x` und `layer_y` der Ebene. Schalte im Studio Auto-Key ein; ist es aus, bleibt eine Ebenenänderung statisch, sofern diese Eigenschaft nicht schon einen Kanal mit Keys hat. Knochenänderungen setzen immer Keys. Siehe [STUDIO.md](STUDIO.md).

---

## Studio

**49. Wie öffne ich das Studio?**
`osatura studio project.json` und öffne die Adresse, die es ausgibt, normalerweise `http://127.0.0.1:<port>`. Es ist eine Single-Page-App, die von OSATURAs eigenem kleinen HTTP-Server bereitgestellt wird (nur Standardbibliothek); es braucht kein Internet.

**50. Ändert das Studio meine Datei, während ich arbeite?**
Nein. Die Datei auf der Festplatte ändert sich nur, wenn du auf Speichern klickst. Schließt du die Seite ohne Speichern, gehen die Änderungen verloren (das Kollaborationslog, falls aktiviert, bewahrt das gesamte Projekt, aber nicht den Rückgängig-Verlauf).

**51. Können sich andere Personen verbinden?**
Nur wenn du eine Nicht-Loopback-Adresse bindest, und dann ist ein Token erforderlich: `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)`. Das Token muss mindestens 16 Zeichen lang sein, zum Beispiel aus `openssl rand -hex 16`. Es gibt keine Verschlüsselung: Nutze einen SSH-Tunnel oder einen Reverse-Proxy mit TLS.

**52. Wohin kommt das Token?**
In den Header `X-Collab-Token` oder das Cookie `osatura_token` (HttpOnly, SameSite=Strict und Secure, wenn die Anfrage unter einem https-`--public-origin` eintrifft). In der URL wird es nie akzeptiert. Ab dem fünften fehlgeschlagenen Versuch erhält diese Client-Adresse HTTP 429 mit wachsender Verzögerung (2, 4 ... bis zu 30 s).

**53. Was bedeuten die HTTP-Fehler 403, 415 und 421?**
421: Der Host-Header ist keiner, den das Studio erwartet. 403: ein fremder `Origin` oder `Sec-Fetch-Site`. 415: ein POST, der kein JSON ist. Das sind Schutzmaßnahmen gegen Seiten in deinem Browser, die den lokalen Server aufrufen; siehe [STUDIO.md](STUDIO.md) und [SECURITY.md](SECURITY.md).

**54. Funktioniert es in jedem Browser?**
Getestet wurde nur in Headless-Chromium. Andere sind ungetestet. Es gibt keinen Befehl zum Zoomen der Zeichenfläche, und ein Panel im Studio für Webcam-Gesichtserkennung existiert, aber eine echte Kamera stand zum Testen nie zur Verfügung ([LIMITATIONS.md](LIMITATIONS.md)).

---

## Gesicht, Sprache und Motion Capture

**55. Erkennt OSATURA Gesichter?**
Es gibt einen kleinen eingebauten Detektor, der nur mit synthetischen Gesichtern trainiert wurde. In 50 synthetischen Testszenen (Seed 9) fand er jedes Gesicht, mit einem mittleren Landmark-Fehler von 0,028 der Gesichtsgröße, und er löste in 50 Szenen ohne Gesicht 2 Fehlalarme aus. Bei Zeichnungen im Cartoon-Stil schnitt er schlecht ab: 0 von 6 kantigen Roboter-Gesichtern wurden gefunden, und die Fälle mit rundem Emoji und Manga-Oval hatten Fehler von 0,164 und 0,166. Er wurde nie mit echten Fotos oder einer echten Webcam ausgeführt. Beachte, dass [LIMITATIONS.md](LIMITATIONS.md) L-13 und der Text des Webcam-Panels im Studio besagen, dass kein Gesichtsdetektor mitgeliefert wird; das ist in Bezug auf diesen synthetischen Detektor veraltet.

**56. Wie funktioniert Lip-Sync?**
Mundsignale wählen zwischen fünf Formen (Ruhe, klein, offen, rund, breit). Ein Signalzeitpunkt mit Dezimalpunkt sind Sekunden; eine nackte Ganzzahl ist ein Frame. `--rhubarb` ändert nur, wie der Buchstabe E klassifiziert wird (rund statt breit). Audio, das länger als das Projekt ist, wird abgeschnitten.

**57. Kann ich aus Video animieren?**
`face-track` liest Video über ffmpeg. `face-bake` schreibt die Signale als Kanäle mit Keys (die Zusammenfassung lautet „+N channels (K keys in them), M trigger keys set“) und kürzt auf die Projektlänge, mit einer stderr-Warnung, die die nicht gebakten Frames zählt. Ein unbekanntes Signal ergibt `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.

---

## Bugs, Sicherheit und Mitwirken

**58. Wie melde ich einen Bug?**
Eröffne ein Issue mit der Kommandozeile, der genauen Ausgabe (auch stderr), dem Exit-Status, dem Build (`osatura --version`) und, wenn möglich, einem minimalen Projekt. Führe vorher `osatura check project.json` und `osatura lint project.json` aus: Viele Probleme werden dort gemeldet. Siehe [CONTRIBUTING.md](CONTRIBUTING.md).

**59. Wie melde ich ein Sicherheitsproblem?**
Privat, nicht in einem öffentlichen Issue; siehe [SECURITY.md](SECURITY.md). Die Adresse im Repository ist `alexio@trom.tf`; das ist dieselbe Adresse wie der Maintainer-Kontakt und wurde nicht durch ein Postfach eines Sicherheitsteams ersetzt.

**60. Gibt es eine Mailingliste, einen Chat oder einen Supportvertrag?**
Nein. Die Dokumente nennen einen Maintainer-Kontakt; die erste Version ist noch eine Entwurfs-Vorabversion.

**61. Wie trage ich bei?**
Signiere jeden Commit (`git commit -s`), halte die Tests grün (`cargo test` und bei Animationsarbeit `osatura ci`) und lies [DEVELOPMENT.md](DEVELOPMENT.md). Die Dokumentation steht unter CC-BY-4.0 und freut sich über Korrekturen, besonders dort, wo eine Seite und das Programm sich widersprechen.

---

## Einschränkungen

**62. Was ist die wichtigste einzelne Einschränkung?**
Es wurde von einem Projekt in einer Umgebung gebaut und getestet. Alles, was nicht als ausgeführt aufgeführt ist, ist „nicht getestet“. Lies [LIMITATIONS.md](LIMITATIONS.md); dort gibt es IDs, die du in Issues zitieren kannst.

**63. Bei welchen Exportformaten ist Vorsicht geboten?**
Video braucht ffmpeg. Lottie und glTF decken nur eine Teilmenge dessen ab, was OSATURA zeichnen und animieren kann; prüfe daher in [EXPORT_FORMATS.md](EXPORT_FORMATS.md), was verloren geht. Der `export`-Fehler bei einer unbekannten Endung listet `.gif .html .json .mp4 .apng .webm .png .glb` auf und lässt `.lottie` aus; `.lottie` (dotLottie) ist ein gültiges Ziel, siehe [EXPORT_FORMATS.md](EXPORT_FORMATS.md), Abschnitt 6.

**64. Ist der WASM-Build nutzbar?**
Er wird auf GitHub Actions für `wasm32-unknown-unknown` gebaut, und der Test des Wrappers besteht gegen das echte Modul unter Node 20; außerdem hängt er als `osatura-web.tar.gz` an der Entwurfsversion. Er wurde nicht in einem Browser ausgeführt, und seine Geschwindigkeit und sein Speicherbedarf sind nicht gemessen ([WASM.md](WASM.md), [LIMITATIONS.md](LIMITATIONS.md)). Betrachte ihn als teilweise verifiziert.

**65. Importiert es Projektdateien anderer Werkzeuge?**
Es importiert SVG, OpenRaster, Lottie-JSON, Bilder, GIF/APNG, BVH-Motion-Capture und MIDI ([EXPORT_FORMATS.md](EXPORT_FORMATS.md), Abschnitte 9 und 10). PSD, `.lottie`-Zip-Dateien oder OCA-Ordner importiert es nicht. Importe erzeugen ein neues Projekt. Beachte, dass `import-lottie in.json in.json` keine Überschreibsperre hat und seine Eingabe ersetzt. Erwarte nicht, dass eine importierte Datei Pixel für Pixel dem Original entspricht.

**66. Ist das Projekt-JSON stabil?**
Es ist „format 1“. Eine neuere Datei wird abgelehnt: `project format 2 is newer than this build supports (1); update OSATURA`. Da es nur eine Alpha-Vorabversion gibt, kann sich Format 1 selbst vor einer stabilen Version noch ändern; siehe [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

---

## Vergleiche mit anderen Werkzeugen

**67. Wie unterscheidet es sich von einem Timeline-basierten Desktop-Animationseditor?**
Das sind interaktive Programme, bei denen das Zeichnen im Vordergrund steht. OSATURA setzt auf die Befehlszeile: Jede Operation ist ein Befehl oder ein Skript, das Projekt ist JSON, das sich per Diff vergleichen lässt, und Renderings sind deterministisch und lassen sich daher in der CI testen. Das Studio ist eine schlanke Webansicht auf derselben Engine und bei Weitem nicht so ausgereift wie ein etablierter Editor.

**68. Wie unterscheidet es sich von einem 2D-Skelettanimationswerkzeug für Spiele?**
Jene Werkzeuge konzentrieren sich auf Meshes, Skins und Runtime-Bibliotheken für bestimmte Spiel-Engines. OSATURA hat ebenfalls Knochen, IK, Constraints und Meshes, aber weniger für die Laufzeit zu bieten: einen eigenen Player in HTML, eine C-ABI und ein WASM-Crate, das ungebaut ist. Wähle nach der Laufzeitumgebung, die du ansprechen musst.

**69. Wie unterscheidet es sich von einem Motion-Graphics-Compositor?**
Ein Compositor ist um Ebenen mit Effekten und Ausdrücken auf Videomaterial herum aufgebaut. OSATURA hat Ebeneneffekte und Ausdrücke, aber sein Schwerpunkt liegt auf Charakter-Rigging und Vektorgrafik, und es macht kein Video-Compositing, kein 3D und kein Color Grading von Material.

**70. Wann ist OSATURA die falsche Wahl?**
Wenn du heute Echtzeitleistung auf einer GPU brauchst, einen ausgereiften Workflow mit Zeichentablett, Frame-für-Frame-Handanimation, ein vom Hersteller unterstütztes Produkt mit stabilem Release und Support oder bestätigtes Verhalten außerhalb von Linux x86-64.

---

## Tabelle zur Fehlersuche

Jede Meldung unten wurde von einem echten Build erzeugt und endet mit Exit-Status 1, sofern nicht anders angegeben. Der Wortlaut einer Meldung kann sich vor einem Release ändern.

| Meldung | Ursache | Lösung |
|---|---|---|
| `cannot read nope.json: No such file or directory (os error 2)` | Falscher Pfad oder Name | Pfad und aktuelles Verzeichnis prüfen |
| `invalid JSON in broken.json: EOF while parsing a value at line 2 column 0` | Die Datei ist kein vollständiges JSON | Die Syntax in der genannten Zeile korrigieren; aus einem Backup oder mit `osatura vcs` wiederherstellen |
| `badparent.json is not a valid project:` then `bone 'a' has parent 'ghost' which is missing or listed after it` | Ein Parent-Knochen fehlt oder steht nach seinem Kind | Parents vor Kindern definieren; den Namen korrigieren |
| `invalid JSON in fmt2.json: project format 2 is newer than this build supports (1); update OSATURA` | Datei aus einem neueren Build | Einen neueren Build verwenden |
| `warning: unknown field bogus (ignored)` (Exit 0) | Ein Schlüssel, den dieser Build nicht kennt | Den Schlüssel entfernen oder korrigieren; auf Tippfehler prüfen |
| `refusing to overwrite the input project X with output of the same name` | Der Name der Ausgabedatei entspricht der Eingabe | Einen anderen Ausgabenamen wählen |
| `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)` | Falsche Endung für `render` | `.png`/`.svg` verwenden, oder `export` für andere Formate |
| `unknown option --bone for 'render' (did you mean --bones?); see: osatura (no arguments) for the usage` | Falsch geschriebene Option | Die vorgeschlagene Option verwenden |
| `unknown export format '.bmp' (use .gif .html .json .mp4 .apng .webm .png .glb)` | Endung nicht exportierbar | Ein aufgeführtes Format wählen (`.lottie` ist ebenfalls gültig) |
| `--scale needs a number in (0, 16]` | Skalierung außerhalb des Bereichs | Einen Wert über 0 und bis 16 verwenden |
| `bad frame number 'x'` | Ein Frame-Argument ist keine ganze Zahl | Eine Ganzzahl übergeben |
| `cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` | ffmpeg nicht im `PATH` | ffmpeg installieren, oder als GIF, HTML oder APNG exportieren |
| `the project has no sound tracks (see: sound add)` | Sound-Befehl bei einem Projekt ohne Ton | Mit `sound add` eine Spur hinzufügen |
| `channel hand_ctrl.BoneOffsetY is locked` | Der Kanal ist gesperrt | Entsperren oder einen anderen Kanal bearbeiten |
| `request 1 (hand_ctrl.BoneOffsetY at 0) was rejected, nothing changed: channel hand_ctrl.BoneOffsetY is locked` | Dasselbe, aus `script` | Zuerst entsperren; es wurde nichts angewendet |
| `request 1 (channel head.BoneAngle) was rejected, nothing changed: ... expression error: unknown variable 'tt'` | Tippfehler in einem Ausdruck | Variablen sind frame, time, fps, frames, pi, t |
| `... unexpected end of expression` | Unvollständiger Ausdruck | Klammern schließen, die Formel vervollständigen |
| `points need an even number of coordinates: 3..1000 points` | Ungerade Koordinatenzahl oder zu wenige Punkte | x,y-Paare angeben: 3 bis 1000 Punkte |
| `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)` | Process Substitution oder Pipe als Eingabe | Zuerst in eine reguläre Datei speichern |
| `sound add: asset '/etc/passwd': absolute paths are not allowed (...)` | Asset außerhalb des Projektordners | Neben das Projekt kopieren, oder `--allow-external-assets` übergeben |
| `cannot read demo.json: N bytes exceed the input limit of M bytes (raise it with --max-input-bytes N)` | Datei über dem Eingabelimit | `--max-input-bytes` erhöhen, wenn du der Datei vertraust |
| `refusing to write through a symbolic link; pass --follow-symlinks ...` | Ausgabe ist ein symbolischer Link | Das Ziel benennen, oder `--follow-symlinks` übergeben |
| `cloth needs a grid mesh; the layer has a free-form (traced) mesh` | Cloth auf einem getracten Mesh | Ein Grid-Mesh für die Ebene bauen |
| `layer 'flag' has no mesh` | Ein Mesh-Befehl auf einer einfachen Ebene | Zuerst ein Mesh erstellen |
| `no bone 'b9'` | Knochenname existiert nicht | Namen mit `stats` oder `explain` prüfen |
| `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)` | Falscher Signalname | Einen Namen aus der Liste verwenden |
| `foot lock needs a root controller bone (no parent)` | Der gewählte Root hat einen Parent | Einen Controller ohne Parent verwenden |
| `scale: frame 27 is outside the project range 0..=24` | Skalierte Keys liegen außerhalb des Projekts | Das Projekt verlängern, weniger skalieren oder `--free` verwenden |
| `--extrapolate must be hold, linear, loop, ping_pong or loop_offset` | Ungültiger Modus | Einen der fünf verwenden |
| `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)` | Port belegt | Einen anderen `--port` wählen oder den anderen Prozess beenden |
| `cannot listen on 127.0.0.1:99999: invalid port value` | Port über 65535 | 1 bis 65535 verwenden |
| `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (...)` | Öffentliche Bindung ohne Token | `--collab-token` hinzufügen, oder auf Loopback bleiben |
| `--collab-token must be at least 16 characters (e.g. \`openssl rand -hex 16\`)` | Schwaches Token | Ein längeres erzeugen |
| `--source-url must be an http:// or https:// URL without spaces or quotes` | Fehlerhafte URL | Eine einfache URL verwenden |
| `--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org` | Origin hat einen Pfad | Nur Schema, Host und Port angeben |
| `output would be invalid: ...` | Die Bearbeitung würde ein ungültiges Projekt hinterlassen | Die Gründe nach dem Doppelpunkt lesen; es wurde nichts geschrieben |
| `budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes ...` | `export-fit` kann das Budget nicht erreichen | Das Budget erhöhen oder das Projekt vereinfachen (die kleinste Datei wird trotzdem geschrieben) |
| `time budget exhausted (60 s ...)` | `export-fit` ist die Zeit ausgegangen | `--max-seconds` erhöhen |
| `fuzz: failure rate 98.00% exceeds --max-fail-rate 50.00%` | Posen haben das Rig gebrochen, und du hast ein Gate verlangt (`--gate` oder `--max-fail-rate`) | Die fehlgeschlagenen Fälle ansehen; die Rate nur erhöhen, wenn es gewollt ist; reines `fuzz` meldet nur (Exit 0) |
| `no .osatura-vcs repository found from ... (run: osatura vcs init <project.json>)` | Noch keine Historie | `vcs init` ausführen |
| `frame 999 out of range` (Studio-API) | Frame jenseits des Projekts | Ein Frame von 0 bis zum letzten Frame verwenden |
| `bad kind 'x' (vector, raster, png)` (Studio-API) | Falsche Art | Eine aufgeführte Art verwenden |
| `nothing selected` (Studio-API) | Ein Auswahlbefehl ohne Auswahl | Zuerst Ebenen oder Knochen auswählen |

Die Exit-Codes von `osatura ci` sind 0 bestanden, 1 fehlgeschlagen, 2 Fehler.

---

*Dokumentation CC-BY-4.0. Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio). Du darfst sie mit Namensnennung teilen und anpassen. Siehe auch: [INDEX.md](INDEX.md), [GLOSSARY.md](GLOSSARY.md).*
