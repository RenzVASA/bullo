# Idées pour la suite

Priorité 1 : valider sur Mac (voir PROTOCOLE_DE_TEST.md), compiler, corriger.

- Mode « focus profond » : plein écran, une seule tâche, fond sonore et minuteur liés.
- Révision : quiz et rappel faits dans la 1.7. Idées : réponse libre corrigée par l'IA, statistiques de progression sans score culpabilisant.
- Dictée d'une tâche en une phrase (« demain 14h maths exercice 3 ») → rappel créé.
- Découpage d'une consigne en micro-étapes avec minuteur par étape.
- Bilan doux de fin de journée (ce qui a été fait, sans score ni culpabilisation).
- Export vers Obsidian / dossier Markdown.
- Versions clair/sombre des icônes et du dock, thème « très calme » sans animation.

## Bullo 2.0 — plusieurs systèmes (plan)

Principe : on continue de coder pour le Mac, et le même code se compile pour Linux puis Windows. Chaque fonction propre à un système est isolée avec `#[cfg(target_os = "…")]` (une version par système, au même endroit) ; ce qui n'existe pas sur un système est masqué dans l'interface plutôt que de planter.

1. **Préparer le terrain (sans rien changer pour les utilisateurs Mac)** : regrouper dans un module « système » tout ce qui est propre au Mac : `osascript` (7 usages), `afplay`, contrôle de Music/Spotify, volume du système, `open`, installations `brew`, `PATH` Homebrew, barre de menus, notifications.
2. **Linux d'abord** (le plus proche du Mac, tous deux de type Unix) : barre système (libayatana-appindicator), notifications (`notify-send`), ouverture (`xdg-open`), son (`paplay`/`aplay`), volume (`wpctl`/`pactl`), lecteurs de musique via MPRIS (`playerctl`), installation des outils via `apt`. Livrables : `.AppImage` et `.deb`. Test : machine virtuelle (UTM) avec Ubuntu.
3. **Automatisation** : GitHub Actions avec une matrice macOS / Linux (puis Windows) qui compile et signe chaque version ; une seule clé de signature pour les mises à jour, un fichier de mise à jour par système.
4. **Windows ensuite** : le plus de différences (chemins, PowerShell, notifications, lecteur multimédia, installation d'outils via `winget`). Livrable : `.msi`. Test sur machine virtuelle Windows.
5. **Tests** : les tests d'interface existants tournent déjà sans le Mac ; on y ajoute une vérification que chaque fonction masquée sur un système n'apparaît pas dans son interface.

Prérequis déjà en place : interface unique en HTML/CSS/JS, base SQLite locale, mises à jour signées.
