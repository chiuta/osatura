<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Privacy

Questo programma non trasferirà alcuna informazione ad altri sistemi in rete, a meno che non sia richiesto esplicitamente dall’utente o dalla persona che lo installa o lo gestisce.

Nel dettaglio, per `osatura`, `osatura-studio`, la libreria C e il modulo WebAssembly:

* **Nessuna telemetria, analisi d’uso, segnalazione di crash, pubblicità o controllo automatico degli aggiornamenti.** I programmi non contengono codice che contatti un server di propria iniziativa.
* **Un solo comando contatta un server, solo quando lo digiti.** `osatura update-check` invia una singola richiesta HTTPS anonima a GitHub (`api.github.com`) per l’elenco pubblico delle versioni, usando il programma `curl` del tuo sistema; GitHub vede il tuo indirizzo IP e quella richiesta, secondo l’[informativa sulla privacy di GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). Non viene scaricato né installato nulla e non viene inviato nulla su di te o sui tuoi progetti. Per non far vedere il tuo indirizzo a GitHub, aggiungi `--proxy-preset tor` (oppure `tor-browser`, `warp`) o `--proxy-url` con un tuo proxy; vedi [CLI_REFERENCE.md](CLI_REFERENCE.md).
* **Lo studio è un server locale.** `osatura studio` e `osatura-studio` restano in ascolto per impostazione predefinita su `127.0.0.1` (solo questo computer). Sono raggiungibili da altre macchine solo se li avvii tu stesso con `--bind` su un altro indirizzo ([SECURITY.md](SECURITY.md)). La pagina che servono non carica nulla da Internet (la sua Content-Security-Policy consente solo il server locale).
* **I link su cui fai clic.** Lo studio e la documentazione contengono link (per esempio a GitHub o a una pagina di donazioni). Seguirne uno apre quel sito nel tuo browser, sotto l’informativa sulla privacy di quel sito.
* **Programmi esterni.** `ffmpeg` viene avviato solo per il video e per alcuni formati audio, solo quando chiedi un’esportazione di quel tipo, e lavora su file locali.
* **Le sessioni condivise conservano uno storico.** Con `--collab-log` lo studio scrive ogni modifica della sessione, con i nomi dei partecipanti, in un file a tua scelta; con token per partecipante i nomi sono quelli che hai creato. Tratta quel file come dati personali dei partecipanti ed eliminalo a sessione conclusa ([SECURITY.md](SECURITY.md), [UNINSTALL.md](../../UNINSTALL.md)). Gli strumenti per i punti di riferimento del volto scrivono i risultati solo nei file che indichi.
* **I tuoi file restano tuoi.** Progetti, render e impostazioni vengono scritti solo dove dici al programma di scriverli, più il piccolo file di impostazioni elencato in [UNINSTALL.md](../../UNINSTALL.md).

Il sito del progetto (<https://osatura.chiuta.ro>) è una pagina statica ospitata da GitHub Pages; si applica l’informativa sulla privacy di GitHub.

Domande: <alexio@trom.tf>.
