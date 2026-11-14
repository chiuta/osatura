<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Confidentialité

Ce programme ne transférera aucune information vers d’autres systèmes en réseau, sauf demande explicite de l’utilisateur ou de la personne qui l’installe ou l’exploite.

En détail, pour `osatura`, `osatura-studio`, la bibliothèque C et le module WebAssembly :

* **Aucune télémétrie, analyse d’usage, remontée de plantages, publicité ni vérification automatique de mises à jour.** Les programmes ne contiennent aucun code qui contacte un serveur de lui-même.
* **Une seule commande contacte un serveur, uniquement quand vous la saisissez.** `osatura update-check` envoie une seule requête HTTPS anonyme à GitHub (`api.github.com`) pour obtenir la liste publique des versions, via le programme `curl` de votre système ; GitHub voit votre adresse IP et cette requête, conformément à la [déclaration de confidentialité de GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). Rien n’est téléchargé ni installé, et rien n’est envoyé à votre sujet ou au sujet de vos projets. Pour que GitHub ne voie pas votre adresse, ajoutez `--proxy-preset tor` (ou `tor-browser`, `warp`) ou `--proxy-url` avec votre propre proxy ; voir [CLI_REFERENCE.md](CLI_REFERENCE.md).
* **Le studio est un serveur local.** `osatura studio` et `osatura-studio` écoutent par défaut sur `127.0.0.1` (cet ordinateur uniquement). Ils ne sont joignables depuis d’autres machines que si vous les lancez vous-même avec `--bind` sur une autre adresse ([SECURITY.md](SECURITY.md)). La page qu’ils servent ne charge rien depuis Internet (sa Content-Security-Policy n’autorise que le serveur local).
* **Les liens sur lesquels vous cliquez.** Le studio et la documentation contiennent des liens (par exemple vers GitHub ou une page de dons). Suivre l’un d’eux ouvre ce site dans votre navigateur, sous la politique de confidentialité de ce site.
* **Programmes externes.** `ffmpeg` n’est lancé que pour la vidéo et certains formats audio, uniquement lorsque vous demandez un tel export, et travaille sur des fichiers locaux.
* **Les sessions partagées conservent un historique.** Avec `--collab-log`, le studio écrit chaque modification de la session, avec les noms des participants, dans un fichier de votre choix ; avec des jetons par participant, les noms sont ceux que vous avez créés. Traitez ce fichier comme des données personnelles des participants et supprimez-le à la fin de la session ([SECURITY.md](SECURITY.md), [UNINSTALL.md](../../UNINSTALL.md)). Les outils de repères faciaux n’écrivent leurs résultats que dans les fichiers que vous nommez.
* **Vos fichiers restent les vôtres.** Les projets, rendus et réglages ne sont écrits que là où vous indiquez au programme de les écrire, plus le petit fichier de réglages mentionné dans [UNINSTALL.md](../../UNINSTALL.md).

Le site du projet (<https://osatura.chiuta.ro>) est une page statique hébergée par GitHub Pages ; la déclaration de confidentialité propre à GitHub s’y applique.

Questions : <alexio@trom.tf>.
