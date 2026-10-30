<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Glosario de OSATURA

Términos del dominio usados en el código y en la documentación, en el orden alfabético del original en inglés. "Véase" remite
al módulo (en `osatura-core/src` salvo que se indique otra cosa) o al documento donde el término se implementa o
se especifica. Documentos: [PROJECT_FORMAT.md](PROJECT_FORMAT.md), [ARCHITECTURE.md](ARCHITECTURE.md).
Las fuentes de las técnicas se enumeran en `CLEANROOM.md`.

**Restricción de aim.** Una restricción que rota un hueso para que su eje +x apunte a la cabeza de otro hueso,
más un desfase. Véase `rig`; PROJECT_FORMAT sección 4.3.

**Ancla.** Un punto de un trazado de Bézier con una posición y manejadores de entrada y de salida opcionales; también puede
llevar pesos de huesos. Véase `path`; PROJECT_FORMAT sección 6.

**Límites de ángulo.** Mínimo y máximo por hueso de la rotación animada, en grados, aplicados después de los
canales y después de cada cadena de IK. Véase `rig`, `engine`.

**Versión de la API.** La versión de la superficie estable de Rust (`api` y `prelude`), actualmente 1.4.0,
comprobada mediante una prueba de instantánea. No es lo mismo que el `format` del archivo ni que el `ABI_VERSION` de C. Véase `api`.

**Recurso (asset).** Un archivo al que hace referencia un proyecto (imagen, mapa de normales, sonido). Las rutas deben ser relativas y permanecer
dentro del directorio del proyecto salvo que se indique `--allow-external-assets`. Véase `assets`; PROJECT_FORMAT
sección 18.

**Bake.** Calcular valores a lo largo del tiempo y almacenarlos como canales o claves ordinarios, de modo que ya
no se necesite la herramienta que los calculó. Se usa en física, clips, magnet, footlock, máquinas de estados y más. Véase
ARCHITECTURE sección 4.

**Flexión (bend).** El signo (+1 o -1) que elige hacia qué lado se pliega una cadena de IK de dos huesos, por ejemplo codo hacia dentro o hacia fuera.
Véase `rig`.

**Curva de Bézier.** Una curva cúbica paramétrica definida por dos anclas y sus manejadores. Se aplana en segmentos
de recta (subdivisión de de Casteljau) para rellenar y medir. Véase `path`.

**Enlace (binding).** Una regla de runtime que asigna un valor externo (una variable de juego, un deslizador) mediante
una expresión, un remapeo y un suavizado a una propiedad o a una entrada de una máquina de estados. Véase `binding`.

**Modo de fusión.** Cómo se combina el color de una capa con lo que hay debajo (multiplicar, trama, superponer
y demás). Las fórmulas siguen W3C Compositing and Blending Level 1. Véase `raster`, `fraster`.

**Blend tree.** Una mezcla 1-D de varios clips controlada por una entrada numérica, usada dentro de una máquina
de estados. Véase `machine`.

**Blob.** Un círculo de una capa líquida (metaball). Véase `fx`.

**Hueso.** Un segmento con nombre del esqueleto con una posición de cabeza, longitud, ángulo de reposo y
padre opcional. Los huesos controlan la geometría con skinning y las capas asociadas. Véase `rig`; PROJECT_FORMAT sección 4.

**Capa booleana.** Una capa que combina trazados vectoriales mediante unión, intersección, diferencia o xor. El
primer trazado es el sujeto y los demás se unen como el recorte. Véase `fx`.

**Rebote (suavizado).** Una curva de suavizado de una pelota que cae y rebota con restitución 0,6 y tres
rebotes. Una derivación original a partir de la caída libre. Véase `anim`.

**Caja delimitadora.** El rectángulo alineado con los ejes más pequeño que rodea una geometría; se usa para alinear, seleccionar y
ajustar mapas de normales. Véase `layout`, `pick`.

**Deformador cage.** Un deformador en el que el contenido dentro de un polígono cerrado sigue los vértices del polígono
mediante coordenadas de valor medio. Véase `cage`; véase también MVC.

**Spline de Catmull-Rom.** Un spline cúbico que pasa por sus puntos de control. El suavizado `smooth` usa
una versión no uniforme a través de las claves vecinas. Véase `anim`, el suavizado de `liquid` en `fx`.

**Canal.** Un valor animado: un objetivo, una propiedad, una fuente (claves, expresión, ruido o dirigido) y
un modo de fusión y un peso. Los canales se ejecutan en el orden de la lista. Véase PROJECT_FORMAT sección 8.

