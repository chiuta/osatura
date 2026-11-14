<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Adatvédelem

Ez a program nem továbbít semmilyen információt más hálózatba kötött rendszereknek, hacsak a felhasználó, illetve a telepítést vagy üzemeltetést végző személy kifejezetten nem kéri.

Részletesen, az `osatura`, az `osatura-studio`, a C könyvtár és a WebAssembly modul esetében:

* **Nincs telemetria, elemzés, összeomlásjelentés, hirdetés vagy automatikus frissítésellenőrzés.** A programok nem tartalmaznak olyan kódot, amely magától szervert keresne fel.
* **Egyetlen parancs keres fel szervert, és csak akkor, ha te kiadod.** Az `osatura update-check` egyetlen névtelen HTTPS-kérést küld a GitHubnak (`api.github.com`) a kiadások nyilvános listájáért, a rendszered `curl` programját használva; a GitHub látja az IP-címedet és ezt a kérést, a [GitHub adatvédelmi nyilatkozata](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) szerint. Semmi nem töltődik le és nem települ, és rólad vagy a projektjeidről semmi nem kerül elküldésre. Hogy a GitHub ne lássa a címedet, add hozzá a `--proxy-preset tor` (vagy `tor-browser`, `warp`) kapcsolót, illetve a `--proxy-url` kapcsolót a saját proxyddal; lásd [CLI_REFERENCE.md](CLI_REFERENCE.md).
* **A stúdió helyi szerver.** Az `osatura studio` és az `osatura-studio` alapértelmezésben a `127.0.0.1` címen figyel (csak erről a számítógépről érhető el). Más gépekről csak akkor érhetők el, ha te magad indítod őket a `--bind` kapcsolóval más címen ([SECURITY.md](SECURITY.md)). Az általuk kiszolgált oldal nem tölt be semmit az internetről (a Content-Security-Policy csak a helyi szervert engedi).
* **A linkek, amelyekre kattintasz.** A stúdió és a dokumentáció linkeket tartalmaz (például a GitHubra vagy egy adományozási oldalra). Ha követsz egyet, az az oldal nyílik meg a böngésződben, annak az oldalnak az adatvédelmi szabályzata szerint.
* **Külső programok.** Az `ffmpeg` csak videóhoz és egyes hangformátumokhoz indul el, csak akkor, ha ilyen exportot kérsz, és helyi fájlokon dolgozik.
* **A megosztott munkamenetek előzményt őriznek.** A `--collab-log` kapcsolóval a stúdió a munkamenet minden szerkesztését, a résztvevők nevével együtt, egy általad választott fájlba írja; résztvevőnkénti tokenekkel a nevek azok, amelyeket te hoztál létre. Kezeld ezt a fájlt a résztvevők személyes adataiként, és töröld a munkamenet végén ([SECURITY.md](SECURITY.md), [UNINSTALL.md](../../UNINSTALL.md)). Az arc-tájékozódási pontokat kezelő eszközök az eredményeiket csak az általad megnevezett fájlokba írják.
* **A fájljaid a tieid maradnak.** A projektek, renderek és beállítások csak oda íródnak, ahová te mondod a programnak, valamint a [UNINSTALL.md](../../UNINSTALL.md) fájlban felsorolt kis beállításfájlba.

A projekt weboldala (<https://osatura.chiuta.ro>) a GitHub Pages által üzemeltetett statikus oldal; rá a GitHub saját adatvédelmi nyilatkozata vonatkozik.

Kérdések: <alexio@trom.tf>.
