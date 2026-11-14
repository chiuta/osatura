<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Política de assinatura de código

Assinatura de código gratuita fornecida pela [SignPath.io](https://about.signpath.io/), certificado da [SignPath Foundation](https://signpath.org/).

*Estado (25 out 2026): o projeto pediu, ou está prestes a pedir, um certificado gratuito da SignPath Foundation. Até uma versão dizer o contrário, os ficheiros para Windows **não** estão assinados com Authenticode; cada ficheiro de versão já traz uma assinatura Sigstore e uma atestação de proveniência de compilação do GitHub (veja [Verificação](#verificação)).*

## O que é assinado

Só são assinados os ficheiros compilados a partir deste repositório ([github.com/chiuta/osatura](https://github.com/chiuta/osatura)) pelo seu fluxo de publicação ([`.github/workflows/release.yml`](../../../.github/workflows/release.yml)) em runners alojados pelo GitHub. Nada compilado num computador pessoal é alguma vez assinado, e nenhum binário de terceiros é assinado.

| Ficheiro (em `osatura-windows-x86_64.zip`) | O que é | Licença |
|---|---|---|
| `osatura.exe` | ferramenta de linha de comandos e estúdio no navegador | AGPL-3.0-or-later |
| `osatura-studio.exe` | o estúdio numa janela nativa (WebView2) | AGPL-3.0-or-later |
| `lib/osatura_capi.dll` | biblioteca C | MPL-2.0 |

Cada ficheiro assinado tem o nome de produto `OSATURA` e a versão da publicação nas suas propriedades (incorporados por `build.rs` em cada crate e verificados pelo fluxo de trabalho antes de qualquer assinatura). A configuração de artefactos da SignPath é mantida em [`.signpath/artifact-configuration.xml`](../../../.signpath/artifact-configuration.xml).

## Funções da equipa

O projeto tem atualmente um único mantenedor, que acumula as três funções:

| Função | Quem |
|---|---|
| Committers e revisores | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |
| Aprovadores | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |

* As alterações de quem não tem acesso de escrita chegam como pull requests e são revistas por um committer antes de serem integradas ([CONTRIBUTING.md](CONTRIBUTING.md)).
* Um pedido de assinatura só é criado pelo fluxo de publicação para uma etiqueta de versão (`vMAJOR.MINOR.PATCH[-pre]`) cujo commit esteja em `main`, e cada pedido é aprovado manualmente na SignPath por um aprovador.
* Todas as pessoas com uma destas funções são obrigadas a usar autenticação multifator no GitHub e na SignPath (condição da SignPath Foundation; o mantenedor confirma-o para cada conta quando o certificado é concedido).
* Qualquer alteração a estas funções é registada neste ficheiro.

## Política de privacidade

Este programa não transferirá qualquer informação para outros sistemas em rede, a menos que o utilizador ou a pessoa que o instala ou opera o peça expressamente. Pormenores: [PRIVACY.md](PRIVACY.md).

## Alterações ao seu sistema e desinstalação

O OSATURA não tem instalador, não escreve nada no registo e não instala qualquer serviço, controlador ou entrada de arranque. Os poucos ficheiros que pode criar e a forma de remover tudo estão listados em [UNINSTALL.md](../../UNINSTALL.md).

## Licenças

Todo o código é de código aberto sob licenças aprovadas pela OSI: MPL-2.0 (bibliotecas) e AGPL-3.0-or-later (ferramenta de linha de comandos, estúdio e aplicação de janela); a documentação é CC-BY-4.0 ([COPYRIGHT](../../../COPYRIGHT), [LICENSING.md](LICENSING.md)). Os ficheiros assinados não contêm código proprietário, e o OSATURA não é oferecido sob qualquer licença comercial ou proprietária (sem licenciamento duplo).

## Verificação

* **Authenticode** (quando disponível): clique direito num ficheiro no Explorador do Windows > Propriedades > Assinaturas digitais; o signatário é a SignPath Foundation.
* **Origem de cada ficheiro de versão**: `gh attestation verify <file> -R chiuta/osatura`, ou `cosign verify-blob` com o pacote `.sigstore.json` ([VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md)).

## Comunicar um problema

Se pensa que um ficheiro assinado para o OSATURA é malicioso, não foi compilado a partir deste repositório ou que esta política foi violada, escreva para <alexio@trom.tf> ou abra uma issue; os relatórios de segurança seguem [SECURITY.md](SECURITY.md). Os problemas com o próprio certificado também podem ser comunicados à [SignPath Foundation](https://signpath.org/).
