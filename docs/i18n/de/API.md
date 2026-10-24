# OSATURA-Rust-API und Stabilitätsrichtlinie

*Fassadenversion 1.4.0 (`osatura_core::api::API_VERSION`). Zielgruppe: Rust-Entwickler, die die Engine einbetten, und Maintainer, die die Fassade ändern. Dokumentationslizenz CC-BY-4.0; Codebeispiele MPL-2.0.*

Dies ist die Referenz für die **unterstützte Rust-API** von `osatura-core`: das Fassadenmodul `osatura_core::api` und dessen Ein-Zeilen-Import `osatura_core::prelude`. Sie listet jedes öffentliche Element auf, die Regeln, die die API stabil halten, und getestete Beispiele. Jedes Beispiel in dieser Datei wird vor der Veröffentlichung der Datei gegen die aktuellen Quellen kompiliert und ausgeführt (siehe [Wie die Beispiele geprüft werden](#wie-die-beispiele-geprüft-werden)).

Inhalt: [Was abgedeckt ist](#was-abgedeckt-ist) - [Bezug](#das-crate-beziehen) - [Stabilitätsrichtlinie](#stabilitätsrichtlinie) - [Semver-Regeln](#semver-regeln) - [Feature-Flags](#feature-flags) - [Thread-Sicherheit](#thread-sicherheit) - [Determinismus](#determinismus) - [Limits](#limits-und-argumentbereiche) - [Fehler](#fehler) - [Referenz: `Animation`](#referenz-animation) - [Referenz: `Player`](#referenz-player) - [Referenz: freie Funktionen](#referenz-freie-funktionen-werkzeuge-versionskontrolle-ci) - [Referenz: Typen](#referenz-typen) - [Beispiele](#beispiele) - [Andere Sprachen](#andere-sprachen) - [Anhang: Maschinen- und Binding-JSON](#anhang-maschinen-json-und-binding-json)

## Was abgedeckt ist

Nur `osatura_core::api` und `osatura_core::prelude` sind die unterstützte öffentliche Rust-API. Die Fassade besteht aus

* `Animation`: einem validierten Projekt samt Undo-Verlauf (laden, abfragen, rendern, exportieren, bearbeiten, Werkzeuge);
* `Player`: einem laufenden Zustandsautomaten mit Layout, Scrollen und Detailstufen (1.1, 1.2);
* freien Funktionen für Versionskontrolle und Animations-CI (1.2 lesend, 1.3 schreibend) und Bitmap-Tracing;
* einfachen Werttypen (`AnimationInfo`, `RenderSettings`, `Image`, `LodLevel`, `SizeFit`, `MocapReport`, `FuzzSummary`, `VcsCommit`, `VcsConflict`, `CiOutcome`, `CiInit`, ...) und den Enums `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome`, `VcsChoice`;
* dem Fehlertyp `ApiError` (implementiert `std::error::Error` und `Display`).

Die C-ABI (`osatura-capi`, Header `osatura.h`) und das WebAssembly-Modul (`osatura-wasm`) sind ausschließlich gegen diese Fassade geschrieben und haben eigene Versionen: [C_ABI.md](C_ABI.md), [WASM.md](WASM.md).

### Was NICHT abgedeckt ist

Jedes andere Modul von `osatura-core` (`engine`, `project`, `edit`, `svg`, `raster`, `physics`, `vcs`, `ci`, ...) und die Re-Exporte auf Wurzelebene (`Engine`, `Project`, ...). Sie sind nur deshalb `pub` und `#[doc(hidden)]`, weil das Kommandozeilenwerkzeug, das Studio, die C-ABI-Tests und die Unit-Tests sie verwenden; sie können sich in jedem Release ändern oder verschwinden, und auch ihre `Err(String)`-Texte sind kein Vertrag. Wenn du etwas brauchst, das nur dort erreichbar ist, bitte um eine Erweiterung der Fassade (ein Minor-Release), statt dich auf Interna zu stützen.

Das Projekt-JSON-Format hat ein eigenes Versionsfeld `format` (derzeit 1); ein Projekt, das von einem neueren Build geschrieben wurde, wird mit `InvalidJson` abgelehnt. Siehe [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Das Crate beziehen

OSATURA wird nicht in einer Registry veröffentlicht; binde es per Pfad (oder Git) aus deinem Workspace ein:

```toml
[dependencies]
osatura-core = { path = "../osatura/osatura-core", default-features = false }
```

`default-features = false` lässt die Rhai-Scripting-Engine weg (siehe [Feature-Flags](#feature-flags)); die Fassade braucht sie nicht. Anschließend `use osatura_core::prelude::*;`. Die Crate-Version (`0.1.0`) und die Fassadenversion (`API_VERSION`) sind voneinander unabhängig: Die Fassade folgt Semver, die Crate-Version verspricht noch nichts. Eine minimal unterstützte Rust-Version ist nicht festgelegt; der Workspace wird mit Rust 1.97 (Edition 2021) entwickelt und getestet.

## Stabilitätsrichtlinie

1. **Die Surface-Datei.** `osatura-core/src/api_surface.txt` listet jedes öffentliche Element von `src/api/` auf (Typen, Felder, Varianten, Konstanten und Methodensignaturen, eines pro Zeile). Der Unit-Test `api::tests::api_surface_snapshot` baut diese Liste aus `api/` neu auf und schlägt fehl, wenn sie von der eingecheckten Datei abweicht. Jede Änderung der Oberfläche ist daher eine bewusste, geprüfte Änderung an beiden Dateien.
2. **Der Prelude-Test.** `osatura-core/tests/prelude_surface.rs` wird als *externes* Crate kompiliert. Er importiert Fassadennamen ausschließlich über `osatura_core::prelude` und ruft eine repräsentative Teilmenge auf (Laden, Rendern, Bearbeiten, Exportieren, `Player`, die Werkzeuge, die Wrapper für Versionskontrolle und CI, Behandlung fehlerhafter Eingaben). Wird ein Name, den er verwendet, nicht mehr aus dem Prelude re-exportiert, oder ändert sich eine Signatur auf eine Weise, die diese Aufrufe bemerken, lässt sich dieser Test nicht mehr kompilieren. Er prüft außerdem `API_VERSION == "1.4.0"`, sodass auch ein Versionssprung dort eine bewusste Änderung ist. Er zählt nicht jeden Prelude-Namen auf, daher bleibt das Hinzufügen eines Namens zur Re-Export-Liste ein manueller Schritt.
3. **Grenzen der Prüfung.** Der Snapshot ist ein Textscan einzeiliger Signaturen in `api/`, kein Diff auf Compiler-Ebene wie bei `cargo-public-api`: Eine semantische Änderung, die den Signaturtext beibehält (zum Beispiel ein anderer Fehler für dieselbe Eingabe), wird nicht erkannt, und eine mehrzeilige Signatur würde übersehen (die Datei hält Signaturen deshalb einzeilig). Die Re-Export-Liste des Prelude (`lib.rs`) wird vom externen Test geprüft, nicht vom Snapshot.
4. **Versteckt heißt nicht unterstützt.** `#[doc(hidden)] pub`-Module existieren für Konsumenten innerhalb des Workspace. Sie liegen außerhalb des Versprechens, auch wenn sie bei dir kompilieren.
5. **So änderst du die Fassade** (Maintainer; auch in [DEVELOPMENT.md](DEVELOPMENT.md)):
   * bearbeite `api/` (einzeilige Signaturen, Doc-Kommentar, ein Doc-Test für neue freie Funktionen, Unit-Tests);
   * füge neue öffentliche Namen zur Re-Export-Liste des `prelude` in `lib.rs` und zu `tests/prelude_surface.rs` hinzu;
   * erhöhe `API_VERSION` (und `API_VERSION_MAJOR` bei Breaking Changes) gemäß den untenstehenden Regeln sowie die in `prelude_surface.rs` geprüfte Version;
   * erzeuge die Surface-Datei bewusst neu: `OSATURA_BLESS=1 cargo test -p osatura-core api::` und prüfe den Diff von `api_surface.txt`;
   * aktualisiere diese Datei sowie `osatura-capi` / `osatura-wasm` nur, wenn das neue Element diese Grenzen überqueren soll (deren ABI-Version ändert sich separat, Ergänzungen behalten sie bei).

## Semver-Regeln

`API_VERSION` ist eine semantische Version ausschließlich der Fassade.

| Änderung | Release |
|---|---|
| Fehlerbehebung, keine Signaturänderung, kein neuer Verhaltensvertrag | Patch (1.4.x) |
| Neue Funktion, Methode, neuer Typ, neue Enum-Variante, neues Struct-Feld, neue Konstante | Minor (1.x.0) |
| Entfernen oder Umbenennen von irgendetwas; Ändern einer Signatur, eines dokumentierten Argumentbereichs so, dass zuvor gültige Eingaben abgelehnt werden, oder der Bedeutung eines dokumentierten Ergebnisses | Major (2.0.0) |

Folgen für Aufrufer:

* **Öffentliche Enums und Structs der Fassade sind `#[non_exhaustive]`.** Füge einen Wildcard-Zweig hinzu, wenn du auf `ApiError`, `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome` oder `VcsChoice` per `match` verzweigst; eine neue Variante ist ein Minor-Release. Außerhalb des Crates kannst du die Structs nicht per Literal konstruieren (und `Image`, `SizeFit` und die Report-Structs sind Ergebnisse, die du nur liest). Einstellungen werden über Konstruktoren gebaut: `RenderSettings::new(scale).with_bones(true)`, `MocapSettings::new().with_smooth(2)`.
* **Strings innerhalb von Ergebnissen sind Daten, kein Vertrag.** `VcsConflict::kind` ist heute einer von `"BothChanged"`, `"DeleteEdit"`, `"KeyRetimed"`, `"OrderConflict"`, `"ArrayConflict"` und *kann in Minor-Releases weitere Werte erhalten*. Die JSON-Reports (`FuzzSummary::json`, `CiOutcome::json`) tragen ihre eigenen Schemanamen (`osatura-fuzz-report` v1, `osatura-ci/1`). Fehlermeldungstexte sind für Menschen gedacht.
* **Verhaltensstabilität.** Rendering und Auswertung sind deterministisch (siehe unten), aber die Pixelausgabe kann sich in einem Minor-Release ändern, wenn ein Fehler im Rasterizer oder in einem Effekt behoben wird; die von der Animations-CI aufgezeichneten Goldens erinnern daran. Hashe gerenderte Pixel nicht über OSATURA-Versionen oder Maschinen hinweg ohne Toleranz.

Versionsverlauf der Fassade: 1.0 (laden, abfragen, rendern, exportieren, bearbeiten, Undo) - 1.1 (`Player`, Zustandsautomaten, Datenbindungen) - 1.2 (Layout, Scrollen und Detailstufen auf `Player`; `lod_levels`, `fit_to_size`, `retarget_bvh`, `fuzz`, `from_image`, `trace_image_svg`, `vcs_log`, `ci_run`) - 1.3 (schreibende Operationen für Versionskontrolle und CI, `DirtyTree` / `MergeInProgress` / `Conflicts`, `CiOutcome::updated` / `blessed`) - 1.4 (Erstellung von Ebenen und Knochen, `bake_physics`, APNG- und EXR-Export, `vcs_branches` / `vcs_branch_delete` / `vcs_status` / `vcs_diff` / `vcs_show` / `vcs_gc` / `vcs_fsck`).

## Feature-Flags

`osatura-core` hat ein Cargo-Feature:

| Feature | Standard | Wirkung |
|---|---|---|
| `scripting` | an | Kompiliert das Rhai-basierte Sandbox-Scripting-Modul (`script.rs`), das nur vom Kommandozeilenbefehl `script` verwendet wird. Zieht `rhai` und darüber `smartstring` (MPL-2.0+, das einzige Third-Party-Crate mit schwachem Copyleft im Baum) hinein. |

Die Fassade hat **keine** Funktionalität hinter einem Feature: Jedes hier dokumentierte Element existiert mit `default-features = false`. `osatura-capi`, `osatura-wasm` und `osatura-gpu` hängen vom Core ohne das Feature ab und enthalten daher kein MPL-Third-Party-Crate; `osatura-cli --no-default-features` tut dasselbe und meldet dann den Befehl `script` als nicht verfügbar. Render-Backends sind keine Features: Der CPU-Rasterizer wird von der Fassade immer verwendet; der optionale GPU-Renderer ist ein eigenes Crate (`osatura-gpu`) und ein CLI-Feature (`gpu`) und über diese API nicht erreichbar.

## Thread-Sicherheit

* `Animation`, `Player`, `ApiError` und `Image` sind `Send + Sync` (für dieses Dokument zur Compile-Zeit geprüft). Die Fassade schreibt keinen globalen Zustand; sie liest nur zwei prozessweite Asset-Einstellungen (das Standard-Asset-Verzeichnis und den Schalter für externe Assets), die nichts in der Fassade ändert.
* Methoden, die `&self` nehmen (`info`, `bone_world`, `render_*`, `export`, `lod_levels`, `fit_to_size`, `fuzz`, `player`, `to_json`, ...), können nebenläufig auf mehreren Threads über dieselbe `Animation` laufen; Methoden, die `&mut self` nehmen (`set_key`, `delete_key`, `undo`, `redo`, `retarget_bvh`, jede `Player`-Eingabe und `advance`), brauchen wie in Rust üblich exklusiven Zugriff. Das Beispiel `render_in_parallel` unten rendert jeden Frame auf einem eigenen Thread und prüft, dass die Bilder denen des Single-Thread-Laufs entsprechen.
* Es gibt kein internes Locking und keinen Hintergrund-Thread. Ein `Player` arbeitet auf seiner eigenen Kopie des Projekts, sodass späteres Bearbeiten der `Animation` einen laufenden Player nie beeinflusst.
* `Animation::from_json` löst relative Bilddateien gegen das **aktuelle Verzeichnis** des Prozesses auf und lehnt absolute Pfade und `..` ab (siehe [SECURITY.md](SECURITY.md)); bette Bilder als Data-URIs ein oder setze das Arbeitsverzeichnis vor dem Laden, wenn ein Projekt auf Dateien verweist.
* Die Funktionen für Versionskontrolle und CI arbeiten auf Dateien: Sie sind nicht dagegen abgesichert, dass ein anderer Prozess im selben Moment dasselbe Projekt bearbeitet (die Fassade prüft vorab auf einen veränderten Arbeitsbaum und einen ausstehenden Merge, aber es gibt keine Sperre).

## Determinismus

Dasselbe Projekt, dieselben Argumente, derselbe OSATURA-Build und dieselbe Maschine: dieselben Bytes. Das gilt für `render_rgba` / `render_png` / `render_svg`, `bone_world`, jeden Export, `lod_levels` und `fuzz` (bei gegebenem Seed), `fit_to_size` sowie für einen `Player` (dieselben Eingaben und dieselben `advance`-Aufrufe ergeben dieselbe Pose; `advance` zerlegt die Zeit in Stücke von höchstens 1/60 s, sodass es nur unterhalb dieser Auflösung darauf ankommt, wie du die Zeit aufteilst). Zufällige Entscheidungen innerhalb der Engine (Noise, Partikel, Fuzzing, Tracing-Cluster) verwenden Generatoren mit Seed, niemals die Uhr. Was *nicht* versprochen wird: identische Pixel an anti-aliasten Kanten zwischen verschiedenen Betriebssystemen, CPUs, Compilern oder `libm`-Versionen, zwischen 32- und 64-Bit-Targets oder zwischen OSATURA-Versionen. Byte-genaue Goldens wurden nur über Prozesse hinweg auf einer Linux-x86-64-Maschine verifiziert (siehe [LIMITATIONS.md](LIMITATIONS.md)); verwende für alles Maschinenübergreifende Toleranzen (die Animations-CI tut das).

## Limits und Argumentbereiche

| Element | Regel |
|---|---|
| `MAX_JSON_BYTES` | 256 MiB; ein längerer Text ergibt `InvalidJson` |
| `MAX_RENDER_SIDE` | 16384 Ausgabepixel pro Seite (`render_*` und `Player::render_*`) |
| Render- / GIF-Skalierung | endlich und innerhalb von `0.05..=4`, sonst `InvalidArgument` |
| Frame-Positionen | müssen endlich sein; Werte außerhalb der Timeline werden auf `0..=frames-1` **begrenzt** (kein Fehler). Gebrochene Frames sind in Ordnung. |
| Keys (`set_key`, `delete_key`) | `frame` und `value` endlich; `prop` ist ein Eigenschaftsname in snake_case aus dem Projekt-JSON (`"bone_angle"`, `"layer_opacity"`, ...) |
| Projekt | `fps` in `(0, 1000]`, `frames` in `1..=200000`, Bühne bis zu 32768 Einheiten pro Seite, höchstens 4096 Knochen, Zahlen jenseits von +-1e7 werden beim Laden abgelehnt; weitere Limits pro Feature werden beim Laden validiert und kommen als `InvalidProject` zurück |
| `Player::advance(dt)` | `dt` endlich, `0..=10` Sekunden |
| `Player::set_canvas_size` | beide Seiten endlich, `> 0`, höchstens 32768 |
| `Player::set_screen_size` | Breite und Höhe endlich und `> 0`; `max_error_px` endlich und `>= 0` |
| `Animation::lod_levels` | `tolerance_px > 0`; `scale` in `(0, 64]`; `levels` in `1..=12` |
| `Animation::fit_to_size` | `max_bytes` in `1000..=1_000_000_000` |
| `Animation::fuzz` | `samples` in `1..=100000` |
| `trace_image_svg`, `Animation::from_image` | `colors` in `2..=64`; PNG- oder JPEG-Bytes (Größenobergrenzen gelten) |
| `vcs_log` | `limit` in `1..=10000` |

## Fehler

Jede fehlbare Funktion gibt `Result<_, ApiError>` zurück. Fehlerhafte Eingaben sind ein Fehlerwert und führen **nie zu einer Panik**; eine Panik, die dennoch in der Engine auftritt, wird an der Fassadengrenze abgefangen und als `ApiError::Internal` zurückgegeben (bitte als Bug melden). `ApiError` implementiert `Display` und `std::error::Error`, sodass `?` mit `Box<dyn Error>` und Typen im Stil von `anyhow` funktioniert.

| Variante | Bedeutung | Typische Quellen |
|---|---|---|
| `InvalidJson(String)` | kein JSON, kein Projekt, falsche Form, zu groß (> `MAX_JSON_BYTES`) oder ein `format`, das neuer ist als dieser Build; auch ein fehlerhafter JSON-Wert, der an `vcs_resolve` übergeben wurde | `Animation::from_json`, `validate_json`, `vcs_resolve` |
| `InvalidProject(Vec<String>)` | geparst, aber die Validierung ist fehlgeschlagen; eine Meldung pro Problem (unbekannter Knochen, ungültiger Bereich, Zyklus, ...) | `from_json`, `from_image`, Neuaufbau der Engine im `Player` |
| `InvalidArgument(String)` | ein Argument des Aufrufers wurde abgelehnt: nicht endliche Zahl, Wert außerhalb des Bereichs, unbekannte Eigenschaft, unbekannte Automaten-/Eingabe-/Scroll-ID, unbrauchbares BVH oder Bild, ungültiger Branch-Name, nichts zu committen, kein Repository | fast jede Funktion |
| `EditRejected(Vec<String>)` | eine Bearbeitung wurde abgelehnt (zum Beispiel ein gesperrter Kanal oder eine gesperrte Ebene, oder ein ungültiges Ergebnis); das Projekt bleibt unverändert | `set_key`, `delete_key`, `retarget_bvh` |
| `Export(String)` | ein Encoder ist fehlgeschlagen oder das Format ist für dieses Projekt nicht möglich | `export`, `render_png`, `fit_to_size` |
| `Internal(String)` | eine abgefangene Panik | beliebig |
| `DirtyTree(String)` | (1.3) die Projektdatei hat nicht committete Änderungen, die die Operation überschreiben würde | `vcs_checkout` (außer mit `force`), `vcs_merge`, `vcs_cherry_pick`, `vcs_revert` |
| `MergeInProgress(String)` | (1.3) ein Merge, Cherry-Pick oder Revert ist unvollendet; auflösen und committen, oder `vcs_abort` | dieselben Funktionen |
| `Conflicts(Vec<String>)` | (1.3) `vcs_commit`, während noch Konflikte bestehen; ein Verweis pro Konflikt | `vcs_commit` |

Eine fehlschlagende Prüfung der Animations-CI ist **kein** Fehler: `ci_run` gibt `Ok(CiOutcome)` mit `exit_code` 1 (eine Prüfung ist fehlgeschlagen) oder 2 (eine Prüfung konnte nicht ausgewertet werden) zurück; nur ein unbrauchbares Manifest ist `Err`. Merge-Konflikte sind ebenfalls ein Ergebnis (`VcsMergeOutcome::Conflicted`), kein Fehler.

## Referenz: `Animation`

Ein geladenes, validiertes Projekt samt seinem Undo-Verlauf. Erzeuge es mit `from_json`, `demo` oder `from_image`. Alle Methoden, die das Projekt lesen, nehmen `&self`.

| Methode | Rückgabe | Hinweise |
|---|---|---|
| `Animation::from_json(&str)` | `Result<Animation, ApiError>` | parsen + validieren. `InvalidJson` / `InvalidProject`. Unbekannte JSON-Felder werden hier stillschweigend ignoriert (der CLI-Befehl `check` meldet sie). |
| `Animation::validate_json(&str)` | `Result<(), ApiError>` | dieselben Prüfungen, behält nichts |
| `Animation::demo()` | `Animation` | kleines eingebautes Rig (6 Knochen, 48 Frames bei 24 fps, 640 x 480); praktisch für Tests und erste Schritte |
| `Animation::from_image(&[u8], colors, name)` | `Result<Animation, ApiError>` | trackt PNG-/JPEG-Bytes zu gefüllten Vektor-Ebenen auf einer Bühne in Bildgröße |
| `to_json(&self)` | `String` | das aktuelle Projekt als formatiertes JSON (Roundtrip über `from_json`) |
| `info(&self)` | `AnimationInfo` | `name`, `width`, `height` (Bühneneinheiten), `fps`, `frames` |
| `frame_count(&self)`, `fps(&self)` | `u32`, `f32` | |
| `bone_names(&self)` | `Vec<String>` | Rig-Reihenfolge = Reihenfolge von `bone_world` |
| `layer_names(&self)` | `Vec<String>` | Ebenenpfade `group/child`, Tiefensuche |
| `bone_world(&self, frame)` | `Result<Vec<[f32; 6]>, ApiError>` | Weltmatrix pro Knochen `[a b c d tx ty]`; ein knochenlokaler Punkt `(x, y)` wird auf `(a*x + c*y + tx, b*x + d*y + ty)` abgebildet. Frame begrenzt. Wertet das gesamte Rig aus (IK, Constraints, Federn, Ausdrücke). |
| `machine_names(&self)` | `Vec<String>` | im Projekt gespeicherte Laufzeit-Zustandsautomaten |
| `player(&self, machine)` | `Result<Player, ApiError>` | startet den benannten Automaten bei Zeit 0 auf einer Kopie des Projekts |
| `render_rgba(&self, frame, &RenderSettings)` | `Result<Image, ApiError>` | RGBA8 mit Straight-Alpha, zeilenweise, oberste Zeile zuerst |
| `render_png(&self, frame, &RenderSettings)` | `Result<Vec<u8>, ApiError>` | eine PNG-Datei im Speicher |
| `render_svg(&self, frame, show_bones: bool)` | `Result<String, ApiError>` | ein SVG-Dokument (Vektorausgabe hat keine Skalierung) |
| `export(&self, ExportFormat, scale)` | `Result<Vec<u8>, ApiError>` | `Gif` (Endlosschleife, gesamte Timeline; `scale` innerhalb von `0.05..=4`), `Glb` (binäres glTF 2.0), `Lottie` (JSON-Bytes), `Html` (eine in sich geschlossene Player-Seite); `scale` wird von den anderen drei ignoriert. Was jedes Format behält oder verwirft: [EXPORT_FORMATS.md](EXPORT_FORMATS.md). |
| `set_key(&mut self, target, prop, frame, value, Interpolation)` | `Result<(), ApiError>` | setzt oder fügt einen Keyframe auf `target.prop` hinzu; validiert, rückgängig machbar, beachtet Kanal- und Ebenensperren (`EditRejected`) |
| `delete_key(&mut self, target, prop, frame)` | `Result<bool, ApiError>` | `true`, wenn ein Key entfernt wurde |
| `add_rect_layer(&mut self, name, x, y, w, h, fill, parent)`, `add_ellipse_layer(&mut self, name, cx, cy, rx, ry, fill, parent)` | `Result<(), ApiError>` | (1.4) hängt eine gefüllte Formebene an (auf oberster Ebene oder in die durch `parent` benannte Gruppe der obersten Ebene); eindeutiger Name ohne `/`, endliche positive Größen, eine gültige Farbe; rückgängig machbar |
| `add_group_layer(&mut self, name)` | `Result<(), ApiError>` | (1.4) hängt eine leere Gruppenebene an |
| `add_bone(&mut self, name, parent, length, x, y, angle)` | `Result<(), ApiError>` | (1.4) hängt einen Knochen an (`parent` muss existieren, Winkel in Grad); rückgängig machbar |
| `bake_physics(&mut self, chain, preset, overwrite)` | `Result<PhysicsBake, ApiError>` | (1.4) simuliert eine Knochenkette (Wurzel zuerst) mit `hair`, `tail`, `rope`, `antenna` oder `cape` und speichert Winkelkanäle als einen Undo-Schritt; mit `overwrite = false` bleiben von Hand gekeyte Knochen erhalten und werden in `skipped` aufgeführt |
| `render_exr(&self, frame, &RenderSettings, float32)` | `Result<Vec<u8>, ApiError>` | (1.4) OpenEXR-Datei (linear, premultiplied; HALF oder FLOAT) aus dem Float-Compositor |
| `undo(&mut self)`, `redo(&mut self)` | `bool` | `false`, wenn es nichts rückgängig zu machen bzw. zu wiederholen gibt |
| `lod_levels(&self, tolerance_px, scale, levels)` | `Result<Vec<LodLevel>, ApiError>` | Stufe 0 ist das Original, Stufe k >= 1 bleibt bei `scale` innerhalb von `tolerance_px * 2^(k-1)` Ausgabepixeln; jede Stufe trägt ihre deklarierte Schranke und ein Projekt-JSON. Deterministisch; die Schranke deckt die Timeline-Wiedergabe ab (nicht die Posen von Zustandsautomaten). In nicht optimierten Builds langsam (Release verwenden). |
| `fit_to_size(&self, FitFormat, max_bytes)` | `Result<SizeFit, ApiError>` | kleinster Export innerhalb des Budgets durch schrittweises Senken der Qualitätsparameter; wenn das nicht möglich ist, das kleinste Ergebnis mit `met == false` und einem `reason` |
| `retarget_bvh(&mut self, bvh_text, &MocapSettings)` | `Result<MocapReport, ApiError>` | BVH-Retargeting als **eine rückgängig machbare Bearbeitung**; Knochen werden nach Name und Topologie zugeordnet; gesperrte Kanäle werden übersprungen und gemeldet |
| `fuzz(&self, samples, seed)` | `Result<FuzzSummary, ApiError>` | Pose-Fuzzing; die Animation wird nur gelesen; vollständiger Report als JSON |

`Interpolation` des Segments, das an einem Key beginnt: `Linear`, `Step` (halten), `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` (Catmull-Rom durch benachbarte Keys). Die reichhaltigeren Kurven des Projektformats (kubische Bézier, Bounce, Back, Spring) werden über die CLI und das Studio erstellt, nicht über `set_key`.

## Referenz: `Player`

Ein laufender Zustandsautomat (interaktive Wiedergabe). Wird von `Animation::player` erzeugt; arbeitet auf einer Kopie des Projekts; deterministisch. Zustandsautomaten und Datenbindungen sind einfaches Projekt-JSON (`machines`, `bindings`), zusammengefasst im [Anhang](#anhang-maschinen-json-und-binding-json).

| Methode | Rückgabe | Hinweise |
|---|---|---|
| `set_bool(name, bool)`, `set_number(name, f64)`, `fire(name)` | `Result<(), ApiError>` | setzt eine Eingabe per Name; die falsche Art von Eingabe oder ein unbekannter Name ergibt `InvalidArgument`. Ein Trigger bleibt gesetzt, bis ein Übergang, der ihn prüft, auslöst (er wird verbraucht) oder bis zum Ende des nächsten `advance`; `hold: true` hält ihn, bis er verbraucht wird. |
| `set_value(path, f64)` | `Result<(), ApiError>` | veröffentlicht einen externen Wert für die Datenbindungen (`"player.health"`). Eine Scroll-Region, deren `input` dieser Pfad ist, übernimmt den Wert als ihren Offset in Pixeln entlang ihrer Hauptachse (das Scrollen ruht dann). |
| `advance(dt)` | `Result<(), ApiError>` | schaltet Automat und Scroll-Regionen um `dt` Sekunden (`0..=10`) weiter |
| `state(layer)` | `Option<String>` | aktueller Zustand einer Automatenebene; `None` für eine unbekannte Ebene |
| `events_json()` | `String` | anstehende Ereignisse `[{"time":..,"layer":..,"kind":"entered"|"exited","state":..}]`, nicht verbraucht |
| `take_events_json()`, `clear_events()` | `String`, `()` | verbraucht / verwirft die anstehenden Ereignisse |
| `render_rgba(&RenderSettings)`, `render_svg(show_bones)`, `bone_world()` | wie bei `Animation` | die aktuelle Pose; das Bild hat die mit `set_canvas_size` gesetzte Canvas-Größe mal die Skalierung |
| `set_canvas_size(w, h)`, `canvas_size()` | `Result<(), ApiError>`, `(f32, f32)` | (1.2) responsives Layout: Container werden neu gelöst, Scroll-Grenzen folgen |
| `scroll_ids()` | `Vec<String>` | IDs der Scroll-Regionen des Projekts |
| `scroll_drag(id, x, y)` | `Result<(), ApiError>` | roher Zeiger-Offset in Pixeln (Inhalt um so viel nach oben / links verschoben); über die Enden hinaus wird begrenzt oder gummibandartig gedehnt |
| `scroll_release(id, vx, vy)` | `Result<(), ApiError>` | Loslass-Geschwindigkeit in px/s; `advance` lässt dann ausgleiten, einrasten und zurückfedern |
| `scroll_offset(id)`, `scroll_max(id)`, `scroll_settled(id)` | `Option<[f32; 2]>`, `Option<[f32; 2]>`, `Option<bool>` | `None` für eine unbekannte ID |
| `set_screen_size(w_px, h_px, max_error_px)` | `Result<(), ApiError>` | (1.2) wählt die gröbste von acht vorgebauten Detailstufen, deren deklarierte Schranke bei der Skalierung, die den Canvas in diesen Bildschirm einpasst, höchstens `max_error_px` beträgt. Die Stufen werden bei der ersten Verwendung einmal gebaut (Stufe 1 begrenzt durch 0.125 Bühneneinheiten, dann verdoppelnd) und zwischengespeichert; bei `max_error_px` 0 kommen nur verlustfreie Stufen infrage. |
| `clear_screen_size()`, `lod_level()`, `lod_bound_px()` | `()`, `usize`, `f32` | zurück zum Original; verwendete Stufe (0 = Original); ihre deklarierte Schranke in Pixeln |

Hinweise: Die Schranke der Detailstufen deckt Geometrie und Timeline-Keys ab, wie in `lod.rs` dokumentiert (`ASSUMPTIONS`, `LIMITS`); Posen von Zustandsautomaten und Bindungen werden von ihr nicht modelliert, und Pose-Kanäle, die auf eine von einer Stufe entfernte Ebene zielen, werden verworfen. Die Scrub-Proxys des Studios verwenden einen separaten Mechanismus mit eigenem Stufenraster. Der `input` einer Scroll-Region läuft über die Wertetabelle des Players, nicht über die Laufzeit der Datenbindungen, sodass Ausdrücke, Remapping und Glättung dafür nicht gelten.

## Referenz: freie Funktionen (Werkzeuge, Versionskontrolle, CI)

| Funktion | Rückgabe | Wirkung |
|---|---|---|
| `trace_image_svg(&[u8], colors)` | `Result<String, ApiError>` | trackt eine Bitmap zu einem SVG aus gefüllten Regionen |
| `vcs_log(path, limit)` | `Result<Vec<VcsCommit>, ApiError>` | (1.2, nur lesend) Verlauf, neueste zuerst; `path` ist die Projektdatei oder ein beliebiger Pfad innerhalb ihres Repositorys; ein leerer Verlauf ergibt eine leere Liste |
| `ci_run(manifest)` | `Result<CiOutcome, ApiError>` | (1.2) Animations-CI im Verify-Modus (nichts wird neu aufgezeichnet); relative Pfade im Manifest werden neben diesem aufgelöst |
| `vcs_init(project_file, author)` | `Result<(), ApiError>` | (1.3) beginnt einen Verlauf neben der Datei auf dem Branch `main`, noch ohne Commit; schlägt fehl, wenn bereits einer existiert |
| `vcs_commit(path, message, author)` | `Result<String, ApiError>` | Commit-ID; schließt einen Merge ab, sobald kein Konflikt mehr übrig ist; `Conflicts`, solange welche bestehen; nichts zu committen oder keine Nachricht ist `InvalidArgument` |
| `vcs_branch(path, name, at)`, `vcs_tag(path, name, at)` | `Result<String, ApiError>` | erstellt einen Branch / Tag (Standard: aktueller Commit); es wird nicht dorthin gewechselt; Tags bewegen sich nie; die Commit-ID wird zurückgegeben |
| `vcs_checkout(path, target, force)` | `Result<String, ApiError>` | wechselt die Datei auf einen Branch (bleibt darauf) oder eine beliebige Revision (detached); `DirtyTree`, sofern nicht `force`; `MergeInProgress` |
| `vcs_merge(path, rev, no_ff, message, author)` | `Result<VcsMergeOutcome, ApiError>` | `UpToDate`, `FastForward(id)`, `Merged(id)` oder `Conflicted(Vec<VcsConflict>)` |
| `vcs_cherry_pick(path, rev, author)`, `vcs_revert(path, rev, author)` | `Result<VcsMergeOutcome, ApiError>` | spielt die Änderung eines Commits erneut ein / macht sie rückgängig |
| `vcs_conflicts(path)` | `Result<Vec<VcsConflict>, ApiError>` | die noch ausstehenden Konflikte (leer, wenn keine) |
| `vcs_resolve(path, conflict_path, &VcsChoice)` | `Result<usize, ApiError>` | löst einen Konflikt mit `Ours`, `Theirs` oder `Value(json text)`; gibt zurück, wie viele noch übrig sind |
| `vcs_abort(path)` | `Result<(), ApiError>` | verwirft einen unvollendeten Merge / Cherry-Pick / Revert und stellt den letzten Commit wieder her |
| `vcs_branches(path)` | `Result<Vec<VcsBranch>, ApiError>` | (1.4) die Branches mit ihrem Commit und der Angabe, ob ausgecheckt |
| `vcs_branch_delete(path, name)` | `Result<(), ApiError>` | (1.4) löscht einen Branch; der ausgecheckte Branch und unbekannte Namen werden abgelehnt; Objekte bleiben bis `vcs_gc` bestehen |
| `vcs_status(path)` | `Result<VcsStatus, ApiError>` | (1.4) Branch, HEAD, nicht committete semantische Änderungen, unvollendeter Merge, Konfliktpfade |
| `vcs_diff(path, a, b)` | `Result<Vec<VcsChange>, ApiError>` | (1.4) semantische Unterschiede zwischen zwei Revisionen; `None` ist die Arbeitsdatei |
| `vcs_show(path, rev)` | `Result<VcsShow, ApiError>` | (1.4) ein Commit und seine Änderungen gegenüber seinem ersten Elternteil |
| `vcs_gc(path)` | `Result<VcsGc, ApiError>` | (1.4) löscht unerreichbare Objekte; bricht ab, ohne zu löschen, wenn der Verlauf nicht gelesen werden kann |
| `vcs_fsck(path)` | `Result<Vec<String>, ApiError>` | (1.4) Integritätsprobleme, leer, wenn alles in Ordnung ist |
| `ci_init(project, manifest, write_goldens)` | `Result<CiInit, ApiError>` | schreibt ein kalibriertes Standard-Manifest (überschreibt nie eine bestehende Datei; Standardname `osatura-ci.json` neben dem Projekt); zeichnet auf Wunsch gleich die Goldens auf |
| `ci_run_mode(manifest, CiMode)` | `Result<CiOutcome, ApiError>` | `Verify`, `Update` (zeichnet alle Goldens der Render-Prüfungen neu auf), `Bless` (übernimmt nur fehlschlagende oder fehlende) |
| `ci_bless(manifest)` | `Result<CiOutcome, ApiError>` | Kurzform für `Bless` |

Der Verlauf wird neben der Projektdatei gespeichert (`.osatura-vcs/`); Commit-Nachrichten und Autoren sind einfacher Text. Ein Konflikt enthält den Identitätsverweis `path`, eine `kind`, eine `description` und die Werte `base` / `ours` / `theirs` als JSON-Text (`None` = dort nicht vorhanden). Nach einem Merge mit Konflikten enthält die Projektdatei den zusammengeführten Zustand, wobei die konfligierenden Elemente den Wert des aktuellen Branches haben, und ein ausstehender Zustand wird festgehalten, bis du jeden Konflikt auflöst und `vcs_commit` aufrufst, oder `vcs_abort`. Die binären Assets, auf die ein Projekt verweist, werden nicht versioniert, nur ihre Pfadstrings. Der semantische Merge, seine Konfliktarten und seine Grenzen sind in [ROADMAP.md](ROADMAP.md) Abschnitt 9 (Punkt I-01) und in [LIMITATIONS.md](LIMITATIONS.md) beschrieben.

Nicht in der Fassade: Überschreibungen der Knochenzuordnung beim Mocap, Tracing-Feintuning über die Farbanzahl hinaus, Fit-Optionen über Format und Budget hinaus, GPU-Rendering, das Studio, Zusammenarbeit, der Agent-Server, die vielen Authoring- und Bake-Befehle (Physik, Foot Lock, Magnete, Autorig, ...), Löschen / Auflisten von Branches, Diff, Show, Status, GC, Fsck sowie CI-Optionen wie Prune, `--only` und Artefakte. Verwende dafür die CLI ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

## Referenz: Typen

| Typ | Felder / Varianten |
|---|---|
| `AnimationInfo` | `name: String`, `width: f32`, `height: f32` (Bühneneinheiten), `fps: f32`, `frames: u32` |
| `RenderSettings` | `scale: f32` (0.05..=4), `show_bones: bool`, `transparent: bool`; `new(scale)`, `with_bones(bool)`, `with_transparent(bool)`; `Default` = Skalierung 1, beide aus |
| `Image` | `width: u32`, `height: u32`, `pixels: Vec<u8>` (RGBA8, Straight-Alpha) |
| `ExportFormat` | `Gif`, `Glb`, `Lottie`, `Html`, `Apng` (1.4) |
| `PhysicsBake` (1.4) | `added: usize`, `skipped: Vec<String>` |
| `VcsBranch` (1.4) | `name`, `commit`, `current: bool` |
| `VcsChange` (1.4) | `kind` (`"Added"`, `"Removed"`, `"Changed"`, `"Moved"`, `"Retimed"`), `path`, `description` |
| `VcsStatus` (1.4) | `branch`, `head`, `changes: Vec<VcsChange>`, `merge`, `conflicts: Vec<String>` |
| `VcsShow` (1.4) | `commit: VcsCommit`, `changes: Vec<VcsChange>` |
| `VcsGc` (1.4) | `kept`, `removed` |
| `Interpolation` | `Linear`, `Step`, `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` |
| `LodLevel` | `level`, `declared_px: f64`, `bound_stage: f64`, `path_anchors`, `keys`, `layers`, `json: String`; `bound_at(screen_scale) -> f64` |
| `FitFormat` | `Lottie`, `Html`, `Gif`, `Apng`, `Project` |
| `SizeFit` | `met: bool`, `size: usize`, `bytes: Vec<u8>`, `ssim_min: f64`, `reason: Option<String>` |
| `MocapSettings` | `in_place`, `mirror`, `smooth` (Radius des gleitenden Mittelwerts in Frames); `new()`, `with_in_place`, `with_mirror`, `with_smooth`; `Default` |
| `MocapReport` | `matched: Vec<(String, String)>` (BVH-Gelenk, Rig-Knochen), `unmatched_joints`, `unmatched_bones`, `frames`, `bones_used`, `blocked` (gesperrtes `target.prop`), `notes` |
| `FuzzSummary` | `samples`, `failures`, `failure_rate: f32`, `max_severity: f32`, `json: String` |
| `VcsCommit` | `id`, `author`, `time: u64` (Unix-Sekunden), `message` |
| `VcsConflict` | `path`, `kind`, `description`, `base`, `ours`, `theirs: Option<String>` |
| `VcsMergeOutcome` | `UpToDate`, `FastForward(String)`, `Merged(String)`, `Conflicted(Vec<VcsConflict>)` |
| `VcsChoice` | `Ours`, `Theirs`, `Value(String)` |
| `CiMode` | `Verify`, `Update`, `Bless` |
| `CiOutcome` | `total`, `passed`, `failed`, `errors`, `exit_code: i32` (0 gut, 1 fehlgeschlagen, 2 nicht auswertbar), `json: String`, `updated`, `blessed` |
| `CiInit` | `manifest: String`, `checks: Vec<(String, String)>` (Art, Name), `recorded: Option<CiOutcome>` |
| Konstanten | `API_VERSION: &str`, `API_VERSION_MAJOR: u32`, `MAX_RENDER_SIDE: u32`, `MAX_JSON_BYTES: usize` |

## Beispiele

Alle diese Beispiele sind vollständig und wurden ausgeführt. Sie setzen `use osatura_core::prelude::*;` voraus.

### Laden, auswerten, rendern, exportieren

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Load a project, query it, render one frame and export the whole animation.
pub fn quick(project_path: &str) -> Result<(), Box<dyn Error>> {
    let json = std::fs::read_to_string(project_path)?;
    let mut anim = Animation::from_json(&json)?;           // parse + validate; typed error otherwise

    // Facts about the project.
    let info = anim.info();
    println!("{}: {}x{} @ {} fps, {} frames", info.name, info.width, info.height, info.fps, info.frames);

    // Evaluate: world matrices of every bone at frame 10 (six floats per bone: a b c d tx ty).
    let names = anim.bone_names();
    let world = anim.bone_world(10.0)?;
    let m = world[0];
    // A bone-local point (x, y) lands at (a*x + c*y + tx, b*x + d*y + ty).
    let (x, y) = (m[0] * 10.0 + m[2] * 0.0 + m[4], m[1] * 10.0 + m[3] * 0.0 + m[5]);
    println!("tip of {} (10 units along its x axis) is at ({x:.1}, {y:.1})", names[0]);

    // Edit through the undoable editor, then render.
    anim.set_key(&names[0], "bone_angle", 10.0, 25.0, Interpolation::EaseInOut)?;
    let png = anim.render_png(10.0, &RenderSettings::new(1.0).with_bones(true))?;
    std::fs::write("frame10.png", &png)?;
    let svg = anim.render_svg(10.0, false)?;
    std::fs::write("frame10.svg", svg)?;

    // Raw pixels: straight-alpha RGBA8, row-major, top row first.
    let img = anim.render_rgba(10.0, &RenderSettings::new(0.5).with_transparent(true))?;
    assert_eq!(img.pixels.len(), img.width as usize * img.height as usize * 4);

    // Whole-animation exports.
    let glb = anim.export(ExportFormat::Glb, 1.0)?;
    let gif = anim.export(ExportFormat::Gif, 0.5)?;        // `scale` only matters for GIF
    std::fs::write("anim.glb", &glb)?;
    std::fs::write("anim.gif", &gif)?;
    println!("{} bones, {} bytes of glb, {} bytes of gif", names.len(), glb.len(), gif.len());
    Ok(())
}
```

### Bearbeiten, Undo und Fehlerbehandlung

```rust
use osatura_core::prelude::*;

/// Edits are validated, undoable and all-or-nothing.
pub fn edit_and_errors() {
    let mut anim = Animation::demo();                       // small built-in project
    let bone = anim.bone_names()[0].clone();
    let before = anim.to_json();

    anim.set_key(&bone, "bone_angle", 4.0, 30.0, Interpolation::Smooth).unwrap();
    assert!(anim.delete_key(&bone, "bone_angle", 4.0).unwrap());      // true: a key was removed
    assert!(!anim.delete_key(&bone, "bone_angle", 77.0).unwrap());    // false: nothing there
    assert!(anim.undo() && anim.undo());                    // each successful edit is one undo step
    assert_eq!(anim.to_json(), before);
    assert!(anim.redo());

    // Errors are values. Match on them with a wildcard arm: the enum is #[non_exhaustive].
    match anim.set_key(&bone, "no_such_prop", 0.0, 1.0, Interpolation::Linear) {
        Err(ApiError::InvalidArgument(why)) => println!("rejected argument: {why}"),
        Err(ApiError::EditRejected(problems)) => println!("edit refused: {problems:?}"),
        Err(other) => println!("other failure: {other}"),   // Display is implemented for every variant
        Ok(()) => unreachable!(),
    }
    // A rejected edit leaves the project exactly as it was.
    let snapshot = anim.to_json();
    assert!(anim.set_key("no_such_bone", "bone_angle", 1.0, 1.0, Interpolation::Linear).is_err());
    assert_eq!(anim.to_json(), snapshot);

    // Bad input never panics.
    assert!(matches!(Animation::from_json("{ nope"), Err(ApiError::InvalidJson(_))));
    assert!(anim.render_rgba(f32::NAN, &RenderSettings::default()).is_err());
    assert!(anim.render_rgba(0.0, &RenderSettings::new(50.0)).is_err()); // scale must be 0.05..=4
}
```

### Interaktive Wiedergabe mit einem Zustandsautomaten

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Run a state machine of a project (machine.json has machine "hero", layer "body", bool input "excited").
pub fn play(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let machine = anim.machine_names()[0].clone();         // e.g. "hero"
    let mut player = anim.player(&machine)?;               // works on a copy of the project

    player.set_bool("excited", true)?;                     // also set_number, fire (trigger), set_value (binding path)
    for _ in 0..30 {
        player.advance(1.0 / 30.0)?;                       // deterministic; dt in seconds, 0..=10
    }
    println!("body is in state {:?}", player.state("body"));  // Some("wave")
    println!("{}", player.take_events_json());             // [{"time":..,"layer":"body","kind":"entered","state":..}, ..]

    let frame = player.render_rgba(&RenderSettings::new(0.5))?;
    println!("pose rendered at {}x{}", frame.width, frame.height);
    let svg = player.render_svg(false)?;
    assert!(svg.starts_with("<svg"));

    // Same inputs, same advance calls, same pose: a second player is bit-for-bit identical.
    let mut again = anim.player(&machine)?;
    again.set_bool("excited", true)?;
    for _ in 0..30 {
        again.advance(1.0 / 30.0)?;
    }
    assert_eq!(again.render_rgba(&RenderSettings::new(0.5))?, frame);

    // Wrong calls are typed errors, not panics.
    assert!(matches!(player.set_number("excited", 1.0), Err(ApiError::InvalidArgument(_))));
    assert!(matches!(anim.player("no such machine"), Err(ApiError::InvalidArgument(_))));
    Ok(())
}
```

### Layout, Scrollen und Detailstufen

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Layout, scrolling and level of detail on a Player (scroll.json has one scroll region "list").
pub fn layout_and_lod(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let mut player = anim.player(&anim.machine_names()[0])?;

    player.set_canvas_size(100.0, 120.0)?;                 // containers re-solve; renders get this size
    for id in player.scroll_ids() {
        let max = player.scroll_max(&id).unwrap();         // about [0, 80] here: 200 units of content in a 120 unit view
        player.scroll_drag(&id, 0.0, 50.0)?;               // raw pointer offset in px (clamped / rubber-banded)
        assert_eq!(player.scroll_offset(&id), Some([0.0, 50.0]));
        player.scroll_release(&id, 0.0, 900.0)?;           // fling at 900 px/s
        while player.scroll_settled(&id) == Some(false) {
            player.advance(1.0 / 60.0)?;                   // steps the machine and every scroll region
        }
        println!("{id}: max {max:?}, rests at {:?}", player.scroll_offset(&id).unwrap());
    }

    // Pick the coarsest level of detail whose declared error stays within 0.5 px at this screen size.
    player.set_screen_size(100.0, 120.0, 0.5)?;
    println!("level {} (declared bound {} px)", player.lod_level(), player.lod_bound_px());
    player.clear_screen_size();                            // back to the original
    assert_eq!(player.lod_level(), 0);

    // A scroll region whose `input` names a path follows set_value(path, px) instead of a drag.
    player.set_value("ui.scroll", 200.0)?;
    Ok(())
}
```

### Rendern auf mehreren Threads

```rust
use osatura_core::prelude::*;

/// `Animation` and `Player` are `Send + Sync`: read-only calls (`&self`) may run on several threads.
pub fn render_in_parallel() -> Result<(), ApiError> {
    let anim = Animation::demo();
    let frames = anim.frame_count();
    let settings = RenderSettings::new(0.25);
    let (a, s) = (&anim, &settings);                        // shared references are Copy: move them into the threads
    let images: Vec<Result<Image, ApiError>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..frames).map(|f| scope.spawn(move || a.render_rgba(f as f32, s))).collect();
        jobs.into_iter().map(|j| j.join().expect("render thread")).collect()
    });
    // Mutating calls (`&mut self`: set_key, undo, retarget_bvh ...) need exclusive access.
    for (f, img) in images.into_iter().enumerate() {
        assert_eq!(img?, anim.render_rgba(f as f32, &settings)?);   // threads do not change the result
    }
    Ok(())
}
```

### Versionskontrolle und Animations-CI

```rust
use osatura_core::prelude::*;

/// Version control of a project file and animation CI, from Rust.
pub fn history(dir: &std::path::Path) -> Result<(), ApiError> {
    let file = dir.join("hero.json");
    std::fs::write(&file, Animation::demo().to_json()).unwrap();
    let path = file.to_str().unwrap();

    // A tiny helper: re-key the first bone at frame 5 and save the file.
    let edit = |angle: f32| -> Result<(), ApiError> {
        let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
        let bone = a.bone_names()[0].clone();
        a.set_key(&bone, "bone_angle", 5.0, angle, Interpolation::Linear)?;
        std::fs::write(path, a.to_json()).unwrap();
        Ok(())
    };

    vcs_init(path, Some("ana"))?;                           // branch `main`, no commit yet
    vcs_commit(path, Some("base"), None)?;
    vcs_branch(path, "other", None)?;                       // created, not switched to
    edit(20.0)?;
    vcs_commit(path, Some("main edit"), None)?;
    vcs_checkout(path, "other", false)?;                    // DirtyTree if the file has uncommitted changes
    edit(40.0)?;                                            // the same key, another value
    vcs_commit(path, Some("other edit"), None)?;
    vcs_checkout(path, "main", false)?;

    match vcs_merge(path, "other", false, None, None)? {
        VcsMergeOutcome::Conflicted(conflicts) => {         // a normal result, not an error
            for c in &conflicts {
                println!("{} ({}): {}", c.path, c.kind, c.description);
                vcs_resolve(path, &c.path, &VcsChoice::Theirs)?;
            }
            vcs_commit(path, None, None)?;                  // the merge commit
        }
        other => println!("merged without conflicts: {other:?}"),
    }
    for c in vcs_log(path, 10)? {
        println!("{} {} {}", &c.id[..8], c.author, c.message);
    }

    // Animation CI: write a calibrated manifest, record goldens, then verify.
    let init = ci_init(path, None, true)?;                  // never overwrites an existing manifest
    let run = ci_run(&init.manifest)?;                      // verify mode: nothing is re-recorded
    println!("{} checks, {} passed, exit code {}", run.total, run.passed, run.exit_code);
    if run.exit_code == 1 {
        // A failing check is a result, not an Err. After reviewing the report, accept the new look:
        ci_bless(&init.manifest)?;
    }
    Ok(())
}
```

### Mocap, Fuzzing, Tracing, Detailstufen und größenorientierter Export

```rust
use osatura_core::prelude::*;

const RIG: &str = r#"{"format":1,"name":"rig","width":400.0,"height":400.0,"fps":30.0,"frames":10,"background":null,
 "camera":{"position":null,"zoom":1.0,"rotation":0.0},
 "bones":[{"name":"hips","parent":null,"length":20.0,"rest_offset":[200.0,250.0],"rest_angle":-90.0},
          {"name":"spine","parent":"hips","length":40.0,"rest_offset":[20.0,0.0],"rest_angle":0.0},
          {"name":"head","parent":"spine","length":30.0,"rest_offset":[40.0,0.0],"rest_angle":0.0}],
 "ik":[],"constraints":[],"springs":[],"layers":[],"channels":[]}"#;

fn mini_bvh() -> String {
    let mut s = String::from("HIERARCHY\nROOT Hips\n{\n OFFSET 0 100 0\n CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation\n");
    s.push_str(" JOINT Spine\n {\n  OFFSET 0 12 0\n  CHANNELS 3 Zrotation Xrotation Yrotation\n  JOINT Head\n  {\n   OFFSET 0 12 0\n   CHANNELS 3 Zrotation Xrotation Yrotation\n   End Site\n   {\n    OFFSET 0 8 0\n   }\n  }\n }\n}\nMOTION\nFrames: 4\nFrame Time: 0.0333333\n");
    for f in 0..4 {
        s.push_str(&format!("0 100 0 0 0 0 {} 0 0 0 0 0\n", f * 10));
    }
    s
}

/// Motion capture, pose fuzzing, tracing, level of detail and size-targeted export.
pub fn tools() -> Result<(), ApiError> {
    // BVH retargeting: one undoable edit; bones are matched by name and topology.
    let mut rig = Animation::from_json(RIG)?;
    let report = rig.retarget_bvh(&mini_bvh(), &MocapSettings::new().with_smooth(1))?;
    println!("matched {:?}; {} frames on {} bones", report.matched, report.frames, report.bones_used);
    assert!(rig.undo());                                    // the whole retarget is one step

    // Pose fuzzing: where does the rig break?  (deterministic for a given seed)
    let demo = Animation::demo();
    let fuzz = demo.fuzz(50, 1)?;
    println!("{} of {} samples failed (max severity {:.2})", fuzz.failures, fuzz.samples, fuzz.max_severity);
    let _full_report: &str = &fuzz.json;                    // schema "osatura-fuzz-report" v1

    // Bitmap tracing: PNG/JPEG bytes -> SVG, or straight into an animation.
    let png = demo.render_png(0.0, &RenderSettings::new(0.25))?;
    let svg = trace_image_svg(&png, 6)?;
    let traced = Animation::from_image(&png, 6, "traced")?;
    println!("traced SVG: {} bytes; project has {} layers", svg.len(), traced.layer_names().len());

    // Level of detail with a declared error bound, one project JSON per level (level 0 = original).
    for l in demo.lod_levels(0.5, 1.0, 3)? {
        println!("L{}: bound {:.3} px, {} anchors, {} keys", l.level, l.declared_px, l.path_anchors, l.keys);
        let _reduced = Animation::from_json(&l.json)?;
    }

    // Smallest export within a byte budget.
    let fit = demo.fit_to_size(FitFormat::Lottie, 250_000)?;
    println!("met={} {} bytes, worst SSIM {:.3}", fit.met, fit.size, fit.ssim_min);
    Ok(())
}
```

### Wie die Beispiele geprüft werden

Der obige Code ist der Code, der ausgeführt wurde: Diese Datei wird aus den Beispielquellen generiert, die mit `cargo` gegen `osatura-core` kompiliert und ausgeführt wurden (die verwendeten Projektdateien sind die Ausgabe von `osatura demo`, `take-record`, `clip add` und `machine add`, plus ein Projekt mit einem Container und einer Scroll-Region). Die Doc-Tests in `api/` (`cargo test --doc -p osatura-core`: 19 Doc-Tests, 18 auf Fassadenfunktionen in `api/` und das Beispiel auf Crate-Ebene in `lib.rs`; etwa 170 s in einem Debug-Build, weil jeder ein Programm kompiliert) decken die Wrapper für Versionskontrolle und CI im Repository selbst ab.

## Andere Sprachen

Die Fassade ist die einzige Quelle der Wahrheit für die anderen Bindings. Abdeckung:

| Fähigkeit | Rust-Fassade | C-ABI (`osatura.h`) | WebAssembly / JS |
|---|---|---|---|
| Laden, validieren, Info, Knochennamen / Weltmatrizen | ja | `osatura_open`, `_bone_*`, `_frame_count`, ... | `arm_open`, `load()` (keine Knochenmatrizen) |
| RGBA rendern | ja | `osatura_render_rgba` | `arm_render_rgba`, `rgba()` |
| SVG / PNG rendern / Exportformate | ja | nein | SVG ja (`svg()`), PNG / Export nein |
| Keys bearbeiten, Undo / Redo | ja | nein | nein |
| `Player`: Eingaben, Weiterschalten, Zustand, Ereignisse, Rendern | ja | `osatura_player_*` | `arm_player_*`, `startPlayer()`, ... |
| Layout, Scrollen, Detailstufen | ja | `osatura_player_set_canvas`, `_scroll_*`, `_set_screen`, `_lod_level` | `arm_player_set_canvas / layout / scroll_* / set_screen` |
| Werkzeuge, Versionskontrolle, CI | ja | nein | nein |

Die C-ABI-Version bleibt bei 1, weil bisher jede Ergänzung additiv war. Das Wasm-Binary wird in CI gebaut und unter Node getestet, wurde aber nicht in einem Browser ausgeführt; siehe [WASM.md](WASM.md).

## Anhang: Maschinen-JSON und Binding-JSON

Kurze Zusammenfassungen der beiden Projektabschnitte, die der `Player` ausführt (vollständiges Format: [PROJECT_FORMAT.md](PROJECT_FORMAT.md)).

### Maschinen-JSON (Zusammenfassung)

```json
{"name":"hero",
 "inputs":[{"name":"moving","kind":"bool"},{"name":"speed","kind":"number"},{"name":"jump","kind":"trigger"}],
 "layers":[{"name":"body","mask":["arm_*"],"entry":"idle",
   "states":[{"name":"idle","source":{"kind":"clip","clip":"idle"},"blend_in":0.2},
             {"name":"loco","source":{"kind":"blend","input":"speed","children":[
                {"threshold":0,"source":{"kind":"clip","clip":"walk"}},{"threshold":1,"source":{"kind":"clip","clip":"run"}}]}},
             {"name":"jump","source":{"kind":"clip","clip":"jump"},"mode":"once"}],
   "transitions":[{"from":"idle","to":"loco","when":[{"input":"moving"}],"duration":0.25,"ease":{"type":"ease_in_out"}},
                  {"from":"*","to":"jump","when":[{"input":"jump"}],"duration":0.1},
                  {"from":"jump","to":"idle","exit_time":1.0}]}]}
```

Bedingungen sind `{"input","op":"eq|ne|lt|le|gt|ge","value"}` (Standardwerte `eq`, `1`; alle müssen erfüllt sein). `from: "*"` trifft auf jeden Zustand außer dem Ziel zu. Der erste passende Übergang in Deklarationsreihenfolge gewinnt. `exit_time` ist der abgespielte Anteil (0..1) des Zustands. Quellen: `clip` (ein Eintrag aus `Project.clips`), `segment` (`start`, `end` Frames der gekeyten Replace-Modus-Kanäle des Projekts), `region` (ein Marker mit Ende), `blend` (1-D, zwei oder mehr Blattquellen bei aufsteigenden Schwellenwerten). Unbekannte Schlüssel in einem Übergang oder einer Bedingung sind ein Fehler. CLI: `machine add|list|remove|validate`, `machine-sim <project> <script.json> <out_dir|out.json>` (Skript: `{"machine":"hero","duration":2,"events":[{"t":0.5,"set":{"moving":true},"trigger":["jump"],"value":{"player.health":40}}]}`).

### Binding-JSON (Zusammenfassung)

```json
{"name":"hp","path":"player.health","expr":"value / 100","remap":{"in_min":0,"in_max":1,"out_min":-30,"out_max":30},
 "smooth":0.15,"target":{"kind":"prop","target":"arm_up","prop":"bone_angle"}}
```

Pipeline: Wert, dann `expr` (Variable `value`; die Ausdrucksfunktionen der Engine, Quelltext höchstens 4 KB und Verschachtelung höchstens 64), dann `remap` (standardmäßig begrenzt; `curve` glättet per Easing), dann Glättung (Zeitkonstante in Sekunden), dann das Ziel: `{"kind":"prop",...}` (jede Kanaleigenschaft, einschließlich `layer_opacity` und `layer_switch`) oder `{"kind":"input","machine","input"}` (ein Trigger-Eingang löst bei der steigenden Flanke aus). CLI: `bind add|list|remove`, `bind-eval`.

### Layout und Normal Maps pro Ebene

Layout-Container und Scroll-Regionen sind die Projektfelder `containers` / `scrolls` (weggelassen, wenn leer): CLI `container add|list|remove|solve`, `scroll add|list|remove|eval`, `layout-preview <project> <out.svg> --size WxH` und `--size WxH` bei `render`, `frames` und `export` (Container werden für diese Canvas-Größe neu gelöst; die Datei bleibt unverändert). `Layer.normal_map` (`data` Base64-PNG oder `path`, `fit` stretch / tile / exact, `strength`, `green_down`) speist den Nachbearbeitungseffekt `relight` für die Pixel, die diese Ebene oder ein Kind ohne eigene Map abdeckt (CLI `layer-normal`, `fx ... --layer-normals`); sie ist nicht Teil der Fassade.
