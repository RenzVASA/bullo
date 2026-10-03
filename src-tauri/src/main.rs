#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{params, params_from_iter, Connection};
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use std::{fs, io::{Read, Write}, net::TcpListener, process::Command, sync::{atomic::{AtomicBool, Ordering}, Mutex}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use tauri::{ipc::{InvokeBody, Request}, AppHandle, Manager, State};

struct Db(Mutex<Connection>);

#[derive(Serialize, Deserialize, Clone)]
struct Item { id: i64, kind: String, title: String, body: String, done: bool, created: String,
              course: Option<String>, data: Option<String>, audio: Option<String>, parent: Option<i64> }

const COLS: &str = "id,kind,title,body,done,created,course,data,audio,parent";
const PATH_ENV: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin";

fn e<E: ToString>(x: E) -> String { x.to_string() }

// Variables d'environnement : BULLO_* en priorité. FOCUSFLOW_* n'est lu qu'en repli, pour ne pas casser une ancienne installation.
fn ollama_model() -> String {
    std::env::var("BULLO_MODEL").or_else(|_| std::env::var("FOCUSFLOW_MODEL")).unwrap_or_else(|_| "qwen2.5:7b".to_string())
}
fn whisper_model() -> String {
    if let Ok(m) = std::env::var("BULLO_WHISPER_MODEL").or_else(|_| std::env::var("FOCUSFLOW_WHISPER_MODEL")) { return m; }
    let home = std::env::var("HOME").unwrap_or_default();
    let new = format!("{home}/.bullo/ggml-small.bin");
    let old = format!("{home}/.focusflow/ggml-small.bin");
    if !std::path::Path::new(&new).exists() && std::path::Path::new(&old).exists() { old } else { new }
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<Item> {
    Ok(Item { id: r.get(0)?, kind: r.get(1)?, title: r.get(2)?, body: r.get(3)?,
              done: r.get::<_, i64>(4)? != 0, created: r.get(5)?,
              course: r.get(6)?, data: r.get(7)?, audio: r.get(8)?, parent: r.get(9)? })
}

fn query(db: &Db, sql: String, p: Vec<String>) -> Result<Vec<Item>, String> {
    let c = db.0.lock().map_err(e)?;
    let mut s = c.prepare(&sql).map_err(e)?;
    let rows = s.query_map(params_from_iter(p), row).map_err(e)?;
    let out = rows.collect::<Result<Vec<_>, _>>().map_err(e)?;
    Ok(out)
}

#[tauri::command]
async fn add_item(db: State<'_, Db>, kind: String, title: String, body: String) -> Result<i64, String> {
    let c = db.0.lock().map_err(e)?;
    c.execute("INSERT INTO items(kind,title,body) VALUES(?1,?2,?3)", params![kind, title, body]).map_err(e)?;
    Ok(c.last_insert_rowid())
}

#[tauri::command]
async fn list_items(db: State<'_, Db>, kind: Option<String>) -> Result<Vec<Item>, String> {
    match kind {
        Some(k) => query(&db, format!("SELECT {COLS} FROM items WHERE kind=?1 ORDER BY id DESC"), vec![k]),
        None => query(&db, format!("SELECT {COLS} FROM items WHERE parent IS NULL ORDER BY id DESC"), vec![]),
    }
}

#[tauri::command]
async fn search(db: State<'_, Db>, q: String, course: Option<String>) -> Result<Vec<Item>, String> {
    let like = format!("%{}%", q.trim());
    let course = course.unwrap_or_default();
    query(&db, format!("SELECT {COLS} FROM items WHERE parent IS NULL AND (title LIKE ?1 OR body LIKE ?1 OR course LIKE ?1) \
        AND (?2 = '' OR course = ?2 OR substr(title, 1, length(?2) + 3) = ?2 || ' · ') ORDER BY id DESC LIMIT 100"), vec![like, course])
}

#[tauri::command]
async fn set_done(db: State<'_, Db>, id: i64, done: bool) -> Result<(), String> {
    db.0.lock().map_err(e)?.execute("UPDATE items SET done=?1 WHERE id=?2", params![done as i64, id]).map_err(e)?;
    Ok(())
}

#[tauri::command]
async fn delete_item(db: State<'_, Db>, id: i64) -> Result<(), String> {
    db.0.lock().map_err(e)?.execute("DELETE FROM items WHERE id=?1 OR parent=?1", params![id]).map_err(e)?;
    Ok(())
}

// Un cours = UNE entrée (résumé, transcription, audio dans `data`/`audio`) + ses devoirs comme tâches rattachées (parent).
#[tauri::command]
async fn add_course(db: State<'_, Db>, course: String, title: String, body: String, data: String, audio: Option<String>, tasks: Vec<String>) -> Result<i64, String> {
    let mut c = db.0.lock().map_err(e)?;
    let tx = c.transaction().map_err(e)?;
    tx.execute("INSERT INTO items(kind,title,body,course,data,audio) VALUES('cours',?1,?2,?3,?4,?5)", params![title, body, course, data, audio]).map_err(e)?;
    let id = tx.last_insert_rowid();
    for t in tasks {
        tx.execute("INSERT INTO items(kind,title,body,course,parent) VALUES('tache',?1,'',?2,?3)", params![format!("{course} · {t}"), course, id]).map_err(e)?;
    }
    tx.commit().map_err(e)?;
    Ok(id)
}

// Lecture de l'audio dans l'appli : les octets partent vers l'interface (lecteur <audio>), rien d'autre n'est lisible.
#[tauri::command]
fn read_audio(path: String) -> Result<tauri::ipc::Response, String> {
    let ok = ["m4a", "mp4", "webm", "wav", "mp3"].iter().any(|x| path.to_lowercase().ends_with(&format!(".{x}")));
    if !ok { return Err("Format audio non pris en charge.".into()); }
    let data = fs::read(&path).map_err(|_| "Audio introuvable (fichier déplacé ou supprimé ?).".to_string())?;
    Ok(tauri::ipc::Response::new(data))
}

// Écrit un export (texte, PDF, JSON) dans le dossier Téléchargements et renvoie le chemin complet.
#[tauri::command]
fn save_download(app: AppHandle, name: String, bytes: Vec<u8>) -> Result<String, String> {
    let clean: String = name.chars().map(|c| if "/\\:*?\"<>|".contains(c) { '-' } else { c }).collect();
    let clean = clean.trim().trim_start_matches('.').to_string();
    if clean.is_empty() { return Err("Nom de fichier vide.".into()); }
    let dir = app.path().download_dir().map_err(e)?;
    fs::create_dir_all(&dir).map_err(e)?;
    let p = dir.join(clean);
    fs::write(&p, bytes).map_err(e)?;
    Ok(p.to_string_lossy().into_owned())
}

// Reçoit les octets audio bruts (header x-ext = extension), les écrit dans le dossier de l'app.
#[tauri::command]
async fn save_audio(app: AppHandle, req: Request<'_>) -> Result<String, String> {
    let InvokeBody::Raw(data) = req.body() else { return Err("audio manquant".into()) };
    let ext = req.headers().get("x-ext").and_then(|v| v.to_str().ok()).unwrap_or("webm").to_string();
    let dir = audio_dir_of(&app)?;
    fs::create_dir_all(&dir).map_err(e)?;
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map_err(e)?.as_secs();
    let p = dir.join(format!("rec-{ts}.{ext}"));
    fs::write(&p, data).map_err(e)?;
    Ok(p.to_string_lossy().into_owned())
}


// Message précis selon la vraie cause : Ollama éteint, modèle absent, ou délai dépassé.
fn ollama_err(model: &str, er: ureq::Error) -> String {
    match er {
        ureq::Error::Status(404, _) => format!("Le modèle de résumé « {model} » n'est pas téléchargé. Ouvre Réglages > Vérifier les outils, puis « Télécharger »."),
        ureq::Error::Status(c, _) => format!("Ollama a répondu une erreur ({c}). Relance Ollama puis réessaie."),
        ureq::Error::Transport(t) if t.to_string().to_lowercase().contains("timed out") => "Le résumé prend trop de temps (plus de 5 minutes). Essaie avec un texte plus court.".to_string(),
        ureq::Error::Transport(_) => "Ollama ne répond pas. Ouvre l'application Ollama, puis réessaie (Réglages > Vérifier les outils).".to_string(),
    }
}

// ffmpeg (conversion 16 kHz mono) puis whisper.cpp, 100 % local. Thread dédié : l'UI ne fige jamais.
#[tauri::command]
async fn transcribe(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let wav = format!("{path}.wav");
        if !std::path::Path::new(&whisper_model()).exists() { return Err("Le modèle de transcription (Whisper) n'est pas installé. Ouvre Réglages > Vérifier les outils, puis « Télécharger ».".to_string()); }
        let ff = Command::new("ffmpeg").env("PATH", PATH_ENV)
            .args(["-y", "-i", &path, "-ar", "16000", "-ac", "1", &wav]).output()
            .map_err(|_| "ffmpeg introuvable. Installe-le avec : brew install ffmpeg".to_string())?;
        if !ff.status.success() { return Err("Conversion audio impossible avec ffmpeg.".to_string()); }
        let model = whisper_model();
        let w = Command::new("whisper-cli").env("PATH", PATH_ENV)
            .args(["-m", &model, "-f", &wav, "-l", "fr", "-nt", "-t", "4"]).output();
        let _ = fs::remove_file(&wav); // ne pas laisser traîner les .wav
        let w = w.map_err(|_| "whisper-cli introuvable. Installe-le avec : brew install whisper-cpp".to_string())?;
        if !w.status.success() { return Err(format!("Whisper a échoué (modèle : {model}).")); }
        Ok(String::from_utf8_lossy(&w.stdout).lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" "))
    }).await.map_err(e)?
}

// Ollama lit par défaut très peu de texte à la fois (contexte de 2 à 4 k « tokens ») et COUPE sans prévenir ce qui dépasse :
// un long cours n'était donc résumé qu'en partie. On demande ici un contexte adapté à la taille du texte, et au-delà d'un seuil
// on découpe le texte en morceaux résumés séparément (puis fusionnés) plutôt que de perdre la fin.
const OLLAMA_SINGLE_MAX: usize = 40_000; // caractères traités d'un coup (contexte 16 k)
const OLLAMA_CHUNK: usize = 30_000;
fn ollama_ctx(chars: usize) -> u32 {
    let need = chars / 3 + 1500;
    [4096u32, 8192, 16384].into_iter().find(|c| *c as usize >= need).unwrap_or(16384)
}
// Découpe en morceaux d'au plus `max` caractères, de préférence à une fin de phrase ou d'espace.
fn split_text(text: &str, max: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let mut end = (start + max).min(chars.len());
        if end < chars.len() {
            let lo = start + max / 2;
            if let Some(p) = (lo..end).rev().find(|&i| matches!(chars[i], '.' | '!' | '?' | '\n')) { end = p + 1; }
            else if let Some(p) = (lo..end).rev().find(|&i| chars[i].is_whitespace()) { end = p + 1; }
        }
        let piece: String = chars[start..end].iter().collect();
        if !piece.trim().is_empty() { out.push(piece); }
        start = end;
    }
    out
}
fn summarize_one(model: &str, text: &str) -> Result<serde_json::Value, String> {
    let prompt = format!(
        "Tu analyses la transcription d'un cours (ou d'une note vocale) en français, pour un élève dyslexique. Réponds uniquement en JSON avec les clés : \
         \"resume\" (3 à 4 phrases très simples qui couvrent TOUT le texte, du début à la fin), \"a_retenir\" (liste des notions, définitions et idées clés), \
         \"taches\" (devoirs et travail à faire, verbes à l'infinitif), \"echeances\" (dates de contrôles ou de rendus), \
         \"questions\" (points à demander au professeur). N'invente rien.\n\nTranscription :\n{text}");
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(600)).build();
    let r: serde_json::Value = agent.post("http://localhost:11434/api/generate")
        .send_json(serde_json::json!({ "model": model, "prompt": prompt, "stream": false, "format": "json", "keep_alive": "30s",
            "options": { "num_ctx": ollama_ctx(text.chars().count()), "num_predict": 2048 } }))
        .map_err(|er| ollama_err(model, er))?
        .into_json().map_err(e)?;
    serde_json::from_str(r["response"].as_str().unwrap_or("{}")).map_err(e)
}

