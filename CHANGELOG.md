# Historique des versions

Format : une section par version, la plus récente en haut.

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
