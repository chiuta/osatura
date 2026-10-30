<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Formato de projeto do OSATURA

Esta é a referência do arquivo de projeto do OSATURA: um único documento JSON que
contém o palco, o rig, a arte e a animação. Ela foi escrita a partir das structs serde em
`osatura-core/src/project.rs` e dos módulos que elas referenciam, e todos os exemplos das seções 13 a 15 foram verificados com o binário `osatura`
(`osatura check`, `osatura render`, `osatura explain`).

Índice

1. [Convenções e tipos primitivos](#1-convenções-e-tipos-primitivos)
2. [Carregamento, versionamento e campos desconhecidos](#2-carregamento-versionamento-e-campos-desconhecidos)
3. [Campos de nível superior](#3-campos-de-nível-superior)
4. [Rig: ossos, ik, restrições, molas](#4-rig-ossos-ik-restrições-molas)
5. [Camadas](#5-camadas)
6. [Caminhos, âncoras, paint, stroke](#6-caminhos-âncoras-paint-stroke)
7. [Deformadores, efeitos, normal map](#7-deformadores-efeitos-normal-map)
8. [Canais, trilhas, easing](#8-canais-trilhas-easing)
9. [Expressões](#9-expressões)
10. [Câmera, planos, áudio, metadados](#10-câmera-shots-áudio-metadados)
11. [Clips, skins, links, interação, lods](#11-clipes-skins-links-interação-lods)
12. [Regras de validação, limites e mensagens de erro](#12-regras-de-validação-limites-e-mensagens-de-erro)
13. [Exemplo 1: bola quicando](#13-exemplo-1-bola-quicando)
14. [Exemplo 2: braço com rig e IK](#14-exemplo-2-braço-com-rig-e-ik)
15. [Exemplo 3: boca com camada switch](#15-exemplo-3-boca-com-camada-switch)
16. [Ordem de avaliação](#16-ordem-de-avaliação)
17. [Ordem de renderização](#17-ordem-de-renderização)
18. [Assets e caminhos](#18-assets-e-caminhos)
19. [Política de versionamento](#19-política-de-versionamento)
20. [Lacunas conhecidas entre código e comentários](#20-lacunas-conhecidas-entre-código-e-comentários)

## 1. Convenções e tipos primitivos

| Tipo | Forma em JSON | Observações |
| --- | --- | --- |
| number | número JSON | Armazenado como `f32`, a menos que o campo diga que é inteiro. Todo float deve satisfazer `abs(x) <= 1e7` (seção 12). |
| `Vec2` | `[x, y]` | Dois números. As unidades do palco são pixels, +x para a direita, +y para baixo. |
| `Rgba` | string | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, ou um de `black`, `white`, `red`, `green` (0,128,0), `blue`, `yellow`, `transparent`. A saída é sempre em hexadecimal minúsculo. |
| `Paint` | string ou objeto | Uma string de cor é um paint sólido. `{"from": [x,y], "to": [x,y], "stops": [...]}` é um gradiente linear. `{"center": [x,y], "radius": r, "stops": [...]}` é um gradiente radial. O tipo é um enum sem tag (untagged). |
| `Stop` | `{"offset": 0..1, "color": Rgba}` | Os stops são ordenados (de forma estável) no momento do desenho e os offsets são limitados a 0..1 no rasterizador. Um gradiente degenerado pinta o último stop. |
| angle | número | Graus em todo o arquivo (ângulos de ossos, rotação de camada, skew, rotação da câmera, `angle_limits`). |
| time | número | Quadros em chaves e marcadores, segundos em sons, máquinas e emissores. Cada campo diz qual. |
| names | string | Nomes de osso, camada, caminho, deformador, ik e restrição. As seções 5 e 12 listam as restrições de caracteres. |

Uma cor inválida em um contexto de paint produz a mensagem do serde
`data did not match any variant of untagged enum Paint`, porque `Paint` não tem tag e o serde
não consegue dizer qual ramo era o pretendido.

As tabelas de campos abaixo usam esta notação: **Obrig.** significa que o campo não tem valor padrão e deve estar presente;
caso contrário, o valor padrão é mostrado. "Faixa" é o que o carregador ou o engine impõe. Uma faixa escrita
"limitado" (clamped) não é um erro: o engine limita o valor silenciosamente.

## 2. Carregamento, versionamento e campos desconhecidos

`Project::from_json(text)` é o ponto de entrada único usado pela CLI, pela API C, pelo módulo WASM
e pelo Studio. Ele faz, em ordem:

1. Analisa o JSON com o serde. O serde ignora os campos que não conhece.
2. Executa a verificação de magnitude: qualquer literal float cujo valor absoluto seja maior que `1e7` faz o carregamento
   falhar com `invalid JSON in <file>: value ... is outside the supported range of +-1e7 ...`. Os literais inteiros são detectados
   mais tarde, pela nova verificação dentro de `Engine::new` (veja a seção 12).
3. Aplica a barreira de formato: `format` maior que o `FORMAT_VERSION` do build (atualmente `1`) é
   rejeitado com `project format 2 is newer than this build supports (1); update OSATURA`.

`Engine::new` (ou `Engine::with_asset_dir`) então valida o projeto inteiro e devolve todos os
problemas que encontra em uma lista. A CLI os imprime assim:

```text
<file> is not a valid project:
  - first problem
  - second problem
```

Os textos de erro são para humanos. Eles não são um contrato estável; compare o texto apenas em testes.

### Campos desconhecidos

O serde descarta chaves desconhecidas no carregamento, de modo que um erro de digitação como `fil` em vez de `fill` seria perdido em silêncio. O
módulo `unknownfields` compara o JSON de entrada com o projeto reserializado e informa as chaves que
não sobreviveram:

```text
warning: unknown field layers[0].paths[0].fil (ignored)
```

`osatura check` imprime esses avisos. Valores iguais ao padrão não são informados (o gravador
omite os padrões, então um `"visible": true` explícito não causa problema). Algumas estruturas são estritas em vez de
tolerantes e rejeitam chaves desconhecidas com um erro grave: `Transition` e `Condition` de máquina, configuração de CI,
configuração de entrada ao vivo, overlays e configuração de parâmetros de rosto; os arquivos de configuração são lidos com `parse_strict`.

### Ida e volta dos metadados do editor

O projeto é serializado com `skip_serializing_if` para listas opcionais. Um arquivo que você escreve à mão pode
ser mais curto do que o que o `osatura` grava de volta; os dois são equivalentes.

## 3. Campos de nível superior

| Nome em JSON | Tipo | Padrão | Faixa / regra | Significado |
| --- | --- | --- | --- | --- |
| `format` | inteiro | `FORMAT_VERSION` (1) | deve ser `<=` à versão do build | Versão do formato, seção 19. |
| `name` | string | **Obrig.** | qualquer | Nome de exibição. Impresso por `check`. |
| `width` | número | **Obrig.** | `> 0`, `<= 32768` | Largura do palco em pixels. |
| `height` | número | **Obrig.** | `> 0`, `<= 32768` | Altura do palco em pixels. |
| `fps` | número | **Obrig.** | `(0, 1000]` | Quadros por segundo. |
| `frames` | inteiro (`u32`) | **Obrig.** | `1..=200000` | Duração da timeline. Os índices de quadro são `0..frames-1`. Um literal float como `48.0` é rejeitado: `invalid type: floating point `48.0`, expected u32`. |
| `background` | `Rgba` ou null | null (transparente) | | Cor do palco por trás de tudo. |
| `color_space` | string | `"srgb"` (não gravado) | `srgb`, `linear` | Espaço em que os compositores raster e float fazem a mesclagem. `linear`: as cores são convertidas para luz linear, compostas em f32 e convertidas de volta uma única vez (um preto a 50 % sobre branco dá 188/255, e não 128/255). Defina com `osatura color-space`. SVG, Lottie, o renderizador de GPU, `dof` e `fx` o ignoram. |
| `camera` | objeto | padrões | seção 10 | Câmera global. |
| `bones` | array | `[]` | no máximo 4096 | Esqueleto, seção 4. |
| `ik` | array | `[]` | | Cadeias de IK. |
| `constraints` | array | `[]` | | Aim, copy-rotation, pin. |
| `springs` | array | `[]` | | Movimento secundário, gravado (baked) no carregamento. |
| `layers` | array | `[]` | | Camadas raiz, desenhadas na ordem da lista. |
| `channels` | array | `[]` | | Canais de animação, avaliados na ordem da lista. |
| `markers`, `comps`, `events`, `bone_vis`, `channel_meta`, `shots` | arrays | omitidos quando vazios | não validados | Metadados de timeline e do editor, seção 10. |
| `sounds` | array | omitido quando vazio | validado | Trilhas de áudio, seção 10. |
| `skins`, `clips`, `clip_baked`, `lods` | arrays | omitidos quando vazios | não validados no carregamento | Seção 11. |
| `mesh_locks`, `glues` | arrays | omitidos quando vazios | validados | Vínculos de malha entre camadas, seção 11. |
| `machines`, `bindings`, `containers`, `scrolls` | arrays | omitidos quando vazios | validados | Interação e layout, seção 11. |

Projeto válido mínimo:

```json
{"name": "empty", "width": 100, "height": 100, "fps": 24, "frames": 1}
```

Magnitudes: todo float no arquivo deve satisfazer `abs(x) <= 1e7`. Isso vale para coordenadas,
escalas, offsets e valores de chave igualmente. Largura e altura são limitadas ainda a 32768, e a saída
raster a 16384 por lado e 100 megapixels (seção 18).

## 4. Rig: ossos, ik, restrições, molas

### 4.1 `bones[]`

Os ossos devem ser listados começando pelos pais. O pai de um osso deve aparecer antes na lista.

| Nome em JSON | Tipo | Padrão | Faixa | Significado |
| --- | --- | --- | --- | --- |
| `name` | string | **Obrig.** | único | Referenciado por pesos, ik, canais, `Layer.bone`. |
| `parent` | string ou null | null | um osso anterior | Osso pai. Null significa raiz. |
| `length` | número | **Obrig.** | finito, `>= 0` | Comprimento do osso em pixels, ao longo do eixo +x do osso. |
| `rest_offset` | `Vec2` | `[0,0]` | | Posição da cabeça no espaço do pai (espaço do palco para raízes). |
| `rest_angle` | número | 0 | | Rotação de repouso em graus relativa ao pai. |
| `angle_limits` | `[min, max]` ou null | null | `min <= max` | Limite sobre o delta de ângulo animado, em graus. |
| `scale_limits` | `[min, max]` ou null | null | `0 < min <= max` | Limite sobre a escala animada. |

Interações: os canais `bone_angle` somam-se a `rest_angle`; `bone_offset_x/y` somam-se a `rest_offset`;
`bone_scale` multiplica. `clamp_limits` roda depois de todos os canais e antes do FK, e de novo depois de cada
cadeia de IK.

```json
{"name": "upper", "parent": null, "length": 90, "rest_offset": [60, 120],
 "rest_angle": 0, "angle_limits": [-120, 120]}
```

### 4.2 `ik[]`

| Nome em JSON | Tipo | Padrão | Faixa | Significado |
| --- | --- | --- | --- | --- |
| `name` | string | **Obrig.** | único | Alvo dos canais `ik_mix`. |
| `bones` | array de strings | **Obrig.** | pelo menos 2; cada um filho direto do anterior | Cadeia da raiz até a ponta. |
| `target` | string | **Obrig.** | um osso, fora da cadeia | Osso cuja cabeça a ponta tenta alcançar. |
| `bend` | número | 1 | +1 ou -1 | Para que lado uma cadeia de dois ossos dobra. |
| `mix` | número | 1 | limitado a 0..1 | Mistura inicial de FK para IK; anime com `ik_mix`. `<= 0` ignora a cadeia. |
| `iterations` | inteiro | 16 | | Passadas do FABRIK para cadeias com mais de dois ossos. Este é um piso: o solver sai antes quando a ponta fica a menos de 0.01 px e, caso contrário, continua até 256 passadas. |
| `soft` | número | 0 | `0..0.9` | Fração do alcance em que a extensão é suavizada. Com `stretch > 1`, a zona é a última fração do alcance esticado (`reach x stretch`). |
| `stretch` | número | 1 | `1..4` | Fator máximo de esticamento; acima de 1, a escala do osso raiz cresce para alcançar. |

Uma cadeia de dois ossos usa o solver analítico da lei dos cossenos; cadeias mais longas usam o FABRIK. Depois da resolução,
os ângulos são misturados à pose como `old + wrap_pi(new - old) * mix`.

```json
{"name": "reach", "bones": ["upper", "lower"], "target": "target", "bend": 1, "mix": 1}
```

### 4.3 `constraints[]`

Os tipos de restrição usam uma tag interna `"type"`, achatada ao lado de `name`, `bone` e `weight`.

| Nome em JSON | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `name` | string | **Obrig.** | Alvo dos canais `constraint_weight`. |
| `bone` | string | **Obrig.** | O osso restringido. |
| `type` | `aim`, `copy_rotation`, `pin` | **Obrig.** | Tipo. |
| `target` | string | **Obrig.** para `aim`, `pin` | Osso para o qual mirar ou sobre o qual deslizar. |
| `offset` | número | 0 | Somente `aim`: graus extras. |
| `source` | string | **Obrig.** para `copy_rotation` | Osso cuja rotação de mundo é copiada. |
| `weight` | número | 1 | Influência, limitada a 0..1. `<= 0` ignora a restrição. |

Uma restrição não pode ter a si mesma como alvo. As restrições rodam na ordem da lista, depois de todas as cadeias de IK.

```json
{"name": "aim", "bone": "root", "type": "aim", "target": "look", "offset": 0, "weight": 0.5}
```

### 4.4 `springs[]`

Uma mola amortecida que faz a rotação de mundo de um osso ficar atrás da sua rotação animada. Ela é
simulada uma vez no carregamento e gravada (baked), de modo que qualquer quadro pode ser avaliado em qualquer ordem.

| Nome em JSON | Tipo | Padrão | Faixa | Significado |
| --- | --- | --- | --- | --- |
| `bone` | string | **Obrig.** | uma mola por osso | Osso a controlar. |
| `frequency` | número | 3 | `(0, 30]` Hz | Rigidez. |
| `damping` | número | 0.35 | `>= 0` | Razão de amortecimento; abaixo de 1 oscila, 1 é crítico. |
| `warmup` | bool | true | | Faz uma passagem prévia para que os loops comecem em regime estacionário. |

Os ângulos gravados substituem o ângulo do osso depois das restrições (passo 8 da avaliação).

## 5. Camadas

Uma camada é `{"name": ..., "type": ..., <campos do tipo>, <campos comuns>}`. `type` é a tag de
`LayerKind`; seus campos são achatados no objeto da camada.

### 5.1 Campos comuns

| Nome em JSON | Tipo | Padrão | Faixa | Significado |
| --- | --- | --- | --- | --- |
| `name` | string | **Obrig.** | não vazio, sem `/`, sem `::`; o caminho completo deve ser único | Nome da camada. Os filhos são endereçados como `parent/child`. |
| `type` | string | **Obrig.** | `vector`, `group`, `switch`, `liquid`, `boolean`, `emitter`, `image` | Tipo. |
| `transform` | objeto | identidade | | `position` `[0,0]`, `rotation` 0, `scale` `[1,1]`, `skew` 0, `anchor` `[0,0]`. |
| `opacity` | número | 1 | limitado a 0..1 | Multiplica-se na composição da camada. |
| `visible` | bool | true | | As camadas ocultas são ignoradas ao desenhar, mas ainda podem ser fontes de máscara. |
| `blend` | string | `normal` | veja abaixo | Modo de mesclagem. |
| `mask` | `{layer, invert}` | null | sem auto-máscara, sem ciclos | Usa o alfa de outra camada como máscara. |
| `bone` | string | null | um osso conhecido | Fixa a um osso: a matriz de skin do osso é aplicada entre a matriz de palco do pai e a transformação local. |
| `parallax` | número | 1 | | Fator de profundidade da câmera. Só vale o valor da camada de nível superior. |
| `deformers` | array | omitido quando vazio | somente camadas vector e image | Seção 7. |
| `wobble` | objeto | null | | `amp`, `scale` (0.03), `rate` (8), `seed` (0): deslocamento por noise determinístico. Anime `amp` com `layer_wobble`. |
| `reference` | bool | false | | Camada de referência: oculta, a menos que o editor mostre referências. |
| `locked` | bool | false | | Bloqueio do editor, veja abaixo. |
| `normal_map` | objeto | null | | Seção 7. |
| `effects` | array | omitido quando vazio | no máximo 16 | Seção 7. |

Matriz local: `T(position) * R(rotation) * Shear(skew) * S(scale) * T(-anchor)`.

Modos de mesclagem: `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`, `color_dodge`,
`color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `add` (plus-lighter do CSS).

`locked` bloqueia apenas operações do editor: seleção no canvas, gizmos, nudges, exclusões, edições da timeline e
edições de canais (por meio de `assert_unlocked`), para a camada e seus descendentes. A renderização e a
avaliação o ignoram, e as `Op`s simples (seção "Editor ops" em ARCHITECTURE.md) continuam sendo aplicadas.

Os campos comuns são gravados depois dos campos do tipo na saída, mas a ordem é irrelevante na entrada.

### 5.2 `vector`

| Campo | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `paths` | array de path | `[]` | Seção 6. |

### 5.3 `group`

| Campo | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `layers` | array de camadas | `[]` | Os filhos herdam a transformação do grupo. |

### 5.4 `switch`

| Campo | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `layers` | array de camadas | `[]` | Desenhos candidatos. |
| `selected` | inteiro | 0 | Índice inicial do filho. Anime com `layer_switch` (chaves em degrau). |

Exatamente um filho é mostrado. O valor é arredondado; negativo vira 0; valores iguais ou acima da contagem de filhos
viram o último filho. A validação não rejeita chaves fora da faixa. Veja a seção 20 para uma discrepância de
redação do lint sobre isso.

### 5.5 `liquid`

Líquido de metaballs. O campo é a soma de `r^2 / d^2` sobre os blobs; o contorno é a isolinha,
extraída por marching squares em uma grade limitada a 600 por 600 células e suavizada com Catmull-Rom.

| Campo | Tipo | Padrão | Faixa | Significado |
| --- | --- | --- | --- | --- |
| `blobs` | array de `{pos, radius, bone?}` | **Obrig.** | pelo menos 1; `radius > 0` | Um blob. `bone` o fixa a um osso. |
| `threshold` | número | 1 | `> 0` | Nível da isolinha; 1 faz um blob isolado ser um círculo do seu raio. |
| `cell` | número | 3 | `>= 0.25` | Célula da grade em unidades da camada. |
| `fill` | `Paint` | null | | |
| `stroke` | objeto stroke | null | | |

### 5.6 `boolean`

| Campo | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `op` | `union`, `intersect`, `difference`, `xor` | **Obrig.** | Operação. |
| `paths` | array de path | **Obrig.**, pelo menos 2 | `paths[0]` é o sujeito; o recorte é a união dos demais. |

O resultado assume o fill e o stroke do sujeito; a tolerância de achatamento é 0.25. Os caminhos resultantes são
nomeados `<nome do primeiro path><index>`.

### 5.7 `emitter`

`{"type": "emitter", "spec": {...}}`. As partículas são uma função de forma fechada do tempo, de modo que qualquer quadro
pode ser avaliado. Os tempos são em segundos.

| Campo de `spec` | Padrão | Faixa | Significado |
| --- | --- | --- | --- |
| `pos` | `[0,0]` | | Posição do emissor (espaço da camada). |
| `bone` | null | | Fixa a um osso. |
| `rate` | 20 | `(0, 2000]` | Partículas por segundo. |
| `lifetime` | 1 | `> 0` | Segundos. |
| `lifetime_jitter` | 0 | 0..1 | |
| `speed` | 60 | | Pixels por segundo. |
| `speed_jitter` | 0 | 0..1 | |
| `direction` | -90 | | Graus; -90 é para cima. |
| `spread` | 30 | | Graus de abertura do cone. |
| `gravity` | `[0,0]` | | Aceleração, pixels por segundo ao quadrado. |
| `size_start`, `size_end` | 3, 3 | | Raio ao longo da vida. |
| `color_start`, `color_end` | `#fff`, `#fff0` | | Cor ao longo da vida. |
| `seed` | 0 | | Seed determinística. |
| `start` | 0 | | Tempo de início em segundos. |
| `duration` | null | | A emissão para depois desse número de segundos; null é para sempre. |
| `max_particles` | 2000 | | Limite rígido de partículas vivas. |

### 5.8 `image`

| Campo | Tipo | Padrão | Faixa | Significado |
| --- | --- | --- | --- | --- |
| `source` | `{file?, png_base64?}` | **Obrig.** | um dos dois | `file` é um caminho relativo ao diretório do projeto (seção 18); `png_base64` é autocontido. Nenhum dos dois: `image has neither 'png_base64' nor 'file'`. |
| `origin` | `Vec2` | `[0,0]` | | Canto superior esquerdo no espaço da camada. |
| `size` | `Vec2` ou null | null (tamanho em pixels) | | Tamanho exibido. |
| `mesh` | objeto ou null | null | | Malha de deformação. |

`mesh`: `cols`, `rows` (malha em grade, cada um com pelo menos 1), `weights` (pesos de osso por vértice; vazio
significa estático) e, opcionalmente, `free` = `{vertices, triangles, uvs}`. Uma malha livre precisa de pelo menos 3
vértices, pelo menos 1 triângulo, no máximo 200000 vértices e 400000 triângulos, coordenadas finitas,
índices de triângulo válidos e `uvs` vazio ou com o mesmo comprimento de `vertices`. Com deformadores, mas
sem malha, usa-se uma grade de 16 por 16.

```json
{"name": "img2", "type": "image", "source": {"png_base64": "iVBOR..."},
 "origin": [100, 0], "size": [80, 80],
 "mesh": {"free": {"vertices": [[100,0],[180,0],[180,80],[100,80]],
                   "triangles": [[0,1,2],[0,2,3]],
                   "uvs": [[0,0],[8,0],[8,8],[0,8]]}, "weights": []}}
```

## 6. Caminhos, âncoras, paint, stroke

### 6.1 Path (`paths[]`)

| Nome em JSON | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `name` | string | `""` | Um nome vazio é substituído por `path{index}` para fins de endereçamento. Não deve conter `::`. Único dentro da camada. |
| `closed` | bool | false | |
| `anchors` | array | **Obrig.** | Âncoras Bézier. |
| `fill` | `Paint` | null | |
| `stroke` | stroke | null | |
| `morphs` | array de `{name, anchors}` | omitido quando vazio | Blend shapes aditivos. O número de âncoras deve ser igual ao do path base. Peso via `path_morph`. |

### 6.2 Âncora

| Nome em JSON | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `pos` | `Vec2` | **Obrig.** | Posição. |
| `handle_in`, `handle_out` | `Vec2` | null | Pontos de controle no mesmo espaço de `pos` (absolutos, não relativos). Uma alça ausente é igual a `pos` (um canto). |
| `weights` | array de `{bone, weight}` | omitido quando vazio | Skinning. Cada osso deve existir e cada peso é finito e `>= 0`. Os pesos são normalizados na avaliação; vazio significa estático. As alças também recebem skinning. |

### 6.3 Stroke

| Nome em JSON | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `paint` | `Paint` | **Obrig.** | |
| `width` | número | 1 | |
| `cap` | `butt`, `round`, `square` | `round` | |
| `join` | `miter`, `round`, `bevel` | `round` | Um valor desconhecido dá ``unknown variant `foo`, expected one of `miter`, `round`, `bevel` ``. |
| `dash` | array | omitido | Usado apenas se o comprimento for par, pelo menos 2, e cada entrada for `> 0`. |
| `dash_offset` | número | 0 | |
| `width_profile` | array de `[t, factor]` | omitido | Multiplicador de largura linear por partes ao longo do comprimento (`t` em 0..1), constante além das extremidades, limite de mitra 2. Um stroke de largura variável ignora `dash` e `join`. |

Exemplo de um fill com gradiente radial e um stroke (do exemplo da bola):

```json
"fill": {"center": [-8, -10], "radius": 44,
         "stops": [{"offset": 0, "color": "#ffd9a0"}, {"offset": 1, "color": "#d6336c"}]},
"stroke": {"paint": "#3d2c2e", "width": 2}
```

O linter (`osatura lint`) avisa sobre problemas de gradiente (L011), de chaves de switch (L012) e de dados do editor (L013); veja
`osatura-core/src/lint.rs`.

## 7. Deformadores, efeitos, normal map

### 7.1 `deformers[]`

Permitidos apenas em camadas `vector` e `image`. Cada um tem uma tag `"type"` e um `name` (único dentro
da camada, sem `::`). Eles rodam na ordem da lista sobre a geometria no espaço de repouso, antes do skinning e antes do
`wobble`. Os offsets são estáticos; anime-os com os canais de subalvo.

| `type` | Campos | Restrições | Animar com (alvo) |
| --- | --- | --- | --- |
| `lattice` | `cols`, `rows`, `rect` `[x,y,w,h]`, `offsets` | `cols, rows >= 1`; `w, h > 0`; `offsets` vazio ou `(cols+1)*(rows+1)` em ordem de linha | `lattice_dx/dy` (`layer::name::c,r`) |
| `spine` | `rest` (pontos da curva), `offsets`, `radius` (borda suave opcional) | pelo menos 2 pontos | `spine_dx/dy` (`layer::name::i`) |
| `pin` | `pins`, `offsets`, `radius` | 1 a 64 pinos, `radius > 0`, pinos não coincidentes | `pin_dx/dy` (`layer::name::i`) |
| `cage` | `rest` (polígono), `offsets` | 3 a 256 vértices, polígono simples, área diferente de zero | `cage_dx/dy` (`layer::name::i`) |
| `cloth` | como `lattice` | como `lattice` | `lattice_dx/dy` (`layer::name::c,r`) |

O lattice mistura os offsets com pesos de Bézier-Bernstein (suave); o cloth desloca cada vértice da grade
exatamente (bilinear). O pin usa uma função de base radial de suporte compacto de Wendland. O cage usa as
coordenadas de valor médio de Floater: o conteúdo dentro do polígono o segue, e o conteúdo fora dele não é afetado.

```json
"deformers": [
  {"type": "lattice", "name": "lat", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []},
  {"type": "spine", "name": "sp", "rest": [[0,0],[50,0],[100,0]], "offsets": [], "radius": 30},
  {"type": "pin", "name": "pn", "pins": [[0,0],[100,100]], "offsets": [], "radius": 80},
  {"type": "cage", "name": "cg", "rest": [[0,0],[100,0],[100,100],[0,100]], "offsets": []},
  {"type": "cloth", "name": "cl", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []}
]
```

### 7.2 `effects[]`

No máximo 16 por camada. Cada efeito tem `"type"`, `enabled` (padrão true) e os campos abaixo. O
alvo de canal `fxN` indexa o array incluindo os efeitos desativados.

| `type` | Campos | Faixa |
| --- | --- | --- |
| `blur` | `radius` | 0..64 |
| `drop_shadow` | `dx`, `dy`, `blur`, `color`, `opacity` (1) | offsets de no máximo 1024; blur 0..64; opacity 0..1 |
| `tint` | `black`, `white`, `amount` (1) | amount 0..1 |
| `fill` | `color`, `opacity` (1) | opacity 0..1 |

Anime com `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` no alvo `layer::fxN`.

### 7.3 `normal_map`

`{"data": base64-PNG | "path": file, "fit": "stretch" | "tile" | "exact" (padrão stretch),
"strength": 0..8 (padrão 1), "green_down": bool (padrão false)}`. Precisa de `data` ou `path`. Ele alimenta
o pós-efeito `relight`; renders simples não o usam. Como `Layer.normal_map`, ele é ajustado à caixa delimitadora em pixels dessa camada. `fit` `exact` exige um mapa exatamente do tamanho do quadro e, caso contrário, é um erro.

## 8. Canais, trilhas, easing

### 8.1 Canal

```json
{"target": "ball", "prop": "layer_y", "source": "keys",
 "track": {"keys": [{"frame": 0, "value": 40}, {"frame": 24, "value": 170}]},
 "mode": "replace", "weight": 1}
```

| Nome em JSON | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `target` | string | `""` | Nome, ou a forma `layer::sub`. Ignorado para propriedades de câmera. |
| `prop` | string | **Obrig.** | Uma das propriedades em snake_case abaixo. |
| `source` | `keys`, `expr`, `noise`, `driven` | **Obrig.** | Tag achatada, com os campos dessa fonte ao lado dela. |
| `mode` | `replace`, `add`, `multiply` | `replace` | Como o valor se combina com o atual. |
| `weight` | número | 1 | Influência, deve ser finita; mistura em direção ao resultado. |

Combinação: replace é `cur + (v - cur) * w`; add é `cur + v * w`; multiply é
`cur * (1 + (v - 1) * w)`. Valores não finitos são ignorados. Os canais rodam na ordem da lista, de modo que um canal
posterior vê os resultados dos anteriores (por meio de `get(...)` e `driven`).

Fontes:

| `source` | Campos |
| --- | --- |
| `keys` | `track`: `{keys: [{frame, value, ease?}], extrapolate?}` |
| `expr` | `expr`: string (seção 9) |
| `noise` | `amp`, `freq` (ciclos por segundo), `seed` (0), `octaves` (1). Saída em `[-amp, amp]`; baseada em hash, determinística. |
| `driven` | `input` (alvo), `input_prop`, `curve`: uma trilha cujos "quadros" são valores de entrada (por exemplo, do ângulo do cotovelo para a quantidade de protuberância). |

### 8.2 Propriedades

Existem 39 propriedades. A propriedade decide o que `target` significa.

| Grupo | Propriedades | `target` |
| --- | --- | --- |
| Osso | `bone_angle`, `bone_scale`, `bone_offset_x`, `bone_offset_y` | nome do osso |
| Camada | `layer_opacity`, `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`, `layer_skew`, `layer_switch`, `layer_wobble` | caminho da camada (`parent/child`) |
| Câmera | `camera_x`, `camera_y`, `camera_zoom`, `camera_rotation`, `camera_focus`, `camera_blur` | ignorado |
| Rig | `ik_mix`, `constraint_weight` | nome de ik ou de restrição |
| Path | `path_morph`, `point_dx`, `point_dy`, `grad_dx`, `grad_dy` | `layer::path::morphName`, `layer::path::anchorIndex`, `layer::path::fill.from` / `fill.to` / `stroke.from` / `stroke.to` |
| Deformador | `lattice_dx/dy`, `spine_dx/dy`, `pin_dx/dy`, `cage_dx/dy` | `layer::deformer::c,r` (lattice, cloth) ou `layer::deformer::i` |
| Efeito | `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` | `layer::fxN` |

Semântica das alças de gradiente: em gradientes lineares, `.from` e `.to` movem o ponto inicial e o final por
`(dx, dy)`. Em gradientes radiais, `.from` move o centro e `.to` altera o raio por `dx`.
Somente caminhos cujo fill ou stroke é um gradiente têm esses alvos. `layer_switch` exige uma
camada `switch`. Uma propriedade desconhecida lista no erro todas as variantes válidas.

Limites na escrita: opacity 0..1, camera zoom de pelo menos 0.01, camera blur de pelo menos 0, `ik_mix` e
`constraint_weight` 0..1.

### 8.3 Trilha e chave

`Track` = `{"keys": [...], "extrapolate": "hold"}`. Uma chave é `{"frame": number, "value": number,
"ease": {...}}`. As chaves devem ser finitas e estritamente crescentes em quadro:
`channel N ('target'): keys must be finite and strictly increasing in frame`. O ease de uma chave
molda o segmento que começa nessa chave.

Extrapolação (fora da primeira e da última chave): `hold` (padrão), `linear`, `loop`, `ping_pong`,
`loop_offset`.

### 8.4 Easing

`ease` é um objeto com uma tag `type`. O padrão é `{"type": "linear"}`.

| `type` | Parâmetros | Curva |
| --- | --- | --- |
| `linear` | | `u` |
| `step` | | Mantém o valor da chave até a próxima chave. |
| `ease_in` | | `u^2` |
| `ease_out` | | `1 - (1-u)^2` |
| `ease_in_out` | | Smoothstep. |
| `cubic_bezier` | `x1, y1, x2, y2` | Curva estilo CSS por (0,0) e (1,1). |
| `smooth` | | Hermite de Catmull-Rom não uniforme pelas chaves vizinhas. |
| `spring` | `bounces`, `decay` | `1 - e^(-decay*u) * cos(2*pi*bounces*u) * window`, window = `1 - smooth01((u-0.85)/0.15)`; termina exatamente no alvo. |
| `back` | `overshoot` | Cúbica com inclinação inicial `3 + overshoot` (limitado a 0 ou mais). Overshoot 1 tem pico em 1.037 (u = 2/3); overshoot 2 tem pico em 1.1317 (u = 0.5556). |
| `bounce` | | Restituição 0.6, três rebotes, duração total de 3.352 unidades de tempo. Primeiro impacto em u = 0.2983; picos dos rebotes 0.64, 0.8704, 0.953344; termina em 1. |

`back` e `bounce` são derivações originais (veja `CLEANROOM.md`).

## 9. Expressões

Um canal com `"source": "expr"` avalia uma fórmula por quadro. O código-fonte é limitado a 4096 bytes
e a profundidade de aninhamento 64; os erros são informados no momento da validação com uma posição, por exemplo
`channel 2 ('upper'): expression error: unexpected end of expression`.

Variáveis: `frame`, `time` (= frame / fps), `fps`, `frames`, `pi`, `t` (= frame / frames).

Operadores (da menor para a maior precedência): `?:`, `||`, `&&`, comparações, `+ -`, `* / %`, unários `- !`,
`^` (associativo à direita). A divisão por zero dá 0; `%` é euclidiano (`rem_euclid`).

Funções: `sin cos tan asin acos atan atan2 abs sqrt pow min max clamp lerp smoothstep floor ceil
round fract sign mod exp ln noise` (1 ou 2 argumentos) `fbm` (2 ou 3) `wiggle` (2 ou 3) `saw tri
square deg rad`, e `get("target", "prop")`. Um resultado de função não finito vira 0.

`get("target", "prop")` lê o valor atual de qualquer propriedade, inclusive os resultados de canais
anteriores. As duas strings devem ser resolvidas no momento da validação (`channel 2: get("q", "bone_angle"): unknown
target`). Um canal que lê um valor deve vir depois do canal que o escreve.

Exemplo do projeto da bola:

```json
{"target": "shadow", "prop": "layer_opacity", "source": "expr",
 "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"}
```

As expressões de binding (seção 11) usam uma forma restrita: a variável é `value`, e as variáveis de tempo
e `get()` são proibidas.

## 10. Câmera, shots, áudio, metadados

### 10.1 `camera`

| Nome JSON | Tipo | Padrão | Intervalo | Significado |
| --- | --- | --- | --- | --- |
| `position` | `Vec2` ou null | null (centro do palco) | | Ponto do mundo no centro da tela. |
| `zoom` | número | 1 | limitado a no mínimo 0.01 | |
| `rotation` | número | 0 | graus | |
| `focus` | número | 1 | | A profundidade `parallax` da camada que fica nítida (usada por `dof` e `dof-export`). |
| `blur` | número | 12 | finito, `>= 0` | Raio de desfoque em pixels por unidade de distância de parallax. |

Parallax: uma camada com `parallax = p` vê `zoom = 1 + (zoom - 1) * p`,
`pos = centre + (pos - centre) * p`, `rotation = -rotation * p`. Um `focus` ou `blur` não finito
gera `camera focus must be finite and camera blur a finite number >= 0`.

### 10.2 Arrays de metadados (não validados no carregamento)

Estes arrays carregam dados do editor e de ferramentas. O motor os aceita como dados; as operações
que os usam os verificam quando são executadas.

| Array | Campos do item |
| --- | --- |
| `markers` | `name`, `frame`, `end` (opcional; uma região quando definido) |
| `comps` | `name`, `layers`: mapa de caminho de camada para `{visible, opacity, selected?}` |
| `events` | `name`, `frame`, `data` (qualquer JSON) |
| `bone_vis` | `bone`, `from`, `to` (intervalo de quadros em que o osso é exibido) |
| `channel_meta` | `target`, `prop`, `color`, `shy`, `solo`, `lock` (flags do editor; `lock` bloqueia set-key) |
| `shots` | `name`, `x`, `y`, `zoom`, `rotation` (vistas de câmera salvas) |

```json
"markers": [{"name": "intro", "frame": 0, "end": 12}, {"name": "hit", "frame": 20}],
"events": [{"name": "step", "frame": 12, "data": {"sound": "step.wav"}}],
"shots": [{"name": "wide", "x": 200, "y": 150, "zoom": 1, "rotation": 0}]
```

### 10.3 `sounds[]`

| Nome JSON | Tipo | Intervalo | Significado |
| --- | --- | --- | --- |
| `name` | string | único | |
| `file` | string | um caminho de asset (seção 18) | WAV ou outro áudio suportado. |
| `start` | número | segundos | Posicionamento na linha do tempo. |
| `gain` | número | 0..4 | |
| `trim_in`, `trim_out` | número | `trim <= 86400` | Segundos cortados do início e do fim do arquivo. |
| `fade_in`, `fade_out` | número | cabe no clipe aparado | Segundos. |

`check` (e não `Engine::new`) informa um arquivo ausente: `sound 's': cannot read sound file ./nofile.wav: ...`.

## 11. Clipes, skins, links, interação, lods

### 11.1 `clips[]` e `clip_baked`

Um clipe posiciona um `take` reutilizável na linha do tempo. O baking dos clipes acrescenta canais
comuns (depois dos feitos à mão) e guarda um instantâneo em `clip_baked`, de modo que refazer o
baking é idempotente e o trabalho manual nunca é apagado.

| Campo do clipe | Significado |
| --- | --- |
| `name`, `start`, `speed`, `loops` | Posicionamento. |
| `take` | `{name, length, tracks: [{target, prop, keys}]}` |
| `mode` | `replace`, `additive`, `multiply` |
| `weight`, `weight_keys` | Influência; trilha animada opcional. |
| `include`, `exclude` | Listas de padrões que selecionam trilhas. |
| `track_weights` | `[{pattern, prop, weight}]` |
| `fade_in`, `fade_out` | Quadros. |
| `rename` | `[[from, to]]` renomeações de alvo. |

### 11.2 `skins[]`

`{name, group, layers, constraints, iks, bones}`. Skins do mesmo `group` são variantes exclusivas.
A escala de osso por skin é condensada em um canal de passos `bone_scale` do tipo multiply quando
é feito o baking. As regras de skin só são verificadas quando as operações de skin são executadas.

### 11.3 `mesh_locks[]` e `glues[]`

Um par de glue usa `VertexRef` = `{layer, path (default 0), index}`; um mesh lock nomeia caminhos inteiros com `{layer, path}` e uma lista opcional de índices de vértices seguidores (vazia significa todos os vértices).

```json
"mesh_locks": [{"name": "lock", "host": {"layer": "hostm", "path": 0},
                "follower": {"layer": "follm", "path": 0}, "vertices": [], "strength": 1}],
"glues": [{"name": "g", "mode": "rigid", "strength": 1,
           "pairs": [{"a": {"layer": "hostm", "path": 0, "index": 0},
                      "b": {"layer": "s1", "path": 0, "index": 0}}]}]
```

O `mode` de glue é `snap`, `rigid` ou `offset`. Ambos são validados por `links::resolve` e aplicados
no espaço do palco em ordem topológica (passo 12 da avaliação).

### 11.4 `machines[]` e `bindings[]` (somente em tempo de execução)

Máquinas de estados e data bindings são conduzidos pelo Player em tempo de execução. Para exportação
e baking, tornam-se canais `replace` constantes acrescentados ao projeto. Cada máquina precisa de
pelo menos uma camada.

```json
"machines": [{"name": "m",
  "inputs": [{"name": "go", "kind": "bool", "default": 0},
             {"name": "fire", "kind": "trigger", "hold": false}],
  "layers": [{"name": "main", "entry": "idle", "mask": [],
    "states": [
      {"name": "idle", "source": {"kind": "region", "marker": "intro"},
       "mode": "loop", "speed": 1, "blend_in": 0.1},
      {"name": "wave", "source": {"kind": "clip", "clip": "wave"},
       "mode": "once", "speed": 1, "blend_in": 0.1}],
    "transitions": [
      {"from": "idle", "to": "wave", "when": [{"input": "go", "op": "eq", "value": 1}],
       "duration": 0.2, "ease": {"type": "ease_in_out"}},
      {"from": "wave", "to": "idle", "exit_time": 1}]}]}]
```

O `kind` de uma entrada é `bool`, `number` ou `trigger`. O `source.kind` de um estado é `clip`,
`segment`, `region` ou `blend`; `mode` é `loop`, `once` ou `ping_pong`. `Transition` e `Condition`
rejeitam chaves desconhecidas.

```json
"bindings": [
  {"name": "hp", "path": "player.health", "expr": "value/100",
   "remap": {"in_min": 0, "in_max": 1, "out_min": 0, "out_max": 1, "clamp": true,
             "curve": {"type": "linear"}},
   "smooth": 0.1,
   "target": {"kind": "prop", "target": "sw", "prop": "layer_opacity"},
   "mode": "replace", "weight": 1},
  {"name": "go", "path": "input.go",
   "target": {"kind": "input", "machine": "m", "input": "go"}}]
```

O `weight` de um binding deve estar entre 0..1 (`binding 'b': weight must be within 0..1`). Os alvos
de máquinas e bindings são verificados por uma compilação de teste.

### 11.5 `containers[]` e `scrolls[]`

Layout 1-D no estilo flexbox. Campos do container: `id`, `members`, `axis` (`row`, `column`), `gap`,
`cross_gap`, `padding` (4 números), `justify`, `align`, `wrap`, `rect`. Um `rect` tem um `kind`:
`fixed` (`x,y,w,h`), `pin`, `aspect` ou `hug`. Um membro tem exatamente um entre `layer`, `container`
ou `spacer`, além de `grow`, `shrink`, `width`, `height`, `align`, `resize`. Limites: 100000 membros,
profundidade de aninhamento 64. Um scroll tem `id`, `viewport`, `content`, `axis`, `offset`, `elastic`,
`snap_x`, `snap_y`, `fling`, `clip_layer`, `input`.

```json
"containers": [{"id": "row", "axis": "row", "gap": 4, "padding": [2,2,2,2],
  "justify": "start", "align": "start", "wrap": "none",
  "rect": {"kind": "fixed", "x": 0, "y": 0, "w": 200, "h": 60},
  "members": [{"layer": "c1"}, {"spacer": true, "grow": 1},
              {"layer": "c2", "grow": 0, "shrink": 1, "width": 30, "height": 30,
               "align": "center", "resize": true}]}],
"scrolls": [{"id": "sc", "viewport": {"kind": "fixed", "x": 0, "y": 100, "w": 100, "h": 80},
  "content": "list", "axis": "y", "offset": [0, 10],
  "elastic": {"limit": 40, "stiffness": 180, "damping_ratio": 0.9},
  "snap_x": [], "snap_y": [0, 40], "fling": 0.15, "clip_layer": "mask_rect", "input": "scroll.y"}]
```

Os offsets do container são somados à posição da camada e multiplicados na escala (passo 7 da
avaliação). `apply_clip_masks` adiciona referências de máscara ao projeto que recebe, portanto
modifica a cópia do projeto mantida pelo motor.

### 11.6 `lods[]`

Um registro de uma passada de nível de detalhe escrita por `osatura lod`: `level`, `tolerance_px`,
`scale`, `declared_px`, `bound_stage`, `path_px`, `mesh_px`, `keys_px`, `cull_px`, `stats`
(`path_anchors`, `mesh_vertices`, `keys`, `layers`, `bytes`) e `notes`. É documentação do que uma
passada fez; o renderizador nunca o lê.


## 12. Regras de validação, limites e mensagens de erro

`Engine::new` valida em uma única passada e devolve todos os problemas. A tabela lista o que é
imposto; as mensagens entre aspas foram produzidas por `osatura check` em arquivos deliberadamente
quebrados.

### 12.1 Limites globais

| Regra | Limite | Mensagem |
| --- | --- | --- |
| fps | `(0, 1000]` | `fps must be within (0, 1000]` |
| frames | `1..=200000` | `frames must be within 1..=200000` |
| width, height | `> 0` | `width and height must be > 0` |
| width, height | `<= 32768` | `width and height must be at most 32768` |
| qualquer float | `abs <= 1e7` | `value 100000000.0 at /bones/0/length is outside the supported range of +-1e7 (coordinates, scales, offsets and key values must stay below it)` |
| bones | no máximo 4096 | `at most 4096 bones` |
| format | `<= 1` | `project format 2 is newer than this build supports (1); update OSATURA` |
| tamanho do JSON | CLI `--max-input-bytes`, padrão 256 MiB; API `MAX_JSON_BYTES` 256 MiB | |

O limite de 1e7 é aplicado duas vezes. No carregamento, um literal float além dele faz o parse falhar
e a mensagem é envolvida como `invalid JSON in <file>: value ...`. Um literal sem ponto decimal (um
inteiro) passa pelo parse e é capturado pela nova verificação em `Engine::new`, onde aparece como um
item comum da lista. O limite existe para que uma multiplicação posterior não possa estourar `f32` e
chegar ao infinito.

### 12.2 Rig

| Regra | Mensagem |
| --- | --- |
| o pai do osso deve existir antes | `bone 'lower' has parent 'nope' which is missing or listed after it` |
| comprimento do osso finito, `>= 0` | `bone 'lower' has invalid length` |
| `angle_limits` `min <= max`; `scale_limits` `0 < min <= max` | |
| ik precisa de 2 ossos | `ik 'reach': needs at least two bones` |
| cadeia de ik ligada por pai | `ik 'reach': each bone must be a direct child of the previous one` |
| alvo do ik fora da cadeia | |
| soft, stretch | `ik 'reach': soft must be within 0..0.9 and stretch within 1..4` |
| uma restrição não pode ter a si mesma como alvo | |
| spring | `spring on 'upper': frequency must be in (0, 30] Hz and damping >= 0`; uma por osso |

### 12.3 Camadas, caminhos, geometria

| Regra | Mensagem |
| --- | --- |
| nome da camada não vazio, sem `/`, sem `::` | `layer name 'a/b' must be non-empty and contain neither '/' nor '::'` |
| caminho duplicado | `duplicate layer path 'arm'` (e linhas para os alvos animáveis duplicados) |
| nomes de caminhos e deformadores sem `::` e únicos dentro da camada | |
| a contagem de âncoras do morph é igual à do caminho base | |
| os skin weights nomeiam ossos conhecidos; peso finito, `>= 0` | |
| malha livre | pelo menos 3 vértices, pelo menos 1 triângulo, no máximo 200000 / 400000, finitos, índices válidos, `uvs` vazio ou com a mesma contagem |
| malha em grade | `cols`, `rows >= 1` |
| máscara | sem auto-máscara; `mask cycle involving layer 'arm'` |
| fonte de imagem | `layer 'im': image has neither 'png_base64' nor 'file'` |
| liquid | pelo menos 1 blob; `threshold > 0`; `cell >= 0.25`; `radius > 0` do blob |
| boolean | pelo menos 2 caminhos |
| emitter | `rate` em `(0, 2000]`; `lifetime > 0`; jitters 0..1 |
| deformadores | somente em camadas vetoriais e de imagem; veja a seção 7.1 para cada tipo |
| efeitos | no máximo 16; blur e softness 0..64; offsets no máximo 1024; quantidades 0..1 |
| normal map | `strength` 0..8; precisa de `data` ou `path` |
| camera | `camera focus must be finite and camera blur a finite number >= 0` |

### 12.4 Canais

| Regra | Mensagem |
| --- | --- |
| o alvo é resolvido | `channel 0: unknown target 'zzz' for BoneOffsetX` |
| `layer_switch` em uma camada switch | `channel 2: layer_switch needs a switch layer, 'arm' is not one` |
| weight finito | |
| keys finitas e estritamente crescentes | `channel 0 ('target'): keys must be finite and strictly increasing in frame` |
| a expressão é analisada | `channel 2 ('upper'): expression error: unexpected end of expression` |
| alvos e props de `get()` são resolvidos | `channel 2: get("q", "bone_angle"): unknown target` |
| a entrada conduzida é resolvida | |

Nomes de `prop` ou `ease` desconhecidos produzem erros do serde que listam todas as variantes válidas,
por exemplo ``unknown variant `foo`, expected one of ...``. Tipos inteiros incorretos produzem
``invalid type: floating point `48.0`, expected u32``. Um campo obrigatório ausente produz
``missing field `frames` ``.

### 12.5 Outras seções validadas

- Sounds: `gain` 0..4, `trim` no máximo 86400 s, os fades cabem dentro do clipe aparado, nomes únicos.
  A existência do arquivo é verificada por `check`, não por `Engine::new`.
- Machines: pelo menos uma camada (`machine 'm': needs at least one layer`); transições e
  condições rejeitam chaves desconhecidas. Bindings: `binding 'b': weight must be within 0..1`. Seus
  alvos são verificados compilando-os uma vez.
- Containers e scrolls: membros únicos, no máximo 100000, profundidade de aninhamento 64, regras de rect.
- Mesh locks e glues: referências resolvidas por `links::resolve`.
- Arquivos de imagem e de áudio: seção 18.

### 12.6 Dados do editor: avisos, e erros apenas para números não finitos

`Engine::new` e `check` examinam `skins`, `comps`, `markers`, `events`, `shots`, `channel_meta`, `bone_vis`, `clips` e
`lods` (módulo `editordata`). Para que todo projeto salvo anteriormente continue carregando, **somente valores que não podem existir
em um arquivo salvo são erros**: NaN ou infinito em markers, events, shots, `bone_vis`, opacidade de comp, lods e opacidade de camada.
Todo o resto é um **aviso** impresso por `check` (e relatado por `lint` como L013): zoom da câmera `<= 0` (o motor o limita a 0.01),
`opacity` de camada fora de 0..1 (limitada), `ik_mix` e `constraint_weight` fora de 0..1 (limitados), valores de `layer_opacity` e
`camera_zoom` animados por keys fora do seu intervalo, referências desconhecidas a camada/osso/canal, nomes duplicados, tempos fora da
animação, skins inválidas (`skins::validate`), clipes (`clipstack::validate`, e alvos de trilhas de take que não são resolvidos,
respeitando `rename`), `channel_meta` e `lods`. As fixtures de `out/` carregam sem avisos.

`osatura check` imprime os avisos de lint deste tipo; o linter completo (`osatura lint`) tem mais regras (de L001 em diante).

## 13. Exemplo 1: bola quicando

Verificado com `osatura check` (`ok: 'bouncing ball' 320×240 @ 24 fps, 48 frames, 0 bones, 0 ik,
3 layers, 5 channels`) e `osatura render ball.json 7 out.png`. O que ele mostra:

- `layer_y` animado por keys com easing `bounce` no primeiro segmento: a bola cai de y=40 até o
  chão em y=170 e quica três vezes. O primeiro impacto ocorre por volta do quadro 7.2 (24 / 3.352).
  O segmento de 24 a 47 sobe de volta com `ease_out`.
- Uma sombra que segue a bola: `layer_x` e `layer_opacity` são expressões que leem a bola
  com `get()`. Elas vêm listadas depois dos canais da própria bola, porque os canais são executados em ordem.
- Squash no contato: uma expressão retorna 0.8 para `layer_scale_y` quando a bola está abaixo de y=160.
- Círculo a partir de quatro âncoras Bezier com comprimento de handle 16.57 (0.5523 vezes o raio 30).

```json
{
  "format": 1,
  "name": "bouncing ball",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "floor",
      "type": "vector",
      "paths": [
        {
          "name": "line",
          "closed": false,
          "anchors": [{"pos": [0, 200]}, {"pos": [320, 200]}],
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "shadow",
      "type": "vector",
      "paths": [
        {
          "name": "blob",
          "closed": true,
          "anchors": [
            {"pos": [-26, 0], "handle_out": [-26, -5], "handle_in": [-26, 5]},
            {"pos": [0, -6], "handle_in": [-14, -6], "handle_out": [14, -6]},
            {"pos": [26, 0], "handle_in": [26, -5], "handle_out": [26, 5]},
            {"pos": [0, 6], "handle_in": [14, 6], "handle_out": [-14, 6]}
          ],
          "fill": "#00000055"
        }
      ],
      "transform": {"position": [160, 200]}
    },
    {
      "name": "ball",
      "type": "vector",
      "paths": [
        {
          "name": "disc",
          "closed": true,
          "anchors": [
            {"pos": [30, 0], "handle_in": [30, -16.57], "handle_out": [30, 16.57]},
            {"pos": [0, 30], "handle_in": [16.57, 30], "handle_out": [-16.57, 30]},
            {
              "pos": [-30, 0],
              "handle_in": [-30, 16.57],
              "handle_out": [-30, -16.57]
            },
            {
              "pos": [0, -30],
              "handle_in": [-16.57, -30],
              "handle_out": [16.57, -30]
            }
          ],
          "fill": {
            "center": [-8, -10],
            "radius": 44,
            "stops": [
              {"offset": 0, "color": "#ffd9a0"},
              {"offset": 1, "color": "#d6336c"}
            ]
          },
          "stroke": {"paint": "#3d2c2e", "width": 2}
        }
      ],
      "transform": {"position": [160, 40]}
    }
  ],
  "channels": [
    {
      "target": "ball",
      "prop": "layer_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 40, "ease": {"type": "bounce"}},
          {"frame": 24, "value": 170, "ease": {"type": "ease_out"}},
          {"frame": 47, "value": 40}
        ]
      }
    },
    {
      "target": "ball",
      "prop": "layer_x",
      "source": "keys",
      "track": {"keys": [{"frame": 0, "value": 60}, {"frame": 47, "value": 260}]}
    },
    {
      "target": "shadow",
      "prop": "layer_x",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_x\")"
    },
    {
      "target": "shadow",
      "prop": "layer_opacity",
      "source": "expr",
      "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"
    },
    {
      "target": "ball",
      "prop": "layer_scale_y",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_y\") > 160 ? 0.8 : 1",
      "mode": "replace"
    }
  ]
}
```

## 14. Exemplo 2: braço com rig e IK

Verificado com `osatura check` (3 bones, 1 ik, 3 layers, 2 channels). `osatura explain arm.json
24` informa a ponta em (170, 190), igual ao osso `target`, e `osatura render arm.json 24
out.png --bones` desenha o esqueleto. O que ele mostra:

- `upper` é uma raiz com um limite de ângulo; `lower` é seu filho (`rest_offset` `[90, 0]` é igual ao
  comprimento do pai); `target` é um osso raiz livre que a cadeia de IK procura alcançar.
- `reach` é uma cadeia de IK de dois ossos, portanto o solver analítico é usado.
- `sleeve` tem skin: cada âncora lista pesos de osso. `hand` e `goal` são fixados com
  `bone`.
- Mover o osso `target` com canais `bone_offset_x` e `bone_offset_y` faz o braço se esticar.

```json
{
  "format": 1,
  "name": "rigged arm",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "bones": [
    {
      "name": "upper",
      "parent": null,
      "length": 90,
      "rest_offset": [60, 120],
      "rest_angle": 0,
      "angle_limits": [-120, 120]
    },
    {
      "name": "lower",
      "parent": "upper",
      "length": 80,
      "rest_offset": [90, 0],
      "rest_angle": 0
    },
    {
      "name": "target",
      "parent": null,
      "length": 10,
      "rest_offset": [230, 120],
      "rest_angle": 0
    }
  ],
  "ik": [
    {
      "name": "reach",
      "bones": ["upper", "lower"],
      "target": "target",
      "bend": 1,
      "mix": 1
    }
  ],
  "layers": [
    {
      "name": "arm",
      "type": "vector",
      "paths": [
        {
          "name": "sleeve",
          "closed": true,
          "anchors": [
            {"pos": [60, 106], "weights": [{"bone": "upper", "weight": 1}]},
            {
              "pos": [150, 104],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [228, 110], "weights": [{"bone": "lower", "weight": 1}]},
            {"pos": [228, 130], "weights": [{"bone": "lower", "weight": 1}]},
            {
              "pos": [150, 136],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [60, 134], "weights": [{"bone": "upper", "weight": 1}]}
          ],
          "fill": "#ffb347",
          "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
        }
      ]
    },
    {
      "name": "hand",
      "type": "vector",
      "bone": "lower",
      "paths": [
        {
          "name": "palm",
          "closed": true,
          "anchors": [
            {"pos": [246, 120], "handle_in": [246, 111], "handle_out": [246, 129]},
            {"pos": [232, 134], "handle_in": [241, 134], "handle_out": [223, 134]},
            {"pos": [218, 120], "handle_in": [218, 129], "handle_out": [218, 111]},
            {"pos": [232, 106], "handle_in": [223, 106], "handle_out": [241, 106]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "goal",
      "type": "vector",
      "bone": "target",
      "paths": [
        {
          "name": "ring",
          "closed": false,
          "anchors": [{"pos": [222, 120]}, {"pos": [238, 120]}],
          "stroke": {"paint": "#d6336c", "width": 3, "dash": [4, 4]}
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "target",
      "prop": "bone_offset_x",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": -60, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    },
    {
      "target": "target",
      "prop": "bone_offset_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": 70, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    }
  ]
}
```

## 15. Exemplo 3: boca com camada switch

Verificado com `osatura check` e renderizando os quadros 0, 5 e 9, que mostram três bocas
diferentes. Keys fora do intervalo são limitadas: um valor de key 7 mostra o filho 2 e -3 mostra o filho 0.

```json
{
  "format": 1,
  "name": "switch mouth",
  "width": 200,
  "height": 160,
  "fps": 12,
  "frames": 25,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "face",
      "type": "vector",
      "paths": [
        {
          "name": "head",
          "closed": true,
          "anchors": [
            {"pos": [180, 80], "handle_in": [180, 35], "handle_out": [180, 125]},
            {"pos": [100, 150], "handle_in": [145, 150], "handle_out": [55, 150]},
            {"pos": [20, 80], "handle_in": [20, 125], "handle_out": [20, 35]},
            {"pos": [100, 10], "handle_in": [55, 10], "handle_out": [145, 10]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "mouth",
      "type": "switch",
      "selected": 0,
      "layers": [
        {
          "name": "closed",
          "type": "vector",
          "paths": [
            {
              "name": "line",
              "closed": false,
              "anchors": [{"pos": [75, 110]}, {"pos": [125, 110]}],
              "stroke": {"paint": "#3d2c2e", "width": 4, "cap": "round"}
            }
          ]
        },
        {
          "name": "open",
          "type": "vector",
          "paths": [
            {
              "name": "oval",
              "closed": true,
              "anchors": [
                {"pos": [75, 106]},
                {"pos": [125, 106]},
                {
                  "pos": [112, 132],
                  "handle_in": [122, 132],
                  "handle_out": [102, 134]
                },
                {"pos": [88, 132], "handle_in": [98, 134], "handle_out": [78, 132]}
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
            }
          ]
        },
        {
          "name": "round",
          "type": "vector",
          "paths": [
            {
              "name": "o",
              "closed": true,
              "anchors": [
                {
                  "pos": [112, 118],
                  "handle_in": [112, 110],
                  "handle_out": [112, 126]
                },
                {
                  "pos": [100, 130],
                  "handle_in": [106, 130],
                  "handle_out": [94, 130]
                },
                {"pos": [88, 118], "handle_in": [88, 126], "handle_out": [88, 110]},
                {
                  "pos": [100, 106],
                  "handle_in": [94, 106],
                  "handle_out": [106, 106]
                }
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3}
            }
          ]
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "mouth",
      "prop": "layer_switch",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "step"}},
          {"frame": 4, "value": 1, "ease": {"type": "step"}},
          {"frame": 8, "value": 2, "ease": {"type": "step"}},
          {"frame": 12, "value": 1, "ease": {"type": "step"}},
          {"frame": 16, "value": 0, "ease": {"type": "step"}}
        ]
      }
    }
  ]
}
```

## 16. Ordem de avaliação

`Engine::evaluate(frame)` é uma função pura do número do quadro e do projeto. As springs são
simuladas uma única vez quando o motor é construído; nada mais carrega estado entre quadros. Os passos:

1. **Estado inicial** (`initial_state`): transformações estáticas das camadas, opacidade limitada a 0..1, `selected`
   do switch, amplitude do wobble, câmera, mix de `ik`, pesos das restrições e offsets iniciais dos
   slots de parâmetros (offsets de ossos, offsets de deformadores, pesos de morph e assim por diante).
2. **Canais** (`apply_channels`) na ordem da lista, cada um combinando com o valor atual por seu
   `mode` e `weight`. Resultados não finitos são ignorados. As escritas são limitadas conforme listado na seção 8.2.
3. **Limitação por limites** (`clamp_limits`): `angle_limits` e `scale_limits` por osso.
4. **Cinemática direta (FK)**: matrizes de mundo de todos os ossos, pais primeiro.
5. **Cadeias de IK** na ordem da lista, ignorando cadeias com `mix <= 0`. Após cada cadeia a pose é limitada
   e a FK é recalculada, de modo que a cadeia seguinte vê o resultado.
6. **Restrições** na ordem da lista, ignorando `weight <= 0`: aim, copy_rotation, pin.
7. **Containers de layout**: os offsets são somados a x e y da camada e multiplicados na escala.
8. **Springs com baking**: os ângulos do baking substituem o ângulo do osso, depois a FK é recalculada.
9. **Matrizes de skin**: `skin_mat = world * inverse(rest world)` por osso.
10. **Câmera**: uma matriz de câmera por camada raiz, usando o `parallax` da camada raiz (seção 10.1).
11. **Nós** em pré-ordem. A matriz de palco de um nó é `parentStage * boneAttach(skin_mat) * local`.
    A geometria vetorial é construída nesta ordem:
    1. morphs (blend shapes aditivos);
    2. offsets `point_dx/dy` por âncora;
    3. deslocamentos dos handles de gradiente (`grad_dx/dy`);
    4. deformadores na ordem da lista, depois `wobble`;
    5. skinning com pesos normalizados (pesos vazios significam estático; os handles de gradiente também
       recebem skinning).
    A geometria com skinning vive em coordenadas de repouso do palco, e a transformação da camada é aplicada depois dela.
    Um switch mostra um filho (arredondado, limitado). A geometria de liquid, boolean e emitter é
    sintetizada a cada quadro. Uma imagem recebe uma malha em grade ou livre; com deformadores mas sem malha, recebe uma
    grade de 16 por 16.
12. **Mesh locks e glues** no espaço do palco, em ordem topológica.
13. **Reparo de valores não finitos** (`repair_nonfinite`): um nó cuja geometria contém NaN ou infinito
    é ocultado e `Frame.nonfinite` é definido. `evaluate_checked` e `check_all_frames_finite` transformam isso
    em um erro.

Consequências com as quais você pode contar:

- Um canal vê apenas os canais anteriores a ele na lista (por meio de `get` e `driven`).
- O IK sobrepõe a FK nos ossos da cadeia, em proporção a `mix` (a mistura é
  `old + wrap_pi(new - old) * mix`). Com `mix` 0 a cadeia é FK pura.
- As restrições são executadas depois do IK, portanto uma restrição aim sobre um osso de IK prevalece.
- As springs agem por último sobre o ângulo do osso, depois do IK e das restrições.

## 17. Ordem de renderização

- As camadas raiz são desenhadas na ordem da lista, os filhos na ordem da lista dentro do pai.
- Um nó que não está `visible` é ignorado (ele ainda pode ser fonte de máscara).
- Um nó é renderizado em uma camada fora da tela quando tem `opacity < 1`, efeitos, um modo de mesclagem diferente de
  `normal`, ou uma máscara. Etapas: conteúdo; efeitos de camada na ordem da lista (em um canvas com margem); máscara
  (a camada de máscara é desenhada com sua própria transformação; seu alfa, incluindo sua opacidade, multiplica;
  `invert` o inverte); composição com opacidade e modo de mesclagem.
- A profundidade de campo é aplicada somente por `dof` e `dof-export`: raio = `abs(parallax - focus) *
  blur`, no máximo 24 pixels por padrão.
- A sobreposição de ossos (`--bones`) é desenhada por último.

## 18. Assets e caminhos

Os projetos podem referenciar arquivos: `image.source.file`, `normal_map.path`, `sounds[].file`.

- Os caminhos devem ser relativos, não podem conter `..` e, depois de seguir os symlinks, devem permanecer dentro do
  diretório do projeto: `layer 'im': asset '/etc/passwd': absolute paths are not allowed (a project
  may only reference files inside its own directory; use --allow-external-assets to override)` e
  `layer 'im': asset '../x.png': '..' is not allowed (...)`.
- `--allow-external-assets` suspende essa restrição para um único comando. Somente arquivos regulares são
  abertos.
- Limites de tamanho: imagens 64 MB em disco, áudio 256 MB. Imagens decodificadas: cada lado no máximo 16384 e no
  máximo 64 megapixels.
- Limites de saída: raster no máximo 16384 por lado e 100 megapixels; sequências no máximo 2 GiB; orçamento de
  exportação quadros vezes âncoras no máximo 40 milhões.
- Quando a CLI grava um projeto, ela o revalida por meio de `Engine::new` e reescreve os caminhos de assets
  relativos ao local de saída.

Prefira `png_base64` para um arquivo autocontido.

## 19. Política de versionamento

- `format` é um inteiro, atualmente `1` (`FORMAT_VERSION`).
- Um arquivo com `format` maior do que o suportado pela build é recusado com a mensagem da seção 12.1;
  o motor não adivinha.
- Adicionar um campo opcional com valor padrão não é uma mudança de versão. As builds mais antigas o ignoram (e
  `check` avisa sobre ele como um campo desconhecido).
- Remover ou reinterpretar um campo exige um incremento de versão, e o incremento exige uma etapa de migração em
  `Project::from_json` que atualize os documentos mais antigos antes da validação.
- A API da biblioteca Rust tem sua própria versão, `API_VERSION` 1.4.0, protegida pelo teste de snapshot
  `api_surface.txt`; a ABI C tem `ABI_VERSION` 1. Veja `docs/API.md`.

## 20. Lacunas conhecidas entre código e comentários

Estas foram encontradas durante a redação desta referência.

1. **Corrigido.** A regra de keys de switch (`lint::switch_key_warnings`, L012) agora diz o que o motor faz: uma key abaixo de 0 mostra o filho 0,
   uma key igual ou acima da contagem de filhos mostra o último filho (um switch vazio não mostra nada). Teste: testes unitários de `lint`, verificados
   contra o motor.
2. O comentário em `project.rs` diz que arquivos sem `format` são a versão 1, mas o padrão do serde é
   `FORMAT_VERSION`. Os dois concordam hoje apenas porque a constante é 1. Quando a constante for aumentada, um
   arquivo sem `format` seria lido como a versão mais recente.
3. **Corrigido.** `unknownfields` não relata mais uma estrutura vazia escrita explicitamente (`clips[].weight_keys: {"keys": []}`);
   `default_like` é recursivo, e um teste fixa o conjunto de estruturas opcionais que o gravador omite quando vazias.
4. **Corrigido (como avisos).** Os campos do editor são validados; veja a seção 12.6.
5. **Corrigido na wave 7 (L-71).** A geometria de uma camada com skin está no espaço de repouso do palco e a transformação do pai é aplicada depois
   dela. `Layer.bone` agora se fixa com `world = bone_skin * parent_stage * local`: a matriz de skin (mundo do osso x repouso inverso) é
   um movimento no espaço do palco, portanto é aplicada depois de toda a cadeia de pais. Em repouso a matriz de skin é a identidade, então nada
   se move; quando o osso está em **pose**, um grupo pai com escala 2 não dobra mais o deslocamento de uma camada fixada. Uma camada
   sem osso próprio segue o osso do ancestral fixado a osso mais próximo; o osso próprio de uma camada substitui o do ancestral
   (os dois movimentos não se acumulam). Testes: `bone_attach_under_a_transformed_group_composes_bone_after_the_group`,
   `bone_attach_is_inherited_and_a_child_bone_replaces_the_parents`. Projetos que dependiam do antigo deslocamento dobrado (um osso em pose
   sob um grupo com escala ou rotação) são renderizados de forma diferente; projetos sem essa combinação permanecem inalterados.
