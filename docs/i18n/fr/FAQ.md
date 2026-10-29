<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Foire aux questions

*Des réponses courtes, chacune tirée de quelque chose qui a été exécuté ou lu dans l'arborescence des sources et dans les autres documents. Documentation CC-BY-4.0. Lorsqu'une réponse dépend d'une limite, elle l'indique et renvoie à [LIMITATIONS.md](LIMITATIONS.md). Les chaînes d'erreur de la dernière section ont été relevées sur un vrai build ; elles sont citées telles quelles, afin que vous puissiez rechercher dans cette page le texte que vous voyez.*

Sommaire : [Installation et configuration](#installation-et-configuration) - [ffmpeg](#ffmpeg) - [Performances](#performances) - [Déterminisme](#déterminisme) - [Licences et vos sorties](#licences-et-vos-sorties) - [Sécurité des fichiers et chemins des assets](#sécurité-des-fichiers-et-chemins-des-assets) - [Pourquoi la sortie doit différer de l'entrée](#pourquoi-la-sortie-doit-différer-de-lentrée) - [Rigging et animation](#rigging-et-animation) - [Studio](#studio) - [Visage, parole et capture de mouvement](#visage-parole-et-capture-de-mouvement) - [Bugs, sécurité et contributions](#bugs-sécurité-et-contributions) - [Limitations](#limitations) - [Comparaisons avec d'autres outils](#comparaisons-avec-dautres-outils) - [Tableau de dépannage](#tableau-de-dépannage)

---

## Installation et configuration

**1. Comment installer OSATURA ?**
Des archives précompilées (Windows x86-64, macOS arm64, Linux x86-64, et le module WebAssembly) sont préparées comme la préversion en brouillon `v0.1.0-alpha.3` sur <https://github.com/chiuta/osatura/releases> ; elles deviennent téléchargeables quand le propriétaire publie le brouillon, et elles ne sont pas signées (SmartScreen affiche « Éditeur inconnu » ; macOS demande une approbation). Il n'y a pas d'installateur : décompressez l'archive et lancez `osatura`. Ou compilez-le depuis les sources avec une chaîne d'outils Rust stable récente : `cargo build -p osatura-cli` produit `target/debug/osatura` ; ajoutez `--release` pour obtenir `target/release/osatura`. Voir [DEVELOPMENT.md](DEVELOPMENT.md).

**2. Quelles plateformes fonctionnent ?**
Linux x86-64 est l'endroit où le projet a été développé et éprouvé. Sur GitHub Actions, la suite de tests et clippy passent aussi sous macOS (arm64) et Windows (x86-64), et les binaires de release pour ces trois plateformes y sont construits ; mais personne n'a encore utilisé les exécutables Windows ou macOS sur un vrai bureau, ils sont donc « testés par la CI », pas « pris en charge » ([LIMITATIONS.md](LIMITATIONS.md), section 1).

**3. Existe-t-il un installateur graphique ou une application ?**
Il n'y a pas d'installateur. La ligne de commande est le produit. Le studio est une page web locale servie par `osatura studio` ; sous Windows, un double-clic sur `osatura.exe` le démarre et ouvre votre navigateur. Les archives Windows et macOS contiennent aussi `osatura-studio`, une petite fenêtre native autour du même studio (elle n'a été que compilée et auto-testée en CI) ; voir [STUDIO.md](STUDIO.md) et [GETTING_STARTED.md](GETTING_STARTED.md).

**4. Comment démarrer rapidement ?**
Suivez [GETTING_STARTED.md](GETTING_STARTED.md) : il construit un personnage animé à partir d'un projet vide, chaque étape ayant été exécutée, et se termine par un GIF.

**5. Puis-je compiler sans le moteur de script ?**
Oui : `cargo build -p osatura-cli --no-default-features` laisse de côté Rhai, la commande `script` est donc absente (voir [DEVELOPMENT.md](DEVELOPMENT.md)). La fonctionnalité `scripting` est le seul endroit par lequel entre un crate à copyleft faible (smartstring) ; voir [THIRD_PARTY.md](THIRD_PARTY.md).

**6. Faut-il une connexion réseau ou un compte ?**
Non. Le programme n'a besoin ni de compte ni de réseau pour fonctionner (le studio n'a pas de comptes). Par défaut, le studio n'écoute que sur votre propre machine (voir la question 51).

**7. Où le studio conserve-t-il ses réglages ?**
Dans `$OSATURA_UI_CONFIG` s'il est défini, sinon dans `$XDG_CONFIG_HOME/osatura/ui.json`, sinon sous Windows dans `%APPDATA%\OSATURA\ui.json`, sinon dans `~/.config/osatura/ui.json`. Vos projets n'en sont jamais modifiés.

---

## ffmpeg

**8. Ai-je besoin de ffmpeg ?**
Seulement pour l'export MP4 et WebM, pour `sound mix`, pour l'audio mp3/ogg et pour `face-track` sur un fichier vidéo. L'export en GIF, APNG, HTML, séquences PNG, SVG, glTF et JSON n'a besoin de rien d'autre. La liste complète figure dans [CLI_REFERENCE.md](CLI_REFERENCE.md), section 1.6.

**9. ffmpeg est-il fourni avec le projet ?**
Jamais. OSATURA lance `ffmpeg` depuis votre `PATH` comme un programme distinct, et rien d'autre n'est exécuté. La situation de licence et de brevets de votre build de ffmpeg, c'est à vous de la vérifier ; voir [PATENT_NOTES.md](PATENT_NOTES.md).

**10. Que dit le programme quand ffmpeg est absent ?**
`cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` (pour WebM, l'indication se termine par `or export an APNG instead`). Rien n'est écrit.

**11. De quels encodeurs mon ffmpeg a-t-il besoin ?**
`libx264` pour mp4, `libvpx-vp9` ou `libaom-av1` pour webm, `aac` et `libopus` pour le son dans ceux-ci, `libvorbis` et `libmp3lame` pour `sound mix`.

---

## Performances

**12. Pourquoi est-ce si lent ?**
Vous exécutez probablement un build de débogage. Les chiffres des documents proviennent d'un build de débogage et sont 10 à 30 fois plus lents que ceux d'un build release ([CLI_REFERENCE.md](CLI_REFERENCE.md), section 1.7). Sur la machine de test à 2 cœurs, un build de débogage a mis 46 s pour écrire en PNG les 48 images du projet de démonstration et 57 s pour l'exporter en GIF. Un build release neuf a fait les deux mêmes tâches en 1,5 s et 2,2 s. Compilez avec `cargo build --release` et utilisez `target/release/osatura`.

**13. Utilise-t-il le GPU ?**
Le moteur de rendu par défaut est le CPU. Un crate `gpu` et une commande `render-gpu` existent derrière un indicateur de fonctionnalité, mais aucun vrai GPU n'était disponible pendant le développement : sur un GPU logiciel (llvmpipe), une image 640x480 a pris de 253 à 295 ms contre 13,8 ms sur le chemin CPU. Considérez le rendu GPU comme non testé sur du matériel ([LIMITATIONS.md](LIMITATIONS.md)).

**14. Combien coûte un rig par image ?**
Dans un build de débogage, l'exemple `biped` avait une médiane de 64,5 ms par image dans le contrôle de performance de `ci`. Le bake de tissu d'un petit drapeau a pris 0,43 s. Mesurez votre propre projet avec `osatura stats` et `osatura ci` ; les rigs diffèrent beaucoup.

**15. Comment alléger un projet lourd ?**
Utilisez `lod` pour créer des variantes moins coûteuses, `export-fit` pour trouver les réglages d'export qui respectent un budget d'octets, et `proxy-bench` pour comparer des proxies. Voir [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) et [EXPORT_FORMATS.md](EXPORT_FORMATS.md). `export-fit` peut s'arrêter tôt avec `--max-seconds` et indique alors qu'il a manqué de temps.

**16. Un script de plusieurs milliers d'appels échoue. Pourquoi ?**
`osatura script` accepte au plus 5000 requêtes par exécution et fonctionne en tout ou rien : si une requête est rejetée, rien n'est écrit. Découpez un gros travail en plusieurs scripts, chacun lisant la sortie du précédent.

---

## Déterminisme

**17. La sortie est-elle reproductible ?**
Le rendu est conçu pour l'être : le même projet et la même seed donnent les mêmes pixels, et la physique, les particules et le bruit sont initialisés par une seed. C'est ce qui permet à `golden` et `ci` de comparer des rendus. Cela n'a été vérifié que sur un seul type de machine (Linux x86-64) ; une sortie identique au bit près sur d'autres CPU ou compilateurs n'est pas promise.

**18. Pourquoi un bake de physique donne-t-il un résultat différent après que j'ai changé un seul nombre ?**
Le bake est déterministe pour une entrée donnée, mais c'est une simulation : tout changement de masse, de fréquence d'images, de collider ou de seed modifie l'ensemble de l'exécution. Faites le bake avec les mêmes réglages et la même seed pour comparer. Le bake fusionne par défaut avec l'animation existante (`merge=keep` : les canaux existants sont conservés et la commande affiche `kept existing animation on 'b2' (use merge=replace to overwrite)`) ; utilisez `merge=replace` pour écraser.

**19. Le bruit boucle-t-il ?**
Non. Un canal `noise` ne se répète pas de lui-même ; une boucle montrera donc un saut à la jonction. Utilisez plutôt des clés ou une expression telle que `8 * sin(t * 2 * pi)` ; dans les expressions, `t` va de 0 à 1 sur la durée du clip, et non en images ([ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)).

**20. Puis-je vérifier qu'une animation boucle proprement ?**
Lancez `lint`. `make-loop` supprime entièrement le déplacement : dans un test, une balle qui roulait est devenue statique et `lint` a émis un avertissement (L005). `loop-solve` referme la jonction pour un mouvement cyclique mais a laissé un objet en déplacement avec un saut visible (280 à 280 sur son canal de déplacement). Pour un déplacement qui se poursuit, utilisez le mode d'extrapolation `loop_offset`.

---

## Licences et vos sorties

*Ceci est un résumé en langage clair de [LICENSING.md](LICENSING.md), et non un avis juridique. Une revue de la structure de licences par un juriste reste à faire.*

**21. Quelles licences s'appliquent ?**
MPL-2.0 pour la bibliothèque centrale, l'ABI C, et les crates WASM et GPU ; AGPL-3.0-or-later pour le programme en ligne de commande et le studio (`osatura-cli`) ; CC-BY-4.0 pour la documentation ainsi que pour les assets et modèles d'exemple synthétiques.

**22. À qui appartiennent les animations et rendus que je réalise ?**
À vous. Le principe du projet est que les projets, animations et rendus que vous créez avec OSATURA vous appartiennent entièrement ; les licences s'appliquent au logiciel, pas à sa sortie. Exporter un GIF ou lancer une commande n'entraîne aucune obligation.

**23. Puis-je utiliser la sortie à des fins commerciales ?**
Oui, selon le principe ci-dessus. L'exception est tout ce que vous copiez depuis les assets d'exemple : ils sont sous CC-BY-4.0 et exigent une attribution.

**24. Dois-je publier mes projets si j'utilise les parties AGPL ?**
Non. L'AGPL couvre le programme. Si vous exécutez un `osatura-cli` ou un studio modifié de sorte que d'autres personnes l'utilisent via un réseau, vous devez leur proposer ses sources. Le studio le fait avec une page `/source` et un indicateur `--source-url` qui pointe vers votre copie.

**25. Puis-je lier la bibliothèque centrale à une application à code source fermé ?**
Le cœur est sous MPL-2.0, un copyleft au niveau du fichier : vous pouvez l'utiliser depuis du code fermé et devez partager les modifications apportées aux fichiers couverts par la MPL eux-mêmes. La CLI et le studio sont sous AGPL et relèvent d'un autre cas.

**26. Puis-je dire que mon outil « fonctionne avec OSATURA » ?**
Oui : « compatible avec OSATURA » et « réalisé avec OSATURA » sont autorisés. Un build modifié ne doit pas être distribué sous le même nom ([TRADEMARK.md](TRADEMARK.md)).

**27. Puis-je placer ma contribution sous double licence ?**
Il n'existe pas d'accord de licence de contributeur. Les contributions sont acceptées sous la licence du projet avec une signature DCO (`git commit -s`) ; il n'y a donc aucun propriétaire qui pourrait les relicencier. Détails dans [CONTRIBUTING.md](CONTRIBUTING.md).

**28. Et le code tiers ?**
Environ 360 crates tiers (comptés toutes plateformes confondues ; l'application à fenêtre ajoute la boîte à outils Windows / macOS), tous sous des licences permissives sauf six sous MPL-2.0 : smartstring (MPL-2.0 ou ultérieure, uniquement avec la fonctionnalité `scripting`) et cinq que seule l'application à fenêtre peut faire intervenir. Les listes figurent dans [THIRD_PARTY.md](THIRD_PARTY.md) et [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md).

**29. Y a-t-il des préoccupations liées aux brevets ?**
Voir [PATENT_NOTES.md](PATENT_NOTES.md). Ce sont des notes d'ingénierie, pas un avis juridique.

---

## Sécurité des fichiers et chemins des assets

**30. Une commande peut-elle endommager mon projet ?**
Les écritures sont atomiques : le résultat va dans un fichier temporaire du même répertoire, est synchronisé, puis renommé par-dessus la cible ; un plantage laisse donc l'ancien fichier. Les commandes d'édition revalident le projet qu'elles s'apprêtent à écrire et refusent d'en écrire un invalide (`output would be invalid: ...`). La commande `script` n'applique rien si une seule requête échoue.

**31. Est-il sûr d'ouvrir le projet de quelqu'un d'autre ?**
Raisonnablement, mais pas sans condition. Un projet est une donnée, pas du code ; le seul programme externe qu'OSATURA lance est ffmpeg, pour les formats énumérés à la question 8. Le chargeur refuse les périphériques, les tubes et les répertoires, plafonne la taille de l'entrée et refuse les chemins d'assets absolus ou contenant `..`. Les scripts Rhai s'exécutent dans un bac à sable. Lisez malgré tout [SECURITY.md](SECURITY.md) avant de traiter des fichiers non fiables dans un service.

**32. Où un projet cherche-t-il les sons et les images ?**
Relativement au dossier du fichier de projet. Les chemins absolus et tout chemin contenant `..` sont refusés, sauf si vous passez `--allow-external-assets`, et vous devez le repasser à chaque commande ultérieure qui charge ce projet. Exemple tiré de la référence :
`sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)`.

**33. Quelle taille de projet lit-il ?**
Jusqu'à 256 MiB par défaut. Modifiez cette limite avec `--max-input-bytes N`. Les fichiers plus gros échouent avec `cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)` (les nombres sont ceux de l'exemple).

**34. Pourquoi refuse-t-il d'écrire dans un lien symbolique ?**
Pour qu'un lien placé à côté de votre sortie ne puisse pas rediriger une écriture ailleurs. Le message est `refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself`.

**35. Pourquoi ne puis-je pas lire depuis `<(...)` ou un tube ?**
Les entrées doivent être des fichiers ordinaires : `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)`. Écrivez d'abord les données dans un fichier.

**36. Un champ inconnu dans un projet le casse-t-il ?**
Non : `warning: unknown field bogus (ignored)` est affiché et le code de sortie est 0. Les fichiers annexes que vous écrivez pour une commande (contrats, mappings, machines) sont plus stricts : une clé inconnue y est une erreur et rien n'est appliqué.

---

## Pourquoi la sortie doit différer de l'entrée

**37. Pourquoi `render in.png in.png` dit-il « refusing to overwrite » ?**
Pour les commandes qui lisent un projet et écrivent une image, une vidéo ou un autre fichier qui n'est pas un projet, le nom de sortie ne peut pas être celui de l'entrée. Le message est `refusing to overwrite the input project X with output of the same name`. Cela évite le faux pas classique qui consiste à exporter par-dessus votre unique copie du projet.

**38. Alors pourquoi `script` et `autorig` peuvent-ils écrire par-dessus leur propre entrée ?**
La protection couvre les sorties qui ne sont pas des projets. Les commandes qui lisent un projet et écrivent un projet (`script`, `autorig`, `loop-solve` et d'autres) peuvent désigner le même fichier en entrée et en sortie ; l'écriture est atomique, et si la commande échoue rien ne change. Si vous voulez conserver l'original, donnez un nouveau nom, ou utilisez `osatura vcs` pour conserver l'historique ([CI.md](CI.md) et [CLI_REFERENCE.md](CLI_REFERENCE.md)).

**39. Pourquoi `render` refuse-t-il `out.jpg` ?**
`render` n'écrit que du `.png` ou du `.svg` : `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)`. Pour les autres formats, utilisez `export` ; la liste figure dans [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

---

## Rigging et animation

**40. Dans quel sens se trouve le zéro degré d'un os ?**
L'angle d'os 90 pointe vers le bas ; les angles ne se mesurent pas à partir de l'axe x comme dans un manuel de mathématiques. Les canaux sont des écarts par rapport à la pose de repos, et `rest_offset` s'exprime dans le repère du parent ([RIGGING_GUIDE.md](RIGGING_GUIDE.md)).

**41. Ma chaîne d'IK ne s'étire pas ou présente un à-coup. Que sont `soft` et `stretch` ?**
`soft` (0 à 0,9) amortit l'approche de la chaîne vers l'extension complète et `stretch` (1 à 4) lui permet de s'allonger. Les deux sont réservés au JSON, ce ne sont pas des options Rhai ni CLI. Avec `stretch` supérieur à 1, la zone souple est la fin de la portée étirée. Les chaînes de deux os sont résolues analytiquement ; à partir de trois, FABRIK est utilisé.

**42. Mes contraintes donnent des résultats étranges.**
Les contraintes s'exécutent dans l'ordre de la liste, et l'ordre compte. Un aim de poids 0,5 donne la moitié de l'angle (un angle de -74,7 devient -37,4). Réordonnez dans le JSON si une contrainte ultérieure doit voir le résultat d'une contrainte précédente.

**43. Existe-t-il un skinner automatique pour le dessin vectoriel ?**
Pas en ligne de commande. `Project::auto_skin` n'existe que dans la bibliothèque. Pour le dessin vectoriel, vous riguez avec des os parentés à des calques, avec des lattices, des spines ou des pins (JSON uniquement), ou avec des maillages tracés via `mesh-trace`.

**44. `skin-diffuse` et `cloth` se plaignent du maillage.**
Les deux ont besoin d'un maillage, et cloth a besoin d'un maillage en **grille** : `cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Un calque sans aucun maillage donne `layer 'flag' has no mesh`.

**45. `foot-lock` dit qu'il lui faut une racine.**
`foot lock needs a root controller bone (no parent)`. L'os contrôleur racine ne doit avoir aucun os parent.

**46. `scale` refuse mes clés.**
`scale: frame 27 is outside the project range 0..=24` signifie qu'une clé résultante sortirait du projet ; `keyedit scale` est refusé hors limites sauf si vous ajoutez `--free`. Un essai à blanc utilise `-` comme nom de sortie.

**47. Pourquoi mon projet s'est-il allongé après `mocap-retarget` ?**
`mocap-retarget` étend la durée du projet pour l'adapter au mouvement capturé. `follow --chain` peut décaler des clés au-delà de la dernière image, ce que `lint` signale (L007).

**48. Comment animer un calque sans os ?**
Modifiez les canaux `layer_x` et `layer_y` du calque. Dans le studio, activez l'auto-key ; lorsqu'il est désactivé, une modification de calque reste statique sauf si cette propriété a déjà un canal à clés. Les modifications d'os créent toujours des clés. Voir [STUDIO.md](STUDIO.md).

---

## Studio

**49. Comment ouvrir le studio ?**
`osatura studio project.json`, puis ouvrez l'adresse affichée, normalement `http://127.0.0.1:<port>`. C'est une application monopage servie par le petit serveur HTTP propre à OSATURA (bibliothèque standard uniquement) ; elle n'a pas besoin d'Internet.

**50. Le studio modifie-t-il mon fichier pendant que je travaille ?**
Non. Le fichier sur disque ne change que lorsque vous cliquez sur Enregistrer. Fermer la page sans enregistrer fait perdre les modifications (le journal de collaboration, s'il est activé, conserve le projet entier, mais pas l'historique d'annulation).

**51. D'autres personnes peuvent-elles se connecter ?**
Seulement si vous liez une adresse autre que de bouclage, et un jeton est alors exigé : `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (and still sends everything in clear text)`. Le jeton doit comporter au moins 16 caractères, par exemple issu de `openssl rand -hex 16`. Il n'y a pas de chiffrement : utilisez un tunnel SSH ou un reverse proxy avec TLS.

**52. Où mettre le jeton ?**
Dans l'en-tête `X-Collab-Token` ou le cookie `osatura_token` (HttpOnly, SameSite=Strict, et Secure lorsque la requête arrive sous un `--public-origin` en https). Il n'est jamais accepté dans l'URL. À partir de la cinquième tentative échouée, cette adresse cliente reçoit un HTTP 429 avec un délai croissant (2, 4 ... jusqu'à 30 s).

**53. Que signifient les erreurs HTTP 403, 415 et 421 ?**
421 : l'en-tête Host n'est pas de ceux qu'attend le studio. 403 : un `Origin` ou `Sec-Fetch-Site` étranger. 415 : un POST qui n'est pas du JSON. Ce sont des protections contre les pages de votre navigateur qui appelleraient le serveur local ; voir [STUDIO.md](STUDIO.md) et [SECURITY.md](SECURITY.md).

**54. Fonctionne-t-il dans n'importe quel navigateur ?**
Il n'a été testé que dans Chromium sans interface. Les autres ne sont pas testés. Il n'existe pas de commande de zoom du canevas, et un panneau du studio pour le suivi de visage par webcam existe, mais aucune vraie caméra n'a jamais été disponible pour le tester ([LIMITATIONS.md](LIMITATIONS.md)).

---

## Visage, parole et capture de mouvement

**55. OSATURA suit-il les visages ?**
Il existe un petit détecteur intégré, entraîné uniquement sur des visages synthétiques. Sur 50 scènes de test synthétiques (seed 9), il a trouvé tous les visages, avec une erreur moyenne de points de repère de 0,028 de la taille du visage, et il a levé 2 fausses alertes sur 50 scènes sans visage. Sur des dessins de style cartoon, il s'est mal comporté : 0 visage sur 6 de robots anguleux a été trouvé, et les cas de l'emoji rond et de l'ovale manga ont eu des erreurs de 0,164 et 0,166. Il n'a jamais été exécuté sur de vraies photographies ni sur une vraie webcam. Notez que [LIMITATIONS.md](LIMITATIONS.md) L-13 et le texte du panneau webcam du studio disent qu'aucun détecteur de visage n'est fourni ; c'est obsolète en ce qui concerne ce détecteur synthétique.

**56. Comment fonctionne le lip-sync ?**
Les repères de bouche choisissent parmi cinq formes (repos, petite, ouverte, ronde, large). Un temps de repère avec un point décimal est en secondes ; un entier seul est une image. `--rhubarb` ne change que la façon dont la lettre E est classée (ronde au lieu de large). Un audio plus long que le projet est tronqué.

**57. Puis-je animer à partir d'une vidéo ?**
`face-track` lit la vidéo via ffmpeg. `face-bake` écrit les signaux sous forme de canaux à clés (le résumé indique « +N channels (K keys in them), M trigger keys set ») et tronque à la longueur du projet, avec un avertissement sur stderr comptant les images non bakées. Un signal inconnu donne `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.

---

## Bugs, sécurité et contributions

**58. Comment signaler un bug ?**
Ouvrez un ticket avec la ligne de commande, la sortie exacte (stderr aussi), le code de sortie, le build (`osatura --version`) et, si vous le pouvez, un projet minimal. Lancez d'abord `osatura check project.json` et `osatura lint project.json` : de nombreux problèmes y sont signalés. Voir [CONTRIBUTING.md](CONTRIBUTING.md).

**59. Comment signaler un problème de sécurité ?**
En privé, pas dans un ticket public ; voir [SECURITY.md](SECURITY.md). L'adresse indiquée dans le dépôt est `alexio@trom.tf` ; c'est la même que celle du contact du mainteneur et elle n'a pas été remplacée par une boîte aux lettres d'équipe de sécurité.

**60. Existe-t-il une liste de diffusion, un chat ou un contrat de support ?**
Non. Les documents indiquent un seul contact de mainteneur ; la première version est encore une préversion en brouillon.

**61. Comment contribuer ?**
Signez chaque commit (`git commit -s`), maintenez les tests au vert (`cargo test`, et `osatura ci` pour le travail d'animation) et lisez [DEVELOPMENT.md](DEVELOPMENT.md). La documentation est sous CC-BY-4.0 et accueille volontiers les corrections, surtout là où une page et le programme divergent.

---

## Limitations

**62. Quelle est la limite la plus importante ?**
Il a été construit et testé par un seul projet dans un seul environnement. Tout ce qui n'est pas indiqué comme exécuté est « non testé ». Lisez [LIMITATIONS.md](LIMITATIONS.md) ; il comporte des identifiants que vous pouvez citer dans les tickets.

**63. Quels formats d'export demandent de l'attention ?**
La vidéo nécessite ffmpeg. Lottie et glTF couvrent un sous-ensemble de ce qu'OSATURA sait dessiner et animer ; vérifiez donc ce qui est perdu dans [EXPORT_FORMATS.md](EXPORT_FORMATS.md). L'erreur d'`export` pour une extension inconnue énumère `.gif .html .json .mp4 .apng .webm .png .glb`, ce qui omet `.lottie` ; `.lottie` (dotLottie) est une cible valide, voir [EXPORT_FORMATS.md](EXPORT_FORMATS.md), section 6.

**64. Le build WASM est-il utilisable ?**
Il se compile pour `wasm32-unknown-unknown` sur GitHub Actions et le test du wrapper passe sur le vrai module sous Node 20 ; il est joint au brouillon de version sous le nom `osatura-web.tar.gz`. Il n'a pas été exécuté dans un navigateur, et sa vitesse et sa mémoire n'ont pas été mesurées ([WASM.md](WASM.md), [LIMITATIONS.md](LIMITATIONS.md)). Considérez-le comme partiellement vérifié.

**65. Importe-t-il les fichiers de projet d'autres outils ?**
Il importe SVG, OpenRaster, JSON Lottie, images, GIF/APNG, capture de mouvement BVH et MIDI ([EXPORT_FORMATS.md](EXPORT_FORMATS.md), sections 9 et 10). Il n'importe ni PSD, ni fichiers zip `.lottie`, ni dossiers OCA. Les imports créent un nouveau projet. Notez que `import-lottie in.json in.json` n'a pas de protection contre l'écrasement et remplace son entrée. Ne vous attendez pas à ce qu'un fichier importé corresponde pixel pour pixel à l'original.

**66. Le JSON de projet est-il stable ?**
C'est le « format 1 ». Un fichier plus récent est refusé : `project format 2 is newer than this build supports (1); update OSATURA`. Comme seule une préversion alpha existe, le format 1 lui-même peut encore changer avant une version stable ; voir [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

---

## Comparaisons avec d'autres outils

**67. En quoi diffère-t-il d'un éditeur d'animation de bureau fondé sur une timeline ?**
Ceux-ci sont des programmes interactifs, centrés sur le dessin. OSATURA est centré sur la commande : chaque opération est une commande ou un script, le projet est un JSON comparable par diff, et les rendus sont déterministes, ce qui permet de les tester en CI. Le studio est une fine vue web au-dessus du même moteur, et il est bien moins abouti qu'un éditeur mature.

**68. En quoi diffère-t-il d'un outil d'animation squelettique 2D pour les jeux ?**
Ces outils se concentrent sur les maillages, les skins et les bibliothèques runtime pour des moteurs de jeu précis. OSATURA a aussi des os, de l'IK, des contraintes et des maillages, mais une offre runtime plus modeste : son propre lecteur en HTML, une ABI C et un crate WASM non compilé. Choisissez selon le runtime que vous devez cibler.

**69. En quoi diffère-t-il d'un compositeur de motion design ?**
Un compositeur est construit autour de calques avec effets et expressions sur des séquences vidéo. OSATURA a des effets de calque et des expressions, mais son centre est le rigging de personnages et le dessin vectoriel, et il ne fait ni compositing vidéo, ni 3D, ni étalonnage de séquences.

**70. Quand OSATURA est-il le mauvais choix ?**
Lorsque vous avez besoin dès aujourd'hui de performances temps réel sur GPU, d'un flux de travail abouti avec tablette graphique, d'animation image par image dessinée à la main, d'un produit pris en charge par un éditeur avec version stable et support, ou d'un comportement confirmé hors de Linux x86-64.

---

## Tableau de dépannage

Chaque message ci-dessous a été produit par un vrai build et se termine par le code de sortie 1, sauf indication contraire. La formulation d'un message peut changer avant une version publiée.

| Message | Cause | Solution |
|---|---|---|
| `cannot read nope.json: No such file or directory (os error 2)` | Mauvais chemin ou mauvais nom | Vérifiez le chemin et le répertoire courant |
| `invalid JSON in broken.json: EOF while parsing a value at line 2 column 0` | Le fichier n'est pas un JSON complet | Corrigez la syntaxe à la ligne indiquée ; restaurez depuis une sauvegarde ou `osatura vcs` |
| `badparent.json is not a valid project:` then `bone 'a' has parent 'ghost' which is missing or listed after it` | Un os parent est manquant ou listé après son enfant | Définissez les parents avant les enfants ; corrigez le nom |
| `invalid JSON in fmt2.json: project format 2 is newer than this build supports (1); update OSATURA` | Fichier issu d'un build plus récent | Utilisez un build plus récent |
| `warning: unknown field bogus (ignored)` (exit 0) | Une clé que ce build ne connaît pas | Supprimez ou corrigez la clé ; vérifiez une faute de frappe |
| `refusing to overwrite the input project X with output of the same name` | Le nom du fichier de sortie est égal à celui de l'entrée | Choisissez un autre nom de sortie |
| `render writes .png or .svg; 'out.jpg' has another extension (nothing was written)` | Mauvaise extension pour `render` | Utilisez `.png`/`.svg`, ou `export` pour les autres formats |
| `unknown option --bone for 'render' (did you mean --bones?); see: osatura (no arguments) for the usage` | Option mal orthographiée | Utilisez l'option suggérée |
| `unknown export format '.bmp' (use .gif .html .json .mp4 .apng .webm .png .glb)` | Extension non exportable | Choisissez un format listé (`.lottie` est aussi valide) |
| `--scale needs a number in (0, 16]` | Échelle hors limites | Utilisez une valeur supérieure à 0 et au plus égale à 16 |
| `bad frame number 'x'` | Un argument d'image n'est pas un nombre entier | Passez un entier |
| `cannot run ffmpeg (No such file or directory (os error 2)); install it or export a GIF/HTML instead` | ffmpeg absent du `PATH` | Installez ffmpeg, ou exportez en GIF, HTML ou APNG |
| `the project has no sound tracks (see: sound add)` | Commande de son sur un projet muet | Ajoutez une piste avec `sound add` |
| `channel hand_ctrl.BoneOffsetY is locked` | Le canal est verrouillé | Déverrouillez-le, ou modifiez un autre canal |
| `request 1 (hand_ctrl.BoneOffsetY at 0) was rejected, nothing changed: channel hand_ctrl.BoneOffsetY is locked` | Idem, depuis `script` | Déverrouillez d'abord ; rien n'a été appliqué |
| `request 1 (channel head.BoneAngle) was rejected, nothing changed: ... expression error: unknown variable 'tt'` | Faute de frappe dans une expression | Les variables sont frame, time, fps, frames, pi, t |
| `... unexpected end of expression` | Expression inachevée | Fermez les parenthèses, terminez la formule |
| `points need an even number of coordinates: 3..1000 points` | Nombre impair de coordonnées ou trop peu de points | Donnez des paires x,y : de 3 à 1000 points |
| `cannot read /dev/fd/63: not a regular file (devices, pipes and directories are refused)` | Substitution de processus ou tube en entrée | Enregistrez d'abord dans un fichier ordinaire |
| `sound add: asset '/etc/passwd': absolute paths are not allowed (...)` | Asset hors du dossier du projet | Copiez-le à côté du projet, ou passez `--allow-external-assets` |
| `cannot read demo.json: N bytes exceed the input limit of M bytes (raise it with --max-input-bytes N)` | Fichier au-dessus de la limite d'entrée | Augmentez `--max-input-bytes` si vous faites confiance au fichier |
| `refusing to write through a symbolic link; pass --follow-symlinks ...` | La sortie est un lien symbolique | Désignez la cible, ou passez `--follow-symlinks` |
| `cloth needs a grid mesh; the layer has a free-form (traced) mesh` | Cloth sur un maillage tracé | Construisez un maillage en grille pour le calque |
| `layer 'flag' has no mesh` | Une commande de maillage sur un calque simple | Créez d'abord un maillage |
| `no bone 'b9'` | Le nom d'os n'existe pas | Vérifiez les noms avec `stats` ou `explain` |
| `unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)` | Mauvais nom de signal | Utilisez un nom de la liste |
| `foot lock needs a root controller bone (no parent)` | La racine choisie a un parent | Utilisez un contrôleur sans parent |
| `scale: frame 27 is outside the project range 0..=24` | Les clés mises à l'échelle sortent du projet | Allongez le projet, réduisez l'échelle, ou utilisez `--free` |
| `--extrapolate must be hold, linear, loop, ping_pong or loop_offset` | Mode incorrect | Utilisez l'un des cinq |
| `cannot listen on 127.0.0.1:8861: Address already in use (os error 98)` | Port occupé | Choisissez un autre `--port` ou arrêtez l'autre processus |
| `cannot listen on 127.0.0.1:99999: invalid port value` | Port supérieur à 65535 | Utilisez 1 à 65535 |
| `refusing to listen on 0.0.0.0:8851: the studio has no accounts and no TLS, so a non-local address needs --collab-token (...)` | Liaison publique sans jeton | Ajoutez `--collab-token`, ou restez sur l'adresse de bouclage |
| `--collab-token must be at least 16 characters (e.g. \`openssl rand -hex 16\`)` | Jeton faible | Générez-en un plus long |
| `--source-url must be an http:// or https:// URL without spaces or quotes` | URL mal formée | Utilisez une URL simple |
| `--public-origin must be a bare origin (scheme, host and optional port, no path), e.g. https://studio.example.org` | L'origine comporte un chemin | Donnez uniquement schéma, hôte et port |
| `output would be invalid: ...` | La modification laisserait un projet invalide | Lisez les raisons après les deux-points ; rien n'a été écrit |
| `budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes ...` | `export-fit` ne peut pas atteindre le budget | Augmentez le budget ou simplifiez le projet (le plus petit fichier est tout de même écrit) |
| `time budget exhausted (60 s ...)` | `export-fit` a manqué de temps | Augmentez `--max-seconds` |
| `fuzz: failure rate 98.00% exceeds --max-fail-rate 50.00%` | Des poses ont cassé le rig, et vous avez demandé un seuil bloquant (`--gate` ou `--max-fail-rate`) | Examinez les cas en échec ; n'augmentez le taux que si c'est voulu ; `fuzz` seul ne fait que rapporter (exit 0) |
| `no .osatura-vcs repository found from ... (run: osatura vcs init <project.json>)` | Pas encore d'historique | Lancez `vcs init` |
| `frame 999 out of range` (studio API) | Image au-delà du projet | Utilisez une image comprise entre 0 et la dernière image |
| `bad kind 'x' (vector, raster, png)` (studio API) | Mauvais type | Utilisez un type listé |
| `nothing selected` (studio API) | Une commande de sélection sans sélection | Sélectionnez d'abord des calques ou des os |

Les codes de sortie d'`osatura ci` sont 0 réussite, 1 échec, 2 erreur.

---

*Documentation CC-BY-4.0. Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio). Vous pouvez la partager et l'adapter avec attribution. Voir aussi : [INDEX.md](INDEX.md), [GLOSSARY.md](GLOSSARY.md).*