**CIE76.** La diferencia de color original, la distancia euclídea en CIE L*a*b*. Barata, menos precisa
que CIEDE2000. Véase `percep`.

**CIEDE2000.** La fórmula de diferencia de color CIE 2000 (CIE TR 142-2001), implementada tal como se imprime en
Sharma, Wu y Dalal (2005) y comprobada con sus pares de prueba publicados. La usa la CI de animación para
juzgar el cambio visible. Véase `percep`, `ci`; `docs/CI.md`.

**Clean room.** La regla de desarrollo según la cual el código se escribe a partir de estándares públicos y métodos de manual,
sin copiar código y sin reproducir los formatos o nombres de otras herramientas. Cada técnica se registra en
`CLEANROOM.md`.

**Clip.** Un take reutilizable de pistas de animación que puede colocarse en la línea de tiempo con velocidad, bucles,
peso, fundidos y un modo de fusión. Véase `clipstack`; PROJECT_FORMAT sección 11.1.

**Tela (cloth).** Un deformador de cuadrícula cuyos vértices se mueven exactamente según sus propios desplazamientos. También es la simulación
de dinámica basada en posiciones que hornea esos desplazamientos. Véase `cloth`.

**Restricción.** Una regla aplicada después de la IK que limita un hueso: aim, copy rotation o pin. Véase `rig`.

**Contenedor (maquetación).** Un grupo de maquetación unidimensional al estilo flexbox con separación, relleno, justificación,
alineación y ajuste de línea, que posiciona capas y otros contenedores. Véase `containers`.

**Restricción de copy rotation.** Una restricción que hace que un hueso iguale la rotación en el mundo de otro hueso. Véase
`rig`.

**CRDT.** Tipo de datos replicado libre de conflictos: una estructura que las réplicas pueden editar de forma independiente y
fusionar con el mismo resultado sin coordinación. OSATURA usa un CRDT basado en operaciones. Véase `collab`;
ARCHITECTURE sección 10. Relacionados: registro LWW, RGA, reloj de Lamport, reloj vectorial, tombstone.

**Suavizado Bézier cúbico.** Una curva de suavizado al estilo CSS definida por dos puntos de control entre (0,0) y
(1,1). Véase `anim`.

**Deformador.** Un modificador de forma aplicado a una capa vectorial o de imagen en el espacio de reposo: lattice, spine, pin,
cage o cloth. Véase PROJECT_FORMAT sección 7.1.

**Triangulación de Delaunay.** Una triangulación de puntos en la que ningún punto queda dentro del círculo circunscrito de ningún triángulo,
lo que evita triángulos delgados. Se construye con el algoritmo de Bowyer-Watson para mallas de imagen.
Véase `meshtrace`.

**Profundidad de campo (DoF).** Desenfoque que aumenta con la distancia del `parallax` de una capa al `focus` de la cámara.
Lo aplican únicamente los comandos `dof` y `dof-export`. Véase `dof`.

**Determinismo.** La propiedad de que la misma entrada produce la misma salida. La evaluación de un fotograma es pura;
el renderizado es idéntico byte a byte en una misma máquina y compilación, sin verificar entre plataformas. Véase
ARCHITECTURE sección 6.

**Dope sheet.** Una vista de la línea de tiempo que muestra las claves como marcas por canal y las edita en el tiempo. Véase
`timeline`, `keyops`.

**Clave dirigida (driven key).** Un canal cuya entrada es el valor de otra propiedad en lugar del tiempo, por ejemplo del ángulo del codo
al tamaño de un abultamiento. Véase `drivers`; PROJECT_FORMAT sección 8.1.

**Suavizado (easing).** La función de modelado aplicada al segmento que empieza en una clave. Véase `anim`; PROJECT_FORMAT
sección 8.4.

**Editor.** El objeto que posee un proyecto y su historial de deshacer y aplica las ops. Véase `edit`;
ARCHITECTURE sección 5.

**Emisor.** Una capa que produce partículas como función de forma cerrada del tiempo. Véase `fx`.

**Motor (engine).** La forma validada y con nombres resueltos de un proyecto, que evalúa fotogramas. Véase `engine`.

**Sobre (collab).** El mensaje que transporta ops entre réplicas: id de réplica, número de secuencia,
tiempo de Lamport, dependencias y operaciones. Véase `collab`.

**Evaluación.** Calcular la pose, la geometría y la cámara de un fotograma a partir de un proyecto. Una función pura
del fotograma. Véase PROJECT_FORMAT sección 16.

