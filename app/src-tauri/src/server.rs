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
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

pub const DEFAULT_PORT: u16 = 4317;
const INDEX_HTML: &str = include_str!("../../ui/index.html");
const HOOK_EVENTS: [&str; 8] = ["SessionStart", "UserPromptSubmit", "PreToolUse", "PostToolUse", "SubagentStop", "Notification", "Stop", "SessionEnd"];
const MAX_BODY: usize = 4 * 1024 * 1024;

/// Set by the app: opens the Dev Room window (used when DevRoom.exe is started a second time).
pub static ON_SHOW: std::sync::OnceLock<Box<dyn Fn() + Send + Sync>> = std::sync::OnceLock::new();

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
    /// "auto" (wait for phone messages while a phone has Dev Room open), "on" or "off".
    away: String,
    /// Last time a phone or tablet had the office on screen (background watchers do not count).
    remote_seen: u64,
}

impl State {
    fn bump(&mut self) { self.version += 1; }

    fn save(&self) {
        let _ = fs::create_dir_all(&self.dir);
        let data = json!({"sessions": self.sessions, "team": self.team, "token": self.token, "away": self.away});
        let _ = fs::write(self.dir.join("state.json"), serde_json::to_vec_pretty(&data).unwrap_or_default());
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
                if let Some(a) = v.get("away").and_then(|t| t.as_str()) { st.away = a.to_string(); }
            }
        }
        if st.token.len() < 16 { st.token = format!("{:016x}{:016x}", rand::random::<u64>(), rand::random::<u64>()); }
        if !["auto", "on", "off"].contains(&st.away.as_str()) { st.away = "auto".into(); }
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
    format!("curl -s -m {max} -X POST -H \"Content-Type: application/json\" --data-binary @- http://127.0.0.1:{port}/hook || exit 0")
}
fn hook_timeout(ev: &str) -> u64 { if ev == "Stop" { LISTEN_SECS + 60 } else { 3 } }

fn is_ours(entry: &Value) -> bool {
    entry.get("hooks").and_then(|h| h.as_array()).map(|hs| hs.iter().any(|h|
        h.get("command").and_then(|c| c.as_str()).map(|c| c.contains("127.0.0.1:") && c.contains("/hook")).unwrap_or(false)
    )).unwrap_or(false)
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
        arr.retain(|e| !is_ours(e));
        let mut entry = json!({"hooks": [{"type": "command", "command": cmd, "timeout": hook_timeout(ev)}]});
        if ev == "PreToolUse" || ev == "PostToolUse" { entry["matcher"] = json!("*"); }
        arr.push(entry);
    }
    if let Some(dir) = path.parent() { fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
    fs::write(&path, serde_json::to_vec_pretty(&root).unwrap()).map_err(|e| format!("Hindi maisulat ang {}: {e}", path.display()))?;
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

fn apply_hook(st: &mut State, v: &Value) {
    let ev = v.get("hook_event_name").and_then(|x| x.as_str()).unwrap_or("");
    let Some(sid) = v.get("session_id").and_then(|x| x.as_str()).map(String::from) else { return };
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
    let transcript = v.get("transcript_path").and_then(|x| x.as_str()).map(String::from);
    if let Some(t) = &transcript {
        s.insert("transcript".into(), json!(t));
        // Pick up the session's name (/rename or summary) once, the first time we see its transcript.
        if ev != "Stop" && !s.get("title_scanned").and_then(|x| x.as_bool()).unwrap_or(false) {
            let (_, title) = read_transcript(t, true);
            s.insert("title_scanned".into(), json!(true));
            if let Some(title) = title { s.insert("auto_title".into(), json!(title)); }
        }
    }
    s.insert("updated_at".into(), json!(now));
    s.insert("last_event".into(), json!(ev));
    s.remove("live_guess");
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
            let scanned = s.get("title_scanned").and_then(|x| x.as_bool()).unwrap_or(false);
            if let Some(t) = &transcript {
                let (tx, title) = read_transcript(t, !scanned);
                s.insert("title_scanned".into(), json!(true));
                if text.is_none() { text = tx; }
                if let Some(title) = title { s.insert("auto_title".into(), json!(title)); }
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
            m["state"] = json!("delivered");
            m["delivered_at"] = json!(now_ms());
        }
    }
    if ids.is_empty() { return None; }
    Some((ids, format!("Mensahe mula kay Boss (ipinadala mula sa Dev Room app):\n\n{}", texts.join("\n\n"))))
}