// Extraction structurée via Ollama en local. keep_alive court : le modèle est libéré de la RAM juste après.
#[tauri::command]
async fn summarize(text: String) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let model = ollama_model();
        if text.chars().count() <= OLLAMA_SINGLE_MAX { return summarize_one(&model, &text); }
        // Texte très long : un résumé par morceau, puis fusion des listes et un résumé global.
        let mut resumes: Vec<String> = Vec::new();
        let mut merged: std::collections::BTreeMap<&str, Vec<serde_json::Value>> = std::collections::BTreeMap::new();
        for piece in split_text(&text, OLLAMA_CHUNK) {
            let v = summarize_one(&model, &piece)?;
            if let Some(r) = v["resume"].as_str() { if !r.trim().is_empty() { resumes.push(r.trim().to_string()); } }
            for k in ["a_retenir", "taches", "echeances", "questions"] {
                if let Some(a) = v[k].as_array() {
                    let list = merged.entry(k).or_default();
                    for x in a { if !list.contains(x) { list.push(x.clone()); } }
                }
            }
        }
        let joined = resumes.join(" ");
        let resume = if resumes.len() <= 1 { joined } else {
            let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(300)).build();
            let prompt = format!("Voici les résumés des parties successives d'un même cours. Fais un seul résumé de 4 à 6 phrases très simples, en français, qui couvre toutes les parties dans l'ordre. N'invente rien. Réponds uniquement avec le résumé.\n\n{joined}");
            match agent.post("http://localhost:11434/api/generate")
                .send_json(serde_json::json!({ "model": model, "prompt": prompt, "stream": false, "keep_alive": "30s", "options": { "num_ctx": ollama_ctx(joined.chars().count()), "num_predict": 1024 } }))
                .map_err(|er| ollama_err(&model, er)).and_then(|r| r.into_json::<serde_json::Value>().map_err(e)) {
                Ok(r) => { let t = r["response"].as_str().unwrap_or("").trim().to_string(); if t.is_empty() { joined } else { t } }
                Err(_) => joined,
            }
        };
        let mut out = serde_json::json!({ "resume": resume });
        for k in ["a_retenir", "taches", "echeances", "questions"] { out[k] = serde_json::Value::Array(merged.remove(k).unwrap_or_default()); }
        Ok(out)
    }).await.map_err(e)?
}


// ===== « Explique-moi » (méthode Feynman) : l'utilisateur explique, le modèle local pose des questions sans jamais donner la réponse =====
// Le texte ci-dessous est facile à modifier : c'est la consigne envoyée au modèle.
const EXPLAIN_PROMPT: &str = "Tu es un élève curieux de 12 ans. Quelqu'un t'explique un sujet qu'il vient d'étudier. Ton rôle est de trouver ce qui manque ou n'est pas clair dans son explication. Pose au maximum 3 questions courtes et simples, en français. Ne donne JAMAIS la réponse, ne corrige pas directement, ne fais pas de cours. Si l'explication est solide, dis-le simplement et pose une question pour aller un peu plus loin. Ton ton est bienveillant et encourageant.";
const EXPLAIN_BILAN_PROMPT: &str = "Tu es un élève curieux de 12 ans. Quelqu'un vient de t'expliquer un sujet et de répondre à tes questions. Fais un bilan très court, en français, en deux parties exactement. Première partie, qui commence par « Solide : » : 1 à 3 puces (« - ») sur ce qui a été bien expliqué. Deuxième partie, qui commence par « À revoir : » : 1 à 3 puces (« - ») qui nomment des SUJETS à retravailler. Ne donne JAMAIS la réponse ni la correction, ne fais pas de cours : nomme seulement les sujets (par exemple « le rôle de la lumière »). Ton bienveillant et encourageant.";

#[derive(Serialize, Deserialize)]
struct ChatMsg { role: String, content: String }

// Même Ollama local que le résumé et les outils (même adresse, même modèle). Ne bloque jamais l'interface : thread dédié.
#[tauri::command]
async fn explain_turn(topic: String, messages: Vec<ChatMsg>, bilan: bool) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if messages.is_empty() || messages.len() > 12 { return Err("Conversation invalide.".into()); }
        let model = ollama_model();
        let topic: String = topic.chars().take(200).collect();
        let sys = if bilan { EXPLAIN_BILAN_PROMPT.to_string() } else { format!("{EXPLAIN_PROMPT}\n\nSujet étudié : {topic}") };
        let mut msgs = vec![serde_json::json!({ "role": "system", "content": sys })];
        for m in &messages {
            let role = if m.role == "assistant" { "assistant" } else { "user" };
            let content: String = m.content.chars().take(8000).collect();
            msgs.push(serde_json::json!({ "role": role, "content": content }));
        }
        if bilan { msgs.push(serde_json::json!({ "role": "user", "content": format!("Sujet : {topic}\nFais le bilan maintenant.") })); }
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(300)).build();
        let r: serde_json::Value = agent.post("http://localhost:11434/api/chat")
            .send_json(serde_json::json!({ "model": model, "messages": msgs, "stream": false, "keep_alive": "30s", "options": { "num_ctx": 8192 } }))
            .map_err(|er| ollama_err(&model, er))?
            .into_json().map_err(e)?;
        let out = r["message"]["content"].as_str().unwrap_or("").trim().to_string();
        if out.is_empty() { return Err("Le modèle n'a rien répondu. Réessaie.".into()); }
        Ok(out)
    }).await.map_err(e)?
}

#[derive(Serialize)]
struct Explanation { id: i64, created: String, topic: String, explanation: String, turns: String, bilan: String }

// Crée la séance (id absent) ou la met à jour. `turns` et `bilan` sont du JSON gardé tel quel.
#[tauri::command]
async fn explain_save(db: State<'_, Db>, id: Option<i64>, topic: String, explanation: String, turns: String, bilan: String) -> Result<i64, String> {
    let c = db.0.lock().map_err(e)?;
    match id {
        Some(i) => { c.execute("UPDATE explanations SET topic=?2, explanation=?3, turns=?4, bilan=?5 WHERE id=?1", params![i, topic, explanation, turns, bilan]).map_err(e)?; Ok(i) }
        None => { c.execute("INSERT INTO explanations(topic,explanation,turns,bilan) VALUES(?1,?2,?3,?4)", params![topic, explanation, turns, bilan]).map_err(e)?; Ok(c.last_insert_rowid()) }
    }
}

#[tauri::command]
async fn explain_list(db: State<'_, Db>) -> Result<Vec<Explanation>, String> {
    let c = db.0.lock().map_err(e)?;
    let mut s = c.prepare("SELECT id,created,topic,explanation,turns,bilan FROM explanations ORDER BY id DESC").map_err(e)?;
    let rows = s.query_map([], |r| Ok(Explanation { id: r.get(0)?, created: r.get(1)?, topic: r.get(2)?, explanation: r.get(3)?, turns: r.get(4)?, bilan: r.get(5)? })).map_err(e)?;
    let out = rows.collect::<Result<Vec<_>, _>>().map_err(e)?;
    Ok(out)
}

#[tauri::command]
async fn explain_delete(db: State<'_, Db>, id: i64) -> Result<(), String> {
    db.0.lock().map_err(e)?.execute("DELETE FROM explanations WHERE id=?1", params![id]).map_err(e)?;
    Ok(())
}

// Outils IA courts (corriger, simplifier, décomposer, débloquer) via Ollama local.
#[tauri::command]
async fn assist(mode: String, text: String) -> Result<String, String> {
    let instr = match mode.as_str() {
        "corriger" => "Corrige l'orthographe, la grammaire et la clarté de ce texte français écrit par une personne dyslexique. Garde le sens et le ton. Réponds uniquement avec le texte corrigé.",
        "simplifier" => "Explique ce texte très simplement. Réponds par une phrase commençant par « En clair : », puis 3 à 5 lignes « À retenir ». N'invente rien.",
        "decomposer" => "Décompose cette tâche en 3 à 7 petites étapes concrètes, une par ligne, chacune commençant par un verbe à l'infinitif, sans numéro ni tiret. Réponds uniquement avec les lignes.",
        "consigne" => "Voici la consigne d'un devoir. Réponds en français simple avec trois blocs : « À rendre : », « Pour quand : » (« non précisé » si absent), « Étapes : » (3 à 6 lignes). N'invente rien.",
        "reviser" => "Voici un cours. Pose 5 questions de révision courtes et numérotées pour vérifier que l'élève a compris, chacune suivie de « Réponse : » en une phrase. N'invente rien.",
        "bloque" => "La personne est bloquée et ne sait pas par où commencer. Donne UNE seule première petite action très concrète, en une phrase, puis dis-lui de revenir pour la suivante. Ton bienveillant, jamais culpabilisant.",
        _ => return Err("Mode inconnu.".into()),
    };
    tauri::async_runtime::spawn_blocking(move || {
        let model = ollama_model();
        let n = text.chars().count();
        if n > OLLAMA_SINGLE_MAX { return Err(format!("Ce texte est trop long pour cet outil ({n} caractères, maximum {OLLAMA_SINGLE_MAX}). Découpe-le en plusieurs parties pour que rien ne soit perdu.")); }
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(600)).build();
        let r: serde_json::Value = agent.post("http://localhost:11434/api/generate")
            .send_json(serde_json::json!({ "model": model, "prompt": format!("{instr}\n\nTexte :\n{text}"), "stream": false, "keep_alive": "30s",
                "options": { "num_ctx": ollama_ctx(n), "num_predict": 4096 } }))
            .map_err(|er| ollama_err(&model, er))?
            .into_json().map_err(e)?;
        Ok(r["response"].as_str().unwrap_or("").trim().to_string())
    }).await.map_err(e)?
}



#[derive(Serialize, Deserialize)]
struct BackupEnvelope { version: u32, content_sha256: String, content: String }

fn backup_hash(content: &str) -> String {
    let mut h = Sha256::new(); h.update(content.as_bytes());
    format!("{:x}", h.finalize())
}

#[tauri::command]
fn export_backup(_app: AppHandle, db: State<'_, Db>, path: String) -> Result<String, String> {
    let items = query(&db, format!("SELECT {COLS} FROM items ORDER BY id"), vec![])?;
    let content = serde_json::to_string(&items).map_err(e)?;
    let env = BackupEnvelope { version: 1, content_sha256: backup_hash(&content), content };
    let p = std::path::PathBuf::from(path);
    if p.as_os_str().is_empty() { return Err("Fichier de sauvegarde vide.".into()); }
    fs::write(&p, serde_json::to_string_pretty(&env).map_err(e)?).map_err(e)?;
    Ok(p.to_string_lossy().into_owned())
}