**FABRIK.** Forward And Backward Reaching Inverse Kinematics: un solver de IK iterativo que tira alternativamente de la cadena hacia el objetivo desde la punta y luego la vuelve a anclar en la raíz. Se usa para cadenas de
más de dos huesos. Véase `rig`.

**FK.** Cinemática directa: calcular las transformaciones en el mundo de los huesos a partir de las transformaciones de sus padres y
de sus propios ángulos y desplazamientos. Véase `rig`, `engine`.

**Versión del formato.** El campo entero `format` de un proyecto, actualmente 1. Los archivos más nuevos se rechazan.
Véase PROJECT_FORMAT sección 19.

**Fotograma (frame).** (1) Un paso de tiempo de la línea de tiempo, `0..frames-1`. (2) La estructura `Frame`: el resultado de
evaluar un paso (huesos, geometría, cámara, indicadores). Véase `engine`.

**Glue.** Un vínculo que hace que vértices de una malla sigan vértices de otra (ajuste, rígido o con desplazamiento). Véase
`links`.

**Manejador de degradado.** Un punto de control de un degradado lineal o radial que puede animarse con
`grad_dx` y `grad_dy`. Véase PROJECT_FORMAT sección 8.2.

**Capa de grupo.** Una capa que contiene otras capas y les transmite su transformación. Véase `project`.

**Manejador.** Un punto de control de Bézier unido a un ancla. Coordenadas absolutas en el archivo. Véase `path`.

**HOG.** Histograma de gradientes orientados, una característica de forma local (Dalal y Triggs 2005) usada por el detector
de rostros. Véase `facedet`.

**IK.** Cinemática inversa: calcular los ángulos de los huesos para que la punta de una cadena alcance un objetivo. Véase `rig`.
Relacionados: FABRIK, IK suave, IK con estiramiento, flexión.

**Mezcla de IK.** La mezcla entre la pose de FK (0) y el resultado de IK (1) para una cadena. Se anima con `ik_mix`.
Véase PROJECT_FORMAT sección 4.2.

**Malla de imagen.** Una malla de cuadrícula o de triángulos libres sobre una capa de imagen, usada para pesos de huesos y deformadores.
Véase PROJECT_FORMAT sección 5.8.

**Fotograma clave (clave).** Una terna (fotograma, valor, suavizado) en una pista. El suavizado modela el segmento que empieza
en la clave. Véase `anim`.

**L*a*b*.** Espacio de color CIE 1976 diseñado para que las distancias aproximen las diferencias percibidas. Véase
`percep`.

**Reloj de Lamport.** Un contador lógico que ordena eventos en un sistema distribuido sin un reloj compartido
(Lamport 1978). Véase `collab`.

**Lattice.** Un deformador con una cuadrícula de puntos de control sobre un rectángulo cuyos desplazamientos doblan el contenido
mediante mezcla de Bézier-Bernstein. Véase PROJECT_FORMAT sección 7.1.

**Capa.** Un nodo del árbol de la escena: vectorial, grupo, conmutación, líquida, booleana, emisor o imagen. Véase
PROJECT_FORMAT sección 5.

**Efecto de capa.** Un filtro por capa (desenfoque, sombra proyectada, tinte, relleno) renderizado en un lienzo fuera de pantalla.
Véase `layerfx`.

**Nivel de detalle (LOD).** Una versión simplificada de un proyecto producida con una cota de error declarada en
píxeles, para una vista previa rápida o una salida pequeña. Véase `lod`, `osatura lod`.

**Linter.** El comprobador consultivo (`osatura lint`) que informa de problemas de calidad de la animación con códigos
como L011 y L012. Distinto de la validación. Véase `lint`.

**Capa líquida.** Una capa de metaballs cuyo contorno es el isocontorno de un campo de suma de influencias. Véase
`fx`.

**Registro LWW.** Registro de último escritor gana: una celda de CRDT en la que sobrevive la escritura con la marca de tiempo lógica más alta. Se usa para las claves de objeto. Véase `collab`.

**Magnet.** Una herramienta de bake que atrae posiciones hacia un punto con caída smoothstep. Véase `magnet`.

**Marching squares.** Un algoritmo que extrae líneas de contorno de un campo escalar muestreado en una cuadrícula.
Se usa para contornos líquidos y trazado de mapas de bits. Véase `fx`, `trace`, `meshtrace`.

**Marcador.** Un fotograma o una región de fotogramas con nombre en la línea de tiempo. Las máquinas de estados pueden usar una región de marcador como
fuente. Véase `keyops`; PROJECT_FORMAT sección 10.2.

