use crate::{
    client,
    store::{update, AppState, Config},
};
use tauri::State;

/// 获取当前配置
///
/// # 参数
/// * `state` - 应用状态
///
/// # 返回
/// 当前配置的副本
#[tauri::command]
pub fn get_config(state: State<AppState>) -> Config {
    match state.config.lock() {
        Ok(lock) => lock.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// 更新配置
///
/// # 参数
/// * `port` - 新端口号
/// * `key` - 新认证密钥
/// * `state` - 应用状态
///
/// # 返回
/// * `Ok(())` - 配置更新成功
/// * `Err(...)` - 配置更新失败
#[tauri::command]
pub async fn update_config(
    port: u16,
    key: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    match update(port, key.clone(), state.config.clone()).await {
        Ok(need_restart) => {
            // 如果配置发生变化，需要重启服务器
            if need_restart {
                client::restart_server((*state).clone(), port).await;
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}
