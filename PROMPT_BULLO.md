# Prompt maître de Bullo

Ce prompt permet de régénérer l'application entière dans une conversation vierge. Colle tout ce qui suit.

````
Tu es un ingénieur logiciel senior (Rust, Tauri 2, accessibilité). Construis l'application COMPLÈTE « Bullo », de A à Z, prête à lancer. Ne me pose pas de questions : fais des choix raisonnables, note-les dans le README, et livre un projet qui compile.

# 1. Produit
Bullo est un espace de concentration neuro-inclusif pour élèves et étudiants (dyslexie, TDAH, basse vision). L'utilisateur enregistre un cours ou une idée, Bullo transcrit, résume, extrait les devoirs et les dates, crée des rappels, et lit le tout à voix haute. Tout est local : aucune donnée n'est envoyée sur internet.

# 2. Stack et cible
- Tauri 2 + Rust (backend), HTML/CSS/JavaScript sans framework ni bundler (frontend dans src/), SQLite via rusqlite.
- Cible n°1 : macOS Apple Silicon (Mac M1). Le code doit rester portable (Windows/Linux plus tard) : isole tout ce qui est spécifique à macOS (sélecteur de fichiers, `open`, chemins Homebrew) dans des fonctions clairement nommées.
- Identité : nom « Bullo » partout (productName, titre de fenêtre, package.json, Cargo.toml, <title>, textes, exports, base `bullo.db`, clés de stockage `bullo_*`). Aucune trace de « FocusFlow ».

