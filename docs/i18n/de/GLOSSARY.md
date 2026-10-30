<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# OSATURA-Glossar

Fachbegriffe, die im Code und in der Dokumentation verwendet werden, in alphabetischer Reihenfolge (der englischen Begriffe). „Siehe“ verweist auf das
Modul (in `osatura-core/src`, sofern nicht anders angegeben) oder das Dokument, in dem der Begriff implementiert oder
spezifiziert ist. Dokumente: [PROJECT_FORMAT.md](PROJECT_FORMAT.md), [ARCHITECTURE.md](ARCHITECTURE.md).
Quellen für Techniken sind in `CLEANROOM.md` aufgeführt.

**Aim constraint (Aim-Constraint).** Ein Constraint, der einen Knochen so dreht, dass seine +x-Achse auf den Kopf eines anderen Knochens zeigt,
plus einen Offset. Siehe `rig`; PROJECT_FORMAT Abschnitt 4.3.

**Anchor (Anker).** Ein Punkt auf einem Bézier-Pfad mit einer Position und optionalen Eingangs- und Ausgangs-Handles; er kann auch
Knochengewichte tragen. Siehe `path`; PROJECT_FORMAT Abschnitt 6.

**Angle limits (Winkelgrenzen).** Minimum und Maximum der animierten Rotation pro Knochen, in Grad, die nach den
Kanälen und nach jeder IK-Kette erzwungen werden. Siehe `rig`, `engine`.

**API version (API-Version).** Die Version der stabilen Rust-Oberfläche (`api` und `prelude`), derzeit 1.4.0,
durch einen Snapshot-Test geprüft. Nicht dasselbe wie das Dateifeld `format` oder die C-`ABI_VERSION`. Siehe `api`.

**Asset.** Eine Datei, auf die ein Projekt verweist (Bild, Normal Map, Ton). Pfade müssen relativ sein und im
Projektverzeichnis bleiben, sofern nicht `--allow-external-assets` angegeben ist. Siehe `assets`; PROJECT_FORMAT
Abschnitt 18.

**Bake.** Werte über die Zeit berechnen und als gewöhnliche Kanäle oder Keys speichern, sodass sie das berechnende Werkzeug
nicht mehr brauchen. Verwendet von Physik, Clips, Magnet, Footlock, Zustandsautomaten und mehr. Siehe
ARCHITECTURE Abschnitt 4.

**Bend (Beugerichtung).** Das Vorzeichen (+1 oder -1), das bestimmt, in welche Richtung eine Zwei-Knochen-IK-Kette knickt, etwa Ellbogen nach innen oder außen.
Siehe `rig`.

**Bézier curve (Bézier-Kurve).** Eine parametrische kubische Kurve, definiert durch zwei Anker und deren Handles. Zum Füllen und Messen in Liniensegmente
zerlegt (de-Casteljau-Unterteilung). Siehe `path`.

**Binding.** Eine Laufzeitregel, die einen externen Wert (eine Spielvariable, einen Slider) über einen Ausdruck, ein Remap und Glättung auf eine Eigenschaft oder eine Eingabe des Zustandsautomaten abbildet. Siehe `binding`.

**Blend mode (Mischmodus).** Wie sich die Farbe einer Ebene mit dem darunterliegenden Inhalt verbindet (Multiplizieren, Abwedeln, Überlagern
und so weiter). Die Formeln folgen W3C Compositing and Blending Level 1. Siehe `raster`, `fraster`.

**Blend tree.** Eine eindimensionale Mischung mehrerer Clips, gesteuert durch eine Zahleneingabe, verwendet innerhalb eines
Zustandsautomaten. Siehe `machine`.

**Blob.** Ein Kreis einer Liquid-Ebene (Metaball). Siehe `fx`.

**Bone (Knochen).** Ein benanntes Segment des Skeletts mit Kopfposition, Länge, Ruhewinkel und optionalem
Parent. Knochen steuern per Skinning verformte Geometrie und zugewiesene Ebenen. Siehe `rig`; PROJECT_FORMAT Abschnitt 4.

