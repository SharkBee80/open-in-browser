//! HTTP 服务器客户端模块
//!
//! 负责启动和管理 HTTP 服务器，处理浏览器打开请求。

use crate::error::AppError;
use crate::store;
use crate::utils;
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tokio::sync::oneshot;

// 常量定义
/// 认证请求头名称
const AUTH_HEADER: &str = "x-openinbrowser-auth";

/// 服务器句柄，用于管理服务器生命周期
pub struct ServerHandle {
    /// 停止信号发送器
    pub stop_tx: oneshot::Sender<()>,
    /// 服务器任务句柄
    pub join: tokio::task::JoinHandle<()>,
}

impl ServerHandle {
    /// 停止服务器
    pub async fn stop(self) {
        let _ = self.stop_tx.send(());
        let _ = self.join.await;
    }
}

/// 启动 HTTP 服务器
///
/// # 参数
/// * `state` - 应用状态
/// * `port` - 服务器端口
///
/// # 返回
/// 服务器句柄，用于后续停止或重启
pub async fn start_server(state: store::AppState, port: u16) -> ServerHandle {
    // 配置 CORS - 允许所有来源、方法和请求头
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建路由
    let app = Router::new()
        .route("/", get(root))
        .route("/cmd", post(cmd).get(not_found))
        .route("/favicon.ico", get(favicon))
        .fallback(not_found)
        .layer(cors)
        .with_state(state);

    // 绑定监听器
    let listener = tokio::net::TcpListener::bind(format!("localhost:{}", port))
        .await
        .expect("Failed to bind TCP listener");

    // 创建停止信号通道
    let (stop_tx, stop_rx) = oneshot::channel::<()>();

    // 启动服务器任务
    let join = tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            stop_rx.await.ok();
        });
        let _ = server.await;
    });

    ServerHandle { stop_tx, join }
}

/// 重启 HTTP 服务器
///
/// # 参数
/// * `state` - 应用状态
/// * `port` - 新端口
pub async fn restart_server(state: store::AppState, port: u16) {
    // 原子性地取出旧服务器句柄
    let old_handle = {
        let mut lock = state.server_handle.lock().unwrap();
        lock.take()
    };

    // 停止旧服务器
    if let Some(handle) = old_handle {
        handle.stop().await;
    }

    // 启动新服务器
    let handle = start_server(state.clone(), port).await;

    // 原子性地设置新服务器句柄
    {
        let mut lock = state.server_handle.lock().unwrap();
        *lock = Some(handle);
    }
}

/// 根路径处理器
async fn root() -> &'static str {
    "open-in-browser is running.\n\nUse: POST /cmd with JSON.stringify body:\n[\"cmd1 arg1\",\"cmd2 arg2\"]\nAuth header: x-openinbrowser-auth\n"
}

/// Favicon 处理器
async fn favicon() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "image/x-icon".parse().unwrap());
    (headers, store::FAVICON_BYTES)
}

/// 404 未找到处理器
async fn not_found(uri: Uri) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        format!("404 Not Found: {}\n", uri.path()),
    )
}

/// 创建错误响应
///
/// # 参数
/// * `status` - HTTP 状态码
/// * `message` - 错误消息
fn create_error_response(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(json!({ "error": message.into(), "success": false })),
    )
        .into_response()
}

/// 创建 400 错误响应
pub fn bad_request(msg: impl Into<String>) -> Response {
    create_error_response(StatusCode::BAD_REQUEST, msg)
}

/// 创建 401 错误响应
fn unauthorized_response(msg: impl Into<String>) -> Response {
    create_error_response(StatusCode::UNAUTHORIZED, msg)
}

/// 创建 500 错误响应
#[allow(dead_code)]
fn internal_error_response(msg: impl Into<String>) -> Response {
    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, msg)
}

/// 命令处理器 - 执行命令（支持单条和多条命令）
async fn cmd(
    headers: HeaderMap,
    State(state): State<store::AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    // 验证认证信息
    let config = state.config.lock().unwrap();
    if let Err(msg) = verify_auth(&headers, &config.key) {
        return unauthorized_response(msg);
    }

    // 解析命令列表
    let cmd_strings: Vec<String> = if body.is_array() {
        match serde_json::from_value::<Vec<String>>(body) {
            Ok(cmds) => cmds,
            Err(e) => {
                return bad_request(format!("Invalid command array: {}", e));
            }
        }
    } else {
        return bad_request("Missing 'cmd' or 'cmds' field");
    };

    let parsed_commands = match utils::check_command(&cmd_strings) {
        Ok(cmds) => cmds,
        Err(response) => return response,
    };

    utils::execute_commands(&parsed_commands)
}

/// 验证 JWT 认证信息
///
/// # 参数
/// * `headers` - HTTP 请求头
/// * `key` - JWT 密钥
///
/// # 错误
/// 如果认证失败，返回错误消息
fn verify_auth(headers: &HeaderMap, key: &str) -> Result<(), String> {
    // 提取 token
    let token = headers
        .get(AUTH_HEADER)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::AuthMissingHeader.to_string())?;

    // 配置验证器
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = 1;

    // 解码并验证 JWT
    decode::<serde_json::Value>(
        token,
        &DecodingKey::from_secret(key.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::AuthJwtInvalid(e.to_string()).to_string())?;

    Ok(())
}

/// 初始化客户端
///
/// # 参数
/// * `app_handle` - Tauri 应用句柄
/// * `server_handle` - 服务器句柄的共享引用
pub fn init_client(app_handle: &AppHandle, server_handle: Arc<Mutex<Option<ServerHandle>>>) {
    store::APP_HANDLE.set(Arc::new(app_handle.clone())).ok();

    tauri::async_runtime::spawn(async move {
        if let Err(e) = setup_client(server_handle).await {
            eprintln!("init_client failed: {e}");
        }
    });
}

/// 设置客户端（异步）
///
/// # 参数
/// * `server_handle` - 服务器句柄的共享引用
///
/// # 错误
/// 如果初始化失败，返回错误消息
async fn setup_client(server_handle: Arc<Mutex<Option<ServerHandle>>>) -> Result<(), String> {
    use crate::store::{DEFAULT_KEY, DEFAULT_PORT, STORE_FILE};

    let store = store::app_handle()
        .map_err(|e| e.to_string())?
        .store(STORE_FILE)
        .map_err(|e| AppError::StoreOpen(e.to_string()).to_string())?;

    // 读取或创建默认配置
    let port: u16 = store
        .get("port")
        .and_then(|v| v.as_u64())
        .and_then(|v| u16::try_from(v).ok())
        .unwrap_or(DEFAULT_PORT);

    let key: String = store
        .get("key")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| DEFAULT_KEY.to_string());

    // 确保配置持久化（如果缺失则写入默认值）
    store.set("port", serde_json::json!(port));
    store.set("key", serde_json::json!(key));
    let _ = store.save();

    // 创建应用状态
    let app_state = store::AppState {
        config: Arc::new(Mutex::new(store::Config {
            port,
            key: key.clone(),
        })),
        server_handle: Arc::clone(&server_handle),
    };

    // 启动 HTTP 服务器
    let handle = start_server(app_state, port).await;
    {
        let mut lock = server_handle.lock().unwrap();
        *lock = Some(handle);
    }

    Ok(())
}
