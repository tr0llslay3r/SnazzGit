use crate::git::conflict;
use crate::git::types::ConflictFile;

#[tauri::command]
pub async fn get_conflict_diff(path: String, file_path: String) -> Result<ConflictFile, String> {
    tokio::task::spawn_blocking(move || conflict::get_conflict_diff(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_conflict_ours_theirs(
    path: String,
    file_path: String,
    use_ours: bool,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || conflict::resolve_with_stage(&path, &file_path, use_ours))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_resolved_conflict(
    path: String,
    file_path: String,
    content: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        conflict::save_resolved_conflict(&path, &file_path, &content)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
