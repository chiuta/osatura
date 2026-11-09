# Funcție: generare_schemă_json_pentru_proiect

Sursă publică: *JSON Schema: A Media Type for Describing JSON Documents* (core, 2020-12) https://json-schema.org/draft/2020-12/json-schema-core.html și *JSON Schema Validation: A Vocabulary for Structural Validation of JSON* (2020-12) https://json-schema.org/draft/2020-12/json-schema-validation.html; pagina de specificații: https://json-schema.org/specification
Dimensiune estimată: S (mecanism) + M (acoperirea întregului format) · Status: candidat · Data: 2026-11-09 · Limitare acoperită: R-003 („published JSON Schema"; FEATURES: rând „JSON Schema export" marcat nepornit)
Citire sursă în sesiune: da pentru cuvintele-cheie $schema, $id, $ref, $defs, $comment, regula schemei booleene și cuvintele de validare enumerate mai jos (text core citit până la ~100 000 de caractere; secțiunea 11 „unevaluated" nu a fost citită). Documentul core se declară proiect IETF („Internet-Draft"), de tratat ca versiune de lucru.

## Scop
Publică o schemă JSON formală a fișierelor de proiect, ca editoarele și instrumentele externe să valideze proiectele înainte de încărcare și să ofere completare automată.

## Intrări / ieșiri
- Intrare: descrierea internă a formatului (tipuri, câmpuri, câmpuri obligatorii, valori implicite, enumerări, intervale, versiune de format) — în proiect există deja versiunea formatului și lista câmpurilor.
- Ieșire: un document JSON valid cu schemă 2020-12, determinist (aceeași intrare → aceiași octeți), cu chei sortate.
- Erori: tip de câmp necunoscut în descriere; referință circulară nerezolvabilă; interval minim > maxim.