**Boolean layer (Boolean-Ebene).** Eine Ebene, die Vektorpfade per Vereinigung, Schnitt, Differenz oder XOR kombiniert. Der
erste Pfad ist das Subjekt, die übrigen werden als Clip vereinigt. Siehe `fx`.

**Bounce (Easing).** Eine Easing-Kurve eines fallenden und zurückprallenden Balls mit der Restitution 0,6 und drei
Aufprallen. Eine eigenständige Herleitung aus dem freien Fall. Siehe `anim`.

**Bounding box (Begrenzungsrahmen).** Das kleinste achsenparallele Rechteck um eine Geometrie; verwendet für Ausrichtung, Auswahl und
die Anpassung von Normal Maps. Siehe `layout`, `pick`.

**Cage deformer (Cage-Deformer).** Ein Deformer, bei dem Inhalt innerhalb eines geschlossenen Polygons den Eckpunkten des Polygons
folgt, und zwar mit Mean-Value-Koordinaten. Siehe `cage`; auch MVC.

**Catmull-Rom spline (Catmull-Rom-Spline).** Ein kubischer Spline, der durch seine Kontrollpunkte verläuft. Das Easing `smooth` verwendet
eine ungleichförmige Version durch benachbarte Keys. Siehe `anim`, `liquid` smoothing in `fx`.

**Channel (Kanal).** Ein animierter Wert: ein Ziel, eine Eigenschaft, eine Quelle (Keys, Ausdruck, Noise oder gesteuert) sowie
ein Mischmodus und ein Gewicht. Kanäle laufen in Listenreihenfolge. Siehe PROJECT_FORMAT Abschnitt 8.

**CIE76.** Der ursprüngliche Farbunterschied, der euklidische Abstand in CIE L*a*b*. Günstig, weniger genau
als CIEDE2000. Siehe `percep`.

**CIEDE2000.** Die CIE-2000-Formel für Farbunterschiede (CIE TR 142-2001), implementiert wie bei
Sharma, Wu und Dalal (2005) abgedruckt und gegen deren veröffentlichte Testpaare geprüft. Von der Animations-CI verwendet, um
sichtbare Änderungen zu beurteilen. Siehe `percep`, `ci`; `docs/CI.md`.

**Clean room (Clean Room).** Die Entwicklungsregel, dass Code nach öffentlichen Standards und Lehrbuchmethoden geschrieben wird,
ohne kopierten Code und ohne Nachbildung der Formate oder Namen anderer Werkzeuge. Jede Technik wird in
`CLEANROOM.md` protokolliert.

**Clip.** Ein wiederverwendbarer Take aus Animationsspuren, der mit Geschwindigkeit, Loops,
Gewicht, Überblendungen und einem Mischmodus auf der Timeline platziert werden kann. Siehe `clipstack`; PROJECT_FORMAT Abschnitt 11.1.

**Cloth (Stoff).** Ein Grid-Deformer, dessen Eckpunkte sich genau um ihre eigenen Offsets bewegen. Auch die Simulation mit positionsbasierter
Dynamik, die diese Offsets bakt. Siehe `cloth`.

**Constraint.** Eine nach der IK angewendete Regel, die einen Knochen einschränkt: Aim, Copy Rotation oder Pin. Siehe `rig`.

**Container (layout) (Container, Layout).** Eine eindimensionale Layoutgruppe im Flexbox-Stil mit Abstand, Innenabstand, Ausrichtung des Inhalts,
Ausrichtung der Elemente und Umbruch, die Ebenen und andere Container positioniert. Siehe `containers`.

**Copy-rotation constraint (Copy-Rotation-Constraint).** Ein Constraint, der einen Knochen der Weltrotation eines anderen Knochens angleicht. Siehe
`rig`.

**CRDT.** Conflict-free replicated data type: eine Struktur, die Replikate unabhängig voneinander bearbeiten und
ohne Koordination zum selben Ergebnis zusammenführen können. OSATURA verwendet ein operationsbasiertes CRDT. Siehe `collab`;
ARCHITECTURE Abschnitt 10. Verwandt: LWW-Register, RGA, Lamport-Uhr, Vektoruhr, Tombstone.

