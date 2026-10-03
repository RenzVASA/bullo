# Bullo — état réel des fonctionnalités

Légende : **Fait** = testé automatiquement dans un navigateur avec un faux backend Tauri (110 vérifications) ; **À valider sur Mac** = code écrit, mais dépend du Mac réel (Rust non compilé ici, micro, whisper, Ollama, notifications).

| Fonctionnalité | Statut | Remarque |
|---|---|---|
| Renommage Bullo (config, textes, exports, clés) | Fait | Exceptions documentées : migration `focusflow.db`, variables `FOCUSFLOW_*` en repli |
| Logo (sidebar, favicon, icônes) | Fait | Logo vectoriel Bullo. Ton `logo-bullo.jpeg` n'a pas été fourni : dépose-le dans `src/assets/` puis `npm run logo` |
| Enregistrement, sauvegarde du fichier audio | À valider sur Mac | Chemin conservé dès l'arrêt |
| Transcription (ffmpeg + whisper) | À valider sur Mac | Nécessite les outils installés |
| Résumé IA (Ollama) | À valider sur Mac | Message clair si Ollama est absent |
| Réécoute de l'original (après navigation et redémarrage) | Fait (simulé) | Relue depuis le disque à chaque fois ; message explicite si le fichier manque |
| Lecture vocale (résumé, transcription) | Fait (simulé) | Découpée en phrases, pause/reprise/arrêt, erreurs explicites. Dépend des voix du Mac |
| Rappels et alarmes (création, report, édition, onglets) | Fait (interface) | Base SQLite persistante |
| Notifications natives, fenêtre en arrière-plan | À valider sur Mac | Thread Rust + `osascript` ; autoriser « Éditeur de script » dans les notifications |
| Détection des dates dans les devoirs | Fait | « demain 14h », « vendredi », « le 12 octobre », « 12/10 », « dans 2 jours »… |
| Fond sonore (barre fixe, 6 sons, réduire, persistance, pause pendant l'enregistrement) | Fait (simulé) | Sons générés, aucun fichier |
| Polices : OpenDyslexic, Verdana, Arial, Comic Sans, Dyslexie | Fait | Chaque police est vérifiée ; Dyslexie est commerciale (à installer toi-même) |
| Polices : Lexend, Atkinson Hyperlegible | Fait (simulé) / À valider sur Mac | Bouton de téléchargement dans Réglages ; seules les polices disponibles sont listées. Les URL de téléchargement réelles n'ont pas été testées |
| Import d'image : installation de Tesseract en un clic | À valider sur Mac | Flux testé en simulation ; `brew install` jamais exécuté ici |
| Mini lecteur : Son de Bullo / Son de l'ordinateur, détection de la musique (Music, Spotify, onglets navigateur, son système), titre, lecture/pause/suivant/précédent | Fait (simulé) / À valider sur Mac | AppleScript, JXA (touches média), `pmset` : jamais exécutés ici. Autorisations Automatisation + Accessibilité à accorder. Titre d'un navigateur = titre de l'onglet |
| Coupure de tout le son du Mac pendant l'enregistrement (+ rétablissement, filet de sécurité après crash) | Fait (simulé) / À valider sur Mac | `osascript` (sourdine) jamais exécuté ici |
| Boucle, aléatoire, favoris du lecteur (Music/Spotify natifs, favoris Bullo) | Fait (simulé) / À valider sur Mac | Favori natif : Apple Music seulement ; AppleScript non exécuté ici |
| Opera / Opera GX / Vivaldi + Diagnostic du lecteur | À valider sur Mac | Le lecteur intégré d'Opera GX n'est pas détectable par titre |
| Empreinte SHA-256 des clés Google (intégrité) | Fait (simulé) / À valider sur Mac | Code Rust non compilé |
| À propos (vibe codé + IA) et logo (barre latérale, favicon, icônes de l'app) | Fait | Régénérer les icônes avec `npm run icon` si besoin |
| Détection Music/Spotify sans AppleScript (pgrep) | À valider sur Mac | Correctif de l'erreur « Le lecteur n'a pas répondu » ; non reproduit ici |
| Bien-être : parking de pensées, respiration, pauses douces | Fait (simulé) | Raccourci Ctrl+Maj+P à vérifier sur Mac |
| Capture et outils : travail conservé en changeant de page, puis au redémarrage | Fait (simulé) | Le bug de changement de page n'a pas pu être reproduit dans la simulation ; l'état est maintenant sauvegardé en continu |
| Taille, interlignage, espacement, thèmes, daltonien, contraste élevé | Fait | Persistance et assainissement des valeurs testés |
| Raccourcis (personnalisation, conflits) | Fait | |
| Pomodoro flottant | Fait | Désactivé par défaut |
| Exports .txt, .md, .pdf (accents, plusieurs pages) | Fait (simulé) | PDF validé avec qpdf et pdftotext. Écriture dans Téléchargements : commande Rust à valider sur Mac |
| Sauvegarde / restauration des notes et cours | À valider sur Mac | Bug corrigé : l'import échouait à cause d'un paramètre SQL en trop. Ne contient pas les rappels ni l'audio |
| Google Drive : tutoriel, clés dans le Trousseau, OAuth, sauvegarde et restauration | À valider sur Mac | Interface et enchaînement testés en simulation ; OAuth, Trousseau et API Drive jamais exécutés pour de vrai |
| Migration depuis FocusFlow | À valider sur Mac | Copie de la base et des réglages |

Aucune compilation Rust n'a pu être exécutée dans l'environnement de construction (le registre de crates est inaccessible). Seules la syntaxe (rustfmt) et quelques fonctions pures ont été vérifiées. Premier réflexe sur ton Mac : `npm run tauri dev` et lis les éventuelles erreurs de compilation.



| Lecteur léger (v11) | À valider sur Mac | Un seul `pmset` par relevé ; seule l'appli qui joue est interrogée ; titres d'onglet relus toutes les 10 s, pause toutes les 15 s, volume toutes les 15 s, relevé toutes les 4 s. Logique de tri testée à part, Rust non compilé |

## Ce qui n'a pas pu être fait / limites connues
- Rust jamais compilé ici (registre de crates bloqué) : seul rustfmt et quelques fonctions pures ont été vérifiés. Premier `npm run tauri dev` = corriger les éventuelles erreurs.
- Jamais exécuté pour de vrai : OAuth Google, Trousseau, API Drive, AppleScript/JXA (touches média), `pmset`, `brew install`, URL de téléchargement des polices, `fetch_ics`, fenêtre Pronote.
- Lecteur intégré d'Opera GX (barre latérale) : pas un onglet, donc titre indétectable ; bouton Diagnostic pour voir ce que Bullo détecte.
- Bug « capture perdue en changeant de page » : jamais reproduit en simulation ; l'état est désormais sauvegardé en continu.
- « Hachage » de la clé Google : une clé qu'on doit relire ne peut pas être hachée ; elle est chiffrée dans le Trousseau + empreinte SHA-256 de contrôle.
- Anciennes suites de tests t1–t4 perdues ; t5–t12 (~190 vérifications) passent.
- Logo : le fichier d'origine n'a jamais été reçu ; l'image envoyée dans le chat est utilisée.
- Dyslexie est une police payante. Le nom « Renz-VASA » vient de la dictée vocale (`ABOUT_CREDIT` dans `src/index.html`).
- Les noms de menus dans les tutoriels Google Cloud n'ont pas été vérifiés.
- Pronote et carnet scolaire : retirés à ta demande (pas possible pour le moment : pas d'API publique, protocole non officiel).
- Les favoris du lecteur ne sont pas dans la sauvegarde Google Drive.
- Refonte visuelle v10 : vérifiée sur captures (clair, sombre, contraste élevé, fenêtre étroite) ; jamais vue dans la vraie fenêtre Tauri/macOS.