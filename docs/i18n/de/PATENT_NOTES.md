# Patenthinweise

**Dies ist keine Rechtsberatung und kein Freedom-to-Operate-Gutachten.** Das Dokument hält fest, was NICHT geprüft wurde, damit niemand das Fehlen einer Prüfung mit einer Freigabe verwechselt. Die Lizenzen von OSATURA (MPL-2.0 für die Bibliotheken, AGPL-3.0-or-later für die CLI) enthalten Patentlizenzen der Mitwirkenden für den eigenen Code des Autors, sagen aber nichts über Patente Dritter. Eine Patentrecherche durch einen Fachmann wurde nicht durchgeführt.

## Noch ausstehende Prüfungen

| Bereich | Warum er relevant sein kann | Status |
|---|---|---|
| Gesichtsgesteuertes Puppenspiel (`faceparam`, `facedet`, `liveinput`: Gesichts-Landmarken oder Verhältniswerte steuern Rig-Parameter) | Performance-gesteuerte Animation von 2D-Figuren ist ein Feld mit vielen aktuellen kommerziellen Anmeldungen. | Nicht geprüft. Der Code verwendet veröffentlichte Lehrbuchmaße (Eye Aspect Ratio, One-Euro-Filter, HOG + logistische Regression); das schließt Ansprüche auf die Kombination nicht aus. |
| Zustandsautomaten und Datenbindung (`statemachine`, `runtime_machine`, `bind`) | Laufzeitumgebungen für interaktive Animation mit Zustandsautomaten, Blend Trees, Eingaben und View-Model-Bindung sind ein kommerzielles Funktionsfeld. | Nicht geprüft. Eigener Entwurf des Datenmodells, keine Patentfreigabe. |
| FABRIK, Mean-Value-Koordinaten, Bounded Biharmonic Weights (`rig`, `cage`, `weights`) | Veröffentlichte Algorithmen; einige Autoren und Arbeitgeber haben Patente rund um Deformations- und Skinning-Techniken angemeldet. | Nicht geprüft. FABRIK (Aristidou und Lasenby, 2011) und MVC (Floater, 2003) werden nach den Veröffentlichungen verwendet; das Modul weights nutzt einfache harmonische Diffusion, nicht den veröffentlichten Bounded-Biharmonic-Löser. |
| Stencil-then-Cover-Pfadfüllung (`osatura-gpu`) | Die Technik stammt aus der Forschung zum GPU-Pfad-Rendering und aus Herstellererweiterungen. | Nicht geprüft. Hier nur über die Standard-API von wgpu mit einem einfachen Stencil-Puffer verwendet. |
| Video- und Audio-Codecs über ffmpeg (`export` .mp4 / .webm, `soundtrack`) | H.264 und AAC werden von Lizenzpools abgedeckt (MPEG LA / Via LA und andere). VP9 und AV1 werden als lizenzgebührenfrei beworben, doch für beide wurden Patentpool- und Einzelpatentansprüche angekündigt. | Nicht bewertet. VP9/AV1 nicht als freigegeben behandeln. |

## ffmpeg
ffmpeg ist ein **externer Subprozess**, den der Nutzer bereitstellt. OSATURA bindet es nicht ein, liefert es nicht aus und darf es in keinem Release mitliefern: Eine Mitlieferung würde den Verteiler für die eigene Lizenz von ffmpeg (LGPL/GPL je nach Build) und für die oben genannten Codec-Lizenzen verantwortlich machen. Jede Codec-Option ist die eigene Entscheidung des Nutzers auf seiner eigenen Installation.

## Vor einem kommerziellen Release
Lass die oben genannten Bereiche von einem Patentanwalt prüfen, und zwar in den Rechtsordnungen, in denen die Software vertrieben wird, und halte das Ergebnis hier und in CLEANROOM.md (Regel 5) fest.
