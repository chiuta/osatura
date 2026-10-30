<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Translated documentation

The English documents are the reference. Translations exist in **Romanian (ro), French (fr), Italian (it), Spanish (es), Portuguese (pt), German (de) and Hungarian (hu)**. If a translation and the English original differ, the English original is authoritative.

| Language | Index | README |
|---|---|---|
| Română | [ro/INDEX.md](ro/INDEX.md) | [README.ro.md](../../README.ro.md) |
| Français | [fr/INDEX.md](fr/INDEX.md) | [README.fr.md](../../README.fr.md) |
| Italiano | [it/INDEX.md](it/INDEX.md) | [README.it.md](../../README.it.md) |
| Español | [es/INDEX.md](es/INDEX.md) | [README.es.md](../../README.es.md) |
| Português | [pt/INDEX.md](pt/INDEX.md) | [README.pt.md](../../README.pt.md) |
| Deutsch | [de/INDEX.md](de/INDEX.md) | [README.de.md](../../README.de.md) |
| Magyar | [hu/INDEX.md](hu/INDEX.md) | [README.hu.md](../../README.hu.md) |

The landing page `index.html` (built by `scripts/build-site.py`) loads these translations from `site/data/docs.<lang>.js` and shows them in the language chosen in the page. Run the build script after changing a document or a translation.

## Layout

* `README.<lang>.md` translates `README.md`.
* `docs/i18n/<lang>/<NAME>.md` translates `docs/<NAME>.md` or a root document (`CHANGELOG.md`, `CLEANROOM.md`, `CONTRIBUTING.md`, `FEATURES.md`, `LICENSING.md`, `ROADMAP.md`, `TRADEMARK.md`).
* `THIRD_PARTY_NOTICES.md` (generated licence texts) is not translated.

## Rules for translators

A translation keeps the **structure** of its source exactly. `python3 scripts/i18n.py check` verifies these rules.

1. Translate prose, headings, table cells, list items, image alt text and link text. Keep the same number of headings, in the same order and at the same levels.
2. Copy every fenced code block **byte for byte**, including comments inside it. Do not translate command output.
3. Keep every inline code span (`` `like this` ``) unchanged: commands, options, identifiers, file names, JSON keys, field values, error messages.
4. Keep every link target and image path **exactly as in the source**. Translate only the link text. (`python3 scripts/i18n.py fix-links` rewrites the targets afterwards for the translated location and the translated heading anchors.)
5. Keep the first two lines (the SPDX and copyright comments) unchanged.
6. Keep proper names (OSATURA, Rust, Lottie, glTF, ffmpeg, ...), standard and licence names (MPL-2.0, AGPL-3.0-or-later, CC-BY-4.0, DCO), crate names, and ids such as `L-01`, `R-045`, `I-07`.
7. In `LIMITATIONS.md` keep the words `High`, `Medium` and `Low` in the *Severity* column exactly as written (they are values that tools read).
8. Do not add, remove or reorder content. Do not add translator notes. Do not summarise.
9. Dates, numbers and units stay as in the source; month names may be localised.
10. Use the glossary below so that all documents use the same term.
11. Address the reader in the same register in every document of a language: ro informal *tu*, fr *vous*, it *tu*, es *tú*, pt *você*, de *du*, hu informal *te* (imperative or impersonal forms wherever the English does not address the reader directly).

## Glossary

Terms not listed here follow the usual usage of the language; internationally used English terms (rigging, skinning, seed, blend tree, dope sheet, onion skin, take, crate, ABI, CLI, JSON, lint, diff, merge) are kept as they are.

| English | ro | fr | it | es | pt | de | hu |
|---|---|---|---|---|---|---|---|
| bone | os | os | osso | hueso | osso | Knochen | csont |
| rig | rig | rig | rig | rig | rig | Rig | rig |
| skin weights | greutăți de skin | poids de skin | pesi di skin | pesos de skin | pesos de skin | Skin-Gewichte | skin-súlyok |
| deformer | deformator | déformateur | deformatore | deformador | deformador | Deformer | deformer |
| constraint | constrângere | contrainte | vincolo | restricción | restrição | Constraint | kényszer |
| spring | arc | ressort | molla | muelle | mola | Feder | rugó |
| key / keyframe | cheie | clé | chiave | clave | chave | Key | kulcs |
| timeline | timeline | timeline | timeline | línea de tiempo | linha do tempo | Timeline | idővonal |
| graph editor | editor de grafice | éditeur de courbes | editor di curve | editor de curvas | editor de curvas | Kurveneditor | görbeszerkesztő |
| clip | clip | clip | clip | clip | clipe | Clip | klip |
| lip-sync | lip-sync | synchronisation labiale | lip-sync | sincronización labial | sincronização labial | Lippensynchronisation | ajaksinkron |
| state machine | mașină de stări | machine à états | macchina a stati | máquina de estados | máquina de estados | Zustandsautomat | állapotgép |
| runtime | runtime | runtime | runtime | runtime | runtime | Laufzeitumgebung | futtatókörnyezet |
| studio | studio | studio | studio | estudio | estúdio | Studio | stúdió |
| layer | strat | calque | livello | capa | camada | Ebene | réteg |
| frame | cadru | image | fotogramma | fotograma | quadro | Frame | képkocka |
| golden frame | cadru de referință | image de référence | fotogramma di riferimento | fotograma de referencia | quadro de referência | Referenzbild | referenciakocka |
| limitation | limitare | limitation | limitazione | limitación | limitação | Einschränkung | korlátozás |
| workaround | soluție ocolitoare | contournement | soluzione alternativa | alternativa | alternativa | Umgehung | kerülőmegoldás |
| roadmap | roadmap | feuille de route | roadmap | hoja de ruta | roadmap | Roadmap | ütemterv |
| pre-release | pre-lansare | préversion | pre-release | versión preliminar | pré-lançamento | Vorabversion | előzetes kiadás |
| command palette | paletă de comenzi | palette de commandes | palette dei comandi | paleta de comandos | paleta de comandos | Befehlspalette | parancspaletta |
| undo / redo | anulare / refacere | annuler / rétablir | annulla / ripeti | deshacer / rehacer | desfazer / refazer | Rückgängig / Wiederholen | visszavonás / újra |
| facade (API) | fațadă | façade | facciata | fachada | fachada | Fassade | homlokzat |
| shortcut | scurtătură | raccourci | scorciatoia | atajo | atalho | Tastenkürzel | gyorsbillentyű |
| collaboration | colaborare | collaboration | collaborazione | colaboración | colaboração | Zusammenarbeit | együttműködés |
| determinism | determinism | déterminisme | determinismo | determinismo | determinismo | Determinismus | determinizmus |

## Status

Run `python3 scripts/i18n.py status` for the table of translated documents.
