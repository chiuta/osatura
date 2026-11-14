<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Datenschutz

Dieses Programm überträgt keine Informationen an andere vernetzte Systeme, es sei denn, der Nutzer oder die Person, die es installiert oder betreibt, verlangt dies ausdrücklich.

Im Einzelnen gilt für `osatura`, `osatura-studio`, die C-Bibliothek und das WebAssembly-Modul:

* **Keine Telemetrie, Analyse, Absturzberichte, Werbung oder automatischen Update-Prüfungen.** Die Programme enthalten keinen Code, der von sich aus einen Server kontaktiert.
* **Ein einziger Befehl kontaktiert einen Server, nur wenn du ihn eingibst.** `osatura update-check` sendet eine einzige anonyme HTTPS-Anfrage an GitHub (`api.github.com`) für die öffentliche Liste der Releases, mithilfe des Programms `curl` deines Systems; GitHub sieht deine IP-Adresse und diese Anfrage, gemäß der [Datenschutzerklärung von GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). Es wird nichts heruntergeladen oder installiert, und es wird nichts über dich oder deine Projekte gesendet. Damit GitHub deine Adresse nicht sieht, füge `--proxy-preset tor` (oder `tor-browser`, `warp`) oder `--proxy-url` mit deinem eigenen Proxy hinzu; siehe [CLI_REFERENCE.md](CLI_REFERENCE.md).
* **Das Studio ist ein lokaler Server.** `osatura studio` und `osatura-studio` lauschen standardmäßig auf `127.0.0.1` (nur dieser Computer). Von anderen Rechnern sind sie nur erreichbar, wenn du sie selbst mit `--bind` auf einer anderen Adresse startest ([SECURITY.md](SECURITY.md)). Die ausgelieferte Seite lädt nichts aus dem Internet (ihre Content-Security-Policy erlaubt nur den lokalen Server).
* **Links, die du anklickst.** Das Studio und die Dokumentation enthalten Links (zum Beispiel zu GitHub oder zu einer Spendenseite). Wer einem folgt, öffnet diese Website in seinem Browser, unter deren Datenschutzerklärung.
* **Externe Programme.** `ffmpeg` wird nur für Video und einige Audioformate gestartet, nur wenn du einen solchen Export verlangst, und arbeitet mit lokalen Dateien.
* **Geteilte Sitzungen führen einen Verlauf.** Mit `--collab-log` schreibt das Studio jede Änderung der Sitzung samt den Namen der Teilnehmer in eine Datei deiner Wahl; bei Tokens pro Teilnehmer sind die Namen diejenigen, die du erzeugt hast. Behandle diese Datei als personenbezogene Daten der Teilnehmer und lösche sie nach Ende der Sitzung ([SECURITY.md](SECURITY.md), [UNINSTALL.md](../../UNINSTALL.md)). Die Werkzeuge für Gesichts-Landmarken schreiben ihre Ergebnisse nur in die Dateien, die du angibst.
* **Deine Dateien bleiben deine.** Projekte, Renderings und Einstellungen werden nur dorthin geschrieben, wo du es dem Programm sagst, zuzüglich der kleinen Einstellungsdatei, die in [UNINSTALL.md](../../UNINSTALL.md) aufgeführt ist.

Die Website des Projekts (<https://osatura.chiuta.ro>) ist eine statische Seite, die bei GitHub Pages gehostet wird; dafür gilt die eigene Datenschutzerklärung von GitHub.

Fragen: <alexio@trom.tf>.
