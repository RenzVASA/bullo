# Historique des versions

Format : une section par version, la plus récente en haut.

## 1.6.3 — 2026-10-04

### Corrections
- **Notifications** : sur le Mac testé, la notification « envoyée » par le plugin Tauri n'apparaissait pas du tout. Bullo utilise maintenant l'API actuelle de macOS (UserNotifications) : au premier envoi, macOS demande l'autorisation pour **Bullo**.
- **Plus d'« Éditeur de script »** : si macOS refuse la notification, Bullo ne passe plus par osascript. Le son est joué, et la fenêtre de Bullo revient au premier plan avec le message du rappel.
- **Son des notifications personnalisé** : Bullo joue `src-tauri/sounds/notification.wav` (ou .mp3, .m4a, .aiff) ; à défaut, le son « Glass » de macOS.
- Le bouton « Tester la notification » est retiré (outil de diagnostic, inutile dans la version publique).
- En `npm run tauri dev` (pas une vraie application), les notifications macOS ne sont pas disponibles : le test se fait avec l'application compilée.

## 1.6.2 — 2026-10-04

### Diagnostic des notifications
- Le bouton « Tester la notification » (Rappels) dit maintenant **par quel chemin** la notification est partie : au nom de Bullo, ou par le secours de macOS (qui l'affiche au nom de « Éditeur de script »), avec le **motif** de l'échec dans ce cas. Ce motif permettra de corriger pour de bon.
- Ancien texte d'aide (« autorise Éditeur de script ») retiré.

## 1.6.1 — 2026-10-04

### Corrections
- **Notifications** : elles s'affichaient au nom de l'« Éditeur de script » de macOS, et un clic dessus ouvrait cette application. Elles passent maintenant par le système de notifications de Bullo : nom et icône de Bullo. Au premier rappel, macOS peut demander l'autorisation d'envoyer des notifications à **Bullo**.
- En `npm run tauri dev`, macOS peut encore attribuer la notification au Terminal : le test fiable se fait avec l'application installée.

### Présentation
- README, site et feuille de route mis à jour (le badge de version suit désormais automatiquement la dernière Release ; le site présente Réviser, la barre de menus et les modules facultatifs).

## 1.6.0 — 2026-10-04

### Nouveautés
- **Icône dans la barre de menus du Mac** (en haut, à côté de l'heure et de la batterie) avec des actions rapides : **Enregistrer / Arrêter l'enregistrement**, **Note rapide…**, **Poser une pensée…**, **Ouvrir Bullo**, **Quitter Bullo**. « Note rapide » et « Poser une pensée » ouvrent une petite fenêtre (Entrée pour valider, Échap pour fermer) ; le texte arrive dans l'Inbox. Pendant un enregistrement, l'icône affiche « ● REC ».
- **Fermer la fenêtre ne ferme plus Bullo** : il reste dans la barre de menus (réglable dans Réglages › Confort, case « Garder Bullo dans la barre de menus »). Un clic sur l'icône du Dock rouvre la fenêtre. « Quitter Bullo » ferme vraiment l'application.
- **Réviser** (nouveau module, masquable) : Bullo fabrique avec l'IA locale (Ollama) des cartes question/réponse à partir d'un cours ou d'une note — bouton « Créer des cartes » sur chaque cours, ou page Réviser. Tu relis, corriges ou décoches les cartes avant de les garder. Ensuite Bullo te les repose à intervalles croissants (système de Leitner : 1, 3, 7, 14 puis 30 jours ; une carte ratée revient tout de suite). Séances courtes de 10 cartes, lecture à voix haute possible, aucune « série » à maintenir.
- Les cartes sont comprises dans les **sauvegardes** (fichier et Google Drive).
- Une notification système prévient quand une explication ou une réponse est prête alors que la fenêtre est cachée.

### À savoir
- Cargo.lock change au premier build (nouvelles fonctions de Tauri pour la barre de menus) : à enregistrer avec le reste.
- Version Mac uniquement ; la version Windows est un chantier à part.

## 1.5.0 — 2026-10-04

### Nouveautés
- **Mode épuré** : un bouton « Mode épuré » dans le menu (et une case dans Réglages › Confort) réduit le menu à des icônes et retire les textes d'aide, le lecteur de sons et les bandeaux. Tes notes et tes contenus ne changent pas.
- **Sauvegarde complète** : les sauvegardes (fichier et Google Drive) contiennent maintenant aussi tes séances d'**Explique-moi**, en plus des notes, cours et rappels. Les fichiers audio restent hors sauvegarde. Avant toute restauration, l'état actuel est copié (`avant-restauration-….json`), et une ancienne sauvegarde ne supprime plus les rappels ni les séances qu'elle ne contient pas.

### Légal et accessibilité
- **Licences des polices** : les textes de la licence SIL OFL de Bricolage Grotesque, Atkinson Hyperlegible et OpenDyslexic sont maintenant fournis avec l'application (`src/fonts/licences/`) et listés dans `THIRD_PARTY_NOTICES.md`. Une mention est ajoutée dans Réglages › À propos.
- **Boutons du menu nommés pour les lecteurs d'écran** : en fenêtre étroite ou en mode épuré, les libellés disparaissent ; chaque bouton garde maintenant un nom accessible et une infobulle.

### À savoir
- Les mises à jour signées de la 1.4.0 sont utilisées pour la première fois avec cette version : si l'installation en un clic échoue, le message l'explique et le téléchargement manuel reste proposé.

## 1.4.0 — 2026-10-04

### Sécurité
- **Mises à jour signées** : chaque nouvelle version est maintenant signée avec une clé secrète qui ne quitte jamais l'ordinateur du développeur. Avant d'installer quoi que ce soit, Bullo vérifie cette signature : une version modifiée ou publiée par quelqu'un d'autre est refusée, même si elle vient de GitHub.
- La mise à jour s'installe et Bullo redémarre directement, sans script intermédiaire.

### À savoir
- Le passage de la 1.3.3 à la 1.4.0 se fait encore avec l'ancienne méthode (empreinte SHA-256). Toutes les mises à jour suivantes seront signées.

## 1.3.3 — 2026-10-04

### Nouveautés
- **Mise à jour en un clic** : le bandeau et le bouton des Réglages proposent maintenant « Installer et redémarrer ». Bullo télécharge la nouvelle version, vérifie son empreinte de sécurité (SHA-256, publiée avec la version), se ferme, remplace l'application, retire le blocage de macOS et se rouvre tout seul. Les notes et les réglages ne sont pas touchés. « Voir les détails » ouvre la page de la version.
- Si quelque chose ne va pas (pas d'internet, empreinte différente, droits insuffisants), rien n'est installé à moitié : l'ancienne version reste en place, un message clair s'affiche et le téléchargement manuel reste proposé.

### À savoir
- La mise à jour automatique n'existe qu'à partir de cette version : pour passer d'une version plus ancienne à la 1.3.3, il faut encore installer à la main une dernière fois.

## 1.3.2 — 2026-10-04

### Corrections
- **Résumé IA incomplet sur les longs textes** : Ollama lit par défaut très peu de texte d'un coup et coupe le reste sans prévenir. Bullo demande maintenant une fenêtre de lecture adaptée à la taille du texte, et au-delà d'environ 40 000 caractères le texte est découpé en parties résumées une par une puis fusionnées, au lieu de perdre la fin. Le résumé est aussi invité à couvrir tout le texte, du début à la fin.
- **Outils** : un texte trop long pour un outil affiche un message clair au lieu d'être coupé en silence.
- **Inbox** : les notes longues étaient coupées à 160 caractères sans moyen de lire la suite. Il y a maintenant « Lire la suite » / « Réduire » et un bouton **🔊 Écouter** qui lit la note en entier.
- **Inbox** : une phrase longue écrite sur une seule ligne n'était gardée que sur ses 120 premiers caractères. Elle est maintenant conservée en entier.

## 1.3.1 — 2026-10-04

### Amélioration
- **Salutation selon l'heure** sur l'Accueil et l'écran de verrouillage : « Bonjour » le jour, « Bonsoir » à partir de 18 h, avec un petit mot (Bonne matinée, Bon après-midi, Bonne soirée). Si un prénom est renseigné, il s'ajoute : « Bonjour Léa ».
- Le bouton **Réglages** du menu garde toujours son nom : le prénom n'y apparaît plus.

## 1.3.0 — 2026-10-04

### Nouveautés
- **Écran de bienvenue au premier lancement** (3 étapes, on peut tout passer) : prénom facultatif, choix de ce qu'on veut voir, et code d'accès facultatif avec un indice pour s'en souvenir. Les personnes qui utilisaient déjà Bullo ne le voient pas ; il se rouvre à tout moment avec « Refaire la présentation » (Réglages > Profil).
- **Masquer ce qui ne sert pas** : Rappels, Rechercher, Outils, Explique-moi et Bien-être peuvent être cachés du menu (cases dans Réglages > Profil, ou choix rapides « L'essentiel / Études / Tout » à l'accueil). Accueil, Capturer, Inbox et Réglages restent toujours là. Rien n'est supprimé : on recoche et tout revient, données comprises.
- **Indice du code d'accès**, affiché sur l'écran de verrouillage.

### Corrections
- **Onglets des Réglages qui chevauchaient le contenu en faisant défiler** : la barre d'onglets n'est plus collante, plus rien ne passe dessous.
- L'écran de verrouillage passait sous le lecteur « Fond sonore » : il est maintenant au-dessus de tout.

### À savoir
- Il n'y a pas de compte en ligne : Bullo reste 100 % local. Le « compte » est un prénom et un code sur cet ordinateur. Le code est une protection de discrétion, pas un chiffrement des notes. Si on l'oublie, il ne peut pas être retrouvé : d'où l'indice.

## 1.2.1 — 2026-10-04

### Amélioration
- **Réglages plus clairs** : la longue page est maintenant rangée en 6 onglets avec une icône chacun, pour trouver un réglage sans faire défiler : **Apparence** (thème, police, tailles, couleurs, daltonisme), **Confort** (une seule tâche, Pomodoro, raccourcis clavier), **Audio** (fond sonore, favoris, lecture vocale), **Données** (Google Drive, sauvegardes, enregistrements, vérification des outils), **Profil** (prénom, code d'accès, cours) et **À propos** (version, mise à jour). Le dernier onglet ouvert est conservé. Les onglets se parcourent aussi au clavier avec les flèches gauche et droite.
- Aucun réglage n'a été retiré ni modifié : tout est au même endroit qu'avant, simplement rangé.

## 1.2.0 — 2026-10-04

### Nouveautés
- **Explique-moi** (méthode Feynman) : une nouvelle page pour vérifier que tu as vraiment compris ce que tu viens d'étudier. Tu indiques le sujet, tu l'expliques (à l'écrit ou à l'oral, avec la transcription locale de Bullo), et un « élève curieux » local (Ollama) te pose 1 à 3 questions sur ce qui manque, sans jamais donner la réponse. L'échange dure 3 séries au maximum, puis un bilan très court : ce qui est solide, et les sujets à revoir. Tu peux t'arrêter à tout moment.
- **Mes explications** : toutes les séances sont gardées sur ton ordinateur (sujet, explication, questions, réponses, bilan). Tu peux les relire et les supprimer.
- Lecture vocale des questions et du bilan, avec la synthèse vocale de Bullo.
- Rien ne s'ouvre tout seul : « Explique-moi » se lance depuis le menu, depuis un bouton après un résumé dans Capturer, ou depuis la proposition discrète à la fin d'un Pomodoro.

### Corrections
- Les petites notifications (mise à jour, fin de session) ne se superposent plus.

## 1.1.0 — 2026-10-03

### Nouveautés
- **Mises à jour** : Bullo vérifie au démarrage (au plus une fois par jour) si une nouvelle version existe, et te prévient avec un petit bandeau. Tu peux aussi cliquer sur « Rechercher une mise à jour » dans Réglages > À propos. La vérification au démarrage peut être désactivée.
- **Réglages > Vérifier les outils** : contrôle maintenant aussi Ollama et le modèle de résumé, et télécharge en un clic le modèle de transcription (Whisper) et le modèle de résumé.

### Corrections
- **Page Capturer** : elle pouvait ne plus s'ouvrir (le bouton s'allumait mais l'ancienne page restait affichée) quand le résumé de l'IA avait une forme inattendue. Les résumés sont désormais remis en forme, un ancien brouillon abîmé est réparé automatiquement, et si une page plante, un message clair et un bouton « Réparer » s'affichent.
- **Police** : fin du faux message « La police Bullo est introuvable » au démarrage.
- **Transcription et résumé** : les messages d'erreur disent maintenant la vraie cause (modèle absent, Ollama éteint, délai dépassé) au lieu d'un message générique.

## 1.0.0 — 2026-10-03
Première version publique.
- Capture (micro, texte, image/PDF/Word), transcription locale, résumé, devoirs et dates, rappels, lecture vocale, recherche.
- Fond sonore généré par Bullo ou lecture de l'ordinateur (Music, Spotify, navigateurs) avec boucle, aléatoire et favoris ; détection légère.
- Bien-être : parking de pensées, respiration guidée, pauses douces.
- Accessibilité : polices dyslexie, taille, interlignage, thèmes, daltonisme, contraste élevé, mouvement réduit.
- Sauvegarde locale et Google Drive (clés protégées dans le Trousseau macOS).
- Nouveau design « nuit douce » aux couleurs du logo.
