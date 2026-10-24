# API Rust do OSATURA e política de estabilidade

*Versão da fachada 1.4.0 (`osatura_core::api::API_VERSION`). Público-alvo: desenvolvedores Rust que incorporam o engine e mantenedores que alteram a fachada. Licença da documentação CC-BY-4.0; exemplos de código MPL-2.0.*

Esta é a referência da **API Rust suportada** de `osatura-core`: o módulo de fachada `osatura_core::api` e sua importação de uma linha `osatura_core::prelude`. Ela lista todos os itens públicos, as regras que mantêm a API estável e exemplos testados. Todo exemplo deste arquivo é compilado e executado com as fontes atuais antes de o arquivo ser publicado (veja [Como os exemplos são verificados](#como-os-exemplos-são-verificados)).

Conteúdo: [O que está coberto](#o-que-está-coberto) - [Como obter](#como-obter-o-crate) - [Política de estabilidade](#política-de-estabilidade) - [Regras de semver](#regras-de-semver) - [Feature flags](#feature-flags) - [Thread-safety](#thread-safety) - [Determinismo](#determinismo) - [Limites](#limites-e-intervalos-de-argumentos) - [Erros](#erros) - [Referência: `Animation`](#referência-animation) - [Referência: `Player`](#referência-player) - [Referência: funções livres](#referência-funções-livres-ferramentas-controle-de-versão-ci) - [Referência: tipos](#referência-tipos) - [Exemplos](#exemplos) - [Outras linguagens](#outras-linguagens) - [Apêndice: JSON de máquina e de binding](#apêndice-json-de-máquina-e-json-de-binding)

## O que está coberto

Somente `osatura_core::api` e `osatura_core::prelude` são a API Rust pública suportada. A fachada consiste em

* `Animation`: um projeto validado mais seu histórico de desfazer (carregar, consultar, renderizar, exportar, editar, ferramentas);
* `Player`: uma máquina de estados em execução com layout, rolagem e nível de detalhe (1.1, 1.2);
* funções livres para controle de versão e CI de animação (1.2 leitura, 1.3 escrita) e para tracing de bitmaps;
* tipos de valor simples (`AnimationInfo`, `RenderSettings`, `Image`, `LodLevel`, `SizeFit`, `MocapReport`, `FuzzSummary`, `VcsCommit`, `VcsConflict`, `CiOutcome`, `CiInit`, ...) e os enums `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome`, `VcsChoice`;
* o tipo de erro `ApiError` (implementa `std::error::Error` e `Display`).

A C ABI (`osatura-capi`, header `osatura.h`) e o módulo WebAssembly (`osatura-wasm`) são escritos apenas sobre esta fachada e têm suas próprias versões: [C_ABI.md](C_ABI.md), [WASM.md](WASM.md).

### O que NÃO está coberto

Todos os outros módulos de `osatura-core` (`engine`, `project`, `edit`, `svg`, `raster`, `physics`, `vcs`, `ci`, ...) e as reexportações da raiz (`Engine`, `Project`, ...). Eles são `pub` e `#[doc(hidden)]` somente porque a ferramenta de linha de comando, o studio, os testes da C ABI e os testes unitários os usam; podem mudar ou desaparecer em qualquer versão, e seus textos `Err(String)` também não são um contrato. Se você precisar de algo que só é acessível ali, peça uma adição à fachada (uma versão minor) em vez de depender de elementos internos.

O formato JSON do projeto tem seu próprio campo de versão `format` (atualmente 1); um projeto gravado por uma compilação mais nova é recusado com `InvalidJson`. Veja [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

## Como obter o crate

O OSATURA não é publicado em nenhum registry; dependa dele por caminho (ou git) a partir do seu workspace:

```toml
[dependencies]
osatura-core = { path = "../osatura/osatura-core", default-features = false }
```

`default-features = false` remove o motor de scripting Rhai (veja [Feature flags](#feature-flags)); a fachada não precisa dele. Depois, `use osatura_core::prelude::*;`. A versão do crate (`0.1.0`) e a versão da fachada (`API_VERSION`) são independentes: a fachada segue o semver, a versão do crate ainda não promete nada. Nenhuma versão mínima suportada do Rust é declarada; o workspace é desenvolvido e testado com Rust 1.97 (edition 2021).

## Política de estabilidade

1. **O arquivo de superfície.** `osatura-core/src/api_surface.txt` lista todos os itens públicos de `src/api/` (tipos, campos, variantes, constantes e assinaturas de métodos, um por linha). O teste unitário `api::tests::api_surface_snapshot` reconstrói essa lista a partir de `api/` e falha quando ela difere do arquivo versionado. Toda mudança na superfície é, portanto, uma edição deliberada e revisada dos dois arquivos.
2. **O teste do prelude.** `osatura-core/tests/prelude_surface.rs` é compilado como um crate *externo*. Ele importa os nomes da fachada apenas por `osatura_core::prelude` e chama um subconjunto representativo (carregar, renderizar, editar, exportar, `Player`, as ferramentas, os wrappers de controle de versão e de CI, tratamento de entrada inválida). Se um nome que ele usa deixar de ser reexportado pelo prelude, ou uma assinatura mudar de um modo que essas chamadas percebam, esse teste deixa de compilar. Ele também afirma `API_VERSION == "1.4.0"`, de modo que um aumento de versão também é uma edição consciente ali. Ele não enumera todos os nomes do prelude, portanto adicionar um nome à lista de reexportação continua sendo um passo manual.
3. **Limites da verificação.** O snapshot é uma varredura de texto das assinaturas de uma linha em `api/`, não um diff em nível de compilador como o `cargo-public-api`: uma mudança semântica que mantém o texto da assinatura (por exemplo, um erro diferente para a mesma entrada) não é detectada, e uma assinatura de várias linhas passaria despercebida (o arquivo mantém as assinaturas em uma linha por esse motivo). A lista de reexportação do prelude (`lib.rs`) é verificada pelo teste externo, não pelo snapshot.
4. **Oculto não significa suportado.** Os módulos `#[doc(hidden)] pub` existem para consumidores dentro do workspace. Eles estão fora da promessa, mesmo que compilem para você.
5. **Como alterar a fachada** (mantenedores; também em [DEVELOPMENT.md](DEVELOPMENT.md)):
   * edite `api/` (assinaturas de uma linha, comentário de documentação, um doc-test para novas funções livres, testes unitários);
   * adicione os novos nomes públicos à lista de reexportação do `prelude` em `lib.rs` e a `tests/prelude_surface.rs`;
   * aumente `API_VERSION` (e `API_VERSION_MAJOR` se for uma quebra) conforme as regras abaixo, e a versão afirmada em `prelude_surface.rs`;
   * regenere o arquivo de superfície de forma deliberada: `OSATURA_BLESS=1 cargo test -p osatura-core api::` e revise o diff de `api_surface.txt`;
   * atualize este arquivo, e `osatura-capi` / `osatura-wasm` somente se o novo item deve atravessar essas fronteiras (a versão de ABI deles se move separadamente, as adições a mantêm).

## Regras de semver

`API_VERSION` é uma versão semântica apenas da fachada.

| Mudança | Versão |
|---|---|
| Correção de bug, sem mudança de assinatura, sem novo contrato de comportamento | patch (1.4.x) |
| Nova função, método, tipo, variante de enum, campo de struct, constante | minor (1.x.0) |
| Remover ou renomear qualquer coisa; alterar uma assinatura, um intervalo documentado de argumento de modo a rejeitar entradas antes válidas, ou o significado de um resultado documentado | major (2.0.0) |

Consequências para quem chama:

* **Os enums e structs públicos da fachada são `#[non_exhaustive]`.** Adicione um braço curinga ao fazer `match` em `ApiError`, `Interpolation`, `ExportFormat`, `FitFormat`, `CiMode`, `VcsMergeOutcome` ou `VcsChoice`; uma nova variante é uma versão minor. Você não pode construir os structs com um literal fora do crate (e `Image`, `SizeFit` e os structs de relatório são resultados que você apenas lê). As configurações são construídas com construtores: `RenderSettings::new(scale).with_bones(true)`, `MocapSettings::new().with_smooth(2)`.
* **Strings dentro de resultados são dados, não contrato.** `VcsConflict::kind` é hoje um entre `"BothChanged"`, `"DeleteEdit"`, `"KeyRetimed"`, `"OrderConflict"`, `"ArrayConflict"` e *pode ganhar valores em versões minor*. Os relatórios JSON (`FuzzSummary::json`, `CiOutcome::json`) carregam seus próprios nomes de schema (`osatura-fuzz-report` v1, `osatura-ci/1`). Os textos das mensagens de erro são para pessoas.
* **Estabilidade comportamental.** A renderização e a avaliação são determinísticas (veja abaixo), mas a saída em pixels pode mudar em uma versão minor quando um bug do rasterizador ou de um efeito é corrigido; os goldens gravados pelo CI de animação são um lembrete disso. Não calcule hash de pixels renderizados entre versões do OSATURA ou entre máquinas sem uma tolerância.

Histórico de versões da fachada: 1.0 (carregar, consultar, renderizar, exportar, editar, desfazer) - 1.1 (`Player`, máquinas de estados, data bindings) - 1.2 (layout, rolagem e nível de detalhe em `Player`; `lod_levels`, `fit_to_size`, `retarget_bvh`, `fuzz`, `from_image`, `trace_image_svg`, `vcs_log`, `ci_run`) - 1.3 (operações de escrita de controle de versão e de CI, `DirtyTree` / `MergeInProgress` / `Conflicts`, `CiOutcome::updated` / `blessed`) - 1.4 (criação de camadas e de ossos, `bake_physics`, exportação APNG e EXR, `vcs_branches` / `vcs_branch_delete` / `vcs_status` / `vcs_diff` / `vcs_show` / `vcs_gc` / `vcs_fsck`).

## Feature flags

O `osatura-core` tem uma única feature do Cargo:

| Feature | Padrão | Efeito |
|---|---|---|
| `scripting` | ativada | Compila o módulo de scripting em sandbox baseado em Rhai (`script.rs`), usado apenas pelo comando `script` da linha de comando. Traz `rhai` e, por meio dele, `smartstring` (MPL-2.0+, o único crate de terceiros com copyleft fraco na árvore). |

A fachada **não** tem nenhuma funcionalidade atrás de uma feature: todo item documentado aqui existe com `default-features = false`. `osatura-capi`, `osatura-wasm` e `osatura-gpu` dependem do core sem a feature, portanto não contêm nenhum crate de terceiros MPL; `osatura-cli --no-default-features` faz o mesmo e então informa que o comando `script` está indisponível. Os back ends de renderização não são features: o rasterizador de CPU é sempre usado pela fachada; o renderizador de GPU opcional é um crate separado (`osatura-gpu`) e uma feature da CLI (`gpu`) e não é acessível por esta API.

## Thread-safety

* `Animation`, `Player`, `ApiError` e `Image` são `Send + Sync` (verificado em tempo de compilação para este documento). A fachada não grava estado global; ela apenas lê duas configurações de assets de todo o processo (o diretório de assets padrão e a chave de assets externos) que nada na fachada altera.
* Os métodos que recebem `&self` (`info`, `bone_world`, `render_*`, `export`, `lod_levels`, `fit_to_size`, `fuzz`, `player`, `to_json`, ...) podem ser executados simultaneamente em várias threads sobre a mesma `Animation`; os métodos que recebem `&mut self` (`set_key`, `delete_key`, `undo`, `redo`, `retarget_bvh`, toda entrada de `Player` e `advance`) precisam de acesso exclusivo, como de costume em Rust. O exemplo `render_in_parallel` abaixo renderiza cada quadro em sua própria thread e verifica que as imagens são iguais às da execução em uma única thread.
* Não há bloqueio interno nem thread em segundo plano. Um `Player` trabalha sobre sua própria cópia do projeto, de modo que editar depois a `Animation` nunca afeta um player em execução.
* `Animation::from_json` resolve arquivos de imagem relativos em relação ao **diretório atual** do processo e recusa caminhos absolutos e `..` (veja [SECURITY.md](SECURITY.md)); incorpore as imagens como data URIs, ou defina o diretório de trabalho antes de carregar, se um projeto referenciar arquivos.
* As funções de controle de versão e de CI trabalham sobre arquivos: não são seguras contra outro processo que edite o mesmo projeto no mesmo momento (a fachada verifica previamente se há árvore suja e merge pendente, mas não há lock).

## Determinismo

Mesmo projeto, mesmos argumentos, mesma compilação do OSATURA e mesma máquina: mesmos bytes. Isso vale para `render_rgba` / `render_png` / `render_svg`, `bone_world`, toda exportação, `lod_levels` e `fuzz` (para uma dada seed), `fit_to_size` e para um `Player` (as mesmas entradas e as mesmas chamadas a `advance` dão a mesma pose; `advance` divide o tempo em pedaços de no máximo 1/60 s, portanto a forma como você fragmenta o tempo só importa abaixo dessa resolução). As escolhas aleatórias dentro do engine (ruído, partículas, fuzzing, clusters do tracing) usam geradores com seed, nunca o relógio. O que *não* é prometido: pixels de borda com anti-aliasing idênticos entre sistemas operacionais, CPUs, compiladores ou versões de `libm` diferentes, entre alvos de 32 e 64 bits, ou entre versões do OSATURA. Os goldens exatos em nível de byte foram verificados apenas entre processos em uma única máquina Linux x86-64 (veja [LIMITATIONS.md](LIMITATIONS.md)); use tolerâncias (o CI de animação usa) para qualquer coisa entre máquinas.

## Limites e intervalos de argumentos

| Item | Regra |
|---|---|
| `MAX_JSON_BYTES` | 256 MiB; um texto maior é `InvalidJson` |
| `MAX_RENDER_SIDE` | 16384 pixels de saída por lado (`render_*` e `Player::render_*`) |
| Escala de render / GIF | finita e dentro de `0.05..=4`, caso contrário `InvalidArgument` |
| Posições de quadro | devem ser finitas; valores fora da timeline são **limitados** (clamped) a `0..=frames-1` (sem erro). Quadros fracionários são aceitos. |
| Chaves (`set_key`, `delete_key`) | `frame` e `value` finitos; `prop` é um nome de propriedade em snake_case do JSON do projeto (`"bone_angle"`, `"layer_opacity"`, ...) |
| Projeto | `fps` em `(0, 1000]`, `frames` em `1..=200000`, palco de até 32768 unidades por lado, no máximo 4096 ossos, números além de +-1e7 recusados no carregamento; outros limites por recurso são validados no carregamento e voltam como `InvalidProject` |
| `Player::advance(dt)` | `dt` finito, `0..=10` segundos |
| `Player::set_canvas_size` | ambos os lados finitos, `> 0`, no máximo 32768 |
| `Player::set_screen_size` | largura e altura finitas e `> 0`; `max_error_px` finito e `>= 0` |
| `Animation::lod_levels` | `tolerance_px > 0`; `scale` em `(0, 64]`; `levels` em `1..=12` |
| `Animation::fit_to_size` | `max_bytes` em `1000..=1_000_000_000` |
| `Animation::fuzz` | `samples` em `1..=100000` |
| `trace_image_svg`, `Animation::from_image` | `colors` em `2..=64`; bytes de PNG ou JPEG (aplicam-se limites de tamanho) |
| `vcs_log` | `limit` em `1..=10000` |

## Erros

Toda função que pode falhar retorna `Result<_, ApiError>`. Uma entrada inválida é um valor de erro e **nunca causa panic**; um panic que ocorra dentro do engine é capturado na fronteira da fachada e retornado como `ApiError::Internal` (por favor, relate-o como bug). `ApiError` implementa `Display` e `std::error::Error`, de modo que `?` funciona com `Box<dyn Error>` e com tipos no estilo `anyhow`.

| Variante | Significado | Origens típicas |
|---|---|---|
| `InvalidJson(String)` | não é JSON, não é um projeto, formato errado, grande demais (> `MAX_JSON_BYTES`), ou um `format` mais novo que esta compilação; também um valor JSON inválido passado a `vcs_resolve` | `Animation::from_json`, `validate_json`, `vcs_resolve` |
| `InvalidProject(Vec<String>)` | foi interpretado, mas a validação falhou; uma mensagem por problema (osso desconhecido, intervalo inválido, ciclo, ...) | `from_json`, `from_image`, reconstruções do engine em `Player` |
| `InvalidArgument(String)` | um argumento do chamador foi rejeitado: número não finito, valor fora do intervalo, propriedade desconhecida, id de máquina / entrada / rolagem desconhecido, BVH ou imagem inutilizável, nome de branch inválido, nada para commitar, nenhum repositório | quase todas as funções |
| `EditRejected(Vec<String>)` | uma edição foi recusada (por exemplo, um canal ou camada bloqueado, ou um resultado inválido); o projeto permanece inalterado | `set_key`, `delete_key`, `retarget_bvh` |
| `Export(String)` | um encoder falhou ou o formato não é possível para este projeto | `export`, `render_png`, `fit_to_size` |
| `Internal(String)` | um panic capturado | qualquer uma |
| `DirtyTree(String)` | (1.3) o arquivo do projeto tem alterações não commitadas que a operação sobrescreveria | `vcs_checkout` (a menos que `force`), `vcs_merge`, `vcs_cherry_pick`, `vcs_revert` |
| `MergeInProgress(String)` | (1.3) um merge, cherry-pick ou revert está inacabado; resolva e faça commit, ou use `vcs_abort` | as mesmas funções |
| `Conflicts(Vec<String>)` | (1.3) `vcs_commit` enquanto ainda restam conflitos; um ponteiro por conflito | `vcs_commit` |

Uma verificação do CI de animação que falha **não** é um erro: `ci_run` retorna `Ok(CiOutcome)` com `exit_code` 1 (uma verificação falhou) ou 2 (uma verificação não pôde ser avaliada); somente um manifesto inutilizável é `Err`. Os conflitos de merge também são um resultado (`VcsMergeOutcome::Conflicted`), não um erro.

## Referência: `Animation`

Um projeto carregado e validado mais seu histórico de desfazer. Crie-o com `from_json`, `demo` ou `from_image`. Todos os métodos que leem o projeto recebem `&self`.

| Método | Retorna | Observações |
|---|---|---|
| `Animation::from_json(&str)` | `Result<Animation, ApiError>` | interpreta + valida. `InvalidJson` / `InvalidProject`. Campos JSON desconhecidos são ignorados silenciosamente aqui (o comando `check` da CLI os relata). |
| `Animation::validate_json(&str)` | `Result<(), ApiError>` | as mesmas verificações, sem guardar nada |
| `Animation::demo()` | `Animation` | pequeno rig embutido (6 ossos, 48 quadros a 24 fps, 640 x 480); útil para testes e primeiros passos |
| `Animation::from_image(&[u8], colors, name)` | `Result<Animation, ApiError>` | faz o tracing de bytes PNG/JPEG em camadas vetoriais preenchidas, sobre um palco do tamanho da imagem |
| `to_json(&self)` | `String` | o projeto atual como JSON formatado (faz round trip por `from_json`) |
| `info(&self)` | `AnimationInfo` | `name`, `width`, `height` (unidades do palco), `fps`, `frames` |
| `frame_count(&self)`, `fps(&self)` | `u32`, `f32` | |
| `bone_names(&self)` | `Vec<String>` | ordem do rig = ordem de `bone_world` |
| `layer_names(&self)` | `Vec<String>` | caminhos de camada `group/child`, em profundidade |
| `bone_world(&self, frame)` | `Result<Vec<[f32; 6]>, ApiError>` | matriz de mundo por osso `[a b c d tx ty]`; um ponto local do osso `(x, y)` é mapeado para `(a*x + c*y + tx, b*x + d*y + ty)`. Quadro limitado (clamped). Avalia o rig inteiro (IK, constraints, springs, expressões). |
| `machine_names(&self)` | `Vec<String>` | máquinas de estados de runtime armazenadas no projeto |
| `player(&self, machine)` | `Result<Player, ApiError>` | inicia a máquina indicada no tempo 0 sobre uma cópia do projeto |
| `render_rgba(&self, frame, &RenderSettings)` | `Result<Image, ApiError>` | RGBA8 com alfa direto (straight), em ordem de linhas, linha superior primeiro |
| `render_png(&self, frame, &RenderSettings)` | `Result<Vec<u8>, ApiError>` | um arquivo PNG na memória |
| `render_svg(&self, frame, show_bones: bool)` | `Result<String, ApiError>` | um documento SVG (a saída vetorial não tem escala) |
| `export(&self, ExportFormat, scale)` | `Result<Vec<u8>, ApiError>` | `Gif` (em loop, timeline inteira; `scale` dentro de `0.05..=4`), `Glb` (glTF 2.0 binário), `Lottie` (bytes JSON), `Html` (uma página de player autônoma); `scale` é ignorada pelos outros três. O que cada formato mantém ou descarta: [EXPORT_FORMATS.md](EXPORT_FORMATS.md). |
| `set_key(&mut self, target, prop, frame, value, Interpolation)` | `Result<(), ApiError>` | define ou adiciona uma chave em `target.prop`; validado, desfazível, respeita os bloqueios de canal e de camada (`EditRejected`) |
| `delete_key(&mut self, target, prop, frame)` | `Result<bool, ApiError>` | `true` se uma chave foi removida |
| `add_rect_layer(&mut self, name, x, y, w, h, fill, parent)`, `add_ellipse_layer(&mut self, name, cx, cy, rx, ry, fill, parent)` | `Result<(), ApiError>` | (1.4) acrescenta uma camada de forma preenchida (ao nível superior, ou ao grupo de nível superior nomeado por `parent`); nome único sem `/`, tamanhos finitos e positivos, uma cor válida; desfazível |
| `add_group_layer(&mut self, name)` | `Result<(), ApiError>` | (1.4) acrescenta uma camada de grupo vazia |
| `add_bone(&mut self, name, parent, length, x, y, angle)` | `Result<(), ApiError>` | (1.4) acrescenta um osso (`parent` deve existir, ângulo em graus); desfazível |
| `bake_physics(&mut self, chain, preset, overwrite)` | `Result<PhysicsBake, ApiError>` | (1.4) simula uma cadeia de ossos (raiz primeiro) com `hair`, `tail`, `rope`, `antenna` ou `cape` e armazena canais de ângulo como um único passo de desfazer; com `overwrite = false`, os ossos com chaves manuais são mantidos e listados em `skipped` |
| `render_exr(&self, frame, &RenderSettings, float32)` | `Result<Vec<u8>, ApiError>` | (1.4) arquivo OpenEXR (linear, pré-multiplicado; HALF ou FLOAT) a partir do compositor em ponto flutuante |
| `undo(&mut self)`, `redo(&mut self)` | `bool` | `false` quando não há nada a desfazer / refazer |
| `lod_levels(&self, tolerance_px, scale, levels)` | `Result<Vec<LodLevel>, ApiError>` | o nível 0 é o original, o nível k >= 1 permanece dentro de `tolerance_px * 2^(k-1)` pixels de saída em `scale`; cada nível carrega seu limite declarado e um JSON de projeto. Determinístico; o limite cobre a reprodução da timeline (não as poses da máquina de estados). Lento em compilações não otimizadas (use release). |
| `fit_to_size(&self, FitFormat, max_bytes)` | `Result<SizeFit, ApiError>` | a menor exportação dentro do orçamento, reduzindo passo a passo os controles de qualidade; se for impossível, o menor resultado com `met == false` e um `reason` |
| `retarget_bvh(&mut self, bvh_text, &MocapSettings)` | `Result<MocapReport, ApiError>` | retarget de BVH como **uma única edição desfazível**; ossos associados por nome e topologia; canais bloqueados são ignorados e relatados |
| `fuzz(&self, samples, seed)` | `Result<FuzzSummary, ApiError>` | fuzzing de poses; a animação é apenas lida; relatório completo em JSON |

`Interpolation` do segmento que começa em uma chave: `Linear`, `Step` (manter), `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` (Catmull-Rom pelas chaves vizinhas). As curvas mais ricas do formato de projeto (Bezier cúbica, bounce, back, spring) são criadas pela CLI e pelo studio, não por `set_key`.

## Referência: `Player`

Uma máquina de estados em execução (reprodução interativa). Criada por `Animation::player`; trabalha sobre uma cópia do projeto; determinística. As máquinas de estados e os data bindings são JSON de projeto simples (`machines`, `bindings`), resumidos no [apêndice](#apêndice-json-de-máquina-e-json-de-binding).

| Método | Retorna | Observações |
|---|---|---|
| `set_bool(name, bool)`, `set_number(name, f64)`, `fire(name)` | `Result<(), ApiError>` | define uma entrada por nome; o tipo errado de entrada, ou um nome desconhecido, é `InvalidArgument`. Um trigger permanece definido até que uma transição que o testa dispare (ele é consumido) ou até o fim do próximo `advance`; `hold: true` o mantém até ser consumido. |
| `set_value(path, f64)` | `Result<(), ApiError>` | publica um valor externo para os data bindings (`"player.health"`). Uma região de rolagem cujo `input` é este caminho toma o valor como seu deslocamento em pixels ao longo do eixo principal (a rolagem fica então em repouso). |
| `advance(dt)` | `Result<(), ApiError>` | avança a máquina e as regiões de rolagem em `dt` segundos (`0..=10`) |
| `state(layer)` | `Option<String>` | estado atual de uma camada de máquina; `None` para uma camada desconhecida |
| `events_json()` | `String` | eventos pendentes `[{"time":..,"layer":..,"kind":"entered"|"exited","state":..}]`, não consumidos |
| `take_events_json()`, `clear_events()` | `String`, `()` | consomem / descartam os eventos pendentes |
| `render_rgba(&RenderSettings)`, `render_svg(show_bones)`, `bone_world()` | como em `Animation` | a pose atual; a imagem tem o tamanho de canvas definido por `set_canvas_size` vezes a escala |
| `set_canvas_size(w, h)`, `canvas_size()` | `Result<(), ApiError>`, `(f32, f32)` | (1.2) layout responsivo: os contêineres são resolvidos de novo, os limites de rolagem acompanham |
| `scroll_ids()` | `Vec<String>` | ids das regiões de rolagem do projeto |
| `scroll_drag(id, x, y)` | `Result<(), ApiError>` | deslocamento bruto do ponteiro em pixels (conteúdo movido para cima / para a esquerda nessa medida); além das extremidades é limitado (clamped) ou elástico (rubber-banded) |
| `scroll_release(id, vx, vy)` | `Result<(), ApiError>` | velocidade de soltura em px/s; `advance` então lança, ajusta e faz voltar com elasticidade |
| `scroll_offset(id)`, `scroll_max(id)`, `scroll_settled(id)` | `Option<[f32; 2]>`, `Option<[f32; 2]>`, `Option<bool>` | `None` para um id desconhecido |
| `set_screen_size(w_px, h_px, max_error_px)` | `Result<(), ApiError>` | (1.2) escolhe o mais grosseiro entre oito níveis de detalhe pré-construídos cujo limite declarado, na escala que ajusta o canvas a essa tela, seja no máximo `max_error_px`. Os níveis são construídos uma vez no primeiro uso (o nível 1 limitado por 0.125 unidades do palco, dobrando) e mantidos em cache; com `max_error_px` 0 apenas os níveis sem perdas se qualificam. |
| `clear_screen_size()`, `lod_level()`, `lod_bound_px()` | `()`, `usize`, `f32` | volta ao original; nível em uso (0 = original); seu limite declarado em pixels |

Observações: o limite do nível de detalhe cobre a geometria e as chaves da timeline como documentado em `lod.rs` (`ASSUMPTIONS`, `LIMITS`); as poses da máquina de estados e os bindings não são modelados por ele, e os canais de pose que apontam para uma camada removida por um nível são descartados. Os proxies de scrub do studio usam um mecanismo separado, com sua própria grade de níveis. O `input` de uma região de rolagem passa pela tabela de valores do player, não pelo runtime de data bindings, portanto expressões, remapeamento e suavização não se aplicam a ele.

## Referência: funções livres (ferramentas, controle de versão, CI)

| Função | Retorna | Faz |
|---|---|---|
| `trace_image_svg(&[u8], colors)` | `Result<String, ApiError>` | faz o tracing de um bitmap em um SVG de regiões preenchidas |
| `vcs_log(path, limit)` | `Result<Vec<VcsCommit>, ApiError>` | (1.2, somente leitura) histórico do mais novo para o mais antigo; `path` é o arquivo do projeto ou qualquer caminho dentro de seu repositório; um histórico vazio dá uma lista vazia |
| `ci_run(manifest)` | `Result<CiOutcome, ApiError>` | (1.2) CI de animação em modo verify (nada é regravado); os caminhos relativos no manifesto são resolvidos ao lado dele |
| `vcs_init(project_file, author)` | `Result<(), ApiError>` | (1.3) inicia um histórico ao lado do arquivo na branch `main`, ainda sem commit; falha se já existir um |
| `vcs_commit(path, message, author)` | `Result<String, ApiError>` | id do commit; conclui um merge quando não resta nenhum conflito; `Conflicts` enquanto restar algum; nada para commitar ou nenhuma mensagem é `InvalidArgument` |
| `vcs_branch(path, name, at)`, `vcs_tag(path, name, at)` | `Result<String, ApiError>` | cria uma branch / tag (padrão: o commit atual); sem trocar para ela; as tags nunca se movem; o id do commit é retornado |
| `vcs_checkout(path, target, force)` | `Result<String, ApiError>` | troca o arquivo para uma branch (permanece nela) ou para qualquer revisão (detached); `DirtyTree` a menos que `force`; `MergeInProgress` |
| `vcs_merge(path, rev, no_ff, message, author)` | `Result<VcsMergeOutcome, ApiError>` | `UpToDate`, `FastForward(id)`, `Merged(id)` ou `Conflicted(Vec<VcsConflict>)` |
| `vcs_cherry_pick(path, rev, author)`, `vcs_revert(path, rev, author)` | `Result<VcsMergeOutcome, ApiError>` | reaplica / desfaz a alteração de um commit |
| `vcs_conflicts(path)` | `Result<Vec<VcsConflict>, ApiError>` | os conflitos ainda pendentes (vazio quando não há) |
| `vcs_resolve(path, conflict_path, &VcsChoice)` | `Result<usize, ApiError>` | resolve um conflito com `Ours`, `Theirs` ou `Value(json text)`; retorna quantos restam |
| `vcs_abort(path)` | `Result<(), ApiError>` | descarta um merge / cherry-pick / revert inacabado e restaura o último commit |
| `vcs_branches(path)` | `Result<Vec<VcsBranch>, ApiError>` | (1.4) as branches com seu commit e se está em checkout |
| `vcs_branch_delete(path, name)` | `Result<(), ApiError>` | (1.4) exclui uma branch; a branch em checkout e nomes desconhecidos são recusados; os objetos permanecem até `vcs_gc` |
| `vcs_status(path)` | `Result<VcsStatus, ApiError>` | (1.4) branch, HEAD, alterações semânticas não commitadas, merge inacabado, caminhos de conflito |
| `vcs_diff(path, a, b)` | `Result<Vec<VcsChange>, ApiError>` | (1.4) diferenças semânticas entre duas revisões; `None` é o arquivo de trabalho |
| `vcs_show(path, rev)` | `Result<VcsShow, ApiError>` | (1.4) um commit e suas alterações em relação ao primeiro pai |
| `vcs_gc(path)` | `Result<VcsGc, ApiError>` | (1.4) exclui objetos inalcançáveis; aborta sem excluir se o histórico não puder ser lido |
| `vcs_fsck(path)` | `Result<Vec<String>, ApiError>` | (1.4) problemas de integridade, vazio quando tudo está saudável |
| `ci_init(project, manifest, write_goldens)` | `Result<CiInit, ApiError>` | grava um manifesto calibrado padrão (nunca sobrescreve um arquivo existente; nome padrão `osatura-ci.json` ao lado do projeto); opcionalmente grava os goldens agora |
| `ci_run_mode(manifest, CiMode)` | `Result<CiOutcome, ApiError>` | `Verify`, `Update` (regrava todos os goldens das verificações de render), `Bless` (aceita apenas os que falham ou estão ausentes) |
| `ci_bless(manifest)` | `Result<CiOutcome, ApiError>` | atalho para `Bless` |

O histórico é armazenado ao lado do arquivo do projeto (`.osatura-vcs/`); mensagens de commit e autores são texto simples. Um conflito contém o ponteiro de identidade `path`, um `kind`, uma `description` e os valores `base` / `ours` / `theirs` como texto JSON (`None` = ausente ali). Depois de um merge com conflito, o arquivo do projeto contém o estado mesclado com os itens em conflito no valor da branch atual, e um estado pendente é registrado até que você resolva todos os conflitos e faça `vcs_commit`, ou `vcs_abort`. Os assets binários que um projeto referencia não são versionados, apenas suas strings de caminho. O merge semântico, seus tipos de conflito e seus limites estão descritos em [ROADMAP.md](ROADMAP.md) seção 9 (item I-01) e em [LIMITATIONS.md](LIMITATIONS.md).

Fora da fachada: sobrescritas do mapeamento de ossos do mocap, ajustes de tracing além da contagem de cores, opções de fit além do formato e do orçamento, renderização em GPU, o studio, a colaboração, o servidor de agentes, os muitos comandos de criação e de baking (física, foot lock, magnets, autorig, ...), exclusão / listagem de branches, diff, show, status, gc, fsck, e opções de CI como prune, `--only` e artefatos. Use a CLI para isso ([CLI_REFERENCE.md](CLI_REFERENCE.md)).

## Referência: tipos

| Tipo | Campos / variantes |
|---|---|
| `AnimationInfo` | `name: String`, `width: f32`, `height: f32` (unidades do palco), `fps: f32`, `frames: u32` |
| `RenderSettings` | `scale: f32` (0.05..=4), `show_bones: bool`, `transparent: bool`; `new(scale)`, `with_bones(bool)`, `with_transparent(bool)`; `Default` = escala 1, ambos desativados |
| `Image` | `width: u32`, `height: u32`, `pixels: Vec<u8>` (RGBA8, alfa direto) |
| `ExportFormat` | `Gif`, `Glb`, `Lottie`, `Html`, `Apng` (1.4) |
| `PhysicsBake` (1.4) | `added: usize`, `skipped: Vec<String>` |
| `VcsBranch` (1.4) | `name`, `commit`, `current: bool` |
| `VcsChange` (1.4) | `kind` (`"Added"`, `"Removed"`, `"Changed"`, `"Moved"`, `"Retimed"`), `path`, `description` |
| `VcsStatus` (1.4) | `branch`, `head`, `changes: Vec<VcsChange>`, `merge`, `conflicts: Vec<String>` |
| `VcsShow` (1.4) | `commit: VcsCommit`, `changes: Vec<VcsChange>` |
| `VcsGc` (1.4) | `kept`, `removed` |
| `Interpolation` | `Linear`, `Step`, `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth` |
| `LodLevel` | `level`, `declared_px: f64`, `bound_stage: f64`, `path_anchors`, `keys`, `layers`, `json: String`; `bound_at(screen_scale) -> f64` |
| `FitFormat` | `Lottie`, `Html`, `Gif`, `Apng`, `Project` |
| `SizeFit` | `met: bool`, `size: usize`, `bytes: Vec<u8>`, `ssim_min: f64`, `reason: Option<String>` |
| `MocapSettings` | `in_place`, `mirror`, `smooth` (raio da média móvel em quadros); `new()`, `with_in_place`, `with_mirror`, `with_smooth`; `Default` |
| `MocapReport` | `matched: Vec<(String, String)>` (junta do BVH, osso do rig), `unmatched_joints`, `unmatched_bones`, `frames`, `bones_used`, `blocked` (`target.prop` bloqueado), `notes` |
| `FuzzSummary` | `samples`, `failures`, `failure_rate: f32`, `max_severity: f32`, `json: String` |
| `VcsCommit` | `id`, `author`, `time: u64` (segundos Unix), `message` |
| `VcsConflict` | `path`, `kind`, `description`, `base`, `ours`, `theirs: Option<String>` |
| `VcsMergeOutcome` | `UpToDate`, `FastForward(String)`, `Merged(String)`, `Conflicted(Vec<VcsConflict>)` |
| `VcsChoice` | `Ours`, `Theirs`, `Value(String)` |
| `CiMode` | `Verify`, `Update`, `Bless` |
| `CiOutcome` | `total`, `passed`, `failed`, `errors`, `exit_code: i32` (0 bom, 1 falhou, 2 não avaliável), `json: String`, `updated`, `blessed` |
| `CiInit` | `manifest: String`, `checks: Vec<(String, String)>` (tipo, nome), `recorded: Option<CiOutcome>` |
| Constantes | `API_VERSION: &str`, `API_VERSION_MAJOR: u32`, `MAX_RENDER_SIDE: u32`, `MAX_JSON_BYTES: usize` |

## Exemplos

Todos estes exemplos estão completos e foram executados. Eles pressupõem `use osatura_core::prelude::*;`.

### Carregar, avaliar, renderizar, exportar

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Load a project, query it, render one frame and export the whole animation.
pub fn quick(project_path: &str) -> Result<(), Box<dyn Error>> {
    let json = std::fs::read_to_string(project_path)?;
    let mut anim = Animation::from_json(&json)?;           // parse + validate; typed error otherwise

    // Facts about the project.
    let info = anim.info();
    println!("{}: {}x{} @ {} fps, {} frames", info.name, info.width, info.height, info.fps, info.frames);

    // Evaluate: world matrices of every bone at frame 10 (six floats per bone: a b c d tx ty).
    let names = anim.bone_names();
    let world = anim.bone_world(10.0)?;
    let m = world[0];
    // A bone-local point (x, y) lands at (a*x + c*y + tx, b*x + d*y + ty).
    let (x, y) = (m[0] * 10.0 + m[2] * 0.0 + m[4], m[1] * 10.0 + m[3] * 0.0 + m[5]);
    println!("tip of {} (10 units along its x axis) is at ({x:.1}, {y:.1})", names[0]);

    // Edit through the undoable editor, then render.
    anim.set_key(&names[0], "bone_angle", 10.0, 25.0, Interpolation::EaseInOut)?;
    let png = anim.render_png(10.0, &RenderSettings::new(1.0).with_bones(true))?;
    std::fs::write("frame10.png", &png)?;
    let svg = anim.render_svg(10.0, false)?;
    std::fs::write("frame10.svg", svg)?;

    // Raw pixels: straight-alpha RGBA8, row-major, top row first.
    let img = anim.render_rgba(10.0, &RenderSettings::new(0.5).with_transparent(true))?;
    assert_eq!(img.pixels.len(), img.width as usize * img.height as usize * 4);

    // Whole-animation exports.
    let glb = anim.export(ExportFormat::Glb, 1.0)?;
    let gif = anim.export(ExportFormat::Gif, 0.5)?;        // `scale` only matters for GIF
    std::fs::write("anim.glb", &glb)?;
    std::fs::write("anim.gif", &gif)?;
    println!("{} bones, {} bytes of glb, {} bytes of gif", names.len(), glb.len(), gif.len());
    Ok(())
}
```

### Edição, desfazer e tratamento de erros

```rust
use osatura_core::prelude::*;

/// Edits are validated, undoable and all-or-nothing.
pub fn edit_and_errors() {
    let mut anim = Animation::demo();                       // small built-in project
    let bone = anim.bone_names()[0].clone();
    let before = anim.to_json();

    anim.set_key(&bone, "bone_angle", 4.0, 30.0, Interpolation::Smooth).unwrap();
    assert!(anim.delete_key(&bone, "bone_angle", 4.0).unwrap());      // true: a key was removed
    assert!(!anim.delete_key(&bone, "bone_angle", 77.0).unwrap());    // false: nothing there
    assert!(anim.undo() && anim.undo());                    // each successful edit is one undo step
    assert_eq!(anim.to_json(), before);
    assert!(anim.redo());

    // Errors are values. Match on them with a wildcard arm: the enum is #[non_exhaustive].
    match anim.set_key(&bone, "no_such_prop", 0.0, 1.0, Interpolation::Linear) {
        Err(ApiError::InvalidArgument(why)) => println!("rejected argument: {why}"),
        Err(ApiError::EditRejected(problems)) => println!("edit refused: {problems:?}"),
        Err(other) => println!("other failure: {other}"),   // Display is implemented for every variant
        Ok(()) => unreachable!(),
    }
    // A rejected edit leaves the project exactly as it was.
    let snapshot = anim.to_json();
    assert!(anim.set_key("no_such_bone", "bone_angle", 1.0, 1.0, Interpolation::Linear).is_err());
    assert_eq!(anim.to_json(), snapshot);

    // Bad input never panics.
    assert!(matches!(Animation::from_json("{ nope"), Err(ApiError::InvalidJson(_))));
    assert!(anim.render_rgba(f32::NAN, &RenderSettings::default()).is_err());
    assert!(anim.render_rgba(0.0, &RenderSettings::new(50.0)).is_err()); // scale must be 0.05..=4
}
```

### Reprodução interativa com uma máquina de estados

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Run a state machine of a project (machine.json has machine "hero", layer "body", bool input "excited").
pub fn play(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let machine = anim.machine_names()[0].clone();         // e.g. "hero"
    let mut player = anim.player(&machine)?;               // works on a copy of the project

    player.set_bool("excited", true)?;                     // also set_number, fire (trigger), set_value (binding path)
    for _ in 0..30 {
        player.advance(1.0 / 30.0)?;                       // deterministic; dt in seconds, 0..=10
    }
    println!("body is in state {:?}", player.state("body"));  // Some("wave")
    println!("{}", player.take_events_json());             // [{"time":..,"layer":"body","kind":"entered","state":..}, ..]

    let frame = player.render_rgba(&RenderSettings::new(0.5))?;
    println!("pose rendered at {}x{}", frame.width, frame.height);
    let svg = player.render_svg(false)?;
    assert!(svg.starts_with("<svg"));

    // Same inputs, same advance calls, same pose: a second player is bit-for-bit identical.
    let mut again = anim.player(&machine)?;
    again.set_bool("excited", true)?;
    for _ in 0..30 {
        again.advance(1.0 / 30.0)?;
    }
    assert_eq!(again.render_rgba(&RenderSettings::new(0.5))?, frame);

    // Wrong calls are typed errors, not panics.
    assert!(matches!(player.set_number("excited", 1.0), Err(ApiError::InvalidArgument(_))));
    assert!(matches!(anim.player("no such machine"), Err(ApiError::InvalidArgument(_))));
    Ok(())
}
```

### Layout, rolagem e nível de detalhe

```rust
use osatura_core::prelude::*;
use std::error::Error;

/// Layout, scrolling and level of detail on a Player (scroll.json has one scroll region "list").
pub fn layout_and_lod(project_path: &str) -> Result<(), Box<dyn Error>> {
    let anim = Animation::from_json(&std::fs::read_to_string(project_path)?)?;
    let mut player = anim.player(&anim.machine_names()[0])?;

    player.set_canvas_size(100.0, 120.0)?;                 // containers re-solve; renders get this size
    for id in player.scroll_ids() {
        let max = player.scroll_max(&id).unwrap();         // about [0, 80] here: 200 units of content in a 120 unit view
        player.scroll_drag(&id, 0.0, 50.0)?;               // raw pointer offset in px (clamped / rubber-banded)
        assert_eq!(player.scroll_offset(&id), Some([0.0, 50.0]));
        player.scroll_release(&id, 0.0, 900.0)?;           // fling at 900 px/s
        while player.scroll_settled(&id) == Some(false) {
            player.advance(1.0 / 60.0)?;                   // steps the machine and every scroll region
        }
        println!("{id}: max {max:?}, rests at {:?}", player.scroll_offset(&id).unwrap());
    }

    // Pick the coarsest level of detail whose declared error stays within 0.5 px at this screen size.
    player.set_screen_size(100.0, 120.0, 0.5)?;
    println!("level {} (declared bound {} px)", player.lod_level(), player.lod_bound_px());
    player.clear_screen_size();                            // back to the original
    assert_eq!(player.lod_level(), 0);

    // A scroll region whose `input` names a path follows set_value(path, px) instead of a drag.
    player.set_value("ui.scroll", 200.0)?;
    Ok(())
}
```

### Renderização em várias threads

```rust
use osatura_core::prelude::*;

/// `Animation` and `Player` are `Send + Sync`: read-only calls (`&self`) may run on several threads.
pub fn render_in_parallel() -> Result<(), ApiError> {
    let anim = Animation::demo();
    let frames = anim.frame_count();
    let settings = RenderSettings::new(0.25);
    let (a, s) = (&anim, &settings);                        // shared references are Copy: move them into the threads
    let images: Vec<Result<Image, ApiError>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..frames).map(|f| scope.spawn(move || a.render_rgba(f as f32, s))).collect();
        jobs.into_iter().map(|j| j.join().expect("render thread")).collect()
    });
    // Mutating calls (`&mut self`: set_key, undo, retarget_bvh ...) need exclusive access.
    for (f, img) in images.into_iter().enumerate() {
        assert_eq!(img?, anim.render_rgba(f as f32, &settings)?);   // threads do not change the result
    }
    Ok(())
}
```

### Controle de versão e CI de animação

```rust
use osatura_core::prelude::*;

/// Version control of a project file and animation CI, from Rust.
pub fn history(dir: &std::path::Path) -> Result<(), ApiError> {
    let file = dir.join("hero.json");
    std::fs::write(&file, Animation::demo().to_json()).unwrap();
    let path = file.to_str().unwrap();

    // A tiny helper: re-key the first bone at frame 5 and save the file.
    let edit = |angle: f32| -> Result<(), ApiError> {
        let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
        let bone = a.bone_names()[0].clone();
        a.set_key(&bone, "bone_angle", 5.0, angle, Interpolation::Linear)?;
        std::fs::write(path, a.to_json()).unwrap();
        Ok(())
    };

    vcs_init(path, Some("ana"))?;                           // branch `main`, no commit yet
    vcs_commit(path, Some("base"), None)?;
    vcs_branch(path, "other", None)?;                       // created, not switched to
    edit(20.0)?;
    vcs_commit(path, Some("main edit"), None)?;
    vcs_checkout(path, "other", false)?;                    // DirtyTree if the file has uncommitted changes
    edit(40.0)?;                                            // the same key, another value
    vcs_commit(path, Some("other edit"), None)?;
    vcs_checkout(path, "main", false)?;

    match vcs_merge(path, "other", false, None, None)? {
        VcsMergeOutcome::Conflicted(conflicts) => {         // a normal result, not an error
            for c in &conflicts {
                println!("{} ({}): {}", c.path, c.kind, c.description);
                vcs_resolve(path, &c.path, &VcsChoice::Theirs)?;
            }
            vcs_commit(path, None, None)?;                  // the merge commit
        }
        other => println!("merged without conflicts: {other:?}"),
    }
    for c in vcs_log(path, 10)? {
        println!("{} {} {}", &c.id[..8], c.author, c.message);
    }

    // Animation CI: write a calibrated manifest, record goldens, then verify.
    let init = ci_init(path, None, true)?;                  // never overwrites an existing manifest
    let run = ci_run(&init.manifest)?;                      // verify mode: nothing is re-recorded
    println!("{} checks, {} passed, exit code {}", run.total, run.passed, run.exit_code);
    if run.exit_code == 1 {
        // A failing check is a result, not an Err. After reviewing the report, accept the new look:
        ci_bless(&init.manifest)?;
    }
    Ok(())
}
```

### Mocap, fuzzing, tracing, nível de detalhe e exportação com tamanho-alvo

```rust
use osatura_core::prelude::*;

const RIG: &str = r#"{"format":1,"name":"rig","width":400.0,"height":400.0,"fps":30.0,"frames":10,"background":null,
 "camera":{"position":null,"zoom":1.0,"rotation":0.0},
 "bones":[{"name":"hips","parent":null,"length":20.0,"rest_offset":[200.0,250.0],"rest_angle":-90.0},
          {"name":"spine","parent":"hips","length":40.0,"rest_offset":[20.0,0.0],"rest_angle":0.0},
          {"name":"head","parent":"spine","length":30.0,"rest_offset":[40.0,0.0],"rest_angle":0.0}],
 "ik":[],"constraints":[],"springs":[],"layers":[],"channels":[]}"#;

fn mini_bvh() -> String {
    let mut s = String::from("HIERARCHY\nROOT Hips\n{\n OFFSET 0 100 0\n CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation\n");
    s.push_str(" JOINT Spine\n {\n  OFFSET 0 12 0\n  CHANNELS 3 Zrotation Xrotation Yrotation\n  JOINT Head\n  {\n   OFFSET 0 12 0\n   CHANNELS 3 Zrotation Xrotation Yrotation\n   End Site\n   {\n    OFFSET 0 8 0\n   }\n  }\n }\n}\nMOTION\nFrames: 4\nFrame Time: 0.0333333\n");
    for f in 0..4 {
        s.push_str(&format!("0 100 0 0 0 0 {} 0 0 0 0 0\n", f * 10));
    }
    s
}

/// Motion capture, pose fuzzing, tracing, level of detail and size-targeted export.
pub fn tools() -> Result<(), ApiError> {
    // BVH retargeting: one undoable edit; bones are matched by name and topology.
    let mut rig = Animation::from_json(RIG)?;
    let report = rig.retarget_bvh(&mini_bvh(), &MocapSettings::new().with_smooth(1))?;
    println!("matched {:?}; {} frames on {} bones", report.matched, report.frames, report.bones_used);
    assert!(rig.undo());                                    // the whole retarget is one step

    // Pose fuzzing: where does the rig break?  (deterministic for a given seed)
    let demo = Animation::demo();
    let fuzz = demo.fuzz(50, 1)?;
    println!("{} of {} samples failed (max severity {:.2})", fuzz.failures, fuzz.samples, fuzz.max_severity);
    let _full_report: &str = &fuzz.json;                    // schema "osatura-fuzz-report" v1

    // Bitmap tracing: PNG/JPEG bytes -> SVG, or straight into an animation.
    let png = demo.render_png(0.0, &RenderSettings::new(0.25))?;
    let svg = trace_image_svg(&png, 6)?;
    let traced = Animation::from_image(&png, 6, "traced")?;
    println!("traced SVG: {} bytes; project has {} layers", svg.len(), traced.layer_names().len());

    // Level of detail with a declared error bound, one project JSON per level (level 0 = original).
    for l in demo.lod_levels(0.5, 1.0, 3)? {
        println!("L{}: bound {:.3} px, {} anchors, {} keys", l.level, l.declared_px, l.path_anchors, l.keys);
        let _reduced = Animation::from_json(&l.json)?;
    }

    // Smallest export within a byte budget.
    let fit = demo.fit_to_size(FitFormat::Lottie, 250_000)?;
    println!("met={} {} bytes, worst SSIM {:.3}", fit.met, fit.size, fit.ssim_min);
    Ok(())
}
```

### Como os exemplos são verificados

O código acima é o código que foi executado: este arquivo é gerado a partir dos fontes dos exemplos, que foram compilados com `cargo` contra `osatura-core` e executados (os arquivos de projeto usados são a saída de `osatura demo`, `take-record`, `clip add` e `machine add`, mais um projeto com um contêiner e uma região de rolagem). Os doc-tests em `api/` (`cargo test --doc -p osatura-core`: 19 doc-tests, 18 em funções da fachada em `api/` e o exemplo em nível de crate em `lib.rs`; cerca de 170 s em uma compilação de debug, porque cada um compila um programa) cobrem os wrappers de controle de versão e de CI no próprio repositório.

## Outras linguagens

A fachada é a única fonte de verdade para os demais bindings. Cobertura:

| Capacidade | Fachada Rust | C ABI (`osatura.h`) | WebAssembly / JS |
|---|---|---|---|
| Carregar, validar, info, nomes dos ossos / matrizes de mundo | sim | `osatura_open`, `_bone_*`, `_frame_count`, ... | `arm_open`, `load()` (sem matrizes de ossos) |
| Renderizar RGBA | sim | `osatura_render_rgba` | `arm_render_rgba`, `rgba()` |
| Renderizar SVG / PNG / formatos de exportação | sim | não | SVG sim (`svg()`), PNG / exportação não |
| Editar chaves, desfazer / refazer | sim | não | não |
| `Player`: entradas, advance, estado, eventos, render | sim | `osatura_player_*` | `arm_player_*`, `startPlayer()`, ... |
| Layout, rolagem, nível de detalhe | sim | `osatura_player_set_canvas`, `_scroll_*`, `_set_screen`, `_lod_level` | `arm_player_set_canvas / layout / scroll_* / set_screen` |
| Ferramentas, controle de versão, CI | sim | não | não |

A versão da C ABI continua em 1 porque todas as adições até agora foram aditivas. O binário wasm é compilado e testado sob o Node no CI, mas não foi executado em um navegador; veja [WASM.md](WASM.md).

## Apêndice: JSON de máquina e JSON de binding

Resumos curtos das duas seções do projeto que o `Player` executa (formato completo: [PROJECT_FORMAT.md](PROJECT_FORMAT.md)).

### JSON de máquina (resumo)

```json
{"name":"hero",
 "inputs":[{"name":"moving","kind":"bool"},{"name":"speed","kind":"number"},{"name":"jump","kind":"trigger"}],
 "layers":[{"name":"body","mask":["arm_*"],"entry":"idle",
   "states":[{"name":"idle","source":{"kind":"clip","clip":"idle"},"blend_in":0.2},
             {"name":"loco","source":{"kind":"blend","input":"speed","children":[
                {"threshold":0,"source":{"kind":"clip","clip":"walk"}},{"threshold":1,"source":{"kind":"clip","clip":"run"}}]}},
             {"name":"jump","source":{"kind":"clip","clip":"jump"},"mode":"once"}],
   "transitions":[{"from":"idle","to":"loco","when":[{"input":"moving"}],"duration":0.25,"ease":{"type":"ease_in_out"}},
                  {"from":"*","to":"jump","when":[{"input":"jump"}],"duration":0.1},
                  {"from":"jump","to":"idle","exit_time":1.0}]}]}
```

As condições são `{"input","op":"eq|ne|lt|le|gt|ge","value"}` (padrões `eq`, `1`; todas devem ser verdadeiras). `from: "*"` corresponde a qualquer estado, exceto o de destino. A primeira transição correspondente, na ordem de declaração, prevalece. `exit_time` é a fração reproduzida (0..1) do estado. Fontes: `clip` (uma entrada de `Project.clips`), `segment` (quadros `start`, `end` dos canais com chaves em modo replace do projeto), `region` (um marcador com fim), `blend` (1-D, duas ou mais fontes folha em limiares crescentes). Chaves desconhecidas em uma transição ou condição são um erro. CLI: `machine add|list|remove|validate`, `machine-sim <project> <script.json> <out_dir|out.json>` (script: `{"machine":"hero","duration":2,"events":[{"t":0.5,"set":{"moving":true},"trigger":["jump"],"value":{"player.health":40}}]}`).

### JSON de binding (resumo)

```json
{"name":"hp","path":"player.health","expr":"value / 100","remap":{"in_min":0,"in_max":1,"out_min":-30,"out_max":30},
 "smooth":0.15,"target":{"kind":"prop","target":"arm_up","prop":"bone_angle"}}
```

Pipeline: valor, depois `expr` (variável `value`; as funções de expressão do engine, fonte de no máximo 4 KB e aninhamento de no máximo 64), depois `remap` (limitado por padrão; `curve` suaviza), depois a suavização (constante de tempo em segundos), depois o destino: `{"kind":"prop",...}` (qualquer propriedade de canal, incluindo `layer_opacity` e `layer_switch`) ou `{"kind":"input","machine","input"}` (uma entrada trigger dispara na borda de subida). CLI: `bind add|list|remove`, `bind-eval`.

### Layout e normal maps por camada

Os contêineres de layout e as regiões de rolagem são campos do projeto `containers` / `scrolls` (omitidos quando vazios): CLI `container add|list|remove|solve`, `scroll add|list|remove|eval`, `layout-preview <project> <out.svg> --size WxH`, e `--size WxH` em `render`, `frames` e `export` (os contêineres são resolvidos de novo para esse tamanho de canvas; o arquivo permanece inalterado). `Layer.normal_map` (`data` PNG em base64 ou `path`, `fit` stretch / tile / exact, `strength`, `green_down`) alimenta o efeito de pós-processamento `relight` para os pixels que essa camada, ou um filho sem mapa próprio, cobre (CLI `layer-normal`, `fx ... --layer-normals`); não faz parte da fachada.
