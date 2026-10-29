<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Release-Checkliste (Seite des Inhabers)

*Schritte, die nur der Inhaber ausführen kann, in dieser Reihenfolge. Stand 24. Okt. 2026: Das Repository <https://github.com/chiuta/osatura> existiert, GitHub Actions ist grün (Tests unter Linux, macOS und Windows, clippy, wasm32, cargo-deny, REUSE), und die Entwurfs-Vorabversion `v0.1.0-alpha.3` wurde gebaut (Archive für Windows, macOS und Linux, das Web-Modul und `SHA256SUMS.txt`); **noch nichts ist veröffentlicht**. Die Schritte unten, die CI schon erledigt hat, sind abgehakt; die übrigen brauchen eine Person, einen Anwalt oder ein Konto. Keine Rechtsberatung. Mit **[blocker]** markierte Punkte sollten vor einer öffentlichen Ankündigung erledigt sein.*

## 1. Bevor irgendetwas veröffentlicht wird

- [ ] **[blocker]** Den Sicherheitskontakt bestätigen oder ersetzen (`alexio@trom.tf` ist ein Platzhalter in `docs/SECURITY.md`, in `--version` und in den Lizenzdokumenten).
- [ ] **[blocker]** Den Namen „OSATURA" in den Markenregistern recherchieren, die dir wichtig sind (EUIPO, USPTO, OSIM), sowie in App-Stores / Paketregistern. Siehe `TRADEMARK.md`. Bei einer Kollision vor dem ersten Release umbenennen (der Name steckt im Binary, in Crates und Dokumentation).
- [ ] **[blocker]** Einen Anwalt bitten, `LICENSING.md`, die Aufteilung MPL-2.0 / AGPL-3.0, `docs/THIRD_PARTY.md` und `docs/PATENT_NOTES.md` zu prüfen. Entscheiden, ob du Dual-Lizenzierung willst; wenn ja, wird ein CLA *vor* der Annahme externer Beiträge benötigt (heute: nur DCO).
- [ ] Die Bedingungen der Graphics-Gems-Referenz zur Kurvenanpassung prüfen (in `CLEANROOM.md` aus dem Gedächtnis zitiert); die Formulierung zur unabhängigen Implementierung nur beibehalten, wenn sie zutrifft.
- [x] `reuse lint` und `cargo deny check` bestehen in CI (der Job `licences`); führe sie einmal auch auf deinem Rechner aus.
- [ ] Prüfen, dass im Baum keine personenbezogenen Daten, lokalen Pfade oder Tokens liegen (`grep -r "/home/" .`, `grep -ri token`) und dass das mitgelieferte Modell und die Demo-Assets alle selbst erzeugt sind.

## 2. Repository

- [x] Das GitHub-Repository existiert, und der Baum wurde ohne `target/` und `out/` gepusht (`.gitignore`).
- [ ] `COPYRIGHT`, `LICENSES/`, `REUSE.toml`, `.github/FUNDING.yml` behalten (Buy Me a Coffee `echo.of.the.strings`, Patreon `alexio_tf`).
- [ ] Privates Melden von Sicherheitslücken aktivieren (Reiter Security) und `docs/SECURITY.md` darauf verweisen lassen.
- [ ] Beim Start des Studios die Repository-URL mit `--source-url` übergeben (AGPL-Quellcode-Angebot, Route `/source`) und das Angebot an einer frischen Installation testen.

## 3. Erster CI-Lauf (erledigt am 2026-10-24)

- [x] `.github/workflows/ci.yml` ist grün: Tests unter Linux, Windows, macOS und `clippy -D warnings`; der wasm32-Build-Job; der Job für cargo-deny und REUSE.
- [x] Die Fehler des ersten Laufs unter Windows wurden behoben (Stack-Größe, nur unter POSIX lauffähige Tests, Zugriff auf das Kollaborationslog) und in `docs/LIMITATIONS.md` (L-04) vermerkt.
- [x] Der wasm32-Job besteht (Build und Node-Test mit dem echten Modul). **Offen:** die Verifikationsliste in `docs/WASM.md` in einem echten Browser durchgehen und L-01 / L-02 aktualisieren.

## 4. Tests mit Menschen

- [ ] Das Studio 3 bis 5 Animatoren eine Woche lang mit einer kurzen Aufgabe geben (eine Figur riggen, einen Walk Cycle erstellen, ein GIF exportieren). Festhalten, wo sie stecken bleiben; jeden Fall als Issue anlegen.
- [ ] Auf echten Browsern (Chrome, Firefox, Safari), einem Touch-Gerät und in einer Sitzung nur mit Tastatur testen.
- [ ] Auf einem echten **Windows 11** und einem echten **Mac**: das Archiv entpacken, `osatura.exe` (Windows) und `osatura-studio` doppelklicken, ein Projekt riggen und exportieren und berichten, was fehlschlägt (L-74 bis L-76).
- [ ] Gesichtsdetektor / Mocap / Sprachfunktionen mit echten Daten ausprobieren; ehrlich berichten, dass sie begrenzt sind (L-12, L-13).

