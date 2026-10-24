<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Guia de desenvolvimento

*Para quem compila, altera, testa e lança versões do OSATURA. Documentação CC-BY-4.0. Tudo o que está aqui foi extraído da árvore de código-fonte, de `ROADMAP.md` seção 9, de [CI.md](CI.md), [CLEANROOM.md](CLEANROOM.md), [CONTRIBUTING.md](CONTRIBUTING.md) e [LICENSING.md](LICENSING.md); os comandos marcados com "run" foram executados durante a escrita desta página (Linux x86-64, rustc 1.97.0, registro offline).*

Conteúdo: [Ambiente](#1-ambiente) - [Comandos de build e teste](#2-comandos-de-build-e-teste) - [As suítes de teste](#3-as-suítes-de-teste) - [Convenções de código](#4-convenções-de-código) - [Checklist para um novo módulo ou comando](#5-checklist-para-um-novo-módulo-ou-comando) - [Regras clean-room e DCO](#6-regras-clean-room-e-dco) - [Checklist de lançamento](#7-checklist-de-lançamento) - [Integração contínua](#8-integração-contínua) - [Depuração](#9-depuração) - [Profiling](#10-profiling)

## 1. Ambiente

| Necessidade | Motivo | Observações |
|---|---|---|
| Rust (stable, workspace edition 2021) | tudo | desenvolvido com rustc 1.97.0; `Cargo.lock` está versionado, os builds são compatíveis com `--locked` |
| Um compilador C (`cc`) | o exemplo/teste da ABI C (opcional) | veja [C_ABI.md](C_ABI.md) |
| `ffmpeg` / `ffprobe` no `PATH` | exportação de vídeo, decodificação de áudio mp3/ogg, `dof-export` para .mp4/.webm, `soundtrack` | **nunca é incluído nem necessário para compilar**; os comandos que dependem dele avisam e falham sem mensagem de sucesso |
| Node.js >= 18 e `playwright-core` + Chromium | os cinco smoke tests do studio; o teste do wrapper WASM | padrões: `/opt/npm-tools/node_modules/playwright-core`, `/opt/pw-browsers/chromium-1194/chrome-linux/chrome`; substitua com `PLAYWRIGHT_CORE`, `CHROME` |
| `cargo-deny` | política de licenças/proibições/fontes | `scripts/licence-audit.sh` o executa quando instalado (`--no-deny` o ignora) |
| `python3` | `scripts/licence-audit.sh` gera a documentação com um script inline | executa com `python3 -I` |
| Um adaptador Vulkan ou GL | apenas para números em GPU real de `osatura-gpu` | os testes recorrem a um adaptador de software; veja a seção 2 |
| `rustup target wasm32-unknown-unknown`, `wasm-opt` | apenas para [WASM.md](WASM.md) | nunca instalados no ambiente de desenvolvimento; o CI os instala |

Estrutura do workspace (veja [ARCHITECTURE.md](ARCHITECTURE.md) para o projeto):

| Crate | Licença | Função |
|---|---|---|
| `osatura-core` | MPL-2.0 | motor, formatos, ferramentas, a fachada estável `api` ([API.md](API.md)); features: `scripting` (padrão, Rhai) |
| `osatura-cli` | AGPL-3.0-or-later | o binário `osatura`, servidor do studio, servidor de colaboração; features: `scripting` (padrão), `gpu` |
| `osatura-capi` | MPL-2.0 | ABI C ([C_ABI.md](C_ABI.md)) |
| `osatura-wasm` | MPL-2.0 | ABI WebAssembly + wrapper JS ([WASM.md](WASM.md)) |
| `osatura-gpu` | MPL-2.0 | renderizador wgpu opcional; **não** é membro padrão |
| `osatura-gui` | AGPL-3.0-or-later | `osatura-studio`, o estúdio em uma janela nativa (`tao` + `wry`, somente Windows e macOS; um stub nos demais); **não** é membro padrão, `publish = false` |

Arquivos de nível de repositório: `ROADMAP.md` (a seção 9 é o registro datado de funcionalidades e revisões), `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `REUSE.toml`, `deny.toml`, `LICENSES/`, `scripts/`.

## 2. Comandos de build e teste

```sh
cargo build                                  # default members: core, cli, capi, wasm (gpu is opt-in)
cargo test --workspace                       # run: 1343 tests + 19 doc-tests without gpu and gui (section 3)
cargo test -p osatura-core                  # one crate (the engine and the facade tests)
cargo test -p osatura-core api::            # a module or a name filter
cargo test -p osatura-cli --test e2e        # one integration test file
cargo test --doc -p osatura-core            # doc-tests only (the facade examples)
cargo test -p osatura-cli --test spdx_audit # SPDX header audit (0.01 s)
cargo clippy --workspace --all-targets -- -D warnings   # zero warnings (gated in CI since wave 8)
scripts/licence-audit.sh                     # regenerate THIRD_PARTY*, SPDX check, cargo deny
scripts/e2e.sh                               # the end-to-end pipeline in release mode (--debug for debug)
```

Features e partes opcionais:

```sh
cargo build -p osatura-cli --no-default-features         # no Rhai (no `script` command, no smartstring)
cargo build -p osatura-core --no-default-features        # what capi, wasm and gpu use
cargo build -p osatura-cli --features gpu                # adds `render-gpu` (wgpu)
cargo test  -p osatura-gpu                               # GPU tests (31: 18 unit + 13 parity); NOT in the workspace default
OSATURA_GPU_BACKEND=vulkan osatura render-gpu ...     # restrict the wgpu backend: vulkan or gl
```

Regras práticas: use builds de debug para as suítes de teste (o repositório foi desenvolvido pensando em espaço em disco: não copie `target/`); use `--release` apenas para questões de tempo. O snapshot da fachada é aprovado com `OSATURA_BLESS=1 cargo test -p osatura-core api::` (seção 5). `OSATURA_UI_CONFIG` seleciona nos testes o arquivo de configurações de interface do studio, `OSATURA_AUTHOR` o nome do autor nos commits do controle de versão, e `OSATURA_E2E_TIMINGS=<file>` faz o teste e2e gravar os tempos por etapa.

## 3. As suítes de teste

Contagens de `ROADMAP.md` seção 9 e de uma execução completa de `cargo test --workspace --offline` em 22 Oct 2026 para esta página (perfil debug, em uma máquina compartilhada com outros jobs, portanto os tempos são indicativos e **serão mais lentos do que em uma máquina ociosa**):

| Suíte | Testes | Tempo (esta execução) | O que cobre |
|---|---:|---:|---|
| `osatura-core` unit (`src/lib.rs`) | 1167 | 349 s | todos os módulos; inclui `api::tests::api_surface_snapshot` |
| `osatura-core/tests/prelude_surface.rs` | 6 | 0.7 s | a fachada vista de fora do crate (`#[non_exhaustive]`, mapeamento de erros, um subconjunto representativo de chamadas) |
| `osatura-cli` unit (`src/usage_tests.rs`, `src/args.rs`, studio, collab, http) | 80 | 42 s | análise de opções, USAGE vs parser, servidor HTTP, política, log de colaboração |
| `tests/e2e.rs` | 2 | 214 s | todo o pipeline pelo binário real (cerca de 19 s em um build debug ocioso e 40 s em release segundo o ROADMAP; aqui a máquina estava carregada) |
| `tests/fixes.rs`, `tests/wave4.rs`, `tests/robust.rs` | 7, 15, 10 | 19 s, 1.2 s, 1.2 s | regressões das ondas de revisão e testes com entradas hostis |
| `tests/doc_mismatch.rs` | 20 | 1.8 s | regressões entre documentação e comportamento: saídas atômicas, opções estritas, a matriz de aceitação de `--json`/`--quiet`/`--allow-empty`, regras de escala de `golden` |
| `tests/source_offer.rs` | 3 | 0.2 s | oferta de `/source` da seção 13 da AGPL |
| `tests/face_cli.rs` | 1 | 0.8 s | CLI de rastreamento facial |
| `osatura-core/tests/wasm_safety.rs` | 2 | 0.1 s | nenhum uso direto de `Instant` / `SystemTime` / `process::id` / `thread::spawn` fora de `clock.rs` (varredura do código-fonte) |
| `osatura-cli/tests/wave7.rs` | 7 | 7 s | onda 7: opções por subcomando, `follow` / `make-loop`, `color-space`, `collab-token`, arquivo de segredo do studio, intervalo de quadros de `dof` / `render-hdr` / `fx` |
| `tests/spdx_audit.rs` | 2 | 0.01 s | todo `.rs/.js/.mjs/.sh` tem uma linha SPDX igual à licença do seu crate; REUSE.toml cobre o restante |
| `osatura-capi/tests/abi.rs` | 7 | 1.1 s | a ABI C pelo lado Rust (smoke test em C: `osatura-capi/tests/smoke.c`, compilado manualmente, veja [C_ABI.md](C_ABI.md)) |
| `osatura-wasm` unit | 8 | 1.6 s | as funções da ABI no host |
| `osatura-gpu` unit + `tests/parity.rs` | 18 + 13 | 0.2 s + 6.5 s | paridade GPU vs CPU (adaptador de software quando não há GPU) |
| Doc-tests (`osatura-core`) | 19 | 171 s (cada um compila um programa) | os exemplos da fachada (`api/`, `lib.rs`); `osatura-capi`, `osatura-wasm` não têm nenhum |

Total: 1343 testes nos binários de unidade e de integração e 19 doc-tests, contados em 24 Oct 2026 com `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` (os 1368 da frase seguinte incluíam também os testes do crate de GPU; recontado a partir de uma execução completa de `cargo test --workspace` em 22 Oct 2026, após a onda 7; os tempos por suíte acima vêm de uma execução anterior), mais 19 doc-tests; o `cargo test --workspace --offline` completo levou 836 s na máquina compartilhada.

Suítes fora do `cargo test`:

* **Smoke tests do studio (Chromium).** Cinco scripts Node controlam o studio real em Chromium headless com `playwright-core`: `studio_ui_smoke.js` (128 verificações), `studio_collab_smoke.js` (60, duas páginas em uma máquina), `studio_overlays_smoke.js` (89), `studio_suggest_smoke.js` (35) e `studio_demo_smoke.js` (25: a demonstração guiada carrega os 13 passos, nunca grava, restaura o projeto byte a byte e recupera uma aba fechada); os cinco passaram em 24 out 2026 (contagens conforme impressas pelos scripts). Eles **não** são executados por `cargo test` e não fazem parte do CI por padrão. Execução:

  ```sh
  cargo build -p osatura-cli
  node osatura-cli/tests/studio_ui_smoke.js        # same for collab / overlays / suggest / demo
  ```

  Ambiente: `OSATURA_BIN` (padrão `target/debug/osatura`), `PLAYWRIGHT_CORE`, `CHROME` e, para o script de overlays, `SHOTS` (diretório de capturas de tela). Os scripts apagam seus arquivos temporários.
* **Teste do wrapper WASM.** `node osatura-wasm/js/test.mjs` (apenas mock, a menos que você passe o caminho de um projeto e tenha compilado o módulo; veja [WASM.md](WASM.md)).
* **Auditoria de licenças.** `scripts/licence-audit.sh`: reconstrói `docs/THIRD_PARTY.md` e `docs/THIRD_PARTY_NOTICES.md` a partir de `cargo metadata --locked --all-features` (dependências normais e de build, sem as de desenvolvimento), confere a linha SPDX de cada arquivo de código-fonte com a `license` do seu crate e depois executa `cargo deny check licenses bans sources`, a menos que `--no-deny` seja usado. `deny.toml` permite licenças permissivas e MPL-2.0 somente para crates nomeados.
* **clippy.** `cargo clippy --workspace --all-targets -- -D warnings` termina com código 0 e sem avisos (onda 8) e é obrigatório no CI. Quando uma regra do lint não se aplica ao caso, o código traz um `#[allow(clippy::...)]` restrito com um comentário (laços de índice sobre arrays paralelos, `too_many_arguments` em assinaturas estáveis, `manual_clamp` onde o tratamento de NaN difere, nomes inerentes `next` / `mul` que fazem parte da API pública); não adicione allows no nível do crate.
* **Integração com o CI.** O repositório inclui `osatura ci` para verificações de *animação* ([CI.md](CI.md)); é algo diferente do CI do código. O projeto não tem configuração de CI de código na árvore; a seção 8 apresenta uma receita mínima.

Adicionando testes: coloque os testes do motor junto ao código (`#[cfg(test)]`), os testes de comportamento da CLI em `osatura-cli/tests/`, e todo bug corrigido ganha um teste de regressão (as ondas de revisão em [CHANGELOG.md](CHANGELOG.md) listam o que foi corrigido e onde). Os testes não devem deixar arquivos temporários para trás (use drop guards) e não devem depender do relógio nem da rede. Orçamentos de tempo são testados com um relógio de teste determinístico.

## 4. Convenções de código

* **Nada de panics na entrada.** Tudo o que lê um projeto, um arquivo, um argumento ou um byte de rede retorna um erro. Indexação, fatiamento, `unwrap` e estouro aritmético sobre valores não confiáveis são bugs. A fachada (`api/`) também captura panics e retorna `ApiError::Internal` (somente onde o alvo faz unwinding), mas essa é a última linha de defesa, não um princípio de projeto.
* **Política de valores finitos.** Números vindos de arquivos e argumentos são verificados quanto a NaN e infinito e quanto a magnitudes razoáveis (os intervalos da fachada estão em [API.md](API.md): magnitudes até 1e7, fps em (0, 1000], quadros 1..=200000, stage até 32768, 4096 ossos, lado de renderização até 16384 px). Rejeite, não limite (clamp), a menos que a função esteja documentada como limitadora (os quadros são limitados à linha do tempo).
* **Verifique os tamanhos antes de alocar.** Buffers de pixels, tamanhos de malha e contagens de amostras são calculados com aritmética verificada e recusados antes de `vec![..]`.
* **Gravações atômicas.** Grave os resultados com `fsutil::write_atomic` (arquivo temporário no mesmo diretório, `fsync`, renomeação; recusa links simbólicos, a menos que `--follow-symlinks`; recorre a uma sobrescrita in-place com `fsync`, documentada como não atômica, quando o diretório não é gravável). Nunca use `std::fs::write` diretamente em um arquivo do usuário. Os diretórios temporários vêm de `fsutil` (modo 0700, arquivos 0600). Os bytes aleatórios vêm de `fsutil::random_bytes` (`/dev/urandom`).
* **Os assets passam por `assets::resolve`.** Caminhos relativos de imagens, normal maps e sons são resolvidos em relação ao diretório de assets do processo (`assets::default_dir`, definido a partir da pasta do projeto de entrada); caminhos absolutos e `..` são recusados, a menos que `--allow-external-assets`; arquivos de imagem são limitados a 64 MB e os de áudio a 256 MB. Os comandos que gravam um projeto derivado validam em relação ao diretório do projeto de *entrada* e reescrevem ou avisam sobre caminhos relativos de assets (`write_json`).
* **Aleatoriedade determinística.** Todo comportamento aleatório (fuzzing, ruído, jitter, simulação) usa o gerador com seed do próprio motor, com uma opção `--seed`, e é reproduzível bit a bit. Nada de crate `rand`, nada de hora do sistema como seed.
* **Println e pipes fechados.** A CLI sobrepõe `println!`/`print!` com macros que saem silenciosamente em um pipe fechado (`osatura ... | head` não pode dar panic). Continue usando-as em módulos declarados depois das macros em `main.rs`.
* **Os erros dizem o que fazer.** As mensagens informam o que estava errado, qual valor, e o intervalo permitido ou a correção (os testes da fachada verificam esse estilo).
<!-- REUSE-IgnoreStart -->
* **Cabeçalhos SPDX.** Todo arquivo `.rs`, `.js`, `.mjs` e `.sh` começa com `SPDX-License-Identifier:` igual à licença do seu crate (MPL-2.0 para core/capi/wasm/gpu/scripts, AGPL-3.0-or-later para a CLI) e uma linha de copyright; os documentos Markdown trazem o comentário de cabeçalho CC-BY-4.0, como este arquivo. Os demais arquivos (JSON, imagens, os textos de licença) são cobertos por `REUSE.toml`; ao adicionar um novo padrão de caminho, inclua-o ali. Caso contrário, `cargo test -p osatura-cli --test spdx_audit` falha.
<!-- REUSE-IgnoreEnd -->
* **Dependências.** Poucas, em Rust puro, com licença permissiva. Um novo crate exige uma verificação em `deny.toml` (`scripts/licence-audit.sh`), uma linha em `docs/THIRD_PARTY.md` (regenerado) e uma justificativa na entrada do ROADMAP. `osatura-core` é compilado sem features padrão por capi, wasm, gpu e pela CLI, portanto qualquer coisa pesada fica atrás de uma feature.
* **Fachada primeiro.** Toda nova capacidade do motor visível ao usuário que outras linguagens devam alcançar vai em `osatura-core/src/api/` com uma linha de snapshot, um doc-test e um teste de unidade; as camadas C e wasm apenas encapsulam a fachada.

## 5. Checklist para um novo módulo ou comando

1. **Módulo** em `osatura-core/src/<name>.rs`, registrado em `lib.rs`; cabeçalho SPDX; testes de unidade (válidos, limites, entradas hostis, determinismo); sem panics.
2. **Comando da CLI**: uma entrada em `USAGE` (grafia exata de cada opção) em `osatura-cli/src/usage.rs`, o braço do dispatch em um de `osatura-cli/src/cmds_1.rs` a `cmds_6.rs` (ou em `main.rs` para `ui`) e as tabelas de opções em `osatura-cli/src/args.rs`: toda opção que recebe valor deve constar em `VALUE_OPTS`; as flags que não recebem valor, na entrada `CMD_OPTS` do comando onde a tabela global estaria errada (por exemplo, `ci --prune` é uma flag, `export-oca --layers` e `autotrace --png` recebem valores, `keyedit --free` é uma flag). Os testes de unidade USAGE-versus-parser falham se o texto e o parser divergirem. Suporte as flags globais (`--json`, `--quiet`, `--allow-empty`, `--allow-external-assets`, `--follow-symlinks`, `--max-input-bytes`), `<command> --help` e `--key=value`.
3. **Gravar a saída** somente com `fsutil::write_atomic` ou `write_json`; os comandos que não produzem nada terminam com código diferente de zero, a menos que `--allow-empty`.
4. **Documentação**: o comando em [CLI_REFERENCE.md](CLI_REFERENCE.md), se você adicionar um, uma menção em um guia onde for relevante, e a entrada em `ROADMAP.md` seção 9 (o que foi adicionado, o que *não* foi feito: essa lista alimenta [LIMITATIONS.md](LIMITATIONS.md)).
5. **Linha em CLEANROOM.md**: para cada novo algoritmo, registre a fonte pública a partir da qual foi implementado (livro-texto, norma, artigo) e a derivação independente; sem linha, sem merge.
6. **Reaprovar a superfície da fachada** se `api/` mudou: execute `OSATURA_BLESS=1 cargo test -p osatura-core api::`, revise o diff de `osatura-core/src/api_surface.txt` (uma linha removida ou alterada é uma mudança incompatível e exige uma versão major da fachada; uma linha adicionada é uma minor) e depois execute os testes novamente sem a variável. Atualize [API.md](API.md) e os wrappers C/WASM e seus headers, se devem expô-la.
7. **End-to-end**: adicione o comando ao pipeline e2e se ele pertencer ao caminho de produção; adicione uma verificação smoke aos scripts do studio se ele tiver interface.
8. **Execute**: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/licence-audit.sh` (quando dependências ou cabeçalhos mudaram).

## 6. Regras clean-room e DCO

O projeto é uma implementação independente. Em resumo (o texto vinculante é [CLEANROOM.md](CLEANROOM.md) e [CONTRIBUTING.md](CONTRIBUTING.md)):

* Implemente a partir de normas públicas, livros-texto e artigos, e cite-os. Não leia, descompile, copie nem faça engenharia reversa de código, arquivos ou interfaces de ferramentas proprietárias de animação, e não leia nem cole código-fonte de ferramentas concorrentes enquanto escreve uma funcionalidade. Não cole código de projetos com licença incompatível.
* Registre cada novo método na tabela de `CLEANROOM.md` com sua fonte pública.
* As notas sobre patentes ficam em [PATENT_NOTES.md](PATENT_NOTES.md); uma funcionalidade que toque uma área sensível a patentes é discutida ali primeiro. Isto não é aconselhamento jurídico.
* As contribuições são aceitas sob a licença da parte que alteram (entrada igual à saída): MPL-2.0 para as bibliotecas e scripts, AGPL-3.0-or-later para `osatura-cli`, CC-BY-4.0 para a documentação e os assets sintéticos.
* **Sign-off.** Todo commit precisa de `Signed-off-by: Name <email>` (`git commit -s`), certificando o Developer Certificate of Origin 1.1 (texto em CONTRIBUTING.md). O sign-off certifica apenas a origem: ele **não** concede nenhum direito de relicenciamento, e nenhuma licença comercial ou dupla para o código contribuído é possível enquanto não existir um CLA.
* O nome e o logotipo são cobertos por [TRADEMARK.md](TRADEMARK.md).

## 7. Checklist de lançamento

Os primeiros builds são preparados como o pré-lançamento em rascunho `v0.1.0-alpha.3` (todos os crates continuam em 0.1.0; os rótulos de versão "v0.5/v0.6/v0.7" no README, FEATURES e ROADMAP são níveis de funcionalidade, não versões de crates; veja [CHANGELOG.md](CHANGELOG.md)). Para gerar um:

1. **Versão.** Decida a versão de cada crate. A fachada tem sua própria versão (`API_VERSION`, atualmente 1.4.0: major na remoção ou alteração de uma linha da superfície, minor na adição) e `OSATURA_ABI_VERSION` / `arm_abi_version()` (ambos 1) mudam somente em uma mudança incompatível da ABI C ou wasm. Alinhe as versões em `Cargo.toml`, a saída de `osatura --version`, a caixa de status do README e o CHANGELOG.
2. **Gates verdes.** `cargo test --workspace`, `cargo test --doc -p osatura-core`, `cargo test -p osatura-cli --test spdx_audit`, `cargo clippy --workspace --all-targets -- -D warnings`, os cinco smoke tests em Chromium, `scripts/e2e.sh` (release), `cargo test -p osatura-gpu` se o código de GPU mudou.
3. **Avisos de terceiros.** Execute `scripts/licence-audit.sh`: ele regenera `docs/THIRD_PARTY.md` (inventário) e `docs/THIRD_PARTY_NOTICES.md` (textos de licença de tudo o que é distribuído), confere as linhas SPDX e executa `cargo deny`. Faça commit dos arquivos regenerados. Uma distribuição binária deve incluir o pacote de avisos junto ao binário.
4. **Arquivos de licença.** `LICENSE`, `LICENSES/` (textos de todas as licenças usadas) e `REUSE.toml` estão sincronizados com [LICENSING.md](LICENSING.md); a atribuição CC-BY-4.0 da documentação está intacta.
5. **Sem ffmpeg.** Não inclua, não faça link nem baixe o ffmpeg nos artefatos de lançamento. Ele é detectado no `PATH` em tempo de execução e a documentação diz que o usuário o instala.
6. **Oferta de código-fonte da AGPL.** O binário de `osatura-cli` serve sua própria oferta de código-fonte em `/source` (seção 13 da AGPL) e imprime um aviso com `--version`; `release.yml` define `OSATURA_SOURCE_URL` em tempo de compilação como `<server>/<repository>/tree/<commit>`, de modo que os binários de release nomeiam o código-fonte exato a partir do qual foram compilados (o teste de fumaça do workflow verifica isso em `--version`; um `cargo build` simples não tem endereço embutido); quem opera servidores do studio modificados define `--source-url` para o local onde está o seu código-fonte modificado. Testes: `tests/source_offer.rs`. Em um lançamento binário, disponibilize o código-fonte correspondente (tag, tarball) na URL que os usuários possam acessar.
7. **Documentação.** Atualize a caixa de status do README, [LIMITATIONS.md](LIMITATIONS.md) e o CHANGELOG (mova *Unreleased* para a nova versão, com data).
8. **Gere o rascunho no GitHub.** `.github/workflows/release.yml` compila os arquivos para Windows, macOS e Linux e o módulo web a cada push em `main`; um commit em `main` cuja mensagem contenha `[release vX.Y.Z-label]` (ou uma tag `v*` enviada) também cria um *pré-lançamento em rascunho* com os arquivos e `SHA256SUMS.txt` (um rascunho antigo com a mesma tag é substituído). Revise o rascunho e depois publique-o manualmente.
9. **Tag e publicação** com uma tag assinada; anexe checksums. Não publique crates com nomes que não sejam seus; as regras de TRADEMARK.md se aplicam a forks.

## 8. Integração contínua

Duas coisas diferentes compartilham a palavra (um terceiro workflow, `.github/workflows/release.yml`, compila os arquivos de release e o lançamento em rascunho, veja a seção 7):

* **CI de animação** (`osatura ci`): verificações de *projetos* (lint, goldens de renderização, desempenho, determinismo). Descrito por completo em [CI.md](CI.md): manifesto `osatura-ci.json`, limiares, goldens (`--update` versus `bless`), relatórios, exemplos de GitHub Actions e de shell puro (marcados lá como "example, not run").
* **CI do código** deste repositório: `.github/workflows/ci.yml` tem quatro tipos de job: `test` (matriz ubuntu / windows / macos: `cargo test --workspace --exclude osatura-gpu --locked` e clippy com `-D warnings`), `wasm32` (compila `osatura-wasm` para `wasm32-unknown-unknown`, copia o módulo para junto de `osatura.js` e executa `node osatura-wasm/js/test.mjs` com um projeto de demonstração, incluindo sua parte com o módulo real), `gpu parity` (informativo; é ignorado quando não existe adaptador, roda no adaptador DX12 por software do Windows) e `licences` (`cargo-deny check` e `reuse lint`). A toolchain é fixada (1.97.0) para que novos lints do clippy não quebrem o build. O job `test` também compila `osatura-gui` e executa `osatura-studio --selftest` (um autoteste real no Windows e no macOS). **Primeira execução totalmente verde: 2026-10-24** em runners hospedados pelo GitHub. Em caso de falha, os passos imprimem os testes que falharam como anotações do workflow (legíveis sem acesso aos logs). O Windows ignora os testes exclusivos de POSIX (symlinks, permissões, `/dev/zero`, separadores de caminho POSIX).

  ```sh
  cargo test --workspace --locked
  cargo test --doc -p osatura-core --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  scripts/licence-audit.sh            # needs python3 and (optional) cargo-deny
  ```

  Faça cache de `~/.cargo/registry` e `target/` entre as execuções, dê ao teste e2e vários minutos em builds debug e execute os smoke tests em Chromium somente em runners que tenham Chromium e `playwright-core`.

## 9. Depuração

* **Backtraces.** `RUST_BACKTRACE=1 osatura ...` para qualquer panic (um panic na entrada é um bug; reporte-o com o projeto que o provoca).
* **Saída legível por máquina.** `--json` nos comandos que o suportam; `--quiet` para scripts. Códigos de saída: 0 em caso de sucesso, 1 em caso de falha (verificação reprovada, orçamento não atendido, uso incorreto; run: um comando desconhecido e um argumento ausente terminam ambos com 1); `osatura ci` separa 1 (uma verificação falhou) de 2 (erro).
* **Inspecionar um projeto.** `osatura check`, `osatura lint` (código 1 em caso de erros) e `osatura diff a.json b.json` mostram a estrutura e os problemas sem renderizar; `osatura render project.json N out.png` renderiza um quadro, e `out.svg` gera uma saída vetorial que você pode abrir em um navegador (`--bones` mostra o esqueleto).
* **Studio.** Execute `osatura studio project.json --port 0` (imprime a porta real). As ferramentas de desenvolvedor do navegador mostram a API JSON: as requisições são `POST /api/...` simples com `Content-Type: application/json`. Um 421 significa um cabeçalho Host que não está em `--allow-host`; um 403 significa que uma verificação de Origin ou `Sec-Fetch-Site` falhou (use `--public-origin` atrás de um proxy); 415 refere-se ao tipo de conteúdo; 429 é o back-off do token.
* **Etapas da demonstração.** A demonstração guiada do Studio (e a demonstração passo a passo no site do projeto) são geradas a partir de `osatura demo-person` por `scripts/build-studio-tour.mjs` e `scripts/lib/person-stages.mjs`; a compilação falha se a demonstração concluída deixar de coincidir com os números citados nos textos (`EXPECT` na biblioteca). Depois de alterar `osatura-core/src/demo.rs`, execute o script de novo e recompile a CLI.
* **Log de colaboração.** O log de uma sessão compartilhada (`--collab-log`) é texto, com uma linha `<hash> <json>` por operação; uma cadeia quebrada é informada ao carregar.
* **Um teste falha só às vezes.** Procure o relógio, nomes de diretórios temporários e a ordem de iteração de `HashMap`; o código do motor é determinístico por construção, portanto falhas intermitentes são quase sempre falta de higiene do teste.
* **O snapshot da fachada falha.** Você alterou uma assinatura pública: veja a seção 5, item 6.
* **GPU.** `osatura render-gpu <project> <out.png> --adapter software|auto --bench N` (somente em builds com `--features gpu`) compara com o renderizador de CPU (um adaptador de software informa que seus tempos não significam nada); `OSATURA_GPU_BACKEND` seleciona o backend.
* **Sanitizers para a ABI C.** Compile `smoke.c` ou o seu próprio chamador com `-fsanitize=address,undefined` contra a biblioteca estática (feito durante o trabalho de documentação, sem problemas; veja [C_ABI.md](C_ABI.md)).

## 10. Profiling

* **Tempos por etapa do pipeline de produção:** `scripts/e2e.sh` imprime uma tabela (`OSATURA_E2E_TIMINGS` nomeia o arquivo); os números de release em `ROADMAP.md` seção 9 vêm dele.
* **GPU contra CPU:** `osatura render-gpu <project> <out.png> --bench N`.
* **Orçamento de tamanho e velocidade:** `osatura size-report`, `osatura export-fit --max-seconds` (o orçamento de tempo é verificado dentro dos laços de candidatos; uma chamada de ffmpeg ou de quantizador não pode ser interrompida).
* **Custo de renderização de um projeto:** `osatura ci` com um limiar de desempenho ([CI.md](CI.md)) registra o tempo por quadro na sua máquina; os tempos dependem da máquina, portanto compare na mesma máquina.
* **Ferramentas gerais:** compile com `--release` e use `perf record`/`perf report` ou `cargo flamegraph` (não fazem parte do projeto; instale separadamente). Os builds debug são cerca de uma ordem de grandeza mais lentos e não dizem nada sobre a velocidade real.
