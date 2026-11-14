<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Política de firma de código

Firma de código gratuita proporcionada por [SignPath.io](https://about.signpath.io/), certificado de la [SignPath Foundation](https://signpath.org/).

*Estado (25 oct 2026): el proyecto ha solicitado, o está a punto de solicitar, un certificado gratuito de la SignPath Foundation. Hasta que una versión diga lo contrario, los archivos de Windows **no** están firmados con Authenticode; cada archivo de versión ya lleva una firma Sigstore y una atestación de procedencia de compilación de GitHub (véase [Verificación](#verificación)).*

## Qué se firma

Solo se firman los archivos compilados a partir de este repositorio ([github.com/chiuta/osatura](https://github.com/chiuta/osatura)) por su flujo de publicación ([`.github/workflows/release.yml`](../../../.github/workflows/release.yml)) en runners alojados por GitHub. Nada compilado en un ordenador personal se firma nunca, y no se firma ningún binario de terceros.

| Archivo (en `osatura-windows-x86_64.zip`) | Qué es | Licencia |
|---|---|---|
| `osatura.exe` | herramienta de línea de comandos y estudio en el navegador | AGPL-3.0-or-later |
| `osatura-studio.exe` | el estudio en una ventana nativa (WebView2) | AGPL-3.0-or-later |
| `lib/osatura_capi.dll` | biblioteca C | MPL-2.0 |

Cada archivo firmado lleva el nombre de producto `OSATURA` y la versión de la publicación en sus propiedades (incrustados por `build.rs` en cada crate y comprobados por el flujo de trabajo antes de firmar nada). La configuración de artefactos de SignPath se guarda en [`.signpath/artifact-configuration.xml`](../../../.signpath/artifact-configuration.xml).

## Roles del equipo

El proyecto cuenta actualmente con un único mantenedor, que ocupa los tres roles:

| Rol | Quién |
|---|---|
| Committers y revisores | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |
| Aprobadores | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |

* Los cambios de cualquier persona sin acceso de escritura llegan como pull requests y los revisa un committer antes de fusionarlos ([CONTRIBUTING.md](CONTRIBUTING.md)).
* Una solicitud de firma solo la crea el flujo de publicación para una etiqueta de versión (`vMAJOR.MINOR.PATCH[-pre]`) cuyo commit esté en `main`, y cada solicitud la aprueba a mano un aprobador en SignPath.
* Toda persona con uno de estos roles debe usar autenticación multifactor en GitHub y SignPath (condición de la SignPath Foundation; el mantenedor lo confirma para cada cuenta cuando se concede el certificado).
* Cualquier cambio en estos roles se registra en este archivo.

## Política de privacidad

Este programa no transferirá ninguna información a otros sistemas en red salvo que lo solicite expresamente el usuario o la persona que lo instala o lo opera. Detalles: [PRIVACY.md](PRIVACY.md).

## Cambios en tu sistema y desinstalación

OSATURA no tiene instalador, no escribe nada en el registro y no instala ningún servicio, controlador ni entrada de inicio. Los pocos archivos que puede crear y cómo eliminarlo todo se enumeran en [UNINSTALL.md](../../UNINSTALL.md).

## Licencias

Todo el código es de código abierto con licencias aprobadas por la OSI: MPL-2.0 (bibliotecas) y AGPL-3.0-or-later (herramienta de línea de comandos, estudio y aplicación de ventana); la documentación es CC-BY-4.0 ([COPYRIGHT](../../../COPYRIGHT), [LICENSING.md](LICENSING.md)). Los archivos firmados no contienen código propietario, y OSATURA no se ofrece bajo ninguna licencia comercial o propietaria (sin doble licencia).

## Verificación

* **Authenticode** (cuando esté disponible): clic derecho en un archivo en el Explorador de Windows > Propiedades > Firmas digitales; el firmante es SignPath Foundation.
* **Origen de cada archivo de versión**: `gh attestation verify <file> -R chiuta/osatura`, o `cosign verify-blob` con el paquete `.sigstore.json` ([VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md)).

## Informar de un problema

Si crees que un archivo firmado para OSATURA es malicioso, no se compiló a partir de este repositorio o que se incumplió esta política, escribe a <alexio@trom.tf> o abre un issue; los informes de seguridad siguen [SECURITY.md](SECURITY.md). Los problemas con el propio certificado también pueden comunicarse a la [SignPath Foundation](https://signpath.org/).