#[tauri::command]
fn import_backup(db: State<'_, Db>, path: String) -> Result<usize, String> {
    let raw = fs::read_to_string(&path).map_err(|_| "Fichier de sauvegarde introuvable ou illisible.".to_string())?;
    let env: BackupEnvelope = serde_json::from_str(&raw).map_err(|_| "Format de sauvegarde invalide ou fichier corrompu.".to_string())?;
    let actual = backup_hash(&env.content);
    if actual != env.content_sha256 { return Err(format!("Sauvegarde refusée : intégrité invalide (hash SHA-256 attendu {}, obtenu {}). Le fichier a été modifié ou est corrompu.", env.content_sha256, actual)); }
    let items: Vec<Item> = serde_json::from_str(&env.content).map_err(|_| "Contenu de sauvegarde invalide.".to_string())?;
    let mut c = db.0.lock().map_err(e)?; let tx = c.transaction().map_err(e)?;
    tx.execute("DELETE FROM items", []).map_err(e)?;
    for i in &items { tx.execute("INSERT INTO items(id,kind,title,body,done,created,course,data,audio,parent) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![i.id,i.kind,i.title,i.body,i.done as i64,i.created,i.course,i.data,i.audio,i.parent]).map_err(e)?; }
    tx.commit().map_err(e)?; Ok(items.len())
}

// Export Markdown de tout le contenu dans Documents/Bullo-export.md (on n'est jamais prisonnier de l'appli).
#[tauri::command]
async fn export_md(app: AppHandle, db: State<'_, Db>) -> Result<String, String> {
    let items = query(&db, format!("SELECT {COLS} FROM items ORDER BY id"), vec![])?;
    let mut md = String::from("# Export Bullo\n\n");
    for i in items {
        if i.kind == "tache" { md += &format!("- [{}] {}\n", if i.done { "x" } else { " " }, i.title); }
        else { md += &format!("\n## {} ({})\n{}\n", i.title, i.kind, i.body); }
    }
    let p = app.path().document_dir().map_err(e)?.join("Bullo-export.md");
    fs::write(&p, md).map_err(e)?;
    Ok(p.to_string_lossy().into_owned())
}

// ===== Rappels et alarmes =====
// `due` = secondes Unix (UTC). Un thread d'arrière-plan notifie même quand la fenêtre est masquée.
#[derive(Serialize, Deserialize, Clone)]
struct Reminder { id: i64, title: String, due: i64, done: bool, notified: bool }

fn now_secs() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) }

// Notification système : seule partie spécifique à macOS (osascript). Ajouter ici Windows/Linux plus tard.
fn notify_system(title: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        let esc = |t: &str| t.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!("display notification \"{}\" with title \"{}\" sound name \"Glass\"", esc(body), esc(title));
        let _ = Command::new("osascript").args(["-e", script.as_str()]).output();
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = (title, body); }
}

#[tauri::command]
async fn notify(title: String, body: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || notify_system(&title, &body)).await.map_err(e)
}

#[tauri::command]
async fn add_reminder(db: State<'_, Db>, title: String, due: i64) -> Result<i64, String> {
    if title.trim().is_empty() { return Err("Le rappel n'a pas de titre.".into()); }
    let c = db.0.lock().map_err(e)?;
    c.execute("INSERT INTO reminders(title,due) VALUES(?1,?2)", params![title.trim(), due]).map_err(e)?;
    Ok(c.last_insert_rowid())
}

#[tauri::command]
async fn list_reminders(db: State<'_, Db>) -> Result<Vec<Reminder>, String> {
    let c = db.0.lock().map_err(e)?;
    let mut s = c.prepare("SELECT id,title,due,done,notified FROM reminders ORDER BY due").map_err(e)?;
    let rows = s.query_map([], |r| Ok(Reminder { id: r.get(0)?, title: r.get(1)?, due: r.get(2)?, done: r.get::<_, i64>(3)? != 0, notified: r.get::<_, i64>(4)? != 0 })).map_err(e)?;
    let out = rows.collect::<Result<Vec<_>, _>>().map_err(e)?;
    Ok(out)
}

// Modifie un rappel : titre, échéance (réarme la notification), terminé / à refaire.
#[tauri::command]
async fn update_reminder(db: State<'_, Db>, id: i64, title: Option<String>, due: Option<i64>, done: Option<bool>) -> Result<(), String> {
    let c = db.0.lock().map_err(e)?;
    if let Some(t) = title { if !t.trim().is_empty() { c.execute("UPDATE reminders SET title=?1 WHERE id=?2", params![t.trim(), id]).map_err(e)?; } }
    if let Some(d) = due { c.execute("UPDATE reminders SET due=?1, notified=0, done=0 WHERE id=?2", params![d, id]).map_err(e)?; }
    if let Some(x) = done { c.execute("UPDATE reminders SET done=?1 WHERE id=?2", params![x as i64, id]).map_err(e)?; }
    Ok(())
}

#[tauri::command]
async fn delete_reminder(db: State<'_, Db>, id: i64) -> Result<(), String> {
    db.0.lock().map_err(e)?.execute("DELETE FROM reminders WHERE id=?1", params![id]).map_err(e)?;
    Ok(())
}

fn due_reminders(db: &Db) -> Result<Vec<(i64, String)>, String> {
    let c = db.0.lock().map_err(e)?;
    let mut s = c.prepare("SELECT id,title FROM reminders WHERE done=0 AND notified=0 AND due<=?1 ORDER BY due").map_err(e)?;
    let rows = s.query_map(params![now_secs()], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))).map_err(e)?;
    let out = rows.collect::<Result<Vec<_>, _>>().map_err(e)?;
    Ok(out)
}

fn start_reminder_watcher(handle: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(15));
        let db = handle.state::<Db>();
        let due = match due_reminders(&db) { Ok(v) => v, Err(_) => continue };
        for (id, title) in due {
            notify_system("Bullo — rappel", &title);
            if let Ok(c) = db.0.lock() { let _ = c.execute("UPDATE reminders SET notified=1 WHERE id=?1", params![id]); }
        }
    });
}

// ===== Lecteurs externes (Musique / Spotify) : contrôle réel via AppleScript, macOS uniquement =====
// Seuls deux noms d'application sont acceptés (liste blanche) : aucun texte venant de l'interface n'est injecté dans un script.
fn media_app(name: &str) -> Result<&'static str, String> {
    match name { "Music" => Ok("Music"), "Spotify" => Ok("Spotify"), _ => Err("Application inconnue.".into()) }
}

fn run_applescript(script: &str) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let o = Command::new("osascript").args(["-e", script]).output().map_err(e)?;
        if o.status.success() { return Ok(String::from_utf8_lossy(&o.stdout).trim().to_string()); }
        let err = String::from_utf8_lossy(&o.stderr).to_string();
        if err.contains("-1743") || err.contains("not allowed") {
            return Err("macOS bloque le contrôle. Ouvre Réglages Système > Confidentialité et sécurité > Automatisation, puis autorise Bullo à contrôler cette application.".into());
        }
        Err(format!("Le lecteur n'a pas répondu : {}", err.lines().next().unwrap_or("erreur inconnue").trim()))
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = script; Err("Le contrôle de Musique et Spotify n'est disponible que sur Mac pour l'instant.".into()) }
}

// Application ciblée par son identifiant (évite la boîte « Où est Spotify ? » si elle n'est pas installée).
fn app_id(name: &str) -> &'static str { if name == "Spotify" { "com.spotify.client" } else { "com.apple.Music" } }
fn app_label(name: &str) -> &'static str { if name == "Spotify" { "Spotify" } else { "Apple Music" } }
// Détection du processus SANS AppleScript : aucune autorisation requise, jamais d'erreur si l'appli est absente.
fn is_running(name: &str) -> bool {
    Command::new("pgrep").args(["-x", name]).output().map(|o| o.status.success()).unwrap_or(false)
}

#[derive(Serialize)]
struct MediaStatus { app: String, running: bool, state: String, title: String, artist: String, repeat: String, shuffle: String, fav: String }

fn app_status(name: &'static str) -> Result<MediaStatus, String> {
    if !is_running(name) { return Ok(MediaStatus { app: name.to_string(), running: false, state: "stopped".into(), title: String::new(), artist: String::new(), repeat: String::new(), shuffle: String::new(), fav: String::new() }); }
    // Music : song repeat (off/one/all), shuffle enabled, favorited. Spotify : repeating, shuffling (pas de favori scriptable).
    let extra = if name == "Spotify" {
        "set rp to \"\"\n  set sh to \"\"\n  set fv to \"\"\n  try\n    set rp to (repeating as string)\n    if rp is \"true\" then set rp to \"all\"\n    if rp is \"false\" then set rp to \"off\"\n  end try\n  try\n    set sh to (shuffling as string)\n  end try"
    } else {
        "set rp to \"\"\n  set sh to \"\"\n  set fv to \"\"\n  try\n    set rp to (song repeat as string)\n  end try\n  try\n    set sh to (shuffle enabled as string)\n  end try\n  try\n    set fv to (favorited of current track as string)\n  end try"
    };
    let script = format!("with timeout of 4 seconds\ntell application id \"{id}\"\n  set st to \"stopped\"\n  try\n    if player state is playing then set st to \"playing\"\n    if player state is paused then set st to \"paused\"\n  end try\n  set t to \"\"\n  set a to \"\"\n  try\n    set t to name of current track\n    set a to artist of current track\n  end try\n  {extra}\n  return st & \"|||\" & t & \"|||\" & a & \"|||\" & rp & \"|||\" & sh & \"|||\" & fv\nend tell\nend timeout", id = app_id(name), extra = extra);
    let out = run_applescript(&script)?;
    let p: Vec<&str> = out.split("|||").collect();
    let g = |i: usize| p.get(i).unwrap_or(&"").to_string();
    Ok(MediaStatus { app: name.to_string(), running: true, state: if p.is_empty() { "stopped".into() } else { g(0) }, title: g(1), artist: g(2), repeat: g(3), shuffle: g(4), fav: g(5) })
}

#[tauri::command]
async fn media_status(app: String) -> Result<MediaStatus, String> {
    let name = media_app(&app)?;
    tauri::async_runtime::spawn_blocking(move || app_status(name)).await.map_err(e)?
}

#[tauri::command]
async fn media_control(app: String, action: String) -> Result<(), String> {
    NP_FRESH.store(true, Ordering::Relaxed);
    if app == "system" {
        return match action.as_str() { "repeat" | "shuffle" | "favorite" => Err("Cette action n'est disponible que pour Apple Music et Spotify. Pour un navigateur, utilise ses propres boutons.".into()), _ => media_key(action).await };
    }
    let name = media_app(&app)?;
    let spotify = name == "Spotify";
    let stmt: String = match action.as_str() {
        "playpause" => "playpause".into(), "play" => "play".into(), "pause" => "pause".into(), "next" => "next track".into(), "previous" => "previous track".into(),
        "shuffle" => if spotify { "set shuffling to not shuffling".into() } else { "set shuffle enabled to not shuffle enabled".into() },
        "repeat" => if spotify { "set repeating to not repeating".into() } else { "set r to song repeat as string\n    if r is \"off\" then\n      set song repeat to all\n    else if r is \"all\" then\n      set song repeat to one\n    else\n      set song repeat to off\n    end if".into() },
        "favorite" => { if spotify { return Err("Spotify ne permet pas de mettre en favori depuis un autre programme : Bullo l'ajoute à ta liste de favoris Bullo.".into()); } "set favorited of current track to not (favorited of current track)".into() }
        _ => return Err("Action inconnue.".into()),
    };
    tauri::async_runtime::spawn_blocking(move || {
        if !is_running(name) { return Err(format!("{} n'est pas ouvert.", app_label(name))); }
        run_applescript(&format!("with timeout of 4 seconds\ntell application id \"{}\"\n    {}\nend tell\nend timeout", app_id(name), stmt)).map(|_| ())
    }).await.map_err(e)?
}