## 5. Sicherheit beim Betrieb (wenn du das Studio oder die Zusammenarbeit hostest)

- [ ] Hinter einen Reverse-Proxy mit TLS-Terminierung stellen (Rezept: `docs/SECURITY.md` Abschnitt 12); den reinen HTTP-Port niemals ins Internet exponieren.
- [ ] Tokens pro Teilnehmer verwenden (`collab-token`), ein Ablaufdatum setzen, hinter dem Proxy an localhost binden.
- [ ] Eine unabhängige Sicherheitsprüfung in Auftrag geben, bevor gehostete Zusammenarbeit Fremden angeboten wird.

## 6. Release-Artefakte

- [x] Die Release-Binärdateien werden auf von GitHub gehosteten Runnern gebaut (`release.yml`), und `SHA256SUMS.txt` wird im Entwurf erzeugt. **Offen:** den Entwurf `v0.1.0-alpha.3` prüfen, „pre-release“ ankreuzen, ihn veröffentlichen und den Commit taggen (der Tag wird mit dem Release angelegt).
- [x] Provenienz-Signierung: schlüssellose Sigstore-Signaturen und GitHub-Build-Provenienz-Attestierungen für jede Release-Datei (Job `sign` in `release.yml`, in CI geprüft).
- [x] Voraussetzungen der SignPath Foundation im Repository: CODE_SIGNING_POLICY.md (vom README, von der Website und von jedem Release verlinkt), PRIVACY.md, docs/UNINSTALL.md, keine Dual-Lizenzierung (LICENSING.md), Windows-Dateimetadaten (`build.rs`, in release.yml geprüft), Artefaktkonfiguration in `.signpath/`, Job `windows-authenticode`.
- [ ] Inhaber: Zwei-Faktor-Authentifizierung auf GitHub aktivieren (von SignPath für jedes Teammitglied verlangt).
- [ ] Inhaber: ein Release veröffentlichen (SignPath verlangt, dass das Projekt in der Form veröffentlicht ist, die signiert wird), dann unter https://signpath.org/apply den Antrag stellen.
- [ ] Nach der Annahme: in SignPath das Projekt `osatura`, die Signing Policy `release-signing` und die Artefaktkonfiguration `windows-zip` anlegen (`.signpath/artifact-configuration.xml` einfügen), das GitHub-Repository als vertrauenswürdiges Build-System verknüpfen; in GitHub die Repository-Variable `SIGNPATH_ORGANIZATION_ID` und das Secret `SIGNPATH_API_TOKEN` ergänzen. Das nächste Release wird dann nach manueller Freigabe signiert.
- [ ] macOS: Apple Developer ID und Notarisierung (noch nicht geplant); bis dahin warnt Gatekeeper (L-74).
- [ ] Den passenden Quellcode für die AGPL-Teile beilegen (der Tag selbst genügt; ihn verfügbar halten, solange du Binaries verteilst).
- [x] `docs/THIRD_PARTY_NOTICES.md` ist neu generiert (24. Okt. 2026), und jedes Archiv enthält es; vor jedem späteren Release mit `scripts/licence-audit.sh` erneut generieren.
- [ ] Crates.io / npm / Paketregister: prüfen, dass die Namen frei sind und die Lizenzfelder zu `LICENSING.md` passen.

## 7. Ankündigung und Spenden

- [ ] Erster Bildschirm des README: was funktioniert, was nicht (auf `docs/LIMITATIONS.md` verlinken); nicht ohne Einschränkung „Moho-Klasse" behaupten.
- [ ] Die Screenshots aus `docs/img/` und ein kurzes GIF zeigen, das mit dem Demo-Projekt erstellt wurde.
- [ ] Die Spendenlinks (Buy Me a Coffee, Patreon) im README und im About-Fenster des Studios ergänzen; keine Funktionen als Gegenleistung für Spenden versprechen, es sei denn, du kannst das Versprechen halten.
- [ ] Dort posten, wo Animatoren sind (Communities für 2D-Rigging / Indie-Animation); um Fehlerberichte bitten, nicht um Lob.
- [ ] Zeit einplanen, um die ersten Issues schnell zu beantworten; die ersten zwei Wochen entscheiden, ob Mitwirkende bleiben.
