# CI de animação (`osatura ci`)

Trate uma animação como código testável: um manifesto lista verificações, um job de CI as executa, e uma
mudança visual que falha vem acompanhada de um relatório que mostra o que se moveu. Tudo é local, offline e
determinístico; as métricas são as clássicas (CIEDE2000, SSIM, MSE/PSNR, um hash DCT).

```
osatura ci init  <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run   [manifest] [--update] [--bless] [--prune] [--only a,b]
                  [--junit out.xml] [--html out.html] [--json out.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest] [--only a,b] [--prune]
osatura ci diff  a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--pixel-de D] [--aa N]
                  [--ignore x,y,w,h;x,y,w,h] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```

Códigos de saída: **0** todas as verificações passaram (ou apenas info / updated / blessed), **1** pelo menos uma verificação
falhou (ou `ci diff` encontrou uma diferença), **2** algo não pôde ser avaliado (projeto ausente,
manifesto inválido, golden ilegível ou adulterado, opção desconhecida).

## Início rápido

```
osatura ci init animation.json        # writes osatura-ci.json and goldens/ next to the project
osatura ci run --html report.html     # verify; exit 1 and a report on regressions
osatura ci bless                      # after reviewing the report: accept the new look
git add osatura-ci.json goldens       # commit the goldens with the change that caused them
```

`init` calibra o manifesto com o projeto como ele está agora: cerca de seis quadros igualmente espaçados em uma
escala de proxy (lado maior de no máximo 480 px), uma verificação com o renderizador de ponto flutuante em dois deles, determinismo,
validate, um teto para a taxa de falhas do pose-fuzz (taxa medida mais 2 pontos; o rig da demo falha na maioria das
amostras de faixa completa, então seu teto é 100 % e a verificação não protege nada até você reduzi-lo),
limites com 50 % de folga, um orçamento de tamanho de GIF (+50 %), uma verificação da emenda do loop (somente quando o projeto
entra em loop sem falhas hoje) e um orçamento informativo de velocidade de renderização (4 x a mediana medida, que
depende da máquina e de builds debug versus release).

## Manifesto (`osatura-ci.json`)

```json
{
  "schema": "osatura-ci/1",
  "project": "animation.json",
  "goldens": "goldens",
  "defaults": { "max_de": 6.0 },
  "checks": [
    { "kind": "validate", "name": "validate" },
    { "kind": "render", "name": "frames", "frames": [0, 12, 24, 47], "scale": 0.5,
      "thresholds": { "min_ssim": 0.995 }, "ignore": [[500, 0, 140, 40]] },
    { "kind": "render", "name": "tablet", "step": 12, "size": "1024x768", "scale": 0.5 },
    { "kind": "render", "name": "svg-text", "frames": [0], "renderer": "svg" },
    { "kind": "determinism", "name": "determinism", "frames": [0, 47] },
    { "kind": "fuzz", "name": "pose-fuzz", "samples": 200, "max_fail_rate": 0.05 },
    { "kind": "limits", "name": "limits", "max_layers": 200, "max_file_bytes": 2000000 },
    { "kind": "export-size", "name": "gif", "format": "gif", "max_bytes": 800000, "scale": 0.5 },
    { "kind": "loop", "name": "loop-seam" },
    { "kind": "perf", "name": "speed", "budget_ms": 40, "tolerance": 0.5, "enforce": false }
  ]
}
```

Os caminhos são relativos ao manifesto. Campos e tipos desconhecidos são erros; todos os problemas de um manifesto
são relatados juntos (os nomes devem ter de 1 a 64 caracteres entre letras, dígitos, `-`, `_`, `.`; os nomes são
únicos; toda verificação precisa de um projeto; os intervalos são verificados).

| kind | o que verifica | falha quando |
| --- | --- | --- |
| `render` | quadros (lista `frames`, ou `step`, por padrão primeiro/meio/último), `scale`, `size` (os containers de layout são resolvidos de novo), `renderer`: `raster` (tiny-skia), `float` (rasterizador f32, quantizado para 8 bits) ou `svg` (comparação exata de texto) | a imagem difere do golden além dos limiares, não existe golden, ou o tamanho mudou |
| `validate` | o projeto compila; achados do linter na severidade `lint` (`error` padrão, `warn`, `info`, `off`) | qualquer achado desse tipo ou um erro de compilação |
| `fuzz` | o pose fuzzer (`samples`, `seed`, `frame`) | taxa de falhas acima de `max_fail_rate` (padrão 0). A verificação **bloqueia por padrão** (`gate` não definido significa `true`); ela só é relatada, com status `info` e sem nunca falhar, quando `"gate": false` ou `max_fail_rate` é 1 ou mais (um limite que não pode falhar nunca é mostrado como aprovado). O comando `osatura fuzz` é uma ferramenta separada que só gera relatório, a menos que receba `--gate` ou `--max-fail-rate` |
| `limits` | `max_layers`, `max_bones`, `max_channels`, `max_frames`, `max_canvas_pixels`, `max_file_bytes` | uma contagem excede seu limite |
| `export-size` | bytes de `gif`, `apng`, `lottie`, `html` ou `svg-frames` (exportadores existentes) | maior que `max_bytes` |
| `determinism` | cada quadro renderizado duas vezes pelo mesmo motor e por um motor recém-carregado; o SHA-256 dos pixels (ou do texto SVG) deve coincidir; `renderers` padrão raster, float, svg | qualquer digest difere |
| `loop` | emenda entre o último e o primeiro quadro versus a variação média entre quadros (diferença absoluta média de canal em porcentagem); `max_seam_ratio` padrão 2, `max_seam_mad` opcional | emenda maior que o permitido; no máximo 600 quadros |
| `perf` | mediana de ms por quadro, `budget_ms` com `tolerance` relativa (padrão 0.5) | somente quando `enforce` é true; caso contrário, status `info` |

