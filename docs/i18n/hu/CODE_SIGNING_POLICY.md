<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Kódaláírási szabályzat

Ingyenes kódaláírást a [SignPath.io](https://about.signpath.io/) biztosít, a tanúsítványt a [SignPath Foundation](https://signpath.org/) adja.

*Állapot (2026. okt. 25.): a projekt kérelmezett, vagy hamarosan kérelmez egy ingyenes SignPath Foundation tanúsítványt. Amíg egy kiadás mást nem mond, a Windows-fájlok **nincsenek** Authenticode-dal aláírva; minden kiadási fájl már hordoz Sigstore-aláírást és GitHub build-eredet-igazolást (lásd [Ellenőrzés](#ellenőrzés)).*

## Mit írunk alá

Csak azokat a fájlokat írjuk alá, amelyeket ebből a tárolóból ([github.com/chiuta/osatura](https://github.com/chiuta/osatura)) a kiadási munkafolyamata ([`.github/workflows/release.yml`](../../../.github/workflows/release.yml)) épített GitHub által üzemeltetett futtatókon. Személyes számítógépen épített fájlt soha nem írunk alá, és harmadik féltől származó binárist sem.

| Fájl (az `osatura-windows-x86_64.zip` csomagban) | Mi ez | Licenc |
|---|---|---|
| `osatura.exe` | parancssori eszköz és böngészős stúdió | AGPL-3.0-or-later |
| `osatura-studio.exe` | a stúdió natív ablakban (WebView2) | AGPL-3.0-or-later |
| `lib/osatura_capi.dll` | C könyvtár | MPL-2.0 |

Minden aláírt fájl tulajdonságaiban szerepel az `OSATURA` terméknév és a kiadás verziója (a `build.rs` ágyazza be minden crate-ben, és a munkafolyamat ellenőrzi, mielőtt bármit aláírna). A SignPath artefaktumkonfigurációja a [`.signpath/artifact-configuration.xml`](../../../.signpath/artifact-configuration.xml) fájlban található.

## Csapatszerepek

A projektnek jelenleg egyetlen karbantartója van, aki mindhárom szerepet betölti:

| Szerep | Ki |
|---|---|
| Committerek és átnézők | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |
| Jóváhagyók | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |

* Az írási jogosultság nélküliek módosításai pull requestként érkeznek, és egy committer átnézi őket az egyesítés előtt ([CONTRIBUTING.md](CONTRIBUTING.md)).
* Aláírási kérelmet csak a kiadási munkafolyamat hoz létre olyan verziócímkéhez (`vMAJOR.MINOR.PATCH[-pre]`), amelynek commitja a `main` ágon van, és minden kérelmet egy jóváhagyó kézzel hagy jóvá a SignPathban.
* Mindenkinek, akinek ezen szerepek valamelyike van, többfaktoros hitelesítést kell használnia a GitHubon és a SignPathon (a SignPath Foundation feltétele; a karbantartó a tanúsítvány megadásakor minden fiókra megerősíti).
* E szerepek bármely változását ebben a fájlban rögzítjük.

## Adatvédelmi szabályzat

Ez a program nem továbbít semmilyen információt más hálózatba kötött rendszereknek, hacsak a felhasználó, illetve a telepítést vagy üzemeltetést végző személy kifejezetten nem kéri. Részletek: [PRIVACY.md](PRIVACY.md).

## A rendszeredet érintő változtatások és az eltávolítás

Az OSATURÁnak nincs telepítője, nem ír a rendszerleíró adatbázisba, és nem telepít szolgáltatást, illesztőprogramot vagy indítási bejegyzést. A létrehozható kevés fájlt és a mindent eltávolító lépéseket az [UNINSTALL.md](../../UNINSTALL.md) sorolja fel.

## Licencek

Minden kód nyílt forráskódú, OSI által elfogadott licencek alatt: MPL-2.0 (könyvtárak) és AGPL-3.0-or-later (parancssori eszköz, stúdió és ablakos alkalmazás); a dokumentáció CC-BY-4.0 ([COPYRIGHT](../../../COPYRIGHT), [LICENSING.md](LICENSING.md)). Az aláírt fájlok nem tartalmaznak szabadalmaztatott vagy zárt kódot, és az OSATURA nem kapható semmilyen kereskedelmi vagy zárt licenc alatt (nincs kettős licencelés).

## Ellenőrzés

* **Authenticode** (ha elérhetővé válik): jobb kattintás egy fájlon a Windows Intézőben > Tulajdonságok > Digitális aláírások; az aláíró a SignPath Foundation.
* **Minden kiadási fájl eredete**: `gh attestation verify <file> -R chiuta/osatura`, vagy `cosign verify-blob` a `.sigstore.json` csomaggal ([VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md)).

## Probléma jelentése

Ha úgy gondolod, hogy egy az OSATURÁhoz aláírt fájl rosszindulatú, nem ebből a tárolóból épült, vagy hogy ezt a szabályzatot megszegték, írj az <alexio@trom.tf> címre vagy nyiss egy issue-t; a biztonsági bejelentésekre a [SECURITY.md](SECURITY.md) vonatkozik. A tanúsítvány maga körüli problémákat a [SignPath Foundationnek](https://signpath.org/) is jelenteni lehet.