**Máscara.** Una capa cuyo alfa limita la visibilidad de otra capa; puede invertirse. Una capa oculta
puede seguir siendo una máscara. Véase `raster`.

**Merge (CRDT).** Combinar los estados de dos réplicas para que ambas converjan en el mismo documento. Véase
`collab`.

**Bloqueo de malla.** Un vínculo que hace que una malla seguidora siga a una malla anfitriona con una intensidad 0..1. Véase `links`.

**Metaball.** Un blob cuya influencia de campo se mezcla con la de sus vecinos de modo que los blobs cercanos se fusionan. Véase capa
líquida.

**Morph (blend shape).** Un conjunto alternativo de anclas para un trazado, aplicado de forma aditiva con un peso. Véase
PROJECT_FORMAT sección 6.1.

**MVC.** Coordenadas de valor medio (Floater 2003): coordenadas baricéntricas generalizadas para un punto dentro de
un polígono, usadas por el deformador cage. Véase `cage`.

**No destructivo.** Que deja intacto el trabajo existente. Los bakers añaden canales en lugar de reescribir claves, y
el editor registra ops inversas. Véase ARCHITECTURE secciones 4 y 5.

**Mapa de normales.** Una imagen que almacena la dirección de la superficie como RGB, usada por el efecto `relight` y por la
iluminación por capa. Véase `postfx`; PROJECT_FORMAT sección 7.3.

**Op.** Una edición expresada como `set`, `insert`, `remove` o `batch` sobre un puntero JSON RFC 6901; tiene una
inversa para deshacer. Véase `edit`.

**OT.** Transformación operacional: una alternativa a los CRDT que reescribe operaciones concurrentes unas contra otras,
normalmente con un servidor central. OSATURA no la usa; usa un CRDT con un secuenciador central. Véase ARCHITECTURE
sección 10.

**Parallax.** Un factor por capa que escala el movimiento de la cámara, de modo que las capas lejanas se mueven menos. También es el
valor de profundidad usado para la profundidad de campo. Véase PROJECT_FORMAT sección 10.1.

**Partícula.** Un punto de vida corta emitido por un emisor; su posición es una función de forma cerrada del
tiempo. Véase `fx`.

**Ruta (objetivo).** El nombre separado por barras de una capa anidada, como `parent/child`; los subobjetivos usan
`::`, como en `layer::path::morph`. Véase PROJECT_FORMAT sección 5.

**Trazado (vectorial).** Una secuencia de anclas, abierta o cerrada, con relleno, contorno y morphs opcionales. Véase
`path`.

**Restricción pin.** Una restricción que desliza la cabeza de un hueso hasta la cabeza de otro hueso. Véase `rig`.

**Pin warp.** Un deformador en el que los pins arrastrados mueven el arte suavemente dentro de un radio, usando una
función de base radial de soporte compacto (Wendland 1995). Véase `pinwarp`.

**Reproductor (player).** El envoltorio de runtime de la fachada que añade máquinas de estados y enlaces a una animación.
Véase `api`.

**Preorden.** El orden de visita del árbol de capas: un nodo y luego sus hijos. La evaluación lo usa para que
los padres se calculen primero.

**Proxy.** Una imagen de detalle reducido usada durante el scrub, construida a partir de un nivel de detalle. Véase `scrub`, `lod`.

**QEM.** Métrica de error cuádrico (Garland y Heckbert): una medida del error geométrico causado por
colapsar un vértice, usada para simplificar mallas. Véase `lod`.

**Raster.** Una imagen de píxeles, o el módulo que renderiza un fotograma a píxeles con tiny-skia. Véase `raster`.

**RDP.** Ramer-Douglas-Peucker: simplificación recursiva de líneas que conserva los puntos que distan más de una tolerancia
de la cuerda. Véase `trace`, `sizefit`.

**Capa de referencia.** Una capa marcada como `reference`; oculta salvo que el editor muestre las referencias. Véase
PROJECT_FORMAT sección 5.1.

**Remapeo.** Una transformación afín de un rango, con una curva y un límite opcionales, aplicada al valor de un enlace.
Véase `binding`.

**Réplica (collab).** La copia del documento de un participante en el CRDT. Véase `collab`.

**Pose de reposo.** Las transformaciones de los huesos sin animación aplicada; las matrices de skin son `world * inverse(rest
world)`. Véase `rig`.

**RGA.** Array replicado ampliable (Roh et al. 2011): un CRDT de secuencias en el que cada elemento tiene un id único
y los borrados dejan tombstones. Se usa para los arrays JSON. Véase `collab`.

