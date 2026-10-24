<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA

**Um motor de rigging e animação vetorial 2D em Rust: ossos, IK, skinning, deformadores, um runtime de máquina de estados, um conjunto de ferramentas de linha de comando, um estúdio no navegador e bibliotecas embarcáveis, tudo em um único workspace.** Versão em romeno: [README.ro.md](README.ro.md).

![Uma personagem de perfil (cabeça, orelha, rabo de cavalo, tronco, braços com dedos, pernas, pés) caminha no lugar sobre um chão que desliza; saída real de osatura export no projeto de demonstração incluído](docs/img/walk.gif)

*O que produz: a pessoa a caminhar incluída, exportada com `osatura demo-person` e `osatura export` (o projeto inteiro é um único ficheiro JSON). Experimente em dois minutos: [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).*

## Resumo e filosofia

O OSATURA permite construir um personagem ou uma cena a partir de formas vetoriais e imagens, fazer o rig com ossos e restrições, animar com chaves, clipes e física, e exportar como GIF, vídeo, Lottie, glTF, um player HTML autônomo ou um runtime interativo. Ele é feito de peças pequenas e determinísticas: o mesmo projeto e o mesmo seed produzem os mesmos pixels, todas as ferramentas leem e gravam um projeto JSON documentado, e todos os comandos funcionam a partir de um terminal, de modo que a animação pode ser automatizada por scripts, revisada, comparada com diff e testada como código-fonte.

Princípios que moldam o projeto:

* **Implementação independente.** Escrito a partir de especificações públicas, artigos e matemática de livros didáticos. As listas de recursos dos concorrentes foram consultadas apenas quanto ao *escopo*; nenhum código-fonte de terceiros foi copiado, e o projeto não lê nem grava nenhum formato proprietário. Não é um processo clean-room estrito (veja [CLEANROOM.md](docs/i18n/pt/CLEANROOM.md), incluindo sua divulgação de pesquisa) e não é afiliado, derivado nem compatível com o Moho ou qualquer outro produto comercial. O código foi escrito em grande parte com um modelo de IA.
* **Status honesto.** Cada recurso é descrito junto com o que ele *não* faz ([docs/LIMITATIONS.md](docs/i18n/pt/LIMITATIONS.md), [ROADMAP.md](docs/i18n/pt/ROADMAP.md) seção 9).
* **Seguro por padrão.** Sem panics com a entrada, verificações de valores finitos, limites de recursos, gravações atômicas, caminhos de recursos restritos à pasta do projeto ([docs/SECURITY.md](docs/i18n/pt/SECURITY.md)).
* **Formatos abertos em primeiro lugar.** OpenRaster, OCA, SVG, Lottie, glTF, APNG, WebM, EXR; nenhum formato de arquivo proprietário, por princípio.
* **Pegada pequena de dependências**, com uma política de licenças verificada por testes e por `cargo deny`.

## Status

*Nome: o projeto se chamou ARMATURA até 24 out 2026 e foi renomeado para OSATURA depois de uma busca de marca ([docs/LIMITATIONS.md](docs/i18n/pt/LIMITATIONS.md) L-78). O endereço do repositório e o do site também foram renomeados (os links antigos do repositório redirecionam).*

> **Alpha. Primeiros builds preparados como um pré-lançamento em rascunho: `v0.1.0-alpha.3`.** Os executáveis para Windows x86-64, macOS arm64 e Linux x86-64 e o módulo WebAssembly são compilados pelo GitHub Actions e anexados a um pré-lançamento *em rascunho* em <https://github.com/chiuta/osatura/releases> (um rascunho só é visível para o dono até ser publicado manualmente). Todos os crates continuam na versão `0.1.0`; "v0.5 / v0.6 / v0.7" são *níveis de recursos* no histórico, não números de versão ([CHANGELOG.md](docs/i18n/pt/CHANGELOG.md)).
>
> | Verificado | Não verificado |
> |---|---|
> | **Local (Linux x86-64, 24 out 2026):** 1338 testes de `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` mais 19 doc-tests passam (o crate de GPU precisa de um adaptador e o crate de janela precisa de um desktop; antes disso, a contagem incluindo o crate de GPU era 1368). **No GitHub Actions (24 out 2026, toolchain 1.97.0):** a suíte de testes e `clippy -D warnings` passam em Linux, macOS (arm64) e Windows; `cargo deny` e `reuse lint` passam; o módulo WebAssembly compila para `wasm32-unknown-unknown` e o teste do wrapper passa contra o módulo real no Node 20; os binários de release passam em um smoke test (`--version`, `demo`, `check`, `render`) e `osatura-studio --selftest` roda no Windows e no macOS. Quatro smoke tests do estúdio em Chromium headless passam; a CLI, a facade Rust e a C ABI (também sob ASan/UBSan) foram executadas com os exemplos da documentação | **Nenhuma pessoa usou os executáveis em um desktop real com Windows ou macOS**: o inicializador de duplo clique do Windows e o aplicativo de janela `osatura-studio` (WebView2 / WKWebView) foram apenas compilados e autotestados no CI; os binários **não são assinados digitalmente** (SmartScreen "Editor desconhecido", Gatekeeper do macOS); o binário Linux foi compilado apenas em um runner Ubuntu; **o módulo WebAssembly nunca foi executado em um navegador** e sua velocidade e memória não foram medidas; o estúdio nunca foi usado por uma pessoa em produção, nem em um navegador que não seja o Chromium; nenhuma GPU real (apenas um adaptador por software), webcam, fotografia ou arquivo de captura de movimento foi usado; sem auditoria de segurança, revisão jurídica ou busca de patentes |
>
> Leia [docs/LIMITATIONS.md](docs/i18n/pt/LIMITATIONS.md) (78 limitações numeradas, com gravidade e alternativa) antes de depender de qualquer coisa.

