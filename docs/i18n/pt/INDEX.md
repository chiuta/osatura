<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Índice da documentação do OSATURA

*Comece aqui para encontrar a página certa. Todos os documentos são Markdown em inglês sob CC-BY-4.0, exceto os arquivos de terceiros gerados (veja seus cabeçalhos).*

O OSATURA é um motor de rigging e animação vetorial 2D em Rust. É um projeto alpha (os primeiros builds são preparados como o pré-lançamento em rascunho `v0.1.0-alpha.3`; os crates continuam em 0.1.0); leia [LIMITATIONS.md](LIMITATIONS.md) antes de depender de um recurso.

Conteúdo: [Todos os documentos](#todos-os-documentos) - [Trilhas de leitura por público](#trilhas-de-leitura-por-público) - [Documentos na raiz do repositório](#documentos-na-raiz-do-repositório) - [Onde cada pergunta é respondida](#onde-cada-pergunta-é-respondida) - [Convenções](#convenções-usadas-nos-documentos)

## Todos os documentos

| Documento | O que é | Público |
|---|---|---|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Instalação, primeiro projeto, primeira renderização e exportação, primeira sessão no estúdio | todos, primeira leitura |
| [CLI_REFERENCE.md](CLI_REFERENCE.md) | Todos os comandos, opções, códigos de saída e exemplos do `osatura` | usuários, autores de scripts, autores de CI |
| [STUDIO.md](STUDIO.md) | O editor no navegador: painéis, atalhos, colaboração, pré-visualizações de scrub | animadores, riggers |
| [RIGGING_GUIDE.md](RIGGING_GUIDE.md) | Construção de um rig: ossos, IK, skinning, deformadores, drivers, máquinas de estados, layout | riggers, artistas técnicos |
| [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) | Chaves, easing, clipes e takes, física, lip-sync, áudio, escolhas de exportação | animadores |
| [TUTORIAL_SVG.md](../../TUTORIAL_SVG.md) | De um desenho vetorial a um GIF de caminhada: importar, montar por script, exportar (os comandos e as saídas são reais) | artistas |
| [COMPARISON.md](../../COMPARISON.md) | Uma comparação honesta e datada com outras ferramentas de animação 2D | avaliadores |
| [VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md) | Verificação dos arquivos de release: assinaturas Sigstore, proveniência do build, somas SHA-256 | todos que instalam |
| [UNINSTALL.md](../../UNINSTALL.md) | Remoção do OSATURA e de todos os arquivos que ele pode criar | todos que instalam |
| [../CODE_SIGNING_POLICY.md](../CODE_SIGNING_POLICY.md) · [../PRIVACY.md](../PRIVACY.md) | Política de assinatura de código (SignPath Foundation), papéis da equipe, privacidade | todos que instalam, mantenedores |
| [GOOD_FIRST_TASKS.md](../../GOOD_FIRST_TASKS.md) | Pequenas tarefas para uma primeira contribuição | novos colaboradores |
| [PROJECT_FORMAT.md](PROJECT_FORMAT.md) | O formato de projeto JSON, campo por campo | construtores de ferramentas, riggers que editam JSON |
| [EXPORT_FORMATS.md](EXPORT_FORMATS.md) | O que cada formato de exportação e de importação preserva e descarta | construtores de pipelines, integradores |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crates, módulos, fluxo de dados, projeto da avaliação e da renderização | contribuidores, revisores |
| [GLOSSARY.md](GLOSSARY.md) | Termos usados no motor, na documentação e na CLI | todos |
| [FAQ.md](FAQ.md) | Respostas curtas para perguntas comuns | todos |
| [API.md](API.md) | A facade Rust estável (versão 1.4.0): chamadas, erros, limites, exemplos | integradores Rust |
| [C_ABI.md](C_ABI.md) | A interface C (`osatura-capi`): funções, regras de memória, exemplos em C e Python | integradores C, C++, Python, Go, C# |
| [WASM.md](WASM.md) | Módulo WebAssembly e wrapper JavaScript. **Status: compilado e testado sob o Node no CI; executado em Chromium headless, ainda não no Firefox nem no Safari**; inclui o checklist para a primeira execução em um navegador | desenvolvedores web |
| [CI.md](CI.md) | CI de animação (`osatura ci`): manifesto, limiares, quadros de referência, relatórios, exemplos de pipeline | artistas técnicos, engenheiros de build |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Compilação, testes, convenções, checklist de novo módulo, checklist de lançamento, depuração | contribuidores, mantenedores |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | Passos do lado do proprietário antes e depois de publicar o lançamento em rascunho | proprietário |
| [SECURITY.md](SECURITY.md) | Modelo de ameaças, proteções integradas, lacunas conhecidas, relato de problemas, checklist de reforço de segurança | revisores de segurança, responsáveis por implantação |
| [LIMITATIONS.md](LIMITATIONS.md) | Lista numerada de limitações conhecidas com gravidade, alternativa e id do roadmap | todos que avaliam o OSATURA |
| [PATENT_NOTES.md](PATENT_NOTES.md) | Nota sobre o status de patentes (não é aconselhamento jurídico) | licenciados, advogados |
| [THIRD_PARTY.md](THIRD_PARTY.md) | Inventário gerado de crates e licenças de terceiros | licenciados, empacotadores |
| [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) | Textos de licença gerados das dependências, para distribuir com os binários | empacotadores |

## Trilhas de leitura por público

Cada trilha é uma ordem, não uma exigência. Pare quando tiver o que precisa.

**Novo usuário (quer ver funcionando).** [../README.md](../../../README.pt.md) (início rápido) -> [GETTING_STARTED.md](GETTING_STARTED.md) -> [STUDIO.md](STUDIO.md) -> [FAQ.md](FAQ.md) -> [LIMITATIONS.md](LIMITATIONS.md).

**Rigger.** GETTING_STARTED -> [RIGGING_GUIDE.md](RIGGING_GUIDE.md) -> [PROJECT_FORMAT.md](PROJECT_FORMAT.md) (para os campos que você edita à mão) -> [GLOSSARY.md](GLOSSARY.md) -> [CLI_REFERENCE.md](CLI_REFERENCE.md) (os comandos `lint`, `check`, `fuzz`, `autorig`, `machine`) -> [CI.md](CI.md) para proteger o rig.

**Animador.** GETTING_STARTED -> STUDIO -> [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) -> [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (escolha o formato de saída) -> CLI_REFERENCE (`export`, `export-fit`).

**Desenvolvedor de pipeline ou de ferramentas (scripts, outros formatos).** CLI_REFERENCE -> PROJECT_FORMAT -> EXPORT_FORMATS -> [CI.md](CI.md) -> [SECURITY.md](SECURITY.md) seções 4 a 7 (regras de recursos e arquivos).

**Integrador (que embarca o motor).** Visão geral de [ARCHITECTURE.md](ARCHITECTURE.md) -> escolha a camada: Rust ([API.md](API.md)), C e outras linguagens ([C_ABI.md](C_ABI.md)), navegador ([WASM.md](WASM.md), leia primeiro a caixa de status) -> [LIMITATIONS.md](LIMITATIONS.md) seções 1 e 8 -> [SECURITY.md](SECURITY.md) seção 8.

**Contribuidor.** [../CONTRIBUTING.md](CONTRIBUTING.md) (DCO) -> [DEVELOPMENT.md](DEVELOPMENT.md) -> ARCHITECTURE -> [../CLEANROOM.md](CLEANROOM.md) -> [../ROADMAP.md](ROADMAP.md) seção 9 (o que existe e o que não existe) -> LIMITATIONS (escolha algo para corrigir).

**Revisor de segurança ou responsável por implantação.** [SECURITY.md](SECURITY.md) -> [DEVELOPMENT.md](DEVELOPMENT.md) seção 3 (o que é testado) -> LIMITATIONS seções 2 e 11 -> `osatura-cli/src/http.rs`, `studio.rs`, `collab_srv.rs`, `osatura-core/src/assets.rs`, `fsutil.rs` -> limites de [CI.md](CI.md).

**Licenciado, empacotador ou advogado.** [../LICENSING.md](LICENSING.md) -> [../COPYRIGHT](../../../../../COPYRIGHT) e [../LICENSES/](../../../LICENSES/) -> [THIRD_PARTY.md](THIRD_PARTY.md) -> [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) -> [PATENT_NOTES.md](PATENT_NOTES.md) -> [../TRADEMARK.md](TRADEMARK.md) -> [../CLEANROOM.md](CLEANROOM.md) -> checklist de lançamento em [DEVELOPMENT.md](DEVELOPMENT.md) seção 7.

**Mantenedor que prepara um lançamento.** DEVELOPMENT seção 7 -> [../CHANGELOG.md](CHANGELOG.md) -> LIMITATIONS (atualizar) -> caixa de status do README.

## Documentos na raiz do repositório

| Arquivo | Finalidade |
|---|---|
| [../README.md](../../../README.pt.md), [../README.ro.md](../../../README.pt.md) | Visão geral, status, início rápido, resumo das licenças (inglês e romeno) |
| [../CHANGELOG.md](CHANGELOG.md) | Histórico reconstruído; formato Keep-a-Changelog |
| [../ROADMAP.md](ROADMAP.md) | Plano ordenado; **a seção 9 é o registro datado do que cada item faz e não faz** e a autoridade quando os documentos discordam |
| [../FEATURES.md](FEATURES.md) | Matriz de recursos em comparação com outras ferramentas (partes estão desatualizadas, veja LIMITATIONS) |
| [../CLEANROOM.md](CLEANROOM.md) | Registro de procedência e regras para a implementação independente |
| [../CONTRIBUTING.md](CONTRIBUTING.md) | Regras de contribuição, licença de entrada = licença de saída, DCO 1.1 |
| [../LICENSING.md](LICENSING.md), [../COPYRIGHT](../../../../../COPYRIGHT), [../LICENSES/](../../../LICENSES/) | Decisão de licença (Variante A), textos |
| [../TRADEMARK.md](TRADEMARK.md) | Política de nome e logotipo |
| `../REUSE.toml`, `../deny.toml` | Metadados de licença para arquivos que não são código-fonte; política de dependências |

## Onde cada pergunta é respondida

| Pergunta | Página |
|---|---|
| Como instalo e faço minha primeira animação? | GETTING_STARTED |
| O que faz o comando X? | CLI_REFERENCE |
| O que significa este campo do JSON? | PROJECT_FORMAT, GLOSSARY |
| Qual formato de exportação preserva máscaras / gradientes / áudio? | EXPORT_FORMATS |
| Posso usá-lo a partir de Python / C / do navegador? | C_ABI / WASM (status!) |
| É seguro expor o estúdio a uma rede? | SECURITY seção 11 |
| Por que X não funciona? Isso é conhecido? | LIMITATIONS, FAQ |
| Como ele é construído e testado? | DEVELOPMENT |
| Posso usá-lo comercialmente? Quais são as licenças? | README (licença), LICENSING, THIRD_PARTY |
| O que mudou? | CHANGELOG, ROADMAP seção 9 |

## Convenções usadas nos documentos

* **"Verificado"** significa que um comando ou programa foi executado enquanto os documentos eram escritos, em um ambiente com Linux x86-64, sem GPU, sem webcam e apenas com Chromium headless. **"Não testado"** significa exatamente isso. As afirmações sobre WebAssembly em um motor real são sempre não verificadas.
* Os exemplos de código em API.md e C_ABI.md são os programas que foram compilados e executados; os exemplos em WASM.md foram executados apenas contra o mock em JavaScript.
* A seção 9 do ROADMAP é a autoridade sobre o que um recurso faz e não faz. Se um documento discordar dela ou do código, o código prevalece; por favor, relate a diferença.
* Os caminhos são relativos à raiz do repositório, a menos que comecem com `docs/`.
* Os números (contagens de testes, limites) vêm do código-fonte ou de uma execução em 22 out 2026 e podem ficar desatualizados após a próxima alteração; o código-fonte é a autoridade.
