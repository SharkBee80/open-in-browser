mod client;
mod command;
mod config;
mod error;
mod store;
mod tray;
mod utils;

use client::{AppState, AuthCache};
use command::{get_config, update_config};
use config::{Config, DEFAULT_KEY, DEFAULT_PORT};
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState {
        config: Arc::new(Mutex::new(Config {
            port: DEFAULT_PORT,
            key: DEFAULT_KEY.to_string(),
        })),
        server_handle: Arc::new(Mutex::new(None)),
        auth: Arc::new(Mutex::new(AuthCache::new(DEFAULT_KEY.to_string()))),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // init persisted config
            let state = app.state::<AppState>();
            client::init_client(
                &app_handle,
                Arc::clone(&state.server_handle),
                Arc::clone(&state.auth),
            );

            // Setup tray
            tray::setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, update_config])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