**Cubic Bézier easing (kubisches Bézier-Easing).** Eine Easing-Kurve im CSS-Stil, definiert durch zwei Kontrollpunkte zwischen (0,0) und
(1,1). Siehe `anim`.

**Deformer.** Ein Formmodifikator, der im Ruheraum auf eine Vektor- oder Bildebene angewendet wird: Lattice, Spine, Pin,
Cage oder Cloth. Siehe PROJECT_FORMAT Abschnitt 7.1.

**Delaunay triangulation (Delaunay-Triangulierung).** Eine Triangulierung von Punkten, bei der kein Punkt im Umkreis eines Dreiecks
liegt, was dünne Dreiecke vermeidet. Für Bild-Meshes mit dem Bowyer-Watson-Algorithmus aufgebaut.
Siehe `meshtrace`.

**Depth of field (DoF) (Tiefenschärfe).** Unschärfe, die mit dem Abstand des `parallax` einer Ebene vom Kamera-`focus`
zunimmt. Wird nur von den Befehlen `dof` und `dof-export` angewendet. Siehe `dof`.

**Determinism (Determinismus).** Die Eigenschaft, dass dieselbe Eingabe dieselbe Ausgabe liefert. Die Frame-Auswertung ist rein;
das Rendering ist auf einem Rechner und einem Build byte-identisch, über Plattformen hinweg nicht verifiziert. Siehe
ARCHITECTURE Abschnitt 6.

**Dope sheet (Dope Sheet).** Eine Timeline-Ansicht, die Keys als Markierungen pro Kanal zeigt und sie zeitlich bearbeitet. Siehe
`timeline`, `keyops`.

**Driven key (gesteuerter Key).** Ein Kanal, dessen Eingabe der Wert einer anderen Eigenschaft statt der Zeit ist, zum Beispiel vom Ellbogen-
winkel zur Größe einer Ausbeulung. Siehe `drivers`; PROJECT_FORMAT Abschnitt 8.1.

**Easing.** Die Formungsfunktion, die auf das Segment angewendet wird, das an einem Key beginnt. Siehe `anim`; PROJECT_FORMAT
Abschnitt 8.4.

**Editor.** Das Objekt, das ein Projekt und seinen Rückgängig-Verlauf besitzt und Ops anwendet. Siehe `edit`;
ARCHITECTURE Abschnitt 5.

**Emitter.** Eine Ebene, die Partikel als geschlossene Funktion der Zeit erzeugt. Siehe `fx`.

**Engine.** Die validierte, namensaufgelöste Form eines Projekts, die Frames auswertet. Siehe `engine`.

**Envelope (collab) (Envelope, Kollaboration).** Die Nachricht, die Ops zwischen Replikaten transportiert: Replikat-ID, Sequenznummer,
Lamport-Zeit, Abhängigkeiten und Operationen. Siehe `collab`.

**Evaluation (Auswertung).** Die Berechnung von Pose, Geometrie und Kamera für ein Frame aus einem Projekt. Eine reine Funktion
des Frames. Siehe PROJECT_FORMAT Abschnitt 16.

**FABRIK.** Forward And Backward Reaching Inverse Kinematics: ein iterativer IK-Solver, der die Kette abwechselnd
von der Spitze zum Ziel zieht und sie dann an der Wurzel neu verankert. Verwendet für Ketten mit
mehr als zwei Knochen. Siehe `rig`.

**FK.** Vorwärtskinematik: die Berechnung der Welttransformationen von Knochen aus den Transformationen ihrer Parents und
ihren eigenen Winkeln und Offsets. Siehe `rig`, `engine`.

**Format version (Formatversion).** Das ganzzahlige Feld `format` eines Projekts, derzeit 1. Neuere Dateien werden abgelehnt.
Siehe PROJECT_FORMAT Abschnitt 19.

