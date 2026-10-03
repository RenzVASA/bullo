// Usage : npm run set-version -- 1.2.0
// Met à jour d'un coup package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml et la constante de repli de l'interface.
// La version affichée dans Réglages > À propos vient de tauri.conf.json (lue par l'application).
const fs = require("fs");
const v = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(v || "")) { console.error("Donne une version du type 1.2.0 :  npm run set-version -- 1.2.0"); process.exit(1); }
const edit = (file, fn) => { const s = fs.readFileSync(file, "utf8"), t = fn(s); if (s === t) console.warn("Inchangé : " + file); else { fs.writeFileSync(file, t); console.log("OK  " + file); } };
edit("package.json", s => s.replace(/("version":\s*")[^"]+(")/, `$1${v}$2`));
edit("src-tauri/tauri.conf.json", s => s.replace(/("version":\s*")[^"]+(")/, `$1${v}$2`));
edit("src-tauri/Cargo.toml", s => s.replace(/^(version\s*=\s*")[^"]+(")/m, `$1${v}$2`));
edit("src/index.html", s => s.replace(/(const APP_VERSION = ")[^"]+(")/, `$1${v}$2`));
console.log(`Version ${v}. Pense à ajouter une ligne dans CHANGELOG.md puis : git tag v${v}`);
