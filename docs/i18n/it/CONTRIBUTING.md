# Contribuire ad OSATURA

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/README.md) · [Română](https://github.com/chiuta/osatura/blob/main/README.ro.md) · [Français](https://github.com/chiuta/osatura/blob/main/README.fr.md) · [Italiano](https://github.com/chiuta/osatura/blob/main/README.it.md) · [Español](https://github.com/chiuta/osatura/blob/main/README.es.md) · [Português](https://github.com/chiuta/osatura/blob/main/README.pt.md) · [Deutsch](https://github.com/chiuta/osatura/blob/main/README.de.md) · [Magyar](https://github.com/chiuta/osatura/blob/main/README.hu.md)

* **Regola dell'implementazione indipendente.** Contribuisci solo con codice, matematica e formati che puoi legalmente condividere. Non copiare, decompilare né fare reverse engineering di strumenti, file o interfacce proprietari, e non incollare codice da progetti la cui licenza non sia compatibile (vedi CLEANROOM.md). Non leggere né incollare codice sorgente di strumenti concorrenti mentre scrivi una funzionalità. Cita lo standard pubblico o il manuale a partire dal quale hai implementato.
* **Licenza dei contributi (inbound = outbound).** Un contributo è accettato con la licenza della parte che modifica: MPL-2.0 per `osatura-core`, `-capi`, `-wasm`, `-gpu` e `scripts/`; AGPL-3.0-or-later per `osatura-cli`; CC-BY-4.0 per la documentazione e gli asset sintetici. Ogni file sorgente porta una riga SPDX che deve essere uguale alla `license` del proprio crate (verificato da `osatura-cli/tests/spdx_audit.rs`).
* **Il sign-off non concede alcun diritto di cambiare licenza.** Il sign-off qui sotto certifica soltanto l'origine. **Una licenza commerciale o doppia (per esempio una licenza proprietaria per chi integra il motore) NON è attualmente possibile per il codice contribuito**: richiederebbe un Contributor Licence Agreement separato o una concessione outbound esplicita da parte di ciascun contributore, e nessuno dei due esiste. Finché non ne esiste uno, nessuno (compreso il maintainer) può affermare di poter cambiare la licenza dei contributi altrui.
* **Sign-off (DCO).** Ogni commit deve riportare `Signed-off-by: Your Name <email>` (`git commit -s`), che certifica il Developer Certificate of Origin 1.1 riprodotto qui sotto.
* Aggiungi test; esegui `cargo test --workspace` e `scripts/licence-audit.sh` prima di aprire una pull request.

## Developer Certificate of Origin 1.1 (verbatim)

*Riprodotto senza modifiche da https://developercertificate.org/ (Copyright (C) 2004, 2006 The Linux Foundation and its contributors); i suoi stessi termini consentono copie letterali. Questo testo non è coperto dalla CC-BY-4.0 del resto di questo file.*

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```
