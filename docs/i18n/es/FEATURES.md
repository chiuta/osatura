# OSATURA — matriz de funcionalidades y listas de comprobación de referencia y competidores

> Redacción de las versiones: todos los crates son `0.1.0` (las primeras compilaciones son la versión preliminar en borrador `v0.1.0-alpha.3`); "v0.5 / v0.6 / v0.7" en este repositorio son *niveles de funcionalidad* (véase [CHANGELOG.md](CHANGELOG.md)), y la sección 9 de `ROADMAP.md` es la autoridad sobre lo que está construido.

Estado honesto de lo que existe. **✅ hecho y probado · 🟡 parcial · ❌ no implementado.**

> Nota de alcance: OSATURA es un motor de implementación independiente (a partir de especificaciones y artículos públicos; se consultaron las listas de funcionalidades de la competencia para definir el alcance; no se copió código fuente de terceros; véase CLEANROOM.md para la divulgación de la investigación), no una reimplementación de ningún
> producto comercial, y **no** alcanza la paridad con una suite comercial de 25 años
> (GUI madura, herramientas de dibujo, entrada en vivo, 3D, ecosistema). Las primeras tablas están organizadas por
> capacidad; la **segunda mitad de este archivo** es una lista de comprobación versión por versión de las funcionalidades
> documentadas públicamente del producto de referencia (fuentes al final) con
> el estado de OSATURA para cada una. Las "innovaciones" se afirman solo en relación con la técnica genérica,
> no como probadamente ausentes de cualquier otra herramienta.

---

# ★ ESCALERA DE CONSTRUCCIÓN — de lo simple a lo complejo, con estado de copyright / PI (nivel de funcionalidad v0.7, 20-22 oct 2026)

Esta sección es el **único lugar donde ver qué se puede construir, en qué orden y si es seguro**.
Sustituye a las listas dispersas de "candidatos n.º" que aparecen más abajo; los detalles y los ID están en `ROADMAP.md`
(R-xxx / I-xx), el método de sala limpia en `CLEANROOM.md`, las licencias en `LICENSING.md`.

**Estado:** ✅ construido y probado · 🟡 parcialmente construido · ⬜ planificado (sin empezar)
**Estado de PI (tras las alternativas de ROADMAP §8):**
🟢 seguro — estándar abierto, matemáticas de manual o diseño propio ·
🟠 seguro *si* se mantiene una condición (indicada en la última columna) ·
🔴 no construir.
*Filtro de ingeniería, no asesoramiento jurídico; un abogado debe confirmar cada 🟠 antes de una versión comercial.*

## Nivel 1 — Fundamentos (de horas a días cada uno; sin ninguna exposición de PI)
| Funcionalidad | Estado | PI | Base segura / condición |
|---|---|---|---|
| JSON de proyecto, validación, `evaluate(frame)` pura | ✅ | 🟢 | esquema propio |
| Trazados Bézier, rellenos, trazos, degradados, guiones, trazos de ancho variable | ✅ | 🟢 | geometría de manual |
| Huesos FK, límites, IK de 2 huesos/FABRIK, restricciones | ✅ | 🟢 | cinemática de manual |
| Claves, easing, expresiones, ruido | ✅ | 🟢 | diseño propio |
| Editor de operaciones de parche con deshacer/rehacer exactos | ✅ | 🟢 | punteros JSON al estilo RFC 6902 |
| Exportación a SVG, PNG, GIF, reproductor HTML, Lottie | ✅ | 🟢 | especificaciones públicas |
| Registro de procedencia, `stats` / `diff` semántico, regresión de fotogramas de referencia (hash exacto + tolerancia), límites de recursos, prueba de fuzzing por mutación, control de versión del formato de archivo (R-001…R-005) | ✅ | 🟢 | diseño propio |
| Vocabulario estándar de roles propio + etiquetador basado en nombres (`tag`) (R-006) | ✅ | 🟢 | nombres propios, palabras de anatomía sencillas |
| Exportación de JSON Schema (esquema R-003) | ⬜ | 🟢 | esquema propio |

## Nivel 2 — Fundamentos de autoría y salidas abiertas (días)
| Funcionalidad | Estado | PI | Base segura / condición |
|---|---|---|---|
| APNG, hoja de sprites PNG+JSON | ✅ | 🟢 | especificaciones PNG/APNG |
| WebM VP9 / AV1 con alfa (mediante tu ffmpeg) | ✅ | 🟢 | códecs abiertamente especificados; estado de patentes no evaluado |
| Exportación OCA (Open Cel Animation) | ✅ | 🟢 | especificación pública; importación ⬜ |
| Importación de OpenRaster (.ora) | ✅ | 🟢 | especificación abierta, límites contra zip-bombs |
| MP4 / H.264 | ✅ | 🟠 | solo mediante el ffmpeg *del propio usuario*; nunca incluir un codificador |
| Desplazar/escalar/invertir/copiar/pegar claves, easing por lotes, desfase temporal por canal, marcadores y regiones, timecode SMPTE (R-010, R-011, R-044) | ✅ | 🟢 | edición genérica |
| Instantáneas de revisión con poda y recuperación tras fallo (R-014); composiciones de capas (R-015); eventos de la línea de tiempo con cargas útiles (R-045); reasignación de padre conservando la pose de reposo (parte de R-013) | ✅ | 🟢 | edición genérica |
| Alinear/distribuir, capa de referencia, etiquetas shy/solo/bloqueo (R-012, R-013, R-016) | ✅ | 🟢 | edición genérica |
| Onion skin, trayectorias de movimiento (las superposiciones nunca se exportan) (R-020) | ✅ | 🟢 | diseño propio |
| Cadena de postefectos: desenfoque, sombra paralela, resplandor, contorno, posterización, pixelado, niveles/gamma, fundido (parte de R-022) | ✅ | 🟢 | procesamiento de imagen de manual |
| Recorte de trazados, filtros por capa, máscaras, profundidad de campo (R-021, R-024, R-025, R-034) | ✅ (trazados recortados horneados; límites de los efectos de capa: véase ROADMAP §9) | 🟢 | matemáticas de imagen estándar |
| Importación de SVG (trazados, formas, grupos, transformaciones, pinturas sólidas; degradados/texto/recorte se notifican como advertencias) (R-032) | ✅ | 🟢 | gramática W3C SVG 1.1 |
| Importación de BVH (un eje de rotación → ángulos de hueso, reducción acotada de claves) (R-036) | ✅ | 🟢 | formato de texto abierto |
| Exportación del contenedor dotLottie (R-03A) | ✅ | 🟢 | especificación pública del contenedor (zip + manifiesto) |
| Exportación de SVG animado, importación de secuencias de imágenes, salida OpenEXR/16 bits, exportación glTF (R-032, R-037…R-039) | 🟡 (glTF: formas del fotograma 0 más huesos y pesos de morph, véase ROADMAP R-039) | 🟢 | formatos públicos |
| Importación de señales de boca de Papagayo (Moho-switch) y Rhubarb (R-03B) | ✅ | 🟢 | solo se lee su salida de texto plano |

## Nivel 3 — Profundidad de rig y animación (semanas)
| Funcionalidad | Estado | PI | Base segura / condición |
|---|---|---|---|
| Muelles (horneados), lattices (FFD), deformador de columna, capas de conmutación, partículas, líquido, booleanas | ✅ | 🟢 | de manual |
| Sincronización labial de audio → formas de boca, generador de ciclo de caminata, espejo, retime, clip, crossfade | ✅ | 🟢 | diseño propio |
| **Pin warp** (sustituye a la deformación tipo cage / ARAP) | ✅ | 🟢 | RBF de Wendland 1995 |
| **Pesos de skin automáticos por difusión de calor** | ✅ | 🟢 | ecuación de Laplace |
| **Grabación de interpretación** con reducción acotada de claves | ✅ | 🟢 | Douglas–Peucker |
| **Mapeo de entrada** (señales CSV/JSON → parámetros) | ✅ | 🟢 | sin tracker incluido |
| IK suave/elástica (R-051), controladores de pose: deslizador + joystick 2D (R-050), cambio de vista según el ángulo (R-056), auto-rig por etiquetas (R-058) | ✅ | 🟢 | rigging genérico; matemáticas de manual, diseño propio |
| Restricciones de trazado, IK con bloqueo de pie, claves de cambio de padre, skins, imanes (R-040…R-049, R-052…R-055, R-057) | 🟡 (construido: R-040…R-043, R-044, R-047, R-048, R-052…R-055, R-057; véase la sección 9 de ROADMAP para las partes marcadas como NO hechas) | 🟢 | rigging genérico |
| Generadores de reposo (parpadeo, respiración, balanceo) y cadenas de follow-through (R-047, R-048) | ✅ | 🟢 | con seed, deterministas |
| Viento, gravedad, colisiones, tiras de pelo/tela (R-070…R-072) | 🟡 (R-070, R-071 hechos; las tiras de R-072 son solo huesos, la tela es una lámina plana) | 🟢 | manuales de masa-muelle / PBD |

## Nivel 4 — Herramientas inteligentes (de semanas a meses)
| Funcionalidad | Estado | PI | Base segura / condición |
|---|---|---|---|
| **Trazado de imagen → vector** (sustituye al autotrazado con ML) | ✅ | 🟢 | marching squares + RDP |
| **Máquina de estados basada en registro de eventos** + horneado a canales (sustituye a los clones de máquinas de estados de runtime) | ✅ | 🟢 | semántica propia — conservar nuestros nombres |
| **Retargeting por alcance del efector** | ✅ | 🟢 | método propio |
| **Solucionador de coincidencia de poses** (estrategia evolutiva, consciente de los límites de las articulaciones) | ✅ | 🟢 | Beyer & Schwefel |
| **Comprobación de destellos / destellos rojos WCAG 2.3.1 + informe de movimiento de cámara + gemelo de movimiento reducido** | ✅ | 🟠 | solo definiciones públicas; decir "ayuda de ingeniería", nunca "certificado" |
| **Linter de rig**, **contratos de movimiento**, **solucionador de bucles sin costuras**, **entrenador de principios** (I-02, I-04, I-05, I-14) | ✅ | 🟢 | ideas propias |
| **Ajuste de muelles dibujando el overshoot** (I-08) — ajusta frecuencia/amortiguación con el propio integrador del motor | ✅ | 🟢 | idea propia |
| **Explain-this-pixel** para huesos y capas (I-07) | ✅ | 🟢 | idea propia |
| Antes | después | imagen de revisión de diferencias (primer paso de I-06; la métrica perceptual es `ci diff`, SSIM y CIEDE2000) | ✅ | 🟢 | idea propia |
| Iluminación con mapas de normales, parallax, profundidad de campo (R-064, R-065, R-024) | ✅ | 🟢 | de manual |
| Sincronización labial alineada con la transcripción (R-081) | ✅ (alineador propio, `speech.rs`) | 🟠 | usar un alineador con licencia permisiva; comprobar la licencia de su modelo |
| Seguimiento facial/corporal por webcam que alimenta el mapeador de entrada (R-082) | 🟡 (detector propio entrenado solo con rostros sintéticos; webcams reales y fotografías sin probar) | 🟠 | el usuario aporta un tracker permisivo; no incluir pesos de modelo salvo que la licencia lo permita |
| Retargeting completo de mocap BVH con coincidencia automática por jerarquía (R-083) | ✅ | 🟢 | se apoya en la importación de BVH |
| Importación de PSD (R-035) | ⬜ | 🟠 | solo lectura, especificación publicada, sin marcas de Adobe; o usar simplemente ORA |

## Nivel 5 — Plataforma (meses)
| Funcionalidad | Estado | PI | Base segura / condición |
|---|---|---|---|
| Estudio en el navegador con deshacer/rehacer | 🟡 | 🟢 | UI propia; sin copiar diseños ni nombres de otros fabricantes |
| Runtime WASM, API de Rust estable + C ABI, scripting en sandbox (R-090…R-093) | 🟡 (R-091, R-092, R-093 hechos; R-090 el módulo wasm se compila y supera la prueba del envoltorio con Node en CI, y solo se ejecutó en Chromium sin interfaz) | 🟢 | API propia |
| **Servidor de agente / MCP** sobre operaciones de parche: inspeccionar, editar (atómico, deshacible), lint, seguridad, renderizar, guardar solo en su propio archivo (R-094) | ✅ | 🟢 | protocolo JSON-RPC abierto |
| GUI completa: gizmos, línea de tiempo, editor de curvas (R-100…R-103) | 🟡 (R-101, R-102 hechos; las páginas de R-100 y R-103 existen pero el autor nunca las ejecutó en un navegador real) | 🟢 | widgets genéricos; aspecto propio |
| Control de versiones de animación con merge semántico, diff perceptual en CI, explain-this-pixel (I-01, I-06, I-07) | ✅ (I-01 `osatura vcs`, I-06 `osatura ci`, I-07 `explain`) | 🟢 | ideas propias |
| Fuzzing de poses con mapas de calor de tensión, exportación orientada a un tamaño, nivel de detalle determinista con una cota de error declarada (I-03, I-10, I-11) | 🟡 (los tres construidos; la cota de I-11 no cubre efectos, trazados con skin, valores dirigidos por IK ni poses de máquina de estados) | 🟢 | ideas propias, matemáticas de manual |

## Nivel 6 — Frontera (nivel de investigación)
| Funcionalidad | Estado | PI | Base segura / condición |
|---|---|---|---|
| Renderizador GPU con pruebas de paridad con la CPU (R-110) | 🟡 (crate `osatura-gpu` construido y probado; ejecución en hardware limitada, véase ROADMAP) | 🟢 | wgpu (permisiva) |
| Edición multiusuario en tiempo real sobre el op-log (R-111) | 🟡 (hecho con un registro de sesión persistido opcional `--collab-log`; los participantes y el historial de deshacer no se persisten; HTTP simple, token compartido) | 🟢 | literatura sobre CRDT |
| Runtime de enlace de datos / maquetación (R-091, R-095) | ✅ (diseño propio; cribado de patentes sin hacer, docs/PATENT_NOTES.md) | 🟠 | cribado de patentes antes de distribuir |
| Herramientas asistidas por ML (R-112) | 🟡 | 🟠 | solo métodos clásicos, sin pesos de modelo: `autotrace` (k-means, ajuste de Bézier de Schneider, modo de línea de Zhang-Suen) y sugerencias aprendidas de tus poses (PCA, kernels, pequeña MLP opcional en un formato de pesos propio); sin aprendizaje profundo |

