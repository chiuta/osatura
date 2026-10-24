# OSATURA — licencias (DECIDIDO: Variante A, aplicada el 22 oct 2026; véanse LICENSE y LICENSES/)

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/LICENSING.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/LICENSING.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/LICENSING.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/LICENSING.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/LICENSING.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/LICENSING.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/LICENSING.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/LICENSING.md)

**Estado:** Se eligió y aplicó la Variante A que se describe a continuación: MPL-2.0 para las bibliotecas (core, capi, wasm, gpu), AGPL-3.0-or-later para `osatura-cli` (CLI + estudio) y `osatura-gui` (el estudio en una ventana nativa), CC-BY-4.0 para la documentación y los recursos y modelos sintéticos, política de marca en TRADEMARK.md, DCO en CONTRIBUTING.md, cabeceras SPDX en todos los archivos .rs/.js/.mjs/.sh y REUSE.toml para el resto, oferta de código fuente de la sección 13 de la AGPL (`/source`, `--source-url`) y campos `license` de Cargo establecidos. Acciones pendientes del titular: revisión por un abogado, búsqueda de marca (EUIPO/OSIM), análisis de patentes (docs/PATENT_NOTES.md), un CLA si se desea doble licencia, y empaquetar los avisos de terceros con las versiones binarias.


*Esto no es asesoramiento jurídico. La licencia está DECIDIDA (Variante A, véase LICENSE); las secciones siguientes se conservan como justificación y como alternativas consideradas. Haz que un abogado (idealmente uno que conozca el derecho de autor rumano/de la UE) lo confirme antes de la primera versión pública. Contribuciones: no existe ningún derecho de relicenciar ni de aplicar doble licencia a las contribuciones de terceros hasta que haya un CLA (CONTRIBUTING.md).*

## 1. Lo que la licencia debe hacer por ti
Tus ingresos son las **donaciones (Buy Me a Coffee)**, no las tarifas de licencia. Por tanto, la licencia no genera el dinero: lo generan **la buena voluntad, la visibilidad y la confianza**. Tiene tres cometidos:
1. **Maximizar los usuarios** (más usuarios → más patrocinadores): debe ser libre de usar, también en trabajos comerciales.
2. **Impedir que una empresa tome el código, lo cierre y lo venda** (o lo ofrezca como servicio de pago) sin devolver nada.
3. **Conservar tu derecho a vender excepciones** (licencia comercial, soporte) y a defender el nombre.

También debe encajar con tus convicciones **trade-free / TROM**: el software no debe estar mutilado, financiado con publicidad ni vinculado a la recopilación de datos. (Las licencias que prohíben el uso comercial, p. ej. CC-NC o Polyform-NC, *no* son de código abierto y no equivalen a «trade-free»; comprueba la redacción propia de TROM antes de adoptar cualquiera de ellas.)

## 2. Hallazgos que condicionan la elección
- **Las dependencias no son un obstáculo.** Auditoría de todos los crates de terceros (363 en la regeneración del 24 oct 2026 con `scripts/licence-audit.sh`, contados en todas las plataformas juntas tras añadir la aplicación de ventana; eran 160 el 22 oct; la cifra cambia, véase docs/THIRD_PARTY.md): todos son MIT, Apache-2.0, BSD, Zlib, ISC, Unlicense, CC0, Unicode o dobles licencias de estas, salvo seis crates MPL-2.0 con copyleft débil a nivel de archivo: smartstring (MPL-2.0+, alcanzado solo a través de la característica `scripting`, que está ACTIVADA por defecto en `osatura-core` y `osatura-cli`) y `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors` y `option-ext` (alcanzados solo a través de la aplicación de ventana `osatura-gui`); una compilación de las bibliotecas con `--no-default-features` no incluye ningún crate de terceros con MPL. Pueden combinarse con AGPL, MPL o una licencia propietaria; el crate con MPL exige conservar su código fuente y su aviso.
- **Titularidad de los derechos de autor.** Gran parte del código se escribió con Claude. El derecho de autor se asocia, por lo general, a la contribución creativa *humana* (la guía de la Oficina de Derechos de Autor de EE. UU.; el derecho de la UE y el rumano exigen asimismo la creación intelectual propia de un autor humano). Consecuencias prácticas: (a) puedes y debes licenciar el proyecto como su autor/compilador humano, pero (b) no prometas exclusividad sobre cada línea, (c) conserva en el repositorio tus propias notas de diseño, selecciones y ediciones como prueba de autoría, (d) un DCO solo certifica el origen; para relicenciar contribuciones más adelante (p. ej. doble licencia) hace falta un CLA o una cesión de salida explícita, y NO existe ninguno.
- **Ninguna licencia puede obligar a donar.** Nunca condiciones el código a una donación; un fork simplemente quitaría la barrera. Vende *comodidad y extras*, no permiso.

