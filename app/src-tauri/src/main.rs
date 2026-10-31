#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
mod updater;

fn main() {
    // If the port is taken, another Dev Room is already running: just show its UI.
    let port = server::start(server::DEFAULT_PORT).unwrap_or(server::DEFAULT_PORT);
    updater::start();
    tauri::Builder::default()
        .setup(move |app| {
            let url = format!("http://127.0.0.1:{port}/").parse().expect("valid url");
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
                .title("Dev Room")
                .inner_size(1440.0, 900.0)
                .min_inner_size(820.0, 560.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Dev Room");
}
