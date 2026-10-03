# Guide complet : mettre Bullo sur GitHub (de zéro)

Durée : environ 30 minutes la première fois. Tu n'as besoin d'aucune connaissance préalable.

## 0. Ce que tu vas obtenir
- Une page publique avec la présentation de Bullo, les captures d'écran et les instructions.
- Chaque version sauvegardée (si tu casses quelque chose, tu peux revenir en arrière).
- Une vérification automatique à chaque envoi, **qui compile le code Rust sur un Mac de GitHub** : tu verras les erreurs de compilation sans rien installer.
- Un bouton « Télécharger Bullo (.dmg) » pour les autres.

## 1. Créer ton compte GitHub
1. Va sur https://github.com et clique sur « Sign up ».
2. Choisis un nom d'utilisateur (il apparaîtra dans l'adresse de ton projet). Active la double authentification (Settings > Password and authentication).

## 2. Installer les outils sur ton Mac
Ouvre l'app **Terminal**, puis :
```bash
xcode-select --install          # si ce n'est pas déjà fait
brew install git gh             # git = sauvegarde des versions, gh = outil GitHub
git config --global user.name  "Ton nom ou pseudo"
git config --global user.email "ton-adresse@exemple.com"
gh auth login                   # choisis GitHub.com > HTTPS > « Login with a web browser »
```
Astuce confidentialité : dans GitHub > Settings > Emails, coche « Keep my email address private » et utilise l'adresse `…@users.noreply.github.com` dans la commande `user.email`.

## 3. Vérifier qu'aucun secret ne part sur Internet
Avant le premier envoi, dans le dossier du projet :
```bash
cd chemin/vers/bullo
cat .gitignore                  # doit contenir node_modules, target, *.db, *.env, client_secret*.json
grep -rniE "client_secret|GOCSPX|api[_-]?key|password" src src-tauri/src | head
```
- Les clés Google que tu saisis dans l'application sont dans le **Trousseau macOS**, pas dans le projet : rien à publier.
- Si la commande `grep` affiche une vraie clé (une longue suite de lettres et chiffres), **supprime-la du code avant de continuer**.
- Ne mets jamais ta base de données (`*.db`), tes enregistrements (`*.wav`) ni un fichier `client_secret_….json` dans le dépôt (le `.gitignore` les bloque).

## 4. Vérifier le nom, la licence et la version
- **Licence** : le fichier `LICENSE` est en MIT (tout le monde peut utiliser et modifier, sans garantie). Si tu préfères que personne ne puisse réutiliser ton code, supprime-le : sans licence, le code reste « tous droits réservés ». C'est ton choix.
- **Nom dans la licence** : « Renz-VASA » vient de ta dictée. Corrige-le dans `LICENSE` et dans `ABOUT_CREDIT` (`src/index.html`) si besoin.
- **Version** : `npm run set-version -- 1.0.0` change la version partout (package.json, tauri.conf.json, Cargo.toml). C'est elle qui s'affiche dans **Réglages > À propos**.
- **Polices** : OpenDyslexic est sous licence libre (OFL). Lexend et Atkinson ne sont pas dans le dépôt (téléchargées à la demande). **Vérifie la licence des deux polices « Bullo » dans `src/fonts/`** (display.woff2 et body-*.woff2) avant de publier : si tu n'en es pas sûr, retire-les et mets une police libre (par exemple Lexend, licence OFL).
- **Dyslexie** est payante : elle ne doit jamais être mise dans le dépôt (elle n'y est pas).

## 5. Créer le dépôt et envoyer le code
Dans le dossier du projet :
```bash
git init -b main
git add .
git status                      # relis la liste : aucun .db, .wav, .env, node_modules
git commit -m "Première version de Bullo"
gh repo create bullo --public --source=. --remote=origin --push \
  --description "Espace de concentration neuro-inclusif : capture, résumé, rappels, fond sonore. Local, pour Mac."
```
(Sans `gh`, via le site : « New repository » > nom `bullo` > ne coche rien > Create, puis copie les 3 commandes `git remote add origin …`, `git branch -M main`, `git push -u origin main`.)

Ton projet est en ligne : `https://github.com/TON-PSEUDO/bullo`.

## 6. Soigner la page d'accueil
1. Sur la page du dépôt, roue dentée à côté de « About » : ajoute une description et des **Topics** : `tauri`, `rust`, `accessibility`, `dyslexia`, `adhd`, `neurodiversity`, `study`, `macos`, `offline-first`.
2. Le `README.md` s'affiche tout seul. Les captures sont dans `docs/` (ce sont des rendus du simulateur ; remplace-les par de vraies captures de ton Mac quand tu peux : Cmd+Maj+4).
3. Settings > Features : active **Issues** (bugs et idées) et, si tu veux, **Discussions**.

## 7. Utiliser Git au quotidien (3 commandes)
```bash
git add .
git commit -m "Ce que j'ai changé, en une phrase"
git push
```
- Pour tester une idée sans risque : `git switch -c ma-idee`. Si ça te plaît : `git switch main && git merge ma-idee`. Sinon : `git switch main` et la branche est abandonnée.
- Pour voir l'historique : `git log --oneline`. Pour annuler un fichier non sauvegardé : `git restore chemin/du/fichier`.

## 8. Vérification automatique (compile le Rust pour toi)
Le fichier `.github/workflows/ci.yml` se lance à chaque `git push`. Dans l'onglet **Actions** du dépôt :
- coche verte : l'interface est valide **et** le Rust compile ;
- croix rouge : clique dessus, lis l'erreur de `cargo check`, corrige, repousse. (Le Rust n'a jamais été compilé avant : la première fois, il est probable qu'il y ait quelques erreurs à corriger. C'est normal.)