## No construir nunca (🔴)
- Leer o escribir archivos de proyecto nativos de **Live2D, Spine, Moho, Harmony, Rive, Cartoon Animator** (propietarios; exposición a EULA/ingeniería inversa) — usa en su lugar Lottie, OCA, ORA, APNG, SVG.
- **Solucionadores rápidos de deformación de ilustraciones al estilo ARAP** tal como se describen en las patentes de Adobe — pin warp, FFD y skinning cubren el caso de uso.
- Copiar el **diseño de la UI, los iconos, los nombres, el contenido de muestra o los números mágicos/la terminología de los formatos de archivo** de cualquier fabricante.
- Imitar la herramienta de fotosensibilidad **Harding** o afirmar cumplimiento de normas de emisión.
- Incluir codificadores **H.264/H.265/AAC**.

## Condiciones que se aplican a todas las filas 🟢
Registro de procedencia mantenido (`CLEANROOM.md`; implementación independiente, no una sala limpia estricta); solo dependencias permisivas (se regenera con `scripts/licence-audit.sh`; actualmente 363 crates de terceros en todas las plataformas, todos de clase MIT / Apache-2.0 / BSD / Zlib / Unlicense salvo seis crates MPL-2.0 (smartstring mediante la funcionalidad `scripting`, que está activada por defecto, y cinco mediante la aplicación de ventana `osatura-gui`); véase docs/THIRD_PARTY.md); sin ilustraciones ni audio de terceros incluidos sin su propio archivo de licencia; el nombre **OSATURA** autorizado en una búsqueda de marcas antes del lanzamiento.


# Tablas detalladas de capacidades
## Dibujo y formas
| Funcionalidad | Estado | Notas |
|---|---|---|
| Trazados Bézier cúbicos, abiertos/cerrados, tiradores | ✅ | |
| Relleno: sólido, degradado lineal y radial | ✅ | los degradados siguen la transformación de la capa |
| Trazo: ancho, remates, uniones, guiones | ✅ | |
| Trazo de ancho variable (perfil de ancho) | ✅ | contorno poligonal; trazados cerrados → dos anillos |
| Suavizado Catmull-Rom, ayudantes de rectángulo/círculo/elipse/polígono | ✅ | |
| Booleanas de polígonos (unión/intersección/diferencia/xor) | ✅ | capa `boolean`; los agujeros se unen en un único polígono; el resultado es poligonal (aplanado) |
| Capas de metaballs / líquido | ✅ | marching squares, los huesos pueden controlar las manchas |
| Capas de imagen (PNG), incrustadas o en archivo | ✅ | |
| Pinceles de mapa de bits/vectoriales, rellenos de textura, texto/fuentes | ❌ | |
| Dibujo a mano alzada, herramientas de edición de puntos, trazado de imagen | ❌ | sin GUI de dibujo |

## Rigging
| Funcionalidad | Estado | Notas |
|---|---|---|
| Jerarquía de huesos, FK, pose de reposo | ✅ | |
| Límites de ángulo / escala | ✅ | |
| IK: analítica de 2 huesos (flexión ±) | ✅ | |
| IK: cadenas de N huesos (FABRIK) | ✅ | |
| Mezcla FK↔IK (animable) | ✅ | |
| Restricciones: aim, copy-rotation, pin (peso animable) | ✅ | |
| Skinning de mezcla lineal de anclas + tiradores, auto-skin | ✅ | |
| Capas unidas a huesos, jerarquía de capas / grupos | ✅ | |
| Movimiento secundario: muelles amortiguados horneados | ✅ | determinista, de acceso aleatorio (horneado al cargar) |
| Canales dirigidos por pose ("driven keys") | 🟡 | una curva asigna cualquier propiedad a otra; sin UI/acciones dedicadas de smart-bone |
| Física de cuerpo rígido, tela, cuerpo blando | ❌ | |

## Deformación
| Funcionalidad | Estado | Notas |
|---|---|---|
| Desplazamientos por punto (animables) | ✅ | direccionados como `layer::path::index` |
| Morph targets / blend shapes (pesos aditivos) | ✅ | |
| Lattice de forma libre (FFD de Bernstein) | ✅ | puntos de control animables |
| Deformador de columna / curva | ✅ | radio suave opcional |
| Oscilación "boil" (temblor de dibujo a mano) | ✅ | ruido coherente, determinista |
| Deformación de malla de imagen con pesos de hueso | ✅ | malla en cuadrícula; sin editor de mallas |

## Animación
| Funcionalidad | Estado | Notas |
|---|---|---|
| Claves + easing (linear/step/ease/cubic-bezier) | ✅ | |
| Extrapolación: hold/linear/loop/ping-pong/loop-offset | ✅ | |
| Canales de expresión | ✅ | pequeño lenguaje de expresiones seguro; puede leer otros canales |
| Canales de ruido, mezcla add/multiply/replace, pesos | ✅ | |
| Capas de conmutación + exposición (fotograma a fotograma, cambios de boca) | ✅ | |
| Retime/desfase de clips, retarget, espejo L↔R, horneado de crossfade | ✅ | funciones + CLI `mirror`; sin UI de NLA |
| Ciclo de caminata procedural | ✅ | generador sinusoidal simple, no una herramienta profesional de caminata |
| Sincronización labial de audio desde WAV | 🟡 | heurística de volumen/brillo → 5 formas de boca, **no** reconocimiento de fonemas |
| Emisor de partículas determinista | ✅ | forma cerrada: cualquier fotograma se evalúa directamente |
| Onion skin, trayectorias de movimiento | ✅ | CLI `onion` y `trail`, superposiciones del estudio; nunca se exportan |
| Reproducción de audio / scrubbing de forma de onda | 🟡 | comando `waveform` y reproducción de audio del estudio; no se afirma ninguna UI de scrubbing de forma de onda |

## Cámara y composición
| Funcionalidad | Estado | Notas |
|---|---|---|
| Cámara 2D con pan/zoom/rotación (animable) | ✅ | |
| Parallax multiplano por capa de nivel superior | ✅ | |
| Opacidad, modos de mezcla, máscaras alfa (+invertir) | ✅ | |
| Capas / cámara 3D reales, estéreo | ❌ | |

## Salida e interoperabilidad
| Funcionalidad | Estado | Notas |
|---|---|---|
| Fotogramas SVG | ✅ | |
| Fotogramas PNG (rasterizado con tiny-skia) | ✅ | |
| GIF animado | ✅ | |
| MP4 | ✅ | requiere `ffmpeg` en el PATH |
| Reproductor HTML autónomo | ✅ | todos los fotogramas en línea, barra de desplazamiento |
| Lottie JSON | 🟡 | geometría horneada por fotograma; degradados → primer color; sin máscaras/modos de mezcla/imágenes; **no validado en un reproductor Lottie real** |
| Importación de archivos de otras herramientas, PSD, FBX, etc. | ❌ | (deliberadamente: sin formatos propietarios) |
| API de scripting / plugins | 🟡 | `script` Rhai en sandbox mediante solicitudes validadas, más expresiones; sin API de plugins |

## Edición
| Funcionalidad | Estado | Notas |
|---|---|---|
| Modelo de edición JSON-patch con deshacer/rehacer | ✅ | cada edición se valida; las ediciones inválidas se revierten |
| Ayudantes para establecer/borrar claves | ✅ | |
| Estudio web (navegador) | 🟡 | vista previa, barra de desplazamiento, claves de ángulo por hueso, editor JSON, deshacer/rehacer, guardar, exportaciones. **Sin dibujo, sin edición de puntos/huesos en el lienzo, sin editor de curvas.** |

## Cosas que son "mejoras" modestas sobre la base de los manuales
(no se afirma que estén ausentes en otros lugares)
- Cada edición del proyecto es un parche JSON validado e invertible → deshacer nunca puede corromper un proyecto.
- Toda la simulación (muelles, partículas) es determinista de acceso aleatorio: desplaza o renderiza cualquier fotograma en cualquier orden y obtendrás píxeles idénticos.
- Cualquier propiedad — incluidos los puntos de control de los deformadores y los pesos de morph — es direccionable mediante una única ruta `layer::sub::id`, de modo que las expresiones y las curvas dirigidas pueden conectar cualquier cosa con cualquier cosa.
- La validación informa de *todos* los problemas en una sola pasada con los nombres afectados.
- Un solo motor → SVG, raster, GIF, MP4, HTML, Lottie; el mismo `evaluate(frame)` alimenta a todos.

## Pruebas
`cargo test` — 142 pruebas unitarias/de integración (matemáticas, pistas, expresiones, solucionadores de rig, deformadores,
píxeles renderizados, exportadores, deshacer/rehacer del editor, audio, partículas, booleanas, metaballs).

---

# Lista de comprobación frente al producto de referencia, versión por versión

Referencia: Moho (antes Anime Studio), 1999 → 14.5 "SHOWTIME" (12 oct 2026). Las listas de funcionalidades proceden
de páginas de lanzamiento del fabricante y de la prensa especializada (véase Fuentes). Lagunas en el registro: de Anime Studio 6–8,
12.5 y 14.1 no encontré ninguna lista pública detallada, por lo que solo están cubiertas por la
historia general. El estado es el de OSATURA: ✅ existe un equivalente · 🟡 parcial / enfoque distinto · ❌ ausente.

## Primeras versiones (1999 – Anime Studio 8, solo historia general)
| Funcionalidad | OSATURA |
|---|---|
| Partículas | ✅ capa de emisor determinista |
| Máscaras de capa | ✅ máscaras alfa + invertir |
| Onion skins | ✅ (CLI `onion`, superposición del estudio) |
| Restricciones de hueso | ✅ aim / copy-rotation / pin, límites de ángulo y escala |
| Scripting en Lua | 🟡 (Rhai, no Lua; también canales de expresión) |
| Sincronización labial | 🟡 heurística de amplitud desde WAV |
| Física | 🟡 muelles + gravedad de partículas; sin cuerpos rígidos |
| Creación / importación 3D | ❌ |
| Importación de Photoshop | ❌ |
| Compatibilidad con Wacom | ❌ (sin GUI de dibujo) |
| Smart bones, IK | 🟡 canales dirigidos / ✅ 2 huesos + FABRIK |

## Anime Studio 9 (2013)
| Funcionalidad | OSATURA |
|---|---|
| 64 bits, aceleración por GPU | 🟡 Rust nativo de 64 bits; solo rasterizador por CPU (sin GPU) |
| Smart Bones (diales de control de flexión de articulaciones) | 🟡 los canales dirigidos asignan cualquier valor a cualquier otro; sin UI/acciones de diales |
| Gráficos de movimiento editables | ❌ (las curvas son datos con easing cubic-bezier; sin editor de curvas) |
| Línea de tiempo renovada | ❌ (el estudio solo tiene una barra de desplazamiento y marcas de claves) |
| Multitáctil Wacom | ❌ |

## Anime Studio 10 (2014)
| Funcionalidad | OSATURA |
|---|---|
| Restricciones de ángulo independientes (globales) | 🟡 copy-rotation hacia un hueso de control lo aproxima |
| Squash & stretch | 🟡 la escala del hueso es animable; sin squash/stretch automático |
| Diales de Smart Bone / configuración | 🟡 véase arriba |
| Interpolación bounce / elastic / stagger | 🟡 bounce, spring (similar a elastic), back, cubic-bezier ✅; stagger ❌ |
| Cubo de pintura, pincel de manchas, borrador, reducir puntos | ❌ |
| Personajes precargados / contenido de biblioteca | ❌ |
| Importación de objetos 3D | ❌ |

## Anime Studio 11 (2015)
| Funcionalidad | OSATURA |
|---|---|
| Animación fotograma a fotograma | ✅ capas de conmutación + canales de exposición |
| Referenciado de capas (duplicados enlazados) | ❌ |
| Ordenación animada de formas | ❌ (el orden de dibujo es fijo por capa; las capas de conmutación pueden intercambiar) |
| Curvas de movimiento editables | 🟡 datos de curva sí, sin UI de curvas |
| Importación de PSD (incl. actualización en vivo) | ❌ |
| Exportación a FBX / Unity | ❌ |
| Importación/grabación de audio + sincronización labial automática | 🟡 importación de WAV + sincronización labial heurística; sin grabación |
| Fusionar trazos, herramienta Color Points | ❌ |

## Moho 12 (2016) y 12.x
| Funcionalidad | OSATURA |
|---|---|
| Tiradores Bézier | ✅ |
| Herramientas de mano alzada mejoradas | ❌ |
| Smart Warp (mallas personalizadas) | 🟡 malla de imagen en cuadrícula con pesos de hueso; deformadores lattice / de columna para vectores; sin editor de mallas |
| Motion blur | ❌ |
| Huesos pin | 🟡 restricción pin, no huesos pin de un solo punto |
| Importar/exportar acciones entre rigs similares | 🟡 funciones `clip`, `retarget`, `mirror` sobre conjuntos de canales; sin UI de biblioteca de acciones |
| Animar varias capas a la vez | ✅ cualquier número de canales por línea de tiempo |
| UI rediseñada, anclaje de huesos, autocongelación de posición | ❌ |

## Moho 13 (2019)
| Funcionalidad | OSATURA |
|---|---|
| Fotograma a fotograma en mapa de bits + onion skin | 🟡 capas de imagen PNG dentro de capas de conmutación; sin onion skin, sin dibujo en mapa de bits |
| Herramientas de dibujo en mapa de bits / pinceles personalizados | ❌ |
| Herramientas de storyboard y animática | ❌ |
| Ventana de acciones | ❌ (solo clips a nivel de datos) |
| Luces 3D, materiales, sombreado toon | ❌ |
| Más preajustes de salida | 🟡 formatos de exportación por CLI, sin gestor de preajustes |
| Personajes incluidos | ❌ |

## Moho 13.5 (2021) y 13.5.x
| Funcionalidad | OSATURA |
|---|---|
| Vitruvian Bones (intercambiar dibujos *y* conjuntos de huesos) | 🟡 las capas de conmutación intercambian dibujos; sin intercambio de conjuntos de huesos |
| Mallas de cuadriláteros (perspectiva) | 🟡 la malla en cuadrícula + lattice dan deformaciones de forma libre; sin cuadrilátero proyectivo |
| Dinámica de viento (intensidad, dirección, turbulencia) | ❌ (los muelles no tienen viento; los canales de ruido pueden simularlo) |
| Claves relativas | ❌ |
| Variantes de visemas de sincronización labial elegidas al azar | ❌ (el mapeo puede fusionar formas, sin variantes aleatorias) |
| Exportar audio como WAV | ❌ |
| Copiar/pegar claves profundas, ajuste de pivote, marcadores de la línea de tiempo | ❌ (sin GUI) |
| Licencias flotantes | n/a |

