<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Privacidad

Este programa no transferirá ninguna información a otros sistemas en red salvo que lo solicite expresamente el usuario o la persona que lo instala o lo opera.

En detalle, para `osatura`, `osatura-studio`, la biblioteca C y el módulo WebAssembly:

* **Sin telemetría, analítica, informes de fallos, publicidad ni comprobaciones automáticas de actualizaciones.** Los programas no contienen código que contacte con un servidor por sí solo.
* **Un solo comando contacta con un servidor, únicamente cuando lo escribes.** `osatura update-check` hace una única petición HTTPS anónima a GitHub (`api.github.com`) para obtener la lista pública de versiones, usando el programa `curl` de tu sistema; GitHub ve tu dirección IP y esa petición, según la [declaración de privacidad de GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). No se descarga ni se instala nada y no se envía nada sobre ti ni sobre tus proyectos. Para que GitHub no vea tu dirección, añade `--proxy-preset tor` (o `tor-browser`, `warp`) o `--proxy-url` con tu propio proxy; consulta [CLI_REFERENCE.md](CLI_REFERENCE.md).
* **El estudio es un servidor local.** `osatura studio` y `osatura-studio` escuchan por defecto en `127.0.0.1` (solo este ordenador). Solo son accesibles desde otras máquinas si tú mismo los inicias con `--bind` en otra dirección ([SECURITY.md](SECURITY.md)). La página que sirven no carga nada de Internet (su Content-Security-Policy solo permite el servidor local).
* **Los enlaces que pulsas.** El estudio y la documentación contienen enlaces (por ejemplo a GitHub o a una página de donaciones). Seguir uno abre ese sitio en tu navegador, bajo la política de privacidad de ese sitio.
* **Programas externos.** `ffmpeg` se inicia solo para vídeo y algunos formatos de audio, solo cuando pides esa exportación, y trabaja con archivos locales.
* **Las sesiones compartidas guardan un historial.** Con `--collab-log` el estudio escribe cada edición de la sesión, con los nombres de los participantes, en un archivo que tú eliges; con tokens por participante, los nombres son los que tú has creado. Trata ese archivo como datos personales de los participantes y bórralo cuando termine la sesión ([SECURITY.md](SECURITY.md), [UNINSTALL.md](../../UNINSTALL.md)). Las herramientas de puntos de referencia faciales escriben sus resultados solo en los archivos que indiques.
* **Tus archivos siguen siendo tuyos.** Los proyectos, renders y ajustes se escriben solo donde le indicas al programa que los escriba, además del pequeño archivo de ajustes que figura en [UNINSTALL.md](../../UNINSTALL.md).

El sitio web del proyecto (<https://osatura.chiuta.ro>) es una página estática alojada en GitHub Pages; se le aplica la declaración de privacidad propia de GitHub.

Preguntas: <alexio@trom.tf>.