#[tauri::command]
async fn media_volume(app: String, volume: i64) -> Result<(), String> {
    let name = media_app(&app)?;
    let v = volume.clamp(0, 100);
    tauri::async_runtime::spawn_blocking(move || {
        if !is_running(name) { return Ok(()); }
        run_applescript(&format!("tell application id \"{}\" to set sound volume to {}", app_id(name), v)).map(|_| ())
    }).await.map_err(e)?
}

#[tauri::command]
async fn media_open(app: String) -> Result<(), String> {
    let name = media_app(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        Command::new("open").args(["-b", app_id(name)]).output().map_err(e).and_then(|o| if o.status.success() { Ok(()) } else { Err(format!("{} n'est pas installé sur ce Mac.", app_label(name))) })
    }).await.map_err(e)?
}

// ===== Ouvrir une adresse https dans le navigateur par défaut =====
// Ouvre uniquement une adresse https:// (celle que l'utilisateur a saisie) dans son navigateur, jamais autre chose.
#[tauri::command]
fn open_web(url: String) -> Result<(), String> {
    if !url.starts_with("https://") || url.len() > 500 || url.chars().any(|c| c.is_whitespace() || c.is_control()) { return Err("Adresse non valide : elle doit commencer par https://".into()); }
    Command::new("open").arg(url).spawn().map(|_| ()).map_err(e)
}

// ===== Musique en cours, quelle que soit l'application : Music, Spotify, onglets de navigateur, ou n'importe quel son =====
// Touches média du Mac (lecture/pause, suivant, précédent) : elles pilotent l'appli « en lecture » du système, navigateur compris.
// Nécessite d'autoriser Bullo dans Réglages Système > Confidentialité et sécurité > Accessibilité (demandé une seule fois).
fn run_jxa(script: &str) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let o = Command::new("osascript").args(["-l", "JavaScript", "-e", script]).output().map_err(e)?;
        if o.status.success() { return Ok(String::from_utf8_lossy(&o.stdout).trim().to_string()); }
        let err = String::from_utf8_lossy(&o.stderr).to_string();
        Err(format!("Impossible d'envoyer la touche média : {}. Autorise Bullo dans Réglages Système > Confidentialité et sécurité > Accessibilité.", err.lines().next().unwrap_or("erreur inconnue").trim()))
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = script; Err("Les touches média ne sont disponibles que sur Mac pour l'instant.".into()) }
}

async fn media_key(action: String) -> Result<(), String> {
    NP_FRESH.store(true, Ordering::Relaxed);
    let key = match action.as_str() { "playpause" | "play" | "pause" => 16, "next" => 17, "previous" => 18, _ => return Err("Action inconnue.".into()) };
    let script = format!(
        "ObjC.import('Cocoa');\nfunction k(code, down) {{ var ev = $.NSEvent.otherEventWithTypeLocationModifierFlagsTimestampWindowNumberContextSubtypeData1Data2(14, {{x: 0, y: 0}}, down ? 0xa00 : 0xb00, 0, 0, 0, 8, (code << 16) | ((down ? 0xa : 0xb) << 8), -1); $.CGEventPost(0, ev.CGEvent); }}\nk({key}, true); k({key}, false); 'ok'");
    tauri::async_runtime::spawn_blocking(move || run_jxa(&script).map(|_| ())).await.map_err(e)?
}

#[derive(Serialize, Clone)]
struct NowPlaying { kind: String, app: String, label: String, state: String, title: String, artist: String, warn: String, repeat: String, shuffle: String, fav: String, native: bool }

const BROWSERS: [(&str, &str, &str); 8] = [("Safari", "com.apple.Safari", "safari"), ("Google Chrome", "com.google.Chrome", "chrome"), ("Arc", "company.thebrowser.Browser", "chrome"), ("Brave Browser", "com.brave.Browser", "chrome"), ("Microsoft Edge", "com.microsoft.edgemac", "chrome"), ("Opera", "com.operasoftware.Opera", "chrome"), ("Opera GX", "com.operasoftware.OperaGX", "chrome"), ("Vivaldi", "com.vivaldi.Vivaldi", "chrome")];
// Navigateur ouvert ? On cherche son exécutable (Opera et Opera GX ont le même nom de processus, pas le même dossier).
fn browser_running(name: &str) -> bool {
    Command::new("pgrep").args(["-f", &format!("/{}.app/Contents/MacOS/", name)]).output().map(|o| o.status.success()).unwrap_or(false)
}
const MUSIC_SITES: [&str; 8] = ["open.spotify.com", "music.youtube.com", "youtube.com/watch", "music.apple.com", "deezer.com", "soundcloud.com", "tidal.com", "qobuz.com"];

// Titre du premier onglet de musique/vidéo ouvert dans ce navigateur (nécessite l'autorisation Automatisation, demandée une fois).
fn browser_tab(name: &str, id: &str, family: &str) -> Result<Option<String>, String> {
    let cond = MUSIC_SITES.iter().map(|h| format!("u contains \"{h}\"")).collect::<Vec<_>>().join(" or ");
    let (title_prop, url_prop) = if family == "safari" { ("name of t", "URL of t") } else { ("title of t", "URL of t") };
    let script = format!("with timeout of 5 seconds\ntell application id \"{id}\"\n  try\n    repeat with w in windows\n      repeat with t in tabs of w\n        set u to {url_prop}\n        if u is not missing value and ({cond}) then return {title_prop}\n      end repeat\n    end repeat\n  end try\n  return \"\"\nend tell\nend timeout");
    let _ = name;
    let out = run_applescript(&script)?;
    Ok(if out.is_empty() { None } else { Some(out) })
}

// Nettoie un titre d'onglet : « Titre • Artiste » (Spotify web), suffixes de sites retirés.
fn clean_tab_title(raw: &str) -> (String, String) {
    let mut t = raw.trim().to_string();
    for suf in [" - YouTube Music", " - YouTube", " | Deezer", " | Free Listening on SoundCloud", " - Spotify", " - Apple Music"] { if let Some(x) = t.strip_suffix(suf) { t = x.to_string(); } }
    if t.is_empty() || t.starts_with("Spotify") || t == "YouTube" || t == "YouTube Music" { return (String::new(), String::new()); }
    if let Some((a, b)) = t.split_once(" • ") { return (a.trim().to_string(), b.trim().to_string()); }
    (t, String::new())
}

// Un son est-il joué en ce moment ? macOS l'indique par une « assertion » d'audio (pmset). Renvoie le nom du processus si connu.
fn audio_playing() -> Option<String> {
    let o = Command::new("pmset").args(["-g", "assertions"]).output().ok()?;
    let text = String::from_utf8_lossy(&o.stdout).to_string();
    let mut found = false;
    for line in text.lines() {
        let l = line.to_lowercase();
        if !(l.contains("named:") && l.contains("audio")) { continue; }
        found = true;
        let name = line.split("pid").nth(1).and_then(|r| r.split('(').nth(1)).and_then(|r| r.split(')').next()).unwrap_or("").replace(" Helper", "").trim().to_string();
        if !name.is_empty() && name != "coreaudiod" { return Some(name); }
    }
    if found { Some(String::new()) } else { None }
}

// D'où vient le son ? On le déduit du nom du processus qui tient l'« assertion audio » (pmset) : une seule commande légère,
// puis on n'interroge QUE l'application concernée (jamais tous les navigateurs).
#[derive(Debug, PartialEq, Clone)]
enum Src { Native(&'static str), Browser(usize), Other(String) }

fn base_app(raw: &str) -> String {
    let mut n = raw.trim().to_string();
    if let Some(i) = n.find(" Helper") { n.truncate(i); }
    if let Some(i) = n.find(" (") { n.truncate(i); }
    n.trim().to_string()
}
fn classify(raw: &str) -> Src {
    let n = base_app(raw);
    let l = n.to_lowercase();
    if l == "spotify" { return Src::Native("Spotify"); }
    if l == "music" { return Src::Native("Music"); }
    if l.contains("webkit") || l == "safari" { return Src::Browser(0); }
    for (i, (name, _, _)) in BROWSERS.iter().enumerate() { if l == name.to_lowercase() { return Src::Browser(i); } }
    Src::Other(n)
}

// Pour économiser le processeur : un titre d'onglet est relu au plus toutes les 10 s, un état « en pause » toutes les 15 s.
static NP_CACHE: Mutex<Option<(Instant, String, NowPlaying)>> = Mutex::new(None);
static NP_FRESH: AtomicBool = AtomicBool::new(false);
fn np_cached(key: &str, max_s: u64) -> Option<NowPlaying> {
    if NP_FRESH.swap(false, Ordering::Relaxed) { return None; }
    let g = NP_CACHE.lock().ok()?;
    match &*g { Some((t, k, v)) if k == key && t.elapsed() < Duration::from_secs(max_s) => Some(v.clone()), _ => None }
}
fn np_store(key: &str, v: &NowPlaying) { if let Ok(mut g) = NP_CACHE.lock() { *g = Some((Instant::now(), key.to_string(), v.clone())); } }

#[tauri::command]
async fn now_playing() -> Result<NowPlaying, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mk = |kind: &str, app: &str, label: &str, state: &str, title: &str, artist: &str, warn: &str| NowPlaying { kind: kind.into(), app: app.into(), label: label.into(), state: state.into(), title: title.into(), artist: artist.into(), warn: warn.into(), repeat: String::new(), shuffle: String::new(), fav: String::new(), native: false };
        let with_native = |mut n: NowPlaying, st: &MediaStatus| { n.repeat = st.repeat.clone(); n.shuffle = st.shuffle.clone(); n.fav = st.fav.clone(); n.native = true; n };
        // Étape 1 (toujours, très léger) : un son est-il joué, et par quel processus ?
        match audio_playing() {
            Some(raw) => match if raw.is_empty() { Src::Other(String::new()) } else { classify(&raw) } {
                // Music ou Spotify : une seule requête, à cette application seulement.
                Src::Native(name) => {
                    return Ok::<_, String>(match app_status(name) {
                        Ok(st) if st.running => with_native(mk("app", name, app_label(name), if st.state == "playing" { "playing" } else { "paused" }, &st.title, &st.artist, ""), &st),
                        Ok(_) => mk("system", "system", app_label(name), "playing", "", "", ""),
                        Err(er) => mk("system", "system", app_label(name), "playing", "", "", &er),
                    });
                }
                // Un navigateur : on lit le titre de CE navigateur seulement, et pas plus d'une fois toutes les 10 s.
                Src::Browser(i) => {
                    let (name, id, fam) = BROWSERS[i];
                    let key = format!("browser:{name}");
                    if let Some(c) = np_cached(&key, 10) { return Ok(c); }
                    let (title, artist, warn) = match browser_tab(name, id, fam) {
                        Ok(Some(t)) => { let (a, b) = clean_tab_title(&t); (a, b, String::new()) }
                        Ok(None) => (String::new(), String::new(), String::new()),
                        Err(er) => (String::new(), String::new(), format!("{name} : {er}")),
                    };
                    let n = mk("browser", "system", name, "playing", &title, &artist, &warn);
                    np_store(&key, &n);
                    return Ok(n);
                }
                // Autre chose (Netflix dans une app, jeu, etc.) : on affiche juste le nom du processus.
                Src::Other(n) => return Ok(mk("system", "system", if n.is_empty() { "Ordinateur" } else { &n }, "playing", "", "", "")),
            },
            None => {}
        }
        // Étape 2 : aucun son. On ne regarde que Music et Spotify (un seul pgrep), et pas plus d'une fois toutes les 15 s.
        if let Some(c) = np_cached("idle", 15) { return Ok(c); }
        let running = Command::new("pgrep").args(["-lx", "Music|Spotify"]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        let name = if running.contains("Spotify") { Some("Spotify") } else if running.lines().any(|l| l.trim_end().ends_with("Music")) { Some("Music") } else { None };
        let n = match name {
            Some(nm) => match app_status(nm) {
                Ok(st) if st.running => with_native(mk("app", nm, app_label(nm), "paused", &st.title, &st.artist, ""), &st),
                Ok(_) => mk("none", "system", "", "stopped", "", "", ""),
                Err(er) => mk("none", "system", "", "stopped", "", "", &er),
            },
            None => mk("none", "system", "", "stopped", "", "", ""),
        };
        np_store("idle", &n);
        Ok(n)
    }).await.map_err(e)?
}

