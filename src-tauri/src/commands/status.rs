use crate::git::status;
use crate::git::types::WorkingTreeStatus;

#[tauri::command]
pub async fn get_status(path: String) -> Result<WorkingTreeStatus, String> {
    tokio::task::spawn_blocking(move || status::get_status(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stage_file(path: String, file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::stage_file(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unstage_file(path: String, file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::unstage_file(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stage_all(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::stage_all(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unstage_all(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::unstage_all(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn discard_file(path: String, file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::discard_file(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_to_gitignore(path: String, pattern: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::add_to_gitignore(&path, &pattern))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
