# OSATURA — licenze (DECISO: Variante A, applicata il 22 ott 2026; vedi LICENSE e LICENSES/)

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/README.md) · [Română](https://github.com/chiuta/osatura/blob/main/README.ro.md) · [Français](https://github.com/chiuta/osatura/blob/main/README.fr.md) · [Italiano](https://github.com/chiuta/osatura/blob/main/README.it.md) · [Español](https://github.com/chiuta/osatura/blob/main/README.es.md) · [Português](https://github.com/chiuta/osatura/blob/main/README.pt.md) · [Deutsch](https://github.com/chiuta/osatura/blob/main/README.de.md) · [Magyar](https://github.com/chiuta/osatura/blob/main/README.hu.md)

**Stato:** la Variante A qui sotto è stata scelta e applicata: MPL-2.0 per le librerie (core, capi, wasm, gpu), AGPL-3.0-or-later per `osatura-cli` (CLI + studio) e `osatura-gui` (lo studio in una finestra nativa), CC-BY-4.0 per la documentazione e per gli asset e i modelli sintetici, politica sul marchio in TRADEMARK.md, DCO in CONTRIBUTING.md, intestazioni SPDX in tutti i file .rs/.js/.mjs/.sh e REUSE.toml per il resto, offerta del sorgente ai sensi della sezione 13 della AGPL (`/source`, `--source-url`), campi `license` di Cargo impostati. Azioni rimanenti a carico del titolare: revisione legale, ricerca sul marchio (EUIPO/OSIM), verifica dei brevetti (docs/PATENT_NOTES.md), un CLA se si vuole la doppia licenza, inclusione degli avvisi di terze parti nelle release binarie.


*Non è consulenza legale. La licenza è DECISA (Variante A, vedi LICENSE); le sezioni seguenti sono conservate come motivazione e come elenco delle alternative considerate. Fai confermare il tutto da un avvocato (idealmente uno che conosca il diritto d'autore rumeno/UE) prima della prima release pubblica. Contributi: finché non esiste un CLA, non c'è alcun diritto di rilicenziare o di concedere in doppia licenza i contributi di terzi (CONTRIBUTING.md).*

## 1. Cosa deve fare la licenza per te
Il tuo reddito sono le **donazioni (Buy Me a Coffee)**, non i canoni di licenza. Quindi non è la licenza a creare il denaro: lo fanno **benevolenza, visibilità e fiducia**. La licenza ha tre compiti:
1. **Massimizzare gli utenti** (più utenti → più sostenitori): deve essere libera da usare, anche nel lavoro commerciale.
2. **Impedire che un'azienda prenda il codice, lo chiuda e lo venda** (o lo offra come servizio a pagamento) senza restituire nulla.
3. **Mantenere il tuo diritto di vendere eccezioni** (licenza commerciale, supporto) e di difendere il nome.

Deve inoltre conciliarsi con le tue convinzioni **trade-free / TROM**: il software non deve essere reso limitato, finanziato dalla pubblicità o legato alla raccolta di dati. (Le licenze che vietano l'uso commerciale, ad es. CC-NC o Polyform-NC, *non* sono open source e non sono la stessa cosa di "trade-free"; verifica la formulazione di TROM prima di adottarne una.)

## 2. Risultati che vincolano la scelta
- **Le dipendenze non sono un ostacolo.** Audit di tutti i crate di terze parti (363 nella rigenerazione del 24 ott 2026 con `scripts/licence-audit.sh`, contati su tutte le piattaforme insieme dopo l'aggiunta dell'app a finestra; erano 160 il 22 ott; il numero cambia, vedi docs/THIRD_PARTY.md): tutti sono MIT, Apache-2.0, BSD, Zlib, ISC, Unlicense, CC0, Unicode o doppie licenze di queste, tranne sei crate MPL-2.0 con copyleft debole a livello di file: smartstring (MPL-2.0+, raggiunto solo tramite la feature `scripting`, che è ATTIVA per impostazione predefinita in `osatura-core` e `osatura-cli`) e `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors` e `option-ext` (raggiunti solo tramite l'app a finestra `osatura-gui`); una build delle librerie con `--no-default-features` non ha alcun crate di terze parti MPL. Possono essere combinati con AGPL, MPL o una licenza proprietaria; il crate MPL richiede la conservazione del sorgente e dell'avviso.
- **Titolarità del copyright.** Gran parte del codice è stata scritta con Claude. Il diritto d'autore in genere si riferisce al contributo creativo *umano* (le linee guida dello US Copyright Office; anche il diritto UE/rumeno richiede la creazione intellettuale propria di un autore umano). Conseguenze pratiche: (a) puoi e devi concedere il progetto in licenza come suo autore/compilatore umano, ma (b) non promettere l'esclusività di ogni riga, (c) conserva nel repository le tue note di progetto, le tue scelte e le tue modifiche come prova di paternità, (d) un DCO certifica solo l'origine; per rilicenziare i contributi in seguito (ad es. doppia licenza) servono un CLA o una concessione esplicita in uscita, che NON sono in vigore.
- **Nessuna licenza può imporre donazioni.** Non vincolare mai il codice a una donazione; un fork rimuoverebbe semplicemente il vincolo. Vendi *comodità ed extra*, non il permesso.

## 3. Pacchetto consigliato — "Libreria open core, app protetta" (tutte le varianti qui sotto sono pronte da applicare)

### ★ Variante A (consigliata): libreria MPL-2.0 + app AGPL-3.0 + marchio
| Parte | Licenza | Perché |
|---|---|---|
| `osatura-core` (motore, evaluator, formati, exporter) ed eventuali futuri runtime/WASM/C-ABI | **MPL-2.0** | Copyleft a livello di file: chiunque può incorporarlo in giochi/app (chiusi o aperti), ma le modifiche ai *nostri file* devono essere condivise. Include una esplicita **concessione di brevetto** e la cessazione in caso di causa per brevetto. Massimizza l'adozione, che è ciò di cui hanno bisogno le donazioni. |
| `osatura-cli`, `osatura-gui`, studio (browser/finestra) | **AGPL-3.0-or-later** | Impedisce i fork chiusi e i cloni "OSATURA come servizio web a pagamento"; i *progetti e i render* degli utenti restano interamente loro. |
| Nome "OSATURA", logo | **Politica sul marchio** (file separato) | I fork devono cambiare nome; protegge i sostenitori dalle imitazioni. Verifica la disponibilità (EUIPO/OSIM/WIPO) prima del lancio. |
| Rig di esempio, grafica, audio che crei | **CC BY 4.0** (o CC BY-SA) | Liberi da riutilizzare con attribuzione; la grafica di terzi mantiene la propria licenza. |
| Documentazione | **CC BY 4.0** | |
<!-- REUSE-IgnoreStart -->
Cargo: `license = "MPL-2.0"` (core), `license = "AGPL-3.0-or-later"` (cli). SPDX header per file: `// SPDX-License-Identifier: MPL-2.0`.
<!-- REUSE-IgnoreEnd -->
**Doppia licenza:** possibile solo per il codice scritto interamente dal titolare, oppure dopo un CLA/concessione in uscita da parte di ogni contributore. NON possibile oggi per il codice contribuito (CONTRIBUTING.md).

### Variante B: AGPL-3.0-or-later ovunque (+ licenza commerciale su richiesta)
La posizione più forte contro l'appropriazione e la più vicina all'etica trade-free. Costo: aziende e motori di gioco eviteranno di incorporare il runtime, quindi meno utenti e meno sostenitori indiretti. Scegli questa se la protezione conta più della diffusione.

### Variante C: Apache-2.0 ovunque (non scelta; il workspace applica la Variante A: librerie MPL-2.0 e `license = "AGPL-3.0-or-later"` per `osatura-cli`)
Massima diffusione e nessun attrito; concessione di brevetto inclusa. Costo: chiunque — anche una grande azienda — può chiuderlo e venderlo senza alcun obbligo. Le donazioni dipendono allora solo dalla benevolenza. Scegli solo se vuoi essere un servizio di pubblica utilità.

### Variante D: Source-available (BUSL-1.1 o Polyform Shield/NC con data di apertura differita)
Impedisce la concorrenza commerciale per N anni. Costo: **non è open source**; è in conflitto con la narrazione clean-room/formati aperti, con i contributi della comunità e con i valori trade-free; molti utenti e distribuzioni la rifiuteranno. **Sconsigliata.**

| | Diffusione | Anti-appropriazione | Concessione di brevetto | Reddito da doppia licenza | Compatibile con trade-free |
|---|---|---|---|---|---|
| **A core MPL + app AGPL** | alta | buona | sì | sì | sì |
| B AGPL ovunque | media | la più forte | sì (v3) | sì | sì |
| C Apache ovunque | la più alta | nessuna | sì | no | in parte |
| D source-available | bassa | forte | n/d | sì | no |

## 4. Come guadagnare davvero con le donazioni (tutto compatibile con la Variante A)
1. **Pagina Buy Me a Coffee** (`buymeacoffee.com/echo.of.the.strings`) collegata dal README, dal testo di `--version` della CLI, dal footer dello studio e dalla riga di crediti del player HTML (un credito *rimovibile* — mai obbligatorio).
2. **Abbonamenti, non paywall:** livelli mensili per le build in anteprima, voto sulla prossima voce della roadmap, nome in `SUPPORTERS.md`, chat privata. Tutti ricevono lo stesso codice; i membri ricevono *tempo e voce in capitolo*.
3. **Extra a pagamento che non sono il codice:** binari/installer firmati pronti all'uso, pacchetti di asset (rig, sfondi, librerie di cicli di camminata) con una licenza che ne consente l'uso in film/giochi ma non la rivendita del pacchetto stesso, corsi video, workshop.
4. **Commissioni e supporto:** servizi di rigging, exporter su misura, consulenza di integrazione; la doppia licenza commerciale per chi incorpora il codice.
5. **Livelli di sponsorizzazione per le organizzazioni** (logo nel README) e candidature a fondi per l'open source e la creatività (NLnet, programmi tipo Sovereign Tech, fondi per la cultura): pagano proprio per questo tipo di infrastruttura aperta.
6. **Pagina di trasparenza:** pubblica le entrate e ciò che hanno finanziato; fa aumentare in modo misurabile il sostegno ricorrente.
7. Tieni visibili sia **Patreon** (`patreon.com/c/alexio_tf`) sia BMC, ma dai a ciascuno uno scopo distinto (BMC per un ringraziamento una tantum; Patreon per il sostegno ricorrente) così i sostenitori non si confondono.

## 5. Checklist per applicare la Variante A (≈ 1 ora di lavoro una volta scelta)
- [x] `LICENSES/MPL-2.0.txt`, `LICENSES/AGPL-3.0-or-later.txt`, `LICENSES/CC-BY-4.0.txt`, `LICENSES/LicenseRef-third-party.txt` (struttura REUSE) più il riepilogo `LICENSE`; `REUSE.toml` copre i file non sorgente.
- [x] `license =` impostato in tutti e sei i file `Cargo.toml`; intestazioni SPDX in tutti i file .rs/.js/.mjs/.sh, verificate da `osatura-cli/tests/spdx_audit.rs`.
- [x] `docs/THIRD_PARTY.md` e `docs/THIRD_PARTY_NOTICES.md`, entrambi rigenerati da `scripts/licence-audit.sh`.
- [x] `CONTRIBUTING.md` con il testo del DCO 1.1. Un CLA serve ancora prima di qualsiasi doppia licenza.
- [x] `TRADEMARK.md` (consentito: "compatibile con OSATURA", ridistribuzione non modificata; non consentito: build modificate con lo stesso nome).
- [x] `.github/FUNDING.yml` (`buy_me_a_coffee: echo.of.the.strings`) così la piattaforma mostra un pulsante Sponsor.
- [~] Ricerca di marchio: il primo nome ARMATURA collideva con un marchio nelle classi 9/42 (EUIPO, 24 ott 2026), quindi il progetto è stato rinominato OSATURA; la ricerca EUIPO per "osatura" non ha trovato conflitti (un solo risultato non pertinente scaduto, classe 10). I nomi di pacchetto `osatura` su crates.io, npm e PyPI sono liberi (verificato il 24 ott 2026), ma esiste un marchio vicino, OSATRA (Open-Xchange AG, registrazione USA 7459236, classi 9/38/42). Le ricerche OSIM / TMview / WIPO / dominio e il parere di un avvocato sono ancora aperti, vedi [docs/LIMITATIONS.md](LIMITATIONS.md) L-78.
