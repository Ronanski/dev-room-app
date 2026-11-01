//! Dev Room local server.
//!
//! Claude Code hooks POST their JSON payload to `/hook` (loopback only).
//! The UI (desktop window or a phone on the same Wi-Fi) polls `/api/state`.
//! Phones must present the pairing token; loopback callers never need it.

use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

pub const DEFAULT_PORT: u16 = 4317;
const INDEX_HTML: &str = include_str!("../../ui/index.html");
const HOOK_EVENTS: [&str; 8] = ["SessionStart", "UserPromptSubmit", "PreToolUse", "PostToolUse", "SubagentStop", "Notification", "Stop", "SessionEnd"];
const MAX_BODY: usize = 32 * 1024 * 1024;   // a Write tool call can carry a whole file

/// Set by the app: opens the Dev Room window (used when DevRoom.exe is started a second time).
pub static ON_SHOW: std::sync::OnceLock<Box<dyn Fn() + Send + Sync>> = std::sync::OnceLock::new();
/// A per-launch key the app puts in its own window's URL, so only that window drives the embedded Claude view.
pub fn app_key() -> &'static str {
    static K: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    K.get_or_init(|| format!("{:016x}", rand::random::<u64>()))
}
/// Set by the app: shows the official Claude Code chat inside the Dev Room window at a rectangle
/// (CSS pixels of the UI), or hides it (None).
pub static ON_CLAUDE: std::sync::OnceLock<Box<dyn Fn(Option<(f64, f64, f64, f64)>) + Send + Sync>> = std::sync::OnceLock::new();

/// Lock the shared state; a panic in another request must not take the whole server down.
fn lk(m: &Mutex<State>) -> std::sync::MutexGuard<'_, State> { m.lock().unwrap_or_else(|e| e.into_inner()) }

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn clip(s: &str, n: usize) -> String {
    let t: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() <= n { t } else { format!("{}…", t.chars().take(n).collect::<String>().trim_end()) }
}

fn base_name(p: &str) -> String {
    p.trim_end_matches(['/', '\\']).rsplit(['/', '\\']).next().unwrap_or(p).to_string()
}

#[derive(Default)]
struct State {
    sessions: Map<String, Value>,
    team: Value,
    version: u64,
    token: String,
    dir: PathBuf,
    /// "off" (default: a finished turn ends normally), "auto" (wait while a phone has Dev Room open) or "on".
    away: String,
    /// Last time a phone or tablet had the office on screen (background watchers do not count).
    remote_seen: u64,
    /// "Malayo mode": the paired phone can reach Dev Room from anywhere through a tunnel (on by default).
    remote_off: bool,
}

impl State {
    fn bump(&mut self) { self.version += 1; }

    fn save(&self) {
        let _ = fs::create_dir_all(&self.dir);
        let data = json!({"sessions": self.sessions, "team": self.team, "token": self.token, "away": self.away, "away_v": 2, "remote_off": self.remote_off});
        // write then rename, so a crash never leaves a half-written file (which would unpair every phone)
        let tmp = self.dir.join("state.json.tmp");
        if fs::write(&tmp, serde_json::to_vec(&data).unwrap_or_default()).is_ok() { let _ = fs::rename(&tmp, self.dir.join("state.json")); }
    }

    fn load(dir: PathBuf) -> State {
        let mut st = State { dir, team: json!({"lead": null, "ids": [], "names": {}}), ..Default::default() };
        if let Ok(bytes) = fs::read(st.dir.join("state.json")) {
            if let Ok(v) = serde_json::from_slice::<Value>(&bytes) {
                if let Some(m) = v.get("sessions").and_then(|s| s.as_object()) {
                    let cutoff = now_ms().saturating_sub(3 * 24 * 3600 * 1000);
                    for (k, s) in m {
                        if s.get("updated_at").and_then(|t| t.as_u64()).unwrap_or(0) > cutoff {
                            st.sessions.insert(k.clone(), s.clone());
                        }
                    }
                }
                if let Some(t) = v.get("team") { if t.is_object() { st.team = t.clone(); } }
                if let Some(t) = v.get("token").and_then(|t| t.as_str()) { st.token = t.to_string(); }
                st.remote_off = v.get("remote_off").and_then(|x| x.as_bool()).unwrap_or(false);
                // 0.6.0-0.6.2 defaulted to "auto"; a held Stop hook looks like a session that never answers, so start from "off"
                if v.get("away_v").and_then(|x| x.as_u64()).unwrap_or(0) >= 2 {
                    if let Some(a) = v.get("away").and_then(|t| t.as_str()) { st.away = a.to_string(); }
                }
            }
        }
        if st.token.len() < 16 { st.token = format!("{:016x}{:016x}", rand::random::<u64>(), rand::random::<u64>()); }
        if !["auto", "on", "off"].contains(&st.away.as_str()) { st.away = "off".into(); }
        for s in st.sessions.values_mut() { if let Some(o) = s.as_object_mut() { o.insert("listening".into(), json!(false)); } }
        st.version = 1;
        st.save();
        st
    }
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DevRoom")
}

pub fn claude_settings_path() -> PathBuf {
    std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
        .join(".claude").join("settings.json")
}

/// Seconds a finished session may wait for a message from the phone (the Stop hook holds the turn open).
const LISTEN_SECS: u64 = 1200;

fn hook_command(port: u16, ev: &str) -> String {
    // curl.exe ships with Windows 10 and 11; `|| exit 0` keeps Claude quiet when Dev Room is closed.
    let max = if ev == "Stop" { LISTEN_SECS + 50 } else { 2 };
    format!("curl -s --noproxy 127.0.0.1 -m {max} -X POST -H \"Content-Type: application/json\" --data-binary @- http://127.0.0.1:{port}/hook?devroom=1 || exit 0")
}
fn hook_timeout(ev: &str) -> u64 { if ev == "Stop" { LISTEN_SECS + 60 } else { 3 } }

/// A Dev Room hook command: today's (marked with `/hook?devroom=1`) or an older one (curl posting stdin to /hook).
fn is_our_cmd(h: &Value) -> bool {
    h.get("command").and_then(|c| c.as_str()).map(|c| c.contains("/hook?devroom=1")
        || (c.contains("--data-binary @- http://127.0.0.1:") && c.trim_end().ends_with("/hook || exit 0"))).unwrap_or(false)
}
fn is_ours(entry: &Value) -> bool {
    entry.get("hooks").and_then(|h| h.as_array()).map(|hs| hs.iter().any(is_our_cmd)).unwrap_or(false)
}

/// True when settings.json already holds at least one Dev Room hook.
fn has_any_ours() -> bool {
    let Ok(text) = fs::read_to_string(claude_settings_path()) else { return false };
    let Ok(v) = serde_json::from_str::<Value>(&text) else { return false };
    v.get("hooks").and_then(|h| h.as_object()).map(|m| m.values().any(|a| a.as_array().map(|a| a.iter().any(is_ours)).unwrap_or(false))).unwrap_or(false)
}

pub fn hooks_installed(port: u16) -> bool {
    let Ok(text) = fs::read_to_string(claude_settings_path()) else { return false };
    let Ok(v) = serde_json::from_str::<Value>(&text) else { return false };
    HOOK_EVENTS.iter().all(|ev| { let cmd = hook_command(port, ev); v.pointer(&format!("/hooks/{ev}")).and_then(|a| a.as_array()).map(|a| a.iter().any(|e|
        e.get("hooks").and_then(|h| h.as_array()).map(|hs| hs.iter().any(|h| h.get("command").and_then(|c| c.as_str()) == Some(cmd.as_str()))).unwrap_or(false)
    )).unwrap_or(false) })
}

