#!/usr/bin/env bash
# Publie une version de Bullo sur GitHub en une commande :   npm run release
# Ce que fait le script, dans l'ordre :
#   1. vérifie que tout est sauvegardé et envoyé sur GitHub
#   2. fabrique l'application (.dmg) SANS aucun chemin de ton ordinateur
#   3. vérifie qu'aucune trace de ton nom d'utilisateur Mac n'est dans l'application
#   4. crée (ou remplace) la Release GitHub avec le .dmg, le texte du CHANGELOG et l'empreinte SHA-256,
#      plus l'archive de mise à jour signée et latest.json (mise à jour automatique signée)
# Options :  npm run release -- --dry     (montre ce qui serait fait, sans rien fabriquer ni publier)
#            npm run release -- --notes   (met à jour seulement le texte de la Release, sans refabriquer)
set -euo pipefail
cd "$(dirname "$0")/.."

DRY=0; NOTES_ONLY=0
[ "${1:-}" = "--dry" ] && DRY=1
[ "${1:-}" = "--notes" ] && NOTES_ONLY=1
say()  { printf '\n\033[1m%s\033[0m\n' "$*"; }
fail() { printf '\n\033[31mErreur : %s\033[0m\n' "$*" >&2; exit 1; }
run()  { if [ "$DRY" = 1 ]; then echo "[simulation] $*"; else "$@"; fi; }

ME="${USER:-$(id -un)}"
VERSION=$(node -p "require('./package.json').version")
TAG="v$VERSION"
DMG_DIR="src-tauri/target/release/bundle/dmg"
APP_BIN="src-tauri/target/release/bundle/macos/Bullo.app/Contents/MacOS/bullo"
UPD_TGZ="src-tauri/target/release/bundle/macos/Bullo.app.tar.gz"
KEY="$HOME/.tauri/bullo.key"
KEY_PW="$HOME/.tauri/bullo.password"
say "Bullo $VERSION ($TAG)"

# --- 1. Tout est-il sauvegardé et envoyé ? ---
command -v gh >/dev/null || fail "L'outil gh est introuvable. Installe-le avec : brew install gh"
gh auth status >/dev/null 2>&1 || fail "Tu n'es pas connecté à GitHub. Lance : gh auth login"
if [ -n "$(git status --porcelain)" ]; then
  git status --short
  fail "Des changements ne sont pas sauvegardés. Lance : git add . && git commit -m \"Ce que j'ai changé\" && git push"
fi
git fetch -q origin main
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] || fail "Ta version locale et celle de GitHub sont différentes. Lance : git push"
echo "OK : tout est sauvegardé et envoyé."

# --- 2. Texte de la Release : section du CHANGELOG pour cette version ---
NOTES=$(mktemp)
awk -v v="$VERSION" '$0 ~ "^## " v {f=1; next} f && /^## /{exit} f' CHANGELOG.md > "$NOTES"
[ -s "$NOTES" ] || fail "Aucune section « ## $VERSION » dans CHANGELOG.md. Ajoute-en une, puis relance."
cat >> "$NOTES" <<'TXT'

---
**Installer** : ouvre le `.dmg` et glisse **Bullo** dans **Applications**. Puis, **une seule fois**, ouvre le Terminal et lance :

```
xattr -cr /Applications/Bullo.app
```

