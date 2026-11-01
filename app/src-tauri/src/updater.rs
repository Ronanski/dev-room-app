//! In-app auto update from the latest GitHub release (published from main).
//!
//! Every few minutes the app reads `version.json` from the release.
//! - A newer UI (`index.html`) with the same API version is cached in %APPDATA%\DevRoom\ui
//!   and served right away; open windows and phones reload themselves.
//! - A newer `DevRoom.exe` is downloaded next to the running one and swapped in
//!   (running exe -> DevRoom.old.exe). The app restarts by itself while it sits in the tray;
//!   an open window counts down after a minute without input, with a button to wait.

use serde_json::{json, Value};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

pub const BUILD: u64 = match option_env!("DEVROOM_BUILD") {
    Some(b) => parse_u64(b),
    None => 0,
};
pub const VERSION: &str = match option_env!("DEVROOM_VERSION") {
    Some(v) => v,
    None => "dev",
};
pub const API: u64 = parse_u64(include_str!("../../api-version.txt"));
const BASE: &str = "https://github.com/Ronanski/dev-room-app/releases/latest/download/";

const fn parse_u64(s: &str) -> u64 {
    let b = s.as_bytes();
    let (mut i, mut n) = (0, 0u64);
    while i < b.len() {
        if b[i] >= b'0' && b[i] <= b'9' { n = n * 10 + (b[i] - b'0') as u64; }
        i += 1;
    }
    n
}

struct Upd { ui: Option<(u64, String)>, exe_ready: Option<u64>, note: String, latest: Option<Value> }
static STATE: Mutex<Upd> = Mutex::new(Upd { ui: None, exe_ready: None, note: String::new(), latest: None });

/// The newest release info seen (build, api, apk), passed on to the Android app.
pub fn latest() -> Value { STATE.lock().ok().and_then(|s| s.latest.clone()).unwrap_or(Value::Null) }

fn ui_dir() -> PathBuf { crate::server::data_dir().join("ui") }

pub fn ui_html() -> Option<String> { STATE.lock().ok()?.ui.as_ref().map(|(_, h)| h.clone()) }
pub fn ui_build() -> u64 { STATE.lock().ok().and_then(|s| s.ui.as_ref().map(|(b, _)| *b)).unwrap_or(BUILD) }
pub fn exe_ready() -> bool { STATE.lock().map(|s| s.exe_ready.is_some()).unwrap_or(false) }
pub fn note() -> String { STATE.lock().map(|s| s.note.clone()).unwrap_or_default() }

/// Whether the window is hidden in the tray (set by main), so an update can restart without bothering anyone.
pub static HIDDEN: AtomicBool = AtomicBool::new(false);
pub fn set_hidden(h: bool) { HIDDEN.store(h, Ordering::Relaxed); }

fn set_note(n: &str) { if let Ok(mut s) = STATE.lock() { s.note = n.to_string(); } }

fn get(url: &str, limit: u64) -> Result<Vec<u8>, String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(60)).build();
    let resp = agent.get(url).call().map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    resp.into_reader().take(limit).read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

/// Load a cached UI that is newer than the one built into this exe.
fn load_cache() {
    let Ok(meta) = fs::read(ui_dir().join("ui.json")) else { return };
    let Ok(m) = serde_json::from_slice::<Value>(&meta) else { return };
    let (b, api) = (m["build"].as_u64().unwrap_or(0), m["api"].as_u64().unwrap_or(0));
    if b > BUILD && api == API {
        if let Ok(html) = fs::read_to_string(ui_dir().join("index.html")) {
            if let Ok(mut s) = STATE.lock() { s.ui = Some((b, html)); }
        }
    }
}

fn check_once() -> Result<(), String> {
    let v: Value = serde_json::from_slice(&get(&format!("{BASE}version.json"), 64 * 1024)?).map_err(|e| e.to_string())?;
    let (build, api) = (v["build"].as_u64().unwrap_or(0), v["api"].as_u64().unwrap_or(0));
    if let Ok(mut s) = STATE.lock() { s.latest = Some(v.clone()); }
    if build <= BUILD { return Ok(()); }
    // 1. UI hot update (only when the server API it needs is the same as ours)
    if api == API && ui_build() < build {
        let html = String::from_utf8(get(&format!("{BASE}index.html"), 8 * 1024 * 1024)?).map_err(|e| e.to_string())?;
        if html.contains("Dev Room") {
            let _ = fs::create_dir_all(ui_dir());
            let _ = fs::write(ui_dir().join("index.html"), &html);
            let _ = fs::write(ui_dir().join("ui.json"), json!({"build": build, "api": api}).to_string());
            if let Ok(mut s) = STATE.lock() { s.ui = Some((build, html)); }
        }
    }
    // 2. Exe swap, done once per new build; takes effect after a restart
    if STATE.lock().map(|s| s.exe_ready.map(|b| b >= build).unwrap_or(false)).unwrap_or(false) { return Ok(()); }
    if !cfg!(windows) { return Ok(()); }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("no exe dir")?.to_path_buf();
    let bytes = get(&format!("{BASE}DevRoom.exe"), 200 * 1024 * 1024)?;
    if bytes.len() < 1024 * 1024 || &bytes[..2] != b"MZ" { return Err("download looks wrong".into()); }
    let fresh = dir.join("DevRoom.new.exe");
    let old = dir.join("DevRoom.old.exe");
    fs::write(&fresh, &bytes).map_err(|e| format!("Hindi maisulat ang bagong app sa {}: {e}", dir.display()))?;
    let _ = fs::remove_file(&old);
    fs::rename(&exe, &old).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(&fresh, &exe) { let _ = fs::rename(&old, &exe); return Err(e.to_string()); }
    if let Ok(mut s) = STATE.lock() { s.exe_ready = Some(build); }
    Ok(())
}

pub fn start() {
    load_cache();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() { let _ = fs::remove_file(d.join("DevRoom.old.exe")); }
    }
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(15));
        loop {
            match check_once() { Ok(()) => set_note(""), Err(e) => set_note(&format!("Hindi nakapag-check ng update: {e}")) }
            for _ in 0..18 {
                if exe_ready() && HIDDEN.load(Ordering::Relaxed) { restart(true); return; }
                std::thread::sleep(Duration::from_secs(10));
            }
        }
    });
}

/// Start the (already swapped-in) exe and quit this one. The new copy waits for the port.
/// `hidden` keeps the new copy in the tray, like the one it replaces.
pub fn restart(hidden: bool) {
    let hidden = hidden || HIDDEN.load(Ordering::Relaxed);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        if let Ok(exe) = std::env::current_exe() {
            let mut cmd = std::process::Command::new(exe);
            if hidden { cmd.arg("--tray"); }
            let _ = cmd.spawn();
        }
        std::process::exit(0);
    });
}