// ===== Diagnostic du lecteur : montre ce que Bullo voit réellement (à envoyer si quelque chose ne marche pas) =====
#[tauri::command]
async fn player_diagnostic() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut out = String::new();
        let ver = Command::new("sw_vers").arg("-productVersion").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_else(|_| "inconnue".into());
        out.push_str(&format!("macOS {}\n\n", ver));
        for name in ["Music", "Spotify"] {
            out.push_str(&format!("{} : ", app_label(name)));
            match app_status(name) { Ok(s) if !s.running => out.push_str("pas ouvert\n"), Ok(s) => out.push_str(&format!("ouvert · état={} · titre=« {} » · artiste=« {} » · boucle={} · aléatoire={} · favori={}\n", s.state, s.title, s.artist, s.repeat, s.shuffle, s.fav)), Err(er) => out.push_str(&format!("ERREUR {}\n", er)) }
        }
        out.push('\n');
        for (name, id, fam) in BROWSERS {
            out.push_str(&format!("{} : ", name));
            if !browser_running(name) { out.push_str("pas ouvert\n"); continue; }
            match browser_tab(name, id, fam) { Ok(Some(t)) => out.push_str(&format!("onglet musique/vidéo trouvé : « {} »\n", t)), Ok(None) => out.push_str("ouvert, aucun onglet musique/vidéo reconnu (le lecteur intégré d'Opera GX n'est pas un onglet)\n"), Err(er) => out.push_str(&format!("ERREUR {}\n", er)) }
        }
        out.push_str("\nSon détecté par macOS (pmset) :\n");
        let pm = Command::new("pmset").args(["-g", "assertions"]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        let lines: Vec<&str> = pm.lines().filter(|l| l.to_lowercase().contains("audio") || l.to_lowercase().contains("playing")).take(8).collect();
        if lines.is_empty() { out.push_str("  aucun son en cours détecté\n"); } else { for l in lines { out.push_str(&format!("  {}\n", l.trim())); } }
        out.push_str(&format!("\nRésultat de la détection : {}\n", match audio_playing() { Some(n) if n.is_empty() => "un son joue (application inconnue)".to_string(), Some(n) => format!("un son joue ({})", n), None => "aucun son".to_string() }));
        Ok::<_, String>(out)
    }).await.map_err(e)?
}

#[tauri::command]
async fn media_key_test() -> Result<String, String> {
    media_key("playpause".to_string()).await?;
    Ok("Touche lecture/pause envoyée sans erreur. Si rien n'a changé, autorise Bullo dans Réglages Système > Confidentialité et sécurité > Accessibilité, puis relance Bullo.".into())
}

// ===== Son de l'ordinateur : volume et coupure globale (tout le Mac, quelle que soit l'application) =====
#[derive(Serialize)]
struct SysAudio { volume: i64, muted: bool }

#[tauri::command]
async fn system_audio() -> Result<SysAudio, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let out = run_applescript("set v to get volume settings\nreturn ((output volume of v) as string) & \"|\" & ((output muted of v) as string)")?;
        let mut it = out.split('|');
        let volume = it.next().and_then(|x| x.trim().parse::<i64>().ok()).unwrap_or(-1);
        let muted = it.next().map(|x| x.trim() == "true").unwrap_or(false);
        Ok::<_, String>(SysAudio { volume, muted })
    }).await.map_err(e)?
}

#[tauri::command]
async fn system_mute(muted: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || run_applescript(&format!("set volume output muted {}", muted)).map(|_| ())).await.map_err(e)?
}

#[tauri::command]
async fn system_volume(volume: i64) -> Result<(), String> {
    let v = volume.clamp(0, 100);
    tauri::async_runtime::spawn_blocking(move || run_applescript(&format!("set volume output volume {}", v)).map(|_| ())).await.map_err(e)?
}

// ===== Polices téléchargeables (Lexend, Atkinson Hyperlegible : licence SIL OFL) =====
const FONT_FILES: [(&str, &str, &str); 4] = [("lexend-400.woff2", "lexend", "400"), ("lexend-700.woff2", "lexend", "700"), ("atkinson-400.woff2", "atkinson-hyperlegible", "400"), ("atkinson-700.woff2", "atkinson-hyperlegible", "700")];

fn fonts_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> { Ok(app.path().app_data_dir().map_err(e)?.join("fonts")) }

#[tauri::command]
async fn install_fonts(app: AppHandle) -> Result<Vec<String>, String> {
    let dir = fonts_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        fs::create_dir_all(&dir).map_err(e)?;
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(40)).build();
        let mut done = vec![];
        for (file, id, w) in FONT_FILES {
            let urls = [format!("https://cdn.jsdelivr.net/fontsource/fonts/{id}@latest/latin-{w}-normal.woff2"), format!("https://unpkg.com/@fontsource/{id}/files/{id}-latin-{w}-normal.woff2")];
            let mut got: Option<Vec<u8>> = None;
            for u in &urls {
                if let Ok(r) = agent.get(u).call() {
                    let mut buf = vec![]; use std::io::Read;
                    if r.into_reader().take(4_000_000).read_to_end(&mut buf).is_ok() && buf.len() > 5000 && &buf[..4] == b"wOF2" { got = Some(buf); break; }
                }
            }
            let data = got.ok_or_else(|| "Téléchargement impossible : vérifie ta connexion internet puis réessaie.".to_string())?;
            fs::write(dir.join(file), data).map_err(e)?;
            done.push(file.to_string());
        }
        Ok(done)
    }).await.map_err(e)?
}

#[tauri::command]
fn list_fonts(app: AppHandle) -> Result<Vec<String>, String> {
    let dir = fonts_dir(&app)?;
    Ok(FONT_FILES.iter().filter(|(f, _, _)| dir.join(f).exists()).map(|(f, _, _)| f.to_string()).collect())
}

#[tauri::command]
fn read_font(app: AppHandle, file: String) -> Result<tauri::ipc::Response, String> {
    if !FONT_FILES.iter().any(|(f, _, _)| *f == file) { return Err("Police inconnue.".into()); }
    Ok(tauri::ipc::Response::new(fs::read(fonts_dir(&app)?.join(file)).map_err(e)?))
}

// ===== Google Drive : sauvegarde et restauration (OAuth « application de bureau », PKCE, rien ne transite par un serveur tiers) =====
// Les clés (ID client, secret, jeton de renouvellement) sont rangées dans le Trousseau macOS, jamais dans un fichier ni dans les logs.
static GOOGLE_CANCEL: AtomicBool = AtomicBool::new(false);
const KC_ID: &str = "bullo-google-client-id";
const KC_SECRET: &str = "bullo-google-client-secret";
const KC_REFRESH: &str = "bullo-google-refresh-token";
const KC_EMAIL: &str = "bullo-google-email";
const KC_FP: &str = "bullo-google-fingerprint";

fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{:02x}", x)).collect() }
// Empreinte SHA-256 des clés : détecte toute corruption ou modification dans le Trousseau (jamais la clé elle-même).
fn key_fingerprint(id: &str, secret: &str) -> String { hex(&Sha256::digest(format!("bullo|{}|{}", id, secret).as_bytes()))[..16].to_string() }
const DRIVE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file"; // accès limité aux seuls fichiers créés par Bullo
const DRIVE_FOLDER: &str = "Bullo";
const DRIVE_FILE: &str = "bullo-sauvegarde.json";

fn kc_set(name: &str, val: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if val.is_empty() || !val.chars().all(|c| c.is_ascii_alphanumeric() || "-_.~/+=:@".contains(c)) {
            return Err("Cette valeur contient des caractères inattendus : copie-la à nouveau depuis Google, sans espace ni retour à la ligne.".into());
        }
        let o = Command::new("security").args(["add-generic-password", "-a", "bullo", "-s", name, "-w", val, "-U"]).output().map_err(e)?;
        if !o.status.success() { return Err("Le Trousseau macOS a refusé l'enregistrement.".into()); }
        // Relecture de contrôle : on vérifie que la valeur est bien rangée.
        if kc_get(name).as_deref() != Some(val) { return Err("Le Trousseau macOS n'a pas conservé la valeur. Réessaie.".into()); }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = (name, val); Err("Le rangement sécurisé des clés n'est disponible que sur Mac pour l'instant.".into()) }
}
fn kc_get(name: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let o = Command::new("security").args(["find-generic-password", "-a", "bullo", "-s", name, "-w"]).output().ok()?;
        if !o.status.success() { return None; }
        let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if v.is_empty() { None } else { Some(v) }
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = name; None }
}
fn kc_del(name: &str) {
    #[cfg(target_os = "macos")]
    { let _ = Command::new("security").args(["delete-generic-password", "-a", "bullo", "-s", name]).output(); }
    #[cfg(not(target_os = "macos"))]
    { let _ = name; }
}

fn urlenc(s: &str) -> String {
    s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{:02X}", b) }).collect()
}
fn urldec(s: &str) -> String {
    let b = s.as_bytes(); let mut out = Vec::with_capacity(b.len()); let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Some(Ok(v)) = s.get(i + 1..i + 3).map(|h| u8::from_str_radix(h, 16)) { out.push(v); i += 3; continue; }
        }
        out.push(if b[i] == b'+' { b' ' } else { b[i] }); i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
fn b64url(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(T[(n >> 18 & 63) as usize] as char); out.push(T[(n >> 12 & 63) as usize] as char);
        if c.len() > 1 { out.push(T[(n >> 6 & 63) as usize] as char); }
        if c.len() > 2 { out.push(T[(n & 63) as usize] as char); }
    }
    out
}
fn rand_bytes(n: usize) -> Result<Vec<u8>, String> {
    let mut f = fs::File::open("/dev/urandom").map_err(|_| "Générateur aléatoire indisponible.".to_string())?;
    let mut b = vec![0u8; n]; f.read_exact(&mut b).map_err(e)?; Ok(b)
}

// Traduit une erreur réseau/Google en phrase claire pour l'utilisateur.
fn gerr(err: ureq::Error) -> String {
    match err {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            let v: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
            let msg = v["error"]["message"].as_str().or(v["error_description"].as_str()).unwrap_or("").to_string();
            let low = format!("{msg} {body}").to_lowercase();
            if low.contains("accessnotconfigured") || low.contains("has not been used") || low.contains("api has not been enabled") || low.contains("is disabled") {
                return "L'API Google Drive n'est pas activée dans ton projet Google Cloud (étape 2 du tutoriel).".into();
            }
            if low.contains("invalid_client") || low.contains("unauthorized_client") { return "Google ne reconnaît pas ton ID client ou ton secret. Vérifie-les dans le tutoriel (étape 4).".into(); }
            if code == 401 { return "Google a refusé l'accès : reconnecte-toi dans Réglages > Google.".into(); }
            format!("Google a répondu une erreur {code}{}", if msg.is_empty() { String::new() } else { format!(" : {msg}") })
        }
        ureq::Error::Transport(_) => "Impossible de joindre Google : vérifie ta connexion internet.".into(),
    }
}