## Moho 14 (2023)
| Funcionalidad | OSATURA |
|---|---|
| Nuevo motor gráfico, efectos en tiempo real (desenfoque, sombra, ruido) | ❌ sin filtros de desenfoque/sombra/ruido; modos de mezcla ✅ |
| Liquid Shapes (fusionar/restar/intersecar/mezclar) | 🟡 líquido metaball ✅ + booleanas de polígonos ✅; no es el mismo modelo de formas |
| Curvers / Compressible Curvers | 🟡 el deformador de flexión de columna es el análogo más cercano |
| Importación de SVG mejorada | ❌ (solo **exportación** de SVG) |
| Mejoras de mano alzada, nuevos estilos, fusión de formas | ❌ / ❌ / 🟡 (booleanas) |
| Multi-Stroke Fill, Connect & Create | ❌ |
| Smart Line Boil, opciones de pincel | 🟡 oscilación "boil" con frecuencia ajustable ✅; opciones de pincel ❌ |
| Colores de punto, Add Point mejorado | ❌ |
| Dynamic Bones (+ gravedad) | 🟡 huesos de muelle amortiguado ✅; sin término de gravedad |
| Restricciones con retardo | 🟡 los muelles dan retraso respecto a un objetivo; sin restricción de retardo explícita |
| Nuevas opciones de línea de tiempo | ❌ |
| Mejoras de fotograma a fotograma | 🟡 |
| Actualización de la interfaz de scripting (Lua 5.4) | ❌ |

## Moho 14.1 – 14.2 (2023-11, 2024-06)
| Funcionalidad | OSATURA |
|---|---|
| (14.1: no se encontró ningún detalle público) | — |
| Desfase animado del perfil de curva | ❌ (los perfiles de ancho son estáticos) |
| Smart boil para ruido vectorial | 🟡 la oscilación avanza a una frecuencia fija; sin modo "solo mientras se mueve" |
| Herramienta Transform Bone rehecha | ❌ (sin herramientas de lienzo) |
| Visualización de pixelación | ❌ |
| Modo de mezcla Colour Burn | ✅ |
| Espaciado mínimo de pincel menor, capas recortadas de PSD | ❌ |
| Numeración de la línea de tiempo basada en la frecuencia de fotogramas | ❌ |
| Opción de visualización "Turn off masked layers" | ❌ |
| Mejor sincronización labial a frecuencias de fotogramas bajas | 🟡 |
| Nombres multilingües para huesos/capas/marcadores | ✅ (nombres UTF-8) |
| Antialiasing de la máscara Exclude-Strokes, onion skins en color sobre imágenes | ❌ |
| Smart Bones que controlan la dinámica de los huesos | ❌ (los ajustes de los muelles no son animables) |
| Scripts Lua al inicio; indicador de consola en Windows | ❌ / n/a |
| Exportación ProRes con alfa | ❌ (solo GIF/PNG/MP4) |

## Moho 14.3 (2024-12)
| Funcionalidad | OSATURA |
|---|---|
| Opacidad de pincel por punto (según la presión) | ❌ |
| Deriva de color según la presión del lápiz | ❌ |
| Controles de herramienta de mano alzada/ancho de línea | ❌ |

## Moho 14.4 "SIDE QUEST" (2025-11)
| Funcionalidad | OSATURA |
|---|---|
| Exportación a Unreal / Unity / Godot | ❌ |
| Exportación glTF a Blender / herramientas 3D | 🟡 (`.glb`: formas del fotograma 0, huesos, pesos de morph) |
| Smart Bones de 360° | ❌ |
| Create Style from Shape | ❌ |
| Opciones de partículas (replicar a lo largo de una curva, retardo, emisión ordenada, distribución) | 🟡 retardo de inicio ✅, emisión ordenada determinista ✅; replicar a lo largo de una curva ❌ |
| Layer Selector, caída del imán (Magnet falloff), importación de PSD, suavizado de mano alzada, añadidos de Lua | ❌ |
| Creación de ciclos más sencilla | ✅ extrapolación en bucle + generador de ciclo de caminata |

## Moho 14.5 "SHOWTIME" (2026-10-12)
| Funcionalidad | OSATURA |
|---|---|
| Live Playback (webcam / gamepad / micrófono, webcam virtual, OBS) | ❌ |
| Seguimiento facial → huesos / Smart Bones / malla | 🟡 (detector entrenado con datos sintéticos, `face-bake`; no validado con rostros reales) |
| Grabación en vivo en la línea de tiempo, control de varios personajes | ❌ |
| Seguimiento de vídeo (secuencias de imágenes) | ❌ |
| Algoritmos de mano alzada, líneas rectas, recorte de colas | ❌ |
| Cubo de pintura rehecho (Vector / Trace / Recolor) | ❌ |
| Movimiento inverso del imán, selección de puntos por forma | ❌ |
| Categorías de pinceles personalizadas | ❌ |
| Color/opacidad del onion skin, Shift & Trace, onion skins magnéticos | ❌ |
| Filtros / selección múltiple en la ventana de capas | ❌ |
| Rendimiento: deshacer/rehacer más rápido, archivos más ligeros | 🟡 deshacer es una inversa barata del parche JSON; los archivos son JSON plano |
| Exportación MP4/H.264 | ✅ mediante ffmpeg |
| Perfiles de color PNG, importación/exportación EXR | ❌ |
| Exportador por lotes, opciones de CLI `-exr` / `-pngprofile` | 🟡 CLI `frames` / `export`, sin cola de lotes |
| Frecuencia de fotogramas con 3 decimales (p. ej. 23.976) | ✅ |
| La exportación SVG gestiona trazos de ancho variable y formas líquidas | ✅ (los trazos se convierten en contornos, los líquidos en trazados) |
| Liquid hide-edge, formas líquidas solo de trazo, exposición de trazos | ❌ |
| Linux mediante Wine/Proton | ✅ Rust nativo (Linux/macOS/Windows); archivos precompilados para Windows x86-64, macOS arm64 y Linux x86-64 en la versión preliminar en borrador; probado por CI en los tres, por una persona solo en Linux |

## Resumen (solo el producto de referencia; los competidores siguen más abajo)
De las funcionalidades documentadas del producto de referencia, OSATURA cubre el núcleo de **rigging, deformación,
animación procedural, matemáticas de composición y exportación** (en líneas generales: esqueleto/IK/restricciones,
deformación de malla y lattice, claves y expresiones, máscaras/mezclas/degradados, partículas,
metaballs/booleanas, GIF/MP4/HTML/Lottie), pero **no tiene herramientas de dibujo, ni entrada en vivo, ni 3D,
ni E/S de PSD/FBX** (en el momento de la primera revisión; desde entonces se han añadido exportación glTF, salida EXR, scripting Rhai, un estudio con editor de curvas y línea de tiempo, onion skin y efectos de desenfoque, véase la sección 9 de `ROADMAP.md`).
Esos son los bloques grandes, mayormente ligados a la GUI, del producto y constituyen la brecha honesta.

## Fuentes
- Anuncio de Moho 14.5 SHOWTIME, moho.lostmarble.com (blog, 12 oct 2026)
- Página de funcionalidades de Moho Pro, mohoanimation.com/pages/features
- Appcast de Moho 14.4 SIDE QUEST, lostmarble.com/appcasts/Moho14.4Update.html
- Moho 14.3: Digital Production, 27 dic 2024
- Publicación de la versión Moho 14.2, moho.lostmarble.com; lista de tutoriales "Moho 14: What is new?"
- Novedades de Moho Pro 13.5, moho.lostmarble.com; Lester Banks (abr 2021); notas de 13.5.3
- Moho 13: Lester Banks (jun 2019)
- Moho Pro 12: 80.lv; Digital Production (nov 2016)
- Anime Studio 11: Architosh (may 2015); 10: Architosh (mar 2014); 9: Renderosity
- Fechas de versiones: mohoscripting.com/moho_versions; historia general: Wikipedia "Moho (software)"

---

# Anexo: historia de Moho / Anime Studio, versiones 1 – 11 (completado)

Fuente: el PDF del historial de versiones alojado por el fabricante (lostmarble.com, "Anime Studio history", 2015) más las
páginas de prensa de Anime Studio 7 y 8. Sustituye a las filas de "solo historia general" que figuran cerca del principio de la
lista de comprobación de referencia. Ni siquiera aquí se encontraron: detalles de 8.x, 10.1.x, 11.1, 12.5, 14.1.

| Versión | Funcionalidades documentadas | OSATURA |
|---|---|---|
| Moho 1 (1999) | BeOS, port a Windows, salida Flash | ❌ Flash/SWF (obsoleto) |
| 2.x | E/S QuickTime, renderizado hasta 4096², sombreado de capas, capas de conmutación, muestras de color en los estilos, enlace flexible de huesos, importación de Illustrator mejorada | ✅ capas de conmutación, 🟡 enlace por pesos, ❌ importación de Illustrator, ✅ escalas de raster grandes (≤16×) |
| 3.x | Partículas, envoltura de imágenes, máscaras de capa, onion skins, cámara virtual con profundidad, deformación de imágenes, capas de conmutación mejoradas, compatibilidad con tabletas, timecode SMPTE, ocultar ilustraciones fuera de pantalla, máscaras reversibles, aleatorizador de grosor de línea, zonas seguras, inclinación de capa | ✅ partículas, máscaras (+invertir), cámara, deformación de malla de imagen, inclinación de capa; ❌ onion skin, tableta, timecode, zonas seguras, aleatorizador de grosor |
| 4.x | Curvas de animación gráficas, restricciones de hueso, renderizado por lotes, audio, parámetros animables, ciclos de animación, escalado de claves, proyectos redimensionables, profundidad de partículas, multitudes aleatorizadas, enlace flexible automático, versión para Linux | ✅ restricciones, ciclos (extrapolación en bucle), parámetros animables, proyectos redimensionables, retime con `clip` ≈ escalado de claves, Linux; 🟡 multitudes (partículas), 🟡 audio (solo sincronización labial); ❌ UI de editor de curvas |
| 5.x | Nueva UI, exportación SWF, capas 3D y de notas, acciones, estilos, scripting en Lua | ❌ capas 3D / de notas / Lua; 🟡 acciones (funciones de clip); ✅ los estilos son pintura por trazado |
| Anime Studio 6 | Seguimiento de movimiento, sincronización labial integrada, pincel de dispersión, Lua 5.1, sombreado automático, **morph targets**, secuenciador, vídeo HD | ✅ morph targets; 🟡 sincronización labial; ❌ seguimiento de movimiento, pincel de dispersión, sombreado automático, secuenciador |
| 6.1 / 6.2 | Subida a Facebook, importación de GIF transparente | ❌ / 🟡 exportación GIF ✅, importación ❌ |
| Anime Studio 7 | Física simulada (gravedad, densidad, fricción, elasticidad), diseño de formas 3D, grabación de audio, biblioteca de contenido, importación de escenas de Poser, **herramienta de seguir trazado**, recopilar medios | 🟡 muelles/gravedad de partículas; ❌ física rígida, 3D, grabación, biblioteca, Poser, seguir trazado |
| Anime Studio 8 | Trazado automático de imágenes, Character Wizard, modo para principiantes, conexión de medios en tiempo real, estilos de renderizado globales, rigging de huesos avanzado | ❌ todo (sin trazado, asistente, enlace en vivo, estilos globales) |
| Anime Studio 9 / 9.5 | Gráficos de movimiento editables, Smart Bones, **composiciones de capas**, onion skins mejorados, 64 bits + GPU, keyframing mejorado, línea de tiempo en color; 9.5: mejoras de Smart Bone, desplazamiento de profundidad, controles de capas anidadas, flexi-binding, mejoras de capas de conmutación | véase la tabla de AS 9; ❌ composiciones de capas, desplazamiento de profundidad |
| Anime Studio 10 / 10.1 | Nuevas restricciones, configuración de Smart Bone, bounce/elastic/stagger, proceso de renderizado independiente, herramientas combinadas de huesos+puntos, vista previa de animación, multidocumento, editor de atajos, selección de varias formas, fotograma cero, ocultación de puntos/huesos, edición multicapa, **curvas de ancho variable**, transparencia de textura, fuente de partículas ajustable, profundidad de campo, desenfoque fraccionario; 10.1: autotrazado de imágenes, estilos globales, capas de parche, importación de PSD, medios en tiempo real, Character Wizard | ✅ trazos de ancho variable, ✅ animación multicapa; 🟡 profundidad de campo ❌ desenfoque ❌; ❌ capas de parche, el resto como arriba |
| Anime Studio 11 | Fotograma a fotograma, referenciado de capas, ordenación animada de formas, **objetivos de hueso animados**, **reasignación de padre de huesos animada**, volteo de huesos, agrupación de capas de selección, escala de capa normalizada, exportación por lotes, nuevo formato de archivo | ✅ FbF, ✅ objetivo de IK animado (el objetivo es simplemente un hueso), ❌ *reasignación de padre* animada, 🟡 volteo de huesos (`mirror`), ✅ lotes mediante CLI, ✅ formato de archivo JSON plano |

**Nuevas carencias detectadas aquí** (añadir al roadmap): composiciones de capas (estados de visibilidad de capas guardados),
reasignación de padre de huesos animada, capas de parche, desplazamiento de profundidad, capas de seguir trazado, superposiciones de zonas seguras,
timecode SMPTE, sombreado automático, pincel de dispersión, autotrazado de imágenes.

---

# Funcionalidades de la competencia no cubiertas por la lista de comprobación anterior

Productos revisados: Toon Boom Harmony 24, Spine 4.2, Live2D Cubism 5.x, Cartoon Animator 5.3, Rive,
Synfig Studio, OpenToonz, TVPaint Animation 12, Adobe Animate, Cavalry, Procreate Dreams, Duik (After
Effects), Blender Grease Pencil. Los elementos siguientes son los que **no figuran ya** en la lista de comprobación
de referencia. La calidad de las fuentes varía: la mayoría de las filas proceden de páginas de funcionalidades/lanzamientos de fabricantes que consulté;
las filas marcadas con † proceden de conocimiento general o solo de un título/índice y no se confirmaron en ninguna página.
El estado es el de OSATURA: ✅ · 🟡 · ❌.

