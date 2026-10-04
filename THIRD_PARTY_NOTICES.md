# Éléments tiers

Bullo est publié sous licence [MIT](LICENSE). Il embarque ou utilise les éléments suivants, qui ont chacun leur propre licence.

## Polices incluses dans l'application (SIL Open Font License 1.1)

| Police | Où | Auteurs | Texte de la licence |
|---|---|---|---|
| Bricolage Grotesque | titres (`src/fonts/display.woff2`) | The Bricolage Grotesque Project Authors | `src/fonts/licences/OFL-BricolageGrotesque.txt` |
| Atkinson Hyperlegible | texte courant (`src/fonts/body-400.woff2`, `body-700.woff2`) | Braille Institute of America, Inc. | `src/fonts/licences/OFL-AtkinsonHyperlegible.txt` |
| OpenDyslexic | police « OpenDyslexic » (`src/fonts/dys-*.woff2`) | Abbie Gonzalez | `src/fonts/licences/OFL-OpenDyslexic.txt` |

La licence OFL autorise l'utilisation, la copie et la redistribution (y compris dans une application) à condition de conserver la mention de copyright et le texte de la licence, et de ne pas vendre les polices seules. Les fichiers des polices sont fournis au format WOFF2, sans modification de leur dessin.

## Polices téléchargées à la demande de l'utilisateur

**Lexend** et **Atkinson Hyperlegible** (versions « Lexend » et « Atkinson » des Réglages) ne sont pas incluses : Bullo les télécharge depuis un CDN public quand l'utilisateur clique sur « Télécharger ». Elles sont aussi sous licence SIL OFL 1.1. **Dyslexie** est une police commerciale : Bullo ne la fournit pas, elle n'apparaît que si l'utilisateur l'a installée lui-même.

## Outils utilisés mais non redistribués

Bullo ne contient pas et ne redistribue pas : FFmpeg, whisper.cpp et son modèle, Poppler, Tesseract, Ollama et le modèle de langage. Ils sont installés séparément par l'utilisateur (Homebrew, ollama.com) et gardent leurs licences respectives.

## Dépendances du code

Les bibliothèques Rust (`src-tauri/Cargo.toml`) et JavaScript (`package.json`) sont sous leurs licences propres, listées dans leurs paquets (Tauri : MIT/Apache-2.0).
