# Bullo — protocole de test

Coche une case uniquement après l'avoir testé dans l'application (`npm run tauri dev`), avec la console développeur ouverte (clic droit > Inspecter). Note tout message d'erreur exact.

## 0. Démarrage
- [ ] `npm run build` affiche « Bullo frontend syntax OK »
- [ ] `npm run tauri dev` compile sans erreur Rust ; la console n'affiche aucune erreur
- [ ] Titre de fenêtre « Bullo », logo Bullo dans la barre latérale (pas de « ? »)

## 1. Capturer, transcrire, résumer
- [ ] Enregistrer 10 s de voix → « Terminer » → la transcription apparaît (résultat attendu : texte français lisible)
- [ ] Résumer → résumé, à retenir, devoirs, dates, questions
- [ ] « Écouter le texte » lit le résumé en français ; pause, reprise, arrêt fonctionnent
- [ ] « Réécouter l'original » joue l'enregistrement, sans message d'erreur
- [ ] « Enregistrer comme note » puis « Enregistrer dans un cours » : retrouvés dans l'Inbox
- [ ] Fermer complètement l'app, la relancer : « Écouter » (audio du cours) et « Écouter » (transcription) fonctionnent encore
- [ ] Naviguer vers une autre page, revenir, refaire les deux lectures
- [ ] Déplacer le fichier audio puis cliquer « Écouter » : message « Audio introuvable… » (pas « Error »)

## 2. Rappels et alarmes
- [ ] Dans un résumé contenant « devoir pour vendredi », le bouton « ⏰ Rappel » propose vendredi 08:00
- [ ] Page Rappels : nouveau rappel dans 2 minutes → une notification macOS apparaît à l'heure, **fenêtre Bullo en arrière-plan**
- [ ] Alarme « dans 1 minute » ; « Reporter 10 min » ; « Fait » ; « Rétablir » ; « Modifier » ; « Supprimer » (2 clics)
- [ ] Onglets Aujourd'hui / À venir / Terminés ; les rappels sont toujours là après redémarrage

## 3. Fond sonore (barre fixe en bas)
- [ ] La barre est visible sur Accueil, Capturer, Inbox, Rappels, Rechercher, Outils, Réglages, sans cacher de contenu
- [ ] Lecture/pause, précédent/suivant (6 sons : pluie, blanc, rose, brun, café, nature), volume immédiat
- [ ] La flèche réduit/affiche la barre ; l'état est conservé au redémarrage ; aucun « Choose File »
- [ ] Lancer un son puis enregistrer : le son se met en pause pendant l'enregistrement et reprend après
- [ ] Lancer un son, réécouter un audio original : les deux fonctionnent indépendamment

## 4. Réglages et accessibilité
- [ ] Polices : OpenDyslexic, Lexend, Atkinson, Verdana, Arial, Comic Sans MS : l'état « ✅ chargée » ou « ❌ introuvable » est exact, l'aperçu change
- [ ] Taille, interlignage, espacement, thème, couleur d'accent, daltonien, contraste élevé, thème doux, masque de lecture
- [ ] Redémarrer : tout est conservé et appliqué
- [ ] Raccourcis : modifier, conflit signalé, Ctrl+K toujours « Rechercher »
- [ ] Pomodoro : absent au lancement, activable, déplaçable, décompte continu entre les pages
- [ ] Lecture vocale : vitesse, voix, test, arrêt ; message clair si aucune voix française
- [ ] À propos : histoire de Bullo, mentions légales, version

## 5. Exports et sauvegardes
- [ ] Exporter la transcription .txt (accents, retours à la ligne) : fichier dans Téléchargements
- [ ] Inbox > Exporter : un .md et un .pdf lisibles dans Téléchargements
- [ ] Réglages > Sauvegarder / Restaurer mes notes et cours
- [ ] Exporter / importer les réglages (.json)

