// Génère toutes les variantes du logo Bullo dans src/assets, plus les icônes Tauri.
// Source : src/assets/logo-bullo.jpeg (ou .jpg / .png) si présent, sinon src/bullo-logo.svg. Aucun chemin absolu.
// Usage : npm run logo        (puis, sur Mac : npm run tauri icon src/assets/logo-bullo.png  pour l'icône .icns)
const fs = require("fs"), path = require("path"), sharp = require("sharp");
const root = path.join(__dirname, ".."), assets = path.join(root, "src", "assets"), icons = path.join(root, "src-tauri", "icons");
const candidates = ["logo-bullo.jpeg", "logo-bullo.jpg"].map(f => path.join(assets, f)).concat(path.join(root, "src", "bullo-logo.svg"));
const source = candidates.find(f => fs.existsSync(f));
if (!source) { console.error("Aucune source de logo trouvée. Dépose logo-bullo.jpeg dans src/assets/."); process.exit(1); }
const isSvg = source.endsWith(".svg");
console.log("Source du logo :", path.relative(root, source), isSvg ? "(logo vectoriel provisoire : dépose logo-bullo.jpeg dans src/assets/ pour utiliser le tien)" : "");
const png = (n) => (isSvg ? sharp(source, { density: Math.ceil(144 * n / 256) }) : sharp(source)).resize(n, n, { fit: "cover" }).png().toBuffer();
// ICO avec images PNG intégrées (format accepté par Windows, macOS et les navigateurs)
function ico(entries) {
  const head = Buffer.alloc(6); head.writeUInt16LE(1, 2); head.writeUInt16LE(entries.length, 4);
  let offset = 6 + 16 * entries.length; const dirs = [], datas = [];
  for (const { size, buf } of entries) { const d = Buffer.alloc(16); d[0] = size >= 256 ? 0 : size; d[1] = size >= 256 ? 0 : size; d.writeUInt16LE(1, 4); d.writeUInt16LE(32, 6); d.writeUInt32LE(buf.length, 8); d.writeUInt32LE(offset, 12); offset += buf.length; dirs.push(d); datas.push(buf); }
  return Buffer.concat([head, ...dirs, ...datas]);
}
(async () => {
  fs.mkdirSync(assets, { recursive: true }); fs.mkdirSync(icons, { recursive: true });
  const out = {};
  for (const n of [16, 32, 48, 64, 128, 256, 512, 1024]) out[n] = await png(n);
  fs.writeFileSync(path.join(assets, "logo-bullo.png"), out[1024]);
  for (const n of [512, 256, 64]) fs.writeFileSync(path.join(assets, `logo-bullo-${n}.png`), out[n]);
  fs.writeFileSync(path.join(assets, "favicon.png"), out[32]);
  const set = [16, 32, 48, 64, 128, 256].map(size => ({ size, buf: out[size] }));
  fs.writeFileSync(path.join(assets, "favicon.ico"), ico(set)); fs.writeFileSync(path.join(icons, "icon.ico"), ico(set));
  fs.writeFileSync(path.join(icons, "32x32.png"), out[32]); fs.writeFileSync(path.join(icons, "128x128.png"), out[128]);
  fs.writeFileSync(path.join(icons, "128x128@2x.png"), out[256]); fs.writeFileSync(path.join(icons, "icon.png"), out[512]);
  console.log("Variantes générées dans src/assets et src-tauri/icons. Pour l'icône macOS (.icns) : npm run tauri icon src/assets/logo-bullo.png");
})().catch(e => { console.error("Échec :", e.message); process.exit(1); });
