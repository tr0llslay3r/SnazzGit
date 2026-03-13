use crate::git::reflog::{self, ReflogEntry};

#[tauri::command]
pub async fn get_reflog(path: String, limit: usize) -> Result<Vec<ReflogEntry>, String> {
    tokio::task::spawn_blocking(move || reflog::get_reflog(&path, limit))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
