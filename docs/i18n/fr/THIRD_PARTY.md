# Licences des composants tiers (généré)

*Généré par `scripts/licence-audit.sh` à partir de `cargo metadata --all-features` (toutes les cibles, dépendances normales + de compilation, dépendances de développement exclues). Ne pas modifier à la main. Ceci n'est pas un avis juridique ; les expressions SPDX sont celles que chaque crate déclare dans son `Cargo.toml`, elles n'ont pas été vérifiées indépendamment par rapport aux fichiers sources. Les crates de l'espace de travail sont elles-mêmes sous MPL-2.0 (core, capi, wasm, gpu) et AGPL-3.0-or-later (osatura-cli) ; voir LICENSE.*

**363 crates tierces** (nom+version distincts). Groupe A (à examiner) : 6 ; groupe B (licences permissives / domaine public / de données notables) : 27 ; groupe C (style MIT / Apache-2.0) : 330.

## A. À examiner : une clause de copyleft sans alternative entièrement permissive

| Crate | Version | Expression SPDX | Source | Requise par (espace de travail) | Note |
|---|---|---|---|---|---|
| cssparser | 0.37.0 | MPL-2.0 | registre crates.io | osatura-gui | clause de copyleft ou non reconnue ; aucune alternative entièrement permissive dans l'expression ; MPL = copyleft (faible) au niveau du fichier |
| cssparser-macros | 0.7.1 | MPL-2.0 | registre crates.io | osatura-gui | clause de copyleft ou non reconnue ; aucune alternative entièrement permissive dans l'expression ; MPL = copyleft (faible) au niveau du fichier |
| dtoa-short | 0.3.5 | MPL-2.0 | registre crates.io | osatura-gui | clause de copyleft ou non reconnue ; aucune alternative entièrement permissive dans l'expression ; MPL = copyleft (faible) au niveau du fichier |
| option-ext | 0.2.0 | MPL-2.0 | registre crates.io | osatura-gui | clause de copyleft ou non reconnue ; aucune alternative entièrement permissive dans l'expression ; MPL = copyleft (faible) au niveau du fichier |
| selectors | 0.38.0 | MPL-2.0 | registre crates.io | osatura-gui | clause de copyleft ou non reconnue ; aucune alternative entièrement permissive dans l'expression ; MPL = copyleft (faible) au niveau du fichier |
| smartstring | 1.0.1 | MPL-2.0+ | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm | clause de copyleft ou non reconnue ; aucune alternative entièrement permissive dans l'expression ; MPL = copyleft (faible) au niveau du fichier |

## B. Licences permissives, de domaine public ou de fichiers de données notables (aucune option MIT/Apache proposée)

| Crate | Version | Expression SPDX | Source | Requise par (espace de travail) | Note |
|---|---|---|---|---|---|
| arrayref | 0.3.9 | BSD-2-Clause | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm | uniquement les termes BSD-2-Clause |
| foldhash | 0.2.0 | Zlib | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm | uniquement les termes Zlib |
| icu_collections | 2.4.0 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| icu_locale_core | 2.4.0 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| icu_normalizer | 2.4.0 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| icu_normalizer_data | 2.4.0 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| icu_properties | 2.4.0 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| icu_properties_data | 2.4.0 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| icu_provider | 2.4.0 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| libloading | 0.8.9 | ISC | registre crates.io | osatura-cli, osatura-gpu | uniquement les termes ISC |
| litemap | 0.8.3 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| potential_utf | 0.1.6 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| slotmap | 1.1.1 | Zlib | registre crates.io | osatura-cli, osatura-gpu | uniquement les termes Zlib |
| tiny-keccak | 2.0.2 | CC0-1.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm | uniquement les termes CC0-1.0 |
| tiny-skia | 0.12.0 | BSD-3-Clause | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm | uniquement les termes BSD-3-Clause |
| tiny-skia-path | 0.12.0 | BSD-3-Clause | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm | uniquement les termes BSD-3-Clause |
| tinystr | 0.8.5 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm | exige aussi Unicode-3.0 (AND) : termes relatifs aux fichiers de données en plus de MIT/Apache |
| writeable | 0.6.4 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| yoke | 0.8.4 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| yoke-derive | 0.8.4 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| zerofrom | 0.1.8 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| zerofrom-derive | 0.1.8 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| zerotrie | 0.2.5 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| zerovec | 0.11.9 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| zerovec-derive | 0.11.7 | Unicode-3.0 | registre crates.io | osatura-gui | uniquement les termes Unicode-3.0 |
| zlib-rs | 0.6.8 | Zlib | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm | uniquement les termes Zlib |

