//! 应用配置存储模块
//!
//! 负责管理应用配置的持久化存储和状态管理。

pub const FAVICON_BYTES: &[u8] = include_bytes!("../icons/icon.ico");

use crate::{client::ServerHandle, error::AppError};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tokio::sync::OnceCell;

/// 配置文件名
pub const STORE_FILE: &str = "config.json";

/// 默认服务端口
pub const DEFAULT_PORT: u16 = 52798;

/// 默认认证密钥
pub const DEFAULT_KEY: &str = "open-in-browser";

/// 全局应用句柄单例
pub static APP_HANDLE: OnceCell<Arc<AppHandle>> = OnceCell::const_new();

/// 获取全局应用句柄
///
/// # 错误
/// 如果应用句柄未初始化，返回 `MissingAppHandle` 错误
pub fn app_handle() -> Result<&'static Arc<AppHandle>, AppError> {
    APP_HANDLE.get().ok_or(AppError::MissingAppHandle)
}

/// 应用配置结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// HTTP 服务器端口
    pub port: u16,
    /// JWT 认证密钥
    pub key: String,
}

/// 应用状态结构体
///
/// 在 Tauri 命令之间共享的状态
#[derive(Clone)]
pub struct AppState {
    pub(crate) config: Arc<Mutex<Config>>,
    pub(crate) server_handle: Arc<Mutex<Option<ServerHandle>>>,
}

/// 持久化配置到存储文件
///
/// # 参数
/// * `config` - 要保存的配置
///
/// # 错误
/// 如果存储文件打开或保存失败，返回错误字符串
pub fn persist_config(config: &Config) -> Result<(), String> {
    let store = app_handle()
        .map_err(|e| e.to_string())?
        .store(STORE_FILE)
        .map_err(|e| AppError::StoreOpen(e.to_string()).to_string())?;

    store.set("port", serde_json::json!(config.port));
    store.set("key", serde_json::json!(config.key));
    store
        .save()
        .map_err(|e| AppError::StoreSave(e.to_string()).to_string())?;

    Ok(())
}

/// 更新配置
///
/// # 参数
/// * `port` - 新端口
/// * `key` - 新密钥
/// * `config` - 当前配置的可变引用
///
/// # 返回
/// 返回一个布尔值，表示是否需要重启服务器
/// - `Ok(true)` - 配置已更改，需要重启服务器
/// - `Ok(false)` - 配置未更改，无需重启
/// - `Err(...)` - 持久化失败
pub async fn update(port: u16, key: String, config: Arc<Mutex<Config>>) -> Result<bool, String> {
    // 一次性获取锁，读取旧值
    let (old_port, old_key) = {
        let cfg = config.lock().unwrap();
        (cfg.port, cfg.key.clone())
    };

    // 检查是否需要更新
    let need_update = old_port != port || old_key != key;

    if !need_update {
        return Ok(false);
    }

    // 持久化新配置
    persist_config(&Config {
        port,
        key: key.clone(),
    })?;

    // 更新内存中的配置
    {
        let mut cfg = config.lock().unwrap();
        cfg.port = port;
        cfg.key = key;
    }

    Ok(true)
}