## 9. Publier une version téléchargeable (.dmg)
```bash
npm run set-version -- 1.0.0     # si ce n'est pas déjà fait
# ajoute une section dans CHANGELOG.md, puis :
git add . && git commit -m "Version 1.0.0"
git tag v1.0.0
git push && git push --tags
```
Le fichier `.github/workflows/release.yml` fabrique l'application sur un Mac de GitHub (10 à 20 minutes) et crée un **brouillon de Release** (onglet « Releases »). Ouvre-le, vérifie le fichier `.dmg`, relis le texte, puis clique sur « Publish release ».

**Important, application non signée :** sans compte développeur Apple (99 $/an), macOS affiche « Bullo ne peut pas être ouvert ». Le mode d'emploi pour les utilisateurs : *clic droit sur Bullo > Ouvrir > Ouvrir*, ou Réglages Système > Confidentialité et sécurité > « Ouvrir quand même ». Écris-le dans la description de la Release.

Si la fabrication échoue, l'erreur est dans l'onglet Actions. Cause fréquente : une erreur de compilation Rust (voir 8).

## 10. Après la publication
- Ajoute un lien « Télécharger » dans le README vers `Releases > Latest`.
- Réponds aux Issues avec bienveillance ; le fichier `CONTRIBUTING.md` explique les règles aux visiteurs.
- Garde `ROADMAP.md` à jour : c'est la page « ce qui vient ensuite ».
- Pour changer de version plus tard : `npm run set-version -- 1.1.0`, ligne dans `CHANGELOG.md`, commit, `git tag v1.1.0`, `git push --tags`.


## 11. Rendre la page « pro » (5 minutes)
1. **Image de partage** : Settings > General > « Social preview » > Upload `docs/social-preview.png` (c'est l'image qui s'affiche quand tu partages le lien).
2. **About** (roue dentée à droite de la page) : description, site vide, coche « Releases » ; ajoute les Topics.
3. **README** : il est déjà organisé pour les visiteurs (bannière, bouton Télécharger, installation en 3 étapes). Le code est replié dans « Pour les développeurs ».
4. **Release avec le .dmg** (c'est ce qui rend le projet « un clic ») : voir l'étape 9. Chaque Release contient aussi, automatiquement, « Source code (zip) » : c'est le zip du code pour les curieux.
5. **Plan B si l'automatisation échoue** : sur ton Mac, `npm run tauri build`, puis le fichier est dans `src-tauri/target/release/bundle/dmg/Bullo_1.0.0_aarch64.dmg`. Sur GitHub : Releases > « Draft a new release » > Tag `v1.0.0` > glisse le `.dmg` dans « Attach binaries » > Publish.

## Check-list finale avant de passer le dépôt en public
- [ ] Aucune clé ni mot de passe dans le code (étape 3)
- [ ] Licence choisie et nom correct (étape 4)
- [ ] Licence des polices vérifiée (étape 4)
- [ ] `node scripts/check-frontend.js` affiche « syntax OK »
- [ ] Coche verte dans Actions
- [ ] README relu, captures présentes
- [ ] Un test réel sur ton Mac avec PROTOCOLE_DE_TEST.md