# 3. Fonctionnalités (toutes obligatoires)
1. **Capturer** : enregistrement micro, arrêt, sauvegarde réelle du fichier audio sur disque (dossier de l'app, modifiable dans les réglages), puis transcription locale (ffmpeg + whisper.cpp, français). Le chemin du fichier est conservé dès l'arrêt, même si la transcription échoue. Possibilité de coller ou d'importer un texte (PDF, Word, texte, image).
2. **Résumé IA locale** (Ollama, qwen2.5:7b par défaut) : résumé simple en 3-4 phrases, « à retenir », devoirs, échéances, questions pour le professeur. Messages d'erreur clairs si Ollama est absent.
3. **Alarmes et rappels** : à partir du texte ou du résumé, détecte les devoirs et dates (« contrôle vendredi », « à rendre le 12 ») et propose de créer des rappels. Rappels persistés en base, notifications natives au bon moment (même fenêtre en arrière-plan), liste Aujourd'hui / À venir / Terminés, modification, report (snooze) et suppression. Alarme simple (heure ou délai) créable à la main.
4. **Lecture** : bouton « Écouter l'original » (relit le fichier audio enregistré) et bouton « Écouter le texte » (voix de synthèse du Mac, voix française, vitesse réglable, pause/reprise/arrêt). Les deux doivent fonctionner juste après l'enregistrement, après navigation, et après fermeture/réouverture de l'app. Ne JAMAIS conserver une URL blob comme référence durable : relire le fichier depuis le disque via une commande Rust à chaque lecture. Si le fichier est absent, message explicite (« Audio introuvable… »), jamais « Error ».
5. **Cours et notes** : « Enregistrer comme note », « Enregistrer dans un cours » (liste de cours modifiable), carte unique par cours (résumé, à retenir, transcription, audio, devoirs), recherche plein texte avec filtre par cours, tags/couleurs, brouillon automatique.
6. **Outils d'aide** : corriger, simplifier, décomposer en étapes, « je suis bloqué », consigne de devoir, réviser (questions/réponses).
7. **Exports et sauvegardes** : transcription .txt (UTF-8), export global .md + .pdf (accents corrects, plusieurs pages), sauvegarde/restauration JSON de la base avec contrôle d'intégrité SHA-256. Les fichiers exportés sont écrits dans Téléchargements via une commande Rust (les liens <a download> ne sont pas fiables dans Tauri) et le chemin est affiché.
8. **Pomodoro flottant** optionnel (désactivé par défaut), déplaçable, qui ne bloque aucun clic, durées de travail/pause réglables, état conservé quand on change de page.
9. **Mode une seule tâche** (avec bouton de sortie toujours visible) et rappel d'inactivité réglable.

# 4. Mini lecteur « Fond sonore » (barre fixe en bas, style mini-player Spotify)
- Exactement DEUX sources, choisies dans un sélecteur de la barre : « Son de Bullo » (ambiances générées) ou « Son de l'ordinateur ». Mode ordinateur : commande Rust `now_playing` qui détecte, dans l'ordre : Music/Spotify natifs (pgrep + AppleScript par identifiant, titre/artiste/état), puis le titre du premier onglet musique/vidéo des navigateurs ouverts (Safari, Chrome, Arc, Brave, Edge ; hôtes open.spotify.com, music.youtube.com, youtube.com/watch, music.apple.com, deezer, soundcloud, tidal, qobuz ; titre nettoyé, « Titre • Artiste »), puis un son en cours via `pmset -g assertions` (assertions « audio »). Contrôles : AppleScript pour Music/Spotify, et pour tout le reste les TOUCHES MÉDIA système (JXA : NSEvent systemDefined, codes 16/17/18 via CGEventPost), donc pause/suivant/précédent marchent aussi sur un navigateur. Aucun bouton « Ouvrir » : ▶ envoie toujours lecture/pause. Curseur = volume système (`set volume output volume`). Interrogation toutes les 3 s. NSAppleEventsUsageDescription dans Info.plist ; explique les autorisations Automatisation et Accessibilité en cas de refus. Le logo Bullo (src/assets/bullo-player.png) sert de pochette, avec une pastille 🖥 en mode ordinateur.
- Barre fixe en bas de la fenêtre, visible sur toutes les pages, qui ne masque jamais le contenu (marge basse réservée).
- Sons d'ambiance GÉNÉRÉS par Web Audio, sans fichier ni réseau : pluie, bruit blanc, bruit rose, bruit brun, café, nature. Jamais de « Choose File ».
- Contrôles : lecture/pause, précédent, suivant (boucle sur la liste), volume (effet immédiat), nom du son et état (en lecture / à l'arrêt / erreur).
- Flèche pour réduire/afficher la barre (aria-expanded, aria-label), état mémorisé. Son et volume par défaut mémorisés et réglables dans Réglages > Fond sonore.
- Totalement indépendant de l'audio enregistré. Option « couper le son de l'ordinateur pendant un enregistrement » (activée par défaut) : pause de l'ambiance Bullo, pause de Music/Spotify s'ils jouent, ET sourdine système (`set volume output muted true`) pour couper aussi un navigateur ; à l'arrêt, tout est rétabli à l'identique (jamais de dé-sourdine si le Mac l'était déjà), avec un drapeau en localStorage qui rend le son au démarrage suivant si l'appli a planté.
- Pas de fuite : à chaque changement de son, les anciens nœuds audio sont détruits.

# 5. Réglages (une page, sections exactes)
Apparence · Pomodoro · Lecture vocale · Google et sauvegardes · Raccourcis clavier · Fond sonore · Confort et concentration · Profil et confidentialité (prénom, code d'accès) · Enregistrements et outils (dossier audio, vérification ffmpeg/whisper) · À propos (histoire de Bullo, mentions légales, version lue depuis le build).
- **Apparence** : thème clair/sombre/système, taille du texte, interlignage, espacement des lettres, couleur d'accent, mode daltonien (palettes réellement adaptées, pas un simple filtre), contraste élevé (noir/blanc, focus renforcé), thème doux, réduction des animations, masque de lecture, bouton de réinitialisation.
- **Polices** : Bullo (par défaut), OpenDyslexic, Dyslexie (si installée), Lexend, Atkinson Hyperlegible, Verdana, Arial, Comic Sans MS, police du système. Embarque OpenDyslexic ; Lexend et Atkinson sont téléchargés depuis l'appli (bouton dans Réglages, dossier de données, chargés via FontFace) : la liste ne propose QUE les polices réellement disponibles et explique que Dyslexie est payante. CHAQUE police est vérifiée pour de vrai (document.fonts.load pour les embarquées, détection de largeur pour les polices système) : l'écran affiche « ✅ chargée et utilisée » ou « ❌ introuvable », jamais de repli silencieux. Un aperçu en direct montre le rendu. Si la police sauvegardée disparaît, prévenir et revenir à la police par défaut.
- **Persistance** : tous les réglages sont sauvegardés et réappliqués au démarrage, avec validation des valeurs lues (bornes, valeurs inconnues) pour qu'un réglage corrompu ne casse jamais l'app.
- **Raccourcis** : liste, personnalisation par capture de touches, détection des conflits, Ctrl/Cmd+K réservé à la recherche, aucun raccourci actif quand on saisit du texte sans modificateur.
- **Google Drive (réel)** : tutoriel intégré en 6 étapes (projet Cloud, API Drive, écran de consentement + utilisateur test + scope drive.file + publication, ID client « Application de bureau », envoi des clés, connexion), boutons « Ouvrir la page » (liste blanche d'URL Google). OAuth 2.0 PKCE S256 avec écoute locale 127.0.0.1, dossier « Bullo » et fichier bullo-sauvegarde.json (notes, cours, rappels) avec SHA-256. Clés et jeton rangés dans le Trousseau macOS (commande `security`, relecture de contrôle), jamais en clair dans un fichier ; explique à l'utilisateur pourquoi chiffrer plutôt que hacher. Restauration avec copie de sécurité préalable, confirmation en deux clics, sauvegarde auto optionnelle, déconnexion, effacement des clés, annulation de la connexion en cours. Aucun secret dans le dépôt.
- **Empreinte des clés Google** : SHA-256 de « bullo|id|secret » stockée dans le Trousseau et revérifiée à chaque lecture ; connexion refusée si elle diffère ; affichée (16 caractères) dans Réglages.
- **À propos** : logo, version, « Vibe codé par Renz-VASA, avec un peu d'intelligence artificielle ✨ ».
- **Détection des lecteurs** : processus via `pgrep -x` (jamais d'AppleScript sur une appli absente), AppleScript par identifiant (com.apple.Music, com.spotify.client) avec timeout.
- **Lecteur, suite** : boutons boucle/aléatoire/favori. Music : `song repeat` (off/all/one en cycle), `shuffle enabled`, `favorited of current track` ; Spotify : `repeating`, `shuffling` (pas de favori scriptable → favoris Bullo). Navigateurs/son système : boucle et aléatoire grisés avec explication. Favoris Bullo (localStorage, assainis) : titres et ambiances, liste dans Réglages avec « Rechercher » (https Spotify). Navigateurs reconnus : Safari, Chrome, Arc, Brave, Edge, Opera, Opera GX, Vivaldi (détection par chemin d'exécutable). Commande `player_diagnostic` + `media_key_test` et bouton « Diagnostic du lecteur » pour rendre visibles les erreurs d'autorisation.
- **Bien-être (onglet dédié)** : parking de pensées (raccourci global Ctrl+Maj+P, notes préfixées 💭, « en faire une tâche »/supprimer), respiration guidée (carrée, 4-7-8, 5-5, compte à rebours texte, sans mouvement si animations réduites), pauses douces optionnelles (rappel d'eau/étirement/regard au loin, suspendues pendant un enregistrement).
- **Travail en arrière-plan** : captures, transcription, résumé et imports continuent quand on change de page (état conservé en mémoire ET sauvegardé pour survivre à un redémarrage, notification quand c'est fini ailleurs).
- **Installer en un clic** : quand un outil manque (tesseract, ffmpeg…), l'erreur Rust est de la forme `[[install:id]]message` et l'interface propose « Installer automatiquement et réessayer » (Homebrew).

# 6. Accessibilité et qualité (non négociable)
- Tout est utilisable au clavier, focus visible, libellés aria, rôles status/alert pour les messages, jamais la couleur seule pour porter une information.
- Pas de alert()/prompt()/confirm() : remplace-les par des boîtes et notifications intégrées. Confirmation en deux clics pour les suppressions et la restauration.
- Chaque action a un état de chargement, de succès et d'échec visibles ; aucun bouton ne reste bloqué ; les clics répétés ne dupliquent rien.
- Aucune erreur dans la console au démarrage ni pendant la navigation. Toute erreur affichée à l'utilisateur est une phrase française explicite et actionnable.
- Textes d'interface en français, tutoiement, ton bienveillant.

# 7. Identité visuelle
Logo : utilise le fichier que je fournis (logo-bullo.jpeg dans mes Téléchargements). Écris un script `scripts/generate-logo-variants.js` (relatif au projet, sans chemin absolu) qui génère logo-bullo.png, -512, -256, -64, favicon.ico et toutes les icônes Tauri (icns, ico, png) à partir de ce fichier. Si le fichier est absent, dis-le et utilise un logo vectoriel provisoire, jamais un « ? ».
Style : doux et calme, fond clair crème ou sombre profond, dégradé accent vert-sauge/violet doux, grandes zones cliquables, coins arrondis, animations discrètes et désactivables.

# 8. Livraison
1. Arborescence complète et tous les fichiers, sans « … » ni morceaux omis.
2. `README.md` : prérequis Mac M1 (Xcode CLT, Rust, Node, `brew install ffmpeg whisper-cpp ollama poppler tesseract`, modèle whisper, `ollama pull qwen2.5:7b`), puis les commandes exactes : `npm install`, `npm run tauri icon <image>`, `npm run tauri dev`, `npm run tauri build`.
3. `scripts/check-frontend.js` (`npm run build`) qui vérifie la syntaxe du JavaScript.
4. `PROTOCOLE_DE_TEST.md` : une case à cocher par fonctionnalité, avec le résultat attendu.
5. Avant de répondre, relis ton code pour : références « FocusFlow » restantes, noms de commandes Rust appelés mais non déclarés dans `generate_handler!`, variables non définies, placeholders SQL en trop, chemins absolus, fuites de ressources audio. Corrige, puis liste honnêtement ce que tu n'as pas pu tester (micro réel, whisper, Ollama, notifications natives sur Mac).
6. Termine par un tableau « fonctionnalité → statut réel (fait / partiel / non fait) ». Ne déclare jamais « fait » ce que tu n'as pas vérifié.
````


## Design v10
Design « nuit douce » : palette du logo (bleu nuit #090e2b, violet #5b4bf5/#9a90ff, cyan #0e9fd8/#46cdfb), rail de navigation flottant avec icônes, cartes de premier niveau empilées (deux feuilles décalées dessous, écho du logo), lecteur en dock flottant avec 4 barres de son animées pendant la lecture, boutons en pilule. Les variables `--bg --panel --ink --mute --line --accent --accent2 --on --rec` sont la seule source de couleur ; contraste élevé, thème doux, daltonisme et mouvement réduit les surchargent. Pas de Pronote ni de carnet scolaire.
## Lecteur léger (v11)
`now_playing` ne lance qu'une commande `pmset -g assertions` par relevé, en déduit l'application qui joue (`classify`) et n'interroge que celle-ci (AppleScript Music/Spotify, ou titre d'onglet du seul navigateur concerné, en cache 10 s). Sans son : un `pgrep -lx "Music|Spotify"` au plus toutes les 15 s. Jamais de parcours de tous les navigateurs. Cache invalidé après une commande de lecture. Relevé toutes les 4 s, volume système toutes les 15 s.
