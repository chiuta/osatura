# Szabadalmi megjegyzések

**Ez nem jogi tanács, és nem is szabad felhasználhatósági (freedom-to-operate) vélemény.** Azt rögzíti, hogy mit NEM ellenőriztünk, hogy senki ne tévessze össze az átvilágítás hiányát a jogtisztasággal. Az OSATURA licencei (MPL-2.0 a könyvtárakra, AGPL-3.0-or-later a CLI-re) a szerző saját kódjára tartalmaznak közreműködői szabadalmi felhatalmazást, de harmadik felek szabadalmairól nem mondanak semmit. Szakember által végzett szabadalmi keresés nem történt.

## Még elvégzetlen átvilágítások

| Terület | Miért lehet fontos | Állapot |
|---|---|---|
| Arcvezérelt bábozás (`faceparam`, `facedet`, `liveinput`: arc-landmarkok vagy arányok vezérlik a rig paramétereit) | A 2D karakterek teljesítményvezérelt animációja a kereskedelmi bejelentések aktív területe. | Nem átvilágított. A kód publikált, tankönyvi mérőszámokat használ (szem-képarány, One-Euro szűrő, HOG + logisztikus regresszió); ez nem zárja ki a kombinációra vonatkozó igényeket. |
| Állapotgépek és adatkötés (`statemachine`, `runtime_machine`, `bind`) | Az állapotgépekkel, blend tree-kkel, bemenetekkel és view-model kötéssel dolgozó interaktív animációs runtime-ok kereskedelmi funkcióterületet jelentenek. | Nem átvilágított. Az adatmodell saját tervezés, nem szabadalmi jogtisztaság. |
| FABRIK, mean-value koordináták, korlátozott biharmonikus súlyok (`rig`, `cage`, `weights`) | Publikált algoritmusok; egyes szerzők és munkáltatók szabadalmakat jelentettek be a deformációs és skinning technikák körül. | Nem átvilágított. A FABRIK-ot (Aristidou és Lasenby, 2011) és az MVC-t (Floater, 2003) a tanulmányok alapján használjuk; a weights modul egyszerű harmonikus diffúziót használ, nem a publikált korlátozott biharmonikus megoldót. |
| Stencil-then-cover útvonalkitöltés (`osatura-gpu`) | A technika a GPU-s útvonalrenderelési kutatásból és gyártói kiterjesztésekből ered. | Nem átvilágított. Itt kizárólag a szabványos wgpu API-n keresztül, egyszerű stencil pufferrel használjuk. |
| Videó- és hangkodekek ffmpeg-en keresztül (`export` .mp4 / .webm, `soundtrack`) | A H.264-re és az AAC-re licencelési közösségek (MPEG LA / Via LA és mások) vonatkoznak. A VP9-et és az AV1-et jogdíjmentesként forgalmazzák, de mindkettőre jelentettek be szabadalmi közösségi és egyéni szabadalmi igényeket. | Nem értékelt. A VP9-et/AV1-et ne kezeld jogtisztaként. |

## ffmpeg
Az ffmpeg egy **külső alfolyamat**, amelyet a felhasználó biztosít. Az OSATURA nem linkeli, nem szállítja, és semmilyen kiadásba nem szabad belecsomagolni: a becsomagolás a terjesztőt tenné felelőssé az ffmpeg saját licencéért (LGPL/GPL, a buildtől függően) és a fenti kodeklicencekért. Minden kodekopció a felhasználó saját döntése a saját telepítésén.

## Kereskedelmi kiadás előtt
Egy szabadalmi ügyvivővel vizsgáltasd át a fenti területeket azokban a joghatóságokban, ahol a szoftvert terjeszteni fogják, és az eredményt rögzítsd itt és a CLEANROOM.md-ben (5. szabály).