### Limiares de render

Definidos em `defaults` (todas as verificações de render) e sobrescritos por verificação em `thresholds`.

| chave | padrão | significado |
| --- | --- | --- |
| `metric` | `de2000` | `de2000` (CIEDE2000) ou `de76` (CIE76, euclidiana em Lab) |
| `pixel_de` | 1.0 | um pixel conta como diferente acima desta diferença de cor |
| `max_de` | 8.0 | maior diferença de cor permitida entre os pixels contados |
| `min_ssim` | 0.99 | menor SSIM médio permitido |
| `max_diff_fraction` | 0.001 | maior fração permitida de pixels contados (0.1 %) |
| `aa_isolated` | 2 | tolerância de anti-aliasing: grupos contados de no máximo esta quantidade de pixels 8-conectados, todos sobre uma borda, são perdoados (0 = desligado) |
| `aa_edge_de` | 0 (desligado) | pixels de borda que diferem em no máximo isto são perdoados (mais folgado que `pixel_de`, apenas nas bordas) |
| `max_phash_distance` | não verificado | maior distância de Hamming dos hashes DCT de 64 bits |

Um quadro cujos pixels são idênticos byte a byte ao golden passa sem nenhuma métrica. Caso contrário, o
quadro é comparado pixel a pixel; cada pixel é composto sobre branco e sobre preto e a maior
diferença é usada, de modo que uma mudança apenas de alfa fica visível. Um pixel é um pixel de borda quando dois vizinhos
opostos (limitados na margem) em qualquer das imagens diferem em mais de 40 em algum canal; o valor do próprio
pixel é ignorado, de modo que um pixel isolado em uma área chapada é uma diferença real e a borda de uma forma
não é. Os retângulos de `ignore` são `[x, y, w, h]` em unidades do palco; eles são igualados nas duas imagens antes de
qualquer métrica, portanto também não contam no SSIM, no MSE nem no hash.

Como orientação, no projeto de demonstração o rasterizador de CPU e o rasterizador float (dois caminhos de código
diferentes para a mesma cena) diferem por SSIM 0.9997, 0.83 % dos pixels acima da diferença de cor 1 (mais 386
perdoados como anti-aliasing), maior diferença de cor 11.3 nas bordas das formas. Um deslocamento de 40 unidades da camada do sol
dá SSIM 0.993, 6.4 % dos pixels, maior diferença de cor 19.6 em uma região.

### Goldens: `--update` versus `bless`

Os goldens ficam em `goldens/` como arquivos PNG nomeados pelos primeiros 24 dígitos hexadecimais de um SHA-256 sobre tamanho e
pixels (quadros idênticos compartilham um arquivo), mais `goldens/index.json` que mapeia verificação e quadro para arquivo.
Os goldens SVG são arquivos `.svg` nomeados da mesma forma. Um golden é conferido com o seu nome quando
é lido, de modo que um arquivo editado à mão ou corrompido é um erro (saída 2), não uma aprovação silenciosa.

* `ci run --update` reescreve todos os goldens das verificações de render selecionadas a partir da renderização atual e
  esquece os quadros que não estão mais listados. Use-o quando um manifesto mudou ou depois de uma mudança global
  intencional.
* `ci bless` (ou `ci run --bless`) aceita apenas os quadros que atualmente falham ou não têm golden e
  deixa os aprovados intocados, de modo que o ruído de tolerância não faça os arquivos oscilarem. O relatório os marca
  como `blessed`.
* `--prune` exclui arquivos golden aos quais nada se refere. Faça o commit de `goldens/` junto com a mudança que o causou.

### Relatórios

`--json` é o resultado completo (schema `osatura-ci-report/1`; métricas, regiões, limiares usados).
`--junit` é XML JUnit com um caso de teste por verificação. `--html` é uma página autocontida (sem recursos
externos): resumo, tabela de verificações e, para cada quadro com falha, o golden, a renderização atual e um
mapa de diferenças como data URIs PNG embutidos, um botão de alternância para oscilar entre golden e atual, e
a lista das regiões que diferem. `--artifacts dir` também grava `dir/<check>/fNNNNN-{golden,actual,diff}.png`
para os quadros com falha. O mapa de diferenças mostra o golden esmaecido, as diferenças contadas em uma rampa de amarelo a
vermelho-escuro conforme a diferença de cor, o anti-aliasing perdoado em azul-claro, as áreas ignoradas hachuradas em cinza
e as caixas de região em magenta. `ci diff` também pode gravar uma faixa lado a lado, uma sobreposição vermelho/ciano e um
GIF de oscilação de dois quadros; use `--flicker` em um par golden/actual para obter um GIF de antes/depois.