## 3. Paquete recomendado — «Biblioteca de núcleo abierto, aplicación protegida» (todas las variantes siguientes están listas para aplicarse)

### ★ Variante A (recomendada): biblioteca MPL-2.0 + aplicaciones AGPL-3.0 + marca
| Parte | Licencia | Por qué |
|---|---|---|
| `osatura-core` (motor, evaluador, formatos, exportadores) y cualquier futuro runtime/WASM/ABI de C | **MPL-2.0** | Copyleft a nivel de archivo: cualquiera puede incrustarlo en juegos/aplicaciones (cerrados o abiertos), pero los cambios en *nuestros archivos* deben compartirse. Incluye una **concesión de patentes** explícita y la terminación en caso de demanda por patentes. Maximiza la adopción, que es lo que necesitan las donaciones. |
| `osatura-cli`, `osatura-gui`, estudio (navegador/ventana) | **AGPL-3.0-or-later** | Impide los forks cerrados y los clones de «OSATURA como servicio web de pago»; los *proyectos y renders* de los usuarios siguen siendo enteramente suyos. |
| Nombre «OSATURA», logotipo | **Política de marca** (archivo aparte) | Los forks deben cambiar de nombre; protege a los patrocinadores de las imitaciones. Comprueba la disponibilidad (EUIPO/OSIM/OMPI) antes del lanzamiento. |
| Rigs de ejemplo, arte, audio que crees | **CC BY 4.0** (o CC BY-SA) | Libres de reutilizar con atribución; el arte de terceros conserva su propia licencia. |
| Documentación | **CC BY 4.0** | |
<!-- REUSE-IgnoreStart -->
Cargo: `license = "MPL-2.0"` (core), `license = "AGPL-3.0-or-later"` (cli). SPDX header per file: `// SPDX-License-Identifier: MPL-2.0`.
<!-- REUSE-IgnoreEnd -->
**Doble licencia:** posible solo para el código que haya escrito únicamente el titular, o tras un CLA/cesión de salida de todos los colaboradores. NO es posible hoy para el código aportado por terceros (CONTRIBUTING.md).

### Variante B: AGPL-3.0-or-later en todo (+ licencia comercial bajo petición)
La posición más fuerte contra el secuestro del proyecto y la más cercana al espíritu trade-free. Coste: las empresas y los motores de juegos evitarán incrustar el runtime, de modo que habrá menos usuarios y menos patrocinadores indirectos. Elígela si la protección importa más que el alcance.

### Variante C: Apache-2.0 en todo (no elegida; el workspace aplica la Variante A: bibliotecas MPL-2.0 y `license = "AGPL-3.0-or-later"` para `osatura-cli`)
Máximo alcance y cero fricción; incluye concesión de patentes. Coste: cualquiera, incluida una gran empresa, puede cerrarlo y venderlo sin ninguna obligación. Las donaciones dependen entonces únicamente de la buena voluntad. Elígela solo si quieres ser un servicio de utilidad pública.

### Variante D: Código disponible (BUSL-1.1 o Polyform Shield/NC con fecha de apertura diferida)
Impide la competencia comercial durante N años. Coste: **no es código abierto**; choca con la historia de clean-room/formatos abiertos, con las contribuciones de la comunidad y con los valores trade-free; muchos usuarios y distribuciones la rechazarán. **No recomendada.**

