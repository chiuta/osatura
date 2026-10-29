<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Lista de comprobación de publicación (lado del titular)

*Pasos que solo puede dar el titular, en orden. Estado a 24 oct 2026: el repositorio <https://github.com/chiuta/osatura> existe, GitHub Actions está en verde (pruebas en Linux, macOS y Windows, clippy, wasm32, cargo-deny, REUSE) y se ha compilado la versión preliminar en borrador `v0.1.0-alpha.3` (archivos de Windows, macOS y Linux, el módulo web y `SHA256SUMS.txt`); **todavía no se ha publicado nada**. Los pasos de abajo que la CI ya hizo están marcados; el resto necesita una persona, un abogado o una cuenta. Esto no es asesoramiento jurídico. Los elementos marcados con **[blocker]** deben hacerse antes de un anuncio público.*

## 1. Antes de publicar nada

- [ ] **[blocker]** Confirma o sustituye el contacto de seguridad (`alexio@trom.tf` es un marcador de posición en `docs/SECURITY.md`, en `--version` y en los documentos de licencias).
- [ ] **[blocker]** Busca el nombre «OSATURA» en los registros de marcas que te interesen (EUIPO, USPTO, OSIM) y en tiendas de aplicaciones / registros de paquetes. Véase `TRADEMARK.md`. Si hay conflicto, cambia el nombre antes de la primera versión (el nombre aparece en el binario, los crates y la documentación).
- [ ] **[blocker]** Pide a un abogado que revise `LICENSING.md`, la división MPL-2.0 / AGPL-3.0, `docs/THIRD_PARTY.md` y `docs/PATENT_NOTES.md`. Decide si quieres doble licencia; si es así, hace falta un CLA *antes* de aceptar contribuciones externas (hoy: solo DCO).
- [ ] Comprueba las condiciones de la referencia de ajuste de curvas de Graphics Gems (citada de memoria en `CLEANROOM.md`); conserva la redacción de implementación independiente solo si es exacta.
- [x] `reuse lint` y `cargo deny check` pasan en CI (el trabajo `licences`); ejecútalos también una vez en tu máquina.
- [ ] Comprueba que en el árbol no haya datos personales, rutas locales ni tokens (`grep -r "/home/" .`, `grep -ri token`), y que el modelo incluido y los recursos de la demo sean todos autogenerados.

## 2. Repositorio

- [x] El repositorio de GitHub existe y el árbol se subió sin `target/` ni `out/` (`.gitignore`).
- [ ] Conserva `COPYRIGHT`, `LICENSES/`, `REUSE.toml`, `.github/FUNDING.yml` (Buy Me a Coffee `echo.of.the.strings`, Patreon `alexio_tf`).
- [ ] Activa la notificación privada de vulnerabilidades (pestaña Security) y haz que `docs/SECURITY.md` apunte a ella.
- [ ] Pasa la URL del repositorio con `--source-url` al ejecutar el estudio (oferta de código fuente de la AGPL, ruta `/source`) y prueba la oferta desde una instalación limpia.

## 3. Primera ejecución de CI (hecha el 2026-10-24)

- [x] `.github/workflows/ci.yml` está en verde: pruebas en Linux, Windows, macOS y `clippy -D warnings`; el trabajo de compilación wasm32; el trabajo de cargo-deny y REUSE.
- [x] Los fallos de la primera ejecución en Windows se corrigieron (tamaño de pila, pruebas exclusivas de POSIX, acceso al registro de colaboración) y se anotaron en `docs/LIMITATIONS.md` (L-04).
- [x] El trabajo wasm32 pasa (compilación y prueba con Node del módulo real). **Pendiente:** ejecutar la lista de verificación de `docs/WASM.md` en un navegador real y actualizar L-01 / L-02.

## 4. Pruebas con personas

