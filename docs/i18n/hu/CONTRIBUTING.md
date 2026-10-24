# Hozzájárulás az OSATURA-hoz

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/CONTRIBUTING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/CONTRIBUTING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/CONTRIBUTING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/CONTRIBUTING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/CONTRIBUTING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/CONTRIBUTING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/CONTRIBUTING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/CONTRIBUTING.md)

* **A független megvalósítás szabálya.** Csak olyan kóddal, matematikával és formátummal járulj hozzá, amelyet jogszerűen megoszthatsz. Ne másolj zárt forrású eszközökből, fájlokból vagy felületekből, ne fordítsd vissza és ne fejtsd vissza őket, és ne illessz be kódot olyan projektekből, amelyek licence nem összeegyeztethető (lásd CLEANROOM.md). Funkció írása közben ne olvasd és ne illeszd be versenytárs eszközök forráskódját. Hivatkozz arra a nyilvános szabványra vagy tankönyvre, amelyből megvalósítottad.
* **A hozzájárulások licence (bejövő = kimenő).** A hozzájárulást annak a résznek a licence alatt fogadjuk el, amelyet módosít: MPL-2.0 az `osatura-core`, `-capi`, `-wasm`, `-gpu` és a `scripts/` esetén; AGPL-3.0-or-later az `osatura-cli` esetén; CC-BY-4.0 a dokumentációra és a szintetikus assetekre. Minden forrásfájl tartalmaz egy SPDX sort, amelynek meg kell egyeznie a crate `license` mezőjével (ezt az `osatura-cli/tests/spdx_audit.rs` ellenőrzi).
* **A sign-off nem ad átlicencelési jogot.** Az alábbi sign-off kizárólag a származást igazolja. **Kereskedelmi vagy kettős licenc (például zárt licenc beágyazóknak) a hozzájárult kódra jelenleg NEM lehetséges**: ehhez külön Contributor Licence Agreement vagy az egyes közreműködők kifejezett kimenő engedélye kellene, és egyik sincs érvényben. Amíg nincs ilyen, senki (a karbantartót is beleértve) nem állíthatja, hogy mások hozzájárulásait átlicencelheti.
* **Sign-off (DCO).** Minden commitnak tartalmaznia kell egy `Signed-off-by: Your Name <email>` sort (`git commit -s`), amely az alább közölt Developer Certificate of Origin 1.1-et tanúsítja.
* Adj hozzá teszteket; a pull request megnyitása előtt futtasd a `cargo test --workspace` és a `scripts/licence-audit.sh` parancsot.

## Developer Certificate of Origin 1.1 (szó szerint)

*Változtatás nélkül átvéve a https://developercertificate.org/ oldalról (Copyright (C) 2004, 2006 The Linux Foundation and its contributors); saját feltételei megengedik a szó szerinti másolatokat. Erre a szövegre nem vonatkozik e fájl többi részének CC-BY-4.0 licence.*

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