## 5 bis. Nouveautés v4
- [ ] Importer une image sans tesseract : bouton « Installer automatiquement et réessayer », puis l'import fonctionne
- [ ] Mini lecteur : choisir Music puis Spotify (autoriser Automation) ; lecture, pause, suivant, volume, titre affichés ; repasser sur Ambiances
- [ ] Lancer un enregistrement avec Spotify en lecture : il se met en pause puis reprend
- [ ] Réglages > Apparence : télécharger Lexend/Atkinson ; elles apparaissent, se vérifient (✅) et restent après redémarrage
- [ ] Lancer une analyse, changer de page : le résultat est là au retour (et une notification s'affiche)
- [ ] Capture : quitter l'appli en plein milieu puis la rouvrir : le brouillon est restauré
- [ ] Google : suivre les 6 étapes ; vérifier que les clés sont dans Trousseau d'accès (« bullo-google-… ») ; se connecter ; Sauvegarder ; vérifier le fichier dans Drive › Bullo
- [ ] Google : Restaurer (la copie de sécurité est créée) ; se déconnecter ; effacer les clés
- [ ] Google : fermer la fenêtre pendant la connexion : elle s'annule proprement

## 5 ter. Lecteur, sourdine, bien-être (v5)
- [ ] Lecteur : seulement 2 choix (Son de Bullo / Son de l'ordinateur) ; le logo s'affiche
- [ ] Son de l'ordinateur + vidéo YouTube dans Safari/Chrome : le curseur règle le volume du Mac
- [ ] Lancer un enregistrement pendant qu'une vidéo YouTube joue : le son du Mac se coupe (icône de volume barrée) ; à l'arrêt il revient
- [ ] Même test avec Spotify : pause pendant l'enregistrement, reprise après
- [ ] Mac déjà en sourdine avant l'enregistrement : il le reste après
- [ ] Bien-être : Ctrl+Maj+P depuis une autre page ouvre le parking ; la pensée apparaît dans l'onglet Bien-être
- [ ] Respiration : phases et compte à rebours ; Arrêter ; changer de page l'arrête
- [ ] Pauses douces toutes les 20 minutes (tester en laissant l'appli ouverte)

## 5 quater. v6
- [ ] Lecteur > Son de l'ordinateur : lancer Spotify/Music puis YouTube dans Chrome ou Safari : le titre s'affiche, lecture/pause/suivant/précédent répondent (autoriser Automatisation et Accessibilité au premier essai)
- [ ] Lecteur : ▶ sans rien détecté envoie lecture/pause au Mac (pas de bouton « Ouvrir »)
- [ ] Notes : ajouter plusieurs notes avec coefficients ; vérifier les moyennes à la main
- [ ] Google : l'empreinte SHA-256 s'affiche avec ✅ après l'enregistrement des clés
- [ ] Réglages : un seul bloc « À propos », avec « Vibe codé par Renz-VASA… » et le logo

## 5 quinquies. v8
- [ ] Opera GX : lancer une vidéo/musique, puis Réglages > Fond sonore > Diagnostic du lecteur : copier le résultat
- [ ] Lecteur en mode ordinateur avec Opera GX : le nom du navigateur s'affiche, ⏯ ⏭ ⏮ répondent ? (sinon « Tester la touche lecture/pause »)
- [ ] Apple Music : 🔁 (off/tout/ce titre), 🔀, ☆ (vérifier dans l'appli Musique)
- [ ] Spotify : 🔁 et 🔀 ; ☆ garde le titre dans Mes favoris du lecteur ; « Rechercher » ouvre Spotify
- [ ] Son de Bullo : ☆ sur 2 ambiances, option « favoris seulement », 🔀 aléatoire

## 6. Parcours global
- [ ] Tous les boutons de navigation et sous-boutons répondent ; aucune erreur console sur l'ensemble du parcours
- [ ] Un parcours complet au clavier uniquement (Tab, Entrée, Échap)


## 5 septies — Design
- [ ] Clair, sombre, contraste élevé, thème doux : tout reste lisible, le focus clavier est visible
- [ ] Lecteur : le dock ne cache aucun contenu (faire défiler jusqu'en bas), les barres bougent seulement pendant la lecture
- [ ] Fenêtre étroite : le menu se réduit aux icônes
## 5 octies — Explique-moi (v1.2)
Avec **Ollama lancé** et le modèle de résumé téléchargé :
- [ ] Le menu contient « Explique-moi » ; en ouvrant l'appli, rien ne s'ouvre tout seul.
- [ ] Sujet vide ou explication très courte : un message te demande de compléter, sans appel au modèle.
- [ ] Sujet + explication écrite, « Commencer » : un indicateur de chargement apparaît, puis 1 à 3 questions courtes.
- [ ] Les questions ne donnent pas la réponse (si le modèle en donne une, note-le).
- [ ] Changer de page pendant que le modèle réfléchit : l'appli reste fluide, un message prévient quand la question est prête.
- [ ] Répondre : 3 séries au maximum, puis le bilan (Solide / À revoir, sous forme de sujets).
- [ ] « Voir mon bilan » avant la fin et « Arrêter » à tout moment fonctionnent.
- [ ] « Écouter les questions » et « Écouter le bilan » lisent le texte à voix haute.
- [ ] Dictée à l'oral : « Dicter à l'oral », parler 10 secondes, terminer : le texte arrive dans la zone, modifiable (micro autorisé).
- [ ] « Mes explications » : la séance est là (sujet, date), on peut l'ouvrir, la relire, la supprimer (avec confirmation).
- [ ] Après avoir fermé et rouvert Bullo, l'historique est toujours là.
- [ ] Polices (OpenDyslexic, Lexend…), tailles et thème sombre : la page les respecte.
- [ ] Fin d'un Pomodoro de travail (si activé) : un bandeau propose « Explique-moi », sans l'ouvrir. Après un résumé dans Capturer : un bouton « Explique-moi » est proposé.

Avec **Ollama fermé** :
- [ ] « Commencer » affiche « Cette fonctionnalité a besoin d'Ollama. Ouvre l'application Ollama, puis réessaie. », rien ne plante, ton texte est conservé.
- [ ] Après avoir ouvert Ollama, « Commencer » fonctionne sans retaper.

## 5 quindecies — Notifications (v1.6.1)
- [ ] Application installée (pas `tauri dev`) : créer un rappel dans 1 minute. La notification porte le nom et l'icône de Bullo ; un clic ne lance plus l'Éditeur de script.
- [ ] Réglages Système › Notifications : « Bullo » apparaît dans la liste.
- [ ] Fenêtre cachée (barre de menus), une réponse d'Explique-moi arrive : même notification.

## 5 quaterdecies — Barre de menus et Réviser (v1.6.0)
- [ ] Une icône Bullo apparaît en haut à droite du Mac (claire sur barre sombre, sombre sur barre claire). Le menu propose les 5 actions.
- [ ] « Note rapide… » : petite fenêtre au premier plan ; écrire, Entrée → la note est dans l'Inbox. Échap ferme sans rien enregistrer. Idem « Poser une pensée… » (titre commençant par 💭).
- [ ] « Enregistrer » : l'enregistrement démarre même fenêtre Bullo fermée (à noter : le Mac peut demander l'autorisation du micro), l'icône affiche « ● REC », le menu propose « Arrêter l'enregistrement ».
- [ ] Fermer la fenêtre (rouge) : Bullo reste dans la barre de menus ; clic sur le Dock ou « Ouvrir Bullo » la rouvre. « Quitter Bullo » ferme tout.
- [ ] Réglages › Confort : décocher « Garder Bullo dans la barre de menus » → fermer la fenêtre quitte l'application.
- [ ] Ollama ouvert, sur un vrai cours (résumé fait) : « Créer des cartes » → des cartes justes, modifiables ; garder ; Réviser › À revoir › Commencer. Réponse ratée → la carte revient dans la séance.
- [ ] Ollama fermé : message clair, texte conservé.
- [ ] Sauvegarder, supprimer des cartes, restaurer : elles reviennent.

## 5 terdecies — Mode épuré, sauvegarde complète, licences (v1.5.0)
- [ ] Menu › « Mode épuré » : le menu passe en icônes, les textes d'aide et le lecteur de sons disparaissent ; un second clic remet tout. Les notes sont intactes.
- [ ] Faire une séance Explique-moi, puis Réglages › Données › Sauvegarder (fichier). Effacer la séance, restaurer : la séance est de retour, avec la copie de sécurité annoncée.
- [ ] Restaurer une ancienne sauvegarde (avant 1.5.0) : le message précise que les rappels et séances existants sont restés tels quels.
- [ ] Réglages › À propos : la mention sur les polices est présente.
- [ ] Mise à jour 1.4.0 → 1.5.0 en un clic (première mise à jour signée) : Bullo redémarre avec le numéro 1.5.0.

## 5 duodecies — Mise à jour en un clic (v1.3.3)
- [ ] Bullo installé dans Applications en 1.3.3, puis une version plus récente publiée : le bandeau affiche « Installer et redémarrer ».
- [ ] Clic : message de téléchargement, Bullo se ferme, puis se rouvre seul avec le nouveau numéro dans Réglages › À propos. Notes et réglages intacts.
- [ ] Sans internet : message d'erreur clair, bouton « Ouvrir la page de téléchargement », l'application reste ouverte.
- [ ] Lancé avec `npm run tauri dev` : message « ne marche que dans l'application installée » (normal).

## 5 undecies — Longs textes et notes (v1.3.2)
- [ ] Inbox : écrire une très longue note (plus de 300 caractères) : « Lire la suite » affiche tout, « Réduire » replie, « 🔊 Écouter » lit jusqu'à la dernière phrase.
- [ ] Outils : coller un long texte, « Enregistrer comme note », puis vérifier dans l'Inbox qu'on peut tout lire et tout écouter.
- [ ] Résumé : transcrire ou coller un cours de plus de 10 minutes de parole : le résumé et « À retenir » parlent aussi de la fin du cours (Ollama doit être ouvert ; plus c'est long, plus c'est lent).

## 5 decies — Bienvenue et modules (v1.3.0)
- [ ] Premier lancement (effacer les données de Bullo ou nouvel utilisateur) : l'écran de bienvenue s'affiche, 3 étapes.
- [ ] Étape 2 : « L'essentiel », « Études », « Tout » cochent les bonnes cases ; décocher Bien-être le retire du menu à la fin.
- [ ] Étape 3 : un code de moins de 4 chiffres ou deux codes différents sont refusés ; sans code, « Terminer sans code » fonctionne.
- [ ] Relancer Bullo avec un code : l'écran de verrouillage affiche l'indice, le bon code ouvre.
- [ ] Réglages > Profil : cocher/décocher un module l'ajoute/le retire du menu immédiatement ; « Refaire la présentation » rouvre l'écran.
- [ ] Réglages : faire défiler, plus aucun onglet ne chevauche le contenu (fenêtre large et étroite).
- [ ] Mise à jour depuis une 1.2.x : pas d'écran de bienvenue, menu complet comme avant.

## 5 nonies — Réglages par onglets (v1.2.1)
- [ ] Réglages affiche 6 onglets avec icône : Apparence, Confort, Audio, Données, Profil, À propos.
- [ ] Un clic sur un onglet n'affiche que ses sections ; les flèches gauche/droite changent d'onglet.
- [ ] Apparence : thème, police, tailles, daltonisme fonctionnent comme avant.
- [ ] Confort : Pomodoro, mode une seule tâche, raccourcis clavier.
- [ ] Audio : fond sonore, favoris, diagnostic du lecteur, vitesse de lecture vocale.
- [ ] Données : Google Drive, sauvegardes, dossier des enregistrements, « Vérifier les outils ».
- [ ] Profil : prénom, code d'accès, liste des cours. À propos : version 1.2.1, « Rechercher une mise à jour ».
- [ ] Quitter Réglages puis y revenir : on retrouve le même onglet.