/// Merge Dev Room's hooks into ~/.claude/settings.json, keeping everything else and writing a backup first.
pub fn install_hooks(port: u16) -> Result<Value, String> {
    let path = claude_settings_path();
    let original = fs::read_to_string(&path).unwrap_or_default();
    let mut root: Value = if original.trim().is_empty() { json!({}) } else {
        serde_json::from_str(&original).map_err(|e| format!("Hindi mabasa ang {}: {e}", path.display()))?
    };
    if !root.is_object() { return Err(format!("Hindi object ang laman ng {}.", path.display())); }
    let mut backup = Value::Null;
    if !original.is_empty() {
        let b = path.with_file_name(format!("settings.json.devroom-backup-{}", now_ms() / 1000));
        fs::write(&b, &original).map_err(|e| format!("Hindi makagawa ng backup: {e}"))?;
        backup = json!(b.display().to_string());
    }
    let hooks = root.as_object_mut().unwrap().entry("hooks").or_insert_with(|| json!({}));
    if !hooks.is_object() { *hooks = json!({}); }
    for ev in HOOK_EVENTS {
        let cmd = hook_command(port, ev);
        let list = hooks.as_object_mut().unwrap().entry(ev).or_insert_with(|| json!([]));
        if !list.is_array() { *list = json!([]); }
        let arr = list.as_array_mut().unwrap();
        // take out only our own hook; an entry that held nothing else goes too
        for e in arr.iter_mut() {
            if let Some(hs) = e.get_mut("hooks").and_then(|h| h.as_array_mut()) { hs.retain(|h| !is_our_cmd(h)); }
        }
        arr.retain(|e| e.get("hooks").and_then(|h| h.as_array()).map(|hs| !hs.is_empty()).unwrap_or(true));
        let mut entry = json!({"hooks": [{"type": "command", "command": cmd, "timeout": hook_timeout(ev)}]});
        if ev == "PreToolUse" || ev == "PostToolUse" { entry["matcher"] = json!("*"); }
        arr.push(entry);
    }
    if let Some(dir) = path.parent() { fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
    let tmp = path.with_extension("json.devroom-tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(&root).unwrap()).map_err(|e| format!("Hindi maisulat ang {}: {e}", path.display()))?;
    fs::rename(&tmp, &path).map_err(|e| format!("Hindi maisulat ang {}: {e}", path.display()))?;
    Ok(json!({"ok": true, "path": path.display().to_string(), "backup": backup}))
}

/// Read the last part of a transcript (JSONL) for the latest assistant text and any session title.
fn read_transcript(path: &str, full_title_scan: bool) -> (Option<String>, Option<String>) {
    let Ok(mut f) = fs::File::open(path) else { return (None, None) };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let tail = 512 * 1024u64;
    let _ = f.seek(SeekFrom::Start(len.saturating_sub(tail)));
    let mut buf = String::new();
    let _ = f.take(tail).read_to_string(&mut buf);
    let lines: Vec<&str> = buf.lines().collect();
    let skip = if len > tail { 1 } else { 0 };
    let (mut text, mut title) = (None, None);
    for line in lines.iter().skip(skip).rev() {
        if text.is_some() && title.is_some() { break; }
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        match v.get("type").and_then(|t| t.as_str()) {
            Some("assistant") if text.is_none() => {
                if let Some(parts) = v.pointer("/message/content").and_then(|c| c.as_array()) {
                    let t: Vec<&str> = parts.iter().filter(|p| p.get("type").and_then(|x| x.as_str()) == Some("text"))
                        .filter_map(|p| p.get("text").and_then(|x| x.as_str())).collect();
                    if !t.is_empty() { text = Some(t.join("\n")); }
                }
            }
            Some("custom-title") if title.is_none() => title = v.get("customTitle").and_then(|x| x.as_str()).map(String::from),
            Some("summary") if title.is_none() => title = v.get("summary").and_then(|x| x.as_str()).map(String::from),
            _ => {}
        }
    }
    if title.is_none() && full_title_scan && len < 64 * 1024 * 1024 {
        if let Ok(f) = fs::File::open(path) {
            for line in BufReader::new(f).lines().map_while(Result::ok) {
                if !line.contains("\"custom-title\"") && !line.contains("\"summary\"") { continue; }
                if let Ok(v) = serde_json::from_str::<Value>(&line) {
                    let t = v.get("customTitle").or_else(|| v.get("summary")).and_then(|x| x.as_str());
                    if let Some(t) = t { title = Some(t.to_string()); }
                }
            }
        }
    }
    (text, title)
}

fn describe_tool(tool: &str, input: &Value) -> (String, String) {
    let s = |k: &str| input.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    match tool {
        "Edit" | "MultiEdit" | "NotebookEdit" => ("Nagko-code.".into(), format!("Ine-edit ang {}.", base_name(&s("file_path")))),
        "Write" => ("Nagko-code.".into(), format!("Isinusulat ang {}.", base_name(&s("file_path")))),
        "Read" => ("Nagbabasa.".into(), format!("Binabasa ang {}.", base_name(&s("file_path")))),
        "Bash" | "PowerShell" => ("Nagpapatakbo.".into(), format!("Pinapatakbo ang `{}`.", clip(&s("command"), 70))),
        "Grep" | "Glob" => ("Naghahanap.".into(), format!("Hinahanap ang \"{}\".", clip(&s("pattern"), 50))),
        "Task" | "Agent" => ("Nagde-delegate.".into(), format!("Nag-utos sa subagent: {}.", clip(&s("description"), 60))),
        "WebFetch" | "WebSearch" => ("Nagre-research.".into(), "Nagbabasa sa web.".into()),
        "TodoWrite" | "TaskCreate" | "TaskUpdate" => ("Nagpaplano.".into(), "Ina-update ang listahan ng gagawin.".into()),
        t if t.starts_with("mcp__") => {
            let name = t.splitn(3, "__").nth(2).unwrap_or(t);
            ("Nagtatrabaho.".into(), format!("Gumagamit ng {name}."))
        }
        t => ("Nagtatrabaho.".into(), format!("Gumagamit ng {t}.")),
    }
}

fn last_question(text: &str) -> Option<String> {
    let t = text.trim();
    if !t.ends_with('?') { return None; }
    let tail: String = t.lines().last().unwrap_or(t).to_string();
    Some(clip(&tail, 220))
}

/// `pre` is the transcript read done outside the lock by `hook_reply` (last assistant text, title).
fn apply_hook(st: &mut State, v: &Value, pre: &Option<(Option<String>, Option<String>)>) {
    let ev = v.get("hook_event_name").and_then(|x| x.as_str()).unwrap_or("");
    let Some(sid) = v.get("session_id").and_then(|x| x.as_str()).map(String::from) else { return };
    if sid.is_empty() || sid.len() > 80 || !sid.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') { return; }
    if ev == "SessionEnd" {
        if st.sessions.remove(&sid).is_some() { st.bump(); st.save(); }
        return;
    }
    if !HOOK_EVENTS.contains(&ev) { return; }
    let now = now_ms();
    let entry = st.sessions.entry(sid.clone()).or_insert_with(|| json!({
        "id": sid, "status": "idle", "detail": "", "ask": "", "recent": "", "short": "", "auto_title": "", "title_scanned": false, "started_at": now
    }));
    let s = entry.as_object_mut().unwrap();
    if let Some(c) = v.get("cwd").and_then(|x| x.as_str()) { s.insert("cwd".into(), json!(c)); }
    if let Some(m) = v.get("permission_mode").and_then(|x| x.as_str()) { s.insert("perm_mode".into(), json!(m)); }
    let transcript = v.get("transcript_path").and_then(|x| x.as_str()).map(String::from);
    if let Some(t) = &transcript {
        s.insert("transcript".into(), json!(t));
        // Pick up the session's name (/rename or summary) once, the first time we see its transcript.
        if ev != "Stop" && !s.get("title_scanned").and_then(|x| x.as_bool()).unwrap_or(false) {
            if let Some((_, title)) = pre {
                s.insert("title_scanned".into(), json!(true));
                if let Some(title) = title { s.insert("auto_title".into(), json!(title.clone())); }
            }
        }
    }
    s.insert("updated_at".into(), json!(now));
    s.insert("hook_at".into(), json!(now));
    s.insert("last_event".into(), json!(ev));
    s.remove("live_guess");
    // a hook after a hand-over means Claude received it
    if let Some(inbox) = s.get_mut("inbox").and_then(|i| i.as_array_mut()) {
        for m in inbox.iter_mut() { if m.get("state").and_then(|x| x.as_str()) == Some("sent") { m["state"] = json!("delivered"); } }
    }
    if s.get("listening").and_then(|x| x.as_bool()).unwrap_or(false) { s.insert("listening".into(), json!(false)); }
    let gen = s.get("listen_gen").and_then(|x| x.as_u64()).unwrap_or(0) + 1;
    s.insert("listen_gen".into(), json!(gen));
    let mut persist = true;
    match ev {
        "SessionStart" => {
            s.insert("status".into(), json!("idle"));
            s.insert("ask".into(), json!(""));
            let how = v.get("source").and_then(|x| x.as_str()).unwrap_or("startup");
            s.insert("recent".into(), json!(if how == "resume" { "Bumalik ang session." } else { "Nagsimula ang session." }));
        }
        "UserPromptSubmit" => {
            s.insert("status".into(), json!("working"));
            s.insert("ask".into(), json!(""));
            s.insert("short".into(), json!("Nagbabasa ng utos."));
            s.insert("prompt_at".into(), json!(now));
            let p = v.get("prompt").and_then(|x| x.as_str()).unwrap_or("");
            push_convo(s, "user", p, 2000);
            // a session without a name is called after its first prompt, not after the folder every session shares
            let first = p.trim();
            if !s.contains_key("first_prompt") && !first.is_empty() && !first.starts_with('<') {
                s.insert("first_prompt".into(), json!(clip(first, 48)));
            }
            s.insert("detail".into(), json!(format!("Inutusan: {}", clip(p, 140))));
        }
        "PreToolUse" => {
            let tool = v.get("tool_name").and_then(|x| x.as_str()).unwrap_or("tool");
            let (short, detail) = describe_tool(tool, v.get("tool_input").unwrap_or(&Value::Null));
            s.insert("status".into(), json!("working"));
            s.insert("ask".into(), json!(""));
            s.insert("short".into(), json!(short));
            s.insert("detail".into(), json!(detail));
            persist = false;
        }
        "PostToolUse" => {
            // The tool ran, so any permission prompt for it has been answered.
            s.insert("status".into(), json!("working"));
            s.insert("ask".into(), json!(""));
            persist = false;
        }
        "SubagentStop" => {
            if s.get("status").and_then(|x| x.as_str()) == Some("idle") {
                s.insert("status".into(), json!("working"));
                s.insert("short".into(), json!("Binabasa ang resulta ng agent."));
                s.insert("detail".into(), json!("Tapos na ang isang agent; binabasa ang resulta."));
            }
            persist = false;
        }
        "Notification" => {
            let msg = v.get("message").and_then(|x| x.as_str()).unwrap_or("");
            let kind = v.get("notification_type").and_then(|x| x.as_str()).unwrap_or("");
            let idle = kind == "idle_prompt" || msg.to_lowercase().contains("waiting for your input");
            if idle {
                if s.get("status").and_then(|x| x.as_str()) != Some("needs") { s.insert("status".into(), json!("idle")); }
            } else {
                s.insert("status".into(), json!("needs"));
                s.insert("ask".into(), json!(clip(msg, 220)));
            }
        }
        "Stop" => {
            let mut text = v.get("last_assistant_message").and_then(|x| x.as_str()).map(String::from);
            if let Some((tx, title)) = pre {
                s.insert("title_scanned".into(), json!(true));
                if text.is_none() { text = tx.clone(); }
                if let Some(title) = title { s.insert("auto_title".into(), json!(title.clone())); }
            }
            let text = text.unwrap_or_default();
            push_convo(s, "assistant", &text, 4000);
            s.insert("recent".into(), json!(clip(&text, 300)));
            s.insert("detail".into(), json!(""));
            match last_question(&text) {
                Some(q) => { s.insert("status".into(), json!("needs")); s.insert("ask".into(), json!(q)); }
                None => { s.insert("status".into(), json!("idle")); s.insert("ask".into(), json!("")); }
            }
        }
        _ => {}
    }
    st.bump();
    if persist { st.save(); }
}

/* ---------- messages from the app to a session ----------
   A message waits in the session's inbox and reaches Claude through the next hook that can carry text:
   PostToolUse / UserPromptSubmit (extra context for the running turn) or Stop (the turn continues with it). */

/// Mark the queued messages as delivered and return their ids and the text for Claude.
fn take_queued(s: &mut Map<String, Value>) -> Option<(Vec<u64>, String)> {
    let inbox = s.get_mut("inbox")?.as_array_mut()?;
    let (mut ids, mut texts) = (Vec::new(), Vec::new());
    for m in inbox.iter_mut() {
        if m.get("state").and_then(|x| x.as_str()) == Some("queued") {
            ids.push(m.get("id").and_then(|x| x.as_u64()).unwrap_or(0));
            texts.push(m.get("text").and_then(|x| x.as_str()).unwrap_or("").to_string());
            m["state"] = json!("sent");
            m["sent_at"] = json!(now_ms());
        }
    }
    if ids.is_empty() { return None; }
    Some((ids, format!("Mensahe mula kay Boss (ipinadala mula sa Dev Room app):\n\n{}", texts.join("\n\n"))))
}

/// Put messages back in the queue when they could not be handed over (the hook was cancelled).
fn requeue(st: &Mutex<State>, sid: &str, ids: &[u64]) {
    let mut g = lk(st);
    if let Some(inbox) = g.sessions.get_mut(sid).and_then(|s| s.get_mut("inbox")).and_then(|i| i.as_array_mut()) {
        for m in inbox.iter_mut() {
            if ids.contains(&m.get("id").and_then(|x| x.as_u64()).unwrap_or(0)) { m["state"] = json!("queued"); }
        }
    }
    g.bump(); g.save();
}

/// The session starts working on a message from the app, like a new prompt.
fn mark_prompted(s: &mut Map<String, Value>, text: &str) {
    let now = now_ms();
    s.insert("status".into(), json!("working"));
    s.insert("ask".into(), json!(""));
    s.insert("short".into(), json!("Nagbabasa ng utos."));
    s.insert("detail".into(), json!(format!("Inutusan: {}", clip(text.rsplit("\n\n").next().unwrap_or(text), 140))));
    s.insert("prompt_at".into(), json!(now));
    s.insert("updated_at".into(), json!(now));
    s.insert("listening".into(), json!(false));
}

/// Stop hooks waiting for a phone message right now (the updater does not restart while any wait).
pub static LISTENERS: AtomicUsize = AtomicUsize::new(0);
struct Waiter;
impl Waiter { fn new() -> Waiter { LISTENERS.fetch_add(1, Ordering::Relaxed); Waiter } }
impl Drop for Waiter { fn drop(&mut self) { LISTENERS.fetch_sub(1, Ordering::Relaxed); } }

fn wants_listen(g: &State, window_ms: u64) -> bool {
    match g.away.as_str() { "on" => true, "off" => false, _ => now_ms().saturating_sub(g.remote_seen) < window_ms }
}

/// Answer a hook. PostToolUse and UserPromptSubmit carry queued messages as extra context. Stop hands them
/// over as a reason to continue, and while a phone has Dev Room open it waits for one.
fn hook_reply(req: Request, st: &Mutex<State>, v: &Value) {
    let ev = v.get("hook_event_name").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let sid = v.get("session_id").and_then(|x| x.as_str()).unwrap_or("").to_string();
    // read the transcript (it can be large) before taking the lock, and only when it is needed
    let pre = v.get("transcript_path").and_then(|x| x.as_str()).and_then(|t| {
        let scanned = lk(st).sessions.get(&sid).and_then(|s| s.get("title_scanned")).and_then(|x| x.as_bool()).unwrap_or(false);
        if ev == "Stop" || !scanned { Some(read_transcript(t, !scanned)) } else { None }
    });
    let mut g = lk(st);
    apply_hook(&mut g, v, &pre);
    // a subagent's tool call carries the parent's session id; messages are for the main session only
    let sub = v.get("agent_id").is_some() || v.get("transcript_path").and_then(|x| x.as_str()).map(|t| t.contains("subagents")).unwrap_or(false);
    if sub { drop(g); return respond_json(req, 200, &json!({})); }
    if ev == "PostToolUse" || ev == "UserPromptSubmit" {
        let got = g.sessions.get_mut(&sid).and_then(|s| s.as_object_mut()).and_then(take_queued);
        let Some((ids, text)) = got else { drop(g); return respond_json(req, 200, &json!({})) };
        g.bump(); g.save(); drop(g);
        let out = json!({"hookSpecificOutput": {"hookEventName": ev, "additionalContext": text}});
        if req.respond(json_response(200, &out)).is_err() { requeue(st, &sid, &ids); }
        return;
    }
    if ev != "Stop" { drop(g); return respond_json(req, 200, &json!({})); }
    let deadline = now_ms() + LISTEN_SECS * 1000;
    let mut gen = 0;
    let mut first = true;
    let _waiter = Waiter::new();
    let got = loop {
        let listen = wants_listen(&g, if first { 120_000 } else { 600_000 }) && LISTENERS.load(Ordering::Relaxed) <= 16;
        let Some(s) = g.sessions.get_mut(&sid).and_then(|s| s.as_object_mut()) else { break None };
        if first { gen = s.get("listen_gen").and_then(|x| x.as_u64()).unwrap_or(0); }
        else if s.get("listen_gen").and_then(|x| x.as_u64()).unwrap_or(0) != gen { break None; }   // the session moved on
        if let Some((ids, text)) = take_queued(s) { mark_prompted(s, &text); g.bump(); g.save(); break Some((ids, text)); }
        if !listen || now_ms() > deadline {
            if !first { s.insert("listening".into(), json!(false)); g.bump(); }
            break None;
        }
        if first { s.insert("listening".into(), json!(true)); s.insert("listen_at".into(), json!(now_ms())); g.bump(); first = false; }
        drop(g);
        std::thread::sleep(std::time::Duration::from_millis(400));
        g = lk(st);
    };
    drop(g);
    match got {
        Some((ids, text)) => {
            let out = json!({"decision": "block", "reason": text});
            if req.respond(json_response(200, &out)).is_err() {
                requeue(st, &sid, &ids);
                let mut g = lk(st);
                if let Some(s) = g.sessions.get_mut(&sid).and_then(|s| s.as_object_mut()) { s.insert("status".into(), json!("idle")); }
                g.bump();
            }
        }
        None => { respond_json(req, 200, &json!({})); deliver_soon(&sid); }
    }
}

/// The visible conversation: your prompts and the session's replies at the end of each turn (newest 40).
fn push_convo(s: &mut Map<String, Value>, role: &str, text: &str, max: usize) {
    let text = text.trim();
    if text.is_empty() { return; }
    let t: String = if text.chars().count() > max { format!("{}…", text.chars().take(max).collect::<String>()) } else { text.to_string() };
    let c = s.entry("convo").or_insert_with(|| json!([]));
    if !c.is_array() { *c = json!([]); }
    let arr = c.as_array_mut().unwrap();
    // the live transcript reader and the hooks report the same lines: keep one
    let now = now_ms();
    if arr.iter().rev().take(8).any(|m| m.get("role").and_then(|x| x.as_str()) == Some(role) && m.get("text").and_then(|x| x.as_str()) == Some(t.as_str())
        && now.saturating_sub(m.get("at").and_then(|x| x.as_u64()).unwrap_or(0)) < 30 * 60 * 1000) { return; }
    arr.push(json!({"role": role, "text": t, "at": now}));
    if arr.len() > 40 { let n = arr.len() - 40; arr.drain(..n); }
    let n = s.get("convo_n").and_then(|x| x.as_u64()).unwrap_or(0) + 1;
    s.insert("convo_n".into(), json!(n));
}

fn record(s: &Value, names: &Value) -> Value {
    let id = s.get("id").and_then(|x| x.as_str()).unwrap_or("");
    let cwd = s.get("cwd").and_then(|x| x.as_str()).unwrap_or("");
    let auto = s.get("auto_title").and_then(|x| x.as_str()).filter(|t| !t.is_empty()).map(String::from);
    let title = names.get(id).and_then(|x| x.as_str()).filter(|t| !t.trim().is_empty()).map(String::from)
        .or(auto)
        .or_else(|| s.get("first_prompt").and_then(|x| x.as_str()).filter(|t| !t.is_empty()).map(String::from))
        .unwrap_or_else(|| if cwd.is_empty() { format!("Session {}", id.chars().take(8).collect::<String>()) } else { base_name(cwd) });
    let bucket = match s.get("status").and_then(|x| x.as_str()).unwrap_or("idle") {
        "working" => "SESSION_STATUS_BUCKET_WORKING",
        "needs" => "SESSION_STATUS_BUCKET_BLOCKED",
        "failed" => "SESSION_STATUS_BUCKET_FAILED",
        _ => "SESSION_STATUS_BUCKET_COMPLETED",
    };
    json!({
        "id": id, "title": title, "status_bucket": bucket,
        "updated_at": s.get("updated_at").cloned().unwrap_or(json!(0)),
        "environment_kind": "bridge", "origin": "local_pc", "cwd": cwd,
        "short": s.get("short").cloned().unwrap_or(json!("")),
        "prompt_at": s.get("prompt_at").cloned().unwrap_or(json!(0)),
        "inbox": s.get("inbox").cloned().unwrap_or(json!([])),
        "convo_n": s.get("convo_n").cloned().unwrap_or(json!(0)),
        "listening": s.get("listening").cloned().unwrap_or(json!(false)),
        "post_turn_summary": {
            "status_detail": s.get("detail").cloned().unwrap_or(json!("")),
            "recent_action": s.get("recent").cloned().unwrap_or(json!("")),
            "needs_action": s.get("ask").cloned().unwrap_or(json!(""))
        }
    })
}

fn lan_ip() -> Option<IpAddr> { local_ip_address::local_ip().ok() }

/// The PC's Tailscale address (100.64.0.0/10), so a paired phone can reach Dev Room away from home.
fn tailscale_ip() -> Option<IpAddr> {
    local_ip_address::list_afinet_netifas().ok()?.into_iter().map(|(_, ip)| ip).find(|ip| match ip {
        IpAddr::V4(v) => { let o = v.octets(); o[0] == 100 && (64..128).contains(&o[1]) }
        _ => false,
    })
}

fn pair_link(port: u16, token: &str) -> Option<String> {
    let lan = lan_ip().map(|ip| format!("http://{ip}:{port}"));
    let ts = tailscale_ip().map(|ip| format!("http://{ip}:{port}"));
    let primary = lan.clone().or(ts.clone())?;
    let mut link = format!("{primary}/?t={token}");
    if let (Some(_), Some(t)) = (&lan, &ts) { link.push_str(&format!("&a={}", t.replace(':', "%3A").replace('/', "%2F"))); }
    Some(link)
}

fn is_loopback(addr: Option<&SocketAddr>) -> bool { addr.map(|a| a.ip().is_loopback()).unwrap_or(false) }

fn header(k: &str, v: &str) -> Header { Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap() }

fn json_response(code: u16, v: &Value) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_data(serde_json::to_vec(v).unwrap_or_default()).with_status_code(StatusCode(code))
        .with_header(header("Content-Type", "application/json; charset=utf-8"))
        .with_header(header("Cache-Control", "no-store"))
}

fn respond_json(req: Request, code: u16, v: &Value) {
    let r = Response::from_data(serde_json::to_vec(v).unwrap_or_default()).with_status_code(StatusCode(code))
        .with_header(header("Content-Type", "application/json; charset=utf-8"))
        .with_header(header("Cache-Control", "no-store"))
        // The Android app's shell (http://localhost) reads /api/state; the pairing token guards the data.
        .with_header(header("Access-Control-Allow-Origin", "*"));
    let _ = req.respond(r);
}

fn read_body(req: &mut Request) -> Option<Value> {
    let mut buf = Vec::new();
    req.as_reader().take(MAX_BODY as u64).read_to_end(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

/// Requests must name this PC (DNS rebinding) and, when a browser sends an Origin, come from Dev Room
/// itself or the Android app shell. A web page open on the PC therefore cannot drive Dev Room.
fn request_allowed(req: &Request, port: u16, local: bool, tun: bool) -> bool {
    let hv = |k: &'static str| req.headers().iter().find(|h| h.field.equiv(k)).map(|h| h.value.as_str().to_ascii_lowercase());
    let Some(host) = hv("Host") else { return false };
    if tun {
        // through the tunnel: the tunnel's own name (or the loopback address cloudflared may use); the token guards everything
        let th = crate::tunnel::host().map(|h| h.to_ascii_lowercase());
        let ok = th.as_deref() == Some(host.as_str()) || host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}");
        if !ok { return false; }
        return match hv("Origin") {
            None => true,
            Some(o) => th.map(|h| o == format!("https://{h}")).unwrap_or(false) || o == "http://localhost" || o == "https://localhost" || o == "capacitor://localhost",
        };
    }
    // An IP address (any of this PC's adapters) or localhost: a rebinding attack always uses a host name.
    let ok_host = host == format!("localhost:{port}")
        || host.parse::<SocketAddr>().map(|a| a.port() == port).unwrap_or(false);
    if !ok_host { return false; }
    match hv("Origin") {
        None => true,
        // the Android app shell's origins only count for phones; on this PC they would be any local web server
        Some(o) => o == format!("http://{host}") || (!local && (o == "http://localhost" || o == "https://localhost" || o == "capacitor://localhost")),
    }
}

fn handle(mut req: Request, st: &Arc<Mutex<State>>, port: u16, tun: bool) {
    // everything that comes through the tunnel is a phone, even though cloudflared connects from this PC
    let local = !tun && is_loopback(req.remote_addr());
    if !request_allowed(&req, port, local, tun) { return respond_json(req, 403, &json!({"error": "origin"})); }
    let has = |k: &'static str| req.headers().iter().find(|h| h.field.equiv(k)).map(|h| h.value.as_str().to_string());
    // Hooks come from curl, never from a browser (browsers always send Origin on POST).
    if req.url().starts_with("/hook") && has("Origin").is_some() { return respond_json(req, 403, &json!({"error": "origin"})); }
    // State-changing calls carry a custom header, which a page on another site cannot add without a preflight we never grant.
    if *req.method() == Method::Post && req.url().starts_with("/api/") && has("X-DevRoom").is_none() {
        return respond_json(req, 403, &json!({"error": "header"}));
    }
    let app_ok = has("X-DevRoom-App").map(|k| k == app_key()).unwrap_or(false);
    let url = req.url().to_string();
    let (path, query) = url.split_once('?').unwrap_or((url.as_str(), ""));
    let path = path.to_string();
    let q: HashMap<String, String> = query.split('&').filter_map(|kv| kv.split_once('=')).map(|(k, v)| (k.to_string(), v.to_string())).collect();
    let token_ok = local || {
        let token = lk(st).token.clone();
        let given = req.headers().iter().find(|h| h.field.equiv("X-DevRoom-Token")).map(|h| h.value.as_str().to_string())
            .or_else(|| q.get("t").cloned()).unwrap_or_default();
        // constant time: the tunnel puts this check on the internet
        given.len() == token.len() && given.bytes().zip(token.bytes()).fold(0u8, |a, (x, y)| a | (x ^ y)) == 0
    };
    match (req.method().clone(), path.as_str()) {
        (Method::Get, "/") | (Method::Get, "/index.html") => {
            let html = crate::updater::ui_html().unwrap_or_else(|| INDEX_HTML.to_string());
            let r = Response::from_string(html).with_header(header("Content-Type", "text/html; charset=utf-8")).with_header(header("Cache-Control", "no-store"));
            let _ = req.respond(r);
        }
        (Method::Get, "/manifest.json") => {
            let m = json!({"name": "Dev Room", "short_name": "Dev Room", "start_url": "/", "display": "standalone", "background_color": "#efe7da", "theme_color": "#2a201a",
                "icons": [{"src": "/icon.svg", "sizes": "any", "type": "image/svg+xml"}]});
            respond_json(req, 200, &m);
        }
        (Method::Get, "/icon.svg") => {
            let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="14" fill="#2a201a"/><rect x="12" y="24" width="32" height="26" rx="3" fill="#c9a27e"/><rect x="44" y="30" width="8" height="12" rx="3" fill="none" stroke="#c9a27e" stroke-width="4"/><rect x="18" y="10" width="4" height="10" fill="#efe7da"/><rect x="28" y="12" width="4" height="8" fill="#efe7da"/></svg>"##;
            let _ = req.respond(Response::from_string(svg).with_header(header("Content-Type", "image/svg+xml")));
        }
        (Method::Post, "/hook") => {
            if !local { return respond_json(req, 403, &json!({"error": "loopback only"})); }
            match read_body(&mut req) { Some(v) => hook_reply(req, st, &v), None => respond_json(req, 200, &json!({})) }
        }
        (Method::Get, "/api/state") => {
            if !token_ok { return respond_json(req, 401, &json!({"error": "pairing"})); }
            let since: u64 = q.get("v").and_then(|v| v.parse().ok()).unwrap_or(0);
            let mut s = lk(st);
            if !local && q.get("bg").map(|b| b != "1").unwrap_or(true) { s.remote_seen = now_ms(); }
            if since == s.version { drop(s); let _ = req.respond(Response::empty(204).with_header(header("Access-Control-Allow-Origin", "*"))); return; }
            let names = s.team.get("names").cloned().unwrap_or(json!({}));
            let mut list: Vec<Value> = s.sessions.values().map(|x| record(x, &names)).collect();
            list.sort_by_key(|r| std::cmp::Reverse(r.get("updated_at").and_then(|t| t.as_u64()).unwrap_or(0)));
            let lan = lan_ip().map(|ip| format!("http://{ip}:{port}/"));
            let body = json!({"version": s.version, "sessions": list, "team": s.team, "local": local,
                "hooks_installed": if local { json!(hooks_installed(port)) } else { Value::Null }, "embed_claude": local && app_ok && ON_CLAUDE.get().is_some(), "away": s.away,
                "settings_path": if local { json!(claude_settings_path().display().to_string()) } else { Value::Null },
                "lan_url": lan, "ts_url": tailscale_ip().map(|ip| format!("http://{ip}:{port}/")),
                "app_build": crate::updater::BUILD, "app_version": crate::updater::VERSION, "ui_build": crate::updater::ui_build(),
                "remote": if local { crate::tunnel::info() } else { Value::Null },
                "instant_chat": claude_bin().is_some(),
                "update_ready": crate::updater::exe_ready(), "update_note": crate::updater::note(),
                "latest": crate::updater::latest()});
            drop(s);
            respond_json(req, 200, &body);
        }
        (Method::Post, "/api/team") => {
            if !token_ok { return respond_json(req, 401, &json!({"error": "pairing"})); }
            let Some(v) = read_body(&mut req) else { return respond_json(req, 400, &json!({"error": "bad json"})) };
            let mut s = lk(st);
            let bad = v.get("lead").map(|x| !x.is_string() && !x.is_null()).unwrap_or(false)
                || v.get("ids").map(|x| !x.is_array()).unwrap_or(false)
                || v.get("names").map(|x| !x.is_object()).unwrap_or(false);
            if bad { drop(s); return respond_json(req, 400, &json!({"error": "bad team"})); }
            let lead = v.get("lead").cloned().filter(|x| x.is_string()).unwrap_or(Value::Null);
            let ids: Vec<Value> = v.get("ids").and_then(|x| x.as_array()).cloned().unwrap_or_default().into_iter().filter(|x| x.is_string()).collect();
            let names: Map<String, Value> = v.get("names").and_then(|x| x.as_object()).cloned().unwrap_or_default()
                .into_iter().filter(|(_, n)| n.is_string()).map(|(k, n)| (k, json!(clip(n.as_str().unwrap(), 120)))).collect();
            s.team = json!({"lead": lead, "ids": ids, "names": names});
            s.bump(); s.save();
            respond_json(req, 200, &json!({"ok": true}));
        }
        (Method::Post, "/api/send") => {
            if !token_ok { return respond_json(req, 401, &json!({"error": "pairing"})); }
            let Some(v) = read_body(&mut req) else { return respond_json(req, 400, &json!({"error": "bad json"})) };
            let sid = v.get("sid").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let text = v.get("text").and_then(|x| x.as_str()).unwrap_or("").trim().chars().take(4000).collect::<String>();
            if text.is_empty() { return respond_json(req, 400, &json!({"error": "empty"})); }
            let mut g = lk(st);
            let Some(s) = g.sessions.get_mut(&sid).and_then(|s| s.as_object_mut()) else { drop(g); return respond_json(req, 404, &json!({"error": "no session"})) };
            // unique even for messages sent in the same millisecond
            static LAST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let id = { let mut cur = LAST.load(std::sync::atomic::Ordering::SeqCst); loop { let next = (cur + 1).max(now_ms()); match LAST.compare_exchange(cur, next, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst) { Ok(_) => break next, Err(c) => cur = c } } };
            let inbox = s.entry("inbox").or_insert_with(|| json!([]));
            if !inbox.is_array() { *inbox = json!([]); }
            let arr = inbox.as_array_mut().unwrap();
            arr.push(json!({"id": id, "text": text, "at": id, "state": "queued"}));
            if arr.len() > 30 { let n = arr.len() - 30; arr.drain(..n); }
            g.bump(); g.save(); drop(g);
            respond_json(req, 200, &json!({"ok": true, "id": id}));
            deliver_now(&sid);
        }
        (Method::Get, "/api/convo") => {
            // Conversations are fetched apart from /api/state, only when a session's convo_n changes.
            if !token_ok { return respond_json(req, 401, &json!({"error": "pairing"})); }
            let g = lk(st);
            let all: Map<String, Value> = g.sessions.iter().map(|(k, s)| (k.clone(), s.get("convo").cloned().unwrap_or(json!([])))).collect();
            drop(g);
            respond_json(req, 200, &Value::Object(all));
        }
        (Method::Post, "/api/claude-view") => {
            if !local || !app_ok { return respond_json(req, 403, &json!({"error": "app window only"})); }
            let v = read_body(&mut req).unwrap_or(Value::Null);
            let n = |k: &str| v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0);
            let rect = if v.get("show").and_then(|x| x.as_bool()).unwrap_or(false) && n("w") > 50.0 && n("h") > 50.0 { Some((n("x"), n("y"), n("w"), n("h"))) } else { None };
            let ok = ON_CLAUDE.get().map(|f| f(rect)).is_some();
            respond_json(req, if ok { 200 } else { 503 }, &json!({"ok": ok}));
        }
        (Method::Post, "/api/remote") => {
            if !local { return respond_json(req, 403, &json!({"error": "loopback only"})); }
            let Some(v) = read_body(&mut req) else { return respond_json(req, 400, &json!({"error": "bad json"})) };
            let on = v.get("on").and_then(|x| x.as_bool()).unwrap_or(true);
            let mut g = lk(st);
            g.remote_off = !on; g.bump(); g.save(); let token = g.token.clone(); drop(g);
            crate::tunnel::set_enabled(on, token);
            respond_json(req, 200, &json!({"ok": true}));
        }
        (Method::Post, "/api/show") => {
            if !local { return respond_json(req, 403, &json!({"error": "loopback only"})); }
            let shown = ON_SHOW.get().map(|f| f()).is_some();
            respond_json(req, if shown { 200 } else { 503 }, &json!({"ok": shown}));
        }
        (Method::Post, "/api/away") => {
            if !token_ok { return respond_json(req, 401, &json!({"error": "pairing"})); }
            let Some(v) = read_body(&mut req) else { return respond_json(req, 400, &json!({"error": "bad json"})) };
            let mode = v.get("mode").and_then(|x| x.as_str()).unwrap_or("auto");
            if !["auto", "on", "off"].contains(&mode) { return respond_json(req, 400, &json!({"error": "mode"})); }
            let mut g = lk(st);
            g.away = mode.to_string(); g.bump(); g.save(); drop(g);
            respond_json(req, 200, &json!({"ok": true}));
        }
        (Method::Post, "/api/install-hooks") => {
            if !local { return respond_json(req, 403, &json!({"error": "Sa PC lang puwedeng i-set up ang hooks."})); }
            match install_hooks(port) {
                Ok(v) => { lk(st).bump(); respond_json(req, 200, &v) }
                Err(e) => respond_json(req, 500, &json!({"error": e})),
            }
        }
        (Method::Get, "/pair.svg") => {
            if !local { return respond_json(req, 403, &json!({"error": "loopback only"})); }
            let token = lk(st).token.clone();
            let Some(link) = pair_link(port, &token) else { return respond_json(req, 404, &json!({"error": "no network address"})) };
            let svg = qrcode::QrCode::new(link.as_bytes()).map(|c| c.render::<qrcode::render::svg::Color>().min_dimensions(240, 240)
                .dark_color(qrcode::render::svg::Color("#2a201a")).light_color(qrcode::render::svg::Color("#ffffff")).build()).unwrap_or_default();
            let _ = req.respond(Response::from_string(svg).with_header(header("Content-Type", "image/svg+xml")).with_header(header("Cache-Control", "no-store")));
        }
        (Method::Post, "/api/restart") => {
            if !local { return respond_json(req, 403, &json!({"error": "loopback only"})); }
            respond_json(req, 200, &json!({"ok": true}));
            crate::updater::restart(q.get("hidden").map(|v| v == "1").unwrap_or(false));
        }
        (Method::Get, "/pair.json") => {
            if !local { return respond_json(req, 403, &json!({"error": "loopback only"})); }
            let token = lk(st).token.clone();
            respond_json(req, 200, &json!({"link": pair_link(port, &token)}));
        }
        _ => respond_json(req, 404, &json!({"error": "not found"})),
    }
}

/// Start the server on a background thread. Returns the port, or None if it is already taken
/// (for example a second copy of Dev Room is running; the window then shows that one).
/// Newest write to a session's transcript or to its subagents' transcripts.
fn latest_write(t: &Path) -> u64 {
    let ms = |p: &Path| fs::metadata(p).and_then(|m| m.modified()).ok()
        .and_then(|m| m.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as u64).unwrap_or(0);
    let mut best = ms(t);
    if let (Some(dir), Some(stem)) = (t.parent(), t.file_stem()) {
        if let Ok(rd) = fs::read_dir(dir.join(stem).join("subagents")) {
            for e in rd.flatten() { best = best.max(ms(&e.path())); }
        }
    }
    best
}

/// A session can keep working after its turn ends (background agents and tasks, or a turn the
/// session resumes on its own) without sending a prompt hook. If its transcript is still being written
/// well after the last hook, it is working; once the writes stop for a while, it is idle again.
fn liveness(st: &Mutex<State>) {
    // 1. which transcripts to look at (under the lock), 2. look at them (without it), 3. apply.
    let todo: Vec<(String, String)> = lk(st).sessions.iter().filter_map(|(k, v)| {
        let st = v.get("status").and_then(|x| x.as_str()).unwrap_or("idle");
        let guess = v.get("live_guess").and_then(|x| x.as_bool()).unwrap_or(false);
        if st != "idle" && !guess { return None; }
        v.get("transcript").and_then(|x| x.as_str()).map(|t| (k.clone(), t.to_string()))
    }).collect();
    let writes: HashMap<String, u64> = todo.into_iter().map(|(k, t)| { let w = latest_write(Path::new(&t)); (k, w) }).collect();
    let now = now_ms();
    let mut s = lk(st);
    let mut changed = false;
    let mut save = false;
    for (sid, v) in s.sessions.iter_mut() {
        let Some(o) = v.as_object_mut() else { continue };
        let status = o.get("status").and_then(|x| x.as_str()).unwrap_or("idle").to_string();
        let hook_at = o.get("updated_at").and_then(|x| x.as_u64()).unwrap_or(0);
        let guess = o.get("live_guess").and_then(|x| x.as_bool()).unwrap_or(false);
        let listening = o.get("listening").and_then(|x| x.as_bool()).unwrap_or(false);
        if let Some(&w) = writes.get(sid) {
            if status == "idle" && !listening {
                if w > hook_at + 5000 && now.saturating_sub(w) < 15_000 {
                    o.insert("status".into(), json!("working"));
                    o.insert("short".into(), json!("Tuloy ang trabaho."));
                    o.insert("detail".into(), json!("May ginagawa pa sa background."));
                    o.insert("live_guess".into(), json!(true));
                    o.insert("live_at".into(), json!(now));
                    changed = true;
                }
            } else if guess && status == "working" && now.saturating_sub(w.max(hook_at)) > 60_000 {
                o.insert("status".into(), json!("idle"));
                o.insert("short".into(), json!(""));
                o.insert("detail".into(), json!(""));
                o.remove("live_guess");
                changed = true;
            }
        }
        // a wait whose hook is long gone
        if listening && now.saturating_sub(o.get("listen_at").and_then(|x| x.as_u64()).unwrap_or(0)) > (LISTEN_SECS + 60) * 1000 {
            o.insert("listening".into(), json!(false));
            changed = true;
        }
        // a message handed over but never acknowledged by a later hook (the hook was cancelled): queue it again
        let last_hook = o.get("hook_at").and_then(|x| x.as_u64()).unwrap_or(0);
        let mut back = false;
        if let Some(inbox) = o.get_mut("inbox").and_then(|i| i.as_array_mut()) {
            for m in inbox.iter_mut() {
                let sent_at = m.get("sent_at").and_then(|x| x.as_u64()).unwrap_or(0);
                if m.get("state").and_then(|x| x.as_str()) == Some("sent") && now.saturating_sub(sent_at) > 180_000 && last_hook < sent_at {
                    m["state"] = json!("queued");
                    back = true;
                }
            }
        }
        if back {
            if status == "working" && last_hook < now.saturating_sub(180_000) { o.insert("status".into(), json!("idle")); }
            changed = true; save = true;
        }
    }
    // forget sessions that ended without SessionEnd (3 days quiet)
    let cutoff = now.saturating_sub(3 * 24 * 3600 * 1000);
    let before = s.sessions.len();
    s.sessions.retain(|_, v| v.get("updated_at").and_then(|t| t.as_u64()).unwrap_or(0) > cutoff);
    if s.sessions.len() != before { changed = true; save = true; }
    if changed { s.bump(); }
    if save { s.save(); }
}

pub fn start(port: u16) -> Option<u16> {
    let mut server = None;
    for _ in 0..40 {
        match Server::http(("0.0.0.0", port)) { Ok(s) => { server = Some(s); break } Err(_) => std::thread::sleep(std::time::Duration::from_millis(250)) }
    }
    let server = server?;
    let st = Arc::new(Mutex::new(State::load(data_dir())));
    // Hooks set up by an older version get the newer events too (same consent, same command).
    if !hooks_installed(port) && has_any_ours() { let _ = install_hooks(port); }
    let _ = SHARED.set(st.clone());
    let tail = st.clone();
    std::thread::spawn(move || loop { std::thread::sleep(std::time::Duration::from_millis(900)); tail_transcripts(&tail); });
    let live = st.clone();
    std::thread::spawn(move || loop { std::thread::sleep(std::time::Duration::from_secs(3)); liveness(&live); });
    // Malayo mode: a loopback-only port for the tunnel, where every request counts as a phone
    if let Ok(ts) = Server::http(("127.0.0.1", crate::tunnel::PORT)) {
        let st2 = st.clone();
        std::thread::spawn(move || {
            for req in ts.incoming_requests() {
                let st = st2.clone();
                std::thread::spawn(move || handle(req, &st, crate::tunnel::PORT, true));
            }
        });
        let g = lk(&st);
        if !g.remote_off { crate::tunnel::set_enabled(true, g.token.clone()); }
    }
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let st = st.clone();
            std::thread::spawn(move || handle(req, &st, port, false));
        }
    });
    Some(port)
}


