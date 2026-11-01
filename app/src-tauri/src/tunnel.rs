//! "Malayo mode": lets the paired phone reach Dev Room from anywhere (mobile data), with no extra app
//! and no account. Dev Room runs Cloudflare's free quick tunnel (cloudflared) to a second, phone-only
//! port, and posts the tunnel's current address to a private ntfy.sh topic that only the paired phone
//! can work out (a hash of the pairing token). Everything through the tunnel needs the pairing token.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The loopback port only the tunnel talks to; every request on it counts as a phone (token required).
pub const PORT: u16 = 4318;

struct Tun { status: String, url: Option<String>, child: Option<Child> }
static T: Mutex<Tun> = Mutex::new(Tun { status: String::new(), url: None, child: None });
static ON: AtomicBool = AtomicBool::new(false);
static RUNNING: AtomicBool = AtomicBool::new(false);

fn set(status: &str, url: Option<String>) { if let Ok(mut t) = T.lock() { t.status = status.into(); t.url = url; } }

/// The tunnel's host name right now (for the Host and Origin checks on the tunnel port).
pub fn host() -> Option<String> { T.lock().ok()?.url.as_ref().map(|u| u.trim_start_matches("https://").to_string()) }

pub fn info() -> Value {
    let t = T.lock();
    let (status, url) = t.map(|t| (t.status.clone(), t.url.clone())).unwrap_or_default();
    json!({"on": ON.load(Ordering::Relaxed), "status": status, "ready": url.is_some()})
}

/// The private ntfy topic for this pairing: anyone without the token cannot work it out.
pub fn topic(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let h = Sha256::digest(format!("devroom-relay:{token}").as_bytes());
    format!("devroom-{}", h.iter().take(16).map(|b| format!("{b:02x}")).collect::<String>())
}

fn bin_path() -> std::path::PathBuf {
    crate::server::data_dir().join("bin").join(if cfg!(windows) { "cloudflared.exe" } else { "cloudflared" })
}

fn download() -> Result<(), String> {
    let name = if cfg!(windows) { "cloudflared-windows-amd64.exe" } else { "cloudflared-linux-amd64" };
    let url = format!("https://github.com/cloudflare/cloudflared/releases/latest/download/{name}");
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(300)).build();
    let resp = agent.get(&url).call().map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    resp.into_reader().take(150 * 1024 * 1024).read_to_end(&mut buf).map_err(|e| e.to_string())?;
    let ok = if cfg!(windows) { buf.starts_with(b"MZ") } else { buf.starts_with(b"\x7fELF") };
    if buf.len() < 5 * 1024 * 1024 || !ok { return Err("mali ang na-download".into()); }
    let p = bin_path();
    std::fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    let tmp = p.with_extension("part");
    std::fs::write(&tmp, &buf).map_err(|e| e.to_string())?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)); }
    std::fs::rename(&tmp, &p).map_err(|e| e.to_string())
}

fn publish(topic: &str, url: &str) -> bool {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build();
    agent.post(&format!("https://ntfy.sh/{topic}")).set("Cache", "yes").send_string(url).is_ok()
}

fn spawn() -> Result<Child, String> {
    let mut c = Command::new(bin_path());
    c.args(["tunnel", "--no-autoupdate", "--url", &format!("http://127.0.0.1:{PORT}")])
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    #[cfg(windows)] { use std::os::windows::process::CommandExt; c.creation_flags(0x0800_0000); }   // no console window
    c.spawn().map_err(|e| e.to_string())
}

