<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Formatos de exportação e importação do OSATURA

O que cada formato de arquivo que o OSATURA grava ou lê preserva, aproxima e descarta, qual comando o produz e como
verificar o resultado. Este texto é licenciado sob CC-BY-4.0. Versão documentada: **osatura 0.1.0**.

Cada linha de comando abaixo foi executada contra o projeto de demonstração embutido (`osatura demo demo.json`) ou contra uma cópia de 8 quadros dele
(`small.json`, o mesmo projeto com `"frames": 8`). Os tamanhos citados vêm desse projeto de 8 quadros e 640x480 e servem apenas
de orientação: eles variam com quadros, âncoras e camadas, não com algo fixo. Os detalhes dos comandos (todas as opções,
todos os erros) estão em [CLI_REFERENCE.md](CLI_REFERENCE.md); o formato de arquivo do próprio projeto está em
[PROJECT_FORMAT.md](PROJECT_FORMAT.md); o status de codecs e patentes está em [PATENT_NOTES.md](PATENT_NOTES.md) (não é aconselhamento jurídico).

## Conteúdo

1. [Qual formato para qual tarefa](#1-qual-formato-para-qual-tarefa)
2. [Formatos de saída em resumo](#2-formatos-de-saída-em-resumo)
3. [Imagens estáticas: SVG, PNG, PNG de 16 bits, OpenEXR](#3-imagens-estáticas)
4. [Animação raster: sprite sheet PNG, GIF, APNG, WebM, MP4](#4-animação-raster)
5. [Player HTML](#5-player-html)
6. [Lottie (.json) e dotLottie (.lottie)](#6-lottie-json-e-dotlottie-lottie)
7. [glTF binário (.glb)](#7-gltf-binário-glb)
8. [Pasta Open Cel Animation (OCA)](#8-pasta-open-cel-animation-oca)
9. [Importação: SVG, OpenRaster, Lottie, imagens, GIF/APNG](#9-importação-svg-openraster-lottie-imagens-gifapng)
10. [Importação: captura de movimento BVH, MIDI e outras entradas ao vivo](#10-importação-captura-de-movimento-bvh-midi-e-outras-entradas-ao-vivo)
11. [Formatos de áudio](#11-formatos-de-áudio)
12. [Matriz de suporte de recursos](#12-matriz-de-suporte-de-recursos)
13. [Como deixar os arquivos menores: export-fit, lod, size-report](#13-como-deixar-os-arquivos-menores)
14. [Requisitos do ffmpeg](#14-requisitos-do-ffmpeg)
15. [Referências de patentes e licenças](#15-referências-de-patentes-e-licenças)
16. [Validando o que você gerou](#16-validando-o-que-você-gerou)
17. [Diferenças em relação a outros documentos](#17-diferenças-em-relação-a-outros-documentos)

---

## 1. Qual formato para qual tarefa

| Você quer... | Use | Por quê |
|---|---|---|
| A aparência exata, compartilhada em um único arquivo, que toca em qualquer navegador e permite navegar pela linha do tempo | `.html` | Quadros SVG inline desenhados pelo renderizador próprio do OSATURA; nada é aproximado. |
| Uma animação vetorial pequena para uma página web ou app com um player Lottie | `.json` ou `.lottie` | Vetorial, independente de resolução, pequeno; mas veja a lista de descartes (imagens, traços de largura variável, áudio...). |
| Uma prévia em loop para chat ou um README | `.gif` | Funciona em qualquer lugar; 256 cores por quadro, transparência de 1 bit, sem som. |
| Um substituto do GIF com cor total e alfa | `.apng` | RGBA de 8 bits sem perdas; maior que o GIF. |
| Um vídeo com som | `.mp4` (H.264 + AAC) ou `.webm` (VP9/AV1 + Opus) | Precisa do `ffmpeg` instalado por você. O `.webm` mantém a transparência. |
| Sprite sheet para uma engine de jogos | `.png` | Um PNG mais um atlas JSON com os retângulos dos tiles. |
| Um esqueleto e movimento para uma ferramenta 3D | `.glb` | Ossos, animação, formas planas como malhas de triângulos. |
| Quadros para ferramentas de composição/pintura | `export-oca` (Open Cel Animation) ou `frames --png` | Cels como PNG, em uma pasta. |
| Imagens estáticas em luz linear ou de alta profundidade de bits | `render-hdr` (`.exr` ou `.png` de 16 bits) | Renderizador de ponto flutuante, OpenEXR de 16 bits ou half/float. |
| Um único quadro estático | `render` (`.svg` ou `.png`) | Vetorial ou raster de 8 bits. |

## 2. Formatos de saída em resumo

| Formato | Comando | Extensão | Precisa de ffmpeg | Alfa | Áudio | Tamanho para a demo de 8 quadros* |
|---|---|---|---|---|---|---|
| SVG (um quadro) | `render` | `.svg` | não | sim | não | alguns KB |
| Sequência SVG/PNG | `frames` | pasta | não | sim | não | um arquivo por quadro |
| PNG (um quadro) | `render` | `.png` | não | sim (8 bits) | não | |
| PNG de 16 bits | `render-hdr` | `.png` | não | sim | não | |
| OpenEXR (half/float) | `render-hdr` | `.exr` | não | sim | não | 466 KB half, 923 KB float (640x480) |
| Sprite sheet PNG + JSON | `export` | `.png` (+ `.atlas.json`) | não | sim | não | |
| GIF | `export` | `.gif` | não | 1 bit | não | 83 KB |
| APNG | `export` | `.apng` | não | sim | não | 131 KB |
| WebM | `export` | `.webm` | sim | sim | sim (Opus) | 10.5 KB |
| MP4 | `export` | `.mp4` | sim | não | sim (AAC) | 8.9 KB |
| Player HTML | `export` | `.html`, `.htm` | somente para áudio | sim | mp3 embutido | 66 KB |
| Lottie | `export` | `.json` | não | sim | não | 47 KB |
| dotLottie | `export` | `.lottie` | não | sim | não | 11.5 KB |
| glTF binário | `export` | `.glb` | não | sim | não | 115 KB |
| Open Cel Animation | `export-oca` | pasta | não | sim | não | um PNG por cel |

\* GIF, APNG, WebM e MP4 com `--scale 0.5` (320x240); formatos vetoriais em tamanho total. "Precisa de ffmpeg" significa um
`ffmpeg` funcional no `PATH` (seção 14). Os nomes de saída são escolhidos pela extensão, sem distinguir maiúsculas de minúsculas; uma extensão desconhecida é recusada e
nada é gravado (`unknown export format '.xyz' (use .gif .html .json .mp4 .apng .webm .png .glb)`, e `.lottie` também é
aceito).

Todos os gravadores se recusam a sobrescrever o projeto de entrada com a saída, gravam por meio de um arquivo temporário e o renomeiam, e se recusam a seguir
links simbólicos a menos que `--follow-symlinks` seja informado (CLI_REFERENCE seção 1.4). A proteção de tamanho dos exportadores vetoriais
(Lottie, HTML, GLB) recusa uma exportação de mais de cerca de 40 milhões de valores (quadros x âncoras, mais ossos e camadas por quadro)
com `export would write about N million values ...`.

## 3. Imagens estáticas

### SVG (`render ... out.svg`, `frames`)

```
$ osatura render demo.json 12 f12.svg --bones
wrote f12.svg
$ osatura frames demo.json fr            # fr/frame_0000.svg ... frame_0047.svg
```
A saída vetorial do renderizador próprio do OSATURA: caminhos, gradientes, traços com perfis de largura, tracejados, modos de mesclagem, máscaras e efeitos de
camada são gravados como SVG. `--bones` desenha a sobreposição do esqueleto. Para `.svg`, `--scale` deixa o viewBox do documento inalterado. **Descartes/limites conhecidos:** o renderizador não informa nenhum; como um dado visualizador de SVG trata modos de mesclagem e filtros depende
desse visualizador. `--proxy N` grava um proxy de navegação simplificado que descarta traços finos, gradientes e modos de mesclagem (não serve para
entregas). **Validação:** abra em um navegador, ou `python3 -c "import xml.dom.minidom,sys; xml.dom.minidom.parse(sys.argv[1])" f12.svg`.

### PNG (`render ... out.png`, `frames --png`)

RGBA de 8 bits, sRGB, renderizado pelo rasterizador próprio do motor. `--scale N` em (0, 16] multiplica o tamanho em pixels. Um projeto com
`"background": null` é renderizado transparente onde nada é desenhado. Por padrão não há gerenciamento de cor: os valores se misturam no espaço codificado em sRGB.
Desde a wave 7, um projeto pode optar pela composição em luz linear (`osatura color-space p.json out.json linear`, campo `color_space`):
as saídas raster, GIF, APNG, vídeo e `render-hdr` passam então a ser compostas em luz linear f32 e convertidas para sRGB uma única vez; SVG e
Lottie não mudam (LIMITATIONS L-22).

### PNG de 16 bits e OpenEXR (`render-hdr`)

```
$ osatura render-hdr small.json 3 f.exr                           # HALF, 640x480, 465,611 bytes
$ osatura render-hdr small.json 3 f2.exr linear --precision float # FLOAT, 923,211 bytes
$ osatura render-hdr small.json 3 f4.png                          # 16-bit/colour RGBA, sRGB-encoded
```
* **OpenEXR:** parte única, scanline, **sem compressão**, canais `A B G R`, luz linear, alfa pré-multiplicado. `HALF` por padrão,
  `FLOAT` com `--precision float` (ou `--float32`). A palavra `linear` não tem efeito no EXR (sempre linear).
* **PNG de 16 bits:** codificado em sRGB, a menos que a palavra `linear` seja informada.
* **Fidelidade:** o renderizador de ponto flutuante carrega mais precisão que o de 8 bits, mas a arte é cor sRGB comum, de modo que o arquivo
  contém valores da faixa de exibição com maior precisão, não luz HDR referenciada à cena. O fundo do projeto não é desenhado.
* **Descartes/limites conhecidos:** um quadro por arquivo; sem compressão, camadas nem AOVs; sem perfil ICC; `--8bit` recorre ao rasterizador de 8 bits
  como fonte.
* **Validação:** `ffprobe f.exr` informa `exr 640x480 gbrapf32le`; qualquer visualizador de EXR o abre.

## 4. Animação raster

Todos estes renderizam primeiro cada quadro pelo motor raster. Um projeto com `"background": null` é renderizado com pixels transparentes
onde nada é desenhado; a demo tem uma cor de fundo (`#f6f1e7`) e portanto é opaca. `--scale N` em (0, 16]
vale para `.gif .apng .webm .mp4 .png`; nos outros formatos é recusado (`unknown option --scale for 'export .json'`).

### Sprite sheet PNG (`export ... out.png`)

```
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.atlas.json
wrote out3.png                    # 4 columns x 2 rows of 160x120 tiles
```
Um PNG com todos os quadros como tiles (`--columns N`, por padrão uma grade quase quadrada: ceil(sqrt(frames)) colunas) e um atlas
`out3.atlas.json` (o nome de saída com a extensão substituída por `.atlas.json`) que lista `{index, x, y, w, h}` por quadro. O alfa é mantido.
O atlas costumava ser `out3.json` e podia sobrescrever silenciosamente um arquivo Lottie com o mesmo nome base; o novo nome evita isso, e o caminho do atlas passa pelas mesmas proteções de entrada/symlink.

### GIF (`export ... out.gif`)

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif                      # 8 frames, 83 KB
```
* Uma imagem por quadro de origem, **em loop infinito**, uma paleta local de até 256 cores por quadro (quantizada, sem dithering).
* **Temporização:** os atrasos do GIF são centésimos de segundo inteiros, e os visualizadores tratam qualquer valor abaixo de 2 cs como 10 cs, então 50 fps é o máximo real.
  Até 50 fps os atrasos têm o erro difundido (29.97 fps alterna 3 e 4 cs), de modo que a duração total fica dentro de 1 cs de
  `frames / fps`. Acima de 50 fps a sequência é **decimada para 50 fps**, a duração é mantida e um aviso é impresso
  (`GIF cannot hold 60 fps ... use APNG, WebM or MP4 for the full rate`).
* **Transparência:** 1 bit (o flag de cor transparente é definido); pixels semitransparentes não são preservados.
* **Descartes conhecidos:** sem áudio (`note: GIF cannot carry audio; ...` quando o projeto tem sons); sem alfa parcial; imagens com largura
  ou altura acima de 65535 pixels são recusadas (`image too large for GIF`).
* **`--report r.json`:** grava a lista de coisas que o formato não conseguiu levar (aqui: a decimação da taxa de quadros, se houver).
* **Dicas de tamanho:** um `--scale` menor, menos quadros, ou `export-fit --format gif` (seção 13). O GIF é o menos eficiente dos formatos
  raster para arte fotográfica ou com muitos gradientes.
* **Validação:** `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` imprime o
  número de quadros (8).

### APNG (`export ... out.apng`)

PNG animado, RGBA de 8 bits, sem perdas, um quadro por quadro do projeto no fps do projeto (0.0417 s por quadro a 24 fps). O alfa total é mantido. Sem áudio. Maior
que o GIF para arte chapada (131 KB contra 83 KB no exemplo) porque não há paleta. A maioria dos navegadores toca APNG; algumas ferramentas não,
e nesse caso use WebM ou GIF. **Validação:** `ffprobe` com `-count_frames` (8 quadros no exemplo).

### WebM (`export ... out.webm`) e MP4 (`export ... out.mp4`)

```
$ osatura export small.json out.mp4  --scale 0.5     # h264, yuv420p, 320x240, 8 frames
$ osatura export small.json out.webm --scale 0.5     # vp9, yuv420p + alpha_mode=1
$ osatura export small.json out.webm --codec av1     # needs an ffmpeg with libaom-av1
```
* **MP4:** H.264 (`libx264`), `yuv420p`, **sem alfa**. Áudio AAC quando o projeto tem trilhas de som.
* **WebM:** VP9 (`libvpx-vp9`, padrão) ou AV1 (`libaom-av1`, `--codec av1`); o canal alfa é mantido (`yuva420p`, tag do contêiner
  `alpha_mode=1`); áudio Opus (`libopus`) quando o projeto tem sons. Qualquer outro valor de `--codec`: `unknown --codec 'foo' (vp9 or av1)`.
* **Som:** as trilhas de som do projeto são mixadas (início, gain, trim, fades; cortadas na duração do projeto) e multiplexadas.
* **Precisa de ffmpeg:** os quadros são renderizados em PNG em um diretório temporário privado, entregues ao `ffmpeg` e removidos depois. Se
  o `ffmpeg` estiver ausente: `cannot run ffmpeg (...); install it or export a GIF/HTML instead` (saída 1, nada gravado). Se faltar
  o encoder: `ffmpeg failed`.
* **Limites conhecidos:** somente 8 bits; o tamanho do vídeo segue o tamanho do projeto e `--scale` (verifique o resultado com `ffprobe`).
  `dof-export` também grava `.mp4`/`.webm`, sem áudio.
* **Validação:** `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4`
  imprimiu `h264,320,240,yuv420p,24/1,8`.
* **Patentes:** veja a seção 15; H.264 e AAC são cobertos por pools de licenciamento, VP9/AV1 não foram avaliados.

## 5. Player HTML

```
$ osatura export small.json out.html
wrote out.html                     # 66 KB for 8 frames
```
Um arquivo HTML autocontido: **cada quadro como SVG inline**, um botão de reproduzir/pausar, um controle deslizante (range input) e um rótulo de quadro,
conduzidos por um único `<script>` inline. Ele **não faz nenhuma requisição de rede**: a única URL no arquivo é o namespace SVG. Para abri-lo não é preciso
nada além de um navegador. A imagem é exatamente o que o renderizador SVG produz, portanto é o mais fiel dos formatos "reproduzíveis"
(máscaras, modos de mesclagem, efeitos, ossos, IK, deformadores e partículas são todos incorporados por baking nos quadros).

* **Áudio:** se o projeto tem sons, o áudio mixado é embutido como um data URI mp3 (precisa do `ffmpeg`; há um aviso acima de 3 MiB e o
  áudio é descartado com um aviso acima de 16 MiB).
* **Tamanho:** cresce linearmente com os quadros (cerca de 7.7 KB por quadro aqui) e com o detalhe vetorial; o arquivo não é comprimido. Dicas
  típicas: sirva-o com gzip (o exemplo de 66 KB tem cerca de 9 KB com gzip), reduza o número de quadros ou o detalhe dos caminhos com `lod` e use
  `export-fit --format html`.
* **Limites:** `--report` é recusado (`unknown option --report for 'export .html'`);
  a proteção de tamanho de exportação vetorial se aplica; o player oferece apenas reproduzir/pausar e um controle deslizante.
* **Validação:** abra-o; `grep -c '<svg' out.html` conta os quadros; `grep -o 'http[^"<]*' out.html` deve listar apenas o namespace SVG.

## 6. Lottie (.json) e dotLottie (.lottie)

```
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ osatura export small.json out.lottie       # a zip: manifest.json + a/animation.json
```

### Como é construído

O arquivo é Bodymovin (`"v": "5.7.0"`) com `fr` = fps do projeto, `ip` = 0, `op` = número de quadros e `w`/`h` = o tamanho do palco. **Cada
camada vetorial vira uma camada de forma cuja geometria de caminho passa por baking a cada quadro no espaço do mundo (keyframes hold)**, de modo que ossos, IK,
skinning, deformadores, springs, partículas, a câmera (incluindo parallax) e as transformações dos pais estão todos *dentro* da geometria. O resultado
toca como o original, mas não é um rig editável: não há ossos, canais nem expressões. Uma camada de fundo sólido é
gravada quando o projeto tem uma cor de fundo. Em consequência, o arquivo cresce com `frames x anchors`; ele é muito maior que um
Lottie feito à mão com animação de transformação (47 KB para a demo de 8 quadros, 11.5 KB dentro de um `.lottie`).

### Exportado

Preenchimentos e traços sólidos e com gradiente (paradas de gradiente com alfa; os pontos finais do gradiente seguem a transformação animada da camada), padrões de
tracejado, caps, joins e miter limit, larguras de traço multiplicadas pela escala da camada, modos de mesclagem, opacidade da camada, track mattes de alfa e de alfa invertido
para máscaras de camada (uma cópia da fonte da máscara atua como matte), efeitos de camada (blur, drop shadow, tint, fill) em
camadas de folha única, e **opacidade de grupo como pré-composição** para que filhos sobrepostos não apareçam uns através dos outros. Os números são
gravados na sua menor forma decimal.

### Com aviso (nunca descartado em silêncio)

Cada item abaixo é impresso no stderr como `warning: [code] layer: message` e gravado em `--report FILE` como
`{"format":"lottie","count":N,"warnings":[{"code","layer","message"}]}`:

| Código | Significado |
|---|---|
| `image-layer` | Camadas de imagem não são exportadas (a saída Lottie contém apenas camadas vetoriais). |
| `variable-width-stroke` | Perfis de largura não são suportados; a largura base constante é usada. |
| `stroke-anisotropic` | Escala não uniforme ou skew na camada; os traços do Lottie são uniformes, então a largura é multiplicada pela média geométrica. |
| `gradient-anisotropic` | Um gradiente em tal camada é aproximado (os gradientes do Lottie não sofrem shear). |
| `blend-add` | O modo de mesclagem `add` não tem equivalente no Lottie; `screen` é usado. |
| `group-blend` | Um modo de mesclagem em um grupo não é exportado. |
| `group-mask` | Uma máscara em um grupo não é exportada (as máscaras em camadas individuais são). |
| `group-effect` | Efeitos de camada em um grupo (de várias camadas) não são exportados. |
| `mask-source-not-vector` | A fonte da máscara não é uma camada vetorial, portanto a máscara não é exportada. |
| `depth-of-field` | Os canais `camera_focus`/`camera_blur` não são aplicados; use `dof-export`. |
| `audio` | O Lottie não pode carregar áudio; N trilha(s) de som não estão incluídas. |

### Descartado sem equivalente no Lottie

Ossos, IK, canais, máquinas de estados, data bindings, containers, regiões de scroll, markers/events, sons, cadeias de pós-efeitos (`fx`),
normal maps e a estrutura editável do rig. As cores de efeitos por camada não podem ser animadas (usa-se o primeiro valor) e
os tamanhos estão em unidades do palco (LIMITATIONS L-23).

### dotLottie (`.lottie`)

Um zip [dotLottie 2.0](https://dotlottie.io/spec/2.0/) com `manifest.json` (`{"animations":[{"id":"animation"}],"generator":"OSATURA","version":"2"}`) e
`a/animation.json` (o mesmo Lottie acima, compactado com deflate: 46,843 bytes viraram um arquivo de 11,515 bytes). `--report` funciona da mesma
forma. **Importar** um `.lottie` não é suportado (`import-lottie` precisa do JSON; descompacte `a/animation.json` antes). Até a v0.1.0-alpha.4 o contêiner era dotLottie 1.0 (`animations/animation.json`, `autoplay`/`loop` no manifesto); o layout 2.0 funciona nos players dotLottie atuais (verificado com @lottiefiles/dotlottie-web).

### Validação e ressalvas

* `python3 -m json.tool out.json > /dev/null` para verificar a boa formação; `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],len(d['layers']))"`.
* `unzip -l out.lottie` deve mostrar exatamente as duas entradas.
* Ida e volta: `osatura import-lottie out.json back.json` seguido de `osatura check back.json` verifica que o OSATURA consegue ler a própria
  saída (a ida e volta tem perdas: o rig se perde).
* **Nenhum player Lottie independente estava disponível** quando isto foi escrito (LIMITATIONS L-28): a saída foi verificada contra o layout de campos do Lottie
  e contra o importador do próprio OSATURA. Teste no seu player de destino (lottie-web, player dotLottie, Skottie, ...) e leia os
  avisos; se a exatidão importa, use `.html` ou vídeo.
* **Dicas de tamanho:** `export-fit --format lottie` pesquisa quatro parâmetros (`decimals`, `path_eps`, `key_tol`, `frame_step`) contra um orçamento de bytes;
  `lod` reduz as âncoras antes da exportação; `size-report` mostra o tamanho do Lottie com baking por camada (seção 13).
* **Patentes:** Lottie e dotLottie são formatos abertos; nada em PATENT_NOTES.md se refere a eles.

## 7. glTF binário (.glb)

```
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
```
glTF 2.0 binário (magic `glTF`, versão 2, `"generator": "osatura"`), apenas recursos do núcleo. O projeto deve ter ossos
(`the project has no bones to export` caso contrário).

* **Exato:** a hierarquia de ossos e sua animação (amostrada a cada quadro, interpolação linear, shear descartado).
* **Preenchimentos** viram malhas planas de triângulos (ear clipping). As formas são rígidas (uma camada com um osso segue o nó desse osso) ou **com skin**
  (JOINTS_0/WEIGHTS_0, 4 influências por vértice) quando toda âncora carrega pesos de osso.
* **Traços** são triangulados em suas próprias malhas (largura, perfil de largura, caps butt/round/square, joins miter/round/bevel, tracejados);
  traços translúcidos podem parecer mais escuros onde os triângulos se sobrepõem.
* **Gradientes** (linear e radial) viram um pequeno PNG gerado usado como textura de cor base; spread pad (clamp).
* **Máscaras** passam por baking recortando triângulos contra o preenchimento da camada de máscara, somente para uma fonte de máscara vetorial com preenchimentos opacos e sem
  traços, e apenas para a forma da máscara no quadro 0. **Camadas de imagem** viram quads texturizados ou malhas de deformação com PNG embutido.
* **Path morphs** viram morph targets com pesos animados quando a verificação por quadro reproduz a geometria do motor; caso contrário,
  o caminho permanece uma forma estática do quadro 0 e o motivo é informado.
* **Não exportado (informado como avisos `[glb-limit]`):** modos de mesclagem diferentes de normal (desenhados como normal), opacidade de camada animada (congelada no
  quadro 0), animação de transformação de camada que não seja de osso, efeitos de camada, deformadores além da sua forma no quadro 0, caminhos animados sem um
  morph válido, formas de máscara animadas, som.
* **Coordenadas:** y é negado (o glTF é y-up); as formas são empilhadas com um pequeno deslocamento em z (0.01 por forma) para a ordem de desenho.
* **Validação:** o cabeçalho é `glTF`, versão 2, e o campo de comprimento é igual ao tamanho do arquivo (one-liner em `python3` na seção 16). Use um validador
  ou visualizador de glTF de sua escolha para a verificação completa; nenhum é incluído e nenhum foi executado para este documento. O exemplo acima tem 22 nós,
  15 malhas, 1 skin, 1 animação e 7 imagens embutidas.
* **Dicas de tamanho:** menos âncoras vetoriais (`lod`), sem gradientes (cada um é uma textura), menos quadros (a animação é amostrada por quadro).
* **Patentes:** formato aberto; nenhuma referência em PATENT_NOTES.md.

## 8. Pasta Open Cel Animation (OCA)

```
$ osatura export-oca small.json oca1                 # one layer "render" with all composited frames
$ osatura export-oca small.json oca2 --layers        # one OCA layer per top-level layer
wrote oca2/small.oca
```
Grava `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) mais cels PNG `layerN/fNNNN.png`. Sem `--layers` há uma camada
`render` com os quadros compostos; com `--layers` cada camada de nível superior é renderizada sozinha sobre transparência (11 camadas para a demo).
`--scale N` multiplica o tamanho do cel. **Descartes conhecidos:** os cels são rasters (nenhum vetor, osso ou canal sobrevive); a opacidade da camada e a mesclagem
não são gravadas (toda camada é um `paintlayer`, opacidade 1); os sons não são exportados; arquivos dentro de uma pasta existente são sobrescritos.
**Importação:** não há comando `import-oca` na CLI. **Validação:** o arquivo `.oca` é JSON (`python3 -m json.tool`); todo cel listado
deve existir como PNG.

## 9. Importação: SVG, OpenRaster, Lottie, imagens, GIF/APNG

As importações criam um projeto **novo** (não se mesclam a um projeto existente), salvo indicação em contrário; use `script` ou edite o JSON para combinar.

### SVG (`import-svg`)

```
$ osatura import-svg art.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
wrote svg.json (3 paths)
```
Subconjunto: `path` (todos os comandos), `rect` (cantos arredondados ignorados), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` com `transform`,
`fill` sólido, `stroke`, `stroke-width`. Resultado: 24 fps, 24 quadros, tamanho a partir de `viewBox` ou `width`/`height` (senão 512x512), **uma camada
vetorial `svg`**. Ignorados com uma linha `warning:` (impressa no **stdout**): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`,
`<linearGradient>`/`<radialGradient>` (uma tinta de gradiente recorre ao cinza), `<image>`, `<use>`, `<foreignObject>`. Limites: 8 MB de SVG,
200,000 âncoras. Para fotografias e rasters use `trace` / `autotrace` (CLI_REFERENCE seção 9).

### OpenRaster (`import-ora`)

```
$ osatura import-ora art.ora ora.json --fps 12 --frames 24
```
Uma **camada de imagem por camada raster** (PNG embutido), ordem de baixo para cima, camadas ocultas mantidas porém invisíveis, grupos achatados, opacidade,
visibilidade e posição por camada, e as operações de composição `svg:` comuns (multiply, screen, overlay, darken, lighten, color-dodge,
color-burn, hard-light, soft-light, difference, exclusion, plus mapeado para add). A opacidade de uma pilha aninhada se multiplica em suas camadas e seu
modo de mesclagem é herdado, mas camadas sobrepostas dentro de uma pilha translúcida **não** são achatadas antes. Nomes duplicados recebem `_1`, `_2`.
Limites: 2000 entradas de arquivo, 128 MiB por entrada, camadas de 64 Mpixel. Um arquivo que não é zip: `not a zip/ORA file: ...` (saída 1). Não há exportação
ORA. A importação de PSD não está implementada (LIMITATIONS L-32).

### Lottie (`import-lottie`)

```
$ osatura import-lottie in.json out.json [fx.json]
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
```
Um **subconjunto** deliberado; cada recurso ignorado é uma linha `warning:`.
* **Importado:** camadas de forma com caminhos `sh`, `rc`, `el`; preenchimentos sólidos e com gradiente (pontos finais/centro/raio de gradiente animados viram
  canais `grad_dx`/`grad_dy`); traços; grupos; transformações de camada (posição, escala, rotação e opacidade estáticas ou com keyframes; caminhos de posição
  curvos passam por baking a cada quadro); modos de mesclagem; uma camada de fundo sólido; camadas de imagem com data URIs PNG/JPEG embutidos; camadas nulas como pais
  (suas transformações passam por baking nos filhos); máscaras de camada (add/subtract/intersect com opacidade, inversão, formas com keys); mattes de luma
  (aproximados); pré-composições; intervalos de entrada/saída da camada (como uma trilha de passos de opacidade); easing de keyframes (hold vira step, handles bezier viram
  cubic-bezier, nenhum vira linear).
* **Caminhos animados:** keys de forma com a mesma topologia (mesma contagem de âncoras e aberto/fechado) viram um caminho base mais um **morph target**
  por key distinta adicional (`k1`, `k2`, ...) com canais de peso `path_morph` que carregam o easing. Uma topologia diferente usa a primeira forma
  e imprime um aviso.
* **Efeitos (`ef`):** Gaussian blur, drop shadow, tint e fill viram **efeitos por camada** com números animados como canais `fx_*`;
  tint/fill/tritone/levels estáticos em camadas de forma passam por baking nas cores. As cores de efeitos não podem ser animadas (usa-se o primeiro valor); os demais
  efeitos geram avisos.
* **Ignorados com um aviso:** texto e outros tipos de camada, assets de áudio, pontos de ancoragem animados (usa-se o primeiro valor), caminhos de movimento curvos em
  propriedades que não sejam a posição, máscaras em uma fonte de track matte, cadeias de pais longas demais para baking, mais de 2000 camadas, animação mais longa
  que 20,000 quadros.
* `fx.json` é mantido por compatibilidade: um array vazio é gravado.
* **Não suportado:** entrada em zip `.lottie`. **Pegadinha:** não há proteção contra sobrescrever a entrada: `import-lottie in.json in.json` substitui o
  Lottie pelo projeto.

### Imagens, GIF e APNG como camadas quadro a quadro

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20      # folder of PNGs
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0                    # animated GIF/APNG
```
`import-seq` pega todos os PNGs de uma pasta na ordem dos nomes de arquivo (todos do mesmo tamanho); `import-anim` lê um GIF ou APNG animado usando os
atrasos do próprio arquivo (um PNG simples dá um quadro). Ambos adicionam uma camada switch com uma imagem embutida por quadro mais um canal de passos; funcionam em um projeto
**existente**. Um GIF longo deixa o projeto grande (cada quadro é um PNG embutido). `trace` e `autotrace` transformam bitmaps em caminhos
vetoriais (`trace` lê o canal alfa; `autotrace` quantiza as cores e ajusta curvas Bezier).

## 10. Importação: captura de movimento BVH, MIDI e outras entradas ao vivo

### BVH (`import-bvh`, `mocap-info`, `mocap-match`, `mocap-retarget`)

```
$ osatura mocap-info walk.bvh
$ osatura import-bvh demo.json walk.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```
`import-bvh` mapeia canais de rotação de juntas escolhidos para ossos como canais `bone_angle` com keys, amostrados no fps e no número de quadros do projeto
(`--axis` seleciona `Zrotation`, `Xrotation` ou `Yrotation`; `--tol` simplifica as keys). `mocap-match` propõe uma correspondência entre juntas e ossos com uma
tabela de confiança e `mocap-retarget` faz o baking de canais acrescentados com movimento da raiz, uma vista (`front|side|top|yaw=DEG`) e escala. Somente o
subconjunto de texto do BVH: sem formatos binários de mocap, sem FBX (por projeto). **Limites (LIMITATIONS L-45):** nenhum arquivo real de captura de movimento foi testado; a projeção
2-D descarta a profundidade; a torção em torno do eixo de um osso, os dedos e o rosto são ignorados; o rig mantém os comprimentos dos seus ossos, então o deslizamento dos pés é apenas
reduzido.

### MIDI e outras entradas ao vivo (`live-map`)

```
$ osatura live-map small.json song.mid mapping.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote out.json
```
Um **Standard MIDI File** (`.mid`; mensagens MIDI 1.0, running status, eventos meta de tempo, divisão em ticks ou SMPTE) ou um JSON de eventos
(`{"events":[{"source","id","value","time"}]}`, valor normalizado 0..1, tempo em segundos) é transformado em canais com keys, events e takes por meio de um
arquivo de mapeamento. Fontes: `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`. No máximo
3,000,000 eventos; tempos dentro de 7 dias. Nada se comunica com um dispositivo MIDI a partir da CLI; o studio usa o Web MIDI do navegador. O MIDI é **somente
importação** (não há exportação MIDI). Arquivos malformados retornam um erro, nunca uma falha abrupta.

## 11. Formatos de áudio

| Uso | Formatos | Observações |
|---|---|---|
| Análise (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) | somente **WAV** | PCM inteiro de 8/16/24/32 bits ou float de 32 bits, qualquer número de canais mixado em mono, até 256 MiB. Converta MP3/OGG antes (`ffmpeg -i in.mp3 out.wav`). |
| Trilhas de som do projeto (`sound add`) | WAV, OGG, MP3, qualquer coisa que o `ffprobe` aceite | Guardado como **referência** (caminho relativo, quadro inicial, gain 0..4, janela de trim, fades); nada é embutido; o arquivo deve permanecer dentro da pasta do projeto, a menos que se use `--allow-external-assets`. |
| Saída mixada (`sound mix`) | `.wav` (PCM 16 bits a 44,1 kHz), `.ogg` (Vorbis q4), `.mp3` (96 kb/s) | Mixada pelo ffmpeg para a **duração do projeto**; `.flac` e outros: `unknown audio format '.flac' (use .wav, .ogg or .mp3)`. |
| Dentro das exportações | `.mp4` AAC, `.webm` Opus, `.html` mp3 embutido | `.gif`/`.apng` ficam silenciosos com uma nota; o Lottie avisa `[audio]`; GLB e OCA descartam o som. |
| Mouth cues (`mouth`) | Texto Papagayo/Moho-switch, TSV do Rhubarb | Não é áudio: são arquivos de cues. |

Encoders usados: `libmp3lame` (mp3), `libvorbis` (ogg), `aac`, `libopus`. Sem pan/EQ/ducking nem loop por trilha (ROADMAP seção 9).

## 12. Matriz de suporte de recursos

As linhas são coisas que um projeto pode conter, as colunas são os formatos de saída. **yes** = levado como está no projeto, **approx** = levado com uma
diferença visível, **baked** = presente apenas como resultados por quadro, **no** = descartado. As células de formatos que não pude verificar em um visualizador
independente descrevem o que o exportador grava.

| Recurso | SVG / PNG | Sprite sheet / APNG | GIF | WebM / MP4 | HTML | Lottie / .lottie | GLB | OCA |
|---|---|---|---|---|---|---|---|---|
| Formas vetoriais, preenchimentos | yes | baked (raster) | baked (raster) | baked (raster) | yes (SVG) | yes | approx (triângulos) | baked (raster) |
| Gradientes | yes | baked | baked | baked | yes | yes (approx se a camada tem skew) | yes (textura) | baked |
| Traços com perfil de largura | yes | baked | baked | baked | yes | approx (largura constante) | yes | baked |
| Tracejados, caps, joins | yes | baked | baked | baked | yes | yes | yes (triangulado) | baked |
| Camadas de imagem (bitmap) | yes | baked | baked | baked | yes | no (`image-layer`) | yes | baked |
| Ossos e animação | baked | baked | baked | baked | baked | baked | **yes (exato)** | baked |
| Skinning / deformadores | baked | baked | baked | baked | baked | baked | com skin; deformadores no quadro 0 | baked |
| Câmera e parallax | baked | baked | baked | baked | baked | baked (os movimentos de câmera estão na geometria) | no (as formas vêm do quadro 0) | baked |
| Modos de mesclagem | yes | baked | baked | baked | yes | approx (`add` vira `screen`; nenhum em grupos) | no (normal) | baked |
| Máscaras de camada | yes | baked | baked | baked | yes | somente camadas individuais | forma do quadro 0, restrita | baked |
| Efeitos de camada (blur, sombra...) | yes | baked | baked | baked | yes | camadas de folha única | no | baked |
| Opacidade animada | baked | baked | baked | baked | baked | yes | no (congelada no quadro 0) | baked |
| Path morphs | baked | baked | baked | baked | baked | baked | morph targets (quando válidos) | baked |
| Profundidade de campo | via `dof-export` | via `dof-export` | via `dof-export` | via `dof-export` | no | no (com aviso) | no | no |
| Cadeia de pós-efeitos (`fx`) | somente PNG | no | no | no | no | no | no | no |
| Transparência | yes | yes (alfa) | 1 bit | WebM yes, MP4 no | yes | yes | yes | yes |
| Áudio | no | no | no | **yes** | yes (mp3) | no (com aviso) | no | no |
| Loop | n/a | sheet: n/a, APNG: yes | yes | depende do player | player | player (o dotLottie 2.0 deixa o loop ao player ou a uma máquina de estados) | n/a | n/a |
| Rig/canais editáveis depois | no | no | no | no | no | no | somente ossos | no |
| Máquinas de estados, bindings, containers, scroll | no | no | no | no | no | no | no | no |
| Independente de resolução | yes (SVG) | no | no | no | yes | yes | yes | no |
| Precisa de ffmpeg | no | no | no | **yes** | somente áudio | no | no | no |

"Baked" significa que o efeito do recurso está nos pixels ou na geometria por quadro, mas o recurso em si não pode ser editado depois.

## 13. Como deixar os arquivos menores

1. **Veja onde estão os bytes:** `osatura size-report project.json` lista o arquivo do projeto, o Lottie com baking e a página HTML por camada,
   imagem e canal (`--json` para scripts).
2. **Ajuste a um orçamento automaticamente:** `osatura export-fit project.json out.json --max-size 50KB --format lottie` pesquisa os parâmetros do exportador e
   mantém o melhor SSIM que couber. Formatos: `lottie`, `html`, `gif`, `apng`, `webm`, `project` (um JSON de projeto enxugado). `--measure gzip` orça o
   tamanho em gzip, `--min-ssim X` define um piso de qualidade, `--max-seconds N` é um limite real de tempo de relógio (o melhor resultado até então é mantido). Sai com código 1 com o
   menor resultado quando o orçamento não pode ser cumprido.
3. **Reduza primeiro a arte:** `osatura lod project.json --tolerance-px 1 --levels 3 --out-dir lv --verify` grava projetos de nível de detalhe com um
   erro declarado (e, com `--verify`, medido) em pixels de saída; `lod-pick --screen-px WxH` escolhe o nível mais grosseiro para uma dada tela.
4. **Por formato:**
   * Lottie/HTML/GLB crescem com `frames x anchors`: menos quadros, menos âncoras, um fps menor, intervalos de keys aparados (ferramentas `keys`).
   * GIF/APNG crescem com pixels x quadros: `--scale 0.5` é uma redução de 4x; o GIF serve para arte chapada, o APNG serve para alfa, WebM/MP4 servem para qualquer coisa com movimento.
   * `.lottie` é o mesmo Lottie compactado com deflate (cerca de 4x menor no exemplo); sirva `.html`/`.json` com gzip.
   * As imagens de um projeto são embutidas como PNG em base64: reduza-as antes de importar.
5. Limites: um Lottie de um rig de 14 camadas não pode ser reduzido arbitrariamente (um orçamento de 1 KB no exemplo informa "every knob is at its floor"); a
   pesquisa precisa de tempo em projetos grandes.

## 14. Requisitos do ffmpeg

O OSATURA nunca inclui nem faz link com o ffmpeg; ele executa o `ffmpeg` (e o `ffprobe`) encontrado no `PATH` como um **subprocesso externo** da sua própria instalação.

| Necessário para | Encoder/decoder | Verificado em |
|---|---|---|
| `export .mp4` | `libx264` (`yuv420p`), `aac` | ffmpeg 6.1.1 |
| `export .webm` (padrão) | `libvpx-vp9`, `libopus` | ffmpeg 6.1.1 |
| `export .webm --codec av1` | `libaom-av1`, `libopus` | ffmpeg 6.1.1 |
| `dof-export .mp4/.webm` | igual ao acima, `--codec` | ffmpeg 6.1.1 |
| `sound mix .ogg` | `libvorbis` | ffmpeg 6.1.1 |
| `sound mix .mp3`, áudio de `.html` | `libmp3lame` | ffmpeg 6.1.1 |
| `face-track` a partir de um vídeo | somente decoders | |
| verificação de conteúdo de `sound add` | `ffprobe` (sem ele, só a existência é verificada para arquivos que não sejam WAV) | |

Verifique o seu com `ffmpeg -hide_banner -encoders | grep -E 'libx264|aac|libvpx-vp9|libaom-av1|libopus|libvorbis|libmp3lame'`. Todo o resto
(GIF, APNG, HTML sem áudio, Lottie, dotLottie, GLB, OCA, PNG, SVG, EXR) funciona sem o ffmpeg.
Se o ffmpeg estiver ausente, o comando falha com `cannot run ffmpeg (...)` e não grava nada; um encoder ausente produz `ffmpeg failed`.

## 15. Referências de patentes e licenças

* [PATENT_NOTES.md](PATENT_NOTES.md) registra o que **não** foi avaliado. Para exportação, a linha relevante é *Codecs de vídeo e áudio via ffmpeg
  (`export` .mp4 / .webm, `soundtrack`)*: H.264 e AAC são cobertos por pools de licenciamento; VP9 e AV1 são divulgados como livres de royalties, mas reivindicações de pools e individuais
  foram anunciadas para ambos; **status: não avaliado, não trate VP9/AV1 como liberados.** Como o ffmpeg é a instalação do próprio usuário, toda
  escolha de codec é responsabilidade do usuário, e o OSATURA não deve incluir o ffmpeg.
* Os gravadores determinísticos de raster, GIF, APNG, PNG, SVG, OpenEXR, OCA, ORA, glTF e Lottie são implementações próprias do OSATURA a partir de descrições públicas de
  formato (clean-room, veja CLEANROOM.md). Seu status de patentes não foi avaliado além disso: PATENT_NOTES.md lista apenas as áreas acima
  (puppeteering guiado por rosto, máquinas de estados e data binding, FABRIK/MVC/pesos, preenchimento stencil-then-cover, codecs).
* GIF: as patentes do LZW expiraram há muito tempo em todas as jurisdições; isto é conhecimento geral, não uma liberação registrada no repositório.
* Licenças dos arquivos produzidos: os arquivos que você exporta são seus; o motor é MPL-2.0 e a CLI é AGPL-3.0-or-later, o que se aplica aos programas, não aos
  arquivos que eles gravam. O studio oferece seu código-fonte correspondente em `/source` (AGPL seção 13).
* Isto não é aconselhamento jurídico.

## 16. Validando o que você gerou

| Formato | Verificação | Esperado |
|---|---|---|
| qualquer projeto | `osatura check out.json` | `ok: ...` (também depois de `import-*`) |
| GIF | `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` | contagem de quadros (8) |
| APNG | o mesmo comando em `out.apng` | contagem de quadros (8) |
| MP4 | `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4` | `h264,320,240,yuv420p,24/1,8` |
| WebM | o mesmo em `out.webm` com `:stream_tags=alpha_mode` | `vp9,...,yuv420p,...,1` (alpha_mode 1) |
| EXR | `ffprobe f.exr` | `exr 640x480 gbrapf32le` |
| Lottie | `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],d['w'],d['h'],len(d['layers']))"` | `5.7.0 24.0 8 640.0 480.0 12` |
| Avisos do Lottie | `osatura export p.json out.json --report rep.json` | `rep.json` lista cada aproximação |
| dotLottie | `unzip -l out.lottie` | somente `manifest.json` e `a/animation.json` |
| Ida e volta do Lottie | `osatura import-lottie out.json back.json && osatura check back.json` | `ok:` |
| GLB | `python3 -c "import struct;b=open('out.glb','rb').read();print(b[:4],struct.unpack('<II',b[4:12]),len(b))"` | `b'glTF' (2, N)` com N igual ao comprimento |
| HTML | `grep -c '<svg' out.html`; `grep -o 'http[^"<]*' out.html` | um `<svg` por quadro; somente a URL do namespace SVG |
| SVG | abra em um navegador ou analise como XML | bem formado |
| Sprite sheet | `python3 -m json.tool out.atlas.json` | `{index,x,y,w,h}` por quadro |
| OCA | `python3 -m json.tool <name>.oca` | todo cel referenciado existe |
| Regressão visual | `osatura ci diff a.png b.png --metric ssim` | saída 0 quando iguais; veja [CI.md](CI.md) |

Para verificações visuais de uma exportação com perdas em comparação com o original, `osatura export-fit` já informa SSIM e CIEDE2000 em relação à renderização original, e
`osatura compare` grava uma imagem de antes/depois/diferença de dois projetos.

## 17. Diferenças em relação a outros documentos

* `docs/LIMITATIONS.md` L-29 diz que a importação de Lottie não aceita assets de imagem, efeitos, precomps nem mattes; o importador atual importa imagens
  com data URIs embutidos, pré-composições, efeitos por camada (blur, drop shadow, tint, fill) e aproxima mattes de luma (seção 9). A lista de
  recursos ignorados na seção 9 é aquilo sobre o qual o comando realmente avisa.
* `docs/LIMITATIONS.md` L-32 lista o OCA entre os formatos de importação; a CLI tem uma **exportação** OCA (`export-oca`) e nenhum comando `import-oca`.
* O texto de uso de `export` nomeia apenas `.gif .html .json .mp4` mais "os formatos abertos"; a lista completa de extensões é a da seção 2.
* Outras divergências entre o texto de uso e o comportamento estão reunidas no final de [CLI_REFERENCE.md](CLI_REFERENCE.md).