## Rigging, restricciones, deformación
| Funcionalidad | Vista en | OSATURA |
|---|---|---|
| Herramientas de pesos: soldar pesos entre mallas, copiar/pegar pesos entre vértices | Spine 4.2 | ❌ (solo auto-skin, sin pintado) |
| Restricciones de física de huesos (inercia, viento, gravedad, mezcla) | Spine 4.2 | 🟡 muelles amortiguados (frecuencia/amortiguación); sin viento/gravedad/inercia |
| Restricción de trazado: los huesos siguen una spline Bézier compuesta | Spine, Rive (Follow Path) | ❌ |
| Mezclas de restricciones, duplicación con huesos, edición múltiple, carpetas | Spine 4.2 | 🟡 la mezcla de IK y el peso de las restricciones son animables; sin carpetas/edición múltiple |
| Skins (intercambiar conjuntos de adjuntos en un esqueleto) | Spine, solos de Rive | 🟡 capas de conmutación |
| "Herramienta de pose" de IK multihueso (arrastrar el extremo de un miembro) | Spine | ❌ (requiere GUI de lienzo) |
| Adjuntos de caja delimitadora / prueba de impacto | Spine | ❌ (no se necesitan para salida offline) |
| Deformación de forma libre, pesos de malla | Spine, Harmony | ✅ lattice + malla de imagen con pesos |
| Cadena de deformadores: Bone, Game Bone, Curve, Envelope, Free-Form, Mesh Warp | Harmony | 🟡 bone ✅ curve (columna) ✅ free-form ✅ mesh warp ✅; envelope/cage ❌; game-bone ❌ |
| Shape-Aware Deformation (puntos, huesos, cages) | Harmony 24 | ❌ |
| Horneado de todos los tipos de deformación | Harmony 24 | 🟡 muelles horneados; geometría horneada por fotograma en la exportación Lottie |
| Controlador maestro / selector de transformación para grupos de deformación | Harmony 24 | ❌ |
| Pegs y jerarquía de capas | Harmony, Animate | ✅ grupos, capas unidas a huesos, jerarquía |
| Auto-muscle, auto foot-roll, salida cinemática † | Harmony | ❌ |
| Deformadores de rotación / warp + parámetros de keyform, glue | Live2D | 🟡 parámetros ≈ canales dirigidos, warp ≈ lattice; glue ❌ |
| Blend shapes que controlan rotación, opacidad, orden de dibujo, color multiply/screen | Live2D 5.0 | 🟡 los pesos de morph controlan solo la geometría |
| Auto-malla con control de densidad uniforme para formas finas | Live2D 5.0 | ❌ (solo cuadrícula regular) |
| El deformador warp cambia de tamaño automáticamente para contener el keyform hijo | Live2D 5.3 | ❌ |
| Pincel de deformación (formas cuadrada/línea/expansión) | Live2D 5.3 | ❌ |
| Deformador facial asistido por IA / generación de movimiento facial | Live2D 5.0 | ❌ |
| Joysticks (control 2D asignado a animaciones) | Rive | 🟡 un canal dirigido asigna una entrada; sin joystick de 2 entradas |
| Restricciones entre objetos arbitrarios (traslación/rotación/escala/distancia) | Rive | 🟡 solo aim, copy-rotation, pin |
| Rigging de mapa de bits con intercalados automáticos | TVPaint 12 | 🟡 malla de imagen ✅; intercalados automáticos ❌ |
| Herramienta Plastic (rigging basado en mallas) | OpenToonz | 🟡 malla de imagen + pesos |
| Auto-rigging, controladores, dinámica de muelles | Duik † | 🟡 muelles ✅; auto-rig ❌ |

## Animación, línea de tiempo, interactividad
| Funcionalidad | Vista en | OSATURA |
|---|---|---|
| Mezcla de animaciones: blend/crossfade de animaciones simultáneas en runtime | Spine | 🟡 modos Add/Multiply + pesos; `crossfade` horneado; sin transiciones en runtime |
| Dopesheet + editor de curvas + vista previa en runtime | Spine, Rive, OpenToonz | ❌ (los datos lo permiten; sin UI) |
| Máquina de estados: entradas, listeners, transiciones con condiciones, estados de mezcla, capas | Rive | ❌ |
| Eventos, disparadores de audio, recursos dinámicos | Rive | ❌ |
| Enlace de datos (elementos de diseño ↔ datos en vivo) | Rive | ❌ (las expresiones pueden leer canales, no datos externos) |
| Scripting incrustado (Luau, JavaScript, ToonzScript) | Rive, Cavalry, OpenToonz | ❌ |
| Maquetaciones adaptables / componentes / bibliotecas | Rive | ❌ |
| Trim Paths, difuminado vectorial, N-slicing | Rive | ❌ |
| Temporización independiente de la resolución, tweening vectorial | Synfig | ✅ |
| Enlace de parámetros entre capas mediante expresiones | Synfig | ✅ |
| Grafo de nodos procedural (generadores, transformadores, efectores), duplicadores | Cavalry | ❌ (sin grafo de nodos; sin instanciado de capas) |
| Animación guiada por datos desde JSON/CSV | Cavalry | ❌ |
| Performing: grabar gestos en vivo como claves | Procreate Dreams | ❌ |
| Atajos de teclado de disparadores del escenario de marionetas, motion pilot (caminatas guiadas por el cursor) | Cartoon Animator 5.3 | ❌ |
| Conmutador de cámaras: cámaras guardadas, cortes con atajo, pista de cambio de cámara | Cartoon Animator 5.3 | 🟡 una cámara; los cortes se pueden simular con claves step |
| Movimientos cinematográficos (dolly zoom, seguimiento de un objetivo) | Cartoon Animator 5.3 | 🟡 cámara animable; sin seguimiento de objetivo |
| Mocap facial (webcam / iPhone), cuerpo guiado por la cabeza, volumen del micrófono | Cartoon Animator, Live2D | ❌ (volumen del micrófono solo mediante WAV offline) |
| Modo de retransmisión en directo (OBS/Twitch, asignación de Stream Deck) | Cartoon Animator 5.3 | ❌ |
| Sincronización labial Motion Sync que genera claves en la línea de tiempo | Live2D 5.0 | 🟡 WAV heurístico → claves de exposición |
| Integración de sincronización labial con Rhubarb | OpenToonz | ❌ (solo heurística) |
| Extracción de root motion (compensación delta) | Spine 4.2 | 🟡 extrapolación loop-offset |
| Sincronización de tiradores de curvas en bucle | Spine 4.2 | ✅ extrapolación en bucle (sin UI de tiradores) |
| Morphing con morph hints | Harmony | 🟡 morph targets + lerp de trazado; sin hints |
| Cámara 2D basada en claves con temporización en la línea de tiempo | TVPaint 12 | ✅ |

## Dibujo, pintura, herramientas de mapa de bits
| Funcionalidad | Vista en | OSATURA |
|---|---|---|
| Pincel: aleatorización (número de sellos, desplazamiento), papeles duales, pinceles multitextura | Harmony 24, TVPaint 12 | ❌ |
| Opacidad del lápiz según presión/inclinación/velocidad/afilado/desvanecimiento | Harmony 24 | ❌ |
| Guías de simetría (vertical, horizontal, dual) | Harmony 24 | ❌ |
| Herramienta de corte (lazo, recorte), editor de contornos, cerrar huecos | Harmony | ❌ |
| Guías de dibujo, guías de perspectiva/regla † | Harmony | ❌ |
| Animación sin papel: onion skin, hojeo, mesa de luz | Harmony, Procreate Dreams, TVPaint | 🟡 (onion skin sí; sin herramientas de dibujo ni mesa de luz) |
| Limpieza de dibujos escaneados, escaneo, captura con cámara, stop-motion | OpenToonz | ❌ |
| Pincel de selección, biblioteca de imágenes | TVPaint 12 | ❌ |
| Nitidez al transformar | TVPaint 12 | n/a |
| Sensibilidad a la presión del lápiz | Synfig | ❌ |
| Sombreado de degradado basado en trazado | Synfig | ❌ |
| Texto con contenido variable/dinámico en runtime, en cualquier idioma | Rive, Procreate Dreams | ❌ |
| Fuentes personalizadas, títulos, créditos | Procreate Dreams | ❌ |
| Gestión de paletas y estilos | OpenToonz | ❌ |
| >50 tipos de capa (fractales, filtros, distorsiones) | Synfig | ❌ (≈7 tipos de capa) |

## Efectos y composición
| Funcionalidad | Vista en | OSATURA |
|---|---|---|
| Red de composición en vista de nodos con puertos con nombre | Harmony 24 | ❌ (árbol de capas + máscaras/mezclas) |
| Mattes, grupos bajo composite | Harmony | ✅ máscaras, grupos aislados |
| Lectura/escritura de CryptoMatte | Harmony 24 | ❌ |
| Toon shader (contorno, posterización) | Harmony 24 | ❌ |
| Máscara de enfoque, intercambio de canales | Harmony 24 | ❌ |
| Familia de desenfoques (radial, giro, direccional), bokeh, ruido fractal, mosaico | OpenToonz | ❌ |
| Particles FX | OpenToonz | ✅ capa de emisor |
| 15 modos de mezcla RGB + 5 alfa, dibujo fuera de pantalla por parte | Live2D 5.3 | 🟡 13 modos de mezcla; renderizado de grupos aislados ✅ |
| Clipping (ID de recorte por parte) | Live2D, Rive | ✅ máscaras alfa |
| Imágenes HDR / HDRI | Synfig | ❌ |
| Importación de vídeo, ProRes, rotoscopia sobre metraje | Procreate Dreams | ❌ |
| Motor de audio (voz en off, música, SFX) | Procreate Dreams | ❌ |

## 3D, pipeline, interoperabilidad, runtimes
| Funcionalidad | Vista en | OSATURA |
|---|---|---|
| Escena 3D, posado de huesos FBX, grafo de escena Vulkan, dibujo en plano de perspectiva | Harmony 24 | ❌ |
| Renderizado externo mediante Maya / Blender | Harmony 24 | ❌ |
| Importación vectorial: PDF / AI / SVG; EXR multicapa; alfa DPX; OpenColorIO (ACES) | Harmony 24 | ❌ |
| Importación de PSD sin Photoshop | Spine 4.2, Rive | ❌ |
| Importación de Lottie / pegado desde Figma | Rive | ❌ (solo **exportación** de Lottie) |
| Empaquetador de texturas / atlas | Spine | ❌ |
| Exportación de esqueleto JSON/binario + 19 runtimes oficiales / más de 40 de la comunidad | Spine | 🟡 JSON propio; sin runtimes |
| Runtimes de código abierto (Web, Apple, Android, React, Flutter, C++, Unity, Unreal) | Rive | ❌ |
| Modo Pixels (ajustar sprites a los píxeles de la pantalla) | Spine 4.2 | ❌ |
| Exportación: WebM con transparencia | Live2D 5.3 | ❌ |
| Exportación: SVG/PDF, secuencias PNG/EXR, profundidad de 16/32 bits, lotes, ProRes | Cavalry | 🟡 SVG, PNG ✅; PDF/EXR/ProRes/alta profundidad de bits ❌ |
| Renderizado en la nube, enlaces para compartir, incrustaciones | Rive | ❌ |
| Renderizado distribuido (Toonz Farm), control de versiones | OpenToonz | ❌ |
| Exportación a PDF / JSON de TVPaint | OpenToonz | ❌ |
| Colaboración de equipo en tiempo real, remezclas de la comunidad | Rive | ❌ |
| Exportación a HTML5 Canvas / web interactiva | Animate † | 🟡 reproductor HTML (prerrenderizado, no interactivo) |
| Linux/macOS/Windows nativos | Synfig, OpenToonz | ✅ |
| Licencia de código abierto | Synfig, OpenToonz | ✅ MIT/Apache-2.0 |
| Vista de jerarquía de capas, símbolos/instancias de biblioteca † | Animate | 🟡 jerarquía ✅; sin biblioteca de instancias |
| UI modular desacoplable, espacios de trabajo personalizados, tema oscuro/claro | TVPaint 12, Live2D 5.0 | ❌ |

## Roadmap candidato (ordenado por valor ÷ esfuerzo, sin GUI al principio)
1. Trim paths + seguimiento de trazado (huesos/capas a lo largo de un Bézier) — Rive, Spine
2. Superposiciones de onion skin + trayectorias de movimiento — casi todas las herramientas 2D
3. Importación de SVG (trazados, degradados) — Moho 14, Harmony, Synfig
4. Términos de viento / gravedad para muelles, dinámica de huesos controlada por entradas al estilo Smart Bone
5. Desenfoque, sombra paralela, resplandor, contorno, posterización (filtros de capa)
6. Instanciado / duplicadores de capas (clonar N a lo largo de un trazado, con desplazamientos por copia)
7. Máquina de estados interactiva en el runtime HTML/JSON (entradas, transiciones, mezcla)
8. Deformador envelope (cage), herramientas de copia/soldadura/suavizado de pesos, auto-malla de densidad uniforme
9. Cortes de cámara / cámaras guardadas, cámara con seguimiento de objetivo
10. Exportación glTF del rig; exportación EXR/PDF/WebM con alfa
11. Capas de texto (requiere carga de fuentes) con seguimiento de trazado
12. GUI del estudio: edición de puntos/huesos en el lienzo, editor de curvas, onion skin

## Fuentes adicionales (competidores)
- Notas de la versión de Toon Boom Harmony 24 y Premium Getting Started Guide (docs.toonboom.com)
- Spine 4.2 "The physics revolution" y "What is Spine?" (esotericsoftware.com)
- Live2D Cubism 5.0 y 5.3 "New Features" (docs.live2d.com)
- Aspectos destacados de Cartoon Animator 5.3 (discussions.reallusion.com)
- Página de funcionalidades de Rive (rive.app/features)
- Funcionalidades de Synfig Studio (zoftwarehub.com); documentación "What's New" de OpenToonz
- TVPaint Animation 12 (tvpaint.com); reseña de Cavalry (superrendersfarm.com)
- Adobe Animate "Layer parenting" (adobe.com/learn); Procreate Dreams (Cartoon Brew)
- Duik: página de etiqueta de Lester Banks (solo título †); Blender Grease Pencil: página del manual no recuperable (no cubierto)

---

# Anexo — Historiales de versiones de la competencia (ronda 2, 2026-10-19)

Leyenda como arriba: ✅ hecho, 🟡 parcial, ❌ todavía no. Solo se enumeran las funcionalidades confirmadas en páginas de fabricantes/proyectos o en la prensa. No se usaron sitios piratas; web.archive.org no es accesible desde este entorno, por lo que algunas versiones más antiguas siguen sin documentarse (véase "Carencias conocidas").

## Toon Boom Harmony 20 (notas de la versión del fabricante)
| Funcionalidad | OSATURA |
|---|---|
| Editor de línea central: eliminación de puntos, opción de suavidad | ❌ |
| Líneas de lápiz con textura → trazos de pincel con textura; cuentagotas de textura de lápiz | ❌ |
| Herramienta de línea curva / en S; pincel de relleno automático; pincel de repintado | ❌ |
| Guías de alineación, ajuste limitado a las capas seleccionadas | ❌ |
| Quick Ease / Ease In / Ease Out sobre las claves seleccionadas | 🟡 easing por clave ✅; comandos de easing por lotes ❌ |
| Marcadores de escena (sin duración, edición ripple) | ❌ |
| Vista de sustitución de dibujos (renombrar/filtrar) | 🟡 capas de conmutación ✅ |
| Nodos de sombreado de superficie: oclusión ambiental, sombra proyectada, mapa de superficie/normales | ❌ |
| **Nodo Weighted Deform** | 🟡 malla de imagen con pesos ✅ |
| Nodo Point Kinematic Output (muestreo de 1/2/3 puntos) | ❌ |
| Nodo de caché de prerrenderizado | 🟡 caché de horneado de muelles ✅ |
| Nodos de fundido de color / curvas / niveles / gamma | ❌ |
| Forma de partículas de región plana usando un dibujo vectorial | 🟡 emisor ✅; forma personalizada ❌ |
| Controladores maestros: deformador sobre deformador, Function Wizard, Slider Wizard | 🟡 canales ✅; UI de asistentes ❌ |
| Scripting en Python (PythonManager) | 🟡 operaciones de parche JSON + CLI; sin scripting incrustado |
| Gestión del espacio de color, renderizado de 16 bits, vídeo con postproceso por script | ❌ (postproceso con ffmpeg a mano) |
| Control Center: copia/importación/ejecución de scripts por lotes de escenas, cola de renderizado | ❌ |

