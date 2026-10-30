<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Sicherheit

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/docs/SECURITY.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/SECURITY.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/SECURITY.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/SECURITY.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/SECURITY.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/SECURITY.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/SECURITY.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/SECURITY.md)

*Bedrohungsmodell, eingebaute Schutzmaßnahmen, bekannte Lücken, wie du ein Problem meldest und wie du das Studio sicher betreibst. Dokumentation unter CC-BY-4.0. Abgeleitet aus dem Quellcode (`osatura-core/src/{api,assets,fsutil,expr,script}.rs`, `osatura-cli/src/{http,studio,collab_srv}.rs`), aus `ROADMAP.md` Abschnitt 9 (Review-Wellen 1 bis 4) und aus den unten genannten Tests. **OSATURA hatte kein unabhängiges Sicherheitsaudit.** Die Review-Wellen hat das Projekt selbst durchgeführt; verstehe diese Seite als Beschreibung der Absicht und dessen, was die Tests abdecken, nicht als Garantie.*

Inhalt: [1 Bedrohungsmodell](#1-bedrohungsmodell) - [2 HTTP-Server des Studios](#2-http-server-des-studios) - [3 Authentifizierung und Zusammenarbeit](#3-authentifizierung-und-zusammenarbeit) - [4 Auflösung von Assets und Eingabegrenzen](#4-auflösung-von-assets-und-eingabegrenzen) - [5 Sandbox für Expressions und Skripte](#5-sandbox-für-expressions-und-skripte) - [6 Dateien, Symlinks und temporäre Daten](#6-dateien-symlinks-und-temporäre-daten) - [7 Externe Programme (ffmpeg)](#7-externe-programme-ffmpeg) - [8 Bibliotheken (C-ABI, WASM, Fassade)](#8-bibliotheken-c-abi-wasm-fassade) - [9 Bekannte Lücken](#9-bekannte-lücken) - [10 Eine Schwachstelle melden](#10-eine-schwachstelle-melden) - [11 Härtungs-Checkliste für Betreiber](#11-härtungs-checkliste-für-betreiber) - [12 Reverse-Proxy mit TLS: nginx und Caddy](#12-reverse-proxy-mit-tls-nginx-und-caddy)

## 1. Bedrohungsmodell

Was OSATURA verarbeitet: **Projektdateien** (JSON), Bilder, Audio, Importdateien im Stil von Lottie / OCA / glTF, Mapping-Dateien und im Studio HTTP-Anfragen eines Browsers. Projektdateien sind im folgenden Modell *nicht vertrauenswürdige Daten*: Ein Projekt, das du von jemand anderem erhältst, kann feindlich sein, und sein Öffnen darf keinen Code ausführen, keine Dateien außerhalb seines Ordners lesen und die Maschine nicht erschöpfen.

| Akteur | Kann | Angenommenes Ziel | Wichtigste Abwehr |
|---|---|---|---|
| Autor eines feindlichen Projekts / einer feindlichen Importdatei | dir eine Datei geben | deine Dateien lesen, das Werkzeug abstürzen lassen oder aufhängen, Code ausführen, die Platte füllen | strenges Parsing, Grenzen für Größe und Tiefe, Verweigerung von Asset-Pfaden, keine Panics bei Eingaben, Sandbox für Expressions und Skripte, atomare Ausgabe |
| Webseite, die du besuchst, während das Studio auf localhost läuft | deinen Browser Anfragen an `127.0.0.1` senden lassen (CSRF, DNS-Rebinding) | dein Projekt bearbeiten, lesen, Dateien schreiben | Host-Allowlist, Prüfungen von Origin / `Sec-Fetch-Site`, JSON-Content-Type, Token mit Cookie `SameSite=Strict`, CSP |
| Anderer Benutzer auf derselben Maschine | sich mit dem Port verbinden, `/tmp` lesen | eine Sitzung oder temporäre Dateien lesen oder kapern | bindet standardmäßig an `127.0.0.1`, private temporäre Verzeichnisse (0700), Logdateien mit 0600, Token erforderlich, um anderswo zu binden |
| Jemand in deinem Netzwerk, wenn das Studio über localhost hinaus gebunden ist | den Port erreichen | das Token verwenden oder erraten, Verbindungen erschöpfen | verpflichtendes Token (16+ Zeichen), Vergleich in konstanter Zeit, Back-off, Obergrenzen für Verbindungen und Größe; **kein TLS** |
| Ein Mitarbeitender mit dem Token | die geteilte Sitzung bearbeiten | das Projekt beschädigen | Undo-Verlauf, das per Hash verkettete Log; Mitarbeitenden wird das Projekt anvertraut, es gibt keine Konten oder Rollen |
| Feindlicher Aufrufer der Bibliotheken | schlechte Argumente an die Fassade, die C- oder WASM-ABI übergeben | Speicherkorruption, Panics über die Grenze hinweg | Argumentvalidierung, Größenprüfungen vor der Allokation, abgefangene Panics (wo das Target unwindet) |

Die einzige ausgehende Anfrage, die OSATURA stellen kann, ist `osatura update-check`, und nur, wenn du sie ausführst (ein einziger anonymer HTTPS-GET der öffentlichen Versionsliste über das `curl` des Systems: nur HTTPS, TLS 1.2+, 30 s, 2 MB, der angezeigte Link wird darauf geprüft, auf `github.com/chiuta/osatura` zu liegen). Sie kann über Tor, Tor Browser, Cloudflare WARP oder deinen eigenen Proxy laufen; die Voreinstellungen sind lokale `socks5h://`-Endpunkte, DNS wird also nicht außerhalb des Proxys aufgelöst. Die Tests stehen in `update_cmd.rs`.

Nicht im Umfang: Schutz gegen einen böswilligen lokalen Administrator, eine kompromittierte Build-Toolchain oder Abhängigkeit (siehe [THIRD_PARTY.md](THIRD_PARTY.md) dazu, was ausgeliefert wird; `cargo deny` überwacht Lizenzen, Sperrlisten und Quellen, nicht Schwachstellen), Seitenkanäle und die Sicherheit von ffmpeg selbst.

## 2. HTTP-Server des Studios

`osatura studio` liefert den Editor aus einer eigenen kleinen HTTP/1.1-Implementierung aus (`osatura-cli/src/http.rs`, nur Standardbibliothek, kein Web-Framework von Drittanbietern). Das Verhalten, mit den Tests, die es festschreiben (Unit-Tests von `osatura-cli`, `tests/robust.rs`, `tests/wave4.rs`, `tests/source_offer.rs` und die Chromium-Smoke-Tests):

**Grenzen**

| Grenze | Wert | Ergebnis bei Überschreitung |
|---|---|---|
| Zeit zum Empfang des Request-Kopfes | 10 s | 408 |
| Zeit zum Empfang des Bodys | 20 s | 408 |
| Zeit zum Schreiben einer Antwort | 10 s | Verbindung wird getrennt |
| Größe des Request-Kopfes | 16 KB | 431 |
| Anzahl der Header | 100 | 431 |
| Größe des Bodys | 32 MB | 413 |
| Gleichzeitige Verbindungen | 128 | 503 |
| Chunked-Request-Bodys | nicht unterstützt | 411 |
| Doppelter `Host`-Header | abgelehnt | 400 |
| Steuerzeichen (CR, NUL, DEL…) in einem Header-Wert | abgelehnt (ein Tabulator ist erlaubt) | 400 |

Jede Antwort trägt `Connection: close` (kein Keep-Alive-Zustand, der sich erschöpfen ließe), `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` und eine `Content-Security-Policy`.

**Request-Richtlinie** (`Policy::check`, auf jede Anfrage angewendet):

1. **Host-Allowlist.** Der `Host`-Header muss die gebundene Adresse sein (`127.0.0.1`, `localhost`, der echte Port) oder ein mit `--allow-host` angegebener Name. Andernfalls 421 Misdirected Request. Das verhindert DNS-Rebinding: Eine Seite auf `evil.example`, die zu `127.0.0.1` aufgelöst wird, sendet `Host: evil.example`.
2. **Origin und `Sec-Fetch-Site`.** Anfragen mit einem `Origin` einer anderen Site oder mit `Sec-Fetch-Site: cross-site` / `same-site` werden mit 403 abgelehnt. `Origin: https://<allowed host>` wird für Hosts in `--allow-host` akzeptiert; hinter einem Reverse-Proxy, der die Seite unter einer öffentlichen URL ausliefert, übergib `--public-origin URL`, damit der Origin des Proxys akzeptiert wird. Cross-Origin bleibt abgelehnt.
3. **Content-Type.** `POST` verlangt `Content-Type: application/json` (sonst 415), sodass ein einfaches HTML-Formular nicht an die API senden kann.
4. **Token** (siehe Abschnitt 3), wenn konfiguriert; verpflichtend, wenn `--bind` keine Loopback-Adresse ist.

**Quellcode-Angebot.** Das Studio liefert `/source` aus (auch wenn ein Token erforderlich ist), und `--source-url` setzt die Adresse (die offiziellen Release-Binärdateien verwenden standardmäßig den Commit, aus dem sie gebaut wurden), wie es AGPL-3.0 Abschnitt 13 verlangt (siehe [LICENSING.md](LICENSING.md)).

**Unbekannte oder fehlerhafte Pfade und Methoden** erhalten eine einfache Fehlerantwort; die Editorseite ist in die Binärdatei einkompiliert (`studio.html`) und wird nicht aus dem Dateisystem gelesen, sodass es für statische Inhalte keine Angriffsfläche für Path Traversal gibt.

## 3. Authentifizierung und Zusammenarbeit

Die Option `--collab-token T` (oder das Token, das ein nicht lokales `--bind` verlangt) aktiviert die Zugriffskontrolle; `--collab-log LOG` macht eine geteilte Sitzung persistent.

* **Token-Format.** Mindestens 16 Zeichen aus `A-Za-z0-9._~+/=-`. Alles mit CR, LF, `;` oder anderen Zeichen wird beim Start abgelehnt (Header- und Cookie-Injection).
* **Übergabe.** Header `X-Collab-Token: T` oder das Cookie `osatura_token` (`HttpOnly`, `SameSite=Strict`, zusätzlich `Secure`, wenn die Anfrage unter einem `https://`-Namen von `--public-origin`, mit einem https-`Origin` oder mit `X-Forwarded-Proto: https` kam), gesetzt durch `POST /api/login`. Ein falsches Cookie zählt wie ein falscher Header als fehlgeschlagener Versuch.
* **Vergleich.** In konstanter Zeit (beide Seiten werden mit SHA-256 gehasht und die Digests verglichen), sodass die Antwortzeit keine Präfix-Treffer verrät.
* **Back-off.** Pro Client-IP-Adresse, ab dem **5.** aufeinanderfolgenden Fehlschlag: 2, 4, 8, 16, dann 30 Sekunden mit 429-Antworten; bis zu 512 Client-Adressen werden verfolgt. Das richtige Token wird immer akzeptiert, auch während eines Back-offs (damit ein Angreifer den Eigentümer nicht aussperren kann). Das Back-off bremst das Raten; einen verteilten Angriff stoppt es nicht. **Hinter einem Reverse-Proxy erscheint jeder Client mit der Adresse des Proxys**, sie teilen sich also einen Back-off-Zähler: Eine Flut schlechter Anfragen eines Besuchers verzögert den Fehlerzähler für alle, aber das richtige Token funktioniert weiterhin.
* **Geheimnisse der Teilnehmer.** Jeder Mitarbeitende erhält ein zufälliges 128-Bit-Geheimnis (`X-Collab-Secret`), das seinen Strom identifiziert; höchstens 32 Teilnehmer und 64 Ereignisströme.
* **Integrität des Logs.** Das Sitzungslog ist eine Textdatei, eine Zeile pro Operation: `<hash> <json>`, mit SHA-256 verkettet (Formatversion 2). Das Bearbeiten, Löschen oder Umordnen einer Zeile bricht die Kette, was beim Laden des Logs gemeldet wird. Jede Anfrage wird vor der Antwort per `fdatasync` gesichert, die Datei wird mit Modus 0600 angelegt und atomar neu geschrieben. Die Kette erkennt versehentliche Schäden und beiläufige Manipulation; sie ist **nicht signiert**, sodass jemand, der die Datei schreiben kann, eine gültige Kette neu aufbauen kann.
* **Tokens pro Teilnehmer (Welle 7, optional).** `--collab-secret-file F` ersetzt das eine gemeinsame Bearer-Token durch Tokens, die aus einem Geheimnis abgeleitet werden: `ap1.<name>.<expires>.<mac>` mit `mac` = HMAC-SHA-256 (RFC 2104, eigenes SHA-256, Testvektoren aus RFC 4231 in den Tests) über den Namen und die Frist. Der Host stellt sie mit `osatura collab-token NAME --secret-file F --ttl SECONDS` aus (Standard 8 Stunden, höchstens 30 Tage) und gibt jeder Person ihr eigenes. Ein Token ist an einen Namen gebunden (ein beitretender Teilnehmer erhält diesen Namen, gleich was die Seite sendet, sodass niemand als jemand anderes beitreten kann), läuft von selbst ab, wird durch Neuberechnung geprüft (keine Tabelle auf dem Server, nichts zu stehlen außer dem Geheimnis selbst), und sein Cookie erhält ein `Max-Age`, das der Restlebensdauer entspricht. `--collab-revoke NAMES` verweigert die Tokens dieser Namen vor ihrem Ablauf; das Ändern des Geheimnisses widerruft alles. Die Geheimnisdatei muss privat sein (Modus 600 unter Unix, sonst verweigert das Studio den Start), mindestens 16 Zeichen haben und wird nie als Kommandozeilenargument akzeptiert. `--collab-token` und `--collab-secret-file` lassen sich kombinieren; jedes von beiden erfüllt die Regel, dass ein nicht-Loopback-Bind eine Zugangsberechtigung braucht. Der Ablauf wird **pro Anfrage** geprüft: Ein bereits geöffneter Ereignisstrom bleibt offen, bis er endet, und die nächste Verbindung wird abgelehnt. Ein abgelaufenes oder widerrufenes Token wird mit `401` und dem Grund beantwortet und speist das Raten-Back-off nicht.
* **Ratenlimit (Welle 7).** `--collab-rate N` erlaubt N Anfragen pro Sekunde und Client-Adresse mit Bursts von 2N (Token Bucket); mehr wird mit `429` und `Retry-After` beantwortet. Der Standard ist 100 pro Sekunde, wenn eine Zugangsberechtigung erforderlich ist, sonst kein Limit; 0 schaltet es aus. Es wird nach der Authentifizierung angewendet und pro **Adresse** (Quelle einer Verbindung), nicht pro TCP-Verbindung, weil ein Limit pro Verbindung durch das Öffnen weiterer Verbindungen umgangen wird; hinter einem Reverse-Proxy teilen alle Clients die Adresse des Proxys, begrenze also auch am Proxy (Abschnitt 12).
* **Warnungen zur Bind-Adresse (Welle 7).** Ein Nicht-Loopback-`--bind` gibt beim Start eine Warnung aus (einfaches HTTP; bei `0.0.0.0` / `::`, dass das Studio auf jeder Schnittstelle lauscht; bei einem gemeinsamen Token, dass es keine Namen hat und nie abläuft).
* **Schreibweise des Tokens.** Die Frist in einem Token pro Teilnehmer hat genau eine zulässige Schreibweise (Dezimalziffern, ohne Vorzeichen, ohne führende Nullen, ohne `+`): Jede andere Schreibweise derselben Zahl wird als fehlerhaft abgelehnt, und die Signatur wird vor der Frist beurteilt, sodass ein gefälschtes Token aus der Antwort nichts lernt.
* **Hosts (optional).** `--collab-host NAMES` (erfordert `--collab-secret-file`) nennt die Teilnehmenden, die **speichern** (`/api/save`), das Sitzungslog **kompaktieren** (`/api/collab/compact`) und **widerrufen** dürfen (`/api/collab/revoke`, ein `POST`, das Namen zur Widerrufsliste hinzufügt, höchstens 1000). Alle anderen mit gültigem Token können weiterhin bearbeiten und zusehen und erhalten für diese drei Anfragen `403`. Ohne `--collab-host` ändert sich nichts: Jede Person mit gültiger Zugangsberechtigung darf alles davon, wie bisher. Das gemeinsame `--collab-token` ist an keinen Namen gebunden und gilt als Host.
* **Event-Stream.** Der Event-Stream erfordert eine Zugangsberechtigung (ein Token), aber kein Teilnehmer-Geheimnis; das Geheimnis identifiziert den eigenen Stream einer teilnehmenden Person, es ist kein zweites Passwort.
* **Verbindungen pro Adresse.** Eine Client-Adresse darf höchstens 16 Verbindungen gleichzeitig halten (zusätzlich zur globalen Obergrenze), damit eine einzelne Maschine nicht alle Plätze belegen kann. Loopback-Adressen werden standardmäßig nicht mitgezählt, weil ein Reverse-Proxy auf demselben Host das Studio für jeden Besucher von `127.0.0.1` aus erreicht; begrenze pro Besucher am Proxy (Abschnitt 12).
* **Kein TLS, keine Konten.** Der Server spricht einfaches HTTP. Ein Token, das über ein Netzwerk läuft, ist für jeden auf dem Pfad lesbar. Stelle das Studio hinter einen TLS-terminierenden Reverse-Proxy oder einen SSH- / VPN-Tunnel (Abschnitt 11). Es gibt keine Benutzerkonten oder Berechtigungen pro Projekt; die einzige Rolle ist die optionale Host-Liste oben, und jedes andere gültige Token gewährt Schreibzugriff.

## 4. Auflösung von Assets und Eingabegrenzen

* **Assets** (Bilddateien, Normal-Maps, Sounds, die ein Projekt nennt) werden über `assets::resolve` aufgelöst: relativ zum prozessweiten Asset-Verzeichnis (`assets::default_dir`, normalerweise der Ordner des Eingabeprojekts). **Absolute Pfade und jede `..`-Komponente werden abgelehnt**, es sei denn, das globale Flag `--allow-external-assets` ist gesetzt. Bilddateien sind auf 64 MB begrenzt, Audiodateien auf 256 MB.
* **Projektdateien.** `load_project` liest nur reguläre Dateien (keine Geräte, keine Pipes), bis zu 256 MB (`--max-input-bytes N` zum Ändern). `MAX_JSON_BYTES` der Fassade beträgt 256 MiB.
* **Abgeleitete Projekte.** Befehle, die ein abgeleitetes Projekt schreiben, validieren gegen das Verzeichnis des *Eingabe*projekts; liegt die Ausgabe woanders, werden relative Asset-Pfade vorhandener Dateien umgeschrieben, oder eine Warnung sagt, dass die Ausgabe sie nicht findet (`--allow-external-assets` macht sie absolut).
* **Rendering.** Die Fassade lehnt Bilder ab, die auf einer Seite größer als 16384 px sind (`MAX_RENDER_SIDE`), sowie Skalierungen außerhalb von 0.05 bis 4; die C- und wasm-Schichten lehnen Ausgaben über 256 MiB vor der Allokation ab. Weitere Grenzen der Fassade: fps in (0, 1000], Frames 1 bis 200000, Bühne bis 32768, 4096 Knochen, Beträge innerhalb von 1e7, `lod_levels` 1 bis 12, `fuzz`-Samples bis 100000, `trace`-Farben 2 bis 64 ([API.md](API.md)).
* **Keine Panics bei Eingaben.** Tests mit feindlichen Eingaben liegen in `osatura-cli/tests/robust.rs` und in den Wellentests; Tests im Fuzz-Stil decken die Parser ab. Ein Panic, der von einer Eingabe aus erreichbar ist, gilt als Schwachstelle.
* **Studio-Bodys** sind auf 32 MB begrenzt; Bearbeitungen durchlaufen dieselbe Validierung wie Dateien.

## 5. Sandbox für Expressions und Skripte

* **Expressions** (Wertebindungen, Constraints, Driver) werden von einem hauseigenen Parser geparst: **Quelltext bis 4096 Bytes, Verschachtelungstiefe 64**, Arithmetik, Vergleiche, die festen Variablen `frame`, `time`, `fps`, `frames`, `pi`, `t`, `get("layer","property")` für Driver und eine feste Tabelle eingebauter Funktionen (`sin`, `abs`, `min`, ...); keine Schleifen, keine I/O und keine benutzerdefinierten Funktionen, sodass die Auswertung immer terminiert. Nicht endliche Ergebnisse werden durch die Richtlinie für endliche Werte abgelehnt.
* **Skripte** (Befehl `script`, Feature `scripting`, standardmäßig aktiv) laufen in Rhai mit diesen Grenzen: **500.000 Operationen, 32 Aufrufebenen, Expression-Tiefe (64 auf oberster Ebene, 32 in Funktionen), Strings mit 10.000 Zeichen, Arrays mit 10.000 Elementen, Maps mit 1.000 Einträgen, 5.000 API-Anfragen, Skriptquelltext 64 KB** und `no_module` (kein `import`, also kein Dateizugriff über Module). Die den Skripten bereitgestellte Host-API ist ein fester Satz von Funktionen zum Bearbeiten von Projekten. Baue mit `--no-default-features`, um die Scripting-Engine vollständig zu entfernen (entfernt auch die Abhängigkeit `smartstring`).
* Ein Skript ist weiterhin Code, den du auf einem Projekt auszuführen gewählt hast: Es kann selbst keine Dateien lesen oder schreiben, kann aber das Projekt, das es bearbeitet, innerhalb der obigen Grenzen sehr groß machen, und eine Endlosschleife endet an der Operationsgrenze.

## 6. Dateien, Symlinks und temporäre Daten

* **Atomare Schreibvorgänge.** Ausgaben werden mit `fsutil::write_atomic` geschrieben: eine temporäre Datei im selben Verzeichnis (Name `.<name>.tmp-<pid>-<random>`), `fsync`, `rename`. Ein Absturz hinterlässt die alte oder die neue Datei. **Symbolische Links als Ausgabe werden abgelehnt**, mit Verweis auf `--follow-symlinks` (das deckt jetzt jede vom Benutzer adressierte Ausgabe ab: Berichte und Goldens von `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, Pose-Dateien, Snapshots, `vcs`, das Collab-Log und die von ffmpeg geschriebenen Videodateien, die vorab geprüft werden), sodass ein untergeschobener Link einen Schreibvorgang nicht umleiten kann. Ist das Verzeichnis nicht beschreibbar, die Datei aber schon, ist der Fallback ein Überschreiben an Ort und Stelle mit `fsync`, das **nicht atomar** ist (dokumentiert).
* **Private Daten.** Collab-Logs und andere Dateien, die nur der Eigentümer lesen soll, werden mit Modus 0600 angelegt (`write_atomic_private`). Temporäre Verzeichnisse werden exklusiv mit Modus 0700 angelegt (`private_temp_dir`), sodass ein vorhersagbarer Name in einem geteilten `/tmp` nicht von einem anderen Benutzer vorab angelegt werden kann. Zufallswerte (Tokens für neue Teilnehmer, temporäre Namen) kommen aus `/dev/urandom` (`fsutil::random_bytes`); der Seed-basierte deterministische Generator der Engine wird nie für Geheimnisse verwendet.
* **UI-Einstellungen.** Die `ui.json` des Studios ignoriert unbekannte Schlüssel mit einer Warnung und wird vor dem ersten Speichern nach `ui.json.bak` kopiert (dann `.bak.1`, ...).
* **Windows / nicht-Unix.** Die obigen Berechtigungsmodi gelten unter Unix. Unter Windows und macOS besteht die Testsuite in CI (POSIX-spezifische Prüfungen werden unter Windows übersprungen), aber die Prüfungen zu Symlinks, Berechtigungen und `/dev/urandom` laufen nur unter Linux und macOS, und niemand hat das Windows-Verhalten von Hand geprüft.
* **Unsignierte Binärdateien und die Fenster-App.** Die Release-Programme sind nicht codesigniert; prüfe Downloads gegen `SHA256SUMS.txt`. `osatura-studio` startet denselben lokalen Server (nur `127.0.0.1`, zufälliger freier Port) und zeigt ihn in der System-Webview (WebView2 / WKWebView); diese Webview ist so exponiert wie ein Browser-Tab auf dieser Adresse und wurde keiner eigenen Sicherheitsprüfung unterzogen.

## 7. Externe Programme (ffmpeg)

ffmpeg und ffprobe werden als getrennte Prozesse **nur** für den Videoexport, die mp3- / ogg-Dekodierung, das Mischen von Soundtracks und einige Werkzeuge gestartet; sie werden nie mitgeliefert ([LICENSING.md](LICENSING.md)). Jeder Aufruf:

* übergibt Argumente als Array (keine Shell, keine Zeichenkettenverkettung zu einer Kommandozeile);
* verwendet `-protocol_whitelist file` (eine interne Pipe-Verwendung ergänzt `pipe,fd`), sodass eine Mediendatei ffmpeg nicht dazu bringen kann, `http://`, `concat:` oder andere Netzwerk- / Sonderprotokolle zu öffnen;
* übergibt Dateinamen über `fsutil::file_arg`, das Namen, die mit `-` beginnen oder in der ersten Pfadkomponente `:` oder `|` enthalten, ein `./` voranstellt, sodass ein Dateiname nicht als Option oder URL gelesen werden kann;
* erzwingt bei Sounddateien `.wav`, `.mp3` und `.ogg` den Demuxer mit `-f` (`soundtrack::forced_demuxer`), sodass eine Datei, die nicht das ist, was ihr Name behauptet, abgelehnt statt als irgendein anderer Container geöffnet wird;
* verwendet `-loglevel error`.

ffmpeg selbst parst komplexe Medienformate und hatte eigene Schwachstellen: Halte es aktuell und füttere es auf einer Maschine mit Geheimnissen nicht mit Medien aus nicht vertrauenswürdigen Quellen. Befehle mit Zeitbudget (`export-fit --max-seconds`) können einen einzelnen laufenden ffmpeg-Aufruf nicht unterbrechen.

## 8. Bibliotheken (C-ABI, WASM, Fassade)

* Die **Fassade** validiert jedes Argument, gibt `ApiError` zurück und wandelt Panics in `ApiError::Internal` um, wo das Target unwindet. `Animation` und `Player` sind `Send + Sync` (zur Compile-Zeit geprüft).
* Die **C-ABI** ([C_ABI.md](C_ABI.md)) prüft Nullzeiger, Längen und UTF-8 und gibt Codes zurück; der Aufrufer muss weiterhin gültige Zeiger übergeben und die Pufferregeln einhalten. Der Last-Error-String gilt pro Thread. Die ABI wurde mit den dokumentierten Beispielen unter AddressSanitizer und UndefinedBehaviorSanitizer ausgeführt (sauber). Sie ist nicht gehärtet gegen einen Aufrufer, der die dokumentierten Verträge verletzt (doppeltes Schließen, Verwendung nach dem Schließen, falsche Längen).
* Das **WASM-Modul** ([WASM.md](WASM.md)) wird in CI gebaut und besteht den Test des Wrappers unter Node, und wurde einmal in Headless-Chromium ausgeführt (fehlerhafte Eingaben ergeben Fehler, kein Leck über 60 Öffnen-/Schließen-Zyklen, überdimensionierte Bühnen werden abgelehnt), **wurde aber nicht als Browser-Komponente geprüft und nicht in Firefox oder Safari ausgeführt**, seine Sicherheitseigenschaften sind also nur vom Lesen des Codes und diesem Test her bekannt. Es hat konstruktionsbedingt keinen Dateisystem- oder Netzwerkzugriff; ein Panic lässt die Instanz trappen.
* Keine der beiden Schichten sandboxt das *Projekt*: Das Rendern eines feindlichen Projekts ist durch die obigen Grenzen beschränkt, aber ein Projekt kann innerhalb dieser dennoch viel Arbeit anfordern (viele Ebenen, große Bilder, lange Timelines).

## 9. Bekannte Lücken

Ehrliche Liste; für die weitere Liste funktionaler Einschränkungen siehe [LIMITATIONS.md](LIMITATIONS.md).

1. **Kein TLS und keine Konten** im Studio. Seit Welle 7 kann das Token pro Teilnehmer und befristet sein (Abschnitt 3), was dennoch kein Kontensystem ist: Es gibt keine Rollen, kein Zurücksetzen von Passwörtern und kein eigenes TLS. Natives TLS wurde nicht hinzugefügt, weil kein lizenzkompatibles TLS-Crate in `Cargo.lock` steht (eines hinzuzufügen würde eine große neue Abhängigkeit und kryptografischen Code ohne Audit in ein AGPL-/MPL-Projekt bringen); Abschnitt 12 zeigt den unterstützten Weg, einen TLS-terminierenden Reverse-Proxy.
2. **Das Back-off gilt pro Adresse** und wird hinter einem Proxy geteilt; es bremst Online-Raten, verhindert es aber nicht. Verwende ein langes zufälliges Token (das Minimum von 16 Zeichen ist eine Untergrenze, keine Empfehlung).
3. **Das Collab-Log ist verkettet, nicht signiert.**
4. **Keine auditierte Codebasis.** Kein Penetrationstest durch Dritte und keine formale Fuzzing-Kampagne; die Prüfungen sind die eigenen Tests und Review-Wellen des Projekts.
5. **Abhängigkeiten.** CI führt `cargo deny` aus für Advisories (Schwachstellen, zurückgezogene Crates, nicht mehr gepflegte direkte Abhängigkeiten), Lizenzen, gesperrte Crates (kein TLS- oder HTTP-Client-Stack) und Quellen. Hinweise zu transitiver Wartung werden von Hand geprüft (siehe `deny.toml`). Die Prüfung kennt nur die bisher veröffentlichten Advisories; führe sie vor dem Deployment erneut aus.
6. **Das Asset-Verzeichnis ist prozessweit.** `merge` über mehrere Eingabeverzeichnisse verwendet für Assets das letzte Verzeichnis; nur `write_json` schreibt Asset-Pfade um (andere Schreiber nicht).
7. **Nicht atomarer Fallback**, wenn das Ausgabeverzeichnis nicht beschreibbar ist.
8. **Pfadprüfungen nutzen kanonische Pfade.** `assets::resolve` kanonisiert vorhandene Dateien und lehnt eine Datei ab, deren tatsächlicher Ort außerhalb des Projektverzeichnisses liegt; ein symbolischer Link in einem Projektordner, der nach außen zeigt, wird also abgelehnt (getestet in `assets.rs`); Links, die im Ordner bleiben, funktionieren. Eine Race Condition zwischen der Prüfung und dem späteren Lesen (Time-of-check to time-of-use) durch jemanden, der den Projektordner gleichzeitig verändern kann, ist nicht ausgeschlossen.
9. **ffmpeg** liegt außerhalb unserer Kontrolle (Abschnitt 7).
10. **WASM** ist in einer Browser-Engine ungetestet (es läuft in CI nur unter Node); das Browser-Bedrohungsmodell einer Seite, die es einbettet (CSP, Speichergrenzen), wurde nicht untersucht.
11. **Denial of Service über die Größe.** Die Grenzen beschränken einzelne Anfragen und Dateien, nicht die Gesamtarbeit: Ein gültiges, aber sehr schweres Projekt kann eine CPU lange beschäftigen (die obige Verbindungsobergrenze begrenzt die Parallelität, nicht die Kosten einer Anfrage).

## 10. Eine Schwachstelle melden

* **Eröffne kein öffentliches Issue** für eine Schwachstelle. Sende eine private Meldung an den Maintainer: **`alexio@trom.tf`** (die Adresse im Hinweis des Programms bei `--version` und in den Lizenzdokumenten), oder nutze die private Schwachstellenmeldung von GitHub auf dem Reiter *Security* des Repositorys, sofern sie aktiviert ist.
* Gib an: Version (`osatura --version`) und Plattform, das Projekt oder die Anfrage, die das Problem auslöst (eine minimale Datei ist am besten), was du erwartet hast und was passiert ist, und ob du namentlich genannt werden möchtest.
* Was dich erwartet *(Absicht, kein Vertrag: Dies ist ein Ein-Personen-Projekt)*: eine Bestätigung innerhalb angemessener Zeit, ein im Privaten entwickelter Fix, ein der Suite hinzugefügter Regressionstest, ein Vermerk in [CHANGELOG.md](CHANGELOG.md) und auf Wunsch eine Namensnennung. Bitte lass Zeit für einen Fix, bevor du Details veröffentlichst.
* Unterstützte Versionen: Nur die neueste Vorabversion (derzeit `v0.1.0-alpha.4`) und der aktuelle Baum auf `main` werden unterstützt.
* Meldungen zur Netzwerk-Exposition des Studios, wenn es absichtlich ohne Reverse-Proxy an eine öffentliche Adresse gebunden wird, sind zunächst Dokumentationsthemen: siehe Abschnitt 11.

## 11. Härtungs-Checkliste für Betreiber

Für alle, die `osatura studio` für mehr als eine Person oder außerhalb von localhost betreiben.

1. [ ] Bevorzuge, es **nicht** zu exponieren. Einzelner Benutzer: Lass den Standard-Bind (`127.0.0.1`) und nutze einen SSH-Tunnel (`ssh -L 8080:127.0.0.1:8080 host`).
2. [ ] Wenn exponiert, **terminiere TLS** in einem davorgeschalteten Reverse-Proxy (nginx, Caddy, ...); exponiere den einfachen HTTP-Port nie im Internet.
3. [ ] Verwende ein **langes zufälliges Token** (zum Beispiel 32+ Zeichen aus `openssl rand -base64 24`), übergib es über eine geschützte Datei oder Umgebung des Dienstes statt über die Shell-History und rotiere es, wenn Personen gehen. Denk daran, dass es vollen Schreibzugriff bedeutet.
4. [ ] Starte mit `--bind 127.0.0.1` (oder der privaten Schnittstelle) plus dem Proxy; setze `--allow-host your.domain` und `--public-origin https://your.domain`, damit die Prüfungen von Host und Origin für den Proxy und für nichts sonst bestehen.
5. [ ] Bevorzuge **Tokens pro Teilnehmer** (`--collab-secret-file`, Abschnitt 3) mit kurzem `--ttl` gegenüber einem gemeinsamen Token; halte die Geheimnisdatei mit Modus 600 außerhalb des ausgelieferten Ordners, und benenne mit `--collab-host` die Personen, die speichern, kompaktieren und widerrufen dürfen.
6. [ ] Lass den Proxy **Ratenlimits** setzen und, wenn möglich, nach IP einschränken oder eine zweite Authentifizierungsschicht ergänzen (HTTP Basic über TLS, VPN, SSO-Proxy), denn das eingebaute Back-off sieht nur die Adresse des Proxys.
7. [ ] Laufe als **unprivilegierter Benutzer** in einem dedizierten Verzeichnis, das nur die auszuliefernden Projekte enthält; keine Geheimnisse, keine sensiblen Symlinks. Nutze anderswo ein schreibgeschütztes Dateisystem, einen Container oder systemd-Sandboxing (`ProtectSystem`, `PrivateTmp`, `NoNewPrivileges`).
8. [ ] Halte `--allow-external-assets` und `--follow-symlinks` **aus**.
9. [ ] Setze `--source-url` auf den Ort, an dem der (möglicherweise modifizierte) Quellcode abgerufen werden kann (AGPL Abschnitt 13).
10. [ ] Persistiere mit `--collab-log` in einem privaten Verzeichnis, sichere es und behalte den Modus 0600. Behandle das Log als sensibel: Es enthält den gesamten Bearbeitungsverlauf.
11. [ ] Begrenze Ressourcen auf Betriebssystemebene (Speicher, CPU-Zeit, offene Dateien, Plattenquota); die eingebauten Obergrenzen beschränken einzelne Anfragen, nicht die Gesamtlast.
12. [ ] Halte ffmpeg und die für den Build verwendete Rust-Toolchain aktuell; führe `cargo deny check` (oder `cargo audit`) auf `Cargo.lock` für dein Release aus.
13. [ ] Füttere den Server nicht mit Projekten aus nicht vertrauenswürdigen Quellen, sofern er nicht in der obigen Sandbox läuft. Für Deployments ohne Scripting baue mit `--no-default-features`.
14. [ ] Beobachte die Logs auf Antworten 421, 403, 429 und 503; Spitzen bedeuten Abtasten oder Fehlkonfiguration.

## 12. Reverse-Proxy mit TLS: nginx und Caddy

Das Studio spricht einfaches HTTP. Für alles jenseits deiner eigenen Maschine lass es auf Loopback laufen und einen Proxy TLS terminieren. Diese Rezepte wurden nach der öffentlichen Dokumentation der Proxys und dem oben beschriebenen Verhalten des Studios geschrieben; **sie wurden in der Entwicklungsumgebung nicht ausgeführt** (kein nginx oder Caddy, keine Zertifikate, kein Netzwerk): Teste sie, bevor du dich darauf verlässt.

Starte das Studio auf Loopback und nenne seinen öffentlichen Host, damit die Prüfungen von Host und Origin den Proxy akzeptieren und nichts sonst:

```
osatura studio demo.json --port 8080 --bind 127.0.0.1 \
  --allow-host studio.example.org --public-origin https://studio.example.org \
  --collab-secret-file /etc/osatura/collab.secret --collab-log /var/lib/osatura/session.log
```

Mit einem `https://`-`--public-origin` erhält das Login-Cookie das Attribut `Secure`. Der Ereignisstrom (`/api/collab/events`) sind Server-Sent Events: Der Proxy darf ihn nicht puffern und muss lange Lesevorgänge erlauben.

**nginx** (Zertifikate von deiner CA oder `certbot`):

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

**Caddy** (besorgt und erneuert das Zertifikat selbst; `Caddyfile`):

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

`flush_interval -1` schickt den Ereignisstrom ungepuffert durch. Caddy setzt `X-Forwarded-Proto` und reicht den ursprünglichen `Host` durch. Für ein Ratenlimit der Anfragen nutze das Plugin `caddy-ratelimit` oder schalte das `limit_req` von nginx davor.

Prüfungen nach dem Start: `curl -I https://studio.example.org/` antwortet mit `401` (Login-Seite) und liefert das Studio nie ohne Token aus; `curl -I http://127.0.0.1:8080/ -H 'Host: evil.example'` antwortet mit `421`; das von `/api/login` gesetzte Cookie trägt `Secure; HttpOnly; SameSite=Strict`; das eigene Log des Studios zur Sitzung bleibt privat (Abschnitt 6). Läuft der Proxy auf einer anderen Maschine, binde das Studio an die private Schnittstelle, die der Proxy erreichen kann, sperre den Port per Firewall und denk daran, dass das Studio selbst auf dieser Verbindung weiterhin einfaches HTTP spricht.
