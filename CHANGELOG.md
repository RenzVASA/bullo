# Historique des versions

Format : une section par version, la plus récente en haut.

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