- [ ] Entrega el estudio a entre 3 y 5 animadores durante una semana con una tarea corta (hacer el rig de un personaje, crear un ciclo de caminar, exportar un GIF). Anota dónde se atascan; registra cada caso como una incidencia.
- [ ] Prueba en navegadores reales (Chrome, Firefox, Safari), en un dispositivo táctil y en una sesión solo con teclado.
- [ ] En un **Windows 11** real y en un **Mac** real: descomprime el archivo, haz doble clic en `osatura.exe` (Windows) y en `osatura-studio`, haz el rig y exporta un proyecto, e informa de lo que falle (L-74 a L-76).
- [ ] Prueba las funciones de detector de rostros / mocap / voz con datos reales; informa con honestidad de que son limitadas (L-12, L-13).

## 5. Seguridad del despliegue (si alojas el estudio o la colaboración)

- [ ] Ponlo detrás de un proxy inverso que termine TLS (receta: `docs/SECURITY.md` sección 12); nunca expongas a internet el puerto HTTP sin cifrar.
- [ ] Usa tokens por participante (`collab-token`), establece caducidad y enlaza a localhost detrás del proxy.
- [ ] Encarga una revisión de seguridad independiente antes de ofrecer colaboración alojada a desconocidos.

## 6. Artefactos de la versión

- [x] Los binarios de la versión se compilan en runners alojados en GitHub (`release.yml`) y `SHA256SUMS.txt` se genera en el borrador. **Pendiente:** revisar el borrador `v0.1.0-alpha.3`, marcar "pre-release", publicarlo y etiquetar el commit (la etiqueta se crea con la publicación).
- [x] Firma de procedencia: firmas Sigstore sin clave y atestaciones de procedencia de compilación de GitHub para cada archivo de la versión (trabajo `sign` en `release.yml`, verificado en la CI).
- [x] Requisitos previos de SignPath Foundation en el repositorio: CODE_SIGNING_POLICY.md (enlazado desde el README, el sitio y cada versión), PRIVACY.md, docs/UNINSTALL.md, sin doble licencia (LICENSING.md), metadatos de archivo de Windows (`build.rs`, comprobados en release.yml), configuración de artefactos en `.signpath/`, trabajo `windows-authenticode`.
- [ ] Propietario: activar la autenticación en dos pasos en GitHub (SignPath la exige a todos los miembros del equipo).
- [ ] Propietario: publicar una versión (SignPath exige que el proyecto se haya publicado en la forma que se va a firmar) y después presentar la solicitud en https://signpath.org/apply.
- [ ] Tras la aceptación: en SignPath, crear el proyecto `osatura`, la política de firma `release-signing` y la configuración de artefactos `windows-zip` (pegar `.signpath/artifact-configuration.xml`), y vincular el repositorio de GitHub como sistema de compilación de confianza; en GitHub, añadir la variable de repositorio `SIGNPATH_ORGANIZATION_ID` y el secreto `SIGNPATH_API_TOKEN`. La siguiente versión se firmará entonces tras una aprobación manual.
- [ ] macOS: Apple Developer ID y notarización (aún no previstos); hasta entonces Gatekeeper avisa (L-74).
- [ ] Adjunta el código fuente correspondiente de las partes AGPL (basta con la propia etiqueta; mantenla disponible mientras distribuyas binarios).
- [x] `docs/THIRD_PARTY_NOTICES.md` está regenerado (24 oct 2026) y cada archivo lo incluye; regenéralo de nuevo con `scripts/licence-audit.sh` antes de cada versión posterior.
- [ ] Crates.io / npm / registros de paquetes: comprueba que los nombres estén libres y que los campos de licencia coincidan con `LICENSING.md`.

## 7. Anuncio y donaciones

- [ ] Primera pantalla del README: qué funciona y qué no (enlaza `docs/LIMITATIONS.md`); no afirmes «de clase Moho» sin matizarlo.
- [ ] Muestra las capturas de `docs/img/` y un GIF corto hecho con el proyecto de demostración.
- [ ] Añade los enlaces de donación (Buy Me a Coffee, Patreon) al README y al cuadro «Acerca de» del estudio; no prometas funciones a cambio de donaciones salvo que puedas cumplir la promesa.
- [ ] Publica donde están los animadores (comunidades de rigging 2D / animación indie); pide informes de errores, no elogios.
- [ ] Reserva tiempo para responder rápido a las primeras incidencias; las dos primeras semanas deciden si los colaboradores se quedan.