Ensuite ouvre Bullo normalement. (L'application n'est pas encore signée par Apple : sans cette ligne, macOS affiche « Bullo est endommagé » alors que le fichier est intact. La commande retire seulement l'étiquette « téléchargé depuis Internet ».)

Mac Apple Silicon (M1 et suivants) uniquement.
TXT
echo "Texte de la Release :"; sed 's/^/   | /' "$NOTES"

if [ "$NOTES_ONLY" = 1 ]; then
  say "Mise à jour du texte de la Release $TAG (sans refabriquer)"
  OLD_SHA=$(gh release view "$TAG" --json body --jq .body 2>/dev/null | grep -o 'SHA-256 du .dmg : `[0-9a-f]*`' || true)
  [ -n "$OLD_SHA" ] && printf '\nEmpreinte %s\n' "$OLD_SHA" >> "$NOTES"
  gh release edit "$TAG" --title "Bullo $VERSION" --notes-file "$NOTES"
  rm -f "$NOTES"; echo "OK : texte mis à jour."; exit 0
fi

# --- 3. Fabrication, sans chemins personnels, et signature de la mise à jour ---
[ -f "$KEY" ] && [ -f "$KEY_PW" ] || fail "Clé de signature introuvable ($KEY). Elle est créée par le script bullo-1.4.sh. Si tu as changé de Mac, récupère-la dans iCloud Drive › Cles-de-signature."
export TAURI_SIGNING_PRIVATE_KEY="$(cat "$KEY")"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$(cat "$KEY_PW")"
say "Fabrication de l'application (3 à 5 minutes)"
[ -d node_modules ] || run npm install
export RUSTFLAGS="--remap-path-prefix=$HOME=/home/user ${RUSTFLAGS:-}"
run npm run tauri build

# --- 4. Contrôle de confidentialité ---
say "Contrôle : aucune trace de ton nom d'utilisateur dans l'application"
if [ "$DRY" = 1 ]; then
  echo "[simulation] strings $APP_BIN | grep -i -e \"$ME\" -e \"$HOME\""
else
  [ -f "$APP_BIN" ] || fail "Application introuvable : $APP_BIN"
  if strings "$APP_BIN" | grep -i -q -e "$ME" -e "$HOME"; then
    strings "$APP_BIN" | grep -i -e "$ME" -e "$HOME" | head -3
    fail "Ton nom d'utilisateur est encore dans l'application. Publication annulée (rien n'a été envoyé)."
  fi
  echo "OK : l'application ne contient aucun chemin personnel."
fi

# --- 5. Publication ---
DMG=$(ls "$DMG_DIR"/*.dmg 2>/dev/null | head -1 || true)
if [ "$DRY" = 1 ]; then DMG="$DMG_DIR/Bullo_${VERSION}_aarch64.dmg"; else [ -n "$DMG" ] || fail "Aucun .dmg dans $DMG_DIR"; fi
SHA=$( [ "$DRY" = 1 ] && echo "(calculée à la publication)" || shasum -a 256 "$DMG" | cut -d' ' -f1 )
printf '\nEmpreinte SHA-256 du .dmg : `%s`\n' "$SHA" >> "$NOTES"

# Mise à jour signée : l'archive de l'application, sa signature et latest.json.
UPD_NAME="Bullo_${VERSION}_aarch64.app.tar.gz"
OUT=$(mktemp -d)
if [ "$DRY" = 1 ]; then
  echo "[simulation] latest.json + $UPD_NAME"
else
  [ -f "$UPD_TGZ" ] && [ -f "$UPD_TGZ.sig" ] || fail "Archive de mise à jour signée introuvable ($UPD_TGZ). La signature a-t-elle échoué ?"
  cp "$UPD_TGZ" "$OUT/$UPD_NAME"
  REPO_SLUG=$(gh repo view --json nameWithOwner --jq .nameWithOwner)
  awk -v v="$VERSION" '$0 ~ "^## " v {f=1; next} f && /^## /{exit} f' CHANGELOG.md | sed '/^[[:space:]]*$/d' > "$OUT/notes.txt"
  SIG="$(cat "$UPD_TGZ.sig")" URL="https://github.com/$REPO_SLUG/releases/download/$TAG/$UPD_NAME" V="$VERSION" OUTDIR="$OUT" node -e '
    const fs = require("fs"), o = process.env.OUTDIR;
    fs.writeFileSync(o + "/latest.json", JSON.stringify({
      version: process.env.V,
      notes: fs.readFileSync(o + "/notes.txt", "utf8").trim(),
      pub_date: new Date().toISOString(),
      platforms: { "darwin-aarch64": { signature: process.env.SIG, url: process.env.URL } }
    }, null, 2));'
  echo "OK : mise à jour signée prête ($UPD_NAME)."
fi

say "Publication sur GitHub"
if gh release view "$TAG" >/dev/null 2>&1; then
  echo "La Release $TAG existe déjà : les fichiers et le texte sont remplacés."
  run gh release upload "$TAG" "$DMG" "$OUT/$UPD_NAME" "$OUT/latest.json" --clobber
  run gh release edit "$TAG" --title "Bullo $VERSION" --notes-file "$NOTES" --latest
else
  run gh release create "$TAG" "$DMG" "$OUT/$UPD_NAME" "$OUT/latest.json" --target main --title "Bullo $VERSION" --notes-file "$NOTES" --latest
fi
rm -rf "$OUT"
rm -f "$NOTES"
say "Terminé. Vérifie la page : $(gh repo view --json url --jq .url 2>/dev/null || echo 'ton dépôt')/releases"