**Rig.** El esqueleto y sus auxiliares: huesos, cadenas de IK, restricciones y muelles. Véase PROJECT_FORMAT
sección 4.

**Scrub.** Arrastrar el cabezal de reproducción; el estudio guarda en caché los fotogramas por revisión y usa proxies. Véase `scrub`.

**Skew.** Un ángulo de cizallamiento en la transformación de una capa. Véase PROJECT_FORMAT sección 5.1.

**Skin (animación).** Una variante con nombre de capas, huesos, restricciones o IK que puede activarse en un
grupo, como los vestuarios. Véase `skins`.

**Skinning.** Deformar la geometría mezclando las transformaciones de los huesos, ponderadas por ancla o vértice.
OSATURA normaliza los pesos y trata los pesos vacíos como estáticos. Véase `engine`, `weights`.

**IK suave.** Una opción de IK que suaviza la extensión cerca del alcance máximo para evitar un chasquido hasta quedar recta.
Rango de 0 a 0,9. Véase `rig`.

**SPDX.** Software Package Data Exchange: una forma estándar de declarar la licencia de un archivo, como en
<!-- REUSE-IgnoreStart -->
`SPDX-License-Identifier: MPL-2.0`. Todo archivo fuente lleva una. Véase `LICENSING.md`.
<!-- REUSE-IgnoreEnd -->

**Deformador spine.** Un deformador que dobla la geometría a lo largo de una curva proyectando las anclas sobre una curva
de reposo y volviéndolas a unir a la curva deformada. Véase PROJECT_FORMAT sección 7.1.

**Muelle (hueso).** Un muelle amortiguado de segundo orden que hace que la rotación de un hueso se retrase y se pase; se hornea al
cargar para que pueda evaluarse cualquier fotograma. Véase PROJECT_FORMAT sección 4.4.

**SSIM.** Índice de similitud estructural (Wang et al. 2004): una medida de calidad de imagen a partir de medias locales,
varianzas y covarianza. OSATURA usa una versión por bloques. Véase `percep`.

**Escenario (stage).** El espacio de coordenadas de la imagen de salida: píxeles, +x a la derecha, +y hacia abajo.

**Máquina de estados.** Un grafo de estados de animación con transiciones controladas por entradas. Hay una versión horneada,
con registro de eventos, y una versión de runtime. Véase `statemachine`, `machine`.

**Suavizado escalón (step).** Un suavizado que mantiene el valor de la clave hasta la siguiente clave. Se usa para capas de conmutación. Véase
`anim`.

**IK con estiramiento.** Una opción de IK que escala la cadena hasta un límite (1..4) para alcanzar un objetivo lejano. Véase
`rig`.

**Contorno (stroke).** El perfil de un trazado: pintura, ancho, remate, unión, guiones y un perfil de ancho opcional. Véase
PROJECT_FORMAT sección 6.3.

**Capa de conmutación (switch).** Una capa que muestra exactamente uno de sus hijos, elegido por un canal de índice. Se usa para
la animación fotograma a fotograma y las formas de la boca. Véase PROJECT_FORMAT sección 5.4.

**Take.** Un conjunto con nombre de pistas que forma el contenido de un clip. Véase `clipstack`.

**Tombstone.** Una marca que queda en un array CRDT en lugar de un elemento eliminado para que las operaciones concurrentes
aún puedan ubicarse. La compactación elimina los tombstones que todas las réplicas han visto. Véase `collab`.

**Pista (track).** La lista de claves de un canal, con un modo de extrapolación. Véase `anim`.

**Transformación.** Posición, rotación, escala, skew y ancla de una capa; la matriz local es
`T(position) R Shear S T(-anchor)`. Véase PROJECT_FORMAT sección 5.1.

**Reloj vectorial.** Un vector de contadores por réplica que indica si un evento ocurrió antes que otro o
si son concurrentes (Fidge, Mattern). Véase `collab`.

**Pesos (hueso).** La influencia de cada hueso sobre un ancla o un vértice. Pueden pintarse o calcularse
por difusión. Véase `weights`.

**Wobble.** Un desplazamiento de ruido determinista añadido a la geometría de una capa, con amplitud, escala, frecuencia
y seed. Véase PROJECT_FORMAT sección 5.1.

**Adelgazamiento de Zhang-Suen.** Un algoritmo paralelo que reduce una forma binaria a un esqueleto de un píxel. Lo usa
autotrace para el trazado de línea central. Véase `autotrace`.
