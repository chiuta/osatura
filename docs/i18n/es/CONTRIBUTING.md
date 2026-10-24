# Contribuir a OSATURA

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/README.md) · [Română](https://github.com/chiuta/osatura/blob/main/README.ro.md) · [Français](https://github.com/chiuta/osatura/blob/main/README.fr.md) · [Italiano](https://github.com/chiuta/osatura/blob/main/README.it.md) · [Español](https://github.com/chiuta/osatura/blob/main/README.es.md) · [Português](https://github.com/chiuta/osatura/blob/main/README.pt.md) · [Deutsch](https://github.com/chiuta/osatura/blob/main/README.de.md) · [Magyar](https://github.com/chiuta/osatura/blob/main/README.hu.md)

* **Regla de implementación independiente.** Aporta únicamente código, matemáticas y formatos que puedas compartir legalmente. No copies, descompiles ni apliques ingeniería inversa a herramientas, archivos o interfaces propietarios, y no pegues código de proyectos cuya licencia no sea compatible (véase CLEANROOM.md). No leas ni pegues código fuente de herramientas de la competencia mientras escribes una función. Cita la norma pública o el libro de texto a partir del cual implementaste.
* **Licencia de las contribuciones (entrada = salida).** Una contribución se acepta bajo la licencia de la parte que modifica: MPL-2.0 para `osatura-core`, `-capi`, `-wasm`, `-gpu` y `scripts/`; AGPL-3.0-or-later para `osatura-cli`; CC-BY-4.0 para la documentación y los recursos sintéticos. Cada archivo fuente lleva una línea SPDX que debe ser igual a la `license` de su crate (lo comprueba `osatura-cli/tests/spdx_audit.rs`).
* **El sign-off no concede ningún derecho de relicenciamiento.** El sign-off que figura más abajo certifica únicamente el origen. **Actualmente NO es posible una licencia comercial o dual (por ejemplo, una licencia propietaria para quienes integren el motor) para el código aportado**: haría falta un Contributor Licence Agreement aparte o una cesión de salida explícita de cada contribuyente, y no existe ninguno. Mientras no exista, nadie (incluido el mantenedor) puede afirmar que puede relicenciar las contribuciones de otras personas.
* **Sign-off (DCO).** Cada commit debe llevar `Signed-off-by: Your Name <email>` (`git commit -s`), que certifica el Developer Certificate of Origin 1.1 reproducido a continuación.
* Añade pruebas; ejecuta `cargo test --workspace` y `scripts/licence-audit.sh` antes de abrir una pull request.

## Developer Certificate of Origin 1.1 (literal)

*Reproducido sin cambios de https://developercertificate.org/ (Copyright (C) 2004, 2006 The Linux Foundation and its contributors); sus propios términos permiten copias literales. Este texto no está cubierto por la CC-BY-4.0 del resto de este archivo.*

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