fn parse_query(q: &str) -> std::collections::HashMap<String, String> {
    q.split('&').filter_map(|kv| { let mut it = kv.splitn(2, '='); Some((urldec(it.next()?), urldec(it.next().unwrap_or("")))) }).collect()
}

// Attend le retour du navigateur sur http://127.0.0.1:<port> (adresse locale, jamais exposée au réseau).
fn wait_for_code(listener: &TcpListener, state: &str) -> Result<String, String> {
    listener.set_nonblocking(true).map_err(e)?;
    let start = Instant::now();
    loop {
        if GOOGLE_CANCEL.load(Ordering::SeqCst) { return Err("Connexion annulée.".into()); }
        if start.elapsed() > Duration::from_secs(300) { return Err("Délai dépassé : la connexion Google n'a pas abouti. Réessaie.".into()); }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let _ = stream.set_nonblocking(false); let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let mut buf = [0u8; 8192]; let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let target = req.lines().next().unwrap_or("").split_whitespace().nth(1).unwrap_or("").to_string();
                let query = target.split_once('?').map(|x| x.1.to_string()).unwrap_or_default();
                let params = parse_query(&query);
                if !params.contains_key("code") && !params.contains_key("error") {
                    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"); continue;
                }
                let ok = params.get("state").map(|x| x == state).unwrap_or(false) && params.contains_key("code");
                let page = if ok { "<h2>Bullo est connect&eacute; &agrave; Google.</h2><p>Tu peux fermer cet onglet et revenir dans Bullo.</p>" } else { "<h2>La connexion n'a pas abouti.</h2><p>Reviens dans Bullo et r&eacute;essaie.</p>" };
                let html = format!("<!doctype html><meta charset=utf-8><title>Bullo</title><body style=\"font-family:system-ui;text-align:center;margin-top:20vh\">{page}</body>");
                let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", html.len(), html);
                if let Some(err) = params.get("error") { return Err(if err == "access_denied" { "Tu as refusé l'accès dans Google. Recommence en cliquant sur « Autoriser ».".to_string() } else { format!("Google a refusé la connexion ({err}).") }); }
                if !ok { return Err("Réponse de Google inattendue (sécurité : état invalide). Réessaie.".into()); }
                return Ok(params["code"].clone());
            }
            Err(ref er) if er.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(200)),
            Err(er) => return Err(e(er)),
        }
    }
}

#[derive(Serialize)]
struct GoogleStatus { has_keys: bool, connected: bool, email: String, fingerprint: String, intact: bool }

#[tauri::command]
fn google_status() -> GoogleStatus {
    let (id, secret) = (kc_get(KC_ID), kc_get(KC_SECRET));
    let has_keys = id.is_some() && secret.is_some();
    let stored = kc_get(KC_FP).unwrap_or_default();
    let now = match (&id, &secret) { (Some(i), Some(sc)) => key_fingerprint(i, sc), _ => String::new() };
    GoogleStatus { has_keys, connected: kc_get(KC_REFRESH).is_some(), email: kc_get(KC_EMAIL).unwrap_or_default(), intact: has_keys && (stored.is_empty() || stored == now), fingerprint: now }
}

#[tauri::command]
async fn google_save_keys(client_id: String, client_secret: String) -> Result<(), String> {
    let (id, secret) = (client_id.trim().to_string(), client_secret.trim().to_string());
    if !id.ends_with(".apps.googleusercontent.com") { return Err("L'ID client doit se terminer par « .apps.googleusercontent.com ». Copie-le en entier depuis Google (étape 4).".into()); }
    if secret.len() < 10 { return Err("Le code secret client semble trop court. Copie-le en entier depuis Google (étape 4).".into()); }
    tauri::async_runtime::spawn_blocking(move || { kc_set(KC_ID, &id)?; kc_set(KC_SECRET, &secret)?; kc_set(KC_FP, &key_fingerprint(&id, &secret)) }).await.map_err(e)?
}

#[tauri::command]
async fn google_clear_keys() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| { google_disconnect_blocking(); kc_del(KC_ID); kc_del(KC_SECRET); kc_del(KC_FP); Ok::<(), String>(()) }).await.map_err(e)?
}

fn google_disconnect_blocking() {
    if let Some(rt) = kc_get(KC_REFRESH) { let _ = ureq::post("https://oauth2.googleapis.com/revoke").send_form(&[("token", rt.as_str())]); }
    kc_del(KC_REFRESH); kc_del(KC_EMAIL);
}

#[tauri::command]
async fn google_disconnect() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| { google_disconnect_blocking(); Ok::<(), String>(()) }).await.map_err(e)?
}

#[tauri::command]
fn google_cancel() { GOOGLE_CANCEL.store(true, Ordering::SeqCst); }

#[tauri::command]
async fn google_connect() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        GOOGLE_CANCEL.store(false, Ordering::SeqCst);
        let (cid, secret) = match (kc_get(KC_ID), kc_get(KC_SECRET)) { (Some(a), Some(b)) => (a, b), _ => return Err("Les clés Google ne sont pas encore enregistrées (étape 5 du tutoriel).".to_string()) };
        if let Some(fp) = kc_get(KC_FP) { if fp != key_fingerprint(&cid, &secret) { return Err("Contrôle d'intégrité SHA-256 : tes clés Google ont été modifiées ou corrompues dans le Trousseau. Efface-les puis refais l'étape 5.".to_string()); } }
        let verifier = b64url(&rand_bytes(48)?); let state = b64url(&rand_bytes(24)?);
        let challenge = b64url(&Sha256::digest(verifier.as_bytes()));
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|_| "Impossible d'ouvrir le port local pour la connexion.".to_string())?;
        let redirect = format!("http://127.0.0.1:{}", listener.local_addr().map_err(e)?.port());
        let url = format!("https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&state={}&access_type=offline&prompt=consent",
            urlenc(&cid), urlenc(&redirect), urlenc(DRIVE_SCOPE), challenge, state);
        #[cfg(target_os = "macos")]
        Command::new("open").arg(&url).spawn().map_err(|_| "Impossible d'ouvrir le navigateur.".to_string())?;
        #[cfg(not(target_os = "macos"))]
        return Err(format!("Ouvre ce lien dans ton navigateur : {url}"));
        let code = wait_for_code(&listener, &state)?;
        let r: serde_json::Value = ureq::post("https://oauth2.googleapis.com/token")
            .send_form(&[("code", code.as_str()), ("client_id", cid.as_str()), ("client_secret", secret.as_str()), ("code_verifier", verifier.as_str()), ("redirect_uri", redirect.as_str()), ("grant_type", "authorization_code")])
            .map_err(gerr)?.into_json().map_err(e)?;
        let refresh = r["refresh_token"].as_str().ok_or("Google n'a pas fourni d'autorisation durable. Retire l'accès de Bullo dans ton compte Google (myaccount.google.com/permissions) puis réessaie.")?.to_string();
        kc_set(KC_REFRESH, &refresh)?;
        let email = r["access_token"].as_str().and_then(|t| ureq::get("https://www.googleapis.com/drive/v3/about").query("fields", "user(emailAddress)").set("Authorization", &format!("Bearer {t}")).call().ok())
            .and_then(|x| x.into_json::<serde_json::Value>().ok()).and_then(|v| v["user"]["emailAddress"].as_str().map(String::from)).unwrap_or_default();
        if !email.is_empty() { let _ = kc_set(KC_EMAIL, &email); }
        Ok::<String, String>(email)
    }).await.map_err(e)?
}

fn google_token() -> Result<String, String> {
    let (cid, secret, refresh) = match (kc_get(KC_ID), kc_get(KC_SECRET), kc_get(KC_REFRESH)) {
        (Some(a), Some(b), Some(c)) => (a, b, c), _ => return Err("Tu n'es pas connecté à Google. Va dans Réglages > Google.".into()) };
    match ureq::post("https://oauth2.googleapis.com/token").send_form(&[("client_id", cid.as_str()), ("client_secret", secret.as_str()), ("refresh_token", refresh.as_str()), ("grant_type", "refresh_token")]) {
        Ok(r) => r.into_json::<serde_json::Value>().map_err(e)?["access_token"].as_str().map(String::from).ok_or_else(|| "Google n'a pas renvoyé d'accès. Reconnecte-toi.".to_string()),
        Err(ureq::Error::Status(_, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            if body.contains("invalid_grant") { kc_del(KC_REFRESH); return Err("Ta connexion Google a expiré ou a été retirée. Reconnecte-toi dans Réglages > Google.".into()); }
            if body.contains("invalid_client") { return Err("Google ne reconnaît plus ton ID client. Refais l'étape 4 du tutoriel.".into()); }
            Err(format!("Google a refusé le renouvellement de l'accès. {}", body.chars().take(120).collect::<String>()))
        }
        Err(er) => Err(gerr(er)),
    }
}

fn drive_find(tok: &str, q: &str) -> Result<Option<String>, String> {
    let r = ureq::get("https://www.googleapis.com/drive/v3/files").set("Authorization", &format!("Bearer {tok}"))
        .query("q", q).query("fields", "files(id,name)").query("spaces", "drive").query("pageSize", "1").call().map_err(gerr)?;
    let v: serde_json::Value = r.into_json().map_err(e)?;
    Ok(v["files"][0]["id"].as_str().map(String::from))
}
fn drive_folder(tok: &str) -> Result<String, String> {
    if let Some(id) = drive_find(tok, &format!("mimeType='application/vnd.google-apps.folder' and name='{DRIVE_FOLDER}' and trashed=false"))? { return Ok(id); }
    let v: serde_json::Value = ureq::post("https://www.googleapis.com/drive/v3/files").set("Authorization", &format!("Bearer {tok}"))
        .send_json(serde_json::json!({ "name": DRIVE_FOLDER, "mimeType": "application/vnd.google-apps.folder" })).map_err(gerr)?.into_json().map_err(e)?;
    v["id"].as_str().map(String::from).ok_or_else(|| "Google n'a pas créé le dossier Bullo.".to_string())
}

#[derive(Serialize, Deserialize)]
struct FullBackup { items: Vec<Item>, #[serde(default)] reminders: Vec<Reminder> }

fn reminders_all(db: &Db) -> Result<Vec<Reminder>, String> {
    let c = db.0.lock().map_err(e)?;
    let mut s = c.prepare("SELECT id,title,due,done,notified FROM reminders ORDER BY due").map_err(e)?;
    let rows = s.query_map([], |r| Ok(Reminder { id: r.get(0)?, title: r.get(1)?, due: r.get(2)?, done: r.get::<_, i64>(3)? != 0, notified: r.get::<_, i64>(4)? != 0 })).map_err(e)?;
    let out = rows.collect::<Result<Vec<_>, _>>().map_err(e)?; Ok(out)
}
fn full_backup_envelope(db: &Db) -> Result<(String, usize, usize), String> {
    let items = query(db, format!("SELECT {COLS} FROM items ORDER BY id"), vec![])?;
    let reminders = reminders_all(db)?;
    let (ni, nr) = (items.len(), reminders.len());
    let content = serde_json::to_string(&FullBackup { items, reminders }).map_err(e)?;
    let env = BackupEnvelope { version: 2, content_sha256: backup_hash(&content), content };
    Ok((serde_json::to_string(&env).map_err(e)?, ni, nr))
}
fn restore_all(db: &Db, items: &[Item], reminders: &[Reminder]) -> Result<(), String> {
    let mut c = db.0.lock().map_err(e)?; let tx = c.transaction().map_err(e)?;
    tx.execute("DELETE FROM items", []).map_err(e)?;
    for i in items { tx.execute("INSERT INTO items(id,kind,title,body,done,created,course,data,audio,parent) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![i.id,i.kind,i.title,i.body,i.done as i64,i.created,i.course,i.data,i.audio,i.parent]).map_err(e)?; }
    tx.execute("DELETE FROM reminders", []).map_err(e)?;
    for r in reminders { tx.execute("INSERT INTO reminders(id,title,due,done,notified) VALUES(?1,?2,?3,?4,?5)", params![r.id, r.title, r.due, r.done as i64, r.notified as i64]).map_err(e)?; }
    tx.commit().map_err(e)
}

#[tauri::command]
async fn google_backup(app: AppHandle) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let tok = google_token()?;
        let (body_json, ni, nr) = full_backup_envelope(&app.state::<Db>())?;
        let folder = drive_folder(&tok)?;
        let existing = drive_find(&tok, &format!("name='{DRIVE_FILE}' and '{folder}' in parents and trashed=false"))?;
        let b = "bullo_boundary_7f3a9c21";
        let meta = if existing.is_some() { serde_json::json!({ "name": DRIVE_FILE }) } else { serde_json::json!({ "name": DRIVE_FILE, "parents": [folder] }) };
        let body = format!("--{b}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{meta}\r\n--{b}\r\nContent-Type: application/json\r\n\r\n{body_json}\r\n--{b}--");
        let (method, url) = match &existing {
            Some(id) => ("PATCH", format!("https://www.googleapis.com/upload/drive/v3/files/{id}?uploadType=multipart")),
            None => ("POST", "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart".to_string()),
        };
        ureq::request(method, &url).set("Authorization", &format!("Bearer {tok}")).set("Content-Type", &format!("multipart/related; boundary={b}"))
            .send_bytes(body.as_bytes()).map_err(gerr)?;
        Ok::<_, String>(serde_json::json!({ "items": ni, "reminders": nr, "when": now_secs() }))
    }).await.map_err(e)?
}

#[tauri::command]
async fn google_restore(app: AppHandle) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let tok = google_token()?;
        let folder = drive_find(&tok, &format!("mimeType='application/vnd.google-apps.folder' and name='{DRIVE_FOLDER}' and trashed=false"))?
            .ok_or("Aucune sauvegarde trouvée sur ton Drive. Fais d'abord « Sauvegarder maintenant ».")?;
        let id = drive_find(&tok, &format!("name='{DRIVE_FILE}' and '{folder}' in parents and trashed=false"))?
            .ok_or("Aucune sauvegarde trouvée dans le dossier Bullo de ton Drive.")?;
        let raw = ureq::get(&format!("https://www.googleapis.com/drive/v3/files/{id}")).query("alt", "media").set("Authorization", &format!("Bearer {tok}"))
            .call().map_err(gerr)?.into_string().map_err(e)?;
        let env: BackupEnvelope = serde_json::from_str(&raw).map_err(|_| "La sauvegarde Google est illisible ou corrompue.".to_string())?;
        if backup_hash(&env.content) != env.content_sha256 { return Err("Sauvegarde refusée : le fichier a été modifié ou est corrompu (contrôle d'intégrité SHA-256).".into()); }
        let (items, reminders) = match serde_json::from_str::<FullBackup>(&env.content) {
            Ok(f) => (f.items, f.reminders),
            Err(_) => (serde_json::from_str::<Vec<Item>>(&env.content).map_err(|_| "Contenu de sauvegarde invalide.".to_string())?, vec![]),
        };
        let db = app.state::<Db>();
        // Filet de sécurité : l'état actuel est copié avant d'être remplacé.
        let dir = app.path().app_data_dir().map_err(e)?;
        let (current, _, _) = full_backup_envelope(&db)?;
        let safety = dir.join(format!("avant-restauration-{}.json", now_secs()));
        fs::write(&safety, current).map_err(e)?;
        restore_all(&db, &items, &reminders)?;
        Ok::<_, String>(serde_json::json!({ "items": items.len(), "reminders": reminders.len(), "safety": safety.to_string_lossy() }))
    }).await.map_err(e)?
}

