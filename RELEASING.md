# Publier une version de Bullo (procédure complète)

Tout se fait depuis le **Terminal**, dans le dossier du projet. La publication est automatisée : une seule commande, `npm run release`, fait les vérifications, fabrique l'application, contrôle qu'aucune donnée personnelle n'est dedans et publie la Release sur GitHub.

> Prérequis (une seule fois) : `brew install git gh node`, Rust installé, `gh auth login` fait, et `npm install` lancé dans le dossier. Tout est expliqué dans [GUIDE_GITHUB.md](GUIDE_GITHUB.md).

## Publier une nouvelle version (6 étapes)

| # | Ce que tu fais | Commande |
|---|---|---|
| 1 | Choisis le numéro de version | `npm run set-version -- 1.1.0` |
| 2 | Écris ce qui change, en haut de `CHANGELOG.md` (une section `## 1.1.0 — date`) | ouvre le fichier |
| 3 | Sauvegarde et envoie sur GitHub | `git add .` puis `git commit -m "Version 1.1.0"` puis `git push` |
| 4 | Lance la publication | `npm run release` |
| 5 | Attends 3 à 5 minutes (fabrication) | rien à faire |
| 6 | Vérifie la page des Releases | https://github.com/RenzVASA/bullo/releases |

Pour **voir ce que ferait le script sans rien publier** : `npm run release -- --dry`.
Pour **corriger seulement le texte** de la Release (sans refabriquer) : `npm run release -- --notes`.

### Ce que vérifie le script avant de publier
1. Tes changements sont sauvegardés (`git commit`) et envoyés (`git push`). Sinon il s'arrête et te dit quoi faire.
2. Il existe une section pour cette version dans `CHANGELOG.md` : elle devient le texte de la Release.
3. L'application est fabriquée **sans chemin de ton ordinateur** (option `--remap-path-prefix`).
4. Le binaire final ne contient ni ton nom d'utilisateur Mac ni ton dossier personnel. Sinon, **rien n'est publié**.
5. L'empreinte SHA-256 du `.dmg` est ajoutée à la Release (elle permet de vérifier que le fichier n'a pas été modifié).

## Cas particulier : refaire la 1.0.0 déjà publiée (une seule fois)

La Release `v1.0.0` actuelle a été créée avant la correction de l'identité et des chemins. Pour la remplacer proprement :
```bash
cd /Users/klausvanoosthuyse/Downloads/bullo
git add .
git commit -m "Procédure de publication automatisée"
git push
gh release delete v1.0.0 --cleanup-tag -y      # supprime l'ancienne Release ET son étiquette
npm run release                                 # recrée la 1.0.0 proprement
```
La suppression est nécessaire : sinon l'étiquette `v1.0.0` resterait attachée à l'ancien commit.

## Publier sans le script (à la main, avec la souris)

1. Fabrique l'app : `RUSTFLAGS="--remap-path-prefix=$HOME=/home/user" npm run tauri build`
2. Vérifie : `strings src-tauri/target/release/bundle/macos/Bullo.app/Contents/MacOS/bullo | grep -i "$USER"` → ne doit rien afficher.
3. Sur GitHub : **Releases > Draft a new release**.
4. **Choose a tag** : écris `v1.1.0`, puis « Create new tag on publish ». **Target** : `main`.
5. **Title** : `Bullo 1.1.0`. **Description** : copie la section du CHANGELOG et la phrase d'installation.
6. Glisse `src-tauri/target/release/bundle/dmg/Bullo_1.1.0_aarch64.dmg` dans la zone « Attach binaries ».
7. Laisse « Set as the latest release » coché, puis **Publish release**.

## Supprimer ou corriger

| Je veux… | Avec la souris | Avec le Terminal |
|---|---|---|
| Supprimer une Release | Releases > ouvre-la > « Delete » (poubelle) | `gh release delete v1.0.0 --cleanup-tag -y` |
| Remplacer le .dmg d'une Release (même version) | Releases > crayon « Edit » > retire l'ancien fichier, ajoute le nouveau > Update | `gh release upload v1.0.0 fichier.dmg --clobber` |
| Corriger le texte d'une Release | Releases > « Edit » > Update | `gh release edit v1.0.0 --notes "Nouveau texte"` |
| Corriger l'auteur du dernier commit | non | `git commit --amend --reset-author --no-edit` puis `git push --force` |
| Voir qui est l'auteur des commits | page des Commits | `git log --format='%h %an <%ae>'` |

Si tu supprimes une Release à la main, supprime aussi son étiquette : **Code > Tags** (à droite des branches) > poubelle à côté de `v1.0.0`.

## Problèmes fréquents

| Message | Cause | Solution |
|---|---|---|
| `tauri: command not found` | Les outils ne sont pas installés dans ce dossier | `npm install` |
| `Des changements ne sont pas sauvegardés` | Fichiers modifiés non commités | `git add . && git commit -m "…" && git push` |
| `Ta version locale et celle de GitHub sont différentes` | Un commit n'est pas encore envoyé | `git push` |
| `Aucune section « ## 1.1.0 » dans CHANGELOG.md` | Tu as oublié l'étape 2 | Ajoute la section |
| `Ton nom d'utilisateur est encore dans l'application` | Chemin personnel resté dans le binaire | Copie-moi les 3 lignes affichées |
| `gh: not logged in` | Session GitHub expirée | `gh auth login` |
| macOS : « Bullo est endommagé » (chez toi ou un visiteur) | App non signée téléchargée par un navigateur : macOS pose une étiquette de quarantaine. Le fichier est intact. | Annuler (pas « Corbeille »), puis `xattr -cr /Applications/Bullo.app` |
| macOS : « développeur non identifié » | Idem sur un macOS plus ancien | Clic droit sur Bullo > Ouvrir > Ouvrir, sinon la commande ci-dessus |

## Pour aller plus loin (plus tard)
- **Signature et notarisation Apple** (compte développeur payant, 99 $/an) : supprime le message « endommagé » et l'étape `xattr` pour tes visiteurs. À prévoir si Bullo a des utilisateurs réguliers.
- **Publication automatique** par GitHub (`.github/workflows/release.yml`) : se déclenche quand tu envoies une étiquette (`git tag v1.1.0 && git push --tags`). Non testée : garde `npm run release` comme méthode principale.