**Frame.** (1) Ein Zeitschritt der Timeline, `0..frames-1`. (2) Die Struktur `Frame`: das Ergebnis der
Auswertung eines Schritts (Knochen, Geometrie, Kamera, Flags). Siehe `engine`.

**Glue.** Eine Verknüpfung, die Eckpunkte eines Meshes den Eckpunkten eines anderen folgen lässt (Snap, starr oder mit Offset). Siehe
`links`.

**Gradient handle (Verlaufs-Handle).** Ein Kontrollpunkt eines linearen oder radialen Verlaufs, der mit
`grad_dx` und `grad_dy` animiert werden kann. Siehe PROJECT_FORMAT Abschnitt 8.2.

**Group layer (Gruppenebene).** Eine Ebene, die andere Ebenen enthält und ihre Transformation an sie weitergibt. Siehe `project`.

**Handle.** Ein Bézier-Kontrollpunkt, der an einem Anker hängt. Absolute Koordinaten in der Datei. Siehe `path`.

**HOG.** Histogram of oriented gradients, ein Merkmal lokaler Form (Dalal und Triggs 2005), das vom Gesichts-
detektor verwendet wird. Siehe `facedet`.

**IK.** Inverse Kinematik: die Berechnung von Knochenwinkeln, sodass die Spitze einer Kette ein Ziel erreicht. Siehe `rig`.
Verwandt: FABRIK, Soft-IK, Stretch-IK, Bend.

**IK mix (IK-Mix).** Die Überblendung zwischen der FK-Pose (0) und dem IK-Ergebnis (1) für eine Kette. Animiert mit `ik_mix`.
Siehe PROJECT_FORMAT Abschnitt 4.2.

**Image mesh (Bild-Mesh).** Ein Grid- oder freies Dreiecks-Mesh über einer Bildebene, verwendet für Knochengewichte und Deformer.
Siehe PROJECT_FORMAT Abschnitt 5.8.

**Keyframe (key) (Keyframe, Key).** Ein Tripel (Frame, Wert, Easing) auf einer Spur. Das Easing formt das Segment, das am Key
beginnt. Siehe `anim`.

**L*a*b*.** Der CIE-Farbraum von 1976, so entworfen, dass Abstände wahrgenommene Unterschiede annähern. Siehe
`percep`.

**Lamport clock (Lamport-Uhr).** Ein logischer Zähler, der Ereignisse in einem verteilten System ohne gemeinsame
Uhr ordnet (Lamport 1978). Siehe `collab`.

**Lattice.** Ein Deformer mit einem Gitter aus Kontrollpunkten über einem Rechteck, dessen Offsets den Inhalt
durch Bézier-Bernstein-Mischung verbiegen. Siehe PROJECT_FORMAT Abschnitt 7.1.

**Layer (Ebene).** Ein Knoten im Szenenbaum: Vektor, Gruppe, Switch, Liquid, Boolean, Emitter oder Bild. Siehe
PROJECT_FORMAT Abschnitt 5.

**Layer effect (Ebeneneffekt).** Ein Filter pro Ebene (Weichzeichnung, Schlagschatten, Tönung, Füllung), der auf einer Offscreen-Zeichenfläche gerendert wird.
Siehe `layerfx`.

**Level of detail (LOD) (Detailstufe).** Eine vereinfachte Version eines Projekts, erzeugt unter einer deklarierten Fehlergrenze
in Pixeln, für schnelle Vorschau oder kleine Ausgaben. Siehe `lod`, `osatura lod`.

**Linter.** Die beratende Prüfung (`osatura lint`), die Probleme der Animationsqualität mit Codes
wie L011 und L012 meldet. Getrennt von der Validierung. Siehe `lint`.

**Liquid layer (Liquid-Ebene).** Eine Ebene aus Metaballs, deren Umriss die Iso-Kontur eines Summen-Einfluss-Feldes ist. Siehe
`fx`.

**LWW register (LWW-Register).** Last-writer-wins-Register: eine CRDT-Zelle, in der der Schreibvorgang mit dem höchsten logischen
Zeitstempel überlebt. Verwendet für Objektschlüssel. Siehe `collab`.

