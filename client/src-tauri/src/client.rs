use crate::config::Config;
use crate::error::AppError;
use crate::{store, utils};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;
use tiny_http::{Header, Method, Response, Server, StatusCode};
use tokio::sync::{oneshot, OnceCell};
pub struct AppState {
    pub(crate) config: Arc<Mutex<Config>>,
    pub(crate) server_handle: Arc<Mutex<Option<ServerHandle>>>,
    pub(crate) auth: Arc<Mutex<AuthCache>>,
}

pub struct ServerHandle {
    pub stop_tx: oneshot::Sender<()>,
    pub join: tokio::task::JoinHandle<()>,
}

impl ServerHandle {
    pub async fn stop(self) {
        let _ = self.stop_tx.send(());
        let _ = self.join.await;
    }
}

#[derive(Clone)]
pub(crate) struct AuthCache {
    key: String,
}

impl AuthCache {
    pub fn new(key: String) -> Self {
        Self { key }
    }

    pub fn update_key(&mut self, key: String) {
        self.key = key;
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

pub async fn start_server(auth: Arc<Mutex<AuthCache>>, port: u16) -> Result<ServerHandle, String> {
    let server = Server::http(format!("127.0.0.1:{port}")).map_err(|e| {
        AppError::HttpBind(format!("Failed to bind 127.0.0.1:{port}: {e}")).to_string()
    })?;
    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

    let join = tokio::task::spawn_blocking(move || {
        // Polling loop so we can also observe stop_rx.
        // recv_timeout keeps this thread responsive without spinning.
        loop {
            match stop_rx.try_recv() {
                Ok(()) => break,
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => break,
            }

            let req = match server.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(Some(rq)) => rq,
                Ok(None) => continue,
                Err(_) => break,
            };

            let (path, query) = split_path_and_query(req.url());
            let method = req.method().clone();

            let resp = match (method, path) {
                (Method::Options, _) => options_response(),
                (Method::Get, "/") => root_response(),
                (Method::Get, "/open") => open_response(&req, &auth, query),
                (Method::Get, "/favicon.ico") => favicon_response(),
                _ => not_found_response(path),
            };

            let _ = req.respond(with_cors(resp));
        }
    });

    Ok(ServerHandle { stop_tx, join })
}
fn split_path_and_query(url: &str) -> (&str, &str) {
    match url.split_once('?') {
        Some((p, q)) => (p, q),
        None => (url, ""),
    }
}

fn with_cors(mut resp: Response<std::io::Cursor<Vec<u8>>>) -> Response<std::io::Cursor<Vec<u8>>> {
    let cors_headers = [
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, OPTIONS"),
        (
            "Access-Control-Allow-Headers",
            "Content-Type, x-openinbrowser-auth",
        ),
        ("Access-Control-Max-Age", "86400"),
    ];

    for (name, value) in cors_headers {
        if let Ok(header) = Header::from_bytes(name.as_bytes(), value.as_bytes()) {
            resp = resp.with_header(header);
        }
    }

    resp
}

fn root_response() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(
        "open-in-browser is running.\n\nUse: GET /open?b=[browser_path]&url=[url]\nAuth header: x-openinbrowser-auth\n",
    )
    .with_status_code(StatusCode(200))
}

fn options_response() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string("").with_status_code(StatusCode(204))
}

fn favicon_response() -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes(b"Content-Type", b"image/x-icon").expect("valid header");
    Response::from_data(store::FAVICON_BYTES.to_vec()).with_header(header)
}

fn not_found_response(path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(format!("404 Not Found: {path}\n")).with_status_code(StatusCode(404))
}

fn bad_request_response(msg: impl Into<String>) -> Response<std::io::Cursor<Vec<u8>>> {
    utils::json_response(&serde_json::json!({ "error": msg.into() }), StatusCode(400))
}

fn unauthorized_response(msg: impl Into<String>) -> Response<std::io::Cursor<Vec<u8>>> {
    utils::json_response(&serde_json::json!({ "error": msg.into() }), StatusCode(401))
}

fn internal_error_response(msg: impl Into<String>) -> Response<std::io::Cursor<Vec<u8>>> {
    utils::json_response(&serde_json::json!({ "error": msg.into() }), StatusCode(500))
}

fn open_response(
    req: &tiny_http::Request,
    auth: &Arc<Mutex<AuthCache>>,
    query: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    if let Err(msg) = verify_auth(req, auth) {
        return unauthorized_response(msg);
    }

    let params = parse_query_params(query);
    let browser = match params.get("b") {
        Some(b) if !b.is_empty() => b,
        _ => return bad_request_response("Missing 'b' (browser path) parameter\n"),
    };

    let url = match params.get("url") {
        Some(u) if !u.is_empty() => u,
        _ => return bad_request_response("Missing 'url' parameter\n"),
    };

    let args = match params.get("args") {
        Some(a) if !a.is_empty() => a,
        _ => "",
    };

    match open_with_browser(browser, url, args) {
        Ok(_) => Response::from_string(format!("Success: Opening {url} with {browser}\n"))
            .with_status_code(StatusCode(200)),
        Err(e) => internal_error_response(format!("Error: {e}\n")),
    }
}

