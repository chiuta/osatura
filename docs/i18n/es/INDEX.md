<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Índice de la documentación de OSATURA

*Empieza aquí para encontrar la página adecuada. Todos los documentos son Markdown en inglés bajo CC-BY-4.0, salvo los archivos generados de terceros (véanse sus cabeceras).*

OSATURA es un motor de rigging y animación vectorial 2D en Rust. Es un proyecto alfa (las primeras compilaciones se preparan como la versión preliminar en borrador `v0.1.0-alpha.3`; los crates siguen en 0.1.0); lee [LIMITATIONS.md](LIMITATIONS.md) antes de depender de una función.

Contenido: [Todos los documentos](#todos-los-documentos) - [Rutas de lectura por destinatario](#rutas-de-lectura-por-destinatario) - [Documentos en la raíz del repositorio](#documentos-en-la-raíz-del-repositorio) - [Dónde se responde cada pregunta](#dónde-se-responde-cada-pregunta) - [Convenciones](#convenciones-usadas-en-los-documentos)

## Todos los documentos

| Documento | Qué es | Destinatarios |
|---|---|---|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Instalación, primer proyecto, primer renderizado y exportación, primera sesión en el estudio | todos, primera lectura |
| [CLI_REFERENCE.md](CLI_REFERENCE.md) | Todos los comandos, opciones, códigos de salida y ejemplos de `osatura` | usuarios, autores de scripts, autores de CI |
| [STUDIO.md](STUDIO.md) | El editor en el navegador: paneles, atajos, colaboración, vistas previas de scrub | animadores, riggers |
| [RIGGING_GUIDE.md](RIGGING_GUIDE.md) | Construcción de un rig: huesos, IK, skinning, deformadores, drivers, máquinas de estados, maquetación | riggers, artistas técnicos |
| [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) | Claves, suavizado, clips y takes, física, lip-sync, audio, elección de exportación | animadores |
| [TUTORIAL_SVG.md](../../TUTORIAL_SVG.md) | De un dibujo vectorial a un GIF caminando: importar, preparar el rig por script, exportar (los comandos y las salidas son reales) | artistas |
| [COMPARISON.md](../../COMPARISON.md) | Una comparación honesta y fechada con otras herramientas de animación 2D | evaluadores |
| [VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md) | Comprobación de los archivos de las versiones: firmas Sigstore, procedencia de la compilación, sumas SHA-256 | todos los que instalan |
| [UNINSTALL.md](../../UNINSTALL.md) | Cómo quitar OSATURA y todos los archivos que puede crear | todos los que instalan |
| [../CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md) · [../PRIVACY.md](../../../PRIVACY.md) | Política de firma de código (SignPath Foundation), roles del equipo, privacidad | todos los que instalan, mantenedores |
| [GOOD_FIRST_TASKS.md](../../GOOD_FIRST_TASKS.md) | Tareas pequeñas para una primera contribución | colaboradores nuevos |
| [PROJECT_FORMAT.md](PROJECT_FORMAT.md) | El formato de proyecto JSON campo por campo | creadores de herramientas, riggers que editan JSON |
| [EXPORT_FORMATS.md](EXPORT_FORMATS.md) | Qué conserva y qué descarta cada formato de exportación e importación | creadores de pipelines, integradores |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crates, módulos, flujo de datos, diseño de la evaluación y del renderizado | colaboradores, revisores |
| [GLOSSARY.md](GLOSSARY.md) | Términos usados en el motor, la documentación y la CLI | todos |
| [FAQ.md](FAQ.md) | Respuestas breves a preguntas frecuentes | todos |
| [API.md](API.md) | La fachada estable de Rust (versión 1.4.0): llamadas, errores, límites, ejemplos | integradores de Rust |
| [C_ABI.md](C_ABI.md) | La interfaz de C (`osatura-capi`): funciones, reglas de memoria, ejemplos en C y Python | integradores de C, C++, Python, Go y C# |
| [WASM.md](WASM.md) | Módulo WebAssembly y envoltorio de JavaScript. **Estado: compilado y probado con Node en CI; ejecutado en Chromium sin interfaz, todavía no en Firefox ni Safari**; incluye la lista de comprobación para la primera ejecución en un navegador | desarrolladores web |
| [CI.md](CI.md) | CI de animación (`osatura ci`): manifiesto, umbrales, fotogramas de referencia, informes, ejemplos de pipeline | artistas técnicos, ingenieros de compilación |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Compilación, pruebas, convenciones, lista de comprobación de módulos nuevos, lista de comprobación de publicación, depuración | colaboradores, mantenedores |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | Pasos del propietario antes y después de publicar la versión en borrador | propietario |
| [SECURITY.md](SECURITY.md) | Modelo de amenazas, protecciones integradas, carencias conocidas, notificación, lista de comprobación de refuerzo | revisores de seguridad, responsables de despliegue |
| [LIMITATIONS.md](LIMITATIONS.md) | Lista numerada de limitaciones conocidas con gravedad, alternativa e id de la hoja de ruta | todos los que evalúan OSATURA |
| [PATENT_NOTES.md](PATENT_NOTES.md) | Nota sobre el estado de las patentes (no es asesoramiento jurídico) | licenciatarios, abogados |
| [THIRD_PARTY.md](THIRD_PARTY.md) | Inventario generado de crates y licencias de terceros | licenciatarios, empaquetadores |
| [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) | Textos de licencia generados de las dependencias, para distribuir con los binarios | empaquetadores |

## Rutas de lectura por destinatario

Cada ruta es un orden, no un requisito. Detente cuando tengas lo que necesitas.

**Usuario nuevo (quiere ver que funciona).** [../README.md](../../../README.es.md) (inicio rápido) -> [GETTING_STARTED.md](GETTING_STARTED.md) -> [STUDIO.md](STUDIO.md) -> [FAQ.md](FAQ.md) -> [LIMITATIONS.md](LIMITATIONS.md).

**Rigger.** GETTING_STARTED -> [RIGGING_GUIDE.md](RIGGING_GUIDE.md) -> [PROJECT_FORMAT.md](PROJECT_FORMAT.md) (para los campos que editas a mano) -> [GLOSSARY.md](GLOSSARY.md) -> [CLI_REFERENCE.md](CLI_REFERENCE.md) (los comandos `lint`, `check`, `fuzz`, `autorig`, `machine`) -> [CI.md](CI.md) para proteger el rig.

**Animador.** GETTING_STARTED -> STUDIO -> [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) -> [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (elige el formato de salida) -> CLI_REFERENCE (`export`, `export-fit`).

**Desarrollador de pipelines o herramientas (scripts, otros formatos).** CLI_REFERENCE -> PROJECT_FORMAT -> EXPORT_FORMATS -> [CI.md](CI.md) -> [SECURITY.md](SECURITY.md) secciones 4 a 7 (reglas de recursos y archivos).

**Integrador (que incrusta el motor).** Visión general de [ARCHITECTURE.md](ARCHITECTURE.md) -> elige la capa: Rust ([API.md](API.md)), C y otros lenguajes ([C_ABI.md](C_ABI.md)), navegador ([WASM.md](WASM.md), lee primero el cuadro de estado) -> [LIMITATIONS.md](LIMITATIONS.md) secciones 1 y 8 -> [SECURITY.md](SECURITY.md) sección 8.

**Colaborador.** [../CONTRIBUTING.md](CONTRIBUTING.md) (DCO) -> [DEVELOPMENT.md](DEVELOPMENT.md) -> ARCHITECTURE -> [../CLEANROOM.md](CLEANROOM.md) -> [../ROADMAP.md](ROADMAP.md) sección 9 (qué existe y qué no) -> LIMITATIONS (elige algo que corregir).

**Revisor de seguridad o responsable de despliegue.** [SECURITY.md](SECURITY.md) -> [DEVELOPMENT.md](DEVELOPMENT.md) sección 3 (qué se prueba) -> LIMITATIONS secciones 2 y 11 -> `osatura-cli/src/http.rs`, `studio.rs`, `collab_srv.rs`, `osatura-core/src/assets.rs`, `fsutil.rs` -> límites de [CI.md](CI.md).

**Licenciatario, empaquetador o abogado.** [../LICENSING.md](LICENSING.md) -> [../COPYRIGHT](../../../../../COPYRIGHT) y [../LICENSES/](../../../LICENSES/) -> [THIRD_PARTY.md](THIRD_PARTY.md) -> [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) -> [PATENT_NOTES.md](PATENT_NOTES.md) -> [../TRADEMARK.md](TRADEMARK.md) -> [../CLEANROOM.md](CLEANROOM.md) -> lista de comprobación de publicación en [DEVELOPMENT.md](DEVELOPMENT.md) sección 7.

**Mantenedor que prepara una publicación.** DEVELOPMENT sección 7 -> [../CHANGELOG.md](CHANGELOG.md) -> LIMITATIONS (actualizar) -> cuadro de estado del README.

## Documentos en la raíz del repositorio

| Archivo | Propósito |
|---|---|
| [../README.md](../../../README.es.md), [../README.ro.md](../../../README.es.md) | Visión general, estado, inicio rápido, resumen de licencias (en inglés y en rumano) |
| [../CHANGELOG.md](CHANGELOG.md) | Historial reconstruido; formato Keep-a-Changelog |
| [../ROADMAP.md](ROADMAP.md) | Plan ordenado; **la sección 9 es el registro fechado de lo que hace y no hace cada elemento** y la autoridad cuando los documentos discrepan |
| [../FEATURES.md](FEATURES.md) | Matriz de funciones frente a otras herramientas (partes desactualizadas, véase LIMITATIONS) |
| [../CLEANROOM.md](CLEANROOM.md) | Registro de procedencia y reglas de la implementación independiente |
| [../CONTRIBUTING.md](CONTRIBUTING.md) | Normas de contribución, licencia de entrada = licencia de salida, DCO 1.1 |
| [../LICENSING.md](LICENSING.md), [../COPYRIGHT](../../../../../COPYRIGHT), [../LICENSES/](../../../LICENSES/) | Decisión de licencia (Variante A), textos |
| [../TRADEMARK.md](TRADEMARK.md) | Política del nombre y del logotipo |
| `../REUSE.toml`, `../deny.toml` | Metadatos de licencia de los archivos que no son código fuente; política de dependencias |

## Dónde se responde cada pregunta

| Pregunta | Página |
|---|---|
| ¿Cómo lo instalo y hago mi primera animación? | GETTING_STARTED |
| ¿Qué hace el comando X? | CLI_REFERENCE |
| ¿Qué significa este campo del JSON? | PROJECT_FORMAT, GLOSSARY |
| ¿Qué formato de exportación conserva máscaras / degradados / audio? | EXPORT_FORMATS |
| ¿Puedo usarlo desde Python / C / el navegador? | C_ABI / WASM (¡estado!) |
| ¿Es seguro exponer el estudio a una red? | SECURITY sección 11 |
| ¿Por qué no funciona X? ¿Es un problema conocido? | LIMITATIONS, FAQ |
| ¿Cómo se compila y se prueba? | DEVELOPMENT |
| ¿Puedo usarlo comercialmente? ¿Cuáles son las licencias? | README (licencias), LICENSING, THIRD_PARTY |
| ¿Qué ha cambiado? | CHANGELOG, ROADMAP sección 9 |

## Convenciones usadas en los documentos

* **"Verificado"** significa que un comando o programa se ejecutó mientras se escribían los documentos, en un entorno con Linux x86-64, sin GPU, sin webcam y solo con Chromium sin interfaz. **"Sin probar"** significa exactamente eso. Las afirmaciones sobre WebAssembly en un motor real siempre están sin verificar.
* Los ejemplos de código de API.md y C_ABI.md son los programas que se compilaron y ejecutaron; los de WASM.md solo se ejecutaron contra el simulacro de JavaScript.
* La sección 9 del ROADMAP es la autoridad sobre lo que hace y no hace una función. Si un documento discrepa de ella o del código, manda el código; por favor, informa de la diferencia.
* Las rutas son relativas a la raíz del repositorio salvo que empiecen por `docs/`.
* Los números (recuentos de pruebas, límites) proceden del código fuente o de una ejecución del 22 de octubre de 2026 y pueden quedar desactualizados tras el próximo cambio; el código fuente es la referencia.
