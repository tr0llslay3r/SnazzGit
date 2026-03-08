use crate::theme::{self, Theme};

#[tauri::command]
pub async fn load_user_themes() -> Result<Vec<Theme>, String> {
    tokio::task::spawn_blocking(theme::load_user_themes)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_user_theme(theme_data: Theme) -> Result<(), String> {
    tokio::task::spawn_blocking(move || theme::save_user_theme(&theme_data))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_user_theme(name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || theme::delete_user_theme(&name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
