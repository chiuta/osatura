# Întrebări pentru titular / etapa de specificare — lot 1

Format: spec · punct · ce lipsește · ce s-a implementat între timp. Completat de sesiunea de implementare (2026-11-09).

## 08 — schema JSON
- Testul de acceptanță 7 (schema generată este validă față de meta-schema dialectului 2020-12) cere un validator extern; nu există offline. S-a verificat doar că schema folosește exclusiv cuvinte-cheie ale dialectului (test automat) și că documentele reale ale proiectului trec prin validatorul-subset propriu. Rămâne de rulat un validator extern la integrare.
- Acoperirea „întregului format" (partea M): sunt descrise câmp cu câmp doar rădăcina, `camera`, `bones`, `ik`; restul listelor (layers, channels, constraints etc.) sunt „obiect liber". Cere specificații separate de câmpuri sau o decizie de a genera descrierea din tipurile serde.
- [necunoscut din spec] politica de versionare a `$id` între versiuni de format; `$id` actual este un URI de exemplu (`https://osatura.example/schema/project-v1.json`), de înlocuit cu domeniul real.
- Nu s-au folosit `unevaluatedProperties` (secțiune necitită); modul strict folosește `additionalProperties: false`.

## 05 — curbe de ritm pe segmente și trepte
- Integrarea în tipul `Ease` (care este `Copy`): curba pe segmente are o listă de puncte, deci nu poate fi variantă `Copy` simplă. Opțiuni: (a) scoate `Copy` din `Ease` (373 de utilizări în workspace), (b) bibliotecă de curbe numite la nivel de proiect și variantă `Ease::Named(index)`, (c) număr fix de puncte. Neimplementat: modulul `segease` oferă curbele și validarea, fără legătură cu `Track`/format.
- Intrarea implicită a ultimului punct când el are intrare explicită < 100 %: spec zice „cel puțin 1" (necunoscut). S-a ales: maximul dintre 1 și cea mai mare intrare anterioară; nu e acoperit de teste de acceptanță.
- Plafon ales de implementare: 4096 de puncte/trepte.

## 03 — panoramare stereo
- Conectarea la `sound mix` și câmpul `pan` pe `SoundTrack`: mixarea curentă este un graf de filtre ffmpeg (`soundtrack::track_chain`); modulul `stereopan` este o funcție pură pe eșantioane. Nu s-a ghicit nicio echivalență cu filtrele ffmpeg. Decizie necesară: mixer intern pentru WAV, sau pan-ul se aplică în afara ffmpeg. Neimplementat: câmpul `pan` și apelul din mix.
- Rampa de netezire la schimbarea lui p și mixarea canalelor > 2: [necunoscut] în spec, neimplementate. Panoramarea cu chei (interpolare liniară în timp, p. 7 din spec) este implementată ca `pan_mono_keyed`, fără test de acceptanță al specificației.

## 04 — filtru biquad
- Același punct ca la 03: nu există încă un câmp de filtre pe `SoundTrack` și nici un mixer intern; `biquad` este funcție pură pe eșantioane (mono, sau intercalat pe canale). Decizie necesară înainte de legare.
- [necunoscut din spec] rampa la schimbarea parametrilor în timp și formele cu BW/S: neimplementate. Frecvența peste 0,45·Fs dă avertisment (`warnings`), nu eroare; `Q` > 1000 este eroare (limită a proiectului, cf. spec).
