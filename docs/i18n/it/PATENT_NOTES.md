# Note sui brevetti

**Questa non è consulenza legale né un parere di libertà di operare (freedom-to-operate).** Registra ciò che NON è stato verificato, così che nessuno scambi l'assenza di una verifica per un'autorizzazione. Le licenze di OSATURA (MPL-2.0 per le librerie, AGPL-3.0-or-later per la CLI) contengono concessioni di brevetto dei contributori per il codice scritto dall'autore stesso, ma non dicono nulla sui brevetti di terzi. Non è stata effettuata alcuna ricerca brevettuale da parte di un professionista.

## Verifiche ancora da fare

| Ambito | Perché può essere rilevante | Stato |
|---|---|---|
| Puppeteering guidato dal volto (`faceparam`, `facedet`, `liveinput`: punti di riferimento o rapporti del volto che pilotano i parametri del rig) | L'animazione di personaggi 2D guidata dalla performance è un campo in cui si depositano attivamente domande commerciali. | Non verificato. Il codice usa misure di manuale pubblicate (eye aspect ratio, filtro One-Euro, HOG + regressione logistica); ciò non esclude rivendicazioni sulla loro combinazione. |
| Macchine a stati e data binding (`statemachine`, `runtime_machine`, `bind`) | I runtime di animazione interattiva con macchine a stati, blend tree, input e binding dei view-model sono un'area di funzionalità commerciali. | Non verificato. Progettazione propria del modello dati, non un'autorizzazione brevettuale. |
| FABRIK, mean-value coordinates, bounded biharmonic weights (`rig`, `cage`, `weights`) | Algoritmi pubblicati; alcuni autori e datori di lavoro hanno depositato brevetti su tecniche di deformazione e di skinning. | Non verificato. FABRIK (Aristidou e Lasenby, 2011) e MVC (Floater, 2003) sono usati a partire dagli articoli; il modulo dei pesi usa una semplice diffusione armonica, non il solver biarmonico limitato pubblicato. |
| Riempimento dei tracciati stencil-then-cover (`osatura-gpu`) | La tecnica ha origine nella ricerca sul rendering di tracciati su GPU e nelle estensioni dei produttori. | Non verificato. Qui è usata solo tramite la API standard wgpu con un semplice stencil buffer. |
| Codec video e audio tramite ffmpeg (`export` .mp4 / .webm, `soundtrack`) | H.264 e AAC sono coperti da pool di licenze (MPEG LA / Via LA e altri). VP9 e AV1 sono commercializzati come privi di royalty, ma per entrambi sono state annunciate rivendicazioni di patent pool e di singoli brevetti. | Non valutato. Non considerare VP9/AV1 come autorizzati. |

## ffmpeg
ffmpeg è un **sottoprocesso esterno** fornito dall'utente. OSATURA non lo collega, non lo distribuisce e non deve includerlo in nessuna release: includerlo renderebbe il distributore responsabile della licenza di ffmpeg stesso (LGPL/GPL a seconda della build) e delle licenze dei codec sopra citati. Ogni opzione di codec è una scelta dell'utente sulla propria installazione.

## Prima della release commerciale
Fai esaminare le aree sopra elencate da un consulente in materia di brevetti, nelle giurisdizioni in cui il software sarà distribuito, e registra il risultato qui e in CLEANROOM.md (regola 5).
