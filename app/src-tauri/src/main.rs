#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
mod updater;

use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};

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

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        updater::set_hidden(false);
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn main() {
    // If the port is taken, another Dev Room is already running: just show its UI.
    let port = server::start(server::DEFAULT_PORT).unwrap_or(server::DEFAULT_PORT);
    updater::start();
    // Started with Windows: stay in the tray until the user opens the window.
    let hidden = std::env::args().any(|a| a == "--tray");
    updater::set_hidden(hidden);
    tauri::Builder::default()
        .setup(move |app| {
            let url = format!("http://127.0.0.1:{port}/").parse().expect("valid url");
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
                .title("Dev Room")
                .inner_size(1440.0, 900.0)
                .min_inner_size(820.0, 560.0)
                .visible(!hidden)
                .build()?;

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
                    "open" => show_main(app),
                    "autostart" => set_autostart(auto_item.is_checked().unwrap_or(false)),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, ev| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                        show_main(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                updater::set_hidden(true);
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Dev Room");
}
