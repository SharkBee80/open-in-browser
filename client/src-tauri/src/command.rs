use crate::client::{self, AppState};
use crate::config::Config;
use tauri::State;

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Config {
    match state.config.lock() {
        Ok(lock) => lock.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

#[tauri::command]
pub async fn update_config(
    port: u16,
    key: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    client::update_config(port, key, state).await
}