## OpenToonz 1.7
| Funcionalidad | OSATURA |
|---|---|
| Modo de línea recta para el pincel | ❌ |
| Compatibilidad con pinceles de GIMP + MyPaint | ❌ |
| Modo de relleno Selection+Freehand | ❌ |
| Pegado de imagen desde el portapapeles | ❌ |
| Panel de preproducción / storyboard | ❌ |
| Paneles personalizados, ventana principal translúcida, rotar/voltear el visor | ❌ |
| Escala de Xsheet + navegación por etiquetas | 🟡 datos de línea de tiempo ✅; sin GUI |
| Exportación con información de movimiento de cámara | 🟡 cámara en Frame ✅ |
| **Exportación OCA (Open Cel Animation)** | ❌ |
| Exportación MOV y APNG | 🟡 MP4/GIF ✅; APNG/MOV ❌ |
| Conversión por lotes TZP→TLV | ❌ |
| Floor Bump Iwa, Flow Fx; renderizado en coma flotante + color lineal | ❌ |
| Compatibilidad con Haiku OS | n/a |

## Synfig 1.4.2 (y 1.4.0 de la ronda 1)
| Funcionalidad | OSATURA |
|---|---|
| Exportación Lottie incl. contorno avanzado y capas de desenfoque, gamma | 🟡 exportación Lottie ✅ (formas/transformación); desenfoque ❌ |
| Editor de atajos de teclado, atajos de herramienta de una sola tecla | ❌ |
| TimeLoop editable en el espacio de trabajo | 🟡 clips en bucle ✅ |
| Duplicación de capas Skeleton / Skeleton Deformation con enlaces de huesos | ✅ (los huesos son datos del proyecto) |
| Importador de SVG que gestiona atributos de estilo | ❌ (sin importación de SVG) |

## Elementos de la ronda 1 (páginas de fabricantes, abreviado)
- **Spine 4.2/4.3**: restricciones de física, sliders, mezcla de restricciones, ajuste a píxeles — muelles ✅ (horneados), sliders/mezcla 🟡 (`mix` de IK ✅), modo Pixels ❌.
- **Live2D Cubism 4.0–5.3**: ArtPath, onion skin, exportación WebM con alfa — ArtPath ❌; onion skin y WebM con alfa ✅.
- **Harmony 21/22**: deformadores con pesos, texturas de line-boil — 🟡 / ✅ (oscilación).
- **Cartoon Animator 5** — véanse las tablas de la competencia anteriores.

## Nuevos candidatos de roadmap de esta ronda
13. Exportación OCA (Open Cel Animation) y APNG — OpenToonz 1.7
14. Comandos de easing por lotes / "quick ease" en el editor — Harmony 20
15. Filtros de corrección de color (fundido, niveles, curvas, gamma) — Harmony 20
16. Sombreado de superficie (sombra proyectada / AO) para capas vectoriales — Harmony 20
17. Scripting incrustado (Python/Lua) sobre la API de operaciones de parche — Harmony 20
18. Nodo de caché de prerrenderizado / evaluación en caché — Harmony 20
19. Marcadores de escena + edición ripple en la línea de tiempo — Harmony 20
20. Composiciones de capas, reasignación de padre de huesos animada, capas de parche, seguir trazado, timecode SMPTE, zonas seguras, autotrazado, pincel de dispersión (carencias halladas en la investigación de Moho)

## Carencias conocidas (no recuperables desde las fuentes accesibles)
Anime Studio 8.x, 10.1.x, 11.1; detalles de Moho 12.5 y 14.1; Spine 3.0–4.1 (el blog de 3.7 es solo una página de vídeo); Harmony 17, 23, 25–27 (las páginas de documentación dieron error); Synfig 1.4.1; TVPaint 11; Cartoon Animator 4/5.1/5.2; changelog de Rive.

## Fuentes (ronda 2)
- docs.toonboom.com — notas de la versión de Harmony 20
- en.ubunlog.com — artículo sobre OpenToonz 1.7
- synfig.readthedocs.io — notas de la versión de Synfig 1.4.2
- esotericsoftware.com — resumen de funcionalidades de Spine 3.7 (página de vídeo, sin lista)

---

# Anexo — Fuentes de YouTube (ronda 3, 2026-10-19)

WebSearch restringido a youtube.com devuelve títulos de vídeos; WebFetch devuelve la **descripción/lista de capítulos** de cada vídeo (sin transcripciones). Solo se enumeran las funcionalidades nombradas allí; "novedad en la versión X" es lo que afirma quien sube el vídeo (a menudo el fabricante).

## Moho 14.1 (descripción del vídeo)
| Funcionalidad | OSATURA |
|---|---|
| Mostrar/ocultar tiradores en la herramienta Curvature | ❌ (sin GUI de lienzo) |
| Efectos de forma Shaded y Halo visibles en modo de edición | 🟡 degradados ✅; efectos halo/shade ❌ |
| Efecto de pixelación en Live Project Settings | ❌ |
| Preview Animation: mayor resolución, borde de salida oculto | 🟡 vista previa raster ✅ |
| Diálogos no modales para scripts Lua | ❌ (sin scripting) |
| Modo de mezcla PSD "Linear Dodge (Add)"; modos de mezcla actualizados | 🟡 revisar la lista de mezclas; add/dodge ❌ |
| Indicador de documento sin guardar en la barra de título | ✅ el estudio rastrea el estado de deshacer |
| Correcciones: evaluación de smart-bone, motion blur de subfotograma + dinámica de huesos, tiradores Bézier de longitud cero | n/a |

## Moho 12.5
Las páginas de título/descripción no dieron ninguna lista de funcionalidades (vídeo desactualizado; descarga gratuita de 12.5 retirada). Los títulos relacionados mencionan tutoriales de Smart Bones, rigs 2.5D con Smart Warp e instalación de "Mynd Tools" — Smart Bones 🟡 (canales/morphs), Smart Warp 🟡 (lattice FFD ✅).

## Spine
| Versión | Funcionalidad | OSATURA |
|---|---|---|
| 3.7 | Compatibilidad con audio | ❌ (el WAV solo se analiza para la sincronización labial) |
| 3.7 | Exportaciones renovadas | 🟡 |
| 3.7 | Escribir para buscar | ❌ |
| 3.7 | IK elástica / comprimida / uniforme | ❌ (la IK conserva la longitud) |
| 3.7 | Combinar skins, duplicar skins, marcadores de posición | 🟡 capas de conmutación |
| 3.7 | Copiar/pegar vértices | ❌ |
| 3.7 | Renderizado en cuadrícula de píxeles | ❌ |
| 3.7 | Mezcla de animación aditiva | 🟡 solo morphs aditivos |
| 3.7 | Runtime spine-cpp; reproductor web | ❌ / 🟡 reproductor HTML |
| 3.8 | Trazado de mallas | 🟡 auto-malla en cuadrícula; sin trazado de imagen |
| 3.8 | Empaquetado de texturas poligonal | ❌ |
| 3.8 | **Soft IK** | ❌ |
| 3.8 | Huesos y restricciones de skin | ❌ |
| 3.8 | Proyecto mix-and-match | ❌ |
| 3.8 | Carpetas, reglas, exportación a PSD mejorada, pantalla de bienvenida | ❌ |
| 3.8 | .json/.skel más pequeños; carga binaria en spine-ts | ❌ (solo JSON) |
| 4.0 beta | **Editor de gráficos / curvas** | 🟡 easing por clave ✅; sin editor |
| 4.1 beta | Función Sequence (adjunto de secuencia de imágenes) | ❌ |

## Toon Boom Harmony 25 / 25.2
| Funcionalidad | OSATURA |
|---|---|
| Herramienta Pencil Retouch | ❌ |
| Importar pinceles de Photoshop | ❌ |
| **Breakdown Assistant para animación cut-out** | ❌ (el generador de ciclo de caminata es afín) |
| Bloquear la regla de la línea de tiempo; preferencias de plantillas de acción | ❌ |
| Mejor compatibilidad con archivos de imagen, 3D y USDZ | ❌ |
| Art Layer Manager (intercambiar, borrar, fusionar, duplicar capas de arte; 25.2+) | ❌ (duplicar capas mediante operaciones de parche ✅) |
| Master Controller Grid Wizard | ❌ |

## Serie Cartoon Animator 5
| Funcionalidad | OSATURA |
|---|---|
| 5.1 Edición de ida y vuelta de PSD y vectores | ❌ |
| 5.1 Importación de secuencias de imágenes y GIF | ❌ |
| 5.2 **Motion Pilot** (manejar el movimiento como una marioneta arrastrando) | ❌ |
| 5.2 Animación por trayectoria de movimiento / edición de trazados | ❌ (roadmap n.º 1) |
| Dinámica de muelles, FFD, animación vectorial (existentes) | ✅ / ✅ / ✅ |

## Nuevos candidatos de roadmap de esta ronda
21. Soft IK + modos de IK elástica — Spine 3.8/3.7
22. Capa de secuencia de imágenes (adjunto de lista de fotogramas) e importación de GIF/APNG — Spine 4.1, CA 5.1
23. Filtro de salida de pixelación / ajuste a píxeles — Moho 14.1, Spine 4.2
24. Grabación al estilo Motion Pilot: capturar el arrastre en vivo de un hueso/tirador como claves — CA 5.2
25. Asistente de breakdown (intercalados automáticos con arcos para rigs cut-out) — Harmony 25
26. Trazado de mallas a partir del contorno alfa — Spine 3.8
27. Conjuntos de skins (intercambiar grupos de adjuntos juntos) — Spine 3.7/3.8
28. Pista de audio en el proyecto + sincronización de reproducción (no solo análisis) — Spine 3.7
29. Hooks de scripting en Lua/Python — Moho, Harmony 20

## Fuentes (ronda 3, YouTube)
- Moho 14.1: A Complete Breakdown — youtube.com/watch?v=QT4AiV1wO1k; Moho 14.1 Update — watch?v=FCiBKBjDaNg
- Resumen de funcionalidades de Spine 3.7 — watch?v=Fi0ks2LeYSM; resumen de funcionalidades de Spine 3.8 — watch?v=kU26B5465SQ; editor de curvas de Spine v4-beta — watch?v=37CeFRzrjqc
- Cambios de Harmony 25 — watch?v=kJ58C2DNIk8; Art Layer Manager 25.2+ — watch?v=AZc-8zHucOo
- Cartoon Animator 5.2 Motion Pilot — watch?v=Eu7vO2uGGN4; importación de secuencias/GIF de 5.1 — watch?v=O8EGwVwqgfs

---

# Anexo — Búsqueda exhaustiva de carencias (ronda 4, 2026-10-19)

Fuentes: notas de la versión oficiales de Synfig/Rive, títulos/descripciones de YouTube. Solo se enumeran las funcionalidades nombradas.

## Anime Studio 8 (serie "Pro Feature" de YouTube, títulos + descripciones)
| Funcionalidad | OSATURA |
|---|---|
| Trazado automático de imágenes (mapa de bits → vector editable) | ❌ (roadmap n.º 20) |
| Character Wizard (creación guiada de personajes) | 🟡 rigs de demostración/generador de caminata; sin asistente |
| Importar archivos de Photoshop (PSD) | ❌ |
| Real-Time Media Connection (enlace en vivo con medios externos) | ❌ |
| Herramienta Scale Points | 🟡 operaciones de parche sobre puntos |
| Preajustes de proyecto HD 1080p | ✅ cualquier tamaño |
Anime Studio 10.1 / 11.1: no se encontró ninguna fuente accesible.

## Synfig 1.4.1 – 1.4.4
| Versión | Funcionalidad | OSATURA |
|---|---|---|
| 1.4.1 | Capas de sonido mezcladas en el vídeo exportado | ❌ (sin pista de audio) |
| 1.4.1 | Duplicar capa copia todas las capas seleccionadas | ✅ (operaciones por lotes) |
| 1.4.3 | Aviso de enlace de huesos: enlazar la transformación en lugar del origen | ✅ |
| 1.4.3 | Importación de Papagayo (.pgo); correcciones de importación de SVG; frecuencia de fotogramas de secuencias de imágenes | ❌ / ❌ / ❌ |
| 1.4.3 | Correcciones del renderizado de GIF, compatibilidad con FFmpeg 3.x | ✅ GIF/MP4 |
| 1.4.3 | Autoguardado (hasta cada 15 s) | ❌ (guardado explícito) |
| 1.4.4 | Corrección de la exportación del último fotograma | ✅ cubierto por pruebas |

## Rive (notas de la versión oficiales, sept–oct 2026) — el competidor más activo
| Funcionalidad | OSATURA |
|---|---|
| Conversor String Template (texto a partir de valores de view-model, formato de números) | ❌ |
| Trazos discontinuos (Dash Effect), importados desde SVG | ❌ |
| Mantener Q/W/E + arrastrar para mover/rotar/escalar | ❌ (sin GUI) |
| Formas de pincel animables con claves en modo Animate | ❌ |
| Copiar/pegar claves de restricciones; los conversores conservan las fórmulas | 🟡 restricciones ✅, copiar ❌ |
| Enlace de datos relativo; view models; propiedades Blob | ❌ |
| Scripting en Luau (fetch, modelado de texto, canvas GPU, shaders/WGSL, mallas de imagen instanciadas) | ❌ |
| Estados de mezcla / Mix-by-view-model en la línea de tiempo | 🟡 horneado de crossfade |
| Listeners de teclado, foco y gamepad; entrada de texto; semántica de accesibilidad | ❌ |
| Maquetaciones en cuadrícula y pila, listas virtualizadas, restricciones de desplazamiento y física de scroll | ❌ |
| Set Parent (Alt+P) conservando posición y orden de dibujo | 🟡 reasignación de padre mediante operaciones de parche; conservar mundo ❌ |
| Align to Key Object; desplazamiento del hueso raíz; Scroll to selection | ❌ |
| Intensidad aditiva para rellenos/trazos; canales RGBA separados animables con claves/enlazables | 🟡 morphs aditivos; RGBA con claves ✅ |
| Trazos interiores/exteriores (acceso anticipado) | ❌ |
| Auto Trace (mejor para pixel art) | ❌ |
| Conversión de Flash `.fla`/XFL → Rive | ❌ |
| Preajustes de renderizado: máquina de estados → vídeo | 🟡 clip → MP4/GIF |
| Exportación `.riv` en el plan gratuito; CLI (`rive push/publish/ls/docs`), extensión de VS Code, simulación de entrada headless | 🟡 CLI ✅ (comandos propios) |
| Agente de IA en el editor | ❌ |
| Vista previa en Android; runtimes de React Native / Apple / Android / Flutter | ❌ |
| Historial de revisiones y restauración, sincronización offline, deshacer que conserva el zoom | 🟡 deshacer/rehacer ✅ |

