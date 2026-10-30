# Note despre brevete

**Aceasta nu este consultanță juridică și nici o opinie privind libertatea de operare.** Ea consemnează ce NU a fost verificat, ca nimeni să nu confunde absența unei verificări cu o avizare. Licențele OSATURA (MPL-2.0 pentru biblioteci, AGPL-3.0-or-later pentru CLI) conțin acordări de brevete din partea contribuitorilor pentru codul propriu al autorului, dar nu spun nimic despre brevetele terților. Nu a fost făcută nicio căutare de brevete de către un profesionist.

## Verificări încă nefăcute

| Domeniu | De ce poate conta | Stare |
|---|---|---|
| Marionetizare comandată de față (`faceparam`, `facedet`, `liveinput`: repere sau rapoarte faciale care comandă parametrii rigului) | Animația personajelor 2D comandată de interpretare este un domeniu activ de depuneri comerciale. | Neverificat. Codul folosește măsuri publicate în manuale (raportul de aspect al ochiului, filtrul One-Euro, HOG + regresie logistică); acest lucru nu exclude revendicări asupra combinației. |
| Mașini de stări și legarea datelor (`statemachine`, `runtime_machine`, `bind`) | Runtime-urile de animație interactivă cu mașini de stări, blend tree-uri, intrări și legare la view-model sunt un domeniu comercial de funcționalități. | Neverificat. Proiectare proprie a modelului de date, nu o avizare privind brevetele. |
| FABRIK, coordonate cu valoare medie, ponderi biarmonice mărginite (`rig`, `cage`, `weights`) | Algoritmi publicați; unii autori și angajatori au depus brevete pentru tehnici de deformare și de skinning. | Neverificat. FABRIK (Aristidou și Lasenby, 2011) și MVC (Floater, 2003) sunt folosiți conform lucrărilor; modulul weights folosește difuzie armonică simplă, nu solverul biarmonic mărginit publicat. |
| Umplerea traseelor stencil-apoi-acoperire (`osatura-gpu`) | Tehnica provine din cercetarea privind randarea traseelor pe GPU și din extensiile furnizorilor. | Neverificat. Folosită aici doar prin API-ul standard wgpu, cu un buffer de stencil simplu. |
| Codecuri video și audio prin ffmpeg (`export` .mp4 / .webm, `soundtrack`) | H.264 și AAC sunt acoperite de grupuri de licențiere (MPEG LA / Via LA și altele). VP9 și AV1 sunt promovate ca fiind fără redevențe, dar pentru ambele au fost anunțate revendicări ale unor grupuri de brevete și revendicări individuale. | Neevaluat. Nu trata VP9/AV1 ca fiind avizate. |

## ffmpeg
ffmpeg este un **subproces extern** furnizat de utilizator. OSATURA nu îl leagă, nu îl livrează și nu trebuie să îl includă în nicio lansare: includerea l-ar face pe distribuitor responsabil pentru licența proprie a ffmpeg (LGPL/GPL, în funcție de compilare) și pentru licențele codecurilor de mai sus. Fiecare opțiune de codec este alegerea proprie a utilizatorului, pe instalarea proprie a acestuia.

## Înainte de lansarea comercială
Cere unui avocat specializat în brevete să examineze domeniile de mai sus, în jurisdicțiile în care va fi distribuit software-ul, și consemnează rezultatul aici și în CLEANROOM.md (regula 5).
