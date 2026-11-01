#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
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
fn make_window(app: &AppHandle, port: u16) {
    let url = format!("http://127.0.0.1:{port}/").parse().expect("valid url");
    let _ = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
        .title("Dev Room")
        .inner_size(1440.0, 900.0)
        .min_inner_size(820.0, 560.0)
        .build();
}

/// The official Claude Code chat (claude.ai/code, with this PC's sessions through Remote Control) in its own window.
fn show_claude(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("claude") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        return;
    }
    let url = "https://claude.ai/code".parse().expect("valid url");
    let _ = tauri::WebviewWindowBuilder::new(app, "claude", tauri::WebviewUrl::External(url))
        .title("Claude · Dev Room")
        .inner_size(1100.0, 860.0)
        .min_inner_size(420.0, 520.0)
        .build();
}

fn show_main(app: &AppHandle, port: u16) {
    updater::set_hidden(false);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    } else {
        make_window(app, port);
    }
}

fn main() {
    // One Dev Room at a time: starting it again just opens the window of the one that is running.
    if !std::env::args().any(|a| a == "--restarted") {
        let url = format!("http://127.0.0.1:{}/api/show", server::DEFAULT_PORT);
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_millis(4000)).build();
        if agent.post(&url).call().is_ok() { return; }
    }
    let Some(port) = server::start(server::DEFAULT_PORT) else {
        // another copy holds the port: show its window and leave
        let url = format!("http://127.0.0.1:{}/api/show", server::DEFAULT_PORT);
        let _ = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(5)).build().post(&url).call();
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
            if !hidden { make_window(app.handle(), port); }
            let ch = app.handle().clone();
            let _ = server::ON_CLAUDE.set(Box::new(move || {
                let h = ch.clone();
                let _ = ch.run_on_main_thread(move || show_claude(&h));
            }));
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
            if let WindowEvent::Destroyed = event { if window.label() == "main" { updater::set_hidden(true); } }
        })
        .build(tauri::generate_context!())
        .expect("error while building Dev Room");
    app.run(|_app, event| {
        // The last window closing must not end the app; only "Isara nang tuluyan" (exit with a code) does.
        if let RunEvent::ExitRequested { api, code, .. } = event {
            if code.is_none() { api.prevent_exit(); }
        }
    });
}
