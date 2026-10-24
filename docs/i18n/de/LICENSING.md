# OSATURA — Lizenzierung (ENTSCHIEDEN: Variante A, angewendet am 22. Okt. 2026; siehe LICENSE und LICENSES/)

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/README.md) · [Română](https://github.com/chiuta/osatura/blob/main/README.ro.md) · [Français](https://github.com/chiuta/osatura/blob/main/README.fr.md) · [Italiano](https://github.com/chiuta/osatura/blob/main/README.it.md) · [Español](https://github.com/chiuta/osatura/blob/main/README.es.md) · [Português](https://github.com/chiuta/osatura/blob/main/README.pt.md) · [Deutsch](https://github.com/chiuta/osatura/blob/main/README.de.md) · [Magyar](https://github.com/chiuta/osatura/blob/main/README.hu.md)

**Status:** Die unten beschriebene Variante A wurde gewählt und angewendet: MPL-2.0 für die Bibliotheken (core, capi, wasm, gpu), AGPL-3.0-or-later für `osatura-cli` (CLI + Studio) und `osatura-gui` (das Studio in einem nativen Fenster), CC-BY-4.0 für Dokumentation und synthetische Assets/Modelle, Markenrichtlinie in TRADEMARK.md, DCO in CONTRIBUTING.md, SPDX-Header in allen .rs/.js/.mjs/.sh-Dateien und REUSE.toml für den Rest, Quellcode-Angebot nach AGPL Abschnitt 13 (`/source`, `--source-url`), Cargo-Felder `license` gesetzt. Verbleibende Aufgaben des Inhabers: Prüfung durch einen Anwalt, Markenrecherche (EUIPO/OSIM), Patent-Screening (docs/PATENT_NOTES.md), ein CLA, falls Dual-Lizenzierung gewünscht ist, Hinweise zu Drittanbieter-Lizenzen den Binär-Releases beilegen.


*Keine Rechtsberatung. Die Lizenz ist ENTSCHIEDEN (Variante A, siehe LICENSE); die folgenden Abschnitte bleiben als Begründung und als Dokumentation der erwogenen Alternativen erhalten. Lass dies vor dem ersten öffentlichen Release von einem Anwalt bestätigen (idealerweise von jemandem, der das rumänische und EU-Urheberrecht kennt). Beiträge: Solange kein CLA vorliegt, besteht kein Recht, Beiträge Dritter umzulizenzieren oder dual zu lizenzieren (CONTRIBUTING.md).*

## 1. Was die Lizenz für dich leisten muss
Dein Einkommen sind **Spenden (Buy Me a Coffee)**, keine Lizenzgebühren. Die Lizenz erzeugt also nicht das Geld — das tun **Wohlwollen, Sichtbarkeit und Vertrauen**. Sie hat drei Aufgaben:
1. **Möglichst viele Nutzer gewinnen** (mehr Nutzer → mehr Unterstützer): Sie muss frei nutzbar sein, auch in kommerzieller Arbeit.
2. **Verhindern, dass eine Firma den Code nimmt, schließt und verkauft** (oder als kostenpflichtigen Dienst hostet), ohne etwas zurückzugeben.
3. **Dein Recht behalten, Ausnahmen zu verkaufen** (kommerzielle Lizenz, Support) und den Namen zu verteidigen.

Sie muss außerdem zu deinen **Trade-free-/TROM-Überzeugungen** passen: Die Software soll nicht beschnitten, werbefinanziert oder an Datensammlung gekoppelt sein. (Lizenzen, die kommerzielle Nutzung verbieten, z. B. CC-NC oder Polyform-NC, sind *nicht* Open Source und nicht dasselbe wie „trade-free"; prüfe den genauen Wortlaut von TROM, bevor du eine davon übernimmst.)

## 2. Erkenntnisse, die die Wahl einschränken
- **Abhängigkeiten sind kein Hindernis.** Audit aller Drittanbieter-Crates (363 bei der Neugenerierung am 24. Okt. 2026 durch `scripts/licence-audit.sh`, über alle Plattformen zusammengezählt, nachdem die Fenster-App hinzugekommen war; am 22. Okt. waren es 160; die Zahl ändert sich, siehe docs/THIRD_PARTY.md): Alle stehen unter MIT, Apache-2.0, BSD, Zlib, ISC, Unlicense, CC0, Unicode oder Dual-Lizenzen daraus, mit Ausnahme von sechs MPL-2.0-Crates mit schwachem Copyleft auf Dateiebene: smartstring (MPL-2.0+, nur über das Feature `scripting` eingebunden, das in `osatura-core` und `osatura-cli` standardmäßig AKTIV ist) sowie `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors` und `option-ext` (nur über die Fenster-App `osatura-gui` eingebunden); ein Build der Bibliotheken mit `--no-default-features` enthält kein MPL-Drittanbieter-Crate. Sie lassen sich mit AGPL, MPL oder einer proprietären Lizenz kombinieren; beim MPL-Crate müssen Quellcode bzw. Hinweis erhalten bleiben.
- **Urheberrechtliche Inhaberschaft.** Ein großer Teil des Codes wurde mit Claude geschrieben. Das Urheberrecht knüpft im Allgemeinen an den *menschlichen* schöpferischen Beitrag an (Richtlinien des US Copyright Office; auch EU-/rumänisches Recht verlangen die eigene geistige Schöpfung eines menschlichen Urhebers). Praktische Folgen: (a) Du kannst und solltest das Projekt als menschlicher Urheber/Bearbeiter lizenzieren, aber (b) versprich keine Exklusivität für jede einzelne Zeile, (c) bewahre deine eigenen Designnotizen, Auswahlentscheidungen und Änderungen im Repository als Nachweis der Urheberschaft auf, (d) ein DCO bescheinigt nur die Herkunft; um Beiträge später umzulizenzieren (z. B. bei Dual-Lizenzierung), braucht es einen CLA oder eine ausdrückliche Rechteeinräumung nach außen, und diese liegt NICHT vor.
- **Keine Lizenz kann Spenden erzwingen.** Schränke Code niemals hinter einer Spende ein; ein Fork würde die Sperre einfach entfernen. Verkaufe *Komfort und Extras*, nicht die Erlaubnis.

## 3. Empfohlenes Paket — „Offene Kernbibliothek, geschützte Anwendung" (alle Varianten unten sind sofort anwendbar)

### ★ Variante A (empfohlen): MPL-2.0-Bibliothek + AGPL-3.0-Anwendungen + Marke
| Teil | Lizenz | Warum |
|---|---|---|
| `osatura-core` (Engine, Evaluator, Formate, Exporter) und jede künftige Laufzeitumgebung/WASM/C-ABI | **MPL-2.0** | Copyleft auf Dateiebene: Jeder darf sie in Spiele/Anwendungen einbetten (geschlossen oder offen), aber Änderungen an *unseren Dateien* müssen geteilt werden. Enthält eine ausdrückliche **Patentlizenz** und Beendigung bei Patentklage. Maximiert die Verbreitung, die für Spenden nötig ist. |
| `osatura-cli`, `osatura-gui`, Studio (Browser/Fenster) | **AGPL-3.0-or-later** | Verhindert geschlossene Forks und „OSATURA als kostenpflichtiger Webdienst"-Klone; die *Projekte und Renderings* der Nutzer bleiben vollständig deren Eigentum. |
| Name „OSATURA", Logo | **Markenrichtlinie** (separate Datei) | Forks müssen umbenannt werden; schützt Unterstützer vor Nachahmungen. Prüfe die Verfügbarkeit (EUIPO/OSIM/WIPO) vor dem Start. |
| Beispiel-Rigs, Grafiken, Audio, die du erstellst | **CC BY 4.0** (oder CC BY-SA) | Frei wiederverwendbar unter Namensnennung; Grafiken Dritter behalten ihre eigene Lizenz. |
| Dokumentation | **CC BY 4.0** | |
<!-- REUSE-IgnoreStart -->
Cargo: `license = "MPL-2.0"` (core), `license = "AGPL-3.0-or-later"` (cli). SPDX-Header pro Datei: `// SPDX-License-Identifier: MPL-2.0`.
<!-- REUSE-IgnoreEnd -->
**Dual-Lizenzierung:** nur für Code möglich, den der Inhaber allein geschrieben hat, oder nach einem CLA/einer Rechteeinräumung durch jeden Mitwirkenden. Für beigetragenen Code heute NICHT möglich (CONTRIBUTING.md).

### Variante B: AGPL-3.0-or-later überall (+ kommerzielle Lizenz auf Anfrage)
Stärkster Schutz gegen Übernahme und am nächsten am Trade-free-Ethos. Preis: Firmen und Spiel-Engines werden das Einbetten der Laufzeitumgebung meiden, also weniger Nutzer und weniger indirekte Unterstützer. Wähle sie, wenn Schutz wichtiger ist als Reichweite.

### Variante C: Apache-2.0 überall (nicht gewählt; der Workspace wendet Variante A an: MPL-2.0-Bibliotheken und `license = "AGPL-3.0-or-later"` für `osatura-cli`)
Maximale Reichweite und keinerlei Reibung; Patentlizenz inbegriffen. Preis: Jeder — auch ein Großkonzern — darf sie schließen und verkaufen, ohne Verpflichtung. Spenden hängen dann allein vom Wohlwollen ab. Wähle sie nur, wenn du eine öffentliche Infrastruktur sein willst.

### Variante D: Source-available (BUSL-1.1 oder Polyform Shield/NC mit verzögertem Öffnungsdatum)
Verhindert N Jahre lang kommerzielle Konkurrenz. Preis: **nicht Open Source**; steht im Widerspruch zur Clean-Room-/offenes-Format-Geschichte, zu Beiträgen aus der Community und zu Trade-free-Werten; viele Nutzer und Distributionen werden sie ablehnen. **Nicht empfohlen.**

| | Reichweite | Schutz gegen Übernahme | Patentlizenz | Einnahmen durch Dual-Lizenz | Passt zu Trade-free |
|---|---|---|---|---|---|
| **A MPL-Kern + AGPL-Anwendungen** | hoch | gut | ja | ja | ja |
| B AGPL überall | mittel | am stärksten | ja (v3) | ja | ja |
| C Apache überall | am höchsten | keiner | ja | nein | teilweise |
| D Source-available | gering | stark | n. z. | ja | nein |

## 4. Wie du mit Spenden tatsächlich Einnahmen erzielst (alles vereinbar mit Variante A)
1. **Buy-Me-a-Coffee-Seite** (`buymeacoffee.com/echo.of.the.strings`), verlinkt aus dem README, dem Text von CLI `--version`, der Fußzeile des Studios und der Credit-Zeile des HTML-Players (ein *entfernbarer* Credit — niemals verpflichtend).
2. **Mitgliedschaften statt Paywalls:** monatliche Stufen für frühe Builds, Abstimmung über den nächsten Roadmap-Punkt, Name in `SUPPORTERS.md`, privater Chat. Alle bekommen denselben Code; Mitglieder bekommen *Zeit und Mitsprache*.
3. **Bezahlte Extras, die nicht der Code sind:** signierte, sofort lauffähige Binärdateien/Installer, Asset-Pakete (Rigs, Hintergründe, Walk-Cycle-Bibliotheken) unter einer Lizenz, die die Nutzung in Filmen/Spielen erlaubt, aber nicht den Weiterverkauf des Pakets selbst, Videokurse, Workshops.
4. **Auftragsarbeiten und Support:** Rigging-Dienstleistungen, individuelle Exporter, Integrationsberatung; die duale kommerzielle Lizenz für Einbettende.
5. **Sponsorenstufen für Organisationen** (Logo im README) und Förderanträge bei Open-Source-/Kreativfonds (NLnet, Programme vom Typ Sovereign Tech, Kulturfonds) — sie bezahlen genau diese Art offener Infrastruktur.
6. **Transparenzseite:** Veröffentliche Einnahmen und was sie finanziert haben; das erhöht nachweislich die wiederkehrende Unterstützung.
7. Halte **Patreon** (`patreon.com/c/alexio_tf`) und BMC beide sichtbar, gib aber jedem einen eigenen Zweck (BMC einmaliges Dankeschön; Patreon wiederkehrend), damit Unterstützer nicht verwirrt werden.

## 5. Checkliste zur Anwendung von Variante A (≈ 1 Stunde Arbeit, sobald gewählt)
- [x] `LICENSES/MPL-2.0.txt`, `LICENSES/AGPL-3.0-or-later.txt`, `LICENSES/CC-BY-4.0.txt`, `LICENSES/LicenseRef-third-party.txt` (REUSE-Layout) sowie die zusammenfassende `LICENSE`; `REUSE.toml` deckt Nicht-Quelldateien ab.
- [x] `license =` in allen sechs `Cargo.toml`-Dateien gesetzt; SPDX-Header in allen .rs/.js/.mjs/.sh-Dateien, geprüft durch `osatura-cli/tests/spdx_audit.rs`.
- [x] `docs/THIRD_PARTY.md` und `docs/THIRD_PARTY_NOTICES.md`, beide durch `scripts/licence-audit.sh` neu generiert.
- [x] `CONTRIBUTING.md` mit dem Text des DCO 1.1. Vor jeder Dual-Lizenz wird weiterhin ein CLA benötigt.
- [x] `TRADEMARK.md` (erlaubt: „compatible with OSATURA", unveränderte Weiterverbreitung; nicht erlaubt: veränderte Builds unter demselben Namen).
- [x] `.github/FUNDING.yml` (`buy_me_a_coffee: echo.of.the.strings`), damit die Plattform einen Sponsor-Button anzeigt.
- [~] Markenrecherche: Der erste Name ARMATURA kollidierte mit einer Marke in den Klassen 9/42 (EUIPO, 24. Okt. 2026), deshalb wurde das Projekt in OSATURA umbenannt; die EUIPO-Recherche nach „osatura“ ergab keinen Konflikt (ein unabhängiger abgelaufener Treffer in Klasse 10). Die Paketnamen `osatura` auf crates.io, npm und PyPI sind frei (geprüft am 24. Okt. 2026), aber es existiert eine ähnliche Marke, OSATRA (Open-Xchange AG, US-Reg. 7459236, Klassen 9/38/42). OSIM- / TMview- / WIPO- / Domain-Recherchen und die Einschätzung eines Anwalts stehen noch aus, siehe [docs/LIMITATIONS.md](LIMITATIONS.md) L-78.
