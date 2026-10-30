# Notes sur les brevets

**Ceci n'est ni un avis juridique ni une opinion de liberté d'exploitation.** Ce document consigne ce qui n'a PAS été vérifié, afin que personne ne prenne l'absence de recherche pour une autorisation. Les licences d'OSATURA (MPL-2.0 pour les bibliothèques, AGPL-3.0-or-later pour la CLI) contiennent des concessions de brevets par les contributeurs pour le code propre de l'auteur, mais elles ne disent rien des brevets de tiers. Aucune recherche de brevets par un professionnel n'a été effectuée.

## Vérifications restant à faire

| Domaine | Pourquoi cela peut importer | Statut |
|---|---|---|
| Marionnettes pilotées par le visage (`faceparam`, `facedet`, `liveinput` : points de repère ou rapports du visage pilotant les paramètres du rig) | L'animation de personnages 2D pilotée par la performance est un domaine actif de dépôts commerciaux. | Non vérifié. Le code utilise des mesures publiées dans les manuels (rapport d'aspect de l'œil, filtre One-Euro, HOG + régression logistique) ; cela n'exclut pas des revendications sur la combinaison. |
| Machines à états et liaison de données (`statemachine`, `runtime_machine`, `bind`) | Les runtimes d'animation interactive avec machines à états, blend trees, entrées et liaison de view-model constituent un domaine de fonctionnalités commerciales. | Non vérifié. Conception propre du modèle de données, pas une autorisation vis-à-vis des brevets. |
| FABRIK, coordonnées de valeur moyenne, poids biharmoniques bornés (`rig`, `cage`, `weights`) | Algorithmes publiés ; certains auteurs et employeurs ont déposé des brevets autour des techniques de déformation et de skinning. | Non vérifié. FABRIK (Aristidou et Lasenby, 2011) et MVC (Floater, 2003) sont utilisés d'après les articles ; le module weights utilise une simple diffusion harmonique, et non le solveur biharmonique borné publié. |
| Remplissage de tracés par stencil puis couverture (`osatura-gpu`) | La technique provient de la recherche sur le rendu de tracés par GPU et d'extensions de fabricants. | Non vérifié. Utilisée ici uniquement via l'API wgpu standard avec un simple tampon stencil. |
| Codecs vidéo et audio via ffmpeg (`export` .mp4 / .webm, `soundtrack`) | H.264 et AAC sont couverts par des pools de licences (MPEG LA / Via LA et autres). VP9 et AV1 sont présentés comme libres de redevances, mais des revendications de pools de brevets et de brevets individuels ont été annoncées pour les deux. | Non évalué. Ne considérez pas VP9/AV1 comme autorisés. |

## ffmpeg
ffmpeg est un **sous-processus externe** fourni par l'utilisateur. OSATURA ne le lie pas, ne le distribue pas et ne doit pas l'inclure dans une version publiée : l'inclure rendrait le distributeur responsable de la licence propre à ffmpeg (LGPL/GPL selon la compilation) et des licences des codecs ci-dessus. Chaque option de codec relève du choix de l'utilisateur, sur sa propre installation.

## Avant une publication commerciale
Faites examiner les domaines ci-dessus par un conseil en propriété industrielle, dans les juridictions où le logiciel sera distribué, et consignez le résultat ici et dans CLEANROOM.md (règle 5).
