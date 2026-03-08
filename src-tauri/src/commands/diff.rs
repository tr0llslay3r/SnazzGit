use crate::git::diff;
use crate::git::types::DiffFile;

#[tauri::command]
pub async fn get_working_diff(
    path: String,
    file_path: String,
    staged: bool,
) -> Result<DiffFile, String> {
    tokio::task::spawn_blocking(move || diff::get_working_diff(&path, &file_path, staged))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_commit_diff(
    path: String,
    commit_id: String,
    file_path: Option<String>,
) -> Result<Vec<DiffFile>, String> {
    tokio::task::spawn_blocking(move || {
        diff::get_commit_diff(&path, &commit_id, file_path.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
