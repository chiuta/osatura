# Proveniență și clean-room — OSATURA (chiuta/osatura)

Document de completat cu fapte reale. Nu declara clean-room pentru ce nu s-a întâmplat așa.

## Ce a existat până acum (completează onest)
- Ținta/țintele: Moho (animație 2D / rigging)
- Cum s-a documentat comportamentul țintei (documentație publică, utilizare, formate observate, decompilare?): ______
- A fost folosit vreun cod, pseudocod, decompilare sau asset al țintei în sesiunile de dezvoltare? DA / NU / NU ȘTIU: ______
- Sesiunile de dezvoltare au avut vreodată acces la binarul sau codul țintei? ______

## Risc de licență
Software comercial proprietar. Verifică EULA pentru clauze de reverse engineering. Formatele de fișier proprii pot avea restricții; preferă formate deschise.

## Dependențe terțe
Rulează `cargo license` (sau `cargo deny check licenses`) și atașează rezultatul. Verifică compatibilitatea cu licența proiectului.

## Pentru lucrul de acum înainte
Folosește fluxul din acest kit (1-analiza → 2-filtru → 3-implementare). Orice funcție nouă inspirată de o țintă trece prin specificație și filtru.

## Recomandare
Pentru lansări comerciale sau donații semnificative, cere o opinie de la un avocat de proprietate intelectuală și fă căutare de mărci (EUIPO), cum ai făcut pentru OSATURA.

## Constatări din audit automat (2026-11-08, doar observații)
- Există deja `CLEANROOM.md`, care afirmă onest că proiectul NU este clean-room în sens strict, ci implementare independentă din specificații publice, cu risc rezidual documentat (inclusiv agent AI de cercetare și model AI la scriere). Acest kit completează documentul, nu îl înlocuiește.
- În cod, „Moho” apare doar în `osatura-core/src/importers.rs`, pentru formatul text simplu de liste de foneme (`MohoSwitch1`, `cadru FONEMĂ`), citit pentru interoperabilitate. Nu s-a găsit alt nume al țintei în sursă.
- Antetele GPL/AGPL găsite sunt cele ale propriilor crate-uri (AGPL pentru `osatura-cli`), conform `LICENSING.md`.
- Dependențe cu MPL-2.0 (cssparser, selectors, smartstring ș.a.) sunt listate în `docs/THIRD_PARTY.md`. `cargo metadata` nu a putut rula în audit (toolchain-ul fixat nu s-a putut descărca), deci nu s-a reverificat independent.

**Limite ale auditului:** căutarea după nume nu poate detecta cod copiat fără nume. Rămân de răspuns întrebările din secțiunea „Ce a existat până acum".
