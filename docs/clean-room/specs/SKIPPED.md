# Funcții deschise care NU au primit specificație în lotul 1

Data: 2026-11-09. Selecția pornește din ROADMAP.md (secțiunea 9, „Not started"), FEATURES.md și docs/LIMITATIONS.md. Motivele sunt cele din cerințele sesiunii: referință numai proprietară, preocupare de brevete, nevoie de date reale, blocaj de sursă sau decizie care nu ține de specificare. „Nespecificat din lipsă de timp/prioritate" este scris ca atare, nu ca motiv tehnic.

## A. Singura referință este comportamentul unui produs proprietar (sau nu există standard public)
| Funcție | Unde apare | Motiv |
|---|---|---|
| Formate native ale altor programe de animație (.moho, .riv, Spine, Live2D, Harmony, Cartoon Animator) | ROADMAP §1, §6; LIMITATIONS | Format proprietar; neacoperit de standard public; exclus prin regulile proiectului. |
| Unelte de desen, pensule, vopsea, buclă de linie, remodelări de interfață, „Smart Bones", „Live Playback", storyboard, Actions, „Layer Selector", „Magnet falloff", conectori etc. din tabelele de concurență | FEATURES.md (rânduri ❌ din tabelele de comparație) | Singura referință ar fi comportamentul/documentația unui produs proprietar. |
| Pictograme/aspect/interacțiuni ale studioului: reguli de tragere a gizmo-urilor, pensulă de ponderi, editor grafic de curbe, editor de mașini de stări, fixarea elementelor (L-14–L-17, L-40, L-49) | LIMITATIONS | Specificația ar copia în practică interfața unor programe cunoscute; nu există standard public. |
| Mutare a pivotului/„proportional resize"/„glue" pe suprafață (L-37), ancorări de os (FEATURES) | LIMITATIONS | Fără standard; sursele sunt manuale de produs. |
| Conversia „în loc" a buclelor de mers și extinderea de membre lider/urmăritor (L-39) | LIMITATIONS | Nicio sursă publică standardizată; ar fi design propriu, nu specificare din sursă. |
| Certificare de fotosensibilitate de nivel difuzare | ROADMAP §6 | Referința este un instrument proprietar; definițiile publice sunt deja implementate (WCAG 2.3.1). |

## B. Preocupare de brevete sau opinie juridică necesară
| Funcție | Motiv |
|---|---|
| Deformare rapidă de mesh tip ARAP pentru ilustrații | ROADMAP §2 o marchează 🔴 (brevete citate acolo). Nu se specifică. |
| Import PSD (R-035, L-32) | Proiectul însuși o amână ca întrebare juridică/de licență pentru formatul publicat de un furnizor; necesită opinie de PI. Nu s-a citit specificația. |
| Codare H.264/AAC/ProRes | ROADMAP §1: conducte de brevete; rămâne extern (ffmpeg la utilizator). |
| Decodare nativă MP3/OGG (L-10) | Alegere deliberată de a păstra ffmpeg extern; evaluarea brevetelor/licențelor codecurilor nu a fost făcută și nu se face aici. |
| Mașină de stări / legare de date (R-091) și alte zone din docs/PATENT_NOTES.md | Ecranare de brevete încă nefăcută. |
| Revizuire de licențiere (L-64) | Chestiune juridică, nu de specificare. |

## C. Necesită date reale, hardware sau utilizatori reali
| Funcție | Motiv |
|---|---|
| Intrare live: MIDI-learn, ieșire MIDI, sysex/presiune polifonică/schimbare de program, cameră web, gamepad (L-12, L-44) | Cere hardware real; specificația MIDI nu a fost accesibilă în sesiune. |
| Detector de fețe pe fotografii/video real (L-13, L-56) | Cere date reale (și considerente de confidențialitate). |
| Retargetare mocap cu fișiere reale (L-45) | Cere fișiere de captură reale. |
| Recunoaștere de foneme / aliniere vorbire reală (L-46) | Cere modele și date de vorbire; nu există standard de comportament. |
| GPU real, platforme reale, studio pe browsere și accesibilitate (L-04, L-05, L-11) | Verificare practică, nu specificație. |
| Executabile semnate/notarizate, proces de lansare, căutare de denumire (L-74, L-77, L-78) | Proces și juridic, nu funcție. |

## D. Blocat: sursa publică nu a putut fi citită sau nu definește comportamentul
| Funcție | Motiv |
|---|---|
| Import OCA (FEATURES: „import ⬜") | Paginile de prezentare au fost citite (depozitul formatului, instrumente care îl exportă), dar lista de câmpuri a formatului nu a putut fi accesată în sesiune (documentația formatului nu era pe lista de adrese permise). Fără câmpuri verificate nu se scrie specificație. |
| Moduri de mască „lighten", „darken", „difference" și detalii de mască (extindere, estompare) la importul Lottie (L-29) | Specificația publică citită (Lottie Specs 1.0, secțiunile Helpers și Constants) definește doar modurile none, add, subtract, intersect și nu menționează extinderea sau estomparea; modurile l/d/f apar fără descriere în documentația comunitară. Comportamentul este [necunoscut] din surse publice. |
| Profiluri ICC / gamă largă de culori (L-22) | Textul specificației ICC nu a fost accesibil în sesiune; nu se specifică din memorie un standard de formate binare. |
| Compresie OpenEXR ZIP/ZIPS/PIZ etc. (L-22) | Din surse citite se știe doar mărimea blocurilor (ZIPS: 1 linie, ZIP: 16 linii) din comentariile unui antet; descrierea pre-procesării octeților a fost găsită numai ca cod-sursă, nu ca text de specificație citit. |
| TLS nativ pentru colaborare (L-18) | Standardul există, dar implementarea proprie a criptografiei nu este recomandată fără audit; ruta corectă este o bibliotecă auditată, deci decizie de arhitectură, nu specificație. |

## E. Sursă publică există, dar nu a fost specificată în acest lot (prioritate/mărime; nu un motiv tehnic)
| Funcție | Sursă probabilă (nevăzută în sesiune) |
|---|---|
| Rasterizor SVG propriu (L-27) | W3C SVG 1.1/2 (mărime L) |
| Coliziuni de pânză: auto-coliziune, rupere, pliuri (L-48) | Literatură publică despre simularea pânzei; domeniu 3D, mărime L |
| Amestec 2D în arbori de amestec (L-49) | Interpolare ponderată după distanță (Shepard 1968) |
| Umbre din harta de normale, lumini direcționale suplimentare (L-24) | Texte despre ecuația de randare; nu s-a ales |
