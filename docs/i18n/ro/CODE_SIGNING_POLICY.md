<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Politica de semnare a codului

Semnare gratuită a codului oferită de [SignPath.io](https://about.signpath.io/), certificat de la [SignPath Foundation](https://signpath.org/).

*Stare (25 oct. 2026): proiectul a solicitat, sau este pe cale să solicite, un certificat gratuit de la SignPath Foundation. Până când o versiune spune altceva, fișierele pentru Windows **nu** sunt semnate Authenticode; fiecare fișier al unei versiuni poartă deja o semnătură Sigstore și o atestare de proveniență a build-ului GitHub (vezi [Verificare](#verificare)).*

## Ce se semnează

Se semnează doar fișierele construite din acest depozit ([github.com/chiuta/osatura](https://github.com/chiuta/osatura)) de fluxul său de lansare ([`.github/workflows/release.yml`](../../../.github/workflows/release.yml)) pe runner-e găzduite de GitHub. Nimic construit pe un calculator personal nu este semnat vreodată și niciun binar al unei terțe părți nu este semnat.

| Fișier (în `osatura-windows-x86_64.zip`) | Ce este | Licență |
|---|---|---|
| `osatura.exe` | unealta în linie de comandă și studioul din browser | AGPL-3.0-or-later |
| `osatura-studio.exe` | studioul într-o fereastră nativă (WebView2) | AGPL-3.0-or-later |
| `lib/osatura_capi.dll` | biblioteca C | MPL-2.0 |

Fiecare fișier semnat are numele de produs `OSATURA` și versiunea lansării în proprietățile fișierului (încorporate de `build.rs` în fiecare crate și verificate de fluxul de lucru înainte de orice semnare). Configurația de artefact SignPath se păstrează în [`.signpath/artifact-configuration.xml`](../../../.signpath/artifact-configuration.xml).

## Rolurile echipei

Proiectul are în prezent un singur mentenant, care deține toate cele trei roluri:

| Rol | Cine |
|---|---|
| Committeri și revizori | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |
| Aprobatori | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |

* Modificările venite de la oricine fără drept de scriere sosesc ca pull request-uri și sunt revizuite de un committer înainte de a fi integrate ([CONTRIBUTING.md](CONTRIBUTING.md)).
* O cerere de semnare este creată doar de fluxul de lansare pentru un tag de versiune (`vMAJOR.MINOR.PATCH[-pre]`) al cărui commit se află pe `main`, iar fiecare cerere este aprobată manual în SignPath de un aprobator.
* Toți cei cu unul dintre aceste roluri trebuie să folosească autentificare cu mai mulți factori pentru GitHub și SignPath (condiție a SignPath Foundation; mentenantul o confirmă pentru fiecare cont la acordarea certificatului).
* Orice modificare a acestor roluri este consemnată în acest fișier.

## Politica de confidențialitate

Acest program nu va transfera nicio informație către alte sisteme conectate în rețea decât dacă utilizatorul sau persoana care îl instalează ori îl operează o cere în mod explicit. Detalii: [PRIVACY.md](PRIVACY.md).

## Modificări ale sistemului tău și dezinstalare

OSATURA nu are instalator, nu scrie nimic în registry și nu instalează niciun serviciu, driver sau intrare de pornire. Puținele fișiere pe care le poate crea și modul de a elimina tot sunt enumerate în [UNINSTALL.md](../../UNINSTALL.md).

## Licențe

Tot codul este open source sub licențe aprobate de OSI: MPL-2.0 (bibliotecile) și AGPL-3.0-or-later (unealta în linie de comandă, studioul și aplicația cu fereastră); documentația este CC-BY-4.0 ([COPYRIGHT](../../../COPYRIGHT), [LICENSING.md](LICENSING.md)). Fișierele semnate nu conțin cod proprietar, iar OSATURA nu este oferit sub nicio licență comercială sau proprietară (fără licențiere duală).

## Verificare

* **Authenticode** (când va fi disponibil): clic dreapta pe un fișier în Windows Explorer > Properties > Digital Signatures; semnatarul este SignPath Foundation.
* **Originea fiecărui fișier de versiune**: `gh attestation verify <file> -R chiuta/osatura` sau `cosign verify-blob` cu pachetul `.sigstore.json` ([VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md)).

## Raportarea unei probleme

Dacă crezi că un fișier semnat pentru OSATURA este rău intenționat, nu a fost construit din acest depozit sau că această politică a fost încălcată, scrie la <alexio@trom.tf> sau deschide un issue; raportările de securitate urmează [SECURITY.md](SECURITY.md). Problemele cu certificatul în sine pot fi raportate și către [SignPath Foundation](https://signpath.org/).
