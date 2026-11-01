#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
mod tunnel;
mod updater;

use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

/// Run `reg.exe` without flashing a console window.
fn reg(args: &[&str]) -> bool {
    let mut cmd = std::process::Command::new("reg");
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

fn autostart_enabled() -> bool { cfg!(windows) && reg(&["query", RUN_KEY, "/v", "DevRoom"]) }

fn set_autostart(on: bool) {
    if !cfg!(windows) { return; }
    if on {
        if let Ok(exe) = std::env::current_exe() {
            let cmd = format!("\"{}\" --tray", exe.display());
            reg(&["add", RUN_KEY, "/v", "DevRoom", "/t", "REG_SZ", "/d", &cmd, "/f"]);
        }
    } else {
        reg(&["delete", RUN_KEY, "/v", "DevRoom", "/f"]);
    }
}

/// The window is created on demand and destroyed when closed, so a closed Dev Room costs almost nothing:
/// only the small server keeps running in the tray for phones, tablets and the hooks.
/// Its URL carries a per-launch key, so only this window can drive the embedded Claude view.
/// It opens where you left it: same place, same size, maximized if it was. `minimized` brings it back
/// minimized on the taskbar (after an update restart), without flashing on screen first.
fn make_window(app: &AppHandle, port: u16, minimized: bool) {
    let url = format!("http://127.0.0.1:{port}/?app={}", server::app_key()).parse().expect("valid url");
    let g = updater::geom();
    let mut b = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
        .title("Dev Room")
        .inner_size(g.map(|g| g.w).unwrap_or(1440.0), g.map(|g| g.h).unwrap_or(900.0))
        .min_inner_size(820.0, 560.0)
        // a minimized start gets its maximized state from show_minimized (a hidden window cannot maximize)
        .maximized(!minimized && g.map(|g| g.max).unwrap_or(false))
        .visible(!minimized);
    // only a place that is still on one of the screens (a monitor may have been unplugged)
    if let Some(g) = g.filter(|g| g.x.is_finite() && g.y.is_finite() && on_screen(app, g)) { b = b.position(g.x, g.y); } else { b = b.center(); }
    let Ok(w) = b.build() else { return };
    if minimized {
        show_minimized(&w, g.map(|g| g.max).unwrap_or(false));
        updater::set_minimized(true);
    }
}

/// Put the (still hidden) window on the taskbar, minimized, without taking focus; opening it from the
/// taskbar brings it back maximized when it was maximized before.
#[cfg(windows)]
fn show_minimized(w: &tauri::WebviewWindow, max: bool) {
    #[repr(C)] struct Pt { x: i32, y: i32 }
    #[repr(C)] struct Rc { l: i32, t: i32, r: i32, b: i32 }
    #[repr(C)] struct Placement { length: u32, flags: u32, show: u32, min: Pt, max: Pt, normal: Rc }
    #[link(name = "user32")]
    extern "system" {
        fn GetWindowPlacement(hwnd: isize, p: *mut Placement) -> i32;
        fn SetWindowPlacement(hwnd: isize, p: *const Placement) -> i32;
    }
    const SW_SHOWMINNOACTIVE: u32 = 7;
    const WPF_RESTORETOMAXIMIZED: u32 = 2;
    let Ok(h) = w.hwnd() else { let _ = w.minimize(); return };
    let hwnd = h.0 as isize;
    let mut p = Placement { length: std::mem::size_of::<Placement>() as u32, flags: 0, show: 0, min: Pt { x: 0, y: 0 }, max: Pt { x: 0, y: 0 }, normal: Rc { l: 0, t: 0, r: 0, b: 0 } };
    unsafe {
        if GetWindowPlacement(hwnd, &mut p) == 0 { let _ = w.minimize(); return; }
        p.length = std::mem::size_of::<Placement>() as u32;
        p.show = SW_SHOWMINNOACTIVE;
        if max { p.flags |= WPF_RESTORETOMAXIMIZED; }
        if SetWindowPlacement(hwnd, &p) == 0 { let _ = w.minimize(); }
    }
}
#[cfg(not(windows))]
fn show_minimized(w: &tauri::WebviewWindow, _max: bool) { let _ = w.show(); let _ = w.minimize(); }

/// Whether the top-left part of the remembered window (its title bar) would be on a screen.
fn on_screen(app: &AppHandle, g: &updater::Geom) -> bool {
    let Ok(mons) = app.available_monitors() else { return false };
    mons.iter().any(|m| {
        let s = m.scale_factor();
        let (mx, my) = (m.position().x as f64 / s, m.position().y as f64 / s);
        let (mw, mh) = (m.size().width as f64 / s, m.size().height as f64 / s);
        g.x + 60.0 > mx && g.x + 60.0 < mx + mw && g.y + 10.0 >= my && g.y + 10.0 < my + mh
    })
}

/// Remember the window's place as it moves and resizes (the normal size is kept while it is maximized).
fn track_geom(w: &tauri::Window) {
    if w.is_minimized().unwrap_or(false) { return; }
    let max = w.is_maximized().unwrap_or(false);
    let s = w.scale_factor().unwrap_or(1.0);
    let pos = w.outer_position().ok();
    let size = w.inner_size().ok();
    updater::note_geom(|g| {
        g.max = max;
        if max { return; }
        if let Some(p) = pos { g.x = p.x as f64 / s; g.y = p.y as f64 / s; }
        if let Some(z) = size { if z.width > 0 && z.height > 0 { g.w = z.width as f64 / s; g.h = z.height as f64 / s; } }
    });
}

/// Pages the embedded Claude view may show: Claude itself and the sign-in providers it uses.
fn claude_allowed(url: &tauri::Url) -> bool {
    if url.scheme() == "about" { return true; }
    if url.scheme() != "https" { return false; }
    let h = url.host_str().unwrap_or("");
    let under = |d: &str| h == d || h.ends_with(&format!(".{d}"));
    under("claude.ai") || under("anthropic.com") || h == "accounts.google.com" || h == "accounts.youtube.com" || h == "appleid.apple.com"
}

/// Anything else opens in the user's own browser, with its address bar.
fn open_outside(url: &tauri::Url) {
    if !matches!(url.scheme(), "http" | "https") { return; }
    #[cfg(windows)]
    { let _ = std::process::Command::new("explorer").arg(url.as_str()).spawn(); }
}

/// The latest rectangle the UI asked for (None = hidden); applied after a slow first creation too.
static CLAUDE_WANT: std::sync::Mutex<Option<(f64, f64, f64, f64)>> = std::sync::Mutex::new(None);
static CLAUDE_BUSY: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The official Claude Code chat (claude.ai/code, with this PC's sessions through Remote Control),
/// shown as a second webview inside the Dev Room window, over the panel's chat area.
/// Runs on a server thread (creating a child webview from the main thread can deadlock on Windows).
fn claude_view(app: &AppHandle, rect: Option<(f64, f64, f64, f64)>) {
    use tauri::{LogicalPosition, LogicalSize};
    *CLAUDE_WANT.lock().unwrap_or_else(|e| e.into_inner()) = rect;
    let _one_at_a_time = CLAUDE_BUSY.lock().unwrap_or_else(|e| e.into_inner());
    if app.webviews().get("claude").is_none() {
        if rect.is_none() { return; }
        let Some(window) = app.get_window("main") else { return };
        let app2 = app.clone();
        let popups = std::sync::atomic::AtomicU32::new(0);
        let url = "https://claude.ai/code".parse().expect("valid url");
        let (x, y, w, h) = rect.unwrap();
        let builder = tauri::webview::WebviewBuilder::new("claude", tauri::WebviewUrl::External(url))
            .on_navigation(|u| { if claude_allowed(u) { true } else { open_outside(u); false } })
            // sign-in windows (Google, Apple, email links) open inside Dev Room in the same profile; any other link opens in the browser
            .on_new_window(move |url, features| {
                if !claude_allowed(&url) { open_outside(&url); return tauri::webview::NewWindowResponse::Deny; }
                let n = popups.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                match tauri::WebviewWindowBuilder::new(&app2, format!("claude-signin-{n}"), tauri::WebviewUrl::External("about:blank".parse().unwrap()))
                    .window_features(features)
                    .on_navigation(|u| { if claude_allowed(u) { true } else { open_outside(u); false } })
                    .title(format!("Claude · {}", url.host_str().unwrap_or("sign in")))
                    .build()
                {
                    Ok(window) => tauri::webview::NewWindowResponse::Create { window },
                    Err(_) => tauri::webview::NewWindowResponse::Deny,
                }
            });
        if window.add_child(builder, LogicalPosition::new(x, y), LogicalSize::new(w, h)).is_err() { return; }
    }
    // apply whatever was asked last (the UI may have moved or hidden the panel while the view was being created)
    let want = *CLAUDE_WANT.lock().unwrap_or_else(|e| e.into_inner());
    let Some(v) = app.webviews().get("claude").cloned() else { return };
    match want {
        Some((x, y, w, h)) => {
            let _ = v.set_position(LogicalPosition::new(x, y));
            let _ = v.set_size(LogicalSize::new(w, h));
            let _ = v.show();
        }
        None => {
            let _ = v.hide();
            // give the keyboard back to Dev Room
            if let Some(main) = app.webviews().get("main") { let _ = main.set_focus(); }
        }
    }
}

fn show_main(app: &AppHandle, port: u16) {
    updater::set_hidden(false);
    updater::set_minimized(false);
    // a plain Window: once the Claude view is added, the window is no longer a single-webview "WebviewWindow"
    if let Some(w) = app.get_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    } else {
        make_window(app, port, false);
    }
}

fn main() {
    // One Dev Room at a time: starting it again just opens the window of the one that is running.
    if !std::env::args().any(|a| a == "--restarted") {
        let url = format!("http://127.0.0.1:{}/api/show", server::DEFAULT_PORT);
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_millis(4000)).build();
        if agent.post(&url).set("X-DevRoom", "1").call().is_ok() { return; }
    }
    let Some(port) = server::start(server::DEFAULT_PORT) else {
        // another copy holds the port: show its window and leave
        let url = format!("http://127.0.0.1:{}/api/show", server::DEFAULT_PORT);
        let _ = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(5)).build().post(&url).set("X-DevRoom", "1").call();
        return;
    };
    updater::start();
    // Started with Windows: stay in the tray until the user opens the window.
    let hidden = std::env::args().any(|a| a == "--tray");
    updater::set_hidden(hidden);
    // First run: start with Windows (hidden in the tray), so phones work without opening anything.
    let marker = server::data_dir().join("autostart-default");
    if cfg!(windows) && !marker.exists() {
        set_autostart(true);
        let _ = std::fs::create_dir_all(server::data_dir());
        let _ = std::fs::write(&marker, "1");
    }
    let app = tauri::Builder::default()
        .setup(move |app| {
            if !hidden {
                // restarted for an update while minimized: come back minimized
                make_window(app.handle(), port, std::env::args().any(|a| a == "--minimized"));
            }
            let ch = app.handle().clone();
            let _ = server::ON_CLAUDE.set(Box::new(move |rect| claude_view(&ch, rect)));
            let handle = app.handle().clone();
            let _ = server::ON_SHOW.set(Box::new(move || {
                let h = handle.clone();
                let _ = handle.run_on_main_thread(move || show_main(&h, port));
            }));

            // Closing the window keeps Dev Room running in the tray, so phones and tablets stay connected.
            let open = MenuItem::with_id(app, "open", "Buksan ang Dev Room", true, None::<&str>)?;
            let auto = CheckMenuItem::with_id(app, "autostart", "Simulan kasabay ng Windows", true, autostart_enabled(), None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Isara nang tuluyan", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &auto, &quit])?;
            let auto_item = auto.clone();
            let mut tray = TrayIconBuilder::with_id("devroom")
                .tooltip("Dev Room: tumatakbo pa rin para sa phone at tablet mo")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, ev| match ev.id().as_ref() {
                    "open" => show_main(app, port),
                    "autostart" => set_autostart(auto_item.is_checked().unwrap_or(false)),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(move |tray, ev| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                        show_main(tray.app_handle(), port);
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" { return; }
            match event {
                WindowEvent::CloseRequested { .. } => { track_geom(window); updater::save_geom(); }
                WindowEvent::Destroyed => { updater::set_hidden(true); updater::set_minimized(false); updater::save_geom(); }
                // minimizing and restoring show up as resizes and focus changes
                WindowEvent::Resized(_) | WindowEvent::Focused(_) => {
                    updater::set_minimized(window.is_minimized().unwrap_or(false));
                    track_geom(window);
                }
                WindowEvent::Moved(_) => track_geom(window),
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building Dev Room");
    app.run(|_app, event| {
        // The last window closing must not end the app; only "Isara nang tuluyan" (exit with a code) does.
        match event {
            RunEvent::ExitRequested { api, code, .. } => { if code.is_none() { api.prevent_exit(); } }
            // never leave the tunnel helper running after Dev Room quits
            RunEvent::Exit => { updater::save_geom(); tunnel::shutdown(); }
            _ => {}
        }
    });
}
