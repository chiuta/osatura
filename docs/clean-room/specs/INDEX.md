Aprobare filtru: prin instrucțiunea titularului din conversație (2026-11-09), nu bifată individual

# Specificații funcționale candidate — lot 1

Data: 2026-11-09 · Etapa 1 din 2 (analiză/specificare). Nu conține cod de produs. Toate specificațiile au status **candidat**: nu au trecut filtrul (căsuțele „Licență", „Brevete" și „Aprobată de mine" din fiecare fișier sunt nebifate) și nu se predau implementării înainte de aprobarea omului.

Surse: numai standarde și documentație publică. Fiecare fișier indică, la începutul lui, ce parte a sursei a fost efectiv citită în sesiune și ce nu (specificațiile 07, 09, 10, 11 se sprijină în mare parte pe cunoștințe de manual marcate [dedus]; 10 și 11 nu au nicio sursă citită).

## Ordinea recomandată de implementare

| Ord. | Fișier | Funcție | Status | Sursă publică | Mărime | Depinde de |
|---|---|---|---|---|---|---|
| 1 | [08-schema-json-export-format-proiect.md](08-schema-json-export-format-proiect.md) | generare_schemă_json_pentru_proiect | parțial: schema acoperă câmp cu câmp rădăcina, camera, oasele și IK; celelalte liste sunt obiecte libere; testul 7 (meta-schema) neverificat, fără validator extern offline | JSON Schema 2020-12 (core + validation), json-schema.org | S (+M acoperire) | — ; recomandat primul: câmpurile noi din 03, 04, 05, 06, 09, 11 se adaugă apoi în descrierea formatului |
| 2 | [05-easing-liniar-pe-segmente-si-trepte.md](05-easing-liniar-pe-segmente-si-trepte.md) | curbă_de_ritm_pe_segmente_și_trepte | parțial: curbele, validarea și presetările sunt implementate cu teste; neconectate la `Ease`/`Track`/format (Ease este Copy; vezi INTREBARI.md) | W3C CSS Easing Functions Level 2 (FPWD, 2024) | S | — |
| 3 | [03-audio-panoramare-stereo.md](03-audio-panoramare-stereo.md) | panoramare_stereo_putere_egală | parțial: funcția și testele de acceptanță 1-8 sunt implementate; neconectată la `sound mix` (mixarea trece prin ffmpeg) și fără câmpul `pan` pe pistă (vezi INTREBARI.md) | W3C Web Audio API (StereoPanner) | S | — |
| 4 | [04-audio-filtru-biquad.md](04-audio-filtru-biquad.md) | filtru_biquad_pe_pistă | parțial: filtrele și testele de acceptanță 1-8 sunt implementate; neconectate la `sound mix`/format de pistă (vezi INTREBARI.md) | W3C Note „Audio EQ Cookbook" (2021) | S | 03 (recomandat: aceeași cale de mixare, nu obligatoriu) |
| 5 | [06-layout-aliniere-pe-linia-de-baza.md](06-layout-aliniere-pe-linia-de-baza.md) | aliniere_pe_linia_de_bază | parțial: funcția și testele de acceptanță 1-8 sunt implementate; neconectată la `containers`/format (câmpuri de strat nespecificate în cod; vezi INTREBARI.md) | W3C CSS Flexible Box Layout Level 1, 8.3 și 9.4 | S | — |
| 6 | [01-gif-decodare-secventa.md](01-gif-decodare-secventa.md) | decodare_secvență_gif | implementat (teste trec); nelegat de `import-anim` (care are deja decodor din crate-ul gif; vezi INTREBARI.md) | GIF89a (CompuServe 1990, copie W3C) | M | — |
| 7 | [02-apng-decodare-secventa.md](02-apng-decodare-secventa.md) | decodare_secvență_apng | implementat (teste trec); nelegat de `import-anim` (vezi INTREBARI.md) | APNG Specification (wiki Mozilla); PNG (W3C) | M | 01 (recomandat: aceeași ieșire „listă de cadre + durate" și același plafon de decompresie); necesită decodorul PNG existent |
| 8 | [10-coliziune-capsula-capsula.md](10-coliziune-capsula-capsula.md) | coliziune_capsulă_capsulă_2d | implementat (teste trec: 1-11 din spec, verificate aritmetic); neconectat la `physics` (vezi INTREBARI.md) | Ericson 2005 §5.1.9; Eberly (note Geometric Tools) — necitite | M | — |
| 9 | [09-iluminare-spot-si-speculara.md](09-iluminare-spot-si-speculara.md) | iluminare_spot_și_speculară | candidat | Manual OpenGL glLight (citit); Blinn 1977; Phong 1975 (necitite) | M | — |
| 10 | [11-estompare-de-miscare-prin-acumulare.md](11-estompare-de-miscare-prin-acumulare.md) | estompare_mișcare_prin_eșantionare_temporală | candidat | Potmesil–Chakravarty 1983; Korein–Badler 1983; Cook et al. 1984 — necitite | M | — (se bazează pe randarea existentă; costă N randări/cadru) |
| 11 | [07-gltf-animatie-transformari-strat.md](07-gltf-animatie-transformari-strat.md) | export_animație_noduri_gltf | candidat | Khronos glTF 2.0 (specificația necitită; citit ghidul Khronos „Animations") | M | 05 (recomandat: maparea tipurilor de interpolare ale proiectului la STEP/LINEAR/CUBICSPLINE) |

Note de ordine: 08 întâi pentru că fiecare spec care adaugă câmpuri (03, 04, 05, 06, 09, 11) trebuie reflectat apoi în schemă; specificațiile S înaintea celor M; 07 la urmă pentru că depinde cel mai mult de o sursă încă necitită.

## Dependențe pe scurt
- 08 ← (03, 04, 05, 06, 09, 11): sincronizare de câmpuri, nu dependență tehnică.
- 04 → 03 (cale de mixare comună, recomandat).
- 02 → 01 (recomandat), 02 → decodor PNG existent.
- 07 → 05 (recomandat).
- 10, 09, 11, 01, 06: independente.

## Ce trebuie făcut înainte de predarea către etapa 2
1. Omul bifează licența documentelor-sursă, brevetele (la nevoie consultare PI) și „Aprobată de mine" în fiecare fișier.
2. Pentru 07, 09, 10, 11: recitirea surselor indicate și actualizarea marcajelor [dedus] → [observat] acolo unde sursa confirmă.
3. Copierea specificațiilor aprobate în `docs/clean-room/3-implementare/spec/`.
