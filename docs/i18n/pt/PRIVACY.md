<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Privacidade

Este programa não transferirá qualquer informação para outros sistemas em rede, a menos que o utilizador ou a pessoa que o instala ou opera o peça expressamente.

Em pormenor, para `osatura`, `osatura-studio`, a biblioteca C e o módulo WebAssembly:

* **Sem telemetria, análise de utilização, relatórios de falhas, publicidade nem verificações automáticas de atualizações.** Os programas não contêm código que contacte um servidor por iniciativa própria.
* **Um único comando contacta um servidor, apenas quando você o escreve.** `osatura update-check` faz um único pedido HTTPS anónimo ao GitHub (`api.github.com`) para obter a lista pública de versões, usando o programa `curl` do seu sistema; o GitHub vê o seu endereço IP e esse pedido, de acordo com a [declaração de privacidade do GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). Nada é descarregado ou instalado e nada é enviado sobre você ou sobre os seus projetos. Para que o GitHub não veja o seu endereço, acrescente `--proxy-preset tor` (ou `tor-browser`, `warp`) ou `--proxy-url` com o seu próprio proxy; veja [CLI_REFERENCE.md](CLI_REFERENCE.md).
* **O estúdio é um servidor local.** `osatura studio` e `osatura-studio` escutam por omissão em `127.0.0.1` (apenas este computador). Só são acessíveis a partir de outras máquinas se você próprio os iniciar com `--bind` noutro endereço ([SECURITY.md](SECURITY.md)). A página que servem não carrega nada da Internet (a sua Content-Security-Policy só permite o servidor local).
* **Ligações em que clica.** O estúdio e a documentação contêm ligações (por exemplo para o GitHub ou para uma página de donativos). Seguir uma abre esse site no seu navegador, sob a política de privacidade desse site.
* **Programas externos.** `ffmpeg` só é iniciado para vídeo e alguns formatos de áudio, apenas quando você pede essa exportação, e trabalha sobre ficheiros locais.
* **As sessões partilhadas guardam um histórico.** Com `--collab-log`, o estúdio escreve cada edição da sessão, com os nomes dos participantes, num ficheiro à sua escolha; com tokens por participante, os nomes são os que você criou. Trate esse ficheiro como dados pessoais dos participantes e apague-o quando a sessão terminar ([SECURITY.md](SECURITY.md), [UNINSTALL.md](../../UNINSTALL.md)). As ferramentas de pontos de referência faciais escrevem os resultados apenas nos ficheiros que você indicar.
* **Os seus ficheiros continuam seus.** Projetos, renderizações e definições são escritos apenas onde você disser ao programa para os escrever, além do pequeno ficheiro de definições indicado em [UNINSTALL.md](../../UNINSTALL.md).

O sítio do projeto (<https://osatura.chiuta.ro>) é uma página estática alojada no GitHub Pages; aplica-se-lhe a declaração de privacidade do próprio GitHub.

Questões: <alexio@trom.tf>.