**Magnet.** Ein Bake-Werkzeug, das Positionen mit Smoothstep-Abfall zu einem Punkt zieht. Siehe `magnet`.

**Marching squares.** Ein Algorithmus, der aus einem abgetasteten Skalarfeld auf einem Gitter Konturlinien extrahiert.
Verwendet für Liquid-Umrisse und das Tracing von Bitmaps. Siehe `fx`, `trace`, `meshtrace`.

**Marker.** Ein benanntes Frame oder ein Frame-Bereich auf der Timeline. Zustandsautomaten können einen Markerbereich als
Quelle verwenden. Siehe `keyops`; PROJECT_FORMAT Abschnitt 10.2.

**Mask (Maske).** Eine Ebene, deren Alpha die Sichtbarkeit einer anderen Ebene begrenzt; kann invertiert sein. Eine ausgeblendete Ebene kann
trotzdem eine Maske sein. Siehe `raster`.

**Merge (CRDT) (Merge, CRDT).** Das Zusammenführen der Zustände zweier Replikate, sodass beide zum selben Dokument konvergieren. Siehe
`collab`.

**Mesh lock.** Eine Verknüpfung, die ein folgendes Mesh einem Host-Mesh mit einer Stärke von 0..1 folgen lässt. Siehe `links`.

**Metaball.** Ein Blob, dessen Feldeinfluss sich mit Nachbarn mischt, sodass nahe Blobs verschmelzen. Siehe Liquid-
Ebene.

**Morph (blend shape) (Morph, Blend Shape).** Ein alternativer Ankersatz für einen Pfad, additiv mit einem Gewicht angewendet. Siehe
PROJECT_FORMAT Abschnitt 6.1.

**MVC.** Mean-Value-Koordinaten (Floater 2003): verallgemeinerte baryzentrische Koordinaten für einen Punkt innerhalb
eines Polygons, verwendet vom Cage-Deformer. Siehe `cage`.

**Non-destructive (nicht destruktiv).** Vorhandene Arbeit bleibt unangetastet. Baker hängen Kanäle an, statt Keys neu zu schreiben, und
der Editor zeichnet inverse Ops auf. Siehe ARCHITECTURE Abschnitte 4 und 5.

**Normal map (Normal Map).** Ein Bild, das die Oberflächenrichtung als RGB speichert, verwendet vom Effekt `relight` und von
der Beleuchtung pro Ebene. Siehe `postfx`; PROJECT_FORMAT Abschnitt 7.3.

**Op.** Eine Bearbeitung, ausgedrückt als `set`, `insert`, `remove` oder `batch` auf einem RFC-6901-JSON-Pointer; sie hat eine
Umkehrung für Rückgängig. Siehe `edit`.

**OT.** Operational Transformation: eine Alternative zu CRDTs, die gleichzeitige Operationen gegeneinander umschreibt,
meist mit einem zentralen Server. OSATURA verwendet sie nicht; es nutzt ein CRDT mit einem zentralen
Sequenzer. Siehe ARCHITECTURE Abschnitt 10.

**Parallax.** Ein Faktor pro Ebene, der die Kamerabewegung skaliert, sodass entfernte Ebenen sich weniger bewegen. Auch der
Tiefenwert für die Tiefenschärfe. Siehe PROJECT_FORMAT Abschnitt 10.1.

**Particle (Partikel).** Ein kurzlebiger Punkt, den ein Emitter ausgibt; seine Position ist eine geschlossene Funktion der
Zeit. Siehe `fx`.

**Path (target) (Pfad, Ziel).** Der durch Schrägstriche getrennte Name einer verschachtelten Ebene, etwa `parent/child`; Unterziele verwenden
`::` wie in `layer::path::morph`. Siehe PROJECT_FORMAT Abschnitt 5.

**Path (vector) (Pfad, Vektor).** Eine Folge von Ankern, offen oder geschlossen, mit optionaler Füllung, Kontur und Morphs. Siehe
`path`.

