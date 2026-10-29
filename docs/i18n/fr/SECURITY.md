<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Sécurité

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/docs/SECURITY.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/SECURITY.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/SECURITY.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/SECURITY.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/SECURITY.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/SECURITY.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/SECURITY.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/SECURITY.md)

*Modèle de menaces, protections intégrées, lacunes connues, manière de signaler un problème et déploiement sûr du studio. Documentation CC-BY-4.0. Dérivé des sources (`osatura-core/src/{api,assets,fsutil,expr,script}.rs`, `osatura-cli/src/{http,studio,collab_srv}.rs`), de la section 9 de `ROADMAP.md` (vagues de revue 1 à 4) et des tests nommés ci-dessous. **OSATURA n'a fait l'objet d'aucun audit de sécurité indépendant.** Les vagues de revue ont été menées par le projet lui-même ; considérez cette page comme une description des intentions et de ce que couvrent les tests, non comme une garantie.*

Sommaire : [1 Modèle de menaces](#1-modèle-de-menaces) - [2 Serveur HTTP du studio](#2-serveur-http-du-studio) - [3 Authentification et collaboration](#3-authentification-et-collaboration) - [4 Résolution des ressources et limites d'entrée](#4-résolution-des-ressources-et-limites-dentrée) - [5 Bac à sable des expressions et des scripts](#5-bac-à-sable-des-expressions-et-des-scripts) - [6 Fichiers, liens symboliques et données temporaires](#6-fichiers-liens-symboliques-et-données-temporaires) - [7 Programmes externes (ffmpeg)](#7-programmes-externes-ffmpeg) - [8 Bibliothèques (ABI C, WASM, façade)](#8-bibliothèques-abi-c-wasm-façade) - [9 Lacunes connues](#9-lacunes-connues) - [10 Signaler une vulnérabilité](#10-signaler-une-vulnérabilité) - [11 Liste de durcissement pour le déployeur](#11-liste-de-durcissement-pour-le-déployeur) - [12 Proxy inverse avec TLS : nginx et Caddy](#12-proxy-inverse-avec-tls--nginx-et-caddy)

## 1. Modèle de menaces

Ce qu'OSATURA traite : des **fichiers de projet** (JSON), des images, de l'audio, des fichiers d'import Lottie / OCA / de type glTF, des fichiers de correspondances et, dans le studio, des requêtes HTTP provenant d'un navigateur. Dans le modèle ci-dessous, les fichiers de projet sont des *données non fiables* : un projet reçu d'une autre personne peut être hostile, et son ouverture ne doit ni exécuter de code, ni lire des fichiers en dehors de son dossier, ni épuiser la machine.

| Acteur | Peut | Objectif supposé | Principales défenses |
|---|---|---|---|
| Auteur d'un projet / fichier d'import hostile | vous donner un fichier | lire vos fichiers, faire planter ou bloquer l'outil, exécuter du code, remplir le disque | analyse stricte, limites de taille et de profondeur, refus des chemins de ressources, aucune panique sur une entrée, bac à sable des expressions et des scripts, sortie atomique |
| Page web que vous visitez pendant que le studio tourne en local | faire envoyer à votre navigateur des requêtes vers `127.0.0.1` (CSRF, DNS rebinding) | modifier votre projet, le lire, écrire des fichiers | liste d'hôtes autorisés, contrôles d'Origin / `Sec-Fetch-Site`, type de contenu JSON, jeton avec cookie `SameSite=Strict`, CSP |
| Autre utilisateur de la même machine | se connecter au port, lire `/tmp` | lire ou détourner une session ou des fichiers temporaires | écoute sur `127.0.0.1` par défaut, répertoires temporaires privés (0700), fichiers de journal en 0600, jeton exigé pour écouter ailleurs |
| Quelqu'un sur votre réseau, lorsque le studio écoute au-delà de localhost | atteindre le port | utiliser ou deviner le jeton, épuiser les connexions | jeton obligatoire (16 caractères ou plus), comparaison à temps constant, temporisation progressive, plafonds de connexions et de taille ; **pas de TLS** |
| Un collaborateur disposant du jeton | modifier la session partagée | endommager le projet | historique d'annulation, journal chaîné par hachage ; les collaborateurs ont la confiance du projet, il n'y a ni comptes ni rôles |
| Appelant hostile des bibliothèques | passer de mauvais arguments à la façade, à l'ABI C ou WASM | corruption mémoire, paniques à travers la frontière | validation des arguments, contrôle des tailles avant allocation, paniques interceptées (là où la cible déroule la pile) |

Hors périmètre : la protection contre un administrateur local malveillant, une chaîne de compilation ou une dépendance compromise (voir [THIRD_PARTY.md](THIRD_PARTY.md) pour ce qui est livré ; `cargo deny` contrôle les licences, les interdictions et les sources, pas les vulnérabilités), les canaux auxiliaires, et la sécurité de ffmpeg lui-même.

## 2. Serveur HTTP du studio

`osatura studio` sert l'éditeur à partir de sa propre petite implémentation HTTP/1.1 (`osatura-cli/src/http.rs`, bibliothèque standard uniquement, aucun framework web tiers). Comportement, avec les tests qui le figent (tests unitaires de `osatura-cli`, `tests/robust.rs`, `tests/wave4.rs`, `tests/source_offer.rs` et les tests de fumée Chromium) :

**Limites**

| Limite | Valeur | Résultat en cas de dépassement |
|---|---|---|
| Délai de réception de l'en-tête de requête | 10 s | 408 |
| Délai de réception du corps | 20 s | 408 |
| Délai d'écriture d'une réponse | 10 s | connexion fermée |
| Taille de l'en-tête de requête | 16 Ko | 431 |
| Nombre d'en-têtes | 100 | 431 |
| Taille du corps | 32 Mo | 413 |
| Connexions simultanées | 128 | 503 |
| Corps de requête en transfert fragmenté (chunked) | non pris en charge | 411 |
| En-tête `Host` en double | refusé | 400 |
| Caractère de contrôle (CR, NUL, DEL…) dans la valeur d'un en-tête | refusé (une tabulation est autorisée) | 400 |

Chaque réponse porte `Connection: close` (aucun état keep-alive à épuiser), `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` et un `Content-Security-Policy`.

**Politique de requête** (`Policy::check`, appliquée à chaque requête) :

1. **Liste d'hôtes autorisés.** L'en-tête `Host` doit être l'adresse d'écoute (`127.0.0.1`, `localhost`, le port réel) ou un nom donné par `--allow-host`. Sinon : 421 Misdirected Request. C'est ce qui arrête le DNS rebinding : une page sur `evil.example` qui se résout en `127.0.0.1` envoie `Host: evil.example`.
2. **Origin et `Sec-Fetch-Site`.** Les requêtes portant un `Origin` d'un autre site, ou `Sec-Fetch-Site: cross-site` / `same-site`, sont refusées avec un 403. `Origin: https://<allowed host>` est accepté pour les hôtes de `--allow-host` ; derrière un proxy inverse qui sert la page à une URL publique, passez `--public-origin URL` afin que l'origine du proxy soit acceptée. Les origines croisées restent refusées.
3. **Type de contenu.** `POST` exige `Content-Type: application/json` (sinon 415), de sorte qu'un simple formulaire HTML ne peut pas poster vers l'API.
4. **Jeton** (voir la section 3), lorsqu'il est configuré ; obligatoire lorsque `--bind` n'est pas une adresse de bouclage.

**Offre de sources.** Le studio sert `/source` (y compris lorsqu'un jeton est exigé) et `--source-url` en définit l'adresse (les binaires de release officiels prennent par défaut le commit à partir duquel ils ont été construits), comme l'exige l'AGPL-3.0 section 13 (voir [LICENSING.md](LICENSING.md)).

**Les chemins et méthodes inconnus ou mal formés** reçoivent une réponse d'erreur simple ; la page de l'éditeur est compilée dans le binaire (`studio.html`) et n'est pas lue depuis le système de fichiers : il n'y a donc aucune surface de traversée de chemin pour le contenu statique.

## 3. Authentification et collaboration

L'option `--collab-token T` (ou le jeton exigé par un `--bind` non local) active le contrôle d'accès ; `--collab-log LOG` rend persistante une session partagée.

* **Format du jeton.** Au moins 16 caractères, parmi `A-Za-z0-9._~+/=-`. Tout ce qui contient CR, LF, `;` ou d'autres caractères est refusé au démarrage (injection d'en-tête et de cookie).
* **Présentation.** En-tête `X-Collab-Token: T`, ou cookie `osatura_token` (`HttpOnly`, `SameSite=Strict`, plus `Secure` lorsque la requête est arrivée sous un nom `--public-origin` en `https://`, un `Origin` https ou `X-Forwarded-Proto: https`) défini par `POST /api/login`. Un mauvais cookie compte comme une tentative échouée, comme un mauvais en-tête.
* **Comparaison.** À temps constant (les deux côtés sont hachés avec SHA-256 et les condensés comparés), de sorte que le temps de réponse ne révèle pas les correspondances de préfixe.
* **Temporisation progressive.** Par adresse IP cliente, à partir du **5e** échec consécutif : 2, 4, 8, 16, puis 30 secondes de réponses 429 ; jusqu'à 512 adresses clientes sont suivies. Le bon jeton est toujours accepté, même pendant une temporisation (un attaquant ne peut donc pas verrouiller le propriétaire dehors). La temporisation ralentit les devinettes ; elle n'arrête pas une attaque distribuée. **Derrière un proxy inverse, chaque client apparaît avec l'adresse du proxy**, ils partagent donc un seul compteur de temporisation : un flot de mauvaises requêtes d'un visiteur retarde le compteur d'échecs pour tous, mais le bon jeton fonctionne toujours.
* **Secrets de participant.** Chaque collaborateur reçoit un secret aléatoire de 128 bits (`X-Collab-Secret`) identifiant son flux ; au plus 32 participants et 64 flux d'événements.
* **Intégrité du journal.** Le journal de session est un fichier texte, une ligne par opération : `<hash> <json>`, chaîné avec SHA-256 (format version 2). Modifier, supprimer ou réordonner une ligne rompt la chaîne, ce qui est signalé au chargement du journal. Chaque requête est `fdatasync`ée avant la réponse, le fichier est créé avec le mode 0600 et réécrit de façon atomique. La chaîne détecte les dommages accidentels et les altérations occasionnelles ; elle n'est **pas signée** : quelqu'un qui peut écrire le fichier peut reconstruire une chaîne valide.
* **Jetons par participant (vague 7, facultatif).** `--collab-secret-file F` remplace le jeton porteur partagé unique par des jetons dérivés d'un secret : `ap1.<name>.<expires>.<mac>` avec `mac` = HMAC-SHA-256 (RFC 2104, SHA-256 maison, vecteurs de test de la RFC 4231 dans les tests) calculé sur le nom et l'échéance. L'hôte les émet avec `osatura collab-token NAME --secret-file F --ttl SECONDS` (8 heures par défaut, 30 jours au plus) et donne à chaque personne le sien. Un jeton est lié à un seul nom (un participant qui rejoint reçoit ce nom quoi que la page envoie, de sorte que personne ne peut se faire passer pour quelqu'un d'autre), expire de lui-même, est vérifié par recalcul (aucune table sur le serveur, rien à y voler hormis le secret lui-même), et son cookie reçoit un `Max-Age` égal à la durée de vie restante. `--collab-revoke NAMES` refuse les jetons de ces noms avant leur expiration ; changer le secret révoque tout. Le fichier secret doit être privé (mode 600 sous Unix, sinon le studio refuse de démarrer), d'au moins 16 caractères, et n'est jamais accepté comme argument de ligne de commande. `--collab-token` et `--collab-secret-file` peuvent être combinés ; l'un ou l'autre satisfait la règle selon laquelle une écoute hors bouclage exige un identifiant. L'expiration est vérifiée **à chaque requête** : un flux d'événements déjà ouvert reste ouvert jusqu'à sa fin, et la connexion suivante est refusée. Un jeton expiré ou révoqué reçoit un `401` avec la raison et n'alimente pas la temporisation anti-devinette.
* **Limite de débit (vague 7).** `--collab-rate N` autorise N requêtes par seconde et par adresse cliente, avec des rafales de 2N (seau à jetons) ; au-delà, la réponse est `429` avec `Retry-After`. La valeur par défaut est de 100 par seconde lorsqu'un identifiant est exigé et aucune limite sinon ; 0 la désactive. Elle s'applique après l'authentification et par **adresse** (la source d'une connexion), pas par connexion TCP, car une limite par connexion se contourne en ouvrant davantage de connexions ; derrière un proxy inverse tous les clients partagent l'adresse du proxy, limitez donc aussi au niveau du proxy (section 12).
* **Avertissements d'adresse d'écoute (vague 7).** Un `--bind` hors bouclage affiche un avertissement au démarrage (HTTP en clair ; pour `0.0.0.0` / `::`, que le studio écoute sur toutes les interfaces ; pour un jeton partagé, qu'il n'a pas de noms et n'expire jamais).
* **Écriture des jetons.** L'échéance d'un jeton par participant n'a qu'une seule forme écrite (chiffres décimaux, sans signe, sans zéro initial, sans `+`) : toute autre graphie du même nombre est refusée comme malformée, et la signature est jugée avant l'échéance, de sorte qu'un jeton forgé n'apprend rien de la réponse.
* **Hôtes (facultatif).** `--collab-host NAMES` (nécessite `--collab-secret-file`) désigne les participants autorisés à **enregistrer** (`/api/save`), à **compacter** le journal de session (`/api/collab/compact`) et à **révoquer** (`/api/collab/revoke`, un `POST` qui ajoute des noms à la liste des révoqués, 1000 au plus). Toute autre personne munie d'un jeton valide peut toujours éditer et observer, et reçoit `403` pour ces trois requêtes. Sans `--collab-host`, rien ne change : chaque détenteur d'un identifiant valide peut tout faire, comme avant. Le `--collab-token` partagé n'est lié à aucun nom et compte comme un hôte.
* **Flux d'événements.** Le flux d'événements exige un identifiant (un jeton) mais pas de secret de participant ; le secret identifie le flux propre à un participant, ce n'est pas un second mot de passe.
* **Connexions par adresse.** Une même adresse cliente ne peut détenir que 16 connexions à la fois (en plus du plafond global), de sorte qu'une seule machine ne peut pas épuiser tous les emplacements. Les adresses de bouclage ne sont pas comptées par défaut, car un proxy inverse situé sur le même hôte atteint le studio depuis `127.0.0.1` pour chaque visiteur ; limitez par visiteur au niveau du proxy (section 12).
* **Pas de TLS, pas de comptes.** Le serveur parle HTTP en clair. Un jeton qui circule sur un réseau est lisible par quiconque se trouve sur le chemin. Placez le studio derrière un proxy inverse qui termine le TLS ou derrière un tunnel SSH / VPN (section 11). Il n'y a ni comptes d'utilisateur, ni permissions par projet ; le seul rôle est la liste facultative d'hôtes ci-dessus, et tout autre jeton valide donne un accès en édition.

## 4. Résolution des ressources et limites d'entrée

* **Les ressources** (fichiers image, normal maps, sons nommés par un projet) sont résolues par `assets::resolve` : par rapport au répertoire de ressources valable pour tout le processus (`assets::default_dir`, normalement le dossier du projet d'entrée). **Les chemins absolus et tout composant `..` sont refusés** sauf si le drapeau global `--allow-external-assets` est donné. Les fichiers image sont limités à 64 Mo, les fichiers audio à 256 Mo.
* **Fichiers de projet.** `load_project` ne lit que des fichiers ordinaires (ni périphériques, ni tubes), jusqu'à 256 Mo (`--max-input-bytes N` pour changer). `MAX_JSON_BYTES` de la façade vaut 256 MiB.
* **Projets dérivés.** Les commandes qui écrivent un projet dérivé valident par rapport au répertoire du projet *d'entrée* ; lorsque la sortie est ailleurs, les chemins de ressources relatifs des fichiers existants sont réécrits ou un avertissement indique que la sortie ne les trouvera pas (`--allow-external-assets` les rend absolus).
* **Rendu.** La façade refuse les images de plus de 16384 px de côté (`MAX_RENDER_SIDE`) et les échelles en dehors de 0,05 à 4 ; les couches C et wasm refusent les sorties supérieures à 256 MiB avant d'allouer. Autres limites de la façade : fps dans (0, 1000], images de 1 à 200000, scène jusqu'à 32768, 4096 os, magnitudes dans 1e7, `lod_levels` de 1 à 12, échantillons de `fuzz` jusqu'à 100000, couleurs de `trace` de 2 à 64 ([API.md](API.md)).
* **Aucune panique sur une entrée.** Les tests d'entrées hostiles se trouvent dans `osatura-cli/tests/robust.rs` et dans les tests de vagues ; des tests de type fuzz couvrent les analyseurs. Une panique atteignable depuis une entrée est traitée comme une vulnérabilité.
* **Les corps du studio** sont limités à 32 Mo ; les modifications passent par la même validation que les fichiers.

## 5. Bac à sable des expressions et des scripts

* **Les expressions** (liaisons de valeurs, contraintes, pilotes) sont analysées par un analyseur maison : **source jusqu'à 4096 octets, profondeur d'imbrication 64**, arithmétique, comparaison, les variables fixes `frame`, `time`, `fps`, `frames`, `pi`, `t`, `get("layer","property")` pour les pilotes, et une table fixe de fonctions intégrées (`sin`, `abs`, `min`, ...) ; pas de boucles, pas d'E/S et pas de fonctions définies par l'utilisateur : l'évaluation se termine donc toujours. Les résultats non finis sont rejetés par la politique des valeurs finies.
* **Les scripts** (commande `script`, feature `scripting`, activée par défaut) s'exécutent dans Rhai avec ces limites : **500 000 opérations, 32 niveaux d'appel, profondeur d'expression (64 au niveau supérieur, 32 dans les fonctions), chaînes de 10 000 caractères, tableaux de 10 000 éléments, maps de 1 000 entrées, 5 000 requêtes d'API, source de script de 64 Ko**, et `no_module` (pas d'`import`, donc aucun accès aux fichiers par des modules). L'API hôte exposée aux scripts est un ensemble fixe de fonctions d'édition de projet. Compilez avec `--no-default-features` pour retirer entièrement le moteur de scripts (cela retire aussi la dépendance `smartstring`).
* Un script reste du code que vous avez choisi d'exécuter sur un projet : il ne peut ni lire ni écrire de fichiers lui-même, mais il peut rendre très volumineux le projet qu'il modifie dans les limites ci-dessus, et une boucle infinie s'arrête à la limite d'opérations.

## 6. Fichiers, liens symboliques et données temporaires

* **Écritures atomiques.** Les sorties sont écrites avec `fsutil::write_atomic` : un fichier temporaire dans le même répertoire (nom `.<name>.tmp-<pid>-<random>`), `fsync`, `rename`. Un plantage laisse l'ancien fichier ou le nouveau. **Les liens symboliques donnés comme sortie sont refusés**, en nommant `--follow-symlinks` (cela couvre désormais toute sortie adressée par l'utilisateur : rapports et images de référence de `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, fichiers de pose, instantanés, `vcs`, le journal de collaboration, et les fichiers vidéo écrits par ffmpeg, qui sont pré-vérifiés), de sorte qu'un lien planté ne peut pas rediriger une écriture. Si le répertoire n'est pas accessible en écriture mais que le fichier l'est, le repli est un écrasement sur place avec `fsync`, qui n'est **pas atomique** (documenté).
* **Données privées.** Les journaux de collaboration et les autres fichiers que seul le propriétaire doit lire sont créés avec le mode 0600 (`write_atomic_private`). Les répertoires temporaires sont créés en exclusivité avec le mode 0700 (`private_temp_dir`), de sorte qu'un nom prévisible dans un `/tmp` partagé ne peut pas être créé à l'avance par un autre utilisateur. Les valeurs aléatoires (jetons des nouveaux participants, noms temporaires) proviennent de `/dev/urandom` (`fsutil::random_bytes`) ; le générateur déterministe à graine du moteur n'est jamais utilisé pour des secrets.
* **Réglages de l'interface.** Le `ui.json` du studio ignore les clés inconnues en émettant un avertissement et est copié dans `ui.json.bak` (puis `.bak.1`, ...) avant la première sauvegarde.
* **Windows / hors Unix.** Les modes de permission ci-dessus s'appliquent sous Unix. Sous Windows et macOS, la suite de tests passe en CI (les vérifications propres à POSIX sont ignorées sous Windows), mais les vérifications des liens symboliques, des permissions et de `/dev/urandom` ne s'exécutent que sous Linux et macOS et personne n'a relu à la main le comportement sous Windows.
* **Binaires non signés et application à fenêtre.** Les exécutables de release ne sont pas signés ; vérifiez les téléchargements avec `SHA256SUMS.txt`. `osatura-studio` démarre le même serveur local (`127.0.0.1` uniquement, port libre aléatoire) et l'affiche dans la vue web du système (WebView2 / WKWebView) ; cette vue web est aussi exposée qu'un onglet de navigateur sur cette adresse et n'a fait l'objet d'aucune revue de sécurité séparée.

## 7. Programmes externes (ffmpeg)

ffmpeg et ffprobe sont lancés comme processus distincts **uniquement** pour l'export vidéo, le décodage mp3 / ogg, le mixage de la bande-son et quelques outils ; ils ne sont jamais fournis avec le projet ([LICENSING.md](LICENSING.md)). Chaque invocation :

* passe les arguments sous forme de tableau (pas de shell, pas de concaténation de chaînes dans une ligne de commande) ;
* utilise `-protocol_whitelist file` (un usage interne de tube ajoute `pipe,fd`), de sorte qu'un fichier média ne peut pas faire ouvrir à ffmpeg `http://`, `concat:` ou d'autres protocoles réseau / spéciaux ;
* passe les noms de fichiers par `fsutil::file_arg`, qui préfixe `./` aux noms commençant par `-` ou contenant `:` ou `|` dans le premier composant du chemin, de sorte qu'un nom de fichier ne puisse pas être lu comme une option ou une URL ;
* pour les fichiers son `.wav`, `.mp3` et `.ogg`, impose le démultiplexeur avec `-f` (`soundtrack::forced_demuxer`), de sorte qu'un fichier qui n'est pas ce que son nom prétend est refusé au lieu d'être ouvert comme un autre conteneur ;
* utilise `-loglevel error`.

ffmpeg analyse lui-même des formats médias complexes et a connu ses propres vulnérabilités : gardez-le à jour et ne lui donnez pas de médias provenant de sources non fiables sur une machine qui détient des secrets. Les commandes à budget de temps (`export-fit --max-seconds`) ne peuvent pas interrompre un appel ffmpeg en cours.

## 8. Bibliothèques (ABI C, WASM, façade)

* La **façade** valide chaque argument, renvoie `ApiError` et convertit les paniques en `ApiError::Internal` là où la cible déroule la pile. `Animation` et `Player` sont `Send + Sync` (vérifié à la compilation).
* L'**ABI C** ([C_ABI.md](C_ABI.md)) vérifie les pointeurs nuls, les longueurs et l'UTF-8 et renvoie des codes ; l'appelant doit toujours passer des pointeurs valides et respecter les règles de tampon. La chaîne de dernière erreur est propre à chaque thread. L'ABI a été exercée sous AddressSanitizer et UndefinedBehaviorSanitizer avec les exemples documentés (sans problème). Elle n'est pas durcie contre un appelant qui viole les contrats documentés (double fermeture, utilisation après fermeture, longueurs erronées).
* Le **module WASM** ([WASM.md](WASM.md)) se compile en CI et passe le test du wrapper sous Node, et a été exécuté une fois dans Chromium sans interface (une entrée invalide donne des erreurs, aucune fuite sur 60 cycles d'ouverture/fermeture, scènes trop grandes refusées), mais **n'a pas été examiné comme composant de navigateur et n'a pas été exécuté dans Firefox ni Safari** ; ses propriétés de sécurité ne sont donc connues que par la lecture du code et ce test. Il n'a par construction aucun accès au système de fichiers ni au réseau ; une panique y fait trapper l'instance.
* Aucune des deux couches ne place le *projet* dans un bac à sable : le rendu d'un projet hostile est borné par les limites ci-dessus, mais un projet peut encore demander beaucoup de travail dans ces limites (nombreux calques, grandes images, longues timelines).

## 9. Lacunes connues

Liste franche ; voir [LIMITATIONS.md](LIMITATIONS.md) pour la liste plus large des limites fonctionnelles.

1. **Pas de TLS ni de comptes** dans le studio. Depuis la vague 7, le jeton peut être propre à chaque participant et expirer (section 3), ce qui n'est toujours pas un système de comptes : il n'y a ni rôles, ni réinitialisation de mot de passe, ni TLS propre. Aucun TLS natif n'a été ajouté car aucune crate TLS compatible avec la licence ne figure dans `Cargo.lock` (en ajouter une introduirait une grosse nouvelle dépendance, et du code cryptographique, dans un projet AGPL/MPL sans audit) ; la section 12 montre la voie prise en charge, un proxy inverse qui termine le TLS.
2. **La temporisation est par adresse** et partagée derrière un proxy ; elle ralentit mais n'empêche pas les devinettes en ligne. Utilisez un long jeton aléatoire (le minimum de 16 caractères est un plancher, pas une recommandation).
3. **Le journal de collaboration est chaîné, non signé.**
4. **Base de code non auditée.** Aucun test d'intrusion par un tiers ni campagne de fuzzing formelle ; les contrôles sont les tests et les vagues de revue du projet.
5. **Dépendances.** La CI exécute `cargo deny` pour les avis de sécurité (vulnérabilités, crates retirées, dépendances directes non maintenues), les licences, les crates interdites (aucune pile TLS ou client HTTP) et les sources. Les avis de maintenance des dépendances transitives sont examinés à la main (voir `deny.toml`). Le contrôle ne connaît que les avis publiés jusqu'ici ; relancez-le avant de déployer.
6. **Le répertoire de ressources est global au processus.** `merge` sur plusieurs répertoires d'entrée utilise le dernier répertoire pour les ressources ; seul `write_json` réécrit les chemins de ressources (les autres écrivains ne le font pas).
7. **Repli non atomique** lorsque le répertoire de sortie n'est pas accessible en écriture.
8. **Les vérifications de chemin utilisent des chemins canoniques.** `assets::resolve` canonicalise les fichiers existants et refuse un fichier dont l'emplacement réel est en dehors du répertoire du projet : un lien symbolique situé dans un dossier de projet et pointant à l'extérieur est donc refusé (testé dans `assets.rs`) ; les liens qui restent à l'intérieur du dossier fonctionnent. Une course entre la vérification et la lecture qui suit (time-of-check to time-of-use) par quelqu'un pouvant modifier le dossier du projet en parallèle n'est pas exclue.
9. **ffmpeg** échappe à notre contrôle (section 7).
10. **WASM** n'est pas testé dans un moteur de navigateur (il ne tourne que sous Node en CI) ; le modèle de menaces navigateur d'une page qui l'intègre (CSP, limites mémoire) n'a pas été examiné.
11. **Déni de service par la taille.** Les limites bornent les requêtes et fichiers pris isolément, pas le travail total : un projet valide mais très lourd peut occuper un CPU longtemps (le plafond de connexions ci-dessus limite la concurrence, pas le coût d'une requête).

## 10. Signaler une vulnérabilité

* **N'ouvrez pas de ticket public** pour une vulnérabilité. Envoyez un rapport privé au mainteneur : **`alexio@trom.tf`** (l'adresse de l'avis affiché par `--version` du programme et des documents de licence), ou utilisez le signalement privé de vulnérabilités de GitHub, dans l'onglet *Security* du dépôt, lorsqu'il est activé.
* Indiquez : la version (`osatura --version`) et la plateforme, le projet ou la requête qui déclenche le problème (un fichier minimal est idéal), ce que vous attendiez et ce qui s'est passé, et si vous souhaitez être crédité.
* À quoi s'attendre *(intention, pas contrat : c'est un projet mené par une seule personne)* : un accusé de réception dans un délai raisonnable, un correctif développé en privé, un test de régression ajouté à la suite, une note dans [CHANGELOG.md](CHANGELOG.md), et un crédit si vous le souhaitez. Laissez le temps d'un correctif avant de publier les détails.
* Versions prises en charge : seules la dernière préversion (actuellement `v0.1.0-alpha.4`) et l'arborescence actuelle sur `main` sont prises en charge.
* Les signalements concernant l'exposition réseau du studio lorsqu'il est délibérément lié à une adresse publique sans proxy inverse relèvent d'abord de la documentation : voir la section 11.

## 11. Liste de durcissement pour le déployeur

Pour quiconque exécute `osatura studio` pour plus d'une personne ou en dehors de localhost.

1. [ ] Préférez **ne pas** l'exposer. Pour un seul utilisateur : gardez l'adresse d'écoute par défaut (`127.0.0.1`) et utilisez un tunnel SSH (`ssh -L 8080:127.0.0.1:8080 host`).
2. [ ] Si vous l'exposez, **terminez le TLS** dans un proxy inverse (nginx, Caddy, ...) placé devant ; n'exposez jamais le port HTTP en clair sur Internet.
3. [ ] Utilisez un **long jeton aléatoire** (par exemple 32 caractères ou plus issus de `openssl rand -base64 24`), transmettez-le par un fichier protégé ou l'environnement du service plutôt que par un historique de shell, et changez-le quand des personnes partent. Rappelez-vous qu'il donne un accès complet en édition.
4. [ ] Démarrez avec `--bind 127.0.0.1` (ou l'interface privée) plus le proxy ; définissez `--allow-host your.domain` et `--public-origin https://your.domain` pour que les contrôles Host et Origin passent pour le proxy et pour rien d'autre.
5. [ ] Préférez des **jetons par participant** (`--collab-secret-file`, section 3) avec un `--ttl` court plutôt qu'un seul jeton partagé ; gardez le fichier secret en mode 600, hors du dossier servi, et désignez avec `--collab-host` les personnes autorisées à enregistrer, compacter et révoquer.
6. [ ] Faites **limiter le débit** par le proxy et, si possible, restreignez par IP ou ajoutez une seconde couche d'authentification (HTTP basic sur TLS, VPN, proxy SSO), car la temporisation intégrée ne voit que l'adresse du proxy.
7. [ ] Exécutez-le comme **utilisateur non privilégié** dans un répertoire dédié ne contenant que les projets à servir ; pas de secrets, pas de liens symboliques sensibles. Utilisez un système de fichiers en lecture seule ailleurs, un conteneur ou le bac à sable de systemd (`ProtectSystem`, `PrivateTmp`, `NoNewPrivileges`).
8. [ ] Gardez `--allow-external-assets` et `--follow-symlinks` **désactivés**.
9. [ ] Définissez `--source-url` vers l'endroit où les sources (éventuellement modifiées) peuvent être récupérées (AGPL section 13).
10. [ ] Rendez persistant avec `--collab-log` dans un répertoire privé, sauvegardez-le et conservez le mode 0600. Traitez le journal comme sensible : il contient tout l'historique des modifications.
11. [ ] Bornez les ressources au niveau du système d'exploitation (mémoire, temps CPU, fichiers ouverts, quota disque) ; les plafonds intégrés limitent les requêtes isolées, pas la charge totale.
12. [ ] Gardez à jour ffmpeg et la chaîne d'outils Rust utilisée pour la compilation ; exécutez `cargo deny check` (ou `cargo audit`) sur `Cargo.lock` pour votre version.
13. [ ] Ne donnez pas au serveur des projets de sources non fiables, sauf s'il s'exécute dans le bac à sable ci-dessus. Pour des déploiements sans scripts, compilez avec `--no-default-features`.
14. [ ] Surveillez dans les journaux les réponses 421, 403, 429 et 503 ; les pics signalent un sondage ou une mauvaise configuration.

## 12. Proxy inverse avec TLS : nginx et Caddy

Le studio parle HTTP en clair. Pour tout usage au-delà de votre propre machine, exécutez-le sur le bouclage et laissez un proxy terminer le TLS. Ces recettes ont été écrites à partir de la documentation publique des proxys et du comportement du studio décrit plus haut ; **elles n'ont pas été exécutées dans l'environnement de rédaction** (ni nginx ni Caddy, ni certificats, ni réseau) : testez-les avant de vous y fier.

Démarrez le studio sur le bouclage et nommez son hôte public, afin que les contrôles Host et Origin acceptent le proxy et rien d'autre :

```
osatura studio demo.json --port 8080 --bind 127.0.0.1 \
  --allow-host studio.example.org --public-origin https://studio.example.org \
  --collab-secret-file /etc/osatura/collab.secret --collab-log /var/lib/osatura/session.log
```

Avec un `--public-origin` en `https://`, le cookie de connexion reçoit l'attribut `Secure`. Le flux d'événements (`/api/collab/events`) est du Server-Sent Events : le proxy ne doit pas le mettre en tampon et doit autoriser de longues lectures.

**nginx** (certificats de votre autorité ou de `certbot`) :

```
limit_req_zone $binary_remote_addr zone=osatura:10m rate=20r/s;

server {
    listen 80;
    server_name studio.example.org;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name studio.example.org;
    ssl_certificate     /etc/letsencrypt/live/studio.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/studio.example.org/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    add_header Strict-Transport-Security "max-age=31536000" always;
    client_max_body_size 64m;

    location / {
        limit_req zone=osatura burst=60 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
    }
    location /api/collab/events {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 1h;
    }
}
```

**Caddy** (obtient et renouvelle lui-même le certificat ; `Caddyfile`) :

```
studio.example.org {
    encode zstd gzip
    header Strict-Transport-Security "max-age=31536000"
    request_body {
        max_size 64MB
    }
    reverse_proxy 127.0.0.1:8080 {
        flush_interval -1
        transport http {
            read_timeout 1h
        }
    }
}
```

`flush_interval -1` envoie le flux d'événements sans mise en tampon. Caddy définit `X-Forwarded-Proto` et transmet le `Host` d'origine. Pour une limite de débit des requêtes, utilisez le plugin `caddy-ratelimit` ou placez devant le `limit_req` de nginx.

Vérifications après le démarrage : `curl -I https://studio.example.org/` répond `401` (page de connexion) et ne sert jamais le studio sans jeton ; `curl -I http://127.0.0.1:8080/ -H 'Host: evil.example'` répond `421` ; le cookie défini par `/api/login` porte `Secure; HttpOnly; SameSite=Strict` ; le journal de session propre au studio reste privé (section 6). Si le proxy tourne sur une autre machine, liez le studio à l'interface privée que le proxy peut atteindre, filtrez le port par un pare-feu, et rappelez-vous que le studio lui-même parle toujours HTTP en clair sur cette liaison.