/* ---------- real-time chat ----------
   Reading: every session's transcript is followed as it is written, so each reply shows in the app as soon as
   Claude writes it (not only when the turn ends).
   Sending: a working session gets the message with its next tool call (seconds). A session that is not working
   is continued right away by Dev Room itself: `claude -p --resume <session>` with the message, in the session's
   folder and permission mode, so it answers immediately. */

static SHARED: std::sync::OnceLock<Arc<Mutex<State>>> = std::sync::OnceLock::new();
static TAIL: Mutex<Option<HashMap<String, (String, u64, bool)>>> = Mutex::new(None);
static RUNS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The text a transcript line shows in the chat: your prompts and Claude's replies (not tool calls or results).
fn transcript_line(v: &Value) -> Option<(&'static str, String)> {
    if v.get("isSidechain").and_then(|x| x.as_bool()).unwrap_or(false) || v.get("isMeta").and_then(|x| x.as_bool()).unwrap_or(false) { return None; }
    let kind = v.get("type").and_then(|x| x.as_str())?;
    let content = v.get("message")?.get("content")?;
    let texts = |c: &Value| -> Option<String> {
        let arr = c.as_array()?;
        if arr.iter().any(|b| b.get("type").and_then(|x| x.as_str()) == Some("tool_result")) { return None; }
        let t: Vec<&str> = arr.iter().filter(|b| b.get("type").and_then(|x| x.as_str()) == Some("text")).filter_map(|b| b.get("text").and_then(|x| x.as_str())).collect();
        (!t.is_empty()).then(|| t.join("\n\n"))
    };
    match kind {
        "assistant" => texts(content).map(|t| ("assistant", t)),
        "user" => {
            let t = content.as_str().map(String::from).or_else(|| texts(content))?;
            (!t.trim_start().starts_with('<')).then_some(("user", t))
        }
        _ => None,
    }
}