## Download

Os arquivos pré-compilados, depois que o dono publicar o rascunho do lançamento, estão na [página de Releases](https://github.com/chiuta/osatura/releases) do projeto:

| Arquivo | Conteúdo |
|---|---|
| `osatura-windows-x86_64.zip` | `osatura.exe` (linha de comando + servidor do estúdio), `osatura-studio.exe` (aplicativo de janela), biblioteca C e cabeçalho, licenças, avisos de terceiros |
| `osatura-macos-arm64.tar.gz` | `osatura`, `osatura-studio` (somente Apple silicon), biblioteca C e cabeçalho, licenças, avisos |
| `osatura-linux-x86_64.tar.gz` | `osatura`, biblioteca C e cabeçalho, licenças, avisos (sem aplicativo de janela: use `osatura studio` e um navegador) |
| `osatura-web.tar.gz` | `osatura_wasm.wasm`, `osatura.js`, demo `index.html` (não verificado em navegadores, veja [docs/WASM.md](docs/i18n/pt/WASM.md)) |
| `SHA256SUMS.txt` | confira cada download com ele |

* **Windows:** dê um duplo clique em `osatura.exe` para criar `Documents\OSATURA\first-rig.json` e abrir o estúdio no navegador, ou dê um duplo clique em `osatura-studio.exe` para ter uma janela própria. Os dois programas não são assinados: escolha *Mais informações* e depois *Executar assim mesmo* no SmartScreen. Veja [docs/GETTING_STARTED.md](docs/i18n/pt/GETTING_STARTED.md).
* **macOS:** aprove o programa em *Ajustes do Sistema > Privacidade e Segurança* na primeira vez.
* **Código-fonte:** cada arquivo informa o commit a partir do qual foi compilado; as partes AGPL são servidas com a sua oferta de código-fonte (`osatura --version`, `/source`).

## Recursos por área

Detalhes e opções exatas: [docs/CLI_REFERENCE.md](docs/i18n/pt/CLI_REFERENCE.md); o que é parcial ou está faltando: [docs/LIMITATIONS.md](docs/i18n/pt/LIMITATIONS.md).

* **Modelo de desenho.** Caminhos vetoriais, preenchimentos, traços (tracejados, largura variável), gradientes lineares e radiais, máscaras, modos de mesclagem, efeitos por camada (desfoque, sombra, tonalização, preenchimento), camadas de imagem com mesh warp, metaballs líquidas, operações booleanas de polígonos, partículas determinísticas, câmera multiplano com tomadas salvas, profundidade de campo.
* **Rigging.** Ossos com cinemática direta; IK de dois ossos, suave, com alongamento e FABRIK; limites, restrições, molas; skinning com ferramentas de pesos; deformadores lattice, spine, morph, wobble e cage; mesh tracing a partir do alfa; pivôs de warp e glue; drivers (sliders, joysticks, troca de vista); skin sets; auto-rig a partir de nomes de camadas; eye clipping; contêineres de layout e regiões de rolagem.
* **Animação.** Chaves com easing Bézier/step/spring/bounce, expressões e canais controlados, camadas de troca, takes e uma pilha de clipes em camadas, conversão para loop e in-place, foot lock, magnets, seguimento de caminho, geradores de idle e overlap, física de cadeia/cabelo/tecido, lip-sync a partir de áudio ou de uma transcrição, movimento de cabeça sensível à fala, retargeting de BVH, entrada ao vivo (MIDI, mouse, gamepad, pontos de referência de webcam que você fornece), faixas de áudio com corte, fade e mixagem.
* **Runtime.** Máquinas de estados com blend trees 1-D, data binding, física de rolagem, nível de detalhe com limite de erro, um sandbox de scripts Rhai, um servidor de agente (MCP) sobre operações de patch.
* **Saída.** SVG, PNG, sequências de PNG e sprite sheets, GIF, APNG, MP4/WebM (por meio do ffmpeg), player HTML autônomo, Lottie e dotLottie, glTF 2.0 binário, OpenEXR/PNG de 16 bits, OCA; exportação com tamanho alvo (`export-fit`). Importação: SVG, OpenRaster, Lottie, BVH, sequências de PNG/GIF/APNG.
* **Ferramentas de qualidade.** `check`, `lint`, `diff` semântico, pose fuzzing com mapas de calor, CI de animação com diff perceptual e quadros de referência ([docs/CI.md](docs/i18n/pt/CI.md)), controle de versão com merge semântico (`vcs`), verificações de segurança para flashes e movimento.
* **Estúdio.** Editor no navegador servido pela CLI: gizmos no canvas, timeline, dope sheet e editor de curvas, overlays, paleta de comandos e atalhos, desfazer/refazer, edição multiusuário em tempo real com token opcional e um log persistente ([docs/STUDIO.md](docs/i18n/pt/STUDIO.md)).
* **Bibliotecas.** Facade Rust estável ([docs/API.md](docs/i18n/pt/API.md)), C ABI ([docs/C_ABI.md](docs/i18n/pt/C_ABI.md)), módulo WebAssembly e wrapper JavaScript (compilados e testados sob o Node no CI, ainda não executados em um navegador; [docs/WASM.md](docs/i18n/pt/WASM.md)), renderizador wgpu opcional.

## Início rápido

Requisitos: uma toolchain Rust estável recente (ou um arquivo pré-compilado da seção [Download](#download): pule a linha `cargo build` e use o `osatura` descompactado). O `ffmpeg` é necessário apenas para a exportação em MP4/WebM e para áudio mp3/ogg; ele nunca é incluído. Todos os comandos abaixo foram executados para este README (build de depuração, Linux). Os builds de release (`cargo build --release`) são muito mais rápidos para renderização e são usados da mesma forma, com `target/release`.

```sh
cargo build -p osatura-cli                       # builds target/debug/osatura
export PATH="$PWD/target/debug:$PATH"             # so that `osatura` is found

osatura --version                                # version, licence, source offer, support links
osatura demo demo.json                           # write the built-in rig demo (640x480, 48 frames)
osatura check demo.json                          # validate: ok: 'OSATURA demo' 640x480 @ 24 fps, 48 frames, 6 bones...
osatura lint demo.json                           # static rig/animation checks: 0 finding(s), 0 error(s)
osatura render demo.json 12 frame12.png --bones  # one frame, with the skeleton overlay
osatura render demo.json 12 frame12.svg          # the same frame as vector output
osatura frames demo.json frames_out --png        # every frame as a PNG
osatura export demo.json demo.gif                # also .html (player), .json (Lottie), .lottie, .mp4 (ffmpeg)
osatura demo-fx fx.json                          # second demo: liquid, booleans, particles
osatura lipsync demo.json voice.wav mouth out.json   # mouth shapes from a WAV file (switch layer "mouth")
osatura studio demo.json --port 8080             # browser editor at http://127.0.0.1:8080
osatura ci init demo.json && osatura ci run     # animation CI: goldens + checks (about 40 s in a debug build)
```

`voice.wav` é qualquer arquivo WAV mono ou estéreo que você fornecer. O estúdio escuta apenas em `127.0.0.1` e recusa requisições para outros nomes de host; para compartilhá-lo, leia antes [docs/SECURITY.md](docs/i18n/pt/SECURITY.md).

Use as bibliotecas:

```sh
cargo build -p osatura-capi                      # target/debug/libosatura_capi.so and .a
gcc -std=c99 -o smoke osatura-capi/tests/smoke.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./smoke demo.json    # abi 1, 48 frames, 320x240, rc=0, checksum=...
```

A facade Rust é `osatura_core::prelude` ([docs/API.md](docs/i18n/pt/API.md)); há mais exemplos em C e Python em [docs/C_ABI.md](docs/i18n/pt/C_ABI.md).

Testes (cerca de 14 minutos em um build de depuração em uma máquina compartilhada; veja [docs/DEVELOPMENT.md](docs/i18n/pt/DEVELOPMENT.md)):

```sh
cargo test --workspace
cargo test --doc -p osatura-core
cargo test -p osatura-cli --test spdx_audit
```

## Estrutura do repositório

| Caminho | Conteúdo | Licença |
|---|---|---|
| `osatura-core/` | motor, formatos, ferramentas, facade estável `api` | MPL-2.0 |
| `osatura-cli/` | o binário `osatura`, estúdio e servidor de colaboração, testes de ponta a ponta e smoke tests | AGPL-3.0-or-later |
| `osatura-gui/` | `osatura-studio`, o estúdio em uma janela nativa (Windows e macOS; um stub no Linux) | AGPL-3.0-or-later |
| `osatura-capi/` | C ABI (`osatura.h`, biblioteca compartilhada e estática) | MPL-2.0 |
| `osatura-wasm/` | ABI WebAssembly, wrapper JavaScript, página de demonstração (compilado no CI) | MPL-2.0 |
| `osatura-gpu/` | renderizador wgpu opcional (não é um membro padrão) | MPL-2.0 |
| `docs/` | documentação (CC-BY-4.0) e arquivos de terceiros gerados | CC-BY-4.0 |
| `.github/workflows/` | CI (`ci.yml`) e builds de release (`release.yml`) | MPL-2.0 |
| `scripts/` | `e2e.sh`, `licence-audit.sh` | MPL-2.0 |
| `LICENSES/`, `REUSE.toml`, `deny.toml` | textos das licenças, metadados de licença, política de dependências | - |
| `ROADMAP.md`, `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `TRADEMARK.md`, `CONTRIBUTING.md`, `CHANGELOG.md` | documentos do projeto | CC-BY-4.0 |

## Mapa da documentação

Comece em [docs/INDEX.md](docs/i18n/pt/INDEX.md) (todos os documentos, públicos-alvo, trilhas de leitura).

| Página | Para |
|---|---|
| [docs/GETTING_STARTED.md](docs/i18n/pt/GETTING_STARTED.md) | primeiro projeto, renderização, exportação |
| [docs/CLI_REFERENCE.md](docs/i18n/pt/CLI_REFERENCE.md) | todos os comandos e opções |
| [docs/STUDIO.md](docs/i18n/pt/STUDIO.md) | o editor no navegador |
| [docs/RIGGING_GUIDE.md](docs/i18n/pt/RIGGING_GUIDE.md) | construção de rigs |
| [docs/ANIMATION_GUIDE.md](docs/i18n/pt/ANIMATION_GUIDE.md) | animação, clipes, física, áudio |
| [docs/PROJECT_FORMAT.md](docs/i18n/pt/PROJECT_FORMAT.md) | o formato de projeto JSON |
| [docs/EXPORT_FORMATS.md](docs/i18n/pt/EXPORT_FORMATS.md) | o que cada formato preserva e descarta |
| [docs/ARCHITECTURE.md](docs/i18n/pt/ARCHITECTURE.md) | projeto e módulos |
| [docs/GLOSSARY.md](docs/i18n/pt/GLOSSARY.md) | termos |
| [docs/FAQ.md](docs/i18n/pt/FAQ.md) | perguntas frequentes |
| [docs/API.md](docs/i18n/pt/API.md) | facade Rust 1.4 |
| [docs/C_ABI.md](docs/i18n/pt/C_ABI.md) | interface C |
| [docs/WASM.md](docs/i18n/pt/WASM.md) | WebAssembly (compilado no CI, não executado em um navegador) |
| [docs/CI.md](docs/i18n/pt/CI.md) | CI de animação |
| [docs/DEVELOPMENT.md](docs/i18n/pt/DEVELOPMENT.md) | build, testes, convenções, lançamento |
| [docs/SECURITY.md](docs/i18n/pt/SECURITY.md) | modelo de ameaças e reforço de segurança |
| [docs/LIMITATIONS.md](docs/i18n/pt/LIMITATIONS.md) | limitações conhecidas |
| [docs/PATENT_NOTES.md](docs/i18n/pt/PATENT_NOTES.md), [docs/THIRD_PARTY.md](docs/i18n/pt/THIRD_PARTY.md), [docs/THIRD_PARTY_NOTICES.md](docs/THIRD_PARTY_NOTICES.md) | patentes, licenças das dependências |
| [CHANGELOG.md](docs/i18n/pt/CHANGELOG.md), [ROADMAP.md](docs/i18n/pt/ROADMAP.md), [FEATURES.md](docs/i18n/pt/FEATURES.md) | histórico, plano e status detalhado, matriz de recursos |
| [CLEANROOM.md](docs/i18n/pt/CLEANROOM.md), [LICENSING.md](docs/i18n/pt/LICENSING.md), [TRADEMARK.md](docs/i18n/pt/TRADEMARK.md), [CONTRIBUTING.md](docs/i18n/pt/CONTRIBUTING.md) | procedência, licenças, política de nome, regras de contribuição |

## Resumo das licenças

* **MPL-2.0** para as bibliotecas `osatura-core`, `osatura-capi`, `osatura-wasm`, `osatura-gpu` e para `scripts/`: você pode vinculá-las ao seu próprio programa, aberto ou fechado; as alterações nesses arquivos devem ser publicadas sob a MPL.
* **AGPL-3.0-or-later** para `osatura-cli` (a linha de comando e o estúdio) e `osatura-gui` (a janela em torno dele). Se você o modificar e permitir que outras pessoas interajam com ele por uma rede, a seção 13 exige que você ofereça a elas o seu código-fonte: `osatura studio --source-url URL` publica onde ele está; `/source` e `osatura --version` trazem a oferta.
* **CC-BY-4.0** para a documentação e os recursos sintéticos (inclusive o modelo sintético de detecção de rostos).
* **Seus projetos e renders continuam sendo seus.**
* O nome e o logotipo não são cobertos por estas licenças ([TRADEMARK.md](docs/i18n/pt/TRADEMARK.md)). O status de patentes não foi avaliado ([docs/PATENT_NOTES.md](docs/i18n/pt/PATENT_NOTES.md)). Uma licença comercial ou dupla para o código contribuído não é possível sem um contrato de licença de contribuidor, e nenhum existe.
* Crates de terceiros: 363 no inventário de 24 out 2026 (todas as plataformas e alvos juntos, o que inclui os crates exclusivos de Windows / macOS / Linux do aplicativo de janela; um dado binário contém um subconjunto), todos permissivos, exceto seis crates de copyleft fraco MPL-2.0: `smartstring` por trás do recurso opcional `scripting`, e cinco (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`) que somente o aplicativo de janela `osatura-gui` pode trazer ([docs/THIRD_PARTY.md](docs/i18n/pt/THIRD_PARTY.md)). O ffmpeg nunca é incluído.

Veja [LICENSE](LICENSE), [LICENSES/](LICENSES/) e [LICENSING.md](docs/i18n/pt/LICENSING.md). Nada disso é aconselhamento jurídico; a revisão por um advogado ainda está pendente.

## Contribuindo

Contribuições são bem-vindas sob a licença da parte que elas alteram. Leia [CONTRIBUTING.md](docs/i18n/pt/CONTRIBUTING.md) (todo commit precisa de uma linha `Signed-off-by`, DCO 1.1), [CLEANROOM.md](docs/i18n/pt/CLEANROOM.md) (implemente apenas a partir de fontes públicas; não copie de código proprietário ou com licença incompatível) e [docs/DEVELOPMENT.md](docs/i18n/pt/DEVELOPMENT.md) (convenções, checklist para um novo módulo, testes). Boas primeiras contribuições: uma correção a partir de [docs/LIMITATIONS.md](docs/i18n/pt/LIMITATIONS.md), uma execução real do checklist do WebAssembly, um teste em desktop real dos executáveis para Windows / macOS, um benchmark em GPU real.

Problemas de segurança: não abra uma issue pública; siga [docs/SECURITY.md](docs/i18n/pt/SECURITY.md) seção 10.

## Apoio

O OSATURA é desenvolvido por Alexandru-Ionuț Chiuță (Alexio). Se ele é útil para você, você pode apoiar o trabalho:

* Buy Me a Coffee: <https://buymeacoffee.com/echo.of.the.strings>
* Patreon: <https://patreon.com/c/alexio_tf>

Contato do mantenedor (também exibido por `osatura --version`): alexio@trom.tf.

## Agradecimentos

* Os autores dos artigos públicos e livros didáticos que os algoritmos seguem, citados por módulo em [CLEANROOM.md](docs/i18n/pt/CLEANROOM.md) (matemática de Bézier e splines, cinemática inversa, skinning por mistura linear, coordenadas de valor médio, ajuste de curvas e outros).
* As especificações abertas que o OSATURA lê e grava: SVG, OpenRaster, OCA, Lottie/dotLottie, glTF, APNG, WebM, OpenEXR, BVH, PNG, GIF.
* Os crates Rust em que ele se baseia, listados com suas licenças em [docs/THIRD_PARTY.md](docs/i18n/pt/THIRD_PARTY.md), em especial `tiny-skia`, `serde`, `png`, `zip`, `rhai` e `wgpu`.
* Todos que apoiam o projeto por meio dos links acima.