fn find_url(line: &str) -> Option<String> {
    let i = line.find("https://")?;
    let u: String = line[i..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.' || *c == '/' || *c == ':').collect();
    let host = u.trim_start_matches("https://");
    (host.ends_with(".trycloudflare.com") && !host.starts_with("api.") && host.matches('.').count() == 2).then_some(u)
}

/// Turn Malayo mode on or off. `token` is the pairing token (its hash names the ntfy topic).
pub fn set_enabled(on: bool, token: String) {
    ON.store(on, Ordering::Relaxed);
    if !on {
        if let Ok(mut t) = T.lock() { if let Some(mut c) = t.child.take() { let _ = c.kill(); let _ = c.wait(); } t.url = None; t.status = String::new(); }
        return;
    }
    if RUNNING.swap(true, Ordering::SeqCst) { return; }
    std::thread::spawn(move || {
        let topic = topic(&token);
        let mut wait = 5;
        while ON.load(Ordering::Relaxed) {
            if !bin_path().exists() {
                set("Inihahanda ang Malayo mode (isang beses lang)…", None);
                if let Err(e) = download() { set(&format!("Hindi ma-download ang kailangan: {e}. Susubukan ulit."), None); sleep_while_on(60); continue; }
            }
            set("Kumokonekta…", None);
            let mut child = match spawn() { Ok(c) => c, Err(e) => { set(&format!("Hindi mapatakbo: {e}. Susubukan ulit."), None); sleep_while_on(wait); wait = (wait * 2).min(300); continue; } };
            let err = child.stderr.take();
            if let Ok(mut t) = T.lock() { t.child = Some(child); }
            let mut published = None::<Instant>;
            if let Some(err) = err {
                for line in BufReader::new(err).lines().map_while(Result::ok) {
                    if let Some(u) = find_url(&line) {
                        set("Handa na: maaabot ka ng phone kahit malayo.", Some(u.clone()));
                        wait = 5;
                        if publish(&topic, &u) { published = Some(Instant::now()); }
                        // keep the address fresh on the relay (it forgets after 12 hours)
                        let (tp, uu) = (topic.clone(), u.clone());
                        std::thread::spawn(move || {
                            loop {
                                std::thread::sleep(Duration::from_secs(60));
                                if host().map(|h| !uu.ends_with(&h)).unwrap_or(true) { break; }
                                if published.map(|p| p.elapsed() > Duration::from_secs(4 * 3600)).unwrap_or(true) && publish(&tp, &uu) { published = Some(Instant::now()); }
                            }
                        });
                    }
                }
            }
            // cloudflared stopped (network change, sleep, update): start again
            if let Ok(mut t) = T.lock() { if let Some(mut c) = t.child.take() { let _ = c.kill(); let _ = c.wait(); } t.url = None; }
            if ON.load(Ordering::Relaxed) { set("Naputol ang Malayo mode; kumokonekta ulit…", None); sleep_while_on(wait); wait = (wait * 2).min(300); }
        }
        RUNNING.store(false, Ordering::SeqCst);
    });
}

fn sleep_while_on(secs: u64) { for _ in 0..secs { if !ON.load(Ordering::Relaxed) { return; } std::thread::sleep(Duration::from_secs(1)); } }

/// Stop the tunnel when Dev Room quits (so no stray cloudflared stays behind).
pub fn shutdown() { if let Ok(mut t) = T.lock() { if let Some(mut c) = t.child.take() { let _ = c.kill(); } } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_the_tunnel_address() {
        assert_eq!(find_url("2026-11-01T10:00:00Z INF |  https://quiet-river-1234.trycloudflare.com                                   |"),
            Some("https://quiet-river-1234.trycloudflare.com".into()));
        assert_eq!(find_url("INF Requesting new quick Tunnel on trycloudflare.com..."), None);
        assert_eq!(find_url("see https://www.cloudflare.com/website-terms/"), None);
        assert_eq!(find_url("https://api.trycloudflare.com/tunnel"), None);
        assert_eq!(find_url("https://a.b.evil.trycloudflare.com"), None);
    }
    #[test]
    fn topic_is_private_and_stable() {
        let t = topic("0123456789abcdef0123456789abcdef");
        assert_eq!(t, topic("0123456789abcdef0123456789abcdef"));
        assert!(t.starts_with("devroom-") && t.len() == 8 + 32);
        assert!(!t.contains("0123456789abcdef"));
        println!("{t}");
    }
}
