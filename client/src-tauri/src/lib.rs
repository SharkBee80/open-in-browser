//! Open in Browser Tauri 客户端应用
//!
//! 这是一个基于 Tauri 框架的桌面应用，提供 HTTP 服务器功能，
//! 允许通过 API 调用在指定浏览器中打开 URL。
//!
//! # 模块
//! * `client` - HTTP 服务器客户端
//! * `command` - Tauri 命令处理
//! * `error` - 错误类型定义
//! * `store` - 配置存储管理
//! * `tray` - 系统托盘管理
//! * `utils` - 实用工具函数

mod client;
mod command;
mod error;
mod store;
mod tray;
mod utils;

use command::{get_config, update_config};
use std::sync::{Arc, Mutex};
use store::{AppState, Config, DEFAULT_KEY, DEFAULT_PORT};
use tauri::Manager;

/// 运行 Tauri 应用
///
/// 这是应用的主入口点，负责：
/// 1. 初始化应用状态
/// 2. 配置 Tauri 插件
/// 3. 设置系统托盘
/// 4. 注册 Tauri 命令
/// 5. 处理窗口事件
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化应用状态（使用默认配置）
    let state = AppState {
        config: Arc::new(Mutex::new(Config {
            port: DEFAULT_PORT,
            key: DEFAULT_KEY.to_string(),
        })),
        server_handle: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        // 注册全局状态
        .manage(state)
        // 初始化插件
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        // 应用设置
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // 初始化客户端（从持久化存储加载配置并启动 HTTP 服务器）
            let state = app.state::<AppState>();
            client::init_client(&app_handle, Arc::clone(&state.server_handle));

            // 设置系统托盘
            tray::setup_tray(app)?;

            Ok(())
        })
        // 注册 Tauri 命令
        .invoke_handler(tauri::generate_handler![get_config, update_config])
        // 处理窗口事件
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // 阻止关闭，改为隐藏窗口
                let _ = window.hide();
                api.prevent_close();
            }
        })
        // 启动应用
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