| | Alcance | Contra el secuestro | Concesión de patentes | Ingresos por doble licencia | Encaja con trade-free |
|---|---|---|---|---|---|
| **A núcleo MPL + aplicaciones AGPL** | alto | bueno | sí | sí | sí |
| B todo AGPL | medio | el más fuerte | sí (v3) | sí | sí |
| C todo Apache | el más alto | ninguno | sí | no | en parte |
| D código disponible | bajo | fuerte | n/a | sí | no |

## 4. Cómo ganar realmente con las donaciones (todo compatible con la Variante A)
1. **Página de Buy Me a Coffee** (`buymeacoffee.com/echo.of.the.strings`) enlazada desde el README, el texto de `--version` de la CLI, el pie del estudio y la línea de créditos del reproductor HTML (un crédito *eliminable*, nunca obligatorio).
2. **Membresías, no muros de pago:** niveles mensuales para compilaciones tempranas, votación sobre el siguiente elemento de la hoja de ruta, nombre en `SUPPORTERS.md`, chat privado. Todos reciben el mismo código; los miembros reciben *tiempo y voz*.
3. **Extras de pago que no son el código:** binarios/instaladores firmados y listos para ejecutar, paquetes de recursos (rigs, fondos, bibliotecas de ciclos de caminar) con una licencia que permita su uso en películas/juegos pero no la reventa del paquete en sí, cursos en vídeo, talleres.
4. **Encargos y soporte:** servicios de rigging, exportadores a medida, consultoría de integración; la licencia comercial dual para quienes lo incrustan.
5. **Niveles de patrocinio para organizaciones** (logotipo en el README) y solicitudes de subvenciones a fondos creativos y de código abierto (NLnet, programas del tipo Sovereign Tech, fondos de cultura): pagan precisamente por este tipo de infraestructura abierta.
6. **Página de transparencia:** publica los ingresos y lo que financiaron; aumenta de forma medible el apoyo recurrente.
7. Mantén visibles **Patreon** (`patreon.com/c/alexio_tf`) y BMC, pero dale a cada uno un propósito distinto (BMC para agradecimientos puntuales; Patreon para apoyo recurrente) para que los patrocinadores no se confundan.

## 5. Lista de comprobación para aplicar la Variante A (≈ 1 hora de trabajo una vez elegida)
- [x] `LICENSES/MPL-2.0.txt`, `LICENSES/AGPL-3.0-or-later.txt`, `LICENSES/CC-BY-4.0.txt`, `LICENSES/LicenseRef-third-party.txt` (estructura REUSE) más el resumen `LICENSE`; `REUSE.toml` cubre los archivos que no son código fuente.
- [x] `license =` establecido en los seis archivos `Cargo.toml`; cabeceras SPDX en todos los archivos .rs/.js/.mjs/.sh, comprobadas por `osatura-cli/tests/spdx_audit.rs`.
- [x] `docs/THIRD_PARTY.md` y `docs/THIRD_PARTY_NOTICES.md`, ambos regenerados por `scripts/licence-audit.sh`.
- [x] `CONTRIBUTING.md` con el texto del DCO 1.1. Sigue haciendo falta un CLA antes de cualquier doble licencia.
- [x] `TRADEMARK.md` (permitido: «compatible con OSATURA», redistribución sin modificar; no permitido: compilaciones modificadas bajo el mismo nombre).
- [x] `.github/FUNDING.yml` (`buy_me_a_coffee: echo.of.the.strings`) para que la plataforma muestre un botón de patrocinio.
- [~] Búsqueda de marca: el primer nombre, ARMATURA, colisionaba con una marca de las clases 9/42 (EUIPO, 24 oct 2026), por lo que el proyecto pasó a llamarse OSATURA; la búsqueda en la EUIPO de «osatura» no encontró ningún conflicto (un resultado sin relación y caducado en la clase 10). Los nombres de paquete `osatura` en crates.io, npm y PyPI están libres (comprobado el 24 oct 2026), pero existe una marca cercana, OSATRA (Open-Xchange AG, registro de EE. UU. 7459236, clases 9/38/42). Las búsquedas en OSIM / TMview / OMPI / de dominios y el dictamen de un abogado siguen pendientes, véase [docs/LIMITATIONS.md](LIMITATIONS.md) L-78.