**Pin constraint (Pin-Constraint).** Ein Constraint, der den Kopf eines Knochens auf den Kopf eines anderen Knochens schiebt. Siehe `rig`.

**Pin warp (Pin-Warp).** Ein Deformer, bei dem gezogene Pins die Grafik innerhalb eines Radius weich bewegen, mit einer
kompakt getragenen radialen Basisfunktion (Wendland 1995). Siehe `pinwarp`.

**Player.** Der Laufzeit-Wrapper in der Fassade, der einer Animation Zustandsautomaten und Bindings hinzufügt.
Siehe `api`.

**Preorder.** Die Besuchsreihenfolge des Ebenenbaums: ein Knoten, dann seine Kinder. Die Auswertung nutzt sie, damit
Parents zuerst berechnet werden.

**Proxy.** Ein Bild mit reduziertem Detail, das beim Scrubben verwendet wird und aus einer Detailstufe aufgebaut ist. Siehe `scrub`, `lod`.

**QEM.** Quadric error metric (Garland und Heckbert): ein Maß für den geometrischen Fehler, der durch das Zusammenfallen eines Eckpunkts entsteht, verwendet zur Mesh-Vereinfachung. Siehe `lod`.

**Raster.** Ein Pixelbild oder das Modul, das ein Frame mit tiny-skia zu Pixeln rendert. Siehe `raster`.

**RDP.** Ramer-Douglas-Peucker: rekursive Linienvereinfachung, die Punkte behält, die weiter als eine Toleranz
von der Sehne entfernt sind. Siehe `trace`, `sizefit`.

**Reference layer (Referenzebene).** Eine als `reference` markierte Ebene; ausgeblendet, sofern der Editor keine Referenzen zeigt. Siehe
PROJECT_FORMAT Abschnitt 5.1.

**Remap.** Eine affine Abbildung eines Bereichs, mit optionaler Kurve und Begrenzung, angewendet auf einen Binding-Wert.
Siehe `binding`.

**Replica (collab) (Replikat, Kollaboration).** Die Kopie des Dokuments eines Teilnehmers im CRDT. Siehe `collab`.

**Rest pose (Ruhepose).** Die Knochentransformationen ohne angewendete Animation; Skin-Matrizen sind `world * inverse(rest
world)`. Siehe `rig`.

**RGA.** Replicated growable array (Roh et al. 2011): ein Sequenz-CRDT, in dem jedes Element eine eindeutige
ID hat und Löschungen Tombstones hinterlassen. Verwendet für JSON-Arrays. Siehe `collab`.

**Rig.** Das Skelett und seine Helfer: Knochen, IK-Ketten, Constraints und Federn. Siehe PROJECT_FORMAT
Abschnitt 4.

**Scrub (Scrubben).** Das Ziehen des Abspielkopfs; das Studio cacht Frames nach Revision und verwendet Proxys. Siehe `scrub`.

**Skew (Scherung).** Ein Scherwinkel in der Transformation einer Ebene. Siehe PROJECT_FORMAT Abschnitt 5.1.

**Skin (animation) (Skin, Animation).** Eine benannte Variante von Ebenen, Knochen, Constraints oder IK, die in einer
Gruppe eingeschaltet werden kann, etwa Kostüme. Siehe `skins`.

**Skinning.** Das Verformen von Geometrie durch Mischen der Transformationen von Knochen, gewichtet pro Anker oder Eckpunkt.
OSATURA normalisiert die Gewichte und behandelt leere Gewichte als statisch. Siehe `engine`, `weights`.

**Soft IK.** Eine IK-Option, die die Streckung nahe der vollen Reichweite abfedert, um ein Einrasten in die gerade Lage zu vermeiden. Bereich 0 bis
0,9. Siehe `rig`.

**SPDX.** Software Package Data Exchange: ein Standard, um die Lizenz einer Datei anzugeben, wie in
<!-- REUSE-IgnoreStart -->
`SPDX-License-Identifier: MPL-2.0`. Jede Quelldatei trägt eine solche Angabe. Siehe `LICENSING.md`.
<!-- REUSE-IgnoreEnd -->

