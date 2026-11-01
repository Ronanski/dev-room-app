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
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

pub const DEFAULT_PORT: u16 = 4317;
const INDEX_HTML: &str = include_str!("../../ui/index.html");
const HOOK_EVENTS: [&str; 6] = ["SessionStart", "UserPromptSubmit", "PreToolUse", "Notification", "Stop", "SessionEnd"];
const MAX_BODY: usize = 4 * 1024 * 1024;

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
}

impl State {
    fn bump(&mut self) { self.version += 1; }

    fn save(&self) {
        let _ = fs::create_dir_all(&self.dir);
        let data = json!({"sessions": self.sessions, "team": self.team, "token": self.token});
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
            }
        }
        if st.token.len() < 16 { st.token = format!("{:016x}{:016x}", rand::random::<u64>(), rand::random::<u64>()); }
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

fn hook_command(port: u16) -> String {
    // curl.exe ships with Windows 10 and 11; `|| exit 0` keeps Claude quiet when Dev Room is closed.
    format!("curl -s -m 2 -X POST -H \"Content-Type: application/json\" --data-binary @- http://127.0.0.1:{port}/hook || exit 0")
}

fn is_ours(entry: &Value) -> bool {
    entry.get("hooks").and_then(|h| h.as_array()).map(|hs| hs.iter().any(|h|
        h.get("command").and_then(|c| c.as_str()).map(|c| c.contains("127.0.0.1:") && c.contains("/hook")).unwrap_or(false)
    )).unwrap_or(false)
}

pub fn hooks_installed(port: u16) -> bool {
    let Ok(text) = fs::read_to_string(claude_settings_path()) else { return false };
    let Ok(v) = serde_json::from_str::<Value>(&text) else { return false };
    let cmd = hook_command(port);
    HOOK_EVENTS.iter().all(|ev| v.pointer(&format!("/hooks/{ev}")).and_then(|a| a.as_array()).map(|a| a.iter().any(|e|
        e.get("hooks").and_then(|h| h.as_array()).map(|hs| hs.iter().any(|h| h.get("command").and_then(|c| c.as_str()) == Some(cmd.as_str()))).unwrap_or(false)
    )).unwrap_or(false))
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
    let cmd = hook_command(port);
    for ev in HOOK_EVENTS {
        let list = hooks.as_object_mut().unwrap().entry(ev).or_insert_with(|| json!([]));
        if !list.is_array() { *list = json!([]); }
        let arr = list.as_array_mut().unwrap();
        arr.retain(|e| !is_ours(e));
        let mut entry = json!({"hooks": [{"type": "command", "command": cmd, "timeout": 3}]});
        if ev == "PreToolUse" { entry["matcher"] = json!("*"); }
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
            if let Some(v) = read_body(&mut req) { apply_hook(&mut st.lock().unwrap(), &v); }
            respond_json(req, 200, &json!({}));
        }
        (Method::Get, "/api/state") => {
            if !token_ok { return respond_json(req, 401, &json!({"error": "pairing"})); }
            let since: u64 = q.get("v").and_then(|v| v.parse().ok()).unwrap_or(0);
            let s = st.lock().unwrap();
            if since == s.version { drop(s); let _ = req.respond(Response::empty(204).with_header(header("Access-Control-Allow-Origin", "*"))); return; }
            let names = s.team.get("names").cloned().unwrap_or(json!({}));
            let mut list: Vec<Value> = s.sessions.values().map(|x| record(x, &names)).collect();
            list.sort_by_key(|r| std::cmp::Reverse(r.get("updated_at").and_then(|t| t.as_u64()).unwrap_or(0)));
            let lan = lan_ip().map(|ip| format!("http://{ip}:{port}/"));
            let body = json!({"version": s.version, "sessions": list, "team": s.team, "local": local,
                "hooks_installed": if local { json!(hooks_installed(port)) } else { Value::Null },
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
            crate::updater::restart();
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
pub fn start(port: u16) -> Option<u16> {
    let mut server = None;
    for _ in 0..40 {
        match Server::http(("0.0.0.0", port)) { Ok(s) => { server = Some(s); break } Err(_) => std::thread::sleep(std::time::Duration::from_millis(250)) }
    }
    let server = server?;
    let st = Arc::new(Mutex::new(State::load(data_dir())));
    std::thread::spawn(move || {
        for req in server.incoming_requests() { handle(req, &st, port); }
    });
    Some(port)
}