fn parse_query_params(query: &str) -> HashMap<String, String> {
    // Minimal parser:
    // - splits on '&'
    // - key/value split on first '='
    // - percent-decodes (+ treated as space)
    let mut out = HashMap::new();
    if query.is_empty() {
        return out;
    }
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        let k = percent_decode_www_form(k);
        if k.is_empty() {
            continue;
        }
        let v = percent_decode_www_form(v);
        out.insert(k, v);
    }
    out
}

fn percent_decode_www_form(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let h1 = from_hex(bytes[i + 1]);
                let h2 = from_hex(bytes[i + 2]);
                if let (Some(a), Some(b)) = (h1, h2) {
                    out.push((a << 4) | b);
                    i += 3;
                } else {
                    // Invalid escape, keep literal '%'
                    out.push(b'%');
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn open_with_browser(browser: &str, url: &str, args: &str) -> std::io::Result<()> {
    let mut command = Command::new(browser);
    command.arg(url);

    if !args.trim().is_empty() {
        let extra_args = shell_words::split(args)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        command.args(&extra_args);
        // 调试：打印实际命令（仅开发用）
        // println!("Executing: {} {} {:?}", browser, url, extra_args);
    }

    command.spawn()?;
    Ok(())
}

fn verify_auth(req: &tiny_http::Request, auth: &Arc<Mutex<AuthCache>>) -> Result<(), String> {
    // Header carries JWT token signed by shared key.
    const HDR: &str = "x-openinbrowser-auth";

    let token = req
        .headers()
        .iter()
        .find(|h| h.field.equiv(HDR))
        .map(|h| h.value.as_str())
        .ok_or_else(|| AppError::AuthMissingHeader.to_string())?;

    let key = {
        let lock = auth
            .lock()
            .map_err(|_| AppError::ConfigLockPoisoned.to_string())?;
        lock.key().to_string()
    };

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = 1;

    decode::<serde_json::Value>(
        token,
        &DecodingKey::from_secret(key.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::AuthJwtInvalid(e.to_string()).to_string())?;

    Ok(())
}

static APP_HANDLE: OnceCell<Arc<AppHandle>> = OnceCell::const_new();
fn app_handle() -> Result<&'static Arc<AppHandle>, AppError> {
    APP_HANDLE.get().ok_or(AppError::MissingAppHandle)
}
pub fn init_client(
    app_handle: &AppHandle,
    server_handle: Arc<Mutex<Option<ServerHandle>>>,
    auth: Arc<Mutex<AuthCache>>,
) {
    APP_HANDLE.set(Arc::new(app_handle.clone())).ok();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = setup_client(server_handle, auth).await {
            eprintln!("init_client failed: {e}");
        }
    });
}

async fn setup_client(
    server_handle: Arc<Mutex<Option<ServerHandle>>>,
    auth: Arc<Mutex<AuthCache>>,
) -> Result<(), String> {
    use crate::config::{DEFAULT_KEY, DEFAULT_PORT, STORE_FILE};
    let store = app_handle()
        .map_err(|e| e.to_string())?
        .store(STORE_FILE)
        .map_err(|e| AppError::StoreOpen(e.to_string()).to_string())?;

    let port: u16 = store
        .get("port")
        .and_then(|v| v.as_u64())
        .and_then(|v| u16::try_from(v).ok())
        .unwrap_or(DEFAULT_PORT);

    let key: String = store
        .get("key")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| DEFAULT_KEY.to_string());

    // If missing/invalid, write fallback so next launch is stable
    store.set("port", serde_json::json!(port));
    store.set("key", serde_json::json!(key));
    let _ = store.save();

    {
        let mut lock = auth
            .lock()
            .map_err(|_| AppError::ConfigLockPoisoned.to_string())?;
        lock.update_key(key.clone());
    }

    // Start HTTP server and keep handle for restarts.
    let handle = start_server(Arc::clone(&auth), port).await?;
    {
        let mut lock = server_handle.lock().unwrap();
        *lock = Some(handle);
    }
    Ok(())
}

pub async fn update_config(
    port: u16,
    key: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let old_port = { state.config.lock().unwrap().port };

    persist_config(&Config {
        port,
        key: key.clone(),
    })?;
    // 1. Update config
    {
        let mut config = state.config.lock().unwrap();
        config.port = port;
        config.key = key;
    }

    {
        if let (Ok(cfg), Ok(mut auth)) = (state.config.lock(), state.auth.lock()) {
            auth.update_key(cfg.key.clone());
        }
    }

    // Restart server only when port changed. Key updates are hot (read from shared Config).
    if old_port != port {
        // (port changed) stop old server and start new one
        let old_handle = { state.server_handle.lock().unwrap().take() };
        if let Some(handle) = old_handle {
            handle.stop().await;
        }

        let new_handle = start_server(Arc::clone(&state.auth), port).await?;
        {
            let mut handle_lock = state.server_handle.lock().unwrap();
            *handle_lock = Some(new_handle);
        }
    }

    Ok(())
}

pub fn persist_config(config: &Config) -> Result<(), String> {
    use crate::config::STORE_FILE;

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