fn tail_transcripts(st: &Mutex<State>) {
    let now = now_ms();
    // (session, transcript, whether it was active in the last 2 minutes)
    let list: Vec<(String, String, bool)> = lk(st).sessions.iter()
        .filter_map(|(k, v)| v.get("transcript").and_then(|x| x.as_str()).map(|t| (k.clone(), t.to_string(),
            now.saturating_sub(v.get("hook_at").and_then(|x| x.as_u64()).unwrap_or(0)) < 120_000))).collect();
    let mut found: Vec<(String, &'static str, String)> = Vec::new();
    {
        let mut guard = TAIL.lock().unwrap_or_else(|e| e.into_inner());
        let tails = guard.get_or_insert_with(HashMap::new);
        tails.retain(|k, _| list.iter().any(|(s, _, _)| s == k));
        for (sid, path, active) in &list {
            let len = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            // The first look starts at the end (earlier lines are in the saved conversation), except for a session
            // active right now: its newest lines may have been written just before we looked (repeats are skipped).
            let e = tails.entry(sid.clone()).or_insert_with(|| (path.clone(), if *active { len.saturating_sub(64 * 1024) } else { len }, *active));
            if &e.0 != path || len < e.1 { *e = (path.clone(), len, false); continue; }
            if len == e.1 { continue; }
            let catch_up = std::mem::replace(&mut e.2, false);
            let fresh = catch_up && e.1 > 0;
            let start = e.1.max(len.saturating_sub(4 * 1024 * 1024));
            let Ok(mut f) = fs::File::open(path) else { continue };
            if f.seek(SeekFrom::Start(start)).is_err() { continue; }
            let mut buf = Vec::new();
            if f.take(len - start).read_to_end(&mut buf).is_err() { continue; }
            // only whole lines; a line still being written is read next time
            let Some(end) = buf.iter().rposition(|&b| b == b'\n') else { continue };
            e.1 = start + end as u64 + 1;
            // reading from the middle of a file: drop the partial first line
            let body = if fresh { buf[..end].splitn(2, |&b| b == b'\n').nth(1).unwrap_or(&[]) } else { &buf[..end] };
            let mut got: Vec<(&'static str, String)> = body.split(|&b| b == b'\n')
                .filter_map(|line| serde_json::from_slice::<Value>(line).ok()).filter_map(|v| transcript_line(&v)).collect();
            // catching up on a session that is already running: only its current turn (from your last prompt)
            if catch_up {
                if let Some(i) = got.iter().rposition(|(r, _)| *r == "user") { got.drain(..i); } else { got.clear(); }
            }
            found.extend(got.into_iter().map(|(r, t)| (sid.clone(), r, t)));
        }
    }
    if found.is_empty() { return; }
    let mut g = lk(st);
    for (sid, role, text) in found {
        if let Some(s) = g.sessions.get_mut(&sid).and_then(|s| s.as_object_mut()) {
            push_convo(s, role, &text, if role == "user" { 2000 } else { 4000 });
            if role == "assistant" { s.insert("recent".into(), json!(clip(&text, 300))); s.insert("updated_at".into(), json!(now_ms())); }
        }
    }
    g.bump();
}

/// The Claude Code program on this PC (the one the sessions use), for continuing a session right away.
pub fn claude_bin() -> Option<PathBuf> {
    static CACHE: Mutex<Option<(PathBuf, u64)>> = Mutex::new(None);
    if let Some((p, at)) = CACHE.lock().ok()?.clone() { if p.exists() || now_ms() - at < 60_000 { return p.exists().then_some(p); } }
    let exe = if cfg!(windows) { "claude.exe" } else { "claude" };
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Some(p) = std::env::var_os("DEVROOM_CLAUDE") { cands.push(PathBuf::from(p)); }
    if let Some(path) = std::env::var_os("PATH") { for d in std::env::split_paths(&path) { cands.push(d.join(exe)); if cfg!(windows) { cands.push(d.join("claude.cmd")); } } }
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from);
    if let Some(h) = &home { cands.push(h.join(".local").join("bin").join(exe)); cands.push(h.join(".claude").join("local").join(exe)); }
    if let Some(a) = std::env::var_os("APPDATA").map(PathBuf::from) { cands.push(a.join("npm").join("claude.cmd")); }
    // the copy the Claude desktop app keeps (newest version folder)
    for base in [std::env::var_os("APPDATA"), std::env::var_os("LOCALAPPDATA")].into_iter().flatten().map(PathBuf::from) {
        if let Ok(rd) = fs::read_dir(base.join("Claude").join("claude-code")) {
            let mut v: Vec<PathBuf> = rd.flatten().map(|e| e.path().join(exe)).filter(|p| p.exists()).collect();
            v.sort_by_key(|p| std::cmp::Reverse(fs::metadata(p).and_then(|m| m.modified()).ok()));
            cands.extend(v);
        }
    }
    let found = cands.into_iter().find(|p| p.is_file());
    if let Ok(mut c) = CACHE.lock() { *c = Some((found.clone().unwrap_or_default(), now_ms())); }
    found
}

/// A session can be continued from here when its turn is over (not working, not waiting on a permission prompt).
fn resumable(s: &Map<String, Value>) -> bool {
    let status = s.get("status").and_then(|x| x.as_str()).unwrap_or("idle");
    let last = s.get("last_event").and_then(|x| x.as_str()).unwrap_or("");
    let listening = s.get("listening").and_then(|x| x.as_bool()).unwrap_or(false);
    !listening && (status == "idle" || (status == "needs" && last == "Stop")) && !s.get("live_guess").and_then(|x| x.as_bool()).unwrap_or(false)
}

fn deliver_soon(sid: &str) {
    let sid = sid.to_string();
    // give the session a moment to settle after its Stop hook
    std::thread::spawn(move || { std::thread::sleep(std::time::Duration::from_millis(1500)); deliver_now(&sid); });
}

/// Hand the queued messages to a session that is not working: continue it right away with them.
fn deliver_now(sid: &str) {
    let Some(st) = SHARED.get().cloned() else { return };
    if !sid.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') || sid.is_empty() { return; }
    let Some(bin) = claude_bin() else { return };
    {
        let mut runs = RUNS.lock().unwrap_or_else(|e| e.into_inner());
        if runs.iter().any(|r| r == sid) { return; }   // already answering: the next tool call carries new messages
        runs.push(sid.to_string());
    }
    let done = |sid: &str| { RUNS.lock().unwrap_or_else(|e| e.into_inner()).retain(|r| r != sid); };
    let job = {
        let mut g = lk(&st);
        let Some(s) = g.sessions.get_mut(sid).and_then(|s| s.as_object_mut()) else { drop(g); return done(sid) };
        if !resumable(s) { drop(g); return done(sid); }
        let (mut texts, mut ids) = (Vec::new(), Vec::new());
        if let Some(inbox) = s.get_mut("inbox").and_then(|i| i.as_array_mut()) {
            for m in inbox.iter_mut() {
                if m.get("state").and_then(|x| x.as_str()) == Some("queued") {
                    texts.push(m.get("text").and_then(|x| x.as_str()).unwrap_or("").to_string());
                    ids.push(m.get("id").cloned().unwrap_or(Value::Null));
                    m["state"] = json!("delivered");
                    m["sent_at"] = json!(now_ms());
                }
            }
        }
        if texts.is_empty() { drop(g); return done(sid); }
        let text = texts.join("\n\n");
        let cwd = s.get("cwd").and_then(|x| x.as_str()).map(PathBuf::from).filter(|p| p.is_dir());
        let mode = s.get("perm_mode").and_then(|x| x.as_str()).filter(|m| ["default", "acceptEdits", "plan", "bypassPermissions", "dontAsk", "auto"].contains(m)).map(String::from);
        mark_prompted(s, &text);
        s.insert("short".into(), json!("Sinasagot ang mensahe mo."));
        g.bump(); g.save();
        (text, cwd, mode, ids)
    };
    let sid = sid.to_string();
    std::thread::spawn(move || {
        let (text, cwd, mode, ids) = job;
        let mut c = std::process::Command::new(&bin);
        c.args(["-p", "--resume", &sid, "--output-format", "json"]);
        if let Some(m) = &mode { c.args(["--permission-mode", m]); }
        if let Some(d) = &cwd { c.current_dir(d); }
        c.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
        #[cfg(windows)] { use std::os::windows::process::CommandExt; c.creation_flags(0x0800_0000); }
        let note = match c.spawn() {
            Err(e) => Some(format!("Hindi mapatakbo ang Claude Code sa PC: {e}")),
            Ok(mut child) => {
                if let Some(mut i) = child.stdin.take() { use std::io::Write; let _ = i.write_all(text.as_bytes()); }
                match child.wait_with_output() {
                    Err(e) => Some(format!("Naputol ang sagot: {e}")),
                    Ok(out) => {
                        let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
                        let err = v.get("is_error").and_then(|x| x.as_bool()).unwrap_or(!out.status.success());
                        if !err { None } else {
                            let why = v.get("result").and_then(|x| x.as_str()).map(String::from)
                                .unwrap_or_else(|| String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("").to_string());
                            Some(format!("Hindi naipasa sa session: {}", clip(&why, 200)))
                        }
                    }
                }
            }
        };
        {
            let mut g = lk(&st);
            if let Some(s) = g.sessions.get_mut(&sid).and_then(|s| s.as_object_mut()) {
                if let Some(n) = &note {
                    push_convo(s, "sys", n, 400);
                    if let Some(inbox) = s.get_mut("inbox").and_then(|i| i.as_array_mut()) {
                        for m in inbox.iter_mut() { if ids.contains(m.get("id").unwrap_or(&Value::Null)) { m["state"] = json!("failed"); } }
                    }
                    if s.get("status").and_then(|x| x.as_str()) == Some("working") { s.insert("status".into(), json!("idle")); s.insert("short".into(), json!("")); }
                }
            }
            g.bump(); g.save();
        }
        RUNS.lock().unwrap_or_else(|e| e.into_inner()).retain(|r| r != &sid);
        // messages sent while it answered and not picked up by a tool call
        deliver_soon(&sid);
    });
}
