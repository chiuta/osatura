<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Confidențialitate

Acest program nu va transfera nicio informație către alte sisteme conectate în rețea decât dacă utilizatorul sau persoana care îl instalează ori îl operează o cere în mod explicit.

În detaliu, pentru `osatura`, `osatura-studio`, biblioteca C și modulul WebAssembly:

* **Fără telemetrie, analiză, raportare de erori, publicitate sau verificări automate de actualizare.** Programele nu conțin cod care contactează de la sine vreun server.
* **O singură comandă contactează un server, doar când o scrii tu.** `osatura update-check` face o singură cerere HTTPS anonimă către GitHub (`api.github.com`) pentru lista publică de versiuni, folosind programul `curl` al sistemului tău; GitHub vede adresa ta IP și acea cerere, conform [declarației de confidențialitate a GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). Nu se descarcă și nu se instalează nimic și nu se trimite nimic despre tine sau despre proiectele tale. Ca să nu-ți afle GitHub adresa, adaugă `--proxy-preset tor` (sau `tor-browser`, `warp`) ori `--proxy-url` cu propriul proxy; vezi [CLI_REFERENCE.md](CLI_REFERENCE.md).
* **Studioul este un server local.** `osatura studio` și `osatura-studio` ascultă implicit pe `127.0.0.1` (doar acest calculator). Sunt accesibile de pe alte mașini doar dacă le pornești chiar tu cu `--bind` pe altă adresă ([SECURITY.md](SECURITY.md)). Pagina pe care o servesc nu încarcă nimic de pe internet (Content-Security-Policy-ul ei permite doar serverul local).
* **Linkurile pe care le accesezi.** Studioul și documentația conțin linkuri (de exemplu către GitHub sau o pagină de donații). Urmarea unuia deschide acel site în browser, sub politica de confidențialitate a site-ului respectiv.
* **Programe externe.** `ffmpeg` este pornit doar pentru video și pentru unele formate audio, doar când ceri un asemenea export, și lucrează pe fișiere locale.
* **Sesiunile partajate păstrează un istoric.** Cu `--collab-log`, studioul scrie fiecare modificare a sesiunii, cu numele participanților, într-un fișier la alegerea ta; cu token-uri pe participant, numele sunt cele pe care le-ai generat. Tratează acel fișier ca date personale ale participanților și șterge-l când sesiunea s-a terminat ([SECURITY.md](SECURITY.md), [UNINSTALL.md](../../UNINSTALL.md)). Uneltele pentru repere faciale scriu rezultatele doar în fișierele pe care le denumești.
* **Fișierele tale rămân ale tale.** Proiectele, randările și setările sunt scrise doar unde îi spui programului să le scrie, plus micul fișier de setări menționat în [UNINSTALL.md](../../UNINSTALL.md).

Site-ul proiectului (<https://osatura.chiuta.ro>) este o pagină statică găzduită de GitHub Pages; i se aplică declarația de confidențialitate proprie a GitHub.

Întrebări: <alexio@trom.tf>.