## Comportament observabil
1. Schema este un obiect sau o valoare booleană [observat, 4.3]. Rădăcina este obiect.
2. Rădăcina conține „$schema" cu un URI absolut normalizat al dialectului: https://json-schema.org/draft/2020-12/schema [observat: URI-ul meta-schemei de dialect din pagina de specificații; cerința „URI cu schemă, normalizat" 8.1.1].
3. Rădăcina conține „$id": URI absolut fără fragment nevid [observat, 8.2.1]. Valoarea o alege proiectul (domeniu propriu); testele folosesc un URI de exemplu.
4. Tipurile reutilizabile (strat, os, cheie, deformator etc.) se pun sub „$defs" și se referă prin „$ref" cu URI-reference [observat, 8.2.3.1, 8.2.4]. „$defs" nu afectează validarea direct [observat].
5. Cuvinte de validare folosite [observat, validare]: „type" (null, boolean, object, array, number, string, integer; „integer" = număr cu parte fracționară zero), „enum", „const", „minimum", „maximum", „exclusiveMinimum", „exclusiveMaximum", „multipleOf" (> 0), „minLength", „maxLength", „pattern" (ECMA-262, neancorat), „minItems", „maxItems", „uniqueItems", „required" (listă de șiruri unice), „minProperties", „maxProperties", „dependentRequired".
6. Cuvinte de adnotare, fără efect asupra validității: „title", „description", „default", „deprecated", „examples" [observat]. „$comment" este pentru autori și nu trebuie afișat utilizatorilor finali [observat, 8.3].
7. Cuvintele care definesc forma obiectelor și a listelor („properties", „additionalProperties", „items", „prefixItems", „oneOf", „anyOf", „allOf") aparțin vocabularului de aplicare din documentul core, a cărui secțiune nu a fost citită în detaliu: [dedus] din cunoașterea generală a standardului; se confirmă la revizuire.
8. Maparea cerută din descrierea internă: câmp numeric întreg → „integer"; real → „number"; text → „string"; enumerare de nume acceptate → „enum" cu exact lista de nume acceptate de încărcător; câmp cu valoare implicită → nu e în „required" și are „default"; câmp fără valoare implicită → în „required"; câmpul de versiune → „const" sau „enum" pe versiunile acceptate; interval acceptat de încărcător → „minimum"/„maximum".
9. Câmpurile necunoscute: încărcătorul le raportează (nu le respinge) [din L-63 al proiectului]; schema nu interzice câmpuri suplimentare în modul implicit; opțional modul „strict" interzice câmpurile suplimentare [dedus, decizie].
10. Numerele cu valoare întreagă în exemple apar fără punct zecimal [dedus].

## Formate / protocoale
JSON conform standardului JSON Schema 2020-12 (RFC 8259 pentru JSON, [dedus]). Fișier de ieșire: text UTF-8.

## Cazuri limită și erori
- Enumerare goală → eroare (enum cere cel puțin un element recomandat [observat: „ideally at least one"]).
- „pattern" nu este ancorat implicit [observat]: modelele din descriere trebuie ancorate explicit dacă e nevoie.
- Valori implicite care nu respectă schema → eroare la generare [dedus: „default ideally valid"].
- Nume de câmp duplicate între tipuri → separate prin „$defs", fără conflict.
- Format fără câmpuri → obiect vid permis.

## Teste de acceptanță
Se folosește o descriere minimală de formă: tip „Punct" cu câmpurile x (real, obligatoriu), y (real, obligatoriu), culoare (text din {„rosu", „verde"}, implicit „rosu"); rădăcină cu câmpurile versiune (întreg, const 3) și puncte (listă de „Punct").
1. Schema generată declară „$schema" = URI-ul dialectului 2020-12 și un „$id" absolut fără fragment.
2. „Punct" apare o singură dată sub „$defs"; câmpul „puncte" îl referă cu „$ref"; „required" al lui Punct = [„x", „y"] (culoare lipsește din listă pentru că are implicit).
3. Câmpul culoare: „type" = „string", „enum" = [„rosu", „verde"], „default" = „rosu".
4. Un document {„versiune": 3, „puncte": [{„x": 1.5, „y": 2}]} este valid; {„versiune": 3, „puncte": [{„x": 1.5}]} este invalid (lipsește „y"); {„versiune": 4, ...} este invalid (const).
5. Un document cu „culoare": „albastru" este invalid (enum).
6. Determinism: două generări consecutive produc fișiere identice octet cu octet; ordinea cheilor este alfabetică.
7. Schema generată este ea însăși validă față de meta-schema dialectului 2020-12 (verificare cu un validator extern la integrare).
8. Eroare: intervalul minim 5 > maxim 3 în descriere → generarea eșuează, cu mesaj.

## Necunoscute
- [necunoscut] Vocabularul de aplicare („properties", „items" etc.) nu a fost citit; testele 2–5 depind de el.
- [necunoscut] Politica de versionare a URI-ului „$id" între versiuni de format.
- [necunoscut] Comportament la „unevaluatedProperties" pentru moduri „strict" cu compunere.
- Documentul core este marcat proiect IETF în sesiunea de citire; statutul oficial al versiunii se verifică.

## Checklist de filtru (verificare.md)
Spec: 08-schema-json-export-format-proiect   Data: 2026-11-09
- [x] Nicio linie de cod, pseudocod sau asamblare copiată
- [x] Niciun nume intern de funcție, variabilă sau clasă din țintă
- [x] Niciun text de UI, asset sau branding din țintă
- [x] Algoritmii sunt descriși prin intrare → ieșire, nu prin structura codului țintei
- [x] Afirmațiile sunt marcate [observat] / [dedus] / [necunoscut]
- [ ] Licența/EULA țintei verificată și notată în `1-analiza/jurnal.md` (rând adăugat; licența documentelor neverificată în sesiune)
- [ ] Specificația nu depinde de brevete sau secrete comerciale (nu s-a făcut verificare de brevete)
- [ ] Aprobată de mine înainte de predare