## C. MIT / Apache-2.0 (ou une double licence les incluant)

| Crate | Version | Expression SPDX | Source | Requise par (espace de travail) | Note |
|---|---|---|---|---|---|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| ahash | 0.8.12 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| android_system_properties | 0.1.6 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| ash | 0.38.0+1.3.281 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| atk | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| atk-sys | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| base64 | 0.23.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| bit-set | 0.10.0 | Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu |  |
| bit-set | 0.8.0 | Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| bit-vec | 0.8.0 | Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| bit-vec | 0.9.1 | Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu |  |
| bitflags | 1.3.2 | MIT/Apache-2.0 | registre crates.io | osatura-gui |  |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| block2 | 0.6.2 | MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 | registry crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| bytes | 1.12.1 | MIT | registre crates.io | osatura-gui |  |
| cairo-rs | 0.18.5 | MIT | registre crates.io | osatura-gui |  |
| cairo-sys-rs | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| cc | 1.10.0 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| cesu8 | 1.1.0 | Apache-2.0/MIT | registre crates.io | osatura-gui |  |
| cfg-expr | 0.15.8 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| cfg_aliases | 0.2.2 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| codespan-reporting | 0.13.1 | Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| color_quant | 1.1.0 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| combine | 4.6.9 | MIT | registre crates.io | osatura-gui |  |
| const-random | 0.1.18 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| const-random-macro | 0.1.16 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| cookie | 0.18.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| core-foundation | 0.10.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| core-graphics | 0.25.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| core-graphics-types | 0.2.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| crc32fast | 1.5.2 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| crossbeam-channel | 0.5.17 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| crunchy | 0.2.4 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| dbus | 0.9.12 | Apache-2.0/MIT | registre crates.io | osatura-gui |  |
| deranged | 0.5.9 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| derive_more | 2.1.1 | MIT | registre crates.io | osatura-gui |  |
| derive_more-impl | 2.1.1 | MIT | registre crates.io | osatura-gui |  |
| digest | 0.10.7 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| dirs | 7.0.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| dirs-sys | 0.5.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| dispatch2 | 0.3.1 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| displaydoc | 0.2.7 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| dlib | 0.5.3 | MIT | registre crates.io | osatura-cli, osatura-gpu |  |
| dlopen2 | 0.8.2 | MIT | registre crates.io | osatura-gui |  |
| dlopen2_derive | 0.4.3 | MIT | registre crates.io | osatura-gui |  |
| document-features | 0.2.12 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| dom_query | 0.28.0 | MIT | registre crates.io | osatura-gui |  |
| dpi | 0.1.2 | Apache-2.0 AND MIT | registre crates.io | osatura-gui |  |
| dtoa | 1.0.11 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | registre crates.io | osatura-gui |  |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| field-offset | 0.3.6 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| find-msvc-tools | 0.1.15 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| flate2 | 1.1.10 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| foreign-types | 0.5.0 | MIT/Apache-2.0 | registre crates.io | osatura-gui |  |
| foreign-types-macros | 0.2.4 | MIT/Apache-2.0 | registre crates.io | osatura-gui |  |
| foreign-types-shared | 0.3.1 | MIT/Apache-2.0 | registre crates.io | osatura-gui |  |
| form_urlencoded | 1.2.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| futures-channel | 0.3.34 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| futures-core | 0.3.34 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| futures-executor | 0.3.34 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| futures-io | 0.3.34 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| futures-macro | 0.3.34 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| futures-task | 0.3.34 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| futures-util | 0.3.34 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| gdk | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| gdk-pixbuf | 0.18.5 | MIT | registre crates.io | osatura-gui |  |
| gdk-pixbuf-sys | 0.18.0 | MIT | registre crates.io | osatura-gui |  |
| gdk-sys | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| gdkwayland-sys | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| gdkx11 | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| gdkx11-sys | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| generic-array | 0.14.7 | MIT | registre crates.io | osatura-gui |  |
| getrandom | 0.2.17 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| gif | 0.14.2 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| gio | 0.18.4 | MIT | registre crates.io | osatura-gui |  |
| gio-sys | 0.18.1 | MIT | registre crates.io | osatura-gui |  |
| gl_generator | 0.14.0 | Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| glib | 0.18.5 | MIT | registre crates.io | osatura-gui |  |
| glib-macros | 0.18.5 | MIT | registre crates.io | osatura-gui |  |
| glib-sys | 0.18.1 | MIT | registre crates.io | osatura-gui |  |
| glow | 0.17.0 | MIT OR Apache-2.0 OR Zlib | registre crates.io | osatura-cli, osatura-gpu |  |
| glutin_wgl_sys | 0.6.1 | Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| gobject-sys | 0.18.0 | MIT | registre crates.io | osatura-gui |  |
| gpu-allocator | 0.28.0 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| gtk | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| gtk-sys | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| gtk3-macros | 0.18.2 | MIT | registre crates.io | osatura-gui |  |
| half | 2.7.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| heck | 0.4.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| heck | 0.5.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| hound | 3.5.1 | Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| html5ever | 0.39.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| http | 1.5.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| i_float | 5.0.0 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| i_key_sort | 0.12.0 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| i_overlay | 9.1.0 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| i_shape | 5.0.0 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| i_tree | 0.20.0 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| idna | 1.1.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| itoa | 1.0.18 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| javascriptcore-rs | 1.1.2 | MIT | registre crates.io | osatura-gui |  |
| javascriptcore-rs-sys | 1.1.1 | MIT | registre crates.io | osatura-gui |  |
| jni | 0.21.1 | MIT/Apache-2.0 | registre crates.io | osatura-gui |  |
| jni-sys | 0.3.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| jni-sys | 0.4.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| jni-sys-macros | 0.4.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| jpeg-decoder | 0.3.2 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| js-sys | 0.3.106 | MIT OR Apache-2.0 | registry crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| khronos-egl | 6.0.0 | MIT/Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| khronos_api | 3.1.0 | Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| libc | 0.2.191 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| libdbus-sys | 0.2.7 | Apache-2.0/MIT | registre crates.io | osatura-gui |  |
| libm | 0.2.16 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| libredox | 0.1.25 | MIT | registre crates.io | osatura-gui |  |
| litrs | 1.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| log | 0.4.34 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| markup5ever | 0.39.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| memchr | 2.8.3 | Unlicense OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| memoffset | 0.9.1 | MIT | registre crates.io | osatura-gui |  |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| naga | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| naga-types | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| ndk | 0.9.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| ndk-context | 0.1.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| ndk-sys | 0.6.0+11769913 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| new_debug_unreachable | 1.0.6 | MIT | registre crates.io | osatura-gui |  |
| nix | 0.31.3 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| num-conv | 0.2.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| num_enum | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| num_enum_derive | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| objc2 | 0.6.5 | MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| objc2-app-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| objc2-cloud-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| objc2-core-data | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| objc2-core-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| objc2-core-graphics | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| objc2-core-image | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| objc2-core-location | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| objc2-core-text | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| objc2-encode | 4.1.0 | MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| objc2-exception-helper | 0.1.1 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| objc2-foundation | 0.3.2 | MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| objc2-io-surface | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| objc2-metal | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| objc2-quartz-core | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| objc2-ui-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| objc2-user-notifications | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| objc2-web-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| ordered-float | 5.5.0 | MIT | registre crates.io | osatura-cli, osatura-gpu |  |
| pango | 0.18.3 | MIT | registre crates.io | osatura-gui |  |
| pango-sys | 0.18.0 | MIT | registre crates.io | osatura-gui |  |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| phf | 0.13.1 | MIT | registre crates.io | osatura-gui |  |
| phf_codegen | 0.13.1 | MIT | registre crates.io | osatura-gui |  |
| phf_generator | 0.13.1 | MIT | registre crates.io | osatura-gui |  |
| phf_macros | 0.13.1 | MIT | registre crates.io | osatura-gui |  |
| phf_shared | 0.13.1 | MIT | registre crates.io | osatura-gui |  |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| png | 0.18.1 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| pollster | 1.0.1 | Apache-2.0/MIT | registre crates.io | osatura-cli, osatura-gpu |  |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| portable-atomic-util | 0.2.8 | Apache-2.0 OR MIT | registre crates.io | osatura-cli, osatura-gpu |  |
| powerfmt | 0.2.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| precomputed-hash | 0.1.1 | MIT | registre crates.io | osatura-gui |  |
| presser | 0.3.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| proc-macro-crate | 1.3.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| proc-macro-crate | 2.0.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| proc-macro-error | 1.0.4 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| proc-macro-error-attr | 1.0.4 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| profiling | 1.0.18 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| quote | 1.0.47 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| range-alloc | 0.1.5 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| raw-window-metal | 1.1.0 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| redox_syscall | 0.5.18 | MIT | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| redox_users | 0.5.3 | MIT | registre crates.io | osatura-gui |  |
| renderdoc-sys | 1.1.0 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| rfd | 0.17.2 | MIT | registry crates.io | osatura-gui |  |
| rhai | 1.26.1 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| rhai_codegen | 3.2.0 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| rustc-hash | 1.1.0 | Apache-2.0/MIT | registre crates.io | osatura-cli, osatura-gpu |  |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| same-file | 1.0.6 | Unlicense/MIT | registre crates.io | osatura-gui |  |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| semver | 1.0.28 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| serde | 1.0.229 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| serde_spanned | 0.6.9 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| servo_arc | 0.4.3 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| shlex | 2.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| simd-adler32 | 0.3.10 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| siphasher | 1.0.4 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| slab | 0.4.12 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| smallvec | 1.16.3 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| soup3 | 0.5.0 | MIT | registre crates.io | osatura-gui |  |
| soup3-sys | 0.5.0 | MIT | registre crates.io | osatura-gui |  |
| spirv | 0.4.0+sdk-1.4.341.0 | Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| strict-num | 0.1.1 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| string_cache | 0.9.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| string_cache_codegen | 0.6.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| syn | 1.0.109 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| syn | 2.0.119 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| syn | 3.0.9 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| synstructure | 0.14.0 | MIT | registre crates.io | osatura-gui |  |
| system-deps | 6.2.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| tao | 0.37.2 | Apache-2.0 | registre crates.io | osatura-gui |  |
| tao-macros | 0.1.4 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| target-lexicon | 0.12.16 | Apache-2.0 WITH LLVM-exception | registre crates.io | osatura-gui |  |
| tendril | 0.5.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| thin-vec | 0.2.21 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| thiserror | 1.0.69 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| time | 0.3.55 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| time-core | 0.1.9 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| time-macros | 0.2.32 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| tokio | 1.54.0 | MIT | registry crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| toml | 0.8.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| toml_datetime | 0.6.3 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| toml_edit | 0.19.15 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| toml_edit | 0.20.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| toml_edit | 0.25.17+spec-1.1.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| typed-path | 0.12.4 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| typenum | 1.20.1 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| unicode-width | 0.2.2 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| url | 2.5.8 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| utf8_iter | 1.0.4 | Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| version-compare | 0.2.1 | MIT | registre crates.io | osatura-gui |  |
| version_check | 0.9.5 | MIT/Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| walkdir | 2.5.0 | Unlicense/MIT | registre crates.io | osatura-gui |  |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| wasm-bindgen | 0.2.129 | MIT OR Apache-2.0 | registry crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| wasm-bindgen-futures | 0.4.79 | MIT OR Apache-2.0 | registry crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| wasm-bindgen-macro | 0.2.129 | MIT OR Apache-2.0 | registry crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| wasm-bindgen-macro-support | 0.2.129 | MIT OR Apache-2.0 | registry crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| wasm-bindgen-shared | 0.2.129 | MIT OR Apache-2.0 | registry crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-gui, osatura-wasm |  |
| wayland-sys | 0.31.11 | MIT | registre crates.io | osatura-cli, osatura-gpu |  |
| web-sys | 0.3.106 | MIT OR Apache-2.0 | registry crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| web-time | 1.1.0 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| web_atoms | 0.2.6 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| webkit2gtk | 2.0.2 | MIT | registre crates.io | osatura-gui |  |
| webkit2gtk-sys | 2.0.2 | MIT | registre crates.io | osatura-gui |  |
| webview2-com | 0.39.1 | MIT | registre crates.io | osatura-gui |  |
| webview2-com-macros | 0.8.1 | MIT | registre crates.io | osatura-gui |  |
| webview2-com-sys | 0.39.1 | MIT | registre crates.io | osatura-gui |  |
| weezl | 0.1.12 | MIT OR Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| wgpu | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| wgpu-core | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| wgpu-core-deps-apple | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| wgpu-core-deps-emscripten | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| wgpu-core-deps-windows-linux-android | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| wgpu-hal | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| wgpu-naga-bridge | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| wgpu-types | 30.0.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu |  |
| winapi | 0.3.9 | MIT/Apache-2.0 | registre crates.io | osatura-gui |  |
| winapi-i686-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | registre crates.io | osatura-gui |  |
| winapi-util | 0.1.11 | Unlicense OR MIT | registre crates.io | osatura-gui |  |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | registre crates.io | osatura-gui |  |
| windows | 0.62.2 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-collections | 0.3.2 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-core | 0.62.2 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-future | 0.3.2 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-implement | 0.60.2 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-interface | 0.59.3 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-numerics | 0.3.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-result | 0.4.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-strings | 0.5.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-sys | 0.45.0 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows-targets | 0.42.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows-threading | 0.2.1 | MIT OR Apache-2.0 | registre crates.io | osatura-cli, osatura-gpu, osatura-gui |  |
| windows-version | 0.1.7 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows_aarch64_gnullvm | 0.42.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows_aarch64_msvc | 0.42.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows_i686_gnu | 0.42.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows_i686_msvc | 0.42.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows_x86_64_gnu | 0.42.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows_x86_64_gnullvm | 0.42.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| windows_x86_64_msvc | 0.42.2 | MIT OR Apache-2.0 | registre crates.io | osatura-gui |  |
| winnow | 0.5.40 | MIT | registre crates.io | osatura-gui |  |
| winnow | 1.0.4 | MIT | registre crates.io | osatura-gui |  |
| wry | 0.57.1 | Apache-2.0 OR MIT | registre crates.io | osatura-gui |  |
| x11 | 2.21.0 | MIT | registre crates.io | osatura-gui |  |
| x11-dl | 2.21.0 | MIT | registre crates.io | osatura-gui |  |
| xml-rs | 0.8.29 | MIT | registre crates.io | osatura-cli, osatura-gpu |  |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| zip | 9.0.5 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| zmij | 1.0.23 | MIT | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |
| zopfli | 0.8.4 | Apache-2.0 | registre crates.io | osatura-capi, osatura-cli, osatura-core, osatura-gpu, osatura-wasm |  |


