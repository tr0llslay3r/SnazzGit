use crate::git::rebase;

#[tauri::command]
pub async fn rebase_onto(path: String, upstream_ref: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || rebase::rebase_onto(&path, &upstream_ref))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rebase_abort(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || rebase::rebase_abort(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rebase_continue(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || rebase::rebase_continue(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn squash_commits(
    path: String,
    count: usize,
    message: String,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || rebase::squash_commits(&path, count, &message))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
