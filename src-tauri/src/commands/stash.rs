use crate::git::stash;
use crate::git::types::StashEntry;

#[tauri::command]
pub async fn stash_list(path: String) -> Result<Vec<StashEntry>, String> {
    tokio::task::spawn_blocking(move || stash::stash_list(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stash_save(path: String, message: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || stash::stash_save(&path, &message))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stash_pop(path: String, index: usize) -> Result<(), String> {
    tokio::task::spawn_blocking(move || stash::stash_pop(&path, index))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stash_apply(path: String, index: usize) -> Result<(), String> {
    tokio::task::spawn_blocking(move || stash::stash_apply(&path, index))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stash_drop(path: String, index: usize) -> Result<(), String> {
    tokio::task::spawn_blocking(move || stash::stash_drop(&path, index))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