/// Put messages back in the queue when they could not be handed over (the hook was cancelled).
fn requeue(st: &Mutex<State>, sid: &str, ids: &[u64]) {
    let mut g = st.lock().unwrap();
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

fn wants_listen(g: &State, window_ms: u64) -> bool {
    match g.away.as_str() { "on" => true, "off" => false, _ => now_ms().saturating_sub(g.remote_seen) < window_ms }
}

/// Answer a hook. PostToolUse and UserPromptSubmit carry queued messages as extra context. Stop hands them
/// over as a reason to continue, and while a phone has Dev Room open it waits for one.
fn hook_reply(req: Request, st: &Mutex<State>, v: &Value) {
    let ev = v.get("hook_event_name").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let sid = v.get("session_id").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let mut g = st.lock().unwrap();
    apply_hook(&mut g, v);
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
    let got = loop {
        let listen = wants_listen(&g, if first { 120_000 } else { 600_000 });
        let Some(s) = g.sessions.get_mut(&sid).and_then(|s| s.as_object_mut()) else { break None };
        if first { gen = s.get("listen_gen").and_then(|x| x.as_u64()).unwrap_or(0); }
        else if s.get("listen_gen").and_then(|x| x.as_u64()).unwrap_or(0) != gen { break None; }   // the session moved on
        if let Some((ids, text)) = take_queued(s) { mark_prompted(s, &text); g.bump(); g.save(); break Some((ids, text)); }
        if !listen || now_ms() > deadline {
            if !first { s.insert("listening".into(), json!(false)); g.bump(); }
            break None;
        }
        if first { s.insert("listening".into(), json!(true)); g.bump(); first = false; }
        drop(g);
        std::thread::sleep(std::time::Duration::from_millis(400));
        g = st.lock().unwrap();
    };
    drop(g);
    match got {
        Some((ids, text)) => {
            let out = json!({"decision": "block", "reason": text});
            if req.respond(json_response(200, &out)).is_err() {
                requeue(st, &sid, &ids);
                let mut g = st.lock().unwrap();
                if let Some(s) = g.sessions.get_mut(&sid).and_then(|s| s.as_object_mut()) { s.insert("status".into(), json!("idle")); }
                g.bump();
            }
        }
        None => respond_json(req, 200, &json!({})),
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
    if arr.last().map(|m| m.get("text").and_then(|x| x.as_str()) == Some(t.as_str())).unwrap_or(false) { return; }
    arr.push(json!({"role": role, "text": t, "at": now_ms()}));
    if arr.len() > 40 { let n = arr.len() - 40; arr.drain(..n); }
    let n = s.get("convo_n").and_then(|x| x.as_u64()).unwrap_or(0) + 1;
    s.insert("convo_n".into(), json!(n));
}

fn record(s: &Value, names: &Value) -> Value {
    let id = s.get("id").and_then(|x| x.as_str()).unwrap_or("");
    let cwd = s.get("cwd").and_then(|x| x.as_str()).unwrap_or("");
    let auto = s.get("auto_title").and_then(|x| x.as_str()).filter(|t| !t.is_empty()).map(String::from);
    let title = names.get(id).and_then(|x| x.as_str()).filter(|t| !t.trim().is_empty()).map(String::from)
        .or(auto).unwrap_or_else(|| if cwd.is_empty() { format!("Session {}", &id[..id.len().min(8)]) } else { base_name(cwd) });
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

fn handle(mut req: Request, st: &Arc<Mutex<State>>, port: u16) {
    let local = is_loopback(req.remote_addr());
    let url = req.url().to_string();
    let (path, query) = url.split_once('?').unwrap_or((url.as_str(), ""));
    let path = path.to_string();
    let q: HashMap<String, String> = query.split('&').filter_map(|kv| kv.split_once('=')).map(|(k, v)| (k.to_string(), v.to_string())).collect();
    let token_ok = local || {
        let token = st.lock().unwrap().token.clone();
        let given = req.headers().iter().find(|h| h.field.equiv("X-DevRoom-Token")).map(|h| h.value.as_str().to_string())
            .or_else(|| q.get("t").cloned()).unwrap_or_default();
        given == token
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
            let mut s = st.lock().unwrap();
            if !local && q.get("bg").map(|b| b != "1").unwrap_or(true) { s.remote_seen = now_ms(); }
            if since == s.version { drop(s); let _ = req.respond(Response::empty(204).with_header(header("Access-Control-Allow-Origin", "*"))); return; }
            let names = s.team.get("names").cloned().unwrap_or(json!({}));
            let mut list: Vec<Value> = s.sessions.values().map(|x| record(x, &names)).collect();
            list.sort_by_key(|r| std::cmp::Reverse(r.get("updated_at").and_then(|t| t.as_u64()).unwrap_or(0)));
            let lan = lan_ip().map(|ip| format!("http://{ip}:{port}/"));
            let body = json!({"version": s.version, "sessions": list, "team": s.team, "local": local,
                "hooks_installed": if local { json!(hooks_installed(port)) } else { Value::Null }, "away": s.away,
                "settings_path": if local { json!(claude_settings_path().display().to_string()) } else { Value::Null },
                "lan_url": lan, "ts_url": tailscale_ip().map(|ip| format!("http://{ip}:{port}/")),
                "app_build": crate::updater::BUILD, "app_version": crate::updater::VERSION, "ui_build": crate::updater::ui_build(),
                "update_ready": crate::updater::exe_ready(), "update_note": crate::updater::note(),
                "latest": crate::updater::latest()});
            drop(s);
            respond_json(req, 200, &body);
        }
        (Method::Post, "/api/team") => {
            if !token_ok { return respond_json(req, 401, &json!({"error": "pairing"})); }
            let Some(v) = read_body(&mut req) else { return respond_json(req, 400, &json!({"error": "bad json"})) };
            let mut s = st.lock().unwrap();
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
            let mut g = st.lock().unwrap();
            let Some(s) = g.sessions.get_mut(&sid).and_then(|s| s.as_object_mut()) else { drop(g); return respond_json(req, 404, &json!({"error": "no session"})) };
            let id = now_ms();
            let inbox = s.entry("inbox").or_insert_with(|| json!([]));
            if !inbox.is_array() { *inbox = json!([]); }
            let arr = inbox.as_array_mut().unwrap();
            arr.push(json!({"id": id, "text": text, "at": id, "state": "queued"}));
            if arr.len() > 30 { let n = arr.len() - 30; arr.drain(..n); }
            g.bump(); g.save(); drop(g);
            respond_json(req, 200, &json!({"ok": true, "id": id}));
        }
        (Method::Get, "/api/convo") => {
            // Conversations are fetched apart from /api/state, only when a session's convo_n changes.
            if !token_ok { return respond_json(req, 401, &json!({"error": "pairing"})); }
            let g = st.lock().unwrap();
            let all: Map<String, Value> = g.sessions.iter().map(|(k, s)| (k.clone(), s.get("convo").cloned().unwrap_or(json!([])))).collect();
            drop(g);
            respond_json(req, 200, &Value::Object(all));
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
            let mut g = st.lock().unwrap();
            g.away = mode.to_string(); g.bump(); g.save(); drop(g);
            respond_json(req, 200, &json!({"ok": true}));
        }
        (Method::Post, "/api/install-hooks") => {
            if !local { return respond_json(req, 403, &json!({"error": "Sa PC lang puwedeng i-set up ang hooks."})); }
            match install_hooks(port) {
                Ok(v) => { st.lock().unwrap().bump(); respond_json(req, 200, &v) }
                Err(e) => respond_json(req, 500, &json!({"error": e})),
            }
        }
        (Method::Get, "/pair.svg") => {
            if !local { return respond_json(req, 403, &json!({"error": "loopback only"})); }
            let token = st.lock().unwrap().token.clone();
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
            let token = st.lock().unwrap().token.clone();
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
    let now = now_ms();
    let mut s = st.lock().unwrap();
    let mut changed = false;
    for v in s.sessions.values_mut() {
        let Some(o) = v.as_object_mut() else { continue };
        let Some(t) = o.get("transcript").and_then(|x| x.as_str()).map(String::from) else { continue };
        let status = o.get("status").and_then(|x| x.as_str()).unwrap_or("idle").to_string();
        let hook_at = o.get("updated_at").and_then(|x| x.as_u64()).unwrap_or(0);
        let guess = o.get("live_guess").and_then(|x| x.as_bool()).unwrap_or(false);
        if status == "idle" {
            let w = latest_write(Path::new(&t));
            if w > hook_at + 5000 && now.saturating_sub(w) < 15_000 {
                o.insert("status".into(), json!("working"));
                o.insert("short".into(), json!("Tuloy ang trabaho."));
                o.insert("detail".into(), json!("May ginagawa pa sa background."));
                o.insert("live_guess".into(), json!(true));
                o.insert("live_at".into(), json!(now));
                changed = true;
            }
        } else if guess && status == "working" {
            let w = latest_write(Path::new(&t)).max(hook_at);
            if now.saturating_sub(w) > 60_000 {
                o.insert("status".into(), json!("idle"));
                o.insert("short".into(), json!(""));
                o.insert("detail".into(), json!(""));
                o.remove("live_guess");
                changed = true;
            }
        }
    }
    if changed { s.bump(); }
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
    let live = st.clone();
    std::thread::spawn(move || loop { std::thread::sleep(std::time::Duration::from_secs(3)); liveness(&live); });
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let st = st.clone();
            std::thread::spawn(move || handle(req, &st, port));
        }
    });
    Some(port)
}