## TVPaint Animation 11 (descripciones de YouTube)
Creación de atajos, subdibujos extraíbles, suavizado de líneas, grabación de dibujo, clic y arrastre del punto de pivote en KeyFramer (11.7.1), Sprite Generator, Inlay Texture FX, guías, vídeo "end of update releases" — todo ❌ (herramientas de pintura raster; OSATURA es vectorial/de rig).

## Toon Boom Harmony 17 (YouTube)
Free Form Deformer 🟡 (lattice FFD ✅), Master Controllers potenciados 🟡, RGB Difference Keyer ❌, Motion Blur ❌ (roadmap n.º 5).

## Cartoon Animator 4 / Spine 3.6 / Harmony 22–23
Se encontraron vídeos (CA 4: plugin de mocap Rokoko, actualización de movimiento 3D a 2D, mapas de normales e iluminación en 4.5; trazados de Spine 3.4; vídeo de funcionalidades de Spine 3.6; herramientas de dibujo de Harmony 22) pero las descripciones no enumeran funcionalidades → no se afirma nada. Destacables solo por el título: **retargeting de captura de movimiento** (CA 4) ❌, **mapas de normales/iluminación** (CA 4.5) ❌, **restricciones de trazado** (Spine 3.4) ❌.

## Nuevos candidatos de roadmap de esta ronda
30. Dash effect + trazos interiores/exteriores
31. Runtime de scripting incrustado (Lua/Luau) con API sin GPU
32. Enlace de datos / view models + entradas de máquina de estados (teclado/puntero/foco) para el reproductor HTML
33. Restricciones de desplazamiento / contenedores de maquetación (cuadrícula, pila) para animación de tipo UI
34. Importación de sincronización labial Papagayo (.pgo); importación al estilo Rhubarb
35. Set Parent conservando la transformación del mundo
36. Autoguardado + historial de revisiones en el estudio
37. Filtros de Motion Blur y RGB keyer
38. Restricciones de trazado (hueso a lo largo de un Bézier) — Spine 3.4, se fusiona con el n.º 1 del roadmap
39. Retargeting de captura de movimiento a rig 2D (BVH/JSON) — CA 4
40. Iluminación con mapas de normales para capas de imagen — CA 4.5
41. Pista de sonido mezclada en la exportación MP4 — Synfig 1.4.1
42. Importación de .fla / XFL y PSD

## Todavía no recuperable
Anime Studio 10.1 / 11.1, notas detalladas de Moho 12.5, páginas de detalle de Spine 3.0–3.6, Harmony 23, notas detalladas de Cartoon Animator 4.x, texto del changelog de TVPaint 11.

## Fuentes (ronda 4)
- synfig.readthedocs.io — notas de la versión 1.4.1, 1.4.3, 1.4.4
- releasebot.io/updates/rive — Rive Editor 0.9–0.9.272, CLI 1.2–1.6
- YouTube: serie Anime Studio 8 Pro Feature (q_HH2zOE-Yk, CUemDIlPH6Y, 408CLy652mU, l9eXLUO9Dus); demo de TVPaint 11 (MoqNrCAUy5k); Harmony 17 (1CU_3RJBcl4); lista de reproducción de Cartoon Animator 4

---

# Anexo — Redes sociales y canales de la comunidad (ronda 5, 2026-10-19)

**Realidad del acceso:** las páginas de X/Twitter están bloqueadas para su obtención por robots.txt (solo se ven fragmentos/títulos de los resultados de búsqueda); Facebook igual; las búsquedas en Reddit con filtro de dominio fueron rechazadas por el proxy de búsqueda y las búsquedas sin filtro no devolvieron hilos de Reddit; el foro de Lost Marble sirve una carcasa de redirección JavaScript sin contenido legible. No se eludió nada. Lo que se pudo leer:

## Canales oficiales de Moho (noticias de moho.lostmarble.com, fragmentos de X)
| Elemento | Notas | OSATURA |
|---|---|---|
| Moho 13.5.3 (abr 2022) | sincronización labial automática mejorada, mejor interfaz, huesos menos intrusivos | ✅ sincronización labial (heurística de WAV) |
| Moho 13.5.5 (X, may 2022) | **Relative Paths** para recursos enlazados | ✅ el directorio de recursos es relativo |
| Moho 14.3 "Mantis Shrimp" (dic 2024) | actualización centrada en los pinceles | ❌ |
| Moho 14.4 "Side Quest" (nov 2025) | exportación a motores de juego y software 3D | 🟡 Lottie/JSON propio; glTF/FBX ❌ |
| Moho 14.5 "Showtime" (oct 2026) | onion skins magnéticos, Shift & Trace, opciones de Liquid Shapes, EXR/PNG con perfiles de color, modo de reproducción en vivo para retransmitir/controlar personajes en tiempo real | véase la matriz principal; modo en vivo ❌ |
| Programa de embajadores (jun 2026); postura oficial: no se planean herramientas de IA (mar 2025) | política | n/a — OSATURA tampoco tiene funcionalidades de IA |
| Webinars (Víctor Paredes, etc.) | tutoriales | n/a |

## Comparaciones de la comunidad (foro de Spine, "Spine vs Moho 12")
- Los usuarios elogian en Moho: la física integrada, el enmascarado de capas, los smart bones, la edición vectorial.
- Los usuarios critican: FFD considerado débil; **sin runtime real**, exportación a juegos limitada a FBX.
- Creature es elogiado por su captura de movimiento integrada (FFD criticado).
- Las restricciones de física de Spine 4.2 reciben reacciones entusiastas en X (publicaciones de usuarios).
OSATURA: física ✅ (muelles horneados), enmascarado 🟡 (clip mediante booleanas), smart bones 🟡, edición vectorial ✅, runtime ❌, mocap ❌.

## Títulos de listas de deseos del foro vistos (contenido no legible)
"Concentrate on Timeline/Workflow improvements" (2024), "Road map for Moho", "Help Shape the Future of Anime Studio", "13.5.2 performance issue" — sugiere que la línea de tiempo/flujo de trabajo y el rendimiento son las áreas más solicitadas; no es una lista de funcionalidades verificada.

## Adiciones al roadmap
43. Modo de reproducción en vivo / retransmisión en tiempo real (entrada de poses por WebSocket → fotogramas renderizados) — Moho 14.5
44. Biblioteca de runtime en un lenguaje pequeño y portable (compilación WASM del evaluador) — la debilidad más citada de Moho
45. Enmascarado de capas como tipo de capa de primera clase (no solo operaciones booleanas)
46. Ergonomía de línea de tiempo/flujo de trabajo en el estudio (editor de curvas, selección múltiple de claves) — listas de deseos de la comunidad

## Fuentes (ronda 5)
- moho.lostmarble.com/blogs/news (17 publicaciones, dic 2021 – oct 2026)
- Fragmentos de búsqueda de X: publicaciones de @MohoAnimation (Relative Paths de 13.5.5; precios de Moho 14), publicaciones de usuarios sobre la física de Spine 4.2
- Hilo del foro de Spine "8033 spine vs moho 12" (de.esotericsoftware.com)
- Títulos de hilos del foro de Lost Marble (contenido ilegible)

---

# Anexo — Listas de deseos y tableros de solicitud de funcionalidades (ronda 6, 2026-10-19)

Solo se enumeran las solicitudes realmente legibles. La columna "OSATURA" indica si la solicitud es algo que el motor ya cubre.

## Moho (foro de Lost Marble)
Legible: solo el índice del foro. Existen las secciones "Feature Request Discussions" y "Unofficial Bug Reports and Feature Requests"; el título de solicitud más reciente visible: **"Dock Windows – Custom Interface"** (18 oct 2026). El hilo de larga trayectoria "Help Shape the Future of Anime Studio" (828 respuestas, 2011–2021) y otros hilos de listas de deseos ("Road map for Moho", "My wish for Moho in 2024 – Concentrate on Timeline/Workflow improvements", "Request for top 10 Anime Studio Pro 6 features") son carcasas de redirección JavaScript → contenido ilegible. Comparaciones concretas entre foros (foro de Spine): física, enmascarado y smart bones deseados en Spine; Moho criticado por un FFD débil y por no tener runtime.
| Solicitud | OSATURA |
|---|---|
| Ventanas acoplables / interfaz personalizada | ❌ (el estudio web tiene un diseño fijo) |
| Mejoras de línea de tiempo / flujo de trabajo | 🟡 |
| Runtime real para juegos | ❌ |
| FFD más potente | ✅ lattice de Bernstein + malla |

## Cartoon Animator (Reallusion Feedback Tracker + hilo de lista de deseos)
| Solicitud (votos cuando se muestran) | OSATURA |
|---|---|
| Curva de clave predeterminada en las preferencias | ✅ easing por clave |
| Edición de claves multicanal | 🟡 operaciones de parche por lotes |
| Ajustes de panel persistentes | ❌ |
| Marcadores y canales codificados por colores (izquierda rojo / derecha verde) | ❌ (espejo L↔R ✅) |
| "Uber key" para mostrar todos los canales animados | ❌ |
| Compatibilidad con SVG (Affinity, Inkscape); pegar trazados vectoriales | ❌ |
| Herramienta/panel de alineación | ❌ |
| Profundidad de campo / filtro de desenfoque | ❌ |
| Importación de GIF/APNG/vídeo por arrastrar y soltar colocada en la línea de tiempo | ❌ |
| Luces puntuales/ambientales con sombras | ❌ |
| Desplazamiento automático de la línea de tiempo | n/a |
| Mejor texto: kerning/interlineado animables | ❌ |
| Color SVG animable | ✅ |
| Ocultar globalmente / canales "shy" | ❌ |
| Paneles acoplados | ❌ |
| Varias escenas por proyecto | ❌ |
| **Panel Auto Animate (movimiento de reposo procedural en huesos)** | 🟡 oscilación de ruido coherente ✅ |
| Capa de dibujo en vivo / propiedades dinámicas editables (18) | ❌ |
| Hitbox y física (14) | 🟡 muelles |
| Importación de MOV transparente (14) | ❌ |
| Panorama de 360° (5); ideas de integración de IA (6) | ❌ |

## Tablero de solicitudes de funcionalidades de Live2D Cubism Editor (30 hilos, 2023–2026)
| Solicitud | OSATURA |
|---|---|
| Compatibilidad con Linux | ✅ |
| Atajos de teclado en la edición manual de mallas | n/a |
| Límite de blendshape hasta −100 % | ✅ morphs sin límite |
| Cambio de tamaño proporcional; herramienta de gravedad; bloqueo de malla a malla | ❌ / ❌ / ❌ |
| Glue para deformadores + pesos de glue precisos | 🟡 pesos ✅ |
| Importar física sin sobrescribir los grupos existentes | ❌ |
| Bloquear parámetros en el animador; edición en bucle vs repetición | ❌ |
| Opciones "Expand Warp based on Child"; cambiar el centro de rotación del warp | ❌ |
| Herramienta tipo "Set Vertex Flow", selección de bordes en bucle, mapeo de malla por trazo de densidad variable | ❌ (auto-malla uniforme) |
| Compatibilidad con pixel art | ❌ |
| Opacidad de vértices; filtrado de partes y visualización más limpia | ❌ |
| Mover claves fácilmente | ✅ operaciones de parche |
| Purgar de la memoria el atlas en caché | n/a |
| Ventana de física: % máximo de salida, edición masiva de rangos de péndulo | 🟡 |
| RISC-V / Cubism Core incrustado | ❌ |
| Drawables opcionales en la especificación CDI3 | n/a |

## Toon Boom Harmony
El foro de usuarios está cerrado y redirige a Discord (no se puede buscar); los hilos antiguos del "Feature Requests board" ("One can only dream: A Feature List", "Software Upgrade Suggestions Part 2", "Bitmap Layer Overhaul LONG OVERDUE", "Various Quality of Life suggestions") solo muestran el aviso de cierre. No se puede recuperar ninguna solicitud.

