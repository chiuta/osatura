# Notas sobre patentes

**Esto no es asesoramiento jurídico ni un dictamen de libertad de operación.** Registra lo que NO se ha comprobado, para que nadie confunda la ausencia de un análisis con una autorización. Las licencias de OSATURA (MPL-2.0 para las bibliotecas, AGPL-3.0-or-later para la CLI) contienen concesiones de patentes de los colaboradores sobre el código propio del autor, pero no dicen nada sobre las patentes de terceros. Un profesional no ha realizado ninguna búsqueda de patentes.

## Análisis aún pendientes

| Área | Por qué puede importar | Estado |
|---|---|---|
| Marionetas controladas por el rostro (`faceparam`, `facedet`, `liveinput`: puntos de referencia o proporciones faciales que controlan parámetros del rig) | La animación de personajes 2D guiada por una interpretación (performance-driven) es un campo activo de solicitudes comerciales. | Sin analizar. El código usa medidas publicadas en libros de texto (relación de aspecto del ojo, filtro One-Euro, HOG + regresión logística); eso no excluye reivindicaciones sobre la combinación. |
| Máquinas de estados y enlace de datos (`statemachine`, `runtime_machine`, `bind`) | Los runtimes de animación interactiva con máquinas de estados, blend trees, entradas y enlace de view-model son un área de funciones comerciales. | Sin analizar. Diseño propio del modelo de datos, no una autorización en materia de patentes. |
| FABRIK, coordenadas de valor medio, pesos biarmónicos acotados (`rig`, `cage`, `weights`) | Algoritmos publicados; algunos autores y empleadores presentaron patentes sobre técnicas de deformación y skinning. | Sin analizar. FABRIK (Aristidou y Lasenby, 2011) y MVC (Floater, 2003) se usan a partir de los artículos; el módulo de pesos usa difusión armónica simple, no el solucionador biarmónico acotado publicado. |
| Relleno de trazados por stencil y luego cobertura (`osatura-gpu`) | La técnica procede de la investigación sobre renderizado de trazados en GPU y de extensiones de fabricantes. | Sin analizar. Aquí se usa solo mediante la API estándar de wgpu con un búfer de stencil simple. |
| Códecs de vídeo y audio mediante ffmpeg (`export` .mp4 / .webm, `soundtrack`) | H.264 y AAC están cubiertos por consorcios de licencias (MPEG LA / Via LA y otros). VP9 y AV1 se comercializan como libres de regalías, pero se han anunciado reivindicaciones de consorcios de patentes y de patentes individuales para ambos. | Sin evaluar. No consideres VP9/AV1 como autorizados. |

## ffmpeg
ffmpeg es un **subproceso externo** que aporta el usuario. OSATURA no lo enlaza, no lo distribuye y no debe incluirlo en ninguna versión: incluirlo haría responsable al distribuidor de la licencia propia de ffmpeg (LGPL/GPL según la compilación) y de las licencias de los códecs mencionados. Cada opción de códec es una elección del propio usuario en su propia instalación.

## Antes de una versión comercial
Haz que un abogado de patentes revise las áreas anteriores, en las jurisdicciones donde se distribuirá el software, y registra el resultado aquí y en CLEANROOM.md (regla 5).