## Sources des algorithmes (idées et constantes, pas de code copié)
- **Ajustement de courbes de Bézier** (`autotrace/`, `lod/`) : la structure de l'algorithme suit P. J. Schneider, « An Algorithm for Automatically Fitting Digitized Curves », *Graphics Gems I* (A. Glassner, dir., Academic Press, 1990) ; le code a été écrit de façon indépendante. La distribution du code de Graphics Gems indique, dans son README/sa note de licence, que son code peut être utilisé dans tout programme, produit ou bibliothèque, commercial ou non, que la mention de la source est appréciée mais non obligatoire, et qu'il est fourni tel quel, sans responsabilité ; seul le fait de s'approprier le texte du code et de le revendre est exclu. Nous n'avons pas copié ce code ; cette note consigne les termes permissifs au cas où un relecteur le demanderait (vérifiez la formulation dans le README du dépôt Graphics Gems).
- **Constantes de hachage** (`noise.rs`) : finaliseur xorshift-multiply de `lowbias32` de Chris Wellons (domaine public, Hash Prospector, 2018) ; 0x9E3779B1 (2^32 / nombre d'or, Knuth) ; 0x85EBCA6B (finaliseur de MurmurHash3, Austin Appleby, domaine public).
- **Easing** : `Ease::Bounce` et `Ease::Back` sont des dérivations propres (CLEANROOM.md) ; les constantes BSD-3-Clause de Robert Penner ne sont plus utilisées.
- Les autres algorithmes sont répertoriés par module, avec leurs articles et manuels, dans CLEANROOM.md.

## Modèle fourni (attribution)
`osatura-core/models/face23_synth.bin` (un fichier de poids de 650 Ko d'un détecteur de visages pour `facedet`) est (c) 2026 Alexandru-Ionuț Chiuță (Alexio), sous licence CC-BY-4.0 (https://creativecommons.org/licenses/by/4.0/ ; mentionnez l'auteur et indiquez les modifications). Il a été entraîné **uniquement sur des visages générés par le générateur de visages synthétiques propre à ce projet** (`facegen.rs`) ; aucune photographie, aucun jeu de données tiers et aucun poids pré-entraîné tiers n'ont été utilisés.
## Compilations sans la crate MPL-2.0+ (smartstring)

Le tableau ci-dessus est l'union sur l'ensemble de l'espace de travail avec toutes les fonctionnalités, où la CLI par défaut active `scripting` et où Cargo unifie les fonctionnalités. Vérifié par paquet avec `cargo tree -i smartstring -e normal,build` :

| Compilation | smartstring dans l'arbre ? |
|---|---|
| `-p osatura-core --no-default-features` | non |
| `-p osatura-cli --no-default-features` | non |
| `-p osatura-capi` | non |
| `-p osatura-wasm` | non |
| `-p osatura-cli` | oui |
