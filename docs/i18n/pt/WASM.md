# OSATURA para WebAssembly

*Versão da ABI 1 (`arm_abi_version()`). Crate `osatura-wasm`, wrapper JavaScript `osatura-wasm/js/osatura.js`, página de demonstração `osatura-wasm/index.html`, script de compilação `osatura-wasm/build-wasm.sh`. Público-alvo: desenvolvedores web que querem reproduzir projetos do OSATURA em um navegador ou no node, e a primeira pessoa que executar o módulo em um navegador. Documentação CC-BY-4.0; exemplos de código MPL-2.0.*

> ## STATUS: COMPILADO E TESTADO NO CI, EXECUTADO APENAS EM CHROMIUM HEADLESS
>
> **Em 24 out 2026 o módulo é compilado no GitHub Actions** (`wasm32-unknown-unknown`, build de depuração em `ci.yml`, build de release em `release.yml`) e **o teste do wrapper passa contra o módulo real sob o Node 20** (`node osatura-wasm/js/test.mjs demo.json`, incluindo sua parte com o módulo real). O build de release está anexado ao pré-lançamento em rascunho `v0.1.0-alpha.3` como `osatura-web.tar.gz` (cerca de 0,9 MB compactado, com `osatura.js` e a página de demonstração). **Ele foi carregado uma vez no Chromium 141 headless no Linux, sem GPU** (resultados no final desta página, produzidos por `osatura-wasm/js/browser-check.js`); Firefox, Safari, celulares e GPUs reais não foram testados, portanto as afirmações sobre eles nesta página não são comportamento observado, e o próprio ambiente de desenvolvimento continua sem conseguir instalar o alvo (o download de sua biblioteca padrão está bloqueado ali).
>
> O que foi testado, e até onde isso vai:
>
> | Camada | Teste | Prova | Não prova |
> |---|---|---|---|
> | As funções Rust da ABI (`arm_*`) | `cargo test -p osatura-wasm`: 8 testes unitários no **host**, chamando diretamente as funções exportadas | verificação de argumentos, códigos de erro, o protocolo do buffer de saída, máquinas de estados, chamadas de layout / rolagem / nível de detalhe, recusa de saídas grandes demais antes da alocação | o comportamento do módulo compilado |
> | O módulo compilado | CI: `cargo build --target wasm32-unknown-unknown`, depois `node osatura-wasm/js/test.mjs demo.json` com o `.wasm` real | que as dependências compilam para wasm32, que o módulo exporta o que o wrapper espera, e que o carregamento, a renderização e as chamadas da máquina de estados funcionam no motor WebAssembly do Node | navegadores, velocidade e memória do release, panics (um trap), a página de demonstração |
> | O wrapper JavaScript | o mesmo script contra um **mock em JavaScript** das exportações (três seções de mock: render, máquina de estados, layout) | as sequências de chamadas do wrapper, a passagem de strings, a tradução de erros, a cópia dos resultados | - |
> | A página de demonstração | carregada no Chromium headless **sem** o arquivo `.wasm` | os caminhos de importação do módulo são resolvidos e a página informa "wasm not found" em vez de falhar | qualquer renderização em um navegador |
> | Dependências | `cargo tree -p osatura-wasm`: sem `rhai` / `smartstring` na árvore | o motor de scripting não faz parte do módulo | - |
>
> **Riscos restantes:** (1) *Resolvido no CI:* a compilação de `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder`, `i_overlay` e de suas dependências para `wasm32-unknown-unknown`. (2) `std::time::Instant` / `SystemTime`, `process::id` e `thread::spawn` causam **panic** em `wasm32-unknown-unknown`: desde a wave 7 o core não os chama mais diretamente. Todo uso passa por `osatura_core::clock` (um tipo de carimbo, `unix_seconds`, `process_id`; o construtor de proxies de LOD recorre à thread chamadora quando não é possível criar uma thread), que neste alvo lê um relógio fornecido pelo hospedeiro (a exportação `arm_set_clock(mono_ms, unix_ms)`, alimentada pelo wrapper JavaScript com `performance.now()` e `Date.now()` antes de `load`, `svg` e `rgba`; um hospedeiro que nunca a chama obtém durações de 0, de modo que os orçamentos de tempo nunca expiram) e nunca causa panic. Um teste (`osatura-core/tests/wasm_safety.rs`) falha quando aparece um novo uso direto fora de `clock.rs`; o teste do Node no CI exercita os caminhos que ele alcança, não todos. O Rhai (que também precisaria da sua feature `no_time`) não está na compilação. (3) Módulos ligados ao sistema de arquivos e a processos compilam, mas falham em tempo de execução: `audio.rs` (abertura de arquivo), `soundtrack.rs` (ffprobe), `snapshots.rs`, `open_formats.rs` (diretórios de exportação), `agent.rs` (salvar), e o carregamento de imagens e normal maps com caminho relativo em `engine.rs`, `meshtrace.rs` e `postfx.rs`; eles retornam erros em vez de funcionar em um navegador, e nenhuma feature do cargo os controla. (4) As funções de ponto flutuante (`sin`, `cos`, `powf`, ...) vêm de uma biblioteca matemática diferente em wasm32, de modo que os pixels renderizados podem diferir ligeiramente dos nativos (o passo 6 da lista de verificação mede isso). (5) Em `wasm32-unknown-unknown` o padrão é `panic = "abort"`, `catch_unwind` não consegue interceptar nada, e um panic do Rust interrompe (trap) a instância inteira (`WebAssembly.RuntimeError: unreachable`). Veja a [lista de verificação](#lista-de-verificação-para-a-primeira-pessoa-capaz-de-executá-lo-em-um-navegador) no final.

Conteúdo: [O que é](#o-que-é) - [Compilação](#compilação) - [Modelo de memória e a ABI](#modelo-de-memória-e-a-abi) - [Referência das exportações](#referência-das-exportações) - [O wrapper JavaScript](#o-wrapper-javascript) - [A página de demonstração](#a-página-de-demonstração) - [Diferenças em relação à C ABI](#diferenças-em-relação-à-c-abi) - [Arestas conhecidas](#arestas-conhecidas) - [Lista de verificação](#lista-de-verificação-para-a-primeira-pessoa-capaz-de-executá-lo-em-um-navegador)

## O que é

`osatura-wasm` é uma `cdylib` de **funções `extern "C"` simples** (sem `wasm-bindgen`, sem código de cola gerado) sobre a fachada Rust estável ([API.md](API.md)). Compilada para `wasm32-unknown-unknown`, ela produz um módulo capaz de abrir um projeto, renderizar quadros como texto SVG ou pixels RGBA e executar máquinas de estados com layout, rolagem e nível de detalhe. Ela não consegue editar, exportar, fazer controle de versão nem executar ferramentas; não tem sistema de arquivos, rede, áudio nem threads. `osatura.js` é um módulo ES sem dependências (navegadores e node >= 18) que esconde o tratamento de ponteiros.

Licença: MPL-2.0 (crate e wrapper). O módulo depende de `osatura-core` com `default-features = false`: sem motor de scripting e sem crate de terceiros com licença MPL.

## Compilação

Você não precisa compilá-lo: o lançamento em rascunho traz `osatura-web.tar.gz` (módulo, wrapper, página de demonstração). Para compilá-lo você mesmo, é preciso uma máquina onde o alvo possa ser instalado (e, opcionalmente, `wasm-opt` do Binaryen para reduzir o tamanho). O CI o compila com `cargo build --release --target wasm32-unknown-unknown -p osatura-wasm --locked`.

```sh
rustup target add wasm32-unknown-unknown
./osatura-wasm/build-wasm.sh
```

O script (`osatura-wasm/build-wasm.sh`, um script de shell POSIX) faz exatamente isto, a partir da raiz do repositório:

1. `cargo build --target wasm32-unknown-unknown --release -p osatura-wasm`
2. copia `target/wasm32-unknown-unknown/release/osatura_wasm.wasm` para `osatura-wasm/osatura_wasm.wasm`;
3. se `wasm-opt` estiver no `PATH`, reescreve esse arquivo com `wasm-opt -Oz`;
4. imprime o tamanho do resultado e uma dica para executar o teste do node ou servir a demonstração.

**O próprio script não é o que o CI executa** (o CI chama `cargo build` diretamente e copia o módulo). Sua dica final diz `node osatura-wasm/js/test.mjs`; com um `.wasm` real presente, o teste precisa, além disso, de um caminho de projeto como primeiro argumento (`node osatura-wasm/js/test.mjs project.json`); caso contrário, ele para com "pass a project JSON path as argv[2]". O módulo de release no rascunho tem cerca de 0,9 MB como arquivo compactado; os tamanhos bruto e com `wasm-opt -Oz` não estão registrados aqui.

Depois sirva a pasta por HTTP (os navegadores não carregam módulos nem `.wasm` a partir de `file://`) e abra a página de demonstração:

```sh
cd osatura-wasm && python3 -m http.server 8000      # then open http://localhost:8000/index.html
```

Se você usar `WebAssembly.instantiateStreaming` (o wrapper faz isso quando você passa uma `Response`), o servidor deve enviar `Content-Type: application/wasm`; o caminho por URL do wrapper usa `fetch(...).arrayBuffer()` e não tem essa exigência. Se a sua página define uma Content-Security-Policy, ela deve permitir a compilação de WebAssembly (`'wasm-unsafe-eval'` em `script-src`).

Um alvo alternativo, `wasm32-wasip1`, compila com `-C panic=unwind` para que `catch_unwind` funcione (o código-fonte o menciona); ele não foi experimentado e precisa de um runtime ou shim WASI.

## Modelo de memória e a ABI

O módulo possui uma **memória linear** (exportada como `memory`). O hospedeiro nunca compartilha ponteiros com o engine, exceto segundo estas regras:

* **Entrada.** Para passar bytes (o JSON do projeto ou um nome), o hospedeiro chama `arm_alloc(len)`, que retorna um ponteiro para `len` bytes **zerados** (nulo quando `len` é 0 ou maior que 256 MiB). O hospedeiro escreve em `new Uint8Array(memory.buffer, ptr, len)`, chama a função com `(ptr, len)` e depois libera o bloco com `arm_free(ptr, len)` usando o **mesmo** ponteiro e comprimento. O engine copia o que precisa; ele não guarda nenhum ponteiro para o seu bloco.
* **Saída.** Os resultados são produzidos em um buffer **dentro do handle**: após uma chamada que retorna um comprimento `n >= 0`, `arm_out_ptr(h)` / `arm_out_len(h)` fornecem os bytes (`n` é igual ao comprimento) e, para imagens, `arm_out_width(h)` / `arm_out_height(h)` fornecem o tamanho (0 após resultados SVG ou de texto). O ponteiro é válido **até a próxima chamada que produza saída, ou até `arm_close`**. O hospedeiro deve copiar os bytes imediatamente (o wrapper faz isso: `.slice()`). Nunca mantenha uma view `Uint8Array` entre chamadas: além de ser sobrescrita, qualquer chamada pode aumentar a memória do módulo, o que desanexa `memory.buffer` e invalida todas as views dele. Releia `memory.buffer` após cada chamada.
* **Erros.** Uma função que falha retorna `-1` (ou um handle nulo) e armazena a mensagem; `arm_error_ptr()` / `arm_error_len()` fornecem seus bytes UTF-8 (sem terminador NUL). O slot é global ao módulo (que tem uma única thread), é sobrescrito pela próxima falha e **não é limpo por sucessos**, portanto leia-o somente logo após uma falha.
* **Strings de entrada.** `(ptr, len)` UTF-8, `len` entre 1 e 4096 (uma string vazia ou mais longa é um erro).
* **Handles.** `arm_open` retorna um ponteiro opaco para um handle (nulo em caso de erro). Um handle contém a animação, o player opcional e o buffer de saída. Feche-o com `arm_close` exatamente uma vez; `null` é permitido.
* **Limites.** JSON do projeto de 1 byte a 256 MiB; `arm_render_rgba` recusa antes de alocar se a imagem exceder 256 MiB (`output would be WxH pixels (N bytes), more than the limit of 268435456 bytes: lower the scale`); escala entre 0.05 e 4; quadros fora da timeline são limitados (clamped); comprimentos retornados como `i32`. A renderização precisa de memória para a imagem mais buffers de trabalho; o espaço de endereçamento de 32 bits (4 GiB, menos em alguns navegadores) é o teto real.
* **Sem threads, sem promessa de panics.** O módulo tem uma única thread. Os panics só são capturados em alvos que fazem unwind; em `wasm32-unknown-unknown` um panic interrompe (trap) a instância e o hospedeiro deve descartá-la e criar uma nova. O engine é escrito para não causar panic com entrada do usuário e verifica os tamanhos antes de alocar, mas esta camada não pode prometer sobreviver a um bug.

## Referência das exportações

37 exportações mais `memory`. `ptr` e `len` são `i32` na assinatura wasm (`usize` em Rust). `h` é um handle. O retorno `-1` = erro, salvo indicação em contrário.

| Exportação | Assinatura (visão Rust) | Resultado |
|---|---|---|
| `arm_abi_version` | `() -> u32` | 1 |
| `arm_set_clock` | `(mono_ms: f64, unix_ms: f64)` | opcional: informa ao módulo o tempo (`performance.now()`, `Date.now()`); valores inválidos são ignorados. Adicionada depois que a versão 1 da ABI foi fixada; o wrapper a chama quando presente |
| `arm_alloc` | `(len) -> *mut u8` | bloco zerado, nulo se `len` for 0 ou > 256 MiB |
| `arm_free` | `(ptr, len)` | libera um bloco de `arm_alloc` (nulo / comprimento zero são ignorados) |
| `arm_open` | `(json_ptr, json_len) -> *mut Handle` | handle ou nulo; valida o projeto |
| `arm_close` | `(h)` | libera o handle |
| `arm_frame_count`, `arm_fps`, `arm_width`, `arm_height` | `(h) -> u32 / f32 / f32 / f32` | `width` / `height` são o tamanho do palco em unidades do palco (sem arredondar); 0 e um erro para um handle nulo |
| `arm_render_svg` | `(h, frame: f32, show_bones: i32) -> i32` | texto SVG no buffer de saída; seu comprimento em bytes |
| `arm_render_rgba` | `(h, frame: f32, scale: f32) -> i32` | RGBA8 com alfa direto no buffer de saída; comprimento em bytes (`w*h*4`); tamanho via `arm_out_width/height` |
| `arm_out_ptr`, `arm_out_len` | `(h) -> *const u8 / usize` | o buffer de saída (nulo / 0 para um handle nulo) |
| `arm_out_width`, `arm_out_height` | `(h) -> u32` | tamanho da imagem do último render RGBA (0 após texto) |
| `arm_machine_count` | `(h) -> u32` | máquinas no projeto |
| `arm_machine_name` | `(h, index) -> i32` | nome no buffer de saída; seu comprimento; -1 para um índice inválido |
| `arm_player_start` | `(h, name_ptr, name_len) -> i32` | inicia (ou reinicia) a máquina; 0 |
| `arm_player_stop` | `(h)` | descarta o player |
| `arm_player_set_bool` | `(h, ptr, len, value: i32) -> i32` | 0 |
| `arm_player_set_number` | `(h, ptr, len, value: f64) -> i32` | 0 |
| `arm_player_fire` | `(h, ptr, len) -> i32` | 0 |
| `arm_player_set_value` | `(h, ptr, len, value: f64) -> i32` | 0; valor externo para bindings / entrada de rolagem |
| `arm_player_advance` | `(h, dt: f64) -> i32` | 0; `dt` em `0..=10` segundos |
| `arm_player_state` | `(h, layer_ptr, layer_len) -> i32` | nome do estado no buffer de saída; comprimento; -1 camada desconhecida |
| `arm_player_events` | `(h) -> i32` | eventos pendentes como JSON no buffer de saída, **consumidos**; comprimento |
| `arm_player_render_svg` | `(h, show_bones: i32) -> i32` | SVG da pose atual no buffer de saída |
| `arm_player_render_rgba` | `(h, scale: f32) -> i32` | RGBA da pose atual; tamanho via `arm_out_width/height` |
| `arm_player_set_canvas` | `(h, w: f32, h: f32) -> i32` | 0 |
| `arm_player_layout` | `(h) -> i32` | JSON `{"canvas":[w,h],"scrolls":[ids],"lod":level}` no buffer de saída |
| `arm_player_scroll_drag` | `(h, id_ptr, id_len, x: f32, y: f32) -> i32` | 0 |
| `arm_player_scroll_release` | `(h, id_ptr, id_len, vx: f32, vy: f32) -> i32` | 0 |
| `arm_player_scroll_info` | `(h, id_ptr, id_len) -> i32` | JSON `{"offset":[x,y],"max":[x,y],"settled":bool}` no buffer de saída |
| `arm_player_set_screen` | `(h, w_px: f32, h_px: f32, max_error_px: f32) -> i32` | 0; escolhe o nível de detalhe |
| `arm_error_ptr`, `arm_error_len` | `() -> *const u8 / usize` | texto do último erro |

Todas as chamadas `arm_player_*`, exceto `arm_player_start`, precisam de um player iniciado (caso contrário, "no player started (call arm_player_start)"). As chamadas do player são os métodos `Player` da fachada ([API.md](API.md)).

## O wrapper JavaScript

`osatura-wasm/js/osatura.js` exporta `class Osatura` (também como exportação padrão). Todo método que altera algo retorna `this`, de modo que as chamadas podem ser encadeadas. Os erros são lançados como `Error` com a mensagem do engine.

| Membro | Descrição |
|---|---|
| `static async instantiate(source, imports = {})` | retorna o objeto bruto de exportações. `source`: um `ArrayBuffer` ou array tipado (bytes), uma `Response` (usa `instantiateStreaming`; requer o MIME `application/wasm`), ou uma string de URL / `URL` (usa `fetch` + `arrayBuffer`). O módulo não tem importações, portanto `imports` permanece `{}`. |
| `static async create(source, project)` | `source` é um conjunto de exportações já instanciado (um objeto com `arm_open`) ou qualquer coisa que `instantiate` aceite; `project` é texto JSON ou um objeto simples (aplica-se `JSON.stringify`). Retorna um `Osatura`. |
| `load(project)` | carrega ou substitui o projeto; lança uma exceção em caso de falha e então **mantém utilizável o projeto carregado anteriormente**. Preenche `frames`, `fps`, `width`, `height`. |
| `frames`, `fps`, `width`, `height` | dados do último `load` (`width` / `height` em unidades do palco, floats) |
| `svg(n, showBones = false)` | `string`: documento SVG do quadro `n` (quadros fracionários permitidos, limitados) |
| `rgba(n, scale = 1)` | `{ width, height, data }`: `data` é um `Uint8ClampedArray` (RGBA com alfa direto, uma cópia que pertence a você) |
| `frame(n, { kind = 'image', scale = 1, bones = false } = {})` | `kind: 'svg'` dá `{ svg }`; caso contrário `{ image: ImageData }` em um navegador, ou `{ width, height, data }` onde `ImageData` não existe (node) |
| `machines()` | `string[]` nomes das máquinas |
| `startPlayer(name)`, `stopPlayer()` | inicia (reinicia) / descarta o player; as entradas voltam aos valores padrão |
| `setBool(name, v)`, `setNumber(name, v)`, `fire(name)`, `setValue(path, v)` | entradas e valores externos; um nome vazio lança `empty string`, um nome com mais de 4096 bytes também lança |
| `advance(dt)` | avança a máquina e as regiões de rolagem em `dt` segundos (`0..=10`) |
| `state(layer)` | `string` estado atual de uma camada de máquina |
| `events()` | `{ time, layer, kind: 'entered' \| 'exited', state }[]`, **consumidos** pela chamada |
| `setCanvas(w, h)` | layout responsivo para um tamanho de canvas em unidades do palco |
| `layout()` | `{ canvas: [w, h], scrolls: string[], lod: number }` |
| `scrollDrag(id, x, y)`, `scrollRelease(id, vx = 0, vy = 0)` | arrastar o ponteiro / velocidade de soltura em px e px/s |
| `scrollInfo(id)` | `{ offset: [x, y], max: [x, y], settled: boolean }` |
| `setScreen(widthPx, heightPx, maxErrorPx = 0.5)` | escolhe o nível de detalhe para um tamanho de tela |
| `playerSvg(showBones = false)`, `playerRgba(scale = 1)` | a pose atual do player |
| `play(canvas, { scale = 1, loop = true, onFrame } = {})` | desenha a timeline em um canvas 2D com `requestAnimationFrame` (recorre a `setTimeout` fora de um navegador); retorna uma função de parada |
| `playMachine(canvas, { scale = 1, onTick } = {})` | avança a máquina iniciada pelo tempo real decorrido (no máximo 0,25 s por tick) e a desenha; `onTick(dt)` é executado antes de cada avanço, portanto defina as entradas ali; retorna uma função de parada |
| `stop()` | interrompe `play` / `playMachine` |
| `close()` | interrompe e libera o handle; chame quando terminar |

O wrapper mantém as consultas a `x.memory.buffer` dentro de cada chamada e copia os resultados com `.slice()`, de modo que o crescimento da memória não pode deixá-lo com uma view obsoleta. Cada objeto `Osatura` possui um handle; crie vários objetos (eles podem compartilhar um mesmo `exports`) para manter vários projetos. Não compartilhe um mesmo objeto entre tarefas assíncronas concorrentes que intercalem chamadas e leituras de resultados: o resultado de cada chamada fica no buffer de saída do handle até a próxima chamada.

### Exemplo (sequência de chamadas)

Esta é a sequência exata de chamadas que o teste do node exercita. **Ela foi executada contra o mock em JavaScript** (o substituto em `test.mjs` com respostas prontas: máquina `hero`, camada `body`, entrada bool `go`, região de rolagem `list`), não contra um módulo real.

```js
import { Osatura } from './js/osatura.js';

// 1. Instantiate the module once (a URL in the browser; in node pass the bytes: readFileSync('osatura_wasm.wasm')).
const exports = await Osatura.instantiate('./osatura_wasm.wasm');

// 2. Load a project (JSON text or a plain object). Throws Error(<engine message>) when it is invalid.
const anim = await Osatura.create(exports, projectJsonText);
console.log(`${anim.width}x${anim.height} stage units, ${anim.frames} frames @ ${anim.fps} fps`);

// 3. Render. Frames may be fractional and are clamped to the timeline.
const svg = anim.svg(10, /* showBones */ false);        // string
const { width, height, data } = anim.rgba(10, 0.5);     // data: Uint8ClampedArray, straight-alpha RGBA, a copy you own

// 4. Interactive playback with a state machine.
console.log(anim.machines());                           // ['hero']
anim.startPlayer('hero').setBool('go', true).advance(0.1);
console.log(anim.state('body'), anim.events());         // 'b', [{time, layer, kind, state}, ...] (consumed)
const poseSvg = anim.playerSvg();
const pose = anim.playerRgba(0.5);

// 5. Layout and scrolling.
anim.setCanvas(100, 120);
anim.scrollDrag('list', 0, 50).scrollRelease('list', 0, 100);
console.log(anim.scrollInfo('list'));                   // { offset: [0, 50], max: [0, 80], settled: false }
anim.setScreen(100, 120, 4);                            // pick a level of detail within 4 px
console.log(anim.layout());                             // { canvas: [100, 120], scrolls: ['list'], lod: 2 }

// 6. Always release the handle (it frees the project inside the module's memory).
anim.close();
```

Em uma página de navegador:

```html
<canvas id="c"></canvas>
<script type="module">
  import { Osatura } from './js/osatura.js';
  const x = await Osatura.instantiate('./osatura_wasm.wasm');
  const text = await (await fetch('./project.json')).text();
  const anim = await Osatura.create(x, text);

  // Timeline playback: requestAnimationFrame loop, loops at the end, returns a stop function.
  const stop = anim.play(document.getElementById('c'), { scale: 1, loop: true });

  // Or run a state machine in real time; onTick(dt) runs before each advance, so set inputs there.
  // anim.startPlayer('hero');
  // const stopMachine = anim.playMachine(canvas, { onTick: () => anim.setBool('go', keyIsDown) });
</script>
```

## A página de demonstração

`osatura-wasm/index.html` é uma página mínima: ela importa o wrapper, instancia `./osatura_wasm.wasm` e oferece um campo de arquivo (projeto `.json`) e um botão Stop. Escolher um arquivo cria um `Osatura`, mostra `WIDTHxHEIGHT, N frames @ FPS fps` e inicia `play(canvas)`; os erros do engine são exibidos no texto ao lado dos botões. Se o módulo não puder ser obtido ou compilado, ela mostra `wasm not found (not built yet): <reason>`. Testada apenas nesse estado "não compilado", no Chromium headless: a página carrega, a importação do módulo é resolvida, a mensagem aparece (com a página 404 do servidor como "módulo", o motivo diz `expected magic word 00 61 73 6d, found 3c 21 44 4f`, veja abaixo). Tudo além disso não foi verificado.

## Diferenças em relação à C ABI

| | C ABI ([C_ABI.md](C_ABI.md)) | WebAssembly |
|---|---|---|
| Strings de entrada | terminadas em NUL | `(ptr, len)` copiadas com `arm_alloc` |
| Resultados de saída | buffers fornecidos pelo chamador | o buffer de saída do handle (`arm_out_ptr/len`), válido até a próxima chamada com saída |
| Consulta de tamanho | um buffer `NULL` retorna o tamanho | desnecessária: a chamada retorna o comprimento e os dados já estão no buffer de saída |
| Texto de erro | string NUL de `osatura_last_error()`, por thread | bytes de `arm_error_ptr/len`, global |
| Matrizes dos ossos | `osatura_bone_world` | não exportada |
| SVG | não exportado | `arm_render_svg`, `arm_player_render_svg` |
| Eventos | padrão de consulta de tamanho, mantidos se o buffer for pequeno demais | sempre consumidos |
| Tamanho do palco | inteiros arredondados | floats |

## Arestas conhecidas

Encontradas ao ler o código; nenhuma delas foi vista falhando:

* `Osatura.instantiate(url)` verifica `response.ok` e os erros de fetch e informa a URL e o status HTTP (testes com fetch simulado em `osatura-wasm/js/test.mjs`; executados sob node, e no CI contra o módulo real).
* `frame()` com `kind: 'svg'` retorna `{ svg }`, não uma string como `svg()`.
* A dica impressa por `build-wasm.sh` agora inclui o argumento `<project.json>` de que `test.mjs` precisa.
* `play()` desenha convertendo cada quadro em RGBA na CPU e chamando `putImageData`; não há áudio, nem pausa, nem busca.
* O wrapper não exporta declarações TypeScript.

## Lista de verificação para a primeira pessoa capaz de executá-lo em um navegador

Os passos 1 e 5 foram feitos pelo CI em 24 out 2026. Os passos 3, 4, 6, 8, 9, 11 e 13 foram feitos no mesmo dia em **Chromium 141 headless (Linux, sem GPU)** com `osatura-wasm/js/browser-check.js`; os números estão na tabela abaixo. **Ainda em aberto:** Firefox e Safari (passos 11 e 12 ali), o passo 12 de CSP e content-type, a profundidade de pilha do passo 10 e os passos 7 e 14 do player. Faça estes passos em ordem; pare na primeira falha e registre a mensagem exata. Os itens marcados (risco) correspondem aos riscos acima. Quando tiver resultados, atualize [LIMITATIONS.md](LIMITATIONS.md) (seção 1), a linha R-090 da seção 9 de ROADMAP.md e a caixa de status desta página.

1. **Instale o alvo e compile.** `rustup target add wasm32-unknown-unknown && ./osatura-wasm/build-wasm.sh`. (risco) Espere ver se `zip`, `tiny-skia`, `png`, `gif`, `jpeg-decoder` e `i_overlay` compilam. Registre o compilador e `rustc --version`.
2. **Sem motor de scripting.** `cargo tree --target wasm32-unknown-unknown -p osatura-wasm | grep -ci 'rhai\|smartstring'` deve imprimir 0.
3. **Examine as exportações.** No node: `WebAssembly.Module.exports(new WebAssembly.Module(fs.readFileSync('osatura-wasm/osatura_wasm.wasm')))`. Espere `memory` e as 37 funções da [tabela de exportações](#referência-das-exportações), e **nenhuma importação** (`WebAssembly.Module.imports(...)` vazio). Se `memory` estiver ausente ou aparecerem importações (por exemplo `env.*` de uma feature da std), o wrapper precisa ser adaptado.
4. **Tamanho.** Registre o tamanho antes e depois de `wasm-opt -Oz`, e os tamanhos com gzip / brotli. Medido uma vez na execução no Chromium: veja a tabela de resultados no final desta página.
5. **Execute o teste do node com um projeto real.** `osatura demo demo.json && node osatura-wasm/js/test.mjs demo.json` deve imprimir `real wasm tests: ok` após as três seções de mock.
6. **Compare com o renderizador nativo.** Para os quadros 0, 12 e 47 da demonstração, nas escalas 0.5 e 1, compare `Osatura.rgba` com `osatura render demo.json N out.png --scale S` decodificado para RGBA. (risco) Espere pequenas diferenças nas bordas com anti-aliasing (biblioteca matemática diferente); relate a diferença absoluta máxima e média e a fração de pixels acima de 24/255. Uma diferença grande aponta para um bug real.
7. **Exercite o player.** Construa o projeto do exemplo da C ABI (demo + `take-record` + `clip add` + `machine add`, veja [C_ABI.md](C_ABI.md)), acione `startPlayer`, `setBool`, `advance`, `state`, `events`, `playerRgba`; e o projeto com um contêiner e uma região de rolagem para `setCanvas`, `scrollDrag`, `scrollRelease`, `scrollInfo`, `setScreen` (o último constrói níveis de detalhe: meça o tempo, ele é lento sem otimização e não teve o tempo medido em wasm).
8. **Caminhos à prova de falhas.** Forneça um projeto inválido, uma string vazia, um arquivo aleatório de 5 MB, um projeto com um caminho de imagem `source.file` e um projeto com uma trilha sonora. Todo caso deve produzir um `Error` lançado com uma mensagem, não um trap. (risco) Um trap (`RuntimeError: unreachable`) em qualquer um deles é um bug a relatar; os candidatos mais prováveis são `Instant` / `SystemTime` ou chamadas ao sistema de arquivos alcançadas por um caminho inesperado.
9. **Render grande.** `rgba(0, 4)` na demonstração (2560 x 1920) e um palco de 12000 x 12000 na escala 1 (deve ser recusado com a mensagem "more than the limit", sem esgotar a memória). Observe o crescimento da memória e confirme que o wrapper continua funcionando depois que a memória cresceu (as views são relidas a cada chamada).
10. **Profundidade da pilha.** `wasm32-unknown-unknown` dá à thread principal uma pilha padrão pequena (1 MiB nas toolchains atuais). Abra um projeto com grupos profundamente aninhados e uma longa cadeia de expressões (veja `osatura-cli/tests/robust.rs` para entradas hostis) e confirme que não ocorre trap por estouro de pilha.
11. **Execução no navegador.** Sirva a pasta, abra `index.html` no Chromium e no Firefox, escolha um projeto, confirme a reprodução, redimensione, feche e reabra vários projetos (verificação de vazamento: a memória não deve crescer sem limite; `close()` libera o handle).
12. **`instantiateStreaming` e CSP.** Sirva com e sem `Content-Type: application/wasm`, e com uma CSP que omita `'wasm-unsafe-eval'`; documente o que acontece.
13. **Desempenho.** Meça o tempo de `rgba(n, 1)` para a demonstração de 640 x 480 (nativo em release: cerca de 14 ms por quadro) e para um projeto pesado; registre os quadros por segundo de `play()` no navegador.
14. **Opcional: panics.** Decida se o comportamento de trap em caso de panic é aceitável para o seu uso. Se não, experimente o caminho `wasm32-wasip1` + `-C panic=unwind` mencionado no código-fonte.

### Resultados da execução no Chromium (24 out 2026)

| Passo | Resultado |
|---|---|
| 4. Tamanho | módulo release de 2.802.534 bytes; 853.804 em gzip; 615.267 em brotli (não passou pelo `wasm-opt`) |
| 6. Comparação com o nativo | quadro 12 da demonstração, 640x480: **0 bytes diferentes** de 1.228.800 (diferença absoluta média e máxima 0) |
| 8. Falha segura | vazio, lixo, JSON de formato errado, 5 MB aleatórios e "sem quadros" lançam cada um um `Error` com mensagem; sem trap |
| 9. Renderização grande | escala 4 (2560x1920) funciona; um palco de 12000x12000 é recusado com a mensagem de limite e o módulo continua funcionando depois |
| 11. Vazamento | 60 ciclos de abrir e fechar: memória linear de 54.591.488 bytes antes e depois |
| 13. Velocidade | `rgba` da demonstração de 640x480: mediana 72,5 ms, p95 96,5 ms, máximo 125,4 ms (cerca de 14 fps; nativo em release cerca de 14 ms) |

Execute-o de novo com `node osatura-wasm/js/browser-check.js <pasta osatura-web descompactada> <project.json>` (precisa do Playwright). A página não gerou erros durante a execução.