**Spine deformer (Spine-Deformer).** Ein Deformer, der Geometrie entlang einer Kurve biegt, indem er Anker auf eine Ruhe-
kurve projiziert und sie an der deformierten Kurve neu befestigt. Siehe PROJECT_FORMAT Abschnitt 7.1.

**Spring (bone) (Feder, Knochen).** Eine gedämpfte Feder zweiter Ordnung, die die Rotation eines Knochens nachlaufen und überschwingen lässt; beim
Laden gebakt, sodass jedes Frame ausgewertet werden kann. Siehe PROJECT_FORMAT Abschnitt 4.4.

**SSIM.** Structural similarity index (Wang et al. 2004): ein Maß für die Bildqualität aus lokalen Mittelwerten,
Varianzen und der Kovarianz. OSATURA verwendet eine Blockversion. Siehe `percep`.

**Stage (Bühne).** Der Koordinatenraum des Ausgabebilds: Pixel, +x nach rechts, +y nach unten.

**State machine (Zustandsautomat).** Ein Graph aus Animationszuständen mit Übergängen, die von Eingaben gesteuert werden. Es gibt eine gebakte
Version mit Ereignislog und eine Laufzeitversion. Siehe `statemachine`, `machine`.

**Step easing (Step-Easing).** Ein Easing, das den Wert des Keys bis zum nächsten Key hält. Verwendet für Switch-Ebenen. Siehe
`anim`.

**Stretch IK.** Eine IK-Option, die die Kette bis zu einer Grenze (1..4) skaliert, um ein entferntes Ziel zu erreichen. Siehe
`rig`.

**Stroke (Kontur).** Der Umriss eines Pfads: Farbe, Breite, Cap, Join, Strichelung und ein optionales Breitenprofil. Siehe
PROJECT_FORMAT Abschnitt 6.3.

**Switch layer (Switch-Ebene).** Eine Ebene, die genau eines ihrer Kinder zeigt, gewählt über einen Index-Kanal. Verwendet für
Frame-für-Frame-Animation und Mundformen. Siehe PROJECT_FORMAT Abschnitt 5.4.

**Take.** Ein benannter Satz von Spuren, der den Inhalt eines Clips bildet. Siehe `clipstack`.

**Tombstone.** Eine Markierung, die in einem CRDT-Array anstelle eines gelöschten Elements zurückbleibt, damit gleichzeitige Operationen
weiterhin platziert werden können. Die Verdichtung entfernt Tombstones, die jedes Replikat gesehen hat. Siehe `collab`.

**Track (Spur).** Die Liste der Keys eines Kanals, mit einem Extrapolationsmodus. Siehe `anim`.

**Transform (Transformation).** Position, Rotation, Skalierung, Scherung und Anker einer Ebene; die lokale Matrix ist
`T(position) R Shear S T(-anchor)`. Siehe PROJECT_FORMAT Abschnitt 5.1.

**Vector clock (Vektoruhr).** Ein Zählervektor pro Replikat, der angibt, ob ein Ereignis vor einem anderen stattfand oder
ob sie nebenläufig sind (Fidge, Mattern). Siehe `collab`.

**Weights (bone) (Gewichte, Knochen).** Der Einfluss jedes Knochens auf einen Anker oder Eckpunkt. Sie können gemalt oder per
Diffusion berechnet werden. Siehe `weights`.

**Wobble.** Eine deterministische Noise-Verschiebung, die zur Geometrie einer Ebene addiert wird, mit Amplitude, Skalierung, Rate
und Seed. Siehe PROJECT_FORMAT Abschnitt 5.1.

**Zhang-Suen thinning (Zhang-Suen-Ausdünnung).** Ein paralleler Algorithmus, der eine binäre Form auf ein Skelett mit Pixelbreite eins reduziert. Verwendet
von autotrace für das Tracing von Mittellinien. Siehe `autotrace`.