// Ouvre uniquement des pages Google précises (liste blanche) dans le navigateur.
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    const OK: [&str; 3] = ["https://console.cloud.google.com/", "https://myaccount.google.com/", "https://drive.google.com/"];
    if !OK.iter().any(|p| url.starts_with(p)) { return Err("Lien non autorisé.".into()); }
    Command::new("open").arg(url).spawn().map(|_| ()).map_err(e)
}

// Dossier des enregistrements : choisi dans Réglages, mémorisé dans audio_dir.txt.
fn audio_dir_of(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let base = app.path().app_data_dir().map_err(e)?;
    Ok(match fs::read_to_string(base.join("audio_dir.txt")) { Ok(s) if !s.trim().is_empty() => s.trim().into(), _ => base.join("audio") })
}
#[tauri::command]
fn get_audio_dir(app: AppHandle) -> Result<String, String> { Ok(audio_dir_of(&app)?.to_string_lossy().into_owned()) }
#[tauri::command]
fn set_audio_dir(app: AppHandle, path: String) -> Result<(), String> {
    let base = app.path().app_data_dir().map_err(e)?;
    fs::create_dir_all(&base).map_err(e)?;
    fs::write(base.join("audio_dir.txt"), path).map_err(e)
}
#[tauri::command]
fn open_path(path: String) -> Result<(), String> { Command::new("open").arg(path).spawn().map(|_| ()).map_err(e) }

// Sélecteur de fichier/dossier natif macOS via osascript (aucune dépendance en plus).
#[tauri::command]
async fn pick(kind: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let script = if kind == "folder" { "POSIX path of (choose folder)" } else if kind == "save" { "POSIX path of (choose file name default name \"bullo-backup.json\")" } else { "POSIX path of (choose file)" };
        let o = Command::new("osascript").args(["-e", script]).output().map_err(e)?;
        if !o.status.success() { return Err("Annulé.".to_string()); }
        Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
    }).await.map_err(e)?
}

#[derive(Serialize)]
struct DependencyStatus { id: String, name: String, ok: bool, install: bool, hint: String }

// Tesseract est considéré prêt seulement si la langue française est disponible.
fn tesseract_ok() -> bool {
    Command::new("tesseract").env("PATH", PATH_ENV).arg("--list-langs").output()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).lines().any(|l| l.trim() == "fra")).unwrap_or(false)
}


fn ollama_tags() -> Option<serde_json::Value> {
    ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(2)).build()
        .get("http://localhost:11434/api/tags").call().ok()?.into_json().ok()
}
fn ollama_has_model() -> bool {
    let want = ollama_model();
    let base = want.split(':').next().unwrap_or("").to_string();
    ollama_tags().and_then(|t| t["models"].as_array().map(|a| a.iter().any(|m| {
        let n = m["name"].as_str().unwrap_or("");
        n == want || (!want.contains(':') && n.split(':').next() == Some(base.as_str()))
    }))).unwrap_or(false)
}

// Téléchargement d'un gros fichier vers un fichier temporaire, puis renommage (pas de fichier à moitié téléchargé).
fn download_to(url: &str, dest: &str) -> Result<(), String> {
    let dest = std::path::PathBuf::from(dest);
    if let Some(d) = dest.parent() { fs::create_dir_all(d).map_err(e)?; }
    let tmp = dest.with_extension("part");
    let resp = ureq::AgentBuilder::new().timeout_connect(std::time::Duration::from_secs(15)).timeout_read(std::time::Duration::from_secs(60)).build()
        .get(url).call().map_err(|_| "Téléchargement impossible : vérifie ta connexion internet puis réessaie.".to_string())?;
    let mut f = fs::File::create(&tmp).map_err(e)?;
    let r = std::io::copy(&mut resp.into_reader(), &mut f);
    drop(f);
    if let Err(er) = r { let _ = fs::remove_file(&tmp); return Err(format!("Téléchargement interrompu : {er}")); }
    fs::rename(&tmp, &dest).map_err(e)
}

#[derive(Serialize)]
struct UpdateInfo { current: String, latest: String, newer: bool, url: String, notes: String }

fn ver_parts(v: &str) -> Vec<u64> { v.trim().trim_start_matches('v').split('.').map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0)).collect() }

// Interroge GitHub (liste publique des versions). Aucune donnée n'est envoyée, à part la requête elle-même.
#[tauri::command]
async fn check_update(current: String) -> Result<UpdateInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let r: serde_json::Value = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(8)).build()
            .get("https://api.github.com/repos/RenzVASA/bullo/releases/latest")
            .set("User-Agent", "Bullo-update-check").set("Accept", "application/vnd.github+json").call()
            .map_err(|_| "Impossible de joindre GitHub (pas d'internet ?).".to_string())?
            .into_json().map_err(e)?;
        let latest = r["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
        if latest.is_empty() { return Err("Aucune version publiée pour l'instant.".into()); }
        let (a, b) = (ver_parts(&latest), ver_parts(&current));
        let n = a.len().max(b.len());
        let pad = |v: &Vec<u64>| { let mut v = v.clone(); v.resize(n, 0); v };
        let newer = pad(&a) > pad(&b);
        let url = r["html_url"].as_str().filter(|u| u.starts_with("https://github.com/RenzVASA/bullo/")).unwrap_or("https://github.com/RenzVASA/bullo/releases/latest").to_string();
        let notes: String = r["body"].as_str().unwrap_or("").chars().take(500).collect();
        Ok(UpdateInfo { current, latest, newer, url, notes })
    }).await.map_err(e)?
}

// ---- Mise à jour automatique : télécharge le .dmg de la dernière version, vérifie son empreinte SHA-256 (publiée dans la Release),
// puis un petit script remplace Bullo.app une fois Bullo fermé, retire la quarantaine et rouvre l'application.
// En cas de souci, l'ancienne version est remise en place et le .dmg est ouvert pour une installation à la main. ----
const UPDATE_SCRIPT: &str = r#"#!/bin/sh
PID="$1"; DMG="$2"; APP="$3"; DIR="$(dirname "$DMG")"
i=0; while kill -0 "$PID" 2>/dev/null && [ $i -lt 60 ]; do sleep 0.5; i=$((i+1)); done
MNT="$(mktemp -d /tmp/bullo-maj.XXXXXX)"
if ! hdiutil attach -nobrowse -readonly -mountpoint "$MNT" "$DMG" >/dev/null 2>&1; then open "$DMG"; open "$APP"; exit 1; fi
SRC="$(ls -d "$MNT"/*.app 2>/dev/null | head -1)"
OK=0
if [ -n "$SRC" ] && mv "$APP" "$APP.old" 2>/dev/null; then
  if ditto "$SRC" "$APP" 2>/dev/null && xattr -cr "$APP" 2>/dev/null; then OK=1; rm -rf "$APP.old"; else rm -rf "$APP"; mv "$APP.old" "$APP"; fi
fi
hdiutil detach "$MNT" -quiet >/dev/null 2>&1
if [ "$OK" = 1 ]; then rm -f "$DMG"; else open "$DMG"; fi
open "$APP"
"#;

