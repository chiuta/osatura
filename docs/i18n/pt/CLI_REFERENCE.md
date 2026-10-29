<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Referência da linha de comando do OSATURA

Referência do binário `osatura` (crate `osatura-cli`, AGPL-3.0-or-later; ele faz link com o motor `osatura-core`, MPL-2.0).
Versão documentada: **osatura 0.1.0**. Este texto é licenciado sob CC-BY-4.0.

Cada sinopse abaixo foi copiada do próprio texto de uso do programa (`osatura` sem argumentos, ou
`osatura <command> --help`). Cada exemplo foi executado com o projeto demo embutido e a saída mostrada
foi reduzida a partir da saída real. Onde o texto de uso e o comportamento real divergem, este documento descreve o
comportamento real e lista a divergência em [Divergências conhecidas entre documentação e comportamento](#divergências-conhecidas-entre-documentação-e-comportamento) no final.

Documentos relacionados: [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (o que cada formato de exportação/importação mantém e descarta),
[CI.md](CI.md) (manifestos de CI de animação), [API.md](API.md) (biblioteca/ABI em C), [PATENT_NOTES.md](PATENT_NOTES.md)
(situação de codecs e patentes, não é aconselhamento jurídico).

## Conteúdo

1. [Convenções comuns a todos os comandos](#1-convenções-comuns-a-todos-os-comandos)
2. [Primeiros passos: o projeto demo](#2-primeiros-passos-o-projeto-demo)
3. [Criação e inspeção de projetos](#3-criação-e-inspeção-de-projetos)
4. [Renderização e exportação](#4-renderização-e-exportação)
5. [Rigging, skins, IK e utilitários de layout](#5-rigging-skins-ik-e-utilitários-de-layout)
6. [Animação, keys, timeline e geradores](#6-animação-keys-timeline-e-geradores)
7. [Física, tecido e deformadores](#7-física-tecido-e-deformadores)
8. [Áudio, lip-sync e sons](#8-áudio-lip-sync-e-sons)
9. [Importação (SVG, OpenRaster, Lottie, BVH, GIF/APNG, sequências de PNG, bitmaps)](#9-importação)
10. [Entrada live, MIDI, rastreamento facial e captura de movimento](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento)
11. [Máquinas de estados, data bindings, contêineres e regiões de rolagem](#11-máquinas-de-estados-vinculações-de-dados-contêineres-e-regiões-de-rolagem)
12. [Efeitos, profundidade de campo, onion skin, trails e câmera](#12-efeitos-profundidade-de-campo-onion-skin-trilhas-e-câmera)
13. [Nível de detalhe, relatórios de tamanho e export-fit](#13-nível-de-detalhe-relatórios-de-tamanho-e-export-fit)
14. [Garantia de qualidade: check, lint, ci, golden, fuzz, diff](#14-garantia-de-qualidade)
15. [Controle de versão (`vcs`)](#15-controle-de-versão-vcs)
16. [Estúdio, colaboração, servidor de agente MCP e registro da UI](#16-studio-colaboração-servidor-de-agente-mcp-e-registro-de-ui)
17. [Banco de poses, sugestões e outros utilitários](#17-banco-de-poses-sugestões-e-outros-auxiliares)
18. [Divergências conhecidas entre documentação e comportamento](#divergências-conhecidas-entre-documentação-e-comportamento)
19. [Índice alfabético de comandos](#índice-alfabético-de-comandos)

---

## 1. Convenções comuns a todos os comandos

### 1.1 Forma da linha de comando

```
osatura <command> [sub-command] <positional arguments...> [options]
```

* As opções podem aparecer **em qualquer lugar** da linha, inclusive depois do subcomando e depois dos argumentos posicionais
  (`osatura render demo.json 0 o.png --scale 2` e `osatura --scale 2 render ...` são equivalentes).
* As **opções com valor** recebem a palavra seguinte (`--scale 2`) ou a forma `--key=value` (`--scale=2`). A forma com `=` funciona para
  toda opção com valor. Um valor que contém `=` também é aceito após um espaço (`--remap a=b`).
* Os **flags** (opções booleanas como `--bones`) não recebem valor. `--bones=1` é recusado:
  `--bones is a flag and takes no value: write --bones (without =), got '--bones=1'`.
* **Opções estritas.** Uma opção (flag ou opção com valor) que o comando não documenta é um erro com uma dica
  "did you mean" (exit 1), em vez de ser ignorada:
  ```
  $ osatura render demo.json 0 o.svg --bone
  unknown option --bone for 'render' (did you mean --bones?); see: osatura render --help
  ```
  A verificação vale para todo comando que tem uma linha de uso, incluindo `check`, `stats`, `lint`, `coach`, `contracts` e
  `machine list` (versões antigas ignoravam silenciosamente flags extras nesses comandos). `--json`, `--quiet` e `--allow-empty` **não** são
  globais: são recusados por todo comando cuja linha de uso não os lista.
  **A verificação é por subcomando (por forma de sinopse), não por comando.** Uma opção listada para uma subforma é recusada
  nas suas irmãs, e a mensagem nomeia a subforma que a aceita:
  ```
  $ osatura ci diff a.png b.png --quiet
  unknown option --quiet for 'ci diff' (--quiet applies to: ci run); see: osatura ci --help
  ```
  Assim, `--quiet` vale apenas para `ci run`, `--json` apenas para `vcs log|status|diff|show`, `ui commands|shortcuts` e a
  forma `trail ... --target`, `--tol` apenas para `golden check` (e `ci diff`), `--scale` apenas para `golden write`, e
  cada edição de `keyedit` aceita somente as opções escritas depois do seu nome na sinopse. (`export` ainda reúne suas duas
  linhas de sinopse, porque o formato é escolhido pela extensão do arquivo; `--codec`, `--columns` e `--scale` que não
  se aplicam à extensão escolhida são reportados pelo próprio `export`.)
* Alguns comandos mais antigos recebem **palavras** `key=value` em vez de `--options` (`cloth`, `physics`, `cage`, `warp-pivot`,
  `soft-scale`, `mesh-lock`, `glue`, `audio-react`, `beats`, `import-seq`, `import-anim`, `layer-pivot`, `chan-meta`,
  `waveform`). Elas são posicionais e escritas exatamente como na sinopse (`gravity=900`, não `--gravity 900`).
* Um **quadro** (frame) é um número (frações permitidas onde fizerem sentido). Um projeto com `frames = N` tem os quadros `0..N-1`.
  As camadas são endereçadas por **caminho** (`group/child`), os ossos por **nome**; a câmera tem o alvo vazio `""`.
* Os **nomes de propriedades** (usados por `--prop`, `slider`, `drive`, arquivos de mapeamento, ...) são os nomes em snake_case:
  `bone_angle bone_scale bone_offset_x bone_offset_y layer_opacity layer_x layer_y layer_rotation layer_scale_x
  layer_scale_y layer_skew layer_switch layer_wobble camera_x camera_y camera_zoom camera_rotation camera_focus
  camera_blur ik_mix constraint_weight path_morph point_dx point_dy lattice_dx lattice_dy spine_dx spine_dy pin_dx
  pin_dy cage_dx cage_dy fx_blur fx_dx fx_dy fx_soft fx_amount grad_dx grad_dy`.  Alguns comandos imprimem as mesmas propriedades em CamelCase (`BoneAngle`) nas suas mensagens; os valores das opções são sempre em snake_case.

### 1.2 Opções globais

| Opção | Significado |
|---|---|
| `--allow-external-assets` | Permite que um projeto nomeie arquivos por caminho absoluto ou fora do seu próprio diretório (sons, imagens, normal maps...). Padrão: recusado. |
| `--follow-symlinks` | Grava no **destino** de um link simbólico informado como arquivo de saída. Padrão: recusado. |
| `--max-input-bytes N` | Maior arquivo de projeto que é lido. Padrão `268435456` (256 MiB). Deve ser um número inteiro positivo. |
| `<command> --help` | Imprime as linhas de uso de um comando na stdout (exit 0). |
| `<command> --version`, `--version` | `osatura 0.1.0`; sem comando, também a linha de licença e a oferta de código-fonte AGPL. |
| `--json`, `--quiet`, `--allow-empty` | Opções por comando, **recusadas** (`unknown option`) pelos comandos cuja sinopse não as lista (`--json`: `pick`, `select-box`, `timeline`, `trail`, `size-report`, `collab-sim`, `vcs`, `ui`, `face-detect`, `pose-suggest`; `--allow-empty`: `physics`, `live-map`, `face-bake`; `--quiet`: `ci run`). Scripts que as passavam uniformemente a todos os comandos precisam parar de fazê-lo. |

Exemplos (verificados):

```
$ osatura --max-input-bytes 100 check demo.json
cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)
[exit 1]

$ osatura check /dev/zero
cannot read /dev/zero: not a regular file (devices, pipes and directories are refused)
[exit 1]

$ ln -s real_target.svg link.svg                       # dangling link
$ osatura render demo.json 0 link.svg
cannot write link.svg: link.svg: refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself
[exit 1]
```

### 1.3 Leitura de arquivos de entrada

* Os arquivos de entrada devem ser **arquivos regulares** de tamanho não superior a `--max-input-bytes`. Dispositivos, pipes, diretórios e
  substituições de processo (`<(...)`) são recusados: grave os dados em um arquivo antes.
* Os arquivos de projeto são JSON. Um projeto com chaves desconhecidas é **aceito** e as chaves desconhecidas são reportadas como
  `warning: unknown field bogus_key (ignored)` por `check` (e por `lint`, `studio`). Os arquivos auxiliares que você escreve para um comando
  (contracts, mapeamentos, máquinas, contêineres, regiões de rolagem, manifestos de ci, mapas de drive ...) são analisados de forma **estrita**: uma chave desconhecida
  é um erro e nada é aplicado (`unknown field(s): elastic.damping (typo? nothing was applied)`).
* Um projeto é validado sempre que é carregado para renderização ou exportação (`<file> is not a valid project:` seguido de
  um problema por linha). Os comandos de edição também recompilam o projeto que estão prestes a gravar e se recusam a gravar um que deixou de ser válido.
* Os **caminhos de assets** dentro de um projeto (arquivos de som, arquivos de imagem, normal maps) são resolvidos em relação ao **diretório do
  arquivo de projeto**. Caminhos absolutos e qualquer caminho que contenha `..` são recusados, a menos que `--allow-external-assets` seja informado:
  ```
  $ osatura sound add s.json o.json x /etc/passwd
  sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)
  ```
  O flag deve ser informado novamente em cada comando posterior que carregue esse projeto (`check ext.json` falha sem ele e funciona com ele).

### 1.4 Gravação de arquivos de saída

* **Gravações atômicas.** A maioria dos comandos grava por meio de um arquivo temporário no **mesmo diretório** (`.<name>.tmp-<pid>-<random>`), executa `fsync` nele,
  copia as permissões do arquivo antigo e faz `rename` sobre o destino, de modo que uma falha ou um disco cheio nunca deixa um arquivo pela metade.
  Se não for possível criar um arquivo temporário (diretório somente leitura, mas arquivo gravável), o arquivo é sobrescrito no local, o que **não** é atômico.
  FIFOs e arquivos especiais como `/dev/stdout` são gravados diretamente.
* **Links simbólicos** como saída são recusados, a menos que `--follow-symlinks` seja informado (então o destino do link é gravado; um link quebrado continua sendo um erro).
  Um diretório informado como arquivo de saída é um erro (`is a directory`).
* **Nunca sobre a entrada.** `render`, `render-hdr`, `dof`, `dof-export`, `export` (também o sidecar `.json` de sprite sheet de um `.png`
  e `--report`), `layout-preview`, `render-gpu`, `export-fit`, `fuzz-grid`, `onion`, `trail` e `golden` recusam uma saída que seja o mesmo arquivo
  que o projeto de entrada: `refusing to overwrite the input project demo.json with output of the same name`.
* Os **comandos de edição** (os que têm um par `<project.json> <out.json>`: `reparent`, `keys`, `physics`, ...) podem gravar **no próprio local**
  (`osatura reparent p.json p.json hand none` funciona e substitui `p.json`). `import-lottie` recusa uma saída (ou `fx.json`) que seja
  o mesmo arquivo que a sua entrada; os demais comandos de importação leem um tipo de arquivo diferente do que gravam.
* **Saídas existentes são sobrescritas silenciosamente** por todos os comandos. Não existe `--force`.
* **Toda saída endereçada pelo usuário** (incluindo relatórios e goldens de `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, os arquivos `face-*`,
  arquivos de pose, snapshots, `vcs` e o log de colaboração do estúdio) passa pelo mesmo gravador atômico que recusa links simbólicos. As
  saídas `.mp4`/`.webm` gravadas pelo ffmpeg são verificadas quanto a link simbólico antes de o ffmpeg iniciar. Pastas pai: `lod`, `export-fit`, `ci` e `face-*`
  criam as pastas ausentes; `fuzz`, `fuzz-grid` e `export` não criam (`No such file or directory`).

### 1.5 stdout, stderr e códigos de saída

* Os resultados ("wrote out.json", tabelas, JSON) vão para a **stdout**. Os erros e a maioria dos avisos vão para a **stderr**. `import-svg` imprime seus
  avisos na stdout; `export` imprime os avisos de Lottie/GIF/GLB na stderr.
* O código de saída **0** significa sucesso. O código de saída **1** significa qualquer erro *ou* uma verificação reprovada: um arquivo ilegível/inválido, argumentos incorretos,
  `lint` com erros, `diff` com diferenças, um contract violado, um golden divergente, uma taxa de falha de fuzz acima do limite,
  um orçamento de `export-fit` que não pôde ser cumprido, `lod --verify` acima do limite, divergência de `collab-sim`, uma verificação de `ci` reprovada.
  O código de saída **2** é usado somente por `ci` (nada pôde ser avaliado: projeto/manifesto ausente, golden adulterado).
* Sem argumentos, com um comando desconhecido, com o número errado de argumentos posicionais ou com `osatura --help`, o uso completo é impresso na
  **stderr** e o exit é **1**. `osatura <command> --help` imprime as linhas de uso desse comando na stdout e termina com exit 0.
  **Exceção (Windows, desde 0.1.0-alpha.2):** quando `osatura.exe` é iniciado por um duplo clique no Explorer (a janela de console foi criada para ele e nenhum outro processo a compartilha) e não tem argumentos, ele não imprime o uso: cria `Documents\OSATURA\first-rig.json` (a partir de `demo-person`) se não existir, executa `studio` sobre esse arquivo em `127.0.0.1:8080`, abre o navegador e mantém o console aberto até você fechá-lo ou pressionar Enter após um erro. Iniciado a partir de um terminal, ele se comporta como descrito acima. O restante do programa roda em uma thread com 256 MiB de pilha (a thread principal do Windows tem apenas 1 MiB).

### 1.6 Programas externos

O `ffmpeg` (encontrado no `PATH`) é necessário para: exportação `.mp4`/`.webm` e `dof-export` para esses formatos, `sound mix`,
áudio em `.html` (mp3) e `face-track` em um arquivo de vídeo. Encoders necessários: `libx264` (mp4), `libvpx-vp9` / `libaom-av1` (webm),
`aac` (som em mp4), `libopus` (som em webm), `libvorbis`, `libmp3lame` (sound mix). Nenhum código nem binário do ffmpeg é incluído; veja
[PATENT_NOTES.md](PATENT_NOTES.md). Nada mais é executado.

### 1.7 Observação sobre desempenho

Todos os tempos neste documento vêm de uma build **debug** e são 10 a 30 vezes mais lentos que uma build release. Os comandos que processam muitos quadros
(`safety`, `golden`, `ci`, `export-fit`, `proxy-bench`) foram executados em uma cópia reduzida de 8 quadros do demo (`small.json`, criada definindo
`"frames": 8`).

---

## 2. Primeiros passos: o projeto demo

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura export demo.json demo.gif
wrote demo.gif
```

O demo tem os ossos `arm_up arm_low hand thigh shin foot_ctrl`, uma cadeia de IK `leg` e as camadas de nível superior
`sky sun hills ground shadow arm leg target blush mouth(switch: closed/open/smile) ink`. Todos os exemplos desta referência o usam,
salvo indicação em contrário.

---

## 3. Criação e inspeção de projetos

### `demo`

**Sinopse:** `osatura demo <out.json>`

Grava o rig demo embutido (640x480, 24 fps, 48 quadros, 6 ossos, perna com IK, 14 camadas) como um arquivo de projeto.
**Saída:** `wrote <out.json>`. **Erros:** argumento ausente (uso, exit 1); caminho não gravável. **Relacionados:** `demo-fx`, `check`.
**Limites:** o demo é um asset fixo; é a base de todos os exemplos deste documento.

```
$ osatura demo demo.json
wrote demo.json
```

### `demo-person`

**Sinopse:** `osatura demo-person <out.json>`

Grava a demonstração da pessoa caminhando (640x480, 24 fps, 32 quadros, 42 ossos, 45 camadas) como um arquivo de projeto: uma pessoa vista de lado com cabeça, orelha e rabo de cavalo, pescoço, tronco, dois braços com mãos e cinco dedos cada um, e duas pernas com pés e dedos dos pés. O ciclo de caminhada tem dois passos e se repete sem emenda: a pelve oscila e se inclina, o tronco se inclina e gira em sentido contrário, a cabeça é mantida nivelada com um pequeno aceno atrasado, os braços balançam contra as pernas com os cotovelos dobrando mais no balanço para a frente, e mãos, dedos, orelha e rabo de cavalo seguem um pouco depois. Cada pé pousa sobre o calcanhar, rola até ficar plano e sai pela ponta do pé; o chão, a cerca e a sombra rolam exatamente na velocidade do pé apoiado, de modo que os pés não deslizam. É o projeto que os inicializadores do Windows (`osatura.exe`, `osatura-studio.exe`) criam como `first-rig.json` na primeira execução; `osatura demo` continua sendo a pequena base dos exemplos deste documento.
**Saída:** `wrote <out.json>`. **Erros:** argumento ausente (uso, exit 1); caminho não gravável. **Relacionados:** `demo`, `studio`.
**Limites:** as chaves são amostradas a cada dois quadros, com uma chave extra antes do quadro 0 e outra depois do último quadro, para que o loop seja suave; `lint` informa essas duas chaves como fora da linha do tempo (informação, não erro). A figura é uma vista lateral, portanto não há giro nem profundidade.

```
$ osatura demo-person person.json && osatura check person.json
wrote person.json
ok: 'OSATURA walking person' 640×480 @ 24 fps, 32 frames, 42 bones, 0 ik, 45 layers, 48 channels
```

### `demo-fx`

**Sinopse:** `osatura demo-fx <out.json>`

Grava o demo "liquid / boolean / particles" (640x480, 72 quadros, 3 ossos, 3 camadas) que exercita as camadas procedurais.

```
$ osatura demo-fx fx.json && osatura check fx.json
wrote fx.json
ok: 'OSATURA fx demo' 640×480 @ 24 fps, 72 frames, 3 bones, 0 ik, 3 layers, 3 channels
```

### `check`

**Sinopse:** `osatura check <project.json>`

Carrega e valida um projeto. Imprime linhas `warning:` (campos desconhecidos, problemas de gradiente, problemas de switch-key e os avisos de dados do editor do código de lint L013: zoom da câmera <= 0, opacidade de camada fora de 0..1, referências ou tempos inválidos em skins, comps, markers, events, shots, channel_meta, bone_vis, clips e lods) e uma linha de resumo `ok:`. Apenas valores que não podem existir em um arquivo salvo (NaN ou infinito nessas estruturas) são erros, de modo que projetos mais antigos continuam carregando.
Faixas de som também são abertas: um arquivo de áudio ilegível/inutilizável é um **erro**. **Exit:** 0 válido; 1 inválido.

```
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check unk.json                      # a project with two unknown keys
warning: unknown field bogus_key (ignored)
warning: unknown field bones[0].lenght (ignored)
ok: 'small' 640×480 @ 24 fps, 8 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check bad3.json                     # IK chain whose target bone does not exist
bad3.json is not a valid project:
  - ik 'leg': unknown target bone 'nope'
[exit 1]

$ osatura check bad.json                      # {"format":1,"name":"x"}
invalid JSON in bad.json: missing field `width`
[exit 1]
```

**Relacionados:** `lint` (qualidade de rig/animação), `ci` (portão de regressão). **Pegadinha:** `check` não renderiza nada; um projeto pode passar em `check` e ainda assim
falhar depois em um quadro (poses `NaN` são capturadas por `render`/`export`, que avaliam todos os quadros antes).

### `stats`

**Sinopse:** `osatura stats <project.json>`

Imprime contagens como JSON: `name, size, fps, frames, seconds, bones, ik_chains, constraints, springs, layers, channels, keyed_channels, keys, markers`.
A saída é sempre JSON; não existe a opção `--json` (ela é recusada como qualquer opção desconhecida).

```
$ osatura stats demo.json
{ "name": "OSATURA demo", "size": [640.0, 480.0], "fps": 24.0, "frames": 48, "seconds": 2.0, "bones": 6,
  "ik_chains": 1, "constraints": 0, "springs": 0, "layers": 14, "channels": 14, "keyed_channels": 8, "keys": 37, "markers": 0 }
```
(a saída real é formatada em várias linhas.) Note que `layers` conta todos os nós, incluindo os filhos de grupos/switches, e que `check` imprime o mesmo número.

### `diff`

**Sinopse:** `osatura diff <a.json> <b.json>`

Diferença semântica entre dois projetos: `+` adicionado, `-` removido, `~` alterado. Não depende da ordem nas listas endereçadas por nome.
**Exit:** 0 idênticos (`no differences`); **1** quando há diferenças (a contagem `N difference(s)` vai para a stderr), de modo que pode servir de portão em um script.

```
$ osatura diff demo.json demo.json
no differences
$ osatura diff demo.json d2.json              # fps 24->30, arm_up longer, layer 'sun' deleted
~ fps: 24 → 30
~ bone 'arm_up' changed (length)
- layer 'sun' removed
3 difference(s)            (stderr)
[exit 1]
```
**Relacionados:** `vcs diff` (histórico), `compare` (diferença de pixels), `ci diff` (diferença de imagens).

### `tag`

**Sinopse:** `osatura tag <project.json>`

Propõe nomes de papéis padrão (`arm.upper.l`, `hand.l`, `face.mouth` ...) para ossos e camadas a partir dos seus nomes, com um grau de confiança, e lista os papéis padrão para os quais nenhum osso foi encontrado. Somente leitura; use os papéis como guia para `mocap-match` ou `autorig`.

```
$ osatura tag demo.json
bone  arm_up                   -> arm.upper.l    (45%)
bone  hand                     -> hand.l         (45%)
layer mouth                    -> face.mouth     (90%)
no bone found for: head, neck, torso, hips, face.eye.l, ...
```

### `explain`

**Sinopse:** `osatura explain <project.json> <bone-or-layer> <frame>`

"Por que está aqui?" - lista as contribuições ordenadas (pose de repouso, cada canal com seu modo e peso, constraints, IK) que produzem a transformação de um osso ou camada em um quadro, e o ângulo resultante de cabeça/ponta/mundo.

```
$ osatura explain demo.json hand 12
- bone 'hand' (length 45), parent 'arm_low'
- ancestry: hand ← arm_low ← arm_up
- rest pose: offset (100.00, 0.00), angle 0.00°
- channel #2 BoneAngle: keys → 20.000 (5 key(s), Hold outside range); mode replace, weight 1.00
- channel #3 BoneAngle: expression `6 * sin(time * 2 * pi * 2)`; mode add, weight 1.00
- result at frame 12: head (174.6, 126.4), tip (203.5, 92.0), world angle -50.0°
```
**Erros:** nome de osso/camada desconhecido; quadro fora da animação.

### `pick`

**Sinopse:** `osatura pick <project.json> --frame N --x X --y Y [--tol PX] [--root] [--bones] [--points a,b] [--locked a,b] [--json]`

O que há sob um ponto do canvas, do mais acima para o mais abaixo (teste de colisão como o estúdio o faz).

| Opção | Significado |
|---|---|
| `--frame N`, `--x X`, `--y Y` | Obrigatórios: quadro e coordenadas do canvas. |
| `--tol PX` | Tolerância de seleção em pixels do canvas (padrão 3). |
| `--root` | Reporta o ancestral de nível superior (`mouth`) em vez da camada mais interna (`mouth/closed`). |
| `--bones` | Considera também os ossos. |
| `--points a,b` | Camadas cujos pontos de controle (âncoras/alças/vértices de malha) podem ser selecionados. |
| `--locked a,b` | Camadas a ignorar (tratadas como bloqueadas). |
| `--json` | `{"hit":{...},"hits":[...]}`. |

```
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4
hit: layer ink (distance 0.00)
  under: layer mouth/closed (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4 --root --locked mouth
hit: layer ink (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 300 --y 240 --json
{"hit":{"distance":0.0,"target":{"kind":"layer","path":"sky"}},"hits":[{"distance":0.0,"target":{"kind":"layer","path":"sky"}}]}
```
**Pegadinha:** a camada de fundo (`sky`) é um alvo de seleção normal, portanto quase sempre há pelo menos um resultado.

### `select-box`

**Sinopse:** `osatura select-box <project.json> --frame N (--rect x0,y0,x1,y1 | --lasso x,y;x,y;x,y) [--mode bbox|contain|intersect] [--root] [--bones] [--json]`

Seleção por retângulo/laço. `--mode` decide quando uma camada conta como dentro: `bbox` (seu retângulo delimitador toca a região), `contain` (totalmente dentro), `intersect` (padrão: sua geometria desenhada intersecta a região). `--root` reporta os ancestrais de nível superior, `--bones` inclui os ossos.
Coloque o valor de `--lasso` entre aspas em um shell (o `;` separa os pontos).

```
$ osatura select-box demo.json --frame 0 --rect 200,95,220,115 --root
layer sky
layer arm
layer mouth
layer ink
$ osatura select-box demo.json --frame 0 --rect 0,0,640,480 --mode contain
layer sun
layer shadow
...
$ osatura select-box demo.json --frame 0 --lasso "100,100;300,100;300,300" --bones --json
{"targets":[{"kind":"bone","name":"arm_low"},{"kind":"bone","name":"hand"},{"kind":"layer","path":"sky"}, ...]}
```

### `timeline`

**Sinopse:** `osatura timeline <project.json> [--json]`

A visão de dope sheet em texto: trilhas por canal agrupadas por osso/camada, com os quadros-chave (`keys @ ...`), canais `expr` / `driven`, bloqueios e cores. Os números `#N` são os índices de canal usados por `keyedit --channels` e pelas seleções `#N@F0..F1`.

```
$ osatura timeline demo.json
timeline: 48 frames at 24 fps, 14 channels
Bones
  arm_up
    #0 bone_angle (keys @ 0 12 24 36 48)
    arm_low
      #1 bone_angle (keys @ 0 12 24 36 48)
      hand
        #2 bone_angle (keys @ 0 12 24 36 48)
        #3 bone_angle (expr)
  foot_ctrl
    #4 bone_offset_x (keys @ 0 12 24 36 48)
...
```
`--json` fornece grupos com ids, pais, filhos, trilhas, `frames`, `locked`.

### `timecode`

**Sinopse:** `osatura timecode <frame|HH:MM:SS;FF> [--fps 24] [--drop] [--nodrop]`

Conversão de timecode SMPTE. Um número simples é um quadro e imprime um timecode; um argumento que contém `:` ou `;` é interpretado e imprime o número do quadro.
29,97, 59,94 e 119,88 fps usam timecode drop-frame (`;` antes dos quadros) por padrão; `--nodrop` conta 30 (ou 60/120) em vez disso; `--drop` força o drop-frame (erro para outras taxas).
`--drop` e `--nodrop` são mutuamente exclusivos.

```
$ osatura timecode 100
00:00:04:04
$ osatura timecode 100 --fps 29.97
00:00:03;10
$ osatura timecode 1800 --fps 29.97 --drop
00:01:00;02
$ osatura timecode 1800 --fps 29.97 --nodrop
00:01:00:00
$ osatura timecode 00:00:04:04
100
$ osatura timecode 00:00:04;04 --fps 24
drop-frame timecode is only defined for 29.97, 59.94 and 119.88 fps         [exit 1]
```
Não lê um projeto; `--fps` tem como padrão 24 (e não o fps de um projeto).

### `snapshot`

**Sinopse:**
```
osatura snapshot save <project.json> <dir> [--keep 20]
osatura snapshot list <project.json> <dir>
osatura snapshot restore <project.json> <dir> <out.json> [--number N]
osatura snapshot recover <project.json> <dir> <out.json>
```
`--keep` pertence somente a `save` e `--number` somente a `restore`; qualquer outra combinação é recusada (`unknown option --keep for 'snapshot list' (applies to: snapshot save)`), assim como um argumento de arquivo ausente ou extra.

Um histórico numerado de um projeto em um diretório (independente de `vcs`). `save` armazena o projeto atual como `rev-NNNNNN-<hash>.json` e mantém apenas as `--keep` revisões mais recentes (padrão 20, os arquivos mais antigos são excluídos); salvar um projeto idêntico à última revisão imprime `unchanged since the last revision` e não armazena nada. `list` imprime as revisões. `restore` grava a revisão `--number N` em `<out.json>`; `recover` grava a revisão mais recente.

```
$ osatura snapshot save s.json snaps --keep 3
saved revision 1 (4922f48a86f24554)
$ osatura snapshot list s.json snaps
     1  4922f48a86f24554  snaps/rev-000001-4922f48a86f24554.json
$ osatura snapshot restore s.json snaps restored.json --number 1
wrote restored.json
$ osatura snapshot recover s.json snaps recovered.json
wrote recovered.json
```
`list` em um diretório inexistente não imprime nada e termina com exit 0. **Relacionados:** `vcs` para branches e merges.

---

## 4. Renderização e exportação

### `render`

**Sinopse:** `osatura render <project.json> <frame> <out.svg|out.png> [--bones] [--scale N] [--clamp] [--proxy N]`

`--proxy N` renderiza um proxy de scrub (N = 1..4): um `.png` é um bitmap com resolução 1/2^N, um `.svg` são vetores simplificados; ambos descartam traços finos, gradientes e blends.

Renderiza um quadro. Um `<frame>` igual ou superior à duração do projeto é um erro (`frame 99 is outside the animation ... valid frames are 0..N-1 ... pass --clamp`); `--clamp` renderiza o último quadro e imprime um aviso na stderr. (Somente `render` tem esta verificação; `dof`, `render-hdr` e `fx` não têm.) A extensão escolhe o formato (`.svg` vetorial, `.png` raster RGBA de 8 bits); qualquer outra extensão é recusada **antes** de qualquer gravação.

| Opção | Padrão | Significado |
|---|---|---|
| `--bones` | desligado | Desenha a sobreposição do esqueleto (cabeças como pontos, ossos como linhas). |
| `--scale N` | 1 | Escala de saída, um número em (0, 16]. Para `.png` multiplica o tamanho em pixels; para `.svg` o documento mantém seu viewBox. |
| `--proxy N` | desligado | Proxy de scrub, N = 1..4: `.png` = bitmap com resolução 1/2^N, `.svg` = vetores simplificados; ambos descartam traços finos, gradientes e modos de mesclagem. Pensado para scrubbing interativo, não para entregas finais. |

```
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura render demo.json 0 /tmp/x.xyz
render writes .png or .svg; '/tmp/x.xyz' has another extension (nothing was written)       [exit 1]
$ osatura render demo.json 0 o.svg --scale 0
--scale needs a number in (0, 16]                                                         [exit 1]
$ osatura render demo.json abc o.svg
bad frame number 'abc'                                                                    [exit 1]
$ osatura render demo.json 2 px.png --proxy 2        # 640x480 project -> 160x120 PNG
wrote px.png
```
**Erros:** projeto ausente/inválido, pose não finita nesse quadro, saída igual ao projeto de entrada. **Relacionados:** `frames`, `render-hdr`, `onion`, `compare`, `fx`.
**Limites:** o rasterizador de PNG é o próprio do motor (8 bits, sRGB; a mesclagem não tem gerenciamento de cor, a menos que o projeto opte por isso com `color-space ... linear`). Efeitos de camada, máscaras e modos de mesclagem são desenhados por ambos.

### `frames`

**Sinopse:** `osatura frames <project.json> <out_dir> [--png] [--bones] [--scale N]`

Renderiza **todos** os quadros em um diretório (criado se não existir) como `frame_0000.svg` ... (ou `.png` com `--png`). Antes, verifica-se que todos os quadros têm poses finitas.

```
$ osatura frames demo.json fr --png
wrote 48 frames to fr
$ ls fr | head -2
frame_0000.png
frame_0001.png
```
**Pegadinha:** arquivos existentes com os mesmos nomes são sobrescritos; arquivos não relacionados no diretório são deixados em paz. Use isto como entrada para `ffmpeg` ou `import-seq`.

### `export`

**Sinopse:**
```
osatura export <project.json> <out.gif> [--scale N] [--report r.json]
osatura export <project.json> <out.html|out.htm>
osatura export <project.json> <out.json|out.lottie> [--report r.json]
osatura export <project.json> <out.mp4> [--scale N]
osatura export <project.json> <out.apng> [--scale N]
osatura export <project.json> <out.webm> [--scale N] [--codec vp9|av1]
osatura export <project.json> <out.png> [--scale N] [--columns N]
osatura export <project.json> <out.glb> [--report r.json]
```
(uma forma por formato; a extensão de saída, sem distinção entre maiúsculas e minúsculas, escolhe a forma. Uma opção que a forma não lista é recusada: `unknown option --scale for 'export .json' (applies to: export .gif, export .mp4, ...)`. Com uma extensão desconhecida ou ausente, é reportado o erro de formato.)

A extensão de saída seleciona o formato:

| Extensão | Resultado | Precisa de ffmpeg | Opções aplicáveis |
|---|---|---|---|
| `.gif` | GIF em loop na taxa de quadros do projeto (veja [EXPORT_FORMATS.md](EXPORT_FORMATS.md)) | não | `--scale`, `--report` |
| `.html`, `.htm` | um player HTML autocontido (quadros SVG embutidos, play/pause, scrubber, áudio embutido opcional) | somente para áudio | - |
| `.json` | Lottie (Bodymovin) | não | `--report` |
| `.lottie` | contêiner dotLottie (zip: `manifest.json` + `a/animation.json`) | não | `--report` |
| `.mp4` | H.264 yuv420p (+ AAC quando o projeto tem sons) | sim (`libx264`, `aac`) | `--scale` |
| `.apng` | PNG animado | não | `--scale` |
| `.webm` | VP9 (padrão) ou AV1 com alfa (+ som Opus) | sim (`libvpx-vp9` / `libaom-av1`, `libopus`) | `--scale`, `--codec vp9\|av1` |
| `.png` | sprite sheet **mais** metadados `<out>.atlas.json` (nunca `<out>.json`) | não | `--scale`, `--columns N` |
| `.glb` | glTF 2.0 binário (ossos, animação, preenchimentos/traços com skin, gradientes, máscaras, imagens, morph targets) | não | `--report` |

Opções:

| Opção | Significado |
|---|---|
| `--scale N` | Escala do raster (0, 16] para `.gif .mp4 .apng .webm .png`. Nos outros formatos é recusada (`unknown option --scale for 'export .json'`); antes da wave 7 era um aviso. |
| `--report FILE` | Grava como JSON a lista do que o formato não pôde transportar (`.gif .json .lottie .glb`): `{"format","count","warnings":[{"code","layer","message"}]}`. Os mesmos avisos são sempre impressos na stderr. O arquivo não pode ser o projeto de entrada. |
| `--codec vp9\|av1` | Somente `.webm` (padrão `vp9`); qualquer outro valor: `unknown --codec 'foo' (vp9 or av1)`. |
| `--columns N` | Somente sprite sheet `.png`: blocos por linha (padrão: uma grade quase quadrada, ceil(sqrt(frames)): 3 colunas para 8 quadros). |

Exemplos (todos verificados em uma cópia de 8 quadros do demo, `--scale 0.5`):

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ cat rep.json
{ "count": 2, "format": "lottie", "warnings": [ {"code": "variable-width-stroke", "layer": "ink", ...}, {"code": "gradient-anisotropic", ...} ] }
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.json
wrote out3.png                          # 4 columns x 2 rows of 160x120 tiles; out3.json lists {index,x,y,w,h} per frame
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
$ osatura export small.json out.mp4 --scale 0.5       # 320x240 h264 yuv420p, 8 frames
wrote out.mp4
$ osatura export small.json out.webm --codec av1      # needs a libaom-av1 build of ffmpeg
```

Erros e recusas:

```
$ osatura export small.json z.xyz
unknown export format '.xyz' (use .gif .html .htm .json .lottie .mp4 .apng .webm .png .glb)
$ osatura export small.json small.json
refusing to overwrite the input project small.json with output of the same name
$ osatura export ora.json ora.glb                       # a project of image layers and no bones
the project has no bones to export
$ PATH=/nonexistent osatura export small.json y.mp4
cannot run ffmpeg (...); install it or export a GIF/HTML instead                          [exit 1, nothing written]
```
Outras recusas: `ffmpeg failed` (encoder ausente), `export would write about N million values ...` (a proteção de tamanho de Lottie/HTML/GLB: o limite é 40 milhões de valores = quadros x (âncoras + ossos + camadas); encurte a animação ou simplifique os caminhos), poses não finitas em qualquer quadro e `image too large for GIF`.

**Saídas / efeitos colaterais:** `.png` também grava os metadados do sprite sheet em `<out-name>.atlas.json` (p. ex. `out.png` -> `out.atlas.json`), de modo que não pode mais substituir um arquivo Lottie `out.json`; o arquivo sidecar está sujeito às mesmas proteções de entrada/link simbólico. `.mp4`/`.webm` renderizam os quadros PNG em um diretório temporário privado que é removido depois.
**Som:** `.mp4` e `.webm` mixam as faixas de som do projeto (início, ganho, corte, fades; cortadas na duração do projeto); `.html` embute um mp3 mixado (aviso acima de 3 MiB, descartado acima de 16 MiB com um aviso); `.gif` e `.apng` imprimem `note: GIF cannot carry audio; ...` e ficam silenciosos; o Lottie avisa `[audio]`.
**Transparência:** um projeto com `"background": null` é renderizado com pixels transparentes onde nada é desenhado. `.apng`, o sprite sheet `.png` e `.webm` (`yuva420p`, tag de contêiner `alpha_mode=1`) mantêm o canal alfa; `.gif` mantém transparência de 1 bit (verificado: o flag de cor transparente está definido); `.mp4` não tem alfa. Um projeto com cor de fundo (o demo tem `#f6f1e7`) é opaco.
**Relacionados:** `export-fit` (ajustar a um orçamento de bytes), `size-report`, `lod`, `dof-export`, `export-oca`, `render-hdr`. Detalhes completos dos formatos: [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

### `export-oca`

**Sinopse:** `osatura export-oca <project.json> <out_dir> [--layers]`

Grava uma pasta **Open Cel Animation**: `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) mais cels PNG `layerN/fNNNN.png`. Sem `--layers` há uma única camada `render` com os quadros compostos; com `--layers`, cada camada de **nível superior** é renderizada isoladamente sobre transparência em sua própria camada OCA (11 camadas para o demo). `--scale N` multiplica o tamanho dos cels.

```
$ osatura export-oca small.json oca2 --layers
wrote oca2/small.oca
$ ls oca2
layer0 layer1 ... layer10 small.oca
```
**Limites:** os cels são rasters (nenhum vetor, osso ou canal sobrevive); a opacidade/mesclagem da camada não é gravada (toda camada é `paintlayer`, com opacidade 1); os sons não são exportados; a pasta é criada se necessário e os arquivos nela são sobrescritos.

### `render-hdr`

**Sinopse:** `osatura render-hdr <project.json> <frame> <out.exr|out.png> [linear] [--precision half|float] [--float32] [--8bit] [--scale N] [--clamp]`

Renderização em ponto flutuante (32 bits) (um `<frame>` fora do projeto é um erro, `--clamp` usa o quadro válido mais próximo, como em `render`): `.exr` grava um **OpenEXR** de uma só parte, por scanline, sem compressão (canais HALF por padrão, FLOAT com `--precision float` ou `--float32`), em luz linear e pré-multiplicado; `.png` grava um **PNG RGBA de 16 bits**, codificado em sRGB, a menos que a palavra literal `linear` seja dada como quinto argumento. O fundo do projeto **não** é desenhado (a imagem é transparente onde nada é desenhado). `--8bit` usa o rasterizador legado de 8 bits como fonte (não pode ser combinado com saída float32). `--scale N` se aplica.

```
$ osatura render-hdr small.json 3 f.exr
wrote f.exr (640x480)                       # 465,611 bytes: HALF
$ osatura render-hdr small.json 3 f2.exr linear --precision float
wrote f2.exr (640x480)                      # 923,211 bytes: FLOAT
$ osatura render-hdr small.json 3 f4.png
wrote f4.png (640x480)                      # PNG 16-bit/colour RGBA
$ osatura render-hdr small.json 3 f6.exr --precision bogus
unknown precision 'bogus' (use half or float)
$ osatura render-hdr small.json 3 f7.jpg
output must end in .exr or .png
```
O `ffprobe` lê o EXR como `exr 640x480 gbrapf32le`. Para `.exr`, a palavra `linear` não tem efeito (o EXR é sempre linear). **Limites:** sem compressão, um quadro por arquivo, sem camadas/AOVs; a arte de origem é cor sRGB comum, portanto o arquivo contém valores da faixa de exibição com maior precisão, não luz HDR referida à cena. Sem gerenciamento de cor além da conversão sRGB<->linear.

### `render-gpu`

**Sinopse:** `osatura render-gpu <project.json> <out.png> [--frame N] [--adapter software|auto] [--scale S] [--bench N]`

Renderização de um quadro na GPU (wgpu). Disponível apenas em builds feitas com `--features gpu`; a build padrão responde:

```
$ osatura render-gpu small.json g.png
render-gpu needs a build with the gpu feature: cargo build -p osatura-cli --features gpu       [exit 1]
```
Em uma build com GPU, cenas não suportadas (imagens, ossos) são reportadas como erros, nunca desenhadas incorretamente. `--bench N` é uma opção de benchmark. Este comando não pôde ser executado na build de teste (sem o recurso gpu), portanto seus flags foram tomados apenas do texto de uso.

### `compare`

**Sinopse:** `osatura compare <a.json> <b.json> <frame> <out.png> [--gain 4 --scale N]`

Grava um PNG: **antes | depois | diferença** lado a lado (o terceiro painel é um mapa de calor, preto = idêntico, vermelho -> amarelo = mudança maior, multiplicada por `--gain`, padrão 4). Os dois projetos devem ser renderizados no mesmo tamanho (`frame sizes differ (...); compare projects of the same size`).

```
$ osatura compare small.json d2.json 3 cmp.png --gain 4 --scale 0.25
wrote cmp.png
```
**Relacionados:** `diff` (estrutura), `ci diff` (dois PNGs, métricas perceptuais), `golden`.

### `proxy-bench`

**Sinopse:** `osatura proxy-bench <project.json> [--frames N]`

Milissegundos por quadro, em uma única thread, com qualidade total e em cada nível de proxy de scrub (veja `render --proxy`).

```
$ osatura proxy-bench small.json --frames 2
2 frames of small (640x480), ms per frame, single thread:
  evaluate only           0.08
  svg (no raster)         0.48   vector proxy: level 0 0.32, level 2 0.24, level 4 0.24
  raster level 0       1360.87   (full quality)
  raster level 1        319.14   (4.3x faster than level 0)
  ...
  raster level 4          6.84   (199.0x faster than level 0)
```
(números de build debug). 

---

## 5. Rigging, skins, IK e utilitários de layout

Todos os comandos desta seção seguem o padrão `osatura <cmd> <project.json> <out.json> ...`: leem um projeto, alteram-no, **revalidam-no**
e gravam `<out.json>` (que pode ser igual à entrada). Em caso de erro, nada é gravado.

### `autorig`

**Sinopse:** `osatura autorig <project.json> <out.json> [seed]`

Adiciona automações padrão deduzidas dos **nomes** de ossos/camadas: cadeias de IK (mix 0, para que o FK continue funcionando), um loop de respiração (precisa de um osso de torso) e piscadas (precisa de camadas de olhos). Imprime uma linha por decisão: `+` adicionado, `-` ignorado com o motivo. Também executa `eye-clip`.

```
$ osatura autorig small.json auto.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
- arm_r: upper/lower bone not found
- leg_l: already has an IK chain
- breathing: no torso bone
- blink: no eye layers
wrote auto.json
```
`seed` (posicional opcional) inicializa a temporização aleatória das piscadas. **Limites:** baseado puramente em nomes (`tag` mostra como os nomes são lidos); nunca cria ossos.

### `slider`

**Sinopse:** `osatura slider <project.json> <out.json> <name> <poses>`, em que `poses` é `at:target.prop=val,target.prop=val;at:...`

Cria um **osso controlador** `ctrl_<name>` cujo `offset_x` comanda várias propriedades entre poses armazenadas (um slider de poses). Anime o `bone_offset_x` do controlador impresso.

```
$ osatura slider small.json sl.json pose "0:arm_up.bone_angle=0;24:arm_up.bone_angle=30"
controller bone: ctrl_pose (animate its offset_x)
wrote sl.json
```
`target.prop` usa os nomes de propriedade em snake_case (`arm_up.angle` falha com `unknown property 'angle'`). O último ponto separa o alvo da propriedade, de modo que caminhos de camada como `grp/child.layer_opacity` funcionam.

### `view-swap`

**Sinopse:** `osatura view-swap <project.json> <out.json> <switch_layer> <bone.prop> <from:child,from:child,...>`

Comanda uma camada switch a partir de uma propriedade de osso com uma função em degraus: os pares `from:child` são ordenados por `from`, e o switch mostra o filho do último par cujo `from` não está acima do valor do driver (`-40:0,0:1,40:2` = filho 0 abaixo de 0, filho 1 a partir de 0, filho 2 a partir de 40). Use para vistas de turn-around (frente/lado/costas) comandadas por um osso de yaw. Os índices dos filhos devem existir (`child index 5 is out of range (the layer has 3)`); a camada deve ser um switch.

```
$ osatura view-swap small.json vs.json mouth foot_ctrl.bone_offset_x "-40:0,0:1,40:2"
wrote vs.json
```

### `align` e `distribute`

**Sinopse:**
`osatura align <project.json> <out.json> <mode> <layer,layer,...|layer> [stage]`  (mode: `left right top bottom center-x center-y`)
`osatura distribute <project.json> <out.json> <x|y> <gaps|centers> <layer,layer,...>`

`align` move camadas para que os retângulos delimitadores de suas curvas achatadas se alinhem (entre si ou, para uma única camada seguida da palavra `stage`, ao canvas). `distribute` espaça três ou mais camadas uniformemente ao longo de um eixo, igualando os **gaps** (espaços) entre os retângulos ou os **centers** (centros).

```
$ osatura align small.json al.json center-x sun,hills stage
wrote al.json
$ osatura align small.json al2.json top ground
select at least two layers (or one with --stage)                 [exit 1]
$ osatura distribute small.json di.json x gaps sun,ground,hills
wrote di.json
```
Estes comandos editam posições de camada (transformação estática), portanto canais com keys em `layer_x`/`layer_y` ainda as sobrepõem. (A mensagem de erro menciona `--stage`; o argumento real é a palavra simples `stage`.)

### `trim`

**Sinopse:** `osatura trim <project.json> <out.json> <layer> <path_index> <start> <end> [offset]`

Faz o bake de um *trim path*: substitui o caminho número `path_index` de uma camada vetorial apenas pela parte entre `start` e `end` (frações de 0..1 do seu comprimento, ambas deslocadas por `offset`; caminhos fechados dão a volta). O resultado é uma **polilinha aberta com o mesmo traço e sem preenchimento** - destrutivo, portanto guarde o original. Para um efeito de draw-on, faça o bake de uma cópia aparada por quadro.

```
$ osatura trim small.json tr.json ink 0 0 0.5 0.25
wrote tr.json
```

### `reparent`

**Sinopse:** `osatura reparent <project.json> <out.json> <bone> <new_parent|none>`

Muda o pai de um osso **sem movê-lo**: o deslocamento/ângulo de repouso são recalculados para que a pose de repouso permaneça inalterada (verificado: depois, apenas `parent`, `rest_angle`, `rest_offset` diferem). `none` o torna uma raiz. Um pai que criaria um ciclo é recusado.

```
$ osatura reparent small.json rp.json hand none
wrote rp.json
$ osatura diff small.json rp.json
~ bone 'hand' changed (parent, rest_angle, rest_offset)
```

### `parent-switch`

**Sinopse:** `osatura parent-switch <project.json> <out.json> <bone> <frame:parent|none,...>`

Faz o bake de re-parenting animado (uma mão agarrando um objeto): em cada quadro de troca, a pose de mundo do osso em relação ao novo pai é capturada e mantida (seguimento rígido), ou congelada no espaço do mundo para `none`. O osso se torna uma raiz e sua pose de mundo é gravada como keys simples de deslocamento/ângulo para todos os quadros; seus canais antigos permanecem no arquivo, mas são sobrepostos pelos gerados no bake. **Limites:** recusado para ossos envolvidos em IK, constraints ou ossos de mola; a escala não é transportada.

```
$ osatura parent-switch demo.json ps.json hand 0:none,24:arm_up
wrote ps.json
```

### `skin`

**Sinopse (subcomandos):**
```
osatura skin <project.json> <out.json> add <group> <name> <layer,layer,...>
osatura skin <project.json> <out.json> rig <group> <name> <constraint,...|-> <ik,...|->
osatura skin <project.json> <out.json> bones <group> <name> <bone,...|->
osatura skin <project.json> <out.json> apply <group> <name>
osatura skin <project.json> <out.json> schedule <group> <frame:name,frame:name,...>
```

Um *conjunto de skins* é identificado por um rótulo `<group>` (um conjunto de "roupas" ou de "expressões"; no exemplo, ele coincide com o nome de um grupo de camadas). Cada skin nomeada mostra um conjunto de camadas, dadas por **caminho de camada** (p. ex. `skins/box`); aplicar uma skin mostra suas camadas e oculta as camadas de todas as outras skins do mesmo grupo.
`add` define uma skin; `rig` lista constraints e cadeias de IK que só ficam ativas enquanto aquela skin estiver ativa; `bones` lista ossos que só existem enquanto ela está ativa (colapsados para escala 0 caso contrário); `apply` torna uma skin ativa de forma estática; `schedule` anima a skin ativa ao longo do tempo como keys em degrau na **opacidade da camada** (portanto não anime de outro modo a opacidade dessas camadas). `-` significa "nenhum".

```
$ osatura skin sc.json sk1.json add skins red skins/box
wrote sk1.json
$ osatura skin sk1.json sk2.json add skins green skins/box2
$ osatura skin sk2.json sk3.json apply skins green
$ osatura skin sk3.json sk4.json schedule skins 0:red,4:green
$ osatura skin sk4.json sk5.json bones skins red -
$ osatura skin sk4.json sk5.json rig skins red - leg
$ osatura skin sc.json x.json add skins red box
no layer 'box'                                                  [exit 1]     (use the full path skins/box)
```
**Limites (ROADMAP §9):** um osso que existe apenas em uma skin nunca é removido do rig avaliado (os índices permanecem; ele é escalado a 0); uma camada com pesos para um osso exclusivo de uma skin, mas que não está nessa skin, desaparece junto com ele, e isso não é verificado.

### `bone-vis`

**Sinopse:** `osatura bone-vis <project.json> <out.json> <bone> <from> <to>|clear`

Mostra o osso nos **overlays** do estúdio apenas em um intervalo de quadros (controle de desordem visual). Não altera a renderização nem a exportação. `clear` remove o intervalo (`range removed`).

### `chan-meta`

**Sinopse:** `osatura chan-meta <project.json> <out.json> <target> <prop|*> [color=#rrggbb] [shy=on|off] [solo=on|off] [lock=on|off]`

Metadados de dope sheet de um canal: cor de exibição, *shy* (oculto nas listas), *solo* (visualizar apenas estes canais no estúdio) e **lock** (recusar edições: canais bloqueados são ignorados por `keys`, recusados por `keyedit`, `audio-react` e `live-map`, e nunca tocados por `clip bake`). A avaliação e a exportação ignoram os quatro. `*` seleciona todas as propriedades do alvo.

```
$ osatura chan-meta small.json cm.json arm_up bone_angle 'color=#ff0000' shy=on lock=on
wrote cm.json
```

### `reference`

**Sinopse:** `osatura reference <project.json> <out.json> <layer> <on|off>`

Marca uma camada como **referência de traçado**: mostrada no estúdio, nunca exportada.

### `parallax`

**Sinopse:** `osatura parallax <project.json> <out.json> <far_layer,...,near_layer> <far_value> <near_value>`

Atribui profundidade multiplano (valores de `parallax` distribuídos entre `far_value` e `near_value`) às camadas de nível superior listadas, na ordem dada (a primeira = a mais distante). Com movimento de câmera, as camadas se deslocam em quantidades diferentes; `dof` desfoca por `|parallax - focus|`.

```
$ osatura parallax small.json pa.json sky,hills,ground 0.2 1.0
wrote pa.json
```

### `eye-clip`

**Sinopse:** `osatura eye-clip <project.json> <out.json>`

Mascara cada camada de íris/pupila pela camada de formato do olho do mesmo lado (encontrada pelo nome, veja `tag`) usando uma máscara de camada comum, de modo que as pupilas nunca saiam do olho enquanto se movem. Também executado por `autorig`. Quando nenhum nome correspondente é encontrado, o arquivo ainda é gravado, sem alterações.

### `comp`

**Sinopse:**
```
osatura comp capture|apply <project.json> <out.json> <name>
osatura comp list <project.json>
```

Estados de camada nomeados ("comps": visibilidade, opacidade e o filho selecionado das camadas switch, por caminho de camada; **não** transformações). `capture` armazena o estado atual como `name`, `apply` o restaura, `list` imprime os nomes (`list` não recebe arquivo de saída e não grava nada; uma quarta palavra legada, `comp list p.json p.json`, ainda é aceita e ignorada).

```
$ osatura comp capture small.json c1.json pose1
wrote c1.json
$ osatura comp list c1.json c1.json
pose1 (14 layers)
$ osatura comp apply c1.json c2.json pose1
wrote c2.json
```

### `retarget`

**Sinopse:** `osatura retarget <source.json> <dest.json> <out.json> --pairs srcChain=dstChain[,..]`

Faz o retarget por *intenção*: a animação do **efetuador** de IK (osso alvo) de cada cadeia de origem é escalada pela razão entre os alcances dos dois membros (soma dos comprimentos dos ossos) e redirecionada para o osso alvo da cadeia de destino. Canais de efetuador sem keys não podem ser escalados; eles são repassados com novo nome e reportados como linhas `note:`. `--pairs srcChain=dstChain[,...]` é obrigatório. No exemplo, origem e destino são o mesmo projeto, o que só serve como smoke test.

```
$ osatura retarget demo.json demo.json rt.json --pairs leg=leg
wrote rt.json (+14 channels)
```
**Relacionados:** `mocap-retarget` para dados BVH; `walk`, `mirror`.

---

## 6. Animação, keys, timeline e geradores

### `keys`

**Sinopse:**
```
osatura keys shift <project.json> <out.json> --by N [--from F --to F --target NAME --prop PROP]
osatura keys scale <project.json> <out.json> --factor X [--pivot F --from F --to F --target NAME --prop PROP]
osatura keys ease <project.json> <out.json> --ease linear|step|ease_in|ease_out|ease_in_out|smooth [--from F --to F --target NAME --prop PROP]
osatura keys reverse <project.json> <out.json> [--from F --to F --target NAME --prop PROP]
```
Cada operação aceita somente as suas próprias opções (`keys shift --factor 2` é recusado); `--by`, `--factor` e `--ease` são obrigatórios na respectiva operação (um `--by` ou `--factor` omitido costumava ser uma operação nula silenciosa).

Operações de keys em massa sobre **todos os canais com keys** (filtradas por `--target` / `--prop`; um intervalo `--from/--to` limita quais keys são tocadas). Os canais bloqueados são ignorados com `skipped locked channel ...`.

| Operação | Requer | Efeito |
|---|---|---|
| `shift` | `--by N` | Move as keys em N quadros. |
| `scale` | `--pivot F --factor X` | Escala os tempos das keys em torno do quadro pivô. |
| `ease` | `--ease E` | Define a suavização das keys (suavizações nomeadas; use `keyedit ease` para `bezier:`). |
| `reverse` | - | Inverte a ordem das keys dentro do intervalo. |

```
$ osatura keys shift small.json k1.json --target arm_up --prop bone_angle --by 2
wrote k1.json (5 key(s) changed)
$ osatura keys scale small.json k2.json --pivot 0 --factor 0.5 --from 0 --to 48
wrote k2.json (37 key(s) changed)
$ osatura keys reverse small.json k4.json
wrote k4.json (37 key(s) changed)
```
Keys movidas para fora de `0..frames-1` permanecem no arquivo (o lint reporta `L007`); este comando não estende a duração do projeto. **Relacionados:** `keyedit` (preciso, baseado em seleção), `loop-solve`.

### `keyedit`

**Sinopse:** `osatura keyedit <project.json> <out.json|-> <sub-command> [options]`

Edições de dope sheet como **um único lote seguro para desfazer**. Com `-` como saída, nada é gravado e a operação planejada é impressa como JSON (dry run). Subcomandos e opções:

```
osatura keyedit <project.json> <out.json|-> move --sel SEL [--dt N] [--dv N] [--collision replace|merge|refuse] [--no-snap] [--free]
osatura keyedit <project.json> <out.json|-> scale --sel SEL --pivot F --factor X | dup --sel SEL --offset N | delete --sel SEL | ease --sel SEL --ease E | value --sel SEL --value V | smooth --sel SEL [--kind monotone|catmull_rom]
osatura keyedit <project.json> <out.json|-> copy --sel SEL --clip clip.json | paste --clip clip.json --at F [--remap from_target:prop=to_target:prop]
osatura keyedit <project.json> <out.json|-> add --target T --prop P --at F [--value V] [--ease E] | ripple --at F --by N --mode insert|delete [--scope 0,1] [--adjust-frames]
osatura keyedit <project.json> <out.json|-> offset --channels 0,1 --dt N | extrapolate --channels 0,1 --extrapolate loop | loop --channels 0,1 [--end F] | handle --sel SEL --side in|out --t F --v V [--mode free|aligned|auto] | lock --target T [--prop P] [--value on|off]
```

* **SEL** são itens `TARGET:PROP[@F0..F1]` unidos por `;`, ou `#CHANNEL[@F0..F1]` (índice do canal obtido em `timeline`), ou `all`. Coloque entre aspas (`;`, `@`, `#`).
* **E** (suavização) é `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`, `smooth`, `bounce`, `bezier:x1,y1,x2,y2` ou um objeto JSON.
* `--collision` decide o que acontece quando uma key movida cai sobre uma key existente (`replace` é o padrão, `merge`, `refuse`); `--no-snap` mantém quadros fracionários; `--free` elimina a restrição a quadros inteiros.
* Linha de saída: `<op>: N keys changed, wrote <out>`. `copy` com uma saída real imprime `copy: 0 keys changed, project unchanged, clip saved`.
* As operações que colocariam uma key fora de `0..=frames` são recusadas com uma mensagem precisa (`duplicate: frame 52 is outside the project range 0..=48`), a menos que `ripple --adjust-frames` estenda o projeto.

```
$ osatura keyedit demo.json - move --sel "arm_up:bone_angle@12..36" --dt 2        # dry run: prints the batch JSON
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", ... } ] } }
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 2 --collision merge
move: 3 keys changed, wrote k1.json
$ osatura keyedit demo.json k5.json ease --sel "arm_up:bone_angle" --ease bezier:0.4,0,0.2,1
ease: 5 keys changed, wrote k5.json
$ osatura keyedit demo.json k7.json smooth --sel "arm_up:bone_angle" --kind monotone
smooth: 4 keys changed, wrote k7.json
$ osatura keyedit demo.json k8.json copy --sel "arm_up:bone_angle@0..12" --clip c2.json
copy: 0 keys changed, project unchanged, clip saved
$ osatura keyedit demo.json k8.json paste --clip c2.json --at 30 --remap arm_up:bone_angle=arm_low:bone_angle
paste: 2 keys changed, wrote k8.json
$ osatura keyedit demo.json k9.json add --target arm_up --prop bone_angle --at 6 --value 5 --ease linear
add: 1 keys changed, wrote k9.json
$ osatura keyedit demo.json k10.json ripple --at 12 --by 6 --mode insert --adjust-frames
ripple: 28 keys changed, wrote k10.json            # project now 54 frames
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 12 --collision refuse
a key already exists at frame 48                    [exit 1]
$ osatura keyedit demo.json k1.json move --sel "zzz:bone_angle" --dt 1
nothing matches 'zzz:bone_angle'                    [exit 1]
$ osatura keyedit demo.json k15.json lock --target arm_up --prop bone_angle --value on
lock: 1 keys changed, wrote k15.json
```
**Pegadinhas:** `delete` com `#0@12..12` remove exatamente uma key; `ripple --mode delete --scope 0,1` limita o deslocamento aos canais 0 e 1; `copy` e `paste` trocam um arquivo de clipe, não a área de transferência do sistema. Canais bloqueados recusam `move/scale/...` (use antes `lock --value off`).

### `marker`

**Sinopse:** `osatura marker <project.json> <out.json> <name> <frame> [--end F]`

Adiciona um marcador de timeline (um ponto nomeado ou, com `--end`, um intervalo). Um marcador com o mesmo nome é substituído. Os marcadores são ordenados por quadro.

```
$ osatura marker small.json m1.json hit 4 --end 6
wrote m1.json (marker 'hit' at 4 = 00:00:00:04)
```

### `event`

**Sinopse:** `osatura event <project.json> <out.json> <name> <frame> [--data JSON]`

Adiciona um evento nomeado (um gatilho de runtime para players) em um quadro, com payload JSON opcional. JSON inválido: `--data is not valid JSON: ...`.

```
$ osatura event small.json ev1.json boom 3 --data '{"power":2}'
wrote ev1.json (1 events)
```

### `idle`

**Sinopse:**
```
osatura idle blink <project.json> <out.json> --target T --prop P [--interval S --jitter X --closed F --seed N]
osatura idle breath <project.json> <out.json> --target T --prop P [--amp A --period F]
osatura idle sway <project.json> <out.json> --target T --prop P [--amp A --freq HZ --seed N]
```
Cada gerador aceita somente as suas próprias opções (`idle breath --seed 2` é recusado).

Gera um canal idle procedural e o **acrescenta** (os canais existentes permanecem).

| Gerador | Opções (padrões) | Alvo típico |
|---|---|---|
| `blink` | `--interval` média de segundos entre piscadas, `--jitter`, `--closed` quadros com o olho fechado, `--seed` | um `layer_switch` (olho aberto/fechado) ou `layer_scale_y` |
| `breath` | `--amp` (0.03), `--period` quadros (3 s na fps do projeto) | `bone_scale` de um torso |
| `sway` | `--amp` (2), `--freq` Hz (0.3), `--seed` (1) | `bone_angle` |

```
$ osatura idle blink small.json i1.json --target mouth --prop layer_switch --interval 1 --seed 3
wrote i1.json
$ osatura idle sway small.json i3.json --target arm_up --prop bone_angle --amp 3 --freq 0.5
wrote i3.json
```
`--target` e `--prop` são obrigatórios (`--target is required`).

### `follow` (duas formas)

**Follow-through (opções):** `osatura follow <project.json> <out.json> --target SRC --prop P --chain a,b,c [--delay F --decay X]`
Adiciona cópias atrasadas e amortecidas de um canal existente (`--target`/`--prop` nomeiam a origem) aos ossos da cadeia: overlapping action / follow-through (`--delay` quadros por elo, padrão 3; `--decay` por elo, padrão 0.7). Uma cópia atrasada é cortada no último quadro do projeto (nenhuma key é gravada depois do quadro `frames`, portanto o L007 do `lint` não dispara); os quadros entre a última key mantida e o corte são amostrados para que a curva atrasada seja exata em cada quadro inteiro.

```
$ osatura follow small.json f1.json --target arm_up --prop bone_angle --chain arm_low,hand --delay 2 --decay 0.6
wrote f1.json (+2 channels)
```

**Seguimento de caminho (posicional):** `osatura follow <project.json> <out.json> <layer|bone:NAME> <layer_with_path> <start_frame> <end_frame> [no-orient]`
Faz o bake de uma camada (ou do osso raiz `bone:NAME`) percorrendo o primeiro caminho de outra camada entre dois quadros, girando para a tangente do caminho, a menos que `no-orient` seja informado.

```
$ osatura follow small.json fp.json target ink 0 7
wrote fp.json
$ osatura follow small.json fp2.json bone:hand ink 0 7 no-orient
path following needs a root bone (no parent)                      [exit 1]
```

### `take-record`, `take-apply`, `take-list`

**Sinopse:**
```
osatura take-record <project.json> <library.json> <name> <from> <to> [target_prefix,...]
osatura take-apply  <project.json> <out.json> <library.json> <name> <at> [from=to,...] [speed] [--blend FRAMES]
osatura take-list   <library.json>
```
Um **take** é uma fatia reutilizável de animação armazenada em um arquivo de biblioteca. `take-record` captura os quadros `from..to` dos canais cujo alvo começa com um dos prefixos (todos os canais quando omitidos); a biblioteca é criada se não existir e um take com o mesmo nome é substituído. `take-apply` cola o take no quadro `at`, opcionalmente fazendo o **retargeting** (pares `from=to` renomeiam alvos) e alterando `speed`; os argumentos são verificados de forma estrita: uma entrada sem `=`, um nome de origem para o qual o take não tem faixa, ou uma velocidade que não seja um número positivo é um erro, e uma biblioteca ilegível nomeia o arquivo (`cannot read the take library ...`); `--blend N` faz crossfade com a animação existente por N quadros nas duas extremidades.

```
$ osatura take-record demo.json lib.json wave 0 24 arm_
recorded 2 tracks into lib.json
$ osatura take-list lib.json
wave (24 frames, 2 tracks)
$ osatura take-apply demo.json ta.json lib.json wave 10 arm_up=arm_low 1.5 --blend 4
applied 2 tracks; wrote ta.json
```

### `clip`

**Sinopse:**
```
osatura clip add <project.json> <out.json> <library.json> <take> <clip> [--at F --speed S --loops N --mode replace|additive|multiply --weight W
                   --include a_*,b --exclude c --track-weight arm_*=1,leg_*=0.2 --rename a=b --fade-in F --fade-out F]
osatura clip list <project.json>
osatura clip remove <project.json> <out.json> <clip>
osatura clip set-weight <project.json> <out.json> <clip> [W] [--keys frame:w,frame:w] [--track PATTERN[:prop]]
osatura clip bake <project.json> <out.json>
```
Uma **pilha de clipes** em camadas (animação não linear): os clipes referenciam takes de uma biblioteca, tocam em `--at` com `--speed`/`--loops` e se mesclam em `replace`, `additive` ou `multiply` com um peso (constante, animado no tempo ou por padrão de faixa). `clip add/set-weight/remove` apenas editam a pilha; **`clip bake` regenera os canais pertencentes à pilha** (canais feitos à mão e bloqueados nunca são tocados).

```
$ osatura clip add demo.json cl1.json lib.json wave w1 --at 6 --speed 1 --loops 2 --mode additive --weight 0.8 --fade-in 2 --include 'arm_*' --rename arm_up=hand
added clip 'w1' (1 clips); run clip bake to generate channels
$ osatura clip list cl1.json
1. w1 take 'wave' frames 6..30 x1 speed 1 Replace weight 1
0 baked channels
$ osatura clip set-weight cl1.json cl2.json w1 0.5
updated clip 'w1'; run clip bake to update channels
$ osatura clip set-weight cl1.json cl3.json w1 --keys 0:0,12:1
$ osatura clip bake cl2.json cl5.json
baked 1 clips into 4 channels (0 replaced); wrote cl5.json
$ osatura clip remove cl2.json cl6.json w1
removed clip 'w1'; run clip bake to update channels
```
(O exemplo de `clip list` mostra o segundo `clip add` com o mesmo nome, que substituiu o primeiro.) **Limites (ROADMAP §9):** os canais gerados no bake são amostrados por quadro inteiro; não existe `clip refresh` para recarregar um take editado da biblioteca.

### `make-loop`

**Sinopse:** `osatura make-loop <project.json> <out.json> [targets|all] [props] [--inclusive]`

Faz os canais `replace` animados à mão terminarem onde começaram: cada curva é amostrada por quadro, a reta do seu primeiro ao seu último valor é subtraída (remoção de deriva) e o primeiro valor é somado de volta. Aplicado ao deslocamento X de um osso que viaja, transforma uma caminhada em uma caminhada no lugar. O período do loop é N = os `frames` do projeto, de modo que tocar `0..N-1` repetidamente não mostra nenhum quadro de emenda duplicado. `--inclusive` é a convenção legada em que o último quadro é igual ao primeiro. Sem argumentos, todo canal com keys é processado. Imprime uma linha `removed drift` por canal alterado.

```
$ osatura make-loop demo.json ml2.json foot_ctrl bone_offset_x
foot_ctrl BoneOffsetX: removed drift 0.79
wrote ml2.json
$ osatura make-loop demo.json ml.json all bone_angle --inclusive
```
**Limites:** remoção linear de tendência (detrending) de manual, não um retiming do movimento; os canais comandados por expressão não são tocados. **O deslocamento é removido de propósito** (um projeto que toca `0..N-1` repetidamente não pode viajar para sempre, então o loop é no lugar): um canal cuja mudança inteira era deslocamento torna-se constante, e o comando imprime uma linha `note: ... is now constant` para ele; o `lint` então reporta esse canal como L005 (informação), o que é esperado. Nomeie os alvos e as props para deixar intacto o deslocamento de outros canais, ou use `loop-solve` para movimento que já é cíclico.

### `loop-solve`

**Sinopse:** `osatura loop-solve <project.json> <out.json> [--window 24] [--inclusive]`

Resolve a **emenda**: mescla o fim dos canais com keys ao seu início ao longo de uma janela, para que a animação faça loop sem um salto. Imprime o erro de emenda antes e depois.

```
$ osatura loop-solve demo.json ls.json --window 24
wrote ls.json: 8 channel(s) adjusted, seam 0.000 -> 0.000
```
`--window` é em quadros (padrão 24). Compare com o contract `loop` do `lint` (`contracts`).

### `walk`

**Sinopse:** `osatura walk <project.json> <out.json> --thigh-l B --thigh-r B [--shin-l B --shin-r B --arm-l B --arm-r B --hips B --period N --stride DEG --knee DEG --bob PX --swing DEG]`

Gera um ciclo de caminhada em loop como canais com keys nos ossos que você nomear (é obrigatória pelo menos uma opção de osso: `name at least one bone (--thigh-l, --thigh-r, ...)`). Padrões: `--period 24` quadros, `--stride 28`, `--knee 40`, `--bob 4`, `--swing 22`. Os canais usam extrapolação em loop e são **acrescentados**.

```
$ osatura walk small.json wk.json --thigh-l arm_up --thigh-r arm_low --period 8 --stride 20
wrote wk.json (+2 channels)
```

### `mirror`

**Sinopse:** `osatura mirror <project.json> <out.json>`

Espelha todos os canais com keys em relação ao eixo vertical: os alvos trocam de lado (prefixos `L_`/`R_`, sufixos `_L`/`_R` e `_l`/`_r`, as palavras `left`/`right`), e as rotações e translações horizontais mudam de sinal. Os canais espelhados **substituem** os canais do projeto (não são adicionados ao lado dos originais). A linha de resultado é `wrote X (N of M channel target(s) swapped sides)`; quando nenhum alvo tem nome de esquerda/direita, ou o projeto não tem canais, um aviso na stderr informa isso (apenas os sinais são invertidos).

### `foot-lock`, `stretch`, `magnet`

```
osatura foot-lock <project.json> <out.json> <ctrl_bone> <ground_y> [tolerance] [min_frames]
osatura stretch   <project.json> <out.json> <follower_bone> <leader_bone> [min=1] [max=3]
osatura magnet    <project.json> <out.json> <ctrl_bone> <bone:NAME|x,y> <radius> <strength 0..1>
```

* **`foot-lock`** amostra a posição de mundo do controlador a cada quadro; sequências de pelo menos `min_frames` quadros em que ele está no chão (o y do palco aponta para baixo: o pé está embaixo quando `y >= ground_y - tolerance`) são *contatos*, e um canal de correção **aditivo** prende o controlador ao ponto onde cada contato começou. A animação original permanece intacta; exclua os canais adicionados para desfazer.
  ```
  $ osatura foot-lock demo.json fl.json foot_ctrl 385 2 2
  contact frames 23..47 pinned at (479.2, 384.1)
  wrote fl.json
  ```
* **`stretch`** faz `follower_bone` mirar a cabeça de `leader_bone` e escalar (uniformemente, entre `min` e `max`, padrões 1 e 3) para que a sua ponta a alcance. Gerado no bake como um canal de ângulo aditivo mais um canal de escala multiplicativo; os filhos do seguidor são escalados junto. O líder não pode depender do seguidor (`the leader depends on the follower (it is one of its descendants)`).
  ```
  $ osatura stretch p.json st.json flw lead 1 2
  corrected 8 frames
  wrote st.json
  ```
* **`magnet`** faz o bake de uma atração por proximidade de um controlador em direção à cabeça de um osso (seguida a cada quadro) ou a um ponto fixo: queda suave, com `strength` total (0..1) no alvo e zero em `radius`. Gerado no bake como canais de deslocamento aditivos.
  ```
  $ osatura magnet p.json mg.json lead 200,100 200 0.8
  pulled on 8 frames
  wrote mg.json
  ```
**Limites (ROADMAP §9):** os três são resultados de bake obtidos por amostragem por quadro: execute-os de novo após editar a animação de origem. `stretch` usa escala uniforme. `foot-lock` e `magnet` esperam um osso controlador raiz.

### `shot`, `camera-seq`

```
osatura shot       <project.json> <out.json> <name> <x> <y> <zoom> [rotation]
osatura camera-seq <project.json> <out.json> <frame:shot[:cut],...>
```
`shot` salva um **plano de câmera** nomeado (centro x, y, zoom, rotação opcional em graus). `camera-seq` anima a câmera através dos planos salvos: em cada `frame` a câmera se move para o plano (suavizada) ou, com o sufixo `:cut`, salta para ele. Grava canais `camera_x/y/zoom/rotation`.

```
$ osatura shot small.json sh.json wide 320 240 1 15
wrote sh.json (1 shots)
$ osatura shot sh.json sh2.json close 400 200 2
wrote sh2.json (2 shots)
$ osatura camera-seq sh2.json cs.json 0:wide,4:close:cut
wrote cs.json
```

### `spring-fit`

**Sinopse:** `osatura spring-fit --start V --samples a,b,c,... [--fps 24]`

Ajusta a **frequência (Hz) e a razão de amortecimento** de uma mola a uma curva de resposta desenhada (amostras por quadro de um degrau de `--start` até o último valor). Imprime o par ajustado e o JSON a colar nas `springs` de um projeto. Não lê um projeto.

```
$ osatura spring-fit --start 0 --samples 0.5,0.9,1.1,1.0,0.98,1.0 --fps 24
frequency 4.68 Hz, damping 0.61 (rms error 0.028 of the step size)
add to the project: {"bone": "<bone>", "frequency": 4.68, "damping": 0.61}
```

### `script`

**Sinopse:** `osatura script <project.json> <out.json> <script.rhai>`

Executa um **script Rhai em sandbox** que edita o projeto por meio de requisições validadas (uma requisição rejeitada deixa tudo inalterado:
`request 5 (box.LayerOpacity at 0) was rejected, nothing changed: ...`). Funções embutidas: `fps() frames() bones() layers() set_key(target, prop, frame, value[, ease]) add_bone(name, parent (empty = root), length, x, y, angle) add_rect add_circle add_polygon|add_blob add_line add_gradient_rect add_ik add_aim add_copy_rotation add_pin add_group(name[, [layers]]) add_switch move_layer set_parent delete_layer delete_bone set_layer set_bone add_channel_expr add_noise add_marker add_event bone_info layer_info world print`.
Um script que não aplica nenhuma requisição imprime `warning: script: 0 requests were applied`. Builds sem o recurso `scripting` respondem `this build has no scripting support`.

```
$ cat t.rhai
add_rect("box", 10.0, 10.0, 50.0, 50.0, "#ff0000");
add_rect("box2", 80.0, 10.0, 50.0, 50.0, "#00ff00");
add_group("skins", ["box", "box2"]);
set_key("skins/box", "layer_opacity", 0, 1.0);
print(fps());
$ osatura script small.json sc.json t.rhai
24.0
4 request(s) applied; wrote sc.json
```
As camadas dentro de um grupo são endereçadas por caminho (`skins/box`). `print` vai para a stdout.

---

## 7. Física, tecido e deformadores

Todos estes comandos fazem o **bake** de uma simulação ou de um resultado geométrico em canais comuns com keys: o projeto continua determinístico e o resultado é reproduzido em qualquer lugar (inclusive em Lottie/GLB, quando aplicável). Execute o comando de novo depois de alterar a animação de origem.
Eles recebem `<project.json> <out.json>` e palavras `key=value` (sem `--`).

### `physics`

**Sinopse:**
```
osatura physics <project.json> <out.json> <bone,bone,...> [gravity=900] [wind=x,y] [gust=0.5] [damping=0.985]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [follow=bone] [--allow-empty]
    [drive=bone:gain;...] [vortex=bone|x,y:swirl[:radial[:radius]];...] [preset=hair|tail|rope|antenna|cape]
    [merge=keep|replace] [lateral=0..1]
```
Faz o bake de uma **simulação de cadeia** de Verlet (cabelo, cauda, corda, antena, capa) nos ossos listados (a raiz da cadeia primeiro) como keys de `bone_angle`, uma por quadro.

| Palavra | Significado |
|---|---|
| `gravity=` | Aceleração para baixo (padrão 900 unidades de palco/s^2). |
| `wind=x,y`, `gust=`, | Vetor de vento constante; amplitude de rajada de um vento modulado por ruído. |
| `damping=` | Amortecimento da velocidade por passo (padrão 0.985). |
| `circle=x,y,r[@bone];...`, `capsule=x1,y1,x2,y2,r[@bone];...` | Colliders, opcionalmente presos a um osso (eles o acompanham). Cápsulas são aceitas (verificado com `capsule=100,300,300,300,20`; uma antiga observação "capsule colliders NOT done" no ROADMAP §9 está desatualizada). |
| `follow=bone` | A raiz da cadeia é carregada junto com o movimento deste osso (inércia). |
| `drive=bone:gain;...` | Corpos que empurram a cadeia (sua velocidade vezes o ganho). |
| `vortex=bone\|x,y:swirl[:radial[:radius]];...` | Campos de redemoinho em torno de um osso ou ponto. |
| `preset=` | Valores iniciais para `hair tail rope antenna cape`; as palavras escritas **depois** de `preset=` o sobrepõem. |
| `merge=keep\|replace` | `keep` (padrão) deixa intactos os ossos que já têm animação; `replace` os sobrescreve. |
| `lateral=0..1` | Com várias cadeias (`bone,bone/bone,bone`), amarra fios vizinhos entre si. |
| `--allow-empty` | Aceita uma execução que não gerou nada no bake. |

```
$ osatura physics p.json ph.json tail1,tail2,tail3 gravity=500 preset=tail
baked 3 bones; wrote ph.json
$ osatura physics p.json ph4.json tail1,tail2/tail3 lateral=0.5            # two strands
baked 3 bones; wrote ph4.json
$ osatura physics p.json ph3.json tail1,tail2,tail3 follow=arm_up circle=300,300,40 wind=10,0 gust=0.5 lateral=0.3 drive=lead:0.5
baked 3 bones; wrote ph3.json
$ osatura physics demo.json ph.json arm_low,hand gravity=500 preset=tail   # these bones are keyed already
kept existing animation on 'arm_low' (use merge=replace to overwrite)
kept existing animation on 'hand' (use merge=replace to overwrite)
physics: nothing baked - 2 bone(s) already have animation and merge=keep (the default) leaves it alone; nothing was written. Use merge=replace to overwrite the existing animation, or --allow-empty to accept an unchanged file     [exit 1]
$ osatura physics demo.json ph.json hand preset=bogus
unknown preset 'bogus' (try hair, tail, rope, antenna, cape)                [exit 1]
```
(`p.json` é o demo mais três ossos sem keys `tail1..tail3` criados com um script Rhai - veja `script`.) **Pegadinha:** um bake que não altera nada é um erro, a menos que se use `--allow-empty`.
**Limites (ROADMAP §9):** os colliders ignoram a escala dos ossos (somente transformação rígida); círculos estáticos colidem apenas em pontos (círculos presos também ao longo dos elos); sem colisão cápsula-cápsula nem autocolisão.

### `cloth`

**Sinopse:**
```
osatura cloth <project.json> <out.json> <layer> [pin=top,left@bone,row:N,col:N,v:N,corners,none] [gravity=900]
    [wind=x,y] [gust=0.5] [gust-hz=0.5] [gust-len=px] [stiffness=1] [shear=0.05] [bend=0.2] [max-stretch=1.1]
    [damping=0.99] [cols=N rows=N] [rect=x,y,w,h] [name=cloth] [substeps=4] [iterations=10] [preroll=frames]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [from=F] [to=F]
```
Faz o bake de uma **superfície de tecido** determinística na malha de uma camada de *imagem*, ou em uma camada *vetorial* quando `rect=x,y,w,h` é informado (a camada vetorial recebe uma malha em grade sobre esse retângulo), como canais de deformação por vértice. `pin=` escolhe os vértices fixados (bordas `top`, `left`..., `corners`, `row:N`, `col:N`, `v:N`, `@bone` prende os pinos a um osso, `none`). `from`/`to` limitam o intervalo de quadros do bake, `preroll` executa a simulação antes do quadro `from` para que o tecido comece em repouso.

```
$ osatura cloth small.json cloth.json sun rect=100,60,120,120 cols=6 rows=6 pin=top gravity=900 wind=40,0
baked cloth on 'sun' (84 channels); wrote cloth.json
$ osatura cloth small.json cloth.json sun
a vector layer needs rect=x,y,w,h                    [exit 1]
$ osatura cloth small.json cloth.json nosuch
no layer at 'nosuch'                                 [exit 1]
```
Os `84 channels` são keys de `lattice_dx` / `lattice_dy` (alvo `sun::cloth::<col>,<row>`) dos 42 pontos não fixados da grade 7x7 (a linha superior está fixada). São canais reais no arquivo, portanto um bake de tecido aumenta o projeto (98 canais em vez de 14 aqui) - use `size-report` para ver o custo. **Limites (ROADMAP §9):** o tecido é planar (sem dobras fora do plano, portanto o cisalhamento é complacente por padrão); ele recusa malhas livres (somente malhas em grade); sem autocolisão; bake por quadro inteiro.

### `cage`

**Sinopse:** `osatura cage <project.json> <out.json> <layer> [rect=x,y,w,h|auto] [hull] [verts=N] [pad=P] [name=cage]`

Adiciona um **deformador cage de coordenadas de valor médio** em torno de uma camada: o conteúdo dentro do cage segue os vértices do cage, o conteúdo fora permanece inalterado. `auto` (padrão quando não há `rect`) usa os limites com margem, `verts=N` é o número de vértices por lado, `hull` usa o casco convexo, `pad=` acrescenta margem à caixa, `name=` dá nome ao deformador (padrão `cage`).

```
$ osatura cage small.json cg.json sun auto verts=3
added cage 'cage' (12 vertices) to 'sun'; wrote cg.json
```
**Limites (ROADMAP §9):** sem ajuste automático do cage além de uma caixa com margem ou do casco convexo (sem traçado côncavo); não há queda suave fora do cage (aparece um degrau visível onde a arte cruza a borda do cage); malhas livres não têm deslocamentos animados por vértice (use um cage).

### `cage-pose`

**Sinopse:** `osatura cage-pose <project.json> <out.json> <layer> <frame> [name=cage] [move=i:dx,dy;...] [all=dx,dy] [ease=linear]`

Cria keys de deslocamento dos vértices do cage em um quadro (canais `cage_dx` / `cage_dy`). `move=` desloca vértices individuais, `all=` desloca todos os vértices. Canais bloqueados são recusados.

```
$ osatura cage-pose cg.json cp.json sun 4 all=5,0
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
$ osatura cage-pose cg.json cp.json sun 4 move=0:5,5
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
```

### `warp-pivot` e `soft-scale`

**Sinopse:**
```
osatura warp-pivot <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=1,4-7] [region=x,y,w,h]
    [pivot=x,y|bone:NAME|centroid|center] [rotate=DEG] [scale=S|sx,sy] [move=dx,dy] [radius=R] [falloff=smooth] [ease=linear]
osatura soft-scale <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=..|region=..] scale=S|sx,sy
    [radius=R] [falloff=smooth] [pivot=..]
```
`warp-pivot` gira/escala/move uma **seleção** de pontos de controle (deformadores cage, lattice, cloth, pin, spine; uma malha de imagem em grade recebe uma grade de warp por vértice) em torno de um pivô e cria a key do resultado no quadro. A seleção é `verts=` (índices e intervalos) e/ou `region=`; uma seleção vazia é um erro (`the selection is empty (give verts= and/or region=)`). `radius=` acrescenta uma **queda suave** (`linear smooth gauss sharp constant`). `soft-scale` é um escalonamento proporcional (seleção suave); sem `verts`/`region`, a malha inteira é redimensionada em torno do centro do seu retângulo delimitador por `scale=F`, `width=W`, `height=H` ou `size=WxH`, com `aspect=keep|free`.

```
$ osatura warp-pivot cg.json wp.json sun 4 verts=0-3 rotate=15 pivot=centroid radius=30 falloff=gauss
warp-pivot on 'sun' keyed at frame 4; wrote wp.json
$ osatura soft-scale cg.json ss.json sun 4 scale=1.2 pivot=center
soft-scale on 'sun' keyed at frame 4; wrote ss.json
```

### `layer-pivot`

**Sinopse:** `osatura layer-pivot <project.json> <out.json> <layer> pivot=x,y|bone:NAME`

Move o pivô (âncora) de uma camada **sem alterar a imagem**, de modo que os canais de rotação e escala girem em torno do novo ponto.

```
$ osatura layer-pivot small.json lp.json sun pivot=100,100
moved the pivot of 'sun'; wrote lp.json
```

### `mesh-lock`

**Sinopse:** `osatura mesh-lock add <project.json> <out.json> <name> host=LAYER[:path] follower=LAYER[:path] [verts=1,4-7] [strength=1]`
e `osatura mesh-lock list|remove <project.json> [<out.json> <name>]`

Faz os vértices do seguidor acompanharem a **superfície da malha hospedeira** a cada quadro (transferência de deslocamento baricêntrica), p. ex. uma camisa acompanhando um corpo. Ciclos são recusados; hospedeira e seguidor devem ter o mesmo `parallax` (`mesh lock 'lock1': host and follower must have the same parallax`).

```
$ osatura mesh-lock add two.json ml1.json lock1 host=ra follower=rb verts=0-1 strength=0.5
added mesh lock 'lock1'; wrote ml1.json
$ osatura mesh-lock list ml1.json
lock1: rb:0 follows ra:0 vertices [0, 1] strength 0.5
1 mesh lock(s)
$ osatura mesh-lock remove ml1.json ml2.json lock1
removed mesh lock 'lock1'; wrote ml2.json
```

### `glue`

**Sinopse:** `osatura glue add <project.json> <out.json> <name> a=LAYER[:path] b=LAYER[:path] pairs=i:j;i:j|auto=RADIUS [mode=rigid|snap|offset] [strength=1]`
e `osatura glue list|remove <project.json> [<out.json> <name>]`

Cola vértices da camada `b` a vértices da camada `a` (b segue a). `pairs=i:j;...` lista explicitamente os pares de vértices, `auto=RADIUS` emparelha cada vértice de `b` que esteja dentro do raio de um vértice de `a`. Modos: `rigid` mantém o deslocamento de repouso no referencial de `a`, `snap` faz com que coincidam, `offset` mantém um deslocamento constante.

```
$ osatura glue add two.json gl1.json g1 a=ra b=rb auto=5 mode=snap
added glue 'g1' (4 pair(s)); wrote gl1.json
$ osatura glue list gl1.json
g1: Snap strength 1 ra:0#0->rb:0#0 ra:0#1->rb:0#1 ra:0#2->rb:0#2 ra:0#3->rb:0#3
1 glue(s)
$ osatura glue add small.json x.json g1 a=sun b=hills auto=20 mode=snap
no vertex of 'hills' lies within 20 of a vertex of 'sun'          [exit 1]
```

### `mesh-trace`

**Sinopse:** `osatura mesh-trace <project.json> <out.json> <layer> [density=24] [tolerance=1.5] [smooth=N] [hotspot=x,y,r;...] [edge=D/4] [grading=0.5] [lambda=0.5] [taubin=1] [smooth-boundary=0] [threshold=128] [bones=R] [hot-size=E] [skin=1]`

Traça a **silhueta alfa de uma camada de imagem** em uma malha triangular livre de densidade variável (mais densa ao longo do contorno, em torno de hot-spots e ossos), suaviza-a e faz o skin aos ossos. Imprime as contagens de vértices/triângulos e a qualidade.

```
$ osatura mesh-trace o.json mt.json top density=12 tolerance=2 bones=0
wrote mt.json: 9589 vertices (1494 on the outline), 17682 triangles, area 1227677 px2
  quality min 0.484 mean 0.890, min angle 18.3 deg, mean edge 3.0 px at the outline / 13.7 px inside
```
(`o.json` é uma importação OpenRaster de 1280x960; um `density` menor produz triângulos menos numerosos e maiores. A execução levou 5 s em uma build debug.) **Limites:** precisa de uma camada de imagem (uma silhueta alfa); uma malha tão densa deixa o arquivo de projeto grande - confira `size-report`. Malhas livres não podem ser simuladas como tecido.

### `skin-diffuse`

**Sinopse:** `osatura skin-diffuse <project.json> <layer> <out.json> [--influences 4]`

Calcula **pesos de skin por difusão de calor** para uma camada com malha a partir dos ossos do projeto, mantendo no máximo `--influences` (padrão 4) ossos por vértice. Observe a ordem dos argumentos: **a camada antes da saída**. A camada deve ser uma camada de imagem com malha.

```
$ osatura skin-diffuse demo.json arm sd.json
layer 'arm' is not an image layer                          [exit 1]
$ osatura skin-diffuse o.json top sd.json
layer 'top' has no mesh                                    [exit 1]
$ osatura skin-diffuse mt.json top sd.json               # mt.json from mesh-trace
wrote sd.json
```

---

## 8. Áudio, lip-sync e sons

As entradas de áudio dos comandos de análise (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) devem ser **WAV** (PCM inteiro de 8/16/24/32 bits ou float de 32 bits, qualquer número de canais mixado para mono, até 256 MiB). MP3/OGG precisam ser convertidos antes (`ffmpeg -i in.mp3 in.wav`). As **faixas** de som armazenadas em um projeto (`sound`) podem ter qualquer formato que o ffmpeg consiga ler.

### `lipsync`

**Sinopse:** `osatura lipsync <project.json> <audio.wav> <switch_layer> <out.json> [--map 0,1,2,3,4]`

Comanda uma **camada switch** (formas da boca) a partir do volume e do brilho espectral: uma heurística de amplitude / cruzamentos por zero que escolhe uma de cinco formas por quadro (`rest, small, open, round, wide`) e remove cintilações. **Não é reconhecimento de fonemas.** `--map` informa o índice do filho do switch para cada uma das cinco formas; o padrão depende do número de filhos do switch: 1 filho -> todas; 2 -> fechada + aberta; 3 -> fechada + aberta + round/wide; 4 -> fechada + small + aberta + round/wide; 5+ -> na ordem acima. Substitui um canal `layer_switch` animado existente dessa camada. A duração do projeto **não** é alterada (um WAV de 2 s em um projeto de 8 quadros cria keys em 8 quadros).

```
$ osatura lipsync small.json speech.wav mouth lip.json
wrote lip.json (2.00s of audio → 8 frames)
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1,2,1,0
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1
--map needs exactly 5 numbers (rest,small,open,round,wide)          [exit 1]
$ osatura lipsync small.json speech.wav ink lip.json
layer 'ink' is not a switch layer                                   [exit 1]
$ osatura lipsync small.json notwav.mp3 mouth x.json
invalid WAV: Failed to read enough bytes.                           [exit 1]
```

### `lipsync-text`

**Sinopse:** `osatura lipsync-text <project.json> <audio.wav> <transcript.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--lang ro|en] [--words-out timing.json|timing.srt]`

Alinha uma **transcrição** ao áudio (alinhamento de energia + texto: palavras e sílabas são distribuídas sobre as regiões de fala encontradas no áudio) e deriva as formas da boca a partir das sílabas. `--lang` (padrão `ro`) seleciona as regras de vogais/sílabas; existem apenas `ro` e `en`. `--words-out` grava os tempos das palavras como JSON (palavras com sílabas) ou, para um nome `.srt`, como legendas.

```
$ osatura lipsync-text small.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (3 words, 2 exposure changes; energy + text alignment, not phoneme recognition)
$ head -c 140 words.json
{ "words": [ { "text": "hello", "start": 0.0, "end": 0.22, "syllables": [ {"text":"hel","start":0.0,"end":0.16}, {"text":"lo",...
$ osatura lipsync-text ... --words-out w.srt ; head -4 w.srt
1
00:00:00,000 --> 00:00:01,220
hello there world
```
**Limites (ROADMAP §9):** sem temporização por letra (nível de fonema), sem modelos de velocidade específicos por falante, sem idiomas além de ro/en. Um tom sintético é alinhado como fala - use gravações reais.

### `speech-motion`

**Sinopse:** `osatura speech-motion <project.json> <audio.wav> <transcript.txt> <out.json> [--head bone] [--brow bone_or_layer] [--nod deg] [--fall deg] [--tilt deg] [--brow-px px] [--seed n] [--lang ro|en] [--words-out timing.json|timing.srt]`

Adiciona acenos de cabeça, quedas, inclinações e elevações de sobrancelha **aditivos**, sincronizados com as palavras alinhadas. `--head` nomeia o osso da cabeça, `--brow` um osso ou camada para as sobrancelhas; as opções de ângulo são em graus, `--brow-px` em pixels, `--seed` fixa a variação.

```
$ osatura speech-motion small.json speech.wav transcript.txt sm.json --head arm_up --lang en
wrote sm.json (1 additive channel(s) from 3 words)
```

### `mouth`

**Sinopse:** `osatura mouth <project.json> <cues.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--rhubarb]`

Importa **cues** de boca prontos em vez de analisar áudio: arquivos Papagayo/Moho-switch (`frame PHONEME` por linha, primeira linha opcional `MohoSwitch1`) ou TSV do Rhubarb (`seconds<TAB>shape`, reconhecido pelo ponto decimal; os segundos são convertidos na fps do projeto). Linhas em branco e comentários `#` são ignorados. Os fonemas são mapeados para as cinco classes de forma (`rest/MBP/X/A` rest; `etc/FV/L/B/G/H` small; `AI/D` open; `O/U/WQ/F` round; `E/C` wide); formas desconhecidas contam como `small`. `--rhubarb` trata o `E` do Rhubarb como *round* (o `E` do Papagayo é wide). O `--map` padrão é `0,1,2,3,4`.

```
$ printf '0 rest\n2 open\n4 round\n6 wide\n' > cues.txt
$ osatura mouth small.json cues.txt mouth mo.json --rhubarb
wrote mo.json (4 cues)
```

### `audio-react`

**Sinopse:** `osatura audio-react <project.json> <out.json> <audio.wav> <target> <prop> [gain=1] [offset=0] [floor=0.05] [attack=1] [release=4] [add=on]`

O volume comanda uma propriedade: uma key por quadro seguindo o nível suavizado (RMS relativo ao quadro mais alto). Saída = (nível filtrado por `floor`) x `gain` + `offset`; `attack`/`release` são os tempos de subida/descida em quadros; `add=on` torna o canal aditivo em vez de substituir. Um canal bloqueado é recusado. As opções são palavras `key=value`; uma desconhecida gera `unknown option 'foo'`.

```
$ osatura audio-react small.json ar.json speech.wav ink layer_rotation gain=10
wrote ar.json
$ osatura audio-react small.json ar.json speech.wav arm_up bone_angle foo=1
unknown option 'foo'                                                [exit 1]
```

### `beats`

**Sinopse:** `osatura beats <project.json> <out.json> <audio.wav> [sensitivity=1.5] [gap=4] [as=markers|events] [name=beat]`

Detecta saltos de volume acima da média recente (`sensitivity` 1.2-3, maior = menos batidas) separados por pelo menos `gap` quadros, e os registra como **markers** de timeline (padrão) ou **events** com o nome `name`. Imprime os quadros encontrados.

```
$ osatura beats small.json bt.json speech.wav as=events name=hit sensitivity=1.2
1 beats at frames [1]; wrote bt.json
$ osatura beats small.json bt.json speech.wav
0 beats at frames []; wrote bt.json
```

### `waveform`

**Sinopse:** `osatura waveform <audio.wav> <out.json> [buckets=800]`

Picos mínimo/máximo para desenhar uma forma de onda: `{"sample_rate","duration","peaks":[[min,max],...]}`. Não lê um projeto.

```
$ osatura waveform speech.wav wf.json buckets=50
wrote wf.json (50 buckets)
```

### `sound`

**Sinopse:**
```
osatura sound list   <project.json>
osatura sound add    <project.json> <out.json> <name> <file.wav|ogg|mp3|...> [start_frame] [gain] [trim/fade options]
osatura sound edit   <project.json> <out.json> <name> [--start F] [--gain G] [trim/fade options]
osatura sound remove <project.json> <out.json> <name>
osatura sound mix    <project.json> <out.wav|out.ogg|out.mp3>
    trim/fade options (seconds, or frames with an f suffix such as 36f): --trim-in T --trim-out T --fade-in T --fade-out T
```
As faixas de som são **referências** (caminho do arquivo relativo ao projeto, quadro inicial, ganho 0..4, janela de corte dentro do arquivo, fades). Nada é embutido no projeto; o arquivo de áudio deve acompanhá-lo e deve estar dentro do diretório do projeto, a menos que `--allow-external-assets` seja informado. `edit` altera somente os campos informados (`0` limpa um corte). `--trim-out 0` significa "até o fim". `sound mix` renderiza apenas o áudio mixado pela **duração do projeto** (via ffmpeg: wav = PCM de 16 bits a 44,1 kHz, ogg = Vorbis q4, mp3 = 96 kb/s).
O áudio mixado também é multiplexado em `.mp4` (AAC) e `.webm` (Opus) e embutido em `.html`; `.gif`/`.apng` não conseguem transportá-lo.

```
$ osatura sound add small.json s1.json music speech.wav 2 0.8 --fade-in 0.5 --trim-out 36f
wrote s1.json
$ osatura sound list s1.json
music: speech.wav at frame 2 gain 0.8 trim 0s..1.5s fade-in 0.5s
$ osatura sound edit s1.json s2.json music --gain 0.5 --trim-in 0.1 --fade-out 4f
$ osatura sound list s2.json
music: speech.wav at frame 2 gain 0.5 trim 0.1s..1.5s fade-in 0.5s fade-out 0.16666667s
$ osatura sound mix s2.json mix.mp3
wrote mix.mp3 (0.33 s)
$ osatura sound mix s2.json mix.flac
unknown audio format '.flac' (use .wav, .ogg or .mp3)            [exit 1]
$ osatura sound add small.json x.json x nosuch.wav              # a missing file is refused, nothing written
```
`check` falha quando um arquivo de som referenciado está ausente ou ilegível. Sem o ffprobe, para arquivos que não são WAV só se verifica a existência (e não se o arquivo é áudio). **Limites (ROADMAP §9):** sem pan/EQ/ducking por faixa, apenas a curva de fade linear, sem loop de um clipe, sem retiming.

---

## 9. Importação

### `import-svg`

**Sinopse:** `osatura import-svg <file.svg> <out.json> [--split]`

Linha de uso: `osatura import-svg <file.svg> <out.json> [--split]  SVG subset -> one vector layer, or one layer per shape named by its id (warnings for the rest)`

Cria um novo projeto (24 fps, 24 quadros; tamanho a partir de `viewBox` ou de `width`/`height`, caso contrário 512x512) com **uma camada vetorial `svg`** que contém as formas de um subconjunto de SVG: `path` (todos os comandos), `rect` (cantos arredondados ignorados), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` com `transform`, e `fill`/`stroke`/`stroke-width` sólidos. Todo o resto é ignorado com uma linha `warning:` **na saída padrão (stdout)** (quando o elemento está fora de `<defs>`; um `<filter>`, `<clipPath>`, `<mask>` ou `<pattern>` dentro de `<defs>` e os atributos `filter`, `clip-path` e `mask` são ignorados sem aviso): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`/`<radialGradient>`, `<image>`, `<use>`, `<foreignObject>`. Um preenchimento em gradiente (`fill="url(#g)"`) recorre ao cinza. Limites: 8 MB de SVG, 200.000 âncoras, um teto de elementos.

```
$ osatura import-svg s.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
warning: <filter> is not supported and was skipped
warning: <image> is not supported and was skipped
wrote svg.json (3 paths)
$ osatura check svg.json
ok: 's' 200×100 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 0 channels
```
Com `--split`, cada forma vira **a sua própria camada**, nomeada com o `id` da forma, na ordem do arquivo. Uma forma sem id recebe o nome de sua tag e posição (`rect1`, `circle6`), um id repetido recebe `_1`, `_2`..., e os caracteres que um nome de camada não pode conter (`/`, `::`, caracteres de controle) viram `_`; os nomes são cortados em 100 caracteres. Essa é a forma a usar quando o desenho vai ser montado parte por parte; veja [TUTORIAL_SVG.md](../../TUTORIAL_SVG.md).

```
$ osatura import-svg pip.svg pip.json --split
wrote pip.json (16 paths, 16 layers)
```
Importar e depois mesclar: não existe um comando que mescle a camada em um projeto existente; use `script` ou edite o JSON.

### `import-ora`

**Sinopse:** `osatura import-ora <file.ora> <out.json> [--fps N] [--frames N]`

OpenRaster -> projeto: uma **camada de imagem por camada raster** (PNG incorporado), ordem de baixo para cima preservada, camadas ocultas mantidas porém invisíveis, grupos achatados, `opacity`, `visibility` e posição (`x`,`y`) por camada, e as operações de composição `svg:` mais comuns (multiply, screen, overlay, darken, lighten, color-dodge/burn, hard/soft-light, difference, exclusion, plus -> add). O projeto tem `--frames` quadros (padrão 1) a `--fps` (padrão 24). A opacidade de uma pilha aninhada é multiplicada nas suas camadas e o seu modo de mesclagem é herdado, mas as camadas sobrepostas dentro de uma pilha translúcida **não** são achatadas antes (uma transparece através da outra). Nomes de camada duplicados recebem os sufixos `_1`, `_2`.

```
$ osatura import-ora t.ora ora.json
wrote ora.json (2 layers)
$ osatura import-ora t.ora ora2.json --fps 12 --frames 24
$ osatura import-ora layer.png bad.json
not a zip/ORA file: invalid Zip archive: Could not find EOCD       [exit 1]
```
Limites: 2000 entradas no arquivo, 128 MiB por entrada, camadas de 64 Mpixel.

### `import-lottie`

**Sinopse:** `osatura import-lottie <in.json> <out.json> [fx.json]`

Importa um arquivo Lottie (Bodymovin) - um **subconjunto** deliberado - como projeto. Imprime `imported N layers; wrote <out>` e uma linha `warning:` por recurso ignorado.

* **Importado:** camadas de formas com caminhos `sh`, `rc`, `el`, preenchimentos sólidos e em gradiente (pontos finais/centro/raio de gradiente animados tornam-se canais `grad_dx/grad_dy`), contornos, grupos, transformações de camada (posição/escala/rotação/opacidade estáticas ou com chaves, trajetórias de posição curvas calculadas quadro a quadro), modos de mesclagem, uma camada de fundo sólida, camadas de imagem com URIs de dados PNG/JPEG incorporados, camadas nulas como pais (as transformações do pai são incorporadas nos filhos), máscaras de camada (add/subtract/intersect com opacidade, inversão, formas com chaves), luma mattes (aproximados), pré-composições, intervalos de entrada/saída da camada (como uma trilha de opacidade em degrau), suavização de chaves (hold -> step; alças bezier -> cubic-bezier; sem alças -> linear).
* **Caminhos animados:** um caminho cujas chaves de forma têm a mesma topologia (mesmo número de âncoras, aberto/fechado) torna-se um caminho base mais um **alvo de morph** por chave distinta adicional (`k1`, `k2`, ...), com canais de peso `path_morph` que carregam a suavização. Topologia diferente: usa-se a primeira forma e um aviso é impresso.
* **Efeitos (`ef`):** desfoque gaussiano, sombra projetada, tint e fill tornam-se **efeitos por camada** (`effects` na camada, qualquer tipo de camada) com números animados como canais `fx_*`; tint/fill/tritone/levels estáticos em camadas de forma são incorporados nas cores. As cores dos efeitos não podem ser animadas (usa-se o primeiro valor); outros efeitos, e efeitos de cor após um desfoque/sombra em camadas que não são de forma, geram avisos. Apenas os primeiros efeitos de cada camada são mantidos.
* **Ignorado com aviso:** texto e outros tipos de camada, recursos de áudio, pontos de ancoragem animados (primeiro valor), trajetórias de movimento curvas em propriedades diferentes de posição (segmentos retos), máscaras em uma origem de track-matte, cadeias de pais longas demais para serem incorporadas, mais de 2000 camadas, animação com mais de 20.000 quadros.
* `fx.json` é mantido por compatibilidade: um array vazio é gravado (desfoque e sombra projetada agora são efeitos por camada dentro do projeto).

```
$ osatura import-lottie out2.json li.json                   # a Lottie file made by `osatura export`
imported 11 layers; wrote li.json
$ osatura import-lottie in.json out.json                    # hand-made file: animated triangle with animated blur, plus a null and a text layer
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
$ osatura check out.json
ok: 't' 200×200 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 2 channels       # channels: blob::path::k1 path_morph, blob::fx0 fx_blur
$ osatura import-lottie out2.json li2.json fx.json
wrote 0 post effect(s) to fx.json
imported 11 layers; wrote li2.json
$ echo '{}' > bad.json; osatura import-lottie bad.json o.json
missing 'w'                                                        [exit 1]
```
**Pegadinha:** `import-lottie in.json in.json` é recusado (`refusing to overwrite the input ...`), também para `fx.json`. **Limites:** a entrada `.lottie` (zip) não é lida (`stream did not contain valid UTF-8`) - descompacte `a/animation.json` antes; o trabalho quadrático em morphs de forma é limitado apenas pelos tetos de tamanho.

### `import-bvh`

**Sinopse:** `osatura import-bvh <project.json> <file.bvh> <out.json> --joint-map Joint=bone,.. [--axis Zrotation --tol 0.5]`

Mapeia canais de rotação de articulações BVH escolhidas para ossos do rig como canais `bone_angle` com chaves, amostrados no fps e na contagem de quadros do projeto. `--joint-map` é obrigatório; `--axis` (padrão `Zrotation`) seleciona qual canal de rotação BVH é lido (`Zrotation`, `Xrotation`, `Yrotation`); `--tol` (padrão 0,5 grau) simplifica as chaves. Para correspondência automática, movimento da raiz, vistas e escala use `mocap-retarget` (seção 10).

```
$ osatura import-bvh demo.json m.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```

### `import-seq`

**Sinopse:** `osatura import-seq <project.json> <out.json> <layer_name> <folder> [hold=1] [loop=on|off] [start=0] [x=0] [y=0]`

Importa uma **sequência de PNGs** (todos os PNGs da pasta, em ordem de nome de arquivo) como uma camada quadro a quadro: uma camada switch chamada `<layer_name>` com uma camada de imagem por quadro e um canal em degrau que expõe cada uma durante `hold` quadros a partir do quadro `start`, na posição `(x, y)`; `loop=on` a repete. Todas as imagens devem ter o mesmo tamanho (`img.png: size 640x480 differs from the first image (160x120)`).

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20
imported 3 images; wrote is.json
```

### `import-anim`

**Sinopse:** `osatura import-anim <project.json> <out.json> <layer_name> <file.gif|file.apng|file.png> [loop=on|off] [start=0] [x=0] [y=0]`

Importa um **GIF animado ou APNG** como uma camada quadro a quadro, usando os próprios atrasos de quadro do arquivo (convertidos em quadros do projeto). Um PNG simples gera um quadro.

```
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0
imported 4 frames as layer 'gifl'; wrote ia.json
$ osatura import-anim demo.json ia2.json apngl dof.apng
imported 3 frames as layer 'apngl'; wrote ia2.json
```
As imagens dos quadros tornam-se PNGs incorporados: um GIF longo deixa o projeto grande.

### `trace`

**Sinopse:** `osatura trace <image.png> <out.json> [--threshold 128 --tolerance 1.0 --min-area 12 --fill #000000]`

Bitmap -> vetor simples para imagens com **transparência**: traça o **canal alfa** do PNG (marching squares em `--threshold`, 0..255, com interpolação linear das bordas), simplifica os contornos (Ramer-Douglas-Peucker) até `--tolerance` pixels, descarta regiões abaixo de `--min-area`, detecta furos por contenção e grava um novo projeto (chamado `traced`, tela = tamanho da imagem) com os contornos como caminhos preenchidos com a cor `--fill`. Uma imagem totalmente opaca produz, portanto, um contorno ao redor da imagem inteira (como no exemplo). Imprime `wrote <out> (N contours)`. Use `autotrace` para imagens coloridas.

```
$ osatura trace img.png tr.json --threshold 128 --tolerance 1.0 --min-area 12 --fill '#112233'
wrote tr.json (1 contours)
```

### `autotrace`

**Sinopse:** `osatura autotrace <image.png|jpg> <out.json|out.svg> [--mode fill|line] [--colors N=8] [--min-area A=8] [--tol T=1] [--smooth N=3] [--corner DEG=55] [--denoise N=0] [--quant kmeans|mediancut] [--seed S] [--fit bezier|polygon|vw] [--zorder area|luminance] [--threshold T] [--invert] [--prune N=4] [--into project.json] [--name L] [--fit-stage] [--png preview.png]`

Vetorização local de bitmap - **sem modelo, sem rede**: quantização de cores (`--colors`, k-means ou median cut), extração de contornos, ajuste de Bezier cúbica de Schneider (`--fit`; `polygon` e Visvalingam-Whyatt `vw` são alternativas), furos por par-ímpar; `--mode line` afina os traços até as linhas centrais (Zhang-Suen, `--prune` remove pequenas farpas). A saída é um novo projeto, um novo `.svg` ou - com `--into project.json` - uma cópia de um projeto existente com um novo grupo de camadas `--name` (padrão `trace`); `--fit-stage` (com `--into`) escala o novo grupo uniformemente para caber e o centraliza no palco do projeto, `--png` grava uma renderização de pré-visualização. Imprime contagens e o **erro medido em relação à origem** (erro absoluto médio, RMSE, PSNR, SSIM).

```
$ osatura autotrace img.png at.json --colors 4 --png prev.png
layer group 'trace' added
wrote at.json: 44 regions, 0 strokes, 44 paths, 324 anchors
fidelity vs source: MAE 13.38/255, RMSE 17.59, PSNR 23.2 dB, SSIM 0.960
$ osatura autotrace img.png at.svg --mode line --threshold 100
wrote at.svg: 0 regions, 11 strokes, 11 paths, 47 anchors
fidelity vs source: MAE 52.47/255, RMSE 58.73, PSNR 12.8 dB, SSIM 0.903
$ osatura autotrace img.png at2.json --into d.json --name traced
layer group 'traced' added
wrote at2.json: 51 regions, 0 strokes, 51 paths, 347 anchors
```
**Limites (ROADMAP §9):** um vetorizador clássico - fotografias viram faixas de cor; não há vetorização aprendida ou semântica. Aqui `--png` é uma opção com valor (nome do arquivo de pré-visualização), diferentemente de outras flags `--png`.

### `autotrace-bench`

**Sinopse:** `osatura autotrace-bench`

Vetoriza formas sintéticas, gradientes e traços embutidos e imprime uma tabela de fidelidade (MAE, RMSE, PSNR, SSIM, contagens de regiões/âncoras, milissegundos). Não recebe argumentos; somente leitura.

```
$ osatura autotrace-bench
image          mode  colors  tol     MAE    RMSE   PSNR   SSIM regions strokes  anchors      ms
shapes         fill       6    1    2.19   12.10   26.5  0.975      11       0       33    13.6
...
```

---

## 10. Entrada ao vivo, MIDI, rastreamento facial e captura de movimento

### `live-map`

**Sinopse:** `osatura live-map <project.json> <events.json|song.mid> <mapping.json> <out.json> [--takes lib.json] [--tol 0.01] [--allow-empty]`

Transforma uma **performance ao vivo gravada** (MIDI, mouse, gamepad, teclas, sinais faciais) em canais com chaves, eventos e takes comuns (não destrutivo, tudo ou nada, bloqueios respeitados). A entrada é um Standard MIDI File `.mid` ou um JSON de eventos `{"events":[{"source","id","value","time"}]}` (`value` normalizado 0..1, `time` em segundos; `source` = `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`; `id` = MIDI `"<channel 1-16>:<number>"`, botão do mouse `"0"`, gamepad `"<pad>:<index>"`, tecla = código DOM como `KeyA`).

O arquivo de mapeamento tem `params` (source/id -> `target`+`prop`, com `in_min in_max out_min out_max deadzone invert curve smooth additive`) e `triggers` (cruzamento de limiar -> ação `jump_to_frame`, `play_take`, `set_key`, ...); chaves desconhecidas são erros. `--takes` é uma biblioteca de takes para as ações `play_take` (deve existir), `--tol` simplifica as chaves.

```
$ cat map.json
{"params":[{"source":"mouse_x","id":"0","target":"arm_up","prop":"bone_angle","out_min":-30,"out_max":30}],
 "triggers":[{"source":"key","id":"KeyA","action":{"type":"jump_to_frame","frame":3}}]}
$ osatura live-map small.json ev.json map.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 1 events (0 after the last frame skipped); wrote out.json
$ osatura live-map small.json song.mid mapm.json outm.json              # MIDI CC 1 on channel 1 -> arm_up
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote outm.json
$ osatura live-map small.json ev.json map.json out.json --takes lib.json    # lib.json missing
cannot read lib.json: No such file or directory (os error 2)             [exit 1]
$ osatura live-map small.json empty.json map.json out.json
note: map 0 (arm_up.BoneAngle): no matching events, nothing keyed
0 events -> +0 channels, ...; wrote out.json
```
**Pegadinha:** se eventos foram lidos mas **nada foi produzido** (erro de digitação em um id, source errado), o comando falha e não grava nada, a menos que se use `--allow-empty`; com zero eventos de entrada, ele apenas grava o projeto inalterado. Os gatilhos de salto são registrados como um evento chamado `jump` com `data.to` (o eixo de tempo não é reescrito).
**Limites (ROADMAP §9):** nenhuma entrada ao vivo de webcam/nível de áudio no studio por meio deste comando (`audio-react` é offline); o studio oferece o lado interativo.

### `drive`

**Sinopse:** `osatura drive <project.json> <signals.csv> <maps.json> <out.json> [--tol 0.5]`

Performance -> chaves a partir de um **CSV**: a linha de cabeçalho nomeia os sinais, cada linha seguinte é um quadro (índice da linha = quadro). `maps.json` é um array de `{input, target, prop, in_min, in_max, out_min, out_max, deadzone, invert, curve, smooth}`; `--tol` (padrão 0,5) simplifica as chaves.

```
$ printf 'jaw,speed\n0,0\n0.5,0.2\n1,0.4\n0.5,0.6\n' > sig.csv
$ echo '[{"input":"jaw","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":40,"smooth":0.2}]' > maps.json
$ osatura drive demo.json sig.csv maps.json dr.json --tol 0.5
wrote dr.json (+1 channels)
```

### `face-synth`, `face-train`, `face-eval`, `face-detect`, `face-track`, `face-calibrate`, `face-bake`

Um pipeline guiado por rosto que é **inteiramente local e treinado apenas com rostos sintéticos** (cabeças estilizadas/de desenho animado, 23 pontos de referência, layout `osatura23`). Um pequeno modelo treinado com dados sintéticos está embutido no binário; `--model` o substitui. **Fotografias, webcams e vídeo real não são validados** (ROADMAP §9; veja também [PATENT_NOTES.md](PATENT_NOTES.md): o puppeteering guiado por rosto não passou por triagem de patentes).

| Comando | Sinopse | O que faz |
|---|---|---|
| `face-synth` | `osatura face-synth <out-dir> [--frames 90] [--size 160] [--seed 7] [--fps 30]` | Animação sintética de cabeça: quadros PNG `000000.png...`, `truth.ndjson` (pontos de referência verdadeiros) e `params.json` (sinais verdadeiros); `--fps` é a taxa de quadros em que os sinais são temporizados (gravada no cabeçalho do NDJSON). |
| `face-train` | `osatura face-train --samples N --seed S --out model.bin [--mine-rounds 1] [--scene N] [--reg-win N] [--reg-cell N] [--det-win N] [--det-cell N] [--ridge-lambda X] [--stages N] [--refine-win N] [--stage-patch N] [--stage-cell N] [--stage-lambda X]` | Treina o detector próprio apenas com rostos sintéticos (sem downloads). `--scene` é o tamanho da imagem de treinamento (64..512); `--reg-win`, `--reg-cell`, `--det-win`, `--det-cell`, `--ridge-lambda`, `--stages`, `--refine-win`, `--stage-patch`, `--stage-cell` e `--stage-lambda` são os parâmetros de ajuste do tamanho do modelo e da regressão (os padrões são os usados no treinamento do modelo embutido). |
| `face-eval` | `osatura face-eval [--model m.bin] [--eval-n 200] [--seed 9] [--scene 128] [--report r.json] [--json]` | Precisão sintética em dados reservados, falsos positivos e varreduras de robustez (JSON; `--report` o grava, `--json` o imprime mesmo com `--report`; `--scene` é o tamanho da imagem de teste). |
| `face-detect` | `osatura face-detect <image> [--model m.bin] [--json] [--overlay o.png] [--threshold T] [--max-faces N] [--min-side N] [--max-side N] [--max-dim N]` | Caixas de rostos e 23 pontos de referência; `--overlay` os desenha em um PNG; `--min-side` / `--max-side` limitam o tamanho do rosto em pixels e `--max-dim` reduz antes uma imagem grande. |
| `face-track` | `osatura face-track <video-or-frame-dir> <out.ndjson> [--model m.bin] [--fps 30] [--max-dim 256]` | NDJSON de pontos de referência para `face-bake`. Um diretório de imagens é lido diretamente; um vídeo passa pelo ffmpeg. |
| `face-calibrate` | `osatura face-calibrate <landmarks.ndjson> <out.json> [--calibrate-frames N] [--map mapfile.json] [--fps F] [--layout L]` | Calcula a calibração do rosto neutro a partir dos primeiros N quadros. |
| `face-bake` | `osatura face-bake <project.json> <landmarks.ndjson> <mapfile.json> <out.json> [--calibrate-frames N\|0] [--calib calib.json] [--fps F] [--layout L] [--tol T] [--allow-empty]` | Incorpora sinais faciais (`mouth_open`, `eye_l_open`, `eye_r_open`, `brow_raise`, `yaw`, `pitch`, `roll`, ...) em canais com um arquivo de mapeamento no estilo de `live-map` cujo `source` é `face`. |

```
$ osatura face-synth syn --frames 6 --size 96 --seed 3
wrote 6 frames (96x96), truth.ndjson and params.json to syn
$ osatura face-train --samples 80 --seed 1 --out m.bin --scene 64        # 47 s in a debug build
wrote m.bin (649900 bytes) in 47.0s: 80 samples, seed 1, mined negatives 9
trained on synthetic faces only: expect stylised/cartoon faces to work, photographs and webcams are NOT validated
$ osatura face-train --samples 60 --seed 1 --out m.bin --scene 2
scene size must be 64..512                                                [exit 1]
$ osatura face-detect syn/000000.png --model m.bin
syn/000000.png: face 0: box centre (48.1, 47.4) side 69.0 score 4.49
$ osatura face-track syn trk.ndjson --model m.bin --fps 15 --max-dim 96
wrote trk.ndjson: 6 frames, face found in 6
$ osatura face-calibrate syn/truth.ndjson calib.json --calibrate-frames 3 --fps 15
6 frames, neutral from the first 3; wrote calib.json
$ echo '{"params":[{"source":"face","id":"mouth_open","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":60}],"triggers":[]}' > map.json
$ osatura face-bake small.json trk.ndjson map.json baked.json --calibrate-frames 3 --fps 15 --tol 0.5
6 landmark frames -> +1 channels (K keys in them), 0 trigger keys set, 0 events; wrote baked.json
$ osatura face-eval --eval-n 6 --model m.bin --report ev.json            # writes the JSON report
```
Esses comandos gravam seus arquivos de modelo/relatório com o gravador atômico que recusa links simbólicos (as pastas pai são criadas). `face-bake` e `live-map` informam `+N channels (K keys in them), M trigger keys set` e emitem um aviso em stderr quando a gravação é mais longa que o projeto (os quadros extras não são incorporados; a duração do projeto permanece inalterada).

### `mocap-info`, `mocap-match`, `mocap-retarget`

```
osatura mocap-info     <file.bvh>
osatura mocap-match    <project.json> <file.bvh> [--map map.json]
osatura mocap-retarget <project.json> <file.bvh> <out.json> [--view front|side|top|yaw=DEG] [--map map.json] [--scale auto|N] [--fps F]
    [--smooth N] [--start F] [--end F] [--rest-frame F] [--min-conf C] [--weight W] [--mirror] [--no-root] [--in-place] [--keep-length] [--footlock BONE --ground Y]
```
* **`mocap-info`** imprime a hierarquia, os canais, a taxa de quadros e a extensão de um arquivo BVH.
  ```
  $ osatura mocap-info m.bvh
  4 joints (2 end sites), 15 channels per frame
  4 frames at 0.0417 s (24.000 fps), duration 0.125 s
  Hips  offset (0.000, 0.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
    LeftUpLeg  offset (5.000, 0.000, 0.000)  [Zrot Xrot Yrot]
  ...
  ```
* **`mocap-match`** imprime a **tabela automática de correspondência articulação -> osso** (slot, articulação BVH, osso do rig, confiança, evidência `via`, razão de comprimento, nota), depois as articulações/ossos sem correspondência e os avisos. Nomes e topologia são ambos usados. `--map` é um objeto JSON `{"bvh joint": "rig bone" | null}` (`null` ou `""` exclui uma articulação; o encapsulamento `{"map": {...}}` é aceito). Sempre termina com código 0 - quando nada corresponde, apenas avisa `no bones matched: supply a --map file`.
  ```
  $ osatura mocap-match demo.json m.bvh --map map2.json         # {"Hips":null,"LeftUpLeg":"thigh","LeftLeg":"shin"}
  slot         bvh joint                  rig bone                conf  via         len x  note
  shin.l       LeftLeg                    shin                    1.00  manual      12.00  manual
  thigh.l      LeftUpLeg                  thigh                   1.00  manual      12.00  manual
  unmatched BVH joints: Hips, RightUpLeg
  ...
  ```
* **`mocap-retarget`** incorpora o movimento como canais **acrescentados**: projeta as rotações 3D na `--view` 2D (`front` por padrão, `side`, `top` ou `yaw=DEG`), escala o movimento da raiz (`--scale auto` ou um número), reamostra para `--fps` (por padrão o do projeto), suaviza (`--smooth` quadros), usa os quadros BVH de `--start..--end`, usa `--rest-frame` como pose de repouso, descarta correspondências abaixo de `--min-conf`, mistura com `--weight`, espelha com `--mirror`, descarta o movimento da raiz com `--no-root`, converte um clipe de deslocamento em um no lugar com `--in-place` e fixa um pé com `--footlock BONE --ground Y`. Imprime a tabela, as notas e `N bones, F frames at R fps, root scale S, +C channels; wrote <out>`. Um clipe mais longo que o projeto **estende a duração do projeto**; uma nota após o resumo informa de qual para qual duração. `--keep-length` mantém a duração do projeto e trunca as chaves no último quadro (uma chave interpolada é colocada ali; uma nota conta as chaves não incorporadas). Sem nada acima do limiar de confiança: `no bone matches to retarget (all below the confidence threshold?)` (exit 1). Observação: para este comando `--scale` é `auto|N`, não a escala de renderização habitual.
  ```
  $ osatura mocap-retarget demo.json m.bvh mr.json --map map2.json --view side --smooth 1 --in-place --no-root
  ... (table)
  2 bones, 4 frames at 24 fps, root scale 12.0000, +2 channels; wrote mr.json
  ```
**Limites (ROADMAP §9):** nenhum arquivo real de captura de movimento foi testado (nenhum é fornecido: os exemplos usam um BVH de 4 quadros escrito à mão); a projeção 2D descarta a profundidade (um osso apontado para a câmera mantém seu último ângulo).

---

## 11. Máquinas de estados, vinculações de dados, contêineres e regiões de rolagem

Estes comandos gerenciam as partes de **interação em tempo de execução** de um projeto: máquinas de estados (`machines`), vinculações de dados
(`bindings`), contêineres de layout (`containers`) e regiões de rolagem (`scrolls`). Todos armazenam seus dados dentro
do arquivo de projeto; as formas `add` leem um pequeno arquivo JSON auxiliar, validam-no e gravam um novo projeto.
O próprio modelo de runtime está documentado em [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

### `machine` (add, list, remove, validate)

**Sinopse:**
```
osatura machine add      <project.json> <out.json> <machine.json>
osatura machine list     <project.json>
osatura machine remove   <project.json> <out.json> <name>
osatura machine validate <project.json>
```

Uma **máquina** tem `inputs` nomeados (`bool`, `number` ou `trigger`) e uma ou mais `layers`; cada camada tem um estado
`entry`, `states` e `transitions`. Um estado reproduz uma **origem**: `{"kind":"segment","start":F,"end":F}` (quadros dos
canais com chaves do projeto), `{"kind":"clip","clip":NAME}`, `{"kind":"region","marker":NAME}` ou uma blend tree 1-D
`{"kind":"blend","input":NAME,"children":[{"threshold":T,"source":{...}}]}`. Campos do estado: `name`, `source`,
`mode` (`loop` por padrão, `once`, `ping_pong`), `speed` (1), `blend_in` (segundos). Campos da transição: `from` (um estado ou
`"*"`), `to`, `when` (uma lista de `{"input":NAME,"op":eq|ne|lt|le|gt|ge,"value":V}`, todas devem valer; `{"input":"go"}`
sozinho significa "igual a 1"), `exit_time` (0..1), `duration` (segundos de crossfade) e `ease`. Chaves desconhecidas em uma transição
são um **erro** (um erro de digitação como `conditions` transformaria, de outro modo, uma transição condicionada em incondicional).

```
$ cat mach.json
{"name":"m","inputs":[{"name":"go","kind":"bool"}],"layers":[{"name":"base","entry":"idle","states":[
 {"name":"idle","source":{"kind":"segment","start":0,"end":12}},
 {"name":"act","source":{"kind":"segment","start":12,"end":36}}],
 "transitions":[{"from":"idle","to":"act","when":[{"input":"go"}],"duration":0.1}]}]}
$ osatura machine add demo.json d1.json mach.json
added machine 'm' (1 machines); wrote d1.json
$ osatura machine list d1.json
m  inputs [go:bool]
  layer base (entry idle, mask []): 2 states, 1 transitions
$ osatura machine validate d1.json
1 machines, 0 bindings: ok
```

`machine remove` recusa remover uma máquina que uma vinculação ainda controla
(`binding 'goBind' still drives machine 'm'`, exit 1): remova primeiro a vinculação.
**Erros:** uma máquina inválida (estado desconhecido, nomes duplicados, referência de entrada incorreta) é rejeitada com uma mensagem e
nada é gravado. **Relacionados:** `machine-sim`, `bind`, `state-bake` (o formato de máquina mais antigo, baseado em quadros).
**Limites:** as máquinas são avaliadas pelo runtime e por `machine-sim`; os exportadores não incorporam nada disso (Lottie, GIF
e os demais arquivos contêm apenas quadros, veja [EXPORT_FORMATS.md](EXPORT_FORMATS.md)).

### `machine-sim`

**Sinopse:** `osatura machine-sim <project.json> <script.json> <out_dir|out.json> [--fps N --svg-frames N]`

Reproduz eventos de entrada temporizados em uma máquina e registra o estado de cada tick. O script é
`{"machine": NAME?, "duration": SECONDS?, "events": [{"t": SECONDS, "set": {"input": value}, "trigger": ["name"],
"value": {"external.path": number}}]}`. `machine` assume por padrão a única máquina do projeto, `duration` o último
evento mais um segundo; `value` alimenta vinculações de dados (veja `bind`). O `--fps` padrão é o fps do projeto.
`--svg-frames N` renderiza adicionalmente N quadros SVG distribuídos ao longo da execução.

```
$ cat sc.json
{"duration":1.5,"events":[{"t":0.5,"value":{"hud.go":1}}]}
$ osatura machine-sim d3.json sc.json sim.json --fps 24 --svg-frames 0
   0.000s  base     Entered idle
   0.500s  base     Exited idle
   0.500s  base     Entered act
37 ticks at 24 fps, 3 events, 0 svg frames; wrote sim.json
$ osatura machine-sim d3.json sc.json simdir --svg-frames 2     # a name that does not end in .json is a folder
...
37 ticks at 24 fps, 3 events, 2 svg frames; wrote simdir/sim.json
$ ls simdir
frame_00000.svg  frame_00036.svg  sim.json
```
`sim.json` contém `machine`, `fps`, `ticks[]` (`tick`, `time`, `states` como pares `[layer,state]`, `fading`) e o
registro de eventos. **Relacionados:** `bind`, `machine`.

### `state-bake`

**Sinopse:** `osatura state-bake <project.json> <machine.json> <events.json> <out.json> [--tol 0.01]`

Incorpora uma máquina de estados **baseada em quadros** (um formato diferente e mais antigo que `machine`) em canais comuns com chaves.
`machine.json` é `{"initial":NAME,"states":[{"name","length","looped","channels":[...]}],"transitions":[{"from":NAME
or "*","to","when":COND,"fade":FRAMES,"ease"}]}`, em que `COND` é um de `{"type":"event","name"}`,
`{"type":"above"|"below"|"equals","input","value"}`, `{"type":"finished"}`, `{"type":"after","frames"}`.
`events.json` é uma lista de `{"frame":N,"name":NAME,"value":V}`. O resultado é acrescentado aos canais do projeto
(`--tol` é a tolerância de redução de chaves), o projeto é compilado e gravado.

```
$ echo '[{"frame":10,"name":"go"}]' > ev.json
$ osatura state-bake demo.json sm.json ev.json baked.json --tol 0.01
wrote baked.json (+1 channels)
```
**Pegadinha:** o canal incorporado é acrescentado depois dos existentes; os canais são executados na ordem da lista, portanto remova ou silencie o
canal original do mesmo alvo se a incorporação deve prevalecer. Um arquivo de máquina no formato `machine`
(com `layers`) é recusado: `invalid JSON in mach.json: missing field 'initial'`.

### `bind` (add, list, remove) e `bind-eval`

**Sinopse:**
```
osatura bind add    <project.json> <out.json> <name> <value.path> <target:prop|@machine:input>
                     [--expr E --remap a,b,c,d --no-clamp --smooth S --mode replace|add|multiply --weight W]
osatura bind list   <project.json>
osatura bind remove <project.json> <out.json> <name>
osatura bind-eval   <project.json> <values.json> [--data out.json]
```

Uma **vinculação de dados** mapeia um caminho de valor externo (por exemplo `hud.size`) para uma propriedade do projeto (`sun:layer_opacity`;
os nomes de propriedade são os em snake_case da seção 1) ou para uma entrada de máquina (`@m:go`). `--remap a,b,c,d` mapeia a
faixa de entrada `a..b` para `c..d` (limitada, a menos que se use `--no-clamp`); `--expr` é uma expressão sobre o valor;
`--smooth S` aplica um passa-baixa ao longo de S segundos; `--mode` e `--weight` controlam como o resultado se combina com os canais.

```
$ osatura bind add d1.json d2.json goBind hud.go @m:go
added binding 'goBind' (1 bindings); wrote d2.json
$ osatura bind add d2.json d3.json sz hud.size sun:layer_opacity --remap 0,100,0.2,1 --smooth 0.1
added binding 'sz' (2 bindings); wrote d3.json
$ osatura bind list d3.json
goBind  hud.go -> @m:go
sz  hud.size -> sun:LayerOpacity remap 0..100 -> 0.2..1 clamp smooth 0.1s
$ echo '{"hud.go":1,"hud.size":50}' > vals.json
$ osatura bind-eval d3.json vals.json --data bv.json
goBind = 1
sz = 0.6000000238418579
$ osatura bind add d2.json x.json sz hud.size sun:scale
unknown property 'scale'                                                  [exit 1]
$ osatura bind remove d3.json d7.json sz
removed binding 'sz'
```
`bind-eval` imprime a saída de cada vinculação para os valores dados (`--data` também os grava como JSON). `list` imprime os nomes
de propriedade em CamelCase; a linha de comando sempre recebe snake_case. **Relacionados:** `machine`, `machine-sim` (seu mapa `value`).

### `container` (add, list, remove, solve)

**Sinopse:**
```
osatura container add    <project.json> <out.json> <container.json>
osatura container list   <project.json>
osatura container remove <project.json> <out.json> <id>
osatura container solve  <project.json> [--size WxH --data out.json]
```

Um **contêiner** é uma caixa de layout semelhante ao flexbox: `id`, `members`, `axis` (`row`|`column`), `gap`, `cross_gap`, `padding`
(`[left,top,right,bottom]`), `justify` (`start|center|end|space_between|space_around`), `align`
(`start|center|end|stretch`), `wrap` (`none|wrap`) e `rect`, uma regra para seu próprio retângulo na tela:
`{"kind":"fixed",...}`, `{"kind":"pin","left","right","top","bottom","width","height"}` (distâncias a partir das bordas da tela; quaisquer duas
de cada eixo), `{"kind":"aspect",...}` ou `{"kind":"hug"}`. Um **membro** é exatamente um de `{"layer":PATH}`,
`{"container":ID}` ou `{"spacer":true}`, com `grow`, `shrink`, `width`, `height`, `min_w/max_w/min_h/max_h` e
uma substituição de `align`; `"resize":true` permite que o solver escale a camada para o seu espaço.

```
$ cat cont.json
{"id":"row1","axis":"row","gap":10,"padding":[10,10,10,10],
 "rect":{"kind":"pin","left":10,"right":10,"top":10,"height":100},
 "members":[{"layer":"sun"},{"layer":"blush","grow":1,"resize":true}]}
$ osatura container add d3.json d4.json cont.json
added container 'row1' (1 containers); wrote d4.json
$ osatura container list d4.json
row1: Row, 2 members, gap 10, justify Start, align Start, wrap None, rect {"kind":"pin","left":10.0,...}
$ osatura container solve d4.json --size 800x480 --data solved.json
canvas 800x480
container row1: x 10.00 y 10.00 w 780.00 h 100.00
layer sun: dx -430.000 dy 0.000 scale 1.0000 x 1.0000
layer blush: dx 280.432 dy -158.242 scale 33.8889 x 1.0000
```
Sem `--size`, usa-se o tamanho do próprio projeto. `solve` imprime o retângulo de cada contêiner e, por camada, o deslocamento e a
escala que o layout aplica sobre a animação. **Erros:** uma camada desconhecida, um contêiner aninhado ausente ou um
ciclo é rejeitado por `add` (nada é gravado). **Relacionados:** `scroll`, `layout-preview`.
**Limites:** o layout é aplicado pelo runtime/pré-visualização; os arquivos exportados contêm o layout resolvido para o tamanho do próprio projeto.

### `scroll` (add, list, remove, eval)

**Sinopse:**
```
osatura scroll add    <project.json> <out.json> <scroll.json>
osatura scroll list   <project.json>
osatura scroll remove <project.json> <out.json> <id>
osatura scroll eval   <project.json> <id> [--offset X,Y --size WxH --velocity VX,VY --time S]
```

Uma **região de rolagem** tem `id`, `content` (o id de um contêiner que não é membro de outro contêiner), `viewport`
(uma regra de retângulo como acima), `axis` (`x`, `y` ou ambos), `offset`, `elastic` (`{"limit","stiffness","damping_ratio"}`; efeito de elástico
em vez de um limite rígido), `snap_x`/`snap_y` (deslocamentos de repouso), `fling` (segundos da velocidade de soltura projetados para encontrar
o ponto de aterrissagem, padrão 0,15), `clip_layer` (uma camada retangular usada como máscara do conteúdo) e `input` (um caminho de valor externo que
controla o deslocamento).

```
$ cat scroll.json
{"id":"sc1","content":"row1","axis":"x","viewport":{"kind":"pin","left":0,"right":0,"top":0,"height":120},
 "elastic":{"limit":60,"stiffness":200,"damping_ratio":0.8},"snap_x":[0,100,200]}
$ osatura scroll add d4.json d5.json scroll.json
added scroll 'sc1' (1 scrolls); wrote d5.json
$ osatura scroll list d5.json
sc1: content row1, axis X, offset [0.0, 0.0], elastic true, snap x [0.0, 100.0, 200.0] y [], clip -
$ osatura scroll eval d5.json sc1 --offset 400,0
viewport [0.0, 0.0, 640.0, 120.0] content [-59.92364, 0.0, 640.0, 120.0] max [0.0, 0.0]
requested offset [400.0, 0.0] -> applied [59.92364, 0.0]
$ osatura scroll eval d5.json sc1 --offset 80,0 --velocity -500,0 --time 0.5
...
requested offset [80.0, 0.0] -> applied [44.184174, 0.0]
t 0.100s offset [22.794, 0.000]
t 0.300s offset [0.051, 0.000]
t 0.500s offset [-0.197, 0.000]
```
Sem `--velocity`, `eval` mostra o deslocamento limitado (ou com efeito de elástico) correspondente ao solicitado. Com `--velocity`,
simula a soltura (momento, snap, mola) durante `--time` segundos e imprime a cada 0,1 s. `--size WxH` avalia para outro tamanho de tela.
No exemplo, o conteúdo tem a mesma largura que a viewport, de modo que tudo além de 0 é rolagem excedente e mostra o efeito de elástico.

### `layout-preview`

**Sinopse:** `osatura layout-preview <project.json> <out.svg> --size WxH [--frame N --scroll id=X,Y]`

Renderiza um quadro com o layout **resolvido novamente para outro tamanho de tela** e os contornos dos contêineres desenhados, para que você possa
verificar como um design se reorganiza em um celular ou em uma tela larga. `--scroll id=X,Y` define um deslocamento de rolagem (repita para várias regiões).

```
$ osatura layout-preview d5.json lp.svg --size 800x480 --scroll sc1=50,0
wrote lp.svg (800x480, 1 containers, 2 layers placed)
```

## 12. Efeitos, profundidade de campo, onion skin, trilhas e câmera

### `fx`

**Sinopse:** `osatura fx <project.json> <frame> <chain.json> <out.png> [--scale N] [--layer-normals] [--clamp]`

Um `<frame>` fora do projeto é um erro; `--clamp` renderiza o quadro válido mais próximo.

Renderiza um quadro e executa sobre ele uma **cadeia de pós-efeitos**. `chain.json` é uma lista de efeitos, cada um com um `type`:
`blur {radius}`, `drop_shadow {dx,dy,blur,color,opacity}`, `glow {radius,strength,color?}`, `outline {width,color}`,
`posterize {levels}`, `pixelate {size}`, `levels {in_black,in_white,gamma,out_black?,out_white?}`, `fade {amount}` e
`relight {ambient,bevel,height,lights:[{x,y,z,color?,intensity?,radius?,cast_shadow?,spot?,falloff?}],ground_shadow?,
normal_map?,ambient_color?}`. Sem um `normal_map`, o relight sombreia um bisel feito a partir do canal alfa desfocado.

```
$ cat chain.json
[{"type":"drop_shadow","dx":6,"dy":6,"blur":3,"color":"#000000","opacity":0.5},{"type":"glow","radius":6,"strength":0.5}]
$ osatura fx demo.json 12 chain.json fx.png --scale 0.5
wrote fx.png (2 effect(s))
$ cat relight.json
[{"type":"relight","ambient":0.4,"bevel":6,"height":8,"lights":[{"x":300,"y":100,"z":120,"intensity":1.0}],"ground_shadow":0.3}]
$ osatura fx demo.json 12 relight.json lit.png --scale 0.5
wrote lit.png (1 effect(s))
```
`--layer-normals` sombreia cada camada com seu próprio mapa de normais (veja `layer-normal`) em vez de um único mapa para a imagem inteira.
Uma luz que omite `z` é rejeitada (`missing field 'z'`). **Limites:** a cadeia é um pipeline de imagens estáticas somente em PNG; nenhum destes
efeitos chega ao Lottie, ao GIF nem ao player HTML (o exportador Lottie tem seu próprio subconjunto de efeitos, veja EXPORT_FORMATS.md).

### `color-space`

**Sinopse:** `osatura color-space <project.json> <out.json> srgb|linear`

Define o `color_space` do projeto, o espaço no qual o compositor raster (PNG, GIF, APNG, vídeo, sprite sheet, `compare`, `ci`) e o compositor em ponto flutuante (`render-hdr`) fazem a mesclagem. `srgb` (o padrão, e com o qual todos os golden frames foram gravados) mescla os valores armazenados codificados em sRGB; `linear` converte todas as cores (preenchimentos, contornos, paradas de gradiente, fundo, pixels de imagem) para luz linear, compõe nesse espaço em f32 e converte para sRGB uma única vez no final. Uma camada preta a 50 % sobre branco resulta então em 188/255 em vez de 128/255, e os gradientes interpolam em luz linear. Com `srgb` a chave não é gravada, de modo que arquivos existentes e golden frames não mudam.

```
$ osatura color-space demo.json lin.json linear
wrote lin.json (color_space: linear)
$ osatura render lin.json 0 lin.png
wrote lin.png
$ osatura color-space lin.json x.json p3
unknown colour space 'p3' (srgb or linear)
```
**Limites:** o renderizador de 8 bits passa para o compositor em ponto flutuante nos projetos lineares (rasterizador de cobertura próprio, de modo que as bordas com antialiasing diferem ligeiramente da saída sRGB); as cores dos efeitos de camada (sombra, glow, tint) são usadas como escritas, sem conversão; `dof` e a cadeia de pós-efeitos (`fx`) permanecem em sRGB; SVG, Lottie e o renderizador por GPU ignoram a configuração; a reamostragem de imagens ocorre em sRGB antes da conversão. Os modos de mesclagem são aplicados aos valores lineares.

### `layer-normal`

**Sinopse:** `osatura layer-normal <project.json> <out.json> <layer> <normal.png|none> [--strength S] [--fit stretch|tile|exact] [--green-down]`

Anexa (ou, com `none`, remove) um mapa de normais em espaço tangente a uma camada de imagem para `fx ... --layer-normals`.
`--fit` indica como um mapa de outro tamanho é ajustado (`stretch` por padrão, `tile`, `exact`); `--green-down` inverte o canal
verde (mapas no estilo DirectX); `--strength` escala a inclinação (1 = como armazenado, 0 = plano).

```
$ osatura layer-normal demo.json dn.json sun n.png --strength 1 --fit stretch
wrote dn.json
$ osatura fx dn.json 12 chain.json fxn.png --scale 0.5 --layer-normals
wrote fxn.png (2 effect(s))
$ osatura layer-normal dn.json dn2.json sun none
wrote dn2.json
```

### `onion` (duas formas)

**Sinopse:**
```
osatura onion <project.json> <frame> <out.png> [--before 2 --after 1 --step 1 --scale N]
osatura onion <project.json> <out.svg> --frame N [--before B --after A --step S --keyed --outline --all-parts
               --layers a,b --tint-before C --tint-after C --opacity X --falloff X --ghosts-only]
```
A primeira forma grava um PNG com fantasmas esmaecidos dos quadros anteriores e posteriores. A segunda grava um SVG estático: o
quadro com fantasmas de onion skin coloridos sobre ele. `--keyed` usa os **quadros-chave** mais próximos em vez de passos fixos;
`--outline` desenha os fantasmas como contornos; `--all-parts` cria fantasmas de todas as partes (padrão: partes em movimento), `--layers` os limita,
`--ghosts-only` omite o próprio quadro.

```
$ osatura onion demo.json 12 on.png --before 2 --after 1 --step 2 --scale 0.5
wrote on.png
$ osatura onion demo.json on.svg --frame 12 --before 2 --after 2 --step 3 --outline --opacity 0.4
wrote on.svg (4 ghost(s) around frame 12)
$ osatura onion demo.json on2.svg --frame 12 --keyed --ghosts-only
wrote on2.svg (3 ghost(s) around frame 12)
```
A forma é escolhida pelo segundo argumento: um número significa a forma PNG, um nome terminado em `.svg` a forma SVG.

### `trail` (duas formas)

**Sinopse:**
```
osatura trail <project.json> <bone> <out.svg> [--from F --to F]
osatura trail <project.json> [out.svg] --target layer:PATH|bone:NAME[:head|tip]|point:LAYER:KIND:A[:B] [--from F --to F --smooth N --json]
```
Desenha a **trilha de movimento** da ponta de um osso (primeira forma) ou de qualquer alvo (segunda forma: uma camada, a cabeça/ponta de um osso ou um
ponto de deformador), com uma marca por quadro e marcadores nas chaves. `--smooth N` suaviza a curva; `--json` imprime a
curva amostrada como JSON (e então `out.svg` é opcional).

```
$ osatura trail demo.json hand tr.svg --from 0 --to 24
wrote tr.svg (25 points)
$ osatura trail demo.json tr3.svg --target layer:sun --from 0 --to 24
wrote tr3.svg (25 ticks, 0 key marker(s))
$ osatura trail demo.json tr2.svg --target bone:hand:tip --from 0 --to 24 --smooth 2 --json
{ "curve": [ [212.8, 83.84], [213.77, 83.99], ...
```

### `dof` e `dof-export`

**Sinopse:**
```
osatura dof        <project.json> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24 --scale N --clamp]
osatura dof-export <project.json> <out.gif|out.apng|out.mp4|out.webm|out-folder> [--from F --to F --focus N --strength N --max-radius N --scale N --codec vp9|av1]
```
Um `<frame>` fora do projeto é um erro para `dof` (e para `render-hdr` e `fx`); `--clamp` renderiza em vez disso o quadro válido mais próximo, como em `render`.

Profundidade de campo: cada **camada de nível superior** é desfocada por `|parallax - focus|` vezes `strength` (pixels por unidade de
distância de parallax, limitado por `--max-radius`). Sem `--focus` / `--strength`, os valores seguem os canais com chaves
`camera_focus` / `camera_blur` (foco padrão 1.0). As camadas obtêm sua profundidade de `parallax` (veja a seção 5).
`dof-export` renderiza o intervalo `--from`..`--to` (números de quadro inteiros; padrão: de 0 até o fim) para um GIF animado, um
PNG animado, um vídeo `.mp4`/`.webm` (requer `ffmpeg`; `--codec` assume `vp9` por padrão; nenhum áudio é incluído) ou, para um nome sem
extensão, uma pasta de PNGs numerados (`f0000.png`...). Qualquer outra extensão é recusada (`dof-export writes .gif, .apng,
.mp4, .webm or a folder (no extension)`, nada é gravado).

```
$ osatura dof demo.json 12 dof.png --focus 1.0 --strength 12 --max-radius 24 --scale 0.5
wrote dof.png
$ osatura dof-export demo.json dof.gif --from 0 --to 6 --scale 0.25
wrote dof.gif
$ osatura dof-export demo.json dofdir --from 0 --to 2 --scale 0.25      # dofdir/f0000.png ... f0002.png
wrote dofdir
```
**Relacionados:** `parallax`, `render --proxy`, `shot`/`camera-seq` para movimentos de câmera. **Limites:** o desfoque é uma
aproximação por camada, não um modelo de lente (sem formas de bokeh, sem tratamento de oclusão); não é exportado para Lottie.

## 13. Nível de detalhe, relatórios de tamanho e export-fit

### `size-report`

**Sinopse:** `osatura size-report <project.json> [--tol T] [--json]`

Mostra **onde estão os bytes**: o arquivo de projeto, o Lottie incorporado e a página HTML, por camada, imagem e canal
(chaves, e quantas são removíveis na tolerância `--tol`, padrão 0,05).

```
$ osatura size-report small.json
project file: 17959 bytes (3511 gzip)
  by layer (JSON bytes; embedded image base64 in brackets):
         2983  arm
         2053  leg
  ...
  by channel (keys, removable at tol 0.05):
          420  foot_ctrl.BoneOffsetX  keys 5 removable 1
  ...
baked Lottie: 46843 bytes (11180 gzip), by layer:
         9181  arm
```
`--json` imprime os mesmos dados como JSON (`html.defs_bytes_total`, `frame_bytes_max`, ...). Use-o antes de `export-fit` ou `lod`
para ver qual camada simplificar.

### `lod`

**Sinopse:** `osatura lod <project.json> --tolerance-px 0.5 [--scale S | --size WxH] [--levels 4] [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]`

Constrói **níveis de detalhe** L1..Ln, cada um com um *limite de erro declarado em pixels de saída*: simplificação de caminhos, redução
de chaves, colapso de malha e descarte. O nível k tem tolerância `--tolerance-px x ratio^(k-1)`. `--scale` ou `--size` indica
em que tamanho a saída é exibida (padrão 1 px por unidade). `--out-dir` (padrão `.`) recebe `<name>_L1.json` ... e
`--report` um relatório JSON. `--verify` renderiza `--frames N` quadros e **mede** o desvio real (exit 1 se estiver acima do
limite).

```
$ osatura lod small.json --tolerance-px 1 --levels 3 --out-dir lv --report rep.json --verify --frames 4
level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  path  mesh  keys  cull  measured_px  ratio
L0      0.000       74       0    37      14    17959       0.0000 ...                              -      -
L1      1.000       70       0    37      14    17651       0.3861 0.386 0.000 0.000 0.000       0.3342   1.2x
L2      2.000       69       0    37      14    17573       0.9277 ...                           0.8185   1.1x
L3      4.000       65       0    37      14    17251       1.5372 ...                           1.3367   1.1x
scale 1 px per unit; level k tolerance = 1 px x 2^(k-1); files lv/s_L1..L3.json
  L1: keys: 3 of 8 keyed channels have a documented lever, 0 reduced
  L1: removed 4 path anchor(s)
$ ls lv
s_L1.json  s_L2.json  s_L3.json
```
**Pegadinha:** a economia em uma demonstração pequena, feita à mão, é mínima (cerca de 4 % a 4 px); os LODs compensam em arte vetorizada ou com muitas malhas.
O relatório e os arquivos de nível são gravados com uma gravação simples (não atômica).

### `lod-pick`

**Sinopse:** `osatura lod-pick <project.json> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--out chosen.json]`

Escolhe o **nível mais grosseiro** cujo limite declarado permaneça dentro de `--max-px` pixels quando a arte é exibida em uma
tela de `--screen-px` (a escala é o tamanho da tela sobre o tamanho do projeto). `--out` grava esse nível.

```
$ osatura lod-pick small.json --screen-px 320x240 --out chosen.json
level  tol_px  anchors ...
screen 320x240 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.4638 px, 69 anchors, 0 mesh vertices, 37 keys)
wrote chosen.json
$ osatura lod-pick small.json --screen-px 64x48 --max-px 2 --out chosen2.json
screen 64x48 -> scale 0.1000 px/unit; budget 2 px -> level L4 (declared 0.5310 px, 62 anchors, 0 mesh vertices, 37 keys)
```

### `export-fit`

**Sinopse:** `osatura export-fit <project.json> <out> --max-size 500KB [--format lottie|html|gif|apng|webm|project] [--min-ssim X] [--max-seconds 60] [--measure raw|gzip] [--frames N] [--search ladder|descent|grid] [--report out.json] [--pareto out.csv] [--audit] [--no-baseline]`

Encontra a **exportação de melhor qualidade que cabe em um orçamento de bytes**: pesquisa os parâmetros de qualidade do exportador (Lottie: `decimals`,
`path_eps`, `key_tol`, `frame_step`; os outros formatos têm os seus) e pontua cada candidato com SSIM em relação à
renderização original. `--max-size` aceita `500KB` (1000 bytes), `1.5MB`, `64000`, `200KiB` (1024 bytes). O formato vem de
`--format` ou da extensão de saída (`.json` é Lottie, `.html`, `.gif`, `.apng`, `.webm`); `project` grava um JSON de
projeto enxuto. `--measure gzip` orça o tamanho em gzip (para servidores que comprimem). `--min-ssim` é um piso de qualidade.
`--max-seconds` é um **orçamento real de tempo de relógio** (padrão 60; `0` = ilimitado): ao expirar, o melhor resultado até então é mantido.
`--frames N` é o número de quadros amostrados para o SSIM (padrão 8). `--search` escolhe a estratégia (`descent` por padrão).
`--report` / `--pareto` gravam o registro da busca como JSON / CSV; `--audit` imprime todos os candidatos; `--no-baseline` pula
a comparação com as configurações padrão.

```
$ osatura export-fit small.json fit.json --max-size 30KB --format lottie --report fr.json
lottie -> fit.json: 25756 bytes raw, 5736 gzip (budget 30000 raw); SSIM mean 0.9699, worst 0.9308 at frame 5, max CIEDE2000 68.67; 5 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (...)
$ osatura export-fit small.json fit.html --max-size 60KB --format html --max-seconds 20 --frames 4
html -> fit.html: 59166 bytes raw, 8550 gzip (budget 60000 raw); SSIM mean 1.0000, worst 1.0000 at frame 5, max CIEDE2000 8.54; 1 candidates built
existing exporter at defaults: 66391 bytes raw, 8933 gzip
$ osatura export-fit small.json fit2.json --max-size 1KB --format lottie --max-seconds 10 --frames 3
lottie -> fit2.json: 14751 bytes raw, 2418 gzip (budget 1000 raw); ...
knobs at their floor: decimals, path_eps, key_tol, frame_step
budget not met: the smallest export found is 14751 bytes (raw), above the budget of 1000 bytes: ...          [exit 1]
$ osatura export-fit small.json fit.xyz --max-size 1KB
cannot tell the format from the output name; pass --format lottie|html|gif|apng|webm|project   [exit 1]
```
**Códigos de saída:** 0 quando o resultado cabe no orçamento (mesmo que o orçamento de tempo tenha interrompido a busca); 1 quando o orçamento
não pode ser atendido (a menor exportação ainda é gravada) ou em caso de argumentos inválidos. **Pegadinhas:** trabalha sobre a saída incorporada,
portanto não consegue remover mais do que os parâmetros permitem (um Lottie de 1 KB é impossível para um rig de 14 camadas); cada candidato é uma
exportação completa, de modo que projetos grandes precisam de um `--max-seconds` generoso (um build de depuração é muito mais lento que um build de release). Os arquivos de
relatório são gravados com uma gravação simples. **Relacionados:** `size-report`, `lod`, `export` (seção 4), EXPORT_FORMATS.md (dicas de tamanho).

## 14. Garantia de qualidade

Relacionados, documentados anteriormente: `check` (validação, seção 3), `diff` (diff semântico, seção 3), `compare` (diff de imagem de duas
renderizações, seção 4).

### `lint`

**Sinopse:** `osatura lint <project.json>`

Verificações estáticas de rig e de animação. Imprime uma linha por achado: `Severity CODE [target:property] message`. Exit 1 se algum
achado tiver severidade Error ou superior; Info e Warning terminam com exit 0.
```
$ osatura lint small.json
Info L007 [:CameraX] key at frame 24 lies outside the 0..8 timeline
Info L007 [arm_low:BoneAngle] key at frame 12 lies outside the 0..8 timeline
```
`L007` é uma chave fora da timeline (aqui porque `small.json` é a demonstração cortada para 8 quadros).

### `contracts`

**Sinopse:** `osatura contracts <project.json> <contracts.json>`

**Contratos de movimento**: testes unitários para animação. `contracts.json` é uma lista de contratos com um `type`:
`loop_seamless {tol}`, `tip_range {bone,axis,min,max}`, `max_speed {bone,px_per_frame}` (existem outros tipos no
enum `Contract` do motor). As violações são impressas em stdout, a contagem em stderr, e o código de saída é 1.
```
$ cat contracts.json
[{"type":"loop_seamless","tol":0.5},{"type":"tip_range","bone":"hand","axis":"y","min":0,"max":400},{"type":"max_speed","bone":"hand","px_per_frame":3}]
$ osatura contracts small.json contracts.json
contract 0 violated at frame 8: bone 'arm_up' ends 35.4 px away from where it started (tolerance 0.5)
...
contract 2 violated at frame 6: tip of 'hand' moves 3.3 px/frame (limit 3)
7 violation(s)                                                            [stderr, exit 1]
```

### `coach`

**Sinopse:** `osatura coach <project.json>`

Sugestões de timing e de princípios de animação (overlap, antecipação, easing). Apenas conselhos, sempre exit 0.
```
$ osatura coach small.json
C003 [, arm_low, arm_up, foot_ctrl, hand, mouth, target] frame 0: 7 parts start moving on frame 0; offset them by a few frames (overlap) for a more natural feel
```

### `safety`

**Sinopse:** `osatura safety <project.json> [--scale N]`

Relatório de flashes, flashes vermelhos e movimento de câmera (uma aproximação da WCAG 2.3.1; um auxílio de engenharia, não uma certificação
de broadcast). Renderiza cada quadro em `--scale` (use um valor pequeno para projetos longos).
```
$ osatura safety small.json --scale 0.25
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.06 widths/s, rotation 0°/s, zoom ×1.00/s
```

### `reduced-motion`

**Sinopse:** `osatura reduced-motion <project.json> <out.json> [--factor 0.25]`

Grava uma cópia mais calma: o movimento de câmera e o wobble são escalados por `--factor` (padrão 0,25).
```
$ osatura reduced-motion small.json calm.json --factor 0.25
wrote calm.json
```

### `golden`

**Sinopse:**
```
osatura golden write <project.json> <golden.json> [--scale N]
osatura golden check <project.json> <golden.json> [--tol T]
```
`write --scale N` aceita N em (0, 1]: um valor maior é **recusado** (`golden write: --scale 2 is above 1 ...`, nada é gravado; antes era limitado a 1 silenciosamente). `check` renderiza novamente na escala armazenada no arquivo golden e não tem a opção `--scale`: `golden check ... --scale N` é recusado como opção desconhecida.


Um teste de regressão leve: `write` registra um hash e uma miniatura de 16x16 de cada quadro em `golden.json`; `check`
renderiza novamente e compara. **Exato por padrão** (`--tol 0`): qualquer diferença no hash de pixels falha. `--tol T` (T > 0) afrouxa
a verificação: um hash alterado ainda passa quando a distância entre miniaturas é no máximo T. Um `--tol` negativo ou não finito é um erro.
Para golden frames baseados em imagem com uma métrica perceptual, use `ci`.
```
$ osatura golden write small.json g.json --scale 0.25
wrote g.json (8 frames)
$ osatura golden check small.json g.json
all 8 frames match exactly
$ osatura golden check small.json g.json --tol 0.01
all 8 frames match (within thumbnail tolerance 0.01)
```
Os quadros que diferem são listados (mudança de hash, distância entre miniaturas, número das 256 células da miniatura que diferem, maior diferença
de canal em 255), e o código de saída é 1:
```
$ osatura golden check changed.json g.json
frame 1 differs: pixel hash changed, thumbnail distance 0.003, 12 of 256 thumbnail cells differ, max channel difference 31/255
2 frame(s) differ (exact comparison; --tol T accepts small differences)       [stderr, exit 1]
```
Um quadro presente em apenas um dos lados é informado como `one side only`. **Limites:** o arquivo golden guarda apenas o hash e a
miniatura, de modo que os números informados são em nível de miniatura, não de pixel. Uma mudança invisível na renderização (por exemplo,
uma cor de fundo totalmente coberta por uma camada opaca, como o céu da demonstração) legitimamente deixa o hash inalterado.

### `ci` (init, run, bless, diff)

**Sinopse:**
```
osatura ci init <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run  [manifest] [--update] [--bless] [--prune] [--only a,b] [--junit f.xml] [--html f.html] [--json f.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest]
osatura ci diff a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--ignore x,y,w,h;...] [--aa N] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```
**CI de animação**: golden frames comparados com uma métrica perceptual (SSIM de luma, CIEDE2000), mais verificações estruturais e
orçamentos. O formato do manifesto e o fluxo de trabalho estão descritos em [CI.md](CI.md). `exit 0` = passou, `1` = uma verificação falhou,
`2` = erro (manifesto ilegível, ...).
```
$ osatura ci init small.json --out ci.json
wrote ci.json with 8 checks:
  validate     validate
  render       frames
  render       frames-float
  determinism  determinism
  fuzz         pose-fuzz
  limits       limits
  export-size  gif-size
$ osatura ci run ci.json --junit j.xml --html r.html --json ci_r.json
[PASS   ] validate     validate           project compiles; 8 lint finding(s), none at 'error' or above
[PASS   ] render       frames             6 frame(s) match
[PASS   ] determinism  determinism        6 render(s) repeated byte-identically
[INFO   ] fuzz         pose-fuzz          98 of 100 pose samples fail (98.00%); not a gate: non-gating (gate = false)
[PASS   ] export-size  gif-size           gif export is 82881 bytes (budget 124322)
[INFO   ] perf         render-speed       median 429.67 ms/frame, budget 1633 ms (+50% tolerance = 2449.50 ms)
8 checks: 6 passed, 0 failed, 0 errors, 2 info, 0 updated, 0 blessed (14.7 s)
$ osatura ci diff a.png b.png --metric ssim --out heat.png
size        480x360
SSIM        0.90536   (luma)   channels min 0.90300   (R, G, B)
de2000      max 68.674   mean 1.9434
pixels      11124 differ (6.4375%), 31 forgiven as anti-aliasing, 0 ignored
regions     118
images differ: luma SSIM 0.90536 < 0.99; ...                                [stderr, exit 1]
$ osatura ci run nonexist.json
cannot read manifest nonexist.json: No such file or directory               [exit 2]
```
`init` grava o manifesto e, a menos que se use `--no-goldens`, os PNGs golden na pasta `goldens/` ao lado dele. `run
--update` reescreve os golden frames ausentes ou alterados, `--bless` aceita as novas renderizações como golden frames, `--prune` remove
os arquivos golden que nenhuma verificação referencia mais, `--only a,b` executa as verificações nomeadas. `bless` é `run --bless`. `diff` compara dois PNGs: `--tol` é o limiar
da métrica, `--ignore` mascara retângulos (`x,y,w,h;...`), `--aa N` perdoa diferenças de antialiasing, e `--out`,
`--side`, `--overlay`, `--flicker` gravam um mapa de calor, uma imagem lado a lado, uma sobreposição e um GIF de flicker.
**Pegadinha:** os relatórios (`--junit`, `--html`, `--json`) e os arquivos golden são gravados com o gravador atômico que recusa links simbólicos (criando
as pastas pai); um caminho de relatório que seja link simbólico é recusado depois de as verificações terem sido executadas (as verificações são lentas, portanto passe um caminho real). Um build de depuração é lento: as 8 verificações acima levaram 15 s em um projeto de 8 quadros.

### `fuzz`

**Sinopse:** `osatura fuzz <project.json> [--samples N] [--seed S] [--strategy random|lhs|halton|sweep|corners|pca] [--frame F] [--report out.json] [--heatmap out.svg] [--minimize] [--gate] [--max-fail-rate R] [--props angle,scale,offx,offy] [--only BONES] [--margin DEG] [--db posedb.json] [--ignore KINDS] [--no-deformers]`

**Pose fuzzing**: amostra poses dentro dos limites dos ossos e informa onde o rig quebra (dobras, esticamento, picos,
autointerseção, IK inalcançável, violações de limite, NaN, fora da tela). `--strategy` escolhe o amostrador (`pca` amostra ao longo dos eixos PCA das poses-chave do próprio projeto, ou do banco de poses
dado com `--db`), `--minimize` reduz uma pose com falha a um reprodutor mínimo, `--heatmap` grava um mapa de estresse,
`--ignore` ignora tipos de falha, `--only` restringe a ossos. Sem um gate, `fuzz` é um **relatório**: imprime o resumo e termina com exit 0 (`(report only: exit 0; add --gate, ...)`). `--gate` falha (exit 1) quando qualquer amostra falha; `--max-fail-rate R` (uma **fração** 0..1) implica gate nessa taxa. (A verificação `fuzz` do próprio `ci` tem sua própria configuração `gate` e não é afetada.)
```
$ osatura fuzz small.json --samples 30 --seed 1 --strategy lhs --report fz.json --heatmap hm.svg --max-fail-rate 1.0
wrote fz.json
wrote hm.svg
fuzz 'small': 30 samples (lhs, seed 1, frame 0), 25 parameters, 30 failures (100.0%), max severity 54.62, baseline 0.00, 0.40 ms/sample
  ik_unreachable: 27
  off_canvas: 1
  stretch: 2
$ osatura fuzz small.json --samples 10 --max-fail-rate 0.5
fuzz: failure rate 100.00% exceeds --max-fail-rate 50.00%                   [stderr, exit 1]
```
O alvo de IK da demonstração é deliberadamente alcançável apenas em uma faixa pequena, de modo que quase toda pose aleatória é "inalcançável":
trate a taxa como um número de regressão, não como aprovado/reprovado neste rig. O relatório e o mapa de calor são gravados atomicamente e um caminho que seja link simbólico é recusado (a menos que se use `--follow-symlinks`); a pasta pai deve existir.

### `fuzz-grid`

**Sinopse:** `osatura fuzz-grid <project.json> --x PARAM --y PARAM [--res N] out.svg|out.png|out.json`

Um mapa de severidade de dois parâmetros (`bone:NAME:PROP`, por exemplo `bone:arm_up:angle`) em uma grade `N x N` (padrão 32).
O tipo de saída segue a extensão.
```
$ osatura fuzz-grid small.json --x bone:arm_up:angle --y bone:arm_low:angle --res 8 grid.svg
wrote grid.svg (8x8 cells, 59.4% failing; x = bone:arm_up:angle, y = bone:arm_low:angle)
```
Mesma ressalva de gravação simples que em `fuzz`.

## 15. Controle de versão (`vcs`)

**Sinopse:**
```
osatura vcs init <project.json> [--author A]
osatura vcs commit [-m MSG | --message MSG] [--author A] [--repo DIR]
osatura vcs log [REV] [--limit N] [--json] [--repo DIR]
osatura vcs status [--json] [--repo DIR]
osatura vcs diff [A [B]] [--json] [--repo DIR]
osatura vcs show [REV] [--json] [--repo DIR]
osatura vcs branch [NAME [AT] | -d NAME] [--repo DIR]
osatura vcs tag [NAME [AT]] [--repo DIR]
osatura vcs checkout REV [--force] [--repo DIR]
osatura vcs merge REV [--no-ff] [-m MSG | --message MSG] [--author A] [--abort] [--repo DIR]
osatura vcs cherry-pick|revert REV [--author A] [--repo DIR]
osatura vcs resolve PATH [--ours] [--theirs] [--value V] [--repo DIR]
osatura vcs fsck|gc [--repo DIR]
```
As opções são por subcomando: `--json` apenas em `log`, `status`, `diff` e `show`; `--force` apenas em `checkout`; `--ours`, `--theirs`, `--value` (exatamente uma) apenas em `resolve`.

Um pequeno histórico endereçado por conteúdo para **um único arquivo de projeto** com **mesclagem semântica**: as mudanças são comparadas como
caminhos do documento (JSON pointers), não como linhas de texto, de modo que duas pessoas que editam chaves diferentes nunca entram em conflito. `init` cria
uma pasta `.osatura-vcs/` ao lado do projeto; o branch padrão é `main`. As revisões (`REV`) são nomes de branch, nomes de tag
ou prefixos de id de commit.

```
$ osatura vcs init p.json
initialised ./.osatura-vcs for p.json (no commits yet; run: osatura vcs commit -m "...")
$ osatura vcs commit -m first --author alexio
[5f80a63179] first
$ (edit p.json)
$ osatura vcs status
on branch main
HEAD 5f80a63179
uncommitted changes:
  project: background changed "#f6f1e7" -> "#ffffff"
$ osatura vcs commit -m "white bg"
[29c09d9368] white bg
$ osatura vcs branch alt          # branch alt at 29c09d9368
$ osatura vcs checkout alt        # checked out alt (29c09d9368)
$ osatura vcs log --limit 5
4e21f15940  unknown  30 fps
29c09d9368  unknown  white bg
5f80a63179  alexio  first
$ osatura vcs merge alt           # fast-forward or a merge commit; --no-ff forces a merge commit
merge committed as f08db3f77f
$ osatura vcs tag v1 ; osatura vcs tag
tag v1 at 4e21f15940
v1  4e21f15940
$ osatura vcs fsck
ok: 50 objects, history and refs are consistent
$ osatura vcs gc
gc: kept 50 objects, removed 0 unreachable
```

**Conflitos.** Quando os dois lados alteraram o mesmo valor, `merge` para, mantém o lado *nosso* no arquivo de projeto, lista os
conflitos e termina com exit 1:
```
$ osatura vcs merge main
merge stopped with 1 conflict(s); the project file holds our side for each:
  CONFLICT /background
      project: background: ours "#222222" vs theirs "#333333" (base "#000000")
resolve each with: osatura vcs resolve <path> --ours|--theirs|--value <json>, then: osatura vcs commit   (or: osatura vcs merge --abort)
$ osatura vcs resolve background --theirs
no conflict at 'background' (see 'vcs status')                              [exit 1]   (the path needs its leading slash)
$ osatura vcs resolve /background --theirs
resolved /background; 0 conflict(s) left - run: osatura vcs commit
$ osatura vcs commit -m merged
[bd405f446a] merged
$ osatura vcs merge --abort       # alternatively: merge aborted; project restored to HEAD
```
Enquanto um merge, cherry-pick ou revert não estiver resolvido, `checkout`, `revert` e `commit` são recusados com
`a merge / cherry-pick / revert is in progress: resolve the conflicts and commit, or run 'vcs merge --abort'`.
Outros fatos: `diff` sem argumento compara o arquivo de trabalho com HEAD, `diff A B` duas revisões (`--json` imprime uma lista
de `{kind,path,description}`); `show` imprime um commit e sua lista de mudanças; `branch` sem argumento lista os branches (`*` marca
o atual); `branch -d NAME` recusa excluir o branch em checkout; `checkout --force` descarta as mudanças
não confirmadas; `revert REV` e `cherry-pick REV` criam um novo commit (ou param com conflitos como `merge`); `--repo DIR`
seleciona uma pasta de repositório diferente da atual; `--author` assume `unknown` por padrão; `log --json` imprime os objetos
de commit. **Limites:** um arquivo por repositório, apenas local (sem remotos, sem push/pull), sem rastreamento de renomeação além dos caminhos JSON;
os recursos binários referenciados pelo projeto não são versionados.

## 16. Studio, colaboração, servidor de agente MCP e registro de UI

### `studio`

**Sinopse:** `osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES] [--public-origin URL] [--source-url URL] [--collab-token T (16+ characters)] [--collab-secret-file F [--collab-revoke NAMES] [--collab-host NAMES]] [--collab-rate N] [--collab-log LOG [--collab-nosync]]`

Inicia o **editor no navegador** (desfazer/refazer, timeline, painéis de live e de rosto, VCS e operações no estilo de agente) como um servidor web
local e bloqueia até Ctrl-C. Padrões: `--port 8080`, `--bind 127.0.0.1`. O projeto é validado na inicialização; os salvamentos vão
para o mesmo arquivo (`POST /api/save`).
```
$ osatura studio demo.json --port 18931 --ui-config ui.json
AGPL-3.0 source offer: http://127.0.0.1:18931/source
OSATURA studio on http://127.0.0.1:18931  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in ui.json
```
Opções: `--ui-config F` guarda o layout e os atalhos (padrão `~/.config/osatura/ui.json`, no Windows `%APPDATA%\OSATURA\ui.json`); `--source-url URL` define o
link do código-fonte publicado exibido em `/source` e por `--version` (AGPL-3.0 seção 13; os binários de release oficiais usam por padrão o commit a partir do qual foram compilados); `--allow-host NAMES` e `--public-origin URL` aceitam
requisições sob um nome público atrás de um proxy reverso TLS; `--collab-token T` (pelo menos 16 caracteres, por exemplo
`openssl rand -hex 16`) habilita a sessão multiusuário e é **obrigatório para qualquer `--bind` que não seja loopback**; `--collab-nosync` só é válido junto com `--collab-log` (caso contrário, é um erro); `--collab-log LOG`
persiste a sessão compartilhada e a retoma após uma reinicialização (`--collab-nosync` pula a sincronização na inicialização).

**Opções de proteção** (wave 7; detalhes e o modelo de ameaças em [SECURITY.md](SECURITY.md)): `--collab-secret-file F` substitui
o único bearer compartilhado por **tokens por participante**: F contém um segredo aleatório (primeira linha, pelo menos 16 caracteres, modo 600 no
Unix, caso contrário é recusado) e cada pessoa recebe um token de `osatura collab-token NAME`; um token fica vinculado a um nome
(a entrada usa esse nome, ninguém pode entrar como outra pessoa), expira e é verificado recalculando o seu HMAC-SHA-256, de modo que o
servidor não mantém tabela alguma. `--collab-revoke NAMES` (separados por vírgula, requer `--collab-secret-file`) recusa os tokens desses
nomes antes de expirarem (um servidor em execução também pode revogar com `POST /api/collab/revoke {"names":["ana"]}`, veja abaixo). `--collab-host NAMES`
(separados por vírgula, requer `--collab-secret-file`) indica os participantes cujos tokens podem **salvar o projeto, compactar o log da sessão e revogar
tokens**; todos os demais com um token válido ainda podem editar, mas recebem `403` nessas três requisições. Sem `--collab-host`, todo participante pode
salvar e compactar (como antes) e ninguém além do `--collab-token` compartilhado ou de uma sessão local sem autenticação pode revogar por HTTP; o estúdio informa qual é o caso
na inicialização. `--collab-token` e `--collab-secret-file` podem ser combinados (o token compartilhado passa então a atuar como
chave de administrador); qualquer um dos dois satisfaz a regra de que um bind que não seja loopback exige uma credencial. `--collab-rate N` limita cada endereço
de cliente a N requisições por segundo (rajadas de 2N; resposta `429` com `Retry-After`); o padrão é 100 quando uma credencial é
exigida e nenhum limite caso contrário; 0 o desativa. Um bind que não seja loopback imprime avisos (HTTP simples; todas as interfaces para
`0.0.0.0` / `::`; um token compartilhado não tem nomes nem expiração).

**Regras de segurança** (verificadas): o servidor responde apenas a requisições cujo `Host` seja um de seus próprios nomes
(`curl -H 'Host: evil.example' .../api/state` retorna 421); `Origin` e `Sec-Fetch-Site` de outra página retornam 403; o corpo de um POST
deve ser `application/json` (`curl -X POST -d '{}'` retorna 415). Um token curto é recusado
(`--collab-token must be at least 16 characters`), e `--bind 0.0.0.0` sem token é recusado
(`refusing to listen on 0.0.0.0:PORT: the studio has no accounts and no TLS, ...`). O servidor não tem contas nem TLS: coloque
um proxy TLS na frente para qualquer uso que não seja a sua própria máquina; quando `--public-origin` é uma URL `https://`, o cookie de login recebe o atributo `Secure` para as requisições que chegam sob esse nome (ou com `Origin` https / `X-Forwarded-Proto: https`). A página serve um pequeno favicon SVG em `/favicon.ico`. A API HTTP (por exemplo `GET /api/state`, `GET /api/project`,
`POST /api/op`, `POST /api/save`, `GET /frame/N.svg`) existe para a própria página e não é uma API pública estável.

### `collab-token`

**Sinopse:** `osatura collab-token <name> [--secret-file F] [--ttl 28800]`

Imprime um token de colaboração por participante para um studio iniciado com `--collab-secret-file F`: `ap1.<name>.<expires>.<mac>`,
em que `mac` é o HMAC-SHA-256 hexadecimal do nome e do prazo sob o segredo. `<name>` tem de 1 a 32 caracteres entre letras,
dígitos, `_` e `-`. `--ttl` é a vida em segundos (padrão 28800 = 8 horas, no máximo 2592000 = 30 dias). O segredo é lido do
arquivo `F` (primeira linha) ou, sem `--secret-file`, da variável de ambiente `OSATURA_COLLAB_SECRET`; nunca é
aceito como argumento, onde outros usuários poderiam vê-lo. O token vai para stdout, o prazo para stderr.

```
$ openssl rand -hex 32 > collab.secret && chmod 600 collab.secret
$ osatura studio demo.json --collab-secret-file collab.secret
$ osatura collab-token ana --secret-file collab.secret --ttl 3600
ap1.ana.1790000000.3f0c...
token for 'ana', valid until Unix time 1790000000 (in 3600 s); give it to that person only; ...
$ osatura collab-token ana --secret-file collab.secret --ttl 0
--ttl needs a whole number of seconds (1 to 2592000)
```
A pessoa cola o token na página de login do studio (ele é então mantido em um cookie HttpOnly que expira junto com o token) ou o envia como
`X-Collab-Token`. Um token expirado recebe como resposta `401` com o prazo na mensagem; o nome do participante é obtido do
token. **Relacionados:** `studio`.

### `collab-sim`

**Sinopse:** `osatura collab-sim <project.json> [--clients N] [--ops M] [--seed S] [--undo-pct P] [--replays K] [--require-valid] [--raw] [--json]`

Um arnês de teste para o motor de colaboração: N réplicas editam o projeto simultaneamente com ordem de entrega aleatória e
duplicatas, e em seguida verifica-se que todas contêm o mesmo documento (exit 1 se não). As edições padrão são edições
válidas de projeto (números, valores de chaves, visibilidade, camadas próprias); `--raw` usa edições JSON arbitrárias (normalmente não é um projeto válido);
`--require-valid` faz com que um resultado que não compila também termine com exit 1.
```
$ osatura collab-sim small.json --clients 3 --ops 20 --seed 2 --json
{"buffered":12,"clients":3,"clocks_identical":true,"converged":true,"converged_ok":true,"deliveries":34,"duplicates_dropped":0,"edits":20,"envelopes":17,...}
```
`converged_ok` é apenas a convergência; `ok` também exige que o documento mesclado compile.

### `mcp`

**Sinopse:** `osatura mcp <project.json>`

Um **servidor de agente**: JSON-RPC 2.0 (o Model Context Protocol) sobre stdin/stdout, uma mensagem JSON por linha. Carrega o
projeto e expõe as ferramentas `get_project`, `stats`, `lint`, `coach`, `safety`, `apply_ops`, `set_key`, `undo`, `redo`,
`render_frame` e `save`. Nada é gravado até que `save` seja chamado. Stdout carrega apenas mensagens do protocolo.
```
$ printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lint","arguments":{}}}' | osatura mcp demo.json
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"protocolVersion":"2024-11-05","serverInfo":{"name":"osatura","version":"0.1.0"}}}
{"id":2,"jsonrpc":"2.0","result":{"tools":[{"description":"Return the project JSON, or the value at a JSON pointer ...","name":"get_project",...}]}}
{"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"[]","type":"text"}],"isError":false}}
```
O servidor termina (código 0) quando stdin é fechado.

### `ui`

**Sinopse:**
```
osatura ui commands|shortcuts [--ui-config F] [--json]
osatura ui layout presets|show <name>|validate <file.json>
osatura ui search <query...> [--ui-config F]
```
O **registro de comandos** do studio: `commands` lista cada comando com sua categoria, título e atalhos de teclado;
`shortcuts` lista os atalhos efetivos (os padrões mais os de `--ui-config`) e quaisquer conflitos; `layout presets` lista os
presets de layout de docks, `layout show NAME` imprime um como JSON e `layout validate FILE` verifica um arquivo de layout;
`search` classifica os comandos como a paleta de comandos faz.
```
$ osatura ui commands
Edit
  edit.undo                    Undo                               global:Ctrl+Z
  edit.redo                    Redo                               global:Ctrl+Shift+Z  global:Ctrl+Y
$ osatura ui layout presets
animate  V0.62(H0.74([*stage] | [*layers selection bones live face json]) | [*timeline])  [valid]
rig      H0.66(V0.84([*stage] | [*timeline (folded)]) | V0.50([*layers bones] | [*selection json]))  hidden: live,face  [valid]
compact  V0.62([*stage] | [*timeline layers selection bones live face json])  [valid]
live     H0.58(V0.70([*stage] | [*timeline]) | [*live face bones selection])  hidden: layers,json  [valid]
$ osatura ui layout validate demo.json
demo.json: unknown field `format`, expected one of `version`, `root`, `floating`, `hidden` at line 2 column 10   [exit 1]
```

## 17. Banco de poses, sugestões e outros auxiliares

Estes comandos constroem um **banco de poses** (rotações de ossos nos quadros-chave de um projeto e de cada take de uma biblioteca)
e usam a sua estrutura aprendida (eixos PCA, rede pequena opcional) para sugerir poses. É um auxílio estatístico treinado com
*as suas* poses, não um modelo geral. Seus arquivos de modelo e de relatório são gravados com uma gravação simples.

### `pose-db` (build, info, train-mlp)

**Sinopse:**
```
osatura pose-db build <out.json> [--project P.json] [--library takes.json] [--step N]
osatura pose-db info  <db.json>
osatura pose-db train-mlp <db.json> <out.json|out.bin> [--hidden 64,64] [--epochs N] [--lr R] [--seed S]
```
```
$ osatura pose-db build db.json --project demo.json
wrote db.json: 5 poses of 3 bones in 1 sequence(s)
$ osatura pose-db info db.json
5 poses, 3 bones: arm_up, arm_low, hand
  sequence 0: project (project, 5 poses)
PCA: 2 axes keep 99.2% of the variance (total 950.0 deg^2)
  axis 1: std 27.34 deg, cumulative 78.7%
  axis 2: std 13.95 deg, cumulative 99.2%
typical nearest-pose spacing 14.43 deg, typical step between consecutive poses 35.82 deg
$ osatura pose-db train-mlp db.json mlp.json --hidden 8 --epochs 20 --seed 1
wrote mlp.json: 6 -> [8] -> 3 (epochs 20, loss 1.0189 -> 0.4921)
trained on the library itself (no held-out data here); run pose-bench for held-out accuracy
```
`--project` adiciona as poses nos quadros-chave do projeto, `--library` os takes de uma biblioteca de takes (`take-record`, seção 6),
`--step N` amostra adicionalmente a cada N quadros. A rede pequena (SGD simples) é opcional; use-a com `--model`, abaixo.

### `pose-suggest` (between, complete, next)

**Sinopse:**
```
osatura pose-suggest between  <project.json> <db.json|-> --a F1 --b F2 [--t 0.5] [--snap 0.5] [--top 5] [--apply RANK --out out.json] [--json]
osatura pose-suggest complete <project.json> <db.json|-> --frame F [--fix bone=angle,...] [--reach bone=x,y,...] [--free b1,b2] [--prior 1] [--model m.json] [--apply RANK --out out.json]
osatura pose-suggest next     <project.json> <db.json|-> --frame F [--k 8]
```
`-` no lugar de um banco aprende a partir do próprio projeto. `between` propõe uma pose intermediária de dois quadros que permanece
na variedade aprendida (candidatos `linear`, `pca`, `manifold`); `complete` encontra a pose completa mais plausível dados
ângulos fixos (`--fix`), alvos de ponta (`--reach`) e ossos livres; `next` mostra o que a biblioteca fez depois de poses como a do
quadro. `--apply RANK` (com `--out`) grava o candidato escolhido como chaves.
```
$ osatura pose-suggest between demo.json db.json --a 0 --b 24 --t 0.5 --top 3
between: 5 library poses, 2 PCA axes (99% of the variance)
 1. linear      score 1.00  plausibility 1.00  key at frame 12.0
      linear blend of the angles
      arm_up=0.0 arm_low=-7.5 hand=-10.0
 3. manifold    score 1.00  plausibility 1.00  key at frame 12.0
      PCA blend pulled toward the library
$ osatura pose-suggest complete demo.json db.json --frame 12 --fix arm_up=-40
 1. pca-ls      score 0.96  plausibility 1.00  constraint error 0.18  key at frame 12.0
      regularised least squares in PCA space (start 1)
      arm_up=-39.8 arm_low=24.9 hand=11.2
 2. library-nn  score 0.25  plausibility 1.00  constraint error 15.00  key at frame 12.0
(scores rank candidates against this library; they are heuristics, not probabilities)
$ osatura pose-suggest next demo.json db.json --frame 12 --k 3
 1. successor   score 0.60  plausibility 1.00  key at frame 24.0
      what followed pose 1 of 'project'
```

### `pose-bench`

**Sinopse:** `osatura pose-bench`

Imprime a precisão em dados reservados dos métodos de sugestão em uma **biblioteca sintética semelhante a ciclos de marcha** (24 ciclos de treinamento, 6
reservados; a completação vê 3 de 10 ossos; também treina a rede pequena). Quanto menor, melhor. Não lê nenhum projeto. É
**lento em um build de depuração** (minutos); números de um build de release:
```
completion  mean pose                                   20.96 deg RMS
completion  nearest library pose                         1.77 deg RMS
completion  PCA least squares                            0.51 deg RMS
completion  MLP                                          1.64 deg RMS
between     linear blend                                 1.83 deg RMS
next        Markov mean                                  0.72 deg RMS
```

### `pose-match`

**Sinopse:** `osatura pose-match <project.json> <params.json> <goals.json> <out.json> [--generations N --seed N]`

Busca (um otimizador evolutivo, padrão de 80 gerações, seed 1, faixa válida 1..100000) valores para um conjunto de
parâmetros que coloquem as pontas dos ossos em posições-alvo no espaço do palco. `params.json` é uma lista de
`{"target","prop","min","max","start"}`; `goals.json` é uma lista de `{"bone","at":[x,y],"weight"}`. O resultado é acrescentado como
canais; a perda é a soma ponderada dos erros quadráticos das pontas em px².
```
$ echo '[{"target":"arm_up","prop":"bone_angle","min":-60,"max":40,"start":0},{"target":"arm_low","prop":"bone_angle","min":-60,"max":60,"start":0}]' > params.json
$ echo '[{"bone":"hand","at":[260,200],"weight":1.0}]' > goals.json
$ osatura pose-match demo.json params.json goals.json out.json --generations 30 --seed 3
wrote out.json (loss 3756.156 px²)
```
**Pegadinha:** a perda no exemplo é grande porque as faixas não conseguem alcançar o alvo: verifique o número, não apenas o
código de saída (um resultado é gravado mesmo quando a correspondência é ruim).
## Divergências conhecidas entre documentação e comportamento

Quando o próprio texto de uso do programa (impresso por `osatura` sem argumentos) ou um documento vizinho diz
uma coisa e o programa faz outra, as seções acima descrevem o que o programa realmente faz. As diferenças encontradas
ao verificar cada exemplo, na ordem de probabilidade de causarem problemas:

1. **Corrigido - flags desconhecidas.** Todo comando com uma linha de uso agora rejeita `--words` não documentadas (com uma dica "did you mean"),
   incluindo `check`, `stats`, `lint`, `coach`, `contracts` e `machine list`. `--json`, `--quiet` e `--allow-empty` são
   por comando e recusadas nos demais (uma mudança incompatível para scripts que as passavam em todos os lugares). Desde a wave 5 a verificação é **por subcomando** (`ci diff ... --quiet`, `trail <bone> ... --json`, `ui layout presets --json`, `golden write --tol` são recusados) e também abrange as opções com valor. Testes: `osatura-cli/tests/doc_mismatch.rs` (matriz de aceitação) e `options_are_validated_per_sub_command` em `main.rs`.
2. **Corrigido - saídas com `fs::write` simples.** `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, arquivos de pose, snapshots,
   `vcs` e o log de colaboração usam o gravador atômico que recusa links simbólicos; as saídas do ffmpeg são verificadas previamente. Um teste orientado por tabela
   cobre cerca de 24 comandos. Não coberto de ponta a ponta: `face-train`/`face-eval` (teste unitário apenas no gravador deles).
3. **Corrigido - `golden check`** é exato por padrão; `--tol` o afrouxa; imprime a contagem de diferenças e a maior diferença.
   A afirmação de que "uma mudança na cor de fundo passa" era um artefato da demonstração (a camada do céu cobre o fundo), não um bug.
4. **Corrigido - código de saída de `fuzz`.** O padrão é apenas relatório (exit 0); `--gate` ou `--max-fail-rate` ativam o gate. Veja `fuzz`.
5. **Corrigido - `dof-export`** grava `.mp4` e `.webm` (via ffmpeg, com `--codec`, sem áudio); o texto de uso agora os lista. Um nome
   com qualquer outra extensão é recusado e nada é gravado.
6. **Corrigido - `export` `.png`** grava `<name>.atlas.json`, nunca `<name>.json`; o erro de formato desconhecido lista todas as extensões.
7. **Padrões que o texto de uso não informa:** `studio` escuta em `127.0.0.1:8080`; `fuzz-grid --res` é 32; `export-fit`
   amostra 8 quadros, pesquisa com `descent` e para após 60 s (`--max-seconds 0` = ilimitado); `reduced-motion --factor`
   é 0,25; `size-report --tol` é 0,05; `pose-match` executa 80 gerações com seed 1; `lod --out-dir` é a pasta
   atual.
8. **`machine remove` pode recusar** (`binding 'goBind' still drives machine 'm'`); o texto de uso não menciona a
   verificação de dependência. `machine` e `state-bake` leem dois formatos de máquina diferentes (`layers`/`inputs` versus
   `initial`/`states`); a linha de uso de `state-bake` não tem descrição e não informa o formato.
9. **Corrigido - `stats`** não aceita mais `--json` (a saída é sempre JSON; a linha de uso diz isso).
10. **`vcs`:** o branch padrão é `main` (`checkout master` falha com `unknown revision 'master'`); `vcs resolve`
    recebe o caminho do conflito *com* a barra inicial, exatamente como `merge` o imprime (`/background`), `background` sozinho resulta em
    `no conflict at 'background'`; o texto de uso escreve "see: osatura vcs", mas `osatura vcs` sem argumentos imprime um
    uso de uma linha e termina com exit 1.
11. **Linhas de uso sem descrição.** `state-bake`, `pose-match`, `pose-bench`, `follow`, `keys`, `event`, `marker`,
    `idle`, `mouth`, `beats`, `export`, `frames`, `onion` e outros têm uma sinopse mas nenhuma explicação, ou uma descrição que
    fica em uma linha de continuação; o comportamento deles neste documento vem de tê-los executado.
12. **Nomenclatura das propriedades.** As mensagens e as saídas de `list` imprimem as propriedades em CamelCase (`LayerOpacity`, `BoneAngle`),
    mas toda opção, arquivo de mapeamento e arquivo auxiliar recebe os nomes em snake_case (`layer_opacity`, `bone_angle`); a forma CamelCase
    é rejeitada (`bind add ... sun:LayerOpacity` falha com `unknown property 'LayerOpacity'`).
13. **`cloth`:** o texto de uso fala em "per-vertex deformation"; o resultado incorporado é armazenado como canais `lattice_dx` / `lattice_dy`
    nos pontos da malha do tecido (para a grade padrão de 7x7, os 42 pontos não fixados), veja a seção 7.
14. **`render-gpu`** existe apenas em builds feitos com `--features gpu`; não pôde ser executado no build de teste, de modo que suas opções
    são tomadas do texto de uso e não são verificadas aqui.
15. **Lento em builds de depuração.** `pose-bench`, `face-train`, `ci run` e `export-fit` levam minutos em um build não otimizado.
    Os tempos deste documento vêm do binário de depuração e são muito menores em um build de release.
16. **Corrigido (wave 5) - `golden write --scale`** acima de 1 era limitado a 1 silenciosamente; agora é recusado (a escala está em (0, 1]).
    `golden check` ignorava um `--scale` e usava o armazenado; agora recusa a opção.
17. **Corrigido (wave 5) - sinopses.** Toda opção do texto de uso de um comando está na sua sinopse aqui e vice-versa (teste
    `cli_reference_synopses_match_usage`), e toda linha de uso sem alternativas é repetida aqui palavra por palavra (teste
    `cli_reference_repeats_usage_lines_verbatim`). `render-gpu --features gpu` e `layer-normal ... relight --layer-normals`
    nas descrições de uso nomeiam um recurso de build e a opção de outro comando, não opções desses comandos.
18. **Outros documentos estão mais antigos que o programa em dois pontos** (veja [EXPORT_FORMATS.md](EXPORT_FORMATS.md) seção 17):
    `docs/LIMITATIONS.md` L-29 diz que a importação de Lottie não aceita imagens, efeitos, pré-composições nem mattes (o importador agora os trata,
    com os avisos listados na seção 9), e L-32 lista OCA entre os formatos de importação, embora a CLI só tenha
    `export-oca`.
## Índice alfabético de comandos

Cada comando com um resumo de uma linha e a seção que o documenta. Os subcomandos (`machine add`, `vcs commit`, ...) estão na seção do comando pai.

| Comando | Resumo | Seção |
|---|---|---|
| `--version` | imprime a versão, a licença, a oferta de código-fonte e os links de suporte | [1](#1-convenções-comuns-a-todos-os-comandos) |
| `align` | alinha camadas entre si ou ao palco | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `audio-react` | a sonoridade controla uma propriedade | [8](#8-áudio-lip-sync-e-sons) |
| `autorig` | IK, respiração e piscadas configurados a partir de nomes de ossos e camadas | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `autotrace` | vetorização local de bitmap (faixas de cor, ajuste de Bezier) | [9](#9-importação) |
| `autotrace-bench` | tabela de fidelidade do vetorizador em formas sintéticas | [9](#9-importação) |
| `beats` | detecta saltos de sonoridade e grava marcadores ou eventos | [8](#8-áudio-lip-sync-e-sons) |
| `bind` | vinculações de dados: add, list, remove | [11](#11-máquinas-de-estados-vinculações-de-dados-contêineres-e-regiões-de-rolagem) |
| `bind-eval` | avalia as vinculações para valores dados | [11](#11-máquinas-de-estados-vinculações-de-dados-contêineres-e-regiões-de-rolagem) |
| `bone-vis` | mostra um osso nas sobreposições apenas para um intervalo de quadros | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `cage` | adiciona um deformador de gaiola de coordenadas de valor médio | [7](#7-física-tecido-e-deformadores) |
| `cage-pose` | define chaves de deslocamentos dos vértices da gaiola em um quadro | [7](#7-física-tecido-e-deformadores) |
| `camera-seq` | anima a câmera por meio de shots salvos | [6](#6-animação-keys-timeline-e-geradores) |
| `chan-meta` | cor do canal e flags shy, solo e lock | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `check` | valida um projeto (exit 1 se inválido) | [3](#3-criação-e-inspeção-de-projetos) |
| `ci` | CI de animação: golden frames, diff perceptual, verificações estruturais | [14](#14-garantia-de-qualidade) |
| `clip` | coloca um take na pilha de clipes | [6](#6-animação-keys-timeline-e-geradores) |
| `cloth` | incorpora uma simulação determinística de tecido | [7](#7-física-tecido-e-deformadores) |
| `coach` | sugestões de timing/princípios | [14](#14-garantia-de-qualidade) |
| `collab-sim` | teste de convergência da colaboração | [16](#16-studio-colaboração-servidor-de-agente-mcp-e-registro-de-ui) |
| `collab-token` | emite um token de colaboração por participante (HMAC-SHA-256, expira) | [16](#16-studio-colaboração-servidor-de-agente-mcp-e-registro-de-ui) |
| `color-space` | escolhe a composição em sRGB ou em luz linear para o projeto | [12](#12-efeitos-profundidade-de-campo-onion-skin-trilhas-e-câmera) |
| `comp` | estados nomeados de camadas (visibilidade, opacidade, escolha de switch) | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `compare` | imagem antes / depois / diferença de dois projetos | [4](#4-renderização-e-exportação) |
| `container` | contêineres de layout: add, list, remove, solve | [11](#11-máquinas-de-estados-vinculações-de-dados-contêineres-e-regiões-de-rolagem) |
| `contracts` | contratos de movimento (testes unitários para animação) | [14](#14-garantia-de-qualidade) |
| `demo` | grava o projeto de demonstração de rig embutido | [3](#3-criação-e-inspeção-de-projetos) |
| `demo-fx` | grava a demonstração de líquido / booleana / partículas | [3](#3-criação-e-inspeção-de-projetos) |
| `diff` | diferenças semânticas entre dois projetos | [3](#3-criação-e-inspeção-de-projetos) |
| `distribute` | espaça três ou mais camadas uniformemente | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `dof` | renderização com profundidade de campo de um quadro | [12](#12-efeitos-profundidade-de-campo-onion-skin-trilhas-e-câmera) |
| `dof-export` | profundidade de campo animada (GIF/APNG/vídeo/pasta) | [12](#12-efeitos-profundidade-de-campo-onion-skin-trilhas-e-câmera) |
| `drive` | sinais CSV -> chaves por meio de arquivo de mapeamento | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `event` | adiciona um evento de runtime nomeado (com payload JSON opcional) | [6](#6-animação-keys-timeline-e-geradores) |
| `explain` | por que uma propriedade tem o seu valor: contribuições ordenadas | [3](#3-criação-e-inspeção-de-projetos) |
| `export` | exporta GIF, APNG, WebM/MP4, folha PNG, HTML, Lottie, .lottie, glTF/GLB, ... | [4](#4-renderização-e-exportação) |
| `export-fit` | exportação de melhor qualidade que cabe em um orçamento de bytes | [13](#13-nível-de-detalhe-relatórios-de-tamanho-e-export-fit) |
| `export-oca` | grava uma pasta Open Cel Animation | [4](#4-renderização-e-exportação) |
| `eye-clip` | mascara camadas de íris/pupila pelo formato do olho | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `face-bake` | pontos de referência -> chaves | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `face-calibrate` | calibração do rosto neutro | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `face-detect` | caixas de rostos e 23 pontos de referência em uma imagem | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `face-eval` | relatório de precisão do detector em dados reservados | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `face-synth` | animação sintética de cabeça | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `face-track` | NDJSON de pontos de referência a partir de vídeo ou quadros | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `face-train` | treina o detector de rostos com rostos sintéticos | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `follow` | geradores de overlapping action / follow-through | [6](#6-animação-keys-timeline-e-geradores) |
| `foot-lock` | fixa um controlador durante o contato com o solo | [6](#6-animação-keys-timeline-e-geradores) |
| `frames` | renderiza todos os quadros em uma pasta (SVG, ou PNG com --png) | [4](#4-renderização-e-exportação) |
| `fuzz` | encontra poses em que o rig quebra | [14](#14-garantia-de-qualidade) |
| `fuzz-grid` | mapa de severidade de dois parâmetros | [14](#14-garantia-de-qualidade) |
| `fx` | renderiza um quadro com uma cadeia de pós-efeitos | [12](#12-efeitos-profundidade-de-campo-onion-skin-trilhas-e-câmera) |
| `glue` | cola vértices de malha entre si | [7](#7-física-tecido-e-deformadores) |
| `golden` | grava/verifica quadros de regressão | [14](#14-garantia-de-qualidade) |
| `idle` | gera um canal idle procedural | [6](#6-animação-keys-timeline-e-geradores) |
| `import-anim` | GIF/APNG animado -> camada quadro a quadro | [9](#9-importação) |
| `import-bvh` | captura de movimento BVH -> canais | [9](#9-importação) |
| `import-lottie` | subconjunto de Lottie -> projeto | [9](#9-importação) |
| `import-ora` | OpenRaster -> projeto | [9](#9-importação) |
| `import-seq` | sequência de PNGs -> camada quadro a quadro | [9](#9-importação) |
| `import-svg` | subconjunto de SVG -> camada vetorial | [9](#9-importação) |
| `keyedit` | edições de dope sheet como um único lote seguro para desfazer | [6](#6-animação-keys-timeline-e-geradores) |
| `keys` | operações de chaves em massa sobre todos os canais com chaves | [6](#6-animação-keys-timeline-e-geradores) |
| `layer-normal` | anexa um mapa de normais a uma camada | [12](#12-efeitos-profundidade-de-campo-onion-skin-trilhas-e-câmera) |
| `layer-pivot` | move o pivô de uma camada sem alterar a imagem | [7](#7-física-tecido-e-deformadores) |
| `layout-preview` | renderiza com o layout resolvido novamente para outro tamanho de tela | [11](#11-máquinas-de-estados-vinculações-de-dados-contêineres-e-regiões-de-rolagem) |
| `lint` | verificações estáticas de rig/animação (exit 1 em erros) | [14](#14-garantia-de-qualidade) |
| `lipsync` | controla uma camada switch de boca a partir da sonoridade do áudio (não baseado em fonemas) | [8](#8-áudio-lip-sync-e-sons) |
| `lipsync-text` | alinha uma transcrição ao áudio e deriva as formas da boca | [8](#8-áudio-lip-sync-e-sons) |
| `live-map` | entrada ao vivo gravada (MIDI, mouse, gamepad, teclas) -> chaves, takes, eventos | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `lod` | níveis de detalhe com limites de erro em pixels declarados | [13](#13-nível-de-detalhe-relatórios-de-tamanho-e-export-fit) |
| `lod-pick` | nível mais grosseiro que cabe em um orçamento de pixels em uma tela | [13](#13-nível-de-detalhe-relatórios-de-tamanho-e-export-fit) |
| `loop-solve` | faz canais com chaves entrarem em loop sem emendas | [6](#6-animação-keys-timeline-e-geradores) |
| `machine` | máquinas de estados de runtime: add, list, remove, validate | [11](#11-máquinas-de-estados-vinculações-de-dados-contêineres-e-regiões-de-rolagem) |
| `machine-sim` | reproduz entradas temporizadas: log de estados mais quadros SVG | [11](#11-máquinas-de-estados-vinculações-de-dados-contêineres-e-regiões-de-rolagem) |
| `magnet` | incorpora uma atração por proximidade | [6](#6-animação-keys-timeline-e-geradores) |
| `make-loop` | faz um ciclo entrar em loop sem emendas (opcionalmente no lugar) | [6](#6-animação-keys-timeline-e-geradores) |
| `marker` | adiciona um marcador ou intervalo à timeline | [6](#6-animação-keys-timeline-e-geradores) |
| `mcp` | servidor de agente (JSON-RPC sobre stdio) | [16](#16-studio-colaboração-servidor-de-agente-mcp-e-registro-de-ui) |
| `mesh-lock` | seguidores acompanham a superfície de uma malha hospedeira | [7](#7-física-tecido-e-deformadores) |
| `mesh-trace` | vetoriza a silhueta alfa de uma imagem em uma malha triangular com skin | [7](#7-física-tecido-e-deformadores) |
| `mirror` | espelha todos os canais com chaves esquerda<->direita | [6](#6-animação-keys-timeline-e-geradores) |
| `mocap-info` | hierarquia e extensão de um arquivo BVH | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `mocap-match` | correspondência entre articulações BVH e ossos do rig | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `mocap-retarget` | incorpora mocap como canais acrescentados | [10](#10-entrada-ao-vivo-midi-rastreamento-facial-e-captura-de-movimento) |
| `mouth` | importa pistas de boca (Papagayo/Rhubarb) | [8](#8-áudio-lip-sync-e-sons) |
| `onion` | PNG ou SVG de onion skin | [12](#12-efeitos-profundidade-de-campo-onion-skin-trilhas-e-câmera) |
| `parallax` | profundidade multiplano para camadas de nível superior | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `parent-switch` | incorpora um re-parenting animado sem salto | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `physics` | incorpora uma simulação de cadeia | [7](#7-física-tecido-e-deformadores) |
| `pick` | o que há sob um ponto da tela (o mais alto primeiro) | [3](#3-criação-e-inspeção-de-projetos) |
| `pose-bench` | precisão em dados reservados dos métodos de pose | [17](#17-banco-de-poses-sugestões-e-outros-auxiliares) |
| `pose-db` | banco de poses: build, info, train-mlp | [17](#17-banco-de-poses-sugestões-e-outros-auxiliares) |
| `pose-match` | ajusta parâmetros para que as pontas dos ossos alcancem alvos | [17](#17-banco-de-poses-sugestões-e-outros-auxiliares) |
| `pose-suggest` | sugestões de intermediária, completação e próxima pose | [17](#17-banco-de-poses-sugestões-e-outros-auxiliares) |
| `proxy-bench` | milissegundos por quadro em cada nível de proxy | [4](#4-renderização-e-exportação) |
| `reduced-motion` | grava uma cópia mais calma de um projeto | [14](#14-garantia-de-qualidade) |
| `reference` | marca uma camada como referência de decalque (nunca exportada) | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `render` | renderiza um quadro em PNG ou SVG (proxies de scrub, ossos, escala) | [4](#4-renderização-e-exportação) |
| `render-gpu` | renderização por GPU (apenas builds com wgpu) | [4](#4-renderização-e-exportação) |
| `render-hdr` | renderização em ponto flutuante para OpenEXR ou PNG de 16 bits | [4](#4-renderização-e-exportação) |
| `reparent` | altera o pai de um osso mantendo a pose de repouso | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `retarget` | copia a animação entre rigs por pares de cadeias | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `safety` | relatório de flash / flash vermelho / movimento de câmera | [14](#14-garantia-de-qualidade) |
| `script` | executa um script Rhai em sandbox que edita o projeto | [6](#6-animação-keys-timeline-e-geradores) |
| `scroll` | regiões de rolagem: add, list, remove, eval | [11](#11-máquinas-de-estados-vinculações-de-dados-contêineres-e-regiões-de-rolagem) |
| `select-box` | seleção por marquee | [3](#3-criação-e-inspeção-de-projetos) |
| `shot` | salva um shot de câmera | [6](#6-animação-keys-timeline-e-geradores) |
| `size-report` | onde estão os bytes de um projeto/exportação | [13](#13-nível-de-detalhe-relatórios-de-tamanho-e-export-fit) |
| `skin` | skins nomeadas: conjuntos de camadas, ossos exclusivos da skin, restrições, cadeias de IK | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `skin-diffuse` | pesos de skin por difusão de calor | [7](#7-física-tecido-e-deformadores) |
| `slider` | cria um osso controlador que mistura poses armazenadas | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `snapshot` | histórico numerado de revisões de um projeto em uma pasta (save, list, restore) | [3](#3-criação-e-inspeção-de-projetos) |
| `soft-scale` | escala proporcional de uma seleção | [7](#7-física-tecido-e-deformadores) |
| `sound` | trilhas de som do projeto: list, add, edit, remove, mix | [8](#8-áudio-lip-sync-e-sons) |
| `speech-motion` | acenos e levantamentos de sobrancelha aditivos temporizados nas palavras alinhadas | [8](#8-áudio-lip-sync-e-sons) |
| `spring-fit` | ajusta a frequência/amortecimento da mola a uma curva | [6](#6-animação-keys-timeline-e-geradores) |
| `state-bake` | incorpora uma máquina de estados baseada em quadros em chaves | [11](#11-máquinas-de-estados-vinculações-de-dados-contêineres-e-regiões-de-rolagem) |
| `stats` | contagens de ossos, camadas, canais, chaves (JSON) | [3](#3-criação-e-inspeção-de-projetos) |
| `stretch` | incorpora um seguidor que se estica para alcançar | [6](#6-animação-keys-timeline-e-geradores) |
| `studio` | editor no navegador (servidor web local) | [16](#16-studio-colaboração-servidor-de-agente-mcp-e-registro-de-ui) |
| `tag` | propõe nomes de papéis padrão para ossos e camadas | [3](#3-criação-e-inspeção-de-projetos) |
| `take-apply` | aplica um take da biblioteca com cross-fades | [6](#6-animação-keys-timeline-e-geradores) |
| `take-list` | lista os takes de uma biblioteca | [6](#6-animação-keys-timeline-e-geradores) |
| `take-record` | grava um take em uma biblioteca | [6](#6-animação-keys-timeline-e-geradores) |
| `timecode` | quadro <-> timecode SMPTE (com suporte a drop-frame) | [3](#3-criação-e-inspeção-de-projetos) |
| `timeline` | trilhas por canal com chaves, bloqueios e cores | [3](#3-criação-e-inspeção-de-projetos) |
| `trace` | vetoriza o canal alfa de um PNG em um caminho vetorial | [9](#9-importação) |
| `trail` | trilha de movimento de um osso, camada ou ponto | [12](#12-efeitos-profundidade-de-campo-onion-skin-trilhas-e-câmera) |
| `trim` | incorpora um trim path em uma camada vetorial | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `ui` | registro de comandos do studio, atalhos, layouts, busca | [16](#16-studio-colaboração-servidor-de-agente-mcp-e-registro-de-ui) |
| `vcs` | controle de versão com mesclagem semântica para um arquivo de projeto | [15](#15-controle-de-versão-vcs) |
| `view-swap` | controla uma camada switch a partir de uma propriedade de osso (função em degrau) | [5](#5-rigging-skins-ik-e-utilitários-de-layout) |
| `walk` | gera um ciclo de caminhada | [6](#6-animação-keys-timeline-e-geradores) |
| `warp-pivot` | gira/escala/move uma seleção de pontos de controle | [7](#7-física-tecido-e-deformadores) |
| `waveform` | picos mín./máx. para desenho | [8](#8-áudio-lip-sync-e-sons) |