## GitHub Actions (exemplo, não executado)

Somente documentação: este workflow foi escrito para corresponder à CLI, mas **não** foi executado em um serviço de CI
real. Ajuste a instalação do Rust e os caminhos ao seu repositório.

```yaml
name: animation-ci
on:
  pull_request:
  push:
    branches: [main]
jobs:
  animation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        run: rustup toolchain install stable --profile minimal
      - name: Build osatura
        run: cargo build --release -p osatura-cli
      - name: Animation checks
        run: |
          ./target/release/osatura ci run animation/osatura-ci.json \
            --junit ci-out/junit.xml --html ci-out/report.html --json ci-out/report.json \
            --artifacts ci-out/frames
      - name: Upload report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: animation-ci-report
          path: ci-out/
      - name: Publish test results
        if: always()
        uses: mikepenz/action-junit-report@v4   # any JUnit consumer works
        with:
          report_paths: ci-out/junit.xml
```

O código de saída 1 (regressão visual ou um orçamento) e 2 (não foi possível avaliar) fazem o job falhar; o relatório
é enviado de qualquer forma. Para aceitar uma mudança intencional, execute `osatura ci bless` localmente, revise o
diff de `goldens/` e faça o commit.

## Script de shell genérico (exemplo, não executado)

```sh
#!/bin/sh
# ci.sh - run animation checks; works in any CI that can run a shell and keep files.
set -u
MANIFEST="${1:-animation/osatura-ci.json}"
OUT="${OUT:-ci-out}"
mkdir -p "$OUT"

osatura ci run "$MANIFEST" \
  --junit "$OUT/junit.xml" --html "$OUT/report.html" --json "$OUT/report.json" \
  --artifacts "$OUT/frames"
code=$?

case $code in
  0) echo "animation checks passed" ;;
  1) echo "animation regression: open $OUT/report.html" ;;
  *) echo "animation CI could not run (exit $code); see the log above" ;;
esac
exit $code
```

## Determinismo e outras máquinas

O que foi verificado (uma máquina Linux x86-64, o rasterizador de CPU `tiny-skia` e o rasterizador f32):

* renderizar o mesmo quadro duas vezes em um processo, e a partir de um motor recém-carregado, dá bytes
  idênticos (a verificação `determinism`; raster, float e texto SVG);
* os goldens gravados por um processo foram reproduzidos byte a byte por processos posteriores (`init` e depois `run`,
  `bless` e depois `run`), nos dois renderizadores;
* verificações independentes das métricas: todos os 24 pares CIEDE2000 de Sharma, Wu e Dalal (2005) usados nos
  testes unitários são reproduzidos com precisão de 5e-5; nos quadros da demo, o SSIM (depois de corrigido para a região de borda que
  o scikit-image corta), o maior valor de CIEDE2000 e o número de pixels diferentes coincidem
  com o scikit-image 0.26 (0.99330 contra 0.99330 após a correção de borda; 19.648 contra 19.648; 11088
  contra 11088 pixels), e o MSE com o numpy.

O que **não** foi verificado: outros sistemas operacionais, CPUs, compiladores ou versões do Rust. Os renderizadores
usam ponto flutuante (trigonometria, achatamento de curvas, acumulação de cobertura), de modo que uma libm diferente,
contração FMA ou caminho SIMD pode mover um pixel de borda com anti-aliasing em um ou dois níveis. Hashes exatos
só ajudam, portanto, em uma build da ferramenta em um tipo de máquina; entre máquinas, confie nas
tolerâncias (`pixel_de`, `aa_isolated`, `max_diff_fraction`) e mantenha os goldens produzidos pela
mesma imagem de CI que os verifica. O renderizador de GPU não faz parte deste conjunto de testes.

## Limites

* Não há rasterizador de SVG no workspace, portanto uma verificação de render não pode rasterizar a saída SVG e
  comparar pixels; o renderizador `svg` compara o texto SVG exatamente (qualquer reordenação de atributos é uma
  falha).
* As sobreposições de ossos não são renderizadas por nenhuma verificação de CI; o áudio não é comparado.
* A fusão de regiões e a regra de anti-aliasing são heurísticas (teste de borda com um passo fixo de 40, limite de tamanho
  de grupo); um deslocamento subpixel de uma borda longa forma um grupo longo e fino e é relatado, não perdoado.
* Os limiares perceptuais são padrões, não calibrados segundo o julgamento de animadores.
* A verificação de perf mede o tempo de relógio em uma máquina compartilhada; mantenha-a informativa, a menos que o
  runner seja dedicado.
* `ci init` renderiza, faz fuzz e exporta uma vez para calibrar, o que é lento em builds debug.
* Sem integração com pull requests (sem comentário de bot, sem envio automático de GIF de antes/depois): as peças
  (`--html`, `--artifacts`, `ci diff --flicker`) existem, a ligação com um serviço de hospedagem não.