## Synfig (lista de ideas de GSoC 2019–2022, números de issues de GitHub enlazados)
| Solicitud | OSATURA |
|---|---|
| Copiar/pegar waypoints (#2139) | ✅ claves mediante operaciones |
| Cambio de pestañas con Ctrl+Tab, atajos de ventana, selección del área de reproducción, búsqueda de propiedades, alternar reglas, cambio de parámetros con el ratón | ❌ (GUI) |
| Rendimiento: renderizado/exportación más rápidos; caché de parámetros de capa; árbol de tareas de renderizado incremental | ✅ evaluate como función pura; sin caché ❌ |
| Port a GTK 4, compilaciones con Conan/vcpkg | n/a |

## Spine
Las solicitudes van al foro y se trasladan a un tablero público de Trello (no obtenible: robots.txt). Según el foro: la máxima prioridad histórica es la estabilidad del editor y las actualizaciones de runtimes. El plan de la época de Kickstarter (mallas → FFD → skinning → IK pinning) está ya completamente entregado, así que las mallas/FFD/skinning de OSATURA están a la par con esa antigua lista; IK pinning ✅ (restricción Pin). Títulos de solicitudes recientes del foro: compatibilidad con las últimas versiones de three.js / React Three Fiber; compatibilidad de Spine 4.3 en Defold.

## Rive
Roadmap público = "canal de Feature Requests" con etiquetas de estado (en curso / planificado / lanzado); requiere iniciar sesión para leerlo. No se puede recuperar nada más allá de las notas de la versión (ronda 4).

## OpenToonz, TVPaint, Pencil2D
Las páginas de GitHub (OpenToonz/Synfig) están bloqueadas por robots.txt; no se encontró ningún tablero de TVPaint legible; existe un hilo "Feature requests to add" de Pencil2D pero no se abrió (fuera de alcance).

## Adiciones al roadmap a partir de las listas de deseos
47. Capa de auto-animación / reposo procedural (respiración, parpadeo, balanceo de cabeza con rareza e intensidad) — deseo n.º 20 de CA; se apoya en la oscilación existente
48. Profundidad de campo de cámara y filtro de desenfoque — CA
49. Luces (puntuales/ambientales) con sombras proyectadas para capas de imagen — CA
50. Operaciones de alinear / distribuir (operación del editor) — CA
51. Codificación por colores de canales y visibilidad de canales "shy" en el estudio — CA
52. Varias escenas por proyecto + cortes de escena — CA
53. Auto-malla de densidad variable; suavizado de vertex-flow — Live2D
54. Parámetro de centro de rotación del deformador warp; cambio de tamaño proporcional — Live2D
55. Importación de física con fusión sin sobrescribir grupos — Live2D
56. Bloqueo de parámetros por pista durante la edición en bucle — Live2D
57. Diseño del estudio acoplable/personalizable — Moho, CA
58. Caché de parámetros entre fotogramas (evaluación incremental) — Synfig

## Fuentes (ronda 6)
- Reallusion Feedback Tracker: "Wish list for new features for future updates"; tablero de Cartoon Animator 5 (reallusion.com/FeedBackTracker)
- community.live2d.com — solicitudes de funcionalidades de Cubism Editor (página 1 de 7)
- Lista de ideas de Synfig GSoC 2022 (synfig-docs-dev.readthedocs.io)
- Índice del foro de Lost Marble (lostmarble.net/forum); hilos del foro de Spine 2250, 8033; hilo de Kickstarter 1501
- Preguntas frecuentes de Rive: "Does Rive have a public roadmap?"

---

# Anexo — Proyectos de código abierto en plataformas git (ronda 7, 2026-10-19)

**Acceso:** la API de búsqueda de GitHub no está disponible en este entorno ("sessions are bound to their configured repositories"), por lo que los proyectos se encontraron mediante búsqueda web y se leyeron en sus README públicos. Las API de GitLab y Codeberg están bloqueadas por el proxy; las páginas de Codeberg/GitLab solo aparecieron en resultados de búsqueda (Friction era legible). No se pudo obtener el número de estrellas. Varios proyectos son muy nuevos/tempranos — las funcionalidades enumeradas son las que declaran los README, no se verificaron ejecutándolos.

## Herramientas de código abierto similares de rigging / animación
| Proyecto | Plataforma | Funcionalidades destacadas | Comparación con OSATURA |
|---|---|---|---|
| **Rigby2D** (jaisurya-dev-art/rigby2d) | GitHub, MIT/Apache | Editor al estilo Spine (importación de PSD, malla + pintado de pesos, mallas enlazadas, restricciones IK/transform/path/physics/**slider**, skins con huesos específicos de skin, eventos, adjuntos de caja delimitadora/clipping/punto/trazado, dopesheet + editor de curvas), exportación Spine JSON 3.8/4.2/4.3 + .skel, atlas, runtimes JS/Rust/Godot, **servidor MCP integrado** para asistentes de IA, escritorio con Tauri | OSATURA carece de GUI de editor, atlas, runtimes, skins, eventos, restricciones slider/path; tiene evaluador de función pura, efectos de líquido/booleanas/emisor, sincronización labial, generador de caminata |
| **SkelForm** (Retropaint/SkelForm, Rust) | GitHub | IK FABRIK, nativo + web + envoltorio móvil | núcleo de rig similar ✅; sin GUI ❌ |
| **Ankhimate editor** | GitHub, pre-alfa | Modos Setup/Animate, slots + skins, FABRIK de cualquier longitud, restricciones de trazado/física, trazado de silueta de malla desde alfa, pincel de pesos con modos de mezcla + bloqueo de huesos, **líneas de tiempo de FFD**, eventos con audio, marcadores, desfases de pista por hueso, **editor de formato de exportación basado en plantillas**, runtime de referencia en Rust | ❌ en su mayoría GUI; ideas: plantillas de exportación, desfases de pista por hueso |
| **rig2d-rs** (tanhieuno41) | GitHub, Rust | Port de la física de Spine 4.2, IK con **mix/softness/bend/compress/stretch/uniform**, enlace automático de pesos por caída de distancia, suavizar/normalizar/limitar a 4 pesos, empaquetador de atlas por estantes, ghosting de onion skin, importación/exportación Spine JSON | ❌ IK suave/elástica, herramientas de pesos, atlas, interoperabilidad con Spine |
| **Boner2D**, **Skelly**, **omarvision 2d-bones-IK** | GitHub | Herramientas esqueléticas para Unity / diseñadores de juegos | básico |
| **Skeletal-Animator** (ostad-ai) | GitHub, Python | huesos + FABRIK, formas rectángulo/cápsula/círculo, 4 easings, onion skin, ventana de recorte de cámara con claves, **rangos de visibilidad por hueso**, spritesheet PNG+JSON, **MP4/WebM/MOV/GIF transparentes** | ❌ rangos de visibilidad, exportación de vídeo/spritesheet transparente, onion skin |
| **Stretchy Studio** (MangoLion) | GitHub, MIT | importación de PSD/PNG, **auto-rigging con IA** (detección de pose DWPose), asistente de configuración, animación de vértices con malla por parte, skinning de vértices para miembros, **recorte automático de ojos** (iris dentro del ojo), **shape keys al estilo Blender** con deslizadores de influencia, pistas de audio en la línea de tiempo, **exportación Spine 4.0 JSON** | ✅ morphs ≈ shape keys; ❌ auto-rig, recorte de ojos, audio |
| **Iki** (zeikar/iki) | GitHub, MIT | Motor web al estilo Live2D: cuadriláteros dirigidos por parámetros, deformadores de malla warp/cuadrícula, máscaras de stencil, física de muelles/cadenas, importación de PSD, **auto-rigger a partir de capas con nombres de rol** (parpadeo, mirada, giro de cabeza, asentimiento, inclinación, cejas, respiración, física del pelo), esquema JSON abierto con **ID StandardParameter**, servidor MCP + plugin de Claude Code | ✅ esquema JSON, sincronización labial; ❌ ID de parámetros estándar, auto-rigger, máscaras |
| **Umamo** (umamoorg/umamo) | GitHub, Java | Editor compatible con Live2D: lectura/escritura de CMO3/MOC3 (hasta Cubism 5.4), importación de PSD/KRA/CLIP, paleta de comandos, preajustes de atajos de teclado, **CLI de diagnóstico** (volcar/extraer/convertir/comparar modelos, preajustes de calidad de auto-malla); animación aún no soportada | ❌ CLI de diff de modelos, paleta de comandos |
| **Ayatsuri2D** | GitHub | Modelo `.aom` MessagePack (superconjunto de moc3), exportación moc3 v3–6, C ABI, fuzzing en CI, runtime sin pánicos, benchmarks de latencia p99.9 (temprano, "not usable yet") | ❌ formato binario, C ABI, fuzzing |
| **Ayagami** (AyagamiDev) | GitHub, Rust/wgpu | MOC3 hasta SDK 5.0 incl. glue y parámetros de blendshape, recálculo parcial, mezcla lineal/sRGB, mezcla avanzada/fuera de pantalla del SDK 5.3, componente para Godot | ❌ renderizador GPU, MOC3 |
| **kasane-2d**, **img2live**, **claude-live-2d**, **minivt** | GitHub | Modelado tipo Live2D / investigación de imagen única a marioneta | n/a |
| **Glaxnimate** (KDE) | espejo de GitHub / KDE | animación vectorial; las dependencias de compilación muestran trazado de mapas de bits con Potrace, libav, scripting en Python, libarchive; enfoque en Lottie | ❌ trazado, scripting |
| **Friction** (friction/friction, Codeberg) | Codeberg, GPL-3 | animación vectorial + raster para web/vídeo, motion graphics, SVG, Skia + FFmpeg + GLSL (basado en enve) | ❌ animación SVG, shaders |
| **Tahoma2D** | GitHub | 2D + stop-motion, basado en la versión Studio Ghibli de Toonz | ❌ |
| **Pencil2D** | GitHub, GPL-2 | mapa de bits + vector dibujado a mano, onion skin | ❌ |
| **q0idmation** (q0s0id) | GitHub | editor + reproductor + **lenguaje de scripting** | ❌ scripting |
| **svgMotion** (michaelsboost) | GitHub | animación vectorial en el navegador | ❌ |
| **kinetic** | GitHub | línea de tiempo de claves SVG → exportación CSS/GSAP | ❌ exportación CSS/GSAP |
| **dotlottie-rs** (LottieFiles) | GitHub, Rust | **máquinas de estados** (guardas/transiciones/acciones), tematización/**slots** (sobrescrituras en runtime de color/escalar/texto/vector), contenedor zip `.lottie` multianimación, API C + WASM, renderizador ThorVG | 🟡 exportación Lottie ✅; ❌ slots/máquinas de estados/contenedor dotLottie |
| **rive-runtime** | GitHub | Runtime en C++: renderizador vectorial GPU (Metal, Vulkan, D3D11/12, GL/WebGL), envuelto por Apple/Android/Flutter/Unity/Unreal/web | ❌ |
| **spine-runtimes** | GitHub | runtimes para Unity, Godot, Unreal, libGDX, MonoGame, XNA, Flutter, Haxe, C/C++/C#/TS, Android, iOS, GLFW, SDL, SFML; política de una rama por versión del editor | ❌ |
| **Repositorio de herramientas de Moho** (wanderjimenezRD/Moho-Pro-14.4-2D-Animation-Tools) | GitHub | solo un listado; la descripción no da funcionalidades — no evaluado | — |

Aparecieron repositorios de software pirata (p. ej. "activate-free" de Adobe Animate): no se abrieron ni se usaron.

## Ideas que vale la pena adoptar (nuevos candidatos de roadmap)
59. **Servidor MCP / API de agente** sobre el editor de operaciones de parche para que los asistentes de IA puedan hacer rigging y animar (Rigby2D, Iki) — encaja con la arquitectura de JSON-patch de OSATURA
60. **ID de parámetros estándar** (ParamAngleX, EyeOpen, MouthOpen…) para que cualquier host pueda controlar un modelo (Iki)
61. **Auto-rigger a partir de capas con nombres de rol**: parpadeo, mirada, giro/asentimiento/inclinación de cabeza, cejas, respiración, física del pelo (Iki)
62. **Recorte automático de ojos** (iris enmascarado por la forma del ojo) (Stretchy Studio) — mediante capa Boolean
63. **Restricción slider** y **restricción de trazado** (Rigby2D, Spine)
64. Modos de IK **suave / elástica / de compresión / uniforme** (rig2d)
65. **Herramientas de pesos**: suavizar, normalizar, limitar a N, copiar, bloqueo de huesos, modos de mezcla del pincel (rig2d, Ankhimate)
66. **Trazado de silueta de malla desde alfa** (Ankhimate)
67. **Rangos de visibilidad por hueso** (Skeletal-Animator)
68. **Exportación de vídeo con fondo transparente (WebM/MOV con alfa) y spritesheet PNG+JSON** (Skeletal-Animator)
69. **Motor de plantillas de exportación** (formatos de salida definidos por el usuario) (Ankhimate)
70. **Contenedor dotLottie `.lottie` + slots/tematización en runtime + máquinas de estados** (dotlottie-rs)
71. **CLI de diff / volcado / conversión de modelos** (`osatura diff a.json b.json`) (Umamo)
72. **Paleta de comandos + atajos reasignables** en el estudio (Umamo)
73. **Cargador con fuzzing, ruta de runtime sin pánicos, benchmarks de latencia** (Ayatsuri2D) — endurecimiento de robustez de `Engine::new`
74. **Importación/exportación Spine JSON** para interoperabilidad (Rigby2D, rig2d, Stretchy Studio)
75. **Empaquetador de atlas por estantes** para capas de imagen (rig2d, Rigby2D)
76. **Eventos con valores con clave y señales de audio** en la línea de tiempo (Rigby2D, Ankhimate)
77. **Desfases / desplazamiento temporal de pista por hueso** (Ankhimate)

## Fuentes (ronda 7)
- github.com/jaisurya-dev-art/rigby2d; Retropaint/SkelForm; Ankhimate/editor; tanhieuno41/rig2d-rs (release v0.1.0)
- github.com/zeikar/iki; MangoLion/stretchystudio; umamoorg/umamo; Yoru-KenomoVT/ayatsuri_2d; AyagamiDev/ayagami
- github.com/ostad-ai/Skeletal-Animator; KDE/glaxnimate; tahoma2d/tahoma2d; pencil2d/pencil
- github.com/rive-app/rive-runtime; EsotericSoftware/spine-runtimes; LottieFiles/dotlottie-rs
- github.com/topics/animation-editor, /topics/vector-animation
- codeberg.org/friction/friction

---

# Anexo — Otras fuentes públicas (ronda 8, 2026-10-19)

Fuentes: itch.io, Steam, AlternativeTo, Wikipedia, resultados de búsqueda académicos/de patentes, reseñas de herramientas de After Effects. Las afirmaciones proceden de páginas de tiendas/proyectos, no de pruebas prácticas.

## Herramientas de rigging independientes / de nicho (itch.io, Steam)
| Herramienta | Funcionalidades destacadas | OSATURA |
|---|---|---|
| **Inochi Creator / Inochi2D** (Kitsunebi Games, código abierto, beta) | rigging de marionetas 2D abierto orientado a VTubers + runtime en tiempo real; formato propio (no compatible con Live2D/Spine); exportación a GIF/secuencia de imágenes; interfaz multilingüe | 🟡 mismo nicho (formato de rig abierto) |
| **Keyframe.it** (navegador, gratis) | huesos + IK + mallas con pesos + movimiento de muelles; **clips de símbolos de juego** (idle, land, anticipation, win); editor de imágenes integrado + importación de PSD; exportación a GIF, sprite sheet, vídeo, reproductor HTML autónomo, **paquetes Spine 3.8/4.2**; autoguardado en el navegador; conector de agente de IA **keyframe-mcp**; asistente de IA integrado opcional | ✅ reproductor HTML/GIF; ❌ sprite sheet, paquete Spine, MCP, editor en el navegador |
| **RigBone 2D** (Godot) | posado de huesos, pose como fotograma, onion skin, recorte + fondo transparente, secuencia de fotogramas PNG (sin necesidad de runtime) | ❌ onion skin, secuencia PNG con recorte alfa |
| **SpineBones** | slots + adjuntos de imagen, grupos, IK, **vista previa de audio en la línea de tiempo**, imagen de referencia de fondo, exportación JSON / sprite sheet / WebM | ❌ |
| **PySpine, Tangent (sprite animator)** | listados en itch.io; no se leyó texto de funcionalidades | — |
| **Mocap2D / FBX2SPINE** (Steam) | importación de mocap FBX/BVH; **Auto Match** del esqueleto por nombres + jerarquía; **Smart Retargeting** (proyección, bueno para vista lateral); **Direct Retargeting** por eje; **conversión automática in-place** para bucles de locomoción; **IK de contacto con el suelo** (bloqueo de pie); vista previa en vivo; importación/exportación de FBX, BVH, esqueletos Spine, Unity 2D Animation | 🟡 retarget por nombre ✅; ❌ importación de mocap, IK de bloqueo de pie, conversión in-place |
| **VTube Studio** (Steam) | seguimiento por webcam (OpenSeeFace), **boca controlada por voz**, ratón/lápiz como entrada de seguimiento, asignación de cualquier entrada → cualquier parámetro del modelo, atajos de teclado para expresiones/animaciones, "items" PNG fijados que siguen al modelo; solo modelos Live2D | 🟡 canales de parámetros ✅; ❌ asignación de entrada en vivo, atajos de teclado, items fijados |
| VBridger, Vitamins | Herramientas de Steam en torno al VTubing; no evaluadas | — |

## Ecosistema de rigging de After Effects (reseña de School of Motion)
| Funcionalidad | Fuente | OSATURA |
|---|---|---|
| **Controlador joystick** (5 poses clave: origen/derecha/izquierda/arriba/abajo) interpoladas en 2D | Joysticks 'n Sliders | ❌ — candidato fuerte |
| **Sliders** que mezclan un número ilimitado de poses | JnS, Duik | 🟡 canales de morph |
| Estructuras (huesos independientes de la ilustración, reutilizables), autorig | Duik Bassel | ✅ |
| Controladores: slider, slider 2D, ángulo | Duik | ❌ |
| **Connector** (slider/joystick/rotación que enlaza cualquier propiedad con otra), control maestro de propiedades | Duik | 🟡 ¿expresiones? (motor de expresiones ✅ en canales) |
| Claves de pose intermedia entre estados de pose | Duik | ✅ |
| Animar trazados de máscara | JnS | ✅ morphs de trazado |

## Aplicaciones competidoras / alternativas (AlternativeTo, lista de Wikipedia a 2026)
Tahoma2D (fork de OpenToonz, stop-motion), TupiTube, enve (discontinuado; se convirtió en Friction), Expressive Animator (SVG interactivo), Cavalry, Jitter (diseño de movimiento web), Blazy (Figma → animación, exportación CSS/React), Animation Desk, Pivot Animator, FlipaClip, animación de Krita, Blender (mar 2026), Clip Studio Paint (animación), Adobe Character Animator (marionetas con webcam; la página "what's new" está bloqueada por robots.txt), Adobe Animate (última versión listada oct 2023), CrazyTalk, DigiCel FlipBook, Apple Motion, RETAS. Ninguna añade funcionalidades de rigging más allá de lo que cubren las rondas anteriores; ideas nuevas: **preajustes de herramienta de diseño/Figma → animación con exportación CSS/React** (Blazy) y **marionetas con webcam** (Character Animator, VTube Studio).

## Literatura académica y de patentes (solo títulos/resúmenes)
- Jacobson et al. 2011, **Bounded Biharmonic Weights** para deformación en tiempo real — pesos de skinning suaves automáticos a partir de un cage/esqueleto ✅ idea: mejorar `auto_skin` (actualmente basado en distancia).
- Patentes de Adobe US9865073 / US10078910, **"Computer animation of artwork using fast 2-dimensional deformation solving"** (la deformación de malla de Character Animator). *Nota de sala limpia:* OSATURA usa FFD de Bernstein y skinning con pesos, no ese solucionador patentado; cualquier futuro solucionador de tipo ARAP debería revisarse con asesoría jurídica. Esto no es asesoramiento jurídico.
- Auto-rigging en arXiv 2025–26: **ASMR** (esqueleto de malla 3D + skin mediante priors generativos 2D), UniRig, "How to Train Your Dragon" (rigging por difusión), HumanRig, SPRig, UniMate — todos 3D/ML; relevantes solo como futura funcionalidad de "sugerencia de auto-rig".
- Stony Brook TVC 2017 **retargeting automático de pesos de skinning** — copiar pesos entre mallas (✅ roadmap: copia de pesos).

## Nuevos candidatos de roadmap de esta ronda
78. **Controlador joystick**: punto de control 2D que mezcla 5 poses clave (y variantes de N poses) → controla cualquier canal — JnS / Duik / parámetros de Live2D
79. Importación de mocap (BVH) + coincidencia automática de esqueleto + conversión de bucle in-place + IK de bloqueo de pie — Mocap2D
80. Asignación de entrada en vivo: cualquier entrada (ratón, MIDI, nivel de audio, puntos de referencia de webcam) → parámetro, más atajos de teclado para expresiones — VTube Studio
81. Items/props fijados que siguen a un hueso/ancla con su propio desplazamiento — VTube Studio
82. Preajustes de clips de símbolos de juego (idle / land / anticipation / win) — Keyframe.it
83. Exportación de spritesheet (PNG + JSON) y de secuencia PNG transparente con recorte — Keyframe.it, RigBone
84. Vista previa de audio en la línea de tiempo con forma de onda — SpineBones
85. Capa de imagen de referencia (fondo no renderizado) — SpineBones
86. Pesos de auto-skin biharmónicos acotados o por difusión de calor (sustituyendo la caída por distancia) — artículo BBW
87. Preajustes de herramientas de diseño → exportación CSS/React — Blazy
88. Marionetas con seguimiento facial por webcam mediante el estudio (WebRTC + modelo de puntos de referencia) — Character Animator, VTube Studio

## Fuentes (ronda 8)
- itch.io: kitsunebi-games/inochi-creator, keyframeit.itch.io, itch.io/blog/1586091 (RigBone 2.0), itch.io/t/6177139 (SpineBones)
- Steam: FBX2SPINE / Mocap2D (app 2467960), VTube Studio (app 1325860)
- schoolofmotion.com — Joysticks 'n Sliders vs Duik Bassel
- alternativeto.net (alternativas a Moho, Expressive Animator); en.wikipedia.org/wiki/List_of_2D_animation_software
- Jacobson 2011 BBW (homes.cs.washington.edu/~jovan/papers/jacobson-2011-bbw.pdf); patentes US9865073B2 / US10078910; arXiv 2503.13579, 2503.15586, 2504.12451; retargeting de pesos de Stony Brook TVC 2017

---

# Anexo — Adobe Character Animator, conjunto completo de funcionalidades (ronda 9, 2026-10-19)

Fuentes: tabla de versiones de Wikipedia (incl. 26.0 de ene 2026), página de producto de Adobe, página "Behaviors" de Adobe Help y notas de la versión (oct 2018, oct 2020/feb 2021, oct/dic 2021, 2023). El índice "what's new" de Adobe solo devuelve 2023 y 2026; Wikipedia indica que 24.x y 25.x son versiones solo de estabilidad.

## Concepto
Las ilustraciones en capas de Photoshop/Illustrator (también EPS/SVG desde oct 2018) se convierten en una **marioneta** con rig automático a partir de **nombres/etiquetas de capas**; la interpretación se captura en vivo (cara/cuerpo por webcam, micrófono, teclado, ratón, MIDI) como **takes** en una línea de tiempo; las salidas van a secuencias PNG, WAV, Media Encoder (incl. **vídeo con alfa**), Dynamic Link con After Effects / Premiere, y Syphon / Mercury Transmit en directo. El modo Starter es gratuito; Pro requiere Creative Cloud.

## Behaviors (lista completa de Adobe Help)
| Behavior | Qué hace | OSATURA |
|---|---|---|
| **Face** | cabeza, ojos, cejas, nariz, boca desde la webcam; parallax e intensidad de posición de la cabeza | ❌ (sin seguimiento en vivo) — roadmap n.º 88 |
| **Body** (Body Tracker, Sensei AI, 22.0) | brazos, torso, piernas desde la webcam | ❌ |
| **Lip Sync** | visemas de la boca a partir de la voz; precisión asistida por transcripción (22.1); motor 3.4 mejor en F, W-Oo, menos L/M | 🟡 WAV → 5 visemas ✅; alineación de transcripción ❌ |
| **Nutcracker Jaw** | mandíbula inferior a partir de la cara/voz | ❌ |
| **Eye Gaze** | las pupilas se mueven de forma independiente | ✅ restricciones aim/canales |
| **Head & Body Turner** | intercambia grupos de capas (frontal, tres cuartos, lateral) al girar la cabeza/el cuerpo | 🟡 capas de conmutación ✅; sin intercambio automático según el ángulo |
| **Walk** | ciclo de caminata, carrera, énfasis de pose, balanceo de hombros/caderas, **Pin Feet When Standing** (3.4/3.5) | ✅ generador de caminata; ❌ pin-feet |
| **Limb IK** (antes Arm IK) | dirección de flexión + estiramiento para brazos y piernas | 🟡 IK de 2 huesos ✅; estiramiento ❌ |
| **Leader/Follower** (22.5) | da forma a cómo se flexionan/estiran los miembros | ❌ |
| **Dragger** | el ratón/táctil mueve una región | ❌ (el estudio no es interactivo en el lienzo) |
| **Physics / Dangle** | balanceo, colisión, gravedad, viento; squash & stretch, amortiguación | 🟡 muelles ✅; colisión/viento ❌ |
| **Magnets** | los tiradores se acoplan por proximidad | ❌ |
| **Particles** | emite objetos físicos (nieve, disparos de cañón) | ✅ emisor |
| **Motion Lines** | líneas de estela tras objetos rápidos | ❌ |
| **Wiggler** | oscilación en ilustraciones vectoriales | ✅ oscilación/boil |
| **Breath** | movimiento de respiración | 🟡 canal manual; roadmap n.º 61 |
| **Auto Blink** | parpadeos automáticos / parpadeo de capas | ❌ (roadmap n.º 61) |
| **Motion Library** (23.0, más de 350 movimientos, parámetro de intensidad 23.1) | aplicar animaciones de cuerpo completo | 🟡 solo caminata; ❌ biblioteca |
| **Cycle Layers** | recorrer las capas de un grupo | 🟡 capas de conmutación ✅; ciclado de fotogramas ❌ |
| **Layer Picker / Motion Trigger / Fader** | mostrar una capa; cambiar según la dirección del movimiento; fundido al activar un disparador | 🟡 |
| **Transform** | mover/escalar/rotar/opacidad de tiradores | ✅ |
| **Handle Fixer** | bloquear partes de una malla | ✅ pins |
| **Triggers** | la pulsación de una tecla muestra capas/gestos; **panel Controls** con sliders, diales de ángulo, botones, asignación MIDI | ❌ — roadmap n.º 80 |
| **Collide, Dynamic, Cycle Pause, Auto-swap (22.5)** | etiquetas/behaviors para física e intercambios tipo fotograma | ❌ |

## Funcionalidades de línea de tiempo, grabación y flujo de trabajo
| Funcionalidad | Versión | OSATURA |
|---|---|---|
| Takes, replays (grabaciones reutilizables activables), smart replays, replays editables | 2.0 / 22.0 / 23.6 | 🟡 clips ✅; replays ❌ |
| Claves, estiramiento temporal de claves, cámaras de escena, audio activable | 3.0 / 3.2 | ✅ claves/cámara; ❌ audio activable |
| Speech-Aware Animation (movimiento de cabeza/cejas a partir de la voz, Sensei) | 3.4 | ❌ — roadmap n.º 61 |
| Fusionar/secuenciar takes; filas de take en color; filtros de búsqueda (`kind:` `shared:` `armed:`); filas shy; contraer takes | 3.4 | ❌ |
| Ajuste a fotogramas / objetos; ajuste de la línea de tiempo | 3.5 | ❌ |
| Operaciones de área de trabajo: borrado ripple, insertar tiempo, bucle, recorte, exportación | 2.0 | 🟡 operaciones de clip ✅ |
| Panel de problemas de rigging; búsqueda de etiquetas y disparadores | 3.0 | ✅ la validación recoge todos los errores |
| Marcadores de historial; recuperación automática de proyectos dañados (4.2); nuevo formato de archivo compacto y resistente a la corrupción (4.0) | | 🟡 deshacer ✅; recuperación ❌ |
| Puppet Maker (plantillas: pelo, piel, accesorios) | 22.1.1 | ❌ |
| Characterizer (convertir una foto/pintura en una marioneta) | 2.0 | ❌ |
| Quick Export para compartir en redes sociales | 22.5 | 🟡 GIF/MP4 ✅ |
| Instantáneas de escena, cuenta atrás de grabación, ilustraciones personalizadas para botones de control | 1.5 | ❌ |
| Selección de cámara, multimonitor, HiDPI | 2.0 | n/a |
| Salida en vivo (Syphon, Mercury Transmit), extensión Twitch Character Trigger | 1.1 / 2.1 | ❌ — roadmap n.º 43 |
| MIDI, scripts mediante V8 (JavaScript) | 1.1 / 2.0 | ❌ — roadmap n.º 31 |
| Compartir/publicar marionetas, archivos de marioneta compartibles | Beta 5 | 🟡 JSON portable |
| Dynamic Link con AE / Premiere | Beta 5 | ❌ |
| Debilidades señaladas (reseña): curva de aprendizaje pronunciada, dependencia del hardware (webcam/micrófono), largo tiempo de preparación del personaje, Pro de pago, complejidad de configuración de disparadores | | oportunidad: pipeline offline determinista de OSATURA, CLI/JSON |

## Qué destaca para OSATURA (nuevos candidatos de roadmap)
89. **Auto-rig basado en etiquetas a partir de nombres de capa** (cabeza, ojos, cejas, visemas de la boca, etiquetas de brazo/pierna) — idea central de Character Animator; amplía el n.º 61 del roadmap
90. **Takes y replays**: grabar la interpretación de parámetros como clips reutilizables y activables con easing de mezcla
91. **Pin Feet When Standing** y bloqueo de contacto con el suelo para el generador de caminata
92. Comportamientos **Leader/Follower** de estiramiento de miembros; **Magnets**; **Motion Lines**
93. Formato de **Motion Library** (clips de movimiento con nombre con intensidad 0-100 %) 
94. **Head & Body Turner**: intercambio automático de grupos de vistas a partir de un canal de ángulo de guiñada (yaw)
95. **Sincronización labial asistida por transcripción** (alineación forzada de texto con fonemas del WAV)
96. **Movimiento de cabeza/cejas consciente del habla** a partir de la energía y las pausas del audio
97. **Exportación de vídeo con alfa** (ProRes 4444 / WebM VP9 con alfa) — también en Skeletal-Animator (roadmap n.º 68)
98. **Audio activable** y parámetros reactivos al audio

## Fuentes (ronda 9)
- en.wikipedia.org/wiki/Adobe_Character_Animator (lista de funcionalidades y tabla de versiones hasta 26.0, ene 2026)
- adobe.com/products/character-animator.html
- helpx.adobe.com — Behaviors in Character Animator; Feature summaries oct/dic 2021, oct 2020/feb 2021; What's new (2023–2026)
- community.adobe.com — "Character Animator October 2018 detailed changes"
- manytools.com/review/adobe-character-animator (puntos fuertes/débiles)