#[tauri::command]
async fn install_update(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        use std::os::unix::process::CommandExt;
        let exe = std::env::current_exe().map_err(e)?;
        let app_path = exe.ancestors().find(|p| p.extension().map(|x| x == "app").unwrap_or(false)).map(|p| p.to_path_buf())
            .ok_or_else(|| "La mise à jour automatique ne marche que dans l'application installée (pas en mode développement).".to_string())?;
        let r: serde_json::Value = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(10)).build()
            .get("https://api.github.com/repos/RenzVASA/bullo/releases/latest")
            .set("User-Agent", "Bullo-update").set("Accept", "application/vnd.github+json").call()
            .map_err(|_| "Impossible de joindre GitHub (pas d'internet ?).".to_string())?
            .into_json().map_err(e)?;
        let prefix = "https://github.com/RenzVASA/bullo/releases/download/";
        let url = r["assets"].as_array().and_then(|a| a.iter().filter_map(|x| x["browser_download_url"].as_str())
            .find(|u| u.starts_with(prefix) && u.ends_with(".dmg"))).ok_or("Cette version ne contient pas de fichier .dmg à installer.")?.to_string();
        let body = r["body"].as_str().unwrap_or("");
        let want: String = body.split("SHA-256 du .dmg : `").nth(1).map(|t| t.chars().take_while(|c| c.is_ascii_hexdigit()).collect()).unwrap_or_default();
        if want.len() != 64 { return Err("L'empreinte de sécurité (SHA-256) de cette version est introuvable : par prudence, rien n'a été installé. Utilise « Voir la mise à jour » pour la télécharger à la main.".into()); }
        let home = std::env::var("HOME").map_err(e)?;
        let dir = format!("{home}/.bullo/update");
        let dmg = format!("{dir}/Bullo.dmg");
        download_to(&url, &dmg)?;
        let mut h = Sha256::new();
        h.update(fs::read(&dmg).map_err(e)?);
        let got = format!("{:x}", h.finalize());
        if got != want { let _ = fs::remove_file(&dmg); return Err("Le fichier téléchargé ne correspond pas à l'empreinte publiée : il a été supprimé et rien n'a été installé.".into()); }
        let script = format!("{dir}/apply.sh");
        fs::write(&script, UPDATE_SCRIPT).map_err(e)?;
        Command::new("sh").arg(&script).arg(std::process::id().to_string()).arg(&dmg).arg(&app_path)
            .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .process_group(0).spawn().map_err(|_| "Impossible de lancer l'installation.".to_string())?;
        Ok(())
    }).await.map_err(e)??;
    // Le script attend que Bullo soit fermé, remplace l'application puis la rouvre.
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn check_installation() -> Result<Vec<DependencyStatus>, String> {
    let has = |cmd: &str| Command::new("sh").env("PATH", PATH_ENV).args(["-c", &format!("command -v {cmd}")]).output().map(|x| x.status.success()).unwrap_or(false);
    let model = whisper_model();
    let oll = ollama_tags().is_some();
    let oll_model = oll && ollama_has_model();
    Ok(vec![
        DependencyStatus{id:"poppler".into(),name:"Poppler / pdftotext (PDF)".into(),ok:has("pdftotext"),install:true,hint:String::new()},
        DependencyStatus{id:"textutil".into(),name:"textutil (Word .docx, macOS)".into(),ok:has("textutil"),install:false,hint:String::new()},
        DependencyStatus{id:"tesseract".into(),name:"Tesseract + français (texte des images)".into(),ok:tesseract_ok(),install:true,hint:String::new()},
        DependencyStatus{id:"ffmpeg".into(),name:"FFmpeg (audio)".into(),ok:has("ffmpeg"),install:true,hint:String::new()},
        DependencyStatus{id:"whisper".into(),name:"whisper-cli (transcription locale)".into(),ok:has("whisper-cli"),install:true,hint:String::new()},
        DependencyStatus{id:"whisper-model".into(),name:"Modèle de transcription Whisper (≈ 470 Mo)".into(),ok:std::path::Path::new(&model).exists(),install:true,hint:"Téléchargement unique, puis tout marche hors ligne.".into()},
        DependencyStatus{id:"ollama".into(),name:"Ollama (le résumé et les outils IA)".into(),ok:oll,install:false,hint:"Télécharge Ollama sur ollama.com, installe-le et ouvre-le une fois.".into()},
        DependencyStatus{id:"ollama-model".into(),name:format!("Modèle de résumé ({})", ollama_model()),ok:oll_model,install:oll,hint:"Téléchargement unique (≈ 4,7 Go). Ollama doit être ouvert.".into()},
    ])
}

#[tauri::command]
async fn install_dependency(id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || install_dependency_blocking(id)).await.map_err(e)?
}

fn install_dependency_blocking(id: String) -> Result<String, String> {
    if id == "whisper-model" {
        download_to("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin", &whisper_model())?;
        return Ok("Modèle Whisper installé.".into());
    }
    if id == "ollama-model" {
        let model = ollama_model();
        let r = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(3600)).build()
            .post("http://localhost:11434/api/pull").send_json(serde_json::json!({ "model": model, "stream": false }))
            .map_err(|_| "Le téléchargement a échoué. Vérifie qu'Ollama est ouvert et que tu as internet.".to_string())?;
        let v: serde_json::Value = r.into_json().map_err(e)?;
        if let Some(er) = v["error"].as_str() { return Err(er.to_string()); }
        return Ok("Modèle de résumé installé.".into());
    }
    let (program,args): (&str, Vec<&str>) = match id.as_str() {
        "poppler" => ("brew", vec!["install", "poppler"]),
        "tesseract" => ("brew", vec!["install", "tesseract", "tesseract-lang"]),
        "ffmpeg" => ("brew", vec!["install", "ffmpeg"]),
        "whisper" => ("brew", vec!["install", "whisper-cpp"]),
        _ => return Err("Installation automatique indisponible pour cet élément. Consulte le README.".into()),
    };
    let out=Command::new(program).env("PATH", PATH_ENV).args(args).output()
        .map_err(|_| "Homebrew est introuvable. Installe-le d'abord depuis https://brew.sh, puis réessaie.".to_string())?;
    if out.status.success(){Ok("Installation terminée.".into())}else{Err(String::from_utf8_lossy(&out.stderr).trim().to_string())}
}

// Texte depuis un fichier (consignes Classroom téléchargées, captures d'écran…) sans aucune API Google.
#[tauri::command]
async fn import_text(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        let run = |cmd: &str, args: &[&str]| Command::new(cmd).env("PATH", PATH_ENV).args(args).output()
            .map_err(|_| match cmd {
                "tesseract" => "[[install:tesseract]]Pour lire le texte d'une image, Bullo a besoin de Tesseract (reconnaissance de texte).".to_string(),
                "pdftotext" => "[[install:poppler]]Pour lire un PDF, Bullo a besoin de Poppler.".to_string(),
                _ => format!("{cmd} introuvable."),
            });
        let out = match ext.as_str() {
            "txt" | "md" => return fs::read_to_string(&path).map_err(e),
            "docx" | "rtf" | "html" => run("textutil", &["-convert", "txt", "-stdout", &path])?,
            "pdf" => run("pdftotext", &["-layout", &path, "-"])?,
            "png" | "jpg" | "jpeg" => run("tesseract", &[&path, "-", "-l", "fra"])?,
            _ => return Err("Format non pris en charge (txt, md, docx, pdf, png, jpg).".to_string()),
        };
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            if ext == "png" || ext == "jpg" || ext == "jpeg" {
                if err.contains("Failed loading language") || err.contains("Error opening data file") {
                    return Err("[[install:tesseract]]Tesseract est installé, mais sans le français.".to_string());
                }
            }
            return Err(format!("Lecture du fichier impossible. {}", err.lines().next().unwrap_or("")).trim().to_string());
        }
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if text.is_empty() { return Err("Aucun texte reconnu dans ce fichier (image trop floue, ou PDF scanné sans texte).".to_string()); }
        Ok(text)
    }).await.map_err(e)?
}

// Range la fiche du cours (Markdown) et l'audio dans <dossier des enregistrements>/<Cours>/.
#[tauri::command]
fn save_course(app: AppHandle, course: String, title: String, md: String, audio: Option<String>) -> Result<serde_json::Value, String> {
    let name: String = course.chars().map(|c| if "/\\:*?\"<>|".contains(c) { '-' } else { c }).collect();
    let name = name.trim().trim_start_matches('.').to_string();
    if name.is_empty() { return Err("Nom de cours vide.".into()); }
    let dir = audio_dir_of(&app)?.join(&name);
    fs::create_dir_all(&dir).map_err(e)?;
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map_err(e)?.as_secs();
    fs::write(dir.join(format!("cours-{ts}.md")), format!("# {title}\n\n{md}")).map_err(e)?;
    let mut final_audio: Option<String> = None;
    if let Some(a) = audio {
        let a = std::path::PathBuf::from(a);
        if let Some(f) = a.file_name() {
            let to = dir.join(f);
            if fs::rename(&a, &to).is_err() && fs::copy(&a, &to).is_ok() { let _ = fs::remove_file(&a); }
            let kept = if to.exists() { to } else { a };
            final_audio = Some(kept.to_string_lossy().into_owned());
        }
    }
    Ok(serde_json::json!({ "dir": dir.to_string_lossy(), "audio": final_audio }))
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            fs::create_dir_all(&dir)?;
            // Migration : l'identifiant de l'appli a changé (app.focusflow.desktop -> app.bullo.desktop), donc le dossier de données aussi.
            // On copie l'ancienne base et le choix du dossier audio ; les anciens enregistrements restent lisibles (chemins absolus).
            if let Some(parent) = dir.parent() {
                let old_dir = parent.join("app.focusflow.desktop");
                for (from, to) in [("focusflow.db", "bullo.db"), ("audio_dir.txt", "audio_dir.txt")] {
                    let (src, dst) = (old_dir.join(from), dir.join(to));
                    if src.exists() && !dst.exists() { let _ = fs::copy(&src, &dst); }
                }
            }
            // Migration : l'ancienne base (focusflow.db) est copiée vers bullo.db au premier lancement ; l'ancienne reste en filet de sécurité.
            let db_path = dir.join("bullo.db");
            let old_db = dir.join("focusflow.db");
            if !db_path.exists() && old_db.exists() { let _ = fs::copy(&old_db, &db_path); }
            let conn = Connection::open(&db_path)?;
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS items(
                   id INTEGER PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL,
                   body TEXT NOT NULL DEFAULT '', done INTEGER NOT NULL DEFAULT 0,
                   created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);")?;
            // Migration douce : ajoute les colonnes manquantes sans toucher aux anciennes données.
            for col in ["course TEXT", "data TEXT", "audio TEXT", "parent INTEGER"] {
                let _ = conn.execute(&format!("ALTER TABLE items ADD COLUMN {col}"), []);
            }
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS reminders(
                   id INTEGER PRIMARY KEY, title TEXT NOT NULL, due INTEGER NOT NULL,
                   done INTEGER NOT NULL DEFAULT 0, notified INTEGER NOT NULL DEFAULT 0);")?;
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS explanations(
                   id INTEGER PRIMARY KEY, created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                   topic TEXT NOT NULL DEFAULT '', explanation TEXT NOT NULL DEFAULT '',
                   turns TEXT NOT NULL DEFAULT '[]', bilan TEXT NOT NULL DEFAULT '');")?;
            app.manage(Db(Mutex::new(conn)));
            start_reminder_watcher(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![player_diagnostic, media_key_test, now_playing, open_web, system_audio, system_mute, system_volume, add_item, add_course, read_audio, save_download, google_status, google_save_keys, google_clear_keys, google_connect, google_cancel, google_disconnect, google_backup, google_restore, open_url, install_fonts, list_fonts, read_font, notify, media_status, media_control, media_volume, media_open, add_reminder, list_reminders, update_reminder, delete_reminder, list_items, search, set_done, delete_item, save_audio, transcribe, summarize, assist, explain_turn, explain_save, explain_list, explain_delete, export_md, pick, open_path, set_audio_dir, get_audio_dir, check_installation, check_update, install_update, install_dependency, import_text, save_course, export_backup, import_backup])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de Bullo");
}
